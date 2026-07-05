import { useRef, memo } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

export const NbBackToTop = memo(function NbBackToTop() {
  const btnRef = useRef<HTMLButtonElement>(null);
  const stRef = useRef<ScrollTrigger | null>(null);

  useGSAP(() => {
    const mm = gsap.matchMedia();
    mm.add("(prefers-reduced-motion: no-preference)", () => {
      if (!btnRef.current) return;

      gsap.set(btnRef.current, { autoAlpha: 0, y: 16 });

      stRef.current = ScrollTrigger.create({
        start: 500,
        end: "max",
        onUpdate: (self) => {
          if (self.direction === -1 || self.progress === 1) {
            gsap.to(btnRef.current, {
              autoAlpha: 1,
              y: 0,
              duration: 0.25,
              ease: "power2.out",
              overwrite: true,
            });
          } else if (self.direction === 1) {
            gsap.to(btnRef.current, {
              autoAlpha: 0,
              y: 16,
              duration: 0.25,
              ease: "power2.in",
              overwrite: true,
            });
          }
        },
        onLeaveBack: () => {
          gsap.to(btnRef.current, {
            autoAlpha: 0,
            y: 16,
            duration: 0.25,
            ease: "power2.in",
            overwrite: true,
          });
        },
      });
    });
    return () => {
      stRef.current?.kill();
      stRef.current = null;
      mm.revert();
    };
  }, []);

  const scrollToTop = () => {
    gsap.to(window, {
      scrollTo: { y: 0 },
      duration: 0.5,
      ease: "power2.out",
    });
  };

  return (
    <button
      ref={btnRef}
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
      >
        <path d="M12 19V5M5 12l7-7 7 7" />
      </svg>
    </button>
  );
});
