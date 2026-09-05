# MKT-18i — Compose multi-servicio Ollama + VantaDB (+ AnythingLLM verificado)

**Plan:** `docs/plans/2026-09-03-quality-gtm-wave.md` · Wave1 · Ruta: vanta-worker · **Estado:** ⏳ IN PROGRESS (claim server bloqueado por tareas ajenas ERR-TS-01/GOV-TK9 — se ejecuta por asignación explícita, handoff al orquestador)

## Spec (decisiones con evidencia)

| # | Decisión | Elección | Evidencia (verificada 2026-09-03) |
|---|----------|----------|-----------------------------------|
| 1 | ¿AnythingLLM soporta VantaDB como vector DB? | **NO → stop condition del plan disparada; demo = VantaDB+Ollama, fila re-escalada** | `server/.env.example` master (raw.githubusercontent.com/Mintplex-Labs/anything-llm): `VECTOR_DB` ∈ lancedb, chroma, chromacloud, pinecone, astra, pgvector, weaviate, qdrant, milvus, zilliz. Sin VantaDB. Docs oficiales confirman (`docs.anythingllm.com/features/vector-databases`). No se inventa glue. |
| 2 | Tag Ollama | `ollama/ollama:0.33.2` (pin) | Docker Hub tags API: `latest` digest `sha256:020e...` == digest `0.33.2` (pushed 2026-08-28); 0.33.3 solo RC. multi-arch amd64+arm64. |
| 3 | Tag VantaDB | `image: vantadb/server:0.5.0` sobre `build: .` (mantiene camino de SRV-07: si publican imagen oficial, cambiar `build` por `image:` published) | `Cargo.toml:695` version 0.5.0 == `Dockerfile:6` `APP_VERSION=0.5.0`. `build: .` actual no lo toca SRV-07 (él toca Dockerfile + workflow, paths actuales respetados). |
| 4 | Puertos | vantadb `8080` (actual, sin cambio), ollama `11434` (default oficial) | compose actual leído (líneas 4-5); `docs.ollama.com/docker` CPU-only usa `-p 11434:11434` sin env → el server bindea 0.0.0.0 en el contenedor. Sin colisión entre servicios; 3001 (AnythingLLM) no se expone al excluirse. |
| 5 | GPU | Default CPU, nota AMD `rocm` tag / NVIDIA `--gpus` en comentario | `docs.ollama.com/docker`: CPU-only sin flags; AMD usa `ollama/ollama:rocm`; NVIDIA requiere Container Toolkit. |
| 6 | RAM mínima | ~4 GB declarada en cabecera (llama3.2:3b Q4 ≈2GB + nomic-embed ≈0.3GB + server <0.2GB + overhead) | modelos del tutorial 02 existente (`ollama pull llama3.2:3b`, `nomic-embed-text`). |
| 7 | Healthchecks | Solo vantadb (ya existe, no romper); ollama sin healthcheck (imagen oficial sin curl verificado → no inventar) | `docker-compose.yml:13-18` actual; docs Ollama no muestran healthcheck oficial. |
| 8 | Enlace docs | 1 línea en `DEPLOYMENT_GUIDE.md` §3 Docker Compose (único lugar donde ya se documenta docker; README **no** tiene bloque docker — verificado `rg -i docker README.md` = 0 hits) + 1 línea en tutorial 02 (stack Ollama+VantaDB exacto) | grep ejecutado 2026-09-03. |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `docker-compose.yml` (21L), `docker-compose.dev.yml` (35L, standalone con sus propios volumes — agregar servicios al root NO lo afecta), `Dockerfile` (100L, read-only para mí), `vantadb-server/docker-compose.yml` (95L, compose server separado — no reutilizable para demo, sin ollama), `DEPLOYMENT_GUIDE.md:138-212`.
- **Referencias salientes del compose:** Dockerfile (`build: .`) → SRV-07 lo modifica en paralelo; trabajo sobre paths actuales, nota inline de swap a imagen publicada.
- **Referencias entrantes:** grep repo `docker-compose`: SOLO docs/historial + Backlog + SRV-07 plan. Ningún workflow CI ni justfile lo invoca. `docs/web/guides/build-deploy.md:143` lo describe ("despliega solo el server" — queda levemente desactualizado; NOTICED BUT NOT TOUCHING, es archivo web-docs fuera de blast radius).
- **Veredicto:** edición aditiva sobre root compose segura; `config -q` valida integridad; dev compose intacto.

## Pasos

- [x] 1. Discovery: premise AnythingLLM verificada → STOP CONDITION con evidencia → re-escala a VantaDB+Ollama (Spec #1)
- [x] 2. Tags/defaults verificados contra fuentes oficiales (Spec #2,4,5)
- [x] 3. `docker-compose.yml` multi-servicio escrito (RED 0 → GREEN rg=14, 2 servicios, tags `vantadb/server:0.5.0` + `ollama/ollama:0.33.2`)
- [x] 4. Enlaces: DEPLOYMENT_GUIDE §3 (1 línea) + tutorial 02 (1 línea)
- [x] 5. Verify: parse PyYAML OK + assert tags (sin docker CLI en host → nota, run-time diferido); dev compose intacto; commit `abb6594c` (44+ exactas tras limpiar absorción de WIP SRV-07 con reset+restore)
- [x] 6. Cierre: plan Task 6 ✅, avance/operaciones.md, Backlog re-escalado 🔴 upstream (re-aplicado tras ser pisado por `1ad28523` SRV-07), lesson escrita
- [ ] 7. (delegado orquestador) reconcilear claims server ERR-TS-01/GOV-TK9 (stale, bloquean `campaign_update_task_state`); run-time `up -d` en host con daemon

## Contrato (del plan, adaptado a re-escala)

- `rg -ci "ollama|anythingllm|anything-llm" docker-compose.yml` ≥ 2 → cumple por menciones ollama + nota anythingllm
- YAML válido: sin docker CLI → parse PyYAML equivalente + nota; run-time `up -d` **diferida (sin daemon)**
- Tags explícitos en todas las imágenes (re-escalado: 2 servicios, no 3 — AnythingLLM excluido con evidencia)
- RAM mínima en cabecera; comentario AnythingLLM→VantaDB documenta por qué NO hay env de conexión (no existe)
