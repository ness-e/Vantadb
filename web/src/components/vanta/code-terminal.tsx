"use client";

import { useState, useMemo } from "react";
import { Copy, Check, Terminal, Play } from "lucide-react";
import { QUICKSTART_PYTHON } from "./vanta-data";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";
import { Reveal } from "./reveal";
import { useTypingLines } from "@/hooks/use-typing-lines";
import { useLanguage } from "@/lib/language-provider";
import { cn } from "@/lib/utils";

type Tok = {
  t: "plain" | "comment" | "string" | "number" | "keyword" | "builtin" | "func" | "ident" | "op";
  v: string;
};

const KEYWORDS = new Set([
  "import", "as", "def", "return", "from", "class", "if", "else", "elif",
  "for", "while", "in", "not", "and", "or", "None", "True", "False",
  "with", "try", "except", "lambda", "pass", "break", "continue", "self",
]);

const BUILTINS = new Set([
  "print", "len", "range", "str", "int", "float", "list", "dict", "set",
  "tuple", "bool", "open", "isinstance", "enumerate", "zip", "map", "filter",
  "sorted", "reversed", "sum", "min", "max", "abs", "round", "type", "format",
]);

function tokenizeLine(line: string): Tok[] {
  const tokens: Tok[] = [];
  let i = 0;
  while (i < line.length) {
    const rest = line.slice(i);
    // comment
    if (rest.startsWith("#")) {
      tokens.push({ t: "comment", v: rest });
      break;
    }
    // string (double or single quoted, with escapes)
    const strMatch = rest.match(/^("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/);
    if (strMatch) {
      tokens.push({ t: "string", v: strMatch[0] });
      i += strMatch[0].length;
      continue;
    }
    // number
    const numMatch = rest.match(/^\d[\d_]*(\.\d+)?/);
    if (numMatch) {
      tokens.push({ t: "number", v: numMatch[0] });
      i += numMatch[0].length;
      continue;
    }
    // identifier / keyword / builtin / function call
    const idMatch = rest.match(/^[A-Za-z_][A-Za-z0-9_]*/);
    if (idMatch) {
      const word = idMatch[0];
      // Look ahead: if followed by "(" it's a function call
      const afterIdx = i + word.length;
      const after = line[afterIdx];
      let t: Tok["t"] = "ident";
      if (KEYWORDS.has(word)) t = "keyword";
      else if (BUILTINS.has(word)) t = "builtin";
      else if (after === "(") t = "func";
      tokens.push({ t, v: word });
      i += word.length;
      continue;
    }
    // operators / punctuation
    const opMatch = rest.match(/^(==|!=|<=|>=|->|\+=|-=|\*=|\/\/=|\/\/|\*\*|[=+\-*/%<>:,.(){}\[\]])/);
    if (opMatch) {
      tokens.push({ t: "op", v: opMatch[0] });
      i += opMatch[0].length;
      continue;
    }
    // whitespace run
    const wsMatch = rest.match(/^\s+/);
    if (wsMatch) {
      tokens.push({ t: "plain", v: wsMatch[0] });
      i += wsMatch[0].length;
      continue;
    }
    // single char
    tokens.push({ t: "plain", v: rest[0] });
    i += 1;
  }
  return tokens;
}

const TOK_CLASS: Record<Tok["t"], string> = {
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

export function CodeTerminal() {
  const { t } = useLanguage();
  const [copied, setCopied] = useState(false);
  const [running, setRunning] = useState(false);

  const lines = useMemo(() => QUICKSTART_PYTHON.split("\n"), []);

  // Typing animation: reveal lines one-by-one when scrolled into view
  const { ref: typingRef, visibleLines, done: typingDone } = useTypingLines(lines.length, {
    threshold: 0.25,
    lineDelay: 90,
  });

  const copy = async () => {
    const ok = await copyToClipboard(QUICKSTART_PYTHON);
    if (ok) {
      setCopied(true);
      toast.copy(t("terminal.codeCopied"));
      setTimeout(() => setCopied(false), 1600);
    }
  };

  const run = () => {
    setRunning(true);
    setTimeout(() => setRunning(false), 2000);
  };

  return (
    <section className="relative border-b-4 border-black bg-[#FBF9F5]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-40" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-12 lg:gap-10">
          {/* Left — caption / framing */}
          <Reveal direction="right" className="lg:col-span-4">
          <div>
            <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
              5-Minute
              <br />
              <span className="text-outline-neon">Quickstart</span>
            </h2>
            <p className="mt-4 font-tech text-sm leading-relaxed text-black/80 ">
              Initialize a persistent memory store, save structured records with vectors,
              and execute hybrid retrieval in pure Python. No servers. No containers.
            </p>

            <ol className="mt-5 space-y-2">
              {[
                "Open or create a local DB (zero config)",
                "Store a record: payload + metadata + vector",
                "Retrieve exact record by key",
                "Hybrid search: BM25 + Cosine via RRF",
                "Telemetry & safe shutdown",
              ].map((step, i) => (
                <li
                  key={step}
                  className="flex items-start gap-3 border-l-4 border-black bg-[#F2EDE2] px-3 py-2 font-tech text-xs text-black   "
                >
                  <span className="font-display text-base text-[#FF5500]">
                    {i + 1}.
                  </span>
                  <span className="flex-1">{step}</span>
                </li>
              ))}
            </ol>

            <div className="mt-5 border-4 border-black bg-black p-3 shadow-[4px_4px_0_0_#FF5500] ">
              <p className="font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/60">
                Distribution note
              </p>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]">
                The package name is <span className="text-[#FF5500]">vantadb-py</span>, but
                the import uses an underscore:{" "}
                <code className="bg-[#FBF9F5]/10 px-1 text-[#FF5500]">
                  import vantadb_py
                </code>
              </p>
            </div>
          </div>
          </Reveal>

          {/* Right — terminal block */}
          <Reveal direction="left" className="lg:col-span-8">
          <div>
            <div className="border-4 border-black bg-black shadow-[8px_8px_0_0_#000]  ">
              {/* Title bar */}
              <div className="flex items-center justify-between border-b-4 border-[#FBF9F5]/20 bg-[#1A1A1A] px-3 py-2">
                <div className="flex items-center gap-2">
                  <span className="h-3 w-3 border-2 border-[#FBF9F5]/40 bg-[#FF5500]" />
                  <span className="h-3 w-3 border-2 border-[#FBF9F5]/40 bg-[#FBF9F5]/30" />
                  <span className="h-3 w-3 border-2 border-[#FBF9F5]/40 bg-[#FBF9F5]/30" />
                  <span className="ml-3 inline-flex items-center gap-1.5 font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/70">
                    <Terminal className="h-3 w-3 text-[#FF5500]" />
                    quickstart.py · vantadb_py
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={run}
                    className="inline-flex items-center gap-1 border-2 border-[#FBF9F5]/30 bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-wider text-black transition-transform hover:translate-y-[1px]"
                  >
                    <Play className="h-2.5 w-2.5" strokeWidth={3} />
                    {running ? "Running…" : "Run"}
                  </button>
                  <button
                    onClick={copy}
                    className="inline-flex items-center gap-1 border-2 border-[#FBF9F5]/30 bg-[#FBF9F5]/10 px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-wider text-[#FBF9F5] transition-colors hover:bg-[#FBF9F5]/20"
                  >
                    {copied ? (
                      <Check className="h-2.5 w-2.5 text-[#FF5500]" />
                    ) : (
                      <Copy className="h-2.5 w-2.5" />
                    )}
                    {copied ? "Copied" : "Copy"}
                  </button>
                </div>
              </div>

              {/* Code body */}
              <div ref={typingRef} className="scroll-manga max-h-[560px] overflow-auto">
                <pre className="min-w-full p-0 font-tech text-[12px] leading-relaxed sm:text-[13px]">
                  {lines.map((line, idx) => {
                    // Hide lines beyond the typing progress (unless done or running)
                    const isVisible = typingDone || running || idx < visibleLines;
                    if (!isVisible) return null;
                    return (
                    <div
                      key={idx}
                      className={cn(
                        "flex hover:bg-[#FBF9F5]/5",
                        running && idx < 6 && "animate-[vanta-rise_0.5s_ease_both]"
                      )}
                      style={running ? { animationDelay: `${idx * 60}ms` } : undefined}
                    >
                      <span className="sticky left-0 w-10 shrink-0 select-none border-r border-[#FBF9F5]/10 bg-[#1A1A1A] px-2 text-right text-[#FBF9F5]/30">
                        {idx + 1}
                      </span>
                      <code className="flex-1 whitespace-pre px-4">
                        {line.length === 0 ? (
                          <span>&nbsp;</span>
                        ) : (
                          tokenizeLine(line).map((tok, j) => (
                            <span key={j} className={TOK_CLASS[tok.t]}>
                              {tok.v}
                            </span>
                          ))
                        )}
                      </code>
                    </div>
                    );
                  })}
                </pre>
              </div>

              {/* Output bar */}
              {running && (
                <div className="border-t-4 border-[#FF5500] bg-[#0a0a0a] px-4 py-3 font-tech text-[11px] text-[#FF5500]">
                  <span className="animate-blink">▋</span> executing hybrid search ·
                  BM25 + HNSW via RRF · 1.2ms · 100% Recall@10
                </div>
              )}
              {!running && !typingDone && (
                <div className="border-t-2 border-[#FF5500]/40 bg-[#0a0a0a] px-4 py-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]/70">
                  <span className="animate-blink">▋</span> typing · {visibleLines}/{lines.length} lines
                </div>
              )}
              {!running && typingDone && (
                <div className="border-t-2 border-[#FBF9F5]/10 bg-[#0a0a0a] px-4 py-2 font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/40">
                  Expected output · record · stored · hits · hardware_profile
                </div>
              )}
            </div>

            {/* Terminal footer chips */}
            <div className="mt-3 flex flex-wrap items-center gap-2">
              {["zero config", "in-process", "RRF fusion", "WAL durable"].map((c) => (
                <span
                  key={c}
                  className="border-2 border-black bg-[#FBF9F5] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-wider text-black   "
                >
                  {c}
                </span>
              ))}
            </div>
          </div>
          </Reveal>
        </div>
      </div>
    </section>
  );
}
