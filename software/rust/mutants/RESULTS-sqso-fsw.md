# Mutation testing: sqso-fsw

Scope: the crate `sqso-fsw` (2180 mutants in 26 modules). Base commit 55c6f11. The tests that were added are in
commit cca9212 ("sqso-fsw: unit tests for the mutants cargo-mutants found unkilled"). No production code changed.

## Commands

```bash
# Rust unit tests (from software/rust)
cargo mutants -p sqso-fsw --in-place --timeout 120 -o OUT

# The missed mutants against the C suites (from the repository root, clean tree)
cmake -S software/flight -B build && cmake --build build -j8
python3 software/rust/mutants/ctest_missed.py OUT/mutants.out build
```

cargo-mutants 27.1.0, toolchain 1.97.0. For the "before" run, the C-suite pass used a copy of `ctest_missed.py`. It
was run in a scratch copy of the tree while the Rust run was still going: same patch, build and ctest steps, minus
the git cleanliness check, one mutant at a time. For the "after" run, the C-suite pass is `ctest_missed.py` itself,
run unchanged in the worktree.

## Numbers

| | total | caught | missed | unviable | timeout |
|---|---|---|---|---|---|
| Before (55c6f11), Rust unit tests | 2180 | 1671 | 355 | 153 | 1 |
| Before, missed ones against the C suites | 355 | 102 caught by C | 246 | 7 (CMake build failed) | – |
| After (cca9212), Rust unit tests | 2180 | 1991 | 35 | 153 | 1 |
| After, missed ones against the C suites | 35 | 0 caught by C | 35 | 0 | – |

- After the work, 35 mutants survive both the unit tests and the C suites. All 35 are equivalent (list below).
- The one timeout is the same in both runs: `mode.rs:135:15: replace += with *= in Event::from_raw`. With `i *= 1`
  starting from 0 the loop never ends, so the test hangs and cargo-mutants kills it. The mutant is detected.
- Before the work, 7 mutants did not build in the CMake (release) build, although `cargo test` built them. All 7
  replace `/` with `%` on `f64` in `adcs.rs`. The new unit tests catch all of them.
- Safety-critical modules, mutants surviving after the work: keys 0, telecommand 0, mode 0, rf_control 0,
  rf_silence 0, contact_monitor 0, deployment 3, payload_guard 0, eps 0, watchdog 0, fault_log 0, nvm 2. All five
  are equivalent: deployment's are disjoint flag bits, and nvm's are in the `MemFlash` test double.

Missed before, by module (Rust only, then still missed after the C suites): adcs 53/28, assert_handler 9/6,
beacon 31/17, deployment 26/23, eps 6/2, fault_log 5/4, flight_log 24/15, fmath 5/5, gnss 14/13, keys 10/3,
mission_data 3/2, mode 6/6, nvm 5/5, payload_guard 2/1, rf_control 1/1, rf_silence 1/1, safe_string 1/1,
scheduler 42/24, sensors 21/21, system_state 68/66, telemetry 7/2, watchdog 15/7. contact_monitor, executive,
telecommand and uart_comm had none.

## Equivalent survivors (35)

| Mutant | Why no behaviour differs |
|---|---|
| `adcs.rs:199:12: replace > with >= in saturate_dipole` | At `mag == max` the scale is `max / mag == 1.0` exactly, and `x * 1.0 == x` |
| `beacon.rs:40:34`, `40:43`, `40:53: replace \| with ^` (`MEASURED`) | The four flags are distinct single bits, so OR and XOR give the same value |
| `beacon.rs:288:29: replace < with <= in Out::bytes` | The only extra byte written is at `buf.len() - 1`. `Out` is private, and `format_payload` always calls `finish()`, which writes the NUL at `min(len, buf.len() - 1)` and so overwrites that byte |
| `beacon.rs:288:25: replace + with * in Out::bytes` | `len * 1 < buf.len()` is the `<=` mutant above |
| `beacon.rs:379:49`, `379:53: replace + with * in build_frame` | They enlarge the FCS scratch array (284 or 526 bytes instead of 272). Only its first `n` bytes are used. The `+`→`-` and `*` mutants, which shrink it, are killed |
| `beacon.rs:562:47: replace - with + / with / in Beacon::store_ai_text` (2) | Text is stored only if `text_ok` accepts it, and the filter accepts at most `filter::MAX_LEN` = 176 bytes. `min(len, 199)`, `min(len, 201)` and `min(len, 200)` are all `len` |
| `deployment.rs:333:13`, `334:13`, `335:13: replace \| with ^ in Deployment::record` | The four record flags are distinct single bits (0x01, 0x02, 0x04, 0x08), each term either 0 or its flag. The `\|`→`&` mutant is killed |
| `flight_log.rs:236:13: delete match arm level::TRACE in FlightLog::admit` | TRACE is below `MIN_LEVEL` (DEBUG) and is filtered out before the match, so the arm is unreachable |
| `flight_log.rs:259:76: replace - with + / with / in FlightLog::record` (2) | `safe_string::copy` limits the copy to `dest.len() - 1` = 15 bytes, so a bound of 15, 16 or 17 copies the same |
| `flight_log.rs:282:41: replace % with + in FlightLog::record` (read index) | `read` is used only as `(read + index) % BUFFER_SIZE`, and `read + 1 + 64` is congruent to `read + 1` mod 64. Overflow would need about 2.8e17 dropped entries |
| `fmath.rs:44:19`, `45:11`, `46:11` (2) (`r == 1 << 53` branch of `sqrt`) | Dead branch. `mant < 2^54` and is even after the shift, so `n <= 2^106 - 2^53`, `r = isqrt(n) <= 2^53 - 1`, and rounding up needs `n - r² > r`, that is `n > 2^106 - 2^53`. So `r` never reaches 2^53. With `>>` the test becomes `r == 0`, which is also never true (`r >= 2^52`) |
| `fmath.rs:50:35: replace \| with ^ in sqrt` | `frac < 2^52`, and `biased << 52` has its low 52 bits zero |
| `gnss.rs:243:58: replace \| with ^ in checksum_ok` | `hi << 4` and `lo < 16` share no bits |
| `nvm.rs:214:68`, `215:59: delete - in MemFlash::span` | These are error codes of the in-memory test and fuzz flash. A `u32` offset always converts to `usize`, and `start + len` cannot overflow for a slice length on the 64-bit host where `MemFlash` runs. The reachable out-of-range code (line 216) is now tested |
| `safe_string.rs:67:38: replace < with <= in compare` | This runs only when `c1 != c2`, and `as_c_char` is a bijection, so the two values are never equal |
| `sensors.rs:250:47: replace > with == / >= in eps_current` (2) | The simulated discharge current is 0.05, 0.8 or 0.25 A, never 1.0 A or more. The `<` mutant is killed |
| `sensors.rs:361:24: replace match guard !overflow with true in strtoul_u32` | Once `overflow` is set, the result is `c_ulong::MAX` whatever `value` holds |
| `sensors.rs:424:21: replace > with >= in YamlParser::finish` | At `len == 0` the extra `end_chunk` passes an empty line, and `line_of` returns at once on an empty line |
| `sensors.rs:443:28: replace \|\| with && in YamlParser::line_of` | An empty line still yields nothing: it is not `sensors:`, not an item marker, and has no key. A comment line's key starts with `#`, so it never matches a known field |
| `system_state.rs:350:13: delete field persistence_dirty` and `344:17: delete field external_temp_c` in `SystemState::defaults` | The deleted values (`false`, `0.0`) are what `..SystemState::default()` / `..Thermal::default()` supply |
| `system_state.rs:440:35: delete field crc32 in SystemState::compute_crc` | `image_crc` zeroes the CRC field of the image before computing the CRC |
| `telemetry.rs:215:40: replace \| with ^ in adcs` | Bit 0 (`detumbled`) and bit 1 (`sun_acquired`) are disjoint. The `<<`→`>>` mutant is killed |

## Observations (no behaviour change)

The new tests pin the current behaviour, which is also the original C behaviour (`adcs_control.c` before commit
df3a9c2). In three places the module doc comments read slightly differently from the code:

- The rate estimate needs readings more than 10 ms apart (`dt > 0.01`). The doc says "at least 10 ms apart".
- Sun pointing needs |B|² strictly above 1 uT². The doc says "a field of at least 1 uT".
- A detumble started at time 0 arms no timeout (`detumble_start_ms > 0`, as in the C).

No test revealed a bug.
