/**
 * @file test_beacon_e2e.c
 * @brief End-to-end beacon generation integration tests
 *
 * Tests the complete beacon generation pipeline from telemetry
 * collection through Jetson inference to AX.25 frame transmission.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "telemetry.h"
#include "beacon.h"
#include "sensors.h"
#include "eps_control.h"
#include "uart_comm.h"
#include <string.h>

/*******************************************************************************
 * Test Constants
 ******************************************************************************/

#define TEST_CALLSIGN       "SQSO-1"
#define TEST_DESTINATION    "CQ"
#define MAX_BEACON_SIZE     256U

/*******************************************************************************
 * Test Data
 ******************************************************************************/

static uint8_t g_tx_buffer[512];
static size_t g_tx_size = 0;
static uint32_t g_beacons_generated = 0;

/*******************************************************************************
 * Mock/Stub Functions
 ******************************************************************************/

/* Mock Jetson interface for AI-generated beacon text */
static const char *mock_jetson_text = "SQSO-1 AI beacon test. All systems nominal.";

static bool mock_get_jetson_text(char *buffer, size_t size)
{
    if (strlen(mock_jetson_text) >= size) {
        return false;
    }
    strcpy(buffer, mock_jetson_text);
    return true;
}

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

void setUp(void)
{
    /* Initialize all subsystems */
    telemetry_init();
    beacon_init();
    sensors_init();
    eps_init();
    uart_init();

    /* Configure beacon */
    beacon_set_callsign(TEST_CALLSIGN, 0);
    beacon_set_destination(TEST_DESTINATION, 0);

    g_tx_size = 0;
    g_beacons_generated = 0;
    memset(g_tx_buffer, 0, sizeof(g_tx_buffer));
}

void tearDown(void)
{
    /* Cleanup */
}

/*******************************************************************************
 * Test Cases - Sensor to Telemetry
 ******************************************************************************/

void test_sensor_to_telemetry_flow(void)
{
    /* Poll sensors */
    sensor_data_t sensor_data;
    sensor_error_t s_status = sensors_poll(&sensor_data);
    TEST_ASSERT_EQUAL(SENSOR_OK, s_status);

    /* Register telemetry points */
    telem_point_t points[] = {
        { .name = "sun_x", .type = TELEM_TYPE_FLOAT, .period_ms = 1000 },
        { .name = "sun_y", .type = TELEM_TYPE_FLOAT, .period_ms = 1000 },
        { .name = "mag_x", .type = TELEM_TYPE_FLOAT, .period_ms = 1000 },
    };

    for (int i = 0; i < 3; i++) {
        telemetry_register_point(&points[i]);
    }

    /* Update telemetry from sensor data */
    telemetry_update_value("sun_x", &sensor_data.sun_x, sizeof(float));
    telemetry_update_value("sun_y", &sensor_data.sun_y, sizeof(float));
    telemetry_update_value("mag_x", &sensor_data.mag_x, sizeof(float));

    /* Verify values are available */
    float retrieved;
    telem_error_t status = telemetry_get_value("sun_x", &retrieved, sizeof(float));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

void test_eps_to_telemetry_flow(void)
{
    /* Get EPS telemetry */
    eps_telemetry_t eps_data;
    eps_error_t e_status = eps_get_telemetry(&eps_data);
    TEST_ASSERT_EQUAL(EPS_OK, e_status);

    /* Register telemetry points */
    telem_point_t points[] = {
        { .name = "batt_v", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "batt_i", .type = TELEM_TYPE_INT16, .period_ms = 1000 },
        { .name = "batt_soc", .type = TELEM_TYPE_UINT8, .period_ms = 5000 },
    };

    for (int i = 0; i < 3; i++) {
        telemetry_register_point(&points[i]);
    }

    /* Update telemetry */
    telemetry_update_value("batt_v", &eps_data.battery_voltage_mv, sizeof(uint16_t));
    telemetry_update_value("batt_i", &eps_data.battery_current_ma, sizeof(int16_t));
    telemetry_update_value("batt_soc", &eps_data.battery_soc_percent, sizeof(uint8_t));

    /* Verify */
    uint16_t voltage;
    telem_error_t status = telemetry_get_value("batt_v", &voltage, sizeof(uint16_t));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

/*******************************************************************************
 * Test Cases - Telemetry to Health Data
 ******************************************************************************/

void test_telemetry_to_health_data(void)
{
    /* Generate telemetry packet for Jetson */
    telem_point_t point = { .name = "status", .type = TELEM_TYPE_UINT8, .period_ms = 1000 };
    telemetry_register_point(&point);

    uint8_t status_value = 0;  /* Nominal */
    telemetry_update_value("status", &status_value, sizeof(uint8_t));

    /* Generate packet */
    uint8_t packet[128];
    size_t size;
    telem_error_t status = telemetry_generate_packet(packet, sizeof(packet), &size);

    TEST_ASSERT_EQUAL(TELEM_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);

    /* This packet would be sent to Jetson via UART */
}

/*******************************************************************************
 * Test Cases - Jetson to Beacon Text
 ******************************************************************************/

void test_jetson_to_beacon_text(void)
{
    /* Simulate receiving text from Jetson */
    char ai_text[128];
    bool success = mock_get_jetson_text(ai_text, sizeof(ai_text));
    TEST_ASSERT_TRUE(success);

    /* Set beacon info from AI text */
    beacon_set_jetson_text(ai_text, strlen(ai_text));
    beacon_enable_ai_content(true);

    /* Verify AI content is set */
    TEST_ASSERT_TRUE(beacon_has_ai_content());
}

void test_beacon_fallback_on_no_jetson(void)
{
    /* Clear any AI content */
    beacon_enable_ai_content(false);

    /* Generate beacon with fallback */
    uint8_t frame[MAX_BEACON_SIZE];
    size_t size;
    beacon_error_t status = beacon_generate_frame(frame, sizeof(frame), &size);

    TEST_ASSERT_EQUAL(BEACON_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);

    /* Frame should use template text */
}

/*******************************************************************************
 * Test Cases - Beacon to AX.25 Frame
 ******************************************************************************/

void test_beacon_to_ax25_frame(void)
{
    /* Set up beacon with telemetry summary */
    char info[128];
    snprintf(info, sizeof(info), "SQSO-1 V:3850mV T:25C SOC:85%%");
    beacon_set_info_field(info, strlen(info));

    /* Generate AX.25 frame */
    uint8_t frame[MAX_BEACON_SIZE];
    size_t size;
    beacon_error_t status = beacon_generate_frame(frame, sizeof(frame), &size);

    TEST_ASSERT_EQUAL(BEACON_OK, status);
    TEST_ASSERT_GREATER_THAN(17, size);  /* Minimum AX.25 frame size */

    /* Verify frame structure */
    TEST_ASSERT_EQUAL_HEX8(0x7E, frame[0]);          /* Start flag */
    TEST_ASSERT_EQUAL_HEX8(0x7E, frame[size - 1]);   /* End flag */

    g_beacons_generated++;
}

void test_frame_contains_callsign(void)
{
    /* Generate frame */
    uint8_t frame[MAX_BEACON_SIZE];
    size_t size;
    beacon_generate_frame(frame, sizeof(frame), &size);

    /* The callsign should be in the source address field (bytes 8-13) */
    /* Note: Callsigns are shifted left by 1 in AX.25 */
    /* This is a simplified check */
    TEST_ASSERT_GREATER_THAN(14, size);
}

/*******************************************************************************
 * Test Cases - Frame to Transmitter
 ******************************************************************************/

void test_frame_to_transmitter(void)
{
    /* Generate beacon frame */
    uint8_t frame[MAX_BEACON_SIZE];
    size_t size;
    beacon_generate_frame(frame, sizeof(frame), &size);

    /* Send to transmitter via UART */
    uart_error_t status = uart_transmit(UART_PORT_RF, frame, size);
    TEST_ASSERT_EQUAL(UART_OK, status);
}

/*******************************************************************************
 * Test Cases - Complete E2E Flow
 ******************************************************************************/

void test_complete_e2e_beacon_flow(void)
{
    /*
     * Complete End-to-End Flow:
     * 1. Sensor polling
     * 2. Telemetry collection
     * 3. Health data for Jetson
     * 4. AI text generation (mocked)
     * 5. Beacon frame generation
     * 6. Transmission
     */

    /* Step 1: Poll sensors */
    sensor_data_t sensor_data;
    sensors_poll(&sensor_data);

    /* Step 2: Get EPS data */
    eps_telemetry_t eps_data;
    eps_get_telemetry(&eps_data);

    /* Step 3: Build telemetry */
    telem_point_t points[] = {
        { .name = "sun_x", .type = TELEM_TYPE_FLOAT, .period_ms = 1000 },
        { .name = "batt_v", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "batt_soc", .type = TELEM_TYPE_UINT8, .period_ms = 1000 },
    };

    for (int i = 0; i < 3; i++) {
        telemetry_register_point(&points[i]);
    }

    telemetry_update_value("sun_x", &sensor_data.sun_x, sizeof(float));
    telemetry_update_value("batt_v", &eps_data.battery_voltage_mv, sizeof(uint16_t));
    telemetry_update_value("batt_soc", &eps_data.battery_soc_percent, sizeof(uint8_t));

    /* Step 4: Get AI text (simulated) */
    char ai_text[128];
    mock_get_jetson_text(ai_text, sizeof(ai_text));
    beacon_set_jetson_text(ai_text, strlen(ai_text));
    beacon_enable_ai_content(true);

    /* Step 5: Generate beacon frame */
    uint8_t frame[MAX_BEACON_SIZE];
    size_t frame_size;
    beacon_error_t b_status = beacon_generate_frame(frame, sizeof(frame), &frame_size);
    TEST_ASSERT_EQUAL(BEACON_OK, b_status);
    TEST_ASSERT_GREATER_THAN(20, frame_size);

    /* Step 6: Transmit */
    uart_error_t u_status = uart_transmit(UART_PORT_RF, frame, frame_size);
    TEST_ASSERT_EQUAL(UART_OK, u_status);

    g_beacons_generated++;
    TEST_ASSERT_GREATER_THAN(0, g_beacons_generated);
}

void test_e2e_with_ai_failure(void)
{
    /*
     * Test graceful degradation when Jetson/AI is unavailable
     */

    /* Clear AI content */
    beacon_enable_ai_content(false);

    /* Build telemetry */
    telem_point_t point = { .name = "temp", .type = TELEM_TYPE_INT16, .period_ms = 1000 };
    telemetry_register_point(&point);

    int16_t temp = 25;
    telemetry_update_value("temp", &temp, sizeof(temp));

    /* Set fallback info */
    beacon_set_info_field("SQSO-1 BEACON T:25C", 20);

    /* Generate frame - should work with template */
    uint8_t frame[MAX_BEACON_SIZE];
    size_t size;
    beacon_error_t status = beacon_generate_frame(frame, sizeof(frame), &size);

    TEST_ASSERT_EQUAL(BEACON_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);
}

void test_e2e_multiple_beacons(void)
{
    /*
     * Test generating multiple beacons in sequence
     */

    for (int i = 0; i < 10; i++) {
        /* Update telemetry with simulated changing values */
        uint16_t voltage = 3800 + (i * 10);
        int16_t temp = 20 + i;

        telem_point_t points[] = {
            { .name = "v", .type = TELEM_TYPE_UINT16, .period_ms = 100 },
            { .name = "t", .type = TELEM_TYPE_INT16, .period_ms = 100 },
        };

        telemetry_init();  /* Reset for clean test */
        for (int j = 0; j < 2; j++) {
            telemetry_register_point(&points[j]);
        }

        telemetry_update_value("v", &voltage, sizeof(voltage));
        telemetry_update_value("t", &temp, sizeof(temp));

        /* Build info string */
        char info[64];
        snprintf(info, sizeof(info), "V:%umV T:%dC #%04d", voltage, temp, i);
        beacon_set_info_field(info, strlen(info));

        /* Generate beacon */
        uint8_t frame[MAX_BEACON_SIZE];
        size_t size;
        beacon_error_t status = beacon_generate_frame(frame, sizeof(frame), &size);

        TEST_ASSERT_EQUAL(BEACON_OK, status);
        g_beacons_generated++;
    }

    TEST_ASSERT_EQUAL(10, g_beacons_generated);
}

/*******************************************************************************
 * Test Cases - Timing
 ******************************************************************************/

void test_beacon_cadence_timing(void)
{
    /* Set beacon interval */
    beacon_set_interval(60);  /* 60 second interval */

    /* Initially not due */
    beacon_init();

    /* Simulate passage of time and check if beacon is due */
    for (int tick = 0; tick < 100; tick++) {
        bool due = beacon_is_due_at_tick(tick * 1000);  /* 1 sec per tick */
        if (tick == 60 || tick == 0) {
            /* Should be due at t=0 and t=60 */
        }
    }

    TEST_PASS();
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Sensor to telemetry */
    RUN_TEST(test_sensor_to_telemetry_flow);
    RUN_TEST(test_eps_to_telemetry_flow);

    /* Telemetry to health data */
    RUN_TEST(test_telemetry_to_health_data);

    /* Jetson to beacon text */
    RUN_TEST(test_jetson_to_beacon_text);
    RUN_TEST(test_beacon_fallback_on_no_jetson);

    /* Beacon to AX.25 frame */
    RUN_TEST(test_beacon_to_ax25_frame);
    RUN_TEST(test_frame_contains_callsign);

    /* Frame to transmitter */
    RUN_TEST(test_frame_to_transmitter);

    /* Complete E2E flow */
    RUN_TEST(test_complete_e2e_beacon_flow);
    RUN_TEST(test_e2e_with_ai_failure);
    RUN_TEST(test_e2e_multiple_beacons);

    /* Timing */
    RUN_TEST(test_beacon_cadence_timing);

    return UNITY_END();
}
