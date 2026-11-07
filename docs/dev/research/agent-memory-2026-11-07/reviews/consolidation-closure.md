---
title: "Cierre propuesto de revisión independiente de la consolidación"
kind: research
status: proposed
description: "Contraste final de CONSOLIDATED y BENCHMARK-NOTES con la autoridad congelada, cierres T7 y objetos Git V/S"
---

# Cierre propuesto de revisión independiente de la consolidación

## Dictamen propuesto

**Aceptar documentalmente ambos documentos con sus límites explícitos.**
No se detectó contradicción material nueva respecto de los resultados congelados
en las afirmaciones contrastadas. Se registra **una observación baja de
navegación**, no bloqueante, sin corregir los documentos del lead.

Esto cierra una **revisión de síntesis**, no otra auditoría de 42 productos.
Se conserva la autoridad del lead: **41 informes aceptados con límites y
Meilisearch entregado con restricción expresa, E3 incidencias no acreditado**.
No significa diez ejes completos por producto, propuestas aprobadas, tareas
activadas, specs implementables automáticamente ni certificación de runtime.
La aceptación final de esta revisión corresponde al lead.

## Alcance, identidad y fuentes

- Lectura íntegra de [CONSOLIDATED](../CONSOLIDATED.md), L1–322, y
  [BENCHMARK-NOTES](../BENCHMARK-NOTES.md), L1–216.
- Contraste documental con [REVIEW-COVERAGE](../REVIEW-COVERAGE.md), L19–28,
  L121–142; [VANTA-REFERENCE](../VANTA-REFERENCE.md), L13–52 y L56–97;
  [contrato](../EXPANSION-CONTRACT.md), L92–113 y L136–167;
  [METHOD](../METHOD.md), L120–169, como metodología autorada, no rediseñada.
- Cierres T7: [memoria/código](corrections-memory-closure.md),
  [motores/RAG](corrections-engines-closure.md) y
  [plataformas/frameworks](corrections-platforms-closure.md).
  Se contrastaron las disposiciones pertinentes con las fichas fuente; no se
  reabrieron issues, PRs ni validaciones dinámicas.
- **V** = `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
  **S** = `0ad38c542b39a589035b0e17fdedb4c09dca24b9`.
  Ambos objetos commit están disponibles localmente. S utiliza
  `feature-freez/desktop/...`, no bytes del worktree movido/modificado.
- `c561158` y los anclajes históricos/dirty permanecen históricos. Los `??`
  de jobs/JobChip no se convierten en contenido S. Las lecturas complementarias
  locales de otras fichas no se «congelan» reemplazando un prefijo de ruta.
- Las cifras y fórmulas se reciben de las fichas/reviews: **no se calcularon
  scores, denominadores nuevos, rankings ni resultados competitivos**.

Identidad SHA-256 de los dos documentos leídos, no firma ni prueba de sus claims:

| Documento | SHA-256 |
|---|---|
| CONSOLIDATED.md | `9600fae6d6832dc6fdeec1a161e42f132e10b9b0caff948c1713286540752b9b` |
| BENCHMARK-NOTES.md | `ad48a7ef8c9980ef63b26faf1e1da888cde253cad56a705b12814d402d82a300` |

## Matriz 0 + diez ejes — aplicada a la síntesis

«Sustentado» significa correspondencia documental dentro del alcance, no nueva
acreditación de capacidades. «Parcial» conserva la cobertura limitada de las
fuentes; no exige runtime universal. En las notas, N/A significa que ese eje
no es su función: no se les exige duplicar CONSOLIDATED ni las fichas Rust.

| Eje | CONSOLIDATED: estado y rango | BENCHMARK-NOTES: estado y rango |
|---|---|---|
| 0. Contexto/arquitectura | Sustentado: L11–54; V/S, capas y cinco adiciones discriminados | Sustentado: L13–26, L73–162, L183–216; identidad de protocolo y snapshots, no arquitectura de productos |
| 1. Recomendaciones | Sustentado: L31–50; prioridad cualitativa y mecanismos, no adopción de productos | Sustentado: L165–179, L201–207; límites para interpretar/comparar, no diseño nuevo |
| 2. Fichas Rust | Sustentado como consolidación: L54–194; C1–C16 propuestos y detalle delegado a fichas aceptadas | N/A: documento metodológico, no catálogo de candidatos |
| 3. Incidencias | Parcial explícito: L198–215; reportes/fix/pin separados, Meili E3 no acreditado | N/A: no dossier de incidencias; L63–67 preserva prohibición de forge |
| 4. Prerrequisitos Vanta | Sustentado: L219–230; dependencias por operación, no bloqueo global C1/C13 | Sustentado: L165–207; superficies existentes y aceptación futura, sin ejecución |
| 5. Rechazos | Sustentado: L234–236 y L46–50; no sistemas paralelos ni equivalencias falsas | Sustentado: L13–19, L111–113, L167–174; no tabla competitiva ni homologación de métricas |
| 6. Método/límites | Sustentado: L11–14, L240–244, L292–294; licencia, alcance y residual explícitos | Sustentado: L23–26, L69–71, L87–89, L209–216; fuentes derivadas, snapshots y no ejecución |
| 7. Benchmarks | Sustentado como síntesis: L65–72, L248 y L272; remisión a fichas/notas, C2/§7 del lead preservados | Sustentado: L28–67 y L73–207; población, errores y fórmulas conservados |
| 8. Matriz de capacidades | Sustentado: L252–265; «existe» estático, «no acreditado» no ausencia | N/A: contraste de métricas, no matriz funcional de productos |
| 9. Negativos propios | Sustentado/acotado: L269–273; hecho, hipótesis e histórico separados | Sustentado/acotado: L176–207; LoCoMo V estático, no invalidación de Paso 23 ni LongMemEval histórico |
| 10. Escritorio/UI | Parcial explícito: L277–288; componentes reales, SaaS/clientes externos y jobs propuestos separados | N/A: no inspección visual ni inventario de frontend |

No se agregan puntuaciones a esta matriz ni se convierte su estado en una
segunda aceptación por producto. La lista de enlaces de CONSOLIDATED incluye
las 42 fichas; se comprobó navegación, no cobertura por conteo de enlaces.

## Observación baja — anclas técnicas más estrechas que los símbolos enumerados

**Archivo/rango:** CONSOLIDATED L129 y L164; definiciones L299, L307 y L315.
Los archivos y pins son correctos, pero algunas selecciones no incluyen todos
los símbolos que se enumeran junto al enlace:

| Cita actual | Fuente discrepante comprobada mediante `git show V:<ruta>` | Corrección verificable propuesta |
|---|---|---|
| L164/L315: `WriteReceipt/build_write_receipt`, ancla `src/attestation.rs:678–745` | El rango contiene el tipo y límites; `build_write_receipt` está en L920–955, con explicación del builder L914–919 | Añadir ancla separada L914–955; mantener el ancla del tipo/límites |
| L129/L307: `begin/execute/ingest_body`, ancla `vanta-memory/src/ingest/worker.rs:109–309` | `begin` está en L70–88; `execute` empieza en L95. L109 inicia una expresión dentro de `execute` | Añadir L64–153 para begin/execute o ampliar el enlace desde L64 |
| L164/L299: `put_one/put_certified/put_batch_inner`, ancla `src/sdk/api/memory.rs:1029–1529` | `put_one` empieza en L864; el rango enlazado incluye solamente su tramo posterior | Añadir enlace específico al inicio de `put_one` o precisar que se cita su tramo posterior |

**Consecuencia:** obliga a navegar fuera de la selección para localizar el seam
nombrado. **No demuestra símbolos inexistentes, falsa garantía ni bug del
producto.** No altera C13, no reabre las correcciones del lead y no bloquea el
cierre. No se aplicó la corrección: sólo este recibo es editable.

## Contraste de las garantías y residuales sensibles

| Punto de síntesis | Resultado y fuente contrastada |
|---|---|
| C1, L56–63; negativos L269–271 | Correcto: L1 delete-before-put separado de helper RMW, cursor L0 y WAL. V `l1_writer.rs:389–456`, `checkpoint.rs:88–117`, `l0_recorder.rs:348–365,421–483`, `wal.rs:1004–1028`. No se inventa carrera demostrada ni crash/pérdida reproducidos. |
| C3/C4, L74–89 | Correcto: `RecallScope/RecallConfig/InjectionPolicy`, no `MemoryScope` implementado; identidad y revisión son propuestas. VANTA-REFERENCE L48–52 y T7 motores L417–429 preservan caller autenticado/hidratación Quivr no acreditados. |
| C5, L91–98 | Corrección del lead conservada: **struct** `MemoryInput`, V `src/sdk/types/record.rs:184–225`, `confidence_class/derived_from` y padres del mismo namespace. Ni esos campos ni Wiki/relaciones prueban CAS o commit conjunto. |
| C6, L100–107 | La consolidación es propuesta condicionada, no dedup desde cero. T7 memoria L223–267 y mapa L561 distinguen guards de fuentes Hindsight y tags actuales; no atribuye ausencia upstream ni reparación ya ejecutada. |
| C7, L109–116 | Presupuesto/compactación/filtros existen. V `engine.rs:197–218` conserva prefijo protegido sobre presupuesto; `vector.rs:264–300` revalida filtro exacto y sólo activa fallback por hits vacíos. No cobertura completa ni límite exacto de tokens inventados. |
| C8, L118–124 | Selected/Rendered/Delivered y confirmación del host son propuesta. Genlog/cursor no son delivery; la idempotencia local no equivale a exactly-once de red. T7 memoria L610–626 y ficha Letta L190–220 coinciden. |
| C9, L126–133 | V `WikiStore::complete/fail:185–214` usa run_id terminal; `put_page:287–317` no recibe run_id. T7 plataformas L82–105: no fencing por página ni rollback por cancelación, cola durable o publicación global acreditados. |
| C10/C11, L135–149 | Preservan privacidad/ownership y adapters existentes; aprobación de captura no autoriza tools. T7 plataformas L157–290 y T7 motores L246–305 separan conformidad por ruta/binding de capacidades del motor. |
| C12, L151–159 | T7 memoria L396–445, L568–587: ACK upstream aplicado no prueba caller code-index/ACK/recovery Vanta. E sigue integración nueva no acreditada; no segundo scheduler. L159 limita la UI CBM a imports/comentario, no navegación React acreditada. |
| C13, L161–169 | `WriteReceipt` real, hash/relectura, **no firma**. MutationReceipt/DTO nuevos son vistas propuestas; cuatro ejes independientes, progreso por ordinal, causa inicial/secundarias y terminal conservados, no Err tardío que oculte éxitos. V `attestation.rs:678–738,906–955`; fichas Weaviate L235–281 y pgvector L217–243. |
| C14/C15, L171–185 | Auditoría/rebuild/dimensión/capabilities ya existen. V `impl_index.rs:33–78` no demuestra identidad de conjuntos por conteos; se mantiene diagnóstico separado de repair. Modelo/dominio/codec y egress del host no se presentan como controles globales ya implementados. |
| C16, L187–194 y L277–288 | Se conservan tres paneles S reales, no UI jobs shipped ni cancel/restart durable. VANTA-REFERENCE L56–69 y T7 plataformas L525–530: presentación no concede permiso de reparar/restore ni acredita delivery. |

## Cinco adiciones — fuente, seam, delta, dependencias y rechazo

Las cinco tienen fila propia en CONSOLIDATED L46–50, no C17 ni incorporación
silenciosa al filtro original. Se contrastó su efecto en los candidatos, no
se auditó de nuevo su implementación upstream.

| Adición y fuente local | Seam y delta conservados | Dependencias/rechazo comprobados |
|---|---|---|
| [Weaviate](../weaviate.md), L96–147, L216–291, L626–655 | `put_batch_inner`, WAL sync, HNSW/derivados; C13 por ítem/terminal, C7/C15 soporte real, C4/C14 generación; C9 recursos separables y C16 auditoría/backup existentes | No WAL nuevo, RAFT, shard por tenant obligatorio ni ANN importado. L135–138: errores **asignados/sobrescritos**, no acumulados upstream; CONSOLIDATED L207 lo expresa correctamente. Los fallos acumulados pertenecen al contrato propuesto C13. |
| [pgvector](../pgvector.md), L117–140, L217–243, L260–280 | `vector_memory_search/exact_scan`, `VecIndex::search`, explain, capabilities, rebuild/auditoría; C7 reporta parcial no vacío, C15 separa dominio/precisión/codec | Reporte read-only no depende de C13. Continuación sólo si backend la soporta; C13 sólo evidencia por revisión. No PostgreSQL/PGXS ni VACUUM como ready/secure erase; CONSOLIDATED L47/L114/L169 no universaliza dependencias. |
| [Letta](../letta.md), L26–41, L131–220, L254–277 | Landing distinta del complemento Letta Code fijado; assembler, `is_structural/commit`, scheduler; revisión/publicación, validación, reparación cercada, presupuesto/cobertura y delivery separados | No V1/archive, Git primario ni runtime nuevo; política confiable del host. Fetch autenticado/fixes reportados no son resolución universal; compilación no es entrega. CONSOLIDATED L48/L78–79/L122 mantiene esos límites. |
| [LanceDB](../lancedb.md), L198–242, L270–317, L463–485 | Explain/híbrido, ingestors/merge, writer y scheduler/rebuild existentes; lectura+detalle coherentes, freshness fuente+definición, Unknown e índice ligado a revisión/job objetivo | A/D/E diagnóstico no exige C ni MVCC nuevo; F target fijo y G reparación condicionada. No Lance/Arrow/DataFusion paralelos, branches por sesión ni RRF nuevo; `verified_version=None` no fence, job done no cubre escrituras posteriores. CONSOLIDATED L49/L87/L133/L177 respeta el alcance. |
| [Chroma](../chroma.md), L160–199, L276–324, L470–496 | SDK/WAL, materialización/tombstones, `Embedded::rebuild_index` secuencial y auditorías actuales; C13 discrimina dominios y C5/C6/C14 revisión/publicación/retención | Manifiesto/CAS/leases son alternativas condicionadas, no capacidades actuales ni mínimo universal. No wal3/control plane/ONNX obligatorio ni scheduler nuevo; flush no publica todo, cleanup no secure erase. CONSOLIDATED L50/L176 y L282 conserva TUI real distinta de desktop/web. |

## BENCHMARK-NOTES — comprobación de las precisiones congeladas

No se cambió ninguna fórmula. La columna de fuente remite al resultado
documental/ficha, no a una nueva ejecución ni a una consulta remota.

| Precisión y rango de las notas | Fuente congelada contrastada y resultado |
|---|---|
| Mem0 L28–34, L115–151 | T7 memoria L58–82 y ficha Mem0 §7: error agotado puede retornar `[]` y obtener PASS de abstención; no siempre cero ni excepción del driver. Gitlink `4b61c5d…`, corpus `resolve/main`, preparación/backend/población separados. |
| Cognee/BEAM L35–39 | Cognee L685–733 y T7 memoria L103–124: corpus preingerido, config efectiva y población **numérica por métrica**; conteo de entradas no censo de juzgadas ni equivalencia Modal. |
| Graphiti L35–39 | Graphiti L603–644 y T7 memoria L135–177: grupos persistentes reutilizados sin reset; segunda pasada no acredita corpus limpio. Construcción/juez de grafo no se convierte en QA/retrieval. |
| Graphify L40–42 | Graphify L360–381 y T7 memoria L448–490: sólo excluye sentinel no-match; negativa ambigua puede entrar en tokens/ratio. No se transforma salida corta en utilidad entregada. |
| Khoj L43–45 | Khoj L553–605 y cierre Anything/Khoj L171–194: calcula/imprime F1 pero devuelve recall; None excluidos y batch cero posible, no ejecutado. Las notas no proclaman validación global de la ficha ni reabren sus residuales históricos. |
| Weaviate L46–50 | Weaviate L688–726: nominal ≠ lectura efectiva ≠ envío; suma de duraciones concurrentes no tiempo de pared, GraphQL sin validación de vecinos gold. Valores congelados no recalculados. |
| AutoGen/AgBench L51–53 | AutoGen L431–472 y T7 plataformas L324–330: sin directorio, tarea no descubierta; padding sólo sobre filas presentes. Missing/Total no censo externo; fórmula original intacta. |
| Flowise L54–58 | Flowise L728–792 y T7 plataformas L359–377: contadores **simples**, resultados LLM aparte; latencia cliente incluye métricas/awaits de coste; suma flows dividida por filas, no requests. |
| Serviette L59–62 | Serviette L384–422: `compose down -v` destructivo sólo leído, no autorizado/ejecutado; FRAMES/realtime/preparación y denominadores no homologados. |
| Chroma/Meilisearch L63–67 | Chroma L511–552: `TODO: verify recall` no acredita recall calculado. T7 plataformas L499–547: Meili settings-add/ingest y preparación/captura distintos, default de modelo no identidad efectiva; E3 sigue prohibido/no acreditado. No acceso al repositorio Meilisearch ni a su forge en esta revisión. |
| Vanta/MemPalace/Hindsight L73–113, L153–163 | Definiciones any/all, sesiones/payload F1, recall frente a QA y traslado documental a AMB permanecen separados. LongMemEval conserva anclaje mutable expresamente declarado L87–89; no se le atribuye retrospectivamente V. |
| LoCoMo V L176–207 | `git show V:evals/eval_locomo.py`, L197–225, L624–645, L748–760, L811–868: sintético/fake fuera de self-test, payload F1, p50 duplicado como p95 y embeddings opcionalmente ausentes coinciden. No score propio, no comparación QA equivalente ni invalidación de Paso 23. |

C2 y §7 de CONSOLIDATED se mantienen como autoría del lead: esta revisión
comprueba su fidelidad al método y las notas; **no diseña** harness, prompt,
rúbrica, scoring, threshold, muestreo, dataset ni configuración.

## Referencias inmutables y control operativo

- Se resolvieron mediante `git show <SHA>:<ruta>` los **27 destinos V/S**
  definidos en CONSOLIDATED L296–322; pins y archivos existen. La observación
  de anclas anterior no se oculta bajo esa comprobación de existencia.
- Lectura de cuerpos discriminantes V: L1, RMW, cursor L0, `MemoryInput`,
  assembler, filtros, WAL sync, Wiki/worker, recibo y builder, drain HNSW,
  vigencia de derivados y LoCoMo. No se infiere recovery completo de ellos.
- Cuerpos S contrastados: MemoryLens L381–410 (genlog/anchor),
  ActivityPanel L38–91 (auditEvents, cursor/error/no configurado),
  RetrievalLens L138–228 (search/explain, get/vecinos/audit).
  Árbol S consultado con `git ls-tree`; no se atribuyen jobs dirty al pin.
- Todos los destinos de archivo de enlaces Markdown locales de ambos
  documentos existen. Esto no convierte navegación mutable de fichas/closures
  en snapshots Git ni valida automáticamente todos los cuerpos externos.
- Único archivo creado/editado: **este recibo**, mediante `apply_patch`
  absoluto. Comandos utilizados: `Get-Location`, `Get-ChildItem`, `Test-Path`,
  `Get-Content`, lecturas de rangos en memoria, `Get-FileHash`;
  Git de solo lectura `status --short`, `cat-file -t`, `show`, `ls-tree`,
  `ls-files`, `check-ignore -v --no-index`.
- No red, forge, clones/fetch, subagentes, `code_search`, CodeGraph/reindexado, apps,
  servicios, tests, builds, benchmarks, instalaciones, harnesses ni validadores
  de proyecto; ningún runtime iniciado o detenido.
- Sin staging, commits, push o cambios de configuración/ignore; la prohibición
  específica de Git writes prevalece sobre el flujo ordinario de commits.
  No índices, JSON, metadata, backlog, contratos ni otros informes editados.
- El worktree ya contenía cambios concurrentes ajenos; no se declara clean ni
  se atribuyen/revierten. `git ls-files -- <este recibo>` no mostró entrada y
  `git check-ignore -v --no-index` señala `.gitignore:268`, `docs/dev/research/`.
  La ausencia del recibo en status no es pérdida del archivo; no se fuerza add.
- Incidencia instrumental: `CONSTRAINTS.md` no existe en la raíz consultada;
  no se escribió código ni se debilitó ningún gate para cerrar esta revisión.

**Cierre:** dos documentos revisados íntegramente, contraste trazable conservado,
una observación baja de navegación y ningún bloqueante material nuevo acreditado.
Se propone el cierre documental con los parciales/no acreditados anteriores
visibles. No se modifica la autoridad congelada ni se ordena completar Meili E3,
iniciar runtime, implementar candidatos o ejecutar comandos fuera del encargo.
