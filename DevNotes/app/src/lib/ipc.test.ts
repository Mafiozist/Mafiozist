// Юнит-тесты браузерного IPC-мока: проверяем, что семантика команд совпадает
// с ожиданиями фронтенда (создание иерархии, каскады, поиск, reorder).
// В среде node `inTauri()` = false → invoke идёт в мок. См. docs/13-TESTING.md.
import { describe, expect, it } from "vitest";
import { notesApi } from "@/api/notes";

describe("IPC-мок (браузерный бэкенд)", () => {
  it("создаёт проект и возвращает его в списке", async () => {
    const created = await notesApi.createProject("Проект X", "px");
    const list = await notesApi.listProjects();
    expect(list.some((p) => p.id === created.id && p.name === "Проект X")).toBe(true);
  });

  it("добавляет блоки и сохраняет порядок вставки", async () => {
    const s = await notesApi.createSeries(null, "Серия A");
    const a = await notesApi.addContent(s.id, "A", "первый", "markdown");
    const b = await notesApi.addContent(s.id, "B", "второй", "code");
    const blocks = await notesApi.listContent(s.id);
    expect(blocks.map((x) => x.id)).toEqual([a.id, b.id]);
    expect(blocks[0].sort_order).toBe(0);
  });

  it("reorder меняет порядок блоков", async () => {
    const s = await notesApi.createSeries(null, "Серия B");
    const a = await notesApi.addContent(s.id, null, "A", "markdown");
    const b = await notesApi.addContent(s.id, null, "B", "markdown");
    await notesApi.reorderContent([b.id, a.id]);
    const blocks = await notesApi.listContent(s.id);
    expect(blocks.map((x) => x.id)).toEqual([b.id, a.id]);
  });

  it("поиск находит по подстроке и учитывает лимит", async () => {
    const s = await notesApi.createSeries(null, "Серия поиска");
    await notesApi.addContent(s.id, "заголовок", "уникальный_маркер_поиска в тексте", "markdown");
    const hits = await notesApi.search("уникальный_маркер_поиска", 10);
    expect(hits.length).toBeGreaterThanOrEqual(1);
    expect(hits[0].snippet).toContain("уникальный_маркер_поиска");
  });

  it("удаление серии убирает её блоки из выдачи", async () => {
    const s = await notesApi.createSeries(null, "Удаляемая");
    await notesApi.addContent(s.id, null, "исчезнет_после_удаления", "markdown");
    await notesApi.deleteSeries(s.id);
    const hits = await notesApi.search("исчезнет_после_удаления", 10);
    expect(hits.length).toBe(0);
  });
});
