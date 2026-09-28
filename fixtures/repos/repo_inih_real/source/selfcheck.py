#!/usr/bin/env python3
"""Rebuild C oracle goldens and diff against frozen golden/*.events."""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ORACLE_DIR = ROOT / "oracle"
GOLDEN = ROOT / "golden"
FIXTURES = ROOT / "fixtures"
DEFINES = ["-DINI_MAX_LINE=1024", "-DINI_MAX_SECTION=256", "-DINI_MAX_NAME=256"]


def build_oracle() -> Path:
    out = ORACLE_DIR / "oracle_dump"
    cmd = [
        "cc",
        "-O2",
        *DEFINES,
        "-o",
        str(out),
        str(ORACLE_DIR / "oracle_dump.c"),
        str(ROOT / "upstream" / "ini.c"),
    ]
    subprocess.check_call(cmd)
    return out


def main() -> int:
    oracle = build_oracle()
    # Curated numbered fixtures + selected upstream_* used by tests/goldens
    names = sorted(p.name for p in FIXTURES.glob("[0-9]*.ini"))
    names += [
        "upstream_normal.ini",
        "upstream_multi_line.ini",
        "upstream_bad_section.ini",
        "upstream_no_value.ini",
    ]
    failed = []
    for name in names:
        ini = FIXTURES / name
        if not ini.is_file():
            failed.append(f"missing fixture {name}")
            continue
        golden = GOLDEN / (ini.stem + ".events")
        got = subprocess.check_output([str(oracle), str(ini)], text=True)
        if not golden.is_file():
            failed.append(f"missing golden {golden.name}")
            continue
        want = golden.read_text()
        if got != want:
            failed.append(f"MISMATCH {name}\n--- want ---\n{want}--- got ---\n{got}")
    if failed:
        print("SELFCHECK FAIL:")
        for f in failed:
            print(f)
        return 1
    print(f"SELFCHECK OK: {len(names)} fixtures match golden/ (oracle rebuilt)")
    # Sanity: stub should not be required here
    pin = (ROOT / "upstream" / "COMMIT_SHA").read_text().strip()
    print(f"pinned inih commit: {pin}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
