/**
 * @file test_adcs.c
 * @brief ADCS control: modes, B-dot detumble, sun pointing, rate estimate
 *
 * The expectations come from the design, not from the code:
 * docs/hardware/ADCS_DESIGN.md (§4 control laws, §5 modes and timing, §6.1
 * PWM limit, §7 telemetry), docs/interfaces/ICD_OBC_ADCS.md §7 and the
 * comments in adcs_control.h. Where those disagree, docs/BASELINE_PARAMETERS.md
 * governs, then ADCS_DESIGN.md.
 *
 * - Power-on is IDLE with the coils off; DETUMBLE (commanded) runs B-dot until
 *   the rate stays below 0.5 deg/s for 5 min, then hands over to SUNPOINT. A
 *   detumble still running after 90 min is a fault: back to IDLE, fault
 *   logged, detumble not reported complete. SUNPOINT drops to ECLIPSE (coils
 *   off) when the sun sensors read below 5 % and returns when the sun is back.
 *   SAFE does nothing until commanded out.
 * - B-dot: m = -k dB/dt, saturated to 0.2 A m^2, and it removes rotational
 *   energy. Sun pointing: the torque the coils make turns +X toward the sun.
 * - The rate estimate is the body rate perpendicular to the field, per axis.
 *
 * The simulated magnetometer returns the field the test sets, so a tumble is
 * made by turning the body-frame field between control cycles. The clock is
 * moved with smart_qso_sim_advance_ms() and adcs_update() is given
 * smart_qso_now_ms(), the time base adcs_set_mode() and the sensor reads use.
 *
 * @requirement SYS-410 The ADCS shall include 3-axis magnetorquers
 * @requirement SYS-412 Magnetorquers shall be independently controllable
 * @requirement SYS-420 The ADCS shall detumble the spacecraft to <5 deg/s within 7 days
 * @requirement SYS-421 The ADCS shall implement B-dot detumble control
 * @requirement SYS-422 The ADCS shall achieve gravity gradient or sun-pointing stabilization
 * @requirement SRS-F040 The software shall detect and log all fault conditions
 * @requirement SRS-S004 The software shall validate all commands before execution
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>
#include <stdlib.h>
#include <string.h>

#include "adcs_control.h"
#include "fault_mgmt.h"

/*===========================================================================*/
/* Helpers                                                                    */
/*===========================================================================*/

#define TEST_PI 3.14159265358979323846
#define DEG_TO_RAD(d) ((d) * TEST_PI / 180.0)

/** Earth field magnitude used by the tests (ADCS_DESIGN §2.4: 20-60 uT) */
#define FIELD_UT 45.0

/** One control cycle (ADCS_DESIGN §5.3: 1 Hz) */
#define CYCLE_MS 1000U

/** Body-frame magnetic field the simulated magnetometer returns (uT) */
static Vec3_t s_field;

static Vec3_t vec(double x, double y, double z)
{
    Vec3_t v = {x, y, z};
    return v;
}

static Vec3_t scale(const Vec3_t *v, double k)
{
    return vec(v->x * k, v->y * k, v->z * k);
}

static Vec3_t cross(const Vec3_t *a, const Vec3_t *b)
{
    Vec3_t r = {0.0, 0.0, 0.0};
    vec3_cross(a, b, &r);
    return r;
}

static void set_field(const Vec3_t *field_ut)
{
    s_field = *field_ut;
    adcs_set_sim_sensors(&s_field, NULL, NULL);
}

static void set_sun(double x, double y, double z)
{
    Vec3_t sun = vec(x, y, z);
    adcs_set_sim_sensors(NULL, &sun, NULL);
}

/** Rotate v by angle (rad) about the unit vector k (Rodrigues) */
static Vec3_t rotate(const Vec3_t *v, const Vec3_t *k, double angle)
{
    Vec3_t kxv = cross(k, v);
    double kv = vec3_dot(k, v);
    double c = cos(angle);
    double s = sin(angle);
    return vec((v->x * c) + (kxv.x * s) + (k->x * kv * (1.0 - c)),
               (v->y * c) + (kxv.y * s) + (k->y * kv * (1.0 - c)),
               (v->z * c) + (kxv.z * s) + (k->z * kv * (1.0 - c)));
}

/**
 * One control cycle with the body turning at rate_dps about the unit axis:
 * an inertially fixed field turns the other way in the body frame.
 */
static void step(const Vec3_t *axis, double rate_dps, uint64_t dt_ms)
{
    Vec3_t turned = rotate(&s_field, axis, -DEG_TO_RAD(rate_dps) * ((double)dt_ms / 1000.0));
    set_field(&turned);
    smart_qso_sim_advance_ms(dt_ms);
    assert_int_equal(adcs_update(smart_qso_now_ms()), SMART_QSO_OK);
}

/** Tumble about body +Z for the given number of 1 Hz cycles */
static void tumble(double rate_dps, uint32_t cycles)
{
    const Vec3_t z = {0.0, 0.0, 1.0};
    for (uint32_t i = 0U; i < cycles; i++) {
        step(&z, rate_dps, CYCLE_MS);
    }
}

/** Tumble about body +Z in 10 s cycles (long timeouts) */
static void tumble_minutes(double rate_dps, uint32_t minutes)
{
    const Vec3_t z = {0.0, 0.0, 1.0};
    for (uint32_t i = 0U; i < (minutes * 6U); i++) {
        step(&z, rate_dps, 10000U);
    }
}

static AdcsState_t get_state(void)
{
    AdcsState_t st;
    memset(&st, 0, sizeof(st));
    assert_int_equal(adcs_get_state(&st), SMART_QSO_OK);
    return st;
}

static AdcsTelemetry_t get_telemetry(void)
{
    AdcsTelemetry_t tlm;
    memset(&tlm, 0, sizeof(tlm));
    assert_int_equal(adcs_get_telemetry(&tlm), SMART_QSO_OK);
    return tlm;
}

static void assert_coils_off(void)
{
    AdcsState_t st = get_state();
    assert_false(st.mtq.enabled);
    assert_int_equal(st.mtq.pwm[0], 0);
    assert_int_equal(st.mtq.pwm[1], 0);
    assert_int_equal(st.mtq.pwm[2], 0);
    assert_double_equal(vec3_magnitude(&st.mtq.dipole), 0.0, 1e-12);
}

static void assert_within_dipole_limit(const Vec3_t *m)
{
    assert_true(fabs(m->x) <= (ADCS_MAX_DIPOLE + 1e-9));
    assert_true(fabs(m->y) <= (ADCS_MAX_DIPOLE + 1e-9));
    assert_true(fabs(m->z) <= (ADCS_MAX_DIPOLE + 1e-9));
}

static int setup(void **cm_state)
{
    (void)cm_state;
    Vec3_t field = vec(FIELD_UT, 0.0, 0.0);
    set_field(&field);
    set_sun(1.0, 0.0, 0.0);
    (void)fault_mgmt_init();
    (void)fault_log_clear();
    return (adcs_init() == SMART_QSO_OK) ? 0 : -1;
}

/*===========================================================================*/
/* Before initialization (must run first: adcs_init() cannot be undone)       */
/*===========================================================================*/

/** Every entry point refuses to act, and nothing actuates, before adcs_init() */
static void test_calls_before_init_fail(void **cm_state)
{
    (void)cm_state;
    AdcsState_t st;
    AdcsTelemetry_t tlm;
    Vec3_t m = vec(0.1, 0.0, 0.0);

    assert_int_equal(adcs_update(smart_qso_now_ms()), SMART_QSO_ERROR);
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_ERROR);
    assert_int_equal(adcs_get_state(&st), SMART_QSO_ERROR);
    assert_int_equal(adcs_get_state(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(adcs_get_telemetry(&tlm), SMART_QSO_ERROR);
    assert_int_equal(adcs_get_telemetry(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(adcs_apply_dipole(&m), SMART_QSO_ERROR);
    assert_int_equal(adcs_disable_mtq(), SMART_QSO_ERROR);
    assert_false(adcs_is_detumbled());
    assert_false(adcs_sun_visible());
    assert_double_equal(adcs_get_rate_magnitude(), 0.0, 1e-12);
    assert_int_not_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    assert_int_not_equal(adcs_get_mode(), ADCS_MODE_SUNPOINT);
}

/*===========================================================================*/
/* Initialization                                                             */
/*===========================================================================*/

/** Power-on is IDLE with the coils off and nothing detumbled (§5.2) */
static void test_init_idle_coils_off(void **cm_state)
{
    (void)cm_state;
    AdcsState_t st = get_state();

    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
    assert_int_equal(st.mode, ADCS_MODE_IDLE);
    assert_false(st.detumble_complete);
    assert_false(adcs_is_detumbled());
    assert_int_equal(st.control_cycles, 0);
    assert_int_equal(st.fault_count, 0);
    assert_double_equal(adcs_get_rate_magnitude(), 0.0, 1e-12);
    assert_coils_off();
}

/** Re-initializing returns to IDLE from an active mode and clears the state */
static void test_reinit_returns_to_idle(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble(3.0, 5U);
    assert_true(get_state().mtq.enabled);

    assert_int_equal(adcs_init(), SMART_QSO_OK);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
    assert_int_equal(get_state().control_cycles, 0);
    assert_coils_off();
}

/*===========================================================================*/
/* Mode transitions (ADCS_DESIGN §5.1, §5.2, §5.3)                            */
/*===========================================================================*/

/** IDLE keeps the coils off while tumbling; every update is one control cycle */
static void test_idle_keeps_coils_off(void **cm_state)
{
    (void)cm_state;
    tumble(3.0, 10U);

    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
    assert_int_equal(get_state().control_cycles, 10);
    assert_coils_off();
}

/** Commanded DETUMBLE starts the detumble clock and drives the coils */
static void test_command_detumble_starts_bdot(void **cm_state)
{
    (void)cm_state;
    uint64_t before = smart_qso_now_ms();

    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    AdcsState_t st = get_state();
    assert_true(st.detumble_start_ms >= before);
    assert_int_equal(st.settling_start_ms, 0);
    assert_false(st.detumble_complete);

    tumble(3.0, 5U);
    st = get_state();
    assert_int_equal(st.mode, ADCS_MODE_DETUMBLE);
    assert_true(st.mtq.enabled);
    assert_true(vec3_magnitude(&st.mtq.dipole) > 0.0);
}

/**
 * Below 0.5 deg/s for 5 min ends the detumble and hands over to SUNPOINT
 * (§5.1: DETUMBLE exit "w < 0.5 deg/s for 5 min", SUNPOINT entry "Detumble
 * complete"; §5.2 diagram; §5.3 settling time 5 min)
 */
static void test_detumble_complete_hands_over_to_sunpoint(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble(3.0, 30U);
    assert_int_equal(get_state().settling_start_ms, 0);

    tumble(0.45, 1U);                       /* settling starts */
    assert_true(get_state().settling_start_ms > 0U);
    tumble(0.45, 299U);                     /* 4 min 59 s below threshold */
    assert_int_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    assert_false(adcs_is_detumbled());

    tumble(0.45, 3U);                       /* past 5 min */
    assert_true(adcs_is_detumbled());
    assert_int_equal(adcs_get_mode(), ADCS_MODE_SUNPOINT);
    assert_int_equal(get_telemetry().detumble_done, 1);
    assert_int_equal(get_state().fault_count, 0);
    assert_int_equal(fault_log_get_count(), 0);
}

/** A rate spike restarts the 5-minute settling time */
static void test_rate_spike_restarts_settling(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble(3.0, 5U);
    tumble(0.3, 200U);
    tumble(3.0, 1U);
    assert_int_equal(get_state().settling_start_ms, 0);

    tumble(0.3, 200U);                      /* 401 s since first settling */
    assert_int_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    assert_false(adcs_is_detumbled());

    tumble(0.3, 102U);
    assert_true(adcs_is_detumbled());
    assert_int_equal(adcs_get_mode(), ADCS_MODE_SUNPOINT);
}

/** At or above 0.5 deg/s the detumble never settles */
static void test_rate_above_threshold_never_settles(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble(0.55, 600U);

    AdcsState_t st = get_state();
    assert_int_equal(st.mode, ADCS_MODE_DETUMBLE);
    assert_int_equal(st.settling_start_ms, 0);
    assert_false(st.detumble_complete);
}

/**
 * A detumble still running after 90 min is a fault: IDLE, coils off, fault
 * logged, and not reported as complete (§5.2 fault arrow DETUMBLE -> IDLE;
 * §5.3 timeout 90 min, completion declared only by the settling time)
 */
static void test_detumble_timeout_is_fault_not_completion(void **cm_state)
{
    (void)cm_state;
    FaultLogEntry_t entry;

    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble_minutes(2.0, 89U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    assert_int_equal(fault_log_get_count(), 0);

    tumble_minutes(2.0, 2U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
    assert_false(adcs_is_detumbled());
    assert_int_equal(get_telemetry().detumble_done, 0);
    assert_coils_off();

    assert_int_equal(fault_log_get_count(), 1);
    assert_int_equal(fault_log_get_last(&entry), SMART_QSO_OK);
    assert_int_equal(entry.fault_type, FAULT_TYPE_ADCS);
    assert_int_equal(get_state().fault_count, 1);
}

/** The 90-minute limit counts from entry: a repeated command does not extend it */
static void test_repeated_detumble_command_keeps_clock(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    uint64_t start = get_state().detumble_start_ms;
    tumble_minutes(2.0, 60U);

    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    assert_int_equal(get_state().detumble_start_ms, start);
    tumble_minutes(2.0, 31U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
}

/** A new detumble (re-entered from another mode) gets a fresh 90 minutes */
static void test_new_detumble_restarts_clock(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble_minutes(2.0, 60U);
    assert_int_equal(adcs_set_mode(ADCS_MODE_IDLE), SMART_QSO_OK);
    tumble(2.0, 1U);
    assert_coils_off();

    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble_minutes(2.0, 60U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    assert_int_equal(fault_log_get_count(), 0);
}

/** SUNPOINT -> ECLIPSE (coils off) when the sun is lost, back when it returns */
static void test_sunpoint_eclipse_cycle(void **cm_state)
{
    (void)cm_state;
    set_sun(0.0, 1.0, 0.0);                 /* sun 90 deg off +X */
    assert_int_equal(adcs_set_mode(ADCS_MODE_SUNPOINT), SMART_QSO_OK);
    tumble(0.2, 1U);
    AdcsState_t st = get_state();
    assert_int_equal(st.mode, ADCS_MODE_SUNPOINT);
    assert_true(adcs_sun_visible());
    assert_true(st.mtq.enabled);
    assert_true(vec3_magnitude(&st.mtq.dipole) > 0.0);

    set_sun(0.0, 0.0, 0.0);
    tumble(0.2, 1U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_ECLIPSE);
    assert_false(adcs_sun_visible());
    assert_coils_off();
    tumble(0.2, 5U);                        /* drift: stays in ECLIPSE */
    assert_int_equal(adcs_get_mode(), ADCS_MODE_ECLIPSE);
    assert_coils_off();

    set_sun(0.0, 1.0, 0.0);
    tumble(0.2, 1U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_SUNPOINT);
    tumble(0.2, 1U);
    st = get_state();
    assert_int_equal(st.mode, ADCS_MODE_SUNPOINT);
    assert_true(st.mtq.enabled);
    assert_true(vec3_magnitude(&st.mtq.dipole) > 0.0);
}

/** Eclipse threshold is 5 % of full sun (§5.3) */
static void test_eclipse_threshold_5_percent(void **cm_state)
{
    (void)cm_state;
    set_sun(0.0, 0.07, 0.0);                /* 7 % of full sun: sun detected */
    assert_int_equal(adcs_set_mode(ADCS_MODE_SUNPOINT), SMART_QSO_OK);
    tumble(0.2, 1U);
    assert_true(adcs_sun_visible());
    assert_int_equal(adcs_get_mode(), ADCS_MODE_SUNPOINT);

    set_sun(0.0, 0.03, 0.0);                /* 3 %: eclipse */
    tumble(0.2, 1U);
    assert_false(adcs_sun_visible());
    assert_int_equal(adcs_get_mode(), ADCS_MODE_ECLIPSE);
}

/** SAFE: no control and no autonomous exit until commanded out (§5.1) */
static void test_safe_mode_minimal_until_commanded(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_SUNPOINT), SMART_QSO_OK);
    assert_int_equal(adcs_set_mode(ADCS_MODE_SAFE), SMART_QSO_OK);   /* fault */
    set_sun(0.0, 1.0, 0.0);
    tumble(3.0, 30U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_SAFE);
    assert_coils_off();

    assert_int_equal(adcs_set_mode(ADCS_MODE_IDLE), SMART_QSO_OK);   /* power restored */
    tumble(3.0, 1U);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
    assert_coils_off();
}

/** A mode outside the five defined ones is rejected and changes nothing */
static void test_set_mode_rejects_undefined_mode(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    uint64_t start = get_state().detumble_start_ms;

    assert_int_equal(adcs_set_mode((AdcsMode_t)(ADCS_MODE_SAFE + 1)), SMART_QSO_ERROR_INVALID);
    assert_int_equal(adcs_set_mode((AdcsMode_t)99), SMART_QSO_ERROR_INVALID);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_DETUMBLE);
    assert_int_equal(get_state().detumble_start_ms, start);

    tumble(3.0, 2U);
    assert_true(get_state().mtq.enabled);
}

/*===========================================================================*/
/* B-dot control law (ADCS_DESIGN §4.1, ICD §7.2)                             */
/*===========================================================================*/

/** m = -k dB/dt with k = 5e6 A m^2 s/T and dB/dt in uT/s */
static void test_bdot_law_sign_and_units(void **cm_state)
{
    (void)cm_state;
    MagData_t mag;
    Vec3_t m = vec(9.0, 9.0, 9.0);

    memset(&mag, 0, sizeof(mag));
    mag.valid = true;
    mag.field = vec(30.0, 0.0, 40.0);
    mag.field_rate = vec(0.01, -0.02, 0.004);   /* uT/s */

    adcs_compute_bdot(&mag, &m);
    assert_double_equal(m.x, -0.05, 1e-12);
    assert_double_equal(m.y, 0.10, 1e-12);
    assert_double_equal(m.z, -0.02, 1e-12);
}

/** The B-dot torque opposes the body rate: it removes rotational energy */
static void test_bdot_dissipates_rotational_energy(void **cm_state)
{
    (void)cm_state;
    const Vec3_t rates[] = {
        {0.17, 0.0, 0.0}, {0.0, -0.05, 0.02}, {0.01, 0.01, -0.03}, {0.001, 0.0, 0.0005},
    };
    const Vec3_t fields[] = {
        {0.0, 30.0, 40.0}, {45.0, 0.0, 0.0}, {-20.0, 25.0, 10.0}, {10.0, 50.0, -5.0},
    };

    for (size_t i = 0U; i < (sizeof(rates) / sizeof(rates[0])); i++) {
        for (size_t j = 0U; j < (sizeof(fields) / sizeof(fields[0])); j++) {
            MagData_t mag;
            Vec3_t m = vec(0.0, 0.0, 0.0);
            memset(&mag, 0, sizeof(mag));
            mag.valid = true;
            mag.field = fields[j];
            /* Inertially fixed field seen from a rotating body: dB/dt = -w x B */
            Vec3_t wxb = cross(&rates[i], &fields[j]);
            mag.field_rate = scale(&wxb, -1.0);

            adcs_compute_bdot(&mag, &m);
            Vec3_t b_tesla = scale(&fields[j], 1e-6);
            Vec3_t torque = cross(&m, &b_tesla);
            if (vec3_magnitude(&wxb) > 1e-9) {
                assert_true(vec3_dot(&torque, &rates[i]) < 0.0);
            } else {                            /* rate along B: no torque possible */
                assert_true(vec3_dot(&torque, &rates[i]) <= 0.0);
            }
            assert_within_dipole_limit(&m);
        }
    }
}

/** Saturation keeps the dipole within 0.2 A m^2 and keeps its direction */
static void test_bdot_saturates_and_keeps_direction(void **cm_state)
{
    (void)cm_state;
    MagData_t mag;
    Vec3_t m = vec(0.0, 0.0, 0.0);

    memset(&mag, 0, sizeof(mag));
    mag.valid = true;
    mag.field = vec(30.0, 0.0, 40.0);
    mag.field_rate = vec(3.0, -4.0, 0.0);       /* commands -15, +20 A m^2 */

    adcs_compute_bdot(&mag, &m);
    assert_double_equal(vec3_magnitude(&m), ADCS_MAX_DIPOLE, 1e-12);
    assert_double_equal(m.x, -0.6 * ADCS_MAX_DIPOLE, 1e-12);
    assert_double_equal(m.y, 0.8 * ADCS_MAX_DIPOLE, 1e-12);
    assert_double_equal(m.z, 0.0, 1e-12);
    assert_within_dipole_limit(&m);
}

/** Invalid magnetometer data commands zero; NULL arguments are ignored */
static void test_bdot_invalid_inputs(void **cm_state)
{
    (void)cm_state;
    MagData_t mag;
    Vec3_t m = vec(1.0, 2.0, 3.0);

    memset(&mag, 0, sizeof(mag));
    mag.valid = false;
    mag.field_rate = vec(3.0, -4.0, 1.0);
    adcs_compute_bdot(&mag, &m);
    assert_double_equal(vec3_magnitude(&m), 0.0, 1e-12);

    m = vec(1.0, 2.0, 3.0);
    adcs_compute_bdot(NULL, &m);
    assert_double_equal(m.x, 1.0, 1e-12);
    assert_double_equal(m.z, 3.0, 1e-12);
    mag.valid = true;
    adcs_compute_bdot(&mag, NULL);              /* must not crash */
}

/** In DETUMBLE the coils are driven against dB/dt, within the limit */
static void test_detumble_drives_coils_against_bdot(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(adcs_set_mode(ADCS_MODE_DETUMBLE), SMART_QSO_OK);
    tumble(3.0, 3U);

    AdcsState_t st = get_state();
    AdcsTelemetry_t tlm = get_telemetry();
    assert_true(st.mtq.enabled);
    assert_true(vec3_dot(&st.mtq.dipole, &st.mag.field_rate) < 0.0);
    assert_within_dipole_limit(&st.mtq.dipole);

    /* The commanded dipole reaches all three coils as duty cycle */
    assert_true(st.mtq.pwm[1] > 0);             /* dB/dt is along -Y here */
    assert_int_equal(tlm.mtq_x, st.mtq.pwm[0]);
    assert_int_equal(tlm.mtq_y, st.mtq.pwm[1]);
    assert_int_equal(tlm.mtq_z, st.mtq.pwm[2]);
    assert_int_equal(tlm.mode, ADCS_MODE_DETUMBLE);
}

/*===========================================================================*/
/* Rate estimation (ADCS_DESIGN §4.3, §7.1)                                   */
/*===========================================================================*/

/** Tumbles about each body axis are estimated on that axis, with their sign */
static void test_rate_estimate_axis_and_magnitude(void **cm_state)
{
    (void)cm_state;
    struct {
        Vec3_t axis;
        Vec3_t field;
        double rate_dps;
    } cases[] = {
        {{0.0, 0.0, 1.0}, {FIELD_UT, 0.0, 0.0}, 3.0},
        {{0.0, 0.0, 1.0}, {0.0, FIELD_UT, 0.0}, -2.0},
        {{1.0, 0.0, 0.0}, {0.0, 27.0, 36.0}, 1.5},
        {{0.0, 1.0, 0.0}, {36.0, 0.0, -27.0}, -4.0},
    };

    for (size_t i = 0U; i < (sizeof(cases) / sizeof(cases[0])); i++) {
        assert_int_equal(adcs_init(), SMART_QSO_OK);
        set_field(&cases[i].field);
        for (uint32_t n = 0U; n < 5U; n++) {
            step(&cases[i].axis, cases[i].rate_dps, CYCLE_MS);
        }

        double w = DEG_TO_RAD(cases[i].rate_dps);
        AdcsState_t st = get_state();
        AdcsTelemetry_t tlm = get_telemetry();
        assert_double_equal(adcs_get_rate_magnitude(), fabs(w), 0.03 * fabs(w));
        assert_double_equal(st.angular_rate_est.x, cases[i].axis.x * w, 0.03 * fabs(w));
        assert_double_equal(st.angular_rate_est.y, cases[i].axis.y * w, 0.03 * fabs(w));
        assert_double_equal(st.angular_rate_est.z, cases[i].axis.z * w, 0.03 * fabs(w));

        /* Telemetry in 0.01 deg/s */
        double lsb = cases[i].rate_dps * 100.0;
        assert_double_equal((double)tlm.rate_x, cases[i].axis.x * lsb, 0.03 * fabs(lsb) + 1.0);
        assert_double_equal((double)tlm.rate_y, cases[i].axis.y * lsb, 0.03 * fabs(lsb) + 1.0);
        assert_double_equal((double)tlm.rate_z, cases[i].axis.z * lsb, 0.03 * fabs(lsb) + 1.0);
    }
}

/** No estimate from a single sample or from a field too weak to be real */
static void test_rate_estimate_needs_two_samples_and_field(void **cm_state)
{
    (void)cm_state;
    const Vec3_t z = {0.0, 0.0, 1.0};

    step(&z, 5.0, CYCLE_MS);                    /* first sample: nothing to compare */
    assert_double_equal(adcs_get_rate_magnitude(), 0.0, 1e-12);

    assert_int_equal(adcs_init(), SMART_QSO_OK);
    Vec3_t weak = vec(0.5, 0.0, 0.0);           /* 0.5 uT: failed magnetometer */
    set_field(&weak);
    for (uint32_t n = 0U; n < 5U; n++) {
        step(&z, 90.0, CYCLE_MS);
    }
    assert_double_equal(adcs_get_rate_magnitude(), 0.0, 1e-12);
    assert_int_equal(adcs_get_mode(), ADCS_MODE_IDLE);
}

/*===========================================================================*/
/* Sun-pointing control (ADCS_DESIGN §4.2)                                    */
/*===========================================================================*/

/** The torque the coils make turns the +X face toward the sun */
static void test_sunpoint_torque_turns_plus_x_to_sun(void **cm_state)
{
    (void)cm_state;
    const Vec3_t plus_x = {1.0, 0.0, 0.0};
    const Vec3_t suns[] = {
        {0.0, 1.0, 0.0}, {0.6, 0.0, 0.8}, {0.0, -0.6, 0.8}, {-0.6, 0.8, 0.0}, {0.8, 0.36, -0.48},
    };
    const Vec3_t fields[] = {
        {50.0, 0.0, 0.0}, {0.0, 0.0, 45.0}, {0.0, 30.0, 40.0}, {-20.0, 25.0, 10.0}, {10.0, -40.0, 5.0},
    };

    for (size_t i = 0U; i < (sizeof(suns) / sizeof(suns[0])); i++) {
        for (size_t j = 0U; j < (sizeof(fields) / sizeof(fields[0])); j++) {
            SunSensorData_t sun;
            MagData_t mag;
            Vec3_t m = vec(0.0, 0.0, 0.0);
            memset(&sun, 0, sizeof(sun));
            memset(&mag, 0, sizeof(mag));
            sun.sun_vector = suns[i];
            sun.sun_visible = true;
            mag.field = fields[j];
            mag.valid = true;

            adcs_compute_sunpoint(&sun, &mag, &m);
            assert_within_dipole_limit(&m);

            /* Wanted rotation axis: +X x sun. Only torque across B is possible */
            Vec3_t wanted = cross(&plus_x, &suns[i]);
            Vec3_t across = cross(&wanted, &fields[j]);
            Vec3_t b_tesla = scale(&fields[j], 1e-6);
            Vec3_t torque = cross(&m, &b_tesla);
            if (vec3_magnitude(&across) > 1.0) {
                assert_true(vec3_dot(&torque, &wanted) > 0.0);
            } else {
                assert_true(vec3_dot(&torque, &wanted) > -1e-15);
            }
        }
    }
}

/** Sun on +X already: no correction; saturated corrections stay within limits */
static void test_sunpoint_aligned_and_saturation(void **cm_state)
{
    (void)cm_state;
    SunSensorData_t sun;
    MagData_t mag;
    Vec3_t m = vec(1.0, 1.0, 1.0);

    memset(&sun, 0, sizeof(sun));
    memset(&mag, 0, sizeof(mag));
    sun.sun_vector = vec(1.0, 0.0, 0.0);
    sun.sun_visible = true;
    mag.field = vec(0.0, 30.0, 40.0);
    mag.valid = true;
    adcs_compute_sunpoint(&sun, &mag, &m);
    assert_double_equal(vec3_magnitude(&m), 0.0, 1e-12);

    sun.sun_vector = vec(0.0, 0.0, 1.0);        /* 90 deg error: large command */
    adcs_compute_sunpoint(&sun, &mag, &m);
    assert_double_equal(vec3_magnitude(&m), ADCS_MAX_DIPOLE, 1e-9);
}

/** No sun, invalid or too-weak field: zero command; NULL arguments ignored */
static void test_sunpoint_invalid_inputs(void **cm_state)
{
    (void)cm_state;
    SunSensorData_t sun;
    MagData_t mag;
    Vec3_t m = vec(1.0, 1.0, 1.0);

    memset(&sun, 0, sizeof(sun));
    memset(&mag, 0, sizeof(mag));
    sun.sun_vector = vec(0.0, 1.0, 0.0);
    sun.sun_visible = false;
    mag.field = vec(0.0, 30.0, 40.0);
    mag.valid = true;
    adcs_compute_sunpoint(&sun, &mag, &m);
    assert_double_equal(vec3_magnitude(&m), 0.0, 1e-12);

    sun.sun_visible = true;
    mag.valid = false;
    m = vec(1.0, 1.0, 1.0);
    adcs_compute_sunpoint(&sun, &mag, &m);
    assert_double_equal(vec3_magnitude(&m), 0.0, 1e-12);

    mag.valid = true;
    mag.field = vec(0.5, 0.0, 0.0);             /* |B|^2 below 1 uT^2 */
    m = vec(1.0, 1.0, 1.0);
    adcs_compute_sunpoint(&sun, &mag, &m);
    assert_double_equal(vec3_magnitude(&m), 0.0, 1e-12);

    m = vec(1.0, 1.0, 1.0);
    adcs_compute_sunpoint(NULL, &mag, &m);
    adcs_compute_sunpoint(&sun, NULL, &m);
    assert_double_equal(m.x, 1.0, 1e-12);
    adcs_compute_sunpoint(&sun, &mag, NULL);    /* must not crash */
}

/*===========================================================================*/
/* Magnetorquer commands (ADCS_DESIGN §6.1, header MtqCommand_t)              */
/*===========================================================================*/

/** Duty cycle is dipole / 0.2 A m^2 in percent, per axis */
static void test_apply_dipole_pwm_scaling(void **cm_state)
{
    (void)cm_state;
    Vec3_t m = vec(0.1, -0.05, 0.2);

    assert_int_equal(adcs_apply_dipole(&m), SMART_QSO_OK);
    AdcsState_t st = get_state();
    assert_true(st.mtq.enabled);
    assert_int_equal(st.mtq.pwm[0], 50);
    assert_int_equal(st.mtq.pwm[1], -25);
    assert_int_equal(st.mtq.pwm[2], 100);
    assert_double_equal(st.mtq.dipole.y, -0.05, 1e-12);

    assert_int_equal(adcs_disable_mtq(), SMART_QSO_OK);
    assert_coils_off();
    assert_int_equal(adcs_apply_dipole(NULL), SMART_QSO_ERROR_NULL_PTR);
}

/**
 * Commands beyond the coil limit are clamped to +/-100 % with their sign
 * (software duty-cycle limit, §6.1; header: -100 to +100)
 */
static void test_apply_dipole_clamps_out_of_range(void **cm_state)
{
    (void)cm_state;
    Vec3_t m = vec(0.5, -0.5, 100.0);

    assert_int_equal(adcs_apply_dipole(&m), SMART_QSO_OK);
    AdcsState_t st = get_state();
    assert_int_equal(st.mtq.pwm[0], 100);
    assert_int_equal(st.mtq.pwm[1], -100);
    assert_int_equal(st.mtq.pwm[2], 100);

    m = vec(-0.21, 0.3, -1.0e6);
    assert_int_equal(adcs_apply_dipole(&m), SMART_QSO_OK);
    st = get_state();
    assert_int_equal(st.mtq.pwm[0], -100);
    assert_int_equal(st.mtq.pwm[1], 100);
    assert_int_equal(st.mtq.pwm[2], -100);
}

/*===========================================================================*/
/* Utilities                                                                  */
/*===========================================================================*/

static void test_saturate_dipole(void **cm_state)
{
    (void)cm_state;
    Vec3_t m = vec(0.1, -0.1, 0.05);

    adcs_saturate_dipole(&m, ADCS_MAX_DIPOLE);  /* within the limit: untouched */
    assert_double_equal(m.x, 0.1, 1e-12);
    assert_double_equal(m.y, -0.1, 1e-12);
    assert_double_equal(m.z, 0.05, 1e-12);

    m = vec(0.0, 3.0, -4.0);
    adcs_saturate_dipole(&m, ADCS_MAX_DIPOLE);
    assert_double_equal(m.y, 0.6 * ADCS_MAX_DIPOLE, 1e-12);
    assert_double_equal(m.z, -0.8 * ADCS_MAX_DIPOLE, 1e-12);

    m = vec(0.0, 0.0, 0.0);
    adcs_saturate_dipole(&m, 0.0);
    assert_double_equal(vec3_magnitude(&m), 0.0, 1e-12);
    adcs_saturate_dipole(NULL, ADCS_MAX_DIPOLE); /* must not crash */
}

static void test_vector_utilities(void **cm_state)
{
    (void)cm_state;
    Vec3_t a = vec(3.0, 4.0, 12.0);
    Vec3_t x = vec(1.0, 0.0, 0.0);
    Vec3_t y = vec(0.0, 1.0, 0.0);
    Vec3_t r = vec(7.0, 7.0, 7.0);

    assert_double_equal(vec3_magnitude(&a), 13.0, 1e-12);
    assert_double_equal(vec3_dot(&a, &y), 4.0, 1e-12);
    vec3_cross(&x, &y, &r);                     /* right-handed: x cross y = z */
    assert_double_equal(r.x, 0.0, 1e-12);
    assert_double_equal(r.y, 0.0, 1e-12);
    assert_double_equal(r.z, 1.0, 1e-12);

    vec3_normalize(&a);
    assert_double_equal(vec3_magnitude(&a), 1.0, 1e-12);
    assert_double_equal(a.z, 12.0 / 13.0, 1e-12);
    Vec3_t zero = vec(0.0, 0.0, 0.0);
    vec3_normalize(&zero);                      /* no division by zero */
    assert_double_equal(vec3_magnitude(&zero), 0.0, 1e-12);

    assert_double_equal(vec3_magnitude(NULL), 0.0, 1e-12);
    assert_double_equal(vec3_dot(NULL, &y), 0.0, 1e-12);
    assert_double_equal(vec3_dot(&x, NULL), 0.0, 1e-12);
    r = vec(7.0, 7.0, 7.0);
    vec3_cross(NULL, &y, &r);
    vec3_cross(&x, NULL, &r);
    assert_double_equal(r.x, 7.0, 1e-12);
    vec3_cross(&x, &y, NULL);
    vec3_normalize(NULL);
}

/*===========================================================================*/
/* Sensors and telemetry (ADCS_DESIGN §3.2, §7.1)                             */
/*===========================================================================*/

static void test_sensor_reads(void **cm_state)
{
    (void)cm_state;
    MagData_t mag;
    SunSensorData_t sun;
    Vec3_t field = vec(30.0, -12.5, 40.0);
    uint64_t before = smart_qso_now_ms();

    assert_int_equal(adcs_read_magnetometer(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(adcs_read_sun_sensors(NULL), SMART_QSO_ERROR_NULL_PTR);

    set_field(&field);
    set_sun(0.0, 0.6, 0.8);
    adcs_set_sim_sensors(NULL, NULL, NULL);     /* NULLs leave values alone */

    memset(&mag, 0, sizeof(mag));
    assert_int_equal(adcs_read_magnetometer(&mag), SMART_QSO_OK);
    assert_true(mag.valid);
    assert_double_equal(mag.field.y, -12.5, 1e-12);
    assert_true(mag.timestamp_ms >= before);

    memset(&sun, 0, sizeof(sun));
    assert_int_equal(adcs_read_sun_sensors(&sun), SMART_QSO_OK);
    assert_true(sun.sun_visible);
    assert_double_equal(vec3_magnitude(&sun.sun_vector), 1.0, 1e-12);
    assert_double_equal(sun.sun_vector.z, 0.8, 1e-12);
    assert_true(sun.raw[0] > 0U);
    assert_true(sun.timestamp_ms >= before);
}

/** Telemetry units: field 0.1 uT, sun vector 0.01, flags (§7.1) */
static void test_telemetry_scaling(void **cm_state)
{
    (void)cm_state;
    Vec3_t field = vec(30.0, -12.5, 40.0);

    set_field(&field);
    set_sun(0.0, 0.6, 0.8);
    smart_qso_sim_advance_ms(CYCLE_MS);
    assert_int_equal(adcs_update(smart_qso_now_ms()), SMART_QSO_OK);

    AdcsTelemetry_t tlm = get_telemetry();
    assert_int_equal(tlm.mode, ADCS_MODE_IDLE);
    assert_int_equal(tlm.mag_x, 300);
    assert_int_equal(tlm.mag_y, -125);
    assert_int_equal(tlm.mag_z, 400);
    assert_true(abs(tlm.sun_x) <= 1);
    assert_true(abs(tlm.sun_y - 60) <= 1);
    assert_true(abs(tlm.sun_z - 80) <= 1);
    assert_int_equal(tlm.sun_visible, 1);
    assert_int_equal(tlm.detumble_done, 0);
    assert_int_equal(tlm.mtq_x, 0);
}

/*===========================================================================*/
/* Runner                                                                     */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_calls_before_init_fail),   /* must stay first */
        cmocka_unit_test_setup(test_init_idle_coils_off, setup),
        cmocka_unit_test_setup(test_reinit_returns_to_idle, setup),
        cmocka_unit_test_setup(test_idle_keeps_coils_off, setup),
        cmocka_unit_test_setup(test_command_detumble_starts_bdot, setup),
        cmocka_unit_test_setup(test_detumble_complete_hands_over_to_sunpoint, setup),
        cmocka_unit_test_setup(test_rate_spike_restarts_settling, setup),
        cmocka_unit_test_setup(test_rate_above_threshold_never_settles, setup),
        cmocka_unit_test_setup(test_detumble_timeout_is_fault_not_completion, setup),
        cmocka_unit_test_setup(test_repeated_detumble_command_keeps_clock, setup),
        cmocka_unit_test_setup(test_new_detumble_restarts_clock, setup),
        cmocka_unit_test_setup(test_sunpoint_eclipse_cycle, setup),
        cmocka_unit_test_setup(test_eclipse_threshold_5_percent, setup),
        cmocka_unit_test_setup(test_safe_mode_minimal_until_commanded, setup),
        cmocka_unit_test_setup(test_set_mode_rejects_undefined_mode, setup),
        cmocka_unit_test_setup(test_bdot_law_sign_and_units, setup),
        cmocka_unit_test_setup(test_bdot_dissipates_rotational_energy, setup),
        cmocka_unit_test_setup(test_bdot_saturates_and_keeps_direction, setup),
        cmocka_unit_test_setup(test_bdot_invalid_inputs, setup),
        cmocka_unit_test_setup(test_detumble_drives_coils_against_bdot, setup),
        cmocka_unit_test_setup(test_rate_estimate_axis_and_magnitude, setup),
        cmocka_unit_test_setup(test_rate_estimate_needs_two_samples_and_field, setup),
        cmocka_unit_test_setup(test_sunpoint_torque_turns_plus_x_to_sun, setup),
        cmocka_unit_test_setup(test_sunpoint_aligned_and_saturation, setup),
        cmocka_unit_test_setup(test_sunpoint_invalid_inputs, setup),
        cmocka_unit_test_setup(test_apply_dipole_pwm_scaling, setup),
        cmocka_unit_test_setup(test_apply_dipole_clamps_out_of_range, setup),
        cmocka_unit_test_setup(test_saturate_dipole, setup),
        cmocka_unit_test_setup(test_vector_utilities, setup),
        cmocka_unit_test_setup(test_sensor_reads, setup),
        cmocka_unit_test_setup(test_telemetry_scaling, setup),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
