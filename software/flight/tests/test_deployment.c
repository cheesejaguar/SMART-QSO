/**
 * @file test_deployment.c
 * @brief Deployment sequence: 30-minute wait, burn wires, persistence, resets
 *
 * Drives the real sequence through the simulation HAL (separation switch,
 * per-element sense and burn pins, persisted record, reset cause).
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
#include "nvm_record.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_reset.h"
#include "hal/hal_sim.h"

#define SECOND_MS   1000U
#define MINUTE_MS   (60U * 1000U)

/*===========================================================================*/
/* Fixtures and helpers                                                       */
/*===========================================================================*/

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    hal_sim_set_reset_cause(HAL_RESET_POWER_ON);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);   /* released */
    hal_sim_gpio_drive_input(GPIO_PIN_ANTENNA_SENSE, false);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false);
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

static bool burning(DeploymentElement_t element)
{
    HalGpioPin_t pin = (element == DEPLOY_ELEMENT_ANTENNA) ? GPIO_PIN_ANTENNA_BURN : GPIO_PIN_SOLAR_BURN;
    return hal_gpio_get(GPIO_PIN_DEPLOY_EN) && hal_gpio_get(pin);
}

static void assert_no_burn(void)
{
    assert_false(hal_gpio_get(GPIO_PIN_ANTENNA_BURN));
    assert_false(hal_gpio_get(GPIO_PIN_SOLAR_BURN));
    assert_false(hal_gpio_get(GPIO_PIN_DEPLOY_EN));
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
    assert_true(ANTENNA_DEPLOY_BURN_TIME_MS >= 1000);
    assert_true(ANTENNA_DEPLOY_BURN_TIME_MS <= 10000);
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

/** 30 minutes, then antenna only, then solar only, each confirmed by its own sense pin */
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
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
    assert_false(hal_gpio_get(GPIO_PIN_SOLAR_BURN));

    advance(ANTENNA_DEPLOY_BURN_TIME_MS, SECOND_MS);
    assert_no_burn();
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ANTENNA_VERIFY);

    hal_sim_gpio_drive_input(GPIO_PIN_ANTENNA_SENSE, true);
    deployment_tick(SECOND_MS);   /* verified */
    deployment_tick(SECOND_MS);   /* solar fires */
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
    assert_false(hal_gpio_get(GPIO_PIN_ANTENNA_BURN));

    advance(SOLAR_DEPLOY_BURN_TIME_MS, SECOND_MS);
    assert_no_burn();
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, true);
    deployment_tick(SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
    assert_true(deployment_all_complete());
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 1);
    assert_int_equal(attempts(DEPLOY_ELEMENT_SOLAR_PANEL), 1);
}

/** An element whose sense already reads deployed is never fired */
static void test_already_deployed_not_fired(void **state)
{
    (void)state;
    hal_sim_gpio_drive_input(GPIO_PIN_ANTENNA_SENSE, true);
    wait_out();
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_false(hal_gpio_get(GPIO_PIN_ANTENNA_BURN));
    assert_true(burning(DEPLOY_ELEMENT_SOLAR_PANEL));
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 0);
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
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
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
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, true);   /* stowed */
    deployment_tick(SECOND_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_INHIBITED);
    assert_int_equal(deployment_wait_elapsed_s(), 0U);

    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);
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
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));

    reboot(HAL_RESET_WATCHDOG);
    assert_no_burn();
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 1);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ANTENNA_VERIFY);
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
            if (burning(DEPLOY_ELEMENT_ANTENNA)) {
                burns++;
                break;
            }
        }
        reboot(HAL_RESET_BROWNOUT);
        assert_no_burn();
    }

    assert_int_equal(burns, DEPLOYMENT_MAX_RETRIES);
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), DEPLOYMENT_MAX_RETRIES);
}

/** The attempt is persisted before the pin is driven; if it cannot be, no burn */
static void test_persist_failure_blocks_burn(void **state)
{
    (void)state;
    wait_out();                  /* now ANTENNA_PENDING */
    hal_sim_flash_fail_writes(true);
    deployment_tick(SECOND_MS);
    deployment_tick(SECOND_MS);
    assert_no_burn();
    assert_int_equal(attempts(DEPLOY_ELEMENT_ANTENNA), 0);

    hal_sim_flash_fail_writes(false);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
}

/*===========================================================================*/
/* Terminal states, arming, enable                                            */
/*===========================================================================*/

static void complete_sequence(void)
{
    hal_sim_gpio_drive_input(GPIO_PIN_ANTENNA_SENSE, true);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, true);
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
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, true);
    advance(40U * MINUTE_MS, MINUTE_MS);
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_COMPLETE);
    assert_no_burn();
}

/** GSE ARM resets a completed unit; ARMED does not progress while powered */
static void test_arm_holds_until_next_boot(void **state)
{
    (void)state;
    complete_sequence();
    hal_sim_gpio_drive_input(GPIO_PIN_ANTENNA_SENSE, false);
    hal_sim_gpio_drive_input(GPIO_PIN_SOLAR_SENSE, false);

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
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
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
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
}

/** After FAILED, a ground retry is allowed (the wait has been served) */
static void test_force_retry_after_failure(void **state)
{
    (void)state;
    wait_out();
    for (uint32_t i = 0U; i < 20U; i++) {
        advance(2U * MINUTE_MS, SECOND_MS);
    }
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_FAILED);
    assert_true(deployment_force_retry(DEPLOY_ELEMENT_ANTENNA));
    assert_int_equal(deployment_get_state(), DEPLOY_STATE_ANTENNA_PENDING);
    deployment_tick(SECOND_MS);
    assert_true(burning(DEPLOY_ELEMENT_ANTENNA));
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
    };

    return cmocka_run_group_tests_name("Deployment Tests", tests, NULL, NULL);
}
