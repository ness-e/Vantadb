# Plan de Ejecución: Embeddings automáticos (cualquiera del manifest) vía MCP — 2026-09-16

> **Campaign ID:** 6a794efa-ad83-4a19-9fb8-5e6385f36016
> **Inicio:** 2026-09-16
> **Estado:** ⏳ EN PROGRESO (EMB-10 ✅ 2026-09-16; Wave1 en curso)
> **Fuente:** pedido owner 2026-09-16 + FIND-99 (dummy `embed_texts`) + inventario verificado `embeddings/` + Propuesta (matriz REAL/PARCIAL)
> **Autonomous:** false
> **FAIL_MODE:** `parallel` (MAX 3; secuencial interno si colisionan archivos)
> **SPEC:** decisiones Q1-Q5 respondidas por owner vía `question` 2026-09-16 (abajo); sin SPEC.md greenfield adicional (todo fix/wiring sobre comportamiento existente + 2 scripts nuevos).
> **SDP (plan):** base campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full) + por-tarea abajo (≤8, con justificación).
> **Selección owner (Gate P):** Q1 asistente-con-defaults · Q2 3 verificados + resto avisado · Q3 local+ollama/openai · Q4 bloquear+guiar · Q5 fallback avisado con flag.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 11 (EMB-10..20) |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 2 (EMB-17 caché multi-modelo · EMB-11 UX instalador) · ⬇️ resto con contrato mecánico.

## Gate P — decisiones owner (2026-09-16)

Q1→asistente con defaults (Enter=auto, configurable); Q2→3 en disco + resto del manifest solo avisando tamaño/tiempo; Q3→local + ollama/openai (ollama/openai solo si hay servidor/key; si no, documentado-no-ejecutado); Q4→dim distinta bloquea con error claro + comando reindex sugerido (nunca auto-reindex silencioso); Q5→dummy solo como último recurso con `"fallback": true` visible (nunca silencioso, nunca error duro que rompa CI sin modelo).

## Verificación real global (Paso 0, 2026-09-16 — lectura directa + pruebas en vivo)

- `embeddings/models/`: 3 completos en disco (all-MiniLM-L6-v2, multilingual-e5-small default, paraphrase-multilingual-MiniLM-L12-v2; todos 384d, `onnx/model.onnx` + `tokenizer.json` presentes). Manifest 9 modelos (384/512/768/1024/4096).
- Binario actual SIN `embed-local` (0 artefactos `ort`/`tokenizers`/`onnxruntime` en `target/debug/`); default features no lo incluyen (`Cargo.toml:106`); `vanta-cli` requiere solo `cli` (`:302-306`).
- MCP `embed_texts` = dummy hardcodeado (`tools.rs:2607` → `embed_texts_fallback` `:3193-3204`, hash sin semántica). Probado en vivo: sim(gato,felino)=-0.03 vs sim(gato,cuántica)=+0.13.
- `memory_put` (MCP→`Embedded::put`) NO auto-embed (0 llamadas a provider en `src/sdk/`); auto-embed solo existe en IQL (`executor.rs:277-304`, tras `remote-inference`) y query (`physical_plan/vector.rs:56,149`).
- Query MCP pasa hook `None` (`tools.rs:1659`) → recall degrada a keyword (D38). `model` param ignorado (`_model` `:3193`).
- Provider default `ollama` (`config.rs:304`), Ollama NO corre en esta máquina; `VANTADB_LOCAL_MODEL` existe (`config.rs:935-940`, acepta absoluta); path default relativo (frágil por CWD).
- Sin prefijos e5 (`query:`/`passage:`) en `llm.rs` (grep 0 hits) → calidad degradada aunque el modelo cargue.
- Dimensión: error `Vector dimension mismatch` existe (`error.rs:132`) → base monodim; `reindex_hnsw_from_text` existe con tests (FIND-79).
- `aws-lc-rs` ya en el árbol (vía rustls) pero `aws-lc-sys` NO se compila hoy (ring activo) — dato para no prometer swaps gratis.
- Notion: Problema (dimensión semántica: garbage-in = garbage-out → justifica el plan) + Propuesta (retrieval híbrido REAL; `extract_skills`/gobernanza PROPUESTA — fuera de scope). Nuevas features/Plan de accion: sin mapeo → no aplican.

## Tasks

### Wave0 — build (arranca ya, larga, sin dependencias)

**Task EMB-10: build con `embed-local` (+`remote-inference`) + runtime-switchable + cat test**
- Appetite 1d · 🟡 · 🔴 Alta · `Cargo.toml:105-116,302-306`, `src/llm.rs:47-100`, `target/debug/vanta-cli.exe`
- Verificación real: binario sin ort; features existen; dummy probado en vivo.
- Gate Justificación: sin motor en el binario nada posterior funciona; es el desbloqueo raíz.
- Contrato: `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` OK (comando exacto a reverificar en DISCOVERY) + `ort`/tokenizers presentes en el build + con MISMO binario: `VANTADB_EMBEDDING_PROVIDER=local` → cat test con señal (sim(pares)≫sim(impares), umbral a fijar con evidencia) + `VANTADB_EMBEDDING_PROVIDER=ollama` sin servidor → degradación avisada (no crash) → prueba que el switch es solo-config, sin recompilar.
- Pre-mortem: disco C: lleno antes (incidente FIND-94) → `Get-PSDrive` primero + no `clean` sin aviso; build 10-20min (timeout generoso, no matar sano); onnxruntime nativo debe cargar (si falla → dummy, y ESO es hallazgo, no verde).
- Stop: `ort` no compila en este toolchain → Gate V con log (no rabbit hole).
- Risk Register: 🟡×🟢 disco/build largo → chequeo previo + timeouts | 🟢×🟡 onnxruntime no carga → se reporta, no se esconde.
- Cynefin: 🟦 obvio (flags + build + test). ⬆️ 0 / ⬇️ 2 steps.
- DoD: contrato + task file + recitation; commit `feat:` (NO instalar global: PID 3864 tiene el viejo; instalar = EMB-19 coordinado).
- **Hereda:** features workspace, `LocalOnnxProvider`, manifest dims, task FIND-99 (evidencia dummy).
- **Relaciones:** desbloquea EMB-11..20 (todas asumen binario con motor). Nadie depende de EMB-10 para escribirse (11/12 son scripts).
- **Flujo usuario:** ninguno aún (interno). **Flujo caso de uso:** agente pide vectores → motor local responde (sin red).
- **Referencias:** `.opencode/rules/core-engine.md` (llm.rs) + clean-code Apéndice V + Propuesta matriz REAL.
- **Skills (SDP sugerido):** campaign-executor, progreso, systematic-debugging (si falla build), source-driven-development, codebase-memory (blast radius build).
- **Herramientas+MCP:** `cargo build -j 2`, `cargo tree -e features -p vantadb` (verificar flags efectivos), MCP `embed_texts` vía stdio para cat test, `campaign_verify_cmd` (bug exit -1 → bash directa).
- Appetite 1d ≥ 🟡 ✔. Task file `docs/tasks/EMB-10.md` · ✅ COMPLETED 2026-09-16 (`a0f65d29`) · Ruta vanta-worker.
- **Post-cierre EMB-10 (evidencia task file):** fix `token_type_ids` incluido; onnxruntime ≥1.27 OBLIGATORIO (ORT_API_VERSION=27; System32 1.17.1 aborta) → **requisito nuevo para EMB-11** (descargar nativo ≥1.27 + `ORT_DYLIB_PATH`; el ORT 1.30 de Temp es efímero). HALLAZGOS → FIND-100 (alta, abort ort) / FIND-101 (baja, CLI read-only) / FIND-102 (baja, sdk_serialization no compila); FIND-B cubierto por EMB-13 (sin fila).

### Wave1 — instalador (scripts, disjuntos, pueden correr con W0 en vuelo)

**Task EMB-11: asistente de instalación con defaults (Q1+Q2)**
- Appetite 1d · 🟡 · 🟠 · `setup-embeddings.ps1` (nuevo), `embeddings/download.py`, `manifest.json`, `verify.py`
- Gate Justificación: Q1/Q2 owner (auto con Enter, configurable; 3 verificados + resto avisado con tamaño/tiempo).
- Contrato: `-NonInteractive` deja config válida + modelo default presente y verificado (`--check`) + `-Help`/prompts claros + NUNCA escribe secrets a disco (keys solo env de sesión).
- Pre-mortem: descargar modelos grandes sin aviso → tabla tamaño/tiempo + confirmación (Q2);oula/openai piden servidor/key → el wizard los ofrece pero marca requisitos (Q3).
- Risk Register: 🟡×🟢 descarga pesada → confirmación + resume; 🟢×🟡 secreto a disco → prohibido por contrato, review lo audita.
- Cynefin: 🟨 complicado (UX + Q3). ⬆️ 1 / ⬇️ 3 steps.
- DoD: contrato + task file + recitation; commit `feat:`; sin deuda.
- **Hereda:** `download.py --check`, manifest (ids/dims/sizes), `verify.py`, FIND-71 (patterns smoke).
- **Relaciones:** alimenta a EMB-12 (el launcher consume lo que el wizard deja). Paralela con EMB-10/12.
- **Flujo usuario:** instala → corre asistente → Enter Enter → modelo listo + mensaje de éxito con dim/idioma. **Flujo caso de uso:** agente futuro encuentra modelo configurado sin intervención.
- **Referencias:** writing-guidelines (mensajes claros) + security-and-hardening (secrets).
- **Skills:** campaign-executor, progreso, incremental-implementation, test-driven-development, writing-guidelines, security-and-hardening, source-driven-development.
- **Herramientas:** `pwsh -NoProfile`, `python embeddings/download.py --check`, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-11.md` · ⬜ PENDING · Ruta vanta-worker.

**Task EMB-12: script lanzador (env + arranque)**
- Appetite 4h · 🟢 · 🟠 · `vanta-mcp-local.ps1` (nuevo), `opencode.jsonc:77-88`, `config.rs:935-940`
- Gate Justificación: `opencode.jsonc` no tiene ejemplo `environment` (verificado) → env por script (cierto) vs config (no verificado).
- Contrato: el script setea `VANTADB_EMBEDDING_PROVIDER` + `VANTADB_LOCAL_MODEL` (absoluta) y arranca `server --mcp --db <arg>`; `embed_texts` responde dim del modelo activo; secrets (openai key) solo de env de sesión, jamás a disco ni al script.
- Pre-mortem: fijar `--db` por defecto pisa base del usuario → `--db` obligatorio o default temporal explícito.
- Risk Register: 🟢×🟡 db por defecto → arg obligatorio | 🟢×🟢 secreto → audit en review.
- Cynefin: 🟦 obvio. ⬆️ 0 / ⬇️ 2 steps.
- DoD: contrato + task file + recitation; commit `feat:`; sin deuda.
- **Hereda:** bloques MCP `opencode.jsonc`, vars `VANTADB_*`, EMB-11 (lee su config).
- **Relaciones:** consume EMB-11; sirve a EMB-19 (verificación usa el lanzador). Paralela con EMB-10/11.
- **Flujo usuario:** doble clic/comando → server arriba con modelo correcto. **Flujo caso de uso:** OpenCode/otro agente apunta su MCP al lanzador y listo.
- **Referencias:** security-and-hardening + `docs/api/MCP.md:18,218` (stdio, perfil full).
- **Skills:** campaign-executor, progreso, incremental-implementation, security-and-hardening, writing-guidelines.
- **Herramientas:** `pwsh`, MCP stdio smoke (`initialize` + `embed_texts`), `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-12.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave2 — cableado real I (archivos disjuntos: tools.rs / llm.rs / server.rs+docs)

**Task EMB-13: `embed_texts` al proveedor real + fallback avisado (FIND-99 núcleo, Q5)**
- Appetite 1d · 🟡 · 🔴 Alta · `vantadb-mcp/src/handlers/tools.rs:2607,3148-3204`, `src/llm.rs:31-76`
- Gate Justificación: dummy hardcodeado probado en vivo (sim -0.03); Q5 = flag visible.
- Contrato: con modelo real, cat test con señal semántica (umbral con evidencia) + sin modelo → `"fallback": true` en la respuesta (nunca silencioso) + budgeting intacto (128/25k) + tests nuevos.
- Pre-mortem: romper tests EMB-05 que asumen dummy → actualizarlos al comportamiento real (no borrar cobertura).
- Risk Register: 🟡×🟢 tests viejos → se actualizan con evidencia | 🟢×🟢 budgeting → intacto por contrato.
- Cynefin: 🟨 complicado. ⬆️ 0 / ⬇️ 3 steps.
- DoD: contrato + task file + recitation; commit `fix:` (cierra la mitad de FIND-99); sin deuda.
- **Hereda:** `EmbeddingProvider` trait, `McpConfig` budgeting, tests EMB-05, quant `model_qint8` si aplica.
- **Relaciones:** requiere EMB-10 (binario con motor para verificar de verdad). Paralela con EMB-16/18.
- **Flujo usuario:** invisible (misma tool). **Flujo caso de uso:** agente pide vectores → reales con señal; sin modelo → aviso explícito.
- **Referencias:** `.opencode/rules/api-contract.md` (superficie MCP) + `server-mcp.md` + clean-code Apéndice V + doubt-driven-development (falso-positivo es peor que error).
- **Skills:** campaign-executor, progreso, systematic-debugging, test-driven-development, codebase-memory, api-and-interface-design, doubt-driven-development.
- **Herramientas:** `cargo test -p vantadb-mcp`, MCP stdio cat test, `codegraph_explore`, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-13.md` · ⬜ PENDING · Ruta vanta-worker.

**Task EMB-16: prefijos e5 + calidad por familia**
- Appetite 4h · 🟢 · 🟡 · `src/llm.rs` (embed impl, pooling), manifest (familias)
- Gate Justificación: 0 hits `query:/passage:` en `llm.rs` (verificado) → e5 rinde bajo su potencial aunque cargue.
- Contrato: prefijos por familia (e5: `query:`/`passage:`; MiniLM: ninguno) + cat test con margen documentado (números medidos, Regla 11) + tests.
- Pre-mortem: prefijar todo rompe MiniLM → tabla por familia, no global.
- Risk Register: 🟢×🟡 familia equivocada → tabla + test por modelo.
- Cynefin: 🟦 obvio. ⬆️ 0 / ⬇️ 2 steps.
- DoD: contrato + task file + recitation; commit `perf:`/`fix:`; sin deuda.
- **Hereda:** `LocalOnnxProvider::embed`, manifest (familias/dims), pooling existente.
- **Relaciones:** potencia a EMB-13/14/15 (verifican con margen mejor). Paralela (llm.rs vs tools.rs).
- **Flujo usuario:** invisible. **Flujo caso de uso:** misma query, mejores hits.
- **Referencias:** core-engine.md + source-driven-development.
- **Skills:** campaign-executor, progreso, test-driven-development, source-driven-development, systematic-debugging.
- **Herramientas:** `cargo test -p vantadb`, cat test con umbrales, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-16.md` · ⬜ PENDING · Ruta vanta-worker.

**Task EMB-18: regla una-dim-por-base (Q4: bloquear+guiar)**
- Appetite 4h · 🟢 · 🟡 · arranque server MCP (`server.rs`), `error.rs:132` (mismatch existe), `reindex_hnsw_from_text`
- Gate Justificación: Q4 owner (error claro + comando sugerido, nunca auto-reindex silencioso).
- Contrato: base con dim distinta al modelo activo → error que dice dim esperada/obtenida + comando exacto de regeneración + test que lo prueba.
- Pre-mortem: falso positivo en base vacía/mixta-vacía → solo gatear cuando hay vectores.
- Risk Register: 🟢×🟢 base vacía → sin gate.
- Cynefin: 🟦 obvio. ⬆️ 0 / ⬇️ 2 steps.
- DoD: contrato + task file + recitation; commit `feat:`; sin deuda.
- **Hereda:** error mismatch existente, tool reindex con tests (FIND-79), capabilities.
- **Relaciones:** protege cambios de modelo (EMB-11/17). Paralela (server.rs/docs).
- **Flujo usuario:** cambia de modelo → mensaje claro + comando, nada roto en silencio. **Flujo caso de uso:** agente recibe error accionable, no basura.
- **Referencias:** server-mcp.md + documentation-and-adrs + doubt-driven-development.
- **Skills:** campaign-executor, progreso, test-driven-development, documentation-and-adrs, doubt-driven-development.
- **Herramientas:** `cargo test -p vantadb-mcp`, MCP stdio (provocar mismatch), `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-18.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave3 — cableado real II (tools.rs, solo para no colisionar con EMB-13)

**Task EMB-14: auto-embed en `memory_put`/`put_batch` (FIND-99)**
- Appetite 1d · 🟡 · 🔴 Alta · `vantadb-mcp/src/handlers/tools.rs:1303-1360` (`embedded.put/put_batch`), `src/llm.rs`
- Gate Justificación: MCP guarda sin vector siempre (0 llamadas a provider en `src/sdk/`); evidencia GET con `"vector":null`.
- Contrato: put sin vector → guarda CON vector del proveedor activo (get lo muestra) + vector provisto se respeta (no re-embed) + fallo proveedor → guarda sin vector + aviso explícito + tests.
- Pre-mortem: latencia de embed por record en batch → embed_batch (no 1×1) + budgeting existente.
- Risk Register: 🟡×🟢 latencia batch → batch API | 🟢×🟡 doble embed → test vector-provisto.
- Cynefin: 🟨 complicado. ⬆️ 0 / ⬇️ 3 steps.
- DoD: contrato + task file + recitation; commit `feat:` (cierra FIND-99 con EMB-13); sin deuda.
- **Hereda:** `Embedded::put/put_batch`, trait provider, budgeting MCP, tests mcp (91/91 cobertura a no romper).
- **Relaciones:** tras EMB-13 (mismo archivo, orden evita colisión); alimenta a EMB-15/19. Secuencial interno si colisiona.
- **Flujo usuario:** invisible. **Flujo caso de uso:** agente guarda texto → queda buscable por significado sin pasos extra.
- **Referencias:** api-contract.md + server-mcp.md + clean-code Apéndice V.
- **Skills:** campaign-executor, progreso, systematic-debugging, test-driven-development, codebase-memory, api-and-interface-design.
- **Herramientas:** `cargo test -p vantadb-mcp`, MCP stdio (put→get con vector), `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-14.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave4 — cableado real III (tools.rs recall + llm.rs sesiones; secuencial interno si colisionan)

**Task EMB-15: embed de query con el MISMO proveedor**
- Appetite 1d · 🟡 · 🟠 · `tools.rs:1639,1658-1744` (recall/search, hook `None` hoy), `src/physical_plan/vector.rs:56,149`
- Gate Justificación: guardar con un modelo y buscar con otro (o sin ninguno) compara peras con manzanas; hoy degrada a keyword (D38).
- Contrato: `search_memory`/`memory_search` con texto embebe la query con el proveedor activo (mismo que guardado) + prueba de sinónimos (felino↔gato sin palabras comunes) + keyword como fallback solo si no hay proveedor, avisado.
- Pre-mortem: doble cómputo query+docs → batch donde aplique; no tocar ranking RRF.
- Risk Register: 🟡×🟢 mismatch proveedor → factory única, test pareado.
- Cynefin: 🟨 complicado. ⬆️ 0 / ⬇️ 3 steps.
- DoD: contrato + task file + recitation; commit `feat:`; sin deuda.
- **Hereda:** factory única, D38 dual-pool+RRF (intacto), `search_with_method/multi`.
- **Relaciones:** tras EMB-14 (misma zona); verifica con EMB-16 (margen). Paralela con EMB-17 salvo colisión de archivo.
- **Flujo usuario:** invisible. **Flujo caso de uso:** agente busca idea → aparece aunque use otras palabras.
- **Referencias:** api-contract.md + server-mcp.md.
- **Skills:** campaign-executor, progreso, systematic-debugging, test-driven-development, codebase-memory, api-and-interface-design.
- **Herramientas:** `cargo test -p vantadb-mcp`, MCP stdio sinónimos, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-15.md` · ⬜ PENDING · Ruta vanta-worker.

**Task EMB-17: parámetro `model` honrado (switch por nombre)**
- Appetite 1d · 🟡 · 🟡 · `tools.rs:2539-2542,3193` (`_model` ignorado), `src/llm.rs` (sesiones), manifest (nombre→dir)
- Gate Justificación: parámetro público ignorado en silencio (verificado); Q2 promete elegir entre modelos.
- Contrato: `model: "<id del manifest>"` usa ese modelo (caché de sesiones, una por modelo ~450MB c/u, con tope documentado) + id desconocido → error claro con lista válida + test.
- Pre-mortem: RAM ×N modelos → tope + evicción documentada (no silenciosa); dim distinta → EMB-18 gatea.
- Risk Register: 🟡×🟢 RAM → tope+evict avisado | 🟢×🟢 dim → EMB-18.
- Cynefin: 🟨 complicado (caché + concurrencia: `parking_lot::Mutex` por sesión, sin locks nuevos globales — Regla 8). ⬆️ 1 / ⬇️ 3 steps.
- DoD: contrato + task file + recitation; commit `feat:`; sin deuda.
- **Hereda:** manifest (mapa id→dir→dim), `LocalOnnxProvider::new`, EMB-18 (dim gate).
- **Relaciones:** tras EMB-13 (misma zona tools.rs); sirve a EMB-11 (wizard ofrece switch). Secuencial interno si colisiona con EMB-15.
- **Flujo usuario:** pide otro modelo por nombre → funciona o error útil. **Flujo caso de uso:** agente multilingüe ↔ inglés sin reiniciar.
- **Referencias:** api-contract.md + core-engine.md + concurrency (Regla 8: auditar lock-order).
- **Skills:** campaign-executor, progreso, systematic-debugging, test-driven-development, api-and-interface-design, codebase-memory.
- **Herramientas:** `cargo test -p vantadb-mcp`, MCP stdio switch+error, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-17.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave5 — verificación y docs (docs se finalizan tras verify del mismo wave)

**Task EMB-19: verificación punta a punta + reinstalación coordinada**
- Appetite 1d · 🟡 · 🔴 Alta · binario EMB-10, lanzador EMB-12, `~/.cargo/bin/vanta-cli.exe` (bloqueado por PID 3864: coordinar, nunca matar ajeno)
- Gate Justificación: nada cuenta sin prueba e2e con el binario final.
- Contrato: cat test automático verde + sinónimos MCP verde + matriz proveedores (local real; ollama/openai solo si hay servidor/key, si no documentado-no-ejecutado) + suites `mcp_tests`/`memory` verdes + `cargo audit` sin nuevos + binario reinstalado y `tools/list` = 79 vía PATH.
- Pre-mortem: PID 3864 bloquea install → coordinar ventana (no `taskkill` ajeno); ollama/key ausentes → matriz parcial documentada (precedente FIND-69).
- Risk Register: 🟡×🟢 install bloqueado → ventana coordinada | 🟢×🟢 matriz parcial → documentada.
- Cynefin: 🟨 complicado. ⬆️ 0 / ⬇️ 3 steps.
- DoD: contrato + task file + recitation; commit `chore:` (install no se commitea; sí la verificación); sin deuda.
- **Hereda:** todo W0-W4, test-mcp.py por perfil (FIND-82), `validate-docs-coverage`.
- **Relaciones:** cierra la campaña; alimenta a EMB-20 (docs describen lo verificado).
- **Flujo usuario:** reinstala y todo sigue andando mejor. **Flujo caso de uso:** agente con 79 tools reales.
- **Referencias:** task-system.md (verify) + test-suite.md.
- **Skills:** campaign-executor, progreso, test-driven-development, systematic-debugging, codebase-memory.
- **Herramientas:** MCP stdio, `cargo test`, `cargo audit`, `actionlint` si toca yml, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-19.md` · ⬜ PENDING · Ruta vanta-lead (CI/release/instalación).

**Task EMB-20: docs (README + MCP + SKILL)**
- Appetite 4h · 🟢 · 🟢 · `embeddings/README.md`, `docs/api/MCP.md`, `skills/vantadb-mcp/SKILL.md`
- Gate Justificación: lo no documentado no existe para el próximo agente (FIND-67/68/83 doc-driven).
- Contrato: tabla modelo→dim→idioma→tamaño→cuándo usar + cómo activar cada uno (3 proveedores) + regla una-dim-por-base + nota `embed_texts` real + coverage 0 gaps.
- Pre-mortem: documentar comportamiento no verificado → solo lo verde en EMB-19 (Regla 11).
- Risk Register: 🟢×🟢 claims → cada número con fuente.
- Cynefin: 🟦 obvio. ⬆️ 0 / ⬇️ 2 steps.
- DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- **Hereda:** manifest, decisiones Q1-Q5, task files W0-W4 (fuentes).
- **Relaciones:** última; describe lo verificado en EMB-19.
- **Flujo usuario:** lee README y configura sin ayuda. **Flujo caso de uso:** agente lee SKILL y usa bien los límites.
- **Referencias:** documentation-and-adrs + writing-guidelines.
- **Skills:** campaign-executor, progreso, documentation-and-adrs, writing-guidelines, source-driven-development.
- **Herramientas:** `validate-docs-coverage.ps1`, `git diff --check`, `campaign_verify_cmd`.
- Task file `docs/tasks/EMB-20.md` · ⬜ PENDING · Ruta vanta-docs.

## SKIP / DEFER / BLOQUEADO

Nada (Q1-Q5 cerraron el scope; ollama/openai incluidos solo con entorno presente; DEFER-ratificados puntuales viven como salidas válidas dentro de los contratos 10/19).

## Grafo de dependencias / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave0: EMB-10
Wave1: EMB-11 + EMB-12
Wave2: EMB-13 + EMB-16 + EMB-18
Wave3: EMB-14
Wave4: EMB-15 + EMB-17 (secuencial interno si colisionan en tools.rs)
Wave5: EMB-19 + EMB-20 (docs se finalizan tras verify del wave)
```

Órdenes internos: EMB-13 antes que EMB-14/15/17 (mismo archivo); EMB-18 antes que EMB-17 (dim gate); EMB-19 antes de cerrar EMB-20; EMB-10 desbloquea verificación real de 13-17 (sin motor no hay verde verdadero).
FIND-99 (dummy `embed_texts` + sin auto-embed) queda como épica padre: la cierran EMB-13+EMB-14 (su fila ⬜ se marca ✅ al cerrar ambas, sin duplicar tracking).

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Mismo archivo en una wave | waves separan tools.rs (13→14→15/17 secuencial); si colisiona → secuencial interno |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO |
| rustc crash paralelo / OOM | `-j 2` siempre en cargo |
| Build `ort` largo / toolchain C | timebox + Gate V con log (EMB-10 Stop) |
| Disco C: lleno (incidente FIND-94) | `Get-PSDrive` antes de build/descargas |
| Secrets (openai key) a disco | prohibido por contrato (EMB-11/12); review lo audita |
| Modelo dummy silencioso | Q5: flag obligatorio; test semántico lo cazaría |

## Notas

- SKILLS_CARGADAS base sesión: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
- MCPs: CodeGraph/CodebaseMemory (blast radius), campaign (task-system), VantaDB (memoria propia si aplica).
- Routing por tipo: Rust/bindings → vanta-worker; scripts/setup → vanta-worker; docs → vanta-docs; install/release → vanta-lead.
- Backlog: filas EMB-10..20 creadas 2026-09-16 (fuente: este plan). FIND-99 padre de 13/14.

=== RECITATION ===
Objetivo activo: PLAN embeddings-auto (EMB-10..20)
Estado: act (EMB-10 ✅, Wave1 en curso vía sub-agentes)
Última acción: EMB-10 cerrado (`a0f65d29`: build + token_type_ids + cat real) + progreso (Backlog ✅ + avance core-engine) + FIND-100/101/102 + plan anotado
Resultado: ✅
Próxima acción: Wave1 (EMB-11 wizard + EMB-12 launcher en paralelo) — delegadas; luego Wave2
Contrato: plan file con 11 tasks DO + waves + gates; EMB-10 verde verificado
Invariantes: binario global NO instalado (PID 3864); Backlog EMB-10 ✅ + FIND-100/101/102 ⬜; ORT 1.30 Temp efímero (EMB-11 lo re-descarga)
Deuda: ninguna (FIND-100/101/102 trackeados)
Próxima tarea si completa: EMB-13/16/18 (Wave2)
last-synced: 2026-09-16
=== END RECITATION ===

=== RECITATION EMB-10 ===
Campaign ID: 6a794efa-ad83-4a19-9fb8-5e6385f36016
Objetivo activo: EMB-10 build vanta-cli embed-local+remote-inference + runtime-switch + cat test
Estado: completed
Última acción: Steps 1-2 + fix token_type_ids + matriz 3 casos + fmt/clippy/tests/OCR + commit a0f65d29 (2 files, sin push)
Resultado: OK
Próxima acción: Orquestador: push via vanta-lead + filas FIND-A..D + Wave1 EMB-11/12
Contrato: verificacion: cargo build OK 13m36s + libort 4.64MB/libtokenizers 14.40MB + MISMO binario local senal (ranking 12,11,14,13; pares>=0.91 vs impares<=0.78) + ollama-sin-server 200+WARN sin crash + clippy 0 warnings + fmt clean + llm lib 4/4 + OCR sin Critical/High + commit a0f65d29 | evidencia: target/debug/vanta-cli.exe 23.7MB, srv4.log sin Gather-error, HTTP JSON ranking, sanity_embed.py numeros, srv8.log WARN+500-con-server-vivo | artefactos: src/llm.rs (fix), docs/tasks/EMB-10.md, commit a0f65d29 (NO push) | invariantes: no tocar prohibidos (WIP ajeno intacto), no instalar global (PID 3864, EMB-19), ORT 1.30 solo en Temp (fuera del repo) | deuda: FIND-A (ort expect/abort vs graceful), FIND-B (dummy silencioso->EMB-13), FIND-C (query CLI read-only), FIND-D (sdk_serialization no compila); Backlog NO tocado por orden; EMB-11 debe descargar nativo >=1.27 | queda_pendiente: push via vanta-lead; crear filas FIND (orquestador); EMB-11/12 Wave1
Próxima tarea si completa: EMB-11
=== END RECITATION ===
