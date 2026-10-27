/* SMART-QSO OBC (STM32L4R5ZI): the memory map the image links to.
 *
 * FLASH: both banks (2 MB). RAM: SRAM2 and SRAM3 (64 KB + 384 KB) from
 * 0x20030000, so that the statics (.data, .bss, .uninit) are linked at the
 * start of SRAM2, the only RAM with parity (RID AV-07, firmware/src/parity.rs),
 * and the 8 KB stack at the top of SRAM3 (initial SP 0x200A0000). SRAM1
 * (0x20000000-0x2002FFFF) is not linked: parity.rs keeps the NMI's cause
 * record at its top.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 2048K
    RAM   : ORIGIN = 0x20030000, LENGTH = 448K
}

/* Every static in SRAM2 */
ASSERT(_stack_end <= 0x20040000, "SMART-QSO: the statics outgrow SRAM2 (64 KB), the parity-protected RAM (AV-07)");
