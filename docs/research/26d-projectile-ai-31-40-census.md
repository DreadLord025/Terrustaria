# 26d — Projectile aiStyles 31-40 COVERAGE census (W4-d4)

> Researched 2026-11-21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Port task: the MISSING styles in 31-40 are ported verbatim here (the d4
> decade claim). Companion to `26c-projectile-ai-1-105-census.md` (W4-f19),
> whose finding F3 assigns the style-37 arm+row to this task.

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs:16731-19779`). One top-level numeric arm per style; the
  coverage pin `s1_d_4c_every_rowed_style_has_exactly_one_live_arm`
  (`projectile.rs:23770`) enforces one arm per rowed style.
- In-flight probe: `git branch -a` + `git log` — every sibling `proj-ai`
  branch (d1-d21, g1/g2/g3-proj-ai-missing) sits at base `a725acb0` with NO
  code commit, so the five ports below collide with nothing in flight.
- C# arm spans: `AI()`'s if/else chain (`:23199-36815`); kind lists parsed from
  `SetDefaults`' `else if (this.type == N)` chain (`:697-10800`).

## Coverage table (31-40)

P = ported (dispatch line in `projectile.rs`), M = missing (C# body span +
SetDefaults kinds). "arm" = the Rust fn that flies the style.

| style | state | Rust dispatch / C# body | kinds | arm |
| --- | --- | --- | --- | --- |
| 31 | P | :17296 (`:26786-26886` Clentaminator spray) | 145 (+146-149/1015-1017 unrowed) | `projectile_ai_misc::ai_spray` |
| 32 | M | :26887-27048 (timeLeft=10, 255-player/NPC/1000-proj bounce scans at ai0≥20) | 155 BeachBall | this task |
| 33 | P | :17153 (`:27049-27135` flares) | 163/310/1008-1011 | `projectile_ai_light_items::flare_step` |
| 34 | P | :17522 (`:27136-27188` fireworks) | 167-170/415-418 | `projectile_ai_darts::firework_flight` |
| 35 | M | :27189-27291 (ai0>30 fall + ×0.985 drag, 14 clamp, rope-place walk) | 171/475/505/506 rope coils | this task |
| 36 | P | :17598 (`:27292-27471` bees/bat) | 181/566/307/316 | `projectile_ai_thrown::bee_step` + `ai_bat` |
| 37 | M | :27472-27499 (spear trap; row deviation — f19 F3) | 186 SpearTrap | this task |
| 38 | M | :27500-27512 (flame jet trap, 6-tick sound cycle + 188 spawn) | 187 FlamethrowerTrap | this task |
| 39 | M | :27513-27779 (alpha −50 + owner 2000-px cut + piranha homing) | 190 MechanicalPiranha | this task |
| 40 | P | :16973 (`:27780-27783` → `AI_040_LeafBlower`) | 206 Leaf | `ai_leaf_blower` |

**Verdict at census time: 5 of 10 styles armed (31/33/34/36/40); 5 MISSING
(32/35/37/38/39) — this task ports them one style per commit; the table's
state column is flipped to P as each arm lands.**

## Finding F3 carried from W4-f19 — style 37's row was a DELIBERATE stand-in

The Spear Trap's spear (186) is C# `aiStyle 37`, 10×14, tileCollide false,
ignoreWater, friendly AND hostile, penetrate −1, timeLeft 300, trap
(`Projectile.cs:2728-2740`). The port's row WAS the 185 spiky-ball stand-in
(14×14, aiStyle **14**, tileCollide true, timeLeft 900 —
`projectile/data.rs:2264-2273`), a RECORDED deviation (ANALYSIS "The projectile
data-row deviations (S1-D-5)"). Correcting the row to 37 before the arm exists
would park the spear in `_ => {}` forever, so this task lands the arm AND the
row correction TOGETHER and retires the ANALYSIS stand-in note. The stale doc
comments at `projectile.rs:301-306` called 186 "the boulder (aiStyle 37)" — the
boulder is kind 99 / aiStyle 25; 186 is the spear trap. Fixed here.

## Port notes (per style)

- **32 BeachBall (155):** `SetDefaults` `:2426-2433` (44×44, aiStyle 32,
  friendly, netImportant). The AI pins timeLeft=10, counts ai[0], and at
  ai[0]≥20 runs THREE bounce scans (255 players / maxNPCs NPCs / 1000
  projectiles) that reset ai[0], kick velocity off the hit entity's
  direction/velocity and bump ai[1]. The tail is the wet/dry drag + gravity.
  Movement is `HandleMovement`'s type-155 arm (`:18603-18618`: the ai[1]>10
  `Game.BallBounceResult` chat, ai[1]=0, the ×−0.6 axis bounces) + the wet-exit
  Y×0.5 (`:16920-16923`). Pool-side step (the projectile scan reads the slot
  array); the chat rides a pool intent drained to the chat monitor.
- **35 rope coils (171/475/505/506):** `SetDefaults` `:2570-2578` (171/505/506)
  / `:5496-5504` (475) — 14×14, aiStyle 35, penetrate −1, tileCollide false,
  timeLeft 400. The AI counts ai[0], at >30 adds 0.2 gravity + ×0.985 X drag +
  14 Y clamp, spins by (|vx|+|vy|)·direction·0.02, then (owner-gated) runs
  `Collision.TileCollision` + the bottomSlope snap and the 10-cell rope-place
  walk (`WorldGen.IsRope` skip, the 314/platform toggle, the tileCut/165 clear,
  `PlaceTile` 213/353/365/366 per kind), then Kill. Takes `&mut World`.
- **37 spear trap (186):** `SetDefaults` `:2728-2740`. The AI latches the
  anchor (Center − velocity·1.5) on the first tick, points rotation at it, and
  reverses velocity on SolidCollision or the 300-px range cap (outbound) /
  kills on SolidCollision or reaching the anchor (return). tileCollide false →
  free integrate. Row corrected from the 185 stand-in (see F3).
- **38 flame jet (187):** `SetDefaults` `:2741-2751` (6×6, aiStyle 38,
  alpha 255, tileCollide false, ignoreWater, timeLeft 60, trap). The AI counts
  ai[0]; at ≥6 resets, plays Item34 and (owner-gated) spawns the 188 FlamesTrap
  with the jet's velocity/damage/knockBack. aiStyle 38 is on `UpdatePosition`'s
  skip list (`:20435`) → NO integrate. The 188 spawn rides `wants_dart_spawns`,
  the cue rides `wants_sound`. The wiring drain's 187 drop is lifted.
- **39 MechanicalPiranha (190):** `SetDefaults` `:2779-2790` (22×22, aiStyle 39,
  friendly, penetrate −1, alpha 255, ranged, usesLocalNPCImmunity cooldown 14).
  The AI fades alpha −50, kills on owner-inactive/dead/2000-px, then the
  channel-release / target-lock / return-to-owner state machine (ai[0]/ai[1]),
  the 16-px target snap + the 190 separation push (8-px sibling band), the
  4-tick frame cycle and the `AI_AdjustPlayerItemRotationToFaceProjectile`
  item-rotation write. Pool-side step (the sibling scan reads the slot array);
  the player writes ride `&mut Player` + the dummy-item-time intent.

## Deliberate skips (recorded in ANALYSIS)

Dust/gore bodies, `Lighting.AddLight`, `NetMessage.SendData` (the netUpdate
flag is set; the MP relay is the net task's), the beach-ball chat's
`NetworkText` localization folds to `lang::text_args`. The piranha's
`SetDummyItemTime(5)` rides the pool's player-write intent (the App-level item
timers are not reachable from the slot loop — the golf `PlayerWrites`
precedent).
