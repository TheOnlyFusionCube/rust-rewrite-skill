//! m08: include_router prefixes + route resolution order
use fastapi_mini::{json_obj, json_str, APIRouter, App, JsonValue, Outcome, TestClient};

#[test]
fn m08_include_router_prefix() {
    let mut router = APIRouter::new();
    router
        .route("GET", "/ping")
        .handler(|_| Ok(Outcome::json(json_obj(&[("pong", JsonValue::Bool(true))]))));
    let mut app = App::new();
    app.include_router(router, "/api", &[]);
    let c = TestClient::new(app);
    assert_eq!(c.get("/api/ping").status_code(), 200);
    assert_eq!(c.get("/api/ping").json().get("pong"), Some(&JsonValue::Bool(true)));
    assert_eq!(c.get("/ping").status_code(), 404);
}

#[test]
fn m08_nested_prefix_normalize() {
    let mut router = APIRouter::new();
    router
        .route("GET", "/x")
        .handler(|_| Ok(Outcome::json(json_str("ok"))));
    let mut app = App::new();
    app.include_router(router, "api/", &[]); // missing leading slash
    assert_eq!(TestClient::new(app).get("/api/x").json(), json_str("ok"));
}

#[test]
fn m08_first_registered_wins() {
    let mut app = App::new();
    app.route("GET", "/x/{item_id}")
        .path_str("item_id")
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[
                ("who", json_str("first")),
                ("id", json_str(ctx.path_str("item_id"))),
            ])))
        });
    app.route("GET", "/x/{item_id}")
        .path_str("item_id")
        .handler(|_| Ok(Outcome::json(json_obj(&[("who", json_str("second"))]))));
    let j = TestClient::new(app).get("/x/1").json();
    assert_eq!(j.get("who"), Some(&json_str("first")));
}

#[test]
fn m08_router_tags_appended() {
    let mut router = APIRouter::new();
    router
        .route("GET", "/t")
        .name("tagged")
        .handler(|_| Ok(Outcome::json(json_obj(&[]))));
    let mut app = App::with_meta("T", "1");
    app.include_router(router, "/v1", &["extra"]);
    let schema = app.openapi();
    let paths = schema.get("paths").unwrap();
    let op = paths.get("/v1/t").and_then(|p| p.get("get"));
    let tags = op.and_then(|o| o.get("tags"));
    let Some(JsonValue::Array(arr)) = tags else { panic!("no tags {:?}", schema) };
    assert!(arr.iter().any(|t| t.as_str_value() == Some("extra")));
}

#[test]
fn m08_multiple_routers() {
    let mut r1 = APIRouter::new();
    r1.route("GET", "/a").handler(|_| Ok(Outcome::json(json_str("a"))));
    let mut r2 = APIRouter::new();
    r2.route("GET", "/b").handler(|_| Ok(Outcome::json(json_str("b"))));
    let mut app = App::new();
    app.include_router(r1, "/r1", &[]);
    app.include_router(r2, "/r2", &[]);
    let c = TestClient::new(app);
    assert_eq!(c.get("/r1/a").json(), json_str("a"));
    assert_eq!(c.get("/r2/b").json(), json_str("b"));
}

#[test]
fn m08_app_route_and_router_coexist() {
    let mut router = APIRouter::new();
    router
        .route("GET", "/inner")
        .handler(|_| Ok(Outcome::json(json_str("inner"))));
    let mut app = App::new();
    app.route("GET", "/outer")
        .handler(|_| Ok(Outcome::json(json_str("outer"))));
    app.include_router(router, "/p", &[]);
    let c = TestClient::new(app);
    assert_eq!(c.get("/outer").json(), json_str("outer"));
    assert_eq!(c.get("/p/inner").json(), json_str("inner"));
}
