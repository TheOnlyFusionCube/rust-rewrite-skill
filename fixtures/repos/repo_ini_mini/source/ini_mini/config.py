"""INI-like config: parse, typed getters, stringify.

Quirk contract is documented in README.md — keep in sync with Rust tests.
"""

from __future__ import annotations

from typing import OrderedDict as TypingOrderedDict  # noqa: F401 — clarity
from collections import OrderedDict


_TRUE = frozenset({"true", "yes", "1", "on"})
_FALSE = frozenset({"false", "no", "0", "off"})


class IniError(Exception):
    """Structured INI error with exact message shapes."""

    def __init__(self, kind: str, message: str, **fields: object) -> None:
        super().__init__(message)
        self.kind = kind
        self.message = message
        self.fields = fields

    def __str__(self) -> str:
        return self.message

    def __repr__(self) -> str:
        return f"IniError(kind={self.kind!r}, message={self.message!r})"


def _unescape(value: str) -> str:
    """Process backslash escapes: \\\\ \\n \\t \\# \\; and unknown \\X → X."""
    out: list[str] = []
    i = 0
    while i < len(value):
        c = value[i]
        if c == "\\" and i + 1 < len(value):
            n = value[i + 1]
            if n == "n":
                out.append("\n")
            elif n == "t":
                out.append("\t")
            elif n == "\\":
                out.append("\\")
            elif n == "#":
                out.append("#")
            elif n == ";":
                out.append(";")
            else:
                out.append(n)
            i += 2
            continue
        out.append(c)
        i += 1
    return "".join(out)


def _escape(value: str) -> str:
    """Inverse of _unescape for stringify round-trip."""
    out: list[str] = []
    for c in value:
        if c == "\\":
            out.append("\\\\")
        elif c == "\n":
            out.append("\\n")
        elif c == "\t":
            out.append("\\t")
        elif c == "#":
            out.append("\\#")
        elif c == ";":
            out.append("\\;")
        else:
            out.append(c)
    return "".join(out)


class Config:
    def __init__(self) -> None:
        # section -> OrderedDict key -> value (insertion order; duplicate keys last-wins)
        self._data: OrderedDict[str, OrderedDict[str, str]] = OrderedDict()

    def ensure_section(self, section: str) -> None:
        if section not in self._data:
            self._data[section] = OrderedDict()

    def set(self, section: str, key: str, value: str) -> None:
        self.ensure_section(section)
        # last-wins: re-insert moves to end in OrderedDict? In Python 3.7+
        # assignment keeps position. Task: last-wins for *value*; order of first
        # appearance kept for stringify (inih-ish). So update in place.
        if key in self._data[section]:
            self._data[section][key] = value
        else:
            self._data[section][key] = value

    def sections(self) -> list[str]:
        return list(self._data.keys())

    def keys(self, section: str) -> list[str]:
        if section not in self._data:
            raise IniError(
                "missing_section",
                f"section '{section}' not found",
                section=section,
            )
        return list(self._data[section].keys())

    def has_section(self, section: str) -> bool:
        return section in self._data

    def has_key(self, section: str, key: str) -> bool:
        return section in self._data and key in self._data[section]

    def get_str(self, section: str, key: str) -> str:
        if section not in self._data:
            raise IniError(
                "missing_section",
                f"section '{section}' not found",
                section=section,
            )
        if key not in self._data[section]:
            raise IniError(
                "missing_key",
                f"key '{key}' not found in section '{section}'",
                section=section,
                key=key,
            )
        return self._data[section][key]

    def get_str_or(self, section: str, key: str, default: str) -> str:
        try:
            return self.get_str(section, key)
        except IniError:
            return default

    def get_int(self, section: str, key: str) -> int:
        raw = self.get_str(section, key)
        s = raw.strip()
        if not s or not _is_int_token(s):
            raise IniError(
                "bad_int",
                f"value '{raw}' for '{section}.{key}' is not a valid integer",
                section=section,
                key=key,
                value=raw,
            )
        return int(s, 10)

    def get_int_or(self, section: str, key: str, default: int) -> int:
        try:
            return self.get_int(section, key)
        except IniError as e:
            if e.kind in ("missing_section", "missing_key", "bad_int"):
                return default
            raise

    def get_bool(self, section: str, key: str) -> bool:
        raw = self.get_str(section, key)
        s = raw.strip().lower()
        if s in _TRUE:
            return True
        if s in _FALSE:
            return False
        raise IniError(
            "bad_bool",
            f"value '{raw}' for '{section}.{key}' is not a valid boolean",
            section=section,
            key=key,
            value=raw,
        )

    def get_bool_or(self, section: str, key: str, default: bool) -> bool:
        try:
            return self.get_bool(section, key)
        except IniError as e:
            if e.kind in ("missing_section", "missing_key", "bad_bool"):
                return default
            raise

    def to_string(self) -> str:
        """Serialize: [section]\\nkey = value\\n with blank line between sections."""
        parts: list[str] = []
        for i, (sec, kv) in enumerate(self._data.items()):
            if i > 0:
                parts.append("")
            parts.append(f"[{sec}]")
            for k, v in kv.items():
                parts.append(f"{k} = {_escape(v)}")
        if not parts:
            return ""
        return "\n".join(parts) + "\n"


def _is_int_token(s: str) -> bool:
    if not s:
        return False
    if s[0] in "+-":
        return len(s) > 1 and s[1:].isdigit()
    return s.isdigit()


def parse(text: str) -> Config:
    """Parse INI-like text. Raises IniError(kind='parse', ...) on hard errors."""
    cfg = Config()
    section: str | None = None
    for lineno, raw in enumerate(text.splitlines(), start=1):
        # Do not strip yet — need to detect comment after leading ws only
        stripped = raw.strip()
        if not stripped:
            continue
        if stripped.startswith("#") or stripped.startswith(";"):
            continue
        if stripped.startswith("["):
            if "]" not in stripped:
                raise IniError(
                    "parse",
                    f"unclosed section on line {lineno}",
                    line=lineno,
                )
            close = stripped.index("]")
            name = stripped[1:close].strip()
            rest = stripped[close + 1 :].strip()
            if rest:
                raise IniError(
                    "parse",
                    f"section header garbage on line {lineno}",
                    line=lineno,
                )
            section = name
            cfg.ensure_section(section)  # duplicate section → merge
            continue
        if "=" not in stripped:
            # lenient: ignore non-assignment lines
            continue
        key, _, value = stripped.partition("=")
        key = key.strip()
        value = value.strip()
        value = _unescape(value)
        if not key:
            raise IniError(
                "parse",
                f"empty key on line {lineno}",
                line=lineno,
            )
        if section is None:
            section = "DEFAULT"
            cfg.ensure_section(section)
        cfg.set(section, key, value)
    return cfg
