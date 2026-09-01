"use client";

import { useState, useMemo, useRef, useCallback } from "react";
import { useLanguage } from "@/lib/language-provider";
import { Play, RotateCcw, Terminal, Zap, ChevronDown } from "lucide-react";
import { Reveal } from "./reveal";
import { jsTokenizer, TOK_CLASS } from "@/lib/code-tokenizer";
import { cn } from "@/lib/utils";
import { PlaygroundExecutor, type PlaygroundExecutorHandle } from "./playground-executor";

// ── WEB-07 — Sandbox iframe ─────────────────────────────────────────────────
// El código del usuario se ejecuta dentro de <iframe sandbox="allow-scripts
// allow-same-origin" src="/playground-executor.html">. El iframe carga el
// WASM (vantadb_wasm.js + .wasm), expone new Function aislado del DOM
// principal, y devuelve output vía postMessage. Ver playground-executor.tsx
// y public/playground-executor.html. Decisión: allow-same-origin requerido
// para fetch /vanta-wasm/* sin CORS; sin allow-top-navigation/forms/popups
// el snippet no puede navegar, enviar forms ni escapar del sandbox.
// Si el playground algún día acepta código de terceros compartido vía URL,
// mantener esta arquitectura; si se sirve WASM con CORS/blob, reducir a
// allow-scripts solo (ponytail: techo documentado en playground-executor.tsx).

const STARTER_CODE = `const rec = db.put({
  namespace: "agent/main",
  key: "mem-001",
  payload: "hello vanta",
  vector: [0.1, 0.9, 0.5],
});
console.log("stored", rec.key, "->", rec.payload);

const stored = db.get("agent/main", "mem-001");
console.log("get", stored.key, "->", stored.payload);

const hits = db.search({
  namespace: "agent/main",
  query_vector: [0.11, 0.89, 0.55],
  top_k: 5,
});
for (const hit of hits) {
  console.log(hit.record.key, "score=" + hit.score.toFixed(4));
}

db.flush();`;

const EXAMPLES = [
  {
    name: "Full Quickstart",
    code: STARTER_CODE,
  },
  {
    name: "Put & Get",
    code: `const record = db.put({
  namespace: "agent/main",
  key: "memory-001",
  payload: "In-process execution minimizes latency.",
  metadata: { category: "architecture", priority: 1 },
  vector: [0.12, 0.88, 0.54],
});
console.log("put version", record.version);

const stored = db.get("agent/main", "memory-001");
console.log("get", stored.payload);
console.log("metadata", stored.metadata);

db.flush();`,
  },
  {
    name: "Hybrid Search",
    code: `for (let i = 0; i < 5; i++) {
  db.put({
    namespace: "docs",
    key: "doc-" + i,
    payload: "document content " + i,
    vector: [0.1 * i, 0.9 - 0.1 * i, 0.5],
  });
}

// Hybrid search: vector similarity + optional text query
const hits = db.search({
  namespace: "docs",
  query_vector: [0.2, 0.8, 0.5],
  top_k: 5,
});
for (const hit of hits) {
  console.log(hit.record.key, "score=" + hit.score.toFixed(4));
}

db.flush();`,
  },
  {
    name: "Batch Insert",
    code: `const batch = [];
for (let i = 0; i < 100; i++) {
  batch.push({
    namespace: "agent/main",
    key: "mem-" + i,
    payload: "record " + i,
    vector: [i / 100.0, 1.0 - i / 100.0, 0.5],
  });
}
db.put_batch(batch);
console.log("inserted", batch.length, "records");

const hits = db.search({
  namespace: "agent/main",
  query_vector: [0.5, 0.5, 0.5],
  top_k: 10,
});
console.log("found", hits.length, "results");

db.flush();`,
  },
];

export function CodePlayground() {
  const { tt } = useLanguage();
  const [activeExample, setActiveExample] = useState(0);
  const [examplesOpen, setExamplesOpen] = useState(false);
  const [code, setCode] = useState(STARTER_CODE);
  const [output, setOutput] = useState<string[] | null>(null);
  const [running, setRunning] = useState(false);
  const executorRef = useRef<PlaygroundExecutorHandle>(null);
  const [executorReady, setExecutorReady] = useState(false);

  const lineCount = useMemo(() => code.split("\n").length, [code]);

  // Scroll sync: gutter + overlay pre + textarea scroll together
  const gutterRef = useRef<HTMLDivElement>(null);
  const preRef = useRef<HTMLPreElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const syncing = useRef(false);

  const syncScroll = useCallback((source: "gutter" | "pre" | "textarea") => {
    if (syncing.current) return;
    syncing.current = true;
    const src = source === "gutter" ? gutterRef.current : source === "pre" ? preRef.current : textareaRef.current;
    if (!src) {
      syncing.current = false;
      return;
    }
    const { scrollTop, scrollLeft } = src;
    if (gutterRef.current && source !== "gutter") gutterRef.current.scrollTop = scrollTop;
    if (preRef.current && source !== "pre") {
      preRef.current.scrollTop = scrollTop;
      preRef.current.scrollLeft = scrollLeft;
    }
    if (textareaRef.current && source !== "textarea") {
      textareaRef.current.scrollTop = scrollTop;
      textareaRef.current.scrollLeft = scrollLeft;
    }
    requestAnimationFrame(() => {
      syncing.current = false;
    });
  }, []);

  const run = async () => {
    setRunning(true);
    setOutput(null);
    const executor = executorRef.current;
    if (!executor) {
      setOutput(["✗ playground executor not ready", "  iframe ref missing"]);
      setRunning(false);
      return;
    }
    // Wait up to 5s for iframe ready (covers cold load race)
    if (!executor.isReady()) {
      for (let i = 0; i < 10; i++) {
        await new Promise((r) => setTimeout(r, 500));
        if (executor.isReady()) break;
        // nudge iframe via ping (executor also polls, this is extra)
        try {
          const iframe = document.querySelector('iframe[title="VantaDB Playground Executor"]') as HTMLIFrameElement | null;
          iframe?.contentWindow?.postMessage({ type: "ping" }, "*");
        } catch {}
      }
      if (!executor.isReady()) {
        setOutput([
          "✗ playground executor not ready",
          "  iframe sandbox aún no cargó — reintentá en 1s",
        ]);
        setRunning(false);
        return;
      }
    }
    try {
      const result = await executor.execute(code);
      if (result.error) {
        setOutput(["✗ " + result.error]);
      } else {
        setOutput(result.output);
      }
    } catch (err) {
      setOutput(["✗ unexpected error", `  ${err instanceof Error ? err.message : String(err)}`]);
    } finally {
      setRunning(false);
    }
  };

  const reset = () => {
    setActiveExample(0);
    setCode(EXAMPLES[0].code);
    setOutput(null);
  };

  const loadExample = (idx: number) => {
    setActiveExample(idx);
    setCode(EXAMPLES[idx].code);
    setOutput(null);
    setExamplesOpen(false);
  };

  return (
    <section className="relative border-b-4 border-black bg-[#FBF9F5]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-5xl px-4 py-14 sm:px-6">
        <Reveal direction="up">
          <div className="mb-6 flex flex-col gap-3 border-4 border-black bg-[#FF5500] p-6 shadow-[6px_6px_0_0_#000]   sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="mt-3 font-display text-3xl uppercase leading-none text-black sm:text-4xl">
                Code Playground
              </h2>
              <p className="mt-2 max-w-lg font-tech text-xs text-black/80">
                Edit the code and hit Run. Each run opens a real VantaDB instance
                compiled to WebAssembly (vantadb-wasm) and executes your snippet
                against it — in your browser (sandboxed iframe).
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              {/* Examples dropdown */}
              <div className="relative">
                <button
                  onClick={() => setExamplesOpen((o) => !o)}
                  className="press inline-flex items-center gap-2 border-4 border-black bg-[#FBF9F5] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black   "
                  aria-label="Load example"
                  aria-expanded={examplesOpen}
                >
                  <Terminal className="h-3.5 w-3.5" strokeWidth={2.5} />
                  {EXAMPLES[activeExample].name}
                  <ChevronDown className={cn("h-3 w-3 transition-transform", examplesOpen && "rotate-180")} />
                </button>
                {examplesOpen && (
                  <div className="absolute left-0 top-full z-50 mt-1 border-4 border-black bg-[#FBF9F5] shadow-[6px_6px_0_0_#000]   ">
                    {EXAMPLES.map((ex, i) => (
                      <button
                        key={ex.name}
                        onClick={() => loadExample(i)}
                        className={cn(
                          "block w-full border-b-2 border-black px-4 py-2 text-left font-tech text-xs uppercase tracking-wider transition-colors hover:bg-[#FF5500] hover:text-black last:border-b-0 ",
                          i === activeExample ? "bg-[#FF5500] text-black" : "text-black "
                        )}
                      >
                        {ex.name}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <button
                onClick={reset}
                className="press inline-flex items-center gap-2 border-4 border-black bg-[#FBF9F5] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black   "
                aria-label="Reset code"
              >
                <RotateCcw className="h-3.5 w-3.5" strokeWidth={2.5} />
                Reset
              </button>
              <button
                onClick={run}
                disabled={running}
                className="press inline-flex items-center gap-2 border-4 border-black bg-black px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider text-[#FF5500] disabled:opacity-50"
                aria-label="Run code"
              >
                <Play className="h-3.5 w-3.5 fill-current" strokeWidth={2.5} />
                {running ? "Running..." : "Run"}
              </button>
            </div>
          </div>
        </Reveal>

        {/* Hidden sandboxed iframe executor — WEB-07 */}
        <PlaygroundExecutor ref={executorRef} onReady={() => setExecutorReady(true)} />
        {!executorReady && (
          <p className="sr-only" aria-live="polite">Playground executor loading…</p>
        )}

        <Reveal direction="up" delay={60}>
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            {/* Editor */}
            <div className="border-4 border-black bg-black shadow-[6px_6px_0_0_#000]  ">
              <div className="flex items-center justify-between border-b-2 border-[#FBF9F5]/20 bg-[#1A1A1A] px-3 py-2">
                <span className="inline-flex items-center gap-1.5 font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/70">
                  <Terminal className="h-3 w-3 text-[#FF5500]" />
                  playground.js
                </span>
                <span className="font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/30">
                  {lineCount} lines
                </span>
              </div>
              <div className="relative flex h-80 overflow-hidden">
                {/* Line numbers gutter */}
                <div
                  ref={gutterRef}
                  onScroll={() => syncScroll("gutter")}
                  className="scroll-manga shrink-0 select-none overflow-auto border-r border-[#FBF9F5]/10 bg-[#1A1A1A] py-3 pl-2 pr-2 text-right font-tech text-[12px] leading-relaxed text-[#FBF9F5]/25"
                  aria-hidden="true"
                >
                  {code.split("\n").map((_, idx) => (
                    <div key={idx}>{idx + 1}</div>
                  ))}
                </div>
                {/* Code area */}
                <div className="relative flex-1 overflow-hidden">
                  {/* Syntax highlight overlay */}
                  <pre
                    ref={preRef}
                    onScroll={() => syncScroll("pre")}
                    className="scroll-manga pointer-events-none absolute inset-0 overflow-auto p-3 font-tech text-[12px] leading-relaxed"
                    aria-hidden="true"
                  >
                    {code.split("\n").map((line, idx) => (
                      <div key={idx}>
                        {line.length === 0 ? (
                          <span>&nbsp;</span>
                        ) : (
                          jsTokenizer(line).map((tok, j) => (
                            <span key={j} className={TOK_CLASS[tok.t]}>{tok.v}</span>
                          ))
                        )}
                      </div>
                    ))}
                  </pre>
                  {/* Transparent textarea on top */}
                  <textarea
                    ref={textareaRef}
                    onScroll={() => syncScroll("textarea")}
                    value={code}
                    onChange={(e) => setCode(e.target.value)}
                    spellCheck={false}
                    aria-label="JavaScript code editor"
                    className="scroll-manga absolute inset-0 h-full w-full resize-none bg-transparent p-3 font-tech text-[12px] leading-relaxed text-transparent caret-[#FF5500] focus:outline-none"
                    style={{ tabSize: 4 }}
                  />
                </div>
              </div>
            </div>

            {/* Output */}
            <div className="border-4 border-black bg-black shadow-[6px_6px_0_0_#000]  ">
              <div className="flex items-center justify-between border-b-2 border-[#FBF9F5]/20 bg-[#1A1A1A] px-3 py-2">
                <span className="inline-flex items-center gap-1.5 font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/70">
                  <Zap className="h-3 w-3 text-[#FF5500]" />
                  output
                </span>
                {running && (
                  <span className="flex items-center gap-1 font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
                    <span className="animate-blink">▋</span>
                    executing wasm
                  </span>
                )}
              </div>
              <div className="scroll-manga h-80 overflow-auto p-3">
                {output === null && !running && (
                  <p className="font-tech text-[11px] text-[#FBF9F5]/30">
                    {tt("playground.pressRun", "// press Run to execute")}
                  </p>
                )}
                {running && (
                  <div className="space-y-1">
                    <p className="font-tech text-[11px] text-[#FF5500]">
                      <span className="animate-blink">▋</span> executing in sandboxed iframe...
                    </p>
                  </div>
                )}
                {output && !running && (
                  <div className="space-y-0.5">
                    {output.some((o) => o.startsWith("✓")) && (
                      <div id="pia-wasm-result" aria-hidden className="hidden" />
                    )}
                    {output.map((line, i) => (
                      <p
                        key={i}
                        className={cn(
                          "font-tech text-[11px] leading-relaxed",
                          line.startsWith("✓")
                            ? "text-[#a3d9a5]"
                            : line.startsWith("→") || line.startsWith("  →")
                              ? "text-[#7ec7ff]"
                              : line.startsWith(">")
                                ? "text-[#ffd479]"
                                : line.startsWith("✗")
                                  ? "text-[#ff7a7a]"
                                  : line.startsWith("◆")
                                    ? "text-[#FF5500] font-bold"
                                    : "text-[#FBF9F5]/60"
                        )}
                      >
                        {line || "\u00A0"}
                      </p>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <p className="mt-4 border-l-4 border-[#FF5500] bg-[#FBF9F5] px-4 py-2 font-tech text-[11px] italic text-black/70  ">
            <span className="font-bold not-italic uppercase tracking-wider">Note:</span>{" "}
            Each Run opens a fresh in-memory VantaDB instance (wasm32 engine) inside a
            sandboxed iframe (<code className="font-mono">allow-scripts</code>). Data is
            not persisted between runs — for browser persistence use{" "}
            <code className="font-mono">await VantaDB.connect_persistent(path)</code>.
          </p>
        </Reveal>
      </div>
    </section>
  );
}
