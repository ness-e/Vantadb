# Bases de Datos Embebidas/Vectoriales: Modelos de Negocio Reales

**Investigación de mercado para VantaDB** · Fecha: 2026-08-25 · Agente: vanta-research (B) · Verificación: páginas oficiales de pricing extraídas el día de la fecha salvo indicación

---

## Resumen Ejecutivo

El patrón dominante en 2025-2026 es **motor/librería open source gratuita + monetización de la operación cloud**. Ningún proyecto de la muestra cobra por el motor embebido en sí: SQLite va más lejos aún (dominio público total) y sobrevive vendiendo soporte, garantías legales y extensiones privativas. El dinero está en cuatro capas, en orden decreciente de margen:

1. **Servicios IA gestionados** sobre la BD (embeddings, reranking, agentes de consulta) — facturados por tokens/requests, margen alto (Weaviate, Pinecone).
2. **Operación cloud**: cómputo por hora/segundo, almacenamiento por GB, unidades de lectura/escritura (MotherDuck, Pinecone, Zilliz, Chroma, Qdrant).
3. **Sync y replicación** como dimensión de cobro explícita — único caso: Turso, que factura los GB sincronizados entre réplicas embebidas y cloud.
4. **Compliance enterprise**: SSO, BYOK, HIPAA, BYOC, SLAs — gates de planes superiores en prácticamente todos.

Para un motor de memoria embebido para agentes IA (el caso VantaDB), el modelo más replicable es **Turso** (motor MIT libre + cloud que cobra sync, backup/PITR y compliance), con piso estilo SQLite (soporte/warranty) y capa alta estilo Weaviate/Pinecone (servicios IA gestionados). Las trampas principales: paywallea el motor core, licencias restrictivas (lección Redis→Valkey), y pricing cerrado sin cifras públicas (LanceDB).

---

## 1. SQLite

| Dimensión | Detalle |
|---|---|
| **Licencia** | Dominio público (dedicación explícita con affidavits firmados archivados en Hwaci; "open-source, not open-contribution") |
| **Split open-core** | No hay edición paga del motor. Extensiones privativas vendidas aparte: SEE (SQLite Encryption Extension) ~US$2,000 licencia perpetua por producto [confianza media]; también CES y TH3 |
| **Modelo de cobro** | (a) Warranty of Title (documento legal que Hwaci vende a empresas que exigen prueba de derechos; los ingresos financian el desarrollo); (b) membresías del SQLite Consortium; (c) contratos de soporte horarios; (d) venta de extensiones de cifrado |
| **Precios** | Motor: US$0 siempre. SEE ~$2,000/producto [media]. Consortium/soporte: cifras no públicas [baja] |
| **Distribución** | Amalgamation C desde sqlite.org; empaquetado como dependencia de sistema en prácticamente todo OS/lenguaje |

**Insight clave:** SQLite demuestra que un motor embebido puede financiarse *sin* cloud propio, pero requiere reputación monolítica (está en todo iPhone/Android/navegador) — no replicable como estrategia primaria para un entrante.

## 2. Turso / libSQL

| Dimensión | Detalle |
|---|---|
| **Licencia** | MIT (libSQL, fork de SQLite; núcleo Turso Database también open source) |
| **Split open-core** | Motor embebido, embedded replicas y sync protocol: gratis/open. Lo pagado vive 100% en cloud: cuotas superiores, retención PITR extendida, audit logs, teams, SSO, BYOK, HIPAA/SOC2, soporte prioritario, Dedicated Cloud/BYOC |
| **Modelo de cobro** | Suscripción mensual por plan + overages por uso. **Único del mercado que factura "Syncs" (GB sincronizados) como dimensión de primer nivel** |
| **Precios** (verificados hoy) | Free $0 (100 DBs, 5GB, 500M rows-read/mes, 10M rows-written, 3GB syncs, PITR 1 día) · Developer **$4.99/mes** (9GB +$0.75/GB, 2.5B reads +$1/B, 25M writes +$1/M, syncs 10GB +$0.35/GB, PITR 10d, audit 3d) · Scaler **$24.92/mes** (24GB +$0.50, 100B reads +$0.80/B, syncs 24GB +$0.25/GB, PITR 30d, Teams) · Pro **$416.58/mes** (50GB +$0.45, 250B +$0.75, syncs 100GB +$0.15/GB, SSO, BYOK, HIPAA, SOC2) · Enterprise custom (Dedicated Cloud, BYOC, SLA 24×7) [ciclo mensual/anual según toggle JS — confianza media] |
| **Distribución** | npm (@libsql/client), crates.io, PyPI (libsql-experimental), brew; CLI turso |

**Insight clave:** el template exacto para VantaDB. El sync embebido↔cloud es la feature puente entre "libre en tu máquina" y "pago cuando crece".

## 3. DuckDB vs MotherDuck

| Dimensión | Detalle |
|---|---|
| **Licencia** | DuckDB: MIT, gobernado por DuckDB Foundation (independencia de vendor como selling point) |
| **Split open-core** | Motor 100% gratis y completo. MotherDuck añade cloud persistente, colaboración, snapshots, RBAC, Flights (pipelines agent-native), instancias Pulse/Standard/Jumbo/Mega/Giga |
| **Modelo de cobro** | Flat por organización + usage (storage + cómputo por segundo + unidades IA) |
| **Precios** (verificados hoy) | Lite $0 (3 usuarios activos + 2 service accounts, 10GB, 10h Pulse/mes) · Business **$250/org/mes + uso**: storage $0.04/GB-mes; instancias **$0.60 / $2.40 / $4.80 / $12 / $24 por hora** facturadas por segundo; read-scaling hasta 16 réplicas ($0.60/h c/u); AI Units $1.00; snapshots 90 días; SLA 99.9% · Enterprise custom (PrivateLink, IP allowlisting, HIPAA BAA, capacidad fixed-cost) |
| **Distribución** | duckdb CLI/extensiones, PyPI/npm/crates/RubyGems/conda; MotherDuck vía signup web |

**Insight clave:** el motor gratuito es el canal de adquisición masivo del cloud pago ("duckling" gratuito de 10GB). La separación fundación/vendor genera la confianza necesaria para ser adoptado como default.

## 4. Chroma

| Dimensión | Detalle |
|---|---|
| **Licencia** | Apache 2.0 |
| **Split open-core** | Motor completo gratis (vector + full-text + metadata search). Cloud serverless añade escala, terabytes, SOC II, equipos, soporte |
| **Modelo de cobro** | Usage-based puro (serverless): escritura, almacenamiento, consulta, red |
| **Precios** (verificados hoy) | Starter $0/mes + uso (**$5 créditos gratis**): escritura **$2.50/GiB**, almacenamiento **$0.33/GiB-mes**, query **$0.0075/TiB consultado** [unidad tal como renderiza la página — posible artefacto], network $0.09/GiB devuelto; 10 DBs, 10 miembros · Team **$250/mes + uso** ($100 créditos): 100 DBs, 30 miembros, Slack, SOC II, descuentos por volumen · Enterprise custom (single-tenant, BYOC, SLAs) |
| **Distribución** | PyPI (chromadb), npm (chromadb), Docker; cloud por signup |

**Insight clave:** free-tier con créditos + cobro por operación de escritura (no leída) — la ingesta es lo caro en vector search. Precio Team idéntico al estándar del sector ($250).

## 5. Qdrant

| Dimensión | Detalle |
|---|---|
| **Licencia** | Apache 2.0 (Rust) |
| **Split open-core** | Motor completo gratis, self-hosted. Cloud añade managed ops, HA, backup/DR, GPU indexing, shard splitting; Premium añade SSO, VPC links, 99.9%; Hybrid/Private Cloud = management plane sobre infra propia |
| **Modelo de cobro** | Uso horario de recursos (vCPU, RAM, storage, backups + tokens de inferencia). Sin fee por query |
| **Precios** | Free forever: 0.5 vCPU/1GB RAM/4GB disk · Standard: usage-based sin precio público fijo (calculadora requiere login); terceros citan desde ~**$0.014/hora** [confianza baja-media]; marketables vía AWS/Azure/GCP · Premium: minimum spend custom |
| **Distribución** | PyPI/npm/crates (cliente), Docker, Helm, marketplaces cloud; **Qdrant Edge**: versión in-process/embebida anunciada en GitHub releases — gratis |

**Insight clave:** señal de mercado directa — un vector DB server-only lanzó su variante embebida (Edge) porque el mercado de agentes la exige. Valida el nicho de VantaDB y acorta su ventana.

## 6. LanceDB

| Dimensión | Detalle |
|---|---|
| **Licencia** | Apache 2.0 (core Rust; Lance format con gobernanza comunitaria formal desde nov-2025; SDKs 1.0 dic-2025) |
| **Split open-core** | Embedded library + formato Lance: gratis y completos. Pagado: LanceDB Cloud serverless, LanceDB Pro (referenciada en blog prosumer), Enterprise (Multimodal Lakehouse, BYOC, SOC2/GDPR/HIPAA) |
| **Modelo de cobro** | Hoy: contacto/ventas para todo lo pago — **lancedb.com/pricing ya no publica cifras** (redirige a formulario). Histórico: free tier serverless + PAYG |
| **Precios** | Sin cifras públicas 2025-2026 [verificado]. Referencia indirecta: benchmark propio "100M vectores ≈ $779/mes" (newsletter may-2026) [media] |
| **Distribución** | PyPI (lancedb), npm (@lancedb/lancedb), crates (lance); posicionamiento agresivo como **memory layer para agentes**: default en CrewAI (12M downloads/mes), plugins OpenClaw/Hermes/MemGPT, Continue IDE |

**Insight clave:** competidor directo en "memoria de agentes". Su jugada 2025-2026: capturar el nicho agéntico con el embedded gratuito y monetizar arriba tras levantar $30M (jun-2025).

## 7. Milvus / Zilliz Cloud

| Dimensión | Detalle |
|---|---|
| **Licencia** | Milvus: Apache 2.0, proyecto graduado LF AI & Data |
| **Split open-core** | Milvus completo gratis (self-host). Zilliz Cloud añade managed ops, backups, monitoring, SLA, SSO/SAML, audit logs, RBAC granular, multi-replica, VPC peering, CMEK, BYOC, zero-copy sobre datos externos |
| **Modelo de cobro** | Serverless por vCU o Dedicated por CU-hora + storage. Precios normalizados "por millón de vectores/mes" |
| **Precios** (verificados hoy) | Free $0 (5GB, 2.5M vCUs/mes, 5 colecciones) · Standard: serverless desde $0; Dedicated ≈**$126/CU-mes** performance-optimized (≈$63/M vectores), capacity ≈$16/M vectores, tiered-storage ≈$5/M vectores (768-dim) · Enterprise desde **$197/mes** (SLA 99.95%, SSO, audit) · Business Critical custom (HIPAA-eligible, CMEK) · BYOC disponible |
| **Distribución** | PyPI/npm/SDKs Go/Java, Docker, Helm; Zilliz Cloud en AWS/GCP/Azure |

**Insight clave:** la única que publica precio normalizado por millón de vectores — benchmark directo para pricing futuro de VantaDB Cloud.

## 8. Weaviate

| Dimensión | Detalle |
|---|---|
| **Licencia** | BSD-3-Clause |
| **Split open-core** | Weaviate OSS completo (hybrid search, multi-tenancy, compresión). WCD añade gestión, HA, backups retenidos, SSO/SAML, PrivateLink, customer keys, HIPAA; add-ons IA solo en cloud: Embeddings gestionados y Query Agent |
| **Modelo de cobro** | Mínimo mensual por plan + dimensión vectorial almacenada + storage + backups + servicios IA por uso |
| **Precios** (verificados hoy) | Free forever (100k objetos, embeddings 2,000 req/día, Query Agent 1,000 req/mes) · Flex desde **$45/mes** PAYG (dimensión desde $0.00465/1M, storage $0.12/GiB, SLA 99.5%) · Premium desde **$400/mes** prepago (SLA 99.95%, SSO, HIPAA, PrivateLink) · **Embeddings**: $0.025–0.065/M tokens según modelo · **Query Agent**: $30/org/mes con 4,000 requests incluidos |
| **Distribución** | PyPI/npm/Go clients, Docker/Kubernetes/Helm, embedded-weaviate (modo dev), DigitalOcean marketplace; Engram GA 2026 |

**Insight clave:** mejor ejemplo de **servicios IA como segunda línea de ingresos**: el Query Agent se cobra por request encima de la BD. Patrón aplicable a un "Memory Agent" de VantaDB.

## 9. Pinecone (cerrado puro)

| Dimensión | Detalle |
|---|---|
| **Licencia** | Propietaria/closed-source (no self-host) |
| **Split open-core** | N/A — todo servicio. BYOC en Enterprise |
| **Modelo de cobro** | Serverless puro: storage + write units + read units con mínimos mensuales; Inference (embeddings/rerank) y Assistant facturados aparte |
| **Precios** (verificados hoy) | Starter gratis (hasta 2GB, 5 índices, 2M writes + 1M reads/mes, us-east-1) · Builder **$20/mes flat** (10GB, Prometheus/Datadog, multi-proyecto) · Standard **mínimo $50/mes** (storage **$0.33/GB-mes**, writes **$4–4.50/M**, reads **$16–18/M**, RBAC, SSO SAML; HIPAA add-on $190/mes) · Enterprise **mínimo $500/mes** (SLA 99.95%, BYOC, CMEK, SCIM) · Inference: embeddings $0.08–0.16/M tokens, rerank $2/1k req · Assistant: input $8/M, output $15/M |
| **Distribución** | SDKs Python/JS, app.pinecone.io, marketplaces cloud; Pinecone Nexus (knowledge engine para agentes) |

**Insight clave:** el único cerrado puro sobrevive siendo el default histórico de RAG, pero su capa Builder $20 flat y el énfasis en agentes muestran que incluso el líder propietario compite por abajo y por el nicho agéntico.

## 10. Redis → Valkey (caso de estudio: cambio de licencia)

| Etapa | Dato |
|---|---|
| Cronología | BSD-3 histórica → **20-mar-2024**: Redis 7.4 pasa a dual RSALv2 + SSPLv1 (no reconocidas por OSI) → fork **Valkey** (BSD-3, Linux Foundation, respaldado por AWS/Google/Oracle; Percona lanza soporte enterprise) → **1-may-2025**: Redis 8 añade **AGPLv3** (tri-license) |
| Reacción del ecosistema | Hyperscalers adoptaron Valkey (ElastiCache for Valkey más barato); Percona monetiza el fork; parte de la comunidad permaneció en Valkey pese a la reapertura parcial |
| Monetización Redis Ltd | Redis Cloud/Enterprise + soporte |

**Lección estructural:** cambiar la licencia del motor rompe confianza permanentemente, crea forks financiados por tus propios competidores, y el retorno posterior no recupera el ecosistema perdido. Para VantaDB: la licencia permisiva del motor es un activo de distribución, no una concesión.

---

## Tabla Comparativa Consolidada

| Proyecto | Licencia motor | Motor OSS gratis | Modelo de cobro principal | Entrada de pago | Dimensión distintiva | Canal principal |
|---|---|---|---|---|---|---|
| SQLite | Dominio público | ✅ todo | Soporte + warranty + extensiones privativas | ~$2,000 (SEE) [media] | Licencia perpetua por producto | Empaquetado universal |
| Turso/libSQL | MIT | ✅ todo | Plan mensual + overages | $4.99/mes | **GB sincronizados (Syncs)** | npm/crates/PyPI + cloud |
| DuckDB | MIT | ✅ todo | Cloud flat + cómputo/segundo | $0 Lite → $250/org | Instancias $0.60–$24/h | PyPI/npm/CLI + cloud |
| Chroma | Apache 2.0 | ✅ todo | Serverless usage-only | $0 + créditos | GiB escritos ($2.50) | PyPI/npm + cloud |
| Qdrant | Apache 2.0 | ✅ todo (+ Edge embebido) | Recursos por hora | $0 free 1GB | vCPU/RAM/GB-hora | Docker/PyPI + marketplaces |
| LanceDB | Apache 2.0 | ✅ todo | Contact-sales | n/p | n/p | PyPI/npm/crates |
| Milvus/Zilliz | Apache 2.0 | ✅ todo | CU-mes o vCU serverless | $0 free 5GB → $197 Ent | **$/M vectores-mes** ($5–63) | PyPI/Docker + cloud |
| Weaviate | BSD-3 | ✅ todo (+ embedded dev) | Mínimo mensual + dimensión vectorial | $45/mes | **Dimensión vectorial/1M** + tokens IA | PyPI/npm/Docker + cloud |
| Pinecone | Propietaria | ❌ solo servicio | Serverless con mínimo | $20 flat / $50 mín | Read/write units | SDKs + marketplaces |
| Redis→Valkey | BSD→RSAL/SSPL→+AGPLv3 | ✅ (Valkey) | Cloud/Enterprise + soporte | n/p | Memoria/nodo gestionado | Package managers + clouds |

---

## Lecciones para VantaDB

### Qué modelo copiar

1. **Turso como plantilla maestra** (espejo casi perfecto): motor embebido MIT distribuido gratis por crates/PyPI/npm; cloud que monetiza las tres cosas que un usuario de memoria embebida eventualmente necesita: **sync multi-dispositivo (medido en GB)**, **backup/PITR (retención escalonada 1d→90d como ladder de planes)** y **compliance (SSO/BYOK/HIPAA solo en Pro)**. Escalera $0 → $4.99 → $24.92 → $416.58 → Enterprise.
2. **Piso estilo SQLite**: contratos de soporte/warranty legal para empresas — costo marginal casi cero, credibilidad instantánea.
3. **Capa alta estilo Weaviate/Pinecone**: servicios IA gestionados sobre la memoria (embedding-as-a-service, un "Memory Agent") facturados por tokens/requests — el retrieval puro es commodity, el servicio encima no.

### Qué evitar

1. **Paywallea el motor core**: nadie lo paga — Chroma/Qdrant/LanceDB regalan todo eso; peor aún, Qdrant lanzó Edge embebido gratis y LanceDB se posiciona como memory layer de agentes. La ventana es de distribución y velocidad, no de licencia.
2. **Cambios de licencia restrictivos**: caso Redis→Valkey — fork respaldado por hyperscalers, daño permanente. Con Rust/MIT/Apache/FSL desde día uno, la licencia permisiva es canal de adquisición.
3. **Pricing cerrado sin cifras** (LanceDB): mata la conversión self-serve; funciona solo con marca enterprise instalada.

### Features candidatas naturales a pago (ordenadas por evidencia de mercado)

| Feature | Evidencia de que se paga | Prioridad VantaDB |
|---|---|---|
| Sync embebido↔cloud / multi-dispositivo | Turso factura GB-sync como dimensión de primer nivel | ⭐⭐⭐ máxima |
| Backup cloud + PITR con retención escalonada | Turso (1d/10d/30d/90d), Weaviate (7d/30d/45d), MotherDuck (90d) | ⭐⭐⭐ |
| Embeddings gestionados + agente de consulta semántica | Weaviate Embeddings/Query Agent, Pinecone Inference/Assistant, MotherDuck AI Units | ⭐⭐⭐ alto margen |
| Compliance tier (SSO, BYOK, HIPAA, BYOC, audit logs) | Gate del plan Pro/Premium en todos los casos | ⭐⭐ (cuando haya clientes enterprise) |
| Consola admin + observabilidad avanzada | Pinecone Builder incluye métricas; Weaviate metrics solo ≥Premium | ⭐⭐ |
| HA/replicación gestionada | Estándar en toda oferta cloud | ⭐ (post-cloud) |

### Trampas identificadas (features que la gente NO paga)

- **Telemetría/analytics sola** como producto pago — siempre bundled, nunca SKU independiente.
- **Consola admin básica** — esperada gratis.
- **Performance/indexación superior como tier de pago** — Qdrant/LanceDB regalan GPU indexing y sus mejores algoritmos; el rendimiento es arma de marketing, no de facturación.
- **"Prosumer flat fee" sin cloud detrás** — los tiers flat baratos funcionan porque hay un cloud operando detrás; un binario local con suscripción flat sin servicio asociado no tiene análogo exitoso en la muestra.

### Dato de timing

Tres movimientos 2025-2026 convergen sobre el nicho de VantaDB: Qdrant Edge (embebido), LanceDB como memory layer por defecto de frameworks de agentes (CrewAI 12M downloads/mes), y Pinecone Nexus (knowledge engine para agentes). El mercado confirma demanda; nadie ha consolidado todavía la posición de "SQLite de la memoria de agentes".

---

## Fuentes Consultadas

Verificadas por extracción directa el 2026-08-25:

1. https://turso.tech/pricing — tabla completa de planes y overages
2. https://motherduck.com/pricing/ — planes Lite/Business/Enterprise, instancias, FAQ facturación
3. https://www.trychroma.com/pricing — Starter/Team/Enterprise y rates de uso
4. https://qdrant.tech/pricing/ — Free/Standard/Premium, facturación por recursos
5. https://zilliz.com/pricing — tipos de cluster, calculadora con ejemplo $126.10/mes
6. https://weaviate.io/pricing — Free/Flex/Premium, Embeddings y Query Agent
7. https://www.pinecone.io/pricing/ — tabla completa + Inference + Assistant
8. https://www.sqlite.org/copyright.html — dominio público, Warranty of Title

Consultadas secundarias:

9. https://lancedb.com/pricing — verificada como página de contacto
10. https://lancedb.com/blog/openclaw-memory-from-zero-to-lancedb-pro — existencia LanceDB Pro
11. https://redis.io/legal/licenses/ — tri-license Redis 8
12. https://github.com/redis/redis/pull/13997 — PR AGPLv3 (merged may-2025)
13. https://en.wikipedia.org/wiki/Server_Side_Public_License — cronología Redis
14. https://www.percona.com/about-percona/newsroom/press-releases/percona-announces-comprehensive-enterprise-grade-support-for-open-source-redis-alternative-valkey
15. https://qdrant.tech/documentation/cloud-pricing-payments/ y https://cloud.qdrant.io/calculator
16. https://alternatives.co/software/qdrant/pricing/ [confianza baja-media]
17. https://github.com/qdrant/qdrant/releases — anuncio Qdrant Edge

**Balance de verificación: 8/10 páginas oficiales de pricing resueltas y citadas.**

## Lagunas

1. Precio exacto de SEE (SQLite): ~$2,000 cifra histórica no re-verificada hoy contra sqlite.org/purchase. Confianza media.
2. Base temporal de precios Turso: toggle mensual/anual JS no confirmado. Confianza media.
3. Unidad de query de Chroma Cloud ("$0.0075/TiB"): posible artefacto de renderizado. Confianza media-baja.
4. LanceDB Cloud/Pro: cero cifras públicas; dato "~$779/mes por 100M vectores" es marketing propio. Confianza media.
5. Qdrant Standard: calculadora tras login; cifras de terceros no verificadas en fuente primaria. Confianza baja-media.
6. No verificado: descuentos anuales, créditos startup, precios por región, términos partner.
7. Vigencia: precios corresponden al 2026-08-25; estos vendors cambian pricing cada 6-18 meses — re-verificar antes de decisiones finales de pricing.
