use crate::config::{LOW_BITS, LOW_BITS_MAX_VALUE, MAX_BITS, MAX_SIZE, TOP_BIT_WEIGHT};

pub fn encode_value(value: usize) -> Vec<usize> {
    assert!(
        value <= MAX_SIZE,
        "value ({value}) must be <= MAX_SIZE ({MAX_SIZE})"
    );

    let mut slots = vec![0usize; MAX_BITS];

    let low_value = if value > LOW_BITS_MAX_VALUE {
        slots[MAX_BITS - 1] = 1;
        value - TOP_BIT_WEIGHT
    } else {
        value
    };

    for (i, slot) in slots.iter_mut().take(LOW_BITS).enumerate() {
        *slot = (low_value >> i) & 1;
    }

    slots
}

pub fn decode_value(slots: &[usize]) -> usize {
    let low_value = slots
        .iter()
        .take(LOW_BITS)
        .enumerate()
        .map(|(i, slot)| slot << i)
        .sum::<usize>();

    low_value + slots[MAX_BITS - 1] * TOP_BIT_WEIGHT
}
