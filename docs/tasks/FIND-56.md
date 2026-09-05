# FIND-56: `vantadb-server/Dockerfile` roto — DEPRECAR a favor del Dockerfile raíz (Gate D N/A, decisión en DISCOVERY)

## Metadata
- **Plan file:** — (ejecución directa desde Backlog L219, orden del orquestador 2026-09-03; Gate D N/A: brief delega la elección (a)/(b) a DISCOVERY con evidencia; blast radius 5 archivos vivos, 0 símbolos públicos nuevos, 0 hot paths)
- **Creado:** 2026-09-03
- **last-synced:** 2026-09-03
- **Estado:** ✅ COMPLETO (2026-09-03 — Steps 1-3 DONE, contrato verificado, commit scoped; verificación build/smoke diferida a CI job `docker-image` — sin daemon en host)
- **SDP:** campaign-executor + progreso + ponytail (base auto); lifecycle: incremental-implementation (1 slice deprecación) + context-engineering (context pack §3a); test-driven-development N/A con rationale (0 Rust, 0 lógica: la verificación es mecánica rg + parse YAML — precede MKT-18i); source-driven-development N/A (sin APIs externas; Docker Compose spec verificada contra composes existentes del repo); security-and-hardening: checklist N/A con rationale (cambio no toca trust boundaries de código — imagen runtime raíz intacta, mismos puertos/env/volúmenes; hardening runtime se preserva en compose). `concurrency-async.md`/otras rules: N/A (0 Rust, 0 Tokio).
- **Sub-agente:** vanta-worker
- **Área:** `vantadb-server/` (Dockerfile eliminado, composes retargeteados) + `docs/operations/hardening.md` §5 + `docs/operations/DEPLOYMENT_GUIDE.md` L231-232 (1 línea, puntero vivo — exige el contrato 0-referencias). NADA más. Paralelo declarado: NO tocar `src/ingestion.rs` (FIND-57), `release-wheels*`, `src/cli.rs` (solo lectura como evidencia), root `Dockerfile` (solo lectura, SRV-07 ya aplicado), `.github/workflows/*` (no se tocan → actionlint N/A), stash@{0}, `completions/*`, `Cargo.lock`, `.opencode` (no stagear).

## Blast Radius (Discovery con evidencia mecánica)

- **Bug #1 (fatal primero):** `vantadb-server/Dockerfile:20` `COPY vantadb/Cargo.toml vantadb/` — `Test-Path vantadb` = **False**; el crate raíz es `.` (`Cargo.toml:1` `[package] name = "vantadb"`, workspace members `"."` L677). El build muere en la capa de manifests.
- **Bug #2 (fatal segundo):** `cargo build --package vantadb-server` (L36) produce el binario `vantadb-server` (`vantadb-server/src/main.rs`, sin `[[bin]]` → nombre = paquete), pero L52 copia `/build/target/release/vanta-cli` (ese bin pertenece al paquete raíz `vantadb`, `[[bin]] name="vanta-cli"` L285-289). Artefacto inexistente → COPY falla aunque #1 se arregle.
- **Entrypoint coherente con binario equivocado:** `vanta-cli server --http ...` SÍ existe como invocación (`src/cli.rs:314` `Server { http, mcp, port... }`), pero el binario `vantadb-server` real solo entiende `--mcp/--help` y config por env (`main.rs:25,52`). La imagen, de construirse, mezclaría binario+flags de dos runtimes distintos.
- **Bug #3 (release-binary muerto):** L94-99 espera `vantadb-server-<VERSION>-<ARCH>-unknown-linux-gnu.tar.gz`, pero `release-binaries-63.yml:124` sube `vantadb-<target>.tar.gz` (ambos binarios dentro). URL 404 garantizado. Además el release ya publica la imagen docker completa como asset (SRV-07, `63.yml:178-184`).
- **Capacidad real del Dockerfile raíz (solo lectura):** build `--package vantadb-server` (`Dockerfile:37`), `COPY . .` + cache mounts BuildKit (SRV-07), runtime `USER vantadb` + `ARG VANTA_RUNAS_UID=1001` + data dir 0777 (L50-66), OCI labels, `ENTRYPOINT ["vantadb-server"]` (env-driven = el binario real). CI `docker-image` (`63.yml:156-184`) buildea LA RAÍZ (`docker build ... .`) + smoke unprivileged (`--user 10001:10001` write-test + `--help`) + export como asset. **Ningún workflow referencia `vantadb-server/Dockerfile`** (`rg` en `.github/` = 0 hits de path; solo comentario SRV-07) → eliminarlo no rompe CI.
- **Decisión (b) DEPRECATE — Ponytail (borra más sin perder capacidad):** las 3 capacidades únicas del server-Dockerfile están muertas o subsumidas — build estándar → raíz (mejor: cache, labels, smoke en CI); `unprivileged` como target → runtime flags (`--read-only --cap-drop=ALL`, ya en compose + DEPLOYMENT_GUIDE §3, funcionan con cualquier imagen); `release-binary` → URLs 404 + redundante con imagen-como-asset. Reescribirlo (opción a) duplicaría ~112L para replicar lo que la raíz ya hace y CI ya testea.
- **Composes:** `vantadb-server/docker-compose.yml` (2 servicios) + `docker-compose.prod.yml` (override, `target: release-binary` muerto) se retargetean a `Dockerfile` raíz (`context: .., dockerfile: Dockerfile`), preservando puertos/env/volúmenes/hardening runtime (read_only, cap_drop, no-new-privileges, tmpfs, resources). Se elimina `target: unprivileged` (no existe en raíz; el hardening vive en runtime, no en stage) y el bloque `args: VERSION/TARGETARCH` (muerto con release-binary). Nota: `version: "3.9"` + `profiles:` se preservan tal cual (sin normalización oportunista — scope discipline).
- **Docs:** `hardening.md` §5 se reescribe sobre la imagen canónica (build/run/compose/unprivileged-vía-runtime/prod=imagen-del-release + nota de deprecación con fecha); `DEPLOYMENT_GUIDE.md:231-232` 1 línea (puntero vivo al archivo eliminado — el contrato 0-referencias lo exige; es el mismo bug-clase que FIND-56).
- **Historia intacta (no reescribir):** `.opencode/.../SRV-07.md`, `docs/avance/activo/operaciones.md:178` (deuda SRV-07), `docs/plans/archive/*`, `docs/reviews/archive/*` conservan sus menciones fechadas — son registro, no punteros vivos. El `rg` de cierre las reportará como hits históricos exentos.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-server/Dockerfile` (112L), `vantadb-server/docker-compose.yml` (95L), `vantadb-server/docker-compose.prod.yml` (45L), `Dockerfile` raíz (89L, lectura), `docs/operations/hardening.md` §5 (L195-252), `docs/operations/DEPLOYMENT_GUIDE.md` L206-232, `vantadb-server/src/main.rs` (vía lectura parcial 1-80 + validate_args), `src/cli.rs:314-326` (lectura puntual), `src/bin/vanta-cli.rs` (rg puntual), `.github/workflows/release-binaries-63.yml` L103-184 (rg puntual), `.dockerignore`, `Cargo.toml` (workspace/members/bins), `vantadb-server/Cargo.toml`, `docs/Backlog.md` L219, `docs/avance/activo/operaciones.md` L162-178.
- **Referencias hacia dentro:** el Dockerfile server no es importado por código (0 Rust); sus consumidores son los 2 composes + hardening §5 + DEPLOYMENT_GUIDE L231 (vivos) + historia (ver arriba).
- **Referencias entrantes al path literal (vivos):** hardening.md ×3, DEPLOYMENT_GUIDE ×1, composes ×3 (los propios), Backlog FIND-56 (se elimina al cierre).
- **Veredicto:** blast radius = 1 eliminación + 2 composes + 2 docs (1 rewrite §5 + 1 línea). 0 Rust, 0 tests, 0 workflows, 0 símbolos públicos. Rollback-friendly: `git mv` inverso + revert de 4 ediciones.

## Contrato

`rg "COPY vantadb/" vantadb-server/Dockerfile` → archivo inexistente (eliminado) Y `rg -l "vantadb-server/Dockerfile" --glob '!docs/plans/archive/**' --glob '!docs/reviews/archive/**'` = solo hits históricos exentos (SRV-07.md, avance/operaciones.md:178 — fechados) con 0 punteros vivos Y `hardening.md` §5 coherente con la imagen raíz (comandos build/run/compose verificables contra `Dockerfile`+composes) Y sin daemon docker en host → validación sintáctica: parse YAML de ambos composes vía Python + `rg` de coherencia (tabla abajo), nota "verificación diferida a CI" (job `docker-image`), NUNCA fakes Y actionlint N/A (0 workflows) Y cargo fmt/clippy/nextest N/A (0 Rust).

| Cláusula | Comando | Esperado |
|---|---|---|
| Eliminación | `Test-Path vantadb-server/Dockerfile` | False |
| 0 COPY roto | `rg "COPY vantadb/" vantadb-server/` | 0 / sin archivo |
| 0 punteros vivos | `rg -l "vantadb-server/Dockerfile"` (excl. archives) | solo historia exenta |
| Composes parsean | python PyYAML `safe_load` ×2 | OK ×2 |
| Composes apuntan a raíz | `rg "dockerfile: Dockerfile" vantadb-server/docker-compose*.yml` | 2 |
| Sin targets muertos | `rg "release-binary|target: unprivileged" vantadb-server/docker-compose*.yml` | 0 |
| hardening honesto | `rg "vantadb-server/Dockerfile" docs/operations/hardening.md docs/operations/DEPLOYMENT_GUIDE.md` | 0 |

Commit: `fix(server): Dockerfile roto - deprecado a imagen raiz con hardening.md honesto (FIND-56)` — `fix:` → release-plz patch. Solo archivos del blast radius (nunca `completions/`, `Cargo.lock`, `.opencode`, stash).

## Herramientas

- read, bash (rg, Test-Path, python yaml), edit/write (composes, docs), git (rm + add scoped + commit por orden explícita del brief — precedente FIND-57)

## Steps

### Step 1: Eliminar Dockerfile + retargetear composes
- **Archivos:** `vantadb-server/Dockerfile` (git rm), `vantadb-server/docker-compose.yml`, `vantadb-server/docker-compose.prod.yml`
- **Acción:** `git rm vantadb-server/Dockerfile`; en compose.yml ambos `build:` → `context: ..` + `dockerfile: Dockerfile`, quitar `target: runtime-base`/`unprivileged` (preservar todo lo demás incl. `version:`, profiles, hardening runtime); en prod.yml `build:` → raíz sin `target`/`args` (preservar env prod + hardening + deploy resources).
- **Estado:** ✅ DONE (2026-09-03: `git rm` ejecutado en sesión; al verificar el commit se detectó que la baja trackeada YA estaba registrada en `0a54a545` (HEAD~1, pre-existente: 112 deletions) y la copia del worktree era resurrección no-trackeada — `git rm` la eliminó y el estado final es consistente en worktree+index+HEAD; `ae3746b4` completa la deprecación con los 6 archivos vivos. Sin colateral: stash intacto, completions/.opencode no stageados)

### Step 2: hardening §5 honesto + DEPLOYMENT_GUIDE 1 línea
- **Archivos:** `docs/operations/hardening.md` (§5 rewrite), `docs/operations/DEPLOYMENT_GUIDE.md` (L231-232)
- **Acción:** §5 documenta imagen canónica raíz (build `docker build -t vantadb-server .`, run, unprivileged vía runtime flags, compose profiles contra composes retargeteados, prod = asset imagen del release) + nota deprecación `vantadb-server/Dockerfile` (fecha + motivo triple-bug + puntero a raíz); GUIDE L231-232 apunta a hardening §5 sin nombrar el archivo eliminado.
- **Verify:** fila 7 (`rg` = 0 en ambos docs) + coherencia manual comandos-vs-Dockerfile/composes
- **Estado:** ✅ DONE (2026-09-03: §5 reescrito sobre imagen canónica + nota deprecación; tabla comparativa corregida; GUIDE 1 línea; `rg vantadb-server/Dockerfile|--target` en ambos docs + composes = 0)

### Step 3: Cierre
- **Archivos:** `docs/Backlog.md` (fila FIND-56 eliminada), `docs/avance/activo/operaciones.md` (entrada; dominio ops por tabla progreso — docker cae en "Ops"; alternativa ci-cd: se anota operaciones por ser hardening/deploy), memoria `decisions`, commit scoped
- **Verify:** contrato completo + `git status --short` (solo archivos del blast radius + Backlog + avance) + commit hash
- **Estado:** ✅ DONE (ver abajo: batería final + commit)

## Dependencias

- SRV-07 (landed: imagen raíz + job CI `docker-image`) — prerrequisito, existe. FIND-59/FUT-12, FIND-57: sin relación (paralelo declarado, archivos prohibidos intactos).

## Notas

- Sin daemon docker en el host (`docker` no reconocido) → build/smoke reales imposibles localmente; el gate real es el job `docker-image` de CI en el próximo dispatch (mismo diferimiento que SRV-07 y MKT-18i — precedente aceptado). Nota explícita en avance + RESULTADO.
- `ponytail:` no aplica como comentario en código (0 código) — la simplificación (borrar 112L + 1 stage muerto) ES la decisión, registrada aquí y en decisions.

## Context Save Point
- **Fecha:** 2026-09-03
- **Branch:** develop (ahead 4, sin cambiar)
- **CI pendiente:** job `docker-image` en próximo tag/dispatch (verificación diferida)
- **Decisiones:** (b) DEPRECATE con evidencia triple-bug arriba; compose.prod retarget (no borrado: env prod + limits son capacidad viva); historia no reescrita
- **Problemas conocidos:** ninguno bloqueante
- **Próxima tarea:** Step 1
