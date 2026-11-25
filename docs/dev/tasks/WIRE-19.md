---
title: "WIRE-19: dónde vive el wiring del host para que --mcp pueda compartirlo (spec + ADR)"
kind: task
status: complete
description: "La superficie FIND-112 vive en el crate binario vantadb-server y vantadb-mcp no puede alcanzarla. Decisión: los siete items del host se mudan por dueño a vanta-memory (ingest::runner_config + services::scheduler); wire_memory se queda. Entregable: ADR-0063 + tabla de migración ejecutable"
id: WIRE-19
---

# WIRE-19: el hogar del wiring del host

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §9.b (ola MCP/MEM).
- **Fuente:** fila `WIRE-19` de [docs/dev/Backlog.md](../Backlog.md):697-701 (alta 2026-11-23).
- **Esfuerzo:** 🟠 medio — **spec, no implementación** (Appetite 3-4 h).
- **Tipo:** decisión de arquitectura. Entregable = ADR + tabla de migración. **Cero código de producto.**
- **Carril:** A (docs, sin compilar) — `R` en el plan por el blast radius, pero el commit no toca `.rs`.
- **Dependencias:** ninguna para la spec. **Bloquea `MCP-43`** (scheduler en `--mcp`): su interfaz queda definida acá.
- **Orden de ejecución de la migración:** después de `MEM-72` (ambos tocan `vanta-memory/src/ingest/`).
- **Creado:** 2026-11-25.
- **SDP v3** (`campaign_discover_skills_v2`, phase=PLAN, taskType=`MCP server`): base `campaign-executor`, `progreso`, `source-driven-development`; pin `security-and-hardening` (trust boundary); lifecycle `planning-and-task-breakdown`; keyword `documentation-and-adrs`; índice `code-review-and-quality`, `vantadb-mcp`.
  **Cargadas:** `documentation-skill` (obligatoria para `.md` bajo `docs/`), `api-and-interface-design` (core del rol), `documentation-and-adrs`, `codebase-memory` (tools de grafo), `progreso`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Esta tarea (escritura)** | `docs/dev/architecture/adr/ADR-0063-*.md` (nuevo), `docs/dev/tasks/WIRE-19.md` (nuevo). Nada más. |
| **Callers de la superficie a mover** | `vantadb-server/src/main.rs:88-94` (HTTP, dentro de `on_storage_ready`) · `vantadb-mcp/src/wiki.rs:291` (tool `wiki_ingest`) · futuro `vantadb-mcp/src/server.rs:354` (`MCP-43`) |
| **Crates dependientes de `vanta-memory`** | `vantadb-server` (Cargo.toml:33) · `vantadb-mcp` (Cargo.toml:27) · `vanta-proxy` (Cargo.toml:34). Solo los dos primeros cablean el scheduler |
| **Consumidor externo** | `vantadb-pro` (repo aparte, fuera del workspace por regla de open-core). No verificable desde acá → el diseño **no puede** romperlo: de ahí el re-export de compatibilidad |
| **Implicación de seguridad** | El wiring decide **qué fichero de config** honra el proceso y, por transitividad, qué proveedor LLM y qué clave de entorno se usan. Mover esto no es un refactor cosmético |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o por codegraph verbatim):** `vantadb-server/src/scheduler.rs` (192 L) · `vantadb-server/src/main.rs` (L40-119) · `vantadb-server/src/lib.rs` (12 L) · `vantadb-server/Cargo.toml` · `vantadb-mcp/Cargo.toml` · `vanta-memory/Cargo.toml` (L1-60) · `vanta-memory/src/ingest/runner_config.rs` (L62-285) · `vanta-memory/src/ingest/mod.rs` (L19-24) · `vanta-memory/src/services/scheduler.rs` (L1-80, L240-299) · `vantadb-mcp/src/wiki.rs` (L30-40, L285-307, L385-395) · `vantadb-mcp/src/server.rs` (L354, L965, L993-1059) · `src/storage/engine/init.rs` (L120-151, L420-430, L459) · `src/storage/engine/mod.rs` (L336, L505) · `src/sdk/builder.rs` (L97-126) · `Cargo.toml` (L785-811) · `.opencode/rules/{server-mcp,open-core-licensing,api-contract,core-engine}.md` · `docs/dev/architecture/adr/ADR-0054` · `ADR-0062` (estilo) · `docs/dev/architecture/adr/README.md`.
- **Referencias hacia dentro:** `vantadb_server::scheduler::*` es `pub` en la lib del server y la consume su propio `main.rs`; fuera del repo, `vantadb-pro` es el consumidor no verificable. `vanta_memory::ingest::*` es `pub` y `vanta-memory` es **publicable** (DIST-01) → superficie semver.
- **Referencias salientes:** los siete items solo dependen de `IngestRunnerCfg`, `ConcreteRunner`, `build_ingest_runner` y `std::{env, path}`. `wire_memory` depende de `ServerHooks` (seam HTTP) → no se mueve, por neutralidad del host.
- **Veredicto:** **medio en corrección, alto en seguridad.** El riesgo real no es de compilación (el resolver se mueve verbatim con firmas idénticas; los dos wrappers nuevos son delegación pura) sino de **dónde se resuelve la config y dónde se leen credenciales**. El diseño congela ambos: la lectura de `VANTADB_OPENAI_API_KEY` no se toca, y el número de sitios que resuelven `VANTADB_INGEST_CONFIG` baja de dos a uno.

## Spec

### Hallazgo que corrige la premisa de la fila

La fila del Backlog dice que cablear `--mcp` *"obligaría a duplicar el mapeo de proveedores y las reglas R-5 de secretos"*. Medido contra HEAD: **el mapeo de proveedores y la lectura de secretos ya viven en `vanta-memory`** (`runner_config.rs:110,170-237,259-284`; la clave se lee en `:216-217`). Lo que se duplicaría es la **glue fina del host** (~40 líneas): resolución de ruta, parseo del intervalo y el wrapper de degradación P4.

Y de esas tres, **una ya está duplicada hoy, con dos respuestas distintas**:

| Host | Resuelve |
|---|---|
| `vantadb-server` (`scheduler.rs:81-84`) | `<storage_path>/data/vanta-ingest.toml` |
| `vantadb-mcp` (`wiki.rs:394`) | `storage.data_dir()/vanta-ingest.toml` |

Coinciden por accidente en backend de disco (`data_dir == <root>/data`, `init.rs:459`) y **divergen en `BackendKind::InMemory`** (`data_dir` vacío, `init.rs:425-428`): el wiki ingester del MCP lee un `vanta-ingest.toml` relativo al **CWD** mientras el scheduler lee el del storage. Un operador, dos respuestas. Es la clase de drift de `FIND-704`, y está viva.

### El bloqueante real no es "un binario no puede ser dependencia"

`vantadb-server` **tiene** lib (`src/lib.rs`, `pub mod scheduler`) y su `main.rs` la usa como lib. Lo que bloquea es otra cosa:

| # | Bloqueante | Evidencia |
|---|---|---|
| 1 | **Ciclo Cargo** | `vantadb-server/Cargo.toml:31` depende de `vantadb-mcp`; `vantadb-mcp` no declara `vantadb-server` (única mención: un comentario en `:11`) |
| 2 | **Seam equivocado** | lo único host-bound del fichero es `wire_memory`, tipado en `vantadb::cli_server::ServerHooks` (bootstrap HTTP). El path stdio (`server.rs:354`) no tiene `ServerHooks` ni lo tendrá |
| 3 | **Un proceso, un env** | `--http` y `--mcp` son ramas del **mismo binario** (`main.rs:57,60-72,75-99`): `VANTADB_SCHEDULER_INTERVAL_MS` no puede tener dos parsers |

### Tabla de decisión

| Opción | Coste real | Veredicto |
|---|---|---|
| **O1 — cada item con su dueño dentro de `vanta-memory`** (`ingest::runner_config` + `services::scheduler`) | +7 items `pub` en un crate publicable (semver/Hyrum), ~40 LOC movidas + 2 wrappers nuevos de delegación pura, 4 tests movidos, **ningún módulo nuevo** | ✅ **Elegida** |
| O2 — un módulo nuevo `ingest::host` con los siete | los mismos +7 items **más** un `pub mod`, y mete un parser de intervalo de scheduler dentro de `ingest`; compra tidiness con dos costes autoinfligidos y no ahorra nada | ❌ dominada por O1 |
| O3 — crate intermedio (`vanta-host`) | miembro nuevo del workspace + versión + licencia + `cargo-deny`/`machete` + docs + orden de release… para 40 líneas, y con una arista nueva hacia `vanta-memory` (solo necesita **una** dependencia) | ❌ scaffolding (ADR-0031 pone precio a esa promoción) |
| O4 — trait/closure inyectable desde `run_stdio_server*` | rompe la firma `pub` de `vantadb-mcp` (factory genérica → `dyn` o tipo concreto en la firma), y el default standalone vuelve a necesitar la copia dentro de `vantadb-mcp`; mete una Composition Root en una librería (Seemann) | ❌ |
| O5 — feature del binario (el server pasa el wiring) | el entry point no puede construir el factory antes de abrir el engine (writer discovery ocurre en `run_stdio_server_auto`), y en `DatabaseBusy` el proceso es **proxy** donde no puede haber scheduler | ❌ como hogar (su instinto — componer en el entry point — se conserva: `wire_memory` se queda) |
| O6 — copiar las 40 líneas a `vantadb-mcp` | es exactamente lo que ya produjo las dos `ingest_toml_path` divergentes | ❌ |
| O7 — `#[path]` al fichero del server | no compila (importa `ServerHooks`) y rompe `cargo package -p vanta-memory` (Cargo empaqueta solo bajo la raíz del paquete) | ❌ |

### Decisión (detalle completo y consecuencias en el ADR)

Los siete items host-facing de `vantadb-server/src/scheduler.rs:44-115` se mudan al crate que ya posee cada concepto: `ingest_toml_path` **verbatim** + `IngestRunnerCfg::from_storage_path` + `IngestRunnerCfg::runner_factory` (los dos últimos, wrappers nuevos de delegación pura) a `vanta-memory/src/ingest/runner_config.rs` — donde ya viven `from_env_toml`/`apply_env`/`build_runner` —, y `DEFAULT_INTERVAL_MS` + `parse_interval_ms` **verbatim** + `scheduler_interval_ms_from_env_opt` + `scheduler_interval_ms_from_env` a `vanta-memory/src/services/scheduler.rs` (donde vive el loop que gobiernan). El resolver se mueve byte a byte; los dos wrappers nuevos son delegación pura. **Sin módulo nuevo.** `wire_memory` (la composición atada a `ServerHooks`) se queda en el server — por neutralidad del host, no por un ciclo: `vanta-memory` **ya** implementa `ConversationTrigger` (`conversation_hook.rs:47`) y `BackgroundService` (`scheduler.rs:216`) bajo `http-server`. El re-export de compatibilidad va **dentro de `vantadb-server/src/scheduler.rs`** (no en `lib.rs`: el módulo es un fichero, y un `pub use` en la raíz publicaría `vantadb_server::ingest_runner_cfg`, no `::scheduler::…`) y cubre **los seis** nombres que el módulo exportaba. Y `MCP-43` cablea dentro de `run_stdio_server` — la rama writer — con dos guardas obligatorias: `!storage.config().read_only`, y **no** arrancar el loop en el host one-shot de `vanta-cli mcp-call` (`src/cli_handlers/mcp_call.rs:262`), que si no dispararía un pase de ingest por cada hook de terminal.

**Requisito duro verificado:** ningún secreto cambia de sitio. `VANTADB_OPENAI_API_KEY` se lee en `runner_config.rs:216-217` antes y después; el esquema TOML no tiene campo de clave; la única variable que se mueve es `VANTADB_INGEST_CONFIG`, que es una **ruta**. Las reglas R-5 (prefijo único, `warn!` + default, nunca panic) y la regla G0 de `FIND-112` (el TOML nunca lleva clave) se preservan sin cambios de comportamiento.

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Discovery: leer plan §9.b, fila del Backlog, reglas (`server-mcp`, `open-core-licensing`, `api-contract`, `core-engine` R-5), ADR-0054 y ADR-0062 (estilo) | ✅ |
| 2 | Trazar el código real con `codegraph_codegraph_explore` (`ingest_runner_factory`, `ingest_toml_path`, `init_storage`, `Embedded::from_engine`, `spawn_memory_scheduler`) + `rg` de respaldo | ✅ |
| 3 | Verificar los bloqueantes uno por uno (ciclo Cargo, seam `ServerHooks`, mismo binario) y desmentir el "un binario no puede ser dependencia" | ✅ |
| 4 | Medir la superficie: `rg -n 'VANTADB_INGEST\|VANTADB_LLM\|ingest_toml_path\|ingest_runner_factory' --glob '*.rs' \| wc -l` → **49** (primera medición de la sesión: 47; ver nota del contrato) | ✅ |
| 5 | Verificar el requisito duro de secretos contra el código (no contra la prosa): lectura única, esquema TOML sin campo de clave | ✅ |
| 6 | Escribir `ADR-0063` (MADR: contexto, 7 opciones, decisión, consecuencias positivas **y negativas**, tabla de migración, verificación, referencias) | ✅ |
| 7 | Validar el ADR mecánicamente (secciones, ≥2 opciones, links, schema) | ✅ |
| 8 | Review P2-01 con `vanta-review` (agente distinto, ronda 1) → 🔴 CAMBIOS REQUERIDOS (4 high, 5 medium, 5 low), todos incorporados en la revisión del ADR | ✅ |
| 9 | Commit local + fila de `progreso` (Backlog y `avance/` están fuera del write-scope de esta tarea: la fila se entrega en el RESULTADO) | ⬜ |

## Contrato mecánico

| Check | Comando | Resultado |
|---|---|---|
| Superficie sin cambios (no toqué código) | `rg -n 'VANTADB_INGEST\|VANTADB_LLM\|ingest_toml_path\|ingest_runner_factory' --glob '*.rs' \| wc -l` | **49 → 49** (dos métodos independientes: `Measure-Object -Line` y `.Count`). La primera medición de la sesión dio 47: la diferencia son 2 hits de `vanta-memory/src/ingest/observability.rs`, **untracked y en vuelo de `MEM-72`** — no de esta tarea. Prueba de que no toqué `.rs`: `git status` no muestra ningún `.rs` modificado por mí |
| Estructura del ADR (no es de una sola opción) | `rg -c '^### O[0-9]+ ' ADR-0063…` | **7** |
| Secciones MADR obligatorias | `rg -c '^## (Context and problem statement\|Considered options\|Decision outcome)$'` + `^### (Positive\|Negative) consequences$` | 3 + 2 |
| Docs schema + links | `node scripts/docs/check-docs.mjs && node scripts/docs/check-links.mjs` | exit **0** |
| Sin tocar `.rs` | `git status --short` | solo los dos `.md` nuevos |

## Review (P2-01)

Detalle completo (las 15 findings con su resolución) en el [ADR-0063](../architecture/adr/ADR-0063-host-wiring-home-vanta-memory.md) §Review.

| Campo | Valor |
|---|---|
| Reviewer | `vanta-review` (contexto fresco, agente distinto del autor) |
| Ronda | 1 |
| Veredicto ronda 1 | 🔴 **CAMBIOS REQUERIDOS** — 4 high, 5 medium, 5 low; ninguno critical. Contrato mecánico ✅ (docs/links exit 0); seguridad ✅ (secreto con lectura única, TOML sin campo de clave, sin env nuevas) |
| Resolución ronda 1 | Todos incorporados: re-export movido a `scheduler.rs` (H1), +6 items public (H2), D2 reescrita sin el ciclo falso (H3), guardas `read_only` + `mcp-call` one-shot (H4), fila 1 con imports `std` (M1), fila extra para `CONFIGURATION.md:136` (M2), fila 8 borrada — su premisa era falsa (M3), todas las citas re-medidas sobre `HEAD` (M4), O3 con una dependencia (M5), y la alternativa "colocar por dueño" **adoptada como O1** |
| Veredicto ronda 2 | 🔴 **CAMBIOS REQUERIDOS** — 2 high, 2 medium, 4 low. Aprobado el diseño (O1) y cerrados H2/H3/M1/M2/M3/M5; abiertos: el re-export no cubría `ingest_runner_factory` (H-A) y "unset means off" no era implementable sin un segundo `env::var` (H-B, contra R-5) |
| Resolución ronda 2 | Shims para las **dos** funciones que no se pueden `pub use` y check de compat con los seis nombres (H-A); `scheduler_interval_ms_from_env_opt() -> Option<u64>` como primitiva — un solo read site — y `scheduler_interval_ms_from_env()` como su `unwrap_or` para el daemon HTTP, lo que sube la superficie a **+7** (H-B); claim "verbatim/firmas idénticas" corregido (M-A); contrato con 7 opciones (M-B); `:235` → `:216` sobre `HEAD` (M-C) y las cuatro citas menores (L-A…L-D) |
| Veredicto final | ✅ **APROBADO** (ronda de cierre) — H-A y H-B cerrados; el diseño O1 aprobado; contrato mecánico ✅; secretos ✅ sin cambios. Quedan 6 observaciones low de conteo/redacción, todas aplicadas antes del commit |
| Payload P2-01 | `{mode:'fresh', reviewer:'vanta-review', reviewer_context:'sesión de review con contexto fresco, sin acceso al contexto del autor', author_context:'vanta-arch', verdict:'approve'}` |

**Lo que el review confirmó sin cambios** (no re-litigar): el ciclo Cargo, el punto único de lectura del secreto y la ausencia de campo de clave en el TOML, la divergencia de rutas al pie de la letra (incluido que `storage_root` no normaliza ni expande `~`, así que el "agree by accident" es byte a byte), que `&storage.config().storage_path` compila en `wiki.rs:291`, que ningún item movido necesita feature gate, los rangos de borrado de la fila 3, y el punto de inserción alfabético.

## Resultado

**Decisión en una frase:** Los siete items host-facing de `FIND-112` se mudan de `vantadb-server/src/scheduler.rs:44-115` al crate que ya posee cada concepto — `ingest_toml_path` y `parse_interval_ms` verbatim, más los wrappers nuevos `IngestRunnerCfg::from_storage_path`, `IngestRunnerCfg::runner_factory`, `scheduler_interval_ms_from_env_opt` y `scheduler_interval_ms_from_env` (todos delegación pura) — repartidos entre `vanta-memory/src/ingest/runner_config.rs` y `vanta-memory/src/services/scheduler.rs` (sin módulo nuevo, sin feature gate); `wire_memory` se queda en el server, el re-export de compatibilidad vive dentro de `vantadb-server/src/scheduler.rs` y cubre los seis nombres viejos, y `vantadb-mcp` consume la superficie compartida desde `run_stdio_server` — la rama writer, con guardas de `read_only` y de host one-shot.

**Lo que queda pendiente (esto desbloquea, no implementa):** la migración de 7 filas (§Migration surface del ADR) y `MCP-43`. La fila `WIRE-19` **sigue pendiente de implementación** hasta que la migración aterrice; lo que se cierra acá es la decisión y su interfaz.

## Referencias

- [ADR-0063](../architecture/adr/ADR-0063-host-wiring-home-vanta-memory.md) — la decisión, las 7 opciones y la tabla de migración ejecutable.
- [ADR-0054](../architecture/adr/ADR-0054-scheduler-host-vantadb-server.md) — hogar del scheduler (`vantadb-server`); este ADR lo acota, no lo supersede.
- `docs/dev/Backlog.md:697-703` — filas `WIRE-19` / `MCP-43` (contrato y DoD).
- `.opencode/rules/server-mcp.md` · `.opencode/rules/open-core-licensing.md` · `.opencode/rules/api-contract.md` (R-4 rutas reales, R-7 `cfg` en campos `pub`) · `.opencode/rules/core-engine.md` R-5.
