//! The debug commands list (`UIDebugCommandsList`,
//! `GameContent/UI/States/UIDebugCommandsList.cs:15-165`) and its row element
//! (`UIDebugCommandItem`, `Elements/UIDebugCommandItem.cs:12-120`) — W4-d85
//! row 7 (+ the row-9 element half). An `IngameFancyUI` overlay like the
//! achievements menu: the 0.8-wide container at Top 220, the `(33,43,79)*0.95`
//! panel, the 200 px wrapped search bar, the scrolling 30 px command rows,
//! the "Debug Commands" title and the fat Back panel.
//!
//! The registry is the 62 `[DebugCommand]` attributes of
//! `Testing/ChatCommands/ToolkitDebugCommands.cs` (the
//! `DebugCommandProcessor.AddAttributeCommandsFromType` reflection walk,
//! `DebugCommandProcessor.cs:36-47`), sorted `OrdinalIgnoreCase` by name
//! (`PopulateCommandsList`, `UIDebugCommandsList.cs:140-151`).
//!
//! Opened by the `/hh` chat command (`ToolkitDebugCommands.cs:42-47` →
//! `IngameFancyUI.OpenUIState(new UIDebugCommandsList())`). A row click
//! closes the overlay and seeds the chat input (`UIDebugCommandItem.LeftClick`,
//! `:106-112`).
//!
//! Deviations (docs/ANALYSIS.md "Debug lists (W4-d85)"): the fake-item hover
//! tooltip (`DrawSelf`'s `Main.DisplayAndGetFakeItem`, `:51-56`) is not drawn;
//! the search bar's inline typing (`UISearchBar.ToggleTakingText`) opens the
//! port's virtual keyboard instead; the command *processors* are not ported
//! (only the two list-opening commands run).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text::{self, BorderFont};
use super::uistate::{self, Style};
use super::{color, in_rect, panel};
use crate::assets::Lookups;
use crate::lang;

/// `CommandRequirement` (`Testing/ChatCommands/CommandRequirement.cs:8-16`).
pub mod req {
    pub const SINGLE_PLAYER: u32 = 1;
    pub const MULTIPLAYER_CLIENT: u32 = 2;
    pub const MULTIPLAYER_RPC: u32 = 4;
    pub const LOCAL_SERVER: u32 = 8;
    pub const CLIENT_AUTHORITY: u32 = SINGLE_PLAYER | MULTIPLAYER_RPC;
    pub const ANY_AUTHORITY: u32 = CLIENT_AUTHORITY | LOCAL_SERVER;
    pub const CLIENT: u32 = SINGLE_PLAYER | MULTIPLAYER_CLIENT;
    pub const LOCAL: u32 = CLIENT | LOCAL_SERVER;
    pub const ALL: u32 = ANY_AUTHORITY | MULTIPLAYER_CLIENT;
}

/// One `IDebugCommand` — the `[DebugCommand(name, description, requirements,
/// HelpText = …)]` attribute data (`DebugCommandAttribute.cs:40-51`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebugCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub requirements: u32,
    pub help_text: &'static str,
}

/// The 62 `ToolkitDebugCommands` registrations, in
/// `PopulateCommandsList`'s display order (`StringComparer.OrdinalIgnoreCase`
/// over the names, `UIDebugCommandsList.cs:143`). Descriptions/help texts are
/// verbatim from the attributes; the requirement values fold the enum's
/// composite members (`CommandRequirement.cs:12-16`).
pub const COMMANDS: &[DebugCommand] = &[
    DebugCommand { name: "=", description: "repeats latest command", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "checkpoint", description: "Saves a gameplay checkpoint", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "clearmap", description: "Deletes the full map for the world.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "drawaggro", description: "Draws aggro on the screen", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "drawhitboxes", description: "Draws hitboxes on the screen", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "expandedbossdamage", description: "Toggles expanded boss damage, which separates damage, and shows hit count per source.", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "fakenetoffset", description: "Sets the netOffset for all entities to a given value (in pixels).", requirements: req::CLIENT, help_text: "Usage: /fakenetoffset <dx> <dy>" },
    DebugCommand { name: "find", description: "Iterates through all instances of a tile in the world. Use /next to skip through them", requirements: req::CLIENT, help_text: "Usage: /find <id>" },
    DebugCommand { name: "findwall", description: "Iterates through all instances of a wall in the world. Use /next to skip through them", requirements: req::CLIENT, help_text: "Usage: /findwall <id>" },
    DebugCommand { name: "forcegamepad", description: "Prevents keyboard and mouse input from overriding gamepad input state. Keyboard can still use commands and chat", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "frame", description: "Resets all frame data", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hash", description: "Prints out the hash of all saved (non-volatile) tile data", requirements: req::ANY_AUTHORITY, help_text: "" },
    DebugCommand { name: "hh", description: "Opens a list of all the debug commands", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hideall", description: "Stops tiles, walls, and water from drawing", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hidetiles", description: "Stops tiles from drawing on the screen", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hidetiles2", description: "Stops non-solid tiles from drawing on the screen", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hidewalls", description: "Stops walls from drawing on the screen", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hidewater", description: "Stops water from drawing on the screen", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "hurtdummies", description: "Allows projectiles to aim at target dummies.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "latency", description: "Adds latency to incoming and outgoing packets sent by this client.", requirements: req::MULTIPLAYER_CLIENT, help_text: "Usage: /latency <ms>" },
    DebugCommand { name: "light", description: "Toggles the lighting system between active and fullbright.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "loadout", description: "creates a loadout.", requirements: req::CLIENT, help_text: "Usage: /loadout (name)" },
    DebugCommand { name: "loadouts", description: "Opens a list of all the loadouts", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "logitemuse", description: "Reports Player.ApplyItem/AnimTime calls in local chat.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "map", description: "Reveals the full map for the world.", requirements: req::CLIENT, help_text: "Usage: /map [pretty]" },
    DebugCommand { name: "memo", description: "Creates a shortcut command with a given name. Opens the file to write in. One command per line. Accepts arg substitions ({0}, {1}, etc)", requirements: req::CLIENT, help_text: "Usage: /memo <custom-command-name>" },
    DebugCommand { name: "memonum", description: "Creates a memo for a numpad key (0-9). Shorthand for /memo numpad{i}", requirements: req::CLIENT, help_text: "Usage: /memonum <0-9>" },
    DebugCommand { name: "next", description: "Finds the next instance of a tile/wall/object. Requires calling /find, /findwall or /snapshotdiff first", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "nodamagevar", description: "Removes damage variation (the inherent +/- 15% from damage). Useful for gathering specific data on true damage.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "nolimits", description: "No border restrictions", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "notimeouts-client", description: "Toggles client-side timeouts", requirements: req::MULTIPLAYER_CLIENT, help_text: "" },
    DebugCommand { name: "notimeouts-server", description: "Toggles server-side timeouts", requirements: req::MULTIPLAYER_RPC, help_text: "" },
    DebugCommand { name: "pauseworld", description: "Pauses world updates. Use Numpad+/End/Mouse4 to step", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "practice", description: "Toggles practice mode, which returns you to the last checkpoint when you take lethal damage.", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "quickload", description: "Automatically rejoin this world/server with this player whenever the game is launched. Executes /onquickload memo when joining the world. Will relaunch all local clients when used with  Host & Play", requirements: req::CLIENT, help_text: "/quickload  [stop]" },
    DebugCommand { name: "quickload-regen", description: "Automatically regenerate this world whenever the game is launched. Executes /onquickload memo when joining the world", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "quit", description: "Exit world without saving.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "randomizeprojslots", description: "Toggles randomization of projectile slots (whoAmI).", requirements: req::ANY_AUTHORITY, help_text: "" },
    DebugCommand { name: "record", description: "Creates a checkpoint and starts recording inputs", requirements: req::SINGLE_PLAYER, help_text: "Usage: /record [stop]" },
    DebugCommand { name: "reload", description: "Reloads the last save", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "reloadpacks", description: "Reloads resource packs.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "replay", description: "Replays the recorded input capture on loop", requirements: req::SINGLE_PLAYER, help_text: "Usage: /replay [stop]" },
    DebugCommand { name: "restore", description: "Restores the world's tiles to the previously created snapshot.", requirements: req::ANY_AUTHORITY, help_text: "" },
    DebugCommand { name: "return", description: "Returns to the last checkpoint", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "save", description: "Save the player (and the world if single player)", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "saveloadout", description: "saves a loadout to the loadouts menu.", requirements: req::CLIENT, help_text: "Usage: /saveloadout (name)" },
    DebugCommand { name: "setdrawwait", description: "Sets a fixed waiting period to occur during each engine draw call.", requirements: req::CLIENT, help_text: "Usage: /setdrawwait <delay in ms>" },
    DebugCommand { name: "setserverping", description: "Sets a target ping for all players on the server. Clients will automatically adjust /latency to achieve it.", requirements: req::MULTIPLAYER_RPC, help_text: "Usage: /setserverping <ms>" },
    DebugCommand { name: "setupdatewait", description: "Sets a fixed waiting period to occur during each engine update call.", requirements: req::CLIENT, help_text: "Usage: /setupdatewait <delay in ms>" },
    DebugCommand { name: "showdebug", description: "Toggles command reporting.", requirements: req::MULTIPLAYER_RPC | req::LOCAL_SERVER, help_text: "" },
    DebugCommand { name: "showlinks", description: "Draws gamepad link points as an interface overlay", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "shownetoffset", description: "Draws indicators for debugging netOffset", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "showsections", description: "Toggles net section overlay.", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "showunbreakablewalls", description: "Forces unbreakable walls to be visible even when covered by tiles", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "snapclear", description: "Clears previously created snapshot.", requirements: req::ANY_AUTHORITY, help_text: "" },
    DebugCommand { name: "snapload", description: "Loads a snapshot in dev-snapshots.", requirements: req::ANY_AUTHORITY, help_text: "Usage: /snapsave <name>" },
    DebugCommand { name: "snapsave", description: "Saves a snapshot in dev-snapshots.", requirements: req::ANY_AUTHORITY, help_text: "Usage: /snapsave <name>" },
    DebugCommand { name: "snapshot", description: "Creates a snapshot of the current tile state for the world.", requirements: req::ANY_AUTHORITY, help_text: "" },
    DebugCommand { name: "snapshotdiff", description: "Finds differences between the current map and saved snapshot. Use /next to skip through them.", requirements: req::SINGLE_PLAYER, help_text: "" },
    DebugCommand { name: "swap", description: "Swaps the world's tiles with the previously created snapshot.", requirements: req::ANY_AUTHORITY, help_text: "" },
    DebugCommand { name: "togglecore0pinning", description: "Toggles core 0 unpinning on load (this setting is saved).", requirements: req::CLIENT, help_text: "" },
    DebugCommand { name: "toggleinactivewait", description: "Toggles main thread sleeping when window is unfocused (this setting is saved).", requirements: req::CLIENT, help_text: "" },
];

/// `UIDebugCommandItem`'s row height — `Height.Set(30f, 0f)`
/// (`UIDebugCommandItem.cs:30`).
pub const ROW_H: f32 = 30.0;
/// `ListPadding = 5f` (`UIDebugCommandsList.cs:59`).
pub const LIST_PADDING: f32 = 5.0;
/// The row stride (`UIList.RecalculateChildren`, `UIList.cs:119-127`).
pub const ROW_PITCH: f32 = ROW_H + LIST_PADDING;

/// `DoesCommandMatchSearch` (`UIDebugCommandsList.cs:104-120`): the
/// lowercased contents match the name, the description or the help text
/// (each `ToLowerInvariant().Contains`).
pub fn command_matches(lower_contents: &str, command: &DebugCommand) -> bool {
    command.name.to_lowercase().contains(lower_contents)
        || command.description.to_lowercase().contains(lower_contents)
        || command.help_text.to_lowercase().contains(lower_contents)
}

/// `searchbar_OnSearchContentsChanged` (`:86-102`): null → "", lower + trim,
/// whitespace shows everything, otherwise the filtered registry (in the
/// sorted display order).
pub fn visible_commands(search: &str) -> Vec<usize> {
    let text = search.to_lowercase();
    let text = text.trim();
    let all = text.is_empty();
    COMMANDS
        .iter()
        .enumerate()
        .filter(|(_, c)| all || command_matches(text, c))
        .map(|(i, _)| i)
        .collect()
}

/// `CanCurrentlyBeUsed` (`UIDebugCommandItem.cs:73-80`): a SinglePlayer-only
/// command greys out when `Main.netMode != 0`. The port's `is_client` is the
/// `netMode == 1` arm; a hosted server is `netMode == 2` in the C# but the
/// port's host plays locally, so only the client grey-out applies.
pub fn can_currently_be_used(command: &DebugCommand, is_client: bool) -> bool {
    !((command.requirements & !req::SINGLE_PLAYER) == 0 && is_client)
}

/// The `CommandRequirement` flags' `ToString` — the named composites first
/// (the C# `[Flags]` formatter prefers the exact named value), else the
/// numeric form. Only the values the registry uses appear.
pub fn requirement_name(requirements: u32) -> String {
    match requirements {
        req::SINGLE_PLAYER => "SinglePlayer".into(),
        req::MULTIPLAYER_CLIENT => "MultiplayerClient".into(),
        req::MULTIPLAYER_RPC => "MultiplayerRPC".into(),
        req::LOCAL_SERVER => "LocalServer".into(),
        req::CLIENT_AUTHORITY => "ClientAuthority".into(),
        req::ANY_AUTHORITY => "AnyAuthority".into(),
        req::CLIENT => "Client".into(),
        req::LOCAL => "Local".into(),
        req::ALL => "All".into(),
        other => other.to_string(),
    }
}

// --- Geometry (`BuildPage`, `UIDebugCommandsList.cs:30-84`) -------------------

/// The outer container: `Width (0, 0.8)`, `MaxWidth 800`, `MinWidth 600`,
/// `Top 220`, `Height (-220, 1)`, `HAlign 0.5` (`:33-40`).
pub fn container_rect(screen: Vec2) -> [f32; 4] {
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(220.0),
        Style::new(0.0, 0.8),
        Style::new(-220.0, 1.0),
        0.5,
        0.0,
        600.0,
        0.0,
        800.0,
        [0.0; 4],
    )
}

/// The `UIPanel`: `Width (0, 1)`, `Height (-110, 1)` (`:41-45`); its inner
/// rect drops the `UIPanel` ctor's 12 px padding (`UIPanel.cs:32`).
pub fn panel_rect(container: &[f32; 4]) -> [f32; 4] {
    [container[0], container[1], container[2], container[3] - 110.0]
}

/// The `UIWrappedSearchBar`: `Width 200`, `Top 20`, `Height 24`
/// (`UIDebugCommandsList.cs:51-52`, `UIWrappedSearchBar.cs:81`), against the
/// panel's inner rect.
pub fn search_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(*panel, [12.0; 4]);
    [inner[0], inner[1] + 20.0, 200.0, 24.0]
}

/// The `UIList`: `Width (-25, 1)`, `Height (-60, 1)`, `VAlign 1`
/// (`:56-58`), against the panel's inner rect.
pub fn list_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(*panel, [12.0; 4]);
    [inner[0], inner[1] + 60.0, inner[2] - 25.0, inner[3] - 60.0]
}

/// The attached `UIScrollbar`: `Height (0, 1)`, `HAlign 1`, width 20
/// (`:77-81`, `UIScrollbar.cs:59`), against the panel's inner rect.
pub fn scrollbar_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(*panel, [12.0; 4]);
    [inner[0] + inner[2] - 20.0, inner[1], 20.0, inner[3]]
}

/// The row rect at `index` (`UIList.RecalculateChildren`'s lattice,
/// `UIList.cs:119-127`): `Top = index * (30 + 5)`, `Width (0, 1)`.
pub fn row_rect(list: &[f32; 4], index: usize) -> [f32; 4] {
    [list[0], list[1] + index as f32 * ROW_PITCH, list[2], ROW_H]
}

/// The list content height (`UIList.cs:119-127`): N rows = N*(30+5); a
/// single row drops the padding; none = 0.
pub fn content_height(rows: usize) -> f32 {
    match rows {
        0 => 0.0,
        1 => ROW_H,
        n => n as f32 * ROW_PITCH,
    }
}

/// The title panel: `UITextPanel("Debug Commands", 1f, large)` with
/// `HAlign 0.5`, `Top -33`, `SetPadding(13)` (`:61-66`) — the `SetText`
/// minimums size it (`UITextPanel.cs:132-134`: text + 26 wide, 32 + 26 tall).
pub fn title_rect(container: &[f32; 4], text_w: f32) -> [f32; 4] {
    let w = text_w + 26.0;
    [
        container[0] + container[2] * 0.5 - w * 0.5,
        container[1] - 33.0,
        w,
        32.0 + 26.0,
    ]
}

/// The Back panel: `Width (-10, 0.5)`, `Height 50`, `VAlign 1`,
/// `HAlign 0.5`, `Top -45` (`:67-72`).
pub fn back_rect(container: &[f32; 4]) -> [f32; 4] {
    let w = container[2] * 0.5 - 10.0;
    [
        container[0] + container[2] * 0.5 - w * 0.5,
        container[1] + container[3] - 45.0 - 50.0,
        w,
        50.0,
    ]
}

// --- Hits --------------------------------------------------------------------

/// What a press does — the `OnLeftClick` subscriptions of the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// `GoBackClick` → `IngameFancyUI.Close()` (`:122-125`).
    Back,
    /// The search box / magnifier (`Click_SearchArea`,
    /// `UIWrappedSearchBar.cs:222-228`) — the port opens the keyboard.
    Search,
    /// The cancel button (`searchCancelButton_OnClick`, `:153-164`).
    SearchCancel,
    /// A command row (`UIDebugCommandItem.LeftClick`, `:106-112`) — the
    /// index into [`COMMANDS`].
    Row(usize),
    None,
}

/// The click table over the live geometry. `rows` is [`visible_commands`]'
/// output; the row hit respects the list viewport (`UIElement.GetElementAt`
/// descends only through a containing ancestor, `UIElement.cs:327-341`).
pub fn click_at(screen: Vec2, mouse: Vec2, scroll: f32, rows: &[usize]) -> Click {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    if in_rect(mouse, back_rect(&container)) {
        return Click::Back;
    }
    let search = search_rect(&panel);
    if in_rect(mouse, search) {
        // The box panel: `Width (-buttonW-3, 1)`, `HAlign 1`
        // (`UIWrappedSearchBar.cs:110-116`).
        let box_x = search[0] + SEARCH_BUTTON_W + 3.0;
        let box_w = search[2] - SEARCH_BUTTON_W - 3.0;
        // The cancel button sits at the box's right edge (`HAlign 1`,
        // `Left -2`, `:142-147`); the rest opens input.
        let cancel_w = 20.0;
        let cancel = [box_x + box_w - cancel_w - 2.0, search[1], cancel_w, search[3]];
        if in_rect(mouse, cancel) {
            return Click::SearchCancel;
        }
        return Click::Search;
    }
    let list = list_rect(&panel);
    if in_rect(mouse, list) {
        let idx = ((mouse.y - list[1] + scroll) / ROW_PITCH) as usize;
        if let Some(&row) = rows.get(idx) {
            let rect = row_rect(&list, idx);
            let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
            if in_rect(mouse, shifted) {
                return Click::Row(row);
            }
        }
    }
    Click::None
}

/// The wheel target: the list rect or the attached scrollbar
/// (`UserInterface.Update`'s dispatch, `:219-221`).
pub fn wheel_target(screen: Vec2, mouse: Vec2, rows: usize) -> bool {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    let list = list_rect(&panel);
    let attaches = content_height(rows) > list[3];
    in_rect(mouse, list) || (attaches && in_rect(mouse, scrollbar_rect(&panel)))
}

// --- Draw --------------------------------------------------------------------

/// The panel background — `new Color(33, 43, 79) * 0.95f` (`:44`).
const PANEL_BG: [u8; 4] = [31, 40, 75, 242]; // XNA byte truncation of *0.95
/// The title background — `new Color(73, 94, 171)` (`:65`).
const TITLE_BG: [u8; 4] = [73, 94, 171, 255];
/// The search box panel — `new Color(35, 40, 83)` both colours
/// (`UIWrappedSearchBar.cs:117-118`).
const SEARCH_BG: [u8; 4] = [35, 40, 83, 255];
/// The magnifier button's width (`Button_Search_2`, 22x22 —
/// `UIImageButton` is texture-sized, `UIImageButton.cs:30-33`).
const SEARCH_BUTTON_W: f32 = 22.0;
/// The row hover pair (`UIDebugCommandItem.MouseOver`, `:90-91`).
const ROW_HOVER_BG: [u8; 4] = [76, 90, 149, 255];
const ROW_HOVER_BORDER: [u8; 4] = [50, 60, 86, 255];
/// `Color.DarkGray` — the unusable command's text (`:66-67`).
const DARK_GRAY: [f32; 4] = [169.0 / 255.0, 169.0 / 255.0, 169.0 / 255.0, 1.0];
/// `Color.Gold` — the description text (`:63`).
const GOLD: [f32; 4] = [1.0, 215.0 / 255.0, 0.0, 1.0];
/// The port's overlay dim (the achievements/bestiary treatment).
const BACKDROP: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

/// `UIDebugCommandsList.Draw` with the port's overlay treatment.
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    search: &str,
    scroll: f32,
    mouse: Vec2,
    is_client: bool,
) {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);

    // The `UIPanel` body + black border (`UIPanel.DrawSelf`, `:68-77`).
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, panel, color::tint(PANEL_BG));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    draw_search_bar(r, lookups, &panel, search, mouse);
    draw_title(r, lookups, &container);
    draw_back(r, lookups, &container, mouse);

    let rows = visible_commands(search);
    let list = list_rect(&panel);
    // The `UIList` scissor (`OverflowHidden`, S1-H-2): rows straddling the
    // viewport edge are cut, not skipped.
    r.push_clip(list);
    let first = (scroll / ROW_PITCH).floor().max(0.0) as usize;
    let count = ((list[3] + LIST_PADDING) / ROW_PITCH).floor().max(1.0) as usize;
    for slot in 0..=count {
        let Some(&row) = rows.get(first + slot) else { break };
        let rect = row_rect(&list, first + slot);
        let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
        draw_row(r, lookups, shifted, &COMMANDS[row], in_rect(mouse, shifted), is_client);
    }
    r.pop_clip();

    // The scrollbar (`SetView(100, 1000)` re-set by `SetScrollbar` to the
    // list's inner height / content height, `UIList.cs:130-137`).
    let content = content_height(rows.len());
    if content > list[3] {
        panel::draw_scrollbar(
            r,
            lookups,
            scrollbar_rect(&panel),
            list[3],
            content,
            scroll,
            false,
        );
    }
}

/// The `UIWrappedSearchBar` (`AddSearchBar`, `UIWrappedSearchBar.cs:95-151`):
/// the `(35,40,83)` box with the contents (or the `UI.PlayerNameSlot` empty
/// text in gray), the magnifier button on the left and the cancel cross on
/// the right. The inline caret is the keyboard session's (deviation).
fn draw_search_bar(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    panel: &[f32; 4],
    search: &str,
    mouse: Vec2,
) {
    let rect = search_rect(panel);
    // The box panel: `Width (-buttonW-3, 1)`, `HAlign 1` (`:110-116`) — the
    // magnifier is texture-sized 22 (`Button_Search_2`).
    let box_rect = [
        rect[0] + SEARCH_BUTTON_W + 3.0,
        rect[1],
        rect[2] - SEARCH_BUTTON_W - 3.0,
        rect[3],
    ];
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, box_rect, color::tint(SEARCH_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, box_rect, color::tint(SEARCH_BG));
    let (text, tint) = if search.is_empty() {
        (lang::text("UI.PlayerNameSlot"), color::tint([128, 128, 128, 255]))
    } else {
        (search.to_string(), [1.0; 4])
    };
    text::hud_text(r, lookups, box_rect[0] + 4.0, box_rect[1] + 5.0, 0.8, tint, &text);
    // The magnifier (`UIImageButton`, `SetVisibility(1, 1)` — always full).
    if let Some(sheet) = lookups.ui_search_icon {
        super::push_whole(
            r,
            lookups,
            &sheet,
            [rect[0], rect[1] + (rect[3] - 22.0) * 0.5, 22.0, 22.0],
            [1.0; 4],
        );
    }
    // The cancel cross (`HAlign 1, Left -2`, `:142-147`).
    if let Some(sheet) = lookups.ui_search_cancel {
        let w = sheet.width as f32;
        let h = sheet.height as f32;
        let hovered = in_rect(
            mouse,
            [box_rect[0] + box_rect[2] - w - 2.0, rect[1] + (rect[3] - h) * 0.5, w, h],
        );
        super::push_whole(
            r,
            lookups,
            &sheet,
            [box_rect[0] + box_rect[2] - w - 2.0, rect[1] + (rect[3] - h) * 0.5, w, h],
            if hovered { [1.0; 4] } else { [0.7, 0.7, 0.7, 1.0] },
        );
    }
}

/// The "Debug Commands" title (`UITextPanel<string>`, scale 1 large,
/// `SetPadding(13)`, bg `(73,94,171)`, `:61-66`).
fn draw_title(r: &mut SpriteRenderer, lookups: &Lookups, container: &[f32; 4]) {
    let label = "Debug Commands";
    let w = match &lookups.death_font {
        Some(f) => text::font_width(f, label, 1.0),
        None => r.text_width(label, 1.0),
    };
    let rect = title_rect(container, w);
    let style = uistate::TextPanelStyle {
        text_scale: 1.0,
        large: true,
        padding: 13.0,
        bg: TITLE_BG,
        border: uistate::TEXT_PANEL_BORDER,
    };
    uistate::draw_text_panel(r, lookups, rect, &style, label, w, false);
}

/// The Back panel (`UI.Back`, 0.7 large, the `FadedMouseOver`/`Out` pair,
/// `:67-76` / `:127-138`).
fn draw_back(r: &mut SpriteRenderer, lookups: &Lookups, container: &[f32; 4], mouse: Vec2) {
    let rect = back_rect(container);
    let label = lang::text("UI.Back");
    let style = super::cards::fat_button_style();
    let w = text::hud_text_width(r, lookups, &label, style.text_scale);
    uistate::draw_text_panel(r, lookups, rect, &style, &label, w, in_rect(mouse, rect));
}

/// One `UIDebugCommandItem` (`DrawSelf`, `UIDebugCommandItem.cs:44-71`):
/// the transparent panel (hover paints `(76,90,149)`/`(50,60,86)`), the
/// name at inner + (12, -2) scale 1.1 and the description at
/// inner + (208, 23) scale 0.8 anchored to its bottom (`anchorY = 1`).
fn draw_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    command: &DebugCommand,
    hover: bool,
    is_client: bool,
) {
    if hover {
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, rect, color::tint(ROW_HOVER_BG));
        panel::draw_panel(r, lookups, lookups.ui_panel_border, rect, color::tint(ROW_HOVER_BORDER));
    }
    // `SetPadding(6)` (`:32`) — the inner rect the texts ride.
    let inner = uistate::inner(rect, [6.0; 4]);
    let usable = can_currently_be_used(command, is_client);
    let (name_color, desc_color) = if usable {
        ([1.0; 4], GOLD)
    } else {
        (DARK_GRAY, DARK_GRAY)
    };
    // `Utils.DrawBorderString(name, inner + (num + 6, -2), color, 1.1)`
    // (`:69`, `num = 6`).
    text::draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        inner[0] + 12.0,
        inner[1] - 2.0,
        1.1,
        &[[1.0, 1.0]],
        name_color,
        command.name,
    );
    // `Utils.DrawBorderString(desc, inner + (num + 6 + 180 + 16, 2 + 21),
    // color2, 0.8, 0, 1)` (`:70`) — the Y anchor 1 lifts the pen by the
    // scaled line height.
    let desc_y = inner[1] + 23.0 - text::line_height(lookups) * 0.8;
    text::draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        inner[0] + 208.0,
        desc_y,
        0.8,
        &[[1.0, 1.0]],
        desc_color,
        command.description,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registry is the 62 attribute rows, sorted OrdinalIgnoreCase
    /// (`PopulateCommandsList`, `UIDebugCommandsList.cs:140-151`).
    #[test]
    fn the_registry_is_the_62_sorted_attribute_rows() {
        assert_eq!(COMMANDS.len(), 62);
        let mut sorted = true;
        for w in COMMANDS.windows(2) {
            if w[0].name.to_lowercase() > w[1].name.to_lowercase() {
                sorted = false;
            }
        }
        assert!(sorted, "the display order is the OrdinalIgnoreCase sort");
        assert_eq!(COMMANDS[0].name, "=");
        assert_eq!(COMMANDS[12].name, "hh");
        assert_eq!(COMMANDS[61].name, "toggleinactivewait");
        // The composite requirement values fold (`CommandRequirement.cs:12-16`).
        assert_eq!(req::CLIENT, 3);
        assert_eq!(req::ANY_AUTHORITY, 13);
        let showdebug = COMMANDS.iter().find(|c| c.name == "showdebug").unwrap();
        assert_eq!(showdebug.requirements, 12); // MultiplayerRPC | LocalServer
    }

    /// `BuildPage`'s geometry at 1600x900, hand-derived from the C#.
    #[test]
    fn the_layout_matches_the_source_geometry() {
        let screen = Vec2::new(1600.0, 900.0);
        let c = container_rect(screen);
        // 0.8 * 1600 = 1280 clamped to MaxWidth 800; HAlign .5 centres it.
        assert_eq!(c, [400.0, 220.0, 800.0, 680.0]);
        let p = panel_rect(&c);
        assert_eq!(p, [400.0, 220.0, 800.0, 570.0]);
        // The panel inner drops the 12 px UIPanel padding.
        assert_eq!(search_rect(&p), [412.0, 252.0, 200.0, 24.0]);
        // VAlign 1 of the 546 inner: 232 + 546 - 486 = 292.
        assert_eq!(list_rect(&p), [412.0, 292.0, 751.0, 486.0]);
        assert_eq!(scrollbar_rect(&p), [1168.0, 232.0, 20.0, 546.0]);
        assert_eq!(row_rect(&list_rect(&p), 2), [412.0, 362.0, 751.0, 30.0]);
        // Back: W 390, centred; y = 220 + 680 - 45 - 50 = 805.
        assert_eq!(back_rect(&c), [605.0, 805.0, 390.0, 50.0]);
        // The MinWidth clamp: a 640-wide screen keeps 600, not 512.
        let small = container_rect(Vec2::new(640.0, 480.0));
        assert_eq!(small[2], 600.0);
    }

    /// `DoesCommandMatchSearch` walks name, description AND help text
    /// (`:104-120`); whitespace shows everything (`:92-93`).
    #[test]
    fn the_search_filter_matches_all_three_fields() {
        assert_eq!(visible_commands("").len(), 62);
        assert_eq!(visible_commands("   ").len(), 62);
        // Name hit — plus the description hits ("next"/"restore"/"swap"
        // mention snapshots in their descriptions, `:111-114`).
        let rows = visible_commands("SNAP");
        let names: Vec<&str> = rows.iter().map(|&i| COMMANDS[i].name).collect();
        assert_eq!(
            names,
            ["next", "restore", "snapclear", "snapload", "snapsave", "snapshot", "snapshotdiff", "swap"]
        );
        // Description-only hit ("checkpoint" appears in record's description).
        let rows = visible_commands("gameplay checkpoint");
        assert_eq!(rows.len(), 1);
        assert_eq!(COMMANDS[rows[0]].name, "checkpoint");
        // Help-text-only hit.
        let rows = visible_commands("<dx>");
        assert_eq!(rows.len(), 1);
        assert_eq!(COMMANDS[rows[0]].name, "fakenetoffset");
        assert!(visible_commands("zzz-no-such").is_empty());
    }

    /// `CanCurrentlyBeUsed` (`UIDebugCommandItem.cs:73-80`): only the
    /// SinglePlayer-exclusive commands grey out on a client.
    #[test]
    fn only_singleplayer_exclusive_commands_grey_out_on_a_client() {
        let reload = COMMANDS.iter().find(|c| c.name == "reload").unwrap();
        let save = COMMANDS.iter().find(|c| c.name == "save").unwrap();
        assert!(can_currently_be_used(reload, false));
        assert!(!can_currently_be_used(reload, true));
        assert!(can_currently_be_used(save, true)); // Client = SP|MPClient
    }

    /// The content height drops the padding for a single row
    /// (`UIList.cs:122`).
    #[test]
    fn the_content_height_matches_the_uilist_lattice() {
        assert_eq!(content_height(0), 0.0);
        assert_eq!(content_height(1), 30.0);
        assert_eq!(content_height(3), 105.0);
    }

    /// A row click seeds the chat with the lowercased name + trailing space
    /// (`UIDebugCommandItem.LeftClick`, `:110`).
    #[test]
    fn the_row_click_seeds_the_chat_text() {
        let screen = Vec2::new(1600.0, 900.0);
        let rows = visible_commands("");
        // Row 12 ("hh") at scroll 0: y = 292 + 12*35 + 5 = 717.
        let hit = click_at(screen, Vec2::new(500.0, 717.0), 0.0, &rows);
        assert_eq!(hit, Click::Row(12));
        let cmd = &COMMANDS[12];
        assert_eq!(format!("/{} ", cmd.name.to_lowercase()), "/hh ");
        // The Back panel.
        assert_eq!(click_at(screen, Vec2::new(700.0, 820.0), 0.0, &rows), Click::Back);
        // Outside the list viewport the row is unreachable.
        assert_eq!(click_at(screen, Vec2::new(500.0, 280.0), 0.0, &rows), Click::None);
    }
}
