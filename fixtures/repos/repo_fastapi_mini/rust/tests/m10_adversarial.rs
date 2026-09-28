//! m10: adversarial — unicode, empty body, nesting, content-type, 405, dep order
use fastapi_mini::{json_obj, json_str, App, Dependency, JsonValue, Outcome, TestClient};

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
fn m10_unicode_path() {
    let mut app = App::new();
    app.route("GET", "/u/{name}")
        .path_str("name")
        .handler(|ctx| Ok(Outcome::json(json_obj(&[("name", json_str(ctx.path_str("name")))]))));
    let r = TestClient::new(app).get("/u/中文");
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("name"), Some(&json_str("中文")));
}

#[test]
fn m10_unicode_json_body() {
    let mut app = App::new();
    app.route("POST", "/echo")
        .body_json(true)
        .handler(|ctx| Ok(Outcome::json(ctx.body().cloned().unwrap_or(JsonValue::Null))));
    let body = json_obj(&[(
        "nest",
        json_obj(&[(
            "x",
            JsonValue::Array(vec![
                JsonValue::Int(1),
                JsonValue::Int(2),
                json_obj(&[("z", json_str("中"))]),
            ]),
        )]),
    )]);
    let j = TestClient::new(app).post_json("/echo", &body).json();
    assert_eq!(
        j.get("nest")
            .and_then(|n| n.get("x"))
            .and_then(|x| if let JsonValue::Array(a) = x { a.get(2) } else { None })
            .and_then(|z| z.get("z")),
        Some(&json_str("中"))
    );
}

#[test]
fn m10_empty_body_required() {
    let mut app = App::new();
    app.route("POST", "/echo").body_json(true).handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let r = TestClient::new(app).post_raw("/echo", "", "application/json");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["body"], "value_error.missing"));
}

#[test]
fn m10_bad_content_type() {
    let mut app = App::new();
    app.route("POST", "/echo").body_json(true).handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let r = TestClient::new(app).post_raw("/echo", "{}", "text/plain");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["body"], "type_error.json"));
}

#[test]
fn m10_method_not_allowed() {
    let mut app = App::new();
    app.route("GET", "/only-get")
        .handler(|_| Ok(Outcome::json(json_obj(&[("ok", JsonValue::Bool(true))]))));
    let r = TestClient::new(app).request("POST", "/only-get", &[], None, &[]);
    assert_eq!(r.status_code(), 405);
    assert_eq!(r.json(), json_obj(&[("detail", json_str("Method Not Allowed"))]));
    let allow = r.header("allow").unwrap_or("");
    assert!(allow.contains("GET"), "allow={allow}");
}

#[test]
fn m10_method_not_allowed_allow_lists_all() {
    let mut app = App::new();
    app.route("GET", "/x").handler(|_| Ok(Outcome::json(JsonValue::Null)));
    app.route("PUT", "/x").handler(|_| Ok(Outcome::json(JsonValue::Null)));
    let r = TestClient::new(app).request("POST", "/x", &[], None, &[]);
    assert_eq!(r.status_code(), 405);
    let allow = r.header("allow").unwrap_or("");
    assert!(allow.contains("GET") && allow.contains("PUT"), "allow={allow}");
}

#[test]
fn m10_dep_order_nested() {
    let dep_a = Dependency::build()
        .query_int("a", true, None)
        .handler(|ctx| Ok(JsonValue::Int(ctx.query_int("a").unwrap_or(0))));
    let dep_b = Dependency::build()
        .query_int("b", true, None)
        .depend("aa", dep_a)
        .handler(|ctx| {
            Ok(json_obj(&[
                ("a", ctx.dep("aa").clone()),
                ("b", JsonValue::Int(ctx.query_int("b").unwrap_or(0))),
            ]))
        });
    let mut app = App::new();
    app.route("GET", "/dep-order")
        .depend("d", dep_b)
        .handler(|ctx| Ok(Outcome::json(ctx.dep("d").clone())));
    let j = TestClient::new(app)
        .get_query("/dep-order", &[("a", "1"), ("b", "2")])
        .json();
    assert_eq!(j, json_obj(&[("a", JsonValue::Int(1)), ("b", JsonValue::Int(2))]));
}

#[test]
fn m10_put_and_patch() {
    let mut app = App::new();
    app.route("PUT", "/i/{id}")
        .path_int("id")
        .body_json(true)
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[
                ("id", JsonValue::Int(ctx.path_int("id"))),
                ("body", ctx.body().cloned().unwrap_or(JsonValue::Null)),
            ])))
        });
    app.route("PATCH", "/i/{id}")
        .path_int("id")
        .body_json(true)
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[
                ("patch", JsonValue::Bool(true)),
                ("id", JsonValue::Int(ctx.path_int("id"))),
            ])))
        });
    let c = TestClient::new(app);
    let r = c.put_json("/i/3", &json_obj(&[("k", json_str("v"))]));
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("id"), Some(&JsonValue::Int(3)));
    let r = c.patch_json("/i/3", &json_obj(&[]));
    assert_eq!(r.json().get("patch"), Some(&JsonValue::Bool(true)));
}

#[test]
fn m10_query_empty_string_is_present() {
    let mut app = App::new();
    app.route("GET", "/q")
        .query_str("q", true, None)
        .handler(|ctx| Ok(Outcome::json(json_str(ctx.query_str("q").unwrap_or("MISSING")))));
    // empty value still present
    let r = TestClient::new(app).get_query("/q", &[("q", "")]);
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json(), json_str(""));
}
