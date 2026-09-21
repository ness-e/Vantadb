import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont } from "@remotion/google-fonts/Anton";
import { loadFont as loadMono } from "@remotion/google-fonts/SpaceMono";
import { VantaMark } from "./Mark";

const { fontFamily: anton } = loadFont();
const { fontFamily: mono } = loadMono();

/**
 * OrgBadge — banner NUEVO y único para el README del perfil de la org Vantadb.
 * Lockup centrado: mark (VantaMark orbit) + VANTADB + tagline de org + chips.
 *
 * CHROMA-KEY (igual que BannerV3): fondo sólido (#0A0A0A en dark, #FBF9F5 en
 * light) que ffmpeg convierte en transparente (colorkey). El texto se dibuja
 * en el color del tema para que el antialias genere grises reales.
 *
 * GIF-safe: tintas planas, toggles, sin blur, todo con interpolate/Easing,
 * curva maestra Easing.bezier(0.16,1,0.3,1).
 *
 * Dos variantes por tema de GitHub (ver README con #gh-dark-mode-only /
 * #gh-light-mode-only): OrgBadge (dark) y OrgBadgeLight (light).
 *
 * Render: npm run render:org && npm run gif:org
 *         npm run render:org-light && npm run gif:org-light
 */
export const OrgBadge: React.FC<{ theme?: "dark" | "light" }> = ({
  theme = "dark",
}) => {
  const frame = useCurrentFrame();
  const light = theme === "light";

  const BG = light ? "#FBF9F5" : "#0A0A0A";
  const INK = light ? "#000000" : "#FFFFFF";
  const NEON = "#FF5500";
  const DIM = light ? "rgba(0,0,0,0.6)" : "rgba(255,255,255,0.6)";
  const HAIR = light ? "rgba(0,0,0,0.25)" : "rgba(255,255,255,0.28)";
  const PANEL = light ? "#FFFFFF" : "#1A1A1A";

  const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);
  const words = ["V", "A", "N", "T", "A", "D", "B"];
  const isNeon = (i: number) => i >= 5;

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
      [70, -9, 0],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
    );
    return { opacity: prog, translateY };
  };

  const markIn = interpolate(frame, [10, 34], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const markPop = Easing.out(Easing.back(2))(markIn);

  const metaIn = (at: number) =>
    interpolate(frame, [at, at + 10], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: EASE_OUT,
    });
  const chipPop = (at: number) =>
    Easing.out(Easing.back(1.8))(metaIn(at));

  // cursor terminal parpadeante (toggle, loop 30f — encaja en 120f)
  const cursorOn = frame < 44 || frame % 30 < 15;

  // shine sweep única 45→105 (termina antes del loop)
  const shineT = frame >= 45 && frame <= 105 ? frame - 45 : -1;
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
        backgroundColor: BG,
        fontFamily: anton,
        position: "relative",
        overflow: "hidden",
        boxSizing: "border-box",
      }}
    >
      {/* hairline interior */}
      <div
        style={{
          position: "absolute",
          inset: 8,
          border: `1px solid ${HAIR}`,
          zIndex: 10,
          pointerEvents: "none",
        }}
      />

      {/* corner badge: cuadrito de marca */}
      <div
        style={{
          position: "absolute",
          right: 20,
          bottom: 20,
          width: 12,
          height: 12,
          backgroundColor: NEON,
          border: `2px solid ${INK}`,
          zIndex: 9,
          transform: `scale(${chipPop(8)})`,
        }}
      />

      {/* lockup centrado */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: 44,
          zIndex: 5,
        }}
      >
        {/* mark */}
        <div
          style={{
            width: 200,
            height: 200,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: INK,
            transform: `scale(${0.5 + markPop * 0.5})`,
            opacity: markIn,
          }}
        >
          <VantaMark scale={1} variant="orbit" solid />
        </div>

        {/* texto */}
        <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <div style={{ display: "flex", lineHeight: 1 }}>
            {words.map((ch, i) => {
              const { opacity, translateY } = letter(i);
              return (
                <span
                  key={i}
                  style={{
                    fontSize: 118,
                    lineHeight: 1,
                    letterSpacing: 8,
                    textTransform: "uppercase",
                    color: isNeon(i) ? NEON : INK,
                    WebkitTextStroke: isNeon(i) ? `2px ${INK}` : undefined,
                    textShadow: isNeon(i) ? "4px 4px 0 #7A2A00" : "none",
                    transform: `translateY(${translateY}px)`,
                    opacity,
                    display: "inline-block",
                    paddingRight: 4,
                  }}
                >
                  {ch}
                </span>
              );
            })}
          </div>

          {/* tagline org + cursor */}
          <div
            style={{
              opacity: metaIn(30),
              fontFamily: mono,
              fontSize: 15,
              letterSpacing: 4,
              color: INK,
              textTransform: "uppercase",
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
          >
            <span>Local-first memory for AI agents</span>
            <span
              style={{
                width: 10,
                height: 18,
                backgroundColor: cursorOn ? NEON : "transparent",
              }}
            />
          </div>

          {/* chips */}
          <div style={{ display: "flex", gap: 10 }}>
            {["Embedded", "Persistent", "Hybrid"].map((t, i) => (
              <div
                key={t}
                style={{
                  fontFamily: mono,
                  fontSize: 13,
                  fontWeight: 700,
                  letterSpacing: 1,
                  textTransform: "uppercase",
                  color: i === 0 ? (light ? "#000000" : "#FFFFFF") : i === 1 ? INK : NEON,
                  backgroundColor: i === 0 ? NEON : PANEL,
                  border: `2px solid ${INK}`,
                  padding: "4px 10px",
                  transform: `scale(${chipPop(40 + i * 6)})`,
                }}
              >
                {t}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* shine sweep */}
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

      {/* caption inferior */}
      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          bottom: 18,
          textAlign: "center",
          fontFamily: mono,
          fontSize: 11,
          letterSpacing: 5,
          color: DIM,
          textTransform: "uppercase",
          zIndex: 7,
          opacity: metaIn(52),
        }}
      >
        github.com/vantadb
      </div>
    </div>
  );
};
