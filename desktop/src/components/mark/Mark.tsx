import { useRef } from "react";
import type { ReactNode } from "react";
import { useMarkInteraction, usePrefersReducedMotion } from "./use-mark-interaction";
import type { BlinkState } from "./use-mark-interaction";
import "./mark.css";

/**
 * Mark — VantaDB mascot, desktop variant (VS-02).
 * Port of web/src/components/vanta/mark/mark-classic.tsx WITHOUT Anime.js:
 *  - follow: rAF exponential lerp (hook)
 *  - squint: React-computed eye height from mouse distance (hook)
 *  - blink: WAAPI over y/height (hook)
 *  - node pulse: CSS keyframes with transform-box: fill-box
 *  - glow ring: SMIL, not rendered under prefers-reduced-motion
 * Declared loss vs web: outElastic ease of the node pulse → backOut
 * cubic-bezier(0.34, 1.56, 0.64, 1) approximation.
 *
 * The graph and face are exported so MarkStudio reuses the exact same SVG.
 */

const GRAPH_NODES = [
  { x: 12, y: 25, r: 2.2 },
  { x: 88, y: 22, r: 2.8 },
  { x: 22, y: 75, r: 2.2 },
  { x: 78, y: 80, r: 2.4 },
  { x: 50, y: 15, r: 1.6 },
  { x: 94, y: 55, r: 1.8 },
  { x: 6, y: 55, r: 1.8 },
  { x: 50, y: 92, r: 1.6 },
  { x: 35, y: 45, r: 1.4 },
  { x: 68, y: 50, r: 1.4 },
];

const GRAPH_EDGES: [number, number][] = [
  [0, 4], [4, 1], [1, 5], [5, 3], [3, 7], [7, 2], [2, 6], [6, 0],
  [0, 1], [2, 3], [4, 7], [5, 6],
  [8, 4], [8, 0], [8, 2], [9, 5], [9, 1], [9, 3],
];

export function Mark() {
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

  return (
    <div className="vmark" onClick={handleClick}>
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

      {/* Floating annotation labels (manga SFX) */}
      <SfxLabel className="vmark-pos-tl" rotate={-6} color="neon">1.2ms</SfxLabel>
      <SfxLabel className="vmark-pos-tr" rotate={5} color="ink">RRF</SfxLabel>
      <SfxLabel className="vmark-pos-bl" rotate={-3} color="ink">WAL · CRC32C</SfxLabel>
      <SfxLabel className="vmark-pos-br" rotate={4} color="neon">ZERO NET</SfxLabel>

      {/* Corner clip tag */}
      <div className="vmark-tag">IN-PROCESS</div>

      {/* Interactive hint */}
      {mouseInHero && (
        <div className="vmark-hint" role="status">
          {annoyed ? "◆ blink" : "◆ click me · move mouse"}
        </div>
      )}
    </div>
  );
}

export function MarkGraph({
  hoveredNode,
  onNodeHover,
  onNodeClick,
}: {
  hoveredNode: number | null;
  onNodeHover: (idx: number | null) => void;
  onNodeClick: () => void;
}) {
  return (
    <svg className="vmark-graph" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
      {GRAPH_EDGES.map(([a, b], i) => (
        <line
          key={`edge-${i}`}
          x1={GRAPH_NODES[a].x}
          y1={GRAPH_NODES[a].y}
          x2={GRAPH_NODES[b].x}
          y2={GRAPH_NODES[b].y}
          stroke={hoveredNode === a || hoveredNode === b ? "#FF5500" : "currentColor"}
          strokeWidth={hoveredNode === a || hoveredNode === b ? 0.4 : 0.25}
          strokeDasharray="1 1.2"
          className="vmark-edge"
          opacity={hoveredNode === a || hoveredNode === b ? 0.95 : 0.5}
          pointerEvents="none"
        />
      ))}
      {GRAPH_NODES.map((node, i) => (
        <g key={`node-${i}`}>
          {/* Visible node — CSS pulse on hover, ambient stagger pulse otherwise */}
          <circle
            cx={node.x}
            cy={node.y}
            r={node.r}
            fill={hoveredNode === i ? "#FF5500" : "currentColor"}
            className={`vmark-node ${hoveredNode === i ? "vmark-node-pulse" : "vmark-node-ambient"}`}
            style={{ animationDelay: hoveredNode === i ? undefined : `${i * 180}ms` }}
          />
          {/* Invisible hit-target (r=5) — hover + click */}
          <circle
            cx={node.x}
            cy={node.y}
            r="5"
            fill="transparent"
            className="vmark-node"
            onMouseEnter={() => onNodeHover(i)}
            onMouseLeave={() => onNodeHover(null)}
            onClick={(e) => {
              e.stopPropagation();
              onNodeClick();
            }}
          />
        </g>
      ))}
    </svg>
  );
}

export function MarkFace({
  pupilOffset,
  sphereOffset,
  blink,
  annoyed,
  squintHeight,
  leftEyeRef,
  rightEyeRef,
}: {
  pupilOffset: { x: number; y: number };
  sphereOffset: { x: number; y: number };
  blink: BlinkState;
  annoyed: boolean;
  squintHeight: number;
  leftEyeRef: React.RefObject<SVGRectElement | null>;
  rightEyeRef: React.RefObject<SVGRectElement | null>;
}) {
  const reduced = usePrefersReducedMotion();
  return (
    <svg
      className="vmark-face"
      viewBox="0 0 100 100"
      fill="none"
      aria-label="VantaDB interactive mark — click to blink, move mouse to track"
    >
      {/* Outer ring — black border, NO fill */}
      <circle cx="50" cy="50" r="42" fill="none" stroke="#000" strokeWidth="3.5" />

      {/* Subtle glow ring (SMIL) — not rendered under reduced motion */}
      {!reduced && (
        <circle cx="50" cy="50" r="42" fill="none" stroke="#FF5500" strokeWidth="0.6" opacity="0.3">
          <animate attributeName="r" values="42;46;42" dur="3.5s" repeatCount="indefinite" />
          <animate attributeName="opacity" values="0.3;0;0.3" dur="3.5s" repeatCount="indefinite" />
        </circle>
      )}

      {/* Orange sphere — follows mouse (rAF lerp via hook) */}
      <circle
        cx={50 + sphereOffset.x}
        cy={50 + sphereOffset.y}
        r="22"
        fill="#FF5500"
        style={{
          transform: annoyed ? "scale(0.94)" : "scale(1)",
          transformOrigin: `${50 + sphereOffset.x}px ${50 + sphereOffset.y}px`,
          transition: "transform 0.3s ease-out",
        }}
      />

      {/* Eyes — two vertical bars, rounded. While blinking, the blinking
          eye's y/height are omitted so the WAAPI animation owns them. */}
      <rect
        ref={leftEyeRef}
        x={43 + pupilOffset.x - 2}
        {...(blink === "left-closed" || blink === "both-closed" ? {} : { y: 45 + pupilOffset.y, height: squintHeight })}
        width="4"
        fill="#000"
        rx="2"
      />
      <rect
        ref={rightEyeRef}
        x={57 + pupilOffset.x - 2}
        {...(blink === "right-closed" || blink === "both-closed" ? {} : { y: 45 + pupilOffset.y, height: squintHeight })}
        width="4"
        fill="#000"
        rx="2"
      />
    </svg>
  );
}

export function SfxLabel({
  children,
  className = "",
  rotate = 0,
  color = "ink",
}: {
  children: ReactNode;
  className?: string;
  rotate?: number;
  color?: "ink" | "neon";
}) {
  return (
    <span
      className={`vmark-label ${color === "neon" ? "vmark-label-neon" : "vmark-label-ink"} ${className}`}
      style={{ transform: `rotate(${rotate}deg)` }}
    >
      {children}
    </span>
  );
}