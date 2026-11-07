---
title: "Supermemory: integración SaaS y contratos extraíbles para VantaDB"
kind: research
status: proposed
description: "Clientes, MCP, borrado, caché, UI e incidencias públicas; lectura separada de MemoryBench y propuestas Rust"
tags: [research, memory, engineering]
---

# Supermemory: integración SaaS y contratos extraíbles para VantaDB

## Índice

- [0. Identidad, arquitectura y evidencia](#0-identidad-arquitectura-y-evidencia)
- [1. Recomendaciones priorizadas](#1-recomendaciones-priorizadas)
- [2. Fichas de extracción Rust](#2-fichas-de-extracción-rust)
- [3. Incidencias y seguridad](#3-incidencias-y-seguridad)
- [4. Prerrequisitos VantaDB](#4-prerrequisitos-vantadb)
- [5. Rechazos y costes ocultos](#5-rechazos-y-costes-ocultos)
- [6. Advertencias metodológicas](#6-advertencias-metodológicas)
- [7. Benchmarks y adaptación](#7-benchmarks-y-adaptación)
- [8. Matriz de capacidades](#8-matriz-de-capacidades)
- [9. Negativos y falsos positivos de VantaDB](#9-negativos-y-falsos-positivos-de-vantadb)
- [10. Producto UI y Studio](#10-producto-ui-y-studio)
- [11. Cobertura, fuentes y decisiones pendientes](#11-cobertura-fuentes-y-decisiones-pendientes)

## 0. Identidad, arquitectura y evidencia

### 0.1 Corte y límites

| Objeto | Referencia usada |
|---|---|
| Proyecto | `supermemoryai/supermemory`, clon `C:\Users\Eros\supermemory`, rama `main` |
| Fuente congelada | `16ed414e0e0c61c946b7687ec813b2064fe75dd4` |
| Captura original | 31 559 estrellas; [candidatos](github-candidates.json) y [manifest](clone-manifest.json), **no recalculados ni alterados** |
| Investigación | Lectura estática y consultas públicas observadas el 2026-11-07; ninguna ejecución de producto, tests, builds o benchmarks |
| Benchmark externo | `supermemoryai/memorybench` en `063b86bfb347336def3a084153f40bfea1e2dfd6`; fuente remota, no cambio del pin del clon |
| VantaDB | Snapshot contractual `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no se persigue HEAD concurrente |
| Studio | `develop-desktop`, HEAD observado `0ad38c542b39a589035b0e17fdedb4c09dca24b9` **con cambios ajenos**; evidencia de worktree, no de release |
| Licencias | Raíz MIT, copyright 2025 Supermemory [S0]; paquete tools declara MIT [S2]. SDKs externos y contrato comercial del servicio no auditados |

**Dictamen:** interesa la frontera aplicación/memoria remota: namespace explícito, perfiles separados de recuperación, caché de continuación, estados de procesamiento, borrado selectivo y provenance visible. **No es una implementación pública acreditada del motor SaaS**. El clon no permite certificar extracción, almacenamiento vectorial, transacciones, replicación, borrado físico o aislamiento de tenants.

Leyenda: **[H]** hecho de fuente leída; **[D]** documentación; **[R]** reporte upstream, no reproducción propia; **[I]** inferencia; **[P]** propuesta; **[?]** laguna. Fuentes S: pin del clon; B: MemoryBench externo; V: Vanta indicado. Se aplicaron [METHOD](METHOD.md), [BENCHMARK-NOTES](BENCHMARK-NOTES.md) y [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md). El antecedente [OpenViking](../../Backlog-OpenViking-VantaDB.md) aporta diseño, no prueba del estado actual de Vanta.

### 0.2 Mapa, stack y dependencias

| Superficie | Implementación realmente leída | Límite |
|---|---|---|
| Raíz | Bun/Turbo, TypeScript ESM; Biome y scripts con subida de sourcemaps a Sentry [S1] | Leer scripts no es ejecutarlos |
| `packages/tools` | AI SDK/OpenAI/Mastra/Voltagent; `supermemory ^5.0.0`, `lru-cache ^11.2.6`, Zod, tsdown, Vitest [S2] | SDK como dependencia, no motor implementado aquí |
| `tools/src/shared` | `memory-client.ts`: fan-out/render; `cache.ts`: LRU [S3][S4] | El texto final no equivale a un registro completo de evidencia |
| `tools/src/vercel` | Turnos, cancelación, inyección y captura post-stream [S5][S14] | No acredita entrega durable exactamente una vez |
| `livekit-sdk-python` | Python ≥3.10, LiveKit Agents, Pydantic y `supermemory>=3.50.0` sin cota superior [S9] | API legacy convive con clientes v5 |
| `apps/mcp` | Herramientas de espacios/documentos/search/profile/forget, Zod, sesión/JWKS [S10] | No auditadas todas las rutas de autorización |
| `apps/docs`, `packages/docs-test` | Documentación v5 y runner de ejemplos TS/Python [S6][S7] | Hay ejemplos con API anterior |
| `apps/web` | `app/page.tsx:1-60`: traslado a consola [S11] | No es la consola SaaS completa |
| `packages/memory-graph` | React; aristas, estados y agrupación por conectividad [S12] | Layout no demuestra ranking ni inferencia del backend |

`CLAUDE.md` describe Cloudflare, Drizzle e `IngestContentWorkflow`; **no se usa como prueba de implementación actual**. `AGENTS.md:29-80` enseña v5 y la frontera de servicio [S8]. Cloud-first clasifica la superficie visible, no todos los despliegues comerciales posibles.

### 0.3 Arquitectura, modelo y API

```text
Aplicación / tool loop / LiveKit / MCP
  credencial + namespace + thread + modo profile/query/full
       → query → caché por turno
       → profile(namespace) || search(namespace, memories)
       → dedup de fuentes visibles → plantilla → parámetros del modelo
                         │ HTTPS + Bearer
                         ▼
SaaS: document → procesamiento asíncrono → chunks / memories / profile
                         └─ respuestas y relaciones → UI/grafo
```

**[D] API v5:** `POST /ns/{namespace}/document`, `/search`, `/profile`, auth Bearer; cada operación nombra namespace. Search admite `chunks/memories/hybrid`, este último default documentado. Readiness distingue `system.status` de `system.dreamingStatus` [S8]. Ninguno acredita fsync/replicación.

**[H] Datos cliente:** `MemoryItem` acepta `memory`, `chunk`, metadata; perfil con arrays `static/dynamic/searchResults` y compatibilidad con strings. `normalizeMemoryFact` elimina prefijos recientes/fecha y normaliza espacios/minúsculas: **no dedup semántico** [S13:171-205]. El grafo muestra documentos, memorias, `updates/extends/derives`, olvido/expiración y fechas [S12]; no determina el esquema físico.

Separar: modo middleware `profile/query/full`; modo API `chunks/memories/hybrid`; estado de materialización. El perfil v5 no recibe query: `getProfileWithSearch` consulta search por separado [S13:106-126].

**Despliegue:** integración con salida de red/credenciales; MCP es fachada, no réplica del motor. `AGENTS.md` documenta servidor local en puerto 6767 y compatibilidad v5 desde `supermemory-server` 0.0.9 [S8]; eso acredita una opción documentada de self-hosting, **no su implementación ni equivalencia de funciones/licencia con SaaS**. Offline, backup/restauración y costes contractuales no certificados. No se ejecutaron setup, npx, Bun, Docker o servicios.

### 0.4 Flujos correctos y fallos

**Recall [H].** `transformParamsWithMemory` obtiene último usuario; sin usuario y fuera de profile limpia contexto previo. Calcula key/detecta turno; continuación con string truthy cacheado reutiliza texto. En miss configura AbortController solo con timeout positivo; llama a `buildMemoriesText`, limpia timer en `finally`, cachea e inyecta. Fan-out usa `Promise.all`, profile opcional y search `memories`, threshold **0.6 observado**, retries cero [S3][S4][S5]. No transferir ese valor a scores Vanta.

**Fallo [H].** Si falla un brazo, `Promise.all` rechaza el conjunto: no conserva éxito parcial. El `finally` limpia timer, no transforma error en vacío. El caller `wrapVercelLanguageModel` sí define `skipMemoryOnError=true` por defecto: retira la inyección y llama al modelo original; false propaga el error [S14:149-180,216-244]. Es fail-open **de disponibilidad de memoria en este wrapper**, no autorización fail-open ni garantía de todas las integraciones. LiveKit captura excepciones para preservar llamada (§3).

**Caché [H].** Max 100; key namespace/thread escapados + modo + mensaje normalizado. No revisión ni TTL en `cache.ts`; vacío no obtiene el mismo hit truthy. Riesgo de stale depende de vida del middleware, no demuestra fuga entre cuentas [S4].

**Escritura [D/H].** Aceptar documento/ID precede procesamiento; provider de MemoryBench preserva `sessionId`, fecha y metadata y espera indexación [B1]. HTTP aceptado ≠ chunks disponibles ≠ memoria extraída. No se certifica captura post-stream exactamente una vez.

**Borrado [H].** `deleteDocument`: GET en namespace, rechaza estado no terminal distinto de `done/failed`, resuelve ID y exige count uno. `forgetMemory` textual: preview dryRun, igualdad normalizada, forget por IDs; similitud no autoriza borrar [S13:64-168].

**Carrera [I].** GET→DELETE y preview→forget no son transacciones visibles. Actualización concurrente requiere revisión/comprobación servidor no certificada. Olvidar memorias ≠ borrar documentos ≠ eliminar backups/cachés/aristas.

**Post-respuesta [H].** `doGenerate` y `doStream` llaman a `saveMemoryAfterResponse` sin await; stream acumula `text-delta` y dispara guardado en `flush`, solo si `addMemory=always` y hay usuario persistible [S14:182-204,246-284]. La función convierte mensajes, espera internamente `addConversation` y captura/loguea error sin devolver recibo al host [S5:151-188]. Ni cierre normal del stream ni respuesta del modelo acreditan que esa promesa terminó. Cancelar el consumidor puede impedir `flush`; el wrapper no muestra aquí spool ni vínculo durable con la entrega.

**Auth MCP [H].** `fetchSession` valida esquema y limita espera; claves opacas se cachean por token en el isolate. OAuth usa JWKS, issuer/audience y exige `sub/organization_id`; errores transitorios de red/JWKS no se clasifican como token inválido [S15:30-53,108-207]. La ventana de caché observada es 60 s y máximo 1000 entradas, **no parámetros recomendados**. RBAC calcula permiso efectivo por sesión/scope [S17]; no se auditó su aplicación en cada herramienta. Namespace de entrada validado no es autorización.

## 1. Recomendaciones priorizadas

| Orden | Decisión | Justificación y condición |
|---|---|---|
| Primero | Outcome por fuente antes del render | Error/timeout no son vacío; extender recall, no crear assembler |
| Primero | Revisión/política en caché de continuación | No servir memoria revocada; inventariar antes cachés reales del host |
| Primero | Recibo con aceptado/indexado/derivado listo | Diagnóstico sin promesas de durability no acreditadas |
| Después | Dedup solo de fuentes seleccionadas | No perder query por perfil que no se muestra |
| Después | Preview de olvido autorizado y revisionado | Evitar destrucción por similitud; extender erasure |
| Después | Confirmación de captura separada de respuesta | El guardado fire-and-forget necesita resultado observable; ficha F |
| Condicionado | Adapter SaaS fuera del núcleo | Requiere caso de producto, consentimiento y contrato comercial |
| Condicionado | Auth inválida frente a backend auth caído | No destruir credenciales por indisponibilidad; ficha G, en transporte |
| Rechazar | Internals privados supuestos y segundo dashboard | No extraíbles o redundantes |

## 2. Fichas de extracción Rust

Todas **[P]**, nombres locales, no IDs oficiales ni APIs existentes. Criterios funcionales no autorizan escribir nuevos benchmarks/prompts/configuraciones.

### A. Recall por brazo, antes del texto

- **Problema:** perfil válido y búsqueda fallida no deberían convertirse en una única excepción indiagnosticable.
- **Fuente/unidad:** fan-out y render [S3:30-74,98-198]. Éxito parcial es mejora propuesta, no garantía upstream.
- **Delta/alternativas:** extender `perform_auto_recall`/`RecallResult` y `assemble_with_recall`; adapter de red opcional. Rechazar reimplementar SaaS o BM25/HNSW/RRF.
- **Contrato hipotético:** `recall(ScopedTurn, Deadline, Cancel) -> Result<RecallBundle, RecallError>`; ítems `{source_key,namespace,revision,trust,category}` y outcomes `Ready/Empty/TimedOut/Cancelled/Failed/Denied` por brazo.
- **Invariantes/errores:** scope autorizado al leer y entregar; error ≠ vacío; tainted no gana autoridad de instrucción; cancelación no publica resultado parcial como completo.
- **Seam:** V1 `context_assemble`, V2 `assemble_with_recall`, V3 `RecallResult/RecallGovernance/InjectionPolicy`; conservar budget/estimador.
- **Migración/persistencia/rollback:** DTO aditivo y dual-read; render tras selección; no migración de payload/reindexado. Desactivar nuevo render sin alterar recuerdos; no persistir respuestas remotas por defecto.
- **Dependencias:** cancelación del host y revisión de fuentes; integrar ledger/budget OpenViking sin duplicarlos.
- **FMEA:** brazo lento→bloqueo→latencia; detectar duración/outcome, deadline compartido. Parcial tratado como total→falsa confianza; degradación explícita. Revocación durante espera→fuga; revalidación de policy.
- **Aceptación/observabilidad:** ambos OK, vacío, timeout de uno, cancelación, revocación y budget mínimo; IDs conservados; contadores/outcomes/latencia sin query ni texto.
- **Mínimo/no objetivos/abierto:** bundle tipado compatible; no ranking nuevo. Resolver frescura antes de reutilizar entre turnos.

### B. Caché de continuación revisionada

- **Problema:** tool loop repite recall; mismo texto no identifica mismo turno, revisión o permiso.
- **Fuente:** key/escape/LRU [S4:8-79] y condición de hit [S5:326-378].
- **Delta/alternativas:** extender caché del host si existe; utilidad Rust solo de identidad/invalidation; rechazar caché global de strings.
- **Contrato hipotético:** `TurnRecallKey {principal_scope,session,turn_id,policy_revision,source_generation,mode}`; `get -> Hit<AuthorizedRefs>/Miss/Stale`; `invalidate(scope,generation)`.
- **Invariantes/errores:** no compartir scopes; hit no revoca `Denied`; nueva revisión no suprimida; `RevisionUnknown` obliga a miss; error no se cachea como vacío.
- **Seam:** antes de V3/render V1-V2; referencias para reautorizar, no prompt completo.
- **Migración/persistencia/rollback:** empezar efímera por sesión; invalidar entradas viejas sin revisión; no disco/reindexado. Desactivar = misses, recuerdos intactos.
- **Dependencias:** A; erasure/generaciones. Cachear selección no confirma entrega: diferente del ledger OpenViking.
- **FMEA:** borrado→hit stale→resurrección contextual; detectar generación e invalidar antes de delivery. Mensaje repetido→colisión lógica; turn_id. Stampede→trabajo duplicado; single-flight cancelable.
- **Aceptación/observabilidad:** dos sesiones/mismo texto, nuevo turno, revocación, borrado y cambio de modo; métricas hit/miss/stale por causa, key opaca sin query.
- **Mínimo/no objetivos/abierto:** mismo turno; no caché distribuida ni TTL inventado. Elegir revisión disponible sin coste mayor que recall.

### C. Dedup de presentación condicionado por modo

- **Problema:** query pierde un hecho por deduplicarlo contra perfil que nunca verá el modelo.
- **Fuente:** comentario y función `deduplicateMemoriesForMode` [S13:292-315], normalización :196-205.
- **Delta/alternativas:** extensión determinista de selección/render; no nuevo `l1_dedup`; adapter por integración duplicaría política.
- **Contrato hipotético:** `select_visible(selected_sources,candidates)->SelectionReport`, IDs/revisiones y descartes con causa; igualdad textual solo fallback declarado.
- **Invariantes/errores:** dedup solo contra fuentes autorizadas/elegidas; no fusionar identidad por minúsculas; `AmbiguousIdentity` conserva evidencia; contradicciones no desaparecen por normalización.
- **Seam:** antes de `inject_recall_block` V2; V3 ya separa persona/contexto dinámico. No cambiar juicio L1.
- **Migración/rollback:** modo sombra de descartes; habilitar por caller; sin reindexado/migración persistente. Volver al render anterior no pierde datos.
- **Dependencias:** A conserva IDs; B incluye modo; coordinar budget/ledger OpenViking.
- **FMEA:** prioridad estática tapa versión nueva→recuerdo incorrecto; detectar revisión/conflicto. Texto normalizado colapsa hechos distintos→pérdida; usar IDs.
- **Aceptación/observabilidad:** query conserva hecho también presente en perfil; full evita repetición; revisiones contradictorias distinguibles; counts/razones sin contenido.
- **Mínimo/no objetivos/abierto:** presentación, no dedup semántico ni pesos nuevos. Definir reemplazo frente a contradicción.

### D. Readiness por proyección y recibo consultable

- **Problema:** consultar tras agregar conversación antes de que exista memoria/profile.
- **Fuente:** readiness documentado [S8:71-80], polling provider [B1:104-194]; escritura/search v5 y consulta auxiliar v3 para dreaming son compatibilidad observada, no modelo a copiar.
- **Delta/alternativas:** extender jobs/recibos Vanta; adapter traduce estados externos; rechazar sleep fijo o scheduler paralelo.
- **Contrato hipotético:** `IngestReceipt {source_id,revision,accepted,durable_evidence,projections}`; `status(receipt,auth)->ReceiptSnapshot`; proyección `Pending/Ready/Stale/Failed/Unsupported`, durability desconocida explícita.
- **Invariantes/errores:** readiness atado a revisión; timeout no declara fallo definitivo; chunks ready ≠ memories ready; borrado invalida proyecciones; no filtrar existencia a principal no autorizado.
- **Seam:** V4 L0Recorder, V5 worker/checkpoints; exposición SDK/MCP y DTO Studio, sin convertir checkpoint de ejecución en memoria semántica.
- **Migración/rollback:** estado aditivo por fuente/revisión; históricos Unknown; backfill solo verificable, reindexar solo derivado ausente. Desactivar exposición mantiene trabajos; no reingesta en rollback.
- **Dependencias:** contrato durable/atomicidad de publicación; misma especificación transversal que recibos Agentmemory y estados OpenViking, con traducciones particulares.
- **FMEA:** respuesta vieja→falso ready; comparar revisión. Polling ilimitado→coste infinito; cancelación/deadline del caller. Error oculto→falsa salud; outcome explícito.
- **Aceptación/observabilidad:** pending, ready separado, extracción fallida, restart, borrado y revisión concurrente; lag/attempts/estados sin payload; no periodicidad inventada.
- **Mínimo/no objetivos/abierto:** status honesto, no exactly-once ni certification de durability SaaS.

### E. Olvido exacto con plan autorizado

- **Problema:** petición de olvido no debe destruir recuerdos por mera similitud ni permitir que el modelo autorice.
- **Fuente:** preview exacto y delete con estado/count [S13:64-168].
- **Delta/alternativas:** extender erasure Vanta; adapter SaaS solo opcional. Rechazar segunda política de borrado fuera del motor.
- **Contrato hipotético:** `plan_forget(auth,refs_or_text)->ForgetPlan{scope,refs_and_revisions,policy_revision}`; `apply(plan,confirmation)->ForgetReport{removed,stale,denied,failed}`.
- **Invariantes/errores:** preview no autoriza commit futuro; `StalePlan/PartialFailure/Denied` explícitos; ID exacto preferido; distinguir fuente/derivado/backup.
- **Seam:** V6 erasure; invalidación B, proyecciones D y ledger. Lo ya entregado al LLM no puede desentregarse.
- **Migración/rollback:** plan no destructivo primero; tombstones compatibles; reindexar afectados con capacidades existentes. Desactivar nuevos planes mantiene tombstones; **no rollback que resucite contenido purgado**.
- **Dependencias:** autorización/revisiones/inventario derivados; lifecycle Agentmemory aporta lección, no estructura KV.
- **FMEA:** cambio preview→apply→borrado de revisión nueva; detectar mismatch/replanificar. Fuera de scope→daño; filtrar antes de preview. Caché conserva dato→reinyección; invalidación.
- **Aceptación/observabilidad:** ID, texto no exacto, cero matches, borrado parcial, plan stale, revocación y consulta posterior; counts/IDs opacos sin texto a olvidar.
- **Mínimo/no objetivos/abierto:** plan exacto gobernado; no borrado físico remoto acreditado. Resolver confirmación y backups en spec.

### F. Captura post-respuesta con outcome propio

- **Caso agente:** una llamada de voz o tool loop termina correctamente y el usuario cree que la preferencia quedó recordada, aunque el guardado sigue pendiente o falla.
- **Fuente/unidad:** `wrapVercelLanguageModel` y `saveMemoryAfterResponse` [S14:182-204,246-284][S5:151-188]. La observación es promesa no esperada + error logueado; no prueba pérdida reproducida.
- **Delta/alternativas:** Vanta ya graba L0 con cursor/replay [V4:274-372]. Propagar confirmación por evento del host y estado de derivación; no reemplazar L0 por concatenación completa de conversación. Extender adapter existente, o mantener captura explícita si el host no tiene lifecycle fiable.
- **Contrato Rust propuesto:** `capture_response(auth, HostEventId, session_revision, ResponseOutcome) -> Result<CaptureHandle, CaptureError>`; `wait(handle, cancel) -> Captured/Deferred/Failed/CancelledWait`. Respuesta completada, memoria aceptada y derivado listo son dimensiones distintas.
- **Ownership/errores:** el host posee tarea pendiente hasta entregarla al scheduler; `SourceInterrupted/QueueFull/Denied/RevisionConflict` no son éxito. No mantener mutex durante red/LLM. Cancelar la espera no borra un commit ya confirmado ni duplica un reintento.
- **Seam/pasos:** 1) acordar ID estable con adapter; 2) mapear a `L0Recorder::record_turn`; 3) publicar recibo D mediante worker [V5]; 4) exponer `flush/status` al cerrar sesión; 5) reintentar con identidad de A de Agentmemory, no por similitud.
- **Persistencia/rollback:** metadata aditiva en recibos existentes o spec única; legacy Unknown. No reindexado de L0 intacto. Deshabilitar captura nueva no elimina pendientes ni revierte evidencia ya guardada.
- **Dependencias/relaciones:** D observa derivados; B se invalida al publicar revisión. El recibo de captura no es un ledger de lo entregado al modelo.
- **FMEA:** cierre proceso antes de promesa → captura incierta → detectar pendiente y recuperar; stream parcial tratado como completo → dato falso → outcome de origen; full history reintentado con ID nuevo → duplicación → ID estable y revisión.
- **Aceptación manual futura:** stream normal/cancelado, error de red tras respuesta, cierre con pendientes, reenvío y revocación. Observar outcomes/counts sin transcripción; no ejecutar en esta investigación.
- **Mínimo/no objetivos:** confirmación honesta en un adapter; no exactamente-una-vez de efectos externos, nuevos prompts ni SDK obligatorio. Abierto: política de conservar respuesta parcial y contrato de cierre del host.

### G. Error de autenticación tipado sin rebajar permisos

- **Caso agente:** JWKS o `/session` caído hace que el cliente descarte una credencial válida o confunda indisponibilidad con memoria vacía.
- **Fuente/unidad:** `TransientAuthError`, `transientAuthErrorFor`, `validateApiKey/validateOAuthToken` [S15:68-207]; RBAC [S17:1-40]. No se traslada la regex de token como verificador.
- **Delta/alternativas:** extender adaptación de errores del transporte SDK/MCP, no `InjectionPolicy` como un servicio OAuth nuevo. Reusar auth existente; adapter externo opcional; rechazar fail-open al contenido ante error de autorización.
- **Contrato propuesto:** `authorize_remote(request, credential_ref, deadline) -> Result<AuthorizedScope, AuthOutcome>` con `InvalidCredential/Denied/BackendUnavailable/Cancelled`; scope con revisión de política y caducidad, nunca token en serialización pública.
- **Seam/pasos:** resolver principal en transporte, traducir resultado antes de [V3], reautorizar al delivery, exponer outcome no sensible al host. Verificar primero middleware auth actual; no se propone una ruta de código imaginaria ni cambios en índices.
- **Concurrencia/commit:** caché por identidad+servidor+policy; no compartir token bruto en logs. Revocación durante espera exige revalidar. Cancelación corta la espera; no puede conceder acceso por conservar un resultado viejo.
- **Persistencia/rollback:** errores/metadata aditivos; caché efímera con invalidación definida por proveedor, sin TTL importado ni migración de memorias/reindexado. Rollback elimina caché nueva, no revoca tombstones.
- **Dependencias:** A conserva Denied separado de Empty; B no puede omitir auth por hit. Revisión jurídica del proveedor y aislamiento del host.
- **FMEA:** outage→invalid_token→logout destructivo, detectar clase y conservar credencial de forma segura; scope viejo→lectura indebida, invalidar/revalidar; error remoto contiene secreto→fuga, códigos/redacción.
- **Aceptación futura:** 401/403, timeout, JWKS inválido, issuer/audience ajeno, revocación con hit y cancelación. No contenido en trazas. No-goals: copiar auth comercial, introducir OAuth al núcleo o certificar RBAC completo.

## 3. Incidencias y seguridad

Consulta pública de cierre: 2026-11-07. Se leyeron cuerpos mediante GitHub `search/issues` y páginas públicas; el estado vivo no está fijado por el SHA. Un enlace a rama/PR no prueba merge ni solución.

| Fuente efectivamente leída | Fecha/estado observado | Mecanismo, alcance y relación con código |
|---|---|---|
| [#1860, LiveKit plugin silently loses all memory on supermemory 5](https://github.com/supermemoryai/supermemory/issues/1860) | 2026-10-31; abierta | Reporta `supermemory>=3.50.0` sin cota superior y firmas 3.x incompatibles con 5.0.1. `memory.py` usa `search.memories`, `profile(**kwargs)` y catches [S16:249-332,350-358]; pyproject [S9]. Es compatibilidad **LiveKit** y degradación silenciosa, no pérdida física del SaaS demostrada. Página vincula #1863; no se leyó merge de esa PR |
| [#1849, Container tag profile schema expects strings but API returns objects](https://github.com/supermemoryai/supermemory/issues/1849) | 2026-10-27; abierta por API de búsqueda | Reporte sobre contrato **v3** de `$fetch` y arrays objeto frente a string. No atribuirlo automáticamente a tools v5: [S13] ya normaliza fact y [S3] usa shape compatible. Conformance por SDK/versión es el delta; medidas p50/p95 son del reportante |
| [#1856, Self-hosted 0.0.9: ingestion blocks the event loop… queued with DELETE refused](https://github.com/supermemoryai/supermemory/issues/1856) | 2026-10-29; abierta por API de búsqueda | macOS arm64, bge local, ingesta grande, list filtrada lenta y documento queued tras reinicio; causa principal síncrona es hipótesis del reportante, implementación servidor no disponible en la muestra. Relaciona #1312/#1302/#1299, **no leídos aquí**. [S13:64-104] acredita rechazo cliente de delete no terminal, no causa del bloqueo. Aplicación Vanta: no retener recursos del query durante derivados, estado recuperable D; no trasladar tiempos ni declarar bug Rust |

**Seguridad discriminante:** la clave compartida por sí sola no identifica al usuario final; namespace debe venir de principal confiable. `getNamespace` tiene default `sm_project_default` [S13:31-39], útil para demo, peligroso si un host multitenant lo usa sin resolver scope. Contenido recuperado sigue tainted; permiso de lectura no autoriza tool ni olvido. `baseUrl` configurable es frontera de confianza de configuración: no aceptar URL del documento/LLM para enviar Bearer. No se acreditó SSRF explotable.

`buildMemoriesText` loguea query recortada y preview completo si logger lo permite [S3:119-133,187-196]; revisar verbose/sinks antes de producción, no afirmar que todo despliegue los emite. Revocar/borrar debe invalidar caché y derivados; backups/retención contractual siguen desconocidos.

**Acceso/fallback:** tras búsquedas API, lecturas individuales a `api.github.com/repos/.../issues/...` respondieron rate-limit público. En la reanudación REST volvió a responder 403 para las tres; `websearch.fetchWebContent` leyó **las tres páginas HTML completas** el 2026-11-07. #1849 incluye comentario del autor que abrió #1850; #1856 incluye aclaración del autor. No se cambiaron credenciales ni metadata ni se fingió lectura de PR/fix.

## 4. Prerrequisitos VantaDB

| Unidad | Primitiva existente que no se reinventa | Condición pendiente de adopción |
|---|---|---|
| A/C | `RecallConfig`, `RecallResult`, gobernanza y `assemble_with_recall` [V2][V3] | Propagación de identidad/outcome por fuente hasta render y delivery; no dar por activa la ruta core híbrida |
| B | Scope y provenance existentes; erasure [V3][V6] | Inventario real de cachés del host, revisión invalidable y autorización en hit |
| D/F | `L0Recorder::record_turn`, `PipelineWorker::run_once/reclaim_stale` [V4][V5] | Definir qué ack acredita persistencia y qué checkpoint acredita proyección; recuperación sin LLM bajo lock |
| E | Borrado/crypto-erasure existente [V6] | Inventario de derivados y confirmación contra revisión; plan no sustituye permiso |
| G | `InjectionPolicy` y transports ya existentes | Resolver auth en borde; revisar traducción de errores actual antes de añadir DTO |

No autorizar specs/implementación hasta discriminar estas garantías. Los tipos propuestos no son APIs Vanta presentes.

## 5. Rechazos y costes ocultos

- No portar internals SaaS imaginados ni usar `CLAUDE.md` histórico como implementación de Cloudflare/DB actual.
- No adoptar namespace por defecto para usuarios distintos; ni confundir cache hit, tool success, documento aceptado y memoria lista.
- No importar scores/thresholds, templates, modelos o política de error “seguir sin memoria” como defaults de Vanta.
- No requerir proveedor/red para motor embebido. Adapter SaaS implica egress, precio, retención, credenciales, dependencia de versión y caída externa.
- No prometer olvido físico por `count>0`, ni desentrega de texto ya enviado, ni aplicar preview sin reautorizar.
- No reemplazar Studio por consola ajena ni fusionar fichero MIT, SDK externo, binario self-hosted y servicio bajo una sola licencia.

## 6. Advertencias metodológicas

La ampliación conserva A-E y completa el tramo interrumpido. Claims comerciales siguen claims; no se recalcularon estrellas/SHAs de inventario ni se ejecutaron ejemplos. La muestra cliente demuestra llamadas, no algoritmo privado. La lectura de la versión 3.x de un endpoint no autoriza declararlo contrato v5.

El pin remoto MemoryBench describe una fuente complementaria, no nueva versión del clon. `package.json` conserva dependencia SDK `^4.0.0`, pero el provider leído hace fetch v5 y una consulta auxiliar v3; **dependencia declarada no determina por sí sola la API efectiva**. Diferenciar superficie acreditada, inferencia y prueba futura en cada recomendación.

## 7. Benchmarks y adaptación

### 7.1 Runner del clon: ejemplos ejecutables, no QA

`packages/docs-test/run.ts::main/runTest` [S7:70-154] enumera ejemplos TS/Python/integraciones, filtra por argumento y hace spawn secuencial con entorno heredado. Éxito = exit code cero; errores de spawn son fallo, sin timeout de proceso en ese cuerpo. No calcula recall, calidad de memoria ni exactitud QA.

El quickstart TS [S18:1-60] usa `docs-test-user`, consulta perfil, imprime respuesta/contexto y guarda una conversación literal. Todavía pasa `containerTag/q` y `client.add({...})`: **ejemplo legacy frente al contrato v5 de [S8]**, no benchmark validado del SDK instalado. No hay gold, juez, split, semilla ni cleanup/readiness en ese ejemplo. Puede llamar red, escribir recuerdos y exponer texto en logs. Su “test passed” es un mensaje, no una aserción semántica.

### 7.2 MemoryBench complementario fijado

Fuente `supermemoryai/memorybench@063b86bfb347336def3a084153f40bfea1e2dfd6`. README y MIT leídos; no clonado ni ejecutado. Se estudió LongMemEval y fases compartidas; no se atribuye cobertura completa de LoCoMo/ConvoMem.

| Eje contractual | Evidencia de runner efectivamente leída |
|---|---|
| Entry point | `src/index.ts:1-4 → cli → runCommand → orchestrator.run`; fases ingest/index/search/answer/evaluate/report [B2][B3] |
| Defaults y continuación | Juez `gpt-4o`; orchestrator default answerer `gpt-4o`. `runCommand` conserva provider/benchmark/settings al continuar y permite cambiar modelo; reusar checkpoint requiere revisar compatibilidad de artefactos, no asumir ejecución fresca [B2:13,165-243][B3:78-138] |
| Dataset/unidad | Loader LongMemEval descarga `longmemeval_s_cleaned.json` de URL HF `main` si falta, parte por pregunta y elimina `has_answer` al partir. Pregunta contiene groundTruth; sesiones nuevas usan `{question_id}-session-{i}`, roles y fechas normalizadas [B4:89-105,160-234]. No acredita hash de dataset/split congelado por ese mero URL |
| Filtros/split | `getQuestions` aplica tipos, offset y limit [B4:240-255]. Orchestrator tiene sampling completo/limit/por categoría; random usa `sort(Math.random()-0.5)` sin semilla en ese cuerpo [B3:35-67]. Son opciones upstream **descritas**, no selección propuesta ni holdout certificado |
| Ingesta | Provider formatea sesión por líneas speaker/role, preserva session ID y metadata/fecha; supportingContext/dreaming solo si config los define [B1:40-50,104-138]. Sin evidence de extracción interna privada |
| Readiness | Poll v5 `system.status`; si done consulta `/v3/documents/{id}?namespace=...` por `dreamingStatus`. Rechazos de polling se omiten y pending continúa; no deadline global en método; errores terminales se cuentan y se avisa, sin throw final [B1:140-194] |
| Efecto en fase index | `runIndexingPhase` registra `failedIds` y aun puede marcar fase completed cuando provider retorna; comparte resultado por containerTag [B5:108-125,176-205]. Por tanto “indexing completed” no significa todos los documentos sanos |
| Search | `runSearchPhase` llama únicamente con pregunta y tag, limit `benchmark.searchLimit ?? 10`, threshold 0.3; provider puede sobreescribir settings [B6:49-85][B1:196-211]. Los valores son del runner, no recomendaciones Vanta |
| Answerer | Usa resultados guardados y fecha, template benchmark/provider/default y configuración de modelo. Cuenta prompt/base y resta para contextTokens; `generateText` usa maxTokens/temperatura del modelo [B7:57-78,119-177]. No se auditó todo template ni registro de alias |
| Juez y ground truth | `runEvaluatePhase` pasa gold e hypothesis al juez y en paralelo evalúa relevancia de chunks con gold [B8:56-85]. Search no recibe gold en su llamada; persistir gold junto a resultados exige aislamiento de artefactos, no prueba por sí solo fuga al retrieval |
| Calidad/denominador | Report incluye únicamente preguntas con checkpoint y evaluate completed; accuracy cuenta score exactamente 1 sobre evaluadas. Preguntas sin respuesta truthy no entran a evaluate [B8:24-29][B9:96-169,238-285]. No equivale a success sobre todas las intentadas |
| Latencia/tokens/coste | Search incluye llamada provider/retries; no ingest/extracción/answer/juez. Total reportado suma fases, no wall time de ejecución concurrente. Latencias agregadas solo de evaluadas y duraciones truthy; arrays vacíos devuelven ceros. Tokens client-side, no factura [B9:46-68,108-146,189-231][B10] |
| Concurrencia/cache | Provider declara default 50, ingest 20, indexing 50; fase usa resolver y overrides. Retry HTTP 429/5xx hasta ocho reintentos con jitter; sin timeout de fetch visible [B1:55-59,81-102]. Checkpoints reutilizan resultados; no warmup independiente certificado |
| Efectos/reset | Descarga HF, JSONs de preguntas/resultados/checkpoints/reports, llamadas de memoria y modelos pagados. `clear` del provider solo warn, **no borra** [B1:214-216]. `--force` del checkpoint no acredita limpieza remota ni reset de tenant |

**Métricas retrieval reales [B11:88-146]:** juzga los primeros k=10 resultados, precisión = relevantes / resultados efectivos, hit = algún relevante, y **`recallAtK = hitAtK`**; `totalRelevant=max(1,relevantRetrieved)`, no total de sesiones gold. MRR usa primer relevante; nDCG idealiza los relevantes observados. En vacío devuelve k=0/ceros; fallo de llamada/parse del juez se degrada a todas las relevancias cero [B11:54-69], no outcome operativo separado. No presentar esto como `recall_all` por sesiones de Vanta ni invalidar por ello todas las cifras comerciales.

### 7.3 Adaptación acotada a METHOD/BENCHMARK-NOTES

Un adapter Vanta futuro debe mapear `UnifiedSession.sessionId → source/session refs`, fechas, principal/tag único, estados de ingesta D y resultado de search con provenance. Reusar SDK/bindings y `RecallConfig` existentes; el namespace saneado/truncado por `toNamespace` no basta como identidad universal sin comprobar colisiones. Describir implementación externa y configuración efectiva; no inventar ruta Rust actual para MemoryBench.

Conservar pistas separadas: retrieval por IDs de sesión en evaluadores Vanta frente a relevancia LLM y QA MemoryBench. No copiar prompts, scoring, threshold 0.3, sampling ni límites. Aplicar protocolo existente: dataset/modelos/splits fijados, gold fuera de ingesta/query, registro de intentadas/completadas/errores, latencia por fase y presupuesto efectivo. MemScore es triple calidad/latencia/contexto, **no coste total ni comparación equivalente**. LoCoMo Vanta tiene límites fake/sintético documentados en [BENCHMARK-NOTES](BENCHMARK-NOTES.md#6-revalidacion-estatica-de-locomo-durante-la-ampliacion), no baseline certificada contra SaaS.

No se proporciona comando listo para ejecutar que pueda escribir en cuenta compartida; la ejecución posterior necesita autorización, instancia desechable y plan de cleanup real, dado que `clear` no lo realiza.

## 8. Matriz de capacidades

Estados por contrato, no ranking ni ausencia global desde la muestra:

| Capacidad | Supermemory observado | Vanta `97c9623` | Decisión |
|---|---|---|---|
| Motor BM25/HNSW/RRF y filtros | No acreditado internamente por cliente/SaaS | Existe; ruta de memoria core configurable [V3] | No portar buscador supuesto |
| Perfil + query | Existe composición y modos [S3][S13] | Parcial para este flujo: personas/recall/assembly existentes [V2][V3] | A/C; no “personas ausentes” |
| Dedup de presentación | Existe condicionado por modo [S13:292-315] | Parcial para ese contrato, dedup L1 ya existe | C solo selección visible |
| Caché revisionada | Parcial: LRU/key sin revisión [S4] | No acreditado por muestra de host | B condicionado a inventario |
| Readiness por derivado | Existe documental; provider mixto v5/v3 [S8][B1] | Parcial para recibo expuesto; worker/checkpoints existen [V5] | D, no scheduler paralelo |
| Captura idempotente post-respuesta | Parcial: ID de conversación y promesa no esperada [S14] | Existe L0 con replay/cursor; recibo host universal no acreditado [V4] | F |
| Borrado exacto | Existe selección por IDs/igualdad normalizada [S13] | Existe erasure; plan fuente/derivados completo no acreditado [V6] | E, no prometer backups |
| Auth/error transitorio | Existe JWT/session/error tipado [S15]; enforcement total no acreditado | Existe InjectionPolicy; transporte remoto específico no acreditado [V3] | G en borde |
| Benchmark QA equivalente | No acreditado: protocolo externo distinto [B8][B11] | No acreditado por el contraste | No superioridad |
| Web/grafo/desktop | Web redirect + componente React existentes [S11][S12]; desktop no acreditado | Studio existe en worktree aparte | Ampliar vistas existentes |
| Paridad self-hosted/SaaS | No acreditado; opción documentada [S8] | No aplicable como requisito de motor local | No inferir inexistencia de local |

## 9. Negativos y falsos positivos de VantaDB

- **Hecho estático:** `record_turn` escribe mensajes y después cursor [V4:348-364]; no acredita recibo durable con estado de todas las proyecciones. No implica pérdida demostrada ni ausencia de replay.
- **Hecho contractual:** assembler protege prefijo aunque exceda budget; `apply_recall_budget` usa caracteres y `core_search` no es default en la muestra [V2][V3]. No afirmar límite exacto en tokens ni uso híbrido universal.
- **No acreditado:** caché del host invalidada por policy/source generation, manifest conjunto de revisión y outcomes separados de auth/retrieval/captura. Son preguntas delimitadas, no bugs actuales.
- **Falso positivo:** Vanta ya tiene BM25/HNSW/RRF, provenance, dedup/supersession, sesiones, budgets, scheduler/checkpoints, erasure y Studio; ninguno se recrea por ver una API SaaS.
- No trasladar bloqueo de event loop Bun del reporte #1856 a Rust/WAL Vanta ni tratar “count” como garantía de purga física en ninguno.

## 10. Producto UI y Studio

**UI realmente leída:** `apps/web/app/page.tsx:1-60` muestra traslado y redirige tras cinco segundos a consola [S11]. No es el código completo del dashboard comercial. El componente `use-graph-data` calcula bordes recent/expiring/forgotten, estilos `document/derives/updates/extends` y conectividad por documento/relación [S12:43-69,115-165]. Flujo: documentos+memorias recibidos → mapa/adyacencia → componentes de layout; color olvidado representa un campo recibido, no prueba borrado backend. No se auditó render, paginación, cancelación ni todo playground.

**Aplicación concreta a Studio existente:** la lectura original corresponde al
worktree modificado `develop-desktop`: `desktop/src/components/memory/MemoryLens.tsx::MemoryLens:488-550`
(sesión/tabs), `components/activity/ActivityPanel.tsx::ActivityPanel:38-89`
(auditEvents paginado) y `components/lens/retrieval/RetrievalLens.tsx::RetrievalLens:54-95`
(búsqueda/filtros/fusión). Estos rangos no se atribuyen al contenido de HEAD.
La referencia inmutable `0ad38c542b39a589035b0e17fdedb4c09dca24b9` conserva
esos componentes bajo `feature-freez/desktop/src/components/`, no bajo las
rutas actuales; véase la [identidad Studio](VANTA-REFERENCE.md#identidad-de-la-evidencia-studio).
No se acredita equivalencia de bytes entre ambas lecturas.
`graph/GraphScene.tsx` se conserva como ruta conocida, no cuerpo revalidado.
No se certificó runtime ni todo el worktree.

Integrar D/F en actividad: aceptado, proyección ready, error y uso/desconocido por separado; A/C en retrieval: evidencia autorizada y razón de descarte; E en memoria: preview exacto/revisión y resultado parcial. Un DTO Rust de las fichas viaja por transporte existente; la UI no decide permisos ni “entregado” desde compilado. Captura pendiente no debe mostrarse como guardada. Reintento/olvido requiere reautorización en backend y logs sin contenido. No añadir dashboard, consola comercial embebida ni paquete React como dependencia del motor.

## 11. Cobertura, fuentes y decisiones pendientes

### Ledger de lectura

| Grupo | Cuerpos/rangos y límite |
|---|---|
| Cliente/tools | [S3] fan-out/render; [S4] completo; [S5:151-188,311-379] captura/recall; [S13:1-205,292-315]; [S14:118-297] caller/captura. No todos los wrappers/tool schemas |
| Auth | [S15:1-207] completo, [S17] RBAC; no enforcement de todas las herramientas MCP ni servidor de auth privado |
| LiveKit | pyproject [S9]; [S16] rutas recall/search/remember/forget/store localizadas y catches leídos; no adapter completo ni CI ejecutada |
| UI/docs | Web 1-60 y hook grafo 1-165; docs-test runner/quickstart completos. Consola SaaS y frontend completo no leídos |
| Benchmark remoto | README, package/LICENSE, provider completo, orchestrator selección/defaults/fases, CLI run completo, loader LongMemEval, fases compartidas search/answer/evaluate/report/indexing, tokens y retrieval-eval. No todo prompt/model alias ni LoCoMo/ConvoMem/judge individual |
| Vanta | Referencia contractual y lectura pin L0/worker/merge; no revisión exhaustiva de todos los transports ni Studio |

**Lagunas restantes:** implementación del motor/servicio y servidor local, ACL end-to-end, semántica de backups/purga, compensación de capturas interrumpidas, datasets/modelos/prompt efectivos de resultados publicados, fórmulas de cada juez QA y pruebas dinámicas. API pública limitada impidió ampliar algunos comentarios; se conservan reportes y no se inventan fixes. No apps/tests/builds/benchmarks/instalaciones/servicios ni validadores globales.

**Plan de separación (>300 líneas):** si se aprueban specs, separar outcomes/identidad/caché (A-C/G), ingesta/olvido/captura (D-F) y adaptación/UI (§7/§10); mantener aquí mapa, decisiones y evidencia. No se crean specs ni se reservan IDs ahora. Autoría completada en el alcance declarado; **no revisión independiente ni aceptación final**.

[S0]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/LICENSE
[S1]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/package.json
[S2]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/package.json
[S3]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/shared/memory-client.ts
[S4]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/shared/cache.ts
[S5]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/vercel/middleware.ts
[S6]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/docs/memorybench/extend-benchmark.mdx
[S7]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/docs-test/run.ts
[S8]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/AGENTS.md
[S9]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/livekit-sdk-python/pyproject.toml
[S10]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/mcp/src/server/tools/index.ts
[S11]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/web/app/page.tsx
[S12]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/memory-graph/src/hooks/use-graph-data.ts
[S13]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/tools-shared.ts
[S14]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/vercel/index.ts
[S15]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/mcp/src/server/auth/index.ts
[S16]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/livekit-sdk-python/src/supermemory_livekit/memory.py
[S17]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/mcp/src/server/auth/rbac.ts
[S18]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/docs-test/tests/typescript/quickstart.ts
[B1]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/providers/supermemory/index.ts
[B2]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/cli/commands/run.ts
[B3]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/index.ts
[B4]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/benchmarks/longmemeval/index.ts
[B5]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/phases/indexing.ts
[B6]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/phases/search.ts
[B7]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/phases/answer.ts
[B8]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/phases/evaluate.ts
[B9]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/phases/report.ts
[B10]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/utils/tokens.ts
[B11]: https://github.com/supermemoryai/memorybench/blob/063b86bfb347336def3a084153f40bfea1e2dfd6/src/orchestrator/phases/retrieval-eval.ts
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L932
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L274-L372
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/erasure.rs

## 12. Cierre incremental: fronteras y contratos sin duplicar infraestructura

### 12.1 Receipts y garantía por etapa

**Corrección preventiva de terminología:** [WriteReceipt](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L657-L735)
ya existe. Usa binding/integridad SHA-256, **no firma criptográfica de un actor**.
D/F deben correlacionar ese recibo con el evento del host y el estado de las
proyecciones, no proponer recibos como concepto ausente. `MutationReceipt`
C13 es **solo propuesta** transversal, no API vigente ni permiso de implementación.
[ErasureReceipt](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/erasure.rs#L86-L108)
también existe; E propone preview/revisión de objetos exactos, no reemplazar
erase_scope ni certificar backups que el recibo declara fuera de alcance.

| Frontera | Hecho cliente a S SHA fijado | Qué no se puede concluir |
|---|---|---|
| Ingesta | `addConversation` se espera dentro de saveMemoryAfterResponse, caller no espera esa promesa [S5][S14] | Stream flushed no prueba persistencia; timeout remoto no prueba que servidor no escribió |
| Recall | Promise.all perfil/search; `maxRetries:0` y abortSignal opcional [S3:30-74]; middleware arma timer y limpia en finally [S5:344-378] | Un brazo fallido no garantiza parcial; abort no demuestra cese del cómputo SaaS |
| Continuación | key namespace/thread/mode/texto normalizado; hit solo si no es turno nuevo y valor truthy [S4][S5:326-338] | LRU no contiene revisión fuente/policy; tampoco prueba ausencia de invalidación fuera de esos cuerpos |
| Update/delete | deleteDocument hace GET, valida terminal y DELETE count=1; forget preview exacto luego IDs [S13:64-168] | No CAS visible entre preview y apply; count no acredita purga física, invalidación de perfil o backup |
| Concurrencia | cache local por instancia y trabajo async de captura [S4][S14] | No single-flight distribuido, transacción global o exactly-once de host→SaaS |

### 12.2 Completar pasos/errores de A–G

Complementos **PROPUESTOS** a las interfaces ya presentes, no nuevos mecanismos:

| Ficha | Pasos ejecutables futuros y contrato que falta resolver | Mínimo, cancelación y rollback |
|---|---|---|
| A bundle | Inventariar outcomes actuales → mapear brazos con scope/revisión → seleccionar con budget → render una vez → reautorizar antes de delivery. `RecallError::{InvalidScope,Storage,Provider,Cancelled}` conserva fuente del fallo | Solo DTO/adapter; sin persistencia de respuesta. Cancelled no equivale Empty; volver al render anterior no modifica fuentes |
| B key | `get(key: &TurnRecallKey) -> Result<CacheLookup, CacheError>` con Hit/Miss/Stale y RevisionUnknown; identificar generación disponible antes de habilitar caché | Efímera; invalidación por generación/policy; no TTL copiado. Desactivar fuerza misses, no recupera prompts revocados |
| C selección | `select_visible(refs: &[AuthorizedRef], candidates: &[CandidateRef]) -> Result<SelectionReport, SelectionError>`; validar IDs/revisiones, dedup solo conjunto elegido, aplicar budget existente | Sin escritura; InvalidIdentity no se normaliza silenciosamente. Cancelación descarta selección incompleta |
| D estado | Correlacionar WriteReceipt/evento → consultar estado existente worker → devolver fuente/revisión/proyecciones. `status` devuelve Denied/NotFound/Stale/Unavailable, no éxito por timeout | Históricos Unknown; sin jobs nuevos. El host puede dejar de esperar sin cancelar un commit durable |
| E olvido | Preview no destructivo → confirmar principal/refs/versiones → usar erasure real → invalidar B → exponer parcial por superficie | `apply` no autoriza ampliar scope. Después de purga no hay rollback de contenido; pendientes se reconcilian con mecanismos existentes |
| F captura | Asignar HostEventId antes de generar → conservar outcome complete/interrupted → entregar a L0/scheduler → consultar D → cerrar host con pendientes visibles | Sin retener locks durante stream/red. Captured no implica derivado ready; cancelación de wait no se reenvía con ID nuevo |
| G auth | Resolver principal en transporte → traducir 401/403 vs backend caído → obtener scope autorizado → revalidar policy en hit/delivery → sanitizar errores | No OAuth en core ni token serializado; fail-closed a contenido, no logout por indisponibilidad. Caché nueva se descarta al desactivar |

Los seams no son carpetas genéricas: A/C→`assemble_with_recall:252-329` [V2],
`RecallConfig:302-335`/`apply_recall_budget:1159-1189` [V3]; D/F→
`L0Recorder::record_turn:274-372` [V4], `PipelineWorker::run_once:246`/
`reclaim_stale:265` y `MemoryTaskHandler::context_assemble:932` [V5];
E→`erase_scope:191`/`emit_cascade_erasure_receipt:232` [V6].
G usa autorización del transporte existente antes de `InjectionPolicy`;
el middleware concreto del futuro host SaaS sigue abierto, **no se inventa
un símbolo Rust implementado** para ese adapter.

### 12.3 Licencia, cobertura y cierre propio

Licencias por componente inspeccionado:
raíz [S0] MIT; tools [S2] MIT; [memory-graph package.json:1-45](https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/memory-graph/package.json#L1-L45)
MIT; LiveKit pyproject [S9:1-36] MIT. No extrapolar a términos del SaaS,
binario self-hosted reportado, SDK `supermemory` descargable, pesos de modelos,
dependencias React/LiveKit ni datos de MemoryBench; cada distribución requiere
su revisión. Traducción Rust no elimina derivación ni notices.

Relectura de autoría: S3 fanout/render, S4 entero, S13 31–168,
S5 316–383, S14 241–289, S11 1–60, S12 43–69; benchmark remoto B1 completo,
B11 completo y B9 1–235, al SHA previamente fijado. Se confirmó `clear` no-op,
`recallAtK=hitAtK`, descarte de no-evaluadas y persistencia de gold/answers;
no se modificó runner, prompt, scoring, umbral o config.
Los snippets de documentación legacy siguen evidencia de deriva de contrato,
no benchmark del motor ni ausencia global de self-hosting.

FMEA y aceptación de §2 permanecen casos **futuros manuales**, no resultados.
No ejecución ni medición de calidad/latencia/coste/seguridad/visual; motor
privado y licencias dependientes siguen lagunas explícitas. Siete mecanismos
quedan entregados a revisión independiente; no autoaceptación ni cierre por
cantidad de encabezados. Se mantiene el plan de separar specs de §11.
