#!/usr/bin/env bash
# The mission-simulation runs behind docs/operations/MISSION_TIMELINE.md §5 and §8 (CDR launch and ConOps review).
# Each is the commissioning scenario (14 days) with one setting changed; results go to <out>/<name>/.
#
#   software/simulation/cdr/launch/run_sil_variants.sh <out>
#
# Separation SOC: the integration SOC less 47 days of self-discharge at 20 °C (about 1.6 points), so
# integration_soc=0.40 separates at 38 %, 0.25 at 23 %, 0.15 at 13 %, 0.10 at 8 %.
set -euo pipefail
out=${1:?usage: run_sil_variants.sh <out>}
repo=$(cd "$(dirname "$0")/../../../.." && pwd)
cd "$repo/software/rust"
cargo build --profile sil -p sqso-sil
sil=./target/sil/sqso_sil
mkdir -p "$out"
run() {
    local name=$1
    shift
    # the simulation exits 1 when a check finds an anomaly; the failed-deployable and low-SOC cases are expected to
    "$sil" --scenario commissioning --out "$out/$name" "$@" > "$out/$name.log" 2>&1 || true
    echo "$name: $(grep -c 'Anomaly' "$out/$name.log" || true) anomalies"
}
run reference
run soc40 --set integration_soc=0.40
run soc25 --set integration_soc=0.25
run soc15 --set integration_soc=0.15
run soc10 --set integration_soc=0.10
run wings --set wings_failed=true
run dbl40 --set integration_soc=0.40 --set wings_failed=true --set adcs_failed=true
run outage3 --set contact_outage_days=3 --set contact_outage_start_day=0
run cold0 --set storage_temp_c=0
