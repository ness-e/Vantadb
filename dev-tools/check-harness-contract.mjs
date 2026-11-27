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
import { readFileSync, readdirSync, statSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, resolve, dirname } from "node:path";
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
const CLAIM_COLUMNS = [/^hallazgo$/, /^descripcion$/];

/**
 * FIND-779 (b): the measurement is read from the CLAIM CELL, never from the row.
 * Two ways the old `HAS_MEASURE.test(text)` failed open:
 *   - `no medido` matched `medido\b`, so a claim walked through its own denial;
 *   - a word in a column nobody judges (`Ver:` says "medido") absolved the claim
 *     cell that carried no measurement at all.
 * A `Medición:` with no value is the FIND-785 shape — a field that reads as
 * fulfilled while measuring nothing — and is not a measurement either.
 */
const MEASURE_FIELD = /(?<!\bsin\s+)Medici[oó]n\s*:([^|·;]*)/i;
const MEASURED_WORD = /\bmedid[oa]s?\b/i;
// FIND-779 (b), P2-01 F8: the negation is not one phrase. Measured fail-opens were
// `no medido`, `no fue medido`, `no está medido`, `sin ser medido`, `sin Medición:`;
// the words between the negator and the participle are pinned to two, and they are
// `\S+`-class WITHOUT punctuation: `\w` cannot consume `está` (F8a) and a bare `\S+`
// eats `embargo,` and turns `sin embargo, medido` into a violation (F8/R3-4).
const MEASURE_NEGATED =
  /\b(no|sin|nunca|jam[aá]s|not)\s+(?:[^\s,;·()]+\s+){0,2}(medid[oa]s?|medir|medici[oó]n|midi[oó])\b/i;
// A field whose whole value is a placeholder is FIND-785 with a colon: it reads as
// filled and measures nothing. `N/A` is covered by the `/i` (fixture 8N-2f).
const MEASURE_PLACEHOLDER = /^(n\/?a|tbd|pendiente|sin\s+datos|[-–—.\s]+)$/i;

export function hasMeasure(cell) {
  const s = String(cell || "");
  // P2-01 F8b: `sin Medición:` is not a field (the negation applies to the FIELD,
  // not only to its value — `sin Medición: 92` used to measure). `\s+` and not `\s`:
  // `sin  Medición:` with two spaces is the same phrase (R3-1).
  const field = s.match(MEASURE_FIELD);
  if (field) {
    // P2-01 O1: the value ends at a separator, so prose that comes AFTER the field
    // (`Medición: rg -> 92 · no medido …`) cannot contaminate it.
    const value = (field[1] || "").trim();
    return (
      /[A-Za-z0-9]/.test(value) && !MEASURE_NEGATED.test(value) && !MEASURE_PLACEHOLDER.test(value)
    );
  }
  if (MEASURE_NEGATED.test(s)) return false; // the negation must negate
  return MEASURED_WORD.test(s);
}

const normHeader = (c) =>
  c
    .replace(/[*`]/g, "")
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .trim()
    .toLowerCase();

const isSeparator = (s) => /^\s*\|[\s:|-]+\|\s*$/.test(s) && s.includes("-");

/**
 * FIND-779 (c): cells of a table row, split on UNESCAPED pipes. A cell may carry
 * `a\|b` (a path with a pipe, an alternation in a regex); splitting naively adds
 * a cell, every index after it shifts by one, and the gate judges the wrong one —
 * in silence, because a clean-looking wrong cell reads as "no claim".
 * Rows AND headers must use this same splitter or the indices disagree again.
 */
export function splitCells(line) {
  return String(line)
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split(/(?<!\\)\|/);
}

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
      return splitCells(lines[i - 1]).map(normHeader);
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
    const cells = splitCells(text);
    // FIND-779 (b): what is judged is a set of CELLS, and the measurement has to
    // live in one of them. Testing the row let a claim pass on a word in a column
    // nobody judges, or on its own negation.
    const unmeasured = (cols) => {
      const values = cols.map((k) => cells[k] || "");
      if (values.some(hasMeasure)) return [];
      return values.filter((c) => CLAIM.test(c));
    };
    if (!idx || !idx.length) {
      // Round 5 (R1): only a data table (has an `ID` column) is judged, and only
      // its prose columns — the Backlog's dashboards (`Phase / Dimensión | Items
      // Totales | …`) and a legit row with no claim are not violations. A renamed
      // claim header (`Finding/Description`, `Problema/Detalle`) still fails.
      if (!isBacklogFile(file) || !header || !header.includes("id")) continue;
      const proseCols = header.map((h, k) => (PROSE_HINT.test(h) ? k : -1)).filter((k) => k >= 0);
      // FIND-779 (d): a data table whose header matches no prose word used to be a
      // SILENT skip — the round-4 hole, reopened by renaming one header. "Cannot
      // judge" is not "nothing to judge": the gate says so by name.
      if (!proseCols.length) {
        out.push(
          `${file}:${line}: added row under a table the gate cannot judge (no recognizable claim column in ${header.join(
            " | ",
          )}) — rename the header to the canonical one or to a prose column`,
        );
        continue;
      }
      const claimed = unmeasured(proseCols);
      if (!claimed.length) continue;
      out.push(
        `${file}:${line}: claim under a non-canonical header (${header.join(" | ")}) — ${claimed[0]
          .trim()
          .slice(0, 80)}`,
      );
      continue;
    }
    const claimed = unmeasured(idx);
    if (!claimed.length) continue;
    out.push(`${file}:${line}: claim without "Medición:" — ${claimed[0].trim().slice(0, 90)}`);
  }
  return out;
}

// ─────────────────────────────────────────────────────────────────────────────
// FIND-811: "re-verificado" is not verified
// ─────────────────────────────────────────────────────────────────────────────

/**
 * A row that CLAIMS a re-verification: `re-verificado` or `verificado <fecha>`.
 * The dated form is the one that rotted: 4 of the 9 rows FIND-607 produced
 * described code deleted 24-27 days before the mark was written, and one of
 * them headed the implementation queue. A date alone proves nothing — the
 * claim has to name what was run (`Medición:` does the same for numbers).
 */
const REVERIFY_CLAIM = /\b(re-)?verificad[oa]s?\b[^|]{0,40}?\b\d{4}-\d{2}-\d{2}\b/i;
// Evidence: a commit sha (letters required, so a bare 8-digit date is not one)
// or a backticked command. `archivo.rs:146` is NOT evidence — that is the
// citation that rots, not the check that caught it rotting.
const COMMIT_SHA = /\b(?=[0-9a-f]*[a-f])[0-9a-f]{7,40}\b/i;
const EVIDENCE_CMD =
  /`[^`]*\b(rg|grep|git (?:log|show|cat-file|blame|ls-files)|cargo (?:check|nextest|clippy|test)|node|pwsh|Select-String|Get-ChildItem|pytest)\b[^`]*`/i;

export function hasReverifyEvidence(cell) {
  const s = String(cell || "");
  return COMMIT_SHA.test(s) || EVIDENCE_CMD.test(s);
}

/**
 * Same surface as the H-02 gate (rows ADDED to the diff). Judged cell by cell
 * and NOT only on the claim columns: the old stamps live in the `Relaciones`
 * column too (`… · verificado 2026-10-04 | — |`), which is where 4 of the 9
 * false rows carried theirs. The evidence must sit in the SAME cell as the
 * claim, so a commit quoted in a neighbouring column cannot absolve it.
 */
export function reverifyViolations(addedRowLines, fileText = (f) => read(join(ROOT, f))) {
  const out = [];
  const cache = new Map();
  for (const { file, line, text } of addedRowLines) {
    if (!/^\s*\|/.test(text)) continue;
    if (!cache.has(file)) {
      try {
        cache.set(file, fileText(file));
      } catch {
        cache.set(file, null);
      }
    }
    const body = cache.get(file);
    if (body === null) continue;
    if (!governingHeader(body, line)) continue; // not in a table
    for (const cell of splitCells(text)) {
      if (REVERIFY_CLAIM.test(cell) && !hasReverifyEvidence(cell)) {
        out.push(
          `${file}:${line}: "re-verificado" without a command or a commit — ${cell
            .trim()
            .slice(0, 90)}`,
        );
      }
    }
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
// FIND-794: a nextest `-E` that selects nothing is not a gate
// ─────────────────────────────────────────────────────────────────────────────
//
// FIND-782 declared `-E 'test(find782)'` and measured 0 tests: nextest matches
// FUNCTION NAMES, and no test was called that. The gate was green with the fix
// reverted. The exit code is not the contract to lean on either — measured on
// 0.9.133, a filtered `run` returns 4, but `--no-tests pass` returns 0 with the
// identical `0 tests run: 0 passed` summary, and `list` returns 0 with zero
// entries (the shape FIND-768/785/788 counted as "0 of 42"). The only durable
// assertion is a population assertion, so it is made MECHANICAL here:
//
//   R1  every nextest RUN that carries `-E` must carry `--no-tests fail`.
//   R2  every positive `test(<name>)` atom in a filterset line must name a
//       function that exists in the workspace (`fn` corpus, substring match
//       because nextest's `test(x)` is a substring match unless `=` is used).
//
// Both are textual, and the ceiling is declared rather than hidden:
//   - a selector assembled in a variable exposes no ATOMS (that is why R1 also
//     follows `X+=(-E "$sel")` to the `run` that expands `"${X[@]}"`: without
//     that, both CI lanes — the only places this rule claimed to cover — were
//     invisible, measured 2026-11-27 by deleting their flag and watching the
//     check stay green);
//   - `test(/regex/)` cannot be verified statically — only counted;
//   - a `test(<a::b::c>)` atom is checked by its LAST segment only: a wrong
//     module prefix passes (a corpus of `fn` names carries no module paths);
//   - the `fn` corpus is workspace-wide, not scoped to the package: an atom that
//     exists in another crate satisfies R2;
//   - `sourcesForNextest` scans `.github/workflows/*.yml`, `dev-tools/**` and the
//     root `Justfile`. `scripts/**` and `.github/actions/**` are NOT scanned;
//   - the auditor file is excluded whole (not by line range), so a real
//     `nextest run` added to it would be invisible;
//   - NEGATED atoms are deliberately ignored (`not test(x)`, `not (…)`): a dead
//     negation leaves the population a SUPERSET, which is not the silent-off
//     this check exists to prevent. Ignoring them is what keeps
//     `gate-common.ps1`'s `$fastGateFilter` — three negations — from becoming a
//     false positive.

/** Join PowerShell backtick and shell backslash continuations into one command. */
export function logicalCommands(text) {
  const lines = text.split(/\r?\n/);
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    const start = i + 1;
    let cmd = lines[i];
    while (/[`\\]\s*$/.test(cmd) && i + 1 < lines.length) {
      cmd = cmd.replace(/[`\\]\s*$/, " ") + lines[++i];
    }
    out.push({ line: start, text: cmd });
  }
  return out;
}

/**
 * Drop a trailing `#` / `//` comment. A `--no-tests fail` that only exists in a
 * tail comment must not satisfy R1 — that is the same "text that reads green
 * while the gate is off" the check exists to refuse. Quotes are respected so a
 * `#` inside a filter or a string is not treated as a comment.
 */
export function stripTrailingComment(cmd) {
  let quote = null;
  for (let i = 0; i < cmd.length; i++) {
    const c = cmd[i];
    if (quote) {
      if (c === quote) quote = null;
      else if (c === "\\" && quote === '"') i++;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      quote = c === "`" ? "'" : c;
      continue;
    }
    if ((c === "#" || (c === "/" && cmd[i + 1] === "/")) && (i === 0 || /\s/.test(cmd[i - 1]))) {
      return cmd.slice(0, i);
    }
  }
  return cmd;
}

/** Selector variables a file feeds into a command: `X+=(-E "$sel")`. */
export function nextestSelectorVars(text) {
  const vars = new Set();
  for (const { text: raw } of logicalCommands(text)) {
    const cmd = stripTrailingComment(raw);
    if (/^\s*(#|\/\/)/.test(cmd)) continue;
    const m =
      cmd.match(/([A-Za-z_]\w*)\s*\+?=\s*\(?\s*-E\b/) ||
      cmd.match(/([A-Za-z_]\w*)\s*\+?=\s*\([^)]*["']-E["']/);
    if (m) vars.add(m[1]);
  }
  return vars;
}

/**
 * Does this command hand a filterset to nextest? Either inline (`-E`) or through
 * a variable the file built with `-E` (`"${run_args[@]}"`). Without the second
 * form R1 was blind to BOTH CI lanes: `certification.yml` and
 * `snapshot-certification.yml` append the selector to an array and never spell
 * `-E` on the `run` — measured: deleting `--no-tests fail` from either left the
 * check green.
 */
function carriesSelector(cmd, selectorVars) {
  if (/(^|[\s,("'])-E(?=[\s,)"']|$)/.test(cmd)) return true;
  for (const v of selectorVars) {
    if (new RegExp(`\\$\\{?${v}(\\[@\\]|\\b)`).test(cmd)) return true;
  }
  return false;
}

/** A logical command that RUNS nextest (`list` only measures; `llvm-cov nextest` runs). */
function nextestRunCommand(cmd) {
  if (!/\bnextest\b/.test(cmd)) return false;
  if (/\bnextest\b["',\s]{1,12}\blist\b/.test(cmd) && !/\brun\b/.test(cmd)) return false;
  if (/\bnextest\b["',\s]{1,12}\brun\b/.test(cmd)) return true;
  return /\bllvm-cov\b[\s\S]{0,40}?\bnextest\b/.test(cmd); // `llvm-cov nextest` implies run
}

/** R1: a nextest run with a selector must pin the population assertion. */
export function nextestGuardViolations(file, text) {
  const bad = [];
  const selectorVars = nextestSelectorVars(text);
  for (const { line, text: raw } of logicalCommands(text)) {
    if (/^\s*(#|\/\/)/.test(raw)) continue; // prose about a command is not a command
    const cmd = stripTrailingComment(raw);
    if (!nextestRunCommand(cmd)) continue;
    if (!carriesSelector(cmd, selectorVars)) continue; // no selector: no population claim
    if (/--no-tests[=,"'\s]{1,6}fail/.test(cmd)) continue; // CLI form and argv-array form
    bad.push(
      `${file}:${line}: nextest run with -E but no \`--no-tests fail\` — a filter that selects 0 tests must fail the step, not pass it`,
    );
  }
  return bad;
}

/** Character ranges of a line that a `not` covers (`not test(x)`, `not (… and …)`). */
function negationRanges(line) {
  const ranges = [];
  const re = /\bnot\b/g;
  let m;
  while ((m = re.exec(line))) {
    let i = m.index + 3;
    while (i < line.length && /\s/.test(line[i])) i++;
    if (line[i] === "(") {
      let depth = 0;
      let j = i;
      for (; j < line.length; j++) {
        if (line[j] === "(") depth++;
        else if (line[j] === ")" && --depth === 0) break;
      }
      ranges.push([m.index, j]);
    } else if (line.startsWith("test(", i)) {
      const close = line.indexOf(")", i);
      ranges.push([m.index, close < 0 ? line.length : close]);
    }
  }
  return ranges;
}

/** R2: a positive `test(<name>)` atom must name something that exists. */
export function nextestAtomViolations(file, text, corpus) {
  const bad = [];
  if (!/\bnextest\b/.test(text)) return bad; // not a nextest surface
  text.split(/\r?\n/).forEach((line, i) => {
    if (/^\s*(#|\/\/)/.test(line)) return; // prose/comment
    // Only a filterset line: `-E <expr>` or a `run_phase`/selector argument.
    if (!/\btest\(/.test(line) || !/(-E\b|\brun_phase\b)/.test(line)) return;
    const negated = negationRanges(line);
    const isNegated = (pos) => negated.some(([a, b]) => pos >= a && pos <= b);
    let from = 0;
    let at;
    while ((at = line.indexOf("test(", from)) >= 0) {
      from = at + 5;
      if (isNegated(at)) continue; // `not (…)` covers every atom inside it
      const close = line.indexOf(")", at);
      if (close < 0) break;
      const atom = line.slice(at + 5, close).trim();
      if (!atom || (atom.startsWith("/") && atom.endsWith("/"))) continue; // regex: unverifiable
      const exact = atom.startsWith("=");
      const name = atom.replace(/^[=~]/, "");
      if (!name) continue;
      const tail = name.split("::").pop();
      const hit = corpus.some((n) => (exact ? n === tail : n.includes(tail)));
      if (!hit) {
        bad.push(
          `${file}:${
            i + 1
          }: test(${atom}) names no function in the workspace — this filter selects nothing, so it gates nothing`,
        );
      }
    }
  });
  return bad;
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
 * The ≥50 signature is NOT false-positive-safe (FIND-779 (a): the old docstring
 * said it was, and a `git rm --cached` of 50+ paths proved it wrong). From the
 * worktree's side the two are identical — a commit that deletes the paths while
 * the files stay on disk. What separates them is the INDEX:
 *   - an intentional untrack leaves the paths UNTRACKED   (`?? <path>`);
 *   - the incident was committed from a stale/empty private index, so the real
 *     index still TRACKED them                          (`A  <path>`).
 * So the verdict is read from the index, and the ceiling is stated instead of
 * promised: a deletion whose index entry was ALSO dropped afterwards cannot be
 * told apart from an intentional untrack. Then the check does not guess — it
 * prints a note naming both hypotheses (≥50 untracked paths, or an inversion
 * below the threshold). Silence is the one answer it no longer gives.
 */
export function repoIntegrityViolations(root = ROOT, ref = "HEAD") {
  const violations = [];
  const notes = [];
  const EMPTY = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
  const git = (a) =>
    execFileSync("git", a, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  let tree;
  try {
    tree = git(["rev-parse", `${ref}^{tree}`]);
  } catch {
    return { violations, notes }; // not a git repo / no commits yet: not this check's business
  }
  if (tree === EMPTY) violations.push(`${ref}'s tree is EMPTY — the repo was committed away`);
  let deleted = [];
  try {
    // `-z`, not `--name-only`: without it git QUOTES non-ASCII paths
    // (`"caf\303\251.md"`), `exists()` is asked about a string that is not a path,
    // and every accented deletion silently stops counting (measured 2026-11-26).
    // `git show`, not `diff-tree`: without `-m`, diff-tree emits NOTHING for a
    // merge commit — measured 61 deletions vs 0, i.e. the check would go blind on
    // exactly the commit shape a bad lane is most likely to produce (P2-01 F1).
    deleted = execFileSync(
      "git",
      ["show", "-z", "--diff-filter=D", "--name-only", "--format=", ref],
      { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    )
      .split("\0")
      .filter(Boolean);
  } catch {
    deleted = [];
  }
  const stillOnDisk = deleted.filter((p) => exists(join(root, p)));
  let tracked = null;
  if (stillOnDisk.length) {
    try {
      tracked = new Set(
        execFileSync("git", ["ls-files", "-z"], {
          cwd: root,
          encoding: "utf8",
          stdio: ["ignore", "pipe", "ignore"],
        })
          .split("\0")
          .filter(Boolean),
      );
    } catch {
      tracked = null; // no index to read: report it, do not guess
    }
  }
  // An unreadable index cannot separate the two hypotheses: fail closed and say why.
  const isTracked = (p) => (tracked ? tracked.has(p) : true);
  const inverted = stillOnDisk.filter(isTracked);
  const untracked = stillOnDisk.filter((p) => !isTracked(p));
  if (stillOnDisk.length && tracked === null) {
    notes.push(
      `could not read the index (git ls-files) — every deleted path is classified as tracked`,
    );
  }
  if (inverted.length >= 50) {
    violations.push(
      `${ref} deleted ${inverted.length} paths that the INDEX still tracks and that are STILL in the working tree (HEAD/worktree inversion) — e.g. ${inverted[0]}`,
    );
  } else if (inverted.length) {
    notes.push(
      `${inverted.length} deleted path(s) are tracked again in the index and exist on disk — inversion in repair, below the 50-path threshold (e.g. ${inverted[0]})`,
    );
  }
  if (untracked.length >= 50) {
    notes.push(
      `${ref} deleted ${untracked.length} paths that exist on disk and are UNTRACKED in the index — consistent with \`git rm --cached\`; a real deletion whose index entry was dropped later looks the same, so this is a note, not a verdict (e.g. ${untracked[0]})`,
    );
  }
  return { violations, notes };
}

function checkRepoIntegrity() {
  const i = args.indexOf("--ref");
  const ref = i >= 0 && args[i + 1] ? args[i + 1] : "HEAD";
  const { violations, notes } = repoIntegrityViolations(ROOT, ref);
  for (const n of notes) console.log(`  note repo-integrity: ${n}`);
  if (verbose) console.log(`  repo-integrity: ${ref} tree ok, deletions classified by the index`);
  return violations;
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

function checkReverify() {
  return reverifyViolations(addedDiffRows());
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

/** FIND-794: every nextest `-E` in CI and in dev-tools, plus the fn corpus. */
function sourcesForNextest() {
  const files = [];
  const wf = join(ROOT, ".github/workflows");
  try {
    for (const e of readdirSync(wf)) if (e.endsWith(".yml")) files.push(join(wf, e));
  } catch {
    /* no workflows: nothing to audit */
  }
  const scan = (dir) => {
    let entries;
    try {
      entries = readdirSync(dir);
    } catch {
      return;
    }
    for (const e of entries) {
      if (e === "node_modules" || e === "reports" || e === "__pycache__") continue;
      const p = join(dir, e);
      if (statSync(p).isDirectory()) scan(p);
      else if (/\.(ps1|mjs|sh)$/.test(e)) files.push(p);
    }
  };
  scan(join(ROOT, "dev-tools"));
  // The Justfile runs nextest too (`just test`); a selector added there would
  // otherwise be invisible. Root-level only: `scripts/**` and
  // `.github/actions/**` are declared out of scope in the ceiling above.
  const justfile = join(ROOT, "Justfile");
  if (exists(justfile)) files.push(justfile);
  // The auditor is not a test surface: its own `--self-test` fixtures quote the
  // violating shapes on purpose. Excluding it costs nothing real — this file
  // runs no nextest — and keeps a fixture from being reported as a violation of
  // the rule it proves. (File-level, not range-level: declared in the ceiling.)
  return files.filter((f) => f !== fileURLToPath(import.meta.url));
}

/** Every `fn <name>` in the workspace: nextest `test(x)` is a substring match. */
function rustFnCorpus() {
  const names = [];
  const SKIP = new Set(["target", "node_modules", ".git", ".opencode", ".codegraph", "dist"]);
  const scan = (dir) => {
    let entries;
    try {
      entries = readdirSync(dir);
    } catch {
      return;
    }
    for (const e of entries) {
      if (SKIP.has(e)) continue;
      const p = join(dir, e);
      let st;
      try {
        st = statSync(p);
      } catch {
        continue;
      }
      if (st.isDirectory()) scan(p);
      else if (e.endsWith(".rs")) {
        const re = /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
        const text = read(p);
        let m;
        while ((m = re.exec(text))) names.push(m[1]);
      }
    }
  };
  scan(ROOT);
  return names;
}

function checkNextestSelect() {
  const bad = [];
  const files = sourcesForNextest();
  const corpus = rustFnCorpus();
  if (!corpus.length)
    bad.push("no Rust `fn` names found — R2 cannot verify any selector (empty corpus)");
  for (const f of files) {
    const text = read(f);
    bad.push(...nextestGuardViolations(rel(f), text));
    bad.push(...nextestAtomViolations(rel(f), text, corpus));
  }
  return bad;
}

const CHECKS = [
  ["commit-path (H-03)", checkCommitPath],
  ["commit-authority (FIND-776)", checkCommitAuthority],
  ["repo-integrity (incidente 72643d9df)", checkRepoIntegrity],
  ["plan-lote (H-05)", checkPlanLote],
  ["medicion (H-02)", checkMedicion],
  ["reverify (FIND-811)", checkReverify],
  ["premise (H-07)", checkPremise],
  ["wave-quotas (H-06)", checkWaveQuotas],
  ["nextest-select (FIND-794)", checkNextestSelect],
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

  // ───────────────────────────────────────────────────────────────────────────
  // FIND-779: one fixture per measured edge. Each one asserts the PROPERTY the
  // edge broke, not "a violation exists": (d) and (e) both end in one violation,
  // so the message has to be part of the assertion or the (d) branch would mask
  // a mutation of (e) — the "fixture passes because another path caught it" trap.
  // ───────────────────────────────────────────────────────────────────────────
  const tableUnder = (headerLine, row) =>
    claimViolations(
      [{ file: "docs/dev/Backlog.md", line: 3, text: row }],
      () => `${headerLine}\n|${"---|".repeat(splitCells(headerLine).length)}\n${row}\n`,
    );
  const claim92 = swap("<descripción: qué se encontró + acción propuesta>", "**92**");

  // 8N-2 (FIND-779 b): the measurement is a cell, and the negation negates.
  ok(
    "8N-2a: 'no medido' does not count as measured",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — no medido")).length ===
      1,
  );
  ok(
    "8N-2b: a 'medido' in a column nobody judges does not absolve the claim",
    viol(claim92.replace("Origen:", "Origen: medido en otra celda ·")).length === 1,
  );
  ok(
    "8N-2c: 'Medición:' with no value measures nothing",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — Medición:")).length ===
      1,
  );
  // P2-01 F8: the negation is not one phrase, and an explicit field beats prose.
  ok(
    "8N-2d: 'no fue medido' / 'sin ser medido' still negate",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — no fue medido"))
      .length === 1 &&
      viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — sin ser medido"))
        .length === 1,
  );
  ok(
    "8N-2e: a real 'Medición:' wins over a negated word in the same cell",
    viol(
      swap(
        "<descripción: qué se encontró + acción propuesta>",
        "**92** — no medido en el informe · Medición: `rg -c unwrap src/` -> 92",
      ),
    ).length === 0,
  );
  ok(
    "8N-2f: a placeholder value is not a measurement",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — Medición: n/a"))
      .length === 1 &&
      viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — Medición: tbd"))
        .length === 1 &&
      viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — Medición: N/A"))
        .length === 1,
  );
  // P2-01 F8a: `no está medido` — the two-word window is `\S+`, not `\w+`.
  ok(
    "8N-2g: 'no está medido' negates too",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — no está medido"))
      .length === 1,
  );
  // P2-01 F8b: `sin Medición:` is not a field, so its value cannot absolve the claim.
  ok(
    "8N-2h: 'sin Medición: <valor>' is not a measurement",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — sin Medición: 92"))
      .length === 1 &&
      viol(
        swap(
          "<descripción: qué se encontró + acción propuesta>",
          "**92** — sin Medición: ver nota",
        ),
      ).length === 1,
  );
  // P2-01 O4: a negated VALUE is not a measurement ("Medición: no medido").
  ok(
    "8N-2i: 'Medición: no medido' is a negated value, not a measurement",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — Medición: no medido"))
      .length === 1,
  );
  // P2-01 O1: prose AFTER a real field must not contaminate its value.
  ok(
    "8N-2j: 'Medición: <cmd> · no medido' keeps its measurement",
    viol(
      swap(
        "<descripción: qué se encontró + acción propuesta>",
        "**92** — Medición: `rg -c unwrap src/` -> 92 · no medido en el informe",
      ),
    ).length === 0,
  );
  // P2-01 R3: the negation is a phrase, not a spelling. Two spaces between `sin` and
  // `Medición:` is the same negation (the lookbehind is `\bsin\s+`).
  ok(
    "8N-2k: 'sin  Medición:' with two spaces is still not a field",
    viol(swap("<descripción: qué se encontró + acción propuesta>", "**92** — sin  Medición: 92"))
      .length === 1,
  );
  // P2-01 R3-4: `sin embargo, medido` is prose, not a denial of a measurement — the
  // word window must not swallow punctuation to reach the participle.
  ok(
    "8N-2l: 'sin embargo, medido en prod' is a measurement, not a negation",
    viol(
      swap(
        "<descripción: qué se encontró + acción propuesta>",
        "**92** — sin embargo, medido en prod",
      ),
    ).length === 0,
  );

  // 8N-3 (FIND-779 c): an escaped pipe must not shift the judged cell.
  ok(
    "8N-3: an escaped pipe in the row does not desalign the judged cell",
    viol(claim92.replace("`ruta:línea`", "`a\\|b.rs:12`")).length === 1,
  );
  // P2-01 F4: the same splitter has to run over the HEADER, or the indices of the
  // two sides disagree. Header with an escaped pipe + claim in Descripción.
  ok(
    "8N-3b: an escaped pipe in the header does not desalign it either",
    tableUnder(
      "| ID | Severidad | Hallazgo | Archivo\\|línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |",
      "| `FIND-9` | 🟠 Media | **<título>** | `a.rs` | 🟢 1h | 🟠 Media | 🟡 Pendiente | **92** | - | - |",
    ).length === 1,
  );

  // 8N-4 (FIND-779 d): a data table with no recognizable claim column is a
  // declared violation ("cannot judge"), not a silent skip.
  const OBS_HEADER = "| ID | Severidad | Observación | Comentario |";
  const synRow = "| `FIND-9` | 🟠 Media | **92 unwraps en prod** | sin cambios |";
  ok(
    "8N-4: a data table the gate cannot judge fails instead of skipping",
    (() => {
      const v = tableUnder(OBS_HEADER, synRow);
      return v.length === 1 && /cannot judge/.test(v[0]);
    })(),
  );

  // 8N-5 (FIND-779 e): the `Problema/Detalle` synHeader branch had no fixture, so
  // "stays at exit 1" was not reproducible from the repo.
  const SYN_HEADER = "| ID | Severidad | Problema | Detalle | Evidencia |";
  ok(
    "8N-5a: synHeader `Problema/Detalle` still fails as non-canonical",
    (() => {
      const v = tableUnder(
        SYN_HEADER,
        "| `FIND-9` | 🟠 Media | **92 unwraps en prod** | sin cambios | - |",
      );
      return v.length === 1 && /non-canonical header/.test(v[0]);
    })(),
  );
  ok(
    "8N-5b: synHeader row whose only 'Medición:' is in an unjudged column still fails",
    (() => {
      const v = tableUnder(
        SYN_HEADER,
        "| `FIND-9` | 🟠 Media | **92 unwraps en prod** | sin cambios | Medición: `rg` -> 92 |",
      );
      // P2-01 F2: assert WHICH branch fired. With `.length === 1` alone this fixture
      // passed through the (d) "cannot judge" branch when PROSE_HINT was mutated —
      // a fixture green for the wrong reason, the trap this file exists to close.
      return v.length === 1 && /non-canonical header/.test(v[0]);
    })(),
  );

  // 8N-1 (FIND-779 a): `repo-integrity` is decided by the index, on real repos.
  // Two disposable git repos: the FP that must NOT fire, and the true positive
  // that must keep firing (the guard against "fixing" it by switching it off).
  const gitRepo = (extra = [], count = 60) => {
    const dir = mkdtempSync(join(tmpdir(), "hcntr-"));
    const g = (a, env) =>
      execFileSync("git", ["-c", "commit.gpgsign=false", ...a], {
        cwd: dir,
        encoding: "utf8",
        env: env ? { ...process.env, ...env } : process.env,
        stdio: ["ignore", "pipe", "ignore"],
      }).trim();
    g(["init", "-q"]);
    g(["config", "user.email", "gate@selftest"]);
    g(["config", "user.name", "gate-selftest"]);
    writeFileSync(join(dir, "keep.txt"), "k\n");
    for (const x of extra) writeFileSync(join(dir, x), "x\n");
    for (let i = 0; i < count; i++) writeFileSync(join(dir, `f${i}.txt`), `${i}\n`);
    g(["add", "-A"]);
    g(["commit", "-q", "-m", "seed"]);
    return { dir, g };
  };
  const SIXTY = Array.from({ length: 60 }, (_, i) => `f${i}.txt`);
  // The stale-index commit: HEAD loses `files` while the real index keeps tracking
  // them — the incident signature, at a size the caller chooses.
  const staleIndexRepo = (count) => {
    const { dir, g } = gitRepo([], count);
    const privateIndex = { GIT_INDEX_FILE: join(dir, ".git", "selftest-index") };
    const gi = (a) => g(a, privateIndex);
    gi(["read-tree", "--empty"]);
    gi(["add", "keep.txt"]);
    const tree = gi(["write-tree"]);
    const stale = g(["commit-tree", tree, "-p", "HEAD", "-m", "stale index"]);
    g(["update-ref", "HEAD", stale]);
    return dir;
  };
  let repo = null;
  try {
    repo = gitRepo();
    repo.g(["rm", "-q", "--cached", ...SIXTY]);
    repo.g(["commit", "-q", "-m", "untrack"]);
    ok(
      "8N-1a: `git rm --cached` of 60 paths is not an inversion",
      repoIntegrityViolations(repo.dir).violations.length === 0,
    );
  } finally {
    if (repo) rmSync(repo.dir, { recursive: true, force: true });
  }
  // 8N-1d (P2-01 F1): a MERGE commit is the shape a bad lane is most likely to
  // produce, and `diff-tree` without `-m` emits nothing for it — the check went
  // blind on 61 deletions (measured: show -z → 61, diff-tree -z → 0).
  let merged = null;
  try {
    merged = gitRepo();
    merged.g(["checkout", "-q", "-b", "side"]);
    writeFileSync(join(merged.dir, "side.txt"), "s\n");
    merged.g(["add", "-A"]);
    merged.g(["commit", "-q", "-m", "side"]);
    const p2 = merged.g(["rev-parse", "HEAD"]);
    merged.g(["checkout", "-q", "-"]);
    const p1 = merged.g(["rev-parse", "HEAD"]);
    const privateIndex = { GIT_INDEX_FILE: join(merged.dir, ".git", "selftest-merge") };
    const gi = (a) => merged.g(a, privateIndex);
    gi(["read-tree", "--empty"]);
    gi(["add", "keep.txt"]);
    const tree = gi(["write-tree"]);
    const evil = merged.g(["commit-tree", tree, "-p", p1, "-p", p2, "-m", "evil merge"]);
    merged.g(["update-ref", "HEAD", evil]);
    ok(
      "8N-1d: a MERGE commit that deletes 60 tracked paths is still an inversion",
      repoIntegrityViolations(merged.dir).violations.length === 1,
    );
  } finally {
    if (merged) rmSync(merged.dir, { recursive: true, force: true });
  }
  // 8N-1e (P2-01 F5): an index the check cannot read must not become "nothing to
  // report" — it fails closed and says why.
  let broken = null;
  try {
    broken = gitRepo();
    broken.g(["rm", "-q", "--cached", ...SIXTY]);
    broken.g(["commit", "-q", "-m", "untrack"]);
    writeFileSync(join(broken.dir, ".git", "index"), "not an index\n");
    const res = repoIntegrityViolations(broken.dir);
    ok(
      "8N-1e: an unreadable index fails closed and says so",
      res.violations.length === 1 && /could not read the index/.test(res.notes.join(" ")),
    );
  } finally {
    if (broken) rmSync(broken.dir, { recursive: true, force: true });
  }

  // 8N-1c: `--name-only` QUOTES non-ASCII paths, so an accented deletion stopped
  // counting as "on disk" (measured 60 of 61 before the `-z` fix).
  let accented = null;
  try {
    accented = gitRepo(["café.md"]);
    accented.g(["rm", "-q", "--cached", "café.md", ...SIXTY]);
    accented.g(["commit", "-q", "-m", "untrack"]);
    const res = repoIntegrityViolations(accented.dir);
    ok(
      "8N-1c: an accented path deleted from HEAD is still seen on disk (61, not 60)",
      res.violations.length === 0 && /\b61 paths\b/.test(res.notes.join(" ")),
    );
  } finally {
    if (accented) rmSync(accented.dir, { recursive: true, force: true });
  }
  let invertedRepo = null;
  try {
    invertedRepo = gitRepo();
    const privateIndex = { GIT_INDEX_FILE: join(invertedRepo.dir, ".git", "selftest-index") };
    const gi = (a) => invertedRepo.g(a, privateIndex);
    gi(["read-tree", "--empty"]);
    gi(["add", "keep.txt"]);
    const tree = gi(["write-tree"]);
    const stale = invertedRepo.g(["commit-tree", tree, "-p", "HEAD", "-m", "stale index"]).trim();
    invertedRepo.g(["update-ref", "HEAD", stale]);
    ok(
      "8N-1b: a stale-index commit deleting 60 TRACKED paths is still an inversion",
      repoIntegrityViolations(invertedRepo.dir).violations.length === 1,
    );
  } finally {
    if (invertedRepo) rmSync(invertedRepo.dir, { recursive: true, force: true });
  }
  // P2-01 O3: "≥50" is the contract's own number, so both sides of the edge are
  // pinned — 50 paths is a violation, 49 is not (a note, not silence).
  let atThreshold = null;
  try {
    atThreshold = staleIndexRepo(50);
    ok(
      "8N-1f: exactly 50 inverted paths is a violation (>= 50, not > 50)",
      repoIntegrityViolations(atThreshold).violations.length === 1,
    );
  } finally {
    if (atThreshold) rmSync(atThreshold, { recursive: true, force: true });
  }
  let belowThreshold = null;
  try {
    belowThreshold = staleIndexRepo(49);
    const res = repoIntegrityViolations(belowThreshold);
    ok(
      "8N-1g: 49 inverted paths is not a violation, but it is not silence either",
      res.violations.length === 0 && /in repair/.test(res.notes.join(" ")),
    );
  } finally {
    if (belowThreshold) rmSync(belowThreshold, { recursive: true, force: true });
  }

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

  // FIND-794: a nextest `-E` that selects nothing must not be able to look green.
  // Each pair is the rule in the two directions: the violating text, then the
  // same text with the population assertion present.
  const FX = "x.ps1";
  ok(
    "nextest-select: -E without --no-tests fail fails",
    nextestGuardViolations(FX, "cargo nextest run -p vantadb -E 'test(proxy)'").length === 1,
  );
  ok(
    "nextest-select: same with --no-tests fail passes",
    nextestGuardViolations(FX, "cargo nextest run -p vantadb -E 'test(proxy)' --no-tests fail")
      .length === 0,
  );
  ok(
    "nextest-select: llvm-cov nextest run is a run too",
    nextestGuardViolations(FX, '$cmd = @("cargo", "llvm-cov", "nextest", "-E", $f)').length === 1,
  );
  ok(
    "nextest-select: `nextest list` only measures, it does not gate",
    nextestGuardViolations(FX, "cargo nextest list -p vantadb -E 'test(proxy)'").length === 0,
  );
  ok(
    "nextest-select: a PowerShell continuation is one command",
    nextestGuardViolations(
      FX,
      "cargo nextest run -p vantadb `\n    -E 'test(proxy)' --build-jobs 2",
    ).length === 1,
  );
  ok(
    "nextest-select: a prose comment about the flag is not a command",
    nextestGuardViolations(FX, "# cargo nextest run -E 'test(x)' sin flag").length === 0,
  );
  const CORPUS = ["test_proxy_reelect", "compat_v090_rollback_fixture", "e2e_l0_to_l3"];
  ok(
    "nextest-select: test(<missing name>) is caught (FIND-782's exact shape)",
    nextestAtomViolations(FX, "nextest run -E 'test(find782)'", CORPUS).length === 1,
  );
  ok(
    "nextest-select: test(<real substring>) passes",
    nextestAtomViolations(FX, "nextest run -E 'test(proxy)'", CORPUS).length === 0,
  );
  ok(
    "nextest-select: test(=exact) demands the exact name",
    nextestAtomViolations(FX, "nextest run -E 'test(=test_prox)'", CORPUS).length === 1,
  );
  ok(
    "nextest-select: a negated atom is not the population (no false positive)",
    nextestAtomViolations(FX, "nextest run -E 'not test(/deserialize_absurd/)'", CORPUS).length ===
      0,
  );
  ok(
    "nextest-select: a De Morgan group is not the population either",
    nextestAtomViolations(FX, "nextest run -E 'not (test(a) and test(b))'", CORPUS).length === 0,
  );
  ok(
    "nextest-select: `1 test(s)` in an echo line is not a filter",
    nextestAtomViolations(
      "snapshot.yml",
      'nextest run --no-tests fail\necho "$phase executed 0 test(s)."',
      CORPUS,
    ).length === 0,
  );
  // Review P2-01 (C2): both CI lanes feed the selector through an array, so a
  // rule that only reads `-E` on the run command was blind to the ONLY places
  // it claimed to cover. These two fixtures are that hole.
  const ARRAY_WF = [
    'expect=("$@"); run_args=("$@")',
    'run_args+=(-E "$selector")',
    'expect+=(-E "$selector")',
    'cargo nextest run --profile audit "${run_args[@]}"',
  ].join("\n");
  ok(
    "nextest-select: array-fed selector without the flag fails (CI shape)",
    nextestGuardViolations("cert.yml", ARRAY_WF).length === 1,
  );
  ok(
    "nextest-select: array-fed selector with the flag passes",
    nextestGuardViolations(
      "cert.yml",
      ARRAY_WF.replace('"${run_args[@]}"', '--no-tests fail "${run_args[@]}"'),
    ).length === 0,
  );
  ok(
    "nextest-select: a nextest run that does NOT take the selector var is not flagged",
    nextestGuardViolations(
      "cert.yml",
      `${ARRAY_WF.replace(
        '"${run_args[@]}"',
        '--no-tests fail "${run_args[@]}"',
      )}\ncargo nextest run -p vantadb --test security_audit`,
    ).length === 0,
  );
  // Review P2-01 (R3): the flag must be in the command, not in a tail comment.
  ok(
    "nextest-select: `--no-tests fail` only in a tail comment fails",
    nextestGuardViolations(FX, "cargo nextest run -E 'test(proxy)' # ver --no-tests fail")
      .length === 1,
  );
  ok(
    "nextest-select: the same real flag with a tail comment passes",
    nextestGuardViolations(
      FX,
      "cargo nextest run -E 'test(proxy)' --no-tests fail # tracked in FIND-794",
    ).length === 0,
  );
  // Review P2-01 (R4): `not (…)` must not hide a POSITIVE atom on the same line.
  ok(
    "nextest-select: a positive atom outside a `not (…)` group is still checked",
    nextestAtomViolations(
      FX,
      "nextest run -E 'not (test(slow_a) or test(slow_b)) and test(NONEXISTENT_ZZZ)'",
      CORPUS,
    ).length === 1,
  );
  ok(
    "nextest-select: the same atom inside the negation is not the population",
    nextestAtomViolations(
      FX,
      "nextest run -E 'not (test(NONEXISTENT_ZZZ) or test(slow_b)) and test(proxy)'",
      CORPUS,
    ).length === 0,
  );

  // FIND-811: the mark that meant nothing. The second fixture is the row this
  // task RETIRED, verbatim from HEAD: proof that the rule can fail on real
  // content, not only on a synthetic one.
  const rv = (row) =>
    reverifyViolations(
      [{ file: "docs/dev/Backlog.md", line: 3, text: row }],
      () => `${HEADER10}\n${row}\n`,
    );
  const FIND271_HEAD =
    "| `FIND-271` | 🟢 Baja | **`CRASH_MODEL.md:75` stale (referencia a WAL browser que no aplica)** | `docs/dev/architecture/CRASH_MODEL.md:75` | 🟢 30min | 🟢 Nice-to-have | 🆕 Pendiente | Detectado por TS-13 (2026-10-04). | Origen: TS-13 (master-plan 0.9.0) · verificado 2026-10-04 | — |";
  ok(
    'reverify: a bare "re-verificado <fecha>" with no command fails',
    rv(swap("<descripción: qué se encontró + acción propuesta>", "(re-verificado 2026-11-22)"))
      .length === 1,
  );
  ok("reverify: the row retired by FIND-811 (verbatim HEAD) fails", rv(FIND271_HEAD).length === 1);
  ok(
    "reverify: the same row with the command that caught it passes",
    rv(
      FIND271_HEAD.replace(
        "verificado 2026-10-04",
        "verificado 2026-10-04 (`git cat-file -e HEAD:docs/dev/architecture/CRASH_MODEL.md` → exit 128)",
      ),
    ).length === 0,
  );
  ok(
    "reverify: a commit sha passes",
    rv(FIND271_HEAD.replace("verificado 2026-10-04", "verificado 2026-10-04 · commit `651d7a30d`"))
      .length === 0,
  );
  // P2-01 round 1, probes the rule's two ways to be wrong.
  const FIND271_SPLIT =
    "| `FIND-271` | 🟢 Baja | **x** | `a.rs:1` | 🟢 30min | 🟢 Nice | 🆕 Pendiente | re-verificado 2026-11-22 | Origen: `651d7a30d` | — |";
  ok(
    "reverify: a commit in ANOTHER column does not absolve the claim cell (FIND-779 b)",
    rv(FIND271_SPLIT).length === 1,
  );
  ok(
    "reverify: undated 'verificado' is not a dated claim",
    rv(
      "| `FIND-271` | 🟢 Baja | **x** | `a.rs:1` | 🟢 30min | 🟢 Nice | 🆕 Pendiente | runner verificado contra `zep-cloud==3.30.0` | Origen: T | — |",
    ).length === 0,
  );
  ok("reverify: a bare 8-digit date is not a commit", hasReverifyEvidence("20261127") === false);
  ok("reverify: a sha with a letter is evidence", hasReverifyEvidence("b312482da") === true);

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
  const rows = addedDiffRows();
  const bad = [...claimViolations(rows), ...reverifyViolations(rows)];
  if (bad.length) {
    for (const b of bad) console.log(`FAIL ${b}`);
    // P2-01 N1: the footer named only the missing-`Medición:` case, while `cannot
    // judge` (a table with no claim column) lands in the same list.
    console.log(
      `\n${bad.length} claim row(s) rejected — see the FAIL lines above (H-02 / FIND-779 / FIND-811)`,
    );
    process.exit(1);
  }
  console.log(
    "ok   row claims (added Backlog/plan rows carry their Medición: and re-verified rows say what was run)",
  );
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
