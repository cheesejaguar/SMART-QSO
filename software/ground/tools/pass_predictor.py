#!/usr/bin/env python3
"""
SMART-QSO Satellite Pass Predictor

Predicts satellite passes and calculates QSO fairness metrics.
Uses SGP4 propagator for orbit prediction.

Document ID: SMART-QSO-GND-003
Version: 1.0

Requires: sgp4, numpy (optional: matplotlib for plotting)
Install: pip install sgp4 numpy
"""

import argparse
import json
import math
import sys
from dataclasses import dataclass, asdict, field
from datetime import datetime, timezone, timedelta
from typing import List, Dict, Optional, Tuple
from pathlib import Path

try:
    from sgp4.api import Satrec, jday
    from sgp4 import exporter
    SGP4_AVAILABLE = True
except ImportError:
    SGP4_AVAILABLE = False


# Constants
EARTH_RADIUS_KM = 6371.0
DEG_TO_RAD = math.pi / 180.0
RAD_TO_DEG = 180.0 / math.pi
MIN_ELEVATION = 5.0  # Minimum elevation for usable pass


@dataclass
class GroundStation:
    """Ground station location."""
    name: str
    latitude: float  # degrees
    longitude: float  # degrees
    altitude: float = 0.0  # meters above sea level
    grid_square: str = ""


@dataclass
class SatellitePass:
    """Predicted satellite pass."""
    aos_utc: str  # Acquisition of signal
    los_utc: str  # Loss of signal
    max_elevation: float  # Maximum elevation in degrees
    duration_seconds: int
    azimuth_aos: float
    azimuth_los: float


@dataclass
class PassPrediction:
    """Pass prediction results for a station."""
    station: GroundStation
    passes: List[SatellitePass]
    total_access_seconds: int
    average_pass_duration: float


@dataclass
class FairnessMetrics:
    """QSO fairness analysis metrics."""
    station_name: str
    grid_square: str
    total_passes: int
    total_access_time_min: float
    passes_per_day: float
    fairness_score: float  # 0-100, higher is more fair


@dataclass
class FairnessReport:
    """Complete fairness analysis report."""
    analysis_period_days: int
    total_stations: int
    metrics: List[FairnessMetrics]
    gini_coefficient: float
    recommendations: List[str]


def geodetic_to_ecef(lat: float, lon: float, alt_m: float) -> Tuple[float, float, float]:
    """
    Convert geodetic coordinates to ECEF.

    Args:
        lat: Latitude in degrees
        lon: Longitude in degrees
        alt_m: Altitude in meters

    Returns:
        (x, y, z) in kilometers
    """
    lat_rad = lat * DEG_TO_RAD
    lon_rad = lon * DEG_TO_RAD
    alt_km = alt_m / 1000.0

    r = EARTH_RADIUS_KM + alt_km
    x = r * math.cos(lat_rad) * math.cos(lon_rad)
    y = r * math.cos(lat_rad) * math.sin(lon_rad)
    z = r * math.sin(lat_rad)

    return (x, y, z)


def calculate_look_angles(
    station: GroundStation,
    sat_pos_km: Tuple[float, float, float],
    time_jd: float
) -> Tuple[float, float, float]:
    """
    Calculate azimuth, elevation, and range from station to satellite.

    Args:
        station: Ground station
        sat_pos_km: Satellite ECEF position in km
        time_jd: Julian date

    Returns:
        (azimuth_deg, elevation_deg, range_km)
    """
    # Station ECEF
    stn_x, stn_y, stn_z = geodetic_to_ecef(
        station.latitude,
        station.longitude,
        station.altitude
    )

    # Vector from station to satellite
    dx = sat_pos_km[0] - stn_x
    dy = sat_pos_km[1] - stn_y
    dz = sat_pos_km[2] - stn_z

    range_km = math.sqrt(dx*dx + dy*dy + dz*dz)

    # Station-centered coordinates
    lat_rad = station.latitude * DEG_TO_RAD
    lon_rad = station.longitude * DEG_TO_RAD

    sin_lat = math.sin(lat_rad)
    cos_lat = math.cos(lat_rad)
    sin_lon = math.sin(lon_rad)
    cos_lon = math.cos(lon_rad)

    # Topocentric coordinates (South, East, Up)
    south = sin_lat * cos_lon * dx + sin_lat * sin_lon * dy - cos_lat * dz
    east = -sin_lon * dx + cos_lon * dy
    up = cos_lat * cos_lon * dx + cos_lat * sin_lon * dy + sin_lat * dz

    # Azimuth and elevation
    azimuth = math.atan2(east, -south) * RAD_TO_DEG
    if azimuth < 0:
        azimuth += 360.0

    elevation = math.asin(up / range_km) * RAD_TO_DEG

    return (azimuth, elevation, range_km)


def predict_passes_simple(
    station: GroundStation,
    start_time: datetime,
    duration_hours: int = 24,
    min_elevation: float = MIN_ELEVATION,
    orbit_period_min: float = 95.0,
    orbit_altitude_km: float = 400.0
) -> List[SatellitePass]:
    """
    Simple pass prediction without TLE (for demo/testing).

    Generates synthetic passes based on typical LEO orbit.

    Args:
        station: Ground station
        start_time: Start of prediction window
        duration_hours: Prediction duration
        min_elevation: Minimum elevation threshold
        orbit_period_min: Orbital period in minutes
        orbit_altitude_km: Orbital altitude

    Returns:
        List of predicted passes
    """
    passes: List[SatellitePass] = []

    # Typical LEO satellite sees ~4-6 passes per day per station
    # Higher latitude stations see more passes
    lat_factor = abs(station.latitude) / 90.0
    passes_per_day = 4 + 3 * lat_factor

    # Generate synthetic passes
    num_passes = int(passes_per_day * duration_hours / 24.0)
    current_time = start_time

    for i in range(num_passes):
        # Vary pass spacing
        gap_minutes = (24 * 60 / passes_per_day) * (0.8 + 0.4 * ((i * 7) % 10) / 10.0)
        current_time += timedelta(minutes=gap_minutes)

        if current_time > start_time + timedelta(hours=duration_hours):
            break

        # Generate pass characteristics
        max_el = min_elevation + (90 - min_elevation) * ((i * 13) % 10) / 10.0
        duration = int(60 + 600 * max_el / 90.0)  # 1-11 minutes

        aos_az = ((i * 37) % 360)
        los_az = (aos_az + 180 + ((i * 23) % 60) - 30) % 360

        sat_pass = SatellitePass(
            aos_utc=current_time.isoformat(),
            los_utc=(current_time + timedelta(seconds=duration)).isoformat(),
            max_elevation=round(max_el, 1),
            duration_seconds=duration,
            azimuth_aos=round(aos_az, 1),
            azimuth_los=round(los_az, 1)
        )
        passes.append(sat_pass)

    return passes


def predict_passes_sgp4(
    station: GroundStation,
    tle_line1: str,
    tle_line2: str,
    start_time: datetime,
    duration_hours: int = 24,
    min_elevation: float = MIN_ELEVATION,
    step_seconds: int = 30
) -> List[SatellitePass]:
    """
    Predict passes using SGP4 propagator.

    Args:
        station: Ground station
        tle_line1: TLE line 1
        tle_line2: TLE line 2
        start_time: Start of prediction window
        duration_hours: Prediction duration
        min_elevation: Minimum elevation threshold
        step_seconds: Time step for propagation

    Returns:
        List of predicted passes
    """
    if not SGP4_AVAILABLE:
        raise RuntimeError("sgp4 package not available")

    satellite = Satrec.twoline2rv(tle_line1, tle_line2)
    passes: List[SatellitePass] = []

    end_time = start_time + timedelta(hours=duration_hours)
    current_time = start_time

    in_pass = False
    pass_start: Optional[datetime] = None
    pass_start_az = 0.0
    max_el = 0.0
    last_az = 0.0

    while current_time < end_time:
        # Get Julian date
        jd, fr = jday(
            current_time.year,
            current_time.month,
            current_time.day,
            current_time.hour,
            current_time.minute,
            current_time.second + current_time.microsecond / 1e6
        )

        # Propagate
        e, r, v = satellite.sgp4(jd, fr)

        if e != 0:
            current_time += timedelta(seconds=step_seconds)
            continue

        # Calculate look angles
        az, el, rng = calculate_look_angles(station, tuple(r), jd + fr)

        if el >= min_elevation:
            if not in_pass:
                # Pass start
                in_pass = True
                pass_start = current_time
                pass_start_az = az
                max_el = el
            else:
                # In pass
                if el > max_el:
                    max_el = el
            last_az = az
        else:
            if in_pass:
                # Pass end
                in_pass = False
                if pass_start is not None:
                    duration = int((current_time - pass_start).total_seconds())
                    sat_pass = SatellitePass(
                        aos_utc=pass_start.isoformat(),
                        los_utc=current_time.isoformat(),
                        max_elevation=round(max_el, 1),
                        duration_seconds=duration,
                        azimuth_aos=round(pass_start_az, 1),
                        azimuth_los=round(last_az, 1)
                    )
                    passes.append(sat_pass)

        current_time += timedelta(seconds=step_seconds)

    return passes


def calculate_fairness(predictions: List[PassPrediction]) -> FairnessReport:
    """
    Calculate QSO fairness metrics across stations.

    Args:
        predictions: Pass predictions for all stations

    Returns:
        Fairness analysis report
    """
    if not predictions:
        return FairnessReport(
            analysis_period_days=0,
            total_stations=0,
            metrics=[],
            gini_coefficient=0.0,
            recommendations=["No stations provided"]
        )

    # Calculate per-station metrics
    metrics: List[FairnessMetrics] = []
    access_times: List[float] = []

    # Estimate analysis period (assume all predictions cover same period)
    if predictions[0].passes:
        first_pass = datetime.fromisoformat(predictions[0].passes[0].aos_utc.replace('Z', '+00:00'))
        last_pass = datetime.fromisoformat(predictions[0].passes[-1].los_utc.replace('Z', '+00:00'))
        period_days = max(1, (last_pass - first_pass).days)
    else:
        period_days = 1

    for pred in predictions:
        total_access_min = pred.total_access_seconds / 60.0
        passes_per_day = len(pred.passes) / period_days if period_days > 0 else 0

        access_times.append(total_access_min)

        metrics.append(FairnessMetrics(
            station_name=pred.station.name,
            grid_square=pred.station.grid_square,
            total_passes=len(pred.passes),
            total_access_time_min=round(total_access_min, 1),
            passes_per_day=round(passes_per_day, 2),
            fairness_score=0.0  # Calculated below
        ))

    # Calculate Gini coefficient
    if access_times:
        sorted_times = sorted(access_times)
        n = len(sorted_times)
        total = sum(sorted_times)

        if total > 0:
            cumulative = 0.0
            gini_sum = 0.0
            for i, t in enumerate(sorted_times):
                cumulative += t
                gini_sum += (2 * (i + 1) - n - 1) * t
            gini = gini_sum / (n * total)
        else:
            gini = 0.0
    else:
        gini = 0.0

    # Calculate fairness scores (inverse of deviation from mean)
    if access_times:
        mean_access = sum(access_times) / len(access_times)
        max_deviation = max(abs(t - mean_access) for t in access_times) if mean_access > 0 else 1

        for i, m in enumerate(metrics):
            deviation = abs(access_times[i] - mean_access)
            fairness = 100 * (1 - deviation / max_deviation) if max_deviation > 0 else 100
            metrics[i] = FairnessMetrics(
                station_name=m.station_name,
                grid_square=m.grid_square,
                total_passes=m.total_passes,
                total_access_time_min=m.total_access_time_min,
                passes_per_day=m.passes_per_day,
                fairness_score=round(fairness, 1)
            )

    # Generate recommendations
    recommendations: List[str] = []

    if gini > 0.3:
        recommendations.append(
            f"High inequality detected (Gini={gini:.2f}). "
            "Consider prioritizing stations with lower access times."
        )

    low_access = [m for m in metrics if m.total_access_time_min < 10]
    if low_access:
        stations = ", ".join(m.station_name for m in low_access)
        recommendations.append(
            f"Stations with limited access (<10 min): {stations}"
        )

    if gini < 0.1:
        recommendations.append("Access distribution is relatively fair.")

    return FairnessReport(
        analysis_period_days=period_days,
        total_stations=len(predictions),
        metrics=metrics,
        gini_coefficient=round(gini, 3),
        recommendations=recommendations
    )


def load_stations(filepath: Path) -> List[GroundStation]:
    """
    Load ground stations from JSON file.

    Args:
        filepath: Path to stations JSON file

    Returns:
        List of ground stations
    """
    with open(filepath, "r") as f:
        data = json.load(f)

    stations: List[GroundStation] = []
    for s in data.get("stations", []):
        stations.append(GroundStation(
            name=s.get("name", "Unknown"),
            latitude=s.get("latitude", 0.0),
            longitude=s.get("longitude", 0.0),
            altitude=s.get("altitude", 0.0),
            grid_square=s.get("grid_square", "")
        ))

    return stations


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="SMART-QSO Satellite Pass Predictor and Fairness Analyzer",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --lat 37.4 --lon -122.0 --duration 48
  %(prog)s --stations stations.json --fairness
  %(prog)s --lat 37.4 --lon -122.0 --tle iss.tle
        """
    )

    parser.add_argument("--lat", type=float, help="Station latitude (degrees)")
    parser.add_argument("--lon", type=float, help="Station longitude (degrees)")
    parser.add_argument("--alt", type=float, default=0, help="Station altitude (meters)")
    parser.add_argument("--name", default="Station", help="Station name")
    parser.add_argument("--grid", default="", help="Grid square")

    parser.add_argument("--stations", help="JSON file with multiple stations")
    parser.add_argument("--tle", help="TLE file for accurate propagation")

    parser.add_argument("--duration", type=int, default=24, help="Prediction duration (hours)")
    parser.add_argument("--min-elevation", type=float, default=MIN_ELEVATION,
                       help="Minimum elevation (degrees)")

    parser.add_argument("--fairness", action="store_true", help="Calculate fairness metrics")
    parser.add_argument("-o", "--output", help="Output JSON file")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    # Build station list
    stations: List[GroundStation] = []

    if args.stations:
        stations = load_stations(Path(args.stations))
    elif args.lat is not None and args.lon is not None:
        stations.append(GroundStation(
            name=args.name,
            latitude=args.lat,
            longitude=args.lon,
            altitude=args.alt,
            grid_square=args.grid
        ))
    else:
        parser.print_help()
        return 1

    if args.verbose:
        print(f"Loaded {len(stations)} station(s)")

    # Load TLE if provided
    tle1, tle2 = None, None
    if args.tle:
        try:
            with open(args.tle, "r") as f:
                lines = [l.strip() for l in f.readlines() if l.strip()]
                if len(lines) >= 2:
                    tle1 = lines[0] if lines[0].startswith("1 ") else lines[1]
                    tle2 = lines[1] if lines[1].startswith("2 ") else lines[2] if len(lines) > 2 else None
        except IOError as e:
            print(f"Warning: Could not load TLE: {e}", file=sys.stderr)

    # Predict passes
    start_time = datetime.now(timezone.utc)
    predictions: List[PassPrediction] = []

    for station in stations:
        if tle1 and tle2 and SGP4_AVAILABLE:
            passes = predict_passes_sgp4(
                station, tle1, tle2, start_time,
                args.duration, args.min_elevation
            )
        else:
            passes = predict_passes_simple(
                station, start_time, args.duration, args.min_elevation
            )

        total_access = sum(p.duration_seconds for p in passes)
        avg_duration = total_access / len(passes) if passes else 0

        predictions.append(PassPrediction(
            station=station,
            passes=passes,
            total_access_seconds=total_access,
            average_pass_duration=round(avg_duration, 1)
        ))

        if args.verbose:
            print(f"Station {station.name}: {len(passes)} passes, "
                  f"{total_access/60:.1f} min total access")

    # Calculate fairness if requested
    fairness_report: Optional[FairnessReport] = None
    if args.fairness and len(predictions) > 1:
        fairness_report = calculate_fairness(predictions)

    # Build output
    output = {
        "prediction_time_utc": start_time.isoformat(),
        "duration_hours": args.duration,
        "min_elevation": args.min_elevation,
        "stations": []
    }

    for pred in predictions:
        station_data = {
            "station": asdict(pred.station),
            "passes": [asdict(p) for p in pred.passes],
            "total_access_seconds": pred.total_access_seconds,
            "average_pass_duration": pred.average_pass_duration
        }
        output["stations"].append(station_data)

    if fairness_report:
        output["fairness"] = asdict(fairness_report)

    # Output
    json_str = json.dumps(output, indent=2)

    if args.output:
        with open(args.output, "w") as f:
            f.write(json_str)
        if args.verbose:
            print(f"Output written to {args.output}")
    else:
        print(json_str)

    return 0


if __name__ == "__main__":
    sys.exit(main())
