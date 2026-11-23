---
title: Redactar ADR-0060 (migración on-disk v0.9.0 → v0.10.0) y probar compatibilidad contra una base creada por v0.9.0
kind: task
status: completed
description: "python -c \"import pathlib; p=sorted(pathlib.Path('docs/dev/architecture/adr').glob('ADR-0060-.md')); assert p, 'falta ADR-0060'; t=p[0].readtext(encoding='utf-8').lower(); faltan=[k for k in..."
id: FIND-631
type: feat
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 47)
---

# FIND-631: Redactar ADR-0060 (migración on-disk `v0.9.0` → `v0.10.0`) y probar la compatibilidad contra una base creada por `v0.9.0`

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (`FIND-631`) / Plan Task 47
- **Esfuerzo:** 🟠 1d
- **Prioridad:** 🔴
- **Tipo:** Mixto (Docs/ADR + Rust integration tests)
- **Turns estimados:** 20
- **Creado:** 2026-11-09T10:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `c7eec35ed`, `bbc1fcbac` (ADR-0060), `275a78df6`, `3cbb2919d`, `6a9427ebc`, `53fcefb26` (fixture y tests), `dc65408f7` (enmienda), `058ee4864` (gating sin `encryption`)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Suite de integración (`tests/compat_v090.rs`), script generador (`dev-tools/scripts/gen-fixture-v090.ps1`) |
| Callees | `vantadb::storage::engine::StorageEngine`, `vantadb::migration::MigrationEngine`, `vantadb::sdk::Embedded`, `ADR-0057`, `FIND-499` |
| Implicaciones | Garantiza compatibilidad hacia atrás en disco `v0.9.0 → v0.10.0` (lectura en sitio sin migración destructiva), coherente con ADR-0057 D7 (`warn_encryption_not_applied`). |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md`, `src/migration.rs`, `src/storage/engine/init.rs`, `src/lib.rs`, `src/error.rs`, `tests/compat_v090.rs`.
- **Archivos referenciados hacia dentro:** `vantadb::migration::MigrationEngine` (`src/lib.rs:129`), `vantadb::error::Error::IncompatibleFormat` (`src/error.rs:180`), `StorageEngine::open_with_config`.
- **Archivos que referencian a los editados:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md` (Task 47 y Task 54 `FIND-640`).
- **Veredicto impacto:** bajo en código de producción (invariantes ya en `init.rs` y `migration.rs`; añade ADR-0060, fixture reproducible de 66,945 bytes `<< 5 MB` y suite de 9 tests de compatibilidad compilables con y sin feature `encryption`).

## Contrato
`python -c "import pathlib; p=sorted(pathlib.Path('docs/dev/architecture/adr').glob('ADR-0060-*.md')); assert p, 'falta ADR-0060'; t=p[0].read_text(encoding='utf-8').lower(); faltan=[k for k in ('v0.9.0','v0.10.0','backup','power','rollback') if k not in t]; assert not faltan, 'ADR-0060 no cubre: '+repr(faltan); print('OK')" && cargo nextest run --profile audit -p vantadb --features encryption --locked --build-jobs 2 -E "test(compat_v090)"`

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Política on-disk `v0.9.0 → v0.10.0` para bases plaintext | A: Abrir y leer/escribir en sitio sin migración implícita (coherente con ADR-0057 D7) / B: Migración automática en `open()` | A | ✅ decidido-por-evidencia (`ADR-0060-on-disk-compatibility-v090-v0100.md:1-183`, `init.rs:1168,1419`) |
| 2 | Generación del fixture `tests/fixtures/v0.9.0/` | A: Script reproducible en `dev-tools/scripts/gen-fixture-v090.ps1` compilando tag `v0.9.0` en worktree temporal + `MANIFEST.json` / B: Fixture manual | A | ✅ decidido-por-evidencia (`dev-tools/scripts/gen-fixture-v090.ps1`, 66,945 bytes) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** Toda base creada por `v0.9.0` (VantaFile v2, WAL v3, HNSW v8, Fjall v3) abre en `v0.10.0` sin pérdida de registros, vectores ni aristas; si se pasa clave de cifrado sobre una base plaintext `v0.9.0` existente, permanece en plaintext con aviso (`ADR-0057` D7); versiones futuras (`v99`) fallan cerrado con `IncompatibleFormat`. Compila tanto con `--features encryption` como con el core gate sin `encryption` (`058ee4864`).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --features encryption --locked --build-jobs 2 -E "test(compat_v090)"` (9/9 PASS) y sin `encryption` (8/8 PASS).
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (cierra brecha de verificación de compatibilidad on-disk `v0.9.0 → v0.10.0`).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato ADR-0060 + 9 tests `compat_v090` pasan con y sin `encryption` |
| **Commit** | Commits atómicos (`c7eec35ed`, `bbc1fcbac`, `275a78df6`, `3cbb2919d`, `6a9427ebc`, `53fcefb26`, `dc65408f7`, `058ee4864`) |
| **Release** | `v0.10.0` abre sin pérdida bases de `v0.9.0` (incluida compatibilidad con `FIND-652` `jiff` RFC3339 byte-idéntico) |

## Herramientas necesarias
- cargo-mcp / nextest
- python3 (validación estructural ADR-0060)

**Skills cargadas (SDP):** `documentation-and-adrs`, `documentation-skill`, `test-driven-development`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — Verifica que `ADR-0057` D7 se honra (no cifrar a medias una base `v0.9.0` plaintext al abrir con clave) y que versiones desconocidas fallan cerrado (`IncompatibleFormat` / `EncryptionKeyRequired`).
- [x] **PERFORMANCE** — Fixture acotado a 66,945 bytes (`<< 5 MB` stop condition); tests `compat_v090` ejecutan sobre copia temporal sin impacto en runtime.

## Steps
- [x] Step 1: Redactar `docs/dev/architecture/adr/ADR-0060-on-disk-compatibility-v090-v0100.md` cubriendo `v0.9.0`, `v0.10.0`, `backup` (`:87-95`), `power` (`:96-101`), `rollback` (`:102-109`), `future` (`:117-121`) y D6/ADR-0057 D7 (`:110-116`).
- [x] Step 2: Crear `dev-tools/scripts/gen-fixture-v090.ps1` y materializar `tests/fixtures/v0.9.0/` + `MANIFEST.json` desde el tag `v0.9.0`.
- [x] Step 3: Implementar `tests/compat_v090.rs` (9 tests: open/read/search/write, D7 plaintext warn con clave, future format `IncompatibleFormat`, rollback, integrity).
- [x] Step 4: Asegurar compilación sin feature `encryption` (`058ee4864` gatea `Config`/`Header` `:19-20`, test keyed `:304` y `EncryptionKeyRequired` `:408`).

## Dependencias
- Task 13: `FIND-499` — `migration::check_integrity()` reconoce formatos actuales planos y cifrados (completada).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` + re-auditoría independiente `audit-w1-storage` (#16) y `audit-w1-crypto` (#18).
- **Enfoque:** Correcto. Política in-place documentada en `ADR-0060` y probada contra bytes reales generados por el binario `v0.9.0`. Verificada ausencia de regresión tras `FIND-652` (`f17caf906` `chrono → jiff`, timestamps `u64` ms y RFC3339 compatible).
- **Cómo se probó:** Contrato Python ADR-0060 `OK` + 9/9 tests en `tests/compat_v090.rs` (`:149,166,174,229,251,267,304-374,377-413,423`) con `--features encryption` y compilación/ejecución limpia sin `encryption` (`058ee4864`).
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
- Fixture `tests/fixtures/v0.9.0/` ocupa 66,945 bytes.
- Creado y sincronizado en Pasada 1 de auditoría (`2026-11-18`) para cerrar la referencia de `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md:1907`.
