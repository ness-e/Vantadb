#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Benchmark for tokenizer performance comparison.
//!
//! Compares basic ASCII tokenizer vs advanced Tantivy tokenizer
//! with stemming, stopwords removal, and Unicode folding.

#[cfg(feature = "advanced-tokenizer")]
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
#[cfg(feature = "advanced-tokenizer")]
use std::hint::black_box;
#[cfg(feature = "advanced-tokenizer")]
use vantadb::tokenizer::{
    build_advanced_analyzer, tokenize_advanced, tokenize_advanced_default, tokenize_with_analyzer,
    AdvancedTokenizerConfig,
};

#[cfg(feature = "advanced-tokenizer")]
fn bench_advanced_tokenizer_default(c: &mut Criterion) {
    let text = "The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";

    c.bench_function("advanced_tokenizer_default", |b| {
        b.iter(|| tokenize_advanced_default(black_box(text)))
    });
}

#[cfg(feature = "advanced-tokenizer")]
fn bench_advanced_tokenizer_configurations(c: &mut Criterion) {
    let text = "The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";

    let mut group = c.benchmark_group("advanced_tokenizer_configs");

    // Basic (no stemming, no stopwords)
    let config_basic = AdvancedTokenizerConfig {
        apply_stemming: false,
        remove_stopwords: false,
        ..Default::default()
    };
    group.bench_with_input("basic", &config_basic, |b, config| {
        b.iter(|| tokenize_advanced(black_box(text), black_box(config)))
    });

    // Stemming only
    let config_stemming = AdvancedTokenizerConfig {
        apply_stemming: true,
        remove_stopwords: false,
        ..Default::default()
    };
    group.bench_with_input("stemming_only", &config_stemming, |b, config| {
        b.iter(|| tokenize_advanced(black_box(text), black_box(config)))
    });

    // Stopwords only
    let config_stopwords = AdvancedTokenizerConfig {
        apply_stemming: false,
        remove_stopwords: true,
        ..Default::default()
    };
    group.bench_with_input("stopwords_only", &config_stopwords, |b, config| {
        b.iter(|| tokenize_advanced(black_box(text), black_box(config)))
    });

    // Full features
    let config_full = AdvancedTokenizerConfig {
        apply_stemming: true,
        remove_stopwords: true,
        ..Default::default()
    };
    group.bench_with_input("full_features", &config_full, |b, config| {
        b.iter(|| tokenize_advanced(black_box(text), black_box(config)))
    });

    group.finish();
}

#[cfg(feature = "advanced-tokenizer")]
fn bench_tokenizer_text_sizes(c: &mut Criterion) {
    let short_text = "The quick brown fox";
    let medium_text = "The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";
    let long_text = medium_text.repeat(10);

    let mut group = c.benchmark_group("text_sizes");

    group.bench_with_input(
        BenchmarkId::new("advanced", "short"),
        &short_text,
        |b, text| b.iter(|| tokenize_advanced_default(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("advanced", "medium"),
        &medium_text,
        |b, text| b.iter(|| tokenize_advanced_default(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("advanced", "long"),
        &long_text,
        |b, text| b.iter(|| tokenize_advanced_default(black_box(text))),
    );

    group.finish();
}

#[cfg(feature = "advanced-tokenizer")]
fn bench_unicode_handling(c: &mut Criterion) {
    let unicode_text = "Café naïve résumé über München. Café naïve résumé über München. Café naïve résumé über München.";

    c.bench_function("unicode_advanced", |b| {
        b.iter(|| tokenize_advanced_default(black_box(unicode_text)))
    });
}

#[cfg(feature = "advanced-tokenizer")]
fn bench_multilingual(c: &mut Criterion) {
    let english = "The quick brown fox jumps over the lazy dog";
    let spanish = "El perro rápido salta sobre el perro perezoso";
    let french = "Le chien rapide saute par-dessus le chien paresseux";
    let german = "Der schnelle Hund springt über den faulen Hund";

    let mut group = c.benchmark_group("multilingual");

    for (lang, text) in [
        ("english", english),
        ("spanish", spanish),
        ("french", french),
        ("german", german),
    ] {
        group.bench_with_input(BenchmarkId::new("advanced", lang), text, |b, text| {
            b.iter(|| tokenize_advanced_default(black_box(text)))
        });
    }

    group.finish();
}

#[cfg(feature = "advanced-tokenizer")]
fn bench_batch_reuse_vs_fresh(c: &mut Criterion) {
    // ERR-044: a batch of N texts must pay one analyzer build (stemming +
    // stopwords pipeline), not N builds. Builds a realistic batch and compares
    // per-text fresh builds against a single reused analyzer.
    let texts: Vec<String> = (0..100)
        .map(|i| {
            format!(
                "The quick brown fox {} jumps over the lazy dog. Café naïve résumé item {}.",
                i, i
            )
        })
        .collect();
    let config = AdvancedTokenizerConfig::default();

    let mut group = c.benchmark_group("batch_analyzer");

    group.bench_function("fresh_build_per_text", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for text in &texts {
                let tokens = tokenize_advanced(black_box(text), &config);
                total += tokens.len();
            }
            black_box(total)
        });
    });

    group.bench_function("reuse_analyzer", |b| {
        b.iter(|| {
            let mut analyzer = build_advanced_analyzer(&config);
            let mut total = 0usize;
            for text in &texts {
                let tokens = tokenize_with_analyzer(&mut analyzer, black_box(text));
                total += tokens.len();
            }
            black_box(total)
        });
    });

    group.finish();
}

#[cfg(feature = "advanced-tokenizer")]
criterion_group!(
    benches,
    bench_advanced_tokenizer_default,
    bench_advanced_tokenizer_configurations,
    bench_tokenizer_text_sizes,
    bench_unicode_handling,
    bench_multilingual,
    bench_batch_reuse_vs_fresh
);

#[cfg(feature = "advanced-tokenizer")]
criterion_main!(benches);

#[cfg(not(feature = "advanced-tokenizer"))]
fn main() {
    println!("Tokenizer benchmarks require the 'advanced-tokenizer' feature to be enabled.");
    println!("Run with: cargo bench --features advanced-tokenizer");
}
