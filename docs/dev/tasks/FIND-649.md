---
title: "FIND-649: Reparar la matriz semanal de compatibilidad de adaptadores (`ci-fleet.yml` `compat-matrix`)"
kind: task
description: "actionlint .github/workflows/ci-fleet.yml limpio + las 18 filas de compat-matrix declaran package: explícito coincidente con los pyproject.toml de integrations/ (langchain-core>=0.3.81), con instalación fail-closed y verificación..."
---

# FIND-649: Reparar la matriz semanal de compatibilidad de adaptadores (`ci-fleet.yml` `compat-matrix`): versiones fijadas que nunca se instalan y langchain 0.3 incompatible

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** Hallazgo del orquestador 2026-11-16 (`docs/dev/Backlog.md` L888) / Plan Task 61
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🟠
- **Tipo:** Mixto (CI/CD + Python adapters)
- **Turns estimados:** 15
- **Creado:** 2026-11-18T12:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `a1a73bfed`, `682db2472`
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/ci-fleet.yml` (job `compat-matrix`, schedule/workflow_dispatch) |
| Callees | `integrations/{langchain,llamaindex,mem0,letta,haystack,crewai,dspy,openai,ollama}/pyproject.toml` |
| Implicaciones | La matriz de compatibilidad instala el nombre de paquete PyPI real de cada framework sin `2>/dev/null`, verifica post-install la versión instalada con `importlib.metadata`, y alinea la cota inferior de `langchain-core` a `>=0.3.81` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-fleet.yml`, `integrations/langchain/pyproject.toml`, `integrations/llamaindex/pyproject.toml`, `integrations/mem0/pyproject.toml`, `integrations/letta/pyproject.toml`, `integrations/haystack/pyproject.toml`, `integrations/crewai/pyproject.toml`, `integrations/dspy/pyproject.toml`, `integrations/openai/pyproject.toml`, `integrations/ollama/pyproject.toml`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** Paquetes PyPI de cada adaptador (`langchain-core`, `llama-index-core`, `mem0ai`, `letta-client`, `haystack-ai`, `crewai`, `dspy`, `openai`, `ollama`)
- **Archivos que referencian a los editados (referencias entrantes):** `integrations/test_pins.py`, `.github/workflows/ci-fleet.yml`
- **Veredicto impacto:** bajo — afecta exclusivamente el job `compat-matrix` de `ci-fleet.yml` y la cota inferior declarada en `integrations/langchain/pyproject.toml`.

## Contrato
`actionlint .github/workflows/ci-fleet.yml` limpio + las 18 filas de `compat-matrix` declaran `package:` explícito coincidente con los `pyproject.toml` de `integrations/*` (`langchain-core>=0.3.81`), con instalación fail-closed y verificación post-install de la versión instalada.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Resolución de incompatibilidad `langchain-core 0.3.0` vs `langgraph-checkpoint>=2,<5` | A: Subir piso de `langchain-core` a `>=0.3.81` (primera versión 0.3.x cuyo `Reviver.__init__` soporta `allowed_objects` requerido por `langgraph-checkpoint`) / B: Fijar `langgraph-checkpoint` antiguo | A | ✅ decidido-por-evidencia (ref: `integrations/langchain/pyproject.toml`, commits `a1a73bfed`, `682db2472`) |
| 2 | Instalación y verificación en `compat-matrix` | A: Declarar `package` explícito por fila en la matriz, instalar sin `2>/dev/null` ni `||` encadenados, y verificar versión exacta post-install / B: Derivar nombre desde el directorio | A | ✅ decidido-por-evidencia (ref: `.github/workflows/ci-fleet.yml:292-390`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Cada fila de `compat-matrix` en `.github/workflows/ci-fleet.yml` debe declarar su `package` PyPI real, el pin `min` debe coincidir exactamente con la cota inferior del `pyproject.toml` del adaptador, la instalación debe ser fail-closed y debe verificarse la versión instalada antes de ejecutar `pytest`.
- **Comandos de verificación:** `actionlint .github/workflows/ci-fleet.yml` + validación Python de paridad entre las 18 filas de `compat-matrix` y `integrations/*/pyproject.toml`.
- **Deuda pendiente:** ninguna en código/manifiestos (pendiente únicamente la ejecución CI real del job `compat-matrix`, gateado a `schedule`/`workflow_dispatch`).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# FIND-649: Reparar la matriz semanal de compatibilidad de adaptadores (ci-fleet.yml compat-matrix)` |
| `lastAction` | Step 3 ✅ — Verificado por `audit-w0-ci` (#24): 18 filas alineadas con `pyproject.toml`, `langchain-core>=0.3.81` y `actionlint` OK |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | Ninguna en código (ejecución CI real ocurrirá en schedule/dispatch) |
| `contract` | `actionlint .github/workflows/ci-fleet.yml` + paridad `pyproject.toml` ↔ 18 filas de matriz → PASS |
| `nextTask` | Wave 5 / `FIND-644` |

```yaml
contract:
  verificacion: "Python pyproject vs matrix (18 filas) + actionlint .github/workflows/ci-fleet.yml -> PASS (audit-w0-ci #24)"
  evidencia:
    - claim: "compat-matrix declara package explícito en las 18 filas, instala fail-closed con verificación post-install y fija langchain-core>=0.3.81"
      evidencia: ".github/workflows/ci-fleet.yml:292-390; integrations/langchain/pyproject.toml; commits a1a73bfed, 682db2472"
      confianza: alta
  artefactos:
    - ".github/workflows/ci-fleet.yml"
    - "integrations/langchain/pyproject.toml"
  invariantes: "18 filas de compat-matrix sincronizadas con las cotas inferiores de integrations/*/pyproject.toml y verificación post-install fail-closed"
  deuda: "Pendiente ejecución real en CI del job compat-matrix por estar gateado a schedule/workflow_dispatch"
  queda_pendiente: "Ejecución en schedule/workflow_dispatch de ci-fleet.yml"
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (elimina falsos verdes y fallos silenciosos en la matriz de compatibilidad de los 9 adaptadores).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Paridad mecánica entre `pyproject.toml` y las 18 filas de `compat-matrix` + `actionlint` limpio |
| **Commit** | Commits `a1a73bfed` y `682db2472`, conventional commits |
| **Release** | Cotas inferiores verificadas para `v0.10.0` (nota: corrida real en CI pendiente por gate `schedule`/`workflow_dispatch`) |

## Herramientas necesarias
- actionlint
- python3 (validación YAML + `pyproject.toml`)

**Skills cargadas (SDP):** `ci-cd-and-automation`, `source-driven-development`, `systematic-debugging`

## Investigation Notes
- Se comprobó mediante inspección de wheels de `langchain-core` que `Reviver.__init__(..., allowed_objects=...)` requiere `langchain-core>=0.3.81` cuando se combina con `langgraph-checkpoint>=2,<5`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** El cron semanal de `ci-fleet.yml` (`compat-matrix`) fallaba en `llamaindex`, `mem0`, `letta` y `haystack` porque derivaba el nombre de paquete como `<dir>-core==<v>` o `<dir>==<v>` y caía en `pip install "<v>"` con `2>/dev/null`; en `crewai`/`letta`/`ollama` instalaba versiones distintas a las cotas de `pyproject.toml`; y en `langchain/0.3` fallaba con `TypeError: Reviver.__init__() got an unexpected keyword argument 'allowed_objects'`.
- **Hipótesis:** Nombres de paquetes PyPI no declarados explícitamente en la matriz, encadenamiento `||` con `2>/dev/null` que enmascaraba errores de `pip`, y cota inferior `langchain-core>=0.3` demasiado baja para `langgraph-checkpoint>=2,<5`.
- **1 variable controlada:** Añadir clave `package` explícita a cada fila de `compat-matrix`, alinear cada versión `min` con la cota inferior del `pyproject.toml` respectivo (subiendo `langchain-core` a `>=0.3.81`), y añadir verificación post-install de versión.
- **Test RED:** La validación de paridad `pyproject.toml` ↔ `ci-fleet.yml` fallaba en 7 adaptadores antes del fix y pasa en las 18 filas tras `a1a73bfed` / `682db2472` (GREEN).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No aplica (configuración de matriz de tests en CI y cotas mínimas de dependencias en adaptadores).
- [x] **PERFORMANCE** — No aplica (job semanal `compat-matrix` fuera del fast gate de PRs).

## Steps

### Step 1: Declarar `package:` explícito y alinear cotas inferiores en `ci-fleet.yml`
- **Archivos:** `.github/workflows/ci-fleet.yml`
- **Acción:** Definir el nombre real del paquete PyPI en las 18 filas de `compat-matrix`, eliminar `2>/dev/null` y `||` encadenados, y verificar con Python la versión instalada post-`pip install`.
- **Verify:** `actionlint .github/workflows/ci-fleet.yml`
- **Estado:** ✅ COMPLETED (`a1a73bfed`, `682db2472`)

### Step 2: Subir cota inferior de `langchain-core` a `>=0.3.81`
- **Archivos:** `integrations/langchain/pyproject.toml`, `.github/workflows/ci-fleet.yml`
- **Acción:** Actualizar el piso de `langchain-core` a `>=0.3.81` tanto en `pyproject.toml` como en la fila `min` de `langchain` en `compat-matrix`.
- **Verify:** Verificación Python de paridad `pyproject.toml` ↔ `ci-fleet.yml`
- **Estado:** ✅ COMPLETED (`a1a73bfed`, `682db2472`)

## Dependencias
- `FIND-579` y `FIND-644` (orden de ola en `integrations/`).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (`d16458a5` en DEUDA-RESIDUAL-6) + `audit-w0-ci` (#24)
- **Enfoque:** Correcto. Las 18 filas de la matriz coinciden con los `pyproject.toml`, la instalación es fail-closed con verificación post-install, y `langchain-core>=0.3.81` resuelve la incompatibilidad con `langgraph-checkpoint`.
- **Cómo se probó:** Validación mecánica Python `pyproject.toml` vs `ci-fleet.yml` (18 filas) + `actionlint` en #24.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3).
- **Veredicto:** ✅ approve

## Notas
- **Nota de auditoría (`audit-w0-ci` #24):** Código y pisos OK (`langchain-core>=0.3.81`, 18 filas sincronizadas, `actionlint` limpio en commits `a1a73bfed` y `682db2472`). Pendiente ejecución real en CI porque el job `compat-matrix` está gateado a `schedule`/`workflow_dispatch` y aún no ha corrido en `develop`.
