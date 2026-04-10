# IQL Mutation Syntax — Especificación Técnica

> **Fase 14 · Carpeta 16_IQL_Mutations · Semana 31-33**

Este documento define la extensión del lenguaje de consultas IQL para soportar
operaciones de **escritura** (INSERT, UPDATE, DELETE, RELATE) además de las
operaciones de lectura ya implementadas (FROM...SIGUE...WHERE...FETCH).

---

## 1. Problema Actual

El parser actual (`src/parser.rs`) solo soporta **lectura**. Toda mutación de 
datos (crear nodos, editar campos, borrar, crear aristas) se hace exclusivamente
por API programática (Rust directo o Python SDK vía PyO3). Esto significa que:

- El CLI (`connectomedb-cli`) no puede insertar datos
- El servidor REST no puede recibir comandos de escritura por query string
- Los agentes de IA no pueden mutar el grafo mediante lenguaje natural traducido a IQL

### Fallas Específicas a Resolver

| # | Falla | Impacto |
|---|-------|---------|
| 1 | No existe `INSERT` en IQL | No se pueden crear nodos por consulta |
| 2 | No existe `UPDATE` en IQL | No se pueden modificar campos por consulta |
| 3 | No existe `DELETE` en IQL | No se pueden eliminar nodos por consulta |
| 4 | No existe `RELATE` en IQL | No se pueden crear aristas/edges por consulta |
| 5 | Ambigüedad en alias de `SIGUE` | `Persona` no queda claro si es tipo o alias |

---

## 2. Sintaxis Propuesta

### 2.1 INSERT — Crear Nodos
```sql
INSERT NODE#<id> TYPE <tipo> { <campo>: <valor>, ... } [VECTOR [<f32>, ...]]
```

**Ejemplos:**
```sql
-- Nodo con datos relacionales solamente
INSERT NODE#101 TYPE Usuario { nombre: "Eros", pais: "VE", edad: 28 }

-- Nodo con datos relacionales + vector de embedding
INSERT NODE#102 TYPE Documento { titulo: "Manual ConnectomeDB" } VECTOR [0.12, -0.45, 0.99, 0.33]

-- Nodo vectorial puro (sin campos relacionales)
INSERT NODE#103 TYPE Embedding {} VECTOR [0.1, 0.4, 0.9]
```

**Mapeo interno al AST:**
```rust
pub enum Statement {
    Query(Query),           // ← ya existe (FROM...SIGUE...WHERE...)
    Insert(InsertStatement),  // ← NUEVO
    Update(UpdateStatement),  // ← NUEVO
    Delete(DeleteStatement),  // ← NUEVO
    Relate(RelateStatement),  // ← NUEVO
}

pub struct InsertStatement {
    pub node_id: u64,
    pub node_type: String,
    pub fields: BTreeMap<String, FieldValue>,
    pub vector: Option<Vec<f32>>,
}
```

### 2.2 UPDATE — Modificar Campos
```sql
UPDATE NODE#<id> SET <campo> = <valor> [, <campo> = <valor>, ...]
```

**Ejemplos:**
```sql
-- Actualizar un campo
UPDATE NODE#101 SET nombre = "Eros Dev"

-- Actualizar múltiples campos
UPDATE NODE#101 SET nombre = "Eros Dev", pais = "US", edad = 29

-- Reemplazar vector completo
UPDATE NODE#102 SET VECTOR [0.55, -0.22, 0.88, 0.11]
```

**Mapeo interno:**
```rust
pub struct UpdateStatement {
    pub node_id: u64,
    pub fields: BTreeMap<String, FieldValue>,
    pub vector: Option<Vec<f32>>,
}
```

### 2.3 DELETE — Eliminar Nodos
```sql
DELETE NODE#<id>
```

**Ejemplos:**
```sql
-- Eliminar un nodo y todas sus aristas asociadas
DELETE NODE#101
```

**Comportamiento:** Al eliminar un nodo, el motor DEBE:
1. Borrar el nodo de RocksDB
2. Borrar todas las aristas que apunten a ese nodo (limpieza de grafo)
3. Remover del índice HNSW si tenía vector
4. Registrar en el WAL como `WalRecord::Delete { id }`

**Mapeo interno:**
```rust
pub struct DeleteStatement {
    pub node_id: u64,
}
```

### 2.4 RELATE — Crear Aristas entre Nodos
```sql
RELATE NODE#<origen> --"<etiqueta>"--> NODE#<destino> [WEIGHT <f32>]
```

**Ejemplos:**
```sql
-- Crear relación simple
RELATE NODE#101 --"amigo"--> NODE#45

-- Crear relación con peso (para PageRank, recomendaciones, etc.)
RELATE NODE#101 --"compro_junto_con"--> NODE#200 WEIGHT 0.95

-- Crear relación bidireccional (ejecuta dos statements)
RELATE NODE#101 --"colega"--> NODE#45 WEIGHT 1.0
RELATE NODE#45 --"colega"--> NODE#101 WEIGHT 1.0
```

**Mapeo interno:**
```rust
pub struct RelateStatement {
    pub source_id: u64,
    pub target_id: u64,
    pub label: String,
    pub weight: Option<f32>,
}
```

### 2.5 FIX — Desambiguación de Alias en SIGUE
**Antes (ambiguo):**
```sql
SIGUE 1..3 "amigo" Persona
-- ¿"Persona" es un tipo de nodo para filtrar o un alias para referirse al resultado?
```

**Después (explícito):**
```sql
SIGUE 1..3 "amigo" AS Persona           -- Alias explícito
SIGUE 1..3 "amigo" TYPE Usuario AS p    -- Filtro por tipo + alias
```

**Cambio en el AST de Traversal:**
```rust
// ANTES:
pub struct Traversal {
    pub min_depth: u32,
    pub max_depth: u32,
    pub edge_label: String,
}

// DESPUÉS:
pub struct Traversal {
    pub min_depth: u32,
    pub max_depth: u32,
    pub edge_label: String,
    pub target_type: Option<String>,  // NUEVO: TYPE filtro
    pub alias: Option<String>,        // NUEVO: AS alias
}
```

---

## 3. Archivos a Modificar

### Parser (Nom Combinators)
| Archivo | Cambio |
|---------|--------|
| `src/query.rs` | Agregar `Statement` enum, `InsertStatement`, `UpdateStatement`, `DeleteStatement`, `RelateStatement` structs. Extender `Traversal` con `target_type` y `alias`. |
| `src/parser.rs` | Agregar funciones: `parse_insert()`, `parse_update()`, `parse_delete()`, `parse_relate()`, `parse_statement()` (dispatcher). Modificar `parse_traversal()` para soportar `AS` y `TYPE`. |

### Ejecución
| Archivo | Cambio |
|---------|--------|
| `src/executor.rs` | Agregar rama de ejecución para cada `Statement` que traduzca a operaciones de `StorageEngine` |
| `src/server.rs` | El endpoint `/api/v1/query` debe aceptar tanto queries de lectura como de escritura |
| `src/bin/connectomedb-cli.rs` | El REPL debe detectar si el input es mutación o lectura |

### Tests
| Archivo | Cambio |
|---------|--------|
| `tests/parser.rs` | Tests para cada nueva sentencia (INSERT, UPDATE, DELETE, RELATE) |
| `tests/mutations.rs` | **[NUEVO]** Test de integración: parsear → ejecutar → verificar en storage |

---

## 4. Compatibilidad con Parser Existente

El parser actual retorna un `Query`. Para no romper la API actual:

```rust
// El dispatcher público:
pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        map(parse_insert, Statement::Insert),
        map(parse_update, Statement::Update),
        map(parse_delete, Statement::Delete),
        map(parse_relate, Statement::Relate),
        map(parse_query, Statement::Query),  // ← fallback al parser original
    ))(input)
}

// parse_query() sigue existiendo sin cambios para retrocompatibilidad
```

---

## 5. Métricas de Éxito

| Métrica | Objetivo |
|---------|----------|
| Parse INSERT | < 1ms para statement con 10 campos + vector 128-dim |
| Parse batch | > 1k statements/sec |
| Roundtrip INSERT→GET | < 5ms (parse + store + retrieve) |
| Tests cubriendo mutations | 100% de las 4 sentencias nuevas |
| Backward-compatible | `parse_query()` sigue funcionando sin cambios |

---

## 6. EBNF Formal (Referencia)

```ebnf
statement     = insert | update | delete | relate | query ;

insert        = "INSERT" , "NODE#" , number , "TYPE" , ident ,
                "{" , field_list , "}" , [ "VECTOR" , vector_lit ] ;

update        = "UPDATE" , "NODE#" , number , "SET" ,
                ( field_assign { "," , field_assign } | "VECTOR" , vector_lit ) ;

delete        = "DELETE" , "NODE#" , number ;

relate        = "RELATE" , "NODE#" , number , "--\"" , ident , "\"-->" ,
                "NODE#" , number , [ "WEIGHT" , float ] ;

query         = "FROM" , ident , [ traversal ] , [ where_clause ] ,
                [ fetch ] , [ rank_by ] , [ temperature ] ;

traversal     = "SIGUE" , number , ".." , number , string ,
                [ "TYPE" , ident ] , [ "AS" , ident ] ;

field_list    = field_assign { "," , field_assign } ;
field_assign  = ident , ":" , value ;
value         = string | number | float | "true" | "false" ;
vector_lit    = "[" , float , { "," , float } , "]" ;
```
