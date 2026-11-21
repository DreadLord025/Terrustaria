# 266 — c42 re-pick CRITICAL: the device-arm halves on `task/W4-c42-wiring-device-arms` CANNOT MERGE AS-IS

> **RENUMBERED 146 → 266 (2026-11-21, registry research/150 §6.3,
> `origin/task/W4-d54-dust-sites-world` `72226e500`):** 146 was won by
> W4-g24's `146-wg-sounds-final.md` (first-add 10:02:44 `2678cfae4`,
> self-rename 107→146); this doc's first-add (10:22:27 `e47e1399f`) was
> second of the three 146 claimants (g24 keeps, this doc → 266, g67 →
> 267). Body unchanged; the original next-free rationale below is kept
> as historical provenance only. All cross-refs on this branch re-pointed.

> **STATUS: ESCALATION — 2026-11-21 10:10 +0200, branch
> `task/W4-e20-wiring-device-arms-2` (FINAL `02ac64ab6`, retire phase).**
> Standalone publication of the research-71 §6a/§7a evidence so the merger
> sees it WITHOUT reading the e20 branch's research-71 copy (c42's own copy
> is the stale pre-§6a doc and never carried the instruction). Number 146
> is the next free research number (INDEX-free check: 145 is taken TWICE on
> other lanes — `145-rod-teleport-union-composition.md` `41dabfbc8` and
> `145-16box-cross-lane-repins.md` `c1586b9cb`; the full taken set across
> all refs ends at 145).
> Code-only lane: no cargo/tests/graphify run; all proofs are git-object
> reads quoted below.

## ⛔ MERGER BLOCKING WARNING

**The device-arm halves of `origin/task/W4-c42-wiring-device-arms`
(tip `79b81ae77`, fold-133) MUST NOT be merged as-is.** They are a stale
re-port (`e8e6b159c`, "union-exec item 4 finish") of the OLD research-71
base `8791f07fe` (cherry-picked as `f1c24c381`), and they:

1. **DO NOT COMPILE** — a slice-`resize` pair (proof §2 below), and
2. **DROP the entire hammer/placement/track-item census** (proof §3).

The canonical device-arm code is the e20 branch
`task/W4-e20-wiring-device-arms-2`, tip `02ac64ab6` (census close
`6d1754719`). **Re-pick from the `6d1754719` chain (§1), NOT from
`8791f07fe`/`f1c24c381`/`e8e6b159c`.** c42's NON-device work (union-exec
doors/trapdoors/tall-gates/light unification, the fold-133 series
`e5569c2a7`…`79b81ae77`, and the `de176ccdd` fix round — verified ANCESTOR
of the c42 tip) is independent and merges on its own terms.

## 1. The exact re-pick commit list — the `6d1754719` chain

Branch `task/W4-e20-wiring-device-arms-2`, base `a725acb04` (= origin/main
at branch point), 25 commits, oldest → newest. The merger takes the
branch STATE at `02ac64ab6` for the device halves; the list is the audit
trail (later doc commits supersede earlier ones):

| # | Commit | Role |
| --- | --- | --- |
| 1 | `cb14ba4fb` | arm-137 fix round: wall traps −10/−9/−8 `speedY=0`; spiky-ball cap anchor+Center (F3), `TrapRequest::Spear` gains `cap_x/cap_y` |
| 2 | `ee6df0f2d` | light-family + 429 WireBulb witnesses |
| 3 | `fd34e2c84` | the three D8-residual arms (314 / 21+467 / 219+642): FlipSwitchTrack, FrameTrack(pound), Hopper, Extractinator |
| 4 | `44d7d20b7` | b=0 neighbour-rule witness (D1) |
| 5 | `474bc8b55` | hopper grab box strict-`<` + floored top-left |
| 6 | `df4dc1c8c` | docs: D8 closed (research-23, ANALYSIS, PORT_MAP, 4 anchors) |
| 7 | `dcbb29f82` | hopper partial-merge remainder (`(placed, remaining)`) |
| 8 | `dfabdb5b8` | cap fixtures re-derived for the verbatim anchor |
| 9 | `2322af906` | lantern witness fix (shared frameX 0/18) |
| 10 | `8791f07fe` | docs: research-71 v1 — **SUPERSEDED, this is the commit c42 wrongly picked** |
| 11 | `f167082ed` | merge-in 1/4: PlaySound(7) empty-slot hopper cue + the case-7 wiring-stream draw |
| 12 | `6336e3b44` | merge-in 2/4: OnceLock build-once switch-options cache |
| 13 | `b0d0836f1` | merge-in 3a/4: f11 witnesses batch A rebased |
| 14 | `5272cd52c` | merge-in 3b/4: f11 witnesses batch B+C + the table pin |
| 15 | `a2d6cdad0` | **the slot-Vec fix** — `&mut Vec<ItemSlot>` signatures (the compile fix c42 LOST) |
| 16 | `cf0b8f9c0` | docs: research-71 EXECUTED (merge-IN phase) |
| 17 | `0cea13365` | §3b.4 hammer-placement: FULL FrameTrack + `PoundOutcome::TrackHammer` + the Item52 cue/wake drain |
| 18 | `865e9d83f` | track-piece census: `edit::place_track`, `minecart::track_item`, the `cell_drops` 314 arm, the placement wake |
| 19 | `f74384e97` | hopper INPUT-side census (grab box, no chest-above arm; IsHopperInRangeOf MP-only) |
| 20 | `0055ac801` | consume draw-independence sweep (all 16 kinds) |
| 21 | `f3f090dd2` | docs: research-71 §6 addendum + the §6a c42 cross-note |
| 22 | `e38aef2f3` | placement-wake witness fix (outer supports) |
| 23 | `29d9dc224` | first-compile mechanical: `cell_drops` 314 arm yields `Vec<(u16,i32)>` |
| 24 | `6d1754719` | final census close: `minecart::is_pressure_plate` (Minecart.cs:549-560) |
| 25 | `02ac64ab6` | docs: research-71 §7 FINAL-FINAL (the four closing censuses) |

**Ancestry proof (run 2026-11-21 ~10:00, this worktree):**
`git merge-base --is-ancestor <c> 79b81ae77` → exit 1 (NOT-ancestor) for
ALL of: `cf0b8f9c0 0055ac801 a2d6cdad0 0cea13365 f74384e97 865e9d83f
29d9dc224 e38aef2f3 6d1754719 f167082ed 6336e3b44 b0d0836f1 5272cd52c
02ac64ab6 8791f07fe`. Exit 0 (ANCESTOR) only for c42's own copies
`f1c24c381` and `e8e6b159c`. `de176ccdd` → ANCESTOR (stays c42's).

## 2. Compile-error proof — the slice-`resize` pair (at tip `79b81ae77`)

`git show 79b81ae77:crates/tr-app/src/world_sim/wiring.rs`:

```text
627:     slots: &mut [crate::item::ItemSlot],     ← try_move_coins_in_chest signature (SLICE)
674:     slots.resize(t + 1, crate::item::EMPTY_SLOT);
726:     slots: &mut [crate::item::ItemSlot],     ← try_put_item_in_chest signature (SLICE)
748:     slots.resize(idx + 1, crate::item::EMPTY_SLOT);
```

`[T]` has no `resize` method (only `Vec<T>` does), and no extension trait
supplies one: `git grep -E "trait .*Resize|fn resize" 79b81ae77 -- crates/`
hits ONLY `tr-render/src/gpu.rs:126` / `post.rs:486/:491` (wgpu surface
resize — unrelated types). **E0599 latent compile error, twice.** This is
the exact defect `a2d6cdad0` fixed on the e20 branch; the kept signatures
there are `&mut Vec<crate::item::ItemSlot>` (`wiring.rs:600/:697` at HEAD).

Routing proof: c42's `hopper_pull` feeds these arms through
`crate::chest::chest_slots_mut` (`wiring.rs:817/:843/:897`), which returns
`Option<&mut [ItemSlot]>` (`chest.rs:228-232`,
`.map(|c| c.slots.as_mut_slice())`) — the slice conversion is baked into
the accessor, so the fix needs BOTH the signatures and the direct
`&mut app.world_chests[idx].slots` routing (the read-only sort path keeps
`chest_slots_mut`), exactly as `a2d6cdad0` does.

Why it stayed latent: the c42 lane is code-only (no cargo run); the error
surfaces at the first build — i.e. AT THE MERGE GATE, blocking everything.

## 3. Missing-half proof — the hammer/placement/track-item census absent

At tip `79b81ae77`:

- `git grep -E "frame_track_hammer|frame_track_place|refit_tracks_around|TrackHammer" -- crates/`
  → **ZERO hits**. `edit/hammer.rs:413-417` still returns
  `PoundOutcome::Consumed` for 314 with the comment "the track-switch
  machinery is unported … seam" — the pre-`0cea13365` state.
- `git grep "fn track_item"` → **ZERO hits**; every `place_track` hit is a
  TEST fixture (`minecart.rs:1568` inside `#[cfg(test)]` at :1536;
  `player.rs:17160` inside the test module) — no `edit::place_track`, no
  `cell_drops` 314 arm in `world_sim/editing.rs` (the fn at :6180 has no
  314 case) — the pre-`865e9d83f` state.
- The doc itself: c42's `docs/research/71-wiring-device-reconcile.md` is
  **byte-identical to `8791f07fe`'s v1** (`git diff 8791f07fe 79b81ae77 --
  docs/research/71-wiring-device-reconcile.md` → 0 bytes; 201 raw lines /
  176 non-blank vs the e20 FINAL's 496 raw lines). No STATUS banner, no
  §6/§6a/§7 — the branch never saw the re-pick instruction. (Research-71
  §7a's "176-line" figure counted non-blank lines; same doc.)

## 4. Session probe — c42 is ALIVE but NOT consuming the re-pick (10:10)

Read-only probe of worktree `C:\Users\DreadLord\Desktop\TR-W4-c42`
(direct file paths only, no junction recursion):

- HEAD `79b81ae77` = origin tip; reflog's last commit **07:24:20** (tip age
  ≈ 2 h 46 m at probe time 10:10) — no new commits since fold-133.
- **BUT the working tree is LIVE-DIRTY**: 10 modified files
  (`git status --porcelain`), `world_sim/wiring.rs` mtime **10:07:50**,
  `tr-core/src/wiring.rs` **10:06:22** — edits ~2 minutes before the probe.
  The c42e session dispatched long ago is **NOT dead**.
- The dirty work is research/95 §8 (the `Collision.EmptyTile` entity-box
  stamp + `NetMessage.SendData(19)` door echoes — `entity_boxes`,
  `set_entity_boxes`, `stamp_wiring_entity_boxes` in the diff), **NOT the
  re-pick**: the dirty `wiring.rs` STILL carries the slice signatures
  (:674/:773) + `slots.resize` (:721/:795), and `TrackHammer` still greps
  zero in the dirty `minecart.rs`/`hammer.rs`.

**Verdict + recommendation:** c42e is alive and mid-flight on §8; it has
not consumed §6a/§7a and shows no sign of doing so (its own research-71
copy lacks the instruction). Do NOT kill or wait on it. Two safe paths,
in preference order:

1. **Merger-side re-pick (recommended):** merge c42's non-device items on
   their own terms, take the e20 branch's device halves wholesale (§1
   list, state at `02ac64ab6`), per research-71 §5+§6a+§7a. No c42
   cooperation needed.
2. **c42-repick-resume dispatch:** if the lead wants c42 to self-heal,
   dispatch a resume task pointing c42e AT THIS DOC (its worktree copy of
   research-71 is stale): revert its `e8e6b159c` device halves, cherry-pick
   the §1 chain (or take the e20 file states), keep its §8 dirty work.
   Only needed if path 1 is rejected.

*All proofs are git-object reads at the named commits; probes were
read-only (`status`, `diff`, `reflog`, `Get-Item` mtimes on direct file
paths). No cargo/tests/rustfmt/graphify run (code-only retire lane).*
