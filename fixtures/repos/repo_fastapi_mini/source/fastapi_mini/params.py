"""Path / Query / Body / Depends parameter markers."""

from __future__ import annotations

from typing import Any, Callable


class ParamBase:
    kind: str = "param"

    def __init__(
        self,
        default: Any = ...,
        *,
        alias: str | None = None,
        required: bool | None = None,
    ) -> None:
        self.default = default
        self.alias = alias
        if required is None:
            self.required = default is ...
        else:
            self.required = bool(required)


class Path(ParamBase):
    kind = "path"

    def __init__(self, default: Any = ..., *, alias: str | None = None) -> None:
        # Path params are always required in this mini subset
        super().__init__(default=default, alias=alias, required=True)


class Query(ParamBase):
    kind = "query"


class Body(ParamBase):
    kind = "body"


class Depends:
    kind = "depends"

    def __init__(self, dependency: Callable[..., Any]) -> None:
        self.dependency = dependency
