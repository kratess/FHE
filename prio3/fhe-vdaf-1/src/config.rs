use openfhe_bgv_rs::BgvParams;

pub const MAX_SIZE: usize = 100;
pub const CLIENTS_NUM: usize = 2;
pub const AGGREGATORS_NUM: usize = 2;
pub const RNG_SEED: u64 = 42;
pub const MAX_BITS: usize = if MAX_SIZE == 0 {
    1
} else {
    (usize::BITS - MAX_SIZE.leading_zeros()) as usize
};
pub const TOTAL_SLOTS: usize = MAX_BITS;
pub const LOW_BITS: usize = MAX_BITS.saturating_sub(1);
pub const LOW_BITS_MAX_VALUE: usize = if LOW_BITS == 0 {
    0
} else {
    (1usize << LOW_BITS) - 1
};
pub const TOP_BIT_WEIGHT: usize = MAX_SIZE - LOW_BITS_MAX_VALUE;

pub const DEFAULT_RUNTIME_DIR: &str = "runtime";

pub fn default_bgv_params() -> BgvParams {
    BgvParams {
        plain_mod: 786433,
        mult_depth: 24,
        batch_size: 128,
        security_level: 128,
    }
}

pub fn sample_client_values() -> [Vec<usize>; CLIENTS_NUM] {
    [vec![51, 7, 4], vec![49, 13]]
}
