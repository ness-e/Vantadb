#!/usr/bin/env node
// Registry statistics for every published VantaDB package (npm + PyPI),
// rendered as a self-contained HTML report.
//
// What it collects, per package:
//   - published version + release history (date, size, yanked?)
//   - downloads: npm  = api.npmjs.org (1d / 7d / 30d / 365d + daily series)
//                 PyPI = pypistats.org daily series (mirrors excluded); the
//                 windows are computed from that series because its `recent`
//                 endpoint reports last_month = 0 for several of our packages
//   - local manifest version -> drift alert when it differs from the published one
//
// Package list is DISCOVERED from the repo (npm package.json / Cargo.toml /
// pyproject.toml), so adding an adapter needs no edit here. Published names
// that have no local manifest (e.g. the standalone `vanta-cli` npm package)
// live in EXTRA_NPM.
//
// Usage: node dev-tools/package-stats.mjs [--out <dir>] [--days 90] [--open]
// Writes: <out>/package-stats.html and <out>/package-stats.json
// Exit:   0 = report written (even with partial failures, which are listed
//         in the report); 1 = nothing could be collected.

import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));

// ── CLI ────────────────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
const arg = (flag, fallback) => {
  const i = argv.indexOf(flag);
  return i >= 0 && argv[i + 1] ? argv[i + 1] : fallback;
};
const OUT_DIR = path.resolve(ROOT, arg("--out", path.join("dev-tools", "reports")));
const DAYS = Math.max(7, Math.min(365, Number(arg("--days", 90))));
const OPEN = argv.includes("--open");

// Published npm packages with no manifest inside this repo.
const EXTRA_NPM = [{ name: "vanta-cli", note: "CLI publicado fuera de este repo" }];

// ── local manifest discovery ───────────────────────────────────────────────
const readJson = (p) => JSON.parse(readFileSync(p, "utf8"));
const readText = (p) => readFileSync(p, "utf8");

function cargoVersion(dir) {
  const manifest = path.join(ROOT, dir, "Cargo.toml");
  if (!existsSync(manifest)) return null;
  const text = readText(manifest);
  if (/version\.workspace\s*=\s*true/.test(text)) {
    const root = readText(path.join(ROOT, "Cargo.toml"));
    const section = root.split("[workspace.package]")[1] ?? "";
    const m = section.match(/^version\s*=\s*"([^"]+)"/m);
    return m ? m[1] : null;
  }
  const m = text.match(/^version\s*=\s*"([^"]+)"/m);
  return m ? m[1] : null;
}

function packageJsonVersion(dir) {
  const manifest = path.join(ROOT, dir, "package.json");
  if (!existsSync(manifest)) return null;
  try {
    return readJson(manifest).version ?? null;
  } catch {
    return null;
  }
}

function pyprojectInfo(dir) {
  const manifest = path.join(ROOT, dir, "pyproject.toml");
  if (!existsSync(manifest)) return null;
  const text = readText(manifest);
  const name = text.match(/^\[project\][\s\S]*?^name\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  const version = text.match(/^\[project\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  return name ? { name, version } : null;
}

const localVersion = (dir) =>
  packageJsonVersion(dir) ?? cargoVersion(dir) ?? pyprojectInfo(dir)?.version ?? null;

// npm: package.json manifests (vantadb-wasm's npm manifest lives in a
// gitignored build dir, so its crate version is the source of truth).
const NPM_DIRS = ["vantadb-ts", "vantadb-node"];
const WASM_DIR = "vantadb-wasm";

// PyPI: every pyproject under integrations/ and providers/ + the core SDK.
const pypiDirs = ["vantadb-python"];
for (const group of ["integrations", "providers"]) {
  const base = path.join(ROOT, group);
  if (!existsSync(base)) continue;
  for (const entry of readdirSync(base, { withFileTypes: true })) {
    if (entry.isDirectory() && existsSync(path.join(base, entry.name, "pyproject.toml"))) {
      pypiDirs.push(`${group}/${entry.name}`);
    }
  }
}

const discovered = [];
for (const dir of NPM_DIRS) {
  const manifest = path.join(ROOT, dir, "package.json");
  if (!existsSync(manifest)) continue;
  discovered.push({ ecosystem: "npm", name: readJson(manifest).name, dir });
}
const wasmCargo = path.join(ROOT, WASM_DIR, "Cargo.toml");
if (existsSync(wasmCargo)) {
  const name = readText(wasmCargo).match(/^name\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  if (name) discovered.push({ ecosystem: "npm", name, dir: WASM_DIR });
}
for (const extra of EXTRA_NPM) {
  discovered.push({ ecosystem: "npm", name: extra.name, dir: null, note: extra.note });
}
for (const dir of pypiDirs) {
  const info = pyprojectInfo(dir);
  if (info?.name) discovered.push({ ecosystem: "pypi", name: info.name, dir });
}

// One registry name == one package; several local dirs may map to the same
// published name (integrations/ollama and providers/ollama -> vantadb-ollama).
const byName = new Map();
for (const pkg of discovered) {
  if (!pkg.name) continue;
  const key = `${pkg.ecosystem}:${pkg.name}`;
  const current = byName.get(key);
  if (!current) {
    byName.set(key, { ...pkg, dirs: pkg.dir ? [pkg.dir] : [] });
  } else if (pkg.dir && !current.dirs.includes(pkg.dir)) {
    current.dirs.push(pkg.dir);
  }
}
const packages = [...byName.values()].map((pkg) => ({
  ...pkg,
  localVersion: pkg.dirs.map(localVersion).find(Boolean) ?? null,
}));
packages.sort((a, b) => a.ecosystem.localeCompare(b.ecosystem) || a.name.localeCompare(b.name));

// ── report helpers ─────────────────────────────────────────────────────────
const esc = (value) =>
  String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");

const nf = (value) => (value ?? 0).toLocaleString("es-ES");
const dateOnly = (value) => (value ? String(value).slice(0, 10) : "—");
const bytes = (value) => {
  if (!value) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let n = value;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i += 1;
  }
  return `${n < 10 && i > 0 ? n.toFixed(1) : Math.round(n)} ${units[i]}`;
};

const SERIES_COLORS = [
  "#5eead4",
  "#a78bfa",
  "#fbbf24",
  "#f472b6",
  "#60a5fa",
  "#34d399",
  "#fb7185",
  "#c084fc",
  "#22d3ee",
  "#facc15",
  "#f97316",
  "#94a3b8",
];

const ecosystemAccent = { npm: "#cb3837", pypi: "#3776ab" };

// ── registry clients ───────────────────────────────────────────────────────
async function fetchJson(url, timeout = 25_000) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  try {
    const res = await fetch(url, {
      signal: controller.signal,
      headers: { accept: "application/json", "user-agent": "vantadb-package-stats" },
    });
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return await res.json();
  } finally {
    clearTimeout(timer);
  }
}

const npmDownloads = async (name, period) => {
  const url = `https://api.npmjs.org/downloads/point/${period}/${encodeURIComponent(name)}`;
  const json = await fetchJson(url);
  return json?.downloads ?? 0;
};

// Rolling windows over a daily series, anchored on the newest date present.
// Used for PyPI, whose `recent` endpoint reports last_month = 0 for several of
// our packages while the daily series holds the real numbers.
function windowSums(series) {
  const sorted = [...series].sort((a, b) => a.day.localeCompare(b.day));
  const lastDay = sorted.at(-1)?.day ?? null;
  const sumSince = (days) => {
    if (!lastDay) return 0;
    const cutoff = new Date(`${lastDay}T00:00:00Z`);
    cutoff.setUTCDate(cutoff.getUTCDate() - (days - 1));
    const iso = cutoff.toISOString().slice(0, 10);
    return sorted.filter((d) => d.day >= iso).reduce((acc, d) => acc + d.downloads, 0);
  };
  return {
    day: sorted.at(-1)?.downloads ?? 0,
    week: sumSince(7),
    month: sumSince(30),
    year: sumSince(365),
    history: { from: sorted[0]?.day ?? null, to: lastDay },
  };
}

const npmSeries = async (name, days) => {
  const end = new Date();
  end.setUTCDate(end.getUTCDate() - 1); // npm's series ends yesterday
  const start = new Date(end);
  start.setUTCDate(start.getUTCDate() - (days - 1));
  const fmt = (d) => d.toISOString().slice(0, 10);
  const url = `https://api.npmjs.org/downloads/range/${fmt(start)}:${fmt(end)}/${encodeURIComponent(name)}`;
  const json = await fetchJson(url);
  return (json?.downloads ?? []).map((d) => ({ day: d.day, downloads: d.downloads }));
};

async function collectNpm(pkg) {
  const packument = await fetchJson(`https://registry.npmjs.org/${encodeURIComponent(pkg.name)}`);
  if (!packument) throw new Error("no existe en el registry npm");
  const latest = packument["dist-tags"]?.latest ?? null;
  const versions = Object.keys(packument.versions ?? {});
  const releases = versions
    .map((version) => {
      const meta = packument.versions[version];
      return {
        version,
        date: packument.time?.[version] ?? null,
        bytes: meta?.dist?.unpackedSize ?? null,
        files: meta?.dist?.fileCount ?? null,
        yanked: Boolean(meta?.deprecated),
      };
    })
    .sort((a, b) => String(a.date).localeCompare(String(b.date)));

  const [day, week, month, year] = await Promise.all([
    npmDownloads(pkg.name, "last-day"),
    npmDownloads(pkg.name, "last-week"),
    npmDownloads(pkg.name, "last-month"),
    npmDownloads(pkg.name, "last-year"),
  ]);
  const series = await npmSeries(pkg.name, DAYS);
  const latestMeta = latest ? packument.versions[latest] : null;

  return {
    ...pkg,
    ecosystem: "npm",
    latest,
    publishedAt: latest ? (packument.time?.[latest] ?? null) : null,
    createdAt: packument.time?.created ?? null,
    downloads: { day, week, month, year },
    series,
    releases,
    bytes: latestMeta?.dist?.unpackedSize ?? null,
    files: latestMeta?.dist?.fileCount ?? null,
    license: latestMeta?.license ?? null,
    requires: latestMeta?.engines?.node ?? latestMeta?.engines?.bun ?? null,
    dependencies: Object.keys(latestMeta?.dependencies ?? {}),
    homepage: packument.homepage ?? latestMeta?.homepage ?? null,
    repository: packument.repository?.url ?? latestMeta?.repository?.url ?? null,
  };
}

async function collectPypi(pkg) {
  const meta = await fetchJson(`https://pypi.org/pypi/${encodeURIComponent(pkg.name)}/json`);
  if (!meta) throw new Error("no existe en PyPI");
  const overall = await fetchJson(`https://pypistats.org/api/packages/${encodeURIComponent(pkg.name)}/overall`);

  const releases = Object.entries(meta.releases ?? {})
    .map(([version, files]) => {
      const uploaded = files.map((f) => f.upload_time_iso_8601).filter(Boolean).sort();
      const sizes = files.map((f) => f.size ?? 0).filter(Boolean);
      return {
        version,
        date: uploaded[0] ?? null,
        bytes: sizes.length ? Math.max(...sizes) : null,
        files: files.length,
        yanked: files.some((f) => f.yanked),
      };
    })
    .sort((a, b) => String(a.date).localeCompare(String(b.date)));

  const series = (overall?.data ?? [])
    .filter((d) => d.category === "without_mirrors")
    .map((d) => ({ day: d.date, downloads: d.downloads }))
    .sort((a, b) => a.day.localeCompare(b.day));

  // pypistats' `recent` endpoint reports last_month = 0 for several of our
  // packages while its daily series has the real numbers -> compute the
  // windows from the series instead of trusting `recent`.
  const windows = windowSums(series);

  const latest = meta.info?.version ?? null;
  const latestFiles = meta.urls ?? [];

  return {
    ...pkg,
    ecosystem: "pypi",
    latest,
    publishedAt: latestFiles[0]?.upload_time_iso_8601 ?? null,
    createdAt: meta.info?.upload_time ?? null,
    downloads: {
      day: windows.day,
      week: windows.week,
      month: windows.month,
      year: windows.year,
    },
    series,
    history: windows.history,
    releases,
    bytes: latestFiles.length ? Math.max(...latestFiles.map((f) => f.size ?? 0)) : null,
    files: latestFiles.length,
    license: meta.info?.license ?? null,
    requires: meta.info?.requires_python ?? null,
    dependencies: meta.info?.requires_dist ?? [],
    homepage: meta.info?.home_page ?? meta.info?.project_url ?? null,
    repository: meta.info?.project_urls?.Repository ?? meta.info?.project_urls?.Homepage ?? null,
  };
}

// ── self-test (no network) ─────────────────────────────────────────────────
// Usage: node dev-tools/package-stats.mjs --self-test
function selfTest() {
  const assert = (ok, msg) => {
    if (!ok) throw new Error(`self-test: ${msg}`);
  };

  // 1. Rolling windows anchored on the newest date of the series.
  const series = [
    { day: "2026-10-01", downloads: 5 },
    { day: "2026-10-19", downloads: 7 },
    { day: "2026-10-20", downloads: 11 },
  ];
  const w = windowSums(series);
  assert(w.day === 11, `day should be the newest value, got ${w.day}`);
  assert(w.week === 18, `week should cover 10-14..10-20, got ${w.week}`);
  assert(w.month === 23, `month should cover 09-21..10-20, got ${w.month}`);
  assert(w.year === 23, `year should cover the whole series, got ${w.year}`);
  assert(w.history.from === "2026-10-01" && w.history.to === "2026-10-20", "history bounds");
  assert(windowSums([]).day === 0, "empty series must not throw");

  // 2. Workspace-inherited Cargo version (vantadb-wasm does not pin one).
  const wasm = cargoVersion("vantadb-wasm");
  const root = readText(path.join(ROOT, "Cargo.toml")).split("[workspace.package]")[1] ?? "";
  const expected = root.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  assert(wasm === expected, `vantadb-wasm version ${wasm} != workspace ${expected}`);

  // 3. Discovery finds the published packages of both ecosystems.
  const npmNames = packages.filter((p) => p.ecosystem === "npm").map((p) => p.name).sort();
  const pypiNames = packages.filter((p) => p.ecosystem === "pypi").map((p) => p.name).sort();
  assert(npmNames.includes("vantadb") && npmNames.includes("vantadb-node"), "npm discovery");
  assert(pypiNames.includes("vantadb-py"), "pypi discovery");
  assert(new Set(packages.map((p) => `${p.ecosystem}:${p.name}`)).size === packages.length, "duplicate names");

  // 4. Byte formatting.
  assert(bytes(1024) === "1.0 KB", `bytes(1024) = ${bytes(1024)}`);
  assert(bytes(null) === "—", "bytes(null)");

  console.log("  self-test: ok");
}

if (argv.includes("--self-test")) {
  selfTest();
  process.exit(0);
}

// ── collection ─────────────────────────────────────────────────────────────
const errors = [];
const collected = [];
for (const pkg of packages) {
  process.stdout.write(`  fetching ${pkg.ecosystem}/${pkg.name} ... `);
  try {
    const data = pkg.ecosystem === "npm" ? await collectNpm(pkg) : await collectPypi(pkg);
    collected.push(data);
    process.stdout.write("ok\n");
  } catch (err) {
    errors.push({ ecosystem: pkg.ecosystem, name: pkg.name, error: String(err?.message ?? err) });
    process.stdout.write(`FAILED (${err?.message ?? err})\n`);
  }
}

// ── derived metrics ────────────────────────────────────────────────────────
const sum = (values) => values.reduce((a, b) => a + (b ?? 0), 0);
const totals = {};
for (const ecosystem of ["npm", "pypi"]) {
  const list = collected.filter((p) => p.ecosystem === ecosystem);
  totals[ecosystem] = {
    packages: list.length,
    day: sum(list.map((p) => p.downloads.day)),
    week: sum(list.map((p) => p.downloads.week)),
    month: sum(list.map((p) => p.downloads.month)),
    year: sum(list.map((p) => p.downloads.year)),
  };
}

const drift = collected
  .filter((p) => p.localVersion && p.latest && p.localVersion !== p.latest)
  .map((p) => ({
    name: p.name,
    ecosystem: p.ecosystem,
    local: p.localVersion,
    published: p.latest,
  }));

const byDownloads = [...collected].sort(
  (a, b) => (b.downloads.month ?? 0) - (a.downloads.month ?? 0) || a.name.localeCompare(b.name),
);

// Stacked daily series per ecosystem (sum of every package per day).
function ecosystemChart(pkgList, colorFor) {
  if (!pkgList.length) return "";
  // Common window: the last DAYS days of the newest date present.
  const maxDay = pkgList.flatMap((p) => p.series.map((d) => d.day)).sort().at(-1);
  const cutoff = new Date(`${maxDay}T00:00:00Z`);
  cutoff.setUTCDate(cutoff.getUTCDate() - (DAYS - 1));
  const iso = cutoff.toISOString().slice(0, 10);
  const windowed = pkgList.map((p) => ({
    ...p,
    series: p.series.filter((d) => d.day >= iso),
  }));
  const days = [...new Set(windowed.flatMap((p) => p.series.map((d) => d.day)))].sort();
  if (!days.length) return '<p class="empty">Sin serie de descargas disponible.</p>';
  const totalsByDay = days.map((day) =>
    windowed.reduce((acc, p) => acc + (p.series.find((d) => d.day === day)?.downloads ?? 0), 0),
  );
  const max = Math.max(...totalsByDay, 1);
  const W = 900;
  const H = 260;
  const padL = 46;
  const padR = 12;
  const padT = 14;
  const padB = 26;
  const plotW = W - padL - padR;
  const plotH = H - padT - padB;
  const x = (i) => padL + (days.length === 1 ? plotW / 2 : (i / (days.length - 1)) * plotW);
  const y = (v) => padT + plotH - (v / max) * plotH;

  const line = totalsByDay.map((v, i) => `${i === 0 ? "M" : "L"}${x(i).toFixed(1)},${y(v).toFixed(1)}`).join(" ");
  const area = `${line} L${x(days.length - 1).toFixed(1)},${(padT + plotH).toFixed(1)} L${x(0).toFixed(1)},${(padT + plotH).toFixed(1)} Z`;

  const grid = [0, 0.25, 0.5, 0.75, 1]
    .map((f) => {
      const value = Math.round(max * f);
      const gy = y(value);
      return `<line x1="${padL}" y1="${gy.toFixed(1)}" x2="${W - padR}" y2="${gy.toFixed(1)}" class="grid"/><text x="${padL - 8}" y="${(gy + 4).toFixed(1)}" class="axis" text-anchor="end">${nf(value)}</text>`;
    })
    .join("");

  const ticks = [0, Math.floor((days.length - 1) / 2), days.length - 1]
    .map((i) => `<text x="${x(i).toFixed(1)}" y="${H - 6}" class="axis" text-anchor="middle">${days[i]}</text>`)
    .join("");

  const points = totalsByDay
    .map(
      (v, i) =>
        `<circle cx="${x(i).toFixed(1)}" cy="${y(v).toFixed(1)}" r="2.4" class="dot"><title>${days[i]} — ${nf(v)} descargas</title></circle>`,
    )
    .join("");

  const legend = pkgList
    .map((p) => {
      const color = colorFor(p.name);
      const value = windowed.find((w) => w.name === p.name)?.series.reduce((acc, d) => acc + d.downloads, 0) ?? 0;
      return `<li><span class="chip" style="background:${color}"></span><span class="chip-name">${esc(p.name)}</span><span class="chip-val">${nf(value)}</span></li>`;
    })
    .join("");

  return `
    <div class="chart">
      <svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Descargas diarias por paquete">
        <defs><linearGradient id="fill-${pkgList[0].ecosystem}" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="${ecosystemAccent[pkgList[0].ecosystem]}" stop-opacity="0.45"/>
          <stop offset="100%" stop-color="${ecosystemAccent[pkgList[0].ecosystem]}" stop-opacity="0.03"/>
        </linearGradient></defs>
        ${grid}
        <path d="${area}" fill="url(#fill-${pkgList[0].ecosystem})"/>
        <path d="${line}" fill="none" stroke="${ecosystemAccent[pkgList[0].ecosystem]}" stroke-width="2" stroke-linejoin="round"/>
        ${points}
        ${ticks}
      </svg>
      <ul class="legend">${legend}</ul>
    </div>`;
}

// Per-package bar sparkline (last 30 days of the collected series).
function sparkline(series) {
  const last30 = series.slice(-30);
  if (!last30.length) return '<span class="muted">—</span>';
  const max = Math.max(...last30.map((d) => d.downloads), 1);
  const W = 120;
  const H = 24;
  const barW = W / last30.length;
  const bars = last30
    .map((d, i) => {
      const h = Math.max(1, (d.downloads / max) * H);
      return `<rect x="${(i * barW).toFixed(2)}" y="${(H - h).toFixed(2)}" width="${Math.max(0.8, barW - 0.8).toFixed(2)}" height="${h.toFixed(2)}" rx="1"><title>${d.day} — ${nf(d.downloads)}</title></rect>`;
    })
    .join("");
  return `<svg class="spark" viewBox="0 0 ${W} ${H}" role="img" aria-label="Descargas 30 días">${bars}</svg>`;
}

const releaseTimeline = (pkg) => {
  if (!pkg.releases.length) return '<p class="empty">Sin historial de releases.</p>';
  const rows = [...pkg.releases]
    .reverse()
    .map(
      (r) => `<tr${r.version === pkg.latest ? ' class="latest"' : ""}>
        <td class="mono">${esc(r.version)}${r.yanked ? ' <span class="tag warn">yanked</span>' : ""}</td>
        <td>${dateOnly(r.date)}</td>
        <td class="num">${bytes(r.bytes)}</td>
        <td class="num">${r.files ?? "—"}</td>
      </tr>`,
    )
    .join("");
  return `<table class="releases"><thead><tr><th>Versión</th><th>Publicada</th><th class="num">Tamaño</th><th class="num">Ficheros</th></tr></thead><tbody>${rows}</tbody></table>`;
};

const packageCard = (pkg, color) => {
  const driftHere = pkg.localVersion && pkg.latest && pkg.localVersion !== pkg.latest;
  const pill = !pkg.localVersion
    ? '<span class="pill">sin local</span>'
    : driftHere
      ? '<span class="pill warn">⚠ drift</span>'
      : '<span class="pill ok">✓ en línea</span>';
  return `
  <article class="card" style="--accent:${color}">
    <header>
      <div>
        <h3>${esc(pkg.name)}</h3>
        <p class="muted">${pkg.ecosystem === "npm" ? "npm" : "PyPI"}${pkg.note ? ` · ${esc(pkg.note)}` : ""}</p>
      </div>
      <div class="versions">
        ${pill}
        <span class="mono muted">local ${esc(pkg.localVersion ?? "—")} · publicado ${esc(pkg.latest ?? "—")}</span>
      </div>
    </header>
    <div class="stats">
      <div><span class="k">1 día</span><span class="v">${nf(pkg.downloads.day)}</span></div>
      <div><span class="k">7 días</span><span class="v">${nf(pkg.downloads.week)}</span></div>
      <div><span class="k">30 días</span><span class="v">${nf(pkg.downloads.month)}</span></div>
      <div><span class="k">365 días</span><span class="v">${pkg.downloads.year === null ? "—" : nf(pkg.downloads.year)}</span></div>
    </div>
    <div class="meta">
      <span>Publicado: ${dateOnly(pkg.publishedAt)}</span>
      ${pkg.history?.from ? `<span>Histórico: ${dateOnly(pkg.history.from)} → ${dateOnly(pkg.history.to)}</span>` : ""}
      <span>Tamaño: ${bytes(pkg.bytes)}</span>
      <span>${esc(pkg.license ?? "sin licencia")}</span>
      <span>${esc(pkg.requires ?? "")}</span>
      <span>${pkg.dependencies.length} deps</span>
    </div>
    ${releaseTimeline(pkg)}
  </article>`;
};

// ── HTML ───────────────────────────────────────────────────────────────────
const now = new Date();
const npmPkgs = byDownloads.filter((p) => p.ecosystem === "npm");
const pypiPkgs = byDownloads.filter((p) => p.ecosystem === "pypi");
const colorOf = (name) => SERIES_COLORS[npmPkgs.concat(pypiPkgs).findIndex((p) => p.name === name) % SERIES_COLORS.length];

const tableRows = byDownloads
  .map((p) => {
    const driftHere = p.localVersion && p.latest && p.localVersion !== p.latest;
    return `<tr>
      <td><span class="chip" style="background:${colorOf(p.name)}"></span><span class="mono">${esc(p.name)}</span></td>
      <td class="muted">${p.ecosystem}</td>
      <td class="mono">${esc(p.localVersion ?? "—")}</td>
      <td class="mono">${esc(p.latest ?? "—")}</td>
      <td>${!p.localVersion ? '<span class="pill">—</span>' : driftHere ? '<span class="pill warn">drift</span>' : '<span class="pill ok">ok</span>'}</td>
      <td class="num">${nf(p.downloads.week)}</td>
      <td class="num">${nf(p.downloads.month)}</td>
      <td class="num">${p.downloads.year === null ? "—" : nf(p.downloads.year)}</td>
      <td class="num">${bytes(p.bytes)}</td>
      <td>${dateOnly(p.publishedAt)}</td>
      <td class="spark-cell">${sparkline(p.series)}</td>
    </tr>`;
  })
  .join("");

const errorBlock = errors.length
  ? `<section class="section"><h2>Fallos de recogida</h2><ul class="errors">${errors
      .map((e) => `<li><span class="mono">${esc(e.ecosystem)}/${esc(e.name)}</span> — ${esc(e.error)}</li>`)
      .join("")}</ul></section>`
  : "";

const html = `<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>VantaDB · estadísticas de paquetes</title>
<style>
  :root {
    --bg: #0b0d10;
    --panel: #12151b;
    --panel-2: #171b22;
    --line: #232833;
    --text: #e9ecf1;
    --muted: #8a93a3;
    --npm: #cb3837;
    --pypi: #3776ab;
    --ok: #34d399;
    --warn: #fbbf24;
  }
  * { box-sizing: border-box; }
  body {
    margin: 0; background: var(--bg); color: var(--text);
    font: 15px/1.55 ui-sans-serif, system-ui, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
  }
  .wrap { max-width: 1180px; margin: 0 auto; padding: 40px 24px 80px; }
  .mono { font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace; }
  .muted { color: var(--muted); }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  header.top { border-bottom: 1px solid var(--line); padding-bottom: 22px; margin-bottom: 28px; }
  h1 { font-size: 26px; margin: 0 0 6px; letter-spacing: -0.02em; }
  h2 { font-size: 15px; text-transform: uppercase; letter-spacing: 0.14em; color: var(--muted); margin: 0 0 14px; font-weight: 600; }
  h3 { font-size: 16px; margin: 0; }
  .sub { color: var(--muted); font-size: 13px; }
  .kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 12px; margin-bottom: 34px; }
  .kpi { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 14px 16px; }
  .kpi .k { display: block; font-size: 11px; text-transform: uppercase; letter-spacing: 0.1em; color: var(--muted); }
  .kpi .v { display: block; font-size: 26px; font-weight: 650; margin-top: 4px; font-variant-numeric: tabular-nums; }
  .kpi .s { display: block; font-size: 12px; color: var(--muted); margin-top: 2px; }
  .section { margin-bottom: 38px; }
  .chart { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 16px; }
  .chart svg { width: 100%; height: auto; display: block; }
  .grid { stroke: #232833; stroke-width: 1; }
  .axis { fill: #6b7484; font-size: 10px; font-family: ui-monospace, monospace; }
  .dot { fill: var(--text); }
  .legend { list-style: none; display: flex; flex-wrap: wrap; gap: 8px 18px; margin: 14px 0 0; padding: 0; font-size: 12px; }
  .legend li { display: flex; align-items: center; gap: 6px; }
  .chip { width: 9px; height: 9px; border-radius: 2px; display: inline-block; flex: none; }
  .chip-name { color: var(--text); }
  .chip-val { color: var(--muted); font-variant-numeric: tabular-nums; }
  table { width: 100%; border-collapse: collapse; font-size: 13px; }
  th { text-align: left; font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--muted); font-weight: 600; padding: 8px 10px; border-bottom: 1px solid var(--line); }
  td { padding: 9px 10px; border-bottom: 1px solid #1a1e26; vertical-align: middle; }
  tbody tr:hover { background: #14181f; }
  .table-wrap { overflow-x: auto; background: var(--panel); border: 1px solid var(--line); border-radius: 12px; }
  .pill { display: inline-block; padding: 2px 8px; border-radius: 999px; font-size: 11px; font-weight: 600; background: #1e2430; color: var(--muted); }
  .pill.ok { background: rgba(52,211,153,.14); color: var(--ok); }
  .pill.warn { background: rgba(251,191,36,.16); color: var(--warn); }
  .tag { font-size: 10px; padding: 1px 6px; border-radius: 4px; background: rgba(251,191,36,.16); color: var(--warn); }
  .spark { width: 120px; height: 24px; }
  .spark rect { fill: var(--muted); }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(430px, 1fr)); gap: 16px; }
  .card { background: var(--panel); border: 1px solid var(--line); border-left: 3px solid var(--accent, var(--line)); border-radius: 12px; padding: 16px 18px; }
  .card header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; margin-bottom: 12px; }
  .versions { text-align: right; font-size: 12px; }
  .stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-bottom: 12px; }
  .stats > div { background: var(--panel-2); border-radius: 8px; padding: 8px 10px; }
  .stats .k { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--muted); }
  .stats .v { display: block; font-size: 17px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .meta { display: flex; flex-wrap: wrap; gap: 6px 14px; font-size: 12px; color: var(--muted); margin-bottom: 12px; }
  .releases { font-size: 12px; }
  .releases tr.latest td { color: var(--text); font-weight: 600; }
  .empty, .errors { color: var(--muted); font-size: 13px; }
  .errors li { margin-bottom: 6px; }
  footer { color: var(--muted); font-size: 12px; border-top: 1px solid var(--line); padding-top: 16px; margin-top: 40px; }
  code { font-family: ui-monospace, monospace; background: var(--panel-2); padding: 1px 5px; border-radius: 4px; font-size: 12px; }
  @media (max-width: 720px) { .cards { grid-template-columns: 1fr; } .stats { grid-template-columns: repeat(2, 1fr); } }
</style>
</head>
<body>
<div class="wrap">
  <header class="top">
    <h1>Estadísticas de paquetes · VantaDB</h1>
    <p class="sub">Generado el ${now.toISOString().replace("T", " ").slice(0, 16)} UTC · ventana de serie ${DAYS} días · ${collected.length} paquetes en ${packages.length} manifiestos locales</p>
  </header>

  <section class="kpis">
    <div class="kpi"><span class="k">Descargas 7 días</span><span class="v">${nf(totals.npm.week + totals.pypi.week)}</span><span class="s">npm ${nf(totals.npm.week)} · PyPI ${nf(totals.pypi.week)}</span></div>
    <div class="kpi"><span class="k">Descargas 30 días</span><span class="v">${nf(totals.npm.month + totals.pypi.month)}</span><span class="s">npm ${nf(totals.npm.month)} · PyPI ${nf(totals.pypi.month)}</span></div>
    <div class="kpi"><span class="k">Descargas 365 días</span><span class="v">${nf(totals.npm.year + totals.pypi.year)}</span><span class="s">npm ${nf(totals.npm.year)} · PyPI ${nf(totals.pypi.year)} (histórico disponible)</span></div>
    <div class="kpi"><span class="k">Paquetes</span><span class="v">${collected.length}</span><span class="s">${totals.npm.packages} npm · ${totals.pypi.packages} PyPI</span></div>
    <div class="kpi"><span class="k">Drift de versión</span><span class="v">${drift.length}</span><span class="s">${drift.length ? drift.map((d) => esc(d.name)).join(", ") : "todo en línea"}</span></div>
    <div class="kpi"><span class="k">Fallos</span><span class="v">${errors.length}</span><span class="s">${errors.length ? "ver sección de errores" : "sin errores"}</span></div>
  </section>

  <section class="section">
    <h2>npm · descargas diarias</h2>
    ${ecosystemChart(npmPkgs, colorOf)}
  </section>

  <section class="section">
    <h2>PyPI · descargas diarias (sin mirrors)</h2>
    ${ecosystemChart(pypiPkgs, colorOf)}
  </section>
  <section class="section">
    <h2>Detalle por paquete</h2>
    <div class="table-wrap">
      <table>
        <thead><tr>
          <th>Paquete</th><th>Registro</th><th>Local</th><th>Publicado (v)</th><th>Estado</th>
          <th class="num">7d</th><th class="num">30d</th><th class="num">365d</th><th class="num">Tamaño</th>
          <th>Fecha pub.</th><th>30 días</th>
        </tr></thead>
        <tbody>${tableRows}</tbody>
      </table>
    </div>
  </section>

  <section class="section">
    <h2>Historial de versiones</h2>
    <div class="cards">
      ${collected.map((p) => packageCard(p, colorOf(p.name))).join("")}
    </div>
  </section>

  ${errorBlock}

  <footer>
    Fuentes: <code>registry.npmjs.org</code> + <code>api.npmjs.org</code> (descargas npm, todos los registries)
    · <code>pypi.org/pypi/&lt;pkg&gt;/json</code> (metadatos PyPI)
    · <code>pypistats.org/api/packages/&lt;pkg&gt;/overall</code> (descargas PyPI, <strong>sin mirrors</strong> — no es comparable 1:1 con npm).
    <br>Las ventanas PyPI (1d/7d/30d/365d) se calculan sobre la serie diaria de pypistats: su endpoint <code>recent</code> devuelve <code>last_month = 0</code> para varios paquetes pese a tener datos reales.
    <br>Versiones locales leídas de los manifiestos del repo (package.json / Cargo.toml / pyproject.toml); el versionado de los bindings maturin sigue al <code>Cargo.toml</code> del crate.
  </footer>
</div>
</body>
</html>
`;

// ── write ──────────────────────────────────────────────────────────────────
mkdirSync(OUT_DIR, { recursive: true });
const htmlPath = path.join(OUT_DIR, "package-stats.html");
const jsonPath = path.join(OUT_DIR, "package-stats.json");
writeFileSync(htmlPath, html, "utf8");
writeFileSync(
  jsonPath,
  JSON.stringify({ generatedAt: now.toISOString(), days: DAYS, totals, drift, errors, packages: collected }, null, 2),
  "utf8",
);

console.log(`\n  HTML  -> ${htmlPath}`);
console.log(`  JSON  -> ${jsonPath}`);
console.log(
  `  ${collected.length} paquetes · ${nf(totals.npm.week + totals.pypi.week)} descargas/7d · ${drift.length} drift · ${errors.length} fallos`,
);

if (OPEN) {
  const { spawn } = await import("node:child_process");
  const cmd = process.platform === "win32" ? "cmd" : "open";
  const args = process.platform === "win32" ? ["/c", "start", "", htmlPath] : [htmlPath];
  spawn(cmd, args, { detached: true, stdio: "ignore" }).unref();
}

process.exit(collected.length ? 0 : 1);
