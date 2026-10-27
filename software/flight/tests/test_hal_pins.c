/**
 * @file test_hal_pins.c
 * @brief The pin allocation of SQSO-ICX-001 B §6 in the HAL headers, and
 *        the C/Rust contract of the round-3 additions (ids 450-499 of
 *        rust_contract.h)
 *
 * Expectations from hal_gpio.h, hal_adc.h, hal_spi.h, hal_i2c.h,
 * hal_flash.h, hal_capture.h, hal_mtq.h and passivation.h:
 *
 * - every logical pin, ADC channel, bus and region the flight software uses
 *   has the value the Rust library assumes;
 * - at reset every output pin of the simulated HAL reads low (no enable,
 *   burn or passivation line high before the software drives it);
 * - the new outputs can be driven and the new inputs read.
 *
 * @requirement SRS-F140 Sensor and actuator drivers on the OBC board's pin allocation
 * @requirement SRS-F155 The final passivation report: three frames over three minutes carrying EOL_STATE, the fuse state and the attempts, before TX-OFF
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>

#include "passivation.h"
#include "rust_contract.h"
#include "hal/hal.h"
#include "hal/hal_gpio.h"
#include "hal/hal_adc.h"
#include "hal/hal_spi.h"
#include "hal/hal_i2c.h"
#include "hal/hal_flash.h"
#include "hal/hal_capture.h"
#include "hal/hal_mtq.h"
#include "hal/hal_sim.h"

static void test_contract_450(void **state)
{
    (void)state;
    static const struct {
        int id;
        double value;
    } k[] = {
        {RUST_CONST_PIN_GYRO_EN, GPIO_PIN_GYRO_EN},
        {RUST_CONST_PIN_IMU2_EN, GPIO_PIN_IMU2_EN},
        {RUST_CONST_PIN_SNS_EN, GPIO_PIN_SNS_EN},
        {RUST_CONST_PIN_GYRO_RB, GPIO_PIN_GYRO_RB},
        {RUST_CONST_PIN_IMU2_RB, GPIO_PIN_IMU2_RB},
        {RUST_CONST_PIN_SNS_RB, GPIO_PIN_SNS_RB},
        {RUST_CONST_PIN_GYRO_RST_N, GPIO_PIN_GYRO_RST_N},
        {RUST_CONST_PIN_GYRO_DRDY, GPIO_PIN_GYRO_DRDY},
        {RUST_CONST_PIN_IMU2_DRDY, GPIO_PIN_IMU2_DRDY},
        {RUST_CONST_PIN_MAG_DRDY, GPIO_PIN_MAG_DRDY},
        {RUST_CONST_PIN_MTQ_X_DIR, GPIO_PIN_MTQ_X_DIR},
        {RUST_CONST_PIN_MTQ_Y_DIR, GPIO_PIN_MTQ_Y_DIR},
        {RUST_CONST_PIN_MTQ_Z_DIR, GPIO_PIN_MTQ_Z_DIR},
        {RUST_CONST_PIN_MTQ_FAULT_N, GPIO_PIN_MTQ_FAULT_N},
        {RUST_CONST_PIN_MTQ_EXPIRED, GPIO_PIN_MTQ_EXPIRED},
        {RUST_CONST_PIN_TONE_OK, GPIO_PIN_TONE_OK},
        {RUST_CONST_PIN_XPDR_KEY, GPIO_PIN_XPDR_KEY},
        {RUST_CONST_PIN_PTT_EXC, GPIO_PIN_PTT_EXC},
        {RUST_CONST_PIN_JET_PG, GPIO_PIN_JET_PG},
        {RUST_CONST_PIN_GNSS_PG, GPIO_PIN_GNSS_PG},
        {RUST_CONST_PIN_WDI, GPIO_PIN_WDI},
        {RUST_CONST_ADC_VSW1_RB, ADC_CHANNEL_VSW1_RB},
        {RUST_CONST_ADC_PT_NTC, ADC_CHANNEL_PT_NTC},
        {RUST_CONST_ADC_MTQ_X_I, ADC_CHANNEL_MTQ_X_I},
        {RUST_CONST_ADC_MTQ_Y_I, ADC_CHANNEL_MTQ_Y_I},
        {RUST_CONST_ADC_MTQ_Z_I, ADC_CHANNEL_MTQ_Z_I},
        {RUST_CONST_ADC_VJET_RB, ADC_CHANNEL_VJET_RB},
        {RUST_CONST_ADC_CHANNEL_COUNT, ADC_CHANNEL_COUNT},
        {RUST_CONST_SPI_BUS_COUNT, SPI_BUS_COUNT},
        {RUST_CONST_I2C_BUS_COUNT, I2C_BUS_COUNT},
        {RUST_CONST_FLASH_REGION_ADCS_CAL, FLASH_REGION_ADCS_CAL},
        {RUST_CONST_CAPTURE_GNSS_PPS, CAPTURE_GNSS_PPS},
        {RUST_CONST_CAPTURE_RTC_1HZ, CAPTURE_RTC_1HZ},
        {RUST_CONST_CAPTURE_PTT_EXC, CAPTURE_PTT_EXC},
        {RUST_CONST_MTQ_PWM_HZ, HAL_MTQ_PWM_HZ},
        {RUST_CONST_MTQ_MAX_BURST_MS, HAL_MTQ_MAX_BURST_MS},
        {RUST_CONST_MTQ_DUTY_FULL, HAL_MTQ_DUTY_FULL},
        {RUST_CONST_I2C_ADDR_BATTERY_MONITOR, I2C_ADDR_BATTERY_MONITOR},
        {RUST_CONST_I2C_ADDR_SOLAR_MONITOR, I2C_ADDR_SOLAR_MONITOR},
        {RUST_CONST_I2C_ADDR_SUN_ADC, I2C_ADDR_SUN_ADC},
        {RUST_CONST_I2C_ADDR_MAGNETOMETER, I2C_ADDR_MAGNETOMETER},
        {RUST_CONST_I2C_ADDR_RTC, I2C_ADDR_RTC},
        {RUST_CONST_PASSIVATION_REPORTING, PASSIVATION_REPORTING},
        {RUST_CONST_PASSIVATION_FINAL_REPORTS, PASSIVATION_FINAL_REPORTS},
        {RUST_CONST_PASSIVATION_REPORT_INTERVAL_MS, PASSIVATION_REPORT_INTERVAL_MS},
        {RUST_CONST_PASSIVATION_REPORT_WINDOW_MS, PASSIVATION_REPORT_WINDOW_MS},
        {RUST_CONST_PASSIVATION_MAX_FIRE_ATTEMPTS, PASSIVATION_MAX_FIRE_ATTEMPTS},
        {RUST_CONST_PASSIVATION_ARM_CONFIRM_MS, PASSIVATION_ARM_CONFIRM_MS},
        {RUST_CONST_PASSIVATION_FIRE_DEADLINE_MS, (double)PASSIVATION_FIRE_DEADLINE_MS},
    };
    for (size_t i = 0U; i < sizeof(k) / sizeof(k[0]); i++) {
        assert_true(fabs(sqso_rust_constant((RustConst_t)k[i].id) - k[i].value) < 0.5);
    }
    assert_int_equal(sizeof(k) / sizeof(k[0]), (size_t)(RUST_CONST_GROUP_450_END - 450));
    assert_true(isnan(sqso_rust_constant((RustConst_t)RUST_CONST_GROUP_450_END)));
}

/** The enable, burn and passivation lines are low at reset */
static void test_outputs_low_at_reset(void **state)
{
    (void)state;
    static const HalGpioPin_t outputs[] = {
        GPIO_PIN_PAYLOAD_EN, GPIO_PIN_RADIO_EN,  GPIO_PIN_ADCS_EN,   GPIO_PIN_HEATER_EN, GPIO_PIN_DEPLOY_EN,
        GPIO_PIN_JETSON_PWR, GPIO_PIN_RF_TX_EN,  GPIO_PIN_BURN_W2A,  GPIO_PIN_BURN_W1A,  GPIO_PIN_TX_PWR_SEL,
        GPIO_PIN_XPDR_EN,    GPIO_PIN_GNSS_EN,   GPIO_PIN_BURN_W1B,  GPIO_PIN_EOL_ARM,   GPIO_PIN_EOL_FIRE,
        GPIO_PIN_BURN_W2B,   GPIO_PIN_LOCK_CLR,  GPIO_PIN_GYRO_EN,   GPIO_PIN_IMU2_EN,   GPIO_PIN_SNS_EN,
        GPIO_PIN_GYRO_RST_N, GPIO_PIN_MTQ_X_DIR, GPIO_PIN_MTQ_Y_DIR, GPIO_PIN_MTQ_Z_DIR, GPIO_PIN_WDI,
    };
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    for (size_t i = 0U; i < sizeof(outputs) / sizeof(outputs[0]); i++) {
        assert_false(hal_gpio_get(outputs[i]));
    }
    assert_int_equal(GPIO_PIN_COUNT, 46);
    assert_int_equal(GPIO_PIN_PTT_EXCITER, GPIO_PIN_PTT_EXC);
}

/** The new lines work through the HAL: outputs driven, inputs read */
static void test_new_lines(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_gpio_config(GPIO_PIN_GYRO_EN, GPIO_DIR_OUTPUT, GPIO_PULL_NONE), SMART_QSO_OK);
    assert_int_equal(hal_gpio_set(GPIO_PIN_GYRO_EN, true), SMART_QSO_OK);
    assert_true(hal_gpio_get(GPIO_PIN_GYRO_EN));
    assert_int_equal(hal_gpio_set(GPIO_PIN_GYRO_EN, false), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_TONE_OK, true);
    assert_true(hal_gpio_get(GPIO_PIN_TONE_OK));
    hal_sim_gpio_drive_input(GPIO_PIN_TONE_OK, false);
    assert_false(hal_gpio_get(GPIO_PIN_TONE_OK));
    assert_int_equal(hal_gpio_set((HalGpioPin_t)GPIO_PIN_COUNT, true), SMART_QSO_ERROR_INVALID);

    double v = 0.0;
    assert_int_equal(hal_adc_init(ADC_RESOLUTION_12BIT, ADC_REF_INTERNAL), SMART_QSO_OK);
    assert_int_equal(hal_adc_read_voltage(ADC_CHANNEL_PT_NTC, &v), SMART_QSO_OK);
    assert_true(v > 0.10 && v < 3.25); /* the sensor's valid range */
    assert_int_equal(hal_adc_read_voltage((HalAdcChannel_t)ADC_CHANNEL_COUNT, &v), SMART_QSO_ERROR_INVALID);

    const uint16_t duty[3] = {500U, 0U, HAL_MTQ_DUTY_FULL};
    const uint16_t over[3] = {HAL_MTQ_DUTY_FULL + 1U, 0U, 0U};
    assert_int_equal(hal_mtq_burst(duty, HAL_MTQ_MAX_BURST_MS + 1U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(hal_mtq_burst(over, 100U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(hal_mtq_burst(duty, HAL_MTQ_MAX_BURST_MS), SMART_QSO_OK);
    assert_true(hal_mtq_active());
    assert_int_equal(hal_mtq_off(), SMART_QSO_OK);
    assert_false(hal_mtq_active());
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_contract_450),
        cmocka_unit_test(test_outputs_low_at_reset),
        cmocka_unit_test(test_new_lines),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
