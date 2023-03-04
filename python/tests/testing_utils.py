import time
import sys

import polib as pypolib
import rspolib

REPS = 10000


def run_polib(cb, impl, polib, reps=REPS):
    start = time.time() * 1000
    for _ in range(reps):
        cb(polib)
    end = time.time() * 1000
    sys.stdout.write(f"{impl} {end - start} ms\n")


def run_polibs(*args):
    opts = {}
    if len(args) == 1:
        cb = args[0]
    elif len(args) == 2:
        opts, cb = args
    else:
        cb = args

    if not isinstance(cb, list):
        cbs = [cb]
    else:
        cbs = cb

    for cb in cbs:
        sys.stdout.write("\n")
        sys.stdout.write(f"{cb.__name__}\n")
        if opts.get("polib", True):
            run_polib(
                cb,
                "    polib",
                pypolib,
                reps=opts.get("reps", REPS),
            )
        if opts.get("rspolib", True):
            run_polib(
                cb,
                "  rspolib",
                rspolib,
                reps=opts.get("reps", REPS),
            )
