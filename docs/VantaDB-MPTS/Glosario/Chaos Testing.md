---
type: glosario-entry
status: stable
tags: [testing, resiliencia, fault-injection]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Chaos Engineering, Fault Injection Testing]
---

# Chaos Testing

## Definición

**Chaos Testing** (o Chaos Engineering) es la práctica de **inyectar fallos controlados** en un sistema para validar su resiliencia y recuperación, asegurando que se comporta correctamente bajo condiciones adversas.

## Tipos de Fallos Inyectados

| Tipo de Fallo | Descripción | Valida |
|---------------|-------------|--------|
| **Process kill** | `kill -9` del proceso | Crash recovery |
| **Disk full** | Llenar disco durante write | Error handling |
| **Network partition** | Aislar nodos (si distribuido) | Consistencia |
| **Power loss** | Corte de energía simulado | Durabilidad [WAL](WAL.md) |
| **Corrupt data** | Modificar archivos en disco | Detección de corrupción |

## Chaos Testing en VantaDB

### Test: Kill -9 Durante Write

```bash
#!/bin/bash
# chaos_test_wal.sh

for i in {1..1000}; do
    # Iniciar proceso que escribe datos
    python write_loop.py &
    PID=$!
    
    # Esperar tiempo aleatorio (10-100 ms)
    sleep 0.0$((RANDOM % 9 + 1))
    
    # Matar proceso abruptamente
    kill -9 $PID
    
    # Reiniciar y verificar integridad
    python verify_integrity.py
    if [ $? -ne 0 ]; then
        echo "❌ Corruption detected at iteration $i"
        exit 1
    fi
done

echo "✅ 1000 crashes simulados, cero corrupción"
```

### Test: Corrupción de WAL

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

## Frameworks de Chaos Testing

| Framework | Lenguaje | Caso de Uso |
|-----------|----------|-------------|
| **Chaos Monkey** | Java | Netflix, microservicios |
| **Jepsen** | Clojure | Bases de datos distribuidas |
| **Maelstrom** | Multi | Testing de protocolos distribuidos |
| **Custom scripts** | Bash/Python | VantaDB (chaos local) |

## Métricas de Resiliencia

| Métrica | Descripción | Objetivo |
|---------|-------------|----------|
| **MTTF** (Mean Time To Failure) | Tiempo promedio hasta fallo | >1000 horas |
| **MTTR** (Mean Time To Recovery) | Tiempo promedio de recuperación | <1 minuto |
| **Data loss probability** | Probabilidad de pérdida de datos | <0.001% |
| **Corruption detection rate** | % de corrupciones detectadas | 100% |

## Estado en VantaDB

**Actual:** Chaos testing básico implementado (kill -9, WAL corruption)

**Roadmap (FASE 3):**
- Suite completa de chaos tests
- Integración en CI nocturno
- Métricas de resiliencia publicadas

## Véase También

- [WAL](WAL.md) — Principal componente validado por chaos testing
- [CRC32C](CRC32C.md) — Detección de corrupción
- [Failpoints](Failpoints.md) — Inyección de fallos a nivel de código
- [CI_CD](CI_CD.md) — Chaos tests en pipeline nocturno

---

*Chaos testing transforma "esperamos que funcione" en "demostramos que funciona bajo estrés".*

