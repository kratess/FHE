use openfhe_bgv_rs::{BgvContext, BgvParams, Ciphertext, PublicKey, Result, SecretKey};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tracing::debug;

const MAX_SIZE: usize = 100;
const AGGREGATORS_NUM: usize = 2;
const MAX_SCALED: usize = MAX_SIZE * AGGREGATORS_NUM;
const MAX_PER_AGGREGATORS_UNITS: usize = MAX_SIZE;
const RNG_SEED: usize = 42;
const RANDOM_BITS_LEN: usize = 64;
const MAX_BITS: usize = if MAX_PER_AGGREGATORS_UNITS == 0 {
    1
} else {
    (usize::BITS - MAX_PER_AGGREGATORS_UNITS.leading_zeros()) as usize
};
const LOW_BITS: usize = MAX_BITS.saturating_sub(1);
const LOW_BITS_MAX_VALUE: usize = if LOW_BITS == 0 {
    0
} else {
    (1usize << LOW_BITS) - 1
};
const TOP_BIT_WEIGHT: usize = MAX_PER_AGGREGATORS_UNITS - LOW_BITS_MAX_VALUE;
#[cfg(test)]
const SIGNATURE_START_SLOT: usize = MAX_BITS;
const VALIDITY_SLOT: usize = MAX_BITS + RANDOM_BITS_LEN;
const TOTAL_SLOTS: usize = VALIDITY_SLOT + 1;

struct Client {}
impl Client {
    fn split_value(value: usize) -> [usize; AGGREGATORS_NUM] {
        assert!(
            value <= MAX_SIZE,
            "value ({value}) must be <= MAX_SIZE ({MAX_SIZE})"
        );
        let value_scaled = value * AGGREGATORS_NUM as usize;
        assert!(
            value_scaled <= MAX_SCALED,
            "value_scaled ({value_scaled}) must be <= MAX_SCALED ({MAX_SCALED})"
        );
        let mut remaining = value_scaled;
        let mut rng = StdRng::seed_from_u64(RNG_SEED as u64);
        let mut result = [0usize; AGGREGATORS_NUM];

        debug!(
            "[split_value] value_scaled={} max_per_agg={} seed={}",
            value_scaled, MAX_PER_AGGREGATORS_UNITS, RNG_SEED
        );

        for i in (0..AGGREGATORS_NUM).rev() {
            let cap_remaining = i * MAX_PER_AGGREGATORS_UNITS;
            let min_to_send = remaining.saturating_sub(cap_remaining);
            let max_to_send = remaining.min(MAX_PER_AGGREGATORS_UNITS);
            debug_assert!(min_to_send <= max_to_send);
            let number = rng.gen_range(min_to_send..=max_to_send);

            result[i] = number;
            remaining -= number;
        }

        assert!(remaining == 0, "remaining ({remaining}) must be 0");
        debug!("[split_value] parts={:?}", result);

        result
    }

    fn random_bits(seed: u64) -> [usize; RANDOM_BITS_LEN] {
        let mut rng = StdRng::seed_from_u64(seed);
        std::array::from_fn(|_| rng.gen_range(0..=1))
    }

    fn signature_bits(value: usize) -> [usize; RANDOM_BITS_LEN] {
        let seed = (RNG_SEED as u64) ^ ((value as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        Self::random_bits(seed)
    }

    fn to_shard(value_units: usize) -> Vec<usize> {
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

    fn shard(
        ctx: &BgvContext,
        pk: &PublicKey,
        value: usize,
    ) -> Result<[Ciphertext<usize>; AGGREGATORS_NUM]> {
        let parts = Self::split_value(value);
        let signature = Self::signature_bits(value);
        let cts: [Result<Ciphertext<usize>>; AGGREGATORS_NUM] =
            std::array::from_fn(|i| {
                let mut shards = Self::to_shard(parts[i]);
                shards.extend(signature.iter().copied());
                // Reserve one extra slot for the aggregator's bit-check result.
                shards.push(0);
                let pt = ctx.make_plaintext(&shards)?;
                ctx.encrypt(pk, &pt)
            });
        let v: Vec<Ciphertext<usize>> = cts.into_iter().collect::<Result<_>>()?;
        Ok(v.try_into()
            .unwrap_or_else(|_| unreachable!("fixed-size array conversion")))
    }
}

struct Aggregator {
    shards: Vec<Ciphertext<usize>>,
}
impl Aggregator {
    fn new() -> Self {
        Self { shards: Vec::new() }
    }

    fn push_shard(&mut self, ct: Ciphertext<usize>) {
        self.shards.push(ct);
    }

    fn append_bit_check_rotation_index() -> i32 {
        -(VALIDITY_SLOT as i32)
    }

    fn append_bit_check_rotation_indices() -> Vec<i32> {
        (1..TOTAL_SLOTS).map(|i| -(i as i32)).collect()
    }

    fn sum_all_slots_slot0(
        ctx: &BgvContext,
        ct: &Ciphertext<usize>,
        slots: usize,
    ) -> Result<Ciphertext<usize>> {
        // OpenFHE `EvalSum` (as wired in this wrapper) returns the total sum in slot 0.
        ctx.eval_sum(ct, slots as i32)
    }
    fn pow_2k(
        ctx: &BgvContext,
        ct: &Ciphertext<usize>,
        k: u32,
    ) -> Result<Ciphertext<usize>> {
        let mut acc = ct.clone();
        for _ in 0..k {
            acc = ctx.eval_mul(&acc, &acc)?;
        }
        Ok(acc)
    }

    fn replicate_slot0_all_slots(
        ctx: &BgvContext,
        ct: &Ciphertext<usize>,
    ) -> Result<Ciphertext<usize>> {
        let mut slot0_mask = vec![0usize; TOTAL_SLOTS];
        slot0_mask[0] = 1;
        let slot0_mask_pt = ctx.make_plaintext(&slot0_mask)?;
        let slot0 = ctx.eval_mul_plain(ct, &slot0_mask_pt)?;

        let mut replicated = slot0.clone();
        for i in 1..TOTAL_SLOTS {
            let rotated = ctx.eval_rotate(&slot0, -(i as i32))?;
            replicated = ctx.eval_add(&replicated, &rotated)?;
        }

        Ok(replicated)
    }

    fn append_bit_check(
        ctx: &BgvContext,
        ct: &Ciphertext<usize>,
    ) -> Result<Ciphertext<usize>> {
        // ct*(ct-1) == 0 iff slots are in {0,1}
        let ones_pt = ctx.make_plaintext(&vec![1usize; TOTAL_SLOTS])?;
        let ct_minus_one = ctx.eval_sub_plain(ct, &ones_pt)?;
        let bit_err_per_slot = ctx.eval_mul(ct, &ct_minus_one)?;

        // Sum errors over the first `total_slots` slots (result in slot 0).
        // (The last slot is reserved for the check bit and is expected to be 0 at this stage,
        // so including it in the sum is safe.)
        let err_sum = Self::sum_all_slots_slot0(ctx, &bit_err_per_slot, TOTAL_SLOTS)?;

        // Little Fermat: x^(p-1) is 0 iff x==0 else 1 (mod p), assuming prime p.
        // We implement this for the plaintext moduli we use here.
        let is_error = match ctx.plain_mod() {
            // p-1 = 2^16
            65537 => Self::pow_2k(ctx, &err_sum, 16)?,
            // p-1 = 786432 = 3 * 2^18
            786433 => {
                let sq = ctx.eval_mul(&err_sum, &err_sum)?;
                let cube = ctx.eval_mul(&sq, &err_sum)?;
                Self::pow_2k(ctx, &cube, 18)?
            }
            p => panic!("unsupported plain_mod={p} for append_bit_check"),
        };

        // Convert is_error -> is_ok = 1 - is_error.
        let zeros_pt = ctx.make_plaintext(&vec![0usize; TOTAL_SLOTS])?;
        let ones_pt = ctx.make_plaintext(&vec![1usize; TOTAL_SLOTS])?;
        let zero = ctx.eval_mul_plain(&is_error, &zeros_pt)?;
        let one = ctx.eval_add_plain(&zero, &ones_pt)?;
        let is_ok = ctx.eval_sub(&one, &is_error)?;

        let is_ok_all_slots = Self::replicate_slot0_all_slots(ctx, &is_ok)?;
        // Move slot0 into the reserved last slot so we can expose the validity count there.
        let is_ok_at_validity_slot =
            ctx.eval_rotate(&is_ok, Self::append_bit_check_rotation_index())?;

        // Place `is_ok` only in the last slot by multiplying by a plaintext mask, then add it.
        let mut mask = vec![0usize; TOTAL_SLOTS];
        mask[VALIDITY_SLOT] = 1;
        let mask_pt = ctx.make_plaintext(&mask)?;
        let is_ok_at_last = ctx.eval_mul_plain(&is_ok_at_validity_slot, &mask_pt)?;

        // Clear the reserved last slot in `ct` (defensive), drop invalid shards by multiplying
        // every slot by the validity bit, then add that bit back into the reserved slot.
        let mut clear_last = vec![1usize; TOTAL_SLOTS];
        clear_last[VALIDITY_SLOT] = 0;
        let clear_last_pt = ctx.make_plaintext(&clear_last)?;
        let ct_cleared = ctx.eval_mul_plain(ct, &clear_last_pt)?;
        let ct_cleared = ctx.eval_mul(&ct_cleared, &is_ok_all_slots)?;

        ctx.eval_add(&ct_cleared, &is_ok_at_last)
    }

    fn aggregate(&self, ctx: &BgvContext) -> Result<Option<Ciphertext<usize>>> {
        let mut sum = None;

        for shard in &self.shards {
            let checked = Self::append_bit_check(ctx, shard)?;
            sum = Some(match sum {
                Some(acc) => ctx.eval_add(&acc, &checked)?,
                None => checked,
            });
        }

        Ok(sum)
    }

    // (Debug helper intentionally omitted here; use `debug_print_ct` on intermediate ciphertexts
    // from the call site when needed.)
}

struct Collector {}
impl Collector {
    fn aggregate(
        ctx: &BgvContext,
        shares: &[Ciphertext<usize>],
    ) -> Result<Option<Ciphertext<usize>>> {
        let mut sum = None;

        for share in shares {
            sum = Some(match sum {
                Some(acc) => ctx.eval_add(&acc, share)?,
                None => share.clone(),
            });
        }

        Ok(sum)
    }

    fn decode_value_units(slots: &[usize]) -> usize {
        let low_value = slots
            .iter()
            .take(LOW_BITS)
            .enumerate()
            .map(|(i, slot)| slot << i)
            .sum::<usize>();

        low_value + slots[MAX_BITS - 1] * TOP_BIT_WEIGHT
    }

    fn rescale_units(value_units: usize) -> usize {
        assert_eq!(
            value_units % AGGREGATORS_NUM,
            0,
            "aggregated units ({value_units}) must divide evenly by AGGREGATORS_NUM ({AGGREGATORS_NUM})"
        );
        value_units / AGGREGATORS_NUM
    }

    #[cfg(test)]
    fn validate_aggregated_share_packs(decoded_shares: &[Vec<usize>]) -> bool {
        let Some(first) = decoded_shares.first() else {
            return false;
        };

        if decoded_shares
            .iter()
            .any(|slots| slots.len() <= VALIDITY_SLOT)
        {
            return false;
        }

        let validity_count = first[VALIDITY_SLOT];
        let signature = &first[SIGNATURE_START_SLOT..VALIDITY_SLOT];

        decoded_shares.iter().all(|slots| {
            slots[VALIDITY_SLOT] == validity_count
                && &slots[SIGNATURE_START_SLOT..VALIDITY_SLOT] == signature
        })
    }
}

fn debug_print_ct(
    ctx: &BgvContext,
    sk: &SecretKey,
    ct: &Ciphertext<usize>,
    slots: usize,
    label: &str,
) -> Result<()> {
    let decoded = ctx.decrypt::<usize>(sk, ct, slots)?;
    debug!("[ct] {label} slots={slots} decoded={decoded:?}");
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let ctx = BgvContext::new(BgvParams {
        plain_mod: 786433,
        mult_depth: 24,
        batch_size: 128,
        security_level: 128,
    })?;
    let (pk, sk) = ctx.keygen();
    ctx.eval_sum_keygen(&sk);
    ctx.eval_rotate_keygen(&sk, &Aggregator::append_bit_check_rotation_indices());

    let mut aggregators: Vec<Aggregator> =
        (0..AGGREGATORS_NUM).map(|_| Aggregator::new()).collect();

    let client_values = [51, 49];
    for value in client_values {
        let sharded = Client::shard(&ctx, &pk, value)?;
        for i in 0..AGGREGATORS_NUM {
            debug_print_ct(&ctx, &sk, &sharded[i], TOTAL_SLOTS, "client_ct")?;
            aggregators[i].push_shard(sharded[i].clone());
        }
    }

    let mut aggregated_shares = Vec::new();
    for (agg_idx, agg) in aggregators.iter().enumerate() {
        println!(
            "aggregator {agg_idx}: aggregating {} encrypted shard(s)",
            agg.shards.len()
        );
        if let Some(aggregated) = agg.aggregate(&ctx)? {
            println!("aggregator {agg_idx}: produced encrypted aggregate share");
            debug_print_ct(
                &ctx,
                &sk,
                &aggregated,
                TOTAL_SLOTS,
                &format!("aggregator_{agg_idx}_aggregate_share"),
            )?;
            aggregated_shares.push(aggregated);
        } else {
            println!("aggregator {agg_idx}: no shards to aggregate");
        }
    }
    println!(
        "collector: received {} encrypted aggregate share(s)",
        aggregated_shares.len()
    );

    if let Some(total_ct) = Collector::aggregate(&ctx, &aggregated_shares)? {
        let decoded = ctx.decrypt::<usize>(&sk, &total_ct, TOTAL_SLOTS)?;
        let value_units = Collector::decode_value_units(&decoded);
        let value = Collector::rescale_units(value_units);
        let expected = client_values.iter().sum::<usize>();
        println!("collector_decoded_slots={decoded:?}");
        println!("collector_total={value} expected={expected}");
        assert_eq!(value, expected);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AGGREGATORS_NUM, Aggregator, Client, Collector, LOW_BITS, LOW_BITS_MAX_VALUE, MAX_BITS,
        MAX_PER_AGGREGATORS_UNITS, MAX_SIZE, RANDOM_BITS_LEN, SIGNATURE_START_SLOT, TOP_BIT_WEIGHT,
        TOTAL_SLOTS, VALIDITY_SLOT,
    };
    use openfhe_bgv_rs::{BgvContext, BgvParams, Ciphertext, PublicKey, SecretKey};
    use std::sync::{Mutex, OnceLock};

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

    #[test]
    fn split_value_preserves_sum_and_bounds() {
        for value in 0..=MAX_SIZE {
            let parts = Client::split_value(value);
            assert_eq!(
                parts.iter().copied().sum::<usize>(),
                value * AGGREGATORS_NUM
            );
            assert!(parts.iter().all(|&p| p <= MAX_PER_AGGREGATORS_UNITS));
        }
    }

    #[test]
    fn split_value_is_deterministic_by_default() {
        assert_eq!(Client::split_value(51), Client::split_value(51));
    }

    #[test]
    fn to_shard_bit_decomposition_sums_to_value() {
        let values_units = [
            0usize, 1, 2, 3, 5, 7, 8, 15, 16, 31, 32, 37, 38, 63, 64, 99, 100,
        ];

        for &value_units in &values_units {
            let shards = Client::to_shard(value_units);
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
            let shards = Client::to_shard(value_units);
            let expected_top_bit = usize::from(value_units > LOW_BITS_MAX_VALUE);
            assert_eq!(shards[MAX_BITS - 1], expected_top_bit);
        }
    }

    #[test]
    fn unit_bit0_represents_fractional_part() {
        // unit=1 corresponds to 1/AGGREGATORS_NUM of the original scale
        assert_eq!(Client::to_shard(1)[0], 1);
    }

    #[test]
    fn random_bits_is_binary() {
        let bits = Client::random_bits(123);
        assert_eq!(bits.len(), RANDOM_BITS_LEN);
        assert!(bits.iter().all(|&b| b == 0 || b == 1));
        assert_eq!(Client::random_bits(123), Client::random_bits(123));
        assert_ne!(Client::random_bits(123), Client::random_bits(124));
    }

    #[test]
    fn signature_bits_is_deterministic_by_value() {
        assert_eq!(Client::signature_bits(51), Client::signature_bits(51));
        assert_ne!(Client::signature_bits(51), Client::signature_bits(52));
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
        assert!(!Collector::validate_aggregated_share_packs(&[vec![
            0usize;
            VALIDITY_SLOT
        ]]));
    }

    #[test]
    fn collector_rescales_aggregated_units() {
        assert_eq!(Collector::rescale_units(102), 51);
    }

    #[test]
    #[should_panic(expected = "must divide evenly")]
    fn collector_rejects_non_integral_rescale() {
        Collector::rescale_units(101);
    }

    fn test_ctx() -> (
        BgvContext,
        PublicKey,
        SecretKey,
    ) {
        let ctx = BgvContext::new(BgvParams {
            plain_mod: 786433,
            mult_depth: 25,
            batch_size: 128,
            security_level: 128,
        })
        .unwrap();
        let (pk, sk) = ctx.keygen();
        ctx.eval_sum_keygen(&sk);
        ctx.eval_rotate_keygen(&sk, &Aggregator::append_bit_check_rotation_indices());
        (ctx, pk, sk)
    }

    fn encrypt_slots(
        ctx: &BgvContext,
        pk: &PublicKey,
        slots: &[usize],
    ) -> Ciphertext<usize> {
        let pt = ctx.make_plaintext(slots).unwrap();
        ctx.encrypt(pk, &pt).unwrap()
    }

    fn encoded_shard_slots(value_units: usize, signature: &[usize; RANDOM_BITS_LEN]) -> Vec<usize> {
        let mut slots = Client::to_shard(value_units);
        slots.extend(signature.iter().copied());
        slots.push(0);
        slots
    }

    #[test]
    fn aggregator_validates_valid_shard_and_masks_invalid_shard() {
        let _guard = lock_guard();
        let (ctx, pk, sk) = test_ctx();
        let signature = Client::signature_bits(51);

        let valid_slots = encoded_shard_slots(37, &signature);
        let valid_ct = encrypt_slots(&ctx, &pk, &valid_slots);
        let checked_valid = Aggregator::append_bit_check(&ctx, &valid_ct).unwrap();
        let decoded_valid = ctx
            .decrypt::<usize>(&sk, &checked_valid, TOTAL_SLOTS)
            .unwrap();
        assert_eq!(
            &decoded_valid[..VALIDITY_SLOT],
            &valid_slots[..VALIDITY_SLOT]
        );
        assert_eq!(decoded_valid[VALIDITY_SLOT], 1);

        let mut invalid_slots = encoded_shard_slots(37, &signature);
        invalid_slots[SIGNATURE_START_SLOT] = 2;
        let invalid_ct = encrypt_slots(&ctx, &pk, &invalid_slots);
        let checked_invalid = Aggregator::append_bit_check(&ctx, &invalid_ct).unwrap();
        let decoded_invalid = ctx
            .decrypt::<usize>(&sk, &checked_invalid, TOTAL_SLOTS)
            .unwrap();
        assert!(
            decoded_invalid[..VALIDITY_SLOT]
                .iter()
                .all(|&slot| slot == 0)
        );
        assert_eq!(decoded_invalid[VALIDITY_SLOT], 0);
    }

    #[test]
    fn collector_rejects_pack_when_one_aggregator_drops_a_matching_shard() {
        let _guard = lock_guard();
        let (ctx, pk, sk) = test_ctx();
        let value = 51usize;
        let parts = Client::split_value(value);
        let signature = Client::signature_bits(value);

        let mut agg0 = Aggregator::new();
        let mut bad_slots = encoded_shard_slots(parts[0], &signature);
        bad_slots[SIGNATURE_START_SLOT] = 2;
        agg0.push_shard(encrypt_slots(&ctx, &pk, &bad_slots));

        let mut agg1 = Aggregator::new();
        let good_slots = encoded_shard_slots(parts[1], &signature);
        agg1.push_shard(encrypt_slots(&ctx, &pk, &good_slots));

        let decoded: Vec<Vec<usize>> = [agg0, agg1]
            .iter()
            .map(|agg| {
                let aggregated = agg.aggregate(&ctx).unwrap().unwrap();
                ctx.decrypt::<usize>(&sk, &aggregated, TOTAL_SLOTS).unwrap()
            })
            .collect();

        assert_eq!(decoded[0][VALIDITY_SLOT], 0);
        assert_eq!(decoded[1][VALIDITY_SLOT], 1);
        assert!(!Collector::validate_aggregated_share_packs(&decoded));
    }
}
