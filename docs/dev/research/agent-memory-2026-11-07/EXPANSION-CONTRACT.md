---
title: "Contrato de ampliacion profunda de los 42 informes comparativos"
kind: research
status: active
description: "Requisitos de profundidad, evidencia y revision para rehacer los 37 estudios y agregar cinco proyectos solicitados expresamente"
tags: [research, memory, engineering]
---

# Contrato de ampliacion profunda de los 42 informes comparativos

## Encargo y correccion de alcance

El propietario rechazo las 37 fichas iniciales: tener diez encabezados y dos
unidades extraibles no replica el estudio de OpenViking. Esta ampliacion exige
investigacion especifica y profundidad comparable, no inflar longitud ni
renombrar recomendaciones genericas. El informe de referencia es
[OpenViking-VantaDB](../../Backlog-OpenViking-VantaDB.md).

Se rehacen los **37 informes existentes** y se agregan **cinco**:
Weaviate, pgvector, Letta, LanceDB y Chroma. Son excepciones explicitas al filtro
anterior de estrellas. No alterar la captura original de 50 candidatos ni
presentar los cinco como si hubieran superado ese filtro. La nueva metadata y
los nuevos clones tienen evidencia separada.

Los informes originales son antecedentes utiles, no evidencia suficiente de
cumplimiento. Preservar sus hallazgos validos, corregir los errores y explicar
los cambios de conclusiones. La implementacion de features queda fuera.

## Indice

- [Estructura obligatoria](#estructura-obligatoria)
- [Profundidad verificable](#profundidad-verificable)
- [Ficha candidata completa](#ficha-candidata-completa)
- [Incidencias y seguridad](#incidencias-y-seguridad)
- [Lectura de benchmarks](#lectura-de-benchmarks)
- [Comparacion con VantaDB](#comparacion-con-vantadb)
- [Revision y cierre](#revision-y-cierre)

## Estructura obligatoria

Cada archivo principal es un informe autonomo, no un resumen que delega sus
diez respuestas en el metodo comun. Debe contener:

0. Identidad del proyecto y del clon, SHA completo, fechas, licencia por
   componente, alcance, evidencias y limites; mapa de carpetas, stack y
   dependencias; arquitectura, modelos de datos, superficies de API y
   despliegue; flujos concretos exitosos y de error.
1. Recomendaciones para VantaDB, priorizadas y justificadas por este proyecto.
2. Funcionalidades, algoritmos, logica y flujos extraibles, con fichas Rust
   suficientemente completas para redactar tareas despues de su aprobacion.
3. Issues, errores y requisitos upstream: fuentes publicas leidas,
   mecanismos, revisiones afectadas, aplicabilidad y validacion discriminante.
4. Correcciones y contratos previos en VantaDB, con evidencia propia o
   condicion explicitada; dependencias por candidato, no bloqueo universal.
5. Lo que no se traslada y por que; alternativas consideradas y costes ocultos.
6. Advertencias metodologicas especificas, no solo texto comun.
7. Benchmarks: lectura de runners, datasets, metricas, denominadores, defaults,
   requisitos, efectos y adaptacion aplicable a VantaDB.
8. Matriz de capacidades verificadas frente a VantaDB: existe, parcial,
   no acreditado o no aplicable, con fuentes y decision por fila.
9. Hallazgos negativos de VantaDB realmente sustentados; riesgos, hipotesis,
   falsos positivos descartados y revalidaciones necesarias separados.
10. Escritorio/UI: producto real, implementacion, flujos, fuente inspeccionada,
    clientes externos y oportunidades para Studio. Si no hay escritorio,
    explicar evidencia y limites, sin inventar uno.

Cerrar con inventario de fuentes, cobertura leida/no leida, dependencias
entre candidatos y validaciones manuales pendientes. Incluir TOC y explicar
como se dividiria el informe si pasa a specs activas.

## Profundidad verificable

- Recorrer implementaciones, no limitarse a README o nombres de directorio.
- Trazar ingesta/escritura, consulta/recuperacion y actualizacion/borrado,
  o los equivalentes reales de la biblioteca. Mostrar fronteras de commit,
  ownership, revision, cancelacion, errores y datos derivados.
- Explicar modelos, formatos, identificadores, filtros, scores, memoria
  conversacional frente a estado de ejecucion y dependencia opcional de LLM.
- Cubrir los mecanismos relevantes encontrados, no detenerse en dos
  candidatos por una restriccion editorial. Bibliotecas pequenas pueden
  justificar menos; la ausencia de oportunidad debe estar fundamentada.
- El informe OpenViking tiene aproximadamente 77 mil caracteres. La diferencia
  de longitud es una senal para revisar cobertura, no una metrica de calidad
  ni un requisito de relleno. Un proyecto diferente requiere su propio espacio.
- Toda afirmacion fuerte lleva archivo/simbolo/rango y enlace a SHA, o URL
  oficial con fecha y etiqueta de documentacion. Un enlace que existe no
  acredita que sostenga la afirmacion.
- No declarar lectura exhaustiva ni afirmar ausencia global desde una muestra.
- No reciclar el mismo DTO para aparentar oportunidades distintas: explicar
  diferencias y relaciones con los candidatos transversales ya existentes.

## Ficha candidata completa

Por cada mecanismo que merezca adopcion:

1. Problema de agente/memoria/contexto que resuelve y escenario concreto.
2. Comportamiento upstream observado, unidad minima y lectura de codigo.
3. Valor incremental frente a VantaDB y por que no duplica una capacidad.
4. Alternativas: extender lo existente, adapter, reimplementar o rechazar.
5. Contrato Rust **propuesto**, inputs, outputs, identidad/revision, errores
   e invariantes; no afirmar que los tipos hipoteticos ya existen.
6. Ubicacion y simbolos reales de integracion en motor, vanta-memory,
   SDK/adaptadores o Studio; no recrear sistemas paralelos.
7. Pasos ordenados, persistencia y migracion compatibles, reindexacion y
   estrategia de desactivacion/rollback cuando sean pertinentes.
8. Dependencias, relaciones con otros candidatos y trabajo existente.
9. FMEA: disparador, mecanismo, consecuencia, deteccion y mitigacion.
10. Criterios funcionales de aceptacion, casos normales y adversos, y
    observabilidad sin contenido sensible por defecto.
11. Alcance minimo, no objetivos y decisiones aun abiertas.

No inventar umbrales, pesos de ranking, prompts, evaluadores ni parametros de
pipeline. No reservar IDs oficiales ni modificar el backlog activo.

## Incidencias y seguridad

Investigar los issues relevantes de cada proyecto, no sustituirlos solamente
por hipotesis propias. Para cada incidencia efectivamente leida registrar:
URL, numero/titulo, estado observado y fecha, sintoma reportado, causa
acreditada o pendiente, codigo relacionado, posible correccion/PR y alcance
de aplicabilidad a VantaDB. Cerrar un issue no prueba correccion.

Una busqueda o acceso que falla debe quedar registrado con herramienta,
consulta/URL y error. Si no se pudo leer la incidencia, no inventar su
contenido ni presentarla como examinada. Agotar fallback razonable y explicar
la laguna; no afirmar que se completo ese eje cuando sigue pendiente.

Distinguir: hecho estatico, reporte upstream, documentacion, inferencia,
hipotesis y reproduccion pendiente. Un defecto de benchmark no acredita
automaticamente invalidacion de todas las cifras del proyecto.

Describir frontera de confianza/configuracion en riesgos de autenticacion,
tenant, rutas/SSRF, contenido hostil, secretos, telemetria y borrado. Scope de
consulta no sustituye autorizacion. Ningun modelo autoriza acceso o borrado.

## Lectura de benchmarks

Esta seccion es el protocolo de autoria del responsable principal. Los autores
describen y citan los evaluadores existentes; no crean ni modifican harnesses,
prompts, rubricas, formulas, thresholds o configuraciones nuevas.

Por cada familia o runner inspeccionado, documentar en el informe:

- Entry point y cadena de llamadas; CLI/config/defaults leidos, con rutas.
- Dataset y esquema/unidad: documento, vector, sesion, turno, pregunta o tarea.
- Preparacion, filtros, split y posible acceso a ground truth.
- Ingesta/extraccion, retrieval, answerer y juez, indicando cuales existen.
- Metricas exactamente como las calcula el codigo; poblacion y denominador;
  tratamiento de errores, vacios, abstenciones y preguntas no completadas.
- Modelos, embeddings, tokenizer, semillas, caches, budgets y concurrencia
  efectivos en fuente; valores desconocidos quedan desconocidos.
- Latencia/coste que incluye y excluye cada medida; warmup frente a medicion.
- Borrado, reset, escrituras persistentes, arranque de servicios, descargas y
  llamadas pagadas; comandos solo documentados para ejecucion manual futura.
- Claim publicado frente a resultado propio: en esta tarea no hay ejecucion.
- Adaptacion concreta: que adapter Vanta necesita, entradas/salidas que mapear,
  invariantes que conservar y que parte del runner no es portable.

Aplicar el protocolo ya autorado en [METHOD](METHOD.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md): separar motor, retrieval, memoria,
respuesta y exito de agente; no comparar metricas distintas ni descartar fallos
silenciosamente; calibrar en desarrollo y congelar holdout; source/dataset
aislados; registro por pregunta; costes y latencias por fase.

Si no se localiza un benchmark, registrar las superficies buscadas. Un test
unitario no es un benchmark de calidad ni prueba de ausencia global de runner.
No usar las lecciones de otros proyectos como si fueran codigo de este.

## Comparacion con VantaDB

La nueva referencia inicial es el HEAD
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, observado limpio.
Los commits ajenos posteriores no invalidan el snapshot pero impiden llamarlo
estado actual sin comprobacion adicional.

Existen BM25/HNSW/RRF, filtros, provenance, sesiones, dedup/supersession,
personas, escenas, skills, scheduler, checkpoints, ingestors, auditoria
textual, budgets de recall/compactacion, gobernanza de inyeccion y Studio.
La ruta legacy/core de recall tiene opciones: no inferir uso por defecto del
core hybrid desde la mera presencia del flag. Recuperacion de hechos y
checkpoint de ejecucion no son el mismo recurso.

La investigacion original c561158/dirty y el informe OpenViking son
antecedentes historicos, no pruebas de defectos actuales. Revalidar puntos
locales relevantes antes de declarar una carencia. Un archivo de test leido
prueba intencion/casos presentes, no resultado ejecutado.

## Revision y cierre

Cada proyecto debe tener autoria e inspeccion adversarial independientes.
El responsable principal juzga evidencia, resuelve contradicciones y revisa
las recomendaciones. No cerrar por numero de encabezados, lineas o citas.

Registrar por proyecto cumplimiento y lagunas de todos los ejes. Una fuente
inaccesible sigue siendo una laguna, aunque el resto del informe este escrito.
El indice no puede decir que todos los proyectos cumplen si faltan dossiers o
revisiones. Los clones y las capturas anteriores se conservan.

Solo se autorizan lectura de codigo, consultas publicas y escritura documental.
No ejecutar apps, builds, tests, benchmarks, instalaciones ni servicios.
Los checks offline de documentacion se realizan al cierre por lotes revisados,
sin red ni reformateo de archivos ajenos. Commits locales; nunca push sin orden.
