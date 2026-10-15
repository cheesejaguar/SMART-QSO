"""
Tests for the orbital lifetime estimator.

These check physical behaviour (monotonicity, bounds) rather than exact
values; formal lifetimes for filings come from NASA DAS.
"""

from datetime import date

import pytest

pytest.importorskip("pymsis")

from orbit_lifetime import (  # noqa: E402
    DensityTable,
    Spacecraft,
    constant_flux,
    cycle_profile,
    propagate,
)


@pytest.fixture(scope="module")
def sso_table():
    return DensityTable(97.4)


@pytest.fixture(scope="module")
def iss_table():
    return DensityTable(51.6)


ONE_U = Spacecraft(mass_kg=1.33, area_m2=0.01)


def test_density_decreases_with_altitude(sso_table):
    assert sso_table.density(400.0, 120.0) > sso_table.density(500.0, 120.0)


def test_density_increases_with_solar_flux(sso_table):
    assert sso_table.density(500.0, 180.0) > sso_table.density(500.0, 70.0)


def test_density_magnitude_at_500km(sso_table):
    # NRLMSIS orbit-averaged density at 500 km spans roughly 1e-14..1e-11
    # kg/m^3 between solar minimum and maximum.
    for f107 in (70.0, 120.0, 220.0):
        assert 1e-14 < sso_table.density(500.0, f107) < 1e-11


def test_altitude_history_is_monotonic(sso_table):
    res = propagate(sso_table, ONE_U, 450.0, constant_flux(120.0))
    alts = [alt for _, alt in res.history]
    assert all(b <= a for a, b in zip(alts, alts[1:]))
    assert res.reentered


def test_lifetime_increases_with_altitude(sso_table):
    flux = constant_flux(120.0)
    low = propagate(sso_table, ONE_U, 450.0, flux).lifetime_years
    high = propagate(sso_table, ONE_U, 500.0, flux).lifetime_years
    assert high > low


def test_lifetime_decreases_with_drag_area(sso_table):
    flux = constant_flux(120.0)
    small = propagate(sso_table, Spacecraft(1.33, 0.01), 500.0, flux).lifetime_years
    large = propagate(sso_table, Spacecraft(1.33, 0.03), 500.0, flux).lifetime_years
    assert large < small


def test_low_flux_is_bounding_case(sso_table):
    low = propagate(sso_table, ONE_U, 500.0, constant_flux(70.0)).lifetime_years
    nominal = propagate(sso_table, ONE_U, 500.0, cycle_profile).lifetime_years
    assert low > nominal


def test_iss_deployment_lifetime_order_of_magnitude(iss_table):
    # A 1U released from the ISS (~410 km) decays in months to a few years.
    res = propagate(iss_table, ONE_U, 410.0, cycle_profile, date(2027, 6, 1))
    assert res.reentered
    assert 0.2 < res.lifetime_years < 4.0


def test_cycle_profile_bounds():
    for year in range(2020, 2046):
        f107 = cycle_profile(date(year, 1, 1))
        assert 65.0 <= f107 <= 190.0
    assert cycle_profile(date(2030, 7, 1)) < cycle_profile(date(2024, 10, 1))
