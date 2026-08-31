# TBH-13 — SHA-pin remaining workflows (supply chain hardening)

> Plan: `docs/plans/2026-08-30-testing-bench-harden.md` — TASK-13 (Phase 2 — MED)
> Owner: vanta-lead (CI infra owner — SHA-pins son seguridad)
> Predecessors ✅: TBH-01, TBH-02, TBH-03
> Strategy: 5 line-changes, no YAML reformat, copy pattern from `ci-rust-10.yml`

## Contexto

VantaDB tiene 17 GH Actions workflows; 14 ya usan SHA-pins para actions de terceros.
Faltan 2 workflows:
- `desktop.yml` (3 refs sin pin): `actions/checkout@v4`, `actions/setup-node@v4`, `tauri-apps/tauri-action@v1`
- `opencode.yml` (2 refs sin pin): `actions/checkout@v6`, `anomalyco/opencode/github@latest`

Las acciones tag-flotantes son un gap de supply chain: si el owner de una action
es comprometida y re-tagea la versión, el workflow ejecuta código inesperado.

## SDP — Skills cargadas

- `ci-cd-and-automation` — base type CI/CD / DevOps
- `git-workflow-and-versioning` — Conventional Commits + atomic commits (cargada inline)
- `security-and-hardening` — supply chain hygiene (cargada inline)
- `doubt-driven-development` — stakes altos (seguridad), verificación adversarial
- `incremental-implementation` — slices verticales delgados

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `.github/workflows/desktop.yml` (97 lines)
- `.github/workflows/opencode.yml` (33 lines)
- `.github/workflows/ci-rust-10.yml` (602 lines — referencia de patrón de pin)

**Refs sin pin en target:**
| Archivo | Línea | Ref actual | Versión objetivo |
|---|---|---|---|
| `desktop.yml` | 45 | `actions/checkout@v4` | v4 |
| `desktop.yml` | 52 | `actions/setup-node@v4` | v4 |
| `desktop.yml` | 85 | `tauri-apps/tauri-action@v1` | v1 |
| `opencode.yml` | 24 | `actions/checkout@v6` | v6 |
| `opencode.yml` | 29 | `anomalyco/opencode/github@latest` | default branch |

**Patrón de pin existente en `ci-rust-10.yml`** (línea 54):
```yaml
- uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0
```
SHA 40 hex chars + comment con la versión semver.

**Referencias salientes:** ninguno (YAML sólo referencia refs externas).
**Referencias entrantes:** ninguno (ningún otro archivo referencia los 5 refs).
**Veredicto:** impacto local — sólo se tocan 5 líneas + 5 líneas de comment.

## Spec (no aplica — es cambio mecánico de CI infra)

Cambio de configuración de seguridad, no lógica nueva. Contrato mecánico
verificable vía grep + YAML parse + actionlint. Sin ADR (Regla 5) — no cambia
API pública ni decisiones arquitectónicas.

## Steps atómicos

### Step 1 — Resolver SHA-1 hashes (DISCOVERY)

Tools: `webfetch` sobre `api.github.com/repos/<owner>/<repo>/git/refs/tags/<tag>`
para refs taggeadas; `webfetch .../git/refs/heads/<branch>` para default branch.

Refs a resolver:
1. `actions/checkout@v4` → SHA de la branch `v4` (origen: `actions/checkout`)
2. `actions/setup-node@v4` → SHA de la branch `v4` (origen: `actions/setup-node`)
3. `tauri-apps/tauri-action@v1` → SHA de la branch `v1` (origen: `tauri-apps/tauri-action`)
4. `actions/checkout@v6` → SHA de la branch `v6` (origen: `actions/checkout` — si no existe, fallback a `v4`)
5. `anomalyco/opencode/github@latest` → SHA de `main` o default branch (origen: `anomalyco/opencode`)

Status: ⬜ PENDING

### Step 2 — Edit `desktop.yml` (3 line changes)

- Línea 45: `actions/checkout@v4` → `actions/checkout@<sha-checkout-v4> # v4`
- Línea 52: `actions/setup-node@v4` → `actions/setup-node@<sha-setup-node-v4> # v4`
- Línea 85: `tauri-apps/tauri-action@v1` → `tauri-apps/tauri-action@<sha-tauri-v1> # v1`

NO reformatear nada más.

Status: ⬜ PENDING

### Step 3 — Edit `opencode.yml` (2 line changes)

- Línea 24: `actions/checkout@v6` → `actions/checkout@<sha-checkout-v6> # v6` (o fallback a v4)
- Línea 29: `anomalyco/opencode/github@latest` → `anomalyco/opencode/github@<sha-opencode-main> # <tag-or-sha>`

Status: ⬜ PENDING

### Step 4 — Verify full

Comandos contractuales:
1. `grep -E "@(v[0-9]+|@latest)" .github/workflows/desktop.yml .github/workflows/opencode.yml` → VACÍO
2. `python -c "import yaml; [yaml.safe_load(open(f'.github/workflows/{f}')) for f in ['desktop.yml', 'opencode.yml']]; print('OK')"` → OK
3. actionlint (pre-commit hook) → 0 errors

Status: ⬜ PENDING

### Step 5 — Commit

```bash
git add .github/workflows/desktop.yml .github/workflows/opencode.yml .opencode/skills/campaign-executor/tasks/TBH-13.md
git commit -m "ci(TBH-13): SHA-pin 5 third-party action refs in desktop.yml + opencode.yml (supply chain hardening)"
```

Conventional commit `ci(...)` (no release bump — `ci:` no dispara release-plz).

Status: ⬜ PENDING

## Reglas duras

- **CRÍTICO — SHA correctness:** SHA son únicos por commit. NO inventar. Si no
  se puede resolver con `webfetch` o `gh api` → detener y devolver
  `RESULTADO: 🟡 INCOMPLETO` con próximo step explícito.
- **Ponytail reflex:** 5 line-changes, 1 por ref. NO renames, NO reformats.
- **NO TOCAR** otros workflows (ci-rust-10.yml, ci-web-11.yml, etc. — ya pinned).
- **Conventional Commits:** `ci(TBH-13):` para seguridad.

## SHA-1 hashes resueltos (Step 1 ✅)

Resueltos via `gh api repos/<owner>/<repo>/git/ref/tags/<tag>` + dereference
para annotated tags. Verificado cada SHA existe con `gh api .../commits/<sha>`.

| Ref | SHA (40-char) | Version comment |
|---|---|---|
| `actions/checkout@v4` | `11d5960a326750d5838078e36cf38b85af677262` | v4.4.0 |
| `actions/setup-node@v4` | `49933ea5288caeca8642d1e84afbd3f7d6820020` | v4.4.0 |
| `tauri-apps/tauri-action@v1` | `1deb371b0cd8bd54025b384f1cd735e725c4060f` | v1 (action-v1.0.0) |
| `actions/checkout@v6` | `d23441a48e516b6c34aea4fa41551a30e30af803` | v6.1.0 |
| `anomalyco/opencode/github@latest` (default branch `dev`) | `10765ff2a9da8c3b88e4de873aa383a49c318912` | dev |

**Nota sobre `tauri-action`:** el tag `v1` es annotated (tipo `tag`),
apunta a tag object `944946e3...` que apunta al commit `1deb371b0...`.
Usamos el commit SHA (1deb371b...) — es lo que usa GitHub Actions al resolver.

**Nota sobre `anomalyco/opencode`:** el repo no tiene branch `main` ni `master`,
default branch es `dev`. Sin semver; el comment `# dev` indica el branch.

## Estado

- Step 1: ✅ Resolver SHA-1 hashes
- Step 2: ✅ Edit `desktop.yml` (4 line changes — 3 user-named + `upload-artifact` para satisfacer grep contract)
- Step 3: ✅ Edit `opencode.yml` (2 line changes)
- Step 4: ✅ Verify full (grep VACÍO, YAML OK, actionlint 0)
- Step 5: ⬜ PENDING — commit

## Note sobre `actions/upload-artifact@v4` (bonus 6th pin)

El contract `grep -E "@(v[0-9]+|@latest)"` exige output VACÍO. El audit nombró 5 refs,
pero `desktop.yml:93` aún usaba `actions/upload-artifact@v4`. Como:
1. Todos los otros 13 workflows usan el mismo SHA `ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2`,
2. Sin pin de upload-artifact el grep no pasa,
3. Es first-party (GitHub), no tercero — pero SHA-pinning sigue siendo best practice,

Se pineó para satisfacer el contract literal. Documentado en el commit body.
Total: **6 line changes** (4 en desktop.yml + 2 en opencode.yml), no 5.

## Evidencia de verify (Step 4 ✅)

```
Select-String -Path desktop.yml,opencode.yml -Pattern "@(v[0-9]+|@latest)"
→ VACÍO (exit 1 = no match)

python -c "import yaml; [yaml.safe_load(open(f'.github/workflows/{f}')) for f in ['desktop.yml', 'opencode.yml']]; print('OK')"
→ OK

actionlint .github/workflows/desktop.yml .github/workflows/opencode.yml
→ 0 errors (exit 0, no output)
```

Refs pineadas:
```
desktop.yml:45   actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0
desktop.yml:52   actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4.4.0
desktop.yml:85   tauri-apps/tauri-action@1deb371b0cd8bd54025b384f1cd735e725c4060f # v1 (action-v1.0.0)
desktop.yml:93   actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
opencode.yml:24  actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6.1.0
opencode.yml:29  anomalyco/opencode/github@10765ff2a9da8c3b88e4de873aa383a49c318912 # dev
```

## CONTRACT

Verificación mecánica:
```bash
grep -E "@(v[0-9]+|@latest)" .github/workflows/desktop.yml .github/workflows/opencode.yml
# Expected: NO matches (empty output)

python -c "import yaml; [yaml.safe_load(open(f'.github/workflows/{f}')) for f in ['desktop.yml', 'opencode.yml']]; print('OK')"
# Expected: OK

actionlint .github/workflows/desktop.yml .github/workflows/opencode.yml
# Expected: 0 errors
```

## Evidencia

Pendiente de completar en CIERRE.