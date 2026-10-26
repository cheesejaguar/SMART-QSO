//! What the run measures, step by step: one [`Sample`] of the whole
//! spacecraft each step, folded into per-orbit statistics and the
//! whole-mission totals the checks (`crate::checks`) judge.

/// The state of the spacecraft and the flight software at one step
#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    /// Seconds since deployment (negative before)
    pub t_s: f64,
    /// Step length, s
    pub dt: f64,
    /// Mean altitude, km
    pub alt_km: f64,
    /// Sun illumination, 0–1
    pub illum: f64,
    /// Beta angle, degrees
    pub beta_deg: f64,
    /// Latitude, longitude, degrees
    pub lat_deg: f64,
    /// Longitude, degrees
    pub lon_deg: f64,
    /// Body rate, degrees/s
    pub rate_dps: f64,
    /// Angle between +X and the sun, degrees
    pub sun_angle_deg: f64,
    /// Array power available (MPP), W
    pub array_mpp_w: f64,
    /// Array power drawn, W
    pub array_w: f64,
    /// Load at the pack, W
    pub load_w: f64,
    /// Battery current, A (+ charging)
    pub batt_a: f64,
    /// Pack voltage, V
    pub vbat: f64,
    /// True state of charge, 0–1
    pub soc: f64,
    /// The gauge's SOC, 0–1
    pub soc_gauge: f64,
    /// The flight software's SOC, 0–1
    pub soc_fsw: f64,
    /// Pack capacity now, Ah
    pub capacity_ah: f64,
    /// Battery temperature, °C
    pub t_batt: f64,
    /// OBC/EPS board, °C
    pub t_obc: f64,
    /// PA, °C
    pub t_pa: f64,
    /// Jetson module, °C
    pub t_jetson: f64,
    /// Coldest and hottest outer panel, °C
    pub t_panel_min: f64,
    /// Hottest outer panel, °C
    pub t_panel_max: f64,
    /// Charging wanted but blocked by temperature
    pub charge_blocked: bool,
    /// OBC powered
    pub obc_on: bool,
    /// Operational state (BOOT … EMERGENCY)
    pub op_state: u32,
    /// Power mode
    pub power_mode: u32,
    /// Payload switched on
    pub payload_on: bool,
    /// Jetson power, W
    pub jetson_w: f64,
    /// Radio rail on
    pub radio_on: bool,
    /// PA rail on
    pub pa_on: bool,
    /// Transmitter keyed / radiating
    pub keyed: bool,
    /// Radiating RF
    pub radiating: bool,
    /// Heater on
    pub heater_on: bool,
    /// ADCS rail on
    pub adcs_on: bool,
    /// Magnetorquer dipole magnitude, A·m²
    pub dipole: f64,
    /// Antennas / wings deployed
    pub antennas: bool,
    /// Wings deployed
    pub wings: bool,
    /// A ground station sees the spacecraft above 0° (best elevation, degrees)
    pub best_elev_deg: f64,
    /// RF silence active (flight software)
    pub rf_silence: bool,
    /// Loss-of-contact inhibit active
    pub contact_inhibit: bool,
    /// TX permitted
    pub tx_permitted: bool,
}

/// One orbit's statistics
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    /// Orbit number (from 1)
    pub n: u64,
    /// Start, s after deployment
    pub start_s: f64,
    /// Length, s
    pub len_s: f64,
    /// Eclipse time, s
    pub eclipse_s: f64,
    /// Energy from the array, Wh
    pub gen_wh: f64,
    /// Energy to the loads, Wh
    pub load_wh: f64,
    /// SOC extremes
    pub soc_min: f64,
    /// Highest SOC
    pub soc_max: f64,
    /// SOC at the start
    pub soc_start: f64,
    /// Depth of discharge in the orbit (max − min SOC within its eclipse)
    pub dod: f64,
    /// Battery temperature extremes
    pub tb_min: f64,
    /// Hottest battery
    pub tb_max: f64,
    /// PA hottest
    pub tpa_max: f64,
    /// Jetson hottest
    pub tj_max: f64,
    /// Seconds radiating
    pub radiating_s: f64,
    /// Payload on, s
    pub payload_s: f64,
    /// Heater on, s
    pub heater_s: f64,
    /// Charge blocked by temperature, s
    pub blocked_s: f64,
    /// Mode changes in the orbit
    pub mode_changes: u32,
    /// Mean beta, degrees
    pub beta_deg: f64,
    /// OBC off, s
    pub obc_off_s: f64,
    /// Mean body rate, degrees/s
    pub rate_dps: f64,
    soc_ecl_max: f64,
    soc_ecl_min: f64,
    last_mode: Option<u32>,
    beta_sum: f64,
    rate_sum: f64,
}

impl Orbit {
    fn new(n: u64, s: &Sample) -> Orbit {
        Orbit {
            n,
            start_s: s.t_s,
            len_s: 0.0,
            eclipse_s: 0.0,
            gen_wh: 0.0,
            load_wh: 0.0,
            soc_min: s.soc,
            soc_max: s.soc,
            soc_start: s.soc,
            dod: 0.0,
            tb_min: s.t_batt,
            tb_max: s.t_batt,
            tpa_max: s.t_pa,
            tj_max: s.t_jetson,
            radiating_s: 0.0,
            payload_s: 0.0,
            heater_s: 0.0,
            blocked_s: 0.0,
            mode_changes: 0,
            beta_deg: 0.0,
            obc_off_s: 0.0,
            rate_dps: 0.0,
            soc_ecl_max: f64::NAN,
            soc_ecl_min: f64::NAN,
            last_mode: None,
            beta_sum: 0.0,
            rate_sum: 0.0,
        }
    }

    fn add(&mut self, s: &Sample) {
        let dt = s.dt;
        self.len_s += dt;
        if s.illum < 0.5 {
            self.eclipse_s += dt;
            self.soc_ecl_max = if self.soc_ecl_max.is_nan() { s.soc } else { self.soc_ecl_max.max(s.soc) };
            self.soc_ecl_min = if self.soc_ecl_min.is_nan() { s.soc } else { self.soc_ecl_min.min(s.soc) };
        }
        self.gen_wh += s.array_w * dt / 3600.0;
        self.load_wh += s.load_w * dt / 3600.0;
        self.soc_min = self.soc_min.min(s.soc);
        self.soc_max = self.soc_max.max(s.soc);
        self.tb_min = self.tb_min.min(s.t_batt);
        self.tb_max = self.tb_max.max(s.t_batt);
        self.tpa_max = self.tpa_max.max(s.t_pa);
        self.tj_max = self.tj_max.max(s.t_jetson);
        if s.radiating {
            self.radiating_s += dt;
        }
        if s.payload_on {
            self.payload_s += dt;
        }
        if s.heater_on {
            self.heater_s += dt;
        }
        if s.charge_blocked {
            self.blocked_s += dt;
        }
        if !s.obc_on {
            self.obc_off_s += dt;
        }
        if s.obc_on {
            if self.last_mode.is_some_and(|m| m != s.op_state) {
                self.mode_changes += 1;
            }
            self.last_mode = Some(s.op_state);
        }
        self.beta_sum += s.beta_deg * dt;
        self.rate_sum += s.rate_dps * dt;
    }

    fn close(&mut self) {
        self.dod = if self.soc_ecl_max.is_nan() { 0.0 } else { self.soc_ecl_max - self.soc_ecl_min };
        if self.len_s > 0.0 {
            self.beta_deg = self.beta_sum / self.len_s;
            self.rate_dps = self.rate_sum / self.len_s;
        }
    }

    /// CSV header of [`Orbit::row`]
    pub const HEADER: &'static [&'static str] = &[
        "orbit",
        "start_s",
        "len_s",
        "eclipse_s",
        "beta_deg",
        "gen_wh",
        "load_wh",
        "soc_start",
        "soc_min",
        "soc_max",
        "dod",
        "tbatt_min",
        "tbatt_max",
        "tpa_max",
        "tjetson_max",
        "radiating_s",
        "payload_s",
        "heater_s",
        "charge_blocked_s",
        "mode_changes",
        "obc_off_s",
        "rate_dps",
    ];

    /// The row for `orbits.csv`
    pub fn row(&self) -> Vec<f64> {
        vec![
            self.n as f64,
            self.start_s,
            self.len_s,
            self.eclipse_s,
            self.beta_deg,
            self.gen_wh,
            self.load_wh,
            self.soc_start,
            self.soc_min,
            self.soc_max,
            self.dod,
            self.tb_min,
            self.tb_max,
            self.tpa_max,
            self.tj_max,
            self.radiating_s,
            self.payload_s,
            self.heater_s,
            self.blocked_s,
            f64::from(self.mode_changes),
            self.obc_off_s,
            self.rate_dps,
        ]
    }
}

/// Per-orbit folding: orbits start at each ascending-node crossing
#[derive(Debug, Default)]
pub struct Orbits {
    current: Option<Orbit>,
    last_lat: f64,
    /// Orbits completed
    pub done: Vec<Orbit>,
}

impl Orbits {
    /// Add a sample; a completed orbit is returned (and kept)
    pub fn add(&mut self, s: &Sample) -> Option<Orbit> {
        let ascending = self.last_lat < 0.0 && s.lat_deg >= 0.0;
        self.last_lat = s.lat_deg;
        let mut finished = None;
        if ascending {
            if let Some(mut o) = self.current.take() {
                o.close();
                self.done.push(o);
                finished = Some(o);
            }
            let n = self.done.len() as u64 + 1;
            self.current = Some(Orbit::new(n, s));
        }
        if let Some(o) = self.current.as_mut() {
            o.add(s);
        }
        finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbits_split_at_ascending_node_and_measure_dod() {
        let mut o = Orbits::default();
        let mut finished = 0;
        for k in 0..20_000 {
            let t = f64::from(k);
            let phase = (t / 5600.0) * std::f64::consts::TAU;
            let s = Sample {
                t_s: t,
                dt: 1.0,
                lat_deg: 90.0 * phase.sin(),
                illum: if phase.cos() > -0.5 { 1.0 } else { 0.0 },
                soc: 0.9 - 0.05 * (phase.cos() < -0.5) as u8 as f64,
                ..Sample::default()
            };
            if o.add(&s).is_some() {
                finished += 1;
            }
        }
        assert_eq!(finished, 2);
        assert!(o.done[1].eclipse_s > 1500.0);
        assert!((o.done[1].len_s - 5600.0).abs() < 2.0);
    }
}
