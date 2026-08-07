import React from "react";
import { useCurrentFrame, interpolate, Easing } from "remotion";

/**
 * VantaMark — Remotion port of web/src/components/vanta/mark/mark-classic.tsx.
 * Same SVG geometry/colors as the hero mark, but frame-driven instead of
 * mouse-driven so it can be rendered deterministically as a GIF.
 *
 * Animation: eyes drift left/right + blink periodically (like the hero's
 * click-blink), sphere does a subtle breathe, glow ring pulses.
 */

const EYE_OPEN = 10;
const EYE_CLOSED = 1.5;
const EYE_X_OPEN = 43;
const EYE_X2_OPEN = 57;

type Blink = "open" | "left" | "right" | "both";

export const VantaMark: React.FC<{
  scale?: number;
}> = ({ scale = 1 }) => {
  const frame = useCurrentFrame();

  // ── eye drift (look left↔right, like tracking) ──
  const cycle = frame % 120; // 4s loop at 30fps
  const drift = Math.sin((cycle / 120) * Math.PI * 2) * 5; // ±5px eye x

  // ── blink schedule ──
  // blink at frames 20, 50, 80 (every ~1s) — single eye each time, alternating
  const local = frame % 120;
  const blinkAt = [
    { at: 20, type: "left" as Blink },
    { at: 50, type: "right" as Blink },
    { at: 80, type: "both" as Blink },
  ];
  const active = blinkAt.find((b) => local >= b.at && local < b.at + 6);

  const eyeHeight = (isClosed: boolean) =>
    isClosed
      ? interpolate(local, [active!.at, active!.at + 3, active!.at + 6], [EYE_OPEN, EYE_CLOSED, EYE_OPEN], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: Easing.inOut(Easing.quad),
        })
      : EYE_OPEN;

  const leftClosed = active ? active.type === "left" || active.type === "both" : false;
  const rightClosed = active ? active.type === "right" || active.type === "both" : false;

  const leftH = leftClosed ? eyeHeight(true) : EYE_OPEN;
  const rightH = rightClosed ? eyeHeight(true) : EYE_OPEN;
  // closed eyes drop to center
  const leftY = leftClosed ? 45 + (EYE_OPEN - leftH) / 2 : 45;
  const rightY = rightClosed ? 45 + (EYE_OPEN - rightH) / 2 : 45;

  // ── sphere breathe ──
  const sphereScale =
    1 + Math.sin((frame / 30) * Math.PI) * 0.02;

  // ── glow pulse ──
  const glowR = interpolate((frame % 105) / 105, [0, 0.5, 1], [42, 46, 42], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const glowO = interpolate((frame % 105) / 105, [0, 0.5, 1], [0.3, 0, 0.3], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <svg
      width={200 * scale}
      height={200 * scale}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="VantaDB mark"
    >
      <defs>
        {/* Soft orange glow under the outer ring so it reads on dark bg */}
        <filter id="mark-ring-glow" x="-40%" y="-40%" width="180%" height="180%">
          <feGaussianBlur stdDeviation="2.4" />
        </filter>
      </defs>

      {/* Orange halo — same ring geometry, blurred, sits under the black ring */}
      <circle
        cx="50"
        cy="50"
        r="42"
        fill="none"
        stroke="#FF5500"
        strokeWidth="3.5"
        opacity="0.95"
        filter="url(#mark-ring-glow)"
      />

      {/* Outer ring — black border, NO fill (transparent) */}
      <circle cx="50" cy="50" r="42" fill="none" stroke="#000000" strokeWidth="3.5" />

      {/* Subtle outer glow ring (pulses) */}
      <circle cx="50" cy="50" r={glowR} fill="none" stroke="#FF5500" strokeWidth="0.6" opacity={glowO} />

      {/* Orange sphere — subtle breathe */}
      <circle cx="50" cy="50" r="22" fill="#FF5500" style={{ transform: `scale(${sphereScale})`, transformOrigin: "50px 50px" }} />

      {/* EYES — two vertical bars, rounded */}
      <rect
        x={EYE_X_OPEN + drift}
        y={leftY}
        width="4"
        height={leftH}
        fill="#000"
        rx="2"
      />
      <rect
        x={EYE_X2_OPEN + drift}
        y={rightY}
        width="4"
        height={rightH}
        fill="#000"
        rx="2"
      />
    </svg>
  );
};
