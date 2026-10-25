/**
 * @file test_executive_jetson.c
 * @brief The executive's Jetson link, end to end in the host simulation
 *
 * Runs the host simulation (smart_qso_flight, about 20 s: 1000 passes of the
 * executive) with a pseudo-terminal as the Jetson port (UART_DEVICE) and
 * plays the Jetson on the other end. A second pseudo-terminal carries the
 * console, so each console line is seen when it is written. What the OBC
 * sends on the line and prints is recorded with the time it arrived; the
 * simulation's clock is CLOCK_REALTIME, the same clock as this driver's, so
 * the time field of a TELEMETRY line can be compared with arrival times.
 * The tests then check the record against docs/interfaces/ICD_OBC_JETSON.md
 * section 4 and the main loop (software/rust/sqso-ffi/src/executive.rs).
 *
 * Two scenarios, one ctest entry each (argv[2]):
 *
 * - "link": the flight sensor table. Telemetry every TELEMETRY_INTERVAL_MS
 *   while the payload is on, none while the power model has it off, the
 *   first one at once when the payload comes back on; each followed by a
 *   BEACON_REQ with the next sequence number; the Jetson's STATUS and
 *   BEACON_RSP answers taken (only the answer to the latest request, only
 *   with a good CRC).
 * - "guard": a sensor table too long for the line, so the status fields do
 *   not fit and the trailing comma is trimmed. A STATUS with the junction at
 *   99 C trips the payload guard; no telemetry follows. The Jetson end is
 *   then closed (the line hangs up) away from the periodic UART health
 *   check, which must notice it in its next window (wall clock within 100 ms
 *   of a multiple of 10 s), and only then.
 *
 * The beacon itself is not checked: every simulated boot starts the 45-minute
 * post-deployment RF silence (the simulated flash is RAM) and the beacon
 * interval is 60 s, so no beacon is sent in a run.
 *
 * The OBC's host port leaves VMIN and VTIME as it finds them and a fresh
 * terminal has VMIN 1, so its non-blocking poll of the line would block
 * until the Jetson sends a byte. The driver sets VMIN 0 and VTIME 0 on the
 * line first, as the target's port behaves (read what has arrived).
 *
 * Usage: test_executive_jetson <smart_qso_flight> link|guard <sensors.yaml>
 *
 * @requirement SRS-F030 Send health telemetry to Jetson via UART
 * @requirement SRS-F031 Receive beacon text from Jetson via UART
 * @requirement SRS-F034 Validate received beacon text
 * @requirement SYS-520 Detect AI payload failure or timeout (UART health check)
 */

/* posix_openpt() and friends: pseudo-terminals stand in for the serial ports */
#define _XOPEN_SOURCE 700

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

/*===========================================================================*/
/* Run parameters                                                            */
/*===========================================================================*/

/** TELEMETRY_INTERVAL_MS given to the simulation */
#define INTERVAL_MS 1000U
/** The simulation's run: 1000 passes with a 20 ms sleep each, at least */
#define RUN_MIN_MS 20000U
/** Longest the driver waits for the simulation to end */
#define RUN_DEADLINE_MS 120000U
/** Health-check window: now % HEALTH_PERIOD_MS < HEALTH_WINDOW_MS */
#define HEALTH_PERIOD_MS 10000U
#define HEALTH_WINDOW_MS 100U
/** Slack for a console line to reach the driver after it was printed */
#define ARRIVAL_SLACK_MS 60U
/** The first pass in a health window is at most one pass (20 ms) into it; with slack */
#define HEALTH_FIRST_PASS_MS 60U
/** The beacon text length the OBC asks for (JETSON_TEXT_MAX) */
#define TEXT_MAX 176U
/** Longest line, CRC field and LF included (JETSON_LINE_MAX) */
#define LINE_MAX_ICD 512U

#define JETSON_TEXT_CAP 640U
#define CONSOLE_TEXT_CAP 240U
#define MAX_JETSON_LINES 256U
#define MAX_CONSOLE_LINES 8192U
#define MAX_FIELDS 64U

/** Beacon texts the Jetson answers with */
#define TEXT_A "Hello, world from the test"
#define TEXT_B "QRV 73"

/** Filler sensors in the "guard" table: 25 telemetry bytes each */
#define FILLER_SENSORS 22U

/*===========================================================================*/
/* The record of a run                                                        */
/*===========================================================================*/

typedef struct {
    uint64_t at_ms;
    char text[JETSON_TEXT_CAP];
    size_t len;  /**< Bytes, without the LF */
} JetsonLine_t;

typedef struct {
    uint64_t at_ms;
    char text[CONSOLE_TEXT_CAP];
} ConsoleLine_t;

typedef struct {
    bool guard;                  /**< Scenario "guard" (else "link") */
    uint64_t start_ms;
    uint64_t end_ms;
    int exit_status;             /**< -1 if it did not exit by itself */
    char slave[64];              /**< The Jetson line's terminal */
    JetsonLine_t jetson[MAX_JETSON_LINES];
    size_t n_jetson;
    bool jetson_overflow;
    ConsoleLine_t console[MAX_CONSOLE_LINES];
    size_t n_console;
    /* "guard" */
    uint64_t status_sent_ms;
    uint64_t trip_ms;            /**< "[EPS] Payload disabled" after the hot STATUS */
    uint64_t close_ms;           /**< Jetson end closed */
    uint64_t window_ms;          /**< Start of the next health window after close_ms */
    bool close_unschedulable;
} Run_t;

static Run_t g_run;

/** Partial line being received on one descriptor */
typedef struct {
    char buf[JETSON_TEXT_CAP];
    size_t len;
    bool overlong;
} Assembler_t;

static uint64_t now_ms(void)
{
    struct timespec ts;
    (void)clock_gettime(CLOCK_REALTIME, &ts);
    return ((uint64_t)ts.tv_sec * 1000U) + ((uint64_t)ts.tv_nsec / 1000000U);
}

/*===========================================================================*/
/* The ICD line format                                                        */
/*===========================================================================*/

/** CRC-16/CCITT-FALSE (ICD section 4.4), written here independently of the OBC's */
static uint16_t crc16(const char *data, size_t len)
{
    unsigned crc = 0xFFFFU;
    for (size_t i = 0U; i < len; i++) {
        crc ^= (unsigned)(unsigned char)data[i] << 8;
        for (unsigned bit = 0U; bit < 8U; bit++) {
            crc = ((crc & 0x8000U) != 0U) ? ((crc << 1) ^ 0x1021U) : (crc << 1);
            crc &= 0xFFFFU;
        }
    }
    return (uint16_t)crc;
}

/** Whether a received line (no LF) is printable ASCII and ends in its correct ",XXXX" */
static bool sealed_ok(const char *line, size_t len)
{
    if ((len < 6U) || (line[len - 5U] != ',')) {
        return false;
    }
    for (size_t i = 0U; i < len; i++) {
        if ((line[i] < 0x20) || (line[i] > 0x7E)) {
            return false;
        }
    }
    char want[5];
    (void)snprintf(want, sizeof(want), "%04X", (unsigned)crc16(line, len - 5U));
    return memcmp(&line[len - 4U], want, 4U) == 0;
}

/** Send body + ",<CRC>\n" (corrupt: a wrong CRC) on the Jetson end */
static void send_line(int fd, const char *body, bool corrupt)
{
    char line[JETSON_TEXT_CAP];
    size_t n = strlen(body);
    unsigned crc = crc16(body, n);
    if (corrupt) {
        crc ^= 0x0101U;
    }
    int len = snprintf(line, sizeof(line), "%s,%04X\n", body, crc);
    if ((len > 0) && ((size_t)len < sizeof(line)) && (fd >= 0)) {
        ssize_t written = write(fd, line, (size_t)len);
        if (written != (ssize_t)len) {
            fprintf(stderr, "[DRIVER] short write to the Jetson line\n");
        }
    }
}

/** Split a copy of a line at commas; the CRC field is dropped. The field count */
static size_t split_fields(const JetsonLine_t *line, char *copy, size_t cap, char **fields)
{
    if ((line->len < 5U) || (line->len >= cap)) {
        return 0U;
    }
    memcpy(copy, line->text, line->len - 5U);
    copy[line->len - 5U] = '\0';
    size_t n = 0U;
    char *p = copy;
    for (;;) {
        if (n == MAX_FIELDS) {
            return n;
        }
        fields[n] = p;
        n++;
        char *comma = strchr(p, ',');
        if (comma == NULL) {
            return n;
        }
        *comma = '\0';
        p = comma + 1;
    }
}

static bool starts_with(const char *s, const char *prefix)
{
    return strncmp(s, prefix, strlen(prefix)) == 0;
}

static bool is_telemetry(const JetsonLine_t *line)
{
    return starts_with(line->text, "TELEMETRY,");
}

/** time_ms of a TELEMETRY line (0 if unreadable) */
static uint64_t telemetry_time(const JetsonLine_t *line)
{
    return strtoull(&line->text[strlen("TELEMETRY,")], NULL, 10);
}

/** Sequence number of a BEACON_REQ line, or -1 */
static long beacon_req_seq(const JetsonLine_t *line)
{
    if (!starts_with(line->text, "BEACON_REQ,")) {
        return -1;
    }
    return strtol(&line->text[strlen("BEACON_REQ,")], NULL, 10);
}

/** Whether s is all decimal digits (at least one) */
static bool all_digits(const char *s)
{
    if (*s == '\0') {
        return false;
    }
    for (; *s != '\0'; s++) {
        if ((*s < '0') || (*s > '9')) {
            return false;
        }
    }
    return true;
}

/** Whether s is <digits>.<exactly `decimals` digits> */
static bool is_fixed(const char *s, size_t decimals)
{
    const char *dot = strchr(s, '.');
    if ((dot == NULL) || (dot == s) || (strlen(dot + 1) != decimals)) {
        return false;
    }
    for (const char *p = s; p < dot; p++) {
        if ((*p < '0') || (*p > '9')) {
            return false;
        }
    }
    return all_digits(dot + 1);
}

/*===========================================================================*/
/* The console record                                                         */
/*===========================================================================*/

/** Index of the first console line at or after `from` containing `needle`, or n_console */
static size_t console_find(const char *needle, size_t from)
{
    for (size_t i = from; i < g_run.n_console; i++) {
        if (strstr(g_run.console[i].text, needle) != NULL) {
            return i;
        }
    }
    return g_run.n_console;
}

static size_t console_count(const char *needle)
{
    size_t count = 0U;
    for (size_t i = console_find(needle, 0U); i < g_run.n_console; i = console_find(needle, i + 1U)) {
        count++;
    }
    return count;
}

/*===========================================================================*/
/* The Jetson's side                                                          */
/*===========================================================================*/

static void on_jetson_line(int fd, const JetsonLine_t *line)
{
    long seq = beacon_req_seq(line);
    if (g_run.guard) {
        if ((seq == 2) && (g_run.status_sent_ms == 0U)) {
            /* Junction at 99 C: at or above JETSON junction limit (95 C) */
            send_line(fd, "STATUS,1,99,5000,50,10", false);
            g_run.status_sent_ms = now_ms();
        }
        return;
    }
    char body[256];
    switch (seq) {
    case 1:
        /* ICD golden STATUS line, and an answer to a request never made */
        send_line(fd, "STATUS,0,45,6800,37,1234", false);
        send_line(fd, "BEACON_RSP,0,Not the latest request", false);
        break;
    case 2:
        send_line(fd, "BEACON_RSP,2,Corrupted on the way", true);
        break;
    case 3:
        (void)snprintf(body, sizeof(body), "BEACON_RSP,3,%s", TEXT_A);
        send_line(fd, body, false);
        break;
    case 4:
        /* Late answer to request 3: stale once request 4 is out */
        send_line(fd, "BEACON_RSP,3,Too late for request four", false);
        break;
    case 5:
        (void)snprintf(body, sizeof(body), "BEACON_RSP,5,%s", TEXT_B);
        send_line(fd, body, false);
        break;
    default:
        break;
    }
}

static void on_console_line(const ConsoleLine_t *line)
{
    if (g_run.guard && (g_run.status_sent_ms != 0U) && (g_run.trip_ms == 0U) &&
        (strstr(line->text, "[EPS] Payload disabled") != NULL)) {
        g_run.trip_ms = line->at_ms;
    }
}

/**
 * "guard": once the trip has been seen and two intervals have passed
 * without telemetry, close the Jetson end at a moment well away from a
 * health-check window, with the next window well before the run ends
 */
static void on_idle(int *jetson_fd)
{
    if (!g_run.guard || (g_run.trip_ms == 0U) || (*jetson_fd < 0) || g_run.close_unschedulable) {
        return;
    }
    uint64_t now = now_ms();
    if (now < (g_run.trip_ms + (2U * INTERVAL_MS) + 500U)) {
        return;
    }
    uint64_t phase = now % HEALTH_PERIOD_MS;
    if ((phase < 500U) || (phase > 9000U)) {
        return;
    }
    uint64_t window = (now - phase) + HEALTH_PERIOD_MS;
    if ((window + HEALTH_WINDOW_MS + 500U) > (g_run.start_ms + RUN_MIN_MS)) {
        g_run.close_unschedulable = true;
        return;
    }
    (void)close(*jetson_fd);
    *jetson_fd = -1;
    g_run.close_ms = now;
    g_run.window_ms = window;
    printf("[DRIVER] Jetson end closed at +%llu ms (phase %llu ms); next health window at +%llu ms\n",
           (unsigned long long)(now - g_run.start_ms), (unsigned long long)phase,
           (unsigned long long)(window - g_run.start_ms));
}

/*===========================================================================*/
/* Running the simulation                                                     */
/*===========================================================================*/

/** Raw, non-blocking terminal: no echo, no line editing, VMIN 0 and VTIME 0 */
static int make_raw(int fd)
{
    struct termios tty;
    if (tcgetattr(fd, &tty) != 0) {
        return -1;
    }
    tty.c_iflag &= ~(tcflag_t)(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
    tty.c_oflag &= ~(tcflag_t)OPOST;
    tty.c_lflag &= ~(tcflag_t)(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
    tty.c_cflag = (tty.c_cflag & ~(tcflag_t)(CSIZE | PARENB)) | CS8;
    tty.c_cc[VMIN] = 0;
    tty.c_cc[VTIME] = 0;
    return tcsetattr(fd, TCSANOW, &tty);
}

/** A pty pair: master (non-blocking) and an open slave, both close-on-exec, raw; slave path in name */
static int open_pty(int *master, int *slave, char *name, size_t cap)
{
    *master = posix_openpt(O_RDWR | O_NOCTTY);
    if ((*master < 0) || (grantpt(*master) != 0) || (unlockpt(*master) != 0)) {
        return -1;
    }
    const char *path = ptsname(*master);
    if ((path == NULL) || (strlen(path) >= cap)) {
        return -1;
    }
    (void)snprintf(name, cap, "%s", path);
    *slave = open(name, O_RDWR | O_NOCTTY);
    if (*slave < 0) {
        return -1;
    }
    /* The driver polls; reading a master never waits */
    (void)fcntl(*master, F_SETFL, fcntl(*master, F_GETFL) | O_NONBLOCK);
    (void)fcntl(*master, F_SETFD, FD_CLOEXEC);
    (void)fcntl(*slave, F_SETFD, FD_CLOEXEC);
    return make_raw(*slave);
}

/** Bytes in; each complete line recorded (and acted on) */
static void feed(Assembler_t *a, const char *data, size_t n, bool jetson, int jetson_fd)
{
    for (size_t i = 0U; i < n; i++) {
        char c = data[i];
        if (c == '\r') {
            continue;
        }
        if (c != '\n') {
            if (a->len < (sizeof(a->buf) - 1U)) {
                a->buf[a->len] = c;
                a->len++;
            } else {
                a->overlong = true;
            }
            continue;
        }
        a->buf[a->len] = '\0';
        uint64_t at = now_ms();
        if (jetson) {
            if (g_run.n_jetson < MAX_JETSON_LINES) {
                JetsonLine_t *line = &g_run.jetson[g_run.n_jetson];
                line->at_ms = at;
                line->len = a->len;
                memcpy(line->text, a->buf, a->len + 1U);
                g_run.n_jetson++;
                if (a->overlong) {
                    g_run.jetson_overflow = true;
                }
                on_jetson_line(jetson_fd, line);
            } else {
                g_run.jetson_overflow = true;
            }
        } else if (g_run.n_console < MAX_CONSOLE_LINES) {
            ConsoleLine_t *line = &g_run.console[g_run.n_console];
            line->at_ms = at;
            size_t n_text = (a->len < (sizeof(line->text) - 1U)) ? a->len : (sizeof(line->text) - 1U);
            memcpy(line->text, a->buf, n_text);
            line->text[n_text] = '\0';
            g_run.n_console++;
            on_console_line(line);
        } else {
            /* Console record full: the checks below see what fits */
        }
        a->len = 0U;
        a->overlong = false;
    }
}

/** Read what is there on a (non-blocking) master; false when nothing is */
static bool pump(int fd, Assembler_t *a, bool jetson, int jetson_fd)
{
    char chunk[1024];
    ssize_t n = read(fd, chunk, sizeof(chunk));
    if (n > 0) {
        feed(a, chunk, (size_t)n, jetson, jetson_fd);
        return true;
    }
    return false;
}

/** A sensor table whose telemetry fields do not leave room for the status fields */
static int write_long_table(const char *path)
{
    FILE *f = fopen(path, "w");
    if (f == NULL) {
        return -1;
    }
    /* The payload guard's inputs first, as in sensors.yaml */
    fprintf(f, "sensors:\n"
               "  - id: BUSV\n    name: Bus Voltage\n    type: eps_voltage\n    channel: bus\n"
               "    units: V\n    period_ms: 1000\n"
               "  - id: JPC\n    name: Jetson Payload Current\n    type: eps_current\n    channel: jetson\n"
               "    units: A\n    period_ms: 1000\n"
               "  - id: PT\n    name: Payload Heat Spreader Temperature\n    type: payload_temperature\n"
               "    channel: payload\n    units: C\n    period_ms: 1000\n");
    for (unsigned i = 1U; i <= FILLER_SENSORS; i++) {
        fprintf(f, "  - id: F%02u\n    name: Filler %02u\n    type: eps_voltage\n    channel: bus\n"
                   "    units: filler_volts_%02u\n    period_ms: 1000\n", i, i, i);
    }
    return (fclose(f) == 0) ? 0 : -1;
}

static void remove_store(const char *variable)
{
    const char *path = getenv(variable);
    if ((path != NULL) && (*path != '\0')) {
        (void)remove(path);
    }
}

static void write_transcript(const char *path)
{
    FILE *f = fopen(path, "w");
    if (f == NULL) {
        return;
    }
    size_t j = 0U;
    size_t c = 0U;
    while ((j < g_run.n_jetson) || (c < g_run.n_console)) {
        bool take_jetson = (j < g_run.n_jetson) &&
                           ((c == g_run.n_console) || (g_run.jetson[j].at_ms <= g_run.console[c].at_ms));
        if (take_jetson) {
            fprintf(f, "%8llu LINE    %s\n", (unsigned long long)(g_run.jetson[j].at_ms - g_run.start_ms),
                    g_run.jetson[j].text);
            j++;
        } else {
            fprintf(f, "%8llu CONSOLE %s\n", (unsigned long long)(g_run.console[c].at_ms - g_run.start_ms),
                    g_run.console[c].text);
            c++;
        }
    }
    (void)fclose(f);
    printf("[DRIVER] Transcript: %s\n", path);
}

static const char *g_sim;
static const char *g_sensors;

static int run_simulation(void **state)
{
    (void)state;
    int jetson_master = -1;
    int jetson_slave = -1;
    int console_master = -1;
    int console_slave = -1;
    char console_name[64];
    if ((open_pty(&jetson_master, &jetson_slave, g_run.slave, sizeof(g_run.slave)) != 0) ||
        (open_pty(&console_master, &console_slave, console_name, sizeof(console_name)) != 0)) {
        fprintf(stderr, "[DRIVER] pseudo-terminal setup failed: %s\n", strerror(errno));
        return -1;
    }

    const char *sensors = g_sensors;
    if (g_run.guard) {
        sensors = "executive_jetson_long_sensors.yaml";
        if (write_long_table(sensors) != 0) {
            fprintf(stderr, "[DRIVER] cannot write %s\n", sensors);
            return -1;
        }
    }
    /* A fresh mission: no fault log, EPS configuration, mission data or state from earlier runs */
    remove_store("SMART_QSO_FAULT_LOG");
    remove_store("SMART_QSO_EPS_CONFIG");
    remove_store("SMART_QSO_MISSION_DATA");
    remove_store("SMART_QSO_STATE_FILE");

    (void)fflush(NULL);
    g_run.start_ms = now_ms();
    pid_t pid = fork();
    if (pid < 0) {
        return -1;
    }
    if (pid == 0) {
        char interval[16];
        (void)snprintf(interval, sizeof(interval), "%u", INTERVAL_MS);
        int con = open(console_name, O_RDWR | O_NOCTTY);
        if ((con < 0) || (dup2(con, STDOUT_FILENO) < 0) || (dup2(con, STDERR_FILENO) < 0) ||
            (setenv("UART_DEVICE", g_run.slave, 1) != 0) || (setenv("TELEMETRY_INTERVAL_MS", interval, 1) != 0) ||
            (setenv("SENSORS_YAML", sensors, 1) != 0)) {
            _exit(126);
        }
        (void)unsetenv("UART_BAUDRATE");
        execl(g_sim, g_sim, (char *)NULL);
        _exit(127);
    }

    Assembler_t jetson_rx = { .len = 0U };
    Assembler_t console_rx = { .len = 0U };
    int status = 0;
    bool exited = false;
    while (!exited) {
        struct pollfd fds[2] = {
            { .fd = console_master, .events = POLLIN, .revents = 0 },
            { .fd = jetson_master, .events = POLLIN, .revents = 0 },
        };
        (void)poll(fds, (jetson_master >= 0) ? 2U : 1U, 10);
        while (pump(console_master, &console_rx, false, jetson_master)) {
        }
        while ((jetson_master >= 0) && pump(jetson_master, &jetson_rx, true, jetson_master)) {
        }
        on_idle(&jetson_master);
        if (waitpid(pid, &status, WNOHANG) == pid) {
            exited = true;
        } else if (now_ms() > (g_run.start_ms + RUN_DEADLINE_MS)) {
            fprintf(stderr, "[DRIVER] simulation did not end; stopping it\n");
            (void)kill(pid, SIGKILL);
            (void)waitpid(pid, &status, 0);
            g_run.exit_status = -1;
            break;
        } else {
            /* keep going */
        }
    }
    g_run.end_ms = now_ms();
    /* What was written before the exit */
    while (pump(console_master, &console_rx, false, -1)) {
    }
    while ((jetson_master >= 0) && pump(jetson_master, &jetson_rx, true, -1)) {
    }
    if (exited) {
        g_run.exit_status = WIFEXITED(status) ? WEXITSTATUS(status) : -1;
    }
    (void)close(console_master);
    (void)close(console_slave);
    (void)close(jetson_slave);
    if (jetson_master >= 0) {
        (void)close(jetson_master);
    }
    write_transcript(g_run.guard ? "executive_jetson_guard.log" : "executive_jetson_link.log");
    return 0;
}

/*===========================================================================*/
/* Checks common to both scenarios                                            */
/*===========================================================================*/

/** Started, opened the line given by UART_DEVICE, ran its passes, shut down cleanly */
static void test_runs_and_shuts_down_cleanly(void **state)
{
    (void)state;
    char opened[128];
    (void)snprintf(opened, sizeof(opened), "[UART] Initialized %s", g_run.slave);
    assert_int_equal(g_run.exit_status, 0);
    assert_true(console_count("[SYSTEM] SMART-QSO Flight Software Started") == 1U);
    assert_true(console_count(opened) == 1U);
    assert_true(console_count("[SYSTEM] Shutdown complete") == 1U);
    assert_true(console_count("Rust panic") == 0U);
    assert_true(console_count("line not built") == 0U);
    assert_true(console_count("Failed to send") == 0U);
    assert_true((g_run.end_ms - g_run.start_ms) >= RUN_MIN_MS);
}

/** Every line is a sealed ICD line of an OBC-to-Jetson type, within 512 bytes */
static void test_lines_are_sealed_icd_lines(void **state)
{
    (void)state;
    assert_false(g_run.jetson_overflow);
    assert_true(g_run.n_jetson >= 4U);
    for (size_t i = 0U; i < g_run.n_jetson; i++) {
        const JetsonLine_t *line = &g_run.jetson[i];
        if (!sealed_ok(line->text, line->len)) {
            fail_msg("line %zu not sealed: %s", i, line->text);
        }
        assert_true((line->len + 1U) <= LINE_MAX_ICD);
        assert_true(is_telemetry(line) || (beacon_req_seq(line) >= 0));
    }
}

/**
 * TELEMETRY then BEACON_REQ,<seq>,176 in turn, the sequence counting up
 * from 1, and the console reports each line sent with its length
 */
static void test_each_telemetry_is_followed_by_a_beacon_request(void **state)
{
    (void)state;
    assert_true((g_run.n_jetson % 2U) == 0U);
    char want[64];
    for (size_t i = 0U; i < g_run.n_jetson; i += 2U) {
        assert_true(is_telemetry(&g_run.jetson[i]));
        long seq = (long)(i / 2U) + 1L;
        (void)snprintf(want, sizeof(want), "BEACON_REQ,%ld,%u,", seq, TEXT_MAX);
        if (!starts_with(g_run.jetson[i + 1U].text, want)) {
            fail_msg("line %zu: want %s, got %s", i + 1U, want, g_run.jetson[i + 1U].text);
        }
    }
    assert_int_equal(console_count("[UART] Sent telemetry to Jetson ("), g_run.n_jetson / 2U);
    assert_int_equal(console_count("[UART] Sent beacon request to Jetson ("), g_run.n_jetson / 2U);
    (void)snprintf(want, sizeof(want), "[UART] Sent telemetry to Jetson (%zu bytes)", g_run.jetson[0].len + 1U);
    assert_true(console_count(want) >= 1U);
}

/**
 * The fixed fields: time_ms on the simulation's clock, SUNLIT or ECLIPSE,
 * soc 0.000-1.000, power mode ACTIVE (the payload runs only in ACTIVE),
 * power limit with one decimal; then KEY=VALUE fields, none empty
 */
static void check_telemetry_header(char **f, size_t n)
{
    assert_true(n > 6U);
    assert_string_equal(f[0], "TELEMETRY");
    assert_true(all_digits(f[1]));
    uint64_t t = strtoull(f[1], NULL, 10);
    assert_true((t >= g_run.start_ms) && (t <= g_run.end_ms));
    assert_true((strcmp(f[2], "SUNLIT") == 0) || (strcmp(f[2], "ECLIPSE") == 0));
    assert_true(is_fixed(f[3], 3U));
    assert_true(strtod(f[3], NULL) <= 1.0);
    assert_string_equal(f[4], "2");
    assert_true(is_fixed(f[5], 1U));
    for (size_t k = 6U; k < n; k++) {
        const char *eq = strchr(f[k], '=');
        if ((eq == NULL) || (eq == f[k]) || (eq[1] == '\0')) {
            fail_msg("field %zu is not KEY=VALUE: '%s'", k, f[k]);
        }
    }
}

/** While the payload is on, telemetry every interval: never sooner, at most a few passes later */
static void test_telemetry_at_the_interval(void **state)
{
    (void)state;
    uint64_t prev = 0U;
    size_t count = 0U;
    for (size_t i = 0U; i < g_run.n_jetson; i++) {
        if (!is_telemetry(&g_run.jetson[i])) {
            continue;
        }
        uint64_t t = telemetry_time(&g_run.jetson[i]);
        if (prev != 0U) {
            /* The line's time is read a little after the pass's time that the interval counts from */
            assert_true((t - prev) >= (INTERVAL_MS - 20U));
            /* Payload off in between (power model or guard): the gap may be longer */
            bool off = false;
            for (size_t c = console_find("[EPS] Payload disabled", 0U); c < g_run.n_console;
                 c = console_find("[EPS] Payload disabled", c + 1U)) {
                off = off || ((g_run.console[c].at_ms + ARRIVAL_SLACK_MS > prev) && (g_run.console[c].at_ms < t));
            }
            if (!off && ((t - prev) > (INTERVAL_MS + 500U))) {
                fail_msg("telemetry gap %llu ms with the payload on", (unsigned long long)(t - prev));
            }
        }
        prev = t;
        count++;
    }
    assert_true(count >= 2U);
}

/*===========================================================================*/
/* Scenario "link"                                                            */
/*===========================================================================*/

/** The fields of every TELEMETRY line with the flight sensor table */
static void test_telemetry_fields(void **state)
{
    (void)state;
    static const char *const sensors[] = { "SET", "BV", "BT", "BUSV", "BDI", "JPC", "PT", "SPV", "SPC", "ST" };
    static const char *const status[] = { "EPS_PAYLOAD", "EPS_RADIO", "EPS_ADCS", "EPS_BEACON",
                                          "LAST_FAULT", "FAULT_COUNT", "UART_HEALTH", "WATCHDOG" };
    const size_t n_sensors = sizeof(sensors) / sizeof(sensors[0]);
    const size_t n_status = sizeof(status) / sizeof(status[0]);
    char copy[JETSON_TEXT_CAP];
    char *f[MAX_FIELDS];
    for (size_t i = 0U; i < g_run.n_jetson; i++) {
        if (!is_telemetry(&g_run.jetson[i])) {
            continue;
        }
        size_t n = split_fields(&g_run.jetson[i], copy, sizeof(copy), f);
        check_telemetry_header(f, n);
        assert_int_equal(n, 6U + n_sensors + n_status);
        char key[32];
        for (size_t k = 0U; k < n_sensors; k++) {
            (void)snprintf(key, sizeof(key), "%s=", sensors[k]);
            assert_true(starts_with(f[6U + k], key));
        }
        for (size_t k = 0U; k < n_status; k++) {
            (void)snprintf(key, sizeof(key), "%s=", status[k]);
            assert_true(starts_with(f[6U + n_sensors + k], key));
        }
        /* The payload is on while telemetry is sent; the line opened */
        assert_string_equal(f[6U + n_sensors], "EPS_PAYLOAD=ON");
        assert_true(all_digits(&f[6U + n_sensors + 5U][strlen("FAULT_COUNT=")]));
        assert_string_equal(f[n - 2U], "UART_HEALTH=OK");
        assert_true((strcmp(f[n - 1U], "WATCHDOG=OK") == 0) || (strcmp(f[n - 1U], "WATCHDOG=TRIGGERED") == 0));
    }
}

/**
 * The power model turns the payload off in eclipse and on again in sunlight.
 * No telemetry while it is off; when it comes back on after more than an
 * interval, the first telemetry goes in the same pass
 */
static void test_telemetry_follows_the_payload_power(void **state)
{
    (void)state;
    size_t resumed = 0U;
    for (size_t d = console_find("[EPS] Payload disabled", 0U); d < g_run.n_console;
         d = console_find("[EPS] Payload disabled", d + 1U)) {
        size_t e = console_find("[EPS] Payload enabled", d);
        if (e == g_run.n_console) {
            break;
        }
        uint64_t disabled = g_run.console[d].at_ms;
        uint64_t enabled = g_run.console[e].at_ms;
        /* The last telemetry before the outage, the first after it began */
        uint64_t before = 0U;
        uint64_t first = 0U;
        for (size_t i = 0U; i < g_run.n_jetson; i++) {
            uint64_t t = telemetry_time(&g_run.jetson[i]);
            if (!is_telemetry(&g_run.jetson[i])) {
                continue;
            }
            if (t <= disabled) {
                before = t;
            } else if (first == 0U) {
                first = t;
            } else {
                /* later lines */
            }
        }
        /* Nothing due yet when the payload came back (a short outage), or the run ended */
        if ((before == 0U) || ((enabled - before) < (INTERVAL_MS + 200U)) ||
            ((first == 0U) && ((g_run.end_ms - enabled) < INTERVAL_MS))) {
            continue;
        }
        /* Sent in the pass that turned the payload on (its console line may arrive a little later) */
        if ((first == 0U) || (first > (enabled + ARRIVAL_SLACK_MS))) {
            fail_msg("payload on at +%llu ms, first telemetry at +%lld ms",
                     (unsigned long long)(enabled - g_run.start_ms),
                     (long long)first - (long long)g_run.start_ms);
        }
        resumed++;
    }
    /* The run's power model gives at least one outage longer than an interval */
    assert_true(resumed >= 1U);

    /* Nothing between a disable and the next enable */
    size_t outages = 0U;
    for (size_t d = console_find("[EPS] Payload disabled", 0U); d < g_run.n_console;
         d = console_find("[EPS] Payload disabled", d + 1U)) {
        uint64_t off = g_run.console[d].at_ms;
        size_t e = console_find("[EPS] Payload enabled", d);
        uint64_t on = (e < g_run.n_console) ? g_run.console[e].at_ms : g_run.end_ms;
        for (size_t i = 0U; i < g_run.n_jetson; i++) {
            uint64_t t = telemetry_time(&g_run.jetson[i]);
            if (is_telemetry(&g_run.jetson[i]) && (t > off) && ((t + ARRIVAL_SLACK_MS) < on)) {
                fail_msg("telemetry at +%llu ms with the payload off", (unsigned long long)(t - g_run.start_ms));
            }
        }
        outages++;
    }
    assert_true(outages >= 1U);
}

/** The STATUS answer reaches the executive (health, junction temperature, power) */
static void test_status_reply_is_taken(void **state)
{
    (void)state;
    assert_int_equal(console_count("[UART] Jetson status: health=0 temp=45C power=6800mW"), 1U);
}

/**
 * Only the answer to the latest request is kept, only with a good CRC:
 * the answers to requests 0 (never made) and 3 (after 4 went out) are
 * dropped, the corrupted one is rejected, those to 3 and 5 are taken
 */
static void test_beacon_answers(void **state)
{
    (void)state;
    char want[96];
    assert_int_equal(console_count("[UART] Beacon text from Jetson ("), 2U);
    (void)snprintf(want, sizeof(want), "[UART] Beacon text from Jetson (%zu chars)", strlen(TEXT_A));
    size_t a = console_find(want, 0U);
    assert_true(a < g_run.n_console);
    (void)snprintf(want, sizeof(want), "[UART] Beacon text from Jetson (%zu chars)", strlen(TEXT_B));
    assert_true(console_find(want, a + 1U) < g_run.n_console);
    assert_int_equal(console_count("[UART] Rejected Jetson line (2)"), 1U);
    assert_int_equal(console_count("[UART] Rejected Jetson line"), 1U);
}

/*===========================================================================*/
/* Scenario "guard"                                                           */
/*===========================================================================*/

/**
 * The sensor fields fill the line, the status fields do not fit, and the
 * comma after the last sensor field is trimmed before the CRC: each field
 * complete, none empty, the sensors in table order
 */
static void test_long_telemetry_line_is_trimmed(void **state)
{
    (void)state;
    char copy[JETSON_TEXT_CAP];
    char *f[MAX_FIELDS];
    size_t checked = 0U;
    for (size_t i = 0U; i < g_run.n_jetson; i++) {
        if (!is_telemetry(&g_run.jetson[i])) {
            continue;
        }
        assert_true(g_run.jetson[i].len >= 450U);
        size_t n = split_fields(&g_run.jetson[i], copy, sizeof(copy), f);
        check_telemetry_header(f, n);
        assert_true(starts_with(f[6], "BUSV="));
        assert_true(starts_with(f[7], "JPC="));
        assert_true(starts_with(f[8], "PT="));
        char want[48];
        for (size_t k = 9U; k < n; k++) {
            (void)snprintf(want, sizeof(want), "F%02zu=", k - 8U);
            assert_true(starts_with(f[k], want));
            (void)snprintf(want, sizeof(want), "filler_volts_%02zu", k - 8U);
            size_t len = strlen(f[k]);
            assert_true((len > strlen(want)) && (strcmp(&f[k][len - strlen(want)], want) == 0));
        }
        /* Some filler fields did not fit either: the line was full */
        assert_true((n - 9U) < FILLER_SENSORS);
        checked++;
    }
    assert_true(checked >= 2U);
}

/**
 * The Jetson's junction temperature (STATUS) is an input of the payload
 * guard: 99 C trips it, the rail goes off, and no telemetry follows
 */
static void test_hot_status_trips_the_payload_guard(void **state)
{
    (void)state;
    size_t s = console_find("[UART] Jetson status: health=1 temp=99C power=5000mW", 0U);
    assert_true(s < g_run.n_console);
    size_t trip = console_find("Jetson junction over limit: rail off", s);
    assert_true(trip < g_run.n_console);
    assert_true(console_find("[EPS] Payload disabled", trip) < g_run.n_console);
    assert_true(g_run.trip_ms != 0U);
    assert_true(console_find("[EPS] Payload enabled", trip) == g_run.n_console);
    for (size_t i = 0U; i < g_run.n_jetson; i++) {
        if (is_telemetry(&g_run.jetson[i]) && (telemetry_time(&g_run.jetson[i]) > g_run.trip_ms)) {
            fail_msg("line after the trip: %s", g_run.jetson[i].text);
        }
    }
    /* Long enough with the payload off to show it */
    assert_true(g_run.end_ms > (g_run.trip_ms + (5U * INTERVAL_MS)));
}

/**
 * The periodic UART health check: the line hung up away from a window is
 * noticed in the next window (now % 10 s < 100 ms), in its first pass,
 * once, and the module tries to reopen it
 */
static void test_health_check_finds_the_dead_line_in_its_window(void **state)
{
    (void)state;
    assert_false(g_run.close_unschedulable);
    assert_true(g_run.close_ms != 0U);
    size_t h = console_find("[UART] Health check failed, attempting recovery", 0U);
    if (h == g_run.n_console) {
        fail_msg("no health check failure after the line closed at phase %llu ms",
                 (unsigned long long)(g_run.close_ms % HEALTH_PERIOD_MS));
    }
    uint64_t at = g_run.console[h].at_ms;
    if ((at < g_run.window_ms) || (at > (g_run.window_ms + HEALTH_FIRST_PASS_MS))) {
        fail_msg("health check failure at phase %llu ms (+%lld ms from the window)",
                 (unsigned long long)(at % HEALTH_PERIOD_MS), (long long)at - (long long)g_run.window_ms);
    }
    assert_int_equal(console_count("[UART] Health check failed"), 1U);
    assert_true(console_find("[UART] Failed to open", h) < g_run.n_console);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(int argc, char **argv)
{
    if ((argc != 4) || ((strcmp(argv[2], "link") != 0) && (strcmp(argv[2], "guard") != 0))) {
        fprintf(stderr, "usage: %s <smart_qso_flight> link|guard <sensors.yaml>\n", argv[0]);
        return 2;
    }
    g_sim = argv[1];
    g_sensors = argv[3];
    g_run.guard = (strcmp(argv[2], "guard") == 0);
    g_run.exit_status = -1;

    const struct CMUnitTest link_tests[] = {
        cmocka_unit_test(test_runs_and_shuts_down_cleanly),
        cmocka_unit_test(test_lines_are_sealed_icd_lines),
        cmocka_unit_test(test_each_telemetry_is_followed_by_a_beacon_request),
        cmocka_unit_test(test_telemetry_fields),
        cmocka_unit_test(test_telemetry_at_the_interval),
        cmocka_unit_test(test_telemetry_follows_the_payload_power),
        cmocka_unit_test(test_status_reply_is_taken),
        cmocka_unit_test(test_beacon_answers),
    };
    const struct CMUnitTest guard_tests[] = {
        cmocka_unit_test(test_runs_and_shuts_down_cleanly),
        cmocka_unit_test(test_lines_are_sealed_icd_lines),
        cmocka_unit_test(test_each_telemetry_is_followed_by_a_beacon_request),
        cmocka_unit_test(test_long_telemetry_line_is_trimmed),
        cmocka_unit_test(test_telemetry_at_the_interval),
        cmocka_unit_test(test_hot_status_trips_the_payload_guard),
        cmocka_unit_test(test_health_check_finds_the_dead_line_in_its_window),
    };
    if (g_run.guard) {
        return cmocka_run_group_tests_name("Executive Jetson link: payload guard and UART health",
                                           guard_tests, run_simulation, NULL);
    }
    return cmocka_run_group_tests_name("Executive Jetson link: telemetry and answers", link_tests,
                                       run_simulation, NULL);
}
