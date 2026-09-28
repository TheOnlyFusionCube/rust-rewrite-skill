# Reliability program Phase A/B/C report (artifact-only)

Times in America/Los_Angeles (PT). Scores from summary.json / cargo / selfcheck only.

## 1. Phase A — Nemotron FastAPI re-run (hardened v7.2)

| run | model | tests | completion | peak | lib_changed | tests_mutated | wall_s | goal_met | binary | status | notes |
|-----|-------|-------|------------|------|-------------|---------------|--------|----------|--------|--------|-------|
| nemotron-fastapi-001 | nemotron-3-ultra-free | 69/70 | 0.9857 | 10/10 | true | false | 1285.078 | true | 0.0 | ERROR | sole miss `m10_bad_content_type`; polish timed out @480s prior harness |
| **nemotron-fastapi-002** | nemotron-3-ultra-free | **70/70** | **1.0** | **10/10** | true | false | **790.35** | true | **1.0** | **PASS** | `m10_bad_content_type` cleared; **no polish turns** (all green after stage10) |
| mimo best `20260928-012328-2a099ec4` | mimo-v2.6-flash-free | 70/70 | 1.0 | 10/10 | true | false | 696.693 | true | 1.0 | PASS | prior best |

Stages (002): ran 1,2,5,8,10; skipped 3,4,6,7,9 already_green. Skip-green + CT422 docs/hints effective.

## 2. Phase B — real inih fixture

- Path: `fixtures/repos/repo_inih_real/`
- Pinned commit: `2bbdec4a366c8c39746ee0982e7ca0febbb044b6` (benhoyt/inih)
- Milestones: **8** (`m01_`…`m08_`); frozen tests: **50**
- Stub baseline: **2/50** passed (completion **0.04**) — `artifacts/opencode-stress/inih-real-baseline/summary.json`
- Selfcheck: OK (19 goldens vs C oracle)
- Suite: `suite.repo-v9-inih-real.json` (v9); **`suite.json` left as v8 ini_mini**
- Layout: `source/upstream/{ini.c,ini.h,LICENSE.txt,COMMIT_SHA}`, `source/oracle/`, `source/fixtures/`, `source/golden/`, `source/selfcheck.py`, `rust/`, `PORTING.md`

## 3. Phase C — both models on real inih

| run | model | tests | completion | peak | lib_changed | tests_mutated | wall_s | goal_met | binary | status | failure modes |
|-----|-------|-------|------------|------|-------------|---------------|--------|----------|--------|--------|---------------|
| mimo-inih-real-001 | mimo-v2.6-flash-free | 50/50 | 1.0 | 8/8 | true | false | 547.622 | true | 1.0 | PASS | none; stage1 solved all; 2–8 skip already_green; no polish |
| nemotron-inih-real-001 | nemotron-3-ultra-free | 50/50 | 1.0 | 8/8 | true | false | 242.372 | true | 1.0 | PASS | none; stage1+2; then skip; brief m02 residual (`m02_test7_blank_via_inline`) fixed in stage2 |

## 4. Confidence

**Works means:** hardened harness + inject/force-edit/staged/skip-green/polish scaffolding; real upstream C vendored + C-oracle goldens; frozen Rust milestone tests for a documented inih **subset**; models can reach 50/50 on that subset.

**Does NOT claim:** full inih rewrite; every compile-time knob (heap, stop-on-first-error, no-value, call-on-new-section, custom allocator); C++ INIReader; or production drop-in for all inih users.

## 5. Exact artifact paths

- Phase A: `artifacts/opencode-stress/nemotron-fastapi-002/{summary.json,results.jsonl,console via nemotron-fastapi-002-console.log}`
- Phase A prior: `artifacts/opencode-stress/nemotron-fastapi-001/summary.json`
- mimo FastAPI best: `artifacts/opencode-stress/20260928-012328-2a099ec4/summary.json`
- Phase B fixture: `fixtures/repos/repo_inih_real/`
- Phase B baseline: `artifacts/opencode-stress/inih-real-baseline/{summary.json,cargo-test-stub.txt}`
- Suite: `suite.repo-v9-inih-real.json`
- Phase C: `artifacts/opencode-stress/mimo-inih-real-001/`, `artifacts/opencode-stress/nemotron-inih-real-001/` (+ `*-console.log`)
