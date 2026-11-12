# Module research template

Every research file in `docs/research/` follows this structure. Write in English,
matching the repo's docs style. Quote exact C# locations (`File.cs:1234`) and exact
Rust locations (`crates/tr-app/src/...rs::symbol`). Verify claims against source
code, never against memory. "Not found" is a valid, valuable answer.

```markdown
# <NN> — <module name>

> Wave <N> · researched <date> · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope
Files/classes in this module (with line counts). One paragraph on responsibility.

## Key entities
- `path::Symbol` (lines) — one-line description. (entry points, state holders, dispatchers)

## Dependencies
- Calls into: ...
- Called by: ...
- Shared data structures: ...

## Port status
| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| ... | PORTED / PARTIAL / MISSING | crates/...rs::sym | what is missing or diverged, with C# line cites |

## Tests
Which cargo tests cover this module (`grep "#\[test\]"` results with file::test names).

## Deviations & risks
Every mismatch found — even in rows ROADMAP marks DONE. Include severity (cosmetic /
 behavioural / determinism / pixel) and whether ROADMAP/ANALYSIS already admits it.

## Task seeds
Concrete future tasks (port X, fix Y, verify Z), each with size S/M/L and any
dependencies between them.
```

Rules:
- One file per module. Never edit another module's file (append-only within your own).
- ROADMAP `DONE` rows are claims to re-verify, not facts: read the Rust code and the
  cited C# before confirming.
- Do not run cargo builds/tests (too slow for research); static cross-reading only.
- Do not modify any repo file except your own research file.
