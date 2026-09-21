# ERR-PY-01 - Unificar providers a jerarquía MOD-20 + code/retriable + to_dict

> **Status:** ✅ COMPLETED (2026-09-02, vanta-worker — primer intento murió por infra antes de empezar; ejecutado fresco)
> **Task ID:** ERR-PY-01
> **Plan:** `docs/plans/2026-09-02-error-observability-excellence.md` (Task 3, Wave 2)
> **Owner:** vanta-worker
> **SDP:** incremental-implementation, test-driven-development, context-engineering (core del agente, embebidas en spec §3), source-driven-development (spec §1.1 + pyo3 0.29 source)

## Goal

Eliminar el HIGH drift: `providers/shared_py.rs::err_to_py` era bucket-4
(`NotFound→PyKeyError`, `_→RuntimeError(format!("{:?}", e))`) que colapsaba 6
variantes finas y filtraba Debug; `vantadb-python` tenía la jerarquía MOD-20
(11 clases en `convert.rs`) no reutilizada. Toda la superficie Python debe
exponer nombres MOD-20 + atributos canónicos `code`/`retriable` con el valor
wire exacto `VANTADB_*` (fijado por ERR-TS-01).

## DISCOVERY (crate boundary — pre-mortem #1)

- `providers/{openai,litellm,ollama}/Cargo.toml` → `vantadb = { path = "../.." }` **solamente**; cero dependencia de `vantadb-python` → `map_vanta_error` no importable.
- Opción (a) verificada y descartada: `vantadb` feature `python_sdk=["pyo3"]` existe pero `src/python.rs` es un `ClientEngine` mínimo — no define excepciones.
- → **Opción (b) del brief:** mirror de la jerarquía en `providers/shared_py.rs` vía `create_exception!` con nombres idénticos. Duplicado ~40L aceptado vs crate nuevo. Techo: `// ponytail: share via vantadb-python re-export si aparece 3er consumidor`.

## Implementación

1. `providers/shared_py.rs`: 11 `create_exception!(vantadb_py, …)` (base `PyRuntimeError`), `err_to_py` con la tabla variante→clase espejo de `map_vanta_error`, `attach_err_meta` (setattr `code`=`e.code()`, `retriable`, `hint`; `Python::attach` — `with_gil` no existe en pyo3 0.29 — porque `err_to_py` corre dentro de `py.detach`), `register_errors(m)` exportable; fallback `VantaValue→Debug` → match exhaustivo nativo (DateTime→RFC3339, Null→None). Firma `fn(VantaError)->PyErr` intacta (21 call-sites providers).
2. `vantadb-python/src/convert.rs`: `map_vanta_error` termina en `attach_err_meta` (49 call-sites sin tocar).
3. `to_dict()`: `create_exception` produce tipos estáticos — sin `#[pymethods]`, `add_class` no aplica. ≤30min → **helper llano** `vantadb.error_to_dict(exc)` (forma §5.2) en `__init__.py`; método `exc.to_dict()` DEFER documentado.
4. `.pyi` espejados: SDK (`code: str`, `retriable: bool`, `hint: str | None` + `error_to_dict`) ×2 archivos + 3 `.pyi` providers con la jerarquía.
5. Sweep colateral: 11 `format!("{:?}", e)` (PyErr leak) → `{}` Display en 3 `python.rs`.

## Verificación (contrato mecánico)

| Check | Resultado |
|-------|-----------|
| `grep -c 'format!("{:?}"' providers/shared_py.rs` == 0 | ✅ 0 |
| `grep -c "code" providers/shared_py.rs` >= 3 | ✅ 7 |
| `cargo check --manifest-path providers/{openai,litellm,ollama}/Cargo.toml --all-targets` | ✅ 0/0/0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 0 (×3 providers idem) |
| `cargo fmt --all -- --check` (raíz + 3 providers) | ✅ 0 |
| `verify_pyi.py` ×3 | ✅ 7/7 methods (wheels devel instalados) |
| `test_typed_errors.py` (maturin rebuild, code/retriable/error_to_dict) | ✅ 10/10 |
| providers pytest (importorskip SDKs) | ✅ openai 13/13; litellm/ollama skip (SDKs no instalados) |
| Runtime probe providers reales | ✅ `ValidationError` + `.code=="VANTADB_VALIDATION_ERROR"` + `.retriable is False` + `isinstance(e, VantaError)` |
| Sanity Rust `err_py01_contract_tests` (patrón PROV-07) | ✅ 3/3 ×3 crates |

**Preexistente NO causada por esta tarea:** 5 fails SDK suite por drift `put_batch(entries=)` stub/tests-vs-native — prueba: mismos fails con `.pyd` viejo (2026-09-01) anterior al rebuild.

## Notas

- BREAKING estrecho (documentado en `docs/api/PYTHON_SDK.md` + avance): providers `NotFound`→`KeyError` ahora `NotFoundError` (base `RuntimeError`, ya no `KeyError`); backward-compat `except RuntimeError/Exception` intacta; `ValueError` de `distance_metric` (PROV-07) intocado.
- Clases de providers son type objects distintos de las del SDK (mismo `__module__` string `vantadb_py`); catching cross-módulo no soportado → upgrade path en comentario `ponytail:`.
