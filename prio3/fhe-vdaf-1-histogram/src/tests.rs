use crate::aggregator::{Aggregator, run_aggregator};
use crate::artifacts::RuntimeLayout;
use crate::client::{Client, run_clients};
use crate::collector::{Collector, run_collector};
use crate::config::{BUCKETS, TOTAL_SLOTS, default_bgv_params};
use crate::encoding::{decode_histogram, encode_bucket};
use openfhe_bgv_rs::{BgvContext, Ciphertext, PublicKey, Result, SecretKey};
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

fn test_ctx() -> (BgvContext, PublicKey, SecretKey) {
    let ctx = BgvContext::new(default_bgv_params()).unwrap();
    let (pk, sk) = ctx.keygen();
    ctx.eval_sum_keygen(&sk);
    ctx.eval_rotate_keygen(&sk, &Aggregator::validation_rotation_indices());
    (ctx, pk, sk)
}

fn encrypt_slots(ctx: &BgvContext, pk: &PublicKey, slots: &[usize]) -> Ciphertext<usize> {
    let pt = ctx.make_plaintext(slots).unwrap();
    ctx.encrypt(pk, &pt).unwrap()
}

fn temp_runtime_dir(label: &str) -> RuntimeLayout {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::new(std::env::temp_dir().join(format!("fhe_vdaf1_histogram_{label}_{nonce}")))
}

fn setup_runtime(layout: &RuntimeLayout) -> Result<()> {
    layout.ensure_dirs()?;
    let ctx = BgvContext::new(default_bgv_params())?;
    let (pk, sk) = ctx.keygen();
    ctx.eval_sum_keygen(&sk);
    ctx.eval_rotate_keygen(&sk, &Aggregator::validation_rotation_indices());
    ctx.save_to_file(layout.context_path())?;
    pk.save_to_file(&ctx, layout.public_key_path())?;
    sk.save_to_file(&ctx, layout.secret_key_path())?;
    ctx.save_eval_mult_key_to_file(layout.eval_mult_key_path())?;
    ctx.save_eval_sum_key_to_file(layout.eval_sum_key_path())?;
    ctx.save_eval_rotate_key_to_file(layout.eval_rotate_key_path())?;
    Ok(())
}

#[test]
fn encode_bucket_uses_one_hot_slots() {
    for bucket in 0..BUCKETS {
        let slots = encode_bucket(bucket);
        assert_eq!(slots.len(), TOTAL_SLOTS);
        assert!(slots.iter().all(|&slot| slot == 0 || slot == 1));
        assert_eq!(slots.iter().sum::<usize>(), 1);
        assert_eq!(slots[bucket], 1);
    }
}

#[test]
fn validator_keeps_valid_input_and_zeros_invalid_input() {
    let _guard = lock_guard();
    let (ctx, pk, sk) = test_ctx();

    let valid_slots = encode_bucket(2);
    let valid_ct = encrypt_slots(&ctx, &pk, &valid_slots);
    let checked_valid = Aggregator::validate_or_zero(&ctx, &valid_ct).unwrap();
    let decoded_valid = ctx
        .decrypt::<usize>(&sk, &checked_valid, TOTAL_SLOTS)
        .unwrap();
    assert_eq!(decoded_valid, valid_slots);

    let mut invalid_slots = encode_bucket(2);
    invalid_slots[0] = 2;
    let invalid_ct = encrypt_slots(&ctx, &pk, &invalid_slots);
    let checked_invalid = Aggregator::validate_or_zero(&ctx, &invalid_ct).unwrap();
    let decoded_invalid = ctx
        .decrypt::<usize>(&sk, &checked_invalid, TOTAL_SLOTS)
        .unwrap();
    assert!(decoded_invalid.iter().all(|&slot| slot == 0));
}

#[test]
fn validator_zeros_binary_but_not_one_hot_input() {
    let _guard = lock_guard();
    let (ctx, pk, sk) = test_ctx();

    let invalid_slots = vec![1, 1, 0, 0, 0];
    let invalid_ct = encrypt_slots(&ctx, &pk, &invalid_slots);
    let checked = Aggregator::validate_or_zero(&ctx, &invalid_ct).unwrap();
    let decoded = ctx.decrypt::<usize>(&sk, &checked, TOTAL_SLOTS).unwrap();
    assert!(decoded.iter().all(|&slot| slot == 0));
}

#[test]
fn collector_decodes_sum_of_aggregator_shares() {
    let _guard = lock_guard();
    let (ctx, pk, sk) = test_ctx();

    let mut agg0 = Aggregator::new();
    agg0.push_input(encrypt_slots(&ctx, &pk, &encode_bucket(0)));
    agg0.push_input(encrypt_slots(&ctx, &pk, &encode_bucket(2)));

    let mut agg1 = Aggregator::new();
    agg1.push_input(encrypt_slots(&ctx, &pk, &encode_bucket(2)));
    agg1.push_input(encrypt_slots(&ctx, &pk, &encode_bucket(4)));

    let shares = [
        agg0.aggregate(&ctx).unwrap().unwrap(),
        agg1.aggregate(&ctx).unwrap().unwrap(),
    ];
    let total_ct = Collector::aggregate(&ctx, &shares).unwrap().unwrap();
    let decoded_slots = ctx.decrypt::<usize>(&sk, &total_ct, TOTAL_SLOTS).unwrap();

    assert_eq!(decode_histogram(&decoded_slots), vec![1, 0, 2, 0, 1]);
}

#[test]
fn file_backed_pipeline_matches_sample_histogram() {
    let _guard = lock_guard();
    let layout = temp_runtime_dir("pipeline");
    setup_runtime(&layout).unwrap();

    let clients = vec![Client::new(vec![0, 2, 2]), Client::new(vec![1, 4])];
    let expected_bucket_sum = run_clients(&layout, &clients).unwrap();

    for aggregator_idx in 0..2 {
        run_aggregator(&layout, aggregator_idx).unwrap();
    }

    let actual = run_collector(&layout).unwrap().unwrap();
    assert_eq!(actual, vec![1, 1, 2, 0, 1]);
    assert_eq!(expected_bucket_sum, 9);

    let output = fs::read_to_string(layout.collector_output_path()).unwrap();
    assert!(output.contains("collector_histogram=[1, 1, 2, 0, 1]"));

    let _ = fs::remove_dir_all(layout.root());
}
