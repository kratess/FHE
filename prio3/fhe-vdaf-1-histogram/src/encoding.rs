use crate::config::BUCKETS;

pub fn encode_bucket(bucket: usize) -> Vec<usize> {
    assert!(
        bucket < BUCKETS,
        "bucket ({bucket}) must be < BUCKETS ({BUCKETS})"
    );
    let mut slots = vec![0usize; BUCKETS];
    slots[bucket] = 1;
    slots
}

pub fn decode_histogram(slots: &[usize]) -> Vec<usize> {
    slots.to_vec()
}
