# 🧬 ConnectomeDB — Plan Maestro Estratégico
### Estado: v0.5.0 · Quantum Cognition · Fase 31 ✅ Completada
> **Versión del análisis:** 2026-04-06 | **Objetivo comercial:** NexusDB v1.0 → HackerNews Top 5

---

## 📍 Punto de Partida Real (Diagnóstico Honesto)

| Dimensión | Estado Actual | Brecha hacia v1.0 |
|:---|:---|:---|
| **Motor Core** | ✅ Fase 31 completa. HNSW+SIMD+RaBitQ+PolarQuant | Fases 32-35 pendientes |
| **Estabilidad** | ⚠️ 9 test suites passing, parser sin subqueries | Mutaciones IQL en maduración |
| **Ecosistema** | ⚠️ PyO3 opcional, MCP STDIO funcional | SDK Python sin `pip install` |
| **Marketing** | ❌ README en ES, sin demo en vivo | README EN + video + Docker Hub |
| **Monetización** | ❌ $0 MRR | Target: $2k MRR en 6 meses |

---

## 🎯 El Pivote Estratégico: Del Laboratorio al Mercado

### Principio Director: "Caballo de Troya"
> **Entra al mercado como SIMPLICIDAD. Retenlos con MAGIA.**

El dev no quiere entender `QuantumNeuron`. Quiere esto:
```bash
docker run -p 8080:8080 nexusdb/nexus:latest
# < 60 segundos después >
# Su agente LangChain tiene memoria persistente local.
```
Una vez dentro, descubre el SleepWorker, el MCP y la compresión cognitiva. *Ahí es cuando se vuelven evangelistas.*

---

## 🗓️ Plan de Ejecución: 4 Semanas al Lanzamiento

> [!IMPORTANT]
> **Reajuste Estratégico de Roadmap:**
> Las siguientes Fases se organizan en Bloques:
> - **Bloque A (Core Stability - Semana 1):** Fase 31B (ThalamicGate & Uncertainty Zones), Fase 32 (Hard-Urgency / NMI), y Fase 35 (MMap Neural Index).
> - **Bloque B (Deferred Post-launch - Meses 3-6):** Fase 33 (Cognitive Plasticity), Fase 34 (Contextual Priming).

### Semana 1 (Días 1-7): Core Stability & Benchmarks + Block A (Fase 31B, 32, 35) ✅ Completado

- [x] **Día 1:** `cargo test --all-features` limpio. Documentar salida en CI.
- [x] **Día 2:** Refactor de API externa solamente. Función `Node` en lugar de `Neuron` como alias público en la HTTP API. Internamente sigue igual.
- [x] **Día 3-4:** Ejecutar `benches/stress_test.rs` con `STRESS_LEVEL=ULTRA` (1M nodos). Capturar resultados en `BENCHMARKS.md` (será el arma arrojadiza en HN).
- [x] **Día 5:** Estabilizar mutaciones IQL: `INSERT`, `UPDATE`, `DELETE`, `RELATE`. El parser NOM no puede arrojar `panic!` en producción.
- [x] **Día 6-7:** Validar `trigger_panic_state` en `chaos_integrity.rs`. Confirmar que RocksDB sobrevive sin corrupción.

**KPIs de la semana:**
- `cargo test` = 0 failures
- Bench: `< 10ms` en KNN 1M nodos vectoriales
- 0 `panic!` no controlados en parser

---

### Semana 2 (Días 8-14): Ecosistema & Integraciones (Bloque B - Fase 37) ⏳ En Progreso

- [ ] **Día 8-9:** Benchmarks (`benches/high_density.rs`) de Alta Densidad con mutaciones Spam (Fricción Logarítmica).
- [ ] **Día 10-11:** Python SDK via PyO3 (`nexusdb-py`). Target: Binding "SQLite-like" `pip install nexusdb-py` funcional sin sobrecarga de red.
- [ ] **Día 12-13:** Dockerization (`debian-slim` multi-stage) con Survival Mode en el arranque (detección de límites RAM en SO y Cgroups).
- [ ] **Día 14:** MCP endpoint y test con Claude Desktop. Las 4 herramientas funcionarán bajo nomenclatura `Node`.

**KPIs de la semana:**
- Python: `from connectomedb import ConnectomeDB; db.search(...)` ✅
- MCP: Claude Desktop conectado en < 2 min de setup
- Docker compose: `docker compose up` → sistema completo online

---

### Semana 3 (Días 15-21): Security, Polish & GTM Assets

- [ ] **Día 15-16:** Mitigación write amplification en SleepWorker. Usar `compact_range_opt` solo cuando entropía > 10k tombstones, no cada ciclo REM.
- [ ] **Día 17:** Límite duro HNSW en Survival Mode (RAM < 8GB). Activar `MmapIndexBackend` automáticamente.
- [ ] **Día 18:** `BENCHMARKS.md` público. Tabla comparativa vs Qdrant/Neo4j/pgvector. Reproducible con un comando.
- [ ] **Día 19-20:** README.md en **Inglés** reescrito. Quickstart en < 60 segundos. Diagrama de arquitectura limpio.
- [ ] **Día 21:** Docker Hub: `docker push nexusdb/nexus:latest` + `nexusdb/nexus:v1.0.0`.

**KPIs de la semana:**
- README: < 5 minutos para que un extraño entienda el valor
- BENCHMARKS.md: todos los números reproducibles
- Docker Hub: imagen pública disponible

---

### Semana 4 (Días 22-28): The GTM Sprint

- [ ] **Día 22:** Demo video de 30 segundos en terminal (asciinema o GIF). Mostrar query híbrido: insertar → buscar semánticamente → traversal de grafo. Un comando, un resultado, tiempo medido.
- [ ] **Día 23:** Artículo: *"Why I built a 3-in-1 database in Rust (and it fits in 15MB)"* → Dev.to + medium.
- [ ] **Día 24:** Post Reddit: `/r/rust` + `/r/selfhosted` + `/r/LocalLLaMA`.
- [ ] **Día 25:** Discord/Slack outreach: LangChain, LlamaIndex, n8n communities.
- [ ] **Día 26:** Preparar HN post. Título definitivo (ver abajo). Draft, review, timing.
- [ ] **Día 27 (Martes, 10 AM EST):** 🚀 **LANZAMIENTO HackerNews**.
- [ ] **Día 28:** Monitorear comentarios. Responder TODOS en < 1 hora. Esto es oro para el algoritmo de HN.

**Título HN (aprobado):**
> *"Show HN: NexusDB – Rust DB unifying vectors, graphs & SQL in a single 15MB binary"*

---

## 🚨 Mitigaciones Críticas (Anti-Patrones a Neutralizar)

### 1. OOM Crash en Edge (HNSW + Survival Mode)
**Riesgo:** Sistema de 8GB hace crash en producción. Usuario Twitter negativo = muerte GTM.
**Fix (Semana 1-2):**
```rust
// En src/index.rs, activar automáticamente si hardware_profile == Survival
if hw.ram_gb < 16 {
    // Forzar MmapIndexBackend para L2 (PolarQuant 3-bit)
    // Solo L1 RaBitQ permanece en RAM (~70% reducción)
}
```
**KPI:** < 500MB RAM total en Survival Mode con 100k nodos.

### 2. Write Amplification (SleepWorker → SSD)
**Riesgo:** En SSDs baratos (NVMe entry-level), la compactación agresiva desgasta el hardware.
**Fix:**
```rust
// Solo compactar cuando tombstones > threshold
if tombstone_count > 10_000 {
    db.compact_range_opt(CF_SHADOW, None, None, &opts);
}
```

### 3. Fricción del Onboarding (DX Gap)
**Riesgo:** "NeuLISP", "QuantumNeuron", "Amygdala Budget" → dev cierra la pestaña.
**Fix:** Capa de traducción en la documentación pública:
| Lo que el dev ve | Lo que es internamente |
|:---|:---|
| `Node` | `UnifiedNode` / `Neuron` |
| `Link` / `Edge` | `Synapse` |
| `Query Engine` | `Cortex` |
| `Memory DB` | `Lobe (Column Family)` |
| `Background Optimizer` | `SleepWorker` |
| `Truth Validator` | `Devil's Advocate` |

---

## 💰 Modelo de Monetización (Open-Core)

```
┌─────────────────────────────────────────────────┐
│  COMMUNITY (Apache 2.0 — SIEMPRE GRATIS)        │
│  - Motor completo, HNSW, NeuLISP, MCP          │
│  - Single-node, local-only                      │
│  - PyO3 SDK, CLI, Docker                        │
└─────────────────────────────────────────────────┘
         ↓ Upgrade natural para equipos
┌─────────────────────────────────────────────────┐
│  PRO / CLOUD ($29–49/mes)                       │
│  - NexusDB Cloud (Fly.io, managed)              │
│  - Backups automáticos                          │
│  - Dashboard de métricas Prometheus             │
│  - Soporte prioritario                          │
└─────────────────────────────────────────────────┘
         ↓ Para Enterprise con compliance
┌─────────────────────────────────────────────────┐
│  ENTERPRISE ($299/mes — BSL para plugins)       │
│  - RBAC avanzado + audit logs                   │
│  - Distributed mode (Raft, v2.0)                │
│  - SLA 99.9%                                    │
│  - On-premise support                           │
└─────────────────────────────────────────────────┘
```

**Targets MRR:**
- Mes 6: $2,000 (40 clientes Pro)
- Mes 9: $8,000 (160 Pro + 3 Enterprise)
- Mes 12: $15,000+ (300 Pro + 10 Enterprise) → Seed Round ready

---

## 🗺️ Roadmap 12 Meses Post-Lanzamiento

```
Q2 2026 — v1.0 LAUNCH
├── HackerNews Top 5
├── 1,000 GitHub Stars (Mes 1)
├── Docker Hub: 500 pulls/semana
└── Primeros 5 clientes Pro ($49/mes)

Q3 2026 — v1.1 ECOSYSTEM
├── WASM Playground en connectomedb.dev
├── LangChain + LlamaIndex integration guide
├── Fases 32 & 33 (Uncertainty Zones + Synaptic Depression)
└── 200 Stars · 20 forks · $2k MRR

Q4 2026 — v1.2 CLOUD BETA
├── NexusDB Cloud (Fly.io managed)
├── Fase 34 (Contextual Priming)
├── Go SDK (alta demanda por microservicios)
└── 500 Stars · $5k MRR · Primeros VCs contactados

Q1 2027 — v2.0 DISTRIBUTED
├── Fase 35 + Raft consensus (openraft crate)
├── Sharding horizontal
├── Enterprise: $299/mes activos
└── Pre-Seed raise: $500k-$1M

Q2 2027 — v3.0 THE AI PLATFORM
├── WASM plugin marketplace
├── Federation Edge (IoT)
├── $10k+ MRR
└── Default DB para agentes IA autónomos
```

---

## ⚡ Decisiones Estratégicas Aprobadas

| Decisión | Elección | Razón |
|:---|:---|:---|
| **Naming público** | NexusDB | Suena a infraestructura seria |
| **Naming motor** | ConnectomeDB / Connectome Engine | "Powered by Connectome Engine" |
| **Licencia Core** | Apache 2.0 (mantener) | No ceder a la tentación BSL por miedo a AWS |
| **Licencia Plugins Enterprise** | BSL para features de pago | Protege monetización sin cerrar la comunidad |
| **Clustering** | Diferido a v2.0 con `openraft` | No reinventar la rueda. WAL → Raft cuando llegue el momento |
| **Python SDK** | PyO3 (prioridad máxima) | 90% del ecosistema IA es Python |
| **Cloud hosting** | Fly.io para Cloud Beta | Sin infra propia hasta traction probada |
| **VC outreach** | Post-lanzamiento, con métricas reales | No levantar antes de tener números |

---

## ⛔ Anti-Patrones a Evitar (Modo Comandante)

1. **Feature Creep pre-lanzamiento.** Las Fases 32-35 son INCREÍBLES técnicamente. Y deben esperar hasta tener 500 stars.
2. **Prometeer clustering antes de que 1000 devs usen single-node.** Sharding sin usuarios = arquitectura de nadie.
3. **README técnico-biologista para el público general.** El dev de mediana empresa no quiere aprender neurociencia.
4. **Ignorar el Python SDK.** Si `pip install connectomedb` falla en macOS ARM, la adopción colapsa. Testear en CI matriz (Linux/macOS/Windows).
5. **No responder comentarios en HN el día del launch.** El fundador que responde = credibilidad = votos.

---

## 📊 KPIs de Control (Dashboard del Comandante)

| Métrica | Semana 1 | Mes 1 | Mes 3 | Mes 6 |
|:---|:---|:---|:---|:---|
| Tests passing | 9/9 ✅ | 12/12 | 15/15 | 20/20 |
| GitHub Stars | — | 1,000 | 2,000 | 5,000 |
| Docker Hub pulls | — | 500 | 2,000 | 10,000 |
| Clientes Pro | — | 0 | 5 | 40 |
| MRR | $0 | $0 | $250 | $2,000 |
| Fases completadas | 31 | 31 | 33 | 35 |

