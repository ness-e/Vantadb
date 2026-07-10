---
title: "VantaDB — Troubleshooting Común (Windows)"
type: reference
status: active
tags: [vantadb, troubleshooting, windows]
last_reviewed: 2026-07-10
language: es
---

# VantaDB — Troubleshooting Común (Windows)

> **Cómo usar:** Cuando encuentres un error inesperado, busca aquí primero por síntoma (exit code, mensaje). Si no está documentado, investígalo y agrega la entrada.
> **Cómo editar:** Agrega nuevos issues al final de la sección correspondiente. Incluye: síntoma, causa raíz, solución y comando exacto.
> **Referencia desde:** `.opencode/AGENTS.md` — sección Troubleshooting References.

---

## Compilación

### `link.exe` exit code: `0xc0000409` (`STATUS_STACK_BUFFER_OVERRUN`)

**Síntoma:** `error: linking with link.exe failed: exit code: 0xc0000409`

**Causa:** `link.exe` llamó a `abort()` vía `__fastfail`. NO es necesariamente un stack overflow real. Ocurre con MSVC 19.44+ al compilar crates grandes (tokio, windows-sys, chumsky) con debug info.

**Solución:** `.cargo/config.toml` ya fuerza `link.exe` (MSVC nativo) sobre `rust-lld`. Si persiste:
1. Reducir paralelismo: `CARGO_BUILD_JOBS=2 cargo check`
2. Como fallback extremo: `RUSTFLAGS="-Csymbol-mangling-version=v0" cargo check`

---

### `os error 1455` (page file)

**Síntoma:** `fatal error: could not write to page file,操作系统无法运行 %1 (os error 1455)`

**Causa:** Las compilaciones paralelas en Windows agotan el page file.

**Solución:** `.cargo/config.toml` ya limita `jobs = 2`. Si persiste:
1. Cierra apps pesadas (navegador, VS Code, Docker)
2. Incrementa page file manualmente: `System → Advanced → Performance → Virtual Memory → 16GB+`
3. Build solo el crate que necesitas: `cargo check -p vantadb`

---

### `STATUS_STACK_OVERFLOW` en runtime (exit code `0xc00000fd`)

**Síntoma:** `thread 'main' has overflowed its stack`

**Causa:** El stack por defecto en Windows es 1MB. Algoritmos recursivos profundos o serde pueden excederlo.

**Solución:**
```bash
RUST_MIN_STACK=4194304 cargo run    # 4MB stack
RUST_MIN_STACK=8388608 cargo run    # 8MB si sigue fallando
```

---

### `link.exe not found`

**Síntoma:** `linker link.exe not found`

**Causa:** Visual Studio Build Tools no instalado o no detectado.

**Solución:**
```bash
# Verificar instalación
rustup show
# Forzar MSVC toolchain
rustup default stable-x86_64-pc-windows-msvc
# Instalar VS Build Tools si falta: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

---

## Tests (nextest)

### Test cuelga sin salida

**Síntoma:** nextest corre pero un test parece no terminar nunca.

**Diagnóstico:**
```bash
# Presiona 't' en terminal interactiva para ver output en vivo
# O usa SIGUSR1 en Unix
```
O matar y rerun con `--no-capture`:
```bash
cargo nextest run --no-capture -p <crate> --test <test_name>
```

### `SLOW` timeout

**Síntoma:** test marcado `SLOW` y terminado después de 60s × 3 periodos.

**Causa:** El test tarda más de 60s. `slow-timeout` en `.config/nextest.toml` termina tras 3 periodos (3min).

**Solución:** Si es esperado (test de integración pesado), aumentar timeout vía per-test override:
```toml
[profile.audit.overrides]
"test(my_slow_test)".slow-timeout = { period = "120s", terminate-after = 3 }
```

### `LEAK` timeout

**Síntoma:** test marcado `LEAK` después de pasar.

**Causa:** El test creó un proceso hijo que heredó stdout/stderr y no los cerró.

**Solución:** Buscar `std::process::Command` sin `.stdout()`/`.stderr()` apropiado en el test.

### Test se pasa en isolation pero falla en paralelo

**Causa:** Contaminación de estado global compartido (archivos temporales, vars de entorno, puertos).

**Diagnóstico:**
```bash
# Rerun con 1 thread
cargo nextest run --test-threads=1 --test <test_name>
# Si pasa, es race condition por estado compartido
```

---

## Python SDK

### `maturin build` falla con error de enlace

**Causa:** Misma causa que `link.exe` errors arriba.

**Solución:** Usar `maturin build --release` (evita debug info que satura el linker).

### `pip install -e ./vantadb-python` falla

**Causa:** Falta `maturin` o herramienta de compilación Rust.

**Solución:**
```bash
pip install maturin
# Verificar Rust toolchain
rustup show
```

---

## Web (Vite + React)

### `npm install` falla en `web/`

**Síntoma:** Errores de dependencias nativas (node-gyp, sharp, etc.).

**Solución:**
```bash
# Asegurar que Build Tools están en PATH
npm install --ignore-scripts  # salta build steps problemáticos
# O instalar tools necesarias
npm install -g windows-build-tools
```

### HMR no funciona

**Síntoma:** Vite Hot Module Replacement no se actualiza.

**Causa:** Windows con WSL o antivirus bloqueando file watchers.

**Solución:**
```bash
# Forzar polling en Vite
# Agregar a vite.config.ts: server: { watch: { usePolling: true } }
```
O excluir el directorio del proyecto del antivirus.

---

## Git

### `pre-commit` hook falla con `SKIP_CLIPPY=1`

**Síntoma:** El hook bloquea el commit aunque quieras saltar clippy.

**Causa:** El hook respeta `SKIP_CLIPPY=1` (ver header del hook).

**Solución:**
```bash
SKIP_CLIPPY=1 git commit -m "fix: ..."
```

### Saltar hooks completamente

```bash
git commit --no-verify -m "wip: ..."
```

---

## Cargo Tools

### `cargo-deny` encuentra licencia no aprobada

**Solución:** Ver `deny.toml` para la lista de licencias permitidas. Si es necesaria, agregar a `deny.toml` y documentar por qué.

### `cargo audit` encuentra advisory

**Solución:** Seguir recomendación de `cargo audit`. Si no hay fix upstream, evaluar si el crate es crítico para el proyecto y documentar en `docs/advisories.md`.

---

## Cómo Agregar un Nuevo Issue

1. Identifica el síntoma exacto (mensaje de error, exit code)
2. Determina la causa raíz
3. Escribe la solución como un comando ejecutable
4. Agrega al inicio o junto al síntoma más cercano
5. Si el error tiene código de salida único, ponlo como título del bloque
