/**
 * @file test_deployment.c
 * @brief Unit tests for SMART-QSO Deployment Control Module
 *
 * Tests deployment sequencing, state machine transitions, and safety features.
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include "deployment.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int test_setup(void **state) {
    (void)state;
    /* Reset deployment module */
    return 0;
}

static int test_teardown(void **state) {
    (void)state;
    return 0;
}

/*===========================================================================*/
/* Test Cases                                                                 */
/*===========================================================================*/

/**
 * @brief Test deployment module initialization
 */
static void test_deployment_init(void **state) {
    (void)state;

    bool result = deployment_init();
    assert_true(result);

    /* Verify initial state */
    DeploymentState_t init_state = deployment_get_state();
    assert_int_equal(init_state, DEPLOY_STATE_INHIBITED);

    /* Verify not separated initially */
    assert_false(deployment_is_separated());

    /* Verify elements not deployed */
    assert_false(deployment_all_complete());
}

/**
 * @brief Test deployment state names
 */
static void test_deployment_state_names(void **state) {
    (void)state;

    /* Verify all state names are valid strings */
    for (int i = 0; i < DEPLOY_STATE_COUNT; i++) {
        const char *name = deployment_state_name((DeploymentState_t)i);
        assert_non_null(name);
        assert_true(strlen(name) > 0);
    }

    /* Verify invalid state returns something sensible */
    const char *invalid_name = deployment_state_name((DeploymentState_t)99);
    assert_non_null(invalid_name);
}

/**
 * @brief Test get element status with NULL pointer
 */
static void test_deployment_get_element_status_null(void **state) {
    (void)state;

    bool result = deployment_get_element_status(DEPLOY_ELEMENT_ANTENNA, NULL);
    assert_false(result);
}

/**
 * @brief Test get element status with invalid element
 */
static void test_deployment_get_element_status_invalid(void **state) {
    (void)state;

    DeploymentElementStatus_t status;
    bool result = deployment_get_element_status(DEPLOY_ELEMENT_COUNT, &status);
    assert_false(result);
}

/**
 * @brief Test get element status for valid element
 */
static void test_deployment_get_element_status_valid(void **state) {
    (void)state;

    deployment_init();

    DeploymentElementStatus_t status;
    bool result = deployment_get_element_status(DEPLOY_ELEMENT_ANTENNA, &status);
    assert_true(result);

    /* Initially should not be deployed */
    assert_false(status.deployed);
    assert_false(status.deploy_attempted);
    assert_int_equal(status.attempt_count, 0);
}

/**
 * @brief Test get telemetry with NULL pointer
 */
static void test_deployment_get_telemetry_null(void **state) {
    (void)state;

    bool result = deployment_get_telemetry(NULL);
    assert_false(result);
}

/**
 * @brief Test get telemetry valid
 */
static void test_deployment_get_telemetry_valid(void **state) {
    (void)state;

    deployment_init();

    DeploymentTelemetry_t telemetry;
    bool result = deployment_get_telemetry(&telemetry);
    assert_true(result);

    /* Verify initial telemetry state */
    assert_int_equal(telemetry.state, DEPLOY_STATE_INHIBITED);
    assert_int_equal(telemetry.antenna_deployed, 0);
    assert_int_equal(telemetry.solar_deployed, 0);
    assert_int_equal(telemetry.time_since_sep_s, 0);
}

/**
 * @brief Test deployment enable/disable
 */
static void test_deployment_set_enabled(void **state) {
    (void)state;

    deployment_init();

    /* Disable deployment */
    bool result = deployment_set_enabled(false);
    assert_true(result);

    /* Re-enable deployment */
    result = deployment_set_enabled(true);
    assert_true(result);
}

/**
 * @brief Test force retry with invalid element
 */
static void test_deployment_force_retry_invalid(void **state) {
    (void)state;

    bool result = deployment_force_retry(DEPLOY_ELEMENT_COUNT);
    assert_false(result);
}

/**
 * @brief Test force retry for antenna
 */
static void test_deployment_force_retry_antenna(void **state) {
    (void)state;

    deployment_init();

    bool result = deployment_force_retry(DEPLOY_ELEMENT_ANTENNA);
    assert_true(result);

    /* State should change to antenna pending */
    DeploymentState_t new_state = deployment_get_state();
    assert_int_equal(new_state, DEPLOY_STATE_ANTENNA_PENDING);
}

/**
 * @brief Test force retry for solar panel
 */
static void test_deployment_force_retry_solar(void **state) {
    (void)state;

    deployment_init();

    bool result = deployment_force_retry(DEPLOY_ELEMENT_SOLAR_PANEL);
    assert_true(result);

    /* State should change to solar pending */
    DeploymentState_t new_state = deployment_get_state();
    assert_int_equal(new_state, DEPLOY_STATE_SOLAR_PENDING);
}

/**
 * @brief Test deployment constants are reasonable
 */
static void test_deployment_constants(void **state) {
    (void)state;

    /* Verify 30 minute wait time */
    assert_int_equal(DEPLOYMENT_WAIT_TIME_MS, 30 * 60 * 1000);

    /* Verify burn times are reasonable (1-10 seconds) */
    assert_true(ANTENNA_DEPLOY_BURN_TIME_MS >= 1000);
    assert_true(ANTENNA_DEPLOY_BURN_TIME_MS <= 10000);
    assert_true(SOLAR_DEPLOY_BURN_TIME_MS >= 1000);
    assert_true(SOLAR_DEPLOY_BURN_TIME_MS <= 10000);

    /* Verify retry count is reasonable */
    assert_true(DEPLOYMENT_MAX_RETRIES >= 1);
    assert_true(DEPLOYMENT_MAX_RETRIES <= 10);
}

/**
 * @brief Test state persistence save
 */
static void test_deployment_save_state(void **state) {
    (void)state;

    deployment_init();

    bool result = deployment_save_state();
    assert_true(result);
}

/**
 * @brief Test all complete returns false when not complete
 */
static void test_deployment_all_complete_false(void **state) {
    (void)state;

    deployment_init();

    /* Initially nothing should be complete */
    assert_false(deployment_all_complete());
}

/*===========================================================================*/
/* Test Runner                                                                */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup_teardown(test_deployment_init, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_state_names, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_element_status_null, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_element_status_invalid, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_element_status_valid, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_telemetry_null, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_get_telemetry_valid, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_set_enabled, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_force_retry_invalid, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_force_retry_antenna, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_force_retry_solar, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_constants, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_save_state, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_deployment_all_complete_false, test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
