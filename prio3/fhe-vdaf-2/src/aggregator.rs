use crate::artifacts::{sorted_files, RuntimeLayout};
use crate::config::{TOTAL_SLOTS, VALIDITY_SLOT};
use openfhe_bgv_rs::{BgvContext, Ciphertext, Result};

#[derive(Default)]
pub struct Aggregator {
    pub shards: Vec<Ciphertext<usize>>,
}

impl Aggregator {
    pub fn new() -> Self {
        Self { shards: Vec::new() }
    }

    pub fn push_shard(&mut self, ct: Ciphertext<usize>) {
        self.shards.push(ct);
    }

    fn append_bit_check_rotation_index() -> i32 {
        -(VALIDITY_SLOT as i32)
    }

    pub fn append_bit_check_rotation_indices() -> Vec<i32> {
        (1..TOTAL_SLOTS).map(|i| -(i as i32)).collect()
    }

    fn sum_all_slots_slot0(
        ctx: &BgvContext,
        ct: &Ciphertext<usize>,
        slots: usize,
    ) -> Result<Ciphertext<usize>> {
        ctx.eval_sum(ct, slots as i32)
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

    pub fn append_bit_check(
        ctx: &BgvContext,
        ct: &Ciphertext<usize>,
    ) -> Result<Ciphertext<usize>> {
        let ones_pt = ctx.make_plaintext(&vec![1usize; TOTAL_SLOTS])?;
        let ct_minus_one = ctx.eval_sub_plain(ct, &ones_pt)?;
        let bit_err_per_slot = ctx.eval_mul(ct, &ct_minus_one)?;
        let err_sum = Self::sum_all_slots_slot0(ctx, &bit_err_per_slot, TOTAL_SLOTS)?;

        let is_error = match ctx.plain_mod() {
            65537 => Self::pow_2k(ctx, &err_sum, 16)?,
            786433 => {
                let sq = ctx.eval_mul(&err_sum, &err_sum)?;
                let cube = ctx.eval_mul(&sq, &err_sum)?;
                Self::pow_2k(ctx, &cube, 18)?
            }
            p => panic!("unsupported plain_mod={p} for append_bit_check"),
        };

        let zeros_pt = ctx.make_plaintext(&vec![0usize; TOTAL_SLOTS])?;
        let ones_pt = ctx.make_plaintext(&vec![1usize; TOTAL_SLOTS])?;
        let zero = ctx.eval_mul_plain(&is_error, &zeros_pt)?;
        let one = ctx.eval_add_plain(&zero, &ones_pt)?;
        let is_ok = ctx.eval_sub(&one, &is_error)?;

        let is_ok_all_slots = Self::replicate_slot0_all_slots(ctx, &is_ok)?;
        let is_ok_at_validity_slot =
            ctx.eval_rotate(&is_ok, Self::append_bit_check_rotation_index())?;

        let mut mask = vec![0usize; TOTAL_SLOTS];
        mask[VALIDITY_SLOT] = 1;
        let mask_pt = ctx.make_plaintext(&mask)?;
        let is_ok_at_last = ctx.eval_mul_plain(&is_ok_at_validity_slot, &mask_pt)?;

        let mut clear_last = vec![1usize; TOTAL_SLOTS];
        clear_last[VALIDITY_SLOT] = 0;
        let clear_last_pt = ctx.make_plaintext(&clear_last)?;
        let ct_cleared = ctx.eval_mul_plain(ct, &clear_last_pt)?;
        let ct_cleared = ctx.eval_mul(&ct_cleared, &is_ok_all_slots)?;

        ctx.eval_add(&ct_cleared, &is_ok_at_last)
    }

    pub fn aggregate(&self, ctx: &BgvContext) -> Result<Option<Ciphertext<usize>>> {
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
}

pub fn run_aggregator(layout: &RuntimeLayout, aggregator_idx: usize) -> Result<usize> {
    let ctx = layout.load_aggregator_context()?;
    let mut aggregator = Aggregator::new();
    let files = sorted_files(&layout.aggregator_input_dir(aggregator_idx))?;
    let count = files.len();
    for file in files {
        aggregator.push_shard(Ciphertext::load_from_file(&ctx, file)?);
    }

    if let Some(share) = aggregator.aggregate(&ctx)? {
        share.save_to_file(&ctx, layout.aggregate_share_path(aggregator_idx))?;
    }

    Ok(count)
}
