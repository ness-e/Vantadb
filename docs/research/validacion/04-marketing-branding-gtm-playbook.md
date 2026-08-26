# VantaDB — Playbook de Marketing, Branding y Go-To-Market (primeros 1.000 usuarios)

**Fecha:** 2026-08-25 · **Autor:** Agente D (research GTM) · **Presupuesto:** ~cero (founder-led)

---

## Resumen ejecutivo

Los líderes de memoria para agentes (Mem0, Zep, Letta) son todos **cloud/API-first y enterprise**. Ninguno puede decir "tus datos nunca salen de tu máquina" ni "corre en el navegador". VantaDB tiene tres diferenciadores que nadie más combina: **local-first/privacidad, WASM-en-browser, un binario embebido con búsqueda híbrida (BM25+vector+RRF)**. El plan de 90 días concentra el presupuesto-cero en activos que se distribuyen solos: el MCP server (ya existe como crate en el repo), un playground WASM embebible, llms.txt + docs en markdown, y benchmarks honestos reproducibles. Lanzamiento central: Show HN en semana 8, cuando el playground y los benchmarks estén vivos. Conversión: núcleo OSS gratis para siempre + pago por sync E2E/equipo/gestionado.

---

## 1. Positioning y taglines propuestos

### Cómo se posicionan los líderes (verificado 2026-08-25)

| Marca | Posicionamiento en su home | Ángulo |
|---|---|---|
| **Mem0** | "AI memory that persists across sessions and agents" / "Drop-in memory infrastructure" | Infraestructura cloud, escala (150.000+ devs), YC |
| **Zep** | "Agent memory, at enterprise scale" | Grafos temporales (Graphiti), empresa, gobernanza |
| **Letta** | "Agents that remember everything, learn continuously" (herederos de MemGPT) | Laboratorio de investigación, agentes automejorantes |

**Hueco detectado:** los tres requieren servidores/API cloud. El desarrollador que construye agentes con datos sensibles (salud, legal, on-prem) o que quiere memoria en edge/navegador no tiene opción primera-parte. ⚠️ Turso hoy dice "the lightweight database that scales to millions of agents… deploy… in browsers… built for the agentic future": el espacio narrativo "agentic + embedded" se está cerrando. Moverse rápido.

### Alternativas propuestas

**Opción A — Privacidad como lanza (emocional):**
> **"Local-first memory for AI agents."**
> Sub-línea: *Your agents' memory never leaves your machine.*

Público: builders de agentes con datos sensibles; comunidad r/LocalLLaMA. Es el claim que Mem0/Zep/Letta **no pueden copiar** sin contradecir su modelo de negocio.

**Opción B — Analogía SQLite (técnica, recomendada como soporte primario):**
> **"The embedded memory engine for AI agents."**
> Sub-línea: *Hybrid search (BM25 + vectors + RRF). One binary. No server. Like SQLite — for what your agents remember.*

La analogía SQLite comunica instantáneamente: embebido, cero-config, confiable, tuyo. Funciona en HN, Reddit y SEO ("sqlite for ai memory").

**Opción C — Frontera WASM (demostrativa):**
> **"Memory that runs wherever your agent runs — server, laptop, or browser tab."**

Único claim verdadero-e-imposible para la competencia. No como tagline principal sino como **prueba viviente**: el playground en el navegador ES el posicionamiento hecho clic.

**Recomendación:** H1 = Opción A ("Local-first memory for AI agents"), soporte técnico = Opción B, demostración = Opción C. Consistencia en README, sitio, X bio, lanzamientos.

---

## 2. Branding mínimo viable

### Naming / coherencia de ecosistema

- Mantener **un solo nombre en todas las superficies** (descubribilidad > creatividad): PyPI `vantadb` · npm `@vantadb/core` (TS) · `@vantadb/wasm` (browser) · crates.io `vantadb` · `vantadb-mcp` (MCP server). No renombrar el SDK Python distinto: fragmenta búsquedas y word-of-mouth.
- Convención de subproyectos estilo Astral (`ruff`, `uv`) o Turso (`libsql`, `turso`): nombres cortos bajo una casa.

### Identidad visual mínima (semana 1–2)

| Elemento | Decisión |
|---|---|
| Concepto | "Vanta" evoca vantablack → negro casi-total como lienzo; la memoria es luz dentro de la oscuridad |
| Paleta | 2 colores: near-black `#0A0A0C` + un acento eléctrico (cyan `#22D3EE` o lima `#A3E635`). Nada intermedio |
| Tipografía | Inter o Geist Sans (UI) + JetBrains Mono (código/snippets — el código ES el marketing) |
| Logo/favicon | Marca geométrica simple (cuadrado negro con chispa de acento o "V"), legible a 16px |
| og:image / social cards | Plantilla 1200×630 generada desde un frame del **logo-reveal.gif existente** + tagline. Un template, reutilizable por post |
| Tono de voz | Dev-to-dev: código primero, cero hype, números verificables, límites admitidos abiertamente |

### Qué copiar de referencias exitosas

- **Supabase:** framing "open source alternative to X" + docs-first + changelog vivo como canal.
- **Turso:** narrativa agentic/edge + playground interactivo en el sitio.
- **Bun:** benchmarks como identidad de marca (el número ES el marketing).
- **Tauri:** filosofía pública escrita + comunidad Discord cuidada desde v1.

---

## 3. Playbook 90 días (priorizado esfuerzo/impacto)

Semana 1 = inicio inmediato. Esfuerzo: B=bajo, M=medio, A=alto.

| Acción | Esfuerzo | Impacto esperado | Semana |
|---|---|---|---|
| Quickstart <60 s en README + docs (copy-paste verificado en CI) | M | Muy alto — todo lo demás depende de esto | S1 |
| **Publicar VantaDB MCP Server** al MCP Registry oficial + directorios + snippet config Claude Desktop en README | M | Muy alto — distribución agent-native; el crate ya existe en el repo | S1–S2 |
| `llms.txt` + versiones `.md` de docs + JSON-LD (SEO para buscadores IA) | B | Medio-alto — Lighthouse ya audita llms.txt; OpenAI/Anthropic/Gemini publican el suyo | S2 |
| Roadmap público (GitHub Projects) + CHANGELOG vivo por release | B | Medio-alto — confianza founder-led | S2 |
| Benchmarks públicos honestos vs sqlite-vec/Chroma/LanceDB, reproducibles (BENCHMARKS.md estilo uv) | M | Alto — ya existen FND-13 e INV-007 como base | S3–S4 |
| PRs a awesome-lists (awesome-ai-agents, awesome-local-first, awesome-rust, directorios MCP) | B | Medio | S3 |
| Build-in-public en X: thread semanal + GIFs de demo | B (continuo) | Medio — compounding | S1–S12 |
| Post técnico en r/LocalLLaMA (valor, no venta) + r/rust + r/LLMDevs | B | Alto — audiencia exacta local-first | S4–S6 |
| **Playground WASM embebible** en el sitio (memoria corriendo EN tu browser, sin backend) | M | Muy alto — el momento compartible; nadie más lo tiene | S2–S4 |
| Discord propio (#soporte, #show-your-agents) | B | Medio | S5 |
| Integraciones visibles: LangChain, LlamaIndex, CrewAI + página por framework | M-A | Alto — cada integración es un canal de búsqueda | S6–S10 |
| Video demo corto (<3 min) mostrando el demo en-browser | M | Medio | S7 |
| **Show HN**: mar–mié 8–10 am ET, título "Show HN: VantaDB – Local-first memory for AI agents (runs in your browser)", founder presente 24 h, demo sin signup | M | Muy alto | S8 |
| Product Hunt (martes siguiente al HN, mismo activo) | M | Medio | S9 |

Reglas de Show HN (guía oficial, verificada): debe poder **probarse sin registros**, el autor debe estar presente para responder, no pedir upvotes a amigos, proyecto no-trivial. VantaDB cumple las cuatro si el playground está vivo.

---

## 4. Casos de éxito recientes y patrones replicables

| Caso | Qué hizo en su launch (fuentes verificadas) | Patrón replicable con $0 |
|---|---|---|
| **Bun** (5-jul-2022) | Beta pública anunciada por el founder; "incredibly fast all-in-one JavaScript runtime"; explotó en Show HN | Un solo binario + un número de velocidad brutal + fundador respondiendo en el thread |
| **Tauri** (19-jun-2022) | v1.0 tras 9 meses de betas; hilo top en HN; narrativa contra Electron (tamaño, seguridad, privacidad); encuestas de comunidad activas | Enemigo claro (aquí: "memory en cloud que exporta tus datos") + filosofía escrita + Discord |
| **uv / Astral** (15-feb-2024) | "Drop-in replacement for pip" (adopción sin fricción), BENCHMARKS.md público y reproducible (8–115x), stewardship de Rye, roadmap ambicioso declarado | Drop-in API familiar + benchmarks reproducibles en repo + heredar buena voluntad del ecosistema |
| **Excalidraw** (ene-2020, Show HN) | Producto inherentemente compartible → cada usuario distribuía | El equivalente VantaDB: **playground WASM embebible** — quien lo prueba lo puede incrustar/mostrar |
| **Turso** (continuo hasta 2026) | Pivotó narrativa a "database for the age of AI… scales to millions of agents… browsers" | La narrativa agentic convierte; moverse antes de que el hueco local-first se cierre |

Patrón transversal: **número honesto + reemplazo inmediato de algo conocido + fundador visible + artefacto que se comparte solo.**

---

## 5. Conversión prueba → pago

Qué genera confianza (y por tanto pago) en developer tools:

1. **Quickstart <60 s sin email** (además es requisito de Show HN). Medir time-to-first-query y optimizarlo como métrica de producto.
2. **Snippets copy-paste que funcionan**, testeados en CI contra la doc publicada (docs-as-code).
3. **Respuesta a issues <24 h** — en founder-led, la velocidad del founder ES la marca.
4. **Changelog vivo + roadmap público** — demuestra momentum sin gastar.
5. **Honestidad de benchmarks** — incluir casos donde pierde; en 2026 la audiencia HN/Reddit castiga el cherry-picking y premia la honestidad.
6. **Licencia clara:** qué es OSS para siempre y qué es pago, dicho en una línea arriba del fold.

Modelo de monetización coherente con local-first (los líderes cobran por cloud; VantaDB no puede — cobrar por valor alrededor del núcleo):

- **Gratis para siempre:** motor embebido, SDKs, WASM, MCP server.
- **Pago:** sync multi-dispositivo cifrado E2E, backups gestionados, features de equipo (compartir memorias con permisos), soporte prioritario/licencia comercial para equipos. Espejo del modelo Supabase/Turso: núcleo libre, conveniencia de pago.

---

## Fuentes (verificadas 2026-08-25)

1. mem0.ai — posicionamiento "AI memory that persists across sessions and agents", 150k devs. https://mem0.ai
2. letta.com — "remember everything, learn continuously", creadores de MemGPT. https://letta.com
3. zep.ai — "Agent memory, at enterprise scale", Graphiti. https://zep.ai
4. astral.sh/blog/uv — launch de uv (15-feb-2024): drop-in pip, BENCHMARKS.md, principios de producto. https://astral.sh/blog/uv
5. news.ycombinator.com/showhn.html — guía oficial Show HN (sin signup, autor presente, no pedir upvotes). https://news.ycombinator.com/showhn.html
6. Lanzamiento Bun 5-jul-2022: https://news.ycombinator.com/item?id=32245095 · https://www.devclass.com/development/2022/07/06/zig-based-bun-appears-in-beta-an-incredibly-fast-all-in-one-javascript-runtime/1628571
7. Tauri 1.0 (19-jun-2022): https://v1.tauri.app/blog/2022/06/19/tauri-1-0/ · hilo HN: https://news.ycombinator.com/item?id=31764015 · https://v2.tauri.app/blog/tauri-community-growth-and-feedback/
8. turso.tech — "scales to millions of agents… in browsers… agentic future". https://turso.tech · https://docs.turso.tech/libsql
9. llmstxt.org — spec v2 (act. ago-2026): miles de sitios, Mintlify auto-genera, Lighthouse audita, OpenAI/Anthropic/Gemini publican. https://llmstxt.org/
10. github.com/modelcontextprotocol/servers — registro oficial MCP (registry.modelcontextprotocol.io), server de referencia "Memory", SDKs en 10 lenguajes. https://github.com/modelcontextprotocol/servers

Interno: `vantadb-mcp/` (crate con tests, existe en este repo) · `docs/research/FND-13-benchmarks-honestos.md` · `docs/research/INV-007-competitive-benchmark-lancedb-chroma.md` · `logo-reveal.gif` (activo de marca existente).

## Lagunas

- Métricas duras de los primeros 90 días de Bun/Tauri (stars/descargas exactas): patrones cualitativos consistentes entre ≥2 fuentes, sin números.
- Datos de conversión prueba→pago específicos de devtools OSS: anecdóticos (modelos Supabase/Turso), no benchmark público.
- Estado de publicación actual de `vantadb-mcp` en el MCP Registry: el crate existe en el repo; falta verificar si ya está publicado/packaged.
- Reglas vigentes de autopromoción de r/LocalLLaMA y r/LLMDevs: revisarlas el día del post (cambian).
- Precios/empaquetado óptimo del tier de pago: requiere experimentación propia, no hay fuente externa definitiva.
