# PORTING.md — C inih (handler API) → Rust

Real upstream: `source/upstream/ini.{h,c}` @ `source/upstream/COMMIT_SHA`.
Match `source/README.md` quirk table and `source/golden/*.events`.

## Type / API map

| C (inih) | Rust (`inih_real`) |
|----------|--------------------|
| `ini_handler(user, section, name, value)` | `Event { section, name, value }` |
| `ini_parse_string(s, handler, user)` | `parse_string(text) -> ParseOutcome` |
| return `0` / line / `-1`/`-2` | `ParseOutcome.error_line` (`0` ok, `>0` first error line) |
| (handler accumulates) | `ParseOutcome.events: Vec<Event>` in call order |
| (app last-wins map) | `Ini::parse` + `Ini::get(section, name) -> Option<&str>` last-wins |
| section before any `[...]` | `section == ""` |
| multiline continuation | extra `Event` with same `name`, continuation as `value` |

```rust
pub struct Event {
    pub section: String,
    pub name: String,
    pub value: String,
}

pub struct ParseOutcome {
    pub events: Vec<Event>,
    pub error_line: i32, // 0 = success
}

pub fn parse_string(text: &str) -> ParseOutcome;

pub struct Ini { /* from ParseOutcome */ }
impl Ini {
    pub fn parse(text: &str) -> Self;
    pub fn error_line(&self) -> i32;
    pub fn events(&self) -> &[Event];
    pub fn get(&self, section: &str, name: &str) -> Option<&str>; // last-wins
    pub fn has_section(&self, section: &str) -> bool;
    /// First-seen section names in order. Includes `""` if any pre-section keys.
    pub fn sections(&self) -> Vec<String>;
}
```

## Milestone order

`m01_` → `m02_` → `m03_` → `m04_` → `m05_` → `m06_` → `m07_` → `m08_`

Run: `cargo test m01_`, then `m02_`, …

## Hard rules

- **Never** edit, weaken, delete, or skip files under `rust/tests/`.
- Prefer **std-only** Rust.
- Leave `source/` intact (upstream + oracle + goldens).
- Match golden event streams and README quirks **exactly** (including `section == ""`, untrimmed section names, multiline as separate events).
- Do **not** invent `"DEFAULT"` for pre-section keys — inih uses `""`.
