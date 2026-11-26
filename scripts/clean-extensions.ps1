# VS Code Extension Cleanup Script
# Generado: 2026-07-09
#
# Desinstala extensiones redundantes basado en tu sesión de optimización.
# CORRELO DESDE PowerShell ISE O TERMINAL, NO desde VS Code.
#
# Ejecutar: .\scripts\clean-extensions.ps1

$ErrorActionPreference = "Continue"

$toRemove = @(

    # ── JAVA (7) — Usás PHP, no Java ──
    "redhat.java"
    "vscjava.vscode-java-debug"
    "vscjava.vscode-java-test"
    "vscjava.vscode-java-dependency"
    "vscjava.vscode-java-pack"
    "vscjava.vscode-maven"
    "vscjava.vscode-gradle"

    # ── PHP DUPLICADOS (quedan: intelephense + xdebug) ──
    "devsense.composer-php-vscode"
    "devsense.intelli-php-vscode"
    "devsense.phptools-vscode"
    "devsense.profiler-php-vscode"
    "etubaro.php-code-sniffer"
    "mansoorkhan96.php-cs-fixer"
    "rifi2k.format-html-in-php"

    # ── VBA/VBSCRIPT (no usás) ──
    "darfka.vbscript"
    "davikawasaki.vbgenerators"
    "serkonda7.vscode-vba"

    # ── COLABORACIÓN REMOTA ──
    "github.codespaces"
    "ms-vsliveshare.vsliveshare"

    # ── SFTP ──
    "natizyskunk.sftp"

    # ── CSS TOOLS NO USADAS ──
    "blanu.vscode-styled-jsx"
    "stivo.tailwind-fold"

    # ── MARKDOWN REDUNDANTE (quedan: markdownlint + mermaid + all-in-one) ──
    "shd101wyy.markdown-preview-enhanced"
    "bierner.markdown-preview-github-styles"

    # ── RUST REDUNDANTE (rust-analyzer ya lo cubre todo) ──
    "1yib.rust-bundle"
    "dustypomerleau.rust-syntax"

    # ── SNIPPETS JS (queda: es7-react-js-snippets) ──
    "burkeholland.simple-react-snippets"
    "skyran.js-jsx-snippets"
    "pulkitgangwar.nextjs-snippets"
    "willluke.nextjs"
    "shibu.nextjs-js-ts-code-snippets"

    # ── VS CODE NATIVO (ya lo hace el editor sin extensiones) ──
    "formulahendry.auto-close-tag"    # usar "editor.autoClosingTags"
    "formulahendry.auto-rename-tag"   # usar "editor.linkedEditing"
    "bradgashler.htmltagwrap"         # Emmet nativo

    # ── COLOR TOOLS DUPLICADAS (queda: color-highlight) ──
    "anseki.vscode-color"
    "lihui.vs-color-picker"
    "bierner.color-info"

    # ── FORMATEADORES REDUNDANTES ──
    "nikolaosgeorgiou.html-fmt-vscode" # Prettier
    "zignd.html-css-class-completion"  # Tailwind CSS IntelliSense
    "ecmel.vscode-html-css"            # Tailwind CSS IntelliSense
    "dotjoshjohnson.xml"               # solapa con redhat.vscode-xml

    # ── XML/YAML (si no editas seguido) ──
    "redhat.vscode-xml"
    "redhat.vscode-yaml"

    # ── HERRAMIENTAS NICHE NO USADAS ──
    "hookyqr.minify"                   # build tools (esbuild/vite)
    "inu1255.easy-snippet"             # snippets nativos VS Code
    "aykutsarac.jsoncrack-vscode"      # visualización JSON
    "kalimahapps.tailwind-config-viewer"
    "bourhaouta.tailwindshades"
    "idleberg.icon-fonts"
    "sldobri.bunker"
    "moyu.snapcode"

    # ── PRISMA: quedate con la stable ──
    "prisma.prisma-insider"

    # ── PYTHON: env manager redundante con el selector nativo ──
    "ms-python.vscode-python-envs"
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   VS Code Extension Cleanup" -ForegroundColor Cyan
Write-Host "   $($toRemove.Count) extensiones a desinstalar" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

foreach ($ext in $toRemove) {
    Write-Host "Desinstalando: $ext" -NoNewline
    $output = code --uninstall-extension "$ext" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host " ✅" -ForegroundColor Green
    } else {
        Write-Host " ❌ $output" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Limpieza completada" -ForegroundColor Cyan
Write-Host "   Extensiones restantes:" -ForegroundColor Cyan
code --list-extensions
