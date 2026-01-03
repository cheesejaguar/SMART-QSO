/**
 * @file uart_comm.c
 * @brief UART Communication Implementation
 *
 * @requirement SRS-COMM-001 System shall communicate with Jetson payload via UART
 * @requirement SRS-COMM-002 System shall detect and recover from UART failures
 * @requirement SRS-COMM-003 System shall support configurable UART parameters
 */

/* Required for CRTSCTS and other BSD/POSIX extensions */
#define _DEFAULT_SOURCE

#include "uart_comm.h"
#include "fault_mgmt.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>

#ifdef SIMULATION_BUILD
#include <termios.h>
#endif

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** UART file descriptor */
static int s_uart_fd = -1;

/** Current UART configuration */
static UartConfig_t s_config = {
    .device = UART_DEVICE_DEFAULT,
    .baudrate = UART_BAUDRATE_DEFAULT,
    .timeout_ms = 1000
};

/** Telemetry interval in milliseconds */
static uint32_t s_telemetry_interval_ms = SMART_QSO_TELEMETRY_INTERVAL_MS;

/** Flag indicating if UART is initialized */
static bool s_initialized = false;

/*===========================================================================*/
/* Internal Functions                                                         */
/*===========================================================================*/

#ifdef SIMULATION_BUILD
/**
 * @brief Convert baudrate integer to termios constant
 */
static speed_t baudrate_to_speed(int baudrate)
{
    switch (baudrate) {
        case 9600:   return B9600;
        case 19200:  return B19200;
        case 38400:  return B38400;
        case 57600:  return B57600;
        case 115200: return B115200;
        case 230400: return B230400;
        default:     return B115200;
    }
}
#endif

/**
 * @brief Parse UART configuration from environment variables
 */
static void parse_env_config(void)
{
    const char *device = getenv("UART_DEVICE");
    if (device != NULL && strlen(device) < UART_DEVICE_MAX_LEN) {
        strncpy(s_config.device, device, UART_DEVICE_MAX_LEN - 1);
        s_config.device[UART_DEVICE_MAX_LEN - 1] = '\0';
    }

    const char *baudrate = getenv("UART_BAUDRATE");
    if (baudrate != NULL) {
        int baud = atoi(baudrate);
        switch (baud) {
            case 9600:
            case 19200:
            case 38400:
            case 57600:
            case 115200:
            case 230400:
                s_config.baudrate = baud;
                break;
            default:
                fprintf(stderr, "[UART] Invalid baudrate %s, using default\n", baudrate);
                break;
        }
    }

    const char *interval = getenv("TELEMETRY_INTERVAL_MS");
    if (interval != NULL) {
        int ms = atoi(interval);
        if (ms > 0 && ms <= 3600000) {
            s_telemetry_interval_ms = (uint32_t)ms;
        } else {
            fprintf(stderr, "[UART] Invalid interval %s, using default\n", interval);
        }
    }

    printf("[UART] Configuration: device=%s, baudrate=%d, interval=%ums\n",
           s_config.device, s_config.baudrate, s_telemetry_interval_ms);
}

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

SmartQsoResult_t uart_init(void)
{
    if (s_initialized) {
        return SMART_QSO_OK;
    }

    parse_env_config();
    return uart_init_with_config(&s_config);
}

SmartQsoResult_t uart_init_with_config(const UartConfig_t *config)
{
    SMART_QSO_REQUIRE_NOT_NULL(config);

    if (s_initialized) {
        (void)uart_close();
    }

    s_config = *config;

#ifdef SIMULATION_BUILD
    s_uart_fd = open(s_config.device, O_RDWR | O_NOCTTY | O_SYNC);
    if (s_uart_fd < 0) {
        fprintf(stderr, "[UART] Failed to open %s: %s\n",
                s_config.device, strerror(errno));
        return SMART_QSO_ERROR_IO;
    }

    struct termios tty;
    if (tcgetattr(s_uart_fd, &tty) != 0) {
        fprintf(stderr, "[UART] Failed to get attributes: %s\n", strerror(errno));
        close(s_uart_fd);
        s_uart_fd = -1;
        return SMART_QSO_ERROR_IO;
    }

    speed_t speed = baudrate_to_speed(s_config.baudrate);
    if (cfsetospeed(&tty, speed) != 0) {
        fprintf(stderr, "[UART] Failed to set output speed\n");
    }
    if (cfsetispeed(&tty, speed) != 0) {
        fprintf(stderr, "[UART] Failed to set input speed\n");
    }

    /* 8N1 configuration */
    tty.c_cflag = (tty.c_cflag & ~(tcflag_t)CSIZE) | CS8;
    tty.c_cflag &= ~(tcflag_t)(PARENB | PARODD);
    tty.c_cflag &= ~(tcflag_t)CSTOPB;
    tty.c_cflag &= ~(tcflag_t)CRTSCTS;
    tty.c_cflag |= CREAD | CLOCAL;

    /* Raw input */
    tty.c_lflag &= ~(tcflag_t)(ICANON | ECHO | ECHOE | ISIG);

    /* Raw output */
    tty.c_oflag &= ~(tcflag_t)OPOST;

    if (tcsetattr(s_uart_fd, TCSANOW, &tty) != 0) {
        fprintf(stderr, "[UART] Failed to set attributes: %s\n", strerror(errno));
        close(s_uart_fd);
        s_uart_fd = -1;
        return SMART_QSO_ERROR_IO;
    }
#else
    /* Flight hardware implementation placeholder */
    /* TODO: Implement HAL-based UART initialization */
#endif

    s_initialized = true;
    fprintf(stderr, "[UART] Initialized %s at %d baud\n",
            s_config.device, s_config.baudrate);
    return SMART_QSO_OK;
}

SmartQsoResult_t uart_close(void)
{
    if (s_uart_fd >= 0) {
        close(s_uart_fd);
        s_uart_fd = -1;
    }
    s_initialized = false;
    return SMART_QSO_OK;
}

bool uart_is_initialized(void)
{
    return s_initialized;
}

SmartQsoResult_t uart_send(const char *data, size_t len)
{
    SMART_QSO_REQUIRE_NOT_NULL(data);
    SMART_QSO_REQUIRE(len > 0, "Invalid length");

    if (!s_initialized || s_uart_fd < 0) {
        return SMART_QSO_ERROR;
    }

    ssize_t written = write(s_uart_fd, data, len);
    if (written != (ssize_t)len) {
        fprintf(stderr, "[UART] Write failed: %s\n", strerror(errno));

        /* Try to reinitialize UART on error */
        (void)uart_close();
        SmartQsoResult_t result = uart_init_with_config(&s_config);
        if (result != SMART_QSO_OK) {
            return SMART_QSO_ERROR_IO;
        }

        /* Retry once */
        written = write(s_uart_fd, data, len);
        if (written != (ssize_t)len) {
            fprintf(stderr, "[UART] Retry failed: %s\n", strerror(errno));
            return SMART_QSO_ERROR_IO;
        }
    }

#ifdef SIMULATION_BUILD
    /* Ensure data is transmitted */
    if (tcdrain(s_uart_fd) != 0) {
        fprintf(stderr, "[UART] tcdrain failed: %s\n", strerror(errno));
    }
#endif

    return SMART_QSO_OK;
}

SmartQsoResult_t uart_receive(char *buffer, size_t buffer_len, size_t *received)
{
    SMART_QSO_REQUIRE_NOT_NULL(buffer);
    SMART_QSO_REQUIRE(buffer_len > 0, "Invalid buffer length");
    SMART_QSO_REQUIRE_NOT_NULL(received);

    if (!s_initialized || s_uart_fd < 0) {
        *received = 0;
        return SMART_QSO_ERROR;
    }

    ssize_t read_bytes = read(s_uart_fd, buffer, buffer_len);
    if (read_bytes < 0) {
        *received = 0;
        return SMART_QSO_ERROR_IO;
    }

    *received = (size_t)read_bytes;
    return SMART_QSO_OK;
}

SmartQsoResult_t uart_check_health(void)
{
    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

#ifdef SIMULATION_BUILD
    /* Simple health check - try to get UART attributes */
    struct termios tty;
    if (tcgetattr(s_uart_fd, &tty) != 0) {
        fprintf(stderr, "[UART] Health check failed, attempting recovery\n");
        fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_WARNING,
                      "UART health check failed", 0.0);
        (void)uart_close();
        return uart_init_with_config(&s_config);
    }
#endif

    return SMART_QSO_OK;
}

SmartQsoResult_t uart_get_config(UartConfig_t *config)
{
    SMART_QSO_REQUIRE_NOT_NULL(config);
    *config = s_config;
    return SMART_QSO_OK;
}

uint32_t uart_get_telemetry_interval(void)
{
    return s_telemetry_interval_ms;
}
