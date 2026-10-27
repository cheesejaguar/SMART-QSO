"""Shared helpers for the SQSO-HW-DIAG-001 schematics (schemdraw, SVG backend).

Each S*.py script draws one sheet and writes <name>.svg into the directory given as its first argument
(default: ../svg). Conventions on every sheet:
  - a part not selected is drawn as a dashed box and labelled as not selected
  - a value not confirmed is labelled as unconfirmed
  - a net that leaves the sheet ends in a flag with its net name (the same names as the harnesses W1-W10)
"""

import os
import re
import sys

import schemdraw
import schemdraw.elements as elm

schemdraw.use("svg")

TBD = dict(ls="--", color="#555555")


def out_path(name):
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "..", "svg")
    os.makedirs(out, exist_ok=True)
    return os.path.join(out, name + ".svg")


def drawing(name):
    d = schemdraw.Drawing(file=out_path(name), show=False)
    d.config(fontsize=9, unit=2.2, lw=1.1, font="Helvetica")
    return d


def anchor(name):
    """IC pin anchor: the pin name made an identifier ("VC0 (cell 1-)" -> "VC0", "IN+" -> "INP", "IN-" -> "INN")."""
    word = name.split(" ")[0].lstrip("+")
    word = re.sub(r"\+$", "P", re.sub(r"-$", "N", word))
    return re.sub(r"\W", "_", word).strip("_") or "p"


def ic(label, left=(), right=(), top=(), bottom=(), size=None, tbd=False, pinspacing=0.7, spread=()):
    """A block with named pins. Pins are reachable as anchors, e.g. u.VIN.

    Left and right pins are listed bottom to top; top and bottom pins left to right. A side named in `spread`
    has its pins spaced evenly along the whole side instead of at `pinspacing`.
    """
    pins = []
    for side, names in (("left", left), ("right", right), ("top", top), ("bottom", bottom)):
        for i, n in enumerate(names):
            pins.append(elm.IcPin(name=n, side=side, anchorname=anchor(n)))
    if size is None:
        rows = max(len(left), len(right), 1)
        cols = max(len(top), len(bottom), 1)
        side_w = 0.13 * (max(map(len, left), default=0) + max(map(len, right), default=0))
        label_w = 0.13 * max(len(line) for line in label.split("\n"))
        lines = label.count("\n") + 1
        size = (max(3.0, side_w + label_w + 0.8, 1.2 * cols + 1.0),
                max(1.6, pinspacing * rows + 0.6, 0.45 * lines + 0.9))
    e = elm.Ic(pins=pins, size=size, pinspacing=pinspacing, edgepadW=0.4, edgepadH=0.4, lsize=8, plblsize=7).right()
    for side in spread:
        names = {"left": left, "right": right, "top": top, "bottom": bottom}[side]
        span = size[1] if side in ("left", "right") else size[0]
        if len(names) > 1:
            e.side(side[0].upper(), spacing=(span - 1.2) / (len(names) - 1), pad=0.6, label_size=8, pinlabel_size=7)
    e.label(label, "center", fontsize=9)
    if tbd:
        e.style(ls="--")
    return e


def flag(d, at, name, direction="right", length=0.8):
    """A short stub from `at` ending in an off-sheet flag named `name`."""
    line = elm.Line().at(at).length(length)
    getattr(line, direction)()
    d += line
    tag = elm.Tag(width=max(1.0, 0.16 * len(name) + 0.3), height=0.45).label(name, fontsize=8)
    getattr(tag, direction)()
    d += tag.at(line.end)
    return line.end


def note(d, xy, text, fontsize=8, color="#333333"):
    d += elm.Label().at(xy).label(text, fontsize=fontsize, color=color, halign="left", valign="top")


def title(d, xy, sheet, text, sources):
    note(d, xy, f"{sheet}  {text}", fontsize=12, color="black")
    note(d, (xy[0], xy[1] - 0.6),
         "SQSO-HW-DIAG-001 · dashed = part not selected · values from the board build-to designs\n" + sources,
         fontsize=8)


def pin(u, name):
    """The anchor of pin `name` on block `u` (same naming as ic())."""
    return getattr(u, anchor(name))
