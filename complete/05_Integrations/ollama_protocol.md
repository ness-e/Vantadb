# Ollama & LangChain Integration Protocol
> **Status**: 🟡 In Progress — FASE 3B

## 1. Local-First AI Stack Strategy
IADBMS is designed to operate locally alongside LLMs (e.g., Ollama). Instead of using a traditional TCP connection, we expose a REST API that intercepts, caches, and stores context.

## 2. Ollama Compatibility
By providing an `/api/generate` proxy, clients can point their `OLLAMA_HOST` directly to IADBMS. We perform semantic similarity lookups using the CP-Index on the incoming prompt, append the resulting node data as context, then forward the request to the upstream local Ollama daemon.

## 3. LangChain/VectorStore Interface
To support out-of-the-box ecosystems, we expose standard vector store ingestion endpoints:
- `POST /v1/points` (Inserts embeddings directly as UnifiedNodes)
- `POST /v1/search` (Hybrid Query Execution returning ranked UnifiedNodes)

## 4. Multi-Agent Push Streams
For heavy write workloads (many agents pushing state), we will support WebSockets transmitting bincode-serialized `UnifiedNode` structures to avoid JSON overhead.
