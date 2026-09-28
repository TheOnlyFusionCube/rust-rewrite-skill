"""Read CSV-like input and write aggregated output."""

from __future__ import annotations

from pathlib import Path


def read_rows(path: str) -> list[str]:
    text = Path(path).read_text(encoding="utf-8")
    return text.splitlines()


def write_aggregates(path: str, items: list[tuple[str, int]]) -> None:
    lines = [f"{k},{v}" for k, v in items]
    Path(path).write_text("\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
