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
 * 2. PASSIVATE_EXECUTE (0x71, recovery key) inside the window: TX-OFF is
 *    latched (rf_control_tx_off()), the passivated record is persisted in
 *    FLASH_REGION_EOL, RF_INHIBIT_PASSIVATED is set, then in sunlight the
 *    EPS one-shot charge disconnect is fired: GPIO_PIN_EOL_ARM rises (the
 *    EPS's arm latch), 200 ms later GPIO_PIN_EOL_FIRE goes high and is held
 *    at least PASSIVATION_FIRE_MIN_MS, until the EPS controller's EOL_STATE
 *    register reports the charge path open or PASSIVATION_FIRE_MAX_MS; then
 *    both are low. An eclipse before the minimum restarts the fire in the
 *    next sunlight.
 *
 * After that every boot sets RF_INHIBIT_PASSIVATED again, from the record or
 * from the EPS reporting the charge path open, before anything can
 * transmit; no telecommand lifts it. The receiver stays powered. A fire
 * interrupted by a reset runs again at boot, at most three times.
 *
 * @requirement SRS-F110 Two-step authenticated end-of-life passivation
 * @requirement SRS-F111 Passivation order: TX-OFF, record, charge disconnect through two series lines
 * @requirement SRS-F112 Transmitters off for good after passivation
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

/** Where the sequence is */
typedef enum {
    PASSIVATION_IDLE       = 0, /**< Nothing armed */
    PASSIVATION_ARMED      = 1, /**< Armed, PASSIVATE_EXECUTE accepted */
    PASSIVATION_FIRING     = 2, /**< Passivated, the disconnect being fired */
    PASSIVATION_PASSIVATED = 3  /**< Passivated, the fire sequence done */
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

/** Whether the spacecraft is passivated (transmitters off for good) */
bool passivation_passivated(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_PASSIVATION_H */
