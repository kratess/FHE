use crate::artifacts::{sorted_files, RuntimeLayout};
use crate::config::TOTAL_SLOTS;
use openfhe_bgv_rs::{BgvContext, Ciphertext, Result};

#[derive(Default)]
pub struct Aggregator {
    pub inputs: Vec<Ciphertext<usize>>,
}

impl Aggregator {
    pub fn new() -> Self {
        Self { inputs: Vec::new() }
    }

    pub fn push_input(&mut self, ct: Ciphertext<usize>) {
        self.inputs.push(ct);
    }

    pub fn validation_rotation_indices() -> Vec<i32> {
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

    pub fn validate_or_zero(ctx: &BgvContext, ct: &Ciphertext<usize>) -> Result<Ciphertext<usize>> {
        let is_ok_all_slots = Self::validity_mask(ctx, ct)?;
        ctx.eval_mul(ct, &is_ok_all_slots)
    }

    pub fn aggregate(&self, ctx: &BgvContext) -> Result<Option<Ciphertext<usize>>> {
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

pub fn run_aggregator(layout: &RuntimeLayout, aggregator_idx: usize) -> Result<usize> {
    let ctx = layout.load_aggregator_context()?;
    let mut aggregator = Aggregator::new();
    let files = sorted_files(&layout.aggregator_input_dir(aggregator_idx))?;
    let count = files.len();
    for file in files {
        aggregator.push_input(Ciphertext::load_from_file(&ctx, file)?);
    }

    if let Some(share) = aggregator.aggregate(&ctx)? {
        share.save_to_file(&ctx, layout.aggregate_share_path(aggregator_idx))?;
    }

    Ok(count)
}
