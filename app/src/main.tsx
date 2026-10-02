import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

async function boot() {
  // outside Tauri (plain `npm run dev` in a browser) use the in-memory mock backend
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window && (window as any).__TAURI_INTERNALS__?.invoke)) {
    const m = await import("./dev/mockBackend");
    m.installMockBackend();
    (window as any).__store = (await import("./store")).store;
  }
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}
boot();
