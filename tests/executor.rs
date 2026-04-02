use iadbms::index::{CPIndex, cosine_similarity};

#[test]
fn test_cosine_math() {
    let vec_a = vec![1.0, 0.0, 0.0];
    let vec_b = vec![1.0, 0.0, 0.0];
    let vec_c = vec![0.0, 1.0, 0.0];

    assert!((cosine_similarity(&vec_a, &vec_b) - 1.0).abs() < f32::EPSILON);
    assert!((cosine_similarity(&vec_a, &vec_c) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_idx_search() {
    let mut idx = CPIndex::new();
    // Match mask + High sim
    idx.add(1, 0b11, Some(vec![1.0, 0.0]));
    // Match mask + Low sim
    idx.add(2, 0b11, Some(vec![0.0, 1.0]));
    // Fails mask
    idx.add(3, 0b00, Some(vec![1.0, 0.0]));

    let res = idx.search_nearest(&[1.0, 0.0], 0b10, 2);
    // Should get node 1 and 2, but 3 is ignored via bitset
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].0, 1);
    assert_eq!(res[1].0, 2);
}
