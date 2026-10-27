//! The flight software under test: the executive of `sqso_ffi` and every
//! flight module, on the simulated HAL, on the virtual clock.
//!
//! The simulation boots it, steps it, resets it and cuts its power. Its
//! inputs are written and its outputs read only through the HAL and the
//! HAL simulation hooks (`hal_sim.h`), as the hardware would see them; its
//! internal state is read through the flight modules' own C API, for the
//! record only.

use std::cell::RefCell;

use sqso_ffi::executive::Executive;
use sqso_ffi::hal_sim;
use sqso_fsw::eps_telemetry::{self, Telemetry};

/// `hal_gpio.h` pins
pub mod pin {
    /// Jetson payload power enable (output)
    pub const PAYLOAD_EN: u32 = 0;
    /// Radio power enable (output)
    pub const RADIO_EN: u32 = 1;
    /// ADCS power enable (output)
    pub const ADCS_EN: u32 = 2;
    /// Heater enable (output)
    pub const HEATER_EN: u32 = 3;
    /// Deployment master arm (output)
    pub const DEPLOY_EN: u32 = 4;
    /// Status LED (output)
    pub const LED_STATUS: u32 = 5;
    /// Jetson power gate (output)
    pub const JETSON_PWR: u32 = 6;
    /// Transmitter (PA rail) enable (output)
    pub const RF_TX_EN: u32 = 7;
    /// EPS alert (input)
    pub const ALERT: u32 = 8;
    /// Separation switch sense (input; high = stowed)
    pub const DEPLOY_SENSE: u32 = 9;
    /// GSE umbilical present (input)
    pub const UMBILICAL_SENSE: u32 = 10;
    /// Antenna burn wire (output)
    pub const ANTENNA_BURN: u32 = 11;
    /// Solar-wing burn wire (output)
    pub const SOLAR_BURN: u32 = 12;
    /// Antenna deployed sense (input)
    pub const ANTENNA_SENSE: u32 = 13;
    /// Solar wings deployed sense (input)
    pub const SOLAR_SENSE: u32 = 14;
    /// Downlink power select: low 0.5 W, high 1 W (output)
    pub const TX_PWR_SEL: u32 = 15;
    /// FM transponder enable (output)
    pub const XPDR_EN: u32 = 16;
    /// GNSS receiver load switch (output)
    pub const GNSS_EN: u32 = 17;
    /// Solar-wing burn driver B (output)
    pub const SOLAR_BURN_B: u32 = 18;
}

/// `hal_adc.h` channels
pub mod adc {
    /// Battery voltage, V
    pub const VBATT: u32 = 0;
    /// Battery current, A
    pub const IBATT: u32 = 1;
    /// Solar voltage, V
    pub const VSOLAR: u32 = 2;
    /// Solar current, A
    pub const ISOLAR: u32 = 3;
    /// 5 V bus, V
    pub const VBUS: u32 = 4;
    /// MCU temperature, °C
    pub const TEMP_MCU: u32 = 5;
    /// OBC thermistor (payload heat spreader), °C
    pub const TEMP_BOARD: u32 = 6;
    /// PA temperature, °C
    pub const TX_TEMP: u32 = 7;
    /// Forward power, W
    pub const TX_FWD: u32 = 8;
    /// Reflected power, W
    pub const TX_REV: u32 = 9;
}

/// Why the OBC (re)started (`HalResetCause_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetCause {
    /// Power applied: separation, a switch toggle, recovery from a brownout
    PowerOn = 0,
    /// The supply dipped below the brownout threshold
    Brownout = 1,
    /// The independent watchdog
    Watchdog = 2,
    /// A software reset (single-event functional interrupt recovery)
    Software = 3,
}

/// `hal_uart.h` ports
const UART_PORT_DEBUG: u32 = 1;
/// I2C bus of the EPS controller
const EPS_BUS: u32 = 0;
/// Time from reset to the executive's first instruction (boot ROM, clock
/// set-up, C runtime): the target's time base reads about this at start
pub const BOOT_TIME_US: u64 = 150_000;

thread_local! {
    static CONSOLE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn capture(line: &[u8]) {
    let text = String::from_utf8_lossy(line).trim_end().to_string();
    if !text.is_empty() {
        CONSOLE.with(|c| c.borrow_mut().push(text));
    }
}

/// What the flight software reports about itself (its own C API)
#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    /// Operational state (`SystemOperationalState_t`: 0 BOOT, 1 DETUMBLE,
    /// 2 SAFE, 3 IDLE, 4 ACTIVE, 5 EMERGENCY)
    pub op_state: u32,
    /// EPS power mode (0 SAFE, 1 IDLE, 2 ACTIVE)
    pub power_mode: u32,
    /// The SOC the flight software holds, 0–1
    pub soc: f64,
    /// Payload switched on
    pub payload_on: bool,
    /// Deployment state (`DeploymentState_t`)
    pub deploy_state: u32,
    /// RF silence still running
    pub rf_silence: bool,
    /// Transmission permitted (no TX-OFF latch, no inhibit)
    pub tx_permitted: bool,
    /// Loss-of-contact inhibit active
    pub contact_inhibit: bool,
    /// Seconds since the last authenticated command
    pub since_contact_s: u32,
    /// Loss-of-contact timeout, days
    pub contact_timeout_days: u16,
    /// Beacons sent (this boot)
    pub beacons: u32,
    /// Downlink counters
    pub downlink: sqso_fsw::downlink::Counters,
    /// Downlink held for temperature
    pub downlink_over_temp: bool,
    /// Entries in the fault log
    pub faults: usize,
    /// Resets the mission data counted
    pub resets: u32,
    /// Payload energy the mission data counted, Wh
    pub payload_energy_wh: f64,
    /// Telecommands executed
    pub commands_executed: u32,
    /// Telecommands refused for authentication
    pub auth_failures: u32,
    /// Uplink frames received
    pub uplink_frames: u32,
}

/// The flight software and the simulated HAL around it
pub struct Fsw {
    exec: Option<Box<Executive>>,
    boots: u32,
}

impl Default for Fsw {
    fn default() -> Self {
        Self::new()
    }
}

impl Fsw {
    /// Powered off; console output captured from now on
    pub fn new() -> Fsw {
        sqso_ffi::sil::console_sink(Some(capture));
        sqso_ffi::sil::boot_clock(0);
        Fsw { exec: None, boots: 0 }
    }

    /// Whether the OBC is powered and running
    pub fn running(&self) -> bool {
        self.exec.is_some()
    }

    /// Boots so far
    pub fn boots(&self) -> u32 {
        self.boots
    }

    /// Power the OBC (or reset it) and run the flight software's start-up.
    /// The time base restarts near zero, as on the target; the simulated
    /// FRAM keeps its contents (it is non-volatile). False if start-up
    /// failed (the OBC stays off).
    pub fn boot(&mut self, cause: ResetCause) -> bool {
        self.exec = None;
        hal_sim::hal_sim_set_reset_cause(cause as u32);
        sqso_ffi::sil::boot_clock(BOOT_TIME_US);
        let mut exec = Box::new(Executive::BOOT);
        let ok = exec.start();
        self.boots += 1;
        if ok {
            self.exec = Some(exec);
        }
        ok
    }

    /// Cut the OBC's power (nothing is saved)
    pub fn power_off(&mut self) {
        self.exec = None;
    }

    /// Advance the OBC's clock by `dt_us` and run one pass of the main loop
    pub fn step(&mut self, dt_us: u64) {
        sqso_ffi::sil::advance_us(dt_us);
        if let Some(exec) = self.exec.as_mut() {
            exec.step();
        }
    }

    /// The OBC's time base, ms since boot
    pub fn obc_ms(&self) -> u64 {
        sqso_ffi::sil::now_us() / 1000
    }

    /// Whether the hardware watchdog has run out (the HAL's countdown)
    pub fn watchdog_expired(&self) -> bool {
        self.running() && hal_sim::hal_wdt_remaining_ms() == 0
    }

    /// Console and flight-log lines since the last call
    pub fn drain_console(&self) -> Vec<String> {
        CONSOLE.with(|c| std::mem::take(&mut *c.borrow_mut()))
    }

    /// The EPS controller's telemetry registers (as the controller would
    /// present them on I2C)
    pub fn set_eps(&self, t: &Telemetry) {
        let mut regs = [0u8; 1 + eps_telemetry::BLOCK_LEN];
        let mut block = [0u8; eps_telemetry::BLOCK_LEN];
        eps_telemetry::encode(t, &mut block);
        regs[1..].copy_from_slice(&block);
        // SAFETY: regs is a live buffer of its length
        let _ = unsafe { hal_sim::hal_i2c_write(EPS_BUS, eps_telemetry::ADDRESS, regs.as_ptr(), regs.len()) };
    }

    /// An ADC input, in its own unit
    pub fn set_adc(&self, channel: u32, value: f64) {
        let _ = hal_sim::hal_sim_adc_set(channel, value);
    }

    /// An input pin's level, as the outside world drives it
    pub fn drive(&self, pin: u32, level: bool) {
        hal_sim::hal_sim_gpio_drive_input(pin, level);
    }

    /// A pin's level (outputs: what the flight software drives)
    pub fn pin(&self, pin: u32) -> bool {
        self.running() && hal_sim::hal_gpio_get(pin)
    }

    /// The `XPDR_EN` heartbeat as the radio board's detector sees it: the
    /// OBC's HAL time now and when the heartbeat stops or stopped, µs
    /// (`None`: the OBC is off, or the heartbeat has not run since boot;
    /// a level on the pin is not a heartbeat)
    pub fn xpdr_heartbeat(&self) -> Option<(u64, u64)> {
        let end = hal_sim::hal_sim_gpio_heartbeat_end_us(pin::XPDR_EN);
        (self.running() && end > 0).then(|| (hal_sim::hal_timer_get_us(), end))
    }

    /// A received AX.25 frame (with its FCS) from the 435 MHz receiver
    /// through the TNC; false if the receive queue is full
    pub fn uplink(&self, frame: &[u8]) -> bool {
        // SAFETY: frame is a live slice
        self.running() && unsafe { hal_sim::hal_sim_radio_inject(frame.as_ptr(), frame.len()) } == 0
    }

    /// NMEA bytes from the GNSS receiver (`UART_PORT_GNSS`)
    pub fn gnss_bytes(&self, bytes: &[u8]) -> bool {
        // SAFETY: bytes is a live slice
        self.running() && unsafe { hal_sim::hal_sim_uart_inject(2, bytes.as_ptr(), bytes.len()) } == 0
    }

    /// Bytes arriving on the umbilical (GSE) port
    pub fn gse_bytes(&self, bytes: &[u8]) -> bool {
        // SAFETY: bytes is a live slice
        self.running() && unsafe { hal_sim::hal_sim_uart_inject(UART_PORT_DEBUG, bytes.as_ptr(), bytes.len()) } == 0
    }

    /// Frames the flight software handed to the TNC since the last call
    pub fn take_tx(&self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut buf = [0u8; 512];
        loop {
            let mut len = 0usize;
            // SAFETY: buf and len are live
            if unsafe { hal_sim::hal_sim_radio_take_tx(buf.as_mut_ptr(), buf.len(), &mut len) } != 0 {
                break;
            }
            out.push(buf[..len].to_vec());
        }
        out
    }

    /// The ADCS sensors as the flight software reads them: the magnetometer
    /// (body frame, T), the sun sensors' vector (its length the intensity)
    /// and the gyro's rate (rad/s; `None`: no gyro)
    pub fn set_adcs_sensors(
        &self,
        mag_body_t: crate::env::math::V3,
        sun_body: crate::env::math::V3,
        gyro: Option<crate::env::math::V3>,
    ) {
        use sqso_fsw::adcs::Vec3;
        let mag = Vec3::new(mag_body_t.x * 1e6, mag_body_t.y * 1e6, mag_body_t.z * 1e6);
        let sun = Vec3::new(sun_body.x, sun_body.y, sun_body.z);
        let w = gyro.map(|r| Vec3::new(r.x, r.y, r.z));
        let rate = w.as_ref().map_or(core::ptr::null(), |w| w as *const Vec3);
        // SAFETY: live Vec3_t (the rate NULL for no gyro)
        unsafe { sqso_ffi::adcs_control::adcs_set_sim_sensors(&mag, &sun, rate) };
    }

    /// The magnetorquer dipole the flight software commands, averaged over
    /// the control period, body frame, A·m² (zero with the coils off): the
    /// burst's dipole for its 900 ms of each second
    /// (`sqso_fsw::adcs::CoilTiming`), as the mission's 1 s steps see it
    pub fn adcs_dipole(&self) -> crate::env::math::V3 {
        use sqso_fsw::adcs::{CONTROL_PERIOD_MS, MTQ_ON_MS};
        let mut st = sqso_fsw::adcs::Adcs::new().state;
        // SAFETY: st is a live AdcsState_t
        let ok = self.running() && unsafe { sqso_ffi::adcs_control::adcs_get_state(&mut st) } == 0;
        if ok && st.mtq.enabled {
            let on = MTQ_ON_MS as f64 / CONTROL_PERIOD_MS as f64;
            crate::env::math::V3::new(st.mtq.dipole.x, st.mtq.dipole.y, st.mtq.dipole.z) * on
        } else {
            crate::env::math::V3::ZERO
        }
    }

    /// The commanded dipole as a fraction of the coils' maximum (drive power)
    pub fn adcs_dipole_fraction(&self) -> f64 {
        let d = self.adcs_dipole();
        ((d.x.abs() + d.y.abs() + d.z.abs()) / (3.0 * sqso_fsw::adcs::MAX_DIPOLE)).min(1.0)
    }

    /// The flight software's own view of itself
    pub fn snapshot(&self) -> Snapshot {
        use sqso_ffi::*;
        if !self.running() {
            return Snapshot::default();
        }
        let mut dl = downlink::DownlinkStatus::default();
        // SAFETY: dl is a live DownlinkStatus_t
        let _ = unsafe { downlink::downlink_get_status(&mut dl) };
        let mut mission = sqso_fsw::mission_data::MissionData::ZERO;
        // SAFETY: mission is a live MissionData_t
        let _ = unsafe { mission_data::mission_data_get(&mut mission) };
        let mut cmd = cmd_handler::CmdStats::default();
        // SAFETY: cmd is a live CmdStats_t
        let _ = unsafe { cmd_handler::cmd_get_stats(&mut cmd) };
        let mut up = uplink::UplinkStats::default();
        // SAFETY: up is a live UplinkStats_t
        unsafe { uplink::uplink_get_stats(&mut up) };
        Snapshot {
            op_state: system_state::sys_get_operational_state(),
            power_mode: eps_control::eps_get_power_mode(),
            soc: eps_control::eps_get_soc(),
            payload_on: eps_control::eps_is_payload_enabled(),
            deploy_state: deployment::deployment_get_state(),
            rf_silence: rf_silence::rf_silence_active(),
            tx_permitted: rf_control::rf_control_tx_permitted(),
            contact_inhibit: contact_monitor::contact_monitor_inhibited(),
            since_contact_s: contact_monitor::contact_monitor_seconds_since_contact(),
            contact_timeout_days: contact_monitor::contact_monitor_get_timeout_days(),
            beacons: beacon::beacon_get_total_count(),
            downlink: dl.counters,
            downlink_over_temp: dl.over_temp,
            faults: fault_mgmt::fault_log_get_count(),
            resets: mission.reset_count,
            payload_energy_wh: mission.total_energy_wh,
            commands_executed: cmd.total_executed,
            auth_failures: cmd.auth_failures,
            uplink_frames: up.frames,
        }
    }
}
