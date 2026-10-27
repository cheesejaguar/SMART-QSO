//! The run's records, in the output directory:
//!
//! - `timeseries.csv`: every quantity worth plotting, at the record interval
//! - `orbits.csv`: one row per orbit (energy balance, depth of discharge,
//!   extremes, link and payload activity)
//! - `passes.csv`: one row per ground-station pass
//! - `events.jsonl`: mode changes, resets, deployments, faults, commands,
//!   anomalies, one JSON object a line
//! - `console.log`: the flight software's console and flight-log lines with
//!   the mission time (sensor readings excluded)

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// A CSV table with a fixed header
pub struct Table {
    out: BufWriter<File>,
    columns: usize,
    /// Rows written
    pub rows: u64,
}

impl Table {
    /// Create `name` in `dir` with `header`
    pub fn create(dir: &Path, name: &str, header: &[&str]) -> std::io::Result<Table> {
        let mut out = BufWriter::new(File::create(dir.join(name))?);
        writeln!(out, "{}", header.join(","))?;
        Ok(Table { out, columns: header.len(), rows: 0 })
    }

    /// Append one row (numbers printed compactly)
    pub fn row(&mut self, values: &[f64]) {
        debug_assert_eq!(values.len(), self.columns);
        let mut line = String::with_capacity(values.len() * 8);
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            push_num(&mut line, *v);
        }
        let _ = writeln!(self.out, "{line}");
        self.rows += 1;
    }

    /// Flush to disk
    pub fn flush(&mut self) {
        let _ = self.out.flush();
    }
}

/// A number with at most 6 significant figures, integers without a point
pub fn push_num(s: &mut String, v: f64) {
    use std::fmt::Write as _;
    if !v.is_finite() {
        s.push_str("nan");
    } else if v == v.trunc() && v.abs() < 1e15 {
        let _ = write!(s, "{}", v as i64);
    } else {
        let a = v.abs();
        let decimals = if a >= 1000.0 {
            1
        } else if a >= 10.0 {
            3
        } else if a >= 0.01 {
            5
        } else {
            8
        };
        let _ = write!(s, "{v:.decimals$}");
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// JSON-escape a string
pub fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// The event log and the console log
pub struct Logs {
    events: BufWriter<File>,
    console: BufWriter<File>,
    /// Events written, by kind (for the summary)
    pub counts: std::collections::BTreeMap<String, u64>,
    dir: PathBuf,
}

impl Logs {
    /// Create both in `dir`
    pub fn create(dir: &Path) -> std::io::Result<Logs> {
        Ok(Logs {
            events: BufWriter::new(File::create(dir.join("events.jsonl"))?),
            console: BufWriter::new(File::create(dir.join("console.log"))?),
            counts: Default::default(),
            dir: dir.to_path_buf(),
        })
    }

    /// The output directory
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// An event at mission time `t_s` (seconds after deployment; negative
    /// before it) with `kind` and a free-text `detail`
    pub fn event(&mut self, t_s: f64, utc: &str, kind: &str, detail: &str) {
        *self.counts.entry(kind.to_string()).or_default() += 1;
        let _ = writeln!(
            self.events,
            "{{\"t_s\":{t_s:.1},\"utc\":{},\"kind\":{},\"detail\":{}}}",
            json_str(utc),
            json_str(kind),
            json_str(detail)
        );
    }

    /// A flight-software console line
    pub fn console(&mut self, t_s: f64, line: &str) {
        let _ = writeln!(self.console, "{t_s:12.1} {line}");
    }

    /// Flush both
    pub fn flush(&mut self) {
        let _ = self.events.flush();
        let _ = self.console.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_compact() {
        let mut s = String::new();
        for v in [3.0, 0.5, 12.345678, 1234.56, 0.000123, -2.5, f64::NAN] {
            push_num(&mut s, v);
            s.push(' ');
        }
        assert_eq!(s, "3 0.5 12.346 1234.6 0.000123 -2.5 nan ");
    }

    #[test]
    fn json_escapes() {
        assert_eq!(json_str("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }
}
