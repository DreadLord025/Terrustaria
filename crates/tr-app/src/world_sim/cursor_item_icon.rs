//! The `cursorItemIcon*` cursor-overlay feed (W4-g86 cursor-item-icon lane).
//!
//! The full system census + owner contract lives in
//! `docs/research/244-cursor-item-icon-seed.md`: 357 write sites across 7 C#
//! files (Player.cs alone carries 326 writes / 169 `cursorItemIconID`
//! writes — the ledger's "~170 sites"). Phase 1 (this lane) landed the
//! state fields (`cursorItemIconID/Text/Push/Reversed` on App beside g81's
//! `cursor_item_icon_enabled`), the full `MouseOversClear` quad
//! (`Main.cs:46503-46513`, `frame.rs`'s world-branch head) + the :16842
//! once-per-frame Reversed clear (`frame_head`), the layer-40 ID-override
//! draw arm (`DrawInterface_40_InteractItemIcon` `Main.cs:45057-45105`,
//! `ui/mod.rs` — g81's declared gap) and WIRED both batches below at their
//! C# call positions in `editing.rs`. This module holds the pure decode
//! arms in ≤10-site batches; the wiring writes ride the C# field order per
//! function. Zero RNG — every arm is integer maths.
//!
//! Batch 1 (WIRED at `update_editing`'s tile-hover head): the altar hover
//! icons — `TileInteractionsMouseOver`'s tile-26
//! (`Player.cs:34524-34536`) and tile-237 (`:34309-34314`) arms, 4 cursor
//! write sites.
//!
//! Batch 2 (WIRED at `update_editing`'s ItemCheck head): the `ItemCheck`
//! head's held-item icon arms (`Player.cs:42615-42650`) — the fishing
//! pole's bait icon (push 6) and the Clentaminator/Terraformer's solution
//! icon (push 10), 9 cursor write sites + the `Fishing_GetBait` scan
//! (`:42254-42280`) they read.
//!
//! Batch 3 (§5 arms 1-10, WIRED at the tile-hover head): the bed 79
//! (`:33593-33610`, the bottom-side test + the snap-96 Sleeping Icon), the
//! pylon 597 (`:33611-33658`), the constant arms 621/464/699
//! (`:33659-33676`), the candles 33 (`:33677-33682`), the container TEXT
//! arms 21/467 (`:33683-33690` → `TileInteractionsMouseOver_Containers`
//! `:34755-34813`) and the fake-chest icon arms 441/468
//! (`:33691-33732`, `WorldGen.GetChestIcon`). The styled drop tables
//! (`GetItemDrop_Beds/Candles/Chests`, `GetChestIcon` + the
//! `Chest.Initialize` overrides) are ported here — `registry::tile_drop`
//! is the style-0 slice only (research/244 §4.3).

/// `Player.TileInteractionsMouseOver`'s altar arms (`Player.cs:34524-34536`
/// tile 26, `:34309-34314` tile 237): the hover cursor icon for the
/// demon/crimson altar and the Lihzahrd altar. Returns the
/// `cursorItemIconID` the arm writes; `None` = the kind has no altar arm.
///
/// Tile 26 (`:34528-34535`): `num41 = frameX / 18`, then the
/// `while (num41 > 2) { num41 -= 3; num42++; }` column-group loop — group 0
/// (the first 3 frame columns, the demon altar's 3-wide sheet) draws the
/// Demon Altar item 6135, every later group the Crimson Altar 6136. The
/// loop is kept verbatim (NOT `frameX / 54`): the C# truncates `frameX/18`
/// first, so a mid-group column (e.g. `frameX = 90`, column 5) still lands
/// in group 1 exactly like the source.
///
/// Tile 237 (`:34313`): always the Lihzahrd Power Cell 1293 — the summon
/// item, matching the outline feed's `HasItem(1293)` gate
/// (`world_sim/smart_interact.rs`, `FillPotentialTargetTiles` `:453-457`).
///
/// Wiring (research/244 phase 1, WIRED): `update_editing`'s tile-hover head
/// (`editing.rs`, the `TileInteractionsCheck` → `TileInteractionsMouseOver`
/// seam, `Player.cs:32039`) sets `noThrow = 2` + `cursorItemIconEnabled =
/// true` BEFORE the ID write (both arms, `:34526-34527`/`:34311-34312`),
/// and the layer-40 ID override draws it. The g86 outline pass is the
/// SELECTION feedback; this icon is the distinct hover overlay (ANALYSIS
/// "The altar-26 SUMMON-USE chain — census").
pub(crate) fn altar_hover_icon(kind: u16, frame_x: i16) -> Option<u16> {
    match kind {
        // `Player.cs:34524-34536` — the demon/crimson altar.
        26 => {
            let mut num41 = frame_x as i32 / 18;
            let mut num42 = 0;
            while num41 > 2 {
                num41 -= 3;
                num42 += 1;
            }
            Some(if num42 == 0 { 6135 } else { 6136 })
        }
        // `Player.cs:34309-34314` — the Lihzahrd altar.
        237 => Some(1293),
        _ => None,
    }
}

/// `Player.Fishing_GetBait` (`Player.cs:42254-42280`): the bait scan the
/// `ItemCheck` head's pole icon reads — the ammo slots 54-57 FIRST
/// (`inventory[54..58]`, the port's `equip.ammo`), then the grid 0-49
/// (`inventory[0..50]`, NOT the coin rows — the C# loop is `j < 50`).
/// First `stack > 0 && bait > 0` slot wins; returns `(baitPower, baitType)`
/// with `(0, 0)` when no bait is carried. The scan is the SAME one
/// `ConsumeBait` uses (`:52452-52470`, the `fishing_use` precedent) — this
/// is the icon-side read, no consumption. WIRED (research/244 phase 1) via
/// [`item_check_head_icon`] at `update_editing`'s head.
pub(crate) fn fishing_get_bait(
    grid: &[crate::item::ItemSlot],
    ammo: &[crate::item::ItemSlot],
) -> (i32, u16) {
    // `:42258-42266` — the ammo row first.
    for slot in ammo {
        let bait = crate::registry::get(slot.kind).map_or(0, |d| d.bait as i32);
        if slot.count > 0 && bait > 0 {
            return (bait, slot.kind);
        }
    }
    // `:42271-42279` — the grid 0-49 (the coin/ammo rows are NOT scanned).
    for slot in grid.iter().take(50) {
        let bait = crate::registry::get(slot.kind).map_or(0, |d| d.bait as i32);
        if slot.count > 0 && bait > 0 {
            return (bait, slot.kind);
        }
    }
    (0, 0)
}

/// The `cursorItemIconPush` offset the `ItemCheck` head's arms write:
/// the fishing pole's bait icon pushes 6 (`Player.cs:42622`), the
/// Clentaminator's solution icon pushes 10 (`:42633`/`:42645`). The draw
/// adds it to the base 10 (`Main.cs:45089`, `10 + cursorItemIconPush`).
pub(crate) const BAIT_ICON_PUSH: i32 = 6;
/// See [`BAIT_ICON_PUSH`] — the solution arm's push (`Player.cs:42633`).
pub(crate) const SOLUTION_ICON_PUSH: i32 = 10;

/// `Player.ItemCheck`'s head icon arms (`Player.cs:42615-42650`), the
/// local-player block (`Main.myPlayer == whoAmI`, `:42608`). Both arms
/// gate on `!cursorItemIconEnabled` — an earlier hover arm (the tile pass
/// runs first in `Player.Update`) wins; the held-item icon only fills an
/// idle cursor. Returns `(iconID, push)` for the arm that fires, `None`
/// when neither does.
///
/// Arm 1 (`:42615-42624`): a held fishing pole (`item.stack > 0 &&
/// item.fishingPole > 0`) draws the first bait `Fishing_GetBait` finds
/// (`baitType > 0`), push 6.
///
/// Arm 2 (`:42625-42650`): the Clentaminator 779 / Terraformer 5134 draw
/// the first solution matching `item.useAmmo` (780) — the ammo slots
/// 54-57 first (`:42627-42636`), then `inventory[0..54]`
/// (`:42639-42648`, the C# `j < 54` — the 50 grid slots PLUS the 4 coin
/// rows, unlike the bait scan's `j < 50`), push 10. The ammo-row scan
/// breaks on the first match; the 0-53 scan only runs when the ammo row
/// found nothing (`!cursorItemIconEnabled`, `:42637`). The port's layout
/// splits the C# 58-slot array: `grid` = inventory[0..50], `coins` =
/// inventory[50..54], `ammo` = inventory[54..58].
///
/// Wiring (research/244 phase 1, WIRED): `update_editing`'s head
/// (`editing.rs`) runs this AFTER the CCed gate (`:42598`) and BEFORE the
/// countdowns / the early-dismount head (the `ItemCheck_HandleMount`
/// neighbour, `:42652`); on `Some((id, push))` it writes
/// `cursorItemIconEnabled = true`, `cursorItemIconID = id`,
/// `cursorItemIconPush = push` (the C# order). NO `ItemIconCacheUpdate`
/// here — these arms do not arm the linger (only the tool latches do).
pub(crate) fn item_check_head_icon(
    held: crate::item::ItemSlot,
    grid: &[crate::item::ItemSlot],
    coins: &[crate::item::ItemSlot],
    ammo: &[crate::item::ItemSlot],
    icon_enabled: bool,
) -> Option<(u16, i32)> {
    if icon_enabled || held.count == 0 {
        return None;
    }
    let def = crate::registry::get(held.kind)?;
    // `:42615-42624` — the fishing pole's bait icon.
    if def.fishing_pole > 0 {
        let (_, bait_type) = fishing_get_bait(grid, ammo);
        if bait_type > 0 {
            return Some((bait_type, BAIT_ICON_PUSH));
        }
        // The C# falls through to the 779/5134 check with the icon still
        // disabled — a pole is neither, so the arm ends here.
        return None;
    }
    // `:42625-42650` — the Clentaminator/Terraformer's solution icon.
    if held.kind == 779 || held.kind == 5134 {
        // `:42627-42636` — the ammo row (inventory[54..58]).
        for slot in ammo {
            let slot_ammo = crate::registry::get(slot.kind).map_or(0, |d| d.ammo);
            if slot_ammo == def.use_ammo && slot.count > 0 {
                return Some((slot.kind, SOLUTION_ICON_PUSH));
            }
        }
        // `:42639-42648` — inventory[0..54]: the grid then the coin rows.
        for slot in grid.iter().take(50).chain(coins.iter()) {
            let slot_ammo = crate::registry::get(slot.kind).map_or(0, |d| d.ammo);
            if slot_ammo == def.use_ammo && slot.count > 0 {
                return Some((slot.kind, SOLUTION_ICON_PUSH));
            }
        }
    }
    None
}

// ---- Batch 3 (§5 arms 1-10): the styled drop tables + the hover arms ------

/// `WorldGen.GetItemDrop_Beds` (`WorldGen.cs:39905-40087`), verbatim: the
/// bed style's item. Default 224; styles 1-3 → `style + 643`, 4 → 920,
/// 5-8 → `1465 + style`, 9-12 → `1710 + style`, 13-18 → `2066 + style - 13`,
/// then the per-style tail (19..64). Out-of-range keeps the 224 default
/// (the C# `result` init).
pub(crate) fn item_drop_beds(style: i32) -> u16 {
    match style {
        0 => 224,
        1..=3 => (style + 643) as u16,
        4 => 920,
        5..=8 => (1465 + style) as u16,
        9..=12 => (1710 + style) as u16,
        13..=18 => (2066 + style - 13) as u16,
        19 => 2139,
        20 => 2140,
        21 => 2231,
        22 => 2520,
        23 => 2538,
        24 => 2553,
        25 => 2568,
        26 => 2669,
        27 => 2811,
        28 => 3162,
        29 => 3164,
        30 => 3163,
        31 => 3897,
        32 => 3932,
        33 => 3959,
        34 => 4146,
        35 => 4167,
        36 => 4188,
        37 => 4209,
        38 => 4299,
        39 => 4567,
        40 => 5149,
        41 => 5170,
        42 => 5191,
        43 => 5549,
        44 => 5602,
        45 => 5690,
        46 => 5713,
        47 => 5740,
        48 => 5757,
        49 => 5778,
        50 => 5799,
        51 => 5820,
        52 => 5841,
        53 => 5859,
        54 => 5880,
        55 => 5899,
        56 => 5933,
        57 => 5956,
        58 => 5976,
        59 => 5999,
        60 => 6022,
        61 => 6045,
        62 => 6068,
        63 => 6091,
        64 => 6112,
        _ => 224,
    }
}

/// `WorldGen.GetItemDrop_Candles` (`WorldGen.cs:39724-39903`), verbatim:
/// default 105; 1/2/3 → 1405/1406/1407, 4-13 → `2045 + style - 4`,
/// 14-16 → `2153 + style - 14`, then the per-style tail (17..63).
pub(crate) fn item_drop_candles(style: i32) -> u16 {
    match style {
        1 => 1405,
        2 => 1406,
        3 => 1407,
        4..=13 => (2045 + style - 4) as u16,
        14..=16 => (2153 + style - 14) as u16,
        17 => 2236,
        18 => 2523,
        19 => 2542,
        20 => 2556,
        21 => 2571,
        22 => 2648,
        23 => 2649,
        24 => 2650,
        25 => 2651,
        26 => 2818,
        27 => 3171,
        28 => 3173,
        29 => 3172,
        30 => 3890,
        31 => 3936,
        32 => 3962,
        33 => 4150,
        34 => 4171,
        35 => 4192,
        36 => 4213,
        37 => 4303,
        38 => 4571,
        39 => 5153,
        40 => 5174,
        41 => 5195,
        42 => 5553,
        43 => 5606,
        44 => 5694,
        45 => 5717,
        46 => 5743,
        47 => 5760,
        48 => 5781,
        49 => 5802,
        50 => 5823,
        51 => 5844,
        52 => 5862,
        53 => 5883,
        54 => 5902,
        55 => 5936,
        56 => 5959,
        57 => 5979,
        58 => 6002,
        59 => 6025,
        60 => 6048,
        61 => 6071,
        62 => 6094,
        63 => 6115,
        _ => 105,
    }
}

/// `WorldGen.GetItemDrop_Chests` (`WorldGen.cs:41554-41839`), verbatim:
/// the chest style's item. `second_type` = the Chest2 family (tile 467,
/// base 3884, styles 0..37); else the base family (tile 21, base 48,
/// styles 0..51). Out-of-range keeps the family base (the C# `num` init).
pub(crate) fn item_drop_chests(style: i32, second_type: bool) -> u16 {
    if second_type {
        match style {
            1 => 3885,
            2 => 3939,
            3 => 3965,
            4 => 3988,
            5 => 4153,
            6 => 4174,
            7 => 4195,
            8 => 4216,
            9 => 4265,
            10 => 4267,
            11 => 4574,
            12 | 13 => 4712,
            14 => 5156,
            15 => 5177,
            16 => 5198,
            17 => 5556,
            18 => 5609,
            19 => 5697,
            20 => 5720,
            21 => 5745,
            22 => 5763,
            23 => 5784,
            24 => 5805,
            25 => 5826,
            26 => 5846,
            27 => 5865,
            28 => 5886,
            29 => 5905,
            30 => 5939,
            31 => 5962,
            32 => 5982,
            33 => 6005,
            34 => 6028,
            35 => 6051,
            36 => 6074,
            37 => 6118,
            _ => 3884,
        }
    } else {
        match style {
            1 | 2 => 306,
            3 | 4 => 328,
            5 => 343,
            6 => 348,
            7 => 625,
            8 => 626,
            9 => 627,
            10 => 680,
            11 => 681,
            12 => 831,
            13 => 838,
            14 => 914,
            15 => 952,
            16 => 1142,
            17 => 1298,
            18 | 23 => 1528,
            19 | 24 => 1529,
            20 | 25 => 1530,
            21 | 26 => 1531,
            22 | 27 => 1532,
            28 => 2230,
            29 => 2249,
            30 => 2250,
            31 => 2526,
            32 => 2544,
            33 => 2559,
            34 => 2574,
            35 | 36 => 2612,
            37 | 38 => 2613,
            39 | 40 => 2614,
            41 => 2615,
            42 => 2616,
            43 => 2617,
            44 => 2618,
            45 => 2619,
            46 => 2620,
            47 => 2748,
            48 => 2814,
            49 => 3180,
            50 => 3125,
            51 => 3181,
            _ => 48,
        }
    }
}

/// `WorldGen.GetChestIcon` (`WorldGen.cs:57989-58004`) + the
/// `Chest.Initialize` overrides (`Chest.cs:229-257`): the hover icon for
/// a chest style. The bounds are the icon-array lengths (52 / 38,
/// `Chest.cs:3675-3676`), NOT the drop table's style space; out-of-range
/// returns 0 (the C# `result` init). The overrides: base 2→327, 4→329,
/// 23..27→1533..1537, 36/38/40→327; second-type 13→4714.
pub(crate) fn chest_icon(style: i32, containers2: bool) -> u16 {
    if containers2 {
        if !(0..38).contains(&style) {
            return 0;
        }
        if style == 13 {
            return 4714; // `Chest.cs:251`
        }
        item_drop_chests(style, true)
    } else {
        if !(0..52).contains(&style) {
            return 0;
        }
        match style {
            2 | 36 | 38 | 40 => 327, // `Chest.cs:236/:243-245`
            4 => 329,                // `:237`
            23..=27 => (1533 + style - 23) as u16, // `:238-242`
            _ => item_drop_chests(style, false),
        }
    }
}

/// `Player.IsHoveringOverABottomSideOfABed` (`Player.cs:33327-33337`),
/// verbatim: `frameX / 72 == 1` flips the `frameX % 72 < 36` test — the
/// bed's right half mirrors which side is the "bottom" (the foot).
pub(crate) fn bed_bottom_side(frame_x: i16) -> bool {
    let fx = frame_x as i32;
    let mut flag2 = fx % 72 < 36;
    if fx / 72 == 1 {
        flag2 = !flag2;
    }
    flag2
}

/// `Player.IsWithinSnappngRangeToTile` (`Player.cs:17365-17372`),
/// verbatim: the tile centre `(x*16+8, y*16+8)` within
/// `distanceInPixels` of `(Center.X, Bottom.Y - 16)`. The port's
/// `Player::center`/`bottom` are the same `Entity` maths.
pub(crate) fn within_snapping_range(p: &crate::player::Player, x: i32, y: i32, dist_px: f32) -> bool {
    let (cx, _) = p.center();
    let (_, by) = p.bottom();
    let dx = (x * 16 + 8) as f32 - cx;
    let dy = (y * 16 + 8) as f32 - (by - 16.0);
    (dx * dx + dy * dy).sqrt() <= dist_px
}

/// The tile-hover arms 1-6 + 9-10 (§5 rows 1-6/9-10,
/// `Player.cs:33593-33682`/`:33691-33732`): the constant/style decode
/// arms that write `noThrow = 2` + `enabled = true` + an ID. Returns
/// `(id, arm_writes)` — `id == 0` means the arm fired `noThrow = 2` but
/// left enabled/ID untouched (the fake chests' icon-0 refusal,
/// `:33706`/`:33727`); `arm_writes == false` means the arm wrote NOTHING
/// (the bed's out-of-snap top side, `:33597`'s gate fails). `None` = the
/// kind has no arm here (the caller falls through).
///
/// * 79 bed (`:33593-33610`): the bottom side draws
///   `GetItemDrop_Beds(frameY/36)`; the top side draws the Sleeping Icon
///   5013 ONLY within the 96-px snap range.
/// * 597 pylon (`:33611-33658`): `GetPylonItemTypeFromTileStyle(frameX/54)`
///   then the style 1-10 overrides — the SAME ids the base switch returns
///   (`TETeleportationPylon.cs:115-131`), so the override chain is a no-op
///   kept for the citation; the port reuses `PylonType::item_id`
///   (`tr-core/pylons.rs:78`, style 0/out-of-range → 4876).
/// * 621 → 3750, 464 → 3814, 699 → 5482 (the constant arms).
/// * 33 candles (`:33677-33682`): `GetItemDrop_Candles(frameY/22)`.
/// * 441/468 fake chests (`:33691-33732`): `GetChestIcon(frameX/36,
///   kind == 468)`; icon 0 → enabled/ID stay (noThrow still 2). The C#'s
///   `num/num2` origin decode is dead for the icon (the style is the
///   hovered cell's own `frameX/36`).
pub(crate) fn hover_icon_simple(
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    p: &crate::player::Player,
    x: i32,
    y: i32,
) -> Option<(u16, bool)> {
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    match kind {
        79 => {
            if bed_bottom_side(frame_x) {
                Some((item_drop_beds(fy / 36), true))
            } else if within_snapping_range(p, x, y, 96.0) {
                Some((5013, true))
            } else {
                Some((0, false)) // the C# writes nothing (`:33597` fails)
            }
        }
        597 => {
            // `PylonType::from_tile_style` + `item_id` — the style IS the
            // type (`TETeleportationPylon.cs:175-178`); an out-of-range
            // style falls to SurfacePurity's 4876 (the `_ => 4876` arm).
            let id = tr_core::pylons::PylonType::from_tile_style(fx / 54)
                .unwrap_or(tr_core::pylons::PylonType::SurfacePurity)
                .item_id();
            Some((id, true))
        }
        621 => Some((3750, true)),
        464 => Some((3814, true)),
        699 => Some((5482, true)),
        33 => Some((item_drop_candles(fy / 22), true)),
        441 | 468 => {
            let icon = chest_icon(fx / 36, kind == 468);
            if icon != 0 {
                Some((icon, true))
            } else {
                // `:33706`/`:33727` — icon 0: noThrow = 2 fired at
                // `:33693`/`:33714`, but enabled/ID stay untouched.
                Some((0, true))
            }
        }
        _ => None,
    }
}

/// `Player.TileInteractionsMouseOver_Containers` (`Player.cs:34755-34813`)
/// — the chest hover TEXT arm (tiles 21/467). Returns the writes the
/// caller applies: `(id, text, enabled, hovered_chest)`. `id` is the
/// `-1` sentinel unless the style-name collapse finds an icon; `text` is
/// the chest's name (or the style default); `hovered_chest` is the
/// `Chest.FindChest` index (`Player.hoveredChestIndex`, `:34812` — the
/// port has no field yet, returned for the caller's future use).
///
/// The C# order: the 2x2 origin decode (`frameX%36`/`frameY%36`), the
/// `FindChest`, `ID = -1`, then: no chest → `text = array[0]`, enabled;
/// named chest → `text = name`, enabled; else in-range style →
/// `text = array[style]`, enabled. The collapse: when the text EQUALS
/// the style default, `GetChestIcon(style, kind == 467)` replaces it
/// (`text = ""`); `noThrow = 2`; icon > 0 → the 3988→306 remap
/// (`:34805-34807`, the Dead Man's Chest draws the plain Chest icon) +
/// `ID = icon`, enabled.
///
/// `array` = `Lang.chestType` (54 entries, 52.. aliases 0, `Lang.cs:499-506`)
/// for 21, `Lang.chestType2` (38) for 467. The port reads the live
/// localization through `chest::default_chest_name`'s chain (the
/// resource-less fallback names the style's placing item — the same
/// string the C# table carries).
pub(crate) fn container_hover(
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    x: i32,
    y: i32,
    chests: &[crate::chest::WorldChest],
) -> (i32, String, bool, Option<usize>) {
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    // `:34763-34772` — the 2x2 origin.
    let ox = if fx % 36 != 0 { x - 1 } else { x };
    let oy = if fy % 36 != 0 { y - 1 } else { y };
    let found = crate::chest::find_chest(chests, ox, oy);
    let style = fx / 36;
    // `:34775-34778` — the style-name range gate (the array lengths).
    let len = if kind == 467 { 38 } else { 54 };
    let in_range = (0..len).contains(&style);
    // The style default name (`Lang.chestType[style]` / `chestType2`):
    // the port's `default_chest_name` reads the live table + the
    // resource-less fallback; tile 21's 52.. alias folds to style 0
    // inside it (`Lang.cs:503-506`).
    let style_name = crate::chest::default_chest_name(kind, frame_x);
    let mut id: i32 = -1; // `:34774`
    let mut text = String::new();
    let mut enabled = false;
    match found {
        None => {
            // `:34779-34783` — array[0].
            text = crate::chest::default_chest_name(kind, 0);
            enabled = true;
        }
        Some(ci) => {
            let name = &chests[ci].name;
            if !name.is_empty() {
                // `:34786-34790` — the stored name.
                text = name.clone();
                enabled = true;
            } else if in_range {
                // `:34791-34795` — the style default.
                text = style_name.clone();
                enabled = true;
            }
            // `:34796-34800` — the collapse: text == the style default →
            // num5 = GetChestIcon + text = "" (EVEN when the icon is 0 —
            // the C# clears the text before the `num5 > 0` gate).
            let mut num5: i32 = 0;
            if in_range && text == style_name {
                num5 = i32::from(chest_icon(style, kind == 467));
                text.clear();
            }
            // `:34803-34811` — the 3988→306 remap + the ID write; the
            // icon-0 case keeps the -1 sentinel + the enabled the name
            // arms set (the C# `if (num5 > 0)` skips both writes).
            if num5 > 0 {
                id = if num5 == 3988 { 306 } else { num5 };
                enabled = true;
            }
        }
    }
    // `:34802` — noThrow = 2 (the caller writes it).
    (id, text, enabled, found)
}

// ---- Batch 4 (§5 arms 11-20): dresser / doors / clocks / constants --------

/// `WorldGen.GetItemDrop_Clocks` (`WorldGen.cs:40089-40167`), verbatim:
/// default 359; styles 1-5 → `2237 + style - 1`, 6 → 2560, 7 → 2575,
/// 8-23 → `2591 + style - 8`, then the per-style tail (24..65).
pub(crate) fn item_drop_clocks(style: i32) -> u16 {
    match style {
        1..=5 => (2237 + style - 1) as u16,
        6 => 2560,
        7 => 2575,
        8..=23 => (2591 + style - 8) as u16,
        24 => 2809,
        25 => 3126,
        26 => 3128,
        27 => 3127,
        28 => 3898,
        29 => 3899,
        30 => 3900,
        31 => 3901,
        32 => 3902,
        33 => 3940,
        34 => 3966,
        35 => 4154,
        36 => 4175,
        37 => 4196,
        38 => 4217,
        39 => 4306,
        40 => 4575,
        41 => 5157,
        42 => 5178,
        43 => 5199,
        44 => 5557,
        45 => 5610,
        46 => 5698,
        47 => 5721,
        48 => 5746,
        49 => 5764,
        50 => 5785,
        51 => 5806,
        52 => 5827,
        53 => 5847,
        54 => 5866,
        55 => 5887,
        56 => 5906,
        57 => 5940,
        58 => 5963,
        59 => 5983,
        60 => 6006,
        61 => 6029,
        62 => 6052,
        63 => 6075,
        64 => 6097,
        65 => 6119,
        _ => 359,
    }
}

/// `WorldGen.GetDoorItem` (`WorldGen.cs:69200-69407`), verbatim: the door
/// style's item. Default 0 (the C# `result` init — an unknown style drops
/// nothing); 0 → 25, 1/2/3 → 650/651/652, 4-8 → `812 + style`, 9 → 837,
/// 10 → 912, 12-15 → 1137..1140, 16/17/18 → 1411/1412/1413, 19 → 1458,
/// 20-23 → `1709 + style - 20`, then the per-style tail (24..68). Style
/// 11 has NO arm here (the hover arm overrides it to the Temple Key 1141,
/// `Player.cs:33792-33795`).
pub(crate) fn door_item(style: i32) -> u16 {
    match style {
        0 => 25,
        1 => 650,
        2 => 651,
        3 => 652,
        4..=8 => (812 + style) as u16,
        9 => 837,
        10 => 912,
        12 => 1137,
        13 => 1138,
        14 => 1139,
        15 => 1140,
        16 => 1411,
        17 => 1412,
        18 => 1413,
        19 => 1458,
        20..=23 => (1709 + style - 20) as u16,
        24 => 1793,
        25 => 1815,
        26 => 1924,
        27 => 2044,
        28 => 2265,
        29 => 2528,
        30 => 2561,
        31 => 2576,
        32 => 2815,
        33 => 3129,
        34 => 3131,
        35 => 3130,
        36 => 3888,
        37 => 3941,
        38 => 3967,
        39 => 4155,
        40 => 4176,
        41 => 4197,
        42 => 4218,
        43 => 4307,
        44 => 4415,
        45 => 4576,
        46 => 5158,
        47 => 5179,
        48 => 5200,
        49 => 5558,
        50 => 5611,
        51 => 5699,
        52 => 5722,
        53 => 5747,
        54 => 5765,
        55 => 5786,
        56 => 5807,
        57 => 5828,
        58 => 5867,
        59 => 5907,
        60 => 5941,
        61 => 5964,
        62 => 5984,
        63 => 6007,
        64 => 6030,
        65 => 6053,
        66 => 6076,
        67 => 6098,
        68 => 6120,
        _ => 0,
    }
}

/// `WorldGen.GetItemDrop_Dressers` (`WorldGen.cs:49855-50037`) = the
/// `Chest.dresserTypeToIcon` table (`Chest.cs:252-256` fills it from this
/// fn): default 334; styles 1-3 → `646 + style`, 4 → 918, 5-15 →
/// `2386 + style - 5`, then the per-style tail (16..64).
pub(crate) fn item_drop_dressers(style: i32) -> u16 {
    match style {
        1..=3 => (646 + style) as u16,
        4 => 918,
        5..=15 => (2386 + style - 5) as u16,
        16 => 2529,
        17 => 2545,
        18 => 2562,
        19 => 2577,
        20 => 2637,
        21 => 2638,
        22 => 2639,
        23 => 2640,
        24 => 2816,
        25 => 3132,
        26 => 3134,
        27 => 3133,
        28 => 3911,
        29 => 3912,
        30 => 3913,
        31 => 3914,
        32 => 3934,
        33 => 3968,
        34 => 4148,
        35 => 4169,
        36 => 4190,
        37 => 4211,
        38 => 4301,
        39 => 4569,
        40 => 5151,
        41 => 5172,
        42 => 5193,
        43 => 5551,
        44 => 5604,
        45 => 5692,
        46 => 5715,
        47 => 5741,
        48 => 5766,
        49 => 5787,
        50 => 5808,
        51 => 5829,
        52 => 5848,
        53 => 5868,
        54 => 5888,
        55 => 5908,
        56 => 5942,
        57 => 5965,
        58 => 5985,
        59 => 6008,
        60 => 6031,
        61 => 6054,
        62 => 6077,
        63 => 6099,
        64 => 6121,
        _ => 334,
    }
}

/// The dresser hover TEXT arm (§5 row 11, `Player.cs:33733-33771`): the
/// 3x2 origin decode (`frameX%54/18`, `frameY%36`), `FindChest`, the -1
/// sentinel; no chest → `Lang.dresserType[0]`; named → the name; else →
/// `dresserType[frameX/54]`; the collapse (text == the style default) →
/// `dresserTypeToIcon[style]` + text ""; `noThrow = 2` + enabled; then
/// `frameY > 0` → the icon 269 + text "" (`:33766-33770`, the lower half
/// draws the generic Dresser icon). Returns `(id, text)` — enabled is
/// always true (the caller writes it), noThrow always 2.
pub(crate) fn dresser_hover(
    frame_x: i16,
    frame_y: i16,
    x: i32,
    y: i32,
    chests: &[crate::chest::WorldChest],
) -> (i32, String) {
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    // `:33736-33741` — the 3x2 origin.
    let ox = x - fx % 54 / 18;
    let oy = if fy % 36 != 0 { y - 1 } else { y };
    let found = crate::chest::find_chest(chests, ox, oy);
    let style = fx / 54;
    let style_name = crate::chest::default_chest_name(88, frame_x);
    let mut id: i32 = -1; // `:33743`
    let mut text;
    match found {
        None => {
            // `:33744-33747` — dresserType[0].
            text = crate::chest::default_chest_name(88, 0);
        }
        Some(ci) => {
            let name = &chests[ci].name;
            if !name.is_empty() {
                // `:33750-33753` — the stored name.
                text = name.clone();
            } else {
                // `:33754-33757` — the style default.
                text = style_name.clone();
            }
            // `:33758-33762` — the collapse.
            if text == style_name {
                id = i32::from(item_drop_dressers(style));
                text.clear();
            }
        }
    }
    // `:33766-33770` — the lower half draws the generic Dresser 269.
    if fy > 0 {
        id = 269;
        text.clear();
    }
    (id, text)
}

/// The door hover arm (§5 row 12, `Player.cs:33772-33801`): the style
/// decode (`frameY/54` via the subtract-54 loop + `36 * (frameX/54)` for
/// tile 10 / `36 * (frameX/72)` for tile 11), `GetDoorItem(style)`, the
/// style-11 Temple Key 1141 override (`:33792-33795`), the `num9 != 0`
/// gate (`:33796` — icon 0 leaves enabled/ID, noThrow still 2). Returns
/// the icon id (0 = the refusal).
pub(crate) fn door_hover_icon(kind: u16, frame_x: i16, frame_y: i16) -> u16 {
    let fx = frame_x as i32;
    let mut fy = frame_y as i32;
    let mut style = 0;
    while fy >= 54 {
        fy -= 54;
        style += 1;
    }
    if kind == 10 {
        style += 36 * (fx / 54);
    }
    if kind == 11 {
        style += 36 * (fx / 72);
    }
    let mut id = door_item(style);
    if style == 11 {
        id = 1141; // `:33794` — the Temple Key
    }
    id
}

/// The tile-hover arms 13-20 (§5 rows 13-20,
/// `Player.cs:33802-33886`): the clocks 104 + the constant arms
/// 356/663/377/216/387-386/389-388/335. Returns the icon id; `None` = no
/// arm. The 209 Cannon arm (row 17) writes Reversed too — it rides
/// [`cannon_hover`] separately.
pub(crate) fn hover_icon_constants(kind: u16, frame_x: i16, frame_y: i16) -> Option<u16> {
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    match kind {
        104 => Some(item_drop_clocks(fx / 36)),
        356 => Some(3064),
        663 => Some(5381),
        377 => Some(3198),
        216 => {
            // `:33863-33870` — the subtract-40 loop counts the style row.
            let mut n = fy;
            let mut row = 0;
            while n >= 40 {
                n -= 40;
                row += 1;
            }
            Some((970 + row) as u16)
        }
        387 | 386 => Some(3239), // `:33879` (the GetTopLeftAndStyles call is dead)
        389 | 388 => Some(3240), // `:33885`
        335 => Some(2700),       // `:33891`
        _ => None,
    }
}

/// The Cannon hover arm (§5 row 17, tile 209,
/// `Player.cs:33826-33858`): the frameX-band ID decode (928/1337/3369/
/// 3664 — the cannonball variants) + the Reversed flag (`(frameX/18)
/// mod 4 < 2`, the verbatim subtract-4 loop). Returns `(id, reversed)`;
/// the ID stays 0 for frameX >= 360 (the C# writes no ID there — the band
/// ladder ends, but enabled/noThrow still fire).
pub(crate) fn cannon_hover(frame_x: i16) -> (u16, bool) {
    let fx = frame_x as i32;
    let id = if fx < 72 {
        928
    } else if fx < 144 {
        1337
    } else if fx < 216 {
        3369
    } else if fx < 360 {
        3664
    } else {
        0
    };
    // `:33846-33857` — the subtract-4 loop, then `< 2` → Reversed.
    let mut n = fx / 18;
    while n >= 4 {
        n -= 4;
    }
    (id, n < 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-g86 batch 1 — the demon altar's first 3 frame columns (group 0)
    /// draw the Demon Altar item 6135 (`Player.cs:34535`'s `num42 == 0`
    /// arm). WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_g86_the_demon_altar_columns_draw_6135() {
        for frame_x in [0i16, 18, 36] {
            assert_eq!(
                altar_hover_icon(26, frame_x),
                Some(6135),
                "frameX {frame_x}: column {} stays in group 0",
                frame_x / 18
            );
        }
    }

    /// W4-g86 batch 1 — every later column group draws the Crimson Altar
    /// 6136: the subtract-3 loop counts groups (`:34530-34534`). Column 3
    /// (frameX 54) loops once (3 > 2 → 0, group 1); column 6 (frameX 108)
    /// loops twice (6 → 3 → 0, group 2); a MID-group column 5 (frameX 90)
    /// loops once (5 → 2, group 1) — the verbatim loop, not `frameX / 54`,
    /// decides. WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_crimson_altar_columns_draw_6136() {
        for frame_x in [54i16, 72, 90, 108, 162] {
            assert_eq!(
                altar_hover_icon(26, frame_x),
                Some(6136),
                "frameX {frame_x}: column {} leaves group 0",
                frame_x / 18
            );
        }
    }

    /// W4-g86 batch 1 — the Lihzahrd altar 237 always draws the Power Cell
    /// 1293 (`Player.cs:34313`, no frame decode), and a non-altar kind has
    /// no arm (`None` — the caller's match default writes nothing).
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_lihzahrd_altar_draws_the_power_cell() {
        assert_eq!(altar_hover_icon(237, 0), Some(1293));
        assert_eq!(altar_hover_icon(237, 18), Some(1293));
        // The workbench 18 rides its own arm (`:34518-34523`), not this one.
        assert_eq!(altar_hover_icon(18, 0), None);
        assert_eq!(altar_hover_icon(695, 0), None);
    }

    fn slot(kind: u16, count: u32) -> crate::item::ItemSlot {
        crate::item::ItemSlot { kind, name: "", count, prefix: 0 }
    }

    /// W4-g86 batch 2 — `Fishing_GetBait` (`Player.cs:42254-42280`): the
    /// ammo row wins over the grid, the first live bait slot wins inside
    /// a row, an empty stack is skipped, and no bait returns (0, 0).
    /// WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_g86_the_bait_scan_prefers_the_ammo_row() {
        let grid = [crate::item::EMPTY_SLOT; 50];
        let mut ammo = [crate::item::EMPTY_SLOT; 4];
        // Grid-only: the first live bait (2674 Apprentice, bait 15).
        let mut g = grid;
        g[7] = slot(2674, 3);
        g[9] = slot(2675, 1);
        assert_eq!(fishing_get_bait(&g, &ammo), (15, 2674));
        // Ammo row beats the grid even with a weaker bait.
        ammo[2] = slot(2674, 1);
        g[7] = slot(2673, 1); // Truffle Worm, bait 666
        assert_eq!(fishing_get_bait(&g, &ammo), (15, 2674));
        // An empty ammo stack is skipped (`stack > 0`).
        ammo[2] = slot(2674, 0);
        assert_eq!(fishing_get_bait(&g, &ammo), (666, 2673));
        // No bait anywhere.
        let g2 = grid;
        let a2 = [crate::item::EMPTY_SLOT; 4];
        assert_eq!(fishing_get_bait(&g2, &a2), (0, 0));
    }

    /// W4-g86 batch 2 — the pole arm (`Player.cs:42615-42624`): a held
    /// Golden Fishing Rod 2294 (fishingPole 50) draws the bait with push
    /// 6; no bait → no icon; an already-enabled icon blocks the arm
    /// (`!cursorItemIconEnabled`, the hover pass wins). WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_pole_draws_the_bait_push_6() {
        let grid = [crate::item::EMPTY_SLOT; 50];
        let coins = [crate::item::EMPTY_SLOT; 4];
        let mut ammo = [crate::item::EMPTY_SLOT; 4];
        ammo[0] = slot(2675, 5); // Journeyman Bait, bait 30
        let pole = slot(2294, 1);
        assert_eq!(
            item_check_head_icon(pole, &grid, &coins, &ammo, false),
            Some((2675, 6))
        );
        // The hover icon already armed: the head arm refuses.
        assert_eq!(item_check_head_icon(pole, &grid, &coins, &ammo, true), None);
        // No bait: the arm ends (a pole is not 779/5134).
        let empty_ammo = [crate::item::EMPTY_SLOT; 4];
        assert_eq!(item_check_head_icon(pole, &grid, &coins, &empty_ammo, false), None);
        // An empty held stack never arms (`item.stack > 0`).
        assert_eq!(item_check_head_icon(slot(2294, 0), &grid, &coins, &ammo, false), None);
    }

    /// W4-g86 batch 2 — the solution arm (`Player.cs:42625-42650`): the
    /// Clentaminator 779 (useAmmo 780) draws the first ammo-780 solution
    /// with push 10 — the ammo row first, then the grid, then the COIN
    /// rows (`j < 54`); a non-matching ammo (Wooden Arrow 40, `ammo` 40 ≠
    /// 780) is skipped. The Terraformer 5134 rides the same arm.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_clentaminator_draws_the_solution_push_10() {
        let mut grid = [crate::item::EMPTY_SLOT; 50];
        let mut coins = [crate::item::EMPTY_SLOT; 4];
        let mut ammo = [crate::item::EMPTY_SLOT; 4];
        let clent = slot(779, 1);
        // Grid-only Green Solution 780 (ammo 780).
        grid[3] = slot(780, 99);
        assert_eq!(
            item_check_head_icon(clent, &grid, &coins, &ammo, false),
            Some((780, 10))
        );
        // The ammo row beats the grid.
        ammo[1] = slot(780, 5);
        grid[3] = crate::item::EMPTY_SLOT;
        assert_eq!(
            item_check_head_icon(clent, &grid, &coins, &ammo, false),
            Some((780, 10))
        );
        // The coin rows are scanned (`j < 54`) when grid+ammo are empty.
        let empty_ammo = [crate::item::EMPTY_SLOT; 4];
        coins[2] = slot(780, 1);
        assert_eq!(
            item_check_head_icon(clent, &grid, &coins, &empty_ammo, false),
            Some((780, 10))
        );
        // A non-matching ammo (arrow 40) never fires; the Terraformer
        // 5134 shares the arm.
        let coins2 = [crate::item::EMPTY_SLOT; 4];
        let mut g2 = [crate::item::EMPTY_SLOT; 50];
        g2[0] = slot(40, 99);
        assert_eq!(item_check_head_icon(clent, &g2, &coins2, &empty_ammo, false), None);
        g2[0] = slot(780, 1);
        assert_eq!(
            item_check_head_icon(slot(5134, 1), &g2, &coins2, &empty_ammo, false),
            Some((780, 10))
        );
        // A plain held item (Iron Pickaxe 1) has no head arm.
        assert_eq!(item_check_head_icon(slot(1, 1), &g2, &coins2, &empty_ammo, false), None);
    }

    // ---- Batch 3 (§5 arms 1-10) -------------------------------------------
    // WRITTEN NOT RUN (code-only lane). Zero RNG — every arm is integer/
    // string maths.

    /// `GetItemDrop_Beds` (`WorldGen.cs:39905-40087`): the range arms +
    /// the per-style tail + the out-of-range 224 default.
    #[test]
    fn w4_g86_the_bed_drop_table_matches_the_csharp() {
        assert_eq!(item_drop_beds(0), 224);
        assert_eq!(item_drop_beds(1), 644); // 1 + 643
        assert_eq!(item_drop_beds(3), 646);
        assert_eq!(item_drop_beds(4), 920);
        assert_eq!(item_drop_beds(5), 1470); // 1465 + 5
        assert_eq!(item_drop_beds(8), 1473);
        assert_eq!(item_drop_beds(9), 1719); // 1710 + 9
        assert_eq!(item_drop_beds(12), 1722);
        assert_eq!(item_drop_beds(13), 2066); // 2066 + 13 - 13
        assert_eq!(item_drop_beds(18), 2071);
        assert_eq!(item_drop_beds(19), 2139);
        assert_eq!(item_drop_beds(27), 2811);
        assert_eq!(item_drop_beds(40), 5149);
        assert_eq!(item_drop_beds(64), 6112);
        assert_eq!(item_drop_beds(65), 224, "out of range keeps the default");
        assert_eq!(item_drop_beds(-1), 224);
    }

    /// `GetItemDrop_Candles` (`WorldGen.cs:39724-39903`): the range arms +
    /// the tail + the 105 default.
    #[test]
    fn w4_g86_the_candle_drop_table_matches_the_csharp() {
        assert_eq!(item_drop_candles(0), 105);
        assert_eq!(item_drop_candles(1), 1405);
        assert_eq!(item_drop_candles(3), 1407);
        assert_eq!(item_drop_candles(4), 2045); // 2045 + 4 - 4
        assert_eq!(item_drop_candles(13), 2054);
        assert_eq!(item_drop_candles(14), 2153); // 2153 + 14 - 14
        assert_eq!(item_drop_candles(16), 2155);
        assert_eq!(item_drop_candles(17), 2236);
        assert_eq!(item_drop_candles(29), 3172); // the 27/28/29 swap
        assert_eq!(item_drop_candles(63), 6115);
        assert_eq!(item_drop_candles(64), 105, "out of range keeps the default");
    }

    /// `GetItemDrop_Chests` (`WorldGen.cs:41554-41839`) + `GetChestIcon`
    /// (`:57989-58004`) + the `Chest.Initialize` overrides
    /// (`Chest.cs:236-251`): the base/second families, the bounds (52/38)
    /// and the override rows.
    #[test]
    fn w4_g86_the_chest_icon_table_and_overrides() {
        // The base family (tile 21).
        assert_eq!(chest_icon(0, false), 48);
        assert_eq!(chest_icon(1, false), 306);
        assert_eq!(chest_icon(2, false), 327, "the :236 override (306 → 327)");
        assert_eq!(chest_icon(4, false), 329, "the :237 override (328 → 329)");
        assert_eq!(chest_icon(23, false), 1533, "the :238 override");
        assert_eq!(chest_icon(27, false), 1537, "the :242 override");
        assert_eq!(chest_icon(36, false), 327, "the :243 override (2612 → 327)");
        assert_eq!(chest_icon(38, false), 327);
        assert_eq!(chest_icon(40, false), 327);
        assert_eq!(chest_icon(51, false), 3181);
        assert_eq!(chest_icon(52, false), 0, "the 52 bound (:57999)");
        assert_eq!(chest_icon(-1, false), 0);
        // The second family (tile 467).
        assert_eq!(chest_icon(0, true), 3884);
        assert_eq!(chest_icon(4, true), 3988);
        assert_eq!(chest_icon(13, true), 4714, "the :251 override (4712 → 4714)");
        assert_eq!(chest_icon(12, true), 4712);
        assert_eq!(chest_icon(37, true), 6118);
        assert_eq!(chest_icon(38, true), 0, "the 38 bound (:57994)");
    }

    /// `IsHoveringOverABottomSideOfABed` (`Player.cs:33327-33337`): the
    /// `frameX/72 == 1` flip. Left half (0..71): <36 is the bottom; right
    /// half (72..143): the test INVERTS (>=36 within the half is bottom).
    #[test]
    fn w4_g86_the_bed_bottom_side_flips_on_the_right_half() {
        assert!(bed_bottom_side(0)); // 0 % 72 = 0 < 36, no flip
        assert!(bed_bottom_side(18));
        assert!(!bed_bottom_side(36)); // 36 !< 36
        assert!(!bed_bottom_side(54));
        assert!(!bed_bottom_side(72)); // 72 % 72 = 0 < 36, FLIPPED → false
        assert!(!bed_bottom_side(90));
        assert!(bed_bottom_side(108)); // 108 % 72 = 36 !< 36, FLIPPED → true
        assert!(bed_bottom_side(126));
    }

    /// The bed hover arm (`Player.cs:33593-33610`): the bottom side draws
    /// the style's bed item; the top side draws the Sleeping Icon 5013
    /// ONLY within the 96-px snap; out of snap writes NOTHING.
    #[test]
    fn w4_g86_the_bed_hover_arm_splits_bottom_top_and_snap() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut p = crate::player::Player::spawn(&app.world);
        // Park the player so tile (10, 10)'s centre is within 96 px of
        // (Center.X, Bottom.Y - 16): stand the player ON the tile.
        p.x = 10.0 * 16.0;
        p.y = 10.0 * 16.0 - p.height;
        app.player = Some(p);
        let p = app.player.as_ref().unwrap();
        // Bottom side (frameX 0), style 1 (frameY 36) → 644.
        assert_eq!(
            hover_icon_simple(79, 0, 36, p, 10, 10),
            Some((644, true))
        );
        // Top side (frameX 36), in snap → 5013.
        assert_eq!(hover_icon_simple(79, 36, 0, p, 10, 10), Some((5013, true)));
        // Top side, OUT of snap (a far tile) → nothing written.
        assert_eq!(hover_icon_simple(79, 36, 0, p, 60, 60), Some((0, false)));
    }

    /// The pylon hover arm (`Player.cs:33611-33658`): the style IS the
    /// type (`frameX/54`); the override chain returns the SAME ids the
    /// base switch does, so `PylonType::item_id` is the whole decode.
    #[test]
    fn w4_g86_the_pylon_hover_arm_decodes_the_style() {
        let app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let p = crate::player::Player::spawn(&app.world);
        // Style 0 (frameX 0) → SurfacePurity 4876.
        assert_eq!(hover_icon_simple(597, 0, 0, &p, 0, 0), Some((4876, true)));
        // Style 1 (frameX 54) → Jungle 4875 (the :33620 override).
        assert_eq!(hover_icon_simple(597, 54, 0, &p, 0, 0), Some((4875, true)));
        // Style 10 (frameX 540) → Shimmer 5653 (the :33656 override).
        assert_eq!(hover_icon_simple(597, 540, 0, &p, 0, 0), Some((5653, true)));
        // Out-of-range style → the `_ => 4876` default.
        assert_eq!(hover_icon_simple(597, 594, 0, &p, 0, 0), Some((4876, true)));
    }

    /// The constant + candle arms (`Player.cs:33659-33682`) + the
    /// fake-chest icon arms (`:33691-33732`).
    #[test]
    fn w4_g86_the_constant_candle_and_fake_chest_arms() {
        let app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let p = crate::player::Player::spawn(&app.world);
        assert_eq!(hover_icon_simple(621, 0, 0, &p, 0, 0), Some((3750, true)));
        assert_eq!(hover_icon_simple(464, 0, 0, &p, 0, 0), Some((3814, true)));
        assert_eq!(hover_icon_simple(699, 0, 0, &p, 0, 0), Some((5482, true)));
        // Candles: frameY 22 → style 1 → 1405.
        assert_eq!(hover_icon_simple(33, 0, 22, &p, 0, 0), Some((1405, true)));
        // Fake chest 441: frameX 36 → style 1 → 306.
        assert_eq!(hover_icon_simple(441, 36, 0, &p, 0, 0), Some((306, true)));
        // Fake chest 468: frameX 144 → style 4 → 3988.
        assert_eq!(hover_icon_simple(468, 144, 0, &p, 0, 0), Some((3988, true)));
        // Fake chest out-of-range style → icon 0 → noThrow fires, ID stays.
        assert_eq!(hover_icon_simple(441, 1872, 0, &p, 0, 0), Some((0, true)));
        // A kind with no arm here.
        assert_eq!(hover_icon_simple(1, 0, 0, &p, 0, 0), None);
    }

    /// `TileInteractionsMouseOver_Containers` (`Player.cs:34755-34813`):
    /// the no-chest default text, the named-chest text, the style-name
    /// collapse → the icon (the 3988→306 remap), the -1 sentinel. The
    /// lang lock + English init pin the style-name strings (the
    /// `default_chest_name_matches_the_chestui_chain` precedent).
    #[test]
    fn w4_g86_the_container_hover_text_and_icon_collapse() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        // No chest at the origin → array[0] text, enabled, ID -1.
        let (id, text, enabled, found) = container_hover(21, 0, 0, 10, 10, &[]);
        assert_eq!(id, -1);
        assert_eq!(text, "Chest", "the style-0 default name");
        assert!(enabled);
        assert_eq!(found, None);
        // An UNNAMED chest at the origin, style 1 (frameX 36): the text
        // collapses to the Gold Chest icon 306 (the style default == the
        // text), ID 306, text "".
        let chests = vec![crate::chest::WorldChest {
            tile: (10, 10),
            ..Default::default()
        }];
        let (id, text, enabled, found) = container_hover(21, 36, 0, 10, 10, &chests);
        assert_eq!(id, 306, "the style-1 icon collapse");
        assert!(text.is_empty(), "the collapse clears the text");
        assert!(enabled);
        assert_eq!(found, Some(0));
        // A NAMED chest keeps its name (no collapse — the name differs
        // from the style default), ID stays -1.
        let named = vec![crate::chest::WorldChest {
            tile: (10, 10),
            name: "My Stash".into(),
            ..Default::default()
        }];
        let (id, text, enabled, _) = container_hover(21, 36, 0, 10, 10, &named);
        assert_eq!(id, -1);
        assert_eq!(text, "My Stash");
        assert!(enabled);
        // The 3988→306 remap: tile 467 style 4 (frameX 144) unnamed →
        // the Dead Man's Chest icon 3988 remaps to the plain Chest 306.
        let dead = vec![crate::chest::WorldChest {
            tile: (10, 10),
            ..Default::default()
        }];
        let (id, text, _, _) = container_hover(467, 144, 0, 10, 10, &dead);
        assert_eq!(id, 306, "the :34805-34807 remap");
        assert!(text.is_empty());
        // The 2x2 origin decode: a hovered right-half cell (frameX 18)
        // resolves the chest at x-1.
        let off = vec![crate::chest::WorldChest {
            tile: (9, 10),
            ..Default::default()
        }];
        let (_, _, _, found) = container_hover(21, 18, 0, 10, 10, &off);
        assert_eq!(found, Some(0), "frameX%36 != 0 → x-1");
    }

    // ---- Batch 4 (§5 arms 11-20) ------------------------------------------
    // WRITTEN NOT RUN (code-only lane). Zero RNG.

    /// `GetItemDrop_Clocks` (`WorldGen.cs:40089-40167`): the range arms +
    /// the tail + the 359 default.
    #[test]
    fn w4_g86_the_clock_drop_table_matches_the_csharp() {
        assert_eq!(item_drop_clocks(0), 359);
        assert_eq!(item_drop_clocks(1), 2237); // 2237 + 1 - 1
        assert_eq!(item_drop_clocks(5), 2241);
        assert_eq!(item_drop_clocks(6), 2560);
        assert_eq!(item_drop_clocks(7), 2575);
        assert_eq!(item_drop_clocks(8), 2591); // 2591 + 8 - 8
        assert_eq!(item_drop_clocks(23), 2606);
        assert_eq!(item_drop_clocks(24), 2809);
        assert_eq!(item_drop_clocks(27), 3127); // the 25/26/27 swap
        assert_eq!(item_drop_clocks(65), 6119);
        assert_eq!(item_drop_clocks(66), 359, "out of range keeps the default");
    }

    /// `GetDoorItem` (`WorldGen.cs:69200-69407`): the range arms + the
    /// tail + the 0 default; style 11 has NO arm (the hover override).
    #[test]
    fn w4_g86_the_door_item_table_matches_the_csharp() {
        assert_eq!(door_item(0), 25);
        assert_eq!(door_item(1), 650);
        assert_eq!(door_item(3), 652);
        assert_eq!(door_item(4), 816); // 812 + 4
        assert_eq!(door_item(8), 820);
        assert_eq!(door_item(9), 837);
        assert_eq!(door_item(10), 912);
        assert_eq!(door_item(11), 0, "style 11 has no GetDoorItem arm");
        assert_eq!(door_item(12), 1137);
        assert_eq!(door_item(15), 1140);
        assert_eq!(door_item(19), 1458);
        assert_eq!(door_item(20), 1709); // 1709 + 20 - 20
        assert_eq!(door_item(23), 1712);
        assert_eq!(door_item(35), 3130); // the 33/34/35 swap
        assert_eq!(door_item(68), 6120);
        assert_eq!(door_item(69), 0, "out of range keeps the default");
    }

    /// `GetItemDrop_Dressers` (`WorldGen.cs:49855-50037`): the range arms
    /// + the tail + the 334 default.
    #[test]
    fn w4_g86_the_dresser_drop_table_matches_the_csharp() {
        assert_eq!(item_drop_dressers(0), 334);
        assert_eq!(item_drop_dressers(1), 647); // 646 + 1
        assert_eq!(item_drop_dressers(3), 649);
        assert_eq!(item_drop_dressers(4), 918);
        assert_eq!(item_drop_dressers(5), 2386); // 2386 + 5 - 5
        assert_eq!(item_drop_dressers(15), 2396);
        assert_eq!(item_drop_dressers(16), 2529);
        assert_eq!(item_drop_dressers(27), 3133); // the 25/26/27 swap
        assert_eq!(item_drop_dressers(64), 6121);
        assert_eq!(item_drop_dressers(65), 334, "out of range keeps the default");
    }

    /// The door hover arm (`Player.cs:33772-33801`): the style decode
    /// (the subtract-54 loop + the frameX column), the style-11 Temple
    /// Key override, the icon-0 refusal.
    #[test]
    fn w4_g86_the_door_hover_decodes_the_style_and_temple_key() {
        // Tile 10, frameY 0, frameX 0 → style 0 → 25.
        assert_eq!(door_hover_icon(10, 0, 0), 25);
        // Tile 10, frameY 54 → style 1 → 650.
        assert_eq!(door_hover_icon(10, 0, 54), 650);
        // Tile 10, frameX 54 (column 1) → style 36 → 3888.
        assert_eq!(door_hover_icon(10, 54, 0), 3888);
        // Tile 11 (the tall door), frameX 72 → column 1 → style 36.
        assert_eq!(door_hover_icon(11, 72, 0), 3888);
        // Style 11: tile 10 frameY 594 (11 * 54) → the Temple Key 1141.
        assert_eq!(door_hover_icon(10, 0, 594), 1141);
        // An unknown style → 0 (the refusal).
        assert_eq!(door_hover_icon(10, 0, 3726), 0); // style 69
    }

    /// The constant hover arms (`Player.cs:33802-33892`): clocks + the
    /// fixed ids + the 216 subtract-40 row.
    #[test]
    fn w4_g86_the_constant_hover_arms() {
        // Clocks: frameX 36 → style 1 → 2237.
        assert_eq!(hover_icon_constants(104, 36, 0), Some(2237));
        assert_eq!(hover_icon_constants(356, 0, 0), Some(3064));
        assert_eq!(hover_icon_constants(663, 0, 0), Some(5381));
        assert_eq!(hover_icon_constants(377, 0, 0), Some(3198));
        // Firework 216: frameY 40 → row 1 → 971.
        assert_eq!(hover_icon_constants(216, 0, 40), Some(971));
        assert_eq!(hover_icon_constants(216, 0, 0), Some(970));
        assert_eq!(hover_icon_constants(387, 0, 0), Some(3239));
        assert_eq!(hover_icon_constants(386, 0, 0), Some(3239));
        assert_eq!(hover_icon_constants(389, 0, 0), Some(3240));
        assert_eq!(hover_icon_constants(388, 0, 0), Some(3240));
        assert_eq!(hover_icon_constants(335, 0, 0), Some(2700));
        assert_eq!(hover_icon_constants(1, 0, 0), None);
    }

    /// The Cannon hover arm (`Player.cs:33826-33858`): the frameX-band
    /// ID + the Reversed subtract-4 loop.
    #[test]
    fn w4_g86_the_cannon_hover_bands_and_reversed() {
        // frameX 0 → band 928, (0/18)%4 = 0 < 2 → Reversed.
        assert_eq!(cannon_hover(0), (928, true));
        // frameX 36 → 928, (36/18)%4 = 2 !< 2 → not Reversed.
        assert_eq!(cannon_hover(36), (928, false));
        // frameX 72 → band 1337, (72/18)%4 = 0 → Reversed.
        assert_eq!(cannon_hover(72), (1337, true));
        // frameX 144 → band 3369.
        assert_eq!(cannon_hover(144), (3369, true));
        // frameX 216 → band 3664.
        assert_eq!(cannon_hover(216), (3664, true));
        // frameX 360 → the ladder ends, ID 0 (enabled/noThrow still fire).
        assert_eq!(cannon_hover(360), (0, true));
    }

    /// The dresser hover TEXT arm (`Player.cs:33733-33771`): the no-chest
    /// default, the named chest, the style-name collapse → the icon, the
    /// lower-half 269. The lang lock + English init pin the style names.
    #[test]
    fn w4_g86_the_dresser_hover_text_and_icon_collapse() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        // No chest → dresserType[0] ("Dresser"), ID -1.
        let (id, text) = dresser_hover(0, 0, 10, 10, &[]);
        assert_eq!(id, -1);
        assert_eq!(text, "Dresser");
        // An unnamed chest, style 1 (frameX 54): the collapse → the
        // Ebonwood Dresser icon 647, text "".
        let chests = vec![crate::chest::WorldChest {
            tile: (10, 10),
            ..Default::default()
        }];
        let (id, text) = dresser_hover(54, 0, 10, 10, &chests);
        assert_eq!(id, 647, "the style-1 dresserTypeToIcon");
        assert!(text.is_empty());
        // A named chest keeps its name, ID -1.
        let named = vec![crate::chest::WorldChest {
            tile: (10, 10),
            name: "My Dresser".into(),
            ..Default::default()
        }];
        let (id, text) = dresser_hover(54, 0, 10, 10, &named);
        assert_eq!(id, -1);
        assert_eq!(text, "My Dresser");
        // The lower half (frameY > 0) → the generic Dresser 269, text "".
        let (id, text) = dresser_hover(54, 18, 10, 10, &chests);
        assert_eq!(id, 269, ":33768 the lower-half icon");
        assert!(text.is_empty());
        // The 3x2 origin: a hovered right-column cell (frameX 18) resolves
        // the chest at x-1.
        let off = vec![crate::chest::WorldChest {
            tile: (9, 10),
            ..Default::default()
        }];
        let (id, _) = dresser_hover(18, 0, 10, 10, &off);
        assert_eq!(id, 334, "the style-0 icon via the x-1 origin");
    }
}
