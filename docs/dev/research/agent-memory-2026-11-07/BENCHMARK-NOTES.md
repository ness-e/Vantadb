---
title: "Notas de verificacion metodologica de benchmarks de memoria"
kind: research
status: proposed
description: "Contraste de las definiciones de metricas en VantaDB, MemPalace y el submodulo de evaluacion fijado por mem0"
tags: [research, memory, benchmarks]
---

# Notas de verificacion metodologica de benchmarks de memoria

## Dictamen

No hay una tabla de rendimiento competitivo validada por esta investigacion.
Aunque dos proyectos digan LongMemEval y top-5, pueden medir preguntas distintas
y usar definiciones diferentes de exito. Las siguientes observaciones proceden
de lectura del codigo; no son resultados de ejecucion.

Protocolo comun: [METHOD.md](METHOD.md). Los valores publicados por proveedores
deben conservar su etiqueta de claim, aun cuando existan resultados en el repo.

## 1. VantaDB: cobertura completa de sesiones relevantes

Fuente local inspeccionada en el worktree de VantaDB:
[`eval_longmemeval.py`](../../../../evals/eval_longmemeval.py),
`run_longmemeval_run`, lineas 469-535 en el momento de lectura.

- Inserta cada sesion completa bajo el namespace de su pregunta.
- Compara las claves recuperadas con `answer_session_ids`.
- `correct_any` exige interseccion no vacia.
- `correct_all` exige gold no vacio y que todas sus sesiones esten recuperadas.
- El agregado principal `recall_all` excluye las preguntas de abstencion.
- El F1 auxiliar compara la respuesta anotada con payloads recuperados; no
  evalua una respuesta generada por un agente.

El worktree tiene cambios concurrentes: estas lineas son un anclaje de lectura,
no un enlace inmutable al resultado de una version publicada.
El umbral declarado por el runner no prueba que se haya superado.

## 2. MemPalace: el resumen impreso usa recall-any

Fuente: `benchmarks/longmemeval_bench.py`, funciones `evaluate_retrieval`
(lineas 71-80) y bloque de resultados (lineas 3215-3240), en el commit local
registrado en [clone-manifest.json](clone-manifest.json).

- Calcula `recall_any`, `recall_all` y nDCG.
- El resumen `SESSION-LEVEL METRICS` imprime `recall_any@k`.
- Por tanto, su etiqueta resumida `Recall@k` no equivale por si sola al
  `recall_all@k` que VantaDB usa como agregado principal.
- La funcion aislada usa `all(...)` sin guard explicita de gold no vacio.
  Esto describe esa funcion; no demuestra que el flujo completo puntue
  incorrectamente abstenciones. Hay que revisar sus filtros y dataset.

Fuente documental adicional: `benchmarks/BENCHMARKS.md`. El propio documento
advierte que retrieval recall no se compara directamente con QA accuracy.
Tambien registra que el ultimo ajuste hasta el claim de 100% se hizo mirando
tres errores concretos, y distingue una particion dev/holdout posterior.
Es informacion del proveedor, no una certificacion independiente del split.

**Consecuencia:** una comparacion futura debe conservar simultaneamente las
definiciones any/all, la unidad sesion/turno, preguntas de abstencion, split y
presupuesto. No extrapolar una mejora de QA desde una mejora de recall.

## 3. mem0: el benchmark esta en un submodulo fijado

El clon de mem0 no inicializa submodulos. No se debe afirmar que `evaluation/`
contiene un runner inexistente en disco.

La lectura de `.gitmodules` y `git ls-tree HEAD evaluation` identifica:

- Repositorio dependiente: `mem0ai/memory-benchmarks`.
- Commit del gitlink: `4b61c5d31b9c668a12b4f5e78064248a02c82d2b`.
- Se leyeron por la API de contenidos de GitHub cuatro archivos en ese commit,
  sin clonar el proyecto, instalarlo ni ejecutarlo.
- Es inspeccion complementaria de una dependencia de mem0; no una inclusion
  nueva que eluda el filtro de 30.000 estrellas para los clones seleccionados.

Archivos leidos:

1. [README del submodulo](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/README.md).
2. [Runner LongMemEval](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/longmemeval/run.py).
3. [Metricas comunes](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/common/metrics.py).
4. [Cliente de backend](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/common/mem0_client.py).

En el runner, `process_question_retrieval` (lineas 631-725) no comprueba solo
si recupero el id de la sesion anotada. Busca memorias y pide a un juez LLM
un PASS/FAIL de soporte con acceso a la respuesta anotada.
La respuesta anotada se usa en el juez, no es por ello una fuga al retrieval.
Ese resultado es distinto del hit por identificador de sesion de los apartados
anteriores. Tambien existe un modo separado de answerer.

`compute_longmemeval_metrics` (lineas 866-908) agrega los scores presentes en
`evaluations` por cutoff y tipo, y usa su longitud como denominador. Antes de
comparar se debe contrastar ese conjunto emitido con las preguntas intentadas:
no inferir ni ausencia ni presencia de descarte de errores sin seguir el flujo.

El cliente distingue OSS y cloud; ambos backends no son intercambiables por
tener la misma firma Python. Tambien ofrece eliminacion por usuario
(`delete_user`, lineas 350-381). La presencia del metodo no prueba que el runner
lo invoque ni autoriza usarlo sobre usuarios compartidos.

## 4. Hindsight: cambio de propietario del protocolo

El README local `hindsight-dev/benchmarks/README.md` declara que LoCoMo y
LongMemEval se trasladaron a AMB (`vectorize-io/agent-memory-benchmark`).
La razon documentada es evitar dos implementaciones que deriven respecto a los
resultados publicados. Sus comandos pasan por `hindsight-system-evals`.

Esto es una declaracion documental revisada, no una reproduccion de AMB.
No atribuir los scores actuales al harness antiguo ni trasladar sus comandos
sin verificar commit, wrapper, destino de API y efectos de limpieza.
Los microbenchmarks con proveedores mock no certifican calidad semantica.

## 5. Acciones para VantaDB antes de comparar

No se cambia codigo ni se crean configuraciones de evaluacion en esta tarea.

1. Separar mecanica de self-test y certificacion con dataset/backend reales.
2. Mantener calibracion y holdout separados; nunca ajustar con el holdout.
3. Registrar metricas any/all, granularidad, abstencion y presupuesto efectivo.
4. Separar evidencia recuperada, juicio de soporte y respuesta generada.
5. Congelar contratos, modelos y datasets y conservar errores operativos.
6. Medir mejoras mediante ablaciones propias; no mediante tablas promocionales.

El runner LoCoMo actual de VantaDB se leyo de nuevo: permite sintetico por
defecto y fallback a `FakeLocomoStore` fuera de self-test
(`evals/eval_locomo.py`, bloque CLI 826-856). Es un riesgo concreto de etiquetado
de certificacion; no niega los benchmarks propios ya publicados del Paso 23.

## Limites de cobertura

No se ejecutaron preguntas, modelos, jueces ni runners. No se auditaron todos
los prompts ni se recalcularon los resultados guardados por proveedores.
No se extrajeron conclusiones de superioridad, coste real o latencia.
La lectura complementaria remota conserva su commit; el inventario principal
conserva las revisiones locales. El siguiente paso requiere autorizacion manual
para ejecutar comandos y una especificacion de comparacion propia.
