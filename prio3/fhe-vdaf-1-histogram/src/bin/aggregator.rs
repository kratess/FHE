use fhe_vdaf_1_histogram::aggregator::run_aggregator;
use fhe_vdaf_1_histogram::artifacts::RuntimeLayout;
use fhe_vdaf_1_histogram::config::{AGGREGATORS_NUM, DEFAULT_RUNTIME_DIR};
use openfhe_bgv_rs::Result;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let runtime_dir = args
        .next()
        .unwrap_or_else(|| DEFAULT_RUNTIME_DIR.to_string());
    let layout = RuntimeLayout::new(runtime_dir);

    if let Some(id) = args.next() {
        let aggregator_idx = id
            .parse::<usize>()
            .expect("aggregator id must be an integer");
        let count = run_aggregator(&layout, aggregator_idx)?;
        println!("aggregator {aggregator_idx} processed {count} ciphertext(s)");
    } else {
        for aggregator_idx in 0..AGGREGATORS_NUM {
            let count = run_aggregator(&layout, aggregator_idx)?;
            println!("aggregator {aggregator_idx} processed {count} ciphertext(s)");
        }
    }

    Ok(())
}
