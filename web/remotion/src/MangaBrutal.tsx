import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont } from "@remotion/google-fonts/Anton";
import { loadFont as loadMono } from "@remotion/google-fonts/SpaceMono";
import { VantaMark } from "./Mark";

const { fontFamily: anton } = loadFont();
const { fontFamily: mono } = loadMono();

/**
 * MangaBrutal — transparent-background manga variant for the README banner.
 *   - VANTA (white) + DB (neon outline + glow) inline on one line (Anton)
 *   - MARK on the right, popping in
 *   - manga kinetics: radial speed-lines burst + ink splatter flying off the mark
 *   - SFX sticker labels (1.2ms, RRF, WAL·CRC32C, ZERO NET)
 *   - halftone dots as a manga print accent
 * White text so it reads on GitHub's dark background; DB keeps the neon accent.
 */

const INK = "#000000";
const NEON = "#FF5500";
const SMOKE = "#1A1A1A";
const GOLD = "#FFFFFF";

type Sticker = {
  text: string;
  x: number;
  y: number;
  rot: number;
  bg: string;
  appear: number;
};

const STICKERS: Sticker[] = [
  { text: "1.2ms", x: 20, y: 8, rot: -6, bg: NEON, appear: 44 },
  { text: "RRF", x: 168, y: 20, rot: 5, bg: "ink", appear: 54 },
  { text: "WAL", x: 14, y: 196, rot: -3, bg: "ink", appear: 64 },
  { text: "ZERO NET", x: 150, y: 196, rot: 4, bg: NEON, appear: 74 },
];

const SPLATTER = [
  { x: -52, y: -46, r: 5, from: 36, dur: 14 },
  { x: -72, y: 20, r: 3, from: 40, dur: 12 },
  { x: -34, y: 58, r: 2.5, from: 44, dur: 10 },
  { x: 60, y: -54, r: 4, from: 38, dur: 13 },
  { x: 82, y: -22, r: 2, from: 47, dur: 9 },
  { x: 70, y: 54, r: 3.5, from: 42, dur: 12 },
];

export const MangaBrutal: React.FC = () => {
  const frame = useCurrentFrame();

  // ── speed burst: radial lines scrape in from behind the mark ──
  const burst = interpolate(frame, [2, 18], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });

  // ── wordmark: whole line drops with overshoot ──
  const lineIn = interpolate(frame, [10, 30], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const lineY = interpolate(frame, [10, 15, 30], [110, -12, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // ── MARK: elastic pop from right, after the line ──
  const markIn = interpolate(frame, [20, 46], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const markPop = Easing.out(Easing.back(1.9))(markIn);
  const markX = interpolate(frame, [20, 46], [80, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const markScale = 0.5 + markPop * 0.5;

  // ── SFX stickers: staggered pop ──
  const stickerP = (s: Sticker) =>
    Easing.out(Easing.back(2))(interpolate(frame, [s.appear, s.appear + 8], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }));

  // ── splatter: ink drops burst off the mark ──
  const drop = (d: (typeof SPLATTER)[number]) => {
    const p = interpolate(frame, [d.from, d.from + d.dur], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: Easing.out(Easing.cubic),
    });
    return { o: p, r: d.r * p, tx: d.x * (1 - p) * 0.3, ty: d.y * (1 - p) * 0.3 };
  };

  // ── tagline fade ──
  const tagIn = interpolate(frame, [34, 50], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const MARK_CX = 760; // mark center anchor x
  const MARK_CY = 160; // 160

  return (
    <svg
      width={960}
      height={320}
      viewBox="0 0 960 320"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        {/* halftone dot pattern — used as a sparse manga print accent behind text */}
        <pattern id="halftone" width="14" height="14" patternUnits="userSpaceOnUse">
          <circle cx="1.4" cy="1.4" r="1.4" fill="#FFFFFF" opacity="0.35" />
        </pattern>
      </defs>

      {/* ── SPEED LINES: radial burst scrapping in from the mark ── */}
      <g transform={`translate(${MARK_CX}, ${MARK_CY})`} opacity={burst}>
        {Array.from({ length: 40 }).map((_, i) => {
          const a = (i / 40) * Math.PI * 2 + 0.3;
          const r1 = 150;
          const r2 = 330 + (i % 3) * 46;
          return (
            <line
              key={i}
              x1={Math.cos(a) * r1}
              y1={Math.sin(a) * r1}
              x2={Math.cos(a) * r2}
              y2={Math.sin(a) * r2}
              stroke="#FFFFFF"
              strokeOpacity={i % 2 === 0 ? 0.42 : 0.2}
              strokeWidth={1.8}
            />
          );
        })}
      </g>

      {/* faint halftone swatch behind the wordmark (bottom-left accent) */}
      <g transform={`translate(60, 200)`} opacity={tagIn}>
        <rect width={130} height={90} fill="url(#halftone)" />
      </g>

      {/* ── MARK ── */}
      <g opacity={markIn} transform={`translate(${668 + markX}, ${38}) scale(${markScale})`}>
        <VantaMark scale={1.12} />
      </g>

      {/* ── SPLATTER: ink drops burst off the mark ── */}
      {SPLATTER.map((d, i) => {
        const s = drop(d);
        return (
          <circle
            key={i}
            cx={MARK_CX + d.x + s.tx}
            cy={MARK_CY + d.y + s.ty}
            r={s.r}
            fill="#FFFFFF"
            opacity={s.o}
          />
        );
      })}

      {/* ── WORDMARK: VANTA + DB on ONE line ── */}
      <g
        fontFamily={anton}
        opacity={lineIn}
        transform={`translate(64, ${118 + lineY})`}
      >
        {/* "VANTA" solid white — G is not a char here */}
        <text fontSize={150} fontWeight={400} fill={GOLD} letterSpacing="2" style={{ fontFamily: anton }}>
          VANTA
        </text>
        {/* "DB" neon outline + glow, transparent fill */}
        <text
          x={272}
          fontSize={150}
          fill="none"
          stroke={NEON}
          strokeWidth={2.8}
          letterSpacing="2"
          style={{ fontFamily: anton, filter: "drop-shadow(0 0 6px rgba(255,85,0,0.55))" }}
        >
          DB
        </text>
      </g>

      {/* ── tagline: Space Mono, fades under the wordmark ── */}
      <g opacity={tagIn} fontFamily={mono}>
        <text x={66} y={196} fontSize={16} fill="#FFFFFF" fillOpacity={0.85} letterSpacing="3" style={{ fontFamily: mono }}>
          VECTOR DATABASE FOR AI AGENTS
        </text>
        <rect x={66} y={210} width={200} height={26} fill="none" stroke="#FFFFFF" strokeOpacity={0.7} strokeWidth={1.6} />
        <text x={78} y={228} fontSize={14} fill={NEON} fontFamily={mono} letterSpacing="1" style={{ fontFamily: mono }}>
          {"$ vanta init"}
        </text>
      </g>

      {/* ── SFX stickers (manga) around the mark ── */}
      {STICKERS.map((s, i) => {
        const p = stickerP(s);
        return (
          <g
            key={i}
            opacity={p}
            transform={`translate(${668 + s.x + 20 + 3}, ${38 + s.y + 20 + 3}) rotate(${s.rot}) scale(${0.5 + p * 0.5})`}
            style={{ transformOrigin: "0px 0px" }}
          >
            <rect x={-40} y={-13} width={80} height={26} fill={SMOKE} />
          </g>
        );
      })}
      {STICKERS.map((s, i) => {
        const p = stickerP(s);
        return (
          <g
            key={`top-${i}`}
            opacity={p}
            transform={`translate(${668 + s.x + 20}, ${38 + s.y + 20}) rotate(${s.rot}) scale(${0.5 + p * 0.5})`}
            style={{ transformOrigin: "0px 0px" }}
          >
            <rect x={-40} y={-13} width={80} height={26} fill={s.bg === NEON ? NEON : SMOKE} stroke={s.bg === NEON ? SMOKE : NEON} strokeWidth={1.8} />
            <text
              x={0}
              y={4}
              textAnchor="middle"
              fontSize={12}
              fontFamily={mono}
              fill={s.bg === NEON ? NEON : "#FFFFFF"}
              fontWeight={700}
              letterSpacing="1"
            >
              {s.text}
            </text>
          </g>
        );
      })}
    </svg>
  );
};