---
title: Instalador interactivo y personalizado (RES-17)
kind: research
status: active
description: "Benchmark multi-fuente de instaladores interactivos (rustup, ollama, Docker, VS Code, uv, cargo-dist, Homebrew, conda, winget, Tauri/NSIS, Inno Setup, MSI, Python) y recomendación de arquitectura para el instalador v2 de VantaDB: selector de módulos, resolución de binarios/dependencias/config por plataforma, roadmap por fases y riesgos. Alimenta DX-12"
tags: [vantadb, research, cli, user, mcp]
---

# Instalador interactivo y personalizado (RES-17)

> Fila del backlog: `RES-17` (ex `INV-installer-01`; alimenta `DX-12`). Investigación cerrada el 2026-10-01. Todas las URLs se verificaron por fetch directo esa fecha, salvo las marcadas `[cita NO VERIFICADA]`.

## Contexto VantaDB (verificado en repo)

- `scripts/install.ps1` / `scripts/install.sh`: one-liner que detecta OS/arch, descarga `vantadb-<target>.tar.gz` + `.sha256` desde GitHub Releases, verifica checksum (warn-and-continue si falta el asset), hace backup `.bak-<stamp>` del binario previo y encadena el wizard `setup-embeddings.ps1` (descargado del tag de release, vía `pwsh`).
- `setup-embeddings.ps1` (raíz): wizard con Enter=default (`Read-WithDefault`), tabla de modelos con tamaño y tiempo estimado (~10 MB/s), runtime ORT nativo persistente en `%LOCALAPPDATA%\VantaDB\onnxruntime`, probe de Ollama (nunca instala), política de secretos (solo env de sesión, nunca a disco) y test en vivo.
- Release matrix (`.github/workflows/release-binaries.yml`): 5 targets (x86_64/aarch64 linux+darwin, x86_64-pc-windows-msvc); empaqueta `vanta-cli` + `vantadb-server` por target con `.sha256`.
- `vanta-cli` es binario unificado (memoria + `server` HTTP/MCP + migrate + certificate/verify); `vanta-proxy` es crate separado (fuera de `default-members`, experimental); el visor desktop vive en `desktop/` (Tauri v2).
- `embeddings/manifest.json` v1: 9 modelos, desde 80 MB (`all-MiniLM-L6-v2`) hasta 2.27 GB ONNX (`bge-m3`; `qwen3-embedding-8b` es GPU-only sin ONNX, 16 GB). Default: `multilingual-e5-small` (220 MB ONNX).
- Visión owner (2026-10-01, fila `DX-12`): una instalación interactiva y personalizada que pregunte qué habilitar (motor / MCP / server / proxy / visor desktop / embeddings / providers) y descargue o genere binarios + dependencias + configuración.

## Resumen ejecutivo

1. **Dos arquitecturas dominantes** y no son excluyentes: instalador *fetching* (script delgado que detecta plataforma y descarga binarios precompilados — shell/powershell/npm/homebrew de cargo-dist) e instalador *bundling* (MSI/NSIS/pkg que embebe binarios y funciona offline — MSI de cargo-dist, Tauri). El wizard interactivo es una capa delgada sobre un **modelo declarativo de componentes** (rustup profiles, Inno Setup `[Components]`+`Types`, features MSI con `ADDLOCAL`, workloads de VS).
2. **Paridad no-interactiva es obligatoria**: todos los referentes tienen ruta CI (rustup `-y`, Homebrew `NONINTERACTIVE=1`, conda `-b`, Docker `--quiet --accept-license`, VS `--quiet`/`--passive`, winget `--silent --disable-interactivity`). clig.dev lo eleva a regla: nunca *requerir* un prompt; si stdin no es TTY, saltear y exigir flags.
3. **Defaults seguros + divulgación progresiva**: rustup ofrece `minimal`/`default`/`complete` y recomienda `default`; Docker instala per-user por defecto; Tauri `installMode` per-user por defecto. La selección gruesa va primero (perfiles), el detalle después (`rustup component add`).
4. **Cadena de confianza**: HTTPS + checksums sha256 es el piso (VantaDB ya lo hace; cargo-dist y miniforge publican `.sha256`; la API de GitHub expone `digest` por asset). Las firmas son el techo: rustup admite que no firma; el updater de Tauri exige firma; macOS exige codesign+notarización; el Python install manager firma sus índices con Authenticode. VantaDB debería pasar de sha256 a firma (minisign/cosign) antes de promover el visor desktop.
5. **Disciplina de config/estado**: XDG en Unix (`XDG_CONFIG_HOME`/`XDG_DATA_HOME`/`XDG_STATE_HOME`/`XDG_CACHE_HOME`), `%APPDATA%`/`%LOCALAPPDATA%` en Windows; overrides por env var para cada ruta (`CARGO_HOME`/`RUSTUP_HOME`, `HF_HOME`, `UV_NO_MODIFY_PATH`); desinstalación documentada (uv, miniforge, `rustup self uninstall`, `py uninstall --purge`); re-ejecución idempotente (Brewfile `check`/`install`, edición de PATH solo-si-falta de cargo-dist).

## 1. Referentes: instalación interactiva + selección de componentes

### 1.1 Tabla comparativa

| Referente | Selección de componentes | No-interactivo / CI | Verificación | Update | Ciclo de vida / notas |
|---|---|---|---|---|---|
| rustup | Profiles minimal/default/complete + "Customize installation" + `--profile`; componentes post-install con `rustup component add` | `-y`, `--no-modify-path`, `--default-toolchain none` | HTTPS; `.sha256` disponibles; sin firmas (declarado) | `rustup update` / `self update`; `self uninstall` | `CARGO_HOME`/`RUSTUP_HOME`; PATH vía env script; AV con perfil completo (recomiendan minimal en Windows) |
| ollama | Sin selector: 1 binario + modelos on-demand (`ollama run`) | `install.sh` no pregunta; `OLLAMA_VERSION` pin | Sin checksum visible en el script (HTTPS + `--fail`) | Re-run del script reemplaza | systemd/launchd; detección GPU NVIDIA/AMD; `main()` anti-descarga-truncada |
| Docker Desktop | Wizard GUI: per-user vs all-users, backend WSL2/Hyper-V | `install --user/--quiet/--accept-license/--backend` | Checksums en release notes | In-app update | Cambiar modo requiere desinstalar (anti-patrón); flags de proxy |
| VS Code (Inno Setup) | User vs System setup; ZIP portable; tasks (PATH, context menu) | Switches Inno: `/VERYSILENT`, `/MERGETASKS` (FAQ) | Firmas Authenticode | Auto-update in-product | User setup recomendado; PATH requiere consola nueva |
| uv | 1 binario; instaladores standalone + package managers | `install.sh`/`install.ps1`; URL con versión pin | GitHub Releases (sin firma en script) | `uv self update` (re-ejecuta installer); `UV_NO_MODIFY_PATH` | Desinstalación documentada (binarios + caches) |
| cargo-dist | `dist init` UI interactiva: qué installers habilitar (shell, powershell, npm, homebrew, msi) | Los installers no preguntan; fetching vs bundling | `.sha256` por artefacto | Re-run del installer | PATH: env script + `.profile` solo-si-falta; no puede editar el PATH de la shell actual |
| Homebrew / Brewfile | Brewfile declarativo (formulae, casks, mas, vscode, cargo, uv, winget...) | `NONINTERACTIVE=1` | HTTPS + taps confiables; sin firmas propias | `brew bundle` upgrade | `check/install/dump/cleanup` idempotente; mirrors vía `HOMEBREW_*` env |
| miniforge/conda | Interactivo pregunta init de shell | `-b -p <prefix>`; Windows `/S /D=` | SHA256 publicado por release | `conda update` | Desinstalación documentada (reverse init + rm); warnings de PATH (conflictos) |
| winget | `--scope user/machine`, `--id`, `--version`; export/import de lista | `--silent`, `--accept-package-agreements`, `--disable-interactivity` | Hash del instalador verificado por winget | `winget upgrade` | Fuente de terceros; `--proxy`; instalación de múltiples paquetes en secuencia |
| Tauri (NSIS/WiX) | `installMode`: per-user/perMachine/both; WebView2: download/embed/offline/fixed/skip | NSIS `/S`; MSI `qn` | Firma de updater obligatoria; codesign+notarize macOS | `tauri-plugin-updater` (JSON estático o server, firma obligatoria) | Hooks NSIS para deps (VC++ redist con `/passive /norestart`); i18n multi-idioma |
| Inno Setup | `[Components]` + `Types` (full/compact/custom), flags `fixed`/`exclusive` | `/SILENT /VERYSILENT /SUPPRESSMSGBOXES /COMPONENTS /TASKS /MERGETASKS /TYPE /SAVEINF /LOADINF` | Depende del firmado del setup | Re-run | Log `/LOG`; `/CURRENTUSER`/`/ALLUSERS` |
| MSI (WiX) | Features; `ADDLOCAL=f1,f2` o `ADDLOCAL=ALL`; `REMOVE` | `msiexec /q[n|b|r|f] /i|/x /norestart /l*v` | Authenticode del .msi | MSI major upgrade | Propiedades públicas por CLI; ADDLOCAL en CLI, no en Property Table |
| Python install manager | `py install <tag>`; MSIX/MSI; runtimes por usuario | `--dry-run`, `--yes`; winget `--disable-interactivity` | Índices firmados (Authenticode `.cat`) opcional y pinneable | `py install --update` | Offline index (`--download` + `--source`); proxy `HTTP(S)_PROXY`/`NO_PROXY`; `uninstall --purge` |
| VS Build Tools | Workloads/components `--add`, `;includeRecommended/Optional`; `.vsconfig` exportable | `--quiet`/`--passive`, `--norestart`, `--wait` | Firma de Microsoft | `update`/`modify`/`rollback` | Layout offline `--layout` + `--noWeb`; exit codes documentados |

### 1.2 Patrones transversales

**Detección de plataforma/arquitectura.** Ollama mapea `uname -m` (`x86_64`→`amd64`, `aarch64|arm64`→`arm64`) y falla explícito en arquitecturas no soportadas; rustup publica `rustup-init` por target Rust; winget expone `--architecture`. VantaDB ya normaliza a triples Rust en `install.sh`.

**Prompts con defaults.** El patrón común es "Enter = default" y default documentado (rustup `default` profile; Docker per-user; VantaDB `Read-WithDefault`). Los prompts deben mostrar el default en el texto (`[S/n]`) — VantaDB ya lo hace.

**Perfiles antes que componentes.** rustup: `minimal`/`default`/`complete` + "Customize installation" para granularidad. Inno Setup: `Types: full compact custom` + componentes con `Flags: fixed`/`exclusive`. VS: workloads + `--includeRecommended/--includeOptional`. Traducción para VantaDB: `minimal`/`default`/`full`/`custom`.

**Modo no-interactivo.** Flags explícitos (`-y`, `--quiet`, `--silent`, `--disable-interactivity`, `--passive`) y detección de TTY. clig.dev: `--no-input` para desactivar todo prompt; si falta un dato, fallar indicando el flag. Nota: instalar sin TTY jamás debe colgar (riesgo #1 en CI).

**Verificación.** sha256 por artefacto (VantaDB, cargo-dist, miniforge); `digest` en la API de Releases de GitHub; firmas: updater de Tauri (obligatoria), Python index signatures (Authenticode con pin de root/editor), rustup sin firmas (declarado en su página de seguridad). Docker: checksums en release notes.

**Updates.** `uv self update` (re-ejecuta el instalador; `UV_NO_MODIFY_PATH` evita re-editar el shell), `rustup update`, `brew bundle upgrade`, `winget upgrade`, updater de Tauri (JSON estático o server dinámico, firma obligatoria, `204` = sin update, canales por endpoint). VS documenta `update`, `modify`, `rollback` y exit codes.

**PATH y shell.** Los instaladores no pueden modificar el PATH de la shell actual (cargo-dist lo documenta): escriben un env script y una línea `source` solo-si-falta en `.profile`. En Windows el PATH de usuario vs máquina es una decisión de instalación (Docker per-user/all-users; Tauri NSIS `PathEnvVarFeature`; VS Code advierte reiniciar la consola).

**Desinstalación.** `rustup self uninstall`; uv documenta pasos manuales (caches + binarios); miniforge documenta `conda init --reverse` + rm de prefijo + `.condarc`/`.conda`; Python `py uninstall --purge`. Lección: documentar el desinstalador desde el día 1 y no dejar entries de PATH huérfanos.

## 2. Descarga/generación de binarios y dependencias

### 2.1 Binarios por plataforma

- Precompilados por target en GitHub Releases es el estándar de facto; cargo-dist automatiza la matriz y genera instaladores fetching que detectan plataforma y eligen el archivo correcto. VantaDB ya tiene 5 targets y `.sha256` por artefacto.
- Estrategia fetching vs bundling: fetching = artefactos siempre actuales, instalador chico, requiere red y URL estable; bundling = offline, pero single-platform y más pesado (cargo-dist lo declara para MSI; Tauri WebView2 offline agrega ~127 MB).
- Recomendación: CLI/proxy por fetching (como hoy); desktop por bundling (NSIS/MSI) con WebView2 `downloadBootstrapper` por defecto y opción `offlineInstaller` documentada.

### 2.2 Modelos ONNX

- `huggingface_hub` documenta: cache en `HF_HOME` (default `~/.cache/huggingface`, respeta `XDG_CACHE_HOME`), `HF_HUB_CACHE`, timeouts (`HF_HUB_ETAG_TIMEOUT`, `HF_HUB_DOWNLOAD_TIMEOUT`), modo offline (`HF_HUB_OFFLINE=1`), telemetría off (`HF_HUB_DISABLE_TELEMETRY`, `DO_NOT_TRACK`) y caveat de symlinks en Windows (developer mode o `HF_HUB_DISABLE_SYMLINKS`).
- El espejo comunitario por `HF_ENDPOINT` no figura en la página oficial de variables de entorno (verificado: ausente) `[cita NO VERIFICADA]`; no prometer mirrors propios; ofrecer en su lugar modo offline con cache pre-poblada y bundle air-gapped.
- Tamaños reales del manifest: 80 MB–2.27 GB; el wizard ya muestra tamaño/tiempo y eso debe mantenerse como regla (nunca descargar >100 MB sin confirmación explícita).
- ONNX Runtime: en Windows requiere Visual C++ 2019 redistribuible; los builds custom permiten recortar tamaño. El wizard ya resuelve el ORT nativo (>=1.27) en ubicación persistente; eso debe sobrevivir como módulo `embeddings.runtime`.

### 2.3 Runtimes opcionales

- Ollama: los referentes lo instalan con script one-liner (`curl` a `sh` / `irm` a `iex`) o instalador GUI; su script detecta GPU (NVIDIA/AMD/ROCm), crea servicio systemd y usa `OLLAMA_VERSION` para pin. VantaDB debe **detectar y guiar** (como hoy: probe `localhost:11434`), nunca instalar silenciosamente algo system-wide.
- Python para adapters: el nuevo Python install manager soporta `py install`, índice offline (`--download` + `--source`), `--dry-run`, proxy (`HTTP(S)_PROXY`, `NO_PROXY`) y desinstalación; alternativa embebible: `py install 3.14-embed --target=<dir>` (aislado del sistema).
- Node/npm: fuera de alcance MVP; si se habilita, vía package manager del usuario.

### 2.4 Offline / air-gapped y proxies

- Offline: `cargo vendor` + `--offline`/`--frozen` para builds; VS layout `--layout --noWeb`; Tauri `offlineInstaller`; Python offline index; miniforge `-b -p` no toca el shell.
- Proxies corporativos: `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY` documentados por Python; Docker tiene flags `--proxy-http-mode`/`--override-proxy-*`; winget `--proxy`. Riesgo transversal: TLS interception rompe verificaciones si no se confía en la CA corporativa.
- Reintentos/resume: los referentes delegan en curl/wget (`--fail --location --progress-bar` en Ollama; cargo-dist depende de curl/wget del usuario) y no implementan resume sistemático. Oportunidad para VantaDB: `--retry`/reanudación en el downloader propio.

## 3. Configuración generada

### 3.1 Dónde vive

- XDG (Unix): config en `XDG_CONFIG_HOME` (default `~/.config`), datos en `XDG_DATA_HOME` (`~/.local/share`), estado en `XDG_STATE_HOME` (`~/.local/state`), cache en `XDG_CACHE_HOME` (`~/.cache`); ejecutables de usuario en `~/.local/bin`.
- Windows: `%APPDATA%` (Roaming) para config — el Python install manager usa `%AppData%\Python\pymanager.json` — y `%LOCALAPPDATA%` para datos; el wizard de VantaDB ya usa `%LOCALAPPDATA%\VantaDB\onnxruntime`.
- Precedencia documentada en referentes: config file < env vars < CLI flags (Python); registry admin override en Windows.
- VantaDB hoy: `~/.vanta/bin` para el binario. Propuesta: `~/.vanta/` como home lógico (compatibilidad) con subrutas XDG-aware en Unix y `%APPDATA%\VantaDB` para config en Windows; `VANTA_HOME` como override único.

### 3.2 Plantillas y edición idempotente

- Plantillas declarativas: Brewfile (estado deseado, no comandos), `.vsconfig` (export/import de workloads), `dist init` (config de installers). Para VantaDB: un manifest `installer/modules.toml` versionado como fuente de verdad.
- Edición de archivos existentes: patrón cargo-dist/rustup = agregar línea solo-si-falta (`.profile`); bloques MCP por cliente deben usar marcadores (`# >>> vantadb mcp >>>`) para enable/disable sin duplicar.
- Secretos: mantener la política actual (nunca a disco; solo env de sesión). clig.dev: nunca leer secretos por flag (`--password`); usar archivo/stdin.

### 3.3 Idempotencia, updates y desinstalación

- Re-ejecutar debe ser no-op si el estado ya es el deseado (Brewfile `check || install`). VantaDB ya hace backup del binario antes de reemplazar — conservarlo.
- `enable/disable` posterior: Docker exige desinstalar para cambiar per-user/all-users (anti-patrón a evitar); VS `modify`; Tauri `installMode: both`. VantaDB debe modelar el estado en config y permitir `vanta-cli setup --enable/--disable <módulo>`.
- Updates: `uv self update` re-ejecuta el instalador (idempotente); el updater de Tauri con firma + JSON estático sirve para el desktop; el CLI puede seguir con re-descarga + sha256 (y firma en fase 3).
- Migración de config: `vanta-cli migrate` ya existe para datos; agregar `config_version` en el config generado y migraciones explícitas.

## 4. Riesgos y lecciones

| Riesgo | Evidencia | Mitigación propuesta |
|---|---|---|
| Piping de script a shell ejecuta script truncado | Ollama envuelve todo en `main()` para que una descarga parcial no ejecute medio script | Mismo patrón + verificar sha256 del payload antes de ejecutar |
| Instalador cuelga en CI (prompt sin TTY) | clig.dev: prompts solo si stdin es TTY; `--no-input` | Detección TTY + `--non-interactive` con defaults; fallar con mensaje si falta un dato |
| PATH no actualizado en la sesión | cargo-dist documenta que es imposible editar el PATH del shell padre | Imprimir `source <env>` / reiniciar consola; en Windows usar PATH de usuario |
| Antivirus / SmartScreen / Gatekeeper | rustup recomienda `minimal` en Windows por AV con muchos archivos; Tauri: sin notarización macOS muestra "app dañada" | Firma de código (fase 3), evitar miles de archivos, documentar SmartScreen |
| Descargas grandes de modelos | manifest: hasta 2.27 GB ONNX (1.2 GB int8) | Defaults livianos (220 MB), confirmación explícita, estimaciones honestas, cache reutilizable |
| Cambiar modo de instalación requiere reinstalar | Docker per-user/all-users exige uninstall/reinstall | Estado en config + `--enable/--disable`; nunca decisiones irreversibles en prompts |
| Proxies corporativos / TLS interception | Python documenta HTTP(S)_PROXY/NO_PROXY; Docker flags de proxy | Soporte env vars + `--proxy`; mensajes de error que distingan TLS/CA |
| Desinstalación sucia | referentes documentan pasos manuales (uv, miniforge) | `vanta-cli setup uninstall` con limpieza de PATH, config y (opcional) datos/modelos |
| Fatiga de prompts | rustup resuelve con profiles; Inno con Types | Perfiles primero; máximo ~5 preguntas en `default`; resumen final antes de aplicar |
| i18n / accesibilidad de prompts | Tauri NSIS multi-idioma + selector; clig.dev: color nunca único portador de significado, `NO_COLOR` | ES/EN, texto plano, sin depender de color, `--no-color`/`NO_COLOR` |

## 5. Recomendación para VantaDB (instalador v2)

### 5.1 Arquitectura

1. **Un solo wizard nativo: `vanta-cli setup`.** Los scripts `install.ps1`/`install.sh` quedan como bootstrap delgado (detectar target → descargar `vanta-cli` + `.sha256` → ejecutar `vanta-cli setup`). Elimina la dependencia de `pwsh` en Linux/macOS (hoy el wizard es un `.ps1` descargado del tag), la segunda descarga de red y la duplicación de UX en dos lenguajes. Es el patrón de uv/cargo-dist: instalador delgado + binario rico.
2. **Manifest declarativo de módulos** (`installer/modules.toml`, versionado en el repo): cada módulo declara `id`, `deps`, artefactos por target, prompts, default y requisitos. Módulos: `core` (vanta-cli), `mcp` (bloques por cliente), `server` (HTTP), `proxy` (vanta-proxy), `desktop` (Tauri), `embeddings` (ORT + modelo), `providers` (config de proveedores).
3. **Perfiles**: `minimal` (core), `default` (core + mcp + embeddings con modelo default), `full` (todos los módulos con binario publicado), `custom` (selección interactiva). Los perfiles son la primera pregunta; el detalle va después.
4. **Resolución por módulo x plataforma**: tabla target→asset reutilizando la release matrix (5 targets; ampliar a aarch64-windows cuando exista artefacto). Descarga desde GitHub Releases usando `browser_download_url` + `digest` de la API; verificación sha256 obligatoria (fin del warn-and-continue); siguiente paso: firma minisign/cosign con clave pública embebida.
5. **Config**: `~/.vanta/config.toml` versionado (`config_version`), XDG-aware en Unix (`~/.config/vanta` + `~/.local/share/vanta` con fallback `~/.vanta`), `%APPDATA%\VantaDB\config.toml` en Windows; override único `VANTA_HOME`; escritura atómica; secretos nunca persistidos.
6. **Ciclo de vida**: `setup` re-ejecutable e idempotente; `--enable/--disable <módulo>`; `--dry-run`; `--status --json`; `setup uninstall [--purge]`; backups de binarios; bloques con marcadores en configs de terceros.
7. **UX de prompts** (clig.dev): solo TTY; Enter=default visible; una decisión por prompt; resumen + confirmación antes de aplicar; `--non-interactive` usa el perfil default; `NO_COLOR`; progreso solo en TTY; errores con siguiente comando sugerido.
8. **CI/enterprise**: `vanta-cli setup --non-interactive --profile full --modules a,b,c`; todo override por flags/env; `--offline --bundle <dir>` (binarios + modelos + plantillas) para air-gapped; soporte `HTTP(S)_PROXY`/`NO_PROXY`.

### 5.2 Roadmap por fases

| Fase | Alcance | Estimación | Criterio de salida |
|---|---|---|---|
| F1 MVP | `vanta-cli setup` absorbe el wizard actual + selector de módulos (`core`, `mcp`, `server`, `proxy`, `embeddings`), perfiles minimal/default/full, `--non-interactive`, checksum obligatorio, re-ejecución idempotente; scripts encadenan al binario | 1-2 días | Misma funcionalidad que hoy en Win/mac/Linux sin `pwsh`; smoke test en VM limpia |
| F2 Config y ciclo de vida | `config.toml` versionado, `--enable/--disable/--dry-run/--status --json`, `setup uninstall`, marcadores MCP/PATH, backups, docs | 2-4 días | Re-ejecutar no duplica nada; disable limpia sin romper otras configs |
| F3 Confianza y desktop | Módulo `desktop` (NSIS per-user default, WebView2 `downloadBootstrapper`, MSI opcional), updater Tauri firmado, firma minisign/cosign del CLI, bundle offline de modelos, i18n ES/EN | 1-2 semanas | Artefactos firmados; update E2E con firma; bundle air-gapped probado |
| F4 Canales | winget/scoop/brew (tap propio), docs enterprise (proxy/MDM), aarch64-windows, mirrors de modelos si hacen falta | continuo | Instalación desde package manager sin wizard propio |

## 6. Fuentes

### Verificadas por fetch directo (2026-10-01)

- rustup — profiles: https://rust-lang.github.io/rustup/concepts/profiles.html
- rustup — otros métodos de instalación: https://rust-lang.github.io/rustup/installation/other.html
- rustup — instalación: https://rust-lang.github.io/rustup/installation/index.html
- rustup — seguridad: https://rust-lang.github.io/rustup/security.html
- cargo-dist — installers: https://raw.githubusercontent.com/axodotdev/cargo-dist/main/book/src/installers/index.md
- cargo-dist — shell installer: https://raw.githubusercontent.com/axodotdev/cargo-dist/main/book/src/installers/shell.md
- VS Code — instalación Windows: https://code.visualstudio.com/docs/setup/windows
- Docker Desktop — instalación Windows: https://docs.docker.com/desktop/setup/install/windows-install/
- uv — instalación: https://docs.astral.sh/uv/getting-started/installation/
- Tauri — instalador Windows (NSIS/WiX): https://v2.tauri.app/distribute/windows-installer/
- Tauri — updater: https://v2.tauri.app/plugin/updater/
- Tauri — firma macOS: https://v2.tauri.app/distribute/sign/macos/
- CLI Guidelines: https://clig.dev/
- Homebrew — instalación: https://docs.brew.sh/Installation
- Homebrew — Brewfile: https://docs.brew.sh/Brew-Bundle-and-Brewfile
- miniforge: https://raw.githubusercontent.com/conda-forge/miniforge/main/README.md
- XDG Base Directory Spec: https://specifications.freedesktop.org/basedir-spec/latest/
- Hugging Face — variables de entorno: https://huggingface.co/docs/huggingface_hub/package_reference/environment_variables
- ONNX Runtime — instalación: https://onnxruntime.ai/docs/install/
- Inno Setup — componentes: https://jrsoftware.org/ishelp/topic_componentssection.htm
- Inno Setup — parámetros CLI: https://jrsoftware.org/ishelp/topic_setupcmdline.htm
- MSI — opciones de línea de comandos: https://learn.microsoft.com/en-us/windows/win32/msi/command-line-options
- MSI — ADDLOCAL: https://learn.microsoft.com/en-us/windows/win32/msi/addlocal
- Python en Windows (install manager): https://docs.python.org/3/using/windows.html
- Visual Studio — parámetros CLI: https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio
- cargo vendor: https://doc.rust-lang.org/cargo/commands/cargo-vendor.html
- GitHub Releases API: https://docs.github.com/en/rest/releases/releases
- winget install: https://learn.microsoft.com/en-us/windows/package-manager/winget/install
- Ollama — README: https://raw.githubusercontent.com/ollama/ollama/main/README.md
- Ollama — install.sh: https://raw.githubusercontent.com/ollama/ollama/main/scripts/install.sh

### No verificadas (pendientes)

- `HF_ENDPOINT` como espejo de Hugging Face: no aparece en la página oficial de variables de entorno; tratar como mecanismo no garantizado. `[cita NO VERIFICADA]`
- Switches específicos del instalador de VS Code (`/VERYSILENT`, `/MERGETASKS`): la FAQ los documenta pero el contenido no fue accesible en el fetch; el mecanismo subyacente (Inno Setup) sí está verificado. `[cita NO VERIFICADA]`
- `docs.ollama.com/linux` (instalación manual): no verificado por fetch. `[cita NO VERIFICADA]`
