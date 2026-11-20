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
fn paint_or_coating(kind: u16) -> bool {
    registry::get(kind).is_some_and(|d| d.paint > 0 || d.paint_coating > 0)
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

pub const SIZE: f32 = 14.0;
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

/// `WorldItem.UpdateItem`'s air constants (`WorldItem.cs:363-364`):
/// `float gravity = 0.1f; float maxFallSpeed = 7f;` — the wet/shimmer/honey
/// arms override them (`:380-396`), the air case is this pair.
const GRAVITY: f32 = 0.1;
const MAX_FALL: f32 = 7.0;
/// `WorldItem.MoveInWorld` (`WorldItem.cs:854-860`): the horizontal friction
/// `base.velocity.X *= 0.95f` and the snap to rest below `0.1`.
const FRICTION_X: f32 = 0.95;
const FRICTION_EPSILON: f32 = 0.1;

/// The accessory flags `GetItemGrabRange`/`GrabItems` read
/// (`Player.cs:35133-35151`/`:34943-34963`). The port's equipment pass has
/// not landed the Mana Magnet / Treasure Magnet grants (the `manaMagnet`
/// family is a recorded unported seam, `player.rs:986`); the Gold Ring's
/// `gold_ring` flag is live (S1-P-i50); the Lifeforce buff's `lifeMagnet`
/// (`buff_effects.rs:52`) wires here when the accumulator lands on Player.
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
/// against the drop's 14x14 hitbox.
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
    /// The natural roll's prefix (`Item.NewItem`'s `prefix` argument,
    /// `Item.cs:49730`); 0 = none.
    pub prefix: u16,
    /// Top-left of the 14x14 item box, world pixels.
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
}

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
            prefix,
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            age: 0.0,
            phase,
            grab_delay_time: 0,
            grab_delay_player: 255,
            being_grabbed: false,
            on_conveyor: false,
        }
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

    pub fn center(&self) -> Vec2 {
        self.pos + Vec2::splat(SIZE * 0.5)
    }

    /// `WorldItem.CanShimmerAtPosition` (`WorldItem.cs:756-768`): the tile
    /// above the drop's center (the C# reads `Center.X / 16` and
    /// `position.Y / 16 - 1`) is shimmer liquid.
    pub fn over_shimmer(&self, world: &World) -> bool {
        let tx = (self.center().x / tr_core::TILE_PX) as i32;
        let ty = (self.pos.y / tr_core::TILE_PX) as i32 - 1;
        world.tile(tx, ty)
            .is_some_and(|t| tr_core::shimmer::tile_is_shimmer(t))
    }

    /// The drop's 14x14 box overlapping shimmer liquid this tick — the port's
    /// stand-in for the C#'s `Collision.shimmer` flag that
    /// `WorldItem.Update`'s wet half sets (`WorldItem.cs:868-877`).
    fn box_in_shimmer(&self, world: &World) -> bool {
        self.box_in_liquid(world, |t| tr_core::shimmer::tile_is_shimmer(t))
    }

    /// `Collision.WetCollision(position, width, height)` (`WorldItem.cs:870`):
    /// the drop's box overlaps any non-empty liquid cell. This is `base.wet`,
    /// the gate on most `UpdateItem_VisualEffects` light arms.
    pub fn box_is_wet(&self, world: &World) -> bool {
        self.box_in_liquid(world, |t| t.liquid > 0)
    }

    fn box_in_liquid(&self, world: &World, hit: impl Fn(&tr_core::tile::Tile) -> bool) -> bool {
        let x0 = (self.pos.x / tr_core::TILE_PX) as i32;
        let x1 = ((self.pos.x + SIZE) / tr_core::TILE_PX) as i32;
        let y0 = (self.pos.y / tr_core::TILE_PX) as i32;
        let y1 = ((self.pos.y + SIZE) / tr_core::TILE_PX) as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if let Some(t) = world.tile(tx, ty) {
                    if hit(t) {
                        return true;
                    }
                }
            }
        }
        false
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
    fn update_shimmer(&mut self, world: &World, dt: f32) {
        if self.shimmered {
            // `:457-508` — gravity is zero for the transformed item; the rise
            // applies over shimmer liquid (or already shimmerWet), floor -4.
            let over = self.shimmer_wet || self.over_shimmer(world);
            if over && self.vel.y > -4.0 {
                self.vel.y -= 0.05;
            } else if !over {
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
    /// reservation equality is the net-only term (the host owns every drop
    /// in the port). RNG-free. Returns the indices the caller must remove
    /// (descending — the caller `swap_remove`s them highest-first so the
    /// lower indices stay valid; the port's compaction deviation).
    pub(crate) fn combine_nearby(drops: &mut [ItemDrop], i: usize) -> Vec<usize> {
        let mut drained = Vec::new();
        let max = registry::max_stack(drops[i].kind);
        if drops[i].shimmer_time != 0.0
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

    /// One fixed 60 Hz step of `WorldItem.UpdateItem`'s physics half
    /// (`WorldItem.cs:338-453`): the shimmer update, the ground movement and
    /// the grab-delay decrement. The pickup pull/pick lives in the
    /// `GrabItems` step (`world_sim/items.rs`, S1-P-i47) — the C# runs it
    /// from `Player.Update` (`Player.cs:27428`), not from the item update.
    /// `being_grabbed` skips the ground physics for the tick after a pull
    /// (`:425-435`): wet flags clear, the velocity applies raw
    /// (`ApplyMovement`, `:1074-1084`), the flag resets.
    pub fn update(&mut self, world: &World, dt: f32) {
        self.age += dt;
        if self.being_grabbed {
            // `UpdateItem`'s grabbed arm (`WorldItem.cs:425-435`).
            self.shimmer_wet = false;
            self.being_grabbed = false;
            self.on_conveyor = false;
            self.pos += self.vel;
        } else {
            self.shimmer_wet = self.box_in_shimmer(world);
            self.update_shimmer(world, dt);
            if self.shimmered {
                // `WorldItem.cs:457-508` — the transformed item does not
                // fall; its shimmerTime decays via `update_shimmer`
                // (`:538-545`), it is not pinned at 1.
                self.pos += self.vel;
            } else {
                self.move_in_world(world);
            }
        }
        // `WorldItem.cs:441-444` — the grab delay ticks down every update,
        // grabbed or not.
        if self.grab_delay_time > 0 {
            self.grab_delay_time -= 1;
        }
    }

    /// The non-grabbed movement half: `MoveInWorld`'s gravity/friction head
    /// (`WorldItem.cs:849-864`) and the tile-collision pass.
    fn move_in_world(&mut self, world: &World) {
        // `WorldItem.cs:363-364` — the air gravity and the fall cap.
        self.vel.y = (self.vel.y + GRAVITY).min(MAX_FALL);
        // `WorldItem.cs:854-860` — the friction and the `0.1` rest snap, both
        // applied BEFORE the collision pass (`MoveInWorld`'s order).
        self.vel.x *= FRICTION_X;
        if self.vel.x.abs() < FRICTION_EPSILON {
            self.vel.x = 0.0;
        }
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
        let nx = self.pos.x + self.vel.x;
        if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
            // `Collision.TileCollision` (`Collision.cs:2415-2426`) leaves only
            // the residual that fit on the blocked axis (`result.X = …`), so a
            // drop stops dead against a wall instead of bouncing back.
            self.vel.x = 0.0;
        } else {
            self.pos.x = nx;
        }
        let ny = self.pos.y + self.vel.y;
        if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
            if self.vel.y > 0.0 {
                let ty = ((ny + SIZE) / TILE_PX).floor() as i32;
                self.pos.y = ty as f32 * TILE_PX - SIZE;
            } else {
                let ty = (ny / TILE_PX).floor() as i32;
                self.pos.y = (ty + 1) as f32 * TILE_PX;
            }
            // `Collision.cs:2446-2451`: `result.Y` collapses to the flush
            // residual (`+ 0.01f` for `gravDir == 1`), i.e. no restitution —
            // the drop lies where it landed.
            self.vel.y = 0.0;
        } else {
            self.pos.y = ny;
        }
    }

    /// Draws the drop in world space. Vanilla has NO positional bob —
    /// `Main.DrawItem` (`Main.cs:38602-38788`) draws at the resting position
    /// with the velocity spin `rotation = velocity.X * 0.2f` (`:38613`),
    /// suppressed on a conveyor (`item.onConveyor`) or after a shimmer
    /// transform (`:38614-38617`). The rotation pivots on the frame centre
    /// (`vector = frame.Size()/2`, `:38610`). Returns the quad count.
    pub fn draw(
        &self,
        r: &mut tr_render::SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        style: &LightStyle,
        anim_time: f32,
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
        if rot == 0.0 {
            r.push_sprite(
                [self.pos.x + 1.0, self.pos.y + 1.0],
                [SIZE, SIZE],
                uv,
                [c[0], c[1], c[2], 1.0],
            );
        } else {
            r.push_sprite_origin_rot(
                [self.pos.x + 1.0, self.pos.y + 1.0],
                [SIZE, SIZE],
                uv,
                [c[0], c[1], c[2], 1.0],
                rot,
                [SIZE * 0.5, SIZE * 0.5],
            );
        }
        1
    }
}

/// Hash of the item list for the G6 world-range key; includes the animation
/// step and the velocity spin's source (`Main.cs:38613`'s `velocity.X *
/// 0.2`) so a rolling drop redraws.
pub fn render_key(items: &[ItemDrop], anim_step: u32) -> u32 {
    let mut h = items.len() as u32 ^ (anim_step << 24);
    for it in items {
        h = h.wrapping_mul(31).wrapping_add(it.kind as u32);
        h = h.wrapping_mul(31).wrapping_add(it.pos.x.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.pos.y.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.count);
        h = h.wrapping_mul(31).wrapping_add(it.vel.x.to_bits());
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
        let drained = ItemDrop::combine_nearby(&mut drops, 0);
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
        let drained = ItemDrop::combine_nearby(&mut drops, 0);
        assert_eq!(drained, vec![1]);
        assert_eq!(drops[0].count, 8);
        // The Manhattan window: |dx| + |dy| <= 30 (`:167`).
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 0, 120.0, 111.0, 0.0, 0.0, 0.0), // 20+11=31
        ];
        let drained = ItemDrop::combine_nearby(&mut drops, 0);
        assert!(drained.is_empty());
        assert_eq!(drops[0].count, 5, "31 > 30 — no merge");
        // A different prefix never stacks (`Item.CanStack`).
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 81, 105.0, 100.0, 0.0, 0.0, 0.0),
        ];
        assert!(ItemDrop::combine_nearby(&mut drops, 0).is_empty());
        assert_eq!(drops[0].count, 5);
        // The shimmering drop waits (`:159`'s shimmerTime gate).
        let mut drops = vec![
            ItemDrop::new(9, 5, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            ItemDrop::new(9, 3, 0, 105.0, 100.0, 0.0, 0.0, 0.0),
        ];
        drops[0].shimmer_time = 0.5;
        assert!(ItemDrop::combine_nearby(&mut drops, 0).is_empty());
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
        // x 58..162, y 58..184. A 14x14 drop at x 44 (44+14 = 58) TOUCHES
        // the left edge — no intersect; at 45 (45+14 = 59) it overlaps.
        assert!(!grab_rect_intersects(100.0, 100.0, 20.0, 42.0, 42.0, Vec2::new(44.0, 100.0)));
        assert!(grab_rect_intersects(100.0, 100.0, 20.0, 42.0, 42.0, Vec2::new(45.0, 100.0)));
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
        // centre at (300, 200) — pos = centre - 7 (the 14x14 box).
        let mut heart = ItemDrop::new(58, 1, 0, 293.0, 193.0, 0.0, 0.0, 0.0);
        heart.being_grabbed = true;
        pull_item_pickup(heart.center(), &mut heart.vel, pc, 15.0, 5);
        // f32: 15/100 is inexact, so the blend lands within an ulp of -3
        // (the C# runs the same f32 arithmetic).
        assert!((heart.vel.x - -3.0).abs() < 1e-4, "15/5 blend from rest: {}", heart.vel.x);
        assert_eq!(heart.vel.y, 0.0);
        heart.update(&w, 1.0 / 60.0);
        assert!((heart.pos.x - 290.0).abs() < 1e-3, "the blend moves the drop: {}", heart.pos.x);
        assert_eq!(heart.pos.y, 193.0);
        // The second tick blends again: (−3*4 + −15)/5 = −5.4.
        heart.being_grabbed = true;
        pull_item_pickup(heart.center(), &mut heart.vel, pc, 15.0, 5);
        assert!((heart.vel.x - -5.4).abs() < 1e-4, "vel {}", heart.vel.x);
        let mut star = ItemDrop::new(184, 1, 0, 293.0, 193.0, 0.0, 0.0, 0.0);
        star.being_grabbed = true;
        pull_item_pickup(star.center(), &mut star.vel, pc, 12.0, 5);
        assert!((star.vel.x - -2.4).abs() < 1e-4, "12/5 blend: {}", star.vel.x);
    }
}
