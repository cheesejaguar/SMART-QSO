/**
 * @file hal_sim.h
 * @brief Simulation-only HAL hooks for fault injection in tests
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Available only in the host simulation HAL (src/hal/hal_sim.c). Flight HAL
 * implementations do not provide these functions.
 */

#ifndef SMART_QSO_HAL_SIM_H
#define SMART_QSO_HAL_SIM_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "smart_qso.h"
#include "hal/hal_gpio.h"
#include "hal/hal_uart.h"
#include "hal/hal_reset.h"
#include "hal/hal_adc.h"

/**
 * @brief Make every subsequent hal_flash_write() fail with SMART_QSO_ERROR_IO
 *
 * @param[in] fail true to inject write failures, false to restore normal writes
 */
void hal_sim_flash_fail_writes(bool fail);

/**
 * @brief Let the next writes succeed, then fail every later write
 *
 * Simulates power loss part-way through a multi-write update.
 *
 * @param[in] writes Number of writes that still succeed; -1 disables
 */
void hal_sim_flash_fail_writes_after(int32_t writes);

/**
 * @brief Drive the level seen on an input pin (e.g. the umbilical sense line)
 */
void hal_sim_gpio_drive_input(HalGpioPin_t pin, bool state);

/**
 * @brief Queue bytes for hal_uart_read() on a port, as if received
 *
 * @return SMART_QSO_OK, or SMART_QSO_ERROR_NO_MEM if the FIFO is full
 */
SmartQsoResult_t hal_sim_uart_inject(HalUartPort_t port, const uint8_t *data, size_t len);

/**
 * @brief Queue a frame for hal_radio_receive(), as if received on the uplink
 *
 * @return SMART_QSO_OK, or SMART_QSO_ERROR_NO_MEM if the queue is full
 */
SmartQsoResult_t hal_sim_radio_inject(const uint8_t *frame, size_t len);

/**
 * @brief Drop every queued receiver frame
 */
void hal_sim_radio_clear(void);

/**
 * @brief Take the oldest frame given to hal_radio_transmit() (the newest
 *        four are kept)
 *
 * @return SMART_QSO_OK with a frame; SMART_QSO_ERROR_TIMEOUT if none
 */
SmartQsoResult_t hal_sim_radio_take_tx(uint8_t *frame, size_t max_len, size_t *len);

/**
 * @brief Make hal_radio_transmit() return @p result from now on
 *        (SMART_QSO_OK: it takes frames again); kept frames are dropped
 */
void hal_sim_radio_tx_result(SmartQsoResult_t result);

/**
 * @brief Set one simulated ADC input, in its own unit, until hal_adc_init()
 *        restores the defaults
 */
SmartQsoResult_t hal_sim_adc_set(HalAdcChannel_t channel, double value);

/**
 * @brief Set the cause hal_reset_cause() reports, modelling the next boot
 */
void hal_sim_set_reset_cause(HalResetCause_t cause);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_SIM_H */
