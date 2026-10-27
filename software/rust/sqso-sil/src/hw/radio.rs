//! The downlink: the KISS TNC and the VHF transmitter with its PA, coupler
//! and detectors (SQSO-HW-DIAG-001 B4/S4; `hardware/rf/TRANSMITTER_DESIGN.md`).
//!
//! Frames the OBC hands to the TNC queue there and go on the air one after
//! another: keying delay, the frame at 1200 bps with bit stuffing, the
//! tail. RF comes out only while the PA rail is switched on (`RF_TX_EN`)
//! and the exciter powered (`RADIO_EN`); the power-select line picks
//! 0.5 W or 1 W. The forward and reflected detectors and the PA thermistor
//! feed the OBC's ADC, which the downlink module watches.
//!
//! The FM transponder shares the transmitter (RF_PLAN: the beacon is
//! time-shared with the transponder downlink): while its enable holds and
//! a CTCSS-gated carrier is on the uplink, it keys the PA at the selected
//! level and repeats the audio; a TNC frame takes the transmitter first.
//! The enable is the heartbeat detector's ([`heartbeat_enable`]): the
//! board's retriggerable monostable on `XPDR_EN` holds it for
//! [`EN_HOLD_US`] after the last edge of the OBC's heartbeat, so a stopped
//! heartbeat, or a level, disables the transponder within 50 ms
//! (docs/hardware/BUILD_VS_BUY.md §3.2).

use std::collections::VecDeque;

/// Bit rate, bit/s
pub const BIT_RATE: f64 = 1200.0;
/// TNC keying delay and tail, s (`kiss::DOWNLINK_PARAMS`: 300 ms and 10 ms)
pub const TXDELAY_S: f64 = 0.30;
/// TNC tail, s
pub const TXTAIL_S: f64 = 0.01;
/// Mean bit-stuffing overhead on AX.25 text frames
pub const STUFFING: f64 = 1.02;
/// DC input of the PA chain while keyed at 0.5 W out, W (BASELINE §5: 0.5 W RF ≈ 1.5 W DC)
pub const DC_W_LOW: f64 = 1.5;
/// DC input while keyed at 1 W out, W (PA efficiency about 37 %)
pub const DC_W_HIGH: f64 = 2.7;
/// PA quiescent draw while its rail is on and it is not keyed, W
pub const PA_IDLE_W: f64 = 0.04;
/// Receiver, TNC and exciter standby on the RADIO rail, W (BASELINE §5: 0.15 W)
pub const RX_W: f64 = 0.15;
/// Antenna match when deployed: |Γ|² for VSWR 1.3
pub const GAMMA2_DEPLOYED: f64 = 0.017;
/// Antenna match when stowed (element folded against the structure): |Γ|² for VSWR about 8
pub const GAMMA2_STOWED: f64 = 0.60;

/// The heartbeat detector's hold after the last `XPDR_EN` edge, µs
/// (`sqso_fsw::transponder::EN_HOLD_MS`)
pub const EN_HOLD_US: u64 = sqso_fsw::transponder::EN_HOLD_MS as u64 * 1000;

/// The transponder enable (`EN_OK`) the heartbeat detector gives, from the
/// OBC's heartbeat (`Fsw::xpdr_heartbeat()`: its time now and when the wave
/// stops or stopped, µs). The wave's last edge is at its end, or now while
/// it runs; the enable holds until [`EN_HOLD_US`] after it
pub fn heartbeat_enable(heartbeat: Option<(u64, u64)>) -> bool {
    heartbeat.is_some_and(|(now, end)| now < end.saturating_add(EN_HOLD_US))
}

#[derive(Clone, Copy, Debug)]
struct OnAir {
    left_s: f64,
    len: usize,
}

/// Counters of the downlink hardware
#[derive(Clone, Copy, Debug, Default)]
pub struct RadioStats {
    /// Frames received from the OBC
    pub frames: u64,
    /// Frames radiated (RF came out for the whole frame)
    pub radiated: u64,
    /// Frames keyed with no RF out (PA rail off)
    pub silent: u64,
    /// Total keyed time, s
    pub keyed_s: f64,
    /// Total radiating time, s
    pub radiating_s: f64,
    /// Longest single key-down, s
    pub longest_key_s: f64,
    /// Time the transponder keyed the PA, s
    pub xpdr_keyed_s: f64,
    /// Time the transponder radiated, s
    pub xpdr_radiating_s: f64,
}

/// The TNC and transmitter
#[derive(Debug, Default)]
pub struct Radio {
    queue: VecDeque<usize>,
    on_air: Option<OnAir>,
    key_s: f64,
    frame_had_rf: bool,
    /// Forward power now, W
    pub fwd_w: f64,
    /// Reflected power now, W
    pub rev_w: f64,
    /// DC power drawn by the radio rails now, W (RADIO and PA)
    pub dc_w: f64,
    /// Heat in the PA now, W
    pub pa_heat_w: f64,
    /// The transponder has a carrier to repeat now (`XPDR_EN` high and a
    /// CTCSS-valid carrier on the uplink); set by the caller
    pub xpdr: bool,
    /// The transponder keyed the PA in the last step
    pub xpdr_keyed: bool,
    /// Counters
    pub stats: RadioStats,
}

/// A frame's time on the air, s
pub fn airtime_s(len: usize) -> f64 {
    TXDELAY_S + TXTAIL_S + (len as f64 + 4.0) * 8.0 * STUFFING / BIT_RATE
}

impl Radio {
    /// A frame handed to the TNC (addresses through information)
    pub fn send(&mut self, len: usize) {
        self.stats.frames += 1;
        self.queue.push_back(len);
    }

    /// Whether the TNC is keying the transmitter now
    pub fn keyed(&self) -> bool {
        self.on_air.is_some()
    }

    /// Whether anything is queued or on the air
    pub fn busy(&self) -> bool {
        self.on_air.is_some() || !self.queue.is_empty()
    }

    /// Advance `dt_s` with the rails as the pins drive them
    pub fn step(&mut self, dt_s: f64, radio_on: bool, pa_on: bool, high_power: bool, antenna_deployed: bool) {
        if self.on_air.is_none()
            && radio_on
            && let Some(len) = self.queue.pop_front()
        {
            self.on_air = Some(OnAir { left_s: airtime_s(len), len });
            self.key_s = 0.0;
            self.frame_had_rf = pa_on;
        }
        if !radio_on {
            self.queue.clear();
            self.on_air = None;
        }
        let keyed = self.on_air.is_some();
        self.xpdr_keyed = !keyed && self.xpdr && radio_on;
        let radiating = (keyed || self.xpdr_keyed) && pa_on && radio_on;
        if self.xpdr_keyed {
            self.stats.xpdr_keyed_s += dt_s;
            if radiating {
                self.stats.xpdr_radiating_s += dt_s;
            }
        }
        let rf_w = if high_power { 1.0 } else { 0.5 };
        let gamma2 = if antenna_deployed { GAMMA2_DEPLOYED } else { GAMMA2_STOWED };
        self.fwd_w = if radiating { rf_w } else { 0.0 };
        self.rev_w = self.fwd_w * gamma2;
        let keyed_dc = if high_power { DC_W_HIGH } else { DC_W_LOW };
        self.dc_w = if radio_on { RX_W } else { 0.0 }
            + if radiating {
                keyed_dc
            } else if pa_on {
                PA_IDLE_W
            } else {
                0.0
            };
        // Reflected power is dissipated in the PA too
        self.pa_heat_w = if radiating {
            keyed_dc - self.fwd_w + self.rev_w
        } else if pa_on {
            PA_IDLE_W
        } else {
            0.0
        };
        if keyed {
            self.stats.keyed_s += dt_s;
            self.key_s += dt_s;
            if radiating {
                self.stats.radiating_s += dt_s;
            } else {
                self.frame_had_rf = false;
            }
        }
        if let Some(mut f) = self.on_air.take() {
            f.left_s -= dt_s;
            if f.left_s > 0.0 {
                self.on_air = Some(f);
            } else {
                let _ = f.len;
                self.stats.longest_key_s = self.stats.longest_key_s.max(self.key_s);
                if self.frame_had_rf {
                    self.stats.radiated += 1;
                } else {
                    self.stats.silent += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_beacon_frame_is_on_the_air_about_a_second() {
        let t = airtime_s(100);
        assert!(t > 0.95 && t < 1.05, "{t}");
    }

    #[test]
    fn radiates_only_with_the_pa_rail_on() {
        let mut r = Radio::default();
        r.send(60);
        r.step(0.1, true, false, false, true);
        assert!(r.keyed());
        assert_eq!(r.fwd_w, 0.0);
        for _ in 0..20 {
            r.step(0.1, true, false, false, true);
        }
        assert_eq!(r.stats.silent, 1);
        r.send(60);
        for _ in 0..20 {
            r.step(0.1, true, true, true, true);
        }
        assert_eq!(r.stats.radiated, 1);
        assert!(r.stats.radiating_s > 0.7);
    }

    #[test]
    fn the_transponder_keys_the_pa_and_yields_to_a_frame() {
        let mut r = Radio { xpdr: true, ..Radio::default() };
        r.step(1.0, true, true, false, true);
        assert!(r.xpdr_keyed && r.fwd_w == 0.5 && r.dc_w > DC_W_LOW);
        r.send(60);
        r.step(0.1, true, true, false, true);
        assert!(!r.xpdr_keyed && r.keyed());
        assert!((r.stats.xpdr_radiating_s - 1.0).abs() < 1e-9);
        let mut off = Radio { xpdr: true, ..Radio::default() };
        off.step(1.0, false, true, false, true);
        assert!(!off.xpdr_keyed && off.fwd_w == 0.0);
    }

    #[test]
    fn a_stopped_heartbeat_disables_the_transponder_within_50_ms() {
        assert!(!heartbeat_enable(None), "no heartbeat (a level, or the OBC off)");
        // The wave runs until 1.000 s: enabled while it runs and for the
        // monostable's hold after it, off from 50 ms after its last edge
        let end = 1_000_000;
        for now in (0..end).step_by(5_000) {
            assert!(heartbeat_enable(Some((now, end))), "{now}");
        }
        assert!(heartbeat_enable(Some((end + 49_999, end))));
        assert!(!heartbeat_enable(Some((end + 50_000, end))));
        assert!(!heartbeat_enable(Some((end + 10_000_000, end))));
    }

    #[test]
    fn stowed_antenna_reflects() {
        let mut r = Radio::default();
        r.send(60);
        r.step(0.1, true, true, false, false);
        assert!(r.rev_w / r.fwd_w > 0.5);
    }
}
