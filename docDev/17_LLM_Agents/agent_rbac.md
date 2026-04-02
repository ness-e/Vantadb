# Particionamiento por Roles (Agent RBAC)

## 1. Misión
Garantizar la privacidad inter-agente. Un sistema verdaderamente multi-agente hospeda memoria de 10 bots, y no queremos que las memorias del 'Agente Contable' sean recuperadas por la similitud vectorial de una búsqueda del 'Agente Cita Médica'.

## 2. Inyección de Sufijo Hash (Diseño a nivel Storage)
Aprovechar nuestro motor RocksDB en `src/storage.rs`:
Cuando un nodo tiene un Rol (ej. "Admin"), la llave primaria (Primary Key `u64`) en byte se pre-fijará con el Hash del Rol para las colecciones o se almacenará el Rol como un índice secundario físico estricto.
*Decisión:* Para MVP Multi-Agente, introduciremos una columna virtual en el `UnifiedNode` llamada `owner_role`.

## 3. Extensión del IQL Parser
```ebnf
query = "FROM" target_node ["WHERE" conditions] ["ROLE" string_literal]
```
Si el query viene con `ROLE "X"`, el Executor aplicará un filtro estricto antes de iniciar el cálculo de HNSW o el escaneo lineal de BFS, podando la rama completa del grafo si `node.owner_role != "X"`. 
Si no provee `ROLE` asume rol `admin` / Superusuario.
