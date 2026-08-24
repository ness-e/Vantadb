"use client";

import { useEffect, useRef, useState } from "react";
import { Skull, RefreshCw, RotateCcw, Terminal } from "lucide-react";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

type EngineState = "ready" | "crashed" | "recovering";
type LogLevel = "info" | "ok" | "warn" | "error";

interface LogEntry {
  text: string;
  level: LogLevel;
  ts: string;
}

const BOOT_LOGS: LogEntry[] = [
  { ts: "[00:00:01]", text: "System boot initialized", level: "info" },
  { ts: "[00:00:02]", text: "Storage engine opened at path ./agent_memory", level: "info" },
  { ts: "[00:00:03]", text: "Replaying WAL logs... 0 transactions found", level: "info" },
  { ts: "[00:00:04]", text: "Database state: READY", level: "ok" },
];

const CRASH_LOGS: LogEntry[] = [
  { ts: "[00:02:15]", text: "put txn: namespace=memories, key=conv-88", level: "info" },
  { ts: "[00:02:16]", text: "wal: writing page log CRC32C=0xab12de", level: "warn" },
  { ts: "[00:02:17]", text: "!!! CRITICAL FAILURE: PROCESS TERMINATED OUTSIDE CLEAN DISCONNECT !!!", level: "error" },
  { ts: "[00:02:18]", text: "STATUS: OFFLINE", level: "error" },
];

const RECOVER_LOGS: LogEntry[] = [
  { ts: "[00:03:01]", text: "Database reopened. Initializing WAL scan...", level: "warn" },
  { ts: "[00:03:02]", text: "WAL found. Unflushed write at sector index 43", level: "warn" },
  { ts: "[00:03:03]", text: "Checking integrity: verifying CRC32C checksums...", level: "warn" },
  { ts: "[00:03:04]", text: "Checksum 0xab12de OK. Syncing WAL entry 1/1", level: "ok" },
  { ts: "[00:03:05]", text: "WAL sync finished. Rebuilding transient HNSW indexes", level: "ok" },
  { ts: "[00:03:06]", text: "State restored in 0.4ms. 1 transaction recovered.", level: "ok" },
  { ts: "[00:03:07]", text: "STATUS: READY", level: "ok" },
];

export function WalSimulator() {
  const { t, tt } = useLanguage();

  const [logs, setLogs] = useState<LogEntry[]>(BOOT_LOGS);
  const [state, setState] = useState<EngineState>("ready");
  const consoleEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: "smooth", block: "end" });
    }
  }, [logs]);

  const triggerCrash = () => {
    if (state !== "ready") return;
    setState("crashed");
    setLogs((prev) => [...prev, ...CRASH_LOGS]);
  };

  const recoverFromWAL = () => {
    if (state !== "crashed") return;
    setState("recovering");
    setLogs((prev) => [...prev, ...RECOVER_LOGS]);
    window.setTimeout(() => {
      setState("ready");
    }, 1200);
  };

  const reset = () => {
    setState("ready");
    setLogs(BOOT_LOGS);
  };

  const stateMeta: Record<EngineState, { label: string; dot: string; text: string }> = {
    ready: {
      label: tt("walSimulator.state.ready", "READY"),
      dot: "bg-[#FF5500] animate-flicker",
      text: "text-[#FF5500]",
    },
    crashed: {
      label: tt("walSimulator.state.crashed", "CRASHED"),
      dot: "bg-[#FF5500]",
      text: "text-[#FF5500]",
    },
    recovering: {
      label: tt("walSimulator.state.recovering", "RECOVERING"),
      dot: "bg-[#FFB380] animate-pulse",
      text: "text-[#FFB380]",
    },
  };

  const meta = stateMeta[state];

  const levelStyles: Record<LogLevel, string> = {
    info: "text-[#FBF9F5]/70",
    ok: "text-[#FF5500]",
    warn: "text-[#FFB380]",
    error: "text-[#FF5500] font-bold",
  };

  return (
    <section
      aria-label={tt("walSimulator.tagHeader", "Engine · WAL Simulator")}
      className="relative border-b-4 border-black bg-[#F2EDE2]  "
    >
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-30" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("walSimulator.tagHeader", "Engine · WAL Simulator")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-[0.9] sm:text-4xl lg:text-5xl">
                {tt("walSimulator.title", "Crash it. Watch it recover.")}
              </h2>
              <p className="mt-3 max-w-2xl font-tech text-xs text-[#FBF9F5]/70 sm:text-sm">
                {tt("walSimulator.subtitle", "Simulate a process crash and watch the WAL with CRC32C checksums restore state in milliseconds. No data lost — just deterministic recovery.")}
              </p>
            </div>
            <span className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              {tt("walSimulator.tag", "WAL · CRC32C · Crash Recovery")}
            </span>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="border-4 border-black bg-black shadow-[6px_6px_0_0_#000]  ">
            {/* Header — status + actions */}
            <div className="flex flex-col gap-3 border-b-4 border-[#FBF9F5]/20 p-4 sm:flex-row sm:items-center sm:justify-between sm:p-5">
              <div className="flex items-center gap-3">
                <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center border-2 border-[#FF5500] bg-[#FF5500] text-black">
                  <Terminal className="h-5 w-5" strokeWidth={2.5} />
                </span>
                <div className="flex flex-col gap-1">
                  <span className="font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/40">
                    {tt("walSimulator.statusLabel", "STATUS")}
                  </span>
                  <span className={`flex items-center gap-2 font-display text-xl uppercase ${meta.text}`}>
                    <span className={`h-3 w-3 ${meta.dot}`} />
                    {meta.label}
                  </span>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <button
                  type="button"
                  onClick={triggerCrash}
                  disabled={state !== "ready"}
                  className="press-lg inline-flex items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black transition-opacity disabled:cursor-not-allowed disabled:opacity-30"
                >
                  <Skull className="h-3.5 w-3.5" strokeWidth={2.5} />
                  {tt("walSimulator.crashBtn", "Simulate Crash")}
                </button>
                <button
                  type="button"
                  onClick={recoverFromWAL}
                  disabled={state !== "crashed"}
                  className="press-lg inline-flex items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black transition-opacity disabled:cursor-not-allowed disabled:opacity-30"
                >
                  <RefreshCw className="h-3.5 w-3.5" strokeWidth={2.5} />
                  {tt("walSimulator.recoverBtn", "Recover from WAL")}
                </button>
                <button
                  type="button"
                  onClick={reset}
                  className="press-lg inline-flex items-center gap-2 border-4 border-[#FBF9F5]/40 bg-transparent px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider text-[#FBF9F5]/60 transition-colors hover:border-[#FBF9F5] hover:text-[#FBF9F5]"
                >
                  <RotateCcw className="h-3.5 w-3.5" strokeWidth={2.5} />
                  {tt("walSimulator.resetBtn", "Reset")}
                </button>
              </div>
            </div>

            {/* Console — log lines */}
            <div className="max-h-96 overflow-y-auto p-4 font-tech text-xs sm:p-5 sm:text-sm">
              <div className="flex flex-col gap-1.5">
                {logs.map((log, i) => (
                  <div
                    key={`${log.ts}-${log.text}-${i}`}
                    className={`flex items-baseline gap-2 leading-relaxed ${levelStyles[log.level]}`}
                  >
                    <span className="shrink-0 text-[#FBF9F5]/40">{log.ts}</span>
                    <span className={`shrink-0 ${levelStyles[log.level]}`}>›</span>
                    <span className="break-all">{log.text}</span>
                  </div>
                ))}
                {state === "recovering" && (
                  <div className="flex items-baseline gap-2 text-[#FF5500]">
                    <span className="shrink-0 text-[#FBF9F5]/40">[now]</span>
                    <span className="shrink-0 text-[#FF5500]">›</span>
                    <span className="inline-block h-3 w-2 animate-flicker bg-[#FF5500]" />
                  </div>
                )}
                <div ref={consoleEndRef} />
              </div>
            </div>

            {/* Legend */}
            <div className="flex flex-wrap items-center gap-x-5 gap-y-2 border-t-4 border-[#FBF9F5]/20 p-4 font-tech text-[10px] uppercase tracking-wider sm:px-5">
              <span className="flex items-center gap-1.5 text-[#FBF9F5]/40">
                <span className="h-2 w-2 bg-[#FBF9F5]/40" />
                {tt("walSimulator.legend.info", "Info")}
              </span>
              <span className="flex items-center gap-1.5 text-[#FF5500]">
                <span className="h-2 w-2 bg-[#FF5500]" />
                {tt("walSimulator.legend.ok", "OK")}
              </span>
              <span className="flex items-center gap-1.5 text-[#FFB380]">
                <span className="h-2 w-2 bg-[#FFB380]" />
                {tt("walSimulator.legend.warn", "Recovery")}
              </span>
              <span className="flex items-center gap-1.5 text-[#FF5500]">
                <span className="h-2 w-2 bg-[#FF5500]" />
                {tt("walSimulator.legend.error", "Error")}
              </span>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <p className="mt-4 max-w-3xl border-l-4 border-[#FF5500] pl-4 font-tech text-xs leading-relaxed text-black/70 ">
            {tt(
              "walSimulator.footer",
              "Tested with kill -9, power loss, and SIGSEGV. CRC32C detects bit-level corruption — recovery never commits a corrupt entry."
            )}
          </p>
        </Reveal>
      </div>
    </section>
  );
}
