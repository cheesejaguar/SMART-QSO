//! The host simulation's operating system: clock, sleep, files, environment
//! and console, through libc. Compiled only for the host (`target_os` is not
//! `"none"`); on the target the firmware provides the clock and storage.

// Callers arrive as the remaining C modules are ported
#![allow(dead_code)]

use core::ffi::{CStr, c_char, c_int};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

/// The SIL simulation's virtual clock (`crate::sil`): when on, the clock
/// reads it and sleeping advances it, so simulated years pass without
/// waiting
static VIRTUAL_ON: AtomicBool = AtomicBool::new(false);
static VIRTUAL_US: AtomicU64 = AtomicU64::new(0);

/// The SIL simulation's console sink (a `fn(&[u8])` as a usize; 0: stdout)
static SINK: AtomicUsize = AtomicUsize::new(0);

/// Switch the clock to virtual time, starting at `start_us`
pub(crate) fn virtual_clock(start_us: u64) {
    VIRTUAL_US.store(start_us, Ordering::Relaxed);
    VIRTUAL_ON.store(true, Ordering::Relaxed);
}

/// Advance the virtual clock (nothing in wall-clock mode)
pub(crate) fn virtual_advance_us(us: u64) {
    let _ = VIRTUAL_US.fetch_add(us, Ordering::Relaxed);
}

/// Send console and error lines to `sink` instead of stdout and stderr
/// (`None`: back to them)
pub(crate) fn console_sink(sink: Option<fn(&[u8])>) {
    SINK.store(sink.map_or(0, |f| f as usize), Ordering::Relaxed);
}

/// The console sink, if one is set
fn sink() -> Option<fn(&[u8])> {
    match SINK.load(Ordering::Relaxed) {
        0 => None,
        // SAFETY: only console_sink() stores a non-zero value, and it is a
        // fn(&[u8]) converted to usize
        f => Some(unsafe { core::mem::transmute::<usize, fn(&[u8])>(f) }),
    }
}

/// Wall-clock time, ms since the Unix epoch (virtual time if it is on)
pub(crate) fn realtime_ms() -> u64 {
    if VIRTUAL_ON.load(Ordering::Relaxed) {
        return VIRTUAL_US.load(Ordering::Relaxed) / 1000;
    }
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: ts is a valid timespec to write
    if unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) } != 0 {
        return 0;
    }
    let secs = u64::try_from(ts.tv_sec).unwrap_or(0);
    let ms = u64::try_from(ts.tv_nsec).unwrap_or(0) / 1_000_000;
    secs.saturating_mul(1000).saturating_add(ms)
}

/// Wall-clock time, us since the Unix epoch (virtual time if it is on)
pub(crate) fn realtime_us() -> u64 {
    if VIRTUAL_ON.load(Ordering::Relaxed) {
        return VIRTUAL_US.load(Ordering::Relaxed);
    }
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: ts is a valid timespec to write
    if unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) } != 0 {
        return 0;
    }
    let secs = u64::try_from(ts.tv_sec).unwrap_or(0);
    let us = u64::try_from(ts.tv_nsec).unwrap_or(0) / 1000;
    secs.saturating_mul(1_000_000).saturating_add(us)
}

/// Sleep for `us` microseconds (on the virtual clock: advance it)
pub(crate) fn sleep_us(us: u64) {
    if VIRTUAL_ON.load(Ordering::Relaxed) {
        virtual_advance_us(us);
        return;
    }
    let ts = libc::timespec {
        tv_sec: libc::time_t::try_from(us / 1_000_000).unwrap_or(libc::time_t::MAX),
        tv_nsec: libc::c_long::try_from((us % 1_000_000) * 1000).unwrap_or(0),
    };
    // SAFETY: ts is a valid timespec; the remainder pointer may be NULL
    let _ = unsafe { libc::nanosleep(&ts, core::ptr::null_mut()) };
}

/// The value of an environment variable, if set and not empty
pub(crate) fn env(name: &CStr) -> Option<&'static CStr> {
    // SAFETY: name is NUL-terminated; getenv returns NULL or a C string that
    // stays valid while the environment is not modified (the simulation and
    // the tests set variables before the module reads them)
    let value = unsafe { libc::getenv(name.as_ptr()) };
    if value.is_null() {
        return None;
    }
    // SAFETY: non-NULL getenv result is NUL-terminated
    let value = unsafe { CStr::from_ptr(value) };
    if value.is_empty() { None } else { Some(value) }
}

/// A file opened with `fopen`, closed when dropped
struct File(*mut libc::FILE);

impl File {
    fn open(path: &CStr, mode: &CStr) -> Option<File> {
        // SAFETY: both are NUL-terminated
        let f = unsafe { libc::fopen(path.as_ptr(), mode.as_ptr()) };
        if f.is_null() { None } else { Some(File(f)) }
    }

    fn close(self) -> bool {
        let f = self.0;
        core::mem::forget(self);
        // SAFETY: f is an open FILE, closed once
        unsafe { libc::fclose(f) == 0 }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // SAFETY: self.0 is an open FILE, closed once
        let _ = unsafe { libc::fclose(self.0) };
    }
}

/// Replace a file's contents
pub(crate) fn file_write(path: &CStr, data: &[u8]) -> bool {
    let Some(f) = File::open(path, c"wb") else { return false };
    // SAFETY: data is a live slice; f is open
    let written = data.is_empty() || unsafe { libc::fwrite(data.as_ptr().cast(), 1, data.len(), f.0) } == data.len();
    f.close() && written
}

/// Read a file into `buf`; the number of bytes read
pub(crate) fn file_read(path: &CStr, buf: &mut [u8]) -> Option<usize> {
    let f = File::open(path, c"rb")?;
    // SAFETY: buf is a live, writable slice; f is open
    let n = unsafe { libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), f.0) };
    if f.close() { Some(n) } else { None }
}

/// Remove a file (absence is not an error)
pub(crate) fn file_remove(path: &CStr) {
    // SAFETY: path is NUL-terminated
    let _ = unsafe { libc::remove(path.as_ptr()) };
}

/// Write one line to stdout (through C stdio, so it interleaves with C output)
pub(crate) fn print_line(line: &[u8]) {
    if let Some(f) = sink() {
        f(line);
        return;
    }
    let len = c_int::try_from(line.len()).unwrap_or(c_int::MAX);
    // SAFETY: the format takes a length and a pointer to that many bytes
    let _ = unsafe { libc::printf(c"%.*s\n".as_ptr(), len, line.as_ptr().cast::<c_char>()) };
}

/// Write the concatenation of `parts` and a newline to stdout
pub(crate) fn print_parts(parts: &[&[u8]]) {
    if let Some(f) = sink() {
        let mut line = [0u8; 256];
        let mut n = 0;
        for &b in parts.iter().flat_map(|p| p.iter()) {
            if let Some(slot) = line.get_mut(n) {
                *slot = b;
                n += 1;
            }
        }
        f(line.get(..n).unwrap_or(&[]));
        return;
    }
    for part in parts {
        let len = c_int::try_from(part.len()).unwrap_or(c_int::MAX);
        // SAFETY: the format takes a length and a pointer to that many bytes
        let _ = unsafe { libc::printf(c"%.*s".as_ptr(), len, part.as_ptr().cast::<c_char>()) };
    }
    // SAFETY: a NUL-terminated format with no arguments
    let _ = unsafe { libc::printf(c"\n".as_ptr()) };
}

/// Write one line to stderr
pub(crate) fn eprint_line(line: &[u8]) {
    if let Some(f) = sink() {
        f(line);
        return;
    }
    // SAFETY: stdout is flushed first so the two streams stay in order
    unsafe {
        let _ = libc::fflush(core::ptr::null_mut());
        let _ = libc::write(2, line.as_ptr().cast(), line.len());
        let _ = libc::write(2, c"\n".as_ptr().cast(), 1);
    }
}

/// Write raw bytes to stdout
pub(crate) fn print_bytes(data: &[u8]) {
    if let Some(f) = sink() {
        f(data);
        return;
    }
    // SAFETY: flush C stdio first, then write the live slice
    unsafe {
        let _ = libc::fflush(core::ptr::null_mut());
        let _ = libc::write(1, data.as_ptr().cast(), data.len());
    }
}

/// End the process (a fatal error on the host)
pub(crate) fn abort() -> ! {
    // SAFETY: no preconditions
    unsafe { libc::abort() }
}
