"use client";

import {
  BookOpen,
  Terminal,
  Package,
  Boxes,
  Server,
  ArrowRight,
  Copy,
  Check,
  ChevronRight,
  Wrench,
  ShieldCheck,
  ExternalLink,
  Search,
  X,
} from "lucide-react";
import { useState, useMemo, useEffect } from "react";
import {
  VANTA,
  QUICKSTART_PYTHON,
  CLI_COMMANDS,
  DOC_LINKS,
} from "./vanta-data";
import type { View } from "./vanta-data";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";
import { CodePlayground } from "./code-playground";
import { Reveal } from "./reveal";

const SECTIONS = [
  { id: "install", label: "Installation", keywords: "pip install vantadb-py cargo rust binary wheel python" },
  { id: "quickstart", label: "5-Minute Quickstart", keywords: "python quickstart put get search hybrid rrf vantadb_py" },
  { id: "cli", label: "Embedded CLI", keywords: "vanta-cli put list export rebuild-index audit-index repair-text-index command" },
  { id: "server", label: "Server Mode", keywords: "vanta-server binary localhost 8080 network host smartscreen" },
  { id: "docs", label: "Full Docs", keywords: "architecture wal recovery telemetry configuration reliability" },
];

export function DocsView({ onNavigate }: { onNavigate: (v: View) => void }) {
  const [activeSection, setActiveSection] = useState("install");
  const [query, setQuery] = useState("");

  const scrollTo = (id: string) => {
    setActiveSection(id);
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  };

  // Scroll-spy: observe each section and highlight the active one in the sidebar
  useEffect(() => {
    const sectionEls = SECTIONS.map((s) => document.getElementById(s.id)).filter(
      Boolean
    ) as HTMLElement[];
    if (sectionEls.length === 0) return;

    const obs = new IntersectionObserver(
      (entries) => {
        // Find the entry closest to the top that is intersecting
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible.length > 0) {
          setActiveSection(visible[0].target.id);
        }
      },
      { rootMargin: "-80px 0px -60% 0px", threshold: 0 }
    );

    sectionEls.forEach((el) => obs.observe(el));
    return () => obs.disconnect();
  }, []);

  const filteredSections = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return SECTIONS;
    return SECTIONS.filter(
      (s) =>
        s.label.toLowerCase().includes(q) ||
        s.keywords.toLowerCase().includes(q)
    );
  }, [query]);

  const filteredDocs = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return DOC_LINKS;
    return DOC_LINKS.filter(
      (d) =>
        d.name.toLowerCase().includes(q) || d.desc.toLowerCase().includes(q)
    );
  }, [query]);

  return (
    <div className="animate-rise">
      {/* Header */}
      <section className="relative overflow-hidden border-b-4 border-black bg-[#FBF9F5]">
        <div className="pointer-events-none absolute inset-0 grid-tech" aria-hidden />
        <div className="pointer-events-none absolute -left-20 top-0 h-72 w-72 halftone halftone-fade opacity-25" aria-hidden />
        <div className="relative mx-auto max-w-7xl px-4 py-12 sm:px-6 sm:py-16">
          <div className="flex flex-wrap items-center gap-3">
            <span className="inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-3 py-1 font-display text-sm uppercase text-black shadow-[4px_4px_0_0_#000]">
              <BookOpen className="h-4 w-4" strokeWidth={2.5} />
              Manual
            </span>
            <span className="font-tech text-[10px] uppercase tracking-[0.3em] text-black/50">
              For developers · v0.1 MVP
            </span>
          </div>
          <h1 className="mt-5 font-display text-6xl uppercase leading-[0.85] text-black sm:text-8xl">
            Quick
            <br />
            <span className="text-outline-neon">start</span>
          </h1>
          <p className="mt-5 max-w-2xl border-l-4 border-[#FF5500] pl-4 font-tech text-sm leading-relaxed text-black/80 sm:text-base">
            From <code className="bg-black px-1.5 py-0.5 text-[#FF5500]">pip install</code>{" "}
            to your first hybrid search in under five minutes. Zero configuration, zero
            servers, zero network — just durable local memory and RRF-fused retrieval.
          </p>
        </div>
      </section>

      {/* Body: sidebar + content */}
      <section className="relative border-b-4 border-black bg-[#F2EDE2]">
        <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
        <div className="relative mx-auto grid max-w-7xl grid-cols-1 gap-8 px-4 py-12 sm:px-6 lg:grid-cols-12">
          {/* Sidebar TOC */}
          <aside className="lg:col-span-3">
            <div className="sticky top-28 border-4 border-black bg-[#FBF9F5] p-4 shadow-[6px_6px_0_0_#000]   ">
              <h3 className="mb-3 flex items-center gap-2 border-b-2 border-black pb-2 font-display text-sm uppercase text-black  ">
                <BookOpen className="h-4 w-4 text-[#FF5500]" strokeWidth={2.5} />
                Contents
              </h3>

              {/* Search input */}
              <div className="relative mb-3">
                <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-black/40 " strokeWidth={2.5} />
                <input
                  type="text"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Filtrar secciones..."
                  aria-label="Filtrar secciones de documentación"
                  className="w-full border-2 border-black bg-[#F2EDE2] py-1.5 pl-8 pr-7 font-tech text-xs text-black placeholder:text-black/40 focus:border-[#FF5500] focus:outline-none    "
                />
                {query && (
                  <button
                    onClick={() => setQuery("")}
                    className="absolute right-2 top-1/2 flex size-11 -translate-y-1/2 items-center justify-center text-black/50 hover:text-[#FF5500] "
                    aria-label="Limpiar búsqueda"
                  >
                    <X className="h-5 w-5" strokeWidth={2.5} />
                  </button>
                )}
              </div>

              <nav className="space-y-1" aria-label="Secciones de documentación">
                {filteredSections.length === 0 && (
                  <p className="border-2 border-dashed border-black/30 px-2 py-3 text-center font-tech text-[10px] uppercase tracking-wider text-black/40  ">
                    Sin resultados
                  </p>
                )}
                {filteredSections.map((s) => {
                  const i = SECTIONS.findIndex((x) => x.id === s.id);
                  return (
                    <button
                      key={s.id}
                      onClick={() => scrollTo(s.id)}
                      className={`flex w-full items-center gap-2 border-2 px-2 py-1.5 text-left font-tech text-xs font-bold uppercase tracking-wider transition-all ${
                        activeSection === s.id
                          ? "border-black bg-black text-[#FF5500]   "
                          : "border-transparent text-black/60 hover:border-black/20 hover:bg-[#F2EDE2]   "
                      }`}
                    >
                      <span className="font-mono text-[10px] opacity-60">
                        {String(i + 1).padStart(2, "0")}
                      </span>
                      <span className="flex-1">{s.label}</span>
                      {activeSection === s.id && (
                        <ChevronRight className="h-3 w-3" />
                      )}
                    </button>
                  );
                })}
              </nav>
              <div className="mt-4 border-t-2 border-black pt-3 ">
                <p className="mb-2 font-tech text-[9px] font-bold uppercase tracking-wider text-black/40 ">
                  {query ? `Docs filtradas (${filteredDocs.length})` : "Docs completas"}
                </p>
                <ul className="scroll-manga max-h-32 space-y-1 overflow-y-auto pr-1">
                  {filteredDocs.map((d) => (
                    <li key={d.name}>
                      <a
                        href={VANTA.repo}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="block font-tech text-[11px] leading-tight text-black/60 transition-colors hover:text-[#FF5500] "
                      >
                        {d.name}
                      </a>
                    </li>
                  ))}
                </ul>
                <a
                  href={VANTA.repo}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="mt-3 flex items-center gap-2 font-tech text-[11px] text-black/70 transition-colors hover:text-[#FF5500] "
                >
                  <ExternalLink className="h-3 w-3" />
                  Full docs on GitHub
                </a>
              </div>
            </div>
          </aside>

          {/* Main content */}
          <div className="space-y-10 lg:col-span-9">
            {/* Installation */}
            <DocSection id="install" tag="01" title="Installation" icon={Package}>
              <p className="font-tech text-sm leading-relaxed text-black/80">
                VantaDB ships as a native Python package with pre-compiled wheels for
                Windows, macOS, and Linux. Pick the path that matches your stack.
              </p>

              <InstallCard
                badge="Recommended"
                title="Python · pip"
                icon={Terminal}
              >
                <CodeBlock
                  lines={["pip install vantadb-py"]}
                  note="Distribution name is vantadb-py · import as vantadb_py (underscore)"
                />
              </InstallCard>

              <InstallCard title="Rust · Cargo.toml" icon={Boxes}>
                <CodeBlock
                  lines={["[dependencies]", 'vantadb = "0.1"']}
                  note="For Rust-native integration"
                />
              </InstallCard>

              <InstallCard title="Precompiled CLI binary" icon={Wrench}>
                <p className="mb-3 font-tech text-xs text-black/70">
                  Download and install the CLI binary in a single command — no compiling.
                </p>
                <CodeBlock
                  lines={[
                    "# Linux / macOS / WSL",
                    "curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh",
                  ]}
                />
                <div className="mt-2">
                  <CodeBlock
                    lines={[
                      "# Windows (PowerShell)",
                      "irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex",
                    ]}
                  />
                </div>
                <p className="mt-3 font-tech text-[11px] text-black/60">
                  Or via Cargo:{" "}
                  <code className="border border-black/30 bg-[#F2EDE2] px-1 font-mono text-[10px]">
                    cargo install --git https://github.com/ness-e/Vantadb.git --bin vanta-cli
                  </code>
                </p>
              </InstallCard>

              <div className="mt-4 flex items-start gap-3 border-l-4 border-[#FF5500] bg-[#FBF9F5] px-4 py-3">
                <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
                <p className="font-tech text-[11px] text-black/70">
                  <span className="font-bold uppercase tracking-wider">Dev from source:</span>{" "}
                  <code className="font-mono">pip install -e ./vantadb-python</code>
                </p>
              </div>
            </DocSection>

            {/* Quickstart */}
            <DocSection id="quickstart" tag="02" title="5-Minute Quickstart" icon={BookOpen}>
              <p className="font-tech text-sm leading-relaxed text-black/80">
                Initialize a persistent memory store, save structured records with vectors,
                and execute hybrid retrieval in pure Python.
              </p>

              <div className="overflow-hidden border-4 border-black bg-black shadow-[8px_8px_0_0_#000]">
                <div className="flex items-center justify-between border-b-4 border-[#FBF9F5]/20 bg-[#1A1A1A] px-3 py-2">
                  <span className="inline-flex items-center gap-1.5 font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/70">
                    <Terminal className="h-3 w-3 text-[#FF5500]" />
                    quickstart.py
                  </span>
                  <CopyButton text={QUICKSTART_PYTHON} />
                </div>
                <pre className="scroll-manga max-h-[520px] overflow-auto p-4 font-tech text-[12px] leading-relaxed text-[#FBF9F5] sm:text-[13px]">
                  {QUICKSTART_PYTHON}
                </pre>
              </div>

              {/* Step explanations */}
              <div className="mt-4 grid grid-cols-1 gap-3 md:grid-cols-2">
                {[
                  {
                    n: "1",
                    t: "Open or create a DB",
                    d: "Zero configuration. Specify a path and an optional memory budget. The WAL initializes on first write.",
                  },
                  {
                    n: "2",
                    t: "Store a record",
                    d: "put() writes a UTF-8 payload, scalar metadata, and an optional vector under a namespace + key.",
                  },
                  {
                    n: "3",
                    t: "Retrieve by key",
                    d: "get() returns the exact canonical record — the source of truth, always consistent.",
                  },
                  {
                    n: "4",
                    t: "Hybrid search",
                    d: "search() runs BM25 and HNSW in parallel, fusing ranks via RRF. Pass a vector, get top_k hits.",
                  },
                  {
                    n: "5",
                    t: "Telemetry & shutdown",
                    d: "hardware_profile() reports process memory. flush() forces WAL sync. close() releases handles safely.",
                  },
                ].map((s) => (
                  <div
                    key={s.n}
                    className="press flex items-start gap-3 border-4 border-black bg-[#FBF9F5] p-3"
                  >
                    <span className="flex h-7 w-7 shrink-0 items-center justify-center border-2 border-black bg-[#FF5500] font-display text-sm text-black">
                      {s.n}
                    </span>
                    <div>
                      <h4 className="font-tech text-xs font-bold uppercase tracking-wider text-black">
                        {s.t}
                      </h4>
                      <p className="mt-0.5 font-tech text-[11px] leading-relaxed text-black/70">
                        {s.d}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </DocSection>

            {/* CLI */}
            <DocSection id="cli" tag="03" title="Embedded CLI" icon={Terminal}>
              <p className="font-tech text-sm leading-relaxed text-black/80">
                For local development, debugging, or pipeline automation without Python.
                Once installed and on your <code className="font-mono">PATH</code>, use the
                global <code className="font-mono text-[#FF5500]">vanta-cli</code> command.
              </p>

              <div className="space-y-3">
                {CLI_COMMANDS.map((c) => (
                  <CliCard key={c.cmd} cmd={c.cmd} args={c.args} desc={c.desc} />
                ))}
              </div>

              <div className="mt-4 border-4 border-black bg-black p-3">
                <p className="font-tech text-[11px] text-[#FBF9F5]/70">
                  <span className="text-[#FF5500]">Tip:</span> developing inside the repo?{" "}
                  <code className="font-mono text-[#FF5500]">
                    cargo run --bin vanta-cli -- &lt;command&gt;
                    </code>{" "}
                  runs directly from source.
                </p>
              </div>
            </DocSection>

            {/* Server mode */}
            <DocSection id="server" tag="04" title="Optional Server Mode" icon={Server}>
              <p className="font-tech text-sm leading-relaxed text-black/80">
                Run the standalone binary to wrap the embedded core for local development or
                network exposure. This is not the primary product identity — it&apos;s an
                optional wrapper.
              </p>

              <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div className="border-4 border-black bg-[#FBF9F5] p-4">
                  <h4 className="mb-2 font-display text-lg uppercase text-black">
                    Defaults
                  </h4>
                  <ul className="space-y-1.5 font-tech text-[11px] text-black/70">
                    <li className="flex gap-2">
                      <span className="text-[#FF5500]">▸</span>
                      <span>
                        <span className="font-bold">Data dir:</span> creates{" "}
                        <code className="font-mono">vantadb_data/</code> in the cwd
                      </span>
                    </li>
                    <li className="flex gap-2">
                      <span className="text-[#FF5500]">▸</span>
                      <span>
                        <span className="font-bold">Bind:</span>{" "}
                        <code className="font-mono">127.0.0.1:8080</code> (safe localhost)
                      </span>
                    </li>
                  </ul>
                </div>
                <div className="border-4 border-black bg-[#FBF9F5] p-4  ">
                  <h4 className="mb-2 font-display text-lg uppercase text-black ">
                    Network exposure
                  </h4>
                  <CodeBlock
                    lines={["export VANTADB_HOST=0.0.0.0", "./vantadb-server-linux-amd64"]}
                    note="Override the bind address to expose to the network"
                  />
                </div>
              </div>

              <div className="mt-3 flex items-start gap-3 border-4 border-[#FF5500] bg-[#FF5500]/10 px-4 py-3">
                <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
                <p className="font-tech text-[11px] text-black/80">
                  <span className="font-bold uppercase tracking-wider">
                    Windows SmartScreen:
                  </span>{" "}
                  release binaries are unsigned — SmartScreen may warn “Unrecognized
                  Publisher”. Only run binaries from official GitHub Releases.
                </p>
              </div>
            </DocSection>

            {/* Code Playground */}
            <CodePlayground />

            {/* Full docs */}
            <DocSection id="docs" tag="05" title="Full Documentation" icon={BookOpen}>
              <p className="font-tech text-sm leading-relaxed text-black/80">
                The complete documentation set lives in the repository. Each doc covers a
                specific surface of the engine.
              </p>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                {DOC_LINKS.map((d) => (
                  <a
                    key={d.name}
                    href={VANTA.repo}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="press group flex items-start gap-3 border-4 border-black bg-[#FBF9F5] p-3"
                  >
                    <ExternalLink className="mt-0.5 h-4 w-4 shrink-0 text-[#FF5500] transition-transform group-hover:translate-x-0.5" strokeWidth={2.5} />
                    <div>
                      <h4 className="font-tech text-xs font-bold uppercase tracking-wider text-black">
                        {d.name}
                      </h4>
                      <p className="mt-0.5 font-tech text-[11px] text-black/60">{d.desc}</p>
                    </div>
                  </a>
                ))}
              </div>

              {/* Next steps CTA */}
              <div className="mt-6 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500] sm:flex-row">
                <div>
                  <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                    See the numbers
                  </h3>
                  <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                    Explore BENCH-01 and the SIFT1M Phase 2 speedups.
                  </p>
                </div>
                <button
                  onClick={() => onNavigate("benchmarks")}
                  className="press-neon inline-flex shrink-0 items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
                >
                  View Benchmarks
                  <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
                </button>
              </div>
            </DocSection>
          </div>
        </div>
      </section>
    </div>
  );
}

function DocSection({
  id,
  tag,
  title,
  icon: Icon,
  children,
}: {
  id: string;
  tag: string;
  title: string;
  icon: typeof BookOpen;
  children: React.ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-28">
      <Reveal direction="up">
      <div className="mb-4 flex items-center gap-3 border-b-4 border-black pb-3 ">
        <span className="inline-flex h-10 w-10 items-center justify-center border-4 border-black bg-[#FF5500] text-black ">
          <Icon className="h-5 w-5" strokeWidth={2.5} />
        </span>
        <div>
          <span className="font-tech text-[10px] uppercase tracking-[0.3em] text-black/50 ">
            §{tag}
          </span>
          <h2 className="glitch-hover font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
            {title}
          </h2>
        </div>
      </div>
      </Reveal>
      <Reveal direction="up" delay={80}>
      <div className="space-y-4">{children}</div>
      </Reveal>
    </section>
  );
}

function InstallCard({
  title,
  badge,
  icon: Icon,
  children,
}: {
  title: string;
  badge?: string;
  icon: typeof Package;
  children: React.ReactNode;
}) {
  return (
    <div className="border-4 border-black bg-[#FBF9F5] p-4 shadow-[4px_4px_0_0_#000]">
      <div className="mb-3 flex items-center justify-between">
        <h4 className="flex items-center gap-2 font-display text-lg uppercase text-black">
          <Icon className="h-4 w-4 text-[#FF5500]" strokeWidth={2.5} />
          {title}
        </h4>
        {badge && (
          <span className="rotate-[-3deg] border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider text-black">
            {badge}
          </span>
        )}
      </div>
      {children}
    </div>
  );
}

function CodeBlock({ lines, note }: { lines: string[]; note?: string }) {
  const [copied, setCopied] = useState(false);
  const text = lines.join("\n");

  const copy = async () => {
    const ok = await copyToClipboard(text);
    if (ok) {
      setCopied(true);
      toast.copy(text.split("\n")[0].replace(/^#\s*/, "").slice(0, 50));
      setTimeout(() => setCopied(false), 1600);
    }
  };

  return (
    <div className="group/code relative">
      <button
        onClick={copy}
        className="absolute right-2 top-2 z-10 inline-flex h-7 w-7 items-center justify-center border-2 border-[#FBF9F5]/30 bg-[#FBF9F5]/10 text-[#FBF9F5] opacity-0 transition-all hover:bg-[#FF5500] hover:text-black group-hover/code:opacity-100"
        aria-label="Copiar código"
        title="Copiar"
      >
        {copied ? (
          <Check className="h-3.5 w-3.5 text-[#FF5500]" strokeWidth={3} />
        ) : (
          <Copy className="h-3.5 w-3.5" strokeWidth={2.5} />
        )}
      </button>
      <pre className="overflow-x-auto border-2 border-black bg-black p-3 pr-10 font-tech text-[12px] leading-relaxed text-[#FBF9F5]">
        {text}
      </pre>
      {note && (
        <p className="mt-2 font-tech text-[11px] italic text-black/60 ">{note}</p>
      )}
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    const ok = await copyToClipboard(text);
    if (ok) {
      setCopied(true);
      toast.copy("Código copiado");
      setTimeout(() => setCopied(false), 1600);
    }
  };
  return (
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
  );
}

function CliCard({
  cmd,
  args,
  desc,
}: {
  cmd: string;
  args: string;
  desc: string;
}) {
  const [copied, setCopied] = useState(false);
  const fullCmd = `vanta-cli ${cmd} ${args}`;

  const copy = async () => {
    const ok = await copyToClipboard(fullCmd);
    if (ok) {
      setCopied(true);
      toast.copy(fullCmd);
      setTimeout(() => setCopied(false), 1600);
    } else {
      toast.error("No se pudo copiar");
    }
  };

  return (
    <div className="press group flex flex-col gap-2 border-4 border-black bg-[#FBF9F5] p-3   sm:flex-row sm:items-center">
      <div className="flex items-center gap-3 sm:w-56 sm:shrink-0">
        <span className="inline-flex h-7 items-center border-2 border-black bg-black px-2 font-tech text-[10px] font-bold uppercase tracking-wider text-[#FF5500]   ">
          cmd
        </span>
        <code className="font-mono text-sm font-bold text-black ">
          vanta-cli {cmd}
        </code>
      </div>
      <div className="flex-1">
        <div className="flex items-start gap-2">
          <code className="block flex-1 break-all border-l-2 border-black/20 bg-[#F2EDE2] px-2 py-1 font-mono text-[11px] text-black/70   ">
            {args}
          </code>
          <button
            onClick={copy}
            className="inline-flex h-7 w-7 shrink-0 items-center justify-center border-2 border-black bg-[#FBF9F5] text-black transition-all hover:bg-[#FF5500] active:translate-y-[1px]   "
            aria-label={`Copiar comando vanta-cli ${cmd}`}
            title="Copiar comando"
          >
            {copied ? (
              <Check className="h-3.5 w-3.5 text-[#FF5500]" strokeWidth={3} />
            ) : (
              <Copy className="h-3.5 w-3.5" strokeWidth={2.5} />
            )}
          </button>
        </div>
        <p className="mt-1 font-tech text-[11px] text-black/60 ">{desc}</p>
      </div>
    </div>
  );
}
