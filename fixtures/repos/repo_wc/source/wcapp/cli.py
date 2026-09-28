"""CLI entry for wcapp."""

from __future__ import annotations

import sys

from .counts import count_file


def format_counts(lines: int, words: int, nbytes: int, label: str) -> str:
    return f"{lines} {words} {nbytes} {label}"


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if not args:
        print("usage: wcapp FILE [FILE ...]", file=sys.stderr)
        return 2
    for path in args:
        lines, words, nbytes = count_file(path)
        print(format_counts(lines, words, nbytes, path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
