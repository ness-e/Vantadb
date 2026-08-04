# Python Bindings — Reglas

> **Scope:** `vantadb-python/` (PyO3: `src/lib.rs`, `types.rs`, `convert.rs`, `vector.rs`), `providers/{openai,ollama,litellm}/`
> **No tocar aquí:** API pública del core (`api-contract.md`), ecosistema JS (`js-ecosystem.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** DRV-016, INV-008

## Reglas

### R-1: GIL-release eager en operaciones batch + Rayon + fail-fast

- **Must:** en métodos batch (ej. `search_batch_requests` cuando se implemente, §5 de INV-008), liberar el GIL lo antes posible (`py.allow_threads`), ejecutar los requests con `rayon::par_iter` y propagar el primer error con `try_for_each` (fail-fast) para no retener recursos por requests fallidos.
- **Must not:** ejecutar requests batch secuencialmente dentro del GIL ni degradar a un loop con `anyhow` que continúe tras el primer error.
- **Por qué:** INV-008 verificó que no existe `search_batch_requests` en el binding — solo `search_batch` vector-only; el diseño aprobado exige paralelismo Rayon + error eager para throughput y latencia p99.

### R-2: El closure de bindings se cierra en el binding, no en core/wrapper

- **Must:** al liberar el GIL, mover los handlers de notificación/callback al closure que corre fuera del GIL y cerrarlo (dropearlo) en el propio binding PyO3.
- **Must not:** retener el closure en el core o en wrappers que no gestionan el GIL.
- **Por qué:** cerrar en el binding evita referencias colgantes entre el runtime Python y el closure Rust (INV-008 §5 — principio de diseño del método nuevo).

<!-- Referencias cruzadas: → ver api-contract.md, release-ci.md, js-ecosystem.md -->
