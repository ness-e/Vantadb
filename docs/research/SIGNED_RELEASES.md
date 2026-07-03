# Signed Releases para Windows — Investigación

> **Ticket:** DEVOPS-10
> **Fecha:** 2026-07-03
> **Estado:** Investigación completa

---

## 1. Problema: Windows SmartScreen

### ¿Qué lo causa?

Cuando un usuario descarga un binario `.exe` o `.msi` de Windows que **no está firmado digitalmente**, Windows muestra una advertencia de SmartScreen:

> "Windows protected your PC — Microsoft Defender SmartScreen prevented an unrecognized app from starting."

Esto ocurre porque el binario carece de una **firma Authenticode** que certifique su origen. SmartScreen aplica heurísticas de reputación:

1. **Sin firma** → peor reputación, advertencia inmediata
2. **Firmado con cert estándar** → SmartScreen muestra advertencia hasta que el binario acumula suficientes descargas/ejecuciones para ganar reputación
3. **Firmado con EV (Extended Validation) cert** → trust inmediato (sin advertencia) porque Microsoft已验证 la identidad legal del editor

### Impacto en percepción del usuario

- **Tasa de conversión**: Las advertencias de SmartScreen reducen la instalación entre un 30-60% en usuarios no técnicos
- **Percepción de seguridad**: Un binario sin firmar se percibe como amateur o potencialmente malicioso
- **Adopción enterprise**: Políticas de grupo bloquean ejecución de binarios sin firma — imposible adoptar en entornos corporativos
- **Soporte**: Cada advertencia genera dudas y tickets de soporte

### Estado actual en VantaDB

| Aspecto | Estado |
|---|---|
| Builds Windows en CI | ❌ — `release.yml` solo compila Linux + macOS. Windows no está en el matrix |
| Python wheels (Windows) | ✅ — `python_wheels.yml` buildea en `windows-latest` |
| Firma Authenticode | ❌ — No existe |
| GitHub Attestations | ✅ — Ya implementado en pipeline Python (SLSA L2) |
| GPG checksums | ❌ — Mencionado en GO_TO_MARKET.md pero no implementado |
| Scripts de signing | ❌ — No existen |

---

## 2. Solution Options

### Option A: Authenticode Code Signing

**Costo**: $200-500/año (DigiCert, Sectigo)

**Proceso**:
1. Comprar certificado de código (Code Signing) de una CA confiable
2. Proteger la clave privada en un HSM o Azure Key Vault
3. En CI, usar `signtool.exe` para firmar los binarios después del build
4. Publicar binarios firmados como assets del release

**Pros**:
- Estándar de la industria — elimina SmartScreen tras ganar reputación
- Compatible con todas las versiones de Windows
- EV cert da trust inmediato sin período de reputation-building
- Requerido para enterprise adoption

**Contras**:
- Costo recurrente ($200-500/yr para cert estándar, ~$300-500/yr para EV)
- Manejo seguro de clave privada (complejidad operativa)
- El cert estándar **no** elimina SmartScreen inmediatamente — necesita acumular descargas
- Renovación anual
- Solo funciona para Windows (no cubre macOS/Linux)

### Option B: Open Source Signing (SignPath Foundation)

**Costo**: Gratuito para open source

**Proceso**:
1. Aplicar al [SignPath Foundation](https://signpath.org/foundation/) (gratis para proyectos OSS)
2. Integrar `SignPath/actions` en GitHub Actions
3. El signing se hace remoto — la clave nunca sale del HSM de SignPath

**Pros**:
- Gratuito para proyectos open source
- Integración nativa con GitHub Actions
- No requiere manejo local de claves privadas
- Firma Authenticode real (Windows confía en la CA raíz de SignPath)

**Contras**:
- Proceso de aprobación manual (puede tomar semanas)
- Misma limitación que Option A: cert estándar, no EV — SmartScreen sigue mostrando warning inicialmente
- Dependencia de un tercero no tan conocido
- Si el proyecto no es aceptado, no hay alternativa gratuita

### Option C: macOS/Linux Signing

**macOS** (`codesign`):
- Requiere Apple Developer Account ($99/año)
- Usar `codesign` en CI para firmar el binario
- Notarización con Apple Notary Service para evitar Gatekeeper warnings
- **Pros**: Estándar en macOS, elimina "unidentified developer"
- **Contras**: Costo recurrente, build debe correr en macOS (ya es el caso)

**Linux**:
- **No existe un estándar equivalente** a Authenticode o `codesign`
- La alternativa es **GPG + checksums**: firmar el checksum SHA256 con GPG
- Distribuciones empaquetan los binarios — la confianza viene del maintainer, no de una firma binaria
- **Snap** y **Flatpak** tienen sus propios mecanismos (Snap Store, Flathub)
- **Pros**: Sin costo, GPG es estándar en Linux
- **Contras**: No es una firma del binario mismo, el usuario debe verificar manualmente

### Option D: GitHub Attestations (SLSA)

**Costo**: Gratuito

**Proceso**:
```yaml
- uses: actions/attest-build-provenance@v4
  with:
    subject-path: 'release/vantadb-*.tar.gz'
```

**Pros**:
- ✅ **Ya implementado** en el pipeline Python de VantaDB (SLSA Level 2)
- Gratuito, zero configuración adicional
- Prueba criptográfica de que el binario fue construido por GitHub Actions desde un commit específico
- `gh attestation verify` permite a cualquier usuario verificar
- Compatible con SLSA (Supply-chain Levels for Software Artifacts)

**Contras**:
- ❌ **No elimina SmartScreen** — GitHub Attestations no es una firma Authenticode
- Requiere `gh` CLI para verificar (no es transparente para el usuario final)
- Es una solución de **proveniencia** no de **trust de plataforma**

---

## 3. Recomendación

### Estrategia por fases

| Fase | Qué | Costo | Timeline | Impacto SmartScreen |
|---|---|---|---|---|
| **Fase 1 (ahora)** | GitHub Attestations para todos los binarios + GPG checksums | $0 | 1-2 días | ❌ No remueve |
| **Fase 2 (corto plazo)** | Agregar Windows al matrix de `release.yml` + Attestations | $0 | 1-2 días | ❌ No remueve |
| **Fase 3 (cuando haya presupuesto)** | EV Code Signing cert (DigiCert o Sectigo) | ~$300-500/yr | 1 semana setup | ✅ Trust inmediato |
| **Fase 4 (futuro)** | macOS Developer Account + notarization | $99/yr | 2-3 días setup | ✅ Gatekeeper trust |

### Prioridad inmediata: Fase 1 + 2

**Razones**:
1. **Costo cero** y ya hay infraestructura probada en el pipeline Python
2. La falta de Windows builds es un problema mayor que la falta de firma
3. Las GitHub Attestations mejoran la postura de seguridad supply-chain hoy
4. EV cert tiene sentido solo cuando haya suficiente tracción para justificar el gasto recurrente

---

## 4. Plan de Implementación

### Fase 1: GitHub Attestations + GPG Checksums

#### Paso 1: Agregar permisos OIDC al workflow

En `release.yml`, agregar al nivel del job:

```yaml
jobs:
  build:
    permissions:
      id-token: write
      contents: read
      attestations: write
```

#### Paso 2: Generar checksums SHA256

Después del packaging:

```yaml
- name: Generate checksums
  run: |
    cd release
    sha256sum vantadb-*.tar.gz > vantadb-SHA256SUMS.txt
```

#### Paso 3: Firmar checksums con GPG

```yaml
- name: Import GPG key
  uses: crazy-max/ghaction-import-gpg@v6
  with:
    gpg_private_key: ${{ secrets.GPG_PRIVATE_KEY }}
    passphrase: ${{ secrets.GPG_PASSPHRASE }}

- name: Sign checksums
  run: |
    cd release
    gpg --detach-sign --armor vantadb-SHA256SUMS.txt
```

#### Paso 4: Attest build provenance

```yaml
- name: Attest build provenance
  uses: actions/attest-build-provenance@v4
  with:
    subject-path: 'release/vantadb-*.tar.gz'
```

### Fase 2: Agregar Windows al Release Matrix

```yaml
matrix:
  include:
    - target: x86_64-unknown-linux-gnu
      os: ubuntu-latest
    - target: x86_64-apple-darwin
      os: macos-latest
    - target: aarch64-apple-darwin
      os: macos-latest
    - target: x86_64-pc-windows-msvc  # ← NUEVO
      os: windows-latest               # ← NUEVO
```

Ajustar packaging para Windows:

```yaml
- name: Package binaries (Windows)
  if: matrix.os == 'windows-latest'
  run: |
    mkdir -p release
    Copy-Item "target/${{ matrix.target }}/release/vanta-cli.exe" -Destination release/
    Copy-Item "target/${{ matrix.target }}/release/vantadb-server.exe" -Destination release/
    Compress-Archive -Path release/vanta-cli.exe, release/vantadb-server.exe -DestinationPath release/vantadb-${{ matrix.target }}.zip

- name: Package binaries (Unix)
  if: matrix.os != 'windows-latest'
  run: |
    mkdir -p release
    cp target/${{ matrix.target }}/release/vanta-cli release/
    cp target/${{ matrix.target }}/release/vantadb-server release/
    cd release
    tar czf vantadb-${{ matrix.target }}.tar.gz vanta-cli vantadb-server
```

### Verificación por usuarios

```bash
# Verificar proveniencia (requiere gh CLI)
gh attestation verify vantadb-x86_64-pc-windows-msvc.zip \
  --repo ness-e/Vantadb

# Verificar checksum + firma GPG
gpg --verify vantadb-SHA256SUMS.txt.asc vantadb-SHA256SUMS.txt
sha256sum -c vantadb-SHA256SUMS.txt
```

### Pipeline final (diagrama)

```
push v* tag
     │
     ▼
[Build Matrix]
Linux x86_64  │  macOS x86_64  │  macOS arm64  │  Windows x86_64
     │
     ▼
[Package per platform]
.tar.gz (Unix)  │  .zip (Windows)
     │
     ├── [Generate SHA256SUMS.txt]
     ├── [GPG-sign SHA256SUMS.txt]
     ├── [attest-build-provenance@v4] ─── SLSA Level 2
     │
     ▼
[Upload release assets]
     │
     ▼
[gh attestation verify] ← downstream verification
```
