/**
 * @file stm32l4_clock.c
 * @brief STM32L4R5 system clock: 80 MHz from the internal MSI (SW-01)
 *
 * After reset the core runs from MSI at 4 MHz. This raises it to 80 MHz
 * through the main PLL: MSI 4 MHz / M 1 x N 40 = 160 MHz VCO, / R 2 = 80 MHz.
 * 80 MHz is the top of voltage Range 1 normal mode, the reset default, so
 * neither boost mode nor the AHB prescaler step ST's HAL uses above 80 MHz
 * is needed. MSI is used rather than an external crystal because there is
 * no schematic yet; moving to the HSE is a board decision.
 *
 * Flash wait states: ST's table for the STM32L4+ (stm32l4xx_hal_rcc.c)
 * needs 3 for 60-80 MHz in Range 1. Four are set, as margin until the board
 * is brought up, before the clock rises, and read back.
 *
 * Every wait is bounded. If the PLL does not lock or the switch does not
 * take, the core stays on MSI at 4 MHz and the call reports an error.
 *
 * Register addresses and bit positions: ST CMSIS header stm32l4r5xx.h
 * (RCC_BASE 0x40021000, FLASH_R_BASE 0x40022000; RCC_CR PLLON 24, PLLRDY 25;
 * RCC_CFGR SW [1:0], SWS [3:2]; RCC_PLLCFGR PLLSRC [1:0] (MSI = 1),
 * PLLM [7:4] (0 = /1), PLLN [14:8], PLLREN 24, PLLR [26:25] (0 = /2);
 * FLASH_ACR LATENCY [3:0]).
 *
 * Not run on hardware.
 */

#include "hal/stm32l4_clock.h"

#define REG32(addr)            (*(volatile uint32_t *)(uintptr_t)(addr))

#define RCC_CR                 REG32(0x40021000U)
#define RCC_CFGR               REG32(0x40021008U)
#define RCC_PLLCFGR            REG32(0x4002100CU)
#define FLASH_ACR              REG32(0x40022000U)

#define RCC_CR_PLLON           (1UL << 24)
#define RCC_CR_PLLRDY          (1UL << 25)
#define RCC_CFGR_SW_MASK       0x3UL
#define RCC_CFGR_SW_MSI        0x0UL
#define RCC_CFGR_SW_PLL        0x3UL
#define RCC_CFGR_SWS_MASK      0xCUL
#define RCC_CFGR_SWS_PLL       0xCUL
#define RCC_PLLCFGR_SRC_MSI    0x1UL
#define RCC_PLLCFGR_N_POS      8U
#define RCC_PLLCFGR_REN        (1UL << 24)
#define FLASH_ACR_LATENCY_MASK 0xFUL

/** PLL: M = 1 (field 0), N = 40, R = 2 (field 0) */
#define PLL_N                  40UL

/** Flash wait states for 80 MHz in Range 1 (3 needed, one of margin) */
#define FLASH_WAIT_STATES      4UL

/** Upper bound on polls of a ready flag (lock takes well under 1 ms at 4 MHz) */
#define CLOCK_POLL_LIMIT       100000U

static uint32_t s_core_clock_hz = STM32L4_CLOCK_RESET_HZ;

/** Poll until (reg & mask) == want, at most CLOCK_POLL_LIMIT times */
static bool wait_for(volatile const uint32_t *reg, uint32_t mask, uint32_t want)
{
    for (uint32_t polls = 0U; polls < CLOCK_POLL_LIMIT; polls++) {
        if ((*reg & mask) == want) {
            return true;
        }
    }
    return false;
}

static bool start_pll(void)
{
    RCC_CR &= ~RCC_CR_PLLON;
    if (!wait_for(&RCC_CR, RCC_CR_PLLRDY, 0U)) {
        return false;
    }
    RCC_PLLCFGR = RCC_PLLCFGR_SRC_MSI | (PLL_N << RCC_PLLCFGR_N_POS) | RCC_PLLCFGR_REN;
    RCC_CR |= RCC_CR_PLLON;
    return wait_for(&RCC_CR, RCC_CR_PLLRDY, RCC_CR_PLLRDY);
}

SmartQsoResult_t stm32l4_clock_init(void)
{
    if ((RCC_CFGR & RCC_CFGR_SWS_MASK) == RCC_CFGR_SWS_PLL) {
        return SMART_QSO_OK;   /* already running from the PLL */
    }

    /* Wait states before the frequency rises; read back */
    FLASH_ACR = (FLASH_ACR & ~FLASH_ACR_LATENCY_MASK) | FLASH_WAIT_STATES;
    if ((FLASH_ACR & FLASH_ACR_LATENCY_MASK) != FLASH_WAIT_STATES) {
        return SMART_QSO_ERROR;
    }

    if (!start_pll()) {
        RCC_CR &= ~RCC_CR_PLLON;
        return SMART_QSO_ERROR;
    }

    RCC_CFGR = (RCC_CFGR & ~RCC_CFGR_SW_MASK) | RCC_CFGR_SW_PLL;
    if (!wait_for(&RCC_CFGR, RCC_CFGR_SWS_MASK, RCC_CFGR_SWS_PLL)) {
        RCC_CFGR = (RCC_CFGR & ~RCC_CFGR_SW_MASK) | RCC_CFGR_SW_MSI;
        return SMART_QSO_ERROR;
    }

    s_core_clock_hz = STM32L4_CLOCK_TARGET_HZ;
    return SMART_QSO_OK;
}

uint32_t stm32l4_core_clock_hz(void)
{
    return s_core_clock_hz;
}
