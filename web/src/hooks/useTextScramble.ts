import { useCallback, useRef } from "react";
import { animate } from "animejs";

const GLITCH_CHARS = "01X%$&*/#<>[]{}";

export function useTextScramble() {
  const isScrambling = useRef(false);

  const scramble = useCallback((el: HTMLElement, targetText?: string, duration = 800) => {
    if (isScrambling.current || !el) return;
    isScrambling.current = true;

    const originalText = targetText || el.innerText;
    const length = originalText.length;
    const state = { p: 0 };

    animate(state, {
      p: 1,
      duration: duration,
      easing: "easeOutQuad",
      update: () => {
        const revealed = Math.floor(state.p * length);
        let result = originalText.slice(0, revealed);

        for (let i = revealed; i < length; i++) {
          if (originalText[i] === " ") {
            result += " ";
          } else {
            result += GLITCH_CHARS[Math.floor(Math.random() * GLITCH_CHARS.length)];
          }
        }

        el.innerText = result;
      },
      complete: () => {
        el.innerText = originalText;
        isScrambling.current = false;
      },
    });
  }, []);

  return { scramble };
}
