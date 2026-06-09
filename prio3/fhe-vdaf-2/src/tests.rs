use crate::aggregator::{run_aggregator, Aggregator};
use crate::artifacts::RuntimeLayout;
use crate::client::run_client_values;
use crate::collector::{run_collector, Collector};
use crate::config::{
    default_bgv_params, AGGREGATORS_NUM, LOW_BITS, LOW_BITS_MAX_VALUE, MAX_BITS,
    MAX_PER_AGGREGATORS_UNITS, MAX_SIZE, RANDOM_BITS_LEN, SIGNATURE_START_SLOT, TOP_BIT_WEIGHT,
    TOTAL_SLOTS, VALIDITY_SLOT,
};
use crate::encoding::{random_bits, signature_bits, split_value, to_shard};
use openfhe_bgv_rs::{BgvContext, Result};
use std::fs;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

fn openfhe_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_guard() -> std::sync::MutexGuard<'static, ()> {
    match openfhe_test_lock().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn temp_runtime_dir(label: &str) -> RuntimeLayout {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::new(std::env::temp_dir().join(format!("fhe_vdaf2_{label}_{nonce}")))
}

fn setup_runtime(layout: &RuntimeLayout) -> Result<()> {
    layout.ensure_dirs()?;
    let ctx = BgvContext::new(default_bgv_params())?;
    let (pk, sk) = ctx.keygen();
    ctx.eval_sum_keygen(&sk);
    ctx.eval_rotate_keygen(&sk, &Aggregator::append_bit_check_rotation_indices());
    ctx.save_to_file(layout.context_path())?;
    pk.save_to_file(&ctx, layout.public_key_path())?;
    sk.save_to_file(&ctx, layout.secret_key_path())?;
    ctx.save_eval_mult_key_to_file(layout.eval_mult_key_path())?;
    ctx.save_eval_sum_key_to_file(layout.eval_sum_key_path())?;
    ctx.save_eval_rotate_key_to_file(layout.eval_rotate_key_path())?;
    Ok(())
}

#[test]
fn split_value_preserves_sum_and_bounds() {
    for value in 0..=MAX_SIZE {
        let parts = split_value(value);
        assert_eq!(parts.iter().copied().sum::<usize>(), value * AGGREGATORS_NUM);
        assert!(parts.iter().all(|&p| p <= MAX_PER_AGGREGATORS_UNITS));
    }
}

#[test]
fn split_value_is_deterministic_by_default() {
    assert_eq!(split_value(51), split_value(51));
}

#[test]
fn to_shard_bit_decomposition_sums_to_value() {
    let values_units = [0usize, 1, 2, 3, 5, 7, 8, 15, 16, 31, 32, 37, 38, 63, 64, 99, 100];

    for &value_units in &values_units {
        let shards = to_shard(value_units);
        assert_eq!(shards.len(), MAX_BITS);

        for i in 0..LOW_BITS {
            assert!(shards[i] == 0 || shards[i] == 1);
        }

        assert!(shards[MAX_BITS - 1] == 0 || shards[MAX_BITS - 1] == 1);

        let low_value = (0..LOW_BITS).map(|i| (shards[i] & 1) << i).sum::<usize>();
        let reconstructed = low_value + shards[MAX_BITS - 1] * TOP_BIT_WEIGHT;
        assert_eq!(reconstructed, value_units);
    }
}

#[test]
fn top_bit_turns_on_only_above_low_bits_range() {
    for value_units in 0..=MAX_PER_AGGREGATORS_UNITS {
        let shards = to_shard(value_units);
        let expected_top_bit = usize::from(value_units > LOW_BITS_MAX_VALUE);
        assert_eq!(shards[MAX_BITS - 1], expected_top_bit);
    }
}

#[test]
fn unit_bit0_represents_fractional_part() {
    assert_eq!(to_shard(1)[0], 1);
}

#[test]
fn random_bits_is_binary() {
    let bits = random_bits(123);
    assert_eq!(bits.len(), RANDOM_BITS_LEN);
    assert!(bits.iter().all(|&b| b == 0 || b == 1));
    assert_eq!(random_bits(123), random_bits(123));
    assert_ne!(random_bits(123), random_bits(124));
}

#[test]
fn signature_bits_is_deterministic_by_value() {
    assert_eq!(signature_bits(51), signature_bits(51));
    assert_ne!(signature_bits(51), signature_bits(52));
}

#[test]
fn collector_accepts_matching_validity_and_signature_packs() {
    let mut a = vec![0usize; TOTAL_SLOTS];
    let mut b = vec![0usize; TOTAL_SLOTS];
    a[VALIDITY_SLOT] = 1;
    b[VALIDITY_SLOT] = 1;
    a[SIGNATURE_START_SLOT] = 1;
    b[SIGNATURE_START_SLOT] = 1;

    assert!(Collector::validate_aggregated_share_packs(&[a, b]));
}

#[test]
fn collector_rejects_mismatched_validity_packs() {
    let mut a = vec![0usize; TOTAL_SLOTS];
    let mut b = vec![0usize; TOTAL_SLOTS];
    a[VALIDITY_SLOT] = 1;
    b[VALIDITY_SLOT] = 0;

    assert!(!Collector::validate_aggregated_share_packs(&[a, b]));
}

#[test]
fn collector_rejects_mismatched_signature_packs() {
    let mut a = vec![0usize; TOTAL_SLOTS];
    let mut b = vec![0usize; TOTAL_SLOTS];
    a[VALIDITY_SLOT] = 1;
    b[VALIDITY_SLOT] = 1;
    a[SIGNATURE_START_SLOT] = 1;
    b[SIGNATURE_START_SLOT + 1] = 1;

    assert!(!Collector::validate_aggregated_share_packs(&[a, b]));
}

#[test]
fn collector_rejects_empty_or_truncated_packs() {
    assert!(!Collector::validate_aggregated_share_packs(&[]));
    assert!(!Collector::validate_aggregated_share_packs(&[vec![0usize; VALIDITY_SLOT]]));
}

#[test]
fn file_backed_pipeline_matches_sample_total() {
    let _guard = lock_guard();
    let layout = temp_runtime_dir("pipeline");
    setup_runtime(&layout).unwrap();

    let expected = run_client_values(&layout, &[51, 49]).unwrap();
    for aggregator_idx in 0..AGGREGATORS_NUM {
        run_aggregator(&layout, aggregator_idx).unwrap();
    }
    let actual = run_collector(&layout).unwrap().unwrap();

    assert_eq!(actual, expected);

    let output = fs::read_to_string(layout.collector_output_path()).unwrap();
    assert!(output.contains("collector_total=100"));

    let _ = fs::remove_dir_all(layout.root());
}
