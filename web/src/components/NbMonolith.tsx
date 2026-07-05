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
          ".nb-cta-command",
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
          ".nb-cta-sub",
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
          ".nb-meta-tag",
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
        <div className="nb-cta-frame">
          <h2 className="nb-cta-headline">SHIP IT.</h2>

          <div
            className="nb-cta-command"
            onClick={handleCopy}
            role="button"
            tabIndex={0}
            aria-label="Copy install command"
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") handleCopy();
            }}
          >
            <span className="nb-cta-prompt" aria-hidden="true">
              $
            </span>
            <code className="nb-cta-code">{CLI_COMMAND}</code>
            <span className="nb-cta-cursor" aria-hidden="true">
              _
            </span>
            <button
              className="nb-cta-copy"
              onClick={handleCopy}
              aria-label="Copy to clipboard"
              type="button"
            >
              {copied ? "OK" : "[]"}
            </button>
          </div>

          <p className="nb-cta-sub">Zero servers. One line. Infinite context.</p>

          <Link to="/docs" className="nb-btn nb-btn--ghost nb-btn--ghost-light">
            FULL DOCS
          </Link>

          <span ref={feedbackRef} className="nb-cta-feedback" aria-live="polite">
            {copied ? "copied to clipboard" : ""}
          </span>
        </div>

        <div
          className="nb-meta-row"
          style={{ marginTop: "var(--space-xl)", justifyContent: "center" }}
        >
          <span className="nb-meta-tag">ONE BINARY</span>
          <span className="nb-meta-tag">ZERO DEPS</span>
          <span className="nb-meta-tag">MIT LICENSE</span>
        </div>
      </div>
    </section>
  );
});
