/**
 * @file uart_comm.h
 * @brief UART Communication Interface
 *
 * This module provides UART communication functionality for OBC-Jetson
 * communication including initialization, send/receive, and health monitoring.
 *
 * @requirement SRS-COMM-001 System shall communicate with Jetson payload via UART
 * @requirement SRS-COMM-002 System shall detect and recover from UART failures
 * @requirement SRS-COMM-003 System shall support configurable UART parameters
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#ifndef SMART_QSO_UART_COMM_H
#define SMART_QSO_UART_COMM_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* UART Configuration Constants                                               */
/*===========================================================================*/

/** Default UART device path */
#define UART_DEVICE_DEFAULT "/dev/ttyUSB0"

/** Maximum UART device path length */
#define UART_DEVICE_MAX_LEN 64

/** Default baud rate identifier */
#define UART_BAUDRATE_DEFAULT 115200

/** Maximum transmit buffer size */
#define UART_TX_BUFFER_SIZE 2048

/** Maximum receive buffer size */
#define UART_RX_BUFFER_SIZE 1024

/*===========================================================================*/
/* UART Configuration Structure                                               */
/*===========================================================================*/

/**
 * @brief UART Configuration
 *
 * Holds UART port configuration parameters.
 */
typedef struct {
    char device[UART_DEVICE_MAX_LEN];  /**< Device path */
    int baudrate;                       /**< Baud rate */
    uint32_t timeout_ms;                /**< Read timeout in milliseconds */
} UartConfig_t;

/*===========================================================================*/
/* UART Communication API                                                     */
/*===========================================================================*/

/**
 * @brief Initialize UART module with default or environment configuration
 *
 * Parses environment variables for UART configuration:
 * - UART_DEVICE: Device path
 * - UART_BAUDRATE: Baud rate
 * - TELEMETRY_INTERVAL_MS: Telemetry transmission interval
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre None
 * @post UART is initialized or error is logged
 */
SmartQsoResult_t uart_init(void);

/**
 * @brief Initialize UART with specific configuration
 *
 * @param config Pointer to configuration structure
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre config != NULL
 * @post UART is initialized with specified configuration
 */
SmartQsoResult_t uart_init_with_config(const UartConfig_t *config);

/**
 * @brief Close UART port
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @post UART port is closed
 */
SmartQsoResult_t uart_close(void);

/**
 * @brief Check if UART is initialized
 *
 * @return true if UART is initialized, false otherwise
 */
bool uart_is_initialized(void);

/**
 * @brief Send data over UART
 *
 * @param data Pointer to data buffer
 * @param len  Length of data to send
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre data != NULL
 * @pre len > 0
 * @post Data is transmitted or error is logged
 */
SmartQsoResult_t uart_send(const char *data, size_t len);

/**
 * @brief Receive data from UART
 *
 * @param[out] buffer    Buffer to store received data
 * @param      buffer_len Size of buffer
 * @param[out] received  Number of bytes actually received
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre buffer != NULL
 * @pre buffer_len > 0
 * @pre received != NULL
 */
SmartQsoResult_t uart_receive(char *buffer, size_t buffer_len, size_t *received);

/**
 * @brief Check UART health and attempt recovery if needed
 *
 * @return SMART_QSO_OK if healthy, error code if unhealthy
 *
 * @note May reinitialize UART if health check fails
 */
SmartQsoResult_t uart_check_health(void);

/**
 * @brief Get current UART configuration
 *
 * @param[out] config Pointer to configuration structure to fill
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre config != NULL
 */
SmartQsoResult_t uart_get_config(UartConfig_t *config);

/**
 * @brief Get telemetry transmission interval
 *
 * @return Telemetry interval in milliseconds
 */
uint32_t uart_get_telemetry_interval(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_UART_COMM_H */
