/**
 * @file stm32l4_startup.c
 * @brief STM32L4R5 vector table and reset handler (target only, SW-01)
 *
 * The vector table holds the initial stack pointer, the 15 Cortex-M4
 * exception vectors and the device's 95 interrupt vectors (IRQ 0 to 94: the
 * last is DMAMUX1_OVR_IRQn = 94 in ST's CMSIS header stm32l4r5xx.h, 111
 * words in all). The flight software uses only SysTick; every other
 * exception and interrupt goes to unused_vector(), which waits for the
 * independent watchdog to reset the chip.
 *
 * Reset: copy initialised data from flash, zero .bss, enable the FPU, set
 * the 80 MHz clock (stm32l4_clock.c), then main(). main() does not return on
 * the target; if it did, unused_vector() follows.
 *
 * The linker symbols come from cmake/stm32l4r5zi.ld. Not run on hardware.
 */

#include <stddef.h>
#include <stdint.h>

#include "hal/stm32l4_clock.h"

/* Linker script symbols: only their addresses are used */
extern uint32_t _estack[];
extern uint32_t _sidata[];
extern uint32_t _sdata[];
extern uint32_t _edata[];
extern uint32_t _sbss[];
extern uint32_t _ebss[];

int main(void);

void Reset_Handler(void);
void SysTick_Handler(void);   /* hal_stm32l4.c */

/** Coprocessor access control: full access to CP10 and CP11 (the FPU) */
#define SCB_CPACR              (*(volatile uint32_t *)(uintptr_t)0xE000ED88U)
#define SCB_CPACR_FPU_FULL     (0xFUL << 20)

/** Device interrupt vectors after the 16 core entries */
#define IRQ_VECTOR_COUNT       95U

typedef void (*Vector_t)(void);

typedef struct {
    /* cppcheck-suppress unusedStructMember ; read by the core at reset, not by C code */
    const uint32_t *initial_sp;
    /* cppcheck-suppress unusedStructMember ; read by the core on an exception */
    Vector_t core[15];
    /* cppcheck-suppress unusedStructMember ; read by the core on an interrupt */
    Vector_t irq[IRQ_VECTOR_COUNT];
} VectorTable_t;

static void unused_vector(void)
{
    for (;;) {
        /* The IWDG keeps counting; the watchdog resets the chip */
    }
}

#define U unused_vector
#define U10 U, U, U, U, U, U, U, U, U, U

/* Placed at 0x08000000 by the linker script (section .isr_vector) */
__attribute__((section(".isr_vector"), used))
static const VectorTable_t s_vector_table = {
    _estack,
    {
        Reset_Handler,
        U,                /* NMI */
        U,                /* HardFault */
        U,                /* MemManage */
        U,                /* BusFault */
        U,                /* UsageFault */
        NULL, NULL, NULL, NULL,   /* reserved */
        U,                /* SVCall */
        U,                /* DebugMonitor */
        NULL,             /* reserved */
        U,                /* PendSV */
        SysTick_Handler,
    },
    { U10, U10, U10, U10, U10, U10, U10, U10, U10, U, U, U, U, U },
};


__attribute__((no_stack_protector))
void Reset_Handler(void)
{
    /* Section sizes as integers: to C the symbols are distinct objects */
    const uintptr_t data_words = ((uintptr_t)_edata - (uintptr_t)_sdata) / sizeof(uint32_t);
    const uintptr_t bss_words = ((uintptr_t)_ebss - (uintptr_t)_sbss) / sizeof(uint32_t);

    for (uintptr_t i = 0U; i < data_words; i++) {
        _sdata[i] = _sidata[i];
    }
    for (uintptr_t i = 0U; i < bss_words; i++) {
        _sbss[i] = 0U;
    }

    SCB_CPACR |= SCB_CPACR_FPU_FULL;
    __asm volatile ("dsb\n\tisb" ::: "memory");

    (void)stm32l4_clock_init();   /* on failure the core stays on MSI 4 MHz */
    (void)main();
    unused_vector();
}
