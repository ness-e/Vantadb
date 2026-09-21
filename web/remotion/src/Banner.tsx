import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";
import { loadFont } from "@remotion/google-fonts/Anton";
import { VantaMark } from "./Mark";

const { fontFamily } = loadFont();

/**
 * Banner — the README presentation image.
 * "VANTADB" wordmark on the left (Anton display, "DB" outlined neon like the
 * hero), the MARK mascot on the right blinking/moving its eyes — exactly like
 * the hero animation. Transparent background.
 *
 * Render via: npm run render:banner → out/banner.gif
 */
export const Banner: React.FC = () => {
  const frame = useCurrentFrame();
  const dur = 120; // 4s loop at 30fps

  // wordmark char-by-char reveal (drop in)
  const words = ["V", "A", "N", "T", "A", "DB"];
  // ONLY "DB" (index 5) gets the neon-outline treatment — VANTA stays white
  const isNeon = (i: number) => i === 5;

  // subtle whole-word float
  const floatY = -Math.abs(Math.sin((frame / dur) * Math.PI * 2)) * 6;

  // ── ENTRANCE: letters rise from below with a spring-back bounce ──
  // letter i starts its drop at frame i*4; each drops ~90px up with a slight
  // overshoot and fades in. `letter(i)` returns per-letter style.
  const letter = (i: number) => {
    const start = 2 + i * 4;
    const prog = interpolate(frame, [start, start + 12], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: Easing.out(Easing.cubic),
    });
    const translateY = interpolate(frame, [start, start + 6, start + 12], [90, -10, 0], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    });
    return { opacity: prog, translateY };
  };

  // ── ENTRANCE: mark pops/scales in from the right ──
  const markIn = interpolate(frame, [6, 30], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const markOut = Easing.out(Easing.back(1.7))(markIn); // springy overshoot
  const markX = interpolate(frame, [6, 30], [80, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const markScale = 0.6 + markOut * 0.4;

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
      }}
    >
      {/* VANTADB wordmark */}
      <div style={{ display: "flex", transform: `translateY(${floatY}px)`, lineHeight: 1 }}>
        {words.map((c, i) => {
          const { opacity, translateY } = letter(i);
          return (
            <span
              key={i}
              style={{
                fontSize: 128,
                lineHeight: 1,
                letterSpacing: 14,
                textTransform: "uppercase",
                color: isNeon(i) ? "transparent" : "#FFFFFF",
                WebkitTextStroke: isNeon(i) ? "2px #FF5500" : undefined,
                transform: `translateY(${translateY}px)`,
                opacity,
                display: "inline-block",
                paddingRight: 6,
              }}
            >
              {c}
            </span>
          );
        })}
      </div>

      {/* MARK mascot on the right */}
      <div
        style={{
          transform: `translate(${markX}px, 0) scale(${markScale})`,
          opacity: markIn,
          width: 220,
          height: 220,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <VantaMark scale={1.1} />
      </div>
    </div>
  );
};