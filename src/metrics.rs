use prometheus::{Counter, Histogram, Registry, IntCounter};
use std::sync::LazyLock;

// Ensure singleton metrics registry across the binary
pub static METRICS_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

pub static QUERY_LATENCY: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(
        prometheus::HistogramOpts::new("connectome_query_latency_ms", "Query execution times in ms")
    ).unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static OOM_TRIPS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new("connectome_oom_circuit_trips_total", "Governor OOM prevents").unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static CACHE_HITS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new("connectome_cache_hits_total", "CP-Index fast path matches").unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

/// Export utility suitable for the `/metrics` Axum endpoint
pub fn export_metrics_text() -> String {
    use prometheus::TextEncoder;
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = String::new();
    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();
    buffer
}
