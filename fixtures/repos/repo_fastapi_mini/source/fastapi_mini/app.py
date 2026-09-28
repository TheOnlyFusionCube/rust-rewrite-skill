"""App / APIRouter — FastAPI-shaped mini framework (subset)."""

from __future__ import annotations

import inspect
import json
import re
from typing import Any, Callable
from urllib.parse import parse_qs

from .exceptions import HTTPException
from .models import filter_response
from .params import Body, Depends, Path, Query
from .responses import JSONResponse, Response

_PATH_PARAM_RE = re.compile(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}")


def _normalize_path(path: str) -> str:
    if not path.startswith("/"):
        path = "/" + path
    if len(path) > 1 and path.endswith("/"):
        path = path[:-1]
    return path


def _compile_path(path: str) -> tuple[re.Pattern[str], list[str]]:
    names: list[str] = []
    parts: list[str] = []
    i = 0
    for m in _PATH_PARAM_RE.finditer(path):
        parts.append(re.escape(path[i : m.start()]))
        name = m.group(1)
        names.append(name)
        parts.append(f"(?P<{name}>[^/]+)")
        i = m.end()
    parts.append(re.escape(path[i:]))
    pattern = "^" + "".join(parts) + "$"
    return re.compile(pattern), names


def _annotation_type_name(annotation: Any) -> str:
    if annotation is inspect.Parameter.empty or annotation is None:
        return "string"
    if annotation is int:
        return "integer"
    if annotation is float:
        return "number"
    if annotation is bool:
        return "boolean"
    if annotation is str:
        return "string"
    if annotation is dict or annotation is list:
        return "object" if annotation is dict else "array"
    # typing generics / strings
    name = getattr(annotation, "__name__", None) or str(annotation)
    low = name.lower()
    if "int" in low:
        return "integer"
    if "float" in low or "number" in low:
        return "number"
    if "bool" in low:
        return "boolean"
    if "dict" in low or "mapping" in low:
        return "object"
    if "list" in low or "sequence" in low:
        return "array"
    return "string"


def _coerce(value: str, type_name: str, loc: list[Any]) -> Any:
    if type_name == "integer":
        try:
            # reject floats-as-strings like "1.5"
            if re.fullmatch(r"[+-]?\d+", value) is None:
                raise ValueError
            return int(value)
        except ValueError:
            raise _validation_error(loc, "value is not a valid integer", "type_error.integer")
    if type_name == "number":
        try:
            return float(value)
        except ValueError:
            raise _validation_error(loc, "value is not a valid float", "type_error.float")
    if type_name == "boolean":
        low = value.lower()
        if low in ("true", "1"):
            return True
        if low in ("false", "0"):
            return False
        raise _validation_error(loc, "value could not be parsed as boolean", "type_error.bool")
    return value


class _ValidationExc(Exception):
    def __init__(self, errors: list[dict[str, Any]]) -> None:
        self.errors = errors


def _validation_error(loc: list[Any], msg: str, typ: str) -> _ValidationExc:
    return _ValidationExc([{"loc": list(loc), "msg": msg, "type": typ}])


def _merge_validation(*excs: _ValidationExc) -> _ValidationExc:
    errs: list[dict[str, Any]] = []
    for e in excs:
        errs.extend(e.errors)
    return _ValidationExc(errs)


class Route:
    def __init__(
        self,
        method: str,
        path: str,
        endpoint: Callable[..., Any],
        *,
        status_code: int = 200,
        response_model: Any = None,
        name: str | None = None,
        tags: list[str] | None = None,
    ) -> None:
        self.method = method.upper()
        self.path = _normalize_path(path)
        self.endpoint = endpoint
        self.status_code = int(status_code)
        self.response_model = response_model
        self.name = name or endpoint.__name__
        self.tags = list(tags or [])
        self.regex, self.path_param_names = _compile_path(self.path)
        self.params = self._analyze_params()

    def _analyze_params(self) -> list[dict[str, Any]]:
        sig = inspect.signature(self.endpoint)
        hints = {}
        try:
            hints = dict(getattr(self.endpoint, "__annotations__", {}) or {})
        except Exception:
            hints = {}
        out: list[dict[str, Any]] = []
        for pname, param in sig.parameters.items():
            if pname in ("self", "cls"):
                continue
            default = param.default
            annotation = hints.get(pname, param.annotation)
            type_name = _annotation_type_name(annotation)

            if isinstance(default, Depends):
                out.append(
                    {
                        "name": pname,
                        "kind": "depends",
                        "dependency": default.dependency,
                        "type": type_name,
                        "required": True,
                        "default": None,
                        "alias": pname,
                    }
                )
                continue

            kind = "query"
            alias = pname
            required = True
            defval: Any = None

            if isinstance(default, Path):
                kind = "path"
                alias = default.alias or pname
                required = True
                defval = None if default.default is ... else default.default
            elif isinstance(default, Query):
                kind = "query"
                alias = default.alias or pname
                required = default.required
                defval = None if default.default is ... else default.default
            elif isinstance(default, Body):
                kind = "body"
                alias = default.alias or pname
                required = default.required
                defval = None if default.default is ... else default.default
            elif default is inspect.Parameter.empty:
                # bare param: path if in path template else query
                if pname in self.path_param_names:
                    kind = "path"
                else:
                    kind = "query"
                required = True
                defval = None
            else:
                # default value provided → optional query (unless path)
                if pname in self.path_param_names:
                    kind = "path"
                    required = True
                    defval = default
                else:
                    kind = "query"
                    required = False
                    defval = default

            # Body inferred: annotation dict/list or name body
            if kind != "body" and kind != "path" and kind != "depends":
                if type_name in ("object", "array") or pname == "body":
                    # only treat as body for POST/PUT/PATCH
                    if self.method in ("POST", "PUT", "PATCH"):
                        kind = "body"

            if kind == "path":
                required = True

            out.append(
                {
                    "name": pname,
                    "kind": kind,
                    "type": type_name if kind != "body" else type_name,
                    "required": required,
                    "default": defval,
                    "alias": alias,
                    "dependency": None,
                }
            )
        return out


class APIRouter:
    def __init__(self, *, prefix: str = "", tags: list[str] | None = None) -> None:
        self.prefix = _normalize_path(prefix) if prefix else ""
        if self.prefix == "/":
            self.prefix = ""
        self.tags = list(tags or [])
        self.routes: list[Route] = []

    def add_api_route(
        self,
        path: str,
        endpoint: Callable[..., Any],
        *,
        methods: list[str] | None = None,
        status_code: int = 200,
        response_model: Any = None,
        name: str | None = None,
        tags: list[str] | None = None,
    ) -> None:
        methods = methods or ["GET"]
        merged_tags = list(self.tags) + list(tags or [])
        full = _normalize_path((self.prefix or "") + _normalize_path(path))
        for m in methods:
            self.routes.append(
                Route(
                    m,
                    full,
                    endpoint,
                    status_code=status_code,
                    response_model=response_model,
                    name=name,
                    tags=merged_tags,
                )
            )

    def get(self, path: str, **kwargs: Any) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        return self._decorator(path, ["GET"], **kwargs)

    def post(self, path: str, **kwargs: Any) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        return self._decorator(path, ["POST"], **kwargs)

    def put(self, path: str, **kwargs: Any) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        return self._decorator(path, ["PUT"], **kwargs)

    def delete(self, path: str, **kwargs: Any) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        return self._decorator(path, ["DELETE"], **kwargs)

    def patch(self, path: str, **kwargs: Any) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        return self._decorator(path, ["PATCH"], **kwargs)

    def _decorator(
        self, path: str, methods: list[str], **kwargs: Any
    ) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        def deco(fn: Callable[..., Any]) -> Callable[..., Any]:
            self.add_api_route(path, fn, methods=methods, **kwargs)
            return fn

        return deco

    def include_router(self, router: "APIRouter", *, prefix: str = "", tags: list[str] | None = None) -> None:
        pref = _normalize_path(prefix) if prefix else ""
        if pref == "/":
            pref = ""
        extra_tags = list(tags or [])
        for r in router.routes:
            new_path = _normalize_path(pref + r.path)
            # rebuild route with new path
            self.routes.append(
                Route(
                    r.method,
                    new_path,
                    r.endpoint,
                    status_code=r.status_code,
                    response_model=r.response_model,
                    name=r.name,
                    tags=list(r.tags) + extra_tags,
                )
            )


class App(APIRouter):
    def __init__(self, *, title: str = "fastapi_mini", version: str = "0.1.0") -> None:
        super().__init__()
        self.title = title
        self.version = version

    def handle(
        self,
        method: str,
        path: str,
        *,
        query: str | dict[str, Any] | None = None,
        body: str | bytes | None = None,
        headers: dict[str, str] | None = None,
    ) -> Response:
        method = method.upper()
        path_only = path.split("?", 1)[0]
        path_only = _normalize_path(path_only)
        headers = {k.lower(): v for k, v in (headers or {}).items()}

        # Parse query
        qdict: dict[str, list[str]] = {}
        if isinstance(query, dict):
            for k, v in query.items():
                if isinstance(v, list):
                    qdict[k] = [str(x) for x in v]
                else:
                    qdict[k] = [str(v)]
        elif isinstance(query, str) and query:
            qdict = parse_qs(query, keep_blank_values=True)
        elif "?" in path:
            qdict = parse_qs(path.split("?", 1)[1], keep_blank_values=True)

        body_text: str | None
        if body is None:
            body_text = None
        elif isinstance(body, bytes):
            body_text = body.decode("utf-8")
        else:
            body_text = body

        # Find matching routes for path (any method) for 405 detection
        path_matches: list[Route] = []
        method_match: Route | None = None
        match_groups: dict[str, str] | None = None
        for route in self.routes:
            m = route.regex.match(path_only)
            if not m:
                continue
            path_matches.append(route)
            if route.method == method:
                method_match = route
                match_groups = m.groupdict()
                break  # first registered wins (resolution order)

        if not path_matches:
            return JSONResponse({"detail": "Not Found"}, status_code=404)

        if method_match is None:
            allow = sorted({r.method for r in path_matches})
            resp = JSONResponse({"detail": "Method Not Allowed"}, status_code=405)
            resp.headers["allow"] = ", ".join(allow)
            return resp

        assert match_groups is not None
        try:
            return self._invoke(method_match, match_groups, qdict, body_text, headers)
        except _ValidationExc as ve:
            return JSONResponse({"detail": ve.errors}, status_code=422)
        except HTTPException as he:
            return JSONResponse({"detail": he.detail}, status_code=he.status_code)

    def _resolve_depends(
        self,
        dep_fn: Callable[..., Any],
        path_params: dict[str, str],
        qdict: dict[str, list[str]],
        body_text: str | None,
        headers: dict[str, str],
        stack: set[int],
    ) -> Any:
        dep_id = id(dep_fn)
        if dep_id in stack:
            raise HTTPException(500, detail="Circular dependency")
        stack.add(dep_id)
        # Build a temporary Route-like analysis for the dependency
        tmp = Route("GET", "/", dep_fn)
        try:
            kwargs = self._extract_kwargs(tmp, path_params, qdict, body_text, headers, stack)
            result = dep_fn(**kwargs)
            return result
        finally:
            stack.discard(dep_id)

    def _extract_kwargs(
        self,
        route: Route,
        path_params: dict[str, str],
        qdict: dict[str, list[str]],
        body_text: str | None,
        headers: dict[str, str],
        stack: set[int],
    ) -> dict[str, Any]:
        kwargs: dict[str, Any] = {}
        errors: list[dict[str, Any]] = []
        body_parsed = False
        body_json: Any = None

        for p in route.params:
            kind = p["kind"]
            name = p["name"]
            alias = p["alias"]
            typ = p["type"]

            if kind == "depends":
                try:
                    kwargs[name] = self._resolve_depends(
                        p["dependency"], path_params, qdict, body_text, headers, stack
                    )
                except _ValidationExc as ve:
                    errors.extend(ve.errors)
                except HTTPException:
                    raise
                continue

            if kind == "path":
                raw = path_params.get(alias)
                if raw is None:
                    errors.append(
                        {"loc": ["path", alias], "msg": "field required", "type": "value_error.missing"}
                    )
                    continue
                try:
                    kwargs[name] = _coerce(raw, typ, ["path", alias])
                except _ValidationExc as ve:
                    errors.extend(ve.errors)
                continue

            if kind == "query":
                vals = qdict.get(alias)
                if not vals:
                    if p["required"]:
                        errors.append(
                            {
                                "loc": ["query", alias],
                                "msg": "field required",
                                "type": "value_error.missing",
                            }
                        )
                    else:
                        kwargs[name] = p["default"]
                    continue
                raw = vals[0]
                try:
                    kwargs[name] = _coerce(raw, typ, ["query", alias])
                except _ValidationExc as ve:
                    errors.extend(ve.errors)
                continue

            if kind == "body":
                ct = headers.get("content-type", "application/json")
                # empty body
                if body_text is None or body_text == "":
                    if p["required"]:
                        errors.append(
                            {
                                "loc": ["body"],
                                "msg": "field required",
                                "type": "value_error.missing",
                            }
                        )
                    else:
                        kwargs[name] = p["default"]
                    continue
                if "application/json" not in ct and ct.strip() != "":
                    # non-json content-type with body → 422 for this mini
                    errors.append(
                        {
                            "loc": ["body"],
                            "msg": "value is not a valid json",
                            "type": "type_error.json",
                        }
                    )
                    continue
                if not body_parsed:
                    try:
                        body_json = json.loads(body_text)
                        body_parsed = True
                    except json.JSONDecodeError:
                        errors.append(
                            {
                                "loc": ["body"],
                                "msg": "value is not a valid json",
                                "type": "type_error.json",
                            }
                        )
                        continue
                kwargs[name] = body_json
                continue

        if errors:
            raise _ValidationExc(errors)
        return kwargs

    def _invoke(
        self,
        route: Route,
        path_params: dict[str, str],
        qdict: dict[str, list[str]],
        body_text: str | None,
        headers: dict[str, str],
    ) -> Response:
        kwargs = self._extract_kwargs(route, path_params, qdict, body_text, headers, set())
        result = route.endpoint(**kwargs)

        if isinstance(result, Response):
            return result
        if isinstance(result, tuple) and len(result) == 2 and isinstance(result[1], int):
            data, code = result
            data = filter_response(data, route.response_model)
            return JSONResponse(data, status_code=code)

        data = filter_response(result, route.response_model)
        return JSONResponse(data, status_code=route.status_code)

    def openapi(self) -> dict[str, Any]:
        """Return OpenAPI 3.0.3-shaped schema for registered routes (exact subset)."""
        paths: dict[str, Any] = {}
        for route in self.routes:
            path_item = paths.setdefault(route.path, {})
            parameters = []
            request_body = None
            for p in route.params:
                if p["kind"] == "depends":
                    # Expand one level of dependency params into OpenAPI
                    dep_route = Route("GET", "/", p["dependency"])
                    for dp in dep_route.params:
                        if dp["kind"] == "query":
                            parameters.append(
                                {
                                    "name": dp["alias"],
                                    "in": "query",
                                    "required": dp["required"],
                                    "schema": {"type": dp["type"]},
                                }
                            )
                        elif dp["kind"] == "path":
                            parameters.append(
                                {
                                    "name": dp["alias"],
                                    "in": "path",
                                    "required": True,
                                    "schema": {"type": dp["type"]},
                                }
                            )
                    continue
                if p["kind"] == "path":
                    parameters.append(
                        {
                            "name": p["alias"],
                            "in": "path",
                            "required": True,
                            "schema": {"type": p["type"]},
                        }
                    )
                elif p["kind"] == "query":
                    parameters.append(
                        {
                            "name": p["alias"],
                            "in": "query",
                            "required": p["required"],
                            "schema": {"type": p["type"]},
                        }
                    )
                elif p["kind"] == "body":
                    request_body = {
                        "required": p["required"],
                        "content": {
                            "application/json": {
                                "schema": {"type": p["type"] if p["type"] != "string" else "object"}
                            }
                        },
                    }

            op: dict[str, Any] = {
                "operationId": route.name,
                "responses": {
                    str(route.status_code): {
                        "description": "Successful Response",
                        "content": {"application/json": {"schema": {}}},
                    }
                },
            }
            if route.tags:
                op["tags"] = list(route.tags)
            if parameters:
                op["parameters"] = parameters
            if request_body is not None:
                op["requestBody"] = request_body

            path_item[route.method.lower()] = op

        return {
            "openapi": "3.0.3",
            "info": {"title": self.title, "version": self.version},
            "paths": paths,
        }
