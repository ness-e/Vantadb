---
title: "Fase 4 — Costes, monetización y equilibrio"
kind: report
description: "Modelo financiero en USD a 24 meses, coste de reposición y equilibrio operativo separados del gasto histórico no conocido."
---

# Fase 4 — Costes, monetización y equilibrio

[HECHO] Moneda: **USD nominales**. Corte 2026-11-06; horizonte 24 meses hasta 2028-11-06. No se dispuso de contabilidad, facturas, nóminas, contratos, presupuesto comercial ni export de consumo CI. **El gasto histórico real y el valor de empresa son desconocidos.**

[INFERENCIA] El modelo estima coste de reposición y viabilidad bajo supuestos. No es auditoría financiera, valoración de propiedad intelectual, precio de venta, gasto efectivamente incurrido ni recomendación de inversión.

## 1. Coste de desarrollo acumulado: proxy de reposición

[HECHO] La [fase 1](fase1-estadisticas.md) mide líneas no vacías, no SLOC. Para no introducir un COCOMO con parámetros no calibrados, se usa un modelo transparente **LNV × coste unitario**, descompuesto en productividad y coste-hora.

[HECHO] Universo del proxy: extensiones de código, rutas de `src/`, `vanta-memory`, `vanta-proxy`, `vantadb-ffi-core`, `vantadb-mcp`, cuatro bindings/wrapper, `vantadb-server`, `providers`, `integrations`, excluyendo rutas clasificadas como tests y generado identificado. Suma congelada: **198.947 LNV**. Incluye comentarios y tests inline; excluye suites externas, docs, evals y tooling. No es un inventario de «198.947 líneas productivas» ni se añade por separado el coste de cada superficie.

[ESPECULACIÓN] Productividad expresa LNV de ese inventario por hora de **ciclo de desarrollo completo**, incluyendo esfuerzo no representado en líneas: diseño, revisión, tests externos, documentación y retrabajo. Son parámetros elegidos, no benchmarks laborales, tarifas observadas ni estimaciones del tiempo humano real. No requieren una fuente salarial externa porque no se presentan como datos de mercado.

```text
Horas equivalentes H = L / v
Coste de reposición C_dev = (L / v) × w
L = 198.947 LNV
v = productividad equivalente, LNV/h
w = coste-hora fully loaded supuesto, USD/h
```

| Caso de coste | [ESPECULACIÓN] v | [ESPECULACIÓN] w | [INFERENCIA] Horas | [INFERENCIA] C_dev |
|---|---:|---:|---:|---:|
| Reposición eficiente | 80 LNV/h | $40/h | 2.486,84 | **$99.473,50** |
| Reposición de referencia | 40 LNV/h | $75/h | 4.973,68 | **$373.025,63** |
| Reposición intensiva | 20 LNV/h | $120/h | 9.947,35 | **$1.193.682,00** |

[INFERENCIA] El rango amplio es deliberado: automatización/IA, reutilización, código generado no detectado, tests inline, complejidad de storage y deuda pueden cambiar radicalmente productividad. El caso de referencia equivale a 31,09 meses-persona de 160h, **no** a 31 meses calendario ni a salarios pagados.

[HECHO] La historia local contiene 4.250 commits y cinco identidades de autor, dos de bots explícitos. [INFERENCIA] No se convierten commits en horas ni identidades en empleados; tampoco se usa churn como desarrollo neto. La fecha de primer commit no acredita trabajo continuo.

[INFERENCIA] Antes de usar el coste para una decisión de financiación, sustituir LNV por inventario funcional/SLOC validado y contrastar horas reales, entregables reutilizados y porcentaje de automatización. Un coste alto de reposición no demuestra demanda ni disposición a pagar.

## 2. Operación futura: coste económico frente a caja

### Referencias externas verificadas

[HECHO] [GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions), consultado 2026-11-06, indica gratuidad de standard hosted runners en repositorios públicos; larger runners se cobran también en públicos. Self-hosted no genera ese cargo de runner de GitHub, pero su infraestructura no es gratuita por ello.

[HECHO] [Actions runner pricing](https://docs.github.com/en/billing/reference/actions-runner-pricing), misma fecha: Linux x64 2-core $0,006/min, Windows x64 2-core $0,010/min y macOS base $0,062/min. Son referencias tarifarias actuales, no una factura de VantaDB.

[HECHO] La [API del repositorio](https://api.github.com/repos/ness-e/Vantadb) devolvió `private=false` y `visibility=public`. [INFERENCIA] Puede resultar elegible para gratuidad de runners estándar, pero eso no prueba que todos los jobs lo sean ni que la factura total de almacenamiento/cache/larger/self-hosted sea cero.

[ESPECULACIÓN] Ejemplo de coste equivalente si se facturaran 5.000 min Linux + 600 Windows + 300 macOS: `5000×0,006 + 600×0,010 + 300×0,062 = $54,60/mes`. Los minutos **no son consumo medido**. Se conserva la ecuación para reemplazarlos por export de billing, redondeo por job y allowances aplicables.

### Presupuesto económico mensual usado en escenarios

[ESPECULACIÓN] No hay hosting administrado ni SKU comercial comprobado en este análisis. Se modela una operación futura reducida, no un presupuesto de infraestructura ya desplegada:

| Coste fijo mensual | Supuesto | USD/mes |
|---|---|---:|
| [ESPECULACIÓN] Mantenimiento de motor/bindings/seguridad | 40h × $75/h | 3.000 |
| [ESPECULACIÓN] Docs, releases y operación | 8h × $75/h | 600 |
| [ESPECULACIÓN] Infra común, CI no cubierto, docs, distribución y herramientas | Reserva conjunta; no factura ni tarifa de registry | 400 |
| [INFERENCIA] **Fijo económico F** | Suma | **4.000** |

[ESPECULACIÓN] Dentro de la reserva de $400 se asignan $100 a CI/artefactos, $50 a docs/dominio/CDN, $50 a distribución/firmas/registries y $200 a backups/monitorización/herramientas. Son límites de planificación, no cargos de npm/PyPI/crates.io ni afirmación de que la descarga pública se cobre por unidad. No se suman otra vez al fijo.

[INFERENCIA] Conservar distribuciones públicas y docs estáticos puede reducir desembolso. El ahorro de infraestructura no elimina las 48h/mes imputadas de mantenimiento/operación. Incluso ese esfuerzo puede ser insuficiente para la amplitud de superficies de este workspace; el modelo no presupone un SLA 24×7.

## 3. Monetización compatible con el open-core

[HECHO] Restricciones locales consultadas: `.opencode/rules/open-core-licensing.md` y `docs/dev/plans/archive/2026-08-06-oc-vantadb-pro.md`. Core Apache-2.0; pro fuera del workspace y distribuido como artefactos compilados; validación de licencia offline por nodo, expiración y límite de nodos. Son reglas de diseño, **no prueba de un negocio o licenciamiento ya operativo**.

[HECHO] WAL shipping, PITR, cifrado, Prometheus, servidor y TLS ya pertenecen al core abierto según ese marco. [INFERENCIA] No se recomiendan como paywall retroactivo ni se propone moverlos de licencia en esta tarea.

| Canal propuesto | [INFERENCIA] Qué se cobra | [INFERENCIA] Riesgo/condición |
|---|---|---|
| Soporte e integración | Tiempo, respuesta acordada, migración y capacitación | Definir horas incluidas; no confundir soporte con garantía absoluta de datos |
| Managed hosting | Operación, backups verificados, observabilidad, upgrades, aislamiento | Requiere servicio, seguridad, on-call y costes unitarios demostrados; no existe evidencia suficiente aquí para vender SLA |
| Enterprise/pro separado | Capacidades nuevas explícitamente reservadas, governance/operación empresarial permitida por las reglas | Revisión humana del límite core/pro y de contratos; no duplicar ni retirar features Apache existentes |

[ESPECULACIÓN] Para comparar escenarios se emplea un **ticket mensual blended hipotético de $129**, no precio publicado ni revenue actual, y variable `c=$29` por pagador-mes. Es una abstracción de una mezcla futura de soporte/servicio; no supone que todos los usuarios OSS paguen por ejecutar core.

[ESPECULACIÓN] Descomposición del variable $29: infra/almacenamiento $10, presupuesto acotado de inferencia $2, soporte 0,2h×$75=$15 y cobros/contingencia $2. No son tarifas verificadas de proveedor; deben sustituirse con mediciones. El gasto LLM real podría superar ampliamente $2 si se subsidia uso sin límites.

[INFERENCIA] Margen de contribución unitario `m=129−29=$100`, o 77,52% del ingreso, **antes del fijo, impuestos, adquisición, desarrollo comercial adicional, seguros y financiación**. No se presenta como margen neto.

## 4. Cruce con adopción a 12/24 meses

[ESPECULACIÓN] Se reutilizan exactamente Q/a/r/p y la definición de activo de [fase 3](fase3-adopcion.md). Se asume facturación del mes completo a `P_t`, pago puntual, sin desfase de conversión, descuentos ni bad debt. Esto es favorable a ingresos y debe reemplazarse al disponer de cohortes.

```text
Ingresos_t = 129 × P_t
Variable_t = 29 × P_t
Resultado_operativo_t = 100 × P_t − 4.000
Resultado_operativo_acum(T) = Σ(100 × P_t − 4.000), t=1..T
Resultado_con_reposición(T) = Resultado_operativo_acum(T) − C_dev
```

| Escenario | [INFERENCIA] MRR mes 12 | [INFERENCIA] MRR mes 24 | [INFERENCIA] Resultado mes 24 | [INFERENCIA] Operativo acumulado 24m |
|---|---:|---:|---:|---:|
| Conservador | $118,03 | $134,82 | −$3.895,49 | **−$94.032,21** |
| Base | $1.332,96 | $1.709,43 | −$2.674,86 | **−$73.366,23** |
| Optimista | $12.169,13 | $17.960,67 | $9.923,00 | **$117.873,04** |

| Escenario | [INFERENCIA] Pagador-meses 24m | [INFERENCIA] Ingresos 24m | [INFERENCIA] Variable 24m | [INFERENCIA] Fijo 24m |
|---|---:|---:|---:|---:|
| Conservador | 19,68 | $2.538,44 | $570,66 | $96.000 |
| Base | 226,34 | $29.197,57 | $6.563,79 | $96.000 |
| Optimista | 2.138,73 | $275.896,22 | $62.023,18 | $96.000 |

[INFERENCIA] Equilibrio mensual operativo: `F/m = 4000/100 = 40 pagadores` simultáneos. El optimista cruza ese umbral en el **mes 5 del modelo**; conservador y base no lo cruzan en 24 meses. No es una fecha prometida ni evidencia de ventas.

[INFERENCIA] El resultado acumulado a 12 meses es −$47.238,47 / −$40.019,71 / $20.209,53, respectivamente. Un mes rentable, recuperar las pérdidas de arranque y recuperar desarrollo son tres preguntas distintas.

### Recuperar el desarrollo de reposición no es pagar la operación

[INFERENCIA] Restando el coste de reposición de referencia, $373.025,63, el resultado de 24 meses queda en **−$467.057,84 / −$446.391,85 / −$255.152,58**. Ningún escenario recupera esa imputación dentro del horizonte.

[INFERENCIA] Con la reposición eficiente de $99.473,50, el optimista dejaría $18.399,54 tras operación y reposición; conservador y base seguirían negativos. Con reposición intensiva de $1.193.682, el optimista seguiría en −$1.075.808,96.

[INFERENCIA] Para recuperar reposición de referencia y fijo de dos años se necesitan `(373025,625 + 96000) / 100 = 4690,25625 pagador-meses`, promedio **195,43 pagadores durante los 24 meses**. No basta terminar el mes 24 con ese número: el acumulado depende de la rampa.

[INFERENCIA] El desarrollo pasado, si es coste hundido, no debería condicionar por sí solo la decisión marginal de continuar. Se muestra su recuperación por solicitud del análisis, pero la decisión futura exige demanda, riesgos y retorno sobre inversión adicional, no «recuperar cada línea escrita».

## 5. Sensibilidad y costes omitidos

| Cambio aislado | [INFERENCIA] Margen unitario | [INFERENCIA] Pagadores para equilibrio mensual | [INFERENCIA] Promedio 24m para recuperar referencia + fijo |
|---|---:|---:|---:|
| Ticket $79, variable $29 | $50 | 80 | 390,85 |
| Ticket $129, variable $29 | $100 | 40 | 195,43 |
| Ticket $199, variable $29 | $170 | 24 redondeado arriba | 114,96 |
| Ticket $129, variable $49 | $80 | 50 | 244,28 |

[INFERENCIA] La tabla mantiene volumen de clientes fijo al variar precios **solo para aislar aritmética**; en la realidad precio y conversión están relacionados. No debe interpretarse que subir precios conserva la demanda.

[ESPECULACIÓN] Si el fijo económico subiera a $8.000/mes, con el margen base harían falta 80 pagadores. Si el soporte por cliente necesitara 1h en lugar de 0,2h a $75, el variable aumentaría $60 y el margen caería de $100 a $40: equilibrio 100 pagadores. La venta de soporte sin límites es especialmente arriesgada con un mantenimiento concentrado.

[HECHO] No se dispone de CAC, LTV, cohortes comerciales, churn de pago independiente, costes de incidentes ni uso LLM real. [INFERENCIA] Las omisiones hacen que los resultados no sean caja libre ni beneficio neto; tampoco permiten calcular runway real o valoración.

[INFERENCIA] La recomendación sostenible es financiar primero calidad de recuperación/contratos y validar compradores de soporte; posponer una plataforma managed amplia hasta medir costes y responsabilidad operativa. Un hosting prematuro convierte incertidumbre de durabilidad y concurrencia en riesgo contractual.

## 6. Reproducción aritmética

[INFERENCIA] Se calcularon los escenarios con Decimal, sin redondeo intermedio; solo se redondean importes al mostrarlos. Esta receta no ejecuta VantaDB ni consulta sistemas privados:

```python
from decimal import Decimal as D

escenarios = (
    ("conservador", 100, "0.08", "0.85", "0.02"),
    ("base", 300, "0.12", "0.90", "0.04"),
    ("optimista", 1000, "0.18", "0.94", "0.06"),
)
for nombre, q, a, r, p in escenarios:
    activos = D(0)
    resultado = D(0)
    pagador_meses = D(0)
    for mes in range(1, 25):
        activos = D(r) * activos + D(q) * D(a)
        pagadores = D(p) * activos
        pagador_meses += pagadores
        resultado += D(100) * pagadores - D(4000)
        if mes in (12, 24):
            print(nombre, mes, pagadores * D(129), resultado)
    print(nombre, pagador_meses, resultado - D("373025.625"))
```

[HECHO] Navegación: [reporte maestro](REPORT.md) · [estadísticas](fase1-estadisticas.md) · [técnico](fase2-tecnico.md) · [adopción](fase3-adopcion.md).
