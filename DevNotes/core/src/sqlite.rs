//! Инфраструктура: локальное хранилище SQLite + FTS5 и репозитории.
//!
//! Единственный источник правды на устройстве — локальный файл SQLite (local-first,
//! см. `docs/04-ARCHITECTURE.md`). Здесь: применение миграций, CRUD иерархии
//! Project → NoteSeries → NoteContent и полнотекстовый поиск через `note_fts`.
//!
//! ПОЧЕМУ время и id инъектируются: детерминизм тестов (см. `ports`).

use rusqlite::{params, Connection};

use crate::domain::{NoteContent, NoteSeries, Project, SearchHit};
use crate::ports::{Clock, IdGenerator};
use crate::{CoreError, Result};

/// SQL первой миграции (встраивается в бинарник).
const MIGRATION_001: &str = include_str!("../migrations/001_init.sql");

/// Хранилище поверх одного соединения SQLite.
pub struct SqliteStore {
    conn: Connection,
    clock: Box<dyn Clock>,
    ids: Box<dyn IdGenerator>,
}

impl SqliteStore {
    /// Создаёт хранилище на готовом соединении и включает контроль внешних ключей.
    pub fn new(conn: Connection, clock: Box<dyn Clock>, ids: Box<dyn IdGenerator>) -> Result<Self> {
        // Внешние ключи в SQLite по умолчанию выключены — включаем для каскадов.
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn, clock, ids })
    }

    /// Открывает файловую БД по пути.
    pub fn open(path: &str, clock: Box<dyn Clock>, ids: Box<dyn IdGenerator>) -> Result<Self> {
        Self::new(Connection::open(path)?, clock, ids)
    }

    /// Открывает БД в памяти (для тестов и временных сессий).
    pub fn open_in_memory(clock: Box<dyn Clock>, ids: Box<dyn IdGenerator>) -> Result<Self> {
        Self::new(Connection::open_in_memory()?, clock, ids)
    }

    /// Применяет миграции схемы. Идемпотентность обеспечивается тем, что миграция
    /// выполняется на свежей БД; версионирование появится с миграцией 002+.
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(MIGRATION_001)?;
        Ok(())
    }

    // --- Проекты -----------------------------------------------------------

    /// Создаёт проект и возвращает его.
    pub fn create_project(
        &self,
        name: &str,
        short_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Project> {
        let now = self.clock.now_rfc3339();
        let project = Project {
            id: self.ids.new_id(),
            name: name.to_string(),
            short_name: short_name.map(str::to_string),
            description: description.map(str::to_string),
            created_at: now.clone(),
            updated_at: now,
        };
        self.conn.execute(
            "INSERT INTO project (id, name, short_name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project.id,
                project.name,
                project.short_name,
                project.description,
                project.created_at,
                project.updated_at
            ],
        )?;
        Ok(project)
    }

    /// Возвращает все проекты, новые — первыми.
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, short_name, description, created_at, updated_at
             FROM project ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                short_name: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Удаляет проект. Каскадно удаляются его серии и их блоки (FK ON DELETE CASCADE).
    pub fn delete_project(&self, id: &str) -> Result<()> {
        let affected = self
            .conn
            .execute("DELETE FROM project WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    // --- Серии заметок -----------------------------------------------------

    /// Создаёт серию (тему) заметок, опционально привязанную к проекту.
    pub fn create_series(
        &self,
        project_id: Option<&str>,
        title: &str,
        description: Option<&str>,
    ) -> Result<NoteSeries> {
        let now = self.clock.now_rfc3339();
        let series = NoteSeries {
            id: self.ids.new_id(),
            project_id: project_id.map(str::to_string),
            title: title.to_string(),
            description: description.map(str::to_string),
            created_at: now.clone(),
            updated_at: now,
        };
        self.conn.execute(
            "INSERT INTO note_series (id, project_id, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                series.id,
                series.project_id,
                series.title,
                series.description,
                series.created_at,
                series.updated_at
            ],
        )?;
        Ok(series)
    }

    /// Возвращает серии проекта (или все «входящие» при `project_id = None`).
    pub fn list_series(&self, project_id: Option<&str>) -> Result<Vec<NoteSeries>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, title, description, created_at, updated_at
             FROM note_series
             WHERE (?1 IS NULL AND project_id IS NULL) OR project_id = ?1
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(NoteSeries {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Удаляет серию. Каскадно удаляются её блоки (FK ON DELETE CASCADE).
    pub fn delete_series(&self, id: &str) -> Result<()> {
        let affected = self
            .conn
            .execute("DELETE FROM note_series WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    // --- Блоки контента ----------------------------------------------------

    /// Добавляет блок в конец серии (`sort_order = max + 1`).
    pub fn add_content(
        &self,
        series_id: &str,
        title: Option<&str>,
        text: &str,
        content_type: &str,
    ) -> Result<NoteContent> {
        let next_order: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM note_content WHERE series_id = ?1",
            params![series_id],
            |row| row.get(0),
        )?;
        let now = self.clock.now_rfc3339();
        let content = NoteContent {
            id: self.ids.new_id(),
            series_id: series_id.to_string(),
            sort_order: next_order,
            title: title.map(str::to_string),
            text: text.to_string(),
            content_type: content_type.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };
        self.conn.execute(
            "INSERT INTO note_content (id, series_id, sort_order, title, text, type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                content.id,
                content.series_id,
                content.sort_order,
                content.title,
                content.text,
                content.content_type,
                content.created_at,
                content.updated_at
            ],
        )?;
        Ok(content)
    }

    /// Возвращает блоки серии в порядке отображения.
    pub fn list_content(&self, series_id: &str) -> Result<Vec<NoteContent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, series_id, sort_order, title, text, type, created_at, updated_at
             FROM note_content WHERE series_id = ?1
             ORDER BY sort_order ASC",
        )?;
        let rows = stmt.query_map(params![series_id], |row| {
            Ok(NoteContent {
                id: row.get(0)?,
                series_id: row.get(1)?,
                sort_order: row.get(2)?,
                title: row.get(3)?,
                text: row.get(4)?,
                content_type: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Обновляет содержимое блока и метку `updated_at`. Триггеры FTS5 переиндексируют.
    pub fn update_content(
        &self,
        id: &str,
        title: Option<&str>,
        text: &str,
        content_type: &str,
    ) -> Result<()> {
        let now = self.clock.now_rfc3339();
        let affected = self.conn.execute(
            "UPDATE note_content
             SET title = ?2, text = ?3, type = ?4, updated_at = ?5
             WHERE id = ?1",
            params![id, title, text, content_type, now],
        )?;
        if affected == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    /// Удаляет блок (индекс FTS5 обновляется триггером удаления).
    pub fn delete_content(&self, id: &str) -> Result<()> {
        let affected = self
            .conn
            .execute("DELETE FROM note_content WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    /// Переупорядочивает блоки серии согласно переданному порядку id.
    /// `sort_order` каждого блока = его позиция в `ordered_ids`.
    pub fn reorder_content(&self, ordered_ids: &[&str]) -> Result<()> {
        // unchecked_transaction позволяет транзакцию при общем `&self`.
        let tx = self.conn.unchecked_transaction()?;
        for (index, id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE note_content SET sort_order = ?2 WHERE id = ?1",
                params![id, index as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // --- Поиск -------------------------------------------------------------

    /// Полнотекстовый поиск по блокам (FTS5 + bm25). `raw` — «сырой» пользовательский
    /// ввод; преобразуется в безопасный запрос через [`crate::search::to_fts_query`].
    /// Пустой/бессмысленный ввод даёт пустой результат.
    pub fn search(&self, raw: &str, limit: i64) -> Result<Vec<SearchHit>> {
        let query = match crate::search::to_fts_query(raw) {
            Some(q) => q,
            None => return Ok(Vec::new()),
        };
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.series_id, c.title,
                    snippet(note_fts, 1, '[', ']', '…', 12) AS snip,
                    bm25(note_fts, 5.0, 1.0) AS rank
             FROM note_fts
             JOIN note_content c ON c.rowid = note_fts.rowid
             WHERE note_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit], |row| {
            Ok(SearchHit {
                content_id: row.get(0)?,
                series_id: row.get(1)?,
                title: row.get(2)?,
                snippet: row.get(3)?,
                rank: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
