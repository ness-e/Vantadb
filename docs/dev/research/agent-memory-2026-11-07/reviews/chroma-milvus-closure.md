---
title: "Reconfirmación de correcciones de Chroma y Milvus"
kind: research
status: proposed
description: "Cierre propuesto de CH-1, CH-2 y MV-1; MV-2 conserva una contradicción de cobertura documental"
tags: [research, memory, engineering]
---

# Reconfirmación de correcciones de Chroma y Milvus

## Índice

- [Alcance y procedencia](#alcance-y-procedencia)
- [Chroma: recibo durable](#chroma-recibo-durable)
- [Milvus: recibo durable](#milvus-recibo-durable)
- [Veredicto y operaciones](#veredicto-y-operaciones)

## Alcance y procedencia

Segundo revisor de la reconfirmación; revisión independiente del autor.
Se comprobó primero que las correcciones ya estaban incorporadas.
La [revisión anterior](chroma-milvus.md), **343 líneas leídas completas**,
acredita la lectura completa de las entregas anteriores de ambos informes.
No se vuelve a contabilizar esa acreditación como lectura completa nueva.
Esta pasada es de **correcciones y coherencia**, no una segunda auditoría
exhaustiva de los proyectos ni certificación de sus propuestas.

Normativa leída: [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md), en particular
92–110 y 188–202; [VANTA-REFERENCE](../VANTA-REFERENCE.md), 54–82;
[METHOD](../METHOD.md), 107–169; [BENCHMARK-NOTES](../BENCHMARK-NOTES.md),
129–164; [REVIEW-COVERAGE](../REVIEW-COVERAGE.md), 37–60.
La evidencia previa se identifica por rangos del recibo anterior; no por
un ID de ejecución irrecuperable. Las filas heredadas no significan relectura
de todo su código fuente en esta pasada.

Pins comprobados con `git rev-parse HEAD`, sin fetch:

| Proyecto | Clon / snapshot |
|---|---|
| Chroma | `C:\Users\Eros\chroma`, `1a3ac2c7c623adbd03ed2508dc27b98cdf26016d` |
| Milvus | `C:\Users\Eros\milvus`, `eba8be50f84776210d3946d83840dea18924846f` |
| VantaDB | Workspace solicitado, `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` |

Los contrastes nuevos de código usan **`git show <pin>:<ruta>`**.
No se renovó el estado público de issues: fechas, comentarios y merge
permanecen acreditados por la revisión anterior, no por una consulta nueva.
No fue necesaria red pública ni carga de servidores.

## Chroma: recibo durable

**Entrega contrastada:** [chroma.md](../chroma.md), 697 líneas.
Relectura acotada: 1–65, 139–431 y 447–697; no lectura completa nueva.
SHA-256 de la entrega:
`20ACE25BDD0824344697B68CFA917FE69954D6E8601107F7CC40864E4D81E171`.
Antecedente: revisión anterior 27–154; hallazgos CH-1/CH-2, 56–97.

### Ledger 0 + diez ejes

| Eje | Estado | Evidencia anterior identificada y contraste de cierre |
|---|---|---|
| 0. Identidad, arquitectura y flujos | sustentado | Recibo anterior 29–33,42,106–124. Identidad actual 11–18 conserva pins y backends separados; no se reaudita toda la consulta local |
| 1. Recomendaciones | sustentado | Anterior 43; actual 139–151 conserva extensión de audit/rebuild y derivados, no infraestructura nueva |
| 2. Fichas Rust | sustentado | Anterior 44,99–104, con reservas D/E. Actual 276–379 resuelve dominio de E y seam de D; contratos siguen propuestos, locks/publicadores internos quedan limitados |
| 3. Incidencias upstream | sustentado | Anterior 45,139–147; actual 424–431,447–454 conserva reporte/versiones y no acredita fix. Sin nueva comprobación del estado público |
| 4. Prerrequisitos Vanta | sustentado | Anterior 46; actual 285–308,470–484 identifica rebuild real y mantiene inventario de ownership como condición antes de producto |
| 5. Rechazos | sustentado | Anterior 47; actual 486–496 no importa wal3/Cloud ni capacidades ya existentes |
| 6. Advertencias | sustentado | Anterior 48,149–152; actual 498–507 mantiene fsync distinto de prueba de crash y separación de backends |
| 7. Benchmarks | sustentado | Anterior 49,125–129; actual 509–552 conserva cuatro queries, log vacío, resultados no validados, errores y límites de Criterion. Fuente del runner heredada, no medida nueva |
| 8. Matriz | sustentado | Anterior 50; actual 554–571 conserva publicación parcial y recibo no acreditado completo; el seam nuevo no convierte esas garantías en existentes |
| 9. Negativos Vanta | sustentado | Anterior 51,132–135; actual 573–588 mantiene hipótesis separadas de defectos y LoCoMo separado del motor |
| 10. UI/escritorio | sustentado | Anterior 52,130–131; actual 590–608 mantiene TUI, no desktop/web, y Studio por referencia, no código revalidado |

«Sustentado» califica el tratamiento documental dentro del alcance declarado,
no exhaustividad ni implementación aprobada.

### CH-1: corrección verificada; cierre recomendado

**Texto corregido:** `chroma.md:330–345,370–379`. Ya no se limita al
envoltorio de copia: identifica reparación exclusiva de `ef_construction`,
precondiciones/rechazos y aceptación adversa futura.

**Contraste nuevo:** [hnsw_config_repair.rs:146–329](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/cli/src/hnsw_config_repair.rs#L146-L329).
La función empieza en 157; el rango incluye el validador 148–154.

- 174–198 y 210–215 corrigen configuración/schema y metadata legacy, no
  edges ni embeddings. 204–207 exige exactamente un segmento de tipo
  HNSW local persistido.
- Con header presente, 241–261 inspecciona checkpoint, exige mapa y dimensión
  concordante y rechaza SQLite por delante. 264–265 limita el formato a v1.
- 272–289 sustituye esfuerzo inválido por `max(ef solicitado, M)`; si ese
  reemplazo supera 4096 pide rebuild. **No es un algoritmo para reparar M**,
  ni una orden incondicional de reconstruir cualquier índice cuyo M sea grande.
- 291–296 rechaza recovery pendiente porque no demuestra completitud del log.
  Sin header, 298–310 rechaza watermark positivo o archivos residuales;
  no equivale a declarar corrupta toda colección aún no persistida.

**Integridad contrastada:** [inspect_segment:493–660 e inspect_hnsw_metadata:662–780](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/cli/src/hnsw_integrity_check.rs#L493-L780).
551–568 distingue ausencia de checkpoint/stub nativo vacío sin watermark
durable de pérdida parcial; la excepción no cubre archivos arbitrarios dañados.
569–575 informa recovery necesario **sin acreditar logs completos**.
671–705 distingue checkpoint adelantado a SQLite (`FastForward`) del atrasado
(`Corrupt`); 708–779 verifica dimensión y consistencia del mapa.
Fast-forward es reconciliación de offset, no prueba de todas las operaciones
retenidas. Esos casos ahora están en la aceptación futura de E.

No se ejecutó repair, replay ni apertura de un índice. La regla de copia
segura por sí sola sigue sin demostrar recuperabilidad.

### CH-2: seam verificado; cierre recomendado con límites

**Texto corregido:** `chroma.md:285–321`. Identifica operación, orden, efecto
persistente y límites; manifiesto/CAS/lease ya se rotulan alternativas
condicionadas, no otro pipeline obligatorio.

**Código Vanta releído al pin**, común también a MV-1:

- [admin.rs:19–32](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/admin.rs#L19-L32):
  `Embedded::rebuild_index` comprueba read-only/handle y llama
  **vector → derivados → texto → sparse → scalar**, secuencialmente con `?`.
  Compone el reporte al final; no es solo un enum de particiones.
- [impl_rebuild.rs:16–58](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_rebuild.rs#L16-L58):
  deriva operaciones de namespace/payload, aplica batch si no está vacío y
  después llama `write_derived_index_state`.
- [impl_rebuild.rs:155–180](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_rebuild.rs#L155-L180):
  batch textual seguido de `write_text_index_state` y reporte/métrica.
  La evidencia observada es **batch → escritura de estado**; no certifica
  por sí sola todo el contrato de health ni commit conjunto entre ramas.

El mínimo incremental razonable es describir/instrumentar las fases existentes.
La interfaz prepare/publish y sus casos CAS corresponden a una alternativa
propuesta si el inventario la justifica, no a una segunda reconstrucción
exigida ni a tipos existentes. Faltan locks del caller, ownership completo,
publicadores internos por índice y reclamación end-to-end. **No se deduce
carrera ni atomicidad insuficiente** de llamadas secuenciales con `?`.
Cerrar el defecto de identificación del seam no cierra esas validaciones.

### Límites y veredicto Chroma

Se conservan las lagunas anteriores 149–152: consulta local completa,
sysdb transaccional, wal3 writer/GC, Cloud/auth, dependencias legales, Studio
por código y dinámica. Las menciones nuevas a C9b/rangos extendidos están
en E (342–345); el inventario C9/C10 (628–629) sigue siendo una lista acotada,
no evidencia de auditoría exhaustiva ni una fuente adicional inexistente.
**Recomendación: CH-1/CH-2 cerrables documentalmente; informe aceptable con
límites dentro del alcance estático, sujeto al responsable principal.**
Este recibo no constituye aceptación final.

## Milvus: recibo durable

**Entrega contrastada:** [milvus.md](../milvus.md), 867 líneas.
Relectura acotada: 1–33, 171–335, 420–480, 500–737 y 755–867;
no lectura completa nueva. SHA-256 de la entrega:
`6829A293BAB5CD5D90F89F7955AC9811BE9D84C5FBF1303B89042106677BBE09`.
Antecedente: revisión anterior 156–326; MV-1/MV-2 en 184–227.

### Ledger 0 + diez ejes

| Eje | Estado | Evidencia anterior identificada y contraste de cierre |
|---|---|---|
| 0. Identidad, arquitectura y flujos | sustentado | Anterior 158–161,170,236–264; actual 25–31 conserva pin. Flujos completos heredados, no relectura de todos sus loaders/WAL |
| 1. Recomendaciones | sustentado | Anterior 171; actual 171–178 conserva propuestas condicionadas. F/G no son producto aprobado |
| 2. Fichas Rust | parcial | Anterior 172,184–234; actual C 289–328 resuelve seam, G 442–478 se degrada honestamente a requisito; 852–854 todavía afirma A–G completas, véase MV-2 |
| 3. Incidencias upstream | sustentado | Anterior 173,290–305; actual 500–538 mantiene reporte/issue/documentación distintos. No se renovó su estado ni se inspeccionó #54129 |
| 4. Prerrequisitos Vanta | sustentado | Anterior 174; actual 555–575 explicita locks/publicación pendientes y, para G, ambos lados de la proyección no localizados |
| 5. Rechazos | sustentado | Anterior 175; actual 577–588 mantiene rechazo de TSO/etcd, tuning trasladado y retry ciego |
| 6. Advertencias/licencia | sustentado | Anterior 176,321–324; actual 590–610 conserva licencia raíz limitada, recovery documental y Status distinto de nil |
| 7. Benchmarks | sustentado | Anterior 177,266–286; actual 612–685 conserva denominadores, exclusión de latencias fallidas, prepare divergente y fouram separado. No se ejecutó ni releyó todo el runner |
| 8. Matriz | sustentado | Anterior 178; actual 687–704 mantiene barrera/leases no acreditados y G específico no acreditado; V7 no prueba serializador/wildcard |
| 9. Negativos Vanta | sustentado | Anterior 179; actual 706–725 mantiene hipótesis sin diagnóstico de carrera/fuga y LoCoMo por referencia |
| 10. UI/escritorio | sustentado | Anterior 180,307–313; actual 755–761 mantiene superficies externas/Studio separados; código UI heredado, no nueva auditoría ni renderizado |

### MV-1: corrección verificada; cierre recomendado con límites

`milvus.md:289–302,309–319` incorpora exactamente el seam Vanta contrastado
en CH-2: `Embedded::rebuild_index` y batch seguido de estado en
`impl_rebuild.rs`. A/B dependen de ese mismo flujo mediante C
(231,270–271), no de un segundo publicador.
La preferencia es reportar fases/ownership; CAS/manifiesto/lease son
alternativas **si** la concurrencia inventariada lo exige.
La interfaz C (303–308), migración/FMEA/aceptación se leen bajo esa condición,
no como evidencia de que ya existen esos mecanismos ni mínimo incondicional.
Antes de una spec habrá que seleccionar alternativa; esta revisión no exige
implementarla ni completar una auditoría dinámica para identificar el seam.
Las lagunas de locks, lectores y publicación de cada rama se conservan en
296–297,309–314,567. No se ha descubierto un bug actual de Vanta.

### MV-2: corrección parcial; contradicción de cobertura aún abierta

**Corregido:** `milvus.md:442–446` rotula G como **requisito documental
exploratorio, NO extracción de código completada**. 450–454 reconoce que
#54126 sostiene una expectativa, no comportamiento del pin; 571 deja
pendientes validador de `output_fields`, expansión wildcard y proyección
antes/después de reload, más equivalente Vanta con ruta/símbolo.
529–532 y 702 niegan defecto/fuga acreditados y mantienen G no acreditado.
Los escenarios de problema/FMEA (448–449,471–473) se entienden hipotéticos,
no como vulnerabilidades observadas.

**Contradicción restante en el mismo hallazgo, no hallazgo renumerado:**
el recibo final `milvus.md:852–854` todavía proclama
**«siete fichas Rust completas A–G»**. Contradice 442–446 y el campo 2 del
contrato (97): no se leyó la unidad de código de G y se admite expresamente.
La revisión previa 208–227 ya identificaba esta falta y permitía la salida
documental; no hace falta inventar código para cerrarla.

**Corrección verificable pendiente:** cambiar la afirmación de cobertura final
para distinguir fichas propuestas A–F de G, requisito exploratorio abierto,
sin contar G como extracción completa. Tampoco convertir automáticamente
A–F en «seis extracciones de código completas»: D conserva inspiración
documental de recovery, como ya acreditó el recibo anterior 231–234.
La dependencia futura F/G de 832–833 no sustituye ese inventario.
No se exige implementar la propuesta ni localizar código nuevo si se mantiene
honestamente como requisito del issue. No se reabren sus datos públicos ya
acreditados ni se atribuye a `InjectionPolicy` el serializador no leído.

### Límites y veredicto Milvus

Siguen pendientes código de proyección G, WAL/recovery/GC/Knowhere/cgo
completos, propagación de deletes en todos los loaders, SDK/fouram externos,
licencias transitivas, Studio por código y pruebas dinámicas
(recibo anterior 321–324). **Recomendación: cerrar MV-1; mantener MV-2 abierto
solo por la contradicción documental de cobertura.** Tras corregirla, G puede
quedar como requisito exploratorio sin impedir un cierre estático con límites.
No hay aceptación final ni evidencia de fuga/carrera actuales.

## Veredicto y operaciones

| Proyecto | Recomendación al responsable |
|---|---|
| Chroma | Correcciones CH-1/CH-2 verificadas; cierre documental con límites propuesto |
| Milvus | MV-1 verificado; MV-2 parcialmente corregido, falta coherencia del recibo final |

Único archivo creado/editado: este Markdown. Comandos utilizados:
lecturas PowerShell (`Get-Content`, `Get-ChildItem`, `Test-Path`),
`git rev-parse`, `git show`, `Get-FileHash`; escritura mediante `apply_patch`.
Skills consultadas: catálogo, `documentation-skill` y límites de
`doubt-driven-development`; sin delegación anidada ni revisión adicional.
Sin apps/tests/builds/benchmarks/install/servicios, sin iniciar/parar runtime,
sin harness/prompts/config/scoring, sin git writes ni commits.
No se ejecutaron gates globales ni regeneración de índices.
Los hashes fijan las entregas examinadas; cambios posteriores requieren
reconfirmación, no invalidan retrospectivamente estos contrastes.

El responsable decide aceptación final y cualquier corrección/versionado
fuera de este archivo. El residual dinámico permanece fuera del alcance:
ni su ausencia prueba un defecto ni esta revisión lo resuelve.
