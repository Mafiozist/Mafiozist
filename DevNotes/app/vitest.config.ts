// Конфигурация Vitest для юнит-тестов фронтенда.
// Среда node: тестируем чистую логику (IPC-мок, репозиторий) без DOM.
import { defineConfig } from "vitest/config";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
