/**
 * @file test_rust_contract_adcs.c
 * @brief The Rust ADCS module's view of adcs_control.h matches the header
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * adcs_control.h defines its own AdcsState_t, so it cannot share a
 * translation unit with system_state.h and test_rust_contract.c. This is the
 * same check for the types and constants of adcs_control.h that the Rust
 * library (sqso-ffi/src/adcs_control.rs, sqso-fsw/src/adcs.rs) mirrors.
 */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <setjmp.h>
#include <stdint.h>
#include <string.h>
#include <math.h>
#include <cmocka.h>

#include "rust_contract.h"
#include "adcs_control.h"

typedef struct {
    RustLayout_t id;
    const char *name;
    size_t size;
    size_t align;
} Layout_t;

#define LAYOUT(id, type) { id, #type, sizeof(type), _Alignof(type) }

static const Layout_t k_layouts[] = {
    LAYOUT(RUST_LAYOUT_ADCS_VEC3, Vec3_t),
    LAYOUT(RUST_LAYOUT_ADCS_MAG_DATA, MagData_t),
    LAYOUT(RUST_LAYOUT_ADCS_SUN_SENSOR_DATA, SunSensorData_t),
    LAYOUT(RUST_LAYOUT_ADCS_MTQ_COMMAND, MtqCommand_t),
    LAYOUT(RUST_LAYOUT_ADCS_CONTROL_STATE, AdcsState_t),
    LAYOUT(RUST_LAYOUT_ADCS_TELEMETRY, AdcsTelemetry_t),
};

_Static_assert(sizeof(k_layouts) / sizeof(k_layouts[0]) ==
               (size_t)(RUST_LAYOUT_END_200 - RUST_LAYOUT_ADCS_VEC3),
               "every mirrored adcs_control.h type is listed");

static void test_struct_layouts_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_layouts) / sizeof(k_layouts[0]); i++) {
        size_t size = 0U;
        size_t align = 0U;
        assert_true(sqso_rust_layout(k_layouts[i].id, &size, &align));
        if ((size != k_layouts[i].size) || (align != k_layouts[i].align)) {
            print_error("%s: C %zu/%zu, Rust %zu/%zu (size/align)\n", k_layouts[i].name,
                        k_layouts[i].size, k_layouts[i].align, size, align);
        }
        assert_int_equal(size, k_layouts[i].size);
        assert_int_equal(align, k_layouts[i].align);
    }
}

typedef struct {
    RustConst_t id;
    const char *name;
    double value;
} Constant_t;

#define CONSTANT(id, value) { id, #value, (double)(value) }

static const Constant_t k_constants[] = {
    CONSTANT(RUST_CONST_ADCS_MAX_DIPOLE, ADCS_MAX_DIPOLE),
    CONSTANT(RUST_CONST_ADCS_BDOT_GAIN, ADCS_BDOT_GAIN),
    CONSTANT(RUST_CONST_ADCS_SUNPOINT_KP, ADCS_SUNPOINT_KP),
    CONSTANT(RUST_CONST_ADCS_RATE_THRESHOLD, ADCS_DETUMBLE_RATE_THRESHOLD),
    CONSTANT(RUST_CONST_ADCS_SETTLING_MS, ADCS_DETUMBLE_SETTLING_MS),
    CONSTANT(RUST_CONST_ADCS_TIMEOUT_MS, ADCS_DETUMBLE_TIMEOUT_MS),
    CONSTANT(RUST_CONST_ADCS_SUN_SENSORS, ADCS_SUN_SENSOR_COUNT),
    CONSTANT(RUST_CONST_ADCS_MODE_SAFE, ADCS_MODE_SAFE),
    CONSTANT(RUST_CONST_FAULT_TYPE_ADCS, FAULT_TYPE_ADCS),
};

_Static_assert(sizeof(k_constants) / sizeof(k_constants[0]) ==
               (size_t)(RUST_CONST_END_200 - RUST_CONST_ADCS_MAX_DIPOLE),
               "every kept adcs_control.h constant is listed");

static void test_constants_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_constants) / sizeof(k_constants[0]); i++) {
        double rust = sqso_rust_constant(k_constants[i].id);
        /* Exactly the same value: compare the representations */
        bool same = memcmp(&rust, &k_constants[i].value, sizeof(rust)) == 0;
        if (!same) {
            print_error("%s: C %g, Rust %g\n", k_constants[i].name, k_constants[i].value, rust);
        }
        assert_true(same);
    }
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_struct_layouts_match),
        cmocka_unit_test(test_constants_match),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
