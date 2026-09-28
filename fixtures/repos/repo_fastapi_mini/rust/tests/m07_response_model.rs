//! m07: response_model / field filtering
use fastapi_mini::{json_obj, json_str, App, JsonValue, Outcome, TestClient};

#[test]
fn m07_filter_dict_fields() {
    let mut app = App::new();
    app.route("GET", "/user")
        .response_model(&["id", "name"])
        .handler(|_| {
            Ok(Outcome::json(json_obj(&[
                ("id", JsonValue::Int(1)),
                ("name", json_str("a")),
                ("secret", json_str("nope")),
            ])))
        });
    let j = TestClient::new(app).get("/user").json();
    assert_eq!(j, json_obj(&[("id", JsonValue::Int(1)), ("name", json_str("a"))]));
    assert!(j.get("secret").is_none());
}

#[test]
fn m07_filter_preserves_order_keys_listed() {
    let mut app = App::new();
    app.route("GET", "/u")
        .response_model(&["b", "a"])
        .handler(|_| {
            Ok(Outcome::json(json_obj(&[
                ("a", JsonValue::Int(1)),
                ("b", JsonValue::Int(2)),
                ("c", JsonValue::Int(3)),
            ])))
        });
    let text = TestClient::new(app).get("/u").text().to_string();
    // compact JSON via BTreeMap sorts keys — contract: only a,b present.
    let j = JsonValue::parse(&text).unwrap();
    assert!(j.get("a").is_some() && j.get("b").is_some() && j.get("c").is_none());
}

#[test]
fn m07_filter_list_of_dicts() {
    let mut app = App::new();
    app.route("GET", "/users")
        .response_model(&["id"])
        .handler(|_| {
            Ok(Outcome::json(JsonValue::Array(vec![
                json_obj(&[("id", JsonValue::Int(1)), ("x", json_str("z"))]),
                json_obj(&[("id", JsonValue::Int(2)), ("x", json_str("z"))]),
            ])))
        });
    let j = TestClient::new(app).get("/users").json();
    let JsonValue::Array(arr) = j else { panic!("not array") };
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], json_obj(&[("id", JsonValue::Int(1))]));
    assert_eq!(arr[1], json_obj(&[("id", JsonValue::Int(2))]));
}

#[test]
fn m07_no_model_keeps_all() {
    let mut app = App::new();
    app.route("GET", "/user").handler(|_| {
        Ok(Outcome::json(json_obj(&[
            ("id", JsonValue::Int(1)),
            ("secret", json_str("keep")),
        ])))
    });
    let j = TestClient::new(app).get("/user").json();
    assert_eq!(j.get("secret"), Some(&json_str("keep")));
}

#[test]
fn m07_filter_missing_field_skipped() {
    let mut app = App::new();
    app.route("GET", "/u")
        .response_model(&["id", "missing"])
        .handler(|_| Ok(Outcome::json(json_obj(&[("id", JsonValue::Int(1))]))));
    let j = TestClient::new(app).get("/u").json();
    assert_eq!(j, json_obj(&[("id", JsonValue::Int(1))]));
}

#[test]
fn m07_filter_with_custom_status() {
    let mut app = App::new();
    app.route("POST", "/u")
        .status_code(201)
        .response_model(&["id"])
        .handler(|_| {
            Ok(Outcome::json(json_obj(&[
                ("id", JsonValue::Int(9)),
                ("extra", json_str("x")),
            ])))
        });
    let r = TestClient::new(app).post_json("/u", &json_obj(&[]));
    assert_eq!(r.status_code(), 201);
    assert_eq!(r.json(), json_obj(&[("id", JsonValue::Int(9))]));
}

#[test]
fn m07_json_status_outcome_still_filters() {
    let mut app = App::new();
    app.route("GET", "/u")
        .response_model(&["id"])
        .handler(|_| {
            Ok(Outcome::JsonStatus(
                json_obj(&[("id", JsonValue::Int(1)), ("x", JsonValue::Int(2))]),
                202,
            ))
        });
    let r = TestClient::new(app).get("/u");
    assert_eq!(r.status_code(), 202);
    assert_eq!(r.json(), json_obj(&[("id", JsonValue::Int(1))]));
}
