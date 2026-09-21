# VantaDB README GIFs — Design Contract

Fuente de verdad para las composiciones Remotion del README. Derivado de
`reference-design-contract` + `industrial-brutalist-ui` (UN arquetipo por
canvas — nunca mezclar) + reglas de `remotion-best-practices`.

## 1. Contexto del problema

- `assets/banner.gif` (829 KB) = Banner V1 transparente con texto blanco.
  **Pesaba demasiado** y el wordmark era simple.
- `assets/demo.gif` **NO EXISTÍA** → link roto en README.md:91 / README_ES.md:88.
- Bug de pipeline: `--pixel-format=yuva444p10le` (10-bit) en GIF de 8-bit
  indexado → infla el archivo. La vida útil de un GIF de README está en
  colores sólidos y toggles, no en alpha parcial.

## 2. Decisión de arquetipos

| Canvas | Arquetipo | Substrate | Uso |
|---|---|---|---|
| `BannerV3` | **Swiss Industrial Print — LIGHT, transparente** | transparente (lee sobre dark) | README header |
| `DemoTerminal` | **Tactical Telemetry — DARK** | near-black `#0A0A0A` sólido | README demo (sección "Quickstart") |

Banner transparente + texto blanco: se lee sobre el fondo dark de GitHub.
El demo usa fondo sólido porque es un terminal con su propio marco.

## 3. Tokens (consistencia con `web/src/app/globals.css`)

| Token | Valor | Uso |
|---|---|---|
| white | `#FFFFFF` | wordmark VANTA, bordes, registration marks, líneas grid |
| neon | `#FF5500` | ÚNICO accent (DB, stickers, corner) |
| smoke | `#1A1A1A` | fondo oscuro de sticker PERSISTENT |
| dim | `rgba(255,255,255,0.6)` | texto secundario |
| hair | `rgba(255,255,255,0.28)` | líneas grid / hairline |
| near-black | `#0A0A0A` | substrate terminal demo |
| phosphor | `#EAEAEA` | texto terminal demo |

Tipografía: **Anton** (display, wordmark) + **Space Mono** (metadata, terminal,
métricas con `fontVariantNumeric: "tabular-nums"`).

## 4. Reglas de movimiento (remotion-best-practices)

1. **PROHIBIDO CSS transitions** — todo con `interpolate` / `Easing` / `spring`.
2. Curva maestra: `Easing.bezier(0.16, 1, 0.3, 1)` (ease-out expo-ish).
3. Stagger 30–80 ms entre elementos; NUNCA entrada simultánea de todo.
4. Micro-interacciones (emil-design-eng): solo al aterrizar/estado; nunca
   distraer del contenido.
5. GIF-safe: colores sólidos, toggles on/off, sombras duras (NO blur que
   dependa de paleta), opacidad solo binaria.
6. Typewriter por **string slicing**, no por opacidad de caracteres.

## 5. BannerV3 — guion (120 frames @ 30fps = 4s loop, render cada 3er frame)

Layout: wordmark izquierda, mark derecha, eslogan + métricas bajo wordmark.

**Eslogan oficial** (docs/web/standards/product-positioning.md:162):
"Embedded Rust engine for durable local memory and hybrid vector retrieval."

**Métricas comerciales** (NO amarradas a stats del proyecto — no requieren
re-generar cuando cambien benchmarks): LOCAL-FIRST · PERSISTENT · HYBRID SEARCH.

**Patrón fondo** (identidad "vector database"): dot grid blanco escalonado
(radio por columna, estilo ONS) + subtrama naranja (columnas pares) + puntos
especiales que se encienden en ciclo (toggle GIF-safe) + 8 diagonales finas
que se desplazan en loop. La animación vive en diagonales y toggles — el
respiro de TODOS los puntos explota el peso GIF (cada frame difiere en todo
el canvas). SIN marco exterior grueso — solo hairline interior (1px `hair` a
8px) que conecta los registration marks de las esquinas. Mark en modo
`solid`: esfera naranja + ojos desnudos, **sin anillo envolvente** (aro y
glow pulsante quitados en V3).

| Frames | Evento |
|---|---|
| 0–24 | Patrón fondo + registration marks aparecen (scale from 0.98 + fade) |
| 4–44 | Wordmark `VANTADB` cae letra a letra (stagger 3f, overshoot) |
| 10–40 | Mark pop-in con spring (Easing.back) |
| 34–58 | Eslogan + métricas comerciales |
| 45+ | Idle: `LOCAL-FIRST ✓` tick cada 30f; shine sweep 2 pasadas; diagonales desplazándose + toggles |
| 90–120 | Nada nuevo — loop cierra con wordmark breath sutil |

## 6. DemoTerminal — guion (120 frames @ 30fps = 4s loop)

Terminal ASCII 960×320, marco suizo, scanlines fijas.

| Frames | Evento |
|---|---|
| 0–6 | Marco + header aparecen (fade rápido) |
| 6–30 | `$ pip install vantadb` typewriter (2 chars/frame) → `✓ installed in 1.2ms` |
| 34–62 | `$ vanta.put("agents/alice", "loves vector search")` → `✓ stored · key=agents/alice` |
| 66–96 | `$ vanta.search("vector memory")` → `3 hits · BM25+HNSW · 0.9ms` |
| 96–120 | Idle: cursor blink; footer `LOCAL-FIRST · PERSISTENT` |

Cursor: bloque `▌` toggle cada 15f SOLO en la línea activa.

## 7. Pipeline (ffmpeg two-pass, GIF-safe, chroma-key)

El GIF indexado solo tiene transparencia binaria — los píxeles
semi-transparentes del antialiasing del texto (alpha 30-70%) se ven
serruchados. Por eso la composición renderiza sobre near-black `#0A0A0A`
(chroma-key) y ffmpeg lo convierte a transparente: el antialias del texto
blanco genera grises reales (blanco+negro) que el GIF representa bien.

```sh
# 1. frames PNG @2x, cada 3er frame (loop 4s @ 10fps — equilibrio peso/suavidad)
remotion render <Comp> out/<name>-frames --sequence --scale=2 --image-format=png --every-nth-frame=3
# 2. colorkey del chroma + paleta 16 colores + dither none (arte plano)
ffmpeg -framerate 10 -i out/<name>-frames/element-%02d.png \
  -filter_complex "[0:v]scale=960:320:flags=lanczos,colorkey=0x0A0A0A:0.02:0,split[a][b];\
  [a]palettegen=max_colors=16:stats_mode=diff[p];\
  [b][p]paletteuse=dither=none" -loop 0 out/<name>.gif
```

Banner: 16 colores + chroma-key → **~700 KB con bordes suaves**. Demo (fondo
sólido, sin colorkey): 64 colores → **~50 KB**.

## 8. Aceptación

- [ ] banner-v3.gif lee bien sobre fondo dark (transparente vía chroma-key)
- [ ] Bordes del texto suaves (sin serrucho) — antialias conservado en grises
- [ ] demo.gif muestra install + put + search con métricas
- [ ] Peso: banner < 750 KB, demo < 350 KB
- [ ] Loop sin saltos (última frame == primera en reposo)
- [ ] `README.md` y `README_ES.md` apuntan a los GIFs existentes