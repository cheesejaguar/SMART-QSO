/**
 * @file station_id.h
 * @brief Spacecraft station identity on the AX.25 links
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The space station transmits under the licensee's call sign (47 CFR 97.119;
 * decision D1, docs/BASELINE_PARAMETERS.md: licensee KK6GDB). SSID 11 is the
 * conventional APRS SSID for balloons, aircraft and spacecraft. The beacon
 * source address and the telecommand destination address both use this
 * identity; beacon.h BEACON_CALLSIGN is its text form.
 */

#ifndef SMART_QSO_STATION_ID_H
#define SMART_QSO_STATION_ID_H

/** AX.25 call sign (up to 6 characters, A-Z and 0-9) */
#define SQSO_CALLSIGN   "KK6GDB"

/** AX.25 SSID (0-15) */
#define SQSO_SSID       11U

#endif /* SMART_QSO_STATION_ID_H */
