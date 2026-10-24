//! System state (`system_state.h`): the power, thermal, ADCS,
//! communications and mission records behind accessors, with a CRC-32 over
//! the whole record and its persistent image.
//!
//! [`SystemState::image`] is the C `SystemState_t` exactly as it lies in
//! memory (264 bytes, padding zero), which is what is stored and what the
//! CRC covers. The state machine context is held by the caller (the C side
//! hands out a pointer to it) and passed in as [`Sm`].
//!
//! The power mode and the state of charge are the EPS's; this record does
//! not keep a copy of either (SW-14), so the caller fills them in on read.

use sqso_protocol::checksum::crc32;

/// Bytes of the C `SystemState_t`
pub const IMAGE_SIZE: usize = 264;
/// Bytes the CRC covers (`sizeof(SystemState_t) - sizeof(uint32_t)`; the
/// CRC field itself, zeroed, is inside them)
pub const CRC_COVERED: usize = IMAGE_SIZE - 4;
/// Offset of the `crc32` field
pub const CRC_OFFSET: usize = 256;

/// Default battery voltage before a measurement, V
pub const DEFAULT_BATTERY_V: f64 = 3.7;
/// Default temperatures of OBC, EPS, battery and payload, C
pub const DEFAULT_TEMP_C: f32 = 25.0;
/// Over-temperature limit, C (any sensor)
pub const OVER_TEMP_C: f32 = 60.0;
/// Under-temperature limit, C (any sensor)
pub const UNDER_TEMP_C: f32 = -20.0;
/// Default beacon interval, s
pub const DEFAULT_BEACON_INTERVAL_S: u16 = 60;
/// Shortest beacon interval accepted, s
pub const MIN_BEACON_INTERVAL_S: u16 = 10;
/// Longest beacon interval accepted, s
pub const MAX_BEACON_INTERVAL_S: u16 = 120;

/// `PowerState_t` (the power mode and SOC fields are the EPS's on read)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Power {
    /// Battery voltage, V
    pub battery_voltage: f64,
    /// Battery current, A
    pub battery_current: f64,
    /// State of charge, 0-1
    pub state_of_charge: f64,
    /// Solar panel power, W
    pub solar_power: f64,
    /// Energy accumulated, Wh
    pub total_energy_wh: f64,
    /// `PowerMode_t`
    pub power_mode: u32,
    /// Jetson payload enabled
    pub payload_enabled: bool,
    /// Time the current mode was entered, ms (low 32 bits)
    pub mode_entry_time_ms: u32,
}

/// `ThermalState_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Thermal {
    /// OBC temperature, C
    pub obc_temp_c: f32,
    /// EPS temperature, C
    pub eps_temp_c: f32,
    /// Battery temperature, C
    pub battery_temp_c: f32,
    /// Payload temperature, C
    pub payload_temp_c: f32,
    /// External temperature, C
    pub external_temp_c: f32,
    /// Heater on
    pub heater_enabled: bool,
    /// The last reading was above [`OVER_TEMP_C`]
    pub over_temp_flag: bool,
    /// The last reading was below [`UNDER_TEMP_C`]
    pub under_temp_flag: bool,
}

/// `AdcsState_t` (system_state.h)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Adcs {
    /// Magnetometer X, Y, Z, uT
    pub mag_ut: [f32; 3],
    /// Gyroscope X, Y, Z, deg/s
    pub gyro_dps: [f32; 3],
    /// Sun vector X, Y, Z
    pub sun_vector: [f32; 3],
    /// Detumble achieved
    pub detumbled: bool,
    /// Sun acquired
    pub sun_acquired: bool,
    /// Last ADCS update, ms (low 32 bits)
    pub last_update_ms: u32,
}

/// `CommState_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Comm {
    /// Packets transmitted
    pub packets_sent: u32,
    /// Packets received
    pub packets_received: u32,
    /// Failed transmissions
    pub packets_failed: u32,
    /// Last ground contact, ms (low 32 bits)
    pub last_ground_contact_ms: u32,
    /// Beacons transmitted
    pub beacon_count: u32,
    /// Beacon interval, s
    pub beacon_interval_s: u16,
    /// Link active
    pub comm_active: bool,
    /// Last RSSI, dBm
    pub rssi_dbm: i8,
}

/// `MissionState_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mission {
    /// Boot count
    pub boot_count: u32,
    /// Uptime, s
    pub uptime_s: u32,
    /// Total mission uptime, s
    pub total_uptime_s: u32,
    /// Mission elapsed time, ms
    pub mission_time_ms: u64,
    /// QSOs made
    pub qso_count: u32,
    /// Commands received
    pub command_count: u32,
    /// Anomalies detected
    pub anomaly_count: u32,
    /// Safe mode was entered
    pub safe_mode_entry: bool,
}

/// `SmContext_t`, as stored in the record
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sm {
    /// `SmState_t`
    pub current_state: u32,
    /// `SmState_t`
    pub previous_state: u32,
    /// Transitions made
    pub transition_count: u32,
    /// Time of the last transition, ms
    pub last_transition_ms: u64,
    /// Time the current state was entered, ms
    pub state_entry_ms: u64,
    /// Initialized
    pub is_initialized: bool,
}

/// The C `SystemState_t` image, with its alignment (what
/// `sys_get_full_state()` writes)
#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Image(pub [u8; IMAGE_SIZE]);

/// `SystemState_t`, without the state machine context (held by the caller)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SystemState {
    /// Power
    pub power: Power,
    /// Thermal
    pub thermal: Thermal,
    /// ADCS
    pub adcs: Adcs,
    /// Communications
    pub comm: Comm,
    /// Mission
    pub mission: Mission,
    /// Initialized
    pub initialized: bool,
    /// Watchdog status
    pub watchdog_ok: bool,
    /// Changed since the last save
    pub persistence_dirty: bool,
    /// Last change, ms
    pub last_update_ms: u64,
    /// Last save, ms
    pub last_persist_ms: u64,
    /// CRC-32 of the record (see [`SystemState::compute_crc`])
    pub crc32: u32,
}

/// Writes fields at their C offsets
struct Writer<'a>(&'a mut [u8; IMAGE_SIZE]);

impl Writer<'_> {
    fn put(&mut self, offset: usize, bytes: &[u8]) {
        if let Some(dst) = self.0.get_mut(offset..offset + bytes.len()) {
            dst.copy_from_slice(bytes);
        }
    }
    fn f64(&mut self, offset: usize, v: f64) {
        self.put(offset, &v.to_ne_bytes());
    }
    fn f32(&mut self, offset: usize, v: f32) {
        self.put(offset, &v.to_ne_bytes());
    }
    fn u64(&mut self, offset: usize, v: u64) {
        self.put(offset, &v.to_ne_bytes());
    }
    fn u32(&mut self, offset: usize, v: u32) {
        self.put(offset, &v.to_ne_bytes());
    }
    fn u16(&mut self, offset: usize, v: u16) {
        self.put(offset, &v.to_ne_bytes());
    }
    fn u8(&mut self, offset: usize, v: u8) {
        self.put(offset, &[v]);
    }
    fn bool(&mut self, offset: usize, v: bool) {
        self.u8(offset, u8::from(v));
    }
}

/// Reads fields at their C offsets
struct Reader<'a>(&'a [u8; IMAGE_SIZE]);

impl Reader<'_> {
    /// Element `i` of an f32 array at `base`
    fn f32_at(&self, base: usize, i: usize) -> f32 {
        self.f32(base + 4 * i)
    }

    fn get<const N: usize>(&self, offset: usize) -> [u8; N] {
        let mut out = [0u8; N];
        if let Some(src) = self.0.get(offset..offset + N) {
            out.copy_from_slice(src);
        }
        out
    }
    fn f64(&self, offset: usize) -> f64 {
        f64::from_ne_bytes(self.get(offset))
    }
    fn f32(&self, offset: usize) -> f32 {
        f32::from_ne_bytes(self.get(offset))
    }
    fn u64(&self, offset: usize) -> u64 {
        u64::from_ne_bytes(self.get(offset))
    }
    fn u32(&self, offset: usize) -> u32 {
        u32::from_ne_bytes(self.get(offset))
    }
    fn u16(&self, offset: usize) -> u16 {
        u16::from_ne_bytes(self.get(offset))
    }
    fn u8(&self, offset: usize) -> u8 {
        let [b] = self.get::<1>(offset);
        b
    }
    fn bool(&self, offset: usize) -> bool {
        self.u8(offset) != 0
    }
}

// Offsets of the C SystemState_t members (test_rust_contract.c checks the size)
const POWER: usize = 0;
const THERMAL: usize = 56;
const ADCS: usize = 80;
const COMM: usize = 124;
const MISSION: usize = 152;
const SM: usize = 192;
const FLAGS: usize = 232;
const LAST_UPDATE: usize = 240;
const LAST_PERSIST: usize = 248;

impl SystemState {
    /// All zero (before `sys_state_init()`)
    pub const ZERO: SystemState = SystemState {
        power: Power {
            battery_voltage: 0.0,
            battery_current: 0.0,
            state_of_charge: 0.0,
            solar_power: 0.0,
            total_energy_wh: 0.0,
            power_mode: 0,
            payload_enabled: false,
            mode_entry_time_ms: 0,
        },
        thermal: Thermal {
            obc_temp_c: 0.0,
            eps_temp_c: 0.0,
            battery_temp_c: 0.0,
            payload_temp_c: 0.0,
            external_temp_c: 0.0,
            heater_enabled: false,
            over_temp_flag: false,
            under_temp_flag: false,
        },
        adcs: Adcs {
            mag_ut: [0.0; 3],
            gyro_dps: [0.0; 3],
            sun_vector: [0.0; 3],
            detumbled: false,
            sun_acquired: false,
            last_update_ms: 0,
        },
        comm: Comm {
            packets_sent: 0,
            packets_received: 0,
            packets_failed: 0,
            last_ground_contact_ms: 0,
            beacon_count: 0,
            beacon_interval_s: 0,
            comm_active: false,
            rssi_dbm: 0,
        },
        mission: Mission {
            boot_count: 0,
            uptime_s: 0,
            total_uptime_s: 0,
            mission_time_ms: 0,
            qso_count: 0,
            command_count: 0,
            anomaly_count: 0,
            safe_mode_entry: false,
        },
        initialized: false,
        watchdog_ok: false,
        persistence_dirty: false,
        last_update_ms: 0,
        last_persist_ms: 0,
        crc32: 0,
    };

    /// The state `sys_state_init()` sets: zero, then the defaults (the
    /// caller initializes the state machine)
    pub fn defaults(now_ms: u64) -> SystemState {
        SystemState {
            power: Power { battery_voltage: DEFAULT_BATTERY_V, ..Power::default() },
            thermal: Thermal {
                obc_temp_c: DEFAULT_TEMP_C,
                eps_temp_c: DEFAULT_TEMP_C,
                battery_temp_c: DEFAULT_TEMP_C,
                payload_temp_c: DEFAULT_TEMP_C,
                external_temp_c: 0.0,
                ..Thermal::default()
            },
            comm: Comm { beacon_interval_s: DEFAULT_BEACON_INTERVAL_S, ..Comm::default() },
            initialized: true,
            watchdog_ok: true,
            persistence_dirty: false,
            last_update_ms: now_ms,
            ..SystemState::default()
        }
    }

    /// The record as the C struct lies in memory, padding zero
    pub fn image(&self, sm: &Sm) -> Image {
        let mut buf = [0u8; IMAGE_SIZE];
        let mut w = Writer(&mut buf);
        let p = &self.power;
        w.f64(POWER, p.battery_voltage);
        w.f64(POWER + 8, p.battery_current);
        w.f64(POWER + 16, p.state_of_charge);
        w.f64(POWER + 24, p.solar_power);
        w.f64(POWER + 32, p.total_energy_wh);
        w.u32(POWER + 40, p.power_mode);
        w.bool(POWER + 44, p.payload_enabled);
        w.u32(POWER + 48, p.mode_entry_time_ms);
        let t = &self.thermal;
        for (i, v) in
            [t.obc_temp_c, t.eps_temp_c, t.battery_temp_c, t.payload_temp_c, t.external_temp_c].into_iter().enumerate()
        {
            w.f32(THERMAL + 4 * i, v);
        }
        w.bool(THERMAL + 20, t.heater_enabled);
        w.bool(THERMAL + 21, t.over_temp_flag);
        w.bool(THERMAL + 22, t.under_temp_flag);
        let a = &self.adcs;
        for (i, v) in a.mag_ut.into_iter().chain(a.gyro_dps).chain(a.sun_vector).enumerate() {
            w.f32(ADCS + 4 * i, v);
        }
        w.bool(ADCS + 36, a.detumbled);
        w.bool(ADCS + 37, a.sun_acquired);
        w.u32(ADCS + 40, a.last_update_ms);
        let c = &self.comm;
        w.u32(COMM, c.packets_sent);
        w.u32(COMM + 4, c.packets_received);
        w.u32(COMM + 8, c.packets_failed);
        w.u32(COMM + 12, c.last_ground_contact_ms);
        w.u32(COMM + 16, c.beacon_count);
        w.u16(COMM + 20, c.beacon_interval_s);
        w.bool(COMM + 22, c.comm_active);
        w.put(COMM + 23, &c.rssi_dbm.to_ne_bytes());
        let m = &self.mission;
        w.u32(MISSION, m.boot_count);
        w.u32(MISSION + 4, m.uptime_s);
        w.u32(MISSION + 8, m.total_uptime_s);
        w.u64(MISSION + 16, m.mission_time_ms);
        w.u32(MISSION + 24, m.qso_count);
        w.u32(MISSION + 28, m.command_count);
        w.u32(MISSION + 32, m.anomaly_count);
        w.bool(MISSION + 36, m.safe_mode_entry);
        w.u32(SM, sm.current_state);
        w.u32(SM + 4, sm.previous_state);
        w.u32(SM + 8, sm.transition_count);
        w.u64(SM + 16, sm.last_transition_ms);
        w.u64(SM + 24, sm.state_entry_ms);
        w.bool(SM + 32, sm.is_initialized);
        w.bool(FLAGS, self.initialized);
        w.bool(FLAGS + 1, self.watchdog_ok);
        w.bool(FLAGS + 2, self.persistence_dirty);
        w.u64(LAST_UPDATE, self.last_update_ms);
        w.u64(LAST_PERSIST, self.last_persist_ms);
        w.u32(CRC_OFFSET, self.crc32);
        Image(buf)
    }

    /// The record and state machine context of an image (a flag byte other
    /// than 0 reads as true)
    pub fn from_image(image: &Image) -> (SystemState, Sm) {
        let r = Reader(&image.0);
        let state = SystemState {
            power: read_power(&r),
            thermal: read_thermal(&r),
            adcs: read_adcs(&r),
            comm: read_comm(&r),
            mission: read_mission(&r),
            initialized: r.bool(FLAGS),
            watchdog_ok: r.bool(FLAGS + 1),
            persistence_dirty: r.bool(FLAGS + 2),
            last_update_ms: r.u64(LAST_UPDATE),
            last_persist_ms: r.u64(LAST_PERSIST),
            crc32: r.u32(CRC_OFFSET),
        };
        (state, read_sm(&r))
    }

    /// CRC-32 of the record with its CRC field zero
    pub fn compute_crc(&self, sm: &Sm) -> u32 {
        let image = SystemState { crc32: 0, ..*self }.image(sm);
        image_crc(&image)
    }

    /// Mark the record changed
    pub fn mark_dirty(&mut self, now_ms: u64) {
        self.persistence_dirty = true;
        self.last_update_ms = now_ms;
    }

    /// Record a temperature (sensor 0 OBC, 1 EPS, 2 battery, 3 payload,
    /// 4 external); the fault flags follow this reading. False for an
    /// unknown sensor (nothing changes)
    pub fn set_temperature(&mut self, sensor: u8, temp_c: f32, now_ms: u64) -> bool {
        let t = &mut self.thermal;
        let slot = match sensor {
            0 => &mut t.obc_temp_c,
            1 => &mut t.eps_temp_c,
            2 => &mut t.battery_temp_c,
            3 => &mut t.payload_temp_c,
            4 => &mut t.external_temp_c,
            _ => return false,
        };
        *slot = temp_c;
        t.over_temp_flag = temp_c > OVER_TEMP_C;
        t.under_temp_flag = temp_c < UNDER_TEMP_C;
        self.mark_dirty(now_ms);
        true
    }

    /// Whether the last temperature reading was out of limits
    pub fn has_thermal_fault(&self) -> bool {
        self.thermal.over_temp_flag || self.thermal.under_temp_flag
    }

    /// Set the beacon interval; false (unchanged) outside 10-120 s
    pub fn set_beacon_interval(&mut self, interval_s: u16, now_ms: u64) -> bool {
        if !(MIN_BEACON_INTERVAL_S..=MAX_BEACON_INTERVAL_S).contains(&interval_s) {
            return false;
        }
        self.comm.beacon_interval_s = interval_s;
        self.mark_dirty(now_ms);
        true
    }
}

/// CRC-32 of an image's first [`CRC_COVERED`] bytes, its CRC field taken
/// as zero
pub fn image_crc(image: &Image) -> u32 {
    let mut bytes = image.0;
    if let Some(field) = bytes.get_mut(CRC_OFFSET..CRC_OFFSET + 4) {
        field.fill(0);
    }
    crc32(bytes.get(..CRC_COVERED).unwrap_or(&[]))
}

/// The stored CRC of an image
pub fn image_stored_crc(image: &Image) -> u32 {
    Reader(&image.0).u32(CRC_OFFSET)
}

/// A state of charge held to 0-1 (NaN passes, as in C)
pub fn clamp_soc(soc: f64) -> f64 {
    soc.clamp(0.0, 1.0)
}

/// Uptime bookkeeping (`sys_update_uptime()`): whole seconds since the last
/// update are added; the remainder carries to the next call
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Uptime {
    last_ms: u64,
}

impl Uptime {
    /// Never updated
    pub const fn new() -> Uptime {
        Uptime { last_ms: 0 }
    }

    /// Add the time since the last update to `mission`; whether anything
    /// was added
    pub fn update(&mut self, now_ms: u64, mission: &mut Mission) -> bool {
        if self.last_ms == 0 {
            self.last_ms = now_ms;
        }
        let elapsed_ms = now_ms.wrapping_sub(self.last_ms);
        #[allow(clippy::cast_possible_truncation)] // as the C (uint32_t) conversion
        let elapsed_s = (elapsed_ms / 1000) as u32;
        if elapsed_s == 0 {
            return false;
        }
        mission.uptime_s = mission.uptime_s.wrapping_add(elapsed_s);
        mission.total_uptime_s = mission.total_uptime_s.wrapping_add(elapsed_s);
        mission.mission_time_ms = mission.mission_time_ms.wrapping_add(elapsed_ms);
        self.last_ms = now_ms;
        true
    }
}

// Sections of the image (SystemState::from_image)

fn read_power(r: &Reader<'_>) -> Power {
    Power {
        battery_voltage: r.f64(POWER),
        battery_current: r.f64(POWER + 8),
        state_of_charge: r.f64(POWER + 16),
        solar_power: r.f64(POWER + 24),
        total_energy_wh: r.f64(POWER + 32),
        power_mode: r.u32(POWER + 40),
        payload_enabled: r.bool(POWER + 44),
        mode_entry_time_ms: r.u32(POWER + 48),
    }
}

fn read_thermal(r: &Reader<'_>) -> Thermal {
    Thermal {
        obc_temp_c: r.f32_at(THERMAL, 0),
        eps_temp_c: r.f32_at(THERMAL, 1),
        battery_temp_c: r.f32_at(THERMAL, 2),
        payload_temp_c: r.f32_at(THERMAL, 3),
        external_temp_c: r.f32_at(THERMAL, 4),
        heater_enabled: r.bool(THERMAL + 20),
        over_temp_flag: r.bool(THERMAL + 21),
        under_temp_flag: r.bool(THERMAL + 22),
    }
}

fn read_adcs(r: &Reader<'_>) -> Adcs {
    Adcs {
        mag_ut: [r.f32_at(ADCS, 0), r.f32_at(ADCS, 1), r.f32_at(ADCS, 2)],
        gyro_dps: [r.f32_at(ADCS, 3), r.f32_at(ADCS, 4), r.f32_at(ADCS, 5)],
        sun_vector: [r.f32_at(ADCS, 6), r.f32_at(ADCS, 7), r.f32_at(ADCS, 8)],
        detumbled: r.bool(ADCS + 36),
        sun_acquired: r.bool(ADCS + 37),
        last_update_ms: r.u32(ADCS + 40),
    }
}

fn read_comm(r: &Reader<'_>) -> Comm {
    Comm {
        packets_sent: r.u32(COMM),
        packets_received: r.u32(COMM + 4),
        packets_failed: r.u32(COMM + 8),
        last_ground_contact_ms: r.u32(COMM + 12),
        beacon_count: r.u32(COMM + 16),
        beacon_interval_s: r.u16(COMM + 20),
        comm_active: r.bool(COMM + 22),
        rssi_dbm: i8::from_ne_bytes([r.u8(COMM + 23)]),
    }
}

fn read_mission(r: &Reader<'_>) -> Mission {
    Mission {
        boot_count: r.u32(MISSION),
        uptime_s: r.u32(MISSION + 4),
        total_uptime_s: r.u32(MISSION + 8),
        mission_time_ms: r.u64(MISSION + 16),
        qso_count: r.u32(MISSION + 24),
        command_count: r.u32(MISSION + 28),
        anomaly_count: r.u32(MISSION + 32),
        safe_mode_entry: r.bool(MISSION + 36),
    }
}

fn read_sm(r: &Reader<'_>) -> Sm {
    Sm {
        current_state: r.u32(SM),
        previous_state: r.u32(SM + 4),
        transition_count: r.u32(SM + 8),
        last_transition_ms: r.u64(SM + 16),
        state_entry_ms: r.u64(SM + 24),
        is_initialized: r.bool(SM + 32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (SystemState, Sm) {
        let mut s = SystemState::defaults(1234);
        s.power.battery_current = -0.25;
        s.power.power_mode = 2;
        s.power.payload_enabled = true;
        s.power.mode_entry_time_ms = 77;
        s.adcs.mag_ut = [1.5, -2.5, 3.5];
        s.adcs.sun_vector[2] = 0.75;
        s.adcs.detumbled = true;
        s.comm.rssi_dbm = -90;
        s.comm.packets_failed = 3;
        s.mission.mission_time_ms = u64::MAX - 5;
        s.mission.safe_mode_entry = true;
        s.last_persist_ms = 99;
        let sm = Sm {
            current_state: 3,
            previous_state: 2,
            transition_count: 9,
            last_transition_ms: 10,
            state_entry_ms: 11,
            is_initialized: true,
        };
        (s, sm)
    }

    #[test]
    fn zero_is_default() {
        assert_eq!(SystemState::ZERO, SystemState::default());
        assert_eq!(SystemState::ZERO.image(&Sm::default()).0, [0; IMAGE_SIZE]);
    }

    #[test]
    fn image_round_trips() {
        let (s, sm) = sample();
        let image = s.image(&sm);
        assert_eq!(SystemState::from_image(&image), (s, sm));
    }

    #[test]
    fn image_has_the_c_layout() {
        let (s, sm) = sample();
        let image = s.image(&sm).0;
        assert_eq!(&image[0..8], &3.7f64.to_ne_bytes());
        assert_eq!(&image[40..44], &2u32.to_ne_bytes());
        assert_eq!(image[44], 1);
        assert_eq!(&image[56..60], &25.0f32.to_ne_bytes());
        assert_eq!(&image[80..84], &1.5f32.to_ne_bytes());
        assert_eq!(image[116], 1); // adcs.detumbled
        assert_eq!(&image[144..146], &60u16.to_ne_bytes()); // comm.beacon_interval_s
        assert_eq!(image[147], (-90i8).to_ne_bytes()[0]);
        assert_eq!(&image[192..196], &3u32.to_ne_bytes());
        assert_eq!(image[224], 1); // sm.is_initialized
        assert_eq!(&image[232..235], &[1, 1, 0]);
        assert_eq!(&image[240..248], &1234u64.to_ne_bytes());
        // Padding is zero
        for pad in
            [45..48, 52..56, 79..80, 118..120, 148..152, 164..168, 189..192, 204..208, 225..232, 235..240, 260..264]
        {
            assert!(image[pad.clone()].iter().all(|b| *b == 0), "{pad:?}");
        }
    }

    #[test]
    fn crc_covers_everything_but_itself() {
        let (mut s, sm) = sample();
        s.crc32 = s.compute_crc(&sm);
        assert_eq!(image_crc(&s.image(&sm)), s.crc32);
        assert_eq!(image_stored_crc(&s.image(&sm)), s.crc32);
        let before = s.crc32;
        s.crc32 = 0xDEAD_BEEF; // the stored CRC does not enter the CRC
        assert_eq!(s.compute_crc(&sm), before);
        let mut changed = s;
        changed.mission.qso_count += 1;
        assert_ne!(changed.compute_crc(&sm), before);
        let sm2 = Sm { transition_count: 10, ..sm };
        assert_ne!(s.compute_crc(&sm2), before);
    }

    #[test]
    fn temperature_flags_follow_the_last_reading() {
        let mut s = SystemState::defaults(0);
        assert!(s.set_temperature(0, 70.0, 5));
        assert!(s.has_thermal_fault());
        assert!(s.persistence_dirty);
        assert_eq!(s.last_update_ms, 5);
        assert!(s.set_temperature(1, 25.0, 6));
        assert!(!s.has_thermal_fault());
        assert!(s.set_temperature(4, -30.0, 7));
        assert!(s.thermal.under_temp_flag);
        assert_eq!(s.thermal.obc_temp_c, 70.0);
        let before = s;
        assert!(!s.set_temperature(5, 0.0, 8));
        assert_eq!(s, before);
    }

    #[test]
    fn beacon_interval_range() {
        let mut s = SystemState::defaults(0);
        assert!(!s.set_beacon_interval(9, 1));
        assert!(!s.set_beacon_interval(121, 1));
        assert!(!s.persistence_dirty);
        assert!(s.set_beacon_interval(10, 1));
        assert!(s.set_beacon_interval(120, 1));
        assert_eq!(s.comm.beacon_interval_s, 120);
    }

    #[test]
    fn soc_clamped() {
        assert_eq!(clamp_soc(1.5), 1.0);
        assert_eq!(clamp_soc(-0.5), 0.0);
        assert_eq!(clamp_soc(0.75), 0.75);
        assert!(clamp_soc(f64::NAN).is_nan());
    }

    #[test]
    fn uptime_counts_whole_seconds() {
        let mut u = Uptime::new();
        let mut m = Mission::default();
        assert!(!u.update(10_000, &mut m)); // first call starts the clock
        assert!(!u.update(10_999, &mut m));
        assert!(u.update(12_500, &mut m));
        assert_eq!((m.uptime_s, m.total_uptime_s, m.mission_time_ms), (2, 2, 2500));
        assert!(!u.update(13_400, &mut m)); // 0.9 s since 12.5 s
        assert!(u.update(13_500, &mut m));
        assert_eq!(m.uptime_s, 3);
    }
}
