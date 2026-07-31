"use client";

import { useRef, useState, useCallback, useEffect } from "react";
import { animate } from "animejs";

/**
 * MarkCta — variant of the VantaDB mark for the CTA section.
 * Interactive: eyes + sphere look toward whichever CTA button is hovered.
 * Click on a button triggers a UNIQUE animation (different from hero blink):
 *  - install: sphere pulses + eyes go wide (excited)
 *  - docs: sphere spins + eyes squint (focused)
 *  - github: sphere bounces + eyes blink both (happy)
 * Smaller than hero mark, sits on black bg (ring is crema).
 */
export type CtaButton = "install" | "docs" | "github";

export function MarkCta({
  activeButton,
  clickButton,
}: {
  activeButton: CtaButton | null;
  clickButton: { button: CtaButton; timestamp: number } | null;
}) {
  const markRef = useRef<HTMLDivElement>(null);
  const leftEyeRef = useRef<SVGRectElement>(null);
  const rightEyeRef = useRef<SVGRectElement>(null);
  const sphereRef = useRef<SVGCircleElement>(null);
  const ringRef = useRef<SVGCircleElement>(null);
  const [reaction, setReaction] = useState(false);

  // Offsets — LARGE so eyes clearly look toward the button
  // install = far left, docs = up, github = far right
  const offsets: Record<CtaButton, { eyeX: number; eyeY: number; sphereX: number; sphereY: number }> = {
    install: { eyeX: -18, eyeY: 4, sphereX: -6, sphereY: 2 },
    docs: { eyeX: 0, eyeY: -14, sphereX: 0, sphereY: -5 },
    github: { eyeX: 18, eyeY: 4, sphereX: 6, sphereY: 2 },
  };

  const active = activeButton ? offsets[activeButton] : { eyeX: 0, eyeY: 0, sphereX: 0, sphereY: 0 };

  // UNIQUE click reactions — different from hero blink
  const handleClickReaction = useCallback((button: CtaButton) => {
    setReaction(true);

    if (button === "install") {
      // EXCITED: sphere pulses big, eyes go wide (height grows), then back
      if (sphereRef.current) {
        animate(sphereRef.current, {
          r: [22, 28, 22],
          duration: 400,
          ease: "outElastic(1, 0.5)",
        });
      }
      [leftEyeRef, rightEyeRef].forEach((eyeRef) => {
        if (!eyeRef.current) return;
        animate(eyeRef.current, {
          height: [10, 14, 10],
          duration: 400,
          ease: "outElastic(1, 0.5)",
        });
      });
    } else if (button === "docs") {
      // FOCUSED: sphere spins 360, eyes squint narrow
      if (sphereRef.current) {
        animate(sphereRef.current, {
          rotate: 360,
          duration: 500,
          ease: "inOutQuad",
          onComplete: () => {
            if (sphereRef.current) sphereRef.current.style.transform = "";
          },
        });
      }
      [leftEyeRef, rightEyeRef].forEach((eyeRef) => {
        if (!eyeRef.current) return;
        animate(eyeRef.current, {
          height: [10, 3, 10],
          duration: 500,
          ease: "inOutQuad",
        });
      });
    } else if (button === "github") {
      // HAPPY: sphere bounces down+up, both eyes blink together
      if (sphereRef.current) {
        animate(sphereRef.current, {
          cy: [50, 56, 50],
          duration: 350,
          ease: "outBounce",
        });
      }
      [leftEyeRef, rightEyeRef].forEach((eyeRef) => {
        if (!eyeRef.current) return;
        animate(eyeRef.current, {
          height: [10, 1.5, 10],
          duration: 250,
          ease: "inOutQuad",
        });
      });
    }

    // Ring flash on any click
    if (ringRef.current) {
      animate(ringRef.current, {
        strokeWidth: [3.5, 6, 3.5],
        duration: 300,
        ease: "outQuad",
      });
    }

    setTimeout(() => setReaction(false), 600);
  }, []);

  // Trigger reaction on clickButton change
  useEffect(() => {
    if (clickButton && clickButton.timestamp > 0) {
      handleClickReaction(clickButton.button);
    }
  }, [clickButton, handleClickReaction]);

  return (
    <div ref={markRef} className="relative mx-auto h-28 w-28 sm:h-36 sm:w-36">
      <svg
        width="100%"
        height="100%"
        viewBox="0 0 100 100"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-label="VantaDB mark"
        className="drop-shadow-[0_0_20px_rgba(255,85,0,0.3)]"
      >
        {/* Outer ring — crema border on black bg */}
        <circle
          ref={ringRef}
          cx="50"
          cy="50"
          r="42"
          fill="none"
          stroke="#FBF9F5"
          strokeWidth="3.5"
          style={{ transition: "stroke-width 0.2s ease-out" }}
        />

        {/* Glow ring (pulses) */}
        <circle cx="50" cy="50" r="42" fill="none" stroke="#FF5500" strokeWidth="0.6" opacity="0.4">
          <animate attributeName="r" values="42;46;42" dur="3.5s" repeatCount="indefinite" />
          <animate attributeName="opacity" values="0.4;0;0.4" dur="3.5s" repeatCount="indefinite" />
        </circle>

        {/* Orange sphere — shifts toward hovered button, reacts on click */}
        <circle
          ref={sphereRef}
          cx={50 + active.sphereX}
          cy={50 + active.sphereY}
          r="22"
          fill="#FF5500"
          style={{
            transformOrigin: `${50 + active.sphereX}px ${50 + active.sphereY}px`,
            transition: "cx 0.35s cubic-bezier(0.22,1,0.36,1), cy 0.35s cubic-bezier(0.22,1,0.36,1)",
          }}
        />

        {/* Eyes — look toward hovered button. Different Y per button for direction. */}
        <rect
          ref={leftEyeRef}
          x={43 + active.eyeX - 2}
          y={45 + active.eyeY}
          width="4"
          height={10}
          fill="#000"
          rx="2"
          style={{
            transition: "x 0.3s cubic-bezier(0.22,1,0.36,1), y 0.3s cubic-bezier(0.22,1,0.36,1)",
          }}
        />
        <rect
          ref={rightEyeRef}
          x={57 + active.eyeX - 2}
          y={45 + active.eyeY}
          width="4"
          height={10}
          fill="#000"
          rx="2"
          style={{
            transition: "x 0.3s cubic-bezier(0.22,1,0.36,1), y 0.3s cubic-bezier(0.22,1,0.36,1)",
          }}
        />
      </svg>
    </div>
  );
}
