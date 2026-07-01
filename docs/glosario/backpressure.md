---
type: glossary-entry
status: stable
tags: [vantadb, glosario, operaciones, resiliencia]
last_refined: 2026-06
links: "[[README.md]]"
---
#Backpressure

##Definition

**Backpressure** is a flow control mechanism that allows a system to signal its clients to reduce the rate of requests when it is under excessive load, preventing collapse due to resource exhaustion.

## Purpose

| Problema | Sin Backpressure | Con Backpressure |
|----------|------------------|------------------|
| **OOM** | Proceso terminado por kernel | Rechazo controlado de requests |
| **Latencia** | Degradación infinita | Degradación graceful |
| **Cascada** | Fallo en cadena | Aislamiento de fallo |

## Implementation in VantaDB

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

### SDK integration

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

## Configuration

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

## Customer Signaling

###Python

```python
from vantadb import VantaEmbedded, VantaError

try:
    db.put(key="doc1", vector=[...])
except VantaError.BackpressureActive as e:
    print(f"System under load: {e}")
    # Wait and retry
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

## Telemetry Metrics

```rust
pub struct BackpressureMetrics {
    pub rejections_total: Counter,
    pub current_rss_bytes: Gauge,
    pub threshold_ratio: Gauge,
    pub time_under_pressure: Histogram,
}
```

## Mitigation Strategies

### 1. Retry with Exponential Backoff

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

### 2. Client Rate Limiting

```python
import asyncio
from aiolimiter import AsyncLimiter

limiter = AsyncLimiter(100, 1) # 100 requests/second

async def batch_insert(docs):
    for doc in docs:
        async with limiter:
            await async_put(doc)
```

### 3. Queue with Priorities

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

## Relationship with Memory Domains

| Dominio | Monitoreo | Acción |
|---------|-----------|--------|
| **RSS** | sysinfo::Process | Backpressure si >80% |
| **HNSW Lógico** | Estimación interna | Alerta si crece rápido |
| **mmap Resident** | mincore() | Prefetch si >90% |

## See Also

- [[mmap]] — Memory-mapped I/O
- [[wal]] — Write-ahead log
- [[chaos-testing]] — Backpressure validation

---

*Backpressure is essential for the stability of VantaDB under load, preventing OOM kills and graceful degradation.*

