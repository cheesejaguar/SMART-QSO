# SMART-QSO Document Register

**Document Number:** SQSO-DOC-001
**Revision:** B
**Date:** 2026-10-16

---

## 1. Purpose and Rules

This is the controlled list of SMART-QSO documents (findings PI-17 and VV-18). Every file that carries a document number is
listed once in §2, and every test procedure defined inside a larger document is listed in §3. CI runs
`scripts/check_document_ids.py --check`, which fails if a number is the header of more than one file, a test-procedure ID
is defined in more than one place, or a numbered document or procedure is missing from this register.

1. A document's number is the one in its own header line (`**Document Number:**` or `**Document ID:**`). Numbers are
   compared exactly as written. Some documents use the prefix SMART-QSO- and others SQSO- (§5).
2. A new document takes a number that no other document carries and gets a row here in the same change. A retired number
   is not reused.
3. Revision is as written in the document's header.
4. Status shows what the repository supports. No document in the repository records an approval, and no independent PDR
   or CDR has been held (`docs/BASELINE_PARAMETERS.md` §1). Documents that call themselves "CDR Baseline", "Complete" or
   "COMPLIANT" are therefore listed as Draft. Templates are listed as Template. Where a document carries a funding-audit
   notice or a header that limits its standing, the status says so.

**Configuration tags.** No Git tags or other configuration baselines exist yet, so no revision listed here belongs to a
tagged baseline. The first configuration baseline tag is to be created when PDR is held (finding PI-04).

## 2. Documents

| ID | Title | Path | Revision | Status |
|----|-------|------|----------|--------|
| SQSO-BL-001 | SMART-QSO Baseline Parameters (Single Source of Truth) | `docs/BASELINE_PARAMETERS.md` | E | Draft; governs where documents disagree (its §0). Records PI decisions D1–D9 |
| SQSO-OAP-001 | Orbital Access Plan | `docs/ORBITAL_ACCESS.md` | D | Draft; launch chain, backup and decision points DP-1 to DP-9 |
| SQSO-DOC-001 | SMART-QSO Document Register | `docs/DOCUMENT_REGISTER.md` | B | Draft |
| SQSO-ES-001 | SMART-QSO Executive Summary | `docs/EXECUTIVE_SUMMARY.md` | 2.2 | Draft |
| SQSO-CDS-001 | CubeSat Design Specification Compliance Matrix | `docs/compliance/CDS_COMPLIANCE_MATRIX.md` | C | Draft; statuses are design intent, not verified (its notice) |
| SQSO-COMP-004 | Export Control Assessment | `docs/compliance/EXPORT_CONTROL.md` | A | Draft; its system and Jetson classifications are superseded by SQSO-REG-012 |
| SQSO-ICD-010 | Launch Provider Interface Control Document | `docs/compliance/LAUNCH_PROVIDER_ICD.md` | G | Draft; not agreed with any provider (its notice) |
| SQSO-MAT-001 | Materials and Outgassing Compliance List | `docs/compliance/MATERIALS_LIST.md` | E | Draft |
| SQSO-ODAR-001 | Orbital Debris Assessment Report | `docs/compliance/ORBITAL_DEBRIS_ASSESSMENT.md` | A | Superseded by SQSO-REG-018 (its notice) |
| SQSO-BS-001 | Branching Strategy | `docs/development/BRANCHING_STRATEGY.md` | A | Draft |
| SQSO-DEV-003 | Code Review Checklist | `docs/development/CODE_REVIEW_CHECKLIST.md` | B | Draft |
| SQSO-CS-001 | Coding Standards | `docs/development/CODING_STANDARDS.md` | B | Draft |
| SQSO-DEV-004 | Commit Message Standard | `docs/development/COMMIT_MESSAGES.md` | A | Draft |
| SQSO-DEV-005 | Software Release Procedure | `docs/development/RELEASE_PROCEDURE.md` | E | Draft |
| SQSO-DB-001 | SMART-QSO Data Budget | `docs/environment/DATA_BUDGET.md` | 2.2 | Draft |
| SQSO-ETP-001 | Environmental Test Plan | `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` | D | Draft |
| SQSO-LBA-001 | SMART-QSO Link Budget Analysis | `docs/environment/LINK_BUDGET_ANALYSIS.md` | 3.5 | Draft |
| SQSO-MB-001 | SMART-QSO Mass Budget | `docs/environment/MASS_BUDGET.md` | 3.1 | Draft |
| SQSO-OA-001 | SMART-QSO Orbit Analysis | `docs/environment/ORBIT_ANALYSIS.md` | 1.2 | Draft |
| SQSO-PBA-001 | SMART-QSO Power Budget Analysis | `docs/environment/POWER_BUDGET_ANALYSIS.md` | 2.4 | Draft |
| SQSO-BSA-001 | SMART-QSO Battery Safety Assessment | `docs/environment/BATTERY_SAFETY_ASSESSMENT.md` | G | Draft; analysis |
| SQSO-EMC-001 | SMART-QSO Electromagnetic Compatibility Assessment | `docs/environment/EMC_ANALYSIS.md` | D | Draft; analysis |
| SQSO-SA-001 | SMART-QSO Structural Analysis | `docs/environment/STRUCTURAL_ANALYSIS.md` | B | Draft; analysis |
| SQSO-MECH-001 | SMART-QSO Mechanisms Analysis | `docs/environment/MECHANISMS_ANALYSIS.md` | E | Draft; analysis |
| SQSO-ENV-002 | Radiation Environment Analysis | `docs/environment/RADIATION_ANALYSIS.md` | D | Draft |
| SQSO-REL-001 | SMART-QSO Reliability Analysis | `docs/environment/RELIABILITY_ANALYSIS.md` | 2.2 | Draft |
| SQSO-THA-001 | SMART-QSO Thermal Analysis | `docs/environment/THERMAL_ANALYSIS.md` | 2.4 | Draft |
| SQSO-GND-001 | Ground Station Design | `docs/ground/GROUND_STATION_DESIGN.md` | H | Draft |
| SQSO-GSOM-001 | SMART-QSO Ground Station Operations Manual | `docs/ground/GROUND_STATION_OPERATIONS_MANUAL.md` | 1.7 | Draft |
| SQSO-GND-002 | Orbit Determination and Operator Ephemeris | `docs/ground/ORBIT_DETERMINATION_AND_EPHEMERIS.md` | A | Draft; owner: ground segment lead |
| SMART-QSO-ADCS-001 | Attitude Determination and Control Subsystem (ADCS) Design Document | `docs/hardware/ADCS_DESIGN.md` | 1.7 | Draft |
| SQSO-BVB-001 | Build versus Buy: COTS Survey, Gyro, ADCS Gains and Battery | `docs/hardware/BUILD_VS_BUY.md` | C | Draft; vendor data from public pages, no quotes |
| SMART-QSO-EPS-001 | Electrical Power Subsystem (EPS) Design Document | `docs/hardware/EPS_DESIGN.md` | 2.5 | Draft |
| SQSO-HDP-001 | SMART-QSO Hardware Development Plan | `docs/hardware/HARDWARE_DEVELOPMENT_PLAN.md` | 1.0 | Draft |
| SQSO-BOM-001 | SMART-QSO Parts List / Bill of Materials | `docs/hardware/PARTS_LIST.md` | 1.1 | Draft |
| SMART-QSO-PAYLOAD-001 | AI Payload (Jetson Orin Nano) Design Document | `docs/hardware/PAYLOAD_DESIGN.md` | 1.2 | Draft |
| SQSO-PAY-SEL-001 | OBC Selector and Transponder Scheduler: Detailed Design | `docs/payload/OBC_SELECTOR_DESIGN.md` | A | Draft; designed, not implemented |
| SMART-QSO-RF-001 | Radio Frequency (RF) Subsystem Design Document | `docs/hardware/RF_DESIGN.md` | 2.4 | Draft; architecture superseded by the baseline (its notice) |
| SQSO-INT-005 | Anomaly Resolution Procedure | `docs/integration/ANOMALY_RESOLUTION.md` | A | Draft |
| SQSO-ASM-001 | SMART-QSO Assembly Procedure | `docs/integration/ASSEMBLY_PROCEDURE.md` | 1.0 | Draft |
| SQSO-CCP-001 | SMART-QSO Contamination Control Plan | `docs/integration/CONTAMINATION_CONTROL_PLAN.md` | 2.0 | Draft |
| SQSO-INT-004 | Flatsat Configuration | `docs/integration/FLATSAT_CONFIGURATION.md` | A | Draft |
| SQSO-INT-001 | Integration Plan | `docs/integration/INTEGRATION_PLAN.md` | C | Draft |
| SQSO-INT-003 | Pre-Ship Review Checklist | `docs/integration/PRE_SHIP_CHECKLIST.md` | A | Draft |
| SQSO-INT-006 | Test Data Management Plan | `docs/integration/TEST_DATA_MANAGEMENT.md` | B | Draft |
| SMART-QSO-ICD-006 | Interface Control Document: Beacon Format | `docs/interfaces/ICD_BEACON_FORMAT.md` | 1.0 | Draft |
| SQSO-ICD-004 | Interface Control Document: OBC to ADCS Subsystem | `docs/interfaces/ICD_OBC_ADCS.md` | 1.4 | Draft |
| SMART-QSO-ICD-001 | Interface Control Document: OBC-EPS | `docs/interfaces/ICD_OBC_EPS.md` | 1.6 | Draft |
| SMART-QSO-ICD-002 | Interface Control Document: OBC-Jetson | `docs/interfaces/ICD_OBC_JETSON.md` | 1.4 | Draft |
| SQSO-ICD-003 | Interface Control Document: OBC to RF Subsystem | `docs/interfaces/ICD_OBC_RF.md` | 2.3 | Draft |
| SQSO-ICD-005 | Interface Control Document: Space-to-Ground | `docs/interfaces/ICD_SPACE_GROUND.md` | 2.5 | Draft |
| SQSO-ICD-000 | Interface Control Document - Master | `docs/interfaces/INTERFACE_CONTROL_MASTER.md` | C | Draft |
| SQSO-IVP-001 | SMART-QSO Interface Verification Plan | `docs/interfaces/INTERFACE_VERIFICATION_PLAN.md` | 1.0 | Draft |
| SQSO-ICD-013 | Interface Control Document: Telecommand Authentication and TX-OFF | `docs/interfaces/TELECOMMAND_AUTHENTICATION.md` | 0.13 | Draft |
| SQSO-MFG-002 | EEE Parts List | `docs/manufacturing/EEE_PARTS_LIST.md` | G | Draft |
| SQSO-MFG-003 | Parts Temperature-Grade Audit (every flight BOM line, harness and bought items; line data in `PARTS_TEMPERATURE_AUDIT.csv`) | `docs/manufacturing/PARTS_TEMPERATURE_AUDIT.md` | C | Draft; CDR round 3 audit (RID P3T-01), BOM changes for the designers |
| SQSO-WP-001 | Workmanship Plan | `docs/manufacturing/WORKMANSHIP_PLAN.md` | B | Draft |
| SQSO-WKS-001 | SMART-QSO Workmanship Standards | `docs/manufacturing/WORKMANSHIP_STANDARDS.md` | 1.1 | Draft |
| SQSO-MSC-001 | SMART-QSO Mission Success Criteria | `docs/mission/MISSION_SUCCESS_CRITERIA.md` | 2.2 | Draft |
| SQSO-LVR-001 | Launch Vehicle and Dispenser Requirements Flow-Down | `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` | E | Draft; from published provider documents, no mission ICD yet |
| SQSO-SCH-001 | Integrated Master Schedule | `mission/SCHEDULE.md` | D | Draft; planning estimates |
| SQSO-RISK-001 | Risk Register | `mission/RISK_REGISTER.md` | E | Living document |
| SQSO-AR-001 | SMART-QSO Anomaly Report Template | `docs/operations/ANOMALY_REPORT_TEMPLATE.md` | 1.0 | Template |
| SQSO-ARP-001 | SMART-QSO As-Run Procedure Template | `docs/operations/AS_RUN_PROCEDURE_TEMPLATE.md` | 1.0 | Template |
| SQSO-OPS-003 | Command and Telemetry Database | `docs/operations/CMD_TLM_DATABASE.md` | A | Draft; command sections superseded (its notice) |
| SQSO-CMD-001 | SMART-QSO Command Dictionary | `docs/operations/COMMAND_DICTIONARY.md` | 1.3 | Draft |
| SQSO-COMM-001 | SMART-QSO Commissioning Plan | `docs/operations/COMMISSIONING_PLAN.md` | 2.1 | Draft |
| SQSO-CP-001 | SMART-QSO Contingency Procedures | `docs/operations/CONTINGENCY_PROCEDURES.md` | 1.0 | Draft |
| SQSO-EOL-001 | End-of-Life Plan | `docs/operations/END_OF_LIFE_PLAN.md` | D | Draft |
| SQSO-OPS-005 | Launch Campaign Checklist | `docs/operations/LAUNCH_CAMPAIGN_CHECKLIST.md` | B | Draft |
| SQSO-OPS-004 | Launch Site Operations Plan | `docs/operations/LAUNCH_SITE_OPS.md` | C | Draft |
| SQSO-MOP-001 | Mission Operations Plan | `docs/operations/MISSION_OPERATIONS_PLAN.md` | B | Draft |
| SQSO-MRC-001 | SMART-QSO Mission Rules and Constraints | `docs/operations/MISSION_RULES.md` | 1.0 | Draft |
| SQSO-MTL-001 | SMART-QSO Mission Timeline and Event Sequence | `docs/operations/MISSION_TIMELINE.md` | 2.5 | Draft |
| SQSO-OTM-001 | SMART-QSO Operator Training Materials | `docs/operations/OPERATOR_TRAINING_MATERIALS.md` | 1.0 | Draft |
| SQSO-OTP-001 | SMART-QSO Operator Training Plan | `docs/operations/OPERATOR_TRAINING_PLAN.md` | 1.0 | Draft |
| SQSO-SRP-001 | SMART-QSO Software Rollback and Recovery Procedures | `docs/operations/SOFTWARE_ROLLBACK_PROCEDURES.md` | 1.0 | Draft |
| SQSO-TLM-001 | SMART-QSO Telemetry Dictionary | `docs/operations/TELEMETRY_DICTIONARY.md` | 1.4 | Draft |
| SQSO-AUD-006 | Onboard AI Model Selection (Decision D5) | `docs/payload/AI_MODEL_SELECTION.md` | A | Draft |
| SQSO-AUD-001 | SMART-QSO Funding Audit Report | `docs/funding_audit/FUNDING_AUDIT_REPORT.md` | B | Audit record (2026-10-14); under document control from 2026-10-27 (RID P3P-09) |
| SQSO-AUD-002 | Funding Audit — Findings Register | `docs/funding_audit/FINDINGS_REGISTER.md` | B | Living register (`scripts/register_status.py --check`) |
| SQSO-AUD-003 | NASA Standards and Regulatory Compliance Matrix | `docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md` | C | Audit record |
| SQSO-AUD-004 | Launch and Rideshare Trade Study | `docs/funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md` | B | Audit record; superseded for the launch path by SQSO-OAP-001 |
| SQSO-AUD-005 | Lifecycle Cost — Rough Order of Magnitude (ROM) | `docs/funding_audit/LIFECYCLE_COST_ROM.md` | F | Live cost estimate (ROM, no quotes); registered 2026-10-27 (RID P3P-09) |
| SMART-QSO-CMP-001 | Configuration Management Plan (CMP) | `docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` | 1.2 | Draft |
| SQSO-CRL-001 | SMART-QSO Change Request Log | `docs/project/CHANGE_REQUEST_LOG.md` | H | Living document |
| SQSO-HRT-001 | SMART-QSO Hardware Readiness Tracker | `docs/project/HARDWARE_READINESS_TRACKER.md` | 2.1 | Draft |
| SQSO-TRA-001 | Technology Readiness Assessment and Technology Development Plan | `docs/project/TECHNOLOGY_READINESS_ASSESSMENT.md` | A | Draft; assessment for the PDR (RID SE-09) |
| SQSO-HA-001 | SMART-QSO Heritage Assessment | `docs/project/HERITAGE_ASSESSMENT.md` | 2.0 | Draft |
| SQSO-LL-001 | SMART-QSO Lessons Learned | `docs/project/LESSONS_LEARNED.md` | 1.1 | Living document |
| SQSO-MAP-001 | SMART-QSO Mission Assurance Plan | `docs/project/MISSION_ASSURANCE_PLAN.md` | 2.0 | Draft |
| SQSO-PM-001 | Project Plan | `docs/project/PROJECT_PLAN.md` | C | Draft |
| SQSO-PM-002 | CDR Governing Framework, Tailoring and Criteria Compliance Matrix | `docs/project/CDR_CRITERIA_MATRIX.md` | B | Draft |
| SMART-QSO-SEMP-001 | Systems Engineering Management Plan (SEMP) | `docs/project/SEMP.md` | 2.2 | Draft |
| SQSO-REG-001 | IARU Satellite Frequency Coordination — Status | `docs/regulatory/IARU_COORDINATION.md` | F | Draft |
| SQSO-REG-003 | Licensing Pathway Determination | `docs/regulatory/LICENSING_PATHWAY_DETERMINATION.md` | B.1 | Draft |
| SQSO-REG-004 | Regulatory and Launch-Authorization Roadmap | `docs/regulatory/REGULATORY_ROADMAP.md` | D | Draft |
| SQSO-REG-002 | Spectrum Authorization Status | `docs/regulatory/SPECTRUM_LICENSE.md` | D | Draft |
| SQSO-REG-012 | Export Control Classification Memorandum | `docs/regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md` | B | Draft; to be confirmed by export counsel (its header) |
| SQSO-REG-014 | FCC Part 97 In-Space and Post-Space Notifications (Templates) | `docs/regulatory/filings/FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md` | A.2 | Template; not filed |
| SQSO-REG-013 | FCC Part 97 Pre-Space Notification (Draft) | `docs/regulatory/filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md` | A.3 | Draft; not filed |
| SQSO-REG-010 | IARU Amateur Satellite Frequency Coordination Request — Draft (form v40 layout) | `docs/regulatory/filings/IARU_COORDINATION_REQUEST_v40.md` | A.6 | Draft; not submitted |
| SQSO-REG-017 | ITU Appendix 4 Data Sheet (Advance Publication / Notification) | `docs/regulatory/filings/ITU_APPENDIX4_DATA.md` | A.3 | Draft |
| SQSO-REG-016 | Launch Provider Deliverables Package | `docs/regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md` | D | Draft |
| SQSO-REG-011 | NOAA Commercial Remote Sensing — License Applicability Determination | `docs/regulatory/filings/NOAA_REMOTE_SENSING_DETERMINATION.md` | A.1 | Draft |
| SQSO-REG-018 | Orbital Debris Assessment Report (ODAR) | `docs/regulatory/filings/ODAR_SMART-QSO.md` | Draft D.2 | Draft; NASA DAS not yet run, due before the early pre-space filing (its header) |
| SQSO-REG-015 | Space Safety Registration and Conjunction Response Plan | `docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md` | C | Draft |
| SQSO-REQ-001 | Mission Requirements Document | `docs/requirements/MISSION_REQUIREMENTS.md` | F | Draft |
| SMART-QSO-RTM-001 | Requirements Traceability Matrix (RTM) | `docs/requirements/REQUIREMENTS_TRACEABILITY.md` | 1.1 | Draft |
| SQSO-REQ-002 | System Requirements Specification | `docs/requirements/SYSTEM_REQUIREMENTS.md` | G | Draft |
| SQSO-REV-CDR-001 | SMART-QSO Critical Design Review (CDR) | `docs/reviews/CDR/CDR_REVIEW_MINUTES.md` | — | Self-assessment record; no independent CDR held |
| SQSO-CDR-TB-001 | SMART-QSO CDR Technical Baseline | `docs/reviews/CDR/CDR_TECHNICAL_BASELINE.md` | 1.3 | Self-assessment; not an independent review (its notice) |
| SMART-QSO-CDR-001 | SMART-QSO Critical Design Review (CDR) Package | `docs/reviews/CDR_PACKAGE.md` | 1.5 | Self-assessment; not an independent review (its notice) |
| SQSO-REV-003 | CDR Status Summary | `docs/reviews/CDR_STATUS_SUMMARY.md` | C | Draft; status claims not substantiated (its notice) |
| SQSO-FRR-001 | SMART-QSO Flight Readiness Review Package | `docs/reviews/FLIGHT_READINESS_REVIEW.md` | 1.0 | Template |
| SQSO-FSW-DEL-001 | SMART-QSO Flight Software Delivery Package | `docs/reviews/FLIGHT_SOFTWARE_DELIVERY.md` | 1.2 | Draft; status claims not substantiated (its notice) |
| SQSO-FIR-001 | SMART-QSO Formal Inspection Report Template | `docs/reviews/FORMAL_INSPECTION_REPORT.md` | 1.0 | Template |
| SQSO-KI-001 | SMART-QSO Known Issues Document | `docs/reviews/KNOWN_ISSUES.md` | 1.3 | Draft |
| SQSO-LRR-001 | SMART-QSO Launch Readiness Review Package | `docs/reviews/LAUNCH_READINESS_REVIEW.md` | 1.0 | Template |
| SQSO-MOR-001 | SMART-QSO Mission Operations Review Package | `docs/reviews/MISSION_OPERATIONS_REVIEW.md` | 1.0 | Template |
| SQSO-REV-PDR-001 | SMART-QSO Preliminary Design Review (PDR) | `docs/reviews/PDR/PDR_REVIEW_MINUTES.md` | — | Self-assessment record; no independent PDR held |
| SMART-QSO-PDR-001 | SMART-QSO Preliminary Design Review (PDR) Package | `docs/reviews/PDR_PACKAGE.md` | 1.3 | Self-assessment; not an independent review (its notice) |
| SQSO-PSR-001 | SMART-QSO Post-Environmental Review Package | `docs/reviews/POST_ENVIRONMENTAL_REVIEW.md` | 1.1 | Template |
| SQSO-PER-001 | SMART-QSO Pre-Environmental Review Package | `docs/reviews/PRE_ENVIRONMENTAL_REVIEW.md` | 1.1 | Template |
| SQSO-SR-001 | SMART-QSO Safety Review Package | `docs/reviews/SAFETY_REVIEW.md` | 1.0 | Self-assessment record; not an independent review (its notice) |
| SQSO-SRR-001 | SMART-QSO System Requirements Review Package | `docs/reviews/SYSTEM_REQUIREMENTS_REVIEW.md` | 1.1 | Self-assessment record; no independent SRR held |
| SQSO-TRR-001 | SMART-QSO Test Readiness Review Package | `docs/reviews/TEST_READINESS_REVIEW.md` | 1.0 | Template |
| SQSO-REV-004 | SMART-QSO Verification and Validation Package | `docs/reviews/VERIFICATION_VALIDATION_PACKAGE.md` | 1.2 | Draft; status claims not substantiated (its notice) |
| SQSO-SDP-001 | SMART-QSO Software Safety Data Package | `docs/safety/SAFETY_DATA_PACKAGE.md` | 1.2 | Draft |
| SQSO-SDP-002 | SMART-QSO System Safety Data Package | `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` | 1.1 | Draft; 16 hazard reports |
| SQSO-FMEA-002 | SMART-QSO System (Hardware) FMEA and Critical Items List | `docs/safety/SYSTEM_FMEA_CIL.md` | D | Draft; analysis; CIL-14 to CIL-28 from SQSO-FMEA-003 |
| SQSO-FMEA-003 | SMART-QSO Board-Level FMEA of the Build-to Designs | `docs/safety/BOARD_FMEA.md` | B | Draft; analysis; rows in `docs/safety/BOARD_FMEA_ROWS.csv`, tables by `scripts/board_fmea.py` |
| SQSO-SEC-001 | Space System Protection and Security Plan | `docs/safety/SPACE_SYSTEM_PROTECTION_PLAN.md` | A | Draft |
| SQSO-REV-CDR-002 | Design Review against the NPR 7123.1D CDR Criteria (2026-10-27) | `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md` | A | Draft; engineering assessment by AI reviewers, not an independent board |
| SQSO-FMEA-001 | Software Failure Mode and Effects Analysis (FMEA) | `docs/safety/SOFTWARE_FMEA.md` | C | Draft |
| SQSO-SHA-001 | Software Hazard Analysis | `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` | F | Draft |
| SQSO-SSP-001 | Software Safety Plan | `docs/safety/SOFTWARE_SAFETY_PLAN.md` | E | Draft |
| SQSO-SW-008 | Build Configurations | `docs/software/BUILD_CONFIGURATIONS.md` | E | Draft |
| SQSO-SW-006 | Software Configuration Index | `docs/software/CONFIGURATION_INDEX.md` | B | Draft |
| SQSO-RRP-001 | SMART-QSO Rollback and Recovery Procedures | `docs/software/ROLLBACK_RECOVERY_PROCEDURES.md` | 1.2 | Draft |
| SMART-QSO-SDD-001 | Software Design Description (SDD) | `docs/software/SOFTWARE_DESIGN.md` | 1.1 | Draft |
| SMART-QSO-SDP-001 | Software Development Plan (SDP) | `docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` | 1.5 | Draft |
| SQSO-SMP-001 | SMART-QSO Software Maintenance Plan | `docs/software/SOFTWARE_MAINTENANCE_PLAN.md` | 1.1 | Draft |
| SMART-QSO-SRS-001 | Software Requirements Specification (SRS) | `docs/software/SOFTWARE_REQUIREMENTS.md` | 1.10 | Draft |
| SQSO-FSW-012 | NPR 7150.2D Requirements Mapping and Compliance Matrix (Class C) | `docs/software/NPR7150_COMPLIANCE_MATRIX.md` | B | Draft |
| SMART-QSO-STP-001 | Software Test Plan (STP) | `docs/software/SOFTWARE_TEST_PLAN.md` | 1.2 | Draft |
| SQSO-SVD-001 | Software Version Description | `docs/software/SOFTWARE_VERSION_DESCRIPTION.md` | D | Draft |
| SQSO-E2E-001 | SMART-QSO End-to-End Test Plan | `docs/test/END_TO_END_TEST_PLAN.md` | 1.0 | Draft |
| SQSO-FTEG-001 | SMART-QSO Flatsat Test Execution Guide | `docs/test/FLATSAT_TEST_EXECUTION_GUIDE.md` | 1.0 | Draft |
| SQSO-TP-001 | SMART-QSO Flatsat Test Procedures | `docs/test/FLATSAT_TEST_PROCEDURES.md` | 1.0 | Draft |
| SQSO-STC-001 | SMART-QSO System Test Cases | `docs/test/SYSTEM_TEST_CASES.md` | 1.0 | Draft |
| SQSO-SIL-001 | Mission Simulation (Software in the Loop) | `docs/test/SIL_MISSION_SIMULATION.md` | D | Draft; models are analysis-grade, not correlated with hardware |
| SQSO-ATP-001 | SMART-QSO Acceptance Test Procedure | `docs/test_procedures/ACCEPTANCE_TEST_PROCEDURE.md` | 1.0 | Draft; no run recorded |
| TP-ADCS-001 | Test Procedure: ADCS Sensor Test | `docs/test_procedures/TP-ADCS-001_Sensor_Test.md` | B | Draft; no run recorded |
| SQSO-TP-ADCS-002 | Test Procedure: ADCS Actuator Test | `docs/test_procedures/TP-ADCS-002_Actuator_Test.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-ADCS-003 | Test Procedure: ADCS Algorithm Test | `docs/test_procedures/TP-ADCS-003_Algorithm_Test.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-DITL-001 | TP-DITL-001: Day-in-the-Life Test Procedure | `docs/test_procedures/TP-DITL-001_DAY_IN_THE_LIFE.md` | 1.1 | Draft; no run recorded |
| TP-ENV-001 | Test Procedure: Pre-Environmental Baseline | `docs/test_procedures/TP-ENV-001_Pre_Environmental.md` | C | Draft; no run recorded |
| TP-ENV-002 | Test Procedure: Vibration Test | `docs/test_procedures/TP-ENV-002_Vibration.md` | F | Draft; no run recorded |
| TP-ENV-003 | Test Procedure: Thermal Vacuum Test | `docs/test_procedures/TP-ENV-003_Thermal_Vacuum.md` | E | Draft; no run recorded |
| SQSO-TP-ENV-004 | Test Procedure: EMC/EMI Test | `docs/test_procedures/TP-ENV-004_EMC.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-ENV-005 | Test Procedure: Deployment Test (Post-Environmental) | `docs/test_procedures/TP-ENV-005_Deployment_Post_Env.md` | 1.1 | Draft; no run recorded |
| TP-EPS-001 | Test Procedure: EPS Functional Test | `docs/test_procedures/TP-EPS-001_Functional_Test.md` | C | Draft; no run recorded |
| TP-EPS-002 | Test Procedure: EPS Load Profile Test | `docs/test_procedures/TP-EPS-002_Load_Profile.md` | C | Draft; no run recorded |
| SQSO-TP-EPS-003 | Test Procedure: Battery Characterization | `docs/test_procedures/TP-EPS-003_Battery_Characterization.md` | 1.0 | Draft; no run recorded |
| SQSO-TP-EPS-004 | Test Procedure: Solar Array Characterization | `docs/test_procedures/TP-EPS-004_Solar_Array.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-FAT-001 | Test Procedure: Final Acceptance Functional Test | `docs/test_procedures/TP-FAT-001_Final_Functional.md` | 1.3 | Draft; no run recorded |
| SQSO-TP-FAT-002 | Test Procedure: Flight Configuration Verification | `docs/test_procedures/TP-FAT-002_Flight_Config.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-FLATSAT-001 | Test Procedure: Flatsat Software Development | `docs/test_procedures/TP-FLATSAT-001_Software_Development.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-GS-001 | TP-GS-001: Ground Station Validation Test Procedure | `docs/test_procedures/TP-GS-001_Ground_Station_Validation.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-INT-001 | Test Procedure: EPS Integration | `docs/test_procedures/TP-INT-001_EPS_Integration.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-INT-002 | Test Procedure: OBC Integration | `docs/test_procedures/TP-INT-002_OBC_Integration.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-INT-003 | Test Procedure: RF Subsystem Integration | `docs/test_procedures/TP-INT-003_RF_Integration.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-INT-004 | Test Procedure: ADCS Integration | `docs/test_procedures/TP-INT-004_ADCS_Integration.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-INT-005 | Test Procedure: Jetson Payload Integration | `docs/test_procedures/TP-INT-005_Jetson_Integration.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-INT-006 | Test Procedure: Final Assembly Integration | `docs/test_procedures/TP-INT-006_Final_Assembly.md` | 1.3 | Draft; no run recorded |
| TP-JETSON-001 | Test Procedure: Jetson Payload Power Test | `docs/test_procedures/TP-JETSON-001_Power_Test.md` | D | Draft; no run recorded |
| SQSO-TP-JETSON-002 | Test Procedure: Jetson Thermal Test | `docs/test_procedures/TP-JETSON-002_Thermal_Test.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-JETSON-003 | Test Procedure: Jetson Functional Test | `docs/test_procedures/TP-JETSON-003_Functional_Test.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-MECH-001 | Test Procedure: Mechanical Fit Check | `docs/test_procedures/TP-MECH-001_Fit_Check.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-MECH-002 | Test Procedure: Deployable Mechanism Test | `docs/test_procedures/TP-MECH-002_Deployment.md` | 1.0 | Draft; no run recorded |
| SQSO-TP-MECH-003 | Test Procedure: Deployment Switch Test | `docs/test_procedures/TP-MECH-003_Deployment_Switches.md` | 1.0 | Draft; no run recorded |
| TP-OBC-001 | Test Procedure: OBC Functional Test | `docs/test_procedures/TP-OBC-001_Functional_Test.md` | B | Draft; no run recorded |
| SQSO-TP-OBC-002 | Test Procedure: OBC Software Load | `docs/test_procedures/TP-OBC-002_Software_Load.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-OBC-003 | Test Procedure: OBC Timing Verification | `docs/test_procedures/TP-OBC-003_Timing.md` | 1.1 | Draft; no run recorded |
| SQSO-TP-OBC-004 | Test Procedure: Watchdog Verification | `docs/test_procedures/TP-OBC-004_Watchdog.md` | 1.0 | Draft; no run recorded |
| TP-RF-001 | Test Procedure: RF Functional Test | `docs/test_procedures/TP-RF-001_Functional_Test.md` | D | Draft; no run recorded |
| SQSO-TP-RF-002 | Test Procedure: RF Conducted Power | `docs/test_procedures/TP-RF-002_Conducted_Power.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-RF-003 | Test Procedure: Spurious Emissions | `docs/test_procedures/TP-RF-003_Spurious_Emissions.md` | 1.2 | Draft; no run recorded |
| SQSO-TP-RF-004 | Test Procedure: Modulation Quality | `docs/test_procedures/TP-RF-004_Modulation.md` | 1.0 | Draft; no run recorded |
| SQSO-TP-RF-005 | Test Procedure: Antenna Deployment | `docs/test_procedures/TP-RF-005_Antenna_Deployment.md` | 1.1 | Draft; no run recorded |
| TP-RF-006 | Test Procedure: Telecommand TX-OFF at the Antenna Port | `docs/test_procedures/TP-RF-006_TX_OFF.md` | Draft | Draft; no run recorded |
| TP-SYS-001 | Test Procedure: Comprehensive Functional Test | `docs/test_procedures/TP-SYS-001_Comprehensive_Functional.md` | A | Draft; no run recorded |
| TP-SYS-002 | Test Procedure: Day-In-The-Life Test | `docs/test_procedures/TP-SYS-002_Day_In_The_Life.md` | D | Draft; no run recorded |
| SQSO-CAP-001 | Configuration Audit Procedures | `docs/verification/CONFIGURATION_AUDIT_PROCEDURES.md` | 1.1 | Draft |
| SQSO-CVM-001 | SMART-QSO ConOps Validation Matrix | `docs/verification/CONOPS_VALIDATION_MATRIX.md` | 1.0 | Draft |
| SQSO-RTM-001 | SMART-QSO Requirements Traceability Matrix | `docs/verification/REQUIREMENTS_TRACEABILITY_MATRIX.md` | 1.2 | Draft |
| SQSO-VER-RF-001 | RF Test Results Document | `docs/verification/RF_TEST_RESULTS.md` | 1.2 | Template |
| SMART-QSO-TP-001 | SMART-QSO Test Procedures | `docs/verification/TEST_PROCEDURES.md` | 1.1 | Draft |
| SQSO-VCRM-001 | Verification Cross-Reference Matrix (VCRM) | `docs/verification/VERIFICATION_MATRIX.md` | D | Draft |
| SQSO-VIR-001 | Verification Item Register | `docs/verification/VI_REGISTER.md` | A | Generated (`scripts/gen_vi_register.py --check` in CI; RID P3P-01) |
| SQSO-VVP-001 | SMART-QSO Verification and Validation Plan | `docs/verification/VERIFICATION_VALIDATION_PLAN.md` | 1.3 | Draft |
| SQSO-ADCS-ACT-001 | ADCS Actuator Design Document | `hardware/adcs/ACTUATOR_DESIGN.md` | 1.0 | Draft |
| SQSO-ADCS-003 | ADCS Electrical Design Specification | `hardware/adcs/ADCS_ELECTRICAL_DESIGN.md` | 1.0 | Draft |
| SQSO-ADCS-SEN-001 | ADCS Sensor Design Document | `hardware/adcs/SENSOR_DESIGN.md` | 1.1 | Draft |
| SQSO-HW-DIAG-001 | System Diagrams: Block, Wiring and Schematic | `hardware/diagrams/README.md` | C | Draft; preliminary, parts not selected |
| SQSO-EPS-BAT-001 | Battery Design Document | `hardware/eps/BATTERY_DESIGN.md` | 2.10 | Draft |
| SQSO-EPS-003 | EPS Electrical Design Specification | `hardware/eps/EPS_ELECTRICAL_DESIGN.md` | 1.4 | Draft |
| SQSO-EPS-PDU-001 | Power Distribution Unit (PDU) Design Document | `hardware/eps/PDU_DESIGN.md` | 1.5 | Draft |
| SQSO-EPS-SA-001 | Solar Array Design Document | `hardware/eps/SOLAR_ARRAY_DESIGN.md` | 2.2 | Draft |
| SQSO-HAR-001 | Harness Design (wire list, routing, fill, derating, mass) | `hardware/harness/HARNESS_DESIGN.md` | 2.0 | Draft; CDR build-to harness design, not yet built |
| SQSO-CDH-OBC-001 | On-Board Computer (OBC) Design Document | `hardware/obc/OBC_DESIGN.md` | 1.7 | Draft |
| SQSO-OBC-003 | OBC Electrical Design Specification | `hardware/obc/OBC_ELECTRICAL_DESIGN.md` | 1.1 | Draft |
| SQSO-OBC-002 | Storage Design Document | `hardware/obc/STORAGE_DESIGN.md` | 1.7 | Draft |
| SQSO-PAY-JET-001 | Jetson Orin Nano Integration Design Document | `hardware/payload-jetson/INTEGRATION_DESIGN.md` | 1.1 | Draft |
| SQSO-JET-002 | Jetson Payload Power Management | `hardware/payload-jetson/POWER_MANAGEMENT.md` | 1.1 | Draft |
| SQSO-RF-ANT-001 | Antenna Design Document | `hardware/rf/ANTENNA_DESIGN.md` | 2.3 | Draft |
| SQSO-RF-003 | RF Electrical Design Specification | `hardware/rf/RF_ELECTRICAL_DESIGN.md` | 1.3 | Draft |
| SQSO-RF-TX-001 | RF Transmitter Design Document | `hardware/rf/TRANSMITTER_DESIGN.md` | 1.6 | Draft |
| SQSO-STR-001 | Structural Design Document | `hardware/structure/STRUCTURAL_DESIGN.md` | 1.5 | Draft |
| SQSO-MECH-CAD-001 | SMART-QSO Mechanical Design: CAD Model and Structural Finite-Element Model | `hardware/mechanical/MECHANICAL_DESIGN.md` | B | Draft; envelope level |
| SQSO-ICX-001 | SMART-QSO Spacecraft Electrical Interconnect (master pin allocation) | `hardware/electrical/INTERCONNECT.md` | D | Draft; build-to design, not yet fabricated |
| SQSO-RF-BRD-001 | SMART-QSO RF Board: Build-to Design | `hardware/electrical/rf/RF_BOARD_DESIGN.md` | F | Draft; schematic and BOM, not yet fabricated |
| SQSO-EPS-BRD-001 | SMART-QSO EPS Board: Build-to Design | `hardware/electrical/eps/EPS_BOARD_DESIGN.md` | G | Draft; schematic and BOM, not yet fabricated |
| SQSO-OBC-BRD-001 | SMART-QSO OBC Board: Build-to Design | `hardware/electrical/obc/OBC_BOARD_DESIGN.md` | E | Draft; schematic and BOM, not yet fabricated |
| SQSO-JET-BRD-001 | Jetson Orin Nano Carrier Board Design | `hardware/electrical/jetson/CARRIER_BOARD_DESIGN.md` | E | Draft; schematic and BOM, not yet fabricated |
| SQSO-THM-001 | Thermal Design Document | `hardware/structure/THERMAL_DESIGN.md` | 1.7 | Draft |
| SQSO-FSW-AUTO-001 | Autonomy Design | `software/flight/docs/AUTONOMY_DESIGN.md` | B | Draft |
| SQSO-FSW-BOOT-001 | Bootloader Requirements | `software/flight/docs/BOOTLOADER_REQUIREMENTS.md` | B | Draft |
| SQSO-FSW-FDIR-001 | Fault Detection, Isolation, and Recovery Design | `software/flight/docs/FDIR_DESIGN.md` | A | Draft |
| SQSO-FSW-008 | Include Dependencies | `software/flight/docs/INCLUDE_DEPENDENCIES.md` | B | Draft |
| SMART-QSO-JPL-POT-001 | JPL Power of Ten Rules Compliance Audit | `software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` | 1.2 | Draft |
| SQSO-SW-007 | Loop Bounds Documentation | `software/flight/docs/LOOP_BOUNDS.md` | B | Draft |
| SQSO-FSW-006 | Memory Allocation Policy | `software/flight/docs/MEMORY_POLICY.md` | H | Draft |
| SQSO-FSW-010 | MISRA C:2012 Deviation Log | `software/flight/docs/MISRA_DEVIATIONS.md` | C | Draft |
| SQSO-FSW-009 | Naming Convention | `software/flight/docs/NAMING_CONVENTION.md` | B | Draft |
| SQSO-FSW-RAD-001 | Radiation Mitigation Design | `software/flight/docs/RADIATION_MITIGATION.md` | C | Draft |
| SMART-QSO-API-DOC-001 | SMART-QSO Flight Software API Documentation | `software/flight/docs/README.md` | 1.1 | Draft |
| SQSO-FSW-011 | RTOS Architecture | `software/flight/docs/RTOS_ARCHITECTURE.md` | D | Draft |
| SQSO-FSW-TIME-001 | Time Management Design | `software/flight/docs/TIME_MANAGEMENT.md` | B | Draft |
| SQSO-FSW-007 | Timing Analysis | `software/flight/docs/TIMING_ANALYSIS.md` | B | Draft |
| SMART-QSO-HIL-001 | Hardware-in-the-Loop (HIL) Test Plan | `software/flight/tests/hil/HIL_TEST_PLAN.md` | 1.4 | Draft |
| SMART-QSO-REG-001 | Regression Test Suite | `software/flight/tests/regression/REGRESSION_TESTS.md` | 1.1 | Draft |
| SMART-QSO-GND-000 | SMART-QSO Ground Software Tools | `software/ground/tools/README.md` | 1.1 | Draft |

## 3. Test Procedures Defined Inside Documents

Each procedure file in `docs/test_procedures/` is listed in §2 under its document number, written either TP-… or
SQSO-TP-…; both forms name the same procedure (TP-INT-001 is the file whose number is SQSO-TP-INT-001). The procedures below
are sections of a larger document. Each test-procedure ID names exactly one procedure.

| ID | Procedure | Path | Section |
|----|-----------|------|---------|
| TP-001 | Boot and Initialization Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.1 |
| TP-002 | Mode Transition Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.2 |
| TP-003 | Telemetry Generation Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.3 |
| TP-004 | Beacon Generation Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.4 |
| TP-005 | Fault Response Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.5 |
| TP-006 | Power Mode Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.6 |
| TP-007 | Extended Duration Test | `docs/test/FLATSAT_TEST_PROCEDURES.md` | §4.7 |
| TP-UNIT-001 | CRC32 Module Tests | `docs/verification/TEST_PROCEDURES.md` | §3.1 |
| TP-UNIT-002 | EPS Control Module Tests | `docs/verification/TEST_PROCEDURES.md` | §3.2 |
| TP-UNIT-003 | Fault Management Module Tests | `docs/verification/TEST_PROCEDURES.md` | §3.3 |
| TP-UNIT-004 | Sensor Module Tests | `docs/verification/TEST_PROCEDURES.md` | §3.4 |
| TP-UNIT-005 | Mission Data Module Tests | `docs/verification/TEST_PROCEDURES.md` | §3.5 |
| TP-UNIT-006 | UART Communication Module Tests | `docs/verification/TEST_PROCEDURES.md` | §3.6 |
| TP-SWI-001 | EPS + Sensors Integration | `docs/verification/TEST_PROCEDURES.md` | §4.1 |
| TP-SWI-002 | OBC + Jetson Communication | `docs/verification/TEST_PROCEDURES.md` | §4.2 |
| TP-SWI-003 | Power Mode Sequence | `docs/verification/TEST_PROCEDURES.md` | §4.3 |
| TP-SWI-004 | Fault Injection | `docs/verification/TEST_PROCEDURES.md` | §4.4 |
| TP-SWS-001 | End-to-End Beacon Generation | `docs/verification/TEST_PROCEDURES.md` | §5.1 |
| TP-SWS-003 | Performance Timing | `docs/verification/TEST_PROCEDURES.md` | §5.3 |
| TP-ACC-001 | Mission Scenario - Nominal Pass | `docs/verification/TEST_PROCEDURES.md` | §6.1 |
| TP-ACC-002 | Mission Scenario - Eclipse Operations | `docs/verification/TEST_PROCEDURES.md` | §6.2 |

## 4. Changes Made on 2026-10-16

Six numbers were each the header of two files, and three subjects had duplicate documents (findings PI-17 and VV-18).

**Renumbered** (different documents that shared a number):

| Path | Number before | Number now | Kept the old number |
|------|---------------|------------|---------------------|
| `docs/development/RELEASE_PROCEDURE.md` | SQSO-DEV-004 | SQSO-DEV-005 | `docs/development/COMMIT_MESSAGES.md` |
| `docs/integration/TEST_DATA_MANAGEMENT.md` | SQSO-INT-005 | SQSO-INT-006 | `docs/integration/ANOMALY_RESOLUTION.md` |
| `docs/software/BUILD_CONFIGURATIONS.md` | SQSO-SW-007 | SQSO-SW-008 | `software/flight/docs/LOOP_BOUNDS.md` |
| `docs/compliance/LAUNCH_PROVIDER_ICD.md` | SQSO-ICD-004 | SQSO-ICD-010 | `docs/interfaces/ICD_OBC_ADCS.md` |
| `docs/reviews/VERIFICATION_VALIDATION_PACKAGE.md` | SQSO-VVP-001 | SQSO-REV-004 | `docs/verification/VERIFICATION_VALIDATION_PLAN.md` |

**Merged** (duplicates of one document). The superseded files no longer carry a number; each is a short pointer kept so
that existing references resolve.

| Superseded file | Its former number | Merged into |
|-----------------|-------------------|-------------|
| `docs/reviews/SOFTWARE_VERSION_DESCRIPTION.md` | SQSO-SVD-001 | `docs/software/SOFTWARE_VERSION_DESCRIPTION.md` (SQSO-SVD-001 Rev B) |
| `docs/reviews/CONFIGURATION_AUDIT_CHECKLIST.md` | SQSO-CA-001 | `docs/verification/CONFIGURATION_AUDIT_PROCEDURES.md` (SQSO-CAP-001 Version 1.1) |
| `docs/reviews/CONFIGURATION_AUDIT_CHECKLISTS.md` | SQSO-CAC-001 | `docs/verification/CONFIGURATION_AUDIT_PROCEDURES.md` (SQSO-CAP-001 Version 1.1) |
| `docs/hardware/OBC_DESIGN.md` | SMART-QSO-OBC-001 | `hardware/obc/OBC_DESIGN.md` (SQSO-CDH-OBC-001 Version 1.1, Appendix A) |

**Test procedures.** `docs/verification/TEST_PROCEDURES.md` used TP-INT-001 to -004 and TP-SYS-001 to -003 for procedures
other than the files with those IDs; they are now TP-SWI-001 to -004 and TP-SWS-001 and -003 (its §1.4). The ICD master
index and several citations used ICD numbers that belong to other ICDs; they now use the numbers the ICDs carry
(`docs/interfaces/INTERFACE_CONTROL_MASTER.md` §3).

## 5. Known Inconsistencies Not Yet Resolved

- **Two prefixes.** Documents carry SMART-QSO-… or SQSO-… numbers. Some pairs differ only in the prefix:
  SMART-QSO-SDP-001 is the Software Development Plan and SQSO-SDP-001 the Safety Data Package; SMART-QSO-REG-001 is the
  flight regression test suite and SQSO-REG-001 the IARU coordination status; SMART-QSO-TP-001 is the software test
  procedures and SQSO-TP-001 the flatsat test procedures; SMART-QSO-RTM-001 and SQSO-RTM-001 are both traceability
  matrices. One prefix for all documents needs renumbering in `docs/requirements/` and `software/flight/`; decision for
  the PI.
- **Two requirements traceability matrices**: `docs/requirements/REQUIREMENTS_TRACEABILITY.md` (SMART-QSO-RTM-001) and
  `docs/verification/REQUIREMENTS_TRACEABILITY_MATRIX.md` (SQSO-RTM-001). Both now point to the generated
  `docs/verification/RTM.csv` (`scripts/gen_rtm.py`, kept current by CI) as the current traceability record; which
  hand-written matrix to keep is the PI's decision (finding VV-06).
- **Two rollback procedures**: `docs/operations/SOFTWARE_ROLLBACK_PROCEDURES.md` (SQSO-SRP-001) and
  `docs/software/ROLLBACK_RECOVERY_PROCEDURES.md` (SQSO-RRP-001) cover the same subject.
- **Subsystem design pairs.** `docs/hardware/ADCS_DESIGN.md`, `EPS_DESIGN.md`, `RF_DESIGN.md` and `PAYLOAD_DESIGN.md`
  overlap the documents in `hardware/adcs/`, `hardware/eps/`, `hardware/rf/` and `hardware/payload-jetson/`, as the two
  OBC designs did.
- **Numbers cited but carried by no document**: SQSO-SRS-001 (cited in seven documents; the SRS carries
  SMART-QSO-SRS-001), SQSO-EPS-001, SQSO-RF-001, SQSO-OBC-001, SQSO-VVM-001, SQSO-GSD-001, SQSO-HW-OBC-001,
  SQSO-HW-ADCS-001, SQSO-HW-RF-001, SQSO-HW-001, SQSO-HW-004, SQSO-SW-001 and SMART-QSO-RISK-001. SQSO-ICD-011 and
  SQSO-ICD-012 are reserved in the ICD master for interfaces not yet written.
- **Test-procedure IDs cited with no procedure written**: TP-ENV-006, TP-EPS-005 and -006, TP-FLATSAT-002 to -007,
  TP-GSE-001, TP-OBC-005 to -007, TP-PAY-001 to -007, TP-REC-001 and -002, TP-RF-007. Other procedure-ID families
  (E2E-…, TP-E2E-…, STC-…) are not yet checked.

## 6. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-16 | Initial register (findings PI-17, VV-18) | SMART-QSO Team |
| B | 2026-10-16 | Revisions of the documents changed by the traceability work; §5 names the generated RTM as the current record (finding VV-06) | SMART-QSO Team |
