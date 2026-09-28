# rust-rewrite-reliable

**Make C→Rust and Python→Rust ports reliable on mid-tier LLMs** — not only frontier coding agents.

A RIIR-style harness + skill that stops common failure modes (no-edit, leapfrog, test mutation, content-type 422 misses) with:

- injected `PORTING.md` + API surface
- force-edit mandates
- staged milestone gates (unlock ≥60%)
- frozen-test fingerprint anti-cheat
- multi-turn polish with spotlighted regressions

Inspired by [RIIR Bench / RewriteBench](https://rewritebench.com) methodology at **mini-repo and real-upstream-subset** scale.

## Why this exists

Frontier agents can one-shot tiny ports. Mid-tier / free models often:

1. **Only read** — leave `lib.rs` byte-identical to the stub  
2. **Blow context** — dump every test file before writing  
3. **Leapfrog** — chase m10 while m01 is red  
4. **Mutate tests** — soften asserts to look green  
5. **Miss adversarial edges** — e.g. `text/plain` on a JSON body returns 200 instead of 422  

This repo is the scaffolding that lifts those models from ~7% completion to **full clear** on the FastAPI-shaped suite, and to **50/50** on a real `benhoyt/inih` subset — measured on OpenCode free models.

## Measured results (honest, artifact-backed)

| Suite | Model | Completion | Notes |
|-------|-------|------------|-------|
| FastAPI mini blood (no scaffolding) | `mimo-v2.6-flash-free` max | **0.07 (5/70)** | stub-identical `lib.rs` |
| FastAPI mini + reliability | same | **1.0 (70/70)** | staged + polish |
| FastAPI mini + reliability | `nemotron-3-ultra-free` max | **1.0 (70/70)** | after polish/timeout harden |
| Real inih C subset (`@2bbdec4`) | mimo + Nemotron | **1.0 (50/50)** each | real upstream source + goldens |

See [`docs/RESULTS.md`](docs/RESULTS.md) for run ids and walls.

**Claims:** scaffolding + frozen subset tests work for these models on these fixtures.  
**Non-claims:** full FastAPI, full inih, Bun-scale Zig→Rust, or paid frontier-only orchestration.

## Quick start

```bash
# Requires: Python 3.11+, Rust toolchain, OpenCode CLI + free-model auth
python3 run_suite.py --suite suite.repo-v7-fastapi-reliable.json \
  --model opencode/mimo-v2.6-flash-free --variant max

python3 run_suite.py --suite suite.repo-v9-inih-real.json \
  --model opencode/nemotron-3-ultra-free --variant max
```

`run_suite.py` wraps `opencode run` under a PTY (`script -q -c`). Do not call bare `opencode run` without a TTY.

## Layout

```
run_suite.py                 # harness (RIIR-mini scorer + reliability loop)
suite.repo-v7-fastapi-reliable.json
suite.repo-v9-inih-real.json
suite.json                   # default (ini_mini universality check)
fixtures/repos/
  repo_fastapi_mini/         # FastAPI-shaped Python→Rust (70 tests)
  repo_ini_mini/             # inih-shaped Python→Rust (68 tests)
  repo_inih_real/            # real benhoyt/inih C subset (50 tests)
  repo_chip8/                # CHIP-8 blood slice
.cursor/skills/rust-rewrite-reliable/SKILL.md
docs/RESULTS.md
```

## Reliability block (suite JSON)

```json
{
  "inject_porting_docs": true,
  "force_edit_mandate": true,
  "preflight_first_milestone": true,
  "postcheck_lib_changed": true,
  "auto_resume_if_stub": true,
  "staged_milestones": true,
  "stage_unlock_threshold": 0.6,
  "max_stages": 10,
  "polish_timeout_sec": 900,
  "max_polish_turns": 2,
  "wall_budget_sec": 3600,
  "forbid_bulk_test_reads": true
}
```

## Skill

Cursor skill: [`.cursor/skills/rust-rewrite-reliable/SKILL.md`](.cursor/skills/rust-rewrite-reliable/SKILL.md)

Use when porting a working reference into Rust on mid-tier models. Never weaken frozen tests.

## Keywords / topics

`rust` `riir` `c-to-rust` `python-to-rust` `llm` `agent-harness` `opencode` `rewritebench` `differential-testing` `mid-tier-llm` `frozen-tests` `inih` `fastapi`

## License

MIT — see [`LICENSE`](LICENSE). Upstream `inih` sources under `fixtures/repos/repo_inih_real/source/` retain their original license (see that tree).
