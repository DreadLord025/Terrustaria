# Roadmap & systems atlas — Terraria-Rust (1.4.5.8)

**Rewritten 2026-11-02** from a full audit of the decompiled sources: nine per-system
mechanics inventories with `File.cs:line` anchors (run on `decompiled/Terraria`, 1 550
files / ~642 000 lines). This file is both the **task catalog** (take tasks from the
bottom) and the **systems atlas** (use it to scope and order them). It supersedes the
2026-10-29 `docs/TERRARIA_SYSTEMS_CATALOG.md`, which is deleted — its content counts and
several entries were wrong (e.g. invented bosses, "550+ items" instead of ~6 020).

Anchors are the decompile's line numbers. They are **not** decorative: per the golden
rule (agents.md) every port re-reads the cited C# before writing Rust. Constants quoted
here are transcribed from those audits — re-verify against the source when porting.

## How tasks work (branch-per-task, unchanged)

- Tasks are taken from the **Task catalog** at the bottom (or a direct user request).
- `git fetch --prune`, check `git branch -a` — a catalog row is *taken* when a branch
  with its ID exists. Then `git checkout -b <Task-ID>-short-slug` from `main`.
- One branch = one task; merge back per the loop in agents.md (sync → verify → merge →
  delete → graphify refresh).
- Rows marked 🟡 list what is *still missing*; read the linked atlas section before
  starting. In-flight rows name the live branch.

## Legend
>>>>>>> origin/main
