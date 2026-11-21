# 72 — DOC-SWEEP lane 1: render/draw deviations (W4-g12, 2026-11-21)

Branch `task/W4-g12-dust-main-sites-2` (continues 665cd1e8). Code-only
mode: every witness WRITTEN NOT RUN, no cargo/rustc/tests/graphify.
Sources mined: `docs/ANALYSIS.md` (render/draw sections),
`docs/research/09-tile-drawing.md` (tables + deviations + seeds), the
`crates/tr-app/src/render/*` module headers and inline deferral notes
(the 125-hit `deferred|unported|stand-in|skipped|deviation|TODO|parked|
MISSING|fed 0|not ported|approximation` census), plus the named
examples in the task brief. Claim check: every pushed `origin/task/*`
branch tip read (`git log -1` + the relevant `git diff --stat` /
`git grep` on the touched files); "claimed" below means a pushed lane
owns the mechanic's code or its explicit wiring seam.

## Ported this sweep (7 commits, smallest first)

| # | Gap (doc source) | C# | Action / commit |
| --- | --- | --- | --- |
| 1 | Blood-moon night sky ramp + the 25 floor + the moon's red tint (`render/sky.rs:85` "lands with the blood-moon system (M4 events)" — stale, `World.blood_moon` live since the events port) | `Main.cs:63495-63519` (both arms share the red moon ramp; the `:63510-63512` forest write is dead), `:63503-63505`/`:63516-63518` (sky), `:63830-63844` (floor AFTER the phase floor) | PORTED + WIRED: `tr_core::sky::blood_moon_color`/`blood_moon_sky`, `color_of_the_skies(.., blood_moon)`, `DrawSunAndMoon` picks the red tint. Kept: the no-blood `Color.White` moon tail (`:63917-63920`) keeps the port's forest-tint stand-in; the `BloodMoonMonolith` half of the gate unported. 3 witnesses. `2e274271` |
| 2 | Shroom cap ("shroom cap MISSING", research/09 tree row) | `TileDrawing.cs:994-1006` | PORTED + WIRED: `tiles.rs::shroom_cap_frame` + the pre-sprite cap draw (the `frameX>=36` gate, the exact `frameY==18/==36` equalities, `(-22,-26)` anchor, 60x42 at `num4*62`, raw GetColor + case flip); `Shroom_Tops` packed (`AssetInitializer.cs:532`). 2 witnesses. `3fd8941d` |
| 3 | martianGlow event feed-0 (`tiles.rs:723` + `drawdata_cases_a.rs:53` "martianLight is 0 outside the (unported) event" — WRONG reading) | `Main.cs:18238-18262` (`DoUpdate_AnimateTileGlows` ticks ±0.015 with the 1/0 clamps EVERY frame, `:17145` unconditional; Initialize seeds 1/+1 `:66831-66832`), `TileDrawing.cs:498-499` | PORTED + WIRED (the feed): `App.martian_light/_dir` + `tick_martian_light` beside the demon-torch ramp, threaded through `draw_tiles` into `apply_cases_a` — the glow byte now spans 100..250. The glow RECORD stays unused until the TILE-DRAW-2 bind (f15's lane, keep-one at merge; G6 key untouched — no cached pixel reads it). g29's `martian_glow_byte` is the same formula on the unwired tile_pass — dedupe at the pass-family merge. 3 witnesses. `40d443d9` |
| 4 | `DrawGoreBehind` (`render/gore.rs` header "not ported — the back pass needs the tile-batch seam"; the seam exists) | `Main.cs:21842-21871`, capture site `:56934-56940` | PORTED + WIRED: `draws_in_back` (the front gate's exact drip complement) + `draw_behind` at the tile pass's behind-tiles slot (walls → WoF → back gore → behind-tiles NPCs → tiles); `plan_quad_inner` behind arm (framed −2 lift kept, NO 1217 nudge); the `drawBackGore` latch re-derived from the pool predicate; `WorldFrameKey::back_gore` (0 while no drip — the waterfall convention). Direct-screen z-order stays the fused-pass approximation. 4 witnesses. `cd8657fc` |
| 5 | DrawSavings coin pile (g58 P4 "rides the coin-slot lane — deferred here"; g57 "no port surface yet"; g58b dead, NO branch claimed the port — `git log --all -S 'fn draw_savings'` empty) | `ItemSlot.cs:3704-3741` + `DrawMoney :3652-3702` + `Utils.cs:1019-1076` + `CoinSlot.cs:225-254`/`:341` | PORTED + WIRED (the shop site `Main.cs:41791`): `ui/savings.rs` (the four bank totals, the 9999999999 mid-walk cap, the per-bank pile icons 4076/3813/346/87 at 0.65 truncating CenteredRectangle, both DrawMoney layouts with the 999 cap + UpdateSavings anims); `CoinSlotState.savings_entries[4]` + the Savings-first anim loop; `Coin_0..3` strips packed. The reforge (`:41637`) / buy-button (`:43862`) sites ride g57's / the NPC-chat lane's merges — both call shapes ported. Custom-currency route = d122 follow-up. 6 witnesses. `b467699f` |
| 6 | `DrawTileInWater` (`render/liquids.rs:197` "not ported: the generator never places tile 518" — stale: c33 ports PlaceLilyPad fd7fae0e8, loaded worlds carry pads) | `Main.cs:57536-57551` | PORTED, then WIRED by render-sweep-2: `draw_tile_in_water` (the active-518 gate, `liquid/16−3` lift, the `SolidTile(x,y−1) && num>8` ceiling clamp with the full nactive probe, own 16x16 frame at `(x*16, y*16−num)`). The sweep's "only caller is the retro pass" reading was WRONG — `LiquidRenderer.cs:664` (`DrawNormalLiquids`) + `:707` (`DrawShimmer`) call it per visible cell on the LIVE record path, and `TileDrawing.cs:872-875` skips wet pads in the tile pass. Both call sites + the tiles.rs skip wired (2+2 more witnesses). The retro sprite pass (`:57667`) stays g13's dust-feed scope; the fn is shared. 2 witnesses. `9e9877a5` |
| 7 | Wave-A slime scale rows (`entities.rs:3076` "the slimes' 1.25/1.1/1.05 are unported outside the base slime variants"; d34/d35/d37 add NEW kinds only — unclaimed) | `NPC.cs:8977`/`:9526`/`:9710`/`:9850`/`:10761`/`:11344`/`:11354` (147/225 write no row) | PORTED + WIRED: `wave_a_slime_scale` → the ctor chain → `scaled_anchor` (the BlazingWheel composition: the quad grows about the frame centre, the generic shift lands the C# bottom anchor); hitbox stays the unscaled box (C# scale is draw-only). 2 witnesses. `e128ef0d` |

## Claimed by active lanes — SKIPPED (do not double-port)

| Gap | Owner branch (pushed tip) | Evidence |
| --- | --- | --- |
| Retro-lighting draw path (`Lighting.NotRetro` → `oldDrawWater`, the legacy engine residuals) | `W4-d65-legacy-lighting-wof` (65cb42ac) | d65 ports `LegacyLighting.cs` residuals (the WoF gross block `:678-717` landed); the retro LIQUID sprite pass is the same engine's draw side — its dust feeds are g13's (below). This sweep's #6 stages the one tile redraw that pass needs. |
| Retro `oldDrawWater` dust sites (`:57766` sparkle, `:57789`/`:57793` lava bubbles) + the rect machinery | `W4-g13-dust-main-sites-3` (5ea44967b) | `main_dust.rs::retro_liquid_dust_feed` WIRED (35404f8a8) — the rect machinery `retro_surface_rect` is ported there; the sprite draw stays the record path (its own recorded deviation). |
| Pylon dust roll (`TileDrawing.cs:8669-8673` → `TeleportPylonsSystem.SpawnInWorldDust :327-393`) + void-lens dust (`VoidLensHelper.SpawnVoidLensDust :39-75`) | `W4-f6-dust-world-3` (world_dust.rs `pylon_spawn_dust`/`void_lens_dust`, STAGED) | f6's module doc names both as "the pylons.rs documented skips" — the emitters are ported there; the draw-side wiring (the `_rand.Next(4)`/`Next(10)` gates + the `Main.rand` lens rolls) is the dust-wiring wave's seam. `pylons.rs` untouched here. |
| Section-framing eager (`Main.sectionManager.SetAllFramedSectionsAsNeedingRefresh`, `Main.cs:56948`; the port frames eagerly) | `W4-g28..g32-tile-draw-pass-1..5` (g32 8b4fe36c records the section-framing deviation) | The Draw-orchestration family owns the pass order + the framing seam; g32's ANALYSIS section records the eager-framing deviation explicitly. |
| `_martianGlow` rebuild in the Draw head (`TileDrawing.cs:498-499`) on the NEW pass | `W4-g29-tile-draw-pass-2` (`tile_pass.rs::martian_glow_byte`) | Same formula as this sweep's #3 — the pass-family merge dedupes (keep-one); the LIVE tick + the old-pass feed are #3's. |
| Glow-record bind (the `_glow` drop at `tiles.rs:731`, TILE-DRAW-2) | `W4-f15-glow-bind-tiledraw2` (8ca19097) + `W4-g35-glow-records-wire` | f15 wires `apply_cases_a`'s glow return into the draw; g35 the records census. #3 feeds the live value through the SAME `apply_cases_a` seam — no conflict. |
| `ShouldTileShine`/`Main.shine` (research/09 deviation 10, `tiles.rs:1582` "unported (tileShine2)") | `W4-d56-tile-draw-animated` (5d80b38a) | d56 ports `ShouldTileShine :4404-4432` + `Main.shine Main.cs:21557-21660` + the tileShine2 census into `tr-render::animated_tiles` (UNWIRED, tiles.rs contested). |
| Quality-tier constants (research/09 deviation 11) | `W4-g28-tile-draw-pass-1` (528edaee) | `tr-render::quality_light` ports the `:479-495` thresholds incl. the net40 byte-cast wrap. |
| Reverse vines / rising strips / multi-tile vines / multi-tile grass (research/09 vine row "still MISSING") | `W4-g31-tile-draw-pass-4` (aa1c0ffd + 3af480de) | `tile_drains.rs` ports the deferred bucket drains incl. `drain_reverse_vines`/`rising_vine_strip`/`DrawMultiTileVines(InWind)` (UNWIRED). |
| Chest frame rewrite wiring + critter cages (research/09 deviation 5) | `W4-d58-tile-draw-chest-frames`, `W4-g33/g34-drawdata-wire-1/2` | The rewrite is ported (`drawdata_furniture.rs`); the wiring lanes own the tiles.rs call sites. |
| Tile-draw particles (`DrawTiles_EmitParticles :6688-7547`, research/09 "MISSING") | `W4-d60-tile-draw-particles` (4498be17 + 77429fe1) | `tile_draw_particles.rs` ports the whole battery (UNWIRED, two-stream RNG census). |
| Platform slope side-fills + martian conduit glow rows (research/09 deviation 6) | `W4-d59-tile-draw-platform-slopes` (f70498be, WIRED) + `W4-d61..d64-drawdata-cases` | d59 wired the side-fills; the glow rows ride the drawdata-cases lanes. |
| WindGrid gust push (research/09 deviation 8, `tiles.rs:872`/`:880`) | `W4-g68-weather-wind-leaves` + the M1.4-WEATHER-1 row | g68 owns the wind/leaf reconciliation; the grid push is the weather row's recorded out-scope ("the tile-draw seam"). |
| Cloud pool stand-in (`sky.rs:259` "Deterministic stand-in for the live 200-slot cloud array"), cloud parallax, `DrawCloud` byte chain | `W4-g69-weather-clouds-parallax` (e235d3e1 + 40352d4b) | g69 retires `make_clouds`/`CloudSprite`, wires the live pool + `UpdateCloudParallax` + the verbatim colour chain. |
| Snow/desert/ocean bg style tables (`bg.rs:1214`/`:1269` "still unported") | `W4-g71-bg-layers-2` (edc40654..1a4b420e) | g71 ports the setBG slots + `GetLayerYOffset` full switch + the desert cross-fade. |
| Forest/underworld bg layers | `W4-g70-bg-layers-1`, `W4-g72-bg-layers-3` | The four forest sets' sheets (g70) + `bg_styles.rs`/`underworld_bg.rs` (g72). |
| Wire-423 LightSeaGreen rect (`wires.rs:18` "content-gated — no tile 423") | `W4-g81-wire-draw-overlay` (0017d32a) | g81 ports the 423 collection + rect (`Main.cs:58660-58673`). |
| Echo-coating draw residuals (`pylons.rs`/`tiles.rs`/`vines.rs` "the port has neither yet") | `W4-g78-tile-echo-draw` (db4794b3) | g78 ports the sight chain + `IsVisible` keep-one + the wall/tile/vine consumers. |
| Void-lens projectile draw + `AddLight` (`pylons.rs:348` "the hook's job") | the g56 lane (273814c5, pushed inside the pylon-network branch chain) | The projectile-side `VoidLensHelper` draw + light feed are ported there; the TILE-side lens dust is f6's (above). |
| Tile paint recolor equivalence (research/09 deviation 9) | `W4-g79-tile-paint-draw` (0badeaeb) | g79 ports the WorldFile conversion + the pass ladder; the compiled-HLSL deviation stands (the bake is absent from the decompile). |
| Waterfall/lavafall/honeyfall/shimmer draw residuals (`tiles.rs:490` wall-over-fall approximation) | `W4-g73..g76` (the falls family) | The union contract (g73 9a15f3d7) + the per-kind draws; the z-order approximation is the family's recorded deviation. |
| Shimmer sparkle premultiply dimming (`liquids.rs:195`) | `W4-g76-shimmer-draw` (cfbb50e7) | g76's f64-phase split + the RAW-record opacity fix own that quad path. |

## Global gamepad — DOCUMENTED SKIP (no port model, per the brief)

The port has no `PlayerInput.UsingGamepad` (always false, keyboard-only
— `lang.rs:230-231`). Every gamepad-only draw mechanic is therefore
dead code in the port and stays unported, recorded here once instead
of per-site:

* **Gamepad link points** (`UILinkPointNavigator`, the
  `SetupGamepadPoints` families, ids 3000+): ANALYSIS `:14559`,
  `:14681-14682` (`UIEmotesMenu.SetupGamepadPoints2 :330-464`),
  `:14744`, `:6886-6887` (the virtual keyboard's
  `PrepareVirtualKeyboard :177-186` + the `UsingGamepad` hover gate).
* **The reforge button's `SetPosition`** (`Main.cs:41648`) — g57's
  recorded skip, same class.
* **`Inventory_Back17`** (`ItemSlot.cs:2331-2335`, the gamepad
  quick-craft highlight) — ANALYSIS `:11221`; the `Back14` white swap
  (`:17811`) is the same UI-point highlight.
* **The gamepad squiggle** (`GameTipsDisplay :122-127`) — ANALYSIS
  `:24859`; the achievement-icon `UsingGamepad` hover (`Main.cs:42680`/
  `:42725`, the `:42703`/`:42781` link-graph rows) — ANALYSIS
  `:17295-17299`.
* **The keyboard's 50-key grid layer** stays behind `TR_VK_GRID=1`
  (ANALYSIS `:3058-3059`) — the desktop layer is the live one.
* **Smart-cursor gamepad arms** (`smart_cursor.rs:198-211` already
  carries the caller-passed `using_gamepad`/`gamepad_idle` gates — the
  INPUT side is modelled; only the link-point NAVIGATION is out).

No C# port action: these fire only under a gamepad profile the port
does not model. If a gamepad lane ever lands, this table is its
checklist.

## No C# source — DOCUMENTED (cannot port verbatim)

* **HorizonClouds shader** (`NextHorizonRenderer.CloudsEnd :323-353`,
  `GameShaders.Misc["HorizonClouds"]`, registered
  `DyeInitializer.cs:438`): the C# side is fully readable — the
  celestial-body colour (`HorizonHelper.GetCelestialBodyColors`), the
  `max(sunset, sunrise) * celestialVisibility` factor (the night floor
  `celestialVisibility * 0.15`), the `(x, y, z, 0)` shader-specific
  vector with `z = num * clamp(1 - cloudBGAlpha)` — but the pixel
  shader itself is COMPILED HLSL inside the retail `.xnb`
  (`pixelShaderRef`), absent from the decompile. Same class as the
  paint bake (research/09 deviation 9) and LitNature (the vine strip's
  dawn/dusk shader, `vines.rs:15-16`). The C# uniform feed CAN be
  ported when a WGSL approximation is written; until then the clouds
  draw plain (g69's live pool feeds the same `DrawData` list the
  shader would consume — the seam is ready). ANALYSIS `:926`, `:1498`.
* **LitNature shader** (the vine/nature dawn/dusk tint,
  `TileDrawing.cs:9123-9124` consumes it): same compiled-HLSL class;
  the segment colour output is ported (`vines.rs:99` "consumed only
  by the unported LitNature shader").

## Remaining unclaimed gaps (too large for this sweep / need a seam decision)

| Gap | Doc source | Why deferred |
| --- | --- | --- |
| The deferred `_specialTilesX/Y` registry + the two-layer split (research/09 deviations 1/2, seed L2) | `tiles.rs:671-678`, `pylons.rs:25` | The g28-g32 pass family IS this port (UNWIRED on their branches); a parallel registry here would collide wholesale. |
| `GetTileOutlineInfo` danger outline (`TileDrawing.cs:4478`, research/09 "MISSING (debug feature)") | research/09 special-tile table | Needs the smart-interact scan (`Player.SmartInteractLookup :17372-17395` → `SmartInteractSystem.RunQuery`) — the port's smart cursor has no interact-coords lists (`Main.SmartInteractTileCoords[Selected]`, `Main.cs:2035-2037`); a system-sized port, not a draw residual. `InSmartCursorHighlightArea` (`Main.cs:3945-3957`) reads those lists — the pylon/lens `selection_mode = 0` stand-in stays until it lands. |
| The `oldDrawWater` sprite pass itself (the retro liquid draw) | `Main.cs:57553-57940` | g13 ported its dust feeds + rect machinery; the sprite half is a full liquid-draw alternative path (~400 lines) — the retro-liquid follow-up's scope. Its one tile redraw (#6) is ported AND wired into the live record path (render-sweep-2); the retro pass shares the fn. |
| Tile-entity furniture sprites (395/698/520/... research/09 "MISSING") | research/09 special-tile table | d57 (`W4-d57-tile-draw-te-furniture`) owns the TE preview machinery; the item-sprite overlays ride its merge. |
| The Moon Lord / pumpkin / frost boss extra parts (TreeFace/PumpkingFace/SantaTank/IceQueen trails, `entities.rs:3057`/`:1309`) | ANALYSIS M4-NPC-3t/3v | The parts' positions ride the unported `localAI` ellipse machines (3v's recorded parking) — an AI-lane dependency, not a pure draw port. |
| `Main.ambientLava` / `player.lavaOpacity` / storm+rare cloud branches | ANALYSIS `:926-928`, `:1494-1497` | The lava ambience is an audio feed (g73's `Ambience` lane); `lavaOpacity` needs the lava-immunity accessory tick; the storm/rare cloud types need the boss-flag rolls — g69's cloud lane records them as its own out-scope. |

## Verification

Code-only per the task order: no cargo/rustc/rustfmt/tests/dotnet/
graphify invoked. Every port re-read against the cited C# windows
(the ≤100-line reads); every claim checked against the pushed branch
tips (`git log -1` + targeted `git diff`/`git grep`, never a junction
recursion). 22 witnesses WRITTEN NOT RUN across the 7 commits (3+2+3+
4+6+2+2). The G6 `WorldFrameKey` gained ONE field (`back_gore`, #4) —
zero while no drip draws, so gore-free frames keep the pre-sweep cache
behaviour bit-for-bit; the martianLight feed (#3) changes no cached
pixel (the glow record is still dropped pre-bind). Render-sweep-2
(2026-11-21, code-only) corrected #6's staging: the pad redraw is WIRED
into both live record paths + the tiles.rs wet-pad skip, 2 more
witnesses WRITTEN NOT RUN (24 total); no new G6 key field (the pad's
inputs ride world_revision/light_generation/liquid_anim).