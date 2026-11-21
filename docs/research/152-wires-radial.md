# 152 — WiresRadial: the logic half landed, the draw contract (the radial seam)

> Renumbered 135 → 152 (2026-11-21, the INDEX-registry collision —
> g90's `135-g90-persist-folds` was first-pushed; see research/151's
> header for the full note).
>
> W4-f11 · 2026-11-21 · code-only (no cargo/tests/graphify run) · source:
> `decompiled/Terraria/GameContent/UI/WiresUI.cs` (708 lines, re-read whole)
> + `Main.cs:46195-46203` (`DrawInterface_9_WireSelection`).

## 1. What landed (the self-contained logic half)

`crates/tr-app/src/ui/wires_radial.rs` — the `WiresUI.Settings` gates, the
`WiresRadial` state machine and the six-slot toggle dispatch, all pure over
a per-frame input snapshot (the `WireKiteInputs` pattern):

| C# | Port | Notes |
| --- | --- | --- |
| `Settings.DrawToolModeUI` (`:59-73`) | `draw_tool_mode_ui(held_kind)` | 3611/3625 only; the IsAir gate folds into the kind match |
| `Settings.DrawToolAllowActuators` (`:75-90`) | `draw_tool_allow_actuators(held, &mut last)` | the getter MUTATES `_lastActuatorEnabled` (2=Design, 1=Wrench) — the latch lives on `WiresRadial.last_actuator_enabled` |
| `WiresRadial` fields (`:100-106`) | `WiresRadial { position, active, on_wires_menu, line_opacity, last_actuator_enabled }` | `App.wires_radial` (ctor-init both App constructors) |
| `FlowerUpdate` (`:150-187`) | `flower_update` | the right-PRESS toggle (`mouseRight && mouseRightRelease`), the dead/cursor-stack/talk/noThrow/hover refusals, the mouseInterface exemption via `OnWiresMenu` |
| `LineUpdate` (`:114-148`) | `line_update` | the ±0.05 clamp-sign-snap ramp, the 0.75 floor while a tool is held, the cursor-icon/options zero gates |
| the `DrawFlower` click arm (`:363-508`) | `flower_clicks` | the 45-px ring geometry (the 0↔3 swap, `num3 = 4+allow`, `num4 = 11/−0.5`), the cutter's <20-px centre disc, the 19-px ring hits, first-hit-wins, the XOR toggles, the empty-press close |
| the `DrawLine` click arm (`:214-339`) | `line_clicks` | the `(screenW/2, screenH−70)` anchor, the ±(45+15)·opacity spread, the 19·opacity hit radius |
| the slot→channel switch (`:305-325`) | `slot_flag` | 0 Red / 1 Green / 2 Blue / 3 Yellow / 4 Cutter / 5 Actuator (the CURSOR-AREA's 1↔2 colour remap `:556-586` is draw-only) |

Wiring: `App::handle_wires_radial` (`app/clicks.rs`) runs in the pre-gate
click cluster after the inventory handlers (`frame/world.rs` — the C#
DrawInterface_9 sits after DrawInterface_4); a slot hover folds
`player.mouseInterface = true` into `block_mouse` (the port's one
use-suppression seam). The toggles write `App.wire_tool_mode` — the field
the Multicolor Wrench use arm + the mass-wire bill already read, so the
ToolMode channel selection is now REACHABLE in-game (the right-click
flower while holding 3611/3625).

Witnesses (written not run): `w4_f11_the_tool_mode_ui_gates_on_the_two_tools`,
`w4_f11_the_actuator_slot_latches_the_last_tool`,
`w4_f11_the_right_press_toggles_the_flower`,
`w4_f11_the_flower_click_toggles_the_channel`,
`w4_f11_the_line_opacity_ramps_and_gates`.

## 2. The DRAW contract (the remainder — NOT self-contained small)

~450 C# lines of pure sprite draw, gated on infrastructure the port does
not have yet:

1. **Textures**: `TextureAssets.WireUi[12]` (`TextureAssets.cs:613`,
   `Images/UI/WireUI_0..11`) + `TextureAssets.BuilderAcc`
   (`Images/UI/BuilderAcc`, the cursor-area dots' 6×6 source rects
   `(140,2)`/`(148,2)`/`(128,0)`). The port's `assets.rs` loads neither —
   the atlas-load wave owns them (the `WireUi` array is also read by
   `Main.cs:31665-31692`'s wire-tool cursor draws, a second consumer).
2. **The sprite seam**: every slot draws TWO centred sprites (the base
   disc `WireUi[(Cutter?8:0)+(hover?1:0)]` + the colour icon
   `WireUi[2+i]`/`[6/7]`/`[10]`) at `origin = size/2`, `scale =
   _lineOpacity` (line) or 1 (flower), tinted by the on/off/hover colour
   table (`:283-299`/`:455-471`: white when on, (200/120) hover-grey,
   (100/80) idle-grey). The port's UI draws through `SpriteRenderer` with
   its own centred-rect convention — the draw pass is a `ui::wires_radial::draw`
   sibling reading the SAME `WiresRadial` state (no logic duplication).
3. **`DrawCursorArea` (`:511-691`)**: the five/six colour dots beside the
   cursor (`MouseScreen + (10,25)`, the −20/+10 no-actuator shift), the
   1↔2 index remap, the per-channel colours ((253,58,61) red, (83,180,253)
   blue, (83,253,153) green, (253,254,83) yellow, WhiteSmoke actuator),
   the `Lerp(color, Black, 0.65)` off-dim, the 0.7-scale diamond layout
   without actuators vs the cross layout with. Pure draw over the same
   ToolMode bits.
4. **Gamepad**: `PlayerInput.UsingGamepad`/`GamepadThumbstickRight/Left`/
   `SmartCursorWanted`/`GamepadCursorAlpha` — the port has no gamepad (the
   smart-cursor precedent folds it); the flower's centre-screen anchor,
   the angle-window hit test (`AngleTowards`, `:418-422`) and the line's
   Vector2.Zero mouse all fold away. When a gamepad lands, the angle
   window is the one LOGIC piece to add (the port's `angle_towards` lives
   in `projectile.rs:14170`).

## 3. Documented folds (the logic half's deviations)

- `player.mouseInterface`/`lastMouseInterface`: no port fields — the
  caller passes `minimap_hover.is_some()` (the one documented mirror);
  the hover WRITE folds into `block_mouse`. When a unified mouseInterface
  field lands, re-point both.
- `Main.SmartInteractShowingGenuine`, `Main.HoveringOverAnNPC`,
  `player.cursorItemIconEnabled/ID`: unported UI census — the snapshot
  carries the gates' shapes, the caller passes false/0 today.
- `Main.drawingPlayerChat` → `chat.open`; `player.talkNPC != -1` →
  `talk_npc.is_some()`; `Main.ingameOptionsWindow || InGameUI.IsVisible` →
  `ingame_options.open || achievements_open || bestiary_open` (the
  `can_pause_game` census).
- The C# runs Update+Draw inside the DRAW pass (after the inventory
  draws); the port runs the logic in the pre-gate click cluster (the UI
  stays live while paused — the same convention the minimap/bestiary
  handlers use). The one-frame input lag is the port's standing
  draw/update split (ANALYSIS).

*All C# anchors re-read at the cited lines in this worktree's
`decompiled/Terraria/` checkout. No cargo/tests/graphify run (code-only
lane); the witnesses are WRITTEN NOT RUN on this branch.*
