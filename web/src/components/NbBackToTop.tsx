import { memo, useEffect, useRef } from "react";
import { animate } from "motion";
import { useReducedMotion } from "../hooks/useReducedMotion";

export const NbBackToTop = memo(function NbBackToTop() {
  const btnRef = useRef<HTMLButtonElement>(null);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    if (!btnRef.current) return;

    if (reducedMotion) return;

    const btn = btnRef.current;
    btn.style.opacity = "0";
    btn.style.transform = "translateY(16px)";

    let lastScrollY = window.scrollY;
    let anim: ReturnType<typeof animate> | null = null;

    const onScroll = () => {
      const currentScrollY = window.scrollY;
      const direction = currentScrollY > lastScrollY ? 1 : -1;
      lastScrollY = currentScrollY;

      anim?.stop();

      if (
        currentScrollY > 500 &&
        (direction === -1 ||
          currentScrollY + window.innerHeight >= document.documentElement.scrollHeight - 10)
      ) {
        anim = animate(
          btn,
          { opacity: 1, y: 0 },
          { duration: 0.25, ease: [0.25, 0.46, 0.45, 0.94] },
        );
      } else if (currentScrollY > 500 && direction === 1) {
        anim = animate(
          btn,
          { opacity: 0, y: 16 },
          { duration: 0.25, ease: [0.55, 0.085, 0.68, 0.53] },
        );
      }
    };

    window.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      window.removeEventListener("scroll", onScroll);
      anim?.stop();
    };
  }, [reducedMotion]);

  const scrollToTop = () => {
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  return (
    <button
      ref={btnRef}
      type="button"
      className="nb-back-to-top"
      onClick={scrollToTop}
      aria-label="Back to top"
    >
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="square"
        strokeLinejoin="miter"
        aria-hidden="true"
        focusable="false"
      >
        <path d="M12 19V5M5 12l7-7 7 7" />
      </svg>
    </button>
  );
});
