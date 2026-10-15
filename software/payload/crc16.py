"""
CRC-16 for the OBC-Jetson link.

Implements the checksum defined in docs/interfaces/ICD_OBC_JETSON.md §4.4:
polynomial 0x1021, initial value 0xFFFF, no input/output reflection and no
final XOR (catalogued as CRC-16/CCITT-FALSE, also CRC-16/IBM-3740).
Check value for b"123456789" is 0x29B1.

Implemented locally rather than through a third-party package so the
interface definition cannot drift with an external library's API or presets.
"""

CRC16_POLY = 0x1021
CRC16_INIT = 0xFFFF
CRC16_CHECK = 0x29B1  # CRC of b"123456789"


def crc16_ccitt(data: bytes, crc: int = CRC16_INIT) -> int:
    """Return the ICD CRC-16 of ``data``, optionally continuing from ``crc``."""
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ CRC16_POLY) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return crc
