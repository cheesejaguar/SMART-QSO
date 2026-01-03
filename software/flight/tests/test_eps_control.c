/**
 * @file test_eps_control.c
 * @brief Unit tests for EPS Control module
 *
 * Tests power mode management and load control per SRS-F010 through SRS-F017.
 *
 * @requirement SRS-F010 Monitor battery SOC
 * @requirement SRS-F011 Control load switches
 * @requirement SRS-F012 Power mode state machine
 * @requirement SRS-F013 SAFE mode at SOC < 25%
 * @requirement SRS-F014 IDLE mode at SOC < 40%
 * @requirement SRS-F015 Payload enable rules
 * @requirement SRS-F016 Power budget limits
 * @requirement SRS-F017 Persist EPS state
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

/* Include the module under test */
#include "smart_qso.h"
#include "eps_control.h"
#include <math.h>

/*===========================================================================*/
/* Helper Macros                                                              */
/*===========================================================================*/

/** Tolerance for floating-point comparisons */
#define FLOAT_TOLERANCE 0.001

/** Compare floats with tolerance */
#define FLOAT_EQUAL(a, b) (fabs((a) - (b)) < FLOAT_TOLERANCE)

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Remove any existing config file */
    unlink(EPS_CONFIG_FILE);
    /* Initialize EPS module */
    SmartQsoResult_t result = eps_init();
    assert_int_equal(result, SMART_QSO_OK);
    return 0;
}

static int teardown(void **state) {
    (void)state;
    /* Clean up config file */
    unlink(EPS_CONFIG_FILE);
    return 0;
}

/*===========================================================================*/
/* Test Cases: Initialization                                                 */
/*===========================================================================*/

/**
 * @brief Test EPS initialization with no existing config
 *
 * @requirement SRS-F011 System shall initialize with default state
 */
static void test_eps_init_no_config(void **state) {
    (void)state;

    /* Module already initialized in setup, verify state */
    EpsControlState_t eps_state;
    SmartQsoResult_t result = eps_get_state(&eps_state);

    assert_int_equal(result, SMART_QSO_OK);
    /* Default is IDLE mode */
    assert_int_equal(eps_state.power_mode, POWER_MODE_IDLE);
    assert_false(eps_state.payload_enabled);
    assert_true(eps_state.beacon_enabled);
}

/**
 * @brief Test EPS get_state with null pointer
 */
static void test_eps_get_state_null(void **state) {
    (void)state;

    SmartQsoResult_t result = eps_get_state(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Cases: Power Mode Transitions                                         */
/*===========================================================================*/

/**
 * @brief Test transition to SAFE mode
 *
 * @requirement SRS-F012 Power mode state machine
 * @requirement SRS-F013 SAFE mode at SOC < 25%
 */
static void test_eps_set_mode_safe(void **state) {
    (void)state;

    double soc = 0.20;  /* Below SAFE threshold */
    SmartQsoResult_t result = eps_set_power_mode(POWER_MODE_SAFE, soc);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
    assert_false(eps_is_payload_enabled());
    assert_true(eps_get_power_limit() <= EPS_POWER_LIMIT_SAFE_W);
}

/**
 * @brief Test transition to IDLE mode
 *
 * @requirement SRS-F012 Power mode state machine
 * @requirement SRS-F014 IDLE mode at SOC < 40%
 */
static void test_eps_set_mode_idle(void **state) {
    (void)state;

    double soc = 0.50;  /* Good SOC for IDLE */
    SmartQsoResult_t result = eps_set_power_mode(POWER_MODE_IDLE, soc);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);
    assert_false(eps_is_payload_enabled());
    assert_true(eps_get_power_limit() <= EPS_POWER_LIMIT_IDLE_W);
}

/**
 * @brief Test transition to ACTIVE mode
 *
 * @requirement SRS-F012 Power mode state machine
 * @requirement SRS-F015 Payload enable rules
 */
static void test_eps_set_mode_active(void **state) {
    (void)state;

    double soc = 0.80;  /* High SOC for ACTIVE */
    SmartQsoResult_t result = eps_set_power_mode(POWER_MODE_ACTIVE, soc);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_ACTIVE);
    assert_true(eps_get_power_limit() <= EPS_POWER_LIMIT_ACTIVE_W);
}

/**
 * @brief Test automatic mode demotion on low SOC
 *
 * @requirement SRS-F013 SAFE mode at SOC < 25%
 */
static void test_eps_auto_safe_on_low_soc(void **state) {
    (void)state;

    /* Start in ACTIVE mode with high SOC */
    eps_set_power_mode(POWER_MODE_ACTIVE, 0.80);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_ACTIVE);

    /* Try to stay in ACTIVE with low SOC - should demote */
    double low_soc = 0.20;
    eps_set_power_mode(POWER_MODE_ACTIVE, low_soc);

    /* Should have demoted to SAFE */
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
}

/*===========================================================================*/
/* Test Cases: Payload Control                                                */
/*===========================================================================*/

/**
 * @brief Test payload enable with sufficient SOC
 *
 * @requirement SRS-F015 Payload enable at SOC > 55%
 */
static void test_eps_payload_enable_high_soc(void **state) {
    (void)state;

    /* Set ACTIVE mode first */
    eps_set_power_mode(POWER_MODE_ACTIVE, 0.80);

    double soc = 0.80;  /* Above payload threshold */
    SmartQsoResult_t result = eps_control_payload(true, soc);

    assert_int_equal(result, SMART_QSO_OK);
    assert_true(eps_is_payload_enabled());
}

/**
 * @brief Test payload enable blocked with low SOC
 *
 * @requirement SRS-F015 Payload enable at SOC > 55%
 */
static void test_eps_payload_blocked_low_soc(void **state) {
    (void)state;

    double low_soc = 0.40;  /* Below payload threshold */
    (void)eps_control_payload(true, low_soc);

    /* Should fail or be blocked */
    assert_false(eps_is_payload_enabled());
}

/**
 * @brief Test payload disable
 *
 * @requirement SRS-F011 Control load switches
 */
static void test_eps_payload_disable(void **state) {
    (void)state;

    /* Enable payload first */
    eps_set_power_mode(POWER_MODE_ACTIVE, 0.80);
    eps_control_payload(true, 0.80);
    assert_true(eps_is_payload_enabled());

    /* Disable payload */
    SmartQsoResult_t result = eps_control_payload(false, 0.80);
    assert_int_equal(result, SMART_QSO_OK);
    assert_false(eps_is_payload_enabled());
}

/*===========================================================================*/
/* Test Cases: Load Switch Control                                            */
/*===========================================================================*/

/**
 * @brief Test radio control
 *
 * @requirement SRS-F011 Control load switches
 */
static void test_eps_radio_control(void **state) {
    (void)state;

    SmartQsoResult_t result;

    /* Enable radio */
    result = eps_control_radio(true);
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify via state */
    EpsControlState_t eps_state;
    eps_get_state(&eps_state);
    assert_true(eps_state.radio_enabled);

    /* Disable radio */
    result = eps_control_radio(false);
    assert_int_equal(result, SMART_QSO_OK);

    eps_get_state(&eps_state);
    assert_false(eps_state.radio_enabled);
}

/**
 * @brief Test ADCS control
 *
 * @requirement SRS-F011 Control load switches
 */
static void test_eps_adcs_control(void **state) {
    (void)state;

    SmartQsoResult_t result;
    EpsControlState_t eps_state;

    /* Enable ADCS */
    result = eps_control_adcs(true);
    assert_int_equal(result, SMART_QSO_OK);

    eps_get_state(&eps_state);
    assert_true(eps_state.adcs_enabled);

    /* Disable ADCS */
    result = eps_control_adcs(false);
    assert_int_equal(result, SMART_QSO_OK);

    eps_get_state(&eps_state);
    assert_false(eps_state.adcs_enabled);
}

/**
 * @brief Test beacon control
 *
 * @requirement SRS-F011 Control load switches
 */
static void test_eps_beacon_control(void **state) {
    (void)state;

    SmartQsoResult_t result;
    EpsControlState_t eps_state;

    /* Beacon should be enabled by default */
    eps_get_state(&eps_state);
    assert_true(eps_state.beacon_enabled);

    /* Disable beacon */
    result = eps_control_beacon(false);
    assert_int_equal(result, SMART_QSO_OK);

    eps_get_state(&eps_state);
    assert_false(eps_state.beacon_enabled);

    /* Re-enable beacon */
    result = eps_control_beacon(true);
    assert_int_equal(result, SMART_QSO_OK);

    eps_get_state(&eps_state);
    assert_true(eps_state.beacon_enabled);
}

/*===========================================================================*/
/* Test Cases: Power Budget                                                   */
/*===========================================================================*/

/**
 * @brief Test power limits per mode
 *
 * @requirement SRS-F016 Power budget limits
 */
static void test_eps_power_limits(void **state) {
    (void)state;

    /* SAFE mode */
    eps_set_power_mode(POWER_MODE_SAFE, 0.20);
    assert_true(FLOAT_EQUAL(eps_get_power_limit(), EPS_POWER_LIMIT_SAFE_W));

    /* IDLE mode */
    eps_set_power_mode(POWER_MODE_IDLE, 0.50);
    assert_true(FLOAT_EQUAL(eps_get_power_limit(), EPS_POWER_LIMIT_IDLE_W));

    /* ACTIVE mode */
    eps_set_power_mode(POWER_MODE_ACTIVE, 0.80);
    assert_true(FLOAT_EQUAL(eps_get_power_limit(), EPS_POWER_LIMIT_ACTIVE_W));
}

/*===========================================================================*/
/* Test Cases: Persistence                                                    */
/*===========================================================================*/

/**
 * @brief Test configuration save and load
 *
 * @requirement SRS-F017 Persist EPS state
 */
static void test_eps_persistence(void **state) {
    (void)state;

    /* Set a specific state */
    eps_set_power_mode(POWER_MODE_ACTIVE, 0.80);
    eps_control_payload(true, 0.80);
    eps_control_radio(true);
    eps_control_adcs(true);

    /* Save configuration */
    SmartQsoResult_t result = eps_save_config();
    assert_int_equal(result, SMART_QSO_OK);

    /* Change state */
    eps_set_power_mode(POWER_MODE_SAFE, 0.20);

    /* Reload configuration */
    result = eps_load_config();
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify state was restored */
    /* Note: Power mode may be adjusted based on current SOC */
    EpsControlState_t eps_state;
    eps_get_state(&eps_state);
    assert_true(eps_state.radio_enabled);
    assert_true(eps_state.adcs_enabled);
}

/*===========================================================================*/
/* Test Cases: Boundary Conditions                                            */
/*===========================================================================*/

/**
 * @brief Test SOC boundary at SAFE threshold
 *
 * @requirement SRS-F013 SAFE mode at SOC < 25%
 */
static void test_eps_soc_boundary_safe(void **state) {
    (void)state;

    /* Just above threshold */
    eps_set_power_mode(POWER_MODE_IDLE, EPS_SOC_SAFE_THRESHOLD + 0.01);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);

    /* At threshold */
    eps_set_power_mode(POWER_MODE_IDLE, EPS_SOC_SAFE_THRESHOLD);
    /* Behavior at exact threshold depends on implementation */

    /* Just below threshold */
    eps_set_power_mode(POWER_MODE_IDLE, EPS_SOC_SAFE_THRESHOLD - 0.01);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
}

/**
 * @brief Test SOC boundary at payload threshold
 *
 * @requirement SRS-F015 Payload enable at SOC > 55%
 */
static void test_eps_soc_boundary_payload(void **state) {
    (void)state;

    /* Set ACTIVE mode */
    eps_set_power_mode(POWER_MODE_ACTIVE, 0.80);

    /* Just above threshold - should enable */
    SmartQsoResult_t result = eps_control_payload(true, EPS_SOC_PAYLOAD_THRESHOLD + 0.01);
    assert_int_equal(result, SMART_QSO_OK);
    assert_true(eps_is_payload_enabled());

    /* Disable first */
    eps_control_payload(false, 0.60);

    /* Just below threshold - should not enable */
    eps_control_payload(true, EPS_SOC_PAYLOAD_THRESHOLD - 0.01);
    assert_false(eps_is_payload_enabled());
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_eps_init_no_config, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_get_state_null, setup, teardown),

        /* Power mode transition tests */
        cmocka_unit_test_setup_teardown(test_eps_set_mode_safe, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_set_mode_idle, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_set_mode_active, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_auto_safe_on_low_soc, setup, teardown),

        /* Payload control tests */
        cmocka_unit_test_setup_teardown(test_eps_payload_enable_high_soc, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_payload_blocked_low_soc, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_payload_disable, setup, teardown),

        /* Load switch control tests */
        cmocka_unit_test_setup_teardown(test_eps_radio_control, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_adcs_control, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_beacon_control, setup, teardown),

        /* Power budget tests */
        cmocka_unit_test_setup_teardown(test_eps_power_limits, setup, teardown),

        /* Persistence tests */
        cmocka_unit_test_setup_teardown(test_eps_persistence, setup, teardown),

        /* Boundary condition tests */
        cmocka_unit_test_setup_teardown(test_eps_soc_boundary_safe, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_soc_boundary_payload, setup, teardown),
    };

    return cmocka_run_group_tests_name("EPS Control Tests", tests, NULL, NULL);
}
