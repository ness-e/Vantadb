"use client";

import { useRef, useEffect, useCallback } from "react";
import { animate } from "animejs";
import { cn } from "@/lib/utils";
import { useMarkInteraction } from "./use-mark-interaction";

/**
 * MarkClassic — the original VantaDB mark variant.
 *  - Outer ring: black border, NO fill (transparent)
 *  - Orange sphere: follows mouse smoothly (Anime.js)
 *  - Two vertical bar eyes: rounded, thicker, shorter — follow mouse + squint with distance
 *  - Graph nodes: interactive (hover = neon + pulse + mark looks) via Anime.js
 *  - Click: blink cycle via Anime.js (collapse + re-expand with outBack)
 *
 * Native, box-less — sits directly on the hero background.
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

export function MarkClassic() {
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
    eyeSmoothMs: 180,
    sphereSmoothMs: 380,
    squintDistance: 600,
    maxEyeHeight: 10,
    minEyeHeight: 3,
  });

  const { pupilOffset, sphereOffset, blink, annoyed, hoveredNode, mouseInHero } = state;

  // Refs for graph node circles (for Anime.js pulse on hover)
  const nodeRefs = useRef<(SVGCircleElement | null)[]>([]);

  // Anime.js: pulse animation when a node is hovered
  useEffect(() => {
    if (hoveredNode === null) return;
    const nodeEl = nodeRefs.current[hoveredNode];
    if (!nodeEl) return;
    // Pulse: scale up + back, with neon glow
    animate(nodeEl, {
      r: [GRAPH_NODES[hoveredNode].r, GRAPH_NODES[hoveredNode].r * 2.5, GRAPH_NODES[hoveredNode].r * 2.2],
      duration: 400,
      ease: "outElastic(1, 0.6)",
    });
  }, [hoveredNode]);

  // Anime.js: stagger pulse all nodes on mount (subtle ambient animation)
  // Skipped under prefers-reduced-motion; instances paused on unmount
  // (anime.js does NOT auto-cleanup on unmount).
  const ambientAnimsRef = useRef<ReturnType<typeof animate>[]>([]);
  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const validNodes = nodeRefs.current.filter(Boolean) as SVGCircleElement[];
    if (validNodes.length === 0) return;
    // Subtle ambient pulse — each node pulses at slightly different time
    validNodes.forEach((node, i) => {
      ambientAnimsRef.current.push(
        animate(node, {
          r: [
            GRAPH_NODES[i].r,
            GRAPH_NODES[i].r * 1.3,
            GRAPH_NODES[i].r,
          ],
          duration: 2400,
          delay: i * 180,
          ease: "inOutSine",
          loop: true,
        })
      );
    });
    return () => {
      ambientAnimsRef.current.forEach((anim) => anim.pause());
      ambientAnimsRef.current = [];
    };
  }, []);

  // Handler for node hover with Anime.js
  const onNodeHover = useCallback((idx: number | null) => {
    handleNodeHover(idx);
  }, [handleNodeHover]);

  return (
    <div
      className="relative mx-auto aspect-square w-full max-w-[460px] cursor-pointer lg:max-w-none"
      onClick={handleClick}
    >
      {/* Background graph — interactive nodes + edges */}
      <svg
        className="absolute inset-0 h-full w-full"
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
      >
        {/* Edges */}
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
            className="text-black/25 transition-all duration-300 "
            opacity={hoveredNode === a || hoveredNode === b ? 0.95 : 0.5}
            pointerEvents="none"
          />
        ))}
        {/* Nodes — interactive, Anime.js pulse on hover */}
        {GRAPH_NODES.map((node, i) => (
          <g key={`node-${i}`}>
            <circle
              ref={(el) => { nodeRefs.current[i] = el; }}
              cx={node.x}
              cy={node.y}
              r={node.r}
              fill={hoveredNode === i ? "#FF5500" : "currentColor"}
              className="text-black/35  cursor-pointer"
              style={{ transformOrigin: `${node.x}px ${node.y}px` }}
            />
            <circle
              cx={node.x}
              cy={node.y}
              r="5"
              fill="transparent"
              className="cursor-pointer"
              onMouseEnter={() => onNodeHover(i)}
              onMouseLeave={() => onNodeHover(null)}
              onClick={(e) => {
                e.stopPropagation();
                handleClick();
              }}
            />
          </g>
        ))}
      </svg>

      {/* The mark — outer ring (border only), native on hero bg */}
      <div ref={markWrapRef} className="absolute inset-0 flex items-center justify-center">
        <svg
          width="78%"
          height="78%"
          viewBox="0 0 100 100"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          aria-label="VantaDB interactive mark — click to blink, move mouse to track"
          className="drop-shadow-[0_0_28px_rgba(255,85,0,0.22)]"
        >
          {/* Outer ring — black border, NO fill */}
          <circle
            cx="50"
            cy="50"
            r="42"
            fill="none"
            stroke="currentColor"
            strokeWidth="3.5"
            className="text-black "
          />

          {/* Subtle glow ring (pulses softly) */}
          <circle cx="50" cy="50" r="42" fill="none" stroke="#FF5500" strokeWidth="0.6" opacity="0.3">
            <animate attributeName="r" values="42;46;42" dur="3.5s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.3;0;0.3" dur="3.5s" repeatCount="indefinite" />
          </circle>

          {/* Orange sphere — follows mouse (Anime.js smooth via hook) */}
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

          {/* EYES — two vertical bars, rounded.
              - x follows mouse via pupilOffset.x (anime.js smooth)
              - When NOT blinking: y=45+pupilOffset.y, height=squintHeight (React controls)
              - When blinking: ONLY the blinking eye's y/height are omitted so anime.js
                controls them. The other eye keeps React attributes. */}
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
      </div>

      {/* Floating annotation labels (manga SFX) */}
      <SfxLabel className="left-0 top-0" rotate={-6} color="neon">
        1.2ms
      </SfxLabel>
      <SfxLabel className="right-0 top-6" rotate={5} color="ink">
        RRF
      </SfxLabel>
      <SfxLabel className="bottom-6 left-0" rotate={-3} color="ink">
        WAL · CRC32C
      </SfxLabel>
      <SfxLabel className="bottom-0 right-0" rotate={4} color="neon">
        ZERO NET
      </SfxLabel>

      {/* Corner clip tag */}
      <div className="absolute -bottom-4 -left-4 z-20 rotate-[-4deg] border-4 border-black bg-[#FF5500] px-3 py-1 font-display text-sm uppercase text-black shadow-[4px_4px_0_0_#000]">
        IN-PROCESS
      </div>

      {/* Interactive hint */}
      {mouseInHero && (
        <div className="absolute bottom-3 right-3 z-20 border-2 border-black bg-black px-2 py-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
          {annoyed ? "◆ blink" : "◆ click me · move mouse"}
        </div>
      )}
    </div>
  );
}

function SfxLabel({
  children,
  className,
  rotate = 0,
  color = "ink",
}: {
  children: React.ReactNode;
  className?: string;
  rotate?: number;
  color?: "ink" | "neon";
}) {
  return (
    <span
      className={cn(
        "absolute z-20 inline-flex items-center border-2 border-black px-2 py-0.5 font-display text-xs uppercase tracking-wider shadow-[2px_2px_0_0_#000]",
        color === "neon" ? "bg-[#FF5500] text-black" : "bg-[#FBF9F5] text-black",
        className
      )}
      style={{ transform: `rotate(${rotate}deg)` }}
    >
      {children}
    </span>
  );
}
