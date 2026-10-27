"""
Accidental-explosion probability of SMART-QSO over its orbital lifetime
(NASA-STD-8719.14C Requirement 4.4-1: < 0.001; 47 CFR 97.207(g)(1)(iii)).

The only stored-energy source that can deflagrate is the 4S1P Li-ion pack
(INR18650-35E, ~50 Wh). No propellant, pressure vessel or ordnance is carried.
Every venting or thermal-runaway event of any cell is counted as an explosion
(conservative: an 18650 vents through its cap; fragmentation of the 1U is not
expected). Failure rates are review allocations (stated below), not data.

Run: python3 explosion_probability.py
"""

T_ORBIT = 10.0  # years in orbit, bounding (life at <= 485 km is <= 8 yr in the cases of debris_lifetime.py)
T_MISSION = 2.0  # years before the commanded passivation (charge-path disconnect)
P_PASSIVATED = 0.5  # probability the passivation command is executed (L3 success ~0.5)

# Overcharge: active regulator failure, then two dormant protections fail, then the CID fails
LAM_CHARGER = 0.02  # /yr, charger CV regulation fails high
LAM_BMS_OV = 0.01  # /yr, BMS over-voltage cut-off latent failure
LAM_HW_OV = 0.01  # /yr, independent comparator latent failure
P_CID = 0.1  # cell current-interrupt device fails to act on demand

# Spontaneous internal short in a screened cell
LAM_CELL_ISC = 1e-6  # per cell-year; order of the field rate commonly quoted for quality 18650 cells
N_CELLS = 4

# External short: insulation breach, then fuse, PTC and BMS short-circuit protection all fail
LAM_BREACH = 1e-3
LAM_FUSE, LAM_PTC, LAM_BMS_SC = 1e-4, 1e-3, 1e-2

# Over-discharge then recharge (copper dissolution): UV cut-off and comparator fail, then recharge
LAM_UV, LAM_HW_UV = 0.01, 0.01
P_DEEP_DRAIN = 0.05  # a load drains the pack below 2 V before the fault is seen


def overcharge(t_exposed):
    # integral over t of lam_c * (lam_b t)(lam_k t) dt = lam_c lam_b lam_k t^3 / 3
    return LAM_CHARGER * LAM_BMS_OV * LAM_HW_OV * t_exposed ** 3 / 3.0 * P_CID


def external_short(t):
    return LAM_BREACH * (LAM_FUSE * t) * (LAM_PTC * t) * (LAM_BMS_SC * t) * t / 4.0


def overdischarge(t):
    return LAM_UV * LAM_HW_UV * t ** 2 / 2.0 * P_DEEP_DRAIN


def main():
    # Charging is possible until passivation; if passivation never happens, until re-entry
    t_charge = P_PASSIVATED * T_MISSION + (1 - P_PASSIVATED) * T_ORBIT
    rows = [
        ("Overcharge (charger, BMS OV, comparator, CID)", overcharge(t_charge)),
        ("Internal short, spontaneous", LAM_CELL_ISC * N_CELLS * T_ORBIT),
        ("External short (insulation, fuse, PTC, BMS)", external_short(T_ORBIT)),
        ("Over-discharge then recharge", overdischarge(t_charge)),
    ]
    total = sum(p for _, p in rows)
    for name, p in rows:
        print(f"| {name} | {p:.1e} |")
    print(f"| **Total** | **{total:.1e}** (requirement < 1e-3, margin x{1e-3/total:.0f}) |")


if __name__ == "__main__":
    main()
