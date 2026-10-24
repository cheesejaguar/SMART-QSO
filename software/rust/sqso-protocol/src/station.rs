//! Spacecraft station identity on the AX.25 links (47 CFR 97.119; decision
//! D1, `docs/BASELINE_PARAMETERS.md`). Must equal `station_id.h`; the C
//! contract test (`tests/test_rust_contract.c`) checks that it does.

/// Licensee call sign (AX.25: up to 6 characters, A-Z and 0-9)
pub const CALLSIGN: &str = "KK6GDB";

/// SSID 11, the conventional APRS SSID for spacecraft
pub const SSID: u8 = 11;
