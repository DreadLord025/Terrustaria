# 14 — Map / MapHelper / minimap layers

> Wave 2 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

C# side. `decompiled/Terraria/Map/MapHelper.cs` (3 950 lines) is the module's god file: the static
colour tables built in `Initialize` (`:209-1789`), the per-tile colour selection
(`CreateMapTile` `:1900-1923`, `GetTileType` `:1925-1985`, `GetWallType` `:1987-2025`,
`GetBackgroundType` `:2027-2076`, `GetTileBaseOption` `:2083-3005`), the paint multiply
(`MapColor` `:1823-1874`), the XNA colour read-out (`GetMapTileXnaColor` `:1876-1893`) and the
`.map` IO (`SaveMap` `:3007`, `InternalSaveMap` `:3030-3322`, `DecompressChunk` `:3325`,
`LoadMapVersionCompressed` `:3347-3585`, `LoadMapVersion1` `:3587-3747`,
`LoadMapVersion2` `:3749-4027`). `Map/MapTile.cs` (90) is the 16-bit packed per-tile record;
`Map/WorldMap.cs` (268) owns the world-sized `MapTile[,]`, the reveal/queue/edge logic
(`UpdateLighting` `:49`, `UpdateType` `:65`/`:70`, `QueueUpdate` `:94`/`:99`,
`UnlockMapSection` `:109`, `UnlockMapTilePretty` `:140`, `Load` `:176`, `ClearEdges` `:260`);
`Map/MapUpdateQueue.cs` (98) is the area/tile invalidate queue fed by worldgen and net code. The
map *draw* side lives outside `Map/`: `MapRenderer.cs` (render-target grid, `QueueChange` `:288`,
`CheckMapTargets` `:357`, `DrawToMap_Section` `:306`, `DrawMap` `:380`),
`Graphics/Light/LightingEngine.cs::ExportToMiniMap` (`:246-270`) which is what actually reveals
tiles, `Main.DrawMap` (`:55279-56444`) plus `Main.UpdateMinimapAnchors` (`:55267-55277`) and
`Main.DrawMapFullscreenBackground` (`:56342-56444`), and the overlay layers
(`PingMapLayer.cs` 46, `SpawnMapLayer.cs` 22, `TeamBasedSpawnMapLayer.cs` 21,
`TeleportPylonsMapLayer.cs` 115, `BossBagMapLayer.cs` 22, `DigtoiseMapLayer.cs` 26,
`MapIconOverlay.cs` 20, `IMapLayer.cs` 5, `MapOverlayDrawContext.cs` 227) and the HUD chrome
(`GameContent/UI/Minimap/MinimapFrame.cs` 184, `MinimapFrameManager.cs` 46,
`MinimapFrameTemplate.cs` 34).

Rust side. `crates/tr-core/src/map/mod.rs` (506) — `MapTile`, `CreateMapTile`, reveal; `colors.rs`
(254) — the packed `colorLookup` and gradients; `option.rs` (802) — the `MapHelper` option port;
`file.rs` (814) — the `.map` v326 byte layout. `crates/tr-app/src/map.rs` (1 161) — full/minimap
geometry, `.map` file IO with `flate2`, `MapBG` selection, head icon, `WorldMap`; and
`crates/tr-app/src/ui/minimap.rs` (807) — the frame chrome, zoom buttons and anchor maths.
Integration: `app/frame.rs:207-222` (the minimap gate / `Main.mH`), `:334` + `app/clicks.rs:279-333`
(button hover/click), `app/frame.rs:1208-1261` (minimap draw branch) and `:1513-1573`
(fullscreen map + head icon), `app/saveglue.rs:202-214` (sidecar save), `persist.rs:39`
(`map_path`), `render/tiles.rs:324` (the light compute whose output the reveal reads).

Responsibility in one paragraph: the map is a persistent, world-sized per-tile record of what the
player has *seen*, stored as a packed colour-table index plus a light byte and a paint id, written
to a sidecar `<world>.map` file, and rendered twice — as a small panel in the HUD and as a
full-screen zoom/pan view — with the colour of each tile derived from the original
`MapHelper.Initialize` lookup tables.

## Key entities

- `crates/tr-core/src/map/mod.rs::MapTile` (35-128) — `ty` (packed `baseType + baseOption`),
  `light`, `extra` (bit 7 `IsChanged`, bit 6 `UpdateQueued`, bits 0-4 `Color`); mirrors
  `MapTile.cs:9-59`.
- `.../map/mod.rs::create_map_tile` (224) — `MapHelper.CreateMapTile` (`MapHelper.cs:1900-1923`):
  tile → wall → background fallback chain.
- `.../map/mod.rs::map_color` (157-202) — `MapHelper.MapColor` (`MapHelper.cs:1823-1874`): shadow /
  negative / multiply paint paths.
- `.../map/mod.rs::get_map_tile_color` (207) — `MapHelper.GetMapTileXnaColor`
  (`MapHelper.cs:1876-1893`).
- `.../map/mod.rs::update_map_lighting` (287) — `WorldMap.UpdateLighting`
  (`WorldMap.cs:49-63`): brighter-light wins, returns "changed".
- `.../map/mod.rs::reveal_rect` (314) — `LightingEngine.ExportToMiniMap`
  (`LightingEngine.cs:246-270`): per-tile `min(255, max(r,g,b)*255)` and the changed list.
- `.../map/colors.rs::MapColorTable` (52-193) + `color_table` (194) — `MapHelper.Initialize`'s
  tables: `lookup`, `tile_lookup`, `wall_lookup`, option counts, `sky/dirt/rock/hell_position`,
  `wall_range_start/end`, `liquid_position`, `snow_types`.
- `.../map/colors.rs::sky_gradient/dirt_gradient/rock_gradient` (37/42/47) — `array5/6/7`
  (`MapHelper.cs:1662-1688`) and `MapHelper.CalcSkyGradient` (`:2078`).
- `.../map/option.rs::get_tile_type` (30) — `MapHelper.GetTileType` (`:1925-1985`).
- `.../map/option.rs::get_tile_base_option` (112-645) — `MapHelper.GetTileBaseOption`
  (`:2083-3005`): the ~900-line switch transcribed case-for-case; substituted cases at
  133/137/141/145.
- `.../map/option.rs::get_wall_type` (652) — `MapHelper.GetWallType` (`:1987-2025`), including the
  `21|88|…|241` zero-paint list and wall 27's stripe option.
- `.../map/option.rs::get_background_type` (681-710) — `MapHelper.GetBackgroundType`
  (`:2027-2076`) with the snow scan.
- `.../map/file.rs::write_map_header` (58) / `write_option_tables` (105) / `encode_tiles` (128) —
  `MapHelper.InternalSaveMap` (`:3030-3322`).
- `.../map/file.rs::read_map_prefix` (319) / `decode_tiles` (410) — `MapHelper.LoadMapVersion2`
  (`:3749-4027`).
- `crates/tr-app/src/map.rs::WorldMap` (386-516) — the state holder (`tiles`, `open`, `scale`,
  `pos`, `minimap_scale`, `dirty`, `needs_full_upload`, `reset_pos`); `update` (425) is the reveal
  driver, `prepare` (480) the clamp/recentre/upload pass.
- `crates/tr-app/src/map.rs::map_full_quad` (88) / `map_frame_rect` (107) / `map_minimap_quad`
  (166) / `head_icon_pos` (76) / `map_bg_index` (231) — the `Main.DrawMap` geometry.
- `crates/tr-app/src/ui/minimap.rs::FRAME_STYLES` (36) / `style_index` (121) /
  `minimap_anchor` (130) / `minimap_origin_raw` (147) / `scale_rect` (171) / `button_hitbox` (202) /
  `update` (280) / `apply` (301) — `MinimapFrameManager` + `MinimapFrame`.
- `crates/tr-app/src/map.rs::save_map_file` (597) / `load_map_file` (622) — the deflate wrapping
  and file IO (`tr-app` is the only crate depending on `flate2`).

## Dependencies

- Calls into: `tr-core::tile::{Tile, LiquidKind, FLAG_FULLBRIGHT}` (`mod.rs:25-28`),
  `tr-core::tile_light::paint_color` (reused by `map_color`), `tr-core::lighting::LightMap`
  (`reveal_rect`), `tr-core::mapdata::{TILE_COUNT, WALL_COUNT}` (table sizing), `tr-render`
  (`SpriteRenderer::ensure_map_texture` / `write_map_region` in `prepare`), `crate::ui::minimap`
  (the `MapScale` batch transform), `crate::assets` (the `MapBG*` and `UI/Minimap/*` sheets).
- Called by: `app/frame.rs:1248` (`WorldMap::draw_minimap`), `:1549` (`draw_full`), `:1518`
  (`prepare`), `app/clicks.rs:333` (`ui::minimap::apply`), `app/saveglue.rs:202-214`
  (`save_map_file`), world load (`WorldMap::load` ← `map.rs:454`), `render/entities.rs`
  (`draw_player_map_icon`, positioned by `map::head_icon_pos`).
- Shared data structures: `MapTile` is the whole persisted unit — `.map` IO (`file.rs`) ↔
  `WorldMap::tiles` ↔ the map texture upload. `World` supplies `world_surface`, `rock_layer`,
  `height`, `seed`, `name` and the sparse `extras` (wall paint) that `create_map_tile` reads.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `MapHelper.Initialize` colour tables | PORTED | `colors.rs::MapColorTable::new` (76) | `tile_option_counts`/`wall_option_counts`/`lookup` packing + `tile_position`/`wall_range_*`/`liquid/sky/dirt/rock/hell_position`/`snow_types` all derived from `TILE_COLORS`/`WALL_COLORS` the same way (`MapHelper.cs:1691-1789`). |
| sky/dirt/rock gradients | PORTED | `colors.rs:37-49` | Same endpoints and denominators (256 sky / 255 dirt+rock), pinned by tests. |
| `MapColor` (paint) | PORTED | `mod.rs::map_color` (157) | Shadow (29), negative (30, wall half-invert), default multiply; `MapHelper.cs:1823-1874`. |
| `GetMapTileXnaColor` | PORTED | `mod.rs::get_map_tile_color` (207) | `i`/`j` still unused (kept for parity, ruling 10). |
| `CreateMapTile` tile→wall→bg chain | PORTED | `mod.rs::create_map_tile` (224) | Including the `baseType == 0 → wall → background` fallbacks (`MapHelper.cs:1900-1923`). |
| `GetTileType` | PARTIAL | `option.rs::get_tile_type` (30) | Caseless for the tile specials (5, 51/697, 19, 184, 160) but `invisibleBlock` (`MapHelper.cs:1933`) is unreachable (ruling 1) and `DebugOptions.ShowUnbreakableWall && wall == 350` (`:1927`) is hardcoded `false`. `WorldGen.IsThisAMushroomTree` (tile 5) not ported (ruling 5). |
| `GetTileBaseOption` (~900-line switch) | PORTED | `option.rs::get_tile_base_option` (112-645) | All 64 `switch` cases transcribed in source order; **four** substituted with `0` and flagged: `461` (SceneMetrics zones, `:133`), `80` (`WorldGen.GetCactusType`, `:137`), `529`/`530` (`WorldGen.GetBiomeInfluence`, `:141`/`:145`). |
| `GetWallType` | PARTIAL | `option.rs::get_wall_type` (652) | `invisibleWall` (`:1989`) and `fullbrightWall` (`:1990`) both unreachable (rulings 1 and 3). |
| `GetBackgroundType` | PARTIAL | `option.rs::get_background_type` (681) | `Main.remixWorld` fixed `false` (ruling 4); `sceneArea`/`sceneSnowiness` not ported so the 7×7-by-10-px snow scan always runs (ruling 8). |
| `MapTile` bitfield | PORTED | `mod.rs::MapTile` (35-128) | `ty` packs C# `Type = baseType + baseOption`; `extra` mirrors `_extraData` (`MapTile.cs:9-59`); `MapTile.Create` (`MapTile.cs:98-101`) sets the changed bit. |
| `WorldMap.UpdateLighting` / reveal | PARTIAL | `mod.rs::update_map_lighting` (287), `reveal_rect` (314) | Semantics ported exactly, but the revealed rect differs from `ExportToMiniMap` — see deviation D10. |
| `LightingEngine.ExportToMiniMap` cadence | PORTED | `map.rs::WorldMap::update` (425) | 4×/s ≤> the engine's 4-state cycle `MinimapUpdate → ExportMetrics → Scan → Blur` (`LightingEngine.cs:81-104`); `Main.mapDelay` skip unmodelled (minor). |
| `MapHelper.SaveMap` / `InternalSaveMap` | PORTED | `file.rs::write_map_header` (58), `write_option_tables` (105), `encode_tiles` (128) | v326 header, `FileMetadata` revision 1, option bitmaps + non-default counts, row RLE; cloud-save path and `Main.statusText` not ported. |
| `MapHelper.LoadMapVersion2` | PORTED | `file.rs::read_map_prefix` (319), `decode_tiles` (410) | Header/seed/world-size validation, option-table remap, RLE body. |
| `LoadMapVersion1` / `LoadMapVersionCompressed` / `DecompressChunk` | MISSING | — | Pre-2 formats never open (`MapHelper.cs:3347-3585`, `:3587-3747`); no error text for them. |
| `WorldMap.Load` / `Save` / `TryGetMapPath` | PORTED | `map.rs::load_map_file` (622), `save_map_file` (597), `persist.rs:39` | `<world>.map` sidecar next to the `.trwld`; loaded at world start, written by `saveglue.rs:202-214`. The catch arm (`WorldMap.cs:217-230`) landed the e18 final sweep (re-folded): a corrupt-but-present file copies to `<path>.bad` + `Clear()`, the absent file returns with neither side effect (`:184-188`). |
| `MapUpdateQueue` (area + tile queues) | PORTED | `map.rs::MapUpdateQueue` | `MapUpdateQueue.cs:21-108` (W4-c35): the 262144 budget, `ClampToWorld` clamps, `RangeFrame`'s inverted rect + border-gated walk, the deterministic drain; net-5/10/20 hooks ride `netglue.rs`, the drain rides `frame/world.rs::map_phase`. |
| `WorldMap.UnlockMapSection` / `UnlockMapTilePretty` | PORTED | `mod.rs::unlock_map_section`/`unlock_map_tile_pretty`, `map.rs::unlock_section`/`unlock_map` | `WorldMap.cs:109-174` (S1-L-3); the `DebugOptions.unlockMap` sweep driver landed W4-e18 (`frame/world.rs::map_reveal_sweep_core`, `Main.cs:61873-61883`); W4-f46 union: `unlock_map` is the `/map` command's verbatim flag path (`ToolkitDebugCommands.cs:480-494`), not a bulk sweep. |
| `WorldMap.ClearEdges` | PORTED | `mod.rs::clear_edges` | `WorldMap.cs:260-285` (S1-L-3): the 40-tile bands clear on `.map` load. |
| `MapRenderer.QueueChange` / refresh grid | PORTED (semantics) | `map.rs::queue_change`/`changes_queued`/`draw_to_map_queues` | `MapRenderer.cs:288-304` (W4-e18): the 2048-per-target threshold arms `refreshMap`, the `refreshMap` no-op gate, the `DrawToMap` queue-clear; the port uploads through the dirty list instead of the 5x2 render-target grid — equivalent output, different cost. |
| `WorldSections` / `Main.sectionManager` reveal sweep | PORTED | `map_sections.rs::MapSections`, `frame/world.rs::map_reveal_sweep_core`/`map_reveal_draw` (call site `frame/draw.rs`) | `WorldSections.cs:6-358` + `Main.cs:61863-61899` (W4-e18 first half): the truncating 200x150 grid, the player-centred spiral with cross-call resume, the 5 ms-budget sweep, the `DebugOptions.unlockMap` arm, the `refreshMap` → `ClearMapDraw` consume, the `updateMap` slot (`LightingEngine.cs:269` / `LegacyLighting.cs:432`), the `loadMap` staged-upload gate. W4-f46 union (research/84): the driver runs on the `DoDraw` half, the paused frame falls through to the `ChangesQueued` else-arm (`:61896-61898`), and the empty export intersect arms `Rectangle.Empty`. |
| `MapRenderer.DrawToMap_Section` | PORTED | `map.rs::draw_to_map_section`/`section_pixels`, painted from `frame/world.rs::map_reveal_draw` | `MapRenderer.cs:306-355` (W4-f46 salvage): the section's world-clipped row-major `GetMapTileXnaColor` cache, one region blit; `checkMap`'s on-demand target creation rides `ensure_map_texture` ahead of the walk (`:322`). |
| `MapRenderer.CheckMapTargets` | PORTED (semantics) | `map.rs::prepare` (480, `uploaded`/`needs_full_upload`) | One texture per world instead of the target grid (`MapRenderer.cs:357-378`). |
| `Main.DrawMap` fullscreen geometry | PORTED | `map.rs::map_scale_clamp` (20), `map_wheel_scale` (28), `map_key_scale` (33), `map_clamp_pos` (38), `map_drag_delta` (49), `map_full_quad` (88), `map_frame_rect` (107) | `Main.cs:55306-55431`, `55342-55359`, `55388-55412`. |
| `Main.DrawMapFullscreenBackground` (`MapBG`) | PARTIAL | `map.rs::map_bg_index` (231) | Ported-later branch order, but the unevaluable zones take the `false` path: `ZoneDungeon`, `ZoneGlowshroom`, `ZoneCorrupt`, `ZoneCrimson`, `ZoneHallow`, `ZoneJungle`, `bloodMoon` (no doubling), `ZoneGraveyard` (`Main.cs:56342-56444`); ANALYSIS.md:1578-1582 admits it. |
| Minimap draw + panel geometry | PORTED | `map.rs::map_minimap_quad` (166) | 240×240 panel, `240/scale` window, 10/10/`maxTilesX-10`/`maxTilesY-19` clamps, `MapScale` batch transform (`Main.cs:55293-55296`, `:55503-55545`, `:55309-55329`). |
| Minimap alpha (`mapMinimapAlpha`) | PORTED | `map.rs::WorldMap::minimap_alpha` + `map_key_alpha`/`map_alpha_clamp`, `app/mod.rs::update_minimap_zoom_keys`, `frame/draw.rs` (backdrop + tile + layer alpha) | `Main.cs:55523-55532` (the clamp + `b` byte), `Player.cs:24921-24928` (the ±0.015 keys); W4-f46 dup fold `a352b16e` (g53 superset + e18 `249a0c1b` alpha state) — D17 closed. |
| Minimap frame chrome + zoom buttons | PORTED | `ui/minimap.rs` (whole file) | Nine styles, hitboxes, hover sheets, `DEFAULT_ZOOM 1.05`, ×1.025/×0.975, clamp 0.2..3 (`MinimapFrame.cs:59-63`, `:107-136`, `:138-208`). |
| Frame-style selection UI | PORTED | `menu/pages.rs:268-284`, `menu/actions.rs:332-333`, `settings.rs:113-115` | `MinimapFrame` config key + `CycleSelection` order. |
| Map style switcher (`mapStyle` 0/1/2) | PARTIAL | icon row `clicks.rs:1572-1602` + `ui/chrome.rs::draw_map_style_icons`; cycle key `app/loop.rs::world_key` (`MapAction::Style`) | The inventory icon row (`Main.cs:41087-41120`, the style icons play `PlaySound(12)` — e18 `4ad0f041` re-fold) and the Tab cycle (`Player.cs:24949-24965`, W4-f46 closeout) are landed; `mapStyle == 2`'s screen-overlay draw (`Main.cs:55548-55578`, `mapOverlayScale`) and style 0's no-HUD state exist as states, but the style-2 draw half is absent (D18). |
| Player head icon | PORTED | `map.rs::head_icon_pos` (76) + `render/entities.rs::draw_player_map_icon` | Position ported (`Main.cs:55937-55954`); the 84×84 `MapHeadRenderer` render target is replaced by a hand-drawn head (ANALYSIS.md:1584-1593). |
| Other players' head icons | PARTIAL | same draw path | `Main.cs:55931-55954` loops all 255 players; single-player only here. |
| NPC / chest / pylon / ping / boss-bag icons | PORTED (f29/e17, keep-one PENDING at merge) | `map_layers.rs` (S1-L-4, on this union) + `map_overlay.rs`/`map_pylons.rs` (W4-f29) vs `map_npc_icons.rs`/`map_player_icons.rs` (W4-e17) | The six `Map/*Layer*.cs` layers landed S1-L-4; the nine `Main.DrawMap` overlay pieces (pylon lines/click/pan, boss/town heads, death markers, revenge coins, misc portals+golf, minimap heads, hover names) are ported TWICE on the sibling branches — the merger takes ONE per research/140 (e17 skeleton + f29 F1-F6 fixes) and the composition note below. The ping detector (`Main.cs:55486-55495`) is `clicks.rs::handle_fullmap_ping` (S1-L-4, on this union). |
| `Lang.BuildMapAtlas` / map text | PORTED | `lang.rs::map_object_name`/`map_object_name_for_tile` + `lang_tables.rs::MAP_LEGEND` | The lookup-id indexed legend table (`Lang.cs:554-572`); the fullscreen hover names ride the overlay pick's hover fn (e17's carries the `chestType[52..]` alias, research/140 §3). |
| Fullscreen map close/`M` toggle, `Main.mapEnabled` gate | PORTED | `map.rs::set_open`, `app/loop.rs::try_open_fullscreen_map` + `world_key`, `clicks.rs::handle_fullmap_close` | `Main.cs:41101-41118`, `Player.cs:29103-29119` (the open arm closes the inventory + dialog + `PlaySound(10)`), `:24967-24986` (the M-key close `PlaySound(11)`), `:4973-4978` (Escape close); W4-e18 final sweep, re-folded onto the union. |

The one DONE row that covers this module, `ROADMAP.md:1598` (`M6-SETTINGS`, "`MapScale` scales the
minimap panel `Main.cs:55273-55274`/`:55309-55329`"), **verifies PASS**: `ui::minimap::
minimap_anchor` (130) implements the two `Main.cs:55273-55275` anchor assignments and
`map_minimap_quad` (166) routes the whole rect through `ui::minimap::scale_rect` (171), which is
the C# `Matrix.CreateScale(mapScale)` batch transform (`Main.cs:55309-55329`); the maths is pinned
by `minimap_panel_scale_applies_to_every_size` and `minimap_quad_scales_the_tile_pass_by_map_scale`.
`PORT_MAP.md:185` ("`GameContent/UI/Minimap/*` + `SelectionHolder` (`CycleSelection`) →
`ui/minimap.rs` … done") also **verifies PASS** for the chrome, with the caveat that
`MinimapFrameManager`'s *style cycling* is exposed from the legacy menu rather than the in-game
options window.

## Tests

`crates/tr-core/src/map/` — 44 tests:

- `colors.rs::tests` (205-284): `lookup_packs_tiles_then_walls_then_gradients`,
  `lookup_length_is_option_counts_plus_774`, `hell_is_last`, `snow_types_point_at_snow_tiles`,
  `sky_gradient_ends_match_the_source`, `dirt_and_rock_gradient_ends`,
  `liquid_colors_are_vanilla`.
- `option.rs::tests` (712-814): `tile_518_options_use_frame_row`,
  `tile_19_options_use_the_waterfall_row`, `tile_21_options_use_frame_column`,
  `tile_89_options_special_case`, `tile_160_repeats_every_nine`, `tile_4_always_zero`,
  `unknown_tile_is_zero`, `zone_dependent_cases_default_to_zero`,
  `get_tile_type_uses_the_option` (also asserts the 461/80/529/530 substitutions at :804-806).
- `mod.rs::tests` (337-543): `stone_tile_maps_to_its_lookup`, `unlit_tile_is_not_revealed`,
  `wall_only_tile_uses_a_wall_colour`, `water_maps_to_a_liquid_colour`,
  `surface_air_uses_the_sky_gradient`, `deep_air_uses_dirt_then_rock_then_hell`,
  `negative_paint_inverts_the_colour`, `paint_multiplies_by_max_channel`,
  `painted_wall_half_inverts`, `fully_lit_returns_the_raw_colour`,
  `changed_bit_is_set_by_create_and_with_light`, `light_zero_tile_is_never_revealed`,
  `brighter_light_wins`, `out_of_bounds_is_ignored`, `reveal_rect_only_touches_lit_tiles`.
- `file.rs::tests` (601-854): `body_round_trips`,
  `variable_light_run_flush_to_row_end_round_trips`,
  `background_categories_round_trip`, `header_is_version_326_and_world_size`,
  `foreign_version_is_rejected`, `negative_table_count_is_rejected`,
  `truncated_prefix_is_rejected`, `truncated_body_is_rejected`, `loads_a_real_vanilla_map`.

`crates/tr-app/src/map.rs::tests` (669-1 161) — 22 tests: `map_round_trips_through_a_file`,
`truncated_file_is_rejected`, `mismatched_world_is_rejected`,
`loading_a_saved_map_arms_the_full_upload`, `loads_a_real_vanilla_map`,
`reveal_window_marks_tiles_around_the_player`, `update_without_a_player_is_a_no_op`,
`scale_clamp_matches_vanilla`, `wheel_zoom_matches_vanilla`, `key_zoom_matches_vanilla`,
`pan_clamps_to_the_world`, `drag_pan_is_cursor_pixels_over_scale`,
`full_quad_matches_drawmap`, `frame_rect_matches_the_4200_branch`,
`minimap_clamps_to_the_world_edge`, `minimap_quad_matches_the_panel`,
`minimap_panel_uses_the_shared_anchor`,
`minimap_quad_scales_the_tile_pass_by_map_scale`, `full_map_scale_is_its_own_axis`,
`bg_none_out_of_world`, `bg_deepest_is_mapbg3`, `bg_lihzahrd_wall_is_mapbg14`,
`bg_underground_chain`, `bg_surface_chain`, `head_icon_pos_matches_vanilla`.

`crates/tr-app/src/ui/minimap.rs::tests` (459-852) — 14 tests:
`frame_styles_match_the_source_table`, `anchors_origin_and_draw_to_match_the_reference_frame`,
`minimap_origin_raw_divides_by_map_scale`,
`minimap_panel_scale_applies_to_every_size`,
`default_frame_position_and_backdrop`,
`default_buttons_abs_top_lefts_and_hitboxes`, `hitbox_floor_22px_applies_below_it`,
`button_under_mouse_resolves_in_the_source_order`,
`missing_sheets_keep_the_floor_hitbox_without_panicking`,
`zoom_maths_match_the_source_constants`,
`style_index_resolves_config_keys_with_the_default_fallback`,
`main_m_h_truth_table`, `button_sizes_read_the_sheet_dimensions_or_zero`,
`update_yields_none_outside_the_buttons_even_when_held`,
`update_hover_without_a_press_and_click_repeat_while_held`,
`apply_matches_the_zoom_handlers_and_clamp`, `held_zoom_in_stops_at_the_clamp`.

Coverage note: there is **no** test for the four substituted `GetTileBaseOption` cases beyond
asserting they return 0, none for the `.map` writer's byte-level equality against a real
`InternalSaveMap` output (one *reader* test against a real vanilla file exists, in both crates),
and none for the fullscreen `MapBG` branches the port cannot evaluate.

## Deviations & risks

Structural (already admitted in `docs/ANALYSIS.md` or the Rust "ruling" comments):

- **D1 — `invisibleBlock` / `invisibleWall` unrepresentable (ruling 1).** behavioural · *admitted*
  (option.rs:26-28, 649-663). `MapHelper.cs:1933` and `:1989` gate the tile/wall colour to
  transparent; the Echo bit lives in the sparse `World.extras`, and both sites are hardcoded
  `let invisible = false`. Echo-painted tiles/walls therefore get a solid map colour instead of
  being hidden.
- **D2 — `fullbrightBlock` → `FLAG_FULLBRIGHT` (ruling 2).** behavioural · *admitted*
  (option.rs:28). `MapHelper.cs:1934` sets `newLight = 255`; the port re-uses the lighting flag, so
  the map light is full-bright exactly when the tile is a light source. Equivalent today, but the
  two bits are not the same bit in the source.
- **D3 — `fullbrightWall` absent (ruling 3).** behavioural · *admitted* (option.rs:650-651). The
  `sTileHeader & 0x8000` wall flag is not stored, so `MapHelper.cs:1990`'s `newLight = 255` is
  unreachable for walls. Glowing walls (e.g. `WallID` 87 variants) are dimmer on the map than in
  vanilla.
- **D4 — `WorldMap.ClearEdges` not ported.** cosmetic/behavioural · *admitted* (ANALYSIS.md:1608,
  file.rs:12). `WorldMap.cs:260` blacks the outer 40 tiles (`BlackEdgeWidth = 40`); the port leaves
  them as whatever the reveal produced, so world edges can look explored.
- **D5 — `remixWorld` fixed `false` (ruling 4).** behaviour-blocking · *admitted* (option.rs:683).
  `MapHelper.cs:2031-2035` (the `newLight = 5; return 100` fast-path) only exists for
  "for the worthy"-remix worlds, which this engine cannot generate.
- **D6 — `IsThisAMushroomTree` not ported (ruling 5).** cosmetic · *admitted* (option.rs:60-61).
  Only tile 5's option (1) differs; mushroom-tree tiles take option 0.
- **D7 — paint id source (ruling 6).** pixel · *admitted* (option.rs:62, 671; mod.rs:259-264).
  `tile.paint()` reads the tile byte; wall paint lives in `World.extras` and is substituted in
  `create_map_tile` only when `new_color` still equals the tile paint.
- **D8 — paint colour float→byte (ruling 7).** pixel · **retired (S1-L-5, 2026-11-14)**.
  The original claim — `paintColor` floats re-quantised to bytes can shift a painted map pixel by
  ±1 per channel — is gone: `paint_color_bytes` (`palette.rs`) is now the exact `WorldGen.paintColor`
  byte table (`WorldGen.cs:43921-44033`), `map_color` multiplies those bytes directly like the C#
  `MapColor`, and `map_color_uses_the_exact_paint_color_bytes` pins the table.
- **D9 — `sceneArea` / `sceneSnowiness` not ported (ruling 8).** behavioural · *admitted*
  (option.rs:691-692). `MapHelper.cs:2044-2067` prefers `SceneMetrics.SnowTileCount` when the tile
  is inside the captured scan area; the port always runs the 10-px-stride neighbour scan
  (`MapHelper.cs:2046-2062`), which is the *other* branch of the same `if`, so most tiles agree but
  a tile whose scan window straddles the `sceneArea` border can pick a different `b`.
- **D10 — reveal window is a fixed 75×47 tile box around the player.** behavioural · **not
  admitted** (ANALYSIS.md:1560-1561 states it without flagging it). C# `ExportToMiniMap` reveals the
  *lighting engine's processed area* — the camera-centred visible rect — shrunk by 28 px on every
  side (`LightingEngine.cs:252`) and intersected with the world bounds inset 40
  (`:253-255`) and `MapHelper.sceneArea` (`:256`). The port reveals
  `map.rs::WorldMap::update` (425-444)'s `px±37`, `py±23` box (75 wide × 47 tall) at 4 Hz, centred
  on the **player** rather than the **view centre** (`Main.cs:55537-55538`). Net effect: less area
  revealed per update than vanilla, and because reveal is cumulative-only with a
  max-light-wins rule, fast travel or a large/high-DPI window leaves a visible trail of
  unrevealed tiles behind. The box is also player-centred while every other map consumer is
  view-centred.
- **D11 — `.map` encoder closes a row-end full run the way the source writes it.** determinism ·
  *admitted* (file.rs:208-214 region, ANALYSIS.md:1614-1617). `encode_tiles` records a light run
  that reaches the row end with its run length (`num6 = 0`, `MapHelper.cs:3214-3225`), so the
  port's own decoder round-trips it; a byte-level vanilla `LoadMapVersion2` would misread that one
  case. Any future cross-check against vanilla's reader must expect this one asymmetry.
- **D12 — `FileMetadata` revision counter is always 1.** determinism · *admitted* (file.rs:11-13).
  `Main.MapFileMetadata.IncrementAndWrite` is replaced by a fresh metadata (`file.rs:61-65`), so
  the output is deterministic; the on-disk revision never grows and two saves of an unchanged world
  are byte-identical.
- **D13 — pre-2 `.map` formats never open.** behavioural · *not admitted* (no Rust code for
  `MapHelper.cs:3347-3585`/`:3587-3747`). `LoadMapVersion1`/`LoadMapVersionCompressed`/
  `DecompressChunk` have no counterpart, so a legacy map file is rejected as a foreign version
  (`file.rs::read_map_prefix`) rather than migrated; only v326 loads.
- **D14 — no `MapUpdateQueue`.** behavioural · *closed (W4-c35)* — `tr-app/src/map.rs::MapUpdateQueue`
  ports `MapUpdateQueue.cs:21-108` (the 262144 budget with the flag-armed-before-cap drop, the
  `ClampToWorld` edge clamps, `RangeFrame`'s inverted rect + `[start-1, end+1]` border-gated walk,
  the deterministic `UpdateTiles`-FIFO-then-`UpdateAreas`-column-major drain); the net-5/10/20
  enqueue hooks ride `app/netglue.rs` and the drain rides `frame/world.rs::map_phase`. Historical
  finding: tile edits made outside the player's reveal window
  (worldgen after load, net-received tile changes `MessageBuffer.cs:1153`,
  `NetMessage.cs:2456`, `WorldGen.cs:80347/80402/81334`) never refresh their map pixel until the
  reveal window revisits them with ≥ light. Since `update_map_lighting` keeps the *brighter* light
  and only rebuilds when `MapTile.Equals` fails, a tile whose colour changed while its light is
  already high can also stay stale until the changed bit forces a rebuild.
- **D15 — no `UnlockMapSection` / `UnlockMapTilePretty` / `DebugOptions.unlockMap`.** behavioural ·
  *closed (S1-L-3 + W4-e18)*. The reveal primitives landed S1-L-3 (`mod.rs::unlock_map_section`/
  `unlock_map_tile_pretty`, `WorldMap.cs:109-174`); W4-e18 first half landed the driver — the
  `Main.cs:61871-61883` section sweep walks `MapSections::get_next_map_draw` under its 5 ms budget,
  the `DebugOptions.unlockMap != 0 && SectionLoaded` arm reveals each section, and the exhausted
  sweep zeroes the option unless a refresh is pending. The `/map` debug row now arms
  `debug_unlock_map = 2` + `refresh_map` (`ToolkitDebugCommands.cs:486-487`) instead of walking
  every section directly.
- **D16 — `MapBG` unevaluable zones.** cosmetic · *admitted* (ANALYSIS.md:1578-1582). `ZoneDungeon`,
  `ZoneGlowshroom`, `ZoneCorrupt`, `ZoneCrimson`, `ZoneHallow`, `ZoneJungle`, `bloodMoon` and
  `ZoneGraveyard` all take the `false` path, so the fullscreen backdrop shows the wrong sheet in
  those biomes until their zone evaluation is wired.
- **D17 — minimap has no alpha config.** cosmetic · CLOSED (W4-f46 dup
  fold, `a352b16e`): `WorldMap.minimap_alpha` (`Main.cs:66753` seed 1), the
  MapAlphaUp/Down keys step it ±0.015 (`Player.cs:24921-24928`), the
  draw-side clamp 0.01..1 (`Main.cs:55523-55530`, `map::map_alpha_clamp`),
  the backdrop + tile pass (`Color(b×4)`, `MapRenderer.cs:438`) + layer walk
  read the live alpha. The C# has NO settings key for it — the alpha is
  key-driven runtime state, not config (the old seed's "config key" reading
  was wrong; e18 `249a0c1b` reached the same conclusion).
- **D18 — `mapStyle` switcher's style-2 draw half absent.** behavioural · *not admitted*
  (keys half CLOSED, W4-f46 dup fold `a352b16e`). The
  switcher itself is landed (the inventory icon row `Main.cs:41087-41120` + the Tab cycle key
  `Player.cs:24949-24965`, W4-f46 closeout), and the style-2 KEY rows now step
  `WorldMap.overlay_scale`/`overlay_alpha` (`Player.cs:24930-24947`, seeds 2.5/0.35
  `Main.cs:66754-66755`, clamps `:55550-55566`); only the `mapStyle == 2`
  screen-overlay map DRAW (`Main.cs:55548-55578`) still has no counterpart
  (style 0 correctly draws no map — `DrawMap`'s branch chain has no style-0 arm).
- **D19 — map icon overlays entirely missing.** behavioural · **closed across
  branches (S1-L-4 + W4-f29/W4-e17, keep-one PENDING at merge)**. The six
  `Map/*Layer*.cs` layers landed S1-L-4 (`map_layers.rs`, on this union); the
  nine `Main.DrawMap` overlay pieces (pylon lines/click/pan, boss/town heads,
  death markers, revenge coins, misc portals+golf, minimap heads, hover names)
  are ported on BOTH siblings — research/140 adjudicates e17 as the skeleton
  (`8500c3e5`) with f29's six fixes F1-F6 folding onto it (`d5aa3621`). The
  ping detector (`Main.cs:55486-55495`) is `clicks.rs::handle_fullmap_ping`.
  Until the overlay pick merges, this union alone still lacks the nine pieces.
- **D20 — head icon is a hand-drawn stand-in.** pixel · *admitted* (ANALYSIS.md:1584-1593).
  Vanilla renders an 84×84 `MapHeadRenderer` render target; the port draws the head, eyes, whites
  and hair plus the outline rings directly, so a head will not match vanilla pixel-for-pixel, and
  only the local player's head is drawn (no 255-player loop).
- **D21 — four `GetTileBaseOption` cases substituted.** cosmetic · *admitted* (option.rs:102-108).
  `461` (Corrupt/Crimson/Hallow tiles take a zone-dependent option) and `80`/`529`/`530`
  (cactus/biome-influence tiles) always resolve to option 0, so those tiles show their base colour
  in every biome.
- **D22 — `Main.mapDelay` and `Main.refreshMap`/`Main.clearMap` not modelled.** behavioural ·
  *partial (W4-e18 residual census + final sweep, re-folded onto the union)*. The
  `refreshMap`/`clearMap` statics landed the residual census (`WorldMap::refresh_map`/
  `clear_map`, the `DrawToMap` tail + the `clearMap` consume in `prepare`); the `mapDelay`
  first-frames skip (`LightingEngine.cs:82-89`) landed the final sweep
  (`WorldMap::map_delay`, ctor 2 `Main.cs:66504`, world-load re-arm `WorldGen.cs:6947`
  via `reset_map_delay` + `enter_loaded_world`, gating `update`'s head). The
  content-lost-triggered full refresh (`MapRenderer.cs:363-377`) stays unmodelled — the port's
  `uploaded`/`needs_full_upload` pair is equivalent for a correct upload but never re-runs a
  full sweep after a GPU reset.
- **D23 — the `.map` deflate stream is not byte-identical to vanilla's (S1-L-5).** determinism ·
  *admitted* (map.rs:706-711, `tools/MapDeflateHarness`, ANALYSIS.md "The `.map` deflate stream").
  The wrapper semantics are faithful (raw deflate, no zlib wrapper, at the `CompressionMode.Compress`
  ctor default = `CompressionLevel.Optimal`/zlib level 6 — the port previously used `fast()`), but
  the vanilla runtime's `clrcompression.dll` (.NET FW 4.8.1) embeds a Microsoft-patched zlib whose
  output no stock zlib reproduces: for the frozen S1-L-5 body it emits 35 414 bytes against
  zlib 1.3.1's 36 300 at *every* (level, memLevel, strategy) combination probed via `zlib-rs`
  (`zrs-probe`), while .NET 10's own stock-zlib `DeflateStream` produces 36 295. The committed
  reference (`tools/MapDeflateHarness/fw48-reference.deflate`) inflates back onto the port's body
  byte-for-byte, so the layer interoperates — only the compressed bytes differ. The chunking is
  chunk-invariant on the vanilla runtime (the `fw48/` generator reports "chunked == single: yes";
  modern .NET's is not).

Risk summary (updated W4-f46 final-retire): D14 (map invalidation) closed W4-c35, D19 (icon
layers) closed across branches (S1-L-4 + the f29/e17 overlay pick, keep-one pending at merge),
D17 (minimap alpha) closed the dup fold. The remaining behavioural gaps are D18's
`mapStyle == 0/2` draw halves (the switcher + keys landed) and D22's content-lost refresh
(no GPU-reset path in the port).

## Task seeds

- **S — Port `MapUpdateQueue` + `WorldMap.UpdateType`.** The queue
  (`MapUpdateQueue.cs:21-108`), `WorldMap.UpdateType` (`:65-93`) and the call sites
  (`WorldGen.cs:81334`, `MessageBuffer.cs:1153`, `NetMessage.cs:2456`,
  `WorldGen.cs:80347/80402`) are small and self-contained; expose them as
  `tr-core/src/map/mod.rs::{queue_add, queue_update}` fed from `tr-app`'s edit/net glue. Blocks
  nothing; fixes the stale-pixel class of D14.
- **S — Wire `ClearEdges`** (`WorldMap.cs:260`, `BlackEdgeWidth = 40`): one loop over the outer 40
  tiles, run after load and after worldgen. Depends on nothing; fixes D4.
- **S — Minimap alpha config** (`Main.cs:55523-55532`): DONE differently —
  no config key exists in C#; the alpha is key-driven state
  (`WorldMap.minimap_alpha`, W4-f46 dup fold `a352b16e`). D17 closed.
- **M — `MapBG` zone branches** (`Main.cs:56342-56444`): port `ZoneDungeon`, `ZoneGlowshroom`,
  `ZoneCorrupt`/`ZoneCrimson`/`ZoneHallow`/`ZoneJungle` and the `bloodMoon` doubling off
  `SceneMetrics`, plus the `ZoneGraveyard` wall test. Depends on the weather/SceneMetrics module
  (research row 16) for the shared zone evaluation. Fixes D16.
- **M — The eight map overlay layers** (`Map/*Layer*.cs`, `MapIconOverlay.cs`,
  `MapOverlayDrawContext.cs`, `Main.DrawNPCMapIcons`, `Lang.BuildMapAtlas`): pings, team spawn,
  pylon links, boss bags, Digtoise, NPC/chest labels. ~~Needs the `IMapLayer` seam, the ping network
  message and the icon texture whitelist in `assets.rs`. Depends on the netcode research row 17.
  Fixes D19.~~ **DONE (S1-L-4 + W4-f29/W4-e17, keep-one pending)** — the six layers landed S1-L-4,
  the nine `Main.DrawMap` overlay pieces are ported twice on the siblings (research/140: e17
  skeleton + f29 F1-F6); `Lang.BuildMapAtlas` is `lang.rs::map_object_name`.
- **M — Fullscreen map input parity**: ~~`mapStyle` cycling (0/1/2, `Main.cs:41101-41118`), the
  `mapOverlayScale` screen-overlay map (`:55548-55578`), minimap zoom key/wheel wiring, and
  `DebugOptions.unlockMap` (`Main.cs:61873-61882`, `WorldMap.cs:109-174`). Depends on the UI
  framework research row 18 for the in-game options rows.~~ **PARTIAL (W4-e18 + W4-f46)** — the
  `mapStyle` cycling (Tab key + icon clicks), the minimap zoom/alpha keys (both styles) and
  `DebugOptions.unlockMap` landed the first half + closeout + dup fold; the `mapOverlayScale`
  screen-overlay map DRAW (`mapStyle == 2`) stays unported (D18's draw half).
- **L — Replace the four substituted `GetTileBaseOption` cases**: `461` needs `SceneMetrics`
  ZoneCorrupt/Crimson/Hallow (blocks on row 16), `80` needs `WorldGen.GetCactusType`, `529`/`530`
  need `WorldGen.GetBiomeInfluence` (blocks on row 12, worldgen). Fixes D21.
- **L — Byte-exact `.map` writer verification**: capture a vanilla `InternalSaveMap` byte stream and
  assert equality against `encode_tiles` (the reader already has a real-file oracle); also decide
  whether the row-end `num6` run in D11 should be emitted as vanilla writes it and the decoder
  taught the vanilla reading instead of matching our own encoder.
- **L — Legacy `.map` migration**: port `DecompressChunk` + `LoadMapVersion1` /
  `LoadMapVersionCompressed` (`MapHelper.cs:3325-3747`) so old sidecars upgrade to v326 instead of
  being rejected. Fixes D13.
## W4-e18 residual census (2026-11-21, code-only, second half)

Census of every remaining `MapRenderer.cs` / `WorldMap.cs` member and the
fullscreen-map input block against this branch (base main `a725acb0`, e18 tip
`e5b12711`), with the sibling claims read first: `origin/task/W4-e17-map-overlay-layers`
does not exist on the remote (0 commits, nothing claimed);
`origin/task/W4-f29-map-overlay-2` (tip `09f71b61`) claims the nine `Main.DrawMap`
overlay pieces (pylon lines + click/pan state, boss/town heads, death markers,
revenge coins, misc portals/golf, minimap heads, hover names) - none of them
overlap this census.

### Already covered on this branch (verified, no work)

| Piece | C# | Port |
| --- | --- | --- |
| `QueueChange` + 2048 threshold + `ChangesQueued` | `MapRenderer.cs:288-304`, `:37-56` | `map.rs::queue_change`/`changes_queued` (W4-e18 first half) |
| `DrawToMap` head queue-clear + `mapEnabled` gate | `:148-160`, `:130-133` | `draw_to_map_queues` + both consume arms gated (e5b12711) |
| `DrawToMap_Section` | `:306-355` | W4-f46 union: the real pixel paint - `map.rs::draw_to_map_section`/`section_pixels` (e18's dirty-rect push replaced, research/84 §6.2) |
| `DrawAreaToMap`/`DrawChangesToMap` walk | `:203-286` | the dirty-list upload in `prepare` - see deviation below |
| `CheckMapTargets` | `:357-378` | `uploaded`/`needs_full_upload` (single texture, no content-lost) |
| `MapRenderer.DrawMap` (target-grid blit) | `:380-448` | `map_full_quad`/`map_minimap_quad` one-texture quads |
| `WorldMap.UpdateLighting/UpdateType/QueueUpdate` | `WorldMap.cs:49-107` | `tr-core map/mod.rs` free fns + `WorldMap` wrappers (S1-L-3) |
| `UnlockMapSection/UnlockMapTilePretty` | `:109-174` | `unlock_map_section`/`unlock_map_tile_pretty` + the sweep arm (S1-L-3 + e18) |
| `Load`/`TryGetMapPath`/`Save`/`ClearEdges` | `:176-285` | `load_map_file`/`save_map_file`/`persist::map_path`/`clear_edges` |
| `WorldSections` grid + 5 ms sweep + driver | `WorldSections.cs`, `Main.cs:61863-61899` | `map_sections.rs` + `frame/world.rs` (354b50d2/a19852bb; W4-f46 union moves the call to `frame/draw.rs` + fixes the paused else-arm, research/84) |
| Fullscreen zoom/pan/drag maths + wiring | `Main.cs:55342-55424`, `Player.cs:24774-24793`, `:24898-24908`, `:25190-25197` | `map_scale_clamp`/`map_wheel_scale`/`map_key_scale`/`map_clamp_pos`/`map_drag_delta` + `update_map_input`/`loop.rs` wheel - landed |
| `MapTile` bitfield + `Create`/`WithLight`/`Clear`/`Equals` | `MapTile.cs:1-101` | `tr-core map/mod.rs::MapTile` (S1-L-*) |

### Ported this pass (the residuals)

1. **`Main.clearMap`/`mapReady`/`loadMapLock`/`loadMapLastX` statics** (`Main.cs:789-805`) -
   `WorldMap` gains `clear_map`/`map_ready`/`load_map_lock`/`load_map_last_x`. The `DrawToMap`
   tail (`MapRenderer.cs:196-199`) sets `map_ready`, zeroes `load_map_last_x`, lifts
   `load_map` + `load_map_lock` inside `draw_to_map_queues`; the `.map` load arms
   `clear_map`/`load_map`/`load_map_lock` (`WorldMap.cs:211-213`). The `clearMap` consume
   (`MapRenderer.cs:161-173`, gated on `mapEnabled`) rides `prepare` as a full re-upload -
   pixel-identical to clear-then-redraw on the single texture.
2. **`Main.DrawMap`'s `mapReady` gate** (`Main.cs:55283`, `!mapEnabled || !mapReady`) -
   both draw branches (`frame/draw.rs` minimap + fullscreen) now gate on `map_ready`;
   nothing draws before the first completed `DrawToMap`.
3. **`WorldMap.Clear`** (`WorldMap.cs:249-258`) - `WorldMap::clear` zeroes every tile.
4. **`/clearmap` command** (`ToolkitDebugCommands.cs:497-503`) - the debug panel's World
   tab gains row 13 "Clear Map": `clear_map = true; clear(); refresh_map = true`. The
   `/map` row now also arms `clear_map` (`:483`, both toggle arms - previously missing).
5. **The drag's `!CaptureManager.Instance.UsingMap` gate** (`Main.cs:55342`) -
   `update_map_input` skips the whole drag block (position AND grab anchor untouched)
   while the capture interface owns the map.

### Deviations (recorded, deliberate)

- **`DrawAreaToMap`/`DrawChangesToMap`'s `IsChanged` + run-merge walk is unmodelled.**
  The C# gates each upload on `MapTile.IsChanged` (`MapRenderer.cs:215`/`:234`), consumes
  the flag (`WorldMap.ConsumeUpdate`, `:250`/`:270`/`:280`) and merges equal adjacent
  changed tiles into one `MagicPixel` rect (`:264-285`, column-first then row, bounded by
  the target grid). The port's dirty list carries exactly the changed tiles and uploads
  one texel each - identical pixels, different cost; the `IsChanged` bit is never consumed
  (it only feeds the C#'s own redraw filter, and the `.map` writer never reads it). The
  `maxMapUpdates = 250000` per-call budget (`Main.cs:66745`, `MapRenderer.cs:178-188`/
  `:217`/`:236`) is unmodelled for the same reason - the port uploads everything dirty in
  one frame; a single reveal window is <= 169x124 tiles, so the budget never binds in
  practice outside the staged load, which the port completes in one bulk write.
- **`loadMapLastX` stays 0** (the tail's reset value): the port's bulk write completes in
  one frame, so the `WorldGen.cs:6823-6825` load-progress wait and the `Netplay.cs:506-508`
  connect-status percentage have no intermediate states to report. The fields exist for
  the record; the MP consumers are unported.
- **`CheckMapTargets`' content-lost arm** (`MapRenderer.cs:365-374`, `refreshMap` +
  `clearMap` on GPU content loss) stays unmodelled - the port has no render-target grid to
  lose (D22, unchanged).

### Verify-only / phantom rows

- `WorldMap.Update(x, y, light)` (`WorldMap.cs:34-37`) and `SetTile` (`:39-42`): `Update`
  has NO caller anywhere in the decompiled game (grep `\.Map\.Update\(` = 0 hits);
  `SetTile`'s callers are all inside `MapHelper`'s three load formats - the port's
  `decode_tiles` writes the flat array directly (the same store). Nothing to port.
- `WorldMap.IsRevealed` (`:44-47`) - `tile_revealed` (cite-repair row, landed).
- `Main.mapMinimapAlpha` fade (`Main.cs:55523-55532`) - LANDED (W4-f46 dup
  fold `a352b16e`: `WorldMap.minimap_alpha` + the ±0.015 keys + the clamp +
  the tinted draws; D17 closed — the "config key" framing was wrong, C# has
  none).
- The world-clear map reset (`WorldGen.cs:6962-6985`: `Map.Clear()` gated on `mapReady`,
  `clearMap = true; mapTime = 0; updateMap = null; mapReady = false; refreshMap = false`)
  - the port's `swap_world` builds a fresh `App` with a fresh `WorldMap` (all fields at
  their `Main` static defaults), which is the same end state; no incremental reset needed.
- `PanTargetMapFullscreen` (`Main.cs:55361-55384`): the only producer that arms it is
  `TeleportPylonsMapLayer.cs:105` - claimed by W4-f29 (its item 2, the pylon click/pan
  state). Skipped here; the four `Player.cs` clear sites and the `resetMapFull` clear
  (`Main.cs:55418`) ride that task's seam.

Witnesses (written, not run - code-only): 3 `map::tests::w4_e18_*` (draw tail, load arms
clear+lock, Clear zeroes), 2 `debug_panel` (the `/map` row's clearMap arm inside
`s1_l_3_the_reveal_row_unlocks_the_map`,
`w4_e18_the_clearmap_row_wipes_the_tiles_and_arms_the_refresh`), 1
`app::w4_e18_map_drag_capture_tests` (UsingMap skips the whole drag block), plus the
16-row World-tab count update (`s1_e_29_the_world_tab_keeps_its_sixteen_rows`).
## W4-e18 first-half census (2026-11-21, code-only)

Full inventory of `Main.DrawMap` + its helper block (`Main.cs:55283-56650`, the range
this task was seeded with) for pieces NOT among W4-f29's nine overlay claims
(`origin/task/W4-f29-map-overlay-2`, census `e1ffdefc`, ports `09f71b61`/`9320326c`/
`1de1ad19` — read in full). The f29 nine: pylon lines/click/pan, boss/town heads, death
markers, revenge coins, misc portals+golf, minimap heads, hover names. The e17 sibling
branch died claim-only (0 commits); nothing of its scope was landed elsewhere.

### The seed's six candidates, resolved

| Candidate | Verdict | Evidence |
| --- | --- | --- |
| map background/edges draw | LANDED | `map.rs::map_bg_index`/`draw_map_background` (`Main.cs:56342-56444`), `map_full_quad`'s `[10,10]..[w-11,h-20]` window (`:55425-55430`+`:55591-55607`), the minimap edge clamp (`:55580-55589`). |
| grid lines | PHANTOM | No grid-line draw exists anywhere in `DrawMap`/`MapRenderer.DrawMap` (the only "grid" in the range is the debug sections overlay, `:56311-56326`, dev-only). |
| map legend | PHANTOM | No legend draw exists in the range (grep `Legend` over Main.cs's map block = 0 hits). |
| player-position arrow rotation | PHANTOM | The player marker is a HEAD icon (`MapPlayerRenderer.DrawPlayerHead`, `:55954`), never an arrow and never rotated; the fullscreen head is landed (`frame/draw.rs` + `entities.rs::draw_player_map_icon`), the minimap head loop is f29 item 8. |
| fullscreen border/frame | LANDED | `map.rs::map_frame_rect` + `draw_full`'s `Images/Map` 928x248 scroll frame (`Main.cs:55431-55485`); the minimap chrome `DrawTo`/`DrawForeground` (`:55546`/`:55769`) is landed (`ui/minimap.rs`). |
| zoom-level indicator | PHANTOM | No zoom-level readout exists in the range; zoom is input-only (`map_scale_clamp`/`map_key_scale` landed). |

### UNCLAIMED + MISSING (ported by this census)

1. **Fullscreen close button** — `Main.cs:56014-56036`: `MapIcon[0]` at
   `(10, screenHeight - 40)` (`showFrameRate`'s `-15` shift never fires — F10,
   `:18492-18514`, is unported), the inclusive 32x30 hover box flips to `MapIcon[4]`
   white-255 (`Color(n,n,n,n)` tint, grey-130 unhovered), sets `mouseInterface` (the
   port already suppresses world use behind the whole open map,
   `world_sim/editing.rs`), and a press plays `PlaySound(10)` + `mapFullscreen = false`.
   Ported: `ui/chrome.rs::fullmap_close_button_*` + `draw_fullmap_close_button`,
   `app/clicks.rs::handle_fullmap_close` (after the ping detector, the C#'s
   `:55486` < `:56024` order), drawn after the hover text (`frame/draw.rs`).
2. **Fullscreen vanilla cursor + thick border** — `Main.cs:56037-56040`
   (`!gameMenu && !InGameUI.IsVisible`): `DrawCursor(DrawThickCursor())`.
   `DrawThickCursor` (`:48064-48134`): while `ThickMouse` (`MouseBorderColor !=
   Transparent`, `:2599`; default `Color(64,64,64,64)`, `:66514`) four `Cursors[11]`
   rings at the `(0,1),(1,0),(0,-1),(-1,0)` offsets `+ (2,2)`, origin `(2,2)`, scale
   `cursorScale * 1.1`, border tint; returns the `(2,2)` bonus. `DrawCursor`'s
   non-gamepad branch (`:48046-48048`) adds the bonus to the shadow + sprite. Ported:
   `ui/cursor.rs::draw_thick`/`thick_cursor_bonus`/`draw_with_bonus`, `Cursor_11`
   packed (`assets.rs`), wired at the fullscreen draw tail; the OS pointer now hides
   over the map too (`frame.rs`, `FocusHelper.cs:160-163`).

### Verify-only / phantom rows (recorded so no future census re-hunts them)

- `OnPostFullscreenMapDraw` (`Main.cs:3060`, fired `:56062-56064`): ZERO subscribers
  anywhere in `decompiled/` + `decompiled-libs/` (grep `OnPostFullscreenMapDraw +=` =
  0 hits) — a tModLoader-style hook, dead in vanilla. Nothing to port.
- The `inventoryScale = 3` swap around `MouseText` (`:56044-56050`): `MouseTextInner`
  draws the plain hover string at `Vector2.One` (`:20319`) and never reads
  `inventoryScale` — the swap is a no-op for the map's text. The port's hover draw
  (`hud_text` 0.85) is the existing hover-text convention, unchanged.
- `Main.cs:62147-62181`'s fullscreen post-`DrawMap` batch (`DrawFPS`,
  `DrawPlayerChat`, `DrawPendingMouseText`, `DrawCursor`): `DrawFPS` rides
  `showFrameRate` (F10, unported — the close button's `-15` shift with it);
  `DrawPlayerChat`/`DrawPendingMouseText`/the cursor are Draw-interface pieces
  outside this census's `DrawMap` range; the cursor half is ported above via
  `:56037-56040` (the same `DrawCursor(DrawThickCursor())` call).
- `DrawDebugMapOverlays` (`:56299-56340`) — DebugOptions gates, dev-only (f29
  census already recorded it; re-verified).
- `mouseInterface = true` (`:55386`) — the port suppresses use-item/use-tile behind
  the whole open map (`Player.cs:24798-24807`, `world_sim/editing.rs:1944`), a
  superset of the flag's effect; no separate field needed.

Witnesses (written, not run - code-only): 2 `ui::chrome::tests::w4_e18_*` (close-button
geometry + hover flip), 3 `app::clicks::w4_e18_fullmap_close_tests` (press closes,
outside-box no-op + inclusive corner, map-shut no-op), 1
`ui::cursor::tests::w4_e18_thick_cursor_gate_and_bonus` (the ThickMouse packed-colour
gate + the (2,2) bonus).

## W4-e18 → union fold status (2026-11-21, code-only)

**For the e18 final-sweep (e18d) still working on
`task/W4-e18-minimap-reveal-queue`:** the dup-reconcile union lives on
`task/W4-f46-minimap-reveal-2` (research/84 EXECUTED) and is the branch the
merger keeps — the separate e18/f46 versions are dropped. As of this note the
union has folded e18 through tip `670c948d` (close button `ebd54f66`, thick
cursor `254f309b`, census close `670c948d`, cherry-picked per-hunk as
`c06bc2fe`/`6dea2b66`/`0fc306bc`).

Contract for any further e18 commits (zoom-keys/pan-inertia etc.):

- Keep committing on the e18 branch as usual; the union owner re-folds each
  new commit past `670c948d` (fetch tip → cherry-pick → per-hunk resolve).
- Target the UNION's seams, not the old e18 driver shape:
  - the reveal driver is `frame/world.rs::map_reveal_draw` (field-borrow
    associated fn) called from `frame/draw.rs` on the DoDraw half ahead of
    `WorldMap::prepare` — NOT `map_reveal_driver` on the update half;
  - the sweep/consume split: `map_reveal_sweep_core`/`_fields` (GPU-free
    state) + `draw_to_map_section` paint, `map_reveal_consume_fields`
    with the paused `ChangesQueued` else-arm;
  - the click cluster order: `handle_fullmap_ping` → `handle_fullmap_close`
    → `handle_bestiary_clicks` inside `in_world_click_cluster`;
  - the fullscreen draw tail order: hover text → close button → thick
    cursor (`draw_thick` + `draw_with_bonus`) → player head icon;
  - `/map` routes through `WorldMap::unlock_map(&mut i32, bool)`
    (`App.debug_unlock_map`), not inline option arming.
- Zoom-key/pan additions belong with the existing `map_scale_clamp`/
  `map_key_scale` seams and the fullscreen drag block; avoid re-adding an
  update-half reveal driver call — the S1-B-8 orchestrator pin
  (`frame.rs::s1_b_8_phase2_the_world_branch_calls_the_named_phase_fns`)
  walks the frozen phase list and the driver is deliberately absent from it.

**Closeout re-fetch (2026-11-21, second pass):** `git ls-remote origin` +
`git fetch` re-read the e18 tip at closeout time: **still `670c948d`** — zero
commits past the folded base, nothing to re-fold. **SUPERSEDED by the
shared-gap/dup pass (third re-fetch, same day): the tip is now `75289a2db`**
— the e18d final sweep landed 10 commits (`0bd346e78`…`75289a2db`); its
`249a0c1b` key/alpha half is folded into the union per research/84 §10.1
(the keep-one table). **SUPERSEDED AGAIN by the final-retire pass (fourth
re-fetch, same day): the tip is STILL `75289a2db` and ALL NINE remaining
commits are now RE-FOLDED onto the union** (`4ad0f041` TryOpeningFullscreenMap
→ `736c86ce`, `029154fe` Load catch → `5566bf25`, `2fa072d0` mapDelay →
`cd24857e`, `8ec77f0f`+`2e1f97c8` draw-head close → `7d050b7c`, `0d99d968`
draw-tail chat → `b767ef91`, plus the three docs commits into this file /
research/84 / ANALYSIS / PORT_MAP). The e18 branch is fully absorbed — the
merger takes THIS branch and drops e18 wholesale; the re-fetch-at-merge step
only needs to confirm e18 has not advanced past `75289a2db`. The zoom-key half of the
e18d brief turned out to be ALREADY LANDED on the shared base (`a725acb0`:
`map.rs::map_key_scale` `Player.cs:24898-24908` + the held-key wiring in
`app/mod.rs::update_map_input` `:1427-1438` against the rebindable
`MapAction::ZoomIn/ZoomOut` rows, defaults `Add`/`Subtract`
`Main.cs:67190-67191`); the pan-inertia half is a PHANTOM (see the closeout
census below). If e18d still lands commits later, the re-fold recipe above
stands unchanged (fetch tip → cherry-pick each commit past `75289a2db` →
per-hunk resolve onto the union seams).

## W4-f46 closeout census (2026-11-21, code-only, third pass)

The e18d brief's remaining candidates re-censused against the union
(`af1bc80c`) and the C# input block, siblings re-read first
(`origin/task/W4-e17-map-overlay-layers` @ `8500c3e5` — revived from the
claim-only death recorded above; `origin/task/W4-f29-map-overlay-2` @
`e18bf6dd`, census 9/9 ported). Neither sibling claims an input-block piece.

| Candidate | Verdict | Evidence |
| --- | --- | --- |
| fullscreen PAN inertia/friction (`Main.cs:55342` drag-release velocity) | PHANTOM | The drag block (`:55342-55359`) is grab-anchor + per-frame cursor delta only: press re-grabs `grabMapX/Y`, hold applies `-delta * 0.06255 * (16/scale)`, release does NOTHING (no velocity capture, no coast). Grep `mapSpeed|mapAccel|mapMomentum|mapDrag|mapFullscreenVel|inertia|friction` over `Main.cs` = 0 hits. The only eased pan is `PanTargetMapFullscreen`'s 0.15 lerp (`:55361-55385`) — the pylon-marker pan, CLAIMED twice (f29 `map_pylons.rs::pan_target_step` wired in `update_map_input`; e17 `map.rs::prepare`'s lerp). Nothing to port. |
| fullscreen map zoom +/- keybinds | ALREADY LANDED (shared base `a725acb0`) | `map.rs::map_key_scale` (`Player.cs:24898-24908`, ×1.05/×0.95) + `app/mod.rs::update_map_input`'s held-key rows against `MapAction::ZoomIn/ZoomOut` (defaults `Add`/`Subtract`, `Main.cs:67190-67191`), driven from `world_sim/player.rs`'s `map_open` tail. The wheel half (`Player.cs:25190-25197`) rides `loop.rs`'s `MouseWheel` arm. |
| minimap zoom keybinds (style 1, not fullscreen) | UNCLAIMED + MISSING → PORTED this pass | `Player.cs:24911-24920`: held `MapZoomIn`/`MapZoomOut` scale `Main.mapMinimapScale` ×1.025/×0.975 (the button constants, `MinimapFrame.cs:61-63`). Ported: `ui/minimap.rs::apply_key_zoom` + `app/mod.rs::update_minimap_zoom_keys` (the `mapStyle == 1` gate only — the C# arm has no `mapEnabled` check), wired in `world_sim/player.rs`'s not-fullscreen else-arm — the C#'s own `else` of `if (Main.mapFullscreen)` (`:24898`/`:24909`). |
| map-style cycle KEY (Tab) | UNCLAIMED + MISSING → PORTED this pass | `Player.cs:24949-24965`: `cMapStyle` (default `Tab`, `Main.cs:67195`) press edge cycles `Main.mapStyle` 0→1→2→0 with `PlaySound(12)`, NOT-fullscreen arm only. Ported: `app/loop.rs::world_key`'s `MapAction::Style` arm (the rebindable row; `world_key` is the port's press edge, like the `Full` row). The style ICON row's clicks were already landed (`clicks.rs:1572-1602`, `Main.cs:41087-41120` — the "map-style cycle button" candidate resolves to it). |
| `mapAlphaUp/Down` keys (PageUp/PageDown) | PORTED (W4-f46 dup fold `a352b16e`, supersedes this row's D17 deferral) | `Player.cs:24921-24928`/`:24940-24947` ride `mapMinimapAlpha`/`mapOverlayAlpha` — the state now exists (`WorldMap.minimap_alpha`/`overlay_alpha`, seeds 1/0.35 `Main.cs:66753-66755`), the ±0.015 steps + the 0.01..1 clamps ride `update_minimap_zoom_keys`, the backdrop/tile/layer draws read the live alpha. D17 closed, D18 keys-half closed. |

Witnesses (written, not run - code-only): 1
`ui::minimap::tests::w4_f46_key_zoom_matches_the_button_constants_and_order`
(constants, both-held order, clamp), 2
`app::w4_f46_minimap_zoom_key_tests` (held `Add` scales ×1.025 + the rebind
moves the row; the style-1/not-fullscreen/mapEnabled gates), 2
`app::loop::tests::w4_f46_*` (Tab cycles 1→2→0→1 + rebind; dead while
fullscreen).

## W4-f46 × f29/e17 overlay composition (2026-11-21, closeout cross-note)

> **UPDATED (final-retire pass, same day): the keep-one below is RESOLVED —
> f29 EXECUTED research/140 on its branch.** The f29 tip is now `809d08238`:
> a rebuild onto the e17 `8500c3e5` skeleton wholesale (`352644d2`) + the
> six cited fixes F1-F6 (`e78178e1`/`f8df2276`/`22df1dee`/`8ccb7966`/
> `8a29b825`/`b50d96ca`) + the secondary witness folds (`c19521b3`) + the
> docs dedupe (`c15a1318`). **The merger takes the f29 TIP ONLY** — its tree
> IS the e17 skeleton + fixes + 34-witness union; the e17 branch and the
> pre-rebuild f29 commits (`map_overlay.rs`/`map_pylons.rs` era) must not
> re-land (research/140's banner). The paragraphs below were written against
> the PRE-rebuild tips (`e18bf6dd`/`8500c3e5`) and stay as the seam record,
> with these corrections: the pan step now lives in `WorldMap::prepare`
> delegating to the pure `map.rs::pan_target_step` (F6) — NOT f29's old
> `update_map_input` site; the tip carries e17's pan field set
> (`pan_target: bool` + `pan_target_end` + `pan_cancel_press` + `pylon_tile`
> + `cancel_wormhole`), which converts 1:1 onto the union's
> `pan_target: Option<[f32;2]>` (keep the union's representation, fold the
> step + arming); the tip's `prepare` reset arm DOES clear the pan (dedupes
> against the union's `recentre`); the draw wiring is e17's C#-order form
> (fullscreen layer walk LAST at `:55996`, minimap FIRST at `:55703`) with
> the `map_left_press` snapshot + the `map_layers_draw` 4-new-field
> `MapIconInputs` extension (`map_fullscreen`/`pylon_tile`/`screen`/
> `left_press`) — the union's two call sites gain those fields at merge.
> The final merge instruction is research/84 §10.3.

Read in full from git objects: `origin/task/W4-f29-map-overlay-2` @ `e18bf6dd`
(census `e1ffdefc` → ports `09f71b61`/`9320326c`/`1de1ad19`/`a920b704c` →
review `a0516edf` → docs `e18bf6dd`; 9/9 claimed pieces ported) and
`origin/task/W4-e17-map-overlay-layers` @ `8500c3e5` (revived — the
claim-only death recorded in the e18 residual census is stale; phases
`98e2ae01`…`8500c3e5` port the SAME nine pieces: NPC heads
`a8b02614`, player pins/death/misc/revenge `adb5ccd9`/`ce699449`, pylon
lines/click `5c5ad091`). Both branch off `a725acb0` — the union's own base —
so neither contains the e18/f46 reveal work and neither touches the reveal
driver, `map_sections.rs`, the `WorldSections` grid or the `/map` flag path.

**Keep-one warning (the big one): f29 and e17 are DUPLICATE ports of the same
nine overlay pieces** (f29: `map_overlay.rs` + `map_pylons.rs`; e17:
`map_npc_icons.rs` + `map_player_icons.rs` + the `map_layers.rs` pylon half).
The merger takes ONE branch's overlay implementation, never both — the
sheets (`NPC_Head_Boss_*`, `MapDeath`, `Extra_173/175/176`, `Black_Tile`,
`UI_quickicon1`), the `Lookups` fields and the `WorldMap` pylon/pan state
would double. e17's own phase 1b (`2360c5dd`) already deduped its internal
parallel lanes; the cross-branch dedupe is the merger's call (e17 carries the
c21 hover-name base `98e2ae01` + the atlas measurement; f29 carries the
review round `a0516edf` — read both before picking).

**Composition with the union, verified by inspection (no conflicts with the
closeout code):**

- **Draw order.** The union's fullscreen tail is hover text → close button
  (`Main.cs:56014`) → thick cursor (`:56037`) → head icon. Both siblings
  insert their overlay draws BEFORE the hover text — revenge/misc
  (`:55922-55923`), NPC heads (`:55926`), the 255-head loop (`:55931-55976`),
  death markers (`:55977-55986`), the revenge money row (`:55988-55995`) —
  i.e. between `map_layers_draw` and the close button, exactly the C# order.
  The union's close/cursor tail stays last; no shared lines, no double-draw.
  One seam note: the union's single head icon draws AFTER the cursor (the
  base's placement); the siblings' head loop replaces that draw and belongs
  at `:55931` (before the close button) — the merger re-homes it with the
  loop, per the C# write order. The minimap inserts (heads/death between the
  layer walk and `draw_foreground`, `:55705-55778`) slot into the union's
  style-1 branch the same way.
- **The reveal driver is untouched by both.** `map_reveal_draw` runs ahead
  of `WorldMap::prepare` on the DoDraw half; the siblings' draws read
  `map.pos`/`map.scale` after `prepare` — no interaction. The `map_ready`
  gate (`Main.cs:55283`) wraps both draw branches, so their overlays inherit
  it for free.
- **The pan state is claimed TWICE and lands differently:** f29's
  `pan_target: Option<[f32;2]>` steps in `update_map_input`
  (`map_pylons.rs::pan_target_step`, cancel on `ui_left_press`); e17's
  `pan_target: bool` + `pan_target_end` + `pan_cancel_press` steps in
  `WorldMap::prepare` ahead of the clamps (the C# position,
  `Main.cs:55361-55385` < `:55387`). Keep-one with the overlay pick. The
  movement-key pan cancel gap BOTH shared (`Player.cs:24776`/`:24781`/
  `:24786`/`:24791` — each `controlUp/Down/Left/Right` sets
  `PanTargetMapFullscreen = false`; neither sibling's pan-key block cleared
  the target) is CLOSED on the union (W4-f46 shared-gap pass): the union
  carries `WorldMap.pan_target: Option<[f32;2]>` (f29's representation,
  `None` = the C# flag false; e17's bool+end pair converts 1:1 if e17 wins)
  and each held movement row in `update_map_input` sets it to `None` before
  its pan step. The overlay pick folds only its STEP + ARMING hunks onto
  this state; its duplicate field drops. The `resetMapFull` pan clear
  (`Main.cs:55418`, missing in f29's arm; e17's had it) is CLOSED too: the
  union's `WorldMap::recentre` clears the pan FIRST (`:55418`), then takes
  the view-centre override + flag disarm (`:55419-55423`, the C# statement
  order); `prepare`'s reset arm calls it and `set_open`'s re-arm feeds it —
  every reset path clears the pan. The union's `prepare` signature carries
  the extra `map_enabled` param (the clearMap consume gate) — e17's pan
  block folds in as a hunk ahead of the clamps.
- **`set_open` / `WorldMap` fields:** both add `map_pylon_tile` (+ the pan
  state) and clear the pylon selection in the open arm
  (`Player.cs:29113`/`Main.cs:41102`); the union's `set_open` is the base's
  — a clean hunk merge. e17's `prepare` lacks the union's `map_enabled`
  param and clearMap consume; keep the union's body, fold only the pan
  block if e17 wins the keep-one.
- **The closeout input code composes cleanly:** neither sibling touches
  `ui/minimap.rs`, `world_key`, or `world_sim/player.rs`'s map tail — the
  new `apply_key_zoom`/`update_minimap_zoom_keys`/Tab arm have no overlap.
  f29's `update_map_input` pan-step hunk lands after the drag block, ahead
  of the clamps — the union's UsingMap-gated drag block is untouched.
- **Click cluster:** both wire the pylon click through the draw-side
  `map_layers_draw` return (f29 `apply_pylon_click` after the draw region;
  e17 `handle_pylon_map_click` + the `map_left_press` snapshot in
  `frame.rs`), NOT through `in_world_click_cluster` — the union's
  ping → close → bestiary order and the S1-B-8 pin are untouched.

## W4-e18 final sweep (2026-11-21, code-only — RE-FOLDED onto the union by the final-retire pass)

The e18d remaining-census close: the minimap key rows, the fullscreen open/close arms, the
`WorldMap.Load` catch arm, the modern engine's `mapDelay` skip, the fullscreen draw head's
dialog/inventory close and the draw tail's chat pass. Six mechanics, six commits on
`task/W4-e18-minimap-reveal-queue`
(`249a0c1b`/`4ad0f041`/`029154fe`/`2fa072d0`/`8ec77f0f`+`2e1f97c8`/`0d99d968`), plus the
research/84 cross-note addendum (`0bd346e7`) and the doc sweep (`0002fa1f`/`75289a2d`).
**All folded onto this union**: `249a0c1b`'s key/alpha arms by the dup fold (`a352b16e`,
research/84 §10.1 keep-one — the union's `update_minimap_zoom_keys` carries the g53
superset), the other five mechanics by the final-retire pass
(`736c86ce`/`5566bf25`/`cd24857e`/`7d050b7c`/`b767ef91`). The sections below describe the
mechanics under the UNION's names; where e18's own fn names differ the union's keep-one
ruling wins (research/84 §10.1).

### The minimap remaining census (the seeded four, resolved)

| Candidate | Verdict | Evidence |
| --- | --- | --- |
| minimap zoom +/- keys | LANDED (union closeout `08b749bd` + dup fold `a352b16e`) | `Player.cs:24913-24920` (`mapStyle == 1`, map shut): `MapZoomIn` → `mapMinimapScale *= 1.025`, `MapZoomOut` → `* 0.975` — the SAME multipliers as the chrome buttons (`MinimapFrame.cs:122`/`:135`), NOT the fullscreen keys' 1.05/0.95. Union: `ui/minimap.rs::apply_key_zoom` + `app/mod.rs::update_minimap_zoom_keys` (held keys, wired in `world_sim/player.rs`'s not-fullscreen else-arm). The alpha keys (`:24921-24928`, ±0.015) and the Tab style cycle (`:24949-24965`, press edge + `PlaySound(12)`) rode the dup fold + closeout. e18's `249a0c1b` variant (`update_minimap_keys`/`minimap_key_scale`/`map_style_prev`) DROPS per the keep-one. |
| map ping | VERIFIED LANDED (f29/S1-L-4) | `Main.cs:55486-55495`'s double-press detector is `clicks.rs::handle_fullmap_ping` (S1-L-4); the `PingMapLayer` pool (`PingMapLayer.cs:16-51`) is `map.rs::pings`/`add_ping`/`trigger_ping`/`age_pings` + `map_layers.rs:318-337`'s 1x5 10 Hz draw; the MP relay is `netglue.rs`'s `NetPingModule` (`the_map_ping_relays_through_the_ping_module`). The conversion `(MouseScreen - (num - 10*num5, num2 - 10*num5)) / num5` matches `map_origin`'s inset. |
| fullscreen pan inertia/friction | PHANTOM | `Main.cs:55342-55359` is the instantaneous per-frame cursor delta (`num20 *= 0.06255f` then `* (16f / mapFullscreenScale)`); `grabMapX`/`grabMapY` (`Main.cs:817-819`) are the ONLY drag statics — grep `mapFullscreenPos` writes finds no velocity/inertia/friction term anywhere. The only eased pan is `PanTargetMapFullscreen`'s `Vector2.Lerp(pos, end, 0.15)` (`:55361-55384`) — the pylon-click recentre, the overlay pick's step (f29 `map_pylons.rs::pan_target_step` / e17's `prepare` lerp; the union carries `pan_target` + the cancels). No inertia to port. |
| map icon LEGEND | PHANTOM (confirmed) | No legend draw exists in `DrawMap`/`MapRenderer.DrawMap` (the first-half census's grep `Legend` over the map block = 0 hits). `Lang.BuildMapAtlas` (`Lang.cs:554-572`) is the hover-NAME lookup table (`lang.rs::map_object_name`), not a drawn legend. |

### Other DrawMap arms — the unclaimed sweep

Every `Main.DrawMap` arm (`:55283-56650`) is now claimed: the first-half census resolved the
seed's six (bg/edges + frame landed; grid lines/legend/arrow-rotation/zoom-indicator phantoms)
and landed the close button + thick cursor; the overlay siblings' nine pieces cover the
heads/death/revenge/misc/hover/pylon arms (keep-one pending); the residual census covered the
statics/queues/drag gate. The final sweep found THREE more unclaimed arms outside `DrawMap`
proper:

1. **The fullscreen draw head's dialog/inventory close** (`Main.cs:62150-62153`):
   `talkNPC >= 0 || sign >= 0 || (playerInventory && !CaptureManager.Active)` → `ToggleInv`'s
   talk/inventory arms (`Player.cs:5005-5016`/`:5033-5056`), every drawn frame BEFORE `DrawMap`.
   Ported: `frame/draw.rs::close_dialog_behind_fullmap`, wired at `draw_region`'s head ahead of
   the pylon scan. The capture-active arm keeps the inventory; the sign editor has no port state.
2. **The fullscreen draw tail's chat pass** (`Main.cs:62160`): `DrawPlayerChat` runs after
   `DrawMap` — the chat log + input strip draw over the map (the HUD is suppressed, the chat is
   not). Ported: `ui::draw_chat` made pub + called at the fullscreen branch tail (after the
   union's head icon). `DrawFPS` (`:62159`) rides `showFrameRate` (F10, unported);
   `DrawPendingMouseText` (`:62161`) is the map's own `MouseText` (`:56046-56049`).
3. **The Unity-potion teleport block** (`Main.cs:55960-55975`, `:55998-56010`): the
   `HasUnityPotion` + same-team + `netMode == 1` hover → `PlaySound(12)` + `OurFavoriteColor`
   head ring + `Game.TeleportTo` text + click → `UnityTeleport` + `TakeUnityPotion`. **MP-client-
   only** (`netMode == 1`), unported — the netcode ladder (research row 17) owns it; the overlay
   pick's head-loop territory. Recorded as a seed, not ported here.

### The non-DrawMap residuals (ported by the sweep, re-folded onto the union)

- **`TryOpeningFullscreenMap`** (`Player.cs:29103-29119`): the M-key open arm closes the
  inventory (`:29107`), clears the talk-NPC latch + chat text (`:29109`), `PlaySound(10)`
  (`:29111`); scale 2.5 + recentre ride `set_open`. The close arms gain `PlaySound(11)`
  (`:24973` M-key, `:4976` Escape). The inventory map-icon row (`Main.cs:41095-41120`) rides
  the shared helper + the style icons' `PlaySound(12)`. `app/loop.rs::try_open_fullscreen_map`.
  The union's `set_open` → `recentre` also clears `pan_target` (`Main.cs:55418` via the reset
  arm); `MapPylonTile = (-1,-1)` (`:29113`) rides the overlay pick's field.
- **`WorldMap.Load`'s catch arm** (`WorldMap.cs:217-230`): a corrupt-but-present `.map` copies
  to `<path>.bad` (`:227`) + `Clear()` (`:229`); the absent file takes `TryGetMapPath`'s early
  return (`:184-188`) with neither side effect. `map.rs::load` splits the `load_map_file`
  `Ok(false)` fold on `path.exists()`.
- **The modern engine's `mapDelay` skip** (`LightingEngine.cs:82-89`): `WorldMap::map_delay`
  (ctor 2, `Main.cs:66504`; world-load re-arm `WorldGen.cs:6947`) gates `update`'s head before
  the `mapTime` countdown. The legacy engine keeps its own copy (S1-E-19 wave B). D22's
  `mapDelay` half closed.

### Deviations (recorded, deliberate)

- The union's minimap zoom/alpha fn covers BOTH styles (`Player.cs:24911` style-1 rows +
  `:24930-24947` style-2 rows, the g53 superset fold) — e18's own sweep deferred style 2 to
  D18; the union's dup fold closed that keys half. The style-2 DRAW stays D18. The style
  cycle fires in every non-fullscreen style (the shared else row).
- The fullscreen draw head's `ToggleInv` call reaches the talk/inventory arms, NOT the map-close
  first arm (`Player.cs:4972-4978`) — the map is already open, so the C# falls through; the port
  applies exactly the talk/inventory writes.
- The `num77` draw-scale formula (`Main.cs:55916-55919`) is DEAD CODE — `num77 = 1f` (`:55920`)
  then `num77 = Main.UIScale` (`:55921`) unconditionally overwrite it. The port's `draw_full`
  returns the map scale (the pre-existing S1-L-4 seam the overlay pick consumes); the stale
  citation repaired, the seam not re-tuned (the overlay pick's territory).

Witnesses (written, not run - code-only; the union's re-folded set): the dup fold's key/alpha
witnesses stand (`app::w4_f46_minimap_zoom_key_tests` incl. the alpha + style-2 pair,
`map::tests::w4_f46_alpha_and_overlay_clamps_match_the_draw_side`,
`app::loop::tests::w4_f46_*` — e18's 4 key witnesses + `app::w4_e18_minimap_key_tests` drop
with `update_minimap_keys` per research/84 §10.1), plus 1
`app::loop::tests::w4_e18_the_map_open_arm_closes_the_inventory_and_dialog`, 1
`app::clicks::w4_e18_map_icon_row_tests::w4_e18_the_map_icon_row_opens_the_map_and_sets_the_style`,
1 `map::tests::w4_e18_a_corrupt_map_is_copied_to_bad_and_cleared`, 1
`map::tests::w4_e18_the_map_delay_skips_the_first_two_exports`, 1
`app::frame::draw::w4_e18_fullmap_draw_head_tests::w4_e18_the_fullmap_draw_head_closes_the_dialog_and_inventory`.
SIX existing `map::tests` witnesses re-pinned with `map_delay = 0` (e18's five — mapTime/
reveal-window/mapEnabled/scene-box/updateMap — plus the union-only
`w4_f46_the_export_arms_update_map_even_on_an_empty_intersect`, which the e18 branch never had).
