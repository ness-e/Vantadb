"use client";

/**
 * VantaLogoMark — shared SVG mark used in navbar, footer, and CTA.
 * Outer ring: black border, NO fill (transparent).
 * Inner: neon orange sphere.
 * Eyes: two vertical bars (cat eyes).
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
      <rect x="27" y="27" width="2.5" height="10" fill="#000" rx="0.5" />
      <rect x="34.5" y="27" width="2.5" height="10" fill="#000" rx="0.5" />
    </svg>
  );
}
