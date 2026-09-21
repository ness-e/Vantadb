import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont } from "@remotion/google-fonts/Anton";
import { VantaMark } from "./Mark";

const { fontFamily: anton } = loadFont();

/**
 * OrgMark — banner de la org: mark + nombre (el .md ya no trae título,
 * el GIF es el que presenta la marca).
 *
 * Light: solo VANTA pasa a negro; DB (neón + borde blanco) y mark
 * quedan iguales al dark por pedido explícito.
 *
 * Render: npm run render:org-mark && npm run gif:org-mark
 *         npm run render:org-mark-light && npm run gif:org-mark-light
 */
export const OrgMark: React.FC<{ theme?: "dark" | "light" }> = ({
  theme = "dark",
}) => {
  const frame = useCurrentFrame();
  const light = theme === "light";
  const BG = light ? "#FBF9F5" : "#0A0A0A";
  const INK = light ? "#000000" : "#FFFFFF";
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

  const markIn = interpolate(frame, [8, 32], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const markPop = Easing.out(Easing.back(2))(markIn);

  return (
    <div
      style={{
        width: 960,
        height: 240,
        backgroundColor: BG,
        fontFamily: anton,
        position: "relative",
        overflow: "hidden",
        boxSizing: "border-box",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 48,
      }}
    >
      {/* mark */}
      <div
        style={{
          width: 210,
          height: 210,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#FFFFFF", // igual al dark en ambos temas (pedido explícito)
          transform: `scale(${0.5 + markPop * 0.5})`,
          opacity: markIn,
        }}
      >
        <VantaMark scale={1.05} variant="orbit" solid />
      </div>

      {/* nombre */}
      <div style={{ display: "flex", lineHeight: 1 }}>
        {words.map((ch, i) => {
          const { opacity, translateY } = letter(i);
          return (
            <span
              key={i}
              style={{
                fontSize: 132,
                lineHeight: 1,
                letterSpacing: 10,
                textTransform: "uppercase",
                color: isNeon(i) ? "#FF5500" : INK,
                WebkitTextStroke: isNeon(i) ? `2px #FFFFFF` : undefined,
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
    </div>
  );
};
