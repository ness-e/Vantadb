//! DRV-130 T3: VantaFile-backed search benchmark.
//!
//! Measures search_nearest with a populated VantaFile (SSD-I/O emulation)
//! vs in-memory search (no I/O). Includes a compacted variant that rewrites
//! the VantaFile in BFS order to test locality improvement.
//!
//! Run with:
//!   RUST_LOG=debug cargo bench --bench vfile_search 2>&1 | grep PROFILE
//!
//! The debug log shows SearchProfile counters: vfile_reads, unique_pages,
//! compute_ns, candidates_seen — to quantify the I/O vs compute breakdown.

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::path::PathBuf;
use tempfile::tempdir;
use vantadb::index::VectorRepresentations;
use vantadb::index::{CPIndex, HnswConfig, IndexType};
use vantadb::node::{DiskNodeHeader, DistanceMetric, FilterBitset};
use vantadb::storage::archive::{compact_layout, reindex_nodes, traverse_graph};
use vantadb::storage::vfile::VantaFile;

const DIMS: usize = 128;
const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 200;
const TOP_K: usize = 10;
const SEED: u64 = 42;

fn generate_vectors(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut vec: Vec<f32> = (0..dims).map(|_| rng.random_range(-1.0..1.0)).collect();
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vectors.push(vec);
    }
    vectors
}

fn populate_vfile(vstore: &mut VantaFile, vectors: &[Vec<f32>]) {
    let hdr_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    let align: u64 = 64;
    let mut offset = vstore.write_cursor;
    for (id, vec) in vectors.iter().enumerate() {
        let vec_offset = offset + hdr_size;
        let mut header = DiskNodeHeader::new(id as u128);
        header.vector_len = vec.len() as u32;
        header.vector_offset = vec_offset;
        vstore.write_header(offset, &header).unwrap();
        let bytes: Vec<u8> = vec.iter().flat_map(|v| v.to_le_bytes()).collect();
        vstore.mmap_bytes_mut().unwrap()[vec_offset as usize..][..bytes.len()]
            .copy_from_slice(&bytes);
        let node_size = hdr_size + (vec.len() as u64 * 4);
        offset = ((offset + node_size + align - 1) / align) * align;
    }
    vstore.write_cursor = offset;
}

fn build_index(vectors: &[Vec<f32>]) -> CPIndex {
    let config = HnswConfig {
        m: 32,
        m_max0: 64,
        ef_construction: 400,
        ef_search: 100,
        ml: 1.0 / (32_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: IndexType::Hnsw,
    };
    let index = CPIndex::new_with_config(config);
    let align: u64 = 64;
    let hdr_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    let mut offset = align;
    for (id, vec) in vectors.iter().enumerate() {
        let rep = VectorRepresentations::Full(vec.clone());
        index.add(id as u128, FilterBitset::all_set(), rep, offset);
        let node_size = hdr_size + (vec.len() as u64 * 4);
        offset = ((offset + node_size + align - 1) / align) * align;
    }
    index
}

fn search_in_memory(index: &CPIndex, queries: &[Vec<f32>]) {
    for q in queries {
        black_box(index.search_nearest(q, None, None, &FilterBitset::all_set(), TOP_K, None));
    }
}

fn search_with_vfile(index: &CPIndex, queries: &[Vec<f32>], vfile: &VantaFile) {
    for q in queries {
        black_box(index.search_nearest(
            q,
            None,
            None,
            &FilterBitset::all_set(),
            TOP_K,
            Some(vfile),
        ));
    }
}

fn build_compacted(vectors: &[Vec<f32>]) -> (CPIndex, VantaFile) {
    let dir = tempdir().expect("tempdir");
    let vfile_path: PathBuf = [dir.path().to_str().unwrap(), "vstore.vanta"]
        .iter()
        .collect();
    let index = build_index(vectors);

    let hdr_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    let total_size = 64 + vectors.len() as u64 * (hdr_size + 128 * 4 + 64);
    let mut vfile = VantaFile::open(vfile_path, total_size).expect("VantaFile::open");
    populate_vfile(&mut vfile, vectors);

    let ep = index.get_entry_point().expect("entry point");
    let bfs_order = traverse_graph(&index, ep);
    let (offset_map, _new_size) =
        compact_layout(&mut vfile, &index, &bfs_order, hdr_size).expect("compact_layout");
    reindex_nodes(&index, &offset_map);

    (index, vfile)
}

fn bench_vfile_search(c: &mut Criterion) {
    let vectors = generate_vectors(N_VECTORS, DIMS, SEED);
    let queries = generate_vectors(N_QUERIES, DIMS, SEED + 1);

    let index = build_index(&vectors);
    let mut vfile = VantaFile::create_in_memory(16 * 1024 * 1024);
    populate_vfile(&mut vfile, &vectors);

    let (compacted_index, compacted_vfile) = build_compacted(&vectors);

    let mut group = c.benchmark_group("vfile_search");
    group.throughput(criterion::Throughput::Elements(N_QUERIES as u64));
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(5));
    group.warm_up_time(std::time::Duration::from_secs(2));

    group.bench_function("in_memory", |b| {
        b.iter(|| search_in_memory(&index, black_box(&queries)));
    });

    group.bench_function("with_vfile", |b| {
        b.iter(|| search_with_vfile(&index, black_box(&queries), &vfile));
    });

    group.bench_function("with_vfile_compacted", |b| {
        b.iter(|| search_with_vfile(&compacted_index, black_box(&queries), &compacted_vfile));
    });

    group.finish();
}

fn bench_setup_overhead(c: &mut Criterion) {
    let vectors = generate_vectors(N_VECTORS, DIMS, SEED);
    c.bench_function("populate_vfile", |b| {
        b.iter(|| {
            let mut vf = VantaFile::create_in_memory(16 * 1024 * 1024);
            populate_vfile(&mut vf, black_box(&vectors));
        });
    });
    c.bench_function("build_index", |b| {
        b.iter(|| black_box(build_index(black_box(&vectors))));
    });
}

criterion_group!(benches, bench_vfile_search, bench_setup_overhead);
criterion_main!(benches);
