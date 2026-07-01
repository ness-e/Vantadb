# 🐍 VantaDB Python SDK

Bindings oficiales de Python para **VantaDB**, un motor de base de datos embebido y nativo en Rust diseñado para **memoria persistente y recuperación vectorial** en aplicaciones de IA local-first.

## 📦 Instalación

### Desde TestPyPI (Recomendado para pruebas)
```bash
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-py
```

### Desde el código fuente (Desarrollo)
Requiere [Rust](https://rustup.rs/) y [Maturin](https://github.com/PyO3/maturin) instalados.
```bash
# Clonar el repositorio
git clone https://github.com/ness-e/Vantadb.git
cd Vantadb/vantadb-python

# Compilar e instalar en el entorno virtual activo
pip install maturin
maturin develop --release
```

## 🚀 Quickstart

```python
import vantadb_py as vdb

# 1. Abrir o crear una base de datos embebida
db = vdb.VantaDB("./my_agent_memory", memory_limit_bytes=128 * 1024 * 1024)

# 2. Almacenar memoria persistente (payload + vector + metadatos)
db.put_memory(
    namespace="agent/session_1",
    key="fact_001",
    payload="El usuario prefiere respuestas técnicas y directas.",
    metadata={"source": "chat", "priority": "high"},
    vector=[0.1, 0.2, 0.3, 0.4]  # Vector denso (ej. embedding de un modelo local)
)

# 3. Recuperar memoria exacta
record = db.get_memory("agent/session_1", "fact_001")
print(record["payload"])

# 4. busqueda-hibrida (Vectorial + Léxica)
# Nota: Requiere un vector de consulta del mismo tamaño que los almacenados
query_vector = [0.15, 0.25, 0.35, 0.45]
results = db.search_hybrid(
    namespace="agent/session_1",
    query_vector=query_vector,
    query_text="preferencias usuario",
    top_k=5
)

for hit in results:
    print(f"Key: {hit['key']}, Score: {hit['score']:.4f}")

# 5. Monitoreo de recursos (Crítico para agentes locales)
stats = db.memory_stats()
print(f"Uso lógico: {stats['logical_bytes'] / 1024:.2f} KB")
print(f"RSS físico: {stats['physical_rss'] / 1024:.2f} KB")

# 6. Cierre seguro
db.close()
```

## 🤖 Caso de Uso: Memoria para Agentes de IA

VantaDB está optimizado para actuar como **memoria a largo plazo** para agentes autónomos locales (Claude, Gemini, LLaMA, etc.):

- **Persistencia Zero-Copy**: Los datos sobreviven a reinicios del agente sin overhead de serialización.
- **busqueda-hibrida RRF**: Combina similitud semántica (vectores) con coincidencia léxica (BM25) para recuperación precisa de contexto.
- **Control de Memoria Explícito**: `memory_limit_bytes` evita que el agente colapse la RAM del dispositivo host.
- **Embebido**: Sin servidores externos, sin Docker, sin latencia de red. Ideal para dispositivos edge y offline.

## 🛠️ Desarrollo y Testing

```bash
# Ejecutar la suite de tests del SDK
pytest tests/test_sdk.py -v

# Formatear código Python
black tests/ vantadb_python/
```

## 📜 Licencia
Distribuido bajo la licencia del proyecto principal VantaDB. Consulta el `LICENSE` en la raíz del repositorio.
