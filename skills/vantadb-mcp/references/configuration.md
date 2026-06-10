# VantaDB Configuration Guide

## Core Configuration

### Memory Limits

Configure memory limits based on your workload:

```json
{
  "memory_limit_bytes": 512000000
}
```

**Recommendations:**
- Small workloads: 256MB
- Medium workloads: 512MB
- Large workloads: 1GB+
- Production: Monitor and adjust based on usage

### Thread Pool

Configure blocking thread pool for concurrent operations:

```json
{
  "max_blocking_threads": 4
}
```

**Recommendations:**
- Low concurrency: 2-4 threads
- Medium concurrency: 4-8 threads
- High concurrency: 8-16 threads
- CPU-bound workloads: Match physical cores

### Read-Only Mode

Enable read-only mode for production deployments:

```json
{
  "read_only": true
}
```

**Use cases:**
- Production read replicas
- Analytics workloads
- Safety-critical deployments

## HNSW Configuration

### Dimensionality

```json
{
  "hnsw": {
    "dim": 1536
  }
}
```

**Common dimensions:**
- OpenAI ada-002: 1536
- OpenAI text-embedding-3-small: 1536
- Cohere embed-v3: 1024
- Custom embeddings: Match your model

### M Parameter (Connections)

```json
{
  "hnsw": {
    "m": 16
  }
}
```

**Trade-offs:**
- Lower m: Faster build, less memory, lower recall
- Higher m: Slower build, more memory, higher recall

**Recommendations:**
- Small datasets (<10K): 12-16
- Medium datasets (10K-100K): 16-24
- Large datasets (>100K): 24-32

### EF Construction

```json
{
  "hnsw": {
    "ef_construction": 200
  }
}
```

**Trade-offs:**
- Lower ef: Faster build, lower recall
- Higher ef: Slower build, higher recall

**Recommendations:**
- Quick prototyping: 100-150
- Production: 200-400
- High recall: 400-800

### EF Search

```json
{
  "hnsw": {
    "ef_search": 50
  }
}
```

**Trade-offs:**
- Lower ef: Faster search, lower recall
- Higher ef: Slower search, higher recall

**Recommendations:**
- Fast search: 10-25
- Balanced: 50-100
- High recall: 100-200

## Advanced Configuration

### Storage Backend

```json
{
  "backend": {
    "type": "fjall",
    "path": "./vantadb"
  }
}
```

### WAL Configuration

```json
{
  "wal": {
    "enabled": true,
    "sync_mode": "fsync",
    "max_size_bytes": 104857600
  }
}
```

**Sync modes:**
- `none`: No sync (fastest, least safe)
- `fsync`: Full fsync (safe, slower)
- `datasync`: fdatasync (balanced)

### Text Index Configuration

```json
{
  "text_index": {
    "tokenizer": "default",
    "stemming": true,
    "stopwords": true
  }
}
```

### Metrics Configuration

```json
{
  "metrics": {
    "enable_hnsw_stats": true,
    "enable_storage_stats": true,
    "enable_query_stats": true
  }
}
```

## Environment Variables

### VANTADB_PATH

Override storage path:

```bash
export VANTADB_PATH=/custom/path
```

### VANTADB_MEMORY_LIMIT

Override memory limit:

```bash
export VANTADB_MEMORY_LIMIT=1073741824
```

### VANTADB_READ_ONLY

Enable read-only mode:

```bash
export VANTADB_READ_ONLY=1
```

## Namespace Configuration

### Default Namespace

```json
{
  "default_namespace": "default"
}
```

### Namespace Isolation

Configure separate namespaces for different contexts:

```json
{
  "namespaces": {
    "agent": {
      "prefix": "agent/",
      "auto_create": true
    },
    "session": {
      "prefix": "session/",
      "auto_create": true
    }
  }
}
```

## Performance Tuning

### Benchmarking

Test different configurations:

```python
import vantadb_py as vantadb

# Test with different HNSW parameters
configs = [
    {"m": 16, "ef_construction": 200, "ef_search": 50},
    {"m": 24, "ef_construction": 300, "ef_search": 100},
    {"m": 32, "ef_construction": 400, "ef_search": 150}
]

for config in configs:
    db = vantadb.VantaDB("./test", hnsw_config=config)
    # Run benchmarks
    db.close()
```

### Monitoring

Monitor operational metrics:

```python
metrics = db.operational_metrics()
print(f"HNSW nodes: {metrics['hnsw_nodes_count']}")
print(f"Memory usage: {metrics['memory_usage_bytes']}")
```

## Security Configuration

### Access Control

```json
{
  "access_control": {
    "enabled": true,
    "default_policy": "read-only",
    "namespaces": {
      "admin": {
        "policy": "read-write"
      }
    }
  }
}
```

### Encryption

```json
{
  "encryption": {
    "enabled": true,
    "algorithm": "aes-256-gcm",
    "key_path": "/path/to/key"
  }
}
```

## Migration Configuration

### Import/Export

```json
{
  "migration": {
    "import_path": "./import",
    "export_path": "./export",
    "batch_size": 1000
  }
}
```

## Troubleshooting

### High Memory Usage

Reduce memory limit or:
- Decrease HNSW m parameter
- Reduce ef_construction
- Implement periodic cleanup

### Slow Search

Increase ef_search or:
- Rebuild index with higher ef_construction
- Reduce dataset size
- Use metadata filters

### Low Recall

Increase HNSW parameters:
- Increase m
- Increase ef_construction
- Increase ef_search

## Best Practices

1. **Start with defaults** - Use default configuration initially
2. **Benchmark before tuning** - Measure performance before changing parameters
3. **Tune incrementally** - Change one parameter at a time
4. **Monitor metrics** - Track operational metrics continuously
5. **Test with real data** - Use production-like data for configuration testing
6. **Document changes** - Keep track of configuration changes and their impact
