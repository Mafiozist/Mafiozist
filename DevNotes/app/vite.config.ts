// Конфигурация Vite для фронтенда DEVNOTES.
// Порт 1420 фиксирован — его ждёт Tauri (devUrl в src-tauri/tauri.conf.json).
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  // Tauri оперирует статикой из dist/ — относительные пути ассетов.
  base: "./",
  build: {
    target: "es2022",
    outDir: "dist",
  },
});
