use crate::config::{
    AGGREGATORS_NUM, LOW_BITS, LOW_BITS_MAX_VALUE, MAX_BITS, MAX_PER_AGGREGATORS_UNITS, MAX_SCALED,
    MAX_SIZE, RANDOM_BITS_LEN, RNG_SEED, TOP_BIT_WEIGHT,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn split_value(value: usize) -> [usize; AGGREGATORS_NUM] {
    assert!(
        value <= MAX_SIZE,
        "value ({value}) must be <= MAX_SIZE ({MAX_SIZE})"
    );
    let value_scaled = value * AGGREGATORS_NUM;
    assert!(
        value_scaled <= MAX_SCALED,
        "value_scaled ({value_scaled}) must be <= MAX_SCALED ({MAX_SCALED})"
    );
    let mut remaining = value_scaled;
    let mut rng = StdRng::seed_from_u64(RNG_SEED as u64);
    let mut result = [0usize; AGGREGATORS_NUM];

    for i in (0..AGGREGATORS_NUM).rev() {
        let cap_remaining = i * MAX_PER_AGGREGATORS_UNITS;
        let min_to_send = remaining.saturating_sub(cap_remaining);
        let max_to_send = remaining.min(MAX_PER_AGGREGATORS_UNITS);
        let number = rng.gen_range(min_to_send..=max_to_send);
        result[i] = number;
        remaining -= number;
    }

    assert!(remaining == 0, "remaining ({remaining}) must be 0");
    result
}

pub fn random_bits(seed: u64) -> [usize; RANDOM_BITS_LEN] {
    let mut rng = StdRng::seed_from_u64(seed);
    std::array::from_fn(|_| rng.gen_range(0..=1))
}

pub fn signature_bits(value: usize) -> [usize; RANDOM_BITS_LEN] {
    let seed = (RNG_SEED as u64) ^ ((value as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    random_bits(seed)
}

pub fn to_shard(value_units: usize) -> Vec<usize> {
    assert!(
        value_units <= MAX_PER_AGGREGATORS_UNITS,
        "value_units ({value_units}) must be <= MAX_PER_AGGREGATORS_UNITS ({MAX_PER_AGGREGATORS_UNITS})"
    );

    let mut shards = vec![0usize; MAX_BITS];

    let low_value = if value_units > LOW_BITS_MAX_VALUE {
        shards[MAX_BITS - 1] = 1;
        value_units - TOP_BIT_WEIGHT
    } else {
        value_units
    };

    for (i, slot) in shards.iter_mut().take(LOW_BITS).enumerate() {
        *slot = (low_value >> i) & 1;
    }

    shards
}

pub fn decode_value_units(slots: &[usize]) -> usize {
    let low_value = slots
        .iter()
        .take(LOW_BITS)
        .enumerate()
        .map(|(i, slot)| slot << i)
        .sum::<usize>();

    low_value + slots[MAX_BITS - 1] * TOP_BIT_WEIGHT
}

pub fn rescale_units(value_units: usize) -> usize {
    assert_eq!(
        value_units % AGGREGATORS_NUM,
        0,
        "aggregated units ({value_units}) must divide evenly by AGGREGATORS_NUM ({AGGREGATORS_NUM})"
    );
    value_units / AGGREGATORS_NUM
}
