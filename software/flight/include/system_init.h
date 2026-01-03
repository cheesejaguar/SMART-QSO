/**
 * @file system_init.h
 * @brief System initialization module for SMART-QSO CubeSat
 *
 * This module provides hardware initialization, clock configuration,
 * boot reason determination, and self-test execution per NASA Ames
 * SmallSat heritage practices.
 *
 * @note MISRA C:2012 compliant
 * @note NPR 7150.2 Class B software requirements
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#ifndef SYSTEM_INIT_H
#define SYSTEM_INIT_H

#ifdef __cplusplus
extern "C" {
#endif

/*******************************************************************************
 * Includes
 ******************************************************************************/

#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Definitions
 ******************************************************************************/

/** System initialization magic number for validation */
#define SYSTEM_INIT_MAGIC           0x534D5154U  /* "SMQT" */

/** Maximum boot count before forcing safe mode */
#define SYSTEM_MAX_BOOT_COUNT       10U

/** Watchdog timeout in milliseconds */
#define SYSTEM_WATCHDOG_TIMEOUT_MS  5000U

/** Self-test timeout in milliseconds */
#define SYSTEM_SELFTEST_TIMEOUT_MS  30000U

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Boot reason enumeration
 *
 * Identifies why the system was reset, critical for fault analysis
 * and mission operations per NASA-HDBK-4008A guidelines.
 */
typedef enum {
    BOOT_REASON_POWER_ON = 0,       /**< Initial power-on (cold boot) */
    BOOT_REASON_WATCHDOG,           /**< Watchdog timer reset */
    BOOT_REASON_SOFTWARE,           /**< Commanded software reset */
    BOOT_REASON_BROWNOUT,           /**< Brownout detector reset */
    BOOT_REASON_PIN_RESET,          /**< External pin reset */
    BOOT_REASON_LOCKUP,             /**< CPU lockup reset */
    BOOT_REASON_UNKNOWN             /**< Unknown reset cause */
} boot_reason_t;

/**
 * @brief System initialization status
 */
typedef enum {
    INIT_OK = 0,                    /**< Initialization successful */
    INIT_ERROR_CLOCK,               /**< Clock configuration failed */
    INIT_ERROR_PERIPHERAL,          /**< Peripheral init failed */
    INIT_ERROR_WATCHDOG,            /**< Watchdog init failed */
    INIT_ERROR_MEMORY,              /**< Memory test failed */
    INIT_ERROR_CONFIG,              /**< Configuration load failed */
    INIT_ERROR_SELFTEST,            /**< Self-test failed */
    INIT_ERROR_FATAL                /**< Fatal initialization error */
} init_status_t;

/**
 * @brief Self-test results structure
 */
typedef struct {
    bool cpu_test_passed;           /**< CPU instruction test */
    bool ram_test_passed;           /**< RAM pattern test */
    bool flash_crc_passed;          /**< Flash CRC verification */
    bool watchdog_test_passed;      /**< Watchdog responsiveness */
    bool clock_test_passed;         /**< Clock frequency check */
    bool gpio_test_passed;          /**< GPIO loopback test */
    bool adc_test_passed;           /**< ADC reference test */
    bool i2c_test_passed;           /**< I2C bus scan */
    bool uart_test_passed;          /**< UART loopback test */
    bool spi_test_passed;           /**< SPI loopback test */
    uint32_t test_duration_ms;      /**< Total test time */
} selftest_results_t;

/**
 * @brief System initialization context
 */
typedef struct {
    uint32_t magic;                 /**< Validation magic number */
    boot_reason_t boot_reason;      /**< Reason for current boot */
    uint32_t boot_count;            /**< Total boot count */
    uint32_t last_boot_time;        /**< Unix timestamp of last boot */
    init_status_t init_status;      /**< Current init status */
    selftest_results_t selftest;    /**< Self-test results */
    uint32_t clock_freq_hz;         /**< System clock frequency */
    uint32_t uptime_seconds;        /**< Seconds since boot */
} system_context_t;

/**
 * @brief Initialization stage callback type
 *
 * @param stage Current initialization stage (0-based)
 * @param name Stage name string
 */
typedef void (*init_progress_cb_t)(uint8_t stage, const char *name);

/*******************************************************************************
 * Public Function Prototypes
 ******************************************************************************/

/**
 * @brief Initialize the system
 *
 * Performs complete system initialization in the following order:
 * 1. Early hardware init (clocks, power)
 * 2. Peripheral initialization
 * 3. Watchdog initialization
 * 4. Memory tests (if enabled)
 * 5. Configuration load
 * 6. Boot reason determination
 * 7. Self-test execution
 *
 * @return INIT_OK on success, error code otherwise
 */
init_status_t system_init(void);

/**
 * @brief Initialize system clocks
 *
 * Configures system and peripheral clocks per hardware design.
 *
 * @return INIT_OK on success
 */
init_status_t system_init_clocks(void);

/**
 * @brief Initialize all peripherals
 *
 * Initializes GPIO, UART, SPI, I2C, ADC, timers.
 *
 * @return INIT_OK on success
 */
init_status_t system_init_peripherals(void);

/**
 * @brief Initialize and start watchdog timer
 *
 * @param timeout_ms Watchdog timeout in milliseconds
 * @return INIT_OK on success
 */
init_status_t system_init_watchdog(uint32_t timeout_ms);

/**
 * @brief Perform memory self-test
 *
 * Runs pattern tests on SRAM (non-destructive for stack/heap).
 *
 * @return INIT_OK if all tests pass
 */
init_status_t system_memory_test(void);

/**
 * @brief Load configuration from non-volatile storage
 *
 * Loads system configuration from FRAM/Flash and validates CRC.
 *
 * @return INIT_OK on success
 */
init_status_t system_load_config(void);

/**
 * @brief Determine boot reason from hardware registers
 *
 * Reads reset status registers to determine why system rebooted.
 *
 * @return Boot reason enumeration value
 */
boot_reason_t system_get_boot_reason(void);

/**
 * @brief Execute comprehensive self-tests
 *
 * Runs all built-in self-tests (BIST) and populates results.
 *
 * @param results Pointer to store test results
 * @return INIT_OK if all critical tests pass
 */
init_status_t system_run_selftests(selftest_results_t *results);

/**
 * @brief Get system context
 *
 * Returns pointer to read-only system context structure.
 *
 * @return Pointer to system context (never NULL)
 */
const system_context_t *system_get_context(void);

/**
 * @brief Get boot count
 *
 * @return Number of times system has booted
 */
uint32_t system_get_boot_count(void);

/**
 * @brief Get uptime in seconds
 *
 * @return Seconds since system boot
 */
uint32_t system_get_uptime(void);

/**
 * @brief Request software reset
 *
 * Logs reset reason and triggers controlled reset.
 *
 * @param reason String describing reset reason (max 32 chars)
 */
void system_request_reset(const char *reason);

/**
 * @brief Register progress callback
 *
 * Optional callback for initialization progress reporting.
 *
 * @param callback Function to call during init stages
 */
void system_register_progress_callback(init_progress_cb_t callback);

/**
 * @brief Convert init status to string
 *
 * @param status Status code
 * @return Human-readable string (never NULL)
 */
const char *system_status_to_string(init_status_t status);

/**
 * @brief Convert boot reason to string
 *
 * @param reason Boot reason code
 * @return Human-readable string (never NULL)
 */
const char *system_boot_reason_to_string(boot_reason_t reason);

/**
 * @brief Check if system should enter safe mode
 *
 * Returns true if boot count exceeds threshold or critical
 * self-test failures occurred.
 *
 * @return true if safe mode required
 */
bool system_should_enter_safe_mode(void);

/**
 * @brief Pet (refresh) the watchdog timer
 *
 * Must be called periodically to prevent watchdog reset.
 */
void system_pet_watchdog(void);

#ifdef __cplusplus
}
#endif

#endif /* SYSTEM_INIT_H */
