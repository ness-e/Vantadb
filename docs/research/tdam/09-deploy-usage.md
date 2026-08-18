# TDAM — 09: Deploy + Uso + Scripts + SDK — Investigación profunda

> **Serie:** Investigación fraccionada TDAM (TencentDB Agent Memory) · **Área:** despliegue, operación, plugins, scripts, CLI, SDK · **Repo:** TencentDB-Agent-Memory @ `97f9465` (rama `feat/server_team`) · **Verificación:** 100% contra clone local `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` · **Fecha:** 2026-08-18 · **Estado:** ✅ COMPLETO

## 1. Resumen ejecutivo

Esta área estaba **descubierta** en la serie fraccionada: cubre cómo se despliega y opera TDAM, sus plugins de agente, scripts de mantenimiento, CLI de seed y el SDK. El hallazgo principal: TDAM **no es un solo servicio** sino un stack de 3 imágenes (`memory-core` + `memory-hub` + `proxy`) que se levanta con 3 scripts shell y se consume por un coding agent **sin cambiar protocolo** (solo apuntar `base_url` al proxy). No hay `.github/workflows` en este commit.

## 2. Despliegue

### 2.1 `deploy/global-images/` — scripts shell, NO docker-compose
Contiene: `start-all.sh`, `start-memory-core.sh`, `start-memory-hub.sh`, `start-proxy.sh`, `stop-all.sh` (con `--purge`), `verify.sh` (pre-check LLM, `--skip-llm`), `_lib.sh`, `.env.example`, `README.md`. Flujo: `cp .env.example .env` → `./verify.sh` → `./start-all.sh` (imprime bloque listo para pegar en Claude). Al primer arranque ejecuta `init-admin` (crea `system_admin`, genera `user_key` aleatoria de 32 chars en `./.admin-key`) y valida con `POST /v3/meta/auth/verify`.

**Puertos:** MemoryCore `8420` · Panel `8125` · Knowledge (KS) `8424` · Proxy `8096` (todos configurables en `.env`).

**Env vars clave:** grupo memory (`MEMORY_LLM_BASE_URL/API_KEY/MODEL/PROTOCOL`) y grupo proxy (`PROXY_UPSTREAM_URL/API_KEY/MODEL`) — dos grupos LLM **independientes**; credenciales internas `MEMORY_CORE_GATEWAY_API_KEY`, `MEMORY_CORE_ADMIN_USERNAME/USER_KEY` (defaults `local`/`admin`/`admin` solo para local); `KNOWLEDGE_PUBLIC_BASE_URL` (debe contener `/v3`), `MEMORY_HUB_PROXY_PUBLIC_URL`; volúmenes named `tdai-memory-core-data` y `tdai-panel-data`.

### 2.2 `deploy/panel-knowledge-combined/` — imagen "Memory Hub"
Merge de Panel (Team Memory Control, 8125) + Knowledge Service (KS: wiki/code graph, 8424) en un contenedor (`agentmemory/memory-hub`). Requiere 3 inputs: `metadata-instances.json` montado en `/app/panel/config/`, `KNOWLEDGE_PUBLIC_BASE_URL` (con `/v3`), `KNOWLEDGE_LLM_PROXY_BASE_URL`. Modo `LLM_MODE=custom` permite BYO LLM. Incluye `Dockerfile`, `build.sh`, `publish.sh` (buildx multi-arch + secret-scan), `start-combined.sh`, `.dockerignore`. Datos en `/data/knowledge` (SQLite, git clone, wiki, logs).

### 2.3 `deploy/dockerhub/`
`publish.sh` + `README.md`: publica las 3 imágenes multi-arch (`linux/amd64,linux/arm64`) en namespace `agentmemory` (`memory-core`, `memory-proxy`, `memory-hub`). `VERSION` obligatorio (rechaza `dev-`), `DRY_RUN=1` solo escanea secretos; `cost-guard` (MemoryProxy) e `integrations` (MemoryCore) quedan fuera de la imagen pública con stubs/fallback.

### 2.4 Requisitos
Node.js `>= 22.16.0` + npm; LLM API OpenAI-compatible (obligatorio para extracción); Redis (store por defecto del proxy, conmutables a COS/SQLite/FS/Memory); MemoryCore standalone no requiere nada externo salvo el LLM. **No hay evidencia de MongoDB** en el repo. Modos de deploy: `README.deployment.md` documenta **Standalone** (SQLite + local files, `TDAI_DEPLOY_MODE`) vs **Service** (TCVDB + COS + Redis, multi-tenant).

## 3. Uso y operación (docs raíz)
`README.md` (quickstart 1-comando con `start-all.sh`), `INSTALL.md` (827 líneas: 3 modos — stack completo, Memory Hub solo, proxy con Claude Code; tablas por agente: DeepSeek Harness/Claude Code/Codex/CodeBuddy/WorkBuddy/Hermes/OpenClaw), `INSTALL_CN.md`, `README.docker.md`, `README.deployment.md`, `README_CN.md`, `CHANGELOG.md`, `ROADMAP.md`. Uso típico: abrir Panel `:8125`, login con `user_key`, crear Team/Agent/Task, y apuntar el agente al proxy (`ANTHROPIC_BASE_URL=http://localhost:8096/claude-code/default`).

## 4. Plugins
- **hermes-plugin** (`MemoryCore/hermes-plugin/memory/memory_tencentdb/`): provider Python `MemoryProvider` para Hermes — cliente HTTP fino + `GatewaySupervisor` que arranca el sidecar Node `:8420`; mapea hooks Hermes → endpoints (`prefetch`→`POST /recall`, `sync_turn`→`POST /capture`, `shutdown`→`POST /session/end`); circuit breaker (5 fallos → 60s pausa), back-pressure (máx 4 threads). Se instala por symlink/copy a `hermes-agent/plugins/memory/memory_tencentdb/` (nombre de dir exacto) o vía `scripts/install_hermes_memory_tencentdb.sh`.
- **openclaw-plugin** (`MemoryCore/openclaw-plugin/`): adaptador v3 `memory-tencentdb-client` (SDK npm `@tencentdb-agent-memory/memory-sdk-ts-v2@1.0.0-beta.2`). Hooks: `capture.ts` (agent_end → addConversation L0), `recall.ts` (before_prompt_build → search + inyección). Tools: `tdai_memory_search` (L1), `tdai_conversation_search` (L0), `tdai_read_cos` (archivos vía COS STS). Dual mode `local` (127.0.0.1:8420, apiKey "local") vs `server`. Instalador: `scripts/install-openclaw-plugin.sh` (configura `~/.openclaw/openclaw.json`; campos `hooks.*` version-gated desde OpenClaw `2026.4.24`).

## 5. Scripts y CLI (MemoryCore)
**scripts/**: `bench-checkpoint-lock.ts` (benchmark lock de checkpoint) · `probe-vdb-capacity.ts` / `probe-vdb-reclaimable.ts` / `cleanup-vdb-test-dbs.ts` (probe/limpieza TCVDB) · `start-e2e-gateway.ts` (gateway + init-admin para e2e) · `e2e-memory-prompt-vdb-cos.ts` · `verify-clear-vs-archive.ts` / `verify-tcvdb-clear.ts` · `install-openclaw-plugin.sh` / `install-hermes-plugin.sh` / `install_hermes_memory_tencentdb.sh` · `import-opik-to-memory-core/` · `import-opik-to-memory-skill-py/` · `migrate-v2-to-v3/` · `migrate-sqlite-to-tcvdb/` · `ci/check-skill-queue-isolation.sh`.
**bin/**: `seed-v2.mjs` (launcher thin de seed v2), `read-local-memory.mjs`, `migrate-sqlite-to-tcvdb.mjs`, `export-tencent-vdb.mjs`.
**CLI**: `src/cli/index.ts` registra el namespace `openclaw memory-tdai` (Commander); `cli/commands/seed.ts` implementa `seed --input <file>` que ejecuta L0→L1→L2→L3 (formatos A/B JSON, `--output-dir`, `--config` deep-merge, `--strict-round-role`, `--yes`); salida: `conversations/ records/ scene_blocks/ vectors.db .metadata/`.

## 6. SDK (`sdk/memory-core`)
- **typescript/** (`@tencentdb-agent-memory/memory-sdk-ts-v2`): `MemoryClient` v3 con isolation estricta (`teamId/agentId/userId` obligatorios). Sub-clientes v3: `skill-client.ts`, `metadata-client.ts`, `memory-prompt-client.ts`, `memory-generation-log-client.ts`, `client.ts`, `cos.ts`. Ejemplo real: `new MemoryClient({endpoint:"http://127.0.0.1:8420", apiKey, serviceId, teamId, agentId, userId})` → `addConversation`, `searchAtomic`, `readScenario`, `readCore`.
- **python/** (`tencentdb_agent_memory`, dist `tencentdb-agent-memory-sdk-python`): `MemoryClient` (sync) + `AsyncMemoryClient`; paquetes `v2/client.py` (conversation/atomic/scenario/core/offload compact+ingest/read_file) y `v3/` (`client.py`, `skill_client.py`, `metadata_client.py`, `memory_prompt.py`, `memory_generation_log.py`) + `cos.py`.

## 7. GitHub Actions/CI
`.github/workflows/pr-ci.yml` **SÍ existe** (159 líneas — CI real para PRs a main, verificado por lectura directa; el glob inicial del agente no lo detectó). Además `MemoryCore/scripts/ci/check-skill-queue-isolation.sh` como check de CI local. Assets: `assets/images/` (logo, agentes) referenciados en README.

## 8. Integración en VantaDB
**NO copiar:** el stack de 4 servicios/puertos ni los 3 contenedores. **Sí como referencia directa:**
- Patrón "uso": un coding agent consume memoria **sin cambio de protocolo** — `setup-claude-code.sh` (escribe `env.ANTHROPIC_BASE_URL` + `ANTHROPIC_CUSTOM_HEADERS` en `~/.claude/settings.json`) → aplicable al MCP de VantaDB (`base_url` hacia vanta-server).
- `MemoryProxy/Dockerfile` (node:22-slim, multi-stage, tini, HEALTHCHECK `/health`, config montada en `/data/config.yaml`) como patrón de imagen para vanta-server.
- CLI `seed` (importar conversaciones históricas → pipeline L0-L3) como modelo para el comando de import de VantaDB.
- SDK memory-core (Python + TS con sub-clientes) como referencia estructural para `sdk/python` y `sdk/node` de VantaDB.

## 9. Riesgos / limitaciones
- Defaults de credenciales (`local`/`admin`/`admin`) peligrosos fuera de localhost — README exige reemplazo (confianza: alta, citado en `deploy/global-images/README.md`).
- `hooks.*` del plugin OpenClaw version-gated: config escrita con OpenClaw < 2026.4.24 puede romper el arranque (confianza: alta).
- Imágenes `:latest` locales se reutilizan sin detectar updates remotos; `PULL=1` obligatorio (confianza: alta).
- Sin MongoDB en el repo — cualquier mención a MongoDB sería invención (confianza: alta, ausencia verificada por glob/lectura).

## RESULTADO
- Estado: ✅ COMPLETO
- Archivo: docs/research/tdam/09-deploy-usage.md
- Hallazgo principal: el stack de deploy son 3 imágenes + 3 scripts shell (no docker-compose), con CI mínimo (`pr-ci.yml`) en el repo; el patrón "uso" clave es que un coding agent consume memoria sin cambio de protocolo (solo `base_url`), y `sdk/memory-core` es la referencia estructural para los SDKs de VantaDB.
- Ref clave real: `deploy/global-images/README.md` + `.env.example`, `deploy/panel-knowledge-combined/README.md`, `MemoryCore/src/cli/commands/seed.ts`, `sdk/memory-core/typescript/src/index.ts`