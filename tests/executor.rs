use connectomedb::index::{CPIndex, VectorRepresentations};

#[test]
fn test_cosine_math() {
    let vec_a = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
    let vec_b = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
    let vec_c = VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);

    assert!((vec_a.cosine_similarity(&vec_b).unwrap() - 1.0).abs() < f32::EPSILON);
    assert!((vec_a.cosine_similarity(&vec_c).unwrap() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_idx_search() {
    let mut idx = CPIndex::new();
    // Match mask + High sim
    idx.add(1, 0b11, VectorRepresentations::Full(vec![1.0, 0.0]));
    // Match mask + Low sim
    idx.add(2, 0b11, VectorRepresentations::Full(vec![0.0, 1.0]));
    // Fails mask
    idx.add(3, 0b00, VectorRepresentations::Full(vec![1.0, 0.0]));

    let res = idx.search_nearest(&[1.0, 0.0], None, None, 0b10, 2);
    // Should get node 1 and 2, but 3 is ignored via bitset
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].0, 1);
    assert_eq!(res[1].0, 2);
}
