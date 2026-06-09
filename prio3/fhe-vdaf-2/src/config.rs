use openfhe_bgv_rs::BgvParams;

pub const MAX_SIZE: usize = 100;
pub const AGGREGATORS_NUM: usize = 2;
pub const MAX_SCALED: usize = MAX_SIZE * AGGREGATORS_NUM;
pub const MAX_PER_AGGREGATORS_UNITS: usize = MAX_SIZE;
pub const RNG_SEED: usize = 42;
pub const RANDOM_BITS_LEN: usize = 64;
pub const MAX_BITS: usize = if MAX_PER_AGGREGATORS_UNITS == 0 {
    1
} else {
    (usize::BITS - MAX_PER_AGGREGATORS_UNITS.leading_zeros()) as usize
};
pub const LOW_BITS: usize = MAX_BITS.saturating_sub(1);
pub const LOW_BITS_MAX_VALUE: usize = if LOW_BITS == 0 {
    0
} else {
    (1usize << LOW_BITS) - 1
};
pub const TOP_BIT_WEIGHT: usize = MAX_PER_AGGREGATORS_UNITS - LOW_BITS_MAX_VALUE;
pub const SIGNATURE_START_SLOT: usize = MAX_BITS;
pub const VALIDITY_SLOT: usize = MAX_BITS + RANDOM_BITS_LEN;
pub const TOTAL_SLOTS: usize = VALIDITY_SLOT + 1;

pub const DEFAULT_RUNTIME_DIR: &str = "runtime";

pub fn default_bgv_params() -> BgvParams {
    BgvParams {
        plain_mod: 786433,
        mult_depth: 24,
        batch_size: 128,
        security_level: 128,
    }
}

pub fn sample_client_values() -> Vec<usize> {
    vec![51, 49]
}
