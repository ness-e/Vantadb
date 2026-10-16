import { useRef } from "react";
import { useMarkInteraction } from "./use-mark-interaction";
import { MarkFace, MarkGraph, SfxLabel } from "./Mark";

/**
 * MarkStudio — Mark variant driven by workspace status (VS-02).
 * Reuses the exact same graph, face, interaction hook and SFX labels as the
 * classic Mark; only the copy (labels / tag / hint) changes per status.
 */

export type MarkStudioStatus = "idle" | "loading" | "empty" | "error";

interface LabelSpec {
  text: string;
  color: "neon" | "ink";
}

interface StudioCopy {
  tl: LabelSpec;
  tr: LabelSpec;
  bl: LabelSpec;
  br: LabelSpec;
  tag: string;
  tagClass: string;
  hint: string;
}

const STUDIO_COPY: Record<MarkStudioStatus, StudioCopy> = {
  idle: {
    tl: { text: "1.2ms", color: "neon" },
    tr: { text: "RRF", color: "ink" },
    bl: { text: "WAL · CRC32C", color: "ink" },
    br: { text: "ZERO NET", color: "neon" },
    tag: "IN-PROCESS",
    tagClass: "",
    hint: "◆ click me · move mouse",
  },
  loading: {
    tl: { text: "PARSING", color: "neon" },
    tr: { text: "HNSW", color: "ink" },
    bl: { text: "WAL · FLUSH", color: "ink" },
    br: { text: "HOT", color: "neon" },
    tag: "LOADING",
    tagClass: "",
    hint: "◆ working…",
  },
  empty: {
    tl: { text: "0 BYTES", color: "neon" },
    tr: { text: "EMPTY", color: "ink" },
    bl: { text: "NO DATA", color: "ink" },
    br: { text: "AWAIT", color: "neon" },
    tag: "EMPTY",
    tagClass: "vmark-tag-cream",
    hint: "◆ nothing here — put a memory",
  },
  error: {
    tl: { text: "ERR", color: "neon" },
    tr: { text: "WAL · DIRTY", color: "ink" },
    bl: { text: "RETRY", color: "ink" },
    br: { text: "DOWN", color: "neon" },
    tag: "ERROR",
    tagClass: "vmark-tag-ink",
    hint: "◆ connection lost — retry",
  },
};

export function MarkStudio({ status = "idle" }: { status?: MarkStudioStatus }) {
  const markWrapRef = useRef<HTMLDivElement>(null);
  const {
    state,
    handleClick,
    handleNodeHover,
    leftEyeRef,
    rightEyeRef,
    squintHeight,
  } = useMarkInteraction(markWrapRef, {
    maxEyeOffset: 16,
    maxSphereOffset: 7,
    squintDistance: 600,
    maxEyeHeight: 10,
    minEyeHeight: 3,
  });
  const { pupilOffset, sphereOffset, blink, annoyed, hoveredNode, mouseInHero } = state;
  const copy = STUDIO_COPY[status];

  return (
    <div className="vmark" onClick={handleClick} data-status={status}>
      <MarkGraph hoveredNode={hoveredNode} onNodeHover={handleNodeHover} onNodeClick={handleClick} />

      <div ref={markWrapRef} className="vmark-face-wrap">
        <MarkFace
          pupilOffset={pupilOffset}
          sphereOffset={sphereOffset}
          blink={blink}
          annoyed={annoyed}
          squintHeight={squintHeight}
          leftEyeRef={leftEyeRef}
          rightEyeRef={rightEyeRef}
        />
      </div>

      <SfxLabel className="vmark-pos-tl" rotate={-6} color={copy.tl.color}>{copy.tl.text}</SfxLabel>
      <SfxLabel className="vmark-pos-tr" rotate={5} color={copy.tr.color}>{copy.tr.text}</SfxLabel>
      <SfxLabel className="vmark-pos-bl" rotate={-3} color={copy.bl.color}>{copy.bl.text}</SfxLabel>
      <SfxLabel className="vmark-pos-br" rotate={4} color={copy.br.color}>{copy.br.text}</SfxLabel>

      <div className={`vmark-tag ${copy.tagClass}`}>{copy.tag}</div>

      {mouseInHero && (
        <div className="vmark-hint" role="status">
          {annoyed ? "◆ blink" : copy.hint}
        </div>
      )}
    </div>
  );
}