---
title: "Revisión adversarial documental de Mem0, Cognee, Graphiti y MemPalace"
kind: research
status: proposed
description: "Ledger independiente de evidencia, discrepancias y correcciones verificables para cuatro estudios de memoria"
tags: [research, memory, engineering]
---

# Revisión adversarial documental de Mem0, Cognee, Graphiti y MemPalace

## Alcance y criterio

Recibos por proyecto, conservados al terminar cada contraste. Fecha de lectura:
2026-11-07. Revisión independiente de autoría, exclusivamente estática.
Rigen [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md),
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md) y
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md). Vanta de referencia:
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.

«Sustentado» acredita la afirmación documental dentro de la muestra contrastada,
no seguridad, ejecución, paridad de backends ni aprobación de implementación.
Alta: puede falsear una decisión técnica o evaluación; media: contrato/evidencia
insuficiente para derivar una tarea fiable; baja: precisión local sin cambiar
por sí sola la decisión. Ningún veredicto constituye aceptación final.
No se ejecutaron pruebas, aplicaciones, builds, benchmarks ni validadores; no
se iniciaron/detuvieron servicios, no se hizo fetch ni se modificó git.

## Mem0

**Veredicto propuesto: cambios requeridos.** Informe leído íntegro: 764 líneas;
clon `C:\Users\Eros\mem0`, SHA
`f1791daa86d469249c9b002f62644d58a1eb96a0`, conforme al manifiesto.

| Eje | Estado | Sustento o laguna del informe |
|---|---|---|
| 0. Identidad/arquitectura/flujos | sustentado | L15-244, 463-491: separa vector, history, mensajes, entidades y Platform; el código confirma commits separados |
| 1. Recomendaciones | sustentado | L102-113: seis unidades condicionadas; no atribuye dedup ausente a Vanta |
| 2. Fichas Rust | sustentado | L246-459, 468-475: contratos propuestos, alternativas, FMEA, migración y dependencia C/D; no adopción aprobada |
| 3. Incidencias | parcial | L510-545: cuerpos contrastados; fechas de cierre y PR requieren el límite descrito abajo |
| 4. Prerrequisitos Vanta | sustentado | L157-174, 561-568: writer y RecallConfig reales; no promete atomicidad de reemplazo |
| 5. Rechazos | sustentado | L570-576: LLM incondicional, MD5 como seguridad, pesos y extrapolación Platform rechazados |
| 6. Advertencias | sustentado | L578-592, 737-744: muestra/paridad y licencias pendientes explícitas |
| 7. Benchmarks | parcial | L594-661: distingue soporte/QA/recall, pero falta el fallo operativo ocultado por el cliente y preparación efectiva (M1/M2) |
| 8. Matriz | sustentado | L663-674: estados acotados, entidades no acreditadas ≠ ausencia de grafo |
| 9. Negativos propios | sustentado | L676-689: delete-before-put confirmado; lost update sigue hipótesis; LoCoMo conforme a referencia |
| 10. UI/Studio | sustentado | L691-724: dashboard web, paginación cliente y Studio existente correctamente diferenciados |

### Hallazgos

1. **M1 — Alta — fallo de búsqueda convertido en vacío evaluable.**
   [mem0.md L635-652](../mem0.md#cómo-funciona-realmente-el-runner-longmemeval-fijado)
   sigue excepciones del driver, pero no el resultado efectivo del wrapper.
   En el gitlink `4b61c5d31b9c668a12b4f5e78064248a02c82d2b`,
   [`mem0_client.py L254-312,314-343`](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/common/mem0_client.py#L254-L343)
   captura también errores HTTP y devuelve `[]` al agotar reintentos.
   [`run.py L102-140`](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/longmemeval/run.py#L102-L140)
   permite PASS de abstención con memorias vacías. Consecuencia: una búsqueda
   fallida puede llegar al juez como vacío legítimo, no necesariamente cero ni
   excepción; no queda acreditado un resultado medido. Corrección verificable:
   documentar esta cadena y su efecto sobre abstenciones/denominador, distinguiendo
   error capturado en cliente de excepción no capturada en driver. No cambiar scoring.
2. **M2 — Media — preparación de LongMemEval insuficientemente descrita.**
   `mem0.md L619-626,654-661` omite que el runner descarga
   `longmemeval_s_cleaned.json` desde una URL `resolve/main`, ordena por fecha,
   elimina `has_answer`, forma pares consecutivos y omite pares con contenido vacío
   (`run.py L90-94,194-219,279-311,392-436`). No consta split/hash congelado del
   dataset ni modelo de extracción/embedding efectivo del servidor.
   Son diferencias materiales frente a importar sesiones completas en Vanta.
   Corrección: describir estos pasos y declarar las variables de backend/split
   desconocidas; no deducirlas del default `oss` ni de los modelos del juez.
3. **M3 — Baja — símbolo propio mal nombrado.** `mem0.md L166` dice
   `AutoRecallConfig`; la fuente enlazada declara `RecallConfig` en
   `vanta-memory/src/core/hooks/auto_recall.rs L302-334`.
   Corregir el nombre; `core_search=false` sí está sustentado.

### Evidencia contrastada y límites

- Local: `mem0/memory/main.py L314-404,881-1220,1642-1745,1904-1958,2052-2144`;
  `utils/scoring.py L60-139`; `memory/storage.py L150-191`;
  dashboard `server/dashboard/src/app/(root)/dashboard/memories/page.tsx L29-100`.
- Vanta: `core/abstractions/types.rs L71-154`, `core/record/l1_writer.rs L389-456`,
  `core/hooks/auto_recall.rs L302-335`; referencia para contexto/checkpoint/LoCoMo.
- Fuentes públicas leídas: runner y cliente fijados al gitlink; cuerpos y actividad
  de [#3245](https://github.com/mem0ai/mem0/issues/3245) y
  [#4187](https://github.com/mem0ai/mem0/issues/4187), vía `websearch.fetchWebContent`;
  [#6655](https://github.com/mem0ai/mem0/issues/6655), vía `argus.extract_content`.
  API de #6655 por websearch devolvió rate limit; Argus recuperó cuerpo, no metadatos
  completos de cierre. #3245/#4187 muestran cierres vinculados a #4505/#4188;
  no se leyeron sus diffs y no se certifica el fix por el cierre.
- Sin revisión completa de async/TS/proveedores, IAM/HTTP, LoCoMo/BEAM, modelos,
  SBOM ni licencias de datasets. Studio no se ejecutó; su SHA/rangos se conservan
  como evidencia documental y se contrastará la muestra compartida al cierre.
