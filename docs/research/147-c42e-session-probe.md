# 147 — c42e session probe: ALIVE, mid-flight on research/95 §8 — NOT consuming the re-pick

> **STATUS: PROBE REPORT — 2026-11-21 10:10 +0200, branch
> `task/W4-e20-wiring-device-arms-2` (retire phase).** Read-only probe;
> no file in `TR-W4-c42` was written, no git write ran there. Companion
> to research-146 (the CRITICAL escalation); number 147 next-free at
> publication (taken set across all refs ends at 145; 146 taken by this
> lane's phase-1 commit `e47e1399f`).

## Question

`c42e` was dispatched long ago against `task/W4-c42-wiring-device-arms`.
Is the session dead? If dead, a `c42-repick-resume` dispatch is needed to
consume the research-71 §6a/§7a re-pick instruction (research-146).

## Evidence (all read-only, direct file paths — no junction recursion)

| Probe | Result |
| --- | --- |
| `git ls-remote origin` | `refs/heads/task/W4-c42-wiring-device-arms` = `79b81ae77d173642b22a18e1e4213fbaac554a29` |
| tip commit date | `2026-11-21 07:24:20 +0200` (fold-133 docs, research/95 §10.1 EXECUTED + the 133-fold-receipt) — **tip age ≈ 2 h 46 m at probe time 10:10** |
| worktree registration | `git worktree list`: `C:/Users/DreadLord/Desktop/TR-W4-c42  79b81ae77 [task/W4-c42-wiring-device-arms]` — HEAD == origin tip |
| worktree dir mtime | `2026-11-21 05:33:18` (dir-entry mtime only reflects entry add/remove — NOT a liveness signal) |
| tracked-file mtimes | `world_sim/wiring.rs` **10:07:50**, `tr-core/src/wiring.rs` **10:06:22** (≈ 2 min before probe); `minecart.rs` 06:19:09; research-71 copy 05:35:32; research-95 07:23:44; 133-fold-receipt 07:22:30 |
| `git -C TR-W4-c42 status --porcelain` | **10 modified files**: netglue.rs, chest.rs, editing.rs, editing/doors.rs, npcs.rs, player.rs, projectiles.rs, tile_entities.rs, world_sim/wiring.rs, tr-core/wiring.rs |
| `git -C TR-W4-c42 diff --stat` | 321 insertions / 33 deletions across the 10 files |
| reflog tail | last entry `HEAD@{2026-11-21 07:24:20}: commit … 79b81ae77` — no commits since fold-133, but uncommitted edits are fresh |
| dirty-diff content | research/95 **§8.1** `Collision.EmptyTile(ignoreTiles:true)` entity-box stamp (`entity_boxes` field + `set_entity_boxes` + `cell_blocked_by_entity`, `Collision.cs:1359-1378`) and **§8.2** `NetMessage.SendData(19)` door echoes (`door_echoes`, `Wiring.cs:1450/:1458/:1465/:1480/:1485`, g48b codec contract) |
| re-pick consumed? | **NO.** Dirty `world_sim/wiring.rs` STILL has `slots: &mut [crate::item::ItemSlot]` (:674/:773) + `slots.resize` (:721/:795); `TrackHammer`/`frame_track_hammer`/`frame_track_place`/`refit_tracks_around` grep ZERO in the dirty `minecart.rs` + `edit/hammer.rs`; the worktree's research-71 copy is still the stale `8791f07fe` v1 (mtime 05:35:32, pre-§6a) |

## Verdict

**c42e is ALIVE — not dead.** The 10:06–10:07 mtimes (2 minutes before the
probe) plus a coherent 321-line §8 work-in-progress prove an active
session. It is working research/95 §8 (the entity-box gate + door echoes),
the next items after its fold-133 series — **not** the device-arm re-pick.
Its own research-71 copy predates §6a, so it has never seen the re-pick
instruction and shows no sign of acting on it.

## Recommendation

**No `c42-repick-resume` needed for liveness — but the re-pick still will
not happen by itself.** In preference order:

1. **Merger-side re-pick (recommended, research-146 §1):** take the e20
   branch's device halves wholesale (state at `02ac64ab6`); c42's
   non-device union-exec + fold-133 + `de176ccdd` + the in-flight §8 work
   merge on their own terms. Zero coordination with c42e required; its
   §8 edits touch the wiring engine's gate/echo seams, not the
   hopper/track device bodies, so the overlap is textual-adjacency only.
2. **c42-repick-resume dispatch (only if path 1 is rejected):** hand c42e
   research-146 directly (its worktree research-71 is stale): revert the
   `e8e6b159c` device halves, take the §1 chain states, keep the §8 dirty
   work. Do NOT kill or reset the worktree — the dirty §8 state is real
   uncommitted work (the 2026-11-08 stale-index rule: never clean a
   checkout you do not own).

*Probe commands: `git ls-remote`, `git worktree list`, `git -C … status
--porcelain / diff --stat / diff / reflog`, `Get-Item` LastWriteTime on
direct tracked-file paths, `Select-String` on direct file paths. No
cargo/tests/rustfmt/graphify (code-only retire lane).*
