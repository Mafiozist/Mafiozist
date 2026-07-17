// Обёртка над IPC Tauri с браузерным моком.
//
// ПОЧЕМУ мок: в обычном браузере (vite dev/preview, тесты) нет Tauri-бэкенда.
// Тогда используется in-memory реализация с той же семантикой команд, что и
// Rust-ядро, — приложение работает автономно для разработки UI.
// В десктопе (Tauri) вызовы идут в настоящие команды src-tauri/src/lib.rs.

import type { NoteContent, NoteSeries, Project, SearchHit, ContentType } from "@/domain/types";

/** Признак запуска внутри Tauri (иначе — браузерный мок). */
function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Универсальный вызов команды бэкенда. */
export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri()) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(command, args);
  }
  return mock.dispatch<T>(command, args ?? {});
}

// --- Браузерный мок бэкенда ------------------------------------------------

function nowIso(): string {
  return new Date().toISOString();
}
function uid(): string {
  return crypto.randomUUID();
}

/** In-memory реализация команд, зеркалящая поведение ядра. */
class MockBackend {
  private projects: Project[] = [];
  private series: NoteSeries[] = [];
  private contents: NoteContent[] = [];

  constructor() {
    // Небольшой демонстрационный набор, чтобы UI не был пустым в браузере.
    const p = this.createProject("Демо-проект", "demo", "Пример данных для браузерного режима");
    const s = this.createSeries(p.id, "Rust + SQLite", "Заметки по ядру DevNotes");
    this.addContent(s.id, "FTS5", "SQLite FTS5 даёт полнотекстовый поиск с bm25.", "markdown");
    this.addContent(s.id, "Пример", "tokio::spawn запускает асинхронную задачу.", "code");
  }

  dispatch<T>(command: string, a: Record<string, unknown>): Promise<T> {
    const r = (v: unknown) => Promise.resolve(v as T);
    switch (command) {
      case "list_projects":
        return r([...this.projects].reverse());
      case "create_project":
        return r(this.createProject(a.name as string, a.shortName as string | undefined, a.description as string | undefined));
      case "delete_project":
        return r(this.deleteProject(a.id as string));
      case "list_series":
        return r(this.listSeries((a.projectId as string | undefined) ?? null));
      case "create_series":
        return r(this.createSeries((a.projectId as string | null) ?? null, a.title as string, a.description as string | undefined));
      case "delete_series":
        return r(this.deleteSeries(a.id as string));
      case "list_content":
        return r(this.listContent(a.seriesId as string));
      case "add_content":
        return r(this.addContent(a.seriesId as string, (a.title as string | undefined) ?? null, a.text as string, a.contentType as ContentType));
      case "update_content":
        return r(this.updateContent(a.id as string, (a.title as string | undefined) ?? null, a.text as string, a.contentType as ContentType));
      case "delete_content":
        return r(this.deleteContent(a.id as string));
      case "reorder_content":
        return r(this.reorder(a.orderedIds as string[]));
      case "search":
        return r(this.search(a.query as string, (a.limit as number | undefined) ?? 50));
      default:
        return Promise.reject(new Error(`mock: неизвестная команда ${command}`));
    }
  }

  private createProject(name: string, shortName?: string, description?: string): Project {
    const now = nowIso();
    const project: Project = {
      id: uid(),
      name,
      short_name: shortName ?? null,
      description: description ?? null,
      created_at: now,
      updated_at: now,
    };
    this.projects.push(project);
    return project;
  }

  private deleteProject(id: string): void {
    this.projects = this.projects.filter((p) => p.id !== id);
    const seriesIds = this.series.filter((s) => s.project_id === id).map((s) => s.id);
    this.series = this.series.filter((s) => s.project_id !== id);
    this.contents = this.contents.filter((c) => !seriesIds.includes(c.series_id));
  }

  private listSeries(projectId: string | null): NoteSeries[] {
    return this.series.filter((s) => s.project_id === projectId).reverse();
  }

  private createSeries(projectId: string | null, title: string, description?: string): NoteSeries {
    const now = nowIso();
    const series: NoteSeries = {
      id: uid(),
      project_id: projectId,
      title,
      description: description ?? null,
      created_at: now,
      updated_at: now,
    };
    this.series.push(series);
    return series;
  }

  private deleteSeries(id: string): void {
    this.series = this.series.filter((s) => s.id !== id);
    this.contents = this.contents.filter((c) => c.series_id !== id);
  }

  private listContent(seriesId: string): NoteContent[] {
    return this.contents
      .filter((c) => c.series_id === seriesId)
      .sort((a, b) => a.sort_order - b.sort_order);
  }

  private addContent(seriesId: string, title: string | null, text: string, type: ContentType): NoteContent {
    const now = nowIso();
    const order = this.contents.filter((c) => c.series_id === seriesId).length;
    const content: NoteContent = {
      id: uid(),
      series_id: seriesId,
      sort_order: order,
      title,
      text,
      type,
      created_at: now,
      updated_at: now,
    };
    this.contents.push(content);
    return content;
  }

  private updateContent(id: string, title: string | null, text: string, type: ContentType): void {
    const c = this.contents.find((x) => x.id === id);
    if (c) {
      c.title = title;
      c.text = text;
      c.type = type;
      c.updated_at = nowIso();
    }
  }

  private deleteContent(id: string): void {
    this.contents = this.contents.filter((c) => c.id !== id);
  }

  private reorder(orderedIds: string[]): void {
    orderedIds.forEach((id, index) => {
      const c = this.contents.find((x) => x.id === id);
      if (c) c.sort_order = index;
    });
  }

  private search(query: string, limit: number): SearchHit[] {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return this.contents
      .filter((c) => c.text.toLowerCase().includes(q) || (c.title ?? "").toLowerCase().includes(q))
      .slice(0, limit)
      .map((c) => ({
        content_id: c.id,
        series_id: c.series_id,
        title: c.title,
        snippet: c.text.slice(0, 120),
        rank: 0,
      }));
  }
}

const mock = new MockBackend();
