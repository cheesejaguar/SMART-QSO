# Regulatory (Public, Non‑binding Summary)

- Amateur‑Satellite Service; **IARU satellite frequency coordination** required.
- All transmissions **in the clear**; no encryption or obscuration. Callsign embedded in each beacon per AX.25 frame body and address fields.
- De‑orbit plan ≤ 5 years post‑EOL; passive drag/tether device sized for ~500 km SSO.

## Amateur‑Satellite Service Compliance (Mission Constraints)
- Purpose aligns with amateur and technical self‑training; no commercial or pecuniary interest.
- Transmit‑only beacon on 2 m using AX.25 UI frames at 1200 bps AFSK via Stensat beacon module (see `docs/BEACON_SPEC.md`).
- Identification present each transmission; content is plain‑text status/telemetry; no encryption.
- Command/telemetry, if any, are in the clear and limited to safety and station functions compliant with amateur rules.

## IARU Satellite Frequency Coordination — Step‑by‑Step
The following process tracks typical IARU Amateur‑Satellite coordination. Timelines are indicative; start early (≥9–12 months before launch).

1. Pre‑Coordination Prep (T‑12 to T‑9 months)
   - Confirm operation under Amateur‑Satellite Service and educational/technical aims.
   - Freeze initial RF plan (see `docs/RF_PLAN.md`): transmit‑only downlink on 145.8–146.0 MHz; beacon only.
   - Define emissions: 1200 bps AFSK AX.25 UI; narrowband FM deviation consistent with 2 m channelization. Draft emission designator (to be confirmed by test).
   - Document plain‑text operations (no encryption), callsign identification, duty cycle, and maximum EIRP.

2. Application Dossier (T‑9 to T‑6 months)
   - Mission description (1U CubeSat, objectives, fail‑graceful ops).
   - Orbit: 475–525 km SSO, expected lifetime, de‑orbit plan.
   - RF details: frequency, bandwidth, modulation, bit rate, emission designator (provisional), max TX power, antenna type (2 m quarter‑wave tape monopole), polarization, EIRP, duty cycle.
   - Beacon spec: AX.25 UI, 1200 bps AFSK, frame contents, callsign, cadence (60–120 s adaptive).
   - Ground segment: typical club station reception; no uplink planned.
   - Contact information for responsible amateur organization/station and spacecraft integrator.

3. Submit Coordination Request
   - Complete the IARU Amateur‑Satellite Frequency Coordination form (online) with the dossier above.
   - Attach/link detailed RF plan, beacon framing, and de‑orbit statement.

4. Review & Iteration (Panel Feedback)
   - Respond to questions on sharing, interference avoidance, and duty cycle.
   - Adjust frequency/channel, bandwidth, or beacon cadence as requested to fit regional band plans.

5. Coordination Letter Issued
   - Receive coordination letter specifying authorized frequency(ies)/conditions.
   - Update internal documentation and test plans to match the coordinated parameters.

6. National Administration & ITU Steps (in parallel/to follow local law)
   - Engage your national administration for spacecraft licensing/authorization as required.
   - Coordinate any necessary ITU filings through the administration (e.g., API/A, coordination notices) per national process.

7. Pre‑Launch Notification (T‑1 to T‑2 months)
   - Provide updated frequency, beacon parameters, and tentative TLEs/launch info to IARU panel if changed.
   - Confirm that final RF tests match the coordinated emissions and EIRP limits.

8. Post‑Launch Conduct
   - Operate strictly per coordination letter; verify on‑air spectrum occupancy, deviation, and cadence.
   - Publish beacon frames and logs; promptly report anomalies and any out‑of‑family behavior.
   - Notify IARU and national administration of commissioning status and final orbital elements.

Notes:
- Keep all beacons plain‑text and ensure callsign identification in each frame.
- Maintain the ability to immediately reduce duty‑cycle or cease transmissions if harmful interference is reported.
- Treat any software‑defined updates as configuration only; no encryption or obscuration in over‑the‑air content.
