use fhe_vdaf_2::artifacts::RuntimeLayout;
use fhe_vdaf_2::collector::run_collector;
use fhe_vdaf_2::config::DEFAULT_RUNTIME_DIR;
use openfhe_bgv_rs::Result;

fn main() -> Result<()> {
    let runtime_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_RUNTIME_DIR.to_string());
    let layout = RuntimeLayout::new(runtime_dir);
    match run_collector(&layout)? {
        Some(total) => println!("collector total={total}"),
        None => println!("collector found no aggregate shares"),
    }
    Ok(())
}
