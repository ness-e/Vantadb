"use client";

import { useState, useMemo, useRef, useCallback } from "react";
import { Play, RotateCcw, Terminal, Zap, ChevronDown } from "lucide-react";
import { Reveal } from "./reveal";
import { cn } from "@/lib/utils";

// Inline lightweight Python tokenizer for syntax highlighting overlay
const HL_KEYWORDS = new Set(["import", "as", "def", "return", "from", "class", "if", "else", "elif", "for", "while", "in", "not", "and", "or", "None", "True", "False", "with", "try", "except", "lambda", "pass", "break", "continue", "self"]);
const HL_BUILTINS = new Set(["print", "len", "range", "str", "int", "float", "list", "dict", "set", "tuple", "bool", "open", "enumerate", "zip", "map", "filter", "sorted", "sum", "min", "max", "abs", "round", "type", "format"]);

const HL_CLASS: Record<string, string> = {
  plain: "text-[#FBF9F5]",
  comment: "text-[#8a8a8a] italic",
  string: "text-[#FFB380]",
  number: "text-[#a3d9a5]",
  keyword: "text-[#FF5500] font-bold",
  builtin: "text-[#7ec7ff]",
  func: "text-[#ffd479]",
  ident: "text-[#FBF9F5]",
  op: "text-[#c9c9c9]",
};

function hlTokenize(line: string) {
  const tokens: { t: string; v: string }[] = [];
  let i = 0;
  while (i < line.length) {
    const rest = line.slice(i);
    if (rest.startsWith("#")) { tokens.push({ t: "comment", v: rest }); break; }
    const strMatch = rest.match(/^("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/);
    if (strMatch) { tokens.push({ t: "string", v: strMatch[0] }); i += strMatch[0].length; continue; }
    const numMatch = rest.match(/^\d[\d_]*(\.\d+)?/);
    if (numMatch) { tokens.push({ t: "number", v: numMatch[0] }); i += numMatch[0].length; continue; }
    const idMatch = rest.match(/^[A-Za-z_][A-Za-z0-9_]*/);
    if (idMatch) {
      const word = idMatch[0];
      const after = line[i + word.length];
      let t = "ident";
      if (HL_KEYWORDS.has(word)) t = "keyword";
      else if (HL_BUILTINS.has(word)) t = "builtin";
      else if (after === "(") t = "func";
      tokens.push({ t, v: word });
      i += word.length;
      continue;
    }
    const opMatch = rest.match(/^(==|!=|<=|>=|->|\+=|-=|\*=|\/\/=|\/\/|\*\*|[=+\-*/%<>:,.(){}\[\]])/);
    if (opMatch) { tokens.push({ t: "op", v: opMatch[0] }); i += opMatch[0].length; continue; }
    const wsMatch = rest.match(/^\s+/);
    if (wsMatch) { tokens.push({ t: "plain", v: wsMatch[0] }); i += wsMatch[0].length; continue; }
    tokens.push({ t: "plain", v: rest[0] }); i += 1;
  }
  return tokens;
}

// Simulated execution: pattern-matches the user's Python-ish input and produces
// illustrative output. This is NOT a real Python interpreter — it's a demo.
function simulateRun(code: string): string[] {
  const lines: string[] = [];
  const trimmed = code.trim();

  // Detect key patterns
  if (trimmed.includes("VantaDB(") || trimmed.includes("vantadb.")) {
    lines.push("✓ VantaDB instance initialized (./vanta_data)");
    lines.push("✓ WAL opened · CRC32C checksums active");
  }
  if (trimmed.includes("db.put(")) {
    const putCount = (trimmed.match(/db\.put\(/g) || []).length;
    lines.push(`✓ put() · ${putCount} record(s) stored`);
    lines.push("  → payload + metadata + vector indexed");
  }
  if (trimmed.includes("db.get(")) {
    lines.push("✓ get() · canonical record retrieved");
    lines.push('  → key="memory-001" · version=1');
  }
  if (trimmed.includes("db.search(")) {
    lines.push("✓ search() · hybrid query planned");
    lines.push("  → BM25 path: 47 candidates");
    lines.push("  → HNSW path: 52 candidates (cosine)");
    lines.push("  → RRF fusion: top_k=5 ranked");
    lines.push("  → 1.2ms · 100% Recall@10");
  }
  if (trimmed.includes("db.flush()")) {
    lines.push("✓ flush() · WAL synced to disk");
  }
  if (trimmed.includes("db.close()")) {
    lines.push("✓ close() · handles released safely");
  }
  if (trimmed.includes("print(")) {
    // Extract print arguments
    const printMatches = trimmed.matchAll(/print\(([^)]*)\)/g);
    for (const m of printMatches) {
      let arg = m[1].trim();
      // Strip quotes
      if ((arg.startsWith('"') && arg.endsWith('"')) || (arg.startsWith("'") && arg.endsWith("'"))) {
        arg = arg.slice(1, -1);
      }
      lines.push(`> ${arg}`);
    }
  }
  if (trimmed.includes("import")) {
    lines.push("✓ modules loaded");
  }

  if (lines.length === 0) {
    lines.push("→ (no recognizable VantaDB calls detected)");
    lines.push("  try: db.put(...), db.search(...), db.get(...)");
  }

  lines.push("");
  lines.push(`◆ executed in ${(0.8 + Math.random() * 1.5).toFixed(2)}ms · in-process`);
  return lines;
}

const STARTER_CODE = `import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data")

db.put("agent/main", "mem-001", "hello vanta", vector=[0.1, 0.9, 0.5])
stored = db.get("agent/main", "mem-001")
hits = db.search("agent/main", vector=[0.11, 0.89, 0.55], top_k=5)

print(hits)
db.flush()
db.close()`;

const EXAMPLES = [
  {
    name: "Full Quickstart",
    code: STARTER_CODE,
  },
  {
    name: "Put & Get",
    code: `import vantadb_py as vantadb\n\ndb = vantadb.VantaDB("./vanta_data")\n\n# Store a record with vector\nrecord = db.put(\n    "agent/main",\n    "memory-001",\n    "In-process execution minimizes latency.",\n    metadata={"category": "architecture", "priority": 1},\n    vector=[0.12, 0.88, 0.54],\n)\n\n# Retrieve by exact key\nstored = db.get("agent/main", "memory-001")\nprint(stored)\n\ndb.flush()\ndb.close()`,
  },
  {
    name: "Hybrid Search",
    code: `import vantadb_py as vantadb\n\ndb = vantadb.VantaDB("./vanta_data")\n\n# Insert documents with vectors\nfor i in range(5):\n    db.put("docs", f"doc-{i}", f"document content {i}",\n             vector=[0.1 * i, 0.9 - 0.1 * i, 0.5])\n\n# Hybrid search: BM25 + HNSW via RRF\nhits = db.search("docs", vector=[0.2, 0.8, 0.5], top_k=5)\n\nfor hit in hits:\n    print(f"{hit.key} score={hit.score}")\n\ndb.flush()\ndb.close()`,
  },
  {
    name: "Batch Insert",
    code: `import vantadb_py as vantadb\n\ndb = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)\n\n# Bulk insert 100 records\nfor i in range(100):\n    vec = [i / 100.0, 1.0 - i / 100.0, 0.5]\n    db.put("agent/main", f"mem-{i}", f"record {i}", vector=vec)\n\nprint(f"Inserted 100 records")\n\n# Search across all\nhits = db.search("agent/main", vector=[0.5, 0.5, 0.5], top_k=10)\nprint(f"Found {len(hits)} results")\n\ndb.flush()\ndb.close()`,
  },
];

export function CodePlayground() {
  const [activeExample, setActiveExample] = useState(0);
  const [examplesOpen, setExamplesOpen] = useState(false);
  const [code, setCode] = useState(STARTER_CODE);
  const [output, setOutput] = useState<string[] | null>(null);
  const [running, setRunning] = useState(false);

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
    if (!src) { syncing.current = false; return; }
    const { scrollTop, scrollLeft } = src;
    if (gutterRef.current && source !== "gutter") gutterRef.current.scrollTop = scrollTop;
    if (preRef.current && source !== "pre") { preRef.current.scrollTop = scrollTop; preRef.current.scrollLeft = scrollLeft; }
    if (textareaRef.current && source !== "textarea") { textareaRef.current.scrollTop = scrollTop; textareaRef.current.scrollLeft = scrollLeft; }
    requestAnimationFrame(() => { syncing.current = false; });
  }, []);

  const run = () => {
    setRunning(true);
    setOutput(null);
    setTimeout(() => {
      setOutput(simulateRun(code));
      setRunning(false);
    }, 600);
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
                Edit the code and hit Run. The simulator pattern-matches VantaDB calls
                and produces illustrative output — not a real interpreter.
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

        <Reveal direction="up" delay={60}>
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            {/* Editor */}
            <div className="border-4 border-black bg-black shadow-[6px_6px_0_0_#000]  ">
              <div className="flex items-center justify-between border-b-2 border-[#FBF9F5]/20 bg-[#1A1A1A] px-3 py-2">
                <span className="inline-flex items-center gap-1.5 font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/70">
                  <Terminal className="h-3 w-3 text-[#FF5500]" />
                  playground.py
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
                          hlTokenize(line).map((tok, j) => (
                            <span key={j} className={HL_CLASS[tok.t]}>{tok.v}</span>
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
                    aria-label="Python code editor"
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
                    executing
                  </span>
                )}
              </div>
              <div className="scroll-manga h-80 overflow-auto p-3">
                {output === null && !running && (
                  <p className="font-tech text-[11px] text-[#FBF9F5]/30">
                    {"// press Run to execute"}
                  </p>
                )}
                {running && (
                  <div className="space-y-1">
                    <p className="font-tech text-[11px] text-[#FF5500]">
                      <span className="animate-blink">▋</span> planning query...
                    </p>
                  </div>
                )}
                {output && !running && (
                  <div className="space-y-0.5">
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
            This is a pattern-matching simulator for demo purposes. For real execution,
            install VantaDB with{" "}
            <code className="font-mono">pip install vantadb-py</code> and run locally.
          </p>
        </Reveal>
      </div>
    </section>
  );
}
