//! The EPS electronics between the array, the 4S pack and the loads
//! (SQSO-HW-DIAG-001 B2, S1, S2; BASELINE §5; `hardware/eps/PDU_DESIGN.md`):
//!
//! - the MPPT battery charge regulator: tracks the array's maximum power,
//!   charges at most 0.40 A (constant current, then constant voltage at
//!   4.10 V a cell), and only between 0 °C and 45 °C; outside that window,
//!   or with the pack full, it supplies the loads only;
//! - the EOL passivation latch: once fired, the array is disconnected for
//!   good and the loads drain the pack;
//! - the BMS: cuts the discharge path at 2.8 V a cell (the bus, and the
//!   OBC, go dark) and reconnects at 3.0 V; cuts at 4 A, below the 5 A pack
//!   fuse so the resettable protection acts first (`docs/hardware/EPS_DESIGN.md`
//!   v2.0 §5; the largest load, all four burn resistors at their 0.8 A limit
//!   by a software fault, is 3.3 A);
//! - the 3V3_AO and 5 V bucks and the load switches;
//! - the controller's telemetry (SMART-QSO-ICD-001 §4.2) with a coulomb-
//!   counting fuel gauge that learns the capacity at each full charge.

use sqso_fsw::eps_telemetry::{Telemetry, status};

use crate::env::battery::Pack;

/// MPPT tracking and conversion efficiency
pub const MPPT_EFF: f64 = 0.92;
/// Array-side loss of the blocking diodes and harness (fraction of power)
pub const DIODE_LOSS: f64 = 0.03;
/// Charge current limit, A (BASELINE §5)
pub const CHARGE_LIMIT_A: f64 = 0.40;
/// Constant-voltage set point, V a cell: 4.10 V, not the cell's 4.20 V
/// maximum, because the pack sits at the end of charge for most of the
/// mission (BASELINE §5; `docs/hardware/BUILD_VS_BUY.md` §7)
pub const CV_CELL_V: f64 = 4.10;
/// Charge temperature window, °C (BASELINE §5, SYS-602)
pub const CHARGE_WINDOW_C: (f64, f64) = (0.0, 45.0);
/// BMS under-voltage cut-off and reconnect, V a cell (BASELINE §5)
pub const UV_CUT_V: f64 = 2.8;
/// Reconnect after an under-voltage cut-off, V a cell (open circuit)
pub const UV_RECONNECT_V: f64 = 3.0;
/// BMS over-current cut-off, A (below the 5 A pack fuse)
pub const OC_A: f64 = 4.0;
/// 3V3_AO buck efficiency at its light load
pub const BUCK_3V3_EFF: f64 = 0.85;
/// 5 V buck efficiency
pub const BUCK_5V_EFF: f64 = 0.90;
/// EPS controller, gauge and monitors, W
pub const QUIESCENT_W: f64 = 0.08;
/// Cells in series
pub const CELLS: f64 = 4.0;

/// Loads this step, at their rails
#[derive(Clone, Copy, Debug, Default)]
pub struct Loads {
    /// On the 3V3_AO rail and the switched 3.3 V rails, W
    pub w_3v3: f64,
    /// On the 5 V rail other than the payload (PA), W
    pub w_5v: f64,
    /// The payload (Jetson) on the 5 V rail, W
    pub w_payload: f64,
    /// Straight from the pack (heater), W
    pub w_vbat: f64,
    /// Burn-wire current straight from the pack, A
    pub burn_a: f64,
}

/// The fuel gauge
#[derive(Clone, Copy, Debug)]
pub struct Gauge {
    /// Estimated charge, Ah
    pub charge_ah: f64,
    /// Learned capacity, Ah
    pub capacity_ah: f64,
    /// Current-sense offset, A (a fixed error)
    pub offset_a: f64,
}

impl Gauge {
    /// SOC estimate, 0–1
    pub fn soc(&self) -> f64 {
        (self.charge_ah / self.capacity_ah).clamp(0.0, 1.0)
    }
}

/// What the EPS did this step
#[derive(Clone, Copy, Debug, Default)]
pub struct EpsOut {
    /// Array power at its maximum power point, W
    pub array_mpp_w: f64,
    /// Array power actually drawn, W
    pub array_w: f64,
    /// Load power at the pack's terminals, W
    pub load_w: f64,
    /// Battery current, A (+ charging)
    pub batt_a: f64,
    /// Pack terminal voltage, V
    pub vbat: f64,
    /// Charging was wanted but blocked by temperature
    pub charge_blocked_cold: bool,
    /// Charging blocked hot
    pub charge_blocked_hot: bool,
    /// The bus is up (the BMS has the discharge path closed)
    pub bus_up: bool,
    /// Conversion losses dissipated on the EPS board, W
    pub loss_w: f64,
    /// Heat in the pack, W
    pub pack_heat_w: f64,
}

/// The EPS
#[derive(Clone, Debug)]
pub struct Eps {
    /// The battery
    pub pack: Pack,
    /// The fuel gauge
    pub gauge: Gauge,
    /// The EOL passivation latch has fired
    pub passivated: bool,
    /// The BMS has cut the discharge path (under-voltage)
    pub uv_cut: bool,
    /// The BMS has cut for over-current (latched until the load drops)
    pub oc_trips: u32,
    /// Under-voltage cut-offs so far
    pub uv_trips: u32,
    /// Last telemetry
    pub telemetry: Telemetry,
    /// Charge throughput, Ah
    pub ah_in: f64,
    /// Discharge throughput, Ah
    pub ah_out: f64,
    /// Constant-voltage set point, V a cell ([`CV_CELL_V`] by default)
    pub cv_cell_v: f64,
}

impl Eps {
    /// With `pack` at its present charge; the gauge starts right
    pub fn new(pack: Pack, gauge_offset_a: f64) -> Eps {
        let capacity = pack.capacity_ah();
        let gauge = Gauge { charge_ah: pack.soc() * capacity, capacity_ah: capacity, offset_a: gauge_offset_a };
        Eps {
            pack,
            gauge,
            passivated: false,
            uv_cut: false,
            oc_trips: 0,
            uv_trips: 0,
            telemetry: Telemetry::default(),
            ah_in: 0.0,
            ah_out: 0.0,
            cv_cell_v: CV_CELL_V,
        }
    }

    /// Advance `dt` s with `array_mpp_w` available at the array's maximum
    /// power point (array voltage `array_v`), the loads, the pack at
    /// `batt_c`; `sunlit` is the controller's sun flag (array power above
    /// its threshold)
    pub fn step(&mut self, dt: f64, array_mpp_w: f64, array_v: f64, loads: &Loads, batt_c: f64) -> EpsOut {
        let mut out = EpsOut { array_mpp_w, ..EpsOut::default() };
        let available = if self.passivated { 0.0 } else { array_mpp_w * (1.0 - DIODE_LOSS) * MPPT_EFF };
        let bus_load = loads.w_3v3 / BUCK_3V3_EFF + (loads.w_5v + loads.w_payload) / BUCK_5V_EFF + loads.w_vbat;
        let bus_load = if self.uv_cut { 0.0 } else { bus_load + QUIESCENT_W };
        let ocv = self.pack.ocv();
        let cell_ocv = ocv / CELLS;
        let window = batt_c >= CHARGE_WINDOW_C.0 && batt_c <= CHARGE_WINDOW_C.1;
        // Constant current, tapering to the CV set point
        let taper = ((self.cv_cell_v - cell_ocv) / 0.05).clamp(0.0, 1.0);
        let charge_max = if window && !self.passivated { CHARGE_LIMIT_A * taper.max(0.02) } else { 0.0 };
        // Battery current from the power balance at the pack (one fixed-point
        // pass on the terminal voltage)
        let mut v = ocv.max(1.0);
        let mut i = 0.0;
        for _ in 0..2 {
            let surplus = available - bus_load;
            i = (surplus / v).min(charge_max) - if self.uv_cut { 0.0 } else { loads.burn_a };
            v = self.pack.terminal_v(i).max(1.0);
        }
        if available - bus_load > 0.0 && charge_max <= 0.0 && !self.passivated {
            out.charge_blocked_cold = batt_c < CHARGE_WINDOW_C.0;
            out.charge_blocked_hot = batt_c > CHARGE_WINDOW_C.1;
        }
        // BMS: over-current and under-voltage
        if -i > OC_A {
            self.oc_trips += 1;
            i = 0.0;
            self.uv_cut = true; // the discharge FET opens; the bus drops
        }
        if !self.uv_cut && v / CELLS < UV_CUT_V {
            self.uv_cut = true;
            self.uv_trips += 1;
        } else if self.uv_cut && cell_ocv >= UV_RECONNECT_V && i >= 0.0 {
            self.uv_cut = false;
        }
        if self.uv_cut {
            i = i.max(0.0);
        }
        self.pack.step(i, dt, batt_c);
        if i > 0.0 {
            self.ah_in += i * dt / 3600.0;
        } else {
            self.ah_out -= i * dt / 3600.0;
        }
        out.batt_a = i;
        out.vbat = self.pack.terminal_v(i);
        out.bus_up = !self.uv_cut;
        out.load_w = if self.uv_cut { 0.0 } else { bus_load + loads.burn_a * out.vbat };
        out.array_w = if self.passivated {
            0.0
        } else {
            ((out.load_w - loads.burn_a * out.vbat + i.max(0.0) * out.vbat) / ((1.0 - DIODE_LOSS) * MPPT_EFF))
                .min(array_mpp_w)
                .max(0.0)
        };
        out.loss_w = out.array_w * (1.0 - MPPT_EFF)
            + loads.w_3v3 * (1.0 / BUCK_3V3_EFF - 1.0)
            + (loads.w_5v + loads.w_payload) * (1.0 / BUCK_5V_EFF - 1.0)
            + QUIESCENT_W;
        out.pack_heat_w = self.pack.heat_w(i);
        self.gauge_step(dt, i, cell_ocv);
        self.telemetry = self.telemetry_now(&out, array_v, loads, batt_c);
        out
    }

    fn gauge_step(&mut self, dt: f64, i: f64, cell_ocv: f64) {
        let g = &mut self.gauge;
        g.charge_ah += (i + g.offset_a) * dt / 3600.0;
        // Full charge (CV taper, current down to a trickle): learn the
        // capacity, and reset the count to the charge that the end-of-charge
        // voltage implies on the cell's OCV curve (all of it at 4.20 V, about
        // 90 % at 4.10 V)
        if cell_ocv > self.cv_cell_v - 0.01 && (0.0..0.02).contains(&i) {
            g.capacity_ah = 0.9 * g.capacity_ah + 0.1 * self.pack.capacity_ah();
            g.charge_ah = g.capacity_ah * self.pack.soc();
        }
        g.charge_ah = g.charge_ah.clamp(0.0, g.capacity_ah);
    }

    fn telemetry_now(&self, out: &EpsOut, array_v: f64, loads: &Loads, batt_c: f64) -> Telemetry {
        let mut st = 0u8;
        if out.batt_a > 0.005 {
            st |= status::CHARGING;
        }
        if out.array_mpp_w > 0.3 && !self.passivated {
            st |= status::SUNLIT;
        }
        if self.gauge.soc() < 0.25 {
            st |= status::LOW_BATT;
        }
        if batt_c > CHARGE_WINDOW_C.1 {
            st |= status::OVER_TEMP;
        }
        if batt_c < CHARGE_WINDOW_C.0 {
            st |= status::UNDER_TEMP;
        }
        if self.uv_cut || self.oc_trips > 0 {
            st |= status::FAULT;
        }
        Telemetry {
            status: st,
            soc: self.gauge.soc(),
            vbatt_v: out.vbat,
            ibatt_a: out.batt_a,
            tbatt_c: batt_c,
            vsolar_v: if out.array_mpp_w > 0.05 { array_v } else { 0.2 },
            isolar_a: if array_v > 0.5 { out.array_w / array_v } else { 0.0 },
            vbus_v: if self.uv_cut { 0.0 } else { 5.02 },
            ibus_a: loads.w_payload / 5.02,
        }
    }
}
