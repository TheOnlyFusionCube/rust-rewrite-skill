"""HTTPException — FastAPI-shaped."""

from __future__ import annotations

from typing import Any


class HTTPException(Exception):
    def __init__(self, status_code: int, detail: Any = None) -> None:
        self.status_code = int(status_code)
        self.detail = detail if detail is not None else "Error"
        super().__init__(self.detail)
