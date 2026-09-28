"""TestClient — in-process client mirroring what Rust must expose."""

from __future__ import annotations

import json
from typing import Any, Mapping
from urllib.parse import urlencode

from .app import App
from .responses import Response


class ClientResponse:
    def __init__(self, response: Response) -> None:
        self._response = response
        self.status_code = response.status_code
        self.headers = dict(response.headers)
        self.content = response.body

    @property
    def text(self) -> str:
        return self._response.text

    def json(self) -> Any:
        return json.loads(self.text)


class TestClient:
    def __init__(self, app: App) -> None:
        self.app = app

    def request(
        self,
        method: str,
        url: str,
        *,
        params: Mapping[str, Any] | None = None,
        json_body: Any = None,
        data: str | bytes | None = None,
        headers: Mapping[str, str] | None = None,
    ) -> ClientResponse:
        hdrs = {k.lower(): v for k, v in dict(headers or {}).items()}
        body: str | bytes | None = data
        if json_body is not None:
            body = json.dumps(json_body, ensure_ascii=False, separators=(",", ":"))
            hdrs.setdefault("content-type", "application/json")
        query = None
        if params is not None:
            query = urlencode({k: v for k, v in params.items()}, doseq=True)
        resp = self.app.handle(method, url, query=query, body=body, headers=hdrs)
        return ClientResponse(resp)

    def get(self, url: str, **kwargs: Any) -> ClientResponse:
        return self.request("GET", url, **kwargs)

    def post(self, url: str, **kwargs: Any) -> ClientResponse:
        # accept json= alias
        if "json" in kwargs and "json_body" not in kwargs:
            kwargs["json_body"] = kwargs.pop("json")
        return self.request("POST", url, **kwargs)

    def put(self, url: str, **kwargs: Any) -> ClientResponse:
        if "json" in kwargs and "json_body" not in kwargs:
            kwargs["json_body"] = kwargs.pop("json")
        return self.request("PUT", url, **kwargs)

    def delete(self, url: str, **kwargs: Any) -> ClientResponse:
        return self.request("DELETE", url, **kwargs)

    def patch(self, url: str, **kwargs: Any) -> ClientResponse:
        if "json" in kwargs and "json_body" not in kwargs:
            kwargs["json_body"] = kwargs.pop("json")
        return self.request("PATCH", url, **kwargs)
