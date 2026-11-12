# 31 — Golf / ball physics

> Wave 3 · researched 2026-11-12 · sources: decompiled (`Terraria/GameContent/Golf/*`, `Terraria/Physics/*`) + crates (`tr-app`, `tr-core`) + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The 1.4.5.8 golf subsystem is **not** one class: it is a small namespace plus the
generic `Terraria.Physics` ball engine and a dozen scattered call sites.

| C# file | lines | responsibility |
| --- | --- | --- |
| `decompiled/Terraria/GameContent/Golf/GolfHelper.cs` | 439 | static entry points: shot strength, club table, ball hit, prediction line, contact listener (bounce/friction/hole capture) |
| `decompiled/Terraria/GameContent/Golf/GolfState.cs` | 145 | per-client score state: score timer, hit records, camera tracking, ball return |
| `decompiled/Terraria/GameContent/Golf/GolfBallTrackRecord.cs` | 28 | per-ball hit history → distance-based score |
| `decompiled/Terraria/GameContent/Golf/FancyGolfPredictionLine.cs` | 127 | the aim-guide ribbon (20 simulated steps of the ball physics) |
| `decompiled/Terraria/GameContent/Metadata/TileGolfPhysics.cs` | 15 | per-tile-material golf dampening block (5 floats) |
| `decompiled/Terraria/Physics/BallCollision.cs` | 368 | the generic ball physics engine (sub-stepped integration, circle-vs-tile edge collision, reflection, angular velocity) |
| `decompiled/Terraria/Physics/{BallCollisionEvent,BallPassThroughEvent,BallPassThroughType,BallState,BallStepResult,IBallContactListener,PhysicsProperties}.cs` | 10/8/9/7/21/7/11 | the engine's data carriers |
| `decompiled/Terraria/Physics/CollisionVisualizer.cs` | 73 | debug drawing of the collision edges (DEBUG-only) |

Supporting call sites: `Projectile.AI_149_GolfBall` (`Projectile.cs:20331-20374`),
`Projectile.AI_150_GolfClubHelper` (`Projectile.cs:52417-52490`),
`Player.TryPlacingAGolfBallNearANearbyTee` (`Player.cs:49677-49737`),
`Player.GetPreferredGolfBallToUse` (`Player.cs:49739-49764`),
`Player.AccumulateGolfingScore` (`Player.cs:4932-4940`),
`Item.DefaultToGolfBall` (`Item.cs:35444-35460`) / `DefaultToGolfClub` (`Item.cs:48515-48527`) /
`IsAGolfingItem` (`Item.cs:47952-47962`),
`Main.DrawInterface_Resources_GolfPower` (`Main.cs:43515-43546`),
`Main.DrawInterface_GolfBallIndicator` (`Main.cs:43548-43582`),
`Main.DrawMapIcons_LastGolfballHit` (`Main.cs:56515+`),
`Chest.cs:3156-3262` (Golfer shop, score-gated).

**There is no `GolfCourse`, `GolfWorld` or `GolfData` class in 1.4.5.8** — no course
table, no hole list, no par values. The "course" is plain world tiles
(`TileID.GolfHole` 476, `GolfGrass` 477, `GolfGrassHallowed` 492, cup flag 493, tee 494 —
`TileID.cs:1581-1613`) plus the per-material physics constants. The only
course-level numbers are the score tiers 500 / 1000 / 2000
(`GolfHelper.cs:255-259`, `GolfHelper.PointsNeededForLevel1/2/3`).

## Key entities

C# (`Terraria.GameContent.Golf` / `Terraria.Physics`):
- `Terraria/GameContent/Golf/GolfHelper.cs::GolfHelper` (L12) — static façade; holds the
  shared `PhysicsProperties` (gravity 0.3, drag 0.99 — `GolfHelper.cs:471`) and the
  `ContactListener` (L32).
- `GolfHelper.cs::ContactListener.OnCollision` (L34-82) — bounce model: per-tile
  side/direct impact dampening (`GolfHelper.cs:37`), bumper tiles 421/422 push the ball
  (L41-56), **tile 476 (the hole) captures the ball** when the normal points up and the
  speed ≤ 100 (L57-69).
- `GolfHelper.cs::ContactListener.PutBallInCup` (L84-113) — the hole-capture handler:
  local-player gate, `GolfState.SetScoreTime` when hits > 1, `RecordBallInfo`, `LandBall`,
  `GetGolfBallScore`, `Player.AccumulateGolfingScore`, chat text, `ResetScoreTime`,
  `Wiring.HitSwitch`, `NetMessage.SendData(59)` + `(128)`, then `proj.Kill()`.
- `GolfHelper.cs::ContactListener.OnPassThrough` (L125-147) — liquid/tile drag:
  water ×0.91, honey ×0.8, tile ×`PassThroughDampening`, lava no-op.
- `GolfHelper.cs::CalculateShotStrength(Vector2, ClubProperties)` (L281-289) — the shot
  model: `FindVectorOnOval` projects the aim vector onto the club's min/max strength
  ovals; `AbsoluteStrength = clampedLength * 32`,
  `RelativeStrength = max((len-min)/(max-min), 0.001)`.
- `GolfHelper.cs::GetClubProperties` (L331-354) — the 12-entry club table (item ids
  4039, 4092-4094, 4587-4598) → min/max ovals + `RoughLandResistance` (wedge = 1).
- `GolfHelper.cs::HitGolfBall` (L434-457) — the strike: club-impact dampening per tile
  material, `timeLeft = 18000`, `ai[1]++` (the hit counter), `localAI[1] = 1`,
  `GolfState.RecordSwing`.
- `GolfHelper.cs::StepGolfBall` (L267-270) → `Terraria/Physics/BallCollision.cs::Step`
  (L24-89) — the integration: drag, speed clamp 1000, `num4 = ceil(speed/2)` sub-steps,
  gravity divided by `num4²`, `position += velocity`, circle↔tile-edge closest-point
  collision, `Vector2.Reflect`, angular velocity from the contact normal, `Resting`
  when |vx| < 0.01 and −gravity < vy ≤ 0 after a contact.
- `GolfHelper.cs::ValidateShot` (L406-432) / `IsGolfShotValid` (L392-404) — the aim
  legality box around the player (`-16 ≤ x ≤ 32`, `-16 ≤ y ≤ 16`) and the shot-angle
  clamp to `[−1.5208 rad, 0]`.
- `GolfHelper.cs::DrawPredictionLine` (L459-467) → `FancyGolfPredictionLine.cs::Update`
  (L38-56) — 20 simulated `StepGolfBall` steps with tile 379 temporarily non-solid.
- `GolfState.cs::GolfState` (L6-173) — the per-client state: `golfScoreTime` (max 3600,
  delay 90), `ScoreAdjustment = time/max`, `ShouldScoreHole`, `_hitRecords[1000]`,
  `RecordSwing`, `GetLastHitBall`, `RecordBallInfo`, `LandBall`, `GetGolfBallScore`,
  `ResetGolfBall` (the "return the ball to the swing point" penalty), `WorldClear`,
  `TryGetCameraTrackingPosition`.
- `GolfBallTrackRecord.cs::GetAccumulatedScore` (L15-19) — score =
  `(int)(totalDistance / 16) / (hitsMade + 2)` (distance-based, **not** par-based).
- `Terraria/Physics/PhysicsProperties.cs` (L11) — the two constants (gravity, drag).

Rust:
- `crates/tr-app/src/angler.rs::accumulate_golfing_score` (L572) — the **only** ported
  piece of the golf system (`Player.cs:4932-4940`, 1e9 ceiling). Its own doc comment
  (L568-571) states: *"the only ported piece of the golf system (the balls/cups/scoring
  ride M4-GOLF-1)"*.
- `crates/tr-app/src/player.rs::Player.golfer_score` (L341-343, init L1165) — the
  accumulated score field, never fed by anything.
- `crates/tr-app/src/shop.rs::ShopFlags.golfer_score` (L36-39, init L83),
  `StockWhen::GolfScore(i32, bool)` (L195-198), `StockWhen::GolfPainting(u8)` (L192-194),
  the Golfer shop rows (L548-567) — the gated stock is faithful, but the score is 0.
- `crates/tr-app/src/itemdata.rs` — the golf item rows exist (3989 ball → projectile 721;
  4039/4092-4094/4587-4598 clubs → 722; 4040 cup → tile 476; 4083-4088 flags → 493;
  4089 tee → 494; 4095 whistle), plus `crates/tr-app/src/creative_sacrifices.rs` (L2344+).

## Dependencies

- Calls into (C#): `Terraria.Physics.BallCollision.Step`, `TileMaterials.GetByTileId`
  (→ `TileMaterial.GolfPhysics`, `TileGolfPhysics.cs`), `Main.tileSolid`/`tileSolidTop`,
  `Framing.GetTileSafely`, `Collision.SwitchTiles`, `Wiring.HitSwitch`,
  `NetMessage.SendData(59/128/27)`, `Dust.NewDustPerfect`, `SoundEngine.PlaySound`,
  `ChatHelper.DisplayMessage`, `ProjectileID.Sets.IsAGolfBall`
  (`ProjectileID.cs:428` → {721, 739-752}), `Player.AccumulateGolfingScore`,
  `Main.LocalGolfState`, `Main.MouseWorld`, `TextureAssets.GolfSwingBar*` /
  `GolfBallArrow*` / `GolfBallOutline` / `Extra[33]`.
- Called by (C#): `Main.Update` (`Main.cs:17219` → `GolfState.Update`),
  `Main.DrawInterface*` (`Main.cs:46200-46201`), `Main.DrawMapIcons_LastGolfballHit`,
  `Main` camera tracking (`Main.cs:62741`), `Projectile.AI_149_GolfBall` /
  `AI_150_GolfClubHelper`, `Player` (tee placement, reset, equip-ball), `Chest.cs`
  (Golfer shop tiers), `WorldGen.CheckGolf1x1` (`WorldGen.cs:45260`) and the
  `TileID.Conversion.GolfGrass` chain (`TileID.cs:87`, `WorldGen.cs:55224/55501/55854/56160`).
- Rust callers of the ported piece: `shop.rs` reads `ShopFlags.golfer_score`
  (populated from `Player.golfer_score`), `npc.rs`/`world_sim/npcs.rs` read it for the
  Golfer's chatter tier (`NPC.cs:94792`) and the shop index 22.
- Shared data structures: `TileMaterial`/`TileGolfPhysics` (C#-only today), the
  projectile 721/722 pair, tiles 476/477/492/493/494, `GolfState._hitRecords`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `GolfHelper.StepGolfBall` → `BallCollision.Step` (integration, drag, gravity, reflection) | **MISSING** | — | No ball physics anywhere in `crates/`: grep for `BallCollision|PhysicsProperties|ball_step|BallStep|restitution|angular_velocity` over all `crates/**/*.rs` returns **zero** hits; no `crates/tr-core/src/ball*` file. C#: `BallCollision.cs:24-89` (sub-steps `ceil(speed/2)`, gravity/num4², `Vector2.Reflect`, angular velocity) |
| Ball physics constants (gravity 0.3, drag 0.99) | **MISSING** | — | `GolfHelper.cs:471` `new PhysicsProperties(0.3f, 0.99f)`; ROADMAP:728-729 records the same numbers, but no Rust constant exists |
| Per-tile golf dampening (`TileGolfPhysics`: Direct/Side/Club impact, PassThrough, ImpactDampeningResistanceEfficiency) | **MISSING** | — | `TileGolfPhysics.cs:5-20`; no Rust `TileMaterial` equivalent exists (grep `TileMaterial|tile_material` over `crates/` → 0 files). Values come from the game's TileMaterial content JSON; `assets/` contains no `*TileMaterial*` file, so the numbers are not recoverable from the repo as-is |
| Contact listener: bounce, friction, bumper tiles 421/422 | **MISSING** | — | `GolfHelper.cs:34-82`; no Rust contact listener / `OnCollision` / `OnPassThrough` |
| Hole capture (tile 476 → `PutBallInCup` → score + wiring + net) | **MISSING** | — | `GolfHelper.cs:57-69` (capture test) and `:84-113` (handler) |
| `GolfState` (score timer, hit records, camera tracking, ball return) | **MISSING** | — | `GolfState.cs:6-173`; no `golf_state` / `GolfState` symbol in `crates/` |
| `GolfBallTrackRecord.GetAccumulatedScore` (distance/hits score) | **MISSING** | — | `GolfBallTrackRecord.cs:15-19` |
| Shot strength model (`CalculateShotStrength`, `FindVectorOnOval`, `ShotStrength`) | **MISSING** | — | `GolfHelper.cs:272-289` |
| Club table (`GetClubProperties`, 12 clubs) | **MISSING** | — | `GolfHelper.cs:331-354`; the item rows exist in `itemdata.rs` (L4160/4213-4215/4708-4720) but no club→strength mapping |
| `HitGolfBall` (strike, hit counter, `timeLeft = 18000`) | **MISSING** | — | `GolfHelper.cs:434-457` |
| `IsPlayerHoldingClub` / `Item.IsAGolfingItem` | **MISSING** | — | `GolfHelper.cs:291-303`, `Item.cs:47952-47962`; no Rust `is_golfing_item` helper |
| Projectile 721 ball AI (`AI_149_GolfBall`: rest on tee tile 494, resting state, plate detection, trail rotation) | **MISSING** | — | `Projectile.cs:20331-20374`; `proj_data` has no rows for 721/739-752 |
| Projectile 722 club-helper AI (`AI_150_GolfClubHelper`: aim, release, shoot, 500×500 ball search) | **MISSING** | — | `Projectile.cs:52417-52490`; no Rust row for 722 |
| Tee placement (`TryPlacingAGolfBallNearANearbyTee`, `GetPreferredGolfBallToUse`) | **MISSING** | — | `Player.cs:49677-49764` |
| Prediction line (aim ribbon, 20 steps) | **MISSING** | — | `FancyGolfPredictionLine.cs`; no Rust equivalent, and it needs the physics engine first |
| Golf UI: swing-power bar (`DrawInterface_Resources_GolfPower`) | **MISSING** | — | `Main.cs:43515-43546`; no golf code in `crates/tr-app/src/ui/**` (grep `golf|swing_bar` → only bestiary/housing name strings) |
| Golf UI: ball indicator arrow (`DrawInterface_GolfBallIndicator`) | **MISSING** | — | `Main.cs:43548-43582` |
| Golf UI: map icon for the last hit ball | **MISSING** | — | `Main.cs:56515+` |
| Player accumulated score (`AccumulateGolfingScore`) | **PORTED** | `crates/tr-app/src/angler.rs::accumulate_golfing_score` (L572) | Faithful: 1e9 ceiling, C# `Player.cs:4932-4940`; test `the_golf_score_clamps_at_a_billion` (L403-408) |
| Golfer score plumbing (field, shop gate, chatter tier) | **PORTED but inert** | `crates/tr-app/src/player.rs::Player.golfer_score` (L343), `crates/tr-app/src/shop.rs::ShopFlags.golfer_score` (L39), `StockWhen::GolfScore`/`GolfPainting` | `shop.rs:36-39` documents "the port has no golf system yet, so it [the score] …" — the value is always 0, so the 500/1000/2000 tiers never unlock |
| Golf items / tiles (items 3989, 4039-4095, 4587-4601; tiles 476/477/492/493/494) | **PARTIAL** | `crates/tr-app/src/itemdata.rs`, `crates/tr-app/src/creative_sacrifices.rs` | The item/tile *rows* exist, but tile 476/477/492/493/494 do not appear in `crates/tr-core/src/tiledata/**` (grep → 0 hits), so the hole/tee/grass are not modelled |

## Tests

- `crates/tr-app/src/angler.rs::tests::the_golf_score_clamps_at_a_billion` (L403-408) —
  the only golf-named test in the workspace; covers the 1e9 clamp only.
- The Golfer shop tiers (500/1000/2000 + painting pairs) are covered by the M4-TOWN-18
  shop tests in `crates/tr-app/src/shop.rs`'s test module (per ROADMAP:1422, "the shop's
  always rows + the score tiers + the painting pairs"); they exercise the *gate*, never
  a real score.
- **No test exists for**: ball integration, friction/bounce, hole capture, shot
  strength, club ovals, the prediction line, the golf UI, or the `GolfState` timer.

## Deviations & risks

1. **The entire golf subsystem is absent from the Rust port** — severity: **behavioural**
   (feature-level). Nothing in `crates/` implements the ball physics, the score state,
   the hole capture, the shot model or the golf UI. ROADMAP/ANALYSIS admit this:
   ROADMAP:1658 lists `M7-GOLF-1 … golf (parked)`, ROADMAP:1138 routes the golf files
   to `M7-GOLF-1`, and ANALYSIS "Golfer (M4-TOWN-18)" states the score "stays at their
   score-0 tier until M4-GOLF-1 ports the balls, cups and `GolfState` scoring".
2. **The Golfer's shop tiers are permanently locked** — severity: **behavioural**
   (documented). `shop.rs:36-39` / `angler.rs:568-571` say so explicitly: nothing ever
   calls `accumulate_golfing_score`, so `golfer_score` stays 0 and the 500/1000/2000
   stock rows plus the 4 paintings are unreachable. The *gate logic* itself is faithful
   (mixed `>=`/`>` per row is preserved by `StockWhen::GolfScore(min, inclusive)`).
3. **Holding a club does nothing** — severity: **behavioural**. The clubs are in
   `itemdata.rs` with `use_style: 8` (GolfPlay) and `projectile: Some(722)`, but there is
   no projectile 722, so the swing never fires. `item.rs:54` mentions GolfPlay only in
   the constant list.
4. **No `TileMaterial`/`TileGolfPhysics` in Rust** — severity: **behavioural + data**.
   Even a faithful port of `BallCollision` needs the per-material dampening constants;
   `assets/` has no `*TileMaterial*` JSON, so those five floats per material must be
   sourced from the retail content (or hard-coded from the xnb) at port time. **Risk:**
   the port cannot be verified 1:1 without that data.
5. **The golf tiles are absent from `tiledata`** — severity: **behavioural**. Tiles
   476/477/492/493/494 do not appear in `crates/tr-core/src/tiledata/**`; the tee/cup/
   grass cannot be placed or drawn by the port even though the *items* that place them
   are registered.
6. **"Course/par data" does not exist in 1.4.5.8** — severity: **none (informational)**.
   There is no `GolfCourse`/`GolfWorld`/`GolfData` class and no par table; scoring is
   distance-based (`GolfBallTrackRecord.cs:15-19`). Any plan asking for a "hole/par
   table" is porting a feature the vanilla does not have.
7. **The course geometry has no identified generator** — severity: **medium risk**.
   `WorldGen.cs` contains only `CheckGolf1x1` (`:45260`), `Main.LocalGolfState.WorldClear`
   (`:7001`) and the `TileID.Conversion.GolfGrass` (477/492) conversion arms
   (`:55224/55501/55854/56160`); no pass builds a course. A port must therefore either
   ship the course as data or find the placement path.
8. **No 3D shot** — severity: **none (informational)**. The C# shot is 2D (mouse →
  `Vector2` aim + a strength oval); the "3D-ish" phrasing in the task brief has no
   counterpart in 1.4.5.8.

## Task seeds

- **S** — Extract the golf material data: locate the TileMaterial/TileGolfPhysics values
  (5 floats × N materials) from the retail content and land them as a Rust table
  (new `crates/tr-core/src/golf/materials.rs`). *Blocks everything below.*
- **M** — Port the generic ball engine (`Terraria/Physics/BallCollision.cs:24-89`) to
  `crates/tr-core/src/ball.rs`: sub-stepping (`ceil(speed/2)`), gravity/num4², drag,
  the 1000 speed clamp, circle↔tile-edge closest point, `Vector2.Reflect`, angular
  velocity, the `Moving`/`Resting`/`OutOfBounds` states. Test against the C# constants.
- **M** — Port `GolfHelper` (shot strength + club table + `HitGolfBall` + the contact
  listener incl. bumper tiles 421/422 and the hole test for 476) to
  `crates/tr-app/src/golf.rs`.
- **M** — Port `GolfState` + `GolfBallTrackRecord` (score timer 3600/90, `ScoreAdjustment`,
  `_hitRecords[1000]`, `GetGolfBallScore`, `ResetGolfBall`, camera tracking) and wire
  `Player.golfer_score` ← `accumulate_golfing_score` so the Golfer shop tiers unlock.
- **M** — Port the two projectile AIs: 721 ball (`AI_149_GolfBall`: tee-494 rest check,
  resting state, plate detection, trail rotation) and 722 club helper
  (`AI_150_GolfClubHelper`: 500×500 ball search, aim clamp, release, shoot), plus the
  `proj_data` rows for 721/739-752 and 722.
- **S** — Port `Player.TryPlacingAGolfBallNearANearbyTee` +
  `GetPreferredGolfBallToUse` (tee scan ±2 tiles, nearest-to-mouse tee, kill old balls).
- **S** — Add the golf tiles (476/477/492/493/494) to `tiledata` + the
  `TileID.Conversion.GolfGrass` conversion chain.
- **M** — Port the golf UI: `DrawInterface_Resources_GolfPower` (54 × relativeStrength
  fill) and `DrawInterface_GolfBallIndicator` (the bobbing arrow to the resting ball) in
  `crates/tr-app/src/ui/`.
- **M** — Port `FancyGolfPredictionLine` (20 simulated steps, tile 379 temporarily
  non-solid) — depends on the ball engine and the material table.
- **S** — Netcode: `NetMessage` 59 (tile hit), 128 (golf score), 27 (ball shot) and the
  `Wiring.HitSwitch` cup press.
- **S** — Chat/effects: `Game.BallBounceResultGolf_Single/Plural` text and
  `EmitGolfballExplosion` (the dust burst) — depends on the localization port.
