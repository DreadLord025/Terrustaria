# 100 — DOC-SWEEP lane 21: actuation / inactive tiles

> W4-g80 DOC-SWEEP phase · 2026-11-21 · code-only lane (no cargo/tests/graphify)
> Branch `task/W4-g80-tile-actuator-draw`, continuing af6cd44f (the actuator DRAW census).
> Sources: `decompiled/Terraria/{Tile,WorldGen,Collision,Wiring,Player,Item,Projectile}.cs`,
> `decompiled/Terraria/GameContent/Drawing/{TileDrawing,WallDrawing}.cs`,
> `decompiled/Terraria/ID/{ItemID,TileID}.cs` vs `crates/`.

Scope: the five lane-21 questions — (1) actuated-tile BEHAVIOUR census,
(2) the actuator item's PLACE side, (3) the actuation visual transition,
(4) the next unclaimed parked arm from research/09, (5) inactive walls.
Verdicts below; the one ported gap is the 83 herb-maturity sheet swap (§4).

## 0. The state bits (Tile.cs)

| C# | Bits | Rust | Status |
| --- | --- | --- | --- |
| `Tile.active()` | `sTileHeader & 0x20` | `FLAG_ACTIVE` (`tile.rs:3`) | PORTED |
| `Tile.inActive()` (`Tile.cs:606-621`) | `sTileHeader & 0x40` — the framed-off flag; the tile stays `active`, renders ghosted, collides as air | `FLAG_INACTIVE` (`tile.rs:6`), `Tile::inactive()` (`tile.rs:82-84`) | PORTED |
| `Tile.nactive()` (`Tile.cs:240-247`) | `(sTileHeader & 0x60) == 0x20` — active AND NOT inActive | `t.active() && !t.inactive()` (the `nactive` helpers, e.g. `npc_ai_fighter_scan.rs`) | PORTED |
| `Tile.actuator()` (`Tile.cs:691-706`) | `sTileHeader & 0x800` — the actuator DEVICE bit, independent of inActive | `FLAG_ACTUATOR` (`tile.rs:5`), `Tile::actuator()` (`tile.rs:102-104`) | PORTED |
| `Tile.actColor(Color)` (`Tile.cs:272-280`) | double `0.4` per-channel truncation, alpha kept | `drawdata::act_color_bytes` (af6cd44f) | PORTED |
| `Tile.actColor(ref Vector3)` (`Tile.cs:282-288`) | f32 `*= 0.4f` | `tiles.rs:617-619` + `sliced_light` | PORTED |

Persistence: the .wld loader reads both bits (`wld/mod.rs:598-601`) and the
writer emits them (`wld/sections/tiles.rs:163-167`, `WorldFile.cs:1521-1528`
b3; the header round-trip `HEADER_FLAGS` `:301`). PORTED.

There is NO actuator TILE id — the actuator is item 849 only
(`ItemID.cs:3381` `Actuator = 849`); the device rides the tile's
`sTileHeader` bit. (`wiring.rs:388`'s "138 is the actuator itself" comment
is a mislabel — 138 is the Boulder tile in `TileID.Sets.Boulders`; the set
membership itself is correct.)

## 1. Actuated-tile BEHAVIOUR census

### 1a. The toggle (Wiring.cs) — PORTED

- `Wiring.Actuate` (`Wiring.cs:389-405`): requires the actuator bit, flips
  via `ReActive`/`DeActive` → `wiring.rs::actuate` (`:534-547`). PORTED.
- `Wiring.ActuateForced` (`:407-417`): no actuator-bit gate (the wire-signal
  path) → `wiring.rs::actuate_forced` (`:2949-2958`). PORTED.
- `HitWireSingle` (`:996-1009`): `if (tile.actuator()) ActuateForced(i, j)`
  runs BEFORE the `!active()` return and the device switch →
  `wiring.rs::hit_wire_single` (`:2172`, the call at `:2193`). PORTED.
- `Wiring.DeActive` (`:3195-3223`): the guards — `active`; the 226 Plantera
  bulb below the surface is untouchable before the plant boss; genuinely
  solid (`tileSolid && !NotReallySolid`); the forced-false set
  {314, 379, 386, 387, 388, 389, 476}; the above-cell rule
  (`!above.active() || (!PreventsActuationUnder[above] && CanKillTile)`);
  then `inActive(true)` + `SquareTileFrame(resetFrame: false)` + net sync →
  `wiring.rs::de_active` (`:2968-3004`) with `NOT_REALLY_SOLID`
  (`:376` = {387, 388, 10}, `TileID.cs:592`), `DEACTIVE_SOLID_EXCEPTIONS`
  (`:380`), `PREVENTS_ACTUATION_UNDER` (`:385` = {21, 467, 26, 77, 88, 470,
  475, 237, 597, 441, 468}, `TileID.cs:569`) — all three sets verbatim.
  The `downedPlantBoss` flag is the recorded M5-HARDFORCE stand-in (the bulb
  is always guarded below the surface). PORTED.
- `Wiring.ReActive` (`:3225-3233`): unconditional `inActive(false)` +
  reframe → `wiring.rs::re_active` (`:3008-3014`). PORTED.

### 1b. Collision / WorldGen solidity — PORTED except housing

C# `SolidTile` family (every member consults actuation):

| C# helper | Actuation term | Rust port | Verdict |
| --- | --- | --- | --- |
| `SolidTile(Tile)` `WorldGen.cs:69957-69974` | `nactive()` | `edit/kill.rs::solid_tile` `:580-590` (`active && !inactive`) | PORTED |
| `SolidTile(int,int,noDoors)` `:70111-70132` | `!inActive()` | same helper (the noDoors arm is the caller's) | PORTED |
| `SolidTile2(Tile)` `:70134-70151` / `(int,int)` `:70492-70509` | `!inActive()` | `edit/kill.rs::solid_tile2` `:2158`, `worldgen/evil.rs:1991`, `mini_biomes.rs:1493` | PORTED |
| `SolidTile3` `:70465-70485` | `nactive()` | `stray.rs::solid_tile3` `:27-36`, `tile_light/scanner.rs:1134` | PORTED |
| `SolidTileAllowBottomSlope` `:70167-70189` | `!inActive()` | `edit/kill.rs:597`, `npc_ai_fighter_scan.rs:106-114` | PORTED |
| `SolidTileAllowTopSlope/LeftSlope/RightSlope` `:70320-70378` | `!inActive()` | no ported callers (the slope-join draw uses the raw `.active()` neighbour probes, see below) | N/A |
| `SolidTileNoPlatforms` `:70196-70218` | `!inActive()` | `npc_ai_fighter_scan.rs:92-97` | PORTED |
| `SolidTileNoAttach` `:70511-70528` | `!inActive()` | `edit.rs::solid_tile_no_attach` `:1012-1019` | PORTED |
| `SolidOrSlopedTile` `:69985-70010` | `!inActive()` | `collision.rs::solid_or_sloped_tile` `:551-564` | PORTED |
| `TileEmpty` `:69976-69983` | returns `inActive()` for an active tile | no direct port (callers fold the empty test into their own `nactive` probes) | N/A |
| `Collision.SolidTiles` `Collision.cs:3455-3488` | `active && !inActive && tileSolid && !tileSolidTop` | `npc_ai_caster.rs:155-173`, `npc_ai_bat.rs:86-104`, `wiring.rs::solid_tiles` `:1268-1294` — all carry the term | PORTED |
| `Collision.SolidTiles` (housing caller `WorldGen.cs:5903`) | same | **`housing.rs::solid_tiles_in_box` `:115-127` — GAP, see below** | **MISSING** |
| `Collision.TileCollision` slope/half-brick gates `:3675-3707` | `nactive()` per flag | `collision.rs` (the `:2153-2157` skip at `:1193`, the slope arms) | PORTED |
| `AnyCollision` / `AnyCollisionWithSpecificTiles` `:2813/:2915` | `(!evenActuated && inActive())` | no port counterpart — the only `evenActuated: true` caller is the snow arm, §1d | MISSING (wider AI_010 gap) |

Note: the platform-slope SIDE-FILL draw (`DrawBasicTile`,
`TileDrawing.cs:1524/:1534`) reads its neighbours' `.active()` only — NOT
`inActive()` — so an actuated solid neighbour still triggers the fill in
C#. The port's `FillNeighbor.active = t.active()` (`render/special_tiles.rs:
169`) matches exactly; this is NOT an actuation gap.

**GAP — housing (`housing.rs`, OWNED by W4-d118/d119, NOT touched here):**
`solid_tiles_in_box` (`:115-127`) cites `Collision.cs:1862-1878` — a stale
citation; the real `Collision.SolidTiles` is `:3455-3488`. The port test is
`active && TILE_SOLID` — it drops BOTH `!inActive()` and `!tileSolidTop`.
The `ScoreRoom` home-spot probes drop `nactive()` too: the candidate tile
(`:289`, C# `WorldGen.cs:5903` `!Main.tile[i,j].nactive()`), the side
neighbours (`:299`, C# `nactive()`), and the furniture scan (`:313`, C#
`:5915` `tile.nactive()`). Effect: an actuated floor/wall scores as a home
spot / furniture in the port but not in C#. The housing lanes are in flight
(`task/W4-d118-housing-scoreroom`, `task/W4-d119-housing-relocation`) — this
is recorded for them, not fixed in this lane. (UPDATE 2026-11-21: the
pushed d118 tip 91db266e FIXES `solid_tiles_in_box` — edge guards +
`!inactive` + `!TILE_SOLID_TOP`; the ScoreRoom `nactive` residue remains,
cross-noted with their branch's line numbers in
`docs/research/101-g80-task-seeds.md`.)

### 1c. The `evenActuated` parameter — one C# caller, no port

`Collision.AnyCollisionWithSpecificTiles(..., bool evenActuated = false)`
(`Collision.cs:2813`, the gate at `:2849`) and `AnyCollision(...,
evenActuated = false)` (`:2915`, gate `:2950`). The ONLY `evenActuated: true`
call in the decompile is `Projectile.cs:17601` — the snowfall projectiles
411-414's second collision pass against `TileID.Sets.Falling` (snow falls
through actuated sand but lands on it). The port has no `even_actuated`
parameter anywhere; its falling-tile flight is gravity + landing only
(`projectile.rs:19824-19829` + `:19887-19907`, `falling::landed`), with no
per-tick `TileCollision` at all — a wider AI_010 gap, not actuation-specific.
Recorded for the projectile lane.

### 1d. Other actuation readers — PORTED

The `inActive`/`nactive` census across the port (76 `.inactive()` sites)
covers: grapple (`grapple.rs:149/181/579`), golf (`golf.rs:400`), ball
(`ball.rs:138`), gore (`gore.rs:585`), torch placement (`torch.rs:92/108`),
NPC AI solid probes (the `npc_ai_*` family), town housing stand probes
(`npc_ai_town.rs:789/841`), projectile tile hits (`projectile.rs:12072/
13042/14213`), touch damage (`touch_damage.rs:135`), fishing (`fishing.rs:
205`), liquid settlement (`liquid.rs:786/972`), falling blocks (`falling.rs:
56/61`), smart cursor (`smart_cursor.rs:484/512/546/2347/2674`), growth
(`growth/decorations.rs:29/410`), the lighting scanner (`lighting/legacy.rs:
1217`), and the draw passes (§3). All carry the actuation term where the C#
does. The worldgen-only `nactive` probes (tree/house/dungeon placement) run
on freshly generated worlds with no actuation — the term is vacuous there
and the port's omission is behaviourally identical.

## 2. The actuator item's PLACE side — PORTED (S1-P-i34)

Item data: `ItemID.Actuator = 849` (`ItemID.cs:3381`); `Item.cs:11325-11336`
— useStyle 1, useTurn, useAnimation 15, useTime 10, autoReuse, consumable,
24×28, mech, buyPrice(0,0,10). The port's `itemdata.rs:1015` matches every
field (`consumable: true`, `mech: true`, `tile: None` — the actuator places
no tile, it sets the device bit).

- `WorldGen.PlaceActuator` (`WorldGen.cs:58011-58020`): if no actuator bit →
  sound 0 (Dig) + set the bit + true → `edit.rs::place_actuator`
  (`:120-130`, the sound is the app-side `wire_placed_feedback`). PORTED.
- `WorldGen.KillActuator` (`:58022-58039`): clear the bit + sound 0 + drop
  849 + five dust 50 → `edit.rs::kill_actuator` (`:134-144`) + the app-side
  drop/dust (`world_sim/editing/wiring.rs:122-131`). PORTED.
- The PLACE arm `Player.ItemCheck_UseWiringTools` (`Player.cs:46750-46920`):
  the type gate (`:46752`), the tile-interaction range, `CanDoWireStuffHere`
  (`:46761`, the wall-350 / temple-87 gate), then the 849 branch
  (`:46883-46887`): `sItem.stack > 0 && PlaceActuator` → `ApplyItemTime` +
  net 17/8. The branch decrements NOTHING itself — the generic consumable
  block (`Player.cs:43280-43325`, same tick) burns one because the item is
  `consumable` and `ApplyItemTime` set `itemTime == itemTimeMax`. The port's
  `WiringTool::Actuator` arm (`world_sim/editing/wiring.rs:184-203`) places,
  then burns one from the selected stack — the S1-P-i34 fix (the v1 port
  kept the stack whole; that was WRONG). PORTED.
- The cutter 510 (`:46855-46881`): `KillActuator` FIRST, then yellow/green/
  blue/red wire → `WiringTool::Cut` (`:118-152`), the actuator-first order
  with the 849 drop. PORTED.
- The Actuation Rod 3620 (`:46888-46901`): the actuator-bit gate, the
  per-HOLD lock (`ActuationRodLock`/`ActuationRodLockSetting`, cleared on
  `releaseUseItem` `:18802-18804`), `Wiring.Actuate`, the state-change check
  → `WiringTool::ActuationRod` (`:153-183`). PORTED.
- The `autoActuator` consumer `PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`
  (`Player.cs:40594-40669`): the auto-place-on-build pass (the 11×11 scan,
  the `tileDataCaches` pre-placement snapshot, the same solid/exception
  gates as `DeActive`, `FindItem(849)` + `PlaceActuator` + burn). The flag is
  granted (3624, `world_sim/player.rs:199-201`) and read by the visibility
  gate, but the placement pass is the recorded follow-up
  (`world_sim/player.rs:163-166`, ANALYSIS "The wiring tools"). It needs the
  pre-placement tile snapshot the place route does not yet keep. MISSING —
  recorded, not in this lane's scope (it is a place-route mechanic, not a
  draw one). (PORTED 2026-11-21 in the follow-up round, W4-g80 1284dc63:
  `world_sim/editing/auto_actuator.rs` — the snapshot + the actuator half,
  wired at the `place_tile` tail; the autoPaint half + the block-swap
  consumer stay deferred. ANALYSIS "The autoActuator build pass".)
- The Multicolor Wrench 3625 / `MassWireOperation` (`Wiring.cs:419-453`):
  deferred (the `WiresUI` tool modes), recorded at `world_sim/editing/
  wiring.rs:27-29`. Out of scope.

## 3. The actuation VISUAL transition — INSTANT in C#, port matches

There is NO fade/lerp/timer on actuation in the C#:
- `Wiring.DeActive`/`ReActive` (`Wiring.cs:3216/:3227`) flip the `inActive`
  bit and call `SquareTileFrame(resetFrame: false)` synchronously; no
  animation state is written.
- `TileDrawing` reads `tileCache.inActive()` directly each frame — the
  `GetFinalLight` ghost (`:4380-4382` Color / `:4394-4396` Vector3), the
  positional-switch disco arm (`:1082-1084`), the behind-liquid refusal
  (`:3858`), and `IsTileDangerous`'s `!inActive()` gate (`:4438`). No
  `actuationTimer`/fade field exists on `Tile` or in `Main` (the `Main.cs`
  `actuat` census is all UI/tool-mode/icon code: `:31634`, `:47478-47596`,
  `:58643-58658`).
- The actuator ICON overlay (`Main.cs:58643-58658`, `TextureAssets.Actuator`
  inside `DrawWires`, gated on `Lighting.Brightness > 0`) is owned by the
  pushed W4-g81 lane (6f046998/0017d32a) — not duplicated here.

The port matches: the ghost is computed per frame from `FLAG_INACTIVE`
(`tiles.rs:609-619`), instant on toggle. The `ShouldTileShine`/`Main.shine`
shimmer branch of `GetFinalLight` (`:4384-4387`, the `else if` after the
ghost) is d56's UNWIRED port (5d80b38a, `tr-render::animated_tiles::
final_light`) — the actuation-beats-shine ORDER is pinned there; this lane
does not wire it (tiles.rs is contested by d57/d58/d59/f15). VERIFIED, no
change.

## 4. The parked positional-switch arm — the 83 herb-maturity sheet (PORTED)

research/09's parking list (TILE-DRAW-6): "tile 83's texture override, 323's
frond-column return". The 323 frond return is a documented deviation (no
palm canopy pass); the 83 texture override was the remaining unclaimed small
arm — ported here.

C#: `DrawSingleTile` `case 83` (`TileDrawing.cs:1149-1151`) re-picks
`tileDrawInfo.drawTexture = GetTileDrawTexture(tileCache, tileX, tileY)`;
that overload's `case 83` (`:1450-1455`) sets `key.TileType = 84` when
`WorldGen.IsAlchemyPlantHarvestable(tile.frameX / 18, tileY)` — the mature
(blooming) herb sheet, the frame untouched. The predicate
(`WorldGen.cs:66143-66175`) was already ported for the harvest arm
(`growth/herbs.rs::is_alchemy_plant_harvestable`, `:107-132`).

Port (this lane):
- `TileDrawData.sheet_kind_override: Option<u16>` (`drawdata.rs`) — the
  re-pick; `None` = the tile's own sheet.
- The `case 83` arm in `apply_world_corrections` (which now takes a
  `&WeatherCtx` — the live rain/clouds; the predicate's day/night/moon/time
  inputs read `world.*` internally, exactly as the smart-cursor harvest arm
  does).
- `render/tiles.rs`'s `resolve_tile_frame` call consumes
  `dd.sheet_kind_override.unwrap_or(tile.kind)`; the per-cell `herb_wx`
  WeatherCtx is built from `params.raining` + the new `draw_tiles`
  `rain_cloud_alpha: f32` param (`Main.cloudAlpha`), fed from
  `app/frame/draw.rs` with `rain.visual_alpha()` — the same live value the
  smart-cursor harvest arm uses (`world_sim/smart_cursor.rs:108-113`).
  `remixWorld` stays the M5-SEED-2 stand-in (`false`).
- G6 key: `WorldFrameKey.herb_weather: u32` carries the three non-light
  gates (`cloudAlpha > 0`, `blood_moon || moonPhase == 0`, `world.time >
  40500`), gated on an herb 83 in the view rect (the vine-grid convention).
  `day_time`/`raining`/`time` already ride `light_params`.

Witnesses (WRITTEN, NOT RUN — code-only): `drawdata.rs::
w4_g80_the_harvestable_herb_swaps_to_the_mature_sheet`, `render/tiles.rs::
w4_g80_the_harvestable_herb_draws_the_mature_sheet`, `app/mod.rs::
herb_weather_gates_change_the_frame_key`. ANALYSIS "The actuation/inactive
DOC-SWEEP + the 83 herb-maturity sheet".

## 5. Inactive walls — NONE (verified)

`WallDrawing.cs` has ZERO `inActive`/`actColor`/`actuat` references (grep,
0 matches). `Tile.inActive` is a block-only flag (`sTileHeader & 0x40`);
walls carry `wallColor`/`fullbrightWall`/`invisibleWall` but no actuation
bit. The C# never actuates a wall, and `Wiring.DeActive`'s solid gate
(`tileSolid[type]`) excludes wall-only cells. The port's wall pass
(`render/tiles.rs` wall layer, `tr-core` wall code) has no actuation
handling — correct, matches C#. VERIFIED, no change.

## Summary

| Question | Verdict |
| --- | --- |
| 1. Actuated-tile behaviour | PORTED across collision/wiring/persistence; ONE gap (housing `solid_tiles_in_box` + ScoreRoom `nactive`, owned by d118/d119) + the snow `evenActuated` arm (wider AI_010 gap) — recorded, not fixed |
| 2. Actuator PLACE side | PORTED (S1-P-i34: place/cut/rod + consumption); `autoActuator` build-pass stays the recorded follow-up (PORTED 2026-11-21, W4-g80 1284dc63 — `world_sim/editing/auto_actuator.rs`, ANALYSIS "The autoActuator build pass") |
| 3. Visual transition | INSTANT in C#, port matches; shine branch is d56's unwired port |
| 4. Parked arm | 83 herb-maturity sheet swap PORTED (3 witnesses) |
| 5. Inactive walls | NONE in C#, port matches |

Code-only: no cargo/rustc/rustfmt/tests/graphify invoked. 3 witnesses
WRITTEN, NOT RUN. RNG untouched (the draw pass and the harvest predicate
draw no random numbers).