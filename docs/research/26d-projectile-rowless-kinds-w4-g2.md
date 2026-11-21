# 26d — the rowless proj kinds of d5's F1-F6 (W4-g2 lane b, coding round)

> Coded 2026-11-21 on `task/W4-g2-proj-ai-missing-b` (census `d0708e3b`),
> against the d5 census doc (`26d-projectile-ai-41-50-census.md`,
> `origin/task/W4-d5-proj-ai-41-50`, tip `e46d9eb7`). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` /
> `projectile_ai_spear.rs` at this branch's tip. Code-only mode: no
> cargo/tests/graphify invoked; the witnesses are WRITTEN NOT RUN, verified
> by re-reading the cited C# blocks and hand arithmetic.

## Scope — the nine rowless kinds with LIVE AI arms

d5's findings F1/F4/F5/F6 named nine kinds whose aiStyle arms were ported
but whose `proj_data` rows were missing — `spawn_with_data`'s leading
`proj_data(kind)?` guard silently declined every spawn. Co-claim check
first: d5/d6/d8's pushed branches carry NO rows for these kinds (d8's
`7953d151` rows 410/419-422/434/441/453 — disjoint; d5's own commits are
UNWIRED style modules with module consts, explicitly NOT rows). g1/g3 sit
at the census base with no commits.

| kind | name | style | SetDefaults | d5 finding |
| --- | --- | --- | --- | --- |
| 229 | ChlorophyteOrb | 44 | `:3151-3163` | F6 |
| 243 | BloodCloudMoving | 45 | `:3255-3262` | F5 |
| 244 | BloodCloudRaining | 45 | `:3263-3273` | F5 |
| 245 | BloodRain | 45 | `:3274-3288` | F5 |
| 264 | RainNimbus | 45 | `:3491-3502` | F5 |
| 290 | ShadowBeamHostile | 48 | `:3793-3803` | F1 |
| 291 | InfernoHostileBolt | 50 | `:3804-3814` | F4 |
| 292 | InfernoHostileBlast | 50 | `:3815-3825` | F4 |
| 433 | UFOLaser | 48 | `:5037-5046` | F1 |

## The rows (verbatim, per-field)

Every row carries the C# arm's writes plus the `SetDefaults` reset for the
fields it leaves alone (`:770-799`: penetrate 1, tileCollide true, alpha 0,
scale 1, timeLeft 3600, friendly false — the port's spawn seam folds
friendly per caller). The non-obvious deltas from the friendly siblings:

- **229** keeps the stock `tileCollide = true` (the 228 cloud's false does
  NOT carry over) — the orb dies on tile contact. `light = 0.2` (`:3159`),
  melee (`:3157`), idStatic-10 (`:3161-3162`), scale 1 (NOT the 228's 1.1).
- **245** penetrate **2** (`:3281`) — the 239's 5 does not carry over.
  magic (`:3284`), idStatic-10, ignoreWater, extraUpdates 1, scale 1.1.
- **264** hostile, penetrate -1, timeLeft **120**, NO class flag, NO
  immunity pair — the arm pins rotation, not alpha.
- **292** box **130x130** (`:3817-3818`) — the friendly 296's 150 does not
  carry over.
- **433** keeps the stock penetrate **1** (`:774` — the arm sets none): the
  laser dies on its first hit, unlike the beams' -1. ignoreWater (`:5045`).
  `Sets.MinionShot` already lists 433 (`ProjectileID.cs:431`) — the whip-tag
  gate rides the existing table, no change.
- netImportant: 243/244 (`:3257`/`:3265`); ignoreWater: 244/245/264/433
  (`:3267`/`:3276`/`:3493`/`:5045`) — the moving cloud 243 keeps false.
- projFrames: 229→4, 243→4, 244→6 (`Main.cs:8606-8609`); the rest stock 1.

## The arms

- **F1 (style 48)** — `ai_magnet_bolt`'s delay match now maps
  `HEAT_RAY | SHADOWBEAM_HOSTILE => 3` (`:53990-53992`); the 433 keeps the
  default 0 (`:53997-53998`) — correct, NOT added. The 290's one-shot Item8
  cue at `localAI[0] == 1` POST-increment (`:28229-28232`) rides
  `wants_sound` (d5 recorded it skipped; the S1-E-21h sound layer has since
  landed). The 290 joins the 294's HandleMovement reflect — the C# gate is
  `type == 290 || type == 294` (`:18861`); the 290's broadcast gate
  (`:18873-18878`) is the netMode fold (single-player keeps it on; the
  broadcast itself stays the documented render skip).
- **F4 (style 50)** — `ai_inferno_fork` gains the hostile 291 branch
  (`:28325-28348`): the first-tick Item20 cue + latch, the zero-target
  guard (`vector30 != Vector2.Zero`), the arrival dot
  `Dot(target - Center, velocity) <= 0` → `velocity = Zero` + owner-gated
  Kill; the 10x dust-174 body is dust-only (skipped, no draws); the branch
  returns — but the C# return exits AI() only, so the free move still
  integrates (`kill_step`'s tileCollide-false path). The 292 rides the
  shared Item74/fade branch with **+1** per tick — the `+3` is gated on
  `type == 296` alone (`:28367-28370`), so the hostile blast's fade kill
  lands at tick 541, not 136. The Kill hook generalizes: 291→292 with the
  FULL damage (`:74338` — no 0.75 line), owner preserved
  (`spawn_hostile_ai_owned` owner 0, the `NPC.cs:21219` convention); the
  friendly 295→296 keeps its `(int)(damage * 0.75)`.
- **F5 (style 45)** — the dispatch's kind gates widen to the C#'s own:
  `237|243` → `ai_rain_cloud_moving`, `238|244` → `nimbus_step`,
  `239|245|264` → `ai_rain_drop`. `nimbus_step` takes the 244's cadence 10
  (`:28030`), owner cap 1 (`:28072`), per-kind count scan (`num330 =
  this.type`, `:28057`) and the aged-out `netUpdate` (`:28074`/`:28080` —
  the 238 path gains the sync write it was missing). `ai_rain_drop` pins
  alpha 100 for the 245 (`:28099`) and rotation `atan2(vy, vx) + 1.57` for
  the 264 (`:28103`), and the shimmer bounce (`:28086-28092`) is now LIVE —
  d5's fold-out cited "no shimmer-liquid state on the tile", STALE: the
  tile carries `liquid_kind() == Shimmer` (tr-core tile.rs:140). The Kill
  hook 243→244 (`:74539-74542`) mirrors the 237→238.
- **F6 (style 44)** — `ai_spear_subshot_44` gains the 229 branch
  (`:27922-27940`): the first-tick `CanHitLine(Center, 1, 1, Center +
  velocity * 10, 1, 1)` Item8 cue via `wants_sound` (the int overload,
  style 0), `ai[0]++`, the >20 gravity `+0.3` / X friction `*0.98`; the 4x
  dust-157 `NextVector2Square(-4, 4)` draws are dust-only — skipped with
  their draws (the module's zero-observable-state convention). The
  signature widens to `(p, world)` for the line test. The Kill arm
  (`:74450-74474`) lands as `chlorophyte_orb_kill`: the Item167 death cue
  (every owner — it precedes the owner gate) pushed straight into
  `pending_proj_sounds` (the death hook runs AFTER the sub-step loop's
  `wants_sound` drain — the golf `tick.sounds` precedent), and the
  owner-gated 32-child 1128 burst ported VERBATIM with all 64 `Main.rand`
  draws KEPT in order (X `Next(-25, 26)` then Y `Next(-45, 46)` per child,
  `num533` 0..2 x `num534` 0..16, the `0.35 + 0.125 * (num535 + 1)` velocity
  weight). The 1128 (ChlorophyteClaymoreSporeCloud, aiStyle 92,
  `:10565-10579`) is ROWLESS — another family, out of this task's scope:
  the spawn declines at the `proj_data` guard until its row lands, but the
  draws stay on the UpdateProjectiles stream so the alignment holds. The
  25x dust-157 tail draws nothing (the plain NewDust overload) — skipped.

## Recorded gaps (NOT fixed here — sibling rows outside the task's kinds)

- The 228/732 rows keep `melee = false` though their C# arms set it
  (`:3144`/`:7713`); the 239/294 rows keep `magic = false` though theirs
  set it (`:3214`/`:3844`). Pre-existing census misses on ROWED kinds —
  the row round's scope is the rowless nine; fixing siblings would widen
  the diff beyond the claim. Noted for the reconciliation round.
- The 229's `GetCollisionParams` -20 collision shrink (`:19977`, shared
  with 182/190/33/237/243/866) is unported port-wide (the 237 sibling
  rides the same gap) — the damage box stays the full row box.
- The 229's `Shimmer()` kill/bounce arm (`:22403-22422`, the
  `shimmerWet`-gated family) is unported port-wide (the generic wet block
  has no shimmer hook yet) — the orb's shimmer interaction is a follow-up.
- The 1128 child row + its aiStyle-92 arm (`:32558-32820`) — the orb's
  death burst spawns nothing until it lands (the draws are kept).

## Witnesses (WRITTEN NOT RUN)

- `data.rs::the_w4_g2_rowless_rows_are_verbatim` — the nine-row census:
  box/style/penetrate/life/extra/collide/alpha/scale + the immunity pairs +
  class flags + netImportant/ignoreWater + light + projFrames.
- `projectile.rs::w4_g2_rowless_tests` — 13 tests: the 290's delay-3
  snapshot + one Item8 cue + the one-tick life shape (timeLeft 100 vs 101
  sub-steps — the arm runs 100x, the 101st skips the dead slot), the 433's
  delay-0 + stock penetrate 1, the 290 corridor reflect (life lifted to
  300 for the walk; the row's 100 is pinned by the data witness), the
  291's arrival-dot kill + zero-target guard + the 292 Kill spawn at FULL
  damage/130-box/owner-0, the 292's +1 counter + the 541-tick fade
  boundary, the 295's 0.75 regression guard, the 243→244 settle chain, the
  244's 10-cadence rain + cap-1 netUpdate age-out, the 264's LIVE shimmer
  bounce + rotation pin, the 229's first-tick cue + >20 gravity/friction,
  and the 229 Kill arm's Item167 cue + the 64-draw X-then-Y census with
  the rowless-1128 decline.

## Commits (this round, on `task/W4-g2-proj-ai-missing-b`)

1. `af472214` — rows (4): 290/291/292/433 + magic census + consts.
2. `ee857a36` — rows (5): 229/243/244/245/264 + melee/magic/light/frames.
3. `8d3cdcb1` — F1+F4 arms: the delay table, the 290 cue/reflect, the 291
   arrival branch, the 292 +1 fade, the generalized Kill hook.
4. `da1cc410` — F5+F6 arms: the Blood Rain chain, the live shimmer bounce,
   the 229 orb branch + Kill arm.
5. `c88af9dc` — the data witness + the 291 free-move fidelity fix.
6. `daea150f` — the 13 AI witnesses.
7. (this doc)