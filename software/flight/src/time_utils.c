/**
 * @file time_utils.c
 * @brief Time Utility Functions Implementation
 *
 * Provides time-related utility functions for the flight software.
 */

/* Required for clock_gettime on POSIX-compliant systems */
#define _XOPEN_SOURCE 600

#include "smart_qso.h"
#include "hal/hal_timer.h"

#include <time.h>

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

#ifdef SIMULATION_BUILD
/** Time added by tests (smart_qso_sim_advance_ms) */
static uint64_t s_sim_offset_ms = 0U;

void smart_qso_sim_advance_ms(uint64_t ms)
{
    s_sim_offset_ms += ms;
}

uint64_t smart_qso_sim_offset_ms(void)
{
    return s_sim_offset_ms;
}
#endif

uint64_t smart_qso_now_ms(void)
{
#ifdef SIMULATION_BUILD
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return 0;
    }
    return ((uint64_t)ts.tv_sec * 1000ULL) + (uint64_t)(ts.tv_nsec / 1000000L) + s_sim_offset_ms;
#else
    /* Milliseconds since boot from the HAL time base (SysTick on the
     * STM32L4): monotonic, not wall-clock time */
    return hal_timer_get_ms();
#endif
}
