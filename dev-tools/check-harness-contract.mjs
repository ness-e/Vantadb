#!/usr/bin/env node
/**
 * Harness contract gate — H-02..H-07.
 *
 * Source of truth for what is checked: docs/dev/reviews/2026-11-26-harness-hardening.md
 * (§B). Every check here exists because a rule that lives only as prose can be
 * deleted without anything failing. Each check below FAILS when the contract is
 * missing or contradicted, and `--self-test` proves every rule can fail (a gate
 * that cannot fail is not a gate).
 *
 *   node dev-tools/check-harness-contract.mjs              # all checks
 *   node dev-tools/check-harness-contract.mjs --self-test  # rule fixtures
 *   node dev-tools/check-harness-contract.mjs --plan <file> --verbose
 *
 * Exit: 0 = contract holds, 1 = violation(s), 2 = usage/IO error.
 *
 * DELIBERATE CEILING (ponytail): the prompt checks are text contracts, not a
 * parser — a determined author can phrase a contradiction these regexes miss.
 * What they guarantee is the useful half: the canonical wording cannot be
 * deleted, and the known forbidden idioms cannot be reintroduced silently.
 * The runtime enforcement (and the only one that can prove behaviour) remains
 * the mechanical checks: git-commit-isolated.ps1 content verification,
 * check-closure.ps1, and node --test for the campaign parser.
 */
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative, resolve, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const OPENCODE = join(ROOT, ".opencode");
const args = process.argv.slice(2);
const has = (f) => args.includes(f);
const verbose = has("--verbose");
const selfTest = has("--self-test");
const planArg = (() => {
  const i = args.indexOf("--plan");
  return i >= 0 && args[i + 1] ? resolve(ROOT, args[i + 1]) : null;
})();

// ─────────────────────────────────────────────────────────────────────────────
// Pure helpers (fixture-tested by --self-test)
// ─────────────────────────────────────────────────────────────────────────────

/** H-03: a line that teaches the shared-index commit idiom. */
const FORBIDDEN_COMMIT = [
  /\bgit\s+add\s+(-A\b|--all\b|-p\b|--patch\b|\.(\s|`|$))/i,
  /\bgit\s+add\b[^\n]{0,80}\bgit\s+commit\b/i,
  /\bgit\s+commit\s+(-a\b|-am\b|--all\b)/i,
  /\bgit\s+commit\s+-[mF]\b/i,
  /\bgit\s+commit\b(?!\s+--\s)(?!-)/i, // bare commit on a shared index
];
// Prose that DESCRIBES the anti-pattern is allowed (that is how the runbook
// teaches it); an instruction is not.
const NEGATION =
  /\b(nunca|jam[aá]s|never|prohibido|forbidden|bad:|mal:|antipatr[oó]n|no\s+us(es|ar|e)|no\s+ejecut(es|ar|e))/i;
const COMMIT_OK = /git-commit-isolated|GIT_INDEX_FILE/;

export function isForbiddenCommitLine(line) {
  if (/^\s*(#|\/\/)/.test(line)) return false; // comment / example banner
  if (COMMIT_OK.test(line)) return false; // teaches the helper or the manual escape hatch
  // The negation must PRECEDE the forbidden idiom: "nunca `git add && git
  // commit`" describes the anti-pattern, but "git add -p + git commit (nunca
  // `git add -A`)" is an instruction with a subordinate negation — measured
  // 2026-11-26, it slipped a whole-line negation check (fail-open).
  const hits = FORBIDDEN_COMMIT.map((re) => line.search(re)).filter((i) => i >= 0);
  if (!hits.length) return false;
  const firstHit = Math.min(...hits);
  return !NEGATION.test(line.slice(0, firstHit));
}

/**
 * H-05: a plan declares whether the batch is the user's. When it says "todas",
 * a row may not carry DEFER/SKIP without a recorded user answer.
 */
export function loteViolations(planText) {
  const out = [];
  const header = planText.match(/\*\*Regla del lote:\*\*([^\n]*)/i);
  if (!header) {
    return ['plan has no "**Regla del lote:**" declaration (H-05)'];
  }
  const declared = header[1].toLowerCase();
  if (!/todas|triage libre/.test(declared)) {
    out.push('"Regla del lote" must say "todas se ejecutan" or "triage libre"');
  }
  if (!/todas/.test(declared)) return out; // free triage: DEFER/SKIP are legitimate

  const lines = planText.split(/\r?\n/);
  lines.forEach((line, i) => {
    const isGateRow = /-\s*\*\*Gate Result:\*\*[^\n]*(DEFER|SKIP)/i.test(line);
    const isTableRow = /^\s*\|[^\n]*\|\s*(🟡\s*DEFER|❌\s*SKIP|DEFER|SKIP)\s*\|/i.test(line);
    if (!isGateRow && !isTableRow) return;
    const window = lines.slice(i, i + 3).join("\n");
    if (/(Pregunta|Firma|Respuesta del usuario)\s*:/i.test(window)) return;
    out.push(
      `row defers/skips the user's batch without a recorded answer (line ${i + 1}): ${line
        .trim()
        .slice(0, 100)}`,
    );
  });
  return out;
}

/** H-02: a claim without a re-runnable measurement. */
const CLAIM = /\d|\b(ning[uú]n|ninguna|todos|todas|solo|sólo|nunca)\b/i;
const HAS_MEASURE = /Medici[oó]n\s*:|medido\b/i;

export function claimViolations(addedRowLines) {
  const out = [];
  for (const { file, line, text } of addedRowLines) {
    if (!CLAIM.test(text)) continue;
    if (HAS_MEASURE.test(text)) continue;
    // A row ID / date / effort cell is metadata, not a premise: only rows with
    // prose (the "Hallazgo"/"Descripción"-like cells) are judged.
    if (!/\|\s*[^|]{25,}\|/.test(text)) continue;
    out.push(`${file}:${line}: claim without "Medición:" — ${text.trim().slice(0, 100)}`);
  }
  return out;
}

// ─────────────────────────────────────────────────────────────────────────────
// Plan table parsing (H-06 wave conflicts)
// ─────────────────────────────────────────────────────────────────────────────

const normToken = (t) => t.replace(/`/g, "").replace(/^\.\//, "").trim();
const isPathish = (t) => /\/|\.(rs|yml|yaml|toml|md|py|ts|json)$/.test(t);

function resourcesOf(cell) {
  const ticked = [...cell.matchAll(/`([^`]+)`/g)].map((m) => m[1]);
  const bare = ticked.length ? [] : cell.split(/[+,·]/);
  return [...ticked, ...bare]
    .map(normToken)
    .map((t) => t.split(/\s+/)[0])
    .filter((t) => t && t !== "-" && isPathish(t));
}

function overlaps(a, b) {
  const strip = (s) => s.replace(/\/\*\*?$/, "").replace(/\/\*$/, "");
  const A = strip(a);
  const B = strip(b);
  return A === B || A.startsWith(B + "/") || B.startsWith(A + "/");
}

export function parsePlanRows(md) {
  const lines = md.split(/\r?\n/);
  const rows = [];
  for (let i = 0; i < lines.length; i++) {
    if (!/^\s*\|/.test(lines[i])) continue;
    const header = lines[i]
      .split("|")
      .slice(1, -1)
      .map((c) => c.trim());
    const iId = header.findIndex((h) => /^id$/i.test(h));
    const iWave = header.findIndex((h) => /wave/i.test(h));
    const iRes = header.findIndex((h) => /ficheros|zona/i.test(h));
    const iPre = header.findIndex((h) => /prerequisito/i.test(h));
    const iCarril = header.findIndex((h) => /carril/i.test(h));
    if (iId < 0 || iWave < 0 || iRes < 0) continue;
    for (let j = i + 1; j < lines.length && /^\s*\|/.test(lines[j]); j++) {
      const cells = lines[j]
        .split("|")
        .slice(1, -1)
        .map((c) => c.trim());
      if (!cells.length || /^[-: ]+$/.test(cells[0])) continue;
      const id = (cells[iId] || "").replace(/`/g, "").trim();
      if (!id) continue;
      rows.push({
        id,
        wave: (cells[iWave] || "").replace(/[*`]/g, "").trim(),
        resources: resourcesOf(cells[iRes] || ""),
        prereq: iPre >= 0 ? cells[iPre] || "" : "",
        carril: iCarril >= 0 ? (cells[iCarril] || "").replace(/[*`]/g, "").trim() : "",
      });
    }
    i = i + rows.length ? i : i; // keep scanning after the table
  }
  return rows;
}

export function waveConflicts(rows) {
  const out = [];
  const byWave = new Map();
  for (const r of rows) {
    if (!byWave.has(r.wave)) byWave.set(r.wave, []);
    byWave.get(r.wave).push(r);
  }
  for (const [wave, list] of byWave) {
    for (let a = 0; a < list.length; a++) {
      for (let b = a + 1; b < list.length; b++) {
        const A = list[a];
        const B = list[b];
        const shared = [];
        for (const ra of A.resources)
          for (const rb of B.resources) if (overlaps(ra, rb)) shared.push(`${ra} ~ ${rb}`);
        if (!shared.length) continue;
        const serialized =
          new RegExp(`\\b${B.id}\\b`).test(A.prereq) || new RegExp(`\\b${A.id}\\b`).test(B.prereq);
        if (serialized) continue;
        out.push(
          `wave ${wave}: ${A.id} and ${B.id} share an exclusive resource (${shared[0]}) with no declared prerequisite`,
        );
      }
    }
  }
  return out;
}

export function rustLaneCensus(rows) {
  const byWave = new Map();
  for (const r of rows) {
    if (!/^R|R$|^A?R$/.test(r.carril.replace(/\s/g, ""))) continue;
    byWave.set(r.wave, (byWave.get(r.wave) || 0) + 1);
  }
  return [...byWave]
    .filter(([, n]) => n > 2)
    .map(([w, n]) => `wave ${w}: ${n} rust lanes declared (compiler quota is 2)`);
}

// ─────────────────────────────────────────────────────────────────────────────
// Filesystem helpers
// ─────────────────────────────────────────────────────────────────────────────

function walk(dir, acc = []) {
  let entries;
  try {
    entries = readdirSync(dir);
  } catch {
    return acc;
  }
  for (const e of entries) {
    if (e === "node_modules" || e === ".git") continue;
    const p = join(dir, e);
    if (statSync(p).isDirectory()) walk(p, acc);
    else if (/\.(md|mjs|ps1)$/.test(e)) acc.push(p);
  }
  return acc;
}

const read = (p) => readFileSync(p, "utf8");
const rel = (p) => relative(ROOT, p).replace(/\\/g, "/");
const exists = (p) => {
  try {
    statSync(p);
    return true;
  } catch {
    return false;
  }
};

function activePlans() {
  if (planArg) return [planArg];
  const dir = join(ROOT, "docs/dev/plans");
  let files = [];
  try {
    files = readdirSync(dir).filter((f) => f.endsWith(".md"));
  } catch {
    return [];
  }
  return files.map((f) => join(dir, f)).filter((p) => /^status:\s*active\b/im.test(read(p)));
}

function addedDiffRows() {
  let diff = "";
  try {
    diff = execFileSync("git", ["diff", "-U0", "--", "docs/dev/Backlog.md", "docs/dev/plans"], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"], // CRLF autocrlf warnings are not findings
    });
  } catch {
    return [];
  }
  const out = [];
  let file = "";
  let line = 0;
  for (const raw of diff.split(/\r?\n/)) {
    if (raw.startsWith("+++ b/")) {
      file = raw.slice(6);
      continue;
    }
    const hunk = raw.match(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
    if (hunk) {
      line = Number(hunk[1]);
      continue;
    }
    if (raw.startsWith("+") && !raw.startsWith("+++")) {
      out.push({ file, line, text: raw.slice(1) });
      line++;
    }
  }
  return out;
}

// ─────────────────────────────────────────────────────────────────────────────
// Checks
// ─────────────────────────────────────────────────────────────────────────────

function checkCommitPath() {
  const bad = [];
  const scanDirs = [
    join(OPENCODE, "task-system/prompts"),
    join(OPENCODE, "agents"),
    join(OPENCODE, "commands"),
  ];
  for (const dir of scanDirs) {
    for (const file of walk(dir)) {
      if (!file.endsWith(".md")) continue;
      read(file)
        .split(/\r?\n/)
        .forEach((line, i) => {
          if (isForbiddenCommitLine(line))
            bad.push(`${rel(file)}:${i + 1}: ${line.trim().slice(0, 120)}`);
        });
    }
  }
  const prompts = walk(join(OPENCODE, "task-system/prompts")).filter((f) => f.endsWith(".md"));
  const cites = prompts.filter((f) => /docs\/dev\/workflow\/RULES\.md/.test(read(f)));
  if (!cites.length) {
    bad.push(
      "no prompt cites docs/dev/workflow/RULES.md (§11 runbook) — the helper can be orphaned again",
    );
  }
  if (verbose)
    console.log(
      `  commit-path: ${prompts.length} prompts scanned, runbook cited by ${cites.length}`,
    );
  return bad;
}

function checkPlanLote() {
  const bad = [];
  const template = join(OPENCODE, "task-system/prompts/plan.md");
  if (!exists(template) || !/Regla del lote/.test(read(template))) {
    bad.push(
      '.opencode/task-system/prompts/plan.md: template lost the "Regla del lote" field (H-05)',
    );
  }
  for (const plan of activePlans()) {
    for (const v of loteViolations(read(plan))) bad.push(`${rel(plan)}: ${v}`);
  }
  return bad;
}

function checkMedicion() {
  const bad = [];
  const markers = [
    [
      "task-system/prompts/plan.md",
      /Medici[oó]n/i,
      'plan template lost the "Medición:" field (H-02)',
    ],
    [
      "task-system/prompts/task.md",
      /Medici[oó]n/i,
      'task template lost the "Medición:" field (H-02)',
    ],
    [
      "task-system/prompts/findings.md",
      /Medici[oó]n/i,
      'FIND-* ticket template lost the "Medición:" field (H-02)',
    ],
    [
      "task-system/prompts/pipeline-full.md",
      /re-?medir|re-?medici[oó]n|re-?medí/i,
      "§Cierre no longer re-measures the premise (H-02)",
    ],
    [
      "task-system/prompts/pipeline-full.md",
      /MEDICION:/,
      "RESULTADO block lost the MEDICION: field (H-02)",
    ],
  ];
  for (const [f, re, msg] of markers) {
    const p = join(OPENCODE, f);
    if (!exists(p) || !re.test(read(p))) bad.push(`${f}: ${msg}`);
  }
  for (const v of claimViolations(addedDiffRows())) bad.push(v);
  return bad;
}

function checkPremise() {
  const bad = [];
  const taskPrompt = join(OPENCODE, "task-system/prompts/task.md");
  if (
    !exists(taskPrompt) ||
    !/Alcance obligatorio del review[\s\S]{0,600}?premisa/i.test(read(taskPrompt))
  ) {
    bad.push("task.md: review scope no longer includes the premise (H-07)");
  }
  const full = join(OPENCODE, "task-system/prompts/pipeline-full.md");
  const fullText = exists(full) ? read(full) : "";
  if (!/PREMISA:/.test(fullText))
    bad.push("pipeline-full.md: RESULTADO block lost the PREMISA: field (H-07)");
  const run = join(OPENCODE, "task-system/prompts/pipeline-run.md");
  if (
    !exists(run) ||
    !/PREMISA[^\n]{0,120}(falta|ausente|rechaz|SIN-FORMATO)|(falta|ausente|rechaz)[^\n]{0,120}PREMISA/i.test(
      read(run),
    )
  ) {
    bad.push(
      "pipeline-run.md: the orchestrator no longer rejects ✅ COMPLETO without PREMISA (H-07)",
    );
  }
  return bad;
}

function checkWaveQuotas() {
  const bad = [];
  const run = join(OPENCODE, "task-system/prompts/pipeline-run.md");
  const runText = exists(run) ? read(run) : "";
  if (!/CUPOS POR RECURSO/.test(runText))
    bad.push('pipeline-run.md: lost the "CUPOS POR RECURSO" block (H-06)');
  for (const [f, text] of [
    ["task-system/prompts/pipeline-run.md", runText],
    [
      "commands/pipeline.md",
      exists(join(OPENCODE, "commands/pipeline.md"))
        ? read(join(OPENCODE, "commands/pipeline.md"))
        : "",
    ],
  ]) {
    if (/MAX_CONCURRENT\s*=\s*min\(3/.test(text))
      bad.push(`${f}: uniform MAX_CONCURRENT=min(3,…) is back — the quota is per resource (H-06)`);
  }
  for (const plan of activePlans()) {
    const rows = parsePlanRows(read(plan));
    if (!rows.length) {
      if (verbose) console.log(`  wave-quotas: ${rel(plan)} has no wave table — skipped`);
      continue;
    }
    for (const v of waveConflicts(rows)) bad.push(`${rel(plan)}: ${v}`);
    if (verbose)
      for (const n of rustLaneCensus(rows))
        console.log(`  wave-quotas advisory — ${rel(plan)}: ${n}`);
  }
  return bad;
}

const CHECKS = [
  ["commit-path (H-03)", checkCommitPath],
  ["plan-lote (H-05)", checkPlanLote],
  ["medicion (H-02)", checkMedicion],
  ["premise (H-07)", checkPremise],
  ["wave-quotas (H-06)", checkWaveQuotas],
];

// ─────────────────────────────────────────────────────────────────────────────
// Self-test: proves each rule can fail
// ─────────────────────────────────────────────────────────────────────────────

function runSelfTest() {
  const cases = [];
  const ok = (name, cond) => cases.push([name, !!cond]);

  ok(
    'forbidden: bare "git add . && git commit"',
    isForbiddenCommitLine('git add . && git commit -m "x"'),
  );
  ok(
    'forbidden: "git add -p + git commit"',
    isForbiddenCommitLine("7. git add -p + git commit (solo archivos tocados)"),
  );
  ok('forbidden: "git commit -a -m"', isForbiddenCommitLine('git commit -a -m "fix"'));
  ok('forbidden: bare "git commit"', isForbiddenCommitLine("- despues: `git commit`"));
  ok(
    "allowed: helper invocation",
    !isForbiddenCommitLine("& ./dev-tools/git-commit-isolated.ps1 -Message x -Paths y"),
  );
  ok(
    "allowed: prose that forbids it",
    !isForbiddenCommitLine("nunca `git add && git commit` a secas"),
  );
  ok(
    "forbidden: instruction with a subordinate negation",
    isForbiddenCommitLine(
      "7. git add -p + git commit (solo archivos tocados — nunca `git add -A`)",
    ),
  );

  const noLote = loteViolations("# plan\n\n| # | ID |\n");
  ok("lote: missing declaration fails", noLote.length === 1);
  const badLote = loteViolations(
    "**Regla del lote:** todas se ejecutan\n- **Gate Result:** 🟡 DEFER\n",
  );
  ok("lote: DEFER on a user batch fails", badLote.length === 1);
  const signed = loteViolations(
    "**Regla del lote:** todas se ejecutan\n- **Gate Result:** 🟡 DEFER\n  Pregunta: ¿difiero FIND-1? → Respuesta: sí\n",
  );
  ok("lote: signed DEFER passes", signed.length === 0);
  const free = loteViolations("**Regla del lote:** triage libre\n- **Gate Result:** 🟡 DEFER\n");
  ok("lote: free triage allows DEFER", free.length === 0);

  const claim = claimViolations([
    {
      file: "Backlog.md",
      line: 9,
      text: "| `FIND-9` | x | **92 unwraps en prod** en el motor completo | `src/a.rs:1` |",
    },
  ]);
  ok("medicion: numeric claim without Medición fails", claim.length === 1);
  const measured = claimViolations([
    {
      file: "Backlog.md",
      line: 9,
      text: "| `FIND-9` | x | **92 unwraps en prod** en el motor completo | Medición: `rg -c unwrap` → 92 |",
    },
  ]);
  ok("medicion: measured claim passes", measured.length === 0);

  const W = (id, res, pre = "") => ({
    id,
    wave: "W1",
    resources: resourcesOf(res),
    prereq: pre,
    carril: "R",
  });
  const conflict = waveConflicts([
    W("A-1", "`.github/workflows/ci-rust.yml`"),
    W("A-2", "`.github/workflows/ci-rust.yml`"),
  ]);
  ok("waves: same workflow, no prereq fails", conflict.length === 1);
  const serialized = waveConflicts([
    W("A-1", "`.github/workflows/ci-rust.yml`"),
    W("A-2", "`.github/workflows/ci-rust.yml`", "tras A-1 (mismo fichero)"),
  ]);
  ok("waves: same workflow with prereq passes", serialized.length === 0);
  const sweep = waveConflicts([W("A-1", "`src/**`"), W("A-2", "`src/storage/engine/delete.rs`")]);
  ok("waves: src sweep + src writer fails", sweep.length === 1);

  ok(
    "premise: review scope regex",
    /Alcance obligatorio del review[\s\S]{0,600}?premisa/i.test(
      "**Alcance obligatorio del review:**\n0. **La premisa** — ¿se sostiene?",
    ),
  );
  ok(
    "premise: missing premise detected",
    !/Alcance obligatorio del review[\s\S]{0,600}?premisa/i.test(
      "**Alcance obligatorio del review:**\n1. Enfoque",
    ),
  );

  let failed = 0;
  for (const [name, pass] of cases) {
    console.log(`${pass ? "ok  " : "FAIL"}  ${name}`);
    if (!pass) failed++;
  }
  console.log(`\n${cases.length - failed}/${cases.length} self-test fixtures passed`);
  return failed;
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

if (selfTest) {
  process.exit(runSelfTest() ? 1 : 0);
}

let violations = 0;
for (const [name, fn] of CHECKS) {
  let bad;
  try {
    bad = fn();
  } catch (e) {
    console.log(`ERROR ${name}: ${e.message}`);
    process.exit(2);
  }
  if (bad.length) {
    violations += bad.length;
    console.log(`FAIL ${name}`);
    for (const b of bad) console.log(`  - ${b}`);
  } else {
    console.log(`ok   ${name}`);
  }
}

console.log(
  violations
    ? `\n${violations} harness-contract violation(s) — see docs/dev/reviews/2026-11-26-harness-hardening.md`
    : "\nharness contract holds (5 checks)",
);
process.exit(violations ? 1 : 0);
