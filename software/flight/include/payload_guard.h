/**
 * @file payload_guard.h
 * @brief OBC-enforced Jetson power and temperature limits (SW-16)
 *
 * Implemented in Rust (software/rust/sqso-fsw/src/payload_guard.rs,
 * software/rust/sqso-ffi/src/payload_guard.rs).
 *
 * The OBC decides from its own measurements whether the payload rail may be
 * on, whatever the Jetson reports about itself (BASELINE §6: per-rail current
 * and temperature limits independent of the Jetson). The rail must be off
 * when, while it is on:
 *
 *   - input power is above PG_MAX_INPUT_W for PG_OVERPOWER_SAMPLES samples
 *     in a row, or any sample reaches PG_LATCHUP_INPUT_W;
 *   - the heat spreader (OBC thermistor) reaches PG_SPREADER_MAX_C;
 *   - the Jetson-reported junction temperature reaches PG_JUNCTION_MAX_C;
 *   - no power or spreader measurement has arrived for
 *     PG_TELEMETRY_TIMEOUT_MS: an unsupervised rail is not left on.
 *
 * After a trip the rail stays off for PG_COOLDOWN_MS, and then until a recent
 * spreader reading is PG_RESTART_MARGIN_C below its limit.
 * eps_control_payload() refuses to enable the rail while
 * payload_guard_may_enable() is false.
 *
 * Software is the backstop: latch-up needs a hardware current limit on the
 * rail, which acts in microseconds.
 */

#ifndef PAYLOAD_GUARD_H
#define PAYLOAD_GUARD_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Carrier input limit (BASELINE §6) */
#define PG_MAX_INPUT_W           8.0
/** A single sample this high trips at once (suspected latch-up) */
#define PG_LATCHUP_INPUT_W       12.0
#define PG_OVERPOWER_SAMPLES     2U
/** Shutdown limits (hardware/payload-jetson/POWER_MANAGEMENT.md §5.1) */
#define PG_SPREADER_MAX_C        70.0
#define PG_JUNCTION_MAX_C        95.0
/** The spreader must cool this far below its limit before a restart */
#define PG_RESTART_MARGIN_C      10.0
#define PG_COOLDOWN_MS           600000U
#define PG_TELEMETRY_TIMEOUT_MS  5000U

typedef struct {
    bool   power_valid;     /**< input_w measured (bus voltage x rail current) */
    double input_w;
    bool   spreader_valid;  /**< spreader_c from the OBC thermistor */
    double spreader_c;
    bool   junction_valid;  /**< junction_c reported by the Jetson (STATUS) */
    double junction_c;
} PayloadSample_t;

typedef enum {
    PG_OK = 0,              /**< Nothing to do */
    PG_TRIP_OVERPOWER,
    PG_TRIP_LATCHUP,
    PG_TRIP_SPREADER_TEMP,
    PG_TRIP_JUNCTION_TEMP,
    PG_TRIP_NO_TELEMETRY,
    PG_COOLDOWN             /**< Rail on during a cool-down: turn it off */
} PayloadGuardResult_t;

void payload_guard_init(void);

/**
 * @brief Evaluate one set of measurements
 * @param sample     Latest measurements (NULL counts as none)
 * @param payload_on Payload rail state
 * @param now_ms     Monotonic time
 * @return PG_OK, or why the rail must be turned off now
 */
PayloadGuardResult_t payload_guard_step(const PayloadSample_t *sample, bool payload_on,
                                        uint64_t now_ms);

/** May the rail be turned on now? */
bool payload_guard_may_enable(uint64_t now_ms);

uint32_t payload_guard_trip_count(void);

/** Fault-log text for a result */
const char *payload_guard_reason(PayloadGuardResult_t result);

#ifdef __cplusplus
}
#endif

#endif /* PAYLOAD_GUARD_H */
