# VantaDB Hero Demo — reproduction

A 60-second, five-frame presentation built from **one real local run**
frozen into `hero-run.json` + `hero-evidence.json`. The playback is
edited pacing — not a live terminal — and the player says so.

Pipeline: `examples/hero_demo.py` freezes five real stages →
`hero-player.html` renders the frozen frames (5 × 12 s) →
`record-hero.mjs` records the page with **Playwright `recordVideo`**
→ bundled FFmpeg exports `assets/demo.mp4` / `demo.gif` / `demo-poster.png`.

## Why page video instead of screen capture

The Recordly `wgc-capture` helper was rejected after reading its
published v1.4.0 C++ source: `initialize(HWND)` resolves the target
window via `MonitorFromWindow` and creates the capture item **for the
monitor**, cropping each delivered monitor frame to the window rect —
so a window-targeted call still captures the whole monitor inside the
frame pool. That is a privacy risk for anything else on screen, so no
native screen/window capture is executed. Only Recordly's bundled
FFmpeg 6.1.1 is reused as the encoder. Consult the upstream project for
licenses: https://github.com/webadderallorg/Recordly

## Prerequisites (once)

- `target/release/vanta-cli` + `vantadb-server` built from this checkout
  (`cargo build --release -j2 --bin vanta-cli --bin vantadb-server`).
- Python SDK 0.9 built in its active venv
  (`maturin develop --manifest-path vantadb-python/Cargo.toml --release`).
  Provision the local E5 ONNX encoder assets separately; `hero_demo.py`
  downloads nothing and a fresh clone does not include the model files.
- Node + an isolated playwright-core install (pin, outside the repo):

```powershell
npm --prefix C:\path\to\tools\pw install playwright-core@1.63.0
$env:PLAYWRIGHT_BROWSERS_PATH = "C:\path\to\tools\browsers"
node "C:\path\to\tools\pw\node_modules\playwright-core\cli.js" install chromium
```

- `PLAYWRIGHT_MODULE` must be the **absolute** path to
  `...\playwright-core\index.mjs`; `CHROME_BIN` is optional (otherwise
  the bundled Chromium installed by the CLI above is used).

## Freeze (once per data change)

```powershell
python examples\hero_demo.py --output vhs\hero-run.json --evidence-dir C:\path\to\evidence\hero-run
```

Writes `vhs/hero-run.json` (5 frames) + `vhs/hero-evidence.json`
(full stats + L2 scene block). Renderers read these frozen files; they
never re-run the backend.

## Capture + export

```powershell
$env:PLAYWRIGHT_MODULE = "C:\path\to\tools\pw\node_modules\playwright-core\index.mjs"
node vhs\record-hero.mjs C:\path\to\evidence\hero-browser-raw.webm

$env:FFMPEG  = "C:\path\to\tools\ffmpeg.exe"
$env:FFPROBE = "C:\path\to\tools\ffprobe.exe"
& $env:FFPROBE -v error -show_entries format=duration -of json C:\path\to\evidence\hero-browser-raw.webm
& $env:FFMPEG -n -sseof -60 -i C:\path\to\evidence\hero-browser-raw.webm -t 60 -an `
  -vf "setpts=PTS-STARTPTS,scale=1280:-2:flags=lanczos" -c:v libx264 `
  -preset medium -crf 22 -pix_fmt yuv420p -movflags +faststart C:\path\to\out\demo.mp4
& $env:FFMPEG -n -i C:\path\to\out\demo.mp4 `
  -vf "fps=10,scale=960:-2:flags=lanczos,split[a][b];[a]palettegen=max_colors=96:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5" `
  -loop 0 C:\path\to\out\demo.gif
& $env:FFMPEG -n -ss 6 -i C:\path\to\out\demo.mp4 -frames:v 1 C:\path\to\out\demo-poster.png
```

Create the evidence/output directories before recording. Guard the raw
duration (`>= 60 s`) before trimming; use a fresh output folder you own —
do not write over tracked `assets/` blindly. The encoder commands refuse
to overwrite existing output files.

## Measured facts (this production run)

playwright-core **1.63.0** · Chromium **153.0.8010.12** · raw capture
**62.2 s** WebM → final **60.0 s**, 1280×800 h264, **943 KB** mp4 · GIF
**3.84 MB** (<8 MB) · silent (no audio). Transcript (verbatim frames +
frozen stats/L2): [demo-transcript.txt](demo-transcript.txt).

- `vhs/demo.tape` documents the VHS reproduction spec; VHS was not
  rendered on this Windows host because `ttyd` was unavailable.
- No automatic L2 consolidation, LLM calls, or performance figures are
  claimed. Engine counters shown are CLI stats, not process RSS.
