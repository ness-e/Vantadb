"use client";

import { useEffect, useState } from "react";

/**
 * ScrollProgress — a neon orange progress bar fixed to the top of the viewport.
 * Tracks scroll position relative to total scrollable height.
 * Respects prefers-reduced-motion (disables the width transition).
 */
export function ScrollProgress() {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    let ticking = false;

    const update = () => {
      const scrollTop = window.scrollY;
      const docHeight =
        document.documentElement.scrollHeight - window.innerHeight;
      const pct = docHeight > 0 ? (scrollTop / docHeight) * 100 : 0;
      setProgress(Math.min(Math.max(pct, 0), 100));
      ticking = false;
    };

    const onScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(update);
        ticking = true;
      }
    };

    update();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll, { passive: true });
    return () => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
    };
  }, []);

  return (
    <div
      className="fixed left-0 right-0 top-0 z-[60] h-1.5 bg-transparent"
      aria-hidden="true"
      role="presentation"
    >
      <div
        className="h-full bg-[#FF5500] shadow-[0_0_8px_0_#FF5500]"
        style={{
          width: `${progress}%`,
          transition: "width 80ms linear",
        }}
      />
      {/* Tick marks every 25% for a technical gauge feel */}
      {[25, 50, 75].map((mark) => (
        <div
          key={mark}
          className="absolute top-0 h-1.5 w-px bg-black/20 "
          style={{ left: `${mark}%` }}
        />
      ))}
    </div>
  );
}
