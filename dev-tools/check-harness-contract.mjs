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

const args = process.argv.slice(2);
const ROOT = (() => {
  const i = args.indexOf("--root");
  return i >= 0 && args[i + 1]
    ? resolve(args[i + 1])
    : resolve(dirname(fileURLToPath(import.meta.url)), "..");
})();
const OPENCODE = join(ROOT, ".opencode");
const has = (f) => args.includes(f);
const verbose = has("--verbose");
const selfTest = has("--self-test");
const rowsOnly = has("--rows"); // M2: cheap gate a lane can run when it CREATES a FIND row
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
    const window = lines.slice(i, i + 5).join("\n");
    if (/(Pregunta|Firma|Respuesta del usuario)\s*:/i.test(window)) return;
    out.push(
      `row defers/skips the user's batch without a recorded answer (line ${i + 1}): ${line
        .trim()
        .slice(0, 100)}`,
    );
  });
  return out;
}

/** H-02: a claim without a re-runnable measurement.
 *
 * Judged by COLUMN INDEX of the row's table header, never by cell shape.
 * Round 1 tested the whole line (every ID/effort/wave digit became a "claim");
 * round 2 replaced that with an exclusion list + "two tokens = prose", and
 * `🟡 1d` / `Origen: FIND-764#T-2 commit 9d5f1e7` walked straight through it.
 * A shape list is an eternal patch: the claim lives in `Hallazgo`/`Descripción`
 * by spec (`findings.md`), and every other column is never judged.
 *
 * The inverse hole is closed by construction: inside a claim column ANY digit or
 * universal word is a claim, so `**92**` (one token) is not invisible.
 */
const CLAIM = /\d|\b(ning[uú]n|ninguna|todos|todas|solo|sólo|nunca)\b/i;
const HAS_MEASURE = /Medici[oó]n\s*:|medido\b/i;
const CLAIM_COLUMNS = [/^hallazgo$/, /^descripcion$/];

const normHeader = (c) =>
  c
    .replace(/[*`]/g, "")
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .trim()
    .toLowerCase();

const isSeparator = (s) => /^\s*\|[\s:|-]+\|\s*$/.test(s) && s.includes("-");

/**
 * Column indices to judge for the row at 1-based `line`, read from the table
 * header that governs it. `[]` = table with no claim column (plan wave tables);
 * `null` = the line is not in a table.
 */
/**
 * Normalized cells of the table header that governs 1-based `line`.
 * `null` = the line is not inside a table. (N1: the old message indexed `line-2`,
 * the separator, and printed `|---|---|` as "header".)
 */
export function governingHeader(text, line) {
  const lines = text.split(/\r?\n/);
  for (let i = Math.min(line - 2, lines.length - 1); i >= 0; i--) {
    const l = lines[i];
    if (!/^\s*\|/.test(l)) return null;
    if (isSeparator(l) && i > 0 && /^\s*\|/.test(lines[i - 1])) {
      return lines[i - 1].split("|").slice(1, -1).map(normHeader);
    }
  }
  return null;
}

/**
 * Column indices to judge for the row at 1-based `line`: the `Hallazgo` /
 * `Descripción` columns of its governing header. `[]` = table with no claim
 * column; `null` = the line is not in a table.
 */
export function claimCellIndices(text, line) {
  const header = governingHeader(text, line);
  if (!header) return null;
  const idx = [];
  header.forEach((h, k) => {
    if (CLAIM_COLUMNS.some((re) => re.test(h))) idx.push(k);
  });
  return idx;
}

/**
 * H-02 (review round 4, BLOCKING): a table whose header is `Finding/Description`
 * or `Problema/Detalle` used to make the gate SKIP the row — fail-open. Two
 * lines close it:
 *   (a) the canonical header must exist in the two files that govern claim rows;
 *   (b) a row ADDED to the Backlog under a table without a claim column is a
 *       violation, not a skip (plan wave tables stay exempt: they are not claim
 *       rows and are added by every plan).
 */
const CANON_HEADER =
  /ID\s*\|\s*Severidad\s*\|\s*Hallazgo\s*\|\s*Archivo:l[ií]nea\s*\|\s*Esfuerzo\s*\|\s*Prioridad\s*\|\s*Estado\s*\|\s*Descripci[oó]n\s*\|/;

const isBacklogFile = (f) => /(^|\/)Backlog\.md$/.test(f.replace(/\\/g, "/"));
const PROSE_HINT =
  /hallazgo|finding|problema|brecha|tarea|descripcion|description|detalle|nota|impacto|accion/;

export function claimViolations(addedRowLines, fileText = (f) => read(join(ROOT, f))) {
  const out = [];
  const cache = new Map();
  for (const { file, line, text } of addedRowLines) {
    if (!/^\s*\|/.test(text)) continue; // only table rows
    if (HAS_MEASURE.test(text)) continue;
    if (!cache.has(file)) {
      try {
        cache.set(file, fileText(file));
      } catch {
        cache.set(file, null); // file gone / unreadable: nothing to judge
      }
    }
    const body = cache.get(file);
    if (body === null) continue;
    const header = governingHeader(body, line);
    const idx = claimCellIndices(body, line);
    const cells = text.split("|").slice(1, -1);
    if (!idx || !idx.length) {
      // Round 5 (R1): only a data table (has an `ID` column) is judged, and only
      // its prose columns — the Backlog's dashboards (`Phase / Dimensión | Items
      // Totales | …`) and a legit row with no claim are not violations. A renamed
      // claim header (`Finding/Description`, `Problema/Detalle`) still fails.
      if (!isBacklogFile(file) || !header || !header.includes("id")) continue;
      const proseCols = header.map((h, k) => (PROSE_HINT.test(h) ? k : -1)).filter((k) => k >= 0);
      const claimed = proseCols.map((k) => cells[k] || "").filter((c) => CLAIM.test(c));
      if (!claimed.length) continue;
      out.push(
        `${file}:${line}: claim under a non-canonical header (${header.join(" | ")}) — ${claimed[0]
          .trim()
          .slice(0, 80)}`,
      );
      continue;
    }
    const claimed = idx.map((i) => cells[i] || "").filter((c) => CLAIM.test(c));
    if (!claimed.length) continue;
    out.push(`${file}:${line}: claim without "Medición:" — ${claimed[0].trim().slice(0, 90)}`);
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
  // Trailing slashes are not a different resource: `src/` and `src` are the same.
  const strip = (s) =>
    s
      .replace(/\/+$/, "")
      .replace(/\/\*\*?$/, "")
      .replace(/\/\*$/, "");
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
    // Top level only: plans/archive/** is history, not a queue (B3 pre-mortem).
    files = readdirSync(dir).filter((f) => f.endsWith(".md"));
  } catch {
    return [];
  }
  return files.map((f) => join(dir, f)).filter((p) => isActivePlan(read(p)));
}

/**
 * H-05 (review round 1, B3): a plan WITHOUT `status:` is active. Requiring the
 * field made the gate blind to every plan the template produced (the template
 * emitted no frontmatter) — the same "ignore `status:`" drift the audit
 * condemned in `findPlanFile`. Only an explicit terminal status excludes a plan.
 */
export function isActivePlan(text) {
  const m = text.match(/^status:\s*([^\s#]+)/im);
  if (!m) return true;
  return !/^(superseded|archived|done|completed|closed)$/i.test(m[1]);
}

function addedDiffRows() {
  let diff = "";
  try {
    // -HEAD: staged and unstaged both count. `git diff` alone (worktree vs
    // index) went blind the moment a claim row was staged (review round 1, M1).
    diff = execFileSync(
      "git",
      ["diff", "-U0", "HEAD", "--", "docs/dev/Backlog.md", "docs/dev/plans"],
      {
        cwd: ROOT,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"], // CRLF autocrlf warnings are not findings
      },
    );
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
  // `.opencode/AGENTS.md` is the FIRST flow every agent reads (`:8` said
  // "`git commit` local"): leaving it out of the scan made the gate blind to the
  // most-read contradiction (review round 1, B2).
  const scanFiles = [join(OPENCODE, "AGENTS.md")];
  for (const dir of [
    join(OPENCODE, "task-system/prompts"),
    join(OPENCODE, "agents"),
    join(OPENCODE, "commands"),
  ]) {
    scanFiles.push(...walk(dir).filter((f) => f.endsWith(".md")));
  }
  for (const file of scanFiles) {
    if (!exists(file)) continue;
    read(file)
      .split(/\r?\n/)
      .forEach((line, i) => {
        if (isForbiddenCommitLine(line))
          bad.push(`${rel(file)}:${i + 1}: ${line.trim().slice(0, 120)}`);
      });
  }
  const prompts = walk(join(OPENCODE, "task-system/prompts")).filter((f) => f.endsWith(".md"));
  const cites = prompts.filter((f) => /docs\/dev\/workflow\/RULES\.md/.test(read(f)));
  if (!cites.length) {
    bad.push(
      "no prompt cites docs/dev/workflow/RULES.md (§11 runbook) — the helper can be orphaned again",
    );
  }
  // Citing the path is not the same as the section existing (L4).
  const runbook = join(ROOT, "docs/dev/workflow/RULES.md");
  if (!exists(runbook) || !/^#{2,3}\s*11\b/m.test(read(runbook))) {
    bad.push("docs/dev/workflow/RULES.md: §11 (the commit runbook the prompts cite) is missing");
  }
  if (verbose)
    console.log(
      `  commit-path: ${scanFiles.length} files scanned, runbook cited by ${cites.length}`,
    );
  return bad;
}

/**
 * FIND-776 (H-03 / F4): the commit policy has ONE owner. Four documents used to
 * disagree about who commits, the prompt won in practice (166 commits in 24h),
 * and nothing detected the contradiction. This is that detector: the canonical
 * statement must exist, and no document may restate a conflicting authority.
 *
 * Round 2 declared FIND-776 closed on an `rg` that missed `vanta-lead.md:16`
 * ("único que prepara+ejecuta commit") and `:20` ("git mutating (commit/…)") —
 * the claim was refuted with these two lines. A phrase list is the honest
 * mechanism here: the forms below are the ones that actually appeared.
 */
const AUTHORITY_CANON = /commit local de cierre[^\n]{0,120}lo hace el ejecutor/i;
const AUTHORITY_CONTRADICTION = [
  /[uú]nico\s+que\s+prepara\s*\+\s*ejecuta\s+commit/i,
  /[uú]nico rol con git mutating\s*\(commit/i,
  /only role that (runs|makes) git commit/i,
  /\bsolo (el )?lead (hace |ejecuta )?commit/i,
  /\bthe lead commits\b/i,
  /\b(lead|vanta-lead)\s+commitea\b/i,
  /\bcommit lo ejecuta (el )?lead\b/i,
];

export function authorityViolations(line) {
  return AUTHORITY_CONTRADICTION.some((re) => re.test(line));
}

/**
 * Repository integrity — measured 2026-11-26, incident 72643d9df: a commit was
 * created with the EMPTY tree (2,365 paths deleted) while the working tree kept
 * every one of them, and the commit helper reported only "git commit failed".
 * The dangerous direction is exactly this: HEAD says "gone", the disk says
 * "here" — any lane's bare `git commit` then bakes the deletions forever.
 *
 * Two one-line signatures, both false-positive-safe:
 *   - HEAD's tree is the empty tree;
 *   - the last commit deleted N>=50 paths that are STILL on disk (a real
 *     deletion also removes them from the worktree).
 */
function checkRepoIntegrity() {
  const bad = [];
  const EMPTY = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
  const ref = (() => {
    const i = args.indexOf("--ref");
    return i >= 0 && args[i + 1] ? args[i + 1] : "HEAD";
  })();
  const git = (a) =>
    execFileSync("git", a, {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  let tree;
  try {
    tree = git(["rev-parse", `${ref}^{tree}`]);
  } catch {
    return []; // not a git repo / no commits yet: not this check's business
  }
  if (tree === EMPTY) bad.push(`${ref}'s tree is EMPTY — the repo was committed away`);
  let deleted = [];
  try {
    deleted = git(["show", "--diff-filter=D", "--name-only", "--format=", ref])
      .split("\n")
      .filter(Boolean);
  } catch {
    deleted = [];
  }
  const stillOnDisk = deleted.filter((p) => exists(join(ROOT, p)));
  if (stillOnDisk.length >= 50) {
    bad.push(
      `${ref} deleted ${stillOnDisk.length} paths that are STILL in the working tree (HEAD/worktree inversion) — e.g. ${stillOnDisk[0]}`,
    );
  }
  if (verbose) console.log(`  repo-integrity: ${ref} tree ok, ${deleted.length} deletions checked`);
  return bad;
}

function checkCommitAuthority() {
  const bad = [];
  const files = [
    join(OPENCODE, "AGENTS.md"),
    ...walk(join(OPENCODE, "agents")),
    ...walk(join(OPENCODE, "commands")),
    ...walk(join(OPENCODE, "task-system/prompts")),
  ].filter((f) => f.endsWith(".md"));
  for (const file of files) {
    if (!exists(file)) continue;
    read(file)
      .split(/\r?\n/)
      .forEach((line, i) => {
        if (authorityViolations(line))
          bad.push(`${rel(file)}:${i + 1}: ${line.trim().slice(0, 110)}`);
      });
  }
  const agents = join(OPENCODE, "AGENTS.md");
  if (!exists(agents) || !AUTHORITY_CANON.test(read(agents))) {
    bad.push("AGENTS.md: the canonical commit-authority statement is gone (FIND-776)");
  }
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
  // (a) the canonical header must exist where claim rows are defined. If it is
  // renamed, the gate would otherwise skip every row — measured round 4.
  for (const [p, rel0] of [
    [join(ROOT, "docs/dev/Backlog.md"), "docs/dev/Backlog.md"],
    [
      join(OPENCODE, "task-system/prompts/findings.md"),
      ".opencode/task-system/prompts/findings.md",
    ],
  ]) {
    if (!exists(p)) {
      bad.push(`${rel0}: missing — cannot verify the canonical claim header`);
      continue;
    }
    const text = read(p);
    if (!CANON_HEADER.test(text)) {
      const firstHeader = (text.split(/\r?\n/).find((l) => /^\s*\|.*\|/.test(l)) || "").trim();
      bad.push(
        `${rel0}: no canonical claim header (ID | Severidad | Hallazgo | … | Descripción). First table line: ${
          firstHeader.slice(0, 100) || "none"
        }`,
      );
    }
  }
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
  ["commit-authority (FIND-776)", checkCommitAuthority],
  ["repo-integrity (incidente 72643d9df)", checkRepoIntegrity],
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

  // B1 (round 4): the fixture READS the real row from findings.md. Hardcoding it
  // is what let round 2's `2h` stand in for the spec's `🟡 1d` — if the file
  // changes shape, these fixtures go red instead of lying.
  let findingsText = "";
  try {
    findingsText = read(join(OPENCODE, "task-system/prompts/findings.md"));
  } catch {
    findingsText = "";
  }
  const literalRow = (findingsText.match(/^\|\s*`FIND-<n>`[^\n]*$/m) || [null])[0];
  const literalHeader = (findingsText.match(
    /ID\s*\|\s*Severidad\s*\|\s*Hallazgo[^\n]*Dependencias\s*\|/,
  ) || [null])[0];
  ok("medicion: findings.md exposes the FIND row template", !!literalRow);
  ok("medicion: findings.md exposes the canonical header", !!literalHeader);
  ok(
    "medicion: findings.md header is accepted by CANON_HEADER",
    !!literalHeader && CANON_HEADER.test(literalHeader),
  );

  const HEADER10 = literalHeader
    ? `| ${literalHeader
        .split("|")
        .map((c) => c.trim())
        .filter(Boolean)
        .join(" | ")} |\n|---|---|---|---|---|---|---|---|---|---|`
    : "";
  const viol = (row) =>
    claimViolations(
      [{ file: "docs/dev/Backlog.md", line: 3, text: row }],
      () => `${HEADER10}\n${row}\n`,
    );
  const swap = (from, to) => (literalRow || "").replace(from, to);

  ok(
    "medicion: literal findings.md row (1d + Origen) passes",
    !!literalRow && viol(literalRow).length === 0,
  );
  ok(
    "medicion: claim in Hallazgo fails",
    viol(swap("**<título corto>**", "**92 unwraps en contexto prod**")).length === 1,
  );
  ok(
    "medicion: single-token claim in Descripción fails (inverse hole)",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92**")).length === 1,
  );
  ok(
    "medicion: percentage claim in Descripción fails",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**+92%**")).length === 1,
  );
  ok(
    "medicion: claim WITH Medición passes",
    viol(
      swap(
        "<descripción: qué se encontró + acción propuesta>",
        "Medición: `rg -c unwrap src/` -> 92",
      ),
    ).length === 0,
  );
  ok(
    "medicion: non-canonical header makes an added Backlog row fail (fail-closed)",
    claimViolations(
      [
        {
          file: "docs/dev/Backlog.md",
          line: 3,
          text: "| `DEAD-AA` | Media | **92 unwraps en contexto prod** | `a.rs` | 2h | P2 | P | **92** | r | - |",
        },
      ],
      () => "| ID | Severidad | Finding | Description | Esfuerzo |\n|---|---|---|---|---|\n",
    ).length === 1,
  );
  // R1: the fail-closed rule must not fire on the Backlog's own dashboards nor on
  // a legitimate row that asserts nothing.
  ok(
    "medicion: dashboard row without an ID column is skipped",
    claimViolations(
      [{ file: "docs/dev/Backlog.md", line: 3, text: "| Memory | 12 | 7 | 5 | 0 |" }],
      () =>
        "| Phase / Dimensión | Items Totales | Completadas | Pendientes | Diferidas |\n|---|---|---|---|---|\n",
    ).length === 0,
  );
  ok(
    "medicion: legit row in a Brecha table without a claim passes",
    claimViolations(
      [
        {
          file: "docs/dev/Backlog.md",
          line: 3,
          text: "| `FIND-950` | Media | **Falta el gate de docs** | `a.rs` | 1h | P2 | Abierta | añadir gate |",
        },
      ],
      () =>
        "| ID | Severidad | Brecha / Tarea | Archivos Clave Afectados | Esfuerzo | Prioridad | Estado Real | Acción Concreta |\n|---|---|---|---|---|---|---|---|\n",
    ).length === 0,
  );
  ok(
    "medicion: claim in a Brecha table still fails",
    claimViolations(
      [
        {
          file: "docs/dev/Backlog.md",
          line: 3,
          text: "| `FIND-950` | Media | **92 unwraps en prod** | `a.rs` | 1h | P2 | Abierta | añadir gate |",
        },
      ],
      () =>
        "| ID | Severidad | Brecha / Tarea | Archivos Clave Afectados | Esfuerzo | Prioridad | Estado Real | Acción Concreta |\n|---|---|---|---|---|---|---|---|\n",
    ).length === 1,
  );
  ok(
    "medicion: plan wave table is not fail-closed",
    claimViolations(
      [{ file: "docs/dev/plans/x.md", line: 3, text: "| 1 | `A-1` | W1 | R | `src/**` | 2h |" }],
      () =>
        "| # | ID | Wave | Carril | Ficheros / zona exclusiva | Prerequisito |\n|---|---|---|---|---|---|\n",
    ).length === 0,
  );

  // B3: a plan with no frontmatter is active (the template emitted none).
  ok(
    "plans: status absent is active",
    isActivePlan("# Plan de Ejecución: x\n\n> **Inicio:** 2026-11-26\n"),
  );
  ok("plans: status superseded is not", !isActivePlan("status: superseded\n# x"));
  ok("plans: status active is active", isActivePlan("status: active\n# x"));

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
    "waves: trailing slash conflict fails",
    waveConflicts([W("A-1", "`src/`"), W("A-2", "`src/index/`")]).length === 1,
  );

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

  // FIND-776: the canonical statement passes, each contradictory restatement fails.
  ok(
    "authority: canonical statement passes",
    !authorityViolations(
      "El **commit local de cierre** de una tarea lo hace el ejecutor de esa tarea",
    ),
  );
  ok(
    "authority: 'unico que prepara+ejecuta commit' fails",
    authorityViolations("edit: allow # lead full (único que prepara+ejecuta commit/push/release)"),
  );
  ok(
    "authority: 'git mutating (commit/...)' fails",
    authorityViolations("shell: allow # único rol con git mutating (commit/push/tags/release-plz)"),
  );
  ok(
    "authority: 'the lead commits' fails",
    authorityViolations("never commits (the lead commits)"),
  );
  ok(
    "authority: 'mutating solo lead' without commit passes",
    !authorityViolations("shell: allow # git solo lectura, mutating ❌ solo lead"),
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

// M2: the claim rule must run when the row is CREATED, not only in /harness
// (a row committed inside a normal task never reaches the /harness diff).
if (rowsOnly) {
  const bad = claimViolations(addedDiffRows());
  if (bad.length) {
    for (const b of bad) console.log(`FAIL ${b}`);
    console.log(`\n${bad.length} claim row(s) without "Medición:" — findings.md`);
    process.exit(1);
  }
  console.log("ok   row claims (added Backlog/plan rows carry their Medición:)");
  process.exit(0);
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
    : `\nharness contract holds (${CHECKS.length} checks)`,
);
process.exit(violations ? 1 : 0);
