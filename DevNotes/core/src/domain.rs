//! Доменные сущности DEVNOTES.
//!
//! Перенято из проекта Portfolio (иерархия Project → NoteSeries → NoteContent,
//! теги TechTag). Все временные метки — строки UTC RFC3339; идентификаторы — UUID v7
//! (строки). См. `docs/05-DATA-MODEL.md` и глоссарий `docs/12-GLOSSARY.md`.

use serde::{Deserialize, Serialize};

/// Допустимые типы блока контента.
///
/// Хранится в БД как TEXT; здесь — константы, чтобы не плодить «магические строки».
pub mod content_type {
    pub const MARKDOWN: &str = "markdown";
    pub const CODE: &str = "code";
    pub const IMAGE: &str = "image";
    pub const LINK: &str = "link";

    /// Полный набор допустимых типов (для валидации ввода в use-cases).
    pub const ALL: [&str; 4] = [MARKDOWN, CODE, IMAGE, LINK];

    /// Проверка, что строка — допустимый тип блока.
    pub fn is_valid(value: &str) -> bool {
        ALL.contains(&value)
    }
}

/// Проект — верхнеуровневая группа заметок.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Серия (тема) заметок внутри проекта.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteSeries {
    pub id: String,
    /// Проект-владелец; `None` — «входящие» без проекта.
    pub project_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Блок контента — единица заметки внутри серии.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteContent {
    pub id: String,
    pub series_id: String,
    /// Порядок отображения (drag-and-drop). Чем меньше — тем выше.
    pub sort_order: i64,
    pub title: Option<String>,
    pub text: String,
    /// Тип блока: см. [`content_type`].
    #[serde(rename = "type")]
    pub content_type: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Категория тега технологий (язык / фреймворк / инструмент …).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechTagType {
    pub id: String,
    /// Название категории. В БД колонка называется `type`.
    #[serde(rename = "type")]
    pub type_name: String,
}

/// Тег технологии, которым помечаются серии заметок.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechTag {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// Ссылка на категорию (может отсутствовать).
    pub type_id: Option<String>,
    /// Название категории (join из `tech_tag_type`), удобно для UI.
    #[serde(rename = "typeName")]
    pub type_name: Option<String>,
}

/// Результат полнотекстового поиска (одно совпадение — блок контента).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub content_id: String,
    pub series_id: String,
    pub title: Option<String>,
    /// Сниппет с подсветкой совпадения (маркеры из FTS5 `snippet()`).
    pub snippet: String,
    /// Оценка релевантности bm25 (меньше — релевантнее).
    pub rank: f64,
}
