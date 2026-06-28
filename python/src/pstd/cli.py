from __future__ import annotations

import argparse
import os
import shutil
import sys


def find_pstd_binary() -> str | None:
    configured = os.environ.get("PSTD_BINARY")
    if configured:
        return configured
    return shutil.which("pstd")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="python -m pstd")
    parser.add_argument("args", nargs=argparse.REMAINDER)
    parsed = parser.parse_args(argv)

    binary = find_pstd_binary()
    if not binary:
        print("pstd Rust binary was not found. Set PSTD_BINARY or build the Rust binary first.", file=sys.stderr)
        return 2

    argv_out = [binary, *parsed.args]
    return os.spawnv(os.P_WAIT, binary, argv_out)
