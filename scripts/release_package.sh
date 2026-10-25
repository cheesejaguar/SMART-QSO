#!/usr/bin/env bash
# Build and package a flight-software release (docs/development/RELEASE_PROCEDURE.md §5.2).
#
#   scripts/release_package.sh [--dev] OUT_DIR
#
# Builds the STM32L4R5ZI image with the pinned toolchain and the locked
# dependencies, and writes to OUT_DIR:
#   sqso-firmware-<version>.elf   the image, with symbols (for the debugger)
#   sqso-firmware-<version>.bin   the flash contents from 0x08000000
#   build_info.txt                commit, toolchain, versions, lock-file hashes, sizes
#   stack_usage.txt               the worst-case stack analysis (SRS-C003)
#   SHA256SUMS                    checksums of the files above
#
# A release needs a clean working tree and one version everywhere it is
# set (smart_qso.h, sqso-ffi's VERSION, both Cargo.toml), without "-dev".
# --dev packages a development build instead: the tree may be dirty and
# the version may end in "-dev" (CI does this to keep the script working).
set -euo pipefail

dev=0
if [ "${1:-}" = "--dev" ]; then dev=1; shift; fi
[ $# -eq 1 ] || { echo "usage: $0 [--dev] OUT_DIR" >&2; exit 2; }

repo="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$1"
out="$(cd "$1" && pwd)"
fw="$repo/software/rust/firmware"
fail() { echo "::error::$*" >&2; exit 1; }

# One version in four places
v_header=$(sed -n 's/^#define SMART_QSO_VERSION_STRING "\(.*\)"/\1/p' "$repo/software/flight/include/smart_qso.h")
v_ffi=$(sed -n 's/^pub const VERSION: &str = "\(.*\)";/\1/p' "$repo/software/rust/sqso-ffi/src/executive.rs")
v_ws=$(sed -n 's/^version = "\(.*\)"/\1/p' "$repo/software/rust/Cargo.toml" | head -1)
v_fw=$(sed -n 's/^version = "\(.*\)"/\1/p' "$fw/Cargo.toml" | head -1)
[ -n "$v_header" ] && [ "$v_header" = "$v_ffi" ] || fail "smart_qso.h ($v_header) and sqso-ffi ($v_ffi) versions differ"
[ "$v_ws" = "$v_fw" ] || fail "software/rust ($v_ws) and firmware ($v_fw) Cargo versions differ"
[ "${v_header%-dev}" = "$v_ws" ] || fail "C/Rust version $v_header does not match Cargo version $v_ws"
version="$v_header"

commit=$(git -C "$repo" rev-parse HEAD)
dirty=$(git -C "$repo" status --porcelain --untracked-files=no | wc -l)
if [ "$dev" -eq 0 ]; then
    [ "$dirty" -eq 0 ] || fail "the working tree has uncommitted changes"
    case "$version" in *-dev) fail "version $version is a development version";; esac
fi

# The image (rust-toolchain.toml pins the compiler; --locked the dependencies)
(cd "$fw" && cargo build --release --locked)
elf="$fw/target/thumbv7em-none-eabihf/release/sqso-firmware"
cp "$elf" "$out/sqso-firmware-$version.elf"
arm-none-eabi-objcopy -O binary "$elf" "$out/sqso-firmware-$version.bin"

python3 "$repo/scripts/stack_usage.py" "$elf" --firmware-src "$fw/src" --report "$out/stack_usage.txt" >/dev/null ||
    fail "stack analysis failed (see scripts/stack_usage.py)"

{
    echo "SMART-QSO flight software $version"
    echo "built:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "commit:  $commit$([ "$dirty" -eq 0 ] || echo " (with $dirty uncommitted changes)")"
    echo "release: $([ "$dev" -eq 0 ] && echo yes || echo "no (--dev)")"
    echo
    echo "== toolchain"
    (cd "$fw" && rustc -vV && cargo -V)
    arm-none-eabi-objcopy --version | head -1
    echo
    echo "== lock files (sha256)"
    (cd "$repo" && sha256sum software/rust/Cargo.lock software/rust/firmware/Cargo.lock)
    echo
    echo "== image"
    arm-none-eabi-size -A -x "$elf"
    echo
    echo "== stack (SRS-C003)"
    grep '^Worst case' "$out/stack_usage.txt"
} > "$out/build_info.txt"

(cd "$out" && sha256sum "sqso-firmware-$version.elf" "sqso-firmware-$version.bin" build_info.txt stack_usage.txt > SHA256SUMS)
echo "Packaged $version ($commit) in $out"
cat "$out/SHA256SUMS"
