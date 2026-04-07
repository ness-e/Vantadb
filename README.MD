<div align="center">
  <img src="https://raw.githubusercontent.com/ness-e/ConnectomeDB/main/connectome-web/public/favicon.ico" width="80" alt="NexusDB Logo">
  <h1>NexusDB</h1>
  <p><strong>The vector-graph database that thinks.</strong></p>
  <p>Memoria a Largo Plazo extrema para Agentes IA. Vectores, Grafos y Reglas Lógicas en un motor bio-inspirado escrito en Rust.</p>

  <p>
    <a href="https://connectomedb.dev"><img src="https://img.shields.io/badge/Website-connectomedb.dev-00d4ff?style=flat-square" alt="Website"></a>
    <a href="https://hub.docker.com/r/ness-e/connectomedb"><img src="https://img.shields.io/badge/docker-pull-0ea5e9?style=flat-square&logo=docker" alt="Docker"></a>
    <a href="https://github.com/ness-e/ConnectomeDB/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-a855f7?style=flat-square" alt="License"></a>
    <img src="https://img.shields.io/badge/Rust-1.75+-ce422b?style=flat-square&logo=rust" alt="Rust Version">
  </p>
</div>

---

## 🛑 El Problema: El contexto del agente se fragmenta

Para construir un agente autónomo actualmente, necesitas pegar múltiples bases de datos:
1. **Pinecone/Qdrant** para retención de memoria (búsqueda semántica).
2. **Neo4j** para el mapa de conocimiento (búsqueda de grafos).
3. **PostgreSQL** para payloads de estado (filtros relacionales).

**El resultado:** Escrituras parciales, latencia multiplicada (round trips de red) y 5 credenciales diferentes. Tu agente razona bien, pero opera sobre infraestructura fragmentada.

## 🧠 La Solución: NexusDB

**NexusDB** revoluciona la memoria de Inteligencia Artificial mediante el `UnifiedNode`: una estructura de almacenamiento escrita en Rust (sobre RocksDB) que empaqueta embeddings vectoriales (`HNSW`), relaciones dirigidas explícitas (`Vec<Edge>`) y metadatos relacionales en un **mismo registro local**. 

Diseñada para ser incrustada directamente en tu proceso (vía SDK Python nativo) o consumida externamente vía HTTP/MCP, NexusDB lee, razona y escribe en un mismo bloque de memoria. Sin latencias de red, sin fricción operacional.

---

## ⚡ 60-Second Quickstart

Despliega el motor completo en minutos y dota de Memoria In-Process a tu agente:

### Vía Python SDK (SQLite-like Native Binding)
```bash
pip install nexusdb-py
```
```python
from nexusdb_py import NexusDB

# In-process memory (0 Network Latency)
db = NexusDB("agent_memory", max_memory=1024*1024*1024)
db.insert(1, "The user prefers strictly technical answers", [0.2, -0.5, 0.9])
print(db.query('FROM Node WHERE content ~ "technical" LIMIT 1'))
```

### Instalación en 1 comando con Docker (Modo Survival Sensible)
El contenedor oficial de NexusDB es inteligente: detecta los límites impuestos por Docker (Cgroups) y avisa al motor para que ajuste los backends internos.

```bash
# Arranca el servidor NexusDB limitándolo a 512MB de RAM
# El entorno detectará la presión de memoria y activará "Survival Mode" (MMap Hybrid Backend) sin crashear.
docker run -d \
  --name nexus \
  -m 512m --cpus 1 \
  -p 8080:8080 \
  nexusdb:latest
```

Entra a la Consola IQL (Intelligent Query Language):
```bash
docker exec -it nexus nexusdb-cli
```

---

## 🔍 IQL: El lenguaje de la I.A.

**Intelligent Query Language (IQL)** es el lenguaje de NexusDB. Cruza el espacio semántico, salta por el grafo y aplica filtros ACID.

### RAG Híbrido Real en un Request
```sql
-- Busca semánticamente, filtra por relación de grafo y propiedades estructuradas

LET $vec = fn::embed("arquitectura RAG local");

FROM Producto → COMPRADO → Usuario
WHERE bio ~ "AI Engineer" 
  AND edad > 25
  AND descripcion ~ $vec, min = 0.85
TRAVERSE "sigue_a" 1..3
LIMIT 10;
```
*Resultado en 4ms. Un solo round-trip. Sin glue code.*

### Memoria Persistente de Agentes
```sql
-- Inserta la memoria de contexto en tiempo de ejecución
BEGIN TRANSACTION
  UPDATE Agente#ventas SET estado = "reflexivo"
  INSERT MEMORY#102 FOR Agente#ventas {
    content: "El usuario prefiere respuestas muy técnicas",
    ttl: 7d
  } VECTOR [0.2, -0.5, ...];
COMMIT
```

---

## 📊 Benchmarks vs Sistemas Tradicionales

Pruebas corridas en laptop (16GB RAM) en cold start. Sin Cloud. Sin GPU.

| Operación | NexusDB | Qdrant | Neo4j | pgvector |
|---|---|---|---|---|
| **Vector KNN (100k, 384d)** | `3.8ms` | 5.2ms | *N/A* | 12ms |
| **Graph BFS (depth=3)** | `1.2ms` | *N/A* | 4.5ms | *N/A* |
| **Hybrid (vector+graph+filter)** | `8ms` | *N/A (requiere middleware)* | *N/A* | *N/A* |
| **Cold Start RAM** | `15MB` | 180MB | 2.1GB (JVM) | 400MB |

---

## 🛠️ Arquitectura Interna y Soberanía Cognitiva (v0.5.0)

- **NeuLISP (Inferencia Cognitiva):** Capa de ejecución híbrida orientada al Hardware (SIMD). Las instrucciones S-Expressions no solo ejecutan lógica booleana, sino que retornan Tensores de Certeza (`(Value, TrustScore)`), permitiendo evaluar probabilidades e incertidumbres en tiempo de ejecución de las reglas operativas locales.
- **Soberanía y Lóbulos de Memoria:** El almacenamiento RocksDB está particionado anatómicamente en Column Families: `default` (Primario), `shadow_kernel`, `tombstones` y `deep_memory`.
- **Gobierno Biológico (Presupuesto de Amígdala):** El recolector de basura circadiano (`SleepWorker`) audita y aplica Olvido Bayesiano, pero respeta incondicionalmente un "Presupuesto de Valencia" (máximo 5% de memoria L1 retenida inmutable por alta carga semántica/emocional).
- **Devil's Advocate:** El motor audita proactivamente silenciosamente cada escritura para evitar disonancias lógicas cruzadas en el índice HNSW.
- **CP-Index & SIMD:** Combina máscaras de bits `u128` (Co-location) con SIMD-Accelerated cosine distance (vía crate `wide` AVX-512) para que las macros NeuLISP que usan el operador de similitud `~` se resuelvan en sub-milisegundos.
- **Arqueología Semántica:** Protocolo de Rehidratación de Memoria que recupera datos archivados del Shadow Kernel cuando una Neurona de Resumen tiene baja fidelidad (`TrustScore < 0.4`), con sincronización HNSW inmediata y limpieza circadiana automática.
- **MCP (Model Context Protocol):** Comunicación STDIO (JSON-RPC 2.0) para integración directa con Claude Desktop, Cursor y otros orquestadores de IA.

---

## 🤝 Comunidad y Soporte

NexusDB es open source (Apache 2.0) y construido incansablemente para el ecosistema emergente de IA autónoma.

- 📚 [Documentación Oficial](https://connectomedb.dev/docs)
- 💬 [Discord de la Comunidad](https://discord.gg/connectomedb)
- 🐛 [Reportar un Bug](https://github.com/ness-e/ConnectomeDB/issues)
- 💖 [Sponsor el Desarrollo](https://github.com/sponsors/ness-e)

## Licencia

Este proyecto está bajo la licencia [Apache License Version 2.0](LICENSE).