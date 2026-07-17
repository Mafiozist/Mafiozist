// Кнопка синхронизации: обмен снапшотом через файл (напр. в папке Яндекс.Диска).
// Двусторонний sync выполняет ядро (скачать → LWW-слияние → выгрузить).
import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import { notesApi } from "@/api/notes";
import { cn } from "@/lib/cn";

const DEFAULT_PATH = "devnotes-sync.json";

export function SyncButton() {
  const qc = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  const run = async () => {
    const path = window.prompt(
      "Файл-снапшот для синхронизации (например, путь внутри папки Яндекс.Диска):",
      DEFAULT_PATH,
    );
    if (!path) return;
    setBusy(true);
    setMsg(null);
    try {
      const rep = await notesApi.syncFile(path);
      setMsg(`Готово: применено ${rep.applied}, выгружено ${rep.uploaded_bytes} Б`);
      qc.invalidateQueries(); // обновить все данные после слияния
    } catch (e) {
      setMsg(`Ошибка: ${String(e)}`);
    } finally {
      setBusy(false);
      setTimeout(() => setMsg(null), 5000);
    }
  };

  return (
    <div className="flex items-center gap-2">
      {msg && <span className="text-[11px] text-muted-foreground">{msg}</span>}
      <button
        onClick={run}
        disabled={busy}
        className="flex items-center gap-2 rounded-md border px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent/60 disabled:opacity-50"
      >
        <RefreshCw size={14} className={cn(busy && "animate-spin")} />
        Синхронизация
      </button>
    </div>
  );
}
