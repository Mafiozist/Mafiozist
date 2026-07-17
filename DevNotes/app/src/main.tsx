// Точка входа фронтенда: монтирует App с провайдерами и подключает дизайн-токены.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { AppProviders } from "@/app/providers";
import App from "@/App";
import "@/styles/tokens.css";

const root = document.getElementById("root");
if (!root) throw new Error("Не найден корневой элемент #root");

createRoot(root).render(
  <StrictMode>
    <AppProviders>
      <App />
    </AppProviders>
  </StrictMode>,
);
