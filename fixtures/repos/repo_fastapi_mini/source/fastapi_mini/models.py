"""response_model field filtering helpers."""

from __future__ import annotations

from typing import Any, Iterable


def response_fields(*fields: str) -> tuple[str, ...]:
    """Declare allowed response fields (order preserved). Used as response_model."""
    return tuple(fields)


def filter_response(data: Any, fields: Iterable[str] | None) -> Any:
    if fields is None:
        return data
    field_list = list(fields)
    if isinstance(data, dict):
        return {k: data[k] for k in field_list if k in data}
    if isinstance(data, list):
        return [filter_response(item, field_list) for item in data]
    return data
