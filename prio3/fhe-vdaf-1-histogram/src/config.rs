use openfhe_bgv_rs::BgvParams;

pub const BUCKETS: usize = 5;
pub const CLIENTS_NUM: usize = 2;
pub const AGGREGATORS_NUM: usize = 2;
pub const RNG_SEED: u64 = 42;
pub const TOTAL_SLOTS: usize = BUCKETS;

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
    [vec![0, 2, 2], vec![1, 4]]
}
