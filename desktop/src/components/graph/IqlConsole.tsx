// IqlConsole.tsx (GRAFO-03): consola IQL embebida en la lente GRAFO — editor
// CodeMirror con autocompletado (shim VS-CORE-06 `iqlAutocomplete`), ejecutar
// Ctrl+Enter → `queryIql()`, resultado Read → onHighlight(nodeIds) para el
// canvas R3F, Write/StaleContext → mensaje, errores de parse legibles sin
// stack (vantaErrorMessage). Historial en localStorage `vanta.iql.history`
// (patrón VS-17: dedup no-consecutivo, MAX 10, click → re-ejecutar).
import CodeMirror from "@uiw/react-codemirror";
import { autocompletion, type Completion, type CompletionContext, type CompletionResult } from "@codemirror/autocomplete";
import { keymap } from "@codemirror/view";
import { oneDark } from "@codemirror/theme-one-dark";
import { useCallback, useMemo, useRef, useState } from "react";
import { iqlAutocomplete, queryIql, vantaErrorMessage, type VantaQueryResult } from "../../vanta";
import { TriangleAlert } from "lucide-react";

const HISTORY_KEY = "vanta.iql.history";
const MAX_HISTORY = 10;

function loadHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed)
      ? parsed.filter((e): e is string => typeof e === "string").slice(0, MAX_HISTORY)
      : [];
  } catch {
    return [];
  }
}

function saveHistory(entries: string[]): void {
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(entries));
  } catch {
    // sin persistencia → solo sesión
  }
}

interface Props {
  dark: boolean;
  /** Resultado Read → ids de nodo para resaltar en el canvas R3F. */
  onHighlight: (nodeIds: string[]) => void;
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
}

type Outcome =
  | { kind: "read"; count: number; nodeIds: string[] }
  | { kind: "write"; message: string }
  | { kind: "stale"; nodeId: string }
  | { kind: "error"; message: string };

/** Candidate → CodeMirror completion. */
function toCompletion(label: string): Completion {
  return { label, type: "keyword", boost: 100 };
}

/** Regex de token IQL (identificadores con `.` `#` `:`) para el rango `from`. */
const TOKEN_RE = /[\w#.:]+$/;

/** Source de autocompletado: pasa el texto hasta el cursor al shim VS-CORE-06
 * (`autocomplete_prefix` deriva el token actual internamente). */
async function iqlCompletionSource(ctx: CompletionContext): Promise<CompletionResult | null> {
  const before = ctx.state.sliceDoc(0, ctx.pos);
  const m = TOKEN_RE.exec(before);
  const from = m ? ctx.pos - m[0].length : ctx.pos;
  try {
    const candidates = await iqlAutocomplete(before);
    if (candidates.length === 0) return null;
    return {
      from,
      options: candidates.map(toCompletion),
      // El shim ya filtró por prefijo; desactivar el filtro interno de CM para
      // no descartar tokens con `.`/`#`/`:` (docs: filter=false = sin refilter).
      filter: false,
    };
  } catch {
    return null; // fallback: sin sugerencias, no rompe el editor
  }
}

export default function IqlConsole({ dark, onHighlight, onNotice, onError }: Props) {
  const [value, setValue] = useState("FROM ");
  const [history, setHistory] = useState<string[]>(loadHistory);
  const [running, setRunning] = useState(false);
  const [outcome, setOutcome] = useState<Outcome | null>(null);
  // Callbacks vivos para el keymap (useMemo estable captura closures viejos).
  const cbRef = useRef({ onHighlight, onNotice, onError });
  cbRef.current = { onHighlight, onNotice, onError };

  const runQuery = useCallback(async (raw: string) => {
    const query = raw.trim();
    if (!query || running) return;
    setRunning(true);
    setOutcome(null);
    try {
      const result: VantaQueryResult = await queryIql(query);
      if ("Read" in result) {
        const nodeIds = result.Read.map((r) => r.node_id).filter((x): x is string => !!x);
        cbRef.current.onHighlight(nodeIds);
        setOutcome({
          kind: "read",
          count: result.Read.length,
          nodeIds,
        });
        cbRef.current.onNotice(
          nodeIds.length === result.Read.length
            ? `IQL read: ${result.Read.length} nodo(s) resaltados`
            : `IQL read: ${result.Read.length} registros (${nodeIds.length} con node_id)`,
        );
      } else if ("Write" in result) {
        const w = result.Write;
        setOutcome({ kind: "write", message: w.message });
        cbRef.current.onNotice(`IQL write: ${w.affected_nodes} nodo(s) afectados`);
      } else {
        // StaleContext
        setOutcome({ kind: "stale", nodeId: result.StaleContext.node_id });
        cbRef.current.onNotice("IQL: contexto obsoleto — sincronizá el nodo y reintentá");
      }
    } catch (err) {
      const message = vantaErrorMessage(err);
      setOutcome({ kind: "error", message });
      cbRef.current.onError(message);
    } finally {
      setRunning(false);
    }
  }, [running]);

  const runRef = useRef<(q: string) => void>(() => {});
  runRef.current = (q) => void runQuery(q);

  /** Ejecuta y registra en el historial (dedup no-consecutivo, VS-17). */
  const execute = useCallback(
    (query: string) => {
      const trimmed = query.trim();
      if (!trimmed) return;
      setHistory((prev) => {
        const next = [trimmed, ...prev.filter((e) => e !== trimmed)].slice(0, MAX_HISTORY);
        saveHistory(next);
        return next;
      });
      void runQuery(trimmed);
    },
    [runQuery],
  );

  const extensions = useMemo(
    () => [
      autocompletion({ override: [iqlCompletionSource] }),
      keymap.of([
        {
          key: "Ctrl-Enter",
          run: (view) => {
            runRef.current(view.state.doc.toString());
            return true;
          },
        },
      ]),
    ],
    [],
  );

  const clearHistory = () => {
    setHistory([]);
    saveHistory([]);
  };

  return (
    <div className="flex h-full flex-col border-t-4 border-foreground bg-card">
      {/* Cabecera: título + resultado + acciones */}
      <div className="flex items-center gap-2 border-b border-foreground/40 px-3 py-1">
        <span className="font-tech text-[10px] uppercase tracking-widest text-neon">iql console</span>
        {running && <span className="font-tech text-[10px] text-muted-foreground">ejecutando…</span>}
        {outcome && (
          <span
            className={`font-tech text-[10px] ${outcome.kind === "error" ? "text-red-700" : "text-muted-foreground"}`}
            role="status"
          >
            {outcome.kind === "read" && `✓ ${outcome.count} registros (${outcome.nodeIds.length} resaltados)`}
            {outcome.kind === "write" && `✓ ${outcome.message}`}
            {outcome.kind === "stale" && (
              <>
                <TriangleAlert className="mr-0.5 inline h-3 w-3 align-[-2px]" strokeWidth={2.5} aria-hidden="true" />
                contexto obsoleto ({outcome.nodeId})
              </>
            )}
            {outcome.kind === "error" && `✗ ${outcome.message}`}
          </span>
        )}
        <div className="ml-auto flex items-center gap-2">
          {history.length > 0 && (
            <button
              type="button"
              onClick={clearHistory}
              className="press border border-foreground bg-background px-2 py-0.5 text-[10px] font-semibold"
              title="Limpiar historial"
            >
              ✕ historial
            </button>
          )}
          <button
            type="button"
            disabled={running}
            onClick={() => execute(value)}
            className="press border-2 border-foreground bg-neon px-2 py-0.5 text-[10px] font-bold text-background disabled:opacity-50"
            title="Ejecutar (Ctrl+Enter)"
          >
            ▶ ejecutar
          </button>
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        {/* Editor */}
        <div className="min-w-0 flex-1 border-r border-foreground/40">
          <CodeMirror
            value={value}
            onChange={setValue}
            height="100%"
            theme={dark ? oneDark : "light"}
            extensions={extensions}
            basicSetup={{ foldGutter: false, highlightActiveLine: true }}
            style={{ fontSize: 12 }}
          />
        </div>

        {/* Historial de sesión (re-ejecutable) */}
        {history.length > 0 && (
          <div className="w-56 shrink-0 overflow-y-auto border-l border-foreground/40 bg-background">
            <p className="border-b border-foreground/40 px-2 py-1 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              historial
            </p>
            <ul>
              {history.map((q) => (
                <li key={q}>
                  <button
                    type="button"
                    onClick={() => {
                      setValue(q);
                      execute(q);
                    }}
                    className="block w-full truncate px-2 py-1 text-left font-mono text-[10px] text-muted-foreground hover:bg-card hover:text-foreground"
                    title={`Re-ejecutar: ${q}`}
                  >
                    {q}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}