//! The front-end flow state machine — the `Main.menuMode == 888` half of
//! the single-mode invariant (`Main.cs:51059-51069`): the screen enum, the
//! keyboard session payloads, the flow-state table and the `TR_UI` presets.
//! (A5: extracted from `main.rs`; the behaviour trait that routes the
//! screens lives in this module too.)

use crate::app::App;
use crate::assets;
use crate::menu;
use crate::render::entities::PlayerAssets;
use crate::ui;
use crate::ui::text::font_width;
use glam::Vec2;
use tr_render::SpriteRenderer;
use winit::event::KeyEvent;
use winit::keyboard::{KeyCode, PhysicalKey};

/// The modern front-end screens (`Main.menuMode == 888`, `Main.cs:51059-
/// 51069`). `App.front` owns one of these while `App.menu` is `None` — the
/// same single variable `Main.menuMode` is; exactly one side is live, never
/// both (Review Focus 1). The Task 6-8 screens join as their own variants.
/// The Keyboard payload keeps the live textbox state, so the enum is
/// `Clone`-only (`CharCreation`/`WorldCreation` payloads make it so anyway).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FrontScreen {
    /// `_characterSelectMenu` — the one `UICharacterSelect` singleton
    /// (`Main.cs:66797`); the port's list lives on `App.players`.
    CharSelect,
    /// `_worldSelectMenu` (`Main.cs:66798`); the list lives on `App.worlds`.
    WorldSelect,
    /// Task 7's `UICharacterCreation` — the draft plus the screen state the
    /// C# keeps on the `UIState` itself (`_selectedPicker`, `_lastSelected-
    /// Hairstyle`'s gate, the slider lock). The payload is the exact value the
    /// mode-40 round-trip parks in `App.front_backup`
    /// (`UICharacterCreation.BackupConfirmationState`, `:1215`).
    CharCreation(ui::char_creation::CreationState),
    /// Task 8's `UIWorldCreation` — the draft carries every option the screen
    /// edits (`_optionwWorldName`/`_optionSeed` and the size/difficulty/evil
    /// picks, `UIWorldCreation.cs:51-143`); the C# keeps no other screen state
    /// (the description panel swaps draw-side from the hover), so the payload
    /// is the plain draft.
    WorldCreation(ui::world_creation::NewWorldDraft),
    /// Task 6's `UIVirtualKeyboard` — the plan Task 5 enum arm, live now:
    /// the card renames and `TR_UI=vk` open it. Boxed: the session's `back`
    /// target is another `FrontScreen`.
    Keyboard(Box<KeyboardSession>),
}

/// The App-side payload of `FrontScreen::Keyboard` — the plan's
/// `KeyboardRequest` (`label`, `initial`, on-submit target) split into the
/// [`ui::keyboard::Request`] ctor args, the live [`ui::keyboard::KeyboardState`]
/// both the on-screen keys and the physical keyboard edit, and the screen the
/// Cancel/Submit tails return to (the `_cancelAction` target,
/// `UIVirtualKeyboard.cs:86`; the tails reopen the select screens,
/// `UICharacterListItem.cs:205-216`, `UIWorldListItem.cs:420-433`). `None`
/// backs the in-game chest rename — the tail returns to gameplay
/// (`IngameFancyUI.Close`'s `inFancyUI = false`, `:162-188`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct KeyboardSession {
    pub(crate) request: ui::keyboard::Request,
    pub(crate) state: ui::keyboard::KeyboardState,
    pub(crate) back: Option<FrontScreen>,
    /// Which submit tail runs — the rename notice (Task 6) or the character
    /// creator's naming flows (`Click_Naming` -> `OnFinishedNaming`,
    /// `UICharacterCreation.cs:1659-1689`; `Click_NamingAndCreating` ->
    /// `OnFinishedNamingAndCreating` -> `FinishCreatingCharacter`, `:1668-1702`).
    pub(crate) tail: KeyboardTail,
}

/// The keyboard's on-submit behaviour (`UIVirtualKeyboard.cs:86`'s event pair).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum KeyboardTail {
    /// The card renames: the port has no `.plr` writer, so Submit notices
    /// (Task 6's documented deviation).
    Rename,
    /// `Click_Naming` (`:1659-1666`): the name lands in the draft, the
    /// creation screen returns.
    Name,
    /// `Click_NamingAndCreating` with an empty name (`:1668-1677`): the name
    /// lands, then `FinishCreatingCharacter` runs (`:1701`).
    NameAndCreate,
    /// `Click_SetName`'s submit (`UIWorldCreation.cs:689-695` ->
    /// `OnFinishedSettingName`, `:726-731`): the trimmed name lands in the
    /// world draft, the creation screen returns.
    WorldName,
    /// `OpenSeedInputMenu`'s submit (`:702-708` -> `OnFinishedSettingSeed`,
    /// `:817-831` minus the secret-seed paths): the trimmed seed lands in the
    /// draft, the creation screen returns.
    WorldSeed,
    /// `Click_NamingAndCreating` with an empty name (`UIWorldCreation.cs:
    /// 710-719`): the name lands, then `FinishCreatingWorld` runs.
    WorldNameAndCreate,
    /// The chest rename (`IngameFancyUI.cs:147-156`): submit runs
    /// `ChestUI.RenameChestSubmit` + `Close(quiet: true)`, cancel runs
    /// `ChestUI.RenameChestCancel` + `Close(quiet: true)` — both tails land
    /// back in gameplay.
    ChestRename,
}

/// One click into the front-end flow — the rows of the
/// [`next_flow_state`] table. The card actions carry the clicked index where
/// the App needs it (`FlowClick::CharDelete(i)` parks the confirm target the
/// way `Main.selectedPlayer` is parked, `UICharacterListItem.cs:246-249`).
#[derive(Clone, Copy, Debug)]
pub(crate) enum FlowClick {
    /// `GoBackClick`: `menuMode = 0` (`UICharacterSelect.cs:116-120`).
    CharBack,
    /// `NewCharacterClick` (`:108-114`): `Main.PendingPlayer = new Player()`,
    /// `menuMode = 888` + `SetState(new UICharacterCreation(...))` — a fresh
    /// draft on the creation screen.
    CharNew,
    /// `PlayGame` (`:256-261`) -> `SelectPlayer`'s single-player tail
    /// `menuMode = 6` (`Main.cs:66245-66249`) -> 888 world select
    /// (`:49468-49472`). The card index rides along for the double-click arm
    /// (`UICharacterListItem.cs:68-69`); the port loads nothing per-entry
    /// (spec §9.4), so the index is routing information only.
    CharPlay(usize),
    /// `DeleteButtonClick` (`UICharacterListItem.cs:242-254`): `menuMode = 5`.
    CharDelete(usize),
    /// `GoBackClick`: `menuMode = (!menuMultiplayer) ? 1 : 12`
    /// (`UIWorldSelect.cs:118-122`) — the port is single-player.
    WorldBack,
    /// `PlayGame` (`UIWorldListItem.cs:360-374`): the launch. `WorldGen.
    /// playWorld` + `UIWorldLoad` are the out-of-scope loading flow (spec
    /// §9.4), so the port closes everything and plays the loaded world. The
    /// card index rides along for the double-click arm
    /// (`UIWorldListItem.cs:69`/`:99`) and is routing information only.
    WorldPlay(usize),
    /// `DeleteButtonClick` (`UIWorldListItem.cs:346-358`): `menuMode = 9`.
    WorldDelete(usize),
    /// `NewWorldClick` (`UIWorldSelect.cs:110-116`): the stand-in world name
    /// reads `Main.WorldList.Count + 1` (`:113`) and the Journey default
    /// reads the active player's difficulty (`UIWorldCreation.cs:319-322`),
    /// so the variant carries both. `menuMode = 888` +
    /// `SetState(new UIWorldCreation())`.
    WorldNew(usize, u8),
    /// `Click_GoBack` -> `GoBack` -> `Main.OpenWorldSelectUI()`
    /// (`UIWorldCreation.cs:665-674`) — straight back to world select.
    WorldCreateBack,
    /// `Click_NamingAndCreating` with a filled name ->
    /// `FinishCreatingWorld` (`UIWorldCreation.cs:710-724`, `:845-863`): the
    /// App generates + saves the `.trwld` first, then the table's re-list of
    /// world select takes over (the vanilla tail is the out-of-scope
    /// `UIWorldLoad` flow, spec §9.4).
    WorldCreate,
    /// The confirm pages' Yes row (`selectedMenu == 1`, `Main.cs:49431`,
    /// `:49681`, `:49454`).
    ConfirmYes,
    /// The No row (`selectedMenu == 2`) — Escape's `flag5` target on the
    /// confirms (`:49435-49438`, `:49687-49690`, `:49464-49467`).
    ConfirmNo,
}

/// One hop of the `Main.menuMode` alternation as a pure table — the plan's
/// `next_state(action) -> (front, menu)` flow-state function, returning
/// `(front, front_backup, menu)`. The parked backup rides along so the
/// mode-40 round-trip is assertable in one place (`App.front_backup` is
/// `UICharacterCreation.BackupConfirmationState`, parked at
/// `UICharacterCreation.cs:1215`). Exactly one of `front`/`menu` is `Some` in
/// every result — the callers hold the same invariant on the way in
/// (Review Focus 1).
pub(crate) fn next_flow_state(
    front: Option<FrontScreen>,
    backup: Option<FrontScreen>,
    page: Option<menu::Page>,
    click: FlowClick,
) -> (Option<FrontScreen>, Option<FrontScreen>, Option<menu::Page>) {
    use menu::Page as P;
    match click {
        FlowClick::CharBack => (None, backup, Some(P::Main)),
        // `NewCharacterClick` (`UICharacterSelect.cs:108-114`): a fresh
        // `PendingPlayer`, the creation screen opens in 888.
        FlowClick::CharNew => (
            Some(FrontScreen::CharCreation(
                ui::char_creation::CreationState::new(),
            )),
            None,
            None,
        ),
        FlowClick::CharPlay(_) => (Some(FrontScreen::WorldSelect), None, None),
        // `menuMode = 5` clears the 888 state (`Main.cs:51066-51069`'s else
        // branch — `MenuUI.SetState(null)`).
        FlowClick::CharDelete(_) => (None, backup, Some(P::ConfirmDeletePlayer)),
        FlowClick::WorldBack => (Some(FrontScreen::CharSelect), None, None),
        // The launch closes both sides (the loading flow is out of scope).
        FlowClick::WorldPlay(_) => (None, None, None),
        FlowClick::WorldDelete(_) => (None, backup, Some(P::ConfirmDeleteWorld)),
        // `NewWorldClick` (`UIWorldSelect.cs:110-116`): the world creator
        // opens with the stand-in name and the active player's difficulty
        // default (`SetDefaultOptions`, `UIWorldCreation.cs:313-326`).
        FlowClick::WorldNew(worlds, active) => (
            Some(FrontScreen::WorldCreation(
                ui::world_creation::NewWorldDraft::new(worlds, active),
            )),
            None,
            None,
        ),
        // `GoBack` (`UIWorldCreation.cs:670-674`): `OpenWorldSelectUI` — no
        // discard confirm on this screen (its draft is choice-only).
        FlowClick::WorldCreateBack => (Some(FrontScreen::WorldSelect), None, None),
        // `FinishCreatingWorld` (`:845-863`) runs before the table row; the
        // reopen re-lists like `OpenWorldSelectUI`'s `OnActivate`.
        FlowClick::WorldCreate => (Some(FrontScreen::WorldSelect), None, None),
        // Modes 5/9: Yes and No both reopen the screen the confirm came from
        // (`Main.cs:49434-49439`, `:49684-49692`).
        FlowClick::ConfirmYes | FlowClick::ConfirmNo if page == Some(P::ConfirmDeletePlayer) => {
            (Some(FrontScreen::CharSelect), None, None)
        }
        FlowClick::ConfirmYes | FlowClick::ConfirmNo if page == Some(P::ConfirmDeleteWorld) => {
            (Some(FrontScreen::WorldSelect), None, None)
        }
        // Mode 40 Yes: `BackupConfirmationState = null` +
        // `OpenCharacterSelectUI` (`:49454-49459`).
        FlowClick::ConfirmYes if page == Some(P::ConfirmDiscard) => {
            (Some(FrontScreen::CharSelect), None, None)
        }
        // Mode 40 No: `menuMode = 888` + `SetState(BackupConfirmationState)`
        // (`:49460-49467`) — the exact parked screen returns.
        FlowClick::ConfirmNo if page == Some(P::ConfirmDiscard) => (backup, None, None),
        // Any other combination is a no-op (a click kind its screen never
        // sends); the state passes through unchanged.
        _ => (front, backup, page),
    }
}

/// The `TR_UI=<screen>` presets (the plan Task 5 hook table): the front-end
/// screens plus the legacy pages their clicks park onto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TrUiPreset {
    CharSelect,
    WorldSelect,
    /// Task 7's screen — parsed now, presettable when its variant lands.
    CharNew,
    /// Task 8's screen.
    WorldNew,
    /// Task 6's keyboard.
    Keyboard,
    /// menuMode 3000.
    Credits,
    /// menuMode 5.
    DelPlayer,
    /// menuMode 9.
    DelWorld,
    /// menuMode 40.
    Discard,
}

/// The `TR_UI` name table (the `TR_PAGE` pattern, `menu.rs`
/// `page_from_name`): the plan Task 5 hook list, case-insensitive; unknown
/// names resolve to nothing.
pub(crate) fn tr_ui_preset(name: &str) -> Option<TrUiPreset> {
    Some(match name.to_ascii_lowercase().as_str() {
        "charsel" => TrUiPreset::CharSelect,
        "worldsel" => TrUiPreset::WorldSelect,
        "charnew" => TrUiPreset::CharNew,
        "worldnew" => TrUiPreset::WorldNew,
        "vk" => TrUiPreset::Keyboard,
        "credits" => TrUiPreset::Credits,
        "delplayer" => TrUiPreset::DelPlayer,
        "delworld" => TrUiPreset::DelWorld,
        "discard" => TrUiPreset::Discard,
        _ => return None,
    })
}

// --- the behaviour trait (the C# `UIState` shape) ---------------------------

/// The frame's shared draw context for the front screens — the pieces of
/// `frame()`'s destructure the draw pass reads (the `Main.MenuUI` draw
/// environment; the arm bodies are the `DrawSelf` ports).
pub(crate) struct FrontDraw<'a> {
    pub(crate) renderer: &'a mut SpriteRenderer,
    pub(crate) lookups: &'a assets::Lookups,
    pub(crate) screen: Vec2,
    /// `_characterSelectMenu`'s card list (`App.players` — the enum doc's
    /// note).
    pub(crate) players: &'a [ui::save_list::PlayerEntry],
    /// `_worldSelectMenu`'s card list (`App.worlds`).
    pub(crate) worlds: &'a [ui::save_list::WorldEntry],
    /// `Main.ActivePlayerFileData.Player.difficulty` (`UIWorldSelect.cs:179`):
    /// the active player's difficulty, gating the card grey-out so it matches
    /// the sort key (`App::active_player_difficulty`).
    pub(crate) active_difficulty: u8,
    pub(crate) player_assets: &'a PlayerAssets,
    /// `Main.GlobalTimeWrappedHourly`'s port — the creation preview's clock.
    pub(crate) anim_time: f32,
    pub(crate) mouse: Vec2,
    /// `Main.mouseLeft`'s port for the front screens — the HSL drags ride
    /// off it (`Main.cs:51101`'s role).
    pub(crate) front_mouse_down: bool,
    /// `TR_VK_GRID=1` — the keyboard's 50-key grid layer (Review Focus 6).
    pub(crate) vk_grid: bool,
    /// The caret's `UITextBox._frameCount`.
    pub(crate) frame_count: u64,
}

/// The five front screens' behaviour surface — the C# `UIState` shape: the
/// header, `DrawSelf`, the `Click_*` handler table and the physical-key path
/// per screen, instead of App-sized match arms. The [`FrontScreen`] enum
/// stays the state carrier (`App.front` — each variant is one screen), so
/// the trait is implemented for the enum and every method's match is the
/// per-screen override, exhaustive over the variants.
pub(crate) trait FrontScreenBehaviours {
    /// The screen's static header text — `UITextPanel`'s label ("Select
    /// Player" / "Select World"). The creation screens and the keyboard draw
    /// their own title pieces (the keyboard's label is the session's
    /// [`ui::keyboard::Request::label`]), so they have none.
    fn header(&self) -> String;

    /// The header's Death-font width (`UITextPanel.cs:126-131`, the
    /// `App::front_header_w` convention) measured over the draw pass — the
    /// select screens' draw prologue.
    fn header_w(&self, d: &FrontDraw<'_>) -> f32 {
        match &d.lookups.death_font {
            Some(f) => font_width(f, &self.header(), 1.0),
            None => d.renderer.text_width(&self.header(), 1.0),
        }
    }

    /// `UIState.DrawSelf` — the draw pass over the world backdrop the frame
    /// already rendered (`Main.menuMode == 888`'s branch,
    /// `Main.cs:51059-51064`).
    fn draw(&mut self, d: &mut FrontDraw<'_>);

    /// The screen's click table (the `Click_*` handlers) — one left click on
    /// the live screen (`Main.MouseUI`'s click while `MenuUI` is live). The
    /// screen payload mutates through `app.front` (the live slot), the list
    /// screens mutate `App`'s lists; a returned click routes through
    /// [`App::apply_front_flow`].
    /// `double_click` is the pointer machine's `DoubleClick` for this press
    /// (`UserInterface.cs:236-240`) — the same element hit twice inside 500 ms —
    /// which is what the card tables turn into `PlayGame`
    /// (`UICharacterListItem.cs:68-69`).
    fn click(
        &self,
        app: &mut App,
        screen: Vec2,
        mouse: Vec2,
        double_click: bool,
    ) -> Option<FlowClick>;

    /// The physical-key path while the screen is live (`Main.GetInputText`
    /// feeding the focused text box). Default: nothing — the keyboard
    /// session is the one text box; the world controls stay parked.
    fn key(&self, _app: &mut App, _event: &KeyEvent) {}
}

/// `UIWorldListItem.PlayGame`'s gate (`UIWorldListItem.cs:360-385`): a Play
/// click (button press or `OnLeftDoubleClick`) enters only for a valid world
/// that passes `TryMovingToRejectionMenuIfNeeded`. An invalid world's click is
/// silent (`base._data.IsValid` false, `:362`); a rejection plays the menu cue
/// (`SoundEngine.PlaySound(10)` `:391/:400/:407`), sets the notice
/// (`Main.statusText`, surfaced through `App::front_hint`) and stops before
/// `SetAsActive`/`WorldGen.playWorld`.
fn world_play_click(app: &mut App, i: usize) -> Option<FlowClick> {
    let (valid, game_mode) = match app.worlds.get(i) {
        Some(w) => (w.valid, w.difficulty),
        None => return None,
    };
    if !valid {
        return None; // `base._data.IsValid` (`:362`) - a dead click
    }
    match ui::world_select::rejection_key(app.active_player_difficulty() == 3, game_mode) {
        Some(key) => {
            app.audio.play("Menu_Open"); // `SoundEngine.PlaySound(10)`
            app.front_hint = Some(crate::lang::text(key));
            None
        }
        None => Some(FlowClick::WorldPlay(i)),
    }
}

impl FrontScreenBehaviours for FrontScreen {
    fn header(&self) -> String {
        match self {
            FrontScreen::CharSelect => ui::char_select::select_player(),
            FrontScreen::WorldSelect => ui::world_select::select_world(),
            // The creation screens and the keyboard draw their own title
            // pieces; no static `UITextPanel`.
            _ => String::new(),
        }
    }

    fn draw(&mut self, d: &mut FrontDraw<'_>) {
        match self {
            FrontScreen::CharSelect => {
                let header_w = self.header_w(d);
                ui::char_select::draw(
                    d.renderer,
                    d.lookups,
                    d.player_assets,
                    d.screen,
                    header_w,
                    d.players,
                    d.mouse,
                );
            }
            FrontScreen::WorldSelect => {
                let header_w = self.header_w(d);
                // The active player's difficulty for the joinable grey-out
                // (`Main.ActivePlayerFileData.Player.difficulty`,
                // `UIWorldSelect.cs:179`): the session character's
                // (`App::active_player_difficulty`) — the same key the sort
                // uses, so the grey-out and the order agree.
                ui::world_select::draw(
                    d.renderer,
                    d.lookups,
                    d.screen,
                    header_w,
                    d.worlds,
                    d.active_difficulty,
                    d.mouse,
                );
            }
            FrontScreen::CharCreation(state) => {
                // `Main.MenuUI.Draw`'s `UICharacterCreation` state
                // (`:1876-2022`'s Draw): the panel, the animated preview
                // (`GlobalTimeWrappedHourly`'s clock — `anim_time`), the
                // categories, the one middle menu, Back/Create and the hover
                // tooltip. The HSL drag runs inside (the `CurrentLockedSlider`
                // latch) off `front_mouse_down`.
                ui::char_creation::draw(
                    d.renderer,
                    d.lookups,
                    d.player_assets,
                    state,
                    d.screen,
                    d.anim_time,
                    d.mouse,
                    d.front_mouse_down,
                );
            }
            FrontScreen::WorldCreation(draft) => {
                // `Main.MenuUI.Draw`'s `UIWorldCreation` state (`:1024-1033`,
                // minus the `SetupGamepadPoints` snap points and the
                // `DrawSeedSystems` special-seed ring — both out of scope):
                // the panel, the plates + preview, the three option rows, the
                // description panel and Back/Create.
                ui::world_creation::draw(d.renderer, d.lookups, draft, d.screen, d.mouse);
            }
            FrontScreen::Keyboard(session) => {
                // `Main.MenuUI.Draw`'s keyboard state
                // (`UIVirtualKeyboard.DrawSelf`, `:680-746`): the desktop
                // layer by default (`ShowGamepadHints` is false without a
                // gamepad), the 50-key grid under `TR_VK_GRID=1` (Review
                // Focus 6). `frame_count` is the caret's
                // `UITextBox._frameCount`.
                let l = ui::keyboard::layout(d.screen, d.vk_grid);
                ui::keyboard::draw(
                    d.renderer,
                    d.lookups,
                    &session.state,
                    &session.request,
                    &l,
                    d.vk_grid,
                    d.mouse,
                    d.frame_count,
                );
            }
        }
    }

    fn click(
        &self,
        app: &mut App,
        screen: Vec2,
        mouse: Vec2,
        double_click: bool,
    ) -> Option<FlowClick> {
        match self {
            FrontScreen::CharSelect => {
                let l = ui::char_select::resolved_layout(
                    screen,
                    app.front_header_w(&self.header()),
                    app.players.len(),
                );
                let favorites: Vec<bool> = app.players.iter().map(|p| p.is_favorite).collect();
                match ui::char_select::card_hit(&l, &favorites, mouse, double_click).1 {
                    Some(ui::char_select::CardAction::Back) => Some(FlowClick::CharBack),
                    // `NewCharacterClick` (`:108-114`): a fresh `PendingPlayer`
                    // and the creation screen (`FlowClick::CharNew`).
                    Some(ui::char_select::CardAction::New) => Some(FlowClick::CharNew),
                    // `PlayGame` (`:256-261`): only a playable entry reaches
                    // `SelectPlayer` (`loadStatus == Ok`); the rest is a dead
                    // click, like the source.
                    Some(ui::char_select::CardAction::Play(i))
                        if app.players.get(i).is_some_and(|p| p.playable) =>
                    {
                        Some(FlowClick::CharPlay(i))
                    }
                    // `OnLeftDoubleClick += PlayGame`
                    // (`UICharacterListItem.cs:68-69`): the same card, the
                    // second press inside 500 ms. Routing only — the port has
                    // no per-card selection state (spec §9.4).
                    Some(ui::char_select::CardAction::DoubleClickPlay(i))
                        if app.players.get(i).is_some_and(|p| p.playable) =>
                    {
                        Some(FlowClick::CharPlay(i))
                    }
                    // `FavoriteButtonClick` (`:264-283`): the flag flips and
                    // the list re-orders (`UpdateOrder`). No `.plr` exists to
                    // persist it — the toggle is session-only (documented
                    // deviation).
                    Some(ui::char_select::CardAction::ToggleFavorite(i)) => {
                        if let Some(p) = app.players.get_mut(i) {
                            p.is_favorite = !p.is_favorite;
                        }
                        ui::char_select::sort_players(&mut app.players);
                        None
                    }
                    // `RenameButtonClick` (`:196-206`): the virtual keyboard
                    // opens over the card's name — `Lang.menu[45]` "Enter
                    // Character Name:", `allowEmpty: true`, the ctor's
                    // 20-character cap (`:199`, `UIVirtualKeyboard.cs:159`).
                    // There is no `.plr` writer to rename into (spec §9.4),
                    // so Submit stays the not-ported notice (Task 6's flow).
                    Some(ui::char_select::CardAction::Rename(i)) => {
                        let name = app
                            .players
                            .get(i)
                            .map(|p| p.name.clone())
                            .unwrap_or_default();
                        app.open_keyboard(
                            ui::keyboard::Request {
                                label: ui::keyboard::enter_character_name(),
                                initial: name,
                                allow_empty: true,
                                max_length: 20,
                                context: ui::keyboard::Context::Text,
                            },
                            FrontScreen::CharSelect,
                        );
                        None
                    }
                    // `DeleteButtonClick` (`:242-254`): parks the target, then
                    // `menuMode = 5`. The favourited gate never subscribed the
                    // handler (`:114-117`), so only `deletable` entries reach
                    // it — and the implicit session entry has no file. The
                    // confirm page re-reads the parked name from the list
                    // (`apply_front_flow`).
                    Some(ui::char_select::CardAction::Delete(i))
                        if app.players.get(i).is_some_and(|p| p.deletable) =>
                    {
                        Some(FlowClick::CharDelete(i))
                    }
                    _ => None,
                }
            }
            FrontScreen::CharCreation(_) => {
                // The creator's click table (`UICharacterCreation`'s handlers
                // through the App): the categories, the Back/Create pair, and
                // the live middle menu's controls. The HSL rows drag in the
                // draw pass, not here (`UIColoredSlider`'s lock latch).
                let l = ui::char_creation::layout(screen, 0.0);
                // The screen-state mutations run under the borrow; the
                // navigation clicks clone the payload they need and drop it.
                enum Nav {
                    None,
                    Name(ui::char_creation::CreationState),
                    Create(ui::char_creation::CreationState),
                    Back(ui::char_creation::CreationState),
                }
                let nav = {
                    let Some(FrontScreen::CharCreation(state)) = app.front.as_mut() else {
                        return None;
                    };
                    match ui::char_creation::click_at(&l, state.category, mouse) {
                        // `SelectColorPicker`/`Click_*` (`:1810-1833`): the
                        // picker selects and the middle menu follows
                        // (`middle_for`), seeding `_currentColorHSL` from the
                        // live colour — the draw re-derives it.
                        ui::char_creation::Click::Category(id) => {
                            state.category = id;
                            Nav::None
                        }
                        // `UIDifficultyButton.LeftMouseDown`
                        // (`UIDifficultyButton.cs:67-71`).
                        ui::char_creation::Click::Difficulty(d) => {
                            state.draft.difficulty = d;
                            Nav::None
                        }
                        // `UIHairStyleButton.LeftMouseDown` (`UIHairStyleButton.
                        // cs:66-70`) + `RecordThatHairWasSelected` (`:306-309`):
                        // the auto-assign gate closes.
                        ui::char_creation::Click::Hair(h) => {
                            state.draft.hair = h;
                            state.hair_picked = true;
                            Nav::None
                        }
                        // `Click_CharClothStyle` (`:1277-1291`): the armor
                        // guard is out of scope (no armor preview); the
                        // variant lands and the hair auto-assigns while
                        // untouched (`TryAutoAssigningHair`, `:630-637`).
                        ui::char_creation::Click::ClothStyle(v) => {
                            state.draft.skin_variant = v;
                            if !state.hair_picked {
                                if let Some(hair) =
                                    ui::char_creation::default_hairstyle_for_cloth_style(v)
                                {
                                    state.draft.hair = hair;
                                }
                            }
                            Nav::None
                        }
                        ui::char_creation::Click::Name => Nav::Name(state.clone()),
                        ui::char_creation::Click::Create => Nav::Create(state.clone()),
                        ui::char_creation::Click::Back => Nav::Back(state.clone()),
                        // `Click_CopyHex` (`:1331-1335`): the live
                        // `_hslHexText` readout to the clipboard.
                        ui::char_creation::Click::CopyHex => {
                            let hex = ui::char_creation::hex_text(
                                state.draft.color_of(state.category).unwrap_or([255; 4]),
                            );
                            ui::clipboard::set_text(&hex);
                            Nav::None
                        }
                        // `Click_PasteHex` (`:1337-1347`): a clipboard hex
                        // colour lands on the active category.
                        ui::char_creation::Click::PasteHex => {
                            let category = state.category;
                            if let Some(text) = ui::clipboard::get_text() {
                                ui::char_creation::paste_hex(&mut state.draft, category, &text);
                            }
                            Nav::None
                        }
                        // `Click_RandomizeSingleColor` (`:1779-1787`): the
                        // random colour lands on the active category.
                        ui::char_creation::Click::RandomizeColor => {
                            let category = state.category;
                            let color =
                                ui::char_creation::randomize_single_color(&mut app.prefix_rng);
                            state.draft.set_color_of(category, color);
                            Nav::None
                        }
                        // `Click_CopyPlayerTemplate` (`:1408-1412`).
                        ui::char_creation::Click::CopyTemplate => {
                            ui::clipboard::set_text(&ui::char_creation::player_template(
                                &state.draft,
                            ));
                            Nav::None
                        }
                        // `Click_PastePlayerTemplate` (`:1414-1510`): the
                        // clipboard template lands on the draft (validated).
                        ui::char_creation::Click::PasteTemplate => {
                            if let Some(text) = ui::clipboard::get_text() {
                                ui::char_creation::apply_player_template(
                                    &mut state.draft,
                                    &mut state.hair_picked,
                                    &text,
                                );
                            }
                            Nav::None
                        }
                        // `Click_RandomizePlayer` (`:1545-1657`): the whole
                        // random draft, `Main.rand`-driven.
                        ui::char_creation::Click::RandomizePlayer => {
                            ui::char_creation::randomize_player(
                                &mut app.prefix_rng,
                                &mut state.draft,
                                &mut state.hair_picked,
                            );
                            Nav::None
                        }
                        ui::char_creation::Click::None => Nav::None,
                    }
                };
                match nav {
                    Nav::None => {}
                    Nav::Name(state) => {
                        // `Click_Naming` (`:1659-1666`): the name clears
                        // (`Main.clrInput()`'s effect on the draft), the
                        // keyboard opens with `allowEmpty: true`.
                        let back = FrontScreen::CharCreation(state);
                        app.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: ui::keyboard::enter_character_name(),
                                initial: String::new(),
                                allow_empty: true,
                                max_length: ui::char_creation::MAX_NAME_LENGTH,
                                context: ui::keyboard::Context::Text,
                            },
                            Some(back),
                            KeyboardTail::Name,
                        );
                    }
                    Nav::Create(state) => {
                        // `Click_NamingAndCreating` (`:1668-1682`): an empty
                        // name opens the naming keyboard (no `allowEmpty` —
                        // the ctor default `false`), a filled one finishes.
                        if state.draft.name.is_empty() {
                            let back = FrontScreen::CharCreation(state);
                            app.open_keyboard_with_tail(
                                ui::keyboard::Request {
                                    label: ui::keyboard::enter_character_name(),
                                    initial: String::new(),
                                    allow_empty: false,
                                    max_length: ui::char_creation::MAX_NAME_LENGTH,
                                    context: ui::keyboard::Context::Text,
                                },
                                Some(back),
                                KeyboardTail::NameAndCreate,
                            );
                        } else {
                            app.finish_creating_character();
                        }
                    }
                    Nav::Back(state) => {
                        // `GoBack` (`:1210-1222`): dirty parks the exact
                        // screen for the mode-40 round trip
                        // (`BackupConfirmationState`, `:1215`); clean goes
                        // straight back to the char select
                        // (`OpenCharacterSelectUI`).
                        if state.draft.dirty {
                            let name =
                                (!state.draft.name.is_empty()).then(|| state.draft.name.clone());
                            app.front = None;
                            app.front_backup = Some(FrontScreen::CharCreation(state));
                            app.park_menu();
                            app.menu =
                                Some(menu::Menu::open_confirm_discard(app.settings.clone(), name));
                        } else {
                            app.front = None;
                            app.enter_front(FrontScreen::CharSelect);
                        }
                    }
                }
                None
            }
            FrontScreen::WorldCreation(_) => {
                // The world creator's click table (`UIWorldCreation`'s
                // handlers through the App): the two plate icons, the two
                // plates, the three option rows and the Back/Create pair.
                let l = ui::world_creation::layout(screen, 0.0);
                // The draft mutations run under the borrow; the keyboard
                // clicks clone the draft and drop it, and the navigation
                // clicks ride the flow table like the select arms do.
                enum Nav {
                    None,
                    Name(ui::world_creation::NewWorldDraft),
                    Seed(ui::world_creation::NewWorldDraft),
                    NameAndCreate(ui::world_creation::NewWorldDraft),
                }
                let world_count = app.worlds.len();
                let mut nav = Nav::None;
                let mut flow = None;
                let mut create_draft: Option<ui::world_creation::NewWorldDraft> = None;
                {
                    let Some(FrontScreen::WorldCreation(draft)) = app.front.as_mut() else {
                        return None;
                    };
                    match ui::world_creation::click_at(&l, mouse) {
                        // `ClickRandomizeName` (`:497-503`): the port's
                        // `AssignRandomWorldName` stand-in re-derives the
                        // name (`:1002-1025`'s composition tables have no
                        // localization port — documented).
                        ui::world_creation::Click::RandomName => {
                            draft.assign_random_world_name(world_count);
                        }
                        // `ClickSizeOption`/`ClickDifficultyOption`/
                        // `ClickEvilOption` (`:541-575`): the pick lands and
                        // `UpdatePreviewPlate` re-derives draw-side.
                        ui::world_creation::Click::Size(size) => draft.size = size,
                        ui::world_creation::Click::Difficulty(difficulty) => {
                            draft.difficulty = difficulty;
                        }
                        ui::world_creation::Click::Evil(evil) => draft.evil = evil,
                        // `Click_SetName` (`:689-695`): `Lang.menu[48]`,
                        // `allowEmpty: true`, the 27 cap (`:693`).
                        ui::world_creation::Click::Name => nav = Nav::Name(draft.clone()),
                        // `Click_SetSeed` -> `OpenSeedInputMenu` (`:697-708`):
                        // "UI.EnterSeed", `allowEmpty: true`, no cap (the
                        // source's `int.MaxValue`); the icon's Advanced-tab
                        // click degrades to the same keyboard (plan
                        // Uncertainty 4).
                        ui::world_creation::Click::Seed => nav = Nav::Seed(draft.clone()),
                        // `Click_NamingAndCreating` (`:710-724`): an empty
                        // name opens the naming keyboard (`allowEmpty: false`,
                        // 27), a filled one finishes (`:722`).
                        ui::world_creation::Click::Create => {
                            if draft.name.is_empty() {
                                nav = Nav::NameAndCreate(draft.clone());
                            } else {
                                // `FinishCreatingWorld` (`:845-863`) saves
                                // before the table's re-list takes over.
                                create_draft = Some(draft.clone());
                                flow = Some(FlowClick::WorldCreate);
                            }
                        }
                        // `Click_GoBack` -> `GoBack` -> `OpenWorldSelectUI`
                        // (`:665-674`) — no discard confirm on this screen.
                        ui::world_creation::Click::Back => {
                            flow = Some(FlowClick::WorldCreateBack);
                        }
                        ui::world_creation::Click::None => {}
                    }
                }
                match nav {
                    Nav::None => {}
                    Nav::Name(draft) => {
                        // The ctor's `startingText` is "" — the box opens
                        // empty even on a renamed draft (`:693`).
                        app.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: ui::keyboard::enter_world_name(),
                                initial: String::new(),
                                allow_empty: true,
                                max_length: ui::world_creation::MAX_NAME_LENGTH,
                                context: ui::keyboard::Context::Text,
                            },
                            Some(FrontScreen::WorldCreation(draft)),
                            KeyboardTail::WorldName,
                        );
                    }
                    Nav::Seed(draft) => {
                        // `UI.EnterSeed` ("Enter Seed (Leave Blank For
                        // Random)"), `allowEmpty: true`, `int.MaxValue`
                        // (`UIWorldCreation.cs:706`); the starting text is ""
                        // there too (`:706`).
                        app.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: "Enter Seed (Leave Blank For Random)".into(),
                                initial: String::new(),
                                allow_empty: true,
                                max_length: usize::MAX,
                                context: ui::keyboard::Context::Text,
                            },
                            Some(FrontScreen::WorldCreation(draft)),
                            KeyboardTail::WorldSeed,
                        );
                    }
                    Nav::NameAndCreate(draft) => {
                        app.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: ui::keyboard::enter_world_name(),
                                initial: String::new(),
                                allow_empty: false,
                                max_length: ui::world_creation::MAX_NAME_LENGTH,
                                context: ui::keyboard::Context::Text,
                            },
                            Some(FrontScreen::WorldCreation(draft)),
                            KeyboardTail::WorldNameAndCreate,
                        );
                    }
                }
                // The Create save (`:845-863`) runs before the table hop; a
                // failure shows the not-ported notice (the vanilla flow has
                // no error UI on this hop) and the re-list still happens.
                if let Some(draft) = create_draft {
                    match ui::world_creation::create_world(&draft) {
                        Ok(path) => {
                            log::info!("created '{}' -> {}", draft.name.trim(), path.display());
                        }
                        Err(e) => {
                            app.front_hint = Some(format!("world creation failed: {e:#}"));
                        }
                    }
                }
                flow
            }
            FrontScreen::WorldSelect => {
                // The world screen's ctor is the char select's twin
                // (`UIWorldSelect.cs:40-87`), so the shared `resolved_layout`
                // lives on `char_select` and serves both hit-tests.
                let l = ui::char_select::resolved_layout(
                    screen,
                    app.front_header_w(&self.header()),
                    app.worlds.len(),
                );
                let favorites: Vec<bool> = app.worlds.iter().map(|w| w.is_favorite).collect();
                match ui::world_select::card_hit(&l, &favorites, mouse, double_click).1 {
                    Some(ui::world_select::CardAction::Back) => Some(FlowClick::WorldBack),
                    // `NewWorldClick` (`:110-116`): the stand-in name reads
                    // `Main.WorldList.Count + 1` and the Journey default reads
                    // the active player's difficulty (`UIWorldCreation.cs:
                    // 319-322`) — the session entry's.
                    Some(ui::world_select::CardAction::New) => Some(FlowClick::WorldNew(
                        app.worlds.len(),
                        app.session_entry().difficulty,
                    )),
                    // `PlayGame` (`:360-385`): `_data.IsValid` then
                    // `TryMovingToRejectionMenuIfNeeded` gate the launch. An
                    // invalid world's click is dead; a Journey mismatch sets
                    // the notice and refuses (the port has no `menuMode
                    // 1000000` page — ANALYSIS "World select (S1-H-10)").
                    Some(ui::world_select::CardAction::Play(i)) => world_play_click(app, i),
                    // `OnLeftDoubleClick += PlayGame`
                    // (`UIWorldListItem.cs:69`/`:99`): the same card, the
                    // second press inside 500 ms, through the same gate.
                    Some(ui::world_select::CardAction::DoubleClickPlay(i)) => {
                        world_play_click(app, i)
                    }
                    // `FavoriteButtonClick` (`:439-458`): `ToggleFavorite`
                    // saves (`FileData.cs:38-40`) and the list re-orders. The
                    // port's marker is the `.trfav` sidecar
                    // (`save_list::WorldEntry::is_favorite`'s reader).
                    Some(ui::world_select::CardAction::ToggleFavorite(i)) => {
                        if let Some(w) = app.worlds.get_mut(i) {
                            w.is_favorite = !w.is_favorite;
                            if let Some(path) = &w.path {
                                let marker = path.with_extension("trfav");
                                if w.is_favorite {
                                    let _ = std::fs::write(&marker, b"");
                                } else {
                                    let _ = std::fs::remove_file(&marker);
                                }
                            }
                        }
                        app.resort_worlds();
                        None
                    }
                    // `RenameButtonClick` (`UIWorldListItem.cs:415-421`): the
                    // world twin — `Lang.menu[48]` "Enter World Name:",
                    // `allowEmpty: true`, the 27-character cap (`:419`).
                    Some(ui::world_select::CardAction::Rename(i)) => {
                        let name = app
                            .worlds
                            .get(i)
                            .map(|w| w.name.clone())
                            .unwrap_or_default();
                        app.open_keyboard(
                            ui::keyboard::Request {
                                label: ui::keyboard::enter_world_name(),
                                initial: name,
                                allow_empty: true,
                                max_length: 27,
                                context: ui::keyboard::Context::Text,
                            },
                            FrontScreen::WorldSelect,
                        );
                        None
                    }
                    // `DeleteButtonClick` (`:346-358`): parks the target, then
                    // `menuMode = 9`. The confirm page re-reads the parked
                    // name from the list (`apply_front_flow`).
                    Some(ui::world_select::CardAction::Delete(i))
                        if app.worlds.get(i).is_some_and(|w| w.path.is_some()) =>
                    {
                        Some(FlowClick::WorldDelete(i))
                    }
                    _ => None,
                }
            }
            FrontScreen::Keyboard(_) => {
                // The keyboard's own click table (`UIVirtualKeyboard`'s
                // handlers): the on-screen keys type, the toggles cycle the
                // `KeyState`, Submit re-checks the validation, Cancel runs
                // the `_cancelAction`.
                let Some(FrontScreen::Keyboard(session)) = app.front.as_mut() else {
                    return None;
                };
                let l = ui::keyboard::layout(screen, app.vk_grid);
                match ui::keyboard::click_at(&session.state, &l, app.vk_grid, mouse) {
                    ui::keyboard::Click::Char(c) => ui::keyboard::type_char(&mut session.state, c),
                    // The shift button: `(keyState != Shift) ? Shift : Default`
                    // (`:250-254`).
                    ui::keyboard::Click::Shift => {
                        let next = if session.state.key_state == ui::keyboard::KeyState::Shift {
                            ui::keyboard::KeyState::Default
                        } else {
                            ui::keyboard::KeyState::Shift
                        };
                        session.state.set_key_state(next);
                    }
                    // The symbol button: `(keyState != Symbol) ? Symbol : Default`
                    // (`:284-289`).
                    ui::keyboard::Click::Symbol => {
                        let next = if session.state.key_state == ui::keyboard::KeyState::Symbol {
                            ui::keyboard::KeyState::Default
                        } else {
                            ui::keyboard::KeyState::Symbol
                        };
                        session.state.set_key_state(next);
                    }
                    // `PressSpace` writes without the Shift unlatch (`:491-502`).
                    ui::keyboard::Click::Space => ui::keyboard::write_str(&mut session.state, " "),
                    // The backspace key pops (`:392-397`).
                    ui::keyboard::Click::Backspace => session.state.backspace(),
                    // The desktop pair's handler checks the validation first
                    // (`:358-365`); the small pair's `InternalSubmit` re-checks
                    // too (`:970-978`) — the invalid click is dead either way.
                    // The chest context is always valid (`:615-626`'s
                    // `_editingChest` gate).
                    ui::keyboard::Click::Submit => {
                        if ui::keyboard::text_valid_for(
                            &session.state.text,
                            session.request.allow_empty,
                            session.request.context,
                        ) {
                            app.keyboard_submit();
                        }
                    }
                    // `_cancelAction` (`:319-323`, `:385-389`).
                    ui::keyboard::Click::Cancel => app.keyboard_cancel(),
                    ui::keyboard::Click::None => {}
                }
                None
            }
        }
    }

    fn key(&self, app: &mut App, event: &KeyEvent) {
        match self {
            // The keyboard is the one front screen with text input:
            // `Main.GetInputText` edits its text every frame
            // (`UIVirtualKeyboard.cs:750-788`), Enter submits through
            // `CanSubmit` (`:763-766`) and Escape runs the cancel
            // (`:771-774`'s `TryEscapingMenu`).
            FrontScreen::Keyboard(_) => {
                if let Some(text) = event.text.as_ref() {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            app.keyboard_character(ch);
                        }
                    }
                }
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Backspace) => app.keyboard_backspace(),
                    PhysicalKey::Code(KeyCode::Escape) => app.keyboard_cancel(),
                    PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter) => {
                        app.keyboard_enter()
                    }
                    _ => {}
                }
            }
            // The other screens take no keys: the world controls stay parked
            // while a front screen is live (the gamepad snap points are out
            // of scope, §J.3).
            _ => {}
        }
    }
}

impl App {
    /// The element id under the pointer on the live front screen — the
    /// framework's `GetElementAt` (`UIElement.cs:324-352`), fed to the pointer
    /// machine so its `LastClicked` compares elements and a card's double-click
    /// is recognised (`UICharacterListItem.cs:68-69`). `None` on the screens
    /// without element ids (the creation screens and the keyboard) and when no
    /// front screen is live.
    pub(crate) fn front_hit(&self) -> Option<u32> {
        let front = self.front.as_ref()?;
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        match front {
            FrontScreen::CharSelect => {
                let header = front.header();
                let l = ui::char_select::resolved_layout(
                    screen,
                    self.front_header_w(&header),
                    self.players.len(),
                );
                let favorites: Vec<bool> = self.players.iter().map(|p| p.is_favorite).collect();
                ui::char_select::hit_element(&l, &favorites, self.mouse_pos)
            }
            FrontScreen::WorldSelect => {
                let header = front.header();
                let l = ui::world_select::resolved_layout(
                    screen,
                    self.front_header_w(&header),
                    self.worlds.len(),
                );
                let favorites: Vec<bool> = self.worlds.iter().map(|w| w.is_favorite).collect();
                ui::world_select::hit_element(&l, &favorites, self.mouse_pos)
            }
            _ => None,
        }
    }

    /// One left click on a live front screen: the card-hit table routed into
    /// the flow (`UICharacterSelect.cs:108-133` / `UIWorldSelect.cs:110-135`'s
    /// handlers through the App). The favourite toggles re-order the list like
    /// `FavoriteButtonClick`'s `UpdateOrder` (`UICharacterListItem.cs:280-282`
    /// / `UIWorldListItem.cs:455-458`).
    pub(crate) fn handle_front_click(&mut self) {
        // `UserInterface.Update`'s click gate (`UserInterface.cs:214-218`): the
        // press that lands inside the 200 ms window after a screen switch is
        // dropped, exactly as the C# skips `HandleClick` while
        // `_clickDisabledTimeRemaining > 0` (`:203-204`).
        if self.pointer.clicks_disabled() {
            self.pointer_event = None;
            return;
        }
        // The machine's `DoubleClick` for this press (`:236-240`), consumed
        // here like the C#'s delegate fires once.
        let double_click = self.pointer_event == Some(ui::pointer::PointerEvent::DoubleClick);
        self.pointer_event = None;
        // `FrontScreen` stopped being `Copy` when the keyboard variant grew a
        // payload — the snapshot clones instead of moving out of `self`.
        let Some(front) = self.front.clone() else {
            return;
        };
        // A click clears the notice, like `Menu::activate`'s `hint = None`.
        self.front_hint = None;
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let mouse = self.mouse_pos;
        // The `UIState` dispatch: the live screen owns its `Click_*` table.
        let click = front.click(self, screen, mouse, double_click);
        if let Some(click) = click {
            // Bug 17: the activation cue — the vanilla clicks play the menu
            // open (10) on activation and the menu close (11) on the Back
            // hops (`UICharacterSelect.cs:110/:118`, `UIWorldListItem.cs:365`).
            if matches!(
                click,
                FlowClick::CharBack
                    | FlowClick::WorldBack
                    | FlowClick::WorldCreateBack
                    | FlowClick::ConfirmNo
            ) {
                self.audio.play("Menu_Close"); // `SoundEngine.PlaySound(11)`
            } else {
                self.audio.play("Menu_Open"); // `SoundEngine.PlaySound(10)`
            }
            self.apply_front_flow(front, click);
        }
    }

    /// The click tables' shared flow-apply tail: one [`next_flow_state`]
    /// hop lands on the App state. Review Focus 1: the table result IS the
    /// App's state — `front` is written on every hop, so the page arms below
    /// (`menuMode = 5` / `= 0`) clear the 888 side (`Main.cs:51066-51069`'s
    /// else branch) and the legacy page goes live instead of hiding under the
    /// front screen.
    fn apply_front_flow(&mut self, front: FrontScreen, click: FlowClick) {
        let delete_index = match click {
            FlowClick::CharDelete(i) | FlowClick::WorldDelete(i) => Some(i),
            _ => None,
        };
        // `UIWorldListItem.PlayGame` (`:360-374`) → `Main.LoadWorld`: the
        // card's file is parked on the App, and the flow's launch arm closes
        // both screens. The frame loop performs the swap at the next boundary
        // (BUG-04) — the vanilla enters its loading flow here.
        if let FlowClick::WorldPlay(i) = click {
            if let Some(path) = self.worlds.get(i).and_then(|w| w.path.clone()) {
                self.play_world = Some(path);
            }
        }
        let (front, backup, page) =
            next_flow_state(Some(front), self.front_backup.clone(), None, click);
        self.front_backup = backup;
        self.front = front.clone();
        match page {
            // Mode 5 opens with the parked name/index (`Main.selectedPlayer`,
            // `UICharacterListItem.cs:246-249`) — re-read from the list, which
            // the click gate snapshot identically (`players` is untouched
            // between the gate and this hop).
            Some(menu::Page::ConfirmDeletePlayer) => {
                self.park_menu();
                let name = self
                    .players
                    .get(delete_index.unwrap_or(0))
                    .map(|p| p.name.clone());
                let mut m = menu::Menu::open_confirm_delete_player(
                    self.settings.clone(),
                    name.unwrap_or_default(),
                );
                m.confirm_index = delete_index.unwrap_or(0);
                self.menu = Some(m);
            }
            // Mode 9 (`Main.selectedWorld`, `UIWorldListItem.cs:350-353`).
            Some(menu::Page::ConfirmDeleteWorld) => {
                self.park_menu();
                let name = self
                    .worlds
                    .get(delete_index.unwrap_or(0))
                    .map(|w| w.name.clone());
                let mut m = menu::Menu::open_confirm_delete_world(
                    self.settings.clone(),
                    name.unwrap_or_default(),
                );
                m.confirm_index = delete_index.unwrap_or(0);
                self.menu = Some(m);
            }
            Some(p) => {
                // A plain legacy page (the Back row's `menuMode = 0`).
                self.park_menu();
                let mut m = menu::Menu::new(self.settings.clone(), self.player_name.clone());
                m.page = p;
                self.menu = Some(m);
            }
            None => match front {
                Some(f) => self.enter_front(f),
                None => {
                    // The launch: everything closes and the settings apply
                    // (`close_menu`'s tail).
                    self.front = None;
                    self.park_menu();
                }
            },
        }
    }
}

#[cfg(test)]
pub(crate) mod front_flow_tests {
    use super::*;
    use crate::*;

    // A7 moved `App`/`NetRole` and their satellite types into `crate::app`
    // (and the render/rng modules); the crate-root glob no longer carries
    // them, so the headless-App fixture imports them directly.
    use crate::app::{App, NamedRngs, NetRole};
    use crate::item::{EMPTY_SLOT, HOTBAR};
    use crate::rain::Rain;
    use crate::render::debug::{DebugCache, StatsAcc};
    use crate::render::entities::NpcAssets;
    use crate::render::sky::SkyAssets;
    use crate::render::tiles::{DrawScratch, TileDrawStats};
    use crate::world_sim::step::FixedStepper;
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::time::Instant;
    use tr_core::World;
    use tr_core::lighting::{LightMap, LightMode};
    use tr_core::liquid::LiquidSystem;

    use crate::menu::Page as P;

    #[test]
    fn the_flow_table_routes_every_front_hop() {
        // Char select Back -> menuMode 0 (`UICharacterSelect.cs:116-120`).
        let (front, backup, page) = next_flow_state(
            Some(FrontScreen::CharSelect),
            None,
            None,
            FlowClick::CharBack,
        );
        assert_eq!(
            (front.is_some(), backup.is_some(), page),
            (false, false, Some(P::Main))
        );
        // Play -> the world select (`Main.cs:66245-66249` + `:49468-49472`).
        let (front, backup, page) = next_flow_state(
            Some(FrontScreen::CharSelect),
            None,
            None,
            FlowClick::CharPlay(0),
        );
        assert_eq!(
            (front, backup.is_some(), page.is_some()),
            (Some(FrontScreen::WorldSelect), false, false)
        );
        // Delete -> mode 5 clears the 888 state (`UICharacterListItem.cs:
        // 242-254`, `Main.cs:51066-51069`'s else branch).
        let (front, backup, page) = next_flow_state(
            Some(FrontScreen::CharSelect),
            None,
            None,
            FlowClick::CharDelete(2),
        );
        assert_eq!(
            (front.is_some(), backup.is_some(), page),
            (false, false, Some(P::ConfirmDeletePlayer))
        );
        // World Back -> the char select (`UIWorldSelect.cs:118-122`, the
        // single-player arm of `menuMode = (!menuMultiplayer) ? 1 : 12`).
        let (front, backup, page) = next_flow_state(
            Some(FrontScreen::WorldSelect),
            None,
            None,
            FlowClick::WorldBack,
        );
        assert_eq!(
            (front, backup.is_some(), page.is_some()),
            (Some(FrontScreen::CharSelect), false, false)
        );
        // World Play launches: both sides close (the loading flow is out of
        // scope, spec §9.4).
        let (front, backup, page) = next_flow_state(
            Some(FrontScreen::WorldSelect),
            None,
            None,
            FlowClick::WorldPlay(0),
        );
        assert_eq!(
            (front.is_some(), backup.is_some(), page.is_some()),
            (false, false, false)
        );
        // World Delete -> mode 9 (`UIWorldListItem.cs:346-358`).
        let (front, backup, page) = next_flow_state(
            Some(FrontScreen::WorldSelect),
            None,
            None,
            FlowClick::WorldDelete(1),
        );
        assert_eq!(
            (front.is_some(), backup.is_some(), page),
            (false, false, Some(P::ConfirmDeleteWorld))
        );
    }

    #[test]
    fn confirm_resolutions_reopen_their_screen() {
        // Mode 5: both branches reopen the char select (`Main.cs:49434-49439`).
        for click in [FlowClick::ConfirmYes, FlowClick::ConfirmNo] {
            let (front, backup, page) =
                next_flow_state(None, None, Some(P::ConfirmDeletePlayer), click);
            assert_eq!(front, Some(FrontScreen::CharSelect), "{click:?}");
            assert!((backup.is_none()) && page.is_none(), "{click:?}");
        }
        // Mode 9: the world select (`:49684-49690`).
        for click in [FlowClick::ConfirmYes, FlowClick::ConfirmNo] {
            let (front, backup, page) =
                next_flow_state(None, None, Some(P::ConfirmDeleteWorld), click);
            assert_eq!(front, Some(FrontScreen::WorldSelect), "{click:?}");
            assert!((backup.is_none()) && page.is_none(), "{click:?}");
        }
        // Mode 40 Yes: the parked state drops and the char select reopens
        // (`BackupConfirmationState = null` + `OpenCharacterSelectUI`,
        // `:49454-49459`).
        let (front, backup, page) = next_flow_state(
            None,
            Some(FrontScreen::CharSelect),
            Some(P::ConfirmDiscard),
            FlowClick::ConfirmYes,
        );
        assert_eq!(
            (front.is_some(), backup.is_some(), page.is_some()),
            (true, false, false)
        );
    }

    #[test]
    fn the_mode_40_no_branch_restores_the_exact_parked_screen() {
        // `Main.cs:49460-49467`: `menuMode = 888` +
        // `SetState(BackupConfirmationState)` — the parked value returns
        // untouched and the backup clears (Review Focus 1's round-trip;
        // `App.front_backup` is the port's `BackupConfirmationState`). The
        // creation screen joins with Task 7: its draft/category/grab payload
        // is exactly what `GoBack` parks (`UICharacterCreation.cs:1215`).
        let mut creation = ui::char_creation::CreationState::new();
        creation.category = 4;
        creation.draft.name = "Zealot".into();
        creation.hsl_grab = 2;
        for parked in [
            FrontScreen::CharSelect,
            FrontScreen::WorldSelect,
            FrontScreen::CharCreation(creation),
            FrontScreen::WorldCreation(ui::world_creation::NewWorldDraft::new(1, 0)),
        ] {
            let (front, backup, page) = next_flow_state(
                None,
                Some(parked.clone()),
                Some(P::ConfirmDiscard),
                FlowClick::ConfirmNo,
            );
            assert_eq!(front, Some(parked), "the exact parked screen returns");
            assert!((backup.is_none()) && page.is_none());
        }
    }

    #[test]
    fn bug17_a_front_click_plays_the_activation_cue() {
        // Bug 17: the front-end clicks play the menu open (10) — the vanilla
        // `UICharacterListItem.cs:198` / `UIWorldListItem.cs:365` cues.
        let players = vec![entry("Alpha", true), entry("Beta", true)];
        let mut app = flow_app(Some(FrontScreen::CharSelect), players, Vec::new());
        // The 200 ms entry lockout from `enter_front` must be gone first.
        app.pointer.tick(300.0);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let l = ui::char_select::resolved_layout(screen, app.front_header_w("Select Player"), 2);
        let c0 = ui::char_select::card_layout(l.list, 0);
        let before = app.audio.played_count();
        // The double-click on the card body runs the play flow
        // (`UICharacterListItem.cs:198`'s `PlaySound(10)`).
        app.mouse_pos = Vec2::new(c0.card[0] + c0.card[2] * 0.5, c0.card[1] + c0.card[3] * 0.5);
        app.pointer_event = Some(ui::pointer::PointerEvent::DoubleClick);
        app.handle_front_click();
        assert!(
            app.audio.played_count() > before,
            "the activation plays a cue ({} -> {})",
            before,
            app.audio.played_count()
        );
        assert!(matches!(app.front, Some(FrontScreen::WorldSelect)));
    }

    #[test]
    fn bug17_the_hover_edge_plays_the_menu_tick_once() {
        // Bug 17: the hover edge — entering an element plays one Menu_Tick
        // (`UICharacterSelect.cs:124`'s `FadedMouseOver`), staying inside
        // plays nothing more.
        let mut app = flow_app(Some(FrontScreen::CharSelect), Vec::new(), Vec::new());
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let l = ui::char_select::resolved_layout(screen, app.front_header_w("Select Player"), 0);
        // The Back/New buttons' rect is the hover target.
        let back = l.back;
        app.mouse_pos = Vec2::new(back[0] + back[2] * 0.5, back[1] + back[3] * 0.5);
        let before = app.audio.played_count();
        // The hover edge comparison the loop's cursor-moved path runs.
        let hit = app.front_hit();
        assert!(hit.is_some(), "the pointer is over an element");
        if hit != app.front_hover_hit {
            if hit.is_some() {
                app.audio.play("Menu_Tick");
            }
            app.front_hover_hit = hit;
        }
        assert_eq!(
            app.audio.played_count(),
            before + 1,
            "one tick on the enter"
        );
        // A second hover inside the same element is silent.
        let hit2 = app.front_hit();
        if hit2 != app.front_hover_hit {
            if hit2.is_some() {
                app.audio.play("Menu_Tick");
            }
            app.front_hover_hit = hit2;
        }
        assert_eq!(app.audio.played_count(), before + 1, "no repeat inside");
    }

    #[test]
    fn front_and_menu_are_never_both_some() {
        // Review Focus 1: every row of the flow table keeps the single-mode
        // invariant `Main.menuMode` carries (`Main.cs:51059-51069` — 888
        // draws `MenuUI`, every other mode has `SetState(null)`). The matrix
        // covers every live `FrontScreen` variant — the keyboard session was
        // the Task 6 gap, the creation screen joins with Task 7.
        let session = || {
            let request = ui::keyboard::Request {
                label: ui::keyboard::enter_character_name(),
                initial: String::new(),
                allow_empty: true,
                max_length: 20,
                context: ui::keyboard::Context::Text,
            };
            let state = ui::keyboard::KeyboardState::new(&request);
            Box::new(KeyboardSession {
                request,
                state,
                back: Some(FrontScreen::CharSelect),
                tail: KeyboardTail::Rename,
            })
        };
        let fronts = [
            None,
            Some(FrontScreen::CharSelect),
            Some(FrontScreen::WorldSelect),
            Some(FrontScreen::CharCreation(
                ui::char_creation::CreationState::new(),
            )),
            // The world creator joins with Task 8.
            Some(FrontScreen::WorldCreation(
                ui::world_creation::NewWorldDraft::new(0, 0),
            )),
            Some(FrontScreen::Keyboard(session())),
        ];
        let backs = [
            None,
            Some(FrontScreen::CharSelect),
            Some(FrontScreen::WorldSelect),
            Some(FrontScreen::CharCreation(
                ui::char_creation::CreationState::new(),
            )),
            Some(FrontScreen::WorldCreation(
                ui::world_creation::NewWorldDraft::new(0, 0),
            )),
            Some(FrontScreen::Keyboard(session())),
        ];
        let pages = [
            None,
            Some(P::Main),
            Some(P::ConfirmDeletePlayer),
            Some(P::ConfirmDeleteWorld),
            Some(P::ConfirmDiscard),
            Some(P::Credits),
        ];
        let clicks = [
            FlowClick::CharBack,
            FlowClick::CharNew,
            FlowClick::CharPlay(0),
            FlowClick::CharDelete(0),
            FlowClick::WorldBack,
            FlowClick::WorldPlay(0),
            FlowClick::WorldDelete(0),
            FlowClick::WorldNew(0, 0),
            FlowClick::WorldCreateBack,
            FlowClick::WorldCreate,
            FlowClick::ConfirmYes,
            FlowClick::ConfirmNo,
        ];
        for front in &fronts {
            for backup in &backs {
                for page in &pages {
                    // The invariant holds on the way in: the App never parks
                    // both sides, so front+page inputs are unreachable.
                    if front.is_some() && page.is_some() {
                        continue;
                    }
                    for click in &clicks {
                        let (front, backup, menu) =
                            next_flow_state(front.clone(), backup.clone(), *page, click.clone());
                        assert!(
                            !(front.is_some() && menu.is_some()),
                            "{click:?} from front {front:?}/page {page:?} left both sides live"
                        );
                        let _ = backup;
                    }
                }
            }
        }
    }

    #[test]
    fn dw_2_3_a_double_click_press_routes_to_play() {
        // `UICharacterListItem.cs:68-69`: the second press of the same card
        // inside 500 ms fires `PlayGame`. The machine's `DoubleClick` reaches
        // the screen through `App::pointer_event`, and the card table turns it
        // into the play flow (routing only — the port has no per-card selection
        // state, spec §9.4).
        let players = vec![entry("Alpha", true), entry("Beta", true)];
        let mut app = flow_app(Some(FrontScreen::CharSelect), players.clone(), Vec::new());
        // The 200 ms entry lockout from `enter_front` must be gone first.
        app.pointer.tick(300.0);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let l = ui::char_select::resolved_layout(screen, app.front_header_w("Select Player"), 2);
        let c0 = ui::char_select::card_layout(l.list, 0);
        // The first press on the card body: a plain selection, no card action.
        app.mouse_pos = Vec2::new(c0.card[0] + c0.card[2] * 0.5, c0.card[1] + c0.card[3] * 0.5);
        app.pointer_event = Some(ui::pointer::PointerEvent::Down);
        app.handle_front_click();
        assert!(app.front_hint.is_none());
        // The second press, marked `DoubleClick` by the machine: the play flow.
        app.pointer_event = Some(ui::pointer::PointerEvent::DoubleClick);
        app.handle_front_click();
        assert!(matches!(app.front, Some(FrontScreen::WorldSelect)));
    }

    #[test]
    fn tr_ui_names_map_the_hook_table() {
        // The plan Task 5 hook list (the `TR_PAGE` pattern); unknown names
        // resolve to nothing and the match is case-insensitive.
        assert_eq!(tr_ui_preset("charsel"), Some(TrUiPreset::CharSelect));
        assert_eq!(tr_ui_preset("worldsel"), Some(TrUiPreset::WorldSelect));
        assert_eq!(tr_ui_preset("charnew"), Some(TrUiPreset::CharNew));
        assert_eq!(tr_ui_preset("worldnew"), Some(TrUiPreset::WorldNew));
        assert_eq!(tr_ui_preset("vk"), Some(TrUiPreset::Keyboard));
        assert_eq!(tr_ui_preset("credits"), Some(TrUiPreset::Credits));
        assert_eq!(tr_ui_preset("delplayer"), Some(TrUiPreset::DelPlayer));
        assert_eq!(tr_ui_preset("delworld"), Some(TrUiPreset::DelWorld));
        assert_eq!(tr_ui_preset("discard"), Some(TrUiPreset::Discard));
        assert_eq!(tr_ui_preset("CHARSEL"), Some(TrUiPreset::CharSelect));
        assert_eq!(tr_ui_preset("nonsense"), None);
    }

    // --- App-level click tests (Review Focus 1's regression) -----------------
    //
    // `next_flow_state` is pure — the bug these tests pin was in the
    // *application* of its result: `handle_front_click` set `menu` but never
    // the table's `front`, so the opened page lived under the still-live
    // screen.

    /// `frame()`'s clock advance: the pointer machine's 200 ms state-change
    /// lockout decays with the frame step (`UserInterface.cs:203-204`), which
    /// `frame()` calls through [`App::tick_pointer_clock`] on **every** frame —
    /// whichever UI side is live, the C#'s `Main.UpdateUIStates` reach.
    fn frames(app: &mut App, n: usize) {
        for _ in 0..n {
            app.anim_time += 1.0 / 60.0;
            app.tick_pointer_clock(1.0 / 60.0);
        }
    }

    /// `app/loop.rs:403-435`'s front-screen `MouseInput` arm — the production
    /// press path: the pointer machine only accepts the press outside the
    /// state-change lockout, then the click gate runs the screen's table
    /// either way (the C# skips `HandleClick` while the lockout is live).
    fn front_press(app: &mut App) {
        let now = app.anim_time as f64 * 1000.0;
        let hit = app.front_hit();
        if !app.pointer.clicks_disabled() {
            app.pointer_event = app
                .pointer
                .press(now, ui::pointer::PointerButton::Left, hit);
        }
        app.handle_front_click();
    }

    /// The machine's release edge (`app/loop.rs:430-434`) — ungated in the C#,
    /// so the release must be fed when the press was dropped.
    fn front_release(app: &mut App) {
        let now = app.anim_time as f64 * 1000.0;
        let hit = app.front_hit();
        app.pointer_event = app
            .pointer
            .release(now, ui::pointer::PointerButton::Left, hit);
    }

    /// A centre point inside `rect`, aimed the way the tests point the mouse.
    fn centre(rect: [f32; 4]) -> Vec2 {
        Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
    }

    /// The world screen's card geometry and its hit table, built exactly as the
    /// click handler (`app/flow.rs:765-775`) does.
    fn world_screen(app: &App) -> (Vec2, ui::char_select::Layout, Vec<bool>) {
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let l = ui::world_select::resolved_layout(
            screen,
            app.front_header_w("Select World"),
            app.worlds.len(),
        );
        let favorites: Vec<bool> = app.worlds.iter().map(|w| w.is_favorite).collect();
        (screen, l, favorites)
    }

    /// Enters world select through the real opener and pins the listed worlds
    /// to the fixture. `enter_front` runs `OnActivate`'s re-list from the
    /// player's real `Worlds` directory (`app/mod.rs:1105-1113`), which other
    /// tests leave files in — the click table must read the fixture, not
    /// whatever the disk held, so the list is overwritten after the entry.
    fn open_world_select(app: &mut App, worlds: Vec<ui::save_list::WorldEntry>) {
        app.enter_front(FrontScreen::WorldSelect);
        app.worlds = worlds;
    }

    #[test]
    fn world_select_play_answers_after_the_entry_lockout_expires() {
        // The owner's report: world select opens and every row is dead. The
        // entry arms the 200 ms state-change disable
        // (`UserInterface.cs:178` through `App::enter_front`), and the countdown
        // that clears it used to sit inside the gameplay-only branch of
        // `frame()`, so `clicks_disabled()` stayed true forever.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("W1")]);
        let (screen, l, favorites) = world_screen(&app);
        let c0 = ui::world_select::card_layout(l.list, 0, false);
        let play = centre(c0.play);
        app.mouse_pos = play;
        assert_eq!(
            ui::world_select::card_hit(&l, &favorites, play, false).1,
            Some(ui::world_select::CardAction::Play(0)),
            "the play button's rect is the element the click table reads"
        );

        // Inside the window: the press is dropped, exactly as the C# skips
        // `HandleClick` while `_clickDisabledTimeRemaining > 0` (`:214`).
        assert!(
            app.pointer.clicks_disabled(),
            "the entry locks the first press (UserInterface.cs:178)"
        );
        front_press(&mut app);
        assert!(
            matches!(app.front, Some(FrontScreen::WorldSelect)),
            "a press inside the 200 ms window must not launch the world"
        );
        front_release(&mut app);

        // Past the window the screen answers: `PlayGame`
        // (`UIWorldListItem.cs:360-374`) routes the launch.
        frames(&mut app, 13);
        assert!(
            !app.pointer.clicks_disabled(),
            "the 200 ms lockout must expire while the front screen is live"
        );
        front_press(&mut app);
        assert_eq!(
            app.front, None,
            "`FlowClick::WorldPlay` closes both sides — the world launches"
        );
        let _ = screen;
    }

    #[test]
    fn world_select_card_body_and_back_answer_after_the_lockout() {
        // Both directions of the card table: the body's plain press is a
        // selection, not a launch (`card_hit`'s `(Some(card), None)`), and the
        // fat Back row hops to the character select
        // (`UIWorldSelect.cs:118-122` → `FlowClick::WorldBack`).
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("W1")]);
        frames(&mut app, 13);
        assert!(!app.pointer.clicks_disabled());

        let (_, l, favorites) = world_screen(&app);
        let body = centre(ui::world_select::card_layout(l.list, 0, false).card);
        app.mouse_pos = body;
        assert_eq!(
            ui::world_select::card_hit(&l, &favorites, body, false),
            (Some(0), None),
            "the card body is a plain selection"
        );
        front_press(&mut app);
        assert!(
            matches!(app.front, Some(FrontScreen::WorldSelect)),
            "selecting a card must not leave world select"
        );
        front_release(&mut app);

        let back = centre(l.back);
        app.mouse_pos = back;
        assert_eq!(
            ui::world_select::card_hit(&l, &favorites, back, false).1,
            Some(ui::world_select::CardAction::Back),
            "the Back row's element id is the table row the click reads"
        );
        front_press(&mut app);
        assert_eq!(
            app.front,
            Some(FrontScreen::CharSelect),
            "Back hops to the character select"
        );
    }

    #[test]
    fn world_select_play_gates_validity_and_the_journey_mismatch() {
        // `UIWorldListItem.PlayGame` (`:360-385`): `base._data.IsValid` then
        // `TryMovingToRejectionMenuIfNeeded`. The port surfaces `Main.statusText`
        // through `App::front_hint` (no `menuMode == 1000000` page) and must not
        // park a world.
        let reject = crate::lang::text("UI.PlayerIsCreativeAndWorldIsNotCreative");

        // A Journey player on a Classic world: refused, notice set, cue played.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("Classic")]);
        app.session_draft = Some(ui::char_creation::NewPlayerDraft {
            difficulty: 3,
            ..Default::default()
        });
        frames(&mut app, 13);
        assert!(!app.pointer.clicks_disabled());
        let (_, l, _) = world_screen(&app);
        app.mouse_pos = centre(ui::world_select::card_layout(l.list, 0, false).play);
        let before = app.audio.played_count();
        front_press(&mut app);
        assert!(
            matches!(app.front, Some(FrontScreen::WorldSelect)),
            "a Journey mismatch must not leave world select"
        );
        assert!(
            app.play_world.is_none(),
            "no world is parked on a rejection"
        );
        assert_eq!(
            app.front_hint.as_deref(),
            Some(reject.as_str()),
            "`Main.statusText` carries the source's rejection wording"
        );
        assert_eq!(
            app.audio.played_count(),
            before + 1,
            "the rejection plays `SoundEngine.PlaySound(10)`"
        );

        // The mirror: a Classic player on a Journey world.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        let mut journey = world_entry("Journey");
        journey.difficulty = 3;
        open_world_select(&mut app, vec![journey]);
        frames(&mut app, 13);
        let (_, l, _) = world_screen(&app);
        app.mouse_pos = centre(ui::world_select::card_layout(l.list, 0, false).play);
        front_press(&mut app);
        assert!(matches!(app.front, Some(FrontScreen::WorldSelect)));
        assert_eq!(
            app.front_hint.as_deref(),
            Some(crate::lang::text("UI.PlayerIsNotCreativeAndWorldIsCreative").as_str())
        );

        // An invalid world's Play is a silent dead click (`:362`): no notice,
        // no cue, no launch.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        let mut broken = world_entry("Broken");
        broken.valid = false;
        open_world_select(&mut app, vec![broken]);
        frames(&mut app, 13);
        let (_, l, _) = world_screen(&app);
        app.mouse_pos = centre(ui::world_select::card_layout(l.list, 0, false).play);
        let before = app.audio.played_count();
        front_press(&mut app);
        assert!(matches!(app.front, Some(FrontScreen::WorldSelect)));
        assert!(app.play_world.is_none());
        assert!(app.front_hint.is_none(), "an invalid world notices nothing");
        assert_eq!(app.audio.played_count(), before);

        // The agreeing pair still launches (Classic + Classic).
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("Classic")]);
        frames(&mut app, 13);
        let (_, l, _) = world_screen(&app);
        app.mouse_pos = centre(ui::world_select::card_layout(l.list, 0, false).play);
        front_press(&mut app);
        assert_eq!(
            app.front, None,
            "a joinable world launches (`FlowClick::WorldPlay`)"
        );
    }

    #[test]
    fn world_select_rename_row_opens_the_keyboard_after_the_lockout() {
        // The owner's dead Rename row: `RenameButtonClick`
        // (`UIWorldListItem.cs:415-421`) opens the world-naming keyboard.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("W1")]);
        frames(&mut app, 13);
        assert!(!app.pointer.clicks_disabled());
        let (_, l, _) = world_screen(&app);
        let c0 = ui::world_select::card_layout(l.list, 0, false);
        app.mouse_pos = centre(c0.rename);
        front_press(&mut app);
        assert!(
            matches!(app.front, Some(FrontScreen::Keyboard(_))),
            "Rename opens the world-naming keyboard"
        );
    }

    #[test]
    fn world_select_delete_row_parks_the_confirm_page_after_the_lockout() {
        // The owner's dead Delete row: `DeleteButtonClick`
        // (`UIWorldListItem.cs:346-358`) parks the target, then `menuMode = 9`.
        // The row is gated on the entry having a file (`Main.EraseWorld` needs
        // one), so the fixture entry carries a path (`app/flow.rs:845`).
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("W1")]);
        for w in &mut app.worlds {
            w.path = Some(std::env::temp_dir().join("terrustaria-delete-gate.trwld"));
        }
        frames(&mut app, 13);
        assert!(!app.pointer.clicks_disabled());
        let (_, l, _) = world_screen(&app);
        let c0 = ui::world_select::card_layout(l.list, 0, false);
        app.mouse_pos = centre(c0.delete);
        front_press(&mut app);
        assert!(
            matches!(app.menu, Some(ref m) if m.page == menu::Page::ConfirmDeleteWorld),
            "Delete parks the mode-9 confirm page"
        );
    }

    #[test]
    fn world_select_new_and_favorite_rows_answer_after_the_lockout() {
        // The owner's New and Favorite rows: `NewWorldClick`
        // (`UIWorldSelect.cs:110-116`) opens the creator with the stand-in
        // name, `FavoriteButtonClick` (`UIWorldListItem.cs:439-458`) toggles
        // the list's favourite marker. The fixture entry keeps `path: None`,
        // so the toggle writes no `.trfav` sidecar anywhere.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("W1")]);
        frames(&mut app, 13);
        assert!(!app.pointer.clicks_disabled());
        let (_, l, favorites) = world_screen(&app);
        assert!(!favorites[0]);

        let c0 = ui::world_select::card_layout(l.list, 0, false);
        app.mouse_pos = centre(c0.favorite);
        front_press(&mut app);
        assert!(
            app.worlds[0].is_favorite,
            "Favorite toggles the entry's marker (`ToggleFavorite`)"
        );
        front_release(&mut app);

        app.mouse_pos = centre(l.new);
        front_press(&mut app);
        assert!(
            matches!(app.front, Some(FrontScreen::WorldCreation(_))),
            "New opens the world creator"
        );
    }

    #[test]
    fn entry_lockout_expires_on_a_live_front_screen() {
        // The regression itself, stripped of the click table: the countdown
        // must advance while the 888 side is live. It used to sit inside
        // `frame()`'s `menu.is_none() && front.is_none()` branch, which a live
        // front screen makes unreachable — every button of every front screen
        // was then permanently dead.
        let mut app = flow_app(None, Vec::new(), Vec::new());
        open_world_select(&mut app, vec![world_entry("W1")]);
        assert!(app.pointer.clicks_disabled());
        assert!(
            app.menu.is_none() && app.front.is_some(),
            "the regime the bug lives in: a live front screen, no menu"
        );
        frames(&mut app, 13);
        assert!(!app.pointer.clicks_disabled());
    }

    /// One list entry for the flow tests: playable, optionally deletable (the
    /// `DeleteButtonClick` gate, `UICharacterListItem.cs:242-254`).
    fn entry(name: &str, deletable: bool) -> ui::save_list::PlayerEntry {
        ui::save_list::PlayerEntry {
            name: name.into(),
            difficulty: 0,
            life_max: 100,
            mana_max: 20,
            is_favorite: false,
            playable: true,
            deletable,
            path: None,
            appearance: ui::save_list::Appearance::default(),
            inv_favorites: 0,
        }
    }

    /// A headless App for the flow tests: no window/GPU/renderer, the audio
    /// pair without a content dir, an 8x8 world — everything the flow state
    /// does not read stays at its cheap default. `screen_size()` falls back
    /// to the deterministic 1280x720, so the tests aim the pointer through
    /// the same layout the click handler builds.
    pub(crate) fn flow_app(
        front: Option<FrontScreen>,
        players: Vec<ui::save_list::PlayerEntry>,
        worlds: Vec<ui::save_list::WorldEntry>,
    ) -> App {
        let world = World::new("flow".into(), 8, 8);
        let liquids = LiquidSystem::new(&world);
        App {
            ambient_waterfall: (0.0, 0.0, 0.0),
            ambient_lavafall: (0.0, 0.0, 0.0),
            ambient_counter: 0.0,
            bestiary: tr_core::bestiary::WorldBestiary::new(),
            town_rooms: crate::town::TownRooms::new(),
            shimmered_npcs: crate::town::ShimmeredTownNpcs::new(),
            bestiary_open: false,
            bestiary_selected: None,
            chat: crate::chat::ChatMonitor::default(),
            achievements: crate::achievements::AchievementManager::registry(),
            achievements_open: crate::env::achievements_open(),
            achievements_scroll: 0.0,
            achievements_categories: [true; 4],
            emotes: crate::emotes::EmoteSystem::default(),
            banners: crate::banners::BannerSystem::default(),
            town_look_timer: 0.0,
            town_arrival: crate::town_arrival::ArrivalState::new(),
            reuse_delay: 0.0,
            window: None,
            gpu: None,
            renderer: None,
            post: None,
            noise_tex: None,
            water_mask: None,
            scene_state: tr_core::scene::SceneState::new(),
            sandstorm: tr_core::scene::Sandstorm::new(1),
            filter_driver: crate::app::filters::FilterDriver::new(),
            atlas: None,
            static_sheets: None,
            fonts_cache: None,
            font_atlas: None,
            world,
            lookups: assets::Lookups::default(),
            clouds: Vec::new(),
            sky: SkyAssets::default(),
            bg_style: 0,
            bg_delay: 0,
            bg_instant: 0,
            bg_alpha_front: [0.0; 16],
            bg_alpha_far: [0.0; 16],
            stars: Vec::new(),
            player_assets: PlayerAssets::default(),
            player: None,
            follow_player: false,
            player_step: FixedStepper::new(1.0 / 60.0, 8),
            npc_step: FixedStepper::new(1.0 / 60.0, 8),
            npcs: Vec::new(),
            npc_assets: NpcAssets::default(),
            npc_timer: 0.0,
            next_worm_id: 1,
            shop_npc: None,
            shop_stock: Vec::new(),
            in_reforge_menu: false,
            reforge_slot: crate::item::EMPTY_SLOT,
            reforge_cooldown: 0,
            revenge_markers: Vec::new(),
            revenge_tick: 0,
            shop_mood: 1.0,
            discount_available: false,
            shop_closed_flag: false,
            creative_powers: crate::creative::CreativePowers::new(),
            debug_panel: crate::debug_panel::DebugPanel::new(),
            favorite_slots: Vec::new(),
            play_world: None,
            invasion_spawn_timer: 0.0,
            npc_rng: 0,
            spawn_no_cycle: false,
            attack_cd: 0.0,
            inventory: std::array::from_fn(|_| EMPTY_SLOT),
            equip: inv::EquipSlots::empty(),
            equip_page: 0,
            // `Main.EquipPageSelected` (`Main.cs:42846-42858`) — the page the
            // player picked with the tabs, which `resolve_equip_page()` feeds
            // back into `equip_page` every frame.
            equip_page_selected: 0,
            selected: 0,
            hotbar_scales: [0.75; HOTBAR],
            inventory_open: false,
            crafting_open: false,
            world_chests: Vec::new(),
            chest_scroll_row: 0,
            chest_button_scales: [0.0; 7],
            chest_button_hovered: [false; 7],
            stack_split: 0,
            stack_delay: 0,
            stack_counter: 0,
            super_fast_stack: 0,
            times_tried_to_fast_stack: 0,
            cursor_override: -1,
            chest_default_name: String::new(),
            text_blinker: ui::chest::TextBlinker::default(),
            // The pre-world bootstrap stream; crafting needs a loaded world —
            // `enter_loaded_world` re-seeds it from the world seed
            // (`app/mod.rs`).
            craft_rng: tr_core::unified_random::UnifiedRandom::new(1),
            menu: None,
            front,
            front_backup: None,
            players,
            worlds,
            front_hint: None,
            front_hover_hit: None,
            front_mouse_down: false,
            session_draft: None,
            vk_grid: false,
            settings: settings::Settings::default(),
            player_name: "flow".into(),
            net: NetRole::Off,
            remote_players: HashMap::new(),
            local_id: 0,
            net_timer: 0.0,
            net_state: crate::net::client_state::NOT_CONNECTED,
            net_pending_sections: Vec::new(),
            net_info: None,
            net_status_text: String::new(),
            net_status_max: 0,
            net_status_count: 0,
            net_last_recv: std::time::Instant::now(),
            net_dropped: false,
            net_npcs: std::collections::HashMap::new(),
            net_npc_seen: Vec::new(),
            net_npcs_seen_at: std::collections::HashMap::new(),
            net_npcs_dirty: false,
            net_npc_gen: std::collections::HashMap::new(),
            net_items: std::collections::HashMap::new(),
            net_items_dirty: false,
            net_item_last: std::collections::HashMap::new(),
            net_reserved: std::collections::HashMap::new(),
            net_ps_last: crate::net::PlayerState::default(),
            net_beat: 0,
            net_module_beat: 0,
            net_proj_last: std::collections::HashMap::new(),
            net_proj_spam: std::collections::HashMap::new(),
            net_npc_slots: Vec::new(),
            net_ping_at: None,
            net_rtt_ms: 0,
            join_target: None,
            map: None,
            map_grab: None,
            minimap_hover: None,
            map_style: 1,
            // `Main.teamCooldown` / `Main.hidePVPAndTeamIcons` start clear
            // (`Main.cs:66781` sets only `teamCooldownLen`).
            team_cooldown: 0,
            hide_pvp_and_team_icons: false,
            pvp_icons_forced: false,
            audio: audio::Audio::new(None),
            music: audio::MusicSystem::new(None),
            last_hp: 100,
            autosave: 0.0,
            mouse_item: EMPTY_SLOT,
            ui_left_press: false,
            ui_right_press: false,
            pointer: ui::pointer::PointerState::default(),
            pointer_ms: 0.0,
            pointer_event: None,
            mouse_pos: Vec2::ZERO,
            cursor: ui::cursor::CursorState::default(),
            os_cursor_hidden: None,
            mine_down: false,
            place_down: false,
            mine_target: None,
            mine_timer: 0.0,
            place_prev_down: false,
            place_released: false,
            mine_prev_down: false,
            mine_released: false,
            item_time: 0.0,
            edit_target: None,
            smart_cursor: Default::default(),
            smart_cursor_used: false,
            ingame_options: crate::ui::ingame_options::IngameOptions::new(),
            world_path: PathBuf::new(),
            dev_break: None,
            dev_place: Vec::new(),
            camera: Vec2::ZERO,
            camera_state: Default::default(),
            gfx_quality: Default::default(),
            zoom: 1.0,
            zoom_pinned: false,
            debug: false,
            keys: HashSet::new(),
            keybinds: crate::input_map::default_binds(),
            map_binds: crate::input_map::default_map_binds(),
            mount_prev: false,
            mouse_down: false,
            last_mouse: Vec2::ZERO,
            last_frame: Instant::now(),
            frame_history: VecDeque::new(),
            last_stats: tr_render::sprite::RenderStats::default(),
            bench_frames: None,
            bench_samples: Vec::new(),
            frame_count: 0,
            shot_path: None,
            shot_at: 0,
            atlas_stats: (0, 0, 0, 0),
            light: LightMap::new(),
            scratch: DrawScratch::default(),
            tile_stats: TileDrawStats::default(),
            stats_on: false,
            stats_acc: StatsAcc::default(),
            debug_cache: DebugCache::default(),
            light_mode: LightMode::Color,
            mouse_text_color: 225,
            mouse_text_color_dir: 1,
            demon_torch: 1.0,
            demon_torch_dir: 1.0,
            disco: [255, 0, 0],
            disco_style: 0,
            light_rng_seed: 1,
            anim_time: 0.0,
            has_liquids: false,
            liquids,
            liquid_step: FixedStepper::new(1.0 / 60.0, 8),
            liquid_tick: 0,
            waterfall: tr_core::waterfall::WaterfallManager::new(),
            waterfall_step: FixedStepper::new(1.0 / 60.0, 8),
            wiring: tr_core::wiring::Wiring::new(),
            wiring_step: FixedStepper::new(1.0 / 60.0, 8),
            te_step: FixedStepper::new(1.0 / 60.0, 8),
            growth_step: FixedStepper::new(1.0 / 60.0, 8),
            growth_rng: tr_core::tile_light::FastRandom::new(0),
            ore_tiers: tr_core::ores::SavedOreTiers::new(),
            rain: Rain::new(1),
            sand: crate::sand::Sand::new(1),
            minecart_sparks: crate::minecart::Sparks::new(),
            wind: tr_core::wind::Wind::new(1),
            world_tick_step: FixedStepper::new(1.0 / 60.0, 8),
            atmo: 1.0,
            scene_metrics: tr_core::scene::SceneMetrics::new(),
            ambience: crate::ambience::Ambience::new(),
            ambience_rng: 1,
            tree_wind_counter: 0.0,
            grass_wind_counter: 0.0,
            sunflower_wind_counter: 0.0,
            sunflower_wind_step: FixedStepper::new(1.0 / 60.0, 8),
            leaves: Vec::new(),
            liquid_alpha: [0.0; 15],
            items: Vec::new(),
            item_rng: 0,
            prefix_rng: tr_core::UnifiedRandom::new(0),
            rngs: NamedRngs::new(0),
            item_step: FixedStepper::new(1.0 / 60.0, 8),
            swing: 0.0,
            swing_style: 0,
            swing_aim: 0.0,
            shot_fired: false,
            swing_max: 0.0,
            coins: 0,
            projectiles: crate::projectile::ProjectilePool::new(),
            tomb_falls: Vec::new(),
            lost_coins: 0,
            lost_coin_string: String::new(),
            death_text: String::new(),
            respawn_skip_frames: 0,
            respawn_skip_available: false,
            invasion_progress: -1,
            invasion_progress_max: 0,
            invasion_progress_icon: 0,
            invasion_progress_wave: 0,
            invasion_progress_display_left: 0,
            invasion_progress_alpha: 0.0,
            invasion_progress_near: false,
            tomb_texts: Vec::new(),
            projectile_step: FixedStepper::new(1.0 / 60.0, 8),
            dev_throw: false,
            summon_shot: false,
            world_key: None,
            hell_black_box_bottom: 0.0,
        }
    }

    #[test]
    fn back_click_lands_the_legacy_main_page_with_the_front_cleared() {
        // The Back row's `menuMode = 0` hop (`UICharacterSelect.cs:116-120`
        // through the table) must clear the 888 side (`Main.cs:51066-51069`'s
        // else branch): the port used to leave `front` live, so the Main page
        // was invisible under the screen and the invariant broke.
        let mut app = flow_app(
            Some(FrontScreen::CharSelect),
            vec![entry("flow", false)],
            vec![],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(&ui::char_select::select_player()),
            app.players.len(),
        );
        app.mouse_pos = Vec2::new(l.back[0] + l.back[2] * 0.5, l.back[1] + l.back[3] * 0.5);
        app.handle_front_click();
        // Review Focus 1's invariant: exactly one side is live, and it is the
        // legacy page.
        assert!(app.front.is_none(), "the 888 state clears");
        let m = app.menu.as_ref().expect("the legacy page opens");
        assert_eq!(m.page, P::Main);
    }

    #[test]
    fn delete_click_opens_the_mode_5_confirm_with_the_front_cleared() {
        // `DeleteButtonClick` (`UICharacterListItem.cs:242-254`) -> `menuMode
        // = 5`: the confirm opens alone, parking the target name/index
        // (`Main.selectedPlayer`).
        let mut app = flow_app(
            Some(FrontScreen::CharSelect),
            vec![entry("Bob", true)],
            vec![],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(&ui::char_select::select_player()),
            app.players.len(),
        );
        let delete = ui::char_select::card_layout(l.list, 0).delete;
        app.mouse_pos = Vec2::new(delete[0] + delete[2] * 0.5, delete[1] + delete[3] * 0.5);
        app.handle_front_click();
        assert!(app.front.is_none(), "mode 5 clears the 888 state");
        let m = app.menu.as_ref().expect("the confirm opens");
        assert_eq!(m.page, P::ConfirmDeletePlayer);
        assert_eq!(m.confirm_name.as_deref(), Some("Bob"));
        assert_eq!(m.confirm_index, 0);
    }

    #[test]
    fn world_delete_click_opens_the_mode_9_confirm_with_the_front_cleared() {
        // `DeleteButtonClick` (`UIWorldListItem.cs:346-358`) -> `menuMode = 9`
        // — the world twin of the mode-5 hop.
        let mut app = flow_app(
            Some(FrontScreen::WorldSelect),
            vec![],
            vec![ui::save_list::WorldEntry {
                name: "World 1".into(),
                difficulty: 0,
                size_id: 0,
                is_favorite: false,
                valid: true,
                has_been_played: false,
                path: Some(PathBuf::from("World 1.trwld")),
            }],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(&ui::world_select::select_world()),
            app.worlds.len(),
        );
        let delete = ui::world_select::card_layout(l.list, 0, false).delete;
        app.mouse_pos = Vec2::new(delete[0] + delete[2] * 0.5, delete[1] + delete[3] * 0.5);
        app.handle_front_click();
        assert!(app.front.is_none(), "mode 9 clears the 888 state");
        let m = app.menu.as_ref().expect("the confirm opens");
        assert_eq!(m.page, P::ConfirmDeleteWorld);
        assert_eq!(m.confirm_name.as_deref(), Some("World 1"));
        assert_eq!(m.confirm_index, 0);
    }

    // --- Task 8: the world creator's flow hops --------------------------------

    /// One fake world entry for the creator's flow tests.
    fn world_entry(name: &str) -> ui::save_list::WorldEntry {
        ui::save_list::WorldEntry {
            name: name.into(),
            difficulty: 0,
            size_id: 1,
            is_favorite: false,
            valid: true,
            has_been_played: false,
            path: None,
        }
    }

    fn world_creation_app() -> App {
        flow_app(
            Some(FrontScreen::WorldCreation(
                ui::world_creation::NewWorldDraft::new(1, 0),
            )),
            vec![],
            vec![world_entry("World 1")],
        )
    }

    /// The click handler's layout at the flow tests' deterministic 1280x720.
    fn world_creation_layout(app: &App) -> ui::world_creation::Layout {
        let (sw, sh) = app.screen_size();
        ui::world_creation::layout(Vec2::new(sw, sh), 0.0)
    }

    #[test]
    fn world_new_click_opens_the_creator_with_the_stand_in_name() {
        // `NewWorldClick` (`UIWorldSelect.cs:110-116`): the stand-in name
        // reads `WorldList.Count + 1` (`:113`) — two listed worlds name the
        // draft "World 3".
        let mut app = flow_app(
            Some(FrontScreen::WorldSelect),
            vec![],
            vec![world_entry("Alpha"), world_entry("Beta")],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(&ui::world_select::select_world()),
            app.worlds.len(),
        );
        let new = l.new;
        app.mouse_pos = Vec2::new(new[0] + new[2] * 0.5, new[1] + new[3] * 0.5);
        app.handle_front_click();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the New click opens the world creator");
        };
        assert_eq!(draft.name, "World 3");
        // `SetDefaultOptions` (`UIWorldCreation.cs:313-326`): the Classic
        // session player seeds Normal/Medium/Random.
        assert_eq!(draft.size, 1);
        assert_eq!(draft.difficulty, 0);
        assert_eq!(draft.evil, 0);
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn resort_worlds_uses_the_active_players_difficulty() {
        // `UpdateWorldsList`'s first key (`UIWorldSelect.cs:156`) is
        // `CanWorldBeJoinedByActivePlayer` (`:177-180`): the active player's
        // Journey flag (difficulty 3) must match the world's GameMode 3.
        // `enter_front`'s re-list and the favourite toggle both ride
        // `App::resort_worlds`, so the key is no longer the hardcoded `false`.
        fn world(name: &str, difficulty: u8) -> ui::save_list::WorldEntry {
            ui::save_list::WorldEntry {
                difficulty,
                ..world_entry(name)
            }
        }
        // A Classic session player (`difficulty 0`, the default): the Classic
        // world joins, the Journey world does not, so Classic sorts first.
        let mut app = flow_app(
            Some(FrontScreen::WorldSelect),
            vec![],
            vec![world("Journey", 3), world("Classic", 0)],
        );
        app.resort_worlds();
        let names: Vec<&str> = app.worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["Classic", "Journey"],
            "a Classic player joins Classic worlds first"
        );
        // A Journey session player (`Main.ActivePlayerFileData.Player.difficulty
        // == 3`): the Journey world sorts first.
        app.session_draft = Some(ui::char_creation::NewPlayerDraft {
            difficulty: 3,
            ..Default::default()
        });
        app.worlds = vec![world("Classic", 0), world("Journey", 3)];
        app.resort_worlds();
        let names: Vec<&str> = app.worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["Journey", "Classic"],
            "a Journey player joins Journey worlds first"
        );
    }

    #[test]
    fn world_creation_back_click_reopens_world_select() {
        // `Click_GoBack` -> `GoBack` -> `OpenWorldSelectUI`
        // (`UIWorldCreation.cs:665-674`) — no discard confirm on this screen.
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(l.back[0] + l.back[2] * 0.5, l.back[1] + l.back[3] * 0.5);
        app.handle_front_click();
        assert_eq!(app.front, Some(FrontScreen::WorldSelect));
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn world_creation_option_clicks_update_the_draft() {
        // `ClickSizeOption`/`ClickDifficultyOption`/`ClickEvilOption`/
        // `ClickRandomizeName` (`UIWorldCreation.cs:497-575`): the picks land
        // in the draft; the display order maps Journey/Classic/Expert/Master
        // to the ids 3/0/1/2.
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        app.mouse_pos = centre(l.difficulty_buttons[2]); // Expert
        app.handle_front_click();
        app.mouse_pos = centre(l.size_buttons[2]); // Large
        app.handle_front_click();
        app.mouse_pos = centre(l.evil_buttons[1]); // Corruption
        app.handle_front_click();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the options clicked on the live creator");
        };
        assert_eq!(draft.difficulty, 1);
        assert_eq!(draft.size, 2);
        assert_eq!(draft.evil, 1);
        // The randomize stand-in re-derives the name from the list count.
        app.mouse_pos = centre(l.random_name_icon);
        app.handle_front_click();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(draft.name, "World 2"); // `worlds.len() = 1`
    }

    #[test]
    fn world_creation_name_click_opens_the_naming_keyboard() {
        // `Click_SetName` (`UIWorldCreation.cs:689-695`): `Lang.menu[48]`,
        // empty start, `allowEmpty: true`, the 27 cap; the tail lands the
        // trimmed name back in the draft (`OnFinishedSettingName`, `:726-731`).
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.name_button[0] + l.name_button[2] * 0.5,
            l.name_button[1] + l.name_button[3] * 0.5,
        );
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the name plate opens the keyboard");
        };
        assert_eq!(session.request.label, ui::keyboard::enter_world_name());
        assert!(session.request.initial.is_empty(), "the box opens empty");
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, 27);
        assert_eq!(session.tail, KeyboardTail::WorldName);
        // The submit tail: the name lands, the creator returns (`:730`).
        app.keyboard_character('P');
        app.keyboard_submit();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the creator returns");
        };
        assert_eq!(draft.name, "P");
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn world_creation_seed_click_opens_the_seed_keyboard() {
        // `OpenSeedInputMenu` (`UIWorldCreation.cs:697-708`): "UI.EnterSeed",
        // `allowEmpty: true`, `int.MaxValue`; the seed plate and the icon
        // both route here (the Advanced tab is out of scope).
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.seed_plate[0] + l.seed_plate[2] * 0.5,
            l.seed_plate[1] + l.seed_plate[3] * 0.5,
        );
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the seed plate opens the keyboard");
        };
        assert_eq!(session.request.label, "Enter Seed (Leave Blank For Random)");
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, usize::MAX);
        assert_eq!(session.tail, KeyboardTail::WorldSeed);
        // The submit tail: the trimmed seed lands (`OnFinishedSettingSeed`,
        // `:817-819`), the creator returns.
        app.keyboard_character('5');
        app.keyboard_character('1');
        app.keyboard_submit();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the creator returns");
        };
        assert_eq!(draft.seed, "51");
    }

    #[test]
    fn world_creation_create_with_an_empty_name_opens_the_naming_keyboard() {
        // `Click_NamingAndCreating` (`UIWorldCreation.cs:710-719`): the empty
        // branch opens `Lang.menu[48]` with `allowEmpty: false`; the filled
        // branch's save is covered by the module-level temp-dir test (the
        // App click would write the real world directory).
        let mut app = world_creation_app();
        if let Some(FrontScreen::WorldCreation(draft)) = app.front.as_mut() {
            // The stand-in name is non-empty (`SetDefaultOptions`'s
            // `AssignRandomWorldName`); the empty branch needs the C#'s
            // `string.IsNullOrEmpty(this._optionwWorldName)` (`:713`).
            draft.name = String::new();
        }
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.create[0] + l.create[2] * 0.5,
            l.create[1] + l.create[3] * 0.5,
        );
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the empty-name Create opens the naming keyboard");
        };
        assert!(!session.request.allow_empty);
        assert_eq!(session.tail, KeyboardTail::WorldNameAndCreate);
        assert!(app.menu.is_none(), "the invariant holds");
    }

    // --- Task 6: the keyboard's flow hops ------------------------------------

    /// An App with the keyboard front live over the char select — the session
    /// `RenameButtonClick` builds (`UICharacterListItem.cs:196-206`).
    fn keyboard_app() -> App {
        let request = ui::keyboard::Request {
            label: ui::keyboard::enter_character_name(),
            initial: "Bob".into(),
            allow_empty: true,
            max_length: 20,
            context: ui::keyboard::Context::Text,
        };
        let state = ui::keyboard::KeyboardState::new(&request);
        flow_app(
            Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
                request,
                state,
                back: Some(FrontScreen::CharSelect),
                tail: KeyboardTail::Rename,
            }))),
            vec![entry("Bob", false)],
            vec![],
        )
    }

    #[test]
    fn rename_click_opens_the_keyboard_session() {
        // `RenameButtonClick` (`:196-206`): `Lang.menu[45]`, the card's name,
        // `allowEmpty: true`, the ctor's 20-character cap; the cancel target
        // is the screen the keyboard came from.
        let mut app = flow_app(
            Some(FrontScreen::CharSelect),
            vec![entry("Bob", false)],
            vec![],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(&ui::char_select::select_player()),
            app.players.len(),
        );
        let rename = ui::char_select::card_layout(l.list, 0).rename;
        app.mouse_pos = Vec2::new(rename[0] + rename[2] * 0.5, rename[1] + rename[3] * 0.5);
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the rename click opens the keyboard");
        };
        assert_eq!(session.request.label, ui::keyboard::enter_character_name());
        assert_eq!(session.request.initial, "Bob");
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, 20);
        assert_eq!(session.back, Some(FrontScreen::CharSelect));
        // The state seeds from the request (`UIVirtualKeyboard.cs:429-433`) —
        // a non-empty start stays Default.
        assert_eq!(session.state.text, "Bob");
        assert_eq!(session.state.key_state, ui::keyboard::KeyState::Default);
        // Review Focus 1: the menu side stayed closed while the keyboard is up.
        assert!(app.menu.is_none());
    }

    #[test]
    fn keyboard_cancel_click_reopens_the_parked_screen() {
        // `_cancelAction` -> `GoBackHere` (`:214-216`): the reopen, no notice.
        let mut app = keyboard_app();
        let (sw, sh) = app.screen_size();
        let l = ui::keyboard::layout(Vec2::new(sw, sh), false);
        app.mouse_pos = Vec2::new(
            l.cancel[0] + l.cancel[2] * 0.5,
            l.cancel[1] + l.cancel[3] * 0.5,
        );
        app.handle_front_click();
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_hint.is_none(), "cancel notices nothing");
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn keyboard_submit_reopens_with_the_not_ported_notice() {
        // The rename tail (`:205-213`) reopens the screen; the port's rename
        // itself is the notice (no save-file writer, spec §9.4).
        let mut app = keyboard_app();
        app.keyboard_submit();
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_hint.is_some(), "the submit tail notices");
    }

    #[test]
    fn keyboard_enter_gates_on_the_validation() {
        // `:763-766`: `Main.inputTextEnter && CanSubmit` — an empty text with
        // `allowEmpty: false` keeps the keyboard up; typing fixes it.
        let request = ui::keyboard::Request {
            label: ui::keyboard::enter_character_name(),
            initial: String::new(),
            allow_empty: false,
            max_length: 20,
            context: ui::keyboard::Context::Text,
        };
        let state = ui::keyboard::KeyboardState::new(&request);
        let mut app = flow_app(
            Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
                request,
                state,
                back: Some(FrontScreen::CharSelect),
                tail: KeyboardTail::Rename,
            }))),
            vec![],
            vec![],
        );
        // The empty start seeds Shift (`:430-433`).
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the keyboard is live");
        };
        assert_eq!(session.state.key_state, ui::keyboard::KeyState::Shift);
        app.keyboard_enter();
        assert!(
            matches!(app.front, Some(FrontScreen::Keyboard(_))),
            "the invalid Enter is dead"
        );
        // The physical path never unlatches the seeded Shift (`:778-788`).
        app.keyboard_character('N');
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the keyboard stays live");
        };
        assert_eq!(session.state.text, "N");
        assert_eq!(session.state.key_state, ui::keyboard::KeyState::Shift);
        app.keyboard_enter();
        assert_eq!(
            app.front,
            Some(FrontScreen::CharSelect),
            "the valid Enter submits"
        );
    }

    #[test]
    fn tr_ui_vk_presets_the_keyboard_now() {
        // Task 5 parsed `vk` without presetting; Task 6's variant is live.
        let mut app = flow_app(None, vec![], vec![]);
        app.preset_ui(TrUiPreset::Keyboard);
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("TR_UI=vk presets the keyboard");
        };
        assert_eq!(session.request.label, ui::keyboard::enter_character_name());
        assert!(
            session.state.text.is_empty(),
            "the fresh name is empty (`Player.cs:826`)"
        );
        assert_eq!(
            session.state.key_state,
            ui::keyboard::KeyState::Shift,
            "the empty text seeds Shift (`UIVirtualKeyboard.cs:430-433`)"
        );
        assert_eq!(session.back, Some(FrontScreen::CharSelect));
    }

    #[test]
    fn the_chest_keyboards_save_click_submits_the_empty_text() {
        // `:358-365`'s gate with the ctx-2 `TextIsValidForSubmit`
        // (`:615-626` — `_editingChest` lifts the empty rule): the empty
        // session text submits, and the `ChestUI.RenameChestSubmit` +
        // `Close(quiet: true)` tail (`IngameFancyUI.cs:148-156`) closes into
        // gameplay even on the chest<0 no-op (`ChestUI.cs:844-848`).
        let request = ui::keyboard::Request {
            label: ui::keyboard::enter_new_name(),
            initial: String::new(),
            allow_empty: false,
            max_length: 20,
            context: ui::keyboard::Context::Chest,
        };
        let state = ui::keyboard::KeyboardState::new(&request);
        let mut app = flow_app(
            Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
                request,
                state,
                back: None,
                tail: KeyboardTail::ChestRename,
            }))),
            vec![],
            vec![],
        );
        let (sw, sh) = app.screen_size();
        let l = ui::keyboard::layout(Vec2::new(sw, sh), false);
        app.mouse_pos = Vec2::new(
            l.submit[0] + l.submit[2] * 0.5,
            l.submit[1] + l.submit[3] * 0.5,
        );
        app.handle_front_click();
        assert_eq!(app.front, None, "the Save click closed into gameplay");
        assert!(
            app.world_chests.iter().all(|c| c.name.is_empty()),
            "chest < 0 wrote nothing"
        );
        assert!(app.menu.is_none(), "the single-mode invariant holds");
    }

    // --- Task 7: the character creator's flow hops ---------------------------

    /// One creation screen for the flow tests, optionally pre-dirtied.
    fn creation_app(dirty: bool) -> App {
        let mut state = ui::char_creation::CreationState::new();
        if dirty {
            state.draft.name = "Zealot".into();
            state.draft.refresh_dirty();
        }
        flow_app(Some(FrontScreen::CharCreation(state)), vec![], vec![])
    }

    /// Aims the pointer at a char-creation rect's centre and clicks.
    fn click_creation(app: &mut App, rect: [f32; 4]) {
        app.mouse_pos = Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
        app.handle_front_click();
    }

    #[test]
    fn new_click_opens_a_fresh_creation_screen() {
        // `NewCharacterClick` (`UICharacterSelect.cs:108-114`): a fresh
        // `PendingPlayer`, the creation screen live alone.
        let mut app = flow_app(Some(FrontScreen::CharSelect), vec![], vec![]);
        let (sw, sh) = app.screen_size();
        let l = ui::char_select::resolved_layout(
            Vec2::new(sw, sh),
            app.front_header_w(&ui::char_select::select_player()),
            app.players.len(),
        );
        click_creation(&mut app, l.new);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the New click opens the creator");
        };
        assert_eq!(state.draft, ui::char_creation::NewPlayerDraft::new());
        assert_eq!(state.category, 0, "the CharInfo menu seeds (`:233`)");
        assert!(app.menu.is_none(), "the invariant holds");
        assert!(app.front_hint.is_none());
    }

    #[test]
    fn tr_ui_charnew_presets_the_creation_screen() {
        // The Task 5 hook table's `charnew` now presets the real screen.
        let mut app = flow_app(None, vec![], vec![]);
        app.preset_ui(TrUiPreset::CharNew);
        assert!(matches!(app.front, Some(FrontScreen::CharCreation(_))));
        assert!(app.menu.is_none());
    }

    #[test]
    fn creation_clicks_mutate_the_draft() {
        // Category 4 (Eye) switches the picker; a difficulty row sets the
        // byte (`UIDifficultyButton.cs:67-71`); a cloth-style button maps
        // through `_validClothStyles` and auto-assigns the hair (`:630-637`).
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        // The Info menu is live (category 0): a difficulty row sets the byte.
        click_creation(&mut app, l.difficulty_buttons[3]);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(state.draft.difficulty, 2, "the Hardcore row");
        // The Clothing menu: a cloth-style button maps through
        // `_validClothStyles` and auto-assigns the hair (`:630-637`).
        click_creation(&mut app, l.cloth_buttons[0]);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(state.draft.skin_variant, 0);
        assert_eq!(
            state.draft.hair,
            ui::char_creation::default_hairstyle_for_cloth_style(0).unwrap_or_default(),
            "the untouched hair auto-assigns"
        );
        // Category 4 (Eye) switches the picker (`Click_ColorPicker`).
        click_creation(&mut app, l.categories[4]);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(state.category, 4);
    }

    #[test]
    fn clean_creation_back_reopens_the_char_select() {
        // `GoBack`'s not-dirty branch (`UICharacterCreation.cs:1218-1221`):
        // `OpenCharacterSelectUI`, no backup, no confirm.
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.back);
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_backup.is_none());
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn dirty_creation_back_parks_the_mode_40_round_trip() {
        // `GoBack`'s dirty branch (`:1213-1217`): the exact screen parks in
        // `front_backup` (`BackupConfirmationState`) and menuMode 40 opens —
        // the header carries the draft's name (`:49441-49445`).
        let mut app = creation_app(true);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.back);
        assert!(app.front.is_none(), "mode 40 clears the 888 state");
        let Some(FrontScreen::CharCreation(parked)) = app.front_backup.as_ref() else {
            panic!("the creation screen parks");
        };
        assert_eq!(parked.draft.name, "Zealot");
        assert_eq!(parked.draft.dirty, true);
        let m = app.menu.as_ref().expect("mode 40 opens");
        assert_eq!(m.page, menu::Page::ConfirmDiscard);
        assert_eq!(m.confirm_name.as_deref(), Some("Zealot"));
        // The No branch's restore is the flow table's (`:49460-49467`), and
        // Yes drops the parked state for the char select (`:49454-49459`).
        let (front, backup, page) = next_flow_state(
            app.front.clone(),
            app.front_backup.clone(),
            Some(menu::Page::ConfirmDiscard),
            FlowClick::ConfirmNo,
        );
        assert_eq!(front, Some(FrontScreen::CharCreation(parked.clone())));
        assert!(backup.is_none() && page.is_none());
        let (front, backup, page) = next_flow_state(
            app.front.clone(),
            app.front_backup.clone(),
            Some(menu::Page::ConfirmDiscard),
            FlowClick::ConfirmYes,
        );
        assert_eq!(front, Some(FrontScreen::CharSelect));
        assert!(backup.is_none() && page.is_none());
    }

    #[test]
    fn naming_keyboard_round_trips_into_the_draft() {
        // `Click_Naming` (`:1659-1666`) -> `OnFinishedNaming` (`:1684-1689`):
        // the keyboard opens with `allowEmpty: true`, the trimmed name lands
        // in the parked draft and the creation screen returns.
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.info_name);
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the name plate opens the keyboard");
        };
        assert!(session.request.allow_empty);
        assert_eq!(
            session.request.max_length,
            ui::char_creation::MAX_NAME_LENGTH
        );
        assert!(matches!(session.back, Some(FrontScreen::CharCreation(_))));
        assert_eq!(session.tail, KeyboardTail::Name);
        // The empty name also cleared the draft's own (`:1662`).
        if let Some(FrontScreen::CharCreation(back)) = &session.back {
            assert!(back.draft.name.is_empty());
        }
        if let Some(FrontScreen::Keyboard(session)) = app.front.as_mut() {
            ui::keyboard::write_str(&mut session.state, "  Zealot ");
        }
        app.keyboard_submit();
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creation screen returns");
        };
        assert_eq!(state.draft.name, "Zealot", "`name.Trim()` (`:1686`)");
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn create_with_an_empty_name_opens_the_naming_keyboard() {
        // `Click_NamingAndCreating`'s empty branch (`:1668-1677`): the
        // keyboard opens WITHOUT `allowEmpty` (the ctor default `false`), and
        // the submit tail finishes the creation (`OnFinishedNamingAnd-
        // Creating`, `:1696-1702`).
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.create);
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("Create with no name opens the keyboard");
        };
        assert!(!session.request.allow_empty);
        assert_eq!(session.tail, KeyboardTail::NameAndCreate);
        if let Some(FrontScreen::Keyboard(session)) = app.front.as_mut() {
            ui::keyboard::write_str(&mut session.state, "Zealot");
        }
        app.keyboard_submit();
        assert_eq!(
            app.front,
            Some(FrontScreen::CharSelect),
            "`menuMode = 1` (`:1710`)"
        );
        assert!(app.menu.is_none());
    }

    #[test]
    fn create_with_a_name_records_the_draft_as_the_session_player() {
        // `FinishCreatingCharacter` (`:1704-1711`): the draft records (no
        // `.plr` writer — documented), the session entry carries the name and
        // difficulty, and the char select re-lists.
        let mut app = creation_app(false);
        if let Some(FrontScreen::CharCreation(state)) = app.front.as_mut() {
            state.draft.name = "Zealot".into();
            state.draft.difficulty = 2;
            state.category = 0;
        }
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.create);
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_backup.is_none(), "the backup drops");
        let Some(draft) = app.session_draft.as_ref() else {
            panic!("the draft records");
        };
        assert_eq!(draft.name, "Zealot");
        assert_eq!(draft.difficulty, 2);
        // `Main.LoadPlayers` re-lists with the created entry first-shaped.
        let entry = app.session_entry();
        assert_eq!(entry.name, "Zealot");
        assert_eq!(entry.difficulty, 2);
        assert!(app.players.iter().any(|p| p.name == "Zealot"));
    }

    // -- M3-REFORGE-1: the reforge action (`Main.ReforgeItemInReforgeSlot`,
    // `Main.cs:41902-41933`) + the slot transfer (`ItemSlot.cs:821-836`) --

    fn reforge_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.coins = 100_000;
        app.in_reforge_menu = true;
        app.reforge_slot = crate::item::ItemSlot {
            kind: 1, // the Iron Pickaxe — value 2000, the swords table
            name: "Iron Pickaxe",
            count: 1,
            prefix: 0,
        };
        app
    }

    #[test]
    fn the_reforge_charge_precedes_the_roll() {
        // The purse gate: a short purse leaves everything untouched
        // (`BuyItem(num53)` fails → no `ReforgeItemInReforgeSlot`).
        let mut app = reforge_app();
        app.coins = 100; // the 2000-value pickaxe costs 666
        app.reforge_click();
        assert_eq!(app.coins, 100);
        assert_eq!(
            app.reforge_slot.prefix, 0,
            "the failed charge re-rolls nothing"
        );

        // The funded click: the charge, the roll (Prefix(-2) — never none,
        // on the swords table), the top-tier cooldown.
        let mut app = reforge_app();
        app.reforge_click();
        assert_eq!(app.coins, 100_000 - 666);
        let p = app.reforge_slot.prefix;
        assert_ne!(p, 0, "Prefix(-2) never lands none");
        assert!(
            crate::prefix::PREFIXES_FOR_SWORDS.contains(&p),
            "the pickaxe rolls from the swords table"
        );
        if p == 81 {
            assert_eq!(
                app.reforge_cooldown, 60,
                "the top tier arms the 60-tick lock"
            );
        } else {
            assert_eq!(app.reforge_cooldown, 0);
        }
    }

    #[test]
    fn the_reforge_roll_consumes_the_prefix_stream() {
        // `Prefix(-2)` (`Main.cs:41924`) draws from `Main.rand` — the port's
        // `prefix_rng` (`S1-A-5`), never the npc stream.
        let mut app = reforge_app();
        let npc_rng = app.npc_rng;
        let mut expect = tr_core::UnifiedRandom::new(0); // `flow_app`'s seed
        let (expected, _) = crate::prefix::reforge_roll(app.reforge_slot.kind, &mut expect);
        app.reforge_click();
        assert_eq!(app.reforge_slot.prefix, expected);
        assert_eq!(app.npc_rng, npc_rng, "the npc stream must not advance");
        assert_eq!(app.prefix_rng.peek(), expect.peek());
    }

    #[test]
    fn the_reforge_cooldown_blocks_the_next_click() {
        let mut app = reforge_app();
        app.reforge_cooldown = 10;
        let coins = app.coins;
        app.reforge_click();
        assert_eq!(app.coins, coins, "the cooldown gates the charge");
    }

    #[test]
    fn the_reforge_slot_transfer_follows_the_vanilla_split() {
        // The single-item cursor swaps both ways; a stack splits ONE into
        // the empty slot (`ItemSlot.cs:821-836`).
        let mut app = reforge_app();
        app.mouse_item = crate::item::ItemSlot {
            kind: 41,
            name: "Dye",
            count: 1,
            prefix: 3,
        };
        app.reforge_slot_click();
        assert_eq!(app.reforge_slot.kind, 41);
        assert_eq!(
            app.reforge_slot.prefix, 3,
            "the carried prefix moves with the item"
        );
        assert_eq!(app.mouse_item.kind, 1, "the pickaxe returns to the cursor");

        // The stack split into the EMPTY slot (the fixture's pickaxe goes
        // back to the cursor first).
        let mut app = reforge_app();
        app.reforge_slot = crate::item::EMPTY_SLOT;
        app.mouse_item = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 5,
            prefix: 0,
        };
        app.reforge_slot_click();
        assert_eq!(app.reforge_slot.count, 1);
        assert_eq!(app.mouse_item.count, 4);
        // A stack cannot merge into a held slot — nothing moves.
        app.reforge_slot_click();
        assert_eq!(app.reforge_slot.count, 1);
        assert_eq!(app.mouse_item.count, 4);
    }

    #[test]
    fn closing_the_shop_returns_the_reforge_item() {
        // `Main.cs:41574-41578`'s force-close: the slot item drops back into
        // the inventory.
        let mut app = reforge_app();
        app.close_shop();
        assert!(!app.in_reforge_menu);
        assert_eq!(app.reforge_slot.count, 0);
        assert!(
            app.inventory.iter().any(|sl| sl.kind == 1 && sl.count == 1),
            "the pickaxe landed in the first free slot"
        );
    }

    /// The front end's Play parks the selected world's file on the App
    /// (`UIWorldListItem.PlayGame` → `Main.LoadWorld`, BUG-04) — the click
    /// used to close the screens without loading anything, so the boot world
    /// played no matter which card was pressed.
    #[test]
    fn world_play_parks_the_selected_file() {
        let mut app = flow_app(None, Vec::new(), Vec::new());
        let mut w0 = world_entry("Alpha");
        w0.path = Some(std::path::PathBuf::from("saves/Alpha.trwld"));
        let mut w1 = world_entry("Beta");
        w1.path = Some(std::path::PathBuf::from("saves/Beta.trwld"));
        open_world_select(&mut app, vec![w0, w1]);
        app.pointer.tick(300.0);
        let (_screen, l, _favorites) = world_screen(&app);
        let c1 = ui::world_select::card_layout(l.list, 1, false);
        app.mouse_pos = centre(c1.play);
        app.apply_front_flow(FrontScreen::WorldSelect, FlowClick::WorldPlay(1));
        assert_eq!(
            app.play_world.as_deref(),
            Some(std::path::Path::new("saves/Beta.trwld")),
            "the pressed card's file rides the swap"
        );
        assert!(
            app.front.is_none(),
            "the launch arm closes the front screens"
        );
    }

    /// `UIWorldListItem.PlayGame`'s tail: the loaded world is entered, not
    /// re-listed (`Main.LoadWorld` leaves `Main.gameMenu` false).
    ///
    /// The select-world Play used to load the world and then draw the lobby over
    /// it: `swap_world` rebuilds the App through `App::new`, whose boot default
    /// opens the legacy menu (`menu_state`), and nothing closed it afterwards —
    /// while the multiplayer install ran `client_install_world`'s "close the
    /// menu" tail and therefore worked. Both paths now end on
    /// `App::enter_loaded_world`.
    #[test]
    fn entering_a_loaded_world_clears_both_front_end_sides() {
        let mut app = flow_app(Some(FrontScreen::WorldSelect), Vec::new(), Vec::new());
        app.front_backup = Some(FrontScreen::CharSelect);
        app.front_hint = Some("world load failed: x".into());
        // The state `App::new`'s boot default leaves behind (the fresh App a
        // `swap_world` builds starts with the legacy main page open).
        app.menu = Some(menu::Menu::new(app.settings.clone(), "flow".into()));
        assert!(app.menu.is_some() && app.front.is_some());

        app.enter_loaded_world();

        assert!(app.menu.is_none(), "the lobby page must not come back");
        assert!(app.front.is_none(), "the 888 side clears too");
        assert!(app.front_backup.is_none());
        assert!(app.front_hint.is_none());
    }

    /// The favourites survive the save/load round (BUG-03): two slots
    /// toggled, flattened into the `.plr` bitfield and read back, land on the
    /// same two slots — the App's `favorite_slots` used to be UI state with no
    /// file representation.
    #[test]
    fn the_favorites_round_trip_through_the_bitfield() {
        let mut app = flow_app(None, Vec::new(), Vec::new());
        app.hydrate_favorites(0);
        assert_eq!(app.toggle_favorite(3), 12, "the menu tick id");
        assert!(app.is_favorite(3));
        app.toggle_favorite(9);
        let bits = app.favorite_bits();
        assert_eq!(bits, 1 << 3 | 1 << 9, "the two stars flatten into the bits");
        // The load half: a fresh App hydrates the same two slots.
        let mut loaded = flow_app(None, Vec::new(), Vec::new());
        loaded.hydrate_favorites(bits);
        assert!(loaded.is_favorite(3) && loaded.is_favorite(9));
        assert!(!loaded.is_favorite(4), "the untouched slot stays unstarred");
        // The toggle is reversible and the bits follow it.
        loaded.toggle_favorite(3);
        assert_eq!(loaded.favorite_bits(), 1 << 9);
    }

    /// The front-end half of the audio clock (BUG-01): with a front screen
    /// live, the per-frame `update_menu_music` decides the title track
    /// (`Main.cs:12455-12532`'s `gameMenu` arm), so `cur_music` leaves 0
    /// before any world is entered.
    #[test]
    fn the_front_screen_ticks_the_menu_music() {
        let mut app = flow_app(Some(FrontScreen::WorldSelect), Vec::new(), Vec::new());
        app.front = Some(FrontScreen::WorldSelect);
        assert_eq!(app.music.cur_music, 0, "the boot state is silent");
        app.update_menu_music();
        assert_eq!(
            app.music.cur_music, 51,
            "the world-select screen plays the title track"
        );
        // The menu env is the menu arm regardless of which screen is up.
        let mut no_menu = flow_app(None, Vec::new(), Vec::new());
        no_menu.update_menu_music();
        assert_eq!(
            no_menu.music.cur_music, 51,
            "no front screen: the plain track"
        );
    }

    /// The world-creation screen's seed text feeds the menu ladder — the
    /// vanilla's `WorldGen.*WorldGen` statics (`Main.cs:12463-12494`) are the
    /// front-end's view of the pending seed.
    #[test]
    fn the_creation_screens_seed_drives_the_menu_track() {
        let draft = ui::world_creation::NewWorldDraft::new(1, 0);
        let mut app = flow_app(
            Some(FrontScreen::WorldCreation(draft)),
            Vec::new(),
            Vec::new(),
        );
        if let Some(FrontScreen::WorldCreation(d)) = &mut app.front {
            d.seed = "5162020".into(); // the drunk world's seed text
        }
        app.update_menu_music();
        assert_eq!(app.music.cur_music, 60, "the drunk title track");
        if let Some(FrontScreen::WorldCreation(d)) = &mut app.front {
            d.seed = "celebrationmk10".into(); // the anniversary seed text
        }
        app.update_menu_music();
        assert_eq!(app.music.cur_music, 11, "the anniversary title track");
    }
}
