// Зеркало доменных типов ядра (Rust devnotes-core) на стороне фронтенда.
// Поля camelCase соответствуют serde-сериализации; `content_type` в Rust помечен
// #[serde(rename = "type")], поэтому здесь поле называется `type`.
// См. docs/05-DATA-MODEL.md.

/** Проект — верхнеуровневая группа заметок. */
export interface Project {
  id: string;
  name: string;
  short_name: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}

/** Серия (тема) заметок. */
export interface NoteSeries {
  id: string;
  project_id: string | null;
  title: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

/** Допустимые типы блока контента. */
export type ContentType = "markdown" | "code" | "image" | "link";

/** Блок контента внутри серии. */
export interface NoteContent {
  id: string;
  series_id: string;
  sort_order: number;
  title: string | null;
  text: string;
  type: ContentType;
  created_at: string;
  updated_at: string;
}

/** Категория тега технологий (язык / фреймворк / инструмент …). */
export interface TechTagType {
  id: string;
  type: string;
}

/** Тег технологии, которым помечаются серии заметок. */
export interface TechTag {
  id: string;
  name: string;
  description: string | null;
  type_id: string | null;
  /** Название категории (join из tech_tag_type). */
  typeName: string | null;
}

/** Результат полнотекстового поиска. */
export interface SearchHit {
  content_id: string;
  series_id: string;
  title: string | null;
  snippet: string;
  rank: number;
}

/** Итог синхронизации со снапшотом. */
export interface SyncReport {
  applied: number;
  uploaded_bytes: number;
}
