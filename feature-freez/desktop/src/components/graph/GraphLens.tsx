// GraphLens.tsx (GRAFO-02 + GRAFO-03): lente GRAFO de la surface IQL — visor
// 3D R3F del grafo con expand incremental + consola IQL embebida (GRAFO-03).
// Contrato: nodos toon naranja, aristas líneas, click → BFS desde el nodo
// (limit 50), cap 500 con fade, size ∝ degree, toolbar mínimo (fit / reset /
// labels top-N), nodo activo con halo. La consola ejecuta `queryIql()` y el
// resultado Read resalta los nodos en el canvas (highlightIds → GraphScene).
//
// Chunk lazy en WorkspaceShell (patrón Inspector/CommandPalette): three+drei
// pesan ~600 kB y solo los paga la surface IQL (mitigación "Riesgos" del plan).
import { Canvas } from "@react-three/fiber";
import { Suspense, useCallback, useState } from "react";
import GraphScene from "./GraphScene";
import IqlConsole from "./IqlConsole";
import { MAX_NODES, useGraphData } from "./useGraphData";
import LensShell from "../layout/LensShell";
import { Tag, TriangleAlert } from "lucide-react";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

interface Props {
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
  dark: boolean;
  lang?: DesktopLang;
}

export default function GraphLens({ onNotice, onError, dark, lang = connectionPrefs.get().lang ?? "es" }: Props) {
  const [showLabels, setShowLabels] = useState(true);
  const [fitSignal, setFitSignal] = useState(0);
  const [consoleOpen, setConsoleOpen] = useState(true);
  // Nodos resaltados por la consola IQL (resultado Read) — GRAFO-03. Estado de
  // sesión de la consola, NO del grafo: useGraphData no lo conoce.
  const [highlightIds, setHighlightIds] = useState<ReadonlySet<string>>(new Set());
  const g = useGraphData(onNotice, onError);

  const fit = useCallback(() => setFitSignal((s) => s + 1), []);
  const handleReset = useCallback(() => {
    g.reset();
    fit();
  }, [g, fit]);
  const handleHighlight = useCallback((ids: string[]) => {
    setHighlightIds(new Set(ids));
  }, []);

  return (
    // UX-15: lens-height = utilidad compartida (antes el magic number
    // h-[calc(100dvh-112px)] min-h-[480px] estaba duplicado con SpaceLens).
    <div className="flex lens-height flex-col">
      {/* Header + toolbar (UX-01: LensShell compartido) */}
      <div className="border-b-4 border-foreground bg-card px-4 py-3">
        <LensShell
          title="GRAFO"
          meta={`${g.namespace ?? "—"} · ${g.nodeCount}/${MAX_NODES} nodos · ${g.edgeCount} aristas`}
        />
        <div className="mt-2 flex flex-wrap items-center gap-2">
        {highlightIds.size > 0 && (
          <span className="font-tech text-[10px] font-bold text-cyan-700 dark:text-cyan-300" role="status">
            {tp(lang, "graph.highlighted", "● {n} resaltados", { n: String(highlightIds.size) })}
          </span>
        )}
        {g.busy && <span className="font-tech text-[10px] text-muted-foreground">{tt(lang, "graph.expanding", "expandiendo…")}</span>}
        {g.capped && (
          <span className="font-tech text-[10px] font-bold text-amber-700 dark:text-amber-300" role="status">
            <TriangleAlert className="mr-0.5 inline h-3 w-3 align-[-1px]" strokeWidth={2.5} aria-hidden="true" />
            {tt(lang, "graph.capped", "tope alcanzado — se desvanecen nodos viejos (click para re-expandir)")}
          </span>
        )}
        <div className="ml-auto flex items-center gap-2">
          <button
            type="button"
            aria-pressed={consoleOpen}
            onClick={() => setConsoleOpen((v) => !v)}
            className={`press border-2 border-foreground px-2 py-1 text-[10px] font-semibold ${
              consoleOpen ? "bg-neon text-background" : "bg-background"
            }`}
            title={tt(lang, "graph.iqlTitle", "Mostrar/ocultar consola IQL (GRAFO-03)")}
          >
            ⌨ iql
          </button>
          <button
            type="button"
            onClick={fit}
            className="press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold"
            title={tt(lang, "graph.fitTitle", "Ajustar vista al grafo")}
          >
            ⛶ fit
          </button>
          <button
            type="button"
            onClick={handleReset}
            className="press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold"
            title={tt(lang, "graph.resetTitle", "Reiniciar al seed (hubs del namespace)")}
          >
            ↺ reset
          </button>
          <button
            type="button"
            aria-pressed={showLabels}
            onClick={() => setShowLabels((v) => !v)}
            className={`press border-2 border-foreground px-2 py-1 text-[10px] font-semibold ${
              showLabels ? "bg-neon text-background" : "bg-background"
            }`}
            title={tt(lang, "graph.labelsTitle", "Mostrar/ocultar labels (solo top-20 por degree)")}
          >
            <Tag className="mr-1 inline h-3 w-3 align-[-1px]" strokeWidth={2.5} aria-hidden="true" /> {tt(lang, "graph.labels", "labels")}
          </button>
        </div>
        </div>
      </div>

      {/* Pista de interacción */}
      <p className="border-b border-foreground/40 bg-background px-4 py-1 font-tech text-[10px] text-muted-foreground">
        {tt(lang, "graph.hint", "click en un nodo → expande vecinos (≤50) · arrastrar = orbitar · scroll = zoom · click vacío = deseleccionar")}
        {consoleOpen && tt(lang, "graph.hintConsole", " · ctrl+enter en la consola = ejecutar iql")}
      </p>

      <div
        role="img"
        aria-label={tp(lang, "graph.canvasAria", "Grafo de {nodes} nodos y {edges} aristas{ns}. Click en un nodo expande hasta 50 vecinos; la consola IQL permite consultar.", {
          nodes: String(g.nodeCount),
          edges: String(g.edgeCount),
          ns: g.namespace ? tp(lang, "graph.canvasNs", " del namespace {ns}", { ns: g.namespace }) : "",
        })}
        className={`flex-1 overflow-hidden ${consoleOpen ? "min-h-0" : ""}`}
      >
        <Suspense fallback={<div className="flex h-full items-center justify-center font-tech text-xs text-muted-foreground">{tt(lang, "graph.loading3d", "cargando escena 3D…")}</div>}>
          <Canvas camera={{ position: [0, 0, 24], fov: 50 }} dpr={[1, 1.5]} onPointerMissed={() => g.setActiveId(null)}>
            <GraphScene
              nodes={g.nodes}
              edges={g.edges}
              revision={g.revision}
              activeId={g.activeId}
              highlightIds={highlightIds}
              showLabels={showLabels}
              fitSignal={fitSignal}
              onSelectNode={(id) => g.expand(id)}
            />
          </Canvas>
        </Suspense>
      </div>

      {/* UX-08: alternativa accesible al canvas 3D — lista de nodos para
          teclado/SR (sr-only; vive fuera del role="img" para no ocultarse). */}
      <ul className="sr-only" aria-label={tt(lang, "graph.nodesAria", "Nodos visibles del grafo")}>
        {g.nodes.map((n) => (
          <li key={n.id}>{n.label || n.id}</li>
        ))}
      </ul>

      {/* Consola IQL embebida (GRAFO-03) — panel inferior colapsable */}
      {consoleOpen && (
        <div className="h-[220px] shrink-0">
          <IqlConsole dark={dark} lang={lang} onHighlight={handleHighlight} onNotice={onNotice} onError={onError} />
        </div>
      )}
    </div>
  );
}