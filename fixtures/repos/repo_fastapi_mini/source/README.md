# fastapi_mini — FastAPI-API-compatible subset (Python oracle)

**NOT** full [tiangolo/fastapi](https://github.com/tiangolo/fastapi). This is a mini
in-process HTTP framework with a FastAPI-shaped public surface for RIIR blood tests.

Behavioral oracle for the Rust port under `../rust/`. See also `../PORTING.md`.

## Contract / quirks (must match Rust)

| Behavior | Choice |
|----------|--------|
| Path normalize | Leading `/`; strip trailing `/` except root |
| Route order | **First registered match wins** |
| 404 | `{"detail":"Not Found"}` status 404 |
| 405 | `{"detail":"Method Not Allowed"}` + `allow` header (comma+space methods) |
| 422 shape | `{"detail":[ {"loc":[...], "msg":"...", "type":"..."} ]}` |
| Missing field | `msg="field required"`, `type="value_error.missing"` |
| Bad int | `msg="value is not a valid integer"`, `type="type_error.integer"` |
| Bad float | `msg="value is not a valid float"`, `type="type_error.float"` |
| Bad bool | `msg="value could not be parsed as boolean"`, `type="type_error.bool"` |
| Bad JSON | `msg="value is not a valid json"`, `type="type_error.json"` |
| Bool parse | `true`/`1` → true; `false`/`0` → false (case-insensitive) |
| Int parse | only optional leading `+/-` and digits (reject `1.5`) |
| JSON body CT | `content-type` must contain `application/json` (or empty→treat as json); else 422 type_error.json |
| Empty required body | 422 `loc=["body"]` missing |
| HTTPException | body `{"detail": <detail>}` with given status |
| JSONResponse | exact status + JSON body; sets `content-type: application/json` |
| response_model | `response_fields("a","b")` → keep only listed keys (dict); map over list items |
| Depends | resolve nested deps depth-first; HTTPException propagates; validation merges |
| include_router | prefix concatenated + normalized; tags append |
| OpenAPI | `openapi: "3.0.3"`, `info.title/version`, `paths` with method ops, parameters, requestBody |

## Python API

```python
from fastapi_mini import (
    App, APIRouter, Path, Query, Body, Depends,
    HTTPException, JSONResponse, TestClient, response_fields,
)

app = App(title="Demo", version="1.0.0")

@app.get("/items/{item_id}")
def get_item(item_id: int = Path(), q: str = Query(None)):
    return {"item_id": item_id, "q": q}

@app.post("/items", status_code=201, response_model=response_fields("id", "name"))
def create(body: dict = Body(...)):
    return {"id": 1, "name": body["name"], "extra": "drop"}

client = TestClient(app)
r = client.get("/items/1", params={"q": "x"})
# r.status_code, r.json(), r.text, r.headers
schema = app.openapi()
```

## Self-check

```bash
cd source && PYTHONPATH=. python3 -m tests.selfcheck
```
