# INV-{{MODULO}}-01 — Investigación profunda: qué necesita un {{TIPO}} y evaluación integral de `{{MODULO}}`

> **ACTIVE INSTRUCTION — cargada por `/research <módulo>`** con los valores del
> registro `.opencode/references/research-modules.md`. Read-only: ningún fix;
> hallazgos → informe + decisiones (la fase de decisión la orquesta el comando).

## Objetivo
Producir un informe de investigación profunda, detallada y amplia que evalúe
`{{MODULO}}` contra (a) las necesidades reales de sus usuarios objetivo
({{USUARIOS}}) y (b) el estado del arte de {{CATEGORIA_COMPETIDORES}} — para
determinar con evidencia: **qué tiene, qué le falta, qué es mejorable y qué es
optimizable**.

## Alcance
- **Objeto:** `{{MODULO}}` completo — API pública, flujos, empaquetado, tests, docs.
- **Frontera:** NO auditar el core (`src/`) salvo los contratos que {{MODULO}}
  consume vía SDK.
- Modo **read-only**: ningún fix durante la investigación.

## 1. Usuarios objetivo y su flujo diario
{{USUARIOS}}. Para cada uno: cómo descubren/instalan el paquete, qué esperan del
flujo diario, fricciones conocidas (instalación nativa vs wasm, tamaño del binario,
versiones, timeouts), y evidencia real (issues/discussions de GitHub de los
competidores Y nuestros propios issues).

## 2. Estándares del ecosistema {{ECOSISTEMA}}
Convenciones de empaquetado y distribución del ecosistema (registros, CI de release
multiplataforma, semver, tipados/stubs), patrones de DX esperados (quickstart <5min,
manejo de errores tipado), y cambios recientes del ecosistema que nos afecten.

## 3. Competidores — análisis mínimo obligatorio
{{COMPETIDORES}}
Por cada uno documentar: arquitectura y backend, API pública (lista exacta), flujos
principales, DX de instalación/config, performance publicada (**con fuente
reproducible o marcado "claim sin evidencia"** — Regla 11), licencia, actividad,
y adopción (stars/downloads). Competidor principal para diferenciación:
**{{COMPETIDOR_PRINCIPAL}}**.

## 4. Estado actual de `{{MODULO}}` (interna, con evidencia file:line)
- Inventario completo de la API/surface pública exportada.
- Flujos implementados y su calidad (tests que los cubren, gaps).
- Empaquetado/distribución: qué se publica, dónde, CI de release.
- Docs: {{DOC_API}}, README, ejemplos — verificar existencia y frescura.
- Historial: tareas previas de este módulo en Backlog/tasks (BND-/MOD-/FIND-\*).
- Performance: números medidos existentes o ausencia de ellos.

## 5. Framework de evaluación (score 0-10 por dimensión, con evidencia)
DX de onboarding · completitud funcional · performance/overhead · robustez ·
seguridad · docs & ejemplos · observabilidad · testabilidad · paridad con otros
módulos VantaDB (misma feature en otro binding) · **diferenciación**
(¿por qué elegirnos sobre {{COMPETIDOR_PRINCIPAL}}?).

## Entregables → `docs/reviews/research-{{MODULO}}-<YYYYMMDD>.md`
1. Matriz competencia: features × productos (incluyendo `{{MODULO}}`).
2. Gap analysis priorizado: falta (P0/P1/P2) · mejorable · optimizable.
3. Quick wins (<1 día) vs apuestas estratégicas (>1 semana).
4. **Inventario de hallazgos (APÉNDICE OBLIGATORIO):** TODO hallazgo — incluso los
   menores — con ID secuencial `H-NN`, categoría sugerida
   (APLICAR/MEJORAR/AGREGAR/OPTIMIZAR/ESTRATEGIA/DESCARTAR), severidad, esfuerzo
   estimado (🟢🟡🔴) y `file:line`. Este apéndice alimenta la Fase D del comando
   (decisiones por hallazgo) — **sin él hay pérdida de datos**.
5. Recomendaciones → filas FIND-\* en Backlog según decisión del usuario (Fase D).
6. Claims de performance solo con benchmark reproducible citado (Regla 11).

## Método y fuentes
- Internet: repos/issues/releases de competidores, npm/PyPI/crates.io stats,
  changelogs de ecosistema.
- Interno: `codegraph_explore` de {{MODULO}}, sus tests, Backlog histórico,
  `campaign_memory_read(lessons|decisions)`.
- Skill Discovery (SDP): `source-driven-development`, `coordinated-web-search`;
  formato per-module de referencia: `review-deep`.
- RESULTADO final con SKILLS_CARGADAS y GATES_EVALUADOS.
