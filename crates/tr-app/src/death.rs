//! The death pipeline — `Player.KillMe`'s drop half (`Player.cs:38863`):
//! `DropItems(gemsOnly)` (`:54302`), `DropCoins` (`:54261`) and
//! `DropTombstone` (`:39091`).
//!
//! Softcore/journey death (`difficulty == 0 || == 3`) drops only the large
//! gems the player carries and half the carried coins; mediumcore/hardcore
//! drop the whole carry (the starter tools re-granted) and keep the purse.
//! Every death spawns the falling tombstone projectile that places a
//! tombstone tile on landing.

use glam::Vec2;

use crate::app::{App, RngKey};
use crate::regen::DotDeath;
use tr_core::TILE_PX;

/// `Player.TryDroppingSingleItem` (`Player.cs:54364-54380`): one death drop
/// through `Item.NewItem(velocity: new Vector2(Main.rand.Next(-20, 21) *
/// 0.2f, Main.rand.Next(-20, 1) * 0.2f), X: (int)position.X, Y:
/// (int)position.Y, Width: width, Height: height, ...)` — the X/Y overload
/// (`Item.cs:49649-49652`) centres the spawn at `(X + Width/2, Y +
/// Height/2)`, i.e. the player centre with the position truncated first.
/// The EXPLICIT velocity skips `Item.NewItem`'s scatter draws
/// (`Item.cs:49738-49741`) and the carried prefix rolls nothing
/// (`Item.cs:511-514`) — exactly TWO integer draws per drop
/// (`Next(-20, 21)` then `Next(-20, 1)`), on the caller's `UpdatePlayers`
/// stream (`KillMe` runs inside `Player.Update`, `Main.cs:17754`). No
/// ownership argument → `NewItemOwnership.None`, no grab delay in SP.
fn try_drop_single_item(
    app: &mut App,
    kind: u16,
    count: u32,
    prefix: u16,
    x: f32,
    y: f32,
) {
    if count == 0 {
        // `stack > 0` gate (`Player.cs:54371`); `netMode != 1` folds open.
        return;
    }
    let rng = app.rngs.stream(RngKey::UpdatePlayers);
    let vx = rng.next_range(-20, 21) as f32 * 0.2;
    let vy = rng.next_range(-20, 1) as f32 * 0.2;
    // `(int)position` truncation, then the Width/2, Height/2 centre
    // (`Item.cs:49651`; width 20, height 42 — `Player.defaultWidth/Height`).
    let cx = x as i32 as f32 + crate::player::WIDTH * 0.5;
    let cy = y as i32 as f32 + crate::player::HEIGHT * 0.5;
    app.items.push(crate::item::ItemDrop::new(
        kind,
        count,
        prefix,
        cx - crate::item::SIZE * 0.5,
        cy - crate::item::SIZE * 0.5,
        vx,
        vy,
        0.0,
    ));
}

/// `Player.DropItems(gemsOnly)` (`Player.cs:54302-54362`): softcore/journey
/// death (`difficulty == 0 || == 3`, `KillMe` `:38863`) drops only the large
/// gems from the inventory (`ItemID` 1522-1527 = LargeAmethyst..LargeDiamond,
/// 3643 = LargeAmber, `:54312`). Mediumcore (1) / hardcore (2) drop the whole
/// carry: the starter tools (3507 Copper Shortsword, 3506 Copper Axe, 3509
/// Copper Pickaxe) `TurnToAir` instead (`:54322-54325`), everything else in
/// the inventory, then armor, dye, miscEquips, miscDyes drops
/// (`:54319-54350`); the loadout sweep (`:54351-54355`) is unported (no
/// loadout system), and the starter re-grant into slots 0/1/2
/// (`:54356-54361`, `Prefix(-1)` = the natural spawn roll) closes the pass.
fn drop_items(app: &mut App, x: f32, y: f32, gems_only: bool) {
    // `:54304` — the trash airs first.
    app.equip.trash = crate::item::EMPTY_SLOT;
    const GEMS: [u16; 7] = [1522, 1523, 1524, 1525, 1526, 1527, 3643];
    // The slot snapshot: the C# iterates `this.inventory` by reference and
    // `TryDroppingSingleItem` mutates each slot in place; the port's spawn
    // helper takes the whole `&mut App`, so the walk reads a copy first
    // (the drop ORDER is the slot order either way).
    let inventory = app.inventory;
    if gems_only {
        // `:54309-54316`: the inventory walk in slot order, gems only. The
        // C# array is the full `inventory[59]` — the cursor mirror 58 is
        // part of it (a gem held on the cursor at the death moment drops).
        for (i, slot) in inventory.iter().enumerate() {
            if slot.count > 0 && GEMS.contains(&slot.kind) {
                try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
                app.inventory[i] = crate::item::EMPTY_SLOT;
            }
        }
        if app.mouse_item.count > 0 && GEMS.contains(&app.mouse_item.kind) {
            let it = app.mouse_item;
            try_drop_single_item(app, it.kind, it.count, it.prefix, x, y);
            app.mouse_item = crate::item::EMPTY_SLOT;
        }
        return;
    }
    // `:54319-54330`: the full inventory walk — the starter tools vanish,
    // every other stack drops (`TryDroppingSingleItem` subtracts the whole
    // stack and `TurnToAir`s the slot, `:54375-54379`).
    for (i, slot) in inventory.iter().enumerate() {
        if slot.count == 0 {
            continue;
        }
        if !matches!(slot.kind, 3507 | 3506 | 3509) {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
        }
        app.inventory[i] = crate::item::EMPTY_SLOT;
    }
    // The C# inventory walk covers `this.inventory[59]` (`Player.cs:1576`):
    // grid 0..49, then the coin rows 50..53, the ammo rows 54..57 and the
    // cursor mirror 58. The port splits them (`EquipSlots::coins`/`ammo`,
    // `App.mouse_item`) — each row drops in the same slot order, before the
    // armor arrays. (The purse `App.coins` is the port's copper-total fold;
    // the coin ROWS drop their stacks like every other slot here.)
    let coins = app.equip.coins;
    for (i, slot) in coins.iter().enumerate() {
        if slot.count > 0 {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
            app.equip.coins[i] = crate::item::EMPTY_SLOT;
        }
    }
    let ammo = app.equip.ammo;
    for (i, slot) in ammo.iter().enumerate() {
        if slot.count > 0 {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
            app.equip.ammo[i] = crate::item::EMPTY_SLOT;
        }
    }
    // `inventory[58]` — the cursor mirror (`:5130` writes it every check).
    // `TryDroppingSingleItem` airs the slot, so the later `KillMe` return
    // (`:38937-38944`) sees air and the cursor ends empty.
    if app.mouse_item.count > 0 {
        let it = app.mouse_item;
        try_drop_single_item(app, it.kind, it.count, it.prefix, x, y);
        app.mouse_item = crate::item::EMPTY_SLOT;
    }
    // `:54331-54350`: armor → dye → miscEquips → miscDyes, each array in
    // slot order (the port's `EquipSlots` split; the arrays differ in
    // length, so each walks on its own).
    let armor = app.equip.armor;
    for (i, slot) in armor.iter().enumerate() {
        if slot.count > 0 {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
            app.equip.armor[i] = crate::item::EMPTY_SLOT;
        }
    }
    let dye = app.equip.dye;
    for (i, slot) in dye.iter().enumerate() {
        if slot.count > 0 {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
            app.equip.dye[i] = crate::item::EMPTY_SLOT;
        }
    }
    let misc_equips = app.equip.misc_equips;
    for (i, slot) in misc_equips.iter().enumerate() {
        if slot.count > 0 {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
            app.equip.misc_equips[i] = crate::item::EMPTY_SLOT;
        }
    }
    let misc_dyes = app.equip.misc_dyes;
    for (i, slot) in misc_dyes.iter().enumerate() {
        if slot.count > 0 {
            try_drop_single_item(app, slot.kind, slot.count, slot.prefix, x, y);
            app.equip.misc_dyes[i] = crate::item::EMPTY_SLOT;
        }
    }
    // `:54356-54361`: the starter tools return — shortsword, pickaxe, axe
    // into slots 0/1/2, each `SetDefaults(kind)` + `Prefix(-1)` — the NATURAL
    // spawn roll (1/4 no prefix, the reduced set 1/3 none, `Item.cs:544-559`).
    // The tools sit in `PrefixLegacy.ItemSets.SwordsHammersAxesPicks`
    // (3506/3507/3509), so they DO roll — the draws ride the same
    // `UpdatePlayers` stream (`Item.Prefix` reads `Main.rand`, `Item.cs:528`),
    // in slot order 3507 → 3509 → 3506.
    for (i, kind) in [(0usize, 3507u16), (1, 3509), (2, 3506)] {
        let prefix = crate::prefix::roll_prefix(kind, -1, app.rngs.stream(RngKey::UpdatePlayers));
        app.inventory[i] = crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix,
        };
    }
}

/// `Player.DropCoins` (`Player.cs:54261-54294`): per coin stack,
/// `int num2 = item.stack / 2` is KEPT and `int num3 = item.stack - num2` (the
/// ceiling half) is dropped — `/ 4` in expert, `0` in master, the ported
/// difficulty being classic. The return value is the dropped value
/// (`num += num3 * denomination`), which `KillMe` books as `lostCoins`
/// (`:38931-38936`). The port's coins live in `App.coins` (a copper total),
/// so the purse keeps `coins / 2` and the ceiling half is split greedily into
/// platinum/gold/silver/copper stacks.
/// `Main.ValueToCoins` (`Main.cs:55220-55264`): the copper total → the
/// "1 platinum 2 gold 50 silver 10 copper" string from the localized
/// `Currency.*` names (`.ToLower()` like the C#), the zero denominations
/// skipped, the trailing space stripped — and an empty purse returns ""
/// (the C# prints no copper at 0).
pub fn value_to_coins(value: u64) -> String {
    let mut num = value;
    let mut platinum = 0u64;
    let mut gold = 0u64;
    let mut silver = 0u64;
    while num >= 1_000_000 {
        num -= 1_000_000;
        platinum += 1;
    }
    while num >= 10_000 {
        num -= 10_000;
        gold += 1;
    }
    while num >= 100 {
        num -= 100;
        silver += 1;
    }
    let copper = num;
    let mut text = String::new();
    if platinum > 0 {
        text += &format!(
            "{} {} ",
            platinum,
            crate::lang::text("Currency.Platinum").to_lowercase()
        );
    }
    if gold > 0 {
        text += &format!(
            "{} {} ",
            gold,
            crate::lang::text("Currency.Gold").to_lowercase()
        );
    }
    if silver > 0 {
        text += &format!(
            "{} {} ",
            silver,
            crate::lang::text("Currency.Silver").to_lowercase()
        );
    }
    if copper > 0 {
        text += &format!(
            "{} {} ",
            copper,
            crate::lang::text("Currency.Copper").to_lowercase()
        );
    }
    text.trim_end().to_string()
}

pub fn drop_coins(app: &mut App, x: f32, y: f32) -> u64 {
    // `:54270-54279`: `num2 = stack / 2` stays, `num3 = stack - num2` drops
    // (classic; expert `/ 4`, master `0` — constants-off, the port's
    // difficulty plumbing). The port's purse is a copper total, so the
    // halving rides the total (the recorded fold — the C# halves each coin
    // STACK); mediumcore/hardcore keep the whole purse (`KillMe`
    // `:38925-38936` runs `DropCoins` for difficulty 0/3 only).
    let dropped = app.coins - app.coins / 2;
    if dropped == 0 {
        return 0;
    }
    app.coins -= dropped;
    let mut rest = dropped;
    for (coin_item, denom, cap) in [
        (74u16, 1_000_000u64, 999u64),
        (73, 10_000, 999),
        (72, 100, 999),
        (71, 1, 999),
    ] {
        let count = (rest / denom).min(cap);
        if count == 0 {
            continue;
        }
        rest -= count * denom;
        // `TryDroppingSingleItem` (`Player.cs:54296` → `:54373`): the coin
        // stack drops through the same two-draw velocity pair as every
        // death drop.
        try_drop_single_item(app, coin_item, count as u32, 0, x, y);
    }
    // The vanilla split drops the exact remainder (no denomination cap on the
    // last tier beyond the stack limit; stacks > 999 would split — the ported
    // coin totals stay under it).
    if rest > 0 {
        try_drop_single_item(app, 71, rest as u32, 0, x, y);
    }
    dropped
}

/// The falling tombstone projectile (`Projectile` ids 43/201-205/527-531,
/// `ProjectileID.Tombstone`/`GraveMarker`/.../`RichGraveStone5`). It falls
/// with the scattered X velocity and, on landing, places a Tombstones tile
/// (tile 85, style = the projectile's index) and records the epitaph.
pub struct TombFall {
    pub pos: Vec2,
    pub vel: Vec2,
    /// The vanilla projectile id — also the tile-85 style index.
    pub kind: u16,
    pub text: String,
    pub alive: bool,
}

/// `Player.DropTombstone`'s projectile roll (`Player.cs:39091-39115`):
/// coinsOwned > 100000 -> a rich grave (`Next(5) + 527`), else `Next(6)`:
/// 0 -> the plain Tombstone (43), else a grave marker (`200 + roll`).
/// The X velocity: `Next(-35, 36) * 0.1` until the magnitude clears 2.
pub fn roll_tombstone(rng: &mut tr_core::UnifiedRandom, coins_owned: u64) -> TombFall {
    // `Player.DropTombstone` (`Player.cs:39091-39115`): `Next(-35, 36)`,
    // the |vx| >= 2 retry with `Next(-30, 31)`, `Next(6)` and the rich
    // grave's `Next(5) + 527` — integer draws on the caller's stream.
    let mut vx = rng.next_range(-35, 36) as f32 * 0.1;
    while vx >= -2.0 && vx <= 2.0 {
        vx += rng.next_range(-30, 31) as f32 * 0.1;
    }
    let roll = rng.next_max(6);
    let kind = if coins_owned > 100_000 {
        rng.next_max(5) as u16 + 527
    } else if roll != 0 {
        200 + roll as u16
    } else {
        43
    };
    TombFall {
        pos: Vec2::ZERO,
        vel: Vec2::new(vx, 0.0),
        kind,
        text: String::new(),
        alive: true,
    }
}

/// One fixed 60 Hz step for every falling tombstone: gravity like the
/// tombstone projectile's (`Projectile.ai[1]`-driven fall), landing when the
/// tile below is solid — then the Tombstones tile is placed through
/// `TileObjectData` and the epitaph recorded for the render.
pub fn tick_tomb_falls(app: &mut App, dt: f32) {
    let gravity = 0.3;
    for tomb in app.tomb_falls.iter_mut() {
        if !tomb.alive {
            continue;
        }
        tomb.vel.y = (tomb.vel.y + gravity * dt * 60.0).min(10.0);
        let nx = tomb.pos.x + tomb.vel.x * dt * 60.0;
        let ny = tomb.pos.y + tomb.vel.y * dt * 60.0;
        tomb.pos = Vec2::new(nx, ny);
        let tx = (nx / TILE_PX).floor() as i32;
        let ty = (ny / TILE_PX + 2.0).floor() as i32; // the 2-tall tile's base
        let solid_below = app
            .world
            .tile(tx, ty)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
        if solid_below && tomb.vel.y > 0.0 {
            tomb.alive = false;
            // The tombstone rests 1 tile above the solid base; the style is
            // the projectile's index in the tile's style row.
            let style: i32 = match tomb.kind {
                43 => 0,
                201..=205 => (tomb.kind - 200) as i32,
                527..=531 => (tomb.kind - 527) as i32 + 6,
                _ => 0,
            };
            let px = tomb.pos.x as i32;
            let py = tomb.pos.y as i32;
            if tr_core::edit::place_item_tile(&mut app.world, px, py, 85, style, 1) {
                app.tomb_texts.push((px, py, tomb.text.clone()));
                app.world_key = None;
            }
        }
    }
    app.tomb_falls.retain(|t| t.alive);
}

/// `Lang.cs:1045-1127` `CreateDeathMessage`'s `other` switch — the
/// `PlayerDeathReason.ByOther(N)` → `DeathText.<base>` row family and its
/// variant count, per DoT attribution. The `Main.rand.Next(n) + 1` variant
/// roll the C# rides is passed in (`death_message`).
pub fn death_text_key(death: DotDeath) -> (&'static str, u32) {
    match death {
        DotDeath::Suffocation => ("DeathText.Suffocated", 2),
        DotDeath::Burned => ("DeathText.Burned", 4),
        DotDeath::Poison => ("DeathText.Poisoned", 1),
        DotDeath::Electrified => ("DeathText.Electrocuted", 4),
        DotDeath::Tongued => ("DeathText.WasLicked", 2),
        DotDeath::VampireSunlight => ("DeathText.VampireBurningInDaylight", 6),
        DotDeath::ChlorophyteSpore => ("DeathText.Spored", 1),
        // W4-g89f — the rod's chaos-state death (`Lang.cs:1095-1103`):
        // ByOther(13) Teleport_1, ByOther(14/15) the gendered rows (the
        // SAME en-US text; the port has no Male flag — ANALYSIS W4-g89f).
        DotDeath::Teleport => ("DeathText.Teleport_1", 1),
        DotDeath::TeleportGendered => ("DeathText.Teleport_2_Male", 1),
    }
}

/// `PlayerDeathReason.GetDeathText`: the `DeathText.{base}_{variant}` row
/// formatted with the dead player's name (the key rows carry `{0}`).
/// `variant` is the C# `Main.rand.Next(variants) + 1` result — the caller
/// draws it from its own named stream (`Lang.cs:1057-1127`).
pub fn death_message(death: DotDeath, variant: i32, player_name: &str) -> String {
    let (base, variants) = death_text_key(death);
    // The single-variant rows (`DeathText.Poisoned`, `Slain`, `Spored`) carry
    // no `_N` suffix; the families append the drawn index.
    let key = if variants <= 1 {
        base.to_string()
    } else {
        format!("{base}_{variant}")
    };
    crate::lang::text_args(&key, &[player_name.to_string()])
}

/// `DeathText.Slain` — the non-attributed death row (`Lang.cs:1045`'s
/// `case 4`), formatted with the player name.
pub fn slain_message(player_name: &str) -> String {
    crate::lang::text_args("DeathText.Slain", &[player_name.to_string()])
}

/// The death entry point: the drops at the death spot plus the falling
/// tombstone. `death_text` is the fully-resolved message (the C# NetworkText
/// already carries the player name inside — `"{0} was slain..."`), so no name
/// is prepended here.
///
/// `(x, y)` is the death spot's TOP-LEFT (`base.position`): `KillMe`'s
/// `DropItems`/`DropCoins` spawn at `(int)position + Width/2, Height/2`
/// (`Player.cs:54373` → `Item.cs:49651`). The npcs-pass caller hands the
/// pre-death position, not the centre.
pub fn on_player_death(app: &mut App, x: f32, y: f32, death_text: &str) {
    // `KillMe` `:38863`: `DropItems(difficulty == 0 || difficulty == 3)` —
    // softcore/journey gems-only, mediumcore/hardcore the whole carry. The
    // difficulty rides the session draft (`App::active_player_difficulty`,
    // `Main.ActivePlayerFileData.Player.difficulty`'s stand-in).
    let difficulty = app.active_player_difficulty();
    let gems_only = difficulty == 0 || difficulty == 3;
    drop_items(app, x, y, gems_only);
    // `KillMe`'s coin bookkeeping: `:38849-38853` books `lostCoins =
    // coinsOwned` for EVERY difficulty first; `:38925-38936` overwrites with
    // `DropCoins()`'s dropped value for difficulty 0/3 only (softcore halves
    // the purse, mediumcore/hardcore keep it whole). The death screen's
    // "Dropped coins" line reads `lostCoinString` (`Main.cs:45333`). The
    // rich-grave gate (`:38945` `DropTombstone(coinsOwned, ...)`) reads the
    // pre-death purse, so capture it before the halving.
    let coins_owned = app.coins;
    // `:38849-38853`: `lostCoins = coinsOwned` books the WHOLE purse first
    // (every difficulty — the mediumcore death screen shows it even though
    // the purse stays); `:38925-38936` overwrites with the DROPPED value for
    // difficulty 0/3 only.
    app.lost_coins = coins_owned;
    if gems_only {
        app.lost_coins = drop_coins(app, x, y);
    }
    app.lost_coin_string = value_to_coins(app.lost_coins);
    // `KillMe` `:38937-38944` — the cursor return: `inventory[58]` goes back
    // through `GetItem(ReturnItemFromSlot)` and `Main.mouseItem` reads the
    // REMAINDER (the plain `GetItem`, NOT `GetOrDropItem` — the overflow
    // stays on the cursor, no world spawn). The gems-only walk did NOT drop
    // a non-gem cursor stack (the `:54312` filter), so it returns here; the
    // mediumcore walk already dropped + aired it (`inventory[58]` is part of
    // the 59-slot walk), so this is a no-op there. The return's inventory
    // fill plays the `IsACoin ? 38 : 7` chime
    // (`GetItem_FillIntoOccupiedSlot` `Player.cs:39491` —
    // `ReturnItemFromSlot` is NOT NoSound, `GetItemSettings.cs:59`), on the
    // `UpdatePlayers` stream, once per pickup event (the port's merged-fill
    // convention, `world_sim/items.rs`'s pickup chime).
    if app.mouse_item.count > 0 {
        let it = app.mouse_item;
        let max_stack = crate::registry::max_stack(it.kind);
        let mut pending = it.count;
        while pending > 0 {
            let Some(slot) =
                crate::item::slot_for_pickup(&mut app.inventory, it.kind, it.prefix)
            else {
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
        if it.count > pending {
            app.audio.play_pickup_sound(
                crate::registry::is_coin(it.kind),
                app.rngs.stream(RngKey::UpdatePlayers),
            );
        }
        // The remainder stays on the cursor (`Main.mouseItem = inventory[58]`).
        app.mouse_item.count = pending;
        if pending == 0 {
            app.mouse_item = crate::item::EMPTY_SLOT;
        }
    }
    app.death_text = death_text.to_string();
    let mut tomb = {
        let rng = app.rngs.stream(RngKey::UpdatePlayers);
        roll_tombstone(rng, coins_owned)
    };
    tomb.pos = Vec2::new(x, y);
    tomb.text = death_text.to_string();
    app.tomb_falls.push(tomb);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

    #[test]
    fn death_text_key_matches_the_by_other_table() {
        // DW-1.2/DW-1.3 — Lang.cs:1073-1126: the ByOther(N) switch both drain
        // lanes feed (`Player.cs:19321-19331,19362-19385`); the variant counts
        // are the `Main.rand.Next(n) + 1` families.
        assert_eq!(death_text_key(DotDeath::Suffocation), ("DeathText.Suffocated", 2)); // 7
        assert_eq!(death_text_key(DotDeath::Burned), ("DeathText.Burned", 4)); // 8
        assert_eq!(death_text_key(DotDeath::Poison), ("DeathText.Poisoned", 1)); // 9
        assert_eq!(death_text_key(DotDeath::Electrified), ("DeathText.Electrocuted", 4)); // 10
        assert_eq!(death_text_key(DotDeath::Tongued), ("DeathText.WasLicked", 2)); // 12
        assert_eq!(
            death_text_key(DotDeath::VampireSunlight),
            ("DeathText.VampireBurningInDaylight", 6)
        ); // 22
        assert_eq!(death_text_key(DotDeath::ChlorophyteSpore), ("DeathText.Spored", 1)); // 23
    }

    #[test]
    fn death_message_picks_the_variant_and_formats_the_name() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        // The C# variant index — `Next(2) + 1` → 1 or 2.
        assert_eq!(death_message(DotDeath::Suffocation, 1, "Bob"), "Bob couldn't breathe.");
        assert_eq!(death_message(DotDeath::Suffocation, 2, "Bob"), "Bob was buried alive.");
        // A single-variant family ignores the index.
        assert_eq!(
            death_message(DotDeath::Poison, 1, "Bob"),
            "Bob couldn't find the antidote."
        );
        assert_eq!(slain_message("Bob"), "Bob was slain...");
    }

    #[test]
    fn tombstone_roll_keeps_the_vanilla_id_set() {
        // Player.cs:39091-39115: normal deaths roll Next(6) -> 0 is the plain
        // Tombstone (43), 1..5 are the grave markers (201-205); rich deaths
        // (coins > 100000) roll Next(5) + 527 (the rich gravestones).
        for seed in 1..200i32 {
            let t = roll_tombstone(&mut UnifiedRandom::new(seed), 50);
            assert!(
                t.kind == 43 || (201..=205).contains(&t.kind),
                "normal roll produced kind {}",
                t.kind
            );
        }
        for seed in 1..200i32 {
            let t = roll_tombstone(&mut UnifiedRandom::new(seed), 200_000);
            assert!(
                (527..=531).contains(&t.kind),
                "rich roll produced kind {}",
                t.kind
            );
        }
    }

    #[test]
    fn tombstone_x_scatter_clears_two_pixels() {
        // Player.cs:39093-39095: the X velocity repeats Next(-30,31)*0.1
        // until it clears the 2 px dead zone.
        for seed in 1..100i32 {
            let t = roll_tombstone(&mut UnifiedRandom::new(seed), 50);
            assert!(t.vel.x < -2.0 || t.vel.x > 2.0, "vx {}", t.vel.x);
        }
    }

    #[test]
    fn drop_coins_halving_math_is_the_greedy_split() {
        // The greedy platinum/gold/silver/copper split of the dropped half:
        // 500 copper -> 5 silver (100 each) + nothing else; the totals sum
        // back to exactly the dropped value.
        let dropped = 500u64;
        let mut rest = dropped;
        let mut stacks: Vec<(u64, u64)> = Vec::new();
        for (denom, cap) in [(1_000_000u64, 999u64), (10_000, 999), (100, 999), (1, 999)] {
            let count = (rest / denom).min(cap);
            if count > 0 {
                rest -= count * denom;
                stacks.push((denom, count));
            }
        }
        let total: u64 = stacks.iter().map(|(d, c)| d * c).sum();
        assert_eq!(total, dropped);
        // 500 copper = 5 silver: no platinum/gold, one 5-stack of silver.
        assert_eq!(stacks, vec![(100, 5)]);
    }

#[cfg(test)]
mod revenge_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::UnifiedRandom;

    #[test]
    fn value_to_coins_skips_the_zero_denominations() {
        // `Main.ValueToCoins` (`Main.cs:55220-55264`): the greedy split, the
        // zero tiers skipped, the trailing space stripped, and the empty
        // purse returns "" (the C# prints no copper at 0).
        assert_eq!(value_to_coins(0), "");
        assert_eq!(value_to_coins(10), format!("10 {}", crate::lang::text("Currency.Copper").to_lowercase()));
        assert_eq!(value_to_coins(150), format!("1 {} 50 {}", crate::lang::text("Currency.Silver").to_lowercase(), crate::lang::text("Currency.Copper").to_lowercase()));
        assert_eq!(value_to_coins(10_050), format!("1 {} 50 {}", crate::lang::text("Currency.Gold").to_lowercase(), crate::lang::text("Currency.Copper").to_lowercase()));
        assert_eq!(value_to_coins(1_010_050), format!("1 {} 1 {} 50 {}", crate::lang::text("Currency.Platinum").to_lowercase(), crate::lang::text("Currency.Gold").to_lowercase(), crate::lang::text("Currency.Copper").to_lowercase()));
        assert_eq!(value_to_coins(2_000_000), format!("2 {}", crate::lang::text("Currency.Platinum").to_lowercase()));
    }

    #[test]
    fn the_death_sets_the_lost_coins_state() {
        // `KillMe`'s softcore arm (`Player.cs:38931-38936`): lostCoins =
        // DropCoins() (half the purse), lostCoinString = ValueToCoins, the
        // death text rides the broadcast.
        let mut app = flow_app(None, vec![], vec![]);
        app.player_name = "Tester".into();
        app.coins = 10_050; // 1 gold 50 silver 50 copper → drops 5025
        let before = app.coins;
        on_player_death(&mut app, 100.0, 100.0, "Tester was slain...");
        assert!(app.lost_coins > 0 && app.lost_coins < before);
        assert_eq!(app.lost_coins, before - app.coins, "the purse halves");
        assert_eq!(
            app.lost_coin_string,
            value_to_coins(app.lost_coins),
            "the string mirrors the total"
        );
        assert_eq!(app.death_text, "Tester was slain...");
        // The coin items landed on the ground (the drop pass).
        assert!(app
            .items
            .iter()
            .any(|it| (71..=74).contains(&it.kind)), "the coin stacks dropped");
    }

    #[test]
    fn the_zero_purse_leaves_the_state_clean() {
        // `DropCoins`'s 0-gate: no drop, the lost state 0, the death screen
        // skips the coins line.
        let mut app = flow_app(None, vec![], vec![]);
        app.player_name = "Tester".into();
        app.coins = 0;
        on_player_death(&mut app, 100.0, 100.0, "Tester was slain...");
        assert_eq!(app.lost_coins, 0);
        assert_eq!(app.lost_coin_string, "");
        assert_eq!(app.death_text, "Tester was slain...");
    }

    /// S1-P-i51 — `TryDroppingSingleItem`'s RNG witness (`Player.cs:54373`):
    /// exactly TWO integer draws per death drop — `Next(-20, 21)` then
    /// `Next(-20, 1)`, each `* 0.2f` — on the `UpdatePlayers` stream, in
    /// slot order; the explicit velocity skips `Item.NewItem`'s scatter
    /// (`Item.cs:49738-49741`) and the carried prefix rolls nothing.
    #[test]
    fn the_death_drops_draw_two_integers_per_stack_in_slot_order() {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory[3] = crate::item::ItemSlot {
            kind: 1522, name: crate::registry::name(1522), count: 2, prefix: 0,
        };
        app.inventory[7] = crate::item::ItemSlot {
            kind: 1524, name: crate::registry::name(1524), count: 1, prefix: 0,
        };
        app.coins = 0;
        *app.rngs.stream(RngKey::UpdatePlayers) = UnifiedRandom::new(777);
        // The twin: two gem drops (slot order 3 then 7), two draws each,
        // then the tombstone roll (`roll_tombstone`'s own draws).
        let mut twin = UnifiedRandom::new(777);
        let vx0 = twin.next_range(-20, 21) as f32 * 0.2;
        let vy0 = twin.next_range(-20, 1) as f32 * 0.2;
        let vx1 = twin.next_range(-20, 21) as f32 * 0.2;
        let vy1 = twin.next_range(-20, 1) as f32 * 0.2;
        on_player_death(&mut app, 100.0, 200.0, "Tester was slain...");
        // The two gem drops carry the twin velocities, in slot order.
        let gems: Vec<&crate::item::ItemDrop> = app
            .items
            .iter()
            .filter(|d| d.kind == 1522 || d.kind == 1524)
            .collect();
        assert_eq!(gems.len(), 2);
        assert_eq!(gems[0].kind, 1522, "slot 3 drops first");
        assert_eq!(gems[0].vel.x, vx0);
        assert_eq!(gems[0].vel.y, vy0);
        assert_eq!(gems[1].kind, 1524);
        assert_eq!(gems[1].vel.x, vx1);
        assert_eq!(gems[1].vel.y, vy1);
        // The centre: `(int)position + Width/2, Height/2` (`Item.cs:49651`),
        // the box top-left at centre - SIZE/2.
        let cx = 100.0f32 as i32 as f32 + crate::player::WIDTH * 0.5;
        let cy = 200.0f32 as i32 as f32 + crate::player::HEIGHT * 0.5;
        assert_eq!(gems[0].pos.x, cx - crate::item::SIZE * 0.5);
        assert_eq!(gems[0].pos.y, cy - crate::item::SIZE * 0.5);
        // The stream: the four gem draws then the tombstone roll's draws
        // (`roll_tombstone` — the |vx|>=2 retry loop, Next(6), the rich
        // gate's Next(5) when coinsOwned > 100000; here 0 coins).
        let mut expect = UnifiedRandom::new(777);
        for _ in 0..2 {
            expect.next_range(-20, 21);
            expect.next_range(-20, 1);
        }
        let mut tvx = expect.next_range(-35, 36) as f32 * 0.1;
        while tvx >= -2.0 && tvx <= 2.0 {
            tvx += expect.next_range(-30, 31) as f32 * 0.1;
        }
        expect.next_max(6);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            expect,
            "two draws per gem drop, then the tombstone roll"
        );
    }

    /// S1-P-i51 — mediumcore (`difficulty == 1`) drops the WHOLE carry
    /// (`DropItems(gemsOnly: false)`, `Player.cs:54319-54361`): the starter
    /// tools vanish without dropping, everything else drops, the equipment
    /// arrays follow, and the starter tools return into slots 0/1/2. The
    /// purse stays whole (`KillMe` `:38925`'s difficulty gate).
    #[test]
    fn mediumcore_death_drops_the_whole_carry_and_regrants_the_tools() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut draft = crate::ui::char_creation::NewPlayerDraft::new();
        draft.difficulty = 1; // mediumcore
        app.session_draft = Some(draft);
        // The carry: a starter tool (airs, no drop), a gem, a block stack,
        // and an armor piece + a dye.
        app.inventory[0] = crate::item::ItemSlot {
            kind: 3507, name: crate::registry::name(3507), count: 1, prefix: 0,
        };
        app.inventory[5] = crate::item::ItemSlot {
            kind: 1522, name: crate::registry::name(1522), count: 3, prefix: 0,
        };
        app.inventory[9] = crate::item::ItemSlot {
            kind: 2, name: crate::registry::name(2), count: 50, prefix: 0,
        };
        app.equip.armor[0] = crate::item::ItemSlot {
            kind: 92, name: crate::registry::name(92), count: 1, prefix: 0,
        };
        app.equip.dye[0] = crate::item::ItemSlot {
            kind: 1013, name: crate::registry::name(1013), count: 1, prefix: 0,
        };
        // The 59-slot walk also covers the coin/ammo rows and the cursor
        // mirror (`Player.cs:1576`): a silver row stack, an arrow row stack
        // and a cursor stone stack all drop.
        app.equip.coins[0] = crate::item::ItemSlot {
            kind: 72, name: crate::registry::name(72), count: 30, prefix: 0,
        };
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 40, name: crate::registry::name(40), count: 25, prefix: 0,
        };
        app.mouse_item = crate::item::ItemSlot {
            kind: 3, name: crate::registry::name(3), count: 7, prefix: 0,
        };
        app.coins = 500;
        // The starter re-grant twin: `Prefix(-1)` rolls on the SAME stream
        // after the drop draws, in slot order 3507 → 3509 → 3506.
        let mut expect = app.rngs.stream(RngKey::UpdatePlayers).clone();
        // The drops: gem(3), block(9), silver row, arrow row, cursor stone,
        // armor(0), dye(0) — 7 stacks × two draws.
        for _ in 0..7 {
            expect.next_range(-20, 21);
            expect.next_range(-20, 1);
        }
        let p0 = crate::prefix::roll_prefix(3507, -1, &mut expect);
        let p1 = crate::prefix::roll_prefix(3509, -1, &mut expect);
        let p2 = crate::prefix::roll_prefix(3506, -1, &mut expect);
        // Then the tombstone roll (500 copper — no rich gate).
        let mut tvx = expect.next_range(-35, 36) as f32 * 0.1;
        while tvx >= -2.0 && tvx <= 2.0 {
            tvx += expect.next_range(-30, 31) as f32 * 0.1;
        }
        expect.next_max(6);
        on_player_death(&mut app, 100.0, 100.0, "Tester was slain...");
        // The drops: gem + block + armor + dye + silver + arrows + cursor
        // stone (the starter tool did NOT drop), no PURSE coin drop.
        let kinds: Vec<u16> = app.items.iter().map(|d| d.kind).collect();
        assert!(kinds.contains(&1522), "the gem dropped");
        assert!(kinds.contains(&2), "the block stack dropped");
        assert!(kinds.contains(&92), "the armor dropped");
        assert!(kinds.contains(&1013), "the dye dropped");
        assert!(kinds.contains(&72), "the coin ROW stack dropped (:1576's 59-slot walk)");
        assert!(kinds.contains(&40), "the ammo row stack dropped");
        assert!(kinds.contains(&3), "the cursor stack dropped");
        assert!(!kinds.contains(&3507), "the starter tool vanished, not dropped");
        // The purse stays whole; lostCoins books the whole purse for the
        // death screen (`:38849-38853`).
        assert_eq!(app.coins, 500, "mediumcore keeps the purse");
        assert_eq!(app.lost_coins, 500);
        // The starter re-grant: shortsword/pickaxe/axe into 0/1/2 with the
        // NATURAL `Prefix(-1)` rolls (`:54356-54361` — the tools sit in the
        // swords/axes/picks set, they DO roll).
        assert_eq!(app.inventory[0].kind, 3507);
        assert_eq!(app.inventory[1].kind, 3509);
        assert_eq!(app.inventory[2].kind, 3506);
        assert_eq!(app.inventory[0].prefix, p0, "3507's natural roll");
        assert_eq!(app.inventory[1].prefix, p1, "3509's natural roll");
        assert_eq!(app.inventory[2].prefix, p2, "3506's natural roll");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            expect,
            "the drop draws, the three Prefix(-1) rolls, then the tombstone"
        );
        // The dropped slots aired out (rows + cursor included).
        assert_eq!(app.inventory[5].count, 0);
        assert_eq!(app.inventory[9].count, 0);
        assert_eq!(app.equip.armor[0].count, 0);
        assert_eq!(app.equip.dye[0].count, 0);
        assert_eq!(app.equip.coins[0].count, 0);
        assert_eq!(app.equip.ammo[0].count, 0);
        assert_eq!(app.mouse_item.count, 0, "the cursor aired (the walk dropped it)");
    }

    /// S1-P-i51 — softcore (`difficulty == 0`) drops ONLY the gems
    /// (`DropItems(gemsOnly: true)`, `Player.cs:54309-54316`): a non-gem
    /// stack stays in the inventory, the equipment stays equipped.
    #[test]
    fn softcore_death_drops_only_the_gems() {
        let mut app = flow_app(None, vec![], vec![]);
        // difficulty defaults to 0 (softcore) — no session draft needed.
        app.inventory[5] = crate::item::ItemSlot {
            kind: 1522, name: crate::registry::name(1522), count: 3, prefix: 0,
        };
        app.inventory[9] = crate::item::ItemSlot {
            kind: 2, name: crate::registry::name(2), count: 50, prefix: 0,
        };
        app.equip.armor[0] = crate::item::ItemSlot {
            kind: 92, name: crate::registry::name(92), count: 1, prefix: 0,
        };
        app.coins = 0;
        on_player_death(&mut app, 100.0, 100.0, "Tester was slain...");
        let kinds: Vec<u16> = app.items.iter().map(|d| d.kind).collect();
        assert!(kinds.contains(&1522), "the gem dropped");
        assert!(!kinds.contains(&2), "the block stack stayed");
        assert!(!kinds.contains(&92), "the armor stayed equipped");
        assert_eq!(app.inventory[9].count, 50);
        assert_eq!(app.equip.armor[0].count, 1);
        // No starter re-grant on the gems-only path.
        assert_eq!(app.inventory[0].count, 0);
    }

    /// S1-P-i51 — the softcore cursor return (`KillMe` `:38937-38944`): a
    /// NON-gem cursor stack survives the gems-only walk and returns through
    /// `GetItem(ReturnItemFromSlot)` — the plain `GetItem`, NOT
    /// `GetOrDropItem`: it fits the inventory, the cursor airs, the fill
    /// plays the Grab chime (ONE `Next(-10, 11)` pitch draw) and NO world
    /// spawn runs. A GEM on the cursor drops in the walk itself (the 59-slot
    /// array's `inventory[58]`).
    #[test]
    fn softcore_death_returns_the_non_gem_cursor_and_drops_a_gem_cursor() {
        let mut app = flow_app(None, vec![], vec![]);
        app.mouse_item = crate::item::ItemSlot {
            kind: 3, name: crate::registry::name(3), count: 9, prefix: 0,
        };
        app.coins = 0;
        let mut expect = app.rngs.stream(RngKey::UpdatePlayers).clone();
        let _ = expect.next_range(-10, 11); // the return fill's Grab chime
        // Then the tombstone roll (0 coins).
        let mut tvx = expect.next_range(-35, 36) as f32 * 0.1;
        while tvx >= -2.0 && tvx <= 2.0 {
            tvx += expect.next_range(-30, 31) as f32 * 0.1;
        }
        expect.next_max(6);
        on_player_death(&mut app, 100.0, 100.0, "Tester was slain...");
        assert!(app.items.is_empty(), "the cursor RETURNED, no world drop");
        assert_eq!(app.mouse_item.count, 0, "the cursor aired out");
        assert!(app.inventory.iter().any(|s| s.kind == 3 && s.count == 9));
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            expect,
            "the Grab chime then the tombstone roll — no spawn scatter"
        );
        // The gem cursor: the gems-only walk drops it (two draws), the
        // return then sees air.
        let mut app = flow_app(None, vec![], vec![]);
        app.mouse_item = crate::item::ItemSlot {
            kind: 1522, name: crate::registry::name(1522), count: 2, prefix: 0,
        };
        app.coins = 0;
        on_player_death(&mut app, 100.0, 100.0, "Tester was slain...");
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 1522, "the gem cursor dropped in the walk");
        assert_eq!(app.items[0].count, 2);
        assert_eq!(app.mouse_item.count, 0);
        assert!(!app.inventory.iter().any(|s| s.kind == 1522));
    }
}

}
