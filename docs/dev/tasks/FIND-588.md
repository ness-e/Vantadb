---
title: "FIND-588 — Redactar secretos en las implementaciones `Debug` de `src/config.rs`"
kind: task
description: "Config y sus vistas de dominio (RbacCfg, StorageCfg, ServerCfg, LlmCfg)"
---

# FIND-588 — Redactar secretos en las implementaciones `Debug` de `src/config.rs`

- **Plan:** docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 11)
- **Prioridad:** 🟠 · **Esfuerzo:** 🟢 2h · **Wave:** 1
- **Estado:** ✅ COMPLETED (commit 960420853)
- **SDP:** security-and-hardening, test-driven-development

## Spec

`Config` y sus vistas de dominio (`RbacCfg`, `StorageCfg`, `ServerCfg`, `LlmCfg`)
derivan `#[derive(Debug)]` y exponen secretos en claro ante cualquier `{:?}`
(logs, `tracing`, `dbg!`, panics de test). Se reemplaza el derive por
`impl fmt::Debug` manual que muestra todos los campos en el mismo orden,
pero con `[REDACTED]` donde hay secreto y `None` donde no lo hay.

## Inventario de campos (decisión: secreto o no)

### `Config` (src/config.rs:718)
| Campo | Decisión |
|---|---|
| `openai_api_key` | **secreto** → `[REDACTED]` |
| `api_key` | **secreto** → `[REDACTED]` |
| `alt_api_key` | **secreto** → `[REDACTED]` |
| `jwt_secret` | **secreto** → `[REDACTED]` |
| `encryption_key` | **secreto** → `[REDACTED]` |
| `encryption_key_cmd` | **secreto** (puede llevar `echo <hex>` inline) → `[REDACTED]` |
| `encryption_key_file` | ruta, NO secreto → visible |
| `tls_cert_path` / `tls_key_path` | rutas, NO secretos → visibles |
| `rbac_config` | delega en `RbacCfg` (claves del mapa = tokens) |
| `llm_url`, `audit_log_path`, `dashboard_dir`, `export_base_dir`, `backup_dir`, `local_model_path`, `allowed_origins`, `trusted_proxies` | NO secretos (rutas/URLs/IPs) → visibles |
| resto (numéricos, bools, enums, maps de TTL, `dedup_policy`, `segment_optimizer`, `insert_batch`, `hot_reload_config`) | NO secretos → visibles |

### `RbacCfg` (src/config.rs:215)
| Campo | Decisión |
|---|---|
| `token_role_map` | **las CLAVES son tokens de API** → nunca se imprimen; Debug muestra `"[REDACTED]" => role` por entrada |
| `roles` | nombres de roles y grants de namespace, NO secretos → visible |

### `StorageCfg` (src/config.rs:249)
`encryption_key` y `encryption_key_cmd` → `[REDACTED]`; `encryption_key_file` visible; resto visible.

### `ServerCfg` (src/config.rs:279)
`api_key`, `alt_api_key`, `jwt_secret` → `[REDACTED]`; resto visible (incl. `tls_*_path`, rutas).

### `LlmCfg` (src/config.rs:303)
`openai_api_key` → `[REDACTED]`; resto visible.

### Structs auditadas SIN secretos (derive intacto)
`RbacRoleCfg` (solo listas de namespaces), `EvictionCfg`, `PoolCfg`,
`InsertBatchConfig`/`GroupCommitConfig`, `HotReloadConfig` (subset explícito
sin secretos), `SegmentOptimizerConfig` + `LsmConfig` (knobs de tuning),
`AdvancedTokenizerConfig` (idioma/stopwords — verificado en `src/tokenizer.rs:18`),
`DedupPolicy` (enum), enums `LogFormat`/`SyncMode`/`PrefetchMode`.

## Decisiones de diseño (cerradas por el owner)

- `impl fmt::Debug` manual por struct, `f.debug_struct(...)` con **todos** los
  campos en el mismo orden — solo cambia la representación de los secretos.
- Helper privado `fn redacted(&Option<String>) -> &'static str` →
  `"[REDACTED]"` | `"None"`.
- **Sin `Drop` ni zeroize en `Config`**: campos `pub` movibles; un `Drop`
  impediría mover campos fuera (E0509) y rompe la API justo antes de
  congelarla en `v0.10.0`. El zeroize del material de clave ya ocurre en la
  ruta criptográfica (`crate::crypto`, `Zeroizing`), donde se usa.
- `token_role_map`: Debug imprime entradas `"[REDACTED]" => <role>` — los
  roles siguen visibles para diagnóstico; los tokens nunca.

## Contrato

```
cargo nextest run --profile audit -p vantadb -E "test(config)" --locked --build-jobs 2
```

Línea base negativa: el test nuevo `config_debug_redacts_every_secret` debe
fallar contra el código actual (RED) y pasar tras el cambio (GREEN).

## DoD

- [x] Test `config_debug_redacts_every_secret`: canarios únicos en cada campo
      secreto (incl. token como clave de `token_role_map`); `{cfg:?}` y
      `{cfg:#?}` sin canarios, con `[REDACTED]`; `encryption_key_file` visible;
      mismas comprobaciones sobre `storage_cfg()`, `server_cfg()`, `llm_cfg()`,
      `rbac_cfg()`.
- [x] Contrato verde + `cargo clippy -p vantadb --all-targets --locked -- -D warnings` + `cargo fmt --check`.
- [x] Review `vanta-audit` (ningún secreto fuera del inventario) + commit local.

## Impacto mapeado (Regla 0)

- `src/config.rs` (leído completo): structs `RbacCfg:215`, `StorageCfg:249`,
  `ServerCfg:279`, `LlmCfg:303`, `Config:719`. Único archivo a editar.
- Entrantes: `From<&Config>` para cada vista (`:482-588`); Debug de `Config`
  consumido por cualquier `{:?}`/`tracing` externo — la firma pública no cambia
  (`Config` sigue `Debug`), solo el output.
- `src/tokenizer.rs:18`, `src/storage/engine/mod.rs:280` (leídos): sin campos
  secretos — no se tocan.
- Ningún test existente compara el `Debug` literal de `Config` (verificado con
  grep `{:?}` sobre `src/` y `tests/`).

## Steps

- [x] DISCOVERY + task file
- [x] RED: `config_debug_redacts_every_secret` falla contra código actual
- [x] GREEN: impl `fmt::Debug` manual en `Config`, `RbacCfg`, `StorageCfg`, `ServerCfg`, `LlmCfg` + helper `redacted` + wrapper de `token_role_map`
- [x] VERIFY: contrato + clippy + fmt
- [x] Review vanta-audit (approve) + commit local 960420853
