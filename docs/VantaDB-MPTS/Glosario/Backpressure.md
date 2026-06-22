---
type: glossary-entry
status: stable
tags: [vantadb, glosario, operaciones, resiliencia]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# Backpressure

## Definición

**Backpressure** es un mecanismo de control de flujo que permite a un sistema señalar a sus clientes que reduzcan la tasa de solicitudes cuando está bajo carga excesiva, previniendo el colapso por agotamiento de recursos.

## Propósito

| Problema | Sin Backpressure | Con Backpressure |
|----------|------------------|------------------|
| **OOM** | Proceso terminado por kernel | Rechazo controlado de requests |
| **Latencia** | Degradación infinita | Degradación graceful |
| **Cascada** | Fallo en cadena | Aislamiento de fallo |

## Implementación en VantaDB

### Admission Filter

```rust
pub struct AdmissionFilter {
    sys: sysinfo::System,
    max_rss_bytes: u64,
    threshold: f32,  // Default: 0.8 (80%)
}

impl AdmissionFilter {
    pub fn check(&mut self) -> Result<(), VantaError> {
        self.sys.refresh_memory();
        
        let process = self.sys.process(getpid()).unwrap();
        let rss = process.memory();  // Resident Set Size
        
        if rss as f64 > (self.max_rss_bytes as f64 * self.threshold as f64) {
            return Err(VantaError::BackpressureActive {
                current_rss: rss,
                limit: self.max_rss_bytes,
                threshold: self.threshold,
            });
        }
        
        Ok(())
    }
}
```

### Integración en el SDK

```rust
impl VantaEmbedded {
    pub fn put(&self, node: UnifiedNode) -> Result<()> {
        // Verificar backpressure antes de aceptar
        self.admission_filter.check()?;
        
        // Proceder con la operación
        self.engine.put(node)
    }
}
```

## Configuración

```python
db = VantaEmbedded(
    "./data",
    config={
        "memory": {
            "max_ram_mb": 4096,           # Límite absoluto
            "backpressure_threshold": 0.8  # Activar al 80%
        }
    }
)
```

## Señalización al Cliente

### Python

```python
from vantadb import VantaEmbedded, VantaError

try:
    db.put(key="doc1", vector=[...])
except VantaError.BackpressureActive as e:
    print(f"Sistema bajo carga: {e}")
    # Esperar y reintentar
    time.sleep(1)
    db.put(key="doc1", vector=[...])
```

### HTTP Server (429 Too Many Requests)

```rust
async fn handle_put(req: Request) -> Response {
    match db.put(req.body) {
        Ok(_) => Response::ok(),
        Err(VantaError::BackpressureActive { .. }) => {
            Response::builder()
                .status(429)
                .header("Retry-After", "1")
                .body("System under load")
        }
        Err(e) => Response::error(e),
    }
}
```

## Métricas de Telemetría

```rust
pub struct BackpressureMetrics {
    pub rejections_total: Counter,
    pub current_rss_bytes: Gauge,
    pub threshold_ratio: Gauge,
    pub time_under_pressure: Histogram,
}
```

## Estrategias de Mitigación

### 1. Retry con Exponential Backoff

```python
def put_with_backoff(db, key, vector, max_retries=5):
    for attempt in range(max_retries):
        try:
            db.put(key=key, vector=vector)
            return
        except VantaError.BackpressureActive:
            wait_time = 2 ** attempt  # 1, 2, 4, 8, 16 segundos
            time.sleep(wait_time)
    raise Exception("Max retries exceeded")
```

### 2. Rate Limiting del Cliente

```python
import asyncio
from aiolimiter import AsyncLimiter

limiter = AsyncLimiter(100, 1)  # 100 requests/segundo

async def batch_insert(docs):
    for doc in docs:
        async with limiter:
            await async_put(doc)
```

### 3. Queue con Prioridades

```python
from queue import PriorityQueue

class PriorityInsertQueue:
    def __init__(self, db):
        self.db = db
        self.queue = PriorityQueue()
    
    def enqueue(self, doc, priority=5):
        self.queue.put((priority, doc))
    
    def process(self):
        while not self.queue.empty():
            try:
                priority, doc = self.queue.get()
                self.db.put(doc)
            except BackpressureActive:
                time.sleep(1)
```

## Relación con Dominios de Memoria

| Dominio | Monitoreo | Acción |
|---------|-----------|--------|
| **RSS** | sysinfo::Process | Backpressure si >80% |
| **HNSW Lógico** | Estimación interna | Alerta si crece rápido |
| **mmap Resident** | mincore() | Prefetch si >90% |

## Véase También

- [mmap](mmap.md) — Memory-mapped I/O
- [WAL](WAL.md) — Write-ahead log
- [Chaos Testing](Chaos Testing.md) — Validación de backpressure

---

*Backpressure es esencial para la estabilidad de VantaDB bajo carga, previniendo OOM kills y degradación graceful.*

