use fhe_vdaf_1::artifacts::RuntimeLayout;
use fhe_vdaf_1::client::run_sample_clients;
use fhe_vdaf_1::config::DEFAULT_RUNTIME_DIR;
use openfhe_bgv_rs::Result;

fn main() -> Result<()> {
    let runtime_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_RUNTIME_DIR.to_string());
    let layout = RuntimeLayout::new(runtime_dir);
    let expected_total = run_sample_clients(&layout)?;
    println!("client stage complete: expected_total={expected_total}");
    Ok(())
}
