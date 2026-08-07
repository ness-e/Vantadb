# VantaDB Pro — Feature Inventory (Open Core boundary)

> Source plan: `docs/plans/2026-08-06-oc-vantadb-pro.md`
> Date: 2026-08-06
> Decisión D4: **Features NUEVAS** — el core Apache-2.0 queda intacto. Esta tabla documenta qué gates EXISTENTES en el core son candidatos conceptuales de Pro (referencia para no clonarlos gratis), no tareas de mover código.

Norma del modelo Open Core:

- El core `vantadb` (Apache-2.0) **no se toca** por decisiones de licenciamiento (D4 = features nuevas).
- Grandes features nuevas con moat real -> nacen directamente en `vantadb-pro` (repo privado), fuera del workspace.
- Un feature "gratis pero con límite" o embebible que compite se marca como Pro conceptual en esta tabla **sin** mover código existente.

## Inventario de gates existentes (referencia)

| Feature | Cargo.toml (lines) | Deps | Código (paths:lines) | Categoría | Espectro Pro |
|---------|--------------------|------|----------------------|-----------|--------------|
| `encryption` | :117 (`aes-gcm`,`sha2`) | aes-gcm 0.10, sha2 0.10 | `packageStorage` `src/storage/vfile.rs:8,531,578,651,842,849,862`; `src/lib.rs:53` | Seguridad | **Candidata Pro** (moat clásico) |
| `wal-shipping` | :118 (`reqwest`) | reqwest | `src/lib.rs:138` | Replicación/distribuido | **Candidata Pro** |
| `pitr` | :119 | — (solo cfg) | `src/lib.rs:142` | Point-in-time recovery | **Candidata Pro** |
| `prometheus` | :129 (`prometheus`) | prometheus | `src/metrics/core/registry.rs` (72+ bloques), `src/metrics/core/mod.rs:16-43,564,906`, `src/memory_governor.rs:118` | Observabilidad | Pro (enterprise ya la monetiza en tier) |
| `server` | :121 (`axum`,`tower_governor`,`tower-http`) | axum, tokio | `src/lib.rs:66,72,78`; `src/cli_handlers/server.rs:188,207` | Servicios | Pro (server delgado) |
| `tls` | :128 (`axum-server`,`rustls`) | axum-server, rustls | `src/cli_server.rs:698,817,887` | Seguridad transporte | Pro |

### No-Pro (se quedan libres en el core de forma permanente)
- `fjall`, `rocksdb` (backstores), `arrow`, `cli`, `roaring`, `advanced-tokenizer`, `remote-inference`, `failpoints`, `async-*`, `opentelemetry`, `wasm`, `tui`, `custom-allocator`, `jemalloc`, `bayesian_decay`, `hot-reload`, `python_sdk`.

## Backlog Pro (adiciones nuevas, aún no escritas)
| Feature Pro sugerida | Qué clava | tarjeta |
|---|---|---|
| Multi-tenancy / RBAC | aislamiento cifras org | MoAT |
| Replicación multi-copy / Sync | DR | — |
| Wal shipping + PITR (ya gates) | failover | — |
| TTL / retention policies | compliance | — |
| Admin server + dashboard | UX enterprise | — |
| Audit trail / compliance | — |

## Verificación del meta-modelo
- [x] `Cargo.toml` workspace `members` no incluye `vantadb-pro` (`:591-597`) — se respeta.
- [x] `default-members` `:601-604` sólo core+python — sin cambio.
- [x] `deny.toml` gate MIT/Apache-2.0 aplica al core, no al Pro.
- ✓ `cargo check --no-default-features` (común a cada puerto) se mantiene estable — no se tocaron gates.

> ⚠️ **Ponytail:** esta tabla es referencia para las features que nacen en Pro, no instrucción de borrow del core. Las features `encryption`/`wal-shipping`/`pitr`/`prometheus`/`server`/`tls` siguen viviendo en el core como están. Si algún día migras (decisión `D4B`, humana), este inventario da el mapa de `src/`.