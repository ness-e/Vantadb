# Agnostic Inference Bridge (Puente de Inferencia)

## 1. Misión del Puente
Dotar a IADBMS de la capacidad de comunicarse con Orquestadores de IA (LLMs) nativamente desde Rust por protocolo HTTP, abstrayendo a la base de datos de las particularidades de cada proveedor, pero priorizando fuertemente entornos **Local-First (Ollama)**.

## 2. Abstracción del Cliente (`src/llm/client.rs`)
Para conectividad, usaremos `reqwest` asíncrono.
El cliente tendrá una firma universal estricta:
```rust
async fn generate_embeddings(text: &str, model: &str) -> Result<Vec<f32>, LlmError>
```

## 3. Resolución de APIs (Punto Abierto resuelto)
IADBMS consultará la variable de entorno `OLLAMA_HOST`. Si no existe, caerá por defecto asumiendo que el humano corre Ollama en la misma máquina localmente en `http://localhost:11434`.

## 4. Eficiencia de Conexiones
Para evitar latencia en "Auto-Embeddings" masivos, el puente `LlmClient` utilizará `reqwest::Client` inyectado mediante un pool de conexiones `Arc` (Reference Counting atómico) evitando recrear el Handshake SSL/TCP por cada nuevo bloque de texto a traducir.
