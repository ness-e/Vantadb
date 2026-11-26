---
title: "Fase 3 — Mercado y escenarios de adopción"
kind: report
description: "Fuentes primarias fechadas, proxies de demanda y modelo condicional de adopción a 12 y 24 meses."
---

# Fase 3 — Mercado y escenarios de adopción

[HECHO] Corte de consulta: **2026-11-06, 13:35–13:37 UTC**. El horizonte del modelo va del 2026-11-06 al 2028-11-06, con corte intermedio al 2027-11-06. Este reporte separa actividad de registries, atención pública y organizaciones activas; ninguna de las tres es intercambiable.

## Método y calidad de evidencia

[HECHO] Se usaron búsqueda DuckDuckGo vía websearch, contraste metasearch Bing/DuckDuckGo —sin resultados en esa consulta—, extracción Argus y consultas directas a GitHub/PyPI/npm/crates.io. Las cifras proceden de las APIs primarias, no de snippets ni de benchmarks de proveedores.

[HECHO] GitHub se consultó mediante `gh api repos/<owner>/<repo>` y campos `stargazers_count`, `forks_count`, `created_at`, `pushed_at`, `archived`, `license`. PyPI Stats: `https://pypistats.org/api/packages/<paquete>/overall`, categoría `without_mirrors`, filas diarias; se sumaron ventanas calendario contiguas. npm: endpoints `downloads/point/last-month`. Información del servicio: [PyPI Stats API](https://pypistats.org/api/), [GitHub REST](https://docs.github.com/en/rest/repos/repos#get-a-repository).

[HECHO] No hay una segunda captura histórica comparable de stars en este análisis. **Crecimiento de stars: no determinado**; no se inventa CAGR ni se divide el stock actual por la edad para simular crecimiento. Sí existe comparación de dos ventanas consecutivas de descargas Python. Los resultados de HTTP fallidos se trataron como no disponibles, no como cero.

[INFERENCIA] Stars aproximan atención y descargas aproximan distribución automatizada. CI, reinstalaciones, dependencias y varios artefactos por instalación impiden convertirlas directamente en usuarios, organizaciones, retención o ingresos.

## Atención pública: captura GitHub

[HECHO] Cada fila corresponde al endpoint enlazado, consultado en el corte indicado; forks son stock, no adopción empresarial.

| Proyecto / fuente primaria | Stars | Forks | Repositorio creado |
|---|---:|---:|---|
| [HECHO] [VantaDB](https://api.github.com/repos/ness-e/Vantadb) | 2 | 0 | 2026-04-24 |
| [HECHO] [Chroma](https://api.github.com/repos/chroma-core/chroma) | 29.648 | 2.602 | 2022-10-05 |
| [HECHO] [LanceDB](https://api.github.com/repos/lancedb/lancedb) | 11.850 | 1.152 | 2023-02-28 |
| [HECHO] [Qdrant](https://api.github.com/repos/qdrant/qdrant) | 35.446 | 2.799 | 2020-05-30 |
| [HECHO] [sqlite-vec](https://api.github.com/repos/asg017/sqlite-vec) | 8.240 | 361 | 2024-04-20 |
| [HECHO] [txtai](https://api.github.com/repos/neuml/txtai) | 13.065 | 966 | 2020-08-09 |
| [HECHO] [mem0](https://api.github.com/repos/mem0ai/mem0) | 68.510 | 8.140 | 2023-06-20 |
| [HECHO] [Graphiti](https://api.github.com/repos/getzep/graphiti) | 32.587 | 3.347 | 2024-08-08 |
| [HECHO] [Zep](https://api.github.com/repos/getzep/zep) | 4.998 | 656 | 2023-04-29 |

[HECHO] Todos devolvieron `archived=false`. La fecha de creación remota de VantaDB difiere de su primera fecha de autor local; ver [fase 1](fase1-estadisticas.md). No se suman Zep y Graphiti como comunidades independientes ni se infiere abandono de una fecha de push aislada.

## Distribución Python y variación mensual

[HECHO] Ventana reciente **2026-10-07 a 2026-11-05**, previa **2026-09-07 a 2026-10-06**. Se excluye `with_mirrors` para no duplicar categorías. Variación calculada: `100 × (reciente / previa − 1)` únicamente cuando hay 30 filas diarias en cada ventana.

| Paquete / fuente primaria | Descargas recientes | Previas | Filas diarias reciente/previa | Variación |
|---|---:|---:|---|---:|
| [HECHO] [vantadb-py](https://pypistats.org/api/packages/vantadb-py/overall) | 712 observadas | 879 observadas | 27/24 | **No determinada** |
| [HECHO] [chromadb](https://pypistats.org/api/packages/chromadb/overall) | 8.699.993 | 6.636.980 | 30/30 | +31,08% |
| [HECHO] [lancedb](https://pypistats.org/api/packages/lancedb/overall) | 7.570.065 | 7.784.002 | 30/30 | −2,75% |
| [HECHO] [qdrant-client](https://pypistats.org/api/packages/qdrant-client/overall) | 14.999.433 | 12.301.401 | 30/30 | +21,93% |
| [HECHO] [sqlite-vec](https://pypistats.org/api/packages/sqlite-vec/overall) | 7.193.570 | 5.673.293 | 30/30 | +26,80% |
| [HECHO] [txtai](https://pypistats.org/api/packages/txtai/overall) | 14.427 | 14.229 | 30/30 | +1,39% |
| [HECHO] [mem0ai](https://pypistats.org/api/packages/mem0ai/overall) | 2.597.486 | 2.059.068 | 30/30 | +26,15% |
| [HECHO] [graphiti-core](https://pypistats.org/api/packages/graphiti-core/overall) | 875.693 | 628.172 | 30/30 | +39,40% |
| [HECHO] [zep-python](https://pypistats.org/api/packages/zep-python/overall) | 58.116 | 51.366 | 30/30 | +13,14% |

[HECHO] En VantaDB se recibieron 66 filas dentro de la ventana de 90 días, con suma observada 1.743. No se imputan los días ausentes como cero ni se anualiza una serie incompleta.

[INFERENCIA] El conjunto demuestra actividad de distribución considerable en soluciones cercanas, no un TAM monetario medido. La variación negativa de LanceDB y el desacople entre stars/descargas de txtai invalidan una narrativa de crecimiento uniforme. Una sola comparación mensual está expuesta a calendario, releases y consumidores automatizados.

### VantaDB: metadatos de publicación frente a instalaciones

| Fuente primaria | Resultado observado | Límite |
|---|---|---|
| [HECHO] [PyPI vantadb-py](https://pypi.org/pypi/vantadb-py/json) | Versión actual 0.9.0 | Metadata no prueba instalación en cada plataforma |
| [HECHO] [npm vantadb](https://registry.npmjs.org/vantadb) | latest 0.9.0; repositorio ness-e/Vantadb | Publicado 2026-10-20 |
| [HECHO] [npm vantadb-node](https://registry.npmjs.org/vantadb-node) | latest 0.9.0; descripción binding napi-rs | Publicado 2026-10-20; metadata sin campo repository |
| [HECHO] [npm vantadb-wasm](https://registry.npmjs.org/vantadb-wasm) | latest 0.9.0; repositorio ness-e/Vantadb | Publicado 2026-10-20 |
| [HECHO] [crates.io vantadb](https://crates.io/api/v1/crates/vantadb) | default/max version 0.9.0; 321 descargas acumuladas; recent_downloads 257 | Campo recent sin homologarlo aquí a otra ventana |

[HECHO] Descargas npm de 2026-10-07 a 2026-11-05: [vantadb 152](https://api.npmjs.org/downloads/point/last-month/vantadb), [vantadb-node 85](https://api.npmjs.org/downloads/point/last-month/vantadb-node), [vantadb-wasm 136](https://api.npmjs.org/downloads/point/last-month/vantadb-wasm). **No se suman como usuarios**: wrapper y runtime pueden corresponder a una misma instalación.

[HECHO] Hay deriva documental comprobable: [SURFACE_MANIFEST](../../../api/SURFACE_MANIFEST.md) describe Node como planeado/no publicado, mientras npm devuelve 0.9.0. La wiki incorpora la observación fechada; el manifiesto canónico no se modifica por estar fuera del alcance autorizado. La clasificación EXPERIMENTAL es un eje distinto de la existencia del paquete.

[INFERENCIA] La evidencia disponible es compatible con una fase de distribución inicial y atención pública pequeña. **No demuestra cero usuarios de producción**, ni permite conocer cuántos hay. Tampoco una versión publicada prueba que su código sea idéntico al HEAD auditado.

## Mapa competitivo: capacidades, no rankings publicitarios

| Proyecto | [HECHO] Posicionamiento declarado en fuente oficial | [INFERENCIA] Consecuencia para VantaDB |
|---|---|---|
| [Chroma](https://github.com/chroma-core/chroma) | API Python/JS, modo local y client/server; Cloud anuncia vectorial/híbrida/full-text | «Local» y «híbrido» no son diferenciadores exclusivos; comparar por modalidad concreta |
| [LanceDB Embedded](https://docs.lancedb.com/basics/lancedb-embedded) | Librería embebida en proceso, SDK Python/TS/Rust, disco u object storage; sin servidor obligatorio | Competidor directo del argumento zero-server; medir recursos, ergonomía y garantías, no solo lenguaje |
| [Qdrant](https://github.com/qdrant/qdrant) | Motor vectorial con filtros y despliegue de servicio; cliente Python también tiene modalidades locales | No reducir todo Qdrant a «requiere clúster» ni equiparar qdrant-client local al servidor Rust |
| [sqlite-vec](https://github.com/asg017/sqlite-vec) | Extensión SQLite escrita en C, float/int8/binario, metadata/auxiliary/partition columns; pre-v1 | Ventaja de integración SQLite y simplicidad; VantaDB debe justificar un motor propio |
| [txtai](https://github.com/neuml/txtai) | Framework Python con embeddings DB, sparse/dense, grafos, SQL, pipelines y MCP | «Vector+grafo+SQL/MCP» tampoco es exclusivo; diferenciar despliegue y coste total |
| [mem0](https://github.com/mem0ai/mem0) | Capa de memoria, SDK, servidor y managed; extracción y retrieval multiseñal | Compite por el resultado de memoria del agente, no solo por ANN; no comparar sus cifras managed con VantaDB sin protocolo común |
| [Graphiti/Zep](https://github.com/getzep/graphiti) | Graphiti: framework de grafos temporales con backend/LLM aportados por usuario; Zep: capa enterprise de contexto gestionado | Provenance/tiempo/gobernanza tienen alternativas; VantaDB puede competir por integración embebida, no atribuirse invención exclusiva |

[HECHO] La extracción de LanceDB redirigió de `https://lancedb.com/docs/` a `https://docs.lancedb.com/`; se siguió y consultó la página oficial Embedded. El README actual de mem0 diferencia expresamente benchmarks de su plataforma managed y SDK abierto; no se usan sus porcentajes para afirmar superioridad de ningún motor.

## Diferenciación comprobable y brechas

1. [HECHO] El worktree combina SDK Rust embebido, WAL/backends, ANN, BM25/RRF y grafo: [superficie](../../../../src/lib.rs#L66-L222), [retrieval](../../../../src/sdk/search/mod.rs#L73), [storage](../../../../src/storage/engine/mod.rs). [INFERENCIA] Su integración en un proceso y una familia de contratos es una propuesta defendible; no prueba ventaja de latencia/coste.
2. [HECHO] Hay capa cognitiva separada y hooks Python, con límites de transporte explícitos: [vanta-memory](../../../../vanta-memory/src/lib.rs), [matriz](../../../api/BINDINGS_NAMESPACES.md). [INFERENCIA] Permite un relato de memoria completa, pero no debe anunciarse paridad cognitiva JS/WASM.
3. [HECHO] Hay formatos versionados, controles de memoria y suites de paridad: [config](../../../../src/config.rs), [escenario](../../../../tests/parity/scenario.json). [INFERENCIA] El valor comercial depende de demostrar recuperación y contratos bajo fallo, especialmente por los candidatos de [fase 2](fase2-tecnico.md).
4. [HECHO] La evidencia pública recopilada no identifica clientes de pago ni organizaciones activas verificadas. [INFERENCIA] El déficit principal de go-to-market no se resuelve añadiendo más índices: faltan casos reproducibles, onboarding fiable, referencias y un canal de adquisición medido.
5. [INFERENCIA] «Más rápido», «más barato», «production-ready», «único» o «memoria sin pérdida» requieren comparativas y fault tests comunes; no se conceden por tener Rust/HNSW/WAL ni por documentos de benchmarks sin reproducción actual.

## Modelo de adopción: escenarios condicionales

[ESPECULACIÓN] Unidad: **organización/proyecto activo retenido al final de mes**, no descargas. Se modelan únicamente altas futuras del programa, con `A₀=0` como origen matemático; la base instalada real es desconocida, no se afirma que sea cero. «Activo» significa haber integrado y seguir utilizando el producto en su flujo, definición que habría que validar con entrevistas o señal voluntaria.

[ESPECULACIÓN] Parámetros elegidos para planificación, **no estimados estadísticamente** de stars/registries:

| Parámetro mensual | Conservador | Base | Optimista |
|---|---:|---:|---:|
| [ESPECULACIÓN] Q: prospectos cualificados nuevos, sin duplicados | 100 | 300 | 1.000 |
| [ESPECULACIÓN] a: proporción que integra y activa | 8% | 12% | 18% |
| [ESPECULACIÓN] r: retención mensual de activos | 85% | 90% | 94% |
| [ESPECULACIÓN] p: fracción de activos que paga | 2% | 4% | 6% |
| [INFERENCIA] Altas activas mensuales Q×a | 8 | 36 | 180 |

[ESPECULACIÓN] Ecuaciones, sin crecimiento compuesto de leads oculto:

```text
A_t = r × A_(t-1) + Q × a
A_t = Q × a × (1 − r^t) / (1 − r)
P_t = p × A_t
```

[INFERENCIA] Resultados aritméticos de esos supuestos —redondeados para presentación, no durante el cálculo—:

| Escenario | A₁₂ activos | A₂₄ activos | P₁₂ pagadores esperados | P₂₄ pagadores esperados |
|---|---:|---:|---:|---:|
| [INFERENCIA] Conservador | 45,75 | 52,25 | 0,91 | 1,05 |
| [INFERENCIA] Base | 258,33 | 331,28 | 10,33 | 13,25 |
| [INFERENCIA] Optimista | 1.572,24 | 2.320,50 | 94,33 | 139,23 |

[INFERENCIA] Los pagadores fraccionarios representan un resultado esperado del modelo, no contratos ni clientes existentes. El escenario base **no es «el más probable»**: no se asignan probabilidades. El optimista exige 24.000 prospectos cualificados distintos en dos años y un salto de distribución que la captura actual no demuestra.

[ESPECULACIÓN] Condiciones necesarias por escenario: conservador, mantenimiento y discovery orgánico limitado; base, tutoriales verificables, integraciones mantenidas y demos con seguimiento voluntario; optimista, distribución mediante partners/comunidades y evidencia de uso recurrente. Todos requieren resolver o descartar los riesgos prioritarios de durabilidad y hacer estable la ruta de instalación.

### Sensibilidad y criterio de abandono del supuesto

[INFERENCIA] Con parámetros optimistas salvo retención, `r=90%` produce A₂₄=1.656,42 y P₂₄=99,39; `r=97%` produce A₂₄=3.111,50 y P₂₄=186,69. Pequeñas variaciones de retención alteran sustancialmente el resultado; extrapolar descargas no captura esa incertidumbre.

[INFERENCIA] Con a=12%, p=4%, r=90%, el límite de pagadores cuando t crece es `Q×0,12×0,04/0,10`. Para alcanzar los 40 pagadores del equilibrio operativo de [fase 4](fase4-financiero.md), se requieren **más de 833,33 prospectos cualificados/mes** en régimen; Q=300 no lo alcanza ni prolongando indefinidamente el mismo supuesto.

[INFERENCIA] Si no puede medirse adquisición cualificada, activación o retención, se deben reportar como desconocidas y sustituir este modelo, no presentar sus salidas como forecast calibrado.

## Acciones de validación comercial

1. [INFERENCIA] Elegir un caso estrecho: agente Python local con memoria híbrida y grafo, con restore documentado. Publicar ejemplo y límites por runtime, no una promesa universal.
2. [INFERENCIA] Medir voluntariamente funnel por organizaciones/proyectos con deduplicación consentida; no habilitar telemetría nueva ni rastrear usuarios desde esta tarea.
3. [INFERENCIA] Obtener referencias reproducibles de integración/restore y entrevistas de compra; reemplazar Q/a/r/p por datos de cohortes.
4. [INFERENCIA] Comparar VantaDB y alternativas con mismo dataset, embeddings, filtros, hardware, presupuesto de recuperación y modelo de crash. Incluir coste de operar y migrar; no reutilizar rankings de proveedores.
5. [INFERENCIA] Actualizar en una tarea autorizada el manifiesto y sus gates de publicación para distinguir versión disponible, plataformas comprobadas y madurez.

## Reproducción de las cifras

[HECHO] Receta exacta de ventanas: filtrar `category=="without_mirrors"`, usar fin `2026-11-05`; reciente `2026-10-07..2026-11-05`, previa `2026-09-07..2026-10-06`; contar filas y sumar `downloads`. Exigir 30 filas por ventana antes de calcular variación. Los valores y fechas quedan congelados en estas tablas; una consulta futura puede devolver otra serie.

[INFERENCIA] Receta exacta de escenarios, suficiente para reproducir sin datos privados:

```python
from decimal import Decimal as D

escenarios = (
    ("conservador", 100, "0.08", "0.85", "0.02"),
    ("base", 300, "0.12", "0.90", "0.04"),
    ("optimista", 1000, "0.18", "0.94", "0.06"),
)
for nombre, q, a, r, p in escenarios:
    activos = D(0)
    for mes in range(1, 25):
        activos = D(r) * activos + D(q) * D(a)
        pagadores = D(p) * activos
        if mes in (12, 24):
            print(nombre, mes, activos, pagadores)
```

[HECHO] Navegación: [reporte maestro](REPORT.md) · [estadísticas](fase1-estadisticas.md) · [técnico](fase2-tecnico.md) · [finanzas](fase4-financiero.md).
