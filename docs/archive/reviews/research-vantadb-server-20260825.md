# INV-vantadb-server-01 — Investigación profunda: `vantadb-server`

| Campo | Valor |
|---|---|
| **Fecha** | 2026-08-25 |
| **Origen** | `/research vantadb-server` (registro `.opencode/references/research-modules.md`) |
| **Objeto** | Crate `vantadb-server` (binario thin) + superficie real en `src/cli_server.rs` |
| **Competidores** | Qdrant · Weaviate (embedded/WCS) · Milvus Lite/Standalone · Marqo |
| **Score global** | **8.0 / 10** (previo 2026-08-23: 7.5 — MOD-12/13/14 cerradas desde entonces) |
| **Skills** | progreso · ponytail(full) · source-driven-development · coordinated-web-search |

---

## 1. Usuarios objetivo y flujo diario

**Self-hosters y equipos que necesitan HTTP API.** Flujo esperado: descargar binario
(GitHub Release) o `cargo install`, levantar `vanta-cli server --http --db ./data`,
conectar clientes REST/SDK, asegurar con `VANTADB_API_KEY` + TLS si se expone.
Fricciones detectadas:

- **Distribución:** el crate es `publish = false` (`vantadb-server/Cargo.toml:6`) —
  la fila del registro dice "crates.io / binario local", drift: la vía real es
  binario de GitHub Release. Un self-hoster que busque en crates.io no lo encuentra.
- **Sin imagen Docker oficial** — los 4 competidores la tienen como primer medio de
  adopción (`docker run qdrant/qdrant`, milvus standalone, marqo, weaviate).
- **Multi-proceso sobre la misma BD:** lock exclusivo mata la segunda instancia
  (incidente 2026-08-25; solución propuesta en MCP-35 modo proxy).

## 2. Estándares del ecosistema (server self-hosted vector DB)

Verificado contra docs oficiales (fuentes al pie):

| Control | Qdrant | Weaviate | Milvus Standalone | Milvus Lite | Marqo | **vantadb-server** |
|---|---|---|---|---|---|---|
| API key authn | ✅ admin + read-only + JWT granular | ✅ API key + OIDC | ✅ user/pass + RBAC | ❌ (dev-only) | Cloud only (scopes Admin/RW/Read) | ✅ 1 Bearer key estática |
| Multi-key / rotación | ✅ `alt_api_key` v1.17 sin downtime | ✅ rotate por user (API v1.30) | ✅ users gestionables | ❌ | ✅ (cloud) | ❌ |
| Per-collection scoping | ✅ JWT RBAC v1.9 | ✅ roles/perms | ✅ roles | ❌ | ✅ scopes cloud | ⚠️ solo por método HTTP (L1/L2/L3) |
| TLS | ✅ + mTLS + rotación certs | ✅ (detrás proxy/K8s) | ✅ one/two-way | ❌ | ✅ | ✅ rustls 1.2+/1.3 (feature `tls`) |
| Rate limiting nativo | ❌ (OSS; quotas solo Cloud) | ❌ | ❌ | ❌ | ❌ | ✅ Governor fail-closed 600rpm + burst diferenciado |
| Refuse-to-start expuesto sin auth | ❌ (default abierto, doc advierte) | ❌ (anonymous configurable) | ❌ | n/a | ❌ | ✅ guard FIND-07 + `--allow-insecure` explícito |
| Audit logging consultable | ✅ JSON rotado + `/audit/logs` v1.18 | ⚠️ | ⚠️ | ❌ | ❌ | ✅ JSONL + `GET /api/v2/audit` |
| Tracing ID por request | ✅ `x-request-id` v1.18 | ⚠️ | ⚠️ | ❌ | ❌ | ❌ |
| Docker oficial | ✅ (+unprivileged) | ✅ | ✅ | n/a (pip) | ✅ | ❌ |
| gRPC | ✅ :6334 | ❌ (GraphQL+REST) | ✅ | ✅ interno | ❌ | ❌ (REST only) |

**Lectura:** VantaDB-server es **superior al estado del arte en defaults seguros**
(rate-limit nativo + refuse-to-start, únicos en la tabla) e inferior en gestión de
identidades (una sola key estática, sin scoping por namespace, sin OIDC/JWT) y en
canales de distribución (sin Docker, sin crates.io).

## 3. Estado actual interno (evidencia file:line)

- **Arquitectura confirmada** (coincide con review 2026-08-23): `vantadb-server/src/server.rs:1-4`
  es re-export puro; todo vive en `src/cli_server.rs` (~4k líneas): router
  `app_with_cors` (:228-330, ~37 operaciones), `auth_middleware` (:633-773),
  TLS/shutdown (:1749-1981), dashboard `mount_dashboard`.
- **Auth:** ct_eq constant-time (:666), rate-limit de fallos por IP (:641-672),
  trusted proxies XFF (:573-599), RBAC roles admin/reader/writer por método
  (:229-232, :718-740), token→role map desde config (:461-471).
- **Docs:** `docs/api/HTTP_API.md` (697 líneas, fresca 2026-08-22) + `openapi.yaml`
  como SSOT con parity check `scripts/check_openapi_parity.mjs` — mejor que ningún
  competidor en disciplina de spec.
- **Tests:** 33 tests e2e/integración (review §7); hueco de text-search HTTP cerrado
  por MOD-12 ("server ensures index state at startup" — HTTP_API.md:77,328).
- **Deudas activas trackeadas:** MOD-15 (nits) · REVIEW-10 (god-file) · FIND-24
  (fan-out list O(n), 408 >10k records) · AUD-043 (clippy `unused variable: ns`
  en `options_for`, cli_server.rs:1302, rompe gates) · MCP-35..41 (canal MCP) ·
  DEC-02 (billing/quota diferido) · PRO-05 (admin console enterprise).
- **Performance del servidor HTTP:** sin números propios publicados (Regla 11:
  no hay claim medible; el baseline canónico `canonical_p99` mide el engine, no
  el overhead HTTP).

## 4. Framework de evaluación (score 0-10)

| Dimensión | Score | Justificación |
|---|---|---|
| DX de onboarding | 8 | Quickstart verbatim verificado + OpenAPI 3.1 SSOT; falta Docker/compose |
| Completitud funcional | 9 | ~37 ops: CRUD+lotes+versiones, híbrido, grafos, IQL, mantenimiento, snapshots, threads, skills |
| Performance/overhead | 6.5 | Sin medición del layer HTTP; FIND-24 O(n) en list fan-out |
| Robustez | 8.5 | Circuit breaker, body limit, graceful shutdown, TimeoutLayer (MOD-13), pool |
| Seguridad | 8 | ct_eq, RBAC, trusted proxies, guard FIND-07; faltan multi-key/scoping/OIDC |
| Docs & ejemplos | 9 | Narrativa + spec machine-readable + parity CI — top de la categoría |
| Observabilidad | 8 | Prometheus opcional + metrics v2 JSON + audit consultable; sin tracing-id |
| Testabilidad | 8 | 33 tests socket-real; rate-limit e2e endurecido (MOD-14) |
| Paridad inter-módulo | 8 | REST completo post-ADR-026; gaps solo en canal MCP (trackeados) |
| Diferenciación vs Qdrant | 7 | Único con grafo+IQL+híbrido BM25+cognitiva+MCP embebido y defaults seguros; sin cluster/gRPC/Docker |
| **Global** | **8.0** | |

## 5. Gap analysis priorizado

**Falta (P0/P1):** gestión de identidades multi-usuario (multi-key, scoping,
OIDC) — bloquea adopción "equipo"; imagen Docker; fix AUD-043 (rompe gates).
**Mejorable:** split del god-file; rotación audit log; tracing-id; drift registro/docs.
**Optimizable:** fan-out list (FIND-24); medición de overhead HTTP.

### Quick wins (<1 día)
AUD-043 · MOD-15 · tracing-id en audit · drift registro/ecosistema.

### Apuestas estratégicas (>1 semana)
OIDC/JWT · RBAC por namespace · Docker oficial + guía hardening como posicionamiento.

## 6. Recomendaciones → Fase D

Los hallazgos del Apéndice se deciden uno a uno vía HITL (`question`), con
materialización en Backlog (prefijo SRV-*), wontfix, plan de quick-wins o ADR.

---

## Apéndice — Inventario de hallazgos H-NN

| ID | Hallazgo | Categoría | Severidad | Esfuerzo | Evidencia |
|---|---|---|---|---|---|
| H-01 | Clippy `unused variable: ns` rompe `just verify`/pre-push/CI Fast Gate | APLICAR | 🔴 Alta | 🟢 | `src/cli_server.rs:1302` · Backlog AUD-043 |
| H-02 | Nits MOD-15 agrupados: middleware.rs re-export redundante, feature `sysinfo=[]` vacía, comentario ensure faltante en main.rs, sin constructor `ServerState` para tests | APLICAR | 🟢 Baja | 🟢 | `vantadb-server/src/middleware.rs:1` · `Cargo.toml:33` · Backlog MOD-15 |
| H-03 | God-file `cli_server.rs` ~4k líneas concentra routing+RBAC+TLS+OTEL+tests — blast radius total en un archivo | MEJORAR | 🟠 Media | 🟠 | `src/cli_server.rs` (todo) · Backlog REVIEW-10 |
| H-04 | Una sola API key estática sin rotación; competidores tienen multi-key + rotación sin downtime (qdrant `alt_api_key` v1.17, weaviate rotate API) | AGREGAR | 🟡 Media | 🟡 | `ServerState.api_key` `src/cli_server.rs:455-471` · [qdrant security](https://qdrant.tech/documentation/security/) |
| H-05 | RBAC solo por método HTTP; sin scoping por namespace (qdrant per-collection r/w v1.9; weaviate roles/perms) | AGREGAR | 🟡 Media | 🟡 | `src/cli_server.rs:718-740` · [qdrant](https://qdrant.tech/documentation/security/) · [weaviate](https://docs.weaviate.io/deploy/configuration/authentication) |
| H-06 | Sin OIDC/JWT: solo bearer estático. Requisito de facto para "equipos" enterprise (weaviate OIDC nativo; qdrant JWT RBAC HS256 offline) | AGREGAR (estratégica) | 🟡 Media | 🔴 | `auth_middleware` `src/cli_server.rs:633-773` · fuentes §2 |
| H-07 | Sin imagen Docker oficial ni compose — canal #1 de adopción de los 4 competidores | AGREGAR | 🟡 Media | 🟡 | `vantadb-server/Cargo.toml` (publish=false, sin Dockerfile propio) |
| H-08 | Audit log JSONL sin rotación ni retención (crece indefinido); qdrant v1.17 rota daily + `max_log_files` | MEJORAR | 🟡 Media | 🟢 | `src/audit.rs:89+` · `GET /api/v2/audit` HTTP_API.md:147-150 |
| H-09 | Sin tracing-id por request (`x-request-id`) para correlación cliente↔audit; qdrant v1.18 lo tiene | AGREGAR | 🟢 Baja | 🟢 | middlewares `src/cli_server.rs:860-908` · [qdrant security §tracing-ids] |
| H-10 | Fan-out list all-namespaces O(n): 408 >10k records, re-lista todo por página (= FIND-24, ya trackeado — decidir prioridad/pago) | OPTIMIZAR | 🔴 Alta | 🟠 | `merge_all_namespaces_pages` `src/cli_server.rs:1285-1312` · Backlog FIND-24 |
| H-11 | Posicionamiento "local-first seguro por default" no documentado: somos únicos con rate-limit nativo + refuse-to-start (qdrant/marqo default abiertos); oportunidad de diferenciación + guía hardening comparada | ESTRATEGIA | 🟡 Media | 🟢 (docs) | guard FIND-07 HTTP_API.md:591-618 · [qdrant: "all self-deployed instances are not secure"](https://qdrant.tech/documentation/security/) |
| H-12 | gRPC endpoint secundario (estándar qdrant/milvus-lite) | DESCARTAR sugerido | 🟢 | 🔴 | YAGNI local-first: REST+MCP cubren; costo alto sin demanda medida |
| H-13 | Streaming/SSE para export/traversals grandes (nota informativa del review previo §4) | DESCARTAR sugerido | 🟢 | 🟡 | review modulos/vantadb-server.md §4 |
| H-14 | Drift de distribución: registro del módulo dice "crates.io" pero `publish = false`; docs de instalación no señalan la vía real (GitHub Release binaries) | MEJORAR | 🟢 Baja | 🟢 | `vantadb-server/Cargo.toml:6` · `.opencode/references/research-modules.md:18` |

### Fuentes web (verificación Regla source-driven-development)

- Qdrant Security (oficial): https://qdrant.tech/documentation/security/ — API keys admin/read-only/granular JWT, `alt_api_key` rotación v1.17, audit v1.17-1.18 con tracing IDs, TLS, network bind, hardening. *"By default, all self-deployed Qdrant instances are not secure."*
- Weaviate Authentication (oficial): https://docs.weaviate.io/deploy/configuration/authentication — API key + OIDC + anonymous, RBAC, user management API v1.30.
- Milvus (oficial): https://milvus.io/docs/authenticate.md (authN/RBAC) · https://milvus.io/docs/tls.md (TLS one/two-way gRPC+REST) · https://github.com/milvus-io/milvus-lite (embedded dev-only, sin auth/TLS).
- Marqo (oficial): https://docs.marqo.ai/reference/cloud/find-your-api-key/ (scopes Admin/RW/Read en Cloud; local vía Docker sin auth por defecto).

*Nota de método:* agent-search keyless dio bot_challenge; cascada Argus(yahoo)+extracción directa OK; MetaSearchMCP wedgeado (todos los engines timeout — síntoma conocido, restart de opencode recomendado fuera de esta sesión).
