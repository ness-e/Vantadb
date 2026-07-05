import { useState, useRef, useEffect, useCallback } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP, TextPlugin, ScrollTrigger } from "../lib/gsap";
import "../styles/quickstart.css";

const STEPS = [
  {
    num: "01",
    title: "Install",
    cmd: "pip install vantadb-py",
    desc: "Single package. No native dependencies.",
    output: "Successfully installed vantadb-py-0.1.5",
  },
  {
    num: "02",
    title: "Initialize",
    cmd: 'db = VantaDB("./memory.db")',
    desc: "One import. Database file created automatically.",
    output: "[VantaDB] Initialized successfully.",
  },
  {
    num: "03",
    title: "Store",
    cmd: 'db.put(namespace="agent/main", key="user_42", ..., vector=[...])',
    desc: "Schema-free. Text + metadata + vectors in one call.",
    output: "[VantaDB] Inserted 1 record. Vector stored.",
  },
  {
    num: "04",
    title: "Query",
    cmd: "results = db.search_memory(query=[...], top_k=5)",
    desc: "Semantic + keyword fused in a single query.",
    output: "Found 5 records. Score: 0.92",
  },
];

export function NbQuickstart() {
  const [activeStep, setActiveStep] = useState(0);
  const [hasEntered, setHasEntered] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);
  const codeRef = useRef<HTMLElement>(null);
  const loopRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        ScrollTrigger.create({
          trigger: sectionRef.current,
          start: "top 75%",
          onEnter: () => setHasEntered(true),
          once: true,
        });
      });
    },
    { scope: sectionRef },
  );

  const typeStep = useCallback((stepIndex: number, onComplete: () => void) => {
    const step = STEPS[stepIndex];
    if (!step) {
      onComplete();
      return;
    }

    setActiveStep(stepIndex);

    if (codeRef.current) {
      gsap.killTweensOf(codeRef.current);

      const duration = Math.max(0.25, step.cmd.length * 0.03);

      gsap.to(codeRef.current, {
        duration,
        text: step.cmd,
        ease: "none",
        onComplete,
      });
    } else {
      onComplete();
    }
  }, []);

  useEffect(() => {
    if (!hasEntered) return;

    let cancelled = false;

    const runLoop = () => {
      if (cancelled) return;

      let i = 0;
      const playNext = () => {
        if (cancelled) return;
        if (i >= STEPS.length) {
          loopRef.current = setTimeout(runLoop, 3000);
          return;
        }
        typeStep(i, () => {
          i++;
          setTimeout(playNext, 1200);
        });
      };
      playNext();
    };

    runLoop();

    return () => {
      cancelled = true;
      if (loopRef.current) clearTimeout(loopRef.current);
    };
  }, [hasEntered, typeStep]);

  return (
    <section ref={sectionRef} className="nb-section nb-section--lg" aria-label="Quickstart guide">
      <div className="nb-inner">
        <span className="nb-mono-label">[QUICKSTART]</span>
        <h2 className="nb-section-headline">Zero to running.</h2>
        <p className="nb-section-sub">Four commands. One embedded database.</p>

        <div className="nb-qs-grid">
          {STEPS.map((step, i) => {
            const isActive = activeStep === i;
            return (
              <button
                key={step.num}
                type="button"
                className={`nb-qs-card ${isActive ? "nb-qs-card--active" : ""}`}
                onClick={() => {
                  setActiveStep(i);
                  if (codeRef.current) {
                    gsap.killTweensOf(codeRef.current);
                    gsap.set(codeRef.current, { text: step.cmd });
                  }
                }}
                aria-current={isActive ? "step" : undefined}
              >
                <span className="nb-qs-card-num" aria-hidden="true">
                  {step.num}
                </span>

                <div className="nb-qs-card-cmd">
                  <span className="nb-qs-card-prompt" aria-hidden="true">
                    $
                  </span>
                  {isActive ? (
                    <code ref={codeRef} className="nb-qs-card-code" aria-live="polite" />
                  ) : (
                    <code className="nb-qs-card-code">{step.cmd}</code>
                  )}
                  {isActive && <span className="nb-qs-cursor" aria-hidden="true" />}
                </div>

                <h3 className="nb-qs-card-title">{step.title}</h3>
                <p className="nb-qs-card-desc">{step.desc}</p>

                {isActive && <div className="nb-qs-card-out">{step.output}</div>}
              </button>
            );
          })}
        </div>

        <div className="nb-qs-cta">
          <Link to="/docs" className="nb-arrow" aria-label="Read documentation">
            Read Documentation
          </Link>
        </div>
      </div>
    </section>
  );
}
