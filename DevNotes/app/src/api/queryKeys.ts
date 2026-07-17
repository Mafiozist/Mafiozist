// Генераторы ключей TanStack Query — единообразная инвалидация кэша (как в Portfolio).
export const queryKeys = {
  projects: ["projects"] as const,
  series: (projectId: string | null) => ["series", projectId] as const,
  content: (seriesId: string) => ["content", seriesId] as const,
  tags: ["tags"] as const,
  seriesTags: (seriesId: string) => ["series-tags", seriesId] as const,
  search: (query: string, tagIds: string[]) => ["search", query, tagIds.join(",")] as const,
};
