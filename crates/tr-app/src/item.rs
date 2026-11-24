//! World item drops and the inventory slot record, the first pass of `Terraria.Item`.
//!
//! Blocks broken by the player spawn a small physics entity instead of teleporting into
//! the inventory. Drops fall with gravity, bounce off solid tiles, and are magnetised to
//! the player once they are close and old enough, then added to the first matching or
//! empty hotbar slot. Rendering uses the tile's plain 16x16 frame (we do not extract the
//! separate `Item_*` sprites yet) tinted by the light map.

use glam::Vec2;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::{World, TILE_PX};

use crate::assets;
use crate::registry;

// --- Inventory slots (`Main.inventory` entries) -----------------------------

pub(crate) const HOTBAR: usize = 10;
/// `Player.inventory` length: hotbar plus the four storage rows of the open grid.
pub(crate) const INV: usize = 50;
/// `Player.SetItemAnimation` fallback for kinds outside [`registry::ITEMS`]:
/// 15 ticks — the `SetDefaults1` block default (`Item.cs:1405-1415`; the raw
/// `ResetStats` default is 100, `Item.cs:49020-49022`, which no live kind
/// reaches).
pub(crate) const DEFAULT_USE_ANIMATION_TICKS: u16 = 15;

/// The held item's swing duration in seconds — `useAnimation` ticks at the
/// 60 Hz simulation rate (`Player.ApplyItemAnimation`, `Player.cs:4449-4451`:
/// `itemAnimation = itemAnimationMax = useAnimation`, one tick per frame).
pub(crate) fn use_animation_secs(kind: u16) -> f32 {
    let ticks = registry::get(kind)
        .map(|d| d.use_animation as u16)
        .filter(|t| *t > 0)
        .unwrap_or(DEFAULT_USE_ANIMATION_TICKS);
    ticks as f32 / 60.0
}

/// The prefix-aware variant: `Item.Prefix` scales `useAnimation` (`spd`),
/// and the swing paces off the prefixed value (`Player.cs:4449-4451` reads
/// the instance). Falls back to the raw kind when the prefix is absent.
pub(crate) fn use_animation_secs_of(kind: u16, prefix: u16) -> f32 {
    if prefix == 0 {
        return use_animation_secs(kind);
    }
    crate::prefix::prefixed_stats(kind, prefix)
        .map(|s| s.use_animation)
        .filter(|t| *t > 0)
        .map(|t| t as f32 / 60.0)
        .unwrap_or_else(|| use_animation_secs(kind))
}

/// `Player.ApplyItemTime(sItem)` (`Player.cs:4416-4419` →
/// `SetItemTime(sItem.useTime)`, `:4406-4413`): the kind's `useTime` in
/// seconds at the 60 Hz rate — the ACTION cadence re-arm the mid-swing use
/// arms (the boss spawners / event items, `Player.cs:43759`/`:43925`) read.
/// A kind with no positive `useTime` falls back to its animation length
/// (the port's swing-paced stand-in, same as [`use_animation_secs`]).
pub(crate) fn use_time_secs(kind: u16) -> f32 {
    registry::get(kind)
        .map(|d| d.use_time)
        .filter(|t| *t > 0)
        .map(|t| t as f32 / 60.0)
        .unwrap_or_else(|| use_animation_secs(kind))
}

/// `Item.useStyle` of the kind (`Item.SetDefaults`; the
/// `ItemUseStyleID.cs` constants — 1 Swing, 2 EatFood, 3 Thrust, 4 HoldUp,
/// 5 Shoot, 6 DrinkLong, 7 DrinkOld, 8 GolfPlay, 9 DrinkLiquid, 10 Hidden,
/// 11 MowTheLawn, 12 Guitar, 13 Rapier, 14 RaiseLamp, 15 HoldOrb, 16
/// PlaySound), 0 when unregistered.
pub(crate) fn use_style(kind: u16) -> u8 {
    registry::get(kind).map_or(0, |d| d.use_style)
}

/// `Item.holdStyle` of the kind (`Item.SetDefaults`), 0 when unregistered.
pub(crate) fn hold_style(kind: u16) -> u8 {
    registry::get(kind).map_or(0, |d| d.hold_style)
}

// S1-P-i43's not_ammo/fits_ammo_slot/can_fill_empty_ammo_slot copies were
// deduped at gate V4 — the S1-P-i47 canonical block below (the pickup
// predicates section) carries the same Item.cs:1369-1385 seam.

/// `Player.ItemCheck_StartActualUse`'s sound gate (`Player.cs:51681-51682`):
/// `sItem.UseSound != null && !ItemID.Sets.SkipsInitialUseSound[sItem.type]`
/// — the set is `{2350, 4870}` (`ItemID.cs:408`, the Recall Potion / Potion
/// of Return: their cue plays at the teleport, not the use start). Returns
/// the kind's `SOUND_STYLES` index + `useSoundPitch`
/// (`Player.cs:51696`'s `PlaySound(sItem.UseSound, base.Center,
/// sItem.useSoundPitch)`), `None` when the use start is silent.
pub(crate) fn initial_use_sound(kind: u16) -> Option<(u16, f32)> {
    if matches!(kind, 2350 | 4870) {
        return None;
    }
    let d = registry::get(kind)?;
    Some((d.use_sound?, d.use_sound_pitch))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ItemSlot {
    pub(crate) kind: u16,
    pub(crate) name: &'static str,
    pub(crate) count: u32,
    /// The rolled prefix (`Item.prefix`, `Item.cs:504`'s field); 0 = none.
    /// Two slots stack only when both match (`Item.CanStack`,
    /// `Item.cs:49865-49871`: same type AND same prefix).
    pub(crate) prefix: u16,
}

/// `Item.CommonMaxStack` (`Item.cs:49966`) �?" the default stack limit applied
/// by `ResetStats`/`SetDefaults` before id-specific overrides.
pub(crate) const COMMON_MAX_STACK: u32 = 9999;
/// Compatibility alias; prefer [`COMMON_MAX_STACK`].
pub(crate) const MAX_STACK: u32 = COMMON_MAX_STACK;

pub(crate) const EMPTY_SLOT: ItemSlot = ItemSlot {
    kind: 0,
    name: "",
    count: 0,
    prefix: 0,
};

impl ItemSlot {
    /// `Item.Clone` (`Item.cs:49837-49840`, S1-F-10): a memberwise copy of the
    /// item instance snapshot.
    pub(crate) fn clone_item(self) -> Self {
        self
    }

    /// `Item.Refresh` (`Item.cs:49945-49961`, S1-F-10, simplified): re-apply the
    /// kind's default name and clamp the stack to the current `maxStack`. The
    /// port has no variant system, so `onlyIfVariantChanged` is ignored and the
    /// snapshot is refreshed unconditionally when the slot is not air.
    pub(crate) fn refresh(&mut self, _only_if_variant_changed: bool) {
        if self.kind == 0 || self.count == 0 {
            *self = EMPTY_SLOT;
            return;
        }
        self.name = registry::name(self.kind);
        let max = registry::max_stack(self.kind);
        if self.count > max {
            self.count = max;
        }
    }
}

/// `Player.GetItem`'s grid passes (`Player.cs:39353-39394`, S1-P-i47): the
/// occupied-stack merge scans FORWARD 0..49 (`GetItem_FillIntoOccupiedSlot`
/// `:39487` — `type > 0 && stack < maxStack && CanStack`, so a FULL same-kind
/// stack never answers), then the empty-slot passes: the hotbar forward
/// 0..9 first for usable kinds (`!isACoin && useStyle != 0`, `:39365-39374`),
/// then the whole grid REVERSE 49..0 for the non-favorited item
/// (`:39387-39394`; a world drop / craft result is never favorited, so the
/// forward favorited arm `:39375-39384` is unreachable here). The stacking
/// rule is `Item.CanStack` (`Item.cs:49865-49871`): same kind AND same prefix
/// — a prefixed drop never merges into an unprefixed stack. The ammo
/// pre-pass and the favorite guard live in [`deposit_pickup`]; this helper
/// is the grid-only shape the UI transfer callers share.
pub(crate) fn slot_for_pickup(
    inventory: &mut [ItemSlot; INV],
    kind: u16,
    prefix: u16,
) -> Option<usize> {
    // The occupied merge pass, forward (`Player.cs:39353-39364`).
    for i in 0..INV {
        let s = &inventory[i];
        if s.count > 0
            && s.count < registry::max_stack(kind)
            && s.kind == kind
            && s.prefix == prefix
        {
            return Some(i);
        }
    }
    let init = |inventory: &mut [ItemSlot; INV], i: usize| {
        inventory[i] = ItemSlot {
            kind,
            name: registry::name(kind),
            count: 0,
            prefix,
        };
        Some(i)
    };
    // The usable-kind hotbar arm (`:39365-39374`).
    if use_style(kind) != 0 {
        for i in 0..HOTBAR {
            if inventory[i].count == 0 {
                return init(inventory, i);
            }
        }
    }
    // The non-favorited reverse scan (`:39387-39394`).
    for i in (0..INV).rev() {
        if inventory[i].count == 0 {
            return init(inventory, i);
        }
    }
    None
}

// --- The pickup predicates (S1-P-i47) ----------------------------------------

/// `ItemID.Sets.IsAPickup` (`ItemID.cs:493`): the hearts/stars/nebula/mana
/// pickups — `Player.ItemSpace` always answers `CanTakeItem` for them
/// (`Player.cs:39155-39158`). Their consume-on-touch effects are i52's.
pub(crate) fn is_a_pickup(kind: u16) -> bool {
    matches!(
        kind,
        58 | 184 | 1734 | 1735 | 1867 | 1868 | 3453 | 3454 | 3455 | 4143
    )
}

/// `Item.notAmmo` (`Item.cs`'s `SetDefaults` cases): the acorn 27 (`:1709`),
/// the coins 71-74 (`:2225-2286`), the sand family 169/370/408/1246
/// (`:3464`/`:5721`/`:6146`/`:15462`) and the ale 353 (`:48384`) carry
/// `ammo > 0` but refuse the ammo slots. No other kind sets the flag.
pub(crate) fn not_ammo(kind: u16) -> bool {
    matches!(kind, 27 | 71..=74 | 169 | 353 | 370 | 408 | 1246)
}

/// `Item.PaintOrCoating` (`Item.cs:353-362`): `paint > 0 || paintCoating > 0`.
pub(crate) fn paint_or_coating(kind: u16) -> bool {
    registry::get(kind).is_some_and(|d| d.paint > 0 || d.paint_coating > 0)
}

/// `Player.FindPaintOrCoating` (`Player.cs:41927-41944`): the paint the held
/// brush/roller applies (and the cursor preview shows, `Main.TryGetAmmo`
/// `Main.cs:45134`). The FIRST stocked `PaintOrCoating` in the ammo slots
/// 54..57 wins; else the first over the whole 0..58 carry range — grid
/// 0..49, coins 50..53, then the ammo slots again. `None` = nothing
/// selected. The port's split arrays (`App.inventory` + `EquipSlots`) keep
/// the C# index order (`inv.rs`'s slot-numbering note).
pub(crate) fn find_paint_or_coating(
    inventory: &[ItemSlot; INV],
    equip: &crate::inv::EquipSlots,
) -> Option<ItemSlot> {
    // `:41929-41935` — `for (int i = 54; i < 58; i++)`.
    for s in equip.ammo.iter() {
        if s.count > 0 && paint_or_coating(s.kind) {
            return Some(*s);
        }
    }
    // `:41936-41942` — `for (int j = 0; j < 58; j++)`.
    for s in inventory
        .iter()
        .chain(equip.coins.iter())
        .chain(equip.ammo.iter())
    {
        if s.count > 0 && paint_or_coating(s.kind) {
            return Some(*s);
        }
    }
    None
}

/// `Main.TryGetAmmo`'s brush/roller arm (`Main.cs:45117-45136`), the paint
/// preview riding the cursor beside the held tool icon
/// (`DrawInterface_40_InteractItemIcon` `:45093-45099`): the brush
/// (1071/1543) offsets the preview by (22, 22), the roller (1072/1544) by
/// (28, 28); the previewed item is `FindPaintOrCoating`'s pick. The C#
/// out-defaults are `Color.White` / scale `0.8f` (`:45120-45121`); the draw
/// site re-reads them through `ammoItem.GetAlpha(ammoColor)`
/// (`:45096`; the spectre tools themselves ride the gFade alpha arm
/// `Item.cs:49246-49252` — `Main.gFade` unported — while a previewed paint
/// takes `GetAlpha`'s default arm, `Item.cs:49324-49339`, white at alpha 0).
/// `None` for every other kind — the `FlexibleTileWand` arm (`:45137-45142`)
/// is unported (no wand placement cycle).
pub(crate) fn paint_tool_preview(
    held_kind: u16,
    inventory: &[ItemSlot; INV],
    equip: &crate::inv::EquipSlots,
) -> Option<(ItemSlot, [f32; 2])> {
    let offset = match held_kind {
        1071 | 1543 => [22.0, 22.0],
        1072 | 1544 => [28.0, 28.0],
        _ => return None,
    };
    Some((find_paint_or_coating(inventory, equip)?, offset))
}

/// `Item.FitsAmmoSlot` (`Item.cs:1369-1376`): the `GetItem` ammo pre-pass
/// gate (`Player.cs:39345`). A kind with no ammo/bait value (and not the
/// two special ids 530/849, not painted) fits only when it is the ale 353.
pub(crate) fn fits_ammo_slot(kind: u16) -> bool {
    let d = registry::get(kind);
    let ammo = d.map_or(0, |d| d.ammo);
    let bait = d.map_or(0, |d| d.bait);
    if (kind != 0 && ammo <= 0 && bait == 0 && kind != 530 && kind != 849 && !paint_or_coating(kind))
        || not_ammo(kind)
    {
        return kind == 353;
    }
    true
}

/// `Item.CanFillEmptyAmmoSlot` (`Item.cs:1378-1385`): the bait, the painted
/// items and the nine listed kinds never claim an EMPTY ammo slot; the rest
/// fit unless `notAmmo`.
pub(crate) fn can_fill_empty_ammo_slot(kind: u16) -> bool {
    let bait = registry::get(kind).map_or(0, |d| d.bait);
    if bait == 0
        && !paint_or_coating(kind)
        && !matches!(kind, 353 | 849 | 169 | 75 | 23 | 408 | 370 | 1246)
    {
        return !not_ammo(kind);
    }
    false
}

/// `Item.CanBeEquipped` (`Item.cs:1159-1174`): an armour/accessory/hook/
/// mount/pet kind. `Main.projHook` is the aiStyle-7 set
/// (`projectile_id::is_hook_projectile`), `Main.vanityPet`/`lightPet` the
/// buff tables (`buffs.rs`).
fn can_be_equipped(kind: u16) -> bool {
    let Some(d) = registry::get(kind) else {
        return false;
    };
    let hook = d
        .projectile
        .is_some_and(|p| crate::projectile_id::is_hook_projectile(p as i32));
    if d.head_slot < 0
        && d.body_slot < 0
        && d.leg_slot < 0
        && !d.accessory
        && !hook
        && d.mount_type < 0
    {
        if d.buff_type > 0 {
            if !crate::buffs::is_vanity_pet(d.buff_type) {
                return crate::buffs::is_light_pet(d.buff_type);
            }
            return true;
        }
        return false;
    }
    true
}

/// `Item.OnlyNeedOneInInventory` (`Item.cs:1176-1192`) with the
/// `ItemID.Sets.OnlyNeedOneInInventoryOverride` table (`ItemID.cs:1387-1445`):
/// the large gems/capture items force true, the buckets/doll/sponge force
/// false; otherwise a non-consumable is one-per-inventory unless it is a
/// `useStyle 0` non-equippable material.
pub(crate) fn only_need_one_in_inventory(kind: u16) -> bool {
    // The `true` overrides (`ItemID.cs:1389-1432`).
    if matches!(
        kind,
        1522..=1527
            | 3643
            | 3103
            | 3104
            | 856
            | 1299
            | 329
            | 4346
            | 5391
            | 4767
            | 5453
            | 5309
            | 5454
            | 5323
            | 5455
            | 6190
            | 6195
    ) {
        return true;
    }
    // The `false` overrides (`ItemID.cs:1433-1444`).
    if matches!(kind, 205 | 206 | 207 | 1128 | 267 | 849) {
        return false;
    }
    let Some(d) = registry::get(kind) else {
        return false;
    };
    if !d.consumable {
        if d.use_style == 0 {
            return can_be_equipped(kind);
        }
        return true;
    }
    false
}

/// `Item.CanPassivelyStackInWorld` (`Item.cs:49573-49593`) with the
/// `CanPassivelyStackInWorldOverride` table (`ItemID.cs:1447-1471`): the
/// gate on `WorldItem.TryCombiningIntoNearbyItems` (`WorldItem.cs:159`).
pub(crate) fn can_passively_stack_in_world(kind: u16) -> bool {
    let Some(d) = registry::get(kind) else {
        return false;
    };
    if d.max_stack == 1 {
        return false;
    }
    // The overrides: coins/fallen star never, medals/buckets/sponge always.
    if matches!(kind, 71..=74 | 75) {
        return false;
    }
    if matches!(kind, 3817 | 205 | 206 | 207 | 1128 | 530) {
        return true;
    }
    if only_need_one_in_inventory(kind) {
        return false;
    }
    // `createTile < 0 && createWall <= 0` — the port's `Option` walls map
    // `None` to the C# stock -1 and `Some(0)` to the C# 0 (both `<= 0`).
    let no_tile = d.tile.is_none();
    let no_wall = d.wall.map_or(true, |w| w == 0);
    if no_tile && no_wall && (d.ammo <= 0 || not_ammo(kind)) && !d.consumable && d.dye <= 0 && !paint_or_coating(kind)
    {
        return d.material;
    }
    true
}

/// `Player.ItemSpaceStatus` (`Player.cs:245-263`): the pull gate's answer —
/// whether the item can be taken at all, and whether its destination is the
/// void vault (the faster `PullItem_ToVoidVault` arm, `:34957-34960`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ItemSpaceStatus {
    pub can_take: bool,
    pub to_void_vault: bool,
}

/// `Player.CanItemSlotAcceptPickup` (`Player.cs:39220-39235`): an empty slot
/// always accepts; a favorited one-per-inventory slot refuses; otherwise a
/// partial `Item.CanStack` match accepts. `favorite` reads the vanilla slot
/// index space (grid 0..50, ammo 54..58 — `App::is_favorite`).
fn can_item_slot_accept_pickup(
    slot: &ItemSlot,
    kind: u16,
    prefix: u16,
    favorited: bool,
) -> bool {
    if slot.kind == 0 {
        return true;
    }
    if favorited && only_need_one_in_inventory(slot.kind) {
        return false;
    }
    slot.count < registry::max_stack(slot.kind) && slot.kind == kind && slot.prefix == prefix
}

/// `Player.ItemSpace` (`Player.cs:39153-39197`, S1-P-i47): the pickup pull
/// gate. `IsAPickup` kinds always fit; the `uniqueStack && HasItem` refusal
/// (`:39159-39162`) is unreachable in vanilla — `uniqueStack` is only ever
/// written `false` (`Item.cs:49046`, no `true` site in the decompile); then
/// the grid scan (coins would scan 0..54 — the coin routing is i50's, the
/// port's coins live in `App.coins`), the `ammo > 0 && !notAmmo` ammo-slot
/// arm (`CanGoIntoAmmoOnPickup`, `:39237-39244`), the ungated same-kind ammo
/// merge scan (`:39185-39191`), and the Cofveve void-vault scan
/// (`ItemSpaceForCofveve`, `:39199-39218`). `vault` is `Some` only when
/// `IsVoidVaultEnabled`.
pub(crate) fn item_space(
    inventory: &[ItemSlot; INV],
    ammo: &[ItemSlot; crate::inv::AMMO],
    kind: u16,
    prefix: u16,
    favorite: impl Fn(usize) -> bool,
    vault: Option<&crate::bank::BankStore>,
) -> ItemSpaceStatus {
    if is_a_pickup(kind) {
        return ItemSpaceStatus { can_take: true, to_void_vault: false };
    }
    for i in 0..INV {
        if can_item_slot_accept_pickup(&inventory[i], kind, prefix, favorite(i)) {
            return ItemSpaceStatus { can_take: true, to_void_vault: false };
        }
    }
    let d = registry::get(kind);
    if d.is_some_and(|d| d.ammo > 0) && !not_ammo(kind) {
        for j in 0..crate::inv::AMMO {
            // `CanGoIntoAmmoOnPickup`: an empty ammo slot needs
            // `CanFillEmptyAmmoSlot`; the rest ride the normal accept.
            let slot = &ammo[j];
            let ok = if slot.count == 0 {
                can_fill_empty_ammo_slot(kind)
            } else {
                can_item_slot_accept_pickup(slot, kind, prefix, favorite(crate::inv::ammo_slot_index(j)))
            };
            if ok {
                return ItemSpaceStatus { can_take: true, to_void_vault: false };
            }
        }
    }
    // The ungated merge scan over the ammo rows (`:39185-39191`).
    for slot in ammo.iter() {
        if slot.count > 0
            && slot.count < registry::max_stack(slot.kind)
            && slot.kind == kind
            && slot.prefix == prefix
        {
            return ItemSpaceStatus { can_take: true, to_void_vault: false };
        }
    }
    // `ItemSpaceForCofveve` (`:39199-39218`).
    if let Some(vault) = vault {
        if crate::bank::can_void_vault_accept(kind) {
            for slot in vault.iter() {
                if can_item_slot_accept_pickup(slot, kind, prefix, false) {
                    return ItemSpaceStatus { can_take: true, to_void_vault: true };
                }
            }
        }
    }
    ItemSpaceStatus { can_take: false, to_void_vault: false }
}

/// `Player.GetItem`'s deposit walk for a NON-COIN world pickup
/// (`Player.cs:39330-39394`, S1-P-i47): the `FitsAmmoSlot` → `FillAmmo`
/// pre-pass (`:39345-39352`, `:39264-39328` — partial ammo merges forward,
/// then the first empty ammo slot for `CanFillEmptyAmmoSlot` kinds), the
/// occupied grid merge forward 0..49 with the favorite guard, the
/// usable-kind empty hotbar arm, then the empty grid REVERSE scan.
/// `on_fill(moved)` fires once per accepted slot — the C#'s per-site
/// `PlaySound(7)` + `NotifyItemPickup` (`:39491`/`:39505`/`:39559`/`:39571`,
/// `FillAmmo` `:39274`/`:39321`). Returns the remainder the caller keeps in
/// the world (`PickupItem`'s `ReplaceWith`, `:35103`) or routes to the void
/// vault — the vault is `GetItem`'s LAST resort (`:39395`), the caller's
/// step after this walk. The coin arm (`num2 = -4`, `:39340-39343`) is
/// i50's; `DoCoins` (`:39246-39262`) rides with it.
pub(crate) fn deposit_pickup(
    inventory: &mut [ItemSlot; INV],
    ammo: &mut [ItemSlot; crate::inv::AMMO],
    kind: u16,
    prefix: u16,
    count: u32,
    favorite: impl Fn(usize) -> bool,
    on_fill: &mut dyn FnMut(u32),
) -> u32 {
    let mut left = count;
    if left == 0 || kind == 0 {
        return 0;
    }
    let max = registry::max_stack(kind);
    // `FillAmmo` (`Player.cs:39264-39328`).
    if fits_ammo_slot(kind) {
        for j in 0..crate::inv::AMMO {
            let slot = &mut ammo[j];
            if slot.count == 0 || slot.count >= max || slot.kind != kind || slot.prefix != prefix {
                continue; // `:39268`
            }
            let moved = left.min(max - slot.count);
            slot.count += moved;
            left -= moved;
            on_fill(moved); // `:39274`/`:39293` — one chime per merge
            if left == 0 {
                return 0;
            }
        }
        if can_fill_empty_ammo_slot(kind) {
            for slot in ammo.iter_mut() {
                if slot.count == 0 {
                    *slot = ItemSlot { kind, name: registry::name(kind), count: left, prefix };
                    on_fill(left); // `:39321`
                    return 0;
                }
            }
        }
    }
    // The occupied grid pass, forward (`:39353-39364` + `:39485-39524` —
    // `type > 0 && stack < maxStack && CanItemSlotAcceptPickup`).
    for i in 0..INV {
        if left == 0 {
            return 0;
        }
        if inventory[i].kind == 0
            || !can_item_slot_accept_pickup(&inventory[i], kind, prefix, favorite(i))
        {
            continue;
        }
        let room = max.saturating_sub(inventory[i].count);
        let moved = left.min(room);
        inventory[i].count += moved;
        left -= moved;
        on_fill(moved);
    }
    if left == 0 {
        return 0;
    }
    // The usable-kind empty hotbar arm (`:39365-39374`) —
    // `GetItem_FillEmptyInventorySlot` gates on `type != 0` (`:39553`).
    if use_style(kind) != 0 {
        for i in 0..HOTBAR {
            if inventory[i].kind == 0 {
                inventory[i] = ItemSlot { kind, name: registry::name(kind), count: left, prefix };
                on_fill(left);
                return 0;
            }
        }
    }
    // The non-favorited empty grid REVERSE scan (`:39387-39394`) — a world
    // drop is never favorited, so the forward arm `:39375-39384` is out.
    for i in (0..INV).rev() {
        if inventory[i].kind == 0 {
            inventory[i] = ItemSlot { kind, name: registry::name(kind), count: left, prefix };
            on_fill(left);
            return 0;
        }
    }
    left
}

/// `WorldItem.width = height = 16` (`WorldItem.cs:138`, the ctor's
/// `base.width = (base.height = 16)`) — the drop's hitbox side. Keep-one
/// 144 (the e19 round-3 fold, `e2003946b`): the old 14 px stand-in is
/// retired; every probe (grab rect, WetCollision, LavaCollision, conveyor,
/// TileCollision, the repulsion span, the splash width+12) reads the C#
/// 16. The DRAW quad keeps the 14 px visual centred in the box
/// `base.width = (base.height = 16)`) — the drop's hitbox side. W4-e19 round
/// 3: the old 14 px stand-in is retired; every probe (grab rect, WetCollision,
/// LavaCollision, conveyor, TileCollision, the repulsion span) reads the C#
/// 16. The DRAW quad keeps the 14 px visual stand-in centred in the box
/// ([`ItemDrop::draw`]'s `pos + 1`, `SIZE - 2` — pixels unchanged).
pub const SIZE: f32 = 16.0;
/// `Player.defaultItemGrabRange = 42` (`Player.cs:57121`), the base of
/// `GetItemGrabRange` (`Player.cs:35130-35166`) — the px margin the grab
/// rectangle expands the player's hitbox by (`:34926`).
pub(crate) const DEFAULT_ITEM_GRAB_RANGE: i32 = 42;
/// `Item.coinGrabRange = 350` / `manaGrabRange = 300` / `lifeGrabRange = 250`
/// / `treasureGrabRange = 150` (`Item.cs:49962-49965`).
pub(crate) const COIN_GRAB_RANGE: i32 = 350;
pub(crate) const MANA_GRAB_RANGE: i32 = 300;
pub(crate) const LIFE_GRAB_RANGE: i32 = 250;
pub(crate) const TREASURE_GRAB_RANGE: i32 = 150;
/// `Player.itemGrabSpeed = 0.45f` / `itemGrabSpeedMax = 4f`
/// (`Player.cs:57122-57123`) — `PullItem_Common`'s per-tick acceleration and
/// velocity cap (`:34983-35029`).
pub(crate) const ITEM_GRAB_SPEED: f32 = 0.45;
pub(crate) const ITEM_GRAB_SPEED_MAX: f32 = 4.0;
/// `WorldItem.DefaultGrabDelay = 100` (`WorldItem.cs:110`) — the tick count
/// a `GrabDelayForLocalPlayer`/`ForAllPlayers` spawn locks the drop for
/// (`ApplySpawnOwnership`, `:306-313`), decremented per `UpdateItem`
/// (`:441-444`). The old port's 0.35 s age gate had no C# counterpart:
/// tile-break drops spawn `ReserveForLocalPlayer`, which in netMode 0 sets
/// NO delay (`WorldItem.cs:295-305`).
pub(crate) const DEFAULT_GRAB_DELAY: i32 = 100;
/// `Player.PickupItem`'s heart family (`Player.cs:35059`): the Heart 58, the
/// Candy Apple 1734 (Halloween) and the Candy Cane 1867 (Christmas) — the
/// `Item.NewItem` seasonal remap's products (`Item.cs:49668-49714`).
pub const HEART_KINDS: [u16; 3] = [58, 1734, 1867];
/// `Player.PickupItem`'s star family (`Player.cs:35065`): the Mana Star 184,
/// the Soul Cake 1735 and the Sugar Plum 1868.
pub const STAR_KINDS: [u16; 3] = [184, 1735, 1868];
/// `ItemID.Sets.IsAPickup`'s heart/star subset (`ItemID.cs:493` — the full
/// set also holds the nebula trio 3453-3455 and the Mana Cloak star 4143,
/// unported consume arms): `Player.ItemSpace` returns `CanTakeItem: true`
/// unconditionally for them (`Player.cs:39155-39158`) — an inventory-full
/// player still grabs them, and a FULL-life/full-mana player consumes them
/// too (the heal/restore caps; only the star's PULL is gated at full mana,
/// `Player.cs:34947`).
pub fn is_heart_or_star(kind: u16) -> bool {
    HEART_KINDS.contains(&kind) || STAR_KINDS.contains(&kind)
}
/// `Player.DoCoins` (`Player.cs:39246-39262`, S1-P-i50): a coin slot at
/// exactly 100 of copper/silver/gold (71/72/73) converts to ONE of the next
/// denomination — `SetDefaults(type + 1)` resets the stack to 1 and the
/// prefix to 0 (`Item.cs:49022`/`:48968`) — then merges into the first OTHER
/// same-type slot below 54 that has room (`stack++`, the source `TurnToAir`'d,
/// the merge re-run recursively). Platinum (74) never converts (the C# guard
/// excludes it), and the merge scan never reaches the ammo slots 54-57.
/// `i` is the C# `Player.inventory` index (grid 0..50, coins 50..54, ammo
/// 54..58); `slots` is the port's split storage in that same order
/// ([`carry_slots_mut`]). The recursive call is a loop: after a merge the
/// source is air (type 0), so the C#'s continued no-break scan can only
/// match further EMPTY slots (`type 0 == type 0`, `stack 0 < maxStack`) and
/// bump their stack to 1 while the type stays 0 — `IsAir` (`Item.cs:385-395`)
/// still reads them air and every consumer treats type 0 as empty, so the
/// quirk is invisible; the port's `break` + tail-loop keeps the visible
/// state identical (the merged slot becomes the new `i`, exactly the tail
/// recursion).
pub(crate) fn do_coins(slots: &mut [&mut ItemSlot], mut i: usize) {
    loop {
        // `:39248` — `stack != 100 || (type != 71 && type != 72 && type != 73)`.
        if slots[i].count != 100 || !matches!(slots[i].kind, 71 | 72 | 73) {
            return;
        }
        // `:39252` — `SetDefaults(type + 1)`: one coin of the next
        // denomination, prefix cleared (`ResetStats`, `Item.cs:48968`).
        let upgraded = slots[i].kind + 1;
        *slots[i] = ItemSlot {
            kind: upgraded,
            name: registry::name(upgraded),
            count: 1,
            prefix: 0,
        };
        // `:39253-39261` — the `j < 54` merge scan, ascending.
        let mut merged = None;
        for j in 0..crate::item::INV + crate::inv::COINS {
            if j == i
                || slots[j].kind != upgraded
                || slots[j].count >= registry::max_stack(upgraded)
            {
                continue;
            }
            // `inventory[j].stack++`, `inventory[i].TurnToAir()`, `DoCoins(j)`.
            slots[j].count += 1;
            *slots[i] = EMPTY_SLOT;
            merged = Some(j);
            break;
        }
        match merged {
            Some(j) => i = j,
            None => return,
        }
    }
}

/// The carry slots in C# `Player.inventory` index order — grid 0..50, coin
/// rows 50..54, ammo rows 54..58 (the port's split storage; `Player.cs:1576`'s
/// single 59-slot array, the trash 58 excluded). [`do_coins`] walks this
/// view; the ammo tail is optional (the bank transfers hold no ammo rows —
/// `DoCoins`' merge scan never reads past 54 anyway).
pub(crate) fn carry_slots_mut<'a>(
    inventory: &'a mut [ItemSlot],
    coins: &'a mut [ItemSlot],
    ammo: Option<&'a mut [ItemSlot]>,
) -> Vec<&'a mut ItemSlot> {
    let mut v: Vec<&'a mut ItemSlot> = inventory.iter_mut().chain(coins.iter_mut()).collect();
    if let Some(ammo) = ammo {
        v.extend(ammo.iter_mut());
    }
    v
}

// S1-P-i50 retry note: the former `do_coins_sweep` (a post-fill conversion
// sweep) is REMOVED — its "conversions commute with the fills" claim was
// false. `Player.DoCoins` runs IMMEDIATELY per fill (`Player.cs:39503`/
// `:39519`/`:39569`): a conversion can merge-and-air a coin row that the
// later empty pass (`:39387-39394`, descending) then refills, so a deferred
// sweep changes slot placement. Every fill site now calls [`do_coins`] on
// the slot it just filled, over the [`carry_slots_mut`] view.

/// `WorldItem.UpdateItem`'s air constants (`WorldItem.cs:364-365`):
/// `float gravity = 0.1f; float maxFallSpeed = 7f;` — the wet/shimmer/honey
/// arms override them (`:380-396`), the air case is this pair.
const GRAVITY: f32 = 0.1;
const MAX_FALL: f32 = 7.0;
/// `WorldItem.UpdateItem`'s shimmer-wet arm (`WorldItem.cs:380-385`):
/// `gravity = 0.065f; maxFallSpeed = 4f; wetVelocity = velocity * 0.375f`.
const SHIMMER_GRAVITY: f32 = 0.065;
const SHIMMER_MAX_FALL: f32 = 4.0;
/// `WorldItem.UpdateItem`'s honey-wet arm (`WorldItem.cs:386-391`):
/// `gravity = 0.05f; maxFallSpeed = 3f; wetVelocity = velocity * 0.25f`.
const HONEY_GRAVITY: f32 = 0.05;
const HONEY_MAX_FALL: f32 = 3.0;
/// `WorldItem.UpdateItem`'s water-wet arm (`WorldItem.cs:392-396`):
/// `gravity = 0.08f; maxFallSpeed = 5f` — `wetVelocity` keeps the `:379`
/// air default `* 0.5f` (the arm does not re-scale it).
const WATER_GRAVITY: f32 = 0.08;
const WATER_MAX_FALL: f32 = 5.0;
/// `WorldItem.UpdateItem`'s wet-velocity factors (`WorldItem.cs:379-390`):
/// the air default `wetVelocity = velocity * 0.5f` (`:379`), the shimmer-wet
/// override `* 0.375f` (`:384`), the honey-wet `* 0.25f` (`:390`).
/// `ApplyMovement` (`:1074-1084`) integrates a wet drop at `wetVelocity`
/// while the collision pass runs at full velocity.
const WET_STEP_AIR: f32 = 0.5;
const WET_STEP_SHIMMER: f32 = 0.375;
const WET_STEP_HONEY: f32 = 0.25;
/// `WorldItem.MoveInWorld` (`WorldItem.cs:854-860`): the horizontal friction
/// `base.velocity.X *= 0.95f` and the snap to rest below `0.1`.
const FRICTION_X: f32 = 0.95;
const FRICTION_EPSILON: f32 = 0.1;

/// `ItemID.Sets.ItemNoGravity` (`ItemID.cs:1320`):
/// `CreateBoolSet(520, 521, 575, 549, 548, 547, 3453, 3454, 3455, 3456,
/// 3457, 3458, 3459, 3580, 3581, 4143)` — the floaters (souls, the
/// nebula pickups, the mana star). `MoveInWorld`'s first arm
/// (`WorldItem.cs:774-786`) damps BOTH axes ×0.95 with the 0.1 snap
/// instead of the gravity fall.
pub fn item_no_gravity(kind: u16) -> bool {
    matches!(
        kind,
        520 | 521 | 575 | 549 | 548 | 547 | 3453 | 3454 | 3455 | 3456 | 3457 | 3458 | 3459
            | 3580 | 3581 | 4143
    )
}

/// `WorldItem.UpdateItem`'s gravity ladder (`WorldItem.cs:364-396`):
/// `(gravity, maxFallSpeed, wetVelocity_scale)` — air 0.1/7/0.5,
/// shimmerWet 0.065/4/0.375 FIRST, honeyWet 0.05/3/0.25, else wet
/// 0.08/5/0.5. `UpdateShimmer` zeroes the gravity for a shimmered drop
/// (`:483`) — the caller applies that override after this pair.
pub fn item_gravity_ladder(wet: bool, shimmer_wet: bool, honey_wet: bool) -> (f32, f32, f32) {
    if shimmer_wet {
        (0.065, 4.0, 0.375) // :380-385
    } else if honey_wet {
        (0.05, 3.0, 0.25) // :386-391
    } else if wet {
        (0.08, 5.0, 0.5) // :392-396
    } else {
        (GRAVITY, MAX_FALL, 0.5) // :364-365 + :379
    }

        }/// `ItemID.Sets.ItemNoGravity` (`ItemID.cs:1320`): `CreateBoolSet(520, 521,
/// 575, 549, 548, 547, 3453, 3454, 3455, 3456, 3457, 3458, 3459, 3580, 3581,
/// 4143)` — the souls/fragments/lunar fragments + the Mana pickup 4143 that
/// float (`MoveInWorld`'s no-gravity head, `WorldItem.cs:774-786`): both axes
/// damp ×0.95 with the 0.1 rest snap, NO gravity, NO fall cap.
/// 347 §2.3 keep-one: g15 == e19 byte-equal sets; e19's form wins (it rides
/// the wet-gravity `move_in_world` signature the merge keeps).
const ITEM_NO_GRAVITY: &[u16] = &[
    520, 521, 575, 549, 548, 547, 3453, 3454, 3455, 3456, 3457, 3458, 3459, 3580, 3581, 4143,
];
/// `ItemID.Sets.ItemNoGravity[type]` (`ItemID.cs:1320`).
    ITEM_NO_GRAVITY.contains(&kind)
}

/// The accessory flags `GetItemGrabRange`/`GrabItems` read
/// (`Player.cs:35133-35151`/`:34943-34963`). The Gold Ring's `gold_ring`
/// flag is live (S1-P-i50); the Mana Magnet family landed in S1-P-i52
/// (`player.rs` 2219/2220/2221/4000); the Treasure Magnet 5010 / Hand Of
/// Creation 5126 grants landed in W4-g15 lane 5 (`player.rs`
/// `treasure_magnet`); the Heartreach buff 105 arms `life_magnet`
/// (`player.rs:6518`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GrabFlags {
    pub gold_ring: bool,
    pub mana_magnet: bool,
    pub life_magnet: bool,
    pub treasure_magnet: bool,
    /// `difficulty == 3 && FarPlacementRangePower.IsEnabledForPlayer`
    /// (`Player.cs:35161-35164`) — the port's Journey power flag.
    pub journey_far_range: bool,
}

/// `Player.GetItemGrabRange(Item)` (`Player.cs:35130-35166`): the base 42
/// plus the per-accessory/per-kind margins. The 4143 mana-restore pickup
/// gets the mana margin unconditionally (`:35141-35144`); the void lens
/// 3822 +50 (`:35153-35156`); the nebula pickups +100 (`:35157-35160`).
pub(crate) fn get_item_grab_range(kind: u16, flags: GrabFlags) -> i32 {
    let mut num = DEFAULT_ITEM_GRAB_RANGE;
    if flags.gold_ring && registry::is_coin(kind) {
        num += COIN_GRAB_RANGE;
    }
    if flags.mana_magnet && matches!(kind, 184 | 1735 | 1868) {
        num += MANA_GRAB_RANGE;
    }
    if kind == 4143 {
        num += MANA_GRAB_RANGE;
    }
    if flags.life_magnet && matches!(kind, 58 | 1734 | 1867) {
        num += LIFE_GRAB_RANGE;
    }
    if flags.treasure_magnet {
        num += TREASURE_GRAB_RANGE;
    }
    if kind == 3822 {
        num += 50;
    }
    if matches!(kind, 3453 | 3454 | 3455) {
        num += 100; // `ItemID.Sets.NebulaPickup` (`ItemID.cs:1324`)
    }
    if flags.journey_far_range {
        num += 240;
    }
    num
}

/// `Rectangle.Intersects` (XNA): strict edge overlap on both axes — the
/// touching edges do NOT intersect. The C# casts the float positions to
/// int first (`Player.cs:34926`'s `(int)base.position.X`).
pub(crate) fn rects_intersect(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
) -> bool {
    let (ax, ay, bx, by) = (ax as i32, ay as i32, bx as i32, by as i32);
    let (aw, ah, bw, bh) = (aw as i32, ah as i32, bw as i32, bh as i32);
    bx < ax + aw && ax < bx + bw && by < ay + ah && ay < by + bh
}

/// `GrabItems`' expanded grab rectangle (`Player.cs:34926`):
/// `new Rectangle((int)position.X - itemGrabRange, (int)position.Y -
/// itemGrabRange, width + itemGrabRange * 2, height + itemGrabRange * 2)`
/// against the drop's 16x16 hitbox.
pub(crate) fn grab_rect_intersects(
    px: f32,
    py: f32,
    pw: f32,
    ph: f32,
    range: f32,
    drop_pos: Vec2,
) -> bool {
    rects_intersect(
        px - range,
        py - range,
        pw + range * 2.0,
        ph + range * 2.0,
        drop_pos.x,
        drop_pos.y,
        SIZE,
        SIZE,
    )
}

/// `Player.PullItem_Common` (`Player.cs:34983-35029`): the axis-wise
/// acceleration toward the player — `±itemGrabSpeed` per tick while under
/// the `±itemGrabSpeedMax + playerVx` cap (X only; the Y cap carries no
/// player term, `:35009`/`:35020`), plus the extra `itemGrabSpeed *
/// xPullSpeed` brake while the velocity still points AWAY from the player.
/// `xPullSpeed` is 0.75 for the common arm (`:34971`).
pub(crate) fn pull_item_common(
    drop_center: Vec2,
    vel: &mut Vec2,
    player_center: Vec2,
    player_vx: f32,
    x_pull_speed: f32,
) {
    if player_center.x > drop_center.x {
        if vel.x < ITEM_GRAB_SPEED_MAX + player_vx {
            vel.x += ITEM_GRAB_SPEED;
        }
        if vel.x < 0.0 {
            vel.x += ITEM_GRAB_SPEED * x_pull_speed;
        }
    } else {
        if vel.x > -ITEM_GRAB_SPEED_MAX + player_vx {
            vel.x -= ITEM_GRAB_SPEED;
        }
        if vel.x > 0.0 {
            vel.x -= ITEM_GRAB_SPEED * x_pull_speed;
        }
    }
    if player_center.y > drop_center.y {
        if vel.y < ITEM_GRAB_SPEED_MAX {
            vel.y += ITEM_GRAB_SPEED;
        }
        if vel.y < 0.0 {
            vel.y += ITEM_GRAB_SPEED * x_pull_speed;
        }
    } else {
        if vel.y > -ITEM_GRAB_SPEED_MAX {
            vel.y -= ITEM_GRAB_SPEED;
        }
        if vel.y > 0.0 {
            vel.y -= ITEM_GRAB_SPEED * x_pull_speed;
        }
    }
}

/// `Player.PullItem_Pickup` (`Player.cs:35031-35041`): the magnet/void-vault
/// pull — a `speed`-magnitude vector toward the player's centre, blended
/// into the velocity with weight `1/acc` (`(v*(acc-1) + pull)/acc`).
pub(crate) fn pull_item_pickup(
    drop_center: Vec2,
    vel: &mut Vec2,
    player_center: Vec2,
    speed: f32,
    acc: i32,
) {
    let d = player_center - drop_center;
    let num3 = speed / (d.x * d.x + d.y * d.y).sqrt();
    let pull = d * num3;
    let acc = acc as f32;
    vel.x = (vel.x * (acc - 1.0) + pull.x) / acc;
    vel.y = (vel.y * (acc - 1.0) + pull.y) / acc;
}

/// `NewItemOwnership.GrabDelayForLocalPlayer` (`Item.NewItem`'s ownership
/// argument on the player drop, `Player.cs:5199`): `grabDelayTime = 100`,
/// `grabDelayPlayer = localPlayerIndex` (`WorldItem.cs:306-309`) — the
/// dropping player cannot re-grab for 100 ticks, everyone else can at once.
/// The port's local player index is 0 (S1-P-i51).
pub(crate) const GRAB_DELAY_LOCAL: u8 = 0;
/// `NewItemOwnership.GrabDelayForAllPlayers` — `grabDelayPlayer = 255`
/// (`WorldItem.cs:310-313`); the set is `ItemID.Sets
/// .DropWithGrabDelayForAllPlayers = CreateBoolSet(267)` (`ItemID.cs:430`,
/// the Guide Voodoo Doll alone).
pub(crate) const GRAB_DELAY_ALL: u8 = 255;

/// One armed liquid splash (`WorldItem.cs:886-954` entry / `:965-1033`
/// exit, doc-sweep lane 36): the arm-time gate snapshot + the drop's
/// position. The C# arms run inline inside `MoveInWorld`; the port latches
/// them here and the items pass drains ([`ItemDrop::splash`]) right after
/// the drop's update — same tick, same slot order, the `UpdateItems`
/// stream. The gates are snapshotted because the `:1036-1041` dry clear
/// runs before the drain (the exit arm reads the pre-clear sticky flags).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Splash {
    /// The `:959` exit arm (vs the `:879` entry arm).
    pub exit: bool,
    /// Entry: the LOCAL `flag2` (`:886`, this tick's LavaCollision);
    /// exit: the sticky `base.lavaWet` (`:965`).
    pub lava: bool,
    /// The sticky `base.shimmerWet` at arm time (`:888`/`:967`).
    pub shimmer: bool,
    /// The sticky `base.honeyWet` at arm time (`:915`/`:994`).
    pub honey: bool,
    /// `base.position` at arm time (the dust box + the sound anchor).
    pub pos: Vec2,

        }/// One queued entry/exit splash burst (`WorldItem.cs:881-1035`): the drop
/// detected the wet edge and armed `wetCount = 20`; the items pass drains
/// this into the shared `DustPool` + the case-19 play on the `UpdateItems`
/// stream (the g67 rain-splash contract). The kind picks the dust family:
/// lava → 35 ×5, shimmer → 308 ×10 (the `Next(6)` colour switch), honey →
/// 152 ×5, water → `dustWater()` ×10.
pub(crate) struct ItemSplash {
    /// The drop's top-left at the edge (the dust box anchors on
    /// `position.X - 6`, `position.Y + height/2 - 8`; the water EXIT arm
    /// drops the `- 8`, `:1011`).
    /// `true` = the exit edge (`:959-1035`), `false` = entry (`:881-958`).
    /// The kind flags AT the edge (post-probe, pre-clear): lava wins, then
    /// shimmer, then honey, else water (`:886-940`/`:965-1019`).
}

#[derive(Clone, Debug)]
pub struct ItemDrop {
    pub kind: u16,
    pub count: u32,
    /// `WorldItem.shimmerTime` (`WorldItem.cs:34`) — the 0..1 ramp while the
    /// drop rests in shimmer liquid; the transform fires above 0.9.
    pub shimmer_time: f32,
    /// `WorldItem.shimmered` (`WorldItem.cs:32`) — the item has transformed at
    /// least once (it floats in the liquid afterwards).
    pub shimmered: bool,
    /// The ramp crossed 0.9 during the last [`Self::update`] — the transform
    /// fires that same tick (`WorldItem.cs:531-534` sets `shimmerTime = 0.9f`
    /// and calls `GetShimmered` inside `UpdateShimmer`).
    pub shimmer_pending: bool,
    /// `Entity.shimmerWet` (`Entity.cs:28`) — the drop's box overlaps shimmer
    /// liquid this tick.
    pub shimmer_wet: bool,
    /// `Entity.wet` — the liquid-overlap flag (`Collision.WetCollision`'s
    /// inset probe, `WorldItem.cs:870-956`, doc-sweep lane 36); the
    /// SpawnShimmeredItem modifier also writes it (`:635`).
    pub wet: bool,
    /// `Entity.honeyWet` (`WorldItem.cs:871-874`) — the honey gravity arm
    /// (`:386-391`).
    pub honey_wet: bool,
    /// `Entity.lavaWet` (`WorldItem.cs:865-869`) — the lava-death gate.
    pub lava_wet: bool,
    /// `Entity.wetCount` (`WorldItem.cs:883-885`/`:1042-1045`): the
    /// 20-tick splash cooldown, decremented every update. The splash
    /// dust/sound arms (`:886-954`/`:965-1033`) are PORTED (doc-sweep
    /// lane 36) — [`Self::splash`], drained by the items pass on the
    /// `UpdateItems` stream at the C# call position.
    pub wet_count: i32,
    /// The entry/exit splash latch (doc-sweep lane 36): `move_in_world`
    /// arms it when `wetCount` resets to 20 (`WorldItem.cs:885` entry /
    /// `:964` exit), snapshotting the arm-time gates + position; the items
    /// pass spends it on [`Self::splash`] after the update (the C# runs the
    /// arms inline — the drain is same-tick, pool-pass order, ANALYSIS 121).
    pub splash_due: Option<Splash>,
    /// The shimmered-repulsion brake (`WorldItem.cs:789-862`'s `flag`):
    /// the per-tick scan pass ([`shimmered_repulsion`]) arms it, the
    /// movement half spends it on the extra `velocity.X *= 0.8f`.
    pub repulsion_brake: bool,
    /// The shimmered-item repulsion scan's `flag` (`WorldItem.cs:789-862`):
    /// the items pass runs the slice-level scan before the drop's update and
    /// parks the result here; `move_in_world` consumes it (the
    /// `velocity.X *= 0.8f` tail, `:859-862`) and clears it. Transient.
    pub repulsed: bool,
    /// `Entity.wet` (`WorldItem.cs:879-958`) — `Collision.WetCollision` hit
    /// ANY liquid. Latched: set on entry (`:956`), cleared on exit (`:961`);
    /// the gravity head's water arm (`:392-396`) reads it.
    /// `Entity.honeyWet` (`WorldItem.cs:870-874`) — `Collision.honey` set it;
    /// cleared only when `!wet` (`:1039`). The honey gravity arm (`:386-391`).
    /// `Entity.lavaWet` (`WorldItem.cs:865-869`) — `Collision.LavaCollision`
    /// set it; cleared only when `!wet` (`:1038`). The EXIT splash arm reads
    /// it (`:965`); the ENTRY arm reads the LOCAL probe (`:886` `!flag2`,
    /// the SHIMMER-seal verbatim fix); lava keeps the air gravity pair (no
    /// C# arm).
    /// `Entity.wetCount` (`WorldItem.cs:883-1044`) — the splash cooldown: 20
    /// on an entry/exit burst (`:885`/`:964`), 0 on the grabbed arm (`:428`),
    /// decremented per update while > 0 (`:1042-1045`).
    /// The entry/exit splash the last [`Self::update`] queued
    /// (`WorldItem.cs:881-1035`): the dust + sound need the `App` (the shared
    /// `DustPool`, the `UpdateItems` stream, the audio), so the drop queues
    /// the request and the items pass drains it — the g67 rain-splash /
    /// `Player::take_splash_sound` pattern. `None` = no edge this tick.
    pub(crate) pending_splash: Option<ItemSplash>,
    /// The natural roll's prefix (`Item.NewItem`'s `prefix` argument,
    /// `Item.cs:49730`); 0 = none.
    pub prefix: u16,
    /// Top-left of the 16x16 item box, world pixels.
    pub pos: Vec2,
    pub vel: Vec2,
    /// `WorldItem.timeSinceItemSpawned`'s seconds stand-in (the C# counts
    /// ticks, `WorldItem.cs:437-440`); no longer gates pickup — the vanilla
    /// gate is [`Self::grab_delay_time`].
    pub age: f32,
    /// The spawn scatter's third draw (`Item.NewItem`'s velocity triplet,
    /// the port's pinned RNG order). Vanilla draws no positional bob
    /// (`Main.DrawItem` `Main.cs:38602-38788`); the field survives only to
    /// keep the spawn stream's draw count.
    pub phase: f32,
    /// `WorldItem.grabDelayTime` (`WorldItem.cs:26`): ticks the drop refuses
    /// every grabber (255) or one player; decremented per update
    /// (`:441-444`).
    pub grab_delay_time: i32,
    /// `WorldItem.grabDelayPlayer` (`WorldItem.cs:24`): the locked player
    /// index, 255 = all players (`GrabItems`' gate, `Player.cs:34911`).
    pub grab_delay_player: u8,
    /// `WorldItem.beingGrabbed` (`WorldItem.cs:40`): the pull step sets it;
    /// the next update skips the ground physics and just applies the
    /// velocity (`UpdateItem`'s else arm, `:425-435` → `ApplyMovement`
    /// `:1074-1084`), then clears it.
    pub being_grabbed: bool,
    /// `WorldItem.onConveyor` (`WorldItem.cs:864`'s stored bool): the belt
    /// contact suppresses the draw-side velocity spin (`Main.cs:38613`).
    pub on_conveyor: bool,
    // 347 §2/§3 keep-one: the g15 wet/wetCount/lavaWet/honeyWet field copies
    // deduped onto the e19 declarations above (same C# fields, same maths).
    /// `WorldItem.timeSinceItemSpawned`'s TICK counter (`WorldItem.cs:38`,
    /// incremented at `:437-440` with the 2147483547 cap) — the `% 12`
    /// fallen-star / boss-bag sparkle gate (`:1627`/`:1639`) and the 4143
    /// expiry's `> 300` (`:657`). The seconds stand-in [`Self::age`] stays
    /// for its own consumers.
    pub age_ticks: i32,
    // 347 §2.1 splash keep-one: the g15 `splash_edge` THIRD form RETIRED at
    // the e19 fold — e19's `pending_splash`/`ItemSplash`/`take_splash` (the
    // research/144 SHIMMER SEAL keeper) carries the same entry/exit window.
    /// `WorldItem.enemyGrabDelayTime` (`WorldItem.cs:28`): ticks the drop
    /// refuses the MONSTER grab (`GetPickedUpByMonsters_Special`'s `:405`
    /// gate); decremented per update (`:449-452`). The `ReserveForLocalPlayer`
    /// / `GrabDelayForLocalPlayer` ownership arms set it to `DefaultGrabDelay`
    /// (`Item.cs:49757-49760`). W4-g15 lane 5 — the port's monster grab (the
    /// copper-slime pickup) reads it; the revenge coin-suck's own gate is the
    /// `:405` row too. Distinct from [`Self::grab_delay_time`] (the PLAYER
    /// grab clock).
    pub enemy_grab_delay_time: i32,
    /// `WorldItem.instanced` (`WorldItem.cs:36`): a per-player loot split drop.
    /// The `SendData(21)` pre-switch refuses to sync it (`NetMessage.cs:92-95`)
    /// — it exists only for its reserved owner. No host spawn path sets it yet
    /// (the instanced-loot split is unported); the `90` client apply carries the
    /// reservation through `net_reserved` (W4-g50).
    pub instanced: bool,
}

// 347 §2.1 — the g15 `SplashKind` enum RETIRED with the splash third form;
// e19's `ItemSplash` carries the resolved kind as its lava/shimmer/honey
// flags (the same `:886-954`/`:965-1033` precedence).

impl ItemDrop {
    pub fn new(
        kind: u16,
        count: u32,
        prefix: u16,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        phase: f32,
    ) -> Self {
        Self {
            kind,
            count,
            shimmer_time: 0.0,
            shimmered: false,
            shimmer_pending: false,
            shimmer_wet: false,
            wet: false,
            honey_wet: false,
            lava_wet: false,
            wet_count: 0,
            splash_due: None,
            repulsion_brake: false,
            pending_splash: None,
            prefix,
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            age: 0.0,
            phase,
            grab_delay_time: 0,
            grab_delay_player: 255,
            being_grabbed: false,
            on_conveyor: false,
            age_ticks: 0,
            enemy_grab_delay_time: 0,
            instanced: false,
        }
    }

    /// Takes the queued entry/exit splash (the items pass drains it into the
    /// shared `DustPool` + the case-19 play — the g67 rain-splash pattern).
    pub(crate) fn take_splash(&mut self) -> Option<ItemSplash> {
        self.pending_splash.take()
    }

    /// `Item.IsAir` (`Item.cs:385-395`): type 0 is air, otherwise a non-positive
    /// stack is air. The `SendData(21)` pre-switch reads it to rewrite the send
    /// to `151 SyncItemDespawn` (`NetMessage.cs:96-99`).
    pub fn is_air(&self) -> bool {
        self.kind == 0 || self.count == 0
    }

    /// `WorldItem.ApplySpawnOwnership`'s `GrabDelayForLocalPlayer` /
    /// `GrabDelayForAllPlayers` arms (`WorldItem.cs:306-313`): the 100-tick
    /// lock. `Player.DropSelectedItem` spawns with the local-player delay
    /// (the voodoo doll 267 with the all-players one, `Player.cs:5199`,
    /// `ItemID.cs:430`).
    pub fn apply_grab_delay(&mut self, player: u8) {
        self.grab_delay_time = DEFAULT_GRAB_DELAY;
        self.grab_delay_player = player;
    }

    /// `Item.NewItem`'s `enemyGrabDelayTime` arm (`Item.cs:49757-49760`):
    /// the `ReserveForLocalPlayer` / `GrabDelayForLocalPlayer` ownership
    /// sets the monster-grab clock to `DefaultGrabDelay` (100). W4-g15
    /// lane 5 — the player-drop spawn calls it for the named-player arm
    /// (the `GrabDelayForAllPlayers` arm does NOT set it, `:49757`'s
    /// `ownership == ReserveForLocalPlayer || ownership ==
    /// GrabDelayForLocalPlayer` gate).
    pub fn apply_enemy_grab_delay(&mut self) {
        self.enemy_grab_delay_time = DEFAULT_GRAB_DELAY;
    }

    pub fn center(&self) -> Vec2 {
        self.pos + Vec2::splat(SIZE * 0.5)
    }

    /// `WorldItem.CanShimmerAtPosition` (`WorldItem.cs:756-770`): the RAMP
    /// gate's tile probe — `(int)(Center.X / 16)`, `(int)(position.Y / 16 -
    /// 1f)` (the tile above the drop's top-left), `liquid != 0 && shimmer()`.
    /// Distinct from the shimmered-float rise scan ([`Self::rise_over_shimmer`],
    /// `:493-504`'s two-row centre probe).
    pub fn over_shimmer(&self, world: &World) -> bool {
        let tx = (self.center().x / tr_core::TILE_PX) as i32;
        let ty = (self.pos.y / tr_core::TILE_PX - 1.0) as i32;
        world.tile(tx, ty)
            .is_some_and(|t| tr_core::shimmer::tile_is_shimmer(t))
    }

    /// `Collision.WetCollision(position, width, height)` (`WorldItem.cs:870`,
    /// `Collision.cs:1572-1651`): the CENTRE-INSET probe — the box shrunk to
    /// `min(10, width) × min(height/2, height)` (integer halves), the liquid
    /// surface drop, the FIRST-cell-wins kind flags and the sloped-tile-under-
    /// liquid arm. Returns `(wet, honey, shimmer)` — the `Collision.honey`/
    /// `Collision.shimmer` statics the caller copies (`:871-877`). The shared
    /// NPC-side port (`npc_ai_slime.rs::wet_collision`, W4-f47 phase 2) is the
    /// engine half; doc-sweep lane 36 retired the full-box stand-in. For the
    /// 16x16 drop box the probe is 10x8 at `(pos.x+3, pos.y+4)` (keep-one
    /// 144: the e19 round-3 box fix; the old 14-box read 10x7 at +2/+4).
    pub fn wet_probe(&self, world: &World) -> (bool, bool, bool) {
        crate::npc_ai_slime::wet_collision(world, self.pos.x, self.pos.y, SIZE as i32, SIZE as i32)
    }

    /// `Collision.WetCollision(position, width, height)`'s bool half
    /// (`WorldItem.cs:870`, `Collision.cs:1572-1651`) — the CARRIED
    /// `base.wet` read the `UpdateItem_VisualEffects` light arms gate on
    /// (`app/frame/draw.rs`'s feed). The faithful centre-inset probe (the
    /// `player::wet_collision` port — the same box maths as
    /// [`Self::wet_probe`]'s `npc_ai_slime` engine, minus the kind statics).
    /// Keep-one 144: the e19 lane's `box_is_wet` name kept (its draw-feed
    /// caller), the doc-sweep 36 deletion retired — the probe engines
    /// agree; only the carried-flag vs fresh-probe read differs (the C#
    /// light arms read the Entity flag; the draw feed re-probes — the
    /// recorded S1-E-5 draw-side convention).
    pub fn box_is_wet(&self, world: &World) -> bool {
        crate::player::wet_collision(world, self.pos.x, self.pos.y, SIZE, SIZE)
    }

    /// `Collision.LavaCollision(position, width, height)` (`WorldItem.cs:865`,
    /// `Collision.cs:1653-1684`): the FULL-box probe (no inset) against the
    /// liquid-surface-dropped lava cells — the shared player-side port
    /// (`player.rs::lava_collision`). Doc-sweep lane 36 retired the cell-grid
    /// stand-in (it ignored the surface drop).
    pub fn lava_probe(&self, world: &World) -> bool {
        crate::player::lava_collision(world, self.pos.x, self.pos.y, SIZE, SIZE)

        }    /// `WorldItem.UpdateShimmer`'s shimmered-float rise scan
    /// (`WorldItem.cs:493-504`): when NOT `shimmerWet`, the two rows at the
    /// CENTRE (`(int)(Center.X/16)`, `(int)(Center.Y/16)` and `+1`) — any
    /// shimmer liquid cell rises the drop. The old port reused the ramp's
    /// single above-top-left tile (a different probe).
    fn rise_over_shimmer(&self, world: &World) -> bool {
        let tx = (self.center().x / tr_core::TILE_PX) as i32;
        let ty = (self.center().y / tr_core::TILE_PX) as i32;
        (ty..ty + 2).any(|y| {
            world.tile(tx, y)
                .is_some_and(|t| tr_core::shimmer::tile_is_shimmer(t))
        })
    /// `Collision.WetCollision(position, width, height)` (`WorldItem.cs:870`):
    /// the drop's box overlaps any non-empty liquid cell. This is `base.wet`,
    /// the gate on most `UpdateItem_VisualEffects` light arms. The faithful
    /// probe (the `player::wet_collision` port, `Collision.cs:1572-1651`).
    }

    /// Whether the last [`Self::update`]'s `UpdateShimmer` armed the transform
    /// (`WorldItem.cs:528-535` — the C# caps the ramp at 0.9 and fires for the
    /// local player's drops; the port is SP-only, so the fire is unconditional
    /// here and the client re-own branch is out).
    pub fn shimmer_transform_due(&self) -> bool {
        self.shimmer_pending && !self.shimmered
    }

    /// `WorldItem.UpdateShimmer` (`WorldItem.cs:455-546`): the shimmered
    /// physics (no gravity, the 0.05 rise over shimmer), the ramp ±0.01 and the
    /// 0.9 trigger check. The dust/light arms are render-layer notes.
    /// `gravity` is the `ref float` the C# zeroes for a shimmered drop
    /// (`:483`) — the movement half reads the overridden value.
    fn update_shimmer(&mut self, world: &World, dt: f32, gravity: &mut f32) {

        }    /// 0.9 trigger check. The `:482` light arm feeds from the frame pass
    /// (`entity_lights::shimmer_light`, W4-g20); the dust arm is a render-layer
    /// note.
    fn update_shimmer(&mut self, world: &World, dt: f32) {
        if self.shimmered {
            // `:457-508` — gravity is zero for the transformed item; the rise
            // applies over shimmer liquid (or already shimmerWet), floor -4.
            *gravity = 0.0; // :483
            let over = self.shimmer_wet || self.over_shimmer(world);
            if over && self.vel.y > -4.0 {
                self.vel.y -= 0.05;
            } else if !over {

        }
        }
        }    /// 0.9 trigger check. Returns the gravity override (`Some(0)` for a
    /// shimmered drop — `:483` `gravity = 0f`; `None` keeps the caller's
    /// wet/air pair). The dust/light arms (`:459-482`) are render-layer notes.
    fn update_shimmer(&mut self, world: &World) -> Option<f32> {
        let mut gravity_override = None;
            // `:457-516` — gravity is zero for the transformed item (`:483`).
            gravity_override = Some(0.0);
            // The rise: `shimmerWet` directly (`:484-490`), else the two-row
            // centre scan (`:493-504`); floor -4, else the ×0.9 dry damp.
            let over = if self.shimmer_wet {
                true
            } else {
                self.rise_over_shimmer(world)
            };
            if over {
                if self.vel.y > -4.0 {
                    self.vel.y -= 0.05;
                }
                self.vel.y *= 0.9;
            }
        }
        if self.shimmer_wet && !self.shimmered && self.over_shimmer(world) {
            self.shimmer_time = (self.shimmer_time + 0.01).min(1.0);
            // `:518-536` — the crossing is checked inside the same update, so
            // the value is capped at 0.9 and the transform fires this tick.
            self.shimmer_pending = self.shimmer_time > 0.9;
        } else if self.shimmer_time > 0.0 {
            // `:538-545` — the decay is NOT gated on `!shimmered`: a
            // transformed drop's 1.0 ramp dries back to 0 over 100 ticks,
            // which is what re-opens `GrabItems`' `shimmerTime != 0` gate
            // (`Player.cs:34911`, S1-P-i47).
            self.shimmer_pending = false;
            self.shimmer_time = (self.shimmer_time - 0.01).max(0.0);
        }
        gravity_override
    }

    /// `WorldItem.TryCombiningIntoNearbyItems` (`WorldItem.cs:157-186`,
    /// S1-P-i47): the drop sucks later same-kind drops within the 30 px
    /// Manhattan window into itself, up to its own `maxStack`. The C# scans
    /// `Main.item[whoAmI+1..400]` — the port's list order is the slot order
    /// (the removals are `swap_remove`, a recorded list-vs-array deviation).
    /// The merge lerps position AND velocity by `moved/newStack` toward the
    /// survivor (`:172-174`), and airs the drained source out. The gates:
    /// `shimmerTime == 0`, `CanPassivelyStackInWorld`, `stack < maxStack`
    /// (`:159`); per target `CanStack` + equal `shimmered` (`:167`). The
    /// reservation equality is the net-only term — the caller passes the
    /// `net_reserved` answer (`reserved_elsewhere`, the `:159` gate; the
    /// host owns every drop in SP so it is false there). RNG-free. Returns
    /// the indices the caller must remove
    /// (descending — the caller `swap_remove`s them highest-first so the
    /// lower indices stay valid; the port's compaction deviation).
    /// W4-g15 lane 5 — the reservation gate `playerIndexTheItemIsReservedFor
    /// != Main.myPlayer` (`WorldItem.cs:159`): a drop reserved for ANOTHER
    /// peer does not combine. The port's host owns every drop in SP (the
    /// gate is vacuous there); the MP reservation lives in `net_reserved`
    /// and the caller passes the answer.
    pub(crate) fn combine_nearby(
        drops: &mut [ItemDrop],
        i: usize,
        reserved_elsewhere: bool,
    ) -> Vec<usize> {
        let mut drained = Vec::new();
        let max = registry::max_stack(drops[i].kind);
        if reserved_elsewhere
            || drops[i].shimmer_time != 0.0
            || !can_passively_stack_in_world(drops[i].kind)
            || drops[i].count >= max
        {
            return drained;
        }
        for j in i + 1..drops.len() {
            // NOTE: the C# re-checks no fullness inside the loop — a
            // survivor that filled mid-scan keeps "merging" 0 with
            // `amount = 0`, lerping its position onto each further
            // same-kind neighbour (`:169-174`, the vanilla quirk kept).
            // `Item.CanStack` (`Item.cs:49865-49872`) is type + prefix
            // equality; the `!IsAir` gate is `count > 0` (the port's list
            // has no air slots, but a drained source keeps count 0 until
            // the caller removes it). The reservation-equality term
            // (`worldItem.playerIndexTheItemIsReservedFor == this.…`,
            // `:167`) is the MP per-target gate — the port's host owns
            // every drop in SP (all reserved for the local player), so it
            // is vacuous there; the caller's `reserved_elsewhere` covers
            // the survivor's own `:159` gate.
            let same = drops[j].count > 0
                && drops[j].kind == drops[i].kind
                && drops[j].prefix == drops[i].prefix
                && drops[j].shimmered == drops[i].shimmered
                && ((drops[i].pos.x - drops[j].pos.x).abs()
                    + (drops[i].pos.y - drops[j].pos.y).abs())
                    <= 30.0;
            if !same {
                continue;
            }
            let moved = drops[j].count.min(max - drops[i].count);
            drops[j].count -= moved;
            drops[i].count += moved;
            // `amount = num2 / this.stack` AFTER the add (`:172`).
            let amount = moved as f32 / drops[i].count as f32;
            let (src_pos, src_vel) = (drops[j].pos, drops[j].vel);
            drops[i].pos = src_pos.lerp(drops[i].pos, amount);
            drops[i].vel = src_vel.lerp(drops[i].vel, amount);
            if drops[j].count == 0 {
                drained.push(j); // `TurnToAir` (`:175-178`)
            }
        }
        drained
    }

    /// `WorldItem.MoveInWorld`'s shimmered-repulsion scan
    /// (`WorldItem.cs:789-848`, W4-f47): a shimmered drop pushes away from
    /// up to 50 OTHER shimmered drops whose centres are within
    /// `(width_i + width_j) / 2` on BOTH axes. `num2 = halfWidthSum /
    /// Distance` clamped at 10; the X push is ±0.1·num2 with the ±3·num2
    /// self-cap and the ±3 other-cap (the equal-X arm uses ±3·num2 on the
    /// other). Sets `repulsion_brake` (the `flag`) when any neighbour
    /// matched — the movement half spends it on the extra ×0.8 X friction
    /// (`:859-862`). The C# scans `Main.item[0..400]` skipping self; the
    /// port's list order is the slot order (the recorded deviation).
    /// RNG-free. Runs as a pre-pass over the whole list (the C# reads the
    /// live `Main.item[]` velocities mid-scan; the port's two-phase split
    /// reads the pre-tick velocities — the one-tick lag is the documented
    /// pool-pass convention).
    pub(crate) fn shimmered_repulsion(drops: &mut [ItemDrop]) {
        // Precompute the centres (the scan reads them repeatedly).
        let centers: Vec<Vec2> = drops.iter().map(|d| d.center()).collect();
        for i in 0..drops.len() {
            if !drops[i].shimmered {
                continue;
            }
            let mut flag = false;
            let mut budget = 50;
            for j in 0..drops.len() {
                if i == j || !drops[j].shimmered {
                    continue;
                }
                if budget <= 0 {
                    break;
                }
                budget -= 1;
                // `num2 = (width + other.width) / 2` — both 16 → 16.
                let mut num2 = (SIZE + SIZE) / 2.0;
                let dx = (centers[i].x - centers[j].x).abs();
                let dy = (centers[i].y - centers[j].y).abs();
                if dx > num2 || dy > num2 {
                    continue;
                }
                flag = true;
                let dist = (centers[i] - centers[j]).length();
                num2 /= dist;
                if num2 > 10.0 {
                    num2 = 10.0;
                }
                let push = 0.1 * num2;
                if centers[i].x < centers[j].x {
                    if drops[i].vel.x > -3.0 * num2 {
                        drops[i].vel.x -= push;
                    }
                    if drops[j].vel.x < 3.0 {
                        drops[j].vel.x += push;
                    }
                } else if centers[i].x > centers[j].x {
                    if drops[i].vel.x < 3.0 * num2 {
                        drops[i].vel.x += push;
                    }
                    if drops[j].vel.x > -3.0 {
                        drops[j].vel.x -= push;
                    }
                } else if i < j {
                    // The equal-X arm: the lower slot pushes left.
                    if drops[i].vel.x > -3.0 * num2 {
                        drops[i].vel.x -= push;
                    }
                    if drops[j].vel.x < 3.0 * num2 {
                        drops[j].vel.x += push;
                    }
                }
            }
            drops[i].repulsion_brake = flag;
        }

        }    /// `WorldItem.MoveInWorld`'s shimmered-item repulsion scan
    /// (`WorldItem.cs:790-847`): a shimmered drop pushes apart from every other
    /// shimmered drop whose centre lies within the half-width span on BOTH
    /// axes, up to a 50-neighbour budget. The push scales by `span / distance`
    /// (capped at 10) and nudges each side's `velocity.X` by `0.1 * span` toward
    /// separation, bounded at `±3 * span` for self and `±3` for the neighbour
    /// (the equal-centre tie-break uses `±3 * span` for both and only fires for
    /// the lower slot). Returns the C#'s `flag` (any neighbour touched), which
    /// the caller parks in [`Self::repulsed`] for the `velocity.X *= 0.8f` tail
    /// (`:859-862`). Runs on the live slice so neighbours feel the push this
    /// tick; the port's list order stands in for `Main.item` slot order (the
    /// `combine_nearby` deviation). The span is the drop box [`SIZE`] — the
    /// C# 16 (`WorldItem.width = 16`, `:138`; W4-e19 round 3 retired the old
    /// 14 px stand-in).
    pub(crate) fn repel_shimmered(items: &mut [ItemDrop], i: usize) -> bool {
        if !items[i].shimmered {
            return false;
        let mut flag = false;
        let mut budget = 50i32;
        let self_center = items[i].center();
        for j in 0..items.len() {
            // `:795` — skip self and non-shimmered slots (every port drop is
            // active, so the C# `!Main.item[i].active` term is vacuous).
            if j == i || !items[j].shimmered {
            // `:799-802` — `if (num-- <= 0) break`: the budget counts only
            // shimmered candidates, 50 of them.
            if budget <= 0 {
                break;
            budget -= 1;
            let other_center = items[j].center();
            // `:803` — `num2 = (width + item[i].width) / 2`.
            let mut span = SIZE;
            let dx = self_center.x - other_center.x;
            let dy = self_center.y - other_center.y;
            // `:804` — both axes must lie within the span.
            if !(dx.abs() <= span) || !(dy.abs() <= span) {
            flag = true;
            // `:809-813` — `num2 /= Distance`, capped at 10. A zero distance
            // (coincident centres) divides to +inf and caps to 10, like the C#.
            let dist = (dx * dx + dy * dy).sqrt();
            span /= dist;
            if span > 10.0 {
                span = 10.0;
            let (mut svx, mut ovx) = (items[i].vel.x, items[j].vel.x);
            if self_center.x < other_center.x {
                // `:814-824` — self is left of the neighbour: push self left,
                // neighbour right.
                if svx > -3.0 * span {
                    svx -= 0.1 * span;
                if ovx < 3.0 {
                    ovx += 0.1 * span;
            } else if self_center.x > other_center.x {
                // `:825-835` — self is right: push self right, neighbour left.
                if svx < 3.0 * span {
                    svx += 0.1 * span;
                if ovx > -3.0 {
                    ovx -= 0.1 * span;
            } else if i < j {
                // `:836-846` — coincident X: the lower slot pushes left, the
                // higher right (both bounded at `±3 * span`).
                if ovx < 3.0 * span {
            items[i].vel.x = svx;
            items[j].vel.x = ovx;
        flag
    }

    /// One fixed 60 Hz step of `WorldItem.UpdateItem`'s physics half
    /// (`WorldItem.cs:338-453`): the shimmer update, the ground movement and
    /// the grab-delay decrement. The pickup pull/pick lives in the
    /// `GrabItems` step (`world_sim/items.rs`, S1-P-i47) — the C# runs it
    /// from `Player.Update` (`Player.cs:27428`), not from the item update.
    /// `being_grabbed` skips the ground physics for the tick after a pull
    /// (`:425-435`): wet flags clear, the velocity applies raw
    /// (`ApplyMovement`, `:1074-1084`), the flag resets.
    ///
    /// **RNG-free** (W4-g15): the `UpdateShimmer` sparkle (`:459-481`) and the
    /// splash bursts (`:879-1035`) draw `Main.rand`, so they run in the
    /// dedicated `item_dust` drains at their C# positions, not here. This
    /// method keeps the flag/cooldown arithmetic (`move_in_world`'s edge
    /// arms queue [`Self::pending_splash`] the drain reads — the 347 §2.1
    /// keep-one: e19's `ItemSplash` form, the g15 `splash_edge` retired).
    pub fn update(&mut self, world: &World, dt: f32) {
        self.age += dt;
        // `WorldItem.cs:379-396` — the gravity ladder + the wetVelocity
        // scale, read from the PRE-tick wet flags (the probe that refreshes
        // them runs inside MoveInWorld, `:865-877`, AFTER UpdateShimmer).
        let (mut gravity, max_fall, wet_scale) =
            item_gravity_ladder(self.wet, self.shimmer_wet, self.honey_wet);
        let mut wet_vel = self.vel * wet_scale;
        if self.being_grabbed {
            // `UpdateItem`'s grabbed arm (`WorldItem.cs:425-435`): the wet
            // flags + wetCount clear, the flag resets, ApplyMovement runs
            // with the ladder's wetVelocity (wet is now false → raw).
            // `UpdateItem`'s grabbed arm (`WorldItem.cs:425-435`): ALL the wet
            // flags + the cooldown clear (`:427-431`) and `ApplyMovement`
            // integrates at FULL velocity (`wet` false → `:1082`).
            self.wet = false;
            self.wet_count = 0;
            self.lava_wet = false;
            self.honey_wet = false;
            self.shimmer_wet = false;
            self.being_grabbed = false;
            self.on_conveyor = false;
            self.pos += self.vel; // ApplyMovement, wet false (`:1082`)
        } else {
            // `UpdateShimmer` (`:403`) reads the PREVIOUS tick's
            // `Entity.shimmerWet` — MoveInWorld's probe (`:875-877`) refreshes
            // it later in this same update. The old pre-update full-box
            // re-probe is retired (doc-sweep lane 36): the flag now carries
            // across ticks exactly like the C# Entity field.
            self.update_shimmer(world, dt, &mut gravity);
            self.move_in_world(world, gravity, max_fall, &mut wet_vel);
            // `UpdateItem`'s gravity head (`:364-396`): the air pair, then the
            // ELSE-IF chain shimmer-wet → honey-wet → water-wet, all read from
            // the PREVIOUS tick's flags (`MoveInWorld` re-probes at `:865-878`,
            // AFTER this head). `:379` — `wetVelocity = velocity * 0.5f`; the
            // shimmer arm re-scales to `* 0.375f` (`:384`), honey to `* 0.25f`
            // (`:390`); the water arm keeps the 0.5 default (`:392-396`).
            let (mut gravity, mut max_fall) = (GRAVITY, MAX_FALL);
            let mut wet_step = WET_STEP_AIR;
            if self.shimmer_wet {
                gravity = SHIMMER_GRAVITY;
                max_fall = SHIMMER_MAX_FALL;
                wet_step = WET_STEP_SHIMMER;
            } else if self.honey_wet {
                gravity = HONEY_GRAVITY;
                max_fall = HONEY_MAX_FALL;
                wet_step = WET_STEP_HONEY;
            } else if self.wet {
                gravity = WATER_GRAVITY;
                max_fall = WATER_MAX_FALL;
            }
            // 347 §3.2(c) — the shimmered routing reconciled onto e19's
            // two-cell probe: the g15 `pos += vel` float branch + the
            // `over_shimmer` approximation retired; `update_shimmer`'s
            // `rise_over_shimmer` (`:493-504`) is the C# window. The g15
            // box-union wet probes + `wet_edges` retired too — e19's
            // `move_in_world` carries the faithful `:865-878` re-probe
            // (LavaCollision + WetCollision's honey/shimmer statics) and the
            // `:879-1035` edge arms at their C# positions.
            //
            // `UpdateShimmer` (`:403`) runs BEFORE `MoveInWorld` (`:413`); a
            // shimmered drop overrides gravity to 0 (`:483`).
            if let Some(g) = self.update_shimmer(world) {
                gravity = g;
            // `MoveInWorld` (`:413`) runs for EVERY non-grabbed drop — the
            // shimmered one too (gravity 0, but the friction/conveyor/collision
            // tail still applies; the old port skipped it with a bare
            // `pos += vel`).
            self.move_in_world(world, gravity, max_fall, wet_step);
        }
        // `WorldItem.cs:441-444` — the grab delay ticks down every update,
        // grabbed or not.
        if self.grab_delay_time > 0 {
            self.grab_delay_time -= 1;
        }
        // `WorldItem.cs:449-452` — the enemy-grab delay ticks down every
        // update too (W4-g15 lane 5). The `timeToKeepReservation` decrement
        // (`:445-448`) is the MP reservation clock — the port's host owns
        // every drop in SP, so it has no live consumer (the net_reserved
        // hold rides the client's `22 ItemOwner` handler).
        if self.enemy_grab_delay_time > 0 {
            self.enemy_grab_delay_time -= 1;
        }
    }

    /// `WorldItem.MoveInWorld` (`WorldItem.cs:772-1072`, W4-f47): the
    /// NoGravity / gravity-friction head, the conveyor bend, the wet probes
    /// (the splash arms latch [`Splash`] for the items pass's drain —
    /// doc-sweep lane 36), the
    /// `ignoreAetheriumPlatforms: true` tile collision, `ApplyMovement`
    /// (the wetVelocity integration) and the slope pass. `gravity` /
    /// `max_fall` / `wet_vel` come from `update`'s ladder (the shimmered
    /// override already applied). The shimmered-repulsion scan runs as the
    /// list pre-pass ([`Self::shimmered_repulsion`]) and arms
    /// `repulsion_brake` — spent here on the extra ×0.8 X friction.
    fn move_in_world(&mut self, world: &World, gravity: f32, max_fall: f32, wet_vel: &mut Vec2) {
        if !self.shimmered && item_no_gravity(self.kind) {
            // `:774-786` — the floater arm: both axes ×0.95 + the 0.1 snap.
            self.vel.x *= FRICTION_X;
            if self.vel.x.abs() < FRICTION_EPSILON {
                self.vel.x = 0.0;
            }
            self.vel.y *= FRICTION_X;
            if self.vel.y.abs() < FRICTION_EPSILON {
                self.vel.y = 0.0;
            }
        } else {
            // `:849-862` — the gravity fall + cap, the X friction + snap,
            // then the repulsion brake's extra ×0.8.
            self.vel.y = (self.vel.y + gravity).min(max_fall);

        }
        }    /// The non-grabbed movement half: `MoveInWorld`'s gravity/friction head
    /// (`WorldItem.cs:849-864`) and the tile-collision pass.
    fn move_in_world(&mut self, world: &World) {
        // `WorldItem.cs:774-786` — the `ItemID.Sets.ItemNoGravity` arm
        // (`ItemID.cs:1320`): souls (520/521), the nebula pickups and the
        // like float — BOTH axes decay ×0.95 with the ±0.1 rest snap, NO
        // gravity and NO fall cap. The port models the two soul ids (the
        // W4-f32 loot lane); the other set members land with their items.
        if !self.shimmered && matches!(self.kind, 520 | 521) {
            self.vel.x *= 0.95;
            if self.vel.x < 0.1 && self.vel.x > -0.1 {
            self.vel.y *= 0.95;
            if self.vel.y < 0.1 && self.vel.y > -0.1 {
            // `WorldItem.cs:363-364` — the air gravity and the fall cap.
            self.vel.y = (self.vel.y + GRAVITY).min(MAX_FALL);
            // `WorldItem.cs:854-860` — the friction and the `0.1` rest snap,
            // both applied BEFORE the collision pass (`MoveInWorld`'s order).
            self.vel.x *= FRICTION_X;
            if self.vel.x.abs() < FRICTION_EPSILON {
                self.vel.x = 0.0;
            }
            if self.repulsion_brake {
                self.vel.x *= 0.8; // :859-862
            }
        }
        self.repulsion_brake = false;
        // `:864` — the belt contacts bend the velocity before the collision.

        }
        }
        }    /// `WorldItem.cs:437-440` — the `timeSinceItemSpawned` tick (the cap
    /// included). The C# runs it AFTER `UpdateItem_VisualEffects` (`:436`),
    /// so the `% 12` sparkle gate and the 4143 `> 300` expiry read the
    /// PRE-increment value this tick; the items pass calls this after the
    /// visual drains (W4-g15).
    pub fn tick_age(&mut self) {
        if self.age_ticks < 2147483547 {
            self.age_ticks += 1;
    }
    // 347 §2.1 — the g15 `wet_edges` state machine RETIRED: e19's
    // `move_in_world` carries the same entry/exit window inline at the C#
    // `:879-1035` positions (with the local-lava entry read, the SHIMMER
    // seal's verbatim fix) and queues `pending_splash`.
    /// The non-grabbed movement half: `MoveInWorld`'s `ItemNoGravity` float
    /// head (`WorldItem.cs:774-786`) or the gravity/friction head
    /// (`:849-862`), then the conveyor + tile-collision pass. `gravity`/
    /// `max_fall` are the caller's resolved pair (the shimmer-wet override or
    /// the shimmered 0); `wet_step` is the head's `wetVelocity` factor
    /// (`:379-390`). The repulsion scan itself (`:790-847`) is the
    /// slice-level [`Self::repel_shimmered`] the items pass runs before this
    /// update; its `flag` rides [`Self::repulsed`].
    fn move_in_world(&mut self, world: &World, gravity: f32, max_fall: f32, wet_step: f32) {
            // `WorldItem.cs:774-786` — the float: BOTH axes ×0.95 with the
            // 0.1 rest snap, no gravity add, no repulsion (the scan lives in
            // the else arm and is shimmered-only anyway).
            // `WorldItem.cs:849-853` — the gravity add and the fall cap.
            // `WorldItem.cs:859-862` — the repulsion flag's damp, AFTER the
            // friction/snap and BEFORE the conveyor pass.
            if self.repulsed {
                self.vel.x *= 0.8;
        self.repulsed = false;
        // `WorldItem.cs:864`: the belt contacts bend the velocity before the
        // AABB pass. The C# stores the bool in `item.onConveyor`, which
        // suppresses the draw-side velocity spin (`Main.cs:38613`).
        let mut bp = [self.pos.x, self.pos.y];
        let mut bv = [self.vel.x, self.vel.y];
        self.on_conveyor = crate::collision::apply_conveyor_belt_movement_to_velocity(
            world,
            &mut bp,
            &mut bv,
            SIZE as i32,
            SIZE as i32,
        );
        self.pos = Vec2::new(bp[0], bp[1]);
        self.vel = Vec2::new(bv[0], bv[1]);
        // `:865-877` — the probes (doc-sweep lane 36): LavaCollision is the
        // FULL box against the surface-dropped lava cells (`:865`, sticky —
        // only ever SET here); WetCollision is the centre-inset probe whose
        // FIRST overlapping cell wins and sets the honey/shimmer statics
        // (`:870-877`, also sticky). The full-box stand-ins are retired.
        let lava_now = self.lava_probe(world);
        if lava_now {
            self.lava_wet = true; // :866-869
        }
        let (now_wet, honey, shimmer) = self.wet_probe(world);
        if honey {
            self.honey_wet = true; // :871-874
        }
        if shimmer {
            self.shimmer_wet = true; // :875-878
        }
        if now_wet {
            if !self.wet && self.wet_count == 0 {
                self.wet_count = 20; // :883-885
                // `:886-954` — the entry splash: the lava gate reads THIS
                // tick's flag2 (the LOCAL, `:886`), the shimmer/honey gates
                // read the sticky Entity flags. The arms drain post-update
                // on the UpdateItems stream ([`Self::splash`]).
                self.splash_due = Some(Splash {
                    exit: false,
                    lava: lava_now,
                    shimmer: self.shimmer_wet,
                    honey: self.honey_wet,
                    pos: self.pos,
                });
            }
            self.wet = true; // :956
        } else if self.wet {
            self.wet = false; // :961
            if self.wet_count == 0 {
                self.wet_count = 20; // :962-964
                // `:965-1033` — the exit splash reads the STICKY flags
                // (`:965` base.lavaWet) BEFORE the `:1036-1041` dry clear —
                // the snapshot keeps them past the clear for the drain.
                self.splash_due = Some(Splash {
                    exit: true,
                    lava: self.lava_wet,
                    shimmer: self.shimmer_wet,
                    honey: self.honey_wet,
                    pos: self.pos,
                });
            }

        }        // `MoveInWorld`'s wet re-probe (`:865-878`), BEFORE the collision +
        // `ApplyMovement` halves: `LavaCollision` (`:865`) then
        // `WetCollision` (`:870`, which also sets the `Collision.honey`/
        // `Collision.shimmer` statics the `:871-878` arms copy). The flags
        // LATCH (`:868`/`:873`/`:877` are set-only); `wet` itself follows the
        // entry/exit edges (`:956`/`:961`) and the `!wet` tail (`:1036-1041`)
        // clears the three kind flags. The probe feeds BOTH the next tick's
        // gravity head and THIS tick's `ApplyMovement` gate (`:1076` reads
        // `base.wet` post-set).
        let lava_now =
            crate::player::lava_collision(world, self.pos.x, self.pos.y, SIZE, SIZE);
            self.lava_wet = true; // `:866-869`
        let (wet_hit, honey_hit, shimmer_hit) = crate::npc_ai_slime::wet_collision(
            world,
            self.pos.x,
            self.pos.y,
            SIZE as i32,
        );
        if honey_hit {
            self.honey_wet = true; // `:871-874`
        if shimmer_hit {
            self.shimmer_wet = true; // `:875-878`
        // `:879-958`/`:959-1035` — the entry/exit edges. Each arms
        // `wetCount = 20` when the cooldown is free and queues the splash
        // burst (the dust family reads the kind flags AT the edge — the
        // entry arm the freshly-set ones, the exit arm the OLD ones before
        // the `!wet` tail clears them).
        if wet_hit {
            if !self.wet {
                if self.wet_count == 0 {
                    self.wet_count = 20; // `:885`
                    self.pending_splash = Some(ItemSplash {
                        pos: self.pos,
                        exit: false,
                        // `:886` reads the LOCAL `flag2` (this tick's
                        // LavaCollision), NOT the sticky `base.lavaWet`.
                        // Behaviourally equal at every entry edge (the
                        // `!wet` tail cleared the sticky flag last tick),
                        // but the local read is the verbatim one — the
                        // f47 keep-one 144's only semantic objection,
                        // resolved on the e19 half (SHIMMER seal).
                        lava: lava_now,
                        shimmer: self.shimmer_wet,
                        honey: self.honey_wet,
                    });
                }
                self.wet = true; // `:956`
            self.wet = false; // `:961`
                self.wet_count = 20; // `:964`
                self.pending_splash = Some(ItemSplash {
        // `:1036-1041` — while dry, the kind flags clear.
        if !self.wet {
            self.lava_wet = false;
            self.honey_wet = false;
            self.shimmer_wet = false;
        // `:1042-1045` — the cooldown ticks at the wet block's tail.
        if self.wet_count > 0 {
            self.wet_count -= 1;
        let wet_now = self.wet;
        // `wetVelocity` (`:379-390`'s head value, `:1046-1066`'s refresh):
        // the collision pass runs at FULL velocity; an axis it overrides
        // keeps that residual in `wetVelocity` too (`:1052-1059`), the free
        // axes integrate at the liquid fraction.
        let mut wet_vel = self.vel * wet_step;
        let pre = self.vel;
        let nx = self.pos.x + self.vel.x;
        if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
            // `Collision.TileCollision` (`Collision.cs:2415-2426`) leaves only
            // the residual that fit on the blocked axis (`result.X = …`), so a
            // drop stops dead against a wall instead of bouncing back (the
            // port's zero-residual stand-in, both wet and dry).
            self.vel.x = 0.0;
        let ny = self.pos.y + self.vel.y;
        if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
            let flush = if self.vel.y > 0.0 {
                let ty = ((ny + SIZE) / TILE_PX).floor() as i32;
                ty as f32 * TILE_PX - SIZE
            } else {
                let ty = (ny / TILE_PX).floor() as i32;
                (ty + 1) as f32 * TILE_PX
            };
            // `Collision.cs:2394`/`:2444`: `result.Y` collapses to the flush
            // residual — no restitution, BOTH arms (the C# runs the same
            // `TileCollision` at `:1051`/`:1064`). `ApplyMovement` steps the
            // drop flush (`:1078` wet via the `:1056-1059` wetVelocity copy,
            // `:1082` dry at full velocity). W4-e19 round 3: the old dry
            // snap-and-zero stand-in is retired — a landing tick carries the
            // residual as velocity; at flush rest the residual is 0, so the
            // resting drop still reads vel 0 after each tick.
            self.vel.y = flush - self.pos.y;
        // `:1052-1059` — the axes the collision changed refresh `wetVelocity`
        // to the new (full) value; the free axes keep the scaled head value.
        if self.vel.x != pre.x {
            wet_vel.x = self.vel.x;
        if self.vel.y != pre.y {
            wet_vel.y = self.vel.y;
        // `ApplyMovement` (`:1074-1084`): a wet drop integrates `wetVelocity`,
        // a dry one the full velocity. The blocked-Y arm's residual steps the
        // drop flush either way (the wet copy above put it in `wet_vel`).
        if wet_now {
            self.pos += wet_vel;
        } else {
            self.pos += self.vel;
        }
        // `:1036-1045` — the dry clear + the wetCount decrement.
        if !self.wet {
            self.lava_wet = false;
            self.honey_wet = false;
            self.shimmer_wet = false;
        }
        if self.wet_count > 0 {
            self.wet_count -= 1;
        }
        // `:1046-1065` — the tile collision with ignoreAetheriumPlatforms.
        // The wet arm copies the blocked axes into wetVelocity; the dry arm
        // just collides.
        let pre = self.vel;
        let tc = crate::collision::tile_collision_opts(
            world,
            [self.pos.x, self.pos.y],
            [self.vel.x, self.vel.y],
            SIZE,
            SIZE,
            false,
            false,
            true,
        );
        self.vel = Vec2::new(tc.vel[0], tc.vel[1]);
        if self.wet {
            if self.vel.x != pre.x {
                wet_vel.x = self.vel.x; // :1052-1055
            }
            if self.vel.y != pre.y {
                wet_vel.y = self.vel.y; // :1056-1059
            }
        }
        // `ApplyMovement` (`:1074-1084`): wet integrates wetVelocity, dry
        // the full velocity.
        if self.wet {
            self.pos += *wet_vel;
        } else {
            self.pos += self.vel;
        }
        // `:1067-1071` — the slope pass (ignoreAetheriumPlatforms true).
        let sc = crate::collision::slope_collision_opts(
            world,
            [self.pos.x, self.pos.y],
            [self.vel.x, self.vel.y],
            SIZE,
            SIZE,
            false,
            true,
        );
        self.pos = Vec2::new(sc.pos[0], sc.pos[1]);
        self.vel = Vec2::new(sc.vel[0], sc.vel[1]);
    }

    /// The armed splash's dust + sound arms (`WorldItem.cs:886-954` entry /
    /// `:965-1033` exit, doc-sweep lane 36 — the cited skip retired). The
    /// items pass calls this right after [`Self::update`] with the
    /// `UpdateItems` stream (`Main.cs:18024`'s swap — the C# arms draw
    /// `Main.rand` inline inside `MoveInWorld`). Verbatim per-kind bodies:
    /// * shimmer: 10x dust 308 at `(pos.X-6, pos.Y + height/2 - 8)` box
    ///   `(width+12, 24)`, each `vy -= 4`, `vx *= 2.5`, `scale = 0.8`,
    ///   `noGravity`, then ONE `Next(6)` colour pick (0 → (255,255,210),
    ///   1 → (190,245,255), 2 → (255,150,255), default → (190,175,255));
    ///   `PlaySound(19, x, y, 4)` entry (`:913`) / style 5 exit (`:992`).
    /// * honey: 5x dust 152, `vy -= 1`, `vx *= 2.5`, `scale = 1.3`,
    ///   `alpha = 100`, `noGravity`; `PlaySound(19, x, y)` (style 1).
    /// * water: 10x `Dust.dustWater()` at the same box (exit: NO −8 on Y,
    ///   `:1011`), `vy -= 4`, `vx *= 2.5`, `scale *= 0.8`, `alpha = 100`,
    ///   `noGravity`; style-1 cue.
    /// * lava: 5x dust 35, `vy -= 1.5`, `vx *= 2.5`, `scale = 1.3`,
    ///   `alpha = 100`, `noGravity`; style-1 cue.
    /// The per-dust `NewDust` draws (position pair, velocity pair, frame
    /// `Next(3)`, scale `Next(-20,21)`, the pressure bands) ride the pool
    /// port; the type-308/33/35/152 tweak chains run inside it. The colour
    /// `Next(6)` draws AFTER the spawn — a culled spawn (sentinel) still
    /// spends its colour draw, the C# writes into dust[6000] verbatim.
    /// `water_style` is `Main.waterStyle` (the caller feeds the live
    /// biome style); the sound draws ride the same stream AFTER the dust
    /// loop (the case-19 pitch roll, `LegacySoundPlayer.cs:881-888`).
    pub fn splash(
        &mut self,
        dust: &mut tr_core::particles::DustPool,
        ctx: &tr_core::particles::SpawnCtx,
        world: &World,
        audio: &mut crate::audio::Audio,
        rng: &mut tr_core::unified_random::UnifiedRandom,
        water_style: i32,
    ) {
        let Some(s) = self.splash_due.take() else {
            return;
        };
        use tr_core::particles::Color;
        // The dust box anchor (`:892` etc.): height/2 is the INTEGER half
        // (16/2 = 8, keep-one 144); the exit water arm drops the −8 (`:1011`).
        let half = SIZE as i32 / 2;
        let (count, dtype, dy, scale_mul, scale_set, alpha, y_fix): (
            i32, i32, f32, bool, f32, i32, f32,
        ) = if s.lava {
            (5, 35, -1.5, false, 1.3, 100, -8.0) // :944-953 / :1023-1032
        } else if s.shimmer {
            (10, 308, -4.0, false, 0.8, 0, -8.0) // :890-913 / :969-992
        } else if s.honey {
            (5, 152, -1.0, false, 1.3, 100, -8.0) // :917-926 / :996-1005
        } else {
            let y_fix = if s.exit { 0.0 } else { -8.0 }; // :1011 vs :932
            (10, tr_core::particles::dust_water(water_style), -4.0, true, 0.8, 100, y_fix)
        };
        let anchor = Vec2::new(s.pos.x - 6.0, s.pos.y + half as f32 + y_fix);
        for _ in 0..count {
            let i = dust.new_dust(
                rng, ctx, world, anchor, SIZE as i32 + 12, 24, dtype,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(i);
            d.velocity.y += dy;
            d.velocity.x *= 2.5;
            if scale_mul {
                d.scale *= scale_set; // the water arm's `scale *= 0.8f`
            } else {
                d.scale = scale_set; // the others SET the scale
            }
            if alpha != 0 {
                // The shimmer arm sets NO alpha (`:890-896`) — the C# leaves
                // NewDust's default 0; the others write 100 explicitly.
                d.alpha = alpha;
            }
            d.no_gravity = true;
            if s.shimmer {
                // `:897-911` / `:976-990` — the colour pick rides AFTER the
                // spawn (one Next(6) per dust, sentinel included).
                d.color = match rng.next_max(6) {
                    0 => Color { r: 255, g: 255, b: 210, a: 255 },
                    1 => Color { r: 190, g: 245, b: 255, a: 255 },
                    2 => Color { r: 255, g: 150, b: 255, a: 255 },
                    _ => Color { r: 190, g: 175, b: 255, a: 255 },
                };
            }
        }
        // The cue: shimmer entry style 4 (`:913`), exit style 5 (`:992`);
        // every other arm the 3-arg overload's default style 1
        // (`SoundEngine.cs:102`).
        let style = if s.shimmer { if s.exit { 5 } else { 4 } } else { 1 };
        audio.play_splash_sound(style, s.pos.x as i32 as f32, s.pos.y as i32 as f32, rng);
    }

    /// Draws the drop in world space. Vanilla has NO positional bob —
    /// `Main.DrawItem` (`Main.cs:38602-38788`) draws at the resting position
    /// with the velocity spin `rotation = velocity.X * 0.2f` (`:38613`),
    /// suppressed on a conveyor (`item.onConveyor`) or after a shimmer
    /// transform (`:38614-38617`). The rotation pivots on the frame centre
    /// (`vector = frame.Size()/2`, `:38610`). Returns the quad count.
    ///
    /// W4-g15 lane 5 — the draw-side family the single-quad pass skipped:
    /// * the shimmer colour fade (`:38621-38634`): a `shimmered` drop draws
    ///   `255*(1-shimmerTime)` on ALL channels; a ramping one scales its lit
    ///   colour by `(1-shimmerTime)` (the per-channel byte truncation),
    /// * the aura echo copies (`:38667-38721`): the boss bags / the fallen
    ///   star / the mana-cloak star orbit tinted ghost copies (the
    ///   `timeSinceItemSpawned/240 + GlobalTimeWrappedHourly*0.04` phase, the
    ///   `%4`/`%5` triangle radius pulse),
    /// * the mana-crystal pulse (`:38722-38726`): 1522-1527/3643 draw
    ///   `(250,250,250, mouseTextColor/2)` at `mouseTextColor/1000 + 0.8`,
    /// * the shimmered additive second copy (`:38732-38735`): alpha 0 rides
    ///   the premultiplied pipeline as additive (the pylon-ring convention).
    /// `mouse_text_color` is `Main.mouseTextColor`; `anim_time % 3600` is
    /// `Main.GlobalTimeWrappedHourly` (the caller folds it).
    pub fn draw(
        &self,
        r: &mut tr_render::SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        style: &LightStyle,
        anim_time: f32,
        mouse_text_color: u8,
    ) -> u32 {
        let uv = if let Some((uv, _, _)) = lookups.item_icon(self.kind) {
            Some(uv)
        } else if let Some(coin) = registry::coin_index(self.kind) {
            lookups.coin_uv(coin)
        } else if registry::projectile(self.kind).is_some() {
            lookups.shuriken_uv()
        } else if let Some(wall) = registry::place_wall(self.kind) {
            lookups.wall_icon_uv(wall)
        } else if let Some(tile) = registry::place_tile(self.kind) {
            lookups.plain_tex(tile).map(|t| t.uv)
        } else {
            None
        };
        let Some(uv) = uv else {
            return 0;
        };
        let tx = (self.pos.x / TILE_PX) as i32;
        let ty = (self.pos.y / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        // `Main.cs:38613-38617` — the spin, gated by conveyor/shimmered.
        let rot = if self.on_conveyor || self.shimmered {
            0.0
        } else {
            self.vel.x * 0.2
        };
        // The visual quad is the 14 px stand-in centred in the 16 px hitbox
        // (`pos + 1`, `SIZE - 2`) — the box-size fix (keep-one 144, the e19
        // round-3 fold) is physics-only, the pixels stay where the pre-fix
        // draw put them (the C# anchors by the frame size,
        // `Main.cs:38610-38612`, an unported per-frame stand-in).
        const VIS: f32 = SIZE - 2.0;
        if rot == 0.0 {

        }        // `Main.cs:38621-38634` — the shimmer colour fade (the byte
        // truncations ride `shimmer_fade_color`).
        let mut col = shimmer_fade_color(c, self.shimmered, self.shimmer_time);
        // `Main.cs:38722-38726` — the mana crystals (1522-1527) and the
        // Celestial Starboard 3643 pulse. The scale rides the quad size +
        // the rotation origin (the C#'s `Draw(..., scale)` about the frame
        // centre).
        let mut scale = 1.0f32;
        if matches!(self.kind, 1522..=1527 | 3643) {
            let (c2, s2) = mana_crystal_pulse(mouse_text_color, self.shimmer_time);
            col = c2;
            scale = s2;
        }
        let mut quads = 0u32;
        // `Main.cs:38667-38721` — the aura echo copies, drawn BEFORE the
        // main sprite (the offsets/tints ride `aura_echo_offsets`).
        let anchor = [self.pos.x + 1.0 + (SIZE - 2.0) * 0.5, self.pos.y + 1.0 + (SIZE - 2.0) * 0.5];
        for (off, col4) in aura_echo_offsets(self.kind, self.age_ticks, anim_time % 3600.0) {
            let pos = [
                anchor[0] + off[0] - (SIZE - 2.0) * 0.5 * scale,
                anchor[1] + off[1] - (SIZE - 2.0) * 0.5 * scale,
            ];
            r.push_sprite_origin_rot(
                pos,
                [(SIZE - 2.0) * scale, (SIZE - 2.0) * scale],
                uv,
                col4,
                rot,
                [(SIZE - 2.0) * 0.5 * scale, (SIZE - 2.0) * 0.5 * scale],
            );
            quads += 1;
        if self.kind == 4143 {
            // `:38720` — the white 128-alpha copy at the anchor, BEFORE the
            // lit main draw.
                [anchor[0] - (SIZE - 2.0) * 0.5, anchor[1] - (SIZE - 2.0) * 0.5],
                [SIZE - 2.0, SIZE - 2.0],
                [1.0, 1.0, 1.0, 128.0 / 255.0],
                [(SIZE - 2.0) * 0.5, (SIZE - 2.0) * 0.5],
        // (`pos + 1`, `SIZE - 2`) — the box-size fix (W4-e19 round 3) is
        // physics-only, the pixels stay where the pre-fix draw put them
        // (the C# anchors by the frame size, `Main.cs:38610-38612`, an
        // unported per-frame stand-in). The g15 draw family (fade/echo/
        // pulse/additive) rides ON the VIS quad — the 347 §2.2 compose:
        // e19's 16-box geometry + g15's DrawItem arms, the scale multiplies
        // the visual size.
        if rot == 0.0 && scale == 1.0 {
            r.push_sprite(
                [self.pos.x + 1.0, self.pos.y + 1.0],
                [VIS, VIS],
                uv,
                col,
            );
        } else {
            r.push_sprite_origin_rot(
                [self.pos.x + 1.0, self.pos.y + 1.0],
                [VIS, VIS],
                [VIS * scale, VIS * scale],
                uv,
                col,
                rot,
                [VIS * 0.5, VIS * 0.5],
                [VIS * 0.5 * scale, VIS * 0.5 * scale],
            );
        }
        quads += 1;
        // `Main.cs:38732-38735` — the shimmered additive copy: the same
        // tint with A=0 (the premultiplied pipeline adds it, the pylon
        // glow-ring convention).
        if self.shimmered {
            let add = [col[0], col[1], col[2], 0.0];
            if rot == 0.0 && scale == 1.0 {
                r.push_sprite([self.pos.x + 1.0, self.pos.y + 1.0], [VIS, VIS], uv, add);
            } else {
                r.push_sprite_origin_rot(
                    [self.pos.x + 1.0, self.pos.y + 1.0],
                    [VIS * scale, VIS * scale],
                    uv,
                    add,
                    rot,
                    [VIS * 0.5 * scale, VIS * 0.5 * scale],
                );
            }
            quads += 1;
        }
        quads
    }
}

/// `Main.DrawItem`'s shimmer colour fade (`Main.cs:38621-38634`), W4-g15
/// lane 5. The C# writes BYTES: a `shimmered` drop replaces every channel
/// with `(byte)(255f * (1 - shimmerTime))`; a ramping one scales the lit
/// colour per channel with the same truncation. The port's tint is the
/// 0..1 float the pipeline multiplies — the byte round-trip keeps the
/// truncation (`(byte)(c*255)` then `*(1-t)` then `/255`). `lit` is the
/// shaded RGB; the alpha stays 1 (the C# `GetAlpha` colour's A is 255 for
/// a plain drop — the `item.alpha` rows are the unported GetAlpha family).
pub(crate) fn shimmer_fade_color(lit: [f32; 3], shimmered: bool, shimmer_time: f32) -> [f32; 4] {
    if shimmered {
        let v = (255.0 * (1.0 - shimmer_time)) as u8 as f32 / 255.0;
        [v, v, v, v]
    } else if shimmer_time > 0.0 {
        let mut col = [lit[0], lit[1], lit[2], 1.0];
        for ch in col.iter_mut() {
            let b = (*ch * 255.0) as u8;
            *ch = (b as f32 * (1.0 - shimmer_time)) as u8 as f32 / 255.0;
        }
        col
    } else {
        [lit[0], lit[1], lit[2], 1.0]
    }
}

/// `Main.DrawItem`'s mana-crystal pulse (`Main.cs:38722-38726`), W4-g15
/// lane 5: kinds 1522-1527/3643 draw `(250, 250, 250, mouseTextColor/2)`
/// (integer division) at scale `(int)mouseTextColor/1000 + 0.8`; a live
/// shimmer ramp scales all four channels by `(1 - shimmerTime)` with the
/// per-channel byte truncation (`:38724`). Returns (colour, scale).
pub(crate) fn mana_crystal_pulse(mouse_text_color: u8, shimmer_time: f32) -> ([f32; 4], f32) {
    let v = 250.0 / 255.0;
    // `Main.mouseTextColor / 2` — INTEGER division (`:38724`).
    let half = (mouse_text_color / 2) as f32;
    let a = half / 255.0;
    let col = if shimmer_time > 0.0 {
        let f = 1.0 - shimmer_time;
        let vb = (250.0 * f) as u8 as f32 / 255.0;
        let ab = (half * f) as u8 as f32 / 255.0;
        [vb, vb, vb, ab]
    } else {
        [v, v, v, a]
    };
    let scale = (mouse_text_color as i32 as f32) / 1000.0 + 0.8;
    (col, scale)
}

/// `Main.DrawItem`'s aura echo copies (`Main.cs:38667-38721`), W4-g15
/// lane 5: the boss bags (`ItemID.Sets.BossBag`), the fallen star 75 and
/// the mana-cloak star 4143 orbit tinted ghost copies about the draw
/// anchor. The phase is `timeSinceItemSpawned/240 +
/// GlobalTimeWrappedHourly*0.04`; the radius pulse is the `%period/2`
/// triangle folded to 0.5..1. Each copy sits at
/// `(0, radius).RotatedBy((k + phase) * 2π) * pulse` with a FIXED colour
/// (no light). Returns (offset, colour) pairs in draw order — the caller
/// adds the anchor and spins each copy with the main sprite's `rot`.
/// `gtw` is `Main.GlobalTimeWrappedHourly` (`anim_time % 3600`).
pub(crate) fn aura_echo_offsets(kind: u16, age_ticks: i32, gtw: f32) -> Vec<([f32; 2], [f32; 4])> {
    // (radius, k-step, pulse period, tint) per ring, in the C# draw order.
    let rings: &[(f32, f32, f32, [u8; 4])] = if crate::entity_lights::BOSS_BAG.contains(&kind) {
        // `:38676-38683` — 4 copies at radius 8 step 0.25, then 3 at
        // radius 4 step 0.34; the `%4` triangle.
        &[
            (8.0, 0.25, 4.0, [90, 70, 255, 50]),
            (4.0, 0.34, 4.0, [140, 120, 255, 77]),
        ]
    } else if kind == 75 {
        // `:38694-38701` — the fallen star: the same two rings, the `%5`
        // triangle, the blue tints.
        &[
            (8.0, 0.25, 5.0, [50, 50, 255, 50]),
            (4.0, 0.34, 5.0, [120, 120, 255, 127]),
        ]
    } else if kind == 4143 {
        // `:38712-38719` — the mana-cloak star: TWO 0.34 rings (8 then 4),
        // the `%5` triangle (the white 128-alpha main copy is the caller's).
        &[
            (8.0, 0.34, 5.0, [30, 30, 155, 60]),
            (4.0, 0.34, 5.0, [60, 60, 127, 57]),
        ]
    } else {
        return Vec::new();
    };
    let phase = age_ticks as f32 / 240.0 + gtw * 0.04;
    let mut out = Vec::with_capacity(7);
    for &(radius, step, period, tint) in rings {
        let mut pulse = gtw % period / 2.0;
        if pulse >= 1.0 {
            pulse = 2.0 - pulse;
        }
        pulse = pulse * 0.5 + 0.5;
        let col = [
            tint[0] as f32 / 255.0,
            tint[1] as f32 / 255.0,
            tint[2] as f32 / 255.0,
            tint[3] as f32 / 255.0,
        ];
        // The C# `for (float k = 0f; k < 1f; k += step)` — the f32
        // accumulation is load-bearing (0.34 lands 3 copies).
        let mut k = 0.0f32;
        while k < 1.0 {
            // `(num6 + num4) * ((float)Math.PI * 2f)` — the f32 sum times
            // the f32 2π; `RotatedBy` takes the double angle and casts its
            // cos/sin to f32 (`Utils.cs:1675-1684`).
            let ang = ((k + phase) * (std::f32::consts::PI * 2.0)) as f64;
            // `(0, radius).RotatedBy(ang)`: `(-sin·r, cos·r)`.
            let off = [
                -(ang.sin() as f32) * radius * pulse,
                (ang.cos() as f32) * radius * pulse,
            ];
            out.push((off, col));
            k += step;
        }
    }
    out
}

/// Hash of the item list for the G6 world-range key; includes the animation
/// step and the velocity spin's source (`Main.cs:38613`'s `velocity.X *
/// 0.2`) so a rolling drop redraws.
///
/// W4-g15 lane 5 — the new draw inputs ride the key ONLY when a drop
/// consumes them (the waterfall/vine zero-when-absent convention):
/// * `shimmer_time`'s bits while a fade is live (`:38621-38634`),
/// * the aura families' `age_ticks` + the full `GlobalTimeWrappedHourly`
///   (`anim_time % 3600`, the `:38669` phase / radius pulse) — a boss bag /
///   fallen star / mana-cloak star on screen animates every frame, so the
///   world range rebuilds every frame while one lies in view;
/// * the mana-crystal pulse reads `mouseTextColor`, already covered by
///   `light_params` (`LightParams::identity` hashes it).
pub fn render_key(items: &[ItemDrop], anim_step: u32, anim_time: f32) -> u32 {
    let mut h = items.len() as u32 ^ (anim_step << 24);
    let mut aura = false;
    for it in items {
        h = h.wrapping_mul(31).wrapping_add(it.kind as u32);
        h = h.wrapping_mul(31).wrapping_add(it.pos.x.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.pos.y.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.count);
        h = h.wrapping_mul(31).wrapping_add(it.vel.x.to_bits());
        if it.shimmered || it.shimmer_time > 0.0 {
            h = h.wrapping_mul(31).wrapping_add(it.shimmer_time.to_bits());
        }
        if it.kind == 75 || it.kind == 4143 || crate::entity_lights::BOSS_BAG.contains(&it.kind) {
            aura = true;
            h = h.wrapping_mul(31).wrapping_add(it.age_ticks as u32);
        }
    }
    if aura {
        h = h.wrapping_mul(31).wrapping_add((anim_time % 3600.0).to_bits());
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// W4-g15 lane 5 — `Main.DrawItem`'s shimmer fade (`Main.cs:38621-
    /// 38634`): a shimmered drop replaces ALL channels with the byte
    /// `(byte)(255*(1-t))`; a ramping drop scales the lit colour per
    /// channel with the same truncation. At t=0.5 the shimmered colour is
    /// `(byte)127.5 = 127` on every channel; a lit 1.0 channel ramps
    /// `(byte)(255*0.5) = 127` too, a lit 0.5 channel `(byte)127.5 = 127`
    /// then `(byte)63.5 = 63`.
    #[test]
    fn w4_g15_the_shimmer_fade_truncates_per_channel() {
        let c = shimmer_fade_color([1.0, 0.5, 0.0], true, 0.5);
        assert_eq!(c, [127.0 / 255.0; 4], "the shimmered flat fade");
        let c = shimmer_fade_color([1.0, 0.5, 0.0], false, 0.5);
        assert_eq!(c[0], 127.0 / 255.0);
        assert_eq!(c[1], 63.0 / 255.0);
        assert_eq!(c[2], 0.0);
        assert_eq!(c[3], 1.0, "the ramp keeps A=1");
        // t=0: the lit colour passes through untouched.
        let c = shimmer_fade_color([0.2, 0.4, 0.6], false, 0.0);
        assert_eq!(c, [0.2, 0.4, 0.6, 1.0]);
        // t=1: the shimmered drop is fully transparent black.
        assert_eq!(shimmer_fade_color([1.0; 3], true, 1.0), [0.0; 4]);
    }

    /// W4-g15 lane 5 — the mana-crystal pulse (`Main.cs:38722-38726`):
    /// `(250,250,250, mouseTextColor/2)` at `mouseTextColor/1000 + 0.8`.
    /// At 225: alpha `(byte)112` (225/2 integer division), scale 1.025.
    /// The shimmer ramp scales all four channels (`:38724`).
    #[test]
    fn w4_g15_the_mana_crystal_pulse_reads_mouse_text_color() {
        let (c, s) = mana_crystal_pulse(225, 0.0);
        assert_eq!(c[0], 250.0 / 255.0);
        assert_eq!(c[3], 112.0 / 255.0, "225/2 = 112 (int division)");
        assert!((s - 1.025).abs() < 1e-6, "225/1000 + 0.8");
        let (c, s) = mana_crystal_pulse(190, 0.0);
        assert_eq!(c[3], 95.0 / 255.0);
        assert!((s - 0.99).abs() < 1e-6);
        // The shimmer ramp at t=0.5: RGB (byte)125, A (byte)(112.5*0.5)=56.
        let (c, _) = mana_crystal_pulse(225, 0.5);
        assert_eq!(c[0], 125.0 / 255.0);
        assert_eq!(c[3], 56.0 / 255.0);
    }

    /// W4-g15 lane 5 — the aura echoes (`Main.cs:38667-38721`): the boss
    /// bag draws 4+3 copies, the fallen star 4+3, the mana-cloak star 3+3;
    /// every other kind draws none. At gtw=0/age=0 the phase is 0 and the
    /// pulse is 0.5 (`0%4/2 = 0 → 0*0.5+0.5`), so the k=0 copy of the
    /// radius-8 ring sits at `(0, 8*0.5) = (0, 4)` — straight DOWN from
    /// the anchor (`(0,r).RotatedBy(0) = (0,r)`).
    #[test]
    fn w4_g15_the_aura_echo_rings_orbit_the_anchor() {
        let e = aura_echo_offsets(3318, 0, 0.0);
        assert_eq!(e.len(), 7, "the boss bag's 4 + 3 copies");
        assert!((e[0].0[0]).abs() < 1e-6);
        assert!((e[0].0[1] - 4.0).abs() < 1e-6, "k=0 at (0, 8*0.5)");
        assert_eq!(e[0].1, [90.0 / 255.0, 70.0 / 255.0, 1.0, 50.0 / 255.0]);
        // The k=0.25 copy: angle π/2 → (-8*0.5, 0) = (-4, 0) (left).
        assert!((e[1].0[0] + 4.0).abs() < 1e-4);
        assert!(e[1].0[1].abs() < 1e-4);
        // The second ring's first copy: radius 4 → (0, 2).
        assert!((e[4].0[1] - 2.0).abs() < 1e-6);
        assert_eq!(e[4].1, [140.0 / 255.0, 120.0 / 255.0, 1.0, 77.0 / 255.0]);
        // The fallen star: the same shape, the blue tints, the %5 pulse.
        let e = aura_echo_offsets(75, 0, 0.0);
        assert_eq!(e.len(), 7);
        assert_eq!(e[0].1, [50.0 / 255.0, 50.0 / 255.0, 1.0, 50.0 / 255.0]);
        // The mana-cloak star: 3 + 3 (both rings step 0.34).
        let e = aura_echo_offsets(4143, 0, 0.0);
        assert_eq!(e.len(), 6);
        assert_eq!(e[0].1, [30.0 / 255.0, 30.0 / 255.0, 155.0 / 255.0, 60.0 / 255.0]);
        // A plain drop: no echoes.
        assert!(aura_echo_offsets(2, 0, 0.0).is_empty());
        // The pulse triangle: gtw=2 → 2%4/2 = 1 → folded 2-1 = 1 →
        // 1*0.5+0.5 = 1.0, so the k=0 copy sits at (0, 8).
        let e = aura_echo_offsets(3318, 0, 2.0);
        assert!((e[0].0[1] - 8.0).abs() < 1e-4, "the pulse peak");
        // The age phase: age_ticks=240 → phase 1.0 → the k=0 copy wraps
        // the full turn back to (0, 4) at gtw=0.
        let e = aura_echo_offsets(3318, 240, 0.0);
        assert!((e[0].0[0]).abs() < 1e-4);
        assert!((e[0].0[1] - 4.0).abs() < 1e-4);
    }

    /// W4-g15 lane 5 — the G6 key coverage: an aura drop's `age_ticks` and
    /// the wrapped clock ride `render_key` (the echoes animate every
    /// frame); a plain-drop world's key ignores the clock (the
    /// zero-when-absent convention keeps the old cache behaviour).
    #[test]
    fn w4_g15_the_render_key_covers_the_aura_inputs_only_when_present() {
        let plain = vec![ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0)];
        assert_eq!(
            render_key(&plain, 0, 1.0),
            render_key(&plain, 0, 2.0),
            "a plain world ignores the clock"
        );
        let mut star = ItemDrop::new(75, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        let aura = vec![star.clone()];
        assert_ne!(
            render_key(&aura, 0, 1.0),
            render_key(&aura, 0, 2.0),
            "the fallen star's echoes read the clock"
        );
        star.age_ticks = 5;
        assert_ne!(
            render_key(&aura, 0, 1.0),
            render_key(&[star], 0, 1.0),
            "the age phase rides the key"
        );
        // The shimmer fade rides the key while live.
        let mut sh = ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        sh.shimmered = true;
        sh.shimmer_time = 0.5;
        let k1 = render_key(&[sh.clone()], 0, 1.0);
        sh.shimmer_time = 0.4;
        assert_ne!(k1, render_key(&[sh], 0, 1.0));
    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, name: "", count, prefix: 0 }
    }

    #[test]
    fn w4_g79_find_paint_or_coating_prefers_the_ammo_slots() {
        // `Player.cs:41927-41944`: the FIRST stocked PaintOrCoating in
        // 54..57 wins over anything in the grid; else the first over 0..58.
        let mut inv = [EMPTY_SLOT; INV];
        let mut equip = crate::inv::EquipSlots::empty();
        // Red Paint 1073 in grid slot 2, Glow Paint 4668 in ammo slot 1.
        inv[2] = slot(1073, 50);
        equip.ammo[1] = slot(4668, 99);
        assert_eq!(
            find_paint_or_coating(&inv, &equip).map(|s| s.kind),
            Some(4668),
            "the ammo-slot scan runs first"
        );
        // Ammo slot 0 beats ammo slot 1 (index order).
        equip.ammo[0] = slot(5344, 1);
        assert_eq!(find_paint_or_coating(&inv, &equip).map(|s| s.kind), Some(5344));
        // An EMPTY ammo slot does not shadow the grid pick.
        equip.ammo = [EMPTY_SLOT; crate::inv::AMMO];
        assert_eq!(find_paint_or_coating(&inv, &equip).map(|s| s.kind), Some(1073));
        // A zero-count grid slot is skipped; the coin row counts (index
        // 50..53 before the ammo re-scan).
        inv[2] = slot(1073, 0);
        equip.coins[3] = slot(1074, 7);
        assert_eq!(find_paint_or_coating(&inv, &equip).map(|s| s.kind), Some(1074));
        // Nothing painted anywhere: None (the C#'s null).
        equip.coins[3] = EMPTY_SLOT;
        inv[9] = slot(8, 99); // torches are not PaintOrCoating
        assert_eq!(find_paint_or_coating(&inv, &equip), None);
    }

    #[test]
    fn w4_g79_the_brush_and_roller_preview_offsets_are_verbatim() {
        // `Main.TryGetAmmo` (`Main.cs:45124-45135`): brush 1071/1543 at
        // (22, 22), roller 1072/1544 at (28, 28); the previewed item is the
        // FindPaintOrCoating pick; every other kind returns false.
        let mut inv = [EMPTY_SLOT; INV];
        let equip = crate::inv::EquipSlots::empty();
        inv[0] = slot(1073, 50);
        for (tool, offset) in [
            (1071u16, [22.0, 22.0]),
            (1543, [22.0, 22.0]),
            (1072, [28.0, 28.0]),
            (1544, [28.0, 28.0]),
        ] {
            let (item, off) =
                paint_tool_preview(tool, &inv, &equip).expect("the tool previews its paint");
            assert_eq!(item.kind, 1073);
            assert_eq!(off, offset, "tool {tool}");
        }
        assert_eq!(paint_tool_preview(8, &inv, &equip), None, "a torch: no arm");
        assert_eq!(paint_tool_preview(5324, &inv, &equip), None, "the wand arm unported");
        // No paint in the inventory: the arm returns false (`:45135`).
        inv[0] = EMPTY_SLOT;
        assert_eq!(paint_tool_preview(1071, &inv, &equip), None);
    }

    /// S1-E-21 wave A — the per-site cue selection: `Item.UseSound` rows
    /// (`Item.cs`'s `SetDefaults` cases + the `DefaultTo*` helpers) resolve
    /// through `SOUND_STYLES` to the legacy `(SoundId 2, Style)` pair, and
    /// the `SkipsInitialUseSound` gate keeps 2350/4870 silent at the use
    /// start (`Player.cs:51681-51682`, `ItemID.cs:408`).
    #[test]
    fn s1_e_21a_the_item_use_cue_selection_follows_the_vanilla_rows() {
        let style_of = |kind: u16| {
            let (idx, _) = initial_use_sound(kind).expect("the kind plays at the use start");
            let s = &crate::soundid::SOUND_STYLES[idx as usize];
            (s.sound_id, s.style, s.name)
        };
        // The Copper Pickaxe 1 (`Item.cs:1401`) and the Iron Broadsword 4
        // (`:1437`) → SoundID.Item1 (`SoundID.cs:1384`).
        assert_eq!(style_of(1), (2, 1, "Item1"));
        assert_eq!(style_of(4), (2, 1, "Item1"));
        // The Mushroom 5 (`Item.cs:1444`) → Item2 (`:1385`).
        assert_eq!(style_of(5), (2, 2, "Item2"));
        // The Lesser Healing Potion 28 — `DefaultToHealingPotion`'s Item3
        // (`Item.cs:48572`, `SoundID.cs:1386`).
        assert_eq!(style_of(28), (2, 3, "Item3"));
        // The Wooden Bow 39 — `DefaultToBow`'s Item5 (`Item.cs:48427`,
        // `SoundID.cs:1388`).
        assert_eq!(style_of(39), (2, 5, "Item5"));
        // The Storm Tiger Staff 4607 — its own case row's Item44
        // (`Item.cs:38142`, `SoundID.cs:1427`); the `DefaultToStaff` helper's
        // Item43 (`Item.cs:48021`) rides the 6171 correction below.
        assert_eq!(style_of(4607), (2, 44, "Item44"));
        // The whips — `DefaultToWhip`'s Item152 (`Item.cs:47819`,
        // `SoundID.cs:1536`).
        assert_eq!(style_of(4672), (2, 152, "Item152"));
        // Abigail's Flower 5114 → the AbigailSummon trackable
        // (`SoundID.cs:1699`).
        assert_eq!(style_of(5114).2, "AbigailSummon");
        assert_eq!(crate::soundid::SOUND_STYLES[440].sound_id, 42);
        // The hand-written 6171 correction rides the staff sound too
        // (`registry.rs`'s ITEM_CORRECTIONS, `Item.cs:47551`).
        assert_eq!(style_of(6171), (2, 43, "Item43"));
        // The Paladin's Hammer 1513 keeps its -0.5 pitch offset
        // (`Item.cs:18499`).
        let (_, pitch) = initial_use_sound(1513).unwrap();
        assert_eq!(pitch, -0.5);
        // The gate: 2350 (Recall Potion) / 4870 (Potion of Return) skip the
        // initial sound (`ItemID.cs:408`); the null rows play nothing —
        // the Compass 393 (`DefaultToVoiceOverrideAccessory`,
        // `Item.cs:47788`) and the golf clubs (`:48524`).
        assert_eq!(initial_use_sound(2350), None);
        assert_eq!(initial_use_sound(4870), None);
        assert_eq!(initial_use_sound(393), None);
        assert_eq!(initial_use_sound(4039), None);
        // The Dirt Block 2 has no UseSound row (`Item.cs:1406-1416`).
        assert_eq!(initial_use_sound(2), None);
    }

    #[test]
    fn s1_e_21a_every_use_sound_style_resolves_to_a_registered_cue() {
        // Every distinct `use_sound` index the generated table names must
        // land on a cue the bank registers: the id-2 family → `Item_<style>`
        // in SFX_CUES (plus 18/19, the style-1 remap targets), the id-42
        // trackables → the pinned first variation in ITEM_CUSTOM_CUES.
        for d in crate::itemdata::ITEMS.iter() {
            let Some(idx) = d.use_sound else { continue };
            let s = &crate::soundid::SOUND_STYLES[idx as usize];
            if s.sound_id == 42 {
                let base = s.trackable.unwrap();
                let cue = if s.variations == 1 {
                    base.to_string()
                } else {
                    format!("{base}_0")
                };
                assert!(
                    crate::audio::ITEM_CUSTOM_CUES.contains(&cue.as_str()),
                    "item {}'s trackable {cue} is unregistered",
                    d.id
                );
            } else {
                assert_eq!(s.sound_id, 2, "item {}'s style id", d.id);
                let cue = format!("Item_{}", s.style);
                assert!(
                    crate::audio::sfx_cue_registered(&cue),
                    "item {}'s cue {cue} is unregistered",
                    d.id
                );
            }
        }
    }

    /// W4-g25 batch 1 — the food/range-if sound provisions (10 sites).
    /// `Item.cs` has ZERO `PlaySound` calls; these pin the `UseSound` data
    /// rows the `SetFoodDefaults` gulp split and the four range-if blocks
    /// provision (the play call is wave A's `Player.cs:51681-51698` seam).
    /// WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_g25_batch1_the_food_and_range_if_sound_rows() {
        let row = |kind: u16| {
            let d = registry::get(kind).expect("the kind is registered");
            let style = d.use_sound.map(|i| crate::soundid::SOUND_STYLES[i as usize].name);
            (style, d.use_style, d.use_sound_pitch)
        };
        // 1. Ale 353 — `DefaultToFood(…, useGulpSound: true)` (`Item.cs:48380`):
        //    gulp → Item3 + useStyle 9 (`:48545`/`:48553`).
        assert_eq!(row(353), (Some("Item3"), 9, 0.0));
        // 2. Marshmallow 967 — NON-gulp `DefaultToFood` (`:48302`) → Item2,
        //    then the case re-styles to 9 AFTER (`:48304`) — the sound stays
        //    Item2 (the style write does not touch UseSound).
        assert_eq!(row(967), (Some("Item2"), 9, 0.0));
        // 3. Cooked Marshmallow 969 — the same non-gulp + post-style pair
        //    (`:48233-48235`).
        assert_eq!(row(969), (Some("Item2"), 9, 0.0));
        // 4. Rock Candy 5645 — gulp `DefaultToFood` (`:48371`) provisions
        //    Item3, then the case OVERRIDES to Item2 (`:48372`); useStyle 9
        //    from the gulp helper stands.
        assert_eq!(row(5645), (Some("Item2"), 9, 0.0));
        // 5. Eggnog 1912 — `IsFood` (`ItemID.cs:498`) routes to
        //    `SetFoodDefaults` (`Item.cs:48689`), whose case calls
        //    `DefaultToHealingPotion` (`:48387` → `:48572` Item3) — NOT the
        //    `SetDefaults2` case 1912 (`:20741`, unreachable for food).
        assert_eq!(row(1912), (Some("Item3"), 9, 0.0));
        // 6. Golden Delight 4022 — non-gulp `DefaultToFood` (`:48045`):
        //    Item2 + useStyle 2 (`:48549`/`:48557`), no post-override.
        assert_eq!(row(4022), (Some("Item2"), 2, 0.0));
        // 7. Life Hair Dye 1977 — the range-if `type >= 1977 && type <= 1986`
        //    (`:21235`) provisions Item3 + useStyle 9 (`:21253-21254`); the
        //    case-label group 1970-1976 (`:21217-21233`) is the gemspark
        //    blocks, NOT this block (1976 stays silent).
        assert_eq!(row(1977), (Some("Item3"), 9, 0.0));
        assert_eq!(row(1976).0, None);
        // 8. Fleshcatcher 2421 — the range-if `type >= 2421 && type <= 2422`
        //    (`:23074`) provisions Item1 (`:23081`), useStyle 1 (`:23076`).
        assert_eq!(row(2421), (Some("Item1"), 1, 0.0));
        // 9. Rockfish 2320 — the `if (type == 2320)` block (`:23099`)
        //    provisions Item1 (`:23112`), useStyle 1 (`:23105`).
        assert_eq!(row(2320), (Some("Item1"), 1, 0.0));
        // 10. Flaming Mace 5012 — the shared 5011/5012 case (`:40333-40334`)
        //     provisions Item1 (`:40351`), useStyle 5 (`:40336`); the
        //     `if (type == 5012)` arm (`:40346`) only swaps the shoot.
        assert_eq!(row(5012), (Some("Item1"), 5, 0.0));
        // The provisions draw NOTHING: `initial_use_sound` is a pure table
        // read (no RNG parameter), and the pitch column is 0 everywhere in
        // this batch (the only nonzero pitch is 1513, wave A's pin).
        for kind in [353u16, 967, 969, 5645, 1912, 4022, 1977, 2421, 2320, 5012] {
            let (_, pitch) = initial_use_sound(kind).expect("the batch plays at the use start");
            assert_eq!(pitch, 0.0, "kind {kind}");
        }
    }

    /// W4-g25 batch 2 — the null/trackable/deprecated/override sound
    /// provisions (10 sites). The explicit `UseSound = null` rows, the
    /// non-`ItemN` style names, the `Deprecated` → air zeroing, and the
    /// post-helper overrides. WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_g25_batch2_the_null_trackable_deprecated_and_override_rows() {
        let row = |kind: u16| {
            let d = registry::get(kind).expect("the kind is registered");
            let style = d.use_sound.map(|i| crate::soundid::SOUND_STYLES[i as usize].name);
            (style, d.use_style, d.use_sound_pitch)
        };
        // 1. Medusa Head 3269 — explicit `UseSound = null` (`Item.cs:30324`),
        //    useStyle 4 (`:30314`): the use start is silent.
        assert_eq!(row(3269), (None, 4, 0.0));
        assert_eq!(initial_use_sound(3269), None);
        // 2. Zenith 4956 — explicit null (`:40004`), useStyle 1 (`:40001`).
        assert_eq!(row(4956), (None, 1, 0.0));
        // 3. Portal Gun 3384 — explicit null (`:30958`), useStyle 5.
        assert_eq!(row(3384), (None, 5, 0.0));
        // 4. Leaf Blower 1178 — `SoundID.LeafBlower` (`:14469`), a NAMED
        //    id-2 style: `(2, 34).WithVolume(0.13)` (`SoundID.cs:1726`),
        //    NOT a trackable. The volume override rides the style row.
        let leaf = &crate::soundid::SOUND_STYLES[registry::get(1178).unwrap().use_sound.unwrap() as usize];
        assert_eq!((leaf.sound_id, leaf.style, leaf.trackable), (2, 34, None));
        assert_eq!(leaf.volume, 0.13);
        // 5. Palworld Pet Chillet 5665 — `SoundID.PalSummon` (`:44514`), a
        //    TRACKABLE (id 42, `CreateTrackable("pal_summon").WithVolume(0.15)`,
        //    `SoundID.cs:1712`); useStyle 1 (`:44511`).
        let pal = &crate::soundid::SOUND_STYLES[registry::get(5665).unwrap().use_sound.unwrap() as usize];
        assert_eq!((pal.sound_id, pal.trackable), (42, Some("pal_summon")));
        assert_eq!(pal.volume, 0.15);
        assert_eq!(row(5665).1, 1);
        // 6. Glacier Fang 6153 — `SoundID.GlacierFang_Charging` (`:47431`),
        //    a NAMED id-2 style `(2, 210)` (`SoundID.cs:1594`), useStyle 5.
        let glacier = &crate::soundid::SOUND_STYLES[registry::get(6153).unwrap().use_sound.unwrap() as usize];
        assert_eq!((glacier.sound_id, glacier.style, glacier.trackable), (2, 210, None));
        assert_eq!(row(6153).1, 5);
        // 7. Vortex Axe 2772 — the case provisions Item1 (`:26478`) BUT the
        //    id is `Deprecated` (`ItemID.cs:1316`), so the epilogue
        //    `TurnToAir` (`Item.cs:48922-48925`) zeroes it: name "None",
        //    useStyle 0, no sound.
        assert_eq!(row(2772), (None, 0, 0.0));
        assert_eq!(registry::get(2772).unwrap().name, "None");
        // 8. First Fractal 4722 — same Deprecated → air zeroing (case
        //    `:38793` Item1, `ItemID.cs:1316`).
        assert_eq!(row(4722), (None, 0, 0.0));
        // 9. Whoopie Cushion 215 — `DefaultToVoiceOverrideAccessory` sets
        //    null (`:4049` → `:47788`), then the case OVERRIDES to Item16
        //    (`:4057`) + useStyle 10 (`:4056`). The override wins.
        assert_eq!(row(215), (Some("Item16"), 10, 0.0));
        // 10. Arc Surge 6173 — `DefaultToStaff` provisions Item43 (`:47556`
        //     → `:48021`), then the case OVERRIDES to Item15 (`:47560`);
        //     useStyle 5 from the staff helper stands.
        assert_eq!(row(6173), (Some("Item15"), 5, 0.0));
        // The deprecated ids answer None through the use-start gate too.
        assert_eq!(initial_use_sound(2772), None);
        assert_eq!(initial_use_sound(4722), None);
    }

    #[test]
    fn s1_f_10_item_slot_clone_and_refresh_snapshot() {
        // `Item.Clone`/`Item.Refresh` (`Item.cs:49837-49840`, `:49945-49961`,
        // S1-F-10): clone copies the instance snapshot; refresh re-applies the
        // kind's defaults and clamps to `maxStack` (CommonMaxStack 9999 for
        // unregistered ids).
        let slot = ItemSlot {
            kind: 2, // Dirt Block, max stack 9999
            name: "Dirt Block",
            count: 10,
            prefix: 0,
        };
        assert_eq!(slot.clone_item(), slot);
        let mut over = ItemSlot {
            kind: 6196, // unregistered -> CommonMaxStack 9999
            name: "Wrong",
            count: 12_000,
            prefix: 0,
        };
        over.refresh(false);
        assert_eq!(over.kind, 6196);
        assert_eq!(over.name, "Item");
        assert_eq!(over.count, 9999);
        let mut air = EMPTY_SLOT;
        air.refresh(false);
        assert_eq!(air, EMPTY_SLOT);
    }

    #[test]
    fn pickup_prefers_stacks_then_empty_slots() {
        // `Player.GetItem` (S1-P-i47): the occupied merge forward first,
        // then the usable-kind empty hotbar arm, then the REVERSE empty
        // grid scan (`Player.cs:39353-39394`).
        let mut inv: [ItemSlot; INV] =
            std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inv[0] = ItemSlot {
            kind: 3,
            name: "Stone Block",
            count: 4,
            prefix: 0,
        };
        assert_eq!(slot_for_pickup(&mut inv, 3, 0), Some(0));
        // The Copper Ore 12 places a tile (useStyle 1) — the empty hotbar
        // arm takes slot 1 (`:39365-39374`).
        assert_eq!(slot_for_pickup(&mut inv, 12, 0), Some(1));
        assert_eq!(inv[1].kind, 12);
        assert_eq!(inv[1].name, "Copper Ore");
        assert_eq!(inv[1].count, 0);
        for slot in &mut inv {
            slot.count = 1;
        }
        assert_eq!(
            slot_for_pickup(&mut inv, 53, 0),
            None,
            "full hotbar must refuse"
        );
        // A useStyle-0 kind (the Lens 38) skips the hotbar arm and lands on
        // the LAST empty grid slot (`:39387-39394`'s reverse scan).
        let mut inv: [ItemSlot; INV] = [EMPTY_SLOT; INV];
        assert_eq!(slot_for_pickup(&mut inv, 38, 0), Some(INV - 1));
        // A FULL same-kind stack never answers the merge pass (`:39487`'s
        // `stack < maxStack`); the drop opens a new slot instead.
        let mut inv: [ItemSlot; INV] = [EMPTY_SLOT; INV];
        inv[0] = ItemSlot { kind: 3, name: "Stone Block", count: registry::max_stack(3), prefix: 0 };
        let i = slot_for_pickup(&mut inv, 3, 0).unwrap();
        assert_ne!(i, 0, "the full stack refuses the merge");
    }

    pub(super) fn ground() -> World {
        let mut w = World::new("items".into(), 32, 32);
        for x in 0..32 {
            for y in 20..32 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn registry_maps_items_to_tiles_walls_and_coins() {
        // `Terraria.ID.ItemID` ids: Dirt Block 2, Stone Wall 26, Shuriken 42.
        assert_eq!(registry::place_wall(30), Some(16)); // WallID.Dirt = 16 (WallID.cs:137)
        assert_eq!(registry::name(30), "Dirt Wall");
        assert_eq!(registry::name(3), "Stone Block");
        assert_eq!(registry::by_wall(16), Some(30)); // the placed (safe) wall
        assert_eq!(registry::by_wall(2), Some(5546)); // DefaultToPlaceableWall(2), Item.cs case 5546
        assert_eq!(registry::projectile(42), Some(crate::projectile::SHURIKEN));
        assert!(registry::is_weapon(42));
        assert_eq!(registry::by_tile(0), Some(2));
        assert_eq!(registry::coin_index(73), Some(2));
        assert_eq!(registry::coin_value(73), 10_000);
        assert_eq!(registry::name(73), "Gold Coin");
        assert_eq!(registry::migrate_kind(0), 2);
        assert_eq!(registry::migrate_kind(10_002), 30);
        assert_eq!(registry::migrate_kind(20_001), 42);
        assert_eq!(registry::migrate_kind(30_002), 73);
    }

    #[test]
    fn drops_fall_and_rest_on_the_ground() {
        let w = ground();
        let mut it = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        for _ in 0..240 {
            it.update(&w, 1.0 / 60.0);
        }
        let feet = it.pos.y + SIZE;
        assert!(
            (feet - 20.0 * TILE_PX).abs() < 2.0,
            "rested at {feet} instead of the floor"
        );
    }

    #[test]
    fn drops_bounce_off_walls() {
        let mut w = ground();
        for y in 14..20 {
            let i = w.index(14, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut it = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 19.0 * TILE_PX, 6.0, 0.0, 0.0);
        for _ in 0..120 {
            it.update(&w, 1.0 / 60.0);
        }
        assert!(it.pos.x + SIZE <= 14.0 * TILE_PX, "x {}", it.pos.x);
    }

    // -----------------------------------------------------------------
    // W4-f47 — the fall-through physics witnesses (WRITTEN NOT RUN)
    // -----------------------------------------------------------------

    /// `WorldItem.cs:379-396` — the gravity ladder: shimmerWet first
    /// (0.065/4/0.375), honeyWet (0.05/3/0.25), wet (0.08/5/0.5), air
    /// (0.1/7/0.5).
    #[test]
    fn w4_f47_the_gravity_ladder_order() {
        assert_eq!(item_gravity_ladder(true, true, true), (0.065, 4.0, 0.375));
        assert_eq!(item_gravity_ladder(true, false, true), (0.05, 3.0, 0.25));
        assert_eq!(item_gravity_ladder(true, false, false), (0.08, 5.0, 0.5));
        assert_eq!(item_gravity_ladder(false, false, false), (0.1, 7.0, 0.5));
        // Dry flags → air even if the kind flags are stale.
        assert_eq!(item_gravity_ladder(false, true, true), (0.1, 7.0, 0.5));
    }

    /// `ItemID.cs:1320` — the NoGravity set membership.
    #[test]
    fn w4_f47_the_no_gravity_set() {
        for k in [520, 521, 575, 549, 548, 547, 3453, 3456, 3459, 3580, 3581, 4143] {
            assert!(item_no_gravity(k), "kind {k}");
        }
        assert!(!item_no_gravity(2)); // Dirt Block falls
        assert!(!item_no_gravity(11)); // Iron Ore falls
    }

    /// `WorldItem.cs:774-786` — a NoGravity drop damps both axes ×0.95 with
    /// the 0.1 snap instead of falling.
    #[test]
    fn w4_f47_a_no_gravity_drop_damps_instead_of_falling() {
        let w = ground();
        // Soul of Light 520, mid-air, velocity (2, 2).
        let mut d = ItemDrop::new(520, 1, 0, 10.0 * TILE_PX, 5.0 * TILE_PX, 2.0, 2.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        // Both axes ×0.95, no gravity added.
        assert!((d.vel.x - 1.9).abs() < 1e-5, "vx {}", d.vel.x);
        assert!((d.vel.y - 1.9).abs() < 1e-5, "vy {}", d.vel.y);
        // Below 0.1 snaps to 0.
        let mut d2 = ItemDrop::new(520, 1, 0, 10.0 * TILE_PX, 5.0 * TILE_PX, 0.05, 0.05, 0.0);
        d2.update(&w, 1.0 / 60.0);
        assert_eq!(d2.vel.x, 0.0);
        assert_eq!(d2.vel.y, 0.0);
    }

    /// `Collision.cs:2332-2335` — the aetherium platform (tile 19,
    /// frameY/18 == 50) is non-solid for the item pass but solid for the
    /// general collision.
    #[test]
    fn w4_f47_the_aetherium_platform_is_item_only_nonsolid() {
        let mut w = World::new("aeth".into(), 32, 32);
        // A platform row at y = 20, style 50 (frameY = 50*18 = 900).
        for x in 0..32 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile { kind: 19, frame_y: 900, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        let pos = [10.0 * TILE_PX, 19.0 * TILE_PX];
        let vel = [0.0, 4.0];
        // The item pass skips the aetherium platform (frameY/18 == 50) —
        // the velocity passes through untouched.
        let item_tc = crate::collision::tile_collision_opts(&w, pos, vel, SIZE, SIZE, false, false, true);
        assert_eq!(item_tc.vel[1], 4.0, "the item falls through the aetherium platform");
        // A NORMAL platform (frameY 0) still blocks the item pass: the
        // residual is ry - (pos.y + SIZE) = 320 - 320 = 0 (the 16 box's
        // bottom lies flush at the platform top — keep-one 144).
        for x in 0..32 {
            let i = w.index(x, 20);
            w.tiles[i].frame_y = 0;
        }
        let item_tc2 = crate::collision::tile_collision_opts(&w, pos, vel, SIZE, SIZE, false, false, true);
        assert!((item_tc2.vel[1] - 0.0).abs() < 1e-5, "a normal platform still blocks: {}", item_tc2.vel[1]);
    }

    /// `WorldItem.cs:789-848` — two overlapping shimmered drops repel: the
    /// left one pushes left, the right one pushes right, and the brake arms.
    #[test]
    fn w4_f47_the_shimmered_repulsion_pushes_apart() {
        // Two shimmered drops 8 px apart (centres), within the 16 px window.
        let mut a = ItemDrop::new(11, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        let mut b = ItemDrop::new(11, 1, 0, 108.0, 100.0, 0.0, 0.0, 0.0);
        a.shimmered = true;
        b.shimmered = true;
        let mut drops = vec![a, b];
        ItemDrop::shimmered_repulsion(&mut drops);
        assert!(drops[0].vel.x < 0.0, "the left drop pushes left: {}", drops[0].vel.x);
        assert!(drops[1].vel.x > 0.0, "the right drop pushes right: {}", drops[1].vel.x);
        assert!(drops[0].repulsion_brake && drops[1].repulsion_brake);
        // Far apart → no push, no brake.
        let mut c = ItemDrop::new(11, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        let mut d = ItemDrop::new(11, 1, 0, 200.0, 100.0, 0.0, 0.0, 0.0);
        c.shimmered = true;
        d.shimmered = true;
        let mut far = vec![c, d];
        ItemDrop::shimmered_repulsion(&mut far);
        assert_eq!(far[0].vel.x, 0.0);
        assert!(!far[0].repulsion_brake);
    }

    /// Keep-one 144 (the e19 round-3 fold, `939e45dc8`'s witness pair):
    /// `Collision.cs:2394`/`:2444` — the blocked-Y flush residual is the
    /// velocity in BOTH arms (the port's `tile_collision_opts` keeps it
    /// verbatim; no dry snap-and-zero stand-in ever existed on this lane).
    /// A dry drop one px above the floor lands flush in ONE tick with the
    /// residual as velocity (`ApplyMovement:1082`); at flush rest the
    /// residual is 0, so the resting drop reads vel 0 (no bounce, no drift).
    #[test]
    fn w4_f47k1_the_dry_flush_landing_carries_the_residual_then_rests() {
        let w = ground();
        // Floor top 20*16 = 320, the 16 box lies flush at pos.y 304.
        let mut land = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 303.0, 0.0, 2.0, 0.0);
        land.update(&w, 1.0 / 60.0);
        assert!((land.pos.y - 304.0).abs() < 1e-4, "dry flush landing: {}", land.pos.y);
        assert!((land.vel.y - 1.0).abs() < 1e-4, "the dry residual: {}", land.vel.y);
        // The next tick: gravity 0.1, the collision re-blocks at residual 0.
        land.update(&w, 1.0 / 60.0);
        assert!((land.pos.y - 304.0).abs() < 1e-4, "flush rest: {}", land.pos.y);
        assert_eq!(land.vel.y, 0.0, "the rest residual is zero");
    }

    /// The wet twin (`:1056-1059` → `ApplyMovement:1078`): the blocked-Y
    /// residual copies FULL into wetVelocity, so the wet drop lands flush
    /// the same tick (the e19 round-3 witness folded, re-derived on this
    /// lane's probe: the inset box finds the shimmer row mid-fall).
    #[test]
    fn w4_f47k1_the_wet_landing_keeps_the_flush_residual() {
        let mut w = ground();
        let j = w.index(10, 19);
        w.tiles[j] = Tile::default();
        w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        w.tiles[j].liquid = 255;
        // One px above flush (303), falling at 2 — the probe's inset box
        // (y 307..315) overlaps row 19, so the tick latches wet.
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 303.0, 0.0, 2.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert!(d.wet && d.shimmer_wet, "the probe latched");
        assert!((d.pos.y - 304.0).abs() < 1e-4, "flush landing: {}", d.pos.y);
        assert!((d.vel.y - 1.0).abs() < 1e-4, "the residual velocity: {}", d.vel.y);
    }

    // -----------------------------------------------------------------
    // Doc-sweep lane 36 — the faithful wet probes + the splash arms
    // (WRITTEN NOT RUN, code-only)
    // -----------------------------------------------------------------

    /// `Collision.cs:1576-1587` — the item's WetCollision probe box is the
    /// CENTRE-INSET 10x8 at (pos+3, pos+4), NOT the full 16x16 box (keep-one
    /// 144: the e19 round-3 box fix; the probe grows with the C# box): a
    /// liquid cell that only the full box's edge overlaps leaves the drop
    /// dry. A shimmer cell at tile (11, 10) spans x 176..192; the drop at
    /// x = 163 has full-box right edge 179 (overlaps) but probe right edge
    /// 163+3+10 = 176 (TOUCHES — strict > fails) — the old full-box
    /// stand-in read wet.
    #[test]
    fn w4_f47ds_the_wet_probe_is_the_centre_inset_box() {
        let mut w = World::new("probe".into(), 32, 80);
        let i = w.index(11, 10);
        w.tiles[i].liquid = 255;
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        w.compute_column_surface();
        w.compute_chunks();
        // pos (163, 160): probe box x 166..176, y 164..172 — the cell
        // x 176..192, y 160..160+0 (liquid 255 → surface drop (1/32)*2 =
        // 0.0625, th 16) — the touching X edge is not an overlap → dry.
        let d = ItemDrop::new(2, 1, 0, 163.0, 160.0, 0.0, 0.0, 0.0);
        assert_eq!(d.wet_probe(&w), (false, false, false));
        // Shift 2 px right: probe x 168..178 overlaps 176..192 → wet +
        // shimmer (the FIRST-cell-wins kind flag).
        let d2 = ItemDrop::new(2, 1, 0, 165.0, 160.0, 0.0, 0.0, 0.0);
        assert_eq!(d2.wet_probe(&w), (true, false, true));
    }

    /// `Collision.cs:1610-1612` — the liquid-surface drop: a quarter-full
    /// cell (liquid 64) drops its surface by (256-64)/32*2 = 12 px, so the
    /// probe box must reach BELOW y+12 to count. The old cell-grid
    /// stand-in ignored the drop.
    #[test]
    fn w4_f47ds_the_wet_probe_respects_the_liquid_surface_drop() {
        let mut w = World::new("surf".into(), 32, 80);
        let i = w.index(10, 10);
        w.tiles[i].liquid = 64; // surface at y 160+12 = 172, depth 4
        w.compute_column_surface();
        w.compute_chunks();
        // Probe y 164..172 — its bottom TOUCHES the 172 surface (strict >
        // fails) → dry.
        let d = ItemDrop::new(2, 1, 0, 160.0, 160.0, 0.0, 0.0, 0.0);
        assert_eq!(d.wet_probe(&w).0, false);
        // Probe y 168..176 overlaps 172..176 → wet.
        let d2 = ItemDrop::new(2, 1, 0, 160.0, 164.0, 0.0, 0.0, 0.0);
        assert_eq!(d2.wet_probe(&w).0, true);
        // LavaCollision keeps the FULL box (no inset) but the same surface
        // drop: pos (160, 156) box y 156..172 TOUCHES 172 — strict > fails.
        let j = w.index(10, 10);
        w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        let d3 = ItemDrop::new(2, 1, 0, 160.0, 156.0, 0.0, 0.0, 0.0);
        assert!(!d3.lava_probe(&w), "the touching edge is not an overlap");
        let d4 = ItemDrop::new(2, 1, 0, 160.0, 157.0, 0.0, 0.0, 0.0);
        assert!(d4.lava_probe(&w), "box y 157..173 overlaps the 172 surface");
    }

    /// `Collision.cs:1610-1612` — the surface drops by (256-liquid)/32*2.
    /// A half-full shimmer cell (liquid 128) has its surface 8 px down; a
    /// drop whose centre-shrunk box sits in the top 8 px is NOT wet. The
    /// old naive full-box overlap reported wet there. (Keep-one 144: the
    /// e19 lane's `box_is_wet` witness folded — the draw-feed bool probe
    /// runs the same box maths as `wet_probe`.)
    #[test]
    fn w4_f47ds_the_box_is_wet_probe_uses_the_liquid_surface_offset() {
        let mut w = ground();
        let j = w.index(10, 19);
        w.tiles[j] = Tile::default();
        w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        w.tiles[j].liquid = 128; // half-full: surface at 19*16 + 8
        // The drop's box top at 19*16 (the cell top), 16 tall → the centre-
        // shrunk probe (8 tall, centred) spans 19*16+4 .. +12; the liquid
        // starts at +8, so it DOES overlap. Park it higher: top at 19*16-8 →
        // probe spans -4 .. +4 relative to the cell → above the +8 surface.
        let d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 19.0 * TILE_PX - 8.0, 0.0, 0.0, 0.0);
        assert!(!d.box_is_wet(&w), "above the half-full surface is dry");
        // A full cell (liquid 255, surface at +0.06) IS wet at the same spot.
        w.tiles[j].liquid = 255;
        assert!(d.box_is_wet(&w), "the full cell wets the box");
    }

    /// `WorldItem.cs:879-956` — the entry latch: a dry drop whose inset
    /// probe finds shimmer arms wetCount 20 + the ENTRY splash (the sticky
    /// shimmer flag gates the arm); the `:1042-1045` decrement runs the
    /// same tick (20 → 19). The flags are STICKY (`:875-878` only sets).
    #[test]
    fn w4_f47ds_the_entry_splash_latches_with_the_sticky_flags() {
        let mut w = World::new("entry".into(), 40, 80);
        for x in 0..40 {
            let i = w.index(x, 12);
            w.tiles[i].liquid = 255;
            w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        }
        w.compute_column_surface();
        w.compute_chunks();
        // Keep-one 144 re-pin: the 16-box probe spans pos.y+4 .. pos.y+12;
        // the row-12 surface sits at 192.06, so pos.y 176 (the old 11*16)
        // gives 180..188 — dry. Park at 184: probe 188..196 overlaps.
        let mut d = ItemDrop::new(2, 1, 0, 30.0 * TILE_PX, 184.0, 0.0, 0.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert!(d.wet && d.shimmer_wet);
        assert_eq!(d.wet_count, 19, "20 armed, decremented same tick");
        let s = d.splash_due.expect("the entry splash armed");
        assert!(!s.exit && s.shimmer && !s.lava && !s.honey);
        // The next tick: already wet → no second latch.
        d.splash_due = None;
        d.update(&w, 1.0 / 60.0);
        assert!(d.splash_due.is_none(), "the entry fires once");
    }

    /// `WorldItem.cs:959-1041` — the exit latch snapshots the STICKY flags
    /// BEFORE the `:1036-1041` dry clear: the drained splash still sees
    /// shimmer even though the drop's flags read false afterwards.
    #[test]
    fn w4_f47ds_the_exit_splash_snapshots_the_pre_clear_flags() {
        let w = World::new("exit".into(), 40, 80);
        let mut d = ItemDrop::new(2, 1, 0, 30.0 * TILE_PX, 11.0 * TILE_PX, 0.0, 0.0, 0.0);
        d.wet = true;
        d.shimmer_wet = true;
        d.update(&w, 1.0 / 60.0);
        assert!(!d.wet && !d.shimmer_wet, "the dry clear ran");
        let s = d.splash_due.expect("the exit splash armed");
        assert!(s.exit && s.shimmer, "the snapshot keeps the pre-clear flag");
        assert_eq!(d.wet_count, 19);
    }

    /// `WorldItem.cs:890-913` — the shimmer entry arm's draw census: 10
    /// dust-308 spawns (6 draws each: pos pair, vel pair, frame Next(3),
    /// scale Next(-20,21)) + 10 Next(6) colours + the style-4 cue's
    /// Next(-20,1) pitch = 71 draws on the caller's stream; the per-dust
    /// modifiers (vy −4 after the 308 tweak's +1, vx ×2.5 after its ×0.5,
    /// scale SET 0.8, noGravity) and the four-colour switch land verbatim.
    #[test]
    fn w4_f47ds_the_shimmer_entry_splash_draws_71_and_colours_verbatim() {
        let w = World::new("splash".into(), 40, 80);
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = tr_core::particles::SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
            ..Default::default()
        };
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = tr_core::unified_random::UnifiedRandom::new(4242);
        let mut d = ItemDrop::new(2, 1, 0, 300.0, 300.0, 0.0, 0.0, 0.0);
        d.splash_due = Some(Splash {
            exit: false,
            lava: false,
            shimmer: true,
            honey: false,
            pos: d.pos,
        });
        d.splash(&mut pool, &ctx, &w, &mut audio, &mut rng, 0);
        assert!(d.splash_due.is_none(), "the latch spent");
        // The twin: 10 x (6 NewDust draws + 1 colour) + 1 pitch.
        let mut twin = tr_core::unified_random::UnifiedRandom::new(4242);
        let mut colours = Vec::new();
        for _ in 0..10 {
            twin.next_max(24); // width+12-4 = 24 (the 16 box, keep-one 144)
            twin.next_max(20); // 24-4 = 20
            twin.next_range(-20, 21);
            twin.next_range(-20, 21);
            twin.next_max(3);
            twin.next_range(-20, 21);
            colours.push(twin.next_max(6));
        }
        twin.next_range(-20, 1); // the style-4 pitch (`:881-885`)
        assert_eq!(rng, twin, "71 draws exactly, in C# order");
        // The dust modifiers: slot 0's scale SET 0.8 (not the roll), the
        // colour from the first pick, noGravity, alpha untouched (0).
        let d0 = pool.slot_mut(0);
        assert_eq!(d0.dust_type, 308);
        assert_eq!(d0.scale, 0.8);
        assert!(d0.no_gravity);
        assert_eq!(d0.alpha, 0, "the shimmer arm sets no alpha");
        let expect = match colours[0] {
            0 => (255u8, 255u8, 210u8),
            1 => (190, 245, 255),
            2 => (255, 150, 255),
            _ => (190, 175, 255),
        };
        assert_eq!((d0.color.r, d0.color.g, d0.color.b), expect);
        // ARBITER 144 fold — the e19 16-box anchor bands (its
        // `..._draws_71_at_the_16_box` witness, re-driven through the
        // keeper method): anchor x = pos.X−6, the roll +4..+27 → 298..321;
        // anchor y = pos.Y + height/2 − 8 = pos.Y + 0 at the 16 box, the
        // roll +4..+23 → 304..323. The old 14-box sat 1 px high.
        assert!((298.0..=321.0).contains(&d0.position.x), "x: {}", d0.position.x);
        assert!((304.0..=323.0).contains(&d0.position.y), "y: {}", d0.position.y);
    }

    /// `WorldItem.cs:944-953` — the lava gate WINS over shimmer/honey (the
    /// `!flag2` else): 5 dust-35, scale 1.3, alpha 100, vy −1.5 after the
    /// type-35 tweak's −0.5 pin, the style-1 cue (one Next(-10,11)).
    #[test]
    fn w4_f47ds_the_lava_splash_wins_the_gate_and_draws_31() {
        let w = World::new("lsplash".into(), 40, 80);
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = tr_core::particles::SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
            ..Default::default()
        };
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = tr_core::unified_random::UnifiedRandom::new(7);
        let mut d = ItemDrop::new(2, 1, 0, 300.0, 300.0, 0.0, 0.0, 0.0);
        d.splash_due = Some(Splash {
            exit: true,
            lava: true,
            shimmer: true, // the lava gate still wins
            honey: true,
            pos: d.pos,
        });
        d.splash(&mut pool, &ctx, &w, &mut audio, &mut rng, 0);
        let mut twin = tr_core::unified_random::UnifiedRandom::new(7);
        for _ in 0..5 {
            twin.next_max(24); // width+12-4 = 24 (the 16 box)
            twin.next_max(20);
            twin.next_range(-20, 21);
            twin.next_range(-20, 21);
            twin.next_max(3);
            twin.next_range(-20, 21);
        }
        twin.next_range(-10, 11); // the style-1 pitch (`:888`)
        assert_eq!(rng, twin, "5x6 + 1 = 31 draws (no colour picks)");
        let d0 = pool.slot_mut(0);
        assert_eq!(d0.dust_type, 35);
        assert_eq!(d0.scale, 1.3);
        assert_eq!(d0.alpha, 100);
        // vy: the roll, then the type-35 pin −0.5 (`Dust.cs:247`), then
        // the arm's −1.5 — the pin is overwritten by the arm's += only in
        // part: −0.5 + −1.5 = −2.0.
        assert_eq!(d0.velocity.y, -2.0);
        assert!(d0.no_gravity);
        // ARBITER 144 fold — the e19 lava census's family count: exactly 5
        // active dust-35 slots, no 308/152 strays (the gate picks ONE arm).
        let n35 = pool.slots().iter().filter(|d| d.active && d.dust_type == 35).count();
        assert_eq!(n35, 5, "the lava family, not 308/152");
    }

    /// ARBITER 144 fold — the e19 `w4_e19_the_water_exit_anchor_drops_the_
    /// minus_8` witness re-driven through the keeper method: the water EXIT
    /// anchor drops the −8 (`WorldItem.cs:1011` vs `:932`); at the 16 box
    /// the exit anchor is `pos.Y + 8` vs the entry's `pos.Y + 0`
    /// (`height/2 − 8` cancels). Same seed → same rolls; the only delta is
    /// the anchor's 8 px. WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_f47k1_the_water_exit_anchor_drops_the_minus_8() {
        let w = World::new("spw".into(), 40, 80);
        let ctx = tr_core::particles::SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
            ..Default::default()
        };
        let mut ys = Vec::new();
        for exit in [false, true] {
            let mut pool = tr_core::particles::DustPool::new();
            let mut audio = crate::audio::Audio::new(None);
            let mut rng = tr_core::unified_random::UnifiedRandom::new(99);
            let mut d = ItemDrop::new(2, 1, 0, 300.0, 300.0, 0.0, 0.0, 0.0);
            d.splash_due = Some(Splash {
                exit,
                lava: false,
                shimmer: false,
                honey: false,
                pos: d.pos,
            });
            d.splash(&mut pool, &ctx, &w, &mut audio, &mut rng, 0);
            let d0 = pool.slot_mut(0);
            assert_eq!(d0.dust_type, tr_core::particles::dust_water(0));
            ys.push(d0.position.y);
        }
        assert_eq!(ys[1] - ys[0], 8.0, "entry {:?} exit {:?}", ys[0], ys[1]);
        assert!((304.0..=323.0).contains(&ys[0]), "entry band at pos.Y+0");
    }

    /// ARBITER FINAL v2 fold тАФ the e19 seal witness
    /// `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe` re-driven
    /// through the keeper latch: the `:886` entry gate reads the LOCAL
    /// `flag2` (this tick's `LavaCollision`), NOT the sticky `base.lavaWet`.
    /// A drop whose sticky lava flag is FALSE at the tick head still queues
    /// the LAVA family when the local probe hits lava this tick: the latch
    /// reads `lava_now`, and the `:866-869` arm sets the sticky flag the
    /// same tick. The e19 seal landed the identical verbatim read тАФ this
    /// twin completes the keeper's witness superset (v2 §fold 4).
    /// WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_f47k2_the_entry_splash_gate_reads_the_local_lava_probe() {
        // The f47 entry-latch fixture shape: a 40x80 world (the slime-engine
        // probe clamps its scan to `height - 40`), the lava pool at row 12.
        let mut w = World::new("entrylava".into(), 40, 80);
        for x in 0..40 {
            let j = w.index(x, 12);
            w.tiles[j].liquid = 255;
            w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        }
        w.compute_column_surface();
        w.compute_chunks();
        // Park at y 184: the inset probe (y+4..+12 = 188..196) overlaps the
        // row-12 surface (192.06); the full-box LavaCollision hits too.
        let mut d = ItemDrop::new(2, 1, 0, 30.0 * TILE_PX, 184.0, 0.0, 0.0, 0.0);
        assert!(!d.lava_wet, "the sticky flag starts false");
        d.update(&w, 1.0 / 60.0);
        assert!(d.wet && d.lava_wet, "the :866-869 arm set the sticky flag");
        let s = d.splash_due.expect("the entry splash armed");
        assert!(!s.exit && s.lava, "the gate read the LOCAL probe (:886)");
        assert_eq!(d.wet_count, 19, "20 armed, decremented same tick");
    }
}

/// S1-P-i47 — the ground-item pickup/stacking witnesses (`Player.GrabItems`
/// `Player.cs:34905-34976`, `GetItem` `:39330-39399`, `ItemSpace`
/// `:39153-39197`, `WorldItem.TryCombiningIntoNearbyItems`
/// `WorldItem.cs:157-186`). Written, not run (the code-only swarm).
#[cfg(test)]
mod s1_p_i47_tests {
    use super::*;
    use super::tests::ground;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, name: registry::name(kind), count, prefix: 0 }
    }

    fn empty_inv() -> [ItemSlot; INV] {
        [EMPTY_SLOT; INV]
    }

    /// `GetItemGrabRange` (`Player.cs:35130-35166`): the base 42 plus each
    /// margin — the accessory flags, the unconditional 4143/3822/nebula
    /// rows, the Journey far-range +240.
    #[test]
    fn the_grab_range_margins_follow_the_csharp_rows() {
        let f = GrabFlags::default();
        assert_eq!(get_item_grab_range(3, f), 42, "the base");
        assert_eq!(get_item_grab_range(4143, f), 42 + 300, "the mana pickup");
        assert_eq!(get_item_grab_range(3822, f), 42 + 50, "the void lens");
        assert_eq!(get_item_grab_range(3453, f), 42 + 100, "the nebula row");
        let gold = GrabFlags { gold_ring: true, ..f };
        assert_eq!(get_item_grab_range(71, gold), 42 + 350, "the gold ring");
        assert_eq!(get_item_grab_range(3, gold), 42, "non-coins unaffected");
        let life = GrabFlags { life_magnet: true, ..f };
        assert_eq!(get_item_grab_range(58, life), 42 + 250, "the heart");
        assert_eq!(get_item_grab_range(1734, life), 42 + 250);
        let mana = GrabFlags { mana_magnet: true, ..f };
        assert_eq!(get_item_grab_range(184, mana), 42 + 300, "the star");
        let treasure = GrabFlags { treasure_magnet: true, ..f };
        assert_eq!(get_item_grab_range(3, treasure), 42 + 150);
        let journey = GrabFlags { journey_far_range: true, ..f };
        assert_eq!(get_item_grab_range(3, journey), 42 + 240);
        // The margins stack (`:35133-35164`'s independent ifs).
        let all = GrabFlags {
            gold_ring: true,
            treasure_magnet: true,
            journey_far_range: true,
            ..f
        };
        assert_eq!(get_item_grab_range(73, all), 42 + 350 + 150 + 240);
    }

    /// `PullItem_Common` (`Player.cs:34983-35029`): ±0.45 per tick under the
    /// ±4 cap (X carries the player's vx, Y does not), plus the 0.75 brake
    /// while the velocity points away.
    #[test]
    fn the_common_pull_accelerates_and_brakes() {
        // Drop to the LEFT/ABOVE the player, at rest: one tick adds 0.45
        // on both axes toward the player.
        let mut vel = Vec2::ZERO;
        pull_item_common(Vec2::ZERO, &mut vel, Vec2::new(30.0, 30.0), 0.0, 0.75);
        assert!((vel.x - 0.45).abs() < 1e-6);
        assert!((vel.y - 0.45).abs() < 1e-6);
        // Moving AWAY at -1: the base step plus the 0.45*0.75 brake.
        let mut vel = Vec2::new(-1.0, -1.0);
        pull_item_common(Vec2::ZERO, &mut vel, Vec2::new(30.0, 30.0), 0.0, 0.75);
        assert!((vel.x - (-1.0 + 0.45 + 0.3375)).abs() < 1e-6);
        assert!((vel.y - (-1.0 + 0.45 + 0.3375)).abs() < 1e-6);
        // At the cap: no further acceleration (X's cap carries the player
        // velocity term `:34987`, Y's does not `:35009`).
        let mut vel = Vec2::new(4.0, 4.0);
        pull_item_common(Vec2::ZERO, &mut vel, Vec2::new(30.0, 30.0), 0.0, 0.75);
        assert_eq!(vel, Vec2::new(4.0, 4.0));
        let mut vel = Vec2::new(4.0, 4.0);
        pull_item_common(Vec2::ZERO, &mut vel, Vec2::new(30.0, 30.0), 2.0, 0.75);
        assert!((vel.x - 4.45).abs() < 1e-6, "the player's run widens the X cap");
        assert_eq!(vel.y, 4.0, "Y's cap carries no player term");
    }

    /// `PullItem_Pickup` (`Player.cs:35031-35041`): the blended
    /// `(v*(acc-1) + speed*dir)/acc` — the magnet arms' (12, 5) pair.
    #[test]
    fn the_magnet_pull_blends_with_weight_one_over_acc() {
        // 3-4-5 geometry: the pull vector is (12*3/5, 12*4/5) = (7.2, 9.6).
        let mut vel = Vec2::new(1.0, -1.0);
        pull_item_pickup(Vec2::ZERO, &mut vel, Vec2::new(3.0, 4.0), 12.0, 5);
        assert!((vel.x - (1.0 * 4.0 + 7.2) / 5.0).abs() < 1e-5);
        assert!((vel.y - (-1.0 * 4.0 + 9.6) / 5.0).abs() < 1e-5);
    }

    /// `ItemSpace` (`Player.cs:39153-39197`): the grid/ammo/vault scans and
    /// the favorited one-per-inventory guard.
    #[test]
    fn item_space_scans_grid_ammo_and_vault() {
        let no_fav = |_: usize| false;
        // An empty grid accepts.
        let s = item_space(&empty_inv(), &[EMPTY_SLOT; 4], 3, 0, no_fav, None);
        assert!(s.can_take && !s.to_void_vault);
        // A full grid refuses…
        let full: [ItemSlot; INV] = std::array::from_fn(|_| slot(2, 9999));
        let s = item_space(&full, &[EMPTY_SLOT; 4], 3, 0, no_fav, None);
        assert!(!s.can_take);
        // …but a partial same-kind AMMO stack answers the ungated scan
        // (`:39185-39191`).
        let mut ammo = [EMPTY_SLOT; 4];
        ammo[2] = slot(3, 10);
        let s = item_space(&full, &ammo, 3, 0, no_fav, None);
        assert!(s.can_take);
        // An empty ammo slot takes an arrow (`CanFillEmptyAmmoSlot`,
        // `Item.cs:1378-1385` — the wooden arrow 40 is not on the refusal
        // list), but not the gel 23 (`:1380`'s explicit id).
        let s = item_space(&full, &[EMPTY_SLOT; 4], 40, 0, no_fav, None);
        assert!(s.can_take, "the arrow claims an empty ammo row");
        let s = item_space(&full, &[EMPTY_SLOT; 4], 23, 0, no_fav, None);
        assert!(!s.can_take, "the gel never claims an empty ammo row");
        // The vault answers with the to-vault flag (`:39192-39195`).
        let vault: crate::bank::BankStore = [EMPTY_SLOT; 40];
        let s = item_space(&full, &[EMPTY_SLOT; 4], 3, 0, no_fav, Some(&vault));
        assert!(s.can_take && s.to_void_vault);
        // `IsAPickup` always fits (`:39155-39158`), even with everything full.
        let full_vault: crate::bank::BankStore = std::array::from_fn(|_| slot(2, 9999));
        let s = item_space(&full, &[slot(2, 9999); 4], 58, 0, no_fav, Some(&full_vault));
        assert!(s.can_take && !s.to_void_vault);
        // The favorited one-per-inventory guard (`CanItemSlotAcceptPickup`
        // `:39226-39229`): the large gem 1522 is an override-true kind
        // (`ItemID.cs:1389`) with a stackable maxStack — a PARTIAL
        // favorited stack refuses the merge, the same stack unfavorited
        // accepts it.
        let mut inv = full;
        inv[0] = slot(1522, 1);
        let fav0 = |i: usize| i == 0;
        let s = item_space(&inv, &[slot(2, 9999); 4], 1522, 0, fav0, None);
        assert!(!s.can_take, "the favorited gem stack refuses");
        let s = item_space(&inv, &[slot(2, 9999); 4], 1522, 0, no_fav, None);
        assert!(s.can_take, "unfavorited it merges");
        // A consumable favorite still merges (`OnlyNeedOneInInventory`
        // false for the dirt 2, `Item.cs:1183-1191`).
        let mut inv2 = full;
        inv2[0] = slot(2, 10);
        let s = item_space(&inv2, &[EMPTY_SLOT; 4], 2, 0, fav0, None);
        assert!(s.can_take, "a consumable favorite still merges");
    }

    /// `Item.FitsAmmoSlot`/`CanFillEmptyAmmoSlot` (`Item.cs:1369-1385`):
    /// the notAmmo set and the empty-slot refusal list.
    #[test]
    fn the_ammo_predicates_match_the_csharp_sets() {
        assert!(fits_ammo_slot(40), "the wooden arrow");
        assert!(!fits_ammo_slot(9), "the wood is no ammo");
        assert!(fits_ammo_slot(353), "the ale's carve-out (:1373)");
        assert!(!fits_ammo_slot(27), "the acorn is notAmmo");
        assert!(!fits_ammo_slot(71), "the coins are notAmmo");
        assert!(!fits_ammo_slot(169), "the sand is notAmmo");
        assert!(can_fill_empty_ammo_slot(40));
        assert!(!can_fill_empty_ammo_slot(23), "the gel refusal (:1380)");
        assert!(!can_fill_empty_ammo_slot(75), "the fallen star refusal");
        assert!(!can_fill_empty_ammo_slot(353), "the ale refusal");
        assert!(!can_fill_empty_ammo_slot(27), "notAmmo refuses");
    }

    /// `Item.CanPassivelyStackInWorld` (`Item.cs:49573-49593` + the
    /// `ItemID.cs:1447-1471` overrides).
    #[test]
    fn the_passive_stack_predicate_matches_the_overrides() {
        assert!(can_passively_stack_in_world(9), "the wood stacks");
        assert!(can_passively_stack_in_world(2), "the dirt stacks");
        assert!(!can_passively_stack_in_world(71), "the coins never");
        assert!(!can_passively_stack_in_world(75), "the fallen star never");
        assert!(can_passively_stack_in_world(205), "the empty bucket override");
        assert!(can_passively_stack_in_world(3817), "the medal override");
        // The pickaxe's maxStack is 9999 but it is a useStyle-1
        // non-consumable → OnlyNeedOneInInventory true → refuses.
        assert!(!can_passively_stack_in_world(1));
        assert!(only_need_one_in_inventory(1));
        assert!(!only_need_one_in_inventory(9), "the consumable wood");
        assert!(only_need_one_in_inventory(50), "the magic mirror");
        assert!(only_need_one_in_inventory(1522), "the large gem override");
        assert!(!only_need_one_in_inventory(267), "the doll override");
    }

    /// `deposit_pickup` — the `GetItem` walk order (S1-P-i47): ammo pre-pass
    /// first, occupied forward, usable hotbar, empty REVERSE.
    #[test]
    fn the_deposit_walk_follows_the_getitem_order() {
        // The arrow lands in a PARTIAL ammo stack first (`FillAmmo`
        // `:39264-39304`); the overflow 16 then rides FillAmmo's OWN empty
        // pass (`CanFillEmptyAmmoSlot`, `:39305-39326`) into ammo[0] — the
        // grid never sees it.
        let mut inv = empty_inv();
        let mut ammo = [EMPTY_SLOT; 4];
        ammo[1] = slot(40, 9990);
        let mut fills = Vec::new();
        let left = deposit_pickup(&mut inv, &mut ammo, 40, 0, 25, |_| false, &mut |m| fills.push(m));
        assert_eq!(left, 0);
        assert_eq!(ammo[1].count, 9999, "the ammo partial took 9");
        assert_eq!(ammo[0].count, 16, "the overflow claimed the empty ammo row");
        assert!(inv.iter().all(|s| s.count == 0), "the grid stayed out");
        assert_eq!(fills, vec![9, 16], "one notify per accepted slot");
        // With NO empty ammo row left, the overflow falls to the grid —
        // the arrow's useStyle is 0 (`Item.cs` case 40), so the hotbar arm
        // skips it and the REVERSE grid scan takes it (`:39387-39394`).
        let mut inv = empty_inv();
        let mut ammo = [slot(2, 9999), slot(40, 9990), slot(2, 9999), slot(2, 9999)];
        let left = deposit_pickup(&mut inv, &mut ammo, 40, 0, 25, |_| false, &mut |_| {});
        assert_eq!(left, 0);
        assert_eq!(ammo[1].count, 9999);
        assert_eq!(inv[0].count, 0, "the useStyle-0 kind skips the hotbar");
        assert_eq!(inv[INV - 1].kind, 40);
        assert_eq!(inv[INV - 1].count, 16);
        // An empty ammo slot takes a whole arrow stack when no partial
        // exists (`CanFillEmptyAmmoSlot`, `:39305-39326`).
        let mut inv = empty_inv();
        let mut ammo = [EMPTY_SLOT; 4];
        let left = deposit_pickup(&mut inv, &mut ammo, 40, 0, 7, |_| false, &mut |_| {});
        assert_eq!(left, 0);
        assert_eq!(ammo[0].count, 7, "the first empty ammo row");
        assert!(inv.iter().all(|s| s.count == 0));
        // The gel never claims an ammo row — it rides the grid.
        let mut inv = empty_inv();
        let mut ammo = [EMPTY_SLOT; 4];
        let left = deposit_pickup(&mut inv, &mut ammo, 23, 0, 5, |_| false, &mut |_| {});
        assert_eq!(left, 0);
        assert!(ammo.iter().all(|s| s.count == 0));
        assert_eq!(inv[INV - 1].kind, 23, "the useStyle-0 gel goes REVERSE");
        // A usable kind (the stone 3, useStyle 1) takes the empty hotbar.
        let mut inv = empty_inv();
        let left = deposit_pickup(&mut inv, &mut [EMPTY_SLOT; 4], 3, 0, 5, |_| false, &mut |_| {});
        assert_eq!(left, 0);
        assert_eq!(inv[0].kind, 3);
        // The occupied merge runs FORWARD before any empty slot: a partial
        // stone in row 30 takes the drop even with the hotbar empty.
        let mut inv = empty_inv();
        inv[30] = slot(3, 100);
        let left = deposit_pickup(&mut inv, &mut [EMPTY_SLOT; 4], 3, 0, 5, |_| false, &mut |_| {});
        assert_eq!(left, 0);
        assert_eq!(inv[30].count, 105);
        assert_eq!(inv[0].count, 0);
        // A favorited CONSUMABLE stack still merges — the guard only fires
        // for `OnlyNeedOneInInventory` kinds (`:39226-39229`; the Work
        // Bench 36 is consumable, `Item.cs:1183`).
        let mut inv = empty_inv();
        inv[49] = slot(36, 1);
        let left = deposit_pickup(&mut inv, &mut [EMPTY_SLOT; 4], 36, 0, 1, |i| i == 49, &mut |_| {});
        assert_eq!(left, 0);
        assert_eq!(inv[49].count, 2);
        // A full grid returns the remainder (the caller's vault/world step).
        let full: [ItemSlot; INV] = std::array::from_fn(|_| slot(2, 9999));
        let mut inv = full;
        let left = deposit_pickup(&mut inv, &mut [slot(2, 9999); 4], 3, 0, 5, |_| false, &mut |_| {});
        assert_eq!(left, 5, "nothing fit — the remainder returns");
    }

    /// `TryCombiningIntoNearbyItems` (`WorldItem.cs:157-186`): the 30 px
    /// Manhattan merge, the cap, the lerp and the drained-source removal.
    #[test]
    fn nearby_drops_combine_up_to_the_cap() {
        let mut drops = vec![
            ItemDrop::new(9, 9990, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 20, 0, 110.0, 105.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 5, 0, 200.0, 100.0, 0.0, 0.0, 0.0), // out of range
        ];
        let drained = ItemDrop::combine_nearby(&mut drops, 0, false);
        assert!(drained.is_empty(), "the source kept 10 — no TurnToAir");
        assert_eq!(drops[0].count, 9999, "the cap");
        assert_eq!(drops[1].count, 11, "20 - 9");
        // The lerp: amount = 9/9999 toward the survivor (`:172-174`).
        let amount = 9.0f32 / 9999.0;
        let want_x = 110.0 + (100.0 - 110.0) * amount;
        assert!((drops[0].pos.x - want_x).abs() < 1e-4);
        assert_eq!(drops[2].count, 5, "the far drop untouched");
        // A small survivor drains the source completely.
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 0, 110.0, 100.0, 0.0, 0.0, 0.0),
        ];
        let drained = ItemDrop::combine_nearby(&mut drops, 0, false);
        assert_eq!(drained, vec![1]);
        assert_eq!(drops[0].count, 8);
        // The Manhattan window: |dx| + |dy| <= 30 (`:167`).
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 0, 120.0, 111.0, 0.0, 0.0, 0.0), // 20+11=31
        ];
        let drained = ItemDrop::combine_nearby(&mut drops, 0, false);
        assert!(drained.is_empty());
        assert_eq!(drops[0].count, 5, "31 > 30 — no merge");
        // A different prefix never stacks (`Item.CanStack`).
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 81, 105.0, 100.0, 0.0, 0.0, 0.0),
        ];
        assert!(ItemDrop::combine_nearby(&mut drops, 0, false).is_empty());
        assert_eq!(drops[0].count, 5);
        // The shimmering drop waits (`:159`'s shimmerTime gate).
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 0, 105.0, 100.0, 0.0, 0.0, 0.0),
        ];
        drops[0].shimmer_time = 0.5;
        assert!(ItemDrop::combine_nearby(&mut drops, 0, false).is_empty());
        // W4-g15 lane 5 — the reservation gate (`:159`): a drop reserved
        // for another peer never combines, even inside the window.
        drops[0].shimmer_time = 0.0;
        assert!(ItemDrop::combine_nearby(&mut drops, 0, true).is_empty());
        assert_eq!(drops[0].count, 5, "the reserved drop kept its stack");
    }

    /// W4-g15 lane 5 — `MoveInWorld`'s ItemNoGravity arm (`WorldItem.cs:774-
    /// 786`): a soul/essence/nebula/fragment (the 16-kind set, `ItemID.cs:1320`)
    /// damps BOTH axes ×0.95 with the 0.1 rest snap, no gravity, no fall cap.
    /// A normal drop keeps the 0.1 gravity + 7 cap. The shimmered case routes
    /// to the float branch (never here).
    #[test]
    fn w4_g15_the_no_gravity_kinds_damp_both_axes_without_falling() {
        let mut w = World::new("float".into(), 32, 32);
        w.compute_column_surface();
        w.compute_chunks();
        // A Soul of Light (520) with a diagonal velocity: both axes ×0.95,
        // no gravity added.
        let mut d = ItemDrop::new(520, 1, 0, 100.0, 100.0, 2.0, 3.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert!((d.vel.x - 2.0 * 0.95).abs() < 1e-6, "X damped, got {}", d.vel.x);
        assert!((d.vel.y - 3.0 * 0.95).abs() < 1e-6, "Y damped (no gravity), got {}", d.vel.y);
        // The rest snap: a small velocity snaps to 0 on both axes.
        let mut d = ItemDrop::new(521, 1, 0, 100.0, 100.0, 0.05, -0.05, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert_eq!(d.vel, Vec2::ZERO, "the 0.1 rest snap on both axes");
        // A normal drop (dirt 2) keeps gravity: Y += 0.1 then the cap.
        let mut d = ItemDrop::new(2, 1, 0, 100.0, 100.0, 2.0, 3.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert!((d.vel.x - 2.0 * 0.95).abs() < 1e-6);
        assert!((d.vel.y - (3.0 + 0.1)).abs() < 1e-6, "gravity added, got {}", d.vel.y);
        // The fall cap: a fast-falling normal drop caps at 7.
        let mut d = ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 20.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert_eq!(d.vel.y, 7.0, "the fall cap");
        // A no-gravity kind does NOT cap (it damps): 20 × 0.95 = 19.
        let mut d = ItemDrop::new(4143, 1, 0, 100.0, 100.0, 0.0, 20.0, 0.0);
        d.update(&w, 1.0 / 60.0);
        assert!((d.vel.y - 19.0).abs() < 1e-6, "no cap on the float kinds");
    }

    /// `WorldItem.UpdateItem`'s grabbed arm (`:425-435` → `ApplyMovement`
    /// `:1074-1084`): the tick after a pull applies the velocity RAW (no
    /// gravity, no friction, no collision) and clears the flag; the grab
    /// delay decrements every tick (`:441-444`).
    #[test]
    fn the_grabbed_tick_applies_the_velocity_raw() {
        let mut w = World::new("grab".into(), 32, 32);
        for x in 0..32 {
            for y in 20..32 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        let mut d = ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        d.being_grabbed = true;
        d.vel = Vec2::new(3.0, -2.0);
        d.grab_delay_time = 100;
        d.update(&w, 1.0 / 60.0);
        assert_eq!(d.pos, Vec2::new(103.0, 98.0), "the raw velocity");
        assert_eq!(d.vel, Vec2::new(3.0, -2.0), "no gravity/friction on the grabbed tick");
        assert!(!d.being_grabbed, "the flag cleared");
        assert_eq!(d.grab_delay_time, 99, "the delay ticks down");
        // The next tick is normal physics again (gravity 0.1).
        d.update(&w, 1.0 / 60.0);
        assert!((d.vel.y - (-2.0 + 0.1)).abs() < 1e-6);
        assert_eq!(d.grab_delay_time, 98);
    }

    /// `Rectangle.Intersects` (XNA): the touching edges do NOT intersect;
    /// the grab rectangle expands by the range on every side
    /// (`Player.cs:34926`).
    #[test]
    fn the_grab_rectangle_is_the_expanded_hitbox() {
        // The player at (100, 100) 20x42, range 42: the rect spans
        // x 58..162, y 58..184. A 16x16 drop at x 42 (42+16 = 58) TOUCHES
        // the left edge — no intersect; at 43 (43+16 = 59) it overlaps.
        assert!(!grab_rect_intersects(100.0, 100.0, 20.0, 42.0, 42.0, Vec2::new(42.0, 100.0)));
        assert!(grab_rect_intersects(100.0, 100.0, 20.0, 42.0, 42.0, Vec2::new(43.0, 100.0)));
        // The direct hitbox: strict overlap.
        assert!(rects_intersect(100.0, 100.0, 20.0, 42.0, 110.0, 120.0, 14.0, 14.0));
        assert!(!rects_intersect(100.0, 100.0, 20.0, 42.0, 120.0, 120.0, 14.0, 14.0));
    }

    // --- S1-P-i50: DoCoins + the Gold Ring grab range ---------------------

    fn coin_slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    /// `Player.DoCoins` (`Player.cs:39246-39262`): 100 copper → 1 silver,
    /// the source slot converts in place when no merge target exists.
    #[test]
    fn s1_p_i50_do_coins_converts_100_copper_to_one_silver() {
        let mut inv: [ItemSlot; INV] = std::array::from_fn(|_| EMPTY_SLOT);
        let mut equip = crate::inv::EquipSlots::empty();
        equip.coins[0] = coin_slot(71, 100);
        let mut slots = carry_slots_mut(&mut inv, &mut equip.coins, Some(&mut equip.ammo));
        do_coins(&mut slots, INV); // C# index 50
        assert_eq!(equip.coins[0].kind, 72, "SetDefaults(type + 1)");
        assert_eq!(equip.coins[0].count, 1, "the stack resets to 1");
        assert_eq!(equip.coins[0].prefix, 0, "ResetStats clears the prefix");
    }

    /// The merge arm (`:39253-39261`): the converted coin joins the first
    /// other same-type slot below 54 with room, the source airs out.
    #[test]
    fn s1_p_i50_do_coins_merges_into_the_first_roomy_slot_and_airs_the_source() {
        let mut inv: [ItemSlot; INV] = std::array::from_fn(|_| EMPTY_SLOT);
        inv[7] = coin_slot(72, 40); // a grid silver partial — the merge target
        let mut equip = crate::inv::EquipSlots::empty();
        equip.coins[0] = coin_slot(71, 100);
        let mut slots = carry_slots_mut(&mut inv, &mut equip.coins, Some(&mut equip.ammo));
        do_coins(&mut slots, INV);
        assert_eq!(inv[7].count, 41, "stack++ on the target");
        assert_eq!(equip.coins[0], EMPTY_SLOT, "TurnToAir on the source");
    }

    /// The chain (`:39259`'s recursive `DoCoins(j)`): 100 copper → silver
    /// merges into a 99-silver → 100 silver → 1 gold.
    #[test]
    fn s1_p_i50_do_coins_chains_through_the_recursive_merge() {
        let mut inv: [ItemSlot; INV] = std::array::from_fn(|_| EMPTY_SLOT);
        inv[3] = coin_slot(72, 99);
        let mut equip = crate::inv::EquipSlots::empty();
        equip.coins[0] = coin_slot(71, 100);
        let mut slots = carry_slots_mut(&mut inv, &mut equip.coins, Some(&mut equip.ammo));
        do_coins(&mut slots, INV);
        assert_eq!(inv[3].kind, 73, "the 100th silver converts to gold");
        assert_eq!(inv[3].count, 1);
        assert_eq!(equip.coins[0], EMPTY_SLOT);
    }

    /// The guard (`:39248`): only exactly 100 of 71/72/73 converts —
    /// platinum (74) never does, 99/101 never do, non-coins never do.
    #[test]
    fn s1_p_i50_do_coins_guard_is_exact_100_of_71_72_73() {
        for (kind, count) in [(71u16, 99u32), (71, 101), (74, 100), (72, 100), (73, 100), (3, 100)] {
            let mut inv: [ItemSlot; INV] = std::array::from_fn(|_| EMPTY_SLOT);
            let mut equip = crate::inv::EquipSlots::empty();
            equip.coins[0] = coin_slot(kind, count);
            let before = equip.coins[0];
            let mut slots = carry_slots_mut(&mut inv, &mut equip.coins, Some(&mut equip.ammo));
            do_coins(&mut slots, INV);
            if matches!(kind, 72 | 73) && count == 100 {
                assert_eq!(equip.coins[0].kind, kind + 1, "{kind} x100 converts");
            } else {
                assert_eq!(equip.coins[0], before, "{kind} x{count} stays");
            }
        }
    }

    /// The merge scan never reaches the ammo slots 54-57 (`j < 54`,
    /// `:39253`): a silver in ammo row 0 is NOT a merge target.
    #[test]
    fn s1_p_i50_do_coins_merge_scan_stops_below_54() {
        let mut inv: [ItemSlot; INV] = std::array::from_fn(|_| EMPTY_SLOT);
        let mut equip = crate::inv::EquipSlots::empty();
        equip.ammo[0] = coin_slot(72, 40); // C# index 54 — out of the scan
        equip.coins[0] = coin_slot(71, 100);
        let mut slots = carry_slots_mut(&mut inv, &mut equip.coins, Some(&mut equip.ammo));
        do_coins(&mut slots, INV);
        assert_eq!(equip.ammo[0].count, 40, "the ammo silver untouched");
        assert_eq!(equip.coins[0].kind, 72, "the conversion stays in place");
        assert_eq!(equip.coins[0].count, 1);
    }

    /// `Player.GetItemGrabRange` (`Player.cs:35130-35166`): base 42, the
    /// Gold Ring adds `Item.coinGrabRange = 350` (`Item.cs:49962`) for
    /// coins ONLY.
    #[test]
    fn s1_p_i50_the_gold_ring_extends_the_coin_grab_range_only() {
        let plain = GrabFlags::default();
        let gold = GrabFlags { gold_ring: true, ..plain };
        assert_eq!(get_item_grab_range(71, plain), 42);
        assert_eq!(get_item_grab_range(71, gold), 392, "42 + 350");
        assert_eq!(get_item_grab_range(74, gold), 392, "platinum is a coin too");
        assert_eq!(get_item_grab_range(3, gold), 42, "stone gets no extension");
        assert_eq!(get_item_grab_range(75, gold), 42, "fallen star is not a coin");
    }

    /// The Gold Ring's pull (`GrabItems`, `Player.cs:34961-34963` →
    /// `PullItem_Pickup(12f, 5)`, `:35031-35041`): a coin 200 px out — far
    /// beyond the base 42 — is pulled with the 4:1 blend
    /// `(v*4 + pull)/5`, `pull = d * (12/dist)`; the grabbed tick applies
    /// the velocity raw (`WorldItem.cs:425-435`). The end-to-end wiring
    /// witness lives in `world_sim/items.rs`
    /// (`s1_p_i50_the_gold_ring_collects_a_far_coin_in_the_world`).
    #[test]
    fn s1_p_i50_the_gold_ring_pulls_a_far_coin_with_the_pickup_blend() {
        let w = ground();
        let pc = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        // Centre 200 px to the right of the player, same height.
        let mut it = ItemDrop::new(71, 1, 0, pc.x + 200.0 - SIZE * 0.5, pc.y - SIZE * 0.5, 0.0, 0.0, 0.0);
        // The grab pass arms `being_grabbed` and writes the pull velocity.
        it.being_grabbed = true;
        pull_item_pickup(it.center(), &mut it.vel, pc, 12.0, 5);
        // pull = d * (12/200); v was 0 → v = pull/5 = d * 12/200/5.
        let expect_vx = -200.0f32 * (12.0 / 200.0) / 5.0;
        assert!((it.vel.x - expect_vx).abs() < 1e-4, "vx {}", it.vel.x);
        assert!(it.vel.y.abs() < 1e-6, "no vertical pull");
        // The grabbed tick applies it raw — no gravity, no friction.
        let before = it.pos;
        it.update(&w, 1.0 / 60.0);
        assert!((it.pos.x - (before.x + expect_vx)).abs() < 1e-4);
        assert_eq!(it.pos.y, before.y, "no gravity on the grabbed tick");
        assert!(!it.being_grabbed, "the flag cleared");
    }

    // ---- S1-P-i52 — the heart/star grab families (gate V4: re-derived on
    // the merged S1-P-i47 GrabItems shape — the range rows ride GrabFlags,
    // the pulls ride pull_item_pickup; the end-to-end arms live in
    // world_sim/items.rs's pickup_tests) ----

    /// `Player.GetItemGrabRange` (`Player.cs:35130-35166`): the 42 base
    /// (`:57121`), +250 `Item.lifeGrabRange` (`Item.cs:49964`) for the heart
    /// trio under `lifeMagnet` (`:35145-35148`), +300 `Item.manaGrabRange`
    /// (`:49963`) for the star trio under `manaMagnet` (`:35137-35140`).
    /// The candy variants ride the SAME rows — the C# tests the type lists,
    /// not the base ids.
    #[test]
    fn s1_p_i52_the_grab_range_rows_add_the_magnet_bonuses() {
        let plain = GrabFlags::default();
        for k in HEART_KINDS {
            assert_eq!(get_item_grab_range(k, plain), 42, "heart {k} plain");
        }
        for k in STAR_KINDS {
            assert_eq!(get_item_grab_range(k, plain), 42, "star {k} plain");
        }
        let life = GrabFlags { life_magnet: true, ..plain };
        for k in HEART_KINDS {
            assert_eq!(get_item_grab_range(k, life), 292, "heart {k} + lifeMagnet");
        }
        for k in STAR_KINDS {
            assert_eq!(get_item_grab_range(k, life), 42, "lifeMagnet ignores stars");
        }
        let mana = GrabFlags { mana_magnet: true, ..plain };
        for k in STAR_KINDS {
            assert_eq!(get_item_grab_range(k, mana), 342, "star {k} + manaMagnet");
        }
        for k in HEART_KINDS {
            assert_eq!(get_item_grab_range(k, mana), 42, "manaMagnet ignores hearts");
        }
        // A generic kind never earns either bonus (`:35137`/`:35145`'s
        // type gates).
        let both = GrabFlags { life_magnet: true, mana_magnet: true, ..plain };
        assert_eq!(get_item_grab_range(12, both), 42);
    }

    /// `Player.PullItem_Pickup` (`Player.cs:35031-35041`): a resting drop
    /// 100 px to the RIGHT of the player, heart + lifeMagnet (speed 15,
    /// acc 5): the pull vector is (-15, 0), the blend `(0*4 + (-15))/5` =
    /// -3 — the first magnet tick moves the heart 3 px. The star +
    /// manaMagnet arm (12/5) blends to -2.4. The grabbed tick
    /// (`WorldItem.cs:425-435`) applies the velocity raw.
    #[test]
    fn s1_p_i52_the_magnet_pull_blends_at_the_csharp_speed_and_acc() {
        let w = ground();
        let pc = Vec2::new(200.0, 200.0);
        // centre at (300, 200) — pos = centre - 8 (the 16x16 box).
        // centre at (300, 200) — pos = centre - 8 (the 16x16 box; the
        // research/145 SHIMMER SEAL repin, `WorldItem.cs:138`).
        let mut heart = ItemDrop::new(58, 1, 0, 292.0, 192.0, 0.0, 0.0, 0.0);
        heart.being_grabbed = true;
        pull_item_pickup(heart.center(), &mut heart.vel, pc, 15.0, 5);
        // f32: 15/100 is inexact, so the blend lands within an ulp of -3
        // (the C# runs the same f32 arithmetic).
        assert!((heart.vel.x - -3.0).abs() < 1e-4, "15/5 blend from rest: {}", heart.vel.x);
        assert_eq!(heart.vel.y, 0.0);
        heart.update(&w, 1.0 / 60.0);
        assert!((heart.pos.x - 289.0).abs() < 1e-3, "the blend moves the drop: {}", heart.pos.x);
        assert_eq!(heart.pos.y, 192.0);
        // The second tick blends again: (−3*4 + −15)/5 = −5.4.
        heart.being_grabbed = true;
        pull_item_pickup(heart.center(), &mut heart.vel, pc, 15.0, 5);
        assert!((heart.vel.x - -5.4).abs() < 1e-4, "vel {}", heart.vel.x);
        let mut star = ItemDrop::new(184, 1, 0, 292.0, 192.0, 0.0, 0.0, 0.0);
        star.being_grabbed = true;
        pull_item_pickup(star.center(), &mut star.vel, pc, 12.0, 5);
        assert!((star.vel.x - -2.4).abs() < 1e-4, "12/5 blend: {}", star.vel.x);
    }

// balance-close (union shared delimiter)
}}
    // ---- W4-e19 — the shimmer liquid-interaction physics census ----------

    /// A 32x32 world with a stone floor at y = 20 and a shimmer pool filling
    /// rows 18-19 (the drop rests in the liquid).
    fn shimmer_pool() -> World {
        let mut w = ground();
        for x in 0..32 {
            for y in 18..=19 {
                let j = w.index(x, y);
                w.tiles[j] = Tile::default();
                w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
                w.tiles[j].liquid = 255;
            }
        }
        w
    }
    #[test]
    fn w4_e19_the_shimmer_wet_gravity_is_0065_cap_4() {
        // `WorldItem.cs:380-385` — a shimmerWet drop falls at 0.065 / cap 4,
        // not the air 0.1 / 7. The flag is read from the PREVIOUS tick.
        let w = shimmer_pool();
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        d.shimmer_wet = true; // the previous tick's probe
        d.update(&w, 1.0 / 60.0);
        // gravity 0.065 applied (the drop is above the pool, dry this tick,
        // but the head read the carried shimmerWet).
        assert!((d.vel.y - 0.065).abs() < 1e-6, "shimmer gravity: {}", d.vel.y);
        // The cap: a fast fall clamps at 4, not 7.
        let mut fast = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 6.0, 0.0);
        fast.shimmer_wet = true;
        fast.update(&w, 1.0 / 60.0);
        assert!((fast.vel.y - 4.0).abs() < 1e-6, "shimmer cap 4: {}", fast.vel.y);
    fn w4_e19_the_shimmered_drop_rises_over_the_two_row_centre_scan() {
        // `WorldItem.cs:493-504` — a shimmered drop NOT shimmerWet rises when
        // either of the two centre rows is shimmer. The probe is at
        // (int)(Center.X/16), (int)(Center.Y/16) and +1 — NOT the ramp's
        // above-top-left tile.
        // Centre in row 18 (the pool's top row): pos.y = 18*16 → centre
        // y = 18*16 + 8 → row 18. shimmerWet false (the box probe is forced
        // off by parking the drop so only the centre row hits).
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        d.shimmered = true;
        d.shimmer_wet = false;
        d.vel = Vec2::new(0.0, 0.0);
        // The rise scan finds row 18 shimmer → vel.y -= 0.05 (the centre row
        // hits even though the 16-box wet probe already latched shimmerWet).
        assert!(d.vel.y < 0.0, "the shimmered drop rises: {}", d.vel.y);
    fn w4_e19_the_shimmered_drop_damps_dry_with_09() {
        // `WorldItem.cs:512-515` — a shimmered drop with NO shimmer under the
        // centre scan damps vel.y *= 0.9 (not the rise).
        let w = ground(); // dry world, no shimmer
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 2.0, 0.0);
        // vel.y 2.0 * 0.9 = 1.8, then gravity 0 (shimmered) → 1.8.
        assert!((d.vel.y - 1.8).abs() < 1e-5, "the dry damp: {}", d.vel.y);
    fn w4_e19_the_wet_probe_uses_the_liquid_surface_offset() {
        // `Collision.cs:1610-1612` — the surface drops by (256-liquid)/32*2.
        // A half-full shimmer cell (liquid 128) has its surface 8 px down; a
        // drop whose centre-shrunk box sits in the top 8 px is NOT wet. The
        // old naive full-box overlap reported wet there.
        let j = w.index(10, 19);
        w.tiles[j] = Tile::default();
        w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        w.tiles[j].liquid = 128; // half-full: surface at 19*16 + 8
        // The drop's box top at 19*16 (the cell top), 16 tall → the centre-
        // shrunk probe (8 tall, centred) spans 19*16+4 .. +12; the liquid
        // starts at +8, so it DOES overlap. Park it higher: top at 19*16-8 →
        // probe spans -4 .. +4 relative to the cell → above the +8 surface.
        let d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 19.0 * TILE_PX - 8.0, 0.0, 0.0, 0.0);
        assert!(!d.box_is_wet(&w), "above the half-full surface is dry");
        // A full cell (liquid 255, surface at +0.06) IS wet at the same spot.
        w.tiles[j].liquid = 255;
        assert!(d.box_is_wet(&w), "the full cell wets the box");
    /// W4-e19 — the shimmered-item repulsion scan (`WorldItem.cs:790-847`).
    /// Two shimmered drops 8 px apart (within the 16 span): the left one is
    /// pushed left, the right one right, by `0.1 * span/distance` capped at
    /// 10 — here distance 8, span 16/8 = 2 → each nudge 0.2.
    fn w4_e19_shimmered_drops_push_apart() {
        let mut a = ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        let mut b = ItemDrop::new(2, 1, 0, 108.0, 100.0, 0.0, 0.0, 0.0);
        a.shimmered = true;
        b.shimmered = true;
        let mut items = vec![a, b];
        let flag = ItemDrop::repel_shimmered(&mut items, 0);
        assert!(flag, "the neighbour was touched");
        // distance 8 → num2 = 16/8 = 2 (≤ 10); the left drop: vx -= 0.1*2.
        assert!((items[0].vel.x - -0.2).abs() < 1e-5, "self pushed left: {}", items[0].vel.x);
        // The neighbour (right): vx += 0.1*2 (`:820-823`, bound < 3).
        assert!((items[1].vel.x - 0.2).abs() < 1e-5, "neighbour pushed right: {}", items[1].vel.x);
        // Running from slot 1 mirrors the pushes (self right, neighbour left).
        let flag1 = ItemDrop::repel_shimmered(&mut items, 1);
        assert!(flag1);
        assert!((items[1].vel.x - 0.4).abs() < 1e-5);
        assert!((items[0].vel.x - -0.4).abs() < 1e-5);
    /// The scan's gates: non-shimmered drops never repel (`:790`/`:795`), and
    /// a neighbour beyond the span on either axis is skipped (`:804`).
    fn w4_e19_the_repulsion_gates() {
        // Plain drops: no scan at all.
        let mut items = vec![
            ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(2, 1, 0, 101.0, 100.0, 0.0, 0.0, 0.0),
        ];
        assert!(!ItemDrop::repel_shimmered(&mut items, 0));
        assert_eq!(items[0].vel.x, 0.0);
        assert_eq!(items[1].vel.x, 0.0);
        // A shimmered self with a NON-shimmered neighbour: skipped.
        items[0].shimmered = true;
        assert_eq!(items[1].vel.x, 0.0, "the plain neighbour is not pushed");
        // Both shimmered but 20 px apart on Y (> the 16 span): skipped.
        items[1].shimmered = true;
        items[1].pos.y = 120.0;
    /// The `span / distance` cap at 10 (`:810-813`): coincident centres give
    /// +inf → 10, and the equal-X tie-break (`:836-846`) pushes the lower slot
    /// left / the higher right, both bounded at `±3 * 10`.
    fn w4_e19_the_repulsion_caps_the_span_at_10_and_breaks_ties_by_slot() {
        let mut b = ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        assert!(flag);
        // distance 0 → num2 capped 10 → the tie-break nudge 0.1*10 = 1.0.
        assert!((items[0].vel.x - -1.0).abs() < 1e-5, "lower slot left: {}", items[0].vel.x);
        assert!((items[1].vel.x - 1.0).abs() < 1e-5, "higher slot right: {}", items[1].vel.x);
        // From the higher slot the tie-break refuses (`whoAmI < i` false).
        assert!(flag1, "flag set even when the tie-break declines the push");
        assert!((items[0].vel.x - -1.0).abs() < 1e-5, "no second push");
        assert!((items[1].vel.x - 1.0).abs() < 1e-5);
    /// The ×0.8 damp tail (`:859-862`): `move_in_world` applies friction THEN
    /// the repulsion damp. A repulsed drop with vx 1.0: ×0.95 → 0.95, ×0.8 →
    /// 0.76. The flag clears after the tick.
    fn w4_e19_the_repulsion_flag_damps_x_by_08_after_friction() {
        let w = ground();
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 1.0, 0.0, 0.0);
        d.repulsed = true;
        assert!((d.vel.x - 0.76).abs() < 1e-5, "0.95 then 0.8: {}", d.vel.x);
        assert!(!d.repulsed, "the flag cleared");
        // Without the flag: plain friction only.
        let mut plain = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 1.0, 0.0, 0.0);
        plain.update(&w, 1.0 / 60.0);
        assert!((plain.vel.x - 0.95).abs() < 1e-5);
    /// W4-e19 — the `ItemNoGravity` float (`WorldItem.cs:774-786`): a Soul of
    /// Light (520, `ItemID.cs:1320`) damps BOTH axes ×0.95 with the 0.1 snap
    /// and takes NO gravity add — it never falls.
    fn w4_e19_the_no_gravity_kinds_float() {
        assert!(item_no_gravity(520) && item_no_gravity(4143) && !item_no_gravity(2));
        let mut soul = ItemDrop::new(520, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 2.0, 2.0, 0.0);
        soul.update(&w, 1.0 / 60.0);
        // ×0.95 on both axes, no +0.1 gravity on Y.
        assert!((soul.vel.x - 1.9).abs() < 1e-5, "X damped: {}", soul.vel.x);
        assert!((soul.vel.y - 1.9).abs() < 1e-5, "Y damped, no gravity: {}", soul.vel.y);
        // The 0.1 rest snap on Y too: a slow float settles to zero.
        let mut slow = ItemDrop::new(521, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.105, 0.105, 0.0);
        slow.update(&w, 1.0 / 60.0);
        assert_eq!(slow.vel.x, 0.0, "0.105*0.95 < 0.1 snaps");
        assert_eq!(slow.vel.y, 0.0);
        // A plain kind still falls (the else arm).
        let mut dirt = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        dirt.update(&w, 1.0 / 60.0);
        assert!((dirt.vel.y - 0.1).abs() < 1e-6);
    /// The float's `!shimmered` gate (`:774`): a shimmered no-gravity kind
    /// takes the ELSE arm — gravity (0 for shimmered) + the X-only friction.
    fn w4_e19_a_shimmered_no_gravity_kind_takes_the_gravity_arm() {
        let w = ground(); // dry: the shimmered drop damps Y ×0.9 (UpdateShimmer)
        soul.shimmered = true;
        // UpdateShimmer's dry damp: Y 2.0×0.9 = 1.8, gravity 0, no Y friction
        // (the else arm frictions X only).
        assert!((soul.vel.y - 1.8).abs() < 1e-5, "the else arm: {}", soul.vel.y);
        assert!((soul.vel.x - 1.9).abs() < 1e-5);
    /// W4-e19 — the `wetVelocity` blend (`WorldItem.cs:379-396`/`:1046-1084`):
    /// a wet drop's collision pass runs at FULL velocity but `ApplyMovement`
    /// integrates the free axes at the liquid fraction — shimmer-wet 0.375
    /// (`:384`), the air default 0.5 (`:379`, the water arm's stand-in until
    /// the per-drop wet flag lands).
    fn w4_e19_the_wet_drop_integrates_at_the_liquid_fraction() {
        // A drop INSIDE the pool (rows 18-19), shimmerWet carried from the
        // previous tick: gravity 0.065, wet_step 0.375.
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 18.0 * TILE_PX + 4.0, 2.0, 0.0, 0.0);
        d.shimmer_wet = true;
        let before = d.pos;
        // X: 2.0 ×0.95 friction = 1.9, integrated ×0.375 = 0.7125.
        assert!(
            (d.pos.x - (before.x + 1.9 * 0.375)).abs() < 1e-4,
            "the shimmer fraction: {}",
            d.pos.x - before.x
        );
        // Y: gravity 0.065, integrated ×0.375 = 0.024375.
            (d.pos.y - (before.y + 0.065 * 0.375)).abs() < 1e-4,
            "the Y fraction: {}",
            d.pos.y - before.y
        // The velocity itself stays FULL (the collision ran at 1.9 / 0.065).
        assert!((d.vel.x - 1.9).abs() < 1e-5);
        assert!((d.vel.y - 0.065).abs() < 1e-6);
    /// The dry arm: full-velocity integration (`:1082`); at flush rest the
    /// collision residual is 0, so the resting drop stays still (round 3
    /// retired the snap-and-zero stand-in — both arms run the C# residual).
    fn w4_e19_the_dry_drop_integrates_at_full_velocity() {
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 2.0, 0.0, 0.0);
        assert!((d.pos.x - (before.x + 1.9)).abs() < 1e-5, "full X: {}", d.pos.x - before.x);
        assert!((d.pos.y - (before.y + 0.1)).abs() < 1e-5, "full Y: {}", d.pos.y - before.y);
        // A dry drop one px above the floor lands flush in ONE tick with the
        // residual as velocity (`Collision.cs:2394` → `ApplyMovement:1082`),
        // the same shape as the wet arm.
        let mut land = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 303.0, 0.0, 2.0, 0.0);
        land.update(&w, 1.0 / 60.0);
        assert!((land.pos.y - 304.0).abs() < 1e-4, "dry flush landing: {}", land.pos.y);
        assert!((land.vel.y - 1.0).abs() < 1e-4, "the dry residual: {}", land.vel.y);
        // The next tick: gravity 0.1, the collision re-blocks at residual 0 —
        // the drop lies flush, vel 0 (no bounce, no drift).
        assert!((land.pos.y - 304.0).abs() < 1e-4, "flush rest: {}", land.pos.y);
        assert_eq!(land.vel.y, 0.0, "the rest residual is zero");
    /// The wet blocked-Y arm keeps the flush residual (`:1056-1059` →
    /// `wetVelocity.Y` full): a shimmer-wet drop landing on the pool floor
    /// steps flush in ONE tick and carries the residual velocity, not the
    /// dry arm's zero.
    fn w4_e19_the_wet_landing_keeps_the_flush_residual() {
        // Resting one px above the floor (floor top 20*16 = 320, box 16 →
        // flush pos.y = 304): pos.y 303, falling at 2 (capped 4 → stays 2).
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 303.0, 0.0, 2.0, 0.0);
        // The blocked Y: residual 304 - 303 = 1.0, copied FULL into
        // wetVelocity.Y → the drop lands flush this tick.
        assert!((d.pos.y - 304.0).abs() < 1e-4, "flush landing: {}", d.pos.y);
        assert!((d.vel.y - 1.0).abs() < 1e-4, "the residual velocity: {}", d.vel.y);
    /// W4-e19 — the honey/water gravity arms (`WorldItem.cs:386-396`): the
    /// ELSE-IF chain reads the PREVIOUS tick's per-drop flags. Honey 0.05 /
    /// cap 3 / wet_step 0.25; water 0.08 / cap 5 / the 0.5 air wet_step.
    fn w4_e19_the_honey_and_water_gravity_arms() {
        let w = ground(); // dry world — the flags are the carried ones
        // Honey-wet: gravity 0.05, cap 3.
        let mut h = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        h.honey_wet = true;
        h.update(&w, 1.0 / 60.0);
        assert!((h.vel.y - 0.05).abs() < 1e-6, "honey gravity: {}", h.vel.y);
        let mut hfast = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 6.0, 0.0);
        hfast.honey_wet = true;
        hfast.update(&w, 1.0 / 60.0);
        assert!((hfast.vel.y - 3.0).abs() < 1e-6, "honey cap 3: {}", hfast.vel.y);
        // Water-wet: gravity 0.08, cap 5.
        let mut aq = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        aq.wet = true;
        aq.update(&w, 1.0 / 60.0);
        assert!((aq.vel.y - 0.08).abs() < 1e-6, "water gravity: {}", aq.vel.y);
        let mut afast = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 6.0, 0.0);
        afast.wet = true;
        afast.update(&w, 1.0 / 60.0);
        assert!((afast.vel.y - 5.0).abs() < 1e-6, "water cap 5: {}", afast.vel.y);
        // The chain order: shimmerWet beats honeyWet beats wet (`:380-396`).
        let mut both = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        both.shimmer_wet = true;
        both.honey_wet = true;
        both.wet = true;
        both.update(&w, 1.0 / 60.0);
        assert!((both.vel.y - 0.065).abs() < 1e-6, "shimmer wins: {}", both.vel.y);
    /// The per-drop wet flags latch from the faithful probes (`:865-878`)
    /// and clear via the `!wet` tail (`:1036-1041`): a drop in a honey pool
    /// carries honeyWet + wet; parked dry, both clear.
    fn w4_e19_the_wet_flags_latch_and_clear() {
                w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Honey);
        // Inside the honey pool: the probe sets honeyWet + wet.
        let mut d = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 18.0 * TILE_PX + 4.0, 0.0, 0.0, 0.0);
        assert!(d.wet && d.honey_wet, "the honey probe latched");
        assert!(!d.shimmer_wet && !d.lava_wet);
        // The NEXT tick's head reads the carried honey flag (0.05).
        let mut d2 = d.clone();
        d2.vel = Vec2::ZERO;
        d2.update(&w, 1.0 / 60.0);
        assert!((d2.vel.y - 0.05).abs() < 1e-6, "the carried honey gravity: {}", d2.vel.y);
        // Parked dry (high above the pool): the exit clears wet, the !wet
        // tail clears honeyWet.
        let mut dry = d.clone();
        dry.pos.y = 5.0 * TILE_PX;
        dry.update(&w, 1.0 / 60.0);
        assert!(!dry.wet && !dry.honey_wet, "the dry tail cleared the flags");
    /// The grabbed arm clears ALL the wet flags (`:427-431`).
    fn w4_e19_the_grabbed_arm_clears_every_wet_flag() {
        d.wet = true;
        d.honey_wet = true;
        d.lava_wet = true;
        d.being_grabbed = true;
        assert!(!d.wet && !d.honey_wet && !d.lava_wet && !d.shimmer_wet);
        assert!(!d.being_grabbed);
        // The full-velocity ApplyMovement (`:1082`): pos += vel, no fraction.
        assert!((d.pos.x - (10.0 * TILE_PX + 1.0)).abs() < 1e-5);
    /// The SHIMMER-seal entry-gate witness: `:886` reads the LOCAL `flag2`
    /// (this tick's `LavaCollision`), NOT the sticky `base.lavaWet`. A drop
    /// whose sticky lava flag is FALSE (hand-set state — the C# never has
    /// this divergence at a reachable entry edge, the seal's verdict) still
    /// bursts the LAVA family when the local probe hits lava this tick: the
    /// queued splash reads `lava_now`, and the sticky flag is set by the
    /// `:866-869` arm the same tick.
    fn w4_e19_the_entry_splash_gate_reads_the_local_lava_probe() {
        // A 40x80 world (the slime-engine probe clamps its scan to
        // `height - 40`), the lava pool at row 12 — the f47 entry-latch
        // fixture shape.
        let mut w = World::new("entrylava".into(), 40, 80);
        for x in 0..40 {
            let j = w.index(x, 12);
            w.tiles[j].liquid = 255;
            w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        w.compute_column_surface();
        w.compute_chunks();
        // Park at y 184: the inset probe (y+4..+12 = 188..196) overlaps the
        // row-12 surface (192.06); the full-box LavaCollision hits too.
        let mut d = ItemDrop::new(2, 1, 0, 30.0 * TILE_PX, 184.0, 0.0, 0.0, 0.0);
        assert!(!d.lava_wet, "the sticky flag starts false");
        assert!(d.wet && d.lava_wet, "the :866-869 arm set the sticky flag");
        let s = d.take_splash().expect("the entry splash armed");
        assert!(!s.exit && s.lava, "the gate read the LOCAL probe (:886)");
        assert_eq!(d.wet_count, 19, "20 armed, decremented same tick");
    /// Folded from the f47 keep-one 144 (`w4_f47ds_the_wet_probe_is_the_
    /// centre_inset_box`, SHIMMER seal): the inset probe is 10x8 at
    /// (pos+3, pos+4) (`Collision.cs:1576-1587` integer halves) — a probe
    /// right edge that TOUCHES the cell's left edge is NOT an overlap
    /// (strict `>`), the old full-box stand-in read wet there. The probe
    /// engine is the `npc_ai_slime::wet_collision` port `move_in_world`
    /// calls (`:870`); the kind statics come from the FIRST overlapping
    /// cell.
    fn w4_e19_the_wet_probe_is_the_centre_inset_box() {
        let mut w = World::new("probe".into(), 32, 80);
        let i = w.index(11, 10);
        w.tiles[i].liquid = 255;
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        // pos (163, 160): probe box x 166..176 — the cell x 176..192; the
        // touching X edge fails the strict overlap -> dry.
        let hit = crate::npc_ai_slime::wet_collision(&w, 163.0, 160.0, SIZE as i32, SIZE as i32);
        assert_eq!(hit, (false, false, false));
        // Shift 2 px right: probe x 168..178 overlaps 176..192 -> wet +
        // shimmer (the FIRST-cell-wins kind flag).
        let hit2 = crate::npc_ai_slime::wet_collision(&w, 165.0, 160.0, SIZE as i32, SIZE as i32);
        assert_eq!(hit2, (true, false, true));
    /// Folded from the f47 keep-one 144 (`w4_f47ds_the_wet_probe_respects_
    /// the_liquid_surface_drop`, SHIMMER seal): the surface drops by
    /// `(256-liquid)/32*2` (`Collision.cs:1610-1612`) — a quarter-full cell
    /// (liquid 64) drops 12 px, the probe bottom TOUCHING the surface is
    /// dry. `LavaCollision` keeps the FULL box (no inset,
    /// `Collision.cs:1653-1684`) with the SAME surface drop.
    fn w4_e19_the_wet_probe_respects_the_liquid_surface_drop() {
        let mut w = World::new("surf".into(), 32, 80);
        let i = w.index(10, 10);
        w.tiles[i].liquid = 64; // surface at y 160+12 = 172, depth 4
        // Probe y 164..172 — bottom TOUCHES the 172 surface -> dry.
        let hit = crate::npc_ai_slime::wet_collision(&w, 160.0, 160.0, SIZE as i32, SIZE as i32);
        assert!(!hit.0);
        // Probe y 168..176 overlaps 172..176 -> wet.
        let hit2 = crate::npc_ai_slime::wet_collision(&w, 160.0, 164.0, SIZE as i32, SIZE as i32);
        assert!(hit2.0);
        // Lava: FULL box, same surface drop. Box y 156..172 TOUCHES -> dry;
        // y 157..173 overlaps the 172 surface -> wet.
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        assert!(!crate::player::lava_collision(&w, 160.0, 156.0, SIZE, SIZE));
        assert!(crate::player::lava_collision(&w, 160.0, 157.0, SIZE, SIZE));

// balance-close (union shared delimiter)
}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}})
}
