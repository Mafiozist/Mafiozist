//! Инфраструктура: локальное хранилище SQLite + FTS5 и репозитории.
//!
//! Единственный источник правды на устройстве — локальный файл SQLite (local-first,
//! см. `docs/04-ARCHITECTURE.md`). Здесь: применение миграций, CRUD иерархии
//! Project → NoteSeries → NoteContent и полнотекстовый поиск через `note_fts`.
//!
//! ПОЧЕМУ время и id инъектируются: детерминизм тестов (см. `ports`).

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};

use crate::domain::{NoteContent, NoteSeries, Project, SearchHit, TechTag, TechTagType};
use crate::ports::{Clock, IdGenerator};
use crate::{CoreError, Result};

/// Строит строку placeholder'ов `?,?,…` для `IN (...)` c `n` элементами.
fn placeholders(n: usize) -> String {
    vec!["?"; n].join(",")
}

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

    // --- Теги технологий ---------------------------------------------------

    /// Создаёт категорию тегов (язык / фреймворк / инструмент …).
    pub fn create_tag_type(&self, name: &str) -> Result<TechTagType> {
        let tag_type = TechTagType {
            id: self.ids.new_id(),
            type_name: name.to_string(),
        };
        self.conn.execute(
            "INSERT INTO tech_tag_type (id, type) VALUES (?1, ?2)",
            params![tag_type.id, tag_type.type_name],
        )?;
        Ok(tag_type)
    }

    /// Возвращает все категории тегов.
    pub fn list_tag_types(&self) -> Result<Vec<TechTagType>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, type FROM tech_tag_type ORDER BY type")?;
        let rows = stmt.query_map([], |row| {
            Ok(TechTagType {
                id: row.get(0)?,
                type_name: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Создаёт тег технологии, опционально в категории.
    pub fn create_tag(
        &self,
        name: &str,
        description: Option<&str>,
        type_id: Option<&str>,
    ) -> Result<TechTag> {
        let id = self.ids.new_id();
        self.conn.execute(
            "INSERT INTO tech_tag (id, name, description, type_id) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, description, type_id],
        )?;
        // Возвращаем с подтянутым именем категории.
        self.get_tag(&id)
    }

    /// Возвращает один тег по id (с именем категории).
    fn get_tag(&self, id: &str) -> Result<TechTag> {
        self.conn
            .query_row(
                "SELECT t.id, t.name, t.description, t.type_id, tt.type
                 FROM tech_tag t LEFT JOIN tech_tag_type tt ON tt.id = t.type_id
                 WHERE t.id = ?1",
                params![id],
                map_tag,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => CoreError::NotFound,
                other => CoreError::Db(other),
            })
    }

    /// Возвращает все теги (с именами категорий), по алфавиту.
    pub fn list_tags(&self) -> Result<Vec<TechTag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.description, t.type_id, tt.type
             FROM tech_tag t LEFT JOIN tech_tag_type tt ON tt.id = t.type_id
             ORDER BY t.name",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Удаляет тег (каскадно снимаются его привязки к сериям).
    pub fn delete_tag(&self, id: &str) -> Result<()> {
        let affected = self
            .conn
            .execute("DELETE FROM tech_tag WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    /// Возвращает теги, привязанные к серии.
    pub fn list_tags_for_series(&self, series_id: &str) -> Result<Vec<TechTag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.description, t.type_id, tt.type
             FROM series_tag st
             JOIN tech_tag t ON t.id = st.tag_id
             LEFT JOIN tech_tag_type tt ON tt.id = t.type_id
             WHERE st.series_id = ?1
             ORDER BY t.name",
        )?;
        let rows = stmt.query_map(params![series_id], map_tag)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Полностью заменяет набор тегов серии переданным списком.
    /// ПОЧЕМУ replace, а не add/remove: UI-переключатель отдаёт итоговый набор —
    /// так проще держать состояние согласованным.
    pub fn set_series_tags(&self, series_id: &str, tag_ids: &[&str]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM series_tag WHERE series_id = ?1",
            params![series_id],
        )?;
        for tag_id in tag_ids {
            tx.execute(
                "INSERT OR IGNORE INTO series_tag (series_id, tag_id) VALUES (?1, ?2)",
                params![series_id, tag_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // --- Поиск -------------------------------------------------------------

    /// Полнотекстовый поиск по блокам (FTS5 + bm25) с опциональным фильтром по тегам.
    ///
    /// - `raw` — «сырой» пользовательский ввод (см. [`crate::search::to_fts_query`]);
    /// - `tag_ids` — если непусто, результат ограничивается блоками серий, у которых
    ///   есть ВСЕ выбранные теги (AND-семантика: больше тегов → уже выборка);
    /// - при пустом тексте, но заданных тегах, возвращаются блоки таких серий без FTS;
    /// - пустой текст и пустые теги → пустой результат.
    pub fn search(&self, raw: &str, tag_ids: &[&str], limit: i64) -> Result<Vec<SearchHit>> {
        let query = crate::search::to_fts_query(raw);
        match (query, tag_ids.is_empty()) {
            // Нечего искать.
            (None, true) => Ok(Vec::new()),

            // Только текст — как раньше.
            (Some(q), true) => {
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
                let rows = stmt.query_map(params![q, limit], map_hit)?;
                Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
            }

            // Текст + фильтр по тегам.
            (Some(q), false) => {
                let sql = format!(
                    "SELECT c.id, c.series_id, c.title,
                            snippet(note_fts, 1, '[', ']', '…', 12) AS snip,
                            bm25(note_fts, 5.0, 1.0) AS rank
                     FROM note_fts
                     JOIN note_content c ON c.rowid = note_fts.rowid
                     WHERE note_fts MATCH ?
                       AND c.series_id IN (
                         SELECT series_id FROM series_tag
                         WHERE tag_id IN ({ph})
                         GROUP BY series_id HAVING COUNT(DISTINCT tag_id) = ?
                       )
                     ORDER BY rank
                     LIMIT ?",
                    ph = placeholders(tag_ids.len())
                );
                let mut values: Vec<Value> = Vec::new();
                values.push(Value::Text(q));
                values.extend(tag_ids.iter().map(|t| Value::Text((*t).to_string())));
                values.push(Value::Integer(tag_ids.len() as i64));
                values.push(Value::Integer(limit));
                let mut stmt = self.conn.prepare(&sql)?;
                let rows = stmt.query_map(params_from_iter(values), map_hit)?;
                Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
            }

            // Только теги (без текста): блоки серий, у которых есть все выбранные теги.
            (None, false) => {
                let sql = format!(
                    "SELECT c.id, c.series_id, c.title, substr(c.text, 1, 120) AS snip, 0.0 AS rank
                     FROM note_content c
                     WHERE c.series_id IN (
                       SELECT series_id FROM series_tag
                       WHERE tag_id IN ({ph})
                       GROUP BY series_id HAVING COUNT(DISTINCT tag_id) = ?
                     )
                     ORDER BY c.updated_at DESC
                     LIMIT ?",
                    ph = placeholders(tag_ids.len())
                );
                let mut values: Vec<Value> = Vec::new();
                values.extend(tag_ids.iter().map(|t| Value::Text((*t).to_string())));
                values.push(Value::Integer(tag_ids.len() as i64));
                values.push(Value::Integer(limit));
                let mut stmt = self.conn.prepare(&sql)?;
                let rows = stmt.query_map(params_from_iter(values), map_hit)?;
                Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
            }
        }
    }
}

/// Маппер строки результата в [`SearchHit`].
fn map_hit(row: &rusqlite::Row) -> rusqlite::Result<SearchHit> {
    Ok(SearchHit {
        content_id: row.get(0)?,
        series_id: row.get(1)?,
        title: row.get(2)?,
        snippet: row.get(3)?,
        rank: row.get(4)?,
    })
}

/// Маппер строки результата в [`TechTag`] (порядок колонок: id, name, description, type_id, type).
fn map_tag(row: &rusqlite::Row) -> rusqlite::Result<TechTag> {
    Ok(TechTag {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        type_id: row.get(3)?,
        type_name: row.get(4)?,
    })
}
