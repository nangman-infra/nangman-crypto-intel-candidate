use super::args::Args;
use super::summary::RunSummary;
use crate::error::AppResult;
use crate::io::{
    read_market_feature_deltas, read_market_regime_contexts, read_structured_packets,
    read_universe_snapshot, write_processing_results,
};
use crate::policy::load_policy;
use crate::scoring::{MarketArtifactInputs, process_packet_with_artifacts};
use crate::time::now_ms;

pub fn run(args: Args) -> AppResult<RunSummary> {
    let policy = load_policy(&args.policy_file)?;
    let packets = read_structured_packets(&args.input_file)?;
    let universe = args
        .universe_snapshot_file
        .as_deref()
        .map(read_universe_snapshot)
        .transpose()?;
    let market_feature_deltas = args
        .market_feature_delta_file
        .as_deref()
        .map(read_market_feature_deltas)
        .transpose()?
        .unwrap_or_default();
    let market_regime_contexts = args
        .market_regime_context_file
        .as_deref()
        .map(read_market_regime_contexts)
        .transpose()?
        .unwrap_or_default();
    let created_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let results = packets
        .into_iter()
        .map(|packet| {
            process_packet_with_artifacts(
                packet,
                &policy,
                MarketArtifactInputs {
                    universe: universe.as_ref(),
                    market_feature_deltas: &market_feature_deltas,
                    market_regime_contexts: &market_regime_contexts,
                },
                created_at_ms,
            )
        })
        .collect::<Vec<_>>();
    let evidence_bundles_created = results
        .iter()
        .filter(|result| result.evidence_bundle.is_some())
        .count();
    let hypothesis_states_created = results
        .iter()
        .filter(|result| result.hypothesis_state.is_some())
        .count();
    let output_files = if let Some(output_dir) = args.output_dir.as_deref() {
        write_processing_results(output_dir, &results)?
            .into_iter()
            .map(|path| path.display().to_string())
            .collect()
    } else {
        println!("{}", serde_json::to_string_pretty(&results)?);
        Vec::new()
    };
    Ok(RunSummary {
        processed_packets: results.len(),
        evidence_bundles_created,
        hypothesis_states_created,
        output_files,
    })
}
