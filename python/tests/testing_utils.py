import time
import sys

import polib as pypolib
import rspolib

REPS = 50


def run_cb(cb, impl, polib, reps=REPS):
    start = time.time() * 1000
    for _ in range(reps):
        cb(polib)
    end = time.time() * 1000
    sys.stdout.write(f"{impl} {end - start} ms\n")


def run_polibs(*cbs, run_polib=True, run_rspolib=True, reps=REPS):
    for cb in cbs:
        if isinstance(cb, tuple):
            polib_cb, rspolib_cb = cb
        else:
            polib_cb = rspolib_cb = cb

        iter_name = (
            polib_cb.__name__.replace("pypolib_", "")
            .replace("rspolib_", "")
            .replace("polib_", "")
        )

        sys.stdout.write("\n")
        sys.stdout.write(f"{iter_name}\n")
        if run_polib:
            run_cb(
                polib_cb,
                "    polib",
                pypolib,
                reps=reps,
            )
        if run_rspolib:
            run_cb(
                rspolib_cb,
                "  rspolib",
                rspolib,
                reps=reps,
            )
