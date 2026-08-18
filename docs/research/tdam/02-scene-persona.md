# TDAM — 02: Scene extraction + Persona — Investigación profunda

> Fecha: 2026-08-18 · Agente: vanta-research · Scope: core/scene, core/persona, core/profile
> Fuente: clone local completo `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` (rama `feat/server_team`)

## 1. Resumen ejecutivo

TDAM implementa L2 (escenas) y L3 (persona) como **agentes LLM con herramientas de archivo**, no como algoritmos deterministas. El `SceneExtractor` le da al LLM acceso sandboxed a `scene_blocks/` (read/write/edit) para consolidar memorias L1 en documentos Markdown narrativos con META. El `PersonaGenerator` resume las escenas **cambiadas** en `persona.md` con un modelo de 4 capas. Un `MemoryPipelineManager` orquesta L0→L3 con timers per-sesión, checkpoint con estado dividido (runner vs pipeline) y triggers de persona en 5 prioridades. Todo L2/L3 es **team+agent scoped**, se persiste como archivos Markdown + `scene_index.json`, y se sincroniza con el store vía ProfileRecord (MD5-checked).

## 2. Arquitectura y flujo

```
L0 recordConversation (l0-recorder.ts) → conversations/YYYY-MM-DD.jsonl (por sesión, cursor atómico)
   │ notifyConversation (pipeline-manager.ts)
   ▼
L1 extractL1Memories (l1-extractor.ts) → LLM JSON {scene_name, memories[]} → dedup → store L1
   │ everyNConversations=5 | idle 30s | warmup 1→2→4→5
   ▼
L2 SceneExtractor.extract() (scene-extractor.ts:141) — timer downward-only (90s delay / 15min min / 60min max)
   │ queryMemoryRecords(updatedAfter=cursor) → agrupa por aislamiento → LLM agente
   ▼
   scene_blocks/{nombre}.md (META + markdown) → syncSceneIndex → .metadata/scene_index.json
   │                                                     → persona.md (append Scene Navigation)
   ▼
L3 PersonaTrigger.shouldGenerate() (5 prioridades) → PersonaGenerator.generateLocalPersona()
   │                                   → LLM agente escribe persona.md (≤2000 chars)
   ▼
   persona.md (body + Scene Navigation) → ProfileRecord sync (store)
```

## 3. Lógica y algoritmos (refs archivo:línea)

- **Segmentación de sesiones**: NO hay algoritmo de segmentación en L2 — la segmentación semántica ocurre en **L1**: el LLM devuelve `SceneSegment[] {scene_name, message_ids, memories[]}` (core/record/l1-extractor.ts:52-62, 519-590). Los prompts L1 tienen dos modos: `chat` (3 tipos, prioridad 0-100, -1 para instrucción global estricta) vs `code`/work (4 tipos work con owner/deadline/status) — `core/prompts/l1-extraction.ts`. L2 agrupa memorias por batch incremental (`updatedAfter` cursor, utils/pipeline-factory.ts:713-743) y por scope de aislamiento (utils/pipeline-factory.ts:770-776).
- **Estrategia de escena**: UPDATE preferido > MERGE > CREATE (máx. 1 nuevo por batch; CREATE exige leer 2 escenas similares antes) — prompt scene-extraction.ts:141-154.
- **Heat**: CREATE=1, UPDATE=old+1, MERGE=sum+1 (scene-extraction.ts:183-185).
- **Límite de escenas**: `maxScenes` default 15 (config.ts:556); warning escalonado: ≥max rojo (MERGE obligatorio), =max-1 naranja (solo UPDATE), ≥max-3 amarillo (scene-extractor.ts:191-200).
- **Soft-delete**: el LLM no tiene exec; borra escribiendo `[DELETED]`; el extractor limpia con unlink + archivos META-only (core/scene/scene-extractor.ts:298-359).
- **Backup/restore defensivo**: antes de invocar al LLM se respalda el estado de `scene_blocks/`; ante fallo del LLM se restaura el backup (extract() → backup → index → prompt → LLM → restore de backup ante error).
- **Normalización de filenames** defensiva post-LLM: espacios→guion, quita puntuación, `.md` minúscula, fallback `scene.md` (filename-normalizer.ts:43-68).
- **Persona incremental**: filtra escenas con `updated > cp.last_persona_time`, precarga su contenido completo en el prompt (core/persona/persona-generator.ts:111-141); skip si no hay cambios y ya existe persona (143-146); post-escritura: strip nav previa, `escapeXmlTags` (anti inyección), append nav fresca, `markPersonaGenerated`.
- **Triggers persona** (persona-trigger.ts:35-96): P1 request explícito (señal LLM `[PERSONA_UPDATE_REQUEST]`, scene-extractor.ts:83-97) → P2 cold start → P2.5 recovery (body vacío) → P3 primera escena → P4 `memories_since_last_persona >= triggerEveryN` (default 50, config.ts:53).

## 4. Funcionalidades / Endpoints

- Tools LLM: `read`/`write`/`edit` (L2 sandbox scene_blocks/), `tdai_read_cos` (lectura genérica por clave relativa con guard anti path-traversal, read-cos.ts:81-91).
- API v2: `/scenario/*` y `/persona/*` (interfaces semánticas que añaden prefijo StoragePaths, read-cos.ts:10-14).
- Profile sync: `listLocalProfiles`, `pullProfilesToLocal` (MD5-check + rename atómico), `syncLocalProfilesToStore` (diff por contentMd5), `ensureL2L3Local` (profile-sync.ts:133-493).
- Métricas: `l2_extraction`, `l3_persona_generation`, latencias L2/L3 (core/scene/scene-extractor.ts:477, core/persona/persona-generator.ts:263), `pipeline_l1_trigger` (utils/pipeline-manager.ts).

## 5. Código clave

- **Formato MMD (Scene Block)**: `-----META-START-----` created/updated/summary/heat `-----META-END-----` + markdown (scene-format.ts:18-48).
- **Índice**: `SceneIndexEntry {filename, summary, heat, created, updated}`; `syncSceneIndex` reconstruye desde los .md (scene-index.ts:9-15, 102-137). **El LLM nunca ve este archivo** (`.metadata/scene_index.json`).
- **Modelos**: `ProfileRecord {id: "profile:v1:"+sha256(scope\0type\0filename), type:"l2"|"l3", content, contentMd5, teamId, agentId, version...}` (core/store/types.ts:266-282). Checkpoint con `runner_states`/`pipeline_states` separados (utils/checkpoint.ts:97-100,136-137).
- **Aislamiento**: L2/L3 son team+agent: `team:${teamId}|agent:${agentId}` — ignora userId/sessionId deliberadamente (profile-sync.ts:20-28).

## 6. UX/UI (navegación MMD)

`generateSceneNavigation` (scene-navigation.ts:43-67) genera una sección al final de persona.md: por escena, `### Path: <abs o scenes/>`, `**热度**: N + emojis 🔥` (≥50,100,200,500,1000), `Summary`, ordenada por heat desc; footer instruye usar `read` (local) o `tdai_read_cos` (COS). Esto es *progressive disclosure*: el agente solo lee el archivo completo cuando lo necesita.

## 7. Integración en VantaDB

- **core puro (Rust/WASM)**: NO copiar el LLM-en-hot-path. El core puede guardar escenas/personas como **nodos** en el grafo (graphrag ya existe parcialmente): nodo escena con `summary/heat/created/updated` (los 4 campos META son un contrato estable), y edges L1→escena (la `scene_name` ya existe en L1RecordRow).
- **vanta-memory**: aquí viven SceneExtractor/PersonaGenerator (LLM-driven, servicios): prompts de escena/persona, triggers por checkpoint, sync de perfiles. El patrón "LLM agente con tools + limpieza defensiva + índice reconstruido por ingeniería" es el activo a replicar.
- **MCP/server**: tools de navegación de escenas (`tdai_read_cos` → tool MCP `scene_read`), listado por heat, query por scene_name.
- **NO copiar**: el split de 4 servicios, el sandboxing por workspaceDir de OpenClaw, la sincronización COS, el offload_server/mmd-handler (subsistema distinto).

## 8. Riesgos / limitaciones / qué NO copiar

- **LLM agente = caro y lento**: L2 timeout 300s, L3 180s; una extracción vacía es un fallo conocido (detectado por `emptyExtraction`, scene-extractor.ts:509-514) — el intento anterior devolvió vacío por esto.
- **Cap de 15 escenas** fuerza MERGEs → pérdida de matices.
- **Heat lo mantiene el LLM**, no un contador real — la semántica "memorias que golpean la escena" del footer es aspiracional.
- **Límite 1500 chars/escena y 2000 chars/persona** → truncamiento de narrativa rica.
- **Prompts en chino** con secciones-skeleton (用户核心特征…) — requieren traducción/adaptación.
- **Seguridad**: la contención depende de workspaceDir + ausencia de exec; en VantaDB, replicar con sandbox propio, no confiar en la buena fe del prompt (el filename-normalizer existe porque el LLM desobedece).
- No hay dedup global de escenas: MERGE es manual por el LLM; perfiles sincronizados por MD5 con reglas de no-borrado ante mismatch (profile-sync.ts:301-316, 349-355).

## RESULTADO
- Estado: ✅ COMPLETO
- Archivo: docs/research/tdam/02-scene-persona.md
- Hallazgo principal: L2/L3 son agentes LLM con herramientas de archivo (no algoritmos), con índice reconstruido por ingeniería y navegación por heat en persona.md; el core de VantaDB solo necesita el contrato META {created, updated, summary, heat} + scene_name en L1.
- Ref clave: `scene-extractor.ts:141` (extract()) + `scene-format.ts:18-48` (formato MMD)