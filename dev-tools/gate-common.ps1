# gate-common.ps1 — Definición canónica compartida del core gate (P2, consolidación 2026-08-25)
#
# Fuente ÚNICA de: features del core check + jobs adaptativos.
# Consumidores: verify.ps1, verify_changed.ps1, audit-all.ps1 (dot-source este archivo).
# Cualquier cambio aquí afecta los tres gates por igual — eso es intencional:
# el gate debe ser determinista entre herramientas (mismos features = mismos
# resultados de compilación/lint). Antes cada script tenía su variante y un
# cambio podía pasar en uno y fallar en otro.
#
# Nota RUST_MIN_STACK queda por-script (verify=32MB, verify_changed=16MB):
# es límite de memoria por hilo, no semántica del gate.

function Get-CoreFeatures {
    # Subset rápido de AGENTS.md "Default Features" (cli+arrow+fjall+roaring+
    # advanced-tokenizer+memmap2+fs2+sysinfo+rayon) para el gate local.
    # sysinfo/arrow/advanced-tokenizer/rayon quedan fuera del gate por tiempo de
    # compilación; roaring entra porque storage lo usa en el path base.
    @("--no-default-features", "--features", "cli,fjall,memmap2,fs2,roaring")
}

function Get-AdaptiveJobs {
    # Jobs según RAM (Windows): >=16GB → min(cores,4); >=4GB → min(cores,2); si no 1.
    $sys = Get-CimInstance Win32_ComputerSystem -ErrorAction SilentlyContinue
    $ram = if ($sys.TotalPhysicalMemory) { [math]::Round($sys.TotalPhysicalMemory / 1GB) } else { 2 }
    $cores = if ($sys.NumberOfLogicalProcessors) { $sys.NumberOfLogicalProcessors } else { 1 }
    if ($ram -ge 16) { [math]::Min($cores, 4) } elseif ($ram -ge 4) { [math]::Min($cores, 2) } else { 1 }
}
