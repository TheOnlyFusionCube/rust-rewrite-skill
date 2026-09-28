"""Config table / lookup API."""

from __future__ import annotations

from typing import Any


_TRUE = {"true", "yes", "1", "on"}
_FALSE = {"false", "no", "0", "off"}


class Config:
    def __init__(self) -> None:
        self._data: dict[str, dict[str, str]] = {}

    def ensure_section(self, section: str) -> None:
        self._data.setdefault(section, {})

    def set(self, section: str, key: str, value: str) -> None:
        self.ensure_section(section)
        self._data[section][key] = value

    def has_section(self, section: str) -> bool:
        return section in self._data

    def get(self, section: str, key: str, default: Any = None) -> Any:
        if section not in self._data:
            return default
        return self._data[section].get(key, default)

    def get_bool(self, section: str, key: str, default: bool = False) -> bool:
        raw = self.get(section, key, None)
        if raw is None:
            return default
        s = str(raw).strip().lower()
        if s in _TRUE:
            return True
        if s in _FALSE:
            return False
        return default

    def get_int(self, section: str, key: str, default: int = 0) -> int:
        raw = self.get(section, key, None)
        if raw is None:
            return default
        try:
            return int(str(raw).strip())
        except ValueError:
            return default
