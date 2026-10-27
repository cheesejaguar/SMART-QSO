/**
 * @file passivation.h
 * @brief End-of-life passivation by telecommand (47 CFR 97.207(g)(1)(iii))
 *
 * Two authenticated telecommands from two key custodians (RIDs REG-01,
 * EPS-06, FSW-07, SMA-13; SQSO-ICD-013 §6.4):
 *
 * 1. PASSIVATE_ARM (0x70, operational key, argument the ASCII code "EOL!")
 *    arms the EPS's one-shot charge disconnect with the EPS controller as a
 *    second processor (RID P3E-11; SMART-QSO-ICD-001 1.5 §4.6): EOL_STATE
 *    must show the arm latch clear and F1 intact (else SMART_QSO_ERROR_IO);
 *    the key 0x3C, 0xC3 is written to EOL_KEY (0x7E) and EOL_STATE must show
 *    the key up (bit 3); GPIO_PIN_EOL_ARM rises, and once EOL_STATE shows the
 *    latch set (bit 0, at most PASSIVATION_ARM_CONFIRM_MS) it drops and
 *    PASSIVATE_EXECUTE is accepted for PASSIVATION_ARM_WINDOW_MS from the
 *    command. A latch already set is armed at once. A reset or the end of
 *    the window disarms the software; the latch clears only at the EPS's
 *    power-up.
 * 2. PASSIVATE_EXECUTE (0x71, recovery key) inside the window, with the
 *    latch still set, commits the passivated record in FLASH_REGION_EOL
 *    (from then on every boot carries the sequence through) and runs, in
 *    this order (RID REG-12):
 *    - the fire, in sunlight, preferably with the EPS board at or above
 *      PASSIVATION_FIRE_MIN_BOARD_C by the INA228 die temperatures (F1's
 *      rating, SMART-QSO-ICD-001 1.6 §4.6): a cold board is waited for at
 *      most PASSIVATION_BOARD_WAIT_MAX_MS of each attempt, and an unreadable
 *      one not at all (RID SMA-B20; the reasons in passivation_waiting()),
 *      while the spacecraft still transmits:
 *      GPIO_PIN_EOL_FIRE high at least PASSIVATION_FIRE_MIN_MS, until
 *      EOL_STATE reports the charge path open (bit 1 set, bit 2 clear) or
 *      PASSIVATION_FIRE_MAX_MS; then low. The open path seen at any time
 *      ends the fire (the array, and the sunlight seen, go with it); An eclipse before the minimum restarts the fire
 *      in the next sunlight; a fire the EPS does not confirm is tried
 *      again, at most PASSIVATION_MAX_FIRE_ATTEMPTS times in all (across
 *      resets too), and PASSIVATION_FIRE_DEADLINE_MS after the fire began
 *      it goes to the report whatever;
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
 * @requirement SRS-F157 Two-processor passivation arm: the EPS controller's key and the OBC's EOL_ARM, the latch confirmed before PASSIVATE_EXECUTE is accepted
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
/** EOL_ARM is held at most this long for the EPS's latch to show set, ms */
#define PASSIVATION_ARM_CONFIRM_MS 2000U
/** The fire gives up this long after it began without an end, ms */
#define PASSIVATION_FIRE_DEADLINE_MS 86400000U
/** Fire sequences at most, across resets */
#define PASSIVATION_MAX_FIRE_ATTEMPTS 3U
/** The fire waits for the EPS board at or above this, °C (F1, ITV4030, rated from −10 °C) */
#define PASSIVATION_FIRE_MIN_BOARD_C 0.0
/** A cold EPS board is waited for at most this long of each attempt, ms (6 h, RID SMA-B20) */
#define PASSIVATION_BOARD_WAIT_MAX_MS 21600000U
/** passivation_waiting(): waiting for sunlight */
#define PASSIVATION_WAIT_SUN 0x01U
/** passivation_waiting(): waiting for the EPS board to reach PASSIVATION_FIRE_MIN_BOARD_C */
#define PASSIVATION_WAIT_BOARD 0x02U
/** passivation_waiting(): the board still cold after PASSIVATION_BOARD_WAIT_MAX_MS: not held for it */
#define PASSIVATION_BOARD_BOUND 0x04U
/** passivation_waiting(): the board's temperature unreadable: not held for it */
#define PASSIVATION_BOARD_UNKNOWN 0x08U
/** Final report frames */
#define PASSIVATION_FINAL_REPORTS 3U
/** Between final report frames, ms */
#define PASSIVATION_REPORT_INTERVAL_MS 60000U
/** From the first report frame to TX-OFF, ms */
#define PASSIVATION_REPORT_WINDOW_MS 180000U

/** Where the sequence is */
typedef enum {
    PASSIVATION_IDLE       = 0, /**< Nothing armed */
    PASSIVATION_ARMED      = 1, /**< Arming (the latch being confirmed) or armed, PASSIVATE_EXECUTE accepted */
    PASSIVATION_FIRING     = 2, /**< Committed, the disconnect being fired (still transmitting) */
    PASSIVATION_PASSIVATED = 3, /**< Silenced: TX-OFF latched, transmitters off for good */
    PASSIVATION_REPORTING  = 4  /**< The fire done, the final report going out (TX-OFF follows) */
} PassivationPhase_t;

/** Load the record at boot and apply it (before rf_control_init()) */
SmartQsoResult_t passivation_init(void);

/**
 * @brief PASSIVATE_ARM
 * @return SMART_QSO_OK (the latch confirmed by passivation_tick());
 *         SMART_QSO_ERROR_INVALID for a wrong code; SMART_QSO_ERROR if
 *         already passivated; SMART_QSO_ERROR_IO if the EPS controller does
 *         not answer, shows F1 not intact, refuses the key, or EOL_ARM
 *         cannot be driven
 */
SmartQsoResult_t passivation_arm(const uint8_t *code, size_t len);

/**
 * @brief PASSIVATE_EXECUTE
 * @return SMART_QSO_OK; SMART_QSO_ERROR if not armed (or the EPS's latch
 *         not set) or already passivated
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

/**
 * @brief Why the fire waits, or went ahead without the EPS board
 *        condition, as last seen (HK1 PS, the final report)
 * @return PASSIVATION_WAIT_SUN | PASSIVATION_WAIT_BOARD |
 *         PASSIVATION_BOARD_BOUND | PASSIVATION_BOARD_UNKNOWN; 0 when the
 *         fire went ahead with every condition met
 */
uint8_t passivation_waiting(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_PASSIVATION_H */
