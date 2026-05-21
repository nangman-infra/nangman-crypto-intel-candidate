use intel_candidate_app::cli::{parse_args, print_help, run};
use std::env;
use std::process;

fn main() {
    match parse_args(env::args().skip(1)).and_then(|args| match args {
        Some(args) => run(args),
        None => {
            print_help();
            Ok(intel_candidate_app::cli::RunSummary {
                processed_packets: 0,
                evidence_bundles_created: 0,
                hypothesis_states_created: 0,
                output_files: Vec::new(),
            })
        }
    }) {
        Ok(summary) => {
            if summary.processed_packets > 0 {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
            }
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}
