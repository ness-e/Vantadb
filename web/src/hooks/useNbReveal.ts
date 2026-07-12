import { animate, inView } from "motion";
import { useReducedMotion } from "./useReducedMotion";

type RevealTarget = string | (HTMLElement | null)[];

interface RevealOptions {
  stagger?: number;
  duration?: number;
  start?: string;
  from?: string;
}

export function useNbReveal(
  target: RevealTarget,
  trigger?: HTMLElement | null,
  options: RevealOptions = {},
): () => void {
  const reducedMotion = useReducedMotion();
  if (reducedMotion) return () => {};

  const elements =
    typeof target === "string"
      ? Array.from(document.querySelectorAll(target))
      : target.filter((e): e is HTMLElement => e !== null);

  if (elements.length === 0) return () => {};

  const { stagger = 0.06, duration = 0.35, from = "0 0 100% 0" } = options;
  const startMargin = "0px 0px -20% 0px";

  const cleanup = inView(
    trigger ?? elements[0]?.parentElement ?? document.body,
    () => {
      animate(
        elements,
        {
          clipPath: [`inset(${from})`, "inset(0)"],
          opacity: [0, 1],
        },
        {
          duration,
          delay: stagger,
          ease: [0.25, 1, 0.5, 1],
        },
      );
    },
    { amount: 0.2, margin: startMargin },
  );

  return () => cleanup?.();
}
