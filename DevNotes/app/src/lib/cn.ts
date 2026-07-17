// Утилита объединения классов (clsx + tailwind-merge) — как в Portfolio.
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Объединяет классы и корректно разрешает конфликты Tailwind. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
