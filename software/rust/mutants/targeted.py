#!/usr/bin/env python3
"""Targeted mutation checks: the documented mutation checks, repeated on Rust.

docs/funding_audit/FINDINGS_REGISTER.md (SW-05, SW-08, SW-10, SW-12, SW-14,
SW-15, SW-16) and docs/interfaces/TELECOMMAND_AUTHENTICATION.md section 10
record behaviours that were mutation-checked on the former C flight code.
Each entry below is the equivalent hand-written mutation of the Rust code
(or, for the SMART_QSO_REQUIRE macro, of the C header that still holds it),
with the tests that must fail when it is applied. Results and the mapping
to the claims: software/rust/mutants/RESULTS-targeted.md.

Usage, from the repository root:
    python3 software/rust/mutants/targeted.py            # every mutation
    python3 software/rust/mutants/targeted.py ID ...     # only these
    python3 software/rust/mutants/targeted.py --list     # ids and claims
    python3 software/rust/mutants/targeted.py --check    # edits apply, no run
    python3 software/rust/mutants/targeted.py --survey [ID ...]
        run the whole Rust workspace and every C suite against each mutant
        and print every failing test (slow; used to write RESULTS)

For each mutation the script checks that each named killing test passes on
the unmutated tree, applies the edit (each old text must occur exactly once
in its file), builds, asserts that each named test now fails, and restores
the file. A mutant that does not build counts as a failure of this script,
not as killed. Kills are written as
    cargo:CRATE:TEST_PATH      cargo test -p CRATE --lib -- --exact TEST_PATH
    ctest:SUITE/CASE           the CMocka case CASE in ctest suite SUITE
The C suites use the CMake build directory BUILD (default: build), which is
configured if absent: cmake -S software/flight -B build.

Exit status 0 only if every selected mutant is killed by every named test.
"""

import os
import re
import signal
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RUST = ROOT / "software" / "rust"
BUILD = Path(os.environ.get("BUILD", ROOT / "build"))

FSW = "software/rust/sqso-fsw/src/"
FFI = "software/rust/sqso-ffi/src/"
PROTO = "software/rust/sqso-protocol/src/"


def m(mid, claim, behaviour, file, edits, kills):
    return {"id": mid, "claim": claim, "behaviour": behaviour, "file": file, "edits": edits, "kills": kills}


MUTATIONS = [
    # ---- SW-05 / TC-AUTH s10: post-deployment RF silence -------------------------------------------------
    m("rfs-por-resumes", "SW-05", "reset-cause rule: a power-on/brownout/pin/unknown reset restarts the count",
      FSW + "rf_silence.rs",
      [("            _ => Reset::PowerLost,\n", "            _ => Reset::PowerKept,\n")],
      ["cargo:sqso-fsw:rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept",
       "ctest:test_rf_silence/test_power_loss_restarts"]),
    m("rfs-any-reset-resumes", "SW-05", "reset-cause rule: only a reset that kept power resumes",
      FSW + "rf_silence.rs",
      [("state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S",
        "state == State::Counting as u32 && elapsed_s < DURATION_S")],
      ["cargo:sqso-fsw:rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept",
       "ctest:test_rf_silence/test_power_loss_restarts"]),
    m("rfs-watchdog-restarts", "SW-05", "reset-cause rule: a watchdog or software reset resumes the count",
      FSW + "rf_silence.rs",
      [("state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S",
        "state == State::Counting as u32 && reset == Reset::PowerLost && elapsed_s < DURATION_S")],
      ["cargo:sqso-fsw:rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept",
       "ctest:test_rf_silence/test_watchdog_reset_resumes"]),
    m("rfs-resume-past-duration", "SW-05", "a resumed count never shortens the silence (count >= 45 min not trusted)",
      FSW + "rf_silence.rs",
      [("state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S",
        "state == State::Counting as u32 && reset == Reset::PowerKept")],
      ["cargo:sqso-fsw:rf_silence::tests::reset_cause_rule_resume_only_when_power_was_kept"]),
    m("rfs-completion-not-saved", "SW-05", "completion persistence: COMPLETE is written when reached",
      FSW + "rf_silence.rs",
      [("            self.state = State::Complete;\n            self.save(io);\n",
        "            self.state = State::Complete;\n")],
      ["cargo:sqso-fsw:rf_silence::tests::completion_persists_across_power_on_resets",
       "ctest:test_rf_silence/test_completion_persists"]),
    m("rfs-completion-lost-on-por", "SW-05", "completion persistence: a power-on reset keeps COMPLETE",
      FSW + "rf_silence.rs",
      [("if state == State::Complete as u32 {", "if state == State::Complete as u32 && reset == Reset::PowerKept {")],
      ["cargo:sqso-fsw:rf_silence::tests::completion_persists_across_power_on_resets",
       "ctest:test_rf_silence/test_completion_persists"]),
    m("rfs-switch-ignored", "SW-05", "switch reset: a stowed switch seen while powered restarts the count",
      FSW + "rf_silence.rs",
      [("        if io.stowed() {\n", "        if false {\n")],
      ["cargo:sqso-fsw:rf_silence::tests::stowed_switch_restarts_even_after_completion",
       "ctest:test_rf_silence/test_switch_toggle_restarts"]),
    m("rfs-switch-ignored-after-completion", "SW-05", "switch reset: also after completion",
      FSW + "rf_silence.rs",
      [("if self.state != State::Counting || self.elapsed_s != 0 {",
        "if self.state == State::Counting && self.elapsed_s != 0 {")],
      ["cargo:sqso-fsw:rf_silence::tests::stowed_switch_restarts_even_after_completion",
       "ctest:test_rf_silence/test_stowed_after_completion_rearms"]),
    m("rfs-switch-restart-not-saved", "SW-05", "switch reset: the restart is persisted",
      FSW + "rf_silence.rs",
      [("                self.restart_count();\n                self.save(io);\n",
        "                self.restart_count();\n")],
      ["cargo:sqso-fsw:rf_silence::tests::stowed_switch_restarts_even_after_completion"]),
    m("rfs-armed-counts", "SW-05", "GSE ARM: ARMED does not count while powered",
      FSW + "rf_silence.rs",
      [("        if self.state != State::Counting {\n            return; // COMPLETE",
        "        if self.state == State::Complete {\n            return; // COMPLETE")],
      ["cargo:sqso-fsw:rf_silence::tests::arm_holds_until_the_next_boot",
       "ctest:test_rf_silence/test_arm_resets_completed_unit"]),
    m("rfs-gse-arm-skipped", "SW-05", "GSE ARM_FLIGHT re-arms the RF silence",
      FFI + "gse_provision.rs",
      [("let silence = unsafe { c::rf_silence_arm() };", "let silence = rc::OK;")],
      ["ctest:test_gse_provision/test_arm_flight"]),
    m("rfs-radio-powered", "SW-05", "the whole radio stays unpowered during the silence",
      FSW + "rf_control.rs",
      [("let radio_on = self.auto_inhibits & INHIBIT_RF_SILENCE == 0;", "let radio_on = true;")],
      ["cargo:sqso-fsw:rf_control::tests::automatic_inhibits_are_independent",
       "ctest:test_rf_silence/test_silent_for_45_minutes"]),
    m("rfs-tx-enable-lifts", "SW-05", "no telecommand shortens the silence (TX_ENABLE clears the latch only)",
      FSW + "rf_control.rs",
      [("        self.inhibited = false;\n        self.drive(io);\n",
        "        self.inhibited = false;\n        self.auto_inhibits = 0;\n        self.drive(io);\n")],
      ["cargo:sqso-fsw:rf_control::tests::automatic_inhibits_are_independent",
       "ctest:test_rf_silence/test_tx_enable_cannot_shorten_silence"]),

    # ---- TC-AUTH s10: contact hook gate and placement ----------------------------------------------------
    m("contact-gate", "TC-AUTH contact hook", "gate: the keyless TX_DISABLE is not contact",
      FSW + "telecommand.rs",
      [("let contact = verified.key != KeyUsed::None;", "let contact = true;")],
      ["cargo:sqso-fsw:telecommand::tests::contact_hook_gate_and_placement",
       "ctest:test_cmd_handler/test_unauthenticated_frames_are_not_contact"]),
    m("contact-before-arglen", "TC-AUTH contact hook", "placement: a rejection before the counter is spent is not contact",
      FSW + "telecommand.rs",
      [("return reject(cmd_id, counter, Rejection::ArgLength, false);",
        "return reject(cmd_id, counter, Rejection::ArgLength, true);")],
      ["cargo:sqso-fsw:telecommand::tests::contact_hook_gate_and_placement",
       "ctest:test_cmd_handler/test_unauthenticated_frames_are_not_contact"]),
    m("contact-before-unknown", "TC-AUTH contact hook", "placement: an unknown command is not contact",
      FSW + "telecommand.rs",
      [("None => return reject(cmd_id, counter, Rejection::UnknownCommand, false),",
        "None => return reject(cmd_id, counter, Rejection::UnknownCommand, true),")],
      ["cargo:sqso-fsw:telecommand::tests::contact_hook_gate_and_placement"]),
    m("contact-before-auth", "TC-AUTH contact hook", "placement: a forged or replayed frame is not contact",
      FSW + "telecommand.rs",
      [("Err(e) => return reject(cmd_id, None, Rejection::AuthFailed(e), false),",
        "Err(e) => return reject(cmd_id, None, Rejection::AuthFailed(e), true),")],
      ["cargo:sqso-fsw:telecommand::tests::contact_hook_gate_and_placement",
       "ctest:test_cmd_handler/test_unauthenticated_frames_are_not_contact"]),
    m("contact-hook-removed", "TC-AUTH contact hook", "hook: an accepted keyed command is contact",
      FFI + "cmd_handler.rs",
      [("        Admission::Accepted { command, contact: was_contact } => {\n            if was_contact {",
        "        Admission::Accepted { command, contact: was_contact } => {\n            if was_contact && false {")],
      ["ctest:test_cmd_handler/test_authentic_command_restores_contact"]),
    m("contact-hook-removed-unpersisted", "TC-AUTH contact hook",
      "hook: a spent counter that could not be persisted is still contact",
      FFI + "cmd_handler.rs",
      [("            if was_contact {\n                contact();\n            }\n            let (code, reason)",
        "            if was_contact && false {\n                contact();\n            }\n            let (code, reason)")],
      ["ctest:test_cmd_handler/test_unpersisted_counter_is_still_contact"]),

    # ---- TC-AUTH s10: GSE umbilical gate -----------------------------------------------------------------
    m("gse-gate-frame", "TC-AUTH GSE gate", "no provisioning frame is executed without the umbilical",
      FFI + "gse_provision.rs",
      [("-> u32 {\n    if !gse_umbilical_present() {", "-> u32 {\n    if false {")],
      ["ctest:test_gse_provision/test_locked_without_umbilical"]),
    m("gse-gate-poll", "TC-AUTH GSE gate", "bytes on the umbilical port without the umbilical are discarded",
      FFI + "gse_provision.rs",
      [("    unsafe {\n        if !gse_umbilical_present() {", "    unsafe {\n        if false {")],
      ["ctest:test_gse_provision/test_poll_reads_umbilical_uart"]),
    m("gse-sense-ignored", "TC-AUTH GSE gate", "the umbilical sense line decides",
      FFI + "gse_provision.rs",
      [("unsafe { c::hal_gpio_get(GPIO_PIN_UMBILICAL_SENSE) }", "true")],
      ["ctest:test_gse_provision/test_locked_without_umbilical"]),

    # ---- TC-AUTH s10: AX.25 callsign filter --------------------------------------------------------------
    m("ax25-dest-ignored", "TC-AUTH callsign filter", "only frames addressed to this station reach the handler",
      FFI + "uplink.rs",
      [("if !ax25::address_matches(ui.dest, callsign.get(..len).unwrap_or(&[]), ssid) {", "if false {")],
      ["ctest:test_uplink/test_other_frames_dropped"]),
    m("ax25-src-checked", "TC-AUTH callsign filter", "the destination, not the source, is matched",
      FFI + "uplink.rs",
      [("ax25::address_matches(ui.dest,", "ax25::address_matches(ui.src,")],
      ["ctest:test_uplink/test_received_tx_disable_silences_station"]),
    m("ax25-ssid-ignored", "TC-AUTH callsign filter", "the SSID must match",
      PROTO + "ax25.rs",
      [(" && (addr[CALL_LEN] & SSID_MASK) == (expected[CALL_LEN] & SSID_MASK)", "")],
      ["cargo:sqso-protocol:ax25::tests::address_encoding",
       "ctest:test_uplink/test_other_frames_dropped"]),
    m("ax25-call-ignored", "TC-AUTH callsign filter", "the call sign must match",
      PROTO + "ax25.rs",
      [("addr[..CALL_LEN] == expected[..CALL_LEN] && ", "")],
      ["cargo:sqso-protocol:ax25::tests::other_call_with_our_ssid_does_not_match",
       "ctest:test_ax25/test_parse_reference_frame"]),

    # ---- SW-08: assertions active in flight builds -------------------------------------------------------
    m("assert-not-reported", "SW-08", "reporting: a failed SMART_QSO_REQUIRE reaches the assertion handler",
      FFI + "assert_handler.rs",
      [("let _ = unsafe { assert_handle_failure(file, short_line, kind::PRECONDITION, severity::ERROR, message) };",
        "let _ = (short_line, kind::PRECONDITION);")],
      ["ctest:test_assert_release/test_failure_reported_and_returns_error"]),
    m("assert-not-counted", "SW-08", "reporting: the failure is counted",
      FSW + "assert_handler.rs",
      [("        s.total_failures = s.total_failures.wrapping_add(1);\n", "")],
      ["cargo:sqso-fsw:assert_handler::tests::severities_and_counts",
       "ctest:test_assert_release/test_failure_reported_and_returns_error"]),
    m("assert-not-logged", "SW-08", "reporting: the failure is logged",
      FSW + "assert_handler.rs",
      [("        self.log_failure(file_name(file), line, kind, severity, message, uptime_s);\n", "")],
      ["cargo:sqso-fsw:assert_handler::tests::severities_and_counts",
       "ctest:test_assert_release/test_failure_reported_and_returns_error"]),
    m("assert-macro-no-return", "SW-08", "early return: a failed SMART_QSO_REQUIRE returns an error (C macro)",
      "software/flight/include/smart_qso.h",
      [("            return (retval); \\\n", "            (void)(retval); \\\n")],
      ["ctest:test_assert_release/test_failure_reported_and_returns_error"]),
    m("assert-rust-check-not-reported", "SW-08", "reporting: a failed check in Rust reaches the assertion handler",
      FFI + "c.rs",
      [("    unsafe { smart_qso_assert_failed(c\"sqso-ffi\".as_ptr(), line, message.as_ptr()) };",
        "    let _ = (message, line);")],
      ["ctest:test_assert_release/test_flight_code_refuses_bad_input"]),
    m("assert-eps-stores-invalid", "SW-08", "early return: eps_set_power_mode does not store an invalid mode",
      FFI + "eps_control.rs",
      [("    let Some(power_mode) = eps::power_mode(mode) else {\n"
        "        c::assert_failed(c\"Invalid power mode\", line!());\n"
        "        return rc::INVALID;\n    };",
        "    let power_mode = match eps::power_mode(mode) {\n        Some(p) => p,\n        None => {\n"
        "            c::assert_failed(c\"Invalid power mode\", line!());\n            PowerMode::Active\n"
        "        }\n    };")],
      ["ctest:test_assert_release/test_flight_code_refuses_bad_input"]),

    # ---- SW-10: deployment sequence ----------------------------------------------------------------------
    m("deploy-sensed-fired", "SW-10", "idempotent: an element sensed as deployed is never fired",
      FSW + "deployment.rs",
      [("if self.element(e).deployed || io.sensed(e) {", "if self.element(e).deployed {")],
      ["cargo:sqso-fsw:deployment::tests::already_deployed_is_never_fired",
       "ctest:test_deployment/test_already_deployed_not_fired"]),
    m("deploy-arm-left-on", "SW-10", "a burn needs the master arm: the arm drops when a burn ends",
      FSW + "deployment.rs",
      [("    fn burn_off<IO: DeployIo>(io: &mut IO) {\n        io.set(Line::Arm, false);\n",
        "    fn burn_off<IO: DeployIo>(io: &mut IO) {\n")],
      ["cargo:sqso-fsw:deployment::tests::full_sequence_one_element_at_a_time",
       "ctest:test_deployment/test_full_sequence"]),
    m("deploy-burn-without-persist", "SW-10", "nothing fires if the attempt cannot be persisted",
      FSW + "deployment.rs",
      [("        if !self.save(io) {\n", "        if !self.save(io) && false {\n")],
      ["cargo:sqso-fsw:deployment::tests::attempt_persisted_before_the_burn",
       "ctest:test_deployment/test_persist_failure_blocks_burn"]),
    m("deploy-burn-before-persist", "SW-10", "the attempt is persisted before the burn line is driven",
      FSW + "deployment.rs",
      [("        self.enter_state(io, active);\n        if !self.save(io) {\n",
        "        self.enter_state(io, active);\n        Self::burn_on(io, e);\n        if !self.save(io) {\n")],
      ["cargo:sqso-fsw:deployment::tests::attempt_persisted_before_the_burn",
       "ctest:test_deployment/test_persist_failure_blocks_burn"]),
    m("deploy-reset-refires", "SW-10", "a reset mid-burn spends the attempt (verify, never re-fire)",
      FSW + "deployment.rs",
      [("State::AntennaActive => State::AntennaVerify,", "State::AntennaActive => State::AntennaPending,")],
      ["cargo:sqso-fsw:deployment::tests::reset_during_burn_spends_the_attempt",
       "ctest:test_deployment/test_reset_during_burn_spends_attempt"]),
    m("deploy-attempts-not-restored", "SW-10", "attempts are bounded across brownout loops",
      FSW + "deployment.rs",
      [("            el.attempt_count = attempts;\n", "            el.attempt_count = 0;\n")],
      ["cargo:sqso-fsw:deployment::tests::brownout_loop_is_bounded_by_persisted_attempts",
       "ctest:test_deployment/test_burn_brownout_loop_is_bounded"]),
    m("deploy-attempts-unbounded", "SW-10", "attempts are bounded (FAILED after MAX_RETRIES)",
      FSW + "deployment.rs",
      [("if self.element(e).attempt_count >= MAX_RETRIES {", "if false {")],
      ["cargo:sqso-fsw:deployment::tests::brownout_loop_is_bounded_by_persisted_attempts",
       "cargo:sqso-fsw:deployment::tests::disabled_holds_and_failure_allows_a_ground_retry",
       "ctest:test_deployment/test_burn_brownout_loop_is_bounded"]),
    m("deploy-por-resumes", "SW-10", "the 30-min wait restarts on a power-on reset",
      FSW + "deployment.rs",
      [("            (_, Reset::PowerLost) => {}\n            (_, Reset::PowerKept) => {\n",
        "            (_, Reset::PowerLost | Reset::PowerKept) => {\n")],
      ["cargo:sqso-fsw:deployment::tests::wait_resumes_only_when_power_was_kept_and_restarts_when_stowed",
       "ctest:test_deployment/test_power_on_reset_restarts_wait"]),
    m("deploy-watchdog-restarts", "SW-10", "the 30-min wait resumes after a watchdog reset",
      FSW + "deployment.rs",
      [("            (_, Reset::PowerLost) => {}\n            (_, Reset::PowerKept) => {\n",
        "            (_, Reset::PowerLost | Reset::PowerKept) => {}\n"
        "            #[allow(unreachable_patterns)]\n            (_, Reset::PowerKept) => {\n")],
      ["cargo:sqso-fsw:deployment::tests::wait_resumes_only_when_power_was_kept_and_restarts_when_stowed",
       "ctest:test_deployment/test_watchdog_reset_resumes_wait"]),
    m("deploy-switch-ignored", "SW-10", "a stowed switch restarts the wait (CDS 2.3.2.4)",
      FSW + "deployment.rs",
      [("        if io.stowed() {\n", "        if false {\n")],
      ["cargo:sqso-fsw:deployment::tests::wait_resumes_only_when_power_was_kept_and_restarts_when_stowed",
       "ctest:test_deployment/test_stowed_switch_restarts_wait"]),
    m("deploy-terminal-follows-switch", "SW-10", "COMPLETE and FAILED ignore the switch",
      FSW + "deployment.rs",
      [("if matches!(self.state, State::Complete | State::Failed | State::Armed) {",
        "if matches!(self.state, State::Armed) {")],
      ["cargo:sqso-fsw:deployment::tests::terminal_states_persist_and_ignore_the_switch_arm_resets",
       "ctest:test_deployment/test_complete_persists_and_ignores_switch"]),
    m("deploy-force-retry-bypass", "SW-10", "force_retry no longer bypasses the 30-min wait",
      FSW + "deployment.rs",
      [("State::Inhibited | State::Waiting | State::Armed | State::AntennaActive | State::SolarActive => {",
        "State::Armed | State::AntennaActive | State::SolarActive => {"),
       ("            State::AntennaPending\n            | State::AntennaVerify\n",
        "            State::Inhibited\n            | State::Waiting\n            | State::AntennaPending\n"
        "            | State::AntennaVerify\n")],
      ["cargo:sqso-fsw:deployment::tests::force_retry_cannot_bypass_the_wait_and_names",
       "ctest:test_deployment/test_force_retry_cannot_bypass_wait"]),
    m("deploy-wait-shortened", "SW-10", "no burn before the 30-min wait is served",
      FSW + "deployment.rs",
      [("if self.wait_elapsed_s >= WAIT_TIME_S && self.deployment_enabled {",
        "if self.wait_elapsed_s >= WAIT_TIME_S - 60 && self.deployment_enabled {")],
      ["cargo:sqso-fsw:deployment::tests::full_sequence_one_element_at_a_time",
       "ctest:test_deployment/test_full_sequence"]),
    m("deploy-gse-arm-skipped", "SW-10", "GSE ARM_FLIGHT re-arms the deployment sequence",
      FFI + "gse_provision.rs",
      [("let deploy = unsafe { c::deployment_arm() };", "let deploy = rc::OK;")],
      ["ctest:test_gse_provision/test_arm_flight"]),

    # ---- SW-12: beacon content filter --------------------------------------------------------------------
    m("filter-pipe", "SW-12", "character set: no '|'", PROTO + "filter.rs",
      [("pub const PUNCTUATION: &str = \" .,!?'\\\":;()/-&\";", "pub const PUNCTUATION: &str = \" .,!?'\\\":;()/-&|\";")],
      ["cargo:sqso-protocol:filter::tests::charset_excludes_the_field_separator",
       "ctest:test_beacon_filter/test_fuzz_properties"]),
    m("filter-dollar", "SW-12", "character set: no '$'", PROTO + "filter.rs",
      [("pub const PUNCTUATION: &str = \" .,!?'\\\":;()/-&\";", "pub const PUNCTUATION: &str = \" .,!?'\\\":;()/-&$\";")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "cargo:sqso-protocol:filter::tests::charset_excludes_the_field_separator",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-at", "SW-12", "character set: no '@'", PROTO + "filter.rs",
      [("pub const PUNCTUATION: &str = \" .,!?'\\\":;()/-&\";", "pub const PUNCTUATION: &str = \" .,!?'\\\":;()/-&@\";")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "cargo:sqso-protocol:filter::tests::charset_excludes_the_field_separator",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-charset-off", "SW-12", "character set enforced", PROTO + "filter.rs",
      [("if !text.iter().all(|c| allowed(*c)) {", "if false {")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-length", "SW-12", "length: 1 to 176 characters", PROTO + "filter.rs",
      [("if text.is_empty() || text.len() > MAX_LEN {", "if text.is_empty() || text.len() > MAX_LEN + 1 {")],
      ["cargo:sqso-protocol:filter::tests::length_limit_is_inclusive"]),
    m("filter-url-off", "SW-12", "no URLs", PROTO + "filter.rs",
      [("if URL_MARKS.iter().any(", "if false && URL_MARKS.iter().any(")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "cargo:sqso-protocol:filter::tests::url_marks_in_any_letter_case",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-url-case", "SW-12", "no URLs, in any letter case", PROTO + "filter.rs",
      [("text.windows(lit.len()).any(|w| w.eq_ignore_ascii_case(lit))", "text.windows(lit.len()).any(|w| w == lit)")],
      ["cargo:sqso-protocol:filter::tests::url_marks_in_any_letter_case"]),
    m("filter-commercial-off", "SW-12", "no commercial words", PROTO + "filter.rs",
      [("if in_list(word, COMMERCIAL) {", "if false {")],
      ["cargo:sqso-protocol:filter::tests::every_list_word_is_caught",
       "ctest:test_beacon_filter/test_word_lists"]),
    m("filter-profanity-off", "SW-12", "no profanity", PROTO + "filter.rs",
      [("if in_list(word, PROFANITY) {", "if false {")],
      ["cargo:sqso-protocol:filter::tests::every_list_word_is_caught",
       "ctest:test_beacon_filter/test_word_lists"]),
    m("filter-substring-match", "SW-12", "word lists match whole words only", PROTO + "filter.rs",
      [("    word.eq_ignore_ascii_case(lit)\n", "    contains_ignore_case(word, lit)\n")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-overlong-off", "SW-12", "encoded-looking: overlong words", PROTO + "filter.rs",
      [("    if word.len() > MAX_WORD {\n", "    if word.len() > MAX_WORD + 20 {\n")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-vowelless-off", "SW-12", "encoded-looking: vowel-less words", PROTO + "filter.rs",
      [("(all_alpha && word.len() >= 5 && !vowel) ||", "false ||")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-hex-off", "SW-12", "encoded-looking: hex-only words", PROTO + "filter.rs",
      [("(all_hex && word.len() >= 8) ||", "false ||")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-case-changes-off", "SW-12", "encoded-looking: scattered capitals", PROTO + "filter.rs",
      [("|| case_changes > 2", "|| case_changes > 20")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-callsign-off", "SW-12", "no call sign but the station's own", PROTO + "filter.rs",
      [("if is_callsign_shape(word) && !is_own_call(word) {", "if false {")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-ssid-any", "SW-12", "no SSID but the station's own", PROTO + "filter.rs",
      [("if value == station::SSID { Verdict::Ok } else { Verdict::Callsign }", "{ let _ = value; Verdict::Ok }")],
      ["cargo:sqso-protocol:filter::tests::ssid_rules",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-numeric-off", "SW-12", "no numbers but 73 and 1U", PROTO + "filter.rs",
      [("if word.iter().any(u8::is_ascii_digit) && !word_is(word, b\"73\")",
        "if false && word.iter().any(u8::is_ascii_digit) && !word_is(word, b\"73\")")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-73-rejected", "SW-12", "\"73\" is allowed", PROTO + "filter.rs",
      [(" && !word_is(word, b\"73\")", "")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-1u-rejected", "SW-12", "\"1U\" is allowed", PROTO + "filter.rs",
      [(" && !word_is(word, b\"1u\")", "")],
      ["cargo:sqso-protocol:filter::tests::shared_vectors",
       "ctest:test_beacon_filter/test_shared_vectors"]),
    m("filter-obc-unfiltered", "SW-12", "the OBC filters Jetson text before it is used",
      FSW + "beacon.rs",
      [("    filter::check(text) == filter::Verdict::Ok\n", "    let _ = text;\n    true\n")],
      ["cargo:sqso-fsw:beacon::tests::given_text_must_pass_the_filter",
       "ctest:test_beacon_filter/test_rejected_ai_text_falls_back"]),
    m("filter-rejection-not-counted", "SW-12", "rejected AI text is counted (ai_rejected_count)",
      FSW + "beacon.rs",
      [("        if given.is_some() && !ok {\n", "        if false {\n")],
      ["cargo:sqso-fsw:beacon::tests::given_text_must_pass_the_filter",
       "ctest:test_beacon_filter/test_rejected_ai_text_falls_back"]),

    # ---- SW-14: the guarded mode manager -----------------------------------------------------------------
    m("mode-emergency-shadowed", "SW-14", "no shadowed rows: SAFE + POWER_CRITICAL reaches EMERGENCY",
      FSW + "mode.rs",
      [("            E::PowerCritical => to(S::Emergency),\n", "            E::PowerCritical => to(S::Safe),\n")],
      ["cargo:sqso-fsw:mode::tests::every_state_is_reachable_from_boot",
       "ctest:test_state_machine/test_sm_every_state_reachable",
       "ctest:test_state_machine/test_sm_critical_power"]),
    m("mode-active-unreachable", "SW-14", "every state reachable: IDLE reaches ACTIVE",
      FSW + "mode.rs",
      [("E::AiReady | E::GroundCmdActive => guarded(S::Active, Guard::ActiveAllowed),",
        "E::AiReady | E::GroundCmdActive => None,")],
      ["cargo:sqso-fsw:mode::tests::every_state_is_reachable_from_boot",
       "ctest:test_state_machine/test_sm_every_state_reachable"]),
    m("mode-guard-dropped", "SW-14", "the guard: every way into ACTIVE is guarded",
      FSW + "mode.rs",
      [("guarded(S::Active, Guard::ActiveAllowed)", "to(S::Active)")],
      ["cargo:sqso-fsw:mode::tests::every_way_into_active_is_guarded",
       "ctest:test_state_machine/test_sm_active_guard"]),
    m("mode-guard-ignored", "SW-14", "the guard: a refused guard blocks the transition",
      FSW + "mode.rs",
      [("Some(Transition { guard: Some(g), .. }) if !guard(g) => Step::GuardFailed,",
        "Some(Transition { guard: Some(g), .. }) if !guard(g) && false => Step::GuardFailed,")],
      ["cargo:sqso-fsw:mode::tests::guard_and_steps",
       "ctest:test_state_machine/test_sm_active_guard"]),
    m("mode-guard-no-battery", "SW-14", "the guard: battery at or above 0.55",
      FFI + "state_machine.rs",
      [("c::eps_get_soc() >= mode::SOC_PAYLOAD && ", "")],
      ["ctest:test_state_machine/test_sm_active_guard"]),
    m("mode-guard-no-payload-guard", "SW-14", "the guard: the payload guard allows the payload",
      FFI + "state_machine.rs",
      [(" && c::payload_guard_may_enable(c::now_ms())", "")],
      ["ctest:test_state_machine/test_sm_active_guard"]),
    m("mode-entry-not-run", "SW-14", "entry actions run on every transition",
      FFI + "state_machine.rs",
      [("            store(ctx, &m);\n            entered(to);\n", "            store(ctx, &m);\n            let _ = to;\n")],
      ["ctest:test_state_machine/test_sm_entry_sets_power_mode"]),
    m("mode-entry-active", "SW-14", "entry action: ACTIVE sets the EPS to ACTIVE",
      FSW + "mode.rs",
      [("State::Active => Entry { power_mode: PowerMode::Active, beacon: None },",
        "State::Active => Entry { power_mode: PowerMode::Idle, beacon: None },")],
      ["cargo:sqso-fsw:mode::tests::entry_actions",
       "ctest:test_state_machine/test_sm_entry_sets_power_mode"]),
    m("mode-entry-safe", "SW-14", "entry action: SAFE sets the EPS to SAFE with the beacon on",
      FSW + "mode.rs",
      [("State::Boot | State::Safe => Entry { power_mode: PowerMode::Safe, beacon: Some(true) },",
        "State::Boot | State::Safe => Entry { power_mode: PowerMode::Idle, beacon: Some(true) },")],
      ["cargo:sqso-fsw:mode::tests::entry_actions",
       "ctest:test_state_machine/test_sm_entry_sets_power_mode"]),
    m("mode-entry-emergency-beacon", "SW-14", "entry action: EMERGENCY sheds the beacon",
      FSW + "mode.rs",
      [("State::Emergency => Entry { power_mode: PowerMode::Safe, beacon: Some(false) },",
        "State::Emergency => Entry { power_mode: PowerMode::Safe, beacon: Some(true) },")],
      ["cargo:sqso-fsw:mode::tests::entry_actions",
       "ctest:test_state_machine/test_sm_critical_power"]),
    m("policy-emergency-threshold", "SW-14", "policy table: EMERGENCY below 0.15 in SAFE",
      FSW + "mode.rs",
      [("State::Safe if soc < SOC_EMERGENCY => Event::PowerCritical,",
        "State::Safe if soc < SOC_SAFE => Event::PowerCritical,")],
      ["cargo:sqso-fsw:mode::tests::power_policy_thresholds",
       "ctest:test_state_machine/test_sm_power_event_policy"]),
    m("policy-safe-exit", "SW-14", "policy table: SAFE returns to IDLE only above 0.40",
      FSW + "mode.rs",
      [("State::Safe if soc > SOC_IDLE => Event::PowerOk,", "State::Safe if soc >= SOC_IDLE => Event::PowerOk,")],
      ["cargo:sqso-fsw:mode::tests::power_policy_hysteresis",
       "ctest:test_state_machine/test_sm_power_event_policy"]),
    m("policy-emergency-exit", "SW-14", "policy table: EMERGENCY left at 0.25",
      FSW + "mode.rs",
      [("State::Emergency if soc >= SOC_SAFE => Event::PowerOk,", "State::Emergency if soc > SOC_SAFE => Event::PowerOk,")],
      ["cargo:sqso-fsw:mode::tests::power_policy_hysteresis",
       "cargo:sqso-fsw:mode::tests::power_policy_thresholds",
       "ctest:test_state_machine/test_sm_power_event_policy"]),
    m("policy-payload-threshold", "SW-14", "policy table: ACTIVE from 0.55, sunlit",
      FSW + "mode.rs",
      [("State::Idle if sunlit && soc >= SOC_PAYLOAD => Event::AiReady,",
        "State::Idle if soc >= SOC_PAYLOAD => Event::AiReady,")],
      ["cargo:sqso-fsw:mode::tests::power_policy_hysteresis",
       "ctest:test_state_machine/test_sm_power_event_policy"]),
    m("policy-active-low", "SW-14", "policy table: ACTIVE goes IDLE below 0.40",
      FSW + "mode.rs",
      [("State::Active if soc < SOC_IDLE => Event::PowerLow,", "State::Active if soc < SOC_SAFE => Event::PowerLow,")],
      ["cargo:sqso-fsw:mode::tests::power_policy_hysteresis",
       "cargo:sqso-fsw:mode::tests::power_policy_thresholds",
       "ctest:test_state_machine/test_sm_power_event_policy"]),
    m("policy-idle-safe", "SW-14", "policy table: IDLE goes SAFE below 0.25",
      FSW + "mode.rs",
      [("State::Idle if soc < SOC_SAFE => Event::PowerCritical,", "State::Idle if soc < SOC_EMERGENCY => Event::PowerCritical,")],
      ["cargo:sqso-fsw:mode::tests::power_policy_hysteresis",
       "cargo:sqso-fsw:mode::tests::power_policy_thresholds",
       "ctest:test_state_machine/test_sm_power_event_policy"]),

    # ---- SW-15: supervised watchdog ----------------------------------------------------------------------
    m("wdt-gate-off", "SW-15", "gate: no refresh while a task is stalled",
      FFI + "watchdog_mgr.rs",
      [("if WDT.with(|m| m.wdt.any_stalled()) == Some(false) {", "if WDT.with(|m| m.wdt.any_stalled()).is_some() {")],
      ["ctest:test_watchdog_mgr/test_stalled_task_lets_hardware_reset"]),
    m("wdt-stall-not-seen", "SW-15", "gate: a stalled task is seen", FSW + "watchdog.rs",
      [("self.tasks.iter().any(|t| t.active && t.stalled)", "self.tasks.iter().any(|t| t.active && t.stalled && false)")],
      ["cargo:sqso-fsw:watchdog::tests::stall_after_three_misses_reported_once",
       "ctest:test_watchdog_mgr/test_stalled_task_lets_hardware_reset"]),
    m("wdt-save-dropped", "SW-15", "save: state is saved (pre-reset callback) when a task stalls",
      FFI + "watchdog_mgr.rs",
      [("            if let Some(cb) = cb {\n                cb();\n            }\n", "            let _ = cb;\n")],
      ["ctest:test_watchdog_mgr/test_stalled_task_lets_hardware_reset"]),
    m("wdt-save-repeated", "SW-15", "save: once per stall", FSW + "watchdog.rs",
      [("if task.miss_count < MISS_THRESHOLD || task.stalled {", "if task.miss_count < MISS_THRESHOLD {")],
      ["cargo:sqso-fsw:watchdog::tests::stall_after_three_misses_reported_once",
       "ctest:test_watchdog_mgr/test_stalled_task_lets_hardware_reset"]),
    m("wdt-recovery", "SW-15", "recovery: a check-in clears the stall", FSW + "watchdog.rs",
      [("        entry.miss_count = 0;\n        entry.stalled = false;\n        entry.total_checkins",
        "        entry.miss_count = 0;\n        entry.total_checkins")],
      ["cargo:sqso-fsw:watchdog::tests::stall_after_three_misses_reported_once",
       "ctest:test_fault_scenarios/test_task_stall_detection"]),
    m("wdt-healthy-refresh", "SW-15", "a healthy task keeps the refresh going", FSW + "watchdog.rs",
      [("if now_ms.wrapping_sub(task.last_checkin_ms) <= u64::from(task.expected_period_ms) {",
        "if now_ms.wrapping_sub(task.last_checkin_ms) <= u64::from(task.expected_period_ms) / 4 {")],
      ["cargo:sqso-fsw:watchdog::tests::stall_after_three_misses_reported_once"]),

    # ---- SW-16: payload guard (FDIR) ---------------------------------------------------------------------
    m("guard-overpower-once", "SW-16", "above 8 W twice in a row (not once)", FSW + "payload_guard.rs",
      [("if self.over_count >= OVERPOWER_SAMPLES {", "if self.over_count >= 1 {")],
      ["cargo:sqso-fsw:payload_guard::tests::overpower_needs_two_in_a_row_latchup_one",
       "ctest:test_payload_guard/test_overpower"]),
    m("guard-overpower-never", "SW-16", "above 8 W twice in a row trips", FSW + "payload_guard.rs",
      [("if self.over_count >= OVERPOWER_SAMPLES {", "if self.over_count > OVERPOWER_SAMPLES {")],
      ["cargo:sqso-fsw:payload_guard::tests::overpower_needs_two_in_a_row_latchup_one",
       "ctest:test_payload_guard/test_overpower"]),
    m("guard-overpower-not-in-a-row", "SW-16", "above 8 W twice *in a row*", FSW + "payload_guard.rs",
      [("self.over_count.saturating_add(1) } else { 0 };", "self.over_count.saturating_add(1) } else { self.over_count };")],
      ["cargo:sqso-fsw:payload_guard::tests::overpower_needs_two_in_a_row_latchup_one",
       "ctest:test_payload_guard/test_overpower"]),
    m("guard-overpower-at-limit", "SW-16", "8 W exactly is not over the limit", FSW + "payload_guard.rs",
      [("if s.input_w > MAX_INPUT_W {", "if s.input_w >= MAX_INPUT_W {")],
      ["cargo:sqso-fsw:payload_guard::tests::overpower_needs_two_in_a_row_latchup_one"]),
    m("guard-latchup-off", "SW-16", "any sample at 12 W trips (latch-up)", FSW + "payload_guard.rs",
      [("if s.input_w >= LATCHUP_INPUT_W {", "if s.input_w > LATCHUP_INPUT_W {")],
      ["cargo:sqso-fsw:payload_guard::tests::overpower_needs_two_in_a_row_latchup_one",
       "ctest:test_payload_guard/test_latchup"]),
    m("guard-spreader-limit", "SW-16", "heat spreader at 70 C trips", FSW + "payload_guard.rs",
      [("if s.spreader_valid && s.spreader_c >= SPREADER_MAX_C {", "if s.spreader_valid && s.spreader_c > SPREADER_MAX_C {")],
      ["cargo:sqso-fsw:payload_guard::tests::temperatures_and_unreported_junction",
       "ctest:test_payload_guard/test_temperatures"]),
    m("guard-junction-limit", "SW-16", "Jetson junction at 95 C trips", FSW + "payload_guard.rs",
      [("if s.junction_valid && s.junction_c >= JUNCTION_MAX_C {", "if s.junction_valid && s.junction_c > JUNCTION_MAX_C {")],
      ["cargo:sqso-fsw:payload_guard::tests::temperatures_and_unreported_junction",
       "ctest:test_payload_guard/test_temperatures"]),
    m("guard-junction-unreported", "SW-16", "an unreported junction is not a reading", FSW + "payload_guard.rs",
      [("if s.junction_valid && s.junction_c >= JUNCTION_MAX_C {", "if s.junction_c >= JUNCTION_MAX_C {")],
      ["cargo:sqso-fsw:payload_guard::tests::temperatures_and_unreported_junction",
       "ctest:test_payload_guard/test_temperatures"]),
    m("guard-timeout-boundary", "SW-16", "no reading for 5 s while on trips (not before)", FSW + "payload_guard.rs",
      [("now_ms >= then_ms && now_ms - then_ms <= TELEMETRY_TIMEOUT_MS", "now_ms >= then_ms && now_ms - then_ms < TELEMETRY_TIMEOUT_MS")],
      ["cargo:sqso-fsw:payload_guard::tests::unsupervised_rail_trips_after_the_grace",
       "ctest:test_payload_guard/test_no_telemetry"]),
    m("guard-spreader-not-supervised", "SW-16", "a missing spreader reading trips (power alone is not enough)",
      FSW + "payload_guard.rs",
      [(" || !age_ok(self.spreader_ms, now_ms)", "")],
      ["cargo:sqso-fsw:payload_guard::tests::unsupervised_rail_trips_after_the_grace",
       "ctest:test_payload_guard/test_no_telemetry"]),
    m("guard-rail-on-no-grace", "SW-16", "the supervision clocks restart when the rail turns on",
      FSW + "payload_guard.rs",
      [("        if payload_on && !self.was_on {\n            self.power_ms = now_ms;\n            self.spreader_ms = now_ms;\n",
        "        if payload_on && !self.was_on {\n")],
      ["cargo:sqso-fsw:payload_guard::tests::unsupervised_rail_trips_after_the_grace",
       "ctest:test_payload_guard/test_no_telemetry"]),
    m("guard-no-cooldown", "SW-16", "after a trip the rail stays off for 10 min", FSW + "payload_guard.rs",
      [("self.tripped && now_ms < self.trip_ms.saturating_add(COOLDOWN_MS)", "self.tripped && now_ms < self.trip_ms")],
      ["cargo:sqso-fsw:payload_guard::tests::cooldown_then_only_when_recently_cool",
       "ctest:test_payload_guard/test_cooldown"]),
    m("guard-restart-margin", "SW-16", "then until the spreader is below 60 C", FSW + "payload_guard.rs",
      [("let spreader_cool = self.spreader_c < SPREADER_MAX_C - RESTART_MARGIN_C;",
        "let spreader_cool = self.spreader_c < SPREADER_MAX_C;")],
      ["cargo:sqso-fsw:payload_guard::tests::cooldown_then_only_when_recently_cool",
       "cargo:sqso-fsw:payload_guard::tests::before_a_trip_only_a_known_hot_spreader_blocks",
       "ctest:test_payload_guard/test_cooldown"]),
    m("guard-restart-stale-reading", "SW-16", "the cool reading must be recent", FSW + "payload_guard.rs",
      [("spreader_recent && spreader_cool // after a trip", "spreader_cool // after a trip")],
      ["cargo:sqso-fsw:payload_guard::tests::cooldown_then_only_when_recently_cool",
       "ctest:test_payload_guard/test_cooldown"]),
    m("guard-eps-ignores", "SW-16", "eps_control_payload() refuses to enable while the guard forbids it",
      FFI + "eps_control.rs",
      [("let guard_allows = !enable || unsafe { payload_guard_may_enable(c::now_ms()) };",
        "let guard_allows = !enable || unsafe { payload_guard_may_enable(c::now_ms()) } || true;")],
      ["ctest:test_payload_guard/test_eps_honours_guard"]),
    m("guard-bus-nan", "SW-16", "the bus-voltage range check rejects NaN", FSW + "eps.rs",
      [("    (BUS_MIN_V..=BUS_MAX_V).contains(&volts)\n", "    !(volts < BUS_MIN_V || volts > BUS_MAX_V)\n")],
      ["cargo:sqso-fsw:eps::tests::soc_and_bus_limits",
       "ctest:EPS_Control_Tests/test_eps_bus_voltage_ok"]),
    m("guard-bus-range", "SW-16", "the bus-voltage range check runs on the measured value", FSW + "eps.rs",
      [("    (BUS_MIN_V..=BUS_MAX_V).contains(&volts)\n", "    !volts.is_nan()\n")],
      ["cargo:sqso-fsw:eps::tests::soc_and_bus_limits",
       "ctest:EPS_Control_Tests/test_eps_bus_voltage_ok"]),
]


def run(argv, timeout=1200, **kw):
    try:
        return subprocess.run(argv, capture_output=True, text=True, cwd=ROOT, timeout=timeout, **kw)
    except subprocess.TimeoutExpired as e:
        return subprocess.CompletedProcess(argv, 124, e.stdout or "", (e.stderr or "") + "\nTIMEOUT")


class Mutant:
    def __init__(self, mut):
        self.mut = mut
        self.path = ROOT / mut["file"]
        self.original = None

    def apply(self):
        text = self.path.read_text()
        self.original = text
        for old, new in self.mut["edits"]:
            n = text.count(old)
            if n != 1:
                raise SystemExit(f"{self.mut['id']}: expected 1 occurrence in {self.mut['file']}, found {n}:\n{old}")
            text = text.replace(old, new)
        self.path.write_text(text)

    def restore(self):
        if self.original is not None:
            self.path.write_text(self.original)
            self.original = None

    def __enter__(self):
        # Restore the file if the run is interrupted or killed
        signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
        self.apply()
        return self

    def __exit__(self, *exc):
        self.restore()


def parse_kill(kill):
    kind, rest = kill.split(":", 1)
    if kind == "cargo":
        crate, test = rest.split(":", 1)
        return kind, crate, test
    suite, case = rest.split("/", 1)
    return kind, suite, case


def ensure_build():
    if not (BUILD / "CTestTestfile.cmake").exists():
        r = run(["cmake", "-S", "software/flight", "-B", str(BUILD)])
        if r.returncode != 0:
            raise SystemExit("cmake configure failed:\n" + r.stdout + r.stderr)


def cmake_build():
    return run(["cmake", "--build", str(BUILD), "-j8"])


def cargo_test(crate, test):
    """'pass', 'fail' or 'missing' for one exactly named unit test"""
    r = subprocess.run(["cargo", "test", "-p", crate, "--lib", "--", "--exact", test],
                       capture_output=True, text=True, cwd=RUST, timeout=1200)
    out = r.stdout + r.stderr
    res = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", out)
    if not res:
        return "nobuild", out
    passed, failed = int(res.group(1)), int(res.group(2))
    if passed + failed != 1:
        return "missing", out
    return ("pass" if passed else "fail"), out


def ctest_case(suite, case):
    r = run(["ctest", "--test-dir", str(BUILD), "-R", f"^{suite}$", "--output-on-failure", "--timeout", "120"])
    out = r.stdout + r.stderr
    if "No tests were found" in out or re.search(r"\b0 tests failed out of 0\b", out):
        return "missing", out
    if re.search(r"\[\s+FAILED\s+\]\s+" + re.escape(case) + r"\b", out):
        return "fail", out
    if r.returncode == 0:
        # a passing run: make sure the case exists (cmocka prints "[ RUN      ] case" only with output, so
        # check the source instead)
        src = ROOT / "software" / "flight" / "tests"
        found = any(re.search(r"\b" + re.escape(case) + r"\b", p.read_text(errors="ignore"))
                    for p in src.rglob("*.c"))
        return ("pass" if found else "missing"), out
    return "other_failure", out


def outcome(kill):
    kind, a, b = parse_kill(kill)
    return cargo_test(a, b) if kind == "cargo" else ctest_case(a, b)


def needs_c(mut):
    return any(k.startswith("ctest:") for k in mut["kills"]) or not mut["file"].startswith("software/rust/")


def verify(selected):
    ensure_build()
    if any(needs_c(x) for x in selected):
        r = cmake_build()
        if r.returncode != 0:
            raise SystemExit("unmutated build fails:\n" + r.stdout[-3000:] + r.stderr[-3000:])
    # Every named test passes on the unmutated tree
    bad = []
    for kill in sorted({k for x in selected for k in x["kills"]}):
        res, _ = outcome(kill)
        if res != "pass":
            bad.append(f"{kill}: {res}")
    if bad:
        print("named tests that do not pass on the unmutated tree:\n  " + "\n  ".join(bad))
        return 1

    failures = 0
    for i, mut in enumerate(selected, 1):
        results = []
        with Mutant(mut):
            built = True
            if needs_c(mut):
                built = cmake_build().returncode == 0
            for kill in mut["kills"]:
                res = outcome(kill)[0] if built else "nobuild"
                results.append((kill, res))
        ok = all(res == "fail" for _, res in results)
        failures += not ok
        print(f"[{i}/{len(selected)}] {'KILLED ' if ok else 'PROBLEM'} {mut['id']}", flush=True)
        for kill, res in results:
            if res != "fail":
                print(f"      {kill}: {res} (expected fail)")
    if any(needs_c(x) for x in selected):
        cmake_build()  # leave the build directory unmutated
    print(f"\n{len(selected) - failures}/{len(selected)} mutants killed by every named test")
    return 1 if failures else 0


def survey(selected):
    """Every failing Rust unit test and C case for each mutant"""
    ensure_build()
    for i, mut in enumerate(selected, 1):
        with Mutant(mut):
            # Only sqso-protocol and sqso-fsw have unit tests, and neither
            # depends on sqso-ffi or the C headers
            if mut["file"].startswith((FSW, PROTO)):
                r = subprocess.run(["cargo", "test", "-p", "sqso-protocol", "-p", "sqso-fsw", "--no-fail-fast"],
                                   capture_output=True, text=True, cwd=RUST, timeout=1800)
                out = r.stdout + r.stderr
                rust_fail = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
                rust_note = "" if "test result" in out else " (cargo test did not build)"
            else:
                rust_fail, rust_note = [], " (no unit tests depend on this file)"
            b = cmake_build()
            if b.returncode != 0:
                c_fail, c_note = [], " (C build failed)"
            else:
                c = run(["ctest", "--test-dir", str(BUILD), "-j8", "--output-on-failure", "--timeout", "120"])
                cout = c.stdout + c.stderr
                suites = re.findall(r"\d+/\d+ Test\s+#\d+: (\S+) \.+\*+\s*(?:Failed|Exception|Timeout)", cout)
                cases = re.findall(r"\[\s+FAILED\s+\]\s+(test_\w+)", cout)
                c_fail, c_note = sorted(set(suites)) + sorted(set(cases)), ""
        print(f"[{i}/{len(selected)}] {mut['id']}", flush=True)
        print(f"    rust{rust_note}: {' '.join(rust_fail) or '-'}")
        print(f"    c{c_note}: {' '.join(c_fail) or '-'}", flush=True)
    cmake_build()
    return 0


def main():
    if "--from" in sys.argv:
        i = sys.argv.index("--from")
        start = sys.argv[i + 1]
        del sys.argv[i:i + 2]
        ids = [x["id"] for x in MUTATIONS]
        sys.argv += ids[ids.index(start):]
    args = sys.argv[1:]
    mode = "verify"
    if args and args[0] in ("--list", "--check", "--survey"):
        mode, args = args[0], args[1:]
    by_id = {x["id"]: x for x in MUTATIONS}
    unknown = [a for a in args if a not in by_id]
    if unknown:
        raise SystemExit("unknown mutation ids: " + " ".join(unknown))
    selected = [by_id[a] for a in args] if args else MUTATIONS
    if mode == "--list":
        for x in selected:
            print(f"{x['id']:38} {x['claim']:24} {x['behaviour']}")
        return 0
    if mode == "--check":
        for x in selected:
            with Mutant(x):
                pass
        print(f"{len(selected)} mutations apply")
        return 0
    if mode == "--survey":
        return survey(selected)
    return verify(selected)


if __name__ == "__main__":
    sys.exit(main())
