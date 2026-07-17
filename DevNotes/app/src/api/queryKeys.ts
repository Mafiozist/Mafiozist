// Генераторы ключей TanStack Query — единообразная инвалидация кэша (как в Portfolio).
export const queryKeys = {
  projects: ["projects"] as const,
  series: (projectId: string | null) => ["series", projectId] as const,
  content: (seriesId: string) => ["content", seriesId] as const,
  search: (query: string) => ["search", query] as const,
};
