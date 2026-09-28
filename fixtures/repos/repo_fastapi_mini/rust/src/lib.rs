//! fastapi_mini — FastAPI-API-compatible subset (NOT full FastAPI).
//! Port from `../source/`. Match README + PORTING.md contracts.
//! Do NOT edit, weaken, or delete files under `tests/`.

#![allow(unused_variables, dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

// ───────────────────────── JsonValue ─────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn parse(s: &str) -> Result<JsonValue, String> {
        let s = s.trim();
        let mut p = Parser { s: s.as_bytes(), i: 0 };
        let v = p.parse_value()?;
        p.skip_ws();
        if p.i != p.s.len() {
            return Err("trailing".into());
        }
        Ok(v)
    }

    pub fn as_str_value(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(m) => m.get(key),
            _ => None,
        }
    }

    pub fn to_string_compact(&self) -> String {
        match self {
            JsonValue::Null => "null".into(),
            JsonValue::Bool(b) => if *b { "true" } else { "false" }.into(),
            JsonValue::Int(i) => i.to_string(),
            JsonValue::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.1}", f)
                } else {
                    let s = f.to_string();
                    s
                }
            }
            JsonValue::Str(s) => format!("\"{}\"", escape_json(s)),
            JsonValue::Array(a) => {
                let parts: Vec<String> = a.iter().map(|v| v.to_string_compact()).collect();
                format!("[{}]", parts.join(","))
            }
            JsonValue::Object(m) => {
                let parts: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", escape_json(k), v.to_string_compact()))
                    .collect();
                format!("{{{}}}", parts.join(","))
            }
        }
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        if self.i >= self.s.len() {
            return Err("eof".into());
        }
        match self.s[self.i] {
            b'n' => self.consume_lit(b"null").map(|_| JsonValue::Null),
            b't' => self.consume_lit(b"true").map(|_| JsonValue::Bool(true)),
            b'f' => self.consume_lit(b"false").map(|_| JsonValue::Bool(false)),
            b'"' => self.parse_string().map(JsonValue::Str),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err("bad".into()),
        }
    }
    fn consume_lit(&mut self, lit: &[u8]) -> Result<(), String> {
        if self.s[self.i..].starts_with(lit) {
            self.i += lit.len();
            Ok(())
        } else {
            Err("lit".into())
        }
    }
    fn parse_string(&mut self) -> Result<String, String> {
        self.i += 1;
        let mut out = String::new();
        while self.i < self.s.len() {
            let c = self.s[self.i];
            self.i += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    if self.i >= self.s.len() {
                        return Err("esc".into());
                    }
                    let e = self.s[self.i];
                    self.i += 1;
                    match e {
                        b'"' | b'\\' | b'/' => out.push(e as char),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            if self.i + 4 > self.s.len() {
                                return Err("u".into());
                            }
                            let hex = std::str::from_utf8(&self.s[self.i..self.i + 4]).map_err(|_| "u")?;
                            let cp = u16::from_str_radix(hex, 16).map_err(|_| "u")?;
                            out.push(char::from_u32(cp as u32).unwrap_or('\u{FFFD}'));
                            self.i += 4;
                        }
                        _ => return Err("esc".into()),
                    }
                }
                _ => out.push(c as char),
            }
        }
        Err("unterminated".into())
    }
    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.i;
        if self.s[self.i] == b'-' {
            self.i += 1;
        }
        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            self.i += 1;
        }
        let mut is_float = false;
        if self.i < self.s.len() && self.s[self.i] == b'.' {
            is_float = true;
            self.i += 1;
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
        }
        if self.i < self.s.len() && (self.s[self.i] == b'e' || self.s[self.i] == b'E') {
            is_float = true;
            self.i += 1;
            if self.i < self.s.len() && (self.s[self.i] == b'+' || self.s[self.i] == b'-') {
                self.i += 1;
            }
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
        }
        let raw = std::str::from_utf8(&self.s[start..self.i]).map_err(|_| "num")?;
        if is_float {
            Ok(JsonValue::Float(raw.parse().map_err(|_| "num")?))
        } else {
            Ok(JsonValue::Int(raw.parse().map_err(|_| "num")?))
        }
    }
    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.i += 1;
        let mut arr = Vec::new();
        self.skip_ws();
        if self.i < self.s.len() && self.s[self.i] == b']' {
            self.i += 1;
            return Ok(JsonValue::Array(arr));
        }
        loop {
            arr.push(self.parse_value()?);
            self.skip_ws();
            if self.i >= self.s.len() {
                return Err("arr".into());
            }
            if self.s[self.i] == b']' {
                self.i += 1;
                break;
            }
            if self.s[self.i] != b',' {
                return Err("arr".into());
            }
            self.i += 1;
        }
        Ok(JsonValue::Array(arr))
    }
    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.i += 1;
        let mut map = BTreeMap::new();
        self.skip_ws();
        if self.i < self.s.len() && self.s[self.i] == b'}' {
            self.i += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_ws();
            if self.i >= self.s.len() || self.s[self.i] != b'"' {
                return Err("obj".into());
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.i >= self.s.len() || self.s[self.i] != b':' {
                return Err("obj".into());
            }
            self.i += 1;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            if self.i >= self.s.len() {
                return Err("obj".into());
            }
            if self.s[self.i] == b'}' {
                self.i += 1;
                break;
            }
            if self.s[self.i] != b',' {
                return Err("obj".into());
            }
            self.i += 1;
        }
        Ok(JsonValue::Object(map))
    }
}

pub fn json_obj(pairs: &[(&str, JsonValue)]) -> JsonValue {
    let mut m = BTreeMap::new();
    for (k, v) in pairs {
        m.insert((*k).to_string(), v.clone());
    }
    JsonValue::Object(m)
}

pub fn json_str(s: &str) -> JsonValue {
    JsonValue::Str(s.to_string())
}

// ───────────────────────── Errors / Response ─────────────────────────

#[derive(Debug, Clone)]
pub struct HTTPException {
    pub status_code: u16,
    pub detail: JsonValue,
}

impl HTTPException {
    pub fn new(status_code: u16, detail: JsonValue) -> Self {
        Self { status_code, detail }
    }
    pub fn str(status_code: u16, detail: &str) -> Self {
        Self {
            status_code,
            detail: JsonValue::Str(detail.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: u16,
    pub body: String,
    pub headers: BTreeMap<String, String>,
}

impl Response {
    pub fn json(status_code: u16, value: &JsonValue) -> Self {
        let mut headers = BTreeMap::new();
        headers.insert("content-type".into(), "application/json".into());
        Self {
            status_code,
            body: value.to_string_compact(),
            headers,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Json(JsonValue),
    JsonStatus(JsonValue, u16),
    Response(Response),
}

impl Outcome {
    pub fn json(v: JsonValue) -> Self {
        Outcome::Json(v)
    }
}

// ───────────────────────── Context ─────────────────────────

#[derive(Debug, Clone)]
pub struct Context {
    pub path_params: BTreeMap<String, String>,
    pub query_params: BTreeMap<String, String>,
    pub body: Option<JsonValue>,
    pub deps: BTreeMap<String, JsonValue>,
}

impl Context {
    pub fn path_str(&self, name: &str) -> &str {
        self.path_params.get(name).map(|s| s.as_str()).unwrap_or("")
    }
    pub fn path_int(&self, name: &str) -> i64 {
        self.path_params
            .get(name)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }
    pub fn query_str(&self, name: &str) -> Option<&str> {
        self.query_params.get(name).map(|s| s.as_str())
    }
    pub fn query_int(&self, name: &str) -> Option<i64> {
        self.query_params.get(name).and_then(|s| s.parse().ok())
    }
    pub fn query_float(&self, name: &str) -> Option<f64> {
        self.query_params.get(name).and_then(|s| s.parse().ok())
    }
    pub fn query_bool(&self, name: &str) -> Option<bool> {
        self.query_params.get(name).map(|s| {
            let l = s.to_ascii_lowercase();
            l == "true" || l == "1"
        })
    }
    pub fn body(&self) -> Option<&JsonValue> {
        self.body.as_ref()
    }
    pub fn dep(&self, name: &str) -> &JsonValue {
        self.deps.get(name).unwrap_or(&JsonValue::Null)
    }
}

// ───────────────────────── Param specs / Dependency ─────────────────────────

#[derive(Clone)]
pub enum ParamType {
    Str,
    Int,
    Float,
    Bool,
}

#[derive(Clone)]
struct PathParam {
    name: String,
    ty: ParamType,
}

#[derive(Clone)]
struct QueryParam {
    name: String,
    ty: ParamType,
    required: bool,
    default: Option<String>,
}

#[derive(Clone)]
struct BodyParam {
    required: bool,
}

type HandlerFn = Arc<dyn Fn(&Context) -> Result<Outcome, HTTPException> + Send + Sync>;
type DepFn = Arc<dyn Fn(&Context) -> Result<JsonValue, HTTPException> + Send + Sync>;

#[derive(Clone)]
pub struct Dependency {
    query: Vec<QueryParam>,
    path: Vec<PathParam>,
    body: Option<BodyParam>,
    deps: Vec<(String, Dependency)>,
    handler: Option<DepFn>,
}

impl Dependency {
    pub fn build() -> Self {
        Self {
            query: vec![],
            path: vec![],
            body: None,
            deps: vec![],
            handler: None,
        }
    }
    pub fn query_str(mut self, name: &str, required: bool, default: Option<&str>) -> Self {
        self.query.push(QueryParam {
            name: name.into(),
            ty: ParamType::Str,
            required,
            default: default.map(|s| s.to_string()),
        });
        self
    }
    pub fn query_int(mut self, name: &str, required: bool, default: Option<i64>) -> Self {
        self.query.push(QueryParam {
            name: name.into(),
            ty: ParamType::Int,
            required,
            default: default.map(|i| i.to_string()),
        });
        self
    }
    pub fn depend(mut self, name: &str, dep: Dependency) -> Self {
        self.deps.push((name.into(), dep));
        self
    }
    pub fn handler<F>(mut self, f: F) -> Self
    where
        F: Fn(&Context) -> Result<JsonValue, HTTPException> + Send + Sync + 'static,
    {
        self.handler = Some(Arc::new(f));
        self
    }
}

// ───────────────────────── Router / App ─────────────────────────

#[derive(Clone)]
struct Route {
    method: String,
    path: String,
    status_code: u16,
    response_model: Option<Vec<String>>,
    tags: Vec<String>,
    name: String,
    path_params: Vec<PathParam>,
    query_params: Vec<QueryParam>,
    body: Option<BodyParam>,
    deps: Vec<(String, Dependency)>,
    handler: Option<HandlerFn>,
}

pub struct RouteBuilder {
    router: *mut APIRouter,
    route: Route,
}

pub struct APIRouter {
    routes: Vec<Route>,
    prefix: String,
    tags: Vec<String>,
}

pub struct App {
    router: APIRouter,
    title: String,
    version: String,
}

impl APIRouter {
    pub fn new() -> Self {
        Self {
            routes: vec![],
            prefix: String::new(),
            tags: vec![],
        }
    }

    pub fn route(&mut self, method: &str, path: &str) -> RouteBuilder {
        let r = Route {
            method: method.to_uppercase(),
            path: normalize_path(path),
            status_code: 200,
            response_model: None,
            tags: self.tags.clone(),
            name: String::new(),
            path_params: vec![],
            query_params: vec![],
            body: None,
            deps: vec![],
            handler: None,
        };
        RouteBuilder {
            router: self as *mut APIRouter,
            route: r,
        }
    }

    pub fn include_router(&mut self, other: APIRouter, prefix: &str, tags: &[&str]) {
        let pref = normalize_path(prefix);
        let pref = if pref == "/" { String::new() } else { pref };
        for mut r in other.routes {
            r.path = normalize_path(&(pref.clone() + &r.path));
            for t in tags {
                r.tags.push((*t).to_string());
            }
            self.routes.push(r);
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_meta("fastapi_mini", "0.1.0")
    }
    pub fn with_meta(title: &str, version: &str) -> Self {
        Self {
            router: APIRouter::new(),
            title: title.into(),
            version: version.into(),
        }
    }
    pub fn route(&mut self, method: &str, path: &str) -> RouteBuilder {
        self.router.route(method, path)
    }
    pub fn include_router(&mut self, other: APIRouter, prefix: &str, tags: &[&str]) {
        self.router.include_router(other, prefix, tags);
    }
    /// STUB: returns empty OpenAPI-ish JSON — model must implement exact schema.
    pub fn openapi(&self) -> JsonValue {
        json_obj(&[
            ("openapi", json_str("3.0.3")),
            (
                "info",
                json_obj(&[
                    ("title", json_str(&self.title)),
                    ("version", json_str(&self.version)),
                ]),
            ),
            ("paths", JsonValue::Object(BTreeMap::new())),
        ])
    }
    pub fn handle(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> Response {
        let _ = (method, path, query, body, headers);
        // STUB: always 404 — implement real routing/validation.
        Response::json(404, &json_obj(&[("detail", json_str("Not Found"))]))
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteBuilder {
    pub fn status_code(mut self, code: u16) -> Self {
        self.route.status_code = code;
        self
    }
    pub fn response_model(mut self, fields: &[&str]) -> Self {
        self.route.response_model = Some(fields.iter().map(|s| (*s).to_string()).collect());
        self
    }
    pub fn tags(mut self, tags: &[&str]) -> Self {
        for t in tags {
            self.route.tags.push((*t).to_string());
        }
        self
    }
    pub fn name(mut self, name: &str) -> Self {
        self.route.name = name.into();
        self
    }
    pub fn path_str(mut self, name: &str) -> Self {
        self.route.path_params.push(PathParam {
            name: name.into(),
            ty: ParamType::Str,
        });
        self
    }
    pub fn path_int(mut self, name: &str) -> Self {
        self.route.path_params.push(PathParam {
            name: name.into(),
            ty: ParamType::Int,
        });
        self
    }
    pub fn query_str(mut self, name: &str, required: bool, default: Option<&str>) -> Self {
        self.route.query_params.push(QueryParam {
            name: name.into(),
            ty: ParamType::Str,
            required,
            default: default.map(|s| s.to_string()),
        });
        self
    }
    pub fn query_int(mut self, name: &str, required: bool, default: Option<i64>) -> Self {
        self.route.query_params.push(QueryParam {
            name: name.into(),
            ty: ParamType::Int,
            required,
            default: default.map(|i| i.to_string()),
        });
        self
    }
    pub fn query_float(mut self, name: &str, required: bool, default: Option<f64>) -> Self {
        self.route.query_params.push(QueryParam {
            name: name.into(),
            ty: ParamType::Float,
            required,
            default: default.map(|f| f.to_string()),
        });
        self
    }
    pub fn query_bool(mut self, name: &str, required: bool, default: Option<bool>) -> Self {
        self.route.query_params.push(QueryParam {
            name: name.into(),
            ty: ParamType::Bool,
            required,
            default: default.map(|b| if b { "true" } else { "false" }.to_string()),
        });
        self
    }
    pub fn body_json(mut self, required: bool) -> Self {
        self.route.body = Some(BodyParam { required });
        self
    }
    pub fn depend(mut self, name: &str, dep: Dependency) -> Self {
        self.route.deps.push((name.into(), dep));
        self
    }
    pub fn handler<F>(mut self, f: F)
    where
        F: Fn(&Context) -> Result<Outcome, HTTPException> + Send + Sync + 'static,
    {
        self.route.handler = Some(Arc::new(f));
        if self.route.name.is_empty() {
            self.route.name = "handler".into();
        }
        unsafe {
            (*self.router).routes.push(self.route);
        }
    }
}

fn normalize_path(path: &str) -> String {
    let mut p = path.to_string();
    if !p.starts_with('/') {
        p.insert(0, '/');
    }
    if p.len() > 1 && p.ends_with('/') {
        p.pop();
    }
    p
}

// ───────────────────────── TestClient ─────────────────────────

pub struct ClientResponse {
    inner: Response,
}

impl ClientResponse {
    pub fn status_code(&self) -> u16 {
        self.inner.status_code
    }
    pub fn text(&self) -> &str {
        &self.inner.body
    }
    pub fn json(&self) -> JsonValue {
        JsonValue::parse(&self.inner.body).unwrap_or(JsonValue::Null)
    }
    pub fn header(&self, name: &str) -> Option<&str> {
        self.inner
            .headers
            .get(&name.to_ascii_lowercase())
            .map(|s| s.as_str())
    }
}

pub struct TestClient {
    app: App,
}

impl TestClient {
    pub fn new(app: App) -> Self {
        Self { app }
    }
    pub fn request(
        &self,
        method: &str,
        url: &str,
        query: &[(&str, &str)],
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> ClientResponse {
        ClientResponse {
            inner: self.app.handle(method, url, query, body, headers),
        }
    }
    pub fn get(&self, url: &str) -> ClientResponse {
        self.request("GET", url, &[], None, &[])
    }
    pub fn get_query(&self, url: &str, query: &[(&str, &str)]) -> ClientResponse {
        self.request("GET", url, query, None, &[])
    }
    pub fn post_json(&self, url: &str, body: &JsonValue) -> ClientResponse {
        self.request(
            "POST",
            url,
            &[],
            Some(&body.to_string_compact()),
            &[("content-type", "application/json")],
        )
    }
    pub fn post_raw(&self, url: &str, body: &str, content_type: &str) -> ClientResponse {
        self.request(
            "POST",
            url,
            &[],
            Some(body),
            &[("content-type", content_type)],
        )
    }
    pub fn put_json(&self, url: &str, body: &JsonValue) -> ClientResponse {
        self.request(
            "PUT",
            url,
            &[],
            Some(&body.to_string_compact()),
            &[("content-type", "application/json")],
        )
    }
    pub fn delete(&self, url: &str) -> ClientResponse {
        self.request("DELETE", url, &[], None, &[])
    }
    pub fn patch_json(&self, url: &str, body: &JsonValue) -> ClientResponse {
        self.request(
            "PATCH",
            url,
            &[],
            Some(&body.to_string_compact()),
            &[("content-type", "application/json")],
        )
    }
}
