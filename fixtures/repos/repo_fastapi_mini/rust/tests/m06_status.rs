//! m06: status codes + HTTPException + response body
use fastapi_mini::{json_obj, json_str, App, HTTPException, JsonValue, Outcome, Response, TestClient};

#[test]
fn m06_default_status_200() {
    let mut app = App::new();
    app.route("GET", "/x").handler(|_| Ok(Outcome::json(json_obj(&[]))));
    assert_eq!(TestClient::new(app).get("/x").status_code(), 200);
}

#[test]
fn m06_route_status_code_201() {
    let mut app = App::new();
    app.route("POST", "/things")
        .status_code(201)
        .handler(|_| Ok(Outcome::json(json_obj(&[("id", JsonValue::Int(1))]))));
    let r = TestClient::new(app).post_json("/things", &json_obj(&[]));
    assert_eq!(r.status_code(), 201);
    assert_eq!(r.json().get("id"), Some(&JsonValue::Int(1)));
}

#[test]
fn m06_http_exception_418() {
    let mut app = App::new();
    app.route("GET", "/boom")
        .handler(|_| Err(HTTPException::str(418, "teapot")));
    let r = TestClient::new(app).get("/boom");
    assert_eq!(r.status_code(), 418);
    assert_eq!(r.json(), json_obj(&[("detail", json_str("teapot"))]));
}

#[test]
fn m06_http_exception_object_detail() {
    let mut app = App::new();
    app.route("GET", "/e").handler(|_| {
        Err(HTTPException::new(
            400,
            json_obj(&[("code", json_str("bad"))]),
        ))
    });
    let r = TestClient::new(app).get("/e");
    assert_eq!(r.status_code(), 400);
    assert_eq!(
        r.json().get("detail").and_then(|d| d.get("code")),
        Some(&json_str("bad"))
    );
}

#[test]
fn m06_outcome_json_status() {
    let mut app = App::new();
    app.route("GET", "/s")
        .handler(|_| Ok(Outcome::JsonStatus(json_obj(&[("x", JsonValue::Int(1))]), 202)));
    let r = TestClient::new(app).get("/s");
    assert_eq!(r.status_code(), 202);
    assert_eq!(r.json().get("x"), Some(&JsonValue::Int(1)));
}

#[test]
fn m06_outcome_response_raw() {
    let mut app = App::new();
    app.route("GET", "/raw").handler(|_| {
        Ok(Outcome::Response(Response::json(
            203,
            &json_obj(&[("raw", JsonValue::Bool(true))]),
        )))
    });
    let r = TestClient::new(app).get("/raw");
    assert_eq!(r.status_code(), 203);
    assert_eq!(r.json().get("raw"), Some(&JsonValue::Bool(true)));
}

#[test]
fn m06_delete_204_empty_object() {
    let mut app = App::new();
    app.route("DELETE", "/x/{id}")
        .path_int("id")
        .status_code(204)
        .handler(|_| Ok(Outcome::json(json_obj(&[]))));
    let r = TestClient::new(app).delete("/x/1");
    assert_eq!(r.status_code(), 204);
}
