//! NVM records over arbitrary flash contents (a corrupted or hostile
//! image): loading never fails unexpectedly, and a save is read back
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_fsw::nvm::{self, MemFlash, RecordStatus};

fuzz_target!(|data: &[u8]| {
    let mut flash = MemFlash::<256>::new();
    let n = data.len().min(256);
    flash.bytes[..n].copy_from_slice(&data[..n]);
    let magic = 0x5351_5453;
    let mut out = [0u8; 16];
    nvm::load(&mut flash, 0, 0, magic, &mut out).expect("in-bounds loads succeed");
    nvm::save(&mut flash, 0, 0, magic, &[0xC3; 16]).expect("in-bounds saves succeed");
    assert_eq!(nvm::load(&mut flash, 0, 0, magic, &mut out), Ok(RecordStatus::Valid));
    assert_eq!(out, [0xC3; 16]);
});
