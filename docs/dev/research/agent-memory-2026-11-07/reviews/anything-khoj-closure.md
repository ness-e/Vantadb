---
title: "Cierre propuesto de revisión — AnythingLLM y Khoj"
kind: research
status: proposed
description: "Tercer contraste independiente postcorrecciones: dos ledgers, evidencia reutilizada, fuentes comprobadas y un residual documental de alcance por operación"
tags: [research, memory, engineering]
---

# Cierre propuesto de revisión — AnythingLLM y Khoj

## Índice

- [Mandato y evidencia previa](#mandato-y-evidencia-previa)
- [AnythingLLM: recibo durable](#anythingllm-recibo-durable)
- [Khoj: recibo durable](#khoj-recibo-durable)
- [Control de alcance y entrega](#control-de-alcance-y-entrega)

## Mandato y evidencia previa

Tercer revisor de **exactamente tres** asignados; contraste documental independiente
de la autoría, realizado el 2026-11-07. No se convocaron otros revisores ni agentes.
Antes de planificar se comprobó que este archivo de cierre no existía. El contraste
partió de las correcciones presentes, sin asumir que los hallazgos siguieran abiertos.

**Lectura completa reconocida:** [revisión independiente previa](anything-khoj.md),
L1-364, incluidos ambos ledgers, recibos locales/públicos, hallazgos y limitaciones.
No se reescribe ese antecedente ni se vuelven a recorrer los tres flujos completos
ya acreditados por proyecto. Se reutiliza explícitamente su evidencia inalterada;
las comprobaciones nuevas sobre cambios están separadas a continuación.
También se leyeron íntegros los informes actuales: AnythingLLM L1-632 y Khoj L1-757.

Contratos leídos: [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md) y
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md). Se consultó el catálogo de skills y se
cargó `documentation-skill`; no se activó un flujo de implementación/certificación.
«Sustentado» significa cobertura documental dentro de los límites acreditados,
no ejecución, aprobación del diseño ni aceptación final.

`git rev-parse HEAD` confirmó las cuatro identidades:

| Árbol | SHA comprobado |
|---|---|
| `C:\Users\Eros\anything-llm` | `308b27662b2f6fd4947791d61c797945c609fcbc` (A) |
| `C:\Users\Eros\khoj` | `ae229ca894c0b80ad84664afcfdde523b5e87057` (K) |
| VantaDB, workspace asignado | `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` |
| `C:\Users\Eros\VantaDB Proyect\develop-desktop` | `0ad38c542b39a589035b0e17fdedb4c09dca24b9` |

`git status --short -- <rutas>` no mostró cambios en las fuentes upstream
releídas aquí. Esto no certifica limpieza global ni revalida todo Vanta/Studio.
Los recibos públicos previos no se consultaron otra vez: no cambió el material
objeto de esas conclusiones y no se afirma un estado remoto más reciente.

## AnythingLLM: recibo durable

**Informe:** [anything-llm.md](../anything-llm.md). Evidencia anterior reutilizada:
[revisión previa](anything-khoj.md), ledger L57-73, fuentes locales L97-135,
recibos públicos L137-152 y límites/veredicto L154-162.
Se mantienen la distinción SPA/shell Desktop, los gitlinks no materializados,
la compensación acotada y los commits separados de ingesta/proyecciones.

### Ledger 0 + diez ejes de AnythingLLM

| Eje | Estado propuesto | Evidencia y límite |
|---|---|---|
| 0. Arquitectura y flujos | sustentado | Informe §1; revisión previa L63,101-124 acredita ingesta, consulta y update/delete. No se repiten los recorridos ni se amplía atomicidad. |
| 1. Recomendaciones | sustentado | Informe §2, L185-200; A–G conservan prioridades y deltas concretos, no otro motor/collector/scheduler. Antecedente L64. |
| 2. Fichas Rust | sustentado | Informe §3, L202-424; once campos agrupados, alternativas, seams, FMEA y criterios futuros. Antecedente L65. Contratos siguen siendo propuestas. |
| 3. Incidencias | sustentado | Informe §4, L426-439; #3958/#6806 y límites contrastados previamente L141-147. Cierre/PR no equivalen a corrección universal. |
| 4. Prerrequisitos Vanta | sustentado | Informe §5, L441-453; capacidades existentes y dependencias por candidato. Fuentes Vanta/Studio acreditadas antes L130-135. |
| 5. Rechazos | sustentado | Informe §6, L455-464 y no-goals A–G; no adopción de bypass, producto completo o refresh destructivo como garantía. Antecedente L68. |
| 6. Advertencias | sustentado | Informe §7, L466-477; shallow, lectura estática, Desktop documental y licencias por componente conservados. Antecedente L69. |
| 7. Benchmarks | sustentado | Informe §8, L479-512; Jest/mock no se convierte en QA, recall o coste medido. Rango completo L164 comprobado de nuevo abajo. |
| 8. Matriz | sustentado | Informe §9, L514-531; reconoce BM25/HNSW/RRF, filtros, worker, dedup y Studio. Deltas condicionados, no carencias globales. Antecedente L71. |
| 9. Negativos Vanta | sustentado | Informe §10, L533-541; preguntas discriminantes y `core_search=false`, no bugs por analogía. Evidencia anterior L72,130-135. |
| 10. UI/escritorio | sustentado | Informe §11, L543-570; fuente SPA y producto Desktop separados. Hook fijado comprobado abajo; shell/IPC y ejecución siguen fuera de cobertura. |

### Contraste propio de la corrección menor

- **Test sync:** informe L484 y definición `[sync-test]` L597 llegan a L164.
  Releído localmente `server/__tests__/jobs/sync-watched-documents.test.js:120-164`,
  SHA A; [fuente fijada][a-test]. L155-157 ejecuta el segundo run y comprueba que
  no reintenta; L158-164 completa la expectativa `"exited"` y cierra el caso.
  El enlace ya permite comprobar la afirmación sin reconstruir el rango.
- **Hook picker:** informe L400-402 enlaza `[picker-hook]`, definido L598 con
  ruta completa y SHA A, L643-L689. Releído ese rango de
  `frontend/src/components/Modals/ManageWorkspace/Documents/hooks/useDocumentPicker.js`;
  [fuente fijada][a-picker]. Confirma carga `NO_LIMIT` cuando falta contenido,
  deduplicación y exclusión de documentos ya asociados; no prueba snapshot
  concurrente ni ejecución UI. Coincide con el límite de §11.

**Hallazgos residuales:** ninguno en las afirmaciones/correcciones contrastadas.
**Dictamen propuesto:** corrección menor cerrable; sin hallazgos documentales
bloqueantes en este alcance. **No aceptación final** ni autorización para A–G.
Se conservan las lagunas anteriores: providers/transportes completos, shell,
gitlinks, auditoría legal transitiva, crash, calidad, coste y rendimiento.

## Khoj: recibo durable

**Informe:** [khoj.md](../khoj.md). Evidencia anterior reutilizada:
[revisión previa](anything-khoj.md), ledger L176-190, hallazgos L192-247,
fuentes locales L249-303, recibos públicos L305-321 y límites L323-332.
No se repiten indexación→consulta→update/delete completos, ni se convierte la
lectura previa de issues en reproducción, merge verificado o auditoría Cloud.

### Ledger 0 + diez ejes de Khoj

| Eje | Estado propuesto | Evidencia y límite |
|---|---|---|
| 0. Arquitectura y flujos | sustentado | Informe §1, L35-177; antecedente L180,253-279. La edición sin agent y su efecto ahora constan en L161-177 y D L303-315; fuente releída abajo. |
| 1. Recomendaciones | sustentado | Informe §2, L179-193; siete mecanismos priorizados y dependencias específicas, sin trasladar servicios al motor. Antecedente L181. |
| 2. Fichas Rust | parcial | Informe §3, L195-440; A–G preservan campos y propuestas no implementadas. D incorpora fuente/efecto, pero FMEA y criterios L327-341 no trasladan el cambio de asociación; residual único abajo. |
| 3. Incidencias | sustentado | Informe §4, L442-458; #1437/#1363/#759 conservan estado histórico y límites, acreditados antes L309-321. No nueva lectura pública ni resultado propio. |
| 4. Prerrequisitos Vanta | sustentado | Informe §5, L460-476; worker/progreso/run_id, scheduler, L1 y contexto existentes. Fuentes previas L299-303 y referencia común; no bloqueo universal. |
| 5. Rechazos | sustentado | Informe §6, L478-489 y no-goals; no producto Python, defaults/ranking, vacío-delete ni clientes como contratos universales. Antecedente L185. |
| 6. Advertencias | sustentado | Informe §7, L491-504; PostgreSQL, UserMemory, runner real, settings con efectos y tests no ejecutados. Antecedente L186,253-259. |
| 7. Benchmarks | sustentado | Informe §8, L506-601; las dos correcciones se contrastaron contra eval.py. No inversión de funciones/rangos, puntuación cero inventada ni conteo de dataset; detalle abajo. |
| 8. Matriz | parcial | Informe §9, L603-621; motores/benchmarks y capacidades Vanta conservados. Tenancy L616 sigue agregando user/agent sin distinguir CRUD por user/id; mismo residual. |
| 9. Negativos Vanta | sustentado | Informe §10, L623-633; riesgos condicionados, checkpoint distinto de WAL y postfiltro distinto de pushdown. Antecedente L189,299-303. |
| 10. UI/escritorio | parcial | Informe §11, L635-675; Next/Electron/plugin/Emacs/TWA siguen acreditados antes L289-298. La fila settings L643 omite la pérdida de asociación al editar; mismo residual, no fallo nuevo de UI. |

### Contraste propio: asociación y alcance por operación

Releídos localmente, SHA K, sin importar módulos:

- `src/khoj/routers/api_memories.py:18-114`: GET lista por user (L27);
  DELETE verifica user/id (L54); PUT verifica user/id (L83), elimina L97 y
  [llama save_memory sin agent, L100-103][k-api].
- `src/khoj/database/adapters/__init__.py:2292-2378`:
  [pull/search, save y delete][k-adapters]. Lectura no default exige user+agent
  (L2301-2306 y L2342-2345); default/sin agente consulta memorias del usuario,
  sujeto a ventana/límite o distancia. Save tiene `agent=None` L2311 y crea
  sin asociación L2323-2326 en esa rama. Delete L2357-2363 exige user/id, no agent.
- `src/khoj/routers/helpers.py:1038-1067`: [caller automático][k-auto] pasa agent
  al create L1062, pero no al delete L1066.
- `src/interface/web/app/settings/page.tsx:660-730`: [UI settings][k-ui] hace
  PUT por ID y envía raw/memory_id, no agent (L694-707); reemplaza el elemento
  tras respuesta correcta. No añade por sí misma un contrato de asociación.

**Resultado positivo:** informe L161-177 y D L303-315 ya describen correctamente
la omisión y la asimetría. Si el reemplazo tiene éxito, una memoria de agente no
default queda sin asociación y sale de su recall específico, aunque siga siendo
del mismo usuario. No demuestra fuga entre usuarios: los filtros user permanecen.
Tampoco demuestra que un LLM haya logrado borrar una memoria de otro agente.

### Hallazgo residual único — media, coherencia documental de D/matriz/UI

La evidencia nueva fue añadida al relato de fuente, pero no a las consecuencias
accionables y resúmenes afectados:

1. **D, informe L327-341:** pasos hablan de propiedad de IDs; FMEA enumera
   NoFacts/disable y criterios «ID ajeno». No hay disparador→consecuencia→detección/
   mitigación para un edit exitoso que pierde asociación, ni criterio que distinga
   otro usuario de otra asociación del mismo usuario. Ese caso no queda cubierto
   por error de embedding, CAS, disable ni ownership genérico.
2. **Matriz L616:** «Existe user/agent, default agent más amplio por usuario»
   conserva solo la síntesis de lectura; omite que edición/delete operan por
   user/id y que edición puede cambiar asociación.
3. **UI L643:** «Edit devuelve ID nuevo» no conserva la otra consecuencia ahora
   acreditada: el mismo edit puede retirar el hecho del recall de su agente.

**Impacto:** una futura tarea derivada de la ficha o de esos resúmenes podría
preservar contenido/ID/usuario sin conservar ni declarar el cambio de asociación.
Es cierre incompleto del primer hallazgo previo, no tres bugs nuevos.
**Corrección documental verificable propuesta:** propagar el alcance por operación
a FMEA/criterios de D, matriz y fila settings; explicitar conservación o cambio
intencional de asociación y distinguir IDs de otro usuario/otra asociación del
mismo usuario. No exige prohibir por principio CRUD user-wide ni diseñar un nuevo
sistema de permisos. No atribuir este comportamiento a Vanta ni inventar fuga.
No requiere reproducción runtime para resolver esta inconsistencia documental.

### Contraste propio: dos correcciones de benchmark

- **Default inválido:** releído `tests/evals/eval.py:34-183,393-431,623-707`.
  [L44-48][k-batch] calcula exactamente
  `BATCH_SIZE=int(os.getenv("BATCH_SIZE", int(SAMPLE_SIZE)/10 if SAMPLE_SIZE else 10))`.
  Sin override, `SAMPLE_SIZE` entero positivo 1..9 da 0; [range L664][k-main]
  tiene paso cero antes de emitir preguntas. Informe L537-544 lo distingue de
  calidad/recall cero, None/dropna y excepciones dentro de futures. Es deducción
  estática, no ejecución. Preparación/ingesta anterior puede tener efectos;
  no se concluye que el fallo ocurra antes de todo I/O.
- **Poblaciones y anclajes corregidos:** [get_articles_by_prompt_id L93-105][k-gold]
  filtra **solo prompt_id**. [index_frames_kb L152-157][k-index] filtra archivos
  en `missing_files` **y** texto no vacío. [L408-410][k-ir] usa `len(articles)`
  para recall sin ese filtro de texto. Informe L551-559 mantiene funciones y
  rangos en ese orden correcto: no reincide en la inversión del parche inicial.
  El gold **puede** incluir artículos no subidos por ese camino; no se afirma
  que falten de toda instancia preexistente. Ocurrencia y magnitud desconocidas:
  no se descargó dataset, no hubo conteos ni máximo de recall calculado.

**Dictamen propuesto:** dos hallazgos de benchmark cerrables; primer hallazgo
parcialmente corregido, con **cambios documentales requeridos** por el residual
único. **No aceptación final**. Se mantienen las lagunas ya declaradas de PR/historia,
datasets, invalidaciones completas, despliegues, permisos y runtime de clientes.
No se propone modificar fórmulas, configuración, scoring, harnesses ni prompts.

## Control de alcance y entrega

Único archivo creado/editado: este cierre. Lecturas mediante `Get-Content`,
`Get-ChildItem`, `Test-Path` y selección/numeración en PowerShell; Git solo lectura
(`rev-parse`, `status` scoped, `check-ignore`). Ninguna red nueva fue necesaria.
No aplicaciones, tests, builds, benchmarks, instalaciones, servicios ni acciones
de inicio/parada. Sin staging, commits, push, fetch, cambios de índices, contratos,
JSON, backlog, código ni informes. Los checks automatizados no se ejecutaron:
la entrega es un dictamen estático propuesto, no certificación.

`git check-ignore -v` confirma `.gitignore:268` para `docs/dev/research/`:
el recibo es durable en disco, pero Git normal no lo mostrará. Preservación/
incorporación futura y aceptación corresponden al responsable principal;
no se modifica ignore ni se fuerza seguimiento en esta revisión.

[a-test]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/__tests__/jobs/sync-watched-documents.test.js#L120-L164
[a-picker]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/components/Modals/ManageWorkspace/Documents/hooks/useDocumentPicker.js#L643-L689
[k-api]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api_memories.py#L18-L114
[k-adapters]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/adapters/__init__.py#L2292-L2378
[k-auto]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/helpers.py#L1038-L1067
[k-ui]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/web/app/settings/page.tsx#L660-L730
[k-batch]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L44-L48
[k-main]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L623-L684
[k-gold]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L93-L105
[k-index]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L152-L157
[k-ir]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L393-L431
