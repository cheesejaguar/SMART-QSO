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
 * @brief Make every subsequent hal_flash_read() fail with SMART_QSO_ERROR_IO
 *        (a dead FRAM), or work again
 */
void hal_sim_flash_fail_reads(bool fail);

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
 * @brief The internal pull a pin was last configured with (HalGpioPull_t)
 *
 * On DEPLOY_SENSE and SOLAR_SENSE, which the board drives through a weak
 * source, a configured internal pull decides the level read, as on the
 * board (RID SMA-B02).
 */
uint32_t hal_sim_gpio_pull(HalGpioPin_t pin);

/**
 * @brief When the GPIO_PIN_XPDR_EN heartbeat stops, or stopped
 *
 * On the HAL time base (hal_timer_get_us()), microseconds; 0 if the
 * heartbeat has not run since hal_gpio_init(), or for another pin. The
 * radio board's monostable holds the transponder enabled until 50 ms after
 * this (docs/hardware/BUILD_VS_BUY.md §3.2).
 */
uint64_t hal_sim_gpio_heartbeat_end_us(HalGpioPin_t pin);

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

/**
 * @brief The simulated EPS controller's end-of-life fire: with the arm latch
 *        set (the key at EOL_KEY 0x7E, then an EOL_ARM rising edge) and
 *        EOL_FIRE held this long, EOL_STATE reports the charge path open
 *        (bit 1 set, bit 2 F1 intact clear); 0: never. 30000 after start-up
 */
void hal_sim_eps_eol_open_ms(uint32_t ms);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_SIM_H */
