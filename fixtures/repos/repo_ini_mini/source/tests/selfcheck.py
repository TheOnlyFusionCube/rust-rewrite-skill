"""Smoke tests for the Python ini_mini oracle (must pass before shipping fixture)."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from ini_mini import Config, IniError, parse


def check(name: str, cond: bool) -> None:
    if not cond:
        raise AssertionError(f"FAIL: {name}")
    print(f"  ok {name}")


def main() -> None:
    print("ini_mini Python selfcheck")

    # m01 basic
    cfg = parse("[db]\nhost = localhost\nport = 5432\n")
    check("has db", cfg.has_section("db"))
    check("host", cfg.get_str("db", "host") == "localhost")
    check("sections", cfg.sections() == ["db"])

    # m02 comments / whitespace
    cfg = parse("  # c\n; c2\n\n  [a]  \n  k  =  v  \n")
    check("trim", cfg.get_str("a", "k") == "v")

    # m03 duplicates / DEFAULT / merge
    cfg = parse("x=1\n[s]\na=1\na=2\n[s]\nb=3\n")
    check("DEFAULT", cfg.get_str("DEFAULT", "x") == "1")
    check("last-wins", cfg.get_str("s", "a") == "2")
    check("merge section", cfg.get_str("s", "b") == "3")
    check("key order", cfg.keys("s") == ["a", "b"])

    # m04 typed
    cfg = parse("[t]\ni=42\nb=yes\nf=OFF\nbad=maybe\n")
    check("int", cfg.get_int("t", "i") == 42)
    check("bool yes", cfg.get_bool("t", "b") is True)
    check("bool off", cfg.get_bool("t", "f") is False)
    try:
        cfg.get_bool("t", "bad")
        check("bad bool raises", False)
    except IniError as e:
        check("bad bool msg", "not a valid boolean" in e.message)

    # m05 errors
    try:
        parse("[open\n")
        check("unclosed raises", False)
    except IniError as e:
        check("unclosed msg", e.message == "unclosed section on line 1")
    try:
        parse("[a] junk\n")
        check("garbage raises", False)
    except IniError as e:
        check("garbage msg", e.message == "section header garbage on line 1")
    cfg = parse("[a]\nk=v\n")
    try:
        cfg.get_str("nope", "k")
        check("miss sec raises", False)
    except IniError as e:
        check("miss sec msg", e.message == "section 'nope' not found")
    try:
        cfg.get_str("a", "nope")
        check("miss key raises", False)
    except IniError as e:
        check("miss key msg", e.message == "key 'nope' not found in section 'a'")
    try:
        parse("[a]\n= v\n")
        check("empty key raises", False)
    except IniError as e:
        check("empty key msg", e.message == "empty key on line 2")

    # m06 round-trip
    cfg = parse("[db]\nhost = localhost\nport = 5432\n\n[app]\nname = demo\n")
    text = cfg.to_string()
    cfg2 = parse(text)
    check("roundtrip host", cfg2.get_str("db", "host") == "localhost")
    check("roundtrip sections", cfg2.sections() == ["db", "app"])

    # m07 unicode / escapes
    cfg = parse("[u]\nname = café\npath = a\\nb\\tc\\\\d\\#x\\;y\\q\n")
    check("unicode", cfg.get_str("u", "name") == "café")
    check("escapes", cfg.get_str("u", "path") == "a\nb\tc\\d#x;yq")
    rt = parse(cfg.to_string())
    check("escape roundtrip", rt.get_str("u", "path") == "a\nb\tc\\d#x;yq")

    # m08 adversarial
    cfg = parse("[]\nx=1\n")
    check("empty section name", cfg.has_section("") and cfg.get_str("", "x") == "1")
    cfg = parse("[a]\nempty =\n")
    check("empty value", cfg.get_str("a", "empty") == "")
    big = "k" * 200 + " = " + ("v" * 500) + "\n"
    cfg = parse("[h]\n" + big)
    check("huge line", len(cfg.get_str("h", "k" * 200)) == 500)
    check("or default", cfg.get_int_or("h", "missing", 7) == 7)

    print("ALL OK")


if __name__ == "__main__":
    main()
