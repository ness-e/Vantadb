---
title: "chaos_test_wal.sh"
type: glossary-entry
status: stable
tags: [testing, resiliencia, fault-injection]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Chaos Engineering, Fault Injection Testing]
---
#ChaosTesting

##Definition

**Chaos Testing** (or Chaos Engineering) is the practice of **injecting controlled faults** into a system to validate its resilience and recovery, ensuring that it behaves correctly under adverse conditions.

## Types of Injected Faults

| Tipo de Fallo | Descripción | Valida |
|---------------|-------------|--------|
| **Process kill** | `kill -9` del proceso | Crash recovery |
| **Disk full** | Llenar disco durante write | Error handling |
| **Network partition** | Aislar nodos (si distribuido) | Consistencia |
| **Power loss** | Corte de energía simulado | Durabilidad [[wal]] |
| **Corrupt data** | Modificar archivos en disco | Detección de corrupción |

## Chaos Testing in VantaDB

### Test: Kill -9 During Write

```bash
#!/bin/bash
# chaos_test_wal.sh

for i in {1..1000}; do
    # Start process that writes data
    python write_loop.py &
    PID=$!
    
    # Wait random time (10-100 ms)
    sleep 0.0$((RANDOM % 9 + 1))
    
    # Kill process abruptly
    kill -9 $PID
    
    # Reboot and verify integrity
    python verify_integrity.py
    if [ $? -ne 0 ]; then
        echo "❌ Corruption detected at iteration $i"
        exit 1
    fi
donated

echo "✅ 1000 simulated crashes, zero corruption"
```

### Test: WAL Corruption

```rust
#[test]
fn test_wal_corruption_recovery() {
    let db = VantaEmbedded::open("./test_data")?;
    
    // Escribir 100 documentos
    for i in 0..100 {
        db.put(&format!("key{}", i), &vec![1.0; 128], "test")?;
    }
    
    // Cerrar DB
    drop(db);
    
    // Corromper WAL manualmente
    corrupt_wal_file("./test_data/wal.log")?;
    
    // Reabrir DB
    let db = VantaEmbedded::open("./test_data")?;
    
    // Verificar que recovery funciona
    let count = db.count()?;
    assert!(count >= 50);  // Al menos la mitad debe recuperarse
    
    Ok(())
}
```

## Chaos Testing Frameworks

| Framework | Lenguaje | Caso de Uso |
|-----------|----------|-------------|
| **Chaos Monkey** | Java | Netflix, microservicios |
| **Jepsen** | Clojure | Bases de datos distribuidas |
| **Maelstrom** | Multi | Testing de protocolos distribuidos |
| **Custom scripts** | Bash/Python | VantaDB (chaos local) |

## Resilience Metrics

| Métrica | Descripción | Objetivo |
|---------|-------------|----------|
| **MTTF** (Mean Time To Failure) | Tiempo promedio hasta fallo | >1000 horas |
| **MTTR** (Mean Time To Recovery) | Tiempo promedio de recuperación | <1 minuto |
| **Data loss probability** | Probabilidad de pérdida de datos | <0.001% |
| **Corruption detection rate** | % de corrupciones detectadas | 100% |

## Status in VantaDB

**Actual:** Chaos testing básico implementado (kill -9, WAL corruption)

**Roadmap (PHASE 3):**
- Complete suite of chaos tests
- Integration into night CI
- Published resilience metrics

## See Also

- [[wal]] — Main component validated by chaos testing
- [[crc32c]] — Corruption detection
- [[failpoints]] — Code-level fault injection
- [[ci-cd]] — Chaos tests in nightly pipeline

---

*Chaos testing transforms "we hope it works" into "we show it works under stress."*

