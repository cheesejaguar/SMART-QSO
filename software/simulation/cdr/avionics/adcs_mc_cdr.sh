#!/usr/bin/env bash
# CDR re-run of the ADCS Monte Carlo (docs/hardware/BUILD_VS_BUY.md §6.4) with
# the magnetometer measurement window (sqso_fsw::adcs::CoilTiming: coils on
# 900 ms of each 1 s cycle, sample after 50 ms settled) and the
# coil-to-magnetometer coupling modelled (sqso_sil::adcs_mc, 1e-4 T per A·m²).
#
#   software/simulation/cdr/avionics/adcs_mc_cdr.sh OUT_DIR
#
# One CSV per case in OUT_DIR, 40 trials of 2 days each (columns: the
# sqso_adcs_mc header; mag_rejected is the flight software's count of
# discarded magnetometer samples, mean per trial).
set -euo pipefail
out=${1:?output directory}
mkdir -p "$out"
cd "$(dirname "$0")/../../../rust"
cargo build --profile sil --locked -p sqso-sil --bin sqso_adcs_mc >/dev/null
mc=./target/sil/sqso_adcs_mc
run() { local name=$1; shift; "$mc" --trials 40 --threads "$(nproc)" "$@" > "$out/$name.csv"; echo "$name: $(tail -1 "$out/$name.csv")"; }
run mg370                --gyro mg370
run bmi088_cal           --gyro bmi088
run bmi088_0p2           --gyro bmi088 --gyro-bias 0.2
run bmi088_0p3           --gyro bmi088 --gyro-bias 0.3
run no_gyro              --gyro none
run tipoff_disp          --gyro mg370 --tipoff-sigma 3.33 --tipoff-max 10
run tipoff_30            --gyro mg370 --tipoff-sigma 10 --tipoff-max 30
run tipoff_50            --gyro mg370 --tipoff-sigma 30 --tipoff-max 50
run dipole_10            --gyro mg370 --dipole 0.010
run dipole_20            --gyro mg370 --dipole 0.020
run sensors_degraded     --gyro mg370 --mag-noise 200 --css 10
run spin4_dipole_10      --gyro mg370 --spin 4 --dipole 0.010
run no_coupling          --gyro mg370 --coupling 0
run coupling_3x          --gyro mg370 --coupling 3e-4
# The window missing: coils on through the sample and the software unaware
run no_window_unaware    --gyro mg370 --mtq-on 1000 --settle 0
run no_window_unaware_ng --gyro none  --mtq-on 1000 --settle 0
# Coils on through the sample, the software checking: every sample rejected
run no_window_detected   --gyro mg370 --mtq-on 1000 --settle 50 --days 0.5
