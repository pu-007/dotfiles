#!/usr/bin/env python3
"""Thin launcher for `python -m wots`."""
import runpy, sys

if __name__ == "__main__":
    sys.argv[0] = "wots"
    runpy.run_module("wots", run_name="__main__")
