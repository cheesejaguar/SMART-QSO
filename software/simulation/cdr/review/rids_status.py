"""Round-2 closures of the RIDs that round 1 assigned for implementation ("IMP").

Maps a RID to its final (disposition, evidence). Filled in as each implementation lands
and is verified by the closure panel. A RID absent here keeps its round-1 disposition.
"""

CLOSURE: dict[str, tuple[str, str]] = {
    # Implemented in round 1 by the avionics reviewer (verified by the gates in its report)
    "LV-01": ("CD", "Commissioning hold: FRAM region 11, COMMISSIONING_RELEASE 0x08; test_ops_state.c; SIL release at 132,811 s"),
    "LV-02": ("CD", "No magnetorquer output before 120 s (SRS-F086); SIL first coil command at 121 s"),
    "LV-04": ("CD", "gnss_schedule.rs: sessions at boot, every 95 min in LEOP, then 6 h; fix in HK1; GNSS_REQUEST 0x60; ground OD VI-LV-13"),
    "LV-07": ("CD", "DEPLOY_RETRY 0x50, retries 50 min apart; AntS over I2C (ICD_OBC_ADCS §12.3)"),
    "LV-14": ("CD", "XPDR_ENABLE 0x34, XPDR_DISABLE 0x35, GNSS_REQUEST 0x60, DEPLOY_RETRY 0x50"),
    "LV-15": ("CD", "RF silence latched once complete (SRS-F087; test_rf_silence.c)"),
    "LV-27": ("CD", "HK1 housekeeping frame every fifth beacon and on GET_HOUSEKEEPING 0x09"),
    "STR-01": ("CD", "BASELINE §4 mass properties; CAD layout: Iyy/max 1.057 CBE, 1.047 MEV, axis within 1.3° (SQSO-MECH-CAD-001); full tensor in the flight law"),
    "STR-15": ("CD", "8 s burn at 0.8 A, two resistors per wing on drivers A (PB10) and B (PE6), alternate attempts; SIL deploy.rs fixed and AntS modelled; VI-STR-06, VI-STR-13"),
    "STR-16": ("CD", "Wings first, then the AntS (SRS-F089); 37.8 mm nominal clearance (SQSO-MECH-CAD-001); stuck-wing case VI with the ISIS ICD"),
    "STR-27": ("CD", "ADCS_SET_GEOMETRY 0x24, presets 0-3, persisted; one-wing Monte Carlo (BUILD_VS_BUY §6.4)"),
    "SE-11": ("CD", "SQSO-SEC-001 Space System Protection and Security Plan (NASA-STD-1006A as practice)"),
    "EMC-05": ("CD", "ETP Rev C.1 §3.6: V-EMC-1, V-EMC-3, V-EMC-6"),
    "RF-12": ("CD", "RF board build-to design: SQSO-RF-BRD-001 (316 parts, ERC 0 errors, BOM with stresses); EM tests VI-RFB-01..09"),
    "EPS-07": ("CD", "S5 and W7 redrawn for the AntS: supply after DS1-DS3, switched by DEPLOY_EN, I2C through a TCA9517 (SQSO-EPS-BRD-001)"),
    "AV-08": ("CD", "Current-limited auto-retry rails: OBC 0.15 A, radio 0.12 A, sensors 0.30 A, GNSS 0.20 A, EPS controller 75 mA, AntS 1.37 A, PA 0.99 A (SQSO-EPS-BRD-001)"),
    "RFB-06": ("CD", "Radio trip 0.12 A with current telemetry; 3V3_AO ±2 % (≥ 3.22 V at the RF connector) (SQSO-EPS-BRD-001)"),
    "MCAD-01": ("CD", "Two coaxial 17-7PH torsion springs, clock-spring flex, damped stop: end-of-travel torque margin +0.14 cold (+0.13 one spring out) (MECHANISMS_ANALYSIS B §4A); VI-STR-04, VI-MECH-01, VI-MECH-02"),
    "MCAD-02": ("CD", "Deck 1.0 mm skeleton 9 g; MEL row 5 re-split, total unchanged (MASS_BUDGET 3.1)"),
    "STR-05": ("CD", "Two centre supports at (0, ±21) mm: Steinberg margins +0.09 to +0.36 (STRUCTURAL_ANALYSIS B); VI-STR-02"),
    "STR-10": ("CD", "Stowed protrusion 6.10 mm nominal, 6.38 mm worst case with the 0.5 mm panel recess (MECHANICAL_DESIGN B); VI-STR-08"),
    "STR-11": ("CD", "−Z face recessed 1.5 mm: AntS 1.25 mm inside the rail ends (MECHANICAL_DESIGN); frame RFQ"),
    "SE-12": ("CA", "Shell FE model (56,258 DOF): first mode 170 Hz, lowest margin +0.72 (STRUCTURAL_ANALYSIS B); materials list SQSO-MAT-001 Rev B"),
    "AV-01": ("CD", "OBC board build-to design SQSO-OBC-BRD-001 (356 parts, ERC 0) and the master pin allocation SQSO-ICX-001: every pin one function"),
    "AV-02": ("CD", "TPS3850H33-Q1 window-watchdog supervisor (5.6 s to 33-45 s window), backup to the 30 s internal watchdog; kick in flight software"),
    "AV-05": ("CD", "TONE_OK PC9, XPDR_KEY PG12, PTT_EXCITER PC7 (timer capture) with pull-downs (SQSO-ICX-001)"),
    "AV-09": ("CD", "UART through SN74LVC1T45-Q1 buffers powered from the carrier's 3.3 V: high impedance when the carrier is off"),
    "AV-10": ("CD", "RV-3028-C7 RTC on the always-on rail, its own I2C bus; time-stamps every reset"),
    "RF-21": ("CD", "Readbacks wired on the RF board (W4 J1-8/9/10) and the OBC (PC9, PG12, PC7)"),
    "EPS-14": ("CD", "GNSS_EN PE5, XPDR_EN PE4 (SQSO-ICX-001)"),
    "GNC-04": ("CH", "Magnetometer: PNI RM3100 on the OBC board, bought-module footprint as fallback (chair accepts OBC_BOARD_DESIGN §24 C1); driver in the flight software"),
    "OBCB-03": ("CD", "Gyro on the OBC through a carrier window (MCAD-04, MECHANICAL_DESIGN D-11)"),
    "OBCB-04": ("CD", "Jetson carrier build-to design SQSO-JET-BRD-001 (137 placements, ERC 0): 5.016 V buck and eFuse to VDD_IN, switched JET_3V3, translated UART"),
    "AV-06": ("CD", "PAYLOAD_EN and JETSON_PWR on separate lines with JET_PG readback; JET_3V3 high only when the whole chain is up (SQSO-JET-BRD-001, SQSO-ICX-001)"),
    "EPSB-06": ("CD", "Consistency pass: B1-B4 redrawn with DS3, diagram findings F1-F17 dispositioned, RADIATION D §5.3 trips, BSA B H8a, ICD-001 1.2, EMC B §5.1 (E6, E8), IARU request A.3 windows; SVGs rendered"),
    "OBCB-05": ("CD", "POWER_BUDGET_ANALYSIS 2.1 §8.6 LEOP case with the magnetorquers at 1.16-1.3 W: first orbit −1.30 Wh, three orbits −3.44 Wh (SOC 40 % at separation falls to 33 %)"),
    "SE-10": ("CD", "SRS 1.6 requirements defined; SQSO-FSW-012 Appendix C mapping; traceability 0 undefined"),
}
