# SMART-QSO Architecture Overview

- **Bus:** 1U CubeSat, deployable solar wings, UHF/VHF transponder, magnetorquer ADCS.
- **Compute:** Ultra‑low‑power OBC (Ambiq Apollo4/STM32L4) + **Jetson Orin Nano Super** (declocked, DVFS‑limited) power‑gated for burst inference.
- **Agents:** Power Manager (forecast & scheduling), QSO Priority (tiny CNN), Adaptive Transponder Controller, Smart Beacon (TinyLM ~1 MB).
- **Fail‑graceful:** if AI payload is off or faults, revert to basic transponder + beacon.
- **Data:** FRAM (state/params), microSD/NAND for logs; beacons via AX.25.
