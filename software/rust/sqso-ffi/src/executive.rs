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

use core::ffi::{CStr, c_char};
use core::fmt::{self, Write};
use core::ptr::{null, null_mut};

use sqso_fsw::beacon::{Content, Measurements, TLM_BV, TLM_SOC, TLM_SUN};
use sqso_fsw::eps::EpsState;
use sqso_fsw::executive::{self as exec, PowerModel};
use sqso_fsw::fault_log::Entry;
use sqso_fsw::mission_data::MissionData;
use sqso_fsw::payload_guard::{Outcome, Sample, TELEMETRY_TIMEOUT_MS};
use sqso_protocol::jetson::{LINE_MAX, TEXT_MAX};

use crate::c::{fault_type, rc, severity};
use crate::jetson_link::{JetsonMsg, JetsonRx};
use crate::sensors::Sensor;
use crate::{
    beacon, cmd_handler, contact_monitor, deployment, eps_control, fault_mgmt, gse_provision, jetson_link,
    mission_data, payload_guard, rf_control, rf_silence, sensors, state_machine, system_state, uart_comm, uplink,
    watchdog_mgr,
};

// watchdog_mgr.h task ids
const WDT_TASK_MAIN_LOOP: u32 = 0;
const WDT_TASK_COMM: u32 = 3;
const WDT_TASK_BEACON: u32 = 5;
const WDT_TASK_PAYLOAD: u32 = 6;
const WDT_TASK_HEALTH: u32 = 7;

// state_machine.h
const STATE_BOOT: u32 = 0;
const EVENT_NONE: u32 = 0;
const EVENT_BOOT_COMPLETE: u32 = 1;
const EVENT_DETUMBLE_COMPLETE: u32 = 2;
const EVENT_THERMAL_FAULT: u32 = 7;

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
    power: PowerModel,
    jetson_temp_c: i32,
    jetson_status_ms: u64,
    jetson_status_seen: bool,
    /// Beacon text most recently received from the Jetson (empty: use a template)
    ai_text: [u8; TEXT_MAX + 1],
    beacon_seq: u16,
    jetson_rx: JetsonRx,
}

impl Executive {
    /// Bring the system up; `None` (after reporting why) if it cannot run
    pub fn start() -> Option<Executive> {
        let mut x = Executive {
            tick: 0,
            telemetry_interval_ms: 0,
            program_start_ms: 0,
            last_telemetry_ms: 0,
            last_health_check_ms: 0,
            last_contact_tick_ms: 0,
            power: PowerModel::START,
            jetson_temp_c: 0,
            jetson_status_ms: 0,
            jetson_status_seen: false,
            ai_text: [0; TEXT_MAX + 1],
            beacon_seq: 0,
            jetson_rx: JetsonRx::EMPTY,
        };
        if !x.init_system() {
            eprint(format_args!("[SYSTEM] Initialization failed"));
            return None;
        }
        x.announce();
        Some(x)
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
        let _ = deployment::deployment_init();
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

        self.last_telemetry_ms = self.program_start_ms;
        self.last_health_check_ms = self.program_start_ms;
        self.last_contact_tick_ms = self.program_start_ms;
        self.telemetry_interval_ms = uart_comm::uart_get_telemetry_interval();
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
    /// The loop has no ADCS yet, so detumble completes at once.
    fn init_mode_manager(&mut self) -> bool {
        let _ = beacon::beacon_init(); // templates until the Jetson's text passes the filter
        payload_guard::payload_guard_init();
        eps_control::eps_update_soc(self.power.soc);
        if system_state::sys_state_init() != rc::OK {
            eprint(format_args!("[SYSTEM] System state init failed"));
            return false;
        }
        sm_event(EVENT_BOOT_COMPLETE);
        sm_event(EVENT_DETUMBLE_COMPLETE);
        true
    }

    fn announce(&self) {
        let mut mission = MissionData::ZERO;
        // SAFETY: mission is a live MissionData_t
        let _ = unsafe { mission_data::mission_data_get(&mut mission) };
        crate::c::console(format_args!("[SYSTEM] SMART-QSO Flight Software Started"));
        crate::c::console(format_args!("[SYSTEM] Version: {VERSION}"));
        crate::c::console(format_args!(
            "[SYSTEM] Mission Phase: {}, Power Mode: {}, SOC: {:.1}%",
            mission.mission_phase,
            eps_control::eps_get_power_mode(),
            self.power.soc * 100.0
        ));
    }

    /// Passes run so far
    pub fn ticks(&self) -> u64 {
        self.tick
    }

    /// One pass of the main loop; each stage checks in with the watchdog
    pub fn step(&mut self) {
        let tick = self.tick;
        self.tick = self.tick.wrapping_add(1);
        self.update_mission_state(tick);
        let now = crate::c::now_ms();

        self.monitor_system_health();
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_MAIN_LOOP);

        // Telemetry and beacon request to the Jetson, and a periodic UART check
        if eps_control::eps_is_payload_enabled()
            && uart_comm::uart_is_initialized()
            && now.wrapping_sub(self.last_telemetry_ms) >= u64::from(self.telemetry_interval_ms)
        {
            self.send_telemetry_to_jetson();
            self.last_telemetry_ms = now;
        }
        if uart_comm::uart_is_initialized() && (now % 10_000) < 100 {
            let _ = uart_comm::uart_check_health();
        }
        let _ = sensors::sensors_poll(now);

        // Jetson replies (beacon text, status), then the OBC's payload limits
        self.poll_jetson();
        self.guard_payload(now);
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_PAYLOAD);

        // Beacon (stays silent while TX-OFF, contact or RF-silence inhibits)
        self.update_beacon_measurements(now);
        self.run_beacon(now);
        let _ = watchdog_mgr::wdt_mgr_checkin(WDT_TASK_BEACON);

        // Telecommands from the 435 MHz receiver, then key provisioning
        // (umbilical only)
        // SAFETY: NULL: no response wanted
        let _ = unsafe { uplink::uplink_poll(null_mut()) };
        let _ = gse_provision::gse_poll();
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

    /// The stand-in power model, then mode changes and mission data
    fn update_mission_state(&mut self, tick: u64) {
        let payload_on = eps_control::eps_is_payload_enabled();
        let discharge = self.power.step(tick, payload_on);
        sensors::sensors_set_environment(self.power.sunlit, self.power.soc);

        // The EPS holds the measured SOC; the state machine turns power
        // conditions into transitions, and its entry actions set the EPS
        // power mode (SW-14)
        eps_control::eps_update_soc(self.power.soc);
        let mut state = STATE_BOOT;
        // SAFETY: the system state's context; state is a live u32
        if unsafe { state_machine::sm_get_state(system_state::sys_get_sm_context(), &mut state) } == rc::OK {
            let event = state_machine::sm_power_event(state, self.power.soc, self.power.sunlit);
            if event != EVENT_NONE {
                sm_event(event);
            }
        }

        let now = crate::c::now_ms();
        let _ = mission_data::mission_data_update_uptime(now.wrapping_sub(self.program_start_ms));
        if let Some(wh) = self.power.payload_energy_wh(discharge, payload_on) {
            let _ = mission_data::mission_data_add_energy(wh);
        }
    }

    /// Every 10 s: temperatures, bus voltage, SOC and the Jetson link
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

    fn send_telemetry_to_jetson(&mut self) {
        if !eps_control::eps_is_payload_enabled() || !uart_comm::uart_is_initialized() {
            return;
        }

        // ICD_OBC_JETSON section 4: TELEMETRY,<fields...>,<CRC>\n
        let mut line = Text::<LINE_MAX>::new();
        let room = LINE_MAX - 6; // ",XXXX\n"
        if !line.append(
            room,
            format_args!(
                "TELEMETRY,{},{},{:.3},{},{:.1},",
                crate::c::now_ms(),
                if self.power.sunlit { "SUNLIT" } else { "ECLIPSE" },
                self.power.soc,
                eps_control::eps_get_power_mode(),
                eps_control::eps_get_power_limit()
            ),
        ) {
            return;
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

        // Ask for the next beacon text
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
                let text = field(&msg.text);
                self.ai_text = [0; TEXT_MAX + 1];
                if let Some(dst) = self.ai_text.get_mut(..text.len().min(TEXT_MAX)) {
                    dst.copy_from_slice(text.get(..dst.len()).unwrap_or(&[]));
                }
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
    fn run_beacon(&mut self, now: u64) {
        beacon::beacon_update_interval(eps_control::eps_get_power_mode());
        if !beacon::beacon_should_transmit(now) {
            return;
        }
        let mut content = Content::ZERO;
        let ai_len = field(&self.ai_text).len();
        let ai: *const c_char = if ai_len > 0 { self.ai_text.as_ptr().cast() } else { null() };
        // SAFETY: content is a live BeaconContent_t; ai is NULL or ai_len bytes
        if unsafe { beacon::beacon_generate_content(&mut content, ai, ai_len) } == rc::OK
            && unsafe { beacon::beacon_transmit(&content) } == rc::OK
        {
            beacon::beacon_mark_transmitted(now);
        }
        self.ai_text = [0; TEXT_MAX + 1];
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
fn init_rf_and_telecommand() {
    let _ = rf_silence::rf_silence_init();
    let _ = contact_monitor::contact_monitor_init();
    let _ = rf_control::rf_control_init();
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
    let Some(mut x) = Executive::start() else { return 1 };
    while max_ticks.is_none_or(|max| x.ticks() < max) && !host::shutdown_requested() {
        x.step();
        crate::os::sleep_us(exec::TICK_MS * 1000);
    }
    x.shutdown();
    0
}
