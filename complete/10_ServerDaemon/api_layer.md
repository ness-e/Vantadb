# Database API Layer & Server Daemon
> **Status**: 🟡 In Progress — FASE 8

## 1. Local-First Daemon
The IADBMS architecture designates a background binary daemon `iadbms-server` which listens on a local port (e.g., `8080`). Agent architectures (such as LangChain or custom Python scripts) interface with this Daemon via REST calls.

## 2. API Design
An `Axum` based web routing system exposes the execution pipeline:
- `POST /api/v1/query`: Accepts raw EBNF string. Parses it via `QueryEngine`, runs physical execution via `Executor`, returns JSON serialized results.
- `PUT /api/v1/nodes`: Direct JSON ingestion mapping to `StorageEngine::put(UnifiedNode)`.

## 3. Asymmetric Scalability
Since this database targets 16GB Single-Node Local environments, Axum is configured tightly for CPU thread affinity running async loops designed to serve local proxies, keeping network overhead strictly below 10ms.
