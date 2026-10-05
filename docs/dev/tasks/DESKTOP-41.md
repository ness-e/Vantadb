---
title: "TASK DESKTOP-41: Smoke-test instalador en VM Windows limpia"
kind: task
description: "Build local de instaladores NSIS+MSI (6m46s, exit 0) + evidencia parcial estática read-only (deep link vanta://, sidecars, WebView2 en ambos instaladores) + checklist ejecutable + FIND-272 (smoke real pendiente: no hay VM limpia disponible)"
---

# TASK DESKTOP-41: Smoke-test instalador en VM Windows limpia

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 32, F0 expandido)
- **Fuente:** Backlog `DESKTOP-41` + Step 3 de `DESKTOP-24` (smoke pendiente desde 2026-08-25)
- **Esfuerzo:** 🟡 1d | **Appetite:** max 2d | **Prioridad:** 🟡
- **Tipo:** Docs + artefactos (build local; **sin cambios de código** — no feature-add: no agrega símbolos/contratos públicos)
- **Turns estimados:** 15-30 (consumidos ~10)
- **Creado:** 2026-10-04 (DISCOVERY) | **last-synced:** 2026-10-04
- **Estado:** ✅ COMPLETED (fallback del plan ejecutado: checklist + FIND-272; **smoke real ⬜ pendiente de VM limpia** — stop condition del plan, sin evidencia simulada)
- **Campaign ID:** master-plan-0.9.0-20261004 (taskId `32`)
- **Incógnitas (uphill):** 0 abiertas — resueltas en DISCOVERY: (1) bundle local **no existía** → build ejecutado y medido (6m46s, exit 0); (2) VM limpia **no disponible** (Hyper-V no enumerable — `Get-VM` → permiso denegado sin elevación; sin ISOs, sin VirtualBox/VMware/Sandbox/multipass, shell sin elevación) → stop condition del plan; (3) deep link: **registro verificado estáticamente en ambos instaladores** (NSIS script :650-653 + MSI Registry table) — funcional ⬜ pendiente en VM
- **Pendientes (downhill):** 5 steps (5 ✅); el smoke real en VM queda como FIND-272 (no es step de esta tarea: requiere entorno que no existe)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (entrantes) | `docs/dev/Backlog.md` (fila DESKTOP-41 → removida al cierre; FIND-272 nueva), `docs/dev/avance/activo/desktop.md` (registro de cierre), `docs/index.md`/`llms.txt` (generados — incluyen el checklist nuevo), `docs/dev/tasks/DESKTOP-24.md` (Step 3 histórico — no se toca), master plan Task 32 (prohibido tocar) |
| Callees (salientes) | `desktop/src-tauri/tauri.conf.json` (bundle targets + `embedBootstrapper` + `resources`), `desktop/src-tauri/binaries/{vanta-cli.exe,vantadb-server.exe}` (gitignored — no se commitean), Tauri bundler (NSIS/WiX), `.github/workflows/desktop.yml` (CI build-only, sin smoke), docs oficiales Tauri v2 (windows-installer, deep-linking) |
| Implicaciones | **Cero código tocado.** Artefactos en `desktop/src-tauri/target/` (gitignored). Diff = docs + Backlog + generados. Sin breaking, sin deps, sin hot paths. El smoke real queda delegado a una sesión con VM limpia (FIND-272) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `desktop/src-tauri/tauri.conf.json` (69L), `desktop/src-tauri/Cargo.toml` (64L), `desktop/README.md` (144L), `docs/dev/tasks/DESKTOP-24.md` (67L), `.github/workflows/desktop.yml` (248L), `docs/dev/tasks/TS-11.md` (formato canónico de cierre, 239L), `.opencode/task-system/prompts/task.md` (398L), `.opencode/task-system/prompts/findings.md` (43L), `scripts/docs/lib.mjs` §deriveKind (:210-236), `scripts/docs/check-docs.mjs` (:20-89)
- **Lecturas puntuales (artefactos generados, read-only):** `desktop/src-tauri/target/release/nsis/x64/installer.nsi` (generado por el bundler), `desktop/src-tauri/target/release/wix/x64/main.wxs` (generado), tablas MSI vía WindowsInstaller COM
- **Referencias hacia dentro (imports/deps):** `tauri.conf.json` ← `tauri-build` (build.rs) + bundler; `binaries/` ← `bundle.resources` globs (`binaries/vanta-cli*`, `binaries/vantadb-server*`) + `child_process.rs::locate_binary()` en runtime; checklist nuevo ← `docs/index.md`/`llms.txt` (generados)
- **Referencias entrantes a los editados:** Backlog ← pipeline/progreso; `docs/dev/avance/activo/desktop.md` ← skill progreso; generados ← `gen-index.mjs --write`
- **Veredicto impacto:** **BAJO** — todo el diff es docs/registros; ningún archivo de código, config de build ni workflow se modifica. Los instaladores son artefactos regenerables (gitignored). Gate D evaluado: **no dispara** (sin símbolos públicos nuevos, sin blast radius >10, contrato del plan sanciona el fallback por stop condition)

## Contrato

> Del plan (Task 32): "instalador smoke-testeado en VM limpia: arranque + sidecar server + deep link `vanta://` + WebView2 (bootstrapper embed) verificados; checklist con evidencia (capturas/logs por paso); hallazgos → FINDs (no fixes apurados fuera de scope); tanto NSIS como MSI probados (o el que falle → FIND con repro)."
>
> **Stop condition aplicada (plan):** "sin VM limpia disponible → entregar checklist ejecutable + FIND del smoke pendiente (no simular evidencia)."

Entregables de esta tarea:

1. **Instaladores generados** (Step 1): `npm run tauri build` exit 0, **6m46s** (cold); NSIS 12.18 MB + MSI 16.75 MB con SHA256. ✅
2. **Evidencia parcial estática read-only** (Steps 2-3): registro `vanta://` verificado en NSIS script y MSI Registry table; sidecars incluidos en ambos; WebView2 bootstrapper embebido en ambos (MSI custom action condicionada + NSIS sección embedBootstrapper). ✅
3. **Checklist ejecutable** (Step 4): [INSTALLER_SMOKE_CHECKLIST.md](../desktop/INSTALLER_SMOKE_CHECKLIST.md) — pasos por instalador con comandos exactos, expected values, captura de evidencia y criterios pass/fail. ✅
4. **FIND-272** (Step 5): smoke real en VM limpia pendiente, con origen y acción. ✅
5. **Smoke real en VM limpia** (arranque + sidecar + deep link funcional + WebView2 install-time): ⬜ **pendiente → FIND-272** (requiere entorno inexistente en esta máquina).

**Comandos de verificación:** por step (abajo); cierre = gates docs (`check-links` · `check-docs` · `gen-index --write` + `--check` · `validate-docs-coverage`) + `pwsh dev-tools/ocr-review.ps1` + review P2-01.

## Spec (SDD — no feature-add; decisiones resueltas por evidencia)

> Gate mecánico: la tarea NO agrega símbolos/contratos públicos (no toca código) → feature-add = false. Igual se documentan las decisiones técnicas abiertas:

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Ejecutar el build aunque no haya VM? | A) sí — artefactos listos + costo medido + habilita inspección/evidencia parcial (pre-mortem #1 lo pide) / B) no — solo checklist | A | ✅ decidido-por-evidencia: plan pre-mortem #1 ("primer step es `npm run tauri build`, medir costo/tiempo") + pre-mortem #2 ("entregar checklist + evidencia parcial") |
| 2 | ¿"Probar" instalando en la máquina dev? | A) no — dev machine tiene WebView2/dev tools/registro previo: NO es limpia → evidencia inválida / B) sí | A | ✅ decidido-por-evidencia: stop condition "no simular evidencia"; la máquina es el entorno de desarrollo (AZW EQ físico, no VM) |
| 3 | Método de evidencia parcial | A) inspección read-only de artefactos (MSI tables vía COM + scripts generados NSIS/WiX) / B) ninguna | A | ✅ decidido-por-evidencia: pre-mortem #3 pide "verificación específica post-install" del deep link — la inspección cubre el **registro en instalación** (no el runtime) |
| 4 | Ubicación del checklist | A) `docs/dev/desktop/INSTALLER_SMOKE_CHECKLIST.md` (durable, junto a ARCHITECTURE.md) / B) solo en el task file | A | ✅ decidido-por-evidencia: el task file se archiva; el owner ejecutará el checklist en la VM (FIND-272) y necesita el artefacto vivo |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **NO instalar en la máquina dev** como sustituto de VM limpia; (2) **no simular evidencia** (prohibido capturas/logs inventados o "asumir" que el smoke pasó); (3) `desktop/src-tauri/binaries/` es gitignored — no commitear sidecars; (4) no tocar `opencode.jsonc`, master plan, `docs/pipeline-state.json`; **PROV-12 en vuelo** — no tocar `providers/**` ni sus workflows; (5) commit **LOCAL**, nunca push (Regla 7); (6) el checklist debe seguir siendo ejecutable sin leer el task file (autocontenido).
- **Comandos de verificación:** `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · `node scripts/docs/gen-index.mjs --check` · `pwsh scripts/validate-docs-coverage.ps1` · markdownlint de los archivos tocados.
- **Deuda pendiente:** smoke real en VM limpia (FIND-272) — incluye verificación funcional del deep link y del WebView2 install-time.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto cero:** no se toca código; no se agregan deps ni artefactos versionados (los instaladores viven en `target/`, gitignored). Sin `unsafe`/`unwrap`/clones. La única "deuda" es la cobertura de verificación incompleta por entorno — registrada como FIND-272, no como deuda de código.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-4 ✅ (build + evidencia estática + checklist + FIND); 5 (smoke real) explícitamente diferido por stop condition del plan con FIND trazable |
| **Commit** | Commit único `docs(desktop):` (local, sin push), gates docs verdes, solo archivos del blast radius (pathspec — WIP ajeno presente: master plan/opencode.jsonc/providers locks) |
| **Release** | n/a — sin cambio de código ni de artefactos versionados (justificado: no hay release que validar) |

## Herramientas necesarias

- `npm run tauri build` (build de instaladores; medido 6m46s cold)
- WindowsInstaller COM (`New-Object -ComObject WindowsInstaller.Installer`) — inspección MSI read-only
- `node scripts/docs/*.mjs` + markdownlint + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1` (cierre)

**Skills cargadas (SDP):** `shipping-and-launch` (checklist pre-launch de distribución) · `ci-cd-and-automation` (artefactos CI vs local, gates) · `documentation-skill` (OBLIGATORIA — se crean/editan `.md` bajo `docs/`) · `source-driven-development` (docs oficiales Tauri v2 windows-installer + deep-linking citadas) · `incremental-implementation` (slices: build → inspección → checklist → cierre). Base auto: campaign-executor, progreso, ponytail. SDP v3: 8 devueltas, 5 cargadas (TDD/doubt/frontend-ui descartadas: sin código).

## Investigation Notes

### DISCOVERY — disponibilidad de VM limpia (2026-10-04)

| Señal | Resultado |
|-------|-----------|
| Hyper-V services | `vmms` + `vmcompute` **Running**; cmdlet `Get-VM` disponible |
| VMs existentes | **no enumerable** — `Get-VM` → *permiso denegado* (shell sin elevación, usuario fuera de Hyper-V Administradores); listing del dir Hyper-V → access denied. Señales sustitutas: sin VBox/VMware/multipass/Sandbox/WSL, sin ISOs, sin store público de VMs → **ninguna VM limpia utilizable** |
| VirtualBox / VMware / multipass | **No instalados** (paths default ausentes; `vmrun`/`VBoxManage` no existen) |
| Windows Sandbox | **No disponible** (`WindowsSandbox.exe` ausente; feature check requiere elevación) |
| WSL | Sin distribuciones instaladas |
| ISOs de Windows | **Ninguna** en `C:\`, `%USERPROFILE%\Downloads`, `D:\` |
| Elevación del shell | **No** (`IsInRole(Administrator) = False`) → crear/gestionar VMs requeriría elevación |
| Esta máquina | AZW EQ (mini PC físico) — entorno de desarrollo con Rust/Node/WebView2; **no es limpia** |

**Conclusión:** no hay VM/entorno limpio disponible → **stop condition del plan** → checklist + FIND, sin simular evidencia.

### Build — costo medido (2026-10-04)

- `cd desktop && npm run tauri build` → **exit 0, 6m46s total** (cargo release cold 5m43s + bundling ~1m; sin target cache previo).
- Artefactos:
  - NSIS: `desktop/src-tauri/target/release/bundle/nsis/vantadb-desktop_0.1.0_x64-setup.exe` — 12.18 MB — SHA256 `6B30B04439E6A493C3A70D9FD1BA7E8F544DAA5742FC9EDA95FA0AEA61511E4F`
  - MSI: `desktop/src-tauri/target/release/bundle/msi/vantadb-desktop_0.1.0_x64_en-US.msi` — 16.75 MB — SHA256 `0069AB1AEEBA33C44D1364DA8001AB246D72C2D49E5A2271C0F66C988716F668`
- El bundler descargó el WebView2 bootstrapper de Microsoft durante el build (`go.microsoft.com/fwlink/p/?LinkId=2124703`) para embeberlo.

### Evidencia parcial estática — read-only, NO sustituto del smoke

**MSI** (WindowsInstaller COM, `msiOpenDatabaseModeReadOnly`):

| Tabla | Hallazgo |
|-------|----------|
| Property | `ALLUSERS=1` (per-machine → Program Files) · `ProductName=vantadb-desktop` · `ProductCode={2B59945A-0283-4D40-80AF-F3AF74F4B368}` · `UpgradeCode={7A138BB2-38A0-52A4-946E-4424D3F1F0BD}` · `ProductVersion=0.1.0` |
| Registry | `Software\Classes\vanta` = `URL Protocol` (vacío) + descripción `URL: protocol` (NSIS escribe `URL:com.vantadb.desktop protocol` — diferencia solo de texto, no funcional) · `...\DefaultIcon` = `"[!Path]",0` · `...\shell\open\command` = `"[!Path]" "%1"` (Root=2 → HKCR) · `Software\vantadb\vantadb-desktop\InstallDir` (HKLM) |
| File | `vantadb-desktop.exe` (19,512,832 B) · `vantadb-server.exe` (12,691,968 B) · `vanta-cli.exe` (10,149,888 B) — **sidecars incluidos** |
| Binary | `MicrosoftEdgeWebview2Setup.exe` embebido |
| CustomAction | `InvokeBootstrapper` (type 1026) → `/silent /install`, deferred, `Return=check`; condición `NOT(REMOVE OR INSTALLED_WEBVIEW2_VERSION)` antes de `InstallFinalize` (`main.wxs:212-217`) |
| Directory | `INSTALLDIR = ProgramFiles64Folder\vantadb-desktop` |

**NSIS** (`target/release/nsis/x64/installer.nsi`, generado por el bundler):

| Línea | Hallazgo |
|-------|----------|
| :39 | `!define INSTALLMODE "currentUser"` → per-user, sin UAC; `$INSTDIR = $LOCALAPPDATA\vantadb-desktop` (:504) |
| :55-57 | `INSTALLWEBVIEW2MODE "embedBootstrapper"` + bootstrapper desde cache local `%LOCALAPPDATA%\tauri\MicrosoftEdgeWebview2Setup.exe` |
| :536+ | Sección WebView2: detecta runtime (GUID `{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}` en HKLM/HKCU EdgeUpdate) y lo omite si ya está |
| :642-643 | `File /a "/oname=vanta-cli.exe"` + `"/oname=vantadb-server.exe"` — **sidecars incluidos** junto al exe |
| :650-653 | Registro `Software\Classes\vanta`: `URL Protocol` + descripción + `DefaultIcon` + `shell\open\command` → `"$INSTDIR\vantadb-desktop.exe" "%1"` (`SHCTX` = HKCU en per-user) |
| :766-776 | Uninstall: borra sidecars y `DeleteRegKey SHCTX "Software\Classes\vanta"` (solo si el comando apunta a esta app) |

**Docs oficiales consultadas (source-driven):**

- [Tauri v2 — Windows Installer](https://v2.tauri.app/distribute/windows-installer/): NSIS default = current user (`%LOCALAPPDATA%`); `embedBootstrapper` = +~1.8 MB y **requiere internet en install-time**; MSI = WiX v3; en Win10 (2018+) / Win11 el runtime WebView2 viene con el OS.
- [Tauri v2 — Deep Linking](https://v2.tauri.app/plugin/deep-linking/): en Windows el deep link llega **como argumento de línea de comandos a un proceso nuevo** (integración con single-instance); "by default the deep link is only registered when your app is installed" → el registro post-install es lo que valida el smoke; trigger: `start vanta://...`.

### Lo que NO se verificó (explícito — va a FIND-272)

- Arranque real de la app **instalada** (ventana + UI sobre WebView2).
- Spawn real del sidecar (`vantadb-server.exe` / `vanta-cli.exe`) desde el install dir.
- Deep link **funcional** (`start vanta://smoke` con app corriendo y cerrada; forwarding single-instance).
- WebView2 install-time (ejecución del bootstrapper embebido en VM sin runtime).
- Upgrade/uninstall limpio post-install.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — VM ausente y deep-link registration resueltos con evidencia; resto es ejecución diferida (FIND-272) |
| Pendientes de ejecución (downhill) | 5 steps (5 ✅); smoke real ⬜ → FIND-272 (entorno externo requerido) |
| % completado | 100% del alcance posible en este entorno (fallback del plan); smoke real = 0% (FIND-272) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** no aplica cambios de trust boundary (sin código, sin deps, sin secrets). Nota: los instaladores son **UNSIGNED** (sin Authenticode — no hay certificado configurado en `tauri.conf.json`); SmartScreen warning es esperado en la VM y debe registrarse en el checklist, no tratarse como fallo del smoke (firma = decisión de distribución, fuera de scope — ver DESKTOP-43/DEVOPS-10).
- **PERFORMANCE:** no aplica — no toca hot paths; el build release (~7 min) queda medido en §Investigation Notes.

## Steps

### Step 1 — Build de instaladores NSIS+MSI (medir costo)
- **Archivos:** `desktop/src-tauri/target/release/bundle/**` (generados, gitignored)
- **Acción:** `cd desktop && npm run tauri build` (cold, sin target previo) con stopwatch; verificar exit 0 + ambos bundles + SHA256.
- **Verify:** exit 0 + `bundle/nsis/*.exe` + `bundle/msi/*.msi` presentes
- **Resultado:** ✅ exit 0 en **6m46s**; NSIS 12.18 MB (`6B30B044…`) + MSI 16.75 MB (`0069AB1A…`)
- **Estado:** ✅

### Step 2 — Inspección estática read-only del MSI (COM)
- **Archivos:** MSI (lectura) — script temporal de inspección (no versionado)
- **Acción:** `WindowsInstaller.Installer` (read-only): tablas Property/Registry/File/Shortcut/Directory/Binary/CustomAction → deep link, sidecars, WebView2, install scope.
- **Verify:** Registry table contiene `Software\Classes\vanta` + `URL Protocol` + `shell\open\command`; File table contiene los 3 exes; Binary contiene el bootstrapper
- **Resultado:** ✅ todo confirmado (tabla completa en §Investigation Notes)
- **Estado:** ✅

### Step 3 — Inspección estática NSIS + WebView2 + validación contra docs Tauri
- **Archivos:** `target/release/nsis/x64/installer.nsi`, `target/release/wix/x64/main.wxs` (lectura)
- **Acción:** grep dirigido: INSTALLMODE, sidecars, protocolo `vanta`, cleanup de uninstall, sección WebView2; contrastar semántica con docs oficiales Tauri v2 (windows-installer, deep-linking).
- **Verify:** :650-653 registro protocolo + :642-643 sidecars + :536+ WebView2 + :766-776 cleanup; docs citadas en §Investigation Notes
- **Resultado:** ✅ confirmado
- **Estado:** ✅

### Step 4 — Checklist ejecutable para VM limpia
- **Archivos:** `docs/dev/desktop/INSTALLER_SMOKE_CHECKLIST.md` (nuevo)
- **Acción:** checklist autocontenido (Test A NSIS / Test B MSI) con comandos exactos, expected values (de la evidencia estática), captura de evidencia por paso, criterios pass/fail, prerequisitos de VM, caveats (unsigned, red para bootstrapper, snapshot reset).
- **Verify:** `node scripts/docs/check-links.mjs` · `check-docs.mjs` · markdownlint
- **Resultado:** ✅ archivo creado (kind `howto`); gates verdes en Step 5
- **Estado:** ✅

### Step 5 — Cierre: FIND-272 + avance + gates docs + OCR + review P2-01 + commit + campaña
- **Archivos:** `docs/dev/Backlog.md` (FIND-272), `docs/dev/avance/activo/desktop.md`, `docs/index.md` + `llms.txt` (generados), este task file
- **Acción:** registrar FIND-272; remover fila DESKTOP-41 del Backlog (completada → avance); `gen-index --write`; verify full docs; OCR delegation; review P2-01 (vanta-review, contexto fresco); commit **LOCAL** `docs(desktop):` (pathspec propio); `campaign_update_task_state(taskId 32, completed)` con recitation + payload review; RESULTADO §7.
- **Verify:** gates docs verdes + veredicto review registrado + commit local
- **Resultado:** ✅ (detalle en §RESULTADO)
- **Estado:** ✅

### Evidencia de verificación (post-implementación, 2026-10-04)

| Gate | Comando | Resultado |
|------|---------|-----------|
| Build | `cd desktop && npm run tauri build` | ✅ exit 0, **6m46s** (cold); 2 bundles |
| Hashes | `Get-FileHash … -Algorithm SHA256` | ✅ NSIS `6B30B044…` · MSI `0069AB1A…` (coinciden con §Investigation Notes) |
| MSI tables | `desktop41-msi-inspect.ps1 -Msi <msi>` (COM read-only) | ✅ Registry `vanta` + File ×3 + Binary bootstrapper + CustomAction condicionada |
| NSIS script | grep dirigido sobre `installer.nsi` | ✅ :39/:55-57/:536+/:642-643/:650-653/:766-776 |
| WiX script | grep sobre `main.wxs` | ✅ :202-217 (RegistrySearch + Binary + InvokeBootstrapper + condición) |
| Sidecars runtime | `vanta-cli.exe --version` · `vantadb-server.exe --help` | ✅ `vanta-cli 0.5.0` · help OK (comandos incluidos en el checklist) |
| check-links | `node scripts/docs/check-links.mjs` | ✅ exit 0; broken markdown links 0 (budget 0) |
| check-docs | `node scripts/docs/check-docs.mjs` | ✅ exit 0; orphans propios resueltos (link en Contrato → checklist) |
| gen-index | `--write` + `--check` | ✅ in sync (1520 docs; +2 propios; nota: incluye el refresh mecánico de la fila PROV-12 — archivo en vuelo) |
| docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| markdownlint | `npx markdownlint-cli2 <4 archivos>` | ✅ 0 issues |
| OCR delegation | `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ **0 archivos reviewables del diff propio** (docs excluidos por diseño; los 7 reviewables son de PROV-12 en vuelo — no se commitean aquí) |

## RESULTADO (§7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5 total steps
PROXIMO_STEP: ninguno — orquestador: Task 32 registrado `completed` en campaña (taskId 32, payload review); smoke real en VM limpia = FIND-272 (owner-assisted); push diferido al owner (Regla 7)
COMMIT_HASH: <ver git log — docs(desktop): DESKTOP-41 (LOCAL, sin push)>
ARCHIVOS: docs/dev/tasks/DESKTOP-41.md · docs/dev/desktop/INSTALLER_SMOKE_CHECKLIST.md · docs/dev/Backlog.md (FIND-272 + fila DESKTOP-41 removida) · docs/dev/avance/activo/desktop.md · docs/index.md · llms.txt · desktop/src-tauri/Cargo.lock (refresh mecánico del build, commit aparte `chore(desktop):`)
VERIFY_CONTRATO: pasa (fallback stop-condition) — entregables 1-4 ✅ (build + evidencia estática + checklist + FIND); entregable 5 (smoke real) ⬜ diferido por entorno, registrado en FIND-272, sin evidencia simulada
BLOQUEO: entorno — no hay VM Windows limpia disponible (Hyper-V no enumerable sin elevación — `Get-VM` → permiso denegado; sin ISOs/alternativas); stop condition del plan aplicada
GATES_EVALUADOS: P:no(plan Task 32 ✅ DO) D:no(fallback sancionado por stop condition; sin símbolos públicos) V:no(verde 1er intento) C:si→FIND-272+auto(WIP ajeno fuera)
SKILLS_CARGADAS: shipping-and-launch, ci-cd-and-automation, documentation-skill, source-driven-development, incremental-implementation (SDP v3: 8 devueltas; TDD/doubt/frontend-ui descartadas — sin código; base auto: campaign-executor/progreso/ponytail)
```

## Dependencias

- **DESKTOP-24 Step 3** — esta tarea ejecuta su cierre por la vía del fallback; el smoke real queda en FIND-272.
- **DESKTOP-25 (CI desktop.yml)** — build-only; sube artefactos `vantadb-desktop-windows-installers` (alternativa para obtener instaladores sin build local).
- **FIND-272** — ejecución del checklist en VM limpia (owner-assisted).
- **En vuelo (no tocar):** PROV-12 (`providers/**` + workflows), master plan (modificado en el tree — pathspec), `opencode.jsonc`.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — sesión fresca `ses_ef5e58bb4ffeaZOBh8XSGS3Ew7` (2026-10-04), contexto distinto al implementador (P2-01).
- **Enfoque:** legitimidad del fallback por stop condition; realidad y precisión de la evidencia (rondas 2-3: M1 VM wording L19/L99/L230, M2 valor descripción MSI, L1 lockfile); ejecutabilidad del checklist; ausencia de evidencia simulada.
- **Cómo se probó:** re-ejecución de los 5 gates docs (exit 0) · hashes SHA256 y bytes exactos · COM read-only (Property/Registry/File/Binary/CustomAction/InstallExecuteSequence/Directory) · re-lectura NSIS/WiX + lib.rs/useDeepLink/WorkspaceShell · sidecars (`vanta-cli 0.5.0`, `vantadb-server --help`) · pathspec de commit y límite WIP · atestación temporal de `HKCU\Software\Classes\vanta` vía P/Invoke (2026-08-18 → pre-existente, no de esta tarea).
- **Checklist anti-hábitos tóxicos:** sin outputs inventados ✅ (2 defectos de precisión detectados y corregidos; cero fabricación) · sin verificación salteada ✅ (smoke diferido solo por stop condition explícita + FIND-272; 5 ítems no verificados enumerados en el task file) · sin "done" sin criterios ✅ (DoD multi-nivel; smoke ⬜) · sin parcial reportado como OK ✅.
- **Veredicto:** ✅ **APPROVE** — ronda 1 ❌ CHANGES-REQUIRED (M1/M2/L1) → fixes aplicados → ronda 2 pre-autorizó el APPROVE condicionado a las 2 correcciones residuales L19/L230, **aplicadas exactamente** (`rg "0 VMs|sin VMs"` = 0 matches) + gates verdes. Fallback stop-condition legítimo; evidencia real y precisa; commit local con pathspec propio y WIP ajeno fuera.

## Notas

- (DISCOVERY) La máquina del owner (AZW EQ) es el entorno de desarrollo — **no** se instaló nada en ella para "probar": sería evidencia inválida y potencialmente destructiva del entorno.
- (DISCOVERY) El build cold fue mucho más barato de lo presupuestado (6m46s vs "puede ser largo"); queda documentado para futuras tareas desktop (DESKTOP-43).
- (S2) **NOTICED BUT NOT TOUCHING:** `docs/dev/tasks/DESKTOP-24.md` Step 3 sigue "⬜ PENDING" — su cierre formal depende del smoke real (FIND-272); no se edita por scope discipline. El vínculo queda por FIND-272 (Origen: DESKTOP-41 · Step 3 DESKTOP-24).
- (S2) **NOTICED BUT NOT TOUCHING:** `desktop/README.md:124-129` ("No public installer yet") sigue siendo correcto — no hay canal público; los instaladores locales no cambian eso.
- (S3) **Side effect del build — `desktop/src-tauri/Cargo.lock`:** el build refrescó el lockfile (estaba stale desde DIST-01: `vanta-memory` **y** `vantadb` 0.7.0 → 0.8.0 + árbol `sha2 0.11`/`digest 0.11`). Es resolución mecánica de cargo (cualquier build desktop lo haría; precedente `80d44b3f` "tauri lockfile bumps"); se commitea **separado** como `chore(desktop):` para no mezclarlo con el diff docs. Sin impacto en el contrato (artefactos ya generados con el lock nuevo).
- Los scripts temporales de inspección MSI viven en `%TEMP%` (no versionados); los comandos quedan inline en el checklist para reproducibilidad.
