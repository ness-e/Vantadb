import { gsap } from "../lib/gsap";

type RevealTarget = string | (HTMLElement | null)[];

interface RevealOptions {
  stagger?: number;
  duration?: number;
  start?: string;
  from?: string;
}

export function useSwissReveal(
  target: RevealTarget,
  trigger?: HTMLElement | null,
  options: RevealOptions = {},
): () => void {
  const elements =
    typeof target === "string"
      ? Array.from(document.querySelectorAll(target))
      : target.filter((e): e is HTMLElement => e !== null);

  if (elements.length === 0) return () => {};

  const { stagger = 0.06, duration = 0.35, start = "top 80%", from = "0 0 100% 0" } = options;

  const mm = gsap.matchMedia();
  mm.add("(prefers-reduced-motion: no-preference)", () => {
    gsap.fromTo(
      elements,
      { clipPath: `inset(${from})`, opacity: 0 },
      {
        clipPath: "inset(0)",
        opacity: 1,
        duration,
        stagger,
        ease: "cubic-bezier(0.25, 1, 0.5, 1)",
        scrollTrigger: {
          trigger: trigger ?? elements[0]?.parentElement ?? undefined,
          start,
        },
      },
    );
  });

  return () => mm.revert();
}
