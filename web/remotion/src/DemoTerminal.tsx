import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont as loadMono } from "@remotion/google-fonts/SpaceMono";

const { fontFamily: mono } = loadMono();

/**
 * DemoTerminal — Tactical Telemetry (DARK) para la sección Quickstart.
 *
 * Arquetipo ÚNICO (industrial-brutalist-ui: Tactical Telemetry & CRT):
 * near-black substrate, phosphor #EAEAEA, neon #FF5500 único accent,
 * marco suizo, scanlines fijas, ASCII puro, tabular-nums en métricas.
 *
 * Reglas remotion-best-practices: typewriter por string slicing (nunca
 * opacidad por carácter); cursor blink = toggle GIF-safe; sin CSS
 * transitions.
 *
 * Render: npm run render:demo && npm run gif:demo
 */
export const DemoTerminal: React.FC = () => {
  const frame = useCurrentFrame();
  const dur = 120;

  const BLACK = "#0A0A0A";
  const PHOSPHOR = "#EAEAEA";
  const NEON = "#FF5500";
  const DIM = "#6E6E68";
  const FRAME = "#2A2A28";

  const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

  // ── typewriter helper: chars visibles por frame ──
  const type = (start: number, text: string, cps = 2.2) => {
    const n = Math.max(0, Math.floor((frame - start) * cps));
    return text.slice(0, Math.min(n, text.length));
  };
  const typed = (start: number, text: string, cps = 2.2) =>
    frame >= start && frame < start + text.length / cps;

  // ── líneas del terminal ──
  const L1 = { start: 8, cmd: 'pip install vantadb', out: "✓ installed in 1.2ms", outAt: 26 };
  const L2 = { start: 38, cmd: 'vanta.put("agents/alice", "loves vector search")', out: "✓ stored · key=agents/alice", outAt: 56 };
  const L3 = { start: 68, cmd: 'vanta.search("vector memory")', out: "3 hits · BM25+HNSW · 0.9ms", outAt: 88 };

  // ── marco + header fade ──
  const shellIn = interpolate(frame, [0, 8], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASE_OUT,
  });

  // ── cursor: bloque en línea activa, toggle 15f (GIF-safe) ──
  const cursorOn = frame % 30 < 15;
  const activeLine = typed(L1.start, L1.cmd) || typed(L2.start, L2.cmd) || typed(L3.start, L3.cmd);

  // ── footer fade ──
  const footerIn = interpolate(frame, [96, 108], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASE_OUT,
  });

  return (
    <div
      style={{
        width: 960,
        height: 320,
        backgroundColor: BLACK,
        fontFamily: mono,
        position: "relative",
        overflow: "hidden",
        boxSizing: "border-box",
      }}
    >
      {/* ── scanlines fijas (patrón repetible, GIF-safe) ── */}
      <svg
        width={960}
        height={320}
        style={{ position: "absolute", inset: 0, opacity: 0.5, zIndex: 2, pointerEvents: "none" }}
      >
        <defs>
          <pattern id="scan" width="4" height="4" patternUnits="userSpaceOnUse">
            <rect width="4" height="1" fill="#FFFFFF" opacity="0.04" />
          </pattern>
        </defs>
        <rect width={960} height={320} fill="url(#scan)" />
      </svg>

      {/* ── marco suizo ── */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          border: `3px solid ${FRAME}`,
          zIndex: 10,
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: 10,
          border: `1px solid ${FRAME}`,
          zIndex: 10,
          pointerEvents: "none",
        }}
      />

      {/* ── header bar ── */}
      <div
        style={{
          position: "absolute",
          top: 22,
          left: 34,
          right: 34,
          display: "flex",
          alignItems: "center",
          gap: 10,
          opacity: shellIn,
          zIndex: 5,
        }}
      >
        <div style={{ width: 8, height: 8, backgroundColor: NEON }} />
        <span
          style={{
            fontSize: 13,
            letterSpacing: 3,
            color: PHOSPHOR,
            textTransform: "uppercase",
          }}
        >
          vantadb — hybrid memory engine
        </span>
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: 11, letterSpacing: 2, color: DIM }}>
          v0.9.0 · local-first
        </span>
      </div>

      {/* ── líneas del terminal ── */}
      <div
        style={{
          position: "absolute",
          top: 78,
          left: 42,
          right: 42,
          fontSize: 21,
          lineHeight: 1.9,
          color: PHOSPHOR,
          zIndex: 5,
          fontVariantNumeric: "tabular-nums",
        }}
      >
        {/* L1: pip install */}
        <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
          <span style={{ color: NEON }}>$</span>
          <span>{type(L1.start, L1.cmd)}</span>
          {activeLine && typed(L1.start, L1.cmd) && cursorOn && (
            <span style={{ color: NEON }}>▌</span>
          )}
        </div>
        <div style={{ color: NEON, minHeight: 40 }}>
          {frame >= L1.outAt ? L1.out : ""}
        </div>

        {/* L2: put */}
        <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
          <span style={{ color: NEON }}>$</span>
          <span>{type(L2.start, L2.cmd, 1.6)}</span>
          {activeLine && typed(L2.start, L2.cmd) && cursorOn && (
            <span style={{ color: NEON }}>▌</span>
          )}
        </div>
        <div style={{ color: DIM, minHeight: 40 }}>
          {frame >= L2.outAt ? L2.out : ""}
        </div>

        {/* L3: search */}
        <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
          <span style={{ color: NEON }}>$</span>
          <span>{type(L3.start, L3.cmd)}</span>
          {activeLine && typed(L3.start, L3.cmd) && cursorOn && (
            <span style={{ color: NEON }}>▌</span>
          )}
        </div>
        <div style={{ color: NEON, minHeight: 40 }}>
          {frame >= L3.outAt ? L3.out : ""}
        </div>
      </div>

      {/* ── footer ── */}
      <div
        style={{
          position: "absolute",
          bottom: 22,
          left: 42,
          right: 42,
          display: "flex",
          justifyContent: "space-between",
          fontSize: 12,
          letterSpacing: 2,
          color: DIM,
          opacity: footerIn,
          zIndex: 5,
        }}
      >
        <span>LOCAL-FIRST · PERSISTENT</span>
        <span style={{ color: NEON }}>BM25 + HNSW + RRF</span>
      </div>

      {/* ── neon corner accents ── */}
      <div style={{ position: "absolute", left: 18, bottom: 18, width: 8, height: 8, backgroundColor: NEON, zIndex: 9 }} />
      <div style={{ position: "absolute", right: 18, top: 18, width: 8, height: 8, backgroundColor: NEON, zIndex: 9 }} />
    </div>
  );
};