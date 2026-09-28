"""INI-like text parser."""

from __future__ import annotations

from .table import Config


def parse(text: str) -> Config:
    cfg = Config()
    section: str | None = None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith(";"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            cfg.ensure_section(section)
            continue
        if "=" not in line:
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        value = value.strip()
        if section is None:
            section = "DEFAULT"
            cfg.ensure_section(section)
        cfg.set(section, key, value)
    return cfg
