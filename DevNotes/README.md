# DEVNOTES

> **Что это за файл.** Корневой README репозитория `DevNotes/` — точка входа в проект. Здесь суть продукта, ключевые возможности, технологический стек, карта репозитория со ссылками на всю проектную документацию (`docs/*`), быстрый старт и статус. Для деталей архитектуры, схемы БД, синка и дизайна — переходите по ссылкам из блока «Связанные документы».

> **Статус проекта:** `Проектирование / ТЗ готово`. Кодовой базы пока нет — репозиторий содержит проектную документацию. Реализация ведётся по [roadmap](docs/roadmap.md).

---

## Связанные документы

| Документ | О чём |
|---|---|
| [docs/vision.md](docs/vision.md) | Видение продукта, целевой пользователь, боль и сценарии использования |
| [docs/architecture.md](docs/architecture.md) | Слои Clean Architecture, обоснование Tauri vs Electron/MAUI/Flutter, IPC-контур |
| [docs/data-model.md](docs/data-model.md) | Доменная модель, ER-диаграмма, инварианты сущностей |
| [docs/database.md](docs/database.md) | SQLite-схема (DDL), миграции, FTS5-индекс, триггеры |
| [docs/search.md](docs/search.md) | Полнотекстовый поиск: FTS5 external content, bm25, снипеты, русская морфология |
| [docs/sync.md](docs/sync.md) | Синхронизация с Яндекс.Диском: OAuth 2.0 + PKCE, oplog, LWW, конфликт-копии |
| [docs/design-system.md](docs/design-system.md) | Дизайн-токены (HSL / shadcn), терминальная эстетика, компоненты, CVA |
| [docs/ui-ux.md](docs/ui-ux.md) | Экраны, командная палитра, горячие клавиши, навигация |
| [docs/roadmap.md](docs/roadmap.md) | MVP → Should → Could; вехи и порядок реализации |
| [docs/wbs.md](docs/wbs.md) | Согласованный WBS (must / should / could / wont), риски |

> Часть документов может быть в работе. Актуальность отслеживается в [roadmap](docs/roadmap.md).

---

## Суть проекта

**DEVNOTES** — кроссплатформенное **десктоп-приложение** для инженерных заметок по технологиям разработки. Local-first «терминал инженерных знаний»: заметки группируются по проектам и темам, доступен мгновенный полнотекстовый поиск по всей базе, а бэкап/синхронизация идут через Яндекс.Диск.

**Для кого.** Разработчик (в первую очередь Full-Stack .NET/React), который ведёт личную базу знаний: сниппеты, разборы багов, конспекты технологий, заметки по проектам. Ему нужна скорость, работа офлайн и полный контроль над данными — без облачного SaaS-замка.

**Какую боль решает.**

- **Разрозненность.** Заметки расползаются по Notion / Obsidian / файлам / issue-трекерам без единой структуры «проект → тема → блок».
- **Медленный поиск.** По растущей базе поиск тормозит; нужен `<50 мс` на 10k блоков.
- **Онлайн-зависимость.** Облачные редакторы требуют сети и отдают данные наружу. DEVNOTES работает **полностью офлайн**, данные лежат в локальном SQLite.
- **Хрупкий бэкап.** Ручное копирование теряется. Здесь — журналируемый синк (oplog) + снапшоты на Яндекс.Диск с разрешением конфликтов.

**Ключевой архитектурный принцип — local-first.** Приложение работает без сети; облако — это канал бэкапа и синхронизации между устройствами, а не источник истины. По облаку **никогда не синкается «живой» файл БД** (это гарантированная потеря данных при двух устройствах) — синкается только журнал изменений `ChangeLog` (oplog) и периодические снапшоты.

---

## Ключевые возможности

- **Иерархия знаний.** `Project` → `NoteSeries` → `NoteContent`. Блоки контента типов **markdown / code / image / link**, сортировка drag-and-drop (`@dnd-kit`, поле `sort_order`).
- **Мгновенный поиск.** SQLite **FTS5** (external content над `NoteContent` + `NoteSeries`), ранжирование **bm25**, подсветка снипетов. Цель — `<50 мс` на 10k блоков.
- **Командная палитра `Ctrl/Cmd+K`.** Поиск, переход к проекту/серии, быстрые действия.
- **Теги технологий.** `TechTag` с категориями `TechTagType` (язык / фреймворк / инструмент / БД / DevOps); фильтрация списков по тегу и проекту.
- **Синк с Яндекс.Диском.** OAuth 2.0 Authorization Code + **PKCE** через loopback-redirect, область `app folder`; токены — в системном keychain. Офлайн-очередь изменений, конфликты — **LWW по `updated_at`** на уровне блока + конфликт-копия.
- **Markdown + код.** Рендер `react-markdown`, подсветка `react-syntax-highlighter`.
- **Обязательные даты.** У каждой сущности `created_at` и `updated_at` — хранение UTC ISO 8601, отображение в локальной таймзоне.
- **Терминальная тема.** Тёмная по умолчанию + светлая; HSL-токены в стиле shadcn/ui, `JetBrains Mono`, акцент `#22c55e`, terminal window, scanlines.

**В планах (Should / Could):** экспорт серии в Markdown/PDF, вложения-изображения (content-addressable), локальные снапшоты (`VACUUM INTO`), избранное/архив, автосохранение черновиков, шаблоны серий, автообновление (Tauri Updater), версионирование блоков, wiki-links, шифрование БД (SQLCipher), quick-capture, WebDAV-каналы, AI-фичи. Подробности — в [roadmap](docs/roadmap.md) и [wbs](docs/wbs.md).

---

## Технологический стек

```text
[ SHELL ] Tauri 2.0 · Rust-core        [ FRONT ] React 19 · TypeScript · Vite 6
[ DB    ] SQLite · rusqlite · FTS5·bm25 [ STATE ] Zustand · TanStack Query
[ SYNC  ] Яндекс.Диск REST · OAuth2·PKCE [ UI   ] Tailwind · CVA · clsx · tw-merge
[ DND   ] @dnd-kit                      [ MD   ] react-markdown · syntax-highlighter
[ ARCH  ] Clean Architecture (Rust+TS)  [ FONT ] JetBrains Mono · accent #22c55e
```

| Слой | Технологии |
|---|---|
| **Оболочка** | Tauri 2.0 (Rust-ядро), сборки Windows / macOS / Linux (в перспективе mobile) |
| **Ядро (Rust)** | тонкий слой: SQL (`rusqlite`), FTS5-индексация, синк, IPC-команды |
| **Хранилище** | SQLite + FTS5 (external content, `bm25`); миграции версионируются |
| **Синхронизация** | Яндекс.Диск Cloud REST API, OAuth 2.0 + PKCE, oplog (`ChangeLog`), LWW |
| **Фронтенд** | React 19, TypeScript, Vite 6, React Router 7 |
| **Состояние/данные** | Zustand + TanStack Query, repository-pattern + генераторы query-key |
| **UI/дизайн** | Tailwind, дизайн-токены HSL (shadcn/ui), CVA + clsx + tailwind-merge |
| **Контент** | `@dnd-kit`, react-markdown, react-syntax-highlighter |

> Обоснование выбора **Tauri 2.0** (vs Electron / .NET MAUI / Flutter) — в [docs/architecture.md](docs/architecture.md). Решение зафиксировано.

---

## Архитектура (обзор)

Слои по мотивам Clean Architecture из Portfolio, зеркально на Rust-ядре и на фронте:

```mermaid
flowchart TB
    subgraph UI["UI · React 19 + TS"]
        V[Компоненты / экраны]
        S[Zustand + TanStack Query]
        R[Repository + query-key]
    end
    subgraph CORE["Rust-ядро · Tauri"]
        UC[UseCases]
        IF[Interfaces]
        INFRA[Infrastructure]
        DB[(SQLite + FTS5)]
        SYNC[Sync · Яндекс.Диск]
    end
    V --> S --> R -->|IPC invoke| UC
    UC --> IF --> INFRA
    INFRA --> DB
    INFRA --> SYNC
    SYNC -->|oplog + snapshot| YD[(Я.Диск app folder)]
```

Границы: вся бизнес-логика UI остаётся в TypeScript; Rust-слой намеренно тонкий (SQL + sync + IPC) — это снижает кривую обучения для .NET/React-команды. Детали и IPC-контракты — в [docs/architecture.md](docs/architecture.md).

---

## Доменная модель (кратко)

```mermaid
erDiagram
    Project ||--o{ NoteSeries : "содержит"
    NoteSeries ||--o{ NoteContent : "содержит блоки"
    NoteContent }o--|| NoteContentType : "тип"
    NoteSeries }o--o{ TechTag : "NoteSeriesTag"
    TechTag }o--|| TechTagType : "категория"
    NoteContent ||--o{ Attachment : "вложения"
    ChangeLog }o--|| Project : "op-журнал (любая сущность)"

    Project { string id PK "UUID v7" string name bool archived string created_at string updated_at }
    NoteSeries { string id PK string project_id FK string title bool pinned string created_at string updated_at }
    NoteContent { string id PK string series_id FK int sort_order string text string type string created_at string updated_at }
```

Инварианты (обязательны во всех документах):

- **Имена сущностей строго:** `Project` / `NoteSeries` / `NoteContent` / `NoteContentType` / `TechTag` / `TechTagType` / `Attachment` / `ChangeLog` / `SyncState`. Без синонимов: «тема» = `NoteSeries`, «блок» = `NoteContent`.
- **ID = UUID v7 (строка), генерируется клиентом** — обязательное условие офлайн-создания и синка.
- **Даты — UTC ISO 8601 в БД**; поля `created_at` / `updated_at` обязательны у всех сущностей.
- **SQLite: snake_case** в схеме; домен на Rust — PascalCase, на TS — camelCase.

Полная модель, справочники и связи — в [docs/data-model.md](docs/data-model.md); DDL — в [docs/database.md](docs/database.md).

---

## Структура репозитория

```text
DevNotes/
├── README.md                  # этот файл — точка входа
├── docs/                      # проектная документация
│   ├── vision.md              # видение, пользователь, боль
│   ├── architecture.md        # слои, обоснование Tauri, IPC
│   ├── data-model.md          # доменная модель, ER-диаграмма
│   ├── database.md            # SQLite DDL, миграции, FTS5, триггеры
│   ├── search.md              # FTS5 / bm25 / снипеты / морфология
│   ├── sync.md                # Яндекс.Диск, OAuth2+PKCE, oplog, LWW
│   ├── design-system.md       # HSL-токены, терминальная эстетика, CVA
│   ├── ui-ux.md               # экраны, командная палитра, хоткеи
│   ├── roadmap.md             # MVP → Should → Could, вехи
│   └── wbs.md                 # согласованный WBS + риски
│
└── (планируется при реализации)
    ├── src-tauri/             # Rust-ядро: Domain/UseCases/Interfaces/Infrastructure
    │   ├── src/
    │   ├── migrations/        # версионируемые миграции SQLite
    │   └── tauri.conf.json
    ├── src/                   # React 19 + TS фронтенд (дизайн-система Portfolio)
    │   ├── domain/            # типы, сущности
    │   ├── repositories/      # repository-pattern + query-key генераторы
    │   ├── features/          # проекты, серии, блоки, поиск, синк
    │   ├── components/ui/     # Button/Card/Input/Badge (CVA)
    │   └── styles/            # HSL-токены, terminal-тема
    ├── package.json
    └── vite.config.ts
```

---

## Быстрый старт

> ⚠️ Проект на стадии проектирования — команды ниже **плейсхолдеры** и активируются по мере реализации (см. [roadmap](docs/roadmap.md)).

**Предпосылки (планируемые):**

- Node.js `>= 20` и `pnpm` (или `npm`)
- Rust `stable` (toolchain для Tauri 2.0)
- Системные зависимости WebView: WebView2 (Windows) / WebKit (macOS) / WebKitGTK (Linux)

```bash
# 1. Клонировать
git clone <repo-url> && cd DevNotes

# 2. Установить зависимости фронтенда
pnpm install                # placeholder

# 3. Запуск в dev-режиме (Tauri + Vite)
pnpm tauri dev              # placeholder

# 4. Продакшн-сборка под текущую платформу
pnpm tauri build           # placeholder
```

Локальная БД SQLite создаётся при первом запуске в пользовательской app-директории; миграции применяются автоматически. Настройка синка с Яндекс.Диском — в [docs/sync.md](docs/sync.md).

---

## Скриншоты

> _Место под скриншоты. Будут добавлены после реализации UI (терминальная тема, командная палитра, редактор блоков, панель поиска)._

```text
┌────────────────────────────────────────────────────────────┐
│  ● ● ●   devnotes — terminal of engineering knowledge        │
├────────────────────────────────────────────────────────────┤
│  > [ скриншот: список проектов / серий ]                     │
│  > [ скриншот: редактор блоков + markdown-превью ]           │
│  > [ скриншот: Ctrl+K командная палитра / FTS5-поиск ]       │
│  > [ скриншот: индикатор синка с Яндекс.Диском ]             │
└────────────────────────────────────────────────────────────┘
```

---

## Статус

| Аспект | Состояние |
|---|---|
| Концепция и ТЗ | ✅ Готово (WBS согласован) |
| Проектная документация `docs/*` | 🟡 В работе |
| Rust-ядро / фронтенд | ⬜ Не начато |
| MVP (must-have) | ⬜ По roadmap |

**Резюме:** _Проектирование / ТЗ готово, реализация — по [roadmap](docs/roadmap.md)._

---

## Источник концепций

Модель данных, дизайн-язык и часть фронтенд-стека перенесены из веб-проекта **Portfolio** (`Mafiozist/Portfolio`, .NET + React): Clean Architecture, сущности `Project/NoteSeries/NoteContent/TechTag`, терминально-хакерская эстетика (JetBrains Mono, акцент `#22c55e`, scanlines), примитивы UI на CVA. Портфолио-часть (`Company`, `Task`, `ExperienceWork/Education`) в десктоп v1 **не переносится** — только упоминается в разделах «перспектива».

---

_Все документы — на русском. Тон — инженерный, конкретный. Единый глоссарий и инварианты см. в [docs/wbs.md](docs/wbs.md) (раздел consistencyNotes)._
