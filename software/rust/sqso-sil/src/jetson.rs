//! The Jetson Orin Nano payload, as the OBC sees it: a power load on the
//! 5 V payload rail and a peer on the serial line (SQSO-ICD-004,
//! `docs/interfaces/ICD_OBC_JETSON.md`).
//!
//! The line is a pseudo-terminal: the flight software opens its slave side
//! as `$UART_DEVICE` through its own serial driver (`uart_comm`), and this
//! model reads and writes the master side, so every byte crosses the
//! flight code's real receive path.
//!
//! Behaviour (ICD §5.2 and BASELINE §6):
//! - its carrier's power-good (JET_PG, the carrier's 3.3 V) rises
//!   [`PG_RISE_S`] after the rail (SQSO-JET-BRD-001 §7.3);
//! - boots in 20–30 s after its rail is switched on (silent meanwhile);
//! - answers each `TELEMETRY` with `STATUS` and each `BEACON_REQ` with a
//!   `BEACON_RSP` after an inference burst of a few seconds;
//! - on `SHUTDOWN` runs `poweroff` for [`POWEROFF_S`]; the carrier then
//!   latches POWER_EN low and JET_PG falls, the module drawing only the
//!   carrier's housekeeping until the OBC removes the rail (§7.4);
//! - draws, at the carrier input (SQSO-JET-BRD-001 §13, the module capped
//!   at 6.5 W): [`BOOT_W`] booting (the carrier document's 30 s × 4.5 W
//!   allocation) and, once running, the capped burst power, 7.19–7.48 W
//!   (drawn once per power-on in that range): the module is held at its cap
//!   by the generation load for the whole burst, not only while a line is
//!   generated (RID JETB-11);
//! - proposes plain-language beacon lines; a small fraction would fail the
//!   onboard content filter (a stray number, a URL), as a real generator's
//!   would.

use std::ffi::CStr;
use std::os::fd::RawFd;

use sqso_protocol::jetson::{self, FromObc, LineRx, Status};

/// Boot time, s (ICD §5.2: typical 20, maximum 30)
pub const BOOT_S: (f64, f64) = (20.0, 30.0);
/// Carrier-input power booting, W (SQSO-JET-BRD-001 §13: 4.5 W)
pub const BOOT_W: f64 = 4.5;
/// Carrier-input power idle (a latch-up's baseline; the module does not
/// idle during a burst), W
pub const IDLE_W: f64 = 3.5;
/// Carrier-input power of the capped burst, W (SQSO-JET-BRD-001 §13: 6.5 W
/// cap, 7.19–7.32 W modelled, 7.48 W with the regulator at its bound)
pub const BURST_W: (f64, f64) = (7.19, 7.48);
/// Input power during inference, W (the top of the capped range)
pub const INFER_W: f64 = 7.48;
/// The carrier's housekeeping after the module's poweroff, W
pub const OFF_LATCHED_W: f64 = 0.05;
/// JET_PG rises this long after the rail, s
pub const PG_RISE_S: f64 = 0.1;
/// `poweroff` takes this long, s
pub const POWEROFF_S: f64 = 3.0;
/// The field the module's input current makes at the magnetometer during a
/// burst, T (SQSO-JET-BRD-001 §19, item JETB-09: about 0.35 µT at 63 mm)
pub const MAG_FIELD_T: f64 = 0.35e-6;
/// Time to generate one beacon line, s (SmolLM2-360M INT4, about 20 tokens)
pub const INFER_S: (f64, f64) = (2.5, 6.0);
/// Fraction of proposed lines the content filter should refuse
pub const BAD_TEXT_FRACTION: f64 = 0.03;

/// Lines the generator proposes (non-numeric, BASELINE §6)
const TEXTS: [&str; 16] = [
    "Greetings from low Earth orbit, have a good contact",
    "The sun is up over the Pacific and the transponder is listening",
    "Clear skies to every station copying this beacon",
    "SMART-QSO says hello to all amateurs on the pass",
    "Quiet bands tonight, perfect for a contact through the bird",
    "Thank you for listening, please share your reports",
    "Orbiting peacefully, power and thermal look healthy",
    "Learning when the operators are busiest, see you next pass",
    "Good morning to the stations under the terminator",
    "Hello from a tiny satellite with a big antenna dream",
    "Keep your contacts short so everyone gets a turn",
    "Listening on the uplink, talking on the downlink",
    "The night side is cold but the battery is warm",
    "Wishing a happy pass to the students tracking us",
    "Seventy three to all, from a small spacecraft",
    "Calm space weather today, enjoy the pass",
];
/// Lines a generator might produce that the filter must refuse
const BAD_TEXTS: [&str; 3] =
    ["Visit www dot example dot com for prizes", "Battery is at 99 percent says the 2nd sensor", "Call 555 0199 now"];

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Off,
    Booting { left_s: f64 },
    Running,
    /// `poweroff` running, s left
    PoweringOff { left_s: f64 },
    /// The carrier latched off after the poweroff (until the rail is cut)
    Latched,
}

/// Counters of the payload's life
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Times the rail was switched on
    pub power_ons: u32,
    /// Boots completed
    pub boots: u32,
    /// Energy drawn at the input, Wh
    pub energy_wh: f64,
    /// Seconds powered
    pub on_s: f64,
    /// Telemetry lines received from the OBC
    pub telemetry_rx: u32,
    /// Beacon requests received
    pub requests: u32,
    /// Beacon lines sent
    pub responses: u32,
    /// Of them, ones meant to fail the filter
    pub bad_responses: u32,
    /// Lines from the OBC that failed the link CRC or format
    pub bad_lines: u32,
    /// `SHUTDOWN` lines received
    pub shutdowns: u32,
    /// Power-offs that cut a running module (no `SHUTDOWN` first)
    pub hard_cuts: u32,
}

/// The Jetson payload model
pub struct Jetson {
    master: RawFd,
    device: String,
    rx: LineRx,
    state: State,
    uptime_s: f64,
    /// A beacon request being answered: (seq, max_len, seconds left)
    pending: Option<(u16, u8, f64)>,
    /// Junction temperature, °C (from the thermal model)
    temp_c: f64,
    power_w: f64,
    /// This power-on's burst power, W
    burst_w: f64,
    /// Time since the rail came on, s (JET_PG)
    rail_s: f64,
    rng: u64,
    /// Counters
    pub stats: Stats,
}

impl Jetson {
    /// Open the pseudo-terminal; the flight software opens [`Jetson::device`]
    pub fn open(seed: u64) -> std::io::Result<Jetson> {
        // SAFETY: plain libc calls on a descriptor this function owns
        unsafe {
            let master = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
            if master < 0 || libc::grantpt(master) != 0 || libc::unlockpt(master) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut name = [0 as libc::c_char; 128];
            if libc::ptsname_r(master, name.as_mut_ptr(), name.len()) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let flags = libc::fcntl(master, libc::F_GETFL);
            let _ = libc::fcntl(master, libc::F_SETFL, flags | libc::O_NONBLOCK);
            let device = CStr::from_ptr(name.as_ptr()).to_string_lossy().into_owned();
            Ok(Jetson {
                master,
                device,
                rx: LineRx::new(),
                state: State::Off,
                uptime_s: 0.0,
                pending: None,
                temp_c: 20.0,
                power_w: 0.0,
                burst_w: BURST_W.0,
                rail_s: 0.0,
                rng: seed | 1,
                stats: Stats::default(),
            })
        }
    }

    /// The slave side's path (for `$UART_DEVICE`)
    pub fn device(&self) -> &str {
        &self.device
    }

    /// Input power now, W
    pub fn power_w(&self) -> f64 {
        self.power_w
    }

    /// Whether it is up and answering
    pub fn running(&self) -> bool {
        self.state == State::Running
    }

    /// The carrier's power-good (JET_PG): the rail on for [`PG_RISE_S`] and
    /// the module not shut down
    pub fn power_good(&self) -> bool {
        !matches!(self.state, State::Off | State::Latched) && self.rail_s >= PG_RISE_S
    }

    fn uniform(&mut self) -> f64 {
        // xorshift64*
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    fn send(&mut self, line: &[u8]) {
        // SAFETY: line is a live slice; the master is open
        let _ = unsafe { libc::write(self.master, line.as_ptr().cast(), line.len()) };
    }

    /// Drain what the OBC wrote (also while off: the line is dead then)
    fn read_obc(&mut self) -> Vec<Vec<u8>> {
        let mut lines = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            // SAFETY: buf is live; the master is open and non-blocking
            let n = unsafe { libc::read(self.master, buf.as_mut_ptr().cast(), buf.len()) };
            let Ok(n) = usize::try_from(n) else { break };
            if n == 0 {
                break;
            }
            for &b in &buf[..n] {
                if let Some(r) = self.rx.feed(b) {
                    match r {
                        Ok(line) => lines.push(line.to_vec()),
                        Err(_) => self.stats.bad_lines += 1,
                    }
                }
            }
        }
        lines
    }

    /// Advance `dt_s`: `powered` is the rail (payload switch and power gate
    /// both on), `junction_c` the module temperature from the thermal model
    pub fn step(&mut self, dt_s: f64, powered: bool, junction_c: f64) {
        self.temp_c = junction_c;
        let lines = self.read_obc();
        if !powered {
            if matches!(self.state, State::Running | State::Booting { .. }) {
                self.stats.hard_cuts += 1;
            }
            self.state = State::Off;
            self.pending = None;
            self.power_w = 0.0;
            self.rail_s = 0.0;
            self.rx.reset();
            return;
        }
        if self.state == State::Off {
            let boot = BOOT_S.0 + (BOOT_S.1 - BOOT_S.0) * self.uniform();
            self.state = State::Booting { left_s: boot };
            self.uptime_s = 0.0;
            self.stats.power_ons += 1;
            self.burst_w = BURST_W.0 + (BURST_W.1 - BURST_W.0) * self.uniform();
        }
        self.rail_s += dt_s;
        self.uptime_s += dt_s;
        self.stats.on_s += dt_s;
        match self.state {
            State::Booting { left_s } => {
                self.power_w = BOOT_W;
                let left = left_s - dt_s;
                if left <= 0.0 {
                    self.state = State::Running;
                    self.stats.boots += 1;
                } else {
                    self.state = State::Booting { left_s: left };
                }
            }
            State::Running => self.run(dt_s, &lines),
            State::PoweringOff { left_s } => {
                self.power_w = BOOT_W;
                let left = left_s - dt_s;
                self.state = if left <= 0.0 { State::Latched } else { State::PoweringOff { left_s: left } };
            }
            State::Latched => self.power_w = OFF_LATCHED_W,
            State::Off => {}
        }
        self.stats.energy_wh += self.power_w * dt_s / 3600.0;
    }

    fn run(&mut self, dt_s: f64, lines: &[Vec<u8>]) {
        for line in lines {
            match jetson::parse_from_obc(line) {
                Ok(FromObc::Telemetry(_)) => {
                    self.stats.telemetry_rx += 1;
                    self.send_status();
                }
                Ok(FromObc::BeaconReq { seq, max_len }) => {
                    self.stats.requests += 1;
                    let t = INFER_S.0 + (INFER_S.1 - INFER_S.0) * self.uniform();
                    self.pending = Some((seq, max_len, t));
                }
                Ok(FromObc::StatusReq) => self.send_status(),
                Ok(FromObc::Shutdown(_)) => {
                    self.stats.shutdowns += 1;
                    self.pending = None;
                    self.state = State::PoweringOff { left_s: POWEROFF_S };
                    self.power_w = BOOT_W;
                    return;
                }
                Err(_) => self.stats.bad_lines += 1,
            }
        }
        // Held at its 6.5 W cap by the burst's load
        self.power_w = self.burst_w;
        if let Some((seq, max_len, left)) = self.pending {
            let left = left - dt_s;
            if left <= 0.0 {
                self.pending = None;
                self.answer(seq, max_len);
            } else {
                self.pending = Some((seq, max_len, left));
            }
        }
    }

    fn answer(&mut self, seq: u16, max_len: u8) {
        let bad = self.uniform() < BAD_TEXT_FRACTION;
        let text = if bad {
            BAD_TEXTS[(self.uniform() * BAD_TEXTS.len() as f64) as usize % BAD_TEXTS.len()]
        } else {
            TEXTS[(self.uniform() * TEXTS.len() as f64) as usize % TEXTS.len()]
        };
        let text = &text[..text.len().min(usize::from(max_len))];
        let mut out = [0u8; jetson::LINE_MAX];
        if let Ok(n) = jetson::beacon_rsp(seq, text, &mut out) {
            self.send(&out[..n]);
            self.stats.responses += 1;
            if bad {
                self.stats.bad_responses += 1;
            }
        }
    }

    fn send_status(&mut self) {
        let s = Status {
            health: if self.temp_c > 85.0 { 2 } else { 0 },
            temp_c: self.temp_c.round().clamp(-128.0, 200.0) as i32,
            power_mw: (self.power_w * 1000.0).round().clamp(0.0, 100_000.0) as u32,
            mem_pct: 37,
            uptime_s: self.uptime_s as u32,
        };
        let mut out = [0u8; jetson::LINE_MAX];
        if let Ok(n) = jetson::status(&s, &mut out) {
            self.send(&out[..n]);
        }
    }
}

impl Drop for Jetson {
    fn drop(&mut self) {
        // SAFETY: the master descriptor is this model's, closed once
        let _ = unsafe { libc::close(self.master) };
    }
}
