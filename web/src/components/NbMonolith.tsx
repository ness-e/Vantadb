import { useRef, useState, useCallback, memo } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";
import "../styles/monolith.css";

const CLI_COMMAND = "pip install vantadb-py";

export const NbMonolith = memo(function NbMonolith() {
  const containerRef = useRef<HTMLElement>(null);
  const feedbackRef = useRef<HTMLSpanElement>(null);
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(CLI_COMMAND);
      setCopied(true);
      if (feedbackRef.current) {
        feedbackRef.current.style.opacity = "1";
      }
      setTimeout(() => {
        setCopied(false);
        if (feedbackRef.current) {
          feedbackRef.current.style.opacity = "0";
        }
      }, 2000);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = CLI_COMMAND;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      if (feedbackRef.current) {
        feedbackRef.current.style.opacity = "1";
      }
      setTimeout(() => {
        setCopied(false);
        if (feedbackRef.current) {
          feedbackRef.current.style.opacity = "0";
        }
      }, 2000);
    }
  }, []);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.fromTo(
          ".monolith-command",
          { clipPath: "inset(0 0 100% 0)", opacity: 0 },
          {
            clipPath: "inset(0)",
            opacity: 1,
            duration: 0.5,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
            scrollTrigger: {
              trigger: containerRef.current,
              start: "top 75%",
            },
          },
        );

        gsap.fromTo(
          ".monolith-subtitle",
          { opacity: 0, y: 12 },
          {
            opacity: 1,
            y: 0,
            duration: 0.35,
            delay: 0.2,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
            scrollTrigger: {
              trigger: containerRef.current,
              start: "top 75%",
            },
          },
        );

        gsap.fromTo(
          ".monolith-telemetry span",
          { opacity: 0 },
          {
            opacity: 1,
            duration: 0.25,
            stagger: 0.12,
            delay: 0.4,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
            scrollTrigger: {
              trigger: containerRef.current,
              start: "top 75%",
            },
          },
        );
      });
    },
    { scope: containerRef },
  );

  return (
    <section
      ref={containerRef}
      className="nb-section nb-section--lg nb-section--dark"
      aria-label="Get started"
    >
      <div className="nb-inner">
        <div className="monolith-block">
          <div className="monolith-command-wrap">
            <span className="monolith-command-prefix" aria-hidden="true">
              $
            </span>
            <code
              className="monolith-command"
              onClick={handleCopy}
              role="button"
              tabIndex={0}
              aria-label="Copy install command"
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") handleCopy();
              }}
            >
              {CLI_COMMAND}
              <span className="monolith-cursor" aria-hidden="true" />
            </code>
            <button
              className="monolith-copy-btn"
              onClick={handleCopy}
              aria-label="Copy to clipboard"
              type="button"
            >
              {copied ? "OK" : "[]"}
            </button>
            <span
              ref={feedbackRef}
              className="monolith-copy-feedback nb-monolith-feedback"
              aria-live="polite"
            >
              {copied ? "copied to clipboard" : ""}
            </span>
          </div>

          <p className="monolith-subtitle">Zero servers. One line. Infinite context.</p>

          <div className="monolith-cta-row">
            <Link to="/docs" className="monolith-cta" aria-label="Get started with VantaDB">
              Get Started
            </Link>
          </div>
        </div>

        <div className="monolith-telemetry" aria-label="Package metadata">
          <span>ONE BINARY</span>
          <span>ZERO DEPS</span>
          <span>MIT LICENSE</span>
        </div>
      </div>
    </section>
  );
});
