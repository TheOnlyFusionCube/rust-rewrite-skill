//! Port of Python inilib: simple INI parser + Config API.
//! Implement parse/Config so milestone tests under tests/ pass.
//! Do not weaken, delete, or edit files under tests/. Prefer std-only.

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Config {
    // Intentionally empty / unused — fill in your fields.
    _priv: (),
}

impl Config {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    pub fn ensure_section(&mut self, _section: &str) {
        todo!("ensure_section")
    }

    pub fn set(&mut self, _section: &str, _key: &str, _value: &str) {
        todo!("set")
    }

    pub fn has_section(&self, _section: &str) -> bool {
        todo!("has_section")
    }

    /// Return Some(value) if present, else `default` (None means missing → None
    /// unless a non-None default is provided — mirror Python get).
    pub fn get(&self, _section: &str, _key: &str, default: Option<&str>) -> Option<String> {
        let _ = default;
        todo!("get")
    }

    pub fn get_bool(&self, _section: &str, _key: &str, default: bool) -> bool {
        let _ = default;
        todo!("get_bool")
    }

    pub fn get_int(&self, _section: &str, _key: &str, default: i64) -> i64 {
        let _ = default;
        todo!("get_int")
    }
}

/// Parse INI-like text into a Config.
/// Rules: [section], key=value, #/;/blank ignored; keys/values trimmed;
/// if key=value appears before any section, use section "DEFAULT".
pub fn parse(_text: &str) -> Config {
    todo!("parse")
}

// Silence unused import in stub; remove when implementing.
#[allow(dead_code)]
fn _keep_hashmap() {
    let _: HashMap<String, String> = HashMap::new();
}
