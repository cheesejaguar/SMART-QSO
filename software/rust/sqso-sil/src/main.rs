//! `sqso_sil`: the SMART-QSO software-in-the-loop mission simulation.
//!
//! ```text
//! sqso_sil [--scenario full|year|commissioning|ants_cold|smoke] [--out DIR] [--set key=value]...
//! ```
//!
//! Runs the flight software from integration to re-entry (the `full`
//! scenario) against the spacecraft and environment models, writes the
//! records and `summary.json` to the output directory, and exits non-zero
//! if any check found an anomaly. `software/simulation/sil/report.py`
//! turns the output into a report with plots.

use std::path::PathBuf;
use std::process::ExitCode;

use sqso_sil::config::Config;

fn usage() -> ExitCode {
    eprintln!("usage: sqso_sil [--scenario full|year|commissioning|ants_cold|smoke] [--out DIR] [--set key=value]...");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut cfg = Config::default();
    let mut out: Option<PathBuf> = None;
    let mut sets = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--scenario" => {
                let Some(name) = args.next() else { return usage() };
                let Some(c) = Config::scenario(&name) else {
                    eprintln!("unknown scenario {name}");
                    return usage();
                };
                cfg = c;
            }
            "--out" => out = args.next().map(PathBuf::from),
            "--set" => {
                let Some(kv) = args.next() else { return usage() };
                sets.push(kv);
            }
            "-h" | "--help" => {
                let _ = usage();
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    for kv in &sets {
        if let Err(e) = cfg.set(kv) {
            eprintln!("{e}");
            return usage();
        }
    }
    let out = out.unwrap_or_else(|| PathBuf::from(format!("sil-out/{}", cfg.scenario)));
    match sqso_sil::mission::run(&cfg, &out) {
        Ok(anomalies) => {
            eprintln!("[SIL] {} anomalies; results in {}", anomalies, out.display());
            if anomalies == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        Err(e) => {
            eprintln!("[SIL] run failed: {e}");
            ExitCode::from(3)
        }
    }
}
