//! Сценарии (use-cases) поверх хранилища.
//!
//! ПОЧЕМУ отдельный слой: здесь живут инварианты уровня приложения (валидация ввода,
//! допустимость типа блока) — до обращения к БД. IPC-слой (src-tauri) вызывает
//! именно эти функции, а не хранилище напрямую. См. `docs/04-ARCHITECTURE.md`.

use crate::domain::{
    content_type, NoteContent, NoteSeries, Project, SearchHit, TechTag, TechTagType,
};
use crate::sqlite::SqliteStore;
use crate::{CoreError, Result};

/// Сервис приложения: тонкая обёртка над [`SqliteStore`] с валидацией.
pub struct NotesService {
    store: SqliteStore,
}

impl NotesService {
    pub fn new(store: SqliteStore) -> Self {
        Self { store }
    }

    /// Доступ к нижележащему хранилищу (для операций без доп. правил).
    pub fn store(&self) -> &SqliteStore {
        &self.store
    }

    /// Создаёт проект. Имя не может быть пустым.
    pub fn create_project(
        &self,
        name: &str,
        short_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Project> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::Invalid(
                "имя проекта не может быть пустым".into(),
            ));
        }
        self.store.create_project(name, short_name, description)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        self.store.list_projects()
    }

    /// Удаляет проект вместе с его сериями и блоками (каскад).
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.store.delete_project(id)
    }

    /// Создаёт серию заметок. Заголовок обязателен.
    pub fn create_series(
        &self,
        project_id: Option<&str>,
        title: &str,
        description: Option<&str>,
    ) -> Result<NoteSeries> {
        let title = title.trim();
        if title.is_empty() {
            return Err(CoreError::Invalid(
                "заголовок серии не может быть пустым".into(),
            ));
        }
        self.store.create_series(project_id, title, description)
    }

    pub fn list_series(&self, project_id: Option<&str>) -> Result<Vec<NoteSeries>> {
        self.store.list_series(project_id)
    }

    /// Удаляет серию вместе с её блоками (каскад).
    pub fn delete_series(&self, id: &str) -> Result<()> {
        self.store.delete_series(id)
    }

    /// Добавляет блок контента. Тип блока валидируется по белому списку.
    pub fn add_content(
        &self,
        series_id: &str,
        title: Option<&str>,
        text: &str,
        content_type_value: &str,
    ) -> Result<NoteContent> {
        if !content_type::is_valid(content_type_value) {
            return Err(CoreError::Invalid(format!(
                "недопустимый тип блока: {content_type_value}"
            )));
        }
        self.store
            .add_content(series_id, title, text, content_type_value)
    }

    pub fn list_content(&self, series_id: &str) -> Result<Vec<NoteContent>> {
        self.store.list_content(series_id)
    }

    /// Обновляет блок. Тип валидируется так же, как при добавлении.
    pub fn update_content(
        &self,
        id: &str,
        title: Option<&str>,
        text: &str,
        content_type_value: &str,
    ) -> Result<()> {
        if !content_type::is_valid(content_type_value) {
            return Err(CoreError::Invalid(format!(
                "недопустимый тип блока: {content_type_value}"
            )));
        }
        self.store
            .update_content(id, title, text, content_type_value)
    }

    pub fn delete_content(&self, id: &str) -> Result<()> {
        self.store.delete_content(id)
    }

    pub fn reorder_content(&self, ordered_ids: &[&str]) -> Result<()> {
        self.store.reorder_content(ordered_ids)
    }

    // --- Теги технологий ---------------------------------------------------

    /// Создаёт категорию тегов. Имя обязательно.
    pub fn create_tag_type(&self, name: &str) -> Result<TechTagType> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::Invalid(
                "имя категории не может быть пустым".into(),
            ));
        }
        self.store.create_tag_type(name)
    }

    pub fn list_tag_types(&self) -> Result<Vec<TechTagType>> {
        self.store.list_tag_types()
    }

    /// Создаёт тег технологии. Имя обязательно.
    pub fn create_tag(
        &self,
        name: &str,
        description: Option<&str>,
        type_id: Option<&str>,
    ) -> Result<TechTag> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::Invalid("имя тега не может быть пустым".into()));
        }
        self.store.create_tag(name, description, type_id)
    }

    pub fn list_tags(&self) -> Result<Vec<TechTag>> {
        self.store.list_tags()
    }

    pub fn delete_tag(&self, id: &str) -> Result<()> {
        self.store.delete_tag(id)
    }

    pub fn list_tags_for_series(&self, series_id: &str) -> Result<Vec<TechTag>> {
        self.store.list_tags_for_series(series_id)
    }

    /// Заменяет набор тегов серии переданным списком.
    pub fn set_series_tags(&self, series_id: &str, tag_ids: &[&str]) -> Result<()> {
        self.store.set_series_tags(series_id, tag_ids)
    }

    // --- Поиск -------------------------------------------------------------

    /// Полнотекстовый поиск по всей БД с опциональным фильтром по тегам технологий.
    /// `limit` ограничивает число результатов.
    pub fn search(&self, raw: &str, tag_ids: &[&str], limit: i64) -> Result<Vec<SearchHit>> {
        self.store.search(raw, tag_ids, limit.clamp(1, 500))
    }
}
