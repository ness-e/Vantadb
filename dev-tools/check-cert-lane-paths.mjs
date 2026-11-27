#!/usr/bin/env node
/**
 * CERT lane path contract - FIND-797.
 *
 * `snapshot-certification.yml` filters its own triggers with `on.push.paths` and
 * `on.pull_request.paths`. Those lists are what decide whether its seven phases
 * ever run on a push, and they used to be a hand-written enumeration: the push
 * ending at `66404bedc` (`dca7d5818..66404bedc`) changed
 * `vanta-memory/src/ingest/runner_config.rs` and four `vantadb-mcp/src/*.rs`
 * - crates its phases 06 and 07 execute - and the lane did not fire at all.
 *
 * This gate derives BOTH sides from the same file, so neither can drift on its own:
 *
 *   phases -> every `run_phase <id> <coverage> '<selector>' -p <crate> [--test <name>]`
 *   filter -> every entry of every `paths:` block
 *
 * Required for each crate a phase runs: its `Cargo.toml`, its `build.rs` when it
 * has one (it compiles for every target) and all of `src/**` - the selectors
 * cannot justify a smaller subset, because `--test` targets import the crate and
 * `test(<name>)` selects by test name, not by module. Required for each
 * `--test <name>`: some entry that glob-covers the test file. Required for every
 * block: the same entries as the others, plus the lane and this script themselves
 * (delete the self-entry and the gate can never run again to notice).
 *
 *   node dev-tools/check-cert-lane-paths.mjs                  # the real lane
 *   node dev-tools/check-cert-lane-paths.mjs --workflow <p>   # a mutated copy
 *   node dev-tools/check-cert-lane-paths.mjs --self-test      # fixtures, both directions
 *
 * Exit: 0 = contract holds, 1 = violation(s), 2 = usage/IO error.
 *
 * DELIBERATE CEILING (ponytail): this is a text contract, not a YAML parser - it
 * reads the two shapes the lane actually uses (a `run_phase` call and a `- "glob"`
 * entry), and fails when it finds a `paths:` key it cannot read that way rather
 * than skipping it. It does not resolve `Cargo.toml` workspace inheritance,
 * `[[test]]` renames, `#[path]`, or a `mod`/`include!` inside a test file (today
 * the root crate's tests are carried by `tests/**` and the three named test files
 * declare no submodule, so the hole is empty - it is not closed). A `nextest`
 * invocation outside a `run_phase` call is invisible to it, and it cannot flag a
 * *stale* entry: a superset only costs runs, and detecting it would need a
 * hand-written allow-list, which is the divergence this gate exists to prevent.
 * What it guarantees is the useful half: a crate added to a phase cannot reach CI
 * without its manifest and `src/**`, no block can drop an entry another one
 * watches, and the lane cannot stop watching itself.
 */
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const flagValue = (flag) => {
  const i = args.indexOf(flag);
  return i >= 0 && args[i + 1] && !args[i + 1].startsWith("--") ? args[i + 1] : null;
};
const ROOT = resolve(flagValue("--root") ?? join(dirname(fileURLToPath(import.meta.url)), ".."));
const WORKFLOW = resolve(
  ROOT,
  flagValue("--workflow") ?? ".github/workflows/snapshot-certification.yml",
);
const selfTest = args.includes("--self-test");

// ─────────────────────────────────────────────────────────────────────────────
// Parsing (fixture-tested by --self-test)
// ─────────────────────────────────────────────────────────────────────────────

/** Every `paths:` block, with the event it belongs to. [{ event, entries }] */
export function pathBlocks(text) {
  const lines = text.split(/\r?\n/);
  const blocks = [];
  let current = null;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!current) {
      if (/^\s{2,}paths:\s*$/.test(line)) {
        let event = "?";
        for (let j = i - 1; j >= 0; j--) {
          const m = lines[j].match(/^ {2}([A-Za-z_]+):\s*$/);
          if (m) {
            event = m[1];
            break;
          }
        }
        current = { event, entries: [] };
        blocks.push(current);
      }
      continue;
    }
    const item = line.match(/^\s*-\s*"([^"]+)"\s*(?:#.*)?$/) || line.match(/^\s*-\s*'([^']+)'\s*$/);
    if (item) {
      current.entries.push(item[1]);
      continue;
    }
    if (line.trim() === "" || line.trim().startsWith("#")) continue;
    current = null; // any other line closes the block
  }
  return blocks.filter((b) => b.entries.length > 0);
}

/**
 * Every `run_phase` invocation, joined across `\` continuations.
 * [{ phase, crates: [name], tests: [name] }]
 *
 * `-p` is matched with `matchAll`: a phase that grew a second `--package` runs a
 * crate this gate must derive too, and taking only the first one left it blind
 * exactly where the contract says it must fail.
 */
export function phases(text) {
  const lines = text.split(/\r?\n/);
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    // Deliberately not `(?:scoped|full)` and not `\S` after the id: a phase whose
    // coverage word is typo'd, or whose arguments moved to the next line, must
    // still be derived, or the crate it runs becomes invisible to this gate (the
    // same silent class it exists to close).
    const head = lines[i].match(/^\s*run_phase\b\s+(\S+)/);
    if (!head) continue;
    let command = lines[i];
    while (/\\\s*$/.test(command) && i + 1 < lines.length) {
      i++;
      command += " " + lines[i];
    }
    const crates = [...command.matchAll(/(?:^|\s)(?:-p|--package)(?:\s+|=)([A-Za-z0-9_-]+)/g)].map(
      (m) => m[1],
    );
    const tests = [...command.matchAll(/(?:^|\s)--test(?:\s+|=)([A-Za-z0-9_-]+)/g)].map(
      (m) => m[1],
    );
    out.push({ phase: head[1], crates, tests });
  }
  return out;
}

/** GitHub path filter semantics: `*` stops at `/`, `**` crosses it. */
export function globToRegExp(glob) {
  let re = "";
  for (let i = 0; i < glob.length; i++) {
    const c = glob[i];
    if (c === "*") {
      if (glob[i + 1] === "*") {
        re += ".*";
        i++;
        if (glob[i + 1] === "/") i++;
      } else re += "[^/]*";
    } else if ("\\^$.|?+()[]{}".includes(c)) re += "\\" + c;
    else re += c;
  }
  return new RegExp("^" + re + "$");
}

const covered = (entries, file) => entries.some((e) => globToRegExp(e).test(file));

// ─────────────────────────────────────────────────────────────────────────────
// Repository facts
// ─────────────────────────────────────────────────────────────────────────────

function packageName(manifest) {
  const text = readFileSync(manifest, "utf8");
  const pkg = text.split(/^\[/m).find((s) => s.startsWith("package]"));
  const m = pkg && pkg.match(/^name\s*=\s*"([^"]+)"/m);
  return m ? m[1] : null;
}

/** package name -> { dir (repo-relative, "" for the root crate), manifest } */
function locateCrate(crate) {
  const nested = join(ROOT, crate, "Cargo.toml");
  if (existsSync(nested)) return { dir: crate, manifest: `${crate}/Cargo.toml` };
  const rootManifest = join(ROOT, "Cargo.toml");
  if (existsSync(rootManifest) && packageName(rootManifest) === crate) {
    return { dir: "", manifest: "Cargo.toml" };
  }
  return null;
}

function walk(dir) {
  if (!existsSync(dir) || !statSync(dir).isDirectory()) return [];
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const full = join(dir, e.name);
    return e.isDirectory() ? walk(full) : [full];
  });
}

/** Repo-relative path of the file behind `cargo nextest --test <name>`, if any. */
function testTargetFile(crateDir, name) {
  const base = join(ROOT, crateDir);
  const direct = [join(base, "tests", `${name}.rs`), join(base, "tests", name, "main.rs")];
  for (const c of direct) if (existsSync(c)) return relative(ROOT, c).replace(/\\/g, "/");
  const nested = walk(join(base, "tests")).find((f) => basename(f) === `${name}.rs`);
  return nested ? relative(ROOT, nested).replace(/\\/g, "/") : null;
}

/** A `--test` belongs to whichever crate of the phase owns the file. */
function testTargetFileFor(run, name) {
  for (const candidate of run.crates) {
    const crate = locateCrate(candidate);
    if (!crate) continue;
    const file = testTargetFile(crate.dir, name);
    if (file) return file;
  }
  return null;
}

/** What the filter must watch so a change to this crate can reach its phases. */
function crateEntries(crate) {
  const entries = [crate.manifest, crate.dir ? `${crate.dir}/src/**` : "src/**"];
  // `build.rs` compiles for every target of the crate: a change there changes the
  // binary the phases run even when no `src/**` file moved.
  if (existsSync(join(ROOT, crate.dir, "build.rs"))) {
    entries.push(crate.dir ? `${crate.dir}/build.rs` : "build.rs");
  }
  return entries;
}

// ─────────────────────────────────────────────────────────────────────────────
// The contract
// ─────────────────────────────────────────────────────────────────────────────

export function audit(text, { selfPaths = [] } = {}) {
  const violations = [];
  const blocks = pathBlocks(text);
  const runs = phases(text);

  // A `paths:` key the reader could not turn into a block (flow style, a reindent)
  // would leave that whole trigger unaudited while the gate still prints `ok`.
  const declaredKeys = text.split(/\r?\n/).filter((l) => /^\s{2,}paths:/.test(l)).length;
  if (declaredKeys !== blocks.length) {
    violations.push(
      `${declaredKeys} \`paths:\` key(s) but ${blocks.length} readable block(s) - one is in a shape this gate cannot audit`,
    );
  }
  if (!blocks.length) violations.push("no `paths:` block found - the lane would run on every push");
  if (!runs.length) violations.push("no `run_phase` invocation found - nothing to derive from");

  // No trigger may watch less than another one: a push touching only the missing
  // entry stops firing while the other trigger still carries it, and nothing said so.
  const union = new Set(blocks.flatMap((b) => b.entries));
  for (const block of blocks) {
    for (const entry of union) {
      if (!block.entries.includes(entry)) {
        violations.push(
          `on.${block.event}.paths: missing "${entry}" that another paths block watches`,
        );
      }
    }
  }

  // The lane must watch itself and this gate: editing either is the mutation that
  // can open the gap, and `paths:` is the only thing that makes this check reachable.
  for (const block of blocks) {
    for (const entry of selfPaths) {
      if (!block.entries.includes(entry)) {
        violations.push(
          `on.${block.event}.paths: the lane must watch "${entry}" or this gate cannot run on its own edit`,
        );
      }
    }
  }

  for (const run of runs) {
    if (!run.crates.length) {
      violations.push(`phase ${run.phase}: no \`-p <crate>\` - cannot derive what it certifies`);
      continue;
    }
    // A missing test file is a phase-level fact, not a per-block one: resolve it
    // once so one typo cannot be reported once per `paths:` block.
    const targets = [];
    for (const name of run.tests) {
      const file = testTargetFileFor(run, name);
      if (!file) {
        const where = run.crates.join("|");
        violations.push(
          `phase ${run.phase}: \`--test ${name}\` has no tests/${name}.rs under \`${where}\``,
        );
        continue;
      }
      targets.push(file);
    }
    for (const name of run.crates) {
      const crate = locateCrate(name);
      if (!crate) {
        violations.push(`phase ${run.phase}: crate \`${name}\` has no Cargo.toml in this repo`);
        continue;
      }
      const required = crateEntries(crate);
      for (const block of blocks) {
        for (const entry of required) {
          if (!block.entries.includes(entry)) {
            violations.push(
              `on.${block.event}.paths: phase ${run.phase} runs crate \`${name}\` but there is no "${entry}" entry`,
            );
          }
        }
      }
    }
    for (const block of blocks) {
      for (const file of targets) {
        if (!covered(block.entries, file)) {
          violations.push(
            `on.${block.event}.paths: phase ${run.phase} runs "${file}" but no entry covers it`,
          );
        }
      }
    }
  }
  return { violations, blocks, runs };
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures - both directions
// ─────────────────────────────────────────────────────────────────────────────

const FIXTURE_GREEN = `on:
  push:
    paths:
      - "Cargo.toml"
      - "build.rs"
      - "src/**"
      - "tests/**"
  pull_request:
    paths:
      - "Cargo.toml"
      - "build.rs"
      - "src/**"
      - "tests/**"
jobs:
  j:
    steps:
      - run: |
          run_phase 01-unit scoped 'test(snapshot)' \\
            -p vantadb --lib --features failpoints
`;

const FIXTURE_RED_CRATE = FIXTURE_GREEN.replace(
  "          run_phase 01-unit",
  "          run_phase 08-mcp scoped 'test(snapshot)' \\\n            -p vantadb-mcp --test mcp_tests\n" +
    "          run_phase 01-unit",
);

const FIXTURE_RED_ENTRY = FIXTURE_GREEN.replace('      - "src/**"\n', "");

function runSelfTest() {
  const cases = [
    // One violation per (missing entry x paths block): both blocks are audited,
    // so a crate that no block covers from the push side does not pass because
    // the pull_request side happens to carry it.
    ["green: phases and paths agree", FIXTURE_GREEN, 0, []],
    ["red: crate added to a phase, absent from paths", FIXTURE_RED_CRATE, 6, []],
    // 1 derived entry gone from `push` + 1 asymmetry against the `pull_request` block.
    ["red: src/** dropped from the push block", FIXTURE_RED_ENTRY, 2, []],
    // The lane must list itself and this gate: without it, deleting that entry
    // turns the gate off and nothing fails.
    [
      "red: the lane does not watch itself",
      FIXTURE_GREEN,
      2,
      [".github/workflows/snapshot-certification.yml"],
    ],
  ];
  let failed = 0;
  for (const [name, fixture, expected, selfPaths] of cases) {
    const { violations } = audit(fixture, { selfPaths });
    const ok = violations.length === expected;
    if (!ok) failed++;
    console.log(
      `${ok ? "ok  " : "FAIL"} ${name} (expected ${expected} violation(s), got ${
        violations.length
      })`,
    );
    if (!ok) for (const v of violations) console.log(`       ${v}`);
  }
  console.log(`\ncert-lane-paths self-test: ${cases.length - failed}/${cases.length}`);
  return failed === 0 ? 0 : 1;
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

function main() {
  let text;
  try {
    text = readFileSync(WORKFLOW, "utf8");
  } catch (err) {
    console.error(`cert-lane-paths: cannot read ${WORKFLOW}: ${err.message}`);
    return 2;
  }

  // Derived, never a literal list: the lane must watch the file it is run from and
  // the script doing the check. When `--workflow` points at a copy outside the repo
  // (a mutation test) the copy cannot be named by an entry, so the real lane path is
  // used - the self-requirement is a property of the lane this gate ships in, not of
  // whatever file is being audited.
  const repoPath = (file) => relative(ROOT, file).replace(/\\/g, "/");
  const inside = (p) => p && !p.startsWith("..");
  const lane = inside(repoPath(WORKFLOW))
    ? repoPath(WORKFLOW)
    : ".github/workflows/snapshot-certification.yml";
  const selfPaths = [lane, repoPath(fileURLToPath(import.meta.url))].filter(inside);

  const { violations, blocks, runs } = audit(text, { selfPaths });
  if (violations.length) {
    console.error(`FAIL cert-lane-paths (FIND-797) - ${WORKFLOW}`);
    for (const v of violations) console.error(`  - ${v}`);
    console.error(`\n${violations.length} cert-lane-paths violation(s)`);
    return 1;
  }

  const crates = [...new Set(runs.flatMap((r) => r.crates))].sort().join(", ");
  console.log(
    `ok   cert-lane-paths (FIND-797): ${runs.length} phase(s), ${blocks.length} paths block(s), crates: ${crates}`,
  );
  return 0;
}

// Only run when invoked directly: `--self-test`, and the cost measurement that
// imports `pathBlocks`, both need the module without its main.
const invokedDirectly =
  process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (invokedDirectly) process.exit(selfTest ? runSelfTest() : main());
