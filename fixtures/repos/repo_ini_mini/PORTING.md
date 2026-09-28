# PORTING.md — Python ini_mini → Rust

This is an **inih-shaped mini** INI parser, **not** a full inih / configparser port.

## Type / concept map

| Python | Rust |
|--------|------|
| `parse(text) -> Config` | `parse(text: &str) -> Result<Config, IniError>` |
| `IniError` (`.kind`, `.message` / `str(e)`) | `IniError` enum; `err.to_string()` / `Display` matches exact messages |
| `Config()` | `Config::new()` |
| `cfg.ensure_section(s)` | `cfg.ensure_section(s)` |
| `cfg.set(s, k, v)` | `cfg.set(s, k, v)` |
| `cfg.sections()` | `cfg.sections() -> Vec<String>` insertion order |
| `cfg.keys(s)` | `cfg.keys(s) -> Result<Vec<String>, IniError>` |
| `cfg.has_section` / `has_key` | same → `bool` |
| `cfg.get_str(s, k)` | `cfg.get_str(s, k) -> Result<String, IniError>` |
| `cfg.get_str_or(s, k, d)` | `cfg.get_str_or(s, k, d) -> String` |
| `cfg.get_int` / `get_int_or` | `Result<i64, _>` / `i64` |
| `cfg.get_bool` / `get_bool_or` | `Result<bool, _>` / `bool` |
| `cfg.to_string()` | `cfg.to_string() -> String` (or `Display`) |

## Error message shapes (exact)

| Kind | Message |
|------|---------|
| parse unclosed | `unclosed section on line {N}` |
| parse garbage | `section header garbage on line {N}` |
| parse empty key | `empty key on line {N}` |
| missing_section | `section '{name}' not found` |
| missing_key | `key '{key}' not found in section '{name}'` |
| bad_int | `value '{raw}' for '{sec}.{key}' is not a valid integer` |
| bad_bool | `value '{raw}' for '{sec}.{key}' is not a valid boolean` |

## Milestone order

`m01_` → `m02_` → `m03_` → `m04_` → `m05_` → `m06_` → `m07_` → `m08_`

Run: `cargo test m01_`, then `m02_`, …

## Hard rules

- **Never** edit, weaken, delete, or skip files under `rust/tests/`.
- Prefer **std-only** Rust.
- Leave `source/` intact as the behavioral oracle.
- Match README quirk rows and error strings **exactly**.
