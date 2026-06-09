use crate::artifacts::RuntimeLayout;
use crate::config::{sample_client_values, AGGREGATORS_NUM, RNG_SEED};
use crate::encoding::encode_value;
use openfhe_bgv_rs::{BgvContext, Ciphertext, PublicKey, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Clone, Debug)]
pub struct Client {
    pub values: Vec<usize>,
}

impl Client {
    pub fn new(values: Vec<usize>) -> Self {
        Self { values }
    }

    pub fn encrypt_inputs(&self, ctx: &BgvContext, pk: &PublicKey) -> Result<Vec<Ciphertext<usize>>> {
        self.values
            .iter()
            .map(|&value| {
                let encoded = encode_value(value);
                let pt = ctx.make_plaintext(&encoded)?;
                ctx.encrypt(pk, &pt)
            })
            .collect()
    }
}

pub fn run_clients(layout: &RuntimeLayout, clients: &[Client]) -> Result<usize> {
    layout.ensure_dirs()?;
    let (ctx, pk) = layout.load_client_material()?;
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
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
            let aggregator_idx = rng.gen_range(0..AGGREGATORS_NUM);
            ct.save_to_file(&ctx, layout.client_ciphertext_path(aggregator_idx, client_idx, input_idx))?;
        }
    }

    Ok(expected_total)
}

pub fn run_sample_clients(layout: &RuntimeLayout) -> Result<usize> {
    let clients = sample_client_values()
        .into_iter()
        .map(Client::new)
        .collect::<Vec<_>>();
    run_clients(layout, &clients)
}
