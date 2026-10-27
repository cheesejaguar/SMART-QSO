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
# Round 3 closure (2026-10-27; ADCS_DESIGN 1.9 §11.4 and §11.6): the gain sweep
# re-run with the current tensors (P3F-G-04, `sweep`); magnetometer gating at the
# mission simulation's duty, the wings stowed with the AntS held for 25 h, the
# AntS 3 h after the wings, one wing at the CBE and MEV tensors with the right,
# the default and the wrong preset (P3F-G-05); the face-photodiode sun sensors
# and the sun-vector knowledge columns (C-22); the one-wing identification
# signatures (C-21, `wing_id.csv`).
#
#   software/simulation/cdr/avionics/adcs_mc_cdr.sh OUT_DIR [sweep]
#
# One CSV per case in OUT_DIR, 40 trials of 2 days each (columns: the
# sqso_adcs_mc header; mag_rejected is the flight software's count of
# discarded magnetometer samples, mean per trial; know_* the sun-vector knowledge
# error in sunlight; rate_tilt_x the mean body rate's tilt toward +X; ants_max the
# worst sunlit angle within 3 h of a late AntS deployment). CARGO_TARGET_DIR is
# honoured.
set -euo pipefail
out=${1:?output directory}
mkdir -p "$out"
cd "$(dirname "$0")/../../../rust"
cargo build --profile sil --locked -p sqso-sil --bin sqso_adcs_mc >/dev/null
mc=${CARGO_TARGET_DIR:-./target}/sil/sqso_adcs_mc
run() { local name=$1; shift; "$mc" --trials 40 --threads "$(nproc)" "$@" > "$out/$name.csv"; echo "$name: $(tail -1 "$out/$name.csv")"; }
if [ "${2:-}" = sweep ]; then
  # Spin rate against nutation gain over Iyy/Izz (decision C-4; ADCS_DESIGN §11.4)
  for r in 1.000 1.005 1.020 1.050; do
    run "sweep_ratio_$r" --gyro mg370 --ratio "$r" --spin 6,7,8,9,10 --nutation 0.0001,0.0003,0.0005,0.001
  done
  run sweep_flight --gyro mg370 --spin 6,7,8,9,10 --nutation 0.0001,0.0003,0.0005,0.001
  exit 0
fi
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
# The plant at the CAD tensors of MASS_BUDGET v3.2 §6.1 and §6.5 (four AntS
# elements, the C-13 ballast, the harness after the M-11 parts audit), deployed,
# CBE 1,520.0 g and MEV 1,754.9 g, about the CG, off-diagonals -sum(m x y) (the
# flight software keeps the mean of the two: sqso_fsw::adcs::INERTIA,
# Inertia::DEPLOYED)
run plant_cbe            --gyro mg370 --plant 2.853,4.202,3.979,0.0108,-0.0080,-0.0027
run plant_mev            --gyro mg370 --plant 3.276,4.760,4.546,0.0115,-0.0158,0.0035
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
# Magnetometer gating (SRS-F143): 1.5 h a day of Jetson, 53 min a day keyed, in blocks
run gated                --gyro mg370 --gating 1.5,53
run gated_dipole_10      --gyro mg370 --gating 1.5,53 --dipole 0.010
run gated_double         --gyro mg370 --gating 3,106
run gated_no_gyro        --gyro none  --gating 1.5,53
# Wings stowed, the AntS held 25 h (C-18 cap): the stowed state (6-24 h), then
# after the AntS deploys at 25 h (statistics from 42 h)
run stowed_held_geo0     --gyro mg370 --wings none --ants-h 25 --days 1 --geometry 0
run stowed_held_geo3     --gyro mg370 --wings none --ants-h 25 --days 1 --geometry 3
run stowed_ants25_geo0   --gyro mg370 --wings none --ants-h 25 --days 3 --geometry 0
run stowed_ants25_geo3   --gyro mg370 --wings none --ants-h 25 --days 3 --geometry 3
# Both wings at 30 min, the AntS 3 h later (C-18 nominal worst case)
run ants_3h              --gyro mg370 --ants-h 3.5
run ants_3h_gated        --gyro mg370 --ants-h 3.5 --gating 1.5,53
# One wing with the plant at the CAD's CBE and MEV (MASS_BUDGET 3.2 §6.1),
# the matching, the default and the wrong preset
run wing1_geo2           --gyro mg370 --wings w1 --geometry 2
run wing2_geo1           --gyro mg370 --wings w2 --geometry 1
for g in 0 1 2; do
  run "wing1_cbe_geo$g"  --gyro mg370 --wings w1 --geometry "$g" --plant 2.818,3.904,3.646,-0.153,-0.018,0.007
  run "wing1_mev_geo$g"  --gyro mg370 --wings w1 --geometry "$g" --plant 3.236,4.429,4.176,-0.170,-0.025,0.012
  run "wing2_cbe_geo$g"  --gyro mg370 --wings w2 --geometry "$g" --plant 2.818,3.905,3.646,0.174,0.002,0.007
  run "wing2_mev_geo$g"  --gyro mg370 --wings w2 --geometry "$g" --plant 3.236,4.430,4.177,0.193,-0.007,0.012
done
run wing1_geo0_gated     --gyro mg370 --wings w1 --geometry 0 --gating 1.5,53
# The face photodiodes (ADS7138 inputs) in place of the generic sun-vector error (C-22)
run pd                   --gyro mg370 --css-model pd
run pd_cutoff            --gyro mg370 --css-model pd_cutoff
run pd_noalbedo          --gyro mg370 --css-model pd_noalbedo
run pd_gated             --gyro mg370 --css-model pd --gating 1.5,53
run pd_dipole_10         --gyro mg370 --css-model pd --dipole 0.010
run pd_no_gyro           --gyro none  --css-model pd
run pd_wing1_geo0        --gyro mg370 --css-model pd --wings w1 --geometry 0
run pd_wing2_geo0        --gyro mg370 --css-model pd --wings w2 --geometry 0
run pd_wing1_geo1        --gyro mg370 --css-model pd --wings w1 --geometry 1
run pd_no_wings_geo0     --gyro mg370 --css-model pd --wings none --geometry 0
run pd_stowed_held_geo0  --gyro mg370 --css-model pd --wings none --ants-h 25 --days 1 --geometry 0
# One-wing identification signatures (C-21)
"$mc" --wing-id > "$out/wing_id.csv"; echo "wing_id: $(wc -l < "$out/wing_id.csv") rows"
