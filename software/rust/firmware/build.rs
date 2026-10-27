//! Puts `memory.x` (the image's memory map, with the statics in SRAM2) on
//! the linker's search path for cortex-m-rt's `link.x`

use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default());
    if std::fs::write(out.join("memory.x"), include_bytes!("memory.x")).is_err() {
        std::process::exit(1);
    }
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}
