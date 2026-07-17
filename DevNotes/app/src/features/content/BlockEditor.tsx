// Правая панель: блоки серии с редактированием на месте и drag-and-drop сортировкой
// (@dnd-kit, как в Portfolio) + форма добавления. Бэкенд: update_content / reorder_content.
import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Plus, Trash2, Clock, GripVertical, Pencil, Check, X } from "lucide-react";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { useUiStore } from "@/stores/uiStore";
import type { ContentType, NoteContent } from "@/domain/types";
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
  // Локальный порядок id для оптимистичного drag-and-drop.
  const [order, setOrder] = useState<string[]>([]);

  const { data: blocks = [] } = useQuery({
    queryKey: queryKeys.content(seriesId ?? "none"),
    queryFn: () => notesApi.listContent(seriesId as string),
    enabled: !!seriesId,
  });

  // Синхронизируем локальный порядок с данными сервера.
  useEffect(() => {
    setOrder(blocks.map((b) => b.id));
  }, [blocks]);

  const invalidate = () =>
    qc.invalidateQueries({ queryKey: queryKeys.content(seriesId as string) });

  const addBlock = useMutation({
    mutationFn: () => notesApi.addContent(seriesId as string, title.trim() || null, text, type),
    onSuccess: () => {
      setTitle("");
      setText("");
      invalidate();
    },
  });

  const removeBlock = useMutation({
    mutationFn: (id: string) => notesApi.deleteContent(id),
    onSuccess: invalidate,
  });

  const saveBlock = useMutation({
    mutationFn: (v: { id: string; title: string | null; text: string; type: ContentType }) =>
      notesApi.updateContent(v.id, v.title, v.text, v.type),
    onSuccess: invalidate,
  });

  const reorder = useMutation({
    mutationFn: (ids: string[]) => notesApi.reorderContent(ids),
    onSuccess: invalidate,
  });

  const sensors = useSensors(
    // Небольшой порог, чтобы клики по кнопкам не воспринимались как перетаскивание.
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  const onDragEnd = (e: DragEndEvent) => {
    const { active, over } = e;
    if (over && active.id !== over.id) {
      const oldIndex = order.indexOf(active.id as string);
      const newIndex = order.indexOf(over.id as string);
      const next = arrayMove(order, oldIndex, newIndex);
      setOrder(next); // оптимистично
      reorder.mutate(next);
    }
  };

  if (!seriesId) {
    return (
      <main className="flex flex-1 items-center justify-center matrix-bg">
        <p className="text-sm text-muted-foreground">
          Выберите серию слева или создайте новую, чтобы начать вести заметки.
        </p>
      </main>
    );
  }

  // Блоки в текущем (возможно оптимистичном) порядке.
  const orderedBlocks = order
    .map((id) => blocks.find((b) => b.id === id))
    .filter((b): b is NoteContent => !!b);

  return (
    <main className="flex flex-1 flex-col overflow-hidden">
      <SeriesTagsBar seriesId={seriesId} />

      <div className="flex-1 space-y-3 overflow-y-auto p-6">
        {orderedBlocks.length === 0 && (
          <p className="text-sm text-muted-foreground">В этой серии ещё нет блоков.</p>
        )}

        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
          <SortableContext items={order} strategy={verticalListSortingStrategy}>
            <div className="space-y-3">
              {orderedBlocks.map((b) => (
                <SortableBlock
                  key={b.id}
                  block={b}
                  onSave={(v) => saveBlock.mutate({ id: b.id, ...v })}
                  onDelete={() => removeBlock.mutate(b.id)}
                />
              ))}
            </div>
          </SortableContext>
        </DndContext>
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
        <TextArea value={text} onChange={(e) => setText(e.target.value)} placeholder="Текст заметки…" />
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

/** Один блок: перетаскиваемый, с режимами просмотра и редактирования. */
function SortableBlock({
  block,
  onSave,
  onDelete,
}: {
  block: NoteContent;
  onSave: (v: { title: string | null; text: string; type: ContentType }) => void;
  onDelete: () => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: block.id,
  });
  const style: React.CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(block.title ?? "");
  const [text, setText] = useState(block.text);
  const [type, setType] = useState<ContentType>(block.type);

  const startEdit = () => {
    setTitle(block.title ?? "");
    setText(block.text);
    setType(block.type);
    setEditing(true);
  };

  const save = () => {
    onSave({ title: title.trim() || null, text, type });
    setEditing(false);
  };

  return (
    <Card ref={setNodeRef} style={style} className="group p-4">
      <div className="mb-2 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          {/* Ручка перетаскивания */}
          <button
            {...attributes}
            {...listeners}
            aria-label="Перетащить блок"
            className="cursor-grab text-muted-foreground hover:text-foreground active:cursor-grabbing"
          >
            <GripVertical size={16} />
          </button>
          <Badge variant={block.type === "code" ? "default" : "secondary"}>{block.type}</Badge>
          {!editing && block.title && <span className="text-sm font-medium">{block.title}</span>}
        </div>
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1 text-xs text-muted-foreground">
            <Clock size={12} />
            {new Date(block.created_at).toLocaleString("ru-RU")}
          </span>
          {!editing && (
            <button
              onClick={startEdit}
              aria-label="Редактировать блок"
              className="text-muted-foreground opacity-0 transition hover:text-primary group-hover:opacity-100"
            >
              <Pencil size={14} />
            </button>
          )}
          <button
            onClick={onDelete}
            aria-label="Удалить блок"
            className="text-muted-foreground opacity-0 transition hover:text-destructive group-hover:opacity-100"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {editing ? (
        <div className="space-y-2">
          <div className="flex gap-2">
            <select
              value={type}
              onChange={(e) => setType(e.target.value as ContentType)}
              className="h-9 rounded-md border border-input bg-background px-2 text-sm"
            >
              {CONTENT_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
            <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Заголовок" className="h-9" />
          </div>
          <TextArea value={text} onChange={(e) => setText(e.target.value)} />
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setEditing(false)}>
              <X size={14} /> Отмена
            </Button>
            <Button size="sm" onClick={save} disabled={!text.trim()}>
              <Check size={14} /> Сохранить
            </Button>
          </div>
        </div>
      ) : block.type === "code" ? (
        <pre className="overflow-x-auto rounded bg-background/60 p-3 text-xs leading-relaxed">
          <code>{block.text}</code>
        </pre>
      ) : (
        <p className="whitespace-pre-wrap text-sm leading-relaxed">{block.text}</p>
      )}
    </Card>
  );
}
