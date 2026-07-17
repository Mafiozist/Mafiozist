# 05 — Модель данных

> **Что это за файл.** Каноническое описание доменной модели DEVNOTES: сущности, их атрибуты, связи и инварианты; полная схема локальной БД **SQLite** (DDL со всеми `CREATE TABLE`, типами, PK/FK, индексами, `CHECK`-ограничениями), виртуальная таблица **FTS5** для мгновенного поиска и триггеры синхронизации индекса; стратегия версионируемых миграций. Модель перенята из веб-проекта **Portfolio (Mafiozist)** и расширена под десктоп local-first: добавлены сущности для вложений, wiki-связей, истории версий, напоминаний, настроек и журнала синхронизации. Отдельный раздел — соответствие таблиц SQLite доменным классам Portfolio (EF Core). Документ — **источник правды по структуре данных**; из него растут репозитории Rust-ядра, TS-типы фронта и миграционные скрипты.

> **Статус:** проектирование · **Дата:** 2026-07-17 · **Язык:** русский, тон инженерный · **Область:** десктоп v1 (Windows / macOS / Linux).

---

## Связанные документы

Пути относительно `DevNotes/`. Канон именования и глоссарий — `CLAUDE.md`.

| Документ | Назначение | Роль для этого файла |
| --- | --- | --- |
| [`CLAUDE.md`](../CLAUDE.md) | Конвенции, глоссарий, инварианты, DoD | Источник правил именования |
| [`docs/01-VISION.md`](01-VISION.md) | Видение, персоны, сценарии | «Зачем» существует модель |
| [`docs/02-SPECIFICATION.md`](02-SPECIFICATION.md) | Большое ТЗ (раздел 11 «Требования к данным») | Функциональные требования к данным |
| [`docs/03-FEATURES.md`](03-FEATURES.md) | Каталог фич MoSCoW | Какие фичи опираются на сущности |
| [`docs/04-SEARCH-FTS5.md`](04-SEARCH-FTS5.md) | FTS5 external content, bm25, токенизация | Детализация поискового индекса |
| [`docs/05-SYNC-YANDEX.md`](05-SYNC-YANDEX.md) | OAuth 2.0 + PKCE, oplog (ChangeLog), LWW | Детализация `ChangeLog`/`SyncState` |
| [`docs/06-DESIGN-SYSTEM.md`](06-DESIGN-SYSTEM.md) | Дизайн-токены, компоненты | Потребитель модели в UI |
| [`docs/07-ADR/`](07-ADR/) | Architecture Decision Records | Обоснование UUID v7, oplog, FTS5 |

> Ссылки на ещё не созданные файлы — плановые. При расхождении формулировок канон — `CLAUDE.md` и `consistencyNotes` из WBS.

---

## 1. Инварианты модели (обязательны для всех таблиц)

Эти правила действуют для **каждой** доменной таблицы; ниже в DDL они не повторяются в прозе, но соблюдаются в схеме.

| # | Инвариант | Реализация в SQLite |
| --- | --- | --- |
| И-1 | **ID = UUID v7 (строка)**, генерируется клиентом до вставки | `id TEXT PRIMARY KEY NOT NULL` (36 симв., lower-case, дефисы) |
| И-2 | **`created_at` и `updated_at` обязательны** у каждой доменной сущности | `TEXT NOT NULL` — UTC **ISO 8601** `YYYY-MM-DDTHH:MM:SS.sssZ` |
| И-3 | Даты хранятся в **UTC**, отображаются в локальной таймзоне пользователя (конвертация на UI) | Строка с суффиксом `Z`; сравнение лексикографическое = хронологическое |
| И-4 | Схема — **snake_case**; домен на Rust — PascalCase структуры, на TS — camelCase поля | Маппинг в репозиториях |
| И-5 | `updated_at` обновляется при любом изменении строки | Триггеры `AFTER UPDATE` (см. §7) |
| И-6 | Любое доменное изменение фиксируется в `change_log` (oplog) | Пишется в use-case-слое одной транзакцией с мутацией |
| И-7 | Внешние ключи включены | `PRAGMA foreign_keys = ON;` при каждом соединении |
| И-8 | Мягкое удаление где важна история/синк; жёсткое — для связок | Поле `deleted_at TEXT NULL` (tombstone) на синкаемых сущностях |

**Почему UUID v7, а не autoincrement.** Local-first + офлайн-создание на нескольких устройствах: суррогатный автоинкремент даёт коллизии при слиянии. UUID v7 монотонно растёт по времени (первые 48 бит — Unix-время в мс), поэтому он одновременно уникален глобально **и** даёт естественную сортировку по created-порядку, что удобно для индексов и курсорной пагинации.

**Тип хранения времени.** SQLite не имеет типа `DATETIME`; выбран `TEXT` в ISO 8601 UTC (а не Unix-epoch INTEGER) ради читаемости при отладке БД и корректной лексикографической сортировки. Точность — миллисекунды.

---

## 2. Обзор сущностей

### 2.1. Ядро иерархии контента

| Сущность (SQLite) | Домен | Назначение | Синкается | В MVP |
| --- | --- | --- | --- | --- |
| `company` | Company | Компания-владелец проектов (из Portfolio) | да | нет¹ |
| `project` | Project | Проект — группа серий заметок | да | **да** |
| `note_series` | NoteSeries | Серия/тема заметок внутри проекта | да | **да** |
| `note_content` | NoteContent | Блок контента серии (единица) | да | **да** |
| `note_content_type` | NoteContentType | Справочник типов блока | seed² | **да** |
| `tech_tag` | TechTag | Тег технологии | да | **да** |
| `tech_tag_type` | TechTagType | Категория тега | seed² | **да** |
| `note_series_tag` | — (связка) | M:N серия ↔ тег | да | **да** |
| `project_tag` | — (связка) | M:N проект ↔ тег | да | **да** |

¹ `company` заведена в схеме для совместимости с Portfolio и будущего, но в UI v1 не используется: `project.company_id` всегда `NULL`. См. §9.
² Справочники наполняются миграцией-сидом, не редактируются пользователем в v1.

### 2.2. Расширения для десктопа

| Сущность (SQLite) | Домен | Назначение | Синкается | В MVP |
| --- | --- | --- | --- | --- |
| `attachment` | Attachment | Вложение блока (файл рядом с БД, content-addressable) | да (файл+мета) | should |
| `note_link` | NoteLink | Wiki-связь `[[...]]` между сериями/блоками + backlinks | да | could |
| `note_version` | NoteVersion | История версий блока (diff/откат) | локально³ | could |
| `reminder` | Reminder | Напоминание по серии/блоку | да | could |
| `setting` | Setting | Настройки приложения (key/value) | нет⁴ | **да** |
| `sync_state` | SyncState | Курсоры/ревизии синка, `device_id` | нет⁴ | **да** |
| `change_log` | ChangeLog | Oplog: журнал изменений для синка | сам является синком | **да** |
| `notes_fts` | — | Виртуальная таблица FTS5 (не доменная) | нет (производная) | **да** |

³ История версий — устройство-локальная (шумный, тяжёлый поток); на облако уходит только «текущее» состояние блока через oplog.
⁴ `setting` и `sync_state` — машинно-специфичны (пути, токены-ссылки, device_id), между устройствами не переносятся.

### 2.3. Словарь синонимов (антидубли)

| Канон | НЕ употреблять |
| --- | --- |
| **NoteSeries** | «тема», «заметка целиком», «страница» |
| **NoteContent** | «блок текста», «параграф», «карточка» |
| **TechTag** | «технология», «метка», «label» |
| **ChangeLog** | «oplog» (допустимо как пояснение), «журнал» без уточнения |

---

## 3. ER-диаграмма

```mermaid
erDiagram
    COMPANY ||--o{ PROJECT : "владеет (v2)"
    PROJECT ||--o{ NOTE_SERIES : "содержит"
    NOTE_SERIES ||--o{ NOTE_CONTENT : "состоит из блоков"
    NOTE_CONTENT_TYPE ||--o{ NOTE_CONTENT : "типизирует"
    NOTE_SERIES }o--o{ TECH_TAG : "note_series_tag"
    PROJECT }o--o{ TECH_TAG : "project_tag"
    TECH_TAG_TYPE ||--o{ TECH_TAG : "категоризирует"
    NOTE_CONTENT ||--o{ ATTACHMENT : "вложения"
    NOTE_CONTENT ||--o{ NOTE_VERSION : "история версий"
    NOTE_SERIES ||--o{ NOTE_LINK : "исходящие ссылки"
    NOTE_SERIES ||--o{ REMINDER : "напоминания"

    COMPANY {
        string id PK
        string name
        string description
        string website
        string created_at
        string updated_at
    }
    PROJECT {
        string id PK
        string company_id FK
        string name
        string short_name
        string description
        int archived
        string created_at
        string updated_at
        string deleted_at
    }
    NOTE_SERIES {
        string id PK
        string project_id FK
        string title
        string description
        int pinned
        string created_at
        string updated_at
        string deleted_at
    }
    NOTE_CONTENT {
        string id PK
        string series_id FK
        int sort_order
        string title
        string text
        int type_id FK
        string language
        string created_at
        string updated_at
        string deleted_at
    }
    NOTE_CONTENT_TYPE {
        int id PK
        string type
    }
    TECH_TAG {
        string id PK
        string name
        int type_id FK
        string description
        string created_at
        string updated_at
    }
    TECH_TAG_TYPE {
        int id PK
        string type
    }
    NOTE_SERIES_TAG {
        string series_id FK
        string tag_id FK
    }
    PROJECT_TAG {
        string project_id FK
        string tag_id FK
    }
    ATTACHMENT {
        string id PK
        string note_content_id FK
        string file_name
        string mime
        int size
        string sha256
        string local_path
        string sync_status
        string created_at
        string updated_at
    }
    NOTE_LINK {
        string id PK
        string source_series_id FK
        string target_series_id FK
        string source_content_id FK
        string raw_target
        string kind
        string created_at
        string updated_at
    }
    NOTE_VERSION {
        string id PK
        string note_content_id FK
        int revision
        string title
        string text
        string diff
        string created_at
    }
    REMINDER {
        string id PK
        string series_id FK
        string content_id FK
        string remind_at
        int done
        string note
        string created_at
        string updated_at
    }
    SETTING {
        string key PK
        string value
        string updated_at
    }
    SYNC_STATE {
        string key PK
        string value
        string updated_at
    }
    CHANGE_LOG {
        string id PK
        string entity
        string entity_id
        string op
        string payload_json
        string ts
        string device_id
        int synced
    }
```

> `notes_fts` (FTS5) на диаграмме не показана — это производная (индекс над `note_content` + `note_series`), не доменная связь. Детали — §6 и `04-SEARCH-FTS5.md`.

### 3.1. Кардинальности и правила каскадов

| Связь | Кардинальность | ON DELETE | Обоснование |
| --- | --- | --- | --- |
| Company → Project | 1:N (опц.) | `SET NULL` | Удаление компании не должно терять проекты |
| Project → NoteSeries | 1:N (опц.) | `SET NULL` | Серия может «висеть» без проекта (инбокс) |
| NoteSeries → NoteContent | 1:N | `CASCADE` | Блоки не существуют без серии |
| NoteContentType → NoteContent | 1:N | `RESTRICT` | Нельзя удалить используемый тип |
| TechTagType → TechTag | 1:N | `RESTRICT` | Нельзя удалить используемую категорию |
| NoteSeries ↔ TechTag | M:N | `CASCADE` (обе стороны) | Связка чистится при удалении любой стороны |
| Project ↔ TechTag | M:N | `CASCADE` | То же |
| NoteContent → Attachment | 1:N | `CASCADE` | Вложение без блока бессмысленно |
| NoteContent → NoteVersion | 1:N | `CASCADE` | История уходит вместе с блоком |
| NoteSeries → NoteLink | 1:N | `CASCADE` | Исходящие ссылки удаляются с серией |
| NoteSeries → Reminder | 1:N | `CASCADE` | Напоминание без цели бессмысленно |

> **Замечание о синке и удалении.** Для синкаемых сущностей «удаление» в UI = проставление `deleted_at` (tombstone) + запись `op='delete'` в `change_log`. Физический `DELETE ... CASCADE` применяется только при жёсткой очистке (compaction) уже синхронизированных tombstone'ов. Это гарантирует, что удаление доедет до второго устройства через oplog, а не «воскреснет».

---

## 4. Каталог полей по сущностям

Ниже — семантика нетривиальных полей. Общие `id/created_at/updated_at/deleted_at` описаны в §1 и не дублируются.

### 4.1. `project`
| Поле | Тип | Null | Смысл |
| --- | --- | --- | --- |
| `company_id` | TEXT | да | FK на `company`; в v1 всегда NULL |
| `name` | TEXT | нет | Название проекта |
| `short_name` | TEXT | да | Короткая метка для чипов/палитры |
| `description` | TEXT | да | Описание (markdown) |
| `archived` | INTEGER | нет | 0/1; архив без удаления (скрыт из основных списков) |

### 4.2. `note_series`
| Поле | Тип | Null | Смысл |
| --- | --- | --- | --- |
| `project_id` | TEXT | да | FK на `project`; NULL = «инбокс» |
| `title` | TEXT | нет | Заголовок серии (индексируется FTS5) |
| `description` | TEXT | да | Краткое описание |
| `pinned` | INTEGER | нет | 0/1; закрепление вверху списка |

### 4.3. `note_content`
| Поле | Тип | Null | Смысл |
| --- | --- | --- | --- |
| `series_id` | TEXT | нет | FK на `note_series` |
| `sort_order` | INTEGER | нет | Порядок блока (drag-and-drop, @dnd-kit) |
| `title` | TEXT | да | Опциональный заголовок блока (индексируется FTS5) |
| `text` | TEXT | нет | Тело блока (markdown/код/URL/подпись) — индексируется FTS5 |
| `type_id` | INTEGER | нет | FK на `note_content_type` |
| `language` | TEXT | да | Язык подсветки для `type='code'` (`ts`, `rust`, `sql`, …) |

> **`sort_order`.** Целое с шагом (например, 1000, 2000, …) — вставка между блоками без переиндексации всей серии. Периодический ре-баланс шага при исчерпании зазора. Уникальность порядка внутри серии обеспечивается на уровне use-case, не БД (перестановки — частая операция).

### 4.4. `note_content_type` (seed)
| id | type |
| --- | --- |
| 1 | `markdown` |
| 2 | `code` |
| 3 | `image` |
| 4 | `link` |

### 4.5. `tech_tag` / `tech_tag_type`
`tech_tag`: `name` (уникально, нормализуется в lower-case для поиска), `type_id` → `tech_tag_type`, `description`.

`tech_tag_type` (seed):
| id | type |
| --- | --- |
| 1 | `language` |
| 2 | `framework` |
| 3 | `tool` |
| 4 | `database` |
| 5 | `devops` |
| 6 | `other` |

### 4.6. `attachment`
| Поле | Тип | Смысл |
| --- | --- | --- |
| `note_content_id` | TEXT | FK на блок |
| `file_name` | TEXT | Отображаемое имя файла |
| `mime` | TEXT | MIME-тип (`image/png`, …) |
| `size` | INTEGER | Размер в байтах |
| `sha256` | TEXT | Хэш содержимого (content-addressable дедупликация) |
| `local_path` | TEXT | Путь в blob-хранилище `<data_dir>/attachments/<sha256[:2]>/<sha256>` |
| `sync_status` | TEXT | `local` \| `queued` \| `synced` \| `remote_only` |

### 4.7. `note_link` (wiki-links / backlinks)
| Поле | Смысл |
| --- | --- |
| `source_series_id` | Откуда ссылка |
| `source_content_id` | Конкретный блок-источник (NULL = на уровне серии) |
| `target_series_id` | Разрезолвленная цель (NULL, если ещё не создана — «висячая» ссылка) |
| `raw_target` | Исходный текст `[[Название/slug]]` — для ре-резолва |
| `kind` | `wiki` \| `mention` |

> **Backlinks** = обратный запрос `SELECT ... WHERE target_series_id = :id`. Ре-резолв висячих ссылок — триггером-приложением при создании серии с совпадающим заголовком/slug.

### 4.8. `note_version`
| Поле | Смысл |
| --- | --- |
| `note_content_id` | Блок, к которому относится ревизия |
| `revision` | Порядковый номер ревизии (1..N) |
| `title` / `text` | Снимок содержимого на момент ревизии |
| `diff` | Unified-diff к предыдущей ревизии (для компактного показа/отката) |
| `created_at` | Момент создания ревизии (нет `updated_at` — версия неизменяема) |

### 4.9. `reminder`
| Поле | Смысл |
| --- | --- |
| `series_id` / `content_id` | Цель напоминания (одно из; `content_id` опц.) |
| `remind_at` | UTC ISO 8601 момент срабатывания |
| `done` | 0/1 — выполнено/снято |
| `note` | Текст напоминания |

### 4.10. `setting` / `sync_state` (key-value)
Плоские таблицы `key TEXT PRIMARY KEY, value TEXT, updated_at TEXT`. Примеры ключей:
- `setting`: `theme=dark`, `locale=ru`, `autosave.debounce_ms=600`, `backup.interval_h=24`.
- `sync_state`: `device_id=<uuid>`, `yadisk.cursor=<opaque>`, `yadisk.last_revision=<n>`, `oplog.last_pushed_ts=<iso>`.

### 4.11. `change_log` (oplog)
| Поле | Смысл |
| --- | --- |
| `id` | UUID v7 записи журнала |
| `entity` | Имя сущности: `project` \| `note_series` \| `note_content` \| … |
| `entity_id` | ID изменённой строки |
| `op` | `insert` \| `update` \| `delete` |
| `payload_json` | Снимок/дельта строки (для применения на др. устройстве) |
| `ts` | UTC ISO 8601 момента операции (основа LWW) |
| `device_id` | Устройство-источник |
| `synced` | 0/1 — выгружено ли в app folder Я.Диска |

Подробности алгоритма выгрузки и LWW-разрешения — `05-SYNC-YANDEX.md`.

---

## 5. Полный DDL SQLite

> Порядок: `PRAGMA` → справочники → ядро → связки → расширения → служебные → FTS5 → триггеры. Весь DDL идемпотентен в рамках миграции `0001` (создание с нуля). Числовые флаги через `CHECK (col IN (0,1))`.

```sql
-- === PRAGMA (устанавливаются на каждое соединение в Rust-слое) ===
PRAGMA journal_mode = WAL;        -- конкурентное чтение при записи
PRAGMA foreign_keys = ON;         -- включить FK-констрейнты
PRAGMA busy_timeout = 5000;       -- ждать блокировку до 5 c
PRAGMA synchronous = NORMAL;      -- баланс скорость/надёжность при WAL

-- ============================================================
--  СПРАВОЧНИКИ (seed через миграцию)
-- ============================================================
CREATE TABLE note_content_type (
    id   INTEGER PRIMARY KEY,
    type TEXT NOT NULL UNIQUE          -- markdown | code | image | link
);

CREATE TABLE tech_tag_type (
    id   INTEGER PRIMARY KEY,
    type TEXT NOT NULL UNIQUE          -- language | framework | tool | database | devops | other
);

-- ============================================================
--  ЯДРО ИЕРАРХИИ
-- ============================================================
CREATE TABLE company (                 -- из Portfolio; в v1 не используется в UI
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    description TEXT,
    website     TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE project (
    id          TEXT PRIMARY KEY NOT NULL,
    company_id  TEXT,
    name        TEXT NOT NULL,
    short_name  TEXT,
    description TEXT,
    archived    INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0,1)),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    FOREIGN KEY (company_id) REFERENCES company(id) ON DELETE SET NULL
);

CREATE TABLE note_series (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT,
    title       TEXT NOT NULL,
    description TEXT,
    pinned      INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0,1)),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE SET NULL
);

CREATE TABLE note_content (
    id          TEXT PRIMARY KEY NOT NULL,
    series_id   TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    title       TEXT,
    text        TEXT NOT NULL DEFAULT '',
    type_id     INTEGER NOT NULL,
    language    TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    FOREIGN KEY (series_id) REFERENCES note_series(id) ON DELETE CASCADE,
    FOREIGN KEY (type_id)   REFERENCES note_content_type(id) ON DELETE RESTRICT
);

-- ============================================================
--  ТЕГИ И СВЯЗКИ M:N
-- ============================================================
CREATE TABLE tech_tag (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    type_id     INTEGER NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    FOREIGN KEY (type_id) REFERENCES tech_tag_type(id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX ux_tech_tag_name ON tech_tag (lower(name));

CREATE TABLE note_series_tag (
    series_id TEXT NOT NULL,
    tag_id    TEXT NOT NULL,
    PRIMARY KEY (series_id, tag_id),
    FOREIGN KEY (series_id) REFERENCES note_series(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)    REFERENCES tech_tag(id)    ON DELETE CASCADE
);

CREATE TABLE project_tag (
    project_id TEXT NOT NULL,
    tag_id     TEXT NOT NULL,
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY (project_id) REFERENCES project(id)  ON DELETE CASCADE,
    FOREIGN KEY (tag_id)     REFERENCES tech_tag(id)  ON DELETE CASCADE
);

-- ============================================================
--  РАСШИРЕНИЯ ДЕСКТОПА
-- ============================================================
CREATE TABLE attachment (
    id              TEXT PRIMARY KEY NOT NULL,
    note_content_id TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    mime            TEXT NOT NULL,
    size            INTEGER NOT NULL DEFAULT 0,
    sha256          TEXT NOT NULL,
    local_path      TEXT NOT NULL,
    sync_status     TEXT NOT NULL DEFAULT 'local'
                    CHECK (sync_status IN ('local','queued','synced','remote_only')),
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT,
    FOREIGN KEY (note_content_id) REFERENCES note_content(id) ON DELETE CASCADE
);
CREATE INDEX ix_attachment_content ON attachment (note_content_id);
CREATE INDEX ix_attachment_sha     ON attachment (sha256);

CREATE TABLE note_link (
    id                TEXT PRIMARY KEY NOT NULL,
    source_series_id  TEXT NOT NULL,
    source_content_id TEXT,
    target_series_id  TEXT,                 -- NULL = висячая ссылка
    raw_target        TEXT NOT NULL,        -- исходный [[...]]
    kind              TEXT NOT NULL DEFAULT 'wiki' CHECK (kind IN ('wiki','mention')),
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    FOREIGN KEY (source_series_id)  REFERENCES note_series(id)  ON DELETE CASCADE,
    FOREIGN KEY (source_content_id) REFERENCES note_content(id) ON DELETE CASCADE,
    FOREIGN KEY (target_series_id)  REFERENCES note_series(id)  ON DELETE SET NULL
);
CREATE INDEX ix_note_link_target ON note_link (target_series_id);
CREATE INDEX ix_note_link_source ON note_link (source_series_id);

CREATE TABLE note_version (
    id              TEXT PRIMARY KEY NOT NULL,
    note_content_id TEXT NOT NULL,
    revision        INTEGER NOT NULL,
    title           TEXT,
    text            TEXT NOT NULL DEFAULT '',
    diff            TEXT,                     -- unified diff к revision-1
    created_at      TEXT NOT NULL,            -- версия неизменяема => updated_at не нужен
    FOREIGN KEY (note_content_id) REFERENCES note_content(id) ON DELETE CASCADE,
    UNIQUE (note_content_id, revision)
);
CREATE INDEX ix_note_version_content ON note_version (note_content_id, revision DESC);

CREATE TABLE reminder (
    id         TEXT PRIMARY KEY NOT NULL,
    series_id  TEXT,
    content_id TEXT,
    remind_at  TEXT NOT NULL,                 -- UTC ISO 8601
    done       INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0,1)),
    note       TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (series_id)  REFERENCES note_series(id)  ON DELETE CASCADE,
    FOREIGN KEY (content_id) REFERENCES note_content(id) ON DELETE CASCADE,
    CHECK (series_id IS NOT NULL OR content_id IS NOT NULL)
);
CREATE INDEX ix_reminder_due ON reminder (remind_at) WHERE done = 0;

-- ============================================================
--  СЛУЖЕБНЫЕ (не синкаются, кроме change_log)
-- ============================================================
CREATE TABLE setting (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE sync_state (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE change_log (
    id           TEXT PRIMARY KEY NOT NULL,   -- UUID v7 => естественный порядок
    entity       TEXT NOT NULL,
    entity_id    TEXT NOT NULL,
    op           TEXT NOT NULL CHECK (op IN ('insert','update','delete')),
    payload_json TEXT,
    ts           TEXT NOT NULL,               -- UTC ISO 8601 (основа LWW)
    device_id    TEXT NOT NULL,
    synced       INTEGER NOT NULL DEFAULT 0 CHECK (synced IN (0,1))
);
CREATE INDEX ix_change_log_unsynced ON change_log (synced, ts) WHERE synced = 0;
CREATE INDEX ix_change_log_entity   ON change_log (entity, entity_id);

-- ============================================================
--  ИНДЕКСЫ ПОД ТИПОВЫЕ ЗАПРОСЫ
-- ============================================================
CREATE INDEX ix_series_project   ON note_series (project_id) WHERE deleted_at IS NULL;
CREATE INDEX ix_series_pinned    ON note_series (pinned, updated_at DESC);
CREATE INDEX ix_content_series   ON note_content (series_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX ix_content_type     ON note_content (type_id);
CREATE INDEX ix_project_archived ON project (archived) WHERE deleted_at IS NULL;
```

---

## 6. FTS5: виртуальная таблица поиска

Полнотекстовый индекс — **external content** над `note_content` (поля `title`, `text`) плюс заголовок серии, подмешиваемый как отдельная колонка. Ранжирование — `bm25`. Целевой SLA — **<50 мс на 10k блоков**. Токенизатор по умолчанию `unicode61` (быстрый, точные словоформы); для русской морфологии предусмотрен fallback на `trigram` (см. риск в WBS и `04-SEARCH-FTS5.md`).

```sql
-- external content: строки хранит note_content, FTS хранит только индекс
CREATE VIRTUAL TABLE notes_fts USING fts5 (
    title,                 -- note_content.title
    text,                  -- note_content.text
    series_title,          -- note_series.title (денормализовано в индекс)
    content='note_content',
    content_rowid='rowid',
    tokenize = 'unicode61 remove_diacritics 2'
);
```

> `content_rowid` = скрытый `rowid` таблицы `note_content` (не UUID — FTS5 требует INTEGER rowid). Маппинг `rowid → id (UUID)` берётся из `note_content` при выдаче результатов.

### 6.1. Пример поискового запроса с bm25 и сниппетом
```sql
SELECT
    c.id,
    c.series_id,
    snippet(notes_fts, 1, '<mark>', '</mark>', '…', 12) AS snippet,
    bm25(notes_fts, 5.0, 1.0, 3.0)                      AS rank  -- веса: title>series>text
FROM notes_fts
JOIN note_content c ON c.rowid = notes_fts.rowid
WHERE notes_fts MATCH :query
  AND c.deleted_at IS NULL
ORDER BY rank            -- bm25: меньше = релевантнее
LIMIT 50;
```

---

## 7. Триггеры

Две группы: (7.1) синхронизация FTS-индекса с `note_content`/`note_series`; (7.2) сопровождающие поля (`updated_at`). Бизнес-триггеры (запись в `change_log`, история версий) — **в use-case-слое Rust**, не в БД: это осознанное решение (тестируемость, единая транзакция, контроль device_id).

### 7.1. Синхронизация FTS5 (external content)
```sql
-- INSERT блока -> добавить в индекс
CREATE TRIGGER trg_content_ai AFTER INSERT ON note_content BEGIN
  INSERT INTO notes_fts(rowid, title, text, series_title)
  VALUES (
    new.rowid, new.title, new.text,
    (SELECT title FROM note_series WHERE id = new.series_id)
  );
END;

-- DELETE блока -> изъять из индекса (спецсинтаксис external content)
CREATE TRIGGER trg_content_ad AFTER DELETE ON note_content BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, text, series_title)
  VALUES ('delete', old.rowid, old.title, old.text,
          (SELECT title FROM note_series WHERE id = old.series_id));
END;

-- UPDATE блока -> delete+insert в индексе
CREATE TRIGGER trg_content_au AFTER UPDATE ON note_content BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, text, series_title)
  VALUES ('delete', old.rowid, old.title, old.text,
          (SELECT title FROM note_series WHERE id = old.series_id));
  INSERT INTO notes_fts(rowid, title, text, series_title)
  VALUES (new.rowid, new.title, new.text,
          (SELECT title FROM note_series WHERE id = new.series_id));
END;

-- переименование серии -> обновить series_title во всех её блоках в индексе
CREATE TRIGGER trg_series_title_au AFTER UPDATE OF title ON note_series BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, text, series_title)
  SELECT 'delete', c.rowid, c.title, c.text, old.title
  FROM note_content c WHERE c.series_id = new.id;
  INSERT INTO notes_fts(rowid, title, text, series_title)
  SELECT c.rowid, c.title, c.text, new.title
  FROM note_content c WHERE c.series_id = new.id;
END;
```

### 7.2. Автообновление `updated_at`
```sql
CREATE TRIGGER trg_project_touch AFTER UPDATE ON project
FOR EACH ROW WHEN new.updated_at = old.updated_at BEGIN
  UPDATE project SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
  WHERE id = old.id;
END;
-- Аналогичные trg_*_touch для note_series, note_content, tech_tag,
-- attachment, note_link, reminder (по одному шаблону).
```

> Условие `WHEN new.updated_at = old.updated_at` предотвращает рекурсию и уважает `updated_at`, уже проставленный из кода (при применении oplog с чужого устройства сохраняем исходный `ts`, иначе LWW сломается).

---

## 8. Стратегия миграций

### 8.1. Принципы
- **Версионирование схемы** через `PRAGMA user_version` (целое, монотонно растёт). Одна миграция = один SQL-скрипт `NNNN_description.sql`, применяется в транзакции.
- Миграции **аддитивны и необратимы вперёд**: только `ADD COLUMN`, новые таблицы/индексы, backfill. Ломающие изменения — через паттерн «новая таблица + копирование + swap» (SQLite не умеет `DROP COLUMN` до 3.35 надёжно; полагаться нельзя из-за старых WebView-сборок).
- Скрипты встроены в бинарь Rust-ядра (`include_str!`), применяются на старте до открытия репозиториев.
- Перед структурной миграцией — авто-`VACUUM INTO` snapshot (см. `03-FEATURES`/бэкапы) для отката.

### 8.2. Раннер (псевдо-Rust)
```rust
fn migrate(conn: &Connection) -> Result<()> {
    let mut v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (target, sql) in MIGRATIONS.iter() {   // отсортированы по номеру
        if *target > v {
            conn.execute_batch("BEGIN")?;
            conn.execute_batch(sql)?;           // DDL + seed + backfill
            conn.execute_batch(&format!("PRAGMA user_version = {target}"))?;
            conn.execute_batch("COMMIT")?;
            v = *target;
        }
    }
    Ok(())
}
```

### 8.3. Реестр миграций (план)
| Версия | Файл | Содержимое |
| --- | --- | --- |
| 1 | `0001_init.sql` | Все таблицы ядра, связки, служебные, индексы (§5) |
| 2 | `0002_seed_types.sql` | Seed `note_content_type`, `tech_tag_type` |
| 3 | `0003_fts.sql` | `notes_fts` + триггеры §7.1; начальный rebuild индекса |
| 4 | `0004_touch_triggers.sql` | Триггеры `updated_at` §7.2 |
| 5 | `0005_attachments.sql` | `attachment` (если выносится из ядра как should-фича) |
| 6 | `0006_wiki_versions.sql` | `note_link`, `note_version` (could-фичи) |
| 7 | `0007_reminders.sql` | `reminder` |

> **Ребилд FTS после схемных изменений**, затрагивающих индексируемые поля: `INSERT INTO notes_fts(notes_fts) VALUES('rebuild');`.

### 8.4. Правила эволюции
- Никогда не переиспользовать номер `user_version`.
- Backfill больших таблиц — батчами по rowid, чтобы не держать долгую блокировку при WAL.
- Изменение набора индексируемых FTS-полей = новая миграция + `rebuild`.
- Совместимость синка: добавление поля не ломает oplog (payload_json версионируется своим `schema_version` в `sync_state`).

---

## 9. Соответствие доменным классам Portfolio (EF Core)

Модель DEVNOTES выведена из веб-проекта Portfolio. Ниже — маппинг сущностей и осознанные отличия десктопа.

| Portfolio (EF Core) | DEVNOTES (SQLite) | Отличия десктопа |
| --- | --- | --- |
| `Company(Id,Name,Description?,Website?,CreatedAt?)` | `company` | + `updated_at`, `deleted_at`; в UI v1 не используется |
| `Project(Id,CompanyId?,Name,ShortName?,Description?,CreatedAt?)` | `project` | + `archived`, `updated_at`, `deleted_at`; `company_id` всегда NULL в v1 |
| `NoteSeries(Id,Title,Description?,CreatedAt?,ProjectId?)` | `note_series` | + `pinned`, `updated_at`, `deleted_at` |
| `NoteContent(Id,SeriesId,SortOrder,Title?,Text,CreatedAt?,Type)` | `note_content` | `Type` → FK `type_id`; + `language`, `updated_at`, `deleted_at` |
| `NoteContentType(Id,Type)` | `note_content_type` | Без изменений (seed) |
| `TechTag(Id,Name,Description?,Type)` | `tech_tag` | `Type` → FK `type_id`; + `created_at/updated_at` |
| `TechTagType(Id,Type)` | `tech_tag_type` | Без изменений (seed) |
| `Task`, `ExperienceWork`, `ExperienceEducation` | — | **Не переносятся** в v1 (портфолио-часть, WBS `wont`) |

**Ключевые сдвиги парадигмы Portfolio → DEVNOTES:**
1. **ID**: EF-автоинкремент/GUID сервера → **клиентский UUID v7 строкой** (условие офлайн-создания).
2. **Время**: `CreatedAt?` (nullable, серверное) → **обязательные `created_at`+`updated_at`** UTC ISO 8601 у всех сущностей.
3. **Тип блока/тега**: enum/навигация EF → явный FK на seed-справочник в SQLite.
4. **Связи тегов**: неявные навигационные коллекции EF → явные таблицы-связки `note_series_tag`, `project_tag`.
5. **Новое для local-first**: `attachment`, `note_link`, `note_version`, `reminder`, `setting`, `sync_state`, `change_log`, `notes_fts` — в Portfolio отсутствуют, добавлены под офлайн, синк и мгновенный поиск.

### 9.1. Соответствие слоям (Clean Architecture)
| Слой | Portfolio | DEVNOTES |
| --- | --- | --- |
| Domain | C# entity-классы | Rust-структуры (PascalCase) / TS-типы (camelCase) |
| Interfaces | `IRepository<T>` | Rust-трейты репозиториев + IPC-контракты Tauri |
| Infrastructure (DB) | EF Core + провайдер БД | rusqlite + этот DDL + миграции |
| Infrastructure (Sync) | — (сервер) | Я.Диск REST (OAuth+PKCE), oplog `change_log` |
| UI | React + repository-pattern | Тот же фронт, TanStack Query + Zustand |

---

## 10. Перспектива (вне v1)

- **Company/Task/Experience**: возврат портфолио-части возможен во v2 — таблицы `company` уже готовы, `project.company_id` зарезервирован.
- **SQLCipher**: шифрование файла БД по мастер-паролю — схема совместима, меняется только слой открытия соединения.
- **Mobile (Tauri 2 iOS/Android)**: та же схема и тот же oplog-синк; отличий в модели данных не требуется.
- **Семантический поиск/AI-автотегирование**: добавление таблицы эмбеддингов `note_embedding(content_id, vector BLOB, model)` отдельной миграцией, не ломая ядро.

---

## 11. Чек-лист соответствия WBS `consistencyNotes`

| Пункт | Выполнено в этом документе |
| --- | --- |
| (1) Канонические имена сущностей | §2, §5 — строго `Project/NoteSeries/NoteContent/...` |
| (2) ID = UUID v7 строкой, клиентский | §1 (И-1), §9 |
| (3) UTC ISO 8601, `created_at`/`updated_at` обязательны | §1 (И-2,И-3), §5 DDL |
| (4) snake_case в схеме, домен Pascal/camel | §1 (И-4), §9.1 |
| (6) FTS5 external content + bm25 + триггеры, <50 мс | §6, §7.1 |
| (7) Синк через oplog `change_log`, LWW, файл БД не синкается | §2, §4.11, §3.1 |
| (8) Слои Domain/UseCases/Interfaces/Infrastructure/UI | §9.1 |
| (10) MVP без Company/Task/Experience | §2.1 (сноска 1), §9, §10 |
```
