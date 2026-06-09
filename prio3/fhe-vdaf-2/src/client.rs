use crate::artifacts::RuntimeLayout;
use crate::config::{sample_client_values, AGGREGATORS_NUM, TOTAL_SLOTS};
use crate::encoding::{signature_bits, split_value, to_shard};
use openfhe_bgv_rs::{BgvContext, Ciphertext, PublicKey, Result};

pub struct Client;

impl Client {
    pub fn shard(
        ctx: &BgvContext,
        pk: &PublicKey,
        value: usize,
    ) -> Result<[Ciphertext<usize>; AGGREGATORS_NUM]> {
        let parts = split_value(value);
        let signature = signature_bits(value);
        let cts: [Result<Ciphertext<usize>>; AGGREGATORS_NUM] = std::array::from_fn(|i| {
            let mut shards = to_shard(parts[i]);
            shards.extend(signature.iter().copied());
            shards.push(0);
            let pt = ctx.make_plaintext(&shards)?;
            ctx.encrypt(pk, &pt)
        });
        let v: Vec<Ciphertext<usize>> = cts.into_iter().collect::<Result<_>>()?;
        Ok(v.try_into()
            .unwrap_or_else(|_| unreachable!("fixed-size array conversion")))
    }
}

pub fn run_client_values(layout: &RuntimeLayout, values: &[usize]) -> Result<usize> {
    layout.ensure_dirs()?;
    let (ctx, pk) = layout.load_client_material()?;
    let mut expected_total = 0usize;

    for (client_idx, value) in values.iter().copied().enumerate() {
        let sharded = Client::shard(&ctx, &pk, value)?;
        expected_total += value;
        for aggregator_idx in 0..AGGREGATORS_NUM {
            sharded[aggregator_idx].save_to_file(
                &ctx,
                layout.shard_path(aggregator_idx, client_idx, 0),
            )?;
        }
    }

    let _ = TOTAL_SLOTS;
    Ok(expected_total)
}

pub fn run_sample_clients(layout: &RuntimeLayout) -> Result<usize> {
    run_client_values(layout, &sample_client_values())
}
