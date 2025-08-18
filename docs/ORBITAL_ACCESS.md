## ORBITAL ACCESS AND NRCSD IDD COMPLIANCE PLAN

This document summarizes how SMART-QSO will comply with the NanoRacks CubeSat Deployer (NRCSD) Interface Definition Document (IDD) for launch vehicle access and integration to ISS/JEM airlock deployment.

- **Primary reference**: [NanoRacks CubeSat Deployer (NRCSD) IDD](https://nanoracks.com/wp-content/uploads/Nanoracks-CubeSat-Deployer-NRCSD-IDD.pdf)

### Mission configuration
- **Form factor**: 1U CubeSat (Cal Poly CDSS compliant) packaged for NRCSD Standard.
- **Propulsion**: None (no pressure vessels or thrusters).
- **Energy storage**: 4S Li-ion 18650 pack, 50 Wh usable; below 80 Wh threshold noted in IDD safety notes.
- **Ops inside NRCSD**: Fully passive; no RF emissions, no motion, no heating.

### Mechanical interface (IDD §4.1)
- **Envelope/rails (4.1.1)**: Comply with NRCSD payload envelope and rail surface/chamfer requirements. No protrusions beyond keep-out; all deployables stowed within envelope.
- **Mass properties (4.1.2)**: Mass within 1U limit; provide CoM and inertia with tolerances. Evidence: calibrated scale and inertia fixture report.
- **RBF/ABF access (4.1.3)**: Tethered, labeled RBF pin or ABF plug accessible via NRCSD access door cutout; documented in handling procedure.
- **Deployment switches (4.1.4)**: Two independent, normally-closed rail switches actuated at ejection; both wired into inhibit chain and readable by OBC after deployment.
- **Deployables (4.1.5)**: No deployment in NRCSD. Post-ejection deployables (e.g., antenna) use non-pyro release, inhibited by deployment switches and timer logic; latch safety margins met.
- **Tip-off compatibility (4.1.6)**: ACS/detumble tolerates NRCSD tip-off; safe-mode detumble on first light.

### Electrical interface and inhibits (IDD §4.2)
- **Three independent inhibits (4.2.1)** between battery and loads:
  1) RBF/ABF hardware removal/insert,
  2) Dual deployment-switch hardware inhibits in series,
  3) Power-path FET inhibit controlled by dwell timer/logic meeting passivity rules.
- **Ground charge isolation (4.2.1-4)**: Charge port cannot energize avionics; reverse blocking and current limit; verified by schematic and test.
- **RBF/ABF function (4.2.1-5/6)**: Ensures unpowered state during handling/integration; only pre-integration battery charging allowed.
- **Electrical interfaces (4.2.2)**: No external electrical interface to NRCSD; provide ESD bonding point per electrical bonding requirements.

### Environmental compliance (IDD §4.3)
- **Acceleration/loads (4.3.1, 4.3.5)**: Structure margins to launch load factors and 1200 N Z-direction integrated loads across load points. Evidence: analysis + inspection.
- **Random vibration (4.3.2.1)**: Protoflight vibe per NRCSD random profiles in 3 axes; post-test functional checks. Evidence: vibe report.
- **Shock (4.3.3)**: Withstand launch/deploy shocks; analysis/test as applicable.
- **On-orbit acceleration (4.3.4)**: No free liquids; micro-g compatible.
- **Thermal environment (4.3.6)**: Thermal balance and vacuum compatibility; CTE/outgassing considered.
- **Humidity (4.3.7)**: Materials/finishes compatible with ISS internal humidity while stowed.
- **Airlock depressurization (4.3.8)**: Venting sized to tolerate JEM airlock depress/press rates; internal volume vent paths provided.

### Safety requirements (IDD §4.4)
- **Frangible materials (4.4.1)**: Minimized; containment as needed. Fastener locking per workmanship.
- **Venting (4.4.2)**: Vent areas sized by volume; screens to prevent FOD egress.
- **Secondary locking (4.4.3)**: Deployables have secondary locks to prevent inadvertent release.
- **Passivity (4.4.5)**: No activation inside NRCSD. Inhibits and timer logic enforce passivity until post-ejection + dwell time.
- **Pyrotechnics (4.4.5)**: None. Use non-pyro (e.g., burn-wire) with redundant inhibits; inactive in deployer.
- **Space debris (4.4.6)**: ODAR shows < 25-yr deorbit per NASA-STD-8719.14; no detachable parts in nominal ops.
- **Batteries (4.4.7)**: Acceptance tested per NanoRacks battery SOW. Cell-level OVP/UVP/OCP; pack fuse near source; ≤ 6" of ≥ 26 AWG from pack to first inhibit; restrained to prevent pouch swelling; controlled charging.
- **Pressure vessels (4.4.8)**: None.
- **Propulsion (4.4.9)**: None.
- **Materials/outgassing (4.4.10)**: Avoid prohibited materials; outgassing TML ≤ 1.0%, CVCM ≤ 0.1%. Evidence: BOM with E595 data.
- **Electrical bonding (4.4.11)**: Chassis bonding to rails; continuity verified.

### Jettison and re-entry (IDD §4.5)
- **Jettison (4.5.1)**: Dynamics compatible with NRCSD; post-ejection detumble and sun-pointing safe-mode.
- **Re-entry (4.5.2)**: ODAR shows demise; no components designed to survive re-entry.

### Documentation and regulatory (IDD §4.6)
- **Regulatory (4.6.1)**: Provide spectrum licenses/authorizations prior to payload handover (amateur coordination, national authority grants; remote sensing as applicable).
- **Deliverables (4.6.2)**: Safety data package, ICA/ICD, structural/thermal analyses, vibe/TVAC reports, EMC/ESD plans, BOM with materials/outgassing, battery test report, fit-check ROA, mass properties report.

### Operational passivity and activation logic
- **Multiple-arm criteria**: Activation requires: (a) both deployment switches opened, (b) RBF removed, (c) dwell timer elapsed (e.g., ≥ 30 min) after ejection. Until then, all loads are unpowered.
- **RF policy**: First RF transmission only after passivity criteria are met and OBC confirms safe attitude/power state.

### Verification and validation (methods: A/T/I/D)
- **Analysis**: Mass properties; structural margins; thermal model; ODAR.
- **Test**: Random vibration; TVAC functional; battery acceptance; ESD/bonding; ejector fit-check; safe-mode/detumble on flatsat.
- **Inspection**: Envelope/rails; workmanship; venting; labels; RBF access.
- **Demonstration**: Post-ejection activation timing and RF inhibit timing on flatsat.
- **Traceability**: Maintain a requirements matrix mapping IDD §5 to verification methods and artifact IDs.

### Milestones and schedule alignment (IDD §3.4.1)
- Follow NanoRacks template milestones (SRR, PDR, CDR, Safety Reviews, Fit Check, Acceptance, Final Stowage) with incremental ICA updates and data deliveries per Table 3.4.1-1.

### Configuration control
- Systems engineering maintains the Interface Control Agreement (ICA) and this `ORBITAL_ACCESS.md` as controlled documents; changes require change control and NanoRacks concurrence when applicable.

### References
- [NRCSD IDD: NanoRacks CubeSat Deployer Interface Definition Document](https://nanoracks.com/wp-content/uploads/Nanoracks-CubeSat-Deployer-NRCSD-IDD.pdf)
- NASA-STD-8719.14, Orbital Debris Mitigation
- NASA SSP 57000 (ISS internal environment references)
