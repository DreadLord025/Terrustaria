# 179 — Echo-coating tile/wall draw census (W4-g78)

> **NUMBER CHAIN: 64 → 179 (FINAL, EXECUTED per registry-150 §10.4 round-1
> row).** Number: **64→179 ROUND-1 EXECUTED** — the d54 registry's §10.4
> round-1 loser table assigns W4-g78 tile-echo-draw-census 02:57:05 → **179**;
> 64 belongs to W4-d64's earlier `64-drawdata-cases-4.md` (11-20T18:54:02
> `161c63898`) — d64 KEEPS 64. This was g78's round-1 owed rename (kept
> through the 78→375 §10.2 rename at `ce6783ffe`). All refs
> (INDEX/ANALYSIS/PORT_MAP/research/76) follow this rename; the old key 64
> survives only in this chain-banner (history).

> W4 lane g78 · 2026-11-21 · sources: decompiled (TileDrawing.cs, WallDrawing.cs,
> Main.cs, Player.cs, SceneMetrics.cs, LightingEngine.cs/LegacyLighting.cs) +
> crates (tr-core scene.rs, tr-app player.rs / render/{tiles,vines,pylons,bg}.rs /
> app/frame/draw.rs) · base `a725acb0` · code-only mode (no cargo run)

## Scope

The Echo-coating draw lane: an Echo-coated (`invisibleBlock()`/`invisibleWall()`,
the port's sparse `World.extras` bits) tile/wall draws NOTHING unless
`Main.ShouldShowInvisibleBlocksAndWalls()` grants sight — the Echo Monolith
(tile 657, item 5345) or the Spectre Goggles (item 4409) / a vanity-worn Echo
Monolith. Co-lanes: **g79** owns paint (`tileColor`/`wallColor` recolour), **d56**
landed the animated-tile draw gate incl. the `IsVisible` veto inside
`tr_render::animated_tiles::tile_draw_gate` (5d80b38a, UNWIRED — read, not
re-ported here); **g77** owns the crack-draw echo conjuncts
(`tile_cracks.rs`); **g70** owns the bg-layers rewrite (`render/bg.rs`).

## The sight chain (C# → port)

| C# site | Behaviour | Port | Verdict |
| --- | --- | --- | --- |
| `Main.ShouldShowInvisibleBlocksAndWalls` `Main.cs:58023-58030` | `EchoMonolith ? true : PerspectivePlayer.CanSeeInvisibleBlocks` | `tr_core::scene::should_show_invisible_blocks_and_walls(echo_monolith, can_see_invisible_blocks)` | **PORTED+WIRED (g78)** — was MISSING (every consumer hardwired false / env-pin only) |
| `SceneMetrics.EchoMonolith` scan `SceneMetrics.cs:541-546` | tile 657 active, `frameY >= 54` (lit row) → true; reset `:818` | `scene.rs::scan_on_screen_tiles` 657 arm (landed pre-wave, `:681`) | PORTED (pre-wave) — g78 now CONSUMES it |
| `Player.CanSeeInvisibleBlocks` field `Player.cs:2814` | bool | `player.rs::can_see_invisible_blocks` | **PORTED (g78)** |
| `ApplyEquipFunctional` case 4409 `Player.cs:14548-14550` | Spectre Goggles → true | `apply_equip_functional` match arm 4409 | **PORTED+WIRED (g78)** |
| `ApplyEquipVanity` case 5345 `Player.cs:14215-14217` (loop `UpdateEquips` `:13009-13015`, slots 13..20) | vanity-worn Echo Monolith → true | `update_equips`' 13..20 scan, kind-only match (no accessory gate — C# calls the vanity arm per usable slot) | **PORTED+WIRED (g78)** — the loop's other rows (wingSlot/hideWolf/5452/ApplyShader) stay unported families |
| `ResetEffects` clear `Player.cs:18742` | false per tick | the ResetEffects block (`player.rs`, beside `night_vision`) | **PORTED (g78)** |
| `TileDrawing.Update` `:379-384` | `_shouldShowInvisibleBlocks` per update; a CHANGE marks all framed sections dirty | `frame/draw.rs` computes ONE `show_invisible` per frame (env pin ∥ the gate); the G6 `WorldFrameKey.light_params` already carries `draw_invisible_walls`, so a sight flip invalidates the cached world range (the port's section-refresh analogue) | **PORTED (g78)** — no key change needed (the input was already keyed, only its value was pinned false) |
| `WallDrawing.Update` `:45` | `_shouldShowInvisibleWalls` = the same gate | the same per-frame value (walls read it via `wall_visible`) | **PORTED (g78)** |
| `LightingEngine.cs:124` / `LegacyLighting.cs:726` `DrawInvisibleWalls` | the light options ride the gate | `LightParams.draw_invisible_walls` (M1.5-LIGHT-3 landed the field + all scanner arms) — g78 feeds the real value | **PORTED (pre-wave) + WIRED (g78)** |

## The draw-site census (`IsVisible` + the echo gates)

| C# site | Behaviour | Port | Verdict |
| --- | --- | --- | --- |
| `TileDrawing.IsVisible` `:1411-1432` | coating bit; kind 19 `frameY/18==48` → forced flag; 541/631 → forced flag; flag → sight | `scene::tile_is_visible` (shared keep-one seam) | **PORTED (g78)** — the pre-wave `tiles.rs` gate read the BARE bit (missed all three overrides); pylons.rs had a private copy (now delegates) |
| `DrawSingleTile` gate `:983-991` + `!flag` return `:1014-1017` | invisible → no sprite, no flame/glow; `(!flag & flag2)` → DrawBlack only when VISIBLE | `tiles.rs` per-cell gate (post-`skip_draw`/unframed, pre-back-rope — the S1-G-1f position) | **FIXED (g78)** — sight-fed + the overrides; the dark-merge now excludes hidden cells (below) |
| `DrawBlackTiles` `Main.cs:57981`, `:58006-58007` | black coverage needs `(!invisibleBlock || flag)` / `(!invisibleWall || flag)` | `tiles.rs` `block_covers`/`wall_covers` in the dark-cell merge | **FIXED (g78)** — a hidden echo cell in a dark cave now shows the parallax, exactly like C# |
| `WallDrawing.DrawWalls` head `:91` | `wall<=0 ∥ FullTile ∥ (318 && !sight) ∥ (invisibleWall && !sight)` → skip | `tiles.rs::wall_visible(wall, invisible_wall, show_invisible)` + the FullTile arm first (C# order) | **FIXED (g78)** — was `wall != 0 && wall != 318` + a separate bare-bit continue |
| `WallDrawing.FullTile` `:258-261` | hidden coated block does NOT hide its wall | the `(!invisible_block ∥ sight)` conjunct before `tile_hides_wall` | **FIXED (g78)** |
| `WallDrawing` white arms `:96-103` + `Main.wallLight[318]=true` `Main.cs:10580` | fullbrightWall OR wall 318 → `Color.White` (before the black test) | `push_wall_lit` white flag = `fullbright_wall ∥ kind==WALL_ECHO`; the zero-light skip exempts 318; the dark merge exempts a shown 318 (`echo_wall_lit`) | **FIXED (g78)** — a shown echo wall draws white even in zero light |
| `GetTileDrawData` case 541 `:6004-6006` | `addFrY = sight ? 0 : 90` | `drawdata_cases_b.rs` case 541 (S1-F-9e landed) | PORTED (pre-wave) — g78 feeds the real sight value (was env-pin) |
| Platform-slope side fills `:1373/:1387`, `:1524/:1534` | `(sight ∥ !neighbour.invisibleBlock())` + the `(!sight && tile.invisibleBlock())` tail | `special_tiles.rs::platform_side_fill` (S1-G-1e, UNWIRED — d59 owns the wiring) | PORTED-UNWIRED (pre-wave) — g78's sight value reaches it through `draw_tiles`' `show_invisible_blocks` when d59 wires |
| `DrawGrass` `:8338` / `DrawAnyDirectionalGrass` `:8395` | `!IsVisible → continue` | the inline sway arms draw after the per-cell gate | COVERED by the g78 gate (the fused pass's documented inline approximation) |
| `DrawVineStrip` `:9121` / `DrawRisingVineStrip` `:9185` | `IsVisible` per segment; the walk continues | `vines.rs` segment `visible` flag — now `tile_is_visible(..., sight)` | **FIXED (g78)** — was `!invisible_block` (sight-blind); reverse vines (549) still unported (S1-G-1 remainder) |
| `DrawMultiTileGrassInWind` `:8957` / `DrawMultiTileVinesInWind` `:9464` | per-cell `IsVisible` | the sunflower per-cell approximation draws after the per-cell gate; the general multi-tile passes unported | COVERED (inline) / MISSING (the passes themselves — S1-G-1 remainder, not g78) |
| `DrawMasterTrophies` `:8610` / `DrawTeleportationPylons` `:8652` | anchor-cell `IsVisible` | `pylons.rs::is_visible` → delegates to `scene::tile_is_visible` (keep-one) | PORTED (S1-G-1d) — g78 feeds the real sight (was env-pin) + dedupes the copy |
| `DrawTrees` `:8052-8069` | NO IsVisible gate (active only) | `foliage.rs` — no gate | MATCHES (nothing to do) |
| `DrawTiles_EmitParticles` `:6690` / trophy dust `:7442` | `IsVisible` gates on dust rolls | d60's unwired particle module / skipped dust | d60/g77 co-lane — not g78 |
| `Main.DrawTileCracks` `:61540` conjuncts | `(flag ∥ !invisible…)` | `tile_cracks.rs` (vacuous-true today) | **g77 co-lane** (c3943a18 re-arms the real gates) — not g78 |
| `Main.WallLightAt(i,j,show)` `:55169-55181` | wallLight table wins; else HIDDEN coating → true (backdrop shows), shown → false | `scene::wall_light_at(wall_is_light, invisible_wall, show)` pure predicate | **PORTED-UNWIRED (g78)** — the `bg.rs::wall_light_at` consumer reads only the table; bg.rs is contested by g70, wiring lands with that merge |
| `TileLightScanner` invisible arms `:3142/:3149/:3246` + `LightIsBlocked` `:157` | hidden echo cells pass light | `tile_light/mod.rs` + `lighting/*` (M1.5-LIGHT-3, d66) | PORTED (pre-wave) — g78's real `draw_invisible_walls` feed activates them |

## Residuals left to co-lanes (recorded, not touched)

1. **g79 (paint)**: `tileColor`/`wallColor` recolour of the SHOWN echo tiles —
   the echo lane draws them through the normal paint path (`push_sprite_paint`),
   so g79's shader work covers them with no echo-specific arm.
2. **g77 (cracks)**: the `tile_cracks.rs` echo conjuncts (its phase 1 re-arms
   them onto the real bits; the sight VALUE it reads becomes live via this
   wave's player field — its `ShouldShowInvisibleBlocksAndWalls` stand-in
   should delegate to `scene::should_show_invisible_blocks_and_walls` at merge).
3. **g70 (bg layers)**: the `bg.rs::wall_light_at` wiring of
   `scene::wall_light_at` (the pure predicate + witness landed here).
4. **d56/d57/d58/d59/d60**: the unwired animated-tile gate / side fills /
   particles consume the same `show_invisible_blocks` input `draw_tiles`
   already carries — no new seam needed at their wiring merges.
5. **Framing** (`framing.rs:1191` note): `GetMergeMask`'s invisible-wall veto
   reads the raw bit with sight assumed false — the framing lane (d109/d110)
   owns that consumer; the sight predicate is available for its merge.

## Witnesses (WRITTEN NOT RUN — code-only)

- `scene.rs`: `w4_g78_the_sight_gate_is_monolith_or_goggles`,
  `w4_g78_the_echo_monolith_scan_reads_the_lit_frame_row`,
  `w4_g78_is_visible_census`, `w4_g78_wall_light_at_census`.
- `player.rs`: `w4_g78_the_spectre_goggles_grant_echo_sight_per_tick`,
  `w4_g78_the_vanity_monolith_grants_but_the_functional_slot_does_not`,
  `w4_g78_either_sight_source_keeps_the_flag`.
- `vines.rs`: `w4_g78_echo_sight_shows_the_coated_segment_without_moving_the_strip`.
- `tiles.rs`: `w4_g78_the_echo_gates_ride_the_sight_input`,
  `w4_g78_the_shown_echo_wall_feeds_white` (source-scan, the S1-B-7 pattern).
- `bg.rs`: `echo_walls_are_invisible_like_walldrawing` extended (coating bit +
  sight lifts + wall-0 refusal).

## RNG / determinism

Zero RNG draws in every arm (the sight chain is pure state). The G6
`WorldFrameKey` is UNCHANGED — `light_params` already keyed
`draw_invisible_walls`; only its value becomes live. Pixel deltas vs the
pre-wave port (all moves TOWARD the C#, pinned by the witnesses):
- uncoated platform row 48 + kinds 541/631 now HIDE without sight (the
  pre-wave bare-bit gate drew them — the `IsVisible` overrides were missing);
- hidden echo cells in dark caves no longer paint black coverage (the
  parallax shows through, `Main.cs:58006-58007`);
- a hidden echo-coated block no longer culls its wall (`FullTile` `:258`);
- with sight granted, coated tiles/walls + wall 318 draw (318 white), the
  light pass treats them as solid, and case 541 flips to frame row 0.
Worlds with none of those cells and no sight source render bit-identical.
