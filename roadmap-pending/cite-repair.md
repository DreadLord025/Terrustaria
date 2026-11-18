# ROADMAP pending update — cite-repair

| Status | Task | Description | Size | Seev | Research | Deps | Verify |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ✅ **DONE 2026-11-17** | 🟡 cite-repair | Four broken C# citations repaired comment-only — `GameDifficultyData.cs:96-100` → `:81-92` (static ctor, `DebuffTimeMultiplier` on `:89`), `WorldMap.cs:406-409` → `:44-47` (`IsRevealed`), `Projectile.cs:14258-13956` → `:13904-14263` (reversed; the per-type whip switch through the aiStyle-165 retarget), `LiquidPlacement.cs:5-10` → `:3-9` (enum declaration) | XS | — | — | — | `cs-cite.py` PASS on all four files, 0 OUT-OF-RANGE |

---

**Verification performed:**
- `python cs-cite.py <each of the 4 files> --repo <worktree>` — VERDICT: PASS, 0 OUT-OF-RANGE, 0 FILE-MISSING for every file
- Comment-only edits: `git diff --stat` shows 4 files changed, 4 insertions(+), 4 deletions(-)
- No cargo run (another agent held the build lock; nothing but comments changed)

**Files changed in this task:**
- `crates/tr-app/src/buffs.rs`, `crates/tr-app/src/map.rs`, `crates/tr-app/src/projectile.rs`, `crates/tr-core/src/tileobject.rs` (one comment line each)
