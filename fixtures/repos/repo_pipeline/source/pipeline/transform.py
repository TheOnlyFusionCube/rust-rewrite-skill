"""Filter and aggregate rows."""

from __future__ import annotations


def parse_line(line: str) -> tuple[str, int] | None:
    s = line.strip()
    if not s or s.startswith("#"):
        return None
    if "," not in s:
        return None
    key, _, rest = s.partition(",")
    key = key.strip()
    rest = rest.strip()
    if not key:
        return None
    try:
        value = int(rest)
    except ValueError:
        return None
    return key, value


def filter_rows(rows: list[tuple[str, int]]) -> list[tuple[str, int]]:
    return [(k, v) for k, v in rows if v > 0]


def aggregate(rows: list[tuple[str, int]]) -> list[tuple[str, int]]:
    totals: dict[str, int] = {}
    for k, v in rows:
        totals[k] = totals.get(k, 0) + v
    return sorted(totals.items(), key=lambda kv: kv[0])


def run_pipeline(lines: list[str]) -> list[tuple[str, int]]:
    parsed: list[tuple[str, int]] = []
    for line in lines:
        item = parse_line(line)
        if item is not None:
            parsed.append(item)
    return aggregate(filter_rows(parsed))
