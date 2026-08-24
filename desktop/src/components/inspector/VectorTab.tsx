// Vector tab (VS-06): stats del vector denso (dimensión/norma L2/min/max),
// sparkline de valores reales, copiar JSON al portapapeles y pegar/analizar un
// JSON de vector (solo inspección — vantaPut no acepta vector en Fase 0).
import { useMemo, useState } from "react";
import type { MemoryRecord } from "../../vanta";

interface Props {
  record: MemoryRecord;
}

function vecStats(v: number[]) {
  let sum = 0;
  let min = Infinity;
  let max = -Infinity;
  for (const x of v) {
    sum += x * x;
    if (x < min) min = x;
    if (x > max) max = x;
  }
  return { dim: v.length, norm: Math.sqrt(sum), min, max };
}

/** Muestreo uniforme a `n` barras (sparkline). */
function sample(v: number[], n: number): number[] {
  const out: number[] = [];
  for (let i = 0; i < n; i++) {
    out.push(v[Math.min(v.length - 1, Math.floor((i / n) * v.length))]);
  }
  return out;
}

function Sparkline({ values }: { values: number[] }) {
  const bars = sample(values, 48);
  const lo = Math.min(...bars);
  const hi = Math.max(...bars);
  const range = hi - lo || 1;
  return (
    <div className="flex h-8 w-full items-end gap-px overflow-hidden border-2 border-foreground bg-card" aria-hidden="true">
      {bars.map((v, i) => {
        const h = 3 + ((v - lo) / range) * 19;
        return (
          <span
            key={i}
            className="flex-1 bg-foreground/80"
            style={{ height: `${Math.round(h)}px` }}
          />
        );
      })}
    </div>
  );
}

function Stats({ v }: { v: number[] }) {
  const s = vecStats(v);
  return (
    <dl className="space-y-1 font-tech text-[11px]">
      <div className="flex justify-between">
        <dt className="uppercase text-muted-foreground">dimensión</dt>
        <dd className="font-mono">{s.dim}</dd>
      </div>
      <div className="flex justify-between">
        <dt className="uppercase text-muted-foreground">norma L2</dt>
        <dd className="font-mono text-neon">{s.norm.toFixed(4)}</dd>
      </div>
      <div className="flex justify-between">
        <dt className="uppercase text-muted-foreground">min / max</dt>
        <dd className="font-mono">
          {s.min.toFixed(4)} / {s.max.toFixed(4)}
        </dd>
      </div>
    </dl>
  );
}

export default function VectorTab({ record }: Props) {
  const vec = record.vector ?? null;
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const [pasteText, setPasteText] = useState("");
  const [pasted, setPasted] = useState<number[] | null>(null);
  const [pasteError, setPasteError] = useState<string | null>(null);

  const pastedStats = useMemo(() => (pasted ? vecStats(pasted) : null), [pasted]);

  async function copyJson() {
    if (!vec) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(vec));
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      // clipboard no disponible — sin crash; el usuario copia a mano.
    }
  }

  function analyzePaste() {
    try {
      const parsed = JSON.parse(pasteText);
      if (!Array.isArray(parsed) || !parsed.every((x) => typeof x === "number")) {
        setPasteError("el JSON debe ser un array de números");
        setPasted(null);
        return;
      }
      setPasted(parsed);
      setPasteError(null);
    } catch {
      setPasteError("JSON inválido");
      setPasted(null);
    }
  }

  const sparseCount = record.sparse_vector ? Object.keys(record.sparse_vector).length : 0;

  return (
    <div>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="press flex w-full items-center justify-between border-2 border-foreground bg-background px-3 py-2 text-xs font-semibold"
        aria-expanded={open}
      >
        <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
          VECTOR {vec ? `${vec.length}d` : ""}
        </span>
        <span>{open ? "▾" : "▸"}</span>
      </button>

      {!open ? null : !vec ? (
        <div className="mt-2 border-2 border-foreground bg-background p-3">
          <p className="font-tech text-[10px] uppercase text-muted-foreground">
            sin vector — solo texto
          </p>
          {sparseCount > 0 && (
            <p className="mt-1 font-tech text-[10px] text-muted-foreground">
              sparse · {sparseCount} términos
            </p>
          )}
        </div>
      ) : (
        <div className="mt-2 space-y-3 border-2 border-foreground bg-background p-3">
          <Stats v={vec} />
          <Sparkline values={vec} />
          <div className="flex gap-2">
            <button
              type="button"
              onClick={copyJson}
              className="press flex-1 border-2 border-foreground bg-card px-2 py-1.5 font-tech text-[10px]"
            >
              {copied ? "✓ COPIADO" : "COPIAR JSON"}
            </button>
          </div>
          {sparseCount > 0 && (
            <p className="font-tech text-[10px] text-muted-foreground">
              sparse · {sparseCount} términos
            </p>
          )}

          <div className="border-t-2 border-dashed border-foreground pt-3">
            <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              pegar JSON (análisis local, no persiste)
            </div>
            <textarea
              value={pasteText}
              onChange={(e) => setPasteText(e.target.value)}
              placeholder="[0.1, -0.2, …]"
              rows={2}
              aria-label="JSON de vector a analizar"
              className="mt-1 w-full border-2 border-foreground bg-background px-2 py-1 font-mono text-[10px]"
            />
            <button
              type="button"
              onClick={analyzePaste}
              className="press mt-1 border-2 border-foreground bg-card px-2 py-1 font-tech text-[10px]"
            >
              ANALIZAR
            </button>
            {pasteError && <p className="mt-1 font-tech text-[10px] text-destructive">{pasteError}</p>}
            {pasted && pastedStats && (
              <div className="mt-2 space-y-2">
                <Stats v={pasted} />
                <Sparkline values={pasted} />
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}