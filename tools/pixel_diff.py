#!/usr/bin/env python3
"""Pixel-exact screenshot diff for regression checks.

Usage:
    python tools/pixel_diff.py before.png after.png [--mask x,y,w,h ...]

Prints the number of differing pixels and a bounding box. Exits with status 1
when any pixel outside the masked regions differs, so it can gate a build.
Requires Pillow and numpy.
"""

import argparse
import sys

import numpy as np
from PIL import Image


def load(path: str) -> np.ndarray:
    return np.asarray(Image.open(path).convert("RGBA"), dtype=np.int16)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("before")
    ap.add_argument("after")
    ap.add_argument(
        "--mask",
        action="append",
        default=[],
        metavar="X,Y,W,H",
        help="ignore a rectangle (for example the debug overlay); repeatable",
    )
    args = ap.parse_args()

    a, b = load(args.before), load(args.after)
    if a.shape != b.shape:
        print(f"size mismatch: {a.shape[1]}x{a.shape[0]} vs {b.shape[1]}x{b.shape[0]}")
        return 1

    diff = np.abs(a - b).max(axis=2)
    for m in args.mask:
        x, y, w, h = (int(v) for v in m.split(","))
        diff[max(0, y) : y + h, max(0, x) : x + w] = 0

    ys, xs = np.nonzero(diff)
    if len(xs) == 0:
        print(f"OK: identical outside masks ({diff.size} pixels checked)")
        return 0

    print(
        f"DIFF: {len(xs)} pixels, max {int(diff.max())}, "
        f"bbox ({xs.min()},{ys.min()})-({xs.max()},{ys.max()})"
    )
    worst = int(np.argmax(diff))
    y, x = divmod(worst, diff.shape[1])
    print(f"worst at {x},{y}: before={tuple(a[y, x])} after={tuple(b[y, x])}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
