#!/usr/bin/env bash
# JPL Power of Ten rule 3: no dynamic memory allocation after initialization.
# Flight code allocates none at all. This fails if any flight object file
# references malloc, calloc, realloc or free.
#
# Usage: check_no_dynamic_memory.sh <object files or archives...>
#
# The simulated HAL (hal_sim.c) is excluded: it emulates flash with heap
# buffers on the host and is not flight code. Run by ctest
# (no_dynamic_memory) on every build.
set -euo pipefail

# ctest passes $<TARGET_OBJECTS:...> as one ";"-separated argument
objects=()
for arg in "$@"; do
    IFS=";" read -r -a parts <<< "$arg"
    objects+=("${parts[@]}")
done
if [ "${#objects[@]}" -eq 0 ]; then
    echo "no object files given"
    exit 1
fi

found=0
checked=0
for obj in "${objects[@]}"; do
    case "$obj" in
        *hal_sim.c.o|*hal_sim.c.obj) continue ;;
    esac
    if [ ! -f "$obj" ]; then
        echo "missing object file: $obj"
        exit 1
    fi
    checked=$((checked + 1))
    # Undefined (referenced) symbols; macOS prefixes C symbols with "_"
    if nm -u "$obj" 2>/dev/null | grep -qE '(^|[[:space:]])_?(malloc|calloc|realloc|free)$'; then
        echo "dynamic memory referenced by $(basename "$obj")"
        found=1
    fi
done

if [ "$found" -ne 0 ]; then
    exit 1
fi
echo "no dynamic memory in ${checked} object files (simulated HAL excluded)"
