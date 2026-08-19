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

interface Props {
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
  dark: boolean;
}

export default function GraphLens({ onNotice, onError, dark }: Props) {
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
    <div className="flex h-[calc(100dvh-112px)] min-h-[480px] flex-col">
      {/* Toolbar mínimo (contrato) */}
      <div className="flex flex-wrap items-center gap-2 border-b-4 border-foreground bg-card px-4 py-2">
        <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
          grafo · {g.namespace ?? "—"}
        </span>
        <span className="font-tech text-[10px] text-muted-foreground">
          {g.nodeCount}/{MAX_NODES} nodos · {g.edgeCount} aristas
        </span>
        {highlightIds.size > 0 && (
          <span className="font-tech text-[10px] font-bold text-cyan-700" role="status">
            ● {highlightIds.size} resaltados
          </span>
        )}
        {g.busy && <span className="font-tech text-[10px] text-muted-foreground">expandiendo…</span>}
        {g.capped && (
          <span className="font-tech text-[10px] font-bold text-amber-700" role="status">
            ⚠ tope alcanzado — se desvanecen nodos viejos (click para re-expandir)
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
            title="Mostrar/ocultar consola IQL (GRAFO-03)"
          >
            ⌨ iql
          </button>
          <button
            type="button"
            onClick={fit}
            className="press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold"
            title="Ajustar vista al grafo"
          >
            ⛶ fit
          </button>
          <button
            type="button"
            onClick={handleReset}
            className="press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold"
            title="Reiniciar al seed (hubs del namespace)"
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
            title="Mostrar/ocultar labels (solo top-20 por degree)"
          >
            🏷 labels
          </button>
        </div>
      </div>

      {/* Pista de interacción */}
      <p className="border-b border-foreground/40 bg-background px-4 py-1 font-tech text-[10px] text-muted-foreground">
        click en un nodo → expande vecinos (≤50) · arrastrar = orbitar · scroll = zoom · click vacío = deseleccionar
        {consoleOpen && " · ctrl+enter en la consola = ejecutar iql"}
      </p>

      <div className={`flex-1 overflow-hidden ${consoleOpen ? "min-h-0" : ""}`}>
        <Suspense fallback={<div className="flex h-full items-center justify-center font-tech text-xs text-muted-foreground">cargando escena 3D…</div>}>
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

      {/* Consola IQL embebida (GRAFO-03) — panel inferior colapsable */}
      {consoleOpen && (
        <div className="h-[220px] shrink-0">
          <IqlConsole dark={dark} onHighlight={handleHighlight} onNotice={onNotice} onError={onError} />
        </div>
      )}
    </div>
  );
}