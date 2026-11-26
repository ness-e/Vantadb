#!/usr/bin/env node
// FIND-706 repro: sustained MCP writes over stdio, per-round latency.
//
// The original 2026-11-23 repro was an ad-hoc `.tasksystem`-style script that
// was never committed (the avance entry only quotes its numbers). This is the
// same method, versioned: isolated DB, N rounds x P `memory_put` calls, wall
// clock per call and per round, no client timeout so a round that crosses the
// 60 s MCP-client budget is *observed* instead of killing the child.
//
// Usage (from the repo root — the relative ONNX model path resolves against cwd):
//   node dev-tools/mcp-latency-repro.mjs --bin "C:/Users/me/.cargo/bin/vantadb-server.exe"
//   node dev-tools/mcp-latency-repro.mjs --bin target/debug/vantadb-server.exe --rounds 10 --puts 3
//
// Flags: --rounds N (10) | --puts P (3) | --bin PATH | --json OUT | --keep
//        --stderr   echo the child's stderr tail

import { spawn } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i === -1 ? fallback : args[i + 1];
};
const has = (name) => args.includes(`--${name}`);

const roundCount = Number(flag("rounds", "10"));
const putsPerRound = Number(flag("puts", "3"));
const bin = resolve(flag("bin", "target/debug/vantadb-server.exe"));
const jsonOut = flag("json", null);
const keepDb = has("keep");
const showStderr = has("stderr");
const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const storage = mkdtempSync(join(tmpdir(), "vanta-f706-"));
const child = spawn(bin, ["--mcp"], {
  cwd: repoRoot,
  stdio: ["pipe", "pipe", "pipe"],
  env: {
    ...process.env,
    VANTADB_STORAGE_PATH: storage,
    VANTADB_EMBEDDING_PROVIDER: process.env.VANTADB_EMBEDDING_PROVIDER ?? "local",
    VANTADB_QUIET: "1",
  },
});

let stderrTail = [];
child.stderr.on("data", (b) => {
  const text = b.toString();
  if (showStderr) process.stderr.write(text);
  stderrTail = stderrTail.concat(text.split("\n").filter(Boolean)).slice(-40);
});
child.on("exit", (code, signal) => {
  if (code !== 0 && code !== null) {
    console.error(`\n[child exited code=${code} signal=${signal}]`);
  }
});

let nextId = 1;
const pending = new Map();
let buf = "";
child.stdout.on("data", (chunk) => {
  buf += chunk.toString();
  let nl;
  while ((nl = buf.indexOf("\n")) !== -1) {
    const line = buf.slice(0, nl).trim();
    buf = buf.slice(nl + 1);
    if (!line) continue;
    let msg;
    try {
      msg = JSON.parse(line);
    } catch {
      continue; // non-JSON chatter on stdout
    }
    const waiter = pending.get(msg.id);
    if (waiter) {
      pending.delete(msg.id);
      waiter(msg);
    }
  }
});

function rpc(method, params, timeoutMs = 0) {
  const id = nextId++;
  const payload = { jsonrpc: "2.0", id, method, params };
  return new Promise((res, rej) => {
    const timer = timeoutMs
      ? setTimeout(() => {
          pending.delete(id);
          rej(new Error(`${method} timed out after ${timeoutMs} ms`));
        }, timeoutMs)
      : null;
    pending.set(id, (msg) => {
      if (timer) clearTimeout(timer);
      if (msg.error) rej(new Error(`${method}: ${JSON.stringify(msg.error)}`));
      else res(msg.result);
    });
    child.stdin.write(JSON.stringify(payload) + "\n");
  });
}

const median = (xs) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
};
const fmt = (ms) => (ms / 1000).toFixed(1).padStart(7);

async function main() {
  const t0 = performance.now();
  await rpc(
    "initialize",
    {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: { name: "find-706-repro", version: "1" },
    },
    120_000,
  );
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) + "\n");
  const initMs = performance.now() - t0;

  const rounds = [];
  const calls = [];
  for (let r = 1; r <= roundCount; r++) {
    const roundStart = performance.now();
    // Sequential on purpose: each call is timed, so parallelising them would
    // measure contention instead of the server's per-write latency.
    for (let p = 1; p <= putsPerRound; p++) {
      const key = `f706-r${r}-p${p}`;
      const payload =
        `FIND-706 sustained write probe round ${r} put ${p}. ` +
        "Embedding latency under steady MCP writes must stay flat, not grow per round. ".repeat(2);
      const start = performance.now();
      await rpc(
        "tools/call",
        {
          name: "memory_put",
          arguments: {
            namespace: "f706",
            key,
            payload,
            metadata: { round: r, put: p, probe: "find-706" },
          },
        },
        300_000, // observe, never kill: the point is to see a round cross 60 s
      );
      const callMs = performance.now() - start;
      calls.push(callMs);
      process.stdout.write(`  round ${String(r).padStart(2)} put ${p}: ${fmt(callMs)} s\n`);
    }
    rounds.push(performance.now() - roundStart);
  }

  console.log(`\nFIND-706 repro — ${putsPerRound} memory_put/round, ${roundCount} rounds`);
  console.log(`binary: ${bin}`);
  console.log(`db: ${storage}`);
  console.log(`initialize: ${fmt(initMs)} s\n`);
  console.log("| round | round total | cumulative |");
  console.log("|------:|------------:|-----------:|");
  let cumulative = 0;
  rounds.forEach((ms, i) => {
    cumulative += ms;
    console.log(`| ${i + 1} | ${fmt(ms)} s | ${fmt(cumulative)} s |`);
  });
  console.log(
    `\nper call: min ${fmt(Math.min(...calls))} s · median ${fmt(median(calls))} s · max ${fmt(
      Math.max(...calls),
    )} s (n=${calls.length})`,
  );
  const last = rounds[rounds.length - 1];
  console.log(
    `round ${rounds.length}: ${fmt(last)} s → client timeout (60 s): ${
      last < 60_000 ? "OK" : "EXCEEDED"
    }`,
  );

  if (jsonOut) {
    writeFileSync(
      jsonOut,
      JSON.stringify(
        {
          binary: bin,
          putsPerRound,
          initializeMs: initMs,
          roundsMs: rounds,
          callsMs: calls,
          storage,
        },
        null,
        2,
      ),
    );
    console.log(`json: ${jsonOut}`);
  }

  if (stderrTail.length) {
    console.log("\nstderr tail:");
    for (const line of stderrTail.slice(-12)) console.log(`  ${line}`);
  }
}

main()
  .catch((e) => {
    console.error(`\nFAILED: ${e.message}`);
    process.exitCode = 1;
  })
  .finally(() => {
    child.stdin.end();
    setTimeout(() => {
      if (child.exitCode === null) child.kill();
      if (!keepDb) rmSync(storage, { recursive: true, force: true });
    }, 1500);
  });
