// UI-состояние (Zustand): выбранный проект/серия, тема, открытие поиска.
// Данные (проекты/серии/блоки) живут в TanStack Query, здесь — только выбор/навигация.
import { create } from "zustand";

interface UiState {
  /** Выбранный проект (`null` — «входящие» без проекта). */
  selectedProjectId: string | null;
  /** Выбранная серия. */
  selectedSeriesId: string | null;
  /** Открыта ли панель поиска (командная палитра). */
  searchOpen: boolean;

  selectProject: (id: string | null) => void;
  selectSeries: (id: string | null) => void;
  setSearchOpen: (open: boolean) => void;
}

export const useUiStore = create<UiState>((set) => ({
  selectedProjectId: null,
  selectedSeriesId: null,
  searchOpen: false,

  // Смена проекта сбрасывает выбранную серию.
  selectProject: (id) => set({ selectedProjectId: id, selectedSeriesId: null }),
  selectSeries: (id) => set({ selectedSeriesId: id }),
  setSearchOpen: (open) => set({ searchOpen: open }),
}));
