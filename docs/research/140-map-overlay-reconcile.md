# 140 — Map-overlay 2-way reconcile (W4-f29 × W4-e17)

**Phase:** reconcile, docs-only. No code changes to e17. This document is the
adjudication record + fold-list + merge order for the two independent ports of
the `Main.DrawMap` overlay second half.

**Branches compared (both off main `a725acb04`, the wave-3-complete base):**

| Lane | Tip | Shape |
| --- | --- | --- |
| `task/W4-e17-map-overlay-layers` | `8500c3e5d` (fetched; **advanced past the first half** — the player-pin/death/misc/revenge lane `adb5ccd98`/`ce6994493`/`26f638264` landed on it too) | `map_npc_icons.rs` (950 ln), `map_player_icons.rs` (708 ln), `map_layers.rs` +582, `map.rs` +277, `clicks.rs` +139, c21 cherry-pick `98e2ae016` (d7f7e9cc map-object slice) |
| `task/W4-f29-map-overlay-2` (this branch) | `9f80c5375` | `map_overlay.rs` (1435 ln), `map_pylons.rs` (473 ln), `map_layers.rs` +224, `map.rs` +24, `app/mod.rs` +150 |

**Headline ruling: e17 is the surviving skeleton.** Its latest tip covers ALL
nine f29 census items (not just NPC-heads/pylon-lines/c21), its draw wiring
follows the C# call order in both branches, and its NPC-head table is keyed
correctly. f29 contributes **six genuine correctness fixes** (§3) plus a
witness set that unions to ~34 (§5). The brief's "YOUR-uniques fold-list
(death/revenge/golf/player-heads/hover-names/LocalizedDuration)" is **obsolete
against the latest e17 tip** — e17 ported all of those independently; what
survives as f29-unique is the fix list below, not whole subsystems.

---

## 1. Per-fn adjudication — `DrawNPCMapIcons`/`2`/`3` (C# `Main.cs:56099-56292`)

Arm-by-arm against the C# (read in full: `:56099-56142` overlay,
`:56144-56220` minimap, `:56222-56292` fullscreen, `:56294-56297`
`CanBeSeen_Townie` = flat `true`).

| Arm (C# cite) | e17 (`map_npc_icons.rs`) | f29 (`map_overlay.rs`) | Verdict |
| --- | --- | --- | --- |
| Boss-head table key (`NPCID.cs:10479`, `NPC.cs:7559` reads `this.type`) | `boss_head_type(NpcKind) -> vanilla type` match; **246/249 Golem heads keep their own types** | `n.kind.net_id()` — the bestiary fold maps `GolemHead`/`GolemHeadFree` → `Golem` → **245**, which has NO table row | **KEEP e17.** f29's Golem head icons never draw (245 ∉ table); the C# keys on `type`, not the folded net id. e17's comment (`map_npc_icons.rs:24-26`) documents exactly this trap. f29's own witness only asserts table-row membership, never the kind→head resolution, so it never caught it. |
| 7 type overrides (`NPC.cs:7560-7604`: Eye ai0≥2→1, Brain hide, Plantera int lifeMax/2→12, twins ai0≥3→16/21, Cultist 439/440 pair) | all seven, type-keyed | all seven, net-id-keyed (same values for the non-folded kinds) | **KEEP e17** (rides the correct key). Behaviour identical for every ported kind except the Golem family. |
| `GetBossHeadRotation` (`:7608-7632`: 68/262 always, 35/127 ai1∈{1,2}, 345 ai0==2) | type-keyed match | net-id-keyed match | **KEEP e17**; equivalent for ported kinds. |
| `GetBossHeadSpriteEffects` (`:7634-7642`: 491 spriteDirection==1 flip) | `boss_head_flip` | `boss_head_flip` | Equivalent; keep e17's. |
| Town gate `active && townNPC && CanBeSeen_Townie` (`:56103`) | `hp>0 && is_town_npc_like()` | `hp>0 && is_town_resident()` | **KEEP e17.** Both sets equal the 21 ported `townNPC=true` kinds today (`npc/data.rs:929/:1144`); `is_town_npc_like` is the documented `townNPC`-flag twin and stays correct if Old Man/Skeleton Merchant kinds land. |
| `TownNPCProfiles.GetHeadIndexSafe` (`TownNPCProfiles.cs:200-207` → profile dict, else `TypeToDefaultHeadIndex`) | **full port**: 28 stacked shimmer rows (normal, shimmered) incl. 37/453→(-1,-1), pet `VariantNPCProfile` arrays (Cat 27.., Dog 33.., Bunny 39.., clamp 0..5), 8 town-slime fixed heads, default-table fallback; rides `Npc.variation_index` | default table only (`type_to_head_index`); doc claims "townNpcVariationIndex unported" | **KEEP e17.** f29's claim is stale: `variation_index` EXISTS in the base (`npc.rs:1313`, save/load round-trips it). The stacked-profile selection is `Profiles.cs:44-52` (index 1 = shimmer, out-of-range → 0) — e17 ports it faithfully; shimmered townies show the wrong head under f29. |
| Town flip `direction > 0` (`:56109-56112`) | `n.facing > 0` | `n.facing > 0` | Equivalent. |
| Anchor `Center + (0,gfxOffY) + netOffset` (`:56113/:56124/:56236`) | `head_anchor` | inline, same math | Equivalent. |
| Destroyer 134 segment-average (`:56125-56137`: walk `Main.npc[(int)ai[0]]` while active && 134..136, ≤15, divide) | `destroyer_head_center` walks **`worm_id` + `seg_index`** chain order, stops at first dead/missing link | `destroyer_average_center` walks **`ai[0]` as child_index+1** | **KEEP e17.** The base never builds a real `ai[0]` chain: `npc_ai_style37.rs:1117` sets `head.ai[0] = head+1` ("any non-zero value closes the first-tick gate") and the links carry no child pointer. f29's walk therefore re-reads the HEAD itself 14 times (self-loop) — the average collapses to the head centre. e17's stable-identity walk matches the port's compaction model (`world_sim/npcs.rs` death-retain) and the C#'s active/type gate. |
| Fullscreen geometry `v/16*scale + X − 10*scale` (`:56237-56242`) | `draw_npc_map_icons_fullscreen` | `head_pos(Fullscreen)` | Equivalent maths; keep e17's three-fn shape (1:1 with the three C# fns). |
| Minimap geometry `(v/16−left)*scale + X2 + xOff`, y `−2*scale/5` (`:56159-56164`) | `draw_npc_map_icons_minimap` | `head_pos(Minimap)` | Equivalent. |
| Minimap clip `(miniMapX+12, +10, W−16, H−14)` strict (`:56165/:56206`) | `in_clip` strict | `head_in_clip` strict | Equivalent; both have witnesses. |
| Hover rect `(float)(Width()/2)` INTEGER half (`:56167/:56243`) | `hover_half` `(w/2)` u32 | `head_hover` `(size as i32/2)` | Equivalent (both truncate). |
| Hover write order: fullscreen town hover BEFORE draw (`:56247-51`), boss draw BEFORE hover (`:56281-89`); minimap both BEFORE (`:56171-75/:56212-16`); overlay none | exact per-mode order | exact per-mode order | Equivalent. |
| Hover text `FullName` / `GivenOrTypeName` (`NPC.cs:6729-6753`, `Game.NPCTitle`) | `full_name`/`given_or_type_name` | `npc_full_name`/`npc_given_or_type_name` | Equivalent; keep e17's. |
| Outlined rings (`AnOutlinedDrawRenderTargetContent.cs:38-62`: 8 black Manhattan-4 + 4 border-2 + body, whole target rotated/flipped about centre, single alpha multiply `Main.cs:56555/:56564`) | `draw_head_outlined` + `push_centered` | `draw_head_outlined` | Equivalent composites (same offsets, same pivot math, border = White×tint). Keep e17's. |
| Notification icon (`Main.cs:56556-59` → `Utils.DrawNotificationIcon` `Utils.cs:442-461`) | **ported** (`draw_notification_icon`: sway `sin(t%2/2·2π)·2π·0.035`, alpha lerp 1→127/255, origin (w/2, h−4)), gated `show_indicator=false` (dialogue queue unported) | sheet packed, draw gated off entirely | **KEEP e17** — the math is landed and only the gate is off; f29 would need it re-written when the dialogue system lands. |
| Overlay pass `DrawNPCMapIcons3` (`:56099-56142`) | `draw_npc_map_icons_overlay`, `#[allow(dead_code)]` UNWIRED | `HeadMode::Overlay`, UNWIRED | Both unwired (the port's `mapStyle==2` branch draws nothing — P4 Uncertainty 7). Keep e17's; the wiring rides the mapStyle-2 task. |

## 2. Per-fn adjudication — pylon lines / `DrawClamped` / click state

C# read in full: `TeleportPylonsMapLayer.cs:16-119`,
`MapOverlayDrawContext.cs:34-243`, `Main.cs:55361-55385`,
`Player.cs:32180-32212` + `:29103-29118`.

| Arm (C# cite) | e17 | f29 | Verdict |
| --- | --- | --- | --- |
| `DrawLine` (`TeleportPylonsMapLayer.cs:18-28`): `Min(5,(int)dist)` BlackTile segments, `Lerp(start,end,i/n)`, rotation `(end−start).ToRotation()`, origin `(0, Width*0.5)`, scale `(dist/n/16, width/16)` | `pylon_line` + pure `pylon_line_segments` | `map_pylons::draw_line` | **KEEP e17** (same maths incl. the origin-pivot shift; the pure segment fn + witness is the cleaner seam). Both read the real Black_Tile dims. |
| Line endpoints = `GetUnclampedDrawRegion(...).Center` (`MapOverlayDrawContext.cs:57-65`): the rect ctor **truncates top-left AND size to int**, `Center` = integer halves | `unclamped_center` = `pos − origin·num + size·num·0.5` — **FLOAT, no truncation** | `pylon_line_endpoint` — `(int)` top-left, `(int)` size, `x + w/2` integer halves | **FOLD f29 (fix F1).** The C# truncates before Centre; e17's float centre drifts up to 1 px per endpoint at fractional scales. Keep e17's method site, replace its body with f29's integer semantics; fold f29's witness. |
| Line block (`:45-64`): fullscreen + `MapPylonTile != (-1,-1)`, black 6 px + white 2 px to every OTHER revealed pylon | inside `map_layers_draw` (the layer's own Draw) | `draw_pylon_lines` called before `map_layers_draw` | **KEEP e17** — the lines live inside the layer in C#; same visual order either way, but e17's placement is the 1:1 structure. |
| `SpriteFrame(11,1){PaddingY=0}` cell (`SpriteFrame.cs:72-77`: width/11 − PaddingX(2), height whole) | **28 px cell** (`width/11 − 2`), column step `width/11` | 30 px cell (`width/11`, no padding) | **KEEP e17.** f29's markers are 2 px too wide and the UV columns bleed into the neighbour cell. e17 has a witness pinning 28. |
| `DrawClamped` rect pull-back (`MapOverlayDrawContext.cs:132-184`): X pass then Y pass on the ALREADY-shifted rect, integer `(int)(over/d·other)` truncations, border 10 | pure `clamp_draw_region` + `draw_clamped` | `clamped_draw_region` (pure) + inline clamp in `draw_pylon_clamped` | **KEEP e17's structure** (method on the context, pure helper). Both clamp maths identical. |
| `DrawClamped` hover (`:185`): `rectangle.Contains(Main.MouseScreen.ToPoint())` — **half-open integer** `[x, x+w)` | `mouse >= rx && mouse <= rx+rw` float **inclusive** | `(mouse as i32) >= rx && < rx+rw` **half-open int** | **FOLD f29 (fix F2).** e17's hover claims the far edge row/column the C# excludes. Replace e17's hover test with f29's; keep everything else. |
| Offscreen arrow (`:199-213`): Extra_299 `Frame(3,3,2,1)`, dir = normalize(true−clamped), reach `(sourceHeight/4)` INTEGER × num2, origin `(0, h/2)`, white×opacity, only when offscreen AND unhovered | ported, integer reach | ported, integer reach | Equivalent; keep e17's. |
| Scale ladder (`:186-198`): offscreen 0.5× (×2 when hovered), onscreen hover 2× | `draw_clamped` + `draw_scaled` (minimap) | `draw_pylon_clamped` + `draw_tinted` (minimap, **no 2× hover scale**) | **KEEP e17.** f29's minimap markers never grow on hover (`draw_tinted` lacks the `scaleIfSelected` arm, `:236-239`); e17 retired `draw_tinted` for the faithful `draw_scaled`. |
| Marker click (`:83-109`): hover + `mouseLeft && mouseLeftRelease` latches LAST hovered; onscreen → close map + sound 11 + `RequestTeleportation`; offscreen → pan; both consume the press | `PylonClick::Teleport/PanTo(PylonInfo)` → `clicks.rs::handle_pylon_map_click` + `request_pylon_teleport` (gate ladder, style-9 Teleport, mount boost, refusal chat) | `PylonClick::Teleport/Pan([f32;2])` → `app/mod.rs::apply_pylon_click` + near-identical `request_pylon_teleport` | **KEEP e17.** Same ladder (both over `tr_core::pylons::handle_teleport_request`); e17's home is `app/clicks.rs` (house architecture: clicks live there), and `PanTo(PylonInfo)` keeps the pylon identity. f29's `mount.height_boost()` accessor is redundant — base `MountState::data()` already gates on `active` (`mount.rs:2465`), so e17's inline read is correct. |
| Press edge for the DRAW-side click (`:83` reads live `Main.mouseLeftRelease`, which the port's click cluster consumes before the draw) | **`map_left_press` snapshot** in `frame_shared_setup` BEFORE the cluster (+ flow.rs test-ctor row) | reads `self.ui_left_press` at the draw (already consumed by the cluster on some paths) | **KEEP e17** — the snapshot is the faithful seam; f29's click can silently miss. |
| `cancelWormHole` (`:81` set on hover; `Main.cs:56006` cleared at the fullscreen tail) | ported (`WorldMap.cancel_wormhole`, set in draw, cleared post-gpu) | absent | **KEEP e17.** |
| Pan step (`Main.cs:55361-55385`): press cancels; dist<1 snaps; lerp 0.15, `1/dist` when `dist·0.15<1`; runs BEFORE the scale/pos clamps; `resetMapFull` also clears the pan (`:55418`) | inline in `WorldMap::prepare` + `pan_cancel_press` snapshot; reset arm clears pan | pure `pan_target_step` in `update_map_input`; reset arm does NOT clear pan | **KEEP e17's wiring** (prepare = the C# site, reset-clear faithful). **FOLD f29's pure `pan_target_step` + witness (F6)** as the helper e17's inline block delegates to — e17 has no pan witness today. |
| 597 right-click (`Player.cs:32180-32212`): `TryOpeningFullscreenMap` (inventory/sign/talk/corner-item clears + sound 10 + scale 2.5 + pylon-tile clear + reset arm, `:29103-29118`), anchor `(x−frameX/18%3, y−frameY/18%4)`, reveal-inclusive rect Inflate 20, integer Centre, scale fit `(screen−40)/size` shrink-only, pan+reset cleared | `doors.rs` inline arm: **full TryOpening clears** (inventory/talk/npcChat/shop + Menu_Open) + `map.rs::pylon_view_fit` | `doors.rs::use_pylon`: inventory clear only (no sound, no talk/shop clears) + `map_pylons::pylon_right_click_view` | **KEEP e17.** Same rect/fit maths (both integer-centre, shrink-only); e17's open-side effects are the faithful `TryOpeningFullscreenMap`. f29's `clear_reset_pos()` accessor retires with the module (e17 writes `reset_pos = false` directly). |
| `set_open(true)` clears `MapPylonTile` (`Player.cs:29113`, `Main.cs:41102`) | yes (`pylon_tile`) | yes (`map_pylon_tile`) | Equivalent; keep e17's field name. |

## 3. f29-unique fold-list (the fixes that survive onto e17)

The brief's subsystem-level fold-list is obsolete (e17's tip ported
death/revenge/golf/player-heads/hover-names independently). What remains are
six targeted fixes, each with a C# cite and a witness:

| # | Fix | C# evidence | e17 site to patch | Witness to fold |
| --- | --- | --- | --- | --- |
| **F1** | Pylon line endpoint: truncate top-left + size to int BEFORE the integer-halves Centre | `MapOverlayDrawContext.cs:64` (`new Rectangle((int)…)`), `:47-59` consumes `.Center` | `map_layers.rs::MapOverlayContext::unclamped_center` (float → int semantics, f29 `map_pylons.rs:82-108` body) | `w4_f29_pylon_line_endpoint_is_the_rect_centre` |
| **F2** | `DrawClamped` hover: half-open integer `Contains`, not float-inclusive | `MapOverlayDrawContext.cs:185` + XNA `Rectangle.Contains(Point)` | `map_layers.rs::draw_clamped` hover test (f29 `map_layers.rs:491-493` body) | extend `w4_e17_the_clamped_region_pulls_back_to_the_border` with the far-edge exclusion |
| **F3** | Misc portal/golf position: `Utils.Floor` is `(int)` TRUNCATION toward zero, not `Math.Floor` | `Utils.cs:1702-1707` (`vec.X = (int)vec.X`), consumed at `Main.cs:56480/:56501/:56522` | `map_player_icons.rs::draw_return_portal` + `draw_golf_marker` (`.floor()` → `as i32 as f32`; negatives differ) | `w4_f29_misc_icon_position_is_floored` (has the negative-coordinate case) |
| **F4** | Revenge markers draw `TextureAssets.Coin[0..3]` 1×8 animation strips, NOT the `Item_71..74` inventory icons | `CoinLossRevengeSystem.cs:262` (`TextureAssets.Coin[n]`), `CoinSlot.cs:242` (`TextureAssets.Coin[coinType−71]`); `lookups.coins` = `Item_71..74` (base `assets.rs:2356`) | pack `Coin_0..3` + `Lookups.coin_anim` (f29 `assets.rs` block); repoint `map_player_icons.rs::draw_revenge_map_icon` + `draw_revenge_mouse_over` from `lookups.coins` to `coin_anim` | f29 asset-pack witness's coin rows (merge into e17's) |
| **F5** | Fullscreen death-marker hover box has NO −4 shift: `x0 = x+4−14s, y0 = y+2−14s`. The `−4/−4` shifts exist ONLY in the minimap inline copy (`:55757-55758`) and the mapStyle-2 copy (`:55683-55684`) | `Main.cs:56455-56462` (`DrawPlayerDeathMarker` — read in full, no `-= 4f`) vs `:55755-55758` | `map_player_icons.rs::draw_death_marker` applies the minimap's net `(0,−2)` box to BOTH branches today → the fullscreen box is off by (−4,−4). Split per f29 (`draw_player_death_marker` fullscreen +4/+2; minimap keeps the −4) or add a `minimap: bool` arm. **Re-derive `w4_e17_death_hover_box_is_the_shifted_head_box`** — it currently pins the wrong fullscreen box | f29 `w4_f29_death_marker_transform` + the aac7375b9 minimap y-offset pin (108.55) |
| **F6** | Pan step as a pure helper + witness (e17's inline lerp is correct but unpinned) | `Main.cs:55361-55385` | `WorldMap::prepare`'s pan block delegates to f29 `map_pylons::pan_target_step` (relocated, e.g. into `map.rs` beside `pylon_view_fit`) | `w4_f29_pan_target_step` |

**Secondary folds (witness/coverage only, no behaviour change):**

- `w4_f29_head_pos_matches_the_two_modes` → adapt onto e17's
  `minimap_pin_pos`/`fullscreen_pin_pos` (e17 pins the fullscreen order but
  not the minimap transform).
- `w4_f29_last_hit_golf_ball_guards` → adapt onto e17's
  `golf_state.rs::last_hit_ball_for_draw` (the cleaner seam: guards stay in
  tr-core; f29's map-side re-derivation retires).
- `w4_f29_clamped_region_on_screen_unchanged` + `_left_overflow` → fold beside
  e17's pull-back witness (they pin the unchanged-rect and truncation cases).
- `w4_f29_localized_duration_largest_unit` → adapt onto e17's
  `ui/resources.rs::localized_duration_full` (see §4 retire list).
- f29's minimap remote-peer absence, `HeadMode` enum, 40-row const table,
  `find_chest_name` helper, `fullscreen_hover_name(tx,ty)` shape: **retire** —
  e17's equivalents are equal or better (e17's hover fn additionally ports the
  `chestType[52..] → chestType[0]` alias, `Lang.cs:503-506`, which f29 lacks;
  f29 would answer the raw missing key for styles ≥ 52).

## 4. Retire list (f29 code that does NOT survive)

- `map_overlay.rs` (whole module) — every fn has an e17 twin adjudicated above;
  the six fixes are extracted first (§3), the witnesses fold per §5.
- `map_pylons.rs` (whole module) — `draw_line`/`draw_pylon_lines`/
  `clamped_draw_region`/`pylon_right_click_view` duplicate e17;
  `pylon_line_endpoint` survives as the F1 body; `pan_target_step` as F6.
- `app/mod.rs::apply_pylon_click` + `request_pylon_teleport` — e17's
  `clicks.rs` pair wins (architecture + press-snapshot seam).
- `mount.rs::height_boost` accessor — redundant (`data()` gates active).
- `map.rs::clear_reset_pos` — e17 writes the field directly.
- `lang.rs::localized_duration(f64, abbreviated, show_all)` — e17's
  `localized_duration_full(i32, abbreviated)` covers both live call sites
  (`(false, false)`); the `showAllAvailableUnits` multi-unit path has NO caller
  in the ported surface — record it as a documented unported arm in ANALYSIS
  rather than keeping dead code. f29's anchors row for it retires with it.
- `Player.last_death_time: f64` epoch form — e17's `Option<SystemTime>` is the
  closer `DateTime.Now` twin; keep e17's field types (the KillMe stamps are
  otherwise identical, `Player.cs:38846-38848`).
- f29's `WorldMap.map_pylon_tile`/`pan_target: Option<[f32;2]>` — e17's
  `pylon_tile`/`pan_target: bool` + `pan_target_end` + `pan_cancel_press` +
  `cancel_wormhole` win (the C# field set).
- f29's draw.rs wiring — e17's follows the C# call order exactly:
  fullscreen revenge/misc `:55922-23` → heads `:55926` → pins `:55931-76` →
  death `:55977-83` → death-text/revenge-row `:55984-95` → **MapIcons.Draw
  `:55996` LAST**; minimap **MapIcons.Draw `:55703` FIRST** → revenge/misc →
  heads `:55709` → pins/death `:55712-66` → foreground → death/row
  `:55772-78`. f29 draws the fullscreen layer walk EARLY (before the heads and
  the revenge row) — markers/lines land under the heads instead of over, and
  the row draws over the markers. e17 also adds the remote-peer pins
  (`PlayerNet` snapshots, team 0/not-hostile documented) and the local-head
  fullscreen name hover (`:55959` writes for `k == myPlayer` too). KEEP e17's
  draw.rs wholesale, then apply F1-F6.
- Shared minor deviation (both lanes, record in ANALYSIS at merge): the C#
  truncates `Main.mouseX/Y` to int in minimap transform space
  (`Main.cs:55512-55513`); neither lane's hover compare reproduces the `(int)`
  step (f29 divides float, e17 folds the scale into the rects). Sub-pixel
  only; fix opportunistically with F2's hover pass.

## 5. Asset dedupe + witness union

**Assets (`assets.rs` STATIC_SHEET_NAMES + Lookups):**

| Sheet block | e17 | f29 | Keep |
| --- | --- | --- | --- |
| `NPC_Head_0..80` (full town range incl. shimmer 54-80, pets 27-53) | full 81 | 17-head map-only subset | **e17** (superset; f29's subset is contained) |
| `NPC_Head_Boss_0..39` | all 40 (deduped vs the boss-bar `_7` row in `2360c5dd3`) | 38 (skips `_7` pre-existing AND **misses `_31`**) | **e17** (the C# `AssetInitializer.cs:410-413` loop packs all 40) |
| `Coin_0..3` animation strips + `Lookups.coin_anim` | absent (wrongly reuses `coins` = Item_71..74) | present | **f29** (fix F4) |
| `MapDeath`, `Extra_173/175/176` | present | present | one copy (e17's block) |
| `Black_Tile` | lookup only (base packs it for the underworld box, `assets.rs:260`; second row deduped) | lookup only | one lookup |
| `UI/UI_quickicon1` | present | present | one copy |
| Lookups boss-head field | `npc_boss_heads` | `npc_head_boss` | **e17 name** (its consumers survive) |
| Atlas measurement | `tools/atlas_e17_check.js`: 8131/8192 | not measured | keep e17's tool; **re-measure after F4 adds the 4 coin strips** (merge-step verification, still under the 8192 bound by the strip sizes) |

**Witness union (raw 25 e17 + 23 f29 = 48 → ~34 after dedupe):**

- e17 keeps all 25: `map_npc_icons` 8 (table, overrides, rotation/flip, town
  profiles, destroyer chain, hover names, hover-half trunc + anchor, minimap
  clip), `map_player_icons` 9 (team borders, can-be-seen, fullscreen pin,
  minimap insets, centered-rect/Contains, death-hover box **[re-derive per
  F5]**, coin tiers, coins split, death pos), `map_layers` 3 (line segments,
  clamp pull-back, 28 px frame), `map.rs` 3 (c21 hover, view fit, open-clears-
  pylon), `lang.rs` 1 (c21 bounds/fill-order), `assets.rs` 1 (pack).
- f29 folds ~9 (adapted to e17 names): F1 endpoint, F3 floored, F5 death
  transform + minimap y-pin, F6 pan step, head-pos two-modes, golf guards,
  clamped on-screen + left-overflow, duration largest-unit; the asset witness
  merges into e17's (coin rows + town-head asserts).
- f29 drops ~14 as duplicates: boss table/overrides/rotation (e17's are
  stronger — kind-keyed), destroyer (wrong model), town profiles (weaker),
  hover names, clip strict, coin tiers, coins split, minimap death clip
  (e17's insets test covers both gates), right-click view (e17's fit test),
  find-chest-name (e17's c21 hover test covers legend/chest/dark/window).
- All folded lang-dependent tests take the `crate::lang::test_lock()`
  (the 9f80c5375 convention — process-global LanguageManager).

**Docs/anchors dedupe:**

- `anchors.txt`: keep e17's 3 rows (`map_player_icons`, `map_npc_icons`,
  `map_layers::pylon_line`); f29's 3 rows retire with their modules EXCEPT the
  pylon-line coverage is already e17's. Add NO row for a retired fn.
- `PORT_MAP.md`: keep e17's two new rows + its S1-L-4 row extension; **fold
  f29's `TeleportPylonsSystem` row edit** (the "map-click UI landed" update —
  e17 never touched that row) repointed at `clicks.rs::handle_pylon_map_click`.
- `ROADMAP.md`: keep e17's S1-L-4 extension.
- `research/14-map-minimap.md`: keep e17's census references; fold f29's
  PORTED-section census + the **Verify-only** notes (quest-fish marker is a
  phantom — grep-verified; mapStyle-2 overlay unwired; unity-potion hover
  MP-only; `DrawDebugMapOverlays` dev-only) so no future census re-hunts them.
- `ANALYSIS.md`: keep e17's two sections; merge f29's deviation notes that
  still apply (notification gate, TE post-place seam, `Utils.Floor`
  truncation — now a FIX not a deviation, DrawMoney row) and add the §4
  shared mouse-truncation note + the `showAllAvailableUnits` unported arm.

## 6. Merge order

1. **e17 → main first** (it is the skeleton; per MAIN-RULES ref-update merge,
   canary + symbol verify after push). Its tip `8500c3e5d` already contains
   the c21 cherry-pick base, so no separate c21 landing is needed for the
   map-object slice.
2. **This branch syncs onto post-e17 main** (`git merge origin/main` in the
   f29 worktree). Conflict resolution is NOT ours/theirs: delete
   `map_overlay.rs`/`map_pylons.rs` and the f29 draw.rs/mod.rs/doors.rs/map.rs
   hunks per §4, keep e17's files, then apply F1-F6 as small cited commits
   (each with its folded witness).
3. **Asset/doc dedupe** per §5 in the same sync (one Coin block, one boss-head
   field, anchors/PORT_MAP/ANALYSIS union). Re-measure the atlas with
   `tools/atlas_e17_check.js` after F4.
4. **Verify on the synced branch**: `cargo test --workspace --skip
   migrates_version_2_kinds_to_item_ids` green (judge by `test result: ok`),
   canary `main_anchors` green, then merge f29 → main per MAIN-RULES and
   re-verify the merge survived (ancestor check + symbol grep).
5. Cleanup: delete both task branches + worktrees per the lifecycle rules.

**Ordering rationale:** e17-first avoids rewriting the larger, correct
skeleton; f29's six fixes are small, independently witnessed patches that
apply cleanly onto e17's named sites. Reversing the order would force
re-porting the shimmer profiles, the type-keyed boss table, the remote pins,
the press snapshot and the C#-order wiring — all of which f29 lacks or got
wrong.

---

*Evidence: full reads of `Main.cs:55288-55429/:55496-55526/:55596-55695/`
`:55700-55794/:55806-55903/:55915-56004/:56099-56338/:56446-56495/:56541-56575/:56631-56643`,
`NPC.cs:7557-7646`, `MapOverlayDrawContext.cs` (full), `TeleportPylonsMapLayer.cs:1-119`,
`SpriteFrame.cs:44-77`, `Player.cs:29103-29118/:32178-32213`,
`CoinLossRevengeSystem.cs:245-294`, `ItemSlot.cs:3650-3689`, `CoinSlot.cs:240-254`,
`Lang.cs:494-518/:550-639/:665/:1158-1202`, `Utils.cs:947-952/:1702-1707`,
`AnOutlinedDrawRenderTargetContent.cs` (full), `OutlinedTextureRenderer.cs` (full),
`TownNPCProfiles.cs::GetHeadIndexSafe`, `Profiles.cs:27-51`, `TileID.cs:487`;
both branch diffs vs `a725acb04` read in full (draw.rs, assets.rs, map.rs,
map_layers.rs, lang.rs, player.rs, doors.rs, clicks.rs, mod.rs, mount.rs,
golf_state.rs, resources.rs, anchors.txt, PORT_MAP.md, ROADMAP.md); base-model
checks `npc/data.rs:929/:1144/:1186`, `bestiary.rs` Golem fold,
`npc_ai_style37.rs:443/:1090-1117`, `mount.rs:2465`, `assets.rs:260/:2356`,
`ui/housing.rs:41`, `ui/minimap.rs::minimap_origin_raw/scale_rect`.*
