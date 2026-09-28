//! m05: Depends injection — nested deps, failure propagation
use fastapi_mini::{json_obj, json_str, App, Dependency, HTTPException, JsonValue, Outcome, TestClient};

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
fn m05_simple_depend() {
    let dep = Dependency::build()
        .query_str("token", true, None)
        .handler(|ctx| Ok(json_str(ctx.query_str("token").unwrap_or(""))));
    let mut app = App::new();
    app.route("GET", "/t")
        .depend("tok", dep)
        .handler(|ctx| Ok(Outcome::json(json_obj(&[("tok", ctx.dep("tok").clone())]))));
    let c = TestClient::new(app);
    let r = c.get_query("/t", &[("token", "abc")]);
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("tok"), Some(&json_str("abc")));
}

#[test]
fn m05_nested_depends() {
    let get_token = Dependency::build()
        .query_str("token", true, None)
        .handler(|ctx| {
            let t = ctx.query_str("token").unwrap_or("");
            if t != "secret" {
                return Err(HTTPException::str(403, "Forbidden"));
            }
            Ok(json_str(t))
        });
    let get_user = Dependency::build()
        .depend("tok", get_token)
        .handler(|ctx| {
            Ok(json_obj(&[
                ("user", json_str("alice")),
                ("token", ctx.dep("tok").clone()),
            ]))
        });
    let mut app = App::new();
    app.route("GET", "/me")
        .depend("user", get_user)
        .handler(|ctx| Ok(Outcome::json(ctx.dep("user").clone())));
    let c = TestClient::new(app);
    let r = c.get_query("/me", &[("token", "secret")]);
    assert_eq!(r.status_code(), 200);
    assert_eq!(r.json().get("user"), Some(&json_str("alice")));
    assert_eq!(r.json().get("token"), Some(&json_str("secret")));
}

#[test]
fn m05_depend_http_exception_propagates() {
    let get_token = Dependency::build()
        .query_str("token", true, None)
        .handler(|ctx| {
            let t = ctx.query_str("token").unwrap_or("");
            if t != "secret" {
                return Err(HTTPException::str(403, "Forbidden"));
            }
            Ok(json_str(t))
        });
    let mut app = App::new();
    app.route("GET", "/me")
        .depend("tok", get_token)
        .handler(|ctx| Ok(Outcome::json(ctx.dep("tok").clone())));
    let c = TestClient::new(app);
    let r = c.get_query("/me", &[("token", "nope")]);
    assert_eq!(r.status_code(), 403);
    assert_eq!(r.json(), json_obj(&[("detail", json_str("Forbidden"))]));
}

#[test]
fn m05_depend_missing_query_422() {
    let dep = Dependency::build()
        .query_str("token", true, None)
        .handler(|ctx| Ok(json_str(ctx.query_str("token").unwrap_or(""))));
    let mut app = App::new();
    app.route("GET", "/t")
        .depend("tok", dep)
        .handler(|ctx| Ok(Outcome::json(ctx.dep("tok").clone())));
    let c = TestClient::new(app);
    let r = c.get("/t");
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["query", "token"], "value_error.missing"));
}

#[test]
fn m05_depend_int_coercion() {
    let dep = Dependency::build()
        .query_int("n", true, None)
        .handler(|ctx| Ok(JsonValue::Int(ctx.query_int("n").unwrap_or(0))));
    let mut app = App::new();
    app.route("GET", "/n")
        .depend("n", dep)
        .handler(|ctx| Ok(Outcome::json(ctx.dep("n").clone())));
    let c = TestClient::new(app);
    assert_eq!(c.get_query("/n", &[("n", "9")]).json(), JsonValue::Int(9));
    let r = c.get_query("/n", &[("n", "x")]);
    assert_eq!(r.status_code(), 422);
    assert!(detail_has(&r.json(), &["query", "n"], "type_error.integer"));
}

#[test]
fn m05_multi_depend_order() {
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
    let c = TestClient::new(app);
    let j = c.get_query("/dep-order", &[("a", "1"), ("b", "2")]).json();
    assert_eq!(j, json_obj(&[("a", JsonValue::Int(1)), ("b", JsonValue::Int(2))]));
}
