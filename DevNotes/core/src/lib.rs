//! # devnotes-core
//!
//! Переносимое ядро приложения DEVNOTES (см. `docs/04-ARCHITECTURE.md`).
//! Слои Clean Architecture в одном крейте, без зависимости от Tauri/WebKit:
//!
//! - [`domain`]   — сущности и типы предметной области;
//! - [`ports`]    — порты (traits) для инъекции времени и идентификаторов;
//! - [`search`]   — построение безопасного FTS5-запроса из пользовательского ввода;
//! - [`sync`]     — разрешение конфликтов синхронизации (LWW) — чистая логика;
//! - [`sqlite`]   — инфраструктура: локальное хранилище SQLite + FTS5 и репозитории;
//! - [`service`]  — сценарии (use-cases) поверх хранилища.
//!
//! ПОЧЕМУ ядро вынесено в отдельный крейт: чтобы всю бизнес-логику и работу с БД
//! можно было компилировать и тестировать без GUI-зависимостей (CI без WebKit).

pub mod domain;
pub mod ports;
pub mod search;
pub mod service;
pub mod snapshot;
pub mod sqlite;
pub mod sync;

/// Единый тип ошибки ядра.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Ошибка нижележащего SQLite.
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    /// Запрошенная сущность не найдена.
    #[error("not found")]
    NotFound,
    /// Некорректный ввод (нарушение инварианта до обращения к БД).
    #[error("invalid input: {0}")]
    Invalid(String),
}

/// Результат операций ядра.
pub type Result<T> = std::result::Result<T, CoreError>;
