# The flight software (software/rust): built with cargo as the static library
# libsqso_flight.a (crate sqso-clib) and imported as the target sqso_rust.
#
# The library and the C flight code call each other, so they form a link
# cycle; link_rust(<target>) links both ways and CMake repeats the archives.
#
# This is the host build (the C test suites and the simulation). The target
# image is the Embassy firmware, software/rust/firmware, built by cargo.

find_program(CARGO cargo HINTS "$ENV{CARGO_HOME}/bin" "$ENV{HOME}/.cargo/bin")
if(NOT CARGO)
    message(FATAL_ERROR "cargo not found: the flight software needs Rust (https://rustup.rs; "
                        "see software/rust/README.md)")
endif()

set(SQSO_RUST_DIR "${CMAKE_CURRENT_LIST_DIR}/../../rust")
set(SQSO_CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo")

set(SQSO_RUST_LIB "${SQSO_CARGO_TARGET_DIR}/release/libsqso_flight.a")

# Coverage (ENABLE_COVERAGE, host only): the Rust library is instrumented
# for LLVM source-based coverage. rustc adds the profiling runtime only when
# it does the final link itself; here a C compiler links, so the runtime
# (the toolchain's profiler_builtins) is added explicitly. Each test then
# writes $LLVM_PROFILE_FILE; scripts/rust_coverage.sh merges and reports.
set(SQSO_RUSTFLAGS "")
add_library(sqso_rust_runtime INTERFACE)
if(ENABLE_COVERAGE)
    set(SQSO_RUSTFLAGS "RUSTFLAGS=-C instrument-coverage")
    find_program(RUSTC rustc HINTS "$ENV{CARGO_HOME}/bin" "$ENV{HOME}/.cargo/bin")
    # Run in the workspace so rust-toolchain.toml selects the pinned toolchain
    execute_process(COMMAND ${RUSTC} --print sysroot WORKING_DIRECTORY "${SQSO_RUST_DIR}"
                    OUTPUT_VARIABLE SQSO_RUST_SYSROOT OUTPUT_STRIP_TRAILING_WHITESPACE)
    execute_process(COMMAND ${RUSTC} --print host-tuple WORKING_DIRECTORY "${SQSO_RUST_DIR}"
                    OUTPUT_VARIABLE SQSO_RUST_HOST OUTPUT_STRIP_TRAILING_WHITESPACE)
    file(GLOB SQSO_PROFILER_RT "${SQSO_RUST_SYSROOT}/lib/rustlib/${SQSO_RUST_HOST}/lib/libprofiler_builtins-*.rlib")
    if(NOT SQSO_PROFILER_RT)
        message(FATAL_ERROR "Rust coverage: profiler_builtins not found in ${SQSO_RUST_SYSROOT}")
    endif()
    target_link_libraries(sqso_rust_runtime INTERFACE ${SQSO_PROFILER_RT})
    target_link_options(sqso_rust_runtime INTERFACE "LINKER:-u,__llvm_profile_runtime")
    message(STATUS "  Rust coverage:     on (${SQSO_PROFILER_RT})")
endif()

# cargo decides what to rebuild, so it runs on every build (a no-op when
# nothing changed)
add_custom_target(sqso_rust_build ALL
    COMMAND ${CMAKE_COMMAND} -E env --unset=MAKEFLAGS --unset=MFLAGS ${SQSO_RUSTFLAGS}
            ${CARGO} build --release --locked -p sqso-clib
            --target-dir "${SQSO_CARGO_TARGET_DIR}"
    WORKING_DIRECTORY "${SQSO_RUST_DIR}"
    BYPRODUCTS "${SQSO_RUST_LIB}"
    COMMENT "Building the flight software library (sqso-clib)"
    USES_TERMINAL
    VERBATIM
)

add_library(sqso_rust STATIC IMPORTED GLOBAL)
set_target_properties(sqso_rust PROPERTIES IMPORTED_LOCATION "${SQSO_RUST_LIB}"
                                           INTERFACE_LINK_LIBRARIES sqso_rust_runtime)
add_dependencies(sqso_rust sqso_rust_build)

# Link the Rust library into <target>, a static library of C flight code or
# an executable. The Rust library calls back into the C library, so for a
# static library the two depend on each other: <target> gets its own
# imported copy of the Rust archive that depends on <target> in turn, and
# CMake repeats the pair on the link line. (A per-library copy keeps the
# cycle local: the shared sqso_rust target stays free of any one library.)
function(link_rust target)
    get_target_property(kind ${target} TYPE)
    if(kind STREQUAL "STATIC_LIBRARY")
        set(rust sqso_rust_for_${target})
        add_library(${rust} STATIC IMPORTED)
        set_target_properties(${rust} PROPERTIES
            IMPORTED_LOCATION "${SQSO_RUST_LIB}"
            INTERFACE_LINK_LIBRARIES "${target};sqso_rust_runtime")
        add_dependencies(${rust} sqso_rust_build)
        target_link_libraries(${target} ${rust})
        set_property(TARGET ${target} PROPERTY LINK_INTERFACE_MULTIPLICITY 3)
    else()
        target_link_libraries(${target} sqso_rust)
    endif()
endfunction()
