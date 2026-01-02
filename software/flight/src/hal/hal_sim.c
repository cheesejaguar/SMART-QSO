/**
 * @file hal_sim.c
 * @brief HAL Simulation Implementation
 *
 * Provides simulation implementations of all HAL interfaces for testing
 * on x86/ARM host systems. This allows the flight software to be tested
 * without actual flight hardware.
 *
 * @requirement SRS-Q022 Software shall support simulation and flight builds
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/time.h>

#include "hal/hal.h"

#ifdef HAL_TARGET_SIMULATION

/*===========================================================================*/
/* Simulation State                                                           */
/*===========================================================================*/

/* GPIO simulation state */
static bool s_gpio_initialized = false;
static bool s_gpio_state[GPIO_PIN_COUNT];
static HalGpioDir_t s_gpio_dir[GPIO_PIN_COUNT];

/* I2C simulation state */
static bool s_i2c_initialized[I2C_BUS_COUNT];

/* Simulated I2C devices */
typedef struct {
    uint8_t addr;
    uint8_t registers[256];
} SimI2cDevice_t;

static SimI2cDevice_t s_i2c_devices[] = {
    { .addr = I2C_ADDR_EPS_CONTROLLER },
    { .addr = I2C_ADDR_BATTERY_MONITOR },
    { .addr = I2C_ADDR_SOLAR_MONITOR },
};
#define SIM_I2C_DEVICE_COUNT (sizeof(s_i2c_devices) / sizeof(s_i2c_devices[0]))

/* UART simulation state */
static bool s_uart_initialized[UART_PORT_COUNT];

/* SPI simulation state */
static bool s_spi_initialized[SPI_BUS_COUNT];

/* Timer simulation state */
static bool s_timer_initialized = false;
static uint64_t s_timer_start_us = 0;

/* ADC simulation state */
static bool s_adc_initialized = false;
static double s_adc_values[ADC_CHANNEL_COUNT] = {
    7.8,    /* VBATT */
    -0.5,   /* IBATT (discharging) */
    9.2,    /* VSOLAR */
    0.45,   /* ISOLAR */
    5.0,    /* VBUS */
    35.0,   /* TEMP_MCU */
    28.0    /* TEMP_BOARD */
};

/* Flash simulation state */
static bool s_flash_initialized = false;
static uint8_t *s_flash_data[FLASH_REGION_COUNT];
static size_t s_flash_sizes[FLASH_REGION_COUNT] = {
    512,    /* MISSION_DATA */
    256,    /* EPS_CONFIG */
    512,    /* SENSOR_CONFIG */
    4096,   /* FAULT_LOG */
    1024    /* BACKUP */
};

/* Watchdog simulation state */
static bool s_wdt_initialized = false;
static bool s_wdt_running = false;
static uint32_t s_wdt_timeout_ms = HAL_WDT_DEFAULT_TIMEOUT_MS;
static uint64_t s_wdt_last_kick_ms = 0;
static bool s_wdt_reset_flag = false;

/*===========================================================================*/
/* Helper Functions                                                           */
/*===========================================================================*/

static uint64_t get_time_us(void) {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (uint64_t)tv.tv_sec * 1000000ULL + (uint64_t)tv.tv_usec;
}

/*===========================================================================*/
/* HAL Initialization                                                         */
/*===========================================================================*/

SmartQsoResult_t hal_init(void) {
    SmartQsoResult_t result;

    /* Initialize timer first (needed by other modules) */
    result = hal_timer_init();
    if (result != SMART_QSO_OK) return result;

    /* Initialize GPIO */
    result = hal_gpio_init();
    if (result != SMART_QSO_OK) return result;

    /* Initialize flash */
    result = hal_flash_init();
    if (result != SMART_QSO_OK) return result;

    return SMART_QSO_OK;
}

SmartQsoResult_t hal_deinit(void) {
    /* Free flash simulation memory */
    for (int i = 0; i < FLASH_REGION_COUNT; i++) {
        if (s_flash_data[i]) {
            free(s_flash_data[i]);
            s_flash_data[i] = NULL;
        }
    }

    s_gpio_initialized = false;
    s_timer_initialized = false;
    s_flash_initialized = false;

    return SMART_QSO_OK;
}

/*===========================================================================*/
/* GPIO Implementation                                                        */
/*===========================================================================*/

SmartQsoResult_t hal_gpio_init(void) {
    memset(s_gpio_state, 0, sizeof(s_gpio_state));
    for (int i = 0; i < GPIO_PIN_COUNT; i++) {
        s_gpio_dir[i] = GPIO_DIR_INPUT;
    }
    s_gpio_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_gpio_config(HalGpioPin_t pin, HalGpioDir_t direction,
                                  HalGpioPull_t pull) {
    (void)pull;  /* Pull not simulated */

    if (!s_gpio_initialized) return SMART_QSO_ERROR;
    if (pin >= GPIO_PIN_COUNT) return SMART_QSO_ERROR_INVALID;

    s_gpio_dir[pin] = direction;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_gpio_set(HalGpioPin_t pin, bool state) {
    if (!s_gpio_initialized) return SMART_QSO_ERROR;
    if (pin >= GPIO_PIN_COUNT) return SMART_QSO_ERROR_INVALID;
    if (s_gpio_dir[pin] != GPIO_DIR_OUTPUT) return SMART_QSO_ERROR_INVALID;

    s_gpio_state[pin] = state;
    return SMART_QSO_OK;
}

bool hal_gpio_get(HalGpioPin_t pin) {
    if (!s_gpio_initialized || pin >= GPIO_PIN_COUNT) return false;
    return s_gpio_state[pin];
}

SmartQsoResult_t hal_gpio_toggle(HalGpioPin_t pin) {
    if (!s_gpio_initialized) return SMART_QSO_ERROR;
    if (pin >= GPIO_PIN_COUNT) return SMART_QSO_ERROR_INVALID;

    s_gpio_state[pin] = !s_gpio_state[pin];
    return SMART_QSO_OK;
}

/*===========================================================================*/
/* I2C Implementation                                                         */
/*===========================================================================*/

SmartQsoResult_t hal_i2c_init(HalI2cBus_t bus, HalI2cSpeed_t speed) {
    (void)speed;

    if (bus >= I2C_BUS_COUNT) return SMART_QSO_ERROR_INVALID;

    /* Initialize simulated device registers */
    for (size_t i = 0; i < SIM_I2C_DEVICE_COUNT; i++) {
        memset(s_i2c_devices[i].registers, 0, sizeof(s_i2c_devices[i].registers));

        /* Set some default values */
        if (s_i2c_devices[i].addr == I2C_ADDR_EPS_CONTROLLER) {
            s_i2c_devices[i].registers[0x00] = 0x43;  /* STATUS: charging, sunlit */
            s_i2c_devices[i].registers[0x01] = 0x01;  /* MODE: IDLE */
            s_i2c_devices[i].registers[0x16] = 75;    /* SOC: 75% */
        }
    }

    s_i2c_initialized[bus] = true;
    return SMART_QSO_OK;
}

static SimI2cDevice_t* find_i2c_device(uint8_t addr) {
    for (size_t i = 0; i < SIM_I2C_DEVICE_COUNT; i++) {
        if (s_i2c_devices[i].addr == addr) {
            return &s_i2c_devices[i];
        }
    }
    return NULL;
}

SmartQsoResult_t hal_i2c_write(HalI2cBus_t bus, uint8_t addr,
                                const uint8_t *data, size_t len) {
    if (bus >= I2C_BUS_COUNT || !s_i2c_initialized[bus]) return SMART_QSO_ERROR;
    if (!data || len == 0) return SMART_QSO_ERROR_NULL_PTR;

    SimI2cDevice_t *dev = find_i2c_device(addr);
    if (!dev) return SMART_QSO_ERROR;

    /* First byte is register address */
    uint8_t reg = data[0];
    for (size_t i = 1; i < len; i++) {
        dev->registers[reg++] = data[i];
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t hal_i2c_read(HalI2cBus_t bus, uint8_t addr,
                               uint8_t *data, size_t len) {
    if (bus >= I2C_BUS_COUNT || !s_i2c_initialized[bus]) return SMART_QSO_ERROR;
    if (!data || len == 0) return SMART_QSO_ERROR_NULL_PTR;

    SimI2cDevice_t *dev = find_i2c_device(addr);
    if (!dev) return SMART_QSO_ERROR;

    /* Read from register 0 by default */
    memcpy(data, dev->registers, len);
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_i2c_write_read(HalI2cBus_t bus, uint8_t addr,
                                     const uint8_t *write_data, size_t write_len,
                                     uint8_t *read_data, size_t read_len) {
    if (!write_data || write_len == 0) return SMART_QSO_ERROR_NULL_PTR;

    SimI2cDevice_t *dev = find_i2c_device(addr);
    if (!dev) return SMART_QSO_ERROR;

    /* First byte of write_data is register address */
    uint8_t reg = write_data[0];
    memcpy(read_data, &dev->registers[reg], read_len);

    return SMART_QSO_OK;
}

bool hal_i2c_device_present(HalI2cBus_t bus, uint8_t addr) {
    (void)bus;
    return find_i2c_device(addr) != NULL;
}

SmartQsoResult_t hal_i2c_recover(HalI2cBus_t bus) {
    (void)bus;
    return SMART_QSO_OK;
}

/*===========================================================================*/
/* UART Implementation                                                        */
/*===========================================================================*/

SmartQsoResult_t hal_uart_init(HalUartPort_t port, const HalUartConfig_t *config) {
    (void)config;

    if (port >= UART_PORT_COUNT) return SMART_QSO_ERROR_INVALID;

    s_uart_initialized[port] = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_uart_write(HalUartPort_t port, const uint8_t *data, size_t len) {
    if (port >= UART_PORT_COUNT || !s_uart_initialized[port]) return SMART_QSO_ERROR;
    if (!data) return SMART_QSO_ERROR_NULL_PTR;

    /* In simulation, just log to stdout for debug port */
    if (port == UART_PORT_DEBUG) {
        fwrite(data, 1, len, stdout);
        fflush(stdout);
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t hal_uart_read(HalUartPort_t port, uint8_t *data, size_t len,
                                uint32_t timeout_ms, size_t *bytes_read) {
    (void)timeout_ms;

    if (port >= UART_PORT_COUNT || !s_uart_initialized[port]) return SMART_QSO_ERROR;
    if (!data || !bytes_read) return SMART_QSO_ERROR_NULL_PTR;

    /* In simulation, return empty (no data available) */
    *bytes_read = 0;
    return SMART_QSO_ERROR_TIMEOUT;
}

SmartQsoResult_t hal_uart_read_line(HalUartPort_t port, char *buffer,
                                     size_t buffer_len, uint32_t timeout_ms,
                                     size_t *line_len) {
    (void)port;
    (void)buffer;
    (void)buffer_len;
    (void)timeout_ms;

    if (line_len) *line_len = 0;
    return SMART_QSO_ERROR_TIMEOUT;
}

size_t hal_uart_available(HalUartPort_t port) {
    (void)port;
    return 0;  /* No data in simulation */
}

SmartQsoResult_t hal_uart_flush(HalUartPort_t port) {
    (void)port;
    return SMART_QSO_OK;
}

/*===========================================================================*/
/* SPI Implementation                                                         */
/*===========================================================================*/

SmartQsoResult_t hal_spi_init(HalSpiBus_t bus, const HalSpiConfig_t *config) {
    (void)config;

    if (bus >= SPI_BUS_COUNT) return SMART_QSO_ERROR_INVALID;

    s_spi_initialized[bus] = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_spi_transfer(HalSpiBus_t bus, const uint8_t *tx_data,
                                   uint8_t *rx_data, size_t len) {
    if (bus >= SPI_BUS_COUNT || !s_spi_initialized[bus]) return SMART_QSO_ERROR;

    /* In simulation, just return zeros for received data */
    if (rx_data) {
        memset(rx_data, 0, len);
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t hal_spi_write(HalSpiBus_t bus, const uint8_t *data, size_t len) {
    return hal_spi_transfer(bus, data, NULL, len);
}

SmartQsoResult_t hal_spi_read(HalSpiBus_t bus, uint8_t *data, size_t len) {
    return hal_spi_transfer(bus, NULL, data, len);
}

SmartQsoResult_t hal_spi_cs_assert(HalSpiBus_t bus) {
    (void)bus;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_spi_cs_deassert(HalSpiBus_t bus) {
    (void)bus;
    return SMART_QSO_OK;
}

/*===========================================================================*/
/* Timer Implementation                                                       */
/*===========================================================================*/

SmartQsoResult_t hal_timer_init(void) {
    s_timer_start_us = get_time_us();
    s_timer_initialized = true;
    return SMART_QSO_OK;
}

uint64_t hal_timer_get_ms(void) {
    if (!s_timer_initialized) return 0;
    return (get_time_us() - s_timer_start_us) / 1000ULL;
}

uint64_t hal_timer_get_us(void) {
    if (!s_timer_initialized) return 0;
    return get_time_us() - s_timer_start_us;
}

void hal_timer_delay_ms(uint32_t ms) {
    usleep(ms * 1000);
}

void hal_timer_delay_us(uint32_t us) {
    usleep(us);
}

bool hal_timer_timeout(uint64_t start_ms, uint32_t timeout_ms) {
    return (hal_timer_get_ms() - start_ms) >= timeout_ms;
}

uint64_t hal_timer_elapsed_ms(uint64_t start_ms) {
    return hal_timer_get_ms() - start_ms;
}

SmartQsoResult_t hal_timer_start_periodic(uint32_t period_ms,
                                           HalTimerCallback_t callback,
                                           void *context,
                                           HalTimerId_t *timer_id) {
    (void)period_ms;
    (void)callback;
    (void)context;
    if (timer_id) *timer_id = 0;
    return SMART_QSO_OK;  /* Not implemented in simulation */
}

SmartQsoResult_t hal_timer_stop(HalTimerId_t timer_id) {
    (void)timer_id;
    return SMART_QSO_OK;
}

/*===========================================================================*/
/* ADC Implementation                                                         */
/*===========================================================================*/

SmartQsoResult_t hal_adc_init(HalAdcResolution_t resolution, HalAdcRef_t reference) {
    (void)resolution;
    (void)reference;

    s_adc_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_adc_read_raw(HalAdcChannel_t channel, uint16_t *value) {
    if (!s_adc_initialized) return SMART_QSO_ERROR;
    if (!value) return SMART_QSO_ERROR_NULL_PTR;
    if (channel >= ADC_CHANNEL_COUNT) return SMART_QSO_ERROR_INVALID;

    /* Convert simulated voltage to raw ADC value (assuming 12-bit, 3.3V ref) */
    double voltage = s_adc_values[channel];
    *value = (uint16_t)((voltage / 3.3) * 4095.0);

    return SMART_QSO_OK;
}

SmartQsoResult_t hal_adc_read_voltage(HalAdcChannel_t channel, double *voltage) {
    if (!s_adc_initialized) return SMART_QSO_ERROR;
    if (!voltage) return SMART_QSO_ERROR_NULL_PTR;
    if (channel >= ADC_CHANNEL_COUNT) return SMART_QSO_ERROR_INVALID;

    *voltage = s_adc_values[channel];
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_adc_read_multiple(const HalAdcChannel_t *channels,
                                        uint16_t *values, size_t count) {
    if (!channels || !values) return SMART_QSO_ERROR_NULL_PTR;

    for (size_t i = 0; i < count; i++) {
        SmartQsoResult_t result = hal_adc_read_raw(channels[i], &values[i]);
        if (result != SMART_QSO_OK) return result;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t hal_adc_calibrate(void) {
    return SMART_QSO_OK;  /* No calibration needed in simulation */
}

/*===========================================================================*/
/* Flash Implementation                                                       */
/*===========================================================================*/

SmartQsoResult_t hal_flash_init(void) {
    /* Allocate simulation memory for each region */
    for (int i = 0; i < FLASH_REGION_COUNT; i++) {
        if (!s_flash_data[i]) {
            s_flash_data[i] = (uint8_t *)calloc(1, s_flash_sizes[i]);
            if (!s_flash_data[i]) return SMART_QSO_ERROR_NO_MEM;
        }
    }

    s_flash_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_flash_read(HalFlashRegion_t region, uint32_t offset,
                                 uint8_t *data, size_t len) {
    if (!s_flash_initialized) return SMART_QSO_ERROR;
    if (!data) return SMART_QSO_ERROR_NULL_PTR;
    if (region >= FLASH_REGION_COUNT) return SMART_QSO_ERROR_INVALID;
    if (offset + len > s_flash_sizes[region]) return SMART_QSO_ERROR_INVALID;

    memcpy(data, &s_flash_data[region][offset], len);
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_flash_write(HalFlashRegion_t region, uint32_t offset,
                                  const uint8_t *data, size_t len) {
    if (!s_flash_initialized) return SMART_QSO_ERROR;
    if (!data) return SMART_QSO_ERROR_NULL_PTR;
    if (region >= FLASH_REGION_COUNT) return SMART_QSO_ERROR_INVALID;
    if (offset + len > s_flash_sizes[region]) return SMART_QSO_ERROR_INVALID;

    memcpy(&s_flash_data[region][offset], data, len);
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_flash_erase(HalFlashRegion_t region) {
    if (!s_flash_initialized) return SMART_QSO_ERROR;
    if (region >= FLASH_REGION_COUNT) return SMART_QSO_ERROR_INVALID;

    memset(s_flash_data[region], 0xFF, s_flash_sizes[region]);
    return SMART_QSO_OK;
}

size_t hal_flash_region_size(HalFlashRegion_t region) {
    if (region >= FLASH_REGION_COUNT) return 0;
    return s_flash_sizes[region];
}

uint32_t hal_flash_region_base(HalFlashRegion_t region) {
    (void)region;
    return 0;  /* Not applicable in simulation */
}

bool hal_flash_busy(void) {
    return false;  /* Never busy in simulation */
}

uint8_t hal_flash_wear_level(HalFlashRegion_t region) {
    (void)region;
    return 0;  /* No wear in simulation */
}

/*===========================================================================*/
/* Watchdog Implementation                                                    */
/*===========================================================================*/

SmartQsoResult_t hal_wdt_init(uint32_t timeout_ms, HalWdtMode_t mode) {
    (void)mode;

    if (timeout_ms < HAL_WDT_MIN_TIMEOUT_MS || timeout_ms > HAL_WDT_MAX_TIMEOUT_MS) {
        return SMART_QSO_ERROR_INVALID;
    }

    s_wdt_timeout_ms = timeout_ms;
    s_wdt_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_wdt_start(void) {
    if (!s_wdt_initialized) return SMART_QSO_ERROR;

    s_wdt_running = true;
    s_wdt_last_kick_ms = hal_timer_get_ms();
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_wdt_stop(void) {
    s_wdt_running = false;
    return SMART_QSO_OK;
}

void hal_wdt_kick(void) {
    if (s_wdt_running) {
        s_wdt_last_kick_ms = hal_timer_get_ms();
    }
}

uint32_t hal_wdt_remaining_ms(void) {
    if (!s_wdt_running) return s_wdt_timeout_ms;

    uint64_t elapsed = hal_timer_get_ms() - s_wdt_last_kick_ms;
    if (elapsed >= s_wdt_timeout_ms) return 0;
    return s_wdt_timeout_ms - (uint32_t)elapsed;
}

bool hal_wdt_caused_reset(void) {
    return s_wdt_reset_flag;
}

void hal_wdt_clear_reset_flag(void) {
    s_wdt_reset_flag = false;
}

SmartQsoResult_t hal_wdt_set_warning_callback(void (*callback)(void *context),
                                               void *context) {
    (void)callback;
    (void)context;
    return SMART_QSO_OK;  /* Not fully implemented in simulation */
}

#endif /* HAL_TARGET_SIMULATION */
