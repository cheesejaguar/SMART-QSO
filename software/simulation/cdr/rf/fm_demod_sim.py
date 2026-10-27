#!/usr/bin/env python3
"""Baseband simulation of the SMART-QSO FM links (CDR, SQSO-LBA-001 v3.0 §3).

What it gives the link budget:

1. The required C/N0 of the 1200 bps AFSK (Bell 202) beacon and telecommand
   through an FM receiver: bit and frame error rate against C/N0, with and
   without an uncorrected Doppler offset (3.5 kHz at 145.9 MHz, 10 kHz at
   435.5 MHz), for a receiver IF of +/-7.5 kHz (a 25 kHz-channel FM filter).
2. The SINAD of an FM voice channel against C/N0 (TIA-603-style test: 1 kHz
   tone at 3 kHz deviation, 750 us de-emphasis, 300-3000 Hz audio), to give
   the SINAD a user hears from the transponder.

Model (numpy only; complex baseband at FS):
  audio -> FM modulator (peak deviation D) -> carrier offset (Doppler)
  -> complex AWGN at a given C/N0 -> IF low-pass (windowed-sinc FIR, +/-BIF/2)
  -> limiter-discriminator (arg z[n] z*[n-1]) -> audio filters -> demodulator.
AFSK demodulator: mark/space correlators over each bit with ideal bit timing
(a genie clock: optimistic by a fraction of a dB against a real DPLL), tone
decision, then NRZI decoding (a tone error flips two decoded bits). Frame
success is taken as (1-BER)^bits (independent errors; FM clicks make errors
bursty, which if anything helps the frame rate at a given BER).

This is an analysis model; the flight value is set by the packet-error-rate
test in SQSO-LBA-001 §8 (verification item VI-RF-03).

Usage: python3 fm_demod_sim.py [--quick] [--out results.json]
"""
from __future__ import annotations

import argparse
import json
import math

import numpy as np

FS = 96_000.0          # complex sample rate, Hz
BAUD = 1200.0
SPB = int(FS / BAUD)   # 80 samples a bit


def fir_lowpass(cut_hz: float, ntaps: int = 257) -> np.ndarray:
    n = np.arange(ntaps) - (ntaps - 1) / 2
    h = np.sinc(2 * cut_hz / FS * n) * np.hamming(ntaps)
    return h / h.sum()


def fir_bandpass(lo: float, hi: float, ntaps: int = 1025) -> np.ndarray:
    return fir_lowpass(hi, ntaps) - fir_lowpass(lo, ntaps)


def one_pole_lowpass(x: np.ndarray, tau_s: float) -> np.ndarray:
    """750 us de-emphasis (first-order RC), by recursive filtering in blocks."""
    a = math.exp(-1.0 / (FS * tau_s))
    y = np.empty_like(x)
    acc = 0.0
    # chunked python loop is fast enough for ~1e6 samples
    for i in range(0, len(x), 4096):
        seg = x[i:i + 4096]
        out = np.empty_like(seg)
        for k, v in enumerate(seg):
            acc = a * acc + (1 - a) * v
            out[k] = acc
        y[i:i + 4096] = out
    return y


def fm_modulate(audio: np.ndarray, dev_hz: float, offset_hz: float) -> np.ndarray:
    """audio normalized to peak 1"""
    phase = 2 * np.pi * np.cumsum(dev_hz * audio + offset_hz) / FS
    return np.exp(1j * phase)


def add_noise(sig: np.ndarray, cn0_dbhz: float, rng: np.random.Generator) -> np.ndarray:
    n0 = 10 ** (-cn0_dbhz / 10)        # carrier power 1
    sigma = math.sqrt(n0 * FS / 2)     # per real dimension, complex white over FS
    return sig + sigma * (rng.standard_normal(len(sig)) + 1j * rng.standard_normal(len(sig)))


def discriminator(z: np.ndarray) -> np.ndarray:
    d = np.angle(z[1:] * np.conj(z[:-1])) * FS / (2 * np.pi)
    return np.concatenate([[0.0], d])


def afsk_audio(bits_tone: np.ndarray) -> np.ndarray:
    """Continuous-phase AFSK: tone 1 = mark 1200 Hz, tone 0 = space 2200 Hz"""
    f = np.where(np.repeat(bits_tone, SPB) == 1, 1200.0, 2200.0)
    return np.sin(2 * np.pi * np.cumsum(f) / FS)


def afsk_ber(cn0: float, nbits: int, dev_hz: float, bif_hz: float, offset_hz: float,
             rng: np.random.Generator) -> float:
    tones = rng.integers(0, 2, nbits)
    audio = afsk_audio(tones)
    z = add_noise(fm_modulate(audio, dev_hz, offset_hz), cn0, rng)
    z = np.convolve(z, fir_lowpass(bif_hz / 2), mode="same")
    d = discriminator(z)
    d = np.convolve(d, fir_bandpass(400.0, 3200.0), mode="same")   # AC coupling + audio band
    t = np.arange(len(d)) / FS
    nb = nbits
    seg = d[: nb * SPB].reshape(nb, SPB)
    tt = t[: nb * SPB].reshape(nb, SPB)
    def energy(f):
        c = (seg * np.cos(2 * np.pi * f * tt)).sum(1)
        s = (seg * np.sin(2 * np.pi * f * tt)).sum(1)
        return c * c + s * s
    dec = (energy(1200.0) > energy(2200.0)).astype(int)
    sl = slice(8, nb - 8)                       # filter edges
    tone_err = np.mean(dec[sl] != tones[sl])
    return float(min(0.5, 2 * tone_err))        # NRZI: each tone error flips two data bits


def tone_sinad(cn0: float, dev_hz: float, bif_hz: float, offset_hz: float, dur_s: float,
               rng: np.random.Generator) -> float:
    n = int(FS * dur_s)
    t = np.arange(n) / FS
    audio = np.sin(2 * np.pi * 1000.0 * t)
    z = add_noise(fm_modulate(audio, dev_hz, offset_hz), cn0, rng)
    z = np.convolve(z, fir_lowpass(bif_hz / 2), mode="same")
    d = discriminator(z)
    d = np.convolve(d, fir_bandpass(300.0, 3000.0), mode="same")
    d = one_pole_lowpass(d, 750e-6)
    d = d[int(0.05 * FS): -int(0.05 * FS)]
    tt = t[int(0.05 * FS): -int(0.05 * FS)]
    # remove the 1 kHz tone by least squares (a notch), what is left is noise + distortion
    a = np.column_stack([np.sin(2 * np.pi * 1000 * tt), np.cos(2 * np.pi * 1000 * tt)])
    coef, *_ = np.linalg.lstsq(a, d, rcond=None)
    resid = d - a @ coef
    return float(10 * math.log10(np.mean(d * d) / np.mean(resid * resid)))


def interp_cross(xs, ys, target, decreasing=True):
    """x where y crosses target (linear in log for BER)"""
    for i in range(len(xs) - 1):
        y0, y1 = ys[i], ys[i + 1]
        if (y0 - target) * (y1 - target) <= 0 and y0 != y1:
            return xs[i] + (target - y0) * (xs[i + 1] - xs[i]) / (y1 - y0)
    return None


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--out", default=None)
    args = ap.parse_args()
    rng = np.random.default_rng(20261027)
    nbits = 20_000 if args.quick else 120_000
    frame_bits = 1850   # 200-byte information field + 16 B header + FCS, with ~3 % bit stuffing

    res: dict = {"fs_hz": FS, "nbits_per_point": nbits, "frame_bits": frame_bits, "afsk": {}, "sinad": {}}
    cn0s = [44, 46, 47, 48, 49, 50, 51, 52, 53, 54, 56, 58]
    cases = {
        "beacon_bif15k_no_doppler": (3000.0, 15000.0, 0.0),
        "beacon_bif15k_doppler3.5k": (3000.0, 15000.0, 3500.0),
        "beacon_bif12k_doppler3.5k": (3000.0, 12000.0, 3500.0),
        "telecommand_bif15k_doppler10k": (3000.0, 15000.0, 10000.0),
        "telecommand_bif15k_residual1k": (3000.0, 15000.0, 1000.0),
        "telecommand_bif25k_doppler10k": (3000.0, 25000.0, 10000.0),
        "telecommand_bif25k_no_doppler": (3000.0, 25000.0, 0.0),
    }
    for name, (dev, bif, off) in cases.items():
        rows = []
        for c in cn0s:
            ber = afsk_ber(c, nbits, dev, bif, off, rng)
            fsp = (1 - ber) ** frame_bits
            rows.append({"cn0_dbhz": c, "ber": ber, "frame_success": fsp})
            print(f"{name:34s} C/N0 {c:5.1f}  BER {ber:.2e}  frame {fsp:.3f}", flush=True)
        xs = [r["cn0_dbhz"] for r in rows]
        lb = [math.log10(max(r["ber"], 1e-7)) for r in rows]
        fs = [r["frame_success"] for r in rows]
        res["afsk"][name] = {
            "dev_hz": dev, "bif_hz": bif, "offset_hz": off, "points": rows,
            "cn0_ber_1e-4": interp_cross(xs, lb, -4.0),
            "cn0_frame_90": interp_cross(xs, fs, 0.90),
            "cn0_frame_50": interp_cross(xs, fs, 0.50),
        }
    for name, (dev, bif, off) in {
        "voice_bif15k_no_offset": (3000.0, 15000.0, 0.0),
        "voice_bif15k_offset3.5k": (3000.0, 15000.0, 3500.0),
    }.items():
        rows = []
        for c in [44, 46, 48, 50, 52, 54, 56, 58, 60, 64]:
            s = tone_sinad(c, dev, bif, off, 0.5 if args.quick else 1.5, rng)
            rows.append({"cn0_dbhz": c, "sinad_db": s})
            print(f"{name:34s} C/N0 {c:5.1f}  SINAD {s:5.1f} dB", flush=True)
        xs = [r["cn0_dbhz"] for r in rows]
        ys = [r["sinad_db"] for r in rows]
        res["sinad"][name] = {"points": rows,
                              "cn0_sinad_12": interp_cross(xs, ys, 12.0),
                              "cn0_sinad_20": interp_cross(xs, ys, 20.0)}
    js = json.dumps(res, indent=1)
    if args.out:
        open(args.out, "w").write(js)
    for k, v in res["afsk"].items():
        print(k, "C/N0 for BER 1e-4:", v["cn0_ber_1e-4"], " 90 % frames:", v["cn0_frame_90"], " 50 %:", v["cn0_frame_50"])
    for k, v in res["sinad"].items():
        print(k, "C/N0 for 12 dB SINAD:", v["cn0_sinad_12"], " 20 dB:", v["cn0_sinad_20"])


if __name__ == "__main__":
    main()
