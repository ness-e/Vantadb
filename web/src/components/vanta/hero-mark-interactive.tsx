"use client";

import { useRef, useState, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";

/**
 * HeroMarkInteractive — native, box-less interactive mark for Hero v2.
 *
 * Spec:
 *  1. Outer ring: black border, NO fill — sits directly on hero bg
 *  2. Orange sphere: follows mouse with SMOOTH fluid animation, clamped inside ring
 *  3. Two vertical bar eyes: rounded corners, thicker, shorter — follow mouse across hero
 *  4. NO box — integrated natively into hero
 *  5. Background graph nodes are INTERACTIVE (hover = mark looks at them)
 *
 * Smoothness: uses requestAnimationFrame throttle + CSS cubic-bezier transitions.
 */

type BlinkState = "open" | "left-closed" | "right-closed" | "both-closed";

// Graph nodes scattered across hero background (interactive)
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

export function HeroMarkInteractive() {
  const heroRef = useRef<HTMLDivElement>(null);
  const markWrapRef = useRef<HTMLDivElement>(null);
  const [pupilOffset, setPupilOffset] = useState({ x: 0, y: 0 });
  const [sphereOffset, setSphereOffset] = useState({ x: 0, y: 0 });
  const [blink, setBlink] = useState<BlinkState>("open");
  const [annoyed, setAnnoyed] = useState(false);
  const [hoveredNode, setHoveredNode] = useState<number | null>(null);
  const [mouseInHero, setMouseInHero] = useState(false);

  // rAF-throttled mouse target — avoids re-rendering on every mousemove
  const targetOffset = useRef({ eyeX: 0, eyeY: 0, sphereX: 0, sphereY: 0 });
  const rafId = useRef<number | null>(null);
  const lastRender = useRef(0);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    const wrap = markWrapRef.current;
    if (!wrap) return;
    const rect = wrap.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    const dx = e.clientX - cx;
    const dy = e.clientY - cy;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const angle = Math.atan2(dy, dx);

    // Compute target offsets (not applied directly — rAF will smooth them)
    const maxEyeOffset = 16;
    const eyeNorm = Math.min(dist / 280, 1);
    const maxSphereOffset = 7;
    const sphereNorm = Math.min(dist / 380, 1);

    targetOffset.current = {
      eyeX: Math.cos(angle) * eyeNorm * maxEyeOffset,
      eyeY: Math.sin(angle) * eyeNorm * maxEyeOffset,
      sphereX: Math.cos(angle) * sphereNorm * maxSphereOffset,
      sphereY: Math.sin(angle) * sphereNorm * maxSphereOffset,
    };

    setMouseInHero(true);

    // rAF throttle — at most one render per frame
    if (rafId.current === null) {
      rafId.current = requestAnimationFrame((ts) => {
        rafId.current = null;
        // Throttle to ~60fps; only update state if 16ms passed
        if (ts - lastRender.current >= 16) {
          lastRender.current = ts;
          setPupilOffset({ x: targetOffset.current.eyeX, y: targetOffset.current.eyeY });
          setSphereOffset({ x: targetOffset.current.sphereX, y: targetOffset.current.sphereY });
        }
      });
    }
  }, []);

  useEffect(() => {
    window.addEventListener("mousemove", handleMouseMove, { passive: true });
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      if (rafId.current !== null) cancelAnimationFrame(rafId.current);
    };
  }, [handleMouseMove]);

  // Click handler — blink, alternating eyes
  const cycle = useRef(0);
  const handleClick = useCallback(() => {
    const states: BlinkState[] = ["left-closed", "right-closed", "both-closed"];
    const next = states[cycle.current % states.length];
    cycle.current += 1;
    setBlink(next);
    setAnnoyed(true);
    setTimeout(() => setBlink("open"), 220);
    setTimeout(() => setAnnoyed(false), 900);
  }, []);

  // Graph node hover — mark looks at the node
  const handleNodeHover = useCallback((idx: number | null) => {
    setHoveredNode(idx);
    if (idx !== null) {
      const node = GRAPH_NODES[idx];
      const dx = node.x - 50;
      const dy = node.y - 50;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const maxEyeOffset = 16;
      targetOffset.current = {
        ...targetOffset.current,
        eyeX: (dx / dist) * maxEyeOffset,
        eyeY: (dy / dist) * maxEyeOffset,
      };
      setPupilOffset({ x: (dx / dist) * maxEyeOffset, y: (dy / dist) * maxEyeOffset });
    }
  }, []);

  return (
    <div
      ref={heroRef}
      className="relative mx-auto aspect-square w-full max-w-[460px] cursor-pointer lg:max-w-none"
      onClick={handleClick}
    >
      {/* Background graph — SINGLE SVG with edges + INTERACTIVE nodes (no pointer-events-none) */}
      <svg
        className="absolute inset-0 h-full w-full"
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
      >
        {/* Edges (non-interactive, but visible) */}
        {GRAPH_EDGES.map(([a, b], i) => (
          <line
            key={`edge-${i}`}
            x1={GRAPH_NODES[a].x}
            y1={GRAPH_NODES[a].y}
            x2={GRAPH_NODES[b].x}
            y2={GRAPH_NODES[b].y}
            stroke={hoveredNode === a || hoveredNode === b ? "#FF5500" : "currentColor"}
            strokeWidth="0.25"
            strokeDasharray="1 1.2"
            className="text-black/25 transition-all duration-300 "
            opacity={hoveredNode === a || hoveredNode === b ? 0.9 : 0.5}
            pointerEvents="none"
          />
        ))}
        {/* Nodes — INTERACTIVE: visible + hoverable in one element */}
        {GRAPH_NODES.map((node, i) => (
          <g key={`node-${i}`}>
            {/* Visible node circle */}
            <circle
              cx={node.x}
              cy={node.y}
              r={hoveredNode === i ? node.r * 2.2 : node.r}
              fill={hoveredNode === i ? "#FF5500" : "currentColor"}
              className="text-black/35 transition-all duration-300  cursor-pointer"
            />
            {/* Invisible larger hit area for easier hover */}
            <circle
              cx={node.x}
              cy={node.y}
              r="5"
              fill="transparent"
              className="cursor-pointer"
              onMouseEnter={() => handleNodeHover(i)}
              onMouseLeave={() => handleNodeHover(null)}
              onClick={(e) => {
                e.stopPropagation();
                handleClick();
              }}
            />
          </g>
        ))}
      </svg>

      {/* The mark — outer ring (border only, no fill), native on hero background */}
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
          {/* Outer ring — black border, NO fill (transparent, sits on hero bg) */}
          <circle
            cx="50"
            cy="50"
            r="42"
            fill="none"
            stroke="currentColor"
            strokeWidth="3.5"
            className="text-black "
          />

          {/* Subtle outer glow ring (pulses softly) */}
          <circle cx="50" cy="50" r="42" fill="none" stroke="#FF5500" strokeWidth="0.6" opacity="0.3">
            <animate attributeName="r" values="42;46;42" dur="3.5s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.3;0;0.3" dur="3.5s" repeatCount="indefinite" />
          </circle>

          {/* Orange sphere — SMOOTH fluid follow via cubic-bezier transition */}
          <circle
            cx={50 + sphereOffset.x}
            cy={50 + sphereOffset.y}
            r="22"
            fill="#FF5500"
            style={{
              transform: annoyed ? "scale(0.94)" : "scale(1)",
              transformOrigin: `${50 + sphereOffset.x}px ${50 + sphereOffset.y}px`,
              transition: "cx 0.6s cubic-bezier(0.22, 1, 0.36, 1), cy 0.6s cubic-bezier(0.22, 1, 0.36, 1), transform 0.3s ease-out",
            }}
          />

          {/* EYES — two vertical bars, ROUNDED corners, THICKER, SHORTER.
              Track mouse across entire hero, gather toward ring edge.
              Smooth fluid follow via cubic-bezier transition on x/y. */}
          <rect
            x={43 + pupilOffset.x - 2}
            y={blink === "left-closed" || blink === "both-closed" ? 50 : 45}
            width="4"
            height={blink === "left-closed" || blink === "both-closed" ? 1.5 : 10}
            fill="#000"
            rx="2"
            style={{
              transition: "x 0.25s cubic-bezier(0.22, 1, 0.36, 1), y 0.15s ease-out, height 0.15s ease-out",
            }}
          />
          <rect
            x={57 + pupilOffset.x - 2}
            y={blink === "right-closed" || blink === "both-closed" ? 50 : 45}
            width="4"
            height={blink === "right-closed" || blink === "both-closed" ? 1.5 : 10}
            fill="#000"
            rx="2"
            style={{
              transition: "x 0.25s cubic-bezier(0.22, 1, 0.36, 1), y 0.15s ease-out, height 0.15s ease-out",
            }}
          />
        </svg>
      </div>

      {/* Floating annotation labels (manga SFX) — native, no box frame */}
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

      {/* Interactive hint — appears when mouse is in hero */}
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
