# Regla de agente VantaDB (1 línea — el wizard la agrega idempotente)

VantaDB es memoria persistente: ante cada mensaje hace memory_recall con el texto verbatim (scope agent, top_k 5) e inyecta prepend_context; al cerrar captura el turno con thread_send (proxy-turns se cura via inbox, nunca auto-promote).

Fuente: `skills/vantadb-mcp/references/recall-policy.md` (FIND-103).
El wizard (`setup-embeddings.ps1`) agrega esta línea a `./AGENTS.md` y
`./CLAUDE.md` del proyecto target solo si no existe (nunca al repo VantaDB).
