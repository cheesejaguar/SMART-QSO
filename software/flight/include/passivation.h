/**
 * @file passivation.h
 * @brief End-of-life passivation by telecommand (47 CFR 97.207(g)(1)(iii))
 *
 * Two authenticated telecommands from two key custodians (RIDs REG-01,
 * EPS-06, FSW-07, SMA-13; SQSO-ICD-013 §6.4):
 *
 * 1. PASSIVATE_ARM (0x70, operational key, argument the ASCII code "EOL!")
 *    arms the sequence for PASSIVATION_ARM_WINDOW_MS. Nothing is driven; a
 *    reset or the end of the window disarms.
 * 2. PASSIVATE_EXECUTE (0x71, recovery key) inside the window commits the
 *    passivated record in FLASH_REGION_EOL (from then on every boot carries
 *    the sequence through) and runs, in this order (RID REG-12):
 *    - the EPS one-shot charge disconnect, in sunlight, while the
 *      spacecraft still transmits: GPIO_PIN_EOL_ARM rises (the EPS's arm
 *      latch), 200 ms later GPIO_PIN_EOL_FIRE goes high and is held at
 *      least PASSIVATION_FIRE_MIN_MS, until the EPS controller's EOL_STATE
 *      register reports the charge path open or PASSIVATION_FIRE_MAX_MS;
 *      then both are low. An eclipse before the minimum restarts the fire
 *      in the next sunlight; a fire the EPS does not confirm is tried
 *      again, at most PASSIVATION_MAX_FIRE_ATTEMPTS times in all (across
 *      resets too);
 *    - the final report: PASSIVATION_FINAL_REPORTS housekeeping frames
 *      (HK1 and HK2, the latter carrying EOL_STATE, the fuse state and the
 *      attempts), PASSIVATION_REPORT_INTERVAL_MS apart
 *      (passivation_take_report());
 *    - PASSIVATION_REPORT_WINDOW_MS after the report started: TX-OFF latched
 *      (rf_control_tx_off()) and RF_INHIBIT_PASSIVATED set. A fire that
 *      failed is silenced the same way after its report.
 *
 * After that every boot sets RF_INHIBIT_PASSIVATED again, from the record or
 * from the EPS reporting the charge path open, before anything can
 * transmit; no telecommand lifts it. The receiver stays powered. A reset
 * during the fire runs it again at boot (within the attempts); a reset
 * during the report sends the report again, then silences.
 *
 * @requirement SRS-F110 Two-step authenticated end-of-life passivation
 * @requirement SRS-F111 Passivation order: record, charge disconnect through two series lines while transmitting, final report, then TX-OFF
 * @requirement SRS-F112 Transmitters off for good after passivation
 * @requirement SRS-F155 The final passivation report: three frames over three minutes carrying EOL_STATE, the fuse state and the attempts, before TX-OFF
 */

#ifndef SMART_QSO_PASSIVATION_H
#define SMART_QSO_PASSIVATION_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/** PASSIVATE_EXECUTE is accepted this long after PASSIVATE_ARM, ms */
#define PASSIVATION_ARM_WINDOW_MS 600000U
/** EOL_FIRE is held at least this long, ms */
#define PASSIVATION_FIRE_MIN_MS   60000U
/** and at most this long without the EPS confirming, ms */
#define PASSIVATION_FIRE_MAX_MS   120000U
/** Fire sequences at most, across resets */
#define PASSIVATION_MAX_FIRE_ATTEMPTS 3U
/** Final report frames */
#define PASSIVATION_FINAL_REPORTS 3U
/** Between final report frames, ms */
#define PASSIVATION_REPORT_INTERVAL_MS 60000U
/** From the first report frame to TX-OFF, ms */
#define PASSIVATION_REPORT_WINDOW_MS 180000U

/** Where the sequence is */
typedef enum {
    PASSIVATION_IDLE       = 0, /**< Nothing armed */
    PASSIVATION_ARMED      = 1, /**< Armed, PASSIVATE_EXECUTE accepted */
    PASSIVATION_FIRING     = 2, /**< Committed, the disconnect being fired (still transmitting) */
    PASSIVATION_PASSIVATED = 3, /**< Silenced: TX-OFF latched, transmitters off for good */
    PASSIVATION_REPORTING  = 4  /**< The fire done, the final report going out (TX-OFF follows) */
} PassivationPhase_t;

/** Load the record at boot and apply it (before rf_control_init()) */
SmartQsoResult_t passivation_init(void);

/**
 * @brief PASSIVATE_ARM
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID for a wrong code;
 *         SMART_QSO_ERROR if already passivated
 */
SmartQsoResult_t passivation_arm(const uint8_t *code, size_t len);

/**
 * @brief PASSIVATE_EXECUTE
 * @return SMART_QSO_OK; SMART_QSO_ERROR if not armed or already passivated
 */
SmartQsoResult_t passivation_execute(void);

/** Every main-loop pass: the arm's window and the fire sequence */
void passivation_tick(bool sunlit);

/** Check the RAM copy against the record (AV-07), every minute */
void passivation_verify(void);

/** The phase now */
PassivationPhase_t passivation_phase(void);

/** Whether the spacecraft is passivated (silenced: transmitters off for good) */
bool passivation_passivated(void);

/**
 * @brief The next final-report frame, if one is due now
 * @return its number, 1 to PASSIVATION_FINAL_REPORTS; 0 if none is due (the
 *         executive then downlinks HK1 and HK2)
 */
uint8_t passivation_take_report(void);

/** The fire sequences run so far (in the record, across resets) */
uint8_t passivation_attempts(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_PASSIVATION_H */
