//! The debug-screen list rows — W4-d82 (research/18b rows 27/28):
//! `UIDebugCommandItem` (`GameContent/UI/Elements/UIDebugCommandItem.cs`) and
//! `UIDebugLoadoutItem` (`UIDebugLoadoutItem.cs`). Both are `UIPanel` rows for
//! the `UIDebugCommandsList`/`UIDebugLoadoutsList` screens (unported screens,
//! research/18 §"UIState subclasses"); the element state machines port here as
//! pure code so the screens can adopt them later (the S1-H-13 pattern).
//!
//! The shared shape: a transparent-or-tinted panel whose `MouseOver` paints the
//! hover colours and `MouseOut` restores the rest colours, a `CompareTo` sort
//! key, and a `LeftClick` that closes the fancy UI and acts. The colour pairs
//! are identical in both classes (`(76,90,149)` bg / `(50,60,86)` border); the
//! loadout's rest state is the pair at 0.2 alpha, the command's is transparent.

/// The hover/rest colour pair both rows share (`UIDebugCommandItem.cs:90-91`,
/// `UIDebugLoadoutItem.cs:129-130`): bg `(76,90,149)`, border `(50,60,86)`.
pub const HOVER_BG: [u8; 4] = [76, 90, 149, 255];
pub const HOVER_BORDER: [u8; 4] = [50, 60, 86, 255];

/// The `MouseOver`/`MouseOut` colour swap both rows perform
/// (`UIDebugCommandItem.cs:87-104`, `UIDebugLoadoutItem.cs:126-141`): hover
/// paints the full pair, out restores the rest pair. The command's rest is
/// transparent (`:33-34`, `:103`); the loadout's rest is the pair × 0.2
/// (`:38-39`, `:139-140`) — XNA `Color * float` truncation (`ui::color::mul`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowColors {
    pub background: [u8; 4],
    pub border: [u8; 4],
}

impl RowColors {
    /// The command row's rest state — both transparent (`:33-34`).
    pub const COMMAND_REST: Self = Self {
        background: [0, 0, 0, 0],
        border: [0, 0, 0, 0],
    };

    /// The loadout row's rest state — the pair × 0.2 (`:38-39`):
    /// 76·0.2=15.2→15, 90·0.2=18, 149·0.2=29.8→29; 50·0.2=10, 60·0.2=12,
    /// 86·0.2=17.2→17; alpha 255·0.2=51.
    pub const LOADOUT_REST: Self = Self {
        background: [15, 18, 29, 51],
        border: [10, 12, 17, 51],
    };

    /// `MouseOver` (`:90-91` / `:129-130`).
    pub fn mouse_over(&mut self) {
        self.background = HOVER_BG;
        self.border = HOVER_BORDER;
    }

    /// `MouseOut` (`:103` / `:139-140`) — back to the row's rest pair.
    pub fn mouse_out(&mut self, rest: Self) {
        *self = rest;
    }
}

/// `CommandRequirement` (`Testing/ChatCommands/CommandRequirement.cs:6-17`),
/// the `[Flags]` enum `CanCurrentlyBeUsed` masks against.
pub mod command_requirement {
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

/// `UIDebugCommandItem.CanCurrentlyBeUsed` (`UIDebugCommandItem.cs:73-80`):
/// a command whose requirements are *only* `SinglePlayer` (the mask
/// `Requirements & ~SinglePlayer == 0`) is unusable when `netMode != 0`.
/// `net_mode` is `Main.netMode` (0 single, 1 client, 2 server).
pub fn command_can_be_used(requirements: u32, net_mode: i32) -> bool {
    if (requirements & !command_requirement::SINGLE_PLAYER) == 0 && net_mode != 0 {
        return false;
    }
    true
}

/// `UIDebugCommandItem.LeftClick` (`:106-112`): the chat line the click seeds —
/// `"/" + Command.Name.ToLower() + " "` (`:110`). The C# also closes the fancy
/// UI and sets `drawingPlayerChat`; those are app seams, the string is the
/// element's own output. The `NewText` confirmation (`:111`) is yellow
/// `(255,255,0)` (`byte.MaxValue, byte.MaxValue, 0`).
pub fn command_chat_seed(name: &str) -> String {
    format!("/{} ", name.to_lowercase())
}

/// `UIDebugCommandItem.CompareTo` (`:82-85`): the `Order` int ascending.
pub fn command_compare(a_order: i32, b_order: i32) -> std::cmp::Ordering {
    a_order.cmp(&b_order)
}

/// `UIDebugLoadoutItem.CompareTo` (`:26-29`): the friendly `Name` string
/// (`string.CompareTo`, culture-aware ordinal-ish; the port uses Rust's byte
/// order — the names are ASCII identifiers, documented deviation).
pub fn loadout_compare(a_name: &str, b_name: &str) -> std::cmp::Ordering {
    a_name.cmp(b_name)
}

/// `UIDebugLoadoutItem.LeftClick`'s chat line (`:147`): `"Loadout " + Name +
/// " set!"` in `ChatColors.Command` = `(250,250,0)` (`ChatColors.cs:28`).
pub fn loadout_chat_line(name: &str) -> String {
    format!("Loadout {name} set!")
}
pub const LOADOUT_CHAT_COLOR: [u8; 4] = [250, 250, 0, 255];

/// `UIDebugLoadoutItem.PositionItems` (`:94-112`): one icon group's band. The
/// container is `Left = horizontalStart` (percent), `Width = end - start`
/// (percent), `Top = vertical` (percent), padding 0; each icon's `HAlign` is
/// `i / (count - 1)` when `count > 1`, else 0.5 (`:105-109`). Returns the
/// container's `(left_pct, width_pct, top_pct)` and the per-icon `HAlign`s.
pub fn position_items(
    count: usize,
    horizontal_start: f32,
    horizontal_end: f32,
    vertical: f32,
) -> ([f32; 3], Vec<f32>) {
    let container = [horizontal_start, horizontal_end - horizontal_start, vertical];
    let aligns = (0..count)
        .map(|i| {
            if count > 1 {
                i as f32 / (count - 1) as f32 // `:108`
            } else {
                0.5 // `:105`
            }
        })
        .collect();
    (container, aligns)
}

/// The four `PositionItems` calls the ctor makes (`:88-91`), in order: the
/// first-5 inventory band, the ammo band (slots 54-57), the first-3 armor band,
/// the vanity/accessory band (armor 3-9). Each is `(start, end, vertical)`.
pub const LOADOUT_BANDS: [[f32; 3]; 4] = [
    [0.1, 0.3, 0.75],  // `:88` list  (inventory 0-4)
    [0.35, 0.45, 0.75], // `:89` list2 (inventory 54-57)
    [0.65, 0.75, 0.25], // `:90` list3 (armor 0-2)
    [0.6, 0.9, 0.75],  // `:91` list4 (armor 3-9)
];

#[cfg(test)]
mod tests {
    use super::*;

    /// `CanCurrentlyBeUsed` (`UIDebugCommandItem.cs:73-80`): a SinglePlayer-only
    /// command is dead in multiplayer; anything with another requirement bit
    /// stays usable; single-player always passes.
    #[test]
    fn the_single_player_gate_matches_the_mask() {
        use command_requirement as r;
        assert!(command_can_be_used(r::SINGLE_PLAYER, 0));
        assert!(!command_can_be_used(r::SINGLE_PLAYER, 1));
        assert!(!command_can_be_used(r::SINGLE_PLAYER, 2));
        // A command with any non-SinglePlayer bit is usable everywhere.
        assert!(command_can_be_used(r::CLIENT, 1));
        assert!(command_can_be_used(r::LOCAL_SERVER, 2));
        assert!(command_can_be_used(r::ALL, 1));
        // Zero requirements: the mask is 0, so it is SinglePlayer-only too.
        assert!(!command_can_be_used(0, 1));
    }

    /// The click outputs (`:110`, `:147`) and the sort keys (`:82-85`, `:26-29`).
    #[test]
    fn the_click_strings_and_sort_keys() {
        assert_eq!(command_chat_seed("GiveItem"), "/giveitem ");
        assert_eq!(loadout_chat_line("Starter"), "Loadout Starter set!");
        assert_eq!(LOADOUT_CHAT_COLOR, [250, 250, 0, 255]);
        assert_eq!(command_compare(1, 2), std::cmp::Ordering::Less);
        assert_eq!(loadout_compare("a", "b"), std::cmp::Ordering::Less);
    }

    /// The hover/rest swap (`:87-104` / `:126-141`), incl. the ×0.2 rest pair.
    #[test]
    fn the_row_colors_swap_and_restore() {
        let mut c = RowColors::COMMAND_REST;
        c.mouse_over();
        assert_eq!(c.background, HOVER_BG);
        assert_eq!(c.border, HOVER_BORDER);
        c.mouse_out(RowColors::COMMAND_REST);
        assert_eq!(c.background, [0, 0, 0, 0]);
        // The loadout rest is the pair truncated at 0.2 (`color::mul`).
        assert_eq!(
            RowColors::LOADOUT_REST.background,
            crate::ui::color::mul(HOVER_BG, 0.2)
        );
        assert_eq!(
            RowColors::LOADOUT_REST.border,
            crate::ui::color::mul(HOVER_BORDER, 0.2)
        );
    }

    /// `PositionItems` (`:94-112`): the band container + the HAlign spread.
    #[test]
    fn the_loadout_bands_spread_their_icons() {
        let (c, a) = position_items(5, 0.1, 0.3, 0.75);
        assert_eq!(c, [0.1, 0.2, 0.75]);
        assert_eq!(a, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        // A single icon centres (`:105`).
        let (_, single) = position_items(1, 0.65, 0.75, 0.25);
        assert_eq!(single, vec![0.5]);
        // The four ctor bands (`:88-91`).
        assert_eq!(LOADOUT_BANDS[0], [0.1, 0.3, 0.75]);
        assert_eq!(LOADOUT_BANDS[3], [0.6, 0.9, 0.75]);
    }
}

