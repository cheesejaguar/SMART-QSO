/**
 * @file hal_stm32l4.c
 * @brief HAL for the STM32L4R5ZI OBC (register level; SW-01, compile-only)
 *
 * Implemented at register level (STM32L4 reference manual RM0432):
 *   - GPIO (RCC AHB2ENR, MODER, PUPDR, BSRR, IDR, ODR)
 *   - time base: SysTick at 1 kHz from the core clock
 *   - independent watchdog (IWDG, LSI 32 kHz)
 *   - reset cause (RCC CSR flags, read once and cleared)
 *
 * Not yet written; these return SMART_QSO_ERROR so the flight software fails
 * safe (for example, unreadable NVM leaves the transmitter inhibited):
 *   - FRAM non-volatile memory over SPI (hal_flash_*)
 *   - UART, I2C, SPI, ADC and the 435 MHz receiver (hal_radio_*)
 *
 * The pin table is PROVISIONAL. hardware/obc/OBC_DESIGN.md and
 * hardware/obc/STORAGE_DESIGN.md assign PA4-PA7 and PC0-PC1 to different
 * functions, so pins here follow OBC_DESIGN where it does not collide with the
 * FRAM SPI bus, and the rest are placeholders until a pinout is baselined.
 *
 * No hardware has run this code: it is compiled by CI only.
 */

#include "hal/hal.h"
#include "hal/hal_reset.h"
#include "hal/hal_radio.h"

#ifdef HAL_TARGET_STM32L4

#include <stddef.h>

/*===========================================================================*/
/* Registers (RM0432)                                                         */
/*===========================================================================*/

#define REG32(addr)            (*(volatile uint32_t *)(uintptr_t)(addr))

#define RCC_BASE               0x40021000U
#define RCC_AHB2ENR            REG32(RCC_BASE + 0x4CU)
#define RCC_CSR                REG32(RCC_BASE + 0x94U)
#define RCC_CSR_RMVF           (1U << 23)
#define RCC_CSR_PINRSTF        (1U << 26)
#define RCC_CSR_BORRSTF        (1U << 27)
#define RCC_CSR_SFTRSTF        (1U << 28)
#define RCC_CSR_IWDGRSTF       (1U << 29)
#define RCC_CSR_WWDGRSTF       (1U << 30)

#define GPIO_BASE(port)        (0x48000000U + ((uint32_t)(port) * 0x400U))
#define GPIO_MODER(port)       REG32(GPIO_BASE(port) + 0x00U)
#define GPIO_PUPDR(port)       REG32(GPIO_BASE(port) + 0x0CU)
#define GPIO_IDR(port)         REG32(GPIO_BASE(port) + 0x10U)
#define GPIO_ODR(port)         REG32(GPIO_BASE(port) + 0x14U)
#define GPIO_BSRR(port)        REG32(GPIO_BASE(port) + 0x18U)

#define IWDG_BASE              0x40003000U
#define IWDG_KR                REG32(IWDG_BASE + 0x00U)
#define IWDG_PR                REG32(IWDG_BASE + 0x04U)
#define IWDG_RLR               REG32(IWDG_BASE + 0x08U)
#define IWDG_KEY_ENABLE        0xCCCCU
#define IWDG_KEY_REFRESH       0xAAAAU
#define IWDG_KEY_ACCESS        0x5555U
#define IWDG_LSI_HZ            32000U
#define IWDG_RELOAD_MAX        4095U

#define SYSTICK_CTRL           REG32(0xE000E010U)
#define SYSTICK_LOAD           REG32(0xE000E014U)
#define SYSTICK_VAL            REG32(0xE000E018U)

/** Core clock: MSI 4 MHz after reset. PROVISIONAL until clock setup exists */
#define SQSO_CORE_CLOCK_HZ     4000000U

/*===========================================================================*/
/* GPIO                                                                       */
/*===========================================================================*/

enum { PORT_A = 0, PORT_B = 1, PORT_C = 2 };

typedef struct {
    uint8_t port;
    uint8_t pin;
    HalGpioDir_t dir;
    HalGpioPull_t pull;
} PinMap_t;

/** PROVISIONAL pin assignment (see file header) */
static const PinMap_t s_pins[GPIO_PIN_COUNT] = {
    [GPIO_PIN_PAYLOAD_EN]       = {PORT_B, 4U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_RADIO_EN]         = {PORT_B, 5U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_ADCS_EN]          = {PORT_B, 6U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_HEATER_EN]        = {PORT_C, 2U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_DEPLOY_EN]        = {PORT_A, 1U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_LED_STATUS]       = {PORT_C, 3U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_JETSON_PWR]       = {PORT_A, 2U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_RF_TX_EN]         = {PORT_A, 3U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_ALERT]            = {PORT_B, 7U,  GPIO_DIR_INPUT,  GPIO_PULL_UP},
    [GPIO_PIN_DEPLOY_SENSE]     = {PORT_A, 0U,  GPIO_DIR_INPUT,  GPIO_PULL_NONE},
    [GPIO_PIN_UMBILICAL_SENSE]  = {PORT_B, 8U,  GPIO_DIR_INPUT,  GPIO_PULL_DOWN},
    [GPIO_PIN_ANTENNA_BURN]     = {PORT_B, 9U,  GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_SOLAR_BURN]       = {PORT_B, 10U, GPIO_DIR_OUTPUT, GPIO_PULL_NONE},
    [GPIO_PIN_ANTENNA_SENSE]    = {PORT_B, 11U, GPIO_DIR_INPUT,  GPIO_PULL_NONE},
    [GPIO_PIN_SOLAR_SENSE]      = {PORT_B, 12U, GPIO_DIR_INPUT,  GPIO_PULL_NONE},
};

static bool valid_pin(HalGpioPin_t pin)
{
    return (uint32_t)pin < (uint32_t)GPIO_PIN_COUNT;
}

SmartQsoResult_t hal_gpio_config(HalGpioPin_t pin, HalGpioDir_t direction, HalGpioPull_t pull)
{
    if (!valid_pin(pin)) {
        return SMART_QSO_ERROR_INVALID;
    }
    uint32_t port = s_pins[pin].port;
    uint32_t shift = 2U * (uint32_t)s_pins[pin].pin;

    uint32_t pupd = (pull == GPIO_PULL_UP) ? 1U : ((pull == GPIO_PULL_DOWN) ? 2U : 0U);
    GPIO_PUPDR(port) = (GPIO_PUPDR(port) & ~(3U << shift)) | (pupd << shift);

    if (direction == GPIO_DIR_OUTPUT) {
        /* Drive low before enabling the output: no glitch on enable lines */
        GPIO_BSRR(port) = 1U << (16U + s_pins[pin].pin);
        GPIO_MODER(port) = (GPIO_MODER(port) & ~(3U << shift)) | (1U << shift);
    } else {
        GPIO_MODER(port) &= ~(3U << shift);
    }
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_gpio_init(void)
{
    RCC_AHB2ENR |= (1U << PORT_A) | (1U << PORT_B) | (1U << PORT_C);
    for (uint32_t i = 0U; i < (uint32_t)GPIO_PIN_COUNT; i++) {
        (void)hal_gpio_config((HalGpioPin_t)i, s_pins[i].dir, s_pins[i].pull);
    }
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_gpio_set(HalGpioPin_t pin, bool state)
{
    if (!valid_pin(pin)) {
        return SMART_QSO_ERROR_INVALID;
    }
    uint32_t bit = (uint32_t)s_pins[pin].pin + (state ? 0U : 16U);
    GPIO_BSRR(s_pins[pin].port) = 1U << bit;
    return SMART_QSO_OK;
}

bool hal_gpio_get(HalGpioPin_t pin)
{
    if (!valid_pin(pin)) {
        return false;
    }
    return (GPIO_IDR(s_pins[pin].port) & (1U << s_pins[pin].pin)) != 0U;
}

SmartQsoResult_t hal_gpio_toggle(HalGpioPin_t pin)
{
    if (!valid_pin(pin)) {
        return SMART_QSO_ERROR_INVALID;
    }
    bool high = (GPIO_ODR(s_pins[pin].port) & (1U << s_pins[pin].pin)) != 0U;
    return hal_gpio_set(pin, !high);
}

/*===========================================================================*/
/* Time base (SysTick, 1 ms)                                                  */
/*===========================================================================*/

static volatile uint64_t s_ms = 0U;
static bool s_timer_initialized = false;

void SysTick_Handler(void);
void SysTick_Handler(void)
{
    s_ms++;
}

SmartQsoResult_t hal_timer_init(void)
{
    SYSTICK_LOAD = (SQSO_CORE_CLOCK_HZ / 1000U) - 1U;
    SYSTICK_VAL = 0U;
    SYSTICK_CTRL = 0x7U;   /* processor clock, interrupt, enable */
    s_timer_initialized = true;
    return SMART_QSO_OK;
}

uint64_t hal_timer_get_ms(void)
{
    uint64_t first;
    uint64_t second;
    /* 64-bit read is not atomic on Cortex-M4: read until stable */
    do {
        first = s_ms;
        second = s_ms;
    } while (first != second);
    return s_timer_initialized ? first : 0U;
}

uint64_t hal_timer_get_us(void)
{
    uint32_t ticks_per_us = SQSO_CORE_CLOCK_HZ / 1000000U;
    uint32_t into_ms = (SYSTICK_LOAD - SYSTICK_VAL) / ticks_per_us;
    return (hal_timer_get_ms() * 1000U) + into_ms;
}

void hal_timer_delay_ms(uint32_t ms)
{
    uint64_t start = hal_timer_get_ms();
    while ((hal_timer_get_ms() - start) < ms) {
        /* busy wait */
    }
}

void hal_timer_delay_us(uint32_t us)
{
    uint64_t start = hal_timer_get_us();
    while ((hal_timer_get_us() - start) < us) {
        /* busy wait */
    }
}

bool hal_timer_timeout(uint64_t start_ms, uint32_t timeout_ms)
{
    return (hal_timer_get_ms() - start_ms) >= timeout_ms;
}

uint64_t hal_timer_elapsed_ms(uint64_t start_ms)
{
    return hal_timer_get_ms() - start_ms;
}

SmartQsoResult_t hal_timer_start_periodic(uint32_t period_ms, HalTimerCallback_t callback,
                                           void *context, HalTimerId_t *timer_id)
{
    (void)period_ms;
    (void)callback;
    (void)context;
    (void)timer_id;
    return SMART_QSO_ERROR;   /* not written: the main loop polls instead */
}

SmartQsoResult_t hal_timer_stop(HalTimerId_t timer_id)
{
    (void)timer_id;
    return SMART_QSO_ERROR;
}

/*===========================================================================*/
/* Reset cause (RCC CSR)                                                      */
/*===========================================================================*/

static bool s_reset_read = false;
static HalResetCause_t s_reset_cause = HAL_RESET_UNKNOWN;

HalResetCause_t hal_reset_cause(void)
{
    if (!s_reset_read) {
        uint32_t csr = RCC_CSR;
        /* NRST is pulled low by every internal reset, so the pin flag is
         * checked last. The L4 has no separate power-on flag: a power-on
         * sets BORRSTF, and both restart the timers alike. */
        if ((csr & (RCC_CSR_IWDGRSTF | RCC_CSR_WWDGRSTF)) != 0U) {
            s_reset_cause = HAL_RESET_WATCHDOG;
        } else if ((csr & RCC_CSR_SFTRSTF) != 0U) {
            s_reset_cause = HAL_RESET_SOFTWARE;
        } else if ((csr & RCC_CSR_BORRSTF) != 0U) {
            s_reset_cause = HAL_RESET_POWER_ON;
        } else if ((csr & RCC_CSR_PINRSTF) != 0U) {
            s_reset_cause = HAL_RESET_PIN;
        } else {
            s_reset_cause = HAL_RESET_UNKNOWN;
        }
        RCC_CSR |= RCC_CSR_RMVF;
        s_reset_read = true;
    }
    return s_reset_cause;
}

/*===========================================================================*/
/* Independent watchdog (IWDG)                                                */
/*===========================================================================*/

static uint32_t s_wdt_timeout_ms = 0U;
static uint64_t s_wdt_last_kick_ms = 0U;
static bool s_wdt_running = false;
static bool s_wdt_reset_flag_cleared = false;

SmartQsoResult_t hal_wdt_init(uint32_t timeout_ms, HalWdtMode_t mode)
{
    if ((mode != WDT_MODE_RESET) || (timeout_ms == 0U)) {
        return SMART_QSO_ERROR_INVALID;   /* the IWDG only resets */
    }
    /* Smallest prescaler (4 << pr) whose reload fits in 12 bits */
    for (uint32_t pr = 0U; pr <= 6U; pr++) {
        uint32_t divider = 4U << pr;
        uint64_t ticks = ((uint64_t)timeout_ms * IWDG_LSI_HZ) / (1000ULL * divider);
        if ((ticks >= 1U) && (ticks <= (IWDG_RELOAD_MAX + 1U))) {
            IWDG_KR = IWDG_KEY_ACCESS;
            IWDG_PR = pr;
            IWDG_RLR = (uint32_t)(ticks - 1U);
            s_wdt_timeout_ms = timeout_ms;
            return SMART_QSO_OK;
        }
    }
    return SMART_QSO_ERROR_INVALID;   /* longer than about 32.8 s */
}

SmartQsoResult_t hal_wdt_start(void)
{
    if (s_wdt_timeout_ms == 0U) {
        return SMART_QSO_ERROR;
    }
    IWDG_KR = IWDG_KEY_ENABLE;
    s_wdt_running = true;
    s_wdt_last_kick_ms = hal_timer_get_ms();
    return SMART_QSO_OK;
}

SmartQsoResult_t hal_wdt_stop(void)
{
    return SMART_QSO_ERROR;   /* the IWDG cannot be stopped once started */
}

void hal_wdt_kick(void)
{
    if (s_wdt_running) {
        IWDG_KR = IWDG_KEY_REFRESH;
        s_wdt_last_kick_ms = hal_timer_get_ms();
    }
}

uint32_t hal_wdt_remaining_ms(void)
{
    if (!s_wdt_running) {
        return s_wdt_timeout_ms;
    }
    uint64_t elapsed = hal_timer_get_ms() - s_wdt_last_kick_ms;
    return (elapsed >= s_wdt_timeout_ms) ? 0U : (s_wdt_timeout_ms - (uint32_t)elapsed);
}

bool hal_wdt_caused_reset(void)
{
    return !s_wdt_reset_flag_cleared && (hal_reset_cause() == HAL_RESET_WATCHDOG);
}

void hal_wdt_clear_reset_flag(void)
{
    s_wdt_reset_flag_cleared = true;
}

SmartQsoResult_t hal_wdt_set_warning_callback(void (*callback)(void *context), void *context)
{
    (void)callback;
    (void)context;
    return SMART_QSO_ERROR;   /* the IWDG has no early warning */
}

/*===========================================================================*/
/* Not yet written: fail safe                                                 */
/*===========================================================================*/

SmartQsoResult_t hal_flash_init(void) { return SMART_QSO_ERROR_IO; }

SmartQsoResult_t hal_flash_read(HalFlashRegion_t region, uint32_t offset, uint8_t *data, size_t len)
{
    (void)region; (void)offset; (void)data; (void)len;
    return SMART_QSO_ERROR_IO;
}

SmartQsoResult_t hal_flash_write(HalFlashRegion_t region, uint32_t offset, const uint8_t *data,
                                 size_t len)
{
    (void)region; (void)offset; (void)data; (void)len;
    return SMART_QSO_ERROR_IO;
}

SmartQsoResult_t hal_flash_erase(HalFlashRegion_t region)
{
    (void)region;
    return SMART_QSO_ERROR_IO;
}

size_t hal_flash_region_size(HalFlashRegion_t region) { (void)region; return 0U; }
uint32_t hal_flash_region_base(HalFlashRegion_t region) { (void)region; return 0U; }
bool hal_flash_busy(void) { return false; }
uint8_t hal_flash_wear_level(HalFlashRegion_t region) { (void)region; return 0U; }

SmartQsoResult_t hal_uart_init(HalUartPort_t port, const HalUartConfig_t *config)
{
    (void)port; (void)config;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_uart_write(HalUartPort_t port, const uint8_t *data, size_t len)
{
    (void)port; (void)data; (void)len;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_uart_read(HalUartPort_t port, uint8_t *data, size_t len, uint32_t timeout_ms,
                               size_t *bytes_read)
{
    (void)port; (void)data; (void)len; (void)timeout_ms; (void)bytes_read;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_uart_read_line(HalUartPort_t port, char *buffer, size_t buffer_len,
                                    uint32_t timeout_ms, size_t *line_len)
{
    (void)port; (void)buffer; (void)buffer_len; (void)timeout_ms; (void)line_len;
    return SMART_QSO_ERROR;
}

size_t hal_uart_available(HalUartPort_t port) { (void)port; return 0U; }
SmartQsoResult_t hal_uart_flush(HalUartPort_t port) { (void)port; return SMART_QSO_ERROR; }

SmartQsoResult_t hal_i2c_init(HalI2cBus_t bus, HalI2cSpeed_t speed)
{
    (void)bus; (void)speed;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_i2c_write(HalI2cBus_t bus, uint8_t addr, const uint8_t *data, size_t len)
{
    (void)bus; (void)addr; (void)data; (void)len;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_i2c_read(HalI2cBus_t bus, uint8_t addr, uint8_t *data, size_t len)
{
    (void)bus; (void)addr; (void)data; (void)len;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_i2c_write_read(HalI2cBus_t bus, uint8_t addr, const uint8_t *write_data,
                                    size_t write_len, uint8_t *read_data, size_t read_len)
{
    (void)bus; (void)addr; (void)write_data; (void)write_len; (void)read_data; (void)read_len;
    return SMART_QSO_ERROR;
}

bool hal_i2c_device_present(HalI2cBus_t bus, uint8_t addr) { (void)bus; (void)addr; return false; }
SmartQsoResult_t hal_i2c_recover(HalI2cBus_t bus) { (void)bus; return SMART_QSO_ERROR; }

SmartQsoResult_t hal_spi_init(HalSpiBus_t bus, const HalSpiConfig_t *config)
{
    (void)bus; (void)config;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_spi_transfer(HalSpiBus_t bus, const uint8_t *tx_data, uint8_t *rx_data,
                                  size_t len)
{
    (void)bus; (void)tx_data; (void)rx_data; (void)len;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_spi_write(HalSpiBus_t bus, const uint8_t *data, size_t len)
{
    (void)bus; (void)data; (void)len;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_spi_read(HalSpiBus_t bus, uint8_t *data, size_t len)
{
    (void)bus; (void)data; (void)len;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_spi_cs_assert(HalSpiBus_t bus) { (void)bus; return SMART_QSO_ERROR; }
SmartQsoResult_t hal_spi_cs_deassert(HalSpiBus_t bus) { (void)bus; return SMART_QSO_ERROR; }

SmartQsoResult_t hal_adc_init(HalAdcResolution_t resolution, HalAdcRef_t reference)
{
    (void)resolution; (void)reference;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_adc_read_raw(HalAdcChannel_t channel, uint16_t *value)
{
    (void)channel; (void)value;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_adc_read_voltage(HalAdcChannel_t channel, double *voltage)
{
    (void)channel; (void)voltage;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_adc_read_multiple(const HalAdcChannel_t *channels, uint16_t *values,
                                       size_t count)
{
    (void)channels; (void)values; (void)count;
    return SMART_QSO_ERROR;
}

SmartQsoResult_t hal_adc_calibrate(void) { return SMART_QSO_ERROR; }

SmartQsoResult_t hal_radio_init(void) { return SMART_QSO_ERROR; }

SmartQsoResult_t hal_radio_receive(uint8_t *frame, size_t max_len, size_t *len)
{
    (void)frame; (void)max_len;
    if (len != NULL) {
        *len = 0U;
    }
    return SMART_QSO_ERROR;
}

#endif /* HAL_TARGET_STM32L4 */
