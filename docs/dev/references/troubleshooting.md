---
title: "VantaDB — Troubleshooting"
type: reference
status: active
tags: [vantadb, references, troubleshooting]
last_reviewed: 2026-09-27
aliases: []
related: [bug-workflow.md, reading-nextest-output.md]
---

# VantaDB — Troubleshooting

> **Cómo usar:** índice de diagnóstico. Para bugs/test failures sigue `bug-workflow.md`; para output de nextest ver `reading-nextest-output.md`; para troubleshooting de usuario ver `docs/user/troubleshooting.md`.
> **Cómo editar:** agregar nuevo síntoma al final de la sección correspondiente con: síntoma, causa raíz, solución, comando exacto.

## Secciones

- Bugs y test failures → `bug-workflow.md`
- Lectura de output nextest (SLOW, LEAK, flaky) → `reading-nextest-output.md`
- Operación y usuario → `docs/user/troubleshooting.md` (si no existe, crearlo antes de enlazar síntomas de usuario aquí)

## Builds y tests locales

### `cargo test` desde la raíz cuelga infinito (puerto `:8080`)

- **Síntoma:** `cargo test` (o `cargo test --test <target>` sin `-p`) desde la raíz del workspace no termina nunca — el proceso arranca un server HTTP real que escucha en `:8080` y bloquea indefinidamente. Incidente real: 2026-09-26, commit `f6c395ef`.
- **Causa raíz:** el build implícito de `default-members` produce una **unificación de features** en el `target/` compartido — `vantadb-server` pide `vantadb/server`, así que `cli_tests` compila con `server` activada y `cmd_server(http=true)` inicia el server HTTP real en vez de devolver error.
- **Solución (defensa en 3 capas):**
  1. **Regla dura:** NUNCA correr `cargo test`/`cargo nextest` sin `-p <crate>` desde la raíz (`.opencode/AGENTS.md` Regla 1, "Regla dura `-p`"). Iteración canónica: `cargo nextest run --profile audit -p vantadb --test <test>`.
  2. **Gate del test:** `test_server_missing_feature` tiene `#[cfg(not(feature = "server"))]` (`tests/cli_tests.rs:1527`, fix `f6c395ef`) → en builds unificados el test queda excluido y no cuelga.
  3. **Defensa de target:** los targets feature-gated llevan `required-features` en `Cargo.toml` (ej: `stress_protocol` → `rayon`, HARD-05) — sin skip silencioso.

**Comandos exactos:**

```bash
# Repro del modo unificado (desde la raíz): debe COMPLETAR sin hang
cargo test --test cli_tests

# Verificación canónica scoped (fast gate)
cargo nextest run --profile audit -p vantadb --test cli_tests
```

- **Si volvés a ver el hang:** matar el proceso; confirmar el listener con `netstat -ano | findstr :8080`; revisar si el comando se invocó sin `-p`.
