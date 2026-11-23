---
title: "Backlog de extracción — Understand Anything"
kind: research
status: active
description: "Seis fichas del informe Understand Anything (importación C12, freshness, vista de contexto, guías derivadas, mapping de IDs e identidad de fuente) como filas del esquema canónico de 10 columnas"
tags: [extraccion, backlog, understand-anything]
---

# Backlog de extracción — Understand Anything

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/understand-anything.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `X-UND-01` | 🟠 | Importación de evidencia de código en dos capas bajo C12 | `src/sdk/api/graph.rs` · `src/sdk/types/record.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: importación única vía el contrato C12 `CodeEvidenceImport` que separa estructura verificable de explicación enriquecida; las aristas sin origen acreditado quedan `Unknown`, nunca `Ast`. · Por qué: el export mezcla estructura y narrativa del modelo; los pesos fijos de relación no son probabilidad de verdad y un resumen no puede pasar por relación determinista. · Acción mínima: validar schema, conservar original y problemas de saneamiento, mapear IDs namespaced y publicar relaciones con origen conocido o estado no acreditado. · Trazabilidad: informe §2/Ficha A · evidencia `understand-anything-plugin/packages/core/src/analyzer/graph-builder.ts:60-207`, `understand-anything-plugin/packages/core/src/types.ts:61-122` (clon `C:\Users\Eros\Understand-Anything`). | X-UND-02 · X-UND-05 · C12 | `MemoryInput` struct (`confidence_class`/`derived_from`), `add_edge`, namespace por repo/productor, revisión inmutable; revalidar FIND-299 antes de spec |
| `X-UND-02` | 🟡 | Freshness de evidencia de código con estado unknown explícito | `vantadb-mcp/src/code_index.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: snapshot por productor con revisión, digest de worktree y estado `fresh`/`dirty`/`stale`/`unknown`, más reconciliación de bajas y relaciones entrantes; la falta de evidencia nunca se declara fresh. · Por qué: una firma idéntica con cuerpo distinto puede cambiar la semántica; «cambio cosmético» estructural no basta para mantener un resumen como vigente ni un merge conserva los derivados de archivos no modificados. · Acción mínima: validar hash de los bytes referenciados, invalidar la explicación cuando cambió la evidencia y publicar nueva generación sin mezclar revisiones. · Trazabilidad: informe §2/Ficha B · evidencia `understand-anything-plugin/packages/core/src/fingerprint.ts:80-110,142-250`, `understand-anything-plugin/packages/core/src/staleness.ts:1-110,472-508`. | X-UND-01 · X-UND-04 | Manifiesto/hash por archivo e ingestors existentes; jobs/checkpoints propios, no watcher paralelo |
| `X-UND-03` | 🟡 | Vista de contexto de código acotada por evidencia y presupuesto | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: slice de grafo por repo/revisión con relaciones, evidencia enlazada, omisiones declaradas y navegación autorizada a la fuente dentro de la raíz permitida. · Por qué: limitar seeds y expandir después no limita los vecinos de un hub; la expansión debe responder al presupuesto global y las rutas físicas requieren validación real frente a symlink, no sólo léxica. · Acción mínima: recuperar seeds autorizadas, expandir dentro del presupuesto compartido, devolver límites explícitos y abrir fuente sólo en raíz permitida. · Trazabilidad: informe §2/Ficha C · evidencia `understand-anything-plugin/src/context-builder.ts:25-76`, `understand-anything-plugin/src/diff-analyzer.ts:22-87`, `understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:97-239`, `understand-anything-plugin/packages/dashboard/vite.config.ts:15-83,125-184`. | X-UND-01 · X-UND-02 · C16 | Context assembly con presupuesto compartido, autorización de repo y Studio existente; un salto no es cierre transitivo ni selección de tests |
| `X-UND-04` | 🟡 | Guía de código con capas/tours como derivado invalidable | `spec` (render de guía; materialización vía `src/sdk/types/record.rs`) | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: guía renderizada desde el grafo cuyas referencias faltantes o stale se exponen; si se materializa usa `MemoryInput` con `confidence_class=Derived` y `derived_from` con padres válidos del mismo namespace, sin confidence explícita. · Por qué: los tours heredan IDs que pueden desaparecer del texto sin aviso; una guía legible puede ocultar referencias faltantes y no puede ser su propia fuente. · Acción mínima: verificar dependencias de nodos, proyectar capas/tour con presupuesto y renderizar refs/estado; guía efímera como alternativa mínima. · Trazabilidad: informe §12.2/Ficha D · evidencia `understand-anything-plugin/src/onboard-builder.ts:7-123`, `understand-anything-plugin/packages/core/src/types.ts:61-122`, `understand-anything-plugin/packages/core/src/staleness.ts:472-508`. | X-UND-01 · X-UND-02 · X-UND-05 | Writer de derivados existente; lifecycle de X-UND-02 acordado antes de persistir; rechazo de padres/clase no se presenta como guía publicada |
| `X-UND-05` | 🟡 | Mapping explícito productor→interno sin reparación silenciosa | `vantadb-mcp/src/code_index.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: mapping que conserva ID original, tipo, ruta, revisión y alias justificado; rechazos `UnknownType`/`Collision`/`DanglingRef` no son éxito y nunca se inventa un endpoint ausente. · Por qué: la normalización upstream puede crear entidades ajenas (p. ej. doble prefijo `file:script:…` del issue #758); un ID saneado no acredita identidad estable ni proveniencia AST. · Acción mínima: validar el export original, construir el mapping una vez y comprobar endpoints/layers/tours antes de publicar; round-trip detectable. · Trazabilidad: informe §12.3/Ficha E · evidencia `understand-anything-plugin/skills/understand/merge-batch-graphs.py:233-276`, `understand-anything-plugin/packages/core/src/schema.ts:440-499`, issue Egonex-AI/Understand-Anything#758. | X-UND-01 · X-UND-02 | Namespace productor y schema acordado; legacy sin mapping queda unknown; dos writers no comparten mapping mutable |
| `X-UND-06` | 🟠 | Resolución autorizada de fuente separada del destino de artefactos | `vantadb-mcp/src/code_index.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: resolución de raíz física autorizada con identidad declarada (repo, revisión, digest) fijada por trabajo; el destino de staging jamás cambia los bytes analizados. · Por qué: reasignar la raíz al checkout principal puede servir evidencia de otra rama (issue #739); un cambio de rama durante la lectura produce conflicto/unknown, no evidencia fresh. · Acción mínima: autorizar la raíz física, registrar revisión/digest de esa fuente y comprobar identidad antes de importar; export manual con fuente explícita como alternativa mínima. · Trazabilidad: informe §12.4/Ficha F · evidencia `understand-anything-plugin/skills/understand/SKILL.md:47-73`, `understand-anything-plugin/packages/mcp/bin/understand-anything-mcp.mjs:54-139`, issue Egonex-AI/Understand-Anything#739. | X-UND-01 · X-UND-02 | `index_repo` con root/namespace y C12; autorización física frente a symlink; no crear ni destruir worktrees desde el adaptador |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/understand-anything.md` (Fichas A–F). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `a6ea402` (`C:\Users\Eros\Understand-Anything`); las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD. El grafo upstream mezcla estructura verificable y narrativa del modelo: los pesos de arista no son probabilidades de verdad.

### X-UND-01 — Importación de evidencia de código en dos capas bajo C12

- **Problema:** el export mezcla estructura verificable y narrativa del modelo; los pesos fijos de relación no son probabilidad de verdad y un resumen no puede pasar por relación determinista.
- **Contrato hipotético:** un único `CodeEvidenceImport` (C12 de CONSOLIDATED); layers/tours son derivados asociados a sus referencias, no otro `CodeExplanationImport` ni otra memoria; una arista sin origen acreditado → `Unknown`, nunca `Ast`; el peso queda reportado, no confianza canónica.
- **Fuente de evidencia (clone):** `understand-anything-plugin/packages/core/src/analyzer/graph-builder.ts:60-207` (`addFileWithAnalysis`, `addImportEdge`, `addCallEdge`), `understand-anything-plugin/packages/core/src/types.ts:61-122` (`KnowledgeGraph`/`GraphEdge`: tipo, dirección, descripción y weight — sin procedencia AST/LLM ni span probatorio).
- **Seam VantaDB:** `src/sdk/types/record.rs` (`MemoryInput`, con `confidence_class`/`derived_from`), `src/sdk/api/graph.rs` (`Embedded::add_edge` — la arista no admite metadata arbitraria: usar evidencia asociada); wiki/scenes existentes.
- **Acciones:**
  1. Validar JSON/schema del export.
  2. Conservar el original y los problemas de saneamiento.
  3. Mapear IDs namespaced (X-UND-05).
  4. Contrastar rutas/spans contra la revisión permitida.
  5. Persistir la explicación como derivada de evidencia.
  6. Publicar relaciones con origen conocido o estado no acreditado.
- **Dependencias:** `MemoryInput`, `add_edge`, namespace por repo/productor y revisión inmutable existentes; X-UND-02, X-UND-05, C12; revalidar FIND-299 antes de spec.
- **FMEA:** `autoFixGraph` vuelve aceptable el formato, no verdadera la afirmación; homónimos/overloads colisionan si sólo hay nombre/archivo; un move cambia el ID. No enlazar por nombre corto ni usar resúmenes como prueba circular.
- **Criterios de aceptación propuestos:** importación idempotente; arista sin fuente no se muestra como AST; schema desconocido no produce escrituras parciales; IDs originales se conservan para auditoría; scopes de repo no se mezclan.
- **Criterio negativo:** una arista sin origen acreditado nunca se publica como `Ast`; un formato «reparado» nunca convierte una afirmación en verdadera.
- **Independencia:** X-UND-01/X-UND-02/X-UND-06 antes de la importación persistente; adapter conservando `Unknown` preferido; CAS de generación y evidencia primero; rollback a la generación previa, nunca delete de memorias humanas.
- **No objetivos:** portar Zod/tree-sitter/el pipeline de subagentes ni copiar los pesos fijos al sistema de confianza de Vanta.

### X-UND-02 — Freshness de evidencia de código con estado `unknown` explícito

- **Problema:** una firma idéntica con cuerpo distinto puede cambiar la semántica; «cambio cosmético» estructural no basta para mantener un resumen vigente; la falta de evidencia nunca se declara fresh.
- **Contrato hipotético:** `CodeSnapshotState { revision, head_seen, generation, worktree_digest, schema_version, changed_files, state, reason }` con `state` ∈ `fresh|dirty|stale|unknown` (+ ahead/behind/diverged); `SemanticInvalidation { evidence_keys, changed_paths, producer_version, model_prompt_fingerprint }`; `unknown` distinto de `stale`.
- **Fuente de evidencia (clone):** `understand-anything-plugin/packages/core/src/fingerprint.ts:80-110,142-250` (`contentHash`, `compareFingerprints`), `understand-anything-plugin/packages/core/src/staleness.ts:1-110,472-508` (`GraphFreshnessResult`, `mergeGraphUpdate` — conserva el resto del objeto: no acredita regeneración de todas las derivaciones ni cierre transitivo de invalidación).
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs` (manifiesto/hash por archivo); procedencia de ingestors y FIND-299; jobs/checkpoints propios, no watcher paralelo.
- **Acciones:**
  1. Recibir el snapshot del productor.
  2. Validar el hash de los bytes referenciados.
  3. Invalidar la explicación si cambió la evidencia.
  4. Reconciliar bajas y relaciones entrantes.
  5. Reconstruir derivaciones/layers/tours afectados.
  6. Publicar la nueva generación sin mezclar revisiones.
- **Dependencias:** manifiesto/hash por archivo e ingestors existentes; jobs/checkpoints; X-UND-01, X-UND-04.
- **FMEA:** firma idéntica y cuerpo distinto cambia la semántica; «COSMETIC» estructural no basta para el resumen LLM; borrar nodos puede borrar aristas entrantes que el delta no repone; layers/tour retienen IDs antiguos.
- **Criterios de aceptación propuestos:** cambios de cuerpo invalidan la evidencia semántica aunque no cambie la firma; commit no disponible produce `unknown`; delete/move no deja navegación a IDs inexistentes; un delta incompleto no se anuncia como snapshot completo.
- **Criterio negativo:** la falta de evidencia nunca se declara `fresh`; un delta incompleto nunca se anuncia como snapshot completo.
- **Independencia:** hash de bytes + productor/revisión evita equiparar cosmético con igualdad semántica; la migración sin owner sólo diagnostica; un fallo parcial conserva lo anterior como stale/unknown.
- **No objetivos:** copiar las heurísticas upstream de clasificación como política de seguridad ni garantía de no regresión funcional.

### X-UND-03 — Vista de contexto de código acotada por evidencia y presupuesto

- **Problema:** limitar las seeds y expandir después no limita los vecinos de un hub; la expansión debe responder al presupuesto global y las rutas físicas requieren validación real frente a symlink, no sólo léxica.
- **Contrato hipotético:** `CodeViewSlice { repo_id, revision, nodes, relations, layers, evidence_links, omitted, freshness, navigation_targets }`; referencias de fuente `{path, span, revision}`, nunca rutas absolutas abiertas.
- **Fuente de evidencia (clone):** `understand-anything-plugin/src/context-builder.ts:25-76` (`buildChatContext`: expande un salto bidireccional tras limitar seeds — no cierre transitivo de impacto), `understand-anything-plugin/src/diff-analyzer.ts:22-87` (`buildDiffContext`: vecinos de un salto), `understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:97-239`, `understand-anything-plugin/packages/dashboard/vite.config.ts:15-83,125-184` (`readSourceFile`: ruta local/allowlist/tamaño/no binario; stat/read no acredita defensa física completa frente a symlink).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` (context assembly con presupuesto compartido) y Studio en `develop-desktop`.
- **Acciones:**
  1. Autorizar el repo antes de leer.
  2. Recuperar las seeds autorizadas.
  3. Expandir dentro del presupuesto compartido.
  4. Devolver límites explícitos de omisión.
  5. Abrir la fuente sólo en la raíz permitida.
- **Dependencias:** context assembly con presupuesto, autorización de repo y Studio existentes; contrato de API/UI; X-UND-01, X-UND-02, C16; no empaquetar servidor.
- **FMEA:** el límite de seeds no limita vecinos de un hub; el JSON pesa más que la selección inicial; una ruta symlink puede escapar aunque sea léxicamente local (limitar la salida total y verificar la ruta física).
- **Criterios de aceptación propuestos:** un hub truncado declara omisiones; seleccionar un nodo muestra evidencia, revisión y estado; token inválido/ruta fuera de raíz se deniegan; no usar vecinos de diff para excluir tests.
- **Criterio negativo:** una expansión truncada nunca se presenta sin omisiones declaradas; un salto nunca es cierre transitivo ni selección de tests.
- **Independencia:** consulta ligada a generación y budget final; sin commit; la cancelación da `incomplete`/descarta la respuesta vieja; desactivar el panel conserva la evidencia.
- **No objetivos:** migrar toda la UI, emitir prompts de evaluación ni diseñar selección de tests; tours/layers son ayudas de comprensión, no ground truth.

### X-UND-04 — Guía de código con capas/tours como derivado invalidable

- **Problema:** onboarding de un agente nuevo sin cargar todo el repo; los tours heredan IDs que pueden desaparecer del texto sin aviso; una guía legible puede ocultar referencias faltantes y no puede ser su propia fuente.
- **Contrato hipotético:** `render_code_guide(generation, authorized_refs, budget) -> Result<GuideView, GuideError>`; referencias faltantes/stale se exponen; si se materializa usa `MemoryInput` con `confidence_class: Some(ConfidenceClass::Derived)` y `derived_from` con padres válidos del mismo namespace, sin confidence explícita.
- **Fuente de evidencia (clone):** `understand-anything-plugin/src/onboard-builder.ts:7-123`, `understand-anything-plugin/packages/core/src/types.ts:61-122` (layers/tour), `understand-anything-plugin/packages/core/src/staleness.ts:472-508` (merge).
- **Seam VantaDB:** spec (render de guía; materialización vía `src/sdk/types/record.rs`); escenas/wiki/context assembly existentes.
- **Acciones:**
  1. Mapear nodos de la generación autorizada.
  2. Verificar dependencias.
  3. Proyectar layers/tour.
  4. Aplicar el budget global.
  5. Renderizar refs/estado. Alternativa mínima: guía efímera; persistencia sólo si el lifecycle X-UND-02 está acordado.
- **Dependencias:** X-UND-01/X-UND-02/X-UND-05; writer de derivados existente; invalidar antes de declarar vigente; la cancelación no publica una guía parcial como completa; el rollback elimina la proyección propia, no las fuentes.
- **FMEA:** un ID borrado queda en el tour → detectar la referencia ausente y advertirla; renumerar una layer parece identidad estable → digest de miembros; una explicación hostil instruye al agente → contenido citado, nunca autoridad de herramientas; materializar con clase/linaje inválidos no se subsana inventando un enum o confidence.
- **Criterios de aceptación propuestos:** delete/rename/body-change invalida la guía; faltantes visibles; ciclos de derivación rechazados; observar conteos/estado sin resúmenes; el rechazo de padres/clase no se presenta como guía vacía ni publicación completa.
- **Criterio negativo:** una guía con referencias faltantes/stale nunca se presenta como completa; el rechazo del writer nunca se subsana inventando enum/confidence.
- **Independencia:** X-UND-03/X-UND-04 después para contexto/enseñanza; la guía efímera es alternativa mínima sin persistencia.
- **No objetivos:** aprender la arquitectura verdadera por clasificación de complejidad, copiar prompts de agentes ni introducir evaluator/umbral pedagógico.

### X-UND-05 — Mapping explícito productor→interno sin reparación silenciosa

- **Problema:** el agente pide un símbolo cuyo ID fue «corregido» y obtiene vacío; la normalización upstream puede crear entidades ajenas (doble prefijo `file:script:…` del issue #758); un ID saneado no acredita identidad estable ni proveniencia AST.
- **Contrato hipotético:** `map_producer_refs(&CodeEvidenceImport, mapping, schema) -> Result<MappingReport, ImportError>`; conserva original, tipo, ruta, revisión, alias justificado y rechazos; `UnknownType | Collision | DanglingRef` no son éxito; nunca se inventa un endpoint ausente.
- **Fuente de evidencia (clone):** `understand-anything-plugin/skills/understand/merge-batch-graphs.py:233-276` (`normalize_node_id`), `understand-anything-plugin/packages/core/src/schema.ts:440-499`, issue Egonex-AI/Understand-Anything#758.
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs` (claves deterministas `file:`/`sym:` existentes; no reinterpretar prefijos externos como claves internas).
- **Acciones:**
  1. Validar el export original.
  2. Construir el mapping una vez.
  3. Comprobar endpoints/layers/tours.
  4. Importar X-UND-01.
  5. Publicar X-UND-02. El rechazo explícito es alternativa a corregir ambiguamente.
- **Dependencias:** X-UND-01/X-UND-02; namespace productor y schema acordado; el legado sin mapping queda unknown; dos writers no comparten mapping mutable.
- **FMEA:** un doble prefijo crea una entidad ajena → colisión/round-trip detectable; endpoint ausente → rechazar la relación sin inventarlo.
- **Criterios de aceptación propuestos:** `script`, `config`, homónimos y reimport mantienen el mapping estable o un error trazable; métricas de remapeos/rechazos sin rutas sensibles.
- **Criterio negativo:** `UnknownType`/`Collision`/`DanglingRef` nunca son éxito; ningún endpoint ausente se inventa para completar una relación.
- **Independencia:** X-UND-05 antes de confiar en IDs saneados; cancelar descarta el plan, el rollback usa el anterior.
- **No objetivos:** copiar el reparador Python completo ni deducir proveniencia AST desde un ID bien formado.

### X-UND-06 — Resolución autorizada de fuente separada del destino de artefactos

- **Problema:** reasignar la raíz al checkout principal puede servir evidencia de otra rama (issue #739); un cambio de rama durante la lectura produce conflicto/unknown, no evidencia fresh.
- **Contrato hipotético:** `resolve_code_source(requested_root, declared_identity, auth) -> Result<AuthorizedSource, SourceError>`; el destino de staging es un control separado, nunca motivo para cambiar los bytes analizados; tipo de aplicación, no segundo DTO de importación.
- **Fuente de evidencia (clone):** `understand-anything-plugin/skills/understand/SKILL.md:47-73`, `understand-anything-plugin/packages/mcp/bin/understand-anything-mcp.mjs:54-139` (`resolveProjectRoot`), issue Egonex-AI/Understand-Anything#739.
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs` (`index_repo` con root/namespace y C12).
- **Acciones:**
  1. Autorizar la raíz física.
  2. Registrar revisión/digest de esa fuente.
  3. Leer el export.
  4. Comprobar la identidad.
  5. Importar X-UND-01/X-UND-02. Mínimo: export manual con fuente explícita; rechazar auto-redirect silencioso.
- **Dependencias:** `index_repo` con root/namespace y C12; autorización física frente a symlink; X-UND-01/X-UND-02; no crear ni destruir worktrees desde el adaptador.
- **FMEA:** la raíz principal confunde repo común con revisión → detectar fuente/commit discrepante; symlink escapa → autorización física.
- **Criterios de aceptación propuestos:** branch-only+dirty+dos worktrees conserva diferencias; observar el mismatch sin publicar roots secretos; un cambio de rama durante la lectura produce conflicto/unknown.
- **Criterio negativo:** el destino de staging nunca cambia los bytes analizados; una fuente discrepante nunca se sirve como evidencia fresh.
- **Independencia:** X-UND-01/X-UND-02/X-UND-06 antes de la importación persistente; depende de X-UND-01/X-UND-02, no de instalar hooks.
- **No objetivos:** no crear ni destruir worktrees desde el adaptador; no generalizar el fallo de una instrucción a todas las APIs.

<!-- audit: 2026-11-23 normalizadas referencias residuales A/B -> X-UND-01/X-UND-02 en fichas X-UND-04 y X-UND-06 -->
