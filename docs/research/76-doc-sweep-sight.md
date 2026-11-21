# 76 — DOC-SWEEP lane 4: the remaining sight-mechanics census (W4-g78)

> **SUPERSEDED IN PART by research/77 (the wiring phase, 2026-11-21):** the
> Hunter NPC-tint chain's 14 debuff arms + the behind-tiles black-out scan
> are now WIRED at the uncontested `entities.rs` colour site (the G6
> `npc_key` widened); the spelunker helper carries the d9-canonical bitset
> + the attach-ready AI arms. The still-UNWIRED halves (the spelunker
> dispatch, the Hunter-floor threading, the 657 use/break arms) have EXACT
> attach hunks published in research/77 for the contested-seam merger. The
> census below stands as the C# evidence.

> W4 lane g78 · DOC-SWEEP phase · 2026-11-21 · sources: decompiled (Main.cs,
> Player.cs, NPC.cs, TileDrawing.cs, WorldGen.cs, Wiring.cs, Item.cs,
> SpelunkerProjectileHelper.cs, TileObjectData.cs, ID/{TileID,NPCID,ItemID}.cs)
> + crates (tr-app player.rs / buff_effects.rs / npc_buffs.rs /
> render/{tiles,entities}.rs / world_sim/editing{,/doors}.rs / registry.rs /
> itemdata.rs, tr-core scene.rs / edit/{place,kill}.rs / wiring.rs /
> tileobject{,/registry}.rs / tiledata/sets.rs, tr-render animated_tiles.rs)
> · base `15049271` (the g78 echo chain) · code-only mode (no cargo/tests/graphify)

## Scope

The four potion sight-buffs (Spelunker 9 / Hunter 17 / Dangersense 111 /
Biome Sight 343) and the two Echo-sight items (Echo Monolith 5345 / tile 657,
Spectre Goggles 4409) — the visibility mechanics the g78 echo-coating lane
(research/179) did NOT cover. g78 landed the `CanSeeInvisibleBlocks` chain
(the monolith/goggles grant → `ShouldShowInvisibleBlocksAndWalls`); THIS
sweep mines the OTHER sight flags, their draw consumers, and the
monolith/goggles item-use + equip seams. Co-lanes: **d56** landed the three
tile predicates (`is_tile_dangerous`/`is_tile_spelunkable`/
`is_tile_biome_sightable`) + the vision-floor byte maths in
`tr_render::animated_tiles` (UNWIRED); **f47** landed
`shimmer::appears_friendly_to_hunter_potion` (UNWIRED); **f12** wires the
`DebuffEffects` accumulator (incl. `danger_sense`) onto the live player.

## The four sight flags (C# → port)

| Buff | C# field / grant / clear | Port | Verdict |
| --- | --- | --- | --- |
| Spelunker 9 | `findTreasure` `Player.cs:2786`; grant `:9981-9984`; clear `ResetEffects :18370` | `player.rs::find_treasure` (grant `:6498`, clear `:6010`) | **PORTED+WIRED** (flag) — the DRAW consumer is d56's vision floor (below) |
| Hunter 17 | `detectCreature` `:2792`; grant `:10021-10024`; clear `:18380` | `player.rs::detect_creature` (grant `:6514`, clear `:6020`) | **PORTED+WIRED** (flag) — the NPC-tint consumer ported HERE (mechanic 2, UNWIRED) |
| Dangersense 111 | `dangerSense` `:1132`; grant `:10189-10192`; clear `:18496` | `buff_effects.rs::DebuffEffects.danger_sense` (`:60`/`:171`) — the PURE accumulator; f12's `buff_frame_wiring.rs:502` folds it onto `Player.danger_sense` | **PORTED-UNWIRED** (f12 owns the live-field wiring) — the DRAW consumer is d56's vision floor |
| Biome Sight 343 | `biomeSight` `:2788`; grant `:9985-9988`; clear `:18371` | `player.rs::biome_sight` (grant `:6500`, clear `:6011`) | **PORTED+WIRED** (flag) — the DRAW consumer is d56's vision floor |

The potion ITEMS all exist and grant the right buff: Spelunker Potion 296
(`buff_type 9`, `buff_time 18000`, `Item.cs:4914-4927`), Hunter Potion 304
(`17`/`28800`, `:5026-5039`), Trapsight/Dangersense Potion 2329
(`111`/`36000`, `:23373-23386`), Biome Sight Potion 5211 (`343`/`18000`,
`:41132-41144`) — `itemdata.rs` rows verified 1:1 (use_style 9, use_time 17,
consumable, value 1000 except 5211's `sellPrice(0,0,2)` = 1000). The drink
path (`potions.rs` / `use_potion_item`) is the S1-P-i37 lane, already landed.

## The tile predicates + the vision floors (d56's lane, read not re-ported)

`DrawSingleTile` (`TileDrawing.cs:897-963`) and
`DrawAnimatedTile_AdjustForVisionChangers` (`:8481-8552`, called from
`DrawGrass :8348`, `DrawAnyDirectionalGrass :8405`, `DrawMultiTileVinesInWind
:8976`, `:9487`) apply THREE byte floors to `tileLight` in C# order, each
behind its flag + predicate:

| Arm | C# site | Floor | Dust roll |
| --- | --- | --- | --- |
| Dangersense | `:897-918` / `:8483-8505` | `IsTileDangerous` → R≥255, G≥50, B≥50 | `(_isActiveAndNotPaused & canDoDust) && Next(30)==0` → type 60, alpha 100, fadeIn 1, vel×0.1, noLight, noGravity |
| Spelunker | `:920-937` / `:8506-8523` | `IsTileSpelunkable` → R≥200, G≥170 (B untouched) | `_isActiveAndNotPaused && ((Next(60)==0) & canDoDust)` → type 204, alpha 150, fadeIn 1, vel×0.1, noLight (the bitwise `&` ROLLS even when dust suppressed) |
| Biome Sight | `:938-963` / `:8524-8551` | `IsTileBiomeSightable` → R/G/B ≥ sightColor | `(_isActiveAndNotPaused & canDoDust) && Next(480)==0` → type 267, alpha 150, newColor=sightColor, fadeIn 1, vel×0.1, noGravity, noLightEmittance |

The predicates (d56 `tr_render::animated_tiles`, UNWIRED):
- `IsTileDangerous` `TileDrawing.cs:4435-4467` — the unconditional list
  (135/137/Boulders/141/210/442/443/444/411/485/85/654/314-pressure-plate/
  getGoodWorld-230/58) + the flat-and-active group (TouchDamageImmediate
  minus cactus-80-outside-DontStarve, 483/482/481/51/229, TouchDamageHot
  cleared by fireWalk, thin-ice 162 gated `!iceSkate`).
- `Main.IsTileSpelunkable` `Main.cs:21783-21798` — the 49-row `tileSpelunker`
  table (`:9420-9467`+`:9651`) + the 185 (frameY 18, frameX 576..=882) / 186
  (frameX 864..=1170) windows.
- `Main.IsTileBiomeSightable` `Main.cs:21800-21818` — CorruptBiomeSight
  (200,100,240) / CrimsonBiomeSight (255,100,100) / HallowBiomeSight
  (255,160,240), the remixWorld 474→corrupt / 195→crimson exceptions. The
  three sets are ALSO generated in `tiledata/sets.rs`
  (`SET_CORRUPT_BIOME_SIGHT`/`SET_CRIMSON_BIOME_SIGHT`/`SET_HALLOW_BIOME_SIGHT`,
  `TileID.cs:578/:587/:582`) — d56 inlined them in tr-render (no tr-core dep);
  at the wiring merge the two should share one copy.

**Consumer census (the vision floors):** `render/tiles.rs::draw_tiles` does
NOT call any vision floor today — the per-cell `light` is the raw
`style.shade(light.get(...))`. The `DrawMultiTileGrassInWind` biome-sight arm
(`:9089-9114`) is the same floor on the sunflower/multi-tile pass. ALL of
these are **MISSING** in the port and belong to d56's wiring merge (the
`tiles.rs` seam is contested by d57/d58/d59/d60/g28-g34/g77/g79/g80/g84). The
g78 echo lane already threads `show_invisible_blocks` through `draw_tiles`;
the vision floors need the four flags + the world seed gates
(getGoodWorld/dontStarve/remix) + the player's iceSkate/fireWalk threaded the
same way — no new seam, the d56 module is the decision maths.

## Mechanic 1 — SpelunkerProjectileHelper (PORTED here, UNWIRED)

The Spelunker Glowstick (473, aiStyle 14) and the fairy/pet spelunker scans
call `Main.instance.SpelunkerProjectileHelper` — a SEPARATE ore-spot dust
layer from the vision floors. `projectile_ai_light_items.rs::glowstick_head`
(`:157-165`) keeps the 473 10-tick `localAI[0]` counter but documents the
`AddSpotToCheck` call as a skip ("no ore-highlight layer"). This sweep ports
the helper itself: `crates/tr-app/src/spelunker_helper.rs`.

| C# site | Behaviour | Port |
| --- | --- | --- |
| `OnPreUpdateAllProjectiles` `SpelunkerProjectileHelper.cs:16-25` | `_clampBox = Rectangle(2,2,maxTilesX-2,maxTilesY-2)`; every 10th call clears both sets | `on_pre_update_all_projectiles` (the box derived per-check from world size) |
| `AddSpotToCheck` `:27-33` | `_positionsChecked.Add(spot)` → new spot runs CheckSpot | `add_spot_to_check` (the f32-bit-keyed HashSet) |
| `CheckSpot` `:35-66` | ±30 window clamped to the box, x-outer/y-inner; active+spelunkable+`!(Length()>30f)`+`_tilesChecked.Add && Next(4)==0` → 204 dust (fadeIn 0.75, vel×0.1, noLight) | `check_spot` (the `SpelunkerDust` request on the caller's DropRng) |
| the 473 head `Projectile.cs:24836-24847` | the counter wrap → `AddSpotToCheck(base.Center)` gated `(Center-player.Center).Length() < screenWidth+30*16` | the distance gate stays with the caller (C# keeps it outside the helper) — UNWIRED |
| `Main.PreUpdateAllProjectiles` `Main.cs:17535-17539` | calls `OnPreUpdateAllProjectiles` once per projectile pass | UNWIRED (the app projectile-update head) |

The OTHER two spelunker scans are already ported in their own lanes (NOT
re-ported here): aiStyle 90's 17-tile `Next(4)` scan (`Projectile.cs:32411-
32434`, d9 `projectile_ai_style90.rs`) and aiStyle 124's 17-tile `Next(8)`
nearest-ore scan (`:35571-35609`, d13 `projectile_ai_pets124.rs`) — both
inline their own `is_tile_spelunkable` copy (the keep-one note: d56's
tr-render copy + this sweep's tr-app copy + d9/d13's inline copies should
collapse to one at the wiring merges).

## Mechanic 2 — the Hunter NPC-tint chain (PORTED here, UNWIRED)

`Main.DrawNPC` calls `npcColor = rCurrentNPC.GetNPCColorTintedByBuffs(npcColor)`
(`Main.cs:22939`) AFTER the behind-tiles black-out scan (`:22922-22938`).
The chain (`NPC.cs:91024-91141`) is 14 debuff arms with COMPOUNDING
multipliers, then the Hunter floor. This sweep ports it:
`crates/tr-app/src/npc_hunter_tint.rs` (`npc_color_tinted_by_buffs` +
`buff_color`).

| C# site | Behaviour | Port |
| --- | --- | --- |
| `buffColor` `NPC.cs:90986-90993` | `(byte)((float)(int)c * m)` per channel (truncate toward zero) | `buff_color` |
| `GetNPCColorTintedByBuffs` `:91024-91141` | the 14-arm chain + the hunter floor | `npc_color_tinted_by_buffs` (the `NpcTintFlags` input) |
| the hunter arm `:91110-91139` | `CanApplyHunterPotionEffects() && lifeMax>1` → friendly (50,255,50) / hostile (255,50,50) per-channel max | the `hunter_active && life_max_gt_1` gate + the `hunter_friendly` pick |
| `CanApplyHunterPotionEffects` `:91019-91022` | `Main.SceneMetrics.PerspectivePlayer.detectCreature` | the caller passes `hunter_active` |
| `AppearsFriendlyToHunterPotion` `:91008-91017` | `friendly \|\| catchItem>0 \|\| (damage==0 && lifeMax==5)`, the 689→false override (`NPCID.cs:10128-10132`) | **f47's** `shimmer::appears_friendly_to_hunter_potion` (KEEP-ONE — this module takes its result as `hunter_friendly`) |
| the draw-order swap `Main.cs:63290-63299` | `detectCreature` moves `DoDraw_DrawNPCsBehindTiles` AFTER `DoDraw_Tiles_Solid` (so hunter-lit NPCs show through tiles) | UNWIRED (the `render/tiles.rs` draw-phase order, contested) |

**Consumer census:** `render/entities.rs::draw_npcs` (`:2148`) computes
`c = style.shade(l, tx, ty, anim_time)` and never calls the tint chain — the
whole `GetNPCColorTintedByBuffs` family (midas/ichor/poison/… tints AND the
hunter floor) is **MISSING**. The consumer is contested by the npc-kind/style
lanes (c10/c4/c6/c7/c9/d126/d127/d34/d35/d38 all touch `entities.rs`); the
wiring lands with their merge. The `NpcTintFlags` input folds the live
`UpdateNPC_BuffSetFlags` flags (`npc_buffs.rs`) — most are already ported
there (poisoned/venom/bleeding/hemorrhage/oiled/dripping_slime/…); midas 72,
ichor 69, betsysCurse, stinky, brokenArmor, chlorophyteSpore, potentAcid,
drippingSparkleSlime are the unported-flag remainder (the buff-wave lanes).

## Mechanic 3 — Dangersense / Biome Sight consumers (census, d56/f12 lanes)

Both flags' DRAW consumers are the d56 vision floors (table above) —
**MISSING** in `render/tiles.rs`, d56's wiring merge. The Dangersense FLAG
itself is the f12 wiring (`buff_effects.rs::danger_sense` → the live
`Player.danger_sense` field f12 adds at `player.rs:2079`); this sweep does
NOT re-port it (f12 owns the live-field seam, and `player.rs` is contested by
53 branches). The Biome Sight flag is already live (`player.rs::biome_sight`).
No new code here — the census records the seam so the d56/f12 merges wire the
real values (the g78 echo lane's `show_invisible_blocks` threading is the
precedent).

## Mechanic 4 — the Echo Monolith (tile 657 / item 5345) use + place + break

| C# site | Behaviour | Port | Verdict |
| --- | --- | --- | --- |
| `Item.cs:42406-42414` case 5345 | `DefaultToPlaceableTile(657,0)`, 22×32, rare 9, `sellPrice(0,1)`=50000, accessory+vanity | `itemdata.rs:5511` (tile Some(657), accessory+vanity, value 50000) | **PORTED** (verified) |
| `TileObjectData.addTile(657)` `TileObjectData.cs:2907-2912` | Style2xX h3, origin (1,2), CoordinateHeights [16,16,16], DrawYOffset 2, UsesCustomCanPlace | `tileobject/registry.rs::D_657` (`:23580`, uses_custom_can_place true) | **PORTED** — placement routes through TileObject (witness added, place.rs) |
| the place route `Player.cs:39771` | CustomPlace → `TileObject.CanPlace`/`Place` | `edit/place.rs::place_item_tile` → `routes_through_tile_object(657,0)` | **PORTED+WIRED** (witness: the 2×3 body, frameY 0/18/36 unlit) |
| the right-click use `Player.cs:32271-32277` | tile 657 (the monolith family 410/480/509/657/658/720/721/725/733) → `PlaySound(28)` + `GamepadEnableGrappleCooldown` + `WorldGen.SwitchMonolith(myX,myY)` | `edit.rs::interact_at` has NO monolith arm (657's `Interaction::None` falls to StationInteract) | **MISSING** — the use arm is unported; `interact_at`/`doors.rs` contested (c42/e20/e21/f11/f34/f41/e28/g24/e17/f29/f32/d79) |
| `WorldGen.SwitchMonolith` `WorldGen.cs:51004-51148` | the 2×3 (3×3 for 733) frameY flip: 410 ±56, 480/509/657/720/721/725/733 ±54, 658 +54-wrap-162; the `Wiring.running` SkipWire tail | `wiring.rs::switch_monolith` (`:2843`) — the WIRED path (`HitWireSingle` case 657 `Wiring.cs:2024`) | **PORTED+WIRED** (wiring) — e21 fixes the kind-match (the base port flipped ANY kind in the rect; C# flips only the nine monolith kinds) |
| the break `WorldGen.Check2xX` `WorldGen.cs:37993-38287` | tile 657 in the `TileFrameImportant` case group (`:85738`) → the 2×3 integrity scan + `Item.NewItem(…,5345)` (`:38221-38224`) | `edit/kill.rs::check_object` has NO 657 arm (only 104 rides Check2xX via `check_clock`) | **MISSING** — the 657 break-check is unported; `check_object` contested (c31/g17/g47/f41) |
| `SceneMetrics.EchoMonolith` scan `SceneMetrics.cs:541-546` | tile 657 active, frameY≥54 → true | `scene.rs::scan_on_screen_tiles` 657 arm (`:749`) | **PORTED+WIRED** (g78 consumes it) |

The use arm (`SwitchMonolith` on right-click) and the break check
(`Check2xX` → drop 5345) are the two MISSING halves. Both seams
(`interact_at`/`doors.rs` and `check_object`) are heavily contested — recorded
here, NOT ported (the wiring `switch_monolith` already exists for the use arm
to call; the break check needs a `check_monolith` sibling of `check_clock`).
The `registry::tile_drop(657)` fallback already resolves 5345 via `by_tile`
(the item places 657), so a future `check_monolith` drop record with
`item: None` would still surface 5345 — but the C# spawns it at the ORIGIN
cell `(num, j)` with a 32×32 box, which the per-cell fallback does not model.

## Mechanic 5 — Spectre Goggles (item 4409) vanity vs functional (VERIFIED)

| C# site | Behaviour | Port | Verdict |
| --- | --- | --- | --- |
| `Item.cs:37391-37395` case 4409 | `DefaultToAccessory(28)`, faceSlot 14 (`ArmorIDs.SpectreGoggles`), `buyPrice(0,10)`=100000, Pink5 | `itemdata.rs:4575` (accessory true, vanity false, value 100000, 28×24) | **PORTED** (verified; faceSlot unmodelled — the port's ItemDef has no face_slot, the cosmetic face draw is unported) |
| `ApplyEquipFunctional` case 4409 `Player.cs:14548-14550` | `CanSeeInvisibleBlocks = true` (the 3..10 functional loop `:12946-12952`) | `player.rs::apply_equip_functional` arm 4409 (`:5033`) | **PORTED+WIRED** (g78) |
| `ApplyEquipVanity` `:14197-14218` | NO 4409 case (only 5452/5345) | the 13..20 vanity loop matches 5345 only | **MATCHES** — a vanity-worn goggles grants NOTHING |
| `RefreshInfoAccsFromItemType` `:12721-12797` | NO 4409 row | n/a | **MATCHES** — no info-accessory side grant |

**Verdict: the goggles grant is FUNCTIONAL-ONLY.** The g78 port matches C#
exactly — the witness (`player.rs::w4_g78_doc_sweep_the_goggles_grant_is_functional_only`)
pins the vanity-slot refusal + the functional grant + the item rows. The
Echo Monolith 5345 is the mirror (vanity-only grant, `ApplyEquipVanity` case
5345 `:14215-14217`; a functional-worn monolith grants nothing — the g78
witness `w4_g78_the_vanity_monolith_grants_but_the_functional_slot_does_not`).
The task's "verify goggles item 4409 equip grant both slots" resolves to:
4409 grants in the FUNCTIONAL slot only, NOT both — the C# has no vanity
4409 arm, so "both slots" would be a divergence. No code change needed.

## Witnesses (WRITTEN NOT RUN — code-only)

- `spelunker_helper.rs` (6): `w4_g78_the_spelunker_table_census`,
  `w4_g78_the_spelunker_frame_windows_are_inclusive`,
  `w4_g78_the_spot_dedupe_and_the_next4_gate`,
  `w4_g78_a_nonzero_roll_marks_the_tile_checked`,
  `w4_g78_the_ten_tick_clear_reopens_the_dedupe`,
  `w4_g78_the_thirty_tile_radius_gate` (the scripted-DropRng roll-count pins).
- `npc_hunter_tint.rs` (6): `w4_g78_buff_color_truncates_toward_zero`,
  `w4_g78_no_display_returns_untouched`,
  `w4_g78_the_hunter_floor_lifts_each_channel`,
  `w4_g78_the_hunter_arm_gates`,
  `w4_g78_ichor_replaces_then_hunter_floors`,
  `w4_g78_the_multipliers_compound_in_arm_order`.
- `player.rs` (1): `w4_g78_doc_sweep_the_goggles_grant_is_functional_only`.
- `edit/place.rs` (1): `w4_g78_doc_sweep_the_echo_monolith_places_through_the_registry`.

## RNG / determinism

- The two new modules draw RNG ONLY through the caller's stream: the
  spelunker helper's `Next(4)` per newly-checked spelunkable tile (the
  `Add && Next(4)` short-circuit — a repeat tile rolls NOTHING); the hunter
  tint chain draws ZERO (pure colour maths). No new stream, no fixed-step
  order change, no `WorldFrameKey` change.
- Both modules are UNWIRED — zero pixel delta until their consumers wire.
  The vision floors (d56) and the NPC tint (this sweep) will change pixels
  TOWARD the C# at their merges: spelunker/dangersense/biome-sight tiles lift
  their light floors + emit dust; hunter-lit NPCs floor to red/green and show
  through solid tiles (the draw-order swap).
- The keep-one targets at the wiring merges: the `is_tile_spelunkable`
  copies (d56 tr-render / this sweep tr-app / d9 / d13) collapse to one; the
  `appears_friendly_to_hunter_potion` stays f47's; the `danger_sense` live
  field stays f12's.

## Residuals left to co-lanes (recorded, not touched)

1. **d56** — the vision floors' `render/tiles.rs` wiring (the four flags +
   the seed gates + iceSkate/fireWalk threaded like g78's sight value).
2. **f12** — the `danger_sense` live-field wiring (`buff_frame_wiring.rs`).
3. **f47** — the `appears_friendly_to_hunter_potion` consumer (the shimmer
   transparency gate `CurrentlyShimmerTransparent`).
4. **npc-kind/style lanes** — the `npc_color_tinted_by_buffs` consumer in
   `render/entities.rs::draw_npcs` + the `DoDraw_WallsTilesNPCs` draw-order
   swap; the unported tint flags (midas/ichor/betsysCurse/stinky/brokenArmor/
   chlorophyteSpore/potentAcid/drippingSparkleSlime).
5. **wiring-device lanes (c42/e20/e21/f11)** — the monolith right-click use
   arm (`interact_at`/`doors.rs` → `SwitchMonolith` + the Mech-28 cue); e21
   already fixes `switch_monolith`'s kind-match.
6. **break-check lanes (c31/g17/g47/f41)** — the 657 `Check2xX` break check
   (`check_monolith` → drop 5345 at the origin cell).
7. **d9/d13** — the aiStyle-90/124 spelunker scans' inline predicate copies
   (keep-one with d56/this sweep at their merges).
