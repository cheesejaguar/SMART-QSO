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
    "SE-10": ("CD", "SRS 1.6 requirements defined; SQSO-FSW-012 Appendix C mapping; traceability 0 undefined"),
}
