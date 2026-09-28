//! inih_real — bounded Rust port target for real upstream benhoyt/inih.
//! Match `../source/README.md` + `../PORTING.md` + `../source/golden/*.events`.
//! Do NOT edit, weaken, or delete files under `tests/`. Prefer std-only.
//!
//! This stub compiles; milestone tests should fail until implemented.

#![allow(unused_variables, dead_code)]

/// One inih handler callback equivalent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub section: String,
    pub name: String,
    pub value: String,
}

/// Result of parsing: ordered events + first error line (0 = success).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutcome {
    pub events: Vec<Event>,
    /// 0 = success; >0 = 1-based line of first parse error (inih keeps parsing).
    pub error_line: i32,
}

/// Parse INI text with default inih semantics (see source/README.md).
pub fn parse_string(_text: &str) -> ParseOutcome {
    // Stub: no events, pretend success so compile works; tests will fail on content.
    ParseOutcome {
        events: Vec::new(),
        error_line: 0,
    }
}

/// Convenience last-wins view over parse events.
#[derive(Debug, Clone)]
pub struct Ini {
    outcome: ParseOutcome,
}

impl Ini {
    pub fn parse(text: &str) -> Self {
        Self {
            outcome: parse_string(text),
        }
    }

    pub fn error_line(&self) -> i32 {
        self.outcome.error_line
    }

    pub fn events(&self) -> &[Event] {
        &self.outcome.events
    }

    /// Last-wins value for (section, name). Multiline continuations: last event wins.
    pub fn get(&self, _section: &str, _name: &str) -> Option<&str> {
        None
    }

    pub fn has_section(&self, _section: &str) -> bool {
        false
    }

    /// First-seen section names in encounter order. Includes `""` if pre-section keys exist.
    pub fn sections(&self) -> Vec<String> {
        Vec::new()
    }
}
