# Plan de Implementación: Adapters de Ecosistema Python (LangChain / LlamaIndex) y Desacoplamiento de MCP (Fase FEAT-01)

Este plan detalla los aspectos de diseño y la especificación de ingeniería para expandir el ecosistema del SDK de VantaDB integrando adapters nativos y compatibles con **LangChain** y **LlamaIndex**, y para consolidar el desacoplamiento arquitectónico de la interfaz experimental de **MCP (Model Context Protocol)** a un crate de Rust autónomo.

---

## 1. Goal Description

La madurez del SDK de VantaDB requiere facilitar su adopción en frameworks de desarrollo de Agentes de IA y pipelines de RAG locales. Esto se desglosa en:
1. **Adapter de LangChain (`langchain-vantadb`):** Implementar la clase `VantaDBVectorStore` heredando de `langchain_core.vectorstores.VectorStore`, permitiendo ingesta síncrona/asíncrona nativa, búsquedas semánticas y búsqueda híbrida sin fricciones en Python.
2. **Adapter de LlamaIndex (`llamaindex-vantadb`):** Implementar una integración para LlamaIndex (`VantaDBVectorStore`) compatible con la interfaz `BasePydanticVectorStore` o `VectorStore`, permitiendo resolver RAG locales robustos.
3. **Desacoplamiento de MCP (`vantadb-mcp`):** Mover el código experimental JSON-RPC e interfaz de herramientas de `vantadb-server/src/mcp.rs` hacia un nuevo crate autónomo `vantadb-mcp`. Esto asegura que el core del servidor principal no dependa ni mantenga acoplamientos innecesarios, respetando el principio de diseño "embedded-first".

---

## 2. User Review Required

> [!IMPORTANT]
> **Estructura de Carpetas del Workspace de Integraciones:**
> Proponemos crear un subdirectorio `packages/` en la raíz para agrupar los módulos del ecosistema de Python desacoplados del SDK base (`vantadb-py`).
> - `packages/langchain-vantadb/` (paquete instalable vía `pip install -e packages/langchain-vantadb`)
> - `packages/llamaindex-vantadb/` (paquete instalable vía `pip install -e packages/llamaindex-vantadb`)
>
> Esto evita sobrecargar la biblioteca principal `vantadb-py` con dependencias pesadas de terceros (`langchain-core` o `llama-index-core`), lo cual degradaría la experiencia para usuarios que solo buscan el motor embebido puro.

> [!WARNING]
> **Desacoplamiento de MCP a Crate `vantadb-mcp`:**
> La extracción de `mcp.rs` requiere cambiar la configuración del espacio de trabajo de Rust. Agregaremos `vantadb-mcp` como miembro de la raíz `Cargo.toml`. El crate `vantadb-server` pasará a consumir `vantadb-mcp` como una dependencia local limpia. Esto no afecta al ejecutable compilado final, pero limpia radicalmente el diseño del servidor.

---

## 3. Open Questions

No hay preguntas de diseño bloqueantes. Los adapters se implementarán con las firmas API estándar y estables de LangChain y LlamaIndex actuales.

---

## 4. Proposed Changes

### Componente 1: Adapter LangChain (`langchain-vantadb`)

#### [NEW] [packages/langchain-vantadb/pyproject.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/langchain-vantadb/pyproject.toml)
**Especificación:** Archivo de configuración del paquete utilizando `setuptools` o `poetry` para gestionar la distribución e instalación de la integración con LangChain.

#### [NEW] [packages/langchain-vantadb/langchain_vantadb/__init__.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/langchain-vantadb/langchain_vantadb/__init__.py)
**Especificación:** Expone la clase pública `VantaDBVectorStore`.

#### [NEW] [packages/langchain-vantadb/langchain_vantadb/vectorstores.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/langchain-vantadb/langchain_vantadb/vectorstores.py)
**Especificación:** Implementar `VantaDBVectorStore` heredando de `langchain_core.vectorstores.VectorStore`.
- Constructor `__init__` que inicializa o acepta una instancia activa de `VantaDB`.
- `add_texts` para mapear los textos entrantes en nodos persistentes con su embedding computado.
- `similarity_search_with_score` para ejecutar búsquedas en HNSW y retornar tuplas de `(Document, float)` convirtiendo distancias a scores de similitud.
- `similarity_search` que retorna listas de `Document`.
- `from_texts` como factory-method de inicialización rápida.

---

### Componente 2: Adapter LlamaIndex (`llamaindex-vantadb`)

#### [NEW] [packages/llamaindex-vantadb/pyproject.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/llamaindex-vantadb/pyproject.toml)
**Especificación:** Declaración del paquete instalable compatible con LlamaIndex.

#### [NEW] [packages/llamaindex-vantadb/llama_index/vector_stores/vantadb/__init__.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/llamaindex-vantadb/llama_index/vector_stores/vantadb/__init__.py)
**Especificación:** Exporta la clase `VantaDBVectorStore` para LlamaIndex.

#### [NEW] [packages/llamaindex-vantadb/llama_index/vector_stores/vantadb/base.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/llamaindex-vantadb/llama_index/vector_stores/vantadb/base.py)
**Especificación:** Implementar `VantaDBVectorStore` heredando de `llama_index.core.vector_stores.types.VectorStore` o compatible.
- Método `add(self, nodes: List[BaseNode], **kwargs) -> List[str]` para procesar nodos LlamaIndex.
- Método `query(self, query: VectorStoreQuery, **kwargs) -> VectorStoreQueryResult` que ejecuta búsquedas semánticas y filtra resultados con metadatos.
- Método `delete(self, ref_doc_id: str, **kwargs) -> None` para remover elementos persistentes.

---

### Componente 3: Crate Autónomo de MCP (`vantadb-mcp`)

#### [NEW] [vantadb-mcp/Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-mcp/Cargo.toml)
**Especificación:**
```toml
[package]
name = "vantadb-mcp"
version = "0.1.4"
edition = "2021"
description = "Autoccontained MCP interface for VantaDB"

[dependencies]
vantadb = { path = "../", features = ["experimental"] }
tokio = { version = "1", features = ["sync", "rt"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### [NEW] [vantadb-mcp/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-mcp/src/lib.rs)
**Especificación:** Trasladar la lógica del servidor stdio JSON-RPC (`handle_initialize`, `handle_tools_list`, `handle_tools_call` y `run_stdio_server`) desde `vantadb-server/src/mcp.rs`.

#### [MODIFY] [vantadb-server/Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/Cargo.toml)
**Especificación:** Agregar `vantadb-mcp` como dependencia del servidor principal:
```toml
vantadb-mcp = { path = "../vantadb-mcp" }
```

#### [DELETE] [vantadb-server/src/mcp.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/mcp.rs)
**Especificación:** Eliminar el archivo acoplado original.

#### [MODIFY] [vantadb-server/src/main.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/main.rs)
**Especificación:** Reemplazar el uso de `crate::mcp::run_stdio_server` por `vantadb_mcp::run_stdio_server`.

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
**Especificación:** Agregar `"vantadb-mcp"` a los miembros de `workspace.members`.

---

## 5. Verification Plan

### Automated Tests
1. **Módulo de Rust (MCP):**
   Validar compilación e integración del nuevo crate:
   ```powershell
   cargo check --workspace --all-targets
   cargo test --test mcp_integration
   ```
2. **Adapters de Python:**
   Escribir tests de smoke en Python utilizando `pytest` para certificar las implementaciones en `packages/`:
   ```powershell
   pytest packages/langchain-vantadb/tests/
   pytest packages/llamaindex-vantadb/tests/
   ```

### Manual Verification
1. Instalar de forma editable los nuevos paquetes:
   ```powershell
   pip install -e packages/langchain-vantadb
   pip install -e packages/llamaindex-vantadb
   ```
2. Ejecutar un script de prueba de RAG local simulado que combine los adaptadores para certificar su robustez e interfaz.
