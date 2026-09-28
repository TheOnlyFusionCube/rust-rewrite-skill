//! m04: JSON body + 422 validation shape
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
fn m04_body_ok() {
    let mut app = App::new();
    app.route("POST", "/items")
        .body_json(true)
        .handler(|ctx| {
            let body = ctx.body().cloned().unwrap_or(JsonValue::Null);
            Ok(Outcome::json(json_obj(&[
                ("ok", JsonValue::Bool(true)),
                ("body", body),
            ])))
        });
    let c = TestClient::new(app);
    let r = c.post_json("/items", &json_obj(&[("name", json_str("a"))]));
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("body").and_then(|b| b.get("name")), Some(&json_str("a")));
}

#[test]
fn m04_body_missing_422() {
    let mut app = App::new();
    app.route("POST", "/items").body_json(true).handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.request("POST", "/items", &[], None, &[("content-type", "application/json")]);
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["body"], "value_error.missing"), "{}", r.text());
}

#[test]
fn m04_body_empty_string_422() {
    let mut app = App::new();
    app.route("POST", "/items").body_json(true).handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.post_raw("/items", "", "application/json");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["body"], "value_error.missing"));
}

#[test]
fn m04_body_bad_json_422() {
    let mut app = App::new();
    app.route("POST", "/items").body_json(true).handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.post_raw("/items", "{bad", "application/json");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["body"], "type_error.json"));
}

#[test]
fn m04_body_optional_none_ok() {
    let mut app = App::new();
    app.route("POST", "/items")
        .body_json(false)
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[(
                "has",
                JsonValue::Bool(ctx.body().is_some()),
            )])))
        });
    let c = TestClient::new(app);
    let r = c.request("POST", "/items", &[], None, &[]);
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("has"), Some(&JsonValue::Bool(false)));
}

#[test]
fn m04_body_array_ok() {
    let mut app = App::new();
    app.route("POST", "/arr")
        .body_json(true)
        .handler(|ctx| Ok(Outcome::json(ctx.body().cloned().unwrap_or(JsonValue::Null))));
    let c = TestClient::new(app);
    let body = JsonValue::Array(vec![JsonValue::Int(1), JsonValue::Int(2)]);
    assert_eq!(c.post_json("/arr", &body).json(), body);
}

#[test]
fn m04_422_detail_is_array() {
    let mut app = App::new();
    app.route("POST", "/items").body_json(true).handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let c = TestClient::new(app);
    let r = c.post_raw("/items", "", "application/json");
    assert!(matches!(r.json().get("detail"), Some(JsonValue::Array(_))));
}
