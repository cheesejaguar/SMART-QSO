/**
 * @file test_telemetry_flow.c
 * @brief End-to-end telemetry flow integration tests
 *
 * Tests the complete telemetry pipeline from sensor sampling
 * through packet generation, storage, and downlink preparation.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "telemetry.h"
#include "sensors.h"
#include "eps_control.h"
#include "beacon.h"
#include "uart_comm.h"
#include <string.h>
#include <time.h>

/*******************************************************************************
 * Test Constants
 ******************************************************************************/

#define TEST_PACKET_SIZE    256U
#define TEST_ITERATIONS     100U

/*******************************************************************************
 * Test Data
 ******************************************************************************/

static uint8_t g_tx_buffer[512];
static size_t g_tx_size = 0;
static uint32_t g_packets_generated = 0;
static uint32_t g_packets_stored = 0;

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

void setUp(void)
{
    /* Initialize all subsystems */
    telemetry_init();
    sensors_init();
    eps_init();
    beacon_init();

    g_tx_size = 0;
    g_packets_generated = 0;
    g_packets_stored = 0;
    memset(g_tx_buffer, 0, sizeof(g_tx_buffer));
}

void tearDown(void)
{
    /* Cleanup after each test */
}

/*******************************************************************************
 * Test Cases - Sensor to Telemetry Flow
 ******************************************************************************/

void test_sensor_data_flows_to_telemetry(void)
{
    /* Register sensor telemetry points */
    telem_point_t points[] = {
        { .name = "sun_sensor_x", .type = TELEM_TYPE_FLOAT, .period_ms = 100 },
        { .name = "sun_sensor_y", .type = TELEM_TYPE_FLOAT, .period_ms = 100 },
        { .name = "mag_x", .type = TELEM_TYPE_FLOAT, .period_ms = 100 },
        { .name = "mag_y", .type = TELEM_TYPE_FLOAT, .period_ms = 100 },
        { .name = "mag_z", .type = TELEM_TYPE_FLOAT, .period_ms = 100 },
    };

    for (int i = 0; i < 5; i++) {
        telem_error_t status = telemetry_register_point(&points[i]);
        TEST_ASSERT_EQUAL(TELEM_OK, status);
    }

    /* Poll sensors and update telemetry */
    sensor_data_t sensor_data;
    sensors_poll(&sensor_data);

    /* Update telemetry values from sensor data */
    telemetry_update_value("sun_sensor_x", &sensor_data.sun_x, sizeof(float));
    telemetry_update_value("sun_sensor_y", &sensor_data.sun_y, sizeof(float));
    telemetry_update_value("mag_x", &sensor_data.mag_x, sizeof(float));
    telemetry_update_value("mag_y", &sensor_data.mag_y, sizeof(float));
    telemetry_update_value("mag_z", &sensor_data.mag_z, sizeof(float));

    /* Verify values can be retrieved */
    float retrieved_value;
    telem_error_t status = telemetry_get_value("sun_sensor_x", &retrieved_value, sizeof(float));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

void test_eps_data_flows_to_telemetry(void)
{
    /* Register EPS telemetry points */
    telem_point_t points[] = {
        { .name = "battery_voltage", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "battery_current", .type = TELEM_TYPE_INT16, .period_ms = 1000 },
        { .name = "battery_soc", .type = TELEM_TYPE_UINT8, .period_ms = 1000 },
        { .name = "solar_power", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "power_mode", .type = TELEM_TYPE_UINT8, .period_ms = 5000 },
    };

    for (int i = 0; i < 5; i++) {
        telem_error_t status = telemetry_register_point(&points[i]);
        TEST_ASSERT_EQUAL(TELEM_OK, status);
    }

    /* Get EPS data and update telemetry */
    eps_telemetry_t eps_data;
    eps_get_telemetry(&eps_data);

    telemetry_update_value("battery_voltage", &eps_data.battery_voltage_mv, sizeof(uint16_t));
    telemetry_update_value("battery_current", &eps_data.battery_current_ma, sizeof(int16_t));
    telemetry_update_value("battery_soc", &eps_data.battery_soc_percent, sizeof(uint8_t));

    /* Verify telemetry contains EPS data */
    uint16_t retrieved_voltage;
    telem_error_t status = telemetry_get_value("battery_voltage", &retrieved_voltage, sizeof(uint16_t));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

/*******************************************************************************
 * Test Cases - Packet Generation Flow
 ******************************************************************************/

void test_telemetry_packet_generation(void)
{
    /* Register some telemetry points */
    telem_point_t point = {
        .name = "test_value",
        .type = TELEM_TYPE_UINT32,
        .period_ms = 100
    };
    telemetry_register_point(&point);

    /* Update value */
    uint32_t value = 0x12345678;
    telemetry_update_value("test_value", &value, sizeof(value));

    /* Generate packet */
    uint8_t packet[256];
    size_t size;
    telem_error_t status = telemetry_generate_packet(packet, sizeof(packet), &size);

    TEST_ASSERT_EQUAL(TELEM_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);

    g_packets_generated++;
}

void test_packet_storage_and_retrieval(void)
{
    /* Generate and store multiple packets */
    for (int i = 0; i < 10; i++) {
        telem_point_t point = { .name = "seq", .type = TELEM_TYPE_UINT32, .period_ms = 100 };
        telemetry_register_point(&point);

        uint32_t seq = i;
        telemetry_update_value("seq", &seq, sizeof(seq));

        uint8_t packet[256];
        size_t size;
        telemetry_generate_packet(packet, sizeof(packet), &size);
        telemetry_store_packet(packet, size);
        g_packets_stored++;

        telemetry_init();  /* Reset for next point */
    }

    /* Verify packets can be retrieved */
    TEST_ASSERT_EQUAL(10, telemetry_get_stored_count());

    /* Retrieve all packets */
    for (int i = 0; i < 10; i++) {
        uint8_t retrieved[256];
        size_t size;
        telem_error_t status = telemetry_retrieve_packet(retrieved, sizeof(retrieved), &size);
        TEST_ASSERT_EQUAL(TELEM_OK, status);
        TEST_ASSERT_GREATER_THAN(0, size);
    }

    TEST_ASSERT_EQUAL(0, telemetry_get_stored_count());
}

void test_circular_buffer_wraparound(void)
{
    /* Fill buffer beyond capacity to test wraparound */
    uint8_t small_packet[] = {0x01, 0x02, 0x03, 0x04};

    for (int i = 0; i < 1000; i++) {
        telemetry_store_packet(small_packet, sizeof(small_packet));
    }

    /* Buffer should not overflow - oldest packets discarded */
    uint32_t stored = telemetry_get_stored_count();
    TEST_ASSERT_GREATER_THAN(0, stored);

    /* Retrieval should work */
    uint8_t retrieved[32];
    size_t size;
    telem_error_t status = telemetry_retrieve_packet(retrieved, sizeof(retrieved), &size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

/*******************************************************************************
 * Test Cases - Beacon Integration
 ******************************************************************************/

void test_telemetry_to_beacon_flow(void)
{
    /* Set up beacon */
    beacon_set_callsign("SQSO-1", 0);
    beacon_set_destination("CQ", 0);

    /* Create telemetry summary for beacon */
    telem_point_t points[] = {
        { .name = "batt_v", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "temp", .type = TELEM_TYPE_INT16, .period_ms = 1000 },
    };

    for (int i = 0; i < 2; i++) {
        telemetry_register_point(&points[i]);
    }

    uint16_t voltage = 3800;
    int16_t temp = 25;
    telemetry_update_value("batt_v", &voltage, sizeof(voltage));
    telemetry_update_value("temp", &temp, sizeof(temp));

    /* Format beacon info with telemetry summary */
    char beacon_info[128];
    snprintf(beacon_info, sizeof(beacon_info),
             "SMART-QSO V:%umV T:%dC",
             voltage, temp);

    beacon_set_info_field(beacon_info, strlen(beacon_info));

    /* Generate beacon frame */
    uint8_t frame[256];
    size_t size;
    beacon_error_t status = beacon_generate_frame(frame, sizeof(frame), &size);

    TEST_ASSERT_EQUAL(BEACON_OK, status);
    TEST_ASSERT_GREATER_THAN(20, size);  /* AX.25 frame has minimum size */
}

/*******************************************************************************
 * Test Cases - Downlink Queue
 ******************************************************************************/

void test_downlink_queue_priority(void)
{
    /* High priority telemetry should be at front of queue */
    uint8_t high_priority[] = {0xFF, 0x01, 0x00};  /* Priority marker */
    uint8_t normal[] = {0x00, 0x01, 0x02};

    telemetry_store_packet(normal, sizeof(normal));
    telemetry_store_packet_priority(high_priority, sizeof(high_priority), true);

    /* First retrieved should be high priority */
    uint8_t retrieved[32];
    size_t size;
    telemetry_retrieve_packet(retrieved, sizeof(retrieved), &size);

    TEST_ASSERT_EQUAL_HEX8(0xFF, retrieved[0]);
}

void test_downlink_during_pass(void)
{
    /* Simulate storing telemetry and draining during a ground pass */

    /* Store 50 packets */
    for (int i = 0; i < 50; i++) {
        uint8_t packet[32];
        packet[0] = (uint8_t)i;
        memset(&packet[1], 0xAA, 31);
        telemetry_store_packet(packet, sizeof(packet));
    }

    /* Simulate downlink - drain packets */
    uint32_t initial_count = telemetry_get_stored_count();
    uint32_t downlinked = 0;

    while (telemetry_get_stored_count() > 0) {
        uint8_t buffer[64];
        size_t size;
        telem_error_t status = telemetry_retrieve_packet(buffer, sizeof(buffer), &size);
        if (status == TELEM_OK) {
            downlinked++;
        } else {
            break;
        }
    }

    TEST_ASSERT_EQUAL(initial_count, downlinked);
    TEST_ASSERT_EQUAL(0, telemetry_get_stored_count());
}

/*******************************************************************************
 * Test Cases - Timing and Sampling
 ******************************************************************************/

void test_sampling_period_adherence(void)
{
    /* Register point with specific period */
    telem_point_t point = {
        .name = "periodic",
        .type = TELEM_TYPE_UINT32,
        .period_ms = 100
    };
    telemetry_register_point(&point);

    /* Simulate multiple sampling cycles */
    uint32_t sample_count = 0;

    for (int tick = 0; tick < 1000; tick++) {
        if (telemetry_should_sample("periodic", tick)) {
            sample_count++;
        }
    }

    /* Should sample approximately 10 times in 1000ms */
    TEST_ASSERT_GREATER_OR_EQUAL(9, sample_count);
    TEST_ASSERT_LESS_OR_EQUAL(11, sample_count);
}

/*******************************************************************************
 * Test Cases - Error Handling
 ******************************************************************************/

void test_telemetry_flow_with_sensor_failure(void)
{
    /* Simulate sensor failure */
    sensors_simulate_failure(SENSOR_SUN_X, true);

    /* Attempt to get sensor data */
    sensor_data_t data;
    sensor_error_t status = sensors_poll(&data);

    /* Telemetry should still be updateable with stale/default values */
    telem_point_t point = {
        .name = "sun_x",
        .type = TELEM_TYPE_FLOAT,
        .period_ms = 100
    };
    telemetry_register_point(&point);

    /* Use last known good or default value */
    float value = 0.0f;  /* Default/stale value */
    telem_error_t telem_status = telemetry_update_value("sun_x", &value, sizeof(value));
    TEST_ASSERT_EQUAL(TELEM_OK, telem_status);

    sensors_simulate_failure(SENSOR_SUN_X, false);  /* Reset */
}

void test_telemetry_flow_with_storage_full(void)
{
    /* Fill storage completely */
    uint8_t packet[256];
    memset(packet, 0xAA, sizeof(packet));

    uint32_t stored = 0;
    while (stored < 10000) {  /* Store many packets */
        telemetry_store_packet(packet, sizeof(packet));
        stored++;
    }

    /* Should still be able to store (with oldest dropped) */
    telem_error_t status = telemetry_store_packet(packet, sizeof(packet));
    TEST_ASSERT_EQUAL(TELEM_OK, status);

    /* Should still be able to retrieve */
    uint8_t retrieved[256];
    size_t size;
    status = telemetry_retrieve_packet(retrieved, sizeof(retrieved), &size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

/*******************************************************************************
 * Test Cases - Integration Scenarios
 ******************************************************************************/

void test_full_telemetry_cycle(void)
{
    /* Complete telemetry cycle: sample, format, store, retrieve */

    /* 1. Register points */
    telem_point_t points[] = {
        { .name = "voltage", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "current", .type = TELEM_TYPE_INT16, .period_ms = 1000 },
        { .name = "temp", .type = TELEM_TYPE_INT16, .period_ms = 5000 },
        { .name = "mode", .type = TELEM_TYPE_UINT8, .period_ms = 10000 },
    };

    for (int i = 0; i < 4; i++) {
        telemetry_register_point(&points[i]);
    }

    /* 2. Sample and update values */
    uint16_t voltage = 3850;
    int16_t current = -250;
    int16_t temp = 35;
    uint8_t mode = 2;

    telemetry_update_value("voltage", &voltage, sizeof(voltage));
    telemetry_update_value("current", &current, sizeof(current));
    telemetry_update_value("temp", &temp, sizeof(temp));
    telemetry_update_value("mode", &mode, sizeof(mode));

    /* 3. Generate packet */
    uint8_t packet[256];
    size_t packet_size;
    telem_error_t status = telemetry_generate_packet(packet, sizeof(packet), &packet_size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);

    /* 4. Store packet */
    status = telemetry_store_packet(packet, packet_size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);

    /* 5. Retrieve and verify */
    uint8_t retrieved[256];
    size_t retrieved_size;
    status = telemetry_retrieve_packet(retrieved, sizeof(retrieved), &retrieved_size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);
    TEST_ASSERT_EQUAL(packet_size, retrieved_size);
    TEST_ASSERT_EQUAL_MEMORY(packet, retrieved, packet_size);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Sensor to telemetry flow */
    RUN_TEST(test_sensor_data_flows_to_telemetry);
    RUN_TEST(test_eps_data_flows_to_telemetry);

    /* Packet generation */
    RUN_TEST(test_telemetry_packet_generation);
    RUN_TEST(test_packet_storage_and_retrieval);
    RUN_TEST(test_circular_buffer_wraparound);

    /* Beacon integration */
    RUN_TEST(test_telemetry_to_beacon_flow);

    /* Downlink queue */
    RUN_TEST(test_downlink_queue_priority);
    RUN_TEST(test_downlink_during_pass);

    /* Timing */
    RUN_TEST(test_sampling_period_adherence);

    /* Error handling */
    RUN_TEST(test_telemetry_flow_with_sensor_failure);
    RUN_TEST(test_telemetry_flow_with_storage_full);

    /* Integration */
    RUN_TEST(test_full_telemetry_cycle);

    return UNITY_END();
}
