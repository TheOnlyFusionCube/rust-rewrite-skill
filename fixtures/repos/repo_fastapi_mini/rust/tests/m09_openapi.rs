//! m09: OpenAPI schema exactness
use fastapi_mini::{json_obj, App, JsonValue, Outcome};

#[test]
fn m09_openapi_version_and_info() {
    let app = App::with_meta("Demo", "2.0.0");
    let s = app.openapi();
    assert_eq!(s.get("openapi"), Some(&JsonValue::Str("3.0.3".into())));
    assert_eq!(
        s.get("info"),
        Some(&json_obj(&[
            ("title", JsonValue::Str("Demo".into())),
            ("version", JsonValue::Str("2.0.0".into())),
        ]))
    );
}

#[test]
fn m09_openapi_get_path_param() {
    let mut app = App::with_meta("Demo", "1.0.0");
    app.route("GET", "/items/{item_id}")
        .path_int("item_id")
        .query_str("q", false, None)
        .tags(&["items"])
        .name("get_item")
        .handler(|ctx| {
            Ok(Outcome::json(json_obj(&[
                ("item_id", JsonValue::Int(ctx.path_int("item_id"))),
            ])))
        });
    let s = app.openapi();
    let paths = s.get("paths").expect("paths");
    let item = paths.get("/items/{item_id}").expect("path key");
    let get = item.get("get").expect("get");
    assert_eq!(get.get("operationId"), Some(&JsonValue::Str("get_item".into())));
    let tags = get.get("tags").unwrap();
    assert!(matches!(tags, JsonValue::Array(a) if a.iter().any(|t| t.as_str_value() == Some("items"))));
    let params = get.get("parameters").expect("parameters");
    let JsonValue::Array(arr) = params else { panic!() };
    let path_p = arr.iter().find(|p| p.get("name").and_then(|n| n.as_str_value()) == Some("item_id"));
    let path_p = path_p.expect("item_id param");
    assert_eq!(path_p.get("in"), Some(&JsonValue::Str("path".into())));
    assert_eq!(path_p.get("required"), Some(&JsonValue::Bool(true)));
    assert_eq!(
        path_p.get("schema").and_then(|sc| sc.get("type")),
        Some(&JsonValue::Str("integer".into()))
    );
    let q = arr.iter().find(|p| p.get("name").and_then(|n| n.as_str_value()) == Some("q"));
    let q = q.expect("q param");
    assert_eq!(q.get("in"), Some(&JsonValue::Str("query".into())));
    assert_eq!(q.get("required"), Some(&JsonValue::Bool(false)));
}

#[test]
fn m09_openapi_post_request_body() {
    let mut app = App::new();
    app.route("POST", "/items")
        .body_json(true)
        .name("create")
        .handler(|ctx| Ok(Outcome::json(ctx.body().cloned().unwrap_or(JsonValue::Null))));
    let s = app.openapi();
    let post = s
        .get("paths")
        .and_then(|p| p.get("/items"))
        .and_then(|p| p.get("post"))
        .expect("post");
    let rb = post.get("requestBody").expect("requestBody");
    assert_eq!(rb.get("required"), Some(&JsonValue::Bool(true)));
    let ct = rb
        .get("content")
        .and_then(|c| c.get("application/json"))
        .and_then(|c| c.get("schema"));
    assert!(ct.is_some());
}

#[test]
fn m09_openapi_response_status_key() {
    let mut app = App::new();
    app.route("POST", "/t")
        .status_code(201)
        .name("t")
        .handler(|_| Ok(Outcome::json(json_obj(&[]))));
    let post = app
        .openapi()
        .get("paths")
        .and_then(|p| p.get("/t"))
        .and_then(|p| p.get("post"))
        .cloned()
        .unwrap();
    let responses = post.get("responses").unwrap();
    assert!(responses.get("201").is_some());
}

#[test]
fn m09_openapi_paths_empty_app() {
    let app = App::new();
    let s = app.openapi();
    assert_eq!(s.get("paths"), Some(&JsonValue::Object(Default::default())));
}

#[test]
fn m09_openapi_two_methods_same_path() {
    let mut app = App::new();
    app.route("GET", "/x").name("gx").handler(|_| Ok(Outcome::json(json_obj(&[]))));
    app.route("POST", "/x").name("px").handler(|_| Ok(Outcome::json(json_obj(&[]))));
    let item = app.openapi().get("paths").and_then(|p| p.get("/x")).cloned().unwrap();
    assert!(item.get("get").is_some());
    assert!(item.get("post").is_some());
}

#[test]
fn m09_default_meta() {
    let app = App::new();
    let info = app.openapi().get("info").cloned().unwrap();
    assert_eq!(info.get("title"), Some(&JsonValue::Str("fastapi_mini".into())));
    assert_eq!(info.get("version"), Some(&JsonValue::Str("0.1.0".into())));
}
