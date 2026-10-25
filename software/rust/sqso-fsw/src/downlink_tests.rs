use super::*;

const COOL: f64 = 25.0;
/// Output present, matched
const GOOD: Sample = Sample { temp_c: COOL, fwd_w: 0.5, rev_w: 0.01 };
/// Output present, VSWR above 3:1
const BAD: Sample = Sample { temp_c: COOL, fwd_w: 0.5, rev_w: 0.2 };
/// No output
const DEAD: Sample = Sample { temp_c: COOL, fwd_w: 0.0, rev_w: 0.0 };

fn ready(now_ms: u64) -> Downlink {
    let mut d = Downlink::new();
    d.init(now_ms);
    d
}

fn send(d: &mut Downlink, now_ms: u64, len: usize) -> Result<Plan, Refusal> {
    d.request(now_ms, len, true, POWER_1W, COOL)
}

/// Poll once a sample is due (after the keying delay) with `s`
fn poll_at(d: &mut Downlink, now_ms: u64, s: Sample) -> Option<Event> {
    d.poll(now_ms, true, || s)
}

/// After a send at `t`, the first time the monitors are read
fn sampling(t: u64) -> u64 {
    t + u64::from(kiss::TXDELAY_10MS) * 10
}

#[test]
fn air_and_uart_times() {
    // Empty frame: 32 bits, 39 stuffed, 33 ms, plus 300 ms delay and 10 ms tail
    assert_eq!(airtime_ms(0), 343);
    // Longest: 332 bytes, 2656 bits, 3188 stuffed, 2657 ms
    assert_eq!(airtime_ms(MAX_FRAME_LEN), 2967);
    assert!(airtime_ms(MAX_FRAME_LEN) <= MAX_KEYDOWN_MS);
    assert_eq!(airtime_ms(usize::MAX), u32::MAX);
    // Five escaped commands (25 bytes), 2 × 10 + 3 frame bytes: 48 bytes at 9600 baud
    assert_eq!(uart_ms(10), 50);
    assert_eq!(uart_ms(usize::MAX), u32::MAX);
}

#[test]
fn nothing_before_init() {
    let mut d = Downlink::new();
    assert_eq!(send(&mut d, 0, 20), Err(Refusal::Inhibited));
    assert_eq!(d.counters.inhibited, 1);
    assert_eq!(Downlink::default(), Downlink::new());
}

#[test]
fn a_frame_goes_at_the_commanded_level() {
    let mut d = ready(1000);
    assert_eq!(send(&mut d, 1000, 100), Ok(Plan { level: POWER_1W, airtime_ms: airtime_ms(100) }));
    assert_eq!(d.counters.sent, 1);
    assert!(d.keyed(1000));
    let mut d = ready(1000);
    assert_eq!(d.request(1000, 100, true, POWER_0W5, COOL).map(|p| p.level), Ok(POWER_0W5));
}

#[test]
fn not_permitted_is_refused() {
    let mut d = ready(0);
    assert_eq!(d.request(0, 20, false, POWER_1W, COOL), Err(Refusal::Inhibited));
    assert_eq!(d.counters.inhibited, 1);
    assert!(!d.keyed(0));
}

#[test]
fn lengths_outside_the_frame_are_refused() {
    let mut d = ready(0);
    assert_eq!(send(&mut d, 0, 0), Err(Refusal::Invalid));
    assert_eq!(send(&mut d, 0, MAX_FRAME_LEN + 1), Err(Refusal::Invalid));
    assert_eq!(d.counters.invalid, 2);
    assert!(send(&mut d, 0, MAX_FRAME_LEN).is_ok());
}

#[test]
fn one_frame_at_a_time() {
    let mut d = ready(0);
    let plan = send(&mut d, 0, 50).unwrap();
    let end = u64::from(uart_ms(50)) + u64::from(plan.airtime_ms);
    assert_eq!(send(&mut d, end - 1, 50), Err(Refusal::Busy));
    assert_eq!(d.counters.busy, 1);
    assert!(!d.keyed(end));
    assert!(send(&mut d, end, 50).is_ok());
}

#[test]
fn keyed_time_is_held_to_the_duty_limit() {
    let mut d = ready(0);
    let mut now = 0u64;
    let mut on_air = 0u64;
    let mut refused = 0u32;
    // An hour of back-to-back longest frames
    while now < 3_600_000 {
        match send(&mut d, now, MAX_FRAME_LEN) {
            Ok(plan) => {
                on_air += u64::from(plan.airtime_ms);
                now += u64::from(uart_ms(MAX_FRAME_LEN)) + u64::from(plan.airtime_ms);
            }
            Err(Refusal::Duty) => {
                refused += 1;
                now += 20;
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    assert!(refused > 0);
    assert_eq!(d.counters.duty, refused);
    // 25 % of the hour, plus the full credit at the start
    assert!(on_air <= now * DUTY_PERCENT / 100 + DUTY_WINDOW_MS * DUTY_PERCENT / 100);
    assert!(on_air >= now * DUTY_PERCENT / 100 - u64::from(airtime_ms(MAX_FRAME_LEN)));
}

#[test]
fn credit_refills_to_one_window_at_most() {
    let mut d = ready(0);
    // Spend nearly everything
    let mut now = 0;
    while send(&mut d, now, MAX_FRAME_LEN).is_ok() {
        now += u64::from(uart_ms(MAX_FRAME_LEN)) + u64::from(airtime_ms(MAX_FRAME_LEN));
    }
    // A day idle refills only one window's worth: five longest frames, not more
    now += 86_400_000;
    let mut burst = 0;
    while send(&mut d, now, MAX_FRAME_LEN).is_ok() {
        burst += 1;
        now += u64::from(uart_ms(MAX_FRAME_LEN)) + u64::from(airtime_ms(MAX_FRAME_LEN));
    }
    assert!((5..=8).contains(&burst), "{burst}");
}

#[test]
fn a_clock_that_steps_back_refills_nothing() {
    let mut d = ready(10_000);
    while send(&mut d, 10_000, MAX_FRAME_LEN).is_ok() {
        d.keyed_until_ms = 0;
    }
    assert_eq!(send(&mut d, 5_000, MAX_FRAME_LEN), Err(Refusal::Duty));
}

#[test]
fn cancel_returns_the_credit() {
    let mut d = ready(0);
    let before = d;
    let plan = send(&mut d, 0, 100).unwrap();
    d.cancel(plan);
    assert!(!d.keyed(0));
    assert_eq!(d.counters.sent, 0);
    assert_eq!(d.counters.hal_errors, 1);
    assert_eq!(d.credit, before.credit);
    assert!(send(&mut d, 0, 100).is_ok());
}

#[test]
fn hot_holds_the_downlink_until_cooled_below_the_resume_point() {
    let mut d = ready(0);
    assert_eq!(d.request(0, 20, true, POWER_1W, TEMP_LIMIT_C + 0.5), Err(Refusal::OverTemperature));
    assert!(d.over_temperature());
    assert_eq!(d.request(0, 20, true, POWER_1W, TEMP_RESUME_C + 1.0), Err(Refusal::OverTemperature));
    assert_eq!(d.counters.over_temperature, 2);
    assert!(d.request(0, 20, true, POWER_1W, TEMP_RESUME_C - 0.5).is_ok());
    assert!(!d.over_temperature());
    // At the limit exactly: not hot
    let mut d = ready(0);
    assert!(d.request(0, 20, true, POWER_1W, TEMP_LIMIT_C).is_ok());
}

#[test]
fn a_failed_temperature_sensor_changes_nothing() {
    let mut d = ready(0);
    assert!(d.request(0, 20, true, POWER_1W, f64::NAN).is_ok());
    let mut d = ready(0);
    let _ = d.request(0, 20, true, POWER_1W, 90.0);
    assert_eq!(d.request(0, 20, true, POWER_1W, f64::NAN), Err(Refusal::OverTemperature));
}

#[test]
fn polling_idle_reads_nothing() {
    let mut d = ready(0);
    assert_eq!(d.poll(0, true, || panic!("read while idle")), None);
}

#[test]
fn nothing_is_read_during_the_keying_delay() {
    let mut d = ready(0);
    send(&mut d, 0, 100).unwrap();
    assert_eq!(d.poll(sampling(0) - 1, true, || panic!("read before the carrier")), None);
    assert_eq!(poll_at(&mut d, sampling(0), GOOD), None);
}

#[test]
fn a_bad_match_caps_the_power_then_backs_off() {
    let mut d = ready(0);
    send(&mut d, 0, 300).unwrap();
    let t = sampling(0);
    assert_eq!(poll_at(&mut d, t, BAD), None);
    assert_eq!(poll_at(&mut d, t + 20, BAD), None);
    assert_eq!(poll_at(&mut d, t + 40, BAD), Some(Event::PowerReduced));
    assert_eq!(d.level_cap(), POWER_0W5);
    assert_eq!(d.counters.vswr_events, 1);

    // Commanded 1 W now goes at 0.5 W
    let next = 10_000;
    assert_eq!(send(&mut d, next, 300).map(|p| p.level), Ok(POWER_0W5));
    let t = sampling(next);
    for k in 0..u64::from(VSWR_STRIKES) - 1 {
        assert_eq!(poll_at(&mut d, t + 20 * k, BAD), None);
    }
    let at = t + 20 * u64::from(VSWR_STRIKES - 1);
    assert_eq!(poll_at(&mut d, at, BAD), Some(Event::BackOff));
    assert!(d.backed_off(at));

    // Nothing until the back-off is over
    let later = at + 20_000;
    assert_eq!(send(&mut d, later, 30), Err(Refusal::BackOff));
    assert_eq!(d.counters.backed_off, 1);
    assert!(!d.backed_off(at + BACKOFF_MS));
    assert_eq!(send(&mut d, at + BACKOFF_MS, 30).map(|p| p.level), Ok(POWER_0W5));
}

#[test]
fn a_good_or_empty_sample_resets_the_strikes() {
    for between in [GOOD, DEAD] {
        let mut d = ready(0);
        send(&mut d, 0, 300).unwrap();
        let t = sampling(0);
        assert_eq!(poll_at(&mut d, t, BAD), None);
        assert_eq!(poll_at(&mut d, t + 20, BAD), None);
        assert_eq!(poll_at(&mut d, t + 40, between), None);
        assert_eq!(poll_at(&mut d, t + 60, BAD), None);
        assert_eq!(poll_at(&mut d, t + 80, BAD), None);
        assert_eq!(d.level_cap(), POWER_1W);
    }
}

#[test]
fn the_vswr_and_presence_boundaries() {
    let mut d = ready(0);
    send(&mut d, 0, 300).unwrap();
    let t = sampling(0);
    // Exactly 3:1 is not a bad match
    let edge = Sample { temp_c: COOL, fwd_w: 0.4, rev_w: 0.1 };
    for k in 0..4 {
        assert_eq!(poll_at(&mut d, t + 20 * k, edge), None);
    }
    // Forward power at the presence threshold counts as output
    let weak = Sample { temp_c: COOL, fwd_w: RF_PRESENT_W, rev_w: RF_PRESENT_W };
    assert_eq!(poll_at(&mut d, t + 100, weak), None);
    assert_eq!(poll_at(&mut d, t + 120, weak), None);
    assert_eq!(poll_at(&mut d, t + 140, weak), Some(Event::PowerReduced));
}

#[test]
fn a_keyed_window_without_output_is_reported_once() {
    let mut d = ready(0);
    let plan = send(&mut d, 0, 100).unwrap();
    let end = u64::from(uart_ms(100)) + u64::from(plan.airtime_ms);
    assert_eq!(poll_at(&mut d, sampling(0), DEAD), None);
    assert_eq!(poll_at(&mut d, end, DEAD), Some(Event::NoOutput));
    assert_eq!(d.counters.no_output, 1);
    assert_eq!(poll_at(&mut d, end + 20, DEAD), None);
}

#[test]
fn a_window_with_output_or_no_samples_is_not_reported() {
    let mut d = ready(0);
    let plan = send(&mut d, 0, 100).unwrap();
    let end = u64::from(uart_ms(100)) + u64::from(plan.airtime_ms);
    assert_eq!(poll_at(&mut d, sampling(0), DEAD), None);
    assert_eq!(poll_at(&mut d, sampling(0) + 20, GOOD), None);
    assert_eq!(poll_at(&mut d, end, DEAD), None);

    let mut d = ready(0);
    send(&mut d, 0, 100).unwrap();
    assert_eq!(poll_at(&mut d, end, DEAD), None);
    assert_eq!(d.counters.no_output, 0);
}

#[test]
fn tx_off_during_a_window_ends_it_quietly() {
    let mut d = ready(0);
    send(&mut d, 0, 100).unwrap();
    assert_eq!(poll_at(&mut d, sampling(0), DEAD), None);
    assert_eq!(d.poll(sampling(0) + 20, false, || panic!("read after TX-OFF")), None);
    assert!(!d.keyed(sampling(0) + 20));
    assert_eq!(d.counters.no_output, 0);
}

#[test]
fn temperature_edges_while_keyed() {
    let mut d = ready(0);
    send(&mut d, 0, 300).unwrap();
    let t = sampling(0);
    let hot = Sample { temp_c: TEMP_LIMIT_C + 1.0, ..GOOD };
    assert_eq!(poll_at(&mut d, t, hot), Some(Event::OverTemperature));
    assert_eq!(poll_at(&mut d, t + 20, hot), None);
    let cooled = Sample { temp_c: TEMP_RESUME_C - 1.0, ..GOOD };
    assert_eq!(poll_at(&mut d, t + 40, cooled), Some(Event::Cooled));
    assert!(!d.over_temperature());
}

#[test]
fn a_failed_power_reading_is_no_sample() {
    let nan_fwd = Sample { fwd_w: f64::NAN, ..BAD };
    let nan_rev = Sample { rev_w: f64::NAN, ..BAD };
    let mut d = ready(0);
    let plan = send(&mut d, 0, 300).unwrap();
    let end = u64::from(uart_ms(300)) + u64::from(plan.airtime_ms);
    let t = sampling(0);
    for k in 0..6 {
        let s = if k % 2 == 0 { nan_fwd } else { nan_rev };
        assert_eq!(poll_at(&mut d, t + 20 * k, s), None);
    }
    assert_eq!(d.level_cap(), POWER_1W);
    // Only failed readings in the window: not reported as no output
    assert_eq!(poll_at(&mut d, end, DEAD), None);
    assert_eq!(d.counters.no_output, 0);
    // A failed reading between bad ones does not reset the strikes
    let mut d = ready(0);
    send(&mut d, 0, 300).unwrap();
    assert_eq!(poll_at(&mut d, t, BAD), None);
    assert_eq!(poll_at(&mut d, t + 20, nan_fwd), None);
    assert_eq!(poll_at(&mut d, t + 40, BAD), None);
    assert_eq!(poll_at(&mut d, t + 60, BAD), Some(Event::PowerReduced));
}
