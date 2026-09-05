# DEVOPS-10 — Agregar Windows code signing al release pipeline

## Tipo
CI/CD / DevOps → vanta-lead (yo mismo)

## Discovery
- `release-binaries-63.yml` existe con matrix para 5 targets
- Windows target: `x86_64-pc-windows-msvc` en `windows-latest`
- **No hay signing step** — solo comprime en .zip + SHA256
- Linux/macOS tienen tar.gz, Windows tiene .zip (7z)
- Tampoco hay authenticode signing ni EV certificate config

## Contexto
Para releases públicos de Windows, el code signing (Authenticode) es crítico para:
- Evitar warnings "Unknown publisher" de SmartScreen/Windows Defender
- Facilitar la instalación en entornos enterprise
- Aumentar confianza del usuario

## Opciones
1. **Azure Key Vault + Azure Trusted Signing** (recomendado, más barato que EV cert físico)
   - Servicio de Microsoft: ~$10/mes
   - Se integra vía `azure-trusted-signing-action` en GitHub
2. **EV Certificate físico** (más caro, $200-400/año)
   - Almacenado como GitHub Secret (PFX + password)
   - Firmado con `signtool` via Windows SDK
3. **Auto-generated self-signed** (gratis, pero menos confiable)
   - No recomendado para producción pública

## Pasos atómicos
1. Decidir estrategia: Azure Trusted Signing por ser la más moderna y barata
2. Agregar signing step post-build en Windows:
   ```yaml
   - name: Sign Windows binaries (Azure Trusted Signing)
     if: runner.os == 'Windows'
     uses: azure/trusted-signing-action@v0
     with:
       endpoint: ${{ secrets.AZURE_TRUSTED_SIGNING_ENDPOINT }}
       trusted-signing-account-name: ${{ secrets.AZURE_SIGNING_ACCOUNT }}
       certificate-profile-name: ${{ secrets.AZURE_CERT_PROFILE }}
       files: |
         target/${{ matrix.target }}/release/vanta-cli.exe
         target/${{ matrix.target }}/release/vantadb-server.exe
   ```
3. Alternativa más simple si no hay Azure: firmar con self-signed + winget-publish
4. Verificar en dry-run que no rompa el pipeline

## Verification
- Workflow YAML válido
- Azure Trusted Signing configurado y accesible
- Binarios firmados verificables con `signtool verify /pa`
- SHA256 checksums post-firma siguen siendo correctos

## Nota
Requiere suscripción Azure + configuración de Azure Trusted Signing. Si no hay acceso Azure ahora, dejar preparado el paso YAML con `if: false` y documentar los secrets necesarios.

## Estado
🔵 DEFERIDO — 2026-07-26 (ponytail). SHA256 + .zip dan integridad básica. Agregar Azure Trusted Signing cuando release público lo requiera. Step YAML preparado arriba, secrets documentados: `AZURE_TRUSTED_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE`.
