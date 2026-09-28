#!/usr/bin/env python3
"""OpenCode stress harness (v8 ini-reliable default; v7 fastapi archived).

Default suite.json is opencode-repo-rewrite-v8-ini-reliable (ini_mini RIIR
with same reliability scaffolding as v7). v7 archived as suite.repo-v7-fastapi-reliable.json;
v6 blood / v5 chip8 / v4 RIIR / v3 / hard-v2 / soft retained.
Runs via `opencode run --variant max -m <model> <prompt>` under a PTY wrapper
(`script -q -c`), scores file/command/riir_cargo_milestones checks, writes
artifacts under artifacts/opencode-stress/.

RIIR mapping (Hassan Hayat 2026, https://rewritebench.com): ordered milestones with
60% unlock, % completion = passed/total across milestone tests, frozen test oracle.
v7 adds: inject PORTING/README/API into prompt, force-edit mandate, m01 preflight,
lib.rs change postcheck, staged milestone loop + auto-resume if still near-stub.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
DEFAULT_SUITE = ROOT / "suite.json"
FIXTURES_ROOT = ROOT / "fixtures"
ARTIFACTS_ROOT = ROOT / "artifacts" / "opencode-stress"
OPENCODE_BIN = Path(os.environ.get("OPENCODE_BIN", os.path.expanduser("~/.local/bin/opencode")))
DEFAULT_MODEL = "opencode/mimo-v2.6-flash-free"
DEFAULT_VARIANT = "max"
DEFAULT_TIMEOUT = 900


def die(msg: str, code: int = 1) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(code)


def require_api_key() -> str:
    key = os.environ.get("OPENCODE_API_KEY", "").strip()
    if not key:
        key_path = Path("/workspace/opencode-zen-api-key.txt")
        if key_path.is_file():
            key = key_path.read_text(encoding="utf-8").strip()
            if key:
                os.environ["OPENCODE_API_KEY"] = key
    if not key:
        die(
            "OPENCODE_API_KEY is not set. Export it or place auth in "
            "~/.local/share/opencode/auth.json / /workspace/opencode-zen-api-key.txt, then re-run. "
            "This harness will not invent a key."
        )
    if "OPENCODE_PERMISSION" not in os.environ:
        os.environ["OPENCODE_PERMISSION"] = json.dumps(
            {
                "*": "allow",
                "bash": "allow",
                "edit": "allow",
                "read": "allow",
                "write": "allow",
                "external_directory": "allow",
                "task": "allow",
                "skill": "allow",
            }
        )
    return key


def ensure_path() -> None:
    local = str(Path.home() / ".local" / "bin")
    path = os.environ.get("PATH", "")
    if local not in path.split(":"):
        os.environ["PATH"] = local + ":" + path


def load_suite(path: Path) -> dict[str, Any]:
    if not path.is_file():
        die(f"suite not found: {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        die(f"invalid suite JSON: {e}")
    if not isinstance(data.get("cases"), list) or not data["cases"]:
        die("suite.cases must be a non-empty list")
    return data


def normalize_text(text: str) -> str:
    """Strip trailing whitespace per line / ends; optional single surrounding markdown fence."""
    s = text.replace("\r\n", "\n").replace("\r", "\n")
    s = s.strip()
    fence = re.match(r"^```(?:\w+)?\s*\n?(.*?)\n?```\s*$", s, re.DOTALL)
    if fence:
        s = fence.group(1).strip()
    # strip trailing ws on each line, keep internal newlines
    lines = [ln.rstrip() for ln in s.split("\n")]
    # drop trailing empty lines
    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines)


def normalize_output(text: str) -> str:
    """Alias used by chat-text scorers (soft suite compat)."""
    return normalize_text(text)


def extract_json_candidate(text: str) -> Any | None:
    s = normalize_output(text)
    try:
        return json.loads(s)
    except json.JSONDecodeError:
        pass
    for pattern in (r"\{.*\}", r"\[.*\]"):
        m = re.search(pattern, s, re.DOTALL)
        if m:
            try:
                return json.loads(m.group(0))
            except json.JSONDecodeError:
                continue
    return None


def normalize_checks(check: Any) -> list[dict[str, Any]]:
    """Accept a single check object or a list; return a list."""
    if isinstance(check, list):
        return check
    if isinstance(check, dict):
        return [check]
    raise ValueError(f"check must be object or list, got {type(check).__name__}")


def score_text_check(check: dict[str, Any], raw_output: str) -> tuple[bool, str]:
    """Legacy chat-output checks (soft suite / residual)."""
    kind = check.get("type")
    cleaned = normalize_output(raw_output)

    if kind == "exact":
        expected = str(check["value"])
        lines = [ln.strip() for ln in cleaned.splitlines() if ln.strip()]
        if cleaned == expected:
            return True, "exact match"
        if expected in lines:
            return True, "exact line match"
        if set(lines) == {expected}:
            return True, "exact unique-line match"
        return False, f"expected exact {expected!r}, got {cleaned[:200]!r}"

    if kind == "regex":
        pattern = check["pattern"]
        flags = re.MULTILINE
        candidates = [cleaned]
        lines = [ln.strip() for ln in cleaned.splitlines() if ln.strip()]
        if lines:
            candidates.append(lines[0])
            candidates.append(lines[-1])
        for c in candidates:
            if re.search(pattern, c, flags):
                return True, f"regex matched on {c[:80]!r}"
        return False, f"regex {pattern!r} did not match {cleaned[:200]!r}"

    if kind == "json_equals":
        expected = check["value"]
        got = extract_json_candidate(raw_output)
        if got is None:
            return False, f"no JSON parseable from {cleaned[:200]!r}"
        if got == expected:
            return True, "json equals"
        return False, f"json mismatch: expected {expected!r}, got {got!r}"

    if kind == "contains":
        needle = str(check["value"])
        if needle in cleaned:
            return True, "contains"
        return False, f"missing substring {needle!r} in {cleaned[:200]!r}"

    return False, f"unknown text check type: {kind!r}"


def score_file_equals(case_workdir: Path, check: dict[str, Any]) -> tuple[bool, str]:
    rel = check["path"]
    path = case_workdir / rel
    if not path.is_file():
        return False, f"file_equals: missing {rel}"
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except OSError as e:
        return False, f"file_equals: read error {rel}: {e}"
    got = normalize_text(raw)
    expected = normalize_text(str(check["value"]))
    if got == expected:
        return True, f"file_equals {rel}"
    return False, f"file_equals {rel}: expected {expected!r}, got {got[:200]!r}"


def score_file_regex(case_workdir: Path, check: dict[str, Any]) -> tuple[bool, str]:
    rel = check["path"]
    path = case_workdir / rel
    if not path.is_file():
        return False, f"file_regex: missing {rel}"
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except OSError as e:
        return False, f"file_regex: read error {rel}: {e}"
    got = normalize_text(raw)
    pattern = check["pattern"]
    if re.search(pattern, got, re.MULTILINE):
        return True, f"file_regex {rel}"
    return False, f"file_regex {rel}: pattern {pattern!r} did not match {got[:200]!r}"


def score_file_json_equals(case_workdir: Path, check: dict[str, Any]) -> tuple[bool, str]:
    rel = check["path"]
    path = case_workdir / rel
    if not path.is_file():
        return False, f"file_json_equals: missing {rel}"
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except OSError as e:
        return False, f"file_json_equals: read error {rel}: {e}"
    # Strip optional fence then parse
    cleaned = normalize_text(raw)
    try:
        got = json.loads(cleaned)
    except json.JSONDecodeError:
        got = extract_json_candidate(raw)
        if got is None:
            return False, f"file_json_equals {rel}: not JSON: {cleaned[:200]!r}"
    expected = check["value"]
    if got == expected:
        return True, f"file_json_equals {rel}"
    return False, f"file_json_equals {rel}: expected {expected!r}, got {got!r}"


def score_command(case_workdir: Path, check: dict[str, Any]) -> tuple[bool, str]:
    cmd = check["cmd"]
    expect_exit = int(check.get("expect_exit", 0))
    timeout_sec = int(check.get("timeout_sec", 60))
    cwd_rel = check.get("cwd")
    cwd = case_workdir / cwd_rel if cwd_rel else case_workdir
    if not cwd.is_dir():
        return False, f"command: cwd missing {cwd}"

    try:
        proc = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout_sec,
            cwd=str(cwd),
            env=os.environ.copy(),
        )
    except subprocess.TimeoutExpired:
        return False, f"command timeout after {timeout_sec}s: {cmd!r}"
    except OSError as e:
        return False, f"command OS error: {e}"

    stdout = proc.stdout or ""
    stderr = proc.stderr or ""
    if proc.returncode != expect_exit:
        return (
            False,
            f"command exit {proc.returncode} != {expect_exit}: {cmd!r}; "
            f"stderr={stderr[:300]!r}",
        )

    if "stdout_exact" in check:
        expected = normalize_text(str(check["stdout_exact"]))
        got = normalize_text(stdout)
        if got != expected:
            return False, f"command stdout_exact: expected {expected!r}, got {got[:200]!r}"

    if "stdout_regex" in check:
        pattern = check["stdout_regex"]
        if not re.search(pattern, stdout, re.MULTILINE):
            return False, f"command stdout_regex {pattern!r} did not match {stdout[:200]!r}"

    return True, f"command ok: {cmd!r}"



def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def frozen_tests_fingerprint(rust_dir: Path, extra_globs: list[str] | None = None) -> dict[str, Any]:
    """SHA256 oracle over rust/tests/** and optional paths (e.g. src with cfg(test)).

    Prefer external tests/ — also fingerprint any src/**/*.rs that still contain
    #[cfg(test)] so moving tests back into lib would be detected if listed.
    """
    files: list[Path] = []
    tests_dir = rust_dir / "tests"
    if tests_dir.is_dir():
        files.extend(sorted(p for p in tests_dir.rglob("*") if p.is_file()))
    # Always scan src for #[cfg(test)] modules — include those files in the oracle
    src_dir = rust_dir / "src"
    if src_dir.is_dir():
        for p in sorted(src_dir.rglob("*.rs")):
            try:
                body = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            if "#[cfg(test)]" in body or "cfg(test)" in body:
                if p not in files:
                    files.append(p)
    if extra_globs:
        for g in extra_globs:
            files.extend(sorted(rust_dir.glob(g)))
    # dedupe preserve order
    seen: set[str] = set()
    uniq: list[Path] = []
    for p in files:
        key = str(p.resolve())
        if key not in seen:
            seen.add(key)
            uniq.append(p)
    per_file: dict[str, str] = {}
    h = hashlib.sha256()
    for p in uniq:
        rel = str(p.relative_to(rust_dir))
        digest = sha256_file(p)
        per_file[rel] = digest
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        h.update(digest.encode("utf-8"))
        h.update(b"\n")
    return {
        "aggregate_sha256": h.hexdigest(),
        "file_count": len(uniq),
        "files": per_file,
    }


def discover_milestone_tests(rust_dir: Path, filter_prefix: str) -> list[str]:
    """Count #[test] fn names under rust/tests that match the filter substring."""
    names: list[str] = []
    tests_dir = rust_dir / "tests"
    if not tests_dir.is_dir():
        return names
    fn_re = re.compile(r"#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z0-9_]+)", re.MULTILINE)
    for p in sorted(tests_dir.rglob("*.rs")):
        try:
            body = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for m in fn_re.finditer(body):
            name = m.group(1)
            if filter_prefix in name:
                names.append(name)
    return names


def parse_cargo_test_totals(output: str) -> tuple[int, int]:
    """Sum passed/failed across all 'test result:' lines in cargo test output."""
    passed = 0
    failed = 0
    for m in re.finditer(
        r"test result:\s*(?:ok|FAILED)\.\s*(\d+)\s+passed;\s*(\d+)\s+failed",
        output,
    ):
        passed += int(m.group(1))
        failed += int(m.group(2))
    return passed, failed


def normalize_milestones(raw: Any) -> list[dict[str, str]]:
    """Accept list of strings (filters) or list of {id, filter} objects."""
    out: list[dict[str, str]] = []
    if not isinstance(raw, list) or not raw:
        raise ValueError("riir_cargo_milestones.milestones must be a non-empty list")
    for i, item in enumerate(raw):
        if isinstance(item, str):
            filt = item
            mid = item.rstrip("_") if item.endswith("_") else item
            out.append({"id": mid, "filter": filt})
        elif isinstance(item, dict):
            filt = str(item.get("filter") or item.get("id") or "")
            mid = str(item.get("id") or filt.rstrip("_"))
            if not filt:
                raise ValueError(f"milestone[{i}] missing filter")
            out.append({"id": mid, "filter": filt})
        else:
            raise ValueError(f"milestone[{i}] must be str or object")
    return out


def score_riir_cargo_milestones(
    case_workdir: Path,
    check: dict[str, Any],
    pre_fingerprint: dict[str, Any] | None,
) -> tuple[bool, str, dict[str, Any]]:
    """Run ordered cargo test filters; compute % completion + 60% unlock peak.

    Returns (passed_binary, reason, detail) where passed_binary means
    completion >= threshold AND tests not mutated.
    """
    cwd_rel = check.get("cwd") or "rust"
    rust_dir = case_workdir / cwd_rel
    if not rust_dir.is_dir():
        return False, f"riir_cargo_milestones: cwd missing {cwd_rel}", {
            "completion": 0.0,
            "tests_mutated": False,
        }

    unlock_threshold = float(check.get("unlock_threshold", 0.60))
    pass_threshold = float(check.get("completion_pass_threshold", 0.75))
    timeout_sec = int(check.get("timeout_sec", 300))
    milestones = normalize_milestones(check.get("milestones"))

    post_fp = frozen_tests_fingerprint(rust_dir)
    tests_mutated = False
    if pre_fingerprint is not None:
        tests_mutated = (
            pre_fingerprint.get("aggregate_sha256") != post_fp.get("aggregate_sha256")
        )

    milestone_rows: list[dict[str, Any]] = []
    total_passed = 0
    total_tests = 0

    for mil in milestones:
        filt = mil["filter"]
        discovered = discover_milestone_tests(rust_dir, filt)
        expected = len(discovered)
        cmd = f"cargo test {shlex.quote(filt)} -- --nocapture"
        try:
            proc = subprocess.run(
                cmd,
                shell=True,
                capture_output=True,
                text=True,
                timeout=timeout_sec,
                cwd=str(rust_dir),
                env=os.environ.copy(),
            )
            blob = (proc.stdout or "") + "\n" + (proc.stderr or "")
            passed_n, failed_n = parse_cargo_test_totals(blob)
            ran = passed_n + failed_n
            # Prefer discovered count as denominator when cargo ran fewer
            # (e.g. compile error → 0 result lines).
            if expected > 0:
                denom = expected
                # If cargo reported more ran than expected, trust cargo
                if ran > expected:
                    denom = ran
                # Clamp passed to denom
                passed_n = min(passed_n, denom)
            else:
                denom = ran
            # Compile failure with expected tests → 0 passed
            if expected > 0 and ran == 0 and proc.returncode != 0:
                passed_n = 0
                denom = expected
                failed_n = expected
        except subprocess.TimeoutExpired:
            passed_n = 0
            denom = expected if expected > 0 else 0
            failed_n = denom
            blob = f"timeout after {timeout_sec}s"
            proc_rc = -1
        except OSError as e:
            return False, f"riir_cargo_milestones OS error: {e}", {
                "completion": 0.0,
                "tests_mutated": tests_mutated,
            }
        else:
            proc_rc = proc.returncode

        frac = (passed_n / denom) if denom else 0.0
        milestone_rows.append(
            {
                "id": mil["id"],
                "filter": filt,
                "passed": passed_n,
                "failed": failed_n if denom else 0,
                "total": denom,
                "fraction": round(frac, 4),
                "discovered_tests": discovered,
                "cargo_returncode": proc_rc,
            }
        )
        total_passed += passed_n
        total_tests += denom

    # 60% unlock rule: m1 always unlocked; unlock next when current >= threshold
    unlocked_count = 1 if milestones else 0
    for i, row in enumerate(milestone_rows):
        row["unlocked"] = i < unlocked_count or (i == 0)
        if i == 0:
            row["unlocked"] = True
        # After evaluating milestone i, maybe unlock i+1
        if row.get("unlocked") and row["total"] > 0 and row["fraction"] >= unlock_threshold:
            if i + 1 < len(milestone_rows):
                unlocked_count = max(unlocked_count, i + 2)  # 1-based count of unlocked
                milestone_rows[i + 1]["unlocked"] = True
        elif i == 0 and row["total"] == 0:
            # no tests — do not unlock further
            pass
        # Mark remaining
    # Fix unlocked flags consistently
    peak_index = 0  # 1-based peak unlocked
    can_see = True
    for i, row in enumerate(milestone_rows):
        if i == 0:
            row["unlocked"] = True
            peak_index = 1
            can_see = row["total"] > 0 and row["fraction"] >= unlock_threshold
        else:
            row["unlocked"] = can_see
            if row["unlocked"]:
                peak_index = i + 1
                can_see = row["total"] > 0 and row["fraction"] >= unlock_threshold
            else:
                can_see = False

    completion = (total_passed / total_tests) if total_tests else 0.0
    binary_ok = (completion >= pass_threshold) and (not tests_mutated)

    detail = {
        "completion": round(completion, 4),
        "passed_tests": total_passed,
        "total_tests": total_tests,
        "peak_milestone_index": peak_index,
        "milestones_unlocked": peak_index,
        "unlock_threshold": unlock_threshold,
        "completion_pass_threshold": pass_threshold,
        "milestones": milestone_rows,
        "tests_mutated": tests_mutated,
        "frozen_tests_before": pre_fingerprint,
        "frozen_tests_after": post_fp,
    }

    if tests_mutated:
        reason = (
            f"riir FAIL: tests mutated (frozen oracle); completion={completion:.4f} "
            f"({total_passed}/{total_tests}); peak_milestone={peak_index}"
        )
        return False, reason, detail

    if binary_ok:
        reason = (
            f"riir ok: completion={completion:.4f} ({total_passed}/{total_tests}); "
            f"peak_milestone={peak_index}/{len(milestones)}"
        )
        return True, reason, detail

    reason = (
        f"riir incomplete: completion={completion:.4f} ({total_passed}/{total_tests}) "
        f"< {pass_threshold}; peak_milestone={peak_index}/{len(milestones)}"
    )
    return False, reason, detail


def score_one_check(
    check: dict[str, Any],
    raw_output: str,
    case_workdir: Path,
) -> tuple[bool, str]:
    kind = check.get("type")
    if kind == "file_equals":
        return score_file_equals(case_workdir, check)
    if kind == "file_regex":
        return score_file_regex(case_workdir, check)
    if kind == "file_json_equals":
        return score_file_json_equals(case_workdir, check)
    if kind == "command":
        return score_command(case_workdir, check)
    if kind == "riir_cargo_milestones":
        return False, "riir_cargo_milestones must be scored via score_case (fingerprint)"
    if kind in ("exact", "regex", "json_equals", "contains"):
        return score_text_check(check, raw_output)
    return False, f"unknown check type: {kind!r}"


def score_case(
    checks: list[dict[str, Any]],
    raw_output: str,
    case_workdir: Path,
    pre_fingerprints: dict[int, dict[str, Any]] | None = None,
) -> tuple[bool, str, list[dict[str, Any]], dict[str, Any]]:
    """ALL checks must pass.

    Returns (passed, summary_reason, per_check details, riir_aggregate).
    riir_aggregate carries completion / milestone breakdown when present.
    """
    pre_fingerprints = pre_fingerprints or {}
    riir_agg: dict[str, Any] = {}
    if raw_output.startswith("__OPENCODE_ERROR__"):
        return False, f"opencode error: {raw_output[:300]}", [], riir_agg

    details: list[dict[str, Any]] = []
    completions: list[float] = []
    for i, check in enumerate(checks):
        kind = check.get("type")
        if kind == "riir_cargo_milestones":
            ok, reason, detail = score_riir_cargo_milestones(
                case_workdir, check, pre_fingerprints.get(i)
            )
            completions.append(float(detail.get("completion") or 0.0))
            riir_agg = detail
            details.append(
                {
                    "index": i,
                    "type": kind,
                    "passed": ok,
                    "reason": reason,
                    "riir": detail,
                }
            )
            if not ok:
                return False, reason, details, riir_agg
        else:
            ok, reason = score_one_check(check, raw_output, case_workdir)
            details.append({"index": i, "type": kind, "passed": ok, "reason": reason})
            if not ok:
                return False, reason, details, riir_agg
    if not details:
        return False, "no checks defined", details, riir_agg
    # If only non-riir checks, treat binary pass as completion 1.0 for suite mean
    if not completions and details and all(d["passed"] for d in details):
        riir_agg = {"completion": 1.0, "legacy_binary": True}
    elif not completions:
        riir_agg = {"completion": 0.0, "legacy_binary": True}
    elif len(completions) == 1:
        pass  # riir_agg already set
    else:
        # multiple riir checks — mean
        mean_c = sum(completions) / len(completions)
        riir_agg = {**riir_agg, "completion": round(mean_c, 4), "per_check_completions": completions}
    return True, "; ".join(d["reason"] for d in details), details, riir_agg


def extract_text_from_opencode_jsonl(blob: str) -> str:
    """Pull assistant text from --format json event stream (ignore log lines)."""
    texts: list[str] = []
    seen_ids: set[str] = set()
    saw_error: list[str] = []
    for line in blob.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("type") == "error":
            err = obj.get("error") or {}
            msg = ""
            if isinstance(err, dict):
                data = err.get("data") or {}
                msg = str(data.get("message") or err.get("name") or err)
            saw_error.append(msg or "opencode error event")
            continue
        if obj.get("type") == "text":
            part = obj.get("part") or {}
            pid = part.get("id") if isinstance(part, dict) else None
            if pid and pid in seen_ids:
                continue
            if pid:
                seen_ids.add(pid)
            t = part.get("text") if isinstance(part, dict) else None
            if t is None:
                t = obj.get("text")
            if t:
                texts.append(str(t))
    deduped: list[str] = []
    for t in texts:
        if not deduped or deduped[-1] != t:
            deduped.append(t)
    out = "\n".join(deduped).strip()
    if not out and saw_error:
        return f"__OPENCODE_ERROR__: {saw_error[-1]}"
    return out


def run_opencode(
    model: str,
    variant: str,
    prompt: str,
    timeout: int,
    work_dir: Path,
) -> dict[str, Any]:
    if not OPENCODE_BIN.is_file() and not OPENCODE_BIN.exists():
        bin_name = "opencode"
    else:
        bin_name = str(OPENCODE_BIN)

    # opencode run hangs after init without a TTY; wrap with `script`.
    # IMPORTANT: prompt must be shell-safe. json.dumps uses double quotes, which
    # still expand backticks and $() in sh -c — that mangled v3 prompts containing
    # `cargo test` and `<lines>`. Use shlex.quote (single-quoted) instead.
    inner = (
        f"{shlex.quote(bin_name)} run --auto --pure --format json "
        f"--variant {shlex.quote(variant)} -m {shlex.quote(model)} {shlex.quote(prompt)}"
    )
    pty_log = work_dir / f".pty-{uuid.uuid4().hex[:8]}.log"
    cmd = ["script", "-q", "-c", inner, str(pty_log)]
    started = time.monotonic()
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(work_dir),
            env=os.environ.copy(),
        )
        elapsed = time.monotonic() - started
        raw_parts: list[str] = []
        try:
            if pty_log.is_file():
                raw_parts.append(pty_log.read_text(encoding="utf-8", errors="replace"))
        except OSError:
            pass
        if not raw_parts:
            raw_parts.append(proc.stdout or "")
            raw_parts.append(proc.stderr or "")
        raw = "\n".join(raw_parts)
        text = extract_text_from_opencode_jsonl(raw)
        is_err = text.startswith("__OPENCODE_ERROR__:")
        if is_err:
            return {
                "ok": False,
                "returncode": proc.returncode,
                "stdout": text,
                "stderr": text,
                "raw": raw[-4000:],
                "elapsed_sec": round(elapsed, 3),
                "timed_out": False,
                "cmd": cmd,
            }
        return {
            "ok": proc.returncode == 0,
            "returncode": proc.returncode,
            "stdout": text,
            "stderr": proc.stderr or "",
            "raw": raw[-4000:],
            "elapsed_sec": round(elapsed, 3),
            "timed_out": False,
            "cmd": cmd,
        }
    except subprocess.TimeoutExpired as e:
        elapsed = time.monotonic() - started
        out = e.stdout if isinstance(e.stdout, str) else ""
        err = e.stderr if isinstance(e.stderr, str) else f"timeout after {timeout}s"
        try:
            if pty_log.is_file():
                out = pty_log.read_text(encoding="utf-8", errors="replace") + "\n" + out
        except OSError:
            pass
        return {
            "ok": False,
            "returncode": -1,
            "stdout": extract_text_from_opencode_jsonl(out),
            "stderr": err,
            "elapsed_sec": round(elapsed, 3),
            "timed_out": True,
            "cmd": cmd,
        }
    except FileNotFoundError:
        return {
            "ok": False,
            "returncode": 127,
            "stdout": "",
            "stderr": f"opencode/script binary not found: {bin_name}",
            "elapsed_sec": 0.0,
            "timed_out": False,
            "cmd": cmd,
        }


def resolve_fixture_dir(case: dict[str, Any]) -> Path | None:
    """Resolve fixture directory for a case.

    Order:
    1. case["fixture_subdir"] relative to fixtures/ (e.g. "repos/repo_wc")
    2. fixtures/repos/<id>/ if that directory exists (v3 convention)
    3. fixtures/<id>/ (v2 hard / legacy)
    """
    cid = case["id"]
    sub = case.get("fixture_subdir")
    if sub:
        p = FIXTURES_ROOT / str(sub)
        return p if p.is_dir() else None
    repos = FIXTURES_ROOT / "repos" / cid
    if repos.is_dir():
        return repos
    legacy = FIXTURES_ROOT / cid
    if legacy.is_dir():
        return legacy
    return None


def prepare_case_workdir(run_work_root: Path, case_id: str, case: dict[str, Any] | None = None) -> Path:
    """Copy fixture tree into run_work_root/<case_id>/ (isolated).

    For v3 repo cases, fixtures/repos/<id>/ usually contains source/ + rust/;
    the entire tree is copied so the model sees both.
    """
    dest = run_work_root / case_id
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True, exist_ok=True)

    case = case or {"id": case_id}
    src = resolve_fixture_dir(case)
    if src is not None and src.is_dir():
        def _ignore(directory: str, names: list[str]) -> set[str]:
            skip = {"target", "__pycache__", ".git", "Cargo.lock"}
            return {n for n in names if n in skip or n.endswith(".pyc")}

        for item in src.iterdir():
            if item.name in ("target", "__pycache__", ".git"):
                continue
            target = dest / item.name
            if item.is_dir():
                shutil.copytree(item, target, ignore=_ignore)
            else:
                shutil.copy2(item, target)

    # permissive opencode.json so headless asks never hang
    cfg_src = ROOT / "opencode.json"
    if cfg_src.is_file():
        (dest / "opencode.json").write_text(cfg_src.read_text(encoding="utf-8"), encoding="utf-8")
    return dest


def make_run_id() -> str:
    ts = datetime.now().astimezone().strftime("%Y%m%d-%H%M%S")
    return f"{ts}-{uuid.uuid4().hex[:8]}"


def write_json(path: Path, obj: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def append_jsonl(path: Path, obj: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(obj, ensure_ascii=False) + "\n")


def write_hypothesis(path: Path, model: str, variant: str, suite: dict[str, Any]) -> None:
    text = f"""# Hypothesis

## Goal
Achieve mean case `completion >= {suite.get('goal_pass_rate', 0.75)}` on the suite
`{suite.get('name', 'suite')}` (version {suite.get('version')}) for model
`{model}` at `--variant {variant}`.

## Suite
- Cases: {len(suite['cases'])}
- v5 blood / v4 RIIR-inspired: ordered milestones, 60% unlock, % completion, frozen tests/
- Soft/hard/v3/v4/v5/v6 archived (`suite.soft.json`, `suite.hard-v2.json`, `suite.repo-v3.json`, `suite.repo-v4-riir.json`, `suite.repo-v5-chip8.json`, `suite.repo-v6-fastapi-blood.json`)

## Method
1. Copy `fixtures/repos/<id>/` → work/<id>/ (`source/` + `rust/` with tests/ milestones).
2. Fingerprint `rust/tests/` (frozen oracle) before the model runs.
3. v7 reliability: inject PORTING/README/API; force-edit mandate; m01 preflight;
   staged milestone opencode turns + auto-resume if lib.rs unchanged; PTY + shlex.quote.
4. Score `riir_cargo_milestones`: `cargo test m0N_` per band; completion=passed/total;
   peak milestone under 60% unlock; FAIL if tests mutated; note lib.rs unchanged.
5. Timeout per case: {suite.get('timeout_sec', DEFAULT_TIMEOUT)}s.
6. Full suite + mean completion >= goal → update `best.json`; else `last-repo-run.json`.

## Risks
- Free-tier rate limits / empty responses / `__OPENCODE_ERROR__`
- Model weakens tests (caught by frozen oracle)
- Cargo compile time under timeout
"""
    path.write_text(text, encoding="utf-8")



# ───────────────────── v7 reliability scaffolding ─────────────────────

def reliability_cfg(suite: dict[str, Any], case: dict[str, Any] | None = None) -> dict[str, Any]:
    """Merge suite.reliability with optional case.reliability overrides."""
    cfg = dict(suite.get("reliability") or {})
    if case and isinstance(case.get("reliability"), dict):
        cfg.update(case["reliability"])
    return cfg


def lib_rs_path(workdir: Path, rust_rel: str = "rust") -> Path:
    return workdir / rust_rel / "src" / "lib.rs"


def content_sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def extract_pub_api_surface(lib_text: str, max_chars: int = 3500) -> str:
    """Keep pub type / method signatures so the model need not re-read all of lib.rs."""
    lines = lib_text.splitlines()
    keep: list[str] = []
    for i, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("pub ") or stripped.startswith("/// STUB"):
            keep.append(line.rstrip())
            # also keep the next signature line if this was a doc comment on stub
            continue
        if "STUB:" in line:
            keep.append(line.rstrip())
    # densify: unique consecutive
    out: list[str] = []
    prev = None
    for ln in keep:
        if ln != prev:
            out.append(ln)
            prev = ln
    blob = "\n".join(out)
    if len(blob) > max_chars:
        blob = blob[: max_chars - 20] + "\n… [truncated]"
    return blob


def read_text_capped(path: Path, max_chars: int) -> str:
    if not path.is_file():
        return f"(missing: {path.name})"
    text = path.read_text(encoding="utf-8", errors="replace")
    if len(text) > max_chars:
        return text[: max_chars - 20] + "\n… [truncated]"
    return text


def preflight_milestone(
    rust_dir: Path,
    filt: str,
    timeout_sec: int = 120,
) -> dict[str, Any]:
    """Run `cargo test <filter>` and return failing/passing test names (pre-model)."""
    if not rust_dir.is_dir():
        return {
            "filter": filt,
            "ok": False,
            "passed": [],
            "failed": [],
            "error": f"missing rust dir {rust_dir}",
        }
    cmd = f"cargo test {shlex.quote(filt)} -- --nocapture"
    try:
        proc = subprocess.run(
            cmd,
            shell=True,
            cwd=str(rust_dir),
            capture_output=True,
            text=True,
            timeout=timeout_sec,
        )
    except subprocess.TimeoutExpired:
        return {
            "filter": filt,
            "ok": False,
            "passed": [],
            "failed": [],
            "error": f"preflight timeout after {timeout_sec}s",
        }
    out = (proc.stdout or "") + "\n" + (proc.stderr or "")
    passed = re.findall(r"^test (\S+) \.\.\. ok$", out, re.M)
    failed = re.findall(r"^test (\S+) \.\.\. FAILED$", out, re.M)
    return {
        "filter": filt,
        "ok": proc.returncode == 0,
        "passed": passed,
        "failed": failed,
        "returncode": proc.returncode,
        "tail": out[-1500:],
    }


def quick_milestone_fraction(
    rust_dir: Path,
    filt: str,
    timeout_sec: int = 180,
) -> dict[str, Any]:
    info = preflight_milestone(rust_dir, filt, timeout_sec=timeout_sec)
    n_pass = len(info.get("passed") or [])
    n_fail = len(info.get("failed") or [])
    total = n_pass + n_fail
    frac = (n_pass / total) if total else 0.0
    return {**info, "passed_n": n_pass, "failed_n": n_fail, "total": total, "fraction": round(frac, 4)}


def build_reliable_prompt(
    base_prompt: str,
    workdir: Path,
    *,
    cfg: dict[str, Any],
    focus_filter: str,
    focus_id: str,
    failing_tests: list[str],
    passed_tests: list[str],
    stage_index: int,
    stage_total: int,
    is_resume: bool = False,
    resume_reason: str = "",
) -> str:
    """Compose prompt with injected docs + force-edit + single-milestone focus."""
    max_inject = int(cfg.get("max_prompt_inject_chars") or 12000)
    parts: list[str] = []

    if is_resume:
        parts.append(
            "## RESUME — PREVIOUS TURN DID NOT FINISH THE PORT\n"
            f"Reason: {resume_reason or 'lib.rs still matches the stub / target tests still failing'}.\n"
            "You already have the workdir. Do NOT re-read every file. "
            "IMMEDIATELY edit rust/src/lib.rs with write/edit tools, then run the cargo test filter below.\n"
        )

    if cfg.get("force_edit_mandate", True):
        parts.append(
            "## HARD MANDATE (failure if violated)\n"
            "1. You MUST modify rust/src/*.rs (especially rust/src/lib.rs) with write/edit tools "
            "BEFORE you finish. A turn that only reads/chats is a FAILED turn.\n"
            "2. Do NOT edit, weaken, delete, rename, #[ignore], or skip anything under rust/tests/.\n"
            "3. Do NOT invent a different public API — keep existing stub signatures; fill bodies "
            "(esp. App::handle and App::openapi).\n"
            "4. Prefer std-only. Leave source/ intact.\n"
        )

    if cfg.get("forbid_bulk_test_reads", True):
        parts.append(
            "## READ BUDGET\n"
            f"- Read at most: PORTING.md (injected below), source/README.md (injected), "
            f"rust/src/lib.rs, and rust/tests/ matching `{focus_filter}` ONLY.\n"
            "- Do NOT open m02_…m10_ test files until the current milestone filter is ≥60% passing.\n"
            "- Do NOT paste huge files back into chat; edit disk files.\n"
        )

    parts.append(
        f"## STAGE {stage_index}/{stage_total} — FOCUS `{focus_id}` (filter `{focus_filter}`)\n"
        f"First target: make `cargo test {focus_filter}` pass from rust/.\n"
        f"Currently failing: {', '.join(failing_tests) if failing_tests else '(none listed — still run the filter)'}\n"
        f"Already passing: {', '.join(passed_tests) if passed_tests else '(none)'}\n"
        "After edits: run that cargo filter, fix assertions against source/ oracle, then stop or "
        "advance only if ≥60% of this filter passes.\n"
    )

    budget_left = max_inject
    if cfg.get("inject_porting_docs", True):
        porting = read_text_capped(workdir / "PORTING.md", min(3500, budget_left // 3))
        readme = read_text_capped(workdir / "source" / "README.md", min(3500, budget_left // 3))
        lib_p = lib_rs_path(workdir)
        api = ""
        if lib_p.is_file():
            api = extract_pub_api_surface(
                lib_p.read_text(encoding="utf-8", errors="replace"),
                max_chars=min(3500, budget_left // 3),
            )
        block = (
            "## INJECTED PORTING.md (already provided — do not re-read unless needed)\n"
            f"```\n{porting}\n```\n\n"
            "## INJECTED source/README.md quirks\n"
            f"```\n{readme}\n```\n\n"
            "## PUBLIC API SURFACE (stub signatures — implement bodies, esp. App::handle)\n"
            f"```\n{api}\n```\n"
        )
        parts.append(block)

    parts.append("## BASE TASK\n" + base_prompt.strip() + "\n")
    parts.append(
        "## DONE FOR THIS STAGE means\n"
        f"- rust/src/lib.rs byte content differs from the initial stub, AND\n"
        f"- `cargo test {focus_filter}` shows real progress on the failing list above.\n"
    )
    return "\n".join(parts).strip() + "\n"


def first_riir_check(case: dict[str, Any]) -> dict[str, Any] | None:
    for ch in normalize_checks(case.get("check") or []):
        if ch.get("type") == "riir_cargo_milestones":
            return ch
    return None


def run_suite(args: argparse.Namespace) -> int:
    ensure_path()
    suite = load_suite(Path(args.suite))
    model = args.model or suite.get("default_model") or DEFAULT_MODEL
    variant = args.variant or suite.get("variant") or DEFAULT_VARIANT
    timeout_default = args.timeout or suite.get("timeout_sec") or DEFAULT_TIMEOUT
    goal = float(suite.get("goal_pass_rate", 0.75))

    if args.dry_run:
        require_api_key()
        print(
            json.dumps(
                {
                    "dry_run": True,
                    "model": model,
                    "variant": variant,
                    "timeout_sec": timeout_default,
                    "suite_name": suite.get("name"),
                    "suite_version": suite.get("version"),
                    "cases": len(suite["cases"]),
                    "case_ids": [c["id"] for c in suite["cases"]],
                    "fixtures_root": str(FIXTURES_ROOT),
                    "artifacts_root": str(ARTIFACTS_ROOT),
                },
                indent=2,
            )
        )
        return 0

    require_api_key()

    if not OPENCODE_BIN.exists():
        from shutil import which

        if not which("opencode"):
            die(f"opencode not found at {OPENCODE_BIN} and not on PATH")

    run_id = args.run_id or make_run_id()
    run_dir = ARTIFACTS_ROOT / run_id
    run_dir.mkdir(parents=True, exist_ok=True)
    results_path = run_dir / "results.jsonl"
    summary_path = run_dir / "summary.json"
    hypothesis_path = run_dir / "hypothesis.md"
    work_root = run_dir / "work"

    if results_path.exists():
        results_path.unlink()

    write_hypothesis(hypothesis_path, model, variant, suite)

    cases = suite["cases"]
    if args.limit is not None:
        cases = cases[: max(0, args.limit)]
    if args.only:
        only = set(args.only)
        cases = [c for c in cases if c["id"] in only]
        missing = only - {c["id"] for c in cases}
        if missing:
            die(f"unknown case ids: {sorted(missing)}")

    passed = 0
    failed = 0
    errors = 0
    results: list[dict[str, Any]] = []

    print(
        f"run_id={run_id} model={model} variant={variant} "
        f"cases={len(cases)} timeout_default={timeout_default}s "
        f"suite={suite.get('name')} v{suite.get('version')}"
    )
    print(f"artifacts={run_dir}")

    for i, case in enumerate(cases, 1):
        cid = case["id"]
        base_prompt = case["prompt"]
        checks = normalize_checks(case["check"])
        case_timeout = int(case.get("timeout_sec") or timeout_default)
        rel_cfg = reliability_cfg(suite, case)
        print(f"[{i}/{len(cases)}] {cid} (timeout={case_timeout}s) ...", flush=True)

        case_workdir = prepare_case_workdir(work_root, cid, case)

        # Frozen-test oracle: fingerprint before the model can edit tests/
        pre_fingerprints: dict[int, dict[str, Any]] = {}
        rust_rel = "rust"
        for ci, ch in enumerate(checks):
            if ch.get("type") == "riir_cargo_milestones":
                rust_rel = ch.get("cwd") or "rust"
                rust_path = case_workdir / rust_rel
                pre_fingerprints[ci] = frozen_tests_fingerprint(rust_path)

        rust_path = case_workdir / rust_rel
        stub_lib = lib_rs_path(case_workdir, rust_rel)
        stub_lib_hash = sha256_file(stub_lib) if stub_lib.is_file() else None
        stub_lib_bytes = stub_lib.stat().st_size if stub_lib.is_file() else 0

        riir_check = first_riir_check(case)
        milestones = list((riir_check or {}).get("milestones") or [])
        unlock_thr = float(
            rel_cfg.get("stage_unlock_threshold")
            or (riir_check or {}).get("unlock_threshold")
            or 0.6
        )

        stage_log: list[dict[str, Any]] = []
        wall_budget = int(rel_cfg.get("wall_budget_sec") or case_timeout)
        stage_timeout = int(rel_cfg.get("stage_timeout_sec") or min(720, case_timeout))
        resume_timeout = int(rel_cfg.get("resume_timeout_sec") or min(480, case_timeout))
        polish_timeout = int(
            rel_cfg.get("polish_timeout_sec") or resume_timeout or min(900, case_timeout)
        )
        max_polish_turns = int(rel_cfg.get("max_polish_turns") or 1)
        max_stages = int(rel_cfg.get("max_stages") or 1)
        use_staged = bool(rel_cfg.get("staged_milestones")) and bool(milestones)
        use_inject = bool(rel_cfg) and (
            rel_cfg.get("inject_porting_docs")
            or rel_cfg.get("force_edit_mandate")
            or rel_cfg.get("preflight_first_milestone")
        )

        case_t0 = time.monotonic()
        raw_chunks: list[str] = []
        exec_result: dict[str, Any] = {
            "ok": True,
            "returncode": 0,
            "elapsed_sec": 0.0,
            "timed_out": False,
            "stderr": "",
        }
        last_prompt = base_prompt
        any_timed_out = False
        opencode_error: str | None = None

        def remaining_budget() -> int:
            used = time.monotonic() - case_t0
            return max(30, int(wall_budget - used))

        def run_one(prompt: str, timeout: int, tag: str) -> dict[str, Any]:
            nonlocal any_timed_out, opencode_error
            to = max(30, min(timeout, remaining_budget()))
            print(f"  → opencode[{tag}] timeout={to}s prompt_chars={len(prompt)}", flush=True)
            er = run_opencode(model, variant, prompt, to, case_workdir)
            raw_chunks.append(er.get("stdout") or "")
            if er.get("timed_out"):
                any_timed_out = True
            out = er.get("stdout") or ""
            if out.startswith("__OPENCODE_ERROR__") or (
                not er.get("ok") and str(er.get("stderr") or "").startswith("__OPENCODE_ERROR__")
            ):
                opencode_error = out if out.startswith("__OPENCODE_ERROR__") else er.get("stderr")
            return er

        if use_inject or use_staged:
            # All milestones available; skip-green + max_stages caps *model* turns
            if use_staged:
                stage_defs = list(milestones) if milestones else [{"id": "all", "filter": ""}]
            else:
                stage_defs = milestones[:1] if milestones else [{"id": "all", "filter": ""}]

            model_turns = 0
            for si, ms in enumerate(stage_defs, 1):
                if remaining_budget() < 60:
                    stage_log.append({"stage": si, "skipped": "wall_budget"})
                    break
                if model_turns >= max(1, max_stages):
                    stage_log.append(
                        {
                            "stage": si,
                            "id": str(ms.get("id")),
                            "skipped": "max_stages_reached",
                        }
                    )
                    break
                focus_id = str(ms.get("id") or f"stage{si}")
                focus_filt = str(ms.get("filter") or "")
                pre = {"passed": [], "failed": [], "fraction": 0.0}
                if focus_filt and rel_cfg.get("preflight_first_milestone", True):
                    pre = quick_milestone_fraction(rust_path, focus_filt, timeout_sec=120)
                    print(
                        f"  preflight {focus_filt}: {pre.get('passed_n', 0)}/{pre.get('total', 0)} "
                        f"failing={pre.get('failed')}",
                        flush=True,
                    )
                    # Skip burning a model turn when this band is already unlocked
                    if float(pre.get("fraction") or 0.0) >= unlock_thr and not (pre.get("failed")):
                        stage_log.append(
                            {
                                "stage": si,
                                "id": focus_id,
                                "filter": focus_filt,
                                "lib_rs_changed": True,  # prior stage edited
                                "fraction": pre.get("fraction"),
                                "passed": pre.get("passed"),
                                "failed": pre.get("failed"),
                                "elapsed_sec": 0.0,
                                "timed_out": False,
                                "resumed": False,
                                "skipped": "already_green",
                            }
                        )
                        print(
                            f"  skip stage {si} {focus_id}: already {pre.get('fraction')} ≥ unlock",
                            flush=True,
                        )
                        continue
                prompt = build_reliable_prompt(
                    base_prompt,
                    case_workdir,
                    cfg=rel_cfg,
                    focus_filter=focus_filt or "m01_",
                    focus_id=focus_id,
                    failing_tests=list(pre.get("failed") or []),
                    passed_tests=list(pre.get("passed") or []),
                    stage_index=si,
                    stage_total=len(stage_defs),
                )
                last_prompt = prompt
                er = run_one(prompt, stage_timeout, f"stage{si}-{focus_id}")
                exec_result = er
                model_turns += 1

                # Post-stage lib.rs check + milestone fraction
                cur_hash = sha256_file(stub_lib) if stub_lib.is_file() else None
                lib_changed = bool(stub_lib_hash and cur_hash and cur_hash != stub_lib_hash)
                post = (
                    quick_milestone_fraction(rust_path, focus_filt, timeout_sec=180)
                    if focus_filt
                    else {"fraction": 0.0, "failed": [], "passed": []}
                )
                stage_entry = {
                    "stage": si,
                    "id": focus_id,
                    "filter": focus_filt,
                    "lib_rs_changed": lib_changed,
                    "fraction": post.get("fraction"),
                    "passed": post.get("passed"),
                    "failed": post.get("failed"),
                    "elapsed_sec": er.get("elapsed_sec"),
                    "timed_out": er.get("timed_out"),
                    "resumed": False,
                }

                need_resume = False
                resume_reason = ""
                if rel_cfg.get("auto_resume_if_stub", True) and remaining_budget() >= 90:
                    if not lib_changed:
                        need_resume = True
                        resume_reason = (
                            f"lib.rs sha256 still equals stub ({stub_lib_hash}); "
                            "no rust/src edit detected"
                        )
                    elif focus_filt and float(post.get("fraction") or 0.0) < unlock_thr:
                        # near-stub progress: still below unlock after an edit
                        if float(post.get("fraction") or 0.0) <= float(pre.get("fraction") or 0.0) + 0.01:
                            need_resume = True
                            resume_reason = (
                                f"tests still failing after edit: {post.get('failed')}; "
                                f"fraction={post.get('fraction')} (was {pre.get('fraction')})"
                            )

                if need_resume:
                    rprompt = build_reliable_prompt(
                        base_prompt,
                        case_workdir,
                        cfg=rel_cfg,
                        focus_filter=focus_filt or "m01_",
                        focus_id=focus_id,
                        failing_tests=list(post.get("failed") or pre.get("failed") or []),
                        passed_tests=list(post.get("passed") or []),
                        stage_index=si,
                        stage_total=len(stage_defs),
                        is_resume=True,
                        resume_reason=resume_reason,
                    )
                    last_prompt = rprompt
                    er2 = run_one(rprompt, resume_timeout, f"resume{si}-{focus_id}")
                    exec_result = er2
                    cur_hash = sha256_file(stub_lib) if stub_lib.is_file() else None
                    lib_changed = bool(stub_lib_hash and cur_hash and cur_hash != stub_lib_hash)
                    post = (
                        quick_milestone_fraction(rust_path, focus_filt, timeout_sec=180)
                        if focus_filt
                        else post
                    )
                    stage_entry.update(
                        {
                            "resumed": True,
                            "resume_reason": resume_reason,
                            "lib_rs_changed": lib_changed,
                            "fraction": post.get("fraction"),
                            "passed": post.get("passed"),
                            "failed": post.get("failed"),
                            "resume_elapsed_sec": er2.get("elapsed_sec"),
                        }
                    )

                stage_log.append(stage_entry)
                print(
                    f"  stage {si} {focus_id}: lib_changed={lib_changed} "
                    f"frac={stage_entry.get('fraction')} resumed={stage_entry.get('resumed')}",
                    flush=True,
                )

                # Gate: do not leapfrog if below unlock
                if focus_filt and float(stage_entry.get("fraction") or 0.0) < unlock_thr:
                    print(
                        f"  stop advance: {focus_id} fraction "
                        f"{stage_entry.get('fraction')} < unlock {unlock_thr}",
                        flush=True,
                    )
                    break
        else:
            # Legacy single-shot path (no reliability block)
            last_prompt = base_prompt
            exec_result = run_one(base_prompt, case_timeout, "single")

        # Optional polish: up to max_polish_turns targeting remaining failing tests
        if (
            use_inject
            and rel_cfg.get("auto_resume_if_stub", True)
            and milestones
            and remaining_budget() >= 120
            and stub_lib.is_file()
        ):
            for polish_i in range(max(1, max_polish_turns)):
                if remaining_budget() < 120:
                    break
                remain_failed: list[str] = []
                remain_filters: list[str] = []
                for ms in milestones:
                    filt = str(ms.get("filter") or "")
                    if not filt:
                        continue
                    info = quick_milestone_fraction(rust_path, filt, timeout_sec=120)
                    if info.get("failed"):
                        remain_failed.extend(list(info["failed"]))
                        remain_filters.append(filt)
                if not remain_failed:
                    break
                focus_filt = remain_filters[0]
                focus_id = f"polish_remaining_{polish_i + 1}"
                print(
                    f"  polish[{polish_i + 1}/{max_polish_turns}]: "
                    f"{len(remain_failed)} failing across {remain_filters}: "
                    f"{remain_failed[:12]}",
                    flush=True,
                )
                # Spotlight content-type 422 if that failure is present
                ct_hint = ""
                if any("bad_content_type" in n or "content_type" in n for n in remain_failed):
                    ct_hint = (
                        "\nHARD CT RULE: required JSON body + content-type that does not "
                        "contain application/json (e.g. text/plain) MUST return 422 with "
                        "detail type_error.json (loc includes body) — never 200 even if "
                        "the body parses as JSON.\n"
                    )
                pprompt = build_reliable_prompt(
                    base_prompt,
                    case_workdir,
                    cfg=rel_cfg,
                    focus_filter=focus_filt,
                    focus_id=focus_id,
                    failing_tests=remain_failed,
                    passed_tests=[],
                    stage_index=len(stage_log) + 1,
                    stage_total=len(stage_log) + 1,
                    is_resume=True,
                    resume_reason=(
                        "Polish remaining failures after staged loop: "
                        + ", ".join(remain_failed[:20])
                    ),
                )
                pprompt += (
                    "\n## POLISH SCOPE\n"
                    + "Remaining failing tests (all bands): "
                    + ", ".join(remain_failed)
                    + "\nEdit rust/src/lib.rs to fix these without breaking earlier green bands. "
                    + "Re-run the relevant cargo test filters after edits.\n"
                    + ct_hint
                )
                last_prompt = pprompt
                er_p = run_one(
                    pprompt,
                    min(polish_timeout, remaining_budget()),
                    f"polish{polish_i + 1}",
                )
                exec_result = er_p
                stage_log.append(
                    {
                        "stage": "polish",
                        "id": focus_id,
                        "filter": ",".join(remain_filters),
                        "failed_before": remain_failed,
                        "elapsed_sec": er_p.get("elapsed_sec"),
                        "timed_out": er_p.get("timed_out"),
                        "resumed": True,
                        "polish_turn": polish_i + 1,
                    }
                )
                if er_p.get("timed_out"):
                    # still allow a subsequent polish turn if budget remains
                    continue

        total_elapsed = round(time.monotonic() - case_t0, 3)
        exec_result = dict(exec_result)
        exec_result["elapsed_sec"] = total_elapsed
        exec_result["timed_out"] = any_timed_out or bool(exec_result.get("timed_out"))

        raw = chr(10).join(raw_chunks)
        riir_detail: dict[str, Any] = {}

        # lib.rs postcheck metadata (never softens tests)
        final_hash = sha256_file(stub_lib) if stub_lib.is_file() else None
        lib_rs_changed = bool(stub_lib_hash and final_hash and final_hash != stub_lib_hash)
        lib_note = ""
        if rel_cfg.get("postcheck_lib_changed", True) and stub_lib_hash:
            if not lib_rs_changed:
                lib_note = (
                    f"lib.rs UNCHANGED vs stub sha256={stub_lib_hash} "
                    f"({stub_lib_bytes} bytes) — model produced no edit"
                )
                print(f"  WARN {lib_note}", flush=True)

        if opencode_error:
            passed_check, reason, check_details, riir_detail = (
                False,
                f"opencode error: {opencode_error[:300]}",
                [],
                {"completion": 0.0},
            )
        else:
            passed_check, reason, check_details, riir_detail = score_case(
                checks, raw, case_workdir, pre_fingerprints
            )
            if exec_result["timed_out"]:
                passed_check = False
                reason = f"timeout after {case_timeout}s; {reason}"
            elif (not passed_check) and (not exec_result["ok"]) and exec_result["returncode"] not in (0,):
                if "missing" in reason or "expected" in reason or "exit" in reason or "riir" in reason:
                    pass
                else:
                    reason = f"opencode exit {exec_result['returncode']}; {reason}"
            if lib_note:
                reason = f"{lib_note}; {reason}"

        # Attach reliability diagnostics into riir detail
        if isinstance(riir_detail, dict):
            riir_detail = {
                **riir_detail,
                "lib_rs_changed": lib_rs_changed,
                "lib_rs_stub_sha256": stub_lib_hash,
                "lib_rs_final_sha256": final_hash,
                "reliability_stages": stage_log,
            }

        completion = float(riir_detail.get("completion") or 0.0)
        if passed_check:
            passed += 1
            status = "PASS"
        elif exec_result["timed_out"] or (
            not passed_check and (opencode_error or raw.startswith("__OPENCODE_ERROR__"))
        ):
            errors += 1
            failed += 1
            status = "ERROR"
        else:
            failed += 1
            status = "FAIL"

        row = {
            "id": cid,
            "category": case.get("category"),
            "status": status,
            "passed": passed_check,
            "completion": round(completion, 4),
            "peak_milestone_index": riir_detail.get("peak_milestone_index"),
            "milestones": riir_detail.get("milestones"),
            "tests_mutated": riir_detail.get("tests_mutated", False),
            "passed_tests": riir_detail.get("passed_tests"),
            "total_tests": riir_detail.get("total_tests"),
            "lib_rs_changed": lib_rs_changed,
            "reliability_stages": stage_log,
            "reason": reason,
            "check_details": check_details,
            "riir": riir_detail if riir_detail else None,
            "elapsed_sec": exec_result["elapsed_sec"],
            "timed_out": exec_result["timed_out"],
            "returncode": exec_result["returncode"],
            "timeout_sec": case_timeout,
            "workdir": str(case_workdir.relative_to(ROOT)) if case_workdir.is_relative_to(ROOT) else str(case_workdir),
            "prompt": last_prompt[:8000],
            "check": checks,
            "stdout": raw[:4000],
            "stderr": (exec_result.get("stderr") or "")[:2000],
            "model": model,
            "variant": variant,
            "ts": datetime.now().astimezone().isoformat(),
        }
        append_jsonl(results_path, row)
        results.append(row)
        print(
            f"  {status} completion={completion:.4f} lib_changed={lib_rs_changed} "
            f"({exec_result['elapsed_sec']}s) {reason[:160]}",
            flush=True,
        )

    total = len(cases)
    binary_pass_rate = (passed / total) if total else 0.0
    completions = [float(r.get("completion") or 0.0) for r in results]
    mean_completion = (sum(completions) / len(completions)) if completions else 0.0
    # v4 primary metric: mean completion; legacy suites without riir still set
    # completion 1.0/0.0 from binary checks via score_case.
    suite_version = int(suite.get("version") or 0)
    if suite_version >= 4:
        primary_score = mean_completion
        goal_met = mean_completion >= goal
    else:
        primary_score = binary_pass_rate
        goal_met = binary_pass_rate >= goal

    summary = {
        "run_id": run_id,
        "model": model,
        "variant": variant,
        "suite": suite.get("name"),
        "suite_version": suite.get("version"),
        "total": total,
        "passed": passed,
        "failed": failed,
        "errors": errors,
        "pass_rate": round(primary_score, 4),
        "binary_pass_rate": round(binary_pass_rate, 4),
        "mean_completion": round(mean_completion, 4),
        "goal_pass_rate": goal,
        "goal_met": goal_met,
        "timeout_sec": timeout_default,
        "started_approx": results[0]["ts"] if results else None,
        "finished": datetime.now().astimezone().isoformat(),
        "methodology": "riir-mini" if suite_version >= 4 else "legacy",
        "artifacts": {
            "hypothesis": str(hypothesis_path.relative_to(ROOT)),
            "results": str(results_path.relative_to(ROOT)),
            "summary": str(summary_path.relative_to(ROOT)),
            "work": str(work_root.relative_to(ROOT)),
        },
        "case_results": [
            {
                "id": r["id"],
                "status": r["status"],
                "passed": r["passed"],
                "completion": r.get("completion"),
                "peak_milestone_index": r.get("peak_milestone_index"),
                "tests_mutated": r.get("tests_mutated"),
                "passed_tests": r.get("passed_tests"),
                "total_tests": r.get("total_tests"),
                "lib_rs_changed": r.get("lib_rs_changed"),
                "elapsed_sec": r["elapsed_sec"],
                "reason": r["reason"][:200],
            }
            for r in results
        ],
    }
    write_json(summary_path, summary)

    best_path = ARTIFACTS_ROOT / "best.json"
    # v3+ repo-rewrite uses last-repo-run.json; v2 hard keeps last-hard-run.json
    last_run_path = ARTIFACTS_ROOT / (
        "last-repo-run.json" if suite_version >= 3 else "last-hard-run.json"
    )
    snapshot = {
        "updated_at": datetime.now().astimezone().isoformat(),
        "pass_rate": round(primary_score, 4),
        "mean_completion": round(mean_completion, 4),
        "binary_pass_rate": round(binary_pass_rate, 4),
        "goal_pass_rate": goal,
        "goal_met": goal_met,
        "model": model,
        "variant": variant,
        "run_id": run_id,
        "summary_path": str(summary_path.relative_to(ROOT)),
        "passed": passed,
        "total": total,
        "failed_ids": [r["id"] for r in results if not r["passed"]],
        "suite": suite.get("name"),
        "suite_version": suite.get("version"),
    }

    # Only promote best.json for a full-suite run (no --only/--limit truncation).
    full_suite = (args.only is None and args.limit is None and total == len(suite["cases"]))
    if goal_met and full_suite:
        should_write = True
        if best_path.is_file():
            try:
                prev = json.loads(best_path.read_text(encoding="utf-8"))
                prev_ver = int(prev.get("suite_version") or 0)
                prev_score = float(prev.get("pass_rate", 0))
                if prev_ver > suite_version:
                    should_write = False
                    print(
                        f"existing best is newer suite_version {prev_ver} > {suite_version}; "
                        f"not updating best.json"
                    )
                elif prev_ver == suite_version and prev_score > primary_score:
                    should_write = False
                    print(
                        f"score {primary_score:.4f} >= goal but lower than existing best "
                        f"{prev_score}; not updating best.json"
                    )
            except (json.JSONDecodeError, TypeError, ValueError):
                should_write = True
        if should_write:
            write_json(best_path, snapshot)
            print(f"Updated {best_path} (pass_rate/mean_completion={primary_score:.4f})")
        write_json(last_run_path, snapshot)
    elif goal_met and not full_suite:
        write_json(last_run_path, snapshot)
        print(
            f"Goal met on partial run ({passed}/{total}, mean_completion={mean_completion:.4f}); "
            f"best.json NOT updated (need full suite); wrote {last_run_path}"
        )
    else:
        write_json(last_run_path, snapshot)
        print(
            f"Goal not met: mean_completion/pass_rate={primary_score:.4f} < {goal}; "
            f"wrote {last_run_path}"
        )
        print("best.json NOT updated")

    print(
        json.dumps(
            {
                "pass_rate": primary_score,
                "mean_completion": mean_completion,
                "binary_pass_rate": binary_pass_rate,
                "passed": passed,
                "total": total,
                "run_id": run_id,
                "goal_met": goal_met,
            },
            indent=2,
        )
    )
    return 0 if goal_met else 2


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="run_suite.py",
        description="OpenCode stress harness (v4 RIIR-mini / v3 archive / v2 hard / soft)",
    )
    p.add_argument("--suite", default=str(DEFAULT_SUITE), help="Path to suite.json")
    p.add_argument("--model", default=None, help=f"Model id (default: suite or {DEFAULT_MODEL})")
    p.add_argument("--variant", default=None, help=f"Variant flag (default: suite or {DEFAULT_VARIANT})")
    p.add_argument(
        "--timeout",
        type=int,
        default=None,
        help=f"Default per-case timeout seconds (default: suite or {DEFAULT_TIMEOUT})",
    )
    p.add_argument("--run-id", default=None, help="Artifact run id (default: timestamp-uuid)")
    p.add_argument("--limit", type=int, default=None, help="Run only the first N cases")
    p.add_argument("--only", nargs="+", default=None, help="Run only these case ids")
    p.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate suite + auth gate; print plan; do not call API",
    )
    return p


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    try:
        code = run_suite(args)
    except SystemExit:
        raise
    except Exception as e:
        die(f"unexpected failure: {e}")
    raise SystemExit(code)


if __name__ == "__main__":
    main()
