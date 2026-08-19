import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont } from "@remotion/google-fonts/Anton";
import { loadFont as loadMono } from "@remotion/google-fonts/SpaceMono";
import { VantaMark } from "./Mark";

const { fontFamily: anton } = loadFont();
const { fontFamily: mono } = loadMono();

/**
 * BannerV3 — wordmark transparente (lee sobre dark) para README header.
 *
 * CHROMA-KEY: el fondo de la composición es #0A0A0A (no "transparent")
 * a propósito. El GIF indexado solo tiene transparencia binaria — los
 * píxeles semi-transparentes del antialiasing del texto (alpha 30-70%)
 * se ven serruchados. Renderizando sobre near-black, el antialias del
 * texto blanco genera grises reales (blanco+negro) que el GIF representa
 * bien; luego ffmpeg convierte el #0A0A0A puro en transparente con
 * `colorkey=0x0A0A0A:0.02:0` (gif:banner-v3). Sobre dark, el halo gris
 * del borde se funde con el fondo → bordes suaves de verdad.
 *
 * Reglas remotion-best-practices: sin CSS transitions — todo interpolate /
 * Easing; curva maestra Easing.bezier(0.16,1,0.3,1); stagger 30-80ms;
 * GIF-safe (colores sólidos, toggles, sombras duras — no blur).
 *
 * Eslogan (docs/web/standards/product-positioning.md:162):
 * "Embedded Rust engine for durable local memory and hybrid vector retrieval."
 *
 * Métricas comerciales (no amarradas a stats que cambien): LOCAL-FIRST,
 * PERSISTENT, HYBRID SEARCH.
 *
 * Render: npm run render:banner-v3 && npm run gif:banner-v3
 */
export const BannerV3: React.FC = () => {
  const frame = useCurrentFrame();
  const dur = 120;

  const WHITE = "#FFFFFF";
  const NEON = "#FF5500";
  const DIM = "rgba(255,255,255,0.6)";
  const HAIR = "rgba(255,255,255,0.28)";

  const words = ["V", "A", "N", "T", "A", "D", "B"];
  const isNeon = (i: number) => i >= 5;

  const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

  // ── PATRÓN FONDO: dot grid animado (identidad "vector database") ──
  // Grid de puntos que respira (radio por fila con fase) + subtrama neon
  // escalonada + puntos especiales que se encienden en ciclo (toggle,
  // GIF-safe — la vida va por radio/toggle, no opacidad parcial).
  const GRID = 56; // spacing entre puntos (menos denso → GIF más liviano)
  const DOT_R = 1.4; // radio base punto blanco
  const DOT_R_NEON = 1.9; // radio punto naranja

  // ondulación: solo leve, y el peso GIF lo llevan las diagonales + toggles.
  // El respiro de TODOS los puntos cambia todo el canvas cada frame → GIF
  // explota. Puntos estáticos + diagonales móviles + toggles puntuales =
  // animación visible con frames que difieren poco.
  const rowBreathe = () => 0.8; // radio constante (patrón estático)

  // puntos "especiales" que se encienden/apagan en ciclo (toggle, GIF-safe)
  const specialDots = [
    { x: 4, y: 1 }, { x: 11, y: 4 }, { x: 17, y: 2 },
    { x: 7, y: 5 }, { x: 14, y: 1 }, { x: 19, y: 4 },
  ];
  const specialOn = (i: number) => {
    const cyclePos = (frame + i * 24) % 36; // 36f por ciclo → ~1.2s
    return cyclePos >= 0 && cyclePos < 11; // 11f encendido, 25f apagado
  };

  // radios de la trama blanca por columna: patrón escalonado
  const colScale = (col: number) =>
    [1, 0.5, 0.75, 1, 0.5, 0.25, 1, 0.75, 0.5, 1, 0.25, 0.75][col % 12];

  // ── líneas diagonales móviles (patrón repetible que se desplaza) ──
  const diagShift = (frame % 40) * 3; // 40f por ciclo → 120px de recorrido

  // ── registration marks en esquinas ──
  const regIn = interpolate(frame, [0, 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASE_OUT,
  });

  // ── wordmark: letras caen con overshoot, stagger 3f ──
  const letter = (i: number) => {
    const start = 4 + i * 3;
    const prog = interpolate(frame, [start, start + 12], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: EASE_OUT,
    });
    const translateY = interpolate(
      frame,
      [start, start + 8, start + 12],
      [80, -10, 0],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
    );
    const squash = interpolate(
      frame,
      [start + 8, start + 11, start + 15],
      [1, 0.95, 1],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
    );
    return { opacity: prog, translateY, squash };
  };

  // ── breath sutil en idle (solo 2-3px, no distrae) ──
  const floatY = -Math.abs(Math.sin((frame / dur) * Math.PI * 2)) * 3;

  // ── neon pulse DB (solo toggle 2 niveles, GIF-safe) ──
  const neonPulse = frame >= 60 ? (frame % 30 < 15 ? 1 : 0) : 1;

  // ── mark pop-in con spring ──
  const markIn = interpolate(frame, [10, 34], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const markPop = Easing.out(Easing.back(2))(markIn);
  const markX = interpolate(frame, [10, 34], [70, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const markScale = 0.5 + markPop * 0.5;

  // ── metadata + métricas (stagger 34/40/46) ──
  const metaIn = (at: number) =>
    interpolate(frame, [at, at + 10], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: EASE_OUT,
    });
  const stickerPop = (at: number) =>
    Easing.out(Easing.back(1.8))(metaIn(at));

  // ── idle: métrica LOCAL-FIRST tickea cada 30f (toggle GIF-safe) ──
  const tick = frame >= 60 ? (frame % 30 < 6 ? 1 : 0) : 0;

  // ── shine sweep 2 pasadas desde frame 45 ──
  const shineT = frame >= 45 ? (frame - 45) % 60 : -1;
  const shineX = interpolate(shineT, [0, 60], [-0.35, 1.35], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const shineOp = shineT >= 0 ? Math.sin((shineT / 60) * Math.PI) * 0.12 : 0;

  return (
    <div
      style={{
        width: 960,
        height: 320,
        // Chroma-key #0A0A0A — ver nota del componente (colorkey en gif:banner-v3)
        backgroundColor: "#0A0A0A",
        fontFamily: anton,
        position: "relative",
        overflow: "hidden",
        boxSizing: "border-box",
      }}
    >
      {/* hairline interior — conecta los registration marks de las esquinas */}
      <div
        style={{
          position: "absolute",
          inset: 8,
          border: `1px solid ${HAIR}`,
          zIndex: 10,
          pointerEvents: "none",
        }}
      />

      {/* ── PATRÓN FONDO: dot grid animado + diagonales móviles ── */}
      <svg
        width={960}
        height={320}
        style={{ position: "absolute", inset: 0, zIndex: 1, pointerEvents: "none" }}
      >
        {/* trama blanca: fila × columna, radio base × escala ONS × onda */}
        {Array.from({ length: 6 }, (_, row) =>
          Array.from({ length: 18 }, (_, col) => {
            const x = col * GRID + GRID / 2;
            const y = row * GRID + GRID / 2;
            return (
              <circle
                key={`${row}-${col}`}
                cx={x}
                cy={y}
                r={DOT_R * colScale(col) * rowBreathe()}
                fill="rgba(255,255,255,0.55)"
              />
            );
          })
        )}
        {/* subtrama naranja: columna impar + fila par (escalonado) */}
        {Array.from({ length: 6 }, (_, row) =>
          Array.from({ length: 18 }, (_, col) => {
            if (col % 2 === 0 || row % 2 === 0) return null;
            const x = col * GRID + GRID / 2;
            const y = row * GRID + GRID / 2;
            return (
              <circle
                key={`n${row}-${col}`}
                cx={x}
                cy={y}
                r={DOT_R_NEON * rowBreathe()}
                fill="rgba(255,85,0,0.7)"
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
              r={DOT_R_NEON * 1.4}
              fill="#FF5500"
            />
          ) : null
        )}
        {/* diagonales móviles: 8 líneas que se desplazan en loop */}
        {Array.from({ length: 8 }).map((_, i) => {
          const baseX = ((i * 140 + diagShift) % 1120) - 80;
          return (
            <line
              key={`d${i}`}
              x1={baseX}
              y1={-20}
              x2={baseX - 130}
              y2={340}
              stroke="rgba(255,255,255,0.10)"
              strokeWidth={i % 3 === 0 ? 2 : 1}
            />
          );
        })}
      </svg>

      {/* ── registration marks ── */}
      {[
        { x: 14, y: 14 },
        { x: 946, y: 14 },
        { x: 14, y: 306 },
        { x: 946, y: 306 },
      ].map((p, i) => (
        <RegistrationMark key={i} {...p} opacity={regIn} />
      ))}

      {/* ── content row: wordmark | mark ── */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          padding: "0 56px",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          zIndex: 5,
        }}
      >
        {/* left: wordmark + metadata */}
        <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
          <div
            style={{
              display: "flex",
              transform: `translateY(${floatY}px)`,
              lineHeight: 1,
            }}
          >
            {words.map((c, i) => {
              const { opacity, translateY, squash } = letter(i);
              return (
                <span
                  key={i}
                  style={{
                    fontSize: 126,
                    lineHeight: 1,
                    letterSpacing: 10,
                    textTransform: "uppercase",
                    // VANTA: blanco sólido; DB: neon + borde blanco
                    color: isNeon(i) ? NEON : WHITE,
                    WebkitTextStroke: isNeon(i) ? "2px #FFFFFF" : undefined,
                    // glow sólido GIF-safe (blur muere en paleta GIF) —
                    // sombra dura offset, sin niveles rgba intermedios
                    textShadow: isNeon(i)
                      ? neonPulse
                        ? "4px 4px 0 #7A2A00"
                        : "3px 3px 0 #7A2A00"
                      : "none",
                    transform: `translateY(${translateY}px) scaleY(${squash})`,
                    transformOrigin: "50% 100%",
                    opacity,
                    display: "inline-block",
                    paddingRight: 4,
                  }}
                >
                  {c}
                </span>
              );
            })}
          </div>

          {/* eslogan oficial (product-positioning.md:162) */}
          <div style={{ opacity: metaIn(34), fontFamily: mono }}>
            <div
              style={{
                fontSize: 14,
                letterSpacing: 3,
                color: WHITE,
                textTransform: "uppercase",
                lineHeight: 1.7,
              }}
            >
              Embedded Rust engine for durable
              <br />
              local memory and hybrid vector retrieval
            </div>
          </div>

          {/* métricas comerciales (LOCAL-FIRST / PERSISTENT / HYBRID SEARCH) */}
          <div style={{ display: "flex", gap: 12, opacity: metaIn(44) }}>
            <div
              style={{
                fontFamily: mono,
                fontSize: 14,
                fontWeight: 700,
                letterSpacing: 1,
                color: "#FFFFFF",
                backgroundColor: NEON,
                border: `2px solid ${WHITE}`,
                padding: "5px 12px",
                transform: `scale(${stickerPop(44)})`,
                fontVariantNumeric: "tabular-nums",
              }}
            >
              LOCAL-FIRST{tick === 1 ? " ✓" : ""}
            </div>
            <div
              style={{
                fontFamily: mono,
                fontSize: 14,
                fontWeight: 700,
                letterSpacing: 1,
                color: WHITE,
                backgroundColor: "#1A1A1A", // sólido — alpha parcial muere en GIF
                border: `2px solid ${WHITE}`,
                padding: "5px 12px",
                transform: `scale(${stickerPop(50)})`,
              }}
            >
              PERSISTENT
            </div>
            <div
              style={{
                fontFamily: mono,
                fontSize: 14,
                fontWeight: 700,
                letterSpacing: 1,
                color: NEON,
                border: `2px solid ${WHITE}`,
                padding: "5px 12px",
                transform: `scale(${stickerPop(56)})`,
              }}
            >
              HYBRID SEARCH
            </div>
          </div>
        </div>

        {/* right: mark */}
        <div
          style={{
            transform: `translate(${markX}px, 0) scale(${markScale})`,
            opacity: markIn,
            width: 230,
            height: 230,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <VantaMark scale={1.15} variant="orbit" solid />
        </div>
      </div>

      {/* ── shine sweep ── */}
      <div
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          width: 640,
          height: 320,
          zIndex: 6,
          pointerEvents: "none",
          background: `linear-gradient(100deg, transparent ${shineX * 100 - 12}%, rgba(255,255,255,0.9) ${shineX * 100}%, transparent ${shineX * 100 + 12}%)`,
          opacity: shineOp,
          mixBlendMode: "screen",
        }}
      />

      {/* corner accent: neon square bottom-right */}
      <div
        style={{
          position: "absolute",
          right: 20,
          bottom: 20,
          width: 10,
          height: 10,
          backgroundColor: NEON,
          zIndex: 9,
        }}
      />
    </div>
  );
};

/* Registration mark: crosshair blanco en esquina (vista sobre frame) */
function RegistrationMark({
  x,
  y,
  opacity,
}: {
  x: number;
  y: number;
  opacity: number;
}) {
  return (
    <div
      style={{
        position: "absolute",
        left: x - 8,
        top: y - 8,
        width: 16,
        height: 16,
        opacity,
        zIndex: 11,
        pointerEvents: "none",
      }}
    >
      <svg width={16} height={16} viewBox="0 0 16 16">
        <circle cx="8" cy="8" r="5" fill="none" stroke="#FFFFFF" strokeWidth="1" />
        <path d="M8 0 V16 M0 8 H16" stroke="#FFFFFF" strokeWidth="1" />
      </svg>
    </div>
  );
}