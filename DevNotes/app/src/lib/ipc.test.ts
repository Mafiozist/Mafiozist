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
    const hits = await notesApi.search("уникальный_маркер_поиска", [], 10);
    expect(hits.length).toBeGreaterThanOrEqual(1);
    expect(hits[0].snippet).toContain("уникальный_маркер_поиска");
  });

  it("удаление серии убирает её блоки из выдачи", async () => {
    const s = await notesApi.createSeries(null, "Удаляемая");
    await notesApi.addContent(s.id, null, "исчезнет_после_удаления", "markdown");
    await notesApi.deleteSeries(s.id);
    const hits = await notesApi.search("исчезнет_после_удаления", [], 10);
    expect(hits.length).toBe(0);
  });

  it("назначает теги серии и заменяет их набор", async () => {
    const a = await notesApi.createTag("ТегA");
    const b = await notesApi.createTag("ТегB");
    const s = await notesApi.createSeries(null, "С тегами");
    await notesApi.setSeriesTags(s.id, [a.id, b.id]);
    expect((await notesApi.listTagsForSeries(s.id)).length).toBe(2);
    await notesApi.setSeriesTags(s.id, [a.id]);
    const tags = await notesApi.listTagsForSeries(s.id);
    expect(tags.map((t) => t.id)).toEqual([a.id]);
  });

  it("поиск фильтрует по тегам (AND-семантика)", async () => {
    const rust = await notesApi.createTag("RustТег");
    const web = await notesApi.createTag("WebТег");
    const s1 = await notesApi.createSeries(null, "Ядро-тест");
    await notesApi.setSeriesTags(s1.id, [rust.id]);
    await notesApi.addContent(s1.id, null, "общий_маркер_тегов", "markdown");
    const s2 = await notesApi.createSeries(null, "Фронт-тест");
    await notesApi.setSeriesTags(s2.id, [web.id]);
    await notesApi.addContent(s2.id, null, "общий_маркер_тегов", "markdown");

    expect((await notesApi.search("общий_маркер_тегов", [], 10)).length).toBe(2);
    const only = await notesApi.search("общий_маркер_тегов", [rust.id], 10);
    expect(only.length).toBe(1);
    expect(only[0].series_id).toBe(s1.id);
    // Оба тега сразу — ни одна серия не имеет обоих.
    expect((await notesApi.search("общий_маркер_тегов", [rust.id, web.id], 10)).length).toBe(0);
  });

  it("поиск только по тегам без текста возвращает блоки серии", async () => {
    const tag = await notesApi.createTag("ТолькоТег");
    const s = await notesApi.createSeries(null, "Без-текста");
    await notesApi.setSeriesTags(s.id, [tag.id]);
    await notesApi.addContent(s.id, null, "любой контент", "markdown");
    const hits = await notesApi.search("", [tag.id], 10);
    expect(hits.length).toBeGreaterThanOrEqual(1);
    expect(hits[0].series_id).toBe(s.id);
  });
});
