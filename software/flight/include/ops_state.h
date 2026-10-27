/**
 * @file ops_state.h
 * @brief Operations state across resets: the commissioning hold and the
 *        ground's transponder enable
 *
 * Commissioning hold (LV-01): set by GSE ARM_FLIGHT, cleared only by the
 * authenticated COMMISSIONING_RELEASE telecommand. While it holds, every way
 * into ACTIVE is refused (the mode manager's ActiveAllowed guard), so the
 * payload stays off, and the transponder stays off.
 *
 * Transponder enable (XPDR_ENABLE / XPDR_DISABLE): a ground gate on the FM
 * transponder, persisted; it adds to the transponder's gates, never
 * overrides them.
 *
 * Stored as a two-slot NVM record in FLASH_REGION_OPS. A blank record (a
 * bench unit before ARM_FLIGHT) means no hold; a corrupt one means held
 * (a lost record must not release a held spacecraft). Before
 * ops_state_init() the hold reads as not set, as for a blank record: the
 * executive calls ops_state_init() before it starts the mode manager.
 *
 * @requirement SRS-F082 The software shall hold the mode at IDLE, the payload and the transponder off from ARM_FLIGHT until an authenticated commissioning release
 * @requirement SRS-F083 The software shall let the ground stop and allow the transponder by telecommand, persisted across resets
 */

#ifndef SMART_QSO_OPS_STATE_H
#define SMART_QSO_OPS_STATE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/** Load the record; a corrupt one sets the hold (and logs a fault) */
SmartQsoResult_t ops_state_init(void);

/** Whether the commissioning hold is in force */
bool ops_commissioning_hold(void);

/**
 * @brief Set (ARM_FLIGHT) or clear (COMMISSIONING_RELEASE) the hold
 * @return SMART_QSO_OK; SMART_QSO_ERROR_IO if not persisted (the hold
 *         changes anyway)
 */
SmartQsoResult_t ops_set_commissioning_hold(bool hold);

/** Whether the ground allows the transponder (false while held) */
bool ops_xpdr_allowed(void);

/**
 * @brief Allow or stop the transponder, persisted
 * @return SMART_QSO_OK; SMART_QSO_ERROR_IO if not persisted
 */
SmartQsoResult_t ops_set_xpdr_enabled(bool enabled);

/**
 * @brief Check the RAM copy against the record (AV-07), every minute
 *
 * Merges toward the safe side (held if either holds, the transponder
 * stopped if either stops it, the record's geometry preset), persists the
 * merge, and logs a disagreement.
 */
void ops_state_verify(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_OPS_STATE_H */
