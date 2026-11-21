//! W4-g87 — the Demon Heart + Minecart Powerup + Pet License permanent
//! upgrade USE arms (`Player.ItemCheck_OwnerOnlyCode`'s `Player.cs:45192-
//! 45424` cluster, the unclaimed remainder after the sibling lanes):
//!
//! * **Demon Heart 3335** — `ItemCheck_UseDemonHeart` (`:45192-45200`): the
//!   permanent expert accessory slot (`extraAccessory`, `:45197`). THE
//!   headline of the lane.
//! * **Minecart Powerup 5289** — `ItemCheck_UseMinecartPowerUp` (`:45202-
//!   45214`): the Super Cart unlock (`unlockedSuperCart`, `:45209`) + the
//!   `QuickSpawnItem(3353)` Minecart Mech (`:45212`).
//! * **Pet Licenses 4829/4830/4910** — `ItemCheck_UsePetLicenses` (`:45352-
//!   45366`) → `LicenseOrExchangePet` (`:45438-45445`) → `NPC.UnlockOrExchangePet`
//!   (`NPC.cs:80894-80918`): the `boughtCat`/`boughtDog`/`boughtBunny` world
//!   unlocks + the `Misc.License*Used` world text.
//!
//! ## The sibling lane boundaries (the g90 DOC-SWEEP census, docs/ANALYSIS.md)
//!
//! The rest of the `:45192-45424` cluster is LANDED elsewhere and is NOT here:
//! the Life Crystal 29 / Mana Crystal 109 (`crystals.rs`, g88), the Life Fruit
//! 1291 (`life_fruit.rs`, g88), the Artisan Loaf 5326 (g90), the Torch God's
//! Favor 5043 (f33), the Combat Book 4382 / Volume Two 5336 + the Peddler's
//! Satchel 5343 (g89/g90), and the shimmer permanents 5337/5338/5339/5340/
//! 5341/5342 (g89/g90). This module owns ONLY the three unclaimed families
//! above (verified against every W4 branch tip).
//!
//! ## The machine
//!
//! The Demon Heart + Minecart Powerup ride the permanent-upgrade
//! `ItemCheck` machine (g88's `life_fruit::permanent_upgrade_item_check`
//! shape, inlined here so the branch is self-contained — the merge dedupes
//! onto g88's shared helper): the use START on the `controlUseItem &&
//! releaseUseItem && itemAnimation == 0` press edge (`:42733`), the grant on
//! the first action tick inside the animation at `ItemTimeIsZero`
//! (`StartActualUse` leaves `itemTime` 0 — the boss-spawner precedent). Both
//! stock `useTime 30 / useAnimation 30 / useStyle 4` (`Item.cs`'s 3335/5289
//! rows), `autoReuse` false — one use per press.
//!
//! The Pet Licenses' gate is DIFFERENT (`:45440`): `ItemTimeIsZero &&
//! (!petBoughtFlag || NPC.AnyNPCs(npcType))`. The first purchase always passes
//! (`!petBoughtFlag`); a re-use only passes while the pet is alive (the
//! exchange). The port ships no TownCat/Dog/Bunny NPC kinds nor the
//! `TownNPCProfiles` variation table, so `NPC.AnyNPCs` is always false (a
//! re-use refuses) and the exchange's `RerollVariation` folds to the fail
//! branch — the unlock (first purchase) is the live path (ANALYSIS deviation).
//!
//! ## Persistence
//!
//! * The `.plr` (vanilla `Player.Serialize`) halves of `extraAccessory`
//!   (`:54776`) and the super-cart BitsByte (`:54914-54918`) are the f44
//!   verify-sweep's lane (`player_file.rs`, the G7 unlock-header block) —
//!   NOT this module's scope.
//! * The `.trplayer` live-session sidecar (`persist.rs`) carries
//!   `extra_accessory` + `unlocked_super_cart` on a new self-identifying
//!   post-loadout line (the g88 research/103 §6 rule-5 contract); the pet
//!   `bought_*` flags are WORLD globals (`.wld`/`.trwld`, never the sidecar).
//!
//! RNG: the Demon Heart + Pet Licenses draw NOTHING. The Minecart Powerup's
//! `QuickSpawnItem` rolls the Mech's prefix on the `UpdateItems` stream (the
//! bobber-catch drain's convention, `world_sim/projectiles.rs:569`). The
//! `NetMessage.SendData(4/7)` sync tails have no single-player port.

use crate::app::App;
use crate::item::ItemSlot;
use crate::registry;
use crate::world_sim::editing::start_swing_extras;

/// `ItemID.DemonHeart` 3335 (`ItemID.cs`, `Player.cs:45194`).
pub(crate) const DEMON_HEART: u16 = 3335;
/// `ItemID.MinecartPowerup` 5289 (`Player.cs:45204`).
pub(crate) const MINECART_POWERUP: u16 = 5289;
/// `ItemID.LicenseCat` 4829 / `LicenseDog` 4830 / `LicenseBunny` 4910
/// (`Player.cs:45354-45364`).
pub(crate) const LICENSE_CAT: u16 = 4829;
pub(crate) const LICENSE_DOG: u16 = 4830;
pub(crate) const LICENSE_BUNNY: u16 = 4910;
/// `ItemID.MinecartMech` 3353 — the Minecart Powerup's `QuickSpawnItem`
/// reward (`Player.cs:45212`).
const MINECART_MECH: u16 = 3353;

/// One 60 Hz tick in seconds (the `recall_teleport.rs` convention).
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The C# INTEGER tick comparisons out of the port's seconds timers
/// (`Player.cs:4064` `ItemTimeIsZero => itemTime == 0`). g88's
/// `life_fruit::ticks_of` twin (the merge dedupes).
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// The held item's prefixed `useTime` in seconds — `ApplyItemTime`
/// (`Player.cs:4416-4419` → `SetItemTime(useTime)`), the instance value.
/// g88's `life_fruit::use_time_secs` twin (the merge dedupes).
fn use_time_secs(held: ItemSlot) -> f32 {
    crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| crate::item::use_time_secs(held.kind))
}

/// The generic consumable tail (`Player.cs:43280-43326`) folded onto the
/// grant tick: one per use (`CanConsumeConsumableItem` true, no
/// ForceConsumption entry), then TurnToAir on the 0-stack (`:42824-42828`,
/// the drink seam's convention `potions.rs:474-487`). g88's
/// `life_fruit::consume_one` twin (the merge dedupes).
fn consume_one(app: &mut App) {
    let slot = &mut app.inventory[app.selected];
    if slot.count > 0 {
        slot.count -= 1; // `:43317-43320`
    }
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT; // `:42824-42828`
    }
}

/// The permanent-upgrade `ItemCheck` machine — g88's
/// `life_fruit::permanent_upgrade_item_check` shape (inlined for branch
/// self-containment; the merge dedupes onto the shared helper). The use
/// START (`ItemCheck_StartActualUse` → `ApplyItemAnimation` + the item's
/// UseSound, `:51680-51698`) on the `controlUseItem && releaseUseItem &&
/// itemAnimation == 0` edge (`:42733`); the grant on the first action tick
/// inside the animation (`ItemTimeIsZero`, `StartActualUse` leaves
/// `itemTime` 0). `grant` is the item's `ItemCheck_Use*` gate + body.
fn permanent_upgrade_item_check(
    app: &mut App,
    press_edge: bool,
    kind: u16,
    grant: fn(&mut App, ItemSlot) -> bool,
) -> bool {
    let held = app.inventory[app.selected];
    if held.kind != kind || held.count == 0 {
        return false;
    }
    // `:42598-42603` CCed → the animation zeroes and ItemCheck returns; the
    // dead player runs UpdateDead instead (the recall machine's head).
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.item_time = 0.0;
        return false;
    }
    if ticks_of(app.swing) > 0 {
        // The per-tick arm (`:43598-43599` run every tick of the animation):
        // the grant fires when ItemTimeIsZero inside the swing.
        grant(app, held);
        return false;
    }
    // The press edge (`:42733`): useStyle 4 (HoldUp) ≠ 0, autoReuse false.
    if app.mine_down && !app.inventory_open && press_edge {
        // ItemCheck_StartActualUse → ApplyItemAnimation + the item's
        // UseSound (`:51680-51698`, the shared start_swing_extras seam).
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        // StartActualUse leaves itemTime 0 → the grant fires THIS tick.
        app.item_time = 0.0;
        grant(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

/// `ItemCheck_UseDemonHeart`'s gate + body (`Player.cs:45194-45199`), run on
/// an action tick inside the animation. Returns true when the heart fired.
fn try_use_demon_heart(app: &mut App, held: ItemSlot) -> bool {
    // `:45194`: `!this.extraAccessory && Main.expertMode` + ItemTimeIsZero.
    // The caller owns the `itemAnimation > 0` half (the swing gate). The
    // expert mirror is `buffs.expert_mode` (the `is_item_slot_unlocked_and_usable`
    // slot-8 read's same flag, `player.rs:4880`).
    let can = app
        .player
        .as_ref()
        .is_some_and(|p| !p.extra_accessory && p.buffs.expert_mode);
    if !can || ticks_of(app.item_time) != 0 {
        return false;
    }
    // `:45196`: ApplyItemTime(sItem).
    app.item_time = use_time_secs(held);
    // `:45197`: extraAccessory = true — the permanent sixth accessory slot.
    // The `IsItemSlotUnlockedAndUsable` slot-8 gate (`:12587`) and the
    // equip-swap row-8 scan (`equip_swap.rs:104`) already consume the flag.
    if let Some(p) = app.player.as_mut() {
        p.extra_accessory = true;
    }
    // `:45198`: NetMessage.SendData(4) — the player-info sync, no SP port.
    // `:43280-43326`: the generic consumable tail — one per use.
    consume_one(app);
    true
}

/// The Demon Heart `ItemCheck` machine — called once per tick from
/// `update_editing` at the permanent-item seam (`Player.cs:43598`, after the
/// Mana Crystal `:43597`). Returns true when it STARTED a use this tick.
pub(crate) fn demon_heart_item_check(app: &mut App, press_edge: bool) -> bool {
    permanent_upgrade_item_check(app, press_edge, DEMON_HEART, try_use_demon_heart)
}

/// `ItemCheck_UseMinecartPowerUp`'s gate + body (`Player.cs:45204-45213`),
/// run on an action tick inside the animation. Unlike the other permanents
/// the use is NOT gated on the unlock flag — a second use still spawns a
/// Mech (the C# pays the stack each time, `:45204` has no `!unlockedSuperCart`).
fn try_use_minecart_powerup(app: &mut App, held: ItemSlot) -> bool {
    // `:45204`: `type == 5289 && itemAnimation > 0 && ItemTimeIsZero` — no
    // flag gate. The caller owns the swing half.
    if ticks_of(app.item_time) != 0 {
        return false;
    }
    // `:45206`: ApplyItemTime(sItem).
    app.item_time = use_time_secs(held);
    // `:45207-45211`: the Super Cart unlock ONCE.
    if let Some(p) = app.player.as_mut() {
        if !p.unlocked_super_cart {
            p.unlocked_super_cart = true; // `:45209`
            // `:45210`: NetMessage.SendData(4) — no SP port.
        }
    }
    // `:45212`: QuickSpawnItem(GetItemSource_OpenItem(5289), 3353) — the
    // Minecart Mech, ALWAYS (even on a re-use).
    quick_spawn_item(app, MINECART_MECH, 1);
    // `:43280-43326`: the generic consumable tail — one per use.
    consume_one(app);
    true
}

/// The Minecart Powerup `ItemCheck` machine — called once per tick from
/// `update_editing` at the permanent-item seam (`Player.cs:43599`, after the
/// Demon Heart `:43598`). Returns true when it STARTED a use this tick.
pub(crate) fn minecart_powerup_item_check(app: &mut App, press_edge: bool) -> bool {
    permanent_upgrade_item_check(app, press_edge, MINECART_POWERUP, try_use_minecart_powerup)
}

/// `Player.QuickSpawnItem(source, item, stack)` (`Player.cs:7170-7199`) →
/// `GetOrDropItem`: the item fills the owner's inventory first (`GetItem`'s
/// slot walk), the overflow drops at the player centre. The prefix rolls on
/// the `UpdateItems` stream (the bobber-catch drain's convention,
/// `world_sim/projectiles.rs:569`); the C# rolls it at spawn (`Prefix(-1)`,
/// `:7176`) and `GetItem` keeps it — the port's overflow-only draw is the
/// recorded order deviation.
fn quick_spawn_item(app: &mut App, kind: u16, stack: u32) {
    if stack == 0 {
        return;
    }
    let prefix =
        crate::prefix::roll_prefix(kind, -1, app.rngs.stream(crate::app::RngKey::UpdateItems));
    let max_stack = registry::max_stack(kind);
    let mut pending = stack;
    while pending > 0 {
        let Some(slot) = crate::item::slot_for_pickup(&mut app.inventory, kind, prefix) else {
            break;
        };
        let room = max_stack.saturating_sub(app.inventory[slot].count);
        if room == 0 {
            break;
        }
        let take = pending.min(room);
        app.inventory[slot].count += take;
        pending -= take;
    }
    if pending > 0 {
        if let Some(p) = app.player.as_ref() {
            let (cx, cy) = p.center();
            crate::world_sim::items::spawn_item_drop_px(
                &mut app.items,
                app.rngs.stream(crate::app::RngKey::UpdateItems),
                kind,
                pending,
                glam::Vec2::new(cx, cy),
            );
        }
    }
}

/// `ItemCheck_UsePetLicenses` (`Player.cs:45352-45366`) → `LicenseOrExchangePet`
/// (`:45438-45445`) → `NPC.UnlockOrExchangePet` (`NPC.cs:80894-80918`). The
/// grant gate (`:45440`) is `ItemTimeIsZero && (!petBoughtFlag ||
/// NPC.AnyNPCs(npcType))`.
fn try_use_pet_license(app: &mut App, held: ItemSlot, npc_type: i32, key: &str) -> bool {
    let bought = match npc_type {
        637 => app.world.bought_cat,
        638 => app.world.bought_dog,
        _ => app.world.bought_bunny,
    };
    // `NPC.AnyNPCs(npcType)` — no town-pet kinds shipped, always false.
    let any_npcs = false;
    // `:45440`: ItemTimeIsZero + (!bought || AnyNPCs). The caller owns the
    // outer `itemAnimation > 0` half (`:45354`).
    if ticks_of(app.item_time) != 0 || (bought && !any_npcs) {
        return false;
    }
    // `:45442`: ApplyItemTime(sItem).
    app.item_time = use_time_secs(held);
    // `NPC.UnlockOrExchangePet`'s `!petBoughtFlag` arm (`NPC.cs:80904-80909`):
    // raise the flag + broadcast the license text. (The exchange arms
    // `:80910-80917` need a live pet + the variation profile — unported; the
    // `NetMessage.TrySendData(7)` world sync `:80908` has no SP port.)
    match npc_type {
        637 => app.world.bought_cat = true,
        638 => app.world.bought_dog = true,
        _ => app.world.bought_bunny = true,
    }
    app.chat
        .new_text(crate::lang::text(key), crate::chat::COLOR_WORLD);
    // `:43280-43326`: the generic consumable tail — one per use.
    consume_one(app);
    true
}

/// The Pet License `ItemCheck` machine — called once per tick from
/// `update_editing` at the permanent-item seam (`Player.cs:43606`, after the
/// Peddler's Satchel `:43605`). The three licenses dispatch by kind; the
/// bought-flag gate lives inside the grant (the shared fn-pointer machine
/// cannot carry the per-license npc type + text key, so the license runs the
/// same press-edge start + per-tick grant inline). Returns true when it
/// STARTED a use this tick.
pub(crate) fn pet_license_item_check(app: &mut App, press_edge: bool) -> bool {
    let held = app.inventory[app.selected];
    let (kind, npc_type, key) = match held.kind {
        LICENSE_CAT => (LICENSE_CAT, 637, "Misc.LicenseCatUsed"),
        LICENSE_DOG => (LICENSE_DOG, 638, "Misc.LicenseDogUsed"),
        LICENSE_BUNNY => (LICENSE_BUNNY, 656, "Misc.LicenseBunnyUsed"),
        _ => return false,
    };
    if held.count == 0 {
        return false;
    }
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.item_time = 0.0;
        return false;
    }
    if ticks_of(app.swing) > 0 {
        try_use_pet_license(app, held, npc_type, key);
        return false;
    }
    if app.mine_down && !app.inventory_open && press_edge {
        app.swing_max = crate::item::use_animation_secs_of(kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, kind);
        app.attack_cd = app.swing_max;
        app.item_time = 0.0;
        try_use_pet_license(app, held, npc_type, key);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

// ---- witnesses (W4-g87) — WRITTEN NOT RUN (code-only mode, no compiler) ----
//
// The rig follows g88's `crystals.rs` convention: the item selected, audio
// MUTED (the use-sound draws are audio-only; muting keeps the RNG witnesses
// off the UpdatePlayers stream). The Demon Heart / Minecart Powerup ride the
// shared permanent-upgrade machine (the press-edge start + the ItemTimeIsZero
// grant); the Pet Licenses run their own inline machine (the bought-flag gate).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::World;

    /// The g88 crystal rig: the item selected, audio muted, a fresh player.
    /// `expert` arms `buffs.expert_mode` (the Demon Heart gate's mirror).
    fn rig(kind: u16, count: u32, expert: bool) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("g87".into(), 64, 64);
        let mut p = Player::spawn(&app.world);
        p.buffs.expert_mode = expert;
        app.player = Some(p);
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    fn press(app: &mut App) {
        app.mine_prev_down = false;
        app.mine_released = true;
        app.mine_down = true;
    }

    /// One per-tick decay + the machine's per-tick arm (the g88 `tick`
    /// convention — the swing/itemTime decay lives in the editing head, the
    /// witness drives it manually then runs the grant arm).
    fn tick_demon(app: &mut App) {
        app.swing = (app.swing - ONE_TICK_SECS).max(0.0);
        app.item_time = (app.item_time - ONE_TICK_SECS).max(0.0);
        demon_heart_item_check(app, false);
    }

    /// `:45194-45197`: in expert mode the Demon Heart grants the permanent
    /// `extraAccessory` slot, re-arms itemTime = useTime (30) and consumes one.
    #[test]
    fn w4_g87_the_demon_heart_grants_the_expert_slot_once() {
        let mut app = rig(DEMON_HEART, 2, true);
        press(&mut app);
        assert!(
            demon_heart_item_check(&mut app, true),
            "the press starts the use"
        );
        let p = app.player.as_ref().unwrap();
        assert!(p.extra_accessory, "extraAccessory = true (`:45197`)");
        assert_eq!(
            ticks_of(app.item_time),
            30,
            "ApplyItemTime re-arms useTime 30"
        );
        assert_eq!(app.inventory[app.selected].count, 1, "one consumed");
        // A second use refuses — the flag is already set (`!extraAccessory`).
        // Drain the swing + cadence, then re-press: the grant gate refuses.
        for _ in 0..40 {
            tick_demon(&mut app);
        }
        app.swing = 0.0;
        app.item_time = 0.0;
        app.attack_cd = 0.0;
        press(&mut app);
        demon_heart_item_check(&mut app, true);
        assert_eq!(
            app.inventory[app.selected].count, 1,
            "the re-use is refused (once ever)"
        );
    }

    /// `:45194`: in CLASSIC mode (no expert) the Demon Heart refuses — the
    /// `Main.expertMode` gate holds, the stack stays intact.
    #[test]
    fn w4_g87_the_demon_heart_refuses_in_classic_mode() {
        let mut app = rig(DEMON_HEART, 2, false);
        press(&mut app);
        // The machine starts the swing, but the grant gate (`!expert`) refuses.
        demon_heart_item_check(&mut app, true);
        assert!(
            !app.player.as_ref().unwrap().extra_accessory,
            "classic stays locked"
        );
        assert_eq!(app.inventory[app.selected].count, 2, "the refusal is free");
    }

    /// `:45204-45212`: the Minecart Powerup unlocks the Super Cart AND spawns
    /// the Minecart Mech (3353) into the inventory; a second use still spawns
    /// a Mech (no flag gate on the use, `:45204`).
    #[test]
    fn w4_g87_the_minecart_powerup_unlocks_the_cart_and_spawns_the_mech() {
        let mut app = rig(MINECART_POWERUP, 2, false);
        press(&mut app);
        assert!(
            minecart_powerup_item_check(&mut app, true),
            "the press starts the use"
        );
        assert!(
            app.player.as_ref().unwrap().unlocked_super_cart,
            "Super Cart unlocked (`:45209`)"
        );
        assert_eq!(
            ticks_of(app.item_time),
            30,
            "ApplyItemTime re-arms useTime 30"
        );
        assert_eq!(app.inventory[app.selected].count, 1, "one powerup consumed");
        // The Mech (3353) landed in the inventory (QuickSpawnItem → GetItem).
        assert!(
            app.inventory
                .iter()
                .any(|s| s.kind == MINECART_MECH && s.count >= 1),
            "the Minecart Mech spawned (`:45212`)"
        );
    }

    /// `:45354-45357` / `NPC.cs:80904-80909`: the Cat License raises
    /// `boughtCat` once and posts the `Misc.LicenseCatUsed` world text; a
    /// re-use refuses (no live pet — `NPC.AnyNPCs` false).
    #[test]
    fn w4_g87_the_cat_license_unlocks_once_and_posts_the_text() {
        let mut app = rig(LICENSE_CAT, 2, false);
        press(&mut app);
        assert!(
            pet_license_item_check(&mut app, true),
            "the press starts the use"
        );
        assert!(app.world.bought_cat, "boughtCat = true");
        assert_eq!(app.inventory[app.selected].count, 1, "one license consumed");
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.color == crate::chat::COLOR_WORLD),
            "the license world text posted"
        );
        // Re-use refuses: the flag is set and no pet is alive.
        for _ in 0..40 {
            app.swing = (app.swing - ONE_TICK_SECS).max(0.0);
            app.item_time = (app.item_time - ONE_TICK_SECS).max(0.0);
            pet_license_item_check(&mut app, false);
        }
        app.swing = 0.0;
        app.item_time = 0.0;
        app.attack_cd = 0.0;
        press(&mut app);
        pet_license_item_check(&mut app, true);
        assert_eq!(
            app.inventory[app.selected].count, 1,
            "the re-use is refused"
        );
    }

    /// The Dog / Bunny licenses raise their own world flags (the three
    /// licenses dispatch by kind, `:45358-45365`).
    #[test]
    fn w4_g87_the_dog_and_bunny_licenses_raise_their_own_flags() {
        let mut dog = rig(LICENSE_DOG, 1, false);
        press(&mut dog);
        assert!(pet_license_item_check(&mut dog, true));
        assert!(dog.world.bought_dog, "boughtDog");
        assert!(!dog.world.bought_cat, "the cat flag stays false");

        let mut bunny = rig(LICENSE_BUNNY, 1, false);
        press(&mut bunny);
        assert!(pet_license_item_check(&mut bunny, true));
        assert!(bunny.world.bought_bunny, "boughtBunny");
        assert!(!bunny.world.bought_dog, "the dog flag stays false");
    }

    /// RNG census: the Demon Heart + Pet License uses draw NOTHING on the
    /// UpdatePlayers stream (the C# arms make no rand call; the muted sound
    /// draws nothing either). The Minecart Powerup's Mech prefix roll rides
    /// UpdateItems, NOT UpdatePlayers.
    #[test]
    fn w4_g87_the_demon_heart_and_license_draw_no_updateplayers() {
        let mut app = rig(DEMON_HEART, 1, true);
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(0x687);
        press(&mut app);
        demon_heart_item_check(&mut app, true);
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone(),
            tr_core::UnifiedRandom::new(0x687),
            "the Demon Heart use draws nothing on UpdatePlayers"
        );

        let mut lic = rig(LICENSE_CAT, 1, false);
        *lic.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(0x687);
        press(&mut lic);
        pet_license_item_check(&mut lic, true);
        assert_eq!(
            lic.rngs.stream(crate::app::RngKey::UpdatePlayers).clone(),
            tr_core::UnifiedRandom::new(0x687),
            "the license use draws nothing on UpdatePlayers"
        );
    }
}
