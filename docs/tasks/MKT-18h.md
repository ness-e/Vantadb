# MKT-18h — Wheels ARM64 Linux + SHA256 reales del Formula Homebrew

- **Plan:** docs/plans/2026-09-03-quality-gtm-wave.md (Task 4, Wave 1)
- **Ruta:** vanta-worker | **Fecha:** 2026-09-03 | **Estado:** ✅ COMPLETED
- **SDP:** discover_skills(BUILD) → 8 candidatos; cargadas: progreso (Trigger 1), base campaign-executor/ponytail vía prompt, incremental/TDD/context embebidas en rol worker. Devención registrada: discovery post-implementación por cambio ≤100 líneas verificado por contrato mecánico.
- **Contrato (user prompt):** 4 cláusulas mecánicas — ver Verificación.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `release-wheels-60.yml` (295L), `release-binaries-63.yml` (150L, SOLO lectura — referencia patrón aarch64, NO modificado), `Formula/vantadb.rb` (61L), `.opencode/rules/release-ci.md`, CI_POLICY §9, maturin-action v1.51.0 action.yml+dist (verificación de inputs).
- **Hacia afuera:** workflow → publish jobs usan `pattern: wheels-*` (compatible con nuevos nombres `wheels-{linux-x86_64,macos,windows,linux-aarch64}`); Formula → no referenciado por código; FAQ:43 menciona `brew install` (comentado "planned" — NO se habilita, decisión de announcement fuera de scope).
- **Referencias entrantes:** ninguna a Formula/ desde workflows o docs activas (grep).
- **Veredicto:** blast radius = 1 workflow (CI release) + 1 fórmula sin dependientes. `completions/*` y `.opencode` sucios en worktree = ruido ajeno, NO stagear.

## Pasos

1. ✅ DISCOVERY: paradero Formula → LOCAL (`Formula/vantadb.rb` en este repo, no tap remoto). SHA reales → assets v0.5.0 con sidecar `.sha256` del workflow de binarios.
2. ✅ Verificación SHA: descarga de los 4 tarballs + `Get-FileHash` local == sidecar CI (4/4 match).
3. ✅ Workflow: matrix include con entrada `aarch64-unknown-linux-gnu` (target → cross container oficial ghcr.io/rust-cross/manylinux_2_28-cross:aarch64, patrón documentado "Hardening Release pipelines" maturin-action v1.51.0); smoke test se omite en jobs cross; artifact names únicos.
4. ✅ Formula: 4 SHA256 reales + remove `bin.install "vantadb-mcp"` (no está en los tarballs → brew install fallaba igual).
5. ✅ actionlint exit 0; `cargo check -p vantadb_py --all-targets` exit 0.
6. ✅ Cierre: Backlog row eliminada, avance/activo/ci-cd.md, plan estado, CI_POLICY §9 precision.

## Verificación (contrato)

| Cláusula | Comando | Resultado |
|---|---|---|
| C1 aarch64 en workflow | `rg -n "aarch64-unknown-linux-gnu" .github/workflows/release-wheels-60.yml` | ✅ 1 match (L56) |
| C2 actionlint | `actionlint .github/workflows/release-wheels-60.yml` | ✅ exit 0 (v1.7.12) |
| C3 Formula sin placeholders | `rg -c "0000000000000000" Formula/vantadb.rb` | ✅ exit 1 (0 matches). Rama "artefacto docs/plans/artifacts/mkt-18h/" N/A: fórmula accesible → fix in-place |
| C4 Rust intacto | `cargo check -p vantadb_py --all-targets` | ✅ exit 0 (nombre real del package; `-p vantadb-python` no existe) |

## Hallazgos / deuda

- **FIND-candidate:** `musllinux: 1_2` era input INEXISTENTE en maturin-action v1.51.0 (no está en action.yml) → config muerta que sugería wheels musl que nunca se produjeron (confirmado: assets v0.5.0 solo manylinux). Removida en este commit como parte del step editado. Wheels musllinux reales → BND-09 (plan).
- **Verificación diferida (stop-condition del plan):** la corrida real del job aarch64 se valida en el próximo `pull_request` a main / tag — el workflow dispara por paths y el patrón está documentado upstream. Cross-compile local en Windows sin docker no viable.
- **NOTICED BUT NOT TOUCHING:** FAQ:43 sigue comentado ("planned") — habilitarlo implica decidir el canal de tap (`brew tap ness-e/Vantadb`) = announcement de producto, delegar a vanta-docs/lead. Distribution del tap: la fórmula vive en el repo principal; `brew tap ness-e/Vantadb` funciona (carpeta Formula/ en root) pero clonaría el repo completo — decisión humana de crear tap dedicado si molesta.
