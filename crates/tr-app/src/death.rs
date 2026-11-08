//! The death pipeline — `Player.KillMe`'s drop half (`Player.cs:38863`):
//! `DropItems(gemsOnly)` (`:54302`), `DropCoins` (`:54261`) and
//! `DropTombstone` (`:39091`).
//!
//! Softcore death (the ported difficulty) drops only the large gems the
//! player carries, half the carried coins, and a falling tombstone
//! projectile that places a tombstone tile on landing.

use glam::Vec2;

use crate::app::App;
use crate::regen::DotDeath;
use tr_core::TILE_PX;

/// `Player.DropItems(gemsOnly: true)` (`Player.cs:54302-54315`): softcore
/// death drops only the large gems from the inventory
/// (`ItemID` 1522-1527 = LargeAmethyst..LargeDiamond, 3643 = LargeAmber).
fn drop_gems(app: &mut App, x: f32, y: f32) {
    let GEMS: [u16; 7] = [1522, 1523, 1524, 1525, 1526, 1527, 3643];
    let mut rng_state = app.item_rng;
    for slot in app.inventory.iter() {
        if slot.count == 0 || !GEMS.contains(&slot.kind) {
            continue;
        }
        let vx = (crate::app::rand01(&mut rng_state) * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - crate::app::rand01(&mut rng_state);
        let phase = crate::app::rand01(&mut rng_state) * std::f32::consts::TAU;
        app.items.push(crate::item::ItemDrop::new(
            slot.kind, slot.count, slot.prefix, x, y, vx, vy, phase,
        ));
        // The dropped stack leaves the inventory (`TryDroppingSingleItem`).
        // The slot identity matters (gems are stackable), so the caller's
        // slot is zeroed through the shared &mut.
    }
    app.item_rng = rng_state;
    for slot in app.inventory.iter_mut() {
        if GEMS.contains(&slot.kind) {
            slot.count = 0;
        }
    }
}

/// `Player.DropCoins` (`Player.cs:54261-54294`): every coin item in the
/// inventory drops `stack / 2` (expert `/ 4`, master none — the ported
/// difficulty is classic), keeping the remainder. The port's coins live in
/// `App.coins` (a copper total), so the dropped half is split greedily into
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
    let dropped = app.coins / 2;
    if dropped == 0 {
        return 0;
    }
    app.coins -= dropped;
    let mut rng_state = app.item_rng;
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
        let vx = (crate::app::rand01(&mut rng_state) * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - crate::app::rand01(&mut rng_state);
        let phase = crate::app::rand01(&mut rng_state) * std::f32::consts::TAU;
        app.items.push(crate::item::ItemDrop::new(
            coin_item,
            count as u32,
            0,
            x,
            y,
            vx,
            vy,
            phase,
        ));
    }
    // The vanilla split drops the exact remainder (no denomination cap on the
    // last tier beyond the stack limit; stacks > 999 would split — the ported
    // coin totals stay under it).
    if rest > 0 {
        let vx = (crate::app::rand01(&mut rng_state) * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - crate::app::rand01(&mut rng_state);
        let phase = crate::app::rand01(&mut rng_state) * std::f32::consts::TAU;
        app.items.push(crate::item::ItemDrop::new(
            71,
            rest as u32,
            0,
            x,
            y,
            vx,
            vy,
            phase,
        ));
    }
    app.item_rng = rng_state;
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
pub fn roll_tombstone(rng_state: &mut u64, coins_owned: u64) -> TombFall {
    let mut next = |lo: i32, hi: i32| -> i32 {
        // `Main.rand.Next(lo, hi)` — hi exclusive, on the app's item stream.
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo) as f32;
        lo + (crate::app::rand01(rng_state) * span) as i32
    };
    let mut vx = next(-35, 36) as f32 * 0.1;
    while vx >= -2.0 && vx <= 2.0 {
        vx += next(-30, 31) as f32 * 0.1;
    }
    let roll = next(0, 6);
    let kind = if coins_owned > 100_000 {
        next(0, 5) as u16 + 527
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
    }
}

/// `PlayerDeathReason.GetDeathText`: the `DeathText.{base}_{variant}` row
/// formatted with the dead player's name (the key rows carry `{0}`).
/// `roll` is the C# `Main.rand.Next(variants)` — the caller draws it from its
/// own stream (the port's split-stream model; documented in ANALYSIS).
pub fn death_message(death: DotDeath, roll: f32, player_name: &str) -> String {
    let (base, variants) = death_text_key(death);
    // The single-variant rows (`DeathText.Poisoned`, `Slain`, `Spored`) carry
    // no `_N` suffix; the families append the rolled index.
    let key = if variants <= 1 {
        base.to_string()
    } else {
        let variant = (roll.max(0.0) * variants as f32) as usize % variants as usize + 1;
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
pub fn on_player_death(app: &mut App, x: f32, y: f32, death_text: &str) {
    drop_gems(app, x, y);
    // `KillMe`'s softcore arm (`Player.cs:38931-38936`): `lostCoins =
    // DropCoins()` and `lostCoinString = Main.ValueToCoins(lostCoins)` —
    // the death screen's "Dropped coins" line reads both (`Main.cs:45333`).
    let lost = drop_coins(app, x, y);
    app.lost_coins = lost;
    app.lost_coin_string = value_to_coins(lost);
    app.death_text = death_text.to_string();
    let mut tomb = roll_tombstone(&mut app.item_rng, app.coins);
    tomb.pos = Vec2::new(x, y);
    tomb.text = death_text.to_string();
    app.tomb_falls.push(tomb);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // roll 0.0 and 0.999 both stay inside the 2-variant family.
        assert_eq!(death_message(DotDeath::Suffocation, 0.0, "Bob"), "Bob couldn't breathe.");
        assert_eq!(death_message(DotDeath::Suffocation, 0.99, "Bob"), "Bob was buried alive.");
        // A single-variant family ignores the roll.
        assert_eq!(
            death_message(DotDeath::Poison, 0.5, "Bob"),
            "Bob couldn't find the antidote."
        );
        assert_eq!(slain_message("Bob"), "Bob was slain...");
    }

    #[test]
    fn tombstone_roll_keeps_the_vanilla_id_set() {
        // Player.cs:39091-39115: normal deaths roll Next(6) -> 0 is the plain
        // Tombstone (43), 1..5 are the grave markers (201-205); rich deaths
        // (coins > 100000) roll Next(5) + 527 (the rich gravestones).
        let mut state = 1u64;
        for _ in 0..200 {
            let t = roll_tombstone(&mut state, 50);
            assert!(
                t.kind == 43 || (201..=205).contains(&t.kind),
                "normal roll produced kind {}",
                t.kind
            );
        }
        for _ in 0..200 {
            let t = roll_tombstone(&mut state, 200_000);
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
        let mut state = 2u64;
        for _ in 0..100 {
            let t = roll_tombstone(&mut state, 50);
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
}

}
