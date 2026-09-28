# repo_inih_real — real upstream inih (subset)

**This is real upstream source** ([benhoyt/inih](https://github.com/benhoyt/inih))
vendored under `upstream/` at the commit in `upstream/COMMIT_SHA`, plus a C oracle
and frozen golden event dumps. It is **NOT** a claim that the Rust crate is a
complete inih rewrite — only the behaviors exercised by `rust/tests/m*.rs` and
reflected in `golden/*.events` are in scope.

## What is frozen

| Item | Choice (inih defaults unless noted) |
|------|-------------------------------------|
| API shape | Handler-style: each `name=value` → one event `(section, name, value)` |
| Keys before `[section]` | `section == ""` (empty string), **not** `"DEFAULT"` |
| Section name trim | **Not trimmed** — `[ section 2 ]` → `" section 2 "` |
| Key / value trim | Leading+trailing whitespace stripped |
| Assignments | `key=value` **or** `key:value` (first `=` or `:`) |
| Duplicate keys | Multiple events; map-style `get` is **last-wins** |
| Duplicate `[section]` | Later keys join same section name; events keep order |
| Start-of-line comments | First non-ws char in `;#` |
| Inline comments | `;` only, and only after whitespace (`INI_INLINE_COMMENT_PREFIXES=";"`) |
| `#` inline | **Not** an inline comment — stays in value |
| Multiline | Enabled: non-blank line with **leading whitespace** continues prior name → **separate event** (same name, continuation value) |
| UTF-8 BOM | Allowed at start of text (`EF BB BF`) |
| No-value lines | Error (line number); parsing continues; later pairs still emit |
| Bad `[section` (no `]`) | Error line; section name unchanged; later pairs still emit |
| Return code | `error_line == 0` success; `>0` = 1-based first error line; (file open `-1` N/A for string parse) |
| Buffer limits in C oracle | `-DINI_MAX_LINE=1024 -DINI_MAX_SECTION=256 -DINI_MAX_NAME=256` (larger than stock 200/50/50). Rust should not silently truncate within these bounds. |
| `INI_ALLOW_NO_VALUE` | **0** (default) |
| `INI_STOP_ON_FIRST_ERROR` | **0** (keep parsing) |
| `INI_CALL_HANDLER_ON_NEW_SECTION` | **0** |

## Oracle

```bash
cd source/oracle
cc -O2 -DINI_MAX_LINE=1024 -DINI_MAX_SECTION=256 -DINI_MAX_NAME=256 \
  -o oracle_dump oracle_dump.c ../upstream/ini.c
./oracle_dump ../fixtures/01_basic.ini
```

Output lines:
- `S=<section>|N=<name>|V=<value>` per handler call (escapes: `\\` `\|` `\n` `\r` `\t`)
- final `e=<error_line>`

## Self-check

```bash
cd source && python3 selfcheck.py
```

Must exit 0 before model runs (rebuilds oracle, diffs goldens).

## Honest scope note

Scaffolding + real upstream C + differential goldens for a **bounded subset**.
Does **not** claim full feature parity with every inih compile-time knob,
heap/stack modes, custom allocators, or the C++ INIReader.
