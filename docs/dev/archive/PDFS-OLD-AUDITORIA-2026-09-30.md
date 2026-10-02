---
title: "Auditoría PDFs OLD — candidatos y procedencia (2026-09-30)"
kind: research
status: archived
description: "Registro de la extracción/lectura de los 8 PDFs históricos de OLD/_pdf-extracted (E1): los 15 candidatos derivados con su destino y las fichas de procedencia/identidad de los documentos dudosos"
tags: [vantadb, archive, old-docs, procedencia]
---

# Auditoría PDFs OLD — candidatos y procedencia (2026-09-30)

> **Origen:** tarea E1 de la ejecución de acciones OLD (2026-09-30). Los PDFs viven **fuera del repo** (`C:\Users\Eros\VantaDB Proyect\OLD\`); se extrajeron a texto con `pypdf` en `OLD/_pdf-extracted/` (8 archivos + `extract.py`) porque los agentes no leen PDF directo. Lectura completa por 3 agentes (≈220 páginas) → **15 candidatos** nuevos, aprobados por el owner el 2026-09-30 y registrados en los backlogs.

## 1. Los 8 documentos (procedencia)

| Archivo (extraído a .txt) | Origen | Fecha aprox. | Estado / valor |
|---|---|---|---|
| `VantaDB_Auditoria_Tecnica.txt` | Auditoría técnica externa (develop branch) | 2026-07 | ✅ Fuente de candidatos (ALT-06 NL_POOL, CRIT-03 ScannIndex, MED-18 runbook, capabilities IVF/ScaNN/DiskANN, jemalloc/mimalloc) |
| `VantaDB_AnalisisTecnico_BusinessProfessional_2026-07-26.txt` | Análisis técnico + negocio v1.0 (autor: ness-e/Vantadb) | 2026-07-26 | ✅ Fuente de candidatos (R2/R5/R7/R8/R9 — 15 recomendaciones R1-R15) |
| `Auditoría Técnica Profunda de Proyecto - Google Gemini.txt` | Análisis generado con Gemini | 2026-07 | ✅ Fuente de candidatos (P1 jemalloc/mimalloc, P1 telemetría/tracing, Q1 OpenTelemetry) |
| `VantaDB_Plan_Maestro_Redireccion_2026-04-13.txt` | Plan maestro de redirección | 2026-04-13 | Histórico — procesado en su momento (referencia) |
| `Hoja de Ruta Técnica para VantaDB_ Modularización...txt` | Hoja de ruta técnica (modularización server/planner) | 2026 | Histórico — direcciones cubiertas por P26/P39/WIRE-07 (referencia) |
| `VantaDB_PRD_Roadmap_90dias_Backlog.docx.txt` | PRD/roadmap 90 días (docx→pdf→txt) | 2026 | Histórico — referencia |
| `De la Intuición al Dominio_ Una Guía Arquitectónica...txt` | Documento conceptual "deep research" (LLM, con footnotes) | s/f | ⚠️ **Huérfano con errores de identidad — ver §3.1** |
| `Vanta DB_ Un Veredicto de Ingeniería Crítica...txt` | Análisis de "Vanta DB" (empresa distinta) | s/f | ❌ **N/A — es de otra empresa (Vanta Inc) — ver §3.2** |

## 2. Los 15 candidatos (destino registrado)

| # | Candidato | Destino | Fila |
|---|---|---|---|
| 1 | `NL_POOL` sin cap — leak en threads efímeros (`src/index/search/pool.rs`) | `docs/dev/Backlog.md` | FIND-213 |
| 2 | ScannIndex: 5 mutex anidados sin orden documentado (riesgo deadlock) | `docs/dev/Backlog.md` | FIND-214 |
| 3 | Persistencia de índices auxiliares (DiskANN disk-I/O + ScaNN) | `docs/dev/backlog-futuro.md` | FUT-14 (ampliada; IVF ✅ serializa — residual = seed) |
| 4 | IVF: semilla hardcodeada (42) — exponer/configurar | `docs/dev/backlog-futuro.md` | FUT-24 |
| 5 | jemalloc/mimalloc: evaluar default (features ya existen) | `docs/dev/Backlog.md` | FIND-215 |
| 6 | OTel/tracing: spans del ciclo de query (hoy solo `http_request`) | `docs/dev/Backlog.md` | FIND-216 |
| 7 | IR runbook: residual (DR runbook ya cubre severidades/escalación/post-mortem) | `docs/dev/Backlog.md` | FIND-217 |
| 8 | R5: política de estabilidad/deprecación de API (`docs/api-stability.md` no existe) | `docs/dev/Backlog.md` | FIND-218 |
| 9 | R7: firmar binarios Windows (Authenticode) | `docs/dev/Backlog-negocio.md` | BIZ-16 |
| 10 | R2: matriz de combinaciones de features (`docs/feature-matrix.md` no existe) | `docs/dev/Backlog.md` | FIND-219 |
| 11 | R8: auditoría externa de seguridad | `docs/dev/Backlog-negocio.md` | BIZ-17 |
| 12 | Ficha de procedencia del PDF "Intuición" | este archivo | §3.1 |
| 13 | R9: telemetría mínima por defecto + `dump_diagnostics()` | `docs/dev/Backlog.md` | FIND-220 |
| 14 | Capa semántica unificada (guía "Intuición") | `docs/dev/backlog-futuro.md` | FUT-25 |
| 15 | Nota de procedencia de los PDFs (este archivo) | este archivo | §1 |

## 3. Fichas de documentos dudosos

### 3.1 "De la Intuición al Dominio" — huérfano con errores de identidad

Documento de análisis conceptual en formato "deep research" (footnotes numeradas + URLs de artículos genéricos de arquitectura), **sin autor ni fecha verificables** (huérfano). Describe a VantaDB de forma idealizada y **contiene detalles fabricados o de otro estado del proyecto**: p. ej. una "arquitectura en capas L0-L3" con `FAISS`/`Nebula Graph`/`SQLite`/`Elasticsearch` como tecnologías del proyecto (VantaDB no usa ninguna de esas) y "SDKs en Python, Go, Java, TypeScript" (Go/Java no existen). **NO usar como fuente de verdad**; valor únicamente conceptual (de ahí se rescató el candidato #14 — FUT-25). No se archiva como doc del repo; queda referenciado acá.

### 3.2 "Vanta DB: Un Veredicto de Ingeniería Crítica" — N/A (otra empresa)

El documento describe a **Vanta Inc** (plataforma de compliance/gestión de riesgo de proveedores; MongoDB Atlas; Serie D de $150M; valuación $4.15B) — **no es VantaDB**. Ningún hallazgo aplica. Descartado completo (N/A). **No re-proponer.**

## 4. Candidatos NO registrados (descartados por verificación HEAD 2026-09-30)

- SECURITY.md policy / SUPPORT.md / migration guides ChromaDB-LanceDB / Python API reference — **ya existen** en el repo (el audit 2026-07 quedó stale en esos puntos).
- IR runbook completo — cubierto por `docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md` (severidades §1, escalación §7, post-incident §5); residual mínimo en FIND-217.
- IVF "sin persistencia" — stale: `serialize_to_bytes` wired en `src/index/core.rs` (residual = FUT-24).

---

> **Registro de la migración de formato:** los backlogs migraron al esquema canónico de 10 columnas (`.opencode/references/backlog-format.md`) el 2026-09-30; detalle en `docs/dev/avance/activo/operaciones.md`.
