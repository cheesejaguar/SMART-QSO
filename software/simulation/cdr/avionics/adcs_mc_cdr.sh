#!/usr/bin/env bash
# CDR re-run of the ADCS Monte Carlo (docs/hardware/BUILD_VS_BUY.md §6.4) with
# the magnetometer measurement window (sqso_fsw::adcs::CoilTiming: coils on
# 900 ms of each 1 s cycle, sample after 50 ms settled) and the
# coil-to-magnetometer coupling modelled (sqso_sil::adcs_mc, 1e-4 T per A·m²),
# the full inertia tensor of layout B (sqso_fsw::adcs::Inertia::DEPLOYED, STR-01)
# and the flight spin of 8 deg/s with nutation damping 3e-4 N m s. The
# robustness cases sweep Iyy/Izz over 1.00 to 1.05; the wing cases deploy one
# wing (or none) with the default and the matching ADCS_SET_GEOMETRY preset
# (STR-27).
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
run bmi088_0p5           --gyro bmi088 --gyro-bias 0.5
run bmi088_uncal         --gyro bmi088_uncal
run no_gyro              --gyro none
run tipoff_disp          --gyro mg370 --tipoff-sigma 3.33 --tipoff-max 10
run tipoff_30            --gyro mg370 --tipoff-sigma 10 --tipoff-max 30
run tipoff_50            --gyro mg370 --tipoff-sigma 30 --tipoff-max 50
run dipole_10            --gyro mg370 --dipole 0.010
run dipole_20            --gyro mg370 --dipole 0.020
run dipole_50            --gyro mg370 --dipole 0.050
run sensors_degraded     --gyro mg370 --mag-noise 200 --css 10
run spin4_dipole_10      --gyro mg370 --spin 4 --dipole 0.010
run spin6_kn1e-4         --gyro mg370 --spin 6 --nutation 0.0001
# Inertia: Iyy/Izz across the range, and the flight software on the major axis
run ratio_1p000          --gyro mg370 --ratio 1.00
run ratio_1p005          --gyro mg370 --ratio 1.005
run ratio_1p020          --gyro mg370 --ratio 1.02
run ratio_1p050          --gyro mg370 --ratio 1.05
run major_axis           --gyro mg370 --fsw-axis plant
# The plant at MASS_BUDGET v3.0 §6.1 layout B rods deployed CBE and MEV (the
# flight software keeps its mean of the two)
run plant_cbe            --gyro mg370 --plant 2.904,4.312,4.141,-0.029,-0.028,-0.044
run plant_mev            --gyro mg370 --plant 3.335,4.889,4.733,-0.031,-0.034,-0.046
# One wing or none deployed: the default geometry and the matching preset
run wing1_geo0           --gyro mg370 --wings w1 --geometry 0
run wing1_geo1           --gyro mg370 --wings w1 --geometry 1
run wing2_geo0           --gyro mg370 --wings w2 --geometry 0
run wing2_geo2           --gyro mg370 --wings w2 --geometry 2
run no_wings_geo0        --gyro mg370 --wings none --geometry 0
run no_wings_geo3        --gyro mg370 --wings none --geometry 3
run no_coupling          --gyro mg370 --coupling 0
run coupling_3x          --gyro mg370 --coupling 3e-4
# The window missing: coils on through the sample and the software unaware
run no_window_unaware    --gyro mg370 --mtq-on 1000 --settle 0
run no_window_unaware_ng --gyro none  --mtq-on 1000 --settle 0
# Coils on through the sample, the software checking: every sample rejected
run no_window_detected   --gyro mg370 --mtq-on 1000 --settle 50 --days 0.5
