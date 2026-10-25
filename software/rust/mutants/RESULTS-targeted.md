# Targeted mutation checks on the Rust implementation

Scope "targeted": the mutation checks that the documentation records as done on the former C flight code, repeated on
the Rust code that replaced it (base commit 55c6f11, 2026-10-24). The claims:

- `docs/funding_audit/FINDINGS_REGISTER.md` rows SW-05, SW-08, SW-10, SW-12, SW-14, SW-15 and SW-16;
- `docs/interfaces/TELECOMMAND_AUTHENTICATION.md` §10: the GSE umbilical gate, the contact hook's gate and placement,
  the RF-silence reset-cause rule, completion persistence and switch reset, and the AX.25 callsign filter.

The C commits name only some of their mutations: c1cc9a7 (RF silence: "POR resuming, completion not persisted,
switch ignored"), a4ca3d2 (contact hook: "removing, ungating or moving the hook before commit"), ce63dc8 (watchdog:
"the gate and the save"), 2b1ef80 (assertions: "both halves"), 6d51847 and 9f25076 ("every guard rule", "every
rule"), 6dd7651 ("7 mutants", not listed), 101ae5e and 165ca56 (GSE gate, callsign filter). Each named mutation is
repeated here; where a commit gives only the behaviour, each rule of that behaviour gets at least one mutation.

## How it was run

The mutations are hand-written edits in `software/rust/mutants/targeted.py` (old text → new text; each old text must
occur exactly once, so a drift in the code fails the script instead of testing nothing). From the repository root:

```bash
python3 software/rust/mutants/targeted.py            # verify: every mutant killed by its named tests
python3 software/rust/mutants/targeted.py ID ...     # some mutants only
python3 software/rust/mutants/targeted.py --list     # ids, claims, behaviours
python3 software/rust/mutants/targeted.py --survey   # every failing test per mutant (used to choose the kills)
```

Verify mode first checks that every named killing test passes on the unmutated tree, then for each mutant applies the
edit, builds (`cargo test -p CRATE --lib -- --exact TEST` for Rust unit tests; `cmake --build build` and `ctest -R`
for C cases, matching the CMocka `[  FAILED  ] case` line), requires every named test to fail, and restores the file
(also on SIGTERM or an exception). A mutant that does not build is reported as a problem, not as killed. Survey mode
runs `cargo test -p sqso-protocol -p sqso-fsw` (the only crates with unit tests) and all 43 ctest suites.

Last verify run: `106/106 mutants killed by every named test`, 18 minutes on a 4-core host (most of it the release
LTO build of the Rust library for the C suites).

## Results

| | Mutants | Killed | Missed | Unviable | Timeout | Equivalent |
|-|--------:|-------:|-------:|---------:|--------:|-----------:|
| Before (existing tests only, survey) | 106 | 103 | 3 | 0 | 0 | 0 |
| After (with the tests below, verify) | 106 | 106 | 0 | 0 | 0 | 0 |

Missed before, now killed by a new test:

- `filter-length` (the 176-character limit off by one): no vector sits at the limit. New
  `sqso-protocol` `filter::tests::length_limit_is_inclusive`.
- `filter-url-case` (URL fragments matched case-sensitively): every URL vector is lower case. New
  `filter::tests::url_marks_in_any_letter_case`.
- `contact-hook-removed-unpersisted` (no contact for an authentic frame whose counter was spent but not persisted):
  observable only through the C ABI. New C case `test_cmd_handler.c` `test_unpersisted_counter_is_still_contact`.

Killed before only by one side, now also by a Rust unit test (so `cargo test` alone catches them):

- `filter-pipe` ('|' allowed): only the C fuzz property caught it, because the one '|' vector also holds '='. New
  `filter::tests::charset_excludes_the_field_separator`.
- `ax25-call-ignored` (call sign not compared): only C `test_ax25/test_parse_reference_frame`. New
  `ax25::tests::other_call_with_our_ssid_does_not_match`.
- `mode-entry-active`, `mode-entry-safe`, `mode-entry-emergency-beacon` (entry actions): only the C suites. New
  `mode::tests::entry_actions`.
- `policy-emergency-threshold` (EMERGENCY at 0.25 instead of 0.15): only C `test_sm_power_event_policy`. New
  `mode::tests::power_policy_thresholds`, which checks every threshold at the boundary.

No equivalent mutants; no production code changed; no test revealed a bug.

The tests another agent added for the contact hook and the RF silence (`telecommand::tests::contact_hook_gate_and_placement`,
`rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept`, `completion_persists_across_power_on_resets`,
`stowed_switch_restarts_even_after_completion`) were checked, not redone: each kills the mutations its doc comment
names (`contact-*`, `rfs-*` below).

Not covered: the executive's cut of the payload rail on a guard trip (`sqso-ffi/src/executive.rs`), which the SW-16
claim ("every guard rule") does not include, and the boot-image CRC (SW-16, not implemented).

## One row per claim

| Claim | Behaviour | Mutants | Killing tests |
|-------|-----------|--------:|---------------|
| SW-05, TC-AUTH §10 RF silence | 45-min silence: power-on/brownout/pin/unknown restarts, watchdog/software resumes and never shortens; completion persists across power-on resets; a stowed switch restarts, even after completion, persisted; ARMED waits for the next boot; GSE ARM_FLIGHT re-arms; the radio is unpowered; TX_ENABLE cannot lift it | 13 | `rf_silence::tests::{reset_cause_rule_resume_only_when_power_was_kept, completion_persists_across_power_on_resets, stowed_switch_restarts_even_after_completion, arm_holds_until_the_next_boot}`, `rf_control::tests::automatic_inhibits_are_independent`; C `test_rf_silence` (8 cases), `test_gse_provision/test_arm_flight` |
| TC-AUTH §10 contact hook | contact only for a keyed frame whose counter was spent (gate: not the keyless TX_DISABLE; placement: not framing, authentication, unknown-command or argument-length rejections); the hook runs for accepted and for spent-but-unpersisted frames | 6 | `telecommand::tests::contact_hook_gate_and_placement`; C `test_cmd_handler/{test_unauthenticated_frames_are_not_contact, test_authentic_command_restores_contact, test_unpersisted_counter_is_still_contact}` |
| TC-AUTH §10 GSE umbilical gate | no provisioning frame executed, and umbilical-port bytes discarded, without the umbilical sense line | 3 | C `test_gse_provision/{test_locked_without_umbilical, test_poll_reads_umbilical_uart}` |
| TC-AUTH §10 AX.25 callsign filter | only frames whose destination (not source) has this call sign and SSID reach the handler | 4 | `ax25::tests::{address_encoding, other_call_with_our_ssid_does_not_match}`; C `test_uplink/{test_other_frames_dropped, test_received_tx_disable_silences_station}`, `test_ax25/test_parse_reference_frame` |
| SW-08 assertions | reporting: a failed `SMART_QSO_REQUIRE` or Rust check reaches the handler, is counted and logged; early return: the macro returns, `eps_set_power_mode` does not store an invalid mode | 6 | `assert_handler::tests::severities_and_counts`; C `test_assert_release/{test_failure_reported_and_returns_error, test_flight_code_refuses_bad_input}` |
| SW-10 deployment | a sensed element is never fired; the master arm drops after a burn; nothing fires if the attempt is not persisted, and it is persisted before the burn; a reset mid-burn spends the attempt; attempts bounded across resets; the 30-min wait restarts on power-on and on a stowed switch, resumes after a watchdog reset, is never shortened; COMPLETE/FAILED ignore the switch; `force_retry` cannot bypass the wait; GSE ARM_FLIGHT re-arms | 14 | `deployment::tests` (9 tests); C `test_deployment` (10 cases), `test_gse_provision/test_arm_flight` |
| SW-12 content filter | length, character set (`\|`, `$`, `@`), URLs in any case, commercial words, profanity (whole words), overlong, vowel-less, hex-only and scattered-capital words, call signs, SSID, numbers but 73 and 1U; the OBC filters Jetson text and counts rejections | 21 | `filter::tests::{shared_vectors, every_list_word_is_caught, ssid_rules, length_limit_is_inclusive, charset_excludes_the_field_separator, url_marks_in_any_letter_case}`, `beacon::tests::given_text_must_pass_the_filter`; C `test_beacon_filter/{test_shared_vectors, test_word_lists, test_fuzz_properties, test_rejected_ai_text_falls_back}` |
| SW-14 mode manager | EMERGENCY reachable (the former shadowed rows); every state reachable; every way into ACTIVE guarded, the guard enforced, on battery ≥ 0.55 and the payload guard; entry actions run and set the EPS mode and the beacon; policy table 0.15/0.25/0.40/0.55 with hysteresis | 16 | `mode::tests::{every_state_is_reachable_from_boot, every_way_into_active_is_guarded, guard_and_steps, entry_actions, power_policy_hysteresis, power_policy_thresholds}`; C `test_state_machine/{test_sm_every_state_reachable, test_sm_critical_power, test_sm_active_guard, test_sm_entry_sets_power_mode, test_sm_power_event_policy}` |
| SW-15 watchdog | no refresh while a task is stalled; the stall is seen; state saved once per stall; a check-in recovers; a task within its period is healthy | 6 | `watchdog::tests::stall_after_three_misses_reported_once`; C `test_watchdog_mgr/test_stalled_task_lets_hardware_reset`, `test_fault_scenarios/test_task_stall_detection` |
| SW-16 payload guard | > 8 W twice in a row (8 W exactly is not over); ≥ 12 W once; spreader ≥ 70 °C; junction ≥ 95 °C, only if reported; no power or spreader reading for 5 s; clocks restart at rail-on; 10-min cool-down, then a recent reading below 60 °C; the EPS refuses enable meanwhile; bus-voltage range rejects NaN | 17 | `payload_guard::tests` (5 tests), `eps::tests::soc_and_bus_limits`; C `test_payload_guard` (6 cases), `EPS_Control_Tests/test_eps_bus_voltage_ok` |
| **Total** | | **106** | |

## Every mutant

Rust tests are `crate` `module::tests::name`; C cases are `suite/case` (ctest suite, CMocka case). "(new)" marks a
test added by this work. Each listed test fails with the mutant applied (verify mode); the survey output also lists
the other tests that fail.

### SW-05

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `rfs-por-resumes` | reset-cause rule: a power-on/brownout/pin/unknown reset restarts the count | sqso-fsw/src/rf_silence.rs: `_ => Reset::PowerLost,` → `_ => Reset::PowerKept,` | `sqso-fsw` `rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept`<br>C `test_rf_silence/test_power_loss_restarts` |
| `rfs-any-reset-resumes` | reset-cause rule: only a reset that kept power resumes | sqso-fsw/src/rf_silence.rs: `state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S` → `state == State::Counting as u32 && elapsed_s < DURATION_S` | `sqso-fsw` `rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept`<br>C `test_rf_silence/test_power_loss_restarts` |
| `rfs-watchdog-restarts` | reset-cause rule: a watchdog or software reset resumes the count | sqso-fsw/src/rf_silence.rs: `state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S` → `state == State::Counting as u32 && reset == Reset::PowerLost && elapsed_s < DURATION_S` | `sqso-fsw` `rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept`<br>C `test_rf_silence/test_watchdog_reset_resumes` |
| `rfs-resume-past-duration` | a resumed count never shortens the silence (count >= 45 min not trusted) | sqso-fsw/src/rf_silence.rs: `state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S` → `state == State::Counting as u32 && reset == Reset::PowerKept` | `sqso-fsw` `rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept` |
| `rfs-completion-not-saved` | completion persistence: COMPLETE is written when reached | sqso-fsw/src/rf_silence.rs: `self.state = State::Complete; / self.save(io);` → `self.state = State::Complete;` | `sqso-fsw` `rf_silence::tests::completion_persists_across_power_on_resets`<br>C `test_rf_silence/test_completion_persists` |
| `rfs-completion-lost-on-por` | completion persistence: a power-on reset keeps COMPLETE | sqso-fsw/src/rf_silence.rs: `if state == State::Complete as u32 {` → `if state == State::Complete as u32 && reset == Reset::PowerKept {` | `sqso-fsw` `rf_silence::tests::completion_persists_across_power_on_resets`<br>C `test_rf_silence/test_completion_persists` |
| `rfs-switch-ignored` | switch reset: a stowed switch seen while powered restarts the count | sqso-fsw/src/rf_silence.rs: `if io.stowed() {` → `if false {` | `sqso-fsw` `rf_silence::tests::stowed_switch_restarts_even_after_completion`<br>C `test_rf_silence/test_switch_toggle_restarts` |
| `rfs-switch-ignored-after-completion` | switch reset: also after completion | sqso-fsw/src/rf_silence.rs: `if self.state != State::Counting \|\| self.elapsed_s != 0 {` → `if self.state == State::Counting && self.elapsed_s != 0 {` | `sqso-fsw` `rf_silence::tests::stowed_switch_restarts_even_after_completion`<br>C `test_rf_silence/test_stowed_after_completion_rearms` |
| `rfs-switch-restart-not-saved` | switch reset: the restart is persisted | sqso-fsw/src/rf_silence.rs: `self.restart_count(); / self.save(io);` → `self.restart_count();` | `sqso-fsw` `rf_silence::tests::stowed_switch_restarts_even_after_completion` |
| `rfs-armed-counts` | GSE ARM: ARMED does not count while powered | sqso-fsw/src/rf_silence.rs: `if self.state != State::Counting { / return; // COMPLETE` → `if self.state == State::Complete { / return; // COMPLETE` | `sqso-fsw` `rf_silence::tests::arm_holds_until_the_next_boot`<br>C `test_rf_silence/test_arm_resets_completed_unit` |
| `rfs-gse-arm-skipped` | GSE ARM_FLIGHT re-arms the RF silence | sqso-ffi/src/gse_provision.rs: `let silence = unsafe { c::rf_silence_arm() };` → `let silence = rc::OK;` | C `test_gse_provision/test_arm_flight` |
| `rfs-radio-powered` | the whole radio stays unpowered during the silence | sqso-fsw/src/rf_control.rs: `let radio_on = self.auto_inhibits & INHIBIT_RF_SILENCE == 0;` → `let radio_on = true;` | `sqso-fsw` `rf_control::tests::automatic_inhibits_are_independent`<br>C `test_rf_silence/test_silent_for_45_minutes` |
| `rfs-tx-enable-lifts` | no telecommand shortens the silence (TX_ENABLE clears the latch only) | sqso-fsw/src/rf_control.rs: `self.inhibited = false; / self.drive(io);` → `self.inhibited = false; / self.auto_inhibits = 0; / self.drive(io);` | `sqso-fsw` `rf_control::tests::automatic_inhibits_are_independent`<br>C `test_rf_silence/test_tx_enable_cannot_shorten_silence` |

### TC-AUTH contact hook

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `contact-gate` | gate: the keyless TX_DISABLE is not contact | sqso-fsw/src/telecommand.rs: `let contact = verified.key != KeyUsed::None;` → `let contact = true;` | `sqso-fsw` `telecommand::tests::contact_hook_gate_and_placement`<br>C `test_cmd_handler/test_unauthenticated_frames_are_not_contact` |
| `contact-before-arglen` | placement: a rejection before the counter is spent is not contact | sqso-fsw/src/telecommand.rs: `return reject(cmd_id, counter, Rejection::ArgLength, false);` → `return reject(cmd_id, counter, Rejection::ArgLength, true);` | `sqso-fsw` `telecommand::tests::contact_hook_gate_and_placement`<br>C `test_cmd_handler/test_unauthenticated_frames_are_not_contact` |
| `contact-before-unknown` | placement: an unknown command is not contact | sqso-fsw/src/telecommand.rs: `None => return reject(cmd_id, counter, Rejection::UnknownCommand, false),` → `None => return reject(cmd_id, counter, Rejection::UnknownCommand, true),` | `sqso-fsw` `telecommand::tests::contact_hook_gate_and_placement` |
| `contact-before-auth` | placement: a forged or replayed frame is not contact | sqso-fsw/src/telecommand.rs: `Err(e) => return reject(cmd_id, None, Rejection::AuthFailed(e), false),` → `Err(e) => return reject(cmd_id, None, Rejection::AuthFailed(e), true),` | `sqso-fsw` `telecommand::tests::contact_hook_gate_and_placement`<br>C `test_cmd_handler/test_unauthenticated_frames_are_not_contact` |
| `contact-hook-removed` | hook: an accepted keyed command is contact | sqso-ffi/src/cmd_handler.rs: `Admission::Accepted { command, contact: was_contact } => { / if was_contact {` → `Admission::Accepted { command, contact: was_contact } => { / if was_contact && false {` | C `test_cmd_handler/test_authentic_command_restores_contact` |
| `contact-hook-removed-unpersisted` | hook: a spent counter that could not be persisted is still contact | sqso-ffi/src/cmd_handler.rs: `if was_contact { / contact(); / } / let (code, reason)` → `if was_contact && false { / contact(); / } / let (code, reason)` | C `test_cmd_handler/test_unpersisted_counter_is_still_contact` (new) |

### TC-AUTH GSE gate

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `gse-gate-frame` | no provisioning frame is executed without the umbilical | sqso-ffi/src/gse_provision.rs: `-> u32 { / if !gse_umbilical_present() {` → `-> u32 { / if false {` | C `test_gse_provision/test_locked_without_umbilical` |
| `gse-gate-poll` | bytes on the umbilical port without the umbilical are discarded | sqso-ffi/src/gse_provision.rs: `unsafe { / if !gse_umbilical_present() {` → `unsafe { / if false {` | C `test_gse_provision/test_poll_reads_umbilical_uart` |
| `gse-sense-ignored` | the umbilical sense line decides | sqso-ffi/src/gse_provision.rs: `unsafe { c::hal_gpio_get(GPIO_PIN_UMBILICAL_SENSE) }` → `true` | C `test_gse_provision/test_locked_without_umbilical` |

### TC-AUTH callsign filter

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `ax25-dest-ignored` | only frames addressed to this station reach the handler | sqso-ffi/src/uplink.rs: `if !ax25::address_matches(ui.dest, callsign.get(..len).unwrap_or(&[]), ssid) {` → `if false {` | C `test_uplink/test_other_frames_dropped` |
| `ax25-src-checked` | the destination, not the source, is matched | sqso-ffi/src/uplink.rs: `ax25::address_matches(ui.dest,` → `ax25::address_matches(ui.src,` | C `test_uplink/test_received_tx_disable_silences_station` |
| `ax25-ssid-ignored` | the SSID must match | sqso-protocol/src/ax25.rs: `&& (addr[CALL_LEN] & SSID_MASK) == (expected[CALL_LEN] & SSID_MASK)` → (deleted) | `sqso-protocol` `ax25::tests::address_encoding`<br>C `test_uplink/test_other_frames_dropped` |
| `ax25-call-ignored` | the call sign must match | sqso-protocol/src/ax25.rs: `addr[..CALL_LEN] == expected[..CALL_LEN] &&` → (deleted) | `sqso-protocol` `ax25::tests::other_call_with_our_ssid_does_not_match` (new)<br>C `test_ax25/test_parse_reference_frame` |

### SW-08

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `assert-not-reported` | reporting: a failed SMART_QSO_REQUIRE reaches the assertion handler | sqso-ffi/src/assert_handler.rs: `let _ = unsafe { assert_handle_failure(file, short_line, kind::PRECONDITION, severity::ERROR, message) };` → `let _ = (short_line, kind::PRECONDITION);` | C `test_assert_release/test_failure_reported_and_returns_error` |
| `assert-not-counted` | reporting: the failure is counted | sqso-fsw/src/assert_handler.rs: `s.total_failures = s.total_failures.wrapping_add(1);` → (deleted) | `sqso-fsw` `assert_handler::tests::severities_and_counts`<br>C `test_assert_release/test_failure_reported_and_returns_error` |
| `assert-not-logged` | reporting: the failure is logged | sqso-fsw/src/assert_handler.rs: `self.log_failure(file_name(file), line, kind, severity, message, uptime_s);` → (deleted) | `sqso-fsw` `assert_handler::tests::severities_and_counts`<br>C `test_assert_release/test_failure_reported_and_returns_error` |
| `assert-macro-no-return` | early return: a failed SMART_QSO_REQUIRE returns an error (C macro) | include/smart_qso.h: `return (retval); \` → `(void)(retval); \` | C `test_assert_release/test_failure_reported_and_returns_error` |
| `assert-rust-check-not-reported` | reporting: a failed check in Rust reaches the assertion handler | sqso-ffi/src/c.rs: `unsafe { smart_qso_assert_failed(c"sqso-ffi".as_ptr(), line, message.as_ptr()) };` → `let _ = (message, line);` | C `test_assert_release/test_flight_code_refuses_bad_input` |
| `assert-eps-stores-invalid` | early return: eps_set_power_mode does not store an invalid mode | sqso-ffi/src/eps_control.rs: `let Some(power_mode) = eps::power_mode(mode) else { / c::assert_failed(c"Invalid power mode", line!()); / return rc::INVALID; / };` → `let power_mode = match eps::power_mode(mode) { / Some(p) => p, / None => { / c::assert_failed(c"Invalid power mode", line!()); / PowerMode::Active / } / };` | C `test_assert_release/test_flight_code_refuses_bad_input` |

### SW-10

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `deploy-sensed-fired` | idempotent: an element sensed as deployed is never fired | sqso-fsw/src/deployment.rs: `if self.element(e).deployed \|\| io.sensed(e) {` → `if self.element(e).deployed {` | `sqso-fsw` `deployment::tests::already_deployed_is_never_fired`<br>C `test_deployment/test_already_deployed_not_fired` |
| `deploy-arm-left-on` | a burn needs the master arm: the arm drops when a burn ends | sqso-fsw/src/deployment.rs: `fn burn_off<IO: DeployIo>(io: &mut IO) { / io.set(Line::Arm, false);` → `fn burn_off<IO: DeployIo>(io: &mut IO) {` | `sqso-fsw` `deployment::tests::full_sequence_one_element_at_a_time`<br>C `test_deployment/test_full_sequence` |
| `deploy-burn-without-persist` | nothing fires if the attempt cannot be persisted | sqso-fsw/src/deployment.rs: `if !self.save(io) {` → `if !self.save(io) && false {` | `sqso-fsw` `deployment::tests::attempt_persisted_before_the_burn`<br>C `test_deployment/test_persist_failure_blocks_burn` |
| `deploy-burn-before-persist` | the attempt is persisted before the burn line is driven | sqso-fsw/src/deployment.rs: `self.enter_state(io, active); / if !self.save(io) {` → `self.enter_state(io, active); / Self::burn_on(io, e); / if !self.save(io) {` | `sqso-fsw` `deployment::tests::attempt_persisted_before_the_burn`<br>C `test_deployment/test_persist_failure_blocks_burn` |
| `deploy-reset-refires` | a reset mid-burn spends the attempt (verify, never re-fire) | sqso-fsw/src/deployment.rs: `State::AntennaActive => State::AntennaVerify,` → `State::AntennaActive => State::AntennaPending,` | `sqso-fsw` `deployment::tests::reset_during_burn_spends_the_attempt`<br>C `test_deployment/test_reset_during_burn_spends_attempt` |
| `deploy-attempts-not-restored` | attempts are bounded across brownout loops | sqso-fsw/src/deployment.rs: `el.attempt_count = attempts;` → `el.attempt_count = 0;` | `sqso-fsw` `deployment::tests::brownout_loop_is_bounded_by_persisted_attempts`<br>C `test_deployment/test_burn_brownout_loop_is_bounded` |
| `deploy-attempts-unbounded` | attempts are bounded (FAILED after MAX_RETRIES) | sqso-fsw/src/deployment.rs: `if self.element(e).attempt_count >= MAX_RETRIES {` → `if false {` | `sqso-fsw` `deployment::tests::brownout_loop_is_bounded_by_persisted_attempts`<br>`sqso-fsw` `deployment::tests::disabled_holds_and_failure_allows_a_ground_retry`<br>C `test_deployment/test_burn_brownout_loop_is_bounded` |
| `deploy-por-resumes` | the 30-min wait restarts on a power-on reset | sqso-fsw/src/deployment.rs: `(_, Reset::PowerLost) => {} / (_, Reset::PowerKept) => {` → `(_, Reset::PowerLost \| Reset::PowerKept) => {` | `sqso-fsw` `deployment::tests::wait_resumes_only_when_power_was_kept_and_restarts_when_stowed`<br>C `test_deployment/test_power_on_reset_restarts_wait` |
| `deploy-watchdog-restarts` | the 30-min wait resumes after a watchdog reset | sqso-fsw/src/deployment.rs: `(_, Reset::PowerLost) => {} / (_, Reset::PowerKept) => {` → `(_, Reset::PowerLost \| Reset::PowerKept) => {} / #[allow(unreachable_patterns)] / (_, Reset::PowerKept) => {` | `sqso-fsw` `deployment::tests::wait_resumes_only_when_power_was_kept_and_restarts_when_stowed`<br>C `test_deployment/test_watchdog_reset_resumes_wait` |
| `deploy-switch-ignored` | a stowed switch restarts the wait (CDS 2.3.2.4) | sqso-fsw/src/deployment.rs: `if io.stowed() {` → `if false {` | `sqso-fsw` `deployment::tests::wait_resumes_only_when_power_was_kept_and_restarts_when_stowed`<br>C `test_deployment/test_stowed_switch_restarts_wait` |
| `deploy-terminal-follows-switch` | COMPLETE and FAILED ignore the switch | sqso-fsw/src/deployment.rs: `if matches!(self.state, State::Complete \| State::Failed \| State::Armed) {` → `if matches!(self.state, State::Armed) {` | `sqso-fsw` `deployment::tests::terminal_states_persist_and_ignore_the_switch_arm_resets`<br>C `test_deployment/test_complete_persists_and_ignores_switch` |
| `deploy-force-retry-bypass` | force_retry no longer bypasses the 30-min wait | sqso-fsw/src/deployment.rs: `State::Inhibited \| State::Waiting \| State::Armed \| State::AntennaActive \| State::SolarActive => {` → `State::Armed \| State::AntennaActive \| State::SolarActive => {`; `State::AntennaPending / \| State::AntennaVerify` → `State::Inhibited / \| State::Waiting / \| State::AntennaPending / \| State::AntennaVerify` | `sqso-fsw` `deployment::tests::force_retry_cannot_bypass_the_wait_and_names`<br>C `test_deployment/test_force_retry_cannot_bypass_wait` |
| `deploy-wait-shortened` | no burn before the 30-min wait is served | sqso-fsw/src/deployment.rs: `if self.wait_elapsed_s >= WAIT_TIME_S && self.deployment_enabled {` → `if self.wait_elapsed_s >= WAIT_TIME_S - 60 && self.deployment_enabled {` | `sqso-fsw` `deployment::tests::full_sequence_one_element_at_a_time`<br>C `test_deployment/test_full_sequence` |
| `deploy-gse-arm-skipped` | GSE ARM_FLIGHT re-arms the deployment sequence | sqso-ffi/src/gse_provision.rs: `let deploy = unsafe { c::deployment_arm() };` → `let deploy = rc::OK;` | C `test_gse_provision/test_arm_flight` |

### SW-12

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `filter-pipe` | character set: no '/' | sqso-protocol/src/filter.rs: `pub const PUNCTUATION: &str = " .,!?'\":;()/-&";` → `pub const PUNCTUATION: &str = " .,!?'\":;()/-&\|";` | `sqso-protocol` `filter::tests::charset_excludes_the_field_separator` (new)<br>C `test_beacon_filter/test_fuzz_properties` |
| `filter-dollar` | character set: no '$' | sqso-protocol/src/filter.rs: `pub const PUNCTUATION: &str = " .,!?'\":;()/-&";` → `pub const PUNCTUATION: &str = " .,!?'\":;()/-&$";` | `sqso-protocol` `filter::tests::shared_vectors`<br>`sqso-protocol` `filter::tests::charset_excludes_the_field_separator` (new)<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-at` | character set: no '@' | sqso-protocol/src/filter.rs: `pub const PUNCTUATION: &str = " .,!?'\":;()/-&";` → `pub const PUNCTUATION: &str = " .,!?'\":;()/-&@";` | `sqso-protocol` `filter::tests::shared_vectors`<br>`sqso-protocol` `filter::tests::charset_excludes_the_field_separator` (new)<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-charset-off` | character set enforced | sqso-protocol/src/filter.rs: `if !text.iter().all(\|c\| allowed(*c)) {` → `if false {` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-length` | length: 1 to 176 characters | sqso-protocol/src/filter.rs: `if text.is_empty() \|\| text.len() > MAX_LEN {` → `if text.is_empty() \|\| text.len() > MAX_LEN + 1 {` | `sqso-protocol` `filter::tests::length_limit_is_inclusive` (new) |
| `filter-url-off` | no URLs | sqso-protocol/src/filter.rs: `if URL_MARKS.iter().any(` → `if false && URL_MARKS.iter().any(` | `sqso-protocol` `filter::tests::shared_vectors`<br>`sqso-protocol` `filter::tests::url_marks_in_any_letter_case` (new)<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-url-case` | no URLs, in any letter case | sqso-protocol/src/filter.rs: `text.windows(lit.len()).any(\|w\| w.eq_ignore_ascii_case(lit))` → `text.windows(lit.len()).any(\|w\| w == lit)` | `sqso-protocol` `filter::tests::url_marks_in_any_letter_case` (new) |
| `filter-commercial-off` | no commercial words | sqso-protocol/src/filter.rs: `if in_list(word, COMMERCIAL) {` → `if false {` | `sqso-protocol` `filter::tests::every_list_word_is_caught`<br>C `test_beacon_filter/test_word_lists` |
| `filter-profanity-off` | no profanity | sqso-protocol/src/filter.rs: `if in_list(word, PROFANITY) {` → `if false {` | `sqso-protocol` `filter::tests::every_list_word_is_caught`<br>C `test_beacon_filter/test_word_lists` |
| `filter-substring-match` | word lists match whole words only | sqso-protocol/src/filter.rs: `word.eq_ignore_ascii_case(lit)` → `contains_ignore_case(word, lit)` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-overlong-off` | encoded-looking: overlong words | sqso-protocol/src/filter.rs: `if word.len() > MAX_WORD {` → `if word.len() > MAX_WORD + 20 {` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-vowelless-off` | encoded-looking: vowel-less words | sqso-protocol/src/filter.rs: `(all_alpha && word.len() >= 5 && !vowel) \|\|` → `false \|\|` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-hex-off` | encoded-looking: hex-only words | sqso-protocol/src/filter.rs: `(all_hex && word.len() >= 8) \|\|` → `false \|\|` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-case-changes-off` | encoded-looking: scattered capitals | sqso-protocol/src/filter.rs: `\|\| case_changes > 2` → `\|\| case_changes > 20` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-callsign-off` | no call sign but the station's own | sqso-protocol/src/filter.rs: `if is_callsign_shape(word) && !is_own_call(word) {` → `if false {` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-ssid-any` | no SSID but the station's own | sqso-protocol/src/filter.rs: `if value == station::SSID { Verdict::Ok } else { Verdict::Callsign }` → `{ let _ = value; Verdict::Ok }` | `sqso-protocol` `filter::tests::ssid_rules`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-numeric-off` | no numbers but 73 and 1U | sqso-protocol/src/filter.rs: `if word.iter().any(u8::is_ascii_digit) && !word_is(word, b"73")` → `if false && word.iter().any(u8::is_ascii_digit) && !word_is(word, b"73")` | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-73-rejected` | "73" is allowed | sqso-protocol/src/filter.rs: `&& !word_is(word, b"73")` → (deleted) | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-1u-rejected` | "1U" is allowed | sqso-protocol/src/filter.rs: `&& !word_is(word, b"1u")` → (deleted) | `sqso-protocol` `filter::tests::shared_vectors`<br>C `test_beacon_filter/test_shared_vectors` |
| `filter-obc-unfiltered` | the OBC filters Jetson text before it is used | sqso-fsw/src/beacon.rs: `filter::check(text) == filter::Verdict::Ok` → `let _ = text; / true` | `sqso-fsw` `beacon::tests::given_text_must_pass_the_filter`<br>C `test_beacon_filter/test_rejected_ai_text_falls_back` |
| `filter-rejection-not-counted` | rejected AI text is counted (ai_rejected_count) | sqso-fsw/src/beacon.rs: `if given.is_some() && !ok {` → `if false {` | `sqso-fsw` `beacon::tests::given_text_must_pass_the_filter`<br>C `test_beacon_filter/test_rejected_ai_text_falls_back` |

### SW-14

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `mode-emergency-shadowed` | no shadowed rows: SAFE + POWER_CRITICAL reaches EMERGENCY | sqso-fsw/src/mode.rs: `E::PowerCritical => to(S::Emergency),` → `E::PowerCritical => to(S::Safe),` | `sqso-fsw` `mode::tests::every_state_is_reachable_from_boot`<br>C `test_state_machine/test_sm_every_state_reachable`<br>C `test_state_machine/test_sm_critical_power` |
| `mode-active-unreachable` | every state reachable: IDLE reaches ACTIVE | sqso-fsw/src/mode.rs: `E::AiReady \| E::GroundCmdActive => guarded(S::Active, Guard::ActiveAllowed),` → `E::AiReady \| E::GroundCmdActive => None,` | `sqso-fsw` `mode::tests::every_state_is_reachable_from_boot`<br>C `test_state_machine/test_sm_every_state_reachable` |
| `mode-guard-dropped` | the guard: every way into ACTIVE is guarded | sqso-fsw/src/mode.rs: `guarded(S::Active, Guard::ActiveAllowed)` → `to(S::Active)` | `sqso-fsw` `mode::tests::every_way_into_active_is_guarded`<br>C `test_state_machine/test_sm_active_guard` |
| `mode-guard-ignored` | the guard: a refused guard blocks the transition | sqso-fsw/src/mode.rs: `Some(Transition { guard: Some(g), .. }) if !guard(g) => Step::GuardFailed,` → `Some(Transition { guard: Some(g), .. }) if !guard(g) && false => Step::GuardFailed,` | `sqso-fsw` `mode::tests::guard_and_steps`<br>C `test_state_machine/test_sm_active_guard` |
| `mode-guard-no-battery` | the guard: battery at or above 0.55 | sqso-ffi/src/state_machine.rs: `c::eps_get_soc() >= mode::SOC_PAYLOAD &&` → (deleted) | C `test_state_machine/test_sm_active_guard` |
| `mode-guard-no-payload-guard` | the guard: the payload guard allows the payload | sqso-ffi/src/state_machine.rs: `&& c::payload_guard_may_enable(c::now_ms())` → (deleted) | C `test_state_machine/test_sm_active_guard` |
| `mode-entry-not-run` | entry actions run on every transition | sqso-ffi/src/state_machine.rs: `store(ctx, &m); / entered(to);` → `store(ctx, &m); / let _ = to;` | C `test_state_machine/test_sm_entry_sets_power_mode` |
| `mode-entry-active` | entry action: ACTIVE sets the EPS to ACTIVE | sqso-fsw/src/mode.rs: `State::Active => Entry { power_mode: PowerMode::Active, beacon: None },` → `State::Active => Entry { power_mode: PowerMode::Idle, beacon: None },` | `sqso-fsw` `mode::tests::entry_actions` (new)<br>C `test_state_machine/test_sm_entry_sets_power_mode` |
| `mode-entry-safe` | entry action: SAFE sets the EPS to SAFE with the beacon on | sqso-fsw/src/mode.rs: `State::Boot \| State::Safe => Entry { power_mode: PowerMode::Safe, beacon: Some(true) },` → `State::Boot \| State::Safe => Entry { power_mode: PowerMode::Idle, beacon: Some(true) },` | `sqso-fsw` `mode::tests::entry_actions` (new)<br>C `test_state_machine/test_sm_entry_sets_power_mode` |
| `mode-entry-emergency-beacon` | entry action: EMERGENCY sheds the beacon | sqso-fsw/src/mode.rs: `State::Emergency => Entry { power_mode: PowerMode::Safe, beacon: Some(false) },` → `State::Emergency => Entry { power_mode: PowerMode::Safe, beacon: Some(true) },` | `sqso-fsw` `mode::tests::entry_actions` (new)<br>C `test_state_machine/test_sm_critical_power` |
| `policy-emergency-threshold` | policy table: EMERGENCY below 0.15 in SAFE | sqso-fsw/src/mode.rs: `State::Safe if soc < SOC_EMERGENCY => Event::PowerCritical,` → `State::Safe if soc < SOC_SAFE => Event::PowerCritical,` | `sqso-fsw` `mode::tests::power_policy_thresholds` (new)<br>C `test_state_machine/test_sm_power_event_policy` |
| `policy-safe-exit` | policy table: SAFE returns to IDLE only above 0.40 | sqso-fsw/src/mode.rs: `State::Safe if soc > SOC_IDLE => Event::PowerOk,` → `State::Safe if soc >= SOC_IDLE => Event::PowerOk,` | `sqso-fsw` `mode::tests::power_policy_hysteresis`<br>C `test_state_machine/test_sm_power_event_policy` |
| `policy-emergency-exit` | policy table: EMERGENCY left at 0.25 | sqso-fsw/src/mode.rs: `State::Emergency if soc >= SOC_SAFE => Event::PowerOk,` → `State::Emergency if soc > SOC_SAFE => Event::PowerOk,` | `sqso-fsw` `mode::tests::power_policy_hysteresis`<br>`sqso-fsw` `mode::tests::power_policy_thresholds` (new)<br>C `test_state_machine/test_sm_power_event_policy` |
| `policy-payload-threshold` | policy table: ACTIVE from 0.55, sunlit | sqso-fsw/src/mode.rs: `State::Idle if sunlit && soc >= SOC_PAYLOAD => Event::AiReady,` → `State::Idle if soc >= SOC_PAYLOAD => Event::AiReady,` | `sqso-fsw` `mode::tests::power_policy_hysteresis`<br>C `test_state_machine/test_sm_power_event_policy` |
| `policy-active-low` | policy table: ACTIVE goes IDLE below 0.40 | sqso-fsw/src/mode.rs: `State::Active if soc < SOC_IDLE => Event::PowerLow,` → `State::Active if soc < SOC_SAFE => Event::PowerLow,` | `sqso-fsw` `mode::tests::power_policy_hysteresis`<br>`sqso-fsw` `mode::tests::power_policy_thresholds` (new)<br>C `test_state_machine/test_sm_power_event_policy` |
| `policy-idle-safe` | policy table: IDLE goes SAFE below 0.25 | sqso-fsw/src/mode.rs: `State::Idle if soc < SOC_SAFE => Event::PowerCritical,` → `State::Idle if soc < SOC_EMERGENCY => Event::PowerCritical,` | `sqso-fsw` `mode::tests::power_policy_hysteresis`<br>`sqso-fsw` `mode::tests::power_policy_thresholds` (new)<br>C `test_state_machine/test_sm_power_event_policy` |

### SW-15

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `wdt-gate-off` | gate: no refresh while a task is stalled | sqso-ffi/src/watchdog_mgr.rs: `if WDT.with(\|m\| m.wdt.any_stalled()) == Some(false) {` → `if WDT.with(\|m\| m.wdt.any_stalled()).is_some() {` | C `test_watchdog_mgr/test_stalled_task_lets_hardware_reset` |
| `wdt-stall-not-seen` | gate: a stalled task is seen | sqso-fsw/src/watchdog.rs: `self.tasks.iter().any(\|t\| t.active && t.stalled)` → `self.tasks.iter().any(\|t\| t.active && t.stalled && false)` | `sqso-fsw` `watchdog::tests::stall_after_three_misses_reported_once`<br>C `test_watchdog_mgr/test_stalled_task_lets_hardware_reset` |
| `wdt-save-dropped` | save: state is saved (pre-reset callback) when a task stalls | sqso-ffi/src/watchdog_mgr.rs: `if let Some(cb) = cb { / cb(); / }` → `let _ = cb;` | C `test_watchdog_mgr/test_stalled_task_lets_hardware_reset` |
| `wdt-save-repeated` | save: once per stall | sqso-fsw/src/watchdog.rs: `if task.miss_count < MISS_THRESHOLD \|\| task.stalled {` → `if task.miss_count < MISS_THRESHOLD {` | `sqso-fsw` `watchdog::tests::stall_after_three_misses_reported_once`<br>C `test_watchdog_mgr/test_stalled_task_lets_hardware_reset` |
| `wdt-recovery` | recovery: a check-in clears the stall | sqso-fsw/src/watchdog.rs: `entry.miss_count = 0; / entry.stalled = false; / entry.total_checkins` → `entry.miss_count = 0; / entry.total_checkins` | `sqso-fsw` `watchdog::tests::stall_after_three_misses_reported_once`<br>C `test_fault_scenarios/test_task_stall_detection` |
| `wdt-healthy-refresh` | a healthy task keeps the refresh going | sqso-fsw/src/watchdog.rs: `if now_ms.wrapping_sub(task.last_checkin_ms) <= u64::from(task.expected_period_ms) {` → `if now_ms.wrapping_sub(task.last_checkin_ms) <= u64::from(task.expected_period_ms) / 4 {` | `sqso-fsw` `watchdog::tests::stall_after_three_misses_reported_once` |

### SW-16

| Mutant | Behaviour | Mutation (file: old → new) | Killing tests |
|--------|-----------|----------------------------|---------------|
| `guard-overpower-once` | above 8 W twice in a row (not once) | sqso-fsw/src/payload_guard.rs: `if self.over_count >= OVERPOWER_SAMPLES {` → `if self.over_count >= 1 {` | `sqso-fsw` `payload_guard::tests::overpower_needs_two_in_a_row_latchup_one`<br>C `test_payload_guard/test_overpower` |
| `guard-overpower-never` | above 8 W twice in a row trips | sqso-fsw/src/payload_guard.rs: `if self.over_count >= OVERPOWER_SAMPLES {` → `if self.over_count > OVERPOWER_SAMPLES {` | `sqso-fsw` `payload_guard::tests::overpower_needs_two_in_a_row_latchup_one`<br>C `test_payload_guard/test_overpower` |
| `guard-overpower-not-in-a-row` | above 8 W twice *in a row* | sqso-fsw/src/payload_guard.rs: `self.over_count.saturating_add(1) } else { 0 };` → `self.over_count.saturating_add(1) } else { self.over_count };` | `sqso-fsw` `payload_guard::tests::overpower_needs_two_in_a_row_latchup_one`<br>C `test_payload_guard/test_overpower` |
| `guard-overpower-at-limit` | 8 W exactly is not over the limit | sqso-fsw/src/payload_guard.rs: `if s.input_w > MAX_INPUT_W {` → `if s.input_w >= MAX_INPUT_W {` | `sqso-fsw` `payload_guard::tests::overpower_needs_two_in_a_row_latchup_one` |
| `guard-latchup-off` | any sample at 12 W trips (latch-up) | sqso-fsw/src/payload_guard.rs: `if s.input_w >= LATCHUP_INPUT_W {` → `if s.input_w > LATCHUP_INPUT_W {` | `sqso-fsw` `payload_guard::tests::overpower_needs_two_in_a_row_latchup_one`<br>C `test_payload_guard/test_latchup` |
| `guard-spreader-limit` | heat spreader at 70 C trips | sqso-fsw/src/payload_guard.rs: `if s.spreader_valid && s.spreader_c >= SPREADER_MAX_C {` → `if s.spreader_valid && s.spreader_c > SPREADER_MAX_C {` | `sqso-fsw` `payload_guard::tests::temperatures_and_unreported_junction`<br>C `test_payload_guard/test_temperatures` |
| `guard-junction-limit` | Jetson junction at 95 C trips | sqso-fsw/src/payload_guard.rs: `if s.junction_valid && s.junction_c >= JUNCTION_MAX_C {` → `if s.junction_valid && s.junction_c > JUNCTION_MAX_C {` | `sqso-fsw` `payload_guard::tests::temperatures_and_unreported_junction`<br>C `test_payload_guard/test_temperatures` |
| `guard-junction-unreported` | an unreported junction is not a reading | sqso-fsw/src/payload_guard.rs: `if s.junction_valid && s.junction_c >= JUNCTION_MAX_C {` → `if s.junction_c >= JUNCTION_MAX_C {` | `sqso-fsw` `payload_guard::tests::temperatures_and_unreported_junction`<br>C `test_payload_guard/test_temperatures` |
| `guard-timeout-boundary` | no reading for 5 s while on trips (not before) | sqso-fsw/src/payload_guard.rs: `now_ms >= then_ms && now_ms - then_ms <= TELEMETRY_TIMEOUT_MS` → `now_ms >= then_ms && now_ms - then_ms < TELEMETRY_TIMEOUT_MS` | `sqso-fsw` `payload_guard::tests::unsupervised_rail_trips_after_the_grace`<br>C `test_payload_guard/test_no_telemetry` |
| `guard-spreader-not-supervised` | a missing spreader reading trips (power alone is not enough) | sqso-fsw/src/payload_guard.rs: `\|\| !age_ok(self.spreader_ms, now_ms)` → (deleted) | `sqso-fsw` `payload_guard::tests::unsupervised_rail_trips_after_the_grace`<br>C `test_payload_guard/test_no_telemetry` |
| `guard-rail-on-no-grace` | the supervision clocks restart when the rail turns on | sqso-fsw/src/payload_guard.rs: `if payload_on && !self.was_on { / self.power_ms = now_ms; / self.spreader_ms = now_ms;` → `if payload_on && !self.was_on {` | `sqso-fsw` `payload_guard::tests::unsupervised_rail_trips_after_the_grace`<br>C `test_payload_guard/test_no_telemetry` |
| `guard-no-cooldown` | after a trip the rail stays off for 10 min | sqso-fsw/src/payload_guard.rs: `self.tripped && now_ms < self.trip_ms.saturating_add(COOLDOWN_MS)` → `self.tripped && now_ms < self.trip_ms` | `sqso-fsw` `payload_guard::tests::cooldown_then_only_when_recently_cool`<br>C `test_payload_guard/test_cooldown` |
| `guard-restart-margin` | then until the spreader is below 60 C | sqso-fsw/src/payload_guard.rs: `let spreader_cool = self.spreader_c < SPREADER_MAX_C - RESTART_MARGIN_C;` → `let spreader_cool = self.spreader_c < SPREADER_MAX_C;` | `sqso-fsw` `payload_guard::tests::cooldown_then_only_when_recently_cool`<br>`sqso-fsw` `payload_guard::tests::before_a_trip_only_a_known_hot_spreader_blocks`<br>C `test_payload_guard/test_cooldown` |
| `guard-restart-stale-reading` | the cool reading must be recent | sqso-fsw/src/payload_guard.rs: `spreader_recent && spreader_cool // after a trip` → `spreader_cool // after a trip` | `sqso-fsw` `payload_guard::tests::cooldown_then_only_when_recently_cool`<br>C `test_payload_guard/test_cooldown` |
| `guard-eps-ignores` | eps_control_payload() refuses to enable while the guard forbids it | sqso-ffi/src/eps_control.rs: `let guard_allows = !enable \|\| unsafe { payload_guard_may_enable(c::now_ms()) };` → `let guard_allows = !enable \|\| unsafe { payload_guard_may_enable(c::now_ms()) } \|\| true;` | C `test_payload_guard/test_eps_honours_guard` |
| `guard-bus-nan` | the bus-voltage range check rejects NaN | sqso-fsw/src/eps.rs: `(BUS_MIN_V..=BUS_MAX_V).contains(&volts)` → `!(volts < BUS_MIN_V \|\| volts > BUS_MAX_V)` | `sqso-fsw` `eps::tests::soc_and_bus_limits`<br>C `EPS_Control_Tests/test_eps_bus_voltage_ok` |
| `guard-bus-range` | the bus-voltage range check runs on the measured value | sqso-fsw/src/eps.rs: `(BUS_MIN_V..=BUS_MAX_V).contains(&volts)` → `!volts.is_nan()` | `sqso-fsw` `eps::tests::soc_and_bus_limits`<br>C `EPS_Control_Tests/test_eps_bus_voltage_ok` |
