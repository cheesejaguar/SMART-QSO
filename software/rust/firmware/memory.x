/* SMART-QSO OBC (STM32L4R5ZI): the memory map the image links to.
 *
 * FLASH: both banks (2 MB). RAM: SRAM2 only (64 KB from 0x20030000), the
 * STM32L4R5's only RAM with parity (RID AV-07, firmware/src/parity.rs): the
 * statics (.data, .bss, .uninit) at its start and the 8 KB stack at its top
 * (initial SP 0x20040000, RID AV-13). SRAM1 (0x20000000-0x2002FFFF) is not
 * linked: parity.rs keeps the NMI's cause record at its top. SRAM3 is not
 * used.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 2048K
    RAM   : ORIGIN = 0x20030000, LENGTH = 64K
}

/* The statics, the 32-byte MPU guard and the 8 KB stack fit SRAM2 */
ASSERT(_stack_end + 32 + 8192 <= 0x20040000, "SMART-QSO: the statics and the 8 KB stack outgrow SRAM2 (64 KB), the parity-protected RAM (AV-07, AV-13)");
