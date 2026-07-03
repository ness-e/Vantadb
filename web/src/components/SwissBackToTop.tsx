import { useRef, memo } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

export const SwissBackToTop = memo(function SwissBackToTop() {
  const btnRef = useRef<HTMLButtonElement>(null);

  useGSAP(() => {
    if (!btnRef.current) return;

    gsap.set(btnRef.current, { autoAlpha: 0, y: 20 });

    ScrollTrigger.create({
      start: 500,
      end: "max",
      onUpdate: (self) => {
        // Mostrar cuando subes el scroll (direction === -1) o cuando llegas al fondo (progress === 1)
        if (self.direction === -1 || self.progress === 1) {
          gsap.to(btnRef.current, {
            autoAlpha: 1,
            y: 0,
            duration: 0.3,
            ease: "power2.out",
            overwrite: true,
          });
        }
        // Esconder cuando bajas
        else if (self.direction === 1) {
          gsap.to(btnRef.current, {
            autoAlpha: 0,
            y: 20,
            duration: 0.3,
            ease: "power2.in",
            overwrite: true,
          });
        }
      },
      onLeaveBack: () => {
        gsap.to(btnRef.current, {
          autoAlpha: 0,
          y: 20,
          duration: 0.3,
          ease: "power2.in",
          overwrite: true,
        });
      },
    });
  }, []);

  const scrollToTop = () => {
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  return (
    <button
      ref={btnRef}
      className="swiss-back-to-top"
      onClick={scrollToTop}
      aria-label="Back to top"
    >
      <svg
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="square"
        strokeLinejoin="miter"
      >
        <path d="M12 19V5M5 12l7-7 7 7" />
      </svg>
    </button>
  );
});
