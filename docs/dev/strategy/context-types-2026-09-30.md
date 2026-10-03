---
title: "Los 5 tipos de contexto (investigación GTM 2026-06, rescatada)"
kind: concept
status: active
description: "Framework de posicionamiento rescatado del OLD: Ephemeral / Session / Project / Entity / Episodic — los 5 tipos de contexto que toda herramienta de IA necesita."
tags: [vantadb, strategy, gtm, contexto]
---

# Los 5 tipos de contexto — Patrón Universal (rescatado 2026-09-30)

> **Origen:** `VantaDB_Investigacion_Contexto_GTM.md` §"Análisis Transversal" (2026-06-13, OLD) — análisis de 21 herramientas de IA. Rescatado 2026-09-30. Contexto actual: `docs/user/AI_IDES.md`, `PRIVACY.md`, `FRAMEWORKS.md` (3 tracks ICP).

**El problema universal (Context Engineering):** la información que un agente aprende en una sesión se pierde al cerrarla; ninguna herramienta maneja los 5 tipos de contexto de forma completa.

| Tipo de Contexto | Qué es | Storage típico actual | VantaDB |
|---|---|---|---|
| **Ephemeral** | Conversación activa, tokens en window | RAM / en-process | TTL + eviction |
| **Session** | Estado entre reintentos dentro de la misma tarea | SQLite / PostgreSQL | WAL checkpointer |
| **Project** | Decisiones de arquitectura, convenciones | Markdown estático | Semantic search + graph |
| **Entity** | Personas, sistemas, conceptos y sus relaciones | ChromaDB (solo vectores) | GraphRAG nativo |
| **Episodic** | "¿Qué hicimos la semana pasada en el módulo X?" | Git history (sin semántica) | Hybrid search temporal |

> *"VantaDB es el primer storage que maneja los 5 tipos en un único store transaccional."* — claim de posicionamiento de la investigación; **validar contra la frontera de producto (DEF-01..08) y la Regla 11 antes de usarlo en público**.

**Usos sugeridos:** base para un post del blog (`BLOG_SERIES_PLAN.md`) + sección "Why VantaDB" en web (pendiente de validación de claims).
