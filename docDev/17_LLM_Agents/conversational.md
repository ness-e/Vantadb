# Nodos de Chat Nativos (Conversational Primitives)

## 1. El Problema Actual de los Chats en Agentes
Guardar el historial de chat (Memoria a corto/largo plazo) suele ser caótico. LangChain requiere bases de datos aparte. Aquí lo integramos.

## 2. El tipo `MessageThread`
Añadimos un macro estructurado dentro de los utilitarios de `src/types/chat.rs`.
Un nodo de tipo `Message` será un `UnifiedNode` normal que obligatoriamente contenga:
{ "role": "system" | "user" | "assistant", "content": "Hola mundo" }

## 3. IQL Azucarado (Syntactic Sugar)
Permitimos azúcar sintáctico en IQL para insertar chats sin escribir toda la sintaxis JSON pesada:
`INSERT MESSAGE USER "Hola mundo" TO THREAD#5`
En el fondo, el parser lo digerirá a un IQL nativo:
`INSERT NODE#... TYPE Message { role: "user", content: "Hola mundo" }` y forzará una arista `RELATE MSG --"belongs_to_thread"--> THREAD#5`.

## 4. Retención Circular (Rolling Window Context)
Se habilitará mediante el `GcWorker` (Fase 13). Si una cuenta pasa los N mensajes, el GC aplicará *Soft Delete* o archivará los más viejos para dejar comprimido únicamente el contexto límite que cabría dentro del token window del LLM.
