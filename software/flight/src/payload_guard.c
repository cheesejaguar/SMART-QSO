/**
 * @file payload_guard.c
 * @brief OBC-enforced Jetson power and temperature limits (SW-16)
 *
 * See payload_guard.h for the rules.
 */

#include "payload_guard.h"

#include <stddef.h>

static bool     s_was_on = false;
static uint64_t s_power_ms = 0U;      /**< Last valid power reading (or rail-on time) */
static uint64_t s_spreader_ms = 0U;   /**< Last valid spreader reading (or rail-on time) */
static bool     s_spreader_seen = false;
static double   s_spreader_c = 0.0;
static uint64_t s_spreader_seen_ms = 0U;
static uint32_t s_over_count = 0U;
static bool     s_tripped = false;
static uint64_t s_trip_ms = 0U;
static uint32_t s_trips = 0U;

void payload_guard_init(void)
{
    s_was_on = false;
    s_power_ms = 0U;
    s_spreader_ms = 0U;
    s_spreader_seen = false;
    s_spreader_c = 0.0;
    s_spreader_seen_ms = 0U;
    s_over_count = 0U;
    s_tripped = false;
    s_trip_ms = 0U;
    s_trips = 0U;
}

static bool in_cooldown(uint64_t now_ms)
{
    return s_tripped && (now_ms < (s_trip_ms + PG_COOLDOWN_MS));
}

static bool age_ok(uint64_t then_ms, uint64_t now_ms)
{
    return (now_ms >= then_ms) && ((now_ms - then_ms) <= PG_TELEMETRY_TIMEOUT_MS);
}

/** Record the readings; restart the supervision clocks when the rail turns on */
static void record(const PayloadSample_t *s, bool payload_on, uint64_t now_ms)
{
    if (payload_on && !s_was_on) {
        s_power_ms = now_ms;
        s_spreader_ms = now_ms;
        s_over_count = 0U;
    }
    s_was_on = payload_on;
    if (s == NULL) {
        return;
    }
    if (s->power_valid) {
        s_power_ms = now_ms;
    }
    if (s->spreader_valid) {
        s_spreader_ms = now_ms;
        s_spreader_seen = true;
        s_spreader_c = s->spreader_c;
        s_spreader_seen_ms = now_ms;
    }
}

static PayloadGuardResult_t check_limits(const PayloadSample_t *s, uint64_t now_ms)
{
    if (!age_ok(s_power_ms, now_ms) || !age_ok(s_spreader_ms, now_ms)) {
        return PG_TRIP_NO_TELEMETRY;
    }
    if (s == NULL) {
        return PG_OK;
    }
    if (s->power_valid) {
        if (s->input_w >= PG_LATCHUP_INPUT_W) {
            return PG_TRIP_LATCHUP;
        }
        s_over_count = (s->input_w > PG_MAX_INPUT_W) ? (s_over_count + 1U) : 0U;
        if (s_over_count >= PG_OVERPOWER_SAMPLES) {
            return PG_TRIP_OVERPOWER;
        }
    }
    if (s->spreader_valid && (s->spreader_c >= PG_SPREADER_MAX_C)) {
        return PG_TRIP_SPREADER_TEMP;
    }
    if (s->junction_valid && (s->junction_c >= PG_JUNCTION_MAX_C)) {
        return PG_TRIP_JUNCTION_TEMP;
    }
    return PG_OK;
}

PayloadGuardResult_t payload_guard_step(const PayloadSample_t *sample, bool payload_on,
                                        uint64_t now_ms)
{
    record(sample, payload_on, now_ms);
    if (!payload_on) {
        return PG_OK;
    }
    if (in_cooldown(now_ms)) {
        return PG_COOLDOWN;
    }

    PayloadGuardResult_t result = check_limits(sample, now_ms);
    if (result != PG_OK) {
        s_tripped = true;
        s_trip_ms = now_ms;
        s_trips++;
        s_over_count = 0U;
    }
    return result;
}

bool payload_guard_may_enable(uint64_t now_ms)
{
    if (in_cooldown(now_ms)) {
        return false;
    }
    bool spreader_recent = s_spreader_seen && age_ok(s_spreader_seen_ms, now_ms);
    bool spreader_cool = s_spreader_c < (PG_SPREADER_MAX_C - PG_RESTART_MARGIN_C);
    if (s_tripped) {
        return spreader_recent && spreader_cool;   /* after a trip, show it has cooled */
    }
    return !spreader_recent || spreader_cool;      /* otherwise refuse only if known hot */
}

uint32_t payload_guard_trip_count(void)
{
    return s_trips;
}

const char *payload_guard_reason(PayloadGuardResult_t result)
{
    switch (result) {
        case PG_OK:                 return "OK";
        case PG_TRIP_OVERPOWER:     return "Payload over 8 W input: rail off";
        case PG_TRIP_LATCHUP:       return "Payload latch-up current: rail off";
        case PG_TRIP_SPREADER_TEMP: return "Payload heat spreader over limit: rail off";
        case PG_TRIP_JUNCTION_TEMP: return "Jetson junction over limit: rail off";
        case PG_TRIP_NO_TELEMETRY:  return "Payload unsupervised (no measurements): rail off";
        case PG_COOLDOWN:           return "Payload rail on during cool-down: rail off";
        default:                    return "Payload guard: unknown result";
    }
}
