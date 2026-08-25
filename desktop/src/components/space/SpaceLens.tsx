// SpaceLens.tsx (ESPACIO-01/02): lente ESPACIO — scatterplot WebGL de embeddings.
// Contrato: list → worker UMAP-js (seed fijo, reproducible) → regl-scatterplot
// con hover (tooltip: key + payload) + selección por lasso (SHIFT+drag).
// ESPACIO-02: la selección habilita la SelectionBar — Exportar (JSONL
// client-side, importable 1:1) y Eliminar (softDeleteBatch → papelera + undo).
//
// Chunk lazy en WorkspaceShell (patrón GraphLens/Inspector): regl-scatterplot
// + regl pesan ~200 kB y solo los paga esta surface.
import createScatterplot from "regl-scatterplot";
import { useEffect, useRef, useState } from "react";
import type { MemoryRecord } from "../../vanta";
import { vantaErrorMessage } from "../../vanta";
import { useProjection, type ProjectionPoint } from "./useProjection";
import SelectionBar, { type SelectionBusy } from "./SelectionBar";
import { recordsToJsonl, downloadText } from "../export/export-jsonl";
import { undoStore } from "../../store/undo";
import { TriangleAlert } from "lucide-react";

function filenameStamp(): string {
  return new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
}

interface Props {
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
  dark: boolean;
  /** Click en un punto → abrir el registro en el Inspector (patrón RetrievalLens). */
  onOpenRecord: (record: MemoryRecord, score: number | null) => void;
}

/** Paleta categórica por namespace (neón D4; `pointColor` indexado por valueA). */
const PALETTE = [
  "#22d3ee",
  "#e879f9",
  "#a3e635",
  "#fbbf24",
  "#a78bfa",
  "#fb7185",
  "#34d399",
  "#38bdf8",
  "#f472b6",
  "#c084fc",
];

function preview(record: MemoryRecord): string {
  if (record.text) {
    const t = record.text.replace(/\s+/g, " ").trim();
    return t.length > 140 ? `${t.slice(0, 137)}…` : t;
  }
  return JSON.stringify(record.metadata ?? {}).slice(0, 140);
}

export default function SpaceLens({ onNotice, onError, dark, onOpenRecord }: Props) {
  const { state, project, setSelected } = useProjection();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const plotRef = useRef<ReturnType<typeof createScatterplot> | null>(null);
  const lassoActiveRef = useRef(false);
  const pointsRef = useRef<ProjectionPoint[]>([]);
  const [namespace, setNamespace] = useState<string>("");
  const [hovered, setHovered] = useState<number | null>(null);
  const [busy, setBusy] = useState<SelectionBusy>(null);

  const { points, namespaces } = state;
  pointsRef.current = points;
  const hoverPoint: ProjectionPoint | undefined =
    hovered != null ? points[hovered] : undefined;

  // ESPACIO-02: la selección del lasso (índices en `points`) → records reales.
  const selectedRecords: MemoryRecord[] = [...state.selected]
    .map((i) => points[i]?.record)
    .filter((r): r is MemoryRecord => r != null);

  // Crear el scatterplot una sola vez; draw ocurre en `tick` (ver abajo).
  useEffect(() => {
    const container = containerRef.current;
    const canvas = canvasRef.current;
    if (!container || !canvas) return;
    // ponytail: resize 1:1 al contenedor; 'auto' en width/height lo hace solo.
    const plot = createScatterplot({
      canvas,
      width: container.clientWidth,
      height: container.clientHeight,
      backgroundColor: dark ? "#0b0b0f" : "#ffffff",
      pointSize: 6,
      pointSizeSelected: 12,
      colorBy: "valueA",
      pointColor: PALETTE,
      pointColorHover: "#FF5500",
      deselectOnEscape: true,
    });
    plotRef.current = plot;

    if (!plot.isSupported) {
      onError("WebGL no soportado — no se puede renderizar el scatterplot");
    }

    plot.subscribe("pointOver", (idx: number) => setHovered(idx));
    plot.subscribe("pointOut", () => setHovered(null));
    plot.subscribe("select", ({ points: sel }: { points: number[] }) => {
      if (!lassoActiveRef.current && sel.length === 1) {
        // Click simple en un punto → Inspector (patrón RetrievalLens).
        const p = pointsRef.current[sel[0]];
        if (p) onOpenRecord(p.record, null);
      } else {
        setSelected(new Set(sel));
      }
    });
    plot.subscribe("deselect", () => setSelected(new Set()));
    plot.subscribe("lassoStart", () => {
      lassoActiveRef.current = true;
    });
    plot.subscribe("lassoEnd", () => {
      lassoActiveRef.current = false;
    });

    const ro = new ResizeObserver(() => {
      void plot.set({
        width: container.clientWidth,
        height: container.clientHeight,
      });
    });
    ro.observe(container);

    return () => {
      ro.disconnect();
      // destroy() limpia los listeners de pub-sub-es internos.
      plot.destroy();
      plotRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dark]);

  // Dibujar los puntos proyectados cuando `points` cambia (column-oriented NDC).
  useEffect(() => {
    const plot = plotRef.current;
    if (!plot || points.length === 0) return;
    const x = new Float32Array(points.length);
    const y = new Float32Array(points.length);
    const valueA = new Float32Array(points.length);
    points.forEach((p, i) => {
      x[i] = p.x;
      y[i] = p.y;
      valueA[i] = p.colorKey;
    });
    void plot.draw({ x, y, valueA });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [points]);

  // Auto-proyección al montar (sin backend → error visible, igual que GraphLens).
  const firstRun = useRef(true);
  useEffect(() => {
    if (!firstRun.current) return;
    firstRun.current = false;
    void project(namespace || undefined);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Aviso único al completar una proyección (no spamea onNotice por re-render).
  const noticedDone = useRef(false);
  useEffect(() => {
    if (state.phase === "done" && !noticedDone.current) {
      noticedDone.current = true;
      onNotice(`Proyección lista: ${state.points.length} puntos (seed fijo)`);
    } else if (state.phase !== "done") {
      noticedDone.current = false;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.phase, state.points.length]);

  const selectedCount = state.selected.size;

  /** ESPACIO-02: exportar la selección como JSONL importable 1:1 (patrón
   * ExportButtons — la app no tiene plugin de dialog; download es el mínimo). */
  async function handleExport() {
    if (busy || selectedRecords.length === 0) return;
    setBusy("export");
    try {
      downloadText(`vanta-selection-${filenameStamp()}.jsonl`, recordsToJsonl(selectedRecords));
      onNotice(`exportados ${selectedRecords.length} registros (JSONL importable 1:1)`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(null);
    }
  }

  /** ESPACIO-02: eliminar la selección → papelera (undoStore, snapshot previo).
   * Confirmación explícita nombra el impacto (05 anti-patrón 7); un solo
   * Ctrl+Z restaura el lote completo (VS-08 softDeleteBatch). */
  async function handleDelete() {
    if (busy || selectedRecords.length === 0) return;
    const ok = window.confirm(
      `Borrar ${selectedRecords.length} registro(s) seleccionado(s)?\n` +
        "Se mueven a la papelera de sesión — Ctrl+Z los restaura.",
    );
    if (!ok) return;
    setBusy("delete");
    try {
      await undoStore.softDeleteBatch(selectedRecords);
      setSelected(new Set());
      onNotice(
        `movidos a papelera ${selectedRecords.length} registros — Ctrl+Z deshace el lote`,
      );
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="flex h-[calc(100dvh-112px)] min-h-[480px] flex-col">
      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-2 border-b-4 border-foreground bg-card px-4 py-2">
        <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
          espacio · {namespace || "todos"}
        </span>
        <span className="font-tech text-[10px] text-muted-foreground">
          {state.phase === "done" ? `${points.length} puntos` : state.phase}
        </span>
        {state.phase === "loading" && (
          <span className="font-tech text-[10px] text-muted-foreground">proyectando…</span>
        )}
        {state.error && (
          <span className="font-tech text-[10px] font-bold text-destructive" role="alert">
            <TriangleAlert className="mr-0.5 inline h-3 w-3 align-[-1px]" strokeWidth={2.5} aria-hidden="true" />
            {state.error}
          </span>
        )}
        <div className="ml-auto flex items-center gap-2">
          <select
            value={namespace}
            onChange={(e) => setNamespace(e.target.value)}
            aria-label="Namespace a proyectar"
            className="border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold"
          >
            <option value="">todos los namespaces</option>
            {namespaces.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => void project(namespace || undefined)}
            className="press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold"
            title="Re-proyectar con UMAP-js (seed fijo = reproducible)"
          >
            ⤒ proyectar
          </button>
        </div>
      </div>

      {/* ESPACIO-02: batch ops sobre la selección (contador + export/delete/clear). */}
      <SelectionBar
        count={selectedCount}
        busy={busy}
        onExport={() => void handleExport()}
        onDelete={() => void handleDelete()}
        onClear={() => setSelected(new Set())}
      />

      {/* Pista de interacción */}
      <p className="border-b border-foreground/40 bg-background px-4 py-1 font-tech text-[10px] text-muted-foreground">
        hover = ver payload · click en un punto = abrir en Inspector · shift+arrastrar = lasso
        → barra de batch ops
        <span className="text-amber-700 dark:text-amber-300"> · UMAP-js distorsiona distancias — solo agrupa por vecindad</span>
      </p>

      {/* Canvas + tooltip overlay */}
      <div ref={containerRef} className="relative flex-1 overflow-hidden">
        {state.phase === "done" && points.length > 0 ? (
          <>
            <canvas ref={canvasRef} className="h-full w-full" />
            {hoverPoint && (
              <div
                className="pointer-events-none absolute bottom-2 left-2 z-10 max-w-[320px] border-2 border-foreground bg-background/95 px-3 py-2 font-tech text-[10px] shadow-ink-sm"
                role="tooltip"
              >
                <div className="font-bold text-neon">
                  {hoverPoint.record.namespace}/{hoverPoint.record.id}
                </div>
                <div className="mt-1 text-muted-foreground">{preview(hoverPoint.record)}</div>
              </div>
            )}
          </>
        ) : (
          <div className="flex h-full items-center justify-center font-tech text-xs text-muted-foreground">
            {state.phase === "loading"
              ? "proyectando embeddings… (worker UMAP-js)"
              : state.phase === "error"
                ? state.error
                : "sin proyección — click en «⤒ proyectar»"}
          </div>
        )}
      </div>
    </div>
  );
}

export type { Props as SpaceLensProps };