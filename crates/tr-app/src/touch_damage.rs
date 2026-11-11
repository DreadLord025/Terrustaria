//! Contact damage from tiles — `Collision.HurtTiles` (`Collision.cs:3115-3204`),
//! `Collision.CanTileHurt` (`:3206-3225`), `Player.GetHurtTile`
//! (`Player.cs:28954-28979`) and `Player.ApplyTouchDamage` (`:28977-29019`).
//!
//! This is the hazard half of the dungeon's spikes: `worldgen::dungeon_features`
//! now paints `TileID.Spike` runs on the floors and ceilings, and this module is
//! what makes standing on one hurt.
//!
//! Ported: the `TileID.Sets.TouchDamageImmediate` table (the nine damaging
//! tiles — spikes, spike beds, thorn bushes, hellstone-adjacent sets), the
//! box scan with its half-brick and slope exemptions, the `Hurt` call with the
//! rolled damage, and the hot/bleeding/suffocate follow-ups' *tables* (their
//! buff applications need the buff system's per-tile wiring — see the
//! deviations below).
//!
//! Deviations: `Player.TouchedTiles` (the movement-traced fallback in
//! `GetHurtTile`) is not modelled — the port's hurt-tile scan covers the same
//! box with the source's own ±1 margin; `TileID.Sets.TouchDamageHot`,
//! `TouchDamageBleeding`, `Suffocate` and `TouchDamageDestroyTile` have no
//! ported members yet, so only the immediate-damage arm fires;
//! `ImmunityCooldownID.TileContactDamage` collapses into the port's single
//! `hurt_cd` (the port carries one immunity window, not the C#'s per-source
//! array).
//!
//! Out: the dungeon traps (`DungeonGlobalTraps`), lava/meteorite touch damage
//! (the same table holds them; they need the liquid pass's tick), and the
//! `hurtDirection` bounce the C#'s `HurtTiles` feeds into `Hurt`.

use tr_core::tile::Tile;
use tr_core::world::World;

/// `TileID.Sets.TouchDamageImmediate` (`TileID.cs:609`):
/// `CreateIntSet(0, 32, 10, 69, 17, 80, 6, 352, 10, 655, 100, 48, 60, 232, 80,
/// 484, 25, 750, 70)` — the pairs are `tile, damage`.
pub const TOUCH_DAMAGE_IMMEDIATE: &[(u16, i32)] = &[
    (32, 10),
    (69, 17),
    (80, 6),
    (352, 10),
    (655, 100),
    (48, 60),
    (232, 80),
    (484, 25),
    (750, 70),
];

/// `TileID.Spike` — the wooden dungeon spike the dungeon pass paints.
pub const SPIKE: u16 = 48;
/// `TileID.WoodenSpike` — the blue dungeon's spike.
pub const WOODEN_SPIKE: u16 = 232;

/// `TileID.Sets.TouchDamageImmediate[type]`, 0 for everything else.
pub fn touch_damage_immediate(kind: u16) -> i32 {
    TOUCH_DAMAGE_IMMEDIATE
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or(0, |(_, d)| *d)
}

/// The tile a box overlap landed on (`Collision.HurtTile`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HurtTile {
    pub kind: u16,
    pub x: i32,
    pub y: i32,
}

/// `Collision.CanTileHurt` (`Collision.cs:3206-3225`): `Tile 230` only on a
/// good world, `Tile 80` (the ash-grass?) only on a Don't Starve world, then
/// the three set tables. Only the immediate-damage table has ported members.
pub fn can_tile_hurt(kind: u16, good_world: bool, dont_starve: bool) -> bool {
    if kind == 230 && !good_world {
        return false;
    }
    if kind == 80 && !dont_starve {
        return false;
    }
    touch_damage_immediate(kind) > 0
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
/// tiles whose open corner the box does not actually reach (`num7 == 2`).
#[allow(clippy::too_many_arguments)]
pub fn hurt_tiles(
    world: &World,
    pos_x: f32,
    pos_y: f32,
    width: i32,
    height: i32,
    good_world: bool,
    dont_starve: bool,
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
            if pos_x + (width as f32) < tile_x
                || pos_x > tile_x + 16.0
                || pos_y + (height as f32) < ty - 0.5
                || pos_y > ty + th + 0.5
                || !can_tile_hurt(t.kind, good_world, dont_starve)
            {
                continue;
            }
            if t.slope() > 0 {
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

/// `Player.GetHurtTile` + `Player.ApplyTouchDamage` (`Player.cs:28954-29019`)
/// for the live player: find the damaging tile under the box and land its
/// rolled damage. `hitDirection` is 0 (the C# passes 0 from
/// `ApplyTouchDamage`), so there is no knockback — spikes in the vanilla knock
/// the player back through the `HurtTiles` bounce, which the port's hurt path
/// does not carry yet.
///
/// Returns `true` when a hit landed.
pub fn apply_touch_damage(
    world: &World,
    player: &mut crate::player::Player,
    rng: &mut u64,
    width: i32,
    height: i32,
) -> bool {
    let good_world = world
        .seeds
        .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let dont_starve = world
        .seeds
        .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    let Some(hit) = hurt_tiles(world, player.x, player.y, width, height, good_world, dont_starve)
    else {
        return false;
    };
    let raw = touch_damage_immediate(hit.kind) as f32;
    if raw <= 0.0 {
        return false;
    }
    // `Main.DamageVar(num, 0f - this.luck)` — damage taken negates the luck
    // (`Player.cs:28982`).
    let mut loot = crate::world_sim::npcs::LootRng(rng);
    let dmg = tr_core::damage::damage_var(&mut loot, raw, -player.luck());
    let cx = player.x + width as f32 * 0.5;
    player.hurt(dmg, cx)
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
        assert_eq!(TOUCH_DAMAGE_IMMEDIATE.len(), 9);
    }

    /// `CanTileHurt`'s two world gates (`Collision.cs:3208-3215`).
    #[test]
    fn the_world_gates_close_two_tiles() {
        assert!(!can_tile_hurt(230, false, false), "tile 230 needs a good world");
        // Its *positive* arm rides `TileID.Sets.TouchDamageBleeding`, which
        // has no ported members yet, so even a good world leaves it inert.
        assert!(!can_tile_hurt(230, true, false));
        assert!(!can_tile_hurt(80, false, false), "tile 80 needs Don't Starve");
        assert!(can_tile_hurt(80, false, true), "tile 80's immediate 6");
        assert!(can_tile_hurt(48, false, false), "spikes always hurt");
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
        let hit = hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0 - 30.0, 20, 42, false, false);
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
        assert_eq!(hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0, 20, 42, false, false), None);
        let mut w = world_with(SPIKE, 11, 30);
        let i = w.index(11, 30);
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTUATOR;
        // `inActive()` needs the actuator's own off-bit; the port's
        // `inactive()` reads that flag.
        w.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        assert_eq!(hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0, 20, 42, false, false), None);
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
        assert!(
            apply_touch_damage(&w, &mut p, &mut rng, 20, 42),
            "the spike lands"
        );
        let taken = before - p.hp;
        assert!(
            (51..=69).contains(&taken),
            "60 minus the ±15 % variance, got {taken}"
        );
        // The 40-tick hurt window (`iframes_for`) swallows the next tick.
        assert!(
            !apply_touch_damage(&w, &mut p, &mut rng, 20, 42),
            "one hit per immunity window"
        );
    }

    /// A half-brick spike only hurts from its lower half (`:3150-3154`).
    #[test]
    fn a_half_brick_spike_hurts_only_below_the_seam() {
        let mut w = world_with(SPIKE, 11, 30);
        let i = w.index(11, 30);
        w.tiles[i].flags |= tr_core::tile::FLAG_HALF_BRICK;
        // Feet at the tile's top edge: above the seam, no hit.
        assert_eq!(
            hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0 - 40.0, 20, 42, false, false),
            None
        );
        // Feet inside the lower half: the hit lands.
        assert!(
            hurt_tiles(&w, 11.0 * 16.0, 30.0 * 16.0 + 8.0, 20, 42, false, false).is_some()
        );
    }
}
