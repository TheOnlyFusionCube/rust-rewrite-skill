# PORTING.md — Python fastapi_mini → Rust

This is a **FastAPI-shaped mini** framework, **not** a full FastAPI port.

## Type / concept map

| Python | Rust |
|--------|------|
| `App(title=, version=)` | `App::with_meta(title, version)` or `App::new()` (defaults `fastapi_mini` / `0.1.0`) |
| `APIRouter()` | `APIRouter::new()` |
| `@app.get(path, status_code=, response_model=, tags=, name=)` | `app.route("GET", path).status_code(n).response_model(&["f"]).tags(&["t"]).name("…").query_*/path_*/body_json().depend(...).handler(...)` |
| `Path()` / bare path param | `.path_int("id")` / `.path_str("name")` |
| `Query(...)` / `Query(default)` | `.query_str/int/float/bool(name, required, default)` |
| `Body(...)` | `.body_json(required)` |
| `Depends(fn)` | `Dependency::build()…handler(...)` then `.depend("name", dep)` |
| `HTTPException(status, detail=)` | `HTTPException::new(status, JsonValue)` / `HTTPException::str(status, "msg")` |
| `JSONResponse(content, status_code=)` | `Outcome::JsonResponse { status, value }` or `Response::json(status, &value)` |
| return `dict` | `Outcome::Json(value)` using default route status |
| `response_fields("a","b")` | `.response_model(&["a","b"])` |
| `app.include_router(r, prefix="/api", tags=[…])` | `app.include_router(router, "/api", &["…"])` |
| `TestClient(app)` | `TestClient::new(app)` |
| `client.get/post(url, params=, json=)` | `client.get(url)`, `client.get_query(url, &[("q","x")])`, `client.post_json(url, &body)` |
| `r.status_code` / `r.json()` / `r.text` / `r.headers` | `r.status_code()` / `r.json()` / `r.text()` / `r.header("allow")` |
| `app.openapi()` → dict | `app.openapi()` → `String` (JSON) or `JsonValue` — tests compare canonical JSON |

## JsonValue (std-friendly)

Implement a small enum: `Null`, `Bool`, `Int`, `Float`, `Str`, `Array`, `Object`.
Tests compare via `JsonValue::parse` + equality, or exact text where noted.

## Milestone order (do not skip ahead for scoring)

`m01_` → `m02_` → `m03_` → `m04_` → `m05_` → `m06_` → `m07_` → `m08_` → `m09_` → `m10_`

Run: `cargo test m01_`, then `m02_`, …

## Content-Type / body 422 (common C-tier miss)

When a route has `.body_json(true)` / required JSON body:
- If `content-type` is present and does **not** contain `application/json` (e.g. `text/plain`), return **422** with `detail[].type == "type_error.json"` and `loc` including `"body"` — even if the raw body bytes parse as JSON.
- Empty `content-type` may be treated as JSON (match README).
- Never return 200 for `post_raw(..., "text/plain")` on a JSON body route.

## Hard rules

- **Never** edit, weaken, delete, or skip files under `rust/tests/`.
- Prefer std-only (a tiny hand-rolled JSON is fine; external crates discouraged).
- Leave `source/` intact as the behavioral oracle.
- Match 422 `detail` list **exactly** (loc/msg/type strings in README).
