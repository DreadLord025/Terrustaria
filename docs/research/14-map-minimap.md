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
| `WorldMap.Load` / `Save` / `TryGetMapPath` | PORTED | `map.rs::load_map_file` (622), `save_map_file` (597), `persist.rs:39` | `<world>.map` sidecar next to the `.trwld`; loaded at world start, written by `saveglue.rs:202-214`. |
| `MapUpdateQueue` (area + tile queues) | MISSING | — | `MapUpdateQueue.cs:21-108`; no `WorldMap.UpdateType` refresh after worldgen/net edits (`WorldGen.cs:80347/80402/81334`, `NetMessage.cs:2456`, `MessageBuffer.cs:1153`). |
| `WorldMap.UnlockMapSection` / `UnlockMapTilePretty` | MISSING | — | `WorldMap.cs:109-174`; the `DebugOptions.unlockMap` debug reveal is absent. |
| `WorldMap.ClearEdges` | MISSING | — | `WorldMap.cs:260`; no blacking of the outer 40 tiles (`BlackEdgeWidth = 40`, `WorldMap.cs:16`). |
| `MapRenderer.QueueChange` / refresh grid | MISSING | — | `MapRenderer.cs:288-304`; the port re-renders from `MapTile` each frame instead of caching into render targets — equivalent output, different cost. |
| `MapRenderer.CheckMapTargets` | PORTED (semantics) | `map.rs::prepare` (480, `uploaded`/`needs_full_upload`) | One texture per world instead of the target grid (`MapRenderer.cs:357-378`). |
| `Main.DrawMap` fullscreen geometry | PORTED | `map.rs::map_scale_clamp` (20), `map_wheel_scale` (28), `map_key_scale` (33), `map_clamp_pos` (38), `map_drag_delta` (49), `map_full_quad` (88), `map_frame_rect` (107) | `Main.cs:55306-55431`, `55342-55359`, `55388-55412`. |
| `Main.DrawMapFullscreenBackground` (`MapBG`) | PARTIAL | `map.rs::map_bg_index` (231) | Ported-later branch order, but the unevaluable zones take the `false` path: `ZoneDungeon`, `ZoneGlowshroom`, `ZoneCorrupt`, `ZoneCrimson`, `ZoneHallow`, `ZoneJungle`, `bloodMoon` (no doubling), `ZoneGraveyard` (`Main.cs:56342-56444`); ANALYSIS.md:1578-1582 admits it. |
| Minimap draw + panel geometry | PORTED | `map.rs::map_minimap_quad` (166) | 240×240 panel, `240/scale` window, 10/10/`maxTilesX-10`/`maxTilesY-19` clamps, `MapScale` batch transform (`Main.cs:55293-55296`, `:55503-55545`, `:55309-55329`). |
| Minimap alpha (`mapMinimapAlpha`) | MISSING | — | `Main.cs:55523-55532`; the panel is always opaque (no fade). |
| Minimap frame chrome + zoom buttons | PORTED | `ui/minimap.rs` (whole file) | Nine styles, hitboxes, hover sheets, `DEFAULT_ZOOM 1.05`, ×1.025/×0.975, clamp 0.2..3 (`MinimapFrame.cs:59-63`, `:107-136`, `:138-208`). |
| Frame-style selection UI | PORTED | `menu/pages.rs:268-284`, `menu/actions.rs:332-333`, `settings.rs:113-115` | `MinimapFrame` config key + `CycleSelection` order. |
| Map style switcher (`mapStyle` 0/1/2) | PARTIAL | `map_style == 1` only | `mapStyle == 2`'s screen-overlay map (`Main.cs:55548-55578`, `mapOverlayScale`) and `mapStyle == 0`'s retro HUD are absent. |
| Player head icon | PORTED | `map.rs::head_icon_pos` (76) + `render/entities.rs::draw_player_map_icon` | Position ported (`Main.cs:55937-55954`); the 84×84 `MapHeadRenderer` render target is replaced by a hand-drawn head (ANALYSIS.md:1584-1593). |
| Other players' head icons | PARTIAL | same draw path | `Main.cs:55931-55954` loops all 255 players; single-player only here. |
| NPC / chest / pylon / ping / boss-bag icons | MISSING | — | `Main.DrawNPCMapIcons` (`Main.cs:55926`), `DrawMap_FindChestName` (`:56631`), all eight `Map/` overlay layer files (no Rust counterpart — grep for `PingMapLayer|MapIconOverlay|TeleportPylons|BossBagMapLayer` in `crates/` returns nothing). |
| `Lang.BuildMapAtlas` / map text | MISSING | — | Pointer/active-player name labels under map icons. |
| Fullscreen map close/`M` toggle, `Main.mapEnabled` gate | PORTED | `map.rs::set_open` (466), `frame.rs:1209`, `clicks.rs` | `Main.cs:41101-41118`, `Player.cs:29105`. |

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
- **D14 — no `MapUpdateQueue`.** behavioural · *not admitted* (no Rust counterpart at all; grep for
  `MapUpdateQueue` in `crates/` is empty). Tile edits made outside the player's reveal window
  (worldgen after load, net-received tile changes `MessageBuffer.cs:1153`,
  `NetMessage.cs:2456`, `WorldGen.cs:80347/80402/81334`) never refresh their map pixel until the
  reveal window revisits them with ≥ light. Since `update_map_lighting` keeps the *brighter* light
  and only rebuilds when `MapTile.Equals` fails, a tile whose colour changed while its light is
  already high can also stay stale until the changed bit forces a rebuild.
- **D15 — no `UnlockMapSection` / `UnlockMapTilePretty` / `DebugOptions.unlockMap`.** behavioural ·
  *not admitted*. The debug "unlock map" reveal (`Main.cs:61873-61882`) and its near/full-bright
  fade (`WorldMap.cs:140-174`) are absent.
- **D16 — `MapBG` unevaluable zones.** cosmetic · *admitted* (ANALYSIS.md:1578-1582). `ZoneDungeon`,
  `ZoneGlowshroom`, `ZoneCorrupt`, `ZoneCrimson`, `ZoneHallow`, `ZoneJungle`, `bloodMoon` and
  `ZoneGraveyard` all take the `false` path, so the fullscreen backdrop shows the wrong sheet in
  those biomes until their zone evaluation is wired.
- **D17 — minimap has no alpha config.** cosmetic · *not admitted*. `Main.mapMinimapAlpha`
  (`Main.cs:55523-55532`) has no config key or clamp; the panel always draws at full alpha.
- **D18 — `mapStyle` switcher absent.** behavioural · *not admitted*. Only `mapStyle == 1` is
  modelled; the `mapStyle == 2` screen-overlay map and its `mapOverlayScale`
  (`Main.cs:55548-55578`) and `mapStyle == 0`'s retro layout have no counterpart.
- **D19 — map icon overlays entirely missing.** behavioural · *not admitted*. Pings
  (`PingMapLayer.cs`), team spawn (`TeamBasedSpawnMapLayer.cs`), pylon links
  (`TeleportPylonsMapLayer.cs`), boss bags (`BossBagMapLayer.cs`), the Digtoise marker
  (`DigtoiseMapLayer.cs`), `MapIconOverlay`'s NPC/player/chest text (`Main.cs:55782-55926`,
  `:56631`) and the golf/potion-of-return icons (`Main.cs:56468-56515`) have no Rust code —
  multiplayer map callouts and single-player NPC/chest labels simply do not exist.
- **D20 — head icon is a hand-drawn stand-in.** pixel · *admitted* (ANALYSIS.md:1584-1593).
  Vanilla renders an 84×84 `MapHeadRenderer` render target; the port draws the head, eyes, whites
  and hair plus the outline rings directly, so a head will not match vanilla pixel-for-pixel, and
  only the local player's head is drawn (no 255-player loop).
- **D21 — four `GetTileBaseOption` cases substituted.** cosmetic · *admitted* (option.rs:102-108).
  `461` (Corrupt/Crimson/Hallow tiles take a zone-dependent option) and `80`/`529`/`530`
  (cactus/biome-influence tiles) always resolve to option 0, so those tiles show their base colour
  in every biome.
- **D22 — `Main.mapDelay` and `Main.refreshMap`/`Main.clearMap` not modelled.** behavioural ·
  *not admitted*. The first-frames skip (`LightingEngine.cs:82-89`) and the
  content-lost-triggered full refresh (`MapRenderer.cs:363-377`) are replaced by the port's
  `uploaded`/`needs_full_upload` pair, which is equivalent for a correct upload but never re-runs a
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

Risk summary: the highest-severity gaps are the map **invalidation** story (D14) — tile changes
outside the reveal window are silently lost to the map — and the missing icon layers (D19), both
behavioural and neither currently recorded in the roadmap or ANALYSIS.

## Task seeds

- **S — Port `MapUpdateQueue` + `WorldMap.UpdateType`.** The queue
  (`MapUpdateQueue.cs:21-108`), `WorldMap.UpdateType` (`:65-93`) and the call sites
  (`WorldGen.cs:81334`, `MessageBuffer.cs:1153`, `NetMessage.cs:2456`,
  `WorldGen.cs:80347/80402`) are small and self-contained; expose them as
  `tr-core/src/map/mod.rs::{queue_add, queue_update}` fed from `tr-app`'s edit/net glue. Blocks
  nothing; fixes the stale-pixel class of D14.
- **S — Wire `ClearEdges`** (`WorldMap.cs:260`, `BlackEdgeWidth = 40`): one loop over the outer 40
  tiles, run after load and after worldgen. Depends on nothing; fixes D4.
- **S — Minimap alpha config** (`Main.cs:55523-55532`): a `MapMinimapAlpha` key in `settings.rs`,
  clamped 0.01..1, threaded through `frame.rs:1248`/`map.rs`. Fixes D17.
- **M — `MapBG` zone branches** (`Main.cs:56342-56444`): port `ZoneDungeon`, `ZoneGlowshroom`,
  `ZoneCorrupt`/`ZoneCrimson`/`ZoneHallow`/`ZoneJungle` and the `bloodMoon` doubling off
  `SceneMetrics`, plus the `ZoneGraveyard` wall test. Depends on the weather/SceneMetrics module
  (research row 16) for the shared zone evaluation. Fixes D16.
- **M — The eight map overlay layers** (`Map/*Layer*.cs`, `MapIconOverlay.cs`,
  `MapOverlayDrawContext.cs`, `Main.DrawNPCMapIcons`, `Lang.BuildMapAtlas`): pings, team spawn,
  pylon links, boss bags, Digtoise, NPC/chest labels. Needs the `IMapLayer` seam, the ping network
  message and the icon texture whitelist in `assets.rs`. Depends on the netcode research row 17.
  Fixes D19.
- **M — Fullscreen map input parity**: `mapStyle` cycling (0/1/2, `Main.cs:41101-41118`), the
  `mapOverlayScale` screen-overlay map (`:55548-55578`), minimap zoom key/wheel wiring, and
  `DebugOptions.unlockMap` (`Main.cs:61873-61882`, `WorldMap.cs:109-174`). Depends on the UI
  framework research row 18 for the in-game options rows.
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
