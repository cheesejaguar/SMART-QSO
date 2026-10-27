#!/usr/bin/env python3
"""
SMART-QSO structural finite-element model (SQSO-MECH-CAD-001 section 8; results in
docs/environment/STRUCTURAL_ANALYSIS.md Version B).

A beam-shell idealisation of the stowed spacecraft, built from the parameters of the CAD model (model.py):
  * shells (flat 4-node MITC4 plate + Wilson incompatible-mode membrane + drilling stiffness): end frames,
    side shear panels, the four PC/104 boards, the torquer deck, the heat spreader and its flanges, the stowed wings;
  * 3D Euler-Bernoulli beams: the four rails (L-angle 8.5 x 8.5 x 1.5 with solid standoffs), the stack columns
    (hex spacer + A286 stud, bending stiffness x 1/3 for the threaded joints as in SQSO-SA-001 A section 5), the
    centre supports, the battery isolators (G-10CR bushing + Ti-6Al-4V screw), the clamp ears, the wing hinges
    and the hold-down post;
  * the battery as a rigid body (mass and inertia from the CAD solids) on its four ears;
  * every other MEL mass as lumped or non-structural mass where the CAD puts it (total = MEL CBE).
Boundary condition as in the dispenser: rail nodes fixed laterally (x, y) over the rail length, axially (z) at the
rail ends; rotations free.  Sensitivity: rails held at the eight rail ends only.

Analyses: normal modes with effective masses; quasi-static design limit load (SpaceX UG v11 Table 4-1:
10 g axial with 17 g lateral, 16 combined directions); random vibration at MPE + 3 dB (UG v11 Table 4-6 + 3 dB,
7.86 Grms) by modal superposition, Q = max(10, sqrt(f)); margins with NASA-STD-5001B protoflight factors.

Usage:  python3 hardware/mechanical/fem.py [--selftest] [--json out.json]
Needs numpy and scipy (and model.py's CadQuery environment for the battery inertia and the part list).
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Sequence, Tuple

for _v in ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS"):
    os.environ.setdefault(_v, "1")
import numpy as np  # noqa: E402
import scipy.sparse as sp
import scipy.sparse.linalg as spla

HERE = os.path.dirname(os.path.abspath(__file__))

# ======================================================================================================
# Materials (SI).  Allowables: MMPDS-style A-basis as in SQSO-SA-001 A section 2 (re-check at drawing release).
# ======================================================================================================
MAT = {
    "Al6061": dict(E=68.9e9, nu=0.33, rho=2700.0, Fty=241e6, Ftu=290e6),
    "FR4": dict(E=18.6e9, nu=0.13, rho=1900.0, Fty=None, Ftu=340e6),
    "A286": dict(E=201e9, nu=0.30, rho=7940.0, Fty=758e6, Ftu=1103e6),
    "Ti64": dict(E=114e9, nu=0.34, rho=4430.0, Fty=827e6, Ftu=896e6),
    "G10CR": dict(E=17e9, nu=0.12, rho=1850.0, Fty=None, Ftu=380e6),
    "silicone": dict(E=3e6, nu=0.45, rho=1100.0, Fty=None, Ftu=None),
}
G0 = 9.80665


# ======================================================================================================
# Model container
# ======================================================================================================
class Model:
    def __init__(self):
        self.xyz: List[np.ndarray] = []
        self.key: Dict[Tuple[int, int, int], int] = {}
        self.shells: List[dict] = []
        self.beams: List[dict] = []
        self.springs: List[dict] = []
        self.lumped: Dict[int, float] = {}
        self.rigid: List[dict] = []          # {master, mass6 (6x6)}
        self.mpc: Dict[int, int] = {}        # slave node -> master node (rigid link)
        self.fixed: Dict[int, List[int]] = {}
        self.tags: Dict[str, List[int]] = {}  # element groups (shell indices or beam indices, by name)

    # nodes in metres; coordinates rounded to 1 micrometre for merging
    def node(self, x, y, z) -> int:
        k = (int(round(x * 1e6)), int(round(y * 1e6)), int(round(z * 1e6)))
        if k not in self.key:
            self.key[k] = len(self.xyz)
            self.xyz.append(np.array([x, y, z], float))
        return self.key[k]

    def nn(self):
        return len(self.xyz)

    def add_mass(self, n, m):
        self.lumped[n] = self.lumped.get(n, 0.0) + m


# ======================================================================================================
# Shell element: MITC4 plate + Wilson Q6 membrane (incompatible modes, condensed) + drilling
# ======================================================================================================
GP = np.array([-1.0, 1.0]) / math.sqrt(3.0)
NAT = np.array([[-1, -1], [1, -1], [1, 1], [-1, 1]], float)


def shape(xi, eta):
    N = 0.25 * (1 + NAT[:, 0] * xi) * (1 + NAT[:, 1] * eta)
    dN = np.vstack([0.25 * NAT[:, 0] * (1 + NAT[:, 1] * eta), 0.25 * NAT[:, 1] * (1 + NAT[:, 0] * xi)])
    return N, dN                                   # dN: 2 x 4 (d/dxi, d/deta)


def shell_frame(X: np.ndarray):
    e1 = X[1] - X[0]
    e1 /= np.linalg.norm(e1)
    n = np.cross(X[1] - X[0], X[3] - X[0])
    e3 = n / np.linalg.norm(n)
    e2 = np.cross(e3, e1)
    R = np.vstack([e1, e2, e3])
    loc = (X - X.mean(axis=0)) @ R.T
    return R, loc[:, :2]


def shell_matrices(X: np.ndarray, t: float, mat: dict, nsm: float = 0.0):
    """Element stiffness (24x24, global), lumped mass (24), and recovery data.  DOF per node: ux uy uz rx ry rz."""
    E, nu, rho = mat["E"], mat["nu"], mat["rho"]
    R, xy = shell_frame(X)
    Dm = E * t / (1 - nu * nu) * np.array([[1, nu, 0], [nu, 1, 0], [0, 0, (1 - nu) / 2]])
    Db = E * t ** 3 / (12 * (1 - nu * nu)) * np.array([[1, nu, 0], [nu, 1, 0], [0, 0, (1 - nu) / 2]])
    Ds = 5.0 / 6.0 * E / (2 * (1 + nu)) * t * np.eye(2)
    # local dof layout per node: u v w tx ty tz -> index 6*i + k
    Kuu = np.zeros((8, 8)); Kua = np.zeros((8, 4)); Kaa = np.zeros((4, 4))
    Kb = np.zeros((12, 12)); Ks = np.zeros((12, 12))
    area = 0.0
    # Jacobian at the centre (for the Wilson modes, Taylor's correction: evaluate at the centre)
    _, dN0 = shape(0, 0)
    J0 = dN0 @ xy
    detJ0 = np.linalg.det(J0)
    J0i = np.linalg.inv(J0)

    # MITC4 tying-point covariant shear B rows (w, bx, by per node -> plate dofs w, tx, ty with bx=ty, by=-tx)
    def cov_shear_rows(xi, eta):
        N, dN = shape(xi, eta)
        J = dN @ xy
        rxi = np.zeros(12); reta = np.zeros(12)
        for i in range(4):
            # e_xiz = dw/dxi + x_xi*bx + y_xi*by
            rxi[3 * i + 0] = dN[0, i]
            rxi[3 * i + 2] += J[0, 0] * N[i]          # bx = ty  (plate dof 2)
            rxi[3 * i + 1] += -J[0, 1] * N[i]         # by = -tx (plate dof 1)
            reta[3 * i + 0] = dN[1, i]
            reta[3 * i + 2] += J[1, 0] * N[i]
            reta[3 * i + 1] += -J[1, 1] * N[i]
        return rxi, reta
    rB = cov_shear_rows(0, -1)[0]
    rD = cov_shear_rows(0, 1)[0]
    rA = cov_shear_rows(-1, 0)[1]
    rC = cov_shear_rows(1, 0)[1]
    for xi in GP:
        for eta in GP:
            N, dN = shape(xi, eta)
            J = dN @ xy
            detJ = np.linalg.det(J)
            Ji = np.linalg.inv(J)
            dNx = Ji @ dN                            # 2 x 4 (d/dx, d/dy)
            area += detJ
            Bm = np.zeros((3, 8))
            for i in range(4):
                Bm[0, 2 * i] = dNx[0, i]
                Bm[1, 2 * i + 1] = dNx[1, i]
                Bm[2, 2 * i] = dNx[1, i]
                Bm[2, 2 * i + 1] = dNx[0, i]
            # Wilson incompatible modes P1 = 1 - xi^2, P2 = 1 - eta^2 (Taylor: centre Jacobian)
            dP = np.array([[-2 * xi, 0.0], [0.0, -2 * eta]])     # rows: d/dxi, d/deta ; cols P1, P2
            dPx = J0i @ dP * (detJ0 / detJ)
            Ba = np.zeros((3, 4))
            for k in range(2):
                Ba[0, 2 * k] = dPx[0, k]
                Ba[1, 2 * k + 1] = dPx[1, k]
                Ba[2, 2 * k] = dPx[1, k]
                Ba[2, 2 * k + 1] = dPx[0, k]
            Kuu += Bm.T @ Dm @ Bm * detJ
            Kua += Bm.T @ Dm @ Ba * detJ
            Kaa += Ba.T @ Dm @ Ba * detJ
            # bending: kappa = [d bx/dx, d by/dy, d bx/dy + d by/dx], bx = ty, by = -tx
            Bb = np.zeros((3, 12))
            for i in range(4):
                Bb[0, 3 * i + 2] = dNx[0, i]
                Bb[1, 3 * i + 1] = -dNx[1, i]
                Bb[2, 3 * i + 2] = dNx[1, i]
                Bb[2, 3 * i + 1] = -dNx[0, i]
            Kb += Bb.T @ Db @ Bb * detJ
            # MITC4 shear: e_xiz linear in eta between B (eta=-1) and D (eta=+1); e_etaz linear in xi (A, C)
            exi = 0.5 * (1 - eta) * rB + 0.5 * (1 + eta) * rD
            eeta = 0.5 * (1 - xi) * rA + 0.5 * (1 + xi) * rC
            Bs = Ji @ np.vstack([exi, eeta])
            Ks += Bs.T @ Ds @ Bs * detJ
    Kc = Kuu - Kua @ np.linalg.solve(Kaa, Kua.T)
    Kl = np.zeros((24, 24))
    im = [6 * i + k for i in range(4) for k in (0, 1)]
    ip = [6 * i + k for i in range(4) for k in (2, 3, 4)]
    Kl[np.ix_(im, im)] += Kc
    Kl[np.ix_(ip, ip)] += Kb + Ks
    kd = 1e-4 * E * t * area
    idr = [6 * i + 5 for i in range(4)]
    Kl[np.ix_(idr, idr)] += kd * (np.eye(4) - 0.25)
    T = np.zeros((24, 24))
    for i in range(8):
        T[3 * i:3 * i + 3, 3 * i:3 * i + 3] = R
    Kg = T.T @ Kl @ T
    m = (rho * t + nsm) * area / 4.0
    Ir = (rho * t ** 3 / 12.0 + nsm * t * t / 12.0) * area / 4.0
    M = np.zeros(24)
    for i in range(4):
        M[6 * i:6 * i + 3] = m
        M[6 * i + 3:6 * i + 6] = max(Ir, 1e-12)
    rec = dict(R=R, xy=xy, Dm=Dm, Db=Db, Kua=Kua, Kaa=Kaa, t=t, E=E, nu=nu)
    return Kg, M, area, rec


def shell_stress(rec, ue_global: np.ndarray):
    """von Mises at the element centre, top and bottom surfaces (Pa), and membrane/bending components."""
    R, xy = rec["R"], rec["xy"]
    ul = np.zeros(24)
    for i in range(8):
        ul[3 * i:3 * i + 3] = R @ ue_global[3 * i:3 * i + 3]
    um = np.array([ul[6 * i + k] for i in range(4) for k in (0, 1)])
    up = np.array([ul[6 * i + k] for i in range(4) for k in (2, 3, 4)])
    alpha = -np.linalg.solve(rec["Kaa"], rec["Kua"].T @ um)
    N, dN = shape(0, 0)
    J = dN @ xy
    dNx = np.linalg.inv(J) @ dN
    eps = np.zeros(3); kap = np.zeros(3)
    for i in range(4):
        eps += np.array([dNx[0, i] * um[2 * i], dNx[1, i] * um[2 * i + 1],
                         dNx[1, i] * um[2 * i] + dNx[0, i] * um[2 * i + 1]])
        kap += np.array([dNx[0, i] * up[3 * i + 2], -dNx[1, i] * up[3 * i + 1],
                         dNx[1, i] * up[3 * i + 2] - dNx[0, i] * up[3 * i + 1]])
    # incompatible modes vanish in the derivative at the centre (dP = 0 at xi = eta = 0): no contribution
    E, nu, t = rec["E"], rec["nu"], rec["t"]
    C = E / (1 - nu * nu) * np.array([[1, nu, 0], [nu, 1, 0], [0, 0, (1 - nu) / 2]])
    sm = C @ eps
    sb = C @ kap * t / 2
    out = []
    for sg in (1, -1):
        s = sm + sg * sb
        out.append(math.sqrt(s[0] ** 2 - s[0] * s[1] + s[1] ** 2 + 3 * s[2] ** 2))
    return max(out), sm, sb


def shell_stress_lin(rec):
    """Linear map from the 24 global element dofs to [sx, sy, sxy] at the top and bottom surfaces (6 x 24)."""
    R, xy = rec["R"], rec["xy"]
    T = np.zeros((24, 24))
    for i in range(8):
        T[3 * i:3 * i + 3, 3 * i:3 * i + 3] = R
    N, dN = shape(0, 0)
    dNx = np.linalg.inv(dN @ xy) @ dN
    Be = np.zeros((3, 24)); Bk = np.zeros((3, 24))
    for i in range(4):
        Be[0, 6 * i + 0] = dNx[0, i]; Be[1, 6 * i + 1] = dNx[1, i]
        Be[2, 6 * i + 0] = dNx[1, i]; Be[2, 6 * i + 1] = dNx[0, i]
        Bk[0, 6 * i + 4] = dNx[0, i]; Bk[1, 6 * i + 3] = -dNx[1, i]
        Bk[2, 6 * i + 4] = dNx[1, i]; Bk[2, 6 * i + 3] = -dNx[0, i]
    E, nu, t = rec["E"], rec["nu"], rec["t"]
    C = E / (1 - nu * nu) * np.array([[1, nu, 0], [nu, 1, 0], [0, 0, (1 - nu) / 2]])
    top = C @ (Be + Bk * t / 2) @ T
    bot = C @ (Be - Bk * t / 2) @ T
    return np.vstack([top, bot])


# ======================================================================================================
# Beam element (3D Euler-Bernoulli, consistent mass)
# ======================================================================================================
def beam_frame(x1, x2, vref):
    ex = x2 - x1
    L = np.linalg.norm(ex)
    ex = ex / L
    v = np.array(vref, float)
    if abs(np.dot(v, ex)) > 0.99:
        v = np.array([0.0, 0.0, 1.0]) if abs(ex[2]) < 0.9 else np.array([1.0, 0.0, 0.0])
    ez = np.cross(ex, v)
    ez /= np.linalg.norm(ez)
    ey = np.cross(ez, ex)
    return L, np.vstack([ex, ey, ez])


def beam_matrices(x1, x2, sec: dict, vref=(0, 0, 1)):
    E, G = sec["E"], sec["G"]
    A, Iy, Iz, J = sec["A"], sec["Iy"], sec["Iz"], sec["J"]
    L, R = beam_frame(x1, x2, vref)
    k = np.zeros((12, 12))
    EA = E * A / L
    GJ = G * J / L
    k[0, 0] = k[6, 6] = EA; k[0, 6] = k[6, 0] = -EA
    k[3, 3] = k[9, 9] = GJ; k[3, 9] = k[9, 3] = -GJ
    for (iv, ith, I, sgn) in ((1, 5, Iz, 1), (2, 4, Iy, -1)):
        a = 12 * E * I / L ** 3; b = 6 * E * I / L ** 2; c = 4 * E * I / L; d = 2 * E * I / L
        jv, jth = iv + 6, ith + 6
        k[iv, iv] += a; k[jv, jv] += a; k[iv, jv] -= a; k[jv, iv] -= a
        k[iv, ith] += sgn * b; k[ith, iv] += sgn * b; k[iv, jth] += sgn * b; k[jth, iv] += sgn * b
        k[jv, ith] -= sgn * b; k[ith, jv] -= sgn * b; k[jv, jth] -= sgn * b; k[jth, jv] -= sgn * b
        k[ith, ith] += c; k[jth, jth] += c; k[ith, jth] += d; k[jth, ith] += d
    m = sec.get("rho", 0.0) * A * L + sec.get("nsm", 0.0) * L
    Ml = np.zeros((12, 12))
    if m > 0:
        # consistent mass (translations), lumped rotary
        mt = m / 420.0 * np.array([[156, 22 * L, 54, -13 * L], [22 * L, 4 * L * L, 13 * L, -3 * L * L],
                                   [54, 13 * L, 156, -22 * L], [-13 * L, -3 * L * L, -22 * L, 4 * L * L]])
        for (iv, ith, sgn) in ((1, 5, 1), (2, 4, -1)):
            idx = [iv, ith, iv + 6, ith + 6]
            S = np.diag([1, sgn, 1, sgn])
            Ml[np.ix_(idx, idx)] += S @ mt @ S
        Ml[0, 0] += m / 3; Ml[6, 6] += m / 3; Ml[0, 6] += m / 6; Ml[6, 0] += m / 6
        ip = (Iy + Iz) / max(A, 1e-30) * m / 2.0
        Ml[3, 3] += ip; Ml[9, 9] += ip
    T = np.zeros((12, 12))
    for i in range(4):
        T[3 * i:3 * i + 3, 3 * i:3 * i + 3] = R
    return T.T @ k @ T, T.T @ Ml @ T, L, R, k, T


def beam_section_round(d, di=0.0):
    A = math.pi / 4 * (d * d - di * di)
    I = math.pi / 64 * (d ** 4 - di ** 4)
    return dict(A=A, Iy=I, Iz=I, J=2 * I, cy=d / 2, cz=d / 2)


def beam_section_rect(b, h):
    A = b * h
    Iy = b * h ** 3 / 12          # about local y (bending in z): h along z
    Iz = h * b ** 3 / 12
    a, c = max(b, h) / 2, min(b, h) / 2
    J = a * c ** 3 * (16 / 3 - 3.36 * c / a * (1 - c ** 4 / (12 * a ** 4)))
    return dict(A=A, Iy=Iy, Iz=Iz, J=J, cy=b / 2, cz=h / 2)


def beam_section_L(a, t):
    """Equal-leg angle a x a x t: properties about the centroidal axes parallel to the legs (local y, z) --
    the product of inertia is ignored (principal-axis rotation of an equal angle; documented idealisation);
    extreme-fibre distances for the stress check."""
    rects = [(0.0, a, 0.0, t), (0.0, t, t, a)]         # (y0, y1, z0, z1)
    A = sum((y1 - y0) * (z1 - z0) for y0, y1, z0, z1 in rects)
    yc = sum((y1 - y0) * (z1 - z0) * (y0 + y1) / 2 for y0, y1, z0, z1 in rects) / A
    zc = sum((y1 - y0) * (z1 - z0) * (z0 + z1) / 2 for y0, y1, z0, z1 in rects) / A
    Iy = sum((y1 - y0) * (z1 - z0) ** 3 / 12 + (y1 - y0) * (z1 - z0) * ((z0 + z1) / 2 - zc) ** 2
             for y0, y1, z0, z1 in rects)
    Iz = sum((z1 - z0) * (y1 - y0) ** 3 / 12 + (y1 - y0) * (z1 - z0) * ((y0 + y1) / 2 - yc) ** 2
             for y0, y1, z0, z1 in rects)
    Iyz = sum((y1 - y0) * (z1 - z0) * ((y0 + y1) / 2 - yc) * ((z0 + z1) / 2 - zc) for y0, y1, z0, z1 in rects)
    Imin = (Iy + Iz) / 2 - math.hypot((Iy - Iz) / 2, Iyz)
    J = 2 * a * t ** 3 / 3
    return dict(A=A, Iy=Imin, Iz=Imin, J=J, cy=a - yc, cz=a - zc, centroid=(yc, zc), Imax=(Iy + Iz) / 2 +
                math.hypot((Iy - Iz) / 2, Iyz))


def beam_stress_lin(b: dict):
    """Linear map from the 12 global dofs to the peak normal stress components at both ends:
    rows [N/A, My*c/Iy, Mz*c/Iz] at end 1 and end 2, and torsion shear (8 x 12)."""
    k, T, sec = b["kloc"], b["T"], b["sec"]
    F = k @ T                                         # local end forces from global displacements
    A, Iy, Iz, J = sec["A"], sec["Iy"], sec["Iz"], sec["J"]
    cy, cz = sec["cy"], sec["cz"]
    rows = []
    for e in (0, 6):
        rows.append(F[e + 0] / A)
        rows.append(F[e + 4] * cz / Iy)
        rows.append(F[e + 5] * cy / Iz)
        rows.append(F[e + 3] * max(cy, cz) / J)
    return np.vstack(rows)


def beam_peak(sv: np.ndarray):
    """Peak normal stress from the 8 components [N, My, Mz, T] x 2 ends (absolute sum, conservative)."""
    s1 = abs(sv[0]) + abs(sv[1]) + abs(sv[2])
    s2 = abs(sv[4]) + abs(sv[5]) + abs(sv[6])
    return max(s1, s2), max(abs(sv[3]), abs(sv[7]))


# ======================================================================================================
# Assembly, constraints, solution
# ======================================================================================================
class Solved:
    pass


def assemble(md: Model):
    n = md.nn()
    ndof = 6 * n
    rows, cols, vals = [], [], []
    mdiag = np.zeros(ndof)
    mrows, mcols, mvals = [], [], []
    connected = np.zeros(n, bool)
    for e in md.shells:
        X = np.array([md.xyz[i] for i in e["n"]])
        K, M, area, rec = shell_matrices(X, e["t"], MAT[e["mat"]], e.get("nsm", 0.0))
        e["rec"], e["area"] = rec, area
        e["S"] = shell_stress_lin(rec)
        dofs = np.array([6 * i + k for i in e["n"] for k in range(6)])
        r, c = np.meshgrid(dofs, dofs, indexing="ij")
        rows.append(r.ravel()); cols.append(c.ravel()); vals.append(K.ravel())
        mdiag[dofs] += M
        connected[e["n"]] = True
    for b in md.beams:
        x1, x2 = md.xyz[b["n"][0]], md.xyz[b["n"][1]]
        mat = MAT[b["mat"]]
        sec = dict(b["sec"]); sec.update(E=b.get("E", mat["E"]) * b.get("kEI", 1.0),
                                        G=mat["E"] / (2 * (1 + mat["nu"])), rho=b.get("rho", mat["rho"]))
        if b.get("kEA") is not None:
            sec["A_k"] = sec["A"]
        K, M, L, R, kloc, T = beam_matrices(x1, x2, sec, b.get("v", (0, 0, 1)))
        if b.get("kEA") is not None:                 # separate axial factor
            f = b["kEA"] / b.get("kEI", 1.0)
            Kl = kloc.copy()
            for (i, j) in ((0, 0), (6, 6), (0, 6), (6, 0)):
                Kl[i, j] *= f
            kloc = Kl
            K = T.T @ kloc @ T
        b.update(L=L, R=R, kloc=kloc, T=T, secE=sec)
        b["S"] = beam_stress_lin(dict(kloc=kloc, T=T, sec=b["sec"]))
        dofs = np.array([6 * i + k for i in b["n"] for k in range(6)])
        r, c = np.meshgrid(dofs, dofs, indexing="ij")
        rows.append(r.ravel()); cols.append(c.ravel()); vals.append(K.ravel())
        mr, mc = np.meshgrid(dofs, dofs, indexing="ij")
        mrows.append(mr.ravel()); mcols.append(mc.ravel()); mvals.append(M.ravel())
        connected[b["n"]] = True
    for s in md.springs:
        i, j = s["n"]
        for d, k in s["k"].items():
            a, b_ = 6 * i + d, 6 * j + d
            rows.append(np.array([a, b_, a, b_])); cols.append(np.array([a, b_, b_, a]))
            vals.append(np.array([k, k, -k, -k]))
        connected[[i, j]] = True
    for nd, m in md.lumped.items():
        mdiag[6 * nd:6 * nd + 3] += m
    for rb in md.rigid:
        dofs = np.arange(6 * rb["master"], 6 * rb["master"] + 6)
        mr, mc = np.meshgrid(dofs, dofs, indexing="ij")
        mrows.append(mr.ravel()); mcols.append(mc.ravel()); mvals.append(rb["M6"].ravel())
        connected[rb["master"]] = True
    K = sp.csr_matrix((np.concatenate(vals), (np.concatenate(rows), np.concatenate(cols))), shape=(ndof, ndof))
    M = sp.diags(mdiag) + (sp.csr_matrix((np.concatenate(mvals), (np.concatenate(mrows), np.concatenate(mcols))),
                                         shape=(ndof, ndof)) if mvals else 0)
    for s, m in md.mpc.items():
        connected[s] = True
        connected[m] = True
    return K.tocsr(), sp.csr_matrix(M), connected


def transformation(md: Model, connected):
    """u_full = T u_red: rigid-link slaves follow their masters; fixed dofs removed; unconnected nodes removed."""
    n = md.nn()
    red = -np.ones(6 * n, int)
    cnt = 0
    for i in range(n):
        if not connected[i] or i in md.mpc:
            continue
        fx = md.fixed.get(i, [])
        for k in range(6):
            if k in fx:
                continue
            red[6 * i + k] = cnt
            cnt += 1
    r, c, v = [], [], []
    for i in range(n):
        if not connected[i]:
            continue
        if i in md.mpc:
            m = md.mpc[i]
            if m in md.mpc:
                raise ValueError("chained rigid link")
            d = md.xyz[i] - md.xyz[m]
            # u_s = u_m + th_m x d ; th_s = th_m
            C = np.zeros((6, 6))
            C[:3, :3] = np.eye(3)
            C[:3, 3:] = np.array([[0, d[2], -d[1]], [-d[2], 0, d[0]], [d[1], -d[0], 0]])
            C[3:, 3:] = np.eye(3)
            for a in range(6):
                for b in range(6):
                    if C[a, b] != 0 and red[6 * m + b] >= 0:
                        r.append(6 * i + a); c.append(red[6 * m + b]); v.append(C[a, b])
        else:
            for k in range(6):
                if red[6 * i + k] >= 0:
                    r.append(6 * i + k); c.append(red[6 * i + k]); v.append(1.0)
    return sp.csr_matrix((v, (r, c)), shape=(6 * n, cnt)), red


def solve_modes(md: Model, nmodes=40):
    K, M, connected = assemble(md)
    T, red = transformation(md, connected)
    Kr = (T.T @ K @ T).tocsc()
    Mr = (T.T @ M @ T).tocsc()
    Kr = 0.5 * (Kr + Kr.T)
    Mr = 0.5 * (Mr + Mr.T)
    w2, phi = spla.eigsh(Kr, k=nmodes, M=Mr, sigma=-1.0, which="LM", tol=1e-8)
    o = np.argsort(w2)
    w2, phi = w2[o], phi[:, o]
    f = np.sqrt(np.maximum(w2, 0)) / (2 * math.pi)
    sol = Solved()
    sol.K, sol.M, sol.T, sol.Kr, sol.Mr, sol.f, sol.phi = K, M, T, Kr, Mr, f, phi
    # effective masses (base translation in x, y, z of every unconstrained node)
    n = md.nn()
    rf = np.zeros((6 * n, 3))
    for i in range(n):
        for d in range(3):
            rf[6 * i + d, d] = 1.0
    # reduced influence vector: masters' translation dofs
    rr = np.zeros((T.shape[1], 3))
    for i in range(n):
        for d in range(3):
            if red[6 * i + d] >= 0:
                rr[red[6 * i + d], d] = 1.0
    sol.r = rr
    mm = np.einsum("ij,ij->j", phi, Mr @ phi)
    L = phi.T @ (Mr @ rr)
    sol.gamma = L / mm[:, None]
    sol.meff = L ** 2 / mm[:, None]
    sol.mtot = np.array([rr[:, d] @ (Mr @ rr[:, d]) for d in range(3)])
    sol.mfull = M.diagonal()[0::6].sum()
    return sol


def static(md: Model, sol, acc_g: Sequence[float]):
    """Inertial load: acceleration acc_g (g, body axes) applied to every mass; returns full displacements."""
    if not hasattr(sol, "lu"):
        sol.lu = spla.splu(sol.Kr.tocsc())
    F = sol.Mr @ (sol.r @ (np.array(acc_g) * G0))
    u = sol.lu.solve(-F)                               # base accelerates +a: inertia load -M a (relative frame)
    return sol.T @ u


# ======================================================================================================
# Self-test: element verification against closed-form solutions
# ======================================================================================================
def plate_model(a, b, t, mat, nx, ny, supports):
    md = Model()
    xs = np.linspace(0, a, nx + 1); ys = np.linspace(0, b, ny + 1)
    for i in range(nx):
        for j in range(ny):
            ns = [md.node(xs[i], ys[j], 0), md.node(xs[i + 1], ys[j], 0), md.node(xs[i + 1], ys[j + 1], 0),
                  md.node(xs[i], ys[j + 1], 0)]
            md.shells.append(dict(n=ns, t=t, mat=mat))
    for i in range(md.nn()):
        x, y, z = md.xyz[i]
        fx = [0, 1, 5]                                 # suppress in-plane / drilling for a plate test
        if supports == "ss" and (abs(x) < 1e-9 or abs(x - a) < 1e-9 or abs(y) < 1e-9 or abs(y - b) < 1e-9):
            fx = fx + [2]
        if supports == "corners" and ((abs(x) < 1e-9 or abs(x - a) < 1e-9) and (abs(y) < 1e-9 or abs(y - b) < 1e-9)):
            fx = fx + [2]
        if supports == "clamped_edge" and abs(x) < 1e-9:
            fx = list(range(6))
        md.fixed[i] = fx
    return md


def selftest(verbose=True):
    res = {}
    E, nu, rho = MAT["FR4"]["E"], MAT["FR4"]["nu"], MAT["FR4"]["rho"]
    a, t = 0.096, 0.0016
    D = E * t ** 3 / (12 * (1 - nu ** 2))
    for sup, lam2, label in (("ss", 2 * math.pi ** 2, "simply supported square, mode (1,1): lambda^2 = 2 pi^2"),
                             ("corners", 7.12, "corner-supported square (Leissa NASA SP-160): lambda^2 = 7.12")):
        md = plate_model(a, a, t, "FR4", 16, 16, sup)
        s = solve_modes(md, 4)
        f_th = lam2 / (2 * math.pi * a * a) * math.sqrt(D / (rho * t))
        res[sup] = dict(fe=float(s.f[0]), theory=f_th, err_pct=100 * (s.f[0] / f_th - 1), label=label)
    # cantilever plate strip as a beam (bending out of plane): f = 1.875^2/(2 pi) sqrt(EI/(m L^4))
    L, b = 0.1, 0.01
    md = plate_model(L, b, t, "FR4", 40, 2, "clamped_edge")
    s = solve_modes(md, 3)
    EI = E / (1 - nu ** 2) * b * t ** 3 / 12
    f_th = 1.875 ** 2 / (2 * math.pi) * math.sqrt(EI / (rho * b * t * L ** 4))
    res["cantilever_strip"] = dict(fe=float(s.f[0]), theory=f_th, err_pct=100 * (s.f[0] / f_th - 1),
                                   label="cantilever strip 100 x 10 x 1.6 mm (plate EI)")
    # in-plane bending of a cantilever plate (membrane, Wilson modes): tip deflection vs Euler beam
    md = Model()
    Lm, h = 0.1, 0.02
    xs = np.linspace(0, Lm, 11); ys = np.linspace(0, h, 3)
    for i in range(10):
        for j in range(2):
            md.shells.append(dict(n=[md.node(xs[i], ys[j], 0), md.node(xs[i + 1], ys[j], 0),
                                     md.node(xs[i + 1], ys[j + 1], 0), md.node(xs[i], ys[j + 1], 0)], t=0.002,
                                  mat="Al6061"))
    for i in range(md.nn()):
        x = md.xyz[i][0]
        md.fixed[i] = list(range(6)) if abs(x) < 1e-9 else [2, 3, 4]
    K, M, conn = assemble(md)
    T, red = transformation(md, conn)
    Kr = (T.T @ K @ T).tocsc()
    F = np.zeros(6 * md.nn())
    tips = [i for i in range(md.nn()) if abs(md.xyz[i][0] - Lm) < 1e-9]
    for i in tips:
        F[6 * i + 1] = 1.0 / len(tips)
    u = T @ spla.spsolve(Kr, T.T @ F)
    d_fe = np.mean([u[6 * i + 1] for i in tips])
    Em = MAT["Al6061"]["E"]
    d_th = 1.0 * Lm ** 3 / (3 * Em * 0.002 * h ** 3 / 12) + 1.0 * Lm / (5 / 6 * Em / (2 * 1.33) * 0.002 * h)
    res["membrane_cantilever"] = dict(fe=d_fe, theory=d_th, err_pct=100 * (d_fe / d_th - 1),
                                      label="in-plane cantilever 100 x 20 x 2 mm, 10 x 2 mesh (Timoshenko beam)")
    # beam cantilever
    md = Model()
    nb = 10
    sec = beam_section_round(0.005)
    for i in range(nb):
        md.beams.append(dict(n=[md.node(i * 0.01, 0, 0), md.node((i + 1) * 0.01, 0, 0)], mat="Al6061", sec=sec,
                             v=(0, 0, 1)))
    md.fixed[md.node(0, 0, 0)] = list(range(6))
    s = solve_modes(md, 3)
    m_l = MAT["Al6061"]["rho"] * sec["A"]
    f_th = 1.875 ** 2 / (2 * math.pi) * math.sqrt(MAT["Al6061"]["E"] * sec["Iy"] / (m_l * 0.1 ** 4))
    res["beam_cantilever"] = dict(fe=float(s.f[0]), theory=f_th, err_pct=100 * (s.f[0] / f_th - 1),
                                  label="beam cantilever 5 mm dia x 100 mm Al")
    if verbose:
        for k, v in res.items():
            print(f"  selftest {k:22s} FE {v['fe']:.6g}  theory {v['theory']:.6g}  error {v['err_pct']:+.2f} %  ({v['label']})")
    return res



# ======================================================================================================
# The spacecraft model
# ======================================================================================================
def fill(vals, lo, hi, h):
    v = sorted(set(round(x, 4) for x in vals if lo - 1e-9 <= x <= hi + 1e-9) | {round(lo, 4), round(hi, 4)})
    out = [v[0]]
    for a, b in zip(v[:-1], v[1:]):
        n = max(1, int(math.ceil((b - a) / h - 1e-9)))
        out += [a + (b - a) * k / n for k in range(1, n + 1)]
    return np.array(out)


def within(L, a, b):
    return [x for x in L if a - 1e-6 <= x <= b + 1e-6]


@dataclass
class SC:
    md: Model
    groups: Dict[str, List[int]]          # group -> shell indices
    bgroups: Dict[str, List[int]]         # group -> beam indices
    nodegroup: Dict[int, str]
    board_supports: Dict[str, List[int]]
    board_nodes: Dict[str, List[int]]
    special: Dict[str, object]
    mass_report: Dict[str, float]


def build_spacecraft(bc="rails", snubber="soft", h=5.0, cs2=False, t_eps=1.6, t_car=1.6, deck_E=0.35,
                     spider_r=2.6, car_window=True, car_frame=True, ti_column=(1, 1), grown=False,
                     harness_tie="deck") -> SC:
    sys.path.insert(0, HERE)
    import model as CAD
    P = CAD.P
    parts = CAD.build_parts()
    mass = CAD.assign_masses(parts, grown)          # grown: MEV masses (every MEL row at its growth)
    byname = {pt.name: pt for pt in parts}
    md = Model()
    mm = 1e-3
    groups: Dict[str, List[int]] = {}
    bgroups: Dict[str, List[int]] = {}
    nodegroup: Dict[int, str] = {}
    W2 = 49.0
    xr = 47.33                     # rail centroid (L 8.5 x 8.5 x 1.5): 2.67 mm in from the outer faces
    zmz, zpz = -47.7, 49.2         # end-plate mid-planes (CDS frame: -Z outer face -48.5 recessed, +Z outer +50)
    zb = dict(deck=-16.0, EPS=P["z_eps"] + 0.8, OBC=P["z_obc"] + 0.8, carrier=P["z_car"] + t_car / 2, RF=P["z_rf"] + 0.8)
    special_t = dict(EPS=t_eps, OBC=1.6, carrier=t_car, RF=1.6, deck=1.0)
    zsp = 30.3
    wxm = 53.6                     # wing substrate mid-plane (x)
    hy, hz0, hz1, hdy, hdz, snz = 38.5, -40.0, 44.85, P["holddown_y"], P["holddown_z"], P["snub_z"]
    cw = tuple(P["car_win"])      # carrier window for the gyro (x0, x1, y0, y1), OBC item C3
    Xm = [-W2, -45.085, -41.5, -40, -38.5, -35.5, -15, 0, 15, 35.5, 38.5, 40, 41.5, 45.085, W2, -24, 24, -12, 12,
          cw[0], cw[1]]
    Ym = [-W2, -47.945, -43, -41.5, -41.0, -38.5, -34, -31, -21, -20, -15, -6.5, 0, 6.5, 15, 20, 21, 26, 31, 38.5,
          41.0, 41.5, 43, 47.945, W2, 27, 33, cw[2], cw[3]]
    Zm = [-56.75, -50, zmz, -43, hz0, -32.9, zb["deck"], zb["EPS"], zb["OBC"], zb["carrier"], zsp, zb["RF"], hz1,
          snz, 49.0, zpz, 50, 56.75]
    X = fill(Xm, -W2, W2, h); Y = fill(Ym, -W2, W2, h); Z = fill(Zm, -56.75, 56.75, h)

    def panel(name, plane, c, ur, vr, t, mat, holes=(), nsm=0.0, U=None, V=None):
        U = within(U if U is not None else (Y if plane == "x" else X), *ur)
        V = within(V if V is not None else (Z if plane in ("x", "y") else Y), *vr)
        idx = []
        for i in range(len(U) - 1):
            for j in range(len(V) - 1):
                uc, vc = (U[i] + U[i + 1]) / 2, (V[j] + V[j + 1]) / 2
                if any(h0 < uc < h1 and g0 < vc < g1 for h0, h1, g0, g1 in holes):
                    continue
                q = [(U[i], V[j]), (U[i + 1], V[j]), (U[i + 1], V[j + 1]), (U[i], V[j + 1])]
                if plane == "z":
                    ns = [md.node(a * mm, b * mm, c * mm) for a, b in q]
                elif plane == "x":
                    ns = [md.node(c * mm, a * mm, b * mm) for a, b in q]
                else:
                    ns = [md.node(a * mm, c * mm, b * mm) for a, b in q]
                md.shells.append(dict(n=ns, t=t * mm, mat=mat, nsm=nsm, group=name))
                idx.append(len(md.shells) - 1)
                for n_ in ns:
                    nodegroup.setdefault(n_, name)
        groups.setdefault(name, []).extend(idx)
        return idx

    def beam(name, p0, p1, mat, sec, v=(0, 0, 1), nseg=1, **kw):
        a = np.array(p0, float) * mm; b = np.array(p1, float) * mm
        ids = [md.node(*a)]
        for k in range(1, nseg + 1):
            ids.append(md.node(*(a + (b - a) * k / nseg)))
        out = []
        for k in range(nseg):
            md.beams.append(dict(n=[ids[k], ids[k + 1]], mat=mat, sec=sec, v=v, group=name, **kw))
            out.append(len(md.beams) - 1)
            for n_ in ids[k:k + 2]:
                nodegroup.setdefault(n_, name)
        bgroups.setdefault(name, []).extend(out)
        return out, ids

    def sec_mm(s):
        o = dict(s)
        for k in ("A",):
            o[k] = s[k] * 1e-6
        for k in ("Iy", "Iz", "J"):
            o[k] = s[k] * 1e-12
        for k in ("cy", "cz"):
            o[k] = s[k] * 1e-3
        return o

    # ---------------- frame ---------------------------------------------------------------------------
    panel("endplate_mz", "z", zmz, (-W2, W2), (-W2, W2), 1.6, "Al6061", holes=[(-35.5, 35.5, -35.5, 35.5)])
    panel("endplate_pz", "z", zpz, (-W2, W2), (-W2, W2), 1.6, "Al6061", holes=[(-15, 15, -15, 15)])
    for s_ in (1, -1):
        panel("shear_x", "x", s_ * W2, (-41.5, 41.5), (zmz, zpz), 1.0, "Al6061")
        panel("shear_y", "y", s_ * W2, (-41.5, 41.5), (zmz, zpz), 1.0, "Al6061")
    Lsec = sec_mm(beam_section_L(8.5, 1.5))
    Ssec = sec_mm(beam_section_rect(8.5, 8.5))
    rail_nodes = {}
    for sx in (1, -1):
        for sy in (1, -1):
            zs = list(Z)
            ids = [md.node(sx * xr * mm, sy * xr * mm, z * mm) for z in zs]
            for k in range(len(zs) - 1):
                z0, z1 = zs[k], zs[k + 1]
                sec = Ssec if (z1 <= -50 + 1e-6 or z0 >= 50 - 1e-6) else Lsec
                md.beams.append(dict(n=[ids[k], ids[k + 1]], mat="Al6061", sec=sec, v=(1, 0, 0), group="rails"))
                bgroups.setdefault("rails", []).append(len(md.beams) - 1)
            for n_ in ids:
                nodegroup.setdefault(n_, "rails")
            rail_nodes[(sx, sy)] = dict(zip([round(z, 4) for z in zs], ids))
    # rigid links: shear-panel side edges and end-plate corner squares to the rail node at the same z
    for i, p in enumerate(md.xyz):
        x, y, z = p / mm
        g = nodegroup.get(i, "")
        if g.startswith("shear") or g.startswith("endplate"):
            if abs(abs(x) - W2) < 1e-6 and abs(abs(y) - 41.5) < 1e-6 or \
               abs(abs(y) - W2) < 1e-6 and abs(abs(x) - 41.5) < 1e-6 or \
               (g.startswith("endplate") and abs(x) >= 41.5 - 1e-6 and abs(y) >= 41.5 - 1e-6):
                key = (int(np.sign(x)), int(np.sign(y)))
                md.mpc[i] = rail_nodes[key][round(z, 4)]
    # ---------------- boards, deck, spreader --------------------------------------------------------------
    bx, by = 45.085, 47.945
    notch = [(-bx - 1, -35.5, -6.5, 6.5)]
    panel("deck", "z", zb["deck"], (-bx, bx), (-by, by), 1.0, "Al6061", holes=notch)
    panel("EPS", "z", zb["EPS"], (-bx, bx), (-by, by), t_eps, "FR4", holes=notch)
    panel("OBC", "z", zb["OBC"], (-bx, bx), (-by, by), 1.6, "FR4")
    panel("carrier", "z", zb["carrier"], (-bx, bx), (-by, by), t_car, "FR4", holes=[cw] if car_window else ())
    panel("RF", "z", zb["RF"], (-bx, bx), (-by, by), 1.6, "FR4")
    panel("spreader", "z", zsp, (-40, 40), (-34, 26), 5.0, "Al6061")
    panel("spreader", "z", zsp, (40, W2), (-20, 20), 3.0, "Al6061")
    panel("spreader", "z", zsp, (-W2, -40), (-20, 20), 3.0, "Al6061")
    # ---------------- columns and centre supports (hex 5 AF Al + M3 A286 stud; EI x 1/3 joint factor) -----
    s5 = 5.0
    A_hex = math.sqrt(3) / 2 * s5 ** 2 - math.pi / 4 * 3.2 ** 2
    I_hex = 5 * math.sqrt(3) / 144 * s5 ** 4 - math.pi / 64 * 3.2 ** 4
    J_hex = 0.1154 * s5 ** 4 * 1.04 - math.pi / 32 * 3.2 ** 4
    d_st = 2.39
    A_st, I_st = math.pi / 4 * d_st ** 2, math.pi / 64 * d_st ** 4
    rE = MAT["A286"]["E"] / MAT["Al6061"]["E"]
    colsec = sec_mm(dict(A=A_hex + rE * A_st, Iy=I_hex + rE * I_st, Iz=I_hex + rE * I_st, J=J_hex, cy=2.89, cz=2.89))
    colsec["share_al"] = I_hex / (I_hex + rE * I_st)
    rT = MAT["Ti64"]["E"] / MAT["Al6061"]["E"]     # Ti-6Al-4V stud at the column next to board A2 (MECHANICAL_DESIGN 4.4)
    colsec_ti = sec_mm(dict(A=A_hex + rT * A_st, Iy=I_hex + rT * I_st, Iz=I_hex + rT * I_st, J=J_hex, cy=2.89, cz=2.89))
    ti_beams = []
    zc = [zmz, -40.0, -32.9, -24.5, zb["deck"], zb["EPS"], zb["OBC"], zb["carrier"], 24.4, 30.0, zb["RF"], 42.4, zpz]
    col_ids = {}
    for sx in (1, -1):
        for sy in (1, -1):
            ids = []
            for k in range(len(zc) - 1):
                ti = ti_column is not None and (sx, sy) == tuple(ti_column)
                e, nd = beam("columns", (sx * 40, sy * 43, zc[k]), (sx * 40, sy * 43, zc[k + 1]), "Al6061",
                             colsec_ti if ti else colsec, v=(1, 0, 0), kEI=1 / 3, kEA=1.0)
                if ti:
                    ti_beams += e
                ids += nd if not ids else nd[1:]
            col_ids[(sx, sy)] = ids
    zcs = [zb["deck"], zb["EPS"], zb["OBC"], zb["carrier"], 24.5, zsp]
    for k in range(len(zcs) - 1):
        beam("centre_support", (0, 21, zcs[k]), (0, 21, zcs[k + 1]), "Al6061", colsec, v=(1, 0, 0), kEI=1 / 3, kEA=1.0)
    beam("centre_support", (0, 0, zsp), (0, 0, zb["RF"]), "Al6061", colsec, v=(1, 0, 0), kEI=1 / 3, kEA=1.0)
    # ---------------- carrier window frame (D-11): Al 2.5 x 3.0 mm on the bottom face, offset from the mid-plane ---
    if car_window and car_frame:
        fw, fh = P["car_frame_w"], P["car_frame_h"]
        d = fh / 2 + t_car / 2
        n_ = MAT["FR4"]["E"] / MAT["Al6061"]["E"]
        bp, tp = 20 * t_car, t_car                     # effective plate strip, 10 t each side
        Ap, Ar = n_ * bp * tp, fw * fh
        yb = Ar * d / (Ap + Ar)
        Iy = n_ * bp * tp ** 3 / 12 + Ap * yb ** 2 + fw * fh ** 3 / 12 + Ar * (d - yb) ** 2 - n_ * bp * tp ** 3 / 12
        rs = beam_section_rect(fw, fh)
        fsec = sec_mm(dict(A=Ar, Iy=Iy, Iz=rs["Iz"], J=rs["J"], cy=fw / 2, cz=d + fh / 2))
        x0, x1, y0, y1 = cw
        xe = [x for x in X if x0 - 1e-6 <= x <= x1 + 1e-6]
        ye = [y for y in Y if y0 - 1e-6 <= y <= y1 + 1e-6]
        segs = [((xe[i], y), (xe[i + 1], y)) for y in (y0, y1) for i in range(len(xe) - 1)] + \
               [((x, ye[i]), (x, ye[i + 1])) for x in (x0, x1) for i in range(len(ye) - 1)]
        for (ax_, ay_), (bx_, by_) in segs:
            beam("carrier_frame", (ax_, ay_, zb["carrier"]), (bx_, by_, zb["carrier"]), "Al6061", fsec, v=(0, 0, 1))
    if cs2:                                  # second support under the module: "deck" deck->EPS->OBC->carrier,
        zc2 = [zb["deck"], zb["EPS"], zb["OBC"], zb["carrier"]]   # "eps" EPS->OBC->carrier (clears the GNSS receiver)
        if cs2 == "eps":
            zc2 = zc2[1:]
        for k in range(len(zc2) - 1):
            beam("centre_support", (0, -21, zc2[k]), (0, -21, zc2[k + 1]), "Al6061", colsec, v=(1, 0, 0), kEI=1 / 3,
                 kEA=1.0)
    # ---------------- battery: rigid body on four ears and G-10CR isolators ------------------------------
    pack_parts = [n for n in ("cell_1", "cell_2", "cell_3", "cell_4", "battery_clamp", "battery_sense_board",
                              "battery_heaters_wrap", "battery_cell_retention") if n in mass]
    m_tot, mc, Ic = 0.0, np.zeros(3), np.zeros((3, 3))
    rows = []
    for n_ in pack_parts:
        v, c, Iu = CAD.solid_props(byname[n_].stowed)
        mk = mass[n_] * 1e-3
        rows.append((mk, c * mm, Iu * (mk / v) * 1e-6))
        m_tot += mk; mc += mk * c * mm
    cg = mc / m_tot
    for mk, c, Ip in rows:
        d = c - cg
        Ic += Ip + mk * (np.dot(d, d) * np.eye(3) - np.outer(d, d))
    master = md.node(*cg)
    nodegroup[master] = "battery"
    M6 = np.zeros((6, 6)); M6[:3, :3] = m_tot * np.eye(3); M6[3:, 3:] = Ic
    md.rigid.append(dict(master=master, M6=M6))
    g10 = MAT["G10CR"]["E"]; eti = MAT["Ti64"]["E"]
    Ab = math.pi / 4 * (6 ** 2 - 2.7 ** 2); Ib = math.pi / 64 * (6 ** 4 - 2.7 ** 4)
    As_, Is_ = math.pi / 4 * 2.0 ** 2, math.pi / 64 * 2.0 ** 4
    Lfe, Lreal = abs(-37.65 - zmz), 8.0
    isosec = sec_mm(dict(A=Ab, Iy=Ib + Is_ * eti / g10, Iz=Ib + Is_ * eti / g10, J=2 * Ib, cy=3.0, cz=3.0))
    isosec["share_bush"] = Ib / (Ib + Is_ * eti / g10)
    earsec = sec_mm(beam_section_rect(10.0, 2.5))
    iso_beams, ear_beams = [], []
    v_iso, _, _ = CAD.solid_props(byname["battery_isolators"].stowed)
    for sx in (1, -1):
        for sy in (1, -1):
            root = md.node(sx * 35.5 * mm, sy * 31 * mm, -37.65 * mm)
            md.mpc[root] = master
            nodegroup[root] = "battery"
            e, nd = beam("battery_ear", (sx * 35.5, sy * 31, -37.65), (sx * 38.5, sy * 31, -37.65), "Al6061", earsec,
                         v=(0, 0, 1), rho=0.0)
            ear_beams += e
            e, nd = beam("battery_isolator", (sx * 38.5, sy * 31, -37.65), (sx * 38.5, sy * 31, zmz), "G10CR",
                         isosec, v=(1, 0, 0), kEI=0.6 * (Lfe / Lreal) ** 3, kEA=0.6 * Lfe / Lreal,
                         rho=mass["battery_isolators"] * 1e-3 / 4 / (Ab * 1e-6 * Lfe * mm))
            iso_beams += e
    # ---------------- stowed wings, hinges, hold-down, snubber -------------------------------------------
    wing_shells = {}
    for s_, tag in ((1, "W1"), (-1, "W2")):
        wing_shells[tag] = panel(f"wing_{tag}", "x", s_ * wxm, (-41.0, 41.0), (-43.0, 49.0), 1.6, "FR4")
        # hinge knuckle (pin): translations only; cup/cone hold-down post: translations only (no moment);
        # stiffness of the bracket/post (Al): normal 2e8 N/m (axial), in-plane 5e6 (hinge) / 2e6 N/m (post)
        for zz in (hz0, hz1):
            a_ = md.node(s_ * W2 * mm, hy * mm, zz * mm); b_ = md.node(s_ * wxm * mm, hy * mm, zz * mm)
            md.springs.append(dict(n=[a_, b_], k={0: 2e8, 1: 5e6, 2: 5e6}, group=f"hinge_{tag}"))
        a_ = md.node(s_ * W2 * mm, hdy * mm, hdz * mm); b_ = md.node(s_ * wxm * mm, hdy * mm, hdz * mm)
        md.springs.append(dict(n=[a_, b_], k={0: 2e8, 1: 2e6, 2: 2e6}, group=f"holddown_{tag}"))
        a_ = md.node(s_ * W2 * mm, hdy * mm, snz * mm); b_ = md.node(s_ * wxm * mm, hdy * mm, snz * mm)
        if snubber == "soft":
            k_sn = MAT["silicone"]["E"] * math.pi * 2e-3 ** 2 / 1.7e-3
            md.springs.append(dict(n=[a_, b_], k={0: k_sn}, group=f"snubber_{tag}"))
        else:                                       # second (shimmed) cup/cone post at the +Z free corner
            md.springs.append(dict(n=[a_, b_], k={0: 2e8, 1: 2e6, 2: 2e6}, group=f"snubber_{tag}"))
        for n_ in (a_, b_):
            nodegroup.setdefault(n_, f"wing_{tag}")
    # an isolated flat-shell panel joined by translational springs has a zero-energy drilling mode: suppress the
    # rotation about the panel normal (global x) at the wing nodes
    for tag in ("W1", "W2"):
        for k in wing_shells[tag]:
            for n_ in md.shells[k]["n"]:
                md.fixed[n_] = sorted(set(md.fixed.get(n_, []) + [3]))
    # ---------------- boundary conditions ----------------------------------------------------------------
    for key, d in rail_nodes.items():
        for z, n_ in d.items():
            if bc == "rails":
                fx = [0, 1] + ([2] if abs(abs(z) - 56.75) < 1e-6 else [])
            else:
                fx = [0, 1, 2] if abs(abs(z) - 56.75) < 1e-6 else []
            if fx:
                md.fixed[n_] = fx

    # ---------------- spiders: point attachments of beams to shells spread over the spacer / washer face ----
    shell_nodes = {}
    for k, e in enumerate(md.shells):
        for n_ in e["n"]:
            shell_nodes.setdefault(n_, e["group"])
    beam_end = set()
    for bm in md.beams:
        if bm["group"] in ("columns", "centre_support", "battery_isolator"):
            beam_end.update(bm["n"])
    XYZall = np.array(md.xyz)
    spider_count = 0
    for c in sorted(beam_end):
        if c not in shell_nodes or c in md.mpc:
            continue
        g = shell_nodes[c]
        d = np.linalg.norm(XYZall - XYZall[c], axis=1)
        near = np.where((d > 1e-9) & (d <= spider_r * mm + 1e-9))[0]
        for n_ in near:
            if n_ in md.mpc or n_ in beam_end or shell_nodes.get(n_) != g or n_ in md.fixed:
                continue
            if abs(XYZall[n_][2] - XYZall[c][2]) > 1e-9:
                continue
            md.mpc[int(n_)] = int(c)
            spider_count += 1
    # ---------------- masses ---------------------------------------------------------------------------
    rep: Dict[str, float] = {}

    def shell_area(idx):
        tot = 0.0
        for k in idx:
            X_ = np.array([md.xyz[i] for i in md.shells[k]["n"]])
            tot += np.linalg.norm(np.cross(X_[1] - X_[0], X_[3] - X_[0]))
        return tot

    def shell_struct_mass(idx):
        return sum(MAT[md.shells[k]["mat"]]["rho"] * md.shells[k]["t"] * shell_area([k]) for k in idx)

    def nsm_to(idx, grams, label):
        A = shell_area(idx)
        for k in idx:
            md.shells[k]["nsm"] = md.shells[k].get("nsm", 0.0) + grams * 1e-3 / A
        rep[label] = rep.get(label, 0.0) + grams

    def lump_near(grams, pts, label, group=None):
        """Spread grams equally over the nodes nearest to each point (mm)."""
        cand = [i for i in range(md.nn()) if (group is None or nodegroup.get(i, "").startswith(group))
                and i not in md.mpc]
        XYZ = np.array([md.xyz[i] for i in cand]) / mm
        for p in pts:
            j = cand[int(np.argmin(np.linalg.norm(XYZ - np.array(p), axis=1)))]
            md.add_mass(j, grams * 1e-3 / len(pts))
        rep[label] = rep.get(label, 0.0) + grams

    def set_density(idx, grams, label, beams=False):
        if beams:
            cur = 0.0
            for k in idx:
                b = md.beams[k]
                L = np.linalg.norm(md.xyz[b["n"][1]] - md.xyz[b["n"][0]])
                cur += b["sec"]["A"] * L
            for k in idx:
                md.beams[k]["rho"] = grams * 1e-3 / cur
        else:
            A = sum(md.shells[k]["t"] * shell_area([k]) for k in idx)
            for k in idx:
                md.shells[k]["rho_override"] = grams * 1e-3 / A
        rep[label] = rep.get(label, 0.0) + grams

    frame_sh = groups["endplate_mz"] + groups["endplate_pz"] + groups["shear_x"] + groups["shear_y"]
    rails_b = bgroups["rails"]
    Vsh = sum(md.shells[k]["t"] * shell_area([k]) for k in frame_sh)
    Vb = sum(md.beams[k]["sec"]["A"] * np.linalg.norm(md.xyz[md.beams[k]["n"][1]] - md.xyz[md.beams[k]["n"][0]])
             for k in rails_b)
    m_frame = sum(v for n_, v in mass.items() if n_.startswith(("rail_", "endplate_", "shear_")))   # row 1
    rho_f = m_frame * 1e-3 / (Vsh + Vb)
    for k in frame_sh:
        md.shells[k]["rho_override"] = rho_f
    for k in rails_b:
        md.beams[k]["rho"] = rho_f
    rep["frame (row 1)"] = m_frame
    # body panels and cells on the shear panels
    for s_ in (1, -1):
        for g_, ax in (("shear_x", 0), ("shear_y", 1)):
            idx = [k for k in groups[g_] if np.sign(md.xyz[md.shells[k]["n"][0]][ax]) == s_]
            tg = ("p" if s_ > 0 else "m") + "xy"[ax]
            nsm_to(idx, mass[f"body_panel_{tg}"] + mass[f"body_cells_{tg}"], "body panels (row 6)")
    nsm_to(groups["endplate_mz"], mass["AntS_housing"] + sum(mass[n] for n in mass if n.startswith("AntS_") and
                                                               n.endswith("_element")), "AntS (row 19)")
    for nm in ("EPS",):
        nsm_to(groups["EPS"], mass["EPS"] + mass["EPS_parts"] - shell_struct_mass(groups["EPS"]) * 1e3, "EPS board")
    nsm_to(groups["OBC"], mass["OBC"] + mass["OBC_parts"] - shell_struct_mass(groups["OBC"]) * 1e3, "OBC board")
    gy = [k for k in groups["OBC"] if all(cw[0] - 1e-6 <= md.xyz[i][0] / mm <= cw[1] + 1e-6 and cw[2] - 1e-6 <=
                                              md.xyz[i][1] / mm <= cw[3] + 1e-6 for i in md.shells[k]["n"])]
    gyro_nodes = sorted({i for k in gy for i in md.shells[k]["n"]})
    nsm_to(gy, mass["gyro_MG370"], "gyro on OBC")
    car_m = mass["Jetson_carrier"] + mass["Jetson_carrier_parts"] + mass.get("carrier_frame_screws", 0.0)      # the window frame is the beams' own mass
    nsm_to(groups["carrier"], car_m - shell_struct_mass(groups["carrier"]) * 1e3, "carrier board")
    jy = P["jet_y"]
    modk = [k for k in groups["carrier"] if all(-34.8 - 1e-6 <= md.xyz[i][0] / mm <= 34.8 + 1e-6 and jy - 26.5 - 1e-6
                                                 <= md.xyz[i][1] / mm <= jy + 22.5 + 1e-6 for i in md.shells[k]["n"])]
    nsm_to(modk, mass["Jetson_module"] + mass["Jetson_TIM_clamp"] + mass["SODIMM_connector"] + mass.get("module_standoffs", 0.0),
           "Jetson module, clamp, SODIMM on the carrier")
    nsm_to(groups["RF"], mass["RF"] + mass["RF_parts"] - shell_struct_mass(groups["RF"]) * 1e3, "RF board")
    set_density(groups["spreader"], mass["heat_spreader"], "spreader + flanges")
    # deck: 1.0 mm Al lightened to its MEL mass (6 g): equivalent plate, stiffness x 0.35
    set_density(groups["deck"], mass["torquer_deck"], "deck (lightened)")
    for k in groups["deck"]:
        md.shells[k]["E_factor"] = deck_E
    # deck items
    lump_near(mass["rod_X"] + mass["rod_clips"] / 3, [(x, 36, -16) for x in (-20, -10, 0, 10, 20)], "X rod", "deck")
    lump_near(mass["GNSS_receiver"], [(x, y, -16) for x in (-15, 5, 25) for y in (-40, -20, 0)], "GNSS receiver",
              "deck")
    lump_near(mass["rod_Y"] + mass["rod_clips"] / 3, [(W2, -16, -32.9), (W2, 16, -32.9)], "Y rod (clips on +X wall)",
              "shear_x")
    # Z rod tied to the -X shear panel only (two clips, z -39 and -8; through the deck and EPS notches without contact,
    # P3S-12), so the deck's motion puts no load on it
    zclip = [(-W2, 0, -39), (-W2, 0, -8)]
    lump_near(mass["rod_Z"] + mass["rod_clips"] / 3, zclip, "Z rod (two wall clips)", "shear_x")
    lump_near(sum(mass[n] for n in mass if n.startswith("magnetometer")), [(24, 27, zpz)], "magnetometer board A2",
              "endplate_pz")
    lump_near(mass["GNSS_patch"] + mass["GNSS_patch_adapter"], [(sx * 15, sy * 15, zpz) for sx in (1, -1)
                                                               for sy in (1, -1)], "GNSS patch + adapter", "endplate_pz")
    for nm, (x, y, z) in (("DS1", (46, 46, -56.75)), ("DS2", (-46, -46, -56.75)), ("DS3", (46, -46, 56.75)),
                          ("SP1", (-46, 46, -56.75)), ("SP2", (46, -46, -56.75))):
        lump_near(mass[nm], [(np.sign(x) * xr, np.sign(y) * xr, z)], "switches, plungers", "rails")
    lump_near(mass["RBF"] + mass["access_port"], [(20, -W2, -43), (-20, -W2, -43)], "RBF, access port", "shear_y")
    if "inertia_ballast" in mass:
        lump_near(mass["inertia_ballast"], [(-17, 7.5, zpz), (-17, -7.5, zpz)], "inertia ballast (row 36)", "endplate_pz")
    # W10 coax down the -X wall gap (y +-22), W4A ribbon in the +X wall gap (y -33.5, z +10 ... +43)
    if "coax_W10" in mass:
        lump_near(mass["coax_W10"], [(-W2, sy * 22, z) for sy in (1, -1) for z in (-40, 0, 30)], "W10 coax", "shear_x")
    if "ribbon_W4A" in mass:
        lump_near(mass["ribbon_W4A"], [(W2, -33.5, z) for z in (12, 26, 40)], "W4A ribbon", "shear_x")
    # harness zones (SQSO-HAR-001 2.0): three points along each zone's run, on what it is tied to (shear panels in the
    # wall gaps, the EPS under it, the deck openings, the end frames, the boards it crosses)
    CAD.harness_distribution()
    conn_g = CAD.harness_distribution.connectors
    for pt in parts:
        if not pt.name.startswith("hz_") or mass[pt.name] <= 0:
            continue
        bb = pt.stowed.BoundingBox()
        c = np.array([(bb.xmin + bb.xmax) / 2, (bb.ymin + bb.ymax) / 2, (bb.zmin + bb.zmax) / 2])
        ext = np.array([bb.xlen, bb.ylen, bb.zlen])
        ax_ = int(np.argmax(ext))
        pts_ = []
        for f_ in (-1 / 3, 0.0, 1 / 3):
            q = c.copy()
            q[ax_] += f_ * ext[ax_]
            pts_.append(tuple(q))
        if max(abs(c[0]), abs(c[1])) >= 43.0:
            grp = "shear"
        elif c[2] > 43.0:
            grp = "endplate_pz"
        elif -16.7 <= c[2] <= -4.0 and c[2] >= -15.0 and harness_tie == "eps":
            grp = "EPS"                     # bounding case (C-15): all the harness under the EPS tied to the EPS
        elif -16.7 <= c[2] <= -4.0 and c[2] >= -15.0:
            # under the EPS: the connector halves on the EPS, the wire on the deck straps (SQSO-HAR-001 2.0 10.3:
            # tie-downs on the shear panels, end frames, deck straps and the battery clamp, none between boards)
            mc = min(conn_g.get(pt.name[3:], 0.0), mass[pt.name])
            if mc > 0:
                lump_near(mc, pts_, "harness connector halves on the EPS", "EPS")
            if mass[pt.name] - mc > 0:
                lump_near(mass[pt.name] - mc, [(q[0], q[1], -16.0) for q in pts_], "harness under the EPS, on the deck",
                          "deck")
            continue
        elif -16.7 <= c[2] <= -4.0:
            grp = "deck"
        elif c[2] < -16.7:
            grp = "endplate_mz"
        elif c[2] > 19.0:
            grp = "carrier"
        else:
            grp = "OBC"
        lump_near(mass[pt.name], pts_, "harness zones (rows 26, 31, 32)", grp)
    if "TB_tabs" in mass:
        lump_near(mass["TB_tabs"], [(W2, 37, -34), (-W2, 37, -34)], "TB tabs", "shear_x")
    if "sun_sensor_PZ" in mass:
        lump_near(mass["sun_sensor_PZ"], [(-36, 36, zpz)], "+-Z sun sensors", "endplate_pz")
        lump_near(mass["sun_sensor_MZ"], [(-W2, 28, -46)], "+-Z sun sensors", "shear_x")
    # wing hardware
    for tag, s_ in (("W1", 1), ("W2", -1)):
        wm = mass[f"{tag}_substrate"] + mass[f"{tag}_cells_back"] + mass[f"{tag}_cells_front"] + \
            mass[f"{tag}_holddown_line"] + mass[f"{tag}_snubber"]
        nsm_to(wing_shells[tag], wm - shell_struct_mass(wing_shells[tag]) * 1e3, f"wing {tag}")
        hm = mass[f"{tag}_hinges"]
        lump_near(hm / 2, [(s_ * wxm, hy, hz0), (s_ * wxm, hy, hz1)], f"hinges {tag} (wing half)", f"wing_{tag}")
        lump_near(hm / 2, [(s_ * W2, hy, hz0), (s_ * W2, hy, hz1)], f"hinges {tag} (frame half)", "shear_x")
        rest = sum(mass[f"{tag}_{n}"] for n in ("hinge_flex", "holddown_post", "deploy_sensor_stop", "burn_board")) + \
            mass.get(f"{tag}_springs", 0.0) + \
            sum(mass[n] for n in mass if n.startswith(f"{tag}_hinge_doubler"))
        lump_near(rest, [(s_ * W2, hy, hz0), (s_ * W2, hdy, hdz), (s_ * W2, hy, hz1)], f"wing hardware {tag} on frame",
                  "shear_x")
    set_density(bgroups["columns"], mass["stack_columns"], "stack columns", beams=True)
    set_density(bgroups["centre_support"], mass["centre_supports"], "centre supports", beams=True)
    rep["battery rigid body"] = m_tot * 1e3
    rep["battery isolators"] = mass["battery_isolators"]
    # distributed: fasteners + staking over the boards and the frame, thermal over the boards
    dist = mass["fasteners_staking_distributed"]
    boards = groups["EPS"] + groups["OBC"] + groups["carrier"] + groups["RF"]
    nsm_to(boards, dist * 0.6, "fasteners, staking (boards)")
    nsm_to(frame_sh, dist * 0.4, "fasteners, staking (frame)")
    nsm_to(boards, mass["thermal_distributed"], "thermal (boards)")
    # board support sets (for the Steinberg relative displacement)
    supports, bnodes = {}, {}
    for g_, z in (("EPS", zb["EPS"]), ("OBC", zb["OBC"]), ("carrier", zb["carrier"]), ("RF", zb["RF"]),
                  ("deck", zb["deck"])):
        pts = [(sx * 40, sy * 43) for sx in (1, -1) for sy in (1, -1)] + [((0, 0) if g_ == "RF" else (0, 21))]
        if cs2 and g_ in (("EPS", "OBC", "carrier") if cs2 == "eps" else ("EPS", "OBC", "carrier", "deck")):
            pts.append((0, -21))
        supports[g_] = [md.node(x * mm, y * mm, z * mm) for x, y in pts]
        bnodes[g_] = sorted({i for k in groups[g_] for i in md.shells[k]["n"]})
    def nearest(pt_mm, group):
        cand = [i for i in range(md.nn()) if nodegroup.get(i, "").startswith(group) and i not in md.mpc]
        XYZ = np.array([md.xyz[i] for i in cand]) / mm
        return cand[int(np.argmin(np.linalg.norm(XYZ - np.array(pt_mm), axis=1)))]
    probes = {"GNSS receiver mounts (deck, piNAV corners)": [nearest((x, y, -16.0), "deck") for x, y in
                                                             ((-17, -42), (27, -42), (-17, 2), (27, 2))],
              "Z rod clips (-X shear panel)": [nearest(q, "shear_x") for q in zclip],
              "battery CG (rigid body)": [master]}
    special = dict(spiders=spider_count, probes=probes, iso_beams=iso_beams, ear_beams=ear_beams, battery_master=master, battery_mass=m_tot,
                   battery_I=Ic, colsec=colsec, isosec=isosec, ti_col_beams=ti_beams, wing_shells=wing_shells, mass_total_cad=sum(mass.values()),
                   gyro_nodes=gyro_nodes, board_t=special_t,
                   window_edge_nodes=sorted({i for k in groups["carrier"] for i in md.shells[k]["n"]
                                             if cw[0] - 5.1 <= md.xyz[i][0] / mm <= cw[1] + 5.1
                                             and cw[2] - 5.1 <= md.xyz[i][1] / mm <= cw[3] + 5.1}))
    return SC(md, groups, bgroups, nodegroup, supports, bnodes, special, rep)


def apply_overrides(md: Model):
    for e in md.shells:
        if "rho_override" in e or "E_factor" in e:
            m = dict(MAT[e["mat"]])
            if "rho_override" in e:
                m["rho"] = e["rho_override"]
            if "E_factor" in e:
                m["E"] *= e["E_factor"]
            key = f"{e['mat']}_{id(e)}"
            MAT[key] = m
            e["mat"] = key


# ======================================================================================================
# Analyses
# ======================================================================================================
def psd_mpe(f, plus3db=True):
    """SpaceX Rideshare UG v11 Table 4-6 MPE (g^2/Hz), log-log interpolation; +3 dB doubles the PSD."""
    bp = np.array([[20, 0.01], [50, 0.015], [700, 0.015], [800, 0.03], [925, 0.03], [2000, 0.00644]])
    lf, la = np.log(bp[:, 0]), np.log(bp[:, 1])
    v = np.exp(np.interp(np.log(f), lf, la))
    return v * (10 ** 0.3 if plus3db else 1.0)


def psd_test(f):
    """Protoflight random test level (docs/environment/ENVIRONMENTAL_TEST_PLAN.md Rev C section 3.1): envelope of the
    UG v11 MPE + 3 dB and the GEVS-B minimum workmanship spectrum, 8.18 Grms.  Log-log interpolation."""
    bp = np.array([[20, 0.0200], [50, 0.0299], [59.9, 0.0299], [80, 0.0400], [500, 0.0400], [668.2, 0.0299],
                   [700, 0.0299], [800, 0.0599], [925, 0.0599], [2000, 0.0128]])
    return np.exp(np.interp(np.log(f), np.log(bp[:, 0]), np.log(bp[:, 1])))


def grms_test():
    f = np.geomspace(20, 2000, 20000)
    return math.sqrt(np.trapezoid(psd_test(f), f))


def grms(plus3db):
    f = np.geomspace(20, 2000, 20000)
    return math.sqrt(np.trapezoid(psd_mpe(f, plus3db), f))


def modal_energy_groups(sc: SC, sol, nm):
    """Kinetic-energy share of each group in mode nm."""
    u = sol.T @ sol.phi[:, nm]
    Mu = sol.M @ u
    e = u * Mu
    n = sc.md.nn()
    by = {}
    for i in range(n):
        g = sc.nodegroup.get(i, "?")
        if i in sc.md.mpc:
            m = sc.md.mpc[i]
            g = sc.nodegroup.get(m, g)
        by[g] = by.get(g, 0.0) + e[6 * i:6 * i + 6].sum()
    tot = sum(by.values())
    return sorted(((k, v / tot) for k, v in by.items()), key=lambda x: -x[1])


def element_stress_ops(sc: SC):
    """Stress operators: for every shell, a 6 x ndof_full sparse map; for beams 8 x ndof_full."""
    md = sc.md
    ops = []
    ndof = 6 * md.nn()
    for k, e in enumerate(md.shells):
        dofs = np.array([6 * i + j for i in e["n"] for j in range(6)])
        ops.append(("shell", k, dofs, e["S"]))
    for k, b in enumerate(md.beams):
        dofs = np.array([6 * i + j for i in b["n"] for j in range(6)])
        ops.append(("beam", k, dofs, b["S"]))
    return ops


def vm_from_comp(s):
    """von Mises from [sx, sy, sxy] (top) and (bottom), arrays (..., 6): max of both surfaces."""
    t = s[..., 0:3]; b = s[..., 3:6]
    f = lambda q: np.sqrt(np.maximum(q[..., 0] ** 2 - q[..., 0] * q[..., 1] + q[..., 1] ** 2 + 3 * q[..., 2] ** 2, 0))
    return np.maximum(f(t), f(b))


def vm_rms_conservative(r):
    """Conservative von Mises from component RMS values (sign of the sx*sy term taken adverse)."""
    t = r[..., 0:3]; b = r[..., 3:6]
    f = lambda q: np.sqrt(q[..., 0] ** 2 + np.abs(q[..., 0] * q[..., 1]) + q[..., 1] ** 2 + 3 * q[..., 2] ** 2)
    return np.maximum(f(t), f(b))


def run(bc="rails", snubber="soft", nmodes=60, verbose=True, **kw):
    sc = build_spacecraft(bc=bc, snubber=snubber, **kw)
    md = sc.md
    apply_overrides(md)
    sol = solve_modes(md, nmodes)
    res: Dict = dict(bc=bc, snubber=snubber, options=kw, spiders=sc.special["spiders"], nodes=md.nn(), shells=len(md.shells), beams=len(md.beams),
                     dofs=int(sol.Kr.shape[0]))
    res["mass_fe_kg"] = float(sol.M.diagonal()[0::6].sum() + sum(r["M6"][0, 0] for r in md.rigid)) \
        if False else None
    # total mass: lumped diag translation x + rigid body + beam consistent mass (x-direction rigid body)
    rx = np.zeros(6 * md.nn()); rx[0::6] = 1.0
    res["mass_fe_g"] = float(rx @ (sol.M @ rx)) * 1e3
    res["mass_cad_g"] = sc.special["mass_total_cad"]
    res["mass_free_g"] = (sol.mtot * 1e3).tolist()
    modes = []
    for k in range(nmodes):
        grp = modal_energy_groups(sc, sol, k)
        modes.append(dict(n=k + 1, f=float(sol.f[k]), meff_g=(sol.meff[k] * 1e3).tolist(),
                          meff_pct=(100 * sol.meff[k] / sol.mtot).tolist(),
                          energy=[(g, round(v, 3)) for g, v in grp[:3]]))
    res["modes"] = [m for m in modes if m["n"] <= 40 or max(m["meff_pct"]) >= 0.5]
    first = {}
    for m in modes:
        g, v = m["energy"][0]
        if v >= 0.4 and g not in first:
            first[g] = dict(mode=m["n"], f=m["f"], share=v)
    res["first_mode_by_group"] = first
    res["cum_meff_pct"] = (100 * sol.meff.sum(axis=0) / sol.mtot).tolist()
    b2k = sol.f <= 2000
    res["cum_meff_pct_2000Hz"] = (100 * sol.meff[b2k].sum(axis=0) / sol.mtot).tolist()
    res["highest_mode_Hz"] = float(sol.f.max())
    if verbose:
        print(f"FE model bc={bc} snubber={snubber}: {md.nn()} nodes, {len(md.shells)} shells, {len(md.beams)} beams, "
              f"{sol.Kr.shape[0]} dofs; mass FE {res['mass_fe_g']:.1f} g vs CAD {res['mass_cad_g']:.1f} g")
        for m in modes[:25]:
            print(f"  mode {m['n']:2d} {m['f']:7.1f} Hz  meff x/y/z {m['meff_pct'][0]:5.1f} {m['meff_pct'][1]:5.1f} "
                  f"{m['meff_pct'][2]:5.1f} %  {m['energy']}")
        print("  cumulative effective mass (first", nmodes, "modes) x/y/z %:", [round(x, 1) for x in res["cum_meff_pct"]])
        print("  first mode by group:", {g: round(v["f"], 1) for g, v in first.items()})
    res["_sc"], res["_sol"] = sc, sol
    return res


def stresses(res, verbose=True):
    """Quasi-static envelope and random 3-sigma stresses at the hot spots; margins."""
    sc, sol = res["_sc"], res["_sol"]
    md = sc.md
    ops = element_stress_ops(sc)
    # --- quasi-static: 10 g axial with 17 g lateral in 8 directions (16 cases) ---
    cases = []
    for ang in range(0, 360, 45):
        for az in (10.0, -10.0):
            cases.append((17 * math.cos(math.radians(ang)), 17 * math.sin(math.radians(ang)), az))
    U = np.column_stack([static(md, sol, c) for c in cases])        # full dofs x cases
    qs = {}
    for kind, k, dofs, S in ops:
        sv = S @ U[dofs, :]
        if kind == "shell":
            qs[(kind, k)] = float(vm_from_comp(sv.T).max())
        else:
            pk = max(beam_peak(sv[:, c])[0] for c in range(sv.shape[1]))
            qs[(kind, k)] = pk
    # beam end forces for joints (isolator, hold-down, columns)
    # --- random: modal superposition, base PSD in each axis, Q = max(10, sqrt f); modal covariance form:
    #     var(c) = s_c^T C s_c with C_ij = Re sum_f H_i conj(H_j) G df (exact for linear superposition) ---
    fn = sol.f
    f = np.unique(np.concatenate([np.geomspace(20, 2000, 2000)] +
                                 [x * np.linspace(0.95, 1.05, 41) for x in fn if 19 < x < 2100]))
    f = f[(f >= 20) & (f <= 2000)]
    w = 2 * math.pi * f
    wn = 2 * math.pi * fn
    Q = np.maximum(10.0, np.sqrt(fn))
    zeta = 1 / (2 * Q)
    H = 1.0 / (wn[None, :] ** 2 - w[:, None] ** 2 + 2j * zeta[None, :] * wn[None, :] * w[:, None])   # nf x nm
    PhiF = sol.T @ sol.phi                                             # full dofs x modes
    G = psd_test(f) * G0 ** 2                                          # (m/s^2)^2/Hz, 8.18 Grms test level
    df = np.gradient(f)
    rows, cols, vals, owner = [], [], [], []
    r0 = 0
    for kind, k, dofs, S in ops:
        nr = S.shape[0]
        rr, cc = np.meshgrid(np.arange(r0, r0 + nr), dofs, indexing="ij")
        rows.append(rr.ravel()); cols.append(cc.ravel()); vals.append(S.ravel())
        owner.append((kind, k, r0, nr))
        r0 += nr
    S_all = sp.csr_matrix((np.concatenate(vals), (np.concatenate(rows), np.concatenate(cols))),
                          shape=(r0, PhiF.shape[0]))
    Sm_all = S_all @ PhiF                                              # comps x modes
    Cs = []
    for ax in range(3):
        Hq = H * (-sol.gamma[:, ax])[None, :]
        C = np.real(Hq.T @ (np.conj(Hq) * (G * df)[:, None]))
        Cs.append(0.5 * (C + C.T))
    rnd = {}
    for ax in range(3):
        var = np.einsum("ij,ij->i", Sm_all @ Cs[ax], Sm_all)
        r = np.sqrt(np.maximum(var, 0))
        for kind, k, a, nr in owner:
            rv = r[a:a + nr]
            if kind == "shell":
                v = float(vm_rms_conservative(rv[None, :])[0])
            else:
                v = max(float(rv[0] + rv[1] + rv[2]), float(rv[4] + rv[5] + rv[6]))
            rnd[(kind, k)] = max(rnd.get((kind, k), 0.0), v)
    rms_disp = {}
    for g_, nodes in sc.board_nodes.items():
        sup = sc.board_supports[g_]
        wz = PhiF[[6 * i + 2 for i in nodes], :]
        ws = PhiF[[6 * i + 2 for i in sup], :].mean(axis=0)
        rel = (wz - ws[None, :])
        var = np.einsum("ij,ij->i", rel @ Cs[2], rel)
        inner = np.array([abs(md.xyz[i][0]) <= 0.0401 and abs(md.xyz[i][1]) <= 0.0431 for i in nodes])
        imax = int(np.argmax(var))
        rms_disp[g_] = (float(np.sqrt(var).max()), float(np.sqrt(var[inner]).max()),
                        (np.array(md.xyz[nodes[imax]]) * 1e3).round(1).tolist())
    res["random_modes_to_Hz"] = float(fn.max())
    # gyro (M-G370PDT0 on the OBC, through the carrier window): absolute acceleration and relative motion
    gn = sc.special["gyro_nodes"]
    gacc = {}
    for ax in range(3):
        Hq = H * (-sol.gamma[:, ax])[None, :]
        for d in range(3):
            rows_ = PhiF[[6 * i + d for i in gn], :].mean(axis=0)
            T = (1.0 if d == ax else 0.0) - (w ** 2) * (Hq @ rows_)
            gacc[f"base{'xyz'[ax]}_resp{'xyz'[d]}"] = float(np.sqrt(np.sum(np.abs(T) ** 2 * G * df)) / G0)
    en = sc.special["window_edge_nodes"]
    rel3 = 0.0
    for ax in range(3):
        for d in range(3):
            rel = PhiF[[6 * i + d for i in en], :] - PhiF[[6 * i + d for i in gn], :].mean(axis=0)[None, :]
            var = np.einsum("ij,ij->i", rel @ Cs[ax], rel)
            rel3 = max(rel3, 3 * float(np.sqrt(np.maximum(var, 0)).max()))
    res["gyro_random"] = dict(grms=gacc, rel_window_3sigma_mm=rel3 * 1e3, input_grms=grms_test())
    # probes: absolute acceleration (max over the probe's nodes) and, for the GNSS receiver, displacement relative to
    # the deck supports and between its mounts (MCAD-12)
    res["probes"] = {}
    dsup = [i for i in sc.board_supports.get("deck", []) if i not in md.mpc]
    for pname, pn in sc.special["probes"].items():
        acc = {}
        for ax in range(3):
            Hq = H * (-sol.gamma[:, ax])[None, :]
            for d in range(3):
                vals = []
                for i in pn:
                    T = (1.0 if d == ax else 0.0) - (w ** 2) * (Hq @ PhiF[6 * i + d, :])
                    vals.append(float(np.sqrt(np.sum(np.abs(T) ** 2 * G * df)) / G0))
                acc[f"base{'xyz'[ax]}_resp{'xyz'[d]}"] = max(vals)
        out_p = dict(grms=acc, grms_max=max(acc.values()), accel_3sigma_g_max=3 * max(acc.values()))
        if pname.startswith("GNSS") and dsup:
            rel_s, rel_m = 0.0, 0.0
            for ax in range(3):
                for d in range(3):
                    rel = PhiF[[6 * i + d for i in pn], :] - PhiF[[6 * i + d for i in dsup], :].mean(axis=0)[None, :]
                    rel_s = max(rel_s, 3 * float(np.sqrt(np.maximum(np.einsum("ij,ij->i", rel @ Cs[ax], rel), 0)).max()))
                    rel2 = PhiF[[6 * i + d for i in pn], :] - PhiF[[6 * i + d for i in pn], :].mean(axis=0)[None, :]
                    rel_m = max(rel_m, 3 * float(np.sqrt(np.maximum(np.einsum("ij,ij->i", rel2 @ Cs[ax], rel2), 0)).max()))
            out_p.update(rel_to_deck_supports_3sigma_mm=rel_s * 1e3, rel_between_mounts_3sigma_mm=rel_m * 1e3)
        res["probes"][pname] = out_p
    # --- hot spots and margins ---
    groups_shell = {}
    for g_, idx in sc.groups.items():
        groups_shell.setdefault(g_, []).extend(idx)
    out = {}

    def report(name, keys, allow_y, allow_u, fsy, fsu, factor=1.0, note=""):
        best = None
        for key in keys:
            sq = qs[key] * factor
            sr = 3 * rnd[key] * factor
            lim = max(sq, sr)
            if best is None or lim > best["limit_MPa"] * 1e6:
                kind, k = key
                ns = md.shells[k]["n"] if kind == "shell" else md.beams[k]["n"]
                loc = (np.mean([md.xyz[i] for i in ns], axis=0) * 1e3).round(1).tolist()
                best = dict(qs_MPa=sq / 1e6, rand3s_MPa=sr / 1e6, limit_MPa=lim / 1e6, elem=list(key), loc_mm=loc,
                            governs="random" if sr > sq else "quasi-static")
        b = best
        b["MS_yield"] = (allow_y / (fsy * b["limit_MPa"] * 1e6) - 1) if allow_y else None
        b["MS_ult"] = allow_u / (fsu * b["limit_MPa"] * 1e6) - 1
        b["note"] = note
        out[name] = b
    al = MAT["Al6061"]
    xr = 47.33e-3
    rails = sc.bgroups["rails"]
    corner = [("beam", k) for k in rails]
    report("rails (L-angle and standoffs, all elements)", corner, al["Fty"], al["Ftu"], 1.25, 1.40)
    near_ends = [("shell", k) for k in sc.groups["endplate_mz"] + sc.groups["endplate_pz"]
                 if any(abs(abs(md.xyz[i][0]) - 0.049) < 1e-6 or abs(abs(md.xyz[i][1]) - 0.049) < 1e-6 or
                        (abs(md.xyz[i][0]) > 0.0414 and abs(md.xyz[i][1]) > 0.0414) for i in md.shells[k]["n"])]
    report("end frames at the rail corners", near_ends, al["Fty"], al["Ftu"], 1.25, 1.40)
    report("end frames (all)", [("shell", k) for k in sc.groups["endplate_mz"] + sc.groups["endplate_pz"]],
           al["Fty"], al["Ftu"], 1.25, 1.40)
    report("side shear panels", [("shell", k) for k in sc.groups["shear_x"] + sc.groups["shear_y"]],
           al["Fty"], al["Ftu"], 1.25, 1.40)
    cs = sc.special["colsec"]
    report("stack columns, Al hex spacer (at the end frames and boards)", [("beam", k) for k in sc.bgroups["columns"]],
           al["Fty"], al["Ftu"], 1.25, 1.40, factor=1.0,
           note="stress from the softened (x 1/3) member's moment on the full section; spacer share")
    tib = set(sc.special.get("ti_col_beams", []))
    if tib:
        report("stack column (+40, +43), Ti-6Al-4V stud thread root", [("beam", k) for k in sorted(tib)],
               MAT["Ti64"]["Fty"], MAT["Ti64"]["Ftu"], 1.25, 1.40, factor=(MAT["Ti64"]["E"] / al["E"]) * (1.195 / 2.89),
               note="stud stress = (E_Ti/E_Al)(c_stud/c_hex) x spacer stress (same curvature)")
    report("stack columns, A286 stud thread root", [("beam", k) for k in sc.bgroups["columns"] if k not in tib],
           MAT["A286"]["Fty"], MAT["A286"]["Ftu"], 1.25, 1.40,
           factor=(MAT["A286"]["E"] / al["E"]) * (1.195 / 2.89),
           note="stud stress = (E_A286/E_Al)(c_stud/c_hex) x spacer stress (same curvature)")
    report("centre supports", [("beam", k) for k in sc.bgroups["centre_support"]], al["Fty"], al["Ftu"], 1.25, 1.40)
    if "carrier_frame" in sc.bgroups:
        report("carrier window frame (Al 6061)", [("beam", k) for k in sc.bgroups["carrier_frame"]], al["Fty"],
               al["Ftu"], 1.25, 1.40)
    isb = sc.special["iso_beams"]
    iso = sc.special["isosec"]
    report("battery isolator, G-10CR bushing (bending + axial)", [("beam", k) for k in isb], None, MAT["G10CR"]["Ftu"],
           None, 2.00, factor=1.0, note="composite at a discontinuity: FS 2.0 ultimate (5001B Table 2)")
    report("battery isolator, Ti-6Al-4V screw", [("beam", k) for k in isb], MAT["Ti64"]["Fty"], MAT["Ti64"]["Ftu"],
           1.25, 1.40, factor=(MAT["Ti64"]["E"] / MAT["G10CR"]["E"]) * (1.0 / 3.0),
           note="screw stress = (E_Ti/E_G10)(c_screw/c_bushing) x bushing stress")
    report("battery bracket ears (Al 6061, root)", [("beam", k) for k in sc.special["ear_beams"]], al["Fty"], al["Ftu"],
           1.25, 1.40)
    report("spreader and flanges", [("shell", k) for k in sc.groups["spreader"]], al["Fty"], al["Ftu"], 1.25, 1.40)
    report("torquer deck (lightened Al, stress on the equivalent plate)", [("shell", k) for k in sc.groups["deck"]],
           al["Fty"], al["Ftu"], 1.25, 1.40, note="equivalent plate: local stress at the lightening holes higher")
    fr4 = MAT["FR4"]
    for g_ in ("EPS", "OBC", "carrier", "RF"):
        report(f"PCB {g_} (FR-4)", [("shell", k) for k in sc.groups[g_]], None, fr4["Ftu"], None, 1.50)
    for g_ in ("W1", "W2"):
        report(f"stowed wing {g_} (FR-4)", [("shell", k) for k in sc.groups[f"wing_{g_}"]], None, fr4["Ftu"], None, 1.50)
    # spring forces: hold-down (line/post normal force), hinge, snubber: QS envelope and random 3 sigma
    hd = {}
    for si, spg in enumerate(md.springs):
        g_ = spg.get("group", "")
        i, j = spg["n"]
        kx = spg["k"][0]
        rel_qs = np.abs(U[6 * j, :] - U[6 * i, :]).max() * kx
        rel = (PhiF[6 * j, :] - PhiF[6 * i, :]) * kx
        v3 = max(3 * math.sqrt(max(float(rel @ Cs[ax] @ rel), 0.0)) for ax in range(3))
        shear = 0.0
        for d in (1, 2):
            if d in spg["k"]:
                shear = max(shear, np.abs(U[6 * j + d, :] - U[6 * i + d, :]).max() * spg["k"][d])
        hd[g_] = dict(normal_qs_N=float(rel_qs), normal_rand3s_N=v3, inplane_qs_N=float(shear))
    # Steinberg
    # Steinberg: Z = 0.00022 B / (c h r sqrt(L)) in, with the board's own thickness h (a thicker board allows less)
    bt = sc.special["board_t"]
    zal = {g_: 0.00022 * 3.78 / (1.75 * (bt[g_] / 25.4) * 1.0 * 1.0) * 25.4e-3 for g_ in rms_disp}   # m
    stb = {g_: dict(z3s_mm=3 * v[0] * 1e3, z3s_inner_mm=3 * v[1] * 1e3, at_mm=v[2], zallow_mm=zal[g_] * 1e3, h_mm=bt[g_],
                    MS=zal[g_] / (3 * v[0]) - 1, MS_inner=zal[g_] / (3 * v[1]) - 1) for g_, v in rms_disp.items()}
    res["stress"] = {k: {kk: (vv if not isinstance(vv, tuple) else list(vv)) for kk, vv in v.items()}
                     for k, v in out.items()}
    res["holddown"] = hd
    res["steinberg"] = stb
    res["grms_mpe"] = grms(False)
    res["grms_mpe3"] = grms(True)
    res["grms_test"] = grms_test()
    if verbose:
        print(f"  random MPE {res['grms_mpe']:.2f} Grms, MPE+3 dB {res['grms_mpe3']:.2f} Grms, analysed at the "
              f"protoflight test envelope {res['grms_test']:.2f} Grms")
        for k, v in out.items():
            ms_y = f"{v['MS_yield']:+.2f}" if v["MS_yield"] is not None else "  -  "
            print(f"  {k:58s} QS {v['qs_MPa']:7.2f}  3s {v['rand3s_MPa']:7.2f}  limit {v['limit_MPa']:7.2f} MPa  "
                  f"MS_y {ms_y}  MS_u {v['MS_ult']:+.2f}  at {v['loc_mm']}")
        for k, v in hd.items():
            print(f"  {k}: normal QS {v['normal_qs_N']:.2f} N, random 3s {v['normal_rand3s_N']:.2f} N, "
                  f"in-plane QS {v['inplane_qs_N']:.2f} N")
        for k, v in stb.items():
            print(f"  Steinberg {k}: Z3s {v['z3s_mm']:.4f} mm (at {v['at_mm']}), inside the columns "
                  f"{v['z3s_inner_mm']:.4f} mm, allowable {v['zallow_mm']:.3f} mm, MS {v['MS']:+.2f} / {v['MS_inner']:+.2f}")
    return res


# design (SQSO-MECH-CAD-001 B, D-11 as amended): gyro window in a 2.0 mm carrier, no frame; Ti stud at (+40, +43)
_D = dict(bc="rails", snubber="soft", cs2="eps", t_car=2.0, car_frame=False)
CASES = {
    "design": dict(_D),
    "single_centre_support": dict(_D, cs2=False),
    "design_2.0mm_EPS_carrier": dict(_D, t_eps=2.0),
    "design_second_holddown": dict(_D, snubber="hard"),
    "design_rail_ends_only": dict(_D, bc="ends"),
    "alt_1.6mm_carrier_framed_window": dict(_D, t_car=1.6, car_frame=True),
    "alt_1.6mm_carrier_window_no_frame": dict(_D, t_car=1.6),
    "reference_1.6mm_carrier_no_window": dict(_D, t_car=1.6, car_window=False),
    "design_MEV": dict(_D, grown=True),                                   # P3S-02: every MEL row at its growth
    "design_2.0mm_EPS_MEV": dict(_D, t_eps=2.0, grown=True),
    "design_harness_on_EPS": dict(_D, harness_tie="eps"),               # C-15 bounding case (P3S-09)
}
NMODES = 450


def _case(name):
    kw = dict(CASES[name])
    bc, sn = kw.pop("bc"), kw.pop("snubber")
    r = run(bc, sn, nmodes=NMODES, verbose=False, **kw)
    r = stresses(r, verbose=False)
    return name, {k: v for k, v in r.items() if not k.startswith("_")}, (r["_sc"].mass_report,
                                                                          r["_sc"].special["battery_mass"],
                                                                          r["_sc"].special["battery_I"].tolist())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest-only", action="store_true")
    ap.add_argument("--json", default=os.path.join(HERE, "fem_results.json"))
    ap.add_argument("--cases", default=",".join(CASES))
    ap.add_argument("--procs", type=int, default=3)
    args = ap.parse_args()
    out = {"selftest": selftest(verbose=True)}
    if args.selftest_only:
        return
    names = [c for c in args.cases.split(",") if c]
    import multiprocessing as mpr
    with mpr.get_context("fork").Pool(min(args.procs, len(names))) as pool:
        results = pool.map(_case, names)
    out["cases"] = {}
    for name, r, extra in results:
        out["cases"][name] = r
        if name == "design":
            out["mass_report_g"], m, I = extra
            out["battery"] = dict(mass_g=m * 1e3, I_kgm2=I)
        print(f"== {name}: first modes " + ", ".join(f"{m['f']:.0f}" for m in r["modes"][:8]) + " Hz")
        print("   first mode by group:", {g: round(v["f"], 1) for g, v in r["first_mode_by_group"].items()})
        for k, v in r["stress"].items():
            ms_y = f"{v['MS_yield']:+.2f}" if v["MS_yield"] is not None else "  -  "
            print(f"   {k:58s} QS {v['qs_MPa']:7.2f} 3s {v['rand3s_MPa']:7.2f} MPa  MS_y {ms_y} MS_u {v['MS_ult']:+.2f}"
                  f"  at {v['loc_mm']}")
        for k, v in r["steinberg"].items():
            print(f"   Steinberg {k:8s} Z3s {v['z3s_mm']:.4f} mm (inner {v['z3s_inner_mm']:.4f}) vs {v['zallow_mm']:.3f}: "
                  f"MS {v['MS']:+.2f} / {v['MS_inner']:+.2f} at {v['at_mm']}")
        for k, v in r["holddown"].items():
            print(f"   {k}: normal QS {v['normal_qs_N']:.2f} N, 3s {v['normal_rand3s_N']:.2f} N")
        print("   gyro random:", {k: round(v, 2) if isinstance(v, float) else v for k, v in r["gyro_random"].items()})
        for pk, pv in r.get("probes", {}).items():
            print(f"   probe {pk}: Grms max {pv['grms_max']:.1f} (3s {pv['accel_3sigma_g_max']:.0f} g)",
                  {k: round(v, 3) for k, v in pv.items() if k.startswith("rel")},
                  {k: round(v, 1) for k, v in pv['grms'].items()})
    with open(args.json, "w") as f:
        json.dump(out, f, indent=1, default=lambda x: float(x) if isinstance(x, (np.floating, np.integer)) else str(x))


if __name__ == "__main__":
    main()
