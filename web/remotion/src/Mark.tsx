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

export const VantaMark: React.FC<{
  scale?: number;
  variant?: "default" | "orbit";
  solid?: boolean;
}> = ({ scale = 1, variant = "default", solid = false }) => {
  const frame = useCurrentFrame();

  // ── eye behavior: DISCRETE, like real eyes ──
  // saccades: eyes jump FAST to a point, then HOLD still (no sine drift)
  const local = frame % 120;
  const SACCADES = [
    { at: 30, target: -5 }, // look left
    { at: 60, target: 5 },  // look right
    { at: 90, target: 0 },  // back to center
  ] as const;
  // held position = target of the last completed saccade
  let saccadeTarget = 0;
  for (const s of SACCADES) if (local >= s.at) saccadeTarget = s.target;
  // in-window saccade: ease-out over 5 frames from previous target
  const activeSaccade = SACCADES.find((s) => local >= s.at && local < s.at + 5);
  let lookOffset: number;
  if (activeSaccade) {
    const prev =
      activeSaccade.at === 30 ? 0 : activeSaccade.at === 60 ? -5 : 5;
    const prog = interpolate(local, [activeSaccade.at, activeSaccade.at + 5], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: Easing.out(Easing.cubic),
    });
    lookOffset = prev + (activeSaccade.target - prev) * prog;
  } else {
    lookOffset = saccadeTarget; // held still between saccades
  }

  // ── blink: BOTH eyes together, fast, every ~1s ──
  const blinkAt = [25, 55, 85];
  const blink = blinkAt.find((at) => local >= at && local < at + 5);
  const blinkH = blink
    ? interpolate(local, [blink, blink + 2, blink + 5], [EYE_OPEN, EYE_CLOSED, EYE_OPEN], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
        easing: Easing.inOut(Easing.quad),
      })
    : EYE_OPEN;

  const leftH = blink ? blinkH : EYE_OPEN;
  const rightH = blink ? blinkH : EYE_OPEN;
  // closed eyes drop to center
  const leftY = blink ? 45 + (EYE_OPEN - leftH) / 2 : 45;
  const rightY = blink ? 45 + (EYE_OPEN - rightH) / 2 : 45;

  // ── sphere: breathe lento + squash sutil al blink ──
  const breathe = 1 + Math.sin((frame / 30) * Math.PI) * 0.02;
  const blinkSquash = blink
    ? 1 - 0.05 * Math.sin(((local - blink) / 5) * Math.PI)
    : 1;
  const sphereScaleX = breathe * (1 + (1 - blinkSquash) * 0.5);
  const sphereScaleY = breathe * blinkSquash;

  // ── glow pulse ──
  const glowR = interpolate((frame % 105) / 105, [0, 0.5, 1], [42, 46, 42], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const glowO = interpolate((frame % 105) / 105, [0, 0.5, 1], [0.3, 0, 0.3], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // ── orbit ring (variant="orbit") — dash rotating + 3 nodes riding it ──
  const orbitDash = -(frame % 120) * 0.8; // dashoffset animado → guiones giran
  const orbitAngle = (frame % 120) * 3; // nodos orbitan el anillo

  return (
    <svg
      width={200 * scale}
      height={200 * scale}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="VantaDB mark"
    >
      {/* Orange edge of the dark ring — SOLID rings that peek out around the
          black ring (blur dies in GIF palette, so no feGaussianBlur here) */}

      <defs>
        {/* Orange glow under the outer ring so it reads on dark bg */}
        <filter id="mark-ring-glow" x="-40%" y="-40%" width="180%" height="180%">
          <feGaussianBlur stdDeviation="2.4" />
        </filter>
      </defs>

      {/* ── sin aro exterior: el mark es solo esfera + ojos (V3) ── */}
      {/* Orange halo — SOLO en modo no-solid (V2/Manga). En solid (V3)
          no hay anillo que envuelva: esfera naranja desnuda. */}
      {!solid && (
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
      )}

      {/* Subtle outer glow ring (pulses) — también envuelve; quitado en solid */}
      {!solid && (
        <circle cx="50" cy="50" r={glowR} fill="none" stroke="#FF5500" strokeWidth="0.6" opacity={glowO} />
      )}

      {/* Orbit variant: rotating dashed ring + nodes riding the ring */}
      {variant === "orbit" && (
        <>
          <circle
            cx="50"
            cy="50"
            r="47"
            fill="none"
            stroke="#FF5500"
            strokeWidth="1.4"
            strokeDasharray="3 9"
            strokeDashoffset={orbitDash}
            opacity="0.7"
          />
          {/* 3 nodes riding the outer dashed ring */}
          <g
            style={{
              transform: `rotate(${orbitAngle}deg)`,
              transformOrigin: "50px 50px",
            }}
          >
            {[0, 120, 240].map((off, i) => (
              <circle
                key={i}
                cx="97"
                cy="50"
                r={i === 0 ? 2.2 : 1.5}
                fill={i === 0 ? "#FF5500" : "#FFFFFF"}
                opacity={i === 0 ? 1 : 1}
              />
            ))}
          </g>
        </>
      )}

      {/* Orange sphere — breathe + squash on blink */}
      <circle
        cx="50"
        cy="50"
        r="22"
        fill="#FF5500"
        style={{
          transform: `scale(${sphereScaleX}, ${sphereScaleY})`,
          transformOrigin: "50px 50px",
        }}
      />

      {/* EYES — two vertical bars, rounded; saccade look + dual blink */}
      <rect
        x={EYE_X_OPEN + lookOffset}
        y={leftY}
        width="4"
        height={leftH}
        fill="#000"
        rx="2"
      />
      <rect
        x={EYE_X2_OPEN + lookOffset}
        y={rightY}
        width="4"
        height={rightH}
        fill="#000"
        rx="2"
      />
    </svg>
  );
};
