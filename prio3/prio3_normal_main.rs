use prio::codec::Encode;
use prio::vdaf::Vdaf;
use prio::vdaf::prio3::{Prio3, Prio3Sum};
use prio::vdaf::{Aggregator, Client, Collector, VerifyTransition, prio3::Prio3Count};
use rand::{RngExt, rng};
use std::fmt::Debug;

// ── Types ─────────────────────────────────────────────────────────────────────

struct ClientMeasurements<M> {
    measurements: Vec<M>,
}

struct ClientReport<V: Vdaf> {
    public_share: V::PublicShare,
    input_shares: Vec<V::InputShare>,
    nonce: [u8; 16],
}

// ── Phase 1: Clients ──────────────────────────────────────────────────────────
// Each client shards its measurement into one public share and one input share
// per aggregator, plus a nonce that binds the report to the verify key.

fn client_phase<V>(vdaf: &V, ctx: &[u8], measurements: &[V::Measurement]) -> Vec<ClientReport<V>>
where
    V: Client<16>,
    V::Measurement: Clone,
{
    let mut rng = rng();

    measurements
        .iter()
        .map(|measurement| {
            let nonce: [u8; 16] = rng.random();
            let (public_share, input_shares) = vdaf.shard(ctx, measurement, &nonce).unwrap();
            ClientReport {
                public_share,
                input_shares,
                nonce,
            }
        })
        .collect()
}

fn run_clients<V>(
    vdaf: &V,
    ctx: &[u8],
    clients_measurements: &[ClientMeasurements<V::Measurement>],
) -> Vec<Vec<ClientReport<V>>>
where
    V: Client<16>,
    V::Measurement: Clone,
{
    clients_measurements
        .iter()
        .map(|cm| client_phase(vdaf, ctx, &cm.measurements))
        .collect()
}

// ── Phase 2: Aggregators ──────────────────────────────────────────────────────
// For each report:
//   1. Every aggregator calls verify_init on its input share → (state, verifier_share)
//   2. All verifier_shares are combined into a single verifier_msg
//   3. Every aggregator calls verify_next → OutputShare
//
// Output: out_shares[agg_id] = Vec of output shares across all reports.

fn aggregator_phase<V>(
    vdaf: &V,
    ctx: &[u8],
    verify_key: &[u8; 32],
    reports: &[ClientReport<V>],
    num_aggregators: usize,
) -> Vec<Vec<V::OutputShare>>
where
    V: Aggregator<32, 16, AggregationParam = ()>,
    V::OutputShare: Clone,
{
    let mut out_shares: Vec<Vec<V::OutputShare>> = vec![vec![]; num_aggregators];

    for report in reports {
        // Step 1 — verify_init
        let mut verifier_states = vec![];
        let mut verifier_shares = vec![];

        for (agg_id, input_share) in report.input_shares.iter().enumerate() {
            println!("input_share {:?}", input_share);

            let (state, share) = vdaf
                .verify_init(
                    verify_key,
                    ctx,
                    agg_id,
                    &(),
                    &report.nonce,
                    &report.public_share,
                    input_share,
                )
                .unwrap();

            verifier_states.push(state);
            verifier_shares.push(share);
        }

        // Step 2 — verifier shares → joint message
        let verifier_msg = vdaf
            .verifier_shares_to_message(ctx, &(), verifier_shares)
            .unwrap();

        // Step 3 — verify_next → output share
        for (agg_id, state) in verifier_states.into_iter().enumerate() {
            let out_share = match vdaf.verify_next(ctx, state, verifier_msg.clone()).unwrap() {
                VerifyTransition::Finish(out_share) => out_share,
                _ => panic!("unexpected intermediate transition for agg_id={agg_id}"),
            };

            out_shares[agg_id].push(out_share);
        }
    }

    out_shares
}

fn run_aggregators<V>(
    vdaf: &V,
    ctx: &[u8],
    verify_key: &[u8; 32],
    all_client_reports: &[Vec<ClientReport<V>>],
    num_aggregators: usize,
) -> Vec<Vec<V::OutputShare>>
where
    V: Aggregator<32, 16, AggregationParam = ()>,
    V::OutputShare: Clone,
{
    let mut out_shares: Vec<Vec<V::OutputShare>> = vec![vec![]; num_aggregators];

    for client_reports in all_client_reports {
        let batch = aggregator_phase(vdaf, ctx, verify_key, client_reports, num_aggregators);

        for (agg_id, shares) in batch.into_iter().enumerate() {
            out_shares[agg_id].extend(shares);
        }
    }

    out_shares
}

// ── Phase 3: Collector ────────────────────────────────────────────────────────
// Each aggregator folds its output shares into one aggregate share, then the
// collector combines all aggregate shares into the final result.

fn collector_phase<V>(
    vdaf: &V,
    out_shares: Vec<Vec<V::OutputShare>>,
    num_measurements: usize,
) -> V::AggregateResult
where
    V: Collector + Aggregator<32, 16, AggregationParam = ()>,
    V::AggregateResult: Debug,
{
    let agg_shares: Vec<V::AggregateShare> = out_shares
        .into_iter()
        .map(|shares| vdaf.aggregate(&(), shares).unwrap())
        .collect();

    vdaf.unshard(&(), agg_shares, num_measurements).unwrap()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn run_vdaf<V>(
    vdaf: &V,
    ctx: &[u8],
    clients_measurements: &[ClientMeasurements<V::Measurement>],
) -> V::AggregateResult
where
    V: Client<16> + Aggregator<32, 16, AggregationParam = ()> + Collector,
    V::Measurement: Clone,
    V::OutputShare: Clone,
    V::AggregateResult: Debug,
{
    let num_aggregators = vdaf.num_aggregators();
    let num_measurements: usize = clients_measurements
        .iter()
        .map(|c| c.measurements.len())
        .sum();
    let verify_key: [u8; 32] = rng().random();

    let reports = run_clients(vdaf, ctx, clients_measurements);
    let out_shares = run_aggregators(vdaf, ctx, &verify_key, &reports, num_aggregators);

    collector_phase(vdaf, out_shares, num_measurements)
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let ctx = b"my context str";

    // ── Count ─────────────────────────────────────────────────────────────────
    let count_vdaf: Prio3Count = Prio3::new_count(2).unwrap();

    let count_clients = vec![
        ClientMeasurements {
            measurements: vec![true, false],
        },
        ClientMeasurements {
            measurements: vec![true],
        },
    ];

    let count_result = run_vdaf(&count_vdaf, ctx, &count_clients);
    println!("Count result: {count_result}");

    // ── Sum ───────────────────────────────────────────────────────────────────
    // max_measurement caps the valid input range to [0, max_measurement]
    let sum_vdaf: Prio3Sum = Prio3::new_sum(2, 100).unwrap();

    let sum_clients = vec![
        ClientMeasurements {
            measurements: vec![10, 20],
        },
        ClientMeasurements {
            measurements: vec![30],
        },
    ];

    let sum_result = run_vdaf(&sum_vdaf, ctx, &sum_clients);
    println!("Sum result: {sum_result}");
}
