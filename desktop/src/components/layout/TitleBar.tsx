import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * TitleBar (FIND-19) — custom window chrome so the app doesn't feel like an
 * embedded web page. Rendered only on the Tauri build (isEmbedded guard in
 * App.tsx); web builds keep native browser chrome.
 * Style: linocut — theme background, 2px black bottom border.
 */

export function TitleBar() {
  const win = getCurrentWindow();

  return (
    <div
      data-tauri-drag-region
      onDoubleClick={() => void win.toggleMaximize()}
      className="flex h-9 shrink-0 select-none items-center justify-between border-b-2 border-black bg-[var(--background)] px-3"
    >
      <span
        data-tauri-drag-region
        className="font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--foreground)]"
      >
        VantaDB Studio
      </span>
      <div className="flex items-center gap-1">
        <button
          aria-label="Minimizar"
          onClick={() => void win.minimize()}
          className="h-6 w-8 border border-black/20 text-[var(--foreground)] hover:bg-black/10"
        >
          –
        </button>
        <button
          aria-label="Maximizar"
          onClick={() => void win.toggleMaximize()}
          className="h-6 w-8 border border-black/20 text-[var(--foreground)] hover:bg-black/10"
        >
          ▢
        </button>
        <button
          aria-label="Cerrar"
          onClick={() => void win.close()}
          className="h-6 w-8 border border-black/20 text-[var(--foreground)] hover:bg-[#C41E25] hover:text-white"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
