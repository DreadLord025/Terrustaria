# 140 — Map-overlay 2-way reconcile (W4-f29 × W4-e17)

**Phase:** ~~reconcile, docs-only~~ → ~~EXECUTED~~ → **SEALED
(2026-11-21, seal phase, code-only, `task/W4-f29-map-overlay-2`)**. The
adjudication stands as the record; the execution log is §7; the
map-family unified merger instruction is §8; the cross-lane seal
receipts (e18g mutual-receipt, f46g compose, d35/d42 atlas) are §11.
No cargo/tests were run (code-only mode) — the witnesses are WRITTEN
NOT RUN. The atlas measurement DID run (§8.4, re-run at seal).

> **MERGER INSTRUCTION — MAP-FAMILY UNIFIED, keep BOTH branch tips (§8).**
> This branch was rebuilt onto the e17 skeleton (`8500c3e5d`) and carries
> F1-F6 as cited patches on top of it (§7); its tip tree IS the e17
> skeleton + the six fixes + the deduped docs/witnesses. The sibling
> `task/W4-f46-minimap-reveal-2` (tip `6fb5e01e8`) carries the e18 reveal/
> keys union (research/84). **f46-union-keep AND f29-overlay-keep — neither
> absorbs the other**: the two diffs are disjoint subsystems sharing six
> conflict sites; compose them per the §8 hunk table (**f46g-ruling:
> f46-union FIRST, f29-TIP SECOND** — §8.0; the table itself is
> order-independent). Do NOT
> cherry-pick the pre-rebuild f29 commits (the `map_overlay.rs`/
> `map_pylons.rs` era, ≤ `d5aa3621f`): those modules are retired and their
> code must not re-land — the exact-hash warning list is §9. The e17 lane
> is DEPRECATED as a separate merge (§10). The e18f style-2 D18 draw
> (`099beeba5`) is NOT a third landing: it re-folds into the f46 union
> BEFORE f46 lands, and the overlay-draw keep-one is ruled in §8.5.
> Review the tip diffs (e17→f29-tip: 12 files; base→f46-tip: 25 files),
> not the branch histories.

**Phase (original):** reconcile, docs-only. No code changes to e17. This document is the
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

## 6. Merge order (SUPERSEDED by §8 — the e17-first plan is retired)

> **§6 is historical.** Step 1 ("e17 → main first") is RETIRED: the f29 tip
> already absorbs the e17 skeleton (§7), and the two-landing compose is
> f46-union-first → f29-TIP-second (§8.0, the f46g-ruling). The original
> text stands below as the execution record's rationale.

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

## 7. Execution log (2026-11-21, code-only, this branch)

The §6 order was executed with one deviation forced by the exec brief: the
branch could not wait for e17 → main, so step 2's "sync onto post-e17 main"
was done as a **direct rebuild onto the e17 tip `8500c3e5d`** — the map-file
set was taken from e17 wholesale (`git checkout 8500c3e5d -- …`), the f29
duplicates deleted, and F1-F6 applied as small cited commits. The resulting
tree is exactly what §6 step 2's conflict resolution would have produced;
the merger keeps the f29 TIP (see the banner + §8 — it lands SECOND onto
post-f46 main per the f46g-ruling; neither branch absorbs the other).

| Commit | Scope |
| --- | --- |
| `352644d24` | Step 1 — rebuild onto the e17 skeleton; retire list §4 applied (`map_overlay.rs`/`map_pylons.rs` deleted, `mount.rs` reverted to base, e17's player.rs/lang.rs/map.rs/draw.rs/clicks.rs/doors.rs/assets.rs/anchors/PORT_MAP/ROADMAP/ANALYSIS taken) |
| `e78178e15` | **F1** — `unclamped_center_int` + the endpoint witness (fractional case added) |
| `f8df2276f` | **F2** — `rect_contains_int` on all three context hovers; the e17 pull-back witness extended + the two clamp witnesses folded |
| `22df1dee8` | **F3** — `utils_floor` on the portal/golf positions + the floored witness (negative case) |
| `8ccb7966b` | **F4** — `Coin_0..3` + `Lookups.coin_anim`, both revenge draws repointed, the asset witness merged, atlas re-measured **8131 px < 8192** |
| `8a29b8253` | **F5** — `draw_death_marker(minimap)` + `death_hover_rect`; the wrong e17 witness re-derived; the death-transform witness folded (minimap y-pin 108.55) |
| `b50d96ca0` | **F6** — `map.rs::pan_target_step` pure helper, `prepare` delegates, the pan witness folded |
| `c19521b32` | Secondary folds — head-pos two-modes, duration largest-unit (test_lock), golf guards (tr-core seam) |
| `c15a13188` | Docs dedupe §5 — ANALYSIS exec section + corrected e17 claims, PORT_MAP pylon row, research/14 SUPERSEDED banner |

**Witness union landed: 34.** e17's 25 (the death-hover one re-derived per
F5) + 9 f29 folds: F1 endpoint, F3 floored, F5 death transform, F6 pan step,
head-pos two-modes, golf guards, clamped on-screen, clamped left-overflow,
duration largest-unit; the F2 far-edge exclusion extends the e17 pull-back
witness and the coin/town-head rows merge into the e17 asset witness (no
new test fns). ~14 f29 duplicates dropped per §5. All WRITTEN NOT RUN
(code-only mode); the atlas measurement DID run (`node
tools/atlas_e17_check.js`: baseline 8123 → 8131 px, OK).

**Verification owed at merge (§6 step 4):** `cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids` green, canary `main_anchors` green,
then the ancestor + symbol checks per MAIN-RULES.

---

## 8. MAP-FAMILY unified merger instruction (retire phase, 2026-11-21)

**Verdict (f46g-ruling): BOTH-branches-keep — f46-union-first →
f29-TIP-second; neither branch absorbs the other.** The two tips carry
disjoint subsystems (f46 = the e18 reveal/sections/keys/close-button union,
research/84; f29 = the e17 overlay skeleton + F1-F6, §7 above — ~4 500 ln
per 84 §10.3's scope check, crates-side diff +3 801/−118 over 16 files,
its own EXECUTED keep-one with 34 witnesses) that share exactly six
conflict files. An absorb-either-way move would force re-porting a whole
subsystem; the per-hunk compose below is small and mechanical.
Cross-record: research/84 §10.3 (on the f46 branch, tip `6fb5e01e8`)
publishes the SAME two-landing order from the reveal side — f46 first,
f29 tip second. The e18f style-2 D18 draw (`099beeba5`) is a re-fold into
the f46 union, NOT a third landing — the overlay-draw keep-one is §8.5.

**Tips compared (re-fetched this RESUME pass, 2026-11-21):**

| Lane | Tip | Diff vs base `a725acb04` |
| --- | --- | --- |
| `task/W4-f29-map-overlay-2` (this branch) | `809d08238` (+ this retire commit) | 23 files; the map-file set IS e17 `8500c3e5d` + F1-F6 (12-file diff vs e17) |
| `task/W4-f46-minimap-reveal-2` | `6fb5e01e8` (FINAL-FINAL; moved past the earlier `05a9f7b64` probe) | 25 files, +3381/−140 crates-side; e18 union base `670c948d` + final-sweep re-folds EXECUTED (84 §10.2) + the witness re-derivation `447494570` (1280×720 headless frame) |

### 8.0 Landing order (f46g-ruling)

1. **f46 tip → main first** (`6fb5e01e8`; MAIN-RULES ref-update merge;
   it carries the whole reveal/input family per 84 §10.3 step 1 — and
   the e18f D18 draw re-fold of §8.5 lands on it BEFORE this merge).
2. **f29 TIP → main second** (`809d08238` + this retire commit), synced
   onto post-f46 main, resolving the six conflict files per §8.2 (the
   table is written f29-side/f46-side, order-independent — the same nine
   hunks appear either way; the ruling picks f46-first to match 84 §10.3
   and because the union's `pan_target: Option`/`recentre`/alpha state
   are the surviving seams the f29 hunks fold ONTO). Its diff already
   contains the whole e17 lane — e17 itself never lands (§10).
3. §8.3 semantic folds + the §8.5 D18 seam land WITH the second merge as
   small cited commits.
4. Verify on the merged state: `cargo test --workspace --skip
   migrates_version_2_kinds_to_item_ids` green (judge by `test result:
   ok`), canary `main_anchors` green, atlas re-measure (§8.4), then the
   ancestor + symbol checks per MAIN-RULES. All witnesses on both
   branches are WRITTEN NOT RUN (code-only mode) — the catalog run at
   merge is the first execution of the witness union: 34 f29-tip fns +
   56 f46-tip fns, zero shared names (verified by `git grep -oE "fn
   w4_"` on both tips), + the 2 D18 witnesses §8.5 folds = ~92.
5. Retire per §10 + the §9 warning list.

### 8.1 File-overlap check

Both diffs touch: `anchors.txt`, `app/clicks.rs`, `app/flow.rs`,
`app/frame.rs`, `app/frame/draw.rs`, `app/mod.rs`, `assets.rs`,
`main.rs`, **`map.rs` (SAME FILE BOTH — the headline overlap)**,
`docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/ROADMAP.md`,
`docs/research/14-map-minimap.md`.

Dry-run `git merge-tree --write-tree --merge-base a725acb04 809d08238
6fb5e01e8` (tree `feddf3178`, reproducible; re-ran this RESUME pass after
the f46 tip moved — the earlier `05a9f7b64` probe gave tree `f80877dba`
with the SAME six files/nine hunks at the SAME line ranges): **6
conflicted files, 9 hunks** — `map.rs` ×3, `frame/draw.rs` ×2,
`clicks.rs` ×1, `main.rs` ×1, `anchors.txt` ×1, `research/14` ×1.
Everything else auto-merges cleanly,
including the heavy shared files: `app/mod.rs` (f46's `section_manager`/
`debug_unlock_map` fields + the four movement-key `pan_target = None`
cancels + f29's `map_left_press` field all coexist), `assets.rs` (f46's
`Cursor_11` + f29's `Coin_0..3`/boss-head/icon blocks, one atlas),
`app/flow.rs` (both test-rig field additions), `app/frame.rs` (f46's
OS-pointer gate + f29's `map_left_press` snapshot at :243),
`frame/world.rs` (f46-only: `map_reveal_draw` driver + the ping→close
click order), `map_layers.rs`/`map_npc_icons.rs`/`map_player_icons.rs`/
`doors.rs`/`lang.rs`/`ui/resources.rs`/`golf_state.rs` (f29-only),
`map_sections.rs`/`ui/chrome.rs`/`ui/cursor.rs`/`ui/minimap.rs`/
`app/loop.rs`/`netglue.rs`/`debug_panel.rs`/`world_sim/player.rs`/
`tr-core/map/mod.rs` (f46-only). The auto-merged `anchors.txt` conflict
resolves to the full 21-row union (3 f29/e17 + 18 f46) with zero base
rows lost — verified row-by-row against all three tips.

### 8.2 Per-hunk compose table (the 9 conflict hunks)

Line numbers are the dry-run tree `feddf3178`'s conflicted files
(re-verified identical to the `f80877dba` probe).

| # | File / hunk | f29 side (HEAD) | f46 side | Resolution |
| --- | --- | --- | --- | --- |
| 1 | `map.rs:656-726` — `WorldMap` field tail | `pylon_tile`, `pan_target: bool`, `pan_target_end`, `pan_cancel_press`, `cancel_wormhole` | `update_map`, `refresh_map`, `change_queues`, `load_map`, `clear_map`, `map_ready`, `load_map_lock`, `load_map_last_x` | **KEEP BOTH field sets, but the pan state unifies on f46's `pan_target: Option<[f32;2]>`** (already declared above the conflict, f46's doc note names this fold): drop f29's `pan_target: bool` + `pan_target_end`; keep `pylon_tile`/`pan_cancel_press`/`cancel_wormhole`. `None` = the C# flag false, `Some(end)` = armed + destination in one field. |
| 2 | `map.rs:754-777` — `WorldMap::new` initialisers | `pylon_tile: (-1,-1)`, `pan_target: false`, `pan_target_end: [0,0]`, `pan_cancel_press: false`, `cancel_wormhole: false` | the eight reveal-statics seeds (`clear_map: true` per `WorldGen.cs:6982`) | **KEEP BOTH**, minus the two retired pan initialisers (the shared `pan_target: None` seed already sits above the hunk). |
| 3 | `map.rs:1264-1275` — `prepare`'s `reset_pos` arm | inline `pan_target = false; pos = center_tile; reset_pos = false` | `self.recentre(center_tile)` | **KEEP f46's `recentre` seam** — it clears the pan FIRST (`Main.cs:55418`) then recentres + disarms; identical semantics on the Option field. f29's inline arm retires. |
| 4 | `frame/draw.rs:2227-2235` — the `prepare` call | `m.pan_cancel_press = self.mine_down && self.map_left_press;` + 6-arg `prepare` | 7-arg `prepare(..., settings.map_enabled)` | **KEEP BOTH**: f29's press-feed line, then f46's 7-arg call (the `map_enabled` param feeds the clearMap consume). |
| 5 | `frame/draw.rs:2520-2595` — fullscreen draw tail after the hover text | the `:55988-55995` revenge `UseMouseOver` money row (`if !death_hovered`) | close button + thick cursor + a STALE single local-head icon (f46's pre-overlay base remnant) | **KEEP f29's money row + f46's close-button and cursor blocks, in that order** (C# `:55988` row → `:56014` button → `:56037` cursor); **DROP f46's single-head block** — f29's 255-loop (local + remote pins, `:55931-55976`, already auto-merged above the hunk) replaces it per 84 §10.3. Mind the shared closing brace: f29's side ends inside `if let Some(i)`, f46's inside `if let Some(p)` — re-balance when splicing. |
| 6 | `clicks.rs:864-1032` — the fn block after `handle_fullmap_ping` | `handle_pylon_map_click` + `request_pylon_teleport` (the gate ladder) | `handle_fullmap_close` | **KEEP BOTH** (disjoint fns, one after another). In `handle_pylon_map_click`'s `PanTo` arm replace `m.pan_target = true; m.pan_target_end = [x,y]` with `m.pan_target = Some([info.x as f32, info.y as f32])` (hunk-1 field unification). The production wiring auto-merged: `frame/world.rs:29-32` ping→close, `draw.rs` pylon-click drain — untouched. |
| 7 | `main.rs:59-64` — module list | `mod map_npc_icons; mod map_player_icons;` | `mod map_sections;` | **KEEP BOTH** (alphabetical: npc_icons, player_icons, sections). |
| 8 | `anchors.txt:899-922` — the appended rows | 3 overlay rows (`draw_misc_map_icons`, `draw_npc_map_icons_fullscreen`, `pylon_line`) | 18 reveal/keys rows | **KEEP BOTH** — all 21 rows, zero deletions (the canary rule). Verified: every row's marker exists in the composed tree. |
| 9 | `research/14:402-939` — appended census sections | the f29 overlay census + its SUPERSEDED banner (points here) | the e18 residual + f46 closeout censuses + the 84 cross-notes | **KEEP BOTH sections** (84 §10 item 6's keep-both rule); f29's banner stays on the f29 section only. |

### 8.3 Semantic folds (auto-merged text that must still change)

These compile-break or mis-behave if the merge is text-only — the dry-run
tree carries them as-is:

1. **`map.rs::prepare` pan block → Option.** The auto-merged body still
   reads `if self.pan_target { pan_target_step(self.pos,
   self.pan_target_end, ...) ... self.pan_target = false }`. Adapt:
   `if let Some(end) = self.pan_target { let (pos, done) =
   pan_target_step(self.pos, end, self.pan_cancel_press); self.pos = pos;
   if done { self.pan_target = None; } }`. The pure F6 `pan_target_step`
   + its witness are representation-agnostic — unchanged.
2. **`world_sim/editing/doors.rs:242` 597-arm pan clear → `m.pan_target
   = None`.** Per 84 §10.3 the arm's TryOpening effects should route
   through f46's `app/loop.rs::try_open_fullscreen_map()` (inventory/
   sign/talk clears + sound 10 + `set_open`; verified present on the
   `6fb5e01e8` tip); f29's inline copy adds `close_shop()` — fold that
   one line into the shared helper (C# `Player.cs:29107-29110` closes
   the shop too) instead of keeping the duplicate. The `pylon_tile`/
   `pylon_view_fit`/`reset_pos = false` tail stays on the f29 arm.
3. **Minimap live alpha into f29's overlay passes (D17 fold, 84 §10.3).**
   The layer-walk call already auto-merged with f46's `m.minimap_alpha`
   last arg + f29's 3-tuple return and extended `MapIconInputs` — no
   edit. But f29's later minimap passes hardcode alpha 1.0:
   `NpcIconView.alpha` (`draw.rs:1679`), `draw_death_marker(..., 1.0,
   ...)` (:1740), and the player-head draws (:1723/:1784) — the C#
   passes `b/255` to `DrawNPCMapIcons2` (`:55709`), the `MapDeath` tint
   (`:55754`) and `DrawPlayerHead` (`:55729`). Replace the 1.0s with
   `m.minimap_alpha`; `draw_player_map_icon` needs an alpha param (or a
   tint multiply at the call). Fullscreen keeps 1.0 (`b = byte.MaxValue`
   at `Main.cs:55291`, never reassigned in the fullscreen arm). The
   revenge/misc passes take no `b` in C# (`:55705-55706`) — leave them.
4. **f46's `pan_target` doc note + anchors row stay truthful**: the note
   says "the overlay pick folds only its lerp step + pylon-click arming
   onto it" — after folds 1-2 that is exactly the state; no edit needed.

### 8.4 Atlas re-measure (ran, this retire pass)

`node tools/atlas_e17_check.js` on this tip: baseline 8123 → **8131 px**
(47 icon sheets, OK). With f46's `UI/Cursor_11` (18×18) added through the
same packer: **8131 px, 9606 sheets — still OK** (the 18-px row packs into
existing slack). Bound 8192; margin 61 px. Re-measured live this RESUME
pass (both runs above re-executed, identical numbers). Re-run at merge
per 84 §10.3.

### 8.5 The e18f style-2 D18 draw keep-one — f29 overlay vs e18 `099beeba5` (f46g-ruling)

**The two are NOT rivals for the same pixels — but where they touch, ONE
implementation lands, and the D18 draw re-folds into the f46 union, NOT
into f29.**

**What e18f ported** (`099beeba5`, on `task/W4-e18-minimap-reveal-queue`,
NOT an ancestor of the f46 tip `6fb5e01e8` — verified `merge-base
--is-ancestor` exit 1): the style-2 overlay DRAW arm verbatim —
`map.rs::draw_overlay` + `clamp_overlay_axes` + `map_overlay_quad`
(`Main.cs:55548-55578` + `:55609`, the `(byte)(255*mapOverlayAlpha)` tint
`:55568`), the `draw.rs` call site (`Main.cs:62231-62235`'s in-world
gate), the `num37` layer-walk call at the overlay transform
(`:55615-55625`), the `:56046` MouseText tail, the pylon-scan gate
widened to style 2, 3 witnesses + 1 anchor row
(`map.rs|pub fn draw_overlay`). The `:55626-55694` icon block is
explicitly LEFT to the f29/e17 overlay keep-one ("never twice" — the
commit message says so).

**What f29 owns**: the nine overlay DRAW pieces (~4 500 ln:
`map_npc_icons.rs` 950 + `map_player_icons.rs` 809 + `map_layers.rs`
+723 + `map.rs` +320 + `clicks.rs` +139 + `assets.rs` +216 + draw wiring
+520 per 84 §10.3), F1-F6, its own EXECUTED rebuild + 34-witness union.
f29 does NOT port the style-2 tile pass — its overlay lane is the
fullscreen/minimap icon passes (`:55626-55694`/`:55922-56004`). The
e18f draw is the `:55548-55625` half the f29 census never claimed.

**Keep-one rulings (per shared piece):**

| Piece | e18f `099beeba5` | f46 union `6fb5e01e8` | f29 tip | KEEP |
| --- | --- | --- | --- | --- |
| `overlay_scale`/`overlay_alpha` fields + seeds 2.5/0.35 | present | present (`a352b16e` fold) | absent | **union's** — e18f copied them byte-identical FROM the union; one copy at merge |
| `map_key_alpha`/`map_alpha_clamp`/`overlay_scale_clamp` helpers | byte-identical copies | originals (`a352b16e`) | absent | **union's** (skip e18f's copies — the e18 branch's own `310c7b9b4` re-fold recipe says exactly this) |
| style-2 key rows ×1.05/×0.95 ±0.015 | in `update_minimap_keys` | in `update_minimap_zoom_keys` (§10.1 keep-one) | absent | **union's fn** — e18f's key rows DROP with its `update_minimap_keys` variant (84 §10.1: the merger drops e18's variant wholesale) |
| `draw_overlay`/`clamp_overlay_axes`/`map_overlay_quad` + the draw.rs call site + num37 walk + MouseText tail + pylon-gate widening | **UNIQUE — the union lacks it** (verified: `git grep "fn draw_overlay" 6fb5e01e8` empty) | absent | absent | **e18f's, re-folded onto the union** (recipe below) |
| the `:55626-55694` icon block (revenge/misc/NPC heads/255-loop/death) | explicitly deferred | single-head remnant (drops per §8.2 hunk 5) | **the full port** | **f29 tip's** — never twice |
| 3 witnesses | `w4_e18_the_style_two_keys_step_the_overlay_axes` (dup — union's `w4_f46_style_two_keys_step_the_overlay_scale_and_alpha` covers the input rows), `w4_e18_the_overlay_clamps_match_the_draw_side` (**unique half** — pins the `WorldMap::clamp_overlay_axes` METHOD, which only the draw arm brings; the union's `w4_f46_alpha_and_overlay_clamps_match_the_draw_side` pins the pure fns only), `w4_e18_the_overlay_quad_is_screen_centred_and_unscaled` (**unique**) | the pure-fn style-2/clamp witnesses | — | **2 fold with the draw arm** (quad + clamps-method; the clamps witness's trailing key-helper asserts dedupe against the union's pure-fn witness at the fold), the keys witness drops with the keys keep-one — matches the e18 census `310c7b9b4`'s "2 fold with the draw arm" |

**Re-fold recipe (lands on the f46 branch BEFORE its merge — NOT on
f29):** cherry-pick `099beeba5` onto `6fb5e01e8`; per-hunk: SKIP the
byte-identical helper/field hunks (map.rs state half), DROP the
`update_minimap_keys` style-2 rows (the union's `update_minimap_zoom_keys`
already steps them), TAKE `draw_overlay`/`clamp_overlay_axes`/
`map_overlay_quad` + the draw.rs call-site hunk + the pylon-scan gate
widening + the 2 draw-arm witnesses (quad + clamps-method; dedupe the
clamps witness's trailing key-helper asserts against the union's
pure-fn witness) + the anchor row + the research/14 MAP-FINAL
section + the PORT_MAP/ANALYSIS rows. The draw.rs call-site hunk applies
against the union's evolved draw.rs (the `447494570` witness re-pin moved
line numbers only, not the seams). The layer-walk call inside the D18
arm constructs `MapIconInputs` WITHOUT f29's 4 extension fields
(`map_fullscreen`/`pylon_tile`/`screen`/`left_press` — verified absent
from the e18f hunk, present on the f29 tip's struct): when the f29 tip
merges second (§8.0 step 2), that THIRD `map_layers_draw` call site gains
the 4 fields like the other two (84 §10.3's "BOTH union call sites"
becomes all three): `map_fullscreen: false`, `pylon_tile: (-1, -1)` (the
style-2 arm draws no network lines — `TeleportPylonsMapLayer.cs:45`'s
fullscreen gate), `screen`, `left_press: false` (the marker click is
fullscreen-only, `:83`/`:90`). The walk's returned `PylonClick`/
`cancel_wormhole` halves are inert at `map_fullscreen: false` — drop
them at the call, keep the hover string for the MouseText tail.

**Why NOT into f29:** folding the D18 draw into f29 would split the
style-2 system across two branches — the keys/alpha state/clamps live on
the union (`a352b16e`), the draw would live on f29, and the f29-first
merge order would land a draw whose clamps/keys arrive one merge later.
f46-first + the D18 re-fold keeps the whole style-2 system on ONE
branch, and f29's second merge only extends the shared `MapIconInputs`
struct — the same seam treatment as the other two call sites. This
matches the e18 branch's own published recipe (`310c7b9b4`: "the
099beeba5 re-fold recipe — skip the byte-identical helpers/fields, drop
the dup key rows, take the draw arm + 2 witnesses + anchor") verbatim.

## 9. Retired-commit warning list — pre-rebuild f29 commits MUST NOT re-land

The eleven commits below are this branch's pre-rebuild era (the
`map_overlay.rs`/`map_pylons.rs` modules + their wiring). The rebuild
`352644d24` deleted those modules and re-took the map-file set from e17
wholesale; **none of this code may re-land** — not by cherry-pick, not by
a stale-tree merge resolution, not by "restoring" a file the merger
thinks was lost. If a merge diff shows `map_overlay.rs` or `map_pylons.rs`
being ADDED, the resolution is wrong — abort and redo per §8.

| Hash (full) | Short | Era scope |
| --- | --- | --- |
| `e1ffdefc5658a44b0caf5bd3eeda51b56cbd059a` | `e1ffdefc5` | census (docs; superseded by the deduped research/14 section) |
| `09f71b61f1afcf04388e01a9ebfe16c2431f01bc` | `09f71b61f` | P1 asset pack (17-head subset; superseded by e17's full 81-head set + F4 coin rows) |
| `9320326cadcaf75cccc6457cb206aecb3003805a` | `9320326ca` | P2 `DrawNPCMapIcons` port (`map_overlay.rs`; net-id Golem bug, default-table townies, self-loop Destroyer walk — §1) |
| `1de1ad19ae0e5d4e9b5ae9b971a85dbb3d4a8f2f` | `1de1ad19a` | P3 death/revenge/misc/minimap-heads/hover/`LocalizedDuration` (`map_overlay.rs` + f64 death epoch + f64 lang form — §4) |
| `a920b704ce4bb1d5b55491fd0743cdd03e85e456` | `a920b704c` | P4 pylon lines/click/pan (`map_pylons.rs`, `apply_pylon_click`, `Mount.height_boost` — §4) |
| `a0516edf4117d34c7055474690a240da2044fcc7` | `a0516edf4` | review fixes onto the retired modules (the surviving semantics re-landed as F1/F3) |
| `e18bf6dd2d9ae2fcba963e8691d6e0a104c92348` | `e18bf6dd2` | docs/anchors for the retired homes (3 anchor rows pointed at `map_overlay.rs` — the live rows point at e17's modules) |
| `aac7375b9cd5905ef8a2896e8b3d044965833cc0` | `aac7375b9` | minimap death y-pin witness fix (the 108.55 pin re-landed inside F5's folded witness) |
| `598f32c6da3b0d6ad07d0926cde02ab3b0a3bb3a` | `598f32c6d` | hover-tile +10 fix onto the retired draw wiring (e17's wiring already carries it) |
| `9f80c5375f98131affc79da35beed5154d3e1e7a` | `9f80c5375` | lang test_lock convention (re-landed in the witness union `c19521b32`) |
| `d5aa3621f29e07f304d0ffa39c4f2114cdfccdb3` | `d5aa3621f` | the reconcile doc itself (this file's earlier revision; the tip's version supersedes it) |

The ten EXEC commits (`352644d24` rebuild, `e78178e15` F1, `f8df2276f`
F2, `22df1dee8` F3, `8ccb7966b` F4, `8a29b8253` F5, `b50d96ca0` F6,
`c19521b32` witness union, `c15a13188` docs dedupe, `809d08238` EXECUTED
status) are the branch's live history — but the merger still reviews the
TIP TREE (12-file diff vs e17 `8500c3e5d`), not the history: the rebuild
commit's diff contains the wholesale deletion of the retired modules and
must not be replayed onto a tree that never had them.

## 10. e17 branch DEPRECATED — drop wholesale, do not merge

`task/W4-e17-map-overlay-layers` (origin tip `8500c3e5da0a1ebfda03123d412e37114e50aeeb`,
re-probed this retire pass: **UNMOVED** since the §7 rebuild — zero
post-skeleton commits) is fully absorbed: this branch's rebuild took its
entire map-file set wholesale, and every e17-touched file outside that
set is byte-identical in the f29 tip (verified: the e17-vs-base file list
is a subset of the f29-vs-base list, and the 12-file e17→f29-tip diff is
exactly F1-F6 + witnesses + docs). **Instruction to the merger: e17 never
lands as its own merge — drop it wholesale.** The original §6 step 1
("e17 → main first") is retired. After the f29 tip lands, delete
`task/W4-e17-map-overlay-layers` local + remote per the lifecycle rules
(its tip is content-contained, though not a git ancestor — the rebuild
was a tree-take, so `git branch -d` will refuse; verify containment by
the 12-file diff review, then delete with `-D` and record this section as
the justification). The same retire rule applies to
`task/W4-e18-minimap-reveal-queue` on the reveal side — with ONE
qualification this RESUME pass verified: the e18 tip moved past the
docs-only `a1a8be6e9` to `310c7b9b4` (the MAP-FINAL P1-P4 commits,
including the D18 draw `099beeba5` + the witness re-derivation
`4cc600a97`). The branch is fully absorbed ONLY AFTER §8.5's D18
re-fold lands on the f46 union — the draw arm is e18-unique content
until then. The witness re-derivation `4cc600a97` mirrors the union's
`447494570` (identical bodies at merge, nothing to fold). Retire e18
after the f46 merge per 84 §10.2/§10.3 + §8.5 here.

## 11. MAP-OVERLAY SEAL (2026-11-21, seal phase) — the four cross-lane receipts

**STATUS: SEALED — no further f29 work of any kind.** Everything below
was re-verified at the fetched seal-phase tips: e18 `5509cb87a` (§10.10
SEAL), f46 `6fb5e01e8` (FINAL-FINAL, unmoved), d35 `1afd327b9` (ATLAS
RETIRE phase 4), d42 `bac408be0` (FINAL doc), this branch `3abf3bf43`
+ this seal commit. Code-only: no cargo/tests/rustfmt/graphify; the
atlas measurement DID run (§11.3). All witnesses remain WRITTEN NOT
RUN — the catalog run at merge is their first execution.

### 11.1 e18g mutual-receipt — §8.5 D18-keep-one vs e18 §10.10's 16-hunk recipe: AGREE

The e18 lane sealed at `5509cb87a` (research/84 §10.10 SEAL +
research/14 census FINAL/SEALED + the MAP SEAL receipt). Their exact
`099beeba5` re-fold recipe (16 hunks) was read hunk-by-hunk against
this file's §8.5 keep-one table. **Verdict: full agreement — same
destination (the f46 union, NOT f29), same TAKE/SKIP/DROP dispositions,
same 2 folding witnesses.** Their table is the more precise instrument
and is hereby ADOPTED as the operative recipe; §8.5's summary recipe
stands as the ruling record. Concordance:

| Piece | §8.5 (this file) | e18 §10.10 SEAL (16 hunks) | Agree? |
| --- | --- | --- | --- |
| 3 helpers `map_key_alpha`/`map_alpha_clamp`/`overlay_scale_clamp` | SKIP — union's `a352b16e` copies stay, ONE copy | SKIP (`map.rs @@ -53`), brace-matched byte-identical `True` ×3 | ✓ |
| `overlay_scale`/`overlay_alpha` field decls + seeds 2.5/0.35 | SKIP — union's stay | SKIP decls (`@@ -425`) + seeds (`@@ -517`); **TAKE the comment refresh** — the union's "D18 out of scope" doc wording is stale post-fold (their precision fix: decls+seeds identical, comments differ) | ✓ — §8.5's "SKIP the field hunks" is refined by their split: decls/seeds skip, comments take |
| `map_overlay_quad` / `clamp_overlay_axes` / `draw_overlay` | TAKE — e18f's, re-folded onto the union | TAKE wholesale (`@@ -121`/`@@ -558`/`@@ -1033`), union grep 0 hits | ✓ |
| style-2 key rows ×1.05/×0.95 ±0.015 | DROP with e18's `update_minimap_keys` variant (union's `update_minimap_zoom_keys` carries them) | DROP (`app/mod.rs @@ -1515` + the `@@ -1474` doc hunk); the folded draw-side `clamp_overlay_axes` is a benign double projection — KEEP for draw-side fidelity | ✓ |
| keys witness `w4_e18_the_style_two_keys_step_the_overlay_axes` | DROP — union's `w4_f46_style_two_keys_step_the_overlay_scale_and_alpha` covers | DROP (`@@ -6106`) **incl. the whole `w4_e18_minimap_key_tests` mod + the `minimap_app` fixture** (union has `w4_f46_minimap_zoom_key_tests`) | ✓ — their fixture/mod drop is the same keep-one, stated wholesale |
| 2 draw-arm witnesses (quad + clamps-method) | FOLD with the draw arm; clamps witness's trailing key-helper asserts dedupe | TAKE (`map.rs @@ -2169`) into the union's tests mod | ✓ |
| draw.rs call site + num37 walk + MouseText tail + pylon-gate widening + clamp call | TAKE | TAKE (`@@ -164` gate, `@@ -1467` comment, `@@ -1574` branch, `@@ -1928` clamp call) | ✓ |
| anchor row `map.rs\|pub fn draw_overlay` | TAKE | TAKE (`anchors.txt @@ -909`) — never delete rows | ✓ |
| docs (ANALYSIS/PORT_MAP/research/14) | fold | APPEND per keep-both, union-named sections win conflicts (their §10.2 rule) | ✓ |

**The shared ruling, restated as the mutual receipt:** the D18 draw
(`Main.cs:55548-55625` half) RE-FOLDS INTO THE F46 UNION, NOT f29 —
the style-2 keys/alpha state/clamps already live on the union
(`a352b16e`), so the draw joins them on ONE branch BEFORE f46 lands;
f29's second landing only extends the shared `MapIconInputs` struct at
the third call site. Both lanes publish the same order and the same
"never twice" icon-block rule.

**The third `MapIconInputs` call site — f29's 4 fields (re-verified
live on this tip at seal).** The f29 tip has exactly TWO
`map_layers_draw` call sites (`app/frame/draw.rs:1489` minimap,
`:2284` fullscreen), both already constructing the extended struct
(`map_layers.rs:462/465/468/471`: `map_fullscreen`, `pylon_tile`,
`screen`, `left_press`). The D18 re-fold's num37 layer walk is the
THIRD site and arrives WITHOUT the 4 fields (e18f predates the
extension); at the f29 second landing it gains, per §8.5:
`map_fullscreen: false`, `pylon_tile: (-1, -1)`
(`TeleportPylonsMapLayer.cs:45`'s fullscreen gate — the style-2 arm
draws no network lines), `screen`, `left_press: false` (the marker
click is fullscreen-only, `:83`/`:90`). The walk's `PylonClick`/
`cancel_wormhole` return halves are inert at `map_fullscreen: false` —
drop them at the call, keep the hover string for the `:56046`
MouseText tail. **The two stale f29-tip markers e18's seal names were
re-read from this tree and CONFIRMED present:** `draw.rs:1432-1434`
still says style-2 "draws nothing (out of P4 scope …)" and
`map_npc_icons.rs:647-650`'s `draw_npc_map_icons_overlay` is still
`#[allow(dead_code)]` UNWIRED. Compose consequence (both lanes agree):
the re-folded style-2 branch OVERWRITES the stale comment (never
kept), and the f29 overlay pick wires `draw_npc_map_icons_overlay` +
inserts the `:55626-55694` icon block INTO that branch exactly as it
inserts into the fullscreen/minimap arms — never a second copy of the
tile pass, the num37 walk or the hover tail. Order is load-bearing:
re-fold BEFORE the f29 landing or the icon block has no seam.

**Containment cross-check (our own re-run):** `git grep -c "fn w4_e18"`
gives 43 on the e18 seal tip vs 36 on the f46 union tip — the same
43-vs-36 their SEALED census table pins (7 e18-only fns: 4 keep-one
drops + 1 keys-witness drop + 2 draw-arm folds). e18 is fully absorbed
in the union MODULO the one owed `099beeba5` re-fold; their seal
receipt and this file's §10 qualification say the same thing.

### 11.2 f46g compose receipt — the final compose-order (f46 §10.3 read at seal)

The f46 tip `6fb5e01e8`'s §10.3 was re-read at seal: **same ruling from
the reveal side** — f46 does NOT absorb the overlay (~4 500 ln, its
own EXECUTED keep-one on the f29 tip with 34 witnesses), the compose
is TWO landings, f46 first, f29 TIP second, e17 drops without a
landing. Our §8 ruling and their §10.3 are the same instruction
published from both sides. Re-verified at seal:

- **Witness union:** `git grep -ohE "fn w4_"` sorted-unique on both
  tips: **34 on this tip, 56 on the f46 tip, ZERO shared names** —
  + the 2 D18 draw-arm witnesses §11.1 folds = **92** at the merged
  state (§8.0 step 4's ~92 made exact).
- **Merge-tree dry-run re-ran at seal:** `git merge-tree --write-tree
  --merge-base a725acb04 809d08238 6fb5e01e8` → tree
  **`feddf31781cdbb7246e60a78c9176a6b1838c051`** — the SAME tree as
  §8.1/§8.2 record; the 9 conflict hunks re-counted from the tree's
  blobs: `map.rs` ×3, `frame/draw.rs` ×2, `clicks.rs` ×1, `main.rs`
  ×1, `anchors.txt` ×1, `research/14` ×1 = **6 files / 9 hunks**, the
  §8.2 table unchanged. (At the pre-seal tip `3abf3bf43` the tree is
  `10223ccfd…` — same 6 files / 9 hunks; the delta vs `809d08238` is
  research/140 text only. The §8.2 line ranges are the `feddf3178`
  pins and stay the merger's reference.)
- **Their stale D18 row:** the f46 §10.3 family table at `6fb5e01e8`
  still reads "`mapStyle == 2` overlay DRAW … UNPORTED — D18 draw
  half (keys landed)". e18's §10.10 SEAL (a) publishes the UPDATE
  instruction (ported `099beeba5`, re-fold owed, style-0 retro is a
  phantom); **the f46 lane owns that edit — this branch never touches
  their file.** Recorded here so the merger sees the row is stale
  even if the f46 lane seals without rewriting it.
- **Their hot-file list = our §8.2/§8.3:** pan Option unification,
  prepare pan block ahead of the clamps, `set_open` pylon-clear join,
  doors-597 routed through `try_open_fullscreen_map` (+ the one
  `close_shop()` line folded into the shared helper), the C#-order
  draw wiring with the 255-loop REPLACING the union's single-head
  remnant, live-alpha args replacing the tip's hardcoded 1.0s (D17),
  `MapIconInputs` 4 fields at the union call sites (all THREE after
  §11.1), one `Coin_0..3` block + one `npc_boss_heads` field +
  `Cursor_11`, research/14 keep-both (their checklist item 6 = our
  hunk 9). No disagreement anywhere.

**FINAL COMPOSE-ORDER (the map family, published jointly by 140 §8/§11
and 84 §10.3/§10.10):**

0. **D18 re-fold onto the f46 union** — cherry-pick `099beeba5` onto
   `6fb5e01e8` per e18 §10.10 SEAL's 16-hunk table (§11.1 concordance;
   TAKE 3 fns + 2 witnesses + draw branch + gate widening + clamp call
   + anchor row + comment refreshes; SKIP helpers/decls/seeds; DROP
   key rows + keys witness + fixture/mod). Lands on the f46 branch.
1. **f46 tip → main FIRST** (MAIN-RULES ref-update merge; the whole
   reveal/input family + the re-folded style-2 draw).
2. **f29 TIP → main SECOND** (`809d08238` + `3abf3bf43` + this seal
   commit), synced onto post-f46 main; resolve the §8.2 nine hunks +
   the §8.3 semantic folds + §11.1's third-call-site 4 fields + the
   stale-comment overwrite + the `draw_npc_map_icons_overlay` wiring.
   Never the pre-rebuild commits (§9); never a separate e17 landing.
3. **Verify on the merged state:** the catalog command (judge by
   `test result: ok`), canary `main_anchors`, atlas re-measure
   (§11.3's tool now measures the composed set), ancestor + symbol
   checks. First execution of the 92-witness union.
4. **Retire:** e17 (contained, §10), e18 (absorbed modulo step 0,
   §10/§11.1), f29 + f46 after their merges; g53's map hunks were
   keep-oned (84 §10.1) — g53 retires with its own scope.

### 11.3 atlas-8131 × d35-compose receipt — the overlay sheets ARE in the 562-union

**d35 `1afd327b9` (ATLAS RETIRE phase 4, research/113 §§13-16):** the
562-name union (`tools/_d35_union_list.json` on that tip) was read and
set-compared against this tip's asset additions at seal:

- **Every overlay sheet this branch adds is in the d35 union.** The
  `a725acb04..HEAD` assets.rs diff adds exactly **109 sheet names**
  (60 `NPC_Head_*` town/critter heads + 40 `NPC_Head_Boss_*` rows
  + `MapDeath` + `Extra_173/175/176` + `Coin_0..3` +
  `UI/UI_quickicon1`; `NPC_Head_Boss_7` is a relocation of a base row
  — the measure tool skips it, net-new 108, the union credits all
  109 names to this family); the union's
  rows owned `W4-e17-map-overlay-layers`/`W4-f29-map-overlay-2` number
  exactly **109** and the two sets are EQUAL both ways (0 missing,
  0 extra — checked by name). The `Coin_0..3` rows carry the shared
  owner `W4-g12-dust-main-sites-2` (one copy in the union — the F4
  dedupe holds at the atlas level too).
- **`UI/Cursor_11` is in the union** with owners
  `W4-e18-minimap-reveal-queue`/`W4-f46-minimap-reveal-2`, 18×18,
  padded 400 px² — the f46 landing brings it, not f29; the composed
  atlas is the §8.4 measurement below.
- **No double-count with d35's budgets:** research/113's eager numbers
  (merged-504 skyline 9757 → pages 8190/1602; worst 9922 → 8160/1754;
  lazy union 7140, worst 7305, +Glow_362 → 7313) ALREADY INCLUDE the
  map-icon rows — their census table marks `NpcHead/NpcHeadBoss 81/40`
  as "EAGER named rows ✓ (e17/f29 append)". Our sheets are eager
  named UI/head rows, NOT NPC-kind sheets, so the lazy-load amendment
  (NPC rows leave `STATIC_SHEET_NAMES` for the per-world page) does
  not move them. The 8131 figure below is THIS branch's baseline
  pack, not an additional demand on top of d35's union numbers.
- **d42 `bac408be0` (research/238) NO-pack verdict confirmed, no
  conflict:** their full runtime set (HEAD static + gore + roster-43)
  overflows the landed skyline packer at **8382 px (+190 OVER 8192**,
  scenarios D/E; the delta-39 variant F is worse at 8418 — sheet
  removal is not monotone); the static-only pack fits (8021) but pins
  171 px headroom and eats the shared budget. Verdict: the 43 NPC-kind
  sheets are NOT packed on d42 — they ride d35's lazy-load wave
  (roster-43 solo page **1768 px**, §10.6 amendment). That overflow is
  the NPC-KIND demand; the map-overlay icon demand (+8 px eager,
  below) is disjoint and already inside d35's eager union. Nothing
  for f29 to pack, drop or re-measure on d42's account.

**Seal-phase re-measure (RAN, this pass):** `node
tools/atlas_e17_check.js` — baseline **8123 px** (9558 sheets) →
**8131 px** (9605 sheets, +8) → with f46's `UI/Cursor_11`: **8131 px,
9606 sheets — still OK**, margin **61 px** under the 8192 bound. The
tool now measures the Cursor_11 composition PERMANENTLY (the seal
commit extends `tools/atlas_e17_check.js` with the third measurement
— the earlier §8.4 "+Cursor_11" number was an ad-hoc variant; it is
now reproducible from the committed tree). Re-run at merge per 84
§10.3; the merger's bound check is d35's union-wide pack, not this
branch-local one.

### 11.4 The map-overlay-family bible — every piece, its home, its landing

| Piece | C# anchor | Home branch | Landing vehicle | Status at seal |
| --- | --- | --- | --- | --- |
| Reveal queue base (`MapUpdateQueue`) | `MapUpdateQueue.cs:21-108` | c35 → base `a725acb04` | already in main-line base | landed |
| `WorldSections` grid + spiral + driver, section paint, `/map` flag, statics, close button + thick cursor, minimap/overlay keys + Tab + alpha, pan state + cancels, open/close arms + cues, Load catch, mapDelay, draw-head close, draw-tail chat | 84 §10.3 family table rows 2-10 | **f46 union `6fb5e01e8`** (e18/f46/g53 folded, keep-ones executed) | compose step 1 | EXECUTED, merge pending |
| **D18 style-2 overlay DRAW** (`draw_overlay`/`clamp_overlay_axes`/`map_overlay_quad` + call site + num37 walk + hover tail + gate widening) | `Main.cs:55548-55625`, `:62231-62235` | e18 `099beeba5` → **re-fold onto f46** | compose step 0 (16-hunk recipe, §11.1) | ported, RE-FOLD OWED |
| Nine overlay draws (boss/town heads, pins/death/misc/revenge, hover names, pylon lines/click/pan) + F1-F6 + the `:55626-55694` icon block | `Main.cs:55922-56292`, `:56446-56539`, `MapOverlayDrawContext.cs`, `TeleportPylonsMapLayer.cs` | **f29 TIP** (e17 skeleton + cited fixes, 34 witnesses) | compose step 2 (§8.2/§8.3 + §11.1 third call site) | EXECUTED, merge pending |
| e17 lane | — | contained in the f29 tip's tree (12-file diff) | **DROPS — never lands** (§10) | deprecated |
| e18 lane | — | absorbed in the f46 union modulo step 0 | retires after step 1 (§10/§11.1) | SEALED (`5509cb87a`) |
| g53 map-key hunks | `Player.cs:24909-24965` | keep-oned into the union (84 §10.1) | ride step 1 | folded |
| Pan inertia / legend / grid lines / arrow rotation / zoom indicator / style-0 retro | — | PHANTOMS (three censuses + e18 seal agree) | nothing to port | closed |
| Unity-potion teleport hover/click | `Main.cs:55960-55975`, `:55998-56010` | UNPORTED seed — MP-client-only | netcode ladder, recorded | open (out of family scope) |
| Content-lost refresh | `MapRenderer.cs:363-377` | UNPORTED — D22 (no GPU-reset path) | recorded | open (out of family scope) |
| Atlas: 109 overlay sheets + `Cursor_11` | `AssetInitializer.cs:369`, `Main.cs:66551` | f29 tip + f46 union, ALL in d35's 562-union (§11.3) | eager named rows; 8131/9606 measured, margin 61 | verified |
| Witness union | — | 34 (f29) + 56 (f46) + 2 (D18 fold) = **92**, zero shared names | first run at compose step 3 | WRITTEN NOT RUN |

**Seal verdict:** the map-overlay family is fully adjudicated,
executed and cross-receipted — f46-union-FIRST → f29-TIP-SECOND with
the D18 re-fold ahead of both, e17/e18 drop/retire, the atlas demand
inside d35's union with 61 px margin, d42's lazy-load verdict
orthogonal. Nothing is owed on THIS branch; every remaining action
is merger-side (compose steps 0-4). The §9 warning list and the §10
retire instructions stand unchanged.

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
