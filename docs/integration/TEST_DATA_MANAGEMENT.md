# Test Data Management Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-INT-005
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NPR 7123.1, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the management, storage, and archival requirements for all test data generated during SMART-QSO verification and validation activities.

## 2. Scope

This document covers:
- Test data collection
- Data storage and backup
- Data retention and archival
- Data access and control
- Traceability requirements

---

## 3. Test Data Categories

### 3.1 Data Classification

| Category | Description | Retention |
|----------|-------------|-----------|
| Level 1 | Flight qualification data | Permanent |
| Level 2 | Acceptance test data | Mission life + 5 years |
| Level 3 | Development test data | 3 years |
| Level 4 | Debug/troubleshooting | 1 year |

### 3.2 Test Data Types

| Type | Format | Example |
|------|--------|---------|
| Telemetry | CSV, binary | Housekeeping packets |
| Environmental | CSV, proprietary | Vibe profiles, thermal logs |
| RF | S-parameter, IQ | Spectrum analyzer captures |
| Images | JPEG, PNG | Visual inspection photos |
| Video | MP4 | Deployment recordings |
| Logs | Text, JSON | Console output, event logs |
| Reports | PDF, Markdown | Test reports |

---

## 4. Data Collection

### 4.1 Collection Requirements

| Requirement | Implementation |
|-------------|----------------|
| Timestamping | UTC, millisecond resolution |
| Unique ID | Test run ID + timestamp |
| Metadata | Operator, configuration, environment |
| Checksums | SHA-256 for all raw data |

### 4.2 Naming Convention

```
<TEST_ID>_<DATE>_<TIME>_<TYPE>.<EXT>

Examples:
TP-EPS-001_20260115_143022_telemetry.csv
TP-ENV-002_20260120_091500_vibration.dat
TP-SYS-001_20260125_160000_report.pdf
```

### 4.3 Metadata Requirements

```json
{
    "test_id": "TP-EPS-001",
    "run_id": "TP-EPS-001-R003",
    "date": "2026-01-15",
    "time_utc": "14:30:22",
    "operator": "J. Smith",
    "witness": "A. Jones",
    "configuration": {
        "sw_version": "1.2.0",
        "hw_revision": "C",
        "serial_number": "FM-001"
    },
    "environment": {
        "temperature_c": 23.5,
        "humidity_pct": 45,
        "pressure_mbar": 1013
    },
    "equipment": [
        {"name": "Power Supply", "model": "Keysight E36312A", "cal_date": "2025-06-15"},
        {"name": "DMM", "model": "Keysight 34465A", "cal_date": "2025-08-20"}
    ],
    "checksum_sha256": "a1b2c3d4..."
}
```

---

## 5. Data Storage

### 5.1 Storage Architecture

```
test_data/
├── raw/                    # Original unprocessed data
│   ├── TP-EPS-001/
│   ├── TP-ENV-002/
│   └── ...
├── processed/              # Analyzed/reduced data
│   ├── TP-EPS-001/
│   └── ...
├── reports/                # Test reports
│   ├── unit_test/
│   ├── integration/
│   └── environmental/
├── media/                  # Photos and video
│   ├── inspections/
│   └── deployments/
└── archive/                # Compressed archives
    ├── 2026-Q1/
    └── ...
```

### 5.2 Storage Requirements

| Tier | Medium | Capacity | Purpose |
|------|--------|----------|---------|
| Primary | SSD/NAS | 2 TB | Active testing |
| Secondary | RAID array | 10 TB | Project archive |
| Offsite | Cloud/tape | 10 TB | Disaster recovery |

### 5.3 Backup Schedule

| Data Type | Backup Frequency | Retention |
|-----------|------------------|-----------|
| Raw test data | Daily (incremental) | Until archived |
| Processed data | Weekly | 2 years |
| Reports | On completion | Permanent |
| Full archive | Monthly | Permanent |

---

## 6. Data Integrity

### 6.1 Integrity Checks

| Check | Frequency | Method |
|-------|-----------|--------|
| File checksum | On write | SHA-256 |
| Backup verification | Weekly | Restore test |
| Archive integrity | Quarterly | Full scan |

### 6.2 Checksum Verification

```bash
#!/bin/bash
# verify_checksums.sh
find raw/ -name "*.csv" -exec sha256sum {} \; > checksums_current.txt
diff checksums_baseline.txt checksums_current.txt
```

### 6.3 Chain of Custody

| Event | Recorded Information |
|-------|---------------------|
| Data creation | Timestamp, operator, test ID |
| Data access | Timestamp, user, purpose |
| Data modification | Timestamp, user, change description |
| Data transfer | Source, destination, method |
| Data deletion | Authorization, reason |

---

## 7. Access Control

### 7.1 Access Levels

| Role | Read Raw | Write Raw | Read Reports | Modify Reports |
|------|----------|-----------|--------------|----------------|
| Test Engineer | Yes | Yes | Yes | Yes |
| Systems Engineer | Yes | No | Yes | Yes |
| QA | Yes | No | Yes | No |
| Project Manager | Yes | No | Yes | No |
| External | No | No | Limited | No |

### 7.2 Data Protection

| Control | Implementation |
|---------|----------------|
| Authentication | Project credentials |
| Authorization | Role-based access |
| Encryption | AES-256 for sensitive data |
| Audit logging | All access recorded |

---

## 8. Data Retention

### 8.1 Retention Schedule

| Data Category | Active | Archive | Total |
|---------------|--------|---------|-------|
| Flight qualification | Mission life | +10 years | Life+10 |
| Acceptance test | Mission life | +5 years | Life+5 |
| Development test | 1 year | +2 years | 3 years |
| Debug data | 6 months | +6 months | 1 year |

### 8.2 Archival Process

1. Verify data completeness
2. Generate checksums
3. Compress with metadata
4. Transfer to archive storage
5. Verify archive integrity
6. Update archive index
7. Remove from active storage (optional)

### 8.3 Archive Format

```
SQSO_TestData_YYYY-QN.tar.gz
├── manifest.json           # Archive contents
├── checksums.sha256        # All file checksums
├── raw/                    # Raw data files
├── processed/              # Processed data
└── reports/                # Test reports
```

---

## 9. Traceability

### 9.1 Requirement Traceability

| Link | Source | Destination |
|------|--------|-------------|
| Test → Requirement | Test ID | Requirement ID |
| Data → Test | Run ID | Test ID |
| Report → Data | Report | Data files |
| Anomaly → Data | AR number | Test run |

### 9.2 Traceability Matrix Update

| Event | Update Required |
|-------|-----------------|
| Test complete | Link test data to requirements |
| Anomaly found | Link anomaly to test data |
| Report issued | Link report to data and requirements |

---

## 10. Quality Assurance

### 10.1 Data Quality Checks

| Check | Frequency | Criteria |
|-------|-----------|----------|
| Completeness | Each test | All required data collected |
| Accuracy | Each test | Within measurement uncertainty |
| Consistency | Each test | No conflicts with prior data |
| Timeliness | Each test | Logged within 24 hours |

### 10.2 Audit Trail

| Event | Recorded |
|-------|----------|
| Test execution | Date, time, operator, configuration |
| Data collection | Files, sizes, checksums |
| Data processing | Tools, versions, parameters |
| Report generation | Author, reviewers, approval |

---

## 11. Tools and Automation

### 11.1 Data Collection Tools

| Tool | Purpose | Format |
|------|---------|--------|
| GSE Software | Telemetry capture | Binary, CSV |
| LabVIEW | Environmental monitoring | TDMS, CSV |
| Spectrum Analyzer | RF measurements | CSV, S2P |
| Oscilloscope | Timing analysis | CSV, WFM |

### 11.2 Automation Scripts

```bash
# collect_test_data.sh - Automated data collection
#!/bin/bash
TEST_ID="$1"
RUN_ID="${TEST_ID}-R$(date +%Y%m%d%H%M%S)"
OUTPUT_DIR="raw/${TEST_ID}/${RUN_ID}"

mkdir -p "$OUTPUT_DIR"
./capture_telemetry.sh "$OUTPUT_DIR"
./generate_metadata.sh "$OUTPUT_DIR" "$TEST_ID"
sha256sum "$OUTPUT_DIR"/* > "$OUTPUT_DIR/checksums.sha256"
```

---

## 12. Anomaly Data

### 12.1 Anomaly Data Requirements

| Requirement | Description |
|-------------|-------------|
| Preservation | All data from anomalous tests preserved |
| Isolation | Anomaly data tagged and segregated |
| Analysis | Root cause analysis data linked |
| Resolution | Corrective action data linked |

### 12.2 Anomaly Package

```
AR-001/
├── original_test_data/     # All data from failed test
├── analysis/               # Investigation data
├── root_cause/             # RCA documentation
├── retest_data/            # Verification retest data
└── closure/                # Closure documentation
```

---

## 13. Deliverables

### 13.1 Standard Deliverables

| Deliverable | Format | Recipient |
|-------------|--------|-----------|
| Test Report | PDF | Project archive |
| Raw Data | Original format | Project archive |
| Summary Data | CSV/Excel | Project team |
| Trend Analysis | Charts/plots | Systems engineering |

### 13.2 External Deliverables

| Deliverable | Recipient | Format |
|-------------|-----------|--------|
| Environmental Test Report | Launch provider | PDF |
| RF Test Data | Frequency coordinator | PDF |
| Qualification Summary | Range safety | PDF |

---

## 14. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
