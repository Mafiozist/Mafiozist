-- Миграция 001 — инициализация схемы DEVNOTES.
-- ПОЧЕМУ так: иерархия Project → NoteSeries → NoteContent перенята из проекта Portfolio
-- (см. docs/05-DATA-MODEL.md). Все доменные сущности обязаны иметь created_at/updated_at
-- (UTC RFC3339). Идентификаторы — UUID v7 в виде TEXT, генерируются клиентом.
-- Полнотекстовый поиск — FTS5 external content над note_content (см. docs/08-SEARCH.md).

-- Проект — верхнеуровневая группа заметок.
CREATE TABLE project (
    id          TEXT PRIMARY KEY,          -- UUID v7
    name        TEXT NOT NULL,
    short_name  TEXT,
    description TEXT,
    created_at  TEXT NOT NULL,             -- UTC RFC3339
    updated_at  TEXT NOT NULL
);

-- Серия (тема) заметок внутри проекта.
CREATE TABLE note_series (
    id          TEXT PRIMARY KEY,
    project_id  TEXT REFERENCES project(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE INDEX idx_series_project ON note_series(project_id);

-- Блок контента — единица заметки внутри серии. Порядок задаётся sort_order (drag-and-drop).
-- type: 'markdown' | 'code' | 'image' | 'link' (см. docs/05-DATA-MODEL.md).
CREATE TABLE note_content (
    id          TEXT PRIMARY KEY,
    series_id   TEXT NOT NULL REFERENCES note_series(id) ON DELETE CASCADE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    title       TEXT,
    text        TEXT NOT NULL,
    type        TEXT NOT NULL DEFAULT 'markdown',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE INDEX idx_content_series ON note_content(series_id, sort_order);

-- Теги технологий и их категории.
CREATE TABLE tech_tag_type (
    id   TEXT PRIMARY KEY,
    type TEXT NOT NULL UNIQUE
);
CREATE TABLE tech_tag (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    type_id     TEXT REFERENCES tech_tag_type(id) ON DELETE SET NULL
);
-- Связь серии ↔ тег (многие-ко-многим).
CREATE TABLE series_tag (
    series_id TEXT NOT NULL REFERENCES note_series(id) ON DELETE CASCADE,
    tag_id    TEXT NOT NULL REFERENCES tech_tag(id)   ON DELETE CASCADE,
    PRIMARY KEY (series_id, tag_id)
);

-- ---------------------------------------------------------------------------
-- Полнотекстовый индекс (FTS5, external content над note_content).
-- Поля title/text индексируются; rowid = note_content.rowid.
-- Токенизатор unicode61 с удалением диакритики (remove_diacritics 2) — без ICU.
-- Ограничение по русской морфологии осознанное: MVP ищет по словоформам + префиксам
-- (см. компромисс в docs/08-SEARCH.md; trigram-фолбэк — на будущее).
CREATE VIRTUAL TABLE note_fts USING fts5(
    title,
    text,
    content       = 'note_content',
    content_rowid = 'rowid',
    tokenize      = 'unicode61 remove_diacritics 2'
);

-- Триггеры поддержания индекса в актуальном состоянии.
CREATE TRIGGER note_content_ai AFTER INSERT ON note_content BEGIN
    INSERT INTO note_fts(rowid, title, text) VALUES (new.rowid, new.title, new.text);
END;
CREATE TRIGGER note_content_ad AFTER DELETE ON note_content BEGIN
    INSERT INTO note_fts(note_fts, rowid, title, text) VALUES ('delete', old.rowid, old.title, old.text);
END;
CREATE TRIGGER note_content_au AFTER UPDATE ON note_content BEGIN
    INSERT INTO note_fts(note_fts, rowid, title, text) VALUES ('delete', old.rowid, old.title, old.text);
    INSERT INTO note_fts(rowid, title, text) VALUES (new.rowid, new.title, new.text);
END;
