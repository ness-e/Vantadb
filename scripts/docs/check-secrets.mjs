#!/usr/bin/env node
// scripts/docs/check-secrets.mjs
//
// PURPOSE
//   Secret-leak gate for the published documentation tree. The docs are browsable
//   on GitHub and an agent-authored documentation pass has just rewritten every
//   file, so the failure this exists to catch is "a credential was written into a
//   document" -- not "a credential is in the environment". The April 2025 CISA
//   advisory is the reason: a private repository was made public by accident and
//   attackers scraped leaked credentials within hours. A public docs tree is a
//   public secret surface, and a leak found in CI costs one commit.
//
//   Zero dependencies, fully offline, Node >= 18. Runs in ~1s on 1649 files, so
//   it is cheap enough for every PR that touches documentation.
//
// WHY PROSE ONLY
//   Everything below is matched against `segment()`'s safe segments only, never
//   the raw file. Frontmatter YAML, fenced code blocks and inline code spans are
//   excluded, and that is not a convenience: docs/ legitimately contains example
//   API keys, placeholder tokens and test fixtures inside code blocks. A scanner
//   that reads code blocks reports every one of them on day one, and a gate that
//   is red on day one gets switched off.
//
//   The same logic already lives in lib.mjs for the link gates, which is why
//   this imports `segment()` rather than reimplementing fence tracking. `proseOf()`
//   is the same filter but throws the line numbers away, and a finding you cannot
//   navigate to is a finding nobody fixes -- so this walks the segments directly
//   and carries the source offset through.
//
// WHAT IS CHECKED
//   AWS access key id (AKIA/ASIA), AWS secret access key, Google API key (AIza),
//   GitHub token (ghp_/gho_/ghu_/ghs_/ghr_/github_pat_), Slack token (xox[baprs]-),
//   Stripe live key (sk_live_/rk_live_), OpenAI/Anthropic style key (sk-, sk-ant-),
//   password/secret/token/api_key assigned a non-placeholder literal in prose,
//   and raw internal hostnames (*.internal|*.corp|*.local|*.lan, RFC1918).
//
// SEVERITY
//   error   - a credential that would authenticate. Budget 0. These are the ones
//             that must fail CI the moment they appear.
//   warning - a shape that is sometimes legitimate. A PEM block is real material
//             in a document about TLS; an internal hostname is a real material in a
//             design note. Reported, budgeted, never red on its own.
//
// NOT CHECKED
//   - Anything inside code blocks or frontmatter (see above, deliberately).
//   - Generic high-entropy strings. Measured on 2026-09-29: a 24-char-mixed-class
//     entropy sweep over prose returns 241 distinct strings, and every one is a
//     file path, a URL, a session id or an embedded base64 PNG. An entropy rule
//     here would be ~100% false positive and would bury the real signal.
//   - Secrets in code, config, git history, or the environment. This gate is
//     about documentation; the other surfaces need a different tool.
//   - Whether a credential is LIVE. Nothing here calls a provider to find out.
//
// USAGE
//   node scripts/docs/check-secrets.mjs                    # gate
//   node scripts/docs/check-secrets.mjs --json             # machine-readable
//   node scripts/docs/check-secrets.mjs --self-test        # rule assertions, no scan
//   node scripts/docs/check-secrets.mjs --max-errors=N     # tolerated errors
//   node scripts/docs/check-secrets.mjs --max-warnings=N   # tolerated warnings
//
// EXIT CODES
//   0 = findings within budget
//   1 = a budget was exceeded, or --self-test failed
//   2 = bad usage, or a malformed allowlist

import { readFileSync, existsSync } from 'node:fs';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { listDocs, readDoc, abs, segment } from './lib.mjs';

// --------------------------------------------------------------------- flags

const ARGS = process.argv.slice(2);
const flag = (name) => ARGS.includes(name);

/** Every .md under a directory, recursively. Only used for --root fixtures. */
const walkMd = (dir) => {
  const out = [];
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name === 'node_modules' || e.name === '.git') continue;
      out.push(...walkMd(p));
    } else if (e.name.toLowerCase().endsWith('.md')) {
      out.push(p);
    }
  }
  return out;
};
const numFlag = (name, dflt) => {
  const hit = ARGS.find((a) => a.startsWith(`--${name}=`));
  if (!hit) return dflt;
  const n = Number(hit.split('=')[1]);
  if (!Number.isFinite(n) || n < 0) {
    console.error(`--${name}= expects a non-negative number, got "${hit}"`);
    process.exit(2);
  }
  return n;
};

const JSON_OUT = flag('--json');
const SELF_TEST = flag('--self-test');

/**
 * Findings, tolerated. Counted per SEVERITY, and the default is the number
 * measured on the corpus the day the gate was written.
 *
 * 2026-09-29: errors 0, warnings 28. Zero errors is the point -- every provider
 * key shape is at zero across 1649 files, including the ~1000 task files the
 * agent pass rewrote.
 *
 * The 28 warnings are all `internal-hostname`, in 9 research notes, and they are
 * anonymisation placeholders rather than real infrastructure: docs.lan (13),
 * blog.lan (4), www.lan (3), info.lan (3), smith.lan (2), reference.lan (1) and
 * ipc.local (2, the desktop IPC endpoint proposed in the quickwins research).
 * They are reported rather than suppressed because the count is the thing worth
 * watching: if a future note names a host that is NOT one of those seven, it is
 * a real disclosure and the number going up is the signal. Lower it as those
 * notes get redacted.
 *
 * Warnings are budgeted separately from errors precisely so this tolerance never
 * softens the error budget. `--max-warnings=28` cannot make an AKIA key pass.
 */
const ERROR_BUDGET = numFlag('max-errors', 0);
const WARNING_BUDGET = numFlag('max-warnings', 28);

// --------------------------------------------------------------------- rules

/**
 * Values that are obviously not a credential even though they sit after a
 * `password =` / `token:` in prose. Kept as one list because the corpus is
 * bilingual: the Spanish words are here for the same reason the English ones are
 * (measured: `token: humo`, `secret: corregido`).
 */
const PLACEHOLDER =
  /^(?:string|str|text|number|int|integer|float|bool|boolean|object|list|array|dict|map|any|null|none|nil|true|false|optional|required|value|name|type|path|url|empty|default|unset|skip|unused|todo|fixme|same|as[_-]?above|change[_-]?me|placeholder|example|redacted|hidden|secret|password|token|apikey|api[_-]?key|session|identifier|\bid\b|user|admin|key|credencial|clave|contrase[nñ]a|secreto|t[eé]rmino|humo|valor|nada)$/i;

/**
 * A credential literal has to survive all of these to be reported:
 *   - long enough to be a credential and short enough not to be a paragraph
 *   - not a placeholder word, not a run of X, not a template reference
 *   - a lowercase letter AND an upper-case-or-digit AND a digit
 * The three-class requirement is what kills `token: String`, `token: write` and
 * `token: get_token` without a word list per language. It is also why a
 * dictionary word that happens to sit after a colon is not a finding.
 */
const isCredentialLiteral = (v) =>
  v.length >= 8 &&
  v.length <= 120 &&
  !PLACEHOLDER.test(v) &&
  !/[xX]{4,}/.test(v) &&
  !/^(?:<|\{|\$)/.test(v) &&
  /[a-z]/.test(v) &&
  /[A-Z0-9]/.test(v) &&
  /\d/.test(v) &&
  !/^[A-Za-z]+$/.test(v);

/** A 40-char AWS secret always mixes cases and carries a digit; prose does not. */
const isAwsSecretShape = (v) => v.length === 40 && /[a-z]/.test(v) && /[A-Z]/.test(v) && /\d/.test(v);

/**
 * `internal-hostname` is a finding whose value IS the hostname, so it is the one
 * rule whose excerpt is not redacted -- masking the evidence of a rule that
 * reports the evidence makes the report unactionable. It is not a credential.
 */
const RULES = [
  { name: 'aws-access-key-id', severity: 'error', re: /\b(?:AKIA|ASIA)[0-9A-Z]{16}\b/g, secret: true },
  {
    name: 'aws-secret-access-key',
    severity: 'error',
    re: /(?<![A-Za-z0-9/+=])[A-Za-z0-9/+=]{40}(?![A-Za-z0-9/+=])/g,
    // `keep` receives the MATCH, not the matched text. Passing the predicate
    // straight through was a dead rule until --self-test caught it: a bare 40-char
    // run in prose is usually a path or a class name, not a secret.
    keep: (m) => isAwsSecretShape(m[0]),
    secret: true,
  },
  { name: 'google-api-key', severity: 'error', re: /AIza[0-9A-Za-z_-]{35}(?![A-Za-z0-9_-])/g, secret: true },
  {
    name: 'github-token',
    severity: 'error',
    re: /\b(?:ghp|gho|ghu|ghs|ghr)_[0-9A-Za-z]{20,}\b|\bgithub_pat_[0-9A-Za-z_]{20,}(?![A-Za-z0-9_])/g,
    secret: true,
  },
  { name: 'slack-token', severity: 'error', re: /\bxox[baprs]-[0-9A-Za-z-]{10,}\b/g, secret: true },
  { name: 'stripe-live-key', severity: 'error', re: /\b(?:sk|rk)_live_[0-9A-Za-z]{10,}\b/g, secret: true },
  { name: 'llm-provider-key', severity: 'error', re: /\bsk-(?:ant-)?[A-Za-z0-9_-]{20,}(?![A-Za-z0-9_-])/g, secret: true },
  {
    name: 'credential-assignment',
    severity: 'error',
    re: /\b(?:password|passwd|secret|token|api[_-]?key|auth[_-]?token)\b\s*[:=]\s*["']?([A-Za-z0-9_+./=!@#$%^&*-]{4,})/gi,
    keep: (m) => isCredentialLiteral(m[1]),
    secret: true,
  },
  {
    // PEM material is a warning, not an error: a document about TLS legitimately
    // shows the block header. The gate's job is to make it a declared decision.
    name: 'private-key-pem',
    severity: 'warning',
    re: /-----BEGIN (?:[A-Z ]*)PRIVATE KEY-----/g,
    secret: true,
  },
  {
    // RFC1918 only (10/8, 172.16/12, 192.168/16). 127.0.0.0/8 is deliberately
    // absent: loopback is not private, it is "your own machine", and docs/api/
    // HTTP_API.md legitimately uses it in prose on 58 lines.
    name: 'internal-hostname',
    severity: 'warning',
    re: /\bhttps?:\/\/([A-Za-z0-9-]*\.(?:internal|corp|local|lan)|(?:10(?:\.[0-9]{1,3}){3}|172\.(?:1[6-9]|2\d|3[01])(?:\.[0-9]{1,3}){2}|192\.168(?:\.[0-9]{1,3}){2}))/g,
    secret: false,
  },
];

// ----------------------------------------------------------------- redaction

/**
 * Mask the middle, keep enough to locate the finding. Never print the whole
 * value: this output lands in CI logs, which get pasted into issues.
 */
function redact(s) {
  if (s.length <= 8) return `${s.slice(0, 2)}${'*'.repeat(Math.max(1, s.length - 4))}${s.slice(-2)}`;
  return `${s.slice(0, 4)}${'*'.repeat(Math.min(12, s.length - 8))}${s.slice(-4)}`;
}

// ----------------------------------------------------------------- allowlist

const ALLOWLIST = abs('scripts/docs/secrets-allowlist.json');

/**
 * Load declared exemptions.
 *
 * The allowlist is how a deliberate security-documentation example is DECLARED
 * rather than silenced: every entry carries a non-empty `reason`, so "why is this
 * allowed" is answered in the repository rather than in someone's memory. An
 * entry without a reason is a hard error, not a warning -- a gate whose exemption
 * mechanism accepts an unexplained entry is a gate with a mute button.
 *
 * `pattern` is matched against the RAW matched text (never the redacted excerpt,
 * which would be unmatchable). `file` may be `*` for repo-wide. `rule`, if given,
 * narrows the entry to one rule.
 */
function loadAllowlist() {
  if (!existsSync(ALLOWLIST)) return [];
  let doc;
  try {
    doc = JSON.parse(readFileSync(ALLOWLIST, 'utf8'));
  } catch (e) {
    console.error(`secrets-allowlist.json is not valid JSON: ${e.message}`);
    process.exit(2);
  }
  const entries = doc.entries ?? doc;
  if (!Array.isArray(entries)) {
    console.error('secrets-allowlist.json must be { "entries": [ ... ] }');
    process.exit(2);
  }
  return entries.map((e, i) => {
    const where = `secrets-allowlist.json entry #${i + 1}`;
    if (!e.pattern) {
      console.error(`${where}: missing "pattern"`);
      process.exit(2);
    }
    if (!e.reason || !String(e.reason).trim()) {
      console.error(`${where}: "reason" is required and must be non-empty. An unexplained exemption is a mute button.`);
      process.exit(2);
    }
    try {
      e.re = new RegExp(e.pattern);
    } catch (err) {
      console.error(`${where}: "pattern" is not a valid regex: ${err.message}`);
      process.exit(2);
    }
    return { file: e.file || '*', rule: e.rule || null, re: e.re, reason: String(e.reason).trim() };
  });
}

const isAllowlisted = (entries, file, rule, raw) =>
  entries.some((e) => (e.file === '*' || e.file === file) && (!e.rule || e.rule === rule) && e.re.test(raw));

// ------------------------------------------------------------------- scanning

/**
 * Prose segments WITH their line numbers.
 *
 * `segment()` returns contiguous slices of the source in order, so tracking the
 * running offset is enough to recover a 1-based line for each one -- which
 * `proseOf()` throws away. Hitting a rule inside a segment then adds the newlines
 * that precede the match.
 */
function proseWithLines(src) {
  const out = [];
  let pos = 0;
  for (const seg of segment(src)) {
    const line = src.slice(0, pos).split('\n').length;
    if (seg.safe && seg.why === 'prose') out.push({ line, text: seg.text });
    pos += seg.text.length;
  }
  return out;
}

const ALLOW = loadAllowlist();

const findings = [];
const byRule = Object.fromEntries(RULES.map((r) => [r.name, 0]));
let suppressed = 0;

// The corpus scan and the tallies run at the bottom of the file, after --self-test
// has had its chance to exit: the self-test is the cheaper job and the one that
// must not be skipped, and it does not need the file list at all. The tallies have
// to sit with the scan for the same reason -- `findings` is still empty up here.

// ------------------------------------------------------------------ self-test

/**
 * One assertion per rule that must fire, plus one per shape that must not. The
 * corpus measured 0 hits on every provider-key rule, so without this the gate
 * would be indistinguishable from a scanner that matches nothing -- a clean run
 * proves nothing when the pattern set has never been shown to match a real key.
 * This is the runnable check; no framework, no fixtures.
 */
const SELF_TEST_CASES = [
  // [text, rule that must fire, rule that must NOT fire]
  ['Rotate AKIAIOSFODNN7EXAMPLE before the demo.', 'aws-access-key-id', 'credential-assignment'],
  ['The id was ASIA2QWERTYUIOPASDFG in the console.', 'aws-access-key-id', null],
  ['Set it to wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY now.', 'aws-secret-access-key', null],
  ['Key AIzaSyD1a2B3c4D5e6F7g8H9i0JkLmNoPqRsTUV is in the env.', 'google-api-key', null],
  ['Use ghp_16C7e42F292c6912E7710c838347Ae178B4a for the release.', 'github-token', null],
  ['The classic token is gho_16C7e42F292c6912E7710c838347Ae178B4a.', 'github-token', null],
  ['Fine-grained: github_pat_11ABCDEFG0aBcDeFgHiJkL_mNoPqRsTuVwXyZ0123.', 'github-token', null],
  ['Post to xoxb-123456789012-1234567890123-AbCdEfGhIjKlMnOpQrStUvWx.', 'slack-token', null],
  ['Charge with sk_live_4eC39HqLyjWDarjtT1zdp7dc.', 'stripe-live-key', null],
  ['Restricted key rk_live_4eC39HqLyjWDarjtT1zdp7dc is worse than none.', 'stripe-live-key', null],
  ['Call the API with sk-proj-abc123DEF456ghi789JKL012mno345PQR678stu.', 'llm-provider-key', null],
  ['Anthropic uses sk-ant-api03-AbCdEf0123456789GhIjKlMnOpQrStUv.', 'llm-provider-key', null],
  ['Generate one with openssl and you get a -----BEGIN RSA PRIVATE KEY----- header.', 'private-key-pem', 'credential-assignment'],
  ['The password = CorrectHorse9Battery is in the vault.', 'credential-assignment', null],
  // The three shapes that killed the first draft of the credential rule. Each one
  // is a real line in the corpus, and each one is not a secret.
  ['The doc says token: None when the session expires.', null, 'credential-assignment'],
  ['Type the token: String in the signature and it compiles.', null, 'credential-assignment'],
  ['Rotate the token every 90 days, see docs/user/operations/DEPLOYMENT_GUIDE.md.', null, 'credential-assignment'],
  ['The endpoint is https://api.internal.example.com/v1 in production.', 'internal-hostname', null],
  ['Mirror the index at http://buildbox.corp/db/index for the mirror job.', 'internal-hostname', null],
  ['The replica sits at https://10.42.7.19:8443 and is only reachable on VPN.', 'internal-hostname', null],
  // Must NOT fire anywhere: these are the shapes the corpus is full of.
  ['AKIA alone is not an access key id.', null, 'aws-access-key-id'],
  ['The sk-fake value in the fixture is a placeholder.', null, 'llm-provider-key'],
  ['Publish to http://127.0.0.1:8080/v1 for the local run.', null, 'internal-hostname'],
  ['Bind the server to 0.0.0.0:3000 when developing.', null, 'internal-hostname'],
  ['The example password is pypi-XXXXXXXXXXXXXXXXXXXXXXXX.', null, 'credential-assignment'],
  ['Docs point at https://github.com/anthropics/anthropic-sdk-python.', null, 'github-token'],
  ['Stripe also has test keys: sk_test_4eC39HqLyjWDarjtT1zdp7dc.', null, 'stripe-live-key'],
  ['Slack test tokens start with xoxb too, but the live one is what matters.', null, 'slack-token'],
  ['A random sentence with no credential shape whatsoever in it at all.', null, 'credential-assignment'],
];

function runSelfTest() {
  let pass = 0;
  const failures = [];
  for (const [text, mustHit, mustNotHit] of SELF_TEST_CASES) {
    const prose = proseWithLines(text).map((p) => p.text).join('');
    const fired = new Set();
    for (const rule of RULES) {
      rule.re.lastIndex = 0;
      for (const m of prose.matchAll(rule.re)) {
        if (rule.keep && !rule.keep(m)) continue;
        fired.add(rule.name);
      }
    }
    if (mustHit && !fired.has(mustHit)) {
      failures.push(`expected ${mustHit} to fire on: ${text}`);
    } else if (mustNotHit && fired.has(mustNotHit)) {
      failures.push(`expected ${mustNotHit} NOT to fire on: ${text}`);
    } else {
      pass++;
    }
  }
  console.log(`self-test            : ${pass}/${SELF_TEST_CASES.length} assertions`);
  for (const f of failures) console.log(`  FAIL  ${f}`);
  console.log(
    failures.length ? '\nSELF-TEST FAILED' : '\nOK: every rule fires on its shape and stays quiet on the corpus\'s real shapes.',
  );
  process.exit(failures.length ? 1 : 0);
}

// --------------------------------------------------------------------- scan

if (SELF_TEST) runSelfTest();

// --root=<dir>  Scan an arbitrary tree instead of docs/.
//
// The measured corpus reports zero credential hits, which means a clean run
// proves nothing on its own: a gate whose every rule matches nothing looks
// exactly like a gate that works. Pointing the scanner at a fixture full of
// synthetic credentials is the only way to tell those two apart, so the root is
// overridable.
const ROOT = (() => {
  const hit = ARGS.find((a) => a.startsWith('--root='));
  return hit ? hit.split('=').slice(1).join('=') : null;
})();

const files = ROOT ? walkMd(ROOT) : listDocs({ includeArchive: true });
if (!ROOT && existsSync(abs('README.md'))) files.push('README.md');

for (const rel of files) {
  let src;
  try {
    // With --root the paths are absolute and must be read as given; readDoc()
    // resolves against docs/ and would silently read nothing.
    src = ROOT ? readFileSync(rel, 'utf8') : readDoc(rel);
  } catch {
    continue; // an unreadable file is not a leak
  }
  for (const { line, text } of proseWithLines(src)) {
    for (const rule of RULES) {
      rule.re.lastIndex = 0;
      for (const m of text.matchAll(rule.re)) {
        if (rule.keep && !rule.keep(m)) continue;
        if (isAllowlisted(ALLOW, rel, rule.name, m[0])) {
          suppressed++;
          continue;
        }
        byRule[rule.name]++;
        findings.push({
          file: rel,
          line: line + (text.slice(0, m.index).match(/\n/g)?.length ?? 0),
          rule: rule.name,
          severity: rule.severity,
          excerpt: rule.secret ? redact(m[0]) : m[0],
        });
      }
    }
  }
}

const errors = findings.filter((f) => f.severity === 'error').length;
const warnings = findings.filter((f) => f.severity === 'warning').length;
const overErrors = errors > ERROR_BUDGET;
const overWarnings = warnings > WARNING_BUDGET;

// --------------------------------------------------------------------- output

if (JSON_OUT) {
  console.log(
    JSON.stringify(
      { filesScanned: files.length, byRule, errors, warnings, budget: { errors: ERROR_BUDGET, warnings: WARNING_BUDGET }, allowlisted: suppressed, failed: overErrors || overWarnings, findings },
      null,
      2,
    ),
  );
} else {
  console.log(`files scanned        : ${files.length} (docs/ + README.md if present)`);
  console.log(`rules                : ${RULES.length} (${RULES.filter((r) => r.severity === 'error').length} error, ${RULES.filter((r) => r.severity === 'warning').length} warning)`);
  console.log(`allowlisted          : ${suppressed}`);
  console.log(`ERRORS               : ${errors} (budget ${ERROR_BUDGET})${overErrors ? '  <-- OVER BUDGET' : ''}`);
  console.log(`warnings             : ${warnings} (budget ${WARNING_BUDGET})${overWarnings ? '  <-- OVER BUDGET' : ''}`);

  const nonZero = Object.entries(byRule).filter(([, n]) => n > 0);
  if (nonZero.length) {
    console.log('\nBY RULE:');
    for (const [name, n] of nonZero.sort((a, b) => b[1] - a[1])) {
      const sev = RULES.find((r) => r.name === name).severity;
      console.log(`  ${String(n).padStart(4)}  ${sev.padEnd(7)} ${name}`);
    }
  } else {
    console.log('\nOK: no credential shape in prose.');
  }

  if (findings.length) {
    const byFile = new Map();
    for (const f of findings) {
      if (!byFile.has(f.file)) byFile.set(f.file, []);
      byFile.get(f.file).push(f);
    }
    console.log(`\nFINDINGS (${findings.length} in ${byFile.size} files), excerpts redacted:`);
    for (const [file, items] of [...byFile].sort((a, b) => b[1].length - a[1].length)) {
      console.log(`  ${file}`);
      for (const it of items.slice(0, 6)) {
        console.log(`      L${it.line}  [${it.severity}] ${it.rule}  ${it.excerpt}`);
      }
      if (items.length > 6) console.log(`      ... +${items.length - 6} more`);
    }
  }

  if (overErrors) {
    console.log(
      `\nOVER BUDGET: ${errors} errors > ${ERROR_BUDGET}. A credential-shaped string in PROSE fails the gate by design. ` +
        'If it is a genuine example, declare it in scripts/docs/secrets-allowlist.json WITH a reason.',
    );
  }
  if (overWarnings) {
    console.log(`\nOVER BUDGET: ${warnings} warnings > ${WARNING_BUDGET}. Lower WARNING_BUDGET as the research notes get redacted.`);
  }
  if (!overErrors && !overWarnings && warnings) {
    console.log(
      `\nwithin budget: ${warnings} of ${WARNING_BUDGET} warnings tolerated. These are internal hostnames, ` +
        'not credentials. They gate only if the count rises.',
    );
  }
}

process.exit(overErrors || overWarnings ? 1 : 0);
