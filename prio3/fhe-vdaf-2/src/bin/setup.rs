use fhe_vdaf_2::aggregator::Aggregator;
use fhe_vdaf_2::artifacts::RuntimeLayout;
use fhe_vdaf_2::config::{default_bgv_params, DEFAULT_RUNTIME_DIR};
use openfhe_bgv_rs::{BgvContext, Result};

fn main() -> Result<()> {
    let runtime_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_RUNTIME_DIR.to_string());
    let layout = RuntimeLayout::new(runtime_dir);
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

    println!("setup complete: {}", layout.setup_dir().display());
    Ok(())
}
