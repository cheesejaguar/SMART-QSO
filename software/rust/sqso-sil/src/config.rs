//! The mission the simulation flies: dates, orbit, spacecraft options,
//! operations plan and the simulation's own settings. Every value a run
//! used is written to `config.json` beside its results.
//!
//! Defaults follow `docs/BASELINE_PARAMETERS.md`; `--set key=value` on the
//! command line changes any of them (the keys are the field names).

use std::fmt::Write as _;

/// Settings of one run
#[derive(Clone, Debug)]
pub struct Config {
    /// Scenario name, for the record
    pub scenario: String,
    /// Random seed (keys, unit-to-unit variation, radiation, link fading)
    pub seed: u64,
    /// Integration and key load (bench), UTC as (y, m, d)
    pub integration_date: (i32, u32, u32),
    /// Deployment from the dispenser, UTC (y, m, d, h) (BASELINE §3: the
    /// planning launch, the last Transporter flight of 2028, about 2028-11)
    pub deploy_utc: (i32, u32, u32, f64),
    /// Deployment altitude, km (BASELINE §3: target 475 km, 465–485 km)
    pub altitude_km: f64,
    /// Local time of the ascending node, h (Transporter SSO: descending
    /// node 10:30, so ascending 22:30)
    pub ltan_h: f64,
    /// Pack state of charge at integration (BASELINE §4: 60 % ± 5 % at the
    /// integrator, about 47 days before launch)
    pub integration_soc: f64,
    /// Storage temperature between integration and deployment, °C
    pub storage_temp_c: f64,
    /// Design life after deployment, days (BASELINE §2: 2 years)
    pub mission_days: f64,
    /// Fire the EOL passivation at the end of the design life
    pub passivate_at_eol: bool,
    /// Simulate the orbit (without the flight software, which is off for
    /// good after passivation) until re-entry
    pub to_reentry: bool,
    /// Simulation step with nothing happening, s
    pub step_s: f64,
    /// Finer step while the transmitter is keyed, a burn is on, or the
    /// Jetson is booting, s
    pub fine_step_s: f64,
    /// Time-series record interval, s
    pub record_s: f64,
    /// Finer record interval for the first `detail_days`, s
    pub detail_record_s: f64,
    /// Days recorded at the finer interval
    pub detail_days: f64,
    /// Burn-wire driver per-resistor current limit, A (`0`: direct drive)
    pub burn_limit_a: f64,
    /// Contact plan: commands per day from the primary station (passes
    /// above `contact_min_elev_deg`)
    pub contacts_per_day: u32,
    /// Lowest elevation for a commanding pass, degrees
    pub contact_min_elev_deg: f64,
    /// Day after deployment the loss-of-contact timeout is set to 14 days
    /// (BASELINE §7: end of commissioning)
    pub commissioning_days: f64,
    /// Fuel-gauge current offset, A (a fixed error)
    pub gauge_offset_a: f64,
    /// Radiation effects on (single-event resets and latch-ups)
    pub radiation: bool,
    /// Days of ground silence to test the loss-of-contact inhibit (0: none)
    pub contact_outage_days: f64,
    /// When that outage starts, days after deployment
    pub contact_outage_start_day: f64,
    /// Battery capacity relative to the 4S1P INR18650-35E pack (resistance
    /// scales inversely)
    pub battery_scale: f64,
    /// Charge regulator's constant-voltage set point, V a cell
    pub charge_v_cell: f64,
    /// Contingency: the magnetorquers never act (the spacecraft tumbles)
    pub adcs_failed: bool,
    /// Contingency: the wings never release
    pub wings_failed: bool,
    /// The end-of-life fuse's heater, Ω (F1: 11.2–20.0 Ω, 15.6 nominal)
    pub eol_heater_ohms: f64,
    /// Contingency: the A resistors never part their lines (the B resistors
    /// of the second attempt do: the four-line sequence)
    pub wing_a_failed: bool,
    /// Separate at the first umbra entry after `deploy_utc` (the AntS
    /// temperature gate's case, decision C-18)
    pub separate_at_eclipse: bool,
    /// The dispenser cold soak: the whole thermal network 2 h in the
    /// dispenser before separation, its wall ending at this temperature, °C
    /// (THERMAL_ANALYSIS 2.4 §4.3 and §4.4, case D3: −20 °C, the AntS at
    /// −18 °C at separation; `Thermal::dispenser_soak`); `None`: every
    /// node at the storage temperature (Version E set only the structure and
    /// the AntS to this temperature)
    pub separation_soak_c: Option<f64>,
    /// Contingency: the Jetson's rail is stuck on (its switch failed closed)
    pub payload_stuck_on: bool,
    /// Years of array radiation degradation added at deployment (0: new)
    pub array_years: f64,
    /// The gyro (`crate::adcs_mc::gyro_preset`: none, mg370 (the flight
    /// gyro, BASELINE §8), bmi088 (the backup, calibrated), bmi088_uncal,
    /// adis16505)
    pub gyro: String,
    /// Contingency: the command receiver fails for good this many days after
    /// deployment (0: never): no frames, no carrier detect, its health line
    /// high (REL-04)
    pub rx_failed_day: f64,
    /// Single-event functional interrupts that hang the command receiver
    /// until its power is cycled (FSW-09)
    pub rx_sefi: bool,
    /// Days between the ground's dumps of stored housekeeping (0: none)
    pub dump_every_days: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scenario: "full".into(),
            seed: 20_280_601,
            // Storage from the integrator's final charge, L-47 d (BASELINE §4)
            integration_date: (2028, 9, 15),
            deploy_utc: (2028, 11, 1, 6.0),
            altitude_km: 475.0,
            ltan_h: 22.5,
            integration_soc: 0.60,
            storage_temp_c: 20.0,
            mission_days: 730.0,
            passivate_at_eol: true,
            to_reentry: true,
            step_s: 1.0,
            fine_step_s: 0.1,
            record_s: 600.0,
            detail_record_s: 10.0,
            detail_days: 3.0,
            burn_limit_a: 0.8,
            contacts_per_day: 2,
            contact_min_elev_deg: crate::env::link::COMMAND_MIN_ELEV_DEG,
            commissioning_days: 14.0,
            gauge_offset_a: 0.002,
            radiation: true,
            contact_outage_days: 0.0,
            contact_outage_start_day: 0.0,
            battery_scale: 1.0,
            charge_v_cell: crate::hw::eps::CV_CELL_V,
            adcs_failed: false,
            wings_failed: false,
            wing_a_failed: false,
            separate_at_eclipse: false,
            separation_soak_c: None,
            eol_heater_ohms: crate::mission::EOL_HEATER_OHMS,
            payload_stuck_on: false,
            array_years: 0.0,
            gyro: "mg370".into(),
            rx_failed_day: 0.0,
            rx_sefi: true,
            dump_every_days: 1.0,
        }
    }
}

impl Config {
    /// A named scenario: `full` (integration to re-entry), `commissioning`
    /// (the first 14 days), `ants_cold` (the first 14 days after a cold
    /// soak and an eclipse separation), `smoke` (the first day, for CI),
    /// `year` (the first year, no passivation)
    pub fn scenario(name: &str) -> Option<Config> {
        let mut c = Config { scenario: name.into(), ..Config::default() };
        match name {
            "full" => {}
            "year" => {
                c.mission_days = 365.0;
                c.passivate_at_eol = false;
                c.to_reentry = false;
            }
            "commissioning" => {
                c.mission_days = 14.0;
                c.passivate_at_eol = false;
                c.to_reentry = false;
                c.record_s = 60.0;
            }
            "ants_cold" => {
                // The AntS contingency (decisions C-18, C-19): the first 14
                // days after an eclipse separation that follows the
                // analysis's 2 h dispenser cold soak (THERMAL_ANALYSIS 2.4
                // §4.4, case D3: the wall at −20 °C)
                c.mission_days = 14.0;
                c.passivate_at_eol = false;
                c.to_reentry = false;
                c.record_s = 60.0;
                c.separate_at_eclipse = true;
                c.separation_soak_c = Some(-20.0);
            }
            "smoke" => {
                c.mission_days = 1.0;
                c.passivate_at_eol = false;
                c.to_reentry = false;
                c.record_s = 60.0;
                c.detail_days = 1.0;
            }
            _ => return None,
        }
        Some(c)
    }

    /// Apply `key=value`
    pub fn set(&mut self, kv: &str) -> Result<(), String> {
        let (k, v) = kv.split_once('=').ok_or_else(|| format!("expected key=value, got {kv}"))?;
        let f = || v.parse::<f64>().map_err(|e| format!("{k}: {e}"));
        let b = || v.parse::<bool>().map_err(|e| format!("{k}: {e}"));
        match k {
            "seed" => self.seed = v.parse().map_err(|e| format!("{k}: {e}"))?,
            "altitude_km" => self.altitude_km = f()?,
            "ltan_h" => self.ltan_h = f()?,
            "integration_soc" => self.integration_soc = f()?,
            "storage_temp_c" => self.storage_temp_c = f()?,
            "mission_days" => self.mission_days = f()?,
            "passivate_at_eol" => self.passivate_at_eol = b()?,
            "to_reentry" => self.to_reentry = b()?,
            "step_s" => self.step_s = f()?,
            "fine_step_s" => self.fine_step_s = f()?,
            "record_s" => self.record_s = f()?,
            "detail_record_s" => self.detail_record_s = f()?,
            "detail_days" => self.detail_days = f()?,
            "burn_limit_a" => self.burn_limit_a = f()?,
            "contacts_per_day" => self.contacts_per_day = v.parse().map_err(|e| format!("{k}: {e}"))?,
            "contact_min_elev_deg" => self.contact_min_elev_deg = f()?,
            "commissioning_days" => self.commissioning_days = f()?,
            "gauge_offset_a" => self.gauge_offset_a = f()?,
            "radiation" => self.radiation = b()?,
            "contact_outage_days" => self.contact_outage_days = f()?,
            "contact_outage_start_day" => self.contact_outage_start_day = f()?,
            "battery_scale" => self.battery_scale = f()?,
            "charge_v_cell" => self.charge_v_cell = f()?,
            "adcs_failed" => self.adcs_failed = b()?,
            "wings_failed" => self.wings_failed = b()?,
            "wing_a_failed" => self.wing_a_failed = b()?,
            "separate_at_eclipse" => self.separate_at_eclipse = b()?,
            "separation_soak_c" => self.separation_soak_c = Some(f()?),
            "eol_heater_ohms" => self.eol_heater_ohms = f()?,
            "payload_stuck_on" => self.payload_stuck_on = b()?,
            "array_years" => self.array_years = f()?,
            "rx_failed_day" => self.rx_failed_day = f()?,
            "rx_sefi" => self.rx_sefi = b()?,
            "dump_every_days" => self.dump_every_days = f()?,
            "gyro" => {
                if crate::adcs_mc::gyro_preset(v).is_none() {
                    return Err(format!("unknown gyro {v}"));
                }
                self.gyro = v.to_string();
            }
            _ => return Err(format!("unknown setting {k}")),
        }
        Ok(())
    }

    /// The settings as JSON
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\n");
        let (y, m, d) = self.integration_date;
        let (dy, dm, dd, dh) = self.deploy_utc;
        let fields: Vec<(&str, String)> = vec![
            ("scenario", format!("\"{}\"", self.scenario)),
            ("seed", self.seed.to_string()),
            ("integration_date", format!("\"{y:04}-{m:02}-{d:02}\"")),
            ("deploy_utc", format!("\"{dy:04}-{dm:02}-{dd:02}T{dh:05.2}h\"")),
            ("altitude_km", self.altitude_km.to_string()),
            ("ltan_h", self.ltan_h.to_string()),
            ("integration_soc", self.integration_soc.to_string()),
            ("storage_temp_c", self.storage_temp_c.to_string()),
            ("mission_days", self.mission_days.to_string()),
            ("passivate_at_eol", self.passivate_at_eol.to_string()),
            ("to_reentry", self.to_reentry.to_string()),
            ("step_s", self.step_s.to_string()),
            ("fine_step_s", self.fine_step_s.to_string()),
            ("record_s", self.record_s.to_string()),
            ("detail_record_s", self.detail_record_s.to_string()),
            ("detail_days", self.detail_days.to_string()),
            ("burn_limit_a", self.burn_limit_a.to_string()),
            ("contacts_per_day", self.contacts_per_day.to_string()),
            ("contact_min_elev_deg", self.contact_min_elev_deg.to_string()),
            ("commissioning_days", self.commissioning_days.to_string()),
            ("gauge_offset_a", self.gauge_offset_a.to_string()),
            ("radiation", self.radiation.to_string()),
            ("contact_outage_days", self.contact_outage_days.to_string()),
            ("contact_outage_start_day", self.contact_outage_start_day.to_string()),
            ("battery_scale", self.battery_scale.to_string()),
            ("charge_v_cell", self.charge_v_cell.to_string()),
            ("adcs_failed", self.adcs_failed.to_string()),
            ("wings_failed", self.wings_failed.to_string()),
            ("wing_a_failed", self.wing_a_failed.to_string()),
            ("separate_at_eclipse", self.separate_at_eclipse.to_string()),
            ("separation_soak_c", self.separation_soak_c.map_or("null".into(), |t| t.to_string())),
            ("eol_heater_ohms", self.eol_heater_ohms.to_string()),
            ("payload_stuck_on", self.payload_stuck_on.to_string()),
            ("array_years", self.array_years.to_string()),
            ("gyro", format!("\"{}\"", self.gyro)),
            ("rx_failed_day", self.rx_failed_day.to_string()),
            ("rx_sefi", self.rx_sefi.to_string()),
            ("dump_every_days", self.dump_every_days.to_string()),
        ];
        for (i, (k, v)) in fields.iter().enumerate() {
            let _ = write!(s, "  \"{k}\": {v}");
            s.push_str(if i + 1 < fields.len() { ",\n" } else { "\n" });
        }
        s.push('}');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenarios_and_overrides() {
        assert!(Config::scenario("nope").is_none());
        let mut c = Config::scenario("smoke").unwrap();
        assert_eq!(c.mission_days, 1.0);
        // BASELINE Rev D: 475 km, planning launch 2028-11, storage L-47 d at 60 %
        assert_eq!((c.altitude_km, c.deploy_utc.1, c.integration_soc), (475.0, 11, 0.60));
        let days =
            crate::env::time::from_utc(2028, 11, 1, 6, 0, 0.0) - crate::env::time::from_utc(2028, 9, 15, 6, 0, 0.0);
        assert_eq!(days / 86_400.0, 47.0);
        c.set("altitude_km=450").unwrap();
        assert_eq!(c.altitude_km, 450.0);
        assert!(c.set("bogus=1").is_err());
        assert!(c.to_json().contains("\"altitude_km\": 450"));
        let cold = Config::scenario("ants_cold").unwrap();
        assert!(cold.separate_at_eclipse && cold.separation_soak_c == Some(-20.0) && cold.mission_days == 14.0);
    }
}
