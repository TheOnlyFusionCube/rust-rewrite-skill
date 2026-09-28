# inilib — simple INI/config parser

Parses a minimal INI-like format:
- `[section]` headers
- `key=value` pairs (value is trimmed; key is trimmed)
- Lines starting with `#` or `;` are comments
- Blank lines ignored

API:
- `parse(text) -> Config`
- `Config.get(section, key, default=None)`
- `Config.get_bool(section, key, default=False)` — true/yes/1/on (case-insensitive)
- `Config.get_int(section, key, default=0)`

## Tests
```bash
cd source && PYTHONPATH=. python3 -m unittest discover -s tests -q
