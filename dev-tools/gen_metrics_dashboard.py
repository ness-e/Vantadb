#!/usr/bin/env python3
"""Generate metricas/dashboard.html — self-contained metrics panel.

Reads every JSON under metricas/ (all optional; missing files render as
"sin datos"), plus data/benchmark/manifest.json and
benchmarks/criterion_baseline.json. No CDN, no build step, no server:
open metricas/dashboard.html directly (file:// works — data is inlined).

Usage:
    python dev-tools/gen_metrics_dashboard.py [--out metricas/dashboard.html]
"""

import argparse
import html
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MET = ROOT / "metricas"
BM = MET / "benchmarks"


def load_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


def esc(v) -> str:
    return html.escape(str(v))


def card(title: str, value: str, sub: str = "") -> str:
    return (
        f'<div class="card"><div class="card-t">{esc(title)}</div>'
        f'<div class="card-v">{esc(value)}</div>'
        + (f'<div class="card-s">{esc(sub)}</div>' if sub else '')
        + "</div>"
    )


def missing_card(title: str) -> str:
    return (
        f'<div class="card missing"><div class="card-t">{esc(title)}</div>'
        '<div class="card-v">—</div><div class="card-s">sin datos</div></div>'
    )


def svg_line(points: list, w: int = 560, h: int = 220, xlab: str = "", ylab: str = "") -> str:
    """Minimalist SVG line chart. points = [(x: float, y: float, label: str)]."""
    if not points:
        return '<p class="muted">sin datos</p>'
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    xmin, xmax = min(xs), max(xs)
    ymin, ymax = 0.0, max(ys) if max(ys) > 0 else 1.0
    if xmax == xmin:
        xmax = xmin + 1.0
    pad_l, pad_b, pad_t, pad_r = 52, 30, 12, 12

    def X(x):
        return pad_l + (x - xmin) / (xmax - xmin) * (w - pad_l - pad_r)

    def Y(y):
        return h - pad_b - (y - ymin) / (ymax - ymin) * (h - pad_b - pad_t)

    pts = " ".join(f"{X(x):.1f},{Y(y):.1f}" for x, y, _ in points)
    dots = "".join(
        f'<circle cx="{X(x):.1f}" cy="{Y(y):.1f}" r="3.5" class="dot">'
        f"<title>{esc(lbl)}: ({x:g}, {y:g})</title></circle>"
        for x, y, lbl in points
    )
    return (
        f'<svg viewBox="0 0 {w} {h}" class="chart" role="img">'
        f'<line x1="{pad_l}" y1="{h-pad_b}" x2="{w-pad_r}" y2="{h-pad_b}" class="axis"/>'
        f'<line x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{h-pad_b}" class="axis"/>'
        f'<text x="{pad_l}" y="{h-8}" class="alab">{esc(xlab)}</text>'
        f'<text x="8" y="{pad_t+8}" class="alab">{esc(ylab)}</text>'
        f'<text x="{w-pad_r}" y="{h-8}" class="alab" text-anchor="end">{xmax:g}</text>'
        f'<text x="{pad_l+2}" y="{pad_t+12}" class="alab">{ymax:g}</text>'
        f'<polyline points="{pts}" class="line"/>'
        f"{dots}</svg>"
    )


def svg_bars(rows: list, w: int = 560) -> str:
    """rows = [(label, value_ms)]. Horizontal log-friendly bars."""
    if not rows:
        return '<p class="muted">sin datos</p>'
    vmax = max(v for _, v in rows) or 1.0
    out = ['<div class="bars">']
    for label, v in rows:
        pct = max(2.0, v / vmax * 100.0)
        out.append(
            f'<div class="brow"><span class="blab">{esc(label)}</span>'
            f'<span class="btrack"><span class="bfill" style="width:{pct:.1f}%"></span></span>'
            f'<span class="bval">{v:g} ms</span></div>'
        )
    out.append("</div>")
    return "".join(out)


def table(headers: list, rows: list) -> str:
    if not rows:
        return '<p class="muted">sin datos</p>'
    th = "".join(f"<th>{esc(h)}</th>" for h in headers)
    tr = "".join(
        "<tr>" + "".join(f"<td>{esc(c)}</td>" for c in r) + "</tr>" for r in rows
    )
    return f'<table><thead><tr>{th}</tr></thead><tbody>{tr}</tbody></table>'


def section(title: str, body: str) -> str:
    return f'<section><h2>{esc(title)}</h2>{body}</section>'


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(MET / "dashboard.html"))
    args = ap.parse_args()

    canon = load_json(BM / "canonical_summary.json")
    reef = load_json(BM / "recall_ef_curve.json")
    rps = load_json(BM / "recall_param_sweep.json")
    rac = load_json(BM / "recall_acorn.json")
    hyb = load_json(BM / "hybrid_recall.json")
    enc = load_json(BM / "encrypted_summary.json")
    bench_rep = load_json(BM / "vanta_benchmark_report.json")
    mem = load_json(MET / "memoria" / "memory_bench_report.json")
    memh = load_json(MET / "memoria" / "memory_harness_report.json")
    emb = load_json(MET / "embeddings" / "embed_bench_report.json")
    cert = load_json(MET / "certificacion" / "vanta_certification.json")
    manifest = load_json(ROOT / "data" / "benchmark" / "manifest.json")
    baseline = load_json(ROOT / "benchmarks" / "criterion_baseline.json")

    try:
        commit = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, cwd=ROOT,
        ).stdout.strip() or "?"
    except Exception:
        commit = "?"
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    # ---- stat cards ----
    cards = []
    if canon:
        cards.append(card("Canonical p50", f"{canon.get('p50_ms', '?')} ms", "search GloVe-100"))
        cards.append(card("Canonical p99", f"{canon.get('p99_ms', '?')} ms", "search GloVe-100"))
        cards.append(card("Canonical recall@10", str(canon.get("recall", "?")), f"n={canon.get('n', '?')}"))
    else:
        cards += [missing_card("Canonical p50"), missing_card("Canonical p99"),
                  missing_card("Canonical recall@10")]
    if hyb:
        cards.append(card("Hybrid recall (BEIR)", str(hyb.get("recall", "?")),
                           f"{hyb.get('hits', '?')}/{hyb.get('total', '?')}"))
    else:
        cards.append(missing_card("Hybrid recall (BEIR)"))
    if mem:
        cards.append(card("Memory recall@k", str(mem.get("recall_at_k", "?")), "memory_bench"))
    else:
        cards.append(missing_card("Memory recall@k"))
    if enc:
        cards.append(card("Encrypted p50", f"{enc.get('search_p50_ms', '?')} ms",
                           f"mode={enc.get('mode', '?')} ram={enc.get('ram_bound', '?')}"))
    else:
        cards.append(missing_card("Encrypted p50"))
    if emb and emb.get("results"):
        r0 = emb["results"][0]
        reals = [r for r in emb["results"] if r.get("embed_real")]
        tag = "real ONNX" if reals else "dummy"
        cards.append(card("Embed p50", f"{r0.get('embed_p50_ms', '?')} ms",
                           f"{r0.get('id', '?')} · {tag}"))
    else:
        cards.append(missing_card("Embed p50"))
    if isinstance(cert, list):
        cards.append(card("Cert runs", str(len(cert)), "historial"))
    else:
        cards.append(missing_card("Cert runs"))

    # ---- recall-QPS curve ----
    if reef and reef.get("cells"):
        pts = [(c["qps"], c["recall"], f"ef={c['ef']}") for c in reef["cells"]]
        curve = svg_line(pts, xlab="QPS →", ylab="recall@10")
        curve_sub = f"GloVe {reef.get('n', '?')} vectores · ef {[c['ef'] for c in reef['cells']]}"
    else:
        curve, curve_sub = '<p class="muted">sin datos (correr hnsw_recall_ef)</p>', ""
    recall_sec = section("Recall vs QPS (GloVe)", curve + (f'<p class="muted">{esc(curve_sub)}</p>' if curve_sub else ""))

    # ---- latency bars ----
    bars = []
    if canon:
        bars += [(f"canonical p50", canon["p50_ms"]), ("canonical p99", canon["p99_ms"])]
    if mem:
        bars += [("memory q-p50", mem.get("q_p50_ms", 0) or 0),
                 ("memory q-p99", mem.get("q_p99_ms", 0) or 0)]
    if bench_rep and isinstance(bench_rep.get("query_vector"), dict):
        qv = bench_rep["query_vector"]
        bars += [("sdk vector p50", qv.get("p50_ms", 0) or 0),
                 ("sdk vector p99", qv.get("p99_ms", 0) or 0)]
    latency_sec = section("Latencias p50/p99 (ms)", svg_bars(
        [(l, float(v)) for l, v in bars if isinstance(v, (int, float))]))

    # ---- param sweep + acorn tables ----
    if rps and rps.get("cells"):
        sweep_rows = [[c.get("m", ""), c.get("ef_c", ""), c.get("ef_s", ""),
                       c.get("recall", ""), c.get("build_s", "")] for c in rps["cells"]]
        sweep = table(["M", "efC", "efS", "recall", "build_s"], sweep_rows)
    else:
        sweep = '<p class="muted">sin datos (correr param_sweep)</p>'
    if rac and rac.get("cells"):
        acorn_rows = [[c.get("selectivity", ""), c.get("recall", "")] for c in rac["cells"]]
        acorn = table(["selectividad", "recall"], acorn_rows)
    else:
        acorn = '<p class="muted">sin datos (correr acorn_filtered_search)</p>'
    tables_sec = section("Sweep paramétrico (sift-128)", sweep) + section(
        "ACORN por selectividad", acorn)

    # ---- history (certificacion runs) ----
    if isinstance(cert, list) and cert:
        hist = []
        for run in cert:
            ts = str(run.get("timestamp", "?"))[:16].replace("T", " ")
            mets = run.get("metrics", {}) or {}
            total = sum(float(v.get("duration_secs", 0) or 0)
                        for v in mets.values() if isinstance(v, dict))
            hist.append([ts, str(len(mets)), f"{total:.1f}s"])
        hist_sec = section("Historial certificación (%d corridas)" % len(cert),
                           table(["corrida", "bloques", "duración total"], hist))
    else:
        hist_sec = section("Historial certificación", '<p class="muted">sin datos</p>')

    # ---- datasets ----
    if manifest and manifest.get("datasets"):
        ds_rows = [[n, d.get("metric", ""),
                    ", ".join(sorted(d.get("files", {}).keys()))]
                   for n, d in manifest["datasets"].items()]
        ds_sec = section("Datasets (data/benchmark)", table(
            ["dataset", "métrica", "archivos"], ds_rows))
    else:
        ds_sec = section("Datasets (data/benchmark)", '<p class="muted">sin manifiesto (correr download_ground_truth.py)</p>')

    # ---- baseline + gaps ----
    base_n = len((baseline or {}).get("benchmarks", {})) if baseline else 0
    expected = ["canonical_summary.json", "recall_ef_curve.json",
                "recall_param_sweep.json", "recall_acorn.json",
                "hybrid_recall.json", "encrypted_summary.json"]
    gaps = [f for f in expected if not (BM / f).exists()]
    gaps_html = ("<ul>" + "".join(f"<li><code>{esc(g)}</code> — generar corriendo su bench</li>"
                                  for g in gaps) + "</ul>") if gaps else "<p>Sin gaps: todos los sinks presentes.</p>"
    meta_sec = section(
        "Baseline y gaps",
        f"<p>Labels en <code>benchmarks/criterion_baseline.json</code>: <strong>{base_n}</strong> "
        "(los nuevos entran vía <code>analyze</code> en nightly).</p>" + gaps_html)

    body = ("".join([
        '<div class="cards">', "".join(cards), "</div>",
        recall_sec, latency_sec, tables_sec, hist_sec, ds_sec, meta_sec,
    ]))

    html_doc = f"""<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>VantaDB — Panel de métricas</title>
<style>
:root {{ color-scheme: light; --ink: #1a1a1a; --mut: #6b7280; --line: #e5e7eb;
  --bg: #fafafa; --card: #fff; --acc: #1d4ed8; --warn: #b45309; }}
* {{ box-sizing: border-box; }}
body {{ font-family: ui-sans-serif, system-ui, "Segoe UI", sans-serif; color: var(--ink);
  background: var(--bg); margin: 0; padding: 24px; max-width: 1024px; }}
header {{ display: flex; justify-content: space-between; align-items: baseline;
  border-bottom: 2px solid var(--ink); padding-bottom: 8px; margin-bottom: 20px; }}
h1 {{ font-size: 22px; margin: 0; }} h2 {{ font-size: 16px; margin: 26px 0 10px; }}
.meta {{ color: var(--mut); font-size: 12px; }}
.cards {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 10px; }}
.card {{ background: var(--card); border: 1px solid var(--line); border-radius: 8px; padding: 12px; }}
.card-t {{ font-size: 11px; color: var(--mut); text-transform: uppercase; letter-spacing: .04em; }}
.card-v {{ font-size: 22px; font-weight: 650; margin: 4px 0; }}
.card-s {{ font-size: 11px; color: var(--mut); }}
.card.missing {{ opacity: .55; }}
table {{ border-collapse: collapse; width: 100%; font-size: 13px; background: var(--card); }}
th, td {{ border: 1px solid var(--line); padding: 6px 9px; text-align: left; }}
th {{ background: #f3f4f6; font-weight: 600; }}
.muted {{ color: var(--mut); font-size: 13px; }}
.chart {{ width: 100%; max-width: 620px; background: var(--card);
  border: 1px solid var(--line); border-radius: 8px; }}
.chart .axis {{ stroke: #9ca3af; stroke-width: 1; }}
.chart .line {{ fill: none; stroke: var(--acc); stroke-width: 2; }}
.chart .dot {{ fill: var(--acc); }}
.chart .alab {{ font-size: 10px; fill: var(--mut); }}
.bars {{ display: flex; flex-direction: column; gap: 6px; max-width: 620px; }}
.brow {{ display: grid; grid-template-columns: 150px 1fr 90px; gap: 8px; align-items: center; font-size: 12px; }}
.blab {{ color: var(--mut); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
.btrack {{ background: #eef0f3; border-radius: 4px; height: 12px; }}
.bfill {{ display: block; background: var(--acc); border-radius: 4px; height: 12px; }}
.bval {{ text-align: right; font-variant-numeric: tabular-nums; }}
code {{ background: #eef0f3; padding: 1px 5px; border-radius: 4px; font-size: 12px; }}
ul {{ font-size: 13px; }}
</style>
</head>
<body>
<header><h1>VantaDB — Panel de métricas</h1>
<span class="meta">generado {esc(now)} · commit {esc(commit)}</span></header>
{body}
</body>
</html>"""

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html_doc, encoding="utf-8")
    print(f"Dashboard -> {out} ({len(html_doc)//1024} KiB)")


if __name__ == "__main__":
    main()
