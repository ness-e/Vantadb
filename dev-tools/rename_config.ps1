$files = @(
    "Dockerfile",
    "docker-compose.yml",
    "start.sh",
    ".github\workflows\rust_ci.yml",
    ".github\workflows\release.yml",
    ".gitignore"
)

foreach ($f in $files) {
    if (Test-Path $f) {
        $content = Get-Content $f -Raw
        $orig = $content
        $content = $content.Replace("connectomedb-server", "vanta-server")
        $content = $content.Replace("connectomedb_data", "vantadb_data")
        $content = $content.Replace("NexusDB", "VantaDB")
        $content = $content.Replace("nexusdb", "vantadb")
        $content = $content.Replace("ConnectomeDB", "VantaDB")
        $content = $content.Replace("connectomedb", "vantadb")
        $content = $content.Replace("CONNECTOMEDB_", "VANTADB_")
        
        if ($content -cne $orig) {
            Set-Content $f -Value $content -NoNewline
            Write-Host "Updated $f"
        }
    }
}
