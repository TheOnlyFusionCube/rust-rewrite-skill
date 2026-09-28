---
name: rust-rewrite-reliable
description: >-
  Use this when porting a working reference (often Python or C) into Rust on
  mid-tier/inferior LLMs: inject port guide + API surface, force early edit of
  rust/src, chunked RIIR milestone gates, frozen-test oracle, differential
  selfcheck, auto-resume on no-edit, multi-turn polish (esp. content-type 422),
  anti-cheat fingerprints; never weaken tests.
---
# rust-rewrite-reliable

Use this skill when porting a working reference (Python, C, or similar) into a Rust crate under mid-tier models that otherwise **only read**, blow context, mutate tests, leapfrog milestones, or miss adversarial 422/content-type edges.

## When to apply

- Repo-rewrite / RIIR-mini with `source/` oracle + `rust/` stub + `PORTING.md`
- Ordered `m01_`…`mN_` cargo filters and frozen `rust/tests/`
- Free/mid-tier models (`--variant max` ok) that need scaffolding discipline
- Harness with reliability flags (inject docs, force-edit, staged loop, resume, polish)

## Non-negotiables

1. **Never** edit, weaken, delete, rename, `#[ignore]`, or skip frozen tests.
2. Prefer **std-only** Rust unless the port guide explicitly allows a crate.
3. Treat `source/` (+ README / PORTING.md) as the **behavioral contract**.
4. Score honesty > looking green: do not invent results; read cargo output.
5. **A turn that only reads/chats and never edits `rust/src/*.rs` is a failed turn.**

## Exact ordered checklist (agents MUST follow)

1. **Ingest contract (cheap)** — Prefer harness-injected `PORTING.md` + README quirks + public API surface. If not injected: read those three only. Do **not** open every milestone test file.
2. **Preflight target** — Note failing names for the **current** filter (usually `m01_`). Identify stub hotspots.
3. **Edit first** — Within the first tool budget after ingest: **write/edit** `rust/src/lib.rs` (keep stub signatures; fill bodies). Do not plan the entire port in chat.
4. **Verify the band** — From `rust/`: `cargo test m0K_` for the current filter. Diff failures against the oracle on the **same** input.
5. **Gate before advancing** — Stay on the current band until ≥ ~60% of its tests pass. Only then open the next filter. Re-run prior bands for regressions.
6. **Integrity** — Confirm `rust/tests/` fingerprint unchanged.
7. **Stop / hand off** — Stage done = `lib.rs` differs from stub **and** current filter shows real progress.

## Failure modes

| Mode | Symptom | Fix |
|------|---------|-----|
| **no-edit** | `lib.rs` sha == stub; only read/bash; step_finish length with huge reasoning | Force-edit mandate; inject docs; forbid reading all tests; auto-resume |
| **test mutation** | Fingerprint drift; `#[ignore]`; deleted asserts | Fail run; restore tests; never score as pass |
| **leapfrog** | Editing for m09 while m01 <60% | Milestone gate; staged prompts; unlock threshold |
| **inventing API** | New type names / different method shapes than stub/tests | Keep stub signatures; implement bodies only |
| **softening 422/OpenAPI** | Looser matchers, partial OpenAPI, wrong msg/type | Match README quirk rows exactly; golden JSON |
| **bad content-type → 200** | `post_raw(..., "text/plain")` on required JSON body returns 200 | If CT lacks `application/json` (and is non-empty), return **422** `type_error.json` with `loc` including `body` — even when body parses as JSON |
| **context blowout** | Reading all milestones + all source before any write | Read budget; inject PORTING/README/API; one filter at a time |
| **chat-only planning** | Long reasoning, 0 write tools, exit 0 | Resume with failing test list; mandate write/edit |
| **polish timeout** | One remaining adversarial fail after stages | Multi-turn polish; longer polish timeout; spotlight remaining names |

## Staged loop protocol (harness + agent)

**Budgets (defaults; suite may override):**
- Wall budget ≈ 3600s per case
- Stage timeout ≈ 600s; resume timeout ≈ 900s; polish timeout ≈ 900s
- Max stages ≈ 10; max polish turns ≈ 2
- Unlock threshold ≈ 0.60 per milestone filter

**Per stage `S` (filter `m0S_`):**
0. Preflight skip: if filter already ≥ unlock and zero failures, skip the model turn.
1. Preflight: list failing test names for `m0S_`.
2. Prompt: inject contract + **MUST edit** + failing list + "do not open later tests".
3. Run model once.
4. Postcheck: `lib.rs` hash ≠ stub; re-run `cargo test m0S_`.
5. If unchanged **or** fraction not improved / still < unlock → **one resume** turn.
6. If fraction < unlock → **stop advancing**. Else start stage S+1.

**Polish:** after stages, while failures remain and budget allows (up to `max_polish_turns`): list all remaining failing names; if any name mentions content_type, inject the HARD CT RULE; edit `lib.rs`; re-score.

**Final:** always run full `riir_cargo_milestones` scoring (all bands, frozen fingerprint).

## Diff / selfcheck commands

```bash
python3 source/tests/selfcheck.py   # or project-equivalent
cd rust && cargo test m01_
cd rust && cargo test m02_   # only after m01_ ≥60%
sha256sum rust/src/lib.rs   # must differ from fixture stub
```

Differential loop: for each failing assert, call the oracle and Rust on the **same** input; align status, body, headers (`allow`), and 422 `detail[]` `loc`/`msg`/`type`.

For JSON body routes: prove `text/plain` + `{}` → 422 before claiming m10 done.

## What "done" means

| Level | Definition |
|-------|------------|
| Stage done | `lib.rs` ≠ stub AND current filter fraction improved toward ≥60% |
| Milestone unlocked | Filter fraction ≥ unlock threshold (default 0.60) |
| Suite binary pass | Mean completion ≥ goal (default 0.75) AND `tests_mutated=false` |
| Honest report | Record run_id, completion, peak milestone, `lib_rs_changed`, wall_s — never invent |

## Prompt skeleton

```
HARD: edit rust/src/lib.rs before finishing; never edit rust/tests/.
Focus: cargo test m0K_ only until ≥60%.
Contract: PORTING.md + README quirks (injected). Keep stub API; fill bodies.
CT: non-json content-type on required JSON body → 422 type_error.json (never 200).
Failing now: <names>. Resume if lib.rs still stub.
```

## Anti-patterns

- Claiming full-framework / full-upstream parity when the fixture is a subset
- Softening 422 / OpenAPI / content-type exactness to chase completion
- Reading every test file before the first edit
- Running paid models when the suite forbids them
- Pasting API keys into artifacts or prompts
- Weakening/deleting frozen tests to inflate scores
