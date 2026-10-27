//! Magnetorquer drive (SQSO-OBC-BRD-001 B §10): a dipole command becomes,
//! per rod, a direction (the bridge's PH line) and a PWM duty on its EN
//! line, for one burst of [`crate::adcs::MTQ_ON_MS`] timed by the hardware
//! (`hal_mtq.h`).
//!
//! The CubeSpace CR0002 rod gives [`ROD_GAIN`] A·m² per ampere; with the
//! bridge in slow decay the average coil current is D · VM / R, so the
//! duty for a dipole m is D = (|m| / gain) · R / VM, with VM the coil
//! supply (VBAT_MTQ, the pack voltage) and R the rod's resistance. The
//! bridges' current limit (104–126 mA) is the hardware's bound above the
//! 87 mA of the 0.2 A·m² saturation.
//!
//! The polarity state of the three rods (the sign of each rod's latest
//! non-zero command, one bit per axis) selects the magnetometer's bias
//! correction for the rods' remanence (AV-V4; [`crate::adcs::Calibration`]).
//!
//! @requirement SRS-F141 Magnetorquer drive: PH before ARM, PWM duty, hardware-timed burst

use crate::adcs::Vec3;

/// Rod gain, A·m² per A (CubeSpace CR0002)
pub const ROD_GAIN: f64 = 2.3;
/// Rod resistance, Ω (CubeSpace brochure value; the design covers 30–80 Ω)
pub const ROD_OHMS: f64 = 51.0;
/// Coil supply used when the pack voltage is not known, V (4S nominal)
pub const VM_DEFAULT_V: f64 = 14.4;
/// Coil supply range the duty is computed for, V (the pack's)
pub const VM_RANGE_V: (f64, f64) = (11.2, 16.8);
/// The bridges' PWM frequency, Hz (`HAL_MTQ_PWM_HZ`)
pub const PWM_HZ: u32 = 40_000;
/// Duty full scale (`HAL_MTQ_DUTY_FULL`)
pub const DUTY_FULL: u16 = 1000;
/// A coil current above this at the magnetometer sample means the coils
/// were not off, A (IPROPI readback)
pub const OFF_CURRENT_A: f64 = 0.005;

/// One burst's drive
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drive {
    /// PH per axis (true: positive dipole)
    pub positive: [bool; 3],
    /// Duty per axis, ‰
    pub duty: [u16; 3],
}

impl Drive {
    /// Every duty zero: hold MTQ_ARM low, no burst
    pub fn is_zero(&self) -> bool {
        self.duty == [0; 3]
    }
}

/// The drive for `dipole` (A·m²) from a coil supply of `vm_v` volts (NaN
/// or out of range: [`VM_DEFAULT_V`])
pub fn drive(dipole: &Vec3, vm_v: f64) -> Drive {
    let vm = if vm_v >= VM_RANGE_V.0 && vm_v <= VM_RANGE_V.1 { vm_v } else { VM_DEFAULT_V };
    let axis = |m: f64| -> (bool, u16) {
        let d = (m.abs() / ROD_GAIN * ROD_OHMS / vm).clamp(0.0, 1.0);
        let permille = crate::fmath::round(d * f64::from(DUTY_FULL));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0..=1000 (NaN gives 0)
        let duty = permille as u16;
        (m > 0.0, duty)
    };
    let (x, y, z) = (axis(dipole.x), axis(dipole.y), axis(dipole.z));
    Drive { positive: [x.0, y.0, z.0], duty: [x.1, y.1, z.1] }
}

/// The rods' polarity state, 0–7: bit n set when axis n's latest non-zero
/// command was positive (a zero duty keeps that axis's bit)
pub fn polarity(previous: u8, d: &Drive) -> u8 {
    let mut state = previous & 0x07;
    for (n, (&positive, &duty)) in d.positive.iter().zip(d.duty.iter()).enumerate() {
        if duty > 0 {
            if positive {
                state |= 1 << n;
            } else {
                state &= !(1 << n);
            }
        }
    }
    state
}

/// Whether the coil currents read at the magnetometer sample show the
/// coils off (any NaN reading counts as off: no readback)
pub fn coils_off(currents_a: [f64; 3]) -> bool {
    !currents_a.iter().any(|i| i.abs() > OFF_CURRENT_A)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duty_direction_and_saturation() {
        let d = drive(&Vec3::new(0.2, -0.1, 0.0), 14.4);
        // 0.2 / 2.3 A = 87 mA; × 51 Ω / 14.4 V = 0.308
        assert_eq!(d.duty, [308, 154, 0]);
        assert_eq!(d.positive, [true, false, false]);
        assert!(!d.is_zero());
        assert_eq!(drive(&Vec3::new(5.0, 0.0, 0.0), 11.2).duty[0], DUTY_FULL);
        assert_eq!(drive(&Vec3::new(0.2, 0.0, 0.0), f64::NAN).duty[0], 308, "unknown supply: nominal");
        assert_eq!(drive(&Vec3::new(0.2, 0.0, 0.0), 30.0).duty[0], 308);
        assert!(drive(&Vec3::ZERO, 14.4).is_zero());
        assert_eq!(drive(&Vec3::new(f64::NAN, 0.0, 0.0), 14.4).duty[0], 0);
    }

    #[test]
    fn polarity_follows_the_last_nonzero_command() {
        let p = polarity(0, &drive(&Vec3::new(0.1, 0.1, -0.1), 14.4));
        assert_eq!(p, 0b011);
        let p = polarity(p, &drive(&Vec3::new(-0.1, 0.0, 0.1), 14.4));
        assert_eq!(p, 0b110, "y kept, x and z flipped");
        assert_eq!(polarity(0xFF, &Drive::default()), 0b111);
    }

    #[test]
    fn coil_currents_at_the_sample() {
        assert!(coils_off([0.0, 0.004, -0.004]));
        assert!(!coils_off([0.0, 0.0, 0.02]));
        assert!(coils_off([f64::NAN; 3]));
    }
}
