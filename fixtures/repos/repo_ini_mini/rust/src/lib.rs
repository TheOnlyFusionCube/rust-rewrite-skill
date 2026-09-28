//! ini_mini — inih-shaped INI parser (NOT full inih).
//! Port from `../source/`. Match README + PORTING.md contracts.
//! Do NOT edit, weaken, or delete files under `tests/`. Prefer std-only.

#![allow(unused_variables, dead_code)]

use std::fmt;

/// Structured INI error. `Display` / `to_string()` must match README message shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IniError {
    Parse { line: usize, message: String },
    MissingSection { section: String },
    MissingKey { section: String, key: String },
    BadInt {
        section: String,
        key: String,
        value: String,
    },
    BadBool {
        section: String,
        key: String,
        value: String,
    },
}

impl fmt::Display for IniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IniError::Parse { message, .. } => write!(f, "{message}"),
            IniError::MissingSection { section } => {
                write!(f, "section '{section}' not found")
            }
            IniError::MissingKey { section, key } => {
                write!(f, "key '{key}' not found in section '{section}'")
            }
            IniError::BadInt {
                section,
                key,
                value,
            } => write!(
                f,
                "value '{value}' for '{section}.{key}' is not a valid integer"
            ),
            IniError::BadBool {
                section,
                key,
                value,
            } => write!(
                f,
                "value '{value}' for '{section}.{key}' is not a valid boolean"
            ),
        }
    }
}

impl std::error::Error for IniError {}

pub type IniResult<T> = Result<T, IniError>;

/// Flat-section INI table. Fill fields; keep method signatures stable.
#[derive(Debug, Default, Clone)]
pub struct Config {
    // Intentionally empty — implement storage (e.g. IndexMap-like via Vec).
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

    pub fn sections(&self) -> Vec<String> {
        todo!("sections")
    }

    pub fn keys(&self, _section: &str) -> IniResult<Vec<String>> {
        todo!("keys")
    }

    pub fn has_section(&self, _section: &str) -> bool {
        todo!("has_section")
    }

    pub fn has_key(&self, _section: &str, _key: &str) -> bool {
        todo!("has_key")
    }

    pub fn get_str(&self, _section: &str, _key: &str) -> IniResult<String> {
        todo!("get_str")
    }

    pub fn get_str_or(&self, section: &str, key: &str, default: &str) -> String {
        match self.get_str(section, key) {
            Ok(v) => v,
            Err(_) => default.to_string(),
        }
    }

    pub fn get_int(&self, _section: &str, _key: &str) -> IniResult<i64> {
        todo!("get_int")
    }

    pub fn get_int_or(&self, section: &str, key: &str, default: i64) -> i64 {
        match self.get_int(section, key) {
            Ok(v) => v,
            Err(_) => default,
        }
    }

    pub fn get_bool(&self, _section: &str, _key: &str) -> IniResult<bool> {
        todo!("get_bool")
    }

    pub fn get_bool_or(&self, section: &str, key: &str, default: bool) -> bool {
        match self.get_bool(section, key) {
            Ok(v) => v,
            Err(_) => default,
        }
    }

    /// Serialize per README: `[section]\nkey = value\n` + blank line between sections.
    pub fn to_string(&self) -> String {
        todo!("to_string")
    }
}

/// Parse INI-like text. See README quirk table.
pub fn parse(_text: &str) -> IniResult<Config> {
    todo!("parse")
}
