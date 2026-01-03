/**
 * @file time_utils.c
 * @brief Time Utility Functions Implementation
 *
 * Provides time-related utility functions for the flight software.
 */

/* Required for clock_gettime on POSIX-compliant systems */
#define _XOPEN_SOURCE 600

#include "smart_qso.h"

#include <time.h>

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

uint64_t smart_qso_now_ms(void)
{
#ifdef SIMULATION_BUILD
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return 0;
    }
    return (uint64_t)ts.tv_sec * 1000ULL + (uint64_t)(ts.tv_nsec / 1000000L);
#else
    /* Flight hardware implementation placeholder */
    /* TODO: Implement HAL-based time retrieval */
    return 0;
#endif
}
