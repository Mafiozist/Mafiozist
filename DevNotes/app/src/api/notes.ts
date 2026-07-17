// Типизированные обёртки над командами бэкенда (repository-слой).
// Имена аргументов соответствуют camelCase-параметрам Tauri-команд.
import { invoke } from "@/lib/ipc";
import type { ContentType, NoteContent, NoteSeries, Project, SearchHit } from "@/domain/types";

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

  // Поиск
  search: (query: string, limit = 50) => invoke<SearchHit[]>("search", { query, limit }),
};
