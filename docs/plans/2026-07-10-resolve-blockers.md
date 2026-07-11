# Resolver Blockers — Plan de Implementación

> **Goal:** Arreglar o destrabar los 4 blockers identificados en FULL_CODEBASE_AUDIT

**Architecture:** 3 fases: (1) entorno Windows, (2) WASM allocator, (3) documentar el resto

**Tech Stack:** Rust, wasm-bindgen, wee_alloc, VS Build Tools

---

### Task 1: Instalar Visual Studio Build Tools (librocksdb-sys)

**Files:**
- Modify: `.cargo/config.toml` (si es necesario)
- System: instalar VS 2022 Build Tools

**Root cause:** `STATUS_DLL_NOT_FOUND` en `librocksdb-sys` porque no hay `cl.exe` ni `link.exe` MSVC en el PATH.

**Step 1: Verificar estado actual**

```powershell
where.exe cl.exe 2>&1
where.exe link.exe 2>&1
```

**Step 2: Instalar VS 2022 Build Tools con workload C++**

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --norestart"
```

**Step 3: Verificar instalación**

```powershell
& "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
where.exe cl.exe
cl.exe /?
```

**Step 4: Probar compilación**

```bash
cargo check --package vantadb
```

---

### Task 2: Agregar wee_alloc a WASM build (WA4)

**Files:**
- Modify: `vantadb-wasm/Cargo.toml`
- Modify: `vantadb-wasm/src/lib.rs`

**Step 1: Agregar wee_alloc dependency a Cargo.toml**

Agregar `wee_alloc = "0.4"` a la sección `[dependencies]`.

**Step 2: Agregar global_allocator en lib.rs**

```rust
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

Colocar después del `#![cfg_attr(...)]` y antes de los `use`.

**Step 3: Verificar**

```bash
cargo check -p vantadb-wasm --target wasm32-unknown-unknown
```

---

### Task 3: Documentar WA2 (WASM code splitting)

**Files:**
- Create: `docs/plans/2026-07-10-wasm-code-splitting.md`

Documentar el approach para dividir `vantadb-wasm` en sub-crates. No implementar.

---

### Task 4: Verificar estado de advisories y deny.toml

**Files:**
- Check: `deny.toml`

Los 3 advisories están correctamente ignorados con rationale actualizado. No hay cambios necesarios.

---

### Task 5: Verificar fjall version

**Files:**
- Check: `Cargo.toml:41`, `Cargo.lock`

fjall 4.0 no existe. La versión 3.1.6 es la última. No hay cambios.
