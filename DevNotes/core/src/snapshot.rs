//! Снапшот состояния БД для синхронизации/бэкапа.
//!
//! ПОЧЕМУ снапшот, а не «живой» файл БД: переносим и сливаем данные по сущностям
//! с LWW-разрешением конфликтов (см. `sync`), а не перезаписываем файл целиком —
//! иначе при двух устройствах гарантированная потеря данных (см. `docs/09-YANDEX-DISK.md`).

use serde::{Deserialize, Serialize};

use crate::domain::{NoteContent, NoteSeries, Project, TechTag, TechTagType};

/// Полный слепок пользовательских данных для обмена между устройствами.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub projects: Vec<Project>,
    pub series: Vec<NoteSeries>,
    pub contents: Vec<NoteContent>,
    pub tag_types: Vec<TechTagType>,
    pub tags: Vec<TechTag>,
    /// Привязки серия ↔ тег как пары (series_id, tag_id).
    pub series_tags: Vec<(String, String)>,
}
