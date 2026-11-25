---
title: "FIND-731 - el flag de update del golden es un booleano por entorno, no un booleano por presencia"
kind: task
status: completed
description: "`VANTADB_PUBLIC_API_UPDATE=` (definida y vacía) entraba en modo update porque `var_os(..).is_some()` devuelve true con `Some(\"\")`: el gate del contrato público se auto-aceptaba."
---

# FIND-731 - un flag vacío no debe reescribir el golden

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`
> **Ruta:** `vanta-worker` · **Carril:** R (lleno — `MCP-44`/`MEM-71` en vuelo) · **Appetite:** 30 min (real: ~1h40, dominado por 5 corridas de rustdoc nightly)
> **Dependencias:** `FIND-722` ✅ (`c8d9e95a`) · `FIND-733` ✅ (`8bc0e57bb`)
> **Origen:** review de `FIND-722` — regeneró el golden con la variable **vacía** y el test devolvió PASS.

## Contexto

`tests/api/public_api.rs` es el gate HARD-01: falla si la superficie pública del
crate cambia sin que el autor regenere `tests/api/public-api.txt` y lo revise.
La válvula de escape es una variable de entorno:

```rust
// antes (tests/api/public_api.rs:123)
if std::env::var_os(UPDATE_ENV).is_some() {
    fs::write(SNAPSHOT, format!("{HEADER}{current}")).expect("failed to write the snapshot");
    eprintln!("[public_api] snapshot updated ({UPDATE_ENV} set): {SNAPSHOT}");
    return;
}
```

`std::env::var_os` devuelve `Some(OsString::from(""))` cuando la variable
**existe vacía**. Con `VAR= cargo nextest run …` (forma bash de "definir y
olvidar", o un `env:` mal escrito en CI) el test reescribía el golden con la
superficie *actual* y devolvía **PASS**: el gate se convertía en
auto-aceptador de cualquier delta de contrato. No es hipotético — pasó en el
review de `FIND-722`, donde el golden se regeneró con la variable vacía.

Semántica correcta para un flag booleano por entorno: **existe y no está
vacío**. Ausente = compare; `""` = compare; `1`/`true` = update.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `tests/api/public_api.rs:1-143` | golden, `HEADER`, `normalize`, `first_diff`, el test `#[ignore]` | el único sitio del flag; cabecera y contrato de regeneración |
| `docs/api/IQL.md` §AST JSON | delta que introdujo `FIND-722` | confirma que el golden ya cambió una vez y que el modo update se usó con la variable vacía |
| `docs/dev/tasks/FIND-722.md:161` | nota del falso verde | origen del hallazgo |
| `docs/dev/Backlog.md:768` | fila `FIND-731` | contrato literal |

**Referencias hacia dentro (quién consume el flag)**

- `.github/workflows/**` — sin `VANTADB_PUBLIC_API_UPDATE`; el job `public-api-snapshot` corre el test en modo compare. No se tocó `.github/**` (prohibido en esta tarea).
- `tests/api/public_api.rs:13-15,41,139` — ayuda de uso en el docstring de módulo, el `HEADER` del golden y el mensaje de fallo: ya documentan `=1`, que sigue siendo el camino válido.

**Referencias hacia afuera**

- Ninguna: el helper es privado y local al test; no cambia superficie `pub`
  (de hecho el golden **no** cambia por este fix — verificado por SHA).

**Veredicto:** cambio contenido en un archivo de test + un helper privado.
No toca `src/**`, ni el golden (el delta artificial del test negativo se revirtió
antes del commit — ver §Evidencia).

## Spec — decisiones

| # | Decisión | Alternativas y por qué se descartan |
|---|---|---|
| 1 | Predicado `update_requested(Option<OsString>) -> bool` = `is_some_and(|v| !v.is_empty())` | **(a)** inline sin helper: imposible testear sin nightly rustdoc (el test real cuesta 13 min la primera corrida). **(b)** chequear `== "1"`: rompe `=true`, documentado en el archivo. **(c)** parsear valores falsy (`0/false/off`): cambia el contrato y va más allá del hallazgo — queda como techo declarado y pineado por test, no como fix silencioso. |
| 2 | Unit tests sobre el predicado, recibiendo `Option<OsString>` en vez de leer el env adentro | Sin `set_var` (unsafe en 2024 y racy con nextest): 4 tests deterministas en 0.05 s, dentro del Fast Gate. El E2E cubre el env real. |
| 3 | No se toca el `HEADER` del golden ni el mensaje de uso | Ya dicen `=1`; el comentario de `UPDATE_ENV` sí se corrigió ("any value" era la semántica del bug), y eso **no** obliga a regenerar el golden (el `HEADER` const es byte-idéntico a las 8 primeras líneas del golden, verificado por el revisor). |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | RED: predicado con semántica vieja (`is_some()`) → `empty_variable_does_not_request_update` FALLA | ✅ |
| 2 | GREEN: `is_some_and(|v| !v.is_empty())` + unit tests (4) | ✅ |
| 3 | E2E escenario **ausente**, golden commiteado → PASS | ✅ |
| 4 | E2E escenario **vacía** → PASS, golden intacto (SHA idéntico) | ✅ |
| 5 | E2E negativo: delta artificial → FAIL con ausente y con vacía | ✅ |
| 6 | E2E escenario **con valor** (`=1`) → modo update, borra el delta | ✅ |
| 7 | Verify + review P2-01 + commit local | ✅ |

## Evidencia E2E

Todas las corridas: `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only --locked`.
La variable "vacía" se setea con `[Environment]::SetEnvironmentVariable('VANTADB_PUBLIC_API_UPDATE','')`
y se verificó que el proceso hijo la ve como **presente y vacía** (`present=True value=''`), que es
exactamente el caso `VAR= cargo test`.

| # | Env | Golden | Resultado | Salida |
|---|-----|--------|-----------|--------|
| 1 | **ausente** (`present=False`) | commiteado | **PASS** | `PASS [767.672s] (1/1) public_api_snapshot_matches_committed_file` · `Summary [767.674s] 1 test run: 1 passed (1 slow), 3 skipped` |
| 2 | **vacía** (`present=True value=''`) | commiteado | **PASS**, golden sin cambios | `PASS [68.654s]` · `GOLDEN_SHA= ACC6574D1783BEB82758F497A5B7BA81931D9DD3DAB5F02D06A5401953BFA895` (= SHA inicial) |
| 3 | **ausente** | + delta `pub fn find731_artificial_delta() -> u8` | **FAIL** | `FAIL [16.395s] (1/1)` · `test result: FAILED. 0 passed; 1 failed` |
| 4 | **vacía** (`present=True value=''`) | + delta | **FAIL** | `FAIL [19.167s]` · `public API surface changed vs tests/api/public-api.txt.` · `line count: snapshot=11161 current=11160` |
| 5 | **con valor** (`present=True value='1'`) | + delta | **PASS** (update) | `PASS [32.842s]` · delta desaparecido (0 ocurrencias) · contenido resultante **idéntico** al golden commiteado (normalizado CRLF) |

Escenario 1 medido antes del fix de comentarios; escenarios 2-5 y los unit tests,
después. El golden se restauró byte-exacto desde backup:

```
GOLDEN_SHA= ACC6574D1783BEB82758F497A5B7BA81931D9DD3DAB5F02D06A5401953BFA895
git diff --stat -- tests/api/public-api.txt   -> (vacío)
git status --porcelain -- tests/api/public-api.txt -> (vacío)
```

**RED (Prove-It):** con `value.is_some()` en lugar del predicado, la corrida da
`assertion failed: !update_requested(Some(OsString::from("")))` — el test
demuestra el bug antes del fix.

**Unit tests (Fast Gate, 0.05 s):** `Summary [0.044s] 4 tests run: 4 passed, 1 skipped`
(1 skipped = el `#[ignore]` del snapshot). Nombres:
`absent_variable_does_not_request_update`, `empty_variable_does_not_request_update`,
`non_empty_variable_requests_update`, `explicit_falsy_values_still_request_update`.

## Techo declarado

`=0`, `=false`, `=no`, `=off` y `=" "` **siguen** significando update: el criterio
pactado es "existe y no está vacío", no "parsear booleano". Está pineado por
`explicit_falsy_values_still_request_update` con un `ponytail:` que nombra el
upgrade path (parsear valores falsy) para que un cambio futuro sea deliberado.

## Review (GATE P2-01 — agente distinto)

- **Revisor:** vanta-review (sesión independiente; sin participación en la implementación).
- **Enfoque:** `is_some_and(|v| !v.is_empty())` sobre `Option<OsString>` = contrato literal del
  Backlog; sin `to_str()`/`into_string()`, así que un valor no-UTF-8 no paniquea. Ruta compare
  intacta para ausente/vacía, ruta update para `1`/`true`. `HEADER` const byte-idéntico a las 8
  primeras líneas del golden → un `=1` legítimo no ensucia el snapshot (verificado).
- **Cómo se probó:** RED con `is_some()` (falla `empty_variable_does_not_request_update`);
  4 unit tests de comportamiento; E2E ausente/vacía/`=1`/delta artificial; re-run del revisor
  `cargo nextest run -p vantadb --test public_api` → 4/4 PASS (1 skipped = el `#[ignore]`);
  golden restaurado byte-exacto (SHA `ACC6574D…`, clean).
- **Veredicto:** ✅ APPROVE. Techo declarado y pineado: `=0`/`= ` siguen significando ON
  (`explicit_falsy_values_still_request_update`), upgrade path anotado.
- **Residuales no bloqueantes (nits):** `public_api.rs:137` dice `({UPDATE_ENV} set)` — ahora es
  "set and non-empty"; `:9-11` habla del test `#[ignore]` cuando el módulo ya tiene 4 tests que
  siempre corren; `#[cfg(test)]` sobre el `mod` es redundante en un target de integración.
- **Clase completa (5 sitios, 3 archivos, NO arreglados acá → filas `FIND-*` del Backlog):**
  `src/hardware/mod.rs:320-321` (`VANTADB_QUIET`/`VANTADB_SILENT`),
  `src/storage/engine/tests/read_only_files.rs:101` (`FIND632_DENY_ALLOWLIST`),
  `tests/core/snapshot_certification.rs:2800` (`FIND319_WINDOWS_NAME_TEST_CHILD`),
  `tests/compat_v090.rs:424` (`VANTADB_V090_CLI`), `src/hardware/mod.rs:749` (`VANTADB_PROFILE_PATH`).

## Verify

| Check | Resultado |
|---|---|
| Contrato E2E (ausente / vacía / `=1` / delta) | ✅ ver §Evidencia E2E |
| `rustfmt --check --edition 2021 tests/api/public_api.rs` | ✅ exit 0 |
| `cargo clippy -p vantadb --test public_api --all-features -- -D warnings` | ✅ `Finished dev profile … in 13m 49s`, sin warnings |
| `cargo nextest run -p vantadb --test public_api` | ✅ 4/4 PASS (1 skipped) |
| `tests/api/public-api.txt` | ✅ byte-idéntico al commit (SHA `ACC6574D…`) |
| Gates workspace (`clippy --workspace`, `nextest --workspace`) | ⏸ no corridos acotados a propósito: el árbol tiene WIP de `MCP-44`/`MEM-71` en `vanta-memory/**`, `vantadb-mcp/**`, `src/**` — el resultado mediría su trabajo, no este |
