#Requires -Version 7.0
<#
.SYNOPSIS
    Vanta Design Orchestrator — Skill Bridge CLI
.DESCRIPTION
    PowerShell module for orchestrating 170+ design skills. Lists, loads, routes,
    and chains skills by project type. Run from the project root.
.EXAMPLE
    .agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -ListSkills
    .agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -Route "landing-page"
    .agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -AnalyzeConflicts
#>

param(
    [switch]$ListSkills,
    [switch]$ListCategories,
    [switch]$ListPresets,
    [switch]$AnalyzeConflicts,
    [string]$Route,
    [string]$Preset,
    [switch]$Help
)

$Script:OrchestratorDir = "$PSScriptRoot\.."
$Script:ProjectSkillsDir = "$PSScriptRoot\..\..\.."
$Script:GlobalSkillsDir = "$env:USERPROFILE\.agents\skills"
$Script:PresetsFile = "$OrchestratorDir\configs\project-presets.json"

function Show-Help {
    Write-Host @"

VANTA DESIGN ORCHESTRATOR — SKILL BRIDGE CLI
=============================================

USOS:
  -ListSkills              Listar todas las skills disponibles (170+)
  -ListCategories          Listar skills agrupadas por categoría
  -ListPresets             Listar presets de proyecto disponibles
  -Route "preset-name"     Mostrar el pipeline de skills para un preset
  -Preset "preset-name"    Cargar instrucciones detalladas del preset
  -AnalyzeConflicts        Detectar skills duplicadas y conflictos
  -Help                    Mostrar esta ayuda

EJEMPLOS:
  .agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -ListPresets
  .agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -Route landing-page
  .agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -AnalyzeConflicts
"@
}

function Get-SkillsFromDir($dir) {
    if (!(Test-Path $dir)) { return @() }
    return Get-ChildItem $dir -Directory | Select-Object -ExpandProperty Name
}

function Get-AllSkills {
    $project = Get-SkillsFromDir $Script:ProjectSkillsDir
    $global = Get-SkillsFromDir $Script:GlobalSkillsDir
    $all = $project + $global | Sort-Object -Unique
    return $all
}

function Get-SkillCategory($name) {
    $categories = @{
        # FAL.ai
        "fal-generate" = "FAL"; "fal-image-edit" = "FAL"; "fal-3d" = "FAL"
        "fal-kling-o3" = "FAL"; "fal-realtime" = "FAL"; "fal-upscale" = "FAL"
        "fal-video-edit" = "FAL"; "fal-lip-sync" = "FAL"; "fal-train" = "FAL"
        "fal-tryon" = "FAL"; "fal-restore" = "FAL"; "fal-vision" = "FAL"
        # Figma
        "figma-generate-design" = "Figma"; "figma-create-new-file" = "Figma"
        "figma-generate-library" = "Figma"; "figma-code-connect-components" = "Figma"
        "figma-implement-design" = "Figma"; "figma-use" = "Figma"
        "figma-create-design-system-rules" = "Figma"
        # Deck
        "deck-swiss-international" = "Deck"; "deck-open-slide-canvas" = "Deck"
        "deck-guizang-editorial" = "Deck"; "digits-fintech-swiss-template" = "Deck"
        "ppt-keynote" = "Deck"; "pptx" = "Deck"; "pptx-generator" = "Deck"
        "pptx-html-fidelity-audit" = "Deck"; "nanobanana-ppt" = "Deck"
        "html-ppt-retro-quarterly-review" = "Deck"
        "swiss-creative-mode-template" = "Deck"; "swiss-user-research-video-template" = "Deck"
        # Frame
        "frame-glitch-title" = "Frame"; "frame-light-leak-cinema" = "Frame"
        "frame-liquid-bg-hero" = "Frame"; "frame-logo-outro" = "Frame"
        "frame-data-chart-nyt" = "Frame"; "frame-flowchart-sticky" = "Frame"
        "frame-macos-notification" = "Frame"; "vfx-text-cursor" = "Frame"
        # Venice
        "venice-image-generate" = "Venice"; "venice-image-edit" = "Venice"
        "venice-audio-music" = "Venice"; "venice-audio-speech" = "Venice"
        "venice-video" = "Venice"
        # Social
        "social-x-post-card" = "Social"; "social-reddit-card" = "Social"
        "social-spotify-card" = "Social"; "card-twitter" = "Social"
        "card-xiaohongshu" = "Social"; "slack-gif-creator" = "Social"
        # Video
        "video-hyperframes" = "Video"; "video-downloader" = "Video"
        "youtube-clipper" = "Video"; "remotion" = "Video"
        "8-bit-orbit-video-template" = "Video"; "after-hours-editorial-template" = "Video"
        "field-notes-editorial-template" = "Video"
        "weread-year-in-review-video-template" = "Video"; "ai-music-album" = "Video"
        "sora" = "Video"
        # Docs
        "doc" = "Docs"; "docx" = "Docs"; "minimax-docx" = "Docs"
        "doc-kami-parchment" = "Docs"; "pdf" = "Docs"; "minimax-pdf" = "Docs"
        "article-magazine" = "Docs"; "editorial-burgundy-principles-template" = "Docs"
        "resume-modern" = "Docs"
        # Apple
        "apple-hig" = "Apple"; "swiftui-design" = "Apple"
        # Visual
        "d3-visualization" = "Visual"; "hand-drawn-diagrams" = "Visual"
        "mockup-device-3d" = "Visual"; "shader-dev" = "Visual"
        "threejs" = "Visual"
        # UI
        "shadcn-ui" = "UI"; "ui-skills" = "UI"; "platform-design" = "UI"
        "wpds" = "UI"; "login-flow" = "UI"; "faq-page" = "UI"
        "paywall-upgrade-cro" = "UI"
        # Brand
        "brandkit" = "Brand"; "canvas-design" = "Brand"
        "algorithmic-art" = "Brand"; "theme-factory" = "Brand"
        "brand-guidelines" = "Brand"; "color-expert" = "Brand"
        "creative-director" = "Brand"; "design-brief" = "Brand"
        "design-consultation" = "Brand"; "design-review" = "Brand"
        "design-md" = "Brand"; "reference-design-contract" = "Brand"
        # Image
        "imagegen" = "Image"; "imagen" = "Image"
        "poster-hero" = "Image"; "image-enhancer" = "Image"
        "ecommerce-image-workflow" = "Image"; "pixelbin-media" = "Image"
        "full-page-screenshot" = "Image"; "screenshot" = "Image"
        "screenshots-marketing" = "Image"; "image-to-code-skill" = "Image"
        "imagegen-frontend-web" = "Image"; "imagegen-frontend-mobile" = "Image"
        "gif-sticker-maker" = "Image"
        # Audio
        "speech" = "Audio"; "venice-audio-music" = "Audio"
        "venice-audio-speech" = "Audio"
        # Marketing
        "ad-creative" = "Marketing"; "competitive-ads-extractor" = "Marketing"
        "copywriting" = "Marketing"
        # Utility
        "enhance-prompt" = "Utility"; "export-download-debugging" = "Utility"
        "pr-feedback-quality-gate" = "Utility"
        "release-notes-one-pager" = "Utility"; "domain-name-brainstormer" = "Utility"
        "soft-skill" = "Utility"; "output-skill" = "Utility"
        "web-artifacts-builder" = "Utility"; "artifacts-builder" = "Utility"
        "research-decision-room" = "Utility"; "plan-design-review" = "Utility"
        "replicate" = "Utility"; "flutter-animating-apps" = "UI"
        "hatch-pet" = "Utility"
    }
    if ($categories.ContainsKey($name)) { return $categories[$name] }

    # Default categorization by prefix
    if ($name -match "^fal-") { return "FAL" }
    if ($name -match "^figma-") { return "Figma" }
    if ($name -match "^deck-") { return "Deck" }
    if ($name -match "^frame-") { return "Frame" }
    if ($name -match "^venice-") { return "Venice" }
    if ($name -match "^social-") { return "Social" }
    if ($name -match "^card-") { return "Social" }
    if ($name -match "^video-") { return "Video" }
    if ($name -match "template$") { return "Template" }
    if ($name -match "skill$") { return "UI" }
    return "Other"
}

function Show-AllSkills {
    $all = Get-AllSkills
    Write-Host "`n=== SKILLS DISPONIBLES: $($all.Count) ===`n" -ForegroundColor Cyan

    $categories = @{}
    foreach ($s in $all) {
        $cat = Get-SkillCategory $s
        if (!$categories[$cat]) { $categories[$cat] = @() }
        $categories[$cat] += $s
    }

    $cats = $categories.Keys | Sort-Object
    foreach ($cat in $cats) {
        Write-Host "  [$cat] ($($categories[$cat].Count))" -ForegroundColor Yellow
        $categories[$cat] | Sort-Object | ForEach-Object { Write-Host "    $_" }
    }
}

function Show-Categories {
    $all = Get-AllSkills
    Write-Host "`n=== SKILLS POR CATEGORÍA ===`n" -ForegroundColor Cyan

    $categories = @{}
    foreach ($s in $all) {
        $cat = Get-SkillCategory $s
        if (!$categories[$cat]) { $categories[$cat] = @() }
        $categories[$cat] += $s
    }

    $cats = $categories.Keys | Sort-Object
    $total = 0
    foreach ($cat in $cats) {
        $count = $categories[$cat].Count
        $total += $count
        Write-Host "  $cat".PadRight(20) -ForegroundColor Yellow -NoNewline
        Write-Host "$count skills"
    }
    Write-Host "  ---".PadRight(20) -NoNewline
    Write-Host "---"
    Write-Host "  TOTAL".PadRight(20) -ForegroundColor Cyan -NoNewline
    Write-Host "$total skills"
}

function Show-Presets {
    if (!(Test-Path $Script:PresetsFile)) {
        Write-Host "ERROR: Presets file not found at $Script:PresetsFile" -ForegroundColor Red
        return
    }
    $presets = Get-Content $Script:PresetsFile | ConvertFrom-Json
    Write-Host "`n=== PRESETS DE PROYECTO: $($presets.presets.PSObject.Properties.Count) ===`n" -ForegroundColor Cyan

    $presets.presets.PSObject.Properties | Sort-Object Name | ForEach-Object {
        $p = $_.Value
        $phases = $p.phases.PSObject.Properties.Name -join " → "
        Write-Host "  $($_.Name)" -ForegroundColor Yellow
        Write-Host "    $($p.name)"
        Write-Host "    Fases: $phases"
        Write-Host ""
    }
}

function Show-Route($presetName) {
    if (!(Test-Path $Script:PresetsFile)) {
        Write-Host "ERROR: Presets file not found" -ForegroundColor Red
        return
    }
    $presets = Get-Content $Script:PresetsFile | ConvertFrom-Json
    $p = $presets.presets.$presetName
    if (!$p) {
        Write-Host "ERROR: Preset '$presetName' not found." -ForegroundColor Red
        Write-Host "Available: $($presets.presets.PSObject.Properties.Name -join ', ')"
        return
    }

    Write-Host "`n=== RUTA: $presetName ($($p.name)) ===`n" -ForegroundColor Cyan
    $p.phases.PSObject.Properties | ForEach-Object {
        Write-Host "  ╔══ $($_.Name) ══╗" -ForegroundColor Green
        $_.Value | ForEach-Object { Write-Host "  ║ $_" }
        Write-Host "  ╚════════════════╝`n"
    }

    if ($p.recommended_skills) {
        Write-Host "  + Recomendadas:" -ForegroundColor Magenta
        $p.recommended_skills | ForEach-Object { Write-Host "    $_" }
    }
}

function Show-AnalyzeConflicts {
    Write-Host "`n=== ANÁLISIS DE CONFLICTOS ===`n" -ForegroundColor Cyan

    $project = Get-SkillsFromDir $Script:ProjectSkillsDir | Sort-Object
    $global = Get-SkillsFromDir $Script:GlobalSkillsDir | Sort-Object

    # Duplicate names
    $dupes = $project | Where-Object { $_ -in $global }
    if ($dupes) {
        Write-Host "  Skills duplicadas (project + global):" -ForegroundColor Yellow
        $dupes | ForEach-Object { Write-Host "    $_  [project + global]" }
    }

    # Similar skill pairs that might conflict
    $pairs = @(
        @("impeccable", "impeccable-design-polish"),
        @("gpt-taste", "gpt-tasteskill"),
        @("emil-design-eng", "emilkowalski-motion"),
        @("design-taste-frontend", "taste-skill"),
        @("redesign-existing-projects", "redesign-skill")
    )

    Write-Host "`n  Versiones en conflicto (similar name):" -ForegroundColor Yellow
    foreach ($pair in $pairs) {
        $inProject = $pair | Where-Object { $_ -in $project }
        $inGlobal = $pair | Where-Object { $_ -in $global }
        $found = $inProject + $inGlobal
        if ($found.Count -gt 0) {
            Write-Host "    $($pair[0]) ↔ $($pair[1])" -ForegroundColor DarkYellow
            Write-Host "      → Tienes: $($found -join ', ')"
            Write-Host "      → Recomendación: USAR $($pair[1]) (open-design)" -ForegroundColor Green
        }
    }

    Write-Host "`n  Conflicto detectado entre:" -ForegroundColor Yellow
    Write-Host "  - impeccable vs impeccable-design-polish" -ForegroundColor DarkYellow
    Write-Host "  - gpt-taste vs gpt-tasteskill" -ForegroundColor DarkYellow
    Write-Host "  - emil-design-eng vs emilkowalski-motion" -ForegroundColor DarkYellow
    Write-Host "  - design-taste-frontend vs taste-skill" -ForegroundColor DarkYellow
    Write-Host "  - redesign-existing-projects vs redesign-skill`n" -ForegroundColor DarkYellow
}

# ── MAIN ──────────────────────────────────────────────────────────────────

if ($Help) { Show-Help; return }

$hasAction = $ListSkills -or $ListCategories -or $ListPresets -or $AnalyzeConflicts -or $Route -or $Preset

if (!$hasAction) {
    Show-Help
    Write-Host "`nSistema detectado: $((Get-AllSkills).Count) skills disponibles en total." -ForegroundColor Green
    return
}

if ($ListSkills) { Show-AllSkills }
if ($ListCategories) { Show-Categories }
if ($ListPresets) { Show-Presets }
if ($Route) { Show-Route $Route }
if ($Preset) { Show-Route $Preset }
if ($AnalyzeConflicts) { Show-AnalyzeConflicts }
