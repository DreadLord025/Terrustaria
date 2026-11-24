# 81 — DOC-SWEEP lane 10: UI residuals (W4-g53, code-only)

Census of the six lane-10 candidates against the C# source, plus the
ANALYSIS / UI.md / research-18/19 residual mine. Four real gaps ported on
`task/W4-g53-ui-achievement-screen` (this branch, DOC-SWEEP phase); two
candidates verified already-landed. All witnesses **written, not run**
(code-only mode — no cargo/tests/graphify invoked).

## The six candidates

### 1. Achievement popup toast — LANDED (S1-E-26 wave A), verified faithful

Chain: `AchievementsHelper.HandleSpecialEvent` →
`AchievementManager.AchievementCompleted` (`AchievementManager.cs:180-187`)
→ `OnAchievementCompleted` → `InGameNotificationsTracker.AddCompleted`
(`UI/InGameNotificationsTracker.cs:75-81`, the `netMode != 2` gate) →
`InGamePopups.AchievementUnlockedPopup` (`UI/InGamePopups.cs:13-148`).

The port (`crates/tr-app/src/ui/notifications.rs`) carries the whole
lifecycle: the 300-tick timer (`:71`), the `Scale`/`Opacity` curves
(`:39-66` — the `>285` arm is the 15-tick fade-IN, the `<30` arm the
fade-out, opacity 0 below scale 0.5), the bottom-centre anchor walk
(`InGameNotificationsTracker.cs:38-55`, `screenH-40`, `PushAnchor`
`-50*opacity`, the `-100` break), the plate rect
`(MeasureString + (58,10)) * (Scale*1.1)` (`:100-102`), the `(64,109,164)`
plate at 0.75/0.5 hover (`:104`), the 64px icon at `(icon%8*66, icon/8*66)`
origin `(0,32)` scale `num*0.3` (`:107`), the border origin `(4, h/2)`
(`:108`), the `mouseTextColor` title at anchor `(1, 0.4)` (`:109`), and the
click → `OpenAchievementsAndGoto` + timer-zero + removal (`:117-142`).

**Font census (the subtle part):** the plate MEASURE uses
`FontAssets.ItemStack` (`InGamePopups.cs:101`) while the title DRAW uses
`Utils.DrawBorderString` = `FontAssets.MouseText` (`Utils.cs:2516`). The
port matches both: `text::hud_item_text_width` for the rect,
`BorderFont::Ui` for the draw (`notifications.rs:194`, `:252`). The
`DynamicSpriteFont.MeasureString` Y is `LineSpacing`
(`ReLogic.Graphics/DynamicSpriteFont.cs:279`), so the port's
`item_font.line_spacing` is the right height.

The expiry quirk is kept: `Update` never sets `ShouldBeRemoved` — an
expired popup stays in the list at opacity 0 until `Clear`
(`WorldGen.cs:6714`'s SaveAndQuit). Unported halves (recorded in the
module doc): the join-request popup (`PlayerWantsToJoinGamePopup`, social
flow) and the notifications-area draw (`DrawInIngameOptions`, `:57-73`).

### 2. In-world mouse-over hover — GAP → PORTED (`f7bb95f8`)

`Main.DrawMouseOver` (`Main.cs:42871-42988`, interface layer 39
`DrawInterface_39_MouseOver` `:44904-44908`) was unported — the c21 branch
landed the **fullscreen-map** hover (`DrawMap_FindChestName`,
`Main.cs:56631-56643`), not the in-world pass. New module
`crates/tr-app/src/ui/world_hover.rs`, wired into `draw_hud` after the
slot/PVP/housing texts:

- **Dropped item** (`:42884-42913`): the first item whose
  `GetDrawHitbox` rect (bottom-centred on `item.Bottom`) contains the 1x1
  world mouse rect prints `AffixName + " (stack)"` in its rare colour.
  The port's hitbox stand-in is the atlas icon quad (the `item.rs::draw`
  convention), falling back to the 16x16 `WorldItem` box
  (`WorldItem.cs:138`).
- **NPC** (`HoverOverNPCs`, `:42990-43114`): the first live,
  non-shimmer-transparent NPC whose frame-rect hover box contains the
  pointer prints `GivenOrTypeName` + `": life/lifeMax"` from the
  `realLife` carrier while `lifeMax > 1 && ShowLife` (`:43090-43099`).
  The special boxes (87-92's 64x64, 677's 20x20, 695/696's 60x40,
  `:43004-43015`) and the exclusions (buried 85/341/629/aiStyle-87 at
  `ai[0]==0`, 694 at `ai[3]==3`, 690/488 never, `:43018`) ride the net id.
- **The draw** is the plain `MouseTextInner` arm (`:20182-20319`): mouse
  +14, +34 X while carrying (`:20208-20211`), the `num2=40` int-truncated
  clamp (`:20236-20243`), the four ±2px shadow passes + colour pass at the
  `mouseTextColor` pulse — **no `DrawInvBG` box** (that is the
  `HoverItem.type > 0` tooltip path, `:20213-20216`).
- **Priority**: `Main.mouseText` clears at layer 24 (`:45709`), the
  carried item sets it (`:45188`), the item walk gates on `!mouseText`
  (`:42884`), the NPC text on `flag && !mouseText` (`:43087`). The port
  gates on the earlier slot/PVP/housing texts + the carried item/head.

Documented drops: the stealthed-remote-player hover (`:42914-42936`), the
sign hover (`:42940-42986`, **claim f22b** — skipped), the elder-slime
transform arm (`:43020-43045`, no kind 685), the `cursorItemIcon*`
interact-icon family (the whole `DrawInterface_40_InteractItemIcon` layer
is unported — `Player.TileInteractionsMouseOver`, `Player.cs:33587+`, is
a separate subsystem), and the talk arm's smart-interact/stinky/651/
mouseInterface gates (`:43048-43064`; the talk route itself already lives
in `world_sim::npcs::talk_to_npc`).

### 3. Buff icon right-click cancel — LANDED (S1-P-i53), verified matches

**There is no confirmation dialog in the C#.** `Main.DrawBuffIcon`'s
hover arm (`Main.cs:43329-43349`) fires `TryRemovingBuff` directly on
`Main.mouseRight && Main.mouseRightRelease` (`:43333`); `TryRemovingBuff`
(`:43370-43388`) removes the non-debuff (not 60/151), dismounts the mount
arm, hides the pet/light-pet misc visuals, and plays sound 12 (`:43382`).
The port matches: `ui/resources.rs::draw_buffs` queues the removal on the
same gate (`buff_cancel_gate`, `app/frame/draw.rs:44-48` — the
end-of-draw latch recompute `:62458-62465`, so it fires on the PRESS
frame, never held/release), `player.rs::try_removing_buff`
(`:3845-3873`) ports the removal arms 1:1, and the drain plays
`Menu_Tick` (`world_sim/player.rs:626-631`). The i53 witnesses pin the
press-edge, the alive-player drain, the pet hide and the no-RNG property.

### 4. Hotbar scroll wheel — LANDED (S1-P-i54) + gate fix (`05ff5c56`)

`Player.HandleHotbarControls` (`Player.cs:31766-31796`):
`num += ScrollWheelDelta / -120` (`:31790` — wheel-UP steps BACKWARD),
applied to the hotbar latch through `ClampHotbarOffset` (`:31895-31906`,
the while-loop wrap). The i54 port (`app/loop.rs`'s MouseWheel arm +
`hotbar.rs::clamp_hotbar_offset`) matches the direction, the wrap and the
latch-vs-live-slot distinction.

**The gap this sweep fixed:** the wheel delta is gated on
`!Main.inFancyUI && !Main.ingameOptionsWindow` (`:31788-31791`) — the
port cycled the hotbar under the bestiary/emotes overlays and the options
window. Now `App::hotbar_wheel_allowed` carries the gate (the
achievements overlay already consumed the wheel earlier in the handler).
The `HotbarPlus`/`HotbarMinus` key half (the radial hotbar,
`HotbarScrollCD`) stays unported with the gamepad layer.

### 5. Minimap zoom keys +/- — GAP → PORTED (`a9774d2a`)

The fullscreen-map keys were already ported (`update_map_input`,
`Player.cs:24898-24908`'s ×1.05/×0.95). The **not-fullscreen** block
(`Player.cs:24909-24965`) was not:

- `mapStyle == 1`: MapZoomIn/Out multiply `mapMinimapScale` by
  **1.025/0.975** (the `MinimapFrame` button multipliers,
  `MinimapFrame.cs:61-63` — NOT the fullscreen's 1.05/0.95);
  MapAlphaUp/Down step `mapMinimapAlpha` ±0.015 (`:24921-24928`).
- `mapStyle == 2`: the overlay's ×1.05/×0.95 + ±0.015
  (`:24930-24947`).
- Clamps at draw: minimap scale 0.2..3, alpha 0.01..1
  (`Main.cs:55515-55530`); overlay scale
  `screenW/worldW .. 16*RenderZoom`, alpha 0.01..1 (`:55550-55566`).
- **Tab** (`cMapStyle`, `Main.cs:67190`) cycles `Main.mapStyle`
  0→1→2→0 on the `releaseMapStyle` press latch with sound 12
  (`:24949-24965`).

`WorldMap` now carries `minimap_alpha`/`overlay_scale`/`overlay_alpha`
(the `Main.cs:66753-66755` seeds 1 / 2.5 / 0.35); the minimap tile pass
tints `Color(b×4)` (`MapRenderer.cs:438`) and the backdrop reads the live
alpha. `App::update_map_style_keys` ticks beside `update_map_input`
(`world_sim/player.rs`'s tail). The overlay DRAW itself stays out of P4
scope — the state rides for the keys.

### 6. Screenshot mode / hide-UI F11 — GAP → PORTED (`6aad8504`)

`Main.DoUpdate_F11_ToggleUI` (`Main.cs:18402-18424`): the `releaseUI`
press latch flips `Main.hideUI` + sound 12. While hidden:

- the interface layer loop stops at layer 8
  (`DrawInterface_8_CheckF11UIHideToggle`, `:46175-46183`, which also
  sets `maxQ = true` — the gfx-quality force-max arm, unported with the
  adaptive loop's `maxQ` writer);
- the world-side player chat bubbles + combat text skip (`:62334`);
- the smart-cursor highlight area returns false (`:3948-3951` — the
  port's smart-cursor highlight is unported, so nothing to gate).

The port gates `draw_hud` (the layer-1_1 emote/NPC-chat bubbles keep
drawing — they precede the check), the minimap/fullmap/toasts/
achievements-overlay/death-text/invasion-bar passes, the combat text +
entity health bars (layer 14), and the game cursor (layer 36; the OS
pointer stays hidden while focused, `FocusHelper.cs:160-163`). The
fullscreen map draws on its own branch (`:62147-62181`) — F11 never
reaches it, matching the port's `map.open` branch.

## The doc mine (ANALYSIS / UI.md / research 18-19)

The research-18/19 gap lists are framework-level (the `UIElement` tree,
`UIList` as a class, snap points, the keybinding rows) — already recorded
as deliberate immediate-mode deviations, not lane-10 residuals. The
small-mechanic residuals found and their disposition:

| Residual | Source | Disposition |
| --- | --- | --- |
| Achievement toast | `InGamePopups.cs:13-148` | LANDED S1-E-26, verified (§1) |
| In-world item/NPC hover | `Main.cs:42871-43114` | PORTED `f7bb95f8` (§2) |
| Buff right-click cancel | `Main.cs:43329-43404` | LANDED i53, verified — no confirm dialog exists (§3) |
| Hotbar wheel fancy-UI gate | `Player.cs:31788-31791` | FIXED `05ff5c56` (§4) |
| Minimap/overlay zoom+alpha keys, Tab cycle | `Player.cs:24909-24965` | PORTED `a9774d2a` (§5) |
| F11 hide-UI | `Main.cs:18402-18424`, `:46175-46183` | PORTED `6aad8504` (§6) |
| Sign hover text | `Main.cs:42940-42986` | SKIPPED — active claim f22b |
| Journey-mode UI | research 19 | SKIPPED — active claim c24b |
| Item tooltip shop/upgrade rows | `Main.cs:20448+` | SKIPPED — d85 territory |
| `cursorItemIcon` interact-icon layer | `Player.cs:33587+`, `Main.cs:45037+` | GAP, recorded — a whole subsystem (the tile-hover icons + `noThrow` arms), too large for this lane; the `no_throw` field keeps the check's shape (`app/mod.rs:647-652`) |
| `DrawNPCMapIcons2` minimap head hover | `Main.cs:56144-56220` | GAP, recorded — the minimap NPC/player head icons themselves are unported (the map-content-icon subsystem, ANALYSIS "Minimap frame and boss bar (P4)"); the hover rides them |
| Gamepad radial hotbar / `HotbarScrollCD` | `Player.cs:31769-31787` | GAP, recorded — the global gamepad layer (no `UILinkPointNavigator` in crates/, the round-2 census) |

## Commits (this phase)

- `6aad8504` — F11 hide-UI (1 witness)
- `a9774d2a` — minimap/overlay map keys + Tab cycle (2 witnesses)
- `05ff5c56` — hotbar wheel fancy-UI gate (1 witness)
- `f7bb95f8` — in-world mouse-over pass (5 witnesses)

All witnesses written, **not run** (code-only mode).
