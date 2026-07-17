// Левая панель: список проектов + создание. Выбор проекта фильтрует серии.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FolderGit2, Plus, Inbox, Trash2 } from "lucide-react";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { useUiStore } from "@/stores/uiStore";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { cn } from "@/lib/cn";

export function ProjectSidebar() {
  const qc = useQueryClient();
  const selectedProjectId = useUiStore((s) => s.selectedProjectId);
  const selectProject = useUiStore((s) => s.selectProject);
  const [name, setName] = useState("");

  const { data: projects = [] } = useQuery({
    queryKey: queryKeys.projects,
    queryFn: notesApi.listProjects,
  });

  const createProject = useMutation({
    mutationFn: (n: string) => notesApi.createProject(n),
    onSuccess: () => {
      setName("");
      qc.invalidateQueries({ queryKey: queryKeys.projects });
    },
  });

  const deleteProject = useMutation({
    mutationFn: (id: string) => notesApi.deleteProject(id),
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: queryKeys.projects });
      // Если удалили выбранный проект — вернуться во «Входящие».
      if (selectedProjectId === id) selectProject(null);
    },
  });

  const submit = () => {
    const trimmed = name.trim();
    if (trimmed) createProject.mutate(trimmed);
  };

  return (
    <aside className="flex h-full w-64 shrink-0 flex-col border-r bg-card/40">
      <div className="flex items-center gap-2 px-4 py-3 text-primary neon-text">
        <FolderGit2 size={18} />
        <span className="text-sm font-semibold tracking-wide">ПРОЕКТЫ</span>
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto px-2">
        {/* Псевдо-проект «Входящие» — серии без проекта. */}
        <ProjectRow
          active={selectedProjectId === null}
          icon={<Inbox size={16} />}
          label="Входящие"
          onClick={() => selectProject(null)}
        />
        {projects.map((p) => (
          <ProjectRow
            key={p.id}
            active={selectedProjectId === p.id}
            icon={<FolderGit2 size={16} />}
            label={p.name}
            onClick={() => selectProject(p.id)}
            onDelete={() => {
              if (confirm(`Удалить проект «${p.name}» со всеми сериями и заметками?`)) {
                deleteProject.mutate(p.id);
              }
            }}
          />
        ))}
      </nav>

      <div className="flex gap-2 border-t p-2">
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder="Новый проект…"
          className="h-9"
        />
        <Button size="icon" onClick={submit} aria-label="Создать проект" className="h-9 w-9 shrink-0">
          <Plus size={16} />
        </Button>
      </div>
    </aside>
  );
}

function ProjectRow({
  active,
  icon,
  label,
  onClick,
  onDelete,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  onDelete?: () => void;
}) {
  return (
    <div
      className={cn(
        "group flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
        active ? "bg-primary/15 text-primary" : "hover:bg-accent/60",
      )}
    >
      <button onClick={onClick} className="flex min-w-0 flex-1 items-center gap-2 text-left">
        {icon}
        <span className="truncate">{label}</span>
      </button>
      {onDelete && (
        <button
          onClick={onDelete}
          aria-label="Удалить проект"
          className="text-muted-foreground opacity-0 transition hover:text-destructive group-hover:opacity-100"
        >
          <Trash2 size={14} />
        </button>
      )}
    </div>
  );
}
