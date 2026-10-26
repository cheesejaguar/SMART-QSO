//! 4S1P Li-ion battery pack: equivalent-circuit cells with aging.
//!
//! Pack (`hardware/eps/BATTERY_DESIGN.md` §2–3, BASELINE §5): four Samsung
//! INR18650-35E cells in series, 3.45 Ah typical, 14.4 V nominal, 49.7 Wh.
//! The pack only reports voltages; the protection logic (thresholds below)
//! belongs to the EPS model.
//!
//! # Cell model (per cell; current positive = charge)
//!
//! * V = OCV(SOC, T) + v₁ + I R₀(T, SOC), one RC pair
//!   dv₁/dt = I/C₁ − v₁/(R₁C₁), integrated exactly over each step.
//! * OCV(SOC) at 25 °C: a relaxed-OCV table typical of a high-energy
//!   NCA/NMC 18650 such as the 35E (3.72 V at 50 %; 4.19 V full, 2.9 V empty
//!   at the 2.65 V, 0.2C cut-off), plus an entropic term dU/dT(SOC) of
//!   −0.3…+0.1 mV/K. **Assumption**: no measured curve of the flight lot
//!   exists.
//! * R₀ = 40 mΩ at 25 °C and mid SOC (35E DC resistance ≈ 35–45 mΩ; the
//!   acceptance limit of §9.1 is 25 mΩ AC), Arrhenius with 28 kJ/mol (× 2.8 at
//!   0 °C, × 7.6 at −20 °C), × up to 1.6 at empty; R₁ = 15 mΩ, τ = 30 s at
//!   25 °C.
//! * Coulombic efficiency 0.998 on charge; self-discharge 1.5 %/month at
//!   25 °C, doubling every 10 °C.
//!
//! # Aging (semi-empirical; assumptions to be replaced by §9.3 test data)
//!
//! * **Calendar fade** Q_cal = k(T, SOC) √t: 3.0 %/√yr at 25 °C and full
//!   charge, Arrhenius 50 kJ/mol, stress 0.4 + 0.6·SOC (form of Schmalstieg
//!   et al., *J. Power Sources* 257, 2014, with √t as is common for SEI
//!   growth). Integrated with the equivalent-time method so it follows
//!   changing temperature and SOC.
//! * **Cycle fade** per discharged ampere-hour, with a depth-of-discharge
//!   stress: each equivalent full cycle (EFC) at depth D costs
//!   g₀ (1 + 8 D) with g₀ = 0.0025 % (100 % DoD: 0.0225 %/EFC, 20 % fade in
//!   ≈ 900 full cycles; 10 % DoD: 0.0045 %/EFC, five times less per EFC,
//!   in line with shallow-cycling data such as Ecker et al., *J. Power
//!   Sources* 248, 2014). Temperature factor: Arrhenius 30 kJ/mol above
//!   25 °C, e^(0.04 (25 − T)) below (lithium plating). BATTERY_DESIGN §3.3
//!   deliberately takes no shallow-cycle credit (500 EFC to 60 %); this model
//!   is the expected case, that analysis the bound.
//! * **Resistance growth**: R₀ and R₁ × (1 + 2.5 × fade).

use super::math::{Rng, interp};

/// Cells in series.
pub const N_CELLS: usize = 4;
/// Typical cell capacity (Ah, 0.2C).
pub const CELL_CAPACITY_AH: f64 = 3.45;
/// Nominal cell voltage (V).
pub const CELL_NOMINAL_V: f64 = 3.60;
/// End-of-charge voltage per cell (V).
pub const CHARGE_V_CELL: f64 = 4.20;
/// BMS over-voltage threshold per cell (V; BASELINE §5 ≤ 4.25 V).
pub const OV_CELL_V: f64 = 4.25;
/// BMS under-voltage threshold per cell (V; BATTERY_DESIGN §4.1).
pub const UV_CELL_V: f64 = 2.8;
/// Operational floor per cell (V; BATTERY_DESIGN §3.1).
pub const FLOOR_CELL_V: f64 = 3.0;
/// BMS over-current trip (A; BATTERY_DESIGN §4.1).
pub const OC_A: f64 = 5.0;
/// Charge temperature window (°C; BASELINE §5).
pub const CHARGE_T_MIN_C: f64 = 0.0;
/// Charge temperature window, upper limit (°C).
pub const CHARGE_T_MAX_C: f64 = 45.0;
/// Discharge temperature window (°C; BATTERY_DESIGN §4.1).
pub const DISCHARGE_T_MIN_C: f64 = -10.0;
/// Discharge temperature window, upper limit (°C).
pub const DISCHARGE_T_MAX_C: f64 = 60.0;

const R_GAS: f64 = 8.314;
const T_REF_K: f64 = 298.15;

const SOC_TAB: [f64; 24] = [
    -0.02, 0.0, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85,
    0.90, 0.95, 1.00, 1.03, 1.05,
];
const OCV_TAB: [f64; 24] = [
    2.50, 2.90, 3.30, 3.43, 3.50, 3.55, 3.58, 3.61, 3.63, 3.66, 3.69, 3.72, 3.76, 3.80, 3.84, 3.88, 3.92, 3.97, 4.02,
    4.07, 4.13, 4.19, 4.28, 4.40,
];
const ENT_SOC: [f64; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
const ENT_V_K: [f64; 6] = [-0.3e-3, -0.2e-3, -0.1e-3, 0.0, 0.05e-3, 0.1e-3];

const R0_25: f64 = 0.040;
const R1_25: f64 = 0.015;
const C1_F: f64 = 2000.0;
const EA_R: f64 = 28_000.0;
const ETA_CHARGE: f64 = 0.998;
const SELF_DISCHARGE_PER_S: f64 = 0.015 / (30.0 * 86_400.0);
const K_CAL_PER_SQRT_S: f64 = 0.030 / 5_617.0; // 3 %/√yr; √(365.25 d) = 5617 √s
const EA_CAL: f64 = 50_000.0;
const G0_PER_EFC: f64 = 0.000_025;
const DOD_STRESS: f64 = 8.0;
const EA_CYC: f64 = 30_000.0;
const R_GROWTH_PER_FADE: f64 = 2.5;

/// Open-circuit voltage of a cell at 25 °C (V).
pub fn ocv_25(soc: f64) -> f64 {
    interp(&SOC_TAB, &OCV_TAB, soc)
}

/// Entropic coefficient dU/dT (V/K) at a state of charge.
pub fn entropic(soc: f64) -> f64 {
    interp(&ENT_SOC, &ENT_V_K, soc)
}

/// Open-circuit voltage of a cell (V) at a state of charge and temperature.
pub fn ocv(soc: f64, temp_c: f64) -> f64 {
    ocv_25(soc) + entropic(soc) * (temp_c - 25.0)
}

fn arrhenius(ea: f64, temp_c: f64) -> f64 {
    let t = temp_c + 273.15;
    (ea / R_GAS * (1.0 / t - 1.0 / T_REF_K)).exp()
}

/// One cell's state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    /// State of charge as a fraction of the present capacity.
    pub soc: f64,
    /// RC-pair voltage (V).
    pub v_rc: f64,
    /// Calendar capacity fade (fraction of the new capacity).
    pub fade_cal: f64,
    /// Cycle capacity fade (fraction).
    pub fade_cyc: f64,
    /// Capacity of this cell relative to the typical cell (manufacturing
    /// spread).
    pub cap_scale: f64,
    /// Resistance relative to the typical cell.
    pub r_scale: f64,
    /// Depth of the present discharge since the last charge (fraction).
    pub depth: f64,
    /// Discharged ampere-hours, lifetime total.
    pub ah_discharged: f64,
}

impl Cell {
    fn new(soc: f64) -> Cell {
        Cell {
            soc,
            v_rc: 0.0,
            fade_cal: 0.0,
            fade_cyc: 0.0,
            cap_scale: 1.0,
            r_scale: 1.0,
            depth: 0.0,
            ah_discharged: 0.0,
        }
    }

    /// Present capacity (Ah).
    pub fn capacity_ah(&self) -> f64 {
        CELL_CAPACITY_AH * self.cap_scale * (1.0 - self.fade_cal - self.fade_cyc).max(0.05)
    }

    fn r_aging(&self) -> f64 {
        self.r_scale * (1.0 + R_GROWTH_PER_FADE * (self.fade_cal + self.fade_cyc))
    }

    /// Ohmic resistance R₀ (Ω) at temperature factor `kt`.
    fn r0(&self, kt: f64) -> f64 {
        R0_25 * kt * (1.0 + 0.6 * (-self.soc.max(0.0) / 0.12).exp()) * self.r_aging()
    }
}

/// Temperature-dependent factors shared by the cells in one step.
#[derive(Clone, Copy, Debug)]
struct TempFactors {
    r: f64,
    cal: f64,
    cyc: f64,
    self_discharge: f64,
}

impl TempFactors {
    fn at(temp_c: f64) -> TempFactors {
        let cyc = if temp_c >= 25.0 { arrhenius(-EA_CYC, temp_c) } else { (0.04 * (25.0 - temp_c)).exp() };
        TempFactors {
            r: arrhenius(EA_R, temp_c),
            cal: arrhenius(-EA_CAL, temp_c),
            cyc,
            self_discharge: 2f64.powf((temp_c - 25.0) / 10.0),
        }
    }
}

/// The battery pack.
#[derive(Clone, Debug, PartialEq)]
pub struct Pack {
    /// The series cells.
    pub cells: [Cell; N_CELLS],
    /// Last temperature seen by [`Pack::step`] (°C).
    pub temp_c: f64,
    /// Time integrated (s).
    pub age_s: f64,
}

impl Pack {
    /// A new pack of identical cells at `soc` and 20 °C.
    pub fn new(soc: f64) -> Pack {
        Pack { cells: [Cell::new(soc); N_CELLS], temp_c: 20.0, age_s: 0.0 }
    }

    /// A new pack with a manufacturing spread (capacity σ 0.5 %, resistance
    /// σ 3 %, SOC σ 0.3 %, after the §9.1 matching).
    pub fn with_spread(soc: f64, rng: &mut Rng) -> Pack {
        let mut p = Pack::new(soc);
        for c in &mut p.cells {
            c.cap_scale = 1.0 + 0.005 * rng.normal();
            c.r_scale = 1.0 + 0.03 * rng.normal();
            c.soc = (soc + 0.003 * rng.normal()).clamp(0.0, 1.0);
        }
        p
    }

    /// Advance by `dt` seconds with pack current `current_a` (positive =
    /// charge) at cell temperature `temp_c`.
    pub fn step(&mut self, current_a: f64, dt: f64, temp_c: f64) {
        self.step_with_bleed(current_a, dt, temp_c, [0.0; N_CELLS]);
    }

    /// As [`Pack::step`], with a balancing bleed current (A, ≥ 0) drawn from
    /// each cell.
    pub fn step_with_bleed(&mut self, current_a: f64, dt: f64, temp_c: f64, bleed_a: [f64; N_CELLS]) {
        if dt <= 0.0 {
            return;
        }
        self.temp_c = temp_c;
        self.age_s += dt;
        let tf = TempFactors::at(temp_c);
        let r1 = R1_25 * tf.r;
        let decay = (-dt / (r1 * C1_F)).exp();
        for (c, &bleed) in self.cells.iter_mut().zip(bleed_a.iter()) {
            let i = current_a - bleed.max(0.0);
            let cap_as = c.capacity_ah() * 3600.0;
            let dq = if i > 0.0 { ETA_CHARGE * i * dt } else { i * dt };
            c.soc += dq / cap_as - SELF_DISCHARGE_PER_S * tf.self_discharge * dt;
            c.soc = c.soc.clamp(-0.02, 1.05);
            let r1c = r1 * c.r_aging();
            c.v_rc = c.v_rc * decay + i * r1c * (1.0 - decay);
            // calendar fade, equivalent-time integration of k √t
            let k = K_CAL_PER_SQRT_S * tf.cal * (0.4 + 0.6 * c.soc.clamp(0.0, 1.0));
            if k > 0.0 {
                let t_eq = (c.fade_cal / k).powi(2);
                c.fade_cal = k * (t_eq + dt).sqrt();
            }
            // cycle fade per discharged Ah, with the depth-of-discharge stress
            if i < 0.0 {
                let ah = -i * dt / 3600.0;
                let efc = ah / CELL_CAPACITY_AH;
                c.depth += efc;
                c.ah_discharged += ah;
                c.fade_cyc += G0_PER_EFC * (1.0 + 2.0 * DOD_STRESS * c.depth) * efc * tf.cyc;
            } else if i > 0.0 {
                c.depth = (c.depth - i * dt / 3600.0 / CELL_CAPACITY_AH).max(0.0);
            }
        }
    }

    /// Terminal voltage of each cell (V) at pack current `current_a`.
    pub fn cell_v(&self, current_a: f64) -> [f64; N_CELLS] {
        let kt = arrhenius(EA_R, self.temp_c);
        let mut v = [0.0; N_CELLS];
        for (out, c) in v.iter_mut().zip(self.cells.iter()) {
            *out = ocv(c.soc, self.temp_c) + c.v_rc + current_a * c.r0(kt);
        }
        v
    }

    /// Pack terminal voltage (V) at pack current `current_a` (positive =
    /// charge).
    pub fn terminal_v(&self, current_a: f64) -> f64 {
        self.cell_v(current_a).iter().sum()
    }

    /// Pack open-circuit voltage (V).
    pub fn ocv(&self) -> f64 {
        self.cells.iter().map(|c| ocv(c.soc, self.temp_c)).sum()
    }

    /// Mean state of charge of the cells (fraction).
    pub fn soc(&self) -> f64 {
        self.cells.iter().map(|c| c.soc).sum::<f64>() / N_CELLS as f64
    }

    /// Lowest cell state of charge.
    pub fn soc_min(&self) -> f64 {
        self.cells.iter().map(|c| c.soc).fold(f64::INFINITY, f64::min)
    }

    /// Present pack capacity (Ah): that of the weakest cell.
    pub fn capacity_ah(&self) -> f64 {
        self.cells.iter().map(Cell::capacity_ah).fold(f64::INFINITY, f64::min)
    }

    /// Stored energy available (Wh) at the present state of charge, nominal
    /// voltage.
    pub fn energy_wh(&self) -> f64 {
        self.capacity_ah() * self.soc().max(0.0) * CELL_NOMINAL_V * N_CELLS as f64
    }

    /// State of health: present capacity / typical new capacity.
    pub fn soh(&self) -> f64 {
        self.capacity_ah() / CELL_CAPACITY_AH
    }

    /// Pack ohmic resistance (Ω) at the present temperature.
    pub fn resistance(&self) -> f64 {
        let kt = arrhenius(EA_R, self.temp_c);
        self.cells.iter().map(|c| c.r0(kt)).sum()
    }

    /// Heat generated in the pack (W) at current `current_a`: I²R₀ + the
    /// RC-pair loss + the reversible (entropic) heat I T dU/dT.
    pub fn heat_w(&self, current_a: f64) -> f64 {
        let kt = arrhenius(EA_R, self.temp_c);
        let r1 = R1_25 * kt;
        let t_k = self.temp_c + 273.15;
        self.cells
            .iter()
            .map(|c| {
                current_a * current_a * c.r0(kt)
                    + c.v_rc * c.v_rc / (r1 * c.r_aging())
                    + current_a * t_k * entropic(c.soc)
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocv_table() {
        let v50 = ocv_25(0.5);
        assert!((3.6..=3.75).contains(&v50), "{v50}");
        assert!((ocv_25(1.0) - 4.19).abs() < 0.01);
        assert!(ocv_25(0.0) < 3.0);
        for k in 1..SOC_TAB.len() {
            assert!(OCV_TAB[k] > OCV_TAB[k - 1] && SOC_TAB[k] > SOC_TAB[k - 1]);
        }
        let p = Pack::new(0.5);
        assert!((p.ocv() - 4.0 * ocv(0.5, 20.0)).abs() < 1e-12);
        assert!((p.capacity_ah() * 4.0 * CELL_NOMINAL_V - 49.7).abs() < 0.1);
    }

    #[test]
    fn coulomb_counting_and_voltage_under_load() {
        let mut p = Pack::new(1.0);
        for _ in 0..3600 {
            p.step(-1.0, 1.0, 25.0);
        }
        let expected = 1.0 - 1.0 / 3.45;
        assert!((p.soc() - expected).abs() < 0.002, "{}", p.soc());
        // under discharge the terminal voltage sags below the OCV
        assert!(p.terminal_v(-1.0) < p.ocv() - 0.15);
        assert!(p.heat_w(-1.0) > 0.15);
        // cold doubles the resistance before 0 °C
        let mut cold = Pack::new(0.5);
        cold.step(0.0, 1.0, -10.0);
        let mut warm = Pack::new(0.5);
        warm.step(0.0, 1.0, 25.0);
        assert!(cold.resistance() > 2.0 * warm.resistance());
        let r = warm.resistance() / 4.0;
        assert!((0.030..=0.060).contains(&r), "{r}");
    }

    #[test]
    fn rc_pair_relaxes() {
        let mut p = Pack::new(0.6);
        for _ in 0..600 {
            p.step(-2.0, 1.0, 25.0);
        }
        let loaded = p.cells[0].v_rc;
        assert!(loaded < -0.02);
        for _ in 0..300 {
            p.step(0.0, 1.0, 25.0);
        }
        assert!(p.cells[0].v_rc.abs() < 1e-3 * loaded.abs() * 100.0);
    }

    /// Fade after `years` of LEO cycling: 35.8 min eclipse discharge to the
    /// depth `dod`, 58.8 min recharge at ≤ 0.40 A (array-limited, §4.3) back
    /// to the start, at `temp_c`. Returns the capacity fade (fraction).
    fn leo_fade(dod: f64, temp_c: f64, years: f64) -> (f64, f64, f64) {
        let mut p = Pack::new(0.98);
        let dt = 60.0;
        let i_dis = dod * CELL_CAPACITY_AH / (35.8 / 60.0);
        let orbits = (years * 365.25 * 1440.0 / 94.6) as u32;
        for _ in 0..orbits {
            for _ in 0..36 {
                p.step(-i_dis, dt, temp_c);
            }
            for _ in 0..59 {
                let i = if p.soc() < 0.98 { 0.40 } else { 0.0 };
                p.step(i, dt, temp_c);
            }
        }
        let c = p.cells[0];
        (1.0 - p.soh(), c.fade_cal, c.fade_cyc)
    }

    #[test]
    fn capacity_fade_after_two_years_of_leo_cycling() {
        let (f10, cal10, cyc10) = leo_fade(0.10, 20.0, 2.0);
        let (f1, cal1, cyc1) = leo_fade(0.013, 20.0, 2.0);
        println!(
            "2-yr fade at 20 °C: 10 % DoD {:.1} % (calendar {:.1} %, cycle {:.1} %); \
             1.3 % DoD {:.1} % (calendar {:.1} %, cycle {:.1} %)",
            f10 * 100.0,
            cal10 * 100.0,
            cyc10 * 100.0,
            f1 * 100.0,
            cal1 * 100.0,
            cyc1 * 100.0
        );
        assert!((0.03..=0.11).contains(&f10), "{f10}");
        assert!(f1 < f10 && f1 > 0.01);
    }

    #[test]
    fn spread_and_bleed() {
        let mut rng = Rng::new(5);
        let mut p = Pack::with_spread(0.8, &mut rng);
        let before = p.cells[2].soc;
        p.step_with_bleed(0.0, 3600.0, 20.0, [0.0, 0.0, 0.1, 0.0]);
        assert!(p.cells[2].soc < before - 0.02);
        assert!(p.soc_min() <= p.soc());
        assert!(p.energy_wh() > 30.0);
    }
}
