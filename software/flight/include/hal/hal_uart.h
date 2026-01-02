/**
 * @file hal_uart.h
 * @brief HAL UART Interface
 *
 * Provides UART abstraction for Jetson payload communication.
 *
 * @requirement SRS-I001 UART interface shall operate at 115200 baud, 8N1
 * @requirement SRS-I002 UART shall use LVTTL voltage levels (3.3V)
 */

#ifndef SMART_QSO_HAL_UART_H
#define SMART_QSO_HAL_UART_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* UART Configuration                                                         */
/*===========================================================================*/

/**
 * @brief UART port enumeration
 */
typedef enum {
    UART_PORT_JETSON = 0,   /**< Jetson payload communication */
    UART_PORT_DEBUG  = 1,   /**< Debug output (if available) */
    UART_PORT_COUNT
} HalUartPort_t;

/**
 * @brief UART baud rates
 */
typedef enum {
    UART_BAUD_9600   = 9600,
    UART_BAUD_19200  = 19200,
    UART_BAUD_38400  = 38400,
    UART_BAUD_57600  = 57600,
    UART_BAUD_115200 = 115200,
    UART_BAUD_230400 = 230400,
    UART_BAUD_460800 = 460800
} HalUartBaud_t;

/**
 * @brief UART parity
 */
typedef enum {
    UART_PARITY_NONE = 0,
    UART_PARITY_ODD  = 1,
    UART_PARITY_EVEN = 2
} HalUartParity_t;

/**
 * @brief UART configuration structure
 */
typedef struct {
    HalUartBaud_t baud_rate;
    uint8_t data_bits;      /**< 7 or 8 */
    uint8_t stop_bits;      /**< 1 or 2 */
    HalUartParity_t parity;
} HalUartConfig_t;

/*===========================================================================*/
/* UART Functions                                                             */
/*===========================================================================*/

/**
 * @brief Initialize UART port
 *
 * @param port   UART port to initialize
 * @param config Configuration parameters
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_uart_init(HalUartPort_t port, const HalUartConfig_t *config);

/**
 * @brief Write data to UART
 *
 * @param port UART port
 * @param data Data to write
 * @param len  Number of bytes to write
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_uart_write(HalUartPort_t port, const uint8_t *data, size_t len);

/**
 * @brief Read data from UART with timeout
 *
 * @param port       UART port
 * @param data       Buffer to store read data
 * @param len        Maximum bytes to read
 * @param timeout_ms Timeout in milliseconds (0 = non-blocking)
 * @param bytes_read Actual bytes read
 * @return SMART_QSO_OK on success, SMART_QSO_ERROR_TIMEOUT on timeout
 */
SmartQsoResult_t hal_uart_read(HalUartPort_t port, uint8_t *data, size_t len,
                                uint32_t timeout_ms, size_t *bytes_read);

/**
 * @brief Read line (until newline or timeout)
 *
 * @param port       UART port
 * @param buffer     Buffer to store line
 * @param buffer_len Buffer size
 * @param timeout_ms Timeout in milliseconds
 * @param line_len   Actual line length
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_uart_read_line(HalUartPort_t port, char *buffer,
                                     size_t buffer_len, uint32_t timeout_ms,
                                     size_t *line_len);

/**
 * @brief Check if data is available to read
 *
 * @param port UART port
 * @return Number of bytes available
 */
size_t hal_uart_available(HalUartPort_t port);

/**
 * @brief Flush UART receive buffer
 *
 * @param port UART port
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_uart_flush(HalUartPort_t port);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_UART_H */
