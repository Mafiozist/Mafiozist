// Панель тегов серии: показывает назначенные теги и позволяет их менять.
// Значение селектора = текущие теги серии; изменение сохраняется через set_series_tags.
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { notesApi } from "@/api/notes";
import { queryKeys } from "@/api/queryKeys";
import { Badge } from "@/components/ui/Badge";
import { TagSelector } from "@/features/tags/TagSelector";

export function SeriesTagsBar({ seriesId }: { seriesId: string }) {
  const qc = useQueryClient();

  const { data: tags = [] } = useQuery({
    queryKey: queryKeys.seriesTags(seriesId),
    queryFn: () => notesApi.listTagsForSeries(seriesId),
  });

  const setTags = useMutation({
    mutationFn: (ids: string[]) => notesApi.setSeriesTags(seriesId, ids),
    onSuccess: () => qc.invalidateQueries({ queryKey: queryKeys.seriesTags(seriesId) }),
  });

  const selectedIds = tags.map((t) => t.id);

  return (
    <div className="flex flex-wrap items-center gap-2 border-b bg-card/40 px-6 py-2">
      {tags.map((t) => (
        <Badge key={t.id} title={t.typeName ?? undefined}>
          {t.name}
        </Badge>
      ))}
      {tags.length === 0 && (
        <span className="text-xs text-muted-foreground">Теги не назначены</span>
      )}
      <TagSelector
        label="Изменить теги"
        value={selectedIds}
        onChange={(ids) => setTags.mutate(ids)}
      />
    </div>
  );
}
