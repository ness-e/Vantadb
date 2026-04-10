use std::collections::BinaryHeap;

#[derive(Clone, PartialEq, Debug)]
struct NodeSimMin(f32, u64);

impl Eq for NodeSimMin {}

impl PartialOrd for NodeSimMin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for NodeSimMin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.0.partial_cmp(&self.0).unwrap_or(std::cmp::Ordering::Equal) {
            std::cmp::Ordering::Equal => self.1.cmp(&other.1),
            cmp => cmp,
        }
    }
}

fn main() {
    let mut heap = BinaryHeap::new();
    heap.push(NodeSimMin(0.5, 1)); // Worst
    heap.push(NodeSimMin(0.9, 2)); // Best
    heap.push(NodeSimMin(0.7, 3)); // Middle

    let popped = heap.peek().unwrap().clone();
    println!("Popped: {:?}", popped); // Should be 0.5 (Greatest according to Ord)

    let sorted = heap.into_sorted_vec();
    println!("Sorted array: {:?}", sorted); 
    // If it prints [0.9, 0.7, 0.5], it means best is at index 0!
}
