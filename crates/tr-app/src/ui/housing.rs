//! `Main.DrawNPCHousesInUI` (`decompiled/Terraria/Main.cs:41993-42107`): the
//! inventory's second tab, where every town NPC the world holds gets one
//! `Inventory_Back11` cell in `NPCHeadID.Sets.HeadListOrder`, followed by the
//! housing-query cell (`NPCHeadID.HousingQuery`, head 0, `Lang.inter[8]`).
//!
//! The C# walks `Main.maxNPCs`, keeps the first *type* it sees for each head
//! (`_npcTypesThatAlreadyDrewAHead` + `Main._npcIndexWhoHoldsHeadIndex`,
//! `:42008-42023`), then lays the surviving heads out in columns on the right
//! edge, wrapping to the next column when a row would land below
//! `screenHeight - 80` (`:42044-42058`). This module ports the head resolution
//! and that layout, plus the hover text (`:42064`: `Main.npc[n].FullName`, or
//! `Lang.inter[8]` for the query cell). The click (`SetMouseNPC` /
//! `SetMouseNPC_ToHousingQuery`, `:42065-42071`) is not ported: it needs a
//! `Main.mouseNPC` housing query, which the port has no model for.

use tr_render::SpriteRenderer;

use crate::assets;
use crate::npc::{Npc, NpcKind};

/// `NPCHeadID.Sets.HeadListOrder` (`ID/NPCHeadID.cs:19-30`) — the draw order,
/// not the head numbering: Guide first, the housing-query 0 last of the first
/// column, and the 46..53 set (the old/extra heads) at the tail.
pub const HEAD_ORDER: [usize; 81] = [
    0, 1, 72, 2, 63, 4, 59, 3, 64, 6, 74, 20, 69, 5, 73, 22, 55, 14, 60, 17, 65, 25, 61, 26, 56,
    18, 78, 15, 77, 24, 79, 7, 57, 8, 62, 9, 75, 23, 70, 10, 71, 12, 76, 19, 66, 13, 68, 16, 58,
    11, 67, 21, 80, 45, 54, 39, 40, 41, 42, 43, 44, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
    38, 46, 49, 51, 48, 53, 52, 47, 50,
];

/// `NPCHeadID.HousingQuery` (`:13`): the "?" cell that starts a housing query.
pub const HOUSING_QUERY: usize = 0;

/// `NPCHeadID.Count` (`:15`): the sheet set is `0..=80` (`HeadListOrder` has 81
/// entries), which is also the length [`Lookups::npc_heads`] is keyed by.
pub const HEAD_COUNT: usize = 81;

/// `NPCHeadID.Sets.CannotBeDrawnInHousingUI` (`:17`): Santa (21) and the
/// Skeleton Merchant's neighbour (80) have no housing entry.
fn cannot_draw_in_housing_ui(head: usize) -> bool {
    matches!(head, 21 | 80)
}

/// `NPC.TypeToDefaultHeadIndex` (`NPC.cs:7510-7553`) — the vanilla NPC type to
/// its head sheet. Everything the port ships outside this switch (the bound /
/// sleeping forms, the bosses, the critters) has no head cell: the C# returns
/// `-1` for them too and the housing loop skips a head index `<= 0`.
pub fn type_to_head_index(ty: i32) -> i32 {
    match ty {
        0 => 0,
        17 => 2,
        18 => 3,
        19 => 6,
        20 => 5,
        22 => 1,
        38 => 4,
        54 => 7,
        107 => 9,
        108 => 10,
        124 => 8,
        142 => 11,
        160 => 12,
        178 => 13,
        207 => 14,
        208 => 15,
        209 => 16,
        227 => 17,
        228 => 18,
        229 => 19,
        353 => 20,
        368 => 21,
        369 => 22,
        441 => 23,
        550 => 24,
        588 => 25,
        633 => 26,
        663 => 45,
        637 => 27,
        638 => 33,
        656 => 39,
        670 => 46,
        678 => 47,
        679 => 48,
        680 => 49,
        681 => 50,
        682 => 51,
        683 => 52,
        684 => 53,
        _ => -1,
    }
}

/// The head cell of a ported NPC, resolved through its vanilla type
/// (`NpcKind::net_id`, i.e. `NPC.type`).
pub fn head_of(kind: NpcKind) -> Option<usize> {
    // The bound forms have no head in vanilla — their types (105/106/123) are
    // absent from `TypeToDefaultHeadIndex` — but `NpcKind::net_id` maps them
    // onto the freed row for the bestiary (`bestiary.rs:1502-1506`), which
    // would hand them the freed NPC's head. Reject them here instead.
    if matches!(
        kind,
        NpcKind::BoundGoblin | NpcKind::BoundWizard | NpcKind::BoundMechanic
    ) {
        return None;
    }
    let head = type_to_head_index(kind.net_id());
    if head <= 0 || head >= HEAD_COUNT as i32 || cannot_draw_in_housing_ui(head as usize) {
        return None;
    }
    Some(head as usize)
}

/// `Main.cs:42008-42023`: one cell per *type* — the head index of the first
/// active NPC of that type wins. Returns `(head, npc index)` in `npcs` order of
/// first appearance, which is the order the C# fills
/// `_npcIndexWhoHoldsHeadIndex`.
pub fn heads_for(npcs: &[Npc]) -> Vec<(usize, usize)> {
    let mut drawn: Vec<(usize, usize)> = Vec::new();
    let mut seen_kinds: Vec<NpcKind> = Vec::new();
    for (i, npc) in npcs.iter().enumerate() {
        if seen_kinds.contains(&npc.kind) {
            continue;
        }
        let Some(head) = head_of(npc.kind) else {
            continue;
        };
        // The C# also refuses a head whose slot is already taken
        // (`_npcIndexWhoHoldsHeadIndex[head] == -1`), which only bites when two
        // types resolve to one head — impossible while every type maps to its
        // own sheet, but cheap to keep faithful.
        if drawn.iter().any(|(h, _)| *h == head) {
            continue;
        }
        seen_kinds.push(npc.kind);
        drawn.push((head, i));
    }
    drawn
}

/// One cell the page draws: `head == HOUSING_QUERY` is the "?" cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub head: usize,
    /// The `Main.npc[]` entry the cell stands for (`None` for the query cell).
    pub npc: Option<usize>,
    pub x: f32,
    pub y: f32,
}

/// `Main.DrawNPCHousesInUI`'s layout loop (`:42044-42058`): walk
/// [`HEAD_ORDER`], skipping heads nobody owns, and wrap a column every time the
/// next row would cross `inventory_top + (screenHeight - 80 - inventory_top)`
/// (`num9 > Main.screenHeight - 80`). A wrap moves the column 48 px left
/// (`num3 -= 48`) and re-bases the rows on `inventory_top`
/// (`num2 -= num9 - inventory_top`).
///
/// `scale` is `Main.inventoryScale` (0.85 normally, 0.755 with a chest open),
/// `inventory_top` is `174 + m_h` at this screen height (`Main.cs:41043`).
pub fn layout(
    heads: &[(usize, usize)],
    screen: [f32; 2],
    inventory_top: f32,
    scale: f32,
) -> Vec<Cell> {
    let mut cells = Vec::new();
    // `num` (row), `num2` (row offset), `num3` (column offset), `num5`
    // (icons in this column).
    let mut row = 0i32;
    let mut num2 = 0i32;
    let mut num3 = 0i32;
    let top = inventory_top as i32;
    for &head in HEAD_ORDER.iter() {
        let npc = if head == HOUSING_QUERY {
            None
        } else {
            match heads.iter().find(|(h, _)| *h == head) {
                Some((_, i)) => Some(*i),
                None => continue,
            }
        };
        // The C# truncates the row to an int before adding the offset
        // (`:42045`: `(int)(inventoryTop + num * 56f * scale) + num2`), so the
        // cells land on whole pixels and the wrap comparison is int-to-int.
        let row_y = |row: i32, num2: i32| (inventory_top + (row as f32 * 56.0) * scale) as i32 + num2;
        let mut x = screen[0] - 64.0 - 28.0 + num3 as f32;
        let mut y = row_y(row, num2);
        if y > (screen[1] - 80.0) as i32 {
            num3 -= 48;
            num2 -= y - top;
            x = screen[0] - 64.0 - 28.0 + num3 as f32;
            y = row_y(row, num2);
        }
        cells.push(Cell { head, npc, x, y: y as f32 });
        row += 1;
    }
    cells
}

/// The cell under the pointer, as `(head, npc)` — the C# hover probe
/// (`:42059-42065`) that feeds `Main.mouseText` and `SetMouseNPC`.
pub fn hover(cells: &[Cell], mouse: [f32; 2], scale: f32) -> Option<(usize, Option<usize>)> {
    let size = 52.0 * scale; // `(int)vector.X * Main.inventoryScale`
    cells.iter().rev().find_map(|c| {
        let inside = mouse[0] >= c.x
            && mouse[0] <= c.x + size
            && mouse[1] >= c.y
            && mouse[1] <= c.y + size;
        inside.then_some((c.head, c.npc))
    })
}

/// `Main.hidePVPAndTeamIcons` (`Main.cs:42053-42056`): when a wrap pushes the
/// column offset to `-144` or further left, vanilla drops the PVP/team icons so
/// the third housing column does not collide with them. The offset is
/// `x - (screenWidth - 64 - 28)` (`:42044`'s `num3`).
pub fn hide_pvp_icons(cells: &[Cell], screen_width: f32) -> bool {
    let origin = screen_width - 64.0 - 28.0;
    cells.iter().any(|c| (c.x - origin) <= -144.0)
}

/// The localized type name of a town kind — the C# reads
/// `Main.npc[num].FullName`, which is the *given* name ("Andrew") plus the
/// type name; the port stores no per-NPC names, so the type name is what the
/// tooltip can honestly show (recorded in `docs/ANALYSIS.md`).
///
/// `Lang.GetNPCName(netId)` (`Lang.cs:188-195`) answers it from the
/// `NPCName.{const name}` key, so the table is the id class' own spellings
/// (`NPCID.cs`) instead of a hand-written list.
fn npc_type_name(kind: NpcKind) -> Option<String> {
    let name = crate::lang::npc_name(kind.cs_id() as i32);
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// `Main.cs:42062-42063`: `text = (num6 != 0) ? Main.npc[num7].FullName :
/// Lang.inter[8].Value`. Returns the hovered cell's label, `None` when the
/// pointer is over no cell.
pub fn hover_text(
    cells: &[Cell],
    mouse: [f32; 2],
    scale: f32,
    npcs: &[Npc],
) -> Option<String> {
    let (head, npc) = hover(cells, mouse, scale)?;
    label(head, npc, npcs)
}

/// The label of one cell — the `InventoryHit::housing` half of [`hover_text`],
/// so the panel can hand a `Copy` pair back to the tooltip layer.
pub fn label(head: usize, npc: Option<usize>, npcs: &[Npc]) -> Option<String> {
    if head == HOUSING_QUERY {
        // `Lang.inter[8]`: the housing-query label.
        return Some(crate::lang::text("Inter.8"));
    }
    let kind = npcs.get(npc?)?.kind;
    Some(match npc_type_name(kind) {
        Some(name) => name,
        // A head with no key keeps its vanilla type surface rather than an
        // empty tooltip.
        None => crate::lang::text("Inter.8"),
    })
}

/// `Main.inventoryBack` — the 0.863 grey every inventory background is tinted
/// with (`Main.cs:21598`'s `new Color(220, 220, 220, 220)`).
const INVENTORY_BACK: [f32; 4] = [220.0 / 255.0; 4];

/// The head icon's draw rect inside a cell: the C# centres the sheet on
/// `(x + 26 * scale, y + 26 * scale)` and shrinks anything wider/taller than 36
/// px to `36 / max(w, h)` (`:42089-42095`).
fn head_rect(cell: &Cell, size: [f32; 2], scale: f32) -> ([f32; 2], [f32; 2]) {
    let icon_scale = if size[0].max(size[1]) > 36.0 {
        36.0 / size[0].max(size[1])
    } else {
        1.0
    };
    let center = [
        cell.x + 26.0 * scale,
        cell.y + 26.0 * scale,
    ];
    let drawn = [size[0] * icon_scale, size[1] * icon_scale];
    (
        [center[0] - drawn[0] / 2.0, center[1] - drawn[1] / 2.0],
        drawn,
    )
}

/// Draw the page: the [`Cell`] backgrounds (`Inventory_Back11`) and their head
/// icons (`NPC_Head_<n>`). A missing sheet simply leaves the cell empty — the
/// port packs only the heads of the town NPCs it ships (`assets.rs`).
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    cells: &[Cell],
    scale: f32,
) {
    let size = [52.0 * scale; 2];
    for cell in cells {
        // `:42079-42084`: `Inventory_Back11` in `Main.inventoryBack`. The
        // `Inventory_Back14` white swap there is the gamepad UI-point
        // highlight (`UILinkPointNavigator.CurrentPoint - 600 == num`), which
        // the port does not model — same reason the grid's `_17` highlight is
        // still open (ROADMAP M7-UI-2 remainder).
        if let Some(bg) = lookups.ui_back11 {
            if let Some(uv) = lookups.hud_uv(Some(&bg)) {
                r.push_ui_sprite([cell.x, cell.y], size, uv, INVENTORY_BACK);
            }
        }
        if let Some(head) = lookups.npc_heads.get(cell.head).copied().flatten() {
            let (pos, drawn) = head_rect(cell, [head.width as f32, head.height as f32], scale);
            if let Some(uv) = lookups.hud_uv(Some(&head)) {
                r.push_ui_sprite(pos, drawn, uv, [1.0; 4]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn npc(kind: NpcKind) -> Npc {
        Npc::new(kind, 0.0, 0.0, 1, 0.0)
    }

    #[test]
    fn the_head_table_matches_the_vanilla_switch() {
        // `NPC.cs:7510-7553`, spot-checked: the ten town NPCs the port
        // guarantees a housing entry for, plus the Guide's query neighbour.
        assert_eq!(type_to_head_index(22), 1, "Guide");
        assert_eq!(type_to_head_index(17), 2, "Merchant");
        assert_eq!(type_to_head_index(18), 3, "Nurse");
        assert_eq!(type_to_head_index(38), 4, "Demolitionist");
        assert_eq!(type_to_head_index(20), 5, "Dryad");
        assert_eq!(type_to_head_index(19), 6, "Arms Dealer");
        assert_eq!(type_to_head_index(54), 7, "Clothier");
        assert_eq!(type_to_head_index(124), 8, "Mechanic");
        assert_eq!(type_to_head_index(107), 9, "Goblin Tinkerer");
        assert_eq!(type_to_head_index(108), 10, "Wizard");
        assert_eq!(type_to_head_index(142), 11, "Witch Doctor");
        assert_eq!(type_to_head_index(227), 17, "Painter");
        assert_eq!(type_to_head_index(663), 45, "Princess");
        assert_eq!(type_to_head_index(4), -1, "the Eye of Cthulhu has no head");
        assert_eq!(type_to_head_index(106), -1, "the bound Wizard has none either");
    }

    #[test]
    fn heads_are_resolved_per_type_in_first_appearance_order() {
        let npcs = vec![
            npc(NpcKind::Guide),
            npc(NpcKind::Slime),
            npc(NpcKind::Guide),
            npc(NpcKind::Merchant),
            npc(NpcKind::BoundWizard),
        ];
        // The bound Wizard is the trap: its `net_id` is the freed Wizard's row
        // (`bestiary.rs:1502`), so a naive lookup would give him head 10.
        assert_eq!(heads_for(&npcs), vec![(1, 0), (2, 3)]);
        // Santa and the Skeleton Merchant's head 80 are excluded outright
        // (`NPCHeadID.Sets.CannotBeDrawnInHousingUI`).
        assert!(cannot_draw_in_housing_ui(21));
        assert!(cannot_draw_in_housing_ui(80));
        assert!(!cannot_draw_in_housing_ui(1));
    }

    #[test]
    fn the_page_wraps_a_column_before_the_bottom_band() {
        // Guide + Merchant + Nurse at 900 px height: `inventory_top` = 324,
        // 56 * 0.85 = 47.6 a row. Rows 0..9 fit (324 + 9*47.6 = 752 < 820);
        // row 10 lands at 800 — still under `900 - 80`; row 11 would be 847.
        let heads: Vec<(usize, usize)> = HEAD_ORDER
            .iter()
            .filter(|h| **h != HOUSING_QUERY && **h < 60)
            .enumerate()
            .map(|(i, h)| (*h, i))
            .collect();
        let cells = layout(&heads, [1600.0, 900.0], 324.0, 0.85);
        // The first cell is the housing query at the top-right corner:
        // `1600 - 64 - 28` = 1508, `324`.
        assert_eq!((cells[0].head, cells[0].x, cells[0].y), (HOUSING_QUERY, 1508.0, 324.0));
        assert_eq!(cells[0].npc, None);
        // Every cell before the wrap shares x; the wrap moves 48 px left and
        // starts the column again at `inventory_top`.
        let wrap = cells.iter().position(|c| c.x < 1508.0).expect("a second column");
        assert_eq!(cells[wrap].x, 1460.0);
        assert_eq!(cells[wrap].y, 324.0, "the wrapped column re-bases on inventoryTop");
        assert!(cells[wrap - 1].y <= 820.0, "the last row of column 1 fits");
        assert!(cells.iter().all(|c| c.y <= 820.0), "no row crosses the band");
    }

    #[test]
    fn the_hover_probe_reads_a_cell() {
        let cells = vec![
            Cell { head: HOUSING_QUERY, npc: None, x: 1508.0, y: 324.0 },
            Cell { head: 2, npc: Some(7), x: 1508.0, y: 371.6 },
        ];
        assert_eq!(hover(&cells, [1540.0, 400.0], 0.85), Some((2, Some(7))));
        assert_eq!(hover(&cells, [1540.0, 330.0], 0.85), Some((HOUSING_QUERY, None)));
        assert_eq!(hover(&cells, [1400.0, 330.0], 0.85), None);
    }

    #[test]
    fn a_third_column_hides_the_pvp_icons() {
        // `Main.cs:42053-42056`: `num3 <= -144` sets `hidePVPAndTeamIcons`.
        let two = vec![
            Cell { head: HOUSING_QUERY, npc: None, x: 1508.0, y: 324.0 },
            Cell { head: 2, npc: Some(0), x: 1460.0, y: 324.0 },
            Cell { head: 3, npc: Some(1), x: 1412.0, y: 324.0 },
        ];
        // Two wraps put the third column at `-96` (`1508 - 96 = 1412`), still
        // right of the `-144` cut, so the icons stay.
        assert!(!hide_pvp_icons(&two, 1600.0), "third column is -96");
        let three = vec![
            Cell { head: HOUSING_QUERY, npc: None, x: 1508.0, y: 324.0 },
            Cell { head: 2, npc: Some(0), x: 1412.0, y: 324.0 },
            Cell { head: 3, npc: Some(1), x: 1364.0, y: 324.0 },
        ];
        assert!(hide_pvp_icons(&three, 1600.0), "the fourth column is -144");
        assert!(!hide_pvp_icons(&three[..1], 1600.0));
    }

    #[test]
    fn the_hover_text_names_the_cell() {
        let cells = vec![
            Cell { head: HOUSING_QUERY, npc: None, x: 1508.0, y: 324.0 },
            Cell { head: 2, npc: Some(0), x: 1508.0, y: 371.0 },
        ];
        let npcs = vec![npc(NpcKind::Merchant)];
        // `Lang.inter[8]` — the housing-query label.
        let query = hover_text(&cells, [1530.0, 340.0], 0.85, &npcs);
        assert_eq!(query.as_deref(), Some(crate::lang::text("Inter.8").as_str()));
        // `Main.npc[0].FullName` → the type name while the port has no given
        // names (`NPCName.Merchant`).
        let merchant = hover_text(&cells, [1530.0, 390.0], 0.85, &npcs);
        assert_eq!(merchant.as_deref(), Some(crate::lang::text("NPCName.Merchant").as_str()));
        // Off the cells: no text.
        assert!(hover_text(&cells, [10.0, 10.0], 0.85, &npcs).is_none());
    }

    #[test]
    fn the_head_icon_shrinks_to_thirty_six_pixels() {
        let cell = Cell { head: 2, npc: Some(0), x: 1508.0, y: 324.0 };
        let (pos, drawn) = head_rect(&cell, [52.0, 52.0], 0.85);
        assert_eq!(drawn, [36.0, 36.0]);
        assert_eq!(pos, [1508.0 + 26.0 * 0.85 - 18.0, 324.0 + 26.0 * 0.85 - 18.0]);
        // A small sheet keeps its own size, centred on the same point.
        let (_, small) = head_rect(&cell, [20.0, 20.0], 0.85);
        assert_eq!(small, [20.0, 20.0]);
    }
}
