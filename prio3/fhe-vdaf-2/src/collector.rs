use crate::artifacts::{sorted_files, RuntimeLayout};
use crate::config::{RANDOM_BITS_LEN, SIGNATURE_START_SLOT, TOTAL_SLOTS, VALIDITY_SLOT};
use crate::encoding::{decode_value_units, rescale_units};
use openfhe_bgv_rs::{BgvContext, Ciphertext, Result};
use std::fs;

pub struct Collector;

impl Collector {
    pub fn aggregate(
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

    pub fn validate_aggregated_share_packs(decoded_shares: &[Vec<usize>]) -> bool {
        let Some(first) = decoded_shares.first() else {
            return false;
        };

        if decoded_shares.iter().any(|slots| slots.len() <= VALIDITY_SLOT) {
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

pub fn run_collector(layout: &RuntimeLayout) -> Result<Option<usize>> {
    let (ctx, sk) = layout.load_collector_material()?;
    let mut shares = Vec::new();
    for path in sorted_files(&layout.aggregator_dir())? {
        shares.push(Ciphertext::load_from_file(&ctx, path)?);
    }

    let Some(total_ct) = Collector::aggregate(&ctx, &shares)? else {
        return Ok(None);
    };

    let decoded = ctx.decrypt::<usize>(&sk, &total_ct, TOTAL_SLOTS)?;
    let value_units = decode_value_units(&decoded);
    let value = rescale_units(value_units);
    fs::write(
        layout.collector_output_path(),
        format!(
            "collector_decoded_slots={decoded:?}\ncollector_total={value}\nsignature_bits_len={RANDOM_BITS_LEN}\n"
        ),
    )?;
    Ok(Some(value))
}
