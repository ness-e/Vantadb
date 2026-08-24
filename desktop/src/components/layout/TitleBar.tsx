import { getCurrentWindow } from "@tauri-apps/api/window";
import { isEmbedded } from "../../transport";

/**
 * TitleBar (FIND-19) — custom window chrome so the app doesn't feel like an
 * embedded web page. Rendered only on the Tauri build (isEmbedded guard in
 * App.tsx); web builds keep native browser chrome.
 *
 * Requires capabilities: core:window:allow-{minimize,toggle-maximize,close,
 * start-dragging} (desktop/src-tauri/capabilities/default.json).
 */

// Lazy: getCurrentWindow() a nivel de módulo explota fuera de Tauri (no hay
// window.__TAURI_INTERNALS__) y mataba TODA la build web en blanco.
function win() {
  if (isEmbedded) return null;
  return getCurrentWindow();
}

function ControlButton(props: {
  label: string;
  glyph: string;
  danger?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={props.label}
      title={props.label}
      onClick={props.onClick}
      className={`flex h-9 w-10 items-center justify-center text-sm transition-colors duration-150 ${
        props.danger
          ? "text-[var(--foreground)] hover:bg-[#C41E25] hover:text-white"
          : "text-[var(--foreground)] hover:bg-black/10 dark:hover:bg-white/15"
      }`}
    >
      {props.glyph}
    </button>
  );
}

export function TitleBar() {
  const w = win();
  return (
    <div
      data-tauri-drag-region
      onDoubleClick={() => void w?.toggleMaximize()}
      className="flex h-9 shrink-0 select-none items-center justify-between border-b-2 border-black bg-[var(--background)]"
    >
      <span
        data-tauri-drag-region
        className="pl-3 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--foreground)]"
      >
        VantaDB Studio
      </span>
      <div className="flex h-9 items-stretch">
        <ControlButton label="Minimizar" glyph="─" onClick={() => void w?.minimize()} />
        <ControlButton
          label="Maximizar / restaurar"
          glyph="□"
          onClick={() => void w?.toggleMaximize()}
        />
        <ControlButton label="Cerrar" glyph="✕" danger onClick={() => void w?.close()} />
      </div>
    </div>
  );
}
