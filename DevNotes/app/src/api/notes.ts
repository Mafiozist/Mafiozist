// Типизированные обёртки над командами бэкенда (repository-слой).
// Имена аргументов соответствуют camelCase-параметрам Tauri-команд.
import { invoke } from "@/lib/ipc";
import type {
  ContentType,
  NoteContent,
  NoteSeries,
  Project,
  SearchHit,
  SyncReport,
  TechTag,
  TechTagType,
} from "@/domain/types";

export const notesApi = {
  // Проекты
  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (name: string, shortName?: string, description?: string) =>
    invoke<Project>("create_project", { name, shortName, description }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  // Серии
  listSeries: (projectId: string | null) =>
    invoke<NoteSeries[]>("list_series", { projectId }),
  createSeries: (projectId: string | null, title: string, description?: string) =>
    invoke<NoteSeries>("create_series", { projectId, title, description }),
  deleteSeries: (id: string) => invoke<void>("delete_series", { id }),

  // Блоки
  listContent: (seriesId: string) => invoke<NoteContent[]>("list_content", { seriesId }),
  addContent: (seriesId: string, title: string | null, text: string, contentType: ContentType) =>
    invoke<NoteContent>("add_content", { seriesId, title, text, contentType }),
  updateContent: (id: string, title: string | null, text: string, contentType: ContentType) =>
    invoke<void>("update_content", { id, title, text, contentType }),
  deleteContent: (id: string) => invoke<void>("delete_content", { id }),
  reorderContent: (orderedIds: string[]) => invoke<void>("reorder_content", { orderedIds }),

  // Теги технологий
  listTags: () => invoke<TechTag[]>("list_tags"),
  createTag: (name: string, description?: string, typeId?: string) =>
    invoke<TechTag>("create_tag", { name, description, typeId }),
  deleteTag: (id: string) => invoke<void>("delete_tag", { id }),
  listTagTypes: () => invoke<TechTagType[]>("list_tag_types"),
  createTagType: (name: string) => invoke<TechTagType>("create_tag_type", { name }),
  listTagsForSeries: (seriesId: string) =>
    invoke<TechTag[]>("list_tags_for_series", { seriesId }),
  setSeriesTags: (seriesId: string, tagIds: string[]) =>
    invoke<void>("set_series_tags", { seriesId, tagIds }),

  // Поиск (опциональный фильтр по тегам технологий)
  search: (query: string, tagIds: string[] = [], limit = 50) =>
    invoke<SearchHit[]>("search", { query, tagIds, limit }),

  // Синхронизация: обмен снапшотом через файл (напр. в папке Яндекс.Диска)
  syncFile: (path: string) => invoke<SyncReport>("sync_file", { path }),
};
