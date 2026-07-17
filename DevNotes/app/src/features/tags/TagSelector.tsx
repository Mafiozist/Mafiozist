// Селектор тегов технологий (перенято из Portfolio TechTagCardSelector):
// дропдаун с поиском и чипами-переключателями, мульти-выбор, создание нового тега.
// Используется двояко: как фильтр в поиске и как назначение тегов серии.
import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Tags, Check, Plus, X } from "lucide-react";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { cn } from "@/lib/cn";

interface TagSelectorProps {
  /** Выбранные id тегов (controlled). */
  value: string[];
  /** Колбэк изменения набора выбранных тегов. */
  onChange: (ids: string[]) => void;
  /** Подпись кнопки. */
  label?: string;
  /** Разрешить создание нового тега прямо из селектора. */
  allowCreate?: boolean;
}

export function TagSelector({ value, onChange, label = "Технологии", allowCreate = true }: TagSelectorProps) {
  const qc = useQueryClient();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  const { data: tags = [] } = useQuery({ queryKey: queryKeys.tags, queryFn: notesApi.listTags });

  const createTag = useMutation({
    mutationFn: (name: string) => notesApi.createTag(name),
    onSuccess: (tag) => {
      setSearch("");
      qc.invalidateQueries({ queryKey: queryKeys.tags });
      onChange([...value, tag.id]); // сразу выбираем созданный тег
    },
  });

  // Закрытие по клику вне компонента.
  useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, []);

  const toggle = (id: string) =>
    onChange(value.includes(id) ? value.filter((x) => x !== id) : [...value, id]);

  const filtered = tags.filter((t) => t.name.toLowerCase().includes(search.toLowerCase()));
  const exactExists = tags.some((t) => t.name.toLowerCase() === search.trim().toLowerCase());

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 rounded-md border px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent/60"
      >
        <Tags size={14} />
        {label}
        {value.length > 0 && (
          <span className="rounded-full bg-primary/20 px-1.5 py-0.5 text-[10px] text-primary">
            {value.length}
          </span>
        )}
      </button>

      {open && (
        <div className="absolute z-50 mt-2 w-72 rounded-lg border bg-card p-0 shadow-2xl">
          <div className="border-b p-2">
            <input
              autoFocus
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Поиск технологий…"
              className="h-8 w-full rounded-md border border-input bg-background px-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
          </div>

          <div className="max-h-56 overflow-y-auto p-2">
            <div className="flex flex-wrap gap-1.5">
              {filtered.map((t) => {
                const active = value.includes(t.id);
                return (
                  <button
                    key={t.id}
                    type="button"
                    onClick={() => toggle(t.id)}
                    className={cn(
                      "inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-xs transition-colors",
                      active
                        ? "border-primary bg-primary/15 text-primary"
                        : "hover:bg-accent/60",
                    )}
                  >
                    {active && <Check size={12} />}
                    {t.name}
                  </button>
                );
              })}
              {filtered.length === 0 && !search && (
                <span className="px-1 py-2 text-xs text-muted-foreground">Тегов пока нет</span>
              )}
            </div>

            {allowCreate && search.trim() && !exactExists && (
              <button
                type="button"
                onClick={() => createTag.mutate(search.trim())}
                className="mt-2 flex w-full items-center gap-1.5 rounded-md border border-dashed px-2 py-1.5 text-xs text-primary hover:bg-accent/60"
              >
                <Plus size={12} />
                Создать «{search.trim()}»
              </button>
            )}
          </div>

          {value.length > 0 && (
            <div className="flex items-center justify-between border-t px-2 py-1.5">
              <span className="text-[10px] text-muted-foreground">Выбрано: {value.length}</span>
              <button
                type="button"
                onClick={() => onChange([])}
                className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-destructive"
              >
                <X size={11} /> сбросить
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
