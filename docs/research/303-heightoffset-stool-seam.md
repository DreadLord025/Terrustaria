# 303 — The HeightOffset family seam + the portableStool deferral (movement-lane task seed)

> **RENUMBER-CHAIN 65 → 303** (registry-150 §7.3 round-1 owed rename, d54 tip `6339141a1`):
> 65 belongs to W4-f17's round-1 winner (`65-kinds-coverage-1.md`,
> 11-20T20:49:39) — f3's doc missed rounds 1–2 (claim `63fbeaa4f` 12:36:05)
> and takes the §7.3-assigned number **303** (FROZEN §9.1 KEEPS: `303=f3`).
> This doc now carries key **303**; refs updated on this branch:
> `docs/research/INDEX.md` (row 303), `docs/ANALYSIS.md` (×3),
> `docs/PORT_MAP.md` (×1). The 378-rename (67→378, §10.2) already landed at
> `3980e0c5b` — unchanged.

**Lane:** the player-move family (pulley / carpet / wallclimb / stealth — W4-f3
canonical). **Status:** census + deferral record, code-only (no compiler invoked).
**Producer:** W4-f3 retire pass, `task/W4-f3-dust-player-3`, 2026-11-21.
**Consumer:** the movement-lane task that lands the mount/stool offset seam
(`portableStool` + the mount-offset family together — NOT self-contained, see §3).

The C# `Player` exposes five derived offset properties. The port folds SOME of
them to 0 and ports OTHERS piecemeal; the stool machine needs the whole family
at once. This doc is the fold-site census (where each property is consumed in
C#, what the port does today, and what the seam task must re-site).

## 1. The family (all `Player.cs`, all getters — no backing fields)

| Property | C# | mount.Active arm | stool IsInUse arm | else |
| --- | --- | --- | --- | --- |
| `HeightOffsetVisual` (float) | `:3237-3251` | `mount.PlayerOffset` | `portableStoolInfo.VisualYOffset` | 0 |
| `HeightOffsetHitboxCenter` (float) | `:3253-3267` | `mount.PlayerOffsetHitbox` | `HeightBoost - VisualYOffset` | 0 |
| `MountXOffset` (float) | `:3269-3279` | `mount.PlayerXOFfset` | — | 0 |
| `HeightOffsetBoost` (int) | `:3281-3295` | `mount.HeightBoost` | `portableStoolInfo.HeightBoost` | 0 |
| `HeightMapOffset` (int) | `:3297-3311` | `mount.PlayerHeadOffset` | `portableStoolInfo.HeightBoost` | 0 |
| `BaseHeight` (float) | `:3335` | `height - HeightOffsetBoost` (derived) | | |

`PortableStoolUsage` (`DataStructures/PortableStoolUsage.cs:3-31`): a 5-field
struct (`HasAStool`, `IsInUse`, `HeightBoost`, `VisualYOffset`, `MapYOffset`)
+ `Reset` + `SetStats`. The equip rows: items **4341 / 5126**
(`Player.cs:14544-14546`, `SetStats(26, 26, 26)` — all three offsets 26);
`Reset` at `:18783` (beside `ResizeHitbox` `:18784`).

## 2. The consumers-list (the seam's blast radius)

### HeightOffsetVisual (the DRAW offset)
* `PlayerDrawSet.cs:428-429` — `mountOffSet = HeightOffsetVisual; Position.Y -= mountOffSet`.
* `PlayerDrawLayers.cs:3979` — `38f - HeightOffsetVisual` (a layer's Y anchor).
* **Port today:** `render/player_layers.rs:142` carries its own per-mount
  `player_offset: i32` table (rows `:2521-2691`), applied at `:1469`; the
  stool arm does not exist. The table duplicates `Mount.PlayerOffset`
  (`mount.rs:2920-2930`, `Mount.cs:407-421`) — the seam should unify them.

### HeightOffsetHitboxCenter (the CENTRE offset)
* `MountedCenter` get/set (`Player.cs:3341`/`:3345`): `position + (width/2,
  BaseHeight/2 + HeightOffsetHitboxCenter)`.
* `MountedTop` get/set (`:3354-3367`): `position.Y + HeightOffsetHitboxCenter`
  (gravDir 1) / `Bottom.Y - HeightOffsetHitboxCenter` (−1); the setters mirror.
* The held-item draw offset (`:42604`, `:42846` —
  `AnimatePlayerAndGetItemFrame(heightOffsetHitboxCenter, item)`).
* **Port today:** `player.rs::mounted_center` (`:5199`) folds to `center()`
  and `mounted_top` (`:5209`) folds to the body top-centre — BOTH offsets 0.
  The consumers already route through the accessors (the seam's leverage):
  `bow_launch.rs` (3 sites: the flail/bow anchor frame), `projectile.rs`
  (7 sites: the sentry-backpack reposition, the minion spawn anchors, the
  `mounted_center` input struct `:14610`), `projectile_ai_spear.rs` (1),
  `world_sim/editing.rs` (1), `world_sim/projectiles.rs` (3). Re-siting the
  two accessors re-sites every consumer without touching them.
  `render/player_layers.rs:1178` documents the draw-side fold
  (`mountOffset = HeightOffsetHitboxCenter` stands at 0).

### MountXOffset
* `PlayerDrawSet.cs:342` — `Position += (MountXOffset * direction, 0)`.
* **Port today:** no fold site (the draw X offset is unported); the seam adds
  it beside the Y offset in the same draw-set pass.

### HeightOffsetBoost (the HITBOX growth)
* `BaseHeight` (`:3335`) — feeds `MountedCenter`, the item-frame centring
  (`:50606-50607`), the MowTheLawn anchor (`:20307`).
* `ResizeHitbox` (`:29211`): `height = 42 + HeightOffsetBoost`.
* `CanFitSpace` (`:6102-6106`): the `42 + heightBoost` clear-spot probe.
* **Port today:** the MOUNT arm is ported piecemeal — `mount.rs` rows carry
  `height_boost` (39 rows), `player.rs:10876` grows the hitbox
  (`grow_hitbox(HEIGHT + equip.height_boost)`, the `:6264-6277`
  bottom-anchored growth), `:11015` the size preview, `:11900-11903` the
  cart body scan. The STOOL arm does not exist; `BaseHeight` is not modelled
  (the port's hitbox already includes the boost, so `center()` ≠ C#
  `MountedCenter` on a boosted mount — the known fold, `player.rs:5193-5198`).

### HeightMapOffset (the MINIMAP head dot)
* `PlayerDrawHeadSet.cs:155` — `Position.Y -= HeightMapOffset`.
* **Port today:** `player.rs::apply_head_offset_from_mount` (`:5306`,
  `Player.cs:51544-51645`) ports the per-mount head-offset TABLE (types
  54/55/56/61) for the projectile-side head anchors (`projectile.rs:13875-
  13883`); `mount.rs` rows carry `player_head_offset` (39 rows). The minimap
  head-dot consumer is the map lane's; the stool arm does not exist.

## 3. The portableStool machine — why it is NOT self-contained

`UpdatePortableStoolUsage` (`Player.cs:29193-29205`, called `:25647`):
`HasAStool && controlUp && !gravControl && !mount.Active && vx == 0 && vy == 0
&& !pulley && grappling[0] == -1` → `CanFitSpace(HeightBoost)` →
`IsInUse = true` + `ResizeHitbox` (`:29207-29222`: bottom-anchored
`position.Y += height`, `width = 20`, `height = 42 + HeightOffsetBoost`).

Every downstream consumer reads the offset family, not the struct:
* `:19614-19617` — `jumpHeight += 5` while in use (the jump-stat tail).
* `:19646` — the FindPulley gate (`IsInUse || (!controlUp && !controlDown)`
  refuses the attach). **Port:** folded false in `find_pulley`
  (`player.rs:10199`, the cited fold).
* `:20768-20776` — the jump-off: `IsInUse = false`, `position.Y -= HeightBoost`,
  the `StoolJump` camera modifier (local player only).
* `Center`/`Bottom`/`Top` (`:3341-3367`) — via HeightOffsetHitboxCenter (§2).
* The held-item draw (`:42604`/`:42846`) — via HeightOffsetHitboxCenter.
* `cPortableStool` dye slot (`:2976`, cleared `:9591`, set `:9735`) — a third
  seam (the dye pass).

A stool-only port would need: the struct + the two equip rows + the usage
machine + `ResizeHitbox`'s stool arm + `CanFitSpace` + the jump-stat tail +
the jump-off unboost + the camera modifier + the offset getters' stool arms
+ the dye slot — i.e. the whole seam. **Deferred WITH the seam, not dropped**
(the W4-f3 close-out (b) verdict, re-verified this pass).

## 4. The seam task's shape (recommendation)

1. Model the five getters on `Player` (mount arm reads the existing
   `mount.rs` accessors — `player_offset` `:2921`, `player_offset_hitbox`
   `:2933`, `height_boost`/`player_head_offset` rows; stool arm reads a new
   `portable_stool: PortableStoolUsage` field).
2. Re-site `mounted_center`/`mounted_top` to the C# expressions — every
   consumer in §2 routes through them already.
3. Unify the draw-side `player_layers.rs` offset table onto the getter
   (drop the duplicate rows), add `MountXOffset` + the head-dot offset.
4. Land the stool machine + its equip rows + the jump tail/unboost +
   `CanFitSpace` + the `StoolJump` camera modifier + the dye slot.
5. Un-fold the FindPulley stool gate (`player.rs:10199`).
6. Witnesses: the offset getters per mount kind (the `mount.rs` frame-table
   witness `:3727` is the precedent), the stool usage gate chain, the
   jump-off unboost position maths, the FindPulley refusal while perched.

**RNG contract:** none of the offsets draw; the stool machine is
deterministic state. The `StoolJump` camera modifier is local-player-only
(`Main.myPlayer == whoAmI`, `:20772`).

## 5. The lane-owner question + the task-seed (W4-f3 seal, 2026-11-21)

**Who owns the movement lane?** The player-move family (pulley / carpet /
wallclimb / stealth) is **W4-f3 canonical** (the PLAYER-MOVE-FAMILY declaration,
ANALYSIS retire item 4). The HeightOffset seam is the family's **deferred
mount/stool offset half** — it is NOT a separate subsystem and NOT self-contained
(§3). The seam touches three lanes at once:

* **The movement lane (f3's family)** — the `mounted_center`/`mounted_top`
  accessors (`player.rs:5204`/`:5214`) route ALL 15 downstream consumers
  (bow_launch / projectile / projectile_ai_spear / editing / world_sim/projectiles);
  the FindPulley stool gate (`player.rs:10199`) is f3's pulley machine.
* **The mount lane** — the five getters' mount arms read `mount.rs`
  (`player_offset` `:2921`, `player_offset_hitbox` `:2933`, the
  `height_boost`/`player_head_offset` rows); the piecemeal `height_boost` growth
  (`player.rs:10876`/`:11015`/`:11900`) is the mount framework's.
* **The draw lane** — the duplicate `player_layers.rs` offset table (`:142`,
  rows `:2521-2691`, applied `:1469`) + `MountXOffset` (`PlayerDrawSet.cs:342`) +
  the head-dot (`PlayerDrawHeadSet.cs:155`).

**Recommendation: the seam is a SINGLE movement-lane task, executed by the
player-move family's owner (the f3 successor / the merger's future wave), NOT
split across the mount/draw lanes.** The leverage is the two accessors (step 2):
re-siting them re-sites every consumer without touching the consumer files, so the
task is one coherent `player.rs` + `mount.rs` change with a draw-side unification
step. Splitting it would re-introduce the fold-to-0 drift the seam exists to
remove. The mount/draw lanes are CONSULTED (the `mount.rs` accessors already exist;
the `player_layers.rs` table is the duplicate to drop), not co-owners.

**Which lane executes the 6-step seam-task?** The **movement lane** (the f3
family's successor). The 6 steps (§4) are ordered so steps 1-2 (the getters + the
accessor re-site) are the movement-lane core; step 3 (the draw unification) is the
one cross-lane edit, done IN the same task because the getter is the single source
the draw table must read; steps 4-5 (the stool machine + the FindPulley un-fold)
are movement-lane; step 6 (witnesses) spans all three but is written by the task
owner. **The merger does NOT execute the seam** — it is a future-wave task seeded
here, deferred WITH the family's merge (the FindPulley stool gate stays folded
false until the seam lands).

### The task-seed (for the merger / future-wave catalog)

> **Task:** the HeightOffset-family mount/stool offset seam + portableStool.
> **Lane:** movement (the player-move family's deferred half). **Size:** M-L.
> **Sev:** B (the fold-to-0 is a documented deviation, not a crash). **Deps:** the
> e1b-141 player-move merge (the family must be canonical first). **Evidence:**
> this doc (§1-§4) + `Player.cs:3237-3311`/`:3335`/`:29193-29222`/`:6102-6106`/
> `:19614`/`:19646`/`:20768-20776` + `Mount.cs:407-433`/`:570-580` +
> `PortableStoolUsage.cs:3-31`. **Verify:** the offset getters per mount kind (the
> `mount.rs` frame-table witness precedent), the stool usage gate chain, the
> jump-off unboost position maths, the FindPulley refusal while perched, the
> `mounted_center`/`mounted_top` re-site (the 15 consumers route through them).
> **Scope:** the 6 steps (§4) as ONE task; the `player_layers.rs` duplicate table
> drops; `MountXOffset` + the head-dot offset land; the `cPortableStool` dye slot
> is a cited third seam (the dye pass). **Out of scope:** the dye pass itself, the
> minimap head-dot consumer (the map lane's). **RNG:** zero draws (deterministic
> state); the `StoolJump` camera modifier is local-player-only.

**Status:** seed (deferred WITH the seam, NOT dropped — the W4-f3 close-out (b)
verdict, re-verified at the seal). The portableStool machine + the offset family
land together in this task; neither lands alone.
