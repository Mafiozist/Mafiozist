//! Разрешение конфликтов синхронизации — чистая логика (без БД и сети).
//!
//! Стратегия по умолчанию — LWW (last-write-wins) по `updated_at` на уровне блока.
//! ПОЧЕМУ важна конфликт-копия: при РАВНОМ времени и РАЗНОМ содержимом «тихо»
//! терять чужую версию недопустимо — создаётся конфликт-копия для ручного слияния.
//! См. `docs/09-YANDEX-DISK.md`.

use std::sync::{Arc, Mutex};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::snapshot::Snapshot;
use crate::sqlite::SqliteStore;
use crate::{CoreError, Result};

/// Исход сравнения локальной и удалённой версий одной записи.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Локальная версия новее — оставить её.
    TakeLocal,
    /// Удалённая версия новее — принять её.
    TakeRemote,
    /// Время совпадает, содержимое различается — оставить локальную и завести конфликт-копию.
    Conflict,
}

/// Разрешает конфликт двух версий записи по их `updated_at` (UTC RFC3339)
/// и признаку равенства содержимого.
///
/// Некорректные метки времени трактуются как «эпоха» (минимально старые),
/// чтобы валидная сторона всегда выигрывала.
pub fn resolve(local_updated_at: &str, remote_updated_at: &str, content_equal: bool) -> Resolution {
    let local = parse(local_updated_at);
    let remote = parse(remote_updated_at);

    match remote.cmp(&local) {
        std::cmp::Ordering::Greater => Resolution::TakeRemote,
        std::cmp::Ordering::Less => Resolution::TakeLocal,
        std::cmp::Ordering::Equal => {
            if content_equal {
                Resolution::TakeLocal
            } else {
                Resolution::Conflict
            }
        }
    }
}

/// Парсит RFC3339 в `OffsetDateTime`; при ошибке — `UNIX_EPOCH`.
fn parse(value: &str) -> OffsetDateTime {
    OffsetDateTime::parse(value, &Rfc3339).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

// ---------------------------------------------------------------------------
// Транспорт синхронизации и движок обмена снапшотами.
// ---------------------------------------------------------------------------

/// Абстракция канала обмена снапшотом (файл, облако и т.п.).
///
/// ПОЧЕМУ trait: движок синхронизации не зависит от конкретного хранилища —
/// его реализуют [`FileTransport`] (локальный файл, в т.ч. в папке Яндекс.Диска),
/// [`InMemoryTransport`] (тесты), а в будущем — REST-адаптер Яндекс.Диска
/// (OAuth 2.0 + PKCE, загрузка/выгрузка файла в app folder — см. `docs/09-YANDEX-DISK.md`).
pub trait SyncTransport {
    /// Скачивает удалённый снапшот (JSON). `None`, если его ещё нет.
    fn download(&self) -> Result<Option<String>>;
    /// Загружает снапшот (JSON), перезаписывая удалённый.
    fn upload(&self, data: &str) -> Result<()>;
}

/// Итог синхронизации.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SyncReport {
    /// Сколько изменений применено из удалённого снапшота.
    pub applied: usize,
    /// Размер выгруженного снапшота в байтах.
    pub uploaded_bytes: usize,
}

/// Двусторонняя синхронизация: скачать удалённый снапшот → слить (LWW) →
/// выгрузить объединённый. Так оба устройства сходятся к одному состоянию.
pub fn sync(store: &SqliteStore, transport: &dyn SyncTransport) -> Result<SyncReport> {
    let applied = match transport.download()? {
        Some(remote) => {
            let snap: Snapshot = serde_json::from_str(&remote)
                .map_err(|e| CoreError::Invalid(format!("некорректный снапшот: {e}")))?;
            store.apply_snapshot(&snap)?
        }
        None => 0,
    };

    let merged = store.export_snapshot()?;
    let data = serde_json::to_string(&merged)
        .map_err(|e| CoreError::Invalid(format!("сериализация снапшота: {e}")))?;
    transport.upload(&data)?;

    Ok(SyncReport {
        applied,
        uploaded_bytes: data.len(),
    })
}

/// Транспорт поверх локального файла. Практичный путь для v0.3: если указать файл
/// внутри синхронизируемой папки Яндекс.Диска (десктоп-клиент), получаем обмен между
/// устройствами без собственного REST-клиента.
pub struct FileTransport {
    path: std::path::PathBuf,
}

impl FileTransport {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl SyncTransport for FileTransport {
    fn download(&self) -> Result<Option<String>> {
        match std::fs::read_to_string(&self.path) {
            Ok(s) => Ok(Some(s)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CoreError::Invalid(format!("чтение снапшота: {e}"))),
        }
    }
    fn upload(&self, data: &str) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::Invalid(format!("создание каталога: {e}")))?;
        }
        std::fs::write(&self.path, data)
            .map_err(|e| CoreError::Invalid(format!("запись снапшота: {e}")))
    }
}

/// In-memory транспорт (для тестов и обмена между двумя store в одном процессе).
#[derive(Clone, Default)]
pub struct InMemoryTransport {
    slot: Arc<Mutex<Option<String>>>,
}

impl InMemoryTransport {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SyncTransport for InMemoryTransport {
    fn download(&self) -> Result<Option<String>> {
        Ok(self.slot.lock().expect("mutex").clone())
    }
    fn upload(&self, data: &str) -> Result<()> {
        *self.slot.lock().expect("mutex") = Some(data.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_remote_wins() {
        assert_eq!(
            resolve("2026-07-17T10:00:00Z", "2026-07-17T10:00:05Z", false),
            Resolution::TakeRemote
        );
    }

    #[test]
    fn newer_local_wins() {
        assert_eq!(
            resolve("2026-07-17T10:00:05Z", "2026-07-17T10:00:00Z", false),
            Resolution::TakeLocal
        );
    }

    #[test]
    fn equal_time_same_content_keeps_local() {
        assert_eq!(
            resolve("2026-07-17T10:00:00Z", "2026-07-17T10:00:00Z", true),
            Resolution::TakeLocal
        );
    }

    #[test]
    fn equal_time_diff_content_conflicts() {
        assert_eq!(
            resolve("2026-07-17T10:00:00Z", "2026-07-17T10:00:00Z", false),
            Resolution::Conflict
        );
    }

    #[test]
    fn invalid_timestamp_loses_to_valid() {
        // Битая локальная метка → удалённая валидная считается новее.
        assert_eq!(
            resolve("not-a-date", "2026-07-17T10:00:00Z", false),
            Resolution::TakeRemote
        );
    }
}
