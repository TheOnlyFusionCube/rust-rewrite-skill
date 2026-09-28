//! m02: path params — types, missing, invalid
use fastapi_mini::{json_obj, json_str, App, JsonValue, Outcome, TestClient};

fn detail_has(v: &JsonValue, loc: &[&str], typ: &str) -> bool {
    let Some(JsonValue::Array(arr)) = v.get("detail") else { return false };
    arr.iter().any(|d| {
        let locs = d.get("loc").and_then(|x| if let JsonValue::Array(a) = x { Some(a) } else { None });
        let t = d.get("type").and_then(|x| x.as_str_value());
        let Some(locs) = locs else { return false };
        if locs.len() != loc.len() { return false; }
        locs.iter().zip(loc.iter()).all(|(a, b)| a.as_str_value() == Some(*b)) && t == Some(typ)
    })
}

#[test]
fn m02_path_int_ok() {
    let mut app = App::new();
    app.route("GET", "/items/{item_id}")
        .path_int("item_id")
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[("item_id", JsonValue::Int(ctx.path_int("item_id")))])))
        });
    let c = TestClient::new(app);
    let r = c.get("/items/42");
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json(), json_obj(&[("item_id", JsonValue::Int(42))]));
}

#[test]
fn m02_path_str_ok() {
    let mut app = App::new();
    app.route("GET", "/u/{name}")
        .path_str("name")
        .handler(|ctx| Ok(Outcome::json(json_obj(&[("name", json_str(ctx.path_str("name")))]))));
    let c = TestClient::new(app);
    assert_eq!(c.get("/u/alice").json().get("name"), Some(&json_str("alice")));
}

#[test]
fn m02_path_int_invalid_422() {
    let mut app = App::new();
    app.route("GET", "/items/{item_id}")
        .path_int("item_id")
        .handler(|ctx| Ok(Outcome::json(JsonValue::Int(ctx.path_int("item_id")))));
    let c = TestClient::new(app);
    let r = c.get("/items/abc");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["path", "item_id"], "type_error.integer"), "{:?}", r.text());
}

#[test]
fn m02_path_int_rejects_float_string() {
    let mut app = App::new();
    app.route("GET", "/n/{n}").path_int("n").handler(|ctx| Ok(Outcome::json(JsonValue::Int(ctx.path_int("n")))));
    let c = TestClient::new(app);
    let r = c.get("/n/1.5");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["path", "n"], "type_error.integer"));
}

#[test]
fn m02_path_two_params() {
    let mut app = App::new();
    app.route("GET", "/a/{x}/b/{y}")
        .path_int("x")
        .path_str("y")
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[
                ("x", JsonValue::Int(ctx.path_int("x"))),
                ("y", json_str(ctx.path_str("y"))),
            ])))
        });
    let c = TestClient::new(app);
    let j = c.get("/a/7/b/zz").json();
    assert_eq!(j.get("x"), Some(&JsonValue::Int(7)));
    assert_eq!(j.get("y"), Some(&json_str("zz")));
}

#[test]
fn m02_path_negative_int() {
    let mut app = App::new();
    app.route("GET", "/n/{n}").path_int("n").handler(|ctx| Ok(Outcome::json(JsonValue::Int(ctx.path_int("n")))));
    let c = TestClient::new(app);
    assert_eq!(c.get("/n/-3").json(), JsonValue::Int(-3));
}

#[test]
fn m02_path_no_match_is_404_not_422() {
    let mut app = App::new();
    app.route("GET", "/items/{item_id}").path_int("item_id").handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    // extra segment → 404
    assert_eq!(c.get("/items/1/extra").status_code(), 404);
}
