"""CLI: pipeline INPUT OUTPUT"""

from __future__ import annotations

import sys

from .io import read_rows, write_aggregates
from .transform import run_pipeline


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if len(args) != 2:
        print("usage: pipeline INPUT OUTPUT", file=sys.stderr)
        return 2
    inp, outp = args
    lines = read_rows(inp)
    items = run_pipeline(lines)
    write_aggregates(outp, items)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
