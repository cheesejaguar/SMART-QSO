/**
 * @file station_id.h
 * @brief Spacecraft station identity on the AX.25 links
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * PLACEHOLDER: the space station call sign is issued to the licensee
 * (decision D1, docs/BASELINE_PARAMETERS.md). Replace both values before any
 * filing or transmission. The beacon source address and the telecommand
 * destination address both use this identity.
 */

#ifndef SMART_QSO_STATION_ID_H
#define SMART_QSO_STATION_ID_H

/** AX.25 call sign (up to 6 characters, A-Z and 0-9) */
#define SQSO_CALLSIGN   "SQSO"

/** AX.25 SSID (0-15) */
#define SQSO_SSID       1U

#endif /* SMART_QSO_STATION_ID_H */
