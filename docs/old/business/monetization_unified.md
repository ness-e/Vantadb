# ConnectomeDB — Estrategia de Negocio y Monetización Unificada

Este documento resume la arquitectura comercial recomendada para rentabilizar el desarrollo del motor de base de datos multimodelo "NexusDB/ConnectomeDB", destinado a despliegues locales y de alta eficiencia para IA.

---

## 1. El Modelo de Licenciamiento Ideal: Open-Core (SaaS / Enterprise Dual Licensing)

No privatizarás el proyecto por completo. Las bases de datos propietarias cerradas rara vez obtienen adopción masiva. La estrategia de ConnectomeDB es un modelo dual:

| Componente | Licencia | Justificación |
|---|---|---|
| Motor Central (storage, parser, executor, HNSW, graph) | **Apache 2.0** | Máxima adopción. Compatible con uso enterprise sin miedos legales. Superior a MIT por cláusula de patentes frente a nubes corporativas. |
| Enterprise Plugins (sharding, backup S3, audit trail, SSO) | **BSL 1.1** (Business Source License) | Modelo probado por MariaDB o CockroachDB. Código visible pero el uso comercial en gran escala requiere licencia. Se convierte en OSS tras 4 años. |
| ConnectomeDB Cloud (SaaS gestionado) | **Propietario** | Ingresos principales. El código del orquestador en la nube es cerrado. |

---

## 2. Métricas y Estadísticas (KPIs) para GTM ("Show me the numbers")

Para volverte viral en foros (HackerNews, Reddit /r/programming), publicarás estas ventajas comparativas:

1. **Memory Footprint:** "ConnectomeDB corre el RAG completo con solo **15 MB de RAM** mientras Neo4j/Weaviate exigen **2 GB** en reposo" (Gracias a zero-copy bincode).
2. **Latencia Vectorial:** "Latencia de **<5 ms** para buscar sobre 1 Millón de vectores combinado con grafos."
3. **Overhead Cero (Ejecución Híbrida):** Buscar biografía, seguir un arco en grafo y buscar similitud toma 1 sola consulta local y 0 latencia de red.
4. **Auto-Embedding:** Delegando la vectorización al backend eliminas cuellos de botella del orquestador.

---

## 3. Tiers de Producto

### Tier 1: Community Edition (GRATIS)
*   **Licencia:** Apache 2.0
*   **Target:** Desarrolladores individuales, startups, labs de IA. 
*   **Incluye:** Motor 3-en-1, IQL, HNSW, Python SDK, Docker, RestAPI (Axum).
*   **No incluye:** Sharding, backups S3, auditorías.

### Tier 2: Pro ($49/mes por nodo)
*   **Licencia:** BSL 1.1
*   **Target:** Equipos de 5-50 personas.
*   **Todo lo del Community +:** Backups a S3, Audit trails, Dashboard Grafana, RBAC Enterprise, Soporte Email 48h.

### Tier 3: Enterprise ($299/mes por nodo)
*   **Licencia:** BSL 1.1 + Acuerdo Oem
*   **Target:** Bancos, Gobierno, Corporativos.
*   **Todo lo de Pro +:** Sharding auto, clustering Raft, SSO/LDAP, Cumplimiento SOC2/HIPAA, Soporte 4h.

### Tier 4: ConnectomeDB Cloud (SaaS — desde $29/mes)
*   **Pricing:** Hobby ($29), Startup ($99), Business ($299), Custom.

---

## 4. Canales de Revenue Adicionales & Proyección de Ingresos

| Canal | Ingresos esperados | Timeline |
|---|---|---|
| **GitHub Sponsors** | $200-500/mes | Mes 1+ |
| **Consulting / Workshops** | $150/h | Mes 3+ |
| **Enterprise Support** | $2,000-10,000/mes por cliente | Mes 6+ |
| **ConnectomeDB Cloud SaaS** | $5,000-50,000/mes | Mes 12+ |
| **Plugin Marketplace** | 30% commission | Mes 12+ |

### Ruta a los primeros $1,000 en 30 días:
*   Semana 1: Activar GitHub Sponsors ($100)
*   Semana 2: Lanzamiento HackerNews → 500 stars → Consultorías ($300).
*   Semana 3: Integración/Post con Ollama → Preventa Pro ($245).
*   Semana 4: Charla DevRel → ($350).

---

## 5. Competencia: Pricing Landscape y Unit Economics

| Competidor | Precio Entry | Observación |
|---|---|---|
| Qdrant | $25/mes | Solo vectores |
| Neo4j | $65/mes | Solo grafos |
| Supabase | $25/mes | Relacional, sin grafos nativos |
| **ConnectomeDB** | **$49 Pro** | **3-en-1 consolidado** |

*Ventaja Competitiva:* Reemplazas DB vectorial + Grafo + Relacional por solo $49/mes. Ahorro del 57%.

**Unit Economics Post-MVP:**
*   Costo Cloud Hosting por tenant: ~$8/mes (ARM VPS + NVMe)
*   Revenue por tenant Startup: $99/mes
*   Gross Margin: 92%
*   Breakeven (1 founder): ~15 tenants ($1,485 MRR).
