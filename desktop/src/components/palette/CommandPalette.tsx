// Command palette (VS-09): Ctrl+K global command menu, cmdk ^1.1.
// Contrato Fase 0 (P9 teclado-first): abrir namespace (→ MEMORIAS, mismo
// comportamiento que el sidebar), buscar key (→ MEMORIAS + búsqueda global),
// exportar (.jsonl desde list — VS-CORE-04 trae export filtrado en Fase 2),
// borrar/undo (hooks que VS-08 conecta a la papelera), toggle tema, abrir
// lentes (ACTIVITY/ÍNDICES/IQL — IQL es Fase 2, la lente sigue placeholder).
//
// Estilo manga/linocut de VS-01: el CSS vive en un <style> local apuntando a
// los data-attrs `cmdk-*` (los portales de Radix Dialog escapan al body, así
// que los selectores son globales — única instancia en la app). Colores via
// vars --color-* de VS-01 → dark mode automático.
//
// Atajos: Ctrl+K abre/cierra (WorkspaceShell); mientras la palette está
// abierta, Alt+E exporta, Alt+T tema, Alt+B buscar key, Alt+D borrar, Alt+U
// undo. Ctrl+Z (undo global) lo registra VS-08 con la papelera.
import { ReactNode, useEffect } from "react";
import { Command, useCommandState } from "cmdk";
import { list, vantaErrorMessage } from "../../vanta";

export type PaletteSurface = "resumen" | "memorias" | "actividad" | "indices" | "iql";

export interface NamespaceOption {
  name: string;
  count: number;
}

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Namespaces vistos en el sidebar (counts client-side, VS-CORE-02 luego). */
  namespaces: NamespaceOption[];
  dark: boolean;
  activeConnection: string | null;
  onNavigate: (surface: PaletteSurface) => void;
  /** q vacío → foco en la búsqueda global; con texto → MEMORIAS + search(). */
  onSearch: (q: string) => void;
  onToggleTheme: () => void;
  /** Hook VS-08: store de undo/papelera (si aún no existe, stub con notice). */
  onUndo?: () => void;
  /** Hook VS-08: soft-delete con papelera (si aún no existe, stub con notice). */
  onDelete?: () => void;
  onError: (msg: string) => void;
}

/** Export Fase 0: JSONL de los primeros 500 registros vía list() (el bridge no
 * expone export_namespace todavía — VS-CORE-04 en Fase 2). */
async function downloadRecordsJsonl(): Promise<void> {
  const records = await list({ limit: 500 });
  const blob = new Blob([records.map((r) => JSON.stringify(r)).join("\n")], {
    type: "application/x-ndjson",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `vanta-export-${new Date().toISOString().replace(/[:.]/g, "-")}.jsonl`;
  a.click();
  URL.revokeObjectURL(url);
}

function PaletteItem({
  value,
  kbd,
  keywords,
  onSelect,
  children,
}: {
  value: string;
  kbd?: string;
  keywords?: string[];
  onSelect: () => void;
  children: ReactNode;
}) {
  return (
    <Command.Item value={value} keywords={keywords} onSelect={onSelect}>
      {children}
      {kbd && <kbd>{kbd}</kbd>}
    </Command.Item>
  );
}

/** Fallback cuando ningún comando matchea: busca la key/query tipeada. */
function SearchKeyFallback({ onSearch }: { onSearch: (q: string) => void }) {
  const search = useCommandState((s) => s.search);
  const q = search.trim();
  return (
    <Command.Item
      value={`buscar-key:${search}`}
      keywords={["buscar", "key", "search", "query"]}
      onSelect={() => {
        if (q) onSearch(q);
      }}
    >
      🔎 Buscar key "{search}"…
    </Command.Item>
  );
}

export default function CommandPalette({
  open,
  onOpenChange,
  namespaces,
  dark,
  activeConnection,
  onNavigate,
  onSearch,
  onToggleTheme,
  onUndo,
  onDelete,
  onError,
}: CommandPaletteProps) {
  // Atajos in-palette (Alt+…): se disparan solo mientras la palette está
  // montada. Alt+letra no inserta texto en el input → sin conflicto con typing.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (!e.altKey) return;
      const k = e.key.toLowerCase();
      const close = (fn: () => void) => {
        fn();
        onOpenChange(false);
      };
      if (k === "e") {
        e.preventDefault();
        close(() => {
          downloadRecordsJsonl().catch((err) => onError(vantaErrorMessage(err)));
        });
      } else if (k === "t") {
        e.preventDefault();
        close(onToggleTheme);
      } else if (k === "b") {
        e.preventDefault();
        close(() => onSearch(""));
      } else if (k === "d") {
        e.preventDefault();
        close(() => onDelete?.());
      } else if (k === "u") {
        e.preventDefault();
        close(() => onUndo?.());
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onOpenChange, onSearch, onToggleTheme, onUndo, onDelete, onError]);

  const run = (fn: () => void) => {
    fn();
    onOpenChange(false);
  };

  return (
    <>
      <Command.Dialog
        open={open}
        onOpenChange={onOpenChange}
        label="Comandos de Vanta Studio"
        overlayClassName="vcmd-overlay"
        contentClassName="vcmd-dialog"
      >
        <Command.Input placeholder="Escribí un comando o buscá una key…" />
        <Command.List>
          <Command.Empty>
            <SearchKeyFallback onSearch={(q) => run(() => onSearch(q))} />
          </Command.Empty>

          <Command.Group heading="Navegación">
            <PaletteItem value="nav-resumen" onSelect={() => run(() => onNavigate("resumen"))}>
              ◫ RESUMEN
            </PaletteItem>
            <PaletteItem
              value="nav-memorias"
              keywords={["memories", "grid", "memorias"]}
              onSelect={() => run(() => onNavigate("memorias"))}
            >
              ▦ MEMORIAS
            </PaletteItem>
          </Command.Group>

          <Command.Group heading="Lentes">
            <PaletteItem
              value="lens-actividad"
              keywords={["activity", "actividad", "timeline"]}
              onSelect={() => run(() => onNavigate("actividad"))}
            >
              ◷ ACTIVITY
            </PaletteItem>
            <PaletteItem
              value="lens-indices"
              keywords={["indices", "indexes", "hnsw", "bm25"]}
              onSelect={() => run(() => onNavigate("indices"))}
            >
              ⠿ ÍNDICES
            </PaletteItem>
            <PaletteItem
              value="lens-iql"
              keywords={["iql", "query", "grafo", "graph"]}
              onSelect={() => run(() => onNavigate("iql"))}
            >
              ⌘ IQL <span className="vcmd-phase">Fase 2</span>
            </PaletteItem>
          </Command.Group>

          <Command.Group heading="Abrir namespace (en MEMORIAS)">
            {namespaces.length === 0 ? (
              <Command.Item value="ns-vacio" disabled>
                sin registros
              </Command.Item>
            ) : (
              namespaces.map((n) => (
                <PaletteItem
                  key={n.name}
                  value={`ns:${n.name}`}
                  keywords={["namespace", n.name]}
                  onSelect={() => run(() => onNavigate("memorias"))}
                >
                  <span className="vcmd-ns">◆ {n.name}</span>
                  <span className="vcmd-count">{n.count}</span>
                </PaletteItem>
              ))
            )}
          </Command.Group>

          <Command.Group heading="Acciones">
            <PaletteItem
              value="accion-buscar"
              kbd="Alt+B"
              keywords={["buscar", "key", "search", "query"]}
              onSelect={() => run(() => onSearch(""))}
            >
              🔎 Buscar key…
            </PaletteItem>
            <PaletteItem
              value="accion-exportar"
              kbd="Alt+E"
              keywords={["exportar", "export", "jsonl", "snapshot"]}
              onSelect={() =>
                run(() => {
                  downloadRecordsJsonl().catch((err) => onError(vantaErrorMessage(err)));
                })
              }
            >
              ⬇ Exportar memorias (.jsonl)
            </PaletteItem>
            <PaletteItem
              value="accion-borrar"
              kbd="Alt+D"
              keywords={["borrar", "delete", "eliminar", "papelera", "trash"]}
              onSelect={() => run(() => onDelete?.())}
            >
              ✕ Borrar registro…
            </PaletteItem>
            <PaletteItem
              value="accion-undo"
              kbd="Ctrl+Z"
              keywords={["deshacer", "undo", "revert"]}
              onSelect={() => run(() => onUndo?.())}
            >
              ↺ Deshacer
            </PaletteItem>
            <PaletteItem
              value="accion-tema"
              kbd="Alt+T"
              keywords={["tema", "theme", "dark", "light", "oscuro", "claro"]}
              onSelect={() => run(onToggleTheme)}
            >
              {dark ? "☀ Tema claro" : "☾ Tema oscuro"}
            </PaletteItem>
          </Command.Group>
        </Command.List>

        {activeConnection && (
          <div className="vcmd-footer">◆ namespace activo · {activeConnection}</div>
        )}
      </Command.Dialog>

      <style>{`
        [cmdk-overlay] { background: rgba(0, 0, 0, 0.55); }
        [cmdk-dialog] {
          position: fixed;
          top: 16vh;
          left: 50%;
          z-index: 60;
          transform: translateX(-50%);
          width: min(620px, 92vw);
          border: 4px solid var(--color-foreground);
          background: var(--color-background);
          color: var(--color-foreground);
          box-shadow: 6px 6px 0 0 var(--color-foreground);
          font-family: var(--font-geist-sans);
        }
        [cmdk-input] {
          width: 100%;
          border: 0;
          border-bottom: 4px solid var(--color-foreground);
          background: transparent;
          padding: 14px 16px;
          font-family: var(--font-anton);
          font-size: 22px;
          letter-spacing: 0.02em;
          color: var(--color-foreground);
          outline: none;
        }
        [cmdk-input]::placeholder { color: var(--color-muted-foreground); }
        [cmdk-list] {
          max-height: 52vh;
          overflow-y: auto;
          scroll-padding-block: 6px;
        }
        [cmdk-list-sizer] { padding: 8px; }
        [cmdk-group-heading] {
          padding: 10px 10px 4px;
          font-family: var(--font-space-mono);
          font-size: 10px;
          letter-spacing: 0.2em;
          text-transform: uppercase;
          color: var(--color-muted-foreground);
        }
        [cmdk-item] {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 8px 10px;
          font-size: 14px;
          cursor: pointer;
          border: 2px solid transparent;
        }
        [cmdk-item][data-selected="true"] {
          background: var(--color-neon);
          color: var(--color-background);
          border-color: var(--color-foreground);
        }
        [cmdk-item][aria-disabled="true"] { opacity: 0.5; cursor: default; }
        [cmdk-item] kbd {
          margin-left: auto;
          border: 2px solid var(--color-foreground);
          background: var(--color-background);
          padding: 1px 6px;
          font-family: var(--font-space-mono);
          font-size: 10px;
          text-transform: uppercase;
          color: var(--color-foreground);
        }
        [cmdk-item][data-selected="true"] kbd {
          border-color: var(--color-background);
          background: transparent;
          color: var(--color-background);
        }
        [cmdk-empty] {
          padding: 12px 10px;
          font-family: var(--font-space-mono);
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.15em;
          color: var(--color-muted-foreground);
        }
        .vcmd-phase {
          margin-left: 4px;
          font-family: var(--font-space-mono);
          font-size: 9px;
          text-transform: uppercase;
          color: var(--color-muted-foreground);
        }
        .vcmd-ns { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .vcmd-count {
          margin-left: auto;
          font-family: var(--font-anton);
          font-size: 16px;
          line-height: 1;
        }
        .vcmd-footer {
          border-top: 4px solid var(--color-foreground);
          padding: 6px 12px;
          font-family: var(--font-space-mono);
          font-size: 10px;
          text-transform: uppercase;
          letter-spacing: 0.15em;
          color: var(--color-muted-foreground);
        }
      `}</style>
    </>
  );
}