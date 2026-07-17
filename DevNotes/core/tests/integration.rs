//! Интеграционные тесты ядра DEVNOTES: реальный SQLite (в памяти) + FTS5.
//!
//! Проверяют слои хранилища и сценариев вместе с настоящей БД и триггерами индекса.
//! Детерминизм обеспечивают тест-двойники [`FixedClock`] и [`SeqIds`] (порты из `ports`).
//! См. `docs/13-TESTING.md`.

use std::sync::atomic::{AtomicU32, Ordering};

use devnotes_core::domain::content_type;
use devnotes_core::ports::{Clock, IdGenerator};
use devnotes_core::service::NotesService;
use devnotes_core::sqlite::SqliteStore;
use devnotes_core::CoreError;

/// Часы, выдающие строго возрастающие метки времени — детерминированный порядок сортировки.
struct FixedClock {
    counter: AtomicU32,
}
impl FixedClock {
    fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }
}
impl Clock for FixedClock {
    fn now_rfc3339(&self) -> String {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        // Возрастающие секунды → однозначный порядок created_at.
        format!("2026-07-17T10:{:02}:{:02}Z", n / 60, n % 60)
    }
}

/// Последовательный генератор id — предсказуемые идентификаторы в тестах.
struct SeqIds {
    counter: AtomicU32,
}
impl SeqIds {
    fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }
}
impl IdGenerator for SeqIds {
    fn new_id(&self) -> String {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("id-{n:06}")
    }
}

/// Хелпер: сервис на свежей БД в памяти с применёнными миграциями.
fn service() -> NotesService {
    let store = SqliteStore::open_in_memory(Box::new(FixedClock::new()), Box::new(SeqIds::new()))
        .expect("open in-memory db");
    store.migrate().expect("run migrations");
    NotesService::new(store)
}

#[test]
fn creates_and_lists_projects() {
    let svc = service();
    svc.create_project("Rust CLI", Some("rcli"), None).unwrap();
    svc.create_project("Web App", None, Some("frontend"))
        .unwrap();

    let projects = svc.list_projects().unwrap();
    assert_eq!(projects.len(), 2);
    // Новые — первыми (created_at DESC): последний созданный «Web App» вверху.
    assert_eq!(projects[0].name, "Web App");
}

#[test]
fn empty_project_name_is_rejected() {
    let svc = service();
    let err = svc.create_project("   ", None, None).unwrap_err();
    assert!(matches!(err, CoreError::Invalid(_)));
}

#[test]
fn content_blocks_keep_insertion_order() {
    let svc = service();
    let s = svc.create_series(None, "Async in Rust", None).unwrap();
    svc.add_content(&s.id, Some("intro"), "первый блок", content_type::MARKDOWN)
        .unwrap();
    svc.add_content(&s.id, None, "второй блок", content_type::CODE)
        .unwrap();

    let blocks = svc.list_content(&s.id).unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].sort_order, 0);
    assert_eq!(blocks[1].sort_order, 1);
    assert_eq!(blocks[0].text, "первый блок");
}

#[test]
fn invalid_content_type_is_rejected() {
    let svc = service();
    let s = svc.create_series(None, "S", None).unwrap();
    let err = svc
        .add_content(&s.id, None, "x", "spreadsheet")
        .unwrap_err();
    assert!(matches!(err, CoreError::Invalid(_)));
}

#[test]
fn fts5_search_finds_updates_and_deletes() {
    let svc = service();
    let s = svc.create_series(None, "Tokio", None).unwrap();
    let c = svc
        .add_content(
            &s.id,
            Some("spawn"),
            "tokio::spawn запускает задачу",
            content_type::CODE,
        )
        .unwrap();

    // Найдено по слову и по префиксу.
    let hits = svc.search("tokio", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].content_id, c.id);
    assert!(
        svc.search("spaw", 10).unwrap().len() == 1,
        "префиксный поиск должен находить"
    );

    // После обновления текст переиндексируется триггером.
    svc.update_content(
        &c.id,
        Some("spawn"),
        "переписали на async-std",
        content_type::CODE,
    )
    .unwrap();
    assert_eq!(
        svc.search("tokio", 10).unwrap().len(),
        0,
        "старый термин исчез из индекса"
    );
    assert_eq!(
        svc.search("async", 10).unwrap().len(),
        1,
        "новый термин появился"
    );

    // После удаления блок исчезает из индекса.
    svc.delete_content(&c.id).unwrap();
    assert_eq!(svc.search("async", 10).unwrap().len(), 0);
}

#[test]
fn search_ranks_title_matches_higher() {
    let svc = service();
    let s = svc.create_series(None, "Ranking", None).unwrap();
    // «postgres» в тексте.
    svc.add_content(
        &s.id,
        Some("misc"),
        "немного про postgres в тексте",
        content_type::MARKDOWN,
    )
    .unwrap();
    // «postgres» в заголовке (вес заголовка выше — bm25 5.0 vs 1.0).
    let title_hit = svc
        .add_content(
            &s.id,
            Some("postgres"),
            "тело без ключевого слова",
            content_type::MARKDOWN,
        )
        .unwrap();

    let hits = svc.search("postgres", 10).unwrap();
    assert_eq!(hits.len(), 2);
    // Меньший rank = релевантнее; совпадение в заголовке должно быть первым.
    assert_eq!(hits[0].content_id, title_hit.id);
}

#[test]
fn reorder_changes_sort_order() {
    let svc = service();
    let s = svc.create_series(None, "Reorder", None).unwrap();
    let a = svc
        .add_content(&s.id, None, "A", content_type::MARKDOWN)
        .unwrap();
    let b = svc
        .add_content(&s.id, None, "B", content_type::MARKDOWN)
        .unwrap();

    svc.reorder_content(&[&b.id, &a.id]).unwrap();

    let blocks = svc.list_content(&s.id).unwrap();
    assert_eq!(blocks[0].id, b.id, "B стал первым после переупорядочивания");
    assert_eq!(blocks[1].id, a.id);
}

#[test]
fn deleting_project_cascades_to_series_and_content() {
    let svc = service();
    let p = svc.create_project("P", None, None).unwrap();
    let s = svc.create_series(Some(&p.id), "S", None).unwrap();
    svc.add_content(
        &s.id,
        None,
        "уникальное_слово_каскад",
        content_type::MARKDOWN,
    )
    .unwrap();

    assert_eq!(svc.search("уникальное_слово_каскад", 10).unwrap().len(), 1);

    svc.delete_project(&p.id).unwrap();

    assert!(
        svc.list_series(Some(&p.id)).unwrap().is_empty(),
        "серии удалены каскадом"
    );
    assert_eq!(
        svc.search("уникальное_слово_каскад", 10).unwrap().len(),
        0,
        "блоки удалены каскадом и вычищены из FTS-индекса"
    );
}

#[test]
fn deleting_missing_content_returns_not_found() {
    let svc = service();
    let err = svc.delete_content("id-does-not-exist").unwrap_err();
    assert!(matches!(err, CoreError::NotFound));
}
