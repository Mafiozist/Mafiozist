// Командная палитра поиска (Ctrl/Cmd+K): мгновенный полнотекстовый поиск по всей БД.
// Запрос дебаунсится; результаты кликабельны и переводят к нужной серии.
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { useUiStore } from "@/stores/uiStore";
import { Input } from "@/components/ui/Input";

export function SearchPalette() {
  const open = useUiStore((s) => s.searchOpen);
  const setOpen = useUiStore((s) => s.setSearchOpen);
  const selectSeries = useUiStore((s) => s.selectSeries);

  const [raw, setRaw] = useState("");
  const query = useDebounced(raw, 150);

  const { data: hits = [] } = useQuery({
    queryKey: queryKeys.search(query),
    queryFn: () => notesApi.search(query, 30),
    enabled: open && query.trim().length > 0,
  });

  // Сброс ввода при закрытии.
  useEffect(() => {
    if (!open) setRaw("");
  }, [open]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-24"
      onClick={() => setOpen(false)}
    >
      <div
        className="w-full max-w-xl overflow-hidden rounded-lg border bg-card shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b px-3">
          <Search size={16} className="text-muted-foreground" />
          <Input
            autoFocus
            value={raw}
            onChange={(e) => setRaw(e.target.value)}
            onKeyDown={(e) => e.key === "Escape" && setOpen(false)}
            placeholder="Поиск по всем заметкам…"
            className="border-0 focus-visible:ring-0"
          />
        </div>

        <ul className="max-h-80 overflow-y-auto p-2">
          {query.trim() && hits.length === 0 && (
            <li className="px-3 py-6 text-center text-sm text-muted-foreground">Ничего не найдено</li>
          )}
          {hits.map((h) => (
            <li key={h.content_id}>
              <button
                onClick={() => {
                  selectSeries(h.series_id);
                  setOpen(false);
                }}
                className="w-full rounded-md px-3 py-2 text-left transition-colors hover:bg-accent/60"
              >
                {h.title && <div className="text-sm font-medium text-primary">{h.title}</div>}
                <div className="truncate text-xs text-muted-foreground">{h.snippet}</div>
              </button>
            </li>
          ))}
        </ul>

        <div className="border-t px-3 py-2 text-xs text-muted-foreground">
          Enter — открыть · Esc — закрыть
        </div>
      </div>
    </div>
  );
}

/** Дебаунс значения на заданную задержку (мс). */
function useDebounced<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const id = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(id);
  }, [value, delayMs]);
  return debounced;
}
