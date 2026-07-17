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
        // ПОЧЕМУ обнуляем наносекунды: RFC3339 с переменной дробной частью ломает
        // лексикографическое сравнение updated_at, на котором построен LWW в SQL
        // (см. sqlite::apply_snapshot). Точность до секунды даёт фиксированную ширину
        // "…:SSZ", где строковый порядок == хронологический.
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("0 is a valid nanosecond")
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
