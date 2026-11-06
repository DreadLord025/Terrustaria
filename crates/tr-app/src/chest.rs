//! The world-chest entity, its store operations and its open/close +
//! locking interaction (M3-CHEST-UI phases 1-2): `Main.chest`'s entity, the
//! statics over it and the `Player.cs` chest branch, ported from
//! `decompiled/Terraria` (1.4.5.8).
//!
//! Shape: the C# store operations are statics over `Main.chest`
//! (`Main.cs:66994`, `new Chest[8000]`) — the port keeps that shape with
//! free functions over the app's `App.world_chests: Vec<WorldChest>` (the
//! field's frozen `Vec` + element shape come from the existing
//! `world_sim/editing.rs` break-refusal closure and its stocked-chest test,
//! which pin `craft::WorldChest { tile, slots }`; `craft.rs` re-exports the
//! moved type). The slot encoding is indexed with implicit air:
//! `slots[i]` is chest slot `i`, airs beyond `slots.len()` are implicit
//! (`(0, 0)`), and there are no trailing airs. Entries are created lazily on
//! the first open ([`try_open_chest_at_cursor`]); placement-time creation is
//! unreachable without `editing.rs` edits (deviation recorded for
//! `docs/ANALYSIS.md`).
//!
//! Out of this phase: the panel (phase 3), slot clicks (phase 4), rename +
//! the `Main.editChest` flush arm of the open path (phase 5), contents-drop
//! on break, `.wld` persistence + `Resize`/per-chest `maxItems` (M6-SAVE-2),
//! banks (M3-STORE-1), worldgen loot (M5-CHEST-gen).

use glam::Vec2;

use crate::app::App;
use crate::item::{EMPTY_SLOT, HOTBAR, INV, ItemSlot};
use crate::player;
use crate::registry;
use tr_core::{TILE_PX, World};

/// One chest slot: `(item kind, stack)` — the C# `Chest.item[]` entry minus
/// the prefix (the port's inventory slots carry no prefix yet; the save
/// format's per-slot prefix byte, `WorldFile.cs:1671`, lands with M6-SAVE-2,
/// which owns the chest slot type's widening). The 2-tuple shape is pinned by
/// the existing `editing.rs` stocked-chest test literal (`slots: vec![(9, 6)]`).
pub(crate) type ChestSlot = (u16, u32);

/// `Main.maxChests` (`Main.cs:1033`), the array size (`Main.cs:66994`) and the
/// scan bound baked into `Chest.FindEmptyChest` (`Chest.cs:524`).
pub(crate) const MAX_CHESTS: usize = 8000;

/// `Chest.DefaultMaxItems` (`Chest.cs:64`). Every chest carries the default
/// until M6-SAVE-2 ports `Resize` (`Chest.cs:118-131`) — the per-chest field
/// is unreadable on the frozen `WorldChest` literal, and with `Resize` out no
/// chest ever deviates from 40, so the constant read is observably identical.
pub(crate) const DEFAULT_MAX_ITEMS: u32 = 40;

/// One world chest's item store (`Main.chest[]`'s `Chest.item`). The port's
/// chest UI/storage owns filling it (`M3-CHEST-UI`); the world save skips the
/// chest section (`M6-SAVE-2`), so this is in-memory only. The C# entity's
/// `x`/`y`/`item` fields (`Chest.cs:74-78`); `index` is the `Vec` position,
/// `name`/`maxItems`/`frame` land with their phases (5 / M6-SAVE-2 / render
/// follow-up).
#[derive(Clone, Debug, Default)]
pub(crate) struct WorldChest {
    /// The chest tile's top-left `(x, y)`.
    pub(crate) tile: (i32, i32),
    /// `(item id, stack)` stacks, at most [`DEFAULT_MAX_ITEMS`] real ones;
    /// indexed with implicit air — see the module doc.
    pub(crate) slots: Vec<ChestSlot>,
}

/// The removed-slot marker: `Main.chest[i] == null`'s stand-in inside the
/// frozen flat `Vec<WorldChest>` (`Chest.RemoveChest`, `Chest.cs:132-141`,
/// nulls the array slot). Unreachable as a world coordinate, so no live
/// lookup can match it.
const REMOVED_TILE: (i32, i32) = (i32::MIN, i32::MIN);

/// Whether `chest` is a removed (null) slot.
fn is_removed(chest: &WorldChest) -> bool {
    chest.tile == REMOVED_TILE
}

impl WorldChest {
    /// `Chest.IsEmpty` (`Chest.cs:166-176`): every slot within `maxItems` is
    /// air (`Item.IsAir`: type 0 and stack 0). The port's implicit airs
    /// beyond `slots.len()` are air by construction, so the scan covers the
    /// whole chest.
    pub(crate) fn is_empty(&self) -> bool {
        self.slots
            .iter()
            .all(|&(kind, stack)| kind == 0 && stack == 0)
    }
}

/// `Chest.FindChest` (`Chest.cs:513-520`): the store index at the coords, or
/// `None` for the C# `-1`. The C#'s O(1) `_chestsByCoords` dict becomes a
/// linear scan (code-shape-free; at most [`MAX_CHESTS`] entries on a cold
/// path).
pub(crate) fn find_chest(chests: &[WorldChest], x: i32, y: i32) -> Option<usize> {
    chests
        .iter()
        .position(|c| !is_removed(c) && c.tile == (x, y))
}

/// `Chest.FindEmptyChest` (`Chest.cs:522-541`): the first null slot index, or
/// `None` for the C# `-1` — either because a chest already exists at the
/// coords (`Chest.cs:529-533`, the collision rule) or because the store is
/// full ([`MAX_CHESTS`]). Removed slots are null and are reused first.
pub(crate) fn find_empty_chest(chests: &[WorldChest], x: i32, y: i32) -> Option<usize> {
    if find_chest(chests, x, y).is_some() {
        return None;
    }
    match chests.iter().position(is_removed) {
        Some(i) => Some(i),
        None if chests.len() < MAX_CHESTS => Some(chests.len()),
        None => None,
    }
}

/// `Chest.CreateChest` (`Chest.cs:590-604`) + `CreateWorldChest`/`Assign`
/// (`Chest.cs:104-119`): claim the first empty slot for the coords and store
/// the fresh (empty) chest. The C# `-1` failure (coords taken, store full —
/// `Chest.cs:595-597`) is caller-visible through [`find_empty_chest`]'s
/// `None`; this is the second step of the find-first sequence the open path
/// uses (`find` → `Some` → open, `None` → create), so its failure paths are
/// caller-contract violations and panic with the invariant (the house
/// `expect` pattern, `craft.rs:443`).
pub(crate) fn create_chest(chests: &mut Vec<WorldChest>, x: i32, y: i32) -> usize {
    let idx = find_empty_chest(chests, x, y).expect(
        "create_chest follows a find-first miss; a taken coord or full store \
         is the C# -1 path (Chest.cs:595-597)",
    );
    // `CreateWorldChest` fills with empty instances (`Chest.cs:107-109`) —
    // the port's empty slot list, air implied beyond the length.
    let fresh = WorldChest {
        tile: (x, y),
        slots: Vec::new(),
    };
    if idx == chests.len() {
        chests.push(fresh);
    } else {
        chests[idx] = fresh;
    }
    idx
}

/// `Chest.RemoveChest` (`Chest.cs:132-141`): the slot becomes null. Index
/// stable — the C# array slot is nulled, not compacted, so held indices
/// (`Player.chest`) stay valid.
pub(crate) fn remove_chest(chests: &mut Vec<WorldChest>, idx: usize) {
    if let Some(slot) = chests.get_mut(idx) {
        if !is_removed(slot) {
            *slot = WorldChest {
                tile: REMOVED_TILE,
                slots: Vec::new(),
            };
        }
    }
}

/// The chest's slots at `idx` — the plan contract's
/// `get(idx) -> Option<&[ChestSlot]>` over `Chest.item[]`
/// (`Chest.cs:74-78`). `None` for an out-of-range index; a removed slot
/// reads as absent.
pub(crate) fn chest_slots(chests: &[WorldChest], idx: usize) -> Option<&[ChestSlot]> {
    chests
        .get(idx)
        .filter(|c| !is_removed(c))
        .map(|c| c.slots.as_slice())
}

/// Mutable [`chest_slots`] — the plan contract's `get_mut`
/// (`Chest.cs:74-78`).
pub(crate) fn chest_slots_mut(chests: &mut [WorldChest], idx: usize) -> Option<&mut [ChestSlot]> {
    chests
        .get_mut(idx)
        .filter(|c| !is_removed(c))
        .map(|c| c.slots.as_mut_slice())
}

/// `Chest.IsLocked` (`Chest.cs:301-313`): the locked-frame decode. A missing
/// tile reads locked (the C# null-tile branch, `Chest.cs:303-305`); tile 21
/// (`TileID.Containers`) is locked in the six inclusive `frameX` ranges; tile
/// 467 (`TileID.Containers2`) iff `frameX / 36 == 13` (style 13). The decode
/// reads `type`/`frameX` without an activity check, as in the C#.
pub(crate) fn is_locked(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    let fx = t.frame_x;
    if t.kind == 21 {
        return (72..=106).contains(&fx)
            || (144..=178).contains(&fx)
            || (828..=1006).contains(&fx)
            || (1296..=1330).contains(&fx)
            || (1368..=1402).contains(&fx)
            || (1440..=1474).contains(&fx);
    }
    if t.kind == 467 {
        return fx / 36 == 13;
    }
    false
}

/// `Chest.IsLockedOrInUse` (`Chest.cs:178-188`): true when any player is
/// browsing the chest (`IsPlayerInChest`, `Chest.cs:267-274`) or the tile is
/// locked. The bank branch (`bankChest` → false) is unported — the port has
/// no bank chests until M3-STORE-1, which must restore the early `false`.
/// The 255-player loop collapses to the local player (single player): their
/// `Player.chest` (`-1` = none) against the index. An absent or removed slot
/// is neither locked nor in use (the C# instance method can only run on a
/// found chest).
pub(crate) fn is_locked_or_in_use(
    chests: &[WorldChest],
    world: &World,
    idx: usize,
    open_player_chest: i32,
) -> bool {
    let Some(chest) = chests.get(idx).filter(|c| !is_removed(c)) else {
        return false;
    };
    if open_player_chest == idx as i32 {
        return true;
    }
    is_locked(world, chest.tile.0, chest.tile.1)
}

/// `Chest.UsingChest` (`Chest.cs:498-509`): the first active player browsing
/// chest `i`, or `-1`. Single-player reading: the local player is the only
/// candidate, so the answer is whether they are browsing an existing (live)
/// slot — the C# player index collapses to this bool.
pub(crate) fn using_chest(chests: &[WorldChest], idx: usize, open_player_chest: i32) -> bool {
    chests.get(idx).is_some_and(|c| !is_removed(c)) && open_player_chest == idx as i32
}

// --- Panel name defaulting (M3-CHEST-UI phase 3) ----------------------------
//
// `ChestUI.DrawName`'s tile-style chain (`ChestUI.cs:124-140`). The vanilla
// strings ride the embedded `LegacyChestType.*` localization JSON, which is
// not on disk in this repo, so the names derive from the port's generated
// item table — the item placing `(tile, place_style)` carries the same en-US
// string (recorded deviation, phase 6 `docs/ANALYSIS.md`).

/// The chest's default name from the tile at the panel's origin
/// (`ChestUI.cs:124-140`): tile 21 → `Lang.chestType[frameX / 36]`, tile 467
/// style 4 → item 3988's name, tile 467 → `Lang.chestType2[frameX / 36]`,
/// tile 88 → `Lang.dresserType[frameX / 54]`. An empty string when the style
/// has no name (styles that cannot occur through placement — see
/// [`chest_type_name`]).
pub(crate) fn default_chest_name(kind: u16, frame_x: i16) -> String {
    if kind == 21 {
        return chest_type_name(frame_x as i32 / 36, 21);
    }
    if kind == 467 {
        // `ChestUI.cs:129-131`: the Dead Man's Chest is style 4.
        let style = frame_x as i32 / 36;
        if style == 4 {
            return crate::registry::name(3988).to_string();
        }
        return chest_type_name(style, 467);
    }
    if kind == 88 {
        return chest_type_name(frame_x as i32 / 54, 88);
    }
    String::new()
}

/// The `LegacyChestType.{style}` / `LegacyChestType2.{style}` /
/// `LegacyDresserType.{style}` string for a `(tile, style)`: the vanilla name
/// of the item that places it. Tile 21's non-placeable locked frames render
/// the vanilla `Locked <twin>` pattern — `Chest.Lock` shifts the style by +1
/// (frameX +36) or +5 (frameX +180, the biome chests 18-22), `Chest.cs:449-464`,
/// and the localization table names those frames "Locked <chest>" (styles
/// 2/4 are the pinned "Locked Gold/Shadow Chest").
fn chest_type_name(style: i32, kind: u16) -> String {
    let find = |place_style: i32| {
        crate::itemdata::ITEMS
            .iter()
            .find(|d| d.tile == Some(kind) && d.place_style == place_style)
            .map(|d| d.name)
    };
    // `Lang.cs:503-506`: `chestType[52..]` aliases `chestType[0]` (the table
    // is 54 entries for a style space that stops at 53).
    let style = if kind == 21 && style >= 52 { 0 } else { style };
    if let Some(name) = find(style) {
        return name.to_string();
    }
    let twin = match (kind, style) {
        (21, 2) | (21, 4) | (21, 36) | (21, 38) | (21, 40) => Some(style - 1),
        (21, 23..=27) => Some(style - 5),
        _ => None,
    };
    twin.and_then(find)
        .map(|name| format!("Locked {name}"))
        .unwrap_or_default()
}

// --- Open/close + locking interaction (M3-CHEST-UI phase 2) ----------------
//
// The `Player.cs` chest branch: the right-click open path
// (`TileInteractionsUse`, `Player.cs:33056-33260`), `Player.OpenChest`
// (`:33505-33526`) and the per-tick range close
// (`HandleBeingInChestRange`, `:31015-31097` + the hitbox table
// `:31099-31124`), plus `Chest.Lock`/`Chest.Unlock` (`Chest.cs:346-496`).
// Out: the rigged-chest arm (`:33112`/:33233/:33253-33258 — wiring, no
// owning task), the net messages (31/33/52/59), `Main.stackSplit = 600`
// (`:33117`/:33232 — the net quick-stack lock, M3-STORE-2), the void-bag
// key arm (`:33200-33221`, M3-STORE-1), the bank tiles 29/97/463/491
// (`:33056`'s non-`BasicChest` half, M3-STORE-1) and the achievements
// (`HandleSpecialEvent(16)` `:33243-33246`, `NotifyProgressionEvent(19/20)`
// `Chest.cs:365`/:382/:403/:477 — no `CondKind` arm yet).

/// `TileID.Sets.BasicChest` (`TileID.cs:570`) — the world-chest tiles the
/// open/lock paths serve. The bank families (29/97/463/491) stay out until
/// M3-STORE-1.
fn is_basic_chest(kind: u16) -> bool {
    kind == 21 || kind == 467
}

/// The 2×2 chest's top-left tile from a clicked cell
/// (`Player.cs:33062-33066`): the frame's sheet column reduced mod 2 (the
/// `for (num74 = frameX / 18; num74 > 1; num74 -= 2)` loop) and the row from
/// `frameY / 18`. `None` unless the cell is an active `BasicChest` tile (the
/// C# interaction chain only runs on active tiles, `:32035-32041`).
fn chest_origin(world: &World, x: i32, y: i32) -> Option<(i32, i32)> {
    let t = world.tile(x, y)?;
    if !t.active() || !is_basic_chest(t.kind) {
        return None;
    }
    let mut col = t.frame_x / 18;
    while col > 1 {
        col -= 2;
    }
    Some((x - col as i32, y - (t.frame_y / 18) as i32))
}

/// The locked chest's key item id (`Player.cs:33145-33172`) — frame-derived,
/// not style-table-derived: 327 (golden key) default, 329 (shadow key) for
/// `frameX` 144-178, `1533 + (style - 23)` (the biome keys) for 828-1006 and
/// 4714 (temple key) for tile 467 style 13.
fn locked_key_id(world: &World, x: i32, y: i32) -> u16 {
    let Some(t) = world.tile(x, y) else {
        return 327;
    };
    let fx = t.frame_x;
    let mut key = 327u16;
    if t.kind == 21 {
        if (144..=178).contains(&fx) {
            key = 329; // `:33150-33153`
        }
        if (828..=1006).contains(&fx) {
            // `:33154-33164` — the style column: `frameX / 18` halved down
            // (`num78`/`num79`), the key ids 1533-1537.
            let mut col = fx / 18;
            let mut style = 0i16;
            while col >= 2 {
                col -= 2;
                style += 1;
            }
            key = (1533 + (style - 23)) as u16;
        }
    } else if t.kind == 467 && fx / 36 == 13 {
        key = 4714; // `:33166-33171`
    }
    key
}

/// `Chest.Unlock` (`Chest.cs:346-426`): shift the locked frames off the 2×2
/// (`-36`, or `-180` for the Plantera-gated styles 23-27; `-36` for tile 467
/// style 13, also gated — `Chest.cs:376`/:397). Any missing tile of the 2×2
/// refuses (`:348-351`); an unsupported style refuses (`default`/`else`).
/// `plant_boss` is `NPC.downedPlantBoss`'s stand-in — the flag has no world
/// home yet (the recorded assumption fallback: the call site passes `true`;
/// flip it to the flag when the boss work lands it). The sound (22,
/// `Chest.cs:408`) and the dust/achievements are the caller's/the phase-6
/// deviation's — this returns whether the unlock happened.
pub(crate) fn unlock_chest(world: &mut World, x: i32, y: i32, plant_boss: bool) -> bool {
    for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
        if world.tile(x + dx, y + dy).is_none() {
            return false;
        }
    }
    // `Chest.cs:352-407` — the style table (`num2 = frameX / 36`).
    let (kind, style) = {
        let t = world.tile(x, y).expect("the origin tile was just checked");
        (t.kind, t.frame_x / 36)
    };
    let shift = match kind {
        21 => match style {
            2 | 4 | 36 | 38 | 40 => Some(36),
            23..=27 => {
                if !plant_boss {
                    return false; // `Chest.cs:376-379`
                }
                Some(180)
            }
            _ => None,
        },
        467 if style == 13 => {
            if !plant_boss {
                return false; // `Chest.cs:397-400`
            }
            Some(36)
        }
        _ => None,
    };
    let Some(shift) = shift else {
        return false;
    };
    // `Chest.cs:409-424` — every 2×2 tile of the chest's type shifts.
    shift_chest_frames(world, x, y, kind, -shift);
    true
}

/// `Chest.Lock` (`Chest.cs:428-496`): the mirror shift (`+36`, or `+180` for
/// the Plantera-gated styles 18-22; tile 467 style 12 → 13, gated —
/// `Chest.cs:454`/:472). Its player-side caller (the lock-item use arm,
/// `Player.cs:41651`) stays out — it is an `editing.rs` tile-use path
/// (PHYS-7's file); the function is ported + tested for the DW-2.5 table.
pub(crate) fn lock_chest(world: &mut World, x: i32, y: i32, plant_boss: bool) -> bool {
    for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
        if world.tile(x + dx, y + dy).is_none() {
            return false;
        }
    }
    // `Chest.cs:434-481` — the style table.
    let (kind, style) = {
        let t = world.tile(x, y).expect("the origin tile was just checked");
        (t.kind, t.frame_x / 36)
    };
    let shift = match kind {
        21 => match style {
            1 | 3 | 35 | 37 | 39 => Some(36),
            18..=22 => {
                if !plant_boss {
                    return false; // `Chest.cs:454-457`
                }
                Some(180)
            }
            _ => None,
        },
        467 if style == 12 => {
            if !plant_boss {
                return false; // `Chest.cs:472-475`
            }
            Some(36)
        }
        _ => None,
    };
    let Some(shift) = shift else {
        return false;
    };
    // `Chest.cs:483-494` — every 2×2 tile of the chest's type shifts.
    shift_chest_frames(world, x, y, kind, shift);
    true
}

/// The shared `Unlock`/`Lock` frame pass (`Chest.cs:409-424`/:483-494): the
/// shift applies to every 2×2 tile whose type matches the chest's.
fn shift_chest_frames(world: &mut World, x: i32, y: i32, kind: u16, shift: i16) {
    for dx in 0..=1 {
        for dy in 0..=1 {
            if let Some(t) = world.tile_mut(x + dx, y + dy) {
                if t.kind == kind {
                    t.frame_x += shift;
                }
            }
        }
    }
}

/// `Player.OpenChest` (`Player.cs:33505-33526`): the index, the crafting
/// panel close (`NewCraftingUI.Close(quiet: true)` `:33516`), the inventory
/// open (`Main.playerInventory = true` `:33517`) and the tile origin. The
/// `ItemSlot.SetGlowForChest`/input-navigator arms are UI state the port
/// does not carry (`:33508-33515`/:33518-33523).
fn open_chest(app: &mut App, x: i32, y: i32, new_chest: usize) {
    app.chest = new_chest as i32;
    app.crafting_open = false;
    app.inventory_open = true;
    app.chest_x = x;
    app.chest_y = y;
}

/// `Player.cs:33230-33252` — the open toggle: the same index closes (sound
/// 11), a fresh open plays 10 (the `frameX 36..72` Shadow-chest special
/// event 16 arm `:33243-33246` is the recorded achievements deviation), a
/// re-open with another chest already open plays 12. Returns the legacy
/// sound id (the `SoundEngine.PlaySound` argument) so the caller plays it
/// and the tests pin it.
fn toggle_open(app: &mut App, x: i32, y: i32, new_chest: usize) -> u32 {
    if new_chest as i32 == app.chest {
        app.chest = -1; // `:33236`
        return 11; // `:33237`
    }
    let was_closed = app.chest == -1; // `:33239`
    open_chest(app, x, y, new_chest);
    if was_closed {
        10 // `:33242`
    } else {
        12 // `:33251`
    }
}

fn play_menu_sound(app: &App, legacy_id: u32) {
    // `SoundID.MenuOpen/MenuClose/MenuTick = 10/11/12` (`SoundID.cs:42/44/46`),
    // played without a position in the C# (`:33242`/:33237/:33251).
    match legacy_id {
        10 => app.audio.play("Menu_Open"),
        11 => app.audio.play("Menu_Close"),
        12 => app.audio.play("Menu_Tick"),
        _ => {}
    }
}

/// The right-click open path (`Player.cs:33056-33260`), the editing.rs
/// interception's callee — mirrors `try_open_shop_at_cursor`'s contract:
/// `true` consumes the click. Every active `BasicChest` tile consumes
/// (`flag2`/`Main.mouseRightRelease = false`, `:33058-33059`), even a locked
/// one the key arm can't open.
pub(crate) fn try_open_chest_at_cursor(app: &mut App, world_pos: Vec2) -> bool {
    let x = (world_pos.x / TILE_PX).floor() as i32;
    let y = (world_pos.y / TILE_PX).floor() as i32;
    let Some((ox, oy)) = chest_origin(&app.world, x, y) else {
        return false;
    };
    // `:33087-33090` — `CloseSign`/`SetTalkNPC(-1)`/the chat-text clears:
    // the port's dialog + shop close (`Main.SetNPCShopIndex(-1)`'s
    // equivalent). The `Main.editChest` flush (`:33091-33096`, sound 12)
    // joins with phase 5's rename state; the `editedChestName` net flush
    // (`:33097-33101`) and the MP arm (`:33103-33119`) are out.
    app.close_shop();
    // `:33142-33228` — the single-player default arm (`num73 == 0` for the
    // `BasicChest` tiles): `flag14` marks a locked-chest click, which blocks
    // the `FindChest` at `:33223-33226`, so the click is consumed without
    // opening — the chest opens on the next click.
    let locked = is_locked(&app.world, ox, oy); // `:33143`
    if locked {
        let key = locked_key_id(&app.world, ox, oy);
        // `:33175` — `flag16`: the shadow key (329) unlocks without
        // consuming; every other key decrements.
        let consumes = key != 329;
        // `:33176-33199` — the key scan over `inventory[0..=58]`: the port's
        // inventory grid (0-49), the coins (50-53), the ammo (54-57) and the
        // trash slot (58) — `EquipSlots` mirrors the C# array layout
        // (`inv.rs:26-34`); the equipment rows are outside the scan. The
        // loop keeps scanning after a success, but `Chest.Unlock` answers
        // false once the chest is unlocked — exactly one key is consumed.
        // (`flag15`'s only reader is the void-bag arm `:33200`, which is out.)
        for (slot, is_trash) in app
            .inventory
            .iter_mut()
            .map(|s| (s, false))
            .chain(app.equip.coins.iter_mut().map(|s| (s, false)))
            .chain(app.equip.ammo.iter_mut().map(|s| (s, false)))
            .chain(std::iter::once((&mut app.equip.trash, true)))
        {
            if slot.kind != key || slot.count == 0 {
                continue;
            }
            // The Plantera gate passes until the world flag exists (the
            // recorded fallback — flip to the flag when the boss work lands).
            if !unlock_chest(&mut app.world, ox, oy, true) {
                continue;
            }
            if consumes {
                // `:33183-33189` — the consumed key's stack decrements; an
                // emptied slot turns to air.
                slot.count -= 1;
                if slot.count == 0 {
                    *slot = EMPTY_SLOT;
                }
                if is_trash {
                    // `:33190-33193` — the trash slot mirrors into the
                    // cursor stack (`Main.mouseItem`).
                    app.mouse_item = *slot;
                }
            }
        }
        // `:33200-33221` — the void-bag arm is M3-STORE-1.
    }
    if !locked {
        // `:33225` — `Chest.FindChest` on an unlocked click. The port creates
        // the store entry lazily on the first open (the placement-time
        // creation is unreachable without editing.rs edits — the recorded
        // deviation; an empty entry answers like an absent one).
        let new_chest = match find_chest(&app.world_chests, ox, oy) {
            Some(i) => i,
            None => create_chest(&mut app.world_chests, ox, oy),
        };
        // `:33230-33252` — the open toggle. (`Main.stackSplit = 600`
        // `:33232` and the rigged-chest arm are out.)
        let sound = toggle_open(app, ox, oy, new_chest);
        play_menu_sound(app, sound);
    }
    true
}

/// `Player.HandleBeingInChestRange` (`Player.cs:31015-31097`) — the per-tick
/// close: out of the chest's interaction range or a dead tile force-closes
/// with sound 11 (`:31078-31090`). The bank-projectile arms
/// (`:31019-31076`) are M3-STORE-1. Runs for alive players only (the C#
/// call site sits after the dead branch's return, `:24752`/:25290) — the
/// `update_player` hook inherits that gating.
pub(crate) fn tick_open(app: &mut App) {
    if app.chest == -1 {
        return; // `:31017`
    }
    let Some(p) = app.player.as_ref() else {
        return;
    };
    // `:31078` — `IsInInteractionRangeToMultiTileHitbox` (`:31099-31124`).
    if !in_interaction_range_to_multi_tile_hitbox(app, p) {
        let id = force_close(app);
        play_menu_sound(app, id); // `:31082` — `SoundEngine.PlaySound(11)`
        return;
    }
    // `:31086-31090` — a dead tile force-closes (the `Main.cs:41820-41823`
    // stale-chest equivalent rides this check).
    if app
        .world
        .tile(app.chest_x, app.chest_y)
        .is_none_or(|t| !t.active())
    {
        let id = force_close(app);
        play_menu_sound(app, id); // `:31088`
    }
}

/// `:31080-31090` — the force close: `chest = -1`. Returns the legacy sound
/// id (11 — `SoundEngine.PlaySound(11)`, played by the caller) so the tests
/// pin it.
fn force_close(app: &mut App) -> u32 {
    app.chest = -1;
    11
}

/// `Player.IsInInteractionRangeToMultiTileHitbox` (`Player.cs:31099-31124`),
/// the `BasicChest` row (`:31107-31109` — the 97/88/29/463/491 rows are the
/// bank/dresser families): the 32×32 rect at the chest origin, deflated by
/// `Inflate(-1, -1)` (`:31119`), the player's center clamped in
/// (`Utils.ClosestPointInRect`, `Utils.cs:1581-1595`), the resulting tile
/// point tested against the `TileReachCheckSettings.Simple` bounds
/// (`DataStructures/TileReachCheckSettings.cs:81-85` — ranges 5/3,
/// `Player.cs:57113-57114`, under the 20 cap; `Collision.InTileBounds`,
/// `Collision.cs:3776`, inclusive).
fn in_interaction_range_to_multi_tile_hitbox(app: &App, p: &player::Player) -> bool {
    let (rx, ry, rw, rh) = (app.chest_x * 16 + 1, app.chest_y * 16 + 1, 30, 30);
    let (center_x, center_y) = p.center();
    let px = center_x.clamp(rx as f32, (rx + rw) as f32);
    let py = center_y.clamp(ry as f32, (ry + rh) as f32);
    // `ToTileCoordinates` — the C# `(int)` truncation.
    let tx = (px / 16.0) as i32;
    let ty = (py / 16.0) as i32;
    let lx = (p.x / 16.0) as i32 - 5;
    let hx = ((p.x + player::WIDTH) / 16.0).ceil() as i32 - 1 + 5;
    let ly = (p.y / 16.0) as i32 - 3;
    let hy = ((p.y + p.height) / 16.0).ceil() as i32 - 1 + 3;
    (lx..=hx).contains(&tx) && (ly..=hy).contains(&ty)
}

// --- Slot interactions (M3-CHEST-UI phase 4) --------------------------------
//
// The chest-side click helpers: the shift transfers (`ItemSlot.OverrideLeftClick`
// cursor 8/9, `ItemSlot.cs:800-850`), `ChestUI.TryPlacingInChest`
// (`ChestUI.cs:1209-1289`) and the favorite context tables (`ItemSlot.cs:
// 386-394`). The per-slot click dispatch itself lives in `app/clicks.rs`
// (`handle_chest_clicks`); the right-click stack-split's timing state is
// `Main.stackSplit`'s cluster (`Main.cs:1693-1697`), ticked by `app/frame.rs`.
// Out: the bank containers (M3-STORE-1 — `GetCurrentContainer`'s chest < -1
// arms and the bank-only `IsBlockedFromTransferIntoChest`, `ChestUI.cs:
// 1291-1302`), the net messages 32/38 (`:1247`/:1258/:1283, `ItemSlot.cs:808`),
// `DoCoins`' roll-over (`Player.cs:39567-39569` — unported everywhere), the
// cursor-override texture draw (see `App.cursor_override`).

/// Reads chest slot `slot` of the store entry `idx` with the implicit-air
/// encoding (decision #4): positions beyond `slots.len()` answer `(0, 0)`,
/// the C#'s air `Chest.item[i]` entries. `None` for a missing/removed entry.
pub(crate) fn chest_slot(chests: &[WorldChest], idx: usize, slot: usize) -> Option<ChestSlot> {
    chest_slots(chests, idx).map(|slots| slots.get(slot).copied().unwrap_or((0, 0)))
}

/// Writes chest slot `slot` of the entry `idx`, extending with airs first when
/// the position is past the stored length (decision #4). The written value is
/// normalized to air — the C#'s `SetDefaults(0)` / `new Item()` writes
/// (`ItemSlot.cs:976-979`, `ChestUI.cs:1244/:1278`) — so the store never holds
/// a half-air slot. `false` for a missing/removed entry.
pub(crate) fn set_chest_slot(
    chests: &mut [WorldChest],
    idx: usize,
    slot: usize,
    value: ChestSlot,
) -> bool {
    let Some(chest) = chests.get_mut(idx).filter(|c| !is_removed(c)) else {
        return false;
    };
    let value = if value.0 == 0 || value.1 == 0 {
        (0, 0)
    } else {
        value
    };
    if slot >= chest.slots.len() {
        chest.slots.resize(slot + 1, (0, 0));
    }
    chest.slots[slot] = value;
    true
}

/// `ItemSlot.canFavoriteAt` (`ItemSlot.cs:386-394`): only the inventory
/// contexts 0/1/2/32 can favorite — a chest slot (3) cannot, so its items
/// never carry the flag and the cursor-3 favorite arm never fires on them.
pub(crate) fn can_favorite_at(context: i32) -> bool {
    matches!(context, 0 | 1 | 2 | 32)
}

/// `ItemSlot.retainFavoriteOnMoveAt` (`ItemSlot.cs:390-394`): the contexts
/// whose `MouseItemSwap` keeps the favorite flag (`:1255-1263` — every other
/// context clears it on both sides). Context 3 is not in the table, so
/// moving an item out of a chest clears `favorited` — the port's `ItemSlot`
/// carries no favorite field yet (item.rs:52-57), so the clear is a no-op
/// with this citation until M7-UI-2 lands the flag.
pub(crate) fn retain_favorite_on_move_at(context: i32) -> bool {
    matches!(context, 0 | 1 | 2 | 32 | 5)
}

/// `Player.ItemSpace(item).CanTakeItemToPersonalInventory`
/// (`Player.cs:39153-39197`) for the shift-out cursor: any grid slot accepts
/// the item (`CanItemSlotAcceptPickup`, `:39220-39232` — empty, or a
/// same-kind partial stack), or the ungated same-kind merge scan over the
/// ammo slots (`:39185-39191`). The `IsAPickup`/`uniqueStack` sets, the
/// `ammo > 0` FillAmmo arm (`:39175-39184`) and the Cofveve void scan
/// (`:39192-39195`) have no port counterparts yet.
pub(crate) fn inventory_has_room_for(app: &App, kind: u16) -> bool {
    if kind == 0 {
        return false;
    }
    let max = registry::max_stack(kind);
    if app
        .inventory
        .iter()
        .any(|s| s.count == 0 || (s.kind == kind && s.count < max))
    {
        return true;
    }
    app.equip
        .ammo
        .iter()
        .any(|s| s.count > 0 && s.kind == kind && s.count < max)
}

/// `ChestUI.TryPlacingInChest` (`ChestUI.cs:1209-1289`) — the shift-in
/// transfer from inventory `slot` into the open chest: a stack-merge pass
/// over every partial same-kind chest slot, then the whole remainder into the
/// first empty slot. The return mirrors the C#'s `flag2`: only the
/// `just_check` branches set it (`:1234-1237`/:1271-1274), so the real pass
/// always answers false — its only caller ignores the value
/// (`ItemSlot.cs:846`) — and mutates without playing a cue for the probe.
/// The bank containers (`chest < -1`) are M3-STORE-1;
/// `IsBlockedFromTransferIntoChest` (`:1291-1302`) is a bank-only rule —
/// noted for STORE-1.
pub(crate) fn try_placing_in_chest(app: &mut App, slot: usize, just_check: bool) -> bool {
    if app.chest < 0 {
        return false; // `GetCurrentContainer`'s world-chest arm (`:1213`)
    }
    let item = app.inventory[slot];
    if item.kind == 0 || item.count == 0 {
        return false; // an air item never merges and never fills (`item.maxStack` 0)
    }
    let chest = app.chest as usize;
    let kind = item.kind;
    let max = registry::max_stack(kind);
    let mut stack = item.count;
    let mut placeable = false;
    // The merge pass (`ChestUI.cs:1221-1261`) — stackable kinds only.
    if max > 1 {
        for i in 0..DEFAULT_MAX_ITEMS as usize {
            let Some(s) = chest_slot(&app.world_chests, chest, i) else {
                break;
            };
            // `:1225` — a full, air (an air slot's maxStack 0 fails
            // `stack >= maxStack`) or alien-kind slot skips.
            if s.0 == 0 || s.1 >= registry::max_stack(s.0) || s.0 != kind {
                continue;
            }
            let slot_max = registry::max_stack(s.0);
            // `:1229-1233` — the amount, capped by the slot's room.
            let mut num = stack;
            if stack + s.1 > slot_max {
                num = slot_max - s.1;
            }
            if just_check {
                placeable = placeable || num > 0; // `:1234-1237`
                break;
            }
            // `:1239-1241` — the merge, sound 7. (The `:1251-1255`
            // `item2[i].type == 0` re-clone arm is dead behind the `:1225`
            // skip: a non-skipped slot already carries the transferring kind.)
            set_chest_slot(&mut app.world_chests, chest, i, (kind, s.1 + num));
            app.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:1241`)
            stack -= num;
            if stack == 0 {
                break; // `:1242-1249` — `SetDefaults(0)`
            }
        }
    }
    // The empty-slot pass (`ChestUI.cs:1262-1287`) — one slot takes the whole
    // remainder.
    if stack > 0 {
        for j in 0..DEFAULT_MAX_ITEMS as usize {
            let Some(s) = chest_slot(&app.world_chests, chest, j) else {
                break;
            };
            if s.1 != 0 {
                continue; // `:1267`
            }
            if just_check {
                placeable = true; // `:1271-1274`
                break;
            }
            app.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:1276`)
            set_chest_slot(&mut app.world_chests, chest, j, (kind, stack));
            stack = 0; // `:1277-1278` — `item.Clone()` + `SetDefaults(0)`
            break;
        }
    }
    if !just_check {
        // The drained slot's remainder — air when fully moved
        // (`:1244`/:1278's `SetDefaults(0)` write-through).
        app.inventory[slot] = if stack == 0 {
            EMPTY_SLOT
        } else {
            ItemSlot {
                kind,
                name: registry::name(kind),
                count: stack,
            }
        };
    }
    placeable
}

/// The shift-out transfer of one chest slot into the player's inventory —
/// the cursor-8 arm of `ItemSlot.OverrideLeftClick` (`ItemSlot.cs:800-812`):
/// `inv[slot] = player.GetItem(inv[slot], QuickTransferFromSlot)`.
/// `GetItemSettings.QuickTransferFromSlot` (`GetItemSettings.cs:58`) is
/// NoText-only. The remainder stays in the chest slot. Out: the
/// `FitsAmmoSlot`/`FillAmmo` pre-pass (`Player.cs:39345-39352` — no
/// ammo-accept registry yet), `DoCoins` (`:39567-39569`), the void vault
/// (`:39395-39398`).
pub(crate) fn quick_transfer_from_slot(app: &mut App, chest_slot_idx: usize) {
    /// `GetItem_FillIntoOccupiedSlot`'s merge step (`Player.cs:39485-39524`):
    /// as much of `remainder` as the same-kind partial slot accepts.
    fn merge_into(s: &mut ItemSlot, kind: u16, max: u32, remainder: &mut u32) -> u32 {
        if s.count > 0 && s.kind == kind && s.count < max && *remainder > 0 {
            let moved = (max - s.count).min(*remainder);
            s.count += moved;
            *remainder -= moved;
            moved
        } else {
            0
        }
    }

    /// `GetItem_FillEmptyInventorySlot` (`Player.cs:39551-39578`): one empty
    /// slot takes the whole remainder.
    fn fill_empty(s: &mut ItemSlot, kind: u16, remainder: &mut u32) -> bool {
        if s.count == 0 && *remainder > 0 {
            *s = ItemSlot {
                kind,
                name: registry::name(kind),
                count: *remainder,
            };
            *remainder = 0;
            true
        } else {
            false
        }
    }

    if app.chest < 0 {
        return;
    }
    let chest_idx = app.chest as usize;
    let Some((kind, stack)) = chest_slot(&app.world_chests, chest_idx, chest_slot_idx) else {
        return;
    };
    if kind == 0 || stack == 0 {
        return;
    }
    let max = registry::max_stack(kind);
    let is_coin = registry::is_coin(kind); // `Item.IsACoin` (`Item.cs:373-383`)
    let mut remainder = stack;
    let App {
        inventory,
        equip,
        audio,
        world_chests,
        ..
    } = app;
    // The occupied-slot pass (`Player.cs:39353-39364`): coins start at the C#
    // coin slots 50-53 (the `num2 = -4` arm, `:39340-39343`), then the grid.
    // `:39444` — `IsACoin ? 38 : 7` per accepted slot; the port's coin cue is
    // "Coin_0" (the drop-pickup precedent).
    if is_coin {
        for s in equip.coins.iter_mut() {
            if merge_into(s, kind, max, &mut remainder) > 0 {
                audio.play("Coin_0");
            }
            if remainder == 0 {
                break;
            }
        }
    }
    for s in inventory[..INV].iter_mut() {
        if merge_into(s, kind, max, &mut remainder) > 0 {
            audio.play("Grab"); // `:39444` — the non-coin arm
        }
        if remainder == 0 {
            break;
        }
    }
    // The empty-slot passes (`Player.cs:39365-39394`): the hotbar-first arm
    // for usable kinds (`useStyle != 0`, `:39365-39374`), then the
    // non-favorited REVERSE scan (`:39387-39394`; chest items cannot be
    // favorited — [`can_favorite_at`] excludes ctx 3 — so the favorited
    // forward arm `:39375-39384` is unreachable here). Coins scan the coin
    // slots descending first (`num = 54`).
    if remainder > 0 && crate::item::use_style(kind) != 0 {
        for s in inventory[..HOTBAR].iter_mut() {
            if fill_empty(s, kind, &mut remainder) {
                audio.play("Grab"); // `:39559`
                break;
            }
        }
    }
    if remainder > 0 && is_coin {
        for s in equip.coins.iter_mut().rev() {
            if fill_empty(s, kind, &mut remainder) {
                audio.play("Coin_0"); // `:39559`
                break;
            }
        }
    }
    if remainder > 0 {
        for s in inventory[..INV].iter_mut().rev() {
            if fill_empty(s, kind, &mut remainder) {
                audio.play("Grab"); // `:39559`
                break;
            }
        }
    }
    // `:800-812`'s `inv[slot] = ...` — the unplaced remainder stays in the
    // chest slot.
    set_chest_slot(world_chests, chest_idx, chest_slot_idx, (kind, remainder));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemSlot;
    use crate::player::Player;

    fn world() -> World {
        World::new("t".into(), 120, 120)
    }

    fn set_tile(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].frame_x = frame_x;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    /// DW-1.1 — `CreateChest`/`FindChest`/`FindEmptyChest`/`RemoveChest`
    /// (`Chest.cs:513-541`, `:590-604`, `:132-141`): the store cycle with the
    /// coord-collision `-1` rule and the first-null-slot reuse.
    #[test]
    fn store_create_find_find_empty_is_empty_remove_cycles() {
        let mut chests: Vec<WorldChest> = Vec::new();
        // Create claims the first null slot (`Chest.cs:522-541`).
        let a = create_chest(&mut chests, 10, 18);
        assert_eq!(a, 0);
        assert_eq!(chests[a].tile, (10, 18));
        assert!(
            chests[a].slots.is_empty(),
            "a fresh chest is empty instances (Chest.cs:107-109)"
        );
        // `FindChest` answers the coords, misses elsewhere (`Chest.cs:513-520`).
        assert_eq!(find_chest(&chests, 10, 18), Some(0));
        assert_eq!(find_chest(&chests, 11, 18), None);
        // A second chest takes the next slot.
        assert_eq!(create_chest(&mut chests, 30, 40), 1);
        // The collision rule (`Chest.cs:529-533`): any chest at the coords
        // refuses the empty-slot search with `-1` — the C# `None`.
        assert_eq!(find_empty_chest(&chests, 10, 18), None);
        assert_eq!(find_empty_chest(&chests, 30, 40), None);
        // Free coords hand out the append slot.
        assert_eq!(find_empty_chest(&chests, 50, 50), Some(2));
        // `RemoveChest` nulls the slot; `FindChest` misses it, the slot reads
        // as absent, and the next empty-slot search reuses the index
        // (`Chest.cs:524-536` first-null rule).
        remove_chest(&mut chests, 0);
        assert_eq!(find_chest(&chests, 10, 18), None);
        assert_eq!(chest_slots(&chests, 0), None);
        assert_eq!(find_empty_chest(&chests, 90, 90), Some(0));
        // `CreateChest` after the removal reclaims slot 0; held indices of
        // the other slots never shift (`Chest.cs:132-141` nulls in place).
        assert_eq!(create_chest(&mut chests, 90, 90), 0);
        assert_eq!(chests[0].tile, (90, 90));
        assert_eq!(chests[1].tile, (30, 40));
    }

    /// DW-1.1 — `Chest.IsEmpty` (`Chest.cs:166-176`) and the empty-entry ≡
    /// absent rule: an emptied entry answers like an absent one for the
    /// break refusal (the `editing.rs` closure's `stack > 0` scan) while
    /// `FindChest` still finds it — the chest exists, it is just empty.
    #[test]
    fn is_empty_and_empty_entry_equivalence() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        assert!(chests[idx].is_empty());
        chests[idx].slots = vec![(9, 6)];
        assert!(!chests[idx].is_empty());
        // A drained slot is air in place (`Item.IsAir`: type 0 and stack 0),
        // not a removed entry.
        chests[idx].slots = vec![(0, 0), (619, 4)];
        assert!(!chests[idx].is_empty());
        chests[idx].slots = vec![(0, 0), (0, 0)];
        assert!(chests[idx].is_empty());
        assert_eq!(
            find_chest(&chests, 10, 18),
            Some(idx),
            "an empty entry is not a removed slot"
        );
    }

    /// DW-1.1 — the contract's `get`/`get_mut` (`chest_slots`/
    /// `chest_slots_mut`): the slot slice at an index, `None` for
    /// out-of-range and removed slots, `get_mut` writing through.
    #[test]
    fn chest_slots_on_removed_and_out_of_range() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        chests[idx].slots = vec![(9, 6), (0, 0), (619, 4)];
        assert_eq!(
            chest_slots(&chests, idx),
            Some(&[(9, 6), (0, 0), (619, 4)][..])
        );
        assert_eq!(chest_slots(&chests, 5), None);
        remove_chest(&mut chests, idx);
        assert_eq!(chest_slots(&chests, idx), None);
        let other = create_chest(&mut chests, 40, 40);
        // The reuse claimed the tombstone's slot; a fresh chest is empty
        // until a slot is stocked (phase 4's placement grows the list).
        chests[other].slots = vec![(9, 6)];
        if let Some(slots) = chest_slots_mut(&mut chests, other) {
            slots[0] = (48, 1);
        }
        assert_eq!(chests[other].slots[0], (48, 1));
    }

    /// DW-1.1 — `Main.maxChests` (`Main.cs:1033`) bounds the store: the 8000th
    /// chest takes the last slot and the next create hits the C# `-1` path
    /// (`Chest.cs:524` scans `i < 8000`). `create_chest` panics on the
    /// caller-contract violation (its doc).
    #[test]
    #[should_panic(expected = "create_chest follows a find-first miss")]
    fn create_chest_refuses_the_8000th_chest() {
        let mut chests: Vec<WorldChest> = Vec::new();
        for i in 0..MAX_CHESTS {
            let x = (i % 1000) as i32;
            let y = 1000 + (i / 1000) as i32;
            assert_eq!(create_chest(&mut chests, x, y), i);
        }
        assert_eq!(
            find_empty_chest(&chests, 5000, 5000),
            None,
            "the store is full"
        );
        create_chest(&mut chests, 5000, 5000);
    }

    /// DW-1.2 — `Chest.IsLocked` (`Chest.cs:301-313`): every locked `frameX`
    /// range pinned at both boundaries, tile 467 style 13, the null-tile
    /// branch, and the no-active-check reading.
    #[test]
    fn is_locked_frame_boundaries_match_chest_cs() {
        let mut w = world();
        // Tile 21 (`TileID.Containers`): the six inclusive ranges — one probe
        // cell per range so the boundaries cannot mask each other.
        let ranges = [
            (72, 106),
            (144, 178),
            (828, 1006),
            (1296, 1330),
            (1368, 1402),
            (1440, 1474),
        ];
        for (n, &(lo, hi)) in ranges.iter().enumerate() {
            let x = 4 + n as i32 * 2;
            set_tile(&mut w, x, 10, 21, (lo - 1) as i16);
            assert!(!is_locked(&w, x, 10), "frameX {lo}-1 unlocks");
            set_tile(&mut w, x, 10, 21, lo as i16);
            assert!(is_locked(&w, x, 10), "frameX {lo} locks");
            set_tile(&mut w, x, 10, 21, hi as i16);
            assert!(is_locked(&w, x, 10), "frameX {hi} locks");
            set_tile(&mut w, x, 10, 21, (hi + 1) as i16);
            assert!(!is_locked(&w, x, 10), "frameX {hi}+1 unlocks");
        }
        // Tile 467 (`TileID.Containers2`): locked iff `frameX / 36 == 13`
        // (styles 0..12 and 14+ are plain chests).
        set_tile(&mut w, 4, 20, 467, 467);
        assert!(!is_locked(&w, 4, 20), "467/36 == 12 unlocks");
        set_tile(&mut w, 4, 20, 467, 468);
        assert!(is_locked(&w, 4, 20), "468/36 == 13 locks");
        set_tile(&mut w, 4, 20, 467, 503);
        assert!(is_locked(&w, 4, 20), "503/36 == 13 locks");
        set_tile(&mut w, 4, 20, 467, 504);
        assert!(!is_locked(&w, 4, 20), "504/36 == 14 unlocks");
        // A non-container tile is never locked; an unframed tile 21
        // (`frameX -1`, the "must be computed" sentinel) is unlocked.
        set_tile(&mut w, 4, 30, 1, 90);
        assert!(!is_locked(&w, 4, 30), "stone is not a chest");
        set_tile(&mut w, 6, 30, 21, -1);
        assert!(!is_locked(&w, 6, 30), "frameX -1 unlocks");
        // The C# null-tile branch (`Chest.cs:303-305`): a missing tile reads
        // locked. And the decode ignores activity, as the C# does.
        assert!(is_locked(&w, 119, 120), "out of bounds ≡ null tile");
        set_tile(&mut w, 8, 30, 21, 90);
        let i = w.index(8, 30);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        assert!(is_locked(&w, 8, 30), "an inactive locked frame still locks");
    }

    /// DW-1.3 — `Chest.IsLockedOrInUse` (`Chest.cs:178-188`): the in-use
    /// player wins before the lock decode; the bank arm is unported
    /// (M3-STORE-1 must restore `bankChest` → false); absent/removed slots
    /// are neither.
    #[test]
    fn is_locked_or_in_use_matches_chest_cs() {
        let mut w = world();
        set_tile(&mut w, 10, 18, 21, 0); // unlocked chest tile
        set_tile(&mut w, 30, 40, 21, 90); // locked chest tile
        let mut chests: Vec<WorldChest> = Vec::new();
        let open = create_chest(&mut chests, 10, 18);
        let locked = create_chest(&mut chests, 30, 40);
        // Idle player: the lock decode decides.
        assert!(!is_locked_or_in_use(&chests, &w, open, -1));
        assert!(
            is_locked_or_in_use(&chests, &w, locked, -1),
            "frameX 90 locks"
        );
        // `IsPlayerInChest` (`Chest.cs:267-274`): any browsing player puts it
        // in use — even an unlocked chest.
        assert!(is_locked_or_in_use(&chests, &w, open, open as i32));
        assert!(is_locked_or_in_use(&chests, &w, locked, locked as i32));
        // A removed or out-of-range slot is neither locked nor in use.
        remove_chest(&mut chests, open);
        assert!(!is_locked_or_in_use(&chests, &w, open, -1));
        assert!(!is_locked_or_in_use(&chests, &w, 9, -1));
    }

    /// `Chest.UsingChest` (`Chest.cs:498-509`) — the single-player reading:
    /// true iff the slot exists (not removed) and the player browses it.
    #[test]
    fn using_chest_follows_the_single_player_reading() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        assert!(using_chest(&chests, idx, idx as i32));
        assert!(!using_chest(&chests, idx, -1), "no chest open");
        assert!(
            !using_chest(&chests, idx, idx as i32 + 1),
            "another chest open"
        );
        remove_chest(&mut chests, idx);
        assert!(
            !using_chest(&chests, idx, idx as i32),
            "a null slot is unusable"
        );
    }

    #[test]
    fn default_chest_name_matches_the_chestui_chain() {
        // `ChestUI.cs:124-140` — the tile-style default the panel shows while
        // the stored name is empty. The strings are the port's generated item
        // names (the vanilla en-US localization's own values).
        assert_eq!(default_chest_name(21, 0), "Chest"); // style 0
        assert_eq!(default_chest_name(21, 36), "Gold Chest"); // style 1
        // The locked twins: `Chest.Lock` shifts +36 (`Chest.cs:449-453`).
        assert_eq!(default_chest_name(21, 72), "Locked Gold Chest");
        assert_eq!(default_chest_name(21, 144), "Locked Shadow Chest");
        // The biome chests shift +180 → styles 23-27 (`Chest.cs:454-458`).
        assert_eq!(default_chest_name(21, 828), "Locked Jungle Chest");
        // `Lang.cs:503-506`: styles 52+ alias style 0.
        assert_eq!(default_chest_name(21, 1908), "Chest");
        // Tile 467: style 4 is the Dead Man's Chest (`ChestUI.cs:129-131`).
        assert_eq!(default_chest_name(467, 144), "Dead Mans Chest");
        assert_eq!(default_chest_name(467, 180), "Solar Chest");
        // Tile 88 dressers read frameX / 54 (`:137-139`).
        assert_eq!(default_chest_name(88, 0), "Dresser");
        assert_eq!(default_chest_name(88, 54), "Ebonwood Dresser");
        // A style with no placing item and no twin rule renders nothing.
        assert_eq!(default_chest_name(467, 0), "");
    }

    // --- Phase 2: the open/close + locking interaction -----------------------

    /// A phase-1 tile probe with the frame's row set — the 2×2 placement
    /// needs both axes (the origin decode reads `frameY / 18`).
    fn set_chest_tile(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].frame_x = frame_x;
        w.tiles[i].frame_y = frame_y;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    /// A full 2×2 chest: the left column carries `origin_frame_x`, the right
    /// column `+18` (the 18 px sheet pitch); the bottom row is `frameY` 18.
    fn place_chest(w: &mut World, x: i32, y: i32, kind: u16, origin_frame_x: i16) {
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            set_chest_tile(
                w,
                x + dx,
                y + dy,
                kind,
                origin_frame_x + if dx == 1 { 18 } else { 0 },
                if dy == 1 { 18 } else { 0 },
            );
        }
    }

    /// A headless `App` over `w` with a spawned player (the `mining_app`
    /// shape, editing.rs:2559-2562).
    fn interact_app(w: World) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        app
    }

    /// The tile center's world position (the editing.rs cursor px → tile
    /// round-trip: `(px / TILE_PX).floor()`).
    fn click(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0)
    }

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
        }
    }

    /// DW-2.1 — the open/close cycle (`Player.cs:33056-33260` +
    /// `OpenChest :33505-33526`): the click sets `chest/chest_x/chest_y` and
    /// opens the inventory, the same chest's click closes (`chest = -1` —
    /// the C# keeps `chestX/Y`, `:33236`), and the toggle's sound ids are
    /// the C# 10 (fresh open `:33242`) / 11 (close `:33237`) / 12 (re-open
    /// with another chest open `:33251`). The store entry is created lazily
    /// on the first open (the phase-1 decision).
    #[test]
    fn dw_2_1_open_close_cycle_sets_chest_state_and_sounds() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        place_chest(&mut w, 30, 40, 21, 0);
        let mut app = interact_app(w);
        app.crafting_open = true;

        // The open click through the public path: state + the lazy entry.
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert_eq!(app.chest, 0);
        assert_eq!((app.chest_x, app.chest_y), (10, 18));
        assert!(app.inventory_open, "`Main.playerInventory = true` (:33517)");
        assert!(!app.crafting_open, "`NewCraftingUI.Close` (:33516)");
        assert_eq!(
            find_chest(&app.world_chests, 10, 18),
            Some(0),
            "the store entry is created on the first open"
        );
        assert!(app.world_chests[0].is_empty(), "a fresh entry is empty");

        // The toggle's sound ids, driven through the same helper the click
        // path uses (`:33230-33252`).
        assert_eq!(toggle_open(&mut app, 10, 18, 0), 11, "same chest closes");
        assert_eq!(app.chest, -1, "the close resets `chest`");
        assert_eq!(
            (app.chest_x, app.chest_y),
            (10, 18),
            "the C# keeps chestX/Y on close (:33236)"
        );
        // A fresh open with nothing open plays 10 (`:33239-33242`).
        let b = create_chest(&mut app.world_chests, 30, 40);
        assert_eq!(toggle_open(&mut app, 30, 40, b), 10);
        assert_eq!(app.chest, b as i32);
        // Opening another chest while one is open plays 12 (`:33248-33251`).
        assert_eq!(toggle_open(&mut app, 10, 18, 0), 12);
        assert_eq!(app.chest, 0);
    }

    /// DW-2.2 — the per-tick close (`HandleBeingInChestRange`,
    /// `Player.cs:31015-31097`): out of the `Simple` reach of the chest's
    /// hitbox, or a dead tile at the origin, force-closes with sound 11 —
    /// the close is the tick's job, not an open-time guard.
    #[test]
    fn dw_2_2_tick_close_forces_close_out_of_range_or_dead_tile() {
        let mut w = world();
        place_chest(&mut w, 10, 10, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 10)));
        assert_eq!(app.chest, 0);

        // In range (the player's center clamps into the chest's deflated
        // rect inside the ±5/±3 bounds): stays open.
        {
            let p = app.player.as_mut().unwrap();
            p.x = 150.0;
            p.y = 150.0;
        }
        tick_open(&mut app);
        assert_eq!(app.chest, 0, "in range stays open");

        // Out of range: the force close (`:31078-31085`, sound 11).
        {
            let p = app.player.as_mut().unwrap();
            p.x = 480.0;
            p.y = 150.0;
        }
        tick_open(&mut app);
        assert_eq!(app.chest, -1, "out of range force-closes");
        assert_eq!(force_close(&mut app), 11, "the close sound is 11");

        // Re-open, then kill the tile: the dead-tile arm (:31086-31090).
        assert!(try_open_chest_at_cursor(&mut app, click(10, 10)));
        assert_eq!(app.chest, 0);
        {
            let p = app.player.as_mut().unwrap();
            p.x = 150.0;
            p.y = 150.0;
        }
        let i = app.world.index(10, 10);
        app.world.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        tick_open(&mut app);
        assert_eq!(app.chest, -1, "a dead tile force-closes");

        // Nothing open: the tick is a no-op (`:31017`).
        tick_open(&mut app);
        assert_eq!(app.chest, -1);
    }

    /// DW-2.3 — the locked chest's click chain (`Player.cs:33142-33228`):
    /// the locked click consumes without opening (`flag14` blocks the
    /// `FindChest`), the right key unlocks and is consumed, and the chest
    /// opens on the next click.
    #[test]
    fn dw_2_3_locked_chest_click_consumes_key_unlocks_then_opens() {
        let mut w = world();
        place_chest(&mut w, 20, 10, 21, 72); // style 2 — the golden key 327
        let mut app = interact_app(w);

        // Locked, no key: the click is consumed, nothing opens, no store
        // entry is created.
        assert!(try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.chest, -1);
        assert_eq!(find_chest(&app.world_chests, 20, 10), None);
        assert!(is_locked(&app.world, 20, 10), "still locked");

        // The right key: the unlock fires, the click is still consumed (no
        // open this click), and the key's stack decrements.
        app.inventory[0] = slot(327, 2);
        assert!(try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.world.tile(20, 10).unwrap().frame_x, 36, "72 - 36");
        assert_eq!(app.world.tile(21, 10).unwrap().frame_x, 54, "90 - 36");
        assert_eq!(app.inventory[0].count, 1, "the key is consumed");
        assert!(!is_locked(&app.world, 20, 10));
        assert_eq!(app.chest, -1, "the unlock consumes the click");
        assert_eq!(find_chest(&app.world_chests, 20, 10), None);

        // The next click opens (`:33223-33226` runs `FindChest` now).
        assert!(try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.chest, 0);
        assert_eq!((app.chest_x, app.chest_y), (20, 10));
    }

    /// DW-2.3 — the shadow key (329) unlocks without consuming
    /// (`flag16 = num77 != 329`, `Player.cs:33175`).
    #[test]
    fn dw_2_3_shadow_key_unlock_does_not_consume() {
        let mut w = world();
        place_chest(&mut w, 40, 10, 21, 144); // style 4 — the shadow key 329
        let mut app = interact_app(w);
        app.inventory[0] = slot(329, 5);
        assert!(try_open_chest_at_cursor(&mut app, click(40, 10)));
        assert_eq!(app.world.tile(40, 10).unwrap().frame_x, 108, "144 - 36");
        assert!(!is_locked(&app.world, 40, 10));
        assert_eq!(app.inventory[0].count, 5, "the shadow key is not consumed");
        assert_eq!(app.chest, -1, "the click is still consumed");
        // And the unlocked chest opens on the next click.
        assert!(try_open_chest_at_cursor(&mut app, click(40, 10)));
        assert_eq!(app.chest, 0);
    }

    /// DW-2.3 — the key scan covers the C# `inventory[0..=58]` array only
    /// (`Player.cs:33176`): the port's coins (50-53) are searched, the
    /// equipment rows are not, and exactly one key is consumed per click
    /// (`Chest.Unlock` answers false once the chest is unlocked).
    #[test]
    fn dw_2_3_key_search_covers_the_59_array_slots_only() {
        let mut w = world();
        place_chest(&mut w, 60, 10, 21, 72);
        let mut app = interact_app(w);
        // An equipment row (outside 0..58): not searched — the chest stays
        // locked and the key is untouched.
        app.equip.armor[0] = slot(327, 1);
        assert!(try_open_chest_at_cursor(&mut app, click(60, 10)));
        assert!(
            is_locked(&app.world, 60, 10),
            "the armor row is not scanned"
        );
        assert_eq!(app.equip.armor[0].count, 1);

        // A coin slot (C# array index 50-53): searched.
        app.equip.coins[0] = slot(327, 3);
        assert!(try_open_chest_at_cursor(&mut app, click(60, 10)));
        assert!(!is_locked(&app.world, 60, 10));
        assert_eq!(app.equip.coins[0].count, 2, "one key consumed");
    }

    /// DW-2.3 — exactly one key per click: the loop keeps scanning after a
    /// success, but the chest is unlocked, so the second matching slot
    /// survives (`Player.cs:33178`'s `Chest.Unlock` continue arm).
    #[test]
    fn dw_2_3_exactly_one_key_is_consumed_per_click() {
        let mut w = world();
        place_chest(&mut w, 20, 40, 21, 72);
        let mut app = interact_app(w);
        app.inventory[0] = slot(327, 1);
        app.inventory[1] = slot(327, 1);
        assert!(try_open_chest_at_cursor(&mut app, click(20, 40)));
        assert_eq!(app.inventory[0].count, 0, "the first key is consumed");
        assert_eq!(app.inventory[0].kind, 0, "the emptied slot is air");
        assert_eq!(app.inventory[1].count, 1, "the second is not");
    }

    /// DW-2.3 — the trash slot's mirror (`Player.cs:33190-33193`): a key
    /// consumed from slot 58 clones the post-decrement slot into
    /// `Main.mouseItem`.
    #[test]
    fn dw_2_3_trash_slot_key_mirrors_into_the_cursor_stack() {
        let mut w = world();
        place_chest(&mut w, 60, 40, 21, 72);
        let mut app = interact_app(w);
        app.equip.trash = slot(327, 2);
        assert!(try_open_chest_at_cursor(&mut app, click(60, 40)));
        assert_eq!(app.equip.trash.count, 1);
        assert_eq!(app.mouse_item.kind, 327, "`Main.mouseItem` mirrors");
        assert_eq!(app.mouse_item.count, 1);
    }

    /// DW-2.4 — the key-id derivation at the frame boundaries
    /// (`Player.cs:33145-33172`): 327 default, 329 in 144-178, 1533-1537
    /// in 828-1006 (style = the halved `frameX / 18`), 4714 for tile 467
    /// style 13.
    #[test]
    fn dw_2_4_key_id_derivation_at_frame_boundaries() {
        let mut w = world();
        let cases: [(i32, u16, i16, u16); 13] = [
            (4, 21, 143, 327),
            (6, 21, 144, 329),
            (8, 21, 177, 329),
            (10, 21, 178, 329),
            (12, 21, 179, 327),
            (14, 21, 827, 327),
            (16, 21, 828, 1533),
            (18, 21, 1006, 1537),
            (20, 21, 1007, 327),
            (22, 467, 467, 327),  // style 12
            (24, 467, 468, 4714), // style 13
            (26, 467, 503, 4714), // style 13's last frame
            (28, 467, 504, 327),  // style 14
        ];
        for &(x, kind, frame_x, key) in &cases {
            set_tile(&mut w, x, 90, kind, frame_x);
            assert_eq!(
                locked_key_id(&w, x, 90),
                key,
                "tile {kind} frameX {frame_x} derives key {key}"
            );
        }
    }

    /// DW-2.5 — `Chest.Unlock`/`Chest.Lock`'s frame tables
    /// (`Chest.cs:346-496`): −36/+36 for the plain styles, ∓180 for the
    /// Plantera-gated ones, the 2×2 shifts as a whole, the unsupported
    /// styles refuse, and the gate reads the flag parameter (the world flag
    /// has no home yet — the recorded fallback).
    #[test]
    fn dw_2_5_lock_unlock_shift_frames_per_style_with_the_plant_boss_gate() {
        let mut w = world();
        // Unlock: (style, origin frameX, shift, gated). The plain rows run
        // ungated; the gated rows are probed both ways below.
        let unlock_cases = [
            (10, 2i16, 72, 36),
            (20, 4, 144, 36),
            (30, 23, 828, 180),
            (40, 36, 1296, 36),
            (50, 38, 1368, 36),
            (60, 40, 1440, 36),
        ];
        for (x, style, frame_x, shift) in unlock_cases {
            place_chest(&mut w, x, 10, 21, frame_x);
            if style == 23 {
                // The Plantera gate (`Chest.cs:376-379`): refused, frames
                // unchanged.
                assert!(!unlock_chest(&mut w, x, 10, false), "style 23 gates");
                assert_eq!(w.tile(x, 10).unwrap().frame_x, frame_x);
            }
            assert!(unlock_chest(&mut w, x, 10, true), "style {style} unlocks");
            assert_eq!(w.tile(x, 10).unwrap().frame_x, frame_x - shift);
            assert_eq!(
                w.tile(x + 1, 10).unwrap().frame_x,
                frame_x + 18 - shift,
                "the right column shifts too"
            );
            assert!(!is_locked(&w, x, 10));
        }
        // An unsupported style refuses (the `default` arm, `Chest.cs:390-391`).
        place_chest(&mut w, 70, 10, 21, 0);
        assert!(!unlock_chest(&mut w, 70, 10, true));

        // Lock: the mirror table (`Chest.cs:440-467`).
        let lock_cases = [
            (10, 1i16, 36, 36),
            (20, 3, 108, 36),
            (30, 18, 648, 180),
            (40, 35, 1260, 36),
            (50, 37, 1332, 36),
            (60, 39, 1404, 36),
        ];
        for (x, style, frame_x, shift) in lock_cases {
            place_chest(&mut w, x, 40, 21, frame_x);
            if style == 18 {
                assert!(!lock_chest(&mut w, x, 40, false), "style 18 gates");
                assert_eq!(w.tile(x, 40).unwrap().frame_x, frame_x);
            }
            assert!(lock_chest(&mut w, x, 40, true), "style {style} locks");
            assert_eq!(w.tile(x, 40).unwrap().frame_x, frame_x + shift);
            assert!(is_locked(&w, x, 40), "the locked frame decodes locked");
        }
        place_chest(&mut w, 70, 40, 21, 72); // already locked — unsupported
        assert!(!lock_chest(&mut w, 70, 40, true));

        // Tile 467: unlock style 13 → 12, lock style 12 → 13, both gated
        // (`Chest.cs:394-406`/:469-480).
        place_chest(&mut w, 10, 70, 467, 468);
        assert!(!unlock_chest(&mut w, 10, 70, false), "467-13 gates");
        assert!(unlock_chest(&mut w, 10, 70, true));
        assert_eq!(w.tile(10, 70).unwrap().frame_x, 432, "468 - 36");
        assert!(lock_chest(&mut w, 10, 70, true));
        assert_eq!(w.tile(10, 70).unwrap().frame_x, 468, "432 + 36");

        // The round trip restores the frame.
        place_chest(&mut w, 40, 70, 21, 36);
        assert!(lock_chest(&mut w, 40, 70, true));
        assert_eq!(w.tile(40, 70).unwrap().frame_x, 72);
        assert!(unlock_chest(&mut w, 40, 70, true));
        assert_eq!(w.tile(40, 70).unwrap().frame_x, 36);
    }

    /// The origin decode (`Player.cs:33062-33066`): a click on any of the
    /// four cells resolves the same 2×2 origin — here the bottom-right.
    #[test]
    fn a_chest_click_on_any_of_the_four_cells_opens_the_same_chest() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(11, 19)));
        assert_eq!(app.chest, 0);
        assert_eq!(
            (app.chest_x, app.chest_y),
            (10, 18),
            "the bottom-right cell"
        );
        // The same chest's origin-cell click closes it (the same index).
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert_eq!(app.chest, -1);
    }

    /// `TileID.Sets.BasicChest` (`TileID.cs:570`) gates the open path: tile
    /// 341 (the craft scan's frozen second entry) is not a `BasicChest`, and
    /// an inactive chest tile never reaches the interaction chain
    /// (`:32035-32041`).
    #[test]
    fn non_basic_chest_and_inactive_tiles_are_not_openable() {
        let mut w = world();
        set_tile(&mut w, 10, 10, 341, 0);
        set_tile(&mut w, 20, 10, 21, 0);
        let mut app = interact_app(w);
        assert!(!try_open_chest_at_cursor(&mut app, click(10, 10)));
        assert_eq!(app.chest, -1);
        let i = app.world.index(20, 10);
        app.world.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        assert!(!try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.chest, -1);
    }

    /// `Chest.Unlock`/`Chest.Lock` refuse a 2×2 with a missing tile
    /// (`Chest.cs:348-351`/:430-433) — the C# null-tile check maps to the
    /// port's bounds check (`world.tile` is `None` out of bounds, the
    /// phase-1 mapping).
    #[test]
    fn unlock_and_lock_refuse_a_missing_2x2_tile() {
        let mut w = world();
        // The right column of this 2×2 hangs off the world edge.
        set_chest_tile(&mut w, 119, 10, 21, 72, 0);
        set_chest_tile(&mut w, 119, 11, 21, 72, 18);
        assert!(!unlock_chest(&mut w, 119, 10, true));
        assert!(!lock_chest(&mut w, 119, 10, true));
        assert_eq!(w.tile(119, 10).unwrap().frame_x, 72, "unchanged");
    }

    // --- Phase 4: the slot-interaction helpers -------------------------------

    /// DW-4.4 — `ItemSlot.canFavoriteAt` (`ItemSlot.cs:386-394`): only the
    /// inventory contexts 0/1/2/32 can favorite — a chest slot (3) can never
    /// carry the flag, so the cursor-3 favorite arm never fires on it.
    #[test]
    fn dw_4_4_can_favorite_at_excludes_chest_context() {
        assert!(can_favorite_at(0));
        assert!(can_favorite_at(1));
        assert!(can_favorite_at(2));
        assert!(can_favorite_at(32));
        assert!(!can_favorite_at(3), "chest slots cannot be favorited");
        assert!(!can_favorite_at(5));
        assert!(!can_favorite_at(13));
    }

    /// DW-4.4 — `ItemSlot.retainFavoriteOnMoveAt` (`ItemSlot.cs:390-394`):
    /// moving out of a chest (3) clears the favorite (`MouseItemSwap`,
    /// `ItemSlot.cs:1255-1263`). The port's `ItemSlot` carries no favorite
    /// field yet (item.rs:52-57), so the clear itself is a no-op with the
    /// citation — the tables are the ported rule, M7-UI-2 consumes them.
    #[test]
    fn dw_4_4_retain_favorite_on_move_excludes_chest_context() {
        for ctx in [0, 1, 2, 32, 5] {
            assert!(retain_favorite_on_move_at(ctx), "ctx {ctx} retains");
        }
        assert!(
            !retain_favorite_on_move_at(3),
            "a chest move clears the favorite"
        );
        assert!(!retain_favorite_on_move_at(4));
    }

    /// The implicit-air slot helpers (decision #4): reads past the stored
    /// length answer air, writes extend with airs first and normalize
    /// half-air values to air, and a removed/missing entry refuses.
    #[test]
    fn chest_slot_read_write_with_implicit_air() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        assert_eq!(
            chest_slot(&chests, idx, 5),
            Some((0, 0)),
            "beyond len is air"
        );
        assert!(set_chest_slot(&mut chests, idx, 35, (9, 3)));
        assert_eq!(chests[idx].slots.len(), 36, "extended with airs first");
        assert_eq!(chests[idx].slots[34], (0, 0), "the extension filled air");
        assert_eq!(chest_slot(&chests, idx, 35), Some((9, 3)));
        // Half-air values normalize to air (the `SetDefaults(0)` writes).
        assert!(set_chest_slot(&mut chests, idx, 35, (9, 0)));
        assert_eq!(chest_slot(&chests, idx, 35), Some((0, 0)));
        assert!(set_chest_slot(&mut chests, idx, 36, (0, 7)));
        assert_eq!(chest_slot(&chests, idx, 36), Some((0, 0)));
        assert!(chests[idx].is_empty(), "the writes left pure air");
        // A removed or out-of-range entry refuses both directions.
        remove_chest(&mut chests, idx);
        assert_eq!(chest_slot(&chests, idx, 0), None);
        assert!(!set_chest_slot(&mut chests, idx, 0, (9, 1)));
        assert_eq!(chest_slot(&chests, 9, 0), None);
    }
}
