//! The executive (was `main.c`): bring-up order, the main loop and shutdown.
//!
//! One pass of the loop ([`Executive::step`]) runs every stage in turn and
//! each stage checks in with the watchdog manager, which refreshes the
//! hardware watchdog only when all of them did (SW-15). The host simulation
//! (`sqso-sim`) and the firmware run the same passes; they differ only in
//! how they wait between them.
//!
//! @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on
//! @requirement SRS-P001 Main control loop executes within 20 ms
//! @requirement SRS-F086 The software shall command no magnetorquer output until 120 s after separation

use core::ffi::{CStr, c_char};
use core::fmt::{self, Write};
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};

use sqso_fsw::beacon::{Content, Measurements, TLM_BV, TLM_SOC, TLM_SUN};
use sqso_fsw::eps::EpsState;
use sqso_fsw::executive::{self as exec, PowerModel};
use sqso_fsw::fault_log::Entry;
use sqso_fsw::fixed::Fixed;
use sqso_fsw::gnss::Receiver;
use sqso_fsw::gnss_schedule::{self, Sessions};
use sqso_fsw::heater::{self, Heater};
use sqso_fsw::housekeeping::{self, Cadence, Housekeeping};
use sqso_fsw::mission_data::MissionData;
use sqso_fsw::payload_guard::{Outcome, Sample, TELEMETRY_TIMEOUT_MS};
use sqso_fsw::payload_schedule::{self, Action, BATCH_LINES, Schedule};
use sqso_fsw::transponder::{self, Transponder};
use sqso_protocol::ax25::{self, Station};
use sqso_protocol::jetson::{LINE_MAX, TEXT_MAX};

use crate::c::{fault_type, rc, severity};
use crate::jetson_link::{JetsonMsg, JetsonRx};
use crate::sensors::Sensor;
use crate::{
    adcs_control, beacon, cmd_handler, contact_monitor, deployment, downlink, eps_control, eps_lockout, fault_mgmt,
    gse_provision, hk_store, jetson_link, mission_data, ops_state, passivation, payload_guard, receiver, rf_control,
    rf_silence, sensors, state_machine, system_state, uart_comm, uplink, watchdog_mgr,
};

/// `ADC_RESOLUTION_12BIT`, `ADC_REF_VDD` (hal_adc.h)
const ADC_RESOLUTION_12BIT: u32 = 12;
const ADC_REF_VDD: u32 = 2;
/// I2C buses (hal_i2c.h): 0 is the EPS and power monitors, 1 the ADCS;
/// standard mode (SMART-QSO-ICD-001 §2.3)
const I2C_BUSES: [u32; 2] = [0, 1];
const I2C_SPEED_HZ: u32 = 100_000;

// watchdog_mgr.h task ids
const WDT_TASK_MAIN_LOOP: u32 = 0;
const WDT_TASK_COMM: u32 = 3;
const WDT_TASK_BEACON: u32 = 5;
const WDT_TASK_PAYLOAD: u32 = 6;
const WDT_TASK_HEALTH: u32 = 7;

// state_machine.h
const STATE_BOOT: u32 = 0;
const STATE_DETUMBLE: u32 = 2;
const STATE_IDLE: u32 = 3;
const STATE_ACTIVE: u32 = 4;
const EVENT_NONE: u32 = 0;
const EVENT_BOOT_COMPLETE: u32 = 1;
const EVENT_DETUMBLE_COMPLETE: u32 = 2;
const EVENT_THERMAL_FAULT: u32 = 7;

// adcs_control.h
const ADCS_MODE_IDLE: u32 = 0;
const ADCS_MODE_DETUMBLE: u32 = 1;
/// ADCS control cycle, ms (1 Hz: ADCS_DESIGN §5)
const ADCS_PERIOD_MS: u64 = 1000;
/// No magnetorquer output until this long after separation, s (SpaceX
/// Rideshare Payload User's Guide v11 §3.3.4; LV-02)
const MTQ_SEPARATION_DELAY_S: u32 = 120;
// deployment.h states in which the separation delay can still be running
const DEPLOY_STATE_INHIBITED: u32 = 0;
const DEPLOY_STATE_WAITING: u32 = 1;
const DEPLOY_STATE_ARMED: u32 = 10;

// hal_gpio.h
const GPIO_PIN_HEATER_EN: u32 = 3;
const GPIO_PIN_XPDR_EN: u32 = 16;
const GPIO_PIN_GNSS_EN: u32 = 17;
const GPIO_PIN_SOLAR_SENSE: u32 = 14;
// hal_uart.h
const UART_PORT_GNSS: u32 = 2;
/// The GNSS receiver's NMEA rate (piNAV-NG default assumed; its ICD sets it)
const GNSS_BAUD: u32 = 9600;
/// GNSS sessions need the state of charge at or above the IDLE threshold
/// (BASELINE §5: above the contingency floor)
const GNSS_SOC_MIN: f64 = 0.40;
const ADC_CHANNEL_TEMP_MCU: u32 = 5;
const GPIO_DIR_OUTPUT: u32 = 1;
const GPIO_PULL_DOWN: u32 = 2;
// hal_adc.h: the transmitter's monitors (as the downlink's)
const ADC_CHANNEL_TX_TEMP: u32 = 7;
const ADC_CHANNEL_TX_FWD: u32 = 8;
/// The battery temperature the heater thermostat uses must be this fresh, ms
const HEATER_SENSOR_MAX_AGE_MS: u64 = 10_000;

// jetson_link.h
const JETSON_MSG_BEACON_RSP: u32 = 0;
const JETSON_MSG_STATUS: u32 = 1;
const JETSON_OK: u32 = 0;

// fault_mgmt.h fault types not in c::fault_type
const FAULT_TYPE_VOLTAGE_RANGE: u32 = 8;
const FAULT_TYPE_HEALTH_THERMAL: u32 = 13;
const FAULT_TYPE_HEALTH_POWER: u32 = 14;
const FAULT_TYPE_HEALTH_COMM: u32 = 15;

/// `SMART_QSO_VERSION_STRING`
pub const VERSION: &str = "0.2.0-dev";
/// Passes in a host simulation run
pub const SIMULATION_TICKS: u64 = exec::SIMULATION_TICKS;
/// `BEACON_MEAS_MAX_AGE_MS`
const BEACON_MEAS_MAX_AGE_MS: u64 = 60_000;
/// The critical persisted state is checked against its records this often,
/// ms (AV-07)
const VERIFY_INTERVAL_MS: u64 = 60_000;

/// GNSS_REQUEST received (the command handler sets it, the next pass acts)
static GNSS_REQUESTED: AtomicBool = AtomicBool::new(false);
/// GET_HOUSEKEEPING received
static HK_REQUESTED: AtomicBool = AtomicBool::new(false);

/// GNSS_REQUEST: a receiver session at the next pass the gates allow
pub(crate) fn request_gnss_session() {
    GNSS_REQUESTED.store(true, Ordering::Relaxed);
}

/// GET_HOUSEKEEPING: a housekeeping frame after the beacon gap
pub(crate) fn request_housekeeping() {
    HK_REQUESTED.store(true, Ordering::Relaxed);
}

/// `HalUartConfig_t`
#[repr(C)]
struct HalUartConfig {
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: u32,
}

/// An error line: stderr on the host, the console on the target
fn eprint(args: fmt::Arguments<'_>) {
    let mut line = Text::<160>::new();
    let _ = line.write_fmt(args);
    #[cfg(not(target_os = "none"))]
    crate::os::eprint_line(line.as_bytes());
    #[cfg(target_os = "none")]
    crate::c::console_line(line.as_bytes());
}

/// Bounded text, like a C buffer written with snprintf: a write that does
/// not fit (with room for the NUL) leaves the text as it was
struct Text<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Text<N> {
    const fn new() -> Self {
        Text { buf: [0; N], len: 0 }
    }

    fn as_bytes(&self) -> &[u8] {
        self.buf.get(..self.len).unwrap_or(&[])
    }

    /// Append `args` if all of it fits below `room` (bytes, NUL included)
    fn append(&mut self, room: usize, args: fmt::Arguments<'_>) -> bool {
        let start = self.len;
        let limit = room.min(N);
        let mut w = Bounded { buf: self.buf.get_mut(..limit).unwrap_or(&mut []), len: start };
        let fits = fmt::write(&mut w, args).is_ok() && w.len < limit;
        if fits {
            self.len = w.len;
        } else {
            self.len = start;
        }
        fits
    }
}

impl<const N: usize> Write for Text<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let n = N.saturating_sub(1);
        let mut w = Bounded { buf: self.buf.get_mut(..n).unwrap_or(&mut []), len: self.len };
        let r = w.write_str(s);
        self.len = w.len;
        r
    }
}

struct Bounded<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Write for Bounded<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            let slot = self.buf.get_mut(self.len).ok_or(fmt::Error)?;
            *slot = b;
            self.len += 1;
        }
        Ok(())
    }
}

fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

/// The text of a NUL-terminated byte field
fn field(bytes: &[u8]) -> &[u8] {
    bytes.split(|&b| b == 0).next().unwrap_or(&[])
}

fn as_str(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or("?")
}

fn add_fault(kind: u32, sev: u32, description: &CStr, soc: f64) {
    // SAFETY: description is a NUL-terminated string
    let _ = unsafe { fault_mgmt::fault_log_add(kind, sev, description.as_ptr(), soc) };
}

fn sm_event(event: u32) {
    // SAFETY: the system state's context; no result wanted
    let _ = unsafe { state_machine::sm_process_event(system_state::sys_get_sm_context(), event, null_mut()) };
}

/// A sensor value read within `max_age_ms` of `now`
fn fresh_sensor(id: &CStr, now: u64, max_age_ms: u64) -> Option<f64> {
    let mut sensor = Sensor::ZERO;
    // SAFETY: id is NUL-terminated; sensor is a live Sensor_t
    let found = unsafe { sensors::sensors_get_by_id(id.as_ptr(), &mut sensor) } == rc::OK;
    (found && exec::fresh(sensor.last_read_ms, now, max_age_ms)).then_some(sensor.last_value)
}

/// Watchdog pre-reset action: persist state before the hardware resets
extern "C" fn save_state_before_reset() {
    let _ = mission_data::mission_data_save();
    let _ = eps_control::eps_save_config();
}

/// The executive's state between passes
pub struct Executive {
    tick: u64,
    telemetry_interval_ms: u32,
    program_start_ms: u64,
    last_telemetry_ms: u64,
    last_health_check_ms: u64,
    last_contact_tick_ms: u64,
    /// When the power state was last updated (payload energy), ms
    last_power_ms: u64,
    power: PowerModel,
    jetson_temp_c: i32,
    jetson_status_ms: u64,
    jetson_status_seen: bool,
    /// Beacon texts received from the Jetson and not yet sent, oldest first
    /// (none: use a template)
    ai_cache: [[u8; TEXT_MAX + 1]; BATCH_LINES],
    /// How many of `ai_cache` hold a text
    ai_cached: usize,
    /// When the Jetson runs (with the EPS controller answering)
    schedule: Schedule,
    /// Battery heater thermostat
    heater: Heater,
    /// The FM transponder's gates
    transponder: Transponder,
    /// GNSS receiver sessions and the NMEA receiver
    gnss: Sessions,
    gnss_rx: Receiver,
    /// The housekeeping frame's cadence
    hk: Cadence,
    /// When the ADCS last ran its control cycle, ms
    last_adcs_ms: u64,
    /// The ADCS was told to detumble since this boot
    detumble_commanded: bool,
    beacon_seq: u16,
    jetson_rx: JetsonRx,
    /// The operating state and fault-log count at the last pass (stored
    /// housekeeping events)
    last_state: u32,
    last_fault_count: usize,
    /// When the critical state was last checked against its records, ms
    last_verify_ms: u64,
}

impl Executive {
    /// The state before [`Executive::start`]. A constant, so the firmware
    /// can place it in a `static` and start it in place: the state is about
    /// 2.7 kB, more than a stack frame should carry (SRS-C003)
    pub const BOOT: Executive = Executive {
        tick: 0,
        telemetry_interval_ms: 0,
        program_start_ms: 0,
        last_telemetry_ms: 0,
        last_health_check_ms: 0,
        last_contact_tick_ms: 0,
        last_power_ms: 0,
        power: PowerModel::START,
        jetson_temp_c: 0,
        jetson_status_ms: 0,
        jetson_status_seen: false,
        ai_cache: [[0; TEXT_MAX + 1]; BATCH_LINES],
        ai_cached: 0,
        schedule: Schedule::new(),
        heater: Heater::new(),
        transponder: Transponder::new(),
        gnss: Sessions::new(),
        gnss_rx: Receiver::new(),
        hk: Cadence::new(),
        last_adcs_ms: 0,
        detumble_commanded: false,
        beacon_seq: 0,
        jetson_rx: JetsonRx::EMPTY,
        last_state: STATE_BOOT,
        last_fault_count: 0,
        last_verify_ms: 0,
    };

    /// Bring the system up from [`Executive::BOOT`]; false (after reporting
    /// why) if it cannot run. Not inlined, here and in [`Executive::step`]:
    /// the caller's frame (the firmware's async main task) would carry the
    /// frames of both at once (SRS-C003)
    #[inline(never)]
    pub fn start(&mut self) -> bool {
        if !self.init_system() {
            eprint(format_args!("[SYSTEM] Initialization failed"));
            return false;
        }
        self.announce();
        true
    }

    fn init_system(&mut self) -> bool {
        // Time base first: on the target every module's timestamps come from
        // it, so nothing may read the time before it runs
        // SAFETY: no arguments (the HAL)
        if unsafe { crate::c::hal_timer_init() } != rc::OK {
            eprint(format_args!("[SYSTEM] Timer init failed"));
            return false;
        }
        self.program_start_ms = crate::c::now_ms();

        // HAL before any module touches GPIO or non-volatile memory; burn-wire
        // lines low as early as possible; fault logging before the rest
        // SAFETY: no arguments (the HAL)
        if unsafe { crate::c::hal_gpio_init() } != rc::OK || unsafe { crate::c::hal_flash_init() } != rc::OK {
            eprint(format_args!("[SYSTEM] GPIO/flash init failed"));
        }
        // The ADC (the downlink transmitter's monitors): 12-bit, VDDA
        // SAFETY: plain values (the HAL)
        if unsafe { crate::c::hal_adc_init(ADC_RESOLUTION_12BIT, ADC_REF_VDD) } != rc::OK {
            eprint(format_args!("[SYSTEM] ADC init failed"));
        }
        // The EPS controller and power monitors, and the ADCS sensors
        for bus in I2C_BUSES {
            // SAFETY: plain values (the HAL)
            if unsafe { crate::c::hal_i2c_init(bus, I2C_SPEED_HZ) } != rc::OK {
                eprint(format_args!("[SYSTEM] I2C bus {bus} init failed"));
            }
        }
        let _ = deployment::deployment_init();
        // The commissioning hold and transponder enable, before anything
        // can ask for ACTIVE
        let _ = ops_state::ops_state_init();
        if fault_mgmt::fault_mgmt_init() != rc::OK {
            eprint(format_args!("[SYSTEM] Fault management init failed"));
        }

        init_rf_and_telecommand();
        if !self.init_power_and_sensors() || !self.init_mode_manager() {
            return false;
        }
        if !init_watchdog() {
            eprint(format_args!("[SYSTEM] Watchdog init failed"));
            return false;
        }
        if uart_comm::uart_init() != rc::OK {
            eprint(format_args!("[WARN] UART initialization failed, telemetry disabled"));
            add_fault(fault_type::INIT, severity::WARNING, c"UART initialization failed", self.power.soc);
        }
        let gnss_uart = HalUartConfig { baud_rate: GNSS_BAUD, data_bits: 8, stop_bits: 1, parity: 0 };
        // SAFETY: gnss_uart is a live HalUartConfig_t
        if unsafe { crate::c::hal_uart_init(UART_PORT_GNSS, (&raw const gnss_uart).cast()) } != rc::OK {
            eprint(format_args!("[SYSTEM] GNSS UART init failed"));
        }

        self.last_telemetry_ms = self.program_start_ms;
        self.last_health_check_ms = self.program_start_ms;
        self.last_contact_tick_ms = self.program_start_ms;
        self.last_power_ms = self.program_start_ms;
        self.last_adcs_ms = self.program_start_ms;
        self.last_verify_ms = self.program_start_ms;
        self.last_fault_count = fault_mgmt::fault_log_get_count();
        self.telemetry_interval_ms = uart_comm::uart_get_telemetry_interval();
        // Stored housekeeping: the newest records of each store
        let _ = hk_store::hk_store_init();
        receiver::receiver_init();
        eps_lockout::eps_lockout_init();
        true
    }

    /// Mission data, EPS and sensors (sensors.yaml on the host, else the defaults)
    fn init_power_and_sensors(&mut self) -> bool {
        if mission_data::mission_data_init() != rc::OK {
            eprint(format_args!("[SYSTEM] Mission data init failed"));
        }
        if eps_control::eps_init() != rc::OK {
            eprint(format_args!("[SYSTEM] EPS init failed"));
        }
        if sensors::sensors_init() != rc::OK {
            eprint(format_args!("[SYSTEM] Sensor init failed"));
            return false;
        }
        load_sensor_table()
    }

    /// Operational state (one mode manager, SW-14): BOOT -> DETUMBLE -> IDLE.
    /// With the EPS controller answering (the spacecraft) DETUMBLE lasts
    /// until the ADCS has detumbled the spacecraft after separation, and
    /// the payload schedule owns the payload's switch-on. Without it (the
    /// stand-in power model of SW-01: the development simulation) there is
    /// no attitude or battery to manage: detumble completes in the first
    /// pass and the payload follows the power mode ([`Executive::run_adcs`],
    /// [`Executive::run_payload_schedule`]).
    fn init_mode_manager(&mut self) -> bool {
        let _ = beacon::beacon_init(); // templates until the Jetson's text passes the filter
        // Beacon interval jitter (RF-04), seeded differently at each boot
        #[allow(clippy::cast_possible_truncation)] // any 32 bits will do as a seed
        let seed = mission_data::mission_data_get_reset_count() ^ (crate::c::now_ms() as u32).rotate_left(16);
        beacon::beacon_set_jitter(sqso_fsw::beacon::JITTER_MS, seed);
        payload_guard::payload_guard_init();
        if adcs_control::adcs_init() != rc::OK {
            eprint(format_args!("[SYSTEM] ADCS init failed"));
        }
        // The spin axis and inertia the ground selected (STR-27)
        let _ = adcs_control::set_geometry(ops_state::ops_adcs_geometry());
        for pin in [GPIO_PIN_HEATER_EN, GPIO_PIN_XPDR_EN, GPIO_PIN_GNSS_EN] {
            // SAFETY: plain values (the HAL)
            let _ = unsafe { crate::c::hal_gpio_config(pin, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN) };
        }

        eps_control::eps_update_soc(self.power.soc);
        if system_state::sys_state_init() != rc::OK {
            eprint(format_args!("[SYSTEM] System state init failed"));
            return false;
        }
        sm_event(EVENT_BOOT_COMPLETE);
        true
    }

    fn announce(&self) {
        let mut mission = MissionData::ZERO;
        // SAFETY: mission is a live MissionData_t
        let _ = unsafe { mission_data::mission_data_get(&mut mission) };
        crate::c::console(format_args!("[SYSTEM] SMART-QSO Flight Software Started"));
        crate::c::console(format_args!("[SYSTEM] Version: {VERSION}"));
        crate::c::console(format_args!(
            "[SYSTEM] Mission Phase: {}, Power Mode: {}, SOC: {}%",
            mission.mission_phase,
            eps_control::eps_get_power_mode(),
            Fixed::new(self.power.soc * 100.0, 1)
        ));
    }

    /// Passes run so far
    pub fn ticks(&self) -> u64 {
        self.tick
    }

    /// One pass of the main loop; each stage checks in with the watchdog
    #[inline(never)]
    pub fn step(&mut self) {
        let tick = self.tick;
        self.tick = self.tick.wrapping_add(1);
        self.update_mission_state(tick);
        let now = crate::c::now_ms();

        self.monitor_system_health();
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_MAIN_LOOP);

        // Attitude and the battery heater
        let state = sm_state();
        self.run_adcs(now, state);
        self.run_heater(now);

        // Telemetry and beacon request to the Jetson (during a batch at least
        // every BATCH_INTERVAL_MS), and a periodic UART check
        let interval = if self.schedule.in_batch() {
            self.telemetry_interval_ms.min(payload_schedule::BATCH_INTERVAL_MS)
        } else {
            self.telemetry_interval_ms
        };
        if eps_control::eps_is_payload_enabled()
            && uart_comm::uart_is_initialized()
            && now.wrapping_sub(self.last_telemetry_ms) >= u64::from(interval)
        {
            if self.send_telemetry_to_jetson() {
                self.request_beacon_text();
            }
            self.last_telemetry_ms = now;
        }
        if uart_comm::uart_is_initialized() && (now % 10_000) < 100 {
            let _ = uart_comm::uart_check_health();
        }
        let _ = sensors::sensors_poll(now);

        // Jetson replies (beacon text, status), the OBC's payload limits, then
        // the batch schedule
        self.poll_jetson();
        self.guard_payload(now);
        self.run_payload_schedule(now, state);
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_PAYLOAD);

        // Beacon (stays silent while TX-OFF, contact or RF-silence inhibits)
        self.update_beacon_measurements(now);
        self.run_beacon(now);
        self.run_housekeeping(now, state);
        run_acks(now);
        run_dump(now);
        self.run_store(now, state);
        downlink::downlink_poll();
        self.run_gnss(now, state);
        self.run_transponder(now, state);
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_BEACON);

        // Telecommands from the 435 MHz receiver, then key provisioning
        // (umbilical only)
        // SAFETY: NULL: no response wanted
        let _ = unsafe { uplink::uplink_poll(null_mut()) };
        let _ = gse_provision::gse_poll();
        self.run_safety_checks(now);
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_COMM);

        // Loss-of-contact, RF-silence and deployment timers
        let elapsed = exec::elapsed_ms(now, self.last_contact_tick_ms);
        contact_monitor::contact_monitor_tick(elapsed);
        rf_silence::rf_silence_tick(elapsed);
        deployment::deployment_tick(elapsed);
        self.last_contact_tick_ms = now;

        if tick.is_multiple_of(exec::PERSISTENCE_INTERVAL_TICKS) {
            let _ = mission_data::mission_data_save();
            let _ = eps_control::eps_save_config();
        }

        // Refresh the hardware watchdog if every stage checked in
        watchdog_mgr::wdt_mgr_kick();
    }

    /// The power state, then mode changes and mission data. State of charge
    /// and sunlight are the EPS controller's when it answers
    /// (SMART-QSO-ICD-001), otherwise the stand-in power model's (SW-01)
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn update_mission_state(&mut self, tick: u64) {
        let payload_on = eps_control::eps_is_payload_enabled();
        let now = crate::c::now_ms();
        let elapsed_ms = now.wrapping_sub(self.last_power_ms);
        self.last_power_ms = now;
        let payload_wh = if let Some(t) = eps_control::eps_telemetry() {
            self.power.soc = t.soc;
            self.power.sunlit = t.sunlit();
            #[allow(clippy::cast_precision_loss)] // a pass is milliseconds long
            let hours = elapsed_ms as f64 / 3_600_000.0;
            payload_on.then_some(t.vbus_v * t.ibus_a * hours)
        } else {
            let discharge = self.power.step(tick, payload_on);
            self.power.payload_energy_wh(discharge, payload_on)
        };
        sensors::sensors_set_environment(self.power.sunlit, self.power.soc);

        // The EPS holds the measured SOC; the state machine turns power
        // conditions into transitions, and its entry actions set the EPS
        // power mode (SW-14)
        eps_control::eps_update_soc(self.power.soc);
        let event = state_machine::sm_power_event(sm_state(), self.power.soc, self.power.sunlit);
        if event != EVENT_NONE {
            sm_event(event);
        }

        let _ = mission_data::mission_data_update_uptime(now.wrapping_sub(self.program_start_ms));
        if let Some(wh) = payload_wh {
            let _ = mission_data::mission_data_add_energy(wh);
        }
    }

    /// Every 10 s: temperatures, bus voltage, SOC and the Jetson link
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn monitor_system_health(&mut self) {
        let now = crate::c::now_ms();
        if now.wrapping_sub(self.last_health_check_ms) < exec::HEALTH_CHECK_INTERVAL_MS {
            return;
        }
        self.last_health_check_ms = now;
        let soc = self.power.soc;

        for i in 0..sensors::sensors_get_count() {
            let mut sensor = Sensor::ZERO;
            // SAFETY: sensor is a live Sensor_t
            if unsafe { sensors::sensors_get(i, &mut sensor) } == rc::OK
                && field(&sensor.type_) == b"eps_temperature"
                && sensor.last_value > exec::HEALTH_TEMP_LIMIT_C
            {
                add_fault(FAULT_TYPE_HEALTH_THERMAL, severity::WARNING, c"High temperature in health check", soc);
                sm_event(EVENT_THERMAL_FAULT);
            }
        }

        // Bus regulation, on the measured value (SW-16)
        if let Some(bus_v) = fresh_sensor(c"BUSV", now, exec::HEALTH_CHECK_INTERVAL_MS)
            && !eps_control::eps_bus_voltage_ok(bus_v)
        {
            add_fault(FAULT_TYPE_VOLTAGE_RANGE, severity::ERROR, c"Bus voltage out of range", soc);
        }

        // Mode change for low power happens in update_mission_state
        if soc < exec::HEALTH_SOC_LOW {
            add_fault(FAULT_TYPE_HEALTH_POWER, severity::WARNING, c"Low SOC in health check", soc);
        }

        if eps_control::eps_is_payload_enabled() && !uart_comm::uart_is_initialized() {
            add_fault(FAULT_TYPE_HEALTH_COMM, severity::WARNING, c"UART communication lost with payload", soc);
            let _ = uart_comm::uart_init();
        }

        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_HEALTH);
    }

    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn send_telemetry_to_jetson(&mut self) -> bool {
        if !eps_control::eps_is_payload_enabled() || !uart_comm::uart_is_initialized() {
            return false;
        }

        // ICD_OBC_JETSON section 4: TELEMETRY,<fields...>,<CRC>\n
        let mut line = Text::<LINE_MAX>::new();
        let room = LINE_MAX - 6; // ",XXXX\n"
        if !line.append(
            room,
            format_args!(
                "TELEMETRY,{},{},{},{},{},",
                crate::c::now_ms(),
                if self.power.sunlit { "SUNLIT" } else { "ECLIPSE" },
                Fixed::new(self.power.soc, 3),
                eps_control::eps_get_power_mode(),
                Fixed::new(eps_control::eps_get_power_limit(), 1)
            ),
        ) {
            return false;
        }

        // Sensor values
        let start = line.len;
        if let Some(rest) = line.buf.get_mut(start..room) {
            // SAFETY: rest is a live buffer of rest.len() bytes
            let n = unsafe { sensors::sensors_format_telemetry(rest.as_mut_ptr().cast(), rest.len()) };
            line.len = (start + n).min(room.saturating_sub(1));
        }

        append_status_fields(&mut line, room);
        if line.as_bytes().last() == Some(&b',') {
            line.len -= 1;
        }

        // SAFETY: line.buf holds line.len bytes and has room for the seal
        let sealed = unsafe { jetson_link::jetson_link_seal(line.buf.as_mut_ptr().cast(), line.len, LINE_MAX) };
        send_line(line.buf.get(..sealed).unwrap_or(&[]), "telemetry");
        true
    }

    /// Ask the Jetson for the next beacon text (after the telemetry line).
    /// Not inlined, and called by `step` rather than by
    /// `send_telemetry_to_jetson`: its line buffer and the telemetry line's
    /// are not on the stack together (SRS-C003)
    #[inline(never)]
    fn request_beacon_text(&mut self) {
        self.beacon_seq = self.beacon_seq.wrapping_add(1);
        let mut request = [0u8; LINE_MAX];
        #[allow(clippy::cast_possible_truncation)] // TEXT_MAX is 176
        // SAFETY: request is a live buffer of LINE_MAX bytes
        let n = unsafe {
            jetson_link::jetson_link_beacon_req(self.beacon_seq, TEXT_MAX as u8, request.as_mut_ptr().cast(), LINE_MAX)
        };
        send_line(request.get(..n).unwrap_or(&[]), "beacon request");
    }

    /// Read Jetson lines: keep the answer to our latest beacon request
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn poll_jetson(&mut self) {
        let mut chunk = [0u8; 64];
        let mut received = 0usize;
        // SAFETY: chunk is a live buffer of 64 bytes; received a live size_t
        if !uart_comm::uart_is_initialized()
            || unsafe { uart_comm::uart_receive(chunk.as_mut_ptr().cast(), chunk.len(), &mut received) } != rc::OK
        {
            return;
        }
        for &byte in chunk.get(..received).unwrap_or(&[]) {
            let mut msg = JetsonMsg::ZERO;
            let mut status = JETSON_OK;
            let c = c_char::from_ne_bytes([byte]);
            // SAFETY: all three are live structs of their C types
            if !unsafe { jetson_link::jetson_rx_feed(&mut self.jetson_rx, c, &mut msg, &mut status) } {
                continue;
            }
            if status != JETSON_OK {
                eprint(format_args!("[UART] Rejected Jetson line ({status})"));
            } else if msg.msg_type == JETSON_MSG_BEACON_RSP && msg.seq == self.beacon_seq {
                self.cache_text(field(&msg.text));
                crate::c::console(format_args!("[UART] Beacon text from Jetson ({} chars)", msg.text_len));
            } else if msg.msg_type == JETSON_MSG_STATUS {
                self.jetson_temp_c = msg.temp_c;
                self.jetson_status_ms = crate::c::now_ms();
                self.jetson_status_seen = true;
                crate::c::console(format_args!(
                    "[UART] Jetson status: health={} temp={}C power={}mW",
                    msg.health, msg.temp_c, msg.power_mw
                ));
            } else {
                // Stale beacon answers, ACK and NACK need no action
            }
        }
    }

    /// OBC-enforced payload limits (SW-16): input power is the bus voltage
    /// times the Jetson rail current, the spreader temperature is the OBC's
    /// own thermistor; the Jetson-reported junction temperature is a
    /// second, not the only, input
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn guard_payload(&mut self, now: u64) {
        let mut sample = Sample::default();
        if let (Some(bus_v), Some(amps)) =
            (fresh_sensor(c"BUSV", now, TELEMETRY_TIMEOUT_MS), fresh_sensor(c"JPC", now, TELEMETRY_TIMEOUT_MS))
        {
            sample.power_valid = true;
            sample.input_w = bus_v * amps;
        }
        if let Some(t) = fresh_sensor(c"PT", now, TELEMETRY_TIMEOUT_MS) {
            sample.spreader_valid = true;
            sample.spreader_c = t;
        }
        if self.jetson_status_seen
            && now >= self.jetson_status_ms
            && now - self.jetson_status_ms <= TELEMETRY_TIMEOUT_MS
        {
            sample.junction_valid = true;
            sample.junction_c = f64::from(self.jetson_temp_c);
        }

        // SAFETY: sample is a live PayloadSample_t
        let result = unsafe { payload_guard::payload_guard_step(&sample, eps_control::eps_is_payload_enabled(), now) };
        let outcome = Outcome::from_raw(result).unwrap_or(Outcome::NoTelemetry);
        if let Some(kind) = exec::guard_fault_type(outcome) {
            let reason = payload_guard::payload_guard_reason(result);
            // SAFETY: reason is a static NUL-terminated string
            let _ = unsafe { fault_mgmt::fault_log_add(kind, severity::ERROR, reason, self.power.soc) };
            let _ = eps_control::eps_control_payload(false, self.power.soc);
        }
    }

    /// Measurements for the beacon telemetry (SW-13). The power model
    /// stands in for the EPS (SW-01); battery voltage is the BV sensor, only
    /// if read recently. There is no QSO detector yet, so the QSO count is
    /// not reported.
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn update_beacon_measurements(&mut self, now: u64) {
        let mut m = Measurements {
            soc: exec::soc_percent(self.power.soc),
            sunlit: u8::from(self.power.sunlit),
            valid: TLM_SOC | TLM_SUN,
            ..Measurements::default()
        };
        if let Some(mv) = fresh_sensor(c"BV", now, BEACON_MEAS_MAX_AGE_MS).and_then(exec::battery_mv) {
            m.battery_mv = mv;
            m.valid |= TLM_BV;
        }
        // SAFETY: m is a live BeaconMeasurements_t
        let _ = unsafe { beacon::beacon_update_measurements(&m, now) };
    }

    /// Send a beacon when one is due: the Jetson's latest text if the content
    /// filter passes it, otherwise a template. Each Jetson text is sent at
    /// most once.
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_beacon(&mut self, now: u64) {
        beacon::beacon_update_interval(eps_control::eps_get_power_mode());
        if !beacon::beacon_should_transmit(now) {
            return;
        }
        let mut content = Content::ZERO;
        let oldest = if self.ai_cached > 0 { self.ai_cache.first() } else { None };
        let ai_len = oldest.map_or(0, |t| field(t).len());
        let ai: *const c_char = oldest.filter(|_| ai_len > 0).map_or(null(), |t| t.as_ptr().cast());
        // SAFETY: content is a live BeaconContent_t; ai is NULL or ai_len bytes
        if unsafe { beacon::beacon_generate_content(&mut content, ai, ai_len) } == rc::OK
            && unsafe { beacon::beacon_transmit(&content) } == rc::OK
        {
            beacon::beacon_mark_transmitted(now);
            self.hk.beacon_sent(now);
        }
        self.drop_oldest_text();
    }

    /// Keep a Jetson beacon text (when the cache is full, the oldest goes)
    fn cache_text(&mut self, text: &[u8]) {
        if self.ai_cached == BATCH_LINES {
            self.drop_oldest_text();
        }
        if let Some(slot) = self.ai_cache.get_mut(self.ai_cached) {
            *slot = [0; TEXT_MAX + 1];
            let n = text.len().min(TEXT_MAX);
            if let (Some(dst), Some(src)) = (slot.get_mut(..n), text.get(..n)) {
                dst.copy_from_slice(src);
            }
            self.ai_cached += 1;
        }
    }

    /// The oldest cached text is spent (each is sent at most once)
    fn drop_oldest_text(&mut self) {
        if self.ai_cached > 0 {
            self.ai_cache.copy_within(1.., 0);
            if let Some(last) = self.ai_cache.last_mut() {
                *last = [0; TEXT_MAX + 1];
            }
            self.ai_cached -= 1;
        }
    }

    /// The ADCS at 1 Hz, once separated and powered (ADCS_DESIGN §5): DETUMBLE
    /// on the first cycle of a boot in the DETUMBLE state; the state machine
    /// leaves DETUMBLE when the ADCS does (detumbled, now SUNPOINT; or timed
    /// out, logged as a fault). In the mission an idle ADCS is told to
    /// detumble again, so it is never left without control
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_adcs(&mut self, now: u64, state: u32) {
        // The host simulation without the mission simulation has no EPS
        // controller and no attitude plant: nothing to detumble. On the target
        // a silent EPS controller must not skip the detumble and report it
        // complete (CDR RID FSW-01), so there the ADCS runs regardless
        if cfg!(not(target_os = "none")) && eps_control::eps_telemetry().is_none() {
            if state == STATE_DETUMBLE {
                sm_event(EVENT_DETUMBLE_COMPLETE); // stand-in power model: nothing to detumble
            }
            return;
        }
        if now.wrapping_sub(self.last_adcs_ms) < ADCS_PERIOD_MS {
            return;
        }
        self.last_adcs_ms = now;
        if !eps_control::eps_is_adcs_enabled() || !deployment::deployment_is_separated() || !separation_delay_served() {
            let _ = adcs_control::adcs_disable_mtq();
            return;
        }
        let mode = adcs_control::adcs_get_mode();
        if (state == STATE_DETUMBLE && !self.detumble_commanded)
            || (state != STATE_DETUMBLE && state != STATE_BOOT && mode == ADCS_MODE_IDLE)
        {
            self.detumble_commanded = true;
            let _ = adcs_control::adcs_set_mode(ADCS_MODE_DETUMBLE);
        }
        let _ = adcs_control::adcs_update(now);
        if state == STATE_DETUMBLE && self.detumble_commanded && adcs_control::adcs_get_mode() != ADCS_MODE_DETUMBLE {
            sm_event(EVENT_DETUMBLE_COMPLETE);
        }
    }

    /// The battery heater thermostat on the battery temperature (BT)
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_heater(&mut self, now: u64) {
        let battery_c = fresh_sensor(c"BT", now, HEATER_SENSOR_MAX_AGE_MS);
        let change = self.heater.step(battery_c);
        let t = Fixed::new(battery_c.unwrap_or(0.0), 1);
        match change {
            heater::Change::None => return,
            heater::Change::On => crate::c::console(format_args!("[THERMAL] Battery heater on ({t} C)")),
            heater::Change::Off => crate::c::console(format_args!("[THERMAL] Battery heater off ({t} C)")),
            heater::Change::NoSensor => crate::c::console(format_args!("[THERMAL] Battery heater off: no BT")),
        }
        // SAFETY: plain values (the HAL)
        let _ = unsafe { crate::c::hal_gpio_set(GPIO_PIN_HEATER_EN, self.heater.on) };
    }

    /// The FM transponder's gates (`sqso_fsw::transponder`): the `XPDR_EN`
    /// heartbeat runs while transmission is permitted, the radio powered,
    /// ACTIVE, sunlit, SOC at least 55 %, the PA cool and the keyed time
    /// within its budget. Its lease is renewed on every pass (0 stops it),
    /// so it stops by itself if this loop stops
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_transponder(&mut self, now: u64, state: u32) {
        let mut eps = EpsState::DEFAULT;
        // SAFETY: eps is a live EpsControlState_t
        let radio_on = unsafe { eps_control::eps_get_state(&mut eps) } == rc::OK && eps.radio_enabled;
        let inputs = transponder::Inputs {
            // The loss-of-contact inhibit stops the transponder even while
            // the beacon is kept for a failed receiver (REL-04)
            permitted: radio_on
                && rf_control::rf_control_tx_permitted()
                && !contact_monitor::contact_monitor_inhibited(),
            allowed: ops_state::ops_xpdr_allowed() && !self.gnss.session_on(),
            active: state == STATE_ACTIVE,
            sunlit: self.power.sunlit,
            soc: self.power.soc,
            pa_c: adc_or_nan(ADC_CHANNEL_TX_TEMP),
            fwd_w: adc_or_nan(ADC_CHANNEL_TX_FWD),
        };
        let change = self.transponder.step(now, &inputs);
        // SAFETY: plain values (the HAL)
        let _ = unsafe { crate::c::hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, self.transponder.heartbeat_lease_ms()) };
        match change {
            transponder::Change::None => {}
            transponder::Change::On => crate::c::console(format_args!("[XPDR] Transponder on")),
            transponder::Change::Off(why) => crate::c::console(format_args!("[XPDR] Transponder off: {}", why.text())),
        }
    }

    /// GNSS receiver sessions (`sqso_fsw::gnss_schedule`, decision D6): the
    /// load switch for a session, the receiver's NMEA into the parser while
    /// it runs. Gates: Jetson off, SOC at least 40 %, not SAFE or EMERGENCY
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_gnss(&mut self, now: u64, state: u32) {
        if GNSS_REQUESTED.swap(false, Ordering::Relaxed) {
            self.gnss.request();
        }
        let operating = matches!(state, STATE_DETUMBLE | STATE_IDLE | STATE_ACTIVE);
        let inputs = gnss_schedule::Inputs {
            leop: ops_state::ops_commissioning_hold(),
            allowed: operating && !eps_control::eps_is_payload_enabled() && self.power.soc >= GNSS_SOC_MIN,
            fix_ms: self.gnss_rx.fix.valid.then_some(self.gnss_rx.fix.rx_ms),
        };
        match self.gnss.step(now, &inputs) {
            gnss_schedule::Action::None => {}
            gnss_schedule::Action::PowerOn => {
                crate::c::console(format_args!("[GNSS] Session {} on", self.gnss.sessions));
                // SAFETY: plain values (the HAL)
                let _ = unsafe { crate::c::hal_gpio_set(GPIO_PIN_GNSS_EN, true) };
            }
            gnss_schedule::Action::PowerOff(outcome) => {
                // SAFETY: plain values (the HAL)
                let _ = unsafe { crate::c::hal_gpio_set(GPIO_PIN_GNSS_EN, false) };
                // SAFETY: plain value (the HAL)
                let _ = unsafe { crate::c::hal_uart_flush(UART_PORT_GNSS) };
                crate::c::console(format_args!("[GNSS] Session off: {}", outcome.text()));
                if self.gnss.no_fix_fault() {
                    add_fault(fault_type::INIT, severity::WARNING, c"GNSS: no fix in three sessions", self.power.soc);
                }
            }
        }
        if !self.gnss.session_on() {
            return;
        }
        // What arrived since the last pass, at most 512 bytes (at 9600 baud a
        // 20 ms pass brings about 19)
        let mut chunk = [0u8; 64];
        for _ in 0..8 {
            let mut got = 0usize;
            // SAFETY: chunk is a live buffer of 64 bytes; got a live size_t
            let read = unsafe { crate::c::hal_uart_read(UART_PORT_GNSS, chunk.as_mut_ptr(), chunk.len(), 0, &mut got) };
            if read != rc::OK || got == 0 {
                break;
            }
            for &b in chunk.get(..got).unwrap_or(&[]) {
                let _ = self.gnss_rx.feed(b, now);
            }
        }
    }

    /// The housekeeping frame (`sqso_fsw::housekeeping`, LV-27): every fifth
    /// beacon and on GET_HOUSEKEEPING, 5 s after the beacon, through the
    /// downlink (which enforces the inhibits and the duty limit)
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_housekeeping(&mut self, now: u64, state: u32) {
        if HK_REQUESTED.swap(false, Ordering::Relaxed) {
            self.hk.request(now);
        }
        if !self.hk.due(now) {
            return;
        }
        let hk = self.housekeeping(now, state);
        let mut info = [0u8; housekeeping::MAX_LEN];
        let n = housekeeping::format(&hk, &mut info);
        let mut frame = [0u8; ax25::MAX_FRAME_LEN];
        let dest = Station { call: "CQ", ssid: 0 };
        let src = Station { call: sqso_protocol::station::CALLSIGN, ssid: sqso_protocol::station::SSID };
        let Ok(len) = ax25::encode_ui(dest, src, info.get(..n).unwrap_or(&[]), false, &mut frame) else {
            self.hk.sent(); // cannot happen for MAX_LEN bytes; drop it
            return;
        };
        // SAFETY: frame holds len bytes
        match unsafe { downlink::downlink_transmit(frame.as_ptr(), len) } {
            rc::BUSY => {} // the downlink is sending: next pass
            sent => {
                if sent == rc::OK {
                    crate::c::console_parts(&[b"[HK] TX: ", info.get(..n).unwrap_or(&[])]);
                }
                self.hk.sent();
            }
        }
    }

    /// The housekeeping values now
    fn housekeeping(&self, now: u64, state: u32) -> Housekeeping {
        let eps = eps_control::eps_telemetry();
        let mut last = Entry::ZERO;
        // SAFETY: last is a live FaultLogEntry_t
        let have_last = unsafe { fault_mgmt::fault_log_get_last(&mut last) } == rc::OK;
        let mut adcs = sqso_fsw::adcs::Telemetry::default();
        // SAFETY: adcs is a live AdcsTelemetry_t
        let have_adcs = unsafe { adcs_control::adcs_get_telemetry(&mut adcs) } == rc::OK;
        let finite = |v: f64| v.is_finite().then_some(v);
        // SAFETY (the block): HAL and module accessors without pointer arguments
        unsafe {
            Housekeeping {
                seq: self.hk.seq,
                uptime_s: now.wrapping_sub(self.program_start_ms) / 1000,
                boots: mission_data::mission_data_get_reset_count(),
                reset_cause: crate::c::hal_reset_cause(),
                mode: state,
                hold: ops_state::ops_commissioning_hold(),
                xpdr: self.transponder.enabled,
                tx_permitted: rf_control::rf_control_tx_permitted(),
                soc: Some(self.power.soc),
                vbatt_v: eps.map(|t| t.vbatt_v),
                ibatt_a: eps.map(|t| t.ibatt_a),
                // The array current is measured at the bus (the solar INA228
                // on the battery side of the trackers): power = VBATT x ISOLAR
                array_w: eps.map(|t| t.vbatt_v * t.isolar_a),
                tbatt_c: eps.map(|t| t.tbatt_c),
                tobc_c: finite(adc_or_nan(ADC_CHANNEL_TEMP_MCU)),
                tpa_c: finite(adc_or_nan(ADC_CHANNEL_TX_TEMP)),
                tjet_c: fresh_sensor(c"PT", now, BEACON_MEAS_MAX_AGE_MS),
                deploy: (
                    deployment::deployment_get_state(),
                    crate::ants::status().map(|a| a.deployed_mask()),
                    crate::c::hal_gpio_get(GPIO_PIN_SOLAR_SENSE),
                ),
                rf_silence: (rf_silence::rf_silence_get_state(), rf_silence::rf_silence_elapsed_seconds()),
                contact_timeout_d: contact_monitor::contact_monitor_get_timeout_days(),
                since_contact_s: contact_monitor::contact_monitor_seconds_since_contact(),
                fault_count: u32::try_from(fault_mgmt::fault_log_get_count()).unwrap_or(u32::MAX),
                last_fault: have_last.then_some((last.fault_type, last.severity)),
                rate_cdps: if have_adcs { [adcs.rate_x, adcs.rate_y, adcs.rate_z] } else { [0; 3] },
                adcs_mode: adcs.mode,
                mag_rejected: adcs_control::mag_rejected(),
                gnss_fix: self
                    .gnss_rx
                    .fix
                    .valid
                    .then(|| (now.saturating_sub(self.gnss_rx.fix.rx_ms) / 1000, self.gnss_rx.fix.sats)),
                gnss_sessions: (self.gnss.sessions, self.gnss.fixes),
                passivation: passivation::passivation_phase(),
                rx: (receiver::receiver_cycles(), receiver::receiver_failed()),
                contacts: cmd_handler::contacts(),
                stored: stored_counts(),
            }
        }
    }

    /// Stored housekeeping (`hk_store.h`, DB-01): an event on a mode change
    /// or a new fault at ERROR or worse; the values when a record is due
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_store(&mut self, now: u64, state: u32) {
        if state != self.last_state {
            self.last_state = state;
            hk_store::hk_store_note_event(sqso_fsw::hk_store::EVENT_MODE);
        }
        let faults = fault_mgmt::fault_log_get_count();
        if faults != self.last_fault_count {
            let mut last = Entry::ZERO;
            // SAFETY: last is a live FaultLogEntry_t
            let have = unsafe { fault_mgmt::fault_log_get_last(&mut last) } == rc::OK;
            if faults > self.last_fault_count && have && u32::from(last.severity) >= severity::ERROR {
                hk_store::hk_store_note_event(sqso_fsw::hk_store::EVENT_FAULT);
            }
            self.last_fault_count = faults;
        }
        if hk_store::due(now) {
            let hk = self.housekeeping(now, state);
            let extra = sqso_fsw::hk_store::Extra {
                heater: self.heater.on,
                payload: eps_control::eps_is_payload_enabled(),
                contact_inhibit: contact_monitor::contact_monitor_inhibited(),
                passivated: passivation::passivation_passivated(),
            };
            hk_store::set_sample(&hk, &extra);
        }
        hk_store::hk_store_step(now);
    }

    /// The end-of-life sequence, the receiver's supervision, the lockout
    /// pulse and, every minute, the check of the critical persisted state
    /// against its records (AV-07)
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_safety_checks(&mut self, now: u64) {
        passivation::passivation_tick(self.power.sunlit);
        let mut dl = downlink::DownlinkStatus::default();
        // SAFETY: dl is a live DownlinkStatus_t
        let _ = unsafe { downlink::downlink_get_status(&mut dl) };
        receiver::receiver_step(dl.keyed || self.transponder.enabled);
        eps_lockout::eps_lockout_poll();
        if now.wrapping_sub(self.last_verify_ms) >= VERIFY_INTERVAL_MS {
            self.last_verify_ms = now;
            rf_control::rf_control_verify();
            ops_state::ops_state_verify();
            passivation::passivation_verify();
        }
    }

    /// With the EPS controller answering, the Jetson in batches
    /// (`sqso_fsw::payload_schedule`); on the stand-in power model it
    /// follows the power mode
    #[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
    fn run_payload_schedule(&mut self, now: u64, state: u32) {
        let measured = eps_control::eps_telemetry().is_some();
        eps_control::eps_hold_payload(measured);
        if !measured {
            return;
        }
        let on = eps_control::eps_is_payload_enabled();
        match self.schedule.step(now, on, state == STATE_ACTIVE, self.ai_cached) {
            Action::None => {}
            Action::PowerOn => {
                crate::c::console(format_args!("[PAYLOAD] Batch: Jetson on"));
                let _ = eps_control::eps_control_payload(true, self.power.soc);
            }
            Action::PowerOff => {
                crate::c::console(format_args!("[PAYLOAD] Batch done: {} lines cached", self.ai_cached));
                let _ = eps_control::eps_control_payload(false, self.power.soc);
            }
        }
    }

    /// Save everything and close the Jetson link
    pub fn shutdown(self) {
        crate::c::console(format_args!("[SYSTEM] Shutting down gracefully..."));
        let _ = mission_data::mission_data_save();
        let _ = eps_control::eps_save_config();
        let _ = fault_mgmt::fault_log_save();
        let _ = uart_comm::uart_close();

        let mut mission = MissionData::ZERO;
        // SAFETY: mission is a live MissionData_t
        let _ = unsafe { mission_data::mission_data_get(&mut mission) };
        crate::c::console(format_args!(
            "[SYSTEM] Shutdown complete. Total uptime: {} ms, Faults: {}",
            mission.total_uptime_ms, mission.fault_count
        ));
    }
}

/// Radio safety and telecommand, before anything can transmit: the
/// post-deployment silence (CDS 2.4.5) and the loss-of-contact inhibit first,
/// so the radio lines are never driven high on a boot that must stay silent;
/// then the latched TX-OFF state (47 CFR 97.207(b)), the key table (with no
/// key only TX-OFF is accepted), the receiver and the GSE port
/// Whether 120 s have passed since separation (LV-02): the deployment wait
/// counts from separation, and every later state is past its 30 minutes. A
/// power-losing reset during the wait restarts the wait, and this delay with
/// it (the conservative direction)
fn separation_delay_served() -> bool {
    match deployment::deployment_get_state() {
        DEPLOY_STATE_INHIBITED | DEPLOY_STATE_ARMED => false,
        DEPLOY_STATE_WAITING => deployment::deployment_wait_elapsed_s() >= MTQ_SEPARATION_DELAY_S,
        _ => true,
    }
}

/// One ADC channel, volts (or its unit); not a number if the read fails
fn adc_or_nan(channel: u32) -> f64 {
    let mut value = 0.0;
    // SAFETY: value is a live f64
    if unsafe { crate::c::hal_adc_read_voltage(channel, &mut value) } == rc::OK { value } else { f64::NAN }
}

/// The next record number of each store (HK1 `ST`)
fn stored_counts() -> [u32; 2] {
    let (mut a, mut b) = (0u32, 0u32);
    // SAFETY: both are live u32
    let _ = unsafe { hk_store::hk_store_counts(&mut a, &mut b) };
    [a, b]
}

/// A UI frame CQ <- KK6GDB-11 carrying `info`, handed to the downlink: its
/// result (`BUSY`: try again next pass)
fn send_ui(info: &[u8]) -> i32 {
    let mut frame = [0u8; ax25::MAX_FRAME_LEN];
    let dest = Station { call: "CQ", ssid: 0 };
    let src = Station { call: sqso_protocol::station::CALLSIGN, ssid: sqso_protocol::station::SSID };
    let Ok(len) = ax25::encode_ui(dest, src, info, false, &mut frame) else { return rc::INVALID };
    // SAFETY: frame holds len bytes
    unsafe { downlink::downlink_transmit(frame.as_ptr(), len) }
}

/// The command acknowledgements (GND-03), one a pass
#[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
fn run_acks(now: u64) {
    let Some(ack) = cmd_handler::next_ack(now) else { return };
    let mut info = [0u8; sqso_fsw::ack::MAX_LEN];
    let n = sqso_fsw::ack::format(&ack, &mut info);
    let text = info.get(..n).unwrap_or(&[]);
    match send_ui(text) {
        rc::BUSY => {} // the downlink is sending: next pass
        sent => {
            if sent == rc::OK {
                crate::c::console_parts(&[b"[ACK] TX: ", text]);
            }
            cmd_handler::ack_sent();
        }
    }
}

/// A stored-housekeeping dump frame (DUMP_TLM) when one is due
#[inline(never)] // a stage of step(): only the deepest one's frame counts (SRS-C003)
fn run_dump(now: u64) {
    let mut info = [0u8; sqso_fsw::hk_store::DUMP_INFO_MAX];
    // SAFETY: info is a live buffer of its length
    let n = unsafe { hk_store::hk_store_dump_peek(now, info.as_mut_ptr(), info.len()) };
    if n == 0 {
        return;
    }
    match send_ui(info.get(..n).unwrap_or(&[])) {
        rc::BUSY => {}
        rc::OK => hk_store::hk_store_dump_commit(now),
        _ => {
            crate::c::console(format_args!("[HKD] Dump stopped: downlink not permitted"));
            hk_store::hk_store_dump_stop();
        }
    }
}

/// The state machine's state (BOOT if it cannot be read)
fn sm_state() -> u32 {
    let mut state = STATE_BOOT;
    // SAFETY: the system state's context; state is a live u32
    let _ = unsafe { state_machine::sm_get_state(system_state::sys_get_sm_context(), &mut state) };
    state
}

fn init_rf_and_telecommand() {
    let _ = rf_silence::rf_silence_init();
    let _ = contact_monitor::contact_monitor_init();
    // A passivated spacecraft's inhibit is set before the lines are driven
    let _ = passivation::passivation_init();
    let _ = rf_control::rf_control_init();
    if downlink::downlink_init() != rc::OK {
        eprint(format_args!("[SYSTEM] Downlink init failed"));
    }
    if cmd_handler::cmd_handler_init() != rc::OK {
        eprint(format_args!("[SYSTEM] Command handler init failed"));
    }
    // SAFETY: a NUL-terminated call sign
    if unsafe { uplink::uplink_init(c"KK6GDB".as_ptr(), sqso_protocol::station::SSID) } != rc::OK {
        eprint(format_args!("[SYSTEM] Telecommand receiver init failed"));
    }
    if gse_provision::gse_init() != rc::OK {
        eprint(format_args!("[SYSTEM] GSE provisioning port init failed"));
    }
}

/// The sensor table: sensors.yaml on the host ($SENSORS_YAML), the built-in
/// table if that cannot be loaded and always on the target (no file system)
fn load_sensor_table() -> bool {
    #[cfg(not(target_os = "none"))]
    {
        let path = crate::os::env(c"SENSORS_YAML").unwrap_or(c"software/flight/sensors.yaml");
        // SAFETY: path is NUL-terminated
        if unsafe { sensors::sensors_load_yaml(path.as_ptr()) } == rc::OK {
            return true;
        }
        eprint(format_args!("[WARN] Using default sensors (could not load {})", as_str(path.to_bytes())));
    }
    if sensors::sensors_load_defaults() != rc::OK {
        eprint(format_args!("[SYSTEM] Failed to load default sensors"));
        return false;
    }
    true
}

/// Arm the supervised hardware watchdog (SW-15): it is refreshed only
/// while every stage of the main loop checks in; a stalled stage stops the
/// refresh and the hardware resets the OBC after saving state
fn init_watchdog() -> bool {
    if watchdog_mgr::wdt_mgr_init() != rc::OK {
        return false;
    }
    let tasks: [(u32, &CStr, u64); 5] = [
        (WDT_TASK_MAIN_LOOP, c"main", 1000),
        (WDT_TASK_COMM, c"uplink", 1000),
        (WDT_TASK_PAYLOAD, c"payload", 1000),
        (WDT_TASK_BEACON, c"beacon", 1000),
        (WDT_TASK_HEALTH, c"health", 2 * exec::HEALTH_CHECK_INTERVAL_MS),
    ];
    for (id, name, period) in tasks {
        let period = u32::try_from(period).unwrap_or(u32::MAX);
        // SAFETY: name is NUL-terminated
        let _ = unsafe { watchdog_mgr::wdt_mgr_register_task(id, name.as_ptr(), period) };
    }
    let _ = watchdog_mgr::wdt_mgr_register_prereset_callback(Some(save_state_before_reset));
    watchdog_mgr::wdt_mgr_start() == rc::OK
}

fn send_line(line: &[u8], what: &str) {
    if line.is_empty() {
        eprint(format_args!("[UART] {what} line not built"));
        return;
    }
    // SAFETY: line is a live slice
    if unsafe { uart_comm::uart_send(line.as_ptr().cast(), line.len()) } == rc::OK {
        crate::c::console(format_args!("[UART] Sent {what} to Jetson ({} bytes)", line.len()));
    } else {
        eprint(format_args!("[UART] Failed to send {what} to Jetson"));
    }
}

/// Append ",key=value" fields for EPS switches, last fault and health
fn append_status_fields(line: &mut Text<LINE_MAX>, room: usize) {
    let mut eps = EpsState::DEFAULT;
    // SAFETY: eps is a live EpsControlState_t
    if unsafe { eps_control::eps_get_state(&mut eps) } == rc::OK {
        let _ = line.append(
            room,
            format_args!(
                "EPS_PAYLOAD={},EPS_RADIO={},EPS_ADCS={},EPS_BEACON={},",
                on_off(eps.payload_enabled),
                on_off(eps.radio_enabled),
                on_off(eps.adcs_enabled),
                on_off(eps.beacon_enabled)
            ),
        );
    }

    // Fault information (description sanitized: no commas inside a field)
    let mut last = Entry::ZERO;
    // SAFETY: last is a live FaultLogEntry_t
    if unsafe { fault_mgmt::fault_log_get_last(&mut last) } == rc::OK {
        let mut description = last.description;
        let len = field(&description).len();
        if let Some(text) = description.get_mut(..len) {
            exec::sanitize_field(text);
        }
        let _ = line.append(
            room,
            format_args!(
                "LAST_FAULT={},FAULT_COUNT={},",
                as_str(description.get(..len).unwrap_or(&[])),
                fault_mgmt::fault_log_get_count()
            ),
        );
    }

    let _ = line.append(
        room,
        format_args!(
            "UART_HEALTH={},WATCHDOG={}",
            if uart_comm::uart_is_initialized() { "OK" } else { "FAIL" },
            if watchdog_mgr::wdt_mgr_was_wdt_reset() || watchdog_mgr::wdt_mgr_any_task_stalled() {
                "TRIGGERED"
            } else {
                "OK"
            }
        ),
    );
}

#[cfg(not(target_os = "none"))]
mod host {
    use core::sync::atomic::{AtomicBool, Ordering};

    static SHUTDOWN: AtomicBool = AtomicBool::new(false);

    /// `$SIM_RF_SILENCE_SERVED` set: boot as after a software reset with the
    /// 45-minute post-deployment silence already served (a COMPLETE record
    /// in the simulated FRAM), so that a 20 s run beacons. The silence's own
    /// rules decide (`rf_silence_init()`); simulation only
    pub(super) fn serve_rf_silence() {
        use sqso_fsw::nvm;
        use sqso_fsw::rf_silence::{DURATION_S, HAL_RESET_SOFTWARE, RECORD_MAGIC, RfSilence, State};
        if crate::os::env(c"SIM_RF_SILENCE_SERVED").is_none() {
            return;
        }
        // SAFETY: no arguments (the simulated HAL)
        let _ = unsafe { crate::c::hal_flash_init() };
        let record = RfSilence::record(State::Complete, DURATION_S);
        let region = crate::rf_silence::FLASH_REGION_RF_SILENCE;
        let _ = nvm::save(&mut crate::c::HalFlash, region, 0, RECORD_MAGIC, &record);
        crate::hal_sim::hal_sim_set_reset_cause(HAL_RESET_SOFTWARE);
    }

    extern "C" fn on_signal(_sig: libc::c_int) {
        SHUTDOWN.store(true, Ordering::Relaxed);
    }

    /// SIGINT and SIGTERM end the loop (state is saved)
    pub(super) fn install_signal_handlers() {
        let handler = on_signal as extern "C" fn(libc::c_int) as libc::sighandler_t;
        for (sig, name) in [(libc::SIGINT, "SIGINT"), (libc::SIGTERM, "SIGTERM")] {
            // SAFETY: the handler only stores to an atomic
            if unsafe { libc::signal(sig, handler) } == libc::SIG_ERR {
                super::eprint(format_args!("[SYSTEM] Failed to set {name} handler"));
            }
        }
    }

    pub(super) fn shutdown_requested() -> bool {
        SHUTDOWN.load(Ordering::Relaxed)
    }

    /// Seed the simulated sensor noise
    pub(super) fn seed_noise() {
        // SAFETY: plain calls; time(NULL) has no pointer to write
        unsafe {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // any seed will do
            libc::srand(libc::time(core::ptr::null_mut()) as libc::c_uint);
        }
    }
}

/// The host simulation (`smart_qso_flight`): bring up, run `max_ticks`
/// passes 20 ms apart (all of them with `None`, until SIGINT or SIGTERM),
/// shut down. Returns the process exit status.
#[cfg(not(target_os = "none"))]
pub fn run_host(max_ticks: Option<u64>) -> i32 {
    host::install_signal_handlers();
    host::seed_noise();
    host::serve_rf_silence();
    let mut x = Executive::BOOT;
    if !x.start() {
        return 1;
    }
    while max_ticks.is_none_or(|max| x.ticks() < max) && !host::shutdown_requested() {
        x.step();
        crate::os::sleep_us(exec::TICK_MS * 1000);
    }
    x.shutdown();
    0
}
