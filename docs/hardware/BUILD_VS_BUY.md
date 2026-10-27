# Build versus Buy: COTS Survey, Gyro, ADCS Gains and Battery

**Document ID**: SQSO-BVB-001
**Version**: F
**Date**: 2026-10-27
**Status**: Draft; vendor data from public pages on 2026-10-26, no quotes yet

This document compares the hardware that does not have to be custom with what CubeSat vendors sell now, and decides
build or buy for each item. It records four decisions the PI delegated: the gyro, the attitude-control gains and spin
rate, the battery size, and the hardware behind the transponder enable and the isolated battery mount. The gains
and the battery are decided by simulation (the ADCS Monte Carlo in §6 and the mission simulation in §7).

`docs/BASELINE_PARAMETERS.md` governs. Where this document changes a baseline value, the baseline carries the change
(Rev D) and this document gives the reason.

Prices are list prices, not quotes. "[unverified]" marks a value seen only in a third-party listing or a search
snippet. cubesatshop.com could not be reached from the research environment.

## 1. Summary

| Item | Decision | Main reason |
|------|----------|-------------|
| 1U structure | **Buy** (ISIS, Pumpkin or Spacemanic; RFQ all three) | 90–120 g against 280 g estimated for a custom frame; CDS rails and switches already qualified. The market is shrinking, so order early |
| Deployable wings, body panels | **Buy** custom panels from a panel vendor (DHV, Spacemanic, ISIS or Pumpkin), AZUR 3G30A cells | Nobody sells a double-sided 1U wing pair. Panel vendors have hinges, staking and cell-laying yield; photodiodes and thermistors come built in |
| EPS | **Build** (keep the 4S charger, BMS and the safety functions), with **ISIS ICEPS2** as the alternate | ICEPS2 Type B is the only 1U-class COTS EPS on a 4S bus. No COTS EPS has the permanent end-of-life charge disconnect (97.207(g)) |
| Battery | **Build: keep 4S1P INR18650-35E (≈ 50 Wh), charge to 4.10 V a cell** (§7) | The pack is oversized for energy (2.5–3×), but the only lighter option worth having is 2S1P, which changes the bus; it saves ≈ 125 g at the cost of re-planning every 14.4 V load. Lowering the charge voltage removes the life penalty of sitting full |
| Isolated battery mount | **Build** (§4) | No COTS pack is isolated; all are coupled to the stack by design |
| OBC | **Build** (STM32L4R5) | No COTS STM32L4 OBC; the Rust/Embassy software and its analyses target the L4R5 |
| Transponder radio | **Build** (the repeat path on the telecommand receiver and the 2 m transmitter) (§3) | No flight-proven U/v FM transponder is for sale; the only U/v-capable catalogue item is unflown |
| Transponder enable (XPDR_EN) | **Build**: hardware gate with a heartbeat enable (§3) | Must fail safe when the OBC hangs; TX-OFF stays the hardware cessation path |
| Antennas | **Buy** ISIS AntS 1U–3U, monopole/hybrid configuration | > 500 units flown since 2010, redundant release, < 90 g |
| Magnetometer | **Build: PNI RM3100 on board A2** at the +Z end frame (OBC board design, SQSO-OBC-BRD-001 §9.4); a bought module (GomSpace M315, AAC MM200 or Spacemanic Lodestone) with I2C at 3.3 V fits the same slot and harness if AV-V4 fails | Single-shot conversion on command in the coils-off window; 15 nT noise; ≈ 3 g with board and harness against 8–12 g |
| Magnetorquers | **Buy** CubeSpace CR0002 rods (> 0.2 A·m², 16.5 g each), or keep the air-core coils | Saves ≈ 80 g; rods add remanence that the magnetometer calibration must handle |
| Gyro | **Buy: Epson M-G370PDT0** primary, **Bosch BMI088** backup on the OBC board (§5) | The Monte Carlo needs ≤ 0.1°/s bias; the M-G370 meets it from the factory and has 20 krad TID data |
| ADCS gains | **B-dot 10⁴; spin 8°/s; spin hold added** (§6) | B-dot at 5·10⁶ left 15 of 40 trials tumbling; without a spin hold 23 of 40 settled with −Y on the Sun; at 2°/s +Y was within 15° of the Sun for only 75 % of sunlit time |

Mass: the buys move the predicted mass by about −200 to −250 g CBE (structure −160 to −190 g, rods −80 g, AntS
+55 g), from 0–4 % margin to roughly 14–18 %. That is an estimate; the MEL has to be re-run with quoted masses.

## 2. Vendor survey

### 2.1 Vendor status, October 2026

The 1U catalogue has shrunk. Several vendors that the repository's older trade studies relied on no longer list 1U
parts.

| Vendor | Status | 1U items still listed |
|--------|--------|-----------------------|
| **Pumpkin** (San Francisco) | Operating (news in April, July and September 2026); focus on 6U SUPERNOVA buses. The store shows the CubeSat Kit chassis walls as "Unavailable" | CubeSat Kit structure parts, AntS-compatible plates, BM 2 battery, custom panels |
| EnduroSat | Operating; **all 1U products delisted** | UHF Transceiver II |
| ISISPACE | Operating; the 1U structure, TRXVU/TRXUV, iMTQ and iOBC pages return 404 | ICEPS2 EPS, 1U solar panels, AntS antenna |
| GomSpace | Operating; 1U–3U lines gone (P31u, ANT430) | M315 magnetometer, FSS sun sensor, AX100 (UHF) |
| AAC Clyde Space | Operating; no 1U structure | STARBUCK-NANO (2S), OPTIMUS packs, PHOTON panels, MM200, SS200 |
| Kongsberg NanoAvionics | Operating; minimum order 3–4 units | EPS 1.0 (2S), MTQ M6P |
| CubeSpace | Operating; no 1U integrated ADCS | CR0002 rods, magnetometers, fine sun sensor |
| Spacemanic | Operating; publishes prices | Structure, Ra panels, Amun PSU (2S), Murgas radio, SAM antenna, Lodestone |
| Ibeos | Acquired by Trident Solutions (2026-03-09) | 14 V EPS and 45 Wh battery, sized for 6U–12U |
| DHV Technology | Operating | 1U panels, PicoEPS, custom deployables |
| AZUR SPACE | Operating (5N Plus); adding cell capacity from H2 2026 | 3G30A (the baselined cell) |
| EXA | Status could not be verified | DMSA/1 1U deployable array (reseller listings only) |
| LETSCOM | Listing only (satsearch, 2026-09) | RepeaterSat VU-X1 FM cross-band repeater, no stated heritage |

### 2.2 Structure and wings

| Product | Mass | Heritage | Price |
|---------|------|----------|-------|
| ISIS 1U structure | 87 g primary, 108 g with secondary | Since 2012 | ≈ €2,150 [unverified]; page removed from isispace.nl |
| Pumpkin CubeSat Kit 1U (walls, base, cover) | not stated | Long | ≈ $2.4–3k for a set [inferred]; store "Unavailable" |
| Spacemanic 1U structure | 115 g | Spacemanic 1U missions | Quote |
| EnduroSat 1U | 120 g | 590+ structures | Delisted |

No vendor sells a double-sided 1U wing pair like `hardware/eps/SOLAR_ARRAY_DESIGN.md` §8. EXA's DMSA/1 is the only
catalogued 1U deployable, and EXA's status is unknown. DHV, Spacemanic, ISIS and Pumpkin build custom deployables
with their own hinges and burn-wire hold-downs. The deployment control (≥ 3 inhibits, ≥ 30 min timer) remains a
project design item whichever is bought.

**Decision: buy the frame, buy the wings and body panels from one panel vendor.** The frame is the largest single
mass lever (about −170 g). Send RFQs to Pumpkin, ISIS and Spacemanic with stock and lead-time questions, and freeze
the choice before the mechanical layout. The RFQ asks for a **recessed −Z face** for the 7 mm AntS within the CDS
standoff (RID STR-11) and the frame's mass, allowables and drawings (STR-20). The Jetson heat path (≥ 1 W/K, BASELINE §6), the GNSS patch and the wing
hinges need custom brackets on whichever frame is chosen.

### 2.3 Solar panels

ISIS (50 g, photodiode and thermistor), Spacemanic Ra (≈ 30 g, from €2,400) and DHV (< 50 g, 2S1P at 4.8 V Vmp) all
sell 1U panels with two 30 % cells. **Decision: buy assembled panels, AZUR 3G30A cells** (or the vendor's
equivalent with `SOLAR_ARRAY_DESIGN.md` re-run). Lead time is 3–7 months, set by cell supply, so order the EM and FM
sets with the wings.

### 2.4 EPS

| Product | Battery | Bus | Mass | Fit |
|---------|---------|-----|------|-----|
| **ISIS ICEPS2 Type B** | 4S1P, 14.4 V, 57.6 Wh, integrated heater | Unregulated battery bus, 4 × 5 V, 4 × 3.3 V, 3 A a channel; 3 MPPT inputs | 310 g | Only current 1U-class COTS EPS on a 4S bus; ISS deployment heritage |
| ISIS ICEPS2 Type A | 2S1P, 7.2 V, 28.8 Wh | as B | 184 g | 2S |
| AAC STARBUCK-NANO + OPTIMUS-30 | 2S Li-polymer, 30 Wh | 8.2 V max; 3.3, 5, 12 V | 92 g + 268 g | 2S |
| Spacemanic Amun Compact | 2S1P, 24 Wh | 3.3, 5 V, VBAT | 185 g | 2S; €3,900 |
| NanoAvionics EPS 1.0 | 2S | 18 outputs | — | 2S; MOQ 3 (≥ €25k) |
| Ibeos 14 V EPS + 45 Wh | 14 V | 3.3, 5, 12 V | not published | 4S-class, sized for 6U–12U; new owner |
| GomSpace P31u | 2S | — | — | Phased out |

**Decision: build the EPS, keep ICEPS2 Type B as the alternate.** ICEPS2 would remove the design and qualification of
the MPPT charger and BMS. Against it: none of these products has the baseline's permanent, non-resettable end-of-life
charge disconnect (97.207(g)). The hardware TX-OFF and the Jetson rail trip should not depend on a vendor's firmware.
Type B has three MPPT inputs for six strings. Its charge window starts at +10 °C at full rate. A custom safety board
would be needed beside it in any case. The flight software already models a 4S charger, BMS and fuel gauge
(`sqso_fsw::eps_control`) with the cell-level protections of BASELINE §5. If an RFQ shows ICEPS2 can be ordered
without its own end-of-life and inhibit logic getting in the way, revisit.

### 2.5 OBC

No COTS OBC uses an STM32L4. The Cortex-M7 boards (EnduroSat, NanoAvionics SatBus 3C2) do not name their MCU, are
sized for 6U and up, and come with minimum orders. Spacemanic's Eddie is an MSP430. **Decision: build.**

### 2.6 Radio and antennas

See §3 for the transponder. For the antennas, **buy the ISIS AntS 1U–3U** in the monopole or hybrid configuration
(two 2 m and two 70 cm tape elements): < 90 g, redundant thermal-knife release, I²C, 2 W RF, flown since 2010 with
more than 500 units. It keeps the baseline's separate monopoles. Expect about +55 g against the MEL's 35 g. It
occupies one ±Z face (98 × 98 × 7 mm), so check it against the deployment switches, the access port and the GNSS
patch. Spacemanic's SAM (€6,200, 55 g) has no monopole option.

### 2.7 ADCS components

An integrated 1U ADCS (Tensor Tech ADCS-MTQ ≈ $20k [unverified], ISIS iMTQ delisted, NanoAvionics M6P MOQ 3) would
duplicate the B-dot and spin laws already in `sqso_fsw::adcs`. **Decision: buy the sensors and rods, build the
control.**

| Function | Buy | Notes |
|----------|-----|-------|
| Magnetometer | Superseded by the OBC board design: PNI RM3100 on board A2 at the +Z end frame, harness W6E (SQSO-OBC-BRD-001 §9.4, chair item C1). Fallback modules: GomSpace M315 (8 g, 15 nT), AAC MM200 (10–12 g, 30 krad) or Spacemanic Lodestone (€800, magnetometer and gyro in < 10 g) | Mount away from the coils and the Jetson |
| Coarse sun sensors | The panel photodiodes (§2.3) | The Monte Carlo used 5° per sensor; 10° made no difference (§6.4) |
| Magnetorquers | CubeSpace CR0002 rods, > 0.2 A·m², 16.5 g each | Or keep the 139 g air-core coils. Rods add remanence and hysteresis that the magnetometer calibration must cover |
| Gyro | §5 | — |

### 2.8 Findings about the older trade studies

These predate the funding audit and should not feed decisions; they are listed for correction.

1. `hardware/structure/STRUCTURE.md` gives 1U frames at 260–300 g; vendor data say 87–120 g. GomSpace and EnduroSat
   no longer sell them.
2. `hardware/eps/README.md` lists products that no vendor catalogue has ("ISIS iEPS-100", "CubeSpace CubePower";
   CubeSpace makes no EPS) and gives every vendor a 4S pack, though most are 2S.
3. `hardware/solar/SOLAR.md` recommends a GomSpace deployable that exists only for 3U and 6U.
4. `docs/hardware/PARTS_LIST.md` 1.0 still has an Apollo4 OBC, a 2-cell battery, an SPV1050 and a Si4463 radio. The
   gyro and battery rows are corrected in 1.1; the rest are not.

Each of these four now carries a superseded notice pointing to the current design (documentation audit, 2026-10-27).

## 3. Transponder and its enable hardware

### 3.1 What exists

No COTS radio is a U/v FM repeater at about 0.5 W RF and 1.5 W DC. The one COTS radio with a documented FM
transponder mode, the ISIS TRXVU, runs V/u (VHF up, UHF down) and draws 4 W in transmit. Its U/v sibling, the TRXUV,
transmits 0.2 W, lists no transponder mode and has been delisted. AAC's PULSAR-VUTRX is V/u and digital. Spacemanic's
Murgas is one band and digital. LETSCOM's VU-X1 is the only U/v-capable FM cross-band repeater listed, with no stated
heritage, an unknown vendor and 5 W DC in transmit.

The amateur heritage transponders are bespoke. AMSAT's Fox-1 series (AO-85, AO-91, AO-92, AO-95) is U/v FM. Its
receiver card carries a 67 Hz CTCSS detector and a 2-minute transmit timer outside the computer, and the repeater
works through computer resets. SO-50 uses two tones and a 10-minute arming window. EASAT-2 (open hardware) uses a
carrier detector and an AC activation signal from its computer.

**Decision: build** the repeat path from the 435 MHz telecommand receiver (always on, 97.207(b)) and the 2 m
transmitter of `hardware/rf/TRANSMITTER_DESIGN.md`. Send RFQs to ISIS (is the TRXUV still orderable, and with a
transponder mode?) and LETSCOM (U/v direction, mass, test data). Neither enters the baseline until confirmed.

### 3.2 XPDR_EN hardware

`XPDR_EN` (TRANSMITTER_DESIGN §10.1) is a level from the OBC: while it is high, the repeat path keys the PA on a
CTCSS-valid uplink. The hardware behind it:

```
435 MHz RX (always on) -- discriminator audio --+-- telecommand demodulator (AFSK 1200)
                                                +-- CTCSS decoder ------------------------- TONE_OK
                                                +-- repeat audio: 300 Hz high-pass (strips the tone),
                                                    3 kHz low-pass, de-/pre-emphasis, limiter -- [mute] --+
OBC XPDR_EN (heartbeat, ≈ 100 Hz) -- retriggerable monostable, ≈ 50 ms ------------ EN_OK               |
Key-down timer, 120 s, reset when the tone drops ---------------------------------- NOT_TOT             |
                                                                                                         |
XPDR_KEY    = EN_OK · TONE_OK · NOT_TOT                                   TNC audio --[2:1 select]--> exciter
PTT_EXCITER = TX_PERMIT_HW · (TNC_PTT + XPDR_KEY · ¬TNC_PTT)                          (select = TNC_PTT)
PA supply   = through the TX_EN hardware switch (TX-OFF)
Readbacks to the OBC: TONE_OK, XPDR_KEY, PTT_EXCITER
```

| Element | Part (example) | Why |
|---------|----------------|-----|
| Heartbeat detector | TI SN74LVC1G123 retriggerable monostable, pull-down on the input | An OBC that hangs with the pin high, resets (high-Z) or browns out drops the transponder within ≈ 50 ms. EASAT-2's AC activation, in the fail-safe direction |
| Key gate | TI SN74LVC1G11 3-input AND | Hardware logic between the enable, the tone and the timer |
| CTCSS decoder and audio conditioning | CML CMX138A (configured at boot by the RF board's TNC microcontroller). The analog FM receivers are discontinued, so the board demodulates in a CC1200 and the repeat audio passes through the TNC microcontroller's firmware (not the OBC flight software); the tone decode, band-limiting, level limit, key gate, 120 s timer and TX permit stay in hardware (SQSO-RF-BRD-001) | Tone decode, tone removal, emphasis and band-limiting in one part, in hardware. A Goertzel detector on the OBC would put flight software in the audio path; rejected |
| Audio select and mute | TI TMUX1219 SPDT | The TNC's frames take the transmitter first, with no software arbitration; receiver noise never rides on the beacon |
| Key-down timer | 120 s hardware timer | Fox-1 practice. A stuck ground carrier is stopped long before the 25 % duty gate would act |
| Repeat-path power (optional) | TI TPS22918 load switch from EN_OK | The tone decoder and filters draw nothing while disabled. It must not switch the 435 MHz receiver |

Safety under TX-OFF: TX_EN, which switches the PA supply, stays the cessation path (97.207(b); BASELINE §7). The
hardware TX permit also feeds the PTT gate, so an exciter keyed into an unpowered PA cannot leak to the antenna. No
state of the transponder logic can power the PA.

Flight software (implemented in the design review, SRS-F080 series): `sqso_fsw::transponder` produces a 100 Hz
heartbeat under a 60 ms software lease renewed every executive pass; the firmware drives PE4 from the TIM7 interrupt,
so the wave stops when the lease runs out, the OBC hangs or it resets. The radio board's ≈ 50 ms retriggerable
monostable treats a level held high as disabled. The mission simulation's radio drops the transponder within 50 ms of
a stopped heartbeat.

## 4. Isolated battery mount

### 4.1 What exists

No COTS pack is thermally isolated. All of them (GomSpace BP4 and BPX, ISIS PBP-4S1P, AAC OPTIMUS, TakeMe2Space
PowerBank-50, Pumpkin BM 2) mount on stack standoffs and stack connectors. TakeMe2Space says its chassis *is* the
thermal interface. All of them put film heaters and sensors at the cells. The only isolation in the literature is
project-level (MinXSS left the copper out of its battery board). **Decision: build the mount** for the in-house pack
(§7).

### 4.2 Conductance

The polymer barely matters: four solid M2.5 polymer spacers conduct about 0.002–0.006 W/K. The fasteners, the
harness and the pack's surface finish set the conductance. Estimates (G = kA/L, contact resistance ignored, so high):

| Path | G (W/K) |
|------|---------|
| 4 × M2.5 G-10CR (or PEEK 450G) shouldered bushings, 8 mm, Ti-6Al-4V screws | 0.013–0.017 |
| Same with stainless screws | 0.030 |
| Flying-lead harness, ≥ 100 mm | 0.008 |
| Radiation, low-ε pack exterior (VDA tape or bare aluminium) | 0.005–0.010 |
| Radiation, black pack in a black enclosure | ≈ 0.07 |
| A PC/104 stack-through connector (104 pins) | up to ≈ 0.2 |
| **Recommended build, to the frame** | **≈ 0.026–0.035** |
| To the OBC stack (no stack connector) | ≈ 0.005 |

### 4.3 Specification (proposed; confirmed by the EM thermal balance test)

- Aluminium 6061-T6 clamp bracket (BATTERY_DESIGN §7.1), clear chromate or bare, ε < 0.1 outside, VDA or aluminised
  polyimide tape facing the enclosure; thermal pad and heater inside.
- 4 × M2.5 shouldered isolator bushings, 6–8 mm effective length, G-10CR or MIL-I-24768/27 (preferred, for creep)
  or PEEK 450G; no metal-to-metal contact between the bracket and the screws or the frame. Outgassing: G-10CR
  0.35 / 0.02 %, PEEK 450G 0.26 / 0.00 % (TML / CVCM, NASA GSFC database), within CDS 2.1.7. Vespel SP-1 fails TML.
- 4 × Ti-6Al-4V M2.5 screws into frame inserts, low preload, staked. Alternative: necked Ti-6Al-4V standoffs
  (≈ 0.023 W/K for four) if polymer creep is unacceptable.
- Flying leads of at least 100 mm to the EPS; no stack connector on the pack.
- Heater ≈ 1 W at 14.4 V (≈ 200 Ω polyimide film, two elements on different cells), the flight software's +5/+9 °C
  set points (raised from +2/+6 °C by the design review, RID THM-02), a series hardware over-temperature cutout at about +40 °C, and a fuse or current limit (MinXSS had a
  heater short in vacuum).
- Target 0.03 W/K to the frame and ≤ 0.01 W/K to the OBC stack. Verify in the EM thermal balance test with the heater
  power stepped (TB-07).
- Structure: the 0.25 kg pack at 30 g gives ≈ 18 N a standoff and ≈ 15 MPa bending in PEEK (125 MPa strength); the
  first lateral mode is near 0.8 kHz, above the 40 Hz requirement. The risk is preload loss by creep and thermal
  cycling, not strength.

### 4.4 Problems found in the repository values

1. THERMAL_DESIGN §4.2 and §7.2 give G-10 as 0.3 W/m·K. NIST G-10CR gives 0.56 (normal) to 0.82 (warp) at 273 K.
2. THERMAL_DESIGN §7.2's "4 × M2.5, 8 mm … total 0.05 W/K" cannot come from the polymer (0.0007 W/K for solid rods
   at 0.3 W/m·K). The drawing must specify the screw material and the surface finish.
3. The mission simulation's battery node has no radiative link; with black surfaces that adds ≈ 0.07 W/K. The
   simulation's 0.05 W/K mount is therefore the value to build to, with a low-ε finish.
4. BATTERY_DESIGN §6.2 ("thermal pad to spacecraft structure") and §7.1 ("M2 fasteners to structure") are stale
   against the isolated mount.

### 4.5 Mission-simulation sensitivity

Runs of `sqso_sil --scenario year` (90 days, LTAN 22:30) with the mount conductance and heater power varied:

| Conductance to the frame | Heater | Lowest pack temperature | Charging refused below 0 °C |
|--------------------------|--------|-------------------------|-----------------------------|
| 0.03 W/K (this design) | 0.5 W | +1.9 °C | never (heater averages 0.09 W) |
| 0.05 W/K (THERMAL_DESIGN, the simulation's value) | 0.5 W | +1.6 °C | never |
| 0.10 W/K (a black pack's radiation added) | 0.5 W | −0.6 °C | 3.8 % of the time: fails SYS-602's check |
| 0.10 W/K | 1.0 W | +1.0 °C | never |

**Decision: a 1 W heater.** 0.5 W is enough at the design conductance, but not if the as-built value comes out at
twice the estimate, which the radiation term alone can do. A thermostatic heater costs energy only while it runs, so
the larger element buys that margin almost for free. The mission simulation keeps the conservative 0.05 W/K and
0.5 W, which pass.

## 5. Gyro

### 5.1 Requirement

The gyro is the rate in the spin law and the only check that the detumble is complete (the magnetometer estimate
cannot see the rate about the field, SIL-05). The Monte Carlo (§6.4) sets the requirement: **bias ≤ 0.1°/s (1σ per
axis) in use**. At 0.2°/s one trial in 40 failed to capture; at 0.3°/s three hit the detumble timeout and the worst
trial's 95th-percentile Sun angle was 153°; at 0.5°/s, 16 of 40. The bias across +Y looks like nutation to the law,
which then drives a real nutation to cancel it. Noise is irrelevant for every candidate. Other requirements: range
≥ ±50°/s, scale factor ≤ 1 %, SPI at 3.3 V, ≤ 60 mW (the ADCS sensor allocation is 0.12 W), ≥ 10 krad TID.

### 5.2 Candidates

| Part | Bias as delivered | Power | Mass | Radiation | Heritage | Price |
|------|-------------------|-------|------|-----------|----------|-------|
| **Epson M-G370PDT0** | **0.10°/s (1σ, factory-compensated −40…+85 °C)**; turn-on repeatability 0.01°/s | 53 mW | 10 g | TID pass at 5, 10 and 20 krad(Si); no SEL data | Int-Ball2 inside the ISS (JAXA, since June 2023); no free flyer | ≈ $1–1.5k (est.) |
| **Bosch BMI088** | ±1°/s typical; ±0.015°/s/K | 16 mW | < 1 g | No data | OreSat0.5 ADCS card (Transporter-11, 2024; results not published); MOVE-III ground tests found it the most stable of their gyros | ≈ $10 |
| ADI ADIS16505-1 | 0.14°/s (x, z) but 1.4°/s (y) repeatability | ≈ 145 mW | — | No data | None found | ≈ $1k |
| ADI ADXRS453 × 3 | ±0.4°/s | ≈ 60 mW | — | No data | None confirmed; EOL notices | ≈ $100 each |
| Sensonor STIM210 | Excellent | ≈ 1.5 W | 52 g | Tested to 10 krad | ≈ 200 CubeSat units | $5–10k |
| TDK ICM-42688-P | ±0.5°/s | ≈ 3 mW | — | No data | None | ≈ $4 |
| Spacemanic Lodestone (module) | Not published | < 10 mW | < 10 g | — | 3+ missions | €800 |

The ADIS parts each exceed the whole sensor power allocation. The STIM210 exceeds it ten times.

### 5.3 Decision

**Primary: Epson M-G370PDT0.** It is the only affordable part that meets the 0.1°/s requirement without project
calibration, and the only one with published TID data (to 20 krad, four times the ≈ 5 krad two-year estimate). Its
space use is inside the ISS only. Mitigations: a current-limited, switchable rail (latch-up), and a lot-specific TID
and proton test if the launch review asks.

**Backup: Bosch BMI088**, on the OBC board, powered only if the primary fails. It has free-flyer heritage and costs
almost nothing in mass or power. As delivered its offset is ten times the requirement, so it is calibrated on a rate
table over −20…+50 °C before integration, to ≤ 0.1°/s residual. The Monte Carlo with a calibrated BMI088 matches the
M-G370 (§6.4); uncalibrated, it fails in most trials.

**With no gyro** (both failed), the flight software falls back to the magnetometer rate estimate. The Monte Carlo
gives a mean Sun angle of 23° and 33 % of sunlit time within 15°. Generation is then 4.2 W mean, 3.7 W worst trial,
still above the tumbling allocation. That is a degraded mode, not a loss.

## 6. ADCS Monte Carlo: tip-off, detumble, spin rate and gains

### 6.1 Method

`sqso_adcs_mc` (`software/rust/sqso-sil/src/bin/adcs_mc.rs`, library `sqso_sil::adcs_mc`) runs the flight
`sqso_fsw::adcs::Adcs` at 1 Hz against the mission simulation's attitude dynamics, with modelled sensors:

| Model | Value |
|-------|-------|
| Dynamics | Rigid body, RK4 at 0.25 s; stowed inertia until the wings deploy at 30 min, deployed after, with the angular momentum conserved |
| Orbit, field, Sun | 500 km sun-synchronous, LTAN 22:30; IGRF-14 tilted dipole; eclipse |
| Disturbances | Residual dipole 3 mA·m² (random direction), gravity gradient, aerodynamic torque |
| Tip-off | Per-axis normal, σ 5°/s, clipped at 15°/s (§6.1.1); also the dispenser distribution (σ 3.33°/s, 3σ = 10°/s) and stress sets up to 30°/s and 50°/s |
| Magnetometer | 50 nT noise, 300 nT bias per axis |
| Sun sensors | 6 cosine sensors, 5° error |
| Gyro | Turn-on bias, Gauss-Markov bias instability, white noise, scale factor and quantisation per §5 |
| Array | The mission simulation's 16-cell model |

#### 6.1.1 Tip-off from comparable dispensers

The rideshare dispensers a 1U would fly in all specify a tip-off **below 10°/s per axis**: Exolaunch EXOpod, EXOpod
Nova and EXOpod AIR (user manuals; 1U centre of gravity within ±20 mm), the Rocket Lab / Planetary Systems CSD data
sheet, and the value the MOVE-II team designed to for the ISIS QuadPack. NanoRacks' NRCSD (ISS) targets below 2°/s.
Measured flight data are sparse. Drop-tower tests of the CSD gave −4°/s nominal, up to −11.5°/s with an offset centre
of gravity. Quetzal-1 (1U) showed ±25°/s at first contact, inflated by its 0.74 A·m² permanent magnet. A rail-contact
estimate puts a 1U near the top of the envelope (about 45°/s, against 6°/s for a 3U), so the stress sets matter.

The nominal set used here (σ 5°/s, clipped at 15°/s) is wider than the dispenser specification read as 3σ
(σ 3.33°/s, magnitude mean 5.3°/s). The stress sets are σ 10°/s clipped at 30°/s, and σ 30°/s clipped at 50°/s.
The comparable dispenser for the baseline launch (D7, Transporter-class SSO) is the EXOpod, with QuadPack as the
alternative.

Each case is 40 trials of 2 days; statistics in steady state are from 0.75 day. "Captured" is the flight
criterion: rate across +Y below 0.5°/s and the spin about +Y below twice the spin rate, held 5 min.

```
cd software/rust
cargo run --profile sil -p sqso-sil --bin sqso_adcs_mc -- --trials 40 --spin 2,4,6 --gyro mg370
```

### 6.2 B-dot gain

The fastest detumble is near k* = 2n(1 + sin ξ) J_min / |B|² ≈ 10⁴ A·m²·s/T (Avanzini and Giulietti, JGCD 35(4),
2012). The flight gain was 5·10⁶. That cancels the rate across the field as fast as the field turns, so the momentum
follows the field and decays slowly:

| Gain | Not detumbled after 2 days | Median | Worst | Trials with a timeout |
|------|----------------------------|--------|-------|-----------------------|
| 5·10⁶ | 15 of 40 | 16 h | 35 h | 36 of 40 |
| **1·10⁴** | **0 of 40** | **0.75 h** | **1.3 h** | **0 of 40** |

### 6.3 Spin law, spin rate and gains

The momentum law τ = −k_h(h − h_s ŝ) − k_n ω⊥ has a second rest state: −Y on the Sun with the spin reversed. With the
earlier law, 23 of 40 trials settled there, wings away from the Sun. A spin-hold term, τ_y −= k_s(I_yy ω_y − h_s),
holds the spin positive; with k_s = 0.005 s⁻¹ no trial did.

Sweep over spin rate, momentum gain k_h and nutation gain k_n (spin hold 0.005, B-dot 10⁴, BMI088 calibrated), Sun
angle to +Y in sunlight:

| Spin | k_h | k_n | Mean | Worst trial p95 | Time within 15° | Worst at eclipse exit |
|------|-----|-----|------|-----------------|-----------------|------------------------|
| 2°/s | 0.005 | 4·10⁻⁵ | 12° | 110° | 75 % | 163° |
| 2°/s | 0.005 | 10⁻⁴ | 13° | 133° | 75 % | 147° |
| 2°/s | 0.02 | 10⁻⁵ | 78° | 104° | 3 % | 140° |
| 4°/s | 0.005 | 10⁻⁴ | 3.9° | 15° | 99.8 % | 19° |
| 6°/s | 0.005 | 10⁻⁴ | 2.5° | 9.8° | 100 % | 12° |

A high momentum gain with weak nutation damping is unstable (the 2°/s, k_h 0.02 row). The spin's gyroscopic
stiffness grows with its rate, so the disturbance torques tilt the axis less, and less is lost through eclipse,
when the Sun term is off.

The rows above used a diagonal inertia with Iyy 5–11 % above Izz. The structures review (RID STR-01) computed the
layout-B tensor: Iyy/max(Ixx, Izz) is only 1.00–1.06 and the products tilt the principal axes. With that tensor
(mean of the CBE and MEV values, products included), 6°/s and k_n = 10⁻⁴ give 8.7° mean and 97.1 % within 15°. The
avionics review re-tuned for it.

**Decision (design review, 2026-10-27): 8°/s, k_h = 0.005 s⁻¹, k_n = 3·10⁻⁴ N·m·s with the gyro and 1·10⁻⁴ without
it (3·10⁻⁴ fails without a gyro: 55° mean), k_s = 0.005 s⁻¹, B-dot 10⁴** (`sqso_fsw::adcs::Gains::FLIGHT`,
`NUTATION_GAIN_NO_GYRO`; `ADCS_BDOT_GAIN`). Coil power once captured 29 mW. At 8°/s the 1 Hz sun-sensor sampling
sees 8° between samples; the law uses only the Sun direction, and the Monte Carlo includes the sampling.

### 6.4 Robustness at the chosen gains

Flight gains, the final CAD inertia tensor with products (MASS_BUDGET 3.2 §6.1 and §6.5: the mean of CBE 1,520.0 g and
MEV 1,754.9 g as the default plant, each as its own row), the magnetometer window and the coil coupling modelled, 40
trials of 2 days per case (`software/simulation/cdr/avionics/adcs_mc_cdr.sh`; re-run in full 2026-10-27 for
`docs/hardware/ADCS_DESIGN.md` 1.9 §11.4 and §11.6, with the gating, AntS, wrong-preset and photodiode cases of the CDR
round 3 closure; slowest capture is the flight software's declared end of the detumble):

| Case (8°/s unless stated) | Not detumbled | Slowest capture | Mean Sun angle | Worst p95 | Within 15° | Worst eclipse exit | Worst-trial generation |
|---------------------------|---------------|-----------------|----------------|-----------|------------|--------------------|------------------------|
| M-G370PDT0 (re-run 2026-10-27 for ADCS_DESIGN 1.9, MASS_BUDGET 3.2 tensors, stowed products corrected) | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 10° | 4.50 W |
| BMI088, calibrated | 0 | 1.3 h | 1.8° | 7.2° | 100 % | 10° | 4.50 W |
| BMI088, bias 0.2°/s | 0 | 1.7 h | 2.5° | 9.5° | 100 % | 10° | 4.50 W |
| BMI088, bias 0.3°/s | 1 (3 timeouts) | 2.3 h | 8.9° | 150° | 92.7 % | 166° | 2.48 W |
| BMI088, bias 0.5°/s | 0 (12 timeouts) | 7.2 h | 26° | 162° | 75.4 % | 166° | 2.48 W |
| BMI088, uncalibrated | 1 (32 timeouts) | 2.5 h | 73° | 166° | 21.5 % | 172° | 2.37 W |
| No gyro (k_n 1·10⁻⁴) | 0 | 1.2 h | 20° | 56° | 41.5 % | 33° | 3.87 W |
| Tip-off σ 3.33°/s (dispenser, 3σ = 10°/s) | 0 | 1.0 h | 1.8° | 7.3° | 100 % | 10° | 4.50 W |
| Tip-off σ 10°/s, up to 30°/s | 0 | 1.8 h | 1.8° | 7.3° | 100 % | 10° | 4.50 W |
| Tip-off σ 30°/s, up to 50°/s | 0 | 1.6 h | 1.8° | 7.3° | 100 % | 10° | 4.50 W |
| Residual dipole 10 mA·m² | 0 | 3.0 h | 4.7° | 21° | 98.0 % | 29° | 4.46 W |
| Residual dipole 20 mA·m² | 0 (3 timeouts) | 11.8 h | 9.6° | 45° | 81.0 % | 58° | 4.21 W |
| Residual dipole 50 mA·m² | 0 (23 timeouts) | 31.7 h | 40° | 170° | 34.0 % | 166° | 2.62 W |
| Magnetometer 200 nT, sun sensors 10° | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 9° | 4.50 W |
| 4°/s, residual dipole 10 mA·m² | 0 | 3.0 h | 11° | 45° | 77.4 % | 60° | 4.20 W |
| 6°/s, k_n 1·10⁻⁴ (the version A gains) | 0 | 1.3 h | 2.4° | 9.6° | 100 % | 12° | 4.50 W |
| Iyy/Izz 1.000 / 1.005 / 1.020 / 1.050 | 0 / 0 / 0 / 0 | 1.3 h / 1.3 h / 1.3 h / 1.3 h | 20° / 2.0° / 1.8° / 1.8° | 162° / 7.7° / 7.2° / 7.2° | 80.0 % / 99.9 % / 100 % / 100 % | 179° / 30° / 9° / 9° | 3.62 W / 4.49 W / 4.50 W / 4.50 W |
| **CAD tensor CBE / MEV (MASS_BUDGET 3.2: 1,520.0 g / 1,754.9 g)** | 0 / 0 | 1.4 h / 1.2 h | 1.8° / 1.7° | 7.0° / 6.8° | 100 % / 100 % | 11° / 8° | 4.50 W / 4.50 W |
| Wing W1 only, geometry preset 0 / 1 / 2 (wrong) | 0 / 0 / 1 | 1.4 h / 1.4 h / 1.4 h | 6.0° / 8.4° / 14° | 12° / 14° / 29° | 99.9 % / 99.9 % / 56.3 % | 24° / 17° / 49° | 3.02 W / 3.06 W / 2.94 W |
| Wing W2 only, geometry preset 0 / 2 / 1 (wrong) | 1 / 0 / 1 | 1.1 h / 1.1 h / 1.2 h | 7.0° / 9.3° / 16° | 14° / 14° / 30° | 99.6 % / 99.9 % / 38.3 % | 26° / 17° / 48° | 3.01 W / 3.07 W / 2.92 W |
| Wings stowed, AntS held 25 h (C-18), preset 0 / 3 | 0 / 0 | 1.2 h / 1.2 h | 2.5° / 4.3° | 8.7° / 11° | 100 % / 100 % | 18° / 17° | 1.53 W / 1.53 W |
| AntS 3 h after the wings (C-18), ungated / gated | 0 / 0 | 1.3 h / 1.3 h | 1.8° / 1.9° | 7.3° / 7.5° | 100 % / 100 % | 10° / 10° | 4.50 W / 4.50 W |
| Magnetometer gating, 1.5 h/day Jetson + 53 min/day keyed (SRS-F143) | 0 | 1.3 h | 1.9° | 7.6° | 100 % | 10° | 4.50 W |
| Gating with residual dipole 10 mA·m² / no gyro | 0 / 0 | 3.0 h / 1.2 h | 5.1° / 17° | 22° / 52° | 96.7 % / 51.4 % | 33° / 41° | 4.45 W / 4.05 W |
| Face photodiodes with Earth albedo (ADCS_DESIGN 1.9 §11.6) / without albedo | 0 / 0 | 1.3 h / 1.3 h | 12° / 2.1° | 21° / 8.4° | 73.6 % / 100 % | 20° / 11° | 4.41 W / 4.50 W |
| Coils on through the sample, software unaware (gyro / no gyro) | 0 / 0 (27 timeouts) | 1.3 h / 5.1 h | 1.8° / 68° | 7.3° / 164° | 100 % / 11.5 % | 10° / 176° | 4.50 W / 2.50 W |

Every nominal case captures within 1.8 h, inside the 3-orbit requirement (ADCS-F001) and the 285 min timeout. (The
"not detumbled" 1 of a one-wing row is the Monte Carlo's own test, rate across the spin axis below 0.5°/s, which a
spin held off the principal axis may not meet; the flight software's own end of the detumble came within 1.4 h, with
no timeout.)
Tip-off is not the driver.

**The residual dipole is.** It needs a requirement: ≤ 3 mA·m² is the design value; 10 mA·m² still works at 8°/s but
not at 4°/s; 20 mA·m² degrades the pointing and 50 mA·m² defeats it. Published 1U values span 1–10 mA·m², with
outliers near 50 (UWE-3) and 150 mA·m² (ESTCube-1). On MOVE-II (1U), a current loop in the solar-cell wiring spun the
satellite past 500°/s. So: back-wire each wing's string (return conductor along the string, no loop area), keep
ferromagnetic parts away from the magnetometer, and measure the flight unit in a Helmholtz cage with the wings deployed
and illuminated. That is a magnetic-cleanliness requirement for the EM (TP-ADCS).

**Comparison with flight practice.** MOVE-II's spinning sun-pointing law (5.73°/s, coils on in eclipse) reached
20.6° ± 7.7° in hardware-in-the-loop tests; this law at 8°/s gives 1.8° in simulation with the generic sun-sensor
error and 12° with the face photodiodes and Earth albedo (ADCS_DESIGN 1.9 §11.6). The difference is the
nutation damping and the spin hold, and the simulation's ideal coils: expect the flight result between the two.

### 6.5 Limits

- The tip-off distribution is a dispenser specification, not a measurement of this spacecraft.
- Rigid body only: the flexible 2 m antenna tapes are not modelled (GNC-03, a verification item).
- The sensor errors are datasheet values.
- Rod remanence and recoil are not modelled (GNC-06); the Monte Carlo is re-run with measured rod data on the EM.
- The coils' field at the magnetometer is modelled; the flight software samples 50 ms after a 900 ms burst. The coil
  driver's own hardware off-timer is part of the OBC board design (AV-03).
- Aerodynamic torque at solar maximum (2.1·10⁻¹² kg/m³ mean at 500 km) is 4.5–8.9·10⁻⁸ N·m for a 1–2 cm centre of
  pressure offset, comparable to a 2 mA·m² residual dipole; it is in the model.
- The final flight inertia comes from the CAD (`docs/environment/MASS_BUDGET.md` 3.1 §6.5); the Iyy/Izz sweep above
  covers its value.

## 7. Battery size

### 7.1 The question

The mission simulation (SQSO-SIL-001 §7) found the pack at the end of charge for most of the mission: depth of
discharge about 2 % per eclipse, lowest SOC in operations 98 %. Is the pack too big?

### 7.2 What sets the size

| Criterion | Energy needed | Pack it implies |
|-----------|---------------|-----------------|
| Eclipse, ≤ 10 % DoD (BASELINE §5) | ≈ 1 Wh per 36 min eclipse | ≈ 10 Wh |
| Worst simulated drawdown (wings and ADCS both failed, §7.3), ≤ 30 % contingency DoD (SYS-112) | ≈ 3.4 Wh | ≈ 11 Wh |
| End-of-life fade (20 %) and margin | — | ×1.25–1.5 |
| **Energy sizing** | | **≈ 15–20 Wh** |
| Baseline | | 50 Wh: 2.5–3 times the energy need |

The pack sits full because the array produces more than the loads use in every orbit, not because it is large. A
pack of any size that closes the eclipse sits full too. So a smaller pack would not cure "sitting full"; a lower
end-of-charge voltage does.

### 7.3 Simulated cases

`sqso_sil --scenario commissioning` (14 days) with the array aged two years (`array_years=2`), the pack scaled
(`battery_scale`: 1.0 = 50 Wh, 0.58 ≈ 29 Wh, 0.29 ≈ 14.5 Wh) and the failures of `config.rs`. Largest depth of
discharge per eclipse, and the lowest SOC:

| Case | 50 Wh | 29 Wh | 14.5 Wh |
|------|-------|-------|---------|
| Nominal | 2.0 %, 98 % | 3.5 %, 97 % | 7.1 %, 94 % |
| Wings not deployed | 2.4 %, 66 % | 4.1 %, 71 % | 7.9 %, 84 % |
| ADCS failed (tumbling) | 2.0 %, 98 % | 3.6 %, 97 % | 7.2 %, 93 % |
| Wings and ADCS both failed | 3.2 %, 62 % | 5.5 %, 64 % | **10.8 %** (above the 10 % rule), 69 % |
| Jetson rail stuck on (both series switches failed) | **flat**: 236 under-voltage trips | **flat**: 322 | **flat**: 473 |

The largest drawdown from any peak, LEOP included, was 1.0 Wh nominal, 1.2 Wh with the wings stowed and 3.4 Wh with
the wings and the ADCS both failed. The lowest SOC in the degraded cases is not a drawdown: the pack leaves the
dispenser at about 58 % (60 % at integration, then storage), and without the wings a larger pack takes longer to
fill. The transponder delivered the same service at every size (98 % of the uplink demand, 80–82 min a day keyed in
the nominal and single-failure cases): the duty and SOC gates bind before the pack does.

No pack survives a Jetson rail that cannot be switched off: the module draws more than the array gives, and the
pack size only sets how soon the BMS cuts out. That case needs the two series switches to be independent (the PDU
load switch on PAYLOAD_EN, and the carrier converter's enable on JETSON_PWR, different parts on different lines,
with the latch-up trip acting on the PDU switch), not a bigger battery.

### 7.4 Options

| Option | Energy | Mass change | Bus | Consequence |
|--------|--------|-------------|-----|-------------|
| **Keep 4S1P INR18650-35E (≈ 50 Wh)** | 2.5–3× need | 0 | 14.4 V | No change to loads or thresholds |
| 4S1P lower-capacity 18650 | ≈ 29 Wh | ≈ 0 (same can) | 14.4 V | No gain |
| 4S1P 14500 cells (≈ 0.8 Ah) | ≈ 12 Wh | ≈ −110 g | 14.4 V | Below the 15–20 Wh sizing; the double failure's 3.4 Wh drawdown is 28 % of it; less cell heritage |
| 2S1P INR18650-35E (ICEPS2 Type A class) | ≈ 25 Wh | ≈ −125 g (two cells, half the holder) | 7.2 V | Every 14.4 V load and threshold changes: Jetson input converter, PA supply, heater, magnetorquer drivers, burn-wire current, `VBATT_PLAUSIBLE_V` and `MEASURED_LOW_BATTERY_V` in the flight software, BASELINE §5 |

### 7.5 Decision (D8)

**Keep the 4S1P ≈ 50 Wh pack, and charge it to 4.10 V a cell (16.4 V) instead of 4.20 V.**

- *Too big for energy, yes, but not worth shrinking now.* The only real saving is 2S1P, ≈ 125 g, and it re-plans the
  bus. The COTS buys of §1 recover 200–250 g without touching the bus. If the MEL re-run with quoted masses still
  leaves under 10 % mass margin, take 2S1P then.
- *What the extra capacity buys:* margin for a stuck load until the OBC or the guard sheds it, storage in the
  dispenser (months at the integrator's state of charge), deployment-burn current at low C-rate (1.9 A is C/1.8),
  and room for cell capacity spread and fade.
- *Charge voltage.* Lithium-ion calendar ageing grows with the state of charge, steeply at the top of charge for
  nickel-rich cells such as the INR18650-35E (Keil et al., *J. Electrochem. Soc.* 163(9), 2016), and the pack spends
  most of the mission at the end of charge. 4.10 V gives up about 9 % of capacity. Even at the end of life, the
  double-failure drawdown (3.4 Wh) is then under 10 % of the pack. It also widens the margin to the BMS's 4.25 V
  over-voltage cut-off from 0.05 V to 0.15 V a cell. The full mission at both set points
  (`sqso_sil --scenario full`, the reference run of SQSO-SIL-001 §7):

  | Set point | Checks | Lowest SOC | Largest DoD | Transponder keyed | Capacity fade, two years |
  |-----------|--------|------------|-------------|-------------------|--------------------------|
  | 4.20 V | 37 pass | 98.3 % | 3.1 % | 56 min a day | 1.4 % |
  | **4.10 V** | **37 pass** | **90.4 %** | **3.2 %** | **57 min a day** | **1.4 %** |

  The mission gives up nothing measurable. The model's calendar fade has only a linear dependence on the state of
  charge, so it shows no difference in fade; the case for 4.10 V rests on the published ageing data and the
  over-voltage margin, and is confirmed by the cell test below.

- The pack is still below the 80 Wh ISS assessment threshold and the 100 Wh FAA note.
- Verify by LEO-profile cycling of flight-lot cells at 4.10 V (BATTERY_DESIGN §9).

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| F | 2026-10-27 | SMART-QSO Team (documentation audit) | Documentation audit (post-CDR 2026): magnetometer decision to the OBC board design (PNI RM3100 on board A2, SQSO-OBC-BRD-001 §9.4, item C1) in §1 and §2.7, the modules kept as the fallback; SA818-U engineering-model receiver removed from §3.2 (the EM uses the flight-design RF board, ETP Rev C §2); §2.8 notes the superseded notices |
| C | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3) | §6.4 re-run with the final CAD tensors (C-13 ballast as re-placed; CBE and MEV rows, the flight software's mean as the default plant) and the round-3 flight software |
| E | 2026-10-27 | SMART-QSO Team (CDR round 3 closure, GN&C) | §6.4 re-run in full (ADCS_DESIGN 1.9: stowed products corrected, MASS_BUDGET 3.2 tensors); rows added for the wrong one-wing preset, the wings stowed with the AntS held 25 h, the AntS 3 h after the wings, magnetometer gating and the face photodiodes with albedo (P3F-G-04, -05, C-21, C-22); the wing rows now ADCS_DESIGN's |
| D | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, fourth part) | §6.4 the M-G370 and CAD CBE/MEV rows re-run at MASS_BUDGET 3.2's tensors (RID GNC-07); the wing rows are ADCS_DESIGN 1.8 §11.4's to copy |
| B | 2026-10-27 | SMART-QSO Team | Design review: frame RFQ with a recessed −Z face (STR-11); heater set points +5/+9 °C (THM-02); §6 gains and spin from the Monte Carlo on the computed inertia (STR-01) |
| A | 2026-10-26 | SMART-QSO Team | Initial issue: COTS survey and build-versus-buy decisions; gyro; ADCS Monte Carlo and gains; battery size; transponder enable and battery mount hardware |
