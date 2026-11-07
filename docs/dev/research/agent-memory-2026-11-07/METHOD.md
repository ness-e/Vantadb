---
title: "Metodo de evaluacion comparativa de memoria y contexto para agentes"
kind: research
status: proposed
description: "Contrato de evidencia, seleccion y comparacion estatica para adaptar ideas del ecosistema de memoria a VantaDB"
tags: [research, memory, engineering]
---

# Metodo de evaluacion comparativa de memoria y contexto para agentes

## Alcance y limites

Investigacion autorizada por el propietario: descubrir, descargar y estudiar
repositorios publicos relacionados con memoria, contexto, registro, conocimiento
y almacenamiento para agentes. No autoriza implementar sus ideas en VantaDB,
instalar dependencias, ejecutar aplicaciones, tests, benchmarks ni servicios.
Los comandos de ejecucion quedan para validacion manual posterior.

La captura de GitHub se realizo el 2026-11-07 UTC; la sesion del propietario
comenzo con fecha 2026-11-06. Los timestamps individuales estan en el JSON.

- Universo observado: 50 candidatos identificados por busqueda y conocimiento
  previo, contrastados mediante `GET /repos/{owner}/{repo}` de GitHub.
- Elegibles nuevos: 37 repositorios con `stargazers_count >= 30000`.
- Excluidos por estrellas: 12, conservados en el mismo inventario.
- OpenViking: 1 repositorio ya estudiado, no se vuelve a clonar ni actualizar.
- No es un censo exhaustivo de GitHub. La busqueda puede omitir proyectos.
- No hay maximo artificial de clones dentro del universo seleccionado.
- Se excluyeron de la investigacion de codigo recopilaciones, cursos y
  aplicaciones verticales sin componente reutilizable directamente pertinente.
- Las estrellas son un filtro solicitado, no evidencia de calidad o seguridad.
- Un proyecto archivado no se oculta: Flowise cumple estrellas, pero su estado
  de mantenimiento debe figurar en el dictamen.

Los valores de estrellas, licencia detectada y fecha proceden exclusivamente de
[github-candidates.json](github-candidates.json). Son un snapshot, no contadores
que se recalculan al renderizar los informes. `NOASSERTION` no equivale a ausencia
de licencia ni autoriza reutilizar codigo.

## Preparacion reproducible

Cada repositorio nuevo se guarda en `C:\Users\Eros\<nombre-repositorio>`.
Clon superficial de una rama, sin tags, submodulos recursivos ni descarga LFS.
No se ejecuta codigo del repositorio. El commit y las omisiones se registran en
[clone-manifest.json](clone-manifest.json). Las conclusiones corresponden al
commit local, no necesariamente a una version estable publicada.

No sobrescribir carpetas preexistentes ni actualizar OpenViking silenciosamente.
El historial superficial no sirve para atribuir la introduccion o correccion
historica de un defecto sin consultar evidencia adicional.

## VantaDB como referencia

HEAD observado al iniciar: `c561158253bb88192590fa406c61e0a27dd7df8d`.
El worktree contiene cambios ajenos concurrentes. El historial OpenViking usa
una base anterior: `632f49187ef8fce8e21abea78fba7ecbb8c5c463`.

Los informes de terceros no deben declarar ausentes las siguientes capacidades:
BM25/HNSW/RRF, procedencia, sesiones, supersession, dedup con juicio LLM opcional,
context assembly y compactacion con presupuesto compartido, personas, escenas,
skills, scheduler, checkpoints, ingestors wiki, auditoria del indice textual y
gate de autenticacion al arrancar fuera de loopback.

Studio vive en el worktree `develop-desktop`, no en la rama principal.
Ya tiene conexiones, memoria, actividad, escenas, skills y diagnostico retrieval.
La presencia de una capacidad no certifica que todas sus rutas esten integradas.
Antes de crear una tarea de implementacion se debe revalidar el estado actual.

## Contrato de cada informe

Cada informe conserva los diez puntos pedidos:

1. Recomendaciones para VantaDB y contexto arquitectonico del proyecto.
2. Partes extraibles, flujo real, lectura de codigo y mapeo a Rust/VantaDB.
3. Defectos, riesgos e issues transferibles, con su nivel de evidencia.
4. Prerrequisitos en VantaDB antes de incorporar la idea.
5. Componentes y practicas que no deben trasladarse.
6. Advertencias metodologicas.
7. Benchmarks existentes: runner, entrada, metricas, dependencias y riesgos.
8. Matriz existe/parcial/no acreditado/no aplicable frente a VantaDB.
9. Debilidades de VantaDB sugeridas por el contraste, no bugs inventados.
10. Escritorio, interfaz e inspeccion: implementado, externo o no localizado.

Cada informe incluye commit, rutas verificadas, enlaces inmutables a codigo,
archivos leidos, superficies no inspeccionadas y validaciones pendientes.
No afirmar que se audito todo un repositorio por haber revisado sus rutas clave.
Una interfaz anunciada o alojada fuera no prueba que su fuente este disponible.

Las fichas candidatas deben identificar problema, unidad minima a extraer,
seam existente, dependencias, errores/FMEA, acciones y aceptacion verificable.
Son propuestas para evaluar: no decisiones de implementacion aprobadas.
No reservar prefijos de backlog nuevos ni duplicar filas existentes.

## Etiquetas de evidencia

- **Codigo:** flujo identificado en archivos y revision local.
- **Documentacion:** declaracion oficial no comprobada en ejecucion.
- **Hipotesis:** riesgo que necesita una reproduccion discriminante.
- **Propuesta:** comportamiento deseado de VantaDB, todavia no implementado.
- **No localizado:** busqueda acotada sin prueba positiva; no ausencia absoluta.
- **Pendiente manual:** no ejecutado por restriccion del propietario.

No elevar una incidencia publicada a defecto vigente sin comprobar revision y
camino afectado. Tampoco declarar arreglado algo solo porque se cerro el issue.
Todo hallazgo de seguridad debe describir configuracion y frontera de confianza.

## Protocolo de comparacion de benchmarks

Este protocolo es un diseno del responsable de la investigacion; los informes
describen los runners upstream y remiten aqui. No se crean harnesses, prompts de
juez, umbrales ni configuraciones de evaluacion durante esta tarea.

### Pistas separadas

1. **Motor:** insercion, consulta, filtros, coherencia y recuperacion.
2. **Retrieval:** evidencias recuperadas frente a evidencia relevante anotada.
3. **Memoria:** sesiones -> extraccion/consolidacion -> recall.
4. **Respuesta:** contexto recuperado -> generacion -> evaluacion declarada.
5. **Agente:** exito de tarea en simulador o entorno controlado.
6. **Producto:** captura -> procesamiento -> recall -> contexto entregado.

No comparar porcentajes entre pistas ni confundir recall con exactitud QA.
No convertir la existencia de un harness en un resultado medido.

### Controles comunes

- Congelar commits, dataset y split, modelos, prompts, budgets, concurrencia,
  backends, versiones del servicio y opciones de importacion.
- Entrenamiento/calibracion separados del holdout; impedir que respuestas,
  evidencias anotadas o sesiones de test entren en la seleccion o memoria.
- Misma generacion y presupuesto para comparar variantes de memoria.
- Inicio con una muestra y una pregunta: demuestra flujo, no calidad.
- Instancias y datos desechables; no ejecutar borrado/restauracion compartidos.
- Registrar por pregunta: intentada, completada, recuperada, respondida,
  juzgada, error operativo, invalida y abstencion.
- No eliminar fallos del denominador silenciosamente.
- Desglosar coste/tiempo de importacion, embeddings, extraccion, recuperacion,
  generacion y juez; separar latencia retrieval de end-to-end.
- Declarar cardinalidad recuperada y presupuesto efectivo de contexto.
- Scores publicados son claims del proveedor hasta reproduccion equivalente.
- Ablaciones: baseline actual, una capacidad nueva y combinaciones justificadas.
- Calidad y performance requieren evidencia propia; no prometer mejora por idea.

### Aplicacion a VantaDB

- LoCoMo: loader del esquema original, dataset real explicito y backend real
  obligatorio para certificacion; fake/sintetico solo en modo declarado.
- LongMemEval: holdout congelado y metricas retrieval separadas de respuestas.
- tau2: adaptar memoria, no el scoring externo; mismo agente/herramientas/modelo,
  memorias train-only y ninguna persistencia de test hacia entrenamiento.
- RAG: retrieval, QA con retrieval y QA oracle como diagnostico separado.
- ANN: Recall-QPS y percentiles, datos/metricas/filtros/recursos equivalentes.
- Contexto: presupuesto, cobertura, reinyeccion, revisiones, aislamiento y
  trazabilidad. No se fija un umbral sin evidencia de baseline.
- Paso 23 de VantaDB ya documenta mediciones; no negar su existencia ni ampliar
  sus limites hasta convertirlo en certificacion de calidad o crash safety.

## Fronteras de adopcion y licencia

La prioridad es estudiar contratos y principios, no traducir productos enteros.
Cambiar Python/TypeScript/C a Rust no elimina automaticamente la derivacion.
No se filtran proyectos por licencia para estudiarlos; toda adopcion de codigo
o material derivado exige revisar archivo, dependencias y obligaciones.
Separar motor embebido, dominio de memoria y adaptadores/UI.
Las llamadas LLM deben seguir siendo opcionales fuera de primitivas puras.

## Definition of Done de esta investigacion

Inventario y exclusiones preservados; clones exitosos identificados por commit;
informe por proyecto con los diez ejes y limites de cobertura; sintesis sin
duplicados; revision adversarial y discrepancias resueltas o abiertas.
No implica que una propuesta este aprobada, que un bug este reproducido ni que
se haya ejecutado un benchmark. La validacion automatizada queda declarada como
pendiente manual cuando no se ha recibido autorizacion.


## Nota sobre enlaces y vigencia

Los enlaces inmutables a upstream (SHA fijado) fijan evidencia de terceros en ese commit. Los enlaces relativos a VantaDB son navegación a rutas/rangos observados en la lectura histórica de la fase inicial (HEAD `c561158` y worktree dirty de entonces), no garantía de identidad con el HEAD vigente — avanzó por commits ajenos (`b64383e` al cierre). No se persigue cada commit: la referencia es el snapshot inicial más la historia. Ningún SHA registrado contiene cambios dirty.
