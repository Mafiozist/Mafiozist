//! Разрешение конфликтов синхронизации — чистая логика (без БД и сети).
//!
//! Стратегия по умолчанию — LWW (last-write-wins) по `updated_at` на уровне блока.
//! ПОЧЕМУ важна конфликт-копия: при РАВНОМ времени и РАЗНОМ содержимом «тихо»
//! терять чужую версию недопустимо — создаётся конфликт-копия для ручного слияния.
//! См. `docs/09-YANDEX-DISK.md`.

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

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
        assert_eq!(resolve("not-a-date", "2026-07-17T10:00:00Z", false), Resolution::TakeRemote);
    }
}
