/**
 * @file test_deployment.c
 * @brief Deployment sequence: 30-minute wait, burn wires, persistence, resets
 *
 * Drives the real sequence through the simulation HAL (separation switch,
 * the wings' two burn drivers and sense pin, the AntS antenna controller on
 * I2C bus 0, the persisted record, the reset cause). Order: the wings first,
 * then the antenna (STR-16); the wings alternate driver A and driver B.
 *
 * @requirement SYS-221 Deployables wait 30 minutes after separation; switch toggle resets the timer
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include "deployment.h"
#include "fault_mgmt.h"
#include "flight_log.h"
#include "nvm_record.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_i2c.h"
#include "hal/hal_reset.h"
#include "hal/hal_sim.h"

#define SECOND_MS   1000U
#define MINUTE_MS   (60U * 1000U)

/* The ISIS AntS controller (software/rust/sqso-fsw/src/ants.rs) */
#define ANTS_BUS        0U
#define ANTS_ADDR       0x31U
#define ANTS_ARM        0xADU
#define ANTS_DEPLOY     0xA5U
#define ANTS_STATUS     0xC3U

static uint8_t ants_reg(uint8_t reg)
{
    uint8_t value = 0U;
    assert_int_equal(hal_i2c_write_read(ANTS_BUS, ANTS_ADDR, &reg, 1U, &value, 1U), SMART_QSO_OK);
    return value;
}

/** Every AntS element switch deployed (status 0x0000) or stowed (0x8888) */
static void set_antenna_sense(bool deployed)
{
    uint8_t status[3] = {ANTS_STATUS, deployed ? 0x00U : 0x88U, deployed ? 0x00U : 0x88U};
    assert_int_equal(hal_i2c_write(ANTS_BUS, ANTS_ADDR, status, sizeof(status)), SMART_QSO_OK);
}

/*===========================================================================*/
/* Fixtures and helpers                                                       */
/*===========================================================================*/

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    hal_sim_set_reset_cause(HAL_RESET_POWER_ON);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, true);   /* DS2 released: high */
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, true);    /* wings stowed: the EPS pull-up */
    assert_int_equal(hal_i2c_init(ANTS_BUS, 100000U), SMART_QSO_OK);
    set_antenna_sense(false);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_DEPLOYMENT), SMART_QSO_OK);
    assert_true(deployment_init());
    return 0;
}

static int teardown(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    return 0;
}

static void advance(uint32_t total_ms, uint32_t step_ms)
{
    for (uint32_t t = 0U; t < total_ms; t += step_ms) {
        deployment_tick(step_ms);
    }
}

/** The AntS was armed and told to deploy, and not cancelled since */
static bool antenna_deploying(void)
{
    return (ants_reg(ANTS_DEPLOY) != 0U) && (ants_reg(ANTS_ARM) == 1U);
}

/** The four wing lines (one resistor each), in W1A, W1B, W2A, W2B order */
static const HalGpioPin_t k_wing_lines[4] = {
    GPIO_PIN_BURN_W1A, GPIO_PIN_BURN_W1B, GPIO_PIN_BURN_W2A, GPIO_PIN_BURN_W2B
};

/** How many wing lines are high */
static unsigned wing_lines_on(void)
{
    unsigned n = 0U;
    for (size_t i = 0U; i < 4U; i++) {
        n += hal_gpio_get(k_wing_lines[i]) ? 1U : 0U;
    }
    return n;
}

/** Exactly this wing line is high (and the master arm) */
static bool only(HalGpioPin_t pin)
{
    return hal_gpio_get(GPIO_PIN_DEPLOY_EN) && hal_gpio_get(pin) && (wing_lines_on() == 1U);
}

static bool burning(DeploymentElement_t element)
{
    bool line = (element == DEPLOY_ELEMENT_ANTENNA) ? antenna_deploying() : (wing_lines_on() > 0U);
    return hal_gpio_get(GPIO_PIN_DEPLOY_EN) && line;
}

static void assert_no_burn(void)
{
    assert_int_equal(wing_lines_on(), 0U);
    assert_false(hal_gpio_get(GPIO_PIN_DEPLOY_EN));
    assert_false(antenna_deploying());
}

static uint8_t attempts(DeploymentElement_t element)
{
    DeploymentElementStatus_t status;
    assert_true(deployment_get_element_status(element, &status));
    return status.attempt_count;
}

static void reboot(HalResetCause_t cause)
{
    hal_sim_set_reset_cause(cause);
    assert_true(deployment_init());
}

/** Separation, then the full 30-minute wait */
static void wait_out(void)
{
    deployment_tick(SECOND_MS);  /* separation seen */
    advance(DEPLOYMENT_WAIT_TIME_MS, MINUTE_MS);
}

/*===========================================================================*/
/* Basic behaviour (kept from the original suite)                             */
/*===========================================================================*/

static void test_deployment_init(void **state)
{
    (void)state;
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_INHIBITED);
    assert_false(deployment_is_separated());
    assert_false(deployment_all_complete());
    assert_no_burn();
}

static void test_deployment_state_names(void **state)
{
    (void)state;
    for (int i = 0; i < DEPLOY_STATE_COUNT; i++) {
        const char *name = deployment_state_name((DeploymentState_t)i);
        assert_non_null(name);
        assert_true(strlen(name) > 0U);
    }
    assert_non_null(deployment_state_name((DeploymentState_t)99));
}

static void test_deployment_get_element_status_args(void **state)
{
    (void)state;
    DeploymentElementStatus_t status;
    assert_false(deployment_get_element_status(DEPLOY_ELEMENT_ANTENNA, NULL));
    assert_false(deployment_get_element_status(DEPLOY_ELEMENT_COUNT, &status));
    assert_true(deployment_get_element_status(DEPLOY_ELEMENT_ANTENNA, &status));
    assert_false(status.deployed);
    assert_int_equal(status.attempt_count, 0);
}

static void test_deployment_get_telemetry(void **state)
{
    (void)state;
    DeploymentTelemetry_t telemetry;
    assert_false(deployment_get_telemetry(NULL));
    assert_true(deployment_get_telemetry(&telemetry));
    assert_int_equal(telemetry.state, DEPLOY_STATE_INHIBITED);
    assert_int_equal(telemetry.antenna_deployed, 0);
    assert_int_equal(telemetry.solar_deployed, 0);
    assert_int_equal(telemetry.time_since_sep_s, 0);
}

static void test_deployment_constants(void **state)
{
    (void)state;
    assert_int_equal(DEPLOYMENT_WAIT_TIME_MS, 30 * 60 * 1000);
    assert_true(ANTENNA_DEPLOY_BURN_TIME_MS >= 40000);   /* four AntS elements, 10 s each */
    assert_true(ANTENNA_DEPLOY_BURN_TIME_MS <= 60000);
    assert_true(SOLAR_DEPLOY_BURN_TIME_MS >= 1000);
    assert_true(SOLAR_DEPLOY_BURN_TIME_MS <= 10000);
    assert_true(DEPLOYMENT_MAX_RETRIES >= 1);
    assert_true(DEPLOYMENT_MAX_RETRIES <= 10);
}

/** force_retry used to jump INHIBITED -> ANTENNA_PENDING, bypassing the
 *  30-minute wait (CDS 2.4.4). It is now refused before the wait is served */
static void test_force_retry_cannot_bypass_wait(void **state)
{
    (void)state;
    assert_false(deployment_force_retry(DEPLOY_ELEMENT_COUNT));
    assert_false(deployment_force_retry(DEPLOY_ELEMENT_ANTENNA));
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_INHIBITED);
    deployment_tick(SECOND_MS);
    assert_false(deployment_force_retry(DEPLOY_ELEMENT_SOLAR_PANEL));
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_WAITING);
    assert_no_burn();
}

/*===========================================================================*/
/* Sequence                                                                   */
/*===========================================================================*/

/** 30 minutes, then the wings one resistor at a time (wing 1 A, then wing
 *  2 A, 8 s each, never two at once), then the antenna only (the AntS),
 *  each confirmed by its own sense */
static void test_full_sequence(void **state)
{
    (void)state;
    deployment_tick(SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_WAITING);
    assert_true(deployment_is_separated());

    advance(DEPLOYMENT_WAIT_TIME_MS - SECOND_MS, SECOND_MS * 60U);
    assert_no_burn();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);  /* PENDING -> fire */
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
    assert_true(only(GPIO_PIN_BURN_W1A));
    assert_false(antenna_deploying());
    /* The burn began on the first of the two ticks above */
    for (uint32_t t = SECOND_MS; t < SOLAR_DEPLOY_BURN_TIME_MS; t += SECOND_MS) {
        assert_true(only(GPIO_PIN_BURN_W1A));
        deployment_tick(SECOND_MS);
    }
    for (uint32_t t = 0U; t < SOLAR_DEPLOY_BURN_TIME_MS; t += SECOND_MS) {
        assert_true(only(GPIO_PIN_BURN_W2A)); /* wing 2 after wing 1 */
        deployment_tick(SECOND_MS);
    }
    assert_no_burn();
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_SOLAR_VERIFY);

    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false);
    deployment_tick(SECOND_MS);   /* verified */
    deployment_tick(SECOND_MS);   /* the AntS is told to deploy */
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
    assert_false(hal_gpio_get(GPIO_PIN_SOLAR_BURN));

    advance(ANTENNA_DEPLOY_BURN_TIME_MS, SECOND_MS);
    assert_no_burn();             /* cancelled and disarmed */
    set_antenna_sense(true);
    deployment_tick(SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
    assert_true(deployment_all_complete());
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 1);
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 1);
}

/** The wings' second attempt uses the B resistors, the third the A
 *  resistors again; once spent, the wings read failed (EPS-10) until a
 *  ground retry */
static void test_wing_retries_alternate_drivers(void **state)
{
    (void)state;
    const uint32_t attempt = (2U * SOLAR_DEPLOY_BURN_TIME_MS) + DEPLOYMENT_RETRY_DELAY_MS + (2U * SECOND_MS);
    wait_out();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_true(only(GPIO_PIN_BURN_W1A));
    advance(attempt, SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
    assert_true(only(GPIO_PIN_BURN_W1B));
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 2);
    advance(SOLAR_DEPLOY_BURN_TIME_MS, SECOND_MS);
    assert_true(only(GPIO_PIN_BURN_W2B));
    advance(attempt - SOLAR_DEPLOY_BURN_TIME_MS, SECOND_MS);
    assert_true(only(GPIO_PIN_BURN_W1A));
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 3);
    assert_false(deployment_wings_failed()); /* still in the wing phase */
    advance(attempt, SECOND_MS);
    assert_true(deployment_wings_failed());
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false);
    assert_false(deployment_wings_failed()); /* the sense reads deployed */
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, true);
}

/** SOLAR_SENSE (SQSO-ICX-001 §3.1 pin 20): low = both wings deployed, high
 *  stowed (the EPS's 10 kΩ + 1 kΩ pull-up), and the input has no internal
 *  pull: a pull-down would decide the level against the pull-up (the
 *  simulated HAL models that) and read deployed with the wings stowed */
static void test_solar_sense_polarity_and_no_pull(void **state)
{
    (void)state;
    assert_int_equal(hal_sim_gpio_pull(GPIO_PIN_SOLAR_SENSE), GPIO_PULL_NONE);
    wait_out();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_true(only(GPIO_PIN_BURN_W1A));    /* stowed (high): the wings are fired */
    advance(2U * SOLAR_DEPLOY_BURN_TIME_MS, SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_SOLAR_VERIFY);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false); /* both hinges closed: low */
    deployment_tick(SECOND_MS);
    assert_true(deployment_get_state() != DEPLOY_STATE_SOLAR_VERIFY); /* verified */
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 1);
    assert_int_equal(hal_gpio_config(GPIO_PIN_SOLAR_SENSE, GPIO_DIR_INPUT, GPIO_PULL_DOWN), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, true);
    assert_false(hal_gpio_get(GPIO_PIN_SOLAR_SENSE));         /* the pull wins */
    assert_int_equal(hal_gpio_config(GPIO_PIN_SOLAR_SENSE, GPIO_DIR_INPUT, GPIO_PULL_NONE), SMART_QSO_OK);
    assert_true(hal_gpio_get(GPIO_PIN_SOLAR_SENSE));
}

/** An element whose sense already reads deployed is never fired */
static void test_already_deployed_not_fired(void **state)
{
    (void)state;
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false);
    wait_out();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_int_equal(wing_lines_on(), 0U);
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 0);
}

/*===========================================================================*/
/* Timer reset rules (CDS 2.4.4, 2.3.2.4)                                     */
/*===========================================================================*/

static void test_watchdog_reset_resumes_wait(void **state)
{
    (void)state;
    deployment_tick(SECOND_MS);
    advance(20U * MINUTE_MS, MINUTE_MS);
    reboot(HAL_RESET_WATCHDOG);
    assert_no_burn();
    assert_int_equal(deployment_wait_elapsed_s(), 20U * 60U);

    advance(10U * MINUTE_MS, MINUTE_MS);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
}

static void test_power_on_reset_restarts_wait(void **state)
{
    (void)state;
    deployment_tick(SECOND_MS);
    advance(25U * MINUTE_MS, MINUTE_MS);
    reboot(HAL_RESET_POWER_ON);
    assert_int_equal(deployment_wait_elapsed_s(), 0U);

    deployment_tick(SECOND_MS);
    advance(29U * MINUTE_MS, MINUTE_MS);
    deployment_tick(SECOND_MS);
    assert_no_burn();
}

static void test_stowed_switch_restarts_wait(void **state)
{
    (void)state;
    deployment_tick(SECOND_MS);
    advance(25U * MINUTE_MS, MINUTE_MS);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);   /* stowed */
    deployment_tick(SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_INHIBITED);
    assert_int_equal(deployment_wait_elapsed_s(), 0U);

    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, true);
    deployment_tick(SECOND_MS);
    advance(29U * MINUTE_MS, MINUTE_MS);
    deployment_tick(SECOND_MS);
    assert_no_burn();
}

/*===========================================================================*/
/* Burn safety across resets                                                  */
/*===========================================================================*/

/** A reset mid-burn spends the attempt: the pin comes up low and the next
 *  step is verification, not another burn */
static void test_reset_during_burn_spends_attempt(void **state)
{
    (void)state;
    wait_out();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));

    reboot(HAL_RESET_WATCHDOG);
    assert_no_burn();
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 1);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_SOLAR_VERIFY);
}

/** A burn that browns out the bus (power-on reset each time) restarts the
 *  wait, and the persisted attempt count bounds the loop */
static void test_burn_brownout_loop_is_bounded(void **state)
{
    (void)state;
    uint32_t burns = 0U;

    for (uint32_t i = 0U; i < (DEPLOYMENT_MAX_RETRIES + 2U); i++) {
        wait_out();
        for (uint32_t s = 0U; s < (2U * 60U); s++) {
            deployment_tick(SECOND_MS);
            if (burning(DEPLOY_ELEMENT_SOLAR_PANEL)) {
                burns++;
                break;
            }
        }
        reboot(HAL_RESET_BROWNOUT);
        assert_no_burn();
    }

    assert_int_equal(burns, DEPLOYMENT_MAX_RETRIES);
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), DEPLOYMENT_MAX_RETRIES);
}

/** The attempt is persisted before the pin is driven; if it cannot be, no burn */
static void test_persist_failure_blocks_burn(void **state)
{
    (void)state;
    wait_out();                  /* now SOLAR_PENDING */
    hal_sim_flash_fail_writes(true);
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_no_burn();
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 0);

    hal_sim_flash_fail_writes(false);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
}

/*===========================================================================*/
/* Terminal states, arming, enable                                            */
/*===========================================================================*/

static void complete_sequence(void)
{
    set_antenna_sense(true);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false);
    wait_out();
    advance(5U * SECOND_MS, SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
}

/** COMPLETE persists across a power-on reset and ignores the switch: re-burning is not harmless */
static void test_complete_persists_and_ignores_switch(void **state)
{
    (void)state;
    complete_sequence();
    reboot(HAL_RESET_POWER_ON);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);
    advance(40U * MINUTE_MS, MINUTE_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
    assert_no_burn();
}

/** GSE ARM resets a completed unit; ARMED does not progress while powered */
static void test_arm_holds_until_next_boot(void **state)
{
    (void)state;
    complete_sequence();
    set_antenna_sense(false);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, true);

    assert_int_equal(deployment_arm(), SMART_QSO_OK);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ARMED);
    advance(60U * MINUTE_MS, MINUTE_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ARMED);
    assert_no_burn();

    reboot(HAL_RESET_POWER_ON);
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 0);
    assert_false(deployment_all_complete());
    wait_out();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
}

/** A corrupt record starts over (fault logged); sense pins still skip deployed elements */
static void test_corrupt_record_starts_over(void **state)
{
    (void)state;
    uint8_t bytes[4];

    complete_sequence();
    for (uint32_t slot = 0U; slot < 2U; slot++) {
        uint32_t offset = (slot * NVM_RECORD_SLOT_SIZE) + 8U;
        assert_int_equal(hal_flash_read(FLASH_REGION_DEPLOYMENT, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
        for (size_t i = 0U; i < sizeof(bytes); i++) {
            bytes[i] ^= 0xFFU;
        }
        assert_int_equal(hal_flash_write(FLASH_REGION_DEPLOYMENT, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
    }
    reboot(HAL_RESET_WATCHDOG);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_INHIBITED);

    wait_out();
    advance(5U * SECOND_MS, SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 0);
}

/** A ground-disabled sequence holds after the wait, across resets */
static void test_disabled_holds(void **state)
{
    (void)state;
    assert_true(deployment_set_enabled(false));
    wait_out();
    advance(5U * MINUTE_MS, MINUTE_MS);
    assert_no_burn();
    reboot(HAL_RESET_WATCHDOG);
    advance(5U * MINUTE_MS, MINUTE_MS);
    assert_no_burn();

    assert_true(deployment_set_enabled(true));
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
}

/** Nothing senses deployed: the wings' attempts run out, the antenna is
 *  still tried (failed wings must not leave it stowed), the sequence ends
 *  FAILED, and a ground retry (DEPLOY_RETRY) is then allowed */
static void test_force_retry_after_failure(void **state)
{
    (void)state;
    wait_out();
    /* Three attempts each, DEPLOYMENT_RETRY_DELAY_MS apart (LV-07) */
    advance((3U * DEPLOYMENT_RETRY_DELAY_MS) + (2U * MINUTE_MS), SECOND_MS);
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), DEPLOYMENT_MAX_RETRIES);
    assert_true(attempts(DEPLOY_ELEMENT_ANTENNA) >= 1U);
    advance((3U * DEPLOYMENT_RETRY_DELAY_MS) + (2U * MINUTE_MS), SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_FAILED);
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), DEPLOYMENT_MAX_RETRIES);
    assert_true(deployment_force_retry(DEPLOY_ELEMENT_ANTENNA));
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ANTENNA_PENDING);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
}

/** The wings never sense deployed: their retries run out (fault-logged),
 *  the already-deployed antenna is not fired, the sequence ends FAILED */
static void test_solar_failure_reported(void **state)
{
    (void)state;
    FaultLogEntry_t entry;

    set_antenna_sense(true);
    wait_out();
    /* Three attempts DEPLOYMENT_RETRY_DELAY_MS apart (spread over the orbit, LV-07) */
    advance((3U * DEPLOYMENT_RETRY_DELAY_MS) + (2U * MINUTE_MS), SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_FAILED);
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), DEPLOYMENT_MAX_RETRIES);
    assert_no_burn();
    assert_int_equal(fault_log_get_last(&entry), SMART_QSO_OK);
    assert_string_equal(entry.description, "Solar panel deployment failed");
}

/** GSE ARM that cannot be persisted reports the I/O error; the burn lines
 *  stay off */
static void test_arm_not_persisted(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(true);
    assert_int_equal(deployment_arm(), SMART_QSO_ERROR_IO);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ARMED);
    assert_no_burn();
}

/** deployment_save_state() persists the wait count between the periodic
 *  saves, so a watchdog reset resumes from it; it fails if flash does */
static void test_save_state_persists_wait(void **state)
{
    (void)state;
    deployment_tick(SECOND_MS);   /* separation seen */
    advance(30U * SECOND_MS, SECOND_MS);
    uint32_t saved = deployment_wait_elapsed_s();
    assert_true(saved > 0U);
    assert_true(deployment_save_state());

    advance(20U * SECOND_MS, SECOND_MS);   /* not saved: the next periodic save is not due */
    hal_sim_flash_fail_writes(true);
    assert_false(deployment_save_state());
    hal_sim_flash_fail_writes(false);

    reboot(HAL_RESET_WATCHDOG);
    assert_int_equal(deployment_wait_elapsed_s(), saved);
}

/** What calls back into deployment from the flight-log callback got */
static struct {
    int calls;
    SmartQsoResult_t arm;
    bool status;
    bool telemetry;
    bool saved;
    uint8_t attempts_seen;
} s_nested;

/** Flight-log output callback: on the first DEPLOY entry, call back into
 *  the deployment module (it is in the middle of a step) */
static void call_back_into_deployment(const LogEntry_t *entry)
{
    DeploymentElementStatus_t status;
    DeploymentTelemetry_t telemetry;

    if ((strcmp(entry->module, "DEPLOY") != 0) || (s_nested.calls > 0)) {
        return;
    }
    s_nested.calls++;
    memset(&status, 0, sizeof(status));
    status.attempt_count = 7U;
    s_nested.arm = deployment_arm();
    s_nested.status = deployment_get_element_status(DEPLOY_ELEMENT_ANTENNA, &status);
    s_nested.attempts_seen = status.attempt_count;
    s_nested.telemetry = deployment_get_telemetry(&telemetry);
    s_nested.saved = deployment_save_state();
}

/** The flight log's output callback runs while a step is in progress: a
 *  call back into deployment from it is refused (busy) and changes nothing */
static void test_reentry_from_log_callback_refused(void **state)
{
    (void)state;
    memset(&s_nested, 0, sizeof(s_nested));
    assert_int_equal(log_init(), SMART_QSO_OK);
    assert_int_equal(log_register_callback(call_back_into_deployment), SMART_QSO_OK);

    deployment_tick(SECOND_MS);   /* separation: logged in the middle of the step */
    assert_int_equal(log_register_callback(NULL), SMART_QSO_OK);

    assert_int_equal(s_nested.calls, 1);
    assert_int_equal(s_nested.arm, SMART_QSO_ERROR_BUSY);
    assert_false(s_nested.status);
    assert_int_equal(s_nested.attempts_seen, 7U);
    assert_false(s_nested.telemetry);
    assert_false(s_nested.saved);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_WAITING);   /* not ARMED */
    assert_true(deployment_is_separated());
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup_teardown(test_deployment_init, setup, teardown),
        cmocka_unit_test_setup_teardown(test_deployment_state_names, setup, teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_element_status_args, setup, teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_telemetry, setup, teardown),
        cmocka_unit_test_setup_teardown(test_deployment_constants, setup, teardown),
        cmocka_unit_test_setup_teardown(test_force_retry_cannot_bypass_wait, setup, teardown),
        cmocka_unit_test_setup_teardown(test_full_sequence, setup, teardown),
        cmocka_unit_test_setup_teardown(test_wing_retries_alternate_drivers, setup, teardown),
        cmocka_unit_test_setup_teardown(test_solar_sense_polarity_and_no_pull, setup, teardown),
        cmocka_unit_test_setup_teardown(test_already_deployed_not_fired, setup, teardown),
        cmocka_unit_test_setup_teardown(test_watchdog_reset_resumes_wait, setup, teardown),
        cmocka_unit_test_setup_teardown(test_power_on_reset_restarts_wait, setup, teardown),
        cmocka_unit_test_setup_teardown(test_stowed_switch_restarts_wait, setup, teardown),
        cmocka_unit_test_setup_teardown(test_reset_during_burn_spends_attempt, setup, teardown),
        cmocka_unit_test_setup_teardown(test_burn_brownout_loop_is_bounded, setup, teardown),
        cmocka_unit_test_setup_teardown(test_persist_failure_blocks_burn, setup, teardown),
        cmocka_unit_test_setup_teardown(test_complete_persists_and_ignores_switch, setup, teardown),
        cmocka_unit_test_setup_teardown(test_arm_holds_until_next_boot, setup, teardown),
        cmocka_unit_test_setup_teardown(test_corrupt_record_starts_over, setup, teardown),
        cmocka_unit_test_setup_teardown(test_disabled_holds, setup, teardown),
        cmocka_unit_test_setup_teardown(test_force_retry_after_failure, setup, teardown),
        cmocka_unit_test_setup_teardown(test_solar_failure_reported, setup, teardown),
        cmocka_unit_test_setup_teardown(test_arm_not_persisted, setup, teardown),
        cmocka_unit_test_setup_teardown(test_save_state_persists_wait, setup, teardown),
        cmocka_unit_test_setup_teardown(test_reentry_from_log_callback_refused, setup, teardown),
    };

    return cmocka_run_group_tests_name("Deployment Tests", tests, NULL, NULL);
}
