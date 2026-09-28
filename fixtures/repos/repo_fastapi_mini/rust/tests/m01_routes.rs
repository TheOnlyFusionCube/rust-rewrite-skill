//! m01: route registration + 404
use fastapi_mini::{json_obj, json_str, App, Outcome, TestClient};

#[test]
fn m01_get_hello_200() {
    let mut app = App::new();
    app.route("GET", "/hello").handler(|_| {
        Ok(Outcome::json(json_obj(&[("msg", json_str("hi"))])))
    });
    let c = TestClient::new(app);
    let r = c.get("/hello");
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json(), json_obj(&[("msg", json_str("hi"))]));
}

#[test]
fn m01_post_echo_path() {
    let mut app = App::new();
    app.route("POST", "/echo").handler(|_| {
        Ok(Outcome::json(json_obj(&[("ok", fastapi_mini::JsonValue::Bool(true))])))
    });
    let c = TestClient::new(app);
    let r = c.post_json("/echo", &json_obj(&[]));
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("ok"), Some(&fastapi_mini::JsonValue::Bool(true)));
}

#[test]
fn m01_404_not_found_body() {
    let app = App::new();
    let c = TestClient::new(app);
    let r = c.get("/nope");
    assert_eq!(r.status_code(), 404);
    assert_eq!(r.json(), json_obj(&[("detail", json_str("Not Found"))]));
}

#[test]
fn m01_trailing_slash_normalized() {
    let mut app = App::new();
    app.route("GET", "/hi/").handler(|_| Ok(Outcome::json(json_str("x"))));
    let c = TestClient::new(app);
    let r = c.get("/hi");
    assert_eq!(r.status_code(), 200);
}

#[test]
fn m01_multiple_methods_same_path() {
    let mut app = App::new();
    app.route("GET", "/item").handler(|_| Ok(Outcome::json(json_str("get"))));
    app.route("POST", "/item").handler(|_| Ok(Outcome::json(json_str("post"))));
    let c = TestClient::new(app);
    assert_eq!(c.get("/item").json(), json_str("get"));
    assert_eq!(c.post_json("/item", &json_obj(&[])).json(), json_str("post"));
}

#[test]
fn m01_root_path() {
    let mut app = App::new();
    app.route("GET", "/").handler(|_| Ok(Outcome::json(json_obj(&[("root", fastapi_mini::JsonValue::Bool(true))]))));
    let c = TestClient::new(app);
    assert_eq!(c.get("/").status_code(), 200);
}

#[test]
fn m01_content_type_json_header() {
    let mut app = App::new();
    app.route("GET", "/j").handler(|_| Ok(Outcome::json(json_obj(&[]))));
    let c = TestClient::new(app);
    let r = c.get("/j");
    assert_eq!(r.status_code(), 200);
    let ct = r.header("content-type").unwrap_or("");
    assert!(ct.contains("application/json"), "ct={ct}");
}
