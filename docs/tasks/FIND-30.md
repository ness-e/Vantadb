# FIND-30 — unused var `ns` en cli_server.rs:1302 (clippy -D warnings blocker)

- **Plan:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md`
- **Estado:** ✅ COMPLETO (verificado, sin cambios necesarios)
- **Esfuerzo:** 🟢 · **Appetite:** max 30m
- **Contrato:** `cargo clippy -p vantadb --features server --all-targets -- -D warnings` pasa

## DISCOVERY

### Blast radius (codegraph_explore + Read src/cli_server.rs:1270-1339)
- Closure `options_for` en `src/cli_server.rs:1330` (NO 1302 — el número del hallazgo
  REVIEW-17 apuntaba al bloque; la línea real del closure es 1330).
- El closure es interno a `records_list` (handler `GET /api/v2/list`), captura
  `filter_ops`/`NS_CAP` y recibe `ns: String` que **nunca se usa** (el fan-out por
  namespace pasa por `names`/`db.namespace_stats`, no por el param).
- `options_for` se usa SOLO dentro de `records_list` (llamado en el loop de merge,
  líneas ~1336+). Ningún caller externo.

### Impacto mapeado (Regla 0)
- **Archivo:** `src/cli_server.rs` — leído completo en zona 1270-1339 + contexto por codegraph.
- **Referencias hacia dentro del cambio:** ninguna (cambio local a un closure privado de función).
- **Referencias entrantes al closure:** ninguna — `options_for` no es pub ni se exporta.
- **Referencias al handler `records_list`:** route registrada en el router axum
  (server feature) — el renombre `ns`→`_ns` no altera la firma ni el routing.
- **Veredicto:** renombre cosmético, impacto nulo. Ya aplicado en árbol.

## EJECUCIÓN

### Step 1 — Confirmar estado del fix (✅ ya resuelto)
- `git log -S "_ns" -- src/cli_server.rs` → `00a85294 feat(server): MOD-13 add request
  TimeoutLayer` — el `_ns` ya fue commiteado por MOD-13 como parte de su trabajo en
  cli_server.rs.
- `rg "\|ns\b" src/cli_server.rs` → sin matches; único closure con param String es
  `|_ns: String|` en línea 1330.
- **No se requirió edición** — el warning ya no existe en el árbol.

### Step 2 — Verificar contrato
- `cargo clippy -p vantadb --features server --all-targets -- -D warnings` → ✅
  `Finished dev profile` sin warnings.
- `cargo check -p vantadb --features server --all-targets` → ✅ Finished sin errores.

## CIERRE

- Verify contrato: ✅ clippy server feature pasa. El hallazgo FIND-30 está resuelto
  desde MOD-13 (`00a85294`); esta tarea confirma el contrato del plan.
- Sin archivos tocados (fix pre-existente) → sin commit propio (lead verifica/commitea).
- Nota para el lead: la task puede cerrarse como ✅ DO con 0 diff — el fix entró con MOD-13.
