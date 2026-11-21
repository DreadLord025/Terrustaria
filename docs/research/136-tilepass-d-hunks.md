# 136 — The d-hunk fold prep (UNION-CONTRACT §8.3 step 3, made mechanical)

Produced on `task/W4-g31-tile-draw-pass-4` (attach DONE `ff917100`),
code-only. Every claim below was re-read from the pushed lane tips via
`git diff a725acb04 <tip> -- <file>` / `git show <tip>:<file>` (full
per-file diffs, not summaries) plus the C# windows in
`decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`. This document
replaces §8.3 step 3's one-paragraph sketch with the per-hunk tables,
the fold order + dependencies, the conflict pairs and the per-witness
re-pin notes. It also CORRECTS §8.3 step 3's ordering (see §0) and
§1.4's `is_visible` keep-one verdict.

Lane tips read (all base `a725acb04`, unchanged since the §8 census):

| Lane | Tip | Payload | State |
|---|---|---|---|
| d56 animated | `5d80b38a0` | `tr-render/animated_tiles.rs` (1254 ln, 24 `#[test]`) | UNWIRED, module-only |
| d57 TE furniture | `70f43aa72` | `render/te_furniture.rs` (2151 ln, 44 `#[test]`) + `render/mod.rs` + `projectile_ai_spear.rs` pub lift | UNWIRED, module-only |
| d58 chest frames | `a13635dcb` | `tr-render/chest_frames.rs` (195 ln) + `chest.rs` (+135) + WIRED `tiles.rs` (+87) / `special_tiles.rs` (−159) / `app/mod.rs` (+63) / `frame/draw.rs` (+36) | WIRED, unconditional |
| d59 platform slopes | `f70498be0` | WIRED `tiles.rs` (+452) + `special_tiles.rs` (+89/−22) | WIRED, unconditional |
| d60 particles | `77429fe1d` | `tile_draw_particles.rs` (2163 ln, 22 `#[test]`) + `main.rs` mod decl | UNWIRED, module-only |
| f15 glow bind (co-lane) | `8ca19097d` | `render/tile_glow.rs` (290 ln) + WIRED `tiles.rs` (+115) + `drawdata_cases_a.rs` (+151/−…) + `assets.rs` (+48) | WIRED on its branch; the glowmask TABLE pass explicitly out of scope |
| g68 wind leaves (co-lane) | `b20480d72` | `foliage.rs` rewrite (+792/−…) incl. UNWIRED `emit_liquid_drops`, WIRED `tiles.rs` leaf-arm swap (+84/−…) / `frame/draw.rs` signature + `tr-core/gore.rs` (+560) | leaf path WIRED; drips UNWIRED |

**Line-number convention:** `tiles.rs` line ranges are the BASE
(`a725acb04`) fused-loop coordinates — the split's source text. Rust
symbol lines are the lane-tip NEW-FILE coordinates. The union seam
names are this branch's post-fold `tile_pass.rs`/`tile_drains.rs`
(§8.2 signatures).

---

## 0. Correction to §8.3 step 3's order (the C# re-read)

§8.3 step 3 sketched: "gate → d58 chest rewrite → d56 gate/light →
black feed → d59 slope family → d60 emission → sprite push". The
C# `DrawSingleTile` re-read fixes the REAL order (three of those
positions were wrong):

* **d60 emission is EARLY, not late**: `DrawTiles_EmitParticles` runs
  at `:965-968`, BEFORE `DrawTiles_GetLightOverride` (`:969`), the
  `flag` gate (`:970-991`) and the black feed (`:988-991`) — and it
  reads the POST-VISION-FLOORS light (`:897-963` ran first). Its own
  `IsVisible` early return is mid-body (`:6952-6955`): the arms above
  it fire even on a hidden cell.
* **d58's chest rewrite is AFTER the gate/black feed**:
  `CacheSpecialDraws_Part2` runs at `:993` (Part1 `:992` first), not
  right after the drawdata dispatch — the drawdata dispatch
  (`GetTileDrawData` `:876`) precedes the vision floors.
* **d56 splits into TWO sites**: the vision floors + dust gates
  (`:897-963`) are the DrawSingleTile PROLOGUE; the gate
  (`tile_draw_gate`, `:970-991`) + final colour (`final_light`,
  `:1019` = `GetFinalLight` `:4358-4389`) are the mid-body decision.

The mechanical per-cell order (the `classify_and_draw_cell` body once
the `:517` driver gate and the g30 `Draw`-switch `:527-723` ran):

| # | C# site | What runs | Owner hunk |
|---|---|---|---|
| 1 | `:876` | `GetTileDrawData` (static table + world corrections + slice B + slice A) | fused loop's existing dd dispatch (KEEP, moved) |
| 2 | `:897-963` | vision floors (danger/spelunker/biome-sight) + the three `_rand` dust gates `Next(30)/Next(60)/Next(480)` | d56 `vision_changer_light` (+ `is_tile_dangerous`/`is_tile_spelunkable`/`is_tile_biome_sightable`) |
| 3 | `:965-968` | `_isActiveAndNotPaused && (!Lighting.UpdateEveryFrame \|\| FastRandom(TileFrameSeed).WithModifier(x,y).Next(4)==0)` → `DrawTiles_EmitParticles` | d60 `draw_tiles_emit_particles` (the caller owns the quarter-roll gate; THROWAWAY FastRandom, never steps `_rand`) |
| 4 | `:969` | `DrawTiles_GetLightOverride` | d56 `light_override` |
| 5 | `:970-991` | `flag` light test + `IsVisible` veto + `DrawBlack` decision | d56 `tile_draw_gate` (glow inputs = f15's record + the glowmask id, §3) |
| 6 | `:988-991` | `drawBlackHelper.DrawBlack(x, y)` when `!flag && visible` | g29 `DrawBlackHelper` feed → `draw_black_strip` at `:728` |
| 7 | `:992` | `CacheSpecialDraws_Part1` (TE anchors, `skipDraw = !flag`) | d57 `te_anchor`/`collect_te_points` (per-cell form, §1 d57 note) |
| 8 | `:993` | `CacheSpecialDraws_Part2` (BasicChest rewrite) | d58 `chest_draw_frame` + `tr_render::chest_frame_adjust` |
| 9 | `:994-1006` | the 72 shroom-cap overlay | fused loop residual (MISSING per research/09 — not a d-hunk) |
| 10 | `:1007-1013` | source rect + `vector` anchor | fused loop's existing rect/anchor code (KEEP, moved) |
| 11 | `:1014-1016` | `if (!flag) return;` — nothing below runs on a black/hidden cell | d56 `DrawGate.draw` |
| 12 | `:1018-1019` | `colorTint = White` → `GetFinalLight` | d56 `final_light` |
| 13 | `:1020-1176` | `DrawBasicTile` incl. the slope family + platform side-fills + half-brick joins + the flames tail `:1723` | d59 slope family + the fused loop's draw arms |
| 14 | `:1178-1342` | the `Main.tileGlowMask` table pass (717 case `:1193-1197`) | UNPORTED table — the 717 arm binds g31 `tile_ctor::lava_light_a` (§3) |
| 15 | `:1343-1403` | the second-switch glow record bind (412 crossfade / plain / platform glow side-fill) | f15 `tile_glow::draw_tile_glow` |
| 16 | `:1404-1408` | the highlight overlay | unported (debug family) |

Stream rule inside the body: steps 2/3 draw on the ONE `_rand`
(`UnifiedRandom` post-merge, §8.3 step 4) in this order; step 3's
quarter-roll is a THROWAWAY `FastRandom(TileFrameSeed)` (g68's
deferred-2 census) and step 3's pool internals draw on `Main.rand` —
three streams, never folded into one.

---

## 1. Per-lane per-hunk tables

Target-seam vocabulary: **CAD** = `TilePassSeams::classify_and_draw_cell`
(the per-cell DrawSingleTile body), **TE-post** =
`post_draw_pass(PostDrawStep::DrawEntitiesHatRacks/DrawEntitiesDisplayDolls)`
(the `PostDrawTiles(true)` half, `:435-441`), **post-step** = the ten
non-solid nature steps (`POST_DRAW_NON_SOLID`, drains), **driver** =
`draw_layer` prologue/epilogue (g29-owned, d-lanes only feed it),
**module** = pure decision maths, no seam of its own.

### d56 — `tr-render/src/animated_tiles.rs` (tip `5d80b38a0`, UNWIRED)

| Hunk (new-file lines) | Symbol | What it draws/decides | C# anchor | Target seam |
|---|---|---|---|---|
| 54-118 | `BOULDERS`/`TOUCH_DAMAGE_*`/`touch_damage`/`touch_hot`/`is_pressure_plate_track` | the danger predicate's tables | `TileID.cs:504` etc., `:4435-4467` | module (CAD step 2 input) |
| 120-163 | `is_tile_dangerous` | Dangersense predicate (accessory + world gates) | `TileDrawing.cs:4435-4467` | CAD step 2 |
| 165-180 | `is_tile_spelunkable` | Spelunker predicate (49-row table + frame windows) | `Main.cs:21783-21798` | CAD step 2 |
| 182-207 | `is_tile_biome_sightable` | Biome Sight predicate + colour (remix exceptions) | `Main.cs:21800-21818` | CAD step 2 |
| 209-334 | `VisionDust`/`VisionLight`/`vision_changer_light` + the three gate consts 30/60/480 | the byte floors + the three dust REQUESTS (caller rolls `Next(30)/Next(60)/Next(480)` on `_rand`; the spelunker arm's bitwise `&` still rolls when active) | `:8481-8552` + the DrawSingleTile prologue `:897-963` | CAD step 2 **and** the drains' `DrainCellLight::cell_light` (the SAME chain at `DrawGrass` `:8348-8352` — `can_do_dust` is the drain's ALREADY-CONSUMED `Next(4)==0`, never re-rolled) |
| 336-431 | `xna_scale_color`/`light_override` | `DrawTiles_GetLightOverride` (541/631/83/84/481-483 arms) | `:6624-6686` | CAD step 4 + `DrainCellLight` (`DrawGrass` `:8349`) |
| 433-511 | `TILE_FLAME`/`IGNORE_DRAW_LIGHT_CONDITIONS`/`DrawGate`/`tile_draw_gate` | `flag` + `IsVisible` veto + `draw_black` | `:970-991`, `IsVisible` `:1411-1432` | CAD steps 5-6 (the black feed = g29 helper) |
| 513-671 | `TILE_SHINE2_STATIC`/`tile_shine2`/`should_tile_shine`/`shine_color` | the shine/shimmer colour arms (59 static rows + the dynamic 185/186 windows, last-write-wins) | `Main.cs:21557-21660`, `:4404-4432` | module (CAD step 12 input) |
| 673-731 | `act_color`/`final_light` | `GetFinalLight`: tint truncate + alpha-255 clamp, fullbright-then-ghost (NOT early return), actuation ×0.4 double, shine arm | `:4358-4389` | CAD step 12 |

No `tiles.rs` hunk exists in d56 — its "attach" is the REPLACEMENT of
the fused loop's dark-merge + gates (base `tiles.rs:786-848` region)
and the actuation-ghost `l` (`:610-619`), exactly as §8.1 censused.
The `is_visible` keep-one note: see §4 correction.

### d57 — `render/te_furniture.rs` (tip `70f43aa72`, UNWIRED)

| Hunk (new-file lines) | Symbol | What it draws/decides | C# anchor | Target seam |
|---|---|---|---|---|
| 65-96 | `te_anchor` | Part1 anchor decode for 395/520/471/470/475 | `:4154-4247` | CAD step 7 (per-cell) |
| 98-143 | `collect_te_points` | the view-rect Part1 walk with per-kind dedup (first-seen row-major = the dictionary insertion order the `DrawEntities_*` walks iterate) | `:4152-4247`, `AddSpecialLegacyPoint` `:6601-6611` | driver-fed collection: run per-cell inside CAD step 7 (dedup map lives in the seams impl, cleared by `LegacyCacheClear`'s five TE steps `:6616-6620`); the walk-order witness stays |
| 145-338 | `ITEM_ICON_PULSE`/`is_disco_item`/`get_item_light`/`draw_item_color_and_scale`/`BOSS_BAG`/`item_get_alpha`/`item_get_color` | the ItemSlot icon colour-and-scale chain | `ItemSlot.cs:2848-2890/:2961-2976/:3760-3801`, `Item.cs:49170-49383` | module (legacy-arm + TE-post input) |
| 340-459 | `IconDraw`/`icon_resolve`/`item_frame_icon`/`weapons_rack_te_icon` | the 395 item-frame + 471 weapons-rack TE icon draws | `DrawSpecialTilesLegacy` `:7951-7957/:7986-8000` | `draw_special_tiles_legacy` (`:736`, over `LegacyCaches.legacy_points`) |
| 461-555 | `FoodPlatterDraw`/`food_platter_draw`/`rack_second_pass`/`tint` | the 520 platter draw (bottom-third food frame, integer origin, +2 Y) | `:7959-7984` | `draw_special_tiles_legacy` |
| 557-656 | `DollPose`/`UsePose`/`supported_use_poses`/`doll_pose_valid` | the doll pose machine (mount forces standing, invalid use-pose falls back) | `TETrainingDummy`/player-draw decision maths | module (TE-post input) |
| 658-862 | `DollDraw`/`display_doll_draw`/`HatRackDraw`/`hat_rack_draw` | the doll/hat-rack player draw decisions (frame 36/72 flips, row picks) | `DrawEntities_DisplayDolls` `:8026-8038`, `DrawEntities_HatRacks` `:8039-8050` | **TE-post** (`PostDrawStep::DrawEntitiesHatRacks` then `DrawEntitiesDisplayDolls` — hat racks FIRST, `POST_DRAW_SOLID`); the `LegacyPlayerRenderer.DrawPlayer` call itself is the wiring step |
| 864-1230 | `DISPLAY_DOLL_DRAW_USE_STYLE`/`IS_A_PHASEBLADE`/`DollProjEnv`/`doll_proj_defaults`/`calculate_frame_from_misc_counter`/`OFFSETS_PLAYER_ONHAND`/`doll_hand_position`/`item_display`/`ai_display_doll` | the doll held-item preview (aiStyle dispatch, hand anchors, the omitted sim-side AI writes enumerated in `70f43aa7`) | `Projectile.cs:22913` + the AI_019 family | module (TE-post input); `projectile_ai_spear.rs::spear_offset` pub lift is its dependency (already in the lane diff) |

Small co-hunks: `render/mod.rs` +5 (the `te_furniture` declaration —
union §1.5 already keeps it), `projectile_ai_spear.rs` `fn` →
`pub(crate) fn spear_offset` (doc-only body change). No RNG anywhere in
d57 (grep census: zero `Next(`/rand usage) — the TE passes are
stream-free.

### d58 — chest frames (tip `a13635dcb`, WIRED)

| Hunk (file @ base lines) | What it does | C# anchor | Target seam |
|---|---|---|---|
| `tiles.rs` `draw_tiles` signature (+5 after `tile_anim:` param, base ~`:325`) | adds `chests: &[crate::chest::WorldChest]` | `Main.chest[]`, `:4311` | CAD step 8 input (the seams impl holds the slice) |
| `tiles.rs` +7 at base `:591` (after `vine_roots_seen`) | the per-draw `chest_positions: HashMap<(i32,i32), Option<i32>>` | `_chestPositions` `:96`, cleared `:6615` | seams-impl storage; `clear_legacy_cache(LegacyCacheClear::ChestPositions)` clears it (§8.2 row already binds) |
| `tiles.rs` +33 inside the dd block (base `:743`, after `apply_cases_a`, before `dd` returns) | the BasicChest rewrite: `chest_draw_frame` → `chest_frame_adjust` → `dd.add_fr_x/add_fr_y/height` | `CacheSpecialDraws_Part2` `:4289-4330`, called `:993` | CAD step 8 — MOVE from the dd block to after the gate/black feed (the C# `:993` position; the wired site is earlier than C# but dd is not read between, so pixels agree — the split must still move it to keep the witness's order claim true) |
| `special_tiles.rs` −159 | `ChestFrameAdjust`/`chest_anchor`/`chest_frame_adjust` + 5 witnesses MOVE to `tr-render/chest_frames.rs` (keep-one) | same | already resolved in-lane; the union keeps the tr-render copy |
| `chest.rs` +135 | `chest_draw_frame` (the `_chestPositions` cache walk, `FindChest` `:4309`), `chest_frame_key` (G6), 4 witnesses | `:4291-4303` | seams-impl helper + G6 |
| `app/mod.rs` +63 | `WorldFrameKey::chest_frames` field + 2 zero-inits + the key-coverage witness | lid animator `Main.cs:17810` | driver/G6 — SURVIVES the split unchanged (the chest rewrite stays inside the cached range) |
| `frame/draw.rs` +36 | the `chest_frames:` key arm (BasicChest-in-view gated) + the `&self.world_chests` call arg | tile_anim convention | driver — survives; the call arg re-points at the seams impl |
| `tr-render/chest_frames.rs` +195 | the moved maths + 6 witnesses (5 moved + the lid-row selection) | `:4289-4330` | module (CAD step 8) |

### d59 — platform slopes (tip `f70498be0`, WIRED)

| Hunk (file @ base lines) | What it does | C# anchor | Target seam |
|---|---|---|---|
| `tiles.rs` base `:1017-1023` (+55/−3) | the slope dispatch: `!SET_DONT_DRAW_TILE_SLOPES` gate → platform arm (flat full rect + `draw_platform_side_fill`) → `SET_HAS_SLOPE_FRAMES` flat arm → generic `push_slope` columns → NEW `half_brick_join` arm BEFORE the cell's own half-brick arm | `DrawBasicTile` `:1510-1629` | CAD step 13 (inside the DrawBasicTile body) |
| `tiles.rs` +205 after base `push_half` (~`:1630`) | `draw_platform_side_fill` (neighbour facts + tint + sheet uv + push), `HalfBrickJoin`/`half_brick_join` (the `:1586` gate chain), `half_brick_join_rects` (the 126/90/144/148/156 join columns; right arm's FIXED columns), `push_half_brick_join` | `:1515-1545`, `:1586-1629` | module-level helpers of CAD step 13 (move with the body) |
| `special_tiles.rs` +89/−22 | `FillNeighbor::from_world` (the REAL Echo bit via `World::invisible_block`), `push_platform_side_fill` gains `flip`/`paint` params + returns the quad count; module doc UNWIRED→partially-wired; 1 new witness | `:1524/:1534`, `Tile.cs:528-531` | module — **f15's glow side-fill shares these gates** (§2 conflict pair C4) |
| `tiles.rs` tests +186 | 5 witnesses (join gates, smooth/same_above flags, both-sides rects, one-sided rects, dispatch-order) | `:1515-1629` | re-pin: §4 |

### d60 — `tile_draw_particles.rs` (tip `77429fe1d`, UNWIRED)

| Hunk (new-file lines) | Symbol | What it emits | C# anchor | Target seam |
|---|---|---|---|---|
| 135-182 | `EmitRand` trait + `UnifiedRandom` impl | the `_rand` draw seam (one method per C# conversion) | — | module (the ONE `_rand` binds it) |
| 164-226 | `EmitCtx`/`ParticleRequest`/`RubbleDustPlan`/`EmitOut` | the Main-statics bundle + orchestrator requests + the d54-deferred rubble plan | `ParticleOrchestraType.cs:36/53/54/58` | seams-impl inputs/outputs |
| 228-365 | `tile_shine`/`torch_dust`/`spawns_nature_flies`/`makes_rubble_dust` | the four data tables | `Main.cs:7134-…` passim | module |
| 367-440 | `solid_tile3`/`solid_tile`/`active_and_walkable`/`any_liquid_at`/`snap_top_left`/`next_vector2_circular(_edge)` | probes + the `Main.rand` vector helpers | `WorldGen.cs:69957+`, `Utils.cs` | module (solid_tile keep-one candidate, §2 C6) |
| 442-469 | `is_visible` | the FOURTH `IsVisible` copy | `:1411-1432` | keep-one (§4 correction) |
| 471-801 | `draw_tiles_emit_particles` | the WHOLE per-cell battery in C# order: 718 star clouds (`:6692`), 244 pumpkin (`:6752`), 565 fog (`:6775`), 215/592/406/452 chain (`:6780-6847`), **192/384 leaf (`:6848-6855`)**, 666/712 poo (`:6856`), 711 boulder (`:6863`), nature flies (`:6881`), rubble gate (`:6901`), the `IsVisible` early return (`:6952`), 238 spore (`:6956`), the 139/463/497/165/42 chain (`:6962-7045` — 139 = the lantern PETALS `:6966-6979`), torch/lava/candle arms (`:7105-7310`, incl. the 646 DISCARDED `Next(2)` `:7256`), 83 herbs (`:7311-7318`), the ambient/shine/shimmer tail (`:7319-7546`) | `:6688-7547` | **CAD step 3** — ONE call, never split per-arm (the arms share the fall-through chains; splitting re-orders `_rand`) |
| 803-1017 | `star_cloud_sparkles`/`pumpkin_candle_gore`/`campfire_smoke`/`chimney_smoke`/`balloon_machine` | the per-arm bodies | as above | module (CAD step 3 internals) |
| 1019-1105 | `emit_living_tree_leaf(_below/_sideways)` | the faithful leaf shed (unconditional `Next(2)`, two-open-cell gate, paint column) | `:7549-7592` | **DUPLICATE of g68's foliage.rs copy — keep-one at merge (§2 C2)** |
| 1107-1295 | `lantern_petals`/`glowing_moss_dust`/`lantern_dust_gore`/`rising_smoke_gate`/`lava_bubble` | petal cadence (RAW frames + `%7`), moss/lantern dust, smoke gate, lava bubbles | `:6966-6979` + the chain arms | module (CAD step 3 internals) |
| 1297-1377 | `emit_alchemy_herb_particles` | the per-style herb blooms (deathweed's TWO independent gates) | `:9605-9655`, called `:7316` | module (CAD step 3; NOT a post-step — the only caller is inside the battery) |
| 1379-1560 | `ambient_dust_tail`/`shine_arm` | the long else-if tail (ore shine `Next(tileShine)` BEFORE the frame windows, the light≤20 early return, gemspark colours) | `:7319-7546` | module (CAD step 3 internals) |

`main.rs` +6: the `mod tile_draw_particles;` declaration (union keeps).
The 646 discarded draw, the torch gate-before-frameX order and the
fall-through chains are pinned by witnesses — the fold must call the
battery as ONE body at step 3 or the stream order breaks.

---

## 2. Fold order + dependencies (the mechanical script)

Precondition: d56-d60 + the g-union + f15 + g68 all on main; §8.3
steps 1-2 done (the fused walk split, ONE seams impl). Then:

**F0 — split-first rule (d58/d59).** d58's and d59's hunks are
UNCONDITIONAL per-cell code INSIDE the fused loop. They cannot be
folded before the split exists: fold them by MOVING the wired hunks
verbatim into the `classify_and_draw_cell` body at their §0 positions
(d58 → step 8, d59 → step 13). Do NOT re-derive them from the lane
modules — the wired hunks are the reviewed text (the d58 dd-block site
moves later in the body; the d59 dispatch replaces the fused loop's
`else if tile.slope() != 0` arm wholesale). Their helper fns
(`draw_platform_side_fill`, `half_brick_join*`, `push_half_brick_join`)
move with the body; `chest_positions` moves into the seams impl.

**F1 — d56 (the light chain) folds FIRST among the unwired lanes.**
Dependency root: d60's battery consumes the post-vision light (step 2
before step 3), and the gate's output decides the black feed (steps
5-6) that g29's helper coalesces. Bind: `vision_changer_light` +
`light_override` + `tile_draw_gate` + `final_light` at steps 2/4/5/12;
the same pair (`vision_changer_light` → `light_override`) binds
`DrainCellLight::cell_light` for the drains (the `can_do_dust` flag is
passed, never re-rolled — §8.2). The fused loop's float
`HIGH_LIGHT`/`MEDIUM_LIGHT` compares are REPLACED by the g28 tiers'
sliced arm (§8.2 row) in the same fold — d56's byte chain is the
authority for the non-sliced path.

**F2 — d60 (the battery) folds SECOND.** Needs F1's light bytes +
the ONE `_rand` + `Main.rand` + the dust/gore pools + the WindGrid in
the seams impl. Bind `draw_tiles_emit_particles` as ONE call at step
3 behind the caller-owned quarter-roll gate (g68 deferred-2 cadence
contract: throwaway `FastRandom(tile_frame_seed).with_modifier(x,y)`,
5-draw seed cycle, emission OUT of the cached range per the
tile_cracks precedent — never widen G6). The inline 192-leaf arm
(base `tiles.rs:1078-1122`, g68-rewired) DROPS — the battery's
`:6848-6855` arm owns it (conflict C2). `EmitOut.particles` feeds the
net-module `Particles` requests; `RubbleDustPlan` stays deferred to
d54 (the plan's three `_rand` draws already consumed — the d54 wiring
must NOT re-roll them).

**F3 — d57 (TE furniture) folds THIRD.** Needs F0's split (the
legacy-points walk is the `:736` epilogue seam) + the g32
`LegacyCaches`. Bind `te_anchor` per-cell at step 7 (the dedup maps
live in the seams impl; `LegacyCacheClear` steps 2-6 clear them,
`LayerCacheClear`'s solid pair clears the doll/hat-rack maps);
`item_frame_icon`/`food_platter_draw`/`weapons_rack_te_icon` into
`draw_special_tiles_legacy` (with the on-main 237/334 arms);
`display_doll_draw`/`hat_rack_draw` + the pose/preview machine into
the two TE-post `None` arms (§8.2 `post_draw_pass` row). The
`DrawPlayer` call needs the entities renderer in the seams impl —
the same handle the `PortedBodyDrain` bodies need (§8.2).

**F4 — d58/d59 witness re-pins + the f15/g68 co-lane reconciliations**
(§3/§5) close the fold. Order within F4 is free (no stream coupling).

Dependency graph: `F0 (split) → F1 (d56) → F2 (d60) → F3 (d57)`;
d58/d59 move at F0; f15's glow record feeds F1's gate inputs (C3) and
its side-fill shares d59's gates (C4); g68's leaf copy is F2's keep-one
winner (C2); g68's drips bind at F0's CAD switch (C5, §5).

### Conflict pairs

* **C1 — d57 TE-post vs d60 petals: NO overlap (the §8.3 question,
  answered).** The petals are tile **139** hanging lanterns, emitted
  inside the battery's `:6962-7045` chain (`:6966-6979`) at CAD step
  3 — NOT a post-step, NOT the 470/475 doll/hat-rack region. d57's
  TE-post passes run at `PostDrawTiles(true)` (`:435-441`), after the
  whole solid `Draw`; d60 never touches `PostDrawTiles`. The only
  shared resource is the ONE `_rand`: d57 draws NOTHING (zero-RNG
  census), so the TE-post passes are stream-transparent. No conflict.
* **C2 — d60 `emit_living_tree_leaf` vs g68 `foliage.rs` copy vs the
  inline arm.** THREE copies of `:7549-7592` exist at the tips: d60's
  module (EmitRand-based), g68's foliage.rs (`FastRandom`-gate based,
  WIRED into the inline arm's replacement), and the retired base
  inline arm (already deleted by g68). Keep-one: **g68's foliage.rs
  copy survives** (it is the wired, witness-covered one — 8 foliage
  witnesses); d60's battery leaf arm calls it (adapting `EmitRand` →
  the concrete `UnifiedRandom`/`FastRandom` pair at the ONE-stream
  merge), d60's three leaf fns DROP with their 1 leaf witness folding
  into g68's `w4_g68_dead_wind_sheds_below_and_sideways` (same
  assertion). The battery's `:6848-6855` GATE stays d60's (it draws
  `Next(max_value)` on `_rand` in battery order — moving the gate to
  the caller would re-order the stream).
* **C3 — d56 `tile_draw_gate` vs f15's dark-merge extension.** f15
  extends the fused loop's `flame_tile` with `|| glow.is_some()` (the
  `:971` `glowTexture != null` term). d56's gate already takes
  `has_glow_texture: bool` + `glow_mask_id: i16` — the fold feeds
  f15's record (`glow.is_some()`) and the glowmask-table id into
  those params and DELETES the fused `flame_tile`/`light_is_zero`
  merge gate (base `:786-848`). The glowmask id has NO port table yet
  (`Main.cs:10024+`; `flame.rs::has_glow_branch` is the flame-only
  subset) — until the table lands, feed `-1` EXCEPT the 717 bind
  (§3), recording the deviation.
* **C4 — d59 `platform_side_fill` gates vs f15's glow side-fill.**
  f15's `tile_glow.rs` calls `special_tiles::platform_side_fill` with
  its own `neighbor()` built on `FillNeighbor::from_tile` — the
  **invisible bit reads false** there, while d59's draw-side
  `draw_platform_side_fill` uses `from_world` (the real Echo bit).
  Same C# gates (`:1373/:1387` mirror `:1524/:1534`), different
  inputs = a semantic divergence at the merge. Resolution: f15's
  `neighbor()` rebinds to `FillNeighbor::from_world` (d59's canonical)
  — one-line fix, recorded as a fold obligation, NOT a behaviour
  change (the C# reads the real bit at both sites).
* **C5 — g68 `emit_liquid_drops` vs g30's `LiquidDrops` class.** g30
  records `liquid_drop_cells` and `continue`s (the `:688-694` switch
  arm); g68 ported the emission body UNWIRED. The bind: CAD's
  `CellClass::LiquidDrops` arm calls `foliage::emit_liquid_drops`
  DIRECTLY per cell (the C# emits inside the switch, before
  `DrawSingleTile` — recording-then-draining would move the `_rand`
  gate draws AFTER the other cells' draws and break the stream).
  The recorded cells stay for the shadow plan census. §8.5 below.
* **C6 — `solid_tile` copies.** g68 foliage.rs, d60, `tile_ctor.rs`,
  `drawdata_cases_a.rs:129` each carry a private `WorldGen.SolidTile`
  twin (identical bodies, verified by read). Keep-one at the fold:
  the lowest-crate copy wins; NOT executed here (all four are on
  different branches). Recorded so the merger doesn't treat them as
  semantic conflicts.
* **C7 — d58 chest-rewrite position.** The wired site (inside the dd
  block, base `:743`) is EARLIER than C# `:993`. Pixels agree today
  (nothing between reads `dd.add_fr_*`), but the split MUST move it
  to step 8 or d58's own order witness
  (`w4_d58_the_chest_rewrite_runs_after_the_dispatch_before_the_rect`)
  pins a position the C# doesn't have. Move, don't re-derive.

---

## 3. The 717 lava glowmask bind (f15 co-lane, `8ca19097d`)

**f15 seam status: LIVE on its branch, NOT on main.** `tile_glow.rs`
(290 ln) binds the SECOND-switch glow record at `:1343-1403` (412
crossfade, plain centred anchor vs platform `vector`, the glow
side-fill sharing d59's gates). Its research doc
(`docs/research/f15-glow-bind-tiledraw2.md`) explicitly scopes OUT
the `Main.tileGlowMask` TABLE pass (`:1178-1342`): "separate wide
task, explicitly out of scope (flame.rs module doc names it)". The
717 arm lives in THAT table pass — so the bind cannot wire against
f15's live seam; it is a BIND CONTRACT for the table-pass lane (or
the family merge), published here.

**The bind contract (717):**

* Site: the glowmask table pass, `TileDrawing.cs:1178-1197`, which
  runs AFTER `DrawBasicTile` (`:1176`, flames tail `:1723`) and
  BEFORE f15's record bind (`:1343`) — i.e. between CAD steps 13 and
  15. It draws ONLY when `Main.tileGlowMask[type] != -1`; for 717 the
  mask id is **362** (`Main.cs:10056`), sheet `Glow_362` (f15's
  `assets.rs` added the 44 `Glow_{id}` sheets — verify 362 is in the
  set at the bind; the mask set was derived from `glow_switch`, the
  TABLE-pass masks are a different census).
* Colour: `let v = tile_ctor::lava_light_a(world, tx, ty,
  time_for_visual_effects);` → XNA `Color(v, v, v, v/2)` — the
  float ctor truncates per channel toward zero:
  `[(v*255.0) as u8; 3]` + alpha `((v/2.0)*255.0) as u8` (the
  `xna_scale_color` convention). `v` is already 0..=1 (the
  `Remap(0.7,1,toMin,1)` clamp), `toMin = 0.8` when the cell above is
  not strict-solid (`tile_ctor.rs:152-157`).
* Rect/anchor: the generic `!flag3` tail (`:1291-1340`) — the cell's
  own `rectangle` (`:1007`, halfBrickHeight-adjusted) at `vector`
  (`:1013`), the same anchor f15's platform arm reuses; the 717 case
  sets no `flag3`, no texture override (the mask pick at `:1180-1184`
  already swapped `drawTexture` to `GlowMask[362]`).
* Gate coupling: `Main.tileGlowMask[717] != -1` ALSO lifts the `:971`
  `flag` — d56's `tile_draw_gate(glow_mask_id: 362)` keeps a fully
  dark 717 cell out of the black merge (C3). The two binds must land
  together or the dark-cell pixels diverge.
* Streams: NONE. `lava_light_a` is pure (the noise table is the
  seed-0 OnceLock, `time_for_visual_effects` is the caller's clock);
  the bind is stream-transparent.
* Witness re-pin: `tile_ctor.rs`'s 6 ctor witnesses stay (pure maths);
  the bind adds ONE order witness (the 717 draw sits between the
  flames tail and the record bind, the colour is `[v,v,v,v/2]`
  truncated) — the `td2_glow_bind_draws_after_the_flames_hook`
  pattern, extended with the table-pass position.

**G6 note:** `time_for_visual_effects` feeds 717's glow colour inside
the cached range — the table-pass lane must key it (the `anim` key
already carries the visual-effects clock for the animator rows;
verify at the bind, do NOT widen silently).

---

## 4. Per-hunk witness re-pin notes

The split deletes the fused-loop text several witnesses scan with
`include_str!("tiles.rs")`. Re-pin map (witness → new home + new
assertion target):

| Witness (lane) | Pins today | Re-pin at the fold |
|---|---|---|
| d58 `w4_d58_the_chest_rewrite_runs_after_the_dispatch_before_the_rect` (tiles.rs) | the dd-block position via `let dd = if active` … `// Resolve the drawn tile frame once` markers | MOVE into the seams-impl file; scan the `classify_and_draw_cell` body: `apply_cases_a` < `is_basic_chest` gate < the rect resolution, ONE `let adj = tr_render::chest_frame_adjust` call, the fresh `chest_positions` in the impl's clear path. **f15 renames the start marker to `let (dd, glow) = if active`** — the merged witness must use f15's marker (its own diff already re-pins the S1-G-1f feed witness the same way). |
| d58 `chest_lid_state_changes_the_frame_key` (app/mod.rs) | the G6 key field | UNCHANGED (the key survives the split; the field list gains no members from the split itself). |
| d58 chest_frames 6 + chest.rs 4 | pure maths/cache walks | UNCHANGED (module-level). |
| d59 `slope_family_dispatch_matches_the_csharp_chain` (tiles.rs) | the `SET_DONT_DRAW_TILE_SLOPES` → platform → side-fill → HasSlopeFrames → columns → join → half-brick order in `draw_tiles` | MOVE with the dispatch into the CAD body; the marker chain is unchanged EXCEPT the enclosing fn (`classify_and_draw_cell`, not `draw_tiles`) and the `#[cfg(test)]` boundary slice. The 4 pure-maths join witnesses are UNCHANGED. |
| d59 `from_world_reads_the_echo_bit…` (special_tiles.rs) | the Echo-bit snapshot | UNCHANGED; gains the C4 obligation (f15's `neighbor()` rebinds to it). |
| f15 `td2_glow_bind_draws_after_the_flames_hook` (tiles.rs) | `draw_tile_flames` < `tile_glow::draw_tile_glow`, ONE bind site, `if let Some(g) = glow`, the `|| glow.is_some()` dark-merge term | MOVE into the CAD body; the `|| glow.is_some()` assertion DROPS (C3: the term becomes `tile_draw_gate(has_glow_texture = glow.is_some(), …)`); re-pin as the gate-call argument assertion. The `anim_row` count witness (3→4) re-pins to the CAD dd block. |
| d56 24 witnesses | pure maths + the four-family chain order | UNCHANGED (module-level, no tiles.rs scan). The `the_four_families_ride_gate_then_final_colour` chain witness becomes the CAD body's order template. |
| d57 44 witnesses | pure maths (anchors, colour chain, poses, preview) | UNCHANGED. The fold ADDS the legacy-walk order witness (the `:736` epilogue over `legacy_points`) + the TE-post order witness (`POST_DRAW_SOLID` hat-racks-first already pinned by g29's `post_draw_tiles_passes` successor). |
| d60 22 witnesses | the battery's gate orders + two-stream census | UNCHANGED except the leaf witness (C2: folds into g68's). The battery is called as ONE body — no per-arm re-pin. |
| g68 11 foliage + 11 gore + 1 sand | the leaf shed + physics + drips | UNCHANGED (foliage.rs survives); the drip witnesses' UNWIRED note drops when C5 binds. |
| g31 attach 4 (tile_attach.rs) | the shadow's frame order/dispatch/determinism/attach-site | REBIND to the real impl per §8.3 step 6 (already prescribed); the attach-site witness's `leaf_rng`-separation assertion re-pins to the ONE `_rand` (the separation is RETIRED at the pixel merge — the leaf rolls and the drain rolls share the stream in call order, §8.3 step 4). |
| g31 rand-stream witness (tile_drains.rs) | the per-pass `Next(4)` flag fold | UNCHANGED — it is the byte-equality template for the merged stream (§8.3 step 4). |

**`is_visible` keep-one CORRECTION (§1.4):** the contract says "d56's
`tr_render` copy" — but d56's tip has NO standalone `is_visible` fn;
the logic is INLINED in `tile_draw_gate` (`animated_tiles.rs:489-503`)
and takes `invisible_block: bool` as a parameter. The only standalone
pub copy is **d60's `tile_draw_particles.rs:442`** (tr-app). The
mechanical keep-one: promote d60's copy to `tr-render` (beside
`animated_tiles`, no tr-core dependency — it takes `&Tile`), d56's
gate calls it, g31/pylons rebind. §1.4's "lowest crate, already pub"
rationale holds; the NAMED survivor changes.

---

## 5. The g68 drip co-lane (§8.5, requested)

g68's deferred-4 (`07396db1c`) ported the ONE unclaimed emission body:
`EmitLiquidDrops` (`TileDrawing.cs:7594-7674`) as
`foliage.rs::emit_liquid_drops` (UNWIRED, 3 witnesses). Its census
confirms the ownership map this document folds against: `:7779-7891`
is exactly `EmitTreeLeaves` (g68, landed); petals/herbs/chimney/star
clouds/pumpkin/torch/rubble are d60's; vision dust is d56's; the
special-point collection is g30's (`liquid_drop_cells` recorded).

**The drip seam (added to §8 as §8.5):** the `Draw` switch's
`:688-694` arm (kinds 373/374/375/461/709 → `EmitLiquidDrops(i, j,
tile, type)` then `continue`) is g30's `CellClass::LiquidDrops`. At
the pixel merge, `classify_and_draw_cell`'s LiquidDrops arm calls
`foliage::emit_liquid_drops(world, y, x, kind, water_style,
zone_corrupt, zone_crimson, zone_hallow, &mut rand, &mut main_rand,
&mut gore)` DIRECTLY (C5: the C# emits inside the switch — the
`Next(num*2)` gate draws on `_rand` BEFORE the cell's `continue`, in
row-major cell order; draining the recorded cells later would
re-order the stream). Inputs: the gate stream is the ONE `_rand`
(the `FastRandom` param adapts), `Main.rand` + the gore pool are the
seams impl's (g68 already threaded them for the leaf path), the
water style is the caller's `waterStyleOverride`-resolved style, the
three zone bools are `Main.SceneMetrics.Zone*` (the seams impl holds
the scene metrics). The wet-tile refusal precedes the roll (no
draw); the anti-cluster box scans the 600 gore slots. Cadence: the
g68 deferred-2 contract applies (emission out of the cached range,
`DrawnThisUpdate` mechanism, never widen G6). The recorded
`liquid_drop_cells` stays as the shadow-plan census input.

**Cadence contract (g68 deferred-2, binding on F2/F3):** the three
C# emission sites have three cadences — DrawSingleTile's throwaway
quarter-roll (5-draw `TileFrameSeed` cycle), the grass passes' real
`_rand.Next(4)` per special-point tile, `EmitTreeLeaves`' bare gate.
The port's per-rebuild approximation is recorded; the pixel merge
moves emission into a per-frame pass (the `tile_cracks.rs`
precedent, `draw.rs:1198-1206`), makes `_rand` PERSISTENT (C# `:191`
— one UnifiedRandom for the renderer's lifetime), and adds the
`DrawnThisUpdate` flag (cleared per world tick `Main.cs:17747`, set
after the first draw `:62478`). The G6 key is NOT widened.

---

## 6. What this document does NOT change

No lane code was touched; no cargo/tests/rustfmt/graphify invoked
(code-only). The §8.3 step-3 ordering correction (§0), the C7 move
obligation, the C4 `from_world` rebind, the §4 `is_visible`
correction and the §5 drip seam are FOLD-TIME obligations recorded
here — the contract's §8.2 signature table is unchanged (every bind
above fits the executed seam shapes).

---

## 7. FINAL-phase addendum (executed on task/W4-g31, code-only)

The g31 TILEPASS final phase executed the items whose files live on
that branch (full record: UNION-CONTRACT §9, commits `d3b5d5fd9` /
`cb70b15d1` / `10530fd88`):

* **§4 `is_visible` correction EXECUTED** — promoted to
  `tr-render/src/tile_visibility.rs` with PRIMITIVE params (kind,
  frame_y, invisible_block, show_invisible_blocks), not the `&Tile`
  this section sketched: tr-render has no tr-core dependency and
  d56's `tile_draw_gate` already takes the same primitives. The
  invisible bit is a parameter (d60 passes `false`; the g31/pylons
  wrappers — rebound, bodies deleted — pass the real
  `World::invisible_block` bit). d56's gate / d60's battery rebind at
  the merge.
* **§3 717 bind EXECUTED as a shadow plan** —
  `tile_attach.rs::lava_glows`, one entry per drawn 717 cell at CAD
  step 14, `Color(v,v,v,v/2)` truncation of `lava_light_a`. Asset
  census answered: `Glow_362.xnb` EXISTS but is NOT in f15's
  `GLOW_SHEET_NAMES` set (as §3 predicted — the table-pass masks are
  a different census); the load + push + `:971` coupling + G6 clock
  keying stay the table-pass lane's obligations.
* **§5 drip seam — g68 tip MOVED**: origin is now `d247a1c9f`
  (wiring-1 `ea9e784f6` … wiring-4), past this document's `b20480d72`
  read. `emit_liquid_drops` is WIRED on g68's branch via
  `emit_liquid_drops_area`'s per-frame rescan + the persistent
  `App.tile_draw_rand`; its signature is unchanged from §5's sketch
  (verified by full `git show` read). The g31 side executed the
  direct-bind ORDER plan (`tile_attach.rs::drip_plan`, row-major,
  witnessed equal to `reg.liquid_drop_cells`); the exact 2-line pixel
  hunk + the "g68's rescan call DROPS at the bind" obligation are
  published in UNION-CONTRACT §8.5.
* **C2/C6 on-branch census**: NO `emit_living_tree_leaf` copy exists
  on the g31 branch (nothing to strip); the two on-branch
  `solid_tile` twins are identical-body (keep-one stays a merge
  obligation). C1/C3/C4/C7 are off-branch pairs — unchanged.

---

## 8. RETIRE-phase addendum: the g84 multitile-mapping answer

Requested: g84's grass/vines inline-arms vs this branch's
DrainPass-buckets seam-mapping. **Full per-kind mapping table +
the five divergences (D1-D5) + the helper keep-one list:
UNION-CONTRACT §8.6** (published there because it is a merger
instruction, not a d-hunk note). Summary of the verdict:

* g84 latest tip `8717e9384` (roster `05de8eb59` + vines bucket
  `b072ddeb7` + docs) ports the SAME `:8738-9032` / `:8829-9603`
  bodies as INLINE fused-walk arms with per-cell frame-grid top-left
  reconstruction — the pre-union stand-in, g28-registry class.
* EVERY g84 inline arm (grass roster incl. the pre-g84 sunflower,
  the 16-kind vine bucket, the `draw_tile_flames` bucket-gate) DROPS
  at the pixel merge; the g30 classify → bucket 2/3 → g31
  `drain_one_pass` path owns them. The g84 arms omit the per-cell
  `_rand.Next(4)` rolls (`:8974`/`:9485`) by design — keeping them
  alive beside the drains double-draws or drops rolls (§8.3 step 4).
* Divergences where the drain is the faithful copy: D1 (530/705
  single-tile halves — g84 sways them, C# `break`s), D2 (519
  per-cell `.max(1)` climb), D5 (jar flame seed — g84 forces a
  non-zero `frame_seed_for` latch; C# jar arms return seed 0).
  D3 is the reverse: g84's `registry::rare` lid feed is closer to
  C# than the drain's `(0, false)` deferral — adopt as the stand-in
  until d57's TeItem store binds. D4: the drain's 698 `num9` should
  read the dd-corrected frameX at the bind (equal today).
* KEEP from g84: the `Extra_260`/`Glow_294` asset loads (the drain
  plans need them), the sim/data co-hunks, the roster census. The
  two g84 render modules DROP wholesale; their 19 witnesses re-pin
  onto the drain bodies (§8.6 helper keep-one list — the six
  body-identical helper twins keep-one onto `tile_drains.rs`).
* The d56/d57/d60 lanes g84's module docs defer to are exactly the
  §1 hunk tables above — no new seam obligations surfaced.

### 8.1 d56-d60 latest-tip re-verify (RETIRE phase)

`git rev-parse` of all five origin tips at the retire read:
d56 `5d80b38a0` / d57 `70f43aa72` / d58 `a13635dcb` /
d59 `f70498be0` / d60 `77429fe1d` — **byte-identical to the §1
census tips (the 136-read). NO lane advanced.** f15 `8ca19097d`
unchanged too; g68 `d247a1c9f` matches the FINAL-phase §7 read
(the only tip that moved since the original census, already
recorded). **Delta-note: NONE for d56-d60** — every §1 hunk table,
§0 order correction, §2 fold script and §4 re-pin map stands
verbatim. Spot re-verification of the two WIRED lanes' move-targets:

* **d58 move-instructions still exact**: the tip's `tiles.rs`
  carries `let mut chest_positions` (the per-draw HashMap, :599),
  the `is_basic_chest(tile.kind)` gate inside the dd block (:756)
  with ONE `let adj = tr_render::chest_frame_adjust` call (:765) —
  the F0 MOVE to CAD step 8 and the C7 order-witness re-pin (§4)
  apply unchanged. The f15 marker rename (`let (dd, glow) = if
  active`, f15 tiles.rs:653) is confirmed still present at f15's
  tip — the merged d58 witness must scan THAT marker.
* **d59 move-instructions still exact**: the tip's dispatch chain
  `tile.slope() != 0 && !SET_DONT_DRAW_TILE_SLOPES` (:1020) →
  platform + `draw_platform_side_fill` (:1045) →
  `SET_HAS_SLOPE_FRAMES` (:1049) → columns → `half_brick_join`
  (:1071) → `push_half_brick_join` (:1075), helpers at
  :1646/:1712/:1750/:1794 — the F0 wholesale replacement of the
  fused `else if tile.slope() != 0` arm and the helper move-list
  are unchanged. `FillNeighbor::from_world` (:179, the real
  `World::invisible_block` bit) + its witness (:725) confirmed —
  the C4 rebind target exists exactly as §2 records.

### 8.2 §9 owner-lane receipts (C1/C3/C4/C7 off-branch dispositions)

The FINAL-phase §9.1 table left four pairs as off-branch merge
obligations. Owner-lane receipts (tips re-read at retire, all
unchanged — obligations STILL OPEN, none executed by their owners):

| Pair | Owner lane / tip | Receipt (read evidence) | Status at retire |
|---|---|---|---|
| C1 (d57 TE-post vs d60 petals) | d57 `70f43aa72` / d60 `77429fe1d` | NO overlap verdict stands (§2): petals = tile 139 inside the battery chain, TE-post = `PostDrawTiles(true)`; d57 zero-RNG census unchanged | CLOSED as a question; no merge action needed |
| C3 (d56 gate vs f15 dark-merge) | f15 `8ca19097d` | f15's tiles.rs still carries `let flame_tile = … \|\| glow.is_some()` (:799-802) + its witness asserting the term (:1925) — the fused gate NOT yet reborn as `tile_draw_gate(has_glow_texture, glow_mask_id)` params | OPEN — fold obligation: feed f15's record into d56's gate params, DELETE the fused term, re-pin the witness as the gate-argument assertion (§4); the 717 half binds `glow_mask_id: 362` together with the table-pass bind (§9.3) |
| C4 (d59 vs f15 side-fill inputs) | f15 `8ca19097d` | f15's `tile_glow.rs::neighbor` (:55-58) STILL maps `FillNeighbor::from_tile` (invisible bit false); d59's `from_world` canonical confirmed at its tip | OPEN — one-line fold fix: `neighbor()` rebinds to `FillNeighbor::from_world` (the C# reads the real bit at both sites) |
| C7 (d58 chest position) | d58 `a13635dcb` | the wired rewrite still sits INSIDE the dd block (:756, earlier than C# `:993`); its order witness still pins the dd-block markers | OPEN — F0 MOVE to CAD step 8 (never re-derive), witness re-pin per §4 with f15's `let (dd, glow)` marker |
| `is_visible` rebinds (the executed keep-one's off-branch halves) | d56 `5d80b38a0` / d60 `77429fe1d` | d56's gate still INLINES the logic (`animated_tiles.rs:489-503`); d60's standalone `pub fn is_visible(tile: &Tile, …)` still at `tile_draw_particles.rs:442` | OPEN — both rebind to `tr_render::tile_visibility::is_visible` (primitive params) at the merge; d60's copy DROPS (its witness folds), d56's gate CALLS it |
| g68 rescan drop (C5's off-branch half) | g68 `d247a1c9f` | `emit_liquid_drops_area` (:273) still WIRED via the frame/draw.rs rescan + persistent `App.tile_draw_rand` | OPEN — at the §8.5 bind, the rescan call DROPS (the CAD LiquidDrops arm owns emission); the persistent rand becomes the ONE `_rand` |

**Final-table verdict: the retire phase found ZERO new lane movement
and ZERO stale instructions.** The §8.3 wiring script, the F0-F4
fold order and the C-pair dispositions are merge-ready as written;
the only additions are §8.6's g84 mapping (new lane, new drop-list)
and this receipt table.

---

## 9. SEAL-VERIFY receipt (this branch, code-only — the bible FINAL)

The five-item seal brief re-verified against the CURRENT pushed tips
(`git fetch --prune` at seal time) and the C# windows. Every item
CONFIRMED; two co-lane tips moved since earlier reads — both deltas
receipted below with ZERO prescription change.

### 9.1 The 85-witness seal (retire tip `9d1e331db` = HEAD at seal)

`git rev-parse HEAD origin/task/W4-g31-tile-draw-pass-4` — both
`9d1e331dbc0df49a94d1ae81b1863f8704bc19c2` (the MERGE-READY retire tip,
pushed). Fresh `#[test]` grep census at HEAD: tile_pass 18 +
special_points 14 + tile_drains 22 + tile_draw_pass 10 +
quality_light 6 = the g-family **70**; tile_ctor 6 + tile_attach 6 +
tile_visibility 3 = **85 total**. Stronger than the counts: all **85
NAMES** of UNION-CONTRACT §10.2 compared set-wise against the actual
`fn` names under each `#[test]` — **zero missing, zero extra** (the
pinned list is exact, not just the totals). §8.3's wiring script and
§10's acceptance checklist stand verbatim.

### 9.2 g83 keep-one hunk-pair (tip `6dd967a2a`, research/123 §9.1)

g83's origin tip is `6dd967a2a` (the seaweed stream-twin fixture fix —
AFTER the §9.1 doc commit; the hunk-pair contract is unchanged by it).
Read evidence at the tip:

* **Hunk A LANDED**: `vines.rs::rising_vine_strip_segments` (:400-407)
  carries the 6th `show_invisible: bool` param; the per-segment push
  reads `visible: is_visible(world, x, y, show_invisible)` (:478);
  `draw_reverse_vines` (:521-533) threads the param (:537); the
  interim fused-pass caller passes `show_invisible_blocks`
  (`tiles.rs:1216` call, seam-4 comment). The new witness
  `w4_g83_the_rising_visible_reads_the_full_is_visible` (:1051) pins
  the sight override + platform-row-48 + 541/631 arms; vines.rs census
  21 `#[test]` as §9.2 of 123 records.
* **Byte-identity VERIFIED**: g83's `is_visible` body (:499-510) vs
  g31's pre-promotion copy at `766a5a5a` (tile_drains.rs :181-192, the
  tip g83 read) — **case-sensitive line-for-line BYTE-IDENTICAL**
  (12 lines). g31's HEAD copy is now the thin wrapper over
  `tr_render::tile_visibility::is_visible` (the FINAL-phase promote);
  the promoted fn's arms are the same logic (`19 if frame_y/18==48`,
  `541|631`, `!flag || show_invisible` ≡ `if flag { return show } true`).
* **Hunk B (g31-side rebind) re-pinned to HEAD coordinates**: 123 §9.1
  quotes the `766a5a5a` base — at HEAD the struct is :528-544 (was
  :525-547), `rising_vine_strip` :553-624 (was :550-622),
  `drain_reverse_vines` :628-639, `is_visible` :185-195 (was :178-191).
  The script is UNCHANGED in substance: DELETE the strip fn + struct,
  import g83's (`RisingVineSegment` field-identical, g83's carries the
  `seam_top` doc — KEEP g83's), rebind the drain body, the 2 strip
  witnesses (`w4_g31_rising_vine_strip_walks_up` :1891 /
  `_breaks_on_non_thread` :1913) rebind with the trailing
  `show_invisible` arg. Step 3's "both copies identical" note updates:
  g83's copy stays byte-identical to the PROMOTED semantics; at the
  family merge g83's `is_visible` rebinds onto
  `tr_render::tile_visibility` like d56/d60 (the §1.4 keep-one), or
  keeps its world-reading wrapper form — either binds zero-conflict.
  Step 5 re-verified: g31's `drain_draw_data` == g83's inline
  `draw_data(kind,x,y,fx,fy,0,solid_above)` + half-brick probe
  (same `TILE_SOLID`/`TILE_SOLID_TOP` gate, same 8/0 half).

### 9.3 Glow_362 load obligation (d35 tip `1afd327b9`, research/113 §16.3)

* Asset re-verified INDEPENDENTLY: `assets/Content/Images/Glow_362.xnb`
  EXISTS — XNBw v5, flags 0x80 (LZ4), 7592 bytes compressed,
  decompressed-size header 311217 = 177 header + 288×270×4 —
  **288×270 = 77,760 px (78.9 kpx padded)** confirmed from the file
  header, not just the doc.
* `Main.tileGlowMask[717] = 362` re-read at `Main.cs:10056` — exact.
* f15 tip `8ca19097d` unchanged; its `assets.rs` `GLOW_SHEET_NAMES`
  (:1727) has **zero** `Glow_362` hits — the load obligation stands.
* d35's measured +8 px: `tools/_d35_retire_measure.json` at its tip
  carries `"scenario": "lazy-union-worst+Glow362", "skyline": 7313`
  vs the 7305 baseline — **7305→7313 (+8)** as 113 §16.3 records; the
  plan's "owes g31" admission + the `:971` gate coupling (C3) stand.
* g84 keep-one re-verified at its NEW tip `e6c2dcd0d` (was `8717e9384`
  at the retire read): the move is **docs-only** (`git diff --stat`
  8717e9384..e6c2dcd0d = ANALYSIS/PORT_MAP/research 09+137/INDEX, zero
  crates/ files) — `assets.rs:498 "Extra_260"` + `:1739 "Glow_294"`
  intact, the §8.6 mapping/drop-list UNCHANGED. g84's new research/137
  is its own keep-one prep read of g31 `766a5a5a`; consistent with
  §8.6, no new obligation.

### 9.4 c29 gemsparkle + rainbow-moss-627 (tip `29c3ff2b5`, research/130)

* **178 gemsparkle = d60-claimed tileShine — CONFIRMED at both ends.**
  C#: `Main.tileShine[178] = 500` (`Main.cs:7516`) +
  `tileShine2[178] = true` (:7517); `[63..68] = 900` (:7523-7528),
  `[566] = 900` (:7529), `[211] = 500` (:7549). d60 tip `77429fe1d`:
  `tile_shine` table `178 | 211 => 500` (:243),
  `63|64|65|66|67|68|566 => 900` (:246); the `shine_arm` 178
  style-colour match (:1532-1545) has the `1 | 6` yellow fold, the
  magenta/blue/green/red/white columns, dust 43 + `velocity *= 0`,
  the dark-light `<= 20` early return (:1501) and the
  `Next(tileShine)`-BEFORE-frame-windows order (:1506) — line-for-line
  the C# `:7413-7482` re-read. The 63-68/566/12/665/204/639/211
  override chain (:1551-1559) matches `:7483-7526`. NOT light
  emission (130's refuted AddLight premise stands). The 178 shine
  rides CAD step 3 (d60's battery) — g31 has no re-port obligation.
* **rainbow-moss 627 → the table-pass bind contract — CONFIRMED.**
  C# re-read: `Main.tileGlowMask[627] = 313` (`Main.cs:10041`), the
  colour switch `case 627: case 628: case 692:` →
  `new Color(Main.DiscoR, Main.DiscoG, Main.DiscoB)`
  (`TileDrawing.cs:1235-1239`), inside the `:1178-1342` pass — this
  doc's §0 row 14. g35's `glowmask.rs` carries `(627, 313)` (:52) +
  witness (:110) at its NEW tip `c5bbe55b0` (was `fd0a99f6` at c29's
  read): the delta is the ITEM/PROJ/NPC glow-record census +
  GenItemData interpreter fix — `git diff fd0a99f6..c5bbe55b0 --
  crates/tr-core/src/glowmask.rs` is EMPTY, the tile table unchanged.
  The disco colour switch is STILL UNPORTED — it lands with the
  table-pass lane under the SAME §3/row-14 bind contract as 717
  (the generic `!flag3` tail `:1291-1340`; 627 sets no flag3, no
  texture override). The port's `drawdata_cases_a.rs` disco parking
  (column 220, :344-345/:1110) is the SECOND-switch record lane —
  a different site, no conflict. 130's Item-5 verdict (no
  TileObjectData def; framing/light/map/sweep landed) stands.

### 9.5 Seal verdict

All five items CONFIRMED with read evidence at the current tips.
Lane-tip register at seal: d56 `5d80b38a0` / d57 `70f43aa72` /
d58 `a13635dcb` / d59 `f70498be0` / d60 `77429fe1d` / f15 `8ca19097d` /
g68 `d247a1c9f` — ALL unchanged since §8.1; g83 `6dd967a2a` (hunk-A
landed, hunk-B open as scripted); c29 `29c3ff2b5`, d35 `1afd327b9`
(the cross-receipts); g84 `e6c2dcd0d` + g35 `c5bbe55b0` (the two
moves, both receipted above — zero prescription change).
**research/136 + UNION-CONTRACT are SEALED — the tilepass-family
bible is FINAL.** The merger executes §8.3 + this doc's F0-F4 +
§9.1's C-dispositions + UNION §10 as written; the only open items
are the recorded merge obligations themselves (C3/C4/C7, the
is_visible/g83 rebinds, the g68 rescan drop, the Glow_362 load +
table-pass bind incl. the 627 disco arm).
