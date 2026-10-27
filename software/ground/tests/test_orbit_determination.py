"""
Orbit determination and operator ephemeris from GNSS fixes (SQSO-GND-002;
RID LV-04, VI-LV-13).

The end-to-end cases fly a simulated SMART-QSO: 475 km sun-synchronous,
LTAN 22:30 (BASELINE §3), integrated with a richer force model than the fit
uses (gravity to degree 20, Sun and Moon, a diurnal density bulge, drag 1.4
times the nominal). The receiver's fixes get noise (2.5 m x PDOP 2 per axis),
are written as GNSS state frames, decoded, fitted, and the products (CCSDS
OEM, TLE) are checked against the truth within the accuracy stated in
SQSO-GND-002 §6.
"""

import json
import math
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
import pytest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "tools"))

import orbit_frames as fr  # noqa: E402
from ephemeris_products import _exp_field, format_tle, read_oem  # noqa: E402
from ephemeris_tool import EXIT_FAIL, EXIT_OK, EXIT_POOR, run  # noqa: E402
from gnss_frame import (GnssFix, fix_time_unix, format_gnss_frame, gps_to_unix, parse_gnss_frame,  # noqa: E402
                        read_archive, resolve_week, unix_to_gps)
from gnss_simulation import (Truth, kaula_coefficients, simulate_fixes, sso_inclination, sso_state,  # noqa: E402
                             write_icgem)
from orbit_dynamics import EGM96_4X4, ForceModel, GravityField, _dp5, propagate  # noqa: E402

EPOCH = datetime(2028, 11, 2, 0, 0, tzinfo=timezone.utc).timestamp()
ORBIT_S = 95 * 60
TOOLS = Path(__file__).resolve().parents[1] / "tools"


# --- GNSS state frame ---------------------------------------------------------

class TestGnssFrame:
    FRAME = "GNS N=12 WK=1022 TOW=345600000 X=-1033480 Y=5901295 Z=3380357 VX=-3225637 VY=-2872451 VZ=5531924 Q=1 SV=7 PD=18"

    def test_parse(self):
        fix = parse_gnss_frame(self.FRAME)
        assert fix.week == 1022 and fix.tow_ms == 345600000
        assert fix.r_ecef_m == (-1033480, 5901295, 3380357)
        assert fix.v_ecef_ms == pytest.approx((-3225.637, -2872.451, 5531.924))
        assert (fix.quality, fix.sats, fix.pdop, fix.seq) == (1, 7, 1.8, 12)
        assert fix.problems() == []

    def test_round_trip(self):
        fix = parse_gnss_frame(self.FRAME)
        assert parse_gnss_frame(format_gnss_frame(fix)) == fix

    def test_not_measured_and_unknown_fields(self):
        fix = parse_gnss_frame("GNS WK=2300 TOW=1000 X=7000000 Y=0 Z=0 VX=- VY=- VZ=- Q=1 SV=- PD=- NEW=5")
        assert fix.v_ecef_ms is None and fix.sats is None and fix.pdop is None

    def test_other_frames_ignored(self):
        assert parse_gnss_frame("HK1 N=7 UP=3600") is None
        assert parse_gnss_frame("de KK6GDB-11: Hello | T=1 M=IDLE") is None

    def test_missing_position_refused(self):
        with pytest.raises(ValueError):
            parse_gnss_frame("GNS WK=2300 TOW=1000 X=7000000 Y=0")
        with pytest.raises(ValueError):
            parse_gnss_frame("GNS WK=2300 TOW=1000 X=7e6 Y=abc Z=0")

    def test_dead_reckoning_and_implausible_refused(self):
        dr = parse_gnss_frame("GNS WK=2300 TOW=1000 X=7000000 Y=0 Z=0 Q=6 SV=7 PD=20")
        assert "not a real fix" in dr.problems()[0]
        assert parse_gnss_frame("GNS WK=2300 TOW=1000 X=6000000 Y=0 Z=0 Q=1 SV=3 PD=20").problems() == [
            "3 satellites", "radius 6000000 m outside LEO"]

    def test_week_rollover(self):
        # 2028-11-02 is GPS week 2547, reported by a 10-bit receiver as 499
        week, tow = unix_to_gps(EPOCH)
        assert week == 2547
        assert resolve_week(499) == 2547
        assert resolve_week(499, near_unix=EPOCH + 3 * 86400) == 2547
        assert resolve_week(2547) == 2547

    def test_gps_utc(self):
        # GPS is 18 s ahead of UTC (TAI - UTC = 37 s)
        week, tow = unix_to_gps(EPOCH)
        assert gps_to_unix(week, tow) == pytest.approx(EPOCH)
        assert (week * 604800 + tow) - (EPOCH - 315964800) == pytest.approx(18.0)

    def test_archive_formats(self, tmp_path):
        sqso = pytest.importorskip("sqso_protocol")
        info = self.FRAME.replace("WK=1022", "WK=342")
        frame = sqso.ax25_ui_frame("CQ", "KK6GDB-11", info.encode(), True)
        other = sqso.ax25_ui_frame("CQ", "N0CALL", info.encode(), True)
        archive = tmp_path / "pass.jsonl"
        archive.write_text("\n".join([
            json.dumps({"rx_utc": "2028-11-02T00:10:00Z", "station": "A", "frame_hex": frame.hex()}),
            json.dumps({"rx_utc": "2028-11-02T00:10:00Z", "station": "B", "info": info}),
            json.dumps({"station": "C", "frame_hex": other.hex()}),
            json.dumps({"info": "HK1 N=1 UP=5"}),
            frame[:-2].hex(),
            "GNS WK=342 TOW=oops X=1 Y=2 Z=3",
            "# comment",
        ]) + "\n")
        received, errors = read_archive([archive])
        assert len(received) == 3  # with FCS, info field, without FCS; not N0CALL, not HK1
        assert all(r.fix.r_ecef_m == (-1033480, 5901295, 3380357) for r in received)
        assert received[0].station == "A" and received[0].rx_unix == pytest.approx(EPOCH + 600)
        assert len(errors) == 1 and "TOW" in errors[0]


# --- Frames and time -----------------------------------------------------------

class TestFrames:
    """Vallado, Fundamentals of Astrodynamics (4th ed.), Example 3-15."""

    T = datetime(2004, 4, 6, 7, 51, 28, 386009, tzinfo=timezone.utc).timestamp()
    R_ITRF = np.array([-1033.4793830, 7901.2952754, 6380.3565958]) * 1e3
    V_ITRF = np.array([-3.225636520, -2.872451450, 5.531924446]) * 1e3
    EOP = dict(dut1_s=-0.4399619, xp_arcsec=-0.140682, yp_arcsec=0.333309)

    def test_itrf_to_teme(self):
        r, v = fr.itrf_to_teme(self.T, self.R_ITRF, self.V_ITRF, **self.EOP)
        assert r == pytest.approx(np.array([5094.18016210, 6127.64465950, 6380.34453270]) * 1e3, abs=0.01)
        assert v == pytest.approx(np.array([-4.746131487, 0.785818041, 5.531931288]) * 1e3, abs=1e-5)

    def test_teme_to_j2000(self):
        r = fr.itrf_to_teme(self.T, self.R_ITRF, **self.EOP)
        # Vallado's J2000 (IAU-76/FK5 without the EOP nutation corrections):
        # the truncated nutation agrees to well under a metre
        j2000 = fr.teme_to_eme2000_matrix(self.T, 32 + 32.184) @ r
        assert j2000 == pytest.approx(np.array([5102.5096, 6123.01152, 6378.1363]) * 1e3, abs=1.0)

    def test_round_trip(self):
        r, v = fr.itrf_to_teme(self.T, self.R_ITRF, self.V_ITRF, **self.EOP)
        rb, vb = fr.teme_to_itrf(self.T, r, v, **self.EOP)
        assert rb == pytest.approx(self.R_ITRF, abs=1e-6) and vb == pytest.approx(self.V_ITRF, abs=1e-9)

    def test_utc_teme_rereferenced_to_ut1(self):
        r_utc = fr.itrf_to_teme(self.T, self.R_ITRF, xp_arcsec=-0.140682, yp_arcsec=0.333309)
        r = fr.teme_utc_to_ut1(self.T, self.EOP["dut1_s"]) @ r_utc
        assert r == pytest.approx(np.array([5094.18016210, 6127.64465950, 6380.34453270]) * 1e3, abs=0.01)

    def test_jacobian_maps_velocity(self):
        r, v = fr.itrf_to_teme(self.T, self.R_ITRF, self.V_ITRF)
        jac = fr.teme_to_itrf_jacobian(self.T)
        assert jac @ np.concatenate([r, v]) == pytest.approx(np.concatenate([self.R_ITRF, self.V_ITRF]), abs=1e-6)


# --- Dynamics ------------------------------------------------------------------

class TestDynamics:
    R = np.array([[4.0e6, -3.0e6, 4.5e6], [6.8e6, 1.0e5, -2.0e5]])

    def test_zonal_closed_form_matches_recursion(self):
        zonal = GravityField.builtin(4, 0)
        general = GravityField(zonal.gm, zonal.radius, zonal.c, zonal.s)
        general.__class__ = type("General", (GravityField,), {"zonal_only": property(lambda self: False)})
        assert zonal.accel_ecef(self.R) == pytest.approx(general.accel_ecef(self.R), rel=1e-12)

    def test_sectoral_term_matches_closed_form(self):
        c, s = EGM96_4X4[(2, 2)]
        field = GravityField.from_normalized({(2, 2): (c, s)}, 2)
        field.c[0, 0] = 0.0
        field = GravityField(field.gm, field.radius, field.c, field.s)
        c22, s22 = field.c[2, 2], field.s[2, 2]
        x, y, rn = self.R[:, :1], self.R[:, 1:2], np.linalg.norm(self.R, axis=1, keepdims=True)
        f = c22 * (x * x - y * y) + 2 * s22 * x * y
        grad = np.concatenate([2 * c22 * x + 2 * s22 * y, -2 * c22 * y + 2 * s22 * x, 0 * x], 1)
        expected = 3 * field.gm * field.radius ** 2 * (grad / rn ** 5 - 5 * f * self.R / rn ** 7)
        assert field.accel_ecef(self.R) == pytest.approx(expected, rel=1e-9)

    def test_icgem_file(self, tmp_path):
        path = tmp_path / "field.gfc"
        write_icgem(path, kaula_coefficients(8), 8)
        field = GravityField.from_icgem(path, 8)
        assert field.degree == 8 and field.c[2, 0] == pytest.approx(-1.08263e-3, rel=1e-5)
        assert GravityField.from_icgem(path, 3).degree == 3

    def test_integrator_against_runge_kutta_reference(self):
        # Two hours, ABM at 60 s against Dormand-Prince at 2 s
        model = ForceModel()
        x0 = sso_state(EPOCH)
        y = propagate(model, EPOCH, x0[None], [1.0], [EPOCH + 7200.0])[0, 0]
        ref = _dp5(model, EPOCH, x0[None], np.ones(1), 7200.0, 3600)[0]
        assert np.linalg.norm(y[:3] - ref[:3]) < 0.01
        back = propagate(model, EPOCH + 7200.0, y[None], [1.0], [EPOCH + 1234.5, EPOCH])[0]
        assert np.linalg.norm(back[1, :3] - x0[:3]) < 0.01
        mid = _dp5(model, EPOCH, x0[None], np.ones(1), 1234.5, 617)[0]
        assert np.linalg.norm(back[0, :3] - mid[:3]) < 0.01

    def test_sun_synchronous_orbit(self):
        a = 6378137.0 + 475e3
        # 97.31 deg at 475 km (97.40 deg is the 500 km value)
        assert math.degrees(sso_inclination(a)) == pytest.approx(97.31, abs=0.01)


# --- End to end ---------------------------------------------------------------

def _frames_archive(path: Path, fixes, rx_offset_s=60.0, extra=()):
    lines = []
    for f in fixes:
        t = fix_time_unix(f, near_unix=EPOCH)
        rx = datetime.fromtimestamp(t + rx_offset_s, tz=timezone.utc).isoformat()
        lines.append(json.dumps({"rx_utc": rx, "station": "KK6GDB", "info": format_gnss_frame(f)}))
    lines += list(extra)
    path.write_text("\n".join(lines) + "\n")
    return path


@pytest.fixture(scope="module")
def truth():
    return Truth(EPOCH, sso_state(EPOCH))


@pytest.fixture(scope="module")
def truth_grid(truth):
    """Truth TEME states every minute over 4 days."""
    times = EPOCH + np.arange(0, 4 * 86400 + 1, 60.0)
    return times, truth.states(times)


def _truth_at(truth_grid, times):
    grid_t, states = truth_grid
    idx = np.searchsorted(grid_t, times)
    assert np.allclose(grid_t[idx], times), "check times on the one-minute truth grid"
    return states[idx]


def _realistic(err, sigma):
    """Covariance realism: within 3 sigma at 99 % of the epochs, 4 sigma at all."""
    ratio = err / sigma
    return np.mean(ratio <= 3.0) >= 0.99 and np.all(ratio <= 4.0)


def _errors(oem, truth_grid):
    """3D position error (m) of an ITRF OEM against the truth, and its 1-sigma,
    at the OEM's covariance epochs."""
    times = oem.cov_times
    states = oem.states[np.isin(oem.times, times)]
    tr = _truth_at(truth_grid, times)
    err = np.linalg.norm(states[:, :3] * 1000.0 - fr.teme_to_itrf(times, tr[:, :3]), axis=1)
    sigma = np.sqrt(np.trace(oem.covariances[:, :3, :3], axis1=1, axis2=2)) * 1000.0
    return times, err, sigma


@pytest.fixture(scope="module")
def leop(truth, tmp_path_factory):
    tmp = tmp_path_factory.mktemp("leop")
    times = EPOCH + np.arange(0, 24 * 3600, ORBIT_S)
    fixes = simulate_fixes(truth, times, sigma_v=0.5, rng=np.random.default_rng(7))
    archive = _frames_archive(tmp / "leop.jsonl", fixes)
    gfc = tmp / "field.gfc"
    write_icgem(gfc, kaula_coefficients(), 12)
    code = run(["--frames", str(archive), "--out-dir", str(tmp / "out"), "--gravity-file", str(gfc),
                "--gravity-degree", "12", "--object-id", "2028-999A", "--span-h", "72", "--cov-every", "10"])
    files = {p.suffix: p for p in (tmp / "out").iterdir()}
    return code, files, max(times)


class TestLeopEveryOrbit:
    """Launch and early operations: a fix every orbit for 24 h (the frame sent
    after each session's fix), gravity to 12 x 12 from an ICGEM file (as with
    EGM2008 in operations). Stated accuracy (SQSO-GND-002 §6): 3D error
    <= 1.5 km at the last fix + 24 h and <= 8 km at + 72 h, error within
    3 sigma at 99 % of the epochs (4 sigma at all), quality GOOD."""

    def test_quality_good(self, leop):
        code, files, _ = leop
        report = json.loads(files[".json"].read_text())
        assert code == EXIT_OK and report["quality"] == "GOOD", report["quality_reasons"]
        assert report["fixes_used"] == 16 and report["sessions"] == 16 and not report["rejected"]
        assert report["drag_scale"] == pytest.approx(1.4, abs=0.15)

    def test_oem_accuracy(self, leop, truth_grid):
        _, files, last = leop
        oem = read_oem(files[".oem"])
        times, err, sigma = _errors(oem, truth_grid)
        at = lambda h: err[np.argmin(np.abs(times - (last + h * 3600)))]  # noqa: E731
        assert at(24) < 1500.0 and at(72) < 8000.0
        assert _realistic(err, sigma)

    def test_oem_format(self, leop):
        _, files, last = leop
        oem = read_oem(files[".oem"])
        assert oem.header["CCSDS_OEM_VERS"] == "3.0" and oem.header["ORIGINATOR"] == "KK6GDB"
        assert oem.meta["REF_FRAME"] == "ITRF2000" and oem.meta["TIME_SYSTEM"] == "UTC"
        assert oem.meta["OBJECT_ID"] == "2028-999A" and oem.meta["CENTER_NAME"] == "EARTH"
        assert len(oem.times) == 72 * 60 + 1 and np.all(np.diff(oem.times) == 60.0)
        assert oem.times[0] >= last and len(oem.cov_times) == 433
        # Positive semi-definite to the precision written (along-track position
        # and velocity are almost fully correlated after a day)
        for c in oem.covariances:
            d = np.sqrt(np.diag(c))
            assert np.all(d > 0) and np.linalg.eigvalsh(c / np.outer(d, d)).min() > -1e-6
        assert "quality GOOD" in files[".oem"].read_text()

    def test_tle(self, leop, truth_grid):
        sgp4 = pytest.importorskip("sgp4.api")
        _, files, last = leop
        lines = files[".tle"].read_text().splitlines()
        sat = sgp4.Satrec.twoline2rv(lines[1], lines[2])
        assert sat.satnum == 99999
        target = math.ceil((last + 86400) / 60) * 60
        jd, frac = divmod(target / 86400.0 + fr.JD_UNIX_EPOCH, 1.0)
        err, r, _ = sat.sgp4(jd, frac)
        truth_r = _truth_at(truth_grid, [target])[0, :3]
        assert err == 0 and np.linalg.norm(np.array(r) * 1000 - truth_r) < 3000.0


@pytest.fixture(scope="module")
def sparse(truth, tmp_path_factory):
    tmp = tmp_path_factory.mktemp("sparse")
    starts = [0, ORBIT_S, 12 * 3600, 12 * 3600 + ORBIT_S]
    times = EPOCH + np.array([s + d for s in starts for d in (0, 10, 20)], float)
    fixes = simulate_fixes(truth, times, sigma_v=None, rng=np.random.default_rng(11))
    bad = fixes[4]
    outlier = GnssFix(bad.week, bad.tow_ms + 5000, tuple(np.array(bad.r_ecef_m) + 2000.0), None, 1, 7, 2.0)
    dr = GnssFix(bad.week, bad.tow_ms + 7000, bad.r_ecef_m, None, 6, 7, 2.0)
    extra = [json.dumps({"station": "W1AW", "info": format_gnss_frame(f)}) for f in (fixes[0], outlier, dr)]
    archive = _frames_archive(tmp / "sparse.jsonl", fixes, extra=extra)
    out = tmp / "out"
    code = run(["--frames", str(archive), "--out-dir", str(out), "--span-h", "48", "--cov-every", "10",
                "--no-tle"])
    files = {p.suffix: p for p in out.iterdir()}
    return code, files, max(times), archive, tmp


class TestFirstEphemerisSparse:
    """The 24 h deadline with only the licensee's station: GNSS_REQUEST on
    four passes in two pass groups 12 h apart, three records each, no
    velocity, the built-in 4 x 4 field; the archive also holds a duplicate,
    a dead-reckoned record and an outlier. Stated accuracy: 3D error
    <= 5 km at + 24 h, covariance realistic as above, quality GOOD or FAIR."""

    def test_quality_and_editing(self, sparse):
        code, files, _, _, _ = sparse
        report = json.loads(files[".json"].read_text())
        assert code == EXIT_OK and report["quality"] in ("GOOD", "FAIR"), report["quality_reasons"]
        assert report["fixes_used"] == 12 and report["sessions"] == 4
        reasons = [r["reason"] for r in report["rejected"]]
        assert len(reasons) == 2
        assert any("not a real fix" in r for r in reasons) and any("sigma" in r for r in reasons)

    def test_oem_accuracy(self, sparse, truth_grid):
        _, files, last, _, _ = sparse
        times, err, sigma = _errors(read_oem(files[".oem"]), truth_grid)
        at24 = np.argmin(np.abs(times - (last + 86400)))
        assert err[at24] < 5000.0
        assert _realistic(err, sigma)

    def test_eme2000_needs_dut1(self, sparse):
        _, _, _, archive, tmp = sparse
        base = ["--frames", str(archive), "--span-h", "3", "--no-tle", "--frame", "EME2000"]
        assert run(base + ["--out-dir", str(tmp / "e1")]) == EXIT_FAIL
        assert run(base + ["--out-dir", str(tmp / "e2"), "--dut1", "0.05"]) == EXIT_OK
        oem = read_oem(next((tmp / "e2").glob("*.oem")))
        assert oem.meta["REF_FRAME"] == "EME2000"
        assert np.linalg.norm(oem.states[0, :3]) == pytest.approx(6378.137 + 475, abs=40)


class TestSingleSession:
    """One 30 s session (one pass, no velocity): the drag is not known, the
    covariance says so, the quality is POOR and the products are marked
    NOT-FOR-UPLOAD; the covariance stays realistic."""

    def test_poor_and_honest(self, truth, truth_grid, tmp_path):
        times = EPOCH + 600.0 + np.arange(0, 30, 1.0)
        archive = _frames_archive(tmp_path / "one.jsonl",
                                  simulate_fixes(truth, times, sigma_v=None, rng=np.random.default_rng(3)))
        out = tmp_path / "out"
        assert run(["--frames", str(archive), "--out-dir", str(out), "--span-h", "24", "--no-tle"]) == EXIT_POOR
        oem_path = next(out.glob("*.oem"))
        assert oem_path.name.startswith("NOT-FOR-UPLOAD_")
        times, err, sigma = _errors(read_oem(oem_path), truth_grid)
        assert _realistic(err, sigma)
        assert sigma[-1] > 20_000.0


class TestRefusals:
    def test_too_few_fixes(self, truth, tmp_path):
        archive = _frames_archive(tmp_path / "two.jsonl",
                                  simulate_fixes(truth, EPOCH + np.array([0.0, 10.0]), sigma_v=None))
        out = tmp_path / "out"
        assert run(["--frames", str(archive), "--out-dir", str(out)]) == EXIT_FAIL
        assert not out.exists() or not any(out.iterdir())

    def test_no_initial_state(self, truth, tmp_path):
        # One position-only fix per orbit: no velocity and no session pair
        archive = _frames_archive(tmp_path / "sparse.jsonl",
                                  simulate_fixes(truth, EPOCH + np.arange(0, 4) * ORBIT_S, sigma_v=None))
        assert run(["--frames", str(archive), "--out-dir", str(tmp_path / "out")]) == EXIT_FAIL


class TestTleFormat:
    def test_exponent_field(self):
        assert _exp_field(0.0) == " 00000+0"
        assert _exp_field(1.0905e-3) == " 10905-2"
        assert _exp_field(-1.2345e-5) == "-12345-4"
        assert _exp_field(0.999996) == " 10000+1"

    def test_lines_and_checksums(self):
        sgp4 = pytest.importorskip("sgp4.api")
        l1, l2 = format_tle(99999, "28999A", EPOCH, 0.0, 1.0905e-3, 97.3078, 15.1787, 0.0003482, 328.2722,
                            61.7932, 15.30824893)
        assert len(l1) == 69 and len(l2) == 69
        assert l1.startswith("1 99999U 28999A   28307.00000000") and l2.startswith("2 99999  97.3078")
        sat = sgp4.Satrec.twoline2rv(l1, l2)
        assert sat.bstar == pytest.approx(1.0905e-3) and sat.ecco == pytest.approx(0.0003482)
        assert math.degrees(sat.inclo) == pytest.approx(97.3078)
