//! Python module `sqso_protocol`: the wire formats of `sqso-protocol` for
//! the Jetson payload agent and the ground tools.
//!
//! Each function mirrors one in the Rust crate. Malformed input raises
//! `LinkError` (a `ValueError`), with `CrcError` for a wrong CRC, or
//! `ValueError` for builder arguments out of range.

#![allow(clippy::needless_pass_by_value)] // PyO3 argument extraction

use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use sqso_protocol::auth::{self, Key};
use sqso_protocol::jetson::{self, FromJetson, FromObc, LINE_MAX, LinkError};
use sqso_protocol::{Command, KeyClass, ax25, checksum, filter, gse};

create_exception!(sqso_protocol, PyLinkError, PyValueError, "A line that does not follow the ICD.");
create_exception!(sqso_protocol, PyCrcError, PyLinkError, "A well-formed line whose CRC does not match.");

fn link_error(e: LinkError) -> PyErr {
    match e {
        LinkError::Crc => PyCrcError::new_err("CRC mismatch"),
        LinkError::Format => PyLinkError::new_err("not a well-formed line, or a field out of range"),
        LinkError::Type => PyLinkError::new_err("not a message for this direction"),
    }
}

fn bytes<'py>(py: Python<'py>, data: &[u8]) -> Bound<'py, PyBytes> {
    PyBytes::new(py, data)
}

/// Build a line with a Rust builder
fn line<'py>(
    py: Python<'py>,
    build: impl FnOnce(&mut [u8]) -> Result<usize, LinkError>,
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; LINE_MAX];
    let n = build(&mut out).map_err(link_error)?;
    Ok(bytes(py, out.get(..n).unwrap_or(&[])))
}

/// CRC-16/CCITT-FALSE of `data`, optionally continuing from `crc`
#[pyfunction]
#[pyo3(signature = (data, crc = 0xFFFF))]
fn crc16_ccitt(data: &[u8], crc: u16) -> u16 {
    checksum::crc16_ccitt_false_from(crc, data)
}

/// CRC-32 (IEEE 802.3, as zlib.crc32)
#[pyfunction]
fn crc32(data: &[u8]) -> u32 {
    checksum::crc32(data)
}

/// AX.25 frame check sequence (CRC-16/X.25)
#[pyfunction]
fn ax25_fcs(data: &[u8]) -> u16 {
    ax25::fcs(data)
}

/// Append ",<CRC>\n" to a message body
#[pyfunction]
fn jetson_seal<'py>(py: Python<'py>, body: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    line(py, |out| jetson::seal(body, out))
}

/// Check framing and CRC of a line; returns its body
#[pyfunction]
fn jetson_check<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    jetson::check(data).map(|body| bytes(py, body)).map_err(link_error)
}

/// Parse a line the Jetson received from the OBC:
/// ("TELEMETRY", time_ms, sunlit, soc, power_mode, power_limit_w, {extras}),
/// ("BEACON_REQ", seq, max_len), ("STATUS_REQ",) or ("SHUTDOWN", reason)
#[pyfunction]
fn jetson_parse_from_obc<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyAny>> {
    let any = match jetson::parse_from_obc(data).map_err(link_error)? {
        FromObc::Telemetry(t) => {
            let extras = PyDict::new(py);
            for (k, v) in t.extras() {
                extras.set_item(k, v)?;
            }
            ("TELEMETRY", t.time_ms, t.sunlit, t.soc, t.power_mode, t.power_limit_w, extras)
                .into_pyobject(py)?
                .into_any()
        }
        FromObc::BeaconReq { seq, max_len } => ("BEACON_REQ", seq, max_len).into_pyobject(py)?.into_any(),
        FromObc::StatusReq => ("STATUS_REQ",).into_pyobject(py)?.into_any(),
        FromObc::Shutdown(reason) => ("SHUTDOWN", reason).into_pyobject(py)?.into_any(),
    };
    Ok(any)
}

/// Parse a line the OBC received from the Jetson:
/// ("BEACON_RSP", seq, text), ("STATUS", health, temp_c, power_mw, mem_pct,
/// uptime_s), ("ACK", what) or ("NACK", reason)
#[pyfunction]
fn jetson_parse_from_jetson<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyAny>> {
    let any = match jetson::parse_from_jetson(data).map_err(link_error)? {
        FromJetson::BeaconRsp { seq, text } => ("BEACON_RSP", seq, text).into_pyobject(py)?.into_any(),
        FromJetson::Status(s) => {
            ("STATUS", s.health, s.temp_c, s.power_mw, s.mem_pct, s.uptime_s).into_pyobject(py)?.into_any()
        }
        FromJetson::Ack(t) => ("ACK", t).into_pyobject(py)?.into_any(),
        FromJetson::Nack(t) => ("NACK", t).into_pyobject(py)?.into_any(),
    };
    Ok(any)
}

fn u16_arg(value: i64, what: &str) -> PyResult<u16> {
    u16::try_from(value).map_err(|_| PyLinkError::new_err(format!("{what} out of range")))
}

/// BEACON_RSP,<seq>,<text>
#[pyfunction]
fn jetson_beacon_rsp<'py>(py: Python<'py>, seq: i64, text: String) -> PyResult<Bound<'py, PyBytes>> {
    let seq = u16_arg(seq, "seq")?;
    line(py, |out| jetson::beacon_rsp(seq, &text, out))
}

/// STATUS,<health>,<temp_c>,<power_mw>,<mem_pct>,<uptime_s>
#[pyfunction]
fn jetson_status<'py>(
    py: Python<'py>,
    health: i64,
    temp_c: i64,
    power_mw: i64,
    mem_pct: i64,
    uptime_s: i64,
) -> PyResult<Bound<'py, PyBytes>> {
    let range = || PyLinkError::new_err("STATUS field out of range");
    let status = jetson::Status {
        health: u8::try_from(health).map_err(|_| range())?,
        temp_c: i32::try_from(temp_c).map_err(|_| range())?,
        power_mw: u32::try_from(power_mw).map_err(|_| range())?,
        mem_pct: u8::try_from(mem_pct).map_err(|_| range())?,
        uptime_s: u32::try_from(uptime_s).map_err(|_| range())?,
    };
    line(py, |out| jetson::status(&status, out))
}

/// ACK,<what>
#[pyfunction]
fn jetson_ack<'py>(py: Python<'py>, what: String) -> PyResult<Bound<'py, PyBytes>> {
    line(py, |out| jetson::ack(&what, out))
}

/// NACK,<reason>
#[pyfunction]
fn jetson_nack<'py>(py: Python<'py>, reason: String) -> PyResult<Bound<'py, PyBytes>> {
    line(py, |out| jetson::nack(&reason, out))
}

/// BEACON_REQ,<seq>,<max_len>
#[pyfunction]
fn jetson_beacon_req<'py>(py: Python<'py>, seq: i64, max_len: i64) -> PyResult<Bound<'py, PyBytes>> {
    let seq = u16_arg(seq, "seq")?;
    let max_len = u8::try_from(max_len).map_err(|_| PyLinkError::new_err("max_len out of range"))?;
    line(py, |out| jetson::beacon_req(seq, max_len, out))
}

/// STATUS_REQ
#[pyfunction]
fn jetson_status_req<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
    line(py, jetson::status_req)
}

/// SHUTDOWN,<reason>
#[pyfunction]
fn jetson_shutdown<'py>(py: Python<'py>, reason: String) -> PyResult<Bound<'py, PyBytes>> {
    line(py, |out| jetson::shutdown(&reason, out))
}

/// Beacon content filter: "OK" or the first rule broken
#[pyfunction]
fn filter_check(text: &Bound<'_, PyAny>) -> PyResult<&'static str> {
    if text.is_none() {
        return Ok(filter::Verdict::Length.name());
    }
    if let Ok(s) = text.extract::<String>() {
        // A non-ASCII character is outside the character set, whatever its encoding
        return Ok(filter::check(s.as_bytes()).name());
    }
    let b: Vec<u8> = text.extract()?;
    Ok(filter::check(&b).name())
}

/// The command dictionary: (name, id, min_args, max_args, key, without_key, summary)
#[pyfunction]
fn commands() -> Vec<(&'static str, u8, u8, u8, &'static str, bool, &'static str)> {
    Command::ALL
        .iter()
        .map(|c| {
            let s = c.spec();
            let key = match s.key {
                KeyClass::Operational => "operational",
                KeyClass::Recovery => "recovery",
                KeyClass::Any => "any",
            };
            (c.name(), c.id(), s.min_args, s.max_args, key, s.without_key, c.summary())
        })
        .collect()
}

fn key(bytes: &[u8]) -> PyResult<Key> {
    Key::from_slice(bytes).ok_or_else(|| PyValueError::new_err(format!("key must be {} bytes", auth::KEY_SIZE)))
}

fn build_error(e: auth::BuildError) -> PyErr {
    PyValueError::new_err(match e {
        auth::BuildError::Counter => "counter must be 1..0xfffffffe",
        auth::BuildError::Args => "args must be at most 64 bytes",
        auth::BuildError::Space => "frame does not fit",
    })
}

/// counter | args | tag for one command
#[pyfunction]
#[pyo3(signature = (key_bytes, cmd_id, counter, args = b"".as_slice()))]
fn tc_payload<'py>(
    py: Python<'py>,
    key_bytes: &[u8],
    cmd_id: u8,
    counter: u32,
    args: &[u8],
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; auth::OVERHEAD + sqso_protocol::MAX_ARGS];
    let n = auth::build_payload(&key(key_bytes)?, cmd_id, counter, args, &mut out).map_err(build_error)?;
    Ok(bytes(py, out.get(..n).unwrap_or(&[])))
}

/// The complete uplink frame for one authenticated command
#[pyfunction]
#[pyo3(signature = (key_bytes, cmd_id, counter, args = b"".as_slice()))]
fn tc_frame<'py>(
    py: Python<'py>,
    key_bytes: &[u8],
    cmd_id: u8,
    counter: u32,
    args: &[u8],
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; sqso_protocol::frame::MAX_LEN];
    let n = auth::build_frame(&key(key_bytes)?, cmd_id, counter, args, &mut out).map_err(build_error)?;
    Ok(bytes(py, out.get(..n).unwrap_or(&[])))
}

/// AX.25 UI frame (PID 0xF0) from the address field through the FCS (or
/// without it, for a KISS TNC that appends it)
#[pyfunction]
#[pyo3(signature = (dest, src, info, fcs = true))]
fn ax25_ui_frame<'py>(
    py: Python<'py>,
    dest: String,
    src: String,
    info: &[u8],
    fcs: bool,
) -> PyResult<Bound<'py, PyBytes>> {
    let station = |s: &str| {
        let upper = s.to_ascii_uppercase();
        ax25::Station::parse(&upper)
            .map(|st| (st.call.to_owned(), st.ssid))
            .map_err(|_| PyValueError::new_err(format!("invalid AX.25 call sign {s:?}")))
    };
    let (dest_call, dest_ssid) = station(&dest)?;
    let (src_call, src_ssid) = station(&src)?;
    let mut out = [0u8; ax25::MAX_FRAME_LEN];
    let n = ax25::encode_ui(
        ax25::Station { call: &dest_call, ssid: dest_ssid },
        ax25::Station { call: &src_call, ssid: src_ssid },
        info,
        fcs,
        &mut out,
    )
    .map_err(|_| PyValueError::new_err("AX.25 frame too long"))?;
    Ok(bytes(py, out.get(..n).unwrap_or(&[])))
}

/// Wrap a frame for a KISS TNC (data frame on the given port)
#[pyfunction]
#[pyo3(signature = (frame, port = 0))]
fn kiss_frame<'py>(py: Python<'py>, frame: &[u8], port: u8) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = vec![0u8; 3 + 2 * frame.len()];
    let n = ax25::kiss_encode(frame, port, &mut out).map_err(|_| PyValueError::new_err("KISS frame too long"))?;
    Ok(bytes(py, out.get(..n).unwrap_or(&[])))
}

/// A GSE umbilical provisioning frame
#[pyfunction]
#[pyo3(signature = (op, slot, key_id, key_bytes = b"".as_slice()))]
fn gse_frame<'py>(py: Python<'py>, op: u8, slot: u8, key_id: u8, key_bytes: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let mut k = [0u8; auth::KEY_SIZE];
    if !key_bytes.is_empty() {
        k = key_bytes.try_into().map_err(|_| PyValueError::new_err(format!("key must be {} bytes", auth::KEY_SIZE)))?;
    }
    Ok(bytes(py, &gse::encode(op, slot, key_id, &k)))
}

fn words(list: &[&core::ffi::CStr]) -> Vec<&'static str> {
    list.iter().filter_map(|w| core::str::from_utf8(w.to_bytes()).ok()).map(|w| &*w.to_owned().leak()).collect()
}

/// The `sqso_protocol` module
#[pymodule(name = "sqso_protocol")]
fn sqso_protocol_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("LinkError", py.get_type::<PyLinkError>())?;
    m.add("CrcError", py.get_type::<PyCrcError>())?;
    m.add("LINE_MAX", jetson::LINE_MAX)?;
    m.add("TEXT_MAX", jetson::TEXT_MAX)?;
    m.add("FILTER_MAX_LEN", filter::MAX_LEN)?;
    m.add("FILTER_MAX_WORD", filter::MAX_WORD)?;
    m.add("PROFANITY", pyo3::types::PyTuple::new(py, words(filter::PROFANITY))?)?;
    m.add("COMMERCIAL", pyo3::types::PyTuple::new(py, words(filter::COMMERCIAL))?)?;
    m.add("URL_MARKS", pyo3::types::PyTuple::new(py, filter::URL_MARKS)?)?;
    m.add("CALLSIGN", sqso_protocol::station::CALLSIGN)?;
    m.add("SSID", sqso_protocol::station::SSID)?;
    m.add("TC_DOMAIN", PyBytes::new(py, auth::DOMAIN))?;
    m.add("TC_KEY_SIZE", auth::KEY_SIZE)?;
    m.add("TC_TAG_SIZE", auth::TAG_SIZE)?;
    m.add("TC_MAX_ARGS", sqso_protocol::MAX_ARGS)?;
    m.add("TC_COUNTER_LIMIT", auth::COUNTER_LIMIT)?;
    m.add_function(wrap_pyfunction!(crc16_ccitt, m)?)?;
    m.add_function(wrap_pyfunction!(crc32, m)?)?;
    m.add_function(wrap_pyfunction!(ax25_fcs, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_seal, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_check, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_parse_from_obc, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_parse_from_jetson, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_beacon_rsp, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_status, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_ack, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_nack, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_beacon_req, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_status_req, m)?)?;
    m.add_function(wrap_pyfunction!(jetson_shutdown, m)?)?;
    m.add_function(wrap_pyfunction!(filter_check, m)?)?;
    m.add_function(wrap_pyfunction!(commands, m)?)?;
    m.add_function(wrap_pyfunction!(tc_payload, m)?)?;
    m.add_function(wrap_pyfunction!(tc_frame, m)?)?;
    m.add_function(wrap_pyfunction!(ax25_ui_frame, m)?)?;
    m.add_function(wrap_pyfunction!(kiss_frame, m)?)?;
    m.add_function(wrap_pyfunction!(gse_frame, m)?)?;
    Ok(())
}
