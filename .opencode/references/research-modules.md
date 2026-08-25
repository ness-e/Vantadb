# Research Modules Registry — fuente única de configuración por módulo

> **Fuente canónica** para el comando `/research <módulo>`. El comando carga esta
> tabla para sustituir los `{{placeholders}}` de `prompts/research-module.md`.
>
> **Cómo se agregan módulos:** SOLO vía el flujo del comando (`/research <nuevo>`):
> si el módulo no está en esta tabla pero existe como directorio en el repo, el
> comando pregunta al usuario (tool `question`) los campos faltantes y agrega la
> fila acá. Prohibido editar filas existentes sin pasar por una investigación.
>
> **Campos:** Módulo · Tipo · Ecosistema (dónde vive el paquete) · Usuarios objetivo ·
> Competidores mínimos · Nota específica (deudas/checkpoints propios del módulo).

| Módulo | Tipo | Ecosistema | Usuarios objetivo | Competidores mínimos | Nota específica |
|--------|------|-----------|-------------------|---------------------|-----------------|
| `vantadb-node` | Binding nativo napi-rs | npm | Devs Node.js/backend que quieren engine embebido sin WASM | `@lancedb/lancedb`, `sqlite-vec` (node), `hnswlib-node`, `Orama`, `usearch` + A/B interno vs `vantadb-ts` WASM | Evaluar prebuilds multiplataforma (CI release), tamaño del `.node`, parity de API con ts |
| `vantadb-python` | SDK PyO3 | PyPI | Devs Python/AI, frameworks de agentes | `chromadb`, `lancedb`, `qdrant-client (local)`, `mem0ai`, `sqlite-vec`, `txtai` | Verificar stubs .pyi anti-drift (MOD-18), jerarquía VantaError (MOD-20), wheels multiplataforma |
| `vantadb-server` | Server wrapper + MCP | crates.io / binario local | Self-hosters, equipos que necesitan HTTP API | `qdrant`, `weaviate embedded`, `milvus lite`, `marqo` | Posicionamiento honesto: es local/lightweight, no compite con clusters; evaluar auth/TLS/rate-limit existentes vs esperados |
| `vantadb-ts` | SDK WASM multi-runtime | npm | Devs JS (Node/Bun/Deno/browser) | `Orama`, `vectra`, `wa-sqlite`+vec, `DuckDB-WASM` | require(esm)/engines (FIND-10), códigos de error tipados, bundle size del WASM |
| `vantadb-wasm` | Bindings WASM standalone | npm (pkg) | Frontend browser-only (sin servidor) | `Orama` (browser), `sql.js-httpvfs`, `DuckDB-WASM`, `vectra` | OPFS persistence (deuda P2-1/P2-8), modo `--mode wasm` (WASM-03), límites de memoria browser |
| `providers` | Adapters de inference | repo (feature `remote-inference`) | Devs que conectan embeddings/chat al engine | `fastembed`, LiteLLM (uso directo), SDKs oficiales openai/ollama, `sentence-transformers` | Paridad de features entre litellm/ollama/openai, remote-inference feature flag, fallback/offline |
| `integrations` | Adapters de frameworks (langchain, llamaindex, dspy, haystack, crewai, letta, mem0, ollama, openai) | repo (por framework) | Devs de cada framework que quieren memoria persistente | Memoria/integración nativa de cada framework (ej: langchain memory modules, llamaindex memory, mem0 como lib) + equivalentes zep/cognee | Sub-investigación por framework o agrupadas; evaluar cobertura idiomática de la API del framework, versión soportada, tests por integración |

## Módulo competidor principal por defecto (para la dimensión "diferenciación")

| Módulo | {{COMPETIDOR_PRINCIPAL}} |
|--------|--------------------------|
| `vantadb-node` | `@lancedb/lancedb` |
| `vantadb-python` | `chromadb` |
| `vantadb-server` | `qdrant` |
| `vantadb-ts` | `Orama` |
| `vantadb-wasm` | `Orama` (browser) |
| `providers` | LiteLLM |
| `integrations` | mem0 (como lib integrable) |

## Docs/api asociadas por módulo (para el check de docs del informe)

| Módulo | Doc canónica |
|--------|--------------|
| `vantadb-node` | `docs/api/NODE_SDK.md` *(verificar existencia; si no existe, es un hallazgo)* |
| `vantadb-python` | `docs/api/PYTHON_SDK.md` + `docs/api/BINDINGS_NAMESPACES.md` |
| `vantadb-server` | `docs/api/HTTP_API.md` |
| `vantadb-ts` | `vantadb-ts/README.md` + `docs/api/BINDINGS_NAMESPACES.md` |
| `vantadb-wasm` | `vantadb-ts/README.md` (compartido) + `vantadb-wasm/demo/README.md` |
| `providers` | `providers/<nombre>/README.md` por provider |
| `integrations` | `integrations/<framework>/README.md` por framework |
