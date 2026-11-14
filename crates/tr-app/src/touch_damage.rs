//! Contact damage from tiles — `Collision.HurtTiles` (`Collision.cs:3115-3204`),
//! `Collision.CanTileHurt` (`:3206-3225`), `Player.GetHurtTile`
//! (`Player.cs:28954-28975`) and `Player.ApplyTouchDamage` (`:28977-29019`).
//!
//! This is the hazard half of the dungeon's spikes: `worldgen::dungeon_features`
//! now paints `TileID.Spike` runs on the floors and ceilings, and this module is
//! what makes standing on one hurt. The same pass carries the vanilla's other
//! tile-contact hazards: the hellstone's burn, the spikes' bleeding, the sands'
//! suffocation and the thorns that break under a touch.
//!
//! Ported: the four `TileID.Sets` tables the C# reads
//! (`TileID.cs:605-609` — `TouchDamageDestroyTile`, `Suffocate`,
//! `TouchDamageHot`, `TouchDamageBleeding` and the `TouchDamageImmediate`
//! value table, all generated into `tr-core/src/tiledata/sets.rs`), the box
//! scan with its half-brick, slope and suffocate-shrink exemptions, the `Hurt`
//! call with the rolled damage, the burning/bleeding/suffocation buffs and the
//! destroy-on-touch arm.
//!
//! Deviations: `Player.TouchedTiles` (the movement-traced fallback in
//! `GetHurtTile`) is not modelled — the port's hurt-tile scan covers the same
//! box with the source's own ±1 margin; `ImmunityCooldownID.TileContactDamage`
//! collapses into the port's single `hurt_cd` (the port carries one immunity
//! window, not the C#'s per-source array); the C#'s `player.fireWalk` (the
//! Obsidian Skull) has no ported accessor, so the hot arm always fires;
//! `Player.shimmering` (the C#'s gate around the whole pass) has no ported
//! player state, so the pass always runs.
//!
//! Out: the dungeon traps (`DungeonGlobalTraps`), lava/meteorite touch damage
//! (the same table holds them; they need the liquid pass's tick), and the
//! `hurtDirection` bounce the C#'s `HurtTiles` feeds into `Hurt`.

use tr_core::loot::DropRng;
use tr_core::tile::Tile;
use tr_core::tiledata::{
    SET_SUFFOCATE, SET_TOUCH_DAMAGE_BLEEDING, SET_TOUCH_DAMAGE_DESTROY_TILE,
    SET_TOUCH_DAMAGE_HOT, SET_TOUCH_DAMAGE_IMMEDIATE,
};
use tr_core::world::World;

/// `TileID.Spike` — the wooden dungeon spike the dungeon pass paints.
pub const SPIKE: u16 = 48;
/// `TileID.WoodenSpike` — the blue dungeon's spike.
pub const WOODEN_SPIKE: u16 = 232;

/// `BuffID.Burning` (`BuffID.cs:307`) — the hot-tile debuff
/// (`Player.ApplyTouchDamage`, `Player.cs:28987-28989`).
pub const BUFF_BURNING: i32 = 67;
/// `BuffID.Bleeding` (`BuffID.cs:231`) — the spike debuff
/// (`Player.cs:28991-28993`).
pub const BUFF_BLEEDING: i32 = 30;
/// `BuffID.Suffocation` (`BuffID.cs:305`) — the sand debuff
/// (`Player.cs:29004`).
pub const BUFF_SUFFOCATION: i32 = 68;

/// `TileID.Sets.TouchDamageImmediate[type]`, 0 for everything else
/// (`TileID.cs:609` — the generated value table in
/// `tr-core/src/tiledata/sets.rs`).
pub fn touch_damage_immediate(kind: u16) -> i32 {
    let i = kind as usize;
    if i < SET_TOUCH_DAMAGE_IMMEDIATE.len() {
        SET_TOUCH_DAMAGE_IMMEDIATE[i]
    } else {
        0
    }
}

/// The tile a box overlap landed on (`Collision.HurtTile`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HurtTile {
    pub kind: u16,
    pub x: i32,
    pub y: i32,
}

/// `Collision.CanTileHurt` (`Collision.cs:3206-3225`): `Tile 230` only on a
/// good world, `Tile 80` (the cactus) only on a Don't Starve world, then the
/// four set tables. `fire_walk` is the C#'s `player.fireWalk` — the hot arm
/// is skipped when the player walks on fire (`player == null || !player.fireWalk`,
/// `:3220`); the port has no Obsidian Skull accessor yet, so it is `false`.
pub fn can_tile_hurt(kind: u16, good_world: bool, dont_starve: bool, fire_walk: bool) -> bool {
    if kind == 230 && !good_world {
        return false;
    }
    if kind == 80 && !dont_starve {
        return false;
    }
    if SET_TOUCH_DAMAGE_BLEEDING.has(kind)
        || SET_SUFFOCATE.has(kind)
        || touch_damage_immediate(kind) > 0
    {
        return true;
    }
    if SET_TOUCH_DAMAGE_HOT.has(kind) && !fire_walk {
        return true;
    }
    false
}

/// `Tile.leftSlope()` (`Tile.cs`): slopes 1 and 3 rise to the left.
fn left_slope(t: &Tile) -> bool {
    matches!(t.slope(), 1 | 3)
}

/// `Tile.rightSlope()`: slopes 2 and 4 rise to the right.
fn right_slope(t: &Tile) -> bool {
    matches!(t.slope(), 2 | 4)
}

/// `Collision.HurtTiles` (`Collision.cs:3115-3204`): the ±1-margin box scan,
/// skipping actuated tiles, half-brick tiles' upper half, and the sloped
/// tiles whose open corner the box does not actually reach (`num7 == 2`). The
/// suffocating tiles shrink the box by two pixels on every side
/// (`num6 = 2`, `:3155-3159`) and are skipped outright when they are sloped
/// (`:3164-3169`).
pub fn hurt_tiles(
    world: &World,
    pos_x: f32,
    pos_y: f32,
    width: i32,
    height: i32,
    good_world: bool,
    dont_starve: bool,
    fire_walk: bool,
) -> Option<HurtTile> {
    let mut x0 = (pos_x / 16.0) as i32 - 1;
    let mut x1 = ((pos_x + width as f32) / 16.0) as i32 + 2;
    let mut y0 = (pos_y / 16.0) as i32 - 1;
    let mut y1 = ((pos_y + height as f32) / 16.0) as i32 + 2;
    x0 = x0.max(0);
    x1 = x1.min(world.width);
    y0 = y0.max(0);
    y1 = y1.min(world.height - 40);
    for i in x0..x1 {
        for j in y0..y1 {
            let t = &world.tiles[world.index(i, j)];
            if t.inactive() || !t.active() {
                continue;
            }
            let mut ty = j as f32 * 16.0;
            let mut th = 16.0f32;
            if t.half_brick() {
                ty += 8.0;
                th -= 8.0;
            }
            let tile_x = i as f32 * 16.0;
            // `:3155-3159`: the suffocating tiles pull the box in by two
            // pixels — the sand only suffocates when the player is properly
            // inside it.
            let num6 = if SET_SUFFOCATE.has(t.kind) { 2.0f32 } else { 0.0f32 };
            if pos_x + (width as f32) - num6 < tile_x
                || pos_x + num6 > tile_x + 16.0
                || pos_y + (height as f32) - num6 < ty - 0.5
                || pos_y + num6 > ty + th + 0.5
                || !can_tile_hurt(t.kind, good_world, dont_starve, fire_walk)
            {
                continue;
            }
            if t.slope() > 0 {
                if num6 > 0.0 {
                    continue; // `:3164-3169`
                }
                let mut num7 = 0;
                if right_slope(t) && pos_x > tile_x {
                    num7 += 1;
                }
                if left_slope(t) && pos_x + (width as f32) < tile_x + 16.0 {
                    num7 += 1;
                }
                if t.bottom_slope() && pos_y > ty {
                    num7 += 1;
                }
                if t.top_slope() && pos_y + (height as f32) < ty + th {
                    num7 += 1;
                }
                if num7 == 2 {
                    continue;
                }
            }
            return Some(HurtTile {
                kind: t.kind,
                x: i,
                y: j,
            });
        }
    }
    None
}

/// The outcome of one touch-damage pass (`Player.ApplyTouchDamage`,
/// `Player.cs:28977-29019`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchDamageOutcome {
    /// The hurt tile the scan found (`Player.GetHurtTile`).
    pub tile: Option<HurtTile>,
    /// The immediate damage landed (the C#'s `Hurt(...) != 0.0`).
    pub damaged: bool,
    /// The tile the destroy set killed — `(x, y)` plus the item it drops
    /// (`TileID.Sets.TouchDamageDestroyTile`, `Player.cs:29011-29018`).
    pub destroyed: Option<(i32, i32, Option<u16>)>,
}

/// `Player.GetHurtTile` + `Player.ApplyTouchDamage` (`Player.cs:28954-29019`)
/// for the live player: find the damaging tile under the box and land its
/// rolled damage, then the follow-up buffs and the destroy-on-touch arm.
/// `hitDirection` is 0 (the C# passes 0 from `ApplyTouchDamage`), so there is
/// no knockback — spikes in the vanilla knock the player back through the
/// `HurtTiles` bounce, which the port's hurt path does not carry yet.
///
/// The C#'s arms, in order (`Player.cs:28979-29018`): the immediate damage
/// (only when `TouchDamageImmediate[tileId] > 0`); the burning and bleeding
/// buffs when the hit did *not* land (the C#'s `flag` is set when `Hurt`
/// returned 0, i.e. the hit was swallowed); the suffocation counter; and the
/// destroy-on-touch kill.
pub fn apply_touch_damage(
    world: &mut World,
    player: &mut crate::player::Player,
    rng: &mut u64,
    width: i32,
    height: i32,
) -> TouchDamageOutcome {
    let good_world = world
        .seeds
        .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let dont_starve = world
        .seeds
        .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    // The C#'s `player.fireWalk` (the Obsidian Skull) has no ported accessor
    // yet — the hot arm always fires (`Player.cs:3220`).
    let fire_walk = false;
    let Some(hit) = hurt_tiles(
        world,
        player.x,
        player.y,
        width,
        height,
        good_world,
        dont_starve,
        fire_walk,
    ) else {
        return TouchDamageOutcome::default();
    };
    let mut outcome = TouchDamageOutcome {
        tile: Some(hit),
        ..Default::default()
    };
    let mut loot = crate::world_sim::npcs::LootRng(rng);
    // `:28980-28984`: the immediate damage, and the C#'s `flag` — set when
    // `Hurt` dealt nothing (the hit was swallowed by an immunity).
    let raw = touch_damage_immediate(hit.kind);
    if raw > 0 {
        // `Main.DamageVar(num, 0f - this.luck)` — damage taken negates the
        // luck (`Player.cs:28982`).
        let dmg = tr_core::damage::damage_var(&mut loot, raw as f32, -player.luck());
        let cx = player.x + width as f32 * 0.5;
        // The same `LootRng` stream feeds the hurt's dodge rolls; `dodgeable`
        // keeps `Hurt`'s default `true` — the hazard tiles pass no `dodgeable`
        // override in the C# (`Player.cs:28982-29019`'s call chain), so a black
        // belt eats spike damage (research S1-C-1 §1.4).
        outcome.damaged = player.hurt(dmg, cx, &mut loot, true);
    }
    let flag = raw > 0 && !outcome.damaged;
    if !flag {
        // `:28986-28994`: the hot tiles burn and the spikes bleed — but only
        // when the hit was not swallowed.
        if SET_TOUCH_DAMAGE_HOT.has(hit.kind) {
            player.buffs.add_buff(BUFF_BURNING, 20);
        }
        if SET_TOUCH_DAMAGE_BLEEDING.has(hit.kind) {
            let duration = loot.next_range(600, 1200);
            player.buffs.add_buff(BUFF_BLEEDING, duration);
        }
    }
    // `:28996-29010`: the suffocating tiles (the sands) count the ticks the
    // player stays in them and suffocate from the sixth tick on. The counter
    // only resets when another hurt tile is touched — the C#'s quirk, kept.
    if SET_SUFFOCATE.has(hit.kind) {
        if player.suffocate_delay < 5 {
            player.suffocate_delay += 1;
        } else {
            player.buffs.add_buff(BUFF_SUFFOCATION, 1);
        }
    } else {
        player.suffocate_delay = 0;
    }
    // `:29011-29018`: the thorns break under a touch (`WorldGen.KillTile`).
    if SET_TOUCH_DAMAGE_DESTROY_TILE.has(hit.kind) {
        let frame_y = world.tile(hit.x, hit.y).map_or(0, |t| t.frame_y);
        let dropped = crate::registry::tile_drop(hit.kind, frame_y);
        tr_core::edit::kill_tile(world, hit.x, hit.y, false);
        outcome.destroyed = Some((hit.x, hit.y, dropped));
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::world::World;

    fn world_with(kind: u16, x: i32, y: i32) -> World {
        let mut w = World::new("hurt".into(), 60, 200);
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    /// The table is the C#'s (`TileID.cs:609`), spikes included.
    #[test]
    fn the_touch_damage_table_matches_the_vanilla_set() {
        assert_eq!(touch_damage_immediate(48), 60, "wooden spikes hurt 60");
        assert_eq!(touch_damage_immediate(232), 80, "blue-dungeon spikes 80");
        assert_eq!(touch_damage_immediate(484), 25);
        assert_eq!(touch_damage_immediate(41), 0, "a dungeon brick is harmless");
        // `CreateIntSet(0, 32, 10, 69, 17, 80, 6, 352, 10, 655, 100, 48, 60,
        // 232, 80, 484, 25, 750, 70)` (`TileID.cs:609`) — the nine damaging
        // tiles.
        let damaging: Vec<u16> = (0..tr_core::tiledata::TILE_COUNT as u16)
            .filter(|k| touch_damage_immediate(*k) > 0)
            .collect();
        assert_eq!(damaging, vec![32, 48, 69, 80, 232, 352, 484, 655, 750]);
    }

    /// The four set tables the C# reads (`TileID.cs:605-609`) — the generated
    /// bitsets carry the vanilla members.
    #[test]
    fn the_four_vanilla_sets_carry_their_members() {
        // `Sets.TouchDamageDestroyTile = CreateBoolSet(32, 69, 352, 655)`.
        for kind in [32u16, 69, 352, 655] {
            assert!(SET_TOUCH_DAMAGE_DESTROY_TILE.has(kind), "tile {kind}");
        }
        assert!(!SET_TOUCH_DAMAGE_DESTROY_TILE.has(48), "the spikes stay");
        // `Sets.Suffocate = CreateBoolSet(53, 112, 116, 123, 224, 234)` — the
        // sands.
        for kind in [53u16, 112, 116, 123, 224, 234] {
            assert!(SET_SUFFOCATE.has(kind), "tile {kind}");
        }
        assert!(!SET_SUFFOCATE.has(41), "a dungeon brick does not suffocate");
        // `Sets.TouchDamageHot = CreateBoolSet(37, 58, 76, 684, 230)`.
        for kind in [37u16, 58, 76, 230, 684] {
            assert!(SET_TOUCH_DAMAGE_HOT.has(kind), "tile {kind}");
        }
        assert!(!SET_TOUCH_DAMAGE_HOT.has(41));
        // `Sets.TouchDamageBleeding = CreateBoolSet(48, 232, 750)`.
        for kind in [48u16, 232, 750] {
            assert!(SET_TOUCH_DAMAGE_BLEEDING.has(kind), "tile {kind}");
        }
        assert!(!SET_TOUCH_DAMAGE_BLEEDING.has(69), "the thorns do not bleed");
    }

    /// `CanTileHurt`'s two world gates (`Collision.cs:3208-3215`) and its four
    /// set arms (`:3216-3224`).
    #[test]
    fn the_world_gates_close_two_tiles() {
        assert!(!can_tile_hurt(230, false, false, false), "tile 230 needs a good world");
        assert!(
            can_tile_hurt(230, true, false, false),
            "the crispy honey block burns on a good world"
        );
        assert!(!can_tile_hurt(80, false, false, false), "tile 80 needs Don't Starve");
        assert!(can_tile_hurt(80, false, true, false), "tile 80's immediate 6");
        assert!(can_tile_hurt(48, false, false, false), "spikes always hurt");
    }

    /// The set arms of `CanTileHurt` (`:3216-3224`): the hot tiles need the
    /// fire walk to be harmless, the sands and the bleeding tiles always hurt.
    #[test]
    fn the_set_arms_of_can_tile_hurt_fire() {
        // Hellstone 58 is hot but carries no immediate damage.
        assert!(can_tile_hurt(58, false, false, false), "the hellstone burns");
        assert!(
            !can_tile_hurt(58, false, false, true),
            "the Obsidian Skull walks through it"
        );
        // Sand 53 suffocates.
        assert!(can_tile_hurt(53, false, false, false));
        // The bleeding spikes hurt through the bleeding set.
        assert!(can_tile_hurt(750, false, false, false), "the damaging spike");
        assert_eq!(touch_damage_immediate(750), 70);
        // A plain brick never hurts.
        assert!(!can_tile_hurt(41, false, false, false));
    }

    /// The suffocate shrink (`Collision.cs:3155-3163`): a box that only
    /// clips the sand's edge does not report it.
    #[test]
    fn the_sand_shrinks_the_box_by_two_pixels() {
        let w = world_with(53, 11, 30);
        let tile_x = 11.0 * 16.0;
        // The player's box ends exactly on the sand's left edge: the shrunk
        // box (`right - 2 < tile_x`) does not reach it.
        assert_eq!(
            hurt_tiles(&w, tile_x - 20.0, 30.0 * 16.0, 20, 42, false, false, false),
            None
        );
        // The shrink is exactly two pixels: one pixel deeper still misses.
        assert_eq!(
            hurt_tiles(
                &w,
                tile_x - 19.0,
                30.0 * 16.0,
                20,
                42,
                false,
                false,
                false
            ),
            None
        );
        // Two pixels deeper and the sand reports.
        assert!(
            hurt_tiles(
                &w,
                tile_x - 18.0,
                30.0 * 16.0,
                20,
                42,
                false,
                false,
                false
            )
            .is_some()
        );
    }

    /// A player box touching a spike tile reports it (`HurtTiles`, `:3115`).
    #[test]
    fn the_player_box_finds_the_spike_it_stands_on() {
        // A 3x3 spike block at (10..13, 30..33); the player's feet at y=30*16.
        let mut w = World::new("hurt".into(), 60, 200);
        for x in 10..13 {
            for y in 30..33 {
                let i = w.index(x, y);
                w.tiles[i].kind = SPIKE;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The player's 20x42 box standing with its feet inside the top row.
        let hit = hurt_tiles(
            &w,
            11.0 * 16.0,
            30.0 * 16.0 - 30.0,
            20,
            42,
            false,
            false,
            false,
        );
        // The ±1 margin means the leftmost touched column is reported first —
        // the C# scan order (`Collision.cs:3138-3141`).
        assert_eq!(
            hit,
            Some(HurtTile {
                kind: SPIKE,
                x: 10,
                y: 30
            })
        );
    }

    /// An ordinary brick never reports, and neither does an actuated spike
    /// (`!tile.inActive()`, `:3143`).
    #[test]
    fn bricks_and_actuated_spikes_do_not_hurt() {
        let w = world_with(41, 11, 30);
        assert_eq!(
            hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0, 20, 42, false, false, false),
            None
        );
        let mut w = world_with(SPIKE, 11, 30);
        let i = w.index(11, 30);
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTUATOR;
        // `inActive()` needs the actuator's own off-bit; the port's
        // `inactive()` reads that flag.
        w.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        assert_eq!(
            hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0, 20, 42, false, false, false),
            None
        );
    }

    /// `Player.ApplyTouchDamage`: standing on a spike lands its rolled 60
    /// (within the `DamageVar` ±15 %), and the hurt window blocks the repeat
    /// on the next tick.
    #[test]
    fn a_spike_lands_one_rolled_hit() {
        let mut w = World::new("hurt".into(), 60, 200);
        for x in 10..13 {
            for y in 30..33 {
                let i = w.index(x, y);
                w.tiles[i].kind = SPIKE;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut p = crate::player::Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        let before = p.hp;
        let mut rng = 7u64;
        let outcome = apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
        assert!(outcome.damaged, "the spike lands");
        let taken = before - p.hp;
        assert!(
            (51..=69).contains(&taken),
            "60 minus the ±15 % variance, got {taken}"
        );
        // The 40-tick hurt window (`iframes_for`) swallows the next tick.
        let again = apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
        assert!(!again.damaged, "one hit per immunity window");
    }

    /// The bleeding arm (`Player.cs:28991-28993`): the spikes bleed for
    /// 600-1200 ticks when the hit lands.
    #[test]
    fn a_spike_bleeds_the_player() {
        let mut w = world_with(SPIKE, 11, 30);
        let mut p = crate::player::Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        let mut rng = 7u64;
        apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
        let bleed = p.buffs.find_buff_index(BUFF_BLEEDING);
        assert!(bleed.is_some(), "the spikes bleed");
        let time = p.buffs.buff_time[bleed.unwrap()];
        assert!((600..1200).contains(&time), "600..1200 ticks, got {time}");
    }

    /// The burning arm (`Player.cs:28987-28989`): the hellstone burns the
    /// player for 20 ticks — it carries no immediate damage, so the arm fires
    /// on the very first tick.
    #[test]
    fn a_hellstone_burns_the_player() {
        let mut w = world_with(58, 11, 30);
        let mut p = crate::player::Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        let mut rng = 7u64;
        let outcome = apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
        assert!(!outcome.damaged, "the hellstone deals no immediate damage");
        let burn = p.buffs.find_buff_index(BUFF_BURNING);
        assert!(burn.is_some(), "the hellstone burns");
        assert_eq!(p.buffs.buff_time[burn.unwrap()], 20);
    }

    /// The suffocation arm (`Player.cs:28996-29010`): the fifth tick in the
    /// sand still counts, the sixth suffocates.
    #[test]
    fn the_sand_suffocates_after_five_ticks() {
        let mut w = world_with(53, 11, 30);
        let mut p = crate::player::Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0;
        let mut rng = 7u64;
        for _ in 0..5 {
            apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
            assert!(
                p.buffs.find_buff_index(BUFF_SUFFOCATION).is_none(),
                "the sand only counts up to five ticks"
            );
        }
        assert_eq!(p.suffocate_delay, 5);
        apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
        assert!(
            p.buffs.find_buff_index(BUFF_SUFFOCATION).is_some(),
            "the sixth tick suffocates"
        );
    }

    /// The destroy-on-touch arm (`Player.cs:29011-29018`): the thorns break
    /// under a touch, and they drop nothing.
    #[test]
    fn a_touch_breaks_the_thorns() {
        let mut w = world_with(69, 11, 30);
        let mut p = crate::player::Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        let mut rng = 7u64;
        let outcome = apply_touch_damage(&mut w, &mut p, &mut rng, 20, 42);
        assert_eq!(outcome.destroyed, Some((11, 30, None)), "the thorn breaks");
        assert!(
            !w.tile(11, 30).is_some_and(|t| t.active()),
            "the thorn tile is gone"
        );
    }
}
