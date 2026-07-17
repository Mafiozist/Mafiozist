# 11 — Реестр файлов (File Index)

> **Что это за файл.** Кросс-справочник по каждому файлу репозитория DEVNOTES: путь, назначение, что можно менять, зависимости. Обновляется при любом создании/удалении/переименовании файла (конвенция [`../CLAUDE.md`](../CLAUDE.md) §3.1). Реализован MVP — ниже перечислены фактические файлы ядра (`core/`), оболочки (`src-tauri/`) и фронтенда (`app/`); нереализованное вынесено в раздел «Планируемое».

## Связанные документы

- [`00-INDEX.md`](00-INDEX.md) — оглавление и маршруты чтения.
- [`../CLAUDE.md`](../CLAUDE.md) — карта репозитория и конвенции.
- [`04-ARCHITECTURE.md`](04-ARCHITECTURE.md) — слои, отражённые в структуре каталогов.

## Формат записи

Для каждого файла: **Путь** · **Назначение** (1 строка) · **Можно менять** (что править безопасно / что требует ADR) · **Зависимости** (от чего зависит / что зависит от него). Для документов «зависимости» = смысловые кросс-ссылки.

---

## Существующие файлы (факт)

### Корень `DevNotes/`

| Путь | Назначение | Можно менять | Зависимости |
| --- | --- | --- | --- |
| `README.md` | Суть проекта для внешнего читателя: что, зачем, стек, структура | Свободно (описание) | Ссылается на `docs/*` |
| `CLAUDE.md` | Состояние проекта, конвенции, роли, DoD, журнал | Разделы обновляются по правилам §3.4/§3.8; решения §1 — через ADR | Источник правды; на него ссылаются все документы |

### Документация `DevNotes/docs/`

| Путь | Назначение | Можно менять | Зависимости |
| --- | --- | --- | --- |
| `docs/00-INDEX.md` | Оглавление документации, маршруты чтения, карта связей | При добавлении/удалении документа | Ссылается на все `docs/*`, `../CLAUDE.md`, `../README.md` |
| `docs/01-VISION.md` | Видение, персоны, сценарии, scope | При изменении продуктового видения | → `02-SPECIFICATION` |
| `docs/02-SPECIFICATION.md` | **Большое ТЗ**: требования, user stories, критерии приёмки | При изменении требований (согласовать с fable) | ← `01-VISION`; → `03..09`, `13` |
| `docs/03-FEATURES.md` | Каталог фич (MoSCoW) | При пере-приоритизации объёма | ↔ `02-SPECIFICATION`, `10-ROADMAP` |
| `docs/04-ARCHITECTURE.md` | Архитектура Tauri 2 + React, слои, IPC, sync | Значимые решения — через ADR | → `05`, `06`, `07`, `08`, `09` |
| `docs/05-DATA-MODEL.md` | Сущности, ER, SQLite DDL, FTS5, миграции, инварианты | Схема — через миграции; менять синхронно с кодом БД | ← `04`; → `08-SEARCH`, `13-TESTING` |
| `docs/06-UI-UX.md` | Дизайн-токены, терминал-эстетика, экраны, тренды | Токены/компоненты — согласованно с реализацией `components/ui` | ← `04`; связан с Portfolio-дизайном |
| `docs/07-TECH-STACK.md` | Обоснование стека, версии, дистрибуция, CI | При смене инструментов — через ADR | ← `04`; → `13-TESTING` (CI) |
| `docs/08-SEARCH.md` | FTS5 external content, bm25, синтаксис, перф-SLA | Синхронно со схемой поиска в `05` и кодом | ← `05`; → `13-TESTING` |
| `docs/09-YANDEX-DISK.md` | OAuth 2.0 + PKCE, oplog, конфликты, офлайн-очередь | Синхронно с sync-движком | ← `04`; → `13-TESTING` |
| `docs/10-ROADMAP.md` | Релизы MVP→v2, вехи, риски | При перепланировании | ← `03-FEATURES` |
| `docs/11-FILE-INDEX.md` | Этот реестр файлов | При любом изменении состава файлов | Ссылается на весь репозиторий |
| `docs/12-GLOSSARY.md` | Канонические определения терминов | При появлении нового термина | Термины используют все документы |
| `docs/13-TESTING.md` | Стратегия unit + интеграционных тестов, CI | При изменении тест-подхода | ← `02`, `04`, `05`, `08`, `09` |
| `docs/WORKLOG.md` | Журнал мульти-агентных прогонов | Только дополняется (append) | Отражается в журнале `../CLAUDE.md` §7 |

---

## Код: ядро `core/` (реализовано, `devnotes-core`)

| Путь | Назначение | Слой | Можно менять |
| --- | --- | --- | --- |
| `core/Cargo.toml` | Манифест крейта ядра, зависимости (rusqlite bundled, uuid, time…) | — | зависимости — осознанно |
| `core/migrations/001_init.sql` | Схема БД + FTS5 external content + триггеры индекса | Infrastructure/DB | только через новую миграцию 002+ |
| `core/src/lib.rs` | Объявление модулей + `CoreError`/`Result` | — | при добавлении модуля |
| `core/src/domain.rs` | Сущности Project/NoteSeries/NoteContent, типы блоков, SearchHit | Domain | синхронно со схемой |
| `core/src/ports.rs` | Порты `Clock`/`IdGenerator` + `SystemClock`/`UuidV7Generator` | Interfaces | добавление портов |
| `core/src/sqlite.rs` | `SqliteStore`: миграции, CRUD, каскады, reorder, FTS5-поиск | Infrastructure/DB | + методы; SQL синхронно со схемой |
| `core/src/search.rs` | `to_fts_query` — безопасный FTS5-запрос из ввода | Domain/логика | правила токенизации |
| `core/src/sync.rs` | `resolve` — LWW-разрешение конфликтов (чистая логика) | Domain/логика | стратегия конфликтов |
| `core/src/service.rs` | `NotesService` — сценарии с валидацией | UseCases | бизнес-правила |
| `core/tests/integration.rs` | 9 интеграционных тестов (реальный SQLite + FTS5) | Тесты | добавлять сценарии |

## Код: оболочка `src-tauri/` (реализовано, Tauri 2)

| Путь | Назначение | Можно менять |
| --- | --- | --- |
| `src-tauri/Cargo.toml` | Манифест оболочки; зависит от `devnotes-core` по пути | зависимости |
| `src-tauri/build.rs` | Скрипт сборки Tauri | обычно нет |
| `src-tauri/tauri.conf.json` | Окно, CSP, bundle, dev/build-команды фронта | конфигурация окна/бандла |
| `src-tauri/capabilities/default.json` | Разрешения (permissions) главного окна | при добавлении плагинов |
| `src-tauri/src/lib.rs` | IPC-команды → `NotesService`; инициализация БД | + команды (тонкий слой) |
| `src-tauri/src/main.rs` | Точка входа десктопа | обычно нет |
| `src-tauri/icons/*` | Иконки бандла — **нужно добавить** перед сборкой | добавить ассеты |

## Код: фронтенд `app/` (реализовано, React 19 + Vite)

| Путь | Назначение | Можно менять |
| --- | --- | --- |
| `app/package.json` | Скрипты (dev/build/typecheck/test) и зависимости | зависимости/скрипты |
| `app/vite.config.ts`, `app/vitest.config.ts` | Конфигурация сборки и тестов | настройки сборки |
| `app/tailwind.config.ts`, `app/postcss.config.js` | Tailwind + дизайн-токены | токены/тема |
| `app/tsconfig.json` | Конфигурация TypeScript | опции компилятора |
| `app/index.html` | HTML-обёртка (тема dark по умолчанию) | метаданные |
| `app/src/main.tsx`, `app/src/App.tsx` | Точка входа + трёхпанельная раскладка + хоткей поиска | композиция экранов |
| `app/src/app/providers.tsx` | Провайдер TanStack Query | настройки кэша |
| `app/src/domain/types.ts` | Зеркало доменных типов (TS) | синхронно с `core/domain.rs` |
| `app/src/api/notes.ts` | Repository-обёртки над IPC-командами | + вызовы |
| `app/src/api/queryKeys.ts` | Генераторы query-key | ключи кэша |
| `app/src/lib/cn.ts` | Утилита классов (clsx + tailwind-merge) | нет |
| `app/src/lib/ipc.ts` | IPC-обёртка Tauri + браузерный мок бэкенда | мок синхронно с командами |
| `app/src/lib/ipc.test.ts` | Vitest-тесты мока (5 тестов) | добавлять сценарии |
| `app/src/stores/uiStore.ts` | Zustand: выбор проекта/серии, открытие поиска | UI-состояние |
| `app/src/components/ui/*` | Примитивы дизайн-системы (Button/Card/Input/TextArea/Badge) | варианты/стили |
| `app/src/features/projects/ProjectSidebar.tsx` | Левая панель: проекты | UI/логика |
| `app/src/features/series/SeriesList.tsx` | Средняя панель: серии | UI/логика |
| `app/src/features/content/BlockEditor.tsx` | Правая панель: блоки + добавление | UI/логика |
| `app/src/features/search/SearchPalette.tsx` | Командная палитра поиска (Ctrl/Cmd+K) + фильтр по тегам | UI/логика |
| `app/src/features/tags/TagSelector.tsx` | Селектор тегов: дропдаун с поиском/чипами/созданием | UI/логика |
| `app/src/features/tags/SeriesTagsBar.tsx` | Панель тегов серии (показ + назначение) | UI/логика |
| `app/src/styles/tokens.css` | Дизайн-токены HSL (терминальная тема) | тема/цвета |

## Код: корень репозитория

| Путь | Назначение | Можно менять |
| --- | --- | --- |
| `.github/workflows/devnotes-ci.yml` | CI: тесты ядра (fmt/clippy/test) + фронтенда (typecheck/test/build) | шаги пайплайна |

## Планируемое (по `10-ROADMAP.md`, ещё не реализовано)

| Путь (план) | Назначение |
| --- | --- |
| `core/src/infrastructure/sync/` или `core/src/yandex.rs` | Клиент Яндекс.Диска, oplog, разрешение конфликтов (v0.3) |
| `core/migrations/002_*.sql` | Версии/вложения/wiki-links/backlinks (v1.0) |
| `app/src/features/sync/`, `app/src/features/settings/` | UI синхронизации и настроек |
| `app/e2e/` | Playwright e2e (по мере готовности UI) |
