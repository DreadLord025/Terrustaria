//! W4-g23 — the Main.cs `PlaySound` census, batch 3 of 3 (code-only).
//!
//! The lane split is line-order thirds over the **271** `SoundEngine.PlaySound`
//! call sites of `decompiled/Terraria/Main.cs` (ascending line order, first
//! `:3244`, last `:66248`): g21 owns sites 1-90 (`:3244`-`:43936`), g22 owns
//! sites 91-180 (`:44118`-`:49727`), **g23 owns sites 181-271
//! (`:49734`-`:66248`)**. All three branch from `a725acb0`. The split is the
//! W4-g21 claim (`7e8a7946`, `docs/ANALYSIS.md`).
//!
//! Two populations live in this window:
//!
//! 1. **The `DrawMenu` inline cues** (`:49734`-`:51052`, 75 sites). Every one
//!    is a menu-row activation playing legacy id 10 (open), 11 (back/close) or
//!    12 (tick/toggle). The rows and toggles themselves are ALREADY ported
//!    (`menu/pages.rs` + `menu/actions.rs`); the per-site cue seam that
//!    replaces the blanket `Menu_Tick` is **W4-g22's in-flight
//!    `Menu::pending_cue`** (uncommitted at probe time). This lane does NOT
//!    re-port that seam — it records the 75 sites as a census
//!    ([`DRAW_MENU_CUE_CENSUS`]) the merger replays against g22's `pending_cue`
//!    mapping to prove the g23 window is covered (the W4-g18 `e8`-keeps-the-
//!    single-copy precedent).
//!
//! 2. **The tail sites** (`:51276`-`:66248`, 16 sites) outside `DrawMenu`.
//!    Thunder (`:58761`) is landed (`rain.rs::thunder_step` +
//!    `ambient_sound::thunder_cue_pick`); the menu-row hover tick (`:51835`)
//!    is landed (`app/loop.rs`'s `m.hover != before`). The rest are ported
//!    here as their *pure* cue-decision halves, unwired behind witnesses (the
//!    `npc_ai_style59` precedent): the colour/volume hover ticks, the
//!    star-game hit pair, the click-pan toggle, the unity-potion map hover,
//!    the fullscreen-map close, the chat `NewText` tail and the
//!    `SelectPlayer`/password/IP cue arms.
//!
//! RNG: the only gameplay draws in this window are the star-game hit pair
//! (`:63168` case 38 then `:63169` case 37). Both ride `Main.rand` in the
//! `DrawStar` statement order (the sky draw pass), slot first then pitch, and
//! only past the audibility gate (`LegacySoundPlayer.cs:369` —
//! `soundVolume == 0` returns null for type > 35). Every other g23 site is a
//! draw-free legacy id (10/11/12) or the already-landed case-43 thunder. The
//! hover ticks and cue arms draw NOTHING.

/// One `DrawMenu` inline-cue site: the C# line, the `menuMode` branch it sits
/// in, the legacy id it plays, and whether the row/toggle is already ported.
/// The cue ids are 10 = `Menu_Open`, 11 = `Menu_Close`, 12 = `Menu_Tick`
/// (`LegacySoundPlayer.cs:750-782`). `ported` is true when `menu/pages.rs`
/// carries the row (the activation exists); the cue itself rides g22's
/// `pending_cue` seam, not this lane.
pub struct DrawMenuCueSite {
    pub line: u32,
    pub menu_mode: i32,
    pub cue: u32,
    pub ported: bool,
}

/// The 75 `DrawMenu` sites of the g23 window, in C# line order
/// (`Main.cs:49734-51052`). Census only — the rows are landed, the cue seam is
/// g22's. The merger checks every row here resolves to the same id through
/// `Menu::pending_cue` once g22 lands.
pub const DRAW_MENU_CUE_CENSUS: &[DrawMenuCueSite] = &[
    // menuMode 11 — the Settings hub (`Main.cs:49706-49774`).
    DrawMenuCueSite { line: 49734, menu_mode: 11, cue: 10, ported: true }, // -> 1111 Video
    DrawMenuCueSite { line: 49741, menu_mode: 11, cue: 11, ported: true }, // -> 26 Volume
    DrawMenuCueSite { line: 49748, menu_mode: 11, cue: 10, ported: true }, // -> 1125 Cursor
    DrawMenuCueSite { line: 49755, menu_mode: 11, cue: 10, ported: true }, // -> 1127 Controls
    DrawMenuCueSite { line: 49762, menu_mode: 11, cue: 10, ported: true }, // -> 1213 Language
    DrawMenuCueSite { line: 49770, menu_mode: 11, cue: 11, ported: true }, // Back -> 0 + SaveSettings
    // menuMode 112 — General (`:49775-49870`).
    DrawMenuCueSite { line: 49796, menu_mode: 112, cue: 12, ported: true }, // autoSave
    DrawMenuCueSite { line: 49817, menu_mode: 112, cue: 12, ported: true }, // autoPause
    DrawMenuCueSite { line: 49838, menu_mode: 112, cue: 12, ported: true }, // mapEnabled
    DrawMenuCueSite { line: 49852, menu_mode: 112, cue: 12, ported: true }, // HidePassword
    DrawMenuCueSite { line: 49859, menu_mode: 112, cue: 12, ported: true }, // PlayWhenUnfocused
    DrawMenuCueSite { line: 49868, menu_mode: 112, cue: 11, ported: true }, // Back -> 11
    // menuMode 1112 — Interface (`:49871-49987`).
    DrawMenuCueSite { line: 49892, menu_mode: 1112, cue: 12, ported: true }, // showItemText
    DrawMenuCueSite { line: 49906, menu_mode: 1112, cue: 12, ported: true }, // invasionProgressMode
    DrawMenuCueSite { line: 49917, menu_mode: 1112, cue: 12, ported: true }, // placementPreview
    DrawMenuCueSite { line: 49924, menu_mode: 1112, cue: 12, ported: true }, // HighlightNewItems
    DrawMenuCueSite { line: 49931, menu_mode: 1112, cue: 12, ported: true }, // MouseShowBuildingGrid
    DrawMenuCueSite { line: 49938, menu_mode: 1112, cue: 12, ported: true }, // GamepadDisableInstructions
    DrawMenuCueSite { line: 49963, menu_mode: 1112, cue: 12, ported: true }, // CycleNPCPortraitMode
    DrawMenuCueSite { line: 49976, menu_mode: 1112, cue: 12, ported: true }, // FlashyEffectsInterface
    DrawMenuCueSite { line: 49985, menu_mode: 1112, cue: 11, ported: true }, // Back -> 11
    // menuMode 1111 — Video (`:49988-50096`).
    DrawMenuCueSite { line: 49995, menu_mode: 1111, cue: 10, ported: true }, // -> 111 Resolution
    DrawMenuCueSite { line: 50002, menu_mode: 1111, cue: 11, ported: true }, // -> 28 Parallax
    DrawMenuCueSite { line: 50009, menu_mode: 1111, cue: 12, ported: true }, // CycleFrameSkipMode
    DrawMenuCueSite { line: 50016, menu_mode: 1111, cue: 12, ported: true }, // NextLightMode
    DrawMenuCueSite { line: 50037, menu_mode: 1111, cue: 12, ported: true }, // qaStyle
    DrawMenuCueSite { line: 50047, menu_mode: 1111, cue: 12, ported: true }, // BackgroundEnabled
    DrawMenuCueSite { line: 50054, menu_mode: 1111, cue: 12, ported: true }, // ChildSafety
    DrawMenuCueSite { line: 50061, menu_mode: 1111, cue: 12, ported: true }, // MinersWobble
    DrawMenuCueSite { line: 50068, menu_mode: 1111, cue: 12, ported: true }, // TilesSwayInWind
    DrawMenuCueSite { line: 50075, menu_mode: 1111, cue: 11, ported: true }, // -> 2008 Effects
    DrawMenuCueSite { line: 50085, menu_mode: 1111, cue: 11, ported: true }, // Back -> 11 + SaveSettings
    // menuMode 2008 — Effects (`:50097-50168`).
    DrawMenuCueSite { line: 50116, menu_mode: 2008, cue: 12, ported: true }, // UseStormEffects
    DrawMenuCueSite { line: 50123, menu_mode: 2008, cue: 12, ported: true }, // ForegroundSunlightEffects
    DrawMenuCueSite { line: 50130, menu_mode: 2008, cue: 12, ported: true }, // UseHeatDistortion
    DrawMenuCueSite { line: 50143, menu_mode: 2008, cue: 12, ported: true }, // WaveQuality
    DrawMenuCueSite { line: 50150, menu_mode: 2008, cue: 12, ported: true }, // UseScreenShake
    DrawMenuCueSite { line: 50157, menu_mode: 2008, cue: 12, ported: true }, // FlashyEffectsWorld
    DrawMenuCueSite { line: 50164, menu_mode: 2008, cue: 11, ported: true }, // Back -> 1111
    // menuMode 111 — Resolution (`:50169-50241`).
    DrawMenuCueSite { line: 50181, menu_mode: 111, cue: 12, ported: true }, // resolution cycle
    DrawMenuCueSite { line: 50201, menu_mode: 111, cue: 12, ported: true }, // PendingBorderlessState
    DrawMenuCueSite { line: 50225, menu_mode: 111, cue: 11, ported: true }, // Apply -> 1111
    DrawMenuCueSite { line: 50238, menu_mode: 111, cue: 11, ported: true }, // Back -> 1111
    // menuMode 1125 — Cursor (`:50242-50317`).
    DrawMenuCueSite { line: 50256, menu_mode: 1125, cue: 10, ported: true }, // -> 25 MouseColor
    DrawMenuCueSite { line: 50265, menu_mode: 1125, cue: 10, ported: true }, // -> 252 BorderColor
    DrawMenuCueSite { line: 50274, menu_mode: 1125, cue: 12, ported: true }, // cSmartCursorModeIsToggle
    DrawMenuCueSite { line: 50281, menu_mode: 1125, cue: 12, ported: true }, // SmartAxeAfterPickaxe
    DrawMenuCueSite { line: 50288, menu_mode: 1125, cue: 12, ported: true }, // SmartBlocksEnabled
    DrawMenuCueSite { line: 50306, menu_mode: 1125, cue: 12, ported: true }, // LockOnHelper.CycleUseModes
    DrawMenuCueSite { line: 50315, menu_mode: 1125, cue: 11, ported: true }, // Back -> 11
    // menuMode 25 / 252 / 26 / 28 — the colour/parallax Back rows.
    DrawMenuCueSite { line: 50337, menu_mode: 25, cue: 11, ported: true }, // Back -> 1125
    DrawMenuCueSite { line: 50359, menu_mode: 252, cue: 11, ported: true }, // Back -> 1125
    DrawMenuCueSite { line: 50378, menu_mode: 26, cue: 11, ported: true }, // Back -> 11
    DrawMenuCueSite { line: 50398, menu_mode: 28, cue: 11, ported: true }, // Back -> 1111
    // menuMode 272727 / 27 — the map/gameplay keybind lists.
    DrawMenuCueSite { line: 50447, menu_mode: 272727, cue: 11, ported: true }, // Back -> 11
    DrawMenuCueSite { line: 50458, menu_mode: 272727, cue: 11, ported: true }, // ResetKeys (map)
    DrawMenuCueSite { line: 50557, menu_mode: 27, cue: 11, ported: true }, // Back -> 11
    DrawMenuCueSite { line: 50563, menu_mode: 27, cue: 11, ported: true }, // ResetKeys (gameplay)
    // menuMode 1127 — Controls (`:50638-50698`).
    DrawMenuCueSite { line: 50652, menu_mode: 1127, cue: 12, ported: true }, // ReversedUpDownArmorSetBonuses
    DrawMenuCueSite { line: 50666, menu_mode: 1127, cue: 12, ported: true }, // quick-trash cycle
    DrawMenuCueSite { line: 50686, menu_mode: 1127, cue: 10, ported: true }, // -> 888 ManageControls
    DrawMenuCueSite { line: 50696, menu_mode: 1127, cue: 11, ported: true }, // Back -> 11
    // menuMode 12 — Multiplayer (`:50699-50736`).
    DrawMenuCueSite { line: 50711, menu_mode: 12, cue: 10, ported: true }, // join -> 1
    DrawMenuCueSite { line: 50718, menu_mode: 12, cue: 10, ported: true }, // host -> 1
    DrawMenuCueSite { line: 50726, menu_mode: 12, cue: 10, ported: true }, // Steam join
    DrawMenuCueSite { line: 50732, menu_mode: 12, cue: 11, ported: true }, // Back -> 0
    // menuMode 13 / 131 — the join IP/port input screens.
    DrawMenuCueSite { line: 50783, menu_mode: 13, cue: 11, ported: true }, // gamepad Back -> 1
    DrawMenuCueSite { line: 50796, menu_mode: 13, cue: 12, ported: true }, // getIP text change
    DrawMenuCueSite { line: 50862, menu_mode: 13, cue: 11, ported: true }, // keyboard Back -> 1
    DrawMenuCueSite { line: 50901, menu_mode: 131, cue: 12, ported: true }, // getPort text change
    DrawMenuCueSite { line: 50957, menu_mode: 131, cue: 11, ported: true }, // Back -> 1
    // menuMode 16 / 1000000 / 1000001 — world-size + the status/rejection tails.
    // NOT legacy `Page`s: the port replaces 16 with the `ui/world_creation.rs`
    // size row and 1000000/1000001 with the front-screen rejection notice
    // (`app/flow.rs:379-384`), so these rows are unported in the legacy menu
    // (`ported: false`); the cues ride the front-end seam instead.
    DrawMenuCueSite { line: 50984, menu_mode: 16, cue: 11, ported: false }, // Back -> 6
    DrawMenuCueSite { line: 51005, menu_mode: 16, cue: 10, ported: false }, // world size -> -7
    DrawMenuCueSite { line: 51020, menu_mode: 1000000, cue: 11, ported: false }, // Back -> 6
    DrawMenuCueSite { line: 51052, menu_mode: 1000001, cue: 11, ported: false }, // Back -> 0 (no ExitAction)
];

/// Looks up one census site by its C# line. `None` for a line outside the
/// g23 `DrawMenu` window (the merger uses this to assert coverage).
pub fn draw_menu_cue(line: u32) -> Option<&'static DrawMenuCueSite> {
    DRAW_MENU_CUE_CENSUS.iter().find(|s| s.line == line)
}

/// The colour-screen focus-change tick (`Main.cs:51274-51277`): after the four
/// HSL bars run, `if (num51 != Main.focusColor) SoundEngine.PlaySound(12)`.
/// `num51` is the focus captured at the block head; the tick fires once when
/// the focused slider/bar changed this frame (enter AND leave). Draw-free
/// legacy id 12. **W4-g23: WIRED** — `menu/mod.rs::draw_color_screen_chrome`
/// runs the capture/reset (`:51117-51118`), maps the highlight flags to the
/// focus ids (`:51149`/`:51181`/`:51218`/`:51255`) and queues through
/// `Menu::pending_ticks`; the frame loop drains after `Menu::draw`.
pub fn color_focus_tick(focus_before: i32, focus_after: i32) -> bool {
    focus_before != focus_after // `:51274`
}

/// The colour +/- button hover tick (`Main.cs:51386-51389`): inside a button's
/// hover rect, `if (Main.focusColor != (num70 + 1) * (num71 + 10))
/// SoundEngine.PlaySound(12)` before `focusColor` is set to that id. `num70`
/// is the row (0 = minus, 1 = plus), `num71` the channel (0 R, 1 G, 2 B), so
/// the ids are 10/11/12 (minus) and 20/21/22 (plus). Fires once on the
/// hover-enter edge. Draw-free id 12.
///
/// **W4-g23 wiring census: DEAD ARM — stays unwired.** The RGB block's
/// `else if (flag)` at `:51284` follows the HSL block's `if (flag)` at
/// `:51110` — the same `flag`, so the branch is unreachable in vanilla
/// (the `:51285-51477` RGB labels/buttons/`colorDelay` machine never runs).
/// The port must NOT wire this cue: there is no live C# call position.
pub fn color_button_hover_tick(focus_color: i32, row: i32, channel: i32) -> bool {
    let id = (row + 1) * (channel + 10); // `:51386`/`:51390`
    focus_color != id
}

/// The volume value-bar hover tick (`Main.cs:51573-51576`): after the
/// three bars run and `rightLock` latches, `if (IngameOptions.rightHover !=
/// rightHover) SoundEngine.PlaySound(12)`. `right_hover_before` is the
/// persistent static captured at the block head (`:51527`, `-1` when none);
/// `right_hover_after` is this frame's hovered bar. Fires once when the
/// hovered bar changed — ENTER, bar-to-bar moves AND LEAVE (the comparison
/// is `!=`, not `!= -1`). Draw-free id 12.
///
/// **W4-g23 wiring census correction:** the parallax block (`:51578-51653`)
/// has NO `PlaySound` of its own — the earlier "same shape covers parallax"
/// note was wrong. Only the menuMode 26 volume block ticks. Wired in
/// `menu/mod.rs::volume_drag_pass` (the `Menu::pending_ticks` drain).
pub fn value_bar_hover_tick(right_hover_before: i32, right_hover_after: i32) -> bool {
    right_hover_before != right_hover_after // `:51573`
}

/// `Main.CanceledGivingServerPassword` (`Main.cs:51950-51956`): the join
/// password screen's cancel plays id 11 (`:51952`) then drops to menuMode 0
/// and disconnects. **W4-g23: WIRED** — `menu/actions.rs`'s `Act::Nav` arm
/// queues the id on the JoinPassword Back row (`:48592-48596`,
/// `selectedMenu == 3`); the port's Back returns to the port screen (the
/// documented plan deviation, `menu/pages.rs`), the cue id is the C#'s.
pub fn canceled_giving_server_password_cue() -> u32 {
    11 // `:51952`
}

/// `Main.OnSubmitServerIP` (`Main.cs:52056-52062`): submitting the join IP
/// plays id 12 (`:52059`) then opens menuMode 131. **W4-g23: WIRED** — both
/// the Accept row (`menu/actions.rs`'s `Act::Nav` arm, `:50865-50868`) and
/// the Enter path (`Menu::submit_join_ip`) queue the id; the frame drain
/// plays it. This is a tail-site call position, NOT one of the 75 DrawMenu
/// census rows — it does not ride g22's seam.
///
/// The sibling text-change ticks (`:50796` getIP, `:50901` getPort,
/// `:48551`/`:48675` the password screens — `GetInputText`'s per-frame
/// string compare) are DrawMenu-inline sites in g22's window boundary
/// census; they ride g22's seam, not this lane.
pub fn on_submit_server_ip_cue() -> u32 {
    12 // `:52059`
}

/// The fullscreen-map unity-potion hover tick (`Main.cs:55960-55967`): while
/// the pointer is over a same-team player's head icon and the local player
/// holds a Unity Potion (netMode 1, not already hovering, wormhole not
/// cancelled), `if (!this.unityMouseOver) SoundEngine.PlaySound(12)` then
/// `unityMouseOver = true`. Fires once on the hover-enter edge. `eligible`
/// folds the team/potion/netMode/wormhole gates (`:55960`); `was_hovering` is
/// `unityMouseOver`. Draw-free id 12.
///
/// **W4-g23 wiring census: DEFERRED — no live consumer.** The port's
/// fullscreen map draws only the LOCAL player's head icon
/// (`app/frame/draw.rs`'s `draw_player_map_icon`); the remote-player head
/// loop (`Main.cs:55917-55976`), `HasUnityPotion` and `cancelWormHole` are
/// unported. Wiring the cue would invent a call position; the machine
/// stays unwired behind witnesses until the remote heads land.
pub fn unity_map_hover_tick(eligible: bool, was_hovering: bool) -> bool {
    eligible && !was_hovering // `:55960` + `:55963`
}

/// The fullscreen-map close-icon click (`Main.cs:56024-56033`): the bottom-left
/// `MapIcon` button, on `mouseLeft && mouseLeftRelease`, plays id 10
/// (`:56031`) then `mapFullscreen = false`. `hovered` is the inclusive rect
/// test (`:56024`), `click_edge` the press. Returns the cue id when the close
/// fires.
///
/// **W4-g23 wiring census: DEFERRED — no live consumer.** The port's
/// fullscreen map has no close icon (the `MapIcon[num87]` button at
/// `(10, screenHeight - 40)` is undrawn); the port closes via Escape/`M`
/// (`app/loop.rs`), whose C# arms (`Player.ToggleInv`/`TryOpeningFullscreenMap`)
/// play NO sound — wiring 10 there would invent a cue. Stays unwired until
/// the icon lands.
pub fn fullscreen_map_close_cue(hovered: bool, click_edge: bool) -> Option<u32> {
    if hovered && click_edge {
        Some(10) // `:56031`
    } else {
        None
    }
}

/// The click-pan control toggle (`Main.cs:62796-62800`, inside
/// `GetPlayerControlledCameraPan`): when the pan button mode is `Click`, the
/// player can toggle pan-with-interact, and the right button just became true
/// (`flag`), `SoundEngine.PlaySound(12)` then `hasClickPanOn = !hasClickPanOn`.
/// Returns the new `hasClickPanOn` when the toggle fires (cue 12), `None`
/// otherwise. `mode_is_click` folds `buttonControlMode == Click`,
/// `can_toggle` is `CanTogglePanControlModeWithInteract()`, `right_just_pressed`
/// is the `Utils.JustBecameTrue(Main.mouseRight, ...)` edge (`:62752`).
///
/// **W4-g23 wiring census: DEFERRED — no live consumer.** The port's
/// `world_sim/camera.rs::update_camera_pan` ports the `UpdateCameraPan`
/// walk (`:62680-62720`) but not the `GetPlayerControlledCameraPan` source
/// (`:62748-62832`): `PlayerInput.CurrentProfile.PanControl_Keyboard`,
/// `hasClickPanOn`, `GetPanControlValues` (the scope/binocular pan factors)
/// and the `_panControlKeyAvailable` latch are unported. The vanilla
/// default mode is `Hold` (0), so the Click arm is unreachable without the
/// profile UI anyway. Stays unwired behind witnesses.
pub fn click_pan_toggle(
    mode_is_click: bool,
    can_toggle: bool,
    right_just_pressed: bool,
    has_click_pan_on: bool,
) -> Option<bool> {
    if mode_is_click && can_toggle && right_just_pressed {
        Some(!has_click_pan_on) // `:62799`, cue 12 at `:62798`
    } else {
        None
    }
}

/// `Main.NewText` / `NewTextMultiline`'s cue tail (`Main.cs:64632-64642`):
/// both overloads push the line to `chatMonitor` then `SoundEngine.PlaySound(12)`
/// (`:64635` / `:64641`). **W4-g23: WIRED** — centralized in
/// `chat.rs::ChatMonitor::new_text` (the `pending_cues` queue); the frame
/// loop drains once per frame (`app/frame.rs`) and the achievement
/// subscriber drains its own lines inline (`app/mod.rs`), so every append
/// plays exactly one 12, in append order.
pub fn new_text_cue() -> u32 {
    12 // `:64635` / `:64641`
}

/// `Main.SelectPlayer`'s cue arms (`Main.cs:66208-66251`): with no pending
/// callback, the multiplayer branch plays id 10 (`:66224`) and the
/// single-player tail plays id 10 (`:66248`). Both are the constant 10.
/// **W4-g23: LANDED-BY-EQUIVALENT** — the port plays `Menu_Open` on the
/// front-screen `CharPlay` click (`app/flow.rs:1269`), the port's
/// `SelectPlayer` call position; no new wiring needed. Recorded for the
/// census. `multiplayer` selects the branch (both draw nothing, both 10).
pub fn select_player_cue(multiplayer: bool) -> u32 {
    let _ = multiplayer; // both arms play 10 (`:66224` / `:66248`)
    10
}

/// One star-game hit's case-38 (coin) decisions
/// (`LegacySoundPlayer.cs:1337-1350`): `Main.starGame` cuts the volume ×0.15
/// (`:1339-1342`), `Main.rand.Next(5)` (`:1343`) picks the `SoundCoin[0..4]`
/// slot, `Main.rand.Next(-40, 41) * 0.002` (`:1347`) rolls the pitch — TWO
/// draws, slot first. This is the SAME case the landed
/// `sound::coin_pickup_case38` models; the star hit reaches it through
/// `PlaySound(38)` (`Main.cs:63168`). Re-exported as a thin wrapper so the
/// star pair reads as one unit; the draw order is slot then pitch.
pub fn star_hit_case38(rng: &mut dyn FnMut(i32, i32) -> i32) -> (i32, f32, f32) {
    crate::sound::coin_pickup_case38(true, rng) // starGame is always on here
}

/// One star-game hit's case-37 (item) decisions
/// (`LegacySoundPlayer.cs:1326-1335`): `Main.rand.Next(57, 59)` (`:1328`)
/// picks the `SoundItem[57..58]` slot, the volume is cut ×0.15 while
/// `Main.starGame` (`:1329` — the star hit always has it on), then
/// `Main.rand.Next(-40, 41) * 0.01` (`:1333`) rolls the pitch — TWO draws,
/// slot first. NOT the case-2 item family (`sound::legacy_family_pitch`'s
/// `Item(_)` default is `Next(-6,7)*0.01`); case 37 is its own arm with the
/// wider ±0.40 window at ×0.01. The caller gates on audibility BEFORE drawing.
pub fn star_hit_case37(rng: &mut dyn FnMut(i32, i32) -> i32) -> (i32, f32, f32) {
    let slot = rng(57, 59); // `:1328`
    let tweak = 0.15; // `:1329`, starGame on
    let pitch = rng(-40, 41) as f32 * 0.01; // `:1333`
    (slot, tweak, pitch)
}

/// The star-game hit's `fallSpeed` reflection (`Main.cs:63158-63167`): when
/// `Main.starGame`, the star is falling (`fallSpeed.Y > 0`) and the pointer is
/// within 70 px, `starsHit++`, then `fallSpeed = (starPos - mouse).Normalize()
/// * 10`, and if the result still points down (`Y > 0`) it is flipped up
/// (`Y *= -1`). Returns the reflected velocity; the caller bumps `starsHit`
/// and plays the case-38 then case-37 pair (`:63168-63169`). `within_70` is
/// the `Vector2.Distance(mouse, starPos) < 70f` gate (`:63158`).
///
/// **W4-g23 wiring census: DEFERRED — no live consumer.** `Main.starGame`
/// is set only by `WorldGen.Reset`'s everything-seed arm
/// (`WorldGen.cs:11212-11215`) and cleared outside worldgen
/// (`Main.cs:62899-62902`); the port's sky pass (`render/sky.rs`) draws
/// static twinkling stars with no `Star.falling`/`fallSpeed`/`Update`
/// (`Star.cs:165-176`) and no `starsHit`. The hit pair's exact RNG slots
/// (case 38 then 37, audibility-gated, bare `Main.rand`) are pinned by the
/// witnesses below; the wiring lands with the falling-star machine.
pub fn star_hit_reflect(
    star_pos: [f32; 2],
    mouse: [f32; 2],
    fall_speed_y: f32,
    within_70: bool,
    star_game: bool,
) -> Option<[f32; 2]> {
    if !(star_game && fall_speed_y > 0.0 && within_70) {
        return None; // `:63158`
    }
    let dx = star_pos[0] - mouse[0]; // `:63161`
    let dy = star_pos[1] - mouse[1];
    let len = (dx * dx + dy * dy).sqrt(); // `:63162` Normalize
    // A zero-length vector normalizes to zero in XNA; guard the divide.
    let (mut nx, mut ny) = if len > 0.0 { (dx / len, dy / len) } else { (0.0, 0.0) };
    nx *= 10.0; // `:63163`
    ny *= 10.0;
    if ny > 0.0 {
        ny *= -1.0; // `:63164-63167`
    }
    Some([nx, ny])
}

/// The full star-game hit sound pair, in C# statement order
/// (`Main.cs:63168-63169`): case 38 FIRST, then case 37. Both draw on
/// `Main.rand` (the `DrawStar` sky pass), slot then pitch, and only when
/// audible (`sound_volume > 0`, the `LegacySoundPlayer.cs:369` guard for
/// type > 35). Returns `None` when silent (no draws), else the two
/// `(slot, volume_tweak, pitch)` results in play order. The four draws are
/// `[Next(5), Next(-40,41)]` (38) then `[Next(57,59), Next(-40,41)]` (37).
pub fn star_hit_sound_pair(
    sound_volume: f32,
    rng: &mut dyn FnMut(i32, i32) -> i32,
) -> Option<((i32, f32, f32), (i32, f32, f32))> {
    if sound_volume <= 0.0 {
        return None; // `:369` — type 38/37 > 35, silent returns null, NO draws
    }
    let c38 = star_hit_case38(rng); // `:63168`
    let c37 = star_hit_case37(rng); // `:63169`
    Some((c38, c37))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- the DrawMenu census (Main.cs:49734-51052) -----------------------

    #[test]
    fn g23_the_census_holds_the_75_draw_menu_sites_in_line_order() {
        assert_eq!(DRAW_MENU_CUE_CENSUS.len(), 75);
        // Strictly ascending C# line order, first :49734, last :51052.
        assert_eq!(DRAW_MENU_CUE_CENSUS.first().unwrap().line, 49734);
        assert_eq!(DRAW_MENU_CUE_CENSUS.last().unwrap().line, 51052);
        for w in DRAW_MENU_CUE_CENSUS.windows(2) {
            assert!(w[0].line < w[1].line, "line order");
        }
        // Every cue is a draw-free menu id (10/11/12).
        assert!(DRAW_MENU_CUE_CENSUS.iter().all(|s| matches!(s.cue, 10 | 11 | 12)));
    }

    #[test]
    fn g23_the_census_ids_match_the_csharp_arms() {
        // Spot-check the id split against the C# branches: the Settings hub
        // opens Video with 10 (:49734) but backs to Volume with 11 (:49741).
        assert_eq!(draw_menu_cue(49734).unwrap().cue, 10);
        assert_eq!(draw_menu_cue(49741).unwrap().cue, 11);
        // The General toggles all tick 12 (:49796 autoSave).
        assert_eq!(draw_menu_cue(49796).unwrap().cue, 12);
        // The Video Effects open is 11 (:50075), not 10 — the C# plays the
        // close cue on that forward hop.
        assert_eq!(draw_menu_cue(50075).unwrap().cue, 11);
        // The Controls ManageControls open is 10 (:50686).
        assert_eq!(draw_menu_cue(50686).unwrap().cue, 10);
        // A line outside the window resolves to None.
        assert!(draw_menu_cue(49727).is_none()); // g22's last site
        assert!(draw_menu_cue(51276).is_none()); // a tail site, not DrawMenu
    }

    #[test]
    fn g23_the_world_size_and_status_rows_are_unported_legacy() {
        // menuMode 16/1000000/1000001 have no legacy Page (the front-end
        // replaces them), so the census marks them unported.
        for line in [50984, 51005, 51020, 51052] {
            assert!(!draw_menu_cue(line).unwrap().ported, "line {line}");
        }
        // The settings rows ARE ported.
        assert!(draw_menu_cue(49796).unwrap().ported);
        // Exactly the four front-end rows are unported.
        assert_eq!(DRAW_MENU_CUE_CENSUS.iter().filter(|s| !s.ported).count(), 4);
    }

    // ---- colour/volume hover ticks (Main.cs:51276/51388/51575) -----------

    #[test]
    fn g23_the_color_focus_tick_fires_on_the_change_edge() {
        assert!(color_focus_tick(0, 1)); // :51274 num51 != focusColor
        assert!(!color_focus_tick(2, 2)); // unchanged, no tick
        assert!(color_focus_tick(-1, 0)); // first focus
    }

    #[test]
    fn g23_the_color_button_ids_are_row_times_channel_plus_ten() {
        // :51386/:51390 — (num70+1)*(num71+10): minus row 10/11/12, plus
        // row 20/21/22. The tick fires only when the id differs.
        assert!(color_button_hover_tick(0, 0, 0)); // id 10, was 0
        assert!(!color_button_hover_tick(10, 0, 0)); // already focused 10
        assert!(color_button_hover_tick(10, 1, 2)); // id 22, was 10
        assert!(!color_button_hover_tick(22, 1, 2)); // id 22 unchanged
    }

    #[test]
    fn g23_the_value_bar_hover_tick_fires_on_the_change_edge() {
        assert!(value_bar_hover_tick(-1, 0)); // :51573 first hover
        assert!(!value_bar_hover_tick(1, 1)); // same bar
        assert!(value_bar_hover_tick(0, 2)); // moved bars
        assert!(value_bar_hover_tick(1, -1)); // left the bars
    }

    // ---- password/IP cue arms (Main.cs:51952/52059) ----------------------

    #[test]
    fn g23_the_password_cancel_closes_and_the_ip_submit_ticks() {
        assert_eq!(canceled_giving_server_password_cue(), 11); // :51952
        assert_eq!(on_submit_server_ip_cue(), 12); // :52059
    }

    // ---- unity-map hover + fullscreen close (Main.cs:55965/56031) --------

    #[test]
    fn g23_the_unity_hover_ticks_once_on_entry() {
        assert!(unity_map_hover_tick(true, false)); // :55963 enter edge
        assert!(!unity_map_hover_tick(true, true)); // already hovering
        assert!(!unity_map_hover_tick(false, false)); // not eligible
        assert!(!unity_map_hover_tick(false, true));
    }

    #[test]
    fn g23_the_fullscreen_close_needs_hover_and_the_click_edge() {
        assert_eq!(fullscreen_map_close_cue(true, true), Some(10)); // :56031
        assert_eq!(fullscreen_map_close_cue(true, false), None); // no press
        assert_eq!(fullscreen_map_close_cue(false, true), None); // off-icon
    }

    // ---- click-pan toggle (Main.cs:62798) --------------------------------

    #[test]
    fn g23_the_click_pan_toggles_only_on_the_right_press_edge() {
        // mode Click + can toggle + right just pressed -> flip, cue 12.
        assert_eq!(click_pan_toggle(true, true, true, false), Some(true));
        assert_eq!(click_pan_toggle(true, true, true, true), Some(false));
        // No edge, no toggle.
        assert_eq!(click_pan_toggle(true, true, false, false), None);
        // Not Click mode, or cannot toggle.
        assert_eq!(click_pan_toggle(false, true, true, false), None);
        assert_eq!(click_pan_toggle(true, false, true, false), None);
    }

    // ---- NewText + SelectPlayer (Main.cs:64635/66224/66248) --------------

    #[test]
    fn g23_the_new_text_tail_ticks_and_select_player_opens() {
        assert_eq!(new_text_cue(), 12); // :64635/:64641
        assert_eq!(select_player_cue(true), 10); // :66224 multiplayer
        assert_eq!(select_player_cue(false), 10); // :66248 single-player
    }

    // ---- star-game hit pair (Main.cs:63168-63169) ------------------------

    #[test]
    fn g23_the_star_hit_case38_draws_slot_then_pitch() {
        let mut draws = Vec::new();
        let (slot, tweak, pitch) = star_hit_case38(&mut |lo, hi| {
            draws.push((lo, hi));
            if lo == 0 { 3 } else { 20 }
        });
        // starGame cuts the volume x0.15; slot Next(5), pitch Next(-40,41)*0.002.
        assert_eq!(slot, 3);
        assert!((tweak - 0.15).abs() < 1e-6);
        assert!((pitch - 20.0 * 0.002).abs() < 1e-6);
        assert_eq!(draws, vec![(0, 5), (-40, 41)]); // slot FIRST
    }

    #[test]
    fn g23_the_star_hit_case37_draws_slot_then_pitch_wider_window() {
        let mut draws = Vec::new();
        let (slot, tweak, pitch) = star_hit_case37(&mut |lo, hi| {
            draws.push((lo, hi));
            if lo == 57 { 58 } else { -40 }
        });
        // slot Next(57,59), pitch Next(-40,41)*0.01 (NOT the case-2 *0.002).
        assert_eq!(slot, 58);
        assert!((tweak - 0.15).abs() < 1e-6);
        assert!((pitch - (-40.0 * 0.01)).abs() < 1e-6);
        assert_eq!(draws, vec![(57, 59), (-40, 41)]);
    }

    #[test]
    fn g23_the_star_pair_plays_38_before_37_four_draws_in_order() {
        let mut draws = Vec::new();
        let pair = star_hit_sound_pair(1.0, &mut |lo, hi| {
            draws.push((lo, hi));
            0
        });
        assert!(pair.is_some());
        // case 38's two draws THEN case 37's two — the C# statement order.
        assert_eq!(draws, vec![(0, 5), (-40, 41), (57, 59), (-40, 41)]);
    }

    #[test]
    fn g23_the_star_pair_is_silent_and_draw_free_at_zero_volume() {
        let mut draws = Vec::new();
        let pair = star_hit_sound_pair(0.0, &mut |lo, hi| {
            draws.push((lo, hi));
            0
        });
        // :369 — type 38/37 > 35, soundVolume 0 returns null BEFORE any draw.
        assert!(pair.is_none());
        assert!(draws.is_empty(), "no gameplay draws while silent");
    }

    #[test]
    fn g23_the_star_reflect_normalizes_times_ten_and_flips_down() {
        // Star at (100,100), mouse at (100,170): delta (0,-70), normalize
        // (0,-1), x10 -> (0,-10). Already up, no flip.
        let v = star_hit_reflect([100.0, 100.0], [100.0, 170.0], 1.0, true, true).unwrap();
        assert!((v[0]).abs() < 1e-6);
        assert!((v[1] + 10.0).abs() < 1e-6);
        // Mouse above the star: delta (0,+70) -> (0,+10) -> flipped to (0,-10).
        let v = star_hit_reflect([100.0, 100.0], [100.0, 30.0], 1.0, true, true).unwrap();
        assert!((v[1] + 10.0).abs() < 1e-6, "the down result flips up");
    }

    #[test]
    fn g23_the_star_reflect_gates_on_game_falling_and_proximity() {
        // Not starGame, or not falling, or out of 70px -> no hit.
        assert!(star_hit_reflect([0.0, 0.0], [0.0, 0.0], 1.0, true, false).is_none());
        assert!(star_hit_reflect([0.0, 0.0], [0.0, 0.0], -1.0, true, true).is_none());
        assert!(star_hit_reflect([0.0, 0.0], [0.0, 0.0], 1.0, false, true).is_none());
    }
}
