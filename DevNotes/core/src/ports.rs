//! Порты (traits) для инъекции зависимостей, дающие детерминизм в тестах.
//!
//! ПОЧЕМУ: время и генерация идентификаторов — недетерминированные источники.
//! Вынеся их в порты, мы можем в тестах подменять их фиксированными реализациями
//! (см. `tests/integration.rs`), а в проде использовать системные.

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

/// Источник текущего времени в формате UTC RFC3339.
pub trait Clock: Send + Sync {
    fn now_rfc3339(&self) -> String;
}

/// Источник новых идентификаторов сущностей.
pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> String;
}

/// Системные часы: текущее время UTC.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        // unwrap безопасен: OffsetDateTime всегда форматируется в RFC3339.
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .expect("RFC3339 formatting cannot fail for a valid OffsetDateTime")
    }
}

/// Генератор UUID v7 (сортируемый по времени, пригоден для офлайн-создания).
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> String {
        Uuid::now_v7().to_string()
    }
}
