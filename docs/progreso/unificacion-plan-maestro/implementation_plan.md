# Reconstrucción del Plan Maestro Unificado de VantaDB

## Contexto del Problema

El archivo `VantaDB_Plan_Maestro_Unificado.md` actual tiene **131 líneas / 12KB**, lo cual representa ~5% del contenido de los **7+ documentos fuente** que suman ~253KB. Se perdieron tareas funcionales concretas, IDs de trazabilidad, métricas de validación, subtareas detalladas, matrices de riesgo y roadmaps temporales.

## Diagnóstico: ¿Qué falta en el unificado actual?

### Tareas funcionales omitidas (por fuente)

| Fuente | Tareas omitidas clave |
|---|---|
| **Plan Deepseek (38KB)** | ARQ-03 (eliminar experimental), ARQ-04 (backend abstraction), ARQ-05 (config unificada), COD-02 (rkyv zero-copy), COD-03 (jemalloc), COD-04 (linting), COD-05 (gestión errores), TST-01/02/03/04, SEC-03/04/05, OPS-01/02/03/04, DB-03/04, PD-01/02/03, MKT-01/02, ORG-01/02/03, SCA-01/02/03, + 20 tablas de riesgo y matrices |
| **Plan Qwen (37KB)** | CODE-01 (telemetría memoria), DEVOPS-01 (CI enterprise), DEVOPS-02 (SLOs), DB-01 (backend tuning), DB-02 (backup/restore), PROD-01 (SDK Python PyPI), STRAT-01 (posicionamiento), ORG-01 (ADRs/mdbook) |
| **Plan Antigraviti (17KB)** | CODE-001 (I/O bloqueante detallado), QA-001 (Jepsen/Maelstrom), DB-001 (Write Amplification LSM), PROD-001 (DX zero-friction), MKT-001 (moat tecnológico), ORG-001 (tribal knowledge), FUT-001 (multi-tenancy serverless) |
| **Plan Maestro Ejecutivo (96KB)** | T0.1-T0.4 (estabilización post-cuarentena), T1.1-T1.5 (HNSW scalability), T2.1-T2.4 (hardening arquitectónico), T3.1-T3.4 (validación producción), T4.1-T4.4 (community launch), T5.1-T5.2 (pre-seed), + mejoras paralelas completas |
| **Roadmap v0.2 (12KB)** | Fases 2-5 (Mmap zero-copy, IQL AST, lock-free, open-core), Fases 7-10 (MCP, cuantización, replicación, GTM) |
| **Deep Research (21KB)** | Análisis de perfilado con comandos concretos, comparativa vs SQLite/Qdrant/Weaviate/Tantivy, backlog de 12 semanas |
| **CSV Seguimiento (32KB)** | ~70 tareas de gobierno, SRR, PDR, CDR con fechas y dependencias |

### Elementos estructurales faltantes

1. **IDs de tarea** — El unificado no tiene IDs trazables (ARQ-01, COD-01, etc.)
2. **Criterios de aceptación** — Solo tiene párrafos generales, no condiciones verificables
3. **Matrices** — Falta Impacto vs Esfuerzo detallada, FMEA por área
4. **Quick Wins** — No hay sección de cambios de <1 día
5. **Áreas de eliminación** — Lista genérica vs lista con paths específicos
6. **Bottlenecks futuros** — Omitidos completamente
7. **Costos ocultos** — Omitidos completamente
8. **Plan Enterprise** — Omitido completamente

## Propuesta: Estructura del nuevo Plan Maestro Unificado

El nuevo documento se organizará así:

```
1. Encabezado y estado del proyecto (commit, versión, posicionamiento)
2. Mapa visual de fases con cronograma Gantt
3. Catálogo de tareas por fase (con IDs, subtareas, criterios, métricas)
   - Fase 0: Estabilización (quick wins, linting, formato, cleanup)
   - Fase 1: Desacoplamiento Core/Server
   - Fase 2: Query Planner AST/IR
   - Fase 3: WAL Hardening y Checkpoints
   - Fase 4: Observabilidad, SRE y Concurrencia
   - Fase 5: Seguridad Enterprise
   - Fase 6: DX, CI/CD y Release Engineering
   - Fase 7: Escalabilidad (Post-GA)
4. Pistas paralelas (mejoras no bloqueantes)
5. Matrices de decisión
   - Impacto vs Esfuerzo
   - Quick Wins (<1 día)
   - Bottlenecks futuros
   - Costos ocultos
   - Riesgos por categoría (técnico, mercado, seguridad, operacional, organizacional)
6. Áreas de eliminación (paths exactos)
7. Áreas de refactor (con esfuerzo estimado)
8. Plan de preparación Enterprise
9. Sección de descarte (FMEA) — ya existe, se mantiene
10. Plan de verificación — ya existe, se expande
```

> [!IMPORTANT]
> **Criterio de completitud:** Cada tarea funcional de código que aparezca en *cualquiera* de los documentos fuente y no esté ya completada, debe tener una entrada en el nuevo unificado con: ID, fase, subtareas, criterio de aceptación y métricas.

## Preguntas abiertas para el usuario

> [!IMPORTANT]
> 1. **¿Debo incluir las tareas de gobierno/gestión del CSV** (SRR, PDR, CDR, revisiones semanales) o solo las tareas de código funcional?
> 2. **¿Conservar los IDs originales** de cada plan (ARQ-01, CODE-01, T0.1, etc.) como referencia cruzada, o asignar IDs nuevos unificados?
> 3. **Las tareas de marketing y community launch** (HackerNews, Discord, blog posts, deck inversores) del Plan Ejecutivo — ¿las incluyo en el unificado o van a un documento separado?

## Verificación

- Comparar línea por línea cada tarea de cada documento fuente contra el unificado resultante
- Ninguna tarea funcional de código se pierde
- Las tareas ya completadas se marcan como `[COMPLETADA]` en vez de eliminarse
