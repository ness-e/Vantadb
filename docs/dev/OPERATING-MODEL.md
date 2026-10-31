---
title: "Modelo Operativo del Sistema VantaDB"
kind: research
status: active
description: "Modelo operativo y marco de gobernabilidad técnica de VantaDB: posicionamiento STRAT-01 ('SQLite para agentes'), arquitectura y ciclo de vida"
tags: [operating-model, architecture, strat-01, governance, business]
---

# Modelo Operativo del Sistema VantaDB

## 1. Posicionamiento Estratégico y Núcleo de Producto (STRAT-01)

Tras la resolución formal de `STRAT-01`, VantaDB se define unívocamente como:

> **El motor de memoria cognitiva embebido y local-first para agentes de IA ("SQLite para agentes").**

Se descarta de manera explícita competir como base de datos de grafos analítica ("sucesor de Kuzu") o como base de datos vectorial genérica en la nube. VantaDB resuelve el problema crítico de la persistencia de contexto, aprendizaje continuo y gobierno de datos para agentes de IA que ejecutan en entornos locales o distribuidos, combinando:
- Inferencia y almacenamiento embebido sin dependencias de infraestructura ni red.
- Durabilidad resistente a caídas mediante Write-Ahead Log (WAL).
- Recuperación híbrida nativa de baja latencia (fusión RRF entre BM25 y HNSW).
- Jerarquía cognitiva de memoria en cuatro niveles (`vanta-memory` L0→L3).
- Ciclo de vida estricto de datos mediante la API unificada de olvido (`forget()`).

---

## 2. Coexistencia de Arquitectura y Worktrees

Para evitar la sobrecarga de dependencias cruzadas entre el motor de base de datos y la interfaz gráfica de usuario, el repositorio implementa una estricta separación de ramas y worktrees:

1. **Rama `develop` (Core & Bindings):**
   - Contiene exclusivamente los crates Rust del motor (`vantadb`, `vantadb-mcp`, `vantadb-server`, `vantadb-ffi-core`), los bindings oficiales (`vantadb-python`, `vantadb-node`, `vantadb-ts`, `vantadb-wasm`) y el arnés de evaluación (`evals/`).
   - Todos los pipelines de CI (`ci-rust.yml`, `docs-gates.yml`) y herramientas de análisis estático (`ast-grep`, `semgrep`) en `develop` están configurados para escanear únicamente este alcance.
2. **Rama y Worktree `develop-desktop` (VantaDB Studio & Web):**
   - Aloja la aplicación de escritorio basada en Tauri (`desktop/`) y el frontend web Next.js (`web/`).
   - Las reglas específicas de frontend (`frontend-web.md`) aplican exclusivamente a este worktree.

---

## 3. Protocolo de Olvido Unificado (`forget`)

El gobierno de memoria exige mecanismos formales para purgar o invalidar información obsoleta o confidencial. El sistema expone el método unificado `forget(key, mode)` con tres garantías operativas:

| Modo | Comportamiento | Casos de Uso |
|---|---|---|
| **Soft** | Inserta una lápida (tombstone) lógica; oculta el nodo de búsquedas y traversals pero preserva el historial para auditorías. | Desactivación de preferencias o corrección de contexto temporal. |
| **Hard** | Desvincula el nodo de los grafos HNSW y BM25, liberando los punteros e invalidando el almacenamiento físico. | Limpieza periódica de sesiones y liberación de espacio. |
| **Certified** | Ejecuta sobreescritura física (zeroization) y emite un `PurgeCertificate` criptográfico (VER-02) con hash SHA-256 verificable. | Cumplimiento estricto de privacidad, GDPR/CCPA y borrado de secretos. |

---

## 4. Perfiles MCP y Capa Cognitiva (`McpProfile::Brain`)

El crate `vantadb-mcp` implementa perfiles especializados para conectar con agentes en IDEs (Claude Code, Cursor, OpenCode):

- **`McpProfile::Brain`:** Expone un conjunto compacto de 12 herramientas esenciales más el meta-despachador `vanta_dispatch`. Esto evita agotar el context window del LLM mientras proporciona capacidades completas para:
  - Ingesta de turnos y hechos crudos (L0/L1).
  - Recuperación contextual híbrida (`vanta_recall`, `vanta_search`).
  - Síntesis de escenas episódicas y consolidación de persona (L2/L3).
  - Gobierno y borrado unificado (`vanta_forget`).

---

## 5. Rieles de Negocio y Monetización (Operación VE)

La estrategia de recaudación comercial para operaciones radicadas en Venezuela (`VE`) se estructura con las siguientes prioridades pragmáticas:

1. **Creem.io (`BIZ-20`):** Merchant of Record primario para ventas con tarjeta internacional, con liquidación directa en **USDC sobre Polygon** hacia wallet privada, eliminando la intermediación bancaria tradicional.
2. **Binance Pay / USDT (`BIZ-10`):** Canal de cobro cripto directo sin comisiones para acuerdos con design partners y pilotos técnicos.
3. **Payoneer → Airtm (`BIZ-12`):** Riel secundario para recepción de pagos institucionales y posterior conversión a moneda local.
4. **Descarte de Polar.sh / Stripe Connect:** Polar.sh queda pospuesto mientras la entidad opere como persona natural residente en Venezuela debido a los bloqueos de onboarding de Stripe Connect Express. Se habilitará tras la constitución de una LLC en el exterior (`BIZ-22`).
