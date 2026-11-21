# 102 — DOC-SWEEP lane 23: the lights gaps

> W4-e7 DOC-SWEEP · 2026-11-21 · code-only (no cargo/tests/graphify) ·
> branch `task/W4-e7-addlight-npc-2` · sources: decompiled + crates +
> the in-flight branch claims (f3/d65/d66/d56/e8).

## Scope

The four cited gaps from the W4-e7 remaining-files AddLight census close
(ANALYSIS "The remaining-files AddLight census (W4-e7 close)", the 4 of the
14 Player sites that stayed dropped) plus the lane-23 lighting census
(legacy vs new mode, light-map decay/refresh, torch flicker global). This
note records what landed here, what is owned by an in-flight branch, and
what stays a cited gap with a wiring recipe. Cross-ref: `10-lighting.md`
(the wave-1 lighting audit; its deviations 2/3 are the decay gap below).

**Claim map (probed in-flight before work).**

| Gap | Owner | State on THIS base |
| --- | --- | --- |
| Snapping Stone trio lights (`Player.cs:24575-24583`) | **this lane** | LANDED here (`5ae32127`) |
| wing-27 Shroomite light (`Player.cs:36026`) | f3 stealth producer | cited skip — producer on unmerged `task/W4-f3-dust-player-3` |
| WoF `gross` legacy light block (`LegacyLighting.cs:678-717`) | **d65** (S1-E-19d) | owned — `task/W4-d65-legacy-lighting-wof` (`c8a7e0e80`+) |
| Modern mask `!inActive` + scanner residuals | **d66** | owned — `task/W4-d66-light-scanner-residuals` |
| Per-tile torch flicker (`TileLightScanner` `Next(28,42)`) | **d56/e8** | landed — `tile_light/scanner.rs::torch_lights` |
| Global flicker oscillators (`masterColor`/`martianLight`) | **this lane** | LANDED here (`999b292af`), readers cited |
| Modern `UpdateLightDecay` table (water-style/buff/SceneState) | unclaimed | cited gap — touches d66's active `blur.rs`/`LightKey` + frozen determinism |

## 1. The Snapping Stone trio — LANDED (`5ae32127`)

The three `Player.Update` ready lights (`Player.cs:24571-24585`), gated on
`accSnappingStoneLightUp` (the getter `:3391-3396` = `accSnappingStoneCooldown
== 0`). Each worn member adds its centre light via the `AddLight(Vector2, …)`
quotient-truncating tile form (`Lighting.cs:140-143`):

| Member | Item | Grant (`ApplyEquipFunctional`) | Light rgb |
| --- | --- | --- | --- |
| `accSnappingStone` | 6167 Snapping Stone | `:14494-14496` (bare flag) | `(0.1, 0.25, 0.4)` |
| `accPyroclast` | 6178 Pyroclast | `:14580-14583` (+ `magmaStone`) | `(0.4, 0.25, 0.1)` |
| `accArmletOfRuin` | 6179 Armlet of Ruin | `:14584-14587` (+ `accWickedArmlet`) | `(0.2, 0.25, 0.6)` |

Three independent `if`s (wearing two members lights both). `ResetEffects`
clears the flags (`:18476-18478`); the cooldown is NOT cleared — a countdown
(the `accSentryBackpackGrabCooldown` shape).

**Ported:** `player.rs` — the four fields (`acc_snapping_stone`/`acc_pyroclast`/
`acc_armlet_of_ruin`/`acc_snapping_stone_cooldown`), spawn defaults, the three
grant arms, the reset clears, `acc_snapping_stone_light_up()` and
`update_snapping_stone()` (the `:28385-28410` countdown, wired at the `:24564`
world-sim position in `world_sim/player.rs` before `Update_AmmoCycler`
`:24566`). `player_lights_b.rs::snapping_stone_lights` rides the ONE
`EntityLights` batch (10 → 13 of the 14 sites). 7 witnesses WRITTEN NOT RUN.

**The strike machine stays cited** (the projectile-family gap, NOT ported
here): `Projectile.TryMakingSnappingStoneStrike` (`Projectile.cs:14283-14395`)
arms the 360-tick cooldown (`:14316`) and spawns the 1121/1131/1132 effect
projectiles. It is RNG-bearing — the per-strike `ParticleOrchestrator` spawns
draw `Main.rand` (`Spawn_SnappingStone` `ParticleOrchestrator.cs:593-646`:
20 dust × `Next(3)` + conditional `Next(6)`; `Spawn_SnappingStoneMelee`
`:648-731`: `NextFloatDirection` + 4 dust × 2 `NextFloatDirection`/`NextFloat`;
`Spawn_SnappingStoneSummon` `:733-813`: 2 `NextFloat` + 23 dust ×
`NextVector2Circular`(2) + `Next(6)`), and the proj rows 1121/1131/1132
(`Projectile.cs:10491-10503`/`:10611-10635`: 10×10, friendly, local-immunity
60, timeLeft 2, hide, noEnchantments) have no `proj_data` rows. The three
call sites: melee `Player.cs:44338-44341` (accPyroclast, type 1), projectile
`Projectile.cs:13397-13408` (melee/ranged → type 1, ranged → type 0,
`changes.AddProjectileTagDamage` → type 2). Wiring recipe: add the three
`proj_data` rows + a `try_making_snapping_stone_strike` sim seam (the
`find_target_with_line_of_sight` `:45627-45660` retarget, the 2-strike loop,
the `dmg += num13` bonus, the Item30/Item20/Item43 sounds at `MountedCenter`),
threading the `UpdatePlayers`/`UpdateProjectiles` stream draws in the C#
order. Until then the cooldown stays 0 in play, so the lights fire whenever a
member is worn — the C# behaviour for a never-struck accessory.

## 2. wing-27 Shroomite light — cited skip (f3 owns the producer)

`PlayerFrame` (`Player.cs:36023-36027`): `wings == 27 && wingsLogic == wings`
→ `num4 = 0.4f * stealth`, `AddLight((int)Center.X/16, (int)Center.Y/16, num4,
num4*0.9f, num4*0.2f)`. On THIS base the port's `stealth` has NO producer (only
ever set in a knockback witness, `player.rs`), so the light would be a constant
`0.4f` — wrong. **The producer LANDED on the in-flight
`task/W4-f3-dust-player-3` branch** (probed: `update_stealth`,
`Player.cs:25733-25899` — the Psycho-Knife/Shroomite/Vortex if-else-if chain,
`stealth` default 1f, `stealth_timer`, `set_vortex`/`vortex_stealth_active`,
the Shroomite set grant `ArmorSetBonuses.cs:370-373`/`:392-394`, the Vortex
toggle `:56775-56778`). It is NOT merged here (verified
`git merge-base --is-ancestor` exit 1 for both HEAD and origin/main).

**Wiring recipe once f3 merges:** add `shroomite_wing_light(p)` to
`player_lights_b.rs` gated `p.wings_logic == 27 && p.has_wings` (the port's
`wings`/`wingsLogic` are ONE field, `player.rs:5155-5163`, so `wingsLogic ==
wings` folds true), `num4 = 0.4 * p.stealth`, rgb `[num4, num4*0.9, num4*0.2]`
at the quotient-truncated centre — the `kazzymodus_wing_light` shape. No RNG
(the `AddLight` line is draw-free; the surrounding `PlayerFrame` dust arms
are the skipped cosmetic layer). NOT stacked on the unmerged branch here.

## 3. LightingEngine legacy vs new mode — census

`Lighting.cs` (586 lines) is the MODE SELECTOR, not an engine: the `Mode`
setter (`:29-61`) routes Color → `NewEngine` (`LightingEngine`, the RGB blur)
and White/Retro/Trippy → `LegacyEngine` (`LegacyLighting`, Mode 1/2/3). Both
engines are ported:

- **New engine** (`LightingEngine.cs`, 281 lines) → `lighting/mod.rs` +
  `blur.rs`: the 4-phase rotation (`:76-113`) collapses to on-demand compute
  (deviation 9 in 10-lighting.md), `ApplyPerFrameLights`' one-frame swap
  (`:208-235`) → the `EntityLights` pair, `ExportToMiniMap` (`:246-270`) →
  `WorldMap::update`. The `UpdateLightDecay` table (`:141-206`) is the cited
  gap (section 4).
- **Legacy engine** (`LegacyLighting.cs`, 1442 lines) → `lighting/legacy.rs`
  (S1-E-19 wave A/B): the renderCount cycle, `doColors_Mode1/2/3` swipes, the
  sky-colour reseed, the minimap export. **The WoF `gross` light block
  (`:678-717`) is OWNED by d65** (S1-E-19d, `task/W4-d65-legacy-lighting-wof`,
  `c8a7e0e80` "Port the WoF gross light block … WofGrossLight seam + driver
  gate"): it adds `LegacyInputs::wof_gross: Option<WofGrossLight>` and the
  driver resolves `Main.wofNPCIndex >= 0 && player.gross`. On THIS base the
  block is still the cited `legacy.rs:843-844` note — d65's merge closes it.
  NOT duplicated here. The `gross` flag itself IS ported
  (`buff_effects_accessory.rs:197`, Horrified 37 while the WoF lives) and the
  WoF statics (`world.wof_npc_index`/`wof_dir`/`wof_x`) are written per tick
  (`world_sim/npcs.rs:4519-4530`), so d65's driver gate has its inputs.
- **Modern mask residuals** — d66 (`task/W4-d66-light-scanner-residuals`)
  regains `!tile.inActive()` in `blur.rs::mask_of` (`TileLightScanner.cs:68`)
  + the scanner border-null gate + 147/147 arm value-exactness. NOT duplicated.

**Retro-path verdict:** no unclaimed retro gap remains — the legacy engine is
ported (S1-E-19), the WoF gross block is d65's, the mask residuals are d66's.

## 4. Light-map decay/refresh — the modern `UpdateLightDecay` gap (cited)

`LightingEngine.UpdateLightDecay` (`LightingEngine.cs:141-206`) sets the
working map's per-medium decay EVERY blur:

| Medium | C# value | Port (`blur.rs::Mask::decay`) |
| --- | --- | --- |
| Air | `0.91f` (`:144`) | `[0.91; 3]` ✓ |
| Solid | `0.56f` (`:145`) | `[0.56; 3]` ✓ |
| CrackedBrick | `0.8f` (`LightMap.cs:51`) | `[0.8; 3]` ✓ |
| Honey | `(0.75, 0.7, 0.6) * 0.91` (`:146`) | ✓ |
| Water | `(0.88, 0.96, 1.015) * 0.91` default (`:153`) | **style-0 hardcoded** ✗ |

The water decay varies per `Main.waterStyle` — 13 arms (`:147-182`: styles
0/1/7/8, 2, 3, 4, 5, 6, 9, 10, 12, 13). The port hardcodes style 0
(`blur.rs:36`); any non-default water style diverges (10-lighting.md
deviation 2). Then the buff/debuff multipliers (`:184-203`): nightVision
×1.03, blind ×0.95, blackout ×0.85, headcovered ×0.85 (air AND solid), and
the `Main.SceneState.airLightDecay`/`solidLightDecay` factors (`:204-205`).
NONE are in `LightParams`/`LightKey` (10-lighting.md deviation 3).

**The SceneState half is partially ported but UNWIRED:** `scene.rs:1520-1521`
ramps `air_light_decay`/`solid_light_decay` toward 1.0 by 0.005 (the
`MoveTowards` `:279-280`), but the targets fold the shimmer/FTW terms to 1
(`SceneState.cs:259-277`: `1 - shimmerAlpha*{0, 0.3}`, the getGoodWorld
Golem-245-near ×0.6 / temple ×0.88 / dungeon ×0.94). The two fields have NO
consumer in the blur — they are computed and dropped. The legacy engine's
equivalent buff multipliers ARE ported (`legacy.rs:1004-1048`, the
`LegacyInputs` night_vision/blind/blackout/headcovered), so the gap is the
NEW engine's only.

**Refresh cadence:** the C# recomputes decay every blur phase (the 4-phase
rotation); the port recomputes on demand per drawn rect with the D2/D3 cache
(`lighting/mod.rs:347-372`). The water jitter is exact (`blur.rs:92-94`,
`WithModifier(i).Next(98,100)/100`, S1-A-6 — 10-lighting.md deviation 4 is
STALE, the multiplicative-hash approximation was replaced); the
`NON_VISIBLE_PADDING` is fixed 18 (`blur.rs:16`, S1-G-10 — deviation 5 STALE).

**Why cited, not ported here:** the fix adds `water_style` + the four buff
bools + the two SceneState factors to `LightParams` AND `LightKey`
(`params.rs:223-287`), threads them into `Mask::decay`, and changes the
cached light output for any non-default water style / active buff — a
determinism-affecting change to the frozen G6 `WorldFrameKey` path that needs
a byte-equality A/B (AGENTS.md "Determinism is a hard invariant"). It also
lands in `blur.rs`/`mod.rs`, which **d66 is actively editing** (the mask
residuals). Porting it here would conflict with d66 and skip the required
A/B. Task seed (10-lighting.md "[M] Water-style decay + buff/debuff decay in
LightParams"): add the fields, thread `Mask::decay`, unit-test styles 2/6/9,
wire the SceneState factors, A/B the cached output. The FTW Golem/temple/
dungeon terms need `SceneMetrics.closest_npc_position[245]` (ported,
`scene_scan.rs:264-303`) + `zone_lihzhard_temple`/`zone_dungeon` (ported) +
the `getGoodWorld` seed flag (`world.seeds FOR_THE_WORTHY`, ported).

## 5. Torch flicker global — census + the oscillators LANDED (`999b292af`)

The PER-TILE torch flicker is landed (d56/e8): `tile_light/scanner.rs::
torch_lights` draws `Next(28,42)*0.005 + (270 - mouseTextColor)/700` per torch
(`TileLightScanner.cs:1023`/`:1065`/`:1163`), the `/1000` chandelier variant
(`:2908`), the campfire `Next(95,106)` (`:170`). The GLOBAL flicker is the
three `Main.DoUpdate_Animate*` oscillators that feed those per-tile terms:

| Oscillator | C# | Step | Port |
| --- | --- | --- | --- |
| `mouseTextColor` | `DoUpdate_AnimateCursorColors` `Main.cs:18216-18224` | ±1, 190..255 | landed (`draw.rs:330-338`) |
| `demonTorch` | `DoUpdate_AnimateTileGlows` `:18240-18250` | ±0.01, 0..1 | landed (`draw.rs:351-359`) |
| `masterColor` | `DoUpdate_AnimateCursorColors` `:18225-18235` | ±0.05, 0..1 | **LANDED here** |
| `martianLight` | `DoUpdate_AnimateTileGlows` `:18251-18261` | ±0.015, 0..1 | **LANDED here** |

All four init 1f/dir 1 except mouseTextColor (225/dir 1) (`Main.cs:66829-66834`).
The two new oscillators ride a shared `bounce_oscillator(value, dir, step)`
helper (`draw.rs:44`, the `demon_torch` precedent), RNG-free, so the flicker
phase is a pure function of the frame count. 3 witnesses WRITTEN NOT RUN.

**The readers stay cited** (the oscillators are the state half; the consumers
are unported):
- `martianLight` → `TileDrawing.cs:498`'s `_martianGlow = (100 + 150*
  martianLight)` byte colour. The port's `drawdata_cases_a::martian_glow`
  (`:107`) takes the value but is fed 0 (`render/tiles.rs:732`) until the glow
  pass (TILE-DRAW-2) + the Martian Madness event land.
- `masterColor` → the rarity -13 master-mode text colour (`Main.cs:20249`
  `(255, masterColor*200*num3, 0)`, `:20562` `(255, masterColor*200, 0)`,
  `PopupText.cs:669`, `TileDrawing.cs:9555`). The port's `ui/tooltip.rs::
  item_name_color` (`:124-125`) and `ui/resources.rs::mouse_text_color`
  (`:499-500`) document -13/-12 as dropped (no master/disco global). Wiring
  recipe: thread `app.master_color` into the tooltip rarity -13 arm and the
  popup-text master colour once those consumers land.

**No global torch-flicker RNG:** the per-tile `Next(28,42)` draws ride the
scanner's `FastRandom` (the `TileLightScanner._random` stream, seeded
per-frame `NextSeed()`), NOT `Main.rand` — the oscillators themselves draw
nothing, so landing them leaves every gameplay stream untouched.

## Verification

Code-only mode: **no cargo/rustc/rustfmt/tests/dotnet/graphify invoked.** The
10 new witnesses (7 Snapping Stone + 3 oscillator) are WRITTEN, NOT RUN. The
first build after the merge must run `cargo test -p tr-app player_lights_b`
plus `cargo test -p tr-app w4_e7` plus the catalog command. The two lantern
commits: `5ae32127` (Snapping Stone trio), `999b292af` (global-flicker
oscillators).