// ScoreBars.tsx (VS-13): desglose de score como barras horizontales apiladas.
// Longitud = encoding primario (Cleveland–McGill P7); color SOLO secundario:
// segmentos en gris escala (texto = foreground sólido, vector = muted rayado)
// + 1 accent neón en el residuo RRF. Encoding redundante: barra + número +
// tooltip (title). Sin explanation → barra de score sola, NO crashea.
import type { CSSProperties } from "react";
import type { ExplanationHit } from "../../../vanta";
import {
  computeSegments,
  type ScoreSegment,
} from "./retrieval-core";
import { tp, tt, type DesktopLang } from "../../../i18n";
import { connectionPrefs } from "../../../store/connections";

interface Props {
  /** Explanation del hit (puede faltar: search sin explain o backend sin soporte). */
  explanation?: ExplanationHit | null;
  /** Score del hit (del SearchResult, siempre presente). */
  score: number;
  /** Mayor score del conjunto — escala común para todas las barras. */
  maxScore: number;
  lang?: DesktopLang;
}

/** Estilo por tipo de segmento — gris escala + rayado CSS inline para el vector
 * (patrón ≠ color: legible en escala de grises y daltonismo, P7/Cleveland–McGill). */
const SEGMENT_STYLE: Record<ScoreSegment["key"], CSSProperties> = {
  text: { background: "var(--color-foreground, #000)" },
  vector: {
    background:
      "repeating-linear-gradient(135deg, var(--color-muted, #ECE6D8) 0 4px, var(--color-muted-foreground, #3A3A3A) 4px 8px)",
  },
  rrf: { background: "var(--color-neon, #FF5500)" },
};

/** Etiqueta de segmento en el idioma activo — retrieval-core sigue autónomo
 * (sin imports) y devuelve solo `key`; la traducción vive acá. */
const SEGMENT_LABEL_KEY: Record<ScoreSegment["key"], string> = {
  text: "retrieval.segText",
  vector: "retrieval.segVector",
  rrf: "retrieval.segRrf",
};
const SEGMENT_LABEL_FB: Record<ScoreSegment["key"], string> = {
  text: "texto (BM25)",
  vector: "vector (HNSW)",
  rrf: "fusión RRF",
};

export default function ScoreBars({ explanation, score, maxScore, lang = connectionPrefs.get().lang ?? "es" }: Props) {
  const bd = computeSegments(explanation, maxScore);
  const hasSegments = bd.segments.length > 0;
  const segLabel = (s: ScoreSegment) => tt(lang, SEGMENT_LABEL_KEY[s.key], SEGMENT_LABEL_FB[s.key]);

  // Tooltip: desglose por término BM25 (tf/df/doc_len/contribution) + ranks.
  const termLines =
    explanation?.bm25_terms?.map(
      (t) => `${t.token}: tf=${t.tf} df=${t.df} len=${t.doc_len} +${t.contribution.toFixed(4)}`,
    ) ?? [];
  const rankLine =
    explanation?.rrf_text_rank != null || explanation?.rrf_vector_rank != null
      ? tp(lang, "retrieval.rankLine", "texto rank={t} · vector rank={v}", { t: String(explanation.rrf_text_rank ?? "—"), v: String(explanation.rrf_vector_rank ?? "—") })
      : null;
  const tooltip = [
    `score ${score.toFixed(4)}`,
    ...bd.segments.map((s) => `${segLabel(s)}: ${s.value.toFixed(4)} (${s.widthPct.toFixed(1)}%)`),
    rankLine,
    termLines.length ? tp(lang, "retrieval.termsLine", "términos: {t}", { t: termLines.join(", ") }) : null,
    bd.missing ? tt(lang, "retrieval.noBreakdown", "sin desglose (explain off)") : null,
  ]
    .filter((l): l is string => !!l)
    .join("\n");

  const ariaLabel = bd.missing
    ? tp(lang, "retrieval.scoreNoBreakdown", "score {s} — sin desglose", { s: score.toFixed(4) })
    : tp(lang, "retrieval.scoreBreakdown", "score {s}: {segs}", {
        s: score.toFixed(4),
        segs: bd.segments.map((s) => `${segLabel(s)} ${s.value.toFixed(4)} (${s.widthPct.toFixed(1)}%)`).join(", "),
      });

  return (
    <div className="flex items-center gap-2" title={tooltip}>
      <div
        role="img"
        aria-label={ariaLabel}
        className="relative h-4 min-w-0 flex-1 overflow-hidden border-2 border-foreground bg-background"
      >
        {hasSegments ? (
          <div className="flex h-full w-full">
            {bd.segments.map((s) => (
              <div
                key={s.key}
                className="relative h-full"
                style={{ width: `${s.widthPct}%`, ...SEGMENT_STYLE[s.key] }}
              >
                {/* tooltip por segmento (redundancia fina sobre el global) */}
                <span className="sr-only">{`${segLabel(s)} ${s.value.toFixed(4)}`}</span>
              </div>
            ))}
          </div>
        ) : (
          /* Sin explanation: barra de score sola (sin segmentos, no crashea). */
          <div
            className="absolute inset-y-0 left-0"
            style={{
              width: `${maxScore > 0 ? Math.min(100, (score / maxScore) * 100) : 0}%`,
              ...SEGMENT_STYLE.vector,
            }}
          />
        )}
      </div>
      <span className="w-14 shrink-0 text-right font-tech text-[11px] text-foreground">
        {score.toFixed(3)}
      </span>
    </div>
  );
}