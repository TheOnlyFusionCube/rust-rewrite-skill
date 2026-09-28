# ini_mini — inih-shaped INI parser (Python oracle)

**NOT** full [inih](https://github.com/benhoyt/inih) / configparser. Mini flat-section
INI with deliberate quirks for RIIR blood tests. Behavioral oracle for `../rust/`.
See also `../PORTING.md`.

## Contract / quirks (must match Rust)

| Behavior | Choice |
|----------|--------|
| Sections | `[name]` — name trimmed inside brackets; empty name `[]` allowed |
| Duplicate section headers | **Merge** into the same section (keys continue) |
| Keys before any section | Implicit section `DEFAULT` |
| Assignments | `key=value` — key and value **trimmed**; empty value OK |
| Duplicate keys | **Last-wins** (value updated; first-seen key order kept for stringify) |
| Comments | Line whose first non-ws char is `#` or `;` — **no** inline comments |
| Blank / non-`=` lines | Ignored (lenient) |
| Unclosed `[foo` | `IniError` message: `unclosed section on line N` (1-based) |
| Junk after `]` | `section header garbage on line N` |
| Empty key (`=v`) | `empty key on line N` |
| Case | Section and key names are **case-sensitive** |
| Escapes in values | `\\` `\\n` `\\t` `\\#` `\\;` ; unknown `\\X` → `X` |
| `get_str` missing section | `section 'NAME' not found` |
| `get_str` missing key | `key 'KEY' not found in section 'NAME'` |
| `get_int` | optional `+/-` + digits only; else `value 'V' for 'SEC.KEY' is not a valid integer` |
| `get_bool` | true: `true/yes/1/on`; false: `false/no/0/off` (case-insensitive); else `value 'V' for 'SEC.KEY' is not a valid boolean` |
| `*_or` helpers | Return default on missing section/key **or** bad type |
| `to_string` / stringify | `[section]\\nkey = value\\n` ; blank line between sections; values re-escaped |
| Unicode | UTF-8 preserved in names and values |
| Nested sections | **Not supported** (flat only) |

## Python API

```python
from ini_mini import parse, Config, IniError

cfg = parse("[db]\\nhost = localhost\\nport = 5432\\n")
assert cfg.get_str("db", "host") == "localhost"
assert cfg.get_int("db", "port") == 5432
assert cfg.get_bool_or("db", "ssl", False) is False
text = cfg.to_string()
```

## Self-check

```bash
cd source && PYTHONPATH=. python3 -m tests.selfcheck
```
