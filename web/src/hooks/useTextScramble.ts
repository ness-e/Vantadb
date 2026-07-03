import { useCallback, useRef } from "react";
import { gsap } from "../lib/gsap";

const GLITCH_CHARS = "01X%$&*/#<>[]{}";

export function useTextScramble() {
  const isScrambling = useRef(false);

  const scramble = useCallback((el: HTMLElement, targetText?: string, duration = 800) => {
    if (isScrambling.current || !el) return;
    isScrambling.current = true;

    const originalText = targetText || el.innerText;
    const length = originalText.length;

    const state = { progress: 0 };

    gsap.to(state, {
      progress: 1,
      duration: duration / 1000,
      ease: "power2.out",
      onUpdate: () => {
        const revealed = Math.floor(state.progress * length);
        let result = originalText.slice(0, revealed);
        for (let i = revealed; i < length; i++) {
          result += originalText[i] === " " ? " " : GLITCH_CHARS[Math.floor(Math.random() * GLITCH_CHARS.length)];
        }
        el.innerText = result;
      },
      onComplete: () => {
        el.innerText = originalText;
        isScrambling.current = false;
      },
    });
  }, []);

  return { scramble };
}
