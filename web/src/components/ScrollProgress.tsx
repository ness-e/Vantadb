import { useEffect, useState } from "react";

const prefersMotion =
  typeof window !== "undefined"
    ? window.matchMedia("(prefers-reduced-motion: no-preference)").matches
    : true;

export function ScrollProgress() {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    if (!prefersMotion) return;

    let frame: number;
    const onScroll = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const scrollTop = window.scrollY;
        const docHeight = document.documentElement.scrollHeight - window.innerHeight;
        setProgress(docHeight > 0 ? Math.min(scrollTop / docHeight, 1) : 0);
      });
    };

    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();

    return () => {
      window.removeEventListener("scroll", onScroll);
      cancelAnimationFrame(frame);
    };
  }, []);

  if (!prefersMotion) return null;

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: `${progress * 100}%`,
        height: 2,
        background: "var(--amber)",
        zIndex: 9999,
        pointerEvents: "none",
        transition: "width 100ms linear",
      }}
      role="progressbar"
      aria-valuenow={Math.round(progress * 100)}
      aria-valuemin={0}
      aria-valuemax={100}
    />
  );
}
