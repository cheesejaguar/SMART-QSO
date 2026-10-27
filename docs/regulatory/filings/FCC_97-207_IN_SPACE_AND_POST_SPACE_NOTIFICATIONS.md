# FCC Part 97 In-Space and Post-Space Notifications (Templates)
## SMART-QSO Amateur Space Station

**Document Number:** SQSO-REG-014
**Revision:** A.1
**Date:** 2026-10-27
**Status:** Templates. Complete and file at the times shown. Fields in `[brackets]` are fill-ins.
**Rule:** 47 CFR 97.207(g)(2) and (g)(3) (eCFR version of 2026-10-01, unchanged). Planned dates: in-space by **2028-06-08** for a 2028-06-01 launch; post-space by **2030-09-01** for an end of transmissions on 2030-06-01 (REGULATORY_ROADMAP Rev B §3)

| Notification | Deadline (rule text) |
|--------------|----------------------|
| In-space | "no later than 7 days following initiation of space station transmissions. This notification must update the information contained in the pre-space notification." |
| Post-space | "no later than 3 months after termination of the space station transmissions. When termination of transmissions is ordered by the FCC, the notification is required no later than 24 hours after termination of transmissions." |

**Filing channel:** written notification to the Space Bureau, FCC, Washington, DC 20554 (97.207(g)). DA 13-445 (2013)
also accepted email copies at amsat.spacecap@fcc.gov. **Confirm the current submission channel with Space Bureau
staff before the pre-space filing**, and use the same channel for these follow-ups.

---

## A. In-Space Station Notification — 47 CFR 97.207(g)(2)

> **To:** Space Bureau, Federal Communications Commission
> **Re:** In-space notification, amateur space station SMART-QSO (KK6GDB)
> **Date:** `[≤ 7 days after first transmission]`
>
> 1. **Licensee:** `[name]`, KK6GDB, `[license class]`, `[address, phone, email]`
> 2. **Reference:** Pre-space notification dated `[date]` (and replacement dated `[date]`, if any); IARU coordination letter `[reference]`; ITU API/A special section `[number]`.
> 3. **Launch:** `[launch vehicle / mission name]`, launched `[UTC date/time]` from `[site]`; deployed `[UTC date/time]` by `[dispenser]`.
> 4. **Initiation of transmissions:** first space-station transmission at `[UTC date/time]` (from onboard log / first ground reception report).
> 5. **Orbit as achieved:** NORAD catalog `[number]`, international designator `[YYYY-NNNX]`; apogee `[ ]` km, perigee `[ ]` km, inclination `[ ]`°, period `[ ]` min (epoch `[ ]`, source: Space-Track/TraCSS).
> 6. **Changes from the pre-space notification:** `[none / list each: frequency, power, emission, orbit, debris disclosures]`. Updated Appendix 4 data attached if any technical parameter changed.
> 7. **Status:** `[e.g., beacon nominal at 145.xxx MHz; transponder not yet enabled; telecommand link verified from stations [call signs]]`.
> 8. **Telecommand stations** able to effect cessation of emissions under 97.207(b): `[call signs and locations]`.
> 9. **Space-safety registration:** registered with `[18 SDS (Space-Track) / TraCSS]` on `[date]`; conjunction-alert contact `[24 h phone/email]`.
>
> Signed: `[licensee]`

**Evidence to file alongside (internal):** first-pass telemetry archive, SatNOGS observation IDs, as-run commissioning
procedure (`docs/operations/COMMISSIONING_PLAN.md`).

---

## B. Post-Space Station Notification — 47 CFR 97.207(g)(3)

> **To:** Space Bureau, Federal Communications Commission
> **Re:** Post-space notification, amateur space station SMART-QSO (KK6GDB), NORAD `[number]`
> **Date:** `[≤ 3 months after termination; ≤ 24 hours if termination was ordered by the FCC]`
>
> 1. **Licensee:** `[name]`, KK6GDB
> 2. **Termination of transmissions:** `[UTC date/time]`; method `[telecommand TX-OFF and passivation / loss of power / re-entry]` (autonomous end-of-life passivation is proposed, not implemented; do not cite it unless it is built and was the cause); confirmed by `[ground observation, SatNOGS observation IDs]`.
> 3. **Reason:** `[end of mission / anomaly / FCC order dated ...]`
> 4. **Passivation status** (compare pre-space disclosure 97.207(g)(1)(iii)): battery charge path `[permanently disabled at UTC ...]`; RF transmitters `[disabled]`; residual stored energy `[description]`.
> 5. **Orbit at termination:** apogee `[ ]` km, perigee `[ ]` km; predicted re-entry `[date range]` (DAS / Space-Track decay prediction).
> 6. **Disposal compliance:** completion of mission `[date]`; predicted re-entry no later than `[date]`, which is `[≤ 5 years]` after mission completion as required by 97.207(g)(1)(vii)(D)(1). `[If re-entry has occurred: TIP/decay date from Space-Track]`.
>
> Signed: `[licensee]`

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial templates (funding audit) |
| A.1 | 2026-10-27 | CDR review (communications panel): rule currency, planned dates, termination methods limited to what is implemented |
