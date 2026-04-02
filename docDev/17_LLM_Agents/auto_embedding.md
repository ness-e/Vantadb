# Auto-Embedding (Generación Diferida Opt-In)

## 1. El Concepto Funcional
Facilitar a los orquestadores el guardado en la base de datos eximiéndolos de calcular los vectores manualmente. El Orquestador manda texto a IADBMS, e IADBMS se voltea y los calcula interactuando transparente con el LLM.

## 2. Detección en el AST (`src/executor.rs`)
La condición para detonar un auto-embedding será estricta y predecible:
1. El IQL debe ser un mutador `INSERT` o `UPDATE`.
2. El Parser detecta que **NO** hay palabra reservada `VECTOR` en el comando.
3. El comando posee un campo nominado, por ejemplo: `{ texto: "..." }`.

## 3. Flujo Lógico de Seguridad
```rust
if statement.vector.is_none() && config.auto_embedding_enabled {
    let raw_text = statement.fields.get("texto").map(|v| v.as_str());
    if let Some(text) = raw_text { // Detonar LLM Request!
        let vec = llm_client.generate_embeddings(text).await?;
        statement.vector = Some(vec);
    }
}
storage.insert(statement);
```

## 4. Eficiencia
El puente se ejecutará concurrente sin congelar peticiones. Su única dependencia técnica externa es asegurar que el "texto objetivo" a convertir tenga una clave nombrada estandarizada o un parámetro dinámico extra para que IADBMS sepa qué propiedad es la que debe pasar por embebido. (Sugiero flag global de configuración para mapear `texto` por defecto).
