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

/**
 * @brief Make every subsequent hal_flash_write() fail with SMART_QSO_ERROR_IO
 *
 * @param[in] fail true to inject write failures, false to restore normal writes
 */
void hal_sim_flash_fail_writes(bool fail);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_SIM_H */
