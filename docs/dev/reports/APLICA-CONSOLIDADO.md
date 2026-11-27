---
title: "Lo que sigue aplicando — consolidado de docs/dev/reports"
kind: report
status: active
description: "Destilado de los 8 reportes de la carpeta: solo lo vigente/útil para VantaDB a 2026-11-26, con lo obsoleto condensado a una línea"
tags: [reports, consolidado, vigencia]
---

# Lo que sigue aplicando — consolidado de `docs/dev/reports/`

> Corte de vigencia: **2026-11-26**. Cada fuente se destila en puntos precisos:
> **vigente** (aplica hoy) vs **obsoleto** (condensado a una línea; el original
> permanece en disco como evidencia histórica — esta carpeta es gitignored,
> borrar originales es irreversible y requiere decisión expresa del owner).

---

## 1. `2026-11-06-analisis-integral/` (5 archivos, 918 líneas)

### Vigente

**Durabilidad (fase2 — la parte accionable):**

1. P0 WAL: segmentos rotados antes de checkpoint completo es escenario de riesgo de replay; contrato de validación = umbral pequeño, multi-shard, crash post-rotación. **Converge con C1/C13/C14 de extracción** — mismo defecto redescubierto 3 semanas después. Verificar si la oleada FIND-7xx lo cerró.
2. P1: crash entre aplicación parcial de stores y commit durable → estado físico no committed. **No mover Commit antes de apply de forma simplista** (traslada el problema a reintentos/visibilidad).
3. Matriz lifecycle/errores FFI pendiente: close idempotente, operación en vuelo, uso tras cierre.
4. Cascada de blast radius: tocar WAL arrastra init/checkpoint/shipping/cifrado; tocar FFI arrastra clientes publicados.
5. Riesgo de deriva de superficie en proxy MCP (perfil anunciado vs comportamiento temporal).

**Adopción (fase3):**

6. Disciplina de evidencia: stars ≠ usuarios, descargas ≠ producción; adquisición/retención no medidas se reportan como desconocidas.
7. La retención es la variable dominante del negocio (r=90%→99 pagadores a 24m; r=97%→187 — casi se duplica).
8. 40 pagadores simultáneos = equilibrio operativo.

**Financiero (fase4):**

9. Aritmética reusable mientras precio/coste no cambien: $4.000 fijo/mes ÷ $100 margen = 40 pagadores break-even.
10. Recuperar reposición ≠ pagar operación: ~195 pagadores promedio durante 24 meses para reponer desarrollo de referencia.
11. Coste hundido: el desarrollo pasado no debe condicionar la decisión marginal de continuar.
12. Recomendación final vigente: financiar calidad de recuperación/contratos y compradores de soporte antes que plataforma managed.

### Obsoleto (condensado)

- Censo fase1 (1.455 archivos, 438k líneas, ratios): inválido tras +20 días de commits.
- "Node planeado/no publicado": resuelto en su momento (npm 0.9.0).
- Versión de referencia 0.9.0 y escenarios de publicación: superseded por v0.10.0.

---

## 2. `2026-10-19-systems-and-ux-analysis.md` (186 líneas)

### Vigente

1. **Mapa de arquitectura por capas** (clientes → puertas MCP/HTTP/proxy → capa cognitiva L1-L3 → motor → persistencia): sigue siendo la descripción estructural correcta.
2. **Fricción de onboarding real**: si el usuario no persiste `ORT_DYLIB_PATH`/`VANTADB_LOCAL_MODEL` en la terminal correcta, el motor degrada silenciosamente a *dummy embeddings* — degradación silenciosa de calidad semántica es un defecto de diseño UX que merece guarda explícita.
3. **Comparativa de modos de despliegue** (embebido <0.5ms / server 1.5-3ms / MCP 5-15ms): sigue siendo la guía de elección correcta.
4. Fricción Node: fallback nativo→WASM degrada a memoria/OPFS sin aviso fuerte.
5. Su FMEA y dictamen fueron consumidos por el plan de remediación 2026-10-19 (FIND-381..385): verificar cierre antes de re-actuar sobre ellos.

### Obsoleto

- Referencias de versión v0.8.0/v0.9.0 y MSRV 1.94.1: superseded.
- DX-04 (confusión registro/nodo): declarado resuelto en 0.9.0 por el propio doc.

---

## 3. `dora.md` (182 líneas)

### Vigente

1. **El método**: medir cycle time por tipo de tarea, CFR, recovery time y throughput desde task files con fechas derivables — instrumento de mejora de proceso reusable.
2. Sus limitaciones declaradas son honestas y aplican a cualquier re-medición futura.

### Obsoleto

- Todas las cifras concretas: son un corte de proceso de hace semanas; el flujo de ejecución cambió radicalmente (harness hardening 2026-11-26, H-02..H-07). **Re-generar el reporte es más barato que leerlo.**

---

## 4. `INDEX.md` (94 líneas)

### Vigente

1. Es el **ledger de auditorías con estado por fila** (vigente/superado/archivado) — su función índice sigue viva y lo mantienen las skills `unified-review`/`progreso`. No tocar.

### Obsoleto

- Nada como documento; las filas históricas son su contenido correcto, no ruido.

---

## Reglas de uso de este consolidado

1. Los originales quedan intactos como evidencia; este archivo es la capa de lectura rápida.
2. Todo punto "vigente" de durabilidad requiere verificación contra HEAD antes de actuar (los cortes son snapshots).
3. Si un original se borra, su sección aquí queda como única síntesis — decidir borrado solo tras confirmar que el punto vigente está canalizado en backlog o en FIND/C activo.
