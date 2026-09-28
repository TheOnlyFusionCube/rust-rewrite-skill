"""Smoke tests for the Python fastapi_mini oracle (must pass before shipping fixture)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from fastapi_mini import (
    App,
    APIRouter,
    Body,
    Depends,
    HTTPException,
    JSONResponse,
    Path as FPath,
    Query,
    TestClient,
    response_fields,
)


def check(name: str, cond: bool) -> None:
    if not cond:
        raise AssertionError(f"FAIL: {name}")
    print(f"  ok {name}")


def main() -> None:
    print("fastapi_mini Python selfcheck")

    # --- m01 routes + 404 ---
    app = App(title="t", version="1.0.0")

    @app.get("/hello")
    def hello():
        return {"msg": "hi"}

    c = TestClient(app)
    r = c.get("/hello")
    check("GET /hello 200", r.status_code == 200 and r.json() == {"msg": "hi"})
    r = c.get("/missing")
    check("404 missing", r.status_code == 404 and r.json() == {"detail": "Not Found"})

    # --- m02 path params ---
    app2 = App()

    @app2.get("/items/{item_id}")
    def get_item(item_id: int = FPath()):
        return {"item_id": item_id}

    c2 = TestClient(app2)
    check("path int", c2.get("/items/42").json() == {"item_id": 42})
    r = c2.get("/items/abc")
    check("path invalid 422", r.status_code == 422)
    detail = r.json()["detail"]
    check(
        "path invalid loc",
        any(d["loc"] == ["path", "item_id"] and d["type"] == "type_error.integer" for d in detail),
    )

    # --- m03 query ---
    app3 = App()

    @app3.get("/search")
    def search(q: str = Query(...), limit: int = Query(10)):
        return {"q": q, "limit": limit}

    c3 = TestClient(app3)
    check("query default", c3.get("/search", params={"q": "x"}).json() == {"q": "x", "limit": 10})
    r = c3.get("/search")
    check("query required 422", r.status_code == 422)
    check(
        "query required loc",
        any(d["loc"] == ["query", "q"] and d["type"] == "value_error.missing" for d in r.json()["detail"]),
    )
    r = c3.get("/search", params={"q": "x", "limit": "nope"})
    check("query coerce fail", r.status_code == 422)

    # --- m04 body ---
    app4 = App()

    @app4.post("/items")
    def create(body: dict = Body(...)):
        return {"ok": True, "body": body}

    c4 = TestClient(app4)
    r = c4.post("/items", json={"name": "a"})
    check("body ok", r.status_code == 200 and r.json()["body"]["name"] == "a")
    r = c4.post("/items")
    check("body missing 422", r.status_code == 422)
    check(
        "body missing loc",
        any(d["loc"] == ["body"] and d["type"] == "value_error.missing" for d in r.json()["detail"]),
    )
    r = c4.post("/items", data="{bad", headers={"content-type": "application/json"})
    check("body bad json 422", r.status_code == 422)

    # --- m05 Depends ---
    app5 = App()

    def get_token(token: str = Query(...)):
        if token != "secret":
            raise HTTPException(403, detail="Forbidden")
        return token

    def get_user(tok: str = Depends(get_token)):
        return {"user": "alice", "token": tok}

    @app5.get("/me")
    def me(user: dict = Depends(get_user)):
        return user

    c5 = TestClient(app5)
    check("nested depends", c5.get("/me", params={"token": "secret"}).json() == {"user": "alice", "token": "secret"})
    r = c5.get("/me", params={"token": "nope"})
    check("depends fail propagate", r.status_code == 403 and r.json() == {"detail": "Forbidden"})
    r = c5.get("/me")
    check("depends missing query 422", r.status_code == 422)

    # --- m06 status + HTTPException + JSONResponse ---
    app6 = App()

    @app6.post("/things", status_code=201)
    def create_thing():
        return {"id": 1}

    @app6.get("/boom")
    def boom():
        raise HTTPException(418, detail="teapot")

    @app6.get("/raw")
    def raw():
        return JSONResponse({"x": 1}, status_code=202)

    c6 = TestClient(app6)
    check("status_code 201", c6.post("/things").status_code == 201)
    check("HTTPException 418", c6.get("/boom").status_code == 418 and c6.get("/boom").json() == {"detail": "teapot"})
    check("JSONResponse 202", c6.get("/raw").status_code == 202)

    # --- m07 response_model ---
    app7 = App()

    @app7.get("/user", response_model=response_fields("id", "name"))
    def user():
        return {"id": 1, "name": "a", "secret": "nope"}

    c7 = TestClient(app7)
    check("response_model filter", c7.get("/user").json() == {"id": 1, "name": "a"})

    # --- m08 include_router ---
    app8 = App()
    router = APIRouter()

    @router.get("/ping")
    def ping():
        return {"pong": True}

    app8.include_router(router, prefix="/api")
    c8 = TestClient(app8)
    check("router prefix", c8.get("/api/ping").json() == {"pong": True})
    check("router no bare", c8.get("/ping").status_code == 404)

    # resolution order: first registered wins
    app8b = App()

    @app8b.get("/x/{item_id}")
    def first(item_id: str):
        return {"who": "first", "id": item_id}

    @app8b.get("/x/{item_id}")
    def second(item_id: str):
        return {"who": "second", "id": item_id}

    c8b = TestClient(app8b)
    check("order first wins", c8b.get("/x/1").json()["who"] == "first")

    # --- m09 OpenAPI ---
    app9 = App(title="Demo", version="2.0.0")

    @app9.get("/items/{item_id}", tags=["items"])
    def oi(item_id: int, q: str = Query(None)):
        return {"item_id": item_id, "q": q}

    @app9.post("/items")
    def oi_post(body: dict = Body(...)):
        return body

    schema = app9.openapi()
    check("openapi version", schema["openapi"] == "3.0.3")
    check("openapi info", schema["info"] == {"title": "Demo", "version": "2.0.0"})
    check("openapi path key", "/items/{item_id}" in schema["paths"])
    get_op = schema["paths"]["/items/{item_id}"]["get"]
    check("openapi tags", get_op.get("tags") == ["items"])
    params = get_op["parameters"]
    check(
        "openapi path param",
        any(p["name"] == "item_id" and p["in"] == "path" and p["required"] is True for p in params),
    )
    check("openapi post body", "requestBody" in schema["paths"]["/items"]["post"])

    # --- m10 adversarial ---
    app10 = App()

    @app10.get("/u/{name}")
    def unicode_name(name: str):
        return {"name": name}

    @app10.post("/echo")
    def echo(body: dict = Body(...)):
        return body

    @app10.get("/only-get")
    def only_get():
        return {"ok": True}

    def dep_a(a: int = Query(...)):
        return a

    def dep_b(b: int = Query(...), aa: int = Depends(dep_a)):
        return {"a": aa, "b": b}

    @app10.get("/dep-order")
    def dep_order(d: dict = Depends(dep_b)):
        return d

    c10 = TestClient(app10)
    r = c10.get("/u/%E4%B8%AD")  # path won't auto-decode percent in our mini — use raw unicode path
    # Our handle receives already-decoded path from TestClient; pass unicode directly:
    r = c10.get("/u/中文")
    check("unicode path", r.status_code == 200 and r.json() == {"name": "中文"})
    r = c10.post("/echo", data="", headers={"content-type": "application/json"})
    check("empty body 422", r.status_code == 422)
    r = c10.post("/echo", json={"nest": {"x": [1, 2, {"z": "中"}]}})
    check("nested json", r.json()["nest"]["x"][2]["z"] == "中")
    r = c10.post("/echo", data="{}", headers={"content-type": "text/plain"})
    check("bad content-type 422", r.status_code == 422)
    r = c10.request("POST", "/only-get")
    check("405 method", r.status_code == 405)
    check("405 allow header", "GET" in r.headers.get("allow", ""))
    r = c10.get("/dep-order", params={"a": "1", "b": "2"})
    check("dep order", r.json() == {"a": 1, "b": 2})

    print("ALL SELFCHECKS PASSED")


if __name__ == "__main__":
    main()
