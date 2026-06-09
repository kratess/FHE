use crate::artifacts::{RuntimeLayout, sorted_files};
use crate::config::TOTAL_SLOTS;
use crate::encoding::decode_histogram;
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
}

pub fn run_collector(layout: &RuntimeLayout) -> Result<Option<Vec<usize>>> {
    let (ctx, sk) = layout.load_collector_material()?;
    let mut shares = Vec::new();
    for path in sorted_files(&layout.aggregator_dir())? {
        shares.push(Ciphertext::load_from_file(&ctx, path)?);
    }

    let Some(total_ct) = Collector::aggregate(&ctx, &shares)? else {
        return Ok(None);
    };

    let decoded_slots = ctx.decrypt::<usize>(&sk, &total_ct, TOTAL_SLOTS)?;
    let histogram = decode_histogram(&decoded_slots);
    fs::write(
        layout.collector_output_path(),
        format!("collector_decoded_slots={decoded_slots:?}\ncollector_histogram={histogram:?}\n"),
    )?;
    Ok(Some(histogram))
}
