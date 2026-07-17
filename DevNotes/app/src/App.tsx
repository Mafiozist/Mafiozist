// Корневой компонент: шапка + трёхпанельная раскладка (проекты | серии | редактор)
// и командная палитра поиска. Хоткей Ctrl/Cmd+K открывает поиск.
import { useEffect } from "react";
import { Terminal, Search } from "lucide-react";
import { ProjectSidebar } from "@/features/projects/ProjectSidebar";
import { SeriesList } from "@/features/series/SeriesList";
import { BlockEditor } from "@/features/content/BlockEditor";
import { SearchPalette } from "@/features/search/SearchPalette";
import { SyncButton } from "@/features/sync/SyncButton";
import { useUiStore } from "@/stores/uiStore";

export default function App() {
  const setSearchOpen = useUiStore((s) => s.setSearchOpen);

  // Глобальный хоткей открытия поиска.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSearchOpen(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setSearchOpen]);

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      {/* Шапка в стиле терминала */}
      <header className="flex h-11 shrink-0 items-center justify-between border-b bg-card/60 px-4">
        <div className="flex items-center gap-2 text-primary neon-text">
          <Terminal size={18} />
          <span className="text-sm font-bold tracking-widest">DEVNOTES</span>
        </div>
        <div className="flex items-center gap-2">
          <SyncButton />
          <button
            onClick={() => setSearchOpen(true)}
            className="flex items-center gap-2 rounded-md border px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent/60"
          >
            <Search size={14} />
            Поиск
            <kbd className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">Ctrl K</kbd>
          </button>
        </div>
      </header>

      {/* Трёхпанельная рабочая область */}
      <div className="flex flex-1 overflow-hidden">
        <ProjectSidebar />
        <SeriesList />
        <BlockEditor />
      </div>

      <SearchPalette />
    </div>
  );
}
