// Правая панель: блоки выбранной серии + добавление нового блока.
// Типы блоков валидируются на бэкенде; здесь — выбор из допустимого набора.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Clock } from "lucide-react";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { useUiStore } from "@/stores/uiStore";
import type { ContentType } from "@/domain/types";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { TextArea } from "@/components/ui/TextArea";
import { Badge } from "@/components/ui/Badge";
import { Card } from "@/components/ui/Card";
import { SeriesTagsBar } from "@/features/tags/SeriesTagsBar";

const CONTENT_TYPES: ContentType[] = ["markdown", "code", "image", "link"];

export function BlockEditor() {
  const qc = useQueryClient();
  const seriesId = useUiStore((s) => s.selectedSeriesId);

  const [title, setTitle] = useState("");
  const [text, setText] = useState("");
  const [type, setType] = useState<ContentType>("markdown");

  const { data: blocks = [] } = useQuery({
    queryKey: queryKeys.content(seriesId ?? "none"),
    queryFn: () => notesApi.listContent(seriesId as string),
    enabled: !!seriesId,
  });

  const addBlock = useMutation({
    mutationFn: () => notesApi.addContent(seriesId as string, title.trim() || null, text, type),
    onSuccess: () => {
      setTitle("");
      setText("");
      qc.invalidateQueries({ queryKey: queryKeys.content(seriesId as string) });
    },
  });

  const removeBlock = useMutation({
    mutationFn: (id: string) => notesApi.deleteContent(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: queryKeys.content(seriesId as string) }),
  });

  if (!seriesId) {
    return (
      <main className="flex flex-1 items-center justify-center matrix-bg">
        <p className="text-sm text-muted-foreground">
          Выберите серию слева или создайте новую, чтобы начать вести заметки.
        </p>
      </main>
    );
  }

  return (
    <main className="flex flex-1 flex-col overflow-hidden">
      {/* Панель тегов технологий текущей серии */}
      <SeriesTagsBar seriesId={seriesId} />

      <div className="flex-1 space-y-3 overflow-y-auto p-6">
        {blocks.length === 0 && (
          <p className="text-sm text-muted-foreground">В этой серии ещё нет блоков.</p>
        )}
        {blocks.map((b) => (
          <Card key={b.id} className="group p-4">
            <div className="mb-2 flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Badge variant={b.type === "code" ? "default" : "secondary"}>{b.type}</Badge>
                {b.title && <span className="text-sm font-medium">{b.title}</span>}
              </div>
              <div className="flex items-center gap-3">
                <span className="flex items-center gap-1 text-xs text-muted-foreground">
                  <Clock size={12} />
                  {new Date(b.created_at).toLocaleString("ru-RU")}
                </span>
                <button
                  onClick={() => removeBlock.mutate(b.id)}
                  aria-label="Удалить блок"
                  className="text-muted-foreground opacity-0 transition hover:text-destructive group-hover:opacity-100"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
            {b.type === "code" ? (
              <pre className="overflow-x-auto rounded bg-background/60 p-3 text-xs leading-relaxed">
                <code>{b.text}</code>
              </pre>
            ) : (
              <p className="whitespace-pre-wrap text-sm leading-relaxed">{b.text}</p>
            )}
          </Card>
        ))}
      </div>

      {/* Форма добавления блока */}
      <div className="space-y-2 border-t bg-card/40 p-4">
        <div className="flex gap-2">
          <select
            value={type}
            onChange={(e) => setType(e.target.value as ContentType)}
            className="h-10 rounded-md border border-input bg-background px-2 text-sm"
          >
            {CONTENT_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Заголовок блока (необязательно)"
          />
        </div>
        <TextArea
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Текст заметки…"
        />
        <div className="flex justify-end">
          <Button onClick={() => text.trim() && addBlock.mutate()} disabled={!text.trim()}>
            <Plus size={16} />
            Добавить блок
          </Button>
        </div>
      </div>
    </main>
  );
}
