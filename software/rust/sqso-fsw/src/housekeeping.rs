//! The downlinked housekeeping frame (`HK1`, LV-27): the state the ground
//! needs to prove commissioning, which the beacon does not carry.
//!
//! An AX.25 UI frame from the station's call sign to `CQ`, like the beacon,
//! whose information field starts `HK1 ` so beacon decoders can tell it
//! apart; the beacon's own format is unchanged (`docs/BEACON_SPEC.md`). The
//! field is plain ASCII `KEY=value` pairs separated by spaces, integers
//! only, `-` for a value not measured. Every value comes from a measurement
//! or the flight software's state, never from the AI payload.
//!
//! | Key | Value | Unit |
//! |-----|-------|------|
//! | `N` | frame sequence (wraps at 65536) | — |
//! | `UP` | seconds since boot | s |
//! | `BOOT` | resets since the mission data began | — |
//! | `RST` | last reset cause (`HalResetCause_t`) | — |
//! | `MODE` | operating state (`state_machine.h`) | — |
//! | `HOLD` | commissioning hold 1/0 | — |
//! | `XP` | transponder enabled now 1/0 | — |
//! | `TX` | transmission permitted 1/0 | — |
//! | `SOC` | state of charge | ‰ |
//! | `VB` | battery voltage | mV |
//! | `IB` | battery current, positive charging | mA |
//! | `PA` | array power | mW |
//! | `TB`, `TO`, `TP`, `TJ` | battery, OBC, PA, Jetson heat-spreader temperature | 0.1 °C |
//! | `DEP` | deployment state (`deployment.h`)/AntS elements deployed (bit n−1 for element n, 15 all; `-` no answer)/wing sense | —/—/1 |
//! | `RFS` | RF-silence state (`rf_silence.h`)/seconds counted | —/s |
//! | `CT` | loss-of-contact timeout | days |
//! | `CS` | time since the last authenticated command | h |
//! | `FC` | fault-log entries | — |
//! | `FL` | newest fault: type/severity (`fault_mgmt.h`) | — |
//! | `W` | body rate x/y/z | 0.01 °/s |
//! | `AM` | ADCS mode (`adcs_control.h`) | — |
//! | `MR` | magnetometer samples rejected (coils on) | — |
//! | `GF` | GNSS: age of the latest fix/satellites used | s/— |
//! | `GS` | GNSS sessions run/with a fix | — |
//! | `PS` | passivation (`passivation.h`: 0 idle, 1 armed, 2 firing, 3 passivated, 4 reporting)/EPS `EOL_STATE` (bit 0 armed, 1 fired, 2 fuse intact; `-` no answer)/fire attempts (RID REG-12)/the fire's reasons (`sqso_fsw::passivation`: bit 0 waiting for sunlight, 1 waiting for the EPS board at 0 °C, 2 fired cold after the 6 h bound, 3 board temperature unreadable; RID SMA-B20) | —/—/—/— |
//! | `RX` | receiver recovery cycles since boot/known failed 1/0 (`receiver.h`) | —/— |
//! | `CN` | authenticated commands counted as contact since boot (with `CS`, the loss-of-contact count, GND-03) | — |
//! | `ST` | stored housekeeping: next five-minute/sixty-second record number (`hk_store.h`, DUMP_TLM) | — |
//! | `FR` | FRAM state (0 working, 1 failed, 2 degraded mode; RID SMA-B10)/failed one-minute checks since boot | —/— |
//! | `AD` | AntS temperature gate (decision C-18): attempts deferred/the AntS's last temperature (`-` none valid) | —/°C |
//!
//! Each frame's information field is at most [`MAX_LEN`] bytes, about 1.9 s
//! on the air at 1200 bps. The text is cut into as many frames as it needs
//! ([`Parts`]): at the spaces between fields, never inside one, each frame
//! after the first starting with the first's tag and sequence (`HK1 N=7 `),
//! so a decoder that reads `KEY=value` pairs merges them by `N`. (Until
//! CDR round 3 the text had to fit one frame; past 250 bytes nothing was
//! formatted and an empty frame went out: SIL finding SIL-40.)
//!
//! **`HK2`** follows each `HK1`, [`AFTER_BEACON_MS`] later: the counters the
//! mission success criteria need (SQSO-MSC-001 2.1 §7, RID SMA-15) and the
//! OBC board's supervision (round 3), the same format ([`Housekeeping2`]):
//!
//! | Key | Value | Unit |
//! |-----|-------|------|
//! | `N` | the HK1 sequence it follows | — |
//! | `CA` | telecommands executed/refused/failed authentication since boot | — |
//! | `WD` | resets by the watchdog (kept across resets) | — |
//! | `XE` | transponder enables since boot/repeat keyed (XPDR_KEY)/exciter keyed (PTT_EXCITER) | —/s/s |
//! | `IA` | array current (INA228 at 0x40, else ISOLAR) | mA |
//! | `HL` | hard-limit violations since boot (Jetson on outside ACTIVE, sunlight or SOC ≥ 55 %, or past its burst limit) | — |
//! | `JB` | Jetson bursts since boot/last burst's energy/its lines/bursts ended by the energy gate | —/J/—/— |
//! | `JP` | Jetson switch-offs clean/cut after the wait/power-good faults/readback faults | — |
//! | `EOL` | EPS `EOL_STATE` (bit 0 armed, 1 fired, 2 fuse intact; `-` no answer)/fire attempts | —/— |
//! | `RT` | mission clock (RV-3028) UNIX time/the last reset's stamp | s/s |
//! | `CK` | the MCU clock against the mission clock's 1 Hz | ppm |
//! | `TR` | latch-up trips of the gyro, BMI088, sun-sensor and magnetometer rails since boot | — |
//! | `GY` | gyro in use (0 M-G370, 1 BMI088, 2 none)/magnetometer samples gated (Jetson, exciter) | —/— |
//! | `MF` | magnetorquer bridge faults/off-timer expiries seen | — |
//! | `PE` | SRAM2 parity on 1/0 (`-` not known: the host build) | — |
//! | `SEP` | separation latched on this boot 1/0/stowed readings since (RID SMA-B02) | —/— |
//! | `PT` | transmitter temperature sensor faulted now 1/0/faults since boot (RID SMA-B19) | —/— |
//! | `SV` | EPS_SUPV (`-` no answer)/heartbeats refused (RID SMA-B09) | —/— |
//! | `CI` | the EPS controller's CHG_INHIBIT (`-` no answer) | — |
//! | `TS` | the TNC's last STATUS (`-` none yet) | — |
//! | `RL` | RX_FAULT self-test passed/STATUS loop query unanswered for a day (RID SMA-B11) | —/— |
//! | `SL` | selector table in use 1/0, then decisions/abstentions of H1 template, H2 transponder bucket, H3 power mode since boot (`sqso_fsw::selector`, RID SMA-15) | —/—/—/—/—/—/— |
//! | `GE` | ADCS geometry preset in use (ADCS_SET_GEOMETRY: 0 both wings, 1 W1 only, 2 W2 only, 3 none; decision C-21) | — |
//! | `SN` | sun vector in the body frame x/y/z (`-` not visible) | 0.001 |
//!
//! @requirement SRS-F126 The housekeeping frame shall carry the loss-of-contact count and the commands counted as contact
//! @requirement SRS-F085 The software shall downlink a housekeeping frame every fifth beacon and on command, carrying temperatures, battery current and voltage, array power, deployment, RF-silence, boot and reset, fault-log head, body rates, contact timeout and GNSS fix age

use core::fmt::Write;

/// The information field's first bytes
pub const PREFIX: &str = "HK1";
/// The longest text of a housekeeping frame before it is cut into frames,
/// bytes (every field at its widest: 341 for HK1, 573 for HK2)
pub const TEXT_MAX: usize = 640;
/// Longest information field, bytes
pub const MAX_LEN: usize = 250;
/// One frame every this many beacons (and on GET_HOUSEKEEPING)
pub const EVERY_BEACONS: u32 = 5;
/// Gap after the beacon before the frame, ms (the downlink sends one frame
/// at a time)
pub const AFTER_BEACON_MS: u64 = 5_000;

/// The values (`None`: not measured)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Housekeeping {
    /// Frame sequence
    pub seq: u16,
    /// Seconds since boot
    pub uptime_s: u64,
    /// Resets
    pub boots: u32,
    /// Last reset cause
    pub reset_cause: u32,
    /// Operating state
    pub mode: u32,
    /// Commissioning hold
    pub hold: bool,
    /// Transponder enabled now
    pub xpdr: bool,
    /// Transmission permitted
    pub tx_permitted: bool,
    /// State of charge, 0–1
    pub soc: Option<f64>,
    /// Battery voltage, V
    pub vbatt_v: Option<f64>,
    /// Battery current, A (positive charging)
    pub ibatt_a: Option<f64>,
    /// Array power, W
    pub array_w: Option<f64>,
    /// Battery temperature, °C
    pub tbatt_c: Option<f64>,
    /// OBC temperature, °C
    pub tobc_c: Option<f64>,
    /// PA temperature, °C
    pub tpa_c: Option<f64>,
    /// Jetson heat-spreader temperature, °C
    pub tjet_c: Option<f64>,
    /// Deployment state, the AntS's deployed-element mask (bit n−1 for
    /// element n; `None` when it does not answer), wing sense
    pub deploy: (u32, Option<u8>, bool),
    /// RF-silence state and seconds counted
    pub rf_silence: (u32, u32),
    /// Loss-of-contact timeout, days
    pub contact_timeout_d: u16,
    /// Seconds since the last authenticated command
    pub since_contact_s: u32,
    /// Fault-log entries
    pub fault_count: u32,
    /// Newest fault: type and severity (`None`: empty log)
    pub last_fault: Option<(u8, u8)>,
    /// Body rate, 0.01 °/s
    pub rate_cdps: [i16; 3],
    /// ADCS mode
    pub adcs_mode: u8,
    /// Magnetometer samples rejected
    pub mag_rejected: u32,
    /// GNSS: age of the latest fix, s, and satellites (`None`: no fix)
    pub gnss_fix: Option<(u64, u8)>,
    /// GNSS sessions run and with a fix
    pub gnss_sessions: (u32, u32),
    /// Passivation phase (`PassivationPhase_t`)
    pub passivation: u32,
    /// The EPS controller's `EOL_STATE` (`None`: no answer) and the fire
    /// attempts (RID REG-12: the final report carries them)
    pub eol: (Option<u8>, u8),
    /// The fire's reasons (`Passivation::waiting`, RID SMA-B20)
    pub eol_wait: u8,
    /// Receiver recovery cycles since boot, and known failed
    pub rx: (u32, bool),
    /// Authenticated commands counted as contact since boot
    pub contacts: u32,
    /// Stored housekeeping: next five-minute and sixty-second record numbers
    pub stored: [u32; 2],
    /// The FRAM (`FR`, RID SMA-B10): 0 working, 1 failed, 2 the degraded
    /// mode; and the failed one-minute checks since boot
    pub fram: (u8, u32),
    /// The AntS temperature gate (`AD`): deferrals, and the last
    /// temperature, °C (`None`: none valid)
    pub ants_gate: (u8, Option<f64>),
}

/// The second frame's values (`None`: not measured)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Housekeeping2 {
    /// The HK1 sequence it follows
    pub seq: u16,
    /// Telecommands executed, refused, failed authentication
    pub commands: (u32, u32, u32),
    /// Watchdog resets
    pub wdt_resets: u32,
    /// Transponder enables, repeat keyed s, exciter keyed s
    pub xpdr: (u32, u32, u32),
    /// Array current, A
    pub array_a: Option<f64>,
    /// Hard-limit violations
    pub hard_limit: u32,
    /// Jetson bursts, last energy J, last lines, gate ends
    pub bursts: (u32, f64, usize, u32),
    /// Jetson switch-offs clean, timed out, power-good faults, readback faults
    pub jetson: (u32, u32, u32, u32),
    /// EPS EOL_STATE, fire attempts
    pub eol: (Option<u8>, u8),
    /// Mission clock now and the last reset's stamp, UNIX s
    pub rtc: (Option<u32>, Option<u32>),
    /// MCU clock against the mission clock, ppm
    pub clock_ppm: Option<f64>,
    /// Rail trips: gyro, BMI088, sun sensors, magnetometer
    pub trips: [u32; crate::adcs_sensors::RAILS],
    /// Gyro in use (0, 1, 2), samples gated
    pub gyro: (u8, u32),
    /// Bridge faults and off-timer expiries
    pub mtq_faults: (u32, u32),
    /// SRAM2 parity on
    pub parity: Option<bool>,
    /// Separation latched on this boot, and the stowed readings since
    /// (`SEP`, RID SMA-B02)
    pub separation: (bool, u32),
    /// The transmitter temperature sensor faulted now, and its faults since
    /// boot (`PT`, RID SMA-B19)
    pub pa_sensor: (bool, u32),
    /// EPS_SUPV (`None`: no answer) and the heartbeats the controller
    /// refused (`SV`, RID SMA-B09)
    pub supv: (Option<u8>, u32),
    /// The EPS controller's `CHG_INHIBIT` (`CI`: bit 0 cold, 1 hot, 2 cell
    /// OV, 3 BMS fault, 4 secondary protector blown, 5 recharge lockout, 6
    /// NTC1 implausible; `None`: no answer; SQSO-ICD-001 1.6 §4.6)
    pub chg_inhibit: Option<u8>,
    /// The TNC's last STATUS answer (`TS`: version / criteria / floor /
    /// calibrated floor / carrier count / frames / uptime; `-` none yet)
    pub tnc: Option<sqso_protocol::kiss::TncStatus>,
    /// `RL`: RX_FAULT passed its power-on self-test / the STATUS loop query
    /// unanswered for a day (RID SMA-B11)
    pub rx_lines: (bool, bool),
    /// `SL`: the selector's table state and per-head counts (RID SMA-15)
    pub selector: crate::selector::Stats,
    /// `GE`: the ADCS geometry preset in use (decision C-21)
    pub geometry: u8,
    /// `SN`: the sun vector in the body frame, unit (`None`: not visible)
    pub sun: Option<[f64; 3]>,
}

struct Out<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Write for Out<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(core::fmt::Error)?;
        let dst = self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        dst.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// An optional scaled value: the rounded integer, or `-` (no `Display`
/// impl: a formatter callback would be an indirect call the stack analysis
/// must resolve, `scripts/stack_usage.py`)
fn opt(v: Option<f64>, scale: f64) -> Option<i32> {
    #[allow(clippy::cast_possible_truncation)] // clamped to i32
    v.filter(|v| v.is_finite()).map(|v| crate::fmath::round(v * scale).clamp(-2.0e9, 2.0e9) as i32)
}

fn put(w: &mut Out<'_>, key: &str, v: Option<i32>) -> core::fmt::Result {
    match v {
        Some(v) => write!(w, "{key}={v} "),
        None => write!(w, "{key}=- "),
    }
}

fn flag(b: bool) -> u8 {
    u8::from(b)
}

/// The fields added at CDR: passivation, receiver, contacts, stored records
fn tail(w: &mut Out<'_>, hk: &Housekeeping) -> core::fmt::Result {
    let [summary, detail] = hk.stored;
    write!(w, "PS={}/", hk.passivation)?;
    opt_u(w, hk.eol.0.map(u32::from))?;
    write!(
        w,
        "/{}/{} RX={}/{} CN={} ST={summary}/{detail} FR={}/{}",
        hk.eol.1,
        hk.eol_wait,
        hk.rx.0,
        flag(hk.rx.1),
        hk.contacts,
        hk.fram.0,
        hk.fram.1
    )?;
    write!(w, " AD={}/", hk.ants_gate.0)?;
    match hk.ants_gate.1.filter(|t| t.is_finite()) {
        #[allow(clippy::cast_possible_truncation)] // a valid reading, -60 to +125 °C
        Some(t) => write!(w, "{}", crate::fmath::round(t) as i32),
        None => w.write_str("-"),
    }
}

/// Write the text into `out`; its length (0 if it does not fit, which
/// [`TEXT_MAX`] bytes always do). [`Parts`] cuts it into frames
pub fn format(hk: &Housekeeping, out: &mut [u8]) -> usize {
    let room = out.len().min(TEXT_MAX);
    let mut w = Out { buf: out.get_mut(..room).unwrap_or(&mut []), len: 0 };
    let ok = write!(
        w,
        "{PREFIX} N={} UP={} BOOT={} RST={} MODE={} HOLD={} XP={} TX={} ",
        hk.seq,
        hk.uptime_s,
        hk.boots,
        hk.reset_cause,
        hk.mode,
        flag(hk.hold),
        flag(hk.xpdr),
        flag(hk.tx_permitted),
    )
    .and_then(|()| put(&mut w, "SOC", opt(hk.soc, 1000.0)))
    .and_then(|()| put(&mut w, "VB", opt(hk.vbatt_v, 1000.0)))
    .and_then(|()| put(&mut w, "IB", opt(hk.ibatt_a, 1000.0)))
    .and_then(|()| put(&mut w, "PA", opt(hk.array_w, 1000.0)))
    .and_then(|()| put(&mut w, "TB", opt(hk.tbatt_c, 10.0)))
    .and_then(|()| put(&mut w, "TO", opt(hk.tobc_c, 10.0)))
    .and_then(|()| put(&mut w, "TP", opt(hk.tpa_c, 10.0)))
    .and_then(|()| put(&mut w, "TJ", opt(hk.tjet_c, 10.0)))
    .and_then(|()| write!(w, "DEP={}/", hk.deploy.0))
    .and_then(|()| match hk.deploy.1 {
        Some(m) => write!(w, "{m}/"),
        None => w.write_str("-/"),
    })
    .and_then(|()| {
        write!(
            w,
            "{} RFS={}/{} CT={} CS={} FC={} ",
            flag(hk.deploy.2),
            hk.rf_silence.0,
            hk.rf_silence.1,
            hk.contact_timeout_d,
            hk.since_contact_s / 3600,
            hk.fault_count,
        )
    })
    .and_then(|()| match hk.last_fault {
        Some((t, s)) => write!(w, "FL={t}/{s} "),
        None => w.write_str("FL=- "),
    })
    .and_then(|()| {
        write!(
            w,
            "W={}/{}/{} AM={} MR={} ",
            hk.rate_cdps[0], hk.rate_cdps[1], hk.rate_cdps[2], hk.adcs_mode, hk.mag_rejected
        )
    })
    .and_then(|()| match hk.gnss_fix {
        Some((age, sats)) => write!(w, "GF={age}/{sats} "),
        None => w.write_str("GF=- "),
    })
    .and_then(|()| write!(w, "GS={}/{} ", hk.gnss_sessions.0, hk.gnss_sessions.1))
    .and_then(|()| tail(&mut w, hk))
    .is_ok();
    if ok { w.len } else { 0 }
}

fn opt_u(w: &mut Out<'_>, v: Option<u32>) -> core::fmt::Result {
    match v {
        Some(v) => write!(w, "{v}"),
        None => w.write_str("-"),
    }
}

/// The second frame's text (`HK2`) into `out`; its length (0 if it does
/// not fit, which [`TEXT_MAX`] bytes always do). [`Parts`] cuts it
pub fn format2(hk: &Housekeeping2, out: &mut [u8]) -> usize {
    let room = out.len().min(TEXT_MAX);
    let mut w = Out { buf: out.get_mut(..room).unwrap_or(&mut []), len: 0 };
    let (bursts, energy, lines, gate) = hk.bursts;
    let ok = write!(
        w,
        "HK2 N={} CA={}/{}/{} WD={} XE={}/{}/{} ",
        hk.seq, hk.commands.0, hk.commands.1, hk.commands.2, hk.wdt_resets, hk.xpdr.0, hk.xpdr.1, hk.xpdr.2
    )
    .and_then(|()| put(&mut w, "IA", opt(hk.array_a, 1000.0)))
    .and_then(|()| {
        write!(w, "HL={} JB={bursts}/", hk.hard_limit)?;
        opt_u(&mut w, opt(Some(energy), 1.0).and_then(|v| u32::try_from(v).ok()))?;
        write!(w, "/{lines}/{gate} JP={}/{}/{}/{} EOL=", hk.jetson.0, hk.jetson.1, hk.jetson.2, hk.jetson.3)
    })
    .and_then(|()| opt_u(&mut w, hk.eol.0.map(u32::from)))
    .and_then(|()| write!(w, "/{} RT=", hk.eol.1))
    .and_then(|()| opt_u(&mut w, hk.rtc.0))
    .and_then(|()| w.write_str("/"))
    .and_then(|()| opt_u(&mut w, hk.rtc.1))
    .and_then(|()| w.write_str(" "))
    .and_then(|()| put(&mut w, "CK", opt(hk.clock_ppm, 1.0)))
    .and_then(|()| {
        write!(
            w,
            "TR={}/{}/{}/{} GY={}/{} MF={}/{} PE=",
            hk.trips[0], hk.trips[1], hk.trips[2], hk.trips[3], hk.gyro.0, hk.gyro.1, hk.mtq_faults.0, hk.mtq_faults.1
        )
    })
    .and_then(|()| opt_u(&mut w, hk.parity.map(u32::from)))
    .and_then(|()| write!(w, " SEP={}/{}", u8::from(hk.separation.0), hk.separation.1))
    .and_then(|()| write!(w, " PT={}/{} SV=", u8::from(hk.pa_sensor.0), hk.pa_sensor.1))
    .and_then(|()| opt_u(&mut w, hk.supv.0.map(u32::from)))
    .and_then(|()| write!(w, "/{} CI=", hk.supv.1))
    .and_then(|()| opt_u(&mut w, hk.chg_inhibit.map(u32::from)))
    .and_then(|()| w.write_str(" TS="))
    .and_then(|()| match hk.tnc {
        Some(t) => write!(
            w,
            "{}/{}/{}/{}/{}/{}/{}",
            t.version, t.criteria, t.floor_dbm, t.floor_cal_dbm, t.carrier_count, t.frames, t.uptime_s
        ),
        None => w.write_str("-"),
    })
    .and_then(|()| write!(w, " RL={}/{}", u8::from(hk.rx_lines.0), u8::from(hk.rx_lines.1)))
    .and_then(|()| {
        let s = &hk.selector;
        let ([d1, d2, d3], [a1, a2, a3]) = (s.decisions, s.abstentions);
        write!(w, " SL={}/{d1}/{a1}/{d2}/{a2}/{d3}/{a3} GE={} SN=", u8::from(s.enabled), hk.geometry)
    })
    .and_then(|()| match hk.sun {
        Some(v) => {
            let [x, y, z] = v.map(|c| opt(Some(c.clamp(-1.0, 1.0)), 1000.0).unwrap_or(0));
            write!(w, "{x}/{y}/{z}")
        }
        None => w.write_str("-"),
    })
    .is_ok();
    if ok { w.len } else { 0 }
}

/// A housekeeping text and the frames it is cut into: each at most
/// [`MAX_LEN`] bytes, cut at the spaces between fields, every frame after
/// the first starting with the text's head (its tag and sequence,
/// `HK1 N=7 `)
#[derive(Clone, Copy, Debug)]
pub struct Parts {
    text: [u8; TEXT_MAX],
    len: usize,
    /// The head's length, its trailing space included
    head: usize,
    /// Where the next frame's fields start
    off: usize,
}

impl Parts {
    /// Nothing to send
    pub const EMPTY: Parts = Parts { text: [0; TEXT_MAX], len: 0, head: 0, off: 0 };

    /// The buffer to format a text into ([`format`], [`format2`]); then
    /// [`Parts::load`] with its length
    pub fn buffer(&mut self) -> &mut [u8; TEXT_MAX] {
        &mut self.text
    }

    /// The text in the buffer is `len` bytes (0: nothing to send): its frames
    /// start from the first
    pub fn load(&mut self, len: usize) {
        self.len = len.min(TEXT_MAX);
        let text = self.text.get(..self.len).unwrap_or(&[]);
        // The head: up to and with the second space ("HK1 N=7 ")
        let mut spaces = text.iter().enumerate().filter(|(_, b)| **b == b' ').map(|(i, _)| i);
        self.head = spaces.nth(1).map_or(0, |i| i + 1);
        self.off = 0;
    }

    /// Whether a frame is left to send
    pub const fn pending(&self) -> bool {
        self.off < self.len
    }

    /// The next frame's information field into `out`, its length and where
    /// the following one starts (0: none left); nothing is consumed until
    /// [`Parts::commit`]
    pub fn peek(&self, out: &mut [u8; MAX_LEN]) -> (usize, usize) {
        if !self.pending() {
            return (0, self.off);
        }
        let text = self.text.get(..self.len).unwrap_or(&[]);
        let (head, start) = if self.off == 0 { (&[][..], 0) } else { (text.get(..self.head).unwrap_or(&[]), self.off) };
        let room = MAX_LEN - head.len();
        let rest = text.get(start..).unwrap_or(&[]);
        let cut = if rest.len() <= room {
            rest.len()
        } else {
            // The last space that leaves a whole field in the frame (past the
            // head in the first); a field longer than a frame (none is) is
            // cut at the frame's end
            let first_field = if self.off == 0 { self.head } else { 1 };
            rest.get(..=room)
                .and_then(|r| r.iter().rposition(|b| *b == b' '))
                .filter(|&i| i >= first_field)
                .unwrap_or(room)
        };
        let body = rest.get(..cut).unwrap_or(&[]);
        let n = head.len() + body.len();
        for (dst, src) in out.iter_mut().zip(head.iter().chain(body.iter())) {
            *dst = *src;
        }
        // Past the separating space
        let next = start + cut + usize::from(rest.get(cut) == Some(&b' '));
        (n, next.min(self.len))
    }

    /// The frame from [`Parts::peek`] went out: `next` is where it said the
    /// following one starts
    pub fn commit(&mut self, next: usize) {
        self.off = next.clamp(self.off, self.len);
    }
}

/// When the next frame is due: every [`EVERY_BEACONS`] beacons, or at once
/// on request
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cadence {
    beacons: u32,
    requested: bool,
    due_ms: Option<u64>,
    /// Sequence of the next frame
    pub seq: u16,
}

impl Cadence {
    /// Nothing sent yet
    pub const fn new() -> Cadence {
        Cadence { beacons: 0, requested: false, due_ms: None, seq: 0 }
    }

    /// GET_HOUSEKEEPING: a frame after the next beacon gap
    pub fn request(&mut self, now_ms: u64) {
        self.requested = true;
        self.due_ms = Some(now_ms.saturating_add(AFTER_BEACON_MS));
    }

    /// A beacon was sent at `now_ms`
    pub fn beacon_sent(&mut self, now_ms: u64) {
        self.beacons = self.beacons.saturating_add(1);
        if self.beacons >= EVERY_BEACONS && self.due_ms.is_none() {
            self.due_ms = Some(now_ms.saturating_add(AFTER_BEACON_MS));
        }
    }

    /// Whether a frame is due at `now_ms`
    pub fn due(&self, now_ms: u64) -> bool {
        self.due_ms.is_some_and(|t| now_ms >= t)
    }

    /// The frame went out (or could not be sent and is dropped)
    pub fn sent(&mut self) {
        self.beacons = 0;
        self.requested = false;
        self.due_ms = None;
        self.seq = self.seq.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    fn text(hk: &Housekeeping) -> std::string::String {
        let mut b = [0u8; TEXT_MAX];
        let n = format(hk, &mut b);
        std::string::String::from_utf8(b[..n].to_vec()).unwrap()
    }

    /// The frames a text is cut into
    fn frames(text: &str) -> std::vec::Vec<std::string::String> {
        let mut p = Parts::EMPTY;
        p.buffer()[..text.len()].copy_from_slice(text.as_bytes());
        p.load(text.len());
        let mut out = std::vec::Vec::new();
        while p.pending() {
            let mut f = [0u8; MAX_LEN];
            let (n, next) = p.peek(&mut f);
            assert!(n > 0 && n <= MAX_LEN, "{n}");
            assert_eq!(p.peek(&mut [0u8; MAX_LEN]), (n, next), "peek consumes nothing");
            p.commit(next);
            out.push(std::string::String::from_utf8(f[..n].to_vec()).unwrap());
        }
        out
    }

    /// The fields of the frames, the repeated heads dropped: the text's own
    fn merged(frames: &[std::string::String]) -> std::string::String {
        let head: std::vec::Vec<&str> = frames[0].splitn(3, ' ').take(2).collect();
        let mut words: std::vec::Vec<&str> = std::vec::Vec::new();
        for (k, f) in frames.iter().enumerate() {
            let mut w: std::vec::Vec<&str> = f.split(' ').collect();
            if k > 0 {
                assert_eq!(&w[..2], &head[..], "each frame starts with the head: {f}");
                w.drain(..2);
            }
            words.extend(w);
        }
        words.join(" ")
    }

    #[test]
    fn every_field_measured_or_dashed() {
        let hk = Housekeeping {
            seq: 7,
            uptime_s: 3600,
            boots: 3,
            reset_cause: 2,
            mode: 3,
            hold: true,
            tx_permitted: true,
            soc: Some(0.8734),
            vbatt_v: Some(15.812),
            ibatt_a: Some(-0.12),
            array_w: Some(2.41),
            tbatt_c: Some(12.3),
            tobc_c: Some(21.5),
            tpa_c: Some(30.1),
            deploy: (8, Some(15), true),
            rf_silence: (3, 2700),
            contact_timeout_d: 30,
            since_contact_s: 7300,
            fault_count: 4,
            last_fault: Some((17, 2)),
            rate_cdps: [12, -5, 600],
            adcs_mode: 2,
            gnss_fix: Some((1200, 7)),
            gnss_sessions: (5, 4),
            passivation: 1,
            eol: (Some(4), 0),
            eol_wait: 3,
            rx: (2, false),
            contacts: 3,
            stored: [12, 60],
            ants_gate: (2, Some(-18.6)),
            ..Housekeeping::default()
        };
        let t = text(&hk);
        assert_eq!(
            t,
            "HK1 N=7 UP=3600 BOOT=3 RST=2 MODE=3 HOLD=1 XP=0 TX=1 SOC=873 VB=15812 IB=-120 PA=2410 TB=123 TO=215 \
             TP=301 TJ=- DEP=8/15/1 RFS=3/2700 CT=30 CS=2 FC=4 FL=17/2 W=12/-5/600 AM=2 MR=0 GF=1200/7 GS=5/4 \
             PS=1/4/0/3 RX=2/0 CN=3 ST=12/60 FR=0/0 AD=2/-19"
        );
        assert!(t.len() <= MAX_LEN);
        assert!(t.is_ascii());
        let empty = text(&Housekeeping { soc: Some(f64::NAN), ..Housekeeping::default() });
        assert!(empty.contains("SOC=- "), "{empty}");
        assert!(empty.contains("FL=- "), "{empty}");
        assert!(empty.contains("GF=- "), "{empty}");
        assert!(empty.contains(" DEP=0/-/0 "), "the AntS not answering: {empty}");
        assert!(empty.contains(" PS=0/-/0/0 "), "the EPS not answering: {empty}");
        assert!(empty.ends_with(" AD=0/-"), "no AntS temperature: {empty}");
    }

    #[test]
    fn worst_case_fits_and_a_small_buffer_gives_nothing() {
        let hk = Housekeeping {
            seq: u16::MAX,
            uptime_s: 99_999_999,
            boots: 99_999,
            reset_cause: 5,
            mode: 5,
            soc: Some(1.0),
            vbatt_v: Some(16.8),
            ibatt_a: Some(-3.5),
            array_w: Some(9.999),
            tbatt_c: Some(-40.0),
            tobc_c: Some(-40.0),
            tpa_c: Some(-40.0),
            tjet_c: Some(-40.0),
            deploy: (10, Some(15), true),
            rf_silence: (3, 2700),
            contact_timeout_d: 90,
            since_contact_s: 90 * 86_400,
            fault_count: 100,
            last_fault: Some((22, 4)),
            rate_cdps: [-32768, -32768, -32768],
            adcs_mode: 4,
            mag_rejected: 4_000_000_000,
            gnss_fix: Some((99_999_999, 24)),
            gnss_sessions: (99_999, 99_999),
            passivation: 4,
            eol: (Some(255), 3),
            eol_wait: 15,
            rx: (99_999, true),
            contacts: 99_999,
            stored: [u32::MAX, u32::MAX],
            fram: (2, u32::MAX),
            ants_gate: (u8::MAX, Some(-59.6)),
            ..Housekeeping::default()
        };
        // Every field at its widest: formatted in full (341 bytes), cut into
        // two frames of whole fields, each with the head, none longer than
        // a frame (SIL-40: this used to pass on an empty text)
        let t = text(&hk);
        assert_eq!(t.len(), 341, "{t}");
        assert!(t.ends_with(" AD=255/-60"), "{t}");
        let f = frames(&t);
        assert_eq!(f.len(), 2, "{f:?}");
        assert!(f[1].starts_with("HK1 N=65535 "), "{f:?}");
        assert_eq!(merged(&f), t);
        let mut small = [0u8; 20];
        assert_eq!(format(&hk, &mut small), 0);
    }

    /// The cutting: a text that fits is one frame; a longer one is cut at a
    /// space with the head repeated; a field longer than a frame is cut at
    /// the frame's end; nothing is consumed without a commit
    #[test]
    fn texts_are_cut_into_frames_of_whole_fields() {
        let short = "HK1 N=3 A=1 B=2";
        assert_eq!(frames(short), [short]);
        let long = std::format!(
            "HK1 N=3 {}",
            (0..45).map(|k| std::format!("K{k}=1234")).collect::<std::vec::Vec<_>>().join(" ")
        );
        let f = frames(&long);
        assert!(f.len() >= 2 && f.iter().all(|x| x.len() <= MAX_LEN), "{f:?}");
        assert_eq!(merged(&f), long);
        // Exactly a frame: one frame; one byte more: two
        let mut exact = std::string::String::from("HK1 N=3 ");
        while exact.len() < MAX_LEN {
            exact.push('X');
        }
        assert_eq!(frames(&exact).len(), 1);
        exact.push_str(" Y=1");
        assert_eq!(frames(&exact).len(), 2);
        // A field longer than the frame's room is cut, and the text still ends
        let giant = std::format!("HK1 N=3 G={}", "9".repeat(400));
        let f = frames(&giant);
        assert_eq!(f[0].len(), MAX_LEN);
        assert!(f.concat().len() > giant.len());
        // Nothing to send
        let mut p = Parts::EMPTY;
        p.load(0);
        assert!(!p.pending());
        assert_eq!(p.peek(&mut [0u8; MAX_LEN]), (0, 0));
        p.commit(7);
        assert!(!p.pending());
    }

    fn text2(hk: &Housekeeping2) -> std::string::String {
        let mut b = [0u8; TEXT_MAX];
        let n = format2(hk, &mut b);
        std::string::String::from_utf8(b[..n].to_vec()).unwrap()
    }

    #[test]
    fn the_second_frame() {
        let hk = Housekeeping2 {
            seq: 7,
            commands: (12, 3, 1),
            wdt_resets: 2,
            xpdr: (4, 3600, 4100),
            array_a: Some(0.412),
            bursts: (6, 1210.4, 10, 0),
            jetson: (5, 1, 0, 0),
            eol: (Some(0x04), 0),
            rtc: (Some(1_900_000_000), None),
            clock_ppm: Some(-12.4),
            trips: [1, 0, 0, 0],
            gyro: (0, 37),
            parity: Some(true),
            separation: (true, 1),
            pa_sensor: (false, 2),
            supv: (Some(0x21), 0),
            chg_inhibit: Some(0x40),
            rx_lines: (true, false),
            ..Housekeeping2::default()
        };
        assert_eq!(
            text2(&hk),
            "HK2 N=7 CA=12/3/1 WD=2 XE=4/3600/4100 IA=412 HL=0 JB=6/1210/10/0 JP=5/1/0/0 EOL=4/0 \
             RT=1900000000/- CK=-12 TR=1/0/0/0 GY=0/37 MF=0/0 PE=1 SEP=1/1 PT=0/2 SV=33/0 CI=64 TS=- RL=1/0 \
             SL=0/0/0/0/0/0/0 GE=0 SN=-"
        );
        // The selector's counts, the preset and the sun (RID SMA-15, C-21)
        let mut sel = crate::selector::Stats { enabled: true, ..crate::selector::Stats::new() };
        sel.record(crate::selector::Head::Template, true);
        sel.record(crate::selector::Head::PowerMode, false);
        let t = text2(&Housekeeping2 { selector: sel, geometry: 1, sun: Some([-0.1383, 0.9894, 0.0443]), ..hk });
        assert!(t.ends_with(" SL=1/1/0/0/0/0/1 GE=1 SN=-138/989/44"), "{t}");
    }

    /// Every field at its widest
    fn worst2() -> Housekeeping2 {
        Housekeeping2 {
            seq: u16::MAX,
            commands: (u32::MAX, u32::MAX, u32::MAX),
            wdt_resets: u32::MAX,
            xpdr: (u32::MAX, u32::MAX, u32::MAX),
            array_a: Some(-2.6),
            hard_limit: u32::MAX,
            bursts: (u32::MAX, 4.0e9, usize::MAX, u32::MAX),
            jetson: (u32::MAX, u32::MAX, u32::MAX, u32::MAX),
            eol: (Some(255), 255),
            rtc: (Some(u32::MAX), Some(u32::MAX)),
            clock_ppm: Some(-1.0e6),
            trips: [u32::MAX; 4],
            gyro: (2, u32::MAX),
            mtq_faults: (u32::MAX, u32::MAX),
            parity: Some(false),
            separation: (true, u32::MAX),
            pa_sensor: (true, u32::MAX),
            supv: (Some(255), u32::MAX),
            chg_inhibit: Some(255),
            tnc: Some(sqso_protocol::kiss::TncStatus {
                version: 255,
                criteria: 63,
                floor_dbm: -128,
                floor_cal_dbm: -128,
                carrier_count: u16::MAX,
                frames: u16::MAX,
                uptime_s: u32::MAX,
            }),
            rx_lines: (true, true),
            selector: crate::selector::Stats {
                enabled: true,
                decisions: [u32::MAX; crate::selector::HEADS],
                abstentions: [u32::MAX; crate::selector::HEADS],
            },
            geometry: u8::MAX,
            sun: Some([-1.0; 3]),
        }
    }

    #[test]
    fn the_second_frame_at_its_widest_and_empty() {
        let worst = worst2();
        let t = text2(&worst);
        assert_eq!(t.len(), 573, "{t}");
        assert!(t.len() <= TEXT_MAX);
        assert!(t.ends_with(" SN=-1000/-1000/-1000"), "formatted in full: {t}");
        let f = frames(&t);
        assert!(f.len() >= 2 && f[1].starts_with("HK2 N=65535 "), "{f:?}");
        assert_eq!(merged(&f), t);
        let none = text2(&Housekeeping2::default());
        assert!(none.contains("IA=- "), "{none}");
        assert!(none.contains("EOL=-/0"), "{none}");
        assert!(none.ends_with("PE=- SEP=0/0 PT=0/0 SV=-/0 CI=- TS=- RL=0/0 SL=0/0/0/0/0/0/0 GE=0 SN=-"), "{none}");
        assert_eq!(format2(&worst, &mut [0u8; 20]), 0);
    }

    #[test]
    fn cadence_every_fifth_beacon_or_on_request() {
        let mut c = Cadence::new();
        for k in 1..EVERY_BEACONS {
            c.beacon_sent(u64::from(k) * 60_000);
            assert!(!c.due(u64::from(k) * 60_000 + 10_000));
        }
        c.beacon_sent(300_000);
        assert!(!c.due(300_000 + AFTER_BEACON_MS - 1));
        // A sixth beacon before the frame goes does not push it back
        c.beacon_sent(300_000 + 1000);
        assert!(c.due(300_000 + AFTER_BEACON_MS));
        c.sent();
        assert_eq!(c.seq, 1);
        assert!(!c.due(400_000));
        c.request(500_000);
        assert!(c.due(500_000 + AFTER_BEACON_MS));
        c.sent();
        assert!(!c.due(u64::MAX));
        assert_eq!(Cadence::default(), Cadence { seq: 0, ..Cadence::new() });
    }
}
