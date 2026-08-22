# Task: GOV-B2 — Runbook DR sin comandos fantasma

- **Plan:** docs/plans/2026-08-22-doc-governance-plan.md (NO editar)
- **Estado:** ⏳ IN PROGRESS
- **Archivos clave:** docs/operations/DISASTER_RECOVERY_RUNBOOK.md; src/cli.rs + src/cli_handlers/{backup,diagnostics}.rs (read-only)

## Impacto mapeado (Regla 0)

**Leídos completos:** DISASTER_RECOVERY_RUNBOOK.md (416L), src/cli.rs (410L), backup.rs:14-113, grep cli_handlers/diagnostics.rs.

**Superficie real de vanta-cli (cli.rs, fuente de verdad):**
- Global: `-d/--db` (global=true), `-v/--verbose`, `--memory-limit`
- `backup --out <dir>` (opcional)
- `restore --input <dir> [--force] [--rebuild]` — **NO existe `--from`, NO existe `--dry-run`**
- `doctor` — **sin ningún flag de subcomando (NO existe `--fix`)**
- `count --namespace <ns> [--filter] [--json]` — namespace OBLIGATORIO
- Otros citados por el runbook y existentes: status, stats --json, server --http, rebuild-index, audit-index --deep, repair-text-index
- Backup genera `MANIFEST.json` con `backup_type/created_at/vantadb_version/base_ref/files[{name,size,crc32c}]` (backup.rs:26-49) — no hay subcomando que lo valide.

**Comandos fantasma a eliminar:** `restore --dry-run` (:233,:266), `restore --from` (:63,:365,:405), `doctor --fix` (:142), `count` sin `--namespace` (:69,:352,:409).

**Referencias entrantes:** docs/operations/master-index.md lista el runbook (solo nombre). Sin inbound a líneas específicas.

**Veredicto:** solo se edita el runbook; sin cambios de código ni de plan.

## Steps

1. ✅ Reescribir secciones con comandos fantasma usando procedimiento validado GOV-A3 (backup→manifest→restore tmp→doctor→count/get), marcado verificado 2026-08-22.
2. ✅ Banner de revisión + last_reviewed + nota PENDIENTE (--dry-run / verify).
3. ✅ Grep global docs/operations/ por comandos fantasma residuales: también corregidos `restore --from` en CONFIGURATION.md:330 y DEPLOYMENT_GUIDE.md:499,502.
4. ✅ Verify: markdownlint-cli2 exit 0 (3 archivos) + rg bidireccional 11/11 OK.

## Resultado

- Comandos fantasma eliminados del runbook: `restore --dry-run` (:233,:266), `restore --from` (:63,:365,:405), `doctor --fix` (:142), `count` sin `--namespace`.
- Procedimiento diario nuevo en §3 "Daily Backup Verification" (full + variante ligera MANIFEST/CRC32C).
- Nota PENDIENTE de verificación nativa incluida.
- Git NO tocado (prohibido por el orquestador); commit pendiente para vanta-lead.
