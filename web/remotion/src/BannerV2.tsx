import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont } from "@remotion/google-fonts/Anton";
import { VantaMark } from "./Mark";

const { fontFamily } = loadFont();

/**
 * BannerV2 — versión mejorada del Banner original (assets/banner.gif).
 * El original (Banner.tsx) NO se toca; esta copia es el campo de mejora.
 *
 * Mejoras vs original:
 * 1. KERNING: "DB" ya no es una span de 2 chars — cada letra es su propia
 *    span con tracking idéntico; el wordmark respira parejo.
 * 2. GLOW NEÓN REAL en "DB": textShadow multicapa + pulse de intensidad,
 *    no solo WebkitTextStroke.
 * 3. CAÍDA CON SPRING: Easing.back en la entrada + micro-squash al aterrizar
 *    (escalaY 1→0.94→1) — las letras "aterrizan" con peso.
 * 4. SHINE SWEEP: barrido diagonal sutil sobre el wordmark durante el idle.
 * 5. MARK con pop spring (0.5→1.06→1) + squash de entrada.
 * 6. Halo ambiental naranja tenue detrás del wordmark para que el blanco
 *    respire en README dark sin manchar el transparente.
 *
 * Render: npm run render:banner-v2 (secuencia PNG @2x) + npm run gif:banner-v2 (ffmpeg two-pass → out/banner-v2.gif)
 */
export const BannerV2: React.FC = () => {
  const frame = useCurrentFrame();
  const dur = 120; // 4s loop at 30fps

  // cada letra separada → tracking consistente (V A N T A D B)
  const words = ["V", "A", "N", "T", "A", "D", "B"];
  const isNeon = (i: number) => i >= 5;

  // ── BACKGROUND: dot grid pattern (identidad "vector database") ──
  // Grid de puntos repetible (estilo ONS/Securitas Dot Space): blanco
  // tenue en trama base + naranja en subtrama escalonada. El GIF no
  // soporta alpha parcial → la vida va por radio/toggle, no opacidad.
  const GRID = 36; // spacing entre puntos
  const DOT_R = 1.6; // radio base punto blanco
  const DOT_R_NEON = 2.0; // radio punto naranja

  // ondulación: cada fila respira en radio con fase desplazada
  const rowBreathe = (row: number) =>
    0.7 + 0.3 * (0.5 + 0.5 * Math.sin((frame / 34) * Math.PI * 2 + row * 0.55));

  // puntos "especiales" que se encienden/apagan en ciclo (toggle, GIF-safe)
  const specialDots = [
    { x: 6, y: 2 }, // fila 6, col 2
    { x: 14, y: 6 },
    { x: 3, y: 7 },
    { x: 11, y: 1 },
    { x: 22, y: 4 },
  ];
  const specialOn = (i: number) => {
    const cyclePos = (frame + i * 26) % 40; // 40f por ciclo → ~1.3s
    return cyclePos >= 0 && cyclePos < 12; // 12f encendido, 28f apagado
  };

  // radios de los puntos de la trama (blanco) por columna: 25/50/75/100%,
  // patrón escalonado como ONS (variación visual sin ruido)
  const colScale = (col: number) =>
    [1, 0.5, 0.75, 1, 0.5, 0.25, 1, 0.75, 0.5, 1, 0.25, 0.75][col % 12];

  // ── wordmark float sutil ──
  const floatY = -Math.abs(Math.sin((frame / dur) * Math.PI * 2)) * 5;
  const floatR = Math.sin((frame / dur) * Math.PI * 2) * 0.6; // ±0.6° balanceo

  // ── glow neón pulse en DB (idle) ──
  const neonPulse =
    0.55 + 0.45 * Math.sin((frame / dur) * Math.PI * 2 + Math.PI);

  // ── ENTRANCE: letras caen con spring + micro-squash al aterrizar ──
  const letter = (i: number) => {
    const start = 2 + i * 4;
    const prog = interpolate(frame, [start, start + 14], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: Easing.out(Easing.cubic),
    });
    // caída con overshoot (sube 90px, rebota a -12, asienta en 0)
    const translateY = interpolate(
      frame,
      [start, start + 8, start + 12, start + 14],
      [90, -12, 4, 0],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
    );
    // micro-squash vertical al aterrizar (solo en la ventana de caída)
    const squash = interpolate(
      frame,
      [start + 10, start + 13, start + 17],
      [1, 0.94, 1],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
    );
    return { opacity: prog, translateY, squash };
  };

  // ── ENTRANCE: mark pop con spring más juguetón ──
  const markIn = interpolate(frame, [6, 26], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const markPop = Easing.out(Easing.back(2.4))(markIn); // overshoot marcado
  const markX = interpolate(frame, [6, 26], [90, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const markScale = 0.5 + markPop * 0.5;
  // squash horizontal al entrar (0.92 en el pico del overshoot)
  const markSquashX =
    markIn > 0.4 && markIn < 0.9
      ? 1 - (1 - Math.sin(((markIn - 0.4) / 0.5) * Math.PI)) * 0.08
      : 1;

  // ── SHINE SWEEP: barrido diagonal en idle (frames 45+, 2 pasadas) ──
  const shineT = (frame - 45) % 60; // 0-59 → 2s por pasada
  const shineX = interpolate(shineT, [0, 60], [-0.35, 1.35], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const shineOp =
    frame >= 45
      ? Math.sin((shineT / 60) * Math.PI) * 0.14
      : 0;

  return (
    <div
      style={{
        width: 960,
        height: 320,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0 64px",
        backgroundColor: "transparent",
        fontFamily,
        position: "relative",
        overflow: "hidden",
      }}
    >
      {/* halo ambiental tenue detrás del wordmark (naranja, ~6%) */}
      <div
        style={{
          position: "absolute",
          left: 20,
          top: 60,
          width: 560,
          height: 200,
          background:
            "radial-gradient(ellipse at 40% 50%, rgba(255,85,0,0.08), transparent 70%)",
        }}
      />

      {/* dot grid background (identidad "vector database") — grid de
          puntos repetible; la vida va por radio/toggle (el GIF no tiene
          alpha parcial) */}
      <svg
        width={960}
        height={320}
        style={{ position: "absolute", inset: 0, zIndex: 1 }}
      >
        {/* trama blanca: fila × columna, radio base × escala ONS × onda */}
        {Array.from({ length: 9 }, (_, row) =>
          Array.from({ length: 27 }, (_, col) => {
            const x = col * GRID + GRID / 2;
            const y = row * GRID + GRID / 2;
            return (
              <circle
                key={`${row}-${col}`}
                cx={x}
                cy={y}
                r={DOT_R * colScale(col) * rowBreathe(row)}
                fill="rgba(255,255,255,0.9)"
              />
            );
          })
        )}
        {/* subtrama naranja: columna impar + fila par (escalonado) */}
        {Array.from({ length: 9 }, (_, row) =>
          Array.from({ length: 27 }, (_, col) => {
            if (col % 2 === 0 || row % 2 === 0) return null;
            const x = col * GRID + GRID / 2;
            const y = row * GRID + GRID / 2;
            return (
              <circle
                key={`n${row}-${col}`}
                cx={x}
                cy={y}
                r={DOT_R_NEON * rowBreathe(row)}
                fill="rgba(255,85,0,0.9)"
              />
            );
          })
        )}
        {/* puntos especiales que se encienden/apagan en ciclo */}
        {specialDots.map((s, i) =>
          specialOn(i) ? (
            <circle
              key={`s${i}`}
              cx={s.x * GRID + GRID / 2}
              cy={s.y * GRID + GRID / 2}
              r={DOT_R_NEON * 1.3}
              fill="rgba(255,85,0,1)"
            />
          ) : null
        )}
      </svg>

      {/* VANTADB wordmark */}
      <div
        style={{
          display: "flex",
          transform: `translateY(${floatY}px) rotate(${floatR}deg)`,
          lineHeight: 1,
          position: "relative",
          zIndex: 2,
          willChange: "transform", // evita shimmer de subpixel en float
          textRendering: "geometricPrecision",
          WebkitFontSmoothing: "antialiased",
        }}
      >
        {words.map((c, i) => {
          const { opacity, translateY, squash } = letter(i);
          return (
            <span
              key={i}
              style={{
                fontSize: 128,
                lineHeight: 1,
                letterSpacing: 14,
                textTransform: "uppercase",
                // VANTA: gradiente sutil blanco→gris; DB: relleno naranja
                // SÓLIDO siempre (el fill transparente+stroke se ve hueco en GIF)
                color: "transparent",
                background: isNeon(i)
                  ? undefined
                  : "linear-gradient(180deg, #FFFFFF 55%, #C9C9C9 100%)",
                WebkitBackgroundClip: isNeon(i) ? undefined : "text",
                WebkitTextFillColor: isNeon(i) ? "#FF5500" : undefined,
                WebkitTextStroke: isNeon(i) ? "1.5px #FF7A33" : undefined,
                // glow neón multicapa + pulse (DB)
                textShadow: isNeon(i)
                  ? `0 0 14px rgba(255,85,0,${0.5 * neonPulse}), 0 0 42px rgba(255,85,0,${0.3 * neonPulse})`
                  : "0 1px 2px rgba(0,0,0,0.25)",
                transform: `translateY(${translateY}px) scaleY(${squash})`,
                transformOrigin: "50% 100%",
                opacity,
                display: "inline-block",
                paddingRight: 6,
                textRendering: "geometricPrecision",
                WebkitFontSmoothing: "antialiased",
              }}
            >
              {c}
            </span>
          );
        })}
      </div>

      {/* shine sweep sobre el wordmark */}
      <div
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          width: 620,
          height: 320,
          zIndex: 3,
          pointerEvents: "none",
          background: `linear-gradient(100deg, transparent ${shineX * 100 - 12}%, rgba(255,255,255,0.9) ${shineX * 100}%, transparent ${shineX * 100 + 12}%)`,
          opacity: shineOp,
          mixBlendMode: "screen",
        }}
      />

      {/* MARK mascot on the right */}
      <div
        style={{
          transform: `translate(${markX}px, 0) scale(${markScale}, ${markScale * markSquashX})`,
          opacity: markIn,
          width: 220,
          height: 220,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          position: "relative",
          zIndex: 2,
        }}
      >
        <VantaMark scale={1.1} variant="orbit" />
      </div>
    </div>
  );
};