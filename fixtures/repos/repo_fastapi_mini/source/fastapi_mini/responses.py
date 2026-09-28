"""Response / JSONResponse."""

from __future__ import annotations

import json
from typing import Any, Mapping


class Response:
    def __init__(
        self,
        content: str | bytes = "",
        status_code: int = 200,
        headers: Mapping[str, str] | None = None,
        media_type: str | None = None,
    ) -> None:
        if isinstance(content, bytes):
            self.body = content
        else:
            self.body = content.encode("utf-8")
        self.status_code = int(status_code)
        self.headers: dict[str, str] = dict(headers or {})
        if media_type:
            self.headers.setdefault("content-type", media_type)

    @property
    def text(self) -> str:
        return self.body.decode("utf-8")


class JSONResponse(Response):
    def __init__(
        self,
        content: Any = None,
        status_code: int = 200,
        headers: Mapping[str, str] | None = None,
    ) -> None:
        body = json.dumps(content, ensure_ascii=False, separators=(",", ":"))
        hdrs = dict(headers or {})
        hdrs.setdefault("content-type", "application/json")
        super().__init__(content=body, status_code=status_code, headers=hdrs)
