---
title: Advanced Tokenizer
type: architecture
status: active
tags: [vantadb, architecture]
last_reviewed: 2026-07-21
aliases: []
---

# Advanced Tokenizer

## Overview

The advanced tokenizer provides multilingual text processing with stemming, stopwords removal, and Unicode folding for improved text search quality across multiple languages. It is built on top of [Tantivy](https://github.com/quickwit-oss/tantivy). The feature is **enabled by default**; you only need to opt out if you want the basic tokenizer.

## Features

- **Stemming**: Reduces words to their root form (e.g., "jumping" → "jump", "quickly" → "quick")
- **Stopwords Removal**: Filters out common words that add little semantic value (e.g., "the", "and", "is")
- **Unicode Folding**: Normalizes Unicode characters to ASCII (e.g., "café" → "cafe", "naïve" → "naive")
- **Multilingual Support**: Supports multiple languages with language-specific stemming and stopwords

## Installation

The `advanced-tokenizer` feature is **enabled by default**. You only need to add it explicitly if you disabled default features:

```toml
[dependencies]
vantadb = { version = "0.5", default-features = false, features = ["advanced-tokenizer"] }
```

To opt out and use the basic tokenizer instead, disable the feature:

```toml
[dependencies]
vantadb = { version = "0.5", default-features = false, features = ["cli", "arrow", "fjall", "roaring", "memmap2", "fs2", "sysinfo", "rayon"] }
```

## Usage

### Basic Usage

When the `advanced-tokenizer` feature is enabled, VantaDB automatically uses the advanced tokenizer for all text indexing and search operations:

```rust
use vantadb::VantaEmbedded;

// The advanced tokenizer is automatically used when the feature is enabled
let db = VantaEmbedded::open("./vanta_data").unwrap();
```

### Runtime Configuration

You can configure the advanced tokenizer at runtime using `VantaConfig`:

```rust
use vantadb::{VantaEmbedded, VantaConfig};
use vantadb::tokenizer::{AdvancedTokenizerConfig, Language};

let config = VantaConfig::default()
    .with_advanced_tokenizer_config(Some(AdvancedTokenizerConfig {
        language: Language::Spanish,
        apply_stemming: true,
        remove_stopwords: true,
        ..Default::default()
    }));

let db = VantaEmbedded::open_with_config(config).unwrap();
```

### Advanced Configuration Options

The `AdvancedTokenizerConfig` struct allows you to customize:

- **language**: Language for stemming and stopwords (English, Spanish, French, German, etc.)
- **apply_stemming**: Whether to reduce words to their root form (default: true)
- **remove_stopwords**: Whether to filter out common words (default: true)
- **max_token_length**: Maximum token length in characters (default: 40)

```rust
use vantadb::tokenizer::{AdvancedTokenizerConfig, Language};

// Custom configuration for Spanish text
let config = AdvancedTokenizerConfig {
    language: Language::Spanish,
    apply_stemming: true,
    remove_stopwords: true,
    max_token_length: 50,
};

// Disable stemming but keep stopwords removal
let config = AdvancedTokenizerConfig {
    language: Language::English,
    apply_stemming: false,
    remove_stopwords: true,
    ..Default::default()
};
```

### Programmatic Tokenization

You can also use the tokenizer functions directly for custom text processing:

```rust
use vantadb::tokenizer::{tokenize_advanced, AdvancedTokenizerConfig, Language};

let config = AdvancedTokenizerConfig {
    language: Language::English,
    ..Default::default()
};

// Tokenize with custom configuration
let tokens = tokenize_advanced("The jumping fox", &config);
```

### Configuration

The advanced tokenizer uses sensible defaults:
- **Language**: English
- **Max Token Length**: 40 characters
- **Remove Stopwords**: Enabled
- **Apply Stemming**: Enabled

### Supported Languages

The advanced tokenizer supports the following languages for stemming and stopwords:

- English
- Spanish
- French
- German
- And more (see Tantivy documentation for the full list)

## Schema Version

When the `advanced-tokenizer` feature is enabled, the text index schema version is automatically upgraded to v4. This ensures proper handling of the improved tokenization.

## Performance Considerations

The advanced tokenizer has some performance overhead compared to the basic ASCII tokenizer:
- **Stemming**: Adds measurable overhead to tokenization
- **Stopwords Removal**: Minimal overhead
- **Unicode Folding**: Minimal overhead

The exact cost depends on language and text length. For most use cases, the improved search quality outweighs the performance cost. If you need maximum performance and only work with ASCII text, consider using the basic tokenizer instead.

## Migration

If you have an existing VantaDB database and want to enable the advanced tokenizer:

1. Enable the feature in your `Cargo.toml` (or use the default feature set)
2. The text index will use schema version v4
3. On open, the engine detects the schema version change (v3 → v4) and rebuilds the text index automatically — you don't need to rebuild it manually
4. Builds must be consistent: don't mix databases created with different tokenizer configurations, and keep the feature enabled across all builds

## Comparison with Basic Tokenizer

| Feature | Basic Tokenizer | Advanced Tokenizer |
|---------|----------------|-------------------|
| Character Set | ASCII only | Unicode with folding |
| Stemming | No | Yes |
| Stopwords Removal | No | Yes |
| Multilingual | Limited | Yes |
| Performance | Fastest | Slightly slower |
| Schema Version | v3 | v4 |

## Examples

### English Text

```rust
let text = "The jumping fox runs quickly";
// Basic tokenizer: ["the", "jumping", "fox", "runs", "quickly"]
// Advanced tokenizer: ["jump", "fox", "run", "quickli"] (stemmed, stopwords removed)
```

### Spanish Text

```rust
let text = "El perro rapido salta sobre el perro perezoso";
// Advanced tokenizer (Spanish): ["perro", "rapid", "salt", "perro", "perezos"]
// Stopwords like "el", "sobre" are removed
```

### Unicode Text

```rust
let text = "Café naïve résumé";
// Basic tokenizer: May not handle Unicode correctly
// Advanced tokenizer: ["cafe", "naiv", "resum"] (Unicode folded)
```

## Troubleshooting

### Schema version mismatch

If you see schema version errors, ensure that:
1. The `advanced-tokenizer` feature is consistently enabled across all builds
2. You're not mixing databases created with different tokenizer configurations

The text index rebuilds automatically when the schema version changes (v3 ↔ v4), so a one-time version mismatch is expected and handled on open.

## Future Enhancements

Potential future improvements:
- Custom stemming rules
- Language detection
- Performance optimizations
- Additional language support

## References

- [Tantivy Documentation](https://docs.rs/tantivy/)
- [[bm25|BM25 Algorithm]]
- [Stemming Algorithms](https://en.wikipedia.org/wiki/Stemming)
