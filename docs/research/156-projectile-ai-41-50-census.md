# 156 — Projectile aiStyles 41-50 dispatch census (W4-d5)

> (Number: was research/26d; renumbered 26d → **156** per the d54
> research/150 collision registry §3 — d12's `26d-projectile-ai-111-120`
> pushed first at 00:06:06, this doc 00:24:13 loses the key.)
> Researched 2026-11-20/21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Cross-checks: f19's coverage census (`26c`, `origin/task/W4-f19-proj-ai-verify-1`)
> rows 41-50 — this lane (d5) owns the decade; d4/d6/g1/g2/g3 probed in-flight
> (zero commits beyond main at probe time).

## Census

| style | C# arm | kinds (SetDefaults) | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 41 | inline 27784-27792 (petal frame latch + spin) | 221 FlowerPetal (`:3066-3078`) | **none** — no arm, no row | **MISSING — ported by this task** |
| 42 | inline 27793-27859 (crystal leaf orbit + shot) | 226 CrystalLeaf (`:3111-3123`) | **none** — no arm, no row; `player.crystalLeaf` unported | **MISSING — ported by this task** |
| 43 | inline 27860-27892 (leaf shot spin + hue) | 227 CrystalLeafShot (`:3124-3136`) | **none** — no arm, no row | **MISSING — ported by this task** |
| 44 | inline 27893-27949 (spore/orb/bolt fade) | 228 SporeCloud (`:3137-3150`), 229 ChlorophyteOrb (`:3151-3163`), 732 ThunderSpearShot (`:7705-7715`) | `projectile_ai_spear::ai_spear_subshot_44` (:552), dispatch 17224 — 228/732 rowed | PORTED — verify-only (229 rowless, F6) |
| 45 | inline 27950-28106 (nimbus/blood clouds + drops) | 237/238/239 (`:3185-3218`), 243/244/245 (`:3255-3288`), 264 RainNimbus (`:3491-3502`) | `ai_rain_cloud_moving` (:8762) + `nimbus_step` (:21773) + `ai_rain_drop` (:8805), dispatch 17784 — 237/238/239 rowed | PORTED — verify-only (blood family + 264 rowless, F5) |
| 46 | inline 28107-28199 (rainbow front/back) | 250 RainbowFront (`:3322-3334`), 251 RainbowBack (`:3335-3350`) | **none** — no arm, no rows | **MISSING — ported by this task** |
| 47 | `AI_047_MagnetSphere` 41403-41444 + `_TryAttacking` 41446-41499 | 254 MagnetSphereBall (`:3369-3377`) | `magnet_sphere_step` (:21382) + `ai_magnet_sphere` (:8444) + dedupe (:21337), dispatch 17771 | PORTED — verify-only (F3) |
| 48 | inline 28204-28242 + `AI_048_GetStartPositionSettingDelay` 53986-54000 | 255 (`:3378-3387`), 260 HeatRay (`:3441-3451`), 290 ShadowBeamHostile (`:3793-3803`), 294 ShadowBeamFriendly (`:3838-3848`), 433 UFOLaser (`:5037-5046`) | `ai_magnet_bolt` (:8508), dispatch 17773 — 255/260/294 rowed | PORTED — verify-only (290/433 rowless, F1) |
| 49 | inline 28243-28322 (bunny flight) + HandleMovement 18833-18860 + Kill 75749-75803 | 281 ExplosiveBunny (`:3676-3687`) | **none** — `CANNON_BUNNY` const only (data.rs:710, anchors the `BunnyCannonCanFire` scan); no row, no arm | **MISSING — ported by this task** |
| 50 | inline 28323-28401 (inferno bolt/blast fade) | 291/292 hostile (`:3804-3825`), 295/296 friendly (`:3849-3869`) | `ai_inferno_fork` (:7877), dispatch 17457 — 295/296 rowed; Kill spawn hooks 74334-74347 wired (`blast_spawned`) | PORTED — verify-only (291/292 rowless, F4) |

## Verify-only notes (ported arms, no behaviour change)

- **44** — the 228 fade (`vel *= 0.96`, `alpha += 2`, kill > 200) and the 732
  fade (`alpha += 12`, kill > 255, rotation) + the shared frame machine are
  verbatim; the 732's `Next(5)` dust gate (`:27913`) and light (`:27920`) ride
  the documented dust/light skips.
- **45** — the moving-cloud fly-to-cursor stop, the raining cloud's ground
  probe / 6-frame sheet / 18000-tick fade / `Next(-14,15)` drop spawn / the
  owner-cap-2 oldest-age-out are verbatim (pool-side `nimbus_step`). The 239
  drop's alpha-50 pin is verbatim.
- **47** — the velocity cache, the signed spin, the >2f damp, the dedupe walk
  and the 8-tick attack cooldown are verbatim.
- **48** — the 5-tile out-of-world kill, the delay-gated centre snapshot and
  the alpha-255 write are verbatim; the 294's reflect movement
  (`HandleMovement` `:18861-18904`) is folded into the arm.
- **50** — the 295's dust-only early return, the 296's `+3` counter and the
  `ai[0] > 540` fade kill are verbatim (the C#'s dust count
  `25 - (ai[0]-180)/2` is ≤ 0 for every `ai[0] > 540`, so the kill tick is
  exact); the first-tick Item74 cue is wired.

## Findings (owed to the wiring/row rounds, NOT fixed here)

- **F1 (style 48)** — `ai_magnet_bolt`'s delay match (:8522-8531) covers
  255→4, 260→3, 294→9, `_ => 0`. The C# helper (`:53988-53998`) also maps
  **290→3**; 290 is rowless so the gap is unreachable today, but the row round
  MUST add the 290 arm (and the 433 keeps the default 0 — correct). The 290's
  `localAI[0] == 1` Item8 cue (`:28229-28232`) is a documented skip.
- **F2 (style 49)** — `ai_style2`'s `162 | 281` ladder arm (projectile.rs
  :2658) mirrors the C#'s **dead** `AI_002` type-281 branch (`:23321-23365`):
  281 carries aiStyle 49 (`:3680`), so `AI_002` never runs for it in C#
  either. Faithful mirror, not a deviation — but the LIVE 49 arm
  (`:28243-28322`) was missing until this task. The bunny's real movement is
  the HandleMovement 281 arm (`:18833-18860`: the slow/`ai[0]==-1` release of
  NPC 614 via `ReleaseNPC`, else the 0.5× bounce) and its real death is the
  Kill 281 arm (`:75749-75803`: gore 76/77 + 4× gore 61-63, the 128×128
  resize + `Damage()`, `ai[0]==-2` → hostile).
- **F3 (style 47)** — recorded in the arm doc: the C#'s
  `Main.rand.Next(num)` target pick (`:41484`) folds to the first candidate
  (single-candidate vanilla parity, ANALYSIS deviation); the 5-frame machine
  (`:41418-41427`) is a documented skip although the slot now carries
  `frame`/`frame_counter` (visual-only, re-derivable). The 20-cap candidate
  array (`:41452-41474`) is unbounded in the port — only matters past 20
  candidates in range.
- **F4 (style 50)** — the hostile 291/292 branches (`:28325-28349`: the
  arrival-dot kill + the 10× dust 174) have no rows; the blast's per-dust
  `Next(-10,11)`×2 + `Next(3,9)` + position `Next(-10,11)`×2 draws
  (`:28385-28399`) are skipped with the dust body (the arm's documented
  zero-observable-state convention). The 291→292 / 295→296 Kill spawns
  (`:74334-74347`, the 296 at 0.75× damage) are wired for the friendly pair.
- **F5 (style 45)** — `ai_rain_drop` folds out the shimmer bounce
  (`:28086-28092`) citing "no shimmer-liquid state on the tile" — STALE: the
  tile now carries `liquid_kind() == Shimmer` (tr-core tile.rs:140). The
  243/244/245 blood-cloud family and the 264 RainNimbus drop (the hostile
  twin, alpha 100 / rotation `+1.57`) are rowless; the 244's cap is **1**
  (`num327 > 1`, `:28072`) vs the 238's 2, and its drop cadence is 10 vs 8
  (`:28030`/`:28039`). Owed to the row round.
- **F6 (style 44)** — the 229 ChlorophyteOrb branch (`:27922-27940`: the
  first-tick `CanHitLine` Item8 cue, `ai[0]++`, the >20 gravity `+0.3`/`×0.98`,
  the 4× dust 157 with `NextVector2Square(-4,4)` — TWO NextFloat per dust)
  is rowless; `ai_spear_subshot_44`'s `_ => 6` fall-through would give it only
  the frame machine. The row round must add the branch with the arm.

## The missing styles — ported by this task (UNWIRED, code-only)

Per the `npc_ai_style25`/`npc_ai_style101` pre-wiring precedent: one pure
module per style, keyed on the vanilla type, with the `SetDefaults` rows as
module consts (NOT `proj_data` rows — adding rows would arm the dispatch's
`_ => {}` fall-through for kinds whose wiring is a later round). The wiring
step adds the rows + the dispatch arms + the player-side `crystalLeaf` /
`crystalLeafCooldown` fields (buff 60's `PetFlag::CrystalLeaf` already lands
in `buff_effects_misc.rs`).

- `projectile_ai_style41.rs` — FlowerPetal 221: the `localAI[0]` frame latch
  (`Next(3)` — the ONE kept draw), the `vel.X * 0.01` spin, the spawn-time
  scale roll (`1 + Next(30) * 0.01`, `:3076`).
- `projectile_ai_style42.rs` — CrystalLeaf 226: the buff-off kill, the
  owner-centre orbit placement (int truncation, gfxOffY, netOffset, the
  gravDir −1 flip + 3.14 rotation), the mouseTextColor scale, the
  local-owner + cooldown gate, the Manhattan-700 `CanHit` scan (INTEGER
  half-box centres, `:27827-27829`), the 12f aim + the
  `GetChaseResults(2160, …)` interception override (≤180 ticks), the 227
  spawn at `Center.X − 4` and the 40-tick cooldown. Damage 100 / KB 10 are
  the `Player.crystalLeafDamage/KB` statics (`Player.cs:57098-57099`).
  RNG-free by construction.
- `projectile_ai_style43.rs` — CrystalLeafShot 227: the `atan2 + 3.14` spin,
  the `soundDelay == 0` one-shot (sound 6 int overload + 8× dust 157 —
  the W4-e3 STAGED emitter `style43_crystal_leaf_burst_227` ACTIVATED on
  this branch as `crystal_leaf_burst_227`: the 8 kept call-site
  `NextFloat` census SUPERSEDED by the pool internals + the POST-spawn
  blend, research/231 row 1's REPLACE contract), the
  `frameCounter++ >= 1` latch (the hue/particle request is rand-free
  render state — skipped), the Kill hook (`:74524-74533`: sound 6 cue
  kept draw-free; the 15× dust-157 bodies + their `NextFloat` blends ride
  the g65 death drain's 227 arm — the g65-Kill-309 compose).
- `projectile_ai_style46.rs` — RainbowFront 250 / RainbowBack 251: the
  shimmer kill (no return — verbatim), the 250's fluff-2 out-of-world kill,
  the owner-gated `localAI[0] > 4 → 3` 251 spawn cadence + the 2400 life cap,
  the rise-weighted `ai[0]` counter, the >30 gravity + signed X friction, the
  `15.95 * scale` velocity renormalization + `atan2 − 1.57` rotation; the
  251's first-tick latch (spriteDirection, rotation, 2400 life), the 0.98
  damp and the 4-way alpha ladder. RNG-free by construction.
- `projectile_ai_style49.rs` — ExplosiveBunny 281: the `ai[0] == -2/-3` death
  states (−2 flips hostile), the 3000-tick Item14 re-arm, the flight branch
  (direction/spriteDirection, the `vel.X*0.05 + dir*0.05` spin, the ≥18
  gravity `+0.28`/`×0.99`, the DOUBLE-compare 15.9 Y cap, the >2 alpha-0, the
  `ai[0]==3` burst's gore `Next(61,64)` + the two NewGore internal kicks —
  kept), the −1 rest branch; the HandleMovement release/bounce arm and the
  Kill hook's full 20-draw gore census + the 128×128 resize + Damage intent.

Witnesses: per-module tests WRITTEN NOT RUN (no compiler/tests on this
branch, the task contract); verified by re-reading the cited C# blocks and
hand arithmetic.

## The RETIRE round (2026-11-21, research/360)

The band's landing census, the style-43 STAGED-emitter activation, the
all-styles-armed/kinds-rowed sweep, the g65-Kill-309 compose and the FINAL
band declaration live in `360-projectile-ai-41-50-final.md` (this doc's
companion; number per registry-150 §8.7's new-doc rule, ≥360 after the
360-ref check). This census doc's rows stay as the port-time record.