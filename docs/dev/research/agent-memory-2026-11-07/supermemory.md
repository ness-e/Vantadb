---
title: "Supermemory: fronteras de integración, caché por turno y grafo visible"
kind: research
status: proposed
description: "Estudio estático de integraciones y UI públicas sin atribuir al clon los internals del motor SaaS"
tags: [research, memory, engineering]
---

# Supermemory: fronteras de integración, caché por turno y grafo visible

## Identidad, superficie y dictamen

- Repositorio `supermemoryai/supermemory`; clon `C:\Users\Eros\supermemory`.
- SHA congelado `16ed414e0e0c61c946b7687ec813b2064fe75dd4`, rama `main`.
- **31 559 estrellas**, MIT: [candidatos](github-candidates.json) y
  [manifest](clone-manifest.json), sin nuevas consultas.
- Método: [METHOD](METHOD.md). Vanta base `c561158`, HEAD comunicado `167435a9e`.
- **Límite central:** el clon acredita integraciones, un servidor MCP,
  documentación, playgrounds y componentes de visualización; las SDKs y
  el motor de memoria no quedan acreditados como implementación completa aquí.
  No se atribuyen algoritmos privados del SaaS a partir de sus respuestas.

**Dictamen:** extraer contratos de recuperación/captura y caché por turno,
no «reimplementar Supermemory». Los nombres updates/extends/derives de la UI
son vocabulario visible, no prueba de cómo el servidor infiere esas relaciones.
Vanta ya tiene retrieval híbrido, provenance, supersession, dedup, assembler,
compaction, scheduler y Studio: el posible valor es su composición y observabilidad.

## 1. Arquitectura, tecnologías, flujo y recomendaciones

**Código/manifiestos:** monorepo Bun/Turbo TypeScript, paquetes Python y React;
`package.json:1-80` declara Bun 1.3.6, Node >=20 y scripts de desarrollo/build.
`postbuild` sube sourcemaps a Sentry [S0]; no es un paso inocuo de lectura.

**Instrucciones del clon:** `AGENTS.md` diferencia API v5 de documentación
legacy y sitúa SDKs fuera de este repo. No se usa el viejo `CLAUDE.md` como
evidencia de que exista hoy un backend `IngestContentWorkflow` completo.
Las APIs documentadas no se verificaron contra servicio real.

Flujo **implementado en integración**, no descripción del motor remoto:

1. Middleware extrae último mensaje de usuario y modalidad profile/query/full.
2. Forma clave de turno; si es continuación y hay texto cacheado lo reutiliza.
3. Si necesita consulta, puede crear AbortController con deadline configurado.
4. `supermemoryProfileSearch` llama en paralelo a profile y search según modo;
   usa namespace, `searchMode: "memories"` y `maxRetries: 0` [S1][S3].
5. `buildMemoriesText` deduplica perfil estático/dinámico y resultados,
   arma datos para una plantilla e inyecta texto en parámetros del modelo.
   Ese formateo **no es el algoritmo de extracción del backend**.
6. El archivo middleware también contiene ruta `saveMemoryAfterResponse`;
   se localizaron sus llamadas, pero no se leyó su cuerpo completo:
   no se certifica aquí persistencia ni readiness de captura.

`MemoryCache` es LRU de hasta 100 entradas. Su clave concatena namespace,
thread, modo y mensaje normalizado, escapando `%` y `:` en segmentos de
namespace/thread [S2]. No incluye por sí misma revisión de memoria ni TTL.
La instancia de caché y el control `isNewUserTurn` son parte de su contexto.

**Recomendación Vanta:** no introducir otro cliente SaaS en el núcleo.
Modelar recall como resultado tipado y acotado que se entrega al assembler
actual; si se cachea un turno, conservar identidad, revisión y política de
invalidación. El fallo de un proveedor no debe confundirse con «no hay recuerdos».

## 2. Unidades mínimas candidatas en Rust

### Unidad A — fan-out de perfil y consulta con resultado parcial explícito

- **Problema:** la latencia de memoria antecede a generación; una excepción
  de un brazo puede hacer perder datos válidos del otro si solo existe una
  promesa y un string de salida.
- **Fuente mínima:** `supermemoryProfileSearch` y `buildMemoriesText` [S1],
  cancelación del middleware [S3].
- **Seam Vanta:** `vanta-memory/src/context_engine/engine.rs` y recall L1;
  integrar en el ensamblado y presupuesto ya compartidos.
- **DTO HIPOTÉTICO Rust:** `RecallBundle { profile_items, query_items,
  source_revisions, branch_outcomes }`; `BranchOutcome` distingue disponible,
  vacío, cancelado y error. No es una API existente acreditada.
- **Interfaz HIPOTÉTICA:** `recall_for_turn(ScopedRecallRequest) ->
  Result<RecallBundle, RecallError>`; renderizado queda fuera del puerto.
- **Acciones concretas:** identificar brazos actuales; propagar deadline y
  cancelación; conservar evidencia; deduplicar por identidad/revisión;
  ensamblar dentro del budget existente; comunicar degradación sin incluir
  texto sensible en errores.
- **Dependencias/relaciones:** provenance/dedup actuales y cancelación de
  providers; Unidad B puede evitar fan-out repetido, pero no cambia permisos
  ni determina relevancia. El perfil no autoriza recuperar otro scope.
- **FMEA:** un brazo lento bloquea ambos; error común descarta perfil válido;
  cadenas formateadas pierden IDs; LLM trata memoria no confiable como
  instrucción; retry invisible aumenta coste.
- **Aceptación funcional propuesta:** sin brazo vectorial sigue disponible
  evidencia textual autorizada; cancelación se distingue de vacío; el
  assembler no rebasa presupuesto ni repite evidencia; cada ítem conserva
  puntero de origen; no se exige LLM ni servicio remoto para primitivas puras.
- **No objetivo:** copiar plantillas, instrucciones del sistema o el umbral
  `0.6` del cliente upstream como criterio de calidad Vanta.

### Unidad B — caché de continuación con identidad y revisión

- **Problema:** un loop de herramientas puede repetir recall del mismo turno;
  reusar texto del turno equivocado puede exponer o reintroducir evidencia vieja.
- **Fuente mínima:** `MemoryCache.makeTurnKey/get/set/clear` [S2] y decisión
  de hit en `transformParamsWithMemory` [S3].
- **Seam Vanta:** sesión/turno del context engine y revisión de fuentes;
  cachear candidatos antes de su presentación, no duplicar almacenamiento.
- **DTO HIPOTÉTICO:** `TurnRecallKey { scope, session, turn_id, mode,
  source_generation, policy_revision }`; `CachedRecall { refs, outcomes }`.
  IDs tipados, no una concatenación ambigua ni hash del mensaje como identidad.
- **Interfaz HIPOTÉTICA:** `lookup(key) -> Option<CachedRecall>` e
  `invalidate(RevisionEvent)`; política de invalidación pertenece a aplicación.
- **Acciones:** mapear caché ya existente; usar ID estable de turno;
  limitar tamaño/vida según contrato vigente; invalidar al borrar/sustituir o
  revocar; cancelar single-flight al cambiar de sesión; registrar solo
  metadatos operativos.
- **Dependencias:** eventos de revisión y permisos, retención y cierre de
  sesión. No añadir caché si el assembler ya cumple el contrato.
- **FMEA:** mismo mensaje en turnos distintos colisiona; scope omitido filtra;
  supresión/revocación no invalida; resultado vacío no se cachea y produce
  fan-out repetido; normalizar espacios modifica identidad de código literal.
- **Aceptación propuesta:** scopes y turnos distintos no comparten entrada;
  continuación del mismo turno puede reutilizarla; revisión/política nueva
  invalida; borrado impide previews recuperables; error no se almacena como
  éxito vacío; cotas y cancelación no dejan tareas huérfanas.
- **No objetivo:** portar LRU o dependencias JS; elegir mecanismo Rust solo
  después de comprobar necesidad y estructuras actuales.

## 3. Defectos, riesgos e issues con nivel de evidencia

- **Código:** `Promise.all` falla como conjunto ante error de profile o search
  ([S1], líneas 45-64). Hay pérdida de resultado parcial en este helper; no se asegura
  que el llamador final no implemente otra política.
- **Código:** logging incluye namespace, preview de consulta y, en debug,
  el texto completo de memorias ([S1], líneas 124-130,190-195). **Riesgo condicionado:**
  logs habilitados/enviados fuera pueden duplicar datos sensibles.
  No se auditó la configuración de logging del servicio ni se afirma exfiltración.
- **Código:** la LRU no tiene TTL/revisión en el cuerpo leído [S2]; el
  middleware solo reutiliza en continuación. **Hipótesis:** escritura o
  revocación durante ese loop puede dejar texto stale; no implica aislamiento
  roto entre namespaces en la implementación actual.
- **Código:** ejemplos docs-test usan `profile({containerTag,q})` y
  `add({containerTag,...})` [S5], mientras AGENTS exige namespace v5.
  **Deriva observada de forma API**, no fallo ejecutado: no se resolvieron
  versiones SDK de ese workspace para determinar compatibilidad efectiva.
- **Issues:** no investigados. Ningún número o estado de incidencia inventado.

## 4. Prerrequisitos Vanta

Revalidar integración actual de recall/assembler, cachés y revisión de fuentes.
Definir política de memoria como datos no confiables, redacción de logs,
cancelación, retención y borrado. Aislamiento y autorización preceden a fan-out.
Conservar la opcionalidad de embeddings/LLM. No implementar namespaces
adicionales si los scopes actuales ya cubren el caso.

## 5. Qué no trasladar

No deducir el motor de contradicciones, dreaming, forgetting o extracción
multimodal desde schemas/README. No acoplar Vanta al endpoint remoto.
No reemplazar su presupuesto de contexto por inyección de strings sin referencias.
No copiar umbrales, prompts ni separar un segundo perfil paralelo sin integrar
personas y escenas existentes. No tomar un SDK externo como código del servidor.

## 6. Método, licencia y superficie no visible

MIT en snapshot y `LICENSE:1-21` [S9]; revisar cada paquete y dependencia antes
de copiar código. Reimplementación independiente requiere diseño propio:
una traducción literal a Rust sigue pudiendo ser derivada.
Clon superficial de rama única; no se descargaron otros repos ni modelos.
Esta lectura no acredita índices, transacciones, datos de entrenamiento,
retención efectiva o aislamiento interno del SaaS.

## 7. Runners upstream, datasets, métricas y efectos

**Runner local leído:** `packages/docs-test/run.ts:1-154` [S4].
Enumera ejemplos TS/Python e integraciones, permite filtro y lanza subprocesos
con entorno heredado. Cuenta éxito por exit code; no mide recall, accuracy,
latencia de ingestión ni capacidad de memoria. Depende de Bun y Python en
`.venv`; cada ejemplo puede necesitar claves y dependencias externas.

En `tests/typescript/quickstart.ts:1-60` [S5] el «dataset» es conversación
literal pequeña con usuario fijo `docs-test-user`: consulta perfil, forma
contexto y envía conversación al backend. No hay aserción de calidad ni polling
de readiness en ese cuerpo. Efectos: llamadas remotas, escritura de memoria y
logging de resultados/contexto; sin cleanup visible. El runner padre no
impone timeout de proceso en el tramo leído.

**MemoryBench:** `apps/docs/memorybench/extend-benchmark.mdx` menciona
LoCoMo/LongMemEval/ConvoMem e interfaz de benchmark, y enlaza a otro repo [S6].
Es **documentación**, no runner de esos datasets inspeccionado en este clon.
No se inventa scoring, judge, budget, costes ni resultados del servicio.
Los claims numéricos del README quedan como claims no reproducidos.
Sin diseñar/cambiar prompts, evaluadores, thresholds, muestreo o configuración;
comparaciones futuras solo según [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Capacidad | Estado usado | Diferencia útil |
|---|---|---|
| BM25/HNSW/RRF | Existe | no se traslada el buscador SaaS |
| Personas/escenas/provenance/supersession/dedup | Existe | conservar IDs al formatear |
| Assembly/compaction | Existe | resultado tipado antes del render |
| Scheduler/checkpoints/ingestors | Existe | no sustituir por middleware |
| Composición perfil + query | Parcial en este contraste | piezas existentes; paridad de flujo no certificada |
| Caché revisionada de continuación | No acreditado | comprobar implementación actual |
| Algoritmos internos SaaS | No aplicable como fuente de código | no visibles en esta lectura |
| Superioridad en MemoryBench | No acreditado | runner externo no inspeccionado |

## 9. Deuda sugerida sin atribuir bugs a Vanta

Verificar si hoy se distinguen errores por brazo y si el render conserva
referencias; revisar invalidación de cachés al borrar/revocar; comprobar
que logs no persisten contexto sensible por defecto. No se afirma que Vanta
falle en esos casos. Revalidar HEAD actual antes de convertir preguntas en tareas.

## 10. UI/escritorio y aplicación a Studio

**Código:** `apps/web/app/page.tsx:1-60` es una página de traslado a consola,
con redirección tras cinco segundos; no el dashboard SaaS [S7].
`packages/memory-graph/src/hooks/use-graph-data.ts:1-165` contiene lógica real
de colores por antigüedad/olvido, estilos de relaciones y agrupación de memorias
por conectividad [S8]. Es frontend sobre datos recibidos, no prueba del
algoritmo servidor que creó cada arista.

Studio Tauri/React ya existe en `develop-desktop`. Aplicación candidata:
mostrar origen, supersession, estado de disponibilidad y borrado de forma
inspeccionable dentro de memoria/diagnóstico actuales. No importar un nuevo
escritorio, ni llamar «olvidada» a una memoria sin contrato backend de retención.

### Ledger de lectura efectiva

| Ref. | Archivo/rango | Evidencia |
|---|---|---|
| S0 | `package.json:1-80` | monorepo/scripts con efectos |
| S1 | `packages/tools/src/shared/memory-client.ts:1-230` | fan-out/dedup/render/logging |
| S2 | `packages/tools/src/shared/cache.ts:1-85` | clave y LRU |
| S3 | `packages/tools/src/vercel/middleware.ts:311-379` | continuación/deadline |
| S4 | `packages/docs-test/run.ts:1-154` | runner local |
| S5 | `packages/docs-test/tests/typescript/quickstart.ts:1-60` | ejemplo real y efectos |
| S6 | `apps/docs/memorybench/extend-benchmark.mdx:1-81` | documentación, no ejecución |
| S7 | `apps/web/app/page.tsx:1-60` | redirección |
| S8 | `packages/memory-graph/src/hooks/use-graph-data.ts:1-165` | presentación/grafo |
| S9 | `LICENSE:1-21` | MIT |

También README `1-75`, AGENTS/CLAUDE y búsqueda acotada de archivos benchmark.
No cubiertos: auth MCP, SDKs externos, ruta completa de captura/streaming,
backend de extracción/índices, MemoryBench externo y consola alojada.
Pendientes: conformance API, costes, readiness, invalidación y aislamiento
en ejecución. No tests/builds/apps/installs, servicios, red, benchmarks,
validadores documentales, commits ni push.

[S0]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/package.json#L1-L80
[S1]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/shared/memory-client.ts#L1-L230
[S2]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/shared/cache.ts#L1-L85
[S3]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/tools/src/vercel/middleware.ts#L311-L379
[S4]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/docs-test/run.ts#L1-L154
[S5]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/docs-test/tests/typescript/quickstart.ts#L1-L60
[S6]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/docs/memorybench/extend-benchmark.mdx#L1-L81
[S7]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/apps/web/app/page.tsx#L1-L60
[S8]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/packages/memory-graph/src/hooks/use-graph-data.ts#L1-L165
[S9]: https://github.com/supermemoryai/supermemory/blob/16ed414e0e0c61c946b7687ec813b2064fe75dd4/LICENSE#L1-L21
