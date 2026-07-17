// Средняя панель: список серий выбранного проекта + создание.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Layers, Plus } from "lucide-react";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { useUiStore } from "@/stores/uiStore";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { cn } from "@/lib/cn";

export function SeriesList() {
  const qc = useQueryClient();
  const projectId = useUiStore((s) => s.selectedProjectId);
  const selectedSeriesId = useUiStore((s) => s.selectedSeriesId);
  const selectSeries = useUiStore((s) => s.selectSeries);
  const [title, setTitle] = useState("");

  const { data: series = [] } = useQuery({
    queryKey: queryKeys.series(projectId),
    queryFn: () => notesApi.listSeries(projectId),
  });

  const createSeries = useMutation({
    mutationFn: (t: string) => notesApi.createSeries(projectId, t),
    onSuccess: (created) => {
      setTitle("");
      qc.invalidateQueries({ queryKey: queryKeys.series(projectId) });
      selectSeries(created.id);
    },
  });

  const submit = () => {
    const trimmed = title.trim();
    if (trimmed) createSeries.mutate(trimmed);
  };

  return (
    <section className="flex h-full w-72 shrink-0 flex-col border-r">
      <div className="flex items-center gap-2 px-4 py-3 text-primary">
        <Layers size={18} />
        <span className="text-sm font-semibold tracking-wide">СЕРИИ</span>
      </div>

      <ul className="flex-1 space-y-1 overflow-y-auto px-2">
        {series.length === 0 && (
          <li className="px-3 py-6 text-center text-xs text-muted-foreground">
            Пока нет серий. Создайте первую ниже.
          </li>
        )}
        {series.map((s) => (
          <li key={s.id}>
            <button
              onClick={() => selectSeries(s.id)}
              className={cn(
                "w-full rounded-md px-3 py-2 text-left text-sm transition-colors",
                selectedSeriesId === s.id ? "bg-primary/15 text-primary" : "hover:bg-accent/60",
              )}
            >
              <div className="truncate font-medium">{s.title}</div>
              <div className="truncate text-xs text-muted-foreground">
                {new Date(s.created_at).toLocaleString("ru-RU")}
              </div>
            </button>
          </li>
        ))}
      </ul>

      <div className="flex gap-2 border-t p-2">
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder="Новая серия…"
          className="h-9"
        />
        <Button size="icon" onClick={submit} aria-label="Создать серию" className="h-9 w-9 shrink-0">
          <Plus size={16} />
        </Button>
      </div>
    </section>
  );
}
