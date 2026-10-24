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
# an executable. For a static library the dependency goes both ways, so
# anything linking it gets both archives, repeated to resolve the cycle.
function(link_rust target)
    get_target_property(kind ${target} TYPE)
    target_link_libraries(${target} sqso_rust)
    if(kind STREQUAL "STATIC_LIBRARY")
        set_property(TARGET sqso_rust APPEND PROPERTY INTERFACE_LINK_LIBRARIES ${target})
        set_property(TARGET ${target} PROPERTY LINK_INTERFACE_MULTIPLICITY 3)
    endif()
endfunction()
