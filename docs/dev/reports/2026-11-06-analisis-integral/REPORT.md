---
title: "VantaDB — Análisis integral del worktree"
kind: report
description: "Síntesis verificable de arquitectura, estadísticas, mercado, finanzas y regeneración documental al 2026-11-06."
---

# VantaDB — Análisis integral del worktree

[HECHO] Corte: **2026-11-06**. Worktree `develop`, HEAD `08c6dc603c009edf8d9b76cee14e7006a8a89ad7`. Se analizaron crates, bindings, evals, tooling y documentación presentes; se excluyeron `desktop/` y `web/` de otros worktrees.

[HECHO] Esta entrega modifica únicamente los cinco reportes, `docs/wiki/**`, `llms.txt` y, por excepción mínima autorizada, el índice generado `docs/index.md`. No se modificaron código, tests, configuración, `.opencode/`, licencia o controles de seguridad. **Sin staging, commits ni push.**

## Dictamen ejecutivo

[INFERENCIA] VantaDB tiene una arquitectura de producto sustancial: motor embebido, retrieval híbrido, grafo, memoria cognitiva y transportes con contratos explícitos. Su principal riesgo no es falta de funcionalidades, sino convertir esa amplitud en garantías demostrables y mantenibles.

[INFERENCIA] La prioridad técnica es **reproducir o descartar** los escenarios de rotación/replay WAL y aplicación anterior a Commit durable. Se observaron rutas que justifican esta prioridad; **no se reprodujo pérdida de datos** y no se etiqueta toda configuración como defectuosa.

[INFERENCIA] La evidencia pública acredita publicación y distribución inicial, no adopción empresarial demostrada. La estrategia más defendible es un caso de uso local estrecho, integración fiable y compradores de soporte, antes de ampliar promesas de plataforma administrada.

[INFERENCIA] Los escenarios comerciales y financieros son **condicionales**, no probabilidades calibradas. El caso base no cubre el coste económico mensual elegido; el optimista sí puede cubrir operación, pero no repone en dos años la inversión de referencia imputada.

## Entregables y evidencia

| Entregable | [HECHO] Contenido | Límite principal |
|---|---|---|
| [Fase 1](fase1-estadisticas.md) | Censo congelado, lenguajes/módulos, historia, churn, deuda y frescura | LNV/físicas, no SLOC exacto; falta tokei/cloc |
| [Fase 2](fase2-tecnico.md) | Arquitectura, FMEA por áreas, controles y pruebas discriminantes | Auditoría estática dirigida; sin pruebas dinámicas |
| [Fase 3](fase3-adopcion.md) | Fuentes primarias, competidores, distribución y escenarios | Sin usuarios únicos/cohortes ni baseline de stars |
| [Fase 4](fase4-financiero.md) | Reposición, operación, monetización y equilibrio | Sin contabilidad, ventas ni costes unitarios reales |
| [Wiki](../../../wiki/index.md) | 38 páginas por audiencia, arquitectura, módulos y transporte | Ejemplos no ejecutados; gates manuales pendientes |
| [Índice general](../../../index.md) y [llms.txt](../../../../llms.txt) | Regeneración oficial, inventario de 219 páginas | URLs remotas de páginas nuevas no publicadas por esta tarea |

[HECHO] Los reportes están bajo `docs/dev/reports/`, ignorado por Git. Existen en disco, pero no aparecerán en un `git diff` o staging normal; tampoco entran en el inventario oficial que filtra ignorados. **No se alteró `.gitignore` para ocultar esta limitación ni se forzó su incorporación al índice.**

## Resultados principales

### Tamaño, mantenimiento y controles

[HECHO] El censo tracked inicial comprende 1.455 archivos presentes; 1.355 archivos de texto reconocido, 438.933 líneas físicas y 394.961 líneas no vacías. Rust: 716 archivos y 264.780 LNV. El ratio de partición por rutas test/no-test es 36,11%, **no cobertura ni ratio SLOC exacto**. Fuente y receta: [fase 1](fase1-estadisticas.md).

[HECHO] Hay ocho miembros del workspace y paquetes independientes, entre ellos Node nativo y TS. Historia alcanzable: 4.250 commits; cinco identidades de autor, no cinco personas; dos identidades son bots explícitos. Tres marcadores TODO útiles tras descartar un «TODO campo» en español. El conteo de marcadores no mide toda la deuda.

[INFERENCIA] La concentración de contribución y la amplitud de motor/bindings/servicios crean riesgo de bus factor y coste de validación. La respuesta sostenible es reducir promesas y reforzar invariantes, no añadir otra superficie sin presupuesto de mantenimiento.

### Mercado y publicación

[HECHO] Las APIs primarias devolvieron VantaDB 0.9.0 en [crates.io](https://crates.io/api/v1/crates/vantadb), [PyPI](https://pypi.org/pypi/vantadb-py/json), [npm wrapper](https://registry.npmjs.org/vantadb), [npm Node](https://registry.npmjs.org/vantadb-node) y [npm WASM](https://registry.npmjs.org/vantadb-wasm), consultadas el 2026-11-06. Metadata verificada **no es instalación ni compatibilidad de plataforma probadas**.

[HECHO] El manifiesto documental canónico todavía clasifica Node como planeado/no publicado; npm confirma 0.9.0. La wiki distingue publicación comprobada y madurez declarada EXPERIMENTAL. Ese documento canónico, fuera del alcance, permanece sin modificar.

[HECHO] [GitHub VantaDB](https://api.github.com/repos/ness-e/Vantadb) devolvió 2 stars y 0 forks en el corte. PyPI Stats entregó 712 descargas observadas de `vantadb-py` en la ventana 2026-10-07..2026-11-05, con filas de solo 27 días; no se imputaron los ausentes. npm: wrapper 152, Node 85, WASM 136. Fuentes, ventanas y comparables: [fase 3](fase3-adopcion.md).

[INFERENCIA] No se suman esos artefactos para obtener usuarios ni se convierten stars en clientes. El crecimiento de descargas de competidores confirma actividad de distribución en el segmento, no que VantaDB pueda capturar un porcentaje arbitrario de ella.

### Escenarios y finanzas

[ESPECULACIÓN] Prospectos cualificados mensuales Q=100/300/1.000; activación a=8%/12%/18%; retención r=85%/90%/94%; conversión de activos a pago p=2%/4%/6%. `A_t=r×A_(t−1)+Q×a`, `P_t=p×A_t`. Son supuestos de planificación, no tasas observadas.

| Escenario | [INFERENCIA] Activos mes 24 | [INFERENCIA] Pagadores esperados mes 24 | [INFERENCIA] MRR mes 24 | [INFERENCIA] Resultado operativo acumulado 24m |
|---|---:|---:|---:|---:|
| Conservador | 52,25 | 1,05 | $134,82 | −$94.032,21 |
| Base | 331,28 | 13,25 | $1.709,43 | −$73.366,23 |
| Optimista | 2.320,50 | 139,23 | $17.960,67 | $117.873,04 |

[ESPECULACIÓN] Supuestos económicos: ticket blended hipotético $129/pagador-mes; variable $29; fijo económico $4.000/mes, incluyendo mantenimiento imputado. [INFERENCIA] Equilibrio mensual: 40 pagadores. El optimista lo cruza en el mes 5 **del modelo**, no en una fecha de venta pronosticada.

[INFERENCIA] Reposición estimada con 198.947 LNV del universo de producto: **$99.473,50 / $373.025,63 / $1.193.682**, según productividad/coste-hora explícitos. No es gasto histórico probado. Restando referencia al resultado de dos años, incluso el optimista queda en **−$255.152,58**. Fórmulas, sensibilidad y exclusiones: [fase 4](fase4-financiero.md).

[INFERENCIA] No se propone cobrar por retirar funciones ya Apache-2.0. Monetización compatible: soporte/implantación, operación managed y capacidades enterprise nuevas permitidas por las reglas open-core, sin prometer un servicio o SLA que no se haya validado.

## Top 5 fortalezas

1. [HECHO] **Integración embebida amplia:** almacenamiento, vector/BM25/RRF, grafo e IQL en una familia de APIs. [Fuente](fase2-tecnico.md#1-arquitectura-y-rutas-críticas).
2. [HECHO] **Controles de persistencia explícitos:** sync modes, versionado, CRC, locks y detección de truncado multi-shard. No equivalen a garantía exhaustiva. [Fuente](fase2-tecnico.md#escritura-y-recuperación).
3. [HECHO] **Contratos multi-runtime:** leaf FFI compartido, IDs exactos y escenario de paridad con excepciones declaradas. [Fuente](fase2-tecnico.md#3-ffi-y-contratos-entre-runtimes).
4. [HECHO] **MCP con controles útiles:** validación, stdout serializado, límite de concurrencia y gobernanza de inyección. [Fuente](fase2-tecnico.md#4-mcp-y-adaptadores-de-ecosistema).
5. [HECHO] **Maquinaria de entrega/documentación:** publicación endurecida y gates de API/versiones/docs presentes en archivos. Su resultado actual no se certificó. [Fuente](fase2-tecnico.md#5-cicd-releases-docs-y-evals).

## Top 5 riesgos

1. [INFERENCIA] **Durabilidad no cerrada bajo rotación y crash transaccional:** mayor prioridad de comprobación; pérdida de datos no reproducida.
2. [HECHO] **Fronteras FFI heterogéneas:** error de gate sin código uniforme, fallback por mensajes y drain WASM no bloqueante. [INFERENCIA] El contrato observado puede diferir del prometido si no se prueba el camino completo.
3. [INFERENCIA] **Deriva writer/proxy MCP y trabajo no cancelable:** config anunciada, timeout y superficie loopback necesitan comprobación; no se demostró bypass general de gobernanza.
4. [INFERENCIA] **Confianza excesiva en controles parciales:** evals no calibrados externamente, checks no bloqueantes/degradados y autoridad documental stale pueden ocultar problemas.
5. [INFERENCIA] **Riesgo comercial y de mantenimiento:** distribución inicial no demuestra compradores; los supuestos base no pagan el coste económico de la amplitud actual.

[HECHO] Evidencia por riesgo, controles mitigantes y reproducer pendientes: [FMEA técnico](fase2-tecnico.md), [mercado](fase3-adopcion.md), [sensibilidad económica](fase4-financiero.md).

## Top 5 recomendaciones accionables

1. [INFERENCIA] **Abrir una tarea específica de crash consistency:** oracle independiente, rotación de uno/varios shards y failpoints apply/Commit; cerrar solo con reproducción o descarte documentado. No aplicar un cambio de orden WAL improvisado.
2. [INFERENCIA] **Ampliar la matriz de paridad a lifecycle y errores:** gate directo, close concurrente, async en vuelo, serialización e IDs grandes; actualizar contratos con los cambios futuros.
3. [INFERENCIA] **Alinear writer/proxy MCP:** probar perfil anunciado, budget efectivo, auth/config por proceso, timeout y saturación; preservar los controles que sí aplica el writer.
4. [INFERENCIA] **Hacer verificable la calidad de release:** evidencia de eval holdout y checks requeridos, skips explícitos, reconciliación de manifiesto/publicación y documentación disponible en clone limpio.
5. [INFERENCIA] **Validar un nicho antes de managed hosting:** caso local Python reproducible, restore e integración mantenida; medir activación/retención voluntaria y contratos de soporte antes de convertir el escenario optimista en presupuesto.

## Wiki e índices: qué se hizo

[HECHO] Se crearon 38 páginas: índice/FAQ; arquitectura; referencias Rust/Python/Node/TS/WASM/HTTP/MCP/paridad; módulos; guías de usuario, desarrollo y contribución. Cada página enlaza fuentes reales; la wiki es una capa explicativa, no sustituye contratos canónicos.

[HECHO] Se corrigieron durante revisión contradicciones de workspace, clase TS actual `Client`, publicación Node, persistencia/copia física, mecanismos de recovery y límites de observabilidad. No se reprodujeron cifras antiguas de performance como garantías actuales.

[HECHO] Generación oficial ejecutada: `node scripts/docs/gen-index.mjs` —dry-run— y `node scripts/docs/gen-index.mjs --write`. El dry-run mostró seis targets posibles y solo dos stale: **`docs/index.md` y `llms.txt`**, ambos permitidos. Los otros cuatro targets y `docs/wiki/index.md` quedaron intactos.

[HECHO] Resultado generado: 219 documentos indexados, 38 nuevos de wiki. Se revisó el diff completo: cambios tracked limitados a esos dos archivos y árbol wiki untracked. Los reportes ignorados no figuran allí.

[HECHO] `llms.txt` utiliza URLs raw de `main` por diseño del generador existente. No se publicó ni se comprobó disponibilidad remota de las páginas nuevas; esos enlaces requerirán la integración/publicación que decida el usuario.

## Limitaciones y controles manuales pendientes

| Estado | [HECHO] Comprobación |
|---|---|
| Realizado | Lectura de código/documentación, censo de texto e historia, consultas primarias de mercado, fórmulas revisadas, revisión editorial/diffs y generación oficial con allowlist |
| No realizado | Tests/builds/arranques, crash reproducers, benchmarks, instalación de artefactos, cobertura runtime o auditoría exhaustiva de seguridad |
| Pendiente | SLOC exacto mediante tokei/cloc, separación estructural de tests inline y crecimiento histórico de stars |
| No disponible | Dogfooding MCP `vantadb`: repetidos fallos al listar herramientas; no se tocaron datos ni se inventó resultado |
| Pendiente del usuario | check-docs, check-links, gen-index --check y markdownlint; no se presentan como PASS |

[INFERENCIA] Desde la raíz del repositorio, ejecutar manualmente y compartir salida/código de retorno:

```powershell
node scripts/docs/check-docs.mjs
node scripts/docs/check-links.mjs
node scripts/docs/gen-index.mjs --check
npx --no-install markdownlint-cli2 "docs/wiki/**/*.md"
git status --porcelain
git diff --stat
```

[HECHO] `--no-install` evita instalar una versión implícita de markdownlint; si no está disponible localmente, esa comprobación queda pendiente, no se modifica configuración para sortearla. Los reportes gitignored pueden requerir inspección explícita, pues los scripts oficiales los omiten.

[INFERENCIA] Si un check falla por contenido preexistente fuera del alcance, registrar el fallo y separar baseline de regresión; no ampliar silenciosamente la misión ni debilitar el gate. La entrega queda **redactada y generada, pendiente de validación manual**, no certificada para release.
