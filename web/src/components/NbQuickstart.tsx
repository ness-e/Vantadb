import { Link } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";
import { gsap, ScrollTrigger, useGSAP } from "../lib/gsap";
import { NbSection, NbSectionHeader } from "./nb";
import "../styles/quickstart.css";

const STEPS = [
  {
    num: "01",
    title: "Install",
    cmd: "pip install vantadb-py",
    output: "Successfully installed vantadb-py-0.1.5",
  },
  {
    num: "02",
    title: "Initialize",
    cmd: 'db = VantaDB("./memory.db")',
    output: "[VantaDB] Initialized.",
  },
  {
    num: "03",
    title: "Store",
    cmd: 'db.put(namespace="agent/main", key="user_42", vector=[...])',
    output: "[VantaDB] Inserted 1 record.",
  },
  {
    num: "04",
    title: "Query",
    cmd: "results = db.search_memory(query=[...], top_k=5)",
    output: "Found 5 records. Score: 0.92",
  },
];

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function highlight(cmd: string): string {
  let html = "";
  let i = 0;
  while (i < cmd.length) {
    if (cmd[i] === '"' || cmd[i] === "'") {
      const q = cmd[i];
      let j = i + 1;
      while (j < cmd.length && cmd[j] !== q) {
        if (cmd[j] === "\\") j++;
        j++;
      }
      if (j < cmd.length) j++;
      html += `<span class="qs-tok-str">${esc(cmd.slice(i, j))}</span>`;
      i = j;
    } else if (/[a-zA-Z_]/.test(cmd[i])) {
      let j = i;
      while (j < cmd.length && /\w/.test(cmd[j])) j++;
      const w = cmd.slice(i, j);
      const kws = new Set([
        "import",
        "from",
        "def",
        "return",
        "if",
        "not",
        "and",
        "or",
        "True",
        "False",
        "None",
        "as",
        "for",
        "in",
      ]);
      if (kws.has(w)) html += `<span class="qs-tok-kw">${esc(w)}</span>`;
      else html += esc(w);
      i = j;
    } else if (/\d/.test(cmd[i])) {
      let j = i;
      while (j < cmd.length && /[\d.]/.test(cmd[j])) j++;
      html += `<span class="qs-tok-num">${esc(cmd.slice(i, j))}</span>`;
      i = j;
    } else {
      html += esc(cmd[i]);
      i++;
    }
  }
  return html;
}

const HIGHLIGHTED = STEPS.map((s) => highlight(s.cmd));

export function NbQuickstart() {
  const [activeStep, setActiveStep] = useState(0);
  const [allComplete, setAllComplete] = useState(false);
  const [hasEntered, setHasEntered] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);
  const codeRefs = useRef<(HTMLElement | null)[]>([]);
  const beamRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);

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

  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;
    STEPS.forEach((_, i) => {
      const el = codeRefs.current[i];
      if (el) {
        el.innerHTML = HIGHLIGHTED[i];
        el.dataset.qsHl = "1";
      }
    });
  }, []);

  const typeStep = useCallback((stepIndex: number, onComplete: () => void) => {
    const step = STEPS[stepIndex];
    if (!step) {
      onComplete();
      return;
    }
    setActiveStep(stepIndex);

    const el = codeRefs.current[stepIndex];
    if (el) {
      gsap.killTweensOf(el);
      el.textContent = "";
      delete el.dataset.qsHl;
      const dur = Math.max(0.25, step.cmd.length * 0.03);
      gsap.to(el, {
        duration: dur,
        text: step.cmd,
        ease: "none",
        onComplete: () => {
          el.innerHTML = HIGHLIGHTED[stepIndex];
          el.dataset.qsHl = "1";
          onComplete();
        },
      });
    } else onComplete();
  }, []);

  const handleClick = useCallback((i: number) => {
    setAllComplete(false);
    setActiveStep(i);
    const el = codeRefs.current[i];
    if (el) {
      gsap.killTweensOf(el);
      el.innerHTML = HIGHLIGHTED[i];
      el.dataset.qsHl = "1";
    }
  }, []);

  useEffect(() => {
    if (!hasEntered) return;
    let cancelled = false;
    const runLoop = () => {
      if (cancelled) return;
      setAllComplete(false);
      let i = 0;
      const playNext = () => {
        if (cancelled) return;
        if (i >= STEPS.length) {
          setActiveStep(-1);
          setAllComplete(true);
          return;
        }
        typeStep(i, () => {
          i++;
          setTimeout(playNext, 1000);
        });
      };
      playNext();
    };
    runLoop();
    return () => {
      cancelled = true;
    };
  }, [hasEntered, typeStep]);

  return (
    <NbSection ref={sectionRef} variant="lg" ariaLabel="Quickstart guide">
      <NbSectionHeader
        monoLabel="[MATRIX]"
        headline="Four commands. One database."
        sub="Click any step to jump. Watch the sequence beam track progress."
      />

      <div className="qs-matrix">
        {/* Sequence beam */}
        <div
          ref={beamRef}
          className={`qs-beam ${allComplete ? "qs-beam--done" : ""}`}
          style={{
            top: activeStep < 0 || activeStep >= 2 ? "50%" : activeStep <= 1 ? "25%" : "25%",
            left: activeStep < 0 ? "50%" : activeStep % 2 === 0 ? "25%" : "75%",
            opacity: activeStep >= 0 ? 1 : 0,
          }}
          aria-hidden="true"
        />

        {/* Grid */}
        {STEPS.map((step, i) => {
          const isActive = !allComplete && activeStep === i;
          const isPast = !allComplete && i < activeStep;
          const col = (i % 2) + 1;
          const row = Math.floor(i / 2) + 1;

          return (
            <button
              key={step.num}
              type="button"
              className={`qs-matrix-card ${isActive ? "qs-matrix-card--active" : ""} ${isPast ? "qs-matrix-card--past" : ""} ${allComplete ? "qs-matrix-card--done" : ""}`}
              style={{ gridRow: row, gridColumn: col }}
              onClick={() => handleClick(i)}
              aria-current={isActive ? "step" : undefined}
            >
              {/* Terminal header */}
              <div className="qs-matrix-term-bar" aria-hidden="true">
                <span className="qs-matrix-term-dot" />
                <span className="qs-matrix-term-dot" />
                <span className="qs-matrix-term-dot" />
                <span className="qs-matrix-term-label">{step.title.toUpperCase()}</span>
                {isActive && <span className="qs-matrix-term-badge">RUN</span>}
                {allComplete && (
                  <span className="qs-matrix-term-badge qs-matrix-term-badge--done">OK</span>
                )}
              </div>

              {/* Command */}
              <div className="qs-matrix-cmd">
                <span className="qs-matrix-prompt">$</span>
                <code
                  ref={(el) => {
                    codeRefs.current[i] = el;
                  }}
                  className="qs-matrix-code"
                  aria-live="polite"
                />
                {isActive && <span className="qs-cursor-qm" aria-hidden="true" />}
              </div>

              {/* Output */}
              <div className="qs-matrix-out">
                <span className="qs-matrix-out-prefix">&gt;</span>
                <span className="qs-matrix-out-text">{step.output}</span>
              </div>
            </button>
          );
        })}
      </div>

      <div className="qs-cta-row">
        <Link to="/docs" className="nb-arrow">
          READ DOCUMENTATION
        </Link>
      </div>
    </NbSection>
  );
}
