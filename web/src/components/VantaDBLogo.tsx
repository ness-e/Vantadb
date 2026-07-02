/**
 * VantaDBLogo — Animated SVG Identity System
 *
 * Concept: "V" letterform converging at a gravitational singularity point.
 * The V's two strokes = data trajectories collapsing into the void.
 * The vertex = event horizon / singularity (Vantablack metaphor).
 * The orbit ellipse = accretion disk, drawn with stroke-dasharray animation.
 *
 * Variants:
 *   mark   → Icon only (V + singularity + orbit ring)
 *   full   → Icon + "VantaDB" wordmark (horizontal)
 *   stacked → Icon + "VantaDB" wordmark (vertical, centered)
 *   badge  → Dark rounded-square container with icon centered
 *
 * Animation philosophy (Emil Kowalski):
 *   - stroke-dasharray draw-in on mount (CSS animation, off main thread)
 *   - Orbit ring rotates continuously — linear, GPU-composited
 *   - Hover: singularity pulse + subtle V stroke glow
 *   - prefers-reduced-motion: removes all motion, preserves opacity fade
 *   - No bounce. No scale(0). All easing is ease-out.
 */

import React from "react";

// ─────────────────────────────────────────────
// SVG GEOMETRY CONSTANTS
// V mark: 64×64 viewBox
// V legs: from (8, 6) and (56, 6) converging to (32, 54)
// Singularity: circle cx=32 cy=54 r=4
// Orbit ellipse: cx=32 cy=54, rx=16 ry=6, rotate(-12deg)
// ─────────────────────────────────────────────

type LogoVariant = "mark" | "full" | "stacked" | "badge";
type LogoSize = "xs" | "sm" | "md" | "lg" | "xl";

interface VantaDBLogoProps {
  variant?: LogoVariant;
  size?: LogoSize;
  /** Override amber with custom color */
  accentColor?: string;
  /** Mute animation (overrides prefers-reduced-motion) */
  noAnimation?: boolean;
  className?: string;
  /** Accessible label */
  "aria-label"?: string;
}

const SIZE_MAP: Record<LogoSize, number> = {
  xs: 20,
  sm: 28,
  md: 40,
  lg: 56,
  xl: 80,
};

// Orbit ellipse circumference ≈ π * (3(rx+ry) - √((3rx+ry)(rx+3ry)))
// For rx=16, ry=6 → ≈ π*(3(22) - √((54)(34))) ≈ π*(66-42.9) ≈ 72.4px
const ORBIT_CIRCUMFERENCE = 72.4;

// V stroke path: approximate stroke length for each leg
// line from (8,6) to (32,54) → √((24)²+(48)²) ≈ 53.7
const V_STROKE_LENGTH = 54;

const VantaDBMark: React.FC<{
  size?: number;
  accentColor?: string;
  noAnimation?: boolean;
  className?: string;
}> = ({
  size = 40,
  accentColor = "var(--amber, #FF6A00)",
  noAnimation = false,
  className = "",
}) => {
  const animClass = noAnimation ? "vdb-logo--no-anim" : "";

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
      className={`vdb-mark ${animClass} ${className}`}
    >
      {/* ── Outer Circle (Stroke) ──────────────── */}
      <circle
        cx="32"
        cy="32"
        r="24"
        fill="none"
        stroke="var(--foreground)"
        strokeWidth="6"
      />

      {/* ── Inner Core (Solid Amber) ───────────── */}
      <circle 
        cx="32" 
        cy="32" 
        r="10" 
        fill={accentColor} 
      />
    </svg>
  );
};

// ─────────────────────────────────────────────
// FULL LOGO: Mark + Wordmark (horizontal)
// ─────────────────────────────────────────────
export const VantaDBLogoFull: React.FC<{
  size?: LogoSize;
  noAnimation?: boolean;
  className?: string;
}> = ({ size = "md", noAnimation = false, className = "" }) => {
  const markPx = SIZE_MAP[size];
  const fontPx = Math.round(markPx * 0.7); // Letra un poco más grande para igualar la proporción de la imagen

  return (
    <div
      className={`vdb-logo-full ${noAnimation ? "vdb-logo--no-anim" : ""} ${className}`}
      role="img"
      aria-label="VantaDB"
      style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}
    >
      <VantaDBMark size={markPx} noAnimation={noAnimation} />
      <span 
        className="vdb-wordmark" 
        style={{ 
          fontSize: `${fontPx}px`, 
          fontWeight: 800, 
          letterSpacing: "-0.02em", 
          color: "var(--foreground)" 
        }} 
        aria-hidden="true"
      >
        VantaDB
      </span>
    </div>
  );
};

// ─────────────────────────────────────────────
// BADGE: Rounded-square container (app icon)
// ─────────────────────────────────────────────
const VantaDBBadge: React.FC<{
  size?: LogoSize;
  noAnimation?: boolean;
  className?: string;
}> = ({ size = "lg", noAnimation = false, className = "" }) => {
  const markPx = SIZE_MAP[size];
  const badgePx = Math.round(markPx * 1.6);
  const radius = Math.round(badgePx * 0.22);

  return (
    <div
      className={`vdb-badge ${noAnimation ? "vdb-logo--no-anim" : ""} ${className}`}
      style={{
        width: badgePx,
        height: badgePx,
        borderRadius: radius,
      }}
      role="img"
      aria-label="VantaDB"
    >
      <VantaDBMark size={markPx} noAnimation={noAnimation} />
    </div>
  );
};

// ─────────────────────────────────────────────
// DEFAULT EXPORT: Polymorphic component
// ─────────────────────────────────────────────
const VantaDBLogo: React.FC<VantaDBLogoProps> = ({
  variant = "full",
  size = "md",
  accentColor,
  noAnimation = false,
  className = "",
  "aria-label": ariaLabel = "VantaDB",
}) => {
  const markPx = SIZE_MAP[size];

  switch (variant) {
    case "mark":
      return (
        <div role="img" aria-label={ariaLabel} className={className}>
          <VantaDBMark size={markPx} accentColor={accentColor} noAnimation={noAnimation} />
        </div>
      );
    case "badge":
      return <VantaDBBadge size={size} noAnimation={noAnimation} className={className} />;
    case "stacked":
      return (
        <div
          className={`vdb-logo-stacked ${noAnimation ? "vdb-logo--no-anim" : ""} ${className}`}
          role="img"
          aria-label={ariaLabel}
        >
          <VantaDBMark size={markPx} accentColor={accentColor} noAnimation={noAnimation} />
          <span
            className="vdb-wordmark vdb-wordmark--stacked"
            style={{ fontSize: `${Math.round(markPx * 0.45)}px` }}
            aria-hidden="true"
          >
            Vanta<span className="vdb-wordmark-db">DB</span>
          </span>
        </div>
      );
    case "full":
    default:
      return <VantaDBLogoFull size={size} noAnimation={noAnimation} className={className} />;
  }
};

export default VantaDBLogo;
