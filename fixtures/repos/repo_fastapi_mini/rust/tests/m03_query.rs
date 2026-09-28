//! m03: query params — defaults, required, coercion fail
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
fn m03_query_required_ok() {
    let mut app = App::new();
    app.route("GET", "/search")
        .query_str("q", true, None)
        .handler(|ctx| Ok(Outcome::json(json_obj(&[("q", json_str(ctx.query_str("q").unwrap_or("")))]))));
    let c = TestClient::new(app);
    let r = c.get_query("/search", &[("q", "hello")]);
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("q"), Some(&json_str("hello")));
}

#[test]
fn m03_query_required_missing_422() {
    let mut app = App::new();
    app.route("GET", "/search")
        .query_str("q", true, None)
        .handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.get("/search");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["query", "q"], "value_error.missing"), "{}", r.text());
}

#[test]
fn m03_query_default_int() {
    let mut app = App::new();
    app.route("GET", "/search")
        .query_str("q", true, None)
        .query_int("limit", false, Some(10))
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[
                ("q", json_str(ctx.query_str("q").unwrap_or(""))),
                ("limit", JsonValue::Int(ctx.query_int("limit").unwrap_or(10))),
            ])))
        });
    let c = TestClient::new(app);
    let j = c.get_query("/search", &[("q", "x")]).json();
    assert_eq!(j.get("limit"), Some(&JsonValue::Int(10)));
}

#[test]
fn m03_query_int_coercion_fail() {
    let mut app = App::new();
    app.route("GET", "/search")
        .query_str("q", true, None)
        .query_int("limit", false, Some(10))
        .handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.get_query("/search", &[("q", "x"), ("limit", "nope")]);
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["query", "limit"], "type_error.integer"));
}

#[test]
fn m03_query_bool_true_false() {
    let mut app = App::new();
    app.route("GET", "/f")
        .query_bool("flag", true, None)
        .handler(|ctx| Ok(Outcome::json(JsonValue::Bool(ctx.query_bool("flag").unwrap_or(false)))));
    let c = TestClient::new(app);
    assert_eq!(c.get_query("/f", &[("flag", "true")]).json(), JsonValue::Bool(true));
    assert_eq!(c.get_query("/f", &[("flag", "0")]).json(), JsonValue::Bool(false));
}

#[test]
fn m03_query_bool_invalid() {
    let mut app = App::new();
    app.route("GET", "/f")
        .query_bool("flag", true, None)
        .handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.get_query("/f", &[("flag", "maybe")]);
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["query", "flag"], "type_error.bool"));
}

#[test]
fn m03_query_float_ok_and_fail() {
    let mut app = App::new();
    app.route("GET", "/g")
        .query_float("x", true, None)
        .handler(|ctx| Ok(Outcome::json(JsonValue::Float(ctx.query_float("x").unwrap_or(0.0)))));
    let c = TestClient::new(app);
    let j = c.get_query("/g", &[("x", "2.5")]).json();
    match j {
        JsonValue::Float(f) => assert!((f - 2.5).abs() < 1e-9),
        other => panic!("{other:?}"),
    }
    let r = c.get_query("/g", &[("x", "zz")]);
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["query", "x"], "type_error.float"));
}
