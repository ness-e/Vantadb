# Plan de Ejecución: Triage de 30 alertas Code Scanning — 2026-09-22

> **Campaign ID:** (asigna `campaign_get_next_task` al arrancar)
> **Inicio:** 2026-09-22
> **Estado:** ✅ COMPLETADO (2026-09-22: 30/30 dismissed con motivo individual — 1 FP error, 2 FP crypto, 1 won't-fix con sunset, 26 test-only; residual 0; +higiene prx07 con tests 12/12)
> **Fuente:** triage READ-ONLY 2026-09-22 (`vanta-audit`: 30/30 ubicaciones leídas con
> contexto + reglas CodeQL/CWE verificadas). Secret-scanning 7/7 resolved: nada que hacer.

## Decisiones del owner (vinculantes, 2026-09-22)

1. **Wave A aprobada para ejecución inmediata**: dismiss #138 como falso positivo documentado.

## Resumen

1 ERROR + 29 WARNINGS. Veredicto del auditor: 1 dismiss inmediato (FP del único ERROR),
0 fix-now en crypto prod (2 FP + 1 accept-risk con plan sunset), 21 supresiones test-only
en batch, 1 micro-higiene. Ningún UB real, ningún secreto en prod.

## Wave A — Dismiss documentado del único ERROR (costo 0, desbloquea gate)

- [x] CSCAN-A1 · Dismiss #138 (`rust/access-invalid-pointer`, `vantadb-node/src/lib.rs:48`)
  como false-positive con justificación: 0 `unsafe` user-written en el archivo (grep
  verificado); el único código raw-pointer es glue generado por el macro `#[napi]`
  (napi-rs), no user-written; struct = `Embedded`+`OpGate` safe; bounds `Send+'static`
  verificados. Regla 4 del proyecto NO dispara Miri (sin `unsafe` nuevo).
  Contrato: alerta → `dismissed` con esta justificación; opcional comentario
  `// CodeQL-suppress` del owner. (Rigor extra opcional: `cargo miri` en
  `vantadb-node` limpio — lo ejecuta el owner, fuera de este plan.)

## Wave B — Crypto prod: documentar, no "arreglar" (FP confirmados + 1 accept-risk)

- [x] CSCAN-B1 · #141 (`hard-coded-crypto`, `src/crypto.rs:301`) = FP: es PBKDF2-HMAC-SHA256
  recomendado con 210k iteraciones (OWASP 2023, cfr. `crypto.rs:118-119`), no una key
  hard-coded. Dismiss con motivo. NO migrar a Argon2 (costo formato+compat, cero ganancia).
- [x] CSCAN-B2 · #119 (`src/crypto.rs:236`, `salt=[0u8;16]`) = FP: zero-init sobrescrito
  por CSPRNG (`rand::rng().fill_bytes`, línea 237) + nonce CSPRNG. Dismiss con motivo.
- [x] CSCAN-B3 · #137 (`weak-hashing`, `src/crypto.rs:285`, `Sha256::digest` fallback) =
  accept-risk con plan: fallback SOLO-lectura para mensajes pre-PBKDF2 (AUDREP-10); el path
  de escritura usa PBKDF2+salt aleatoria. NO eliminar (rompería ficheros viejos). Follow-up
  real: telemetría de hits del fallback legacy + fecha de retirada (sunset).
- [x] CSCAN-B4 · #136 (igual, pero en test `test_legacy_sha256_message_still_decrypts`) =
  suppress-test-only: construye a propósito un vector legacy (documentado líneas 517-521).

## Wave C — Supresión batch test-only + micro-higiene (un solo batch)

- [x] CSCAN-C1 · cleartext-logging en tests (16): `mcp_tests.rs` (5099, 5008, 3263),
  `test_query_embed.rs` (84,108,119 + eprintln skips 96,166,235,176,246,262),
  `test_auto_embed.rs` (49,137). Todos asserts/`eprintln` de CI con corpus sintético
  (gato/felino, batch_ns/k1..k3) — supresión `test-only` en batch.
- [x] CSCAN-C2 · hard-coded `salt` de determinismo (10): `dream_tests.rs` (162,301),
  `dreaming.rs` (99,152,188,231,284), `dream/mod.rs` (953,960,961, bajo `#[cfg(test)]`).
  Son etiquetas para run-id determinista (SipHash no-criptográfico, doc 723-726), no
  passwords/keys — FP + supresión test-only.
- [x] CSCAN-C3 · Micro-higiene `prx07_redact.rs:65`: el assert ecoa el secreto SOLO si el
  test falla — cambiar mensaje a hash/longitud (no `sk-ant-/ghp_` ni en fallo).
  Los 4 de `mem_command.rs` (345,349,361,402) son tests en `src/` con payloads
  sintéticos; verificar que el allow de CodeQL sea por-path `%/tests/**`+`#[cfg(test)]`.
- Contrato Wave C: alertas test → suprimidas/dismissed con motivo `test-only`; verificado que
  NADA en `src/` prod loguea payloads (ya verificado por el auditor).

## Wave D — Verificación

- [x] CSCAN-D1 · `gh api code-scanning/alerts` → residual esperado: solo #137
  (accept-risk con sunset) + supresiones justificadas; 0 ERROR abiertos.
  Contrato: CodeQL verde o 1 aceptado documentado.

## Gates

- **Gate P:** este plan es SOLO análisis+propuesta; dismiss/suppress se ejecutan en
  `/pipeline run` (dismiss en UI/API con justificación escrita por alerta).
- **Stop honesto:** si un dismiss exige cambio de código real (no FP) → se convierte en
  fix con tests, no se dismissa.
- **Appetite:** 1d / dismiss vía UI GitHub o `gh api` (documentar comando exacto por alerta).

## Fuentes (CodeQL/CWE verificados por fetch)

- rust/access-invalid-pointer (error, CWE-476/825): https://codeql.github.com/codeql-query-help/rust/rust-access-invalid-pointer
- rust/hard-coded-cryptographic-value (warning 9.8, CWE-259/321/798/1204): https://codeql.github.com/codeql-query-help/rust/rust-hard-coded-cryptographic-value
- rust/weak-sensitive-data-hashing (warning 7.5, CWE-327/328/916): https://codeql.github.com/codeql-query-help/rust/rust-weak-sensitive-data-hashing
- rust/cleartext-logging (warning 7.5, CWE-312/359/532): https://codeql.github.com/codeql-query-help/rust/rust-cleartext-logging
- Tabla CWE↔query: https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-queries/rust-built-in-queries
- CWE: https://cwe.mitre.org/data/definitions/476.html · …/825.html · …/798.html · …/532.html

=== RECITATION ===
Objetivo activo: PLAN code-scanning-30 — plan creado (SOLO PLAN)
Estado: plan (Wave A: 1 dismiss · Wave B: documentar crypto · Wave C: batch test-only · Wave D: verify)
Última acción: plan creado desde triage 30/30 + CodeQL/CWE verificados
Resultado: ✅
Próxima acción: `/pipeline run docs/plans/2026-09-22-code-scanning.md` (Wave A primero)
Contrato: plan file existe con waves, contratos alerta→dismiss y gates; task files bajo demanda
Próxima tarea: CSCAN-A1 (dismiss #138)
last-synced: 2026-09-22
=== END RECITATION ===
