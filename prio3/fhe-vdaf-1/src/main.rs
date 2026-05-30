use openfhe_bgv_rs::{BgvContext, BgvParams, Ciphertext, PublicKey, Result, SecretKey};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tracing::debug;

const MAX_SIZE: usize = 100;
const CLIENTS_NUM: usize = 2;
const AGGREGATORS_NUM: usize = 2;
const RNG_SEED: u64 = 42;
const MAX_BITS: usize = if MAX_SIZE == 0 {
    1
} else {
    (usize::BITS - MAX_SIZE.leading_zeros()) as usize
};
const TOTAL_SLOTS: usize = MAX_BITS;
const LOW_BITS: usize = MAX_BITS.saturating_sub(1);
const LOW_BITS_MAX_VALUE: usize = if LOW_BITS == 0 {
    0
} else {
    (1usize << LOW_BITS) - 1
};
const TOP_BIT_WEIGHT: usize = MAX_SIZE - LOW_BITS_MAX_VALUE;

struct Client {
    values: Vec<usize>,
}

impl Client {
    fn new(values: Vec<usize>) -> Self {
        Self { values }
    }

    fn encode_value(value: usize) -> Vec<usize> {
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

    fn encrypt_inputs(&self, ctx: &BgvContext, pk: &PublicKey) -> Result<Vec<Ciphertext<usize>>> {
        self.values
            .iter()
            .map(|&value| {
                let encoded = Self::encode_value(value);
                let pt = ctx.make_plaintext(&encoded)?;
                ctx.encrypt(pk, &pt)
            })
            .collect()
    }
}

struct Aggregator {
    inputs: Vec<Ciphertext<usize>>,
}

impl Aggregator {
    fn new() -> Self {
        Self { inputs: Vec::new() }
    }

    fn push_input(&mut self, ct: Ciphertext<usize>) {
        self.inputs.push(ct);
    }

    fn validation_rotation_indices() -> Vec<i32> {
        (1..TOTAL_SLOTS).map(|i| -(i as i32)).collect()
    }

    fn sum_all_slots_slot0(ctx: &BgvContext, ct: &Ciphertext<usize>) -> Result<Ciphertext<usize>> {
        ctx.eval_sum(ct, TOTAL_SLOTS as i32)
    }

    fn pow_2k(ctx: &BgvContext, ct: &Ciphertext<usize>, k: u32) -> Result<Ciphertext<usize>> {
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

    fn validity_mask(ctx: &BgvContext, ct: &Ciphertext<usize>) -> Result<Ciphertext<usize>> {
        let ones_pt = ctx.make_plaintext(&vec![1usize; TOTAL_SLOTS])?;
        let ct_minus_one = ctx.eval_sub_plain(ct, &ones_pt)?;
        let bit_err_per_slot = ctx.eval_mul(ct, &ct_minus_one)?;
        let err_sum = Self::sum_all_slots_slot0(ctx, &bit_err_per_slot)?;

        let is_error = match ctx.plain_mod() {
            65537 => Self::pow_2k(ctx, &err_sum, 16)?,
            786433 => {
                let sq = ctx.eval_mul(&err_sum, &err_sum)?;
                let cube = ctx.eval_mul(&sq, &err_sum)?;
                Self::pow_2k(ctx, &cube, 18)?
            }
            p => panic!("unsupported plain_mod={p} for validity_mask"),
        };

        let ones_pt = ctx.make_plaintext(&vec![1usize; TOTAL_SLOTS])?;
        let zero = ctx.eval_sub(&is_error, &is_error)?;
        let one = ctx.eval_add_plain(&zero, &ones_pt)?;
        let is_ok = ctx.eval_sub(&one, &is_error)?;

        Self::replicate_slot0_all_slots(ctx, &is_ok)
    }

    fn validate_or_zero(ctx: &BgvContext, ct: &Ciphertext<usize>) -> Result<Ciphertext<usize>> {
        let is_ok_all_slots = Self::validity_mask(ctx, ct)?;
        ctx.eval_mul(ct, &is_ok_all_slots)
    }

    fn aggregate(&self, ctx: &BgvContext) -> Result<Option<Ciphertext<usize>>> {
        let mut sum = None;

        for input in &self.inputs {
            let checked = Self::validate_or_zero(ctx, input)?;
            sum = Some(match sum {
                Some(acc) => ctx.eval_add(&acc, &checked)?,
                None => checked,
            });
        }

        Ok(sum)
    }
}

struct Collector;

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

    fn decode_value(slots: &[usize]) -> usize {
        let low_value = slots
            .iter()
            .take(LOW_BITS)
            .enumerate()
            .map(|(i, slot)| slot << i)
            .sum::<usize>();

        low_value + slots[MAX_BITS - 1] * TOP_BIT_WEIGHT
    }
}

fn debug_print_ct(
    ctx: &BgvContext,
    sk: &SecretKey,
    ct: &Ciphertext<usize>,
    label: &str,
) -> Result<()> {
    let decoded = ctx.decrypt::<usize>(sk, ct, TOTAL_SLOTS)?;
    debug!("[ct] {label} slots={TOTAL_SLOTS} decoded={decoded:?}");
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
    ctx.eval_rotate_keygen(&sk, &Aggregator::validation_rotation_indices());

    let clients = [Client::new(vec![51, 7, 4]), Client::new(vec![49, 13])];
    assert_eq!(clients.len(), CLIENTS_NUM);

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let mut aggregators: Vec<Aggregator> =
        (0..AGGREGATORS_NUM).map(|_| Aggregator::new()).collect();

    let mut expected_total = 0usize;
    for (client_idx, client) in clients.iter().enumerate() {
        let encrypted_inputs = client.encrypt_inputs(&ctx, &pk)?;
        for (input_idx, (value, ct)) in client
            .values
            .iter()
            .copied()
            .zip(encrypted_inputs.into_iter())
            .enumerate()
        {
            expected_total += value;
            let encoded = Client::encode_value(value);
            let agg_idx = rng.gen_range(0..AGGREGATORS_NUM);
            println!(
                "client {client_idx} input {input_idx}: value={value} encoded={encoded:?} -> aggregator {agg_idx}"
            );
            debug_print_ct(
                &ctx,
                &sk,
                &ct,
                &format!("client_{client_idx}_input_{input_idx}"),
            )?;
            aggregators[agg_idx].push_input(ct);
        }
    }

    let mut aggregate_shares = Vec::new();
    for (agg_idx, agg) in aggregators.iter().enumerate() {
        println!(
            "aggregator {agg_idx}: validating and aggregating {} encrypted input(s)",
            agg.inputs.len()
        );
        if let Some(share) = agg.aggregate(&ctx)? {
            debug_print_ct(
                &ctx,
                &sk,
                &share,
                &format!("aggregator_{agg_idx}_aggregate_share"),
            )?;
            aggregate_shares.push(share);
        }
    }

    println!(
        "collector: received {} encrypted aggregate share(s)",
        aggregate_shares.len()
    );

    if let Some(total_ct) = Collector::aggregate(&ctx, &aggregate_shares)? {
        let decoded_slots = ctx.decrypt::<usize>(&sk, &total_ct, TOTAL_SLOTS)?;
        let total = Collector::decode_value(&decoded_slots);
        println!("collector_decoded_slots={decoded_slots:?}");
        println!("collector_total={total} expected={expected_total}");
        assert_eq!(total, expected_total);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Aggregator, Client, Collector, MAX_SIZE, TOTAL_SLOTS};
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

    fn test_ctx() -> (BgvContext, PublicKey, SecretKey) {
        let ctx = BgvContext::new(BgvParams {
            plain_mod: 786433,
            mult_depth: 24,
            batch_size: 128,
            security_level: 128,
        })
        .unwrap();
        let (pk, sk) = ctx.keygen();
        ctx.eval_sum_keygen(&sk);
        ctx.eval_rotate_keygen(&sk, &Aggregator::validation_rotation_indices());
        (ctx, pk, sk)
    }

    fn encrypt_slots(ctx: &BgvContext, pk: &PublicKey, slots: &[usize]) -> Ciphertext<usize> {
        let pt = ctx.make_plaintext(slots).unwrap();
        ctx.encrypt(pk, &pt).unwrap()
    }

    #[test]
    fn encode_value_uses_binary_slots() {
        for value in 0..=MAX_SIZE {
            let slots = Client::encode_value(value);
            assert_eq!(slots.len(), TOTAL_SLOTS);
            assert!(slots.iter().all(|&slot| slot == 0 || slot == 1));
            assert_eq!(Collector::decode_value(&slots), value);
        }
    }

    #[test]
    fn max_value_encodes_as_all_ones() {
        let slots = Client::encode_value(MAX_SIZE);
        assert!(slots.iter().all(|&slot| slot == 1));
        assert_eq!(Collector::decode_value(&slots), MAX_SIZE);
    }

    #[test]
    fn validator_keeps_valid_input_and_zeros_invalid_input() {
        let _guard = lock_guard();
        let (ctx, pk, sk) = test_ctx();

        let valid_slots = Client::encode_value(37);
        let valid_ct = encrypt_slots(&ctx, &pk, &valid_slots);
        let checked_valid = Aggregator::validate_or_zero(&ctx, &valid_ct).unwrap();
        let decoded_valid = ctx
            .decrypt::<usize>(&sk, &checked_valid, TOTAL_SLOTS)
            .unwrap();
        assert_eq!(decoded_valid, valid_slots);

        let mut invalid_slots = Client::encode_value(37);
        invalid_slots[0] = 2;
        let invalid_ct = encrypt_slots(&ctx, &pk, &invalid_slots);
        let checked_invalid = Aggregator::validate_or_zero(&ctx, &invalid_ct).unwrap();
        let decoded_invalid = ctx
            .decrypt::<usize>(&sk, &checked_invalid, TOTAL_SLOTS)
            .unwrap();
        assert!(decoded_invalid.iter().all(|&slot| slot == 0));
    }

    #[test]
    fn collector_decodes_sum_of_aggregator_shares() {
        let _guard = lock_guard();
        let (ctx, pk, sk) = test_ctx();

        let mut agg0 = Aggregator::new();
        agg0.push_input(encrypt_slots(&ctx, &pk, &Client::encode_value(10)));
        agg0.push_input(encrypt_slots(&ctx, &pk, &Client::encode_value(20)));

        let mut agg1 = Aggregator::new();
        agg1.push_input(encrypt_slots(&ctx, &pk, &Client::encode_value(30)));
        agg1.push_input(encrypt_slots(&ctx, &pk, &Client::encode_value(40)));

        let shares = [
            agg0.aggregate(&ctx).unwrap().unwrap(),
            agg1.aggregate(&ctx).unwrap().unwrap(),
        ];
        let total_ct = Collector::aggregate(&ctx, &shares).unwrap().unwrap();
        let decoded_slots = ctx.decrypt::<usize>(&sk, &total_ct, TOTAL_SLOTS).unwrap();

        assert_eq!(Collector::decode_value(&decoded_slots), 100);
    }
}
