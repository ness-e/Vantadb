"use client";

/**
 * VantaLogoMark — shared SVG mark used in navbar, footer, and CTA.
 * Outer ring: black border, NO fill (transparent).
 * Inner: neon orange sphere.
 * Eyes: two vertical capsule bars — same geometry as the hero mark
 * (hero-mark-interactive.tsx / mark-classic.tsx), scaled 100→64.
 * Inverts colors in dark mode.
 */
export function VantaLogoMark({ size = 36 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
      className="shrink-0"
    >
      <circle
        cx="32"
        cy="32"
        r="28"
        fill="none"
        stroke="currentColor"
        strokeWidth="4"
        className="text-black "
      />
      <circle cx="32" cy="32" r="14" fill="#FF5500" />
      <rect x="26.24" y="28.8" width="2.56" height="6.4" fill="#000" rx="1.28" />
      <rect x="35.2" y="28.8" width="2.56" height="6.4" fill="#000" rx="1.28" />
    </svg>
  );
}
