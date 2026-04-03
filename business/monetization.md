# ConnectomeDB — Monetization Plan (Detailed)

> **Relación con documentos existentes:**
> Este plan extiende y detalla `monetizacion_estrategia.md` (modelo Open-Core conceptual)
> y `strategy.md` (propuesta de valor y casos de uso).

---

## 1. Modelo de Licenciamiento: Open-Core Dual

### Decisión Final: Apache 2.0 (Motor Core) + BSL 1.1 (Enterprise Features)

| Componente | Licencia | Justificación |
|---|---|---|
| Motor Central (storage, parser, executor, HNSW, graph) | **Apache 2.0** | Máxima adopción. Compatible con uso enterprise sin miedos legales. Superior a MIT por cláusula de patentes. |
| Enterprise Plugins (sharding, backup S3, audit trail, SSO) | **BSL 1.1** (Business Source License) | Modelo probado por MariaDB, CockroachDB, Sentry. Código visible pero uso comercial requiere licencia. Se convierte en OSS tras 4 años. |
| ConnectomeDB Cloud (SaaS gestionado) | **Propietario** | Ingresos principales. El código del cloud orchestrator nunca se abre. |

### ¿Por qué NO MIT?
MIT no tiene cláusula de patentes. Si AWS clona el proyecto (como hizo con Elasticsearch), no tienes protección legal. Apache 2.0 + BSL es el escudo perfecto.

### ¿Por qué NO AGPL?
AGPL asusta a corporativos. Empresas como Google prohíben internamente usar software AGPL. Perderías el segmento enterprise más lucrativo.

---

## 2. Tiers de Producto

### Tier 1: Community Edition (GRATIS)
```
Licencia: Apache 2.0
Target: Desarrolladores individuales, startups, labs de IA
Límites: NINGUNO en funcionalidad core

Incluye:
✅ Motor completo (Vector + Grafo + Relacional)
✅ IQL Parser completo
✅ HNSW Index nativo
✅ Auto-Embedding (Ollama bridge)
✅ REST API (Axum server)
✅ CLI interactivo
✅ Python SDK (PyO3)
✅ Docker image
✅ Prometheus metrics
✅ TTL + Garbage Collector
✅ RBAC básico (owner_role field-level)
✅ Conversational Primitives (INSERT MESSAGE)

No incluye:
❌ Sharding / replicación
❌ Backup automatizado a S3
❌ Audit trail compliance (SOC2, HIPAA)
❌ SSO / LDAP / SAML
❌ Soporte dedicado
❌ SLA garantizado
```

### Tier 2: Pro ($49/mes por nodo)
```
Licencia: BSL 1.1
Target: Equipos de 5-50 personas, empresas medianas

Todo lo de Community +
✅ Backup incremental a S3/GCS (automático)
✅ Audit trail completo (quién modificó qué, cuándo)
✅ Dashboard web de monitoreo (Grafana preconfigurado)
✅ Enterprise RBAC (policies por tenant, organización)
✅ Soporte por email (48h SLA)
✅ Consultoría de migración (2h incluidas)
```

### Tier 3: Enterprise ($299/mes por nodo)
```
Licencia: BSL 1.1 + Acuerdo Enterprise
Target: Corporativos, bancos, gobierno, health-tech

Todo lo de Pro +
✅ Sharding automático (horizontal scaling, v2.0+)
✅ Replicación multi-nodo (raft consensus, v2.0+)
✅ SSO / LDAP / SAML
✅ SOC2 / HIPAA compliance toolkit
✅ Soporte dedicado Slack/Teams (4h SLA)
✅ Onboarding personalizado
✅ Revisión de seguridad trimestral
```

### Tier 4: ConnectomeDB Cloud (SaaS — desde $29/mes)
```
Licencia: Propietario (managed service)
Target: Equipos que no quieren operar infraestructura

Pricing escalonado:
├── Hobby:     $29/mes  — 1GB data, 1k QPS, 1 region
├── Startup:   $99/mes  — 10GB data, 10k QPS, 2 regions
├── Business:  $299/mes — 100GB data, 50k QPS, 3 regions, backups
└── Custom:    Contacto — Ilimitado, SLA 99.99%, dedicated
```

---

## 3. Canales de Revenue Adicionales

| Canal | Ingresos esperados | Timeline |
|---|---|---|
| **GitHub Sponsors** | $200-500/mes | Mes 1+ |
| **Consulting / Workshops** | $150/h | Mes 3+ |
| **Enterprise Support** | $2,000-10,000/mes por cliente | Mes 6+ |
| **ConnectomeDB Cloud SaaS** | $5,000-50,000/mes | Mes 12+ |
| **Training / Certificación** | $500 por persona | Mes 9+ |
| **Plugin Marketplace** (conectores IA) | 30% commission | Mes 12+ |

---

## 4. Proyección de Ingresos

### Basado en trayectorias comparables:
| Métrica | Turso (SQLite Edge) | Qdrant (Vector DB) | ConnectomeDB (Proyección) |
|---|---|---|---|
| Mes 1 | $0 | $0 | $200 (Sponsors) |
| Mes 6 | $2k MRR | $5k MRR | $1,500 MRR |
| Mes 12 | $15k MRR | $30k MRR | $8,000 MRR |
| Mes 24 | $80k MRR | $200k MRR | $40,000 MRR |

### Escenario conservador ConnectomeDB:
```
MES 1-3:   $200-500/mes   → GitHub Sponsors + primeros consulting
MES 3-6:   $500-1,500/mes → 3 clientes Pro ($49 × 3) + Sponsors + consulting
MES 6-12:  $1,500-8,000   → 1 Enterprise ($299) + 5 Pro + Cloud beta
MES 12-24: $8,000-40,000  → Cloud GA + 3 Enterprise + 20 Pro + Plugin fees
```

### Ruta a los primeros $1,000 en 30 días:
```
Semana 1: GitHub Sponsors page live ($100 target)
Semana 2: HackerNews launch → 500 stars → 2 consulting inquiries ($300)
Semana 3: Ollama partnership blog post → 5 Pro early-access signups ($245)
Semana 4: First DevRel talk (recorded) → 3 more Pro signups + tips ($350)
Total:     ~$995 MRR
```

---

## 5. Competencia: Pricing Landscape

| Competidor | Modelo | Precio Entry | Observación |
|---|---|---|---|
| **Qdrant** | Open-core | $25/mes (cloud) | Solo vectores, sin grafos |
| **Neo4j** | Freemium/Cloud | $65/mes (Aura) | Solo grafos, sin vectores nativos |
| **Pinecone** | SaaS puro | $70/mes | Solo vectores, vendor lock-in |
| **Supabase** | Open-core | $25/mes | PostgreSQL, sin grafos ni HNSW nativo |
| **ConnectomeDB** | Open-core | $29/mes Cloud / $49 Pro | **3-en-1: Vector+Grafo+Relacional** |

### Ventaja competitiva en precio:
> "Reemplazas Qdrant ($25) + Neo4j ($65) + Supabase ($25) = **$115/mes**
> con un solo ConnectomeDB Pro: **$49/mes**. Ahorro del **57%.**"

---

## 6. Unit Economics (Post-MVP)

```
Cloud Hosting Cost per tenant:     ~$8/mes (ARM VPS + NVMe)
Revenue per Startup tenant:        $99/mes
Gross Margin per tenant:           92%

Target: 100 Cloud tenants = $9,900 MRR en Gross, $8,300 Net
Breakeven for 1 person:            15 tenants ($1,485 MRR)
```
