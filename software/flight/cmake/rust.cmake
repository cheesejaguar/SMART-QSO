# The Rust modules (software/rust): built with cargo as the static library
# libsqso_ffi.a and imported as the target sqso_rust.
#
# The library and the C flight code call each other, so they form a link
# cycle; link_rust(<target>) links both ways and CMake repeats the archives.
#
# The host build compiles for the host triple; the STM32L4 build compiles
# for thumbv7em-none-eabihf (Cortex-M4F, hard float), the same code.

find_program(CARGO cargo HINTS "$ENV{CARGO_HOME}/bin" "$ENV{HOME}/.cargo/bin")
if(NOT CARGO)
    message(FATAL_ERROR "cargo not found: the flight software needs Rust (https://rustup.rs; "
                        "see software/rust/README.md)")
endif()

set(SQSO_RUST_DIR "${CMAKE_CURRENT_LIST_DIR}/../../rust")
set(SQSO_CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo")

if(SMART_QSO_TARGET STREQUAL "STM32L4")
    set(SQSO_RUST_TRIPLE thumbv7em-none-eabihf)
    set(SQSO_RUST_LIB "${SQSO_CARGO_TARGET_DIR}/${SQSO_RUST_TRIPLE}/release/libsqso_ffi.a")
    set(SQSO_CARGO_TARGET_ARGS --target ${SQSO_RUST_TRIPLE})
else()
    set(SQSO_RUST_LIB "${SQSO_CARGO_TARGET_DIR}/release/libsqso_ffi.a")
    set(SQSO_CARGO_TARGET_ARGS "")
endif()

# cargo decides what to rebuild, so it runs on every build (a no-op when
# nothing changed)
add_custom_target(sqso_rust_build ALL
    COMMAND ${CMAKE_COMMAND} -E env --unset=MAKEFLAGS --unset=MFLAGS
            ${CARGO} build --release --locked -p sqso-ffi ${SQSO_CARGO_TARGET_ARGS}
            --target-dir "${SQSO_CARGO_TARGET_DIR}"
    WORKING_DIRECTORY "${SQSO_RUST_DIR}"
    BYPRODUCTS "${SQSO_RUST_LIB}"
    COMMENT "Building the Rust modules (sqso-ffi)"
    USES_TERMINAL
    VERBATIM
)

add_library(sqso_rust STATIC IMPORTED GLOBAL)
set_target_properties(sqso_rust PROPERTIES IMPORTED_LOCATION "${SQSO_RUST_LIB}")
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
            INTERFACE_LINK_LIBRARIES ${target})
        add_dependencies(${rust} sqso_rust_build)
        target_link_libraries(${target} ${rust})
        set_property(TARGET ${target} PROPERTY LINK_INTERFACE_MULTIPLICITY 3)
    else()
        target_link_libraries(${target} sqso_rust)
    endif()
endfunction()
