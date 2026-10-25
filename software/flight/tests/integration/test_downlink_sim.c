/**
 * @file test_downlink_sim.c
 * @brief The downlink end to end: the host simulation's beacons as the TNC
 *        receives them
 *
 * Runs smart_qso_flight with $TNC_DEVICE set to a file, so the simulated
 * radio HAL writes there what the TNC would receive on its UART, and reads
 * the console. The driver decodes the KISS stream and the AX.25 frames with
 * its own code; it links no flight code.
 *
 * Two scenarios, one ctest entry each (argv[2]):
 *  - served: $SIM_RF_SILENCE_SERVED boots as after a software reset with the
 *    45-minute post-deployment silence served. A beacon is due at once: it
 *    must reach the TNC after the TNC's parameters, as a UI frame from
 *    KK6GDB-11 to CQ whose information field is the console's beacon text,
 *    with the power model's measurements in it.
 *  - silent: a first boot. The silence has 45 minutes to run, so nothing may
 *    reach the TNC.
 *
 * @requirement SRS-I023 Downlink frames to the TNC in KISS framing, with the TNC parameters
 * @requirement SRS-I024 No downlink frame unless transmission is permitted when it is handed over
 * @requirement SRS-F020 The software shall transmit AX.25 beacon at configurable intervals
 * @requirement SRS-F021 The software shall include callsign in all beacon transmissions
 * @requirement SYS-224 No RF signal earlier than 45 minutes after deployment; switch toggle resets the timer
 */

#define _POSIX_C_SOURCE 200809L

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#define KISS_FEND   0xC0U
#define KISS_FESC   0xDBU
#define KISS_TFEND  0xDCU
#define KISS_TFESC  0xDDU

#define MAX_STREAM      16384U
#define MAX_KISS        64U
#define MAX_FRAME       400U
#define MAX_CONSOLE     (256U * 1024U)
#define ADDR_LEN        7U
#define INFO_OFFSET     (2U * ADDR_LEN + 2U)

/** One KISS frame: command byte and contents */
typedef struct {
    uint8_t command;
    uint8_t data[MAX_FRAME];
    size_t len;
} KissFrame_t;

static const char *g_sim;
static const char *g_sensors;
static bool g_served;

static struct {
    int status;
    char console[MAX_CONSOLE];
    KissFrame_t kiss[MAX_KISS];
    size_t n_kiss;
    size_t stream_len;
    bool stream_bad;
} g_run;

/** The TNC's parameters before every frame (sqso_protocol::kiss::DOWNLINK_PARAMS) */
static const uint8_t k_params[5][2] = {{0x01U, 30U}, {0x02U, 255U}, {0x03U, 0U}, {0x04U, 1U}, {0x05U, 1U}};

static void decode_stream(const uint8_t *s, size_t n)
{
    KissFrame_t *cur = NULL;
    bool esc = false;
    for (size_t i = 0U; i < n; i++) {
        uint8_t b = s[i];
        if (b == KISS_FEND) {
            if ((cur != NULL) && (cur->len > 0U)) {
                g_run.n_kiss++;
            }
            cur = (g_run.n_kiss < MAX_KISS) ? &g_run.kiss[g_run.n_kiss] : NULL;
            if (cur != NULL) {
                cur->len = 0U;
            }
            esc = false;
            continue;
        }
        if (cur == NULL) {
            continue;
        }
        if (esc) {
            esc = false;
            if ((b != KISS_TFEND) && (b != KISS_TFESC)) {
                g_run.stream_bad = true;
                continue;
            }
            b = (b == KISS_TFEND) ? KISS_FEND : KISS_FESC;
        } else if (b == KISS_FESC) {
            esc = true;
            continue;
        }
        /* The first byte is the command; the rest is the contents */
        if (cur->len == 0U) {
            cur->command = b;
            cur->len = 1U;
        } else if (cur->len <= MAX_FRAME) {
            cur->data[cur->len - 1U] = b;
            cur->len++;
        } else {
            g_run.stream_bad = true;
        }
    }
    /* Bytes after the last FEND: an unterminated frame */
    if ((cur != NULL) && (cur->len > 0U)) {
        g_run.stream_bad = true;
    }
}

static size_t contents_len(const KissFrame_t *k)
{
    return k->len - 1U;
}

/** Run the simulation once; record its exit status, console and TNC stream */
static int run_simulation(void **state)
{
    (void)state;
    char tnc[] = "/tmp/sqso_tnc_XXXXXX";
    int fd = mkstemp(tnc);
    if (fd < 0) {
        return -1;
    }
    (void)close(fd);
    if ((setenv("TNC_DEVICE", tnc, 1) != 0) || (setenv("SENSORS_YAML", g_sensors, 1) != 0)) {
        return -1;
    }
    (void)unsetenv("UART_DEVICE");
    if (g_served) {
        (void)setenv("SIM_RF_SILENCE_SERVED", "1", 1);
    } else {
        (void)unsetenv("SIM_RF_SILENCE_SERVED");
    }

    char cmd[1024];
    (void)snprintf(cmd, sizeof(cmd), "'%s' 2>&1", g_sim);
    FILE *p = popen(cmd, "r");
    if (p == NULL) {
        return -1;
    }
    size_t used = fread(g_run.console, 1U, sizeof(g_run.console) - 1U, p);
    g_run.console[used] = '\0';
    g_run.status = pclose(p);

    static uint8_t stream[MAX_STREAM];
    FILE *f = fopen(tnc, "rb");
    if (f == NULL) {
        return -1;
    }
    g_run.stream_len = fread(stream, 1U, sizeof(stream), f);
    (void)fclose(f);
    (void)remove(tnc);
    decode_stream(stream, g_run.stream_len);
    return 0;
}

/** The console's "[BEACON] TX: " lines, in order; their count */
static size_t beacon_lines(const char *texts[], size_t max, size_t lens[])
{
    static const char tag[] = "[BEACON] TX: ";
    size_t n = 0U;
    for (const char *at = strstr(g_run.console, tag); at != NULL; at = strstr(at + 1, tag)) {
        const char *text = at + strlen(tag);
        const char *end = strchr(text, '\n');
        if (n < max) {
            texts[n] = text;
            lens[n] = (end != NULL) ? (size_t)(end - text) : strlen(text);
        }
        n++;
    }
    return n;
}

/** The address field's call sign and SSID */
static void decode_address(const uint8_t *a, char call[7], unsigned *ssid)
{
    size_t k = 0U;
    for (size_t i = 0U; i < 6U; i++) {
        char c = (char)(a[i] >> 1);
        if (c != ' ') {
            call[k++] = c;
        }
    }
    call[k] = '\0';
    *ssid = (a[6] >> 1) & 0x0FU;
}

static void test_runs_and_shuts_down_cleanly(void **state)
{
    (void)state;
    assert_true(WIFEXITED(g_run.status));
    assert_int_equal(WEXITSTATUS(g_run.status), 0);
    assert_non_null(strstr(g_run.console, "Shutdown complete"));
    assert_null(strstr(g_run.console, "panic"));
    assert_false(g_run.stream_bad);
}

/** Silent boot: not one byte for the TNC, not one beacon */
static void test_nothing_reaches_the_tnc(void **state)
{
    (void)state;
    const char *texts[1];
    size_t lens[1];
    assert_non_null(strstr(g_run.console, "[RF_SILENCE] state=COUNTING"));
    assert_int_equal(g_run.stream_len, 0U);
    assert_int_equal(beacon_lines(texts, 1U, lens), 0U);
}

/** Every data frame comes right after the five parameter commands */
static void test_parameters_precede_each_frame(void **state)
{
    (void)state;
    size_t frames = 0U;
    size_t params = 0U;
    for (size_t i = 0U; i < g_run.n_kiss; i++) {
        const KissFrame_t *k = &g_run.kiss[i];
        if (k->command == 0x00U) {
            assert_int_equal(params, 5U);
            params = 0U;
            frames++;
            continue;
        }
        assert_true(params < 5U);
        assert_int_equal(k->command, k_params[params][0]);
        assert_int_equal(contents_len(k), 1U);
        assert_int_equal(k->data[0], k_params[params][1]);
        params++;
    }
    assert_int_equal(params, 0U);
    assert_true(frames >= 1U);
}

/** Each data frame is a UI frame from KK6GDB-11 to CQ with printable text */
static void test_frames_are_ui_beacons(void **state)
{
    (void)state;
    for (size_t i = 0U; i < g_run.n_kiss; i++) {
        const KissFrame_t *k = &g_run.kiss[i];
        if (k->command != 0x00U) {
            continue;
        }
        size_t n = contents_len(k);
        assert_true((n > INFO_OFFSET) && (n <= 328U));
        char call[7];
        unsigned ssid = 0U;
        decode_address(&k->data[0], call, &ssid);
        assert_string_equal(call, "CQ");
        assert_int_equal(ssid, 0U);
        assert_int_equal(k->data[ADDR_LEN - 1U] & 0x01U, 0U);
        decode_address(&k->data[ADDR_LEN], call, &ssid);
        assert_string_equal(call, "KK6GDB");
        assert_int_equal(ssid, 11U);
        assert_int_equal(k->data[(2U * ADDR_LEN) - 1U] & 0x01U, 1U);
        assert_int_equal(k->data[2U * ADDR_LEN], 0x03U);
        assert_int_equal(k->data[(2U * ADDR_LEN) + 1U], 0xF0U);
        for (size_t j = INFO_OFFSET; j < n; j++) {
            assert_true((k->data[j] >= 0x20U) && (k->data[j] < 0x7FU));
        }
    }
}

/** The frames are exactly the beacons the console shows, in order */
static void test_frames_are_the_console_beacons(void **state)
{
    (void)state;
    const char *texts[MAX_KISS];
    size_t lens[MAX_KISS];
    size_t lines = beacon_lines(texts, MAX_KISS, lens);
    size_t b = 0U;
    for (size_t i = 0U; i < g_run.n_kiss; i++) {
        const KissFrame_t *k = &g_run.kiss[i];
        if (k->command != 0x00U) {
            continue;
        }
        assert_true(b < lines);
        size_t info = contents_len(k) - INFO_OFFSET;
        assert_int_equal(info, lens[b]);
        assert_memory_equal(&k->data[INFO_OFFSET], texts[b], info);
        b++;
    }
    assert_int_equal(b, lines);
    assert_true(lines >= 1U);
}

/** The beacon carries the power model's measurements, not dashes */
static void test_beacon_carries_measurements(void **state)
{
    (void)state;
    const char *texts[1];
    size_t lens[1];
    assert_true(beacon_lines(texts, 1U, lens) >= 1U);
    char text[512];
    size_t n = (lens[0] < (sizeof(text) - 1U)) ? lens[0] : (sizeof(text) - 1U);
    memcpy(text, texts[0], n);
    text[n] = '\0';
    const char *fields[] = {" SOC=", " BV=", " SUN="};
    for (size_t i = 0U; i < (sizeof(fields) / sizeof(fields[0])); i++) {
        const char *at = strstr(text, fields[i]);
        if ((at == NULL) || (at[strlen(fields[i])] < '0') || (at[strlen(fields[i])] > '9')) {
            fail_msg("no measured%s in \"%s\"", fields[i], text);
        }
    }
}

int main(int argc, char **argv)
{
    if ((argc != 4) || ((strcmp(argv[2], "served") != 0) && (strcmp(argv[2], "silent") != 0))) {
        fprintf(stderr, "usage: %s <smart_qso_flight> served|silent <sensors.yaml>\n", argv[0]);
        return 2;
    }
    g_sim = argv[1];
    g_served = (strcmp(argv[2], "served") == 0);
    g_sensors = argv[3];

    const struct CMUnitTest served[] = {
        cmocka_unit_test(test_runs_and_shuts_down_cleanly),
        cmocka_unit_test(test_parameters_precede_each_frame),
        cmocka_unit_test(test_frames_are_ui_beacons),
        cmocka_unit_test(test_frames_are_the_console_beacons),
        cmocka_unit_test(test_beacon_carries_measurements),
    };
    const struct CMUnitTest silent[] = {
        cmocka_unit_test(test_runs_and_shuts_down_cleanly),
        cmocka_unit_test(test_nothing_reaches_the_tnc),
    };
    if (g_served) {
        return cmocka_run_group_tests_name("Downlink: silence served", served, run_simulation, NULL);
    }
    return cmocka_run_group_tests_name("Downlink: first boot, silent", silent, run_simulation, NULL);
}
