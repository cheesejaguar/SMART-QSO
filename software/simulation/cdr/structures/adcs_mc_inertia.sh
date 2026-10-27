#!/usr/bin/env bash
# SMART-QSO CDR (structures): re-run the ADCS Monte Carlo (sqso_adcs_mc) with the plant inertia
# of docs/environment/MASS_BUDGET.md section 6 instead of the mission simulation's box model.
#
# The repository code is not changed. The script copies software/rust and software/flight to a
# work directory, patches the copy so that sqso-sil's inertia_stowed()/inertia_deployed() read
# SQSO_I_STOWED / SQSO_I_DEPLOYED ("Ixx,Iyy,Izz" in kg m^2) when set, builds sqso_adcs_mc there
# and runs the cases of MASS_BUDGET section 6.4. Output: one CSV row per case on stdout.
#
#   software/simulation/cdr/structures/adcs_mc_inertia.sh [WORKDIR]
#
# Products of inertia are not modelled (the simulation's inertia is diagonal).
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../../../.." && pwd)"
WORK="${1:-$(mktemp -d)}"
mkdir -p "$WORK/software"
tar -C "$REPO/software" --exclude='rust/target' --exclude='rust/fuzz' --exclude='rust/mutants' -cf - rust flight \
  | tar -C "$WORK/software" -xf -
python3 - "$WORK/software/rust/sqso-sil/src/env/attitude.rs" <<'EOF'
import sys
p = sys.argv[1]
s = open(p).read()
s = s.replace("pub fn inertia_stowed() -> V3 {\n",
              "pub fn inertia_stowed() -> V3 {\n    if let Some(v) = env_v3(\"SQSO_I_STOWED\") { return v; }\n", 1)
s = s.replace("pub fn inertia_deployed() -> V3 {\n",
              "pub fn inertia_deployed() -> V3 {\n    if let Some(v) = env_v3(\"SQSO_I_DEPLOYED\") { return v; }\n", 1)
s += """
fn env_v3(k: &str) -> Option<V3> {
    let s = std::env::var(k).ok()?;
    let v: Vec<f64> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (v.len() == 3).then(|| V3::new(v[0], v[1], v[2]))
}
"""
open(p, "w").write(s)
EOF
(cd "$WORK/software/rust" && CARGO_TARGET_DIR="$WORK/target" cargo build --profile sil -p sqso-sil --bin sqso_adcs_mc >&2)
B="$WORK/target/sil/sqso_adcs_mc"
run() { local name=$1 st=$2 dp=$3; shift 3
        printf '%s,' "$name"; SQSO_I_STOWED=$st SQSO_I_DEPLOYED=$dp "$B" --trials 40 --spin 6 --gyro mg370 "$@" | tail -1; }
"$B" --trials 1 --days 0.5 | head -1 | sed 's/^/case,/'
run sil_default "" ""
run A_rods_cbe   "2.664e-3,2.917e-3,2.756e-3" "2.780e-3,4.188e-3,4.141e-3"
run A_rods_mev   "3.077e-3,3.355e-3,3.208e-3" "3.203e-3,4.757e-3,4.733e-3"
run A_coils_mev  "3.367e-3,3.644e-3,3.519e-3" "3.498e-3,5.047e-3,5.049e-3"
run A_rods_cbe_vhf_y "2.664e-3,2.917e-3,2.756e-3" "3.291e-3,3.677e-3,4.141e-3"
run A_rods_cbe_dip10 "2.664e-3,2.917e-3,2.756e-3" "2.780e-3,4.188e-3,4.141e-3" --dipole 0.010
for r in 1.02 1.03 1.05; do
  iy=$(python3 -c "print(4.141e-3*$r)")
  run "ratio_$r" "2.83e-3,3.083e-3,2.756e-3" "2.95e-3,$iy,4.141e-3"
  run "ratio_${r}_dip10" "2.83e-3,3.083e-3,2.756e-3" "2.95e-3,$iy,4.141e-3" --dipole 0.010
done
run B_rods_cbe   "2.788e-3,3.040e-3,2.756e-3" "2.904e-3,4.312e-3,4.141e-3"
run B_rods_mev   "3.209e-3,3.487e-3,3.208e-3" "3.335e-3,4.889e-3,4.733e-3"
run B_coils_cbe  "3.013e-3,3.265e-3,3.016e-3" "3.132e-3,4.536e-3,4.405e-3"
run B_coils_mev  "3.475e-3,3.752e-3,3.519e-3" "3.606e-3,5.154e-3,5.049e-3"
run B_rods_mev_dip10  "3.209e-3,3.487e-3,3.208e-3" "3.335e-3,4.889e-3,4.733e-3" --dipole 0.010
run B_coils_mev_dip10 "3.475e-3,3.752e-3,3.519e-3" "3.606e-3,5.154e-3,5.049e-3" --dipole 0.010
run B_rods_mev_tip30  "3.209e-3,3.487e-3,3.208e-3" "3.335e-3,4.889e-3,4.733e-3" --tipoff-sigma 10 --tipoff-max 30
# AntS configuration options of hardware/rf/ANTENNA_DESIGN.md v2.0 section 3 (layout B, rods)
run antA_rods_cbe "2.788e-3,3.040e-3,2.756e-3" "2.865e-3,4.059e-3,3.851e-3"
run antA_rods_mev "3.209e-3,3.487e-3,3.208e-3" "3.295e-3,4.622e-3,4.426e-3"
run antB_rods_cbe "2.788e-3,3.040e-3,2.756e-3" "3.158e-3,3.763e-3,3.847e-3"
run antB_rods_mev "3.209e-3,3.487e-3,3.208e-3" "3.605e-3,4.307e-3,4.422e-3"
run antC_rods_cbe "2.788e-3,3.040e-3,2.756e-3" "3.156e-3,4.057e-3,4.138e-3"
run antC_rods_mev "3.209e-3,3.487e-3,3.208e-3" "3.603e-3,4.619e-3,4.730e-3"
run antC_rods_cbe_spin8 "2.788e-3,3.040e-3,2.756e-3" "3.156e-3,4.057e-3,4.138e-3" --spin 8 --nutation 3e-4
# gain sweep at the near-degenerate inertia (layout A, rods, MEV)
SQSO_I_STOWED="3.077e-3,3.355e-3,3.208e-3" SQSO_I_DEPLOYED="3.203e-3,4.757e-3,4.733e-3" \
  "$B" --trials 40 --gyro mg370 --spin 6,8 --nutation 1e-4,3e-4,1e-3 --momentum 0.005,0.01 | tail -n +2 | sed 's/^/A_rods_mev_sweep,/'
