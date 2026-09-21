# AUDIT-01 — Fix UAF PyO3 `__array_interface__` (🔴 release-blocker) — ✅ COMPLETED

- **Commit:** `bff30d38`
- **Branch:** develop
- **Fecha:** 2026-08-05
- **Skills:** systematic-debugging, test-driven-development, doubt-driven-development, security-and-hardening
- **Contrato:** (1) test repro no crashea — np.asarray() → drop → acceso al array no es dangling ✅; (2) `cargo +nightly miri test -p vantadb` — **no ejecutable**: MIRI no operativo en esta máquina Windows (`rustup +nightly run miri` → "toolchain 'miri' is not installed") y el bloque auditado es un binding PyO3 `extension-module` (cdylib) que MIRI no puede ejecutar (requiere CPython real); core `vantadb` no fue tocado por este diff. Documentado según contrato. (3) benchmark Python smoke ✅ (`--size 200 --queries 20`).

## Root cause (confirmado empíricamente, no solo por lectura)

- `get_array_interface` (vector.rs:63-73 original) exponía `(self.data.as_ptr() as usize, true)` → NumPy creaba **view zero-copy** sobre el `Box<[f32]>` del pyclass.
- **Modelo real del UAF (verificado con diagnóstico):** NumPy SÍ retiene el pyclass vía `arr.base is vv == True` → el `del vv` a secas NO dispara el UAF (por eso el test de drop pasaba aun con el bug). El dangling real se abre con **mutación**: `__setstate__` reemplaza `self.data` y libera el Box VIEJO mientras el ndarray (cuyo `arr.base` mantiene vivo al pyclass, pero NO al buffer viejo) sigue apuntando a esa memoria liberada. **Observado:** tras `vv.__setstate__([9.0]*4)` + reasignación, `arr[0]` leyó `1.000007` en vez de `5.0` → UB con lectura de memoria liberada.
- Path secundario documentado: `try_numpy_array` (convert.rs:189-203) pasa por el mismo getter pero es seguro porque no muta; **NO se tocó** (como exige la tarea).

## Fix (un solo lugar, cubre drop + __setstate__ + cualquier mutación futura)

`vantadb-python/src/vector.rs` — `get_array_interface` ahora devuelve `data` como **`PyBytes`** (copia little-endian f32 vía `f32::to_le_bytes()`, sin `unsafe`), en vez del puntero crudo. NumPy trata un objeto buffer en `data` como fuente de copia → el ndarray pinna el snapshot `bytes` (inmutable, privado) como `base` → **nunca aliasa** `self.data` del pyclass. El ndarray sobrevive a drop/`__setstate__`/unpickle.

- Verificado post-fix: `arr.base` es `bytes` (no el pyclass), `OWNDATA=False`, `writeable=False` (preserva semántica read-only previa — comportamiento backward-compatible).
- `__setstate__`, `try_numpy_array`, `types.rs` **sin cambios** — el fix en el getter los cubre a todos (Regla 6: saldo deuda neto NEGATIVO — se eliminó la deuda P2-2 "Raw pointer UB en __array_interface__" sin introducir deuda nueva; cero `unsafe`).

## Tests (TDD RED → GREEN, en `vantadb-python/tests/test_sdk.py`)

Clase nueva `TestArrayInterfaceMemorySafety` (3 tests):
1. `test_asarray_does_not_alias_pyclass` — discriminador determinístico: `arr.base is not vv` (con el bug: `base is vv` = True) + valores intactos tras mutar el pyclass.
2. `test_asarray_survives_pyclass_drop` — drop + GC + hammering del allocator (2000 allocs del mismo tamaño) → valores intactos. Safety net (NumPy pinna el pyclass vía base, pero cubre el caso si numpy deja de hacerlo).
3. `test_asarray_survives_setstate_mutation` — **el trigger real del UAF**: `vv.__setstate__([9.0]*4)` + hammering → valores intactos. RED confirmado: con el bug leía 1.000007.

RED inicial: `test_asarray_is_owned_copy_not_view` fallaba con `ValueError: assignment destination is read-only` (view read-only) y el de mutación leía garbage. GREEN: 3/3 pasan.

## Verificación

| Check | Resultado |
|---|---|
| pytest repro tests (3) | ✅ 3 passed |
| pytest tests/test_sdk.py completo | ✅ 45 passed, 0 regresiones |
| benchmark smoke (`--size 200 --queries 20`) | ✅ (full 10K falla con OOM 270KB del engine core — **pre-existente/ambiental**, el bench no usa `np.asarray`, no relacionado con este diff) |
| `cargo check -p vantadb_py` | ✅ Finished dev profile |
| MIRI | ❌ no operativo (documentado) |

## Decisiones / notas

- **NumPy copia vs view sobre bytes:** docs oficiales numpy (`arrays.interface.html`) permiten objeto buffer en `data`; el test de mutación prueba que el ndarray NO aliasa el Box → memory-safe. `arr.base` es el `bytes` snapshot inmutable → equivalente a "congelar/clonar" del contrato. No se buscó OWNDATA=True a propósito: `bytes` es inmutable (más seguro que `bytearray` mutable) y preserva read-only.
- **Pickle roto pre-existente** (`__module__ == 'builtins'`, `PicklingError`) — clase no picklable a pesar de implementar `__getstate__`/`__setstate__`. NO se tocó (scope ajeno a AUDIT-01); el path `__setstate__` queda cubierto por el test de mutación directa. Anotar como follow-up: agregar `module = "vantadb_py"` al `#[pyclass]` para habilitar pickle.
- **Full-size benchmark OOM** (270352 bytes) a 10K vectores: sospechar leak/memoria en engine core — delegar a vanta-tuner/vanta-audit; no es de este diff.
- Deuda técnica liquidada: P2-2 (raw pointer UB en __array_interface__) ✅.
- Fuera de alcance (no tocado): WAL/vector/storage, plan file, otros archivos sucios del working tree.
