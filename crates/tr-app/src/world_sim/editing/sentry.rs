//! The sentry-summon USE machinery (S1-P-i26): the two `ItemCheck_Shoot`
//! sentry arms and their placement/gate helpers, moved verbatim out of the
//! generic `Item.shoot` firing.
//!
//! C# census (`Player.cs`, 1.4.5.8):
//! * arm A — the placeable staff sentries (`:48189-48212`): items 1572
//!   (Frost Hydra → 308), 2366 (Queen Spider → 377), 3571 (Rainbow Crystal →
//!   643), 3569 (Moon Lord Turret → 641), 5119 (Houndius Shootius → 966),
//!   5463 (Dead Cells Barnacle → 1025). `PickSentryLandingSpot` (`:49525-
//!   49543`) picks the cursor/tile spot, the spawn drops at `(0, 15)` (the
//!   barnacle at `(0, 0)` on the raw mouse Y), `ai0` = 60/90 for the
//!   1572/5119, `originalDamage = sItem.damage` (the UNSCALED item damage,
//!   `:47060`/`:48209`), then `UpdateMaxTurrets`.
//! * arm B — the DD2 tower poppers (`:49057-49094`): items 3818-3820
//!   (Flameburst → 663/665/667), 3824-3826 (Ballista → 677/678/679),
//!   3829-3831 (Lightning Aura → 688/689/690), 3832-3834 (Explosive Trap →
//!   691/692/693). `PayDD2CrystalsBeforeUse` (`:42367-42374`) consumes 10
//!   Eternia crystals (3822) in reverse slot order, `FindSentryRestingSpot`
//!   (`:53015-53059`) picks the ground spot, the per-tier `ai0/ai1/localAI[0]`
//!   payload rides the spawn, `originalDamage = sItem.damage`, then
//!   `UpdateMaxTurrets`.
//! * the use gates (`ItemCheck_CheckCanUse_Inner`, `:52079-52111`): the
//!   crystal-payment lock, the `downedDD2EventAnyDifficulty || Ongoing`
//!   gate, the `WouldSpotOverlapWithSentry` overlap refusal (`:52983-53013`)
//!   and the solid-tile refusal.
//!
//! RNG: NONE of these helpers draws `Main.rand` — the placement scans, the
//! overlap test, the crystal payment and the spawn payload are pure
//! arithmetic. The C# sentry spawn makes no crit draw either (`ItemCheck_
//! Shoot` has none; `Projectile.Damage`'s crit block `Projectile.cs:12890-
//! 12901` gates on the melee/ranged/magic flags, which no sentry row sets),
//! so the port's sentry arms skip the generic throw-time crit roll.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::World;

use crate::item::ItemSlot;
use crate::projectile::ProjectilePool;

/// `WorldGen.SolidTile2(Tile)` (`WorldGen.cs:70134-70151`, the indexed
/// overload `:70492-70504`): active, solid, slopeless (a platform with a top
/// slope passes the first disjunct but the port's platforms never carry one),
/// non-halfBrick, not actuated. Unlike `SolidTile` the solid-top tiles PASS.
/// An out-of-world cell is refused (the C# try/catch → false); the in-world
/// null case does not exist in the port's dense array.
pub(super) fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && ((td::SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope())) || t.slope() == 0)
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70131`): active, solid,
/// NON-solidTop, non-halfBrick, slopeless, not actuated. Out-of-world → false.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && !t.inactive()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `Collision.SolidTiles(startX, endX, startY, endY)` (`Collision.cs:3455-
/// 3488`): out of bounds = solid; a cell counts when active, not actuated,
/// solid and non-solidTop (no slope/half-brick term).
fn solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for i in x0..=x1 {
        for j in y0..=y1 {
            if let Some(t) = world.tile(i, j) {
                if t.active()
                    && !t.inactive()
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `Player.PickSentryLandingSpot` (`Player.cs:49525-49543`): the placeable
/// staff sentries' spawn spot. The hover kinds (3571 Rainbow Crystal / 3569
/// Moon Lord Turret / 5463 Barnacle) skip the ground scan; the rest scan the
/// cursor column (and its two neighbours) down to the first `SolidTile2`
/// row, then step one row back up. `spawnX` is the RAW cursor X (unsnapped);
/// `spawnY = row * 16 - 24`. The port keeps `gravDir = 1` (no gravity potion,
/// ANALYSIS), so the C#'s `gravDir == -1` row re-fold is inert.
pub(super) fn pick_sentry_landing_spot(world: &World, item_kind: u16, cursor: Vec2) -> (f32, f32) {
    let hover = item_kind == 3571 || item_kind == 3569 || item_kind == 5463;
    // `(int)(mouseX + screenPosition.X) / 16` — the C# casts to int BEFORE
    // the integer divide (`:49528-49529`).
    let col = (cursor.x as i32) / 16;
    let mut row = (cursor.y as i32) / 16;
    if !hover {
        // `:49536-49538` — the three-column scan; a null (out-of-world) cell
        // stops it, exactly like the C# `Main.tile[..] != null` guard.
        while row < world.height - 10
            && world.tile(col, row).is_some()
            && !solid_tile2(world, col, row)
            && world.tile(col - 1, row).is_some()
            && !solid_tile2(world, col - 1, row)
            && world.tile(col + 1, row).is_some()
            && !solid_tile2(world, col + 1, row)
        {
            row += 1;
        }
        row -= 1;
    }
    let spawn_x = cursor.x;
    let spawn_y = row as f32 * 16.0 - 24.0;
    (spawn_x, spawn_y)
}

/// `Player.FindSentryRestingSpot` (`Player.cs:53015-53059`): the DD2 towers'
/// ground spot. The cursor is `Main.MouseWorld` pulled into the reachable
/// area; `worldX = col * 16 + 8`, `pushYUp = 41` with the per-kind offsets
/// (663/677/678 shift X by the facing; 665 +2 / 667 +3 to pushYUp; the traps
/// 691-693 reset pushYUp to 20, shift X, then +2). The three-column scan
/// finds the first solid row, `i++` then `i--` lands back ON it, `pushYUp -=
/// 14`, `worldY = row * 16`. Returns `(worldX, worldY, pushYUp)`.
pub(super) fn find_sentry_resting_spot(
    world: &World,
    check_proj: u16,
    cursor: Vec2,
    player_center: Vec2,
    facing: i32,
) -> (i32, i32, i32) {
    let point = super::limit_point_to_player_reachable_area(player_center, cursor, world);
    let col = (point.x as i32) / 16;
    let mut row = (point.y as i32) / 16;
    let mut world_x = col * 16 + 8;
    let mut push_y_up = 41;
    match check_proj {
        663 | 677 | 678 => world_x += facing,
        665 => push_y_up += 2,
        667 => push_y_up += 3,
        691 | 692 | 693 => {
            push_y_up = 20;
            world_x += facing;
            push_y_up += 2;
        }
        _ => {}
    }
    // `:53051-53054` — the scan (the C# `flag` is always false).
    while row < world.height - 10
        && world.tile(col, row).is_some()
        && !solid_tile2(world, col, row)
        && world.tile(col - 1, row).is_some()
        && !solid_tile2(world, col - 1, row)
        && world.tile(col + 1, row).is_some()
        && !solid_tile2(world, col + 1, row)
    {
        row += 1;
    }
    row += 1;
    row -= 1;
    push_y_up -= 14;
    let world_y = row * 16;
    (world_x, world_y, push_y_up)
}

/// `Player.WouldSpotOverlapWithSentry` (`Player.cs:52983-53013`): the three
/// probe points `(worldX, worldY-8)`, `(worldX±16, worldY-8)` against every
/// active sentry's hitbox. The lightning-aura hitbox inflates 8 px each side;
/// every other sentry's hitbox recentres to a fixed 30 px width (only when
/// its left edge sits past 30). Any probe contained → overlap.
pub(super) fn would_spot_overlap_with_sentry(
    pool: &ProjectilePool,
    world_x: i32,
    world_y: i32,
    lightning_aura: bool,
) -> bool {
    let probes = [
        (world_x, world_y - 8),
        (world_x + 16, world_y - 8),
        (world_x - 16, world_y - 8),
    ];
    for p in pool.slots() {
        if !p.active || !p.sentry {
            continue;
        }
        let mut hx = p.pos.x as i32;
        let mut hy = p.pos.y as i32;
        let mut hw = p.w as i32;
        let hh = p.h as i32;
        if lightning_aura {
            hx -= 8;
            hw += 16;
        } else if hx > 30 {
            hx += hw / 2;
            hw = 30;
            hx -= hw / 2;
        }
        for &(px, py) in &probes {
            if px >= hx && px < hx + hw && py >= hy && py < hy + hh {
                return true;
            }
        }
    }
    false
}

/// `Projectile.GetBallistraShotDelay` (`Projectile.cs:68653-68669`): 160
/// stock, 100 with setSquireT3, 60 with ballistaPanic, 30 with both. The
/// DD2 Ballista popper seeds the tower's `ai[1]` reload with this.
pub(super) fn ballistra_shot_delay(set_squire_t3: bool, ballista_panic: bool) -> i32 {
    let mut result = 160;
    if set_squire_t3 {
        result = 100;
    }
    if ballista_panic {
        result = 60;
    }
    if ballista_panic && set_squire_t3 {
        result = 30;
    }
    result
}

/// `Projectile.GetExplosiveTrapCooldown` (`Projectile.cs:69376-69388`): 90
/// stock, 60 with setHuntressT2, 30 with setHuntressT3. The DD2 Explosive
/// Trap popper seeds the trap's `localAI[0]` scan cooldown with this.
pub(super) fn explosive_trap_cooldown(set_huntress_t2: bool, set_huntress_t3: bool) -> i32 {
    let mut result = 90;
    if set_huntress_t2 {
        result = 60;
    }
    if set_huntress_t3 {
        result = 30;
    }
    result
}

/// `Player.GetRequiredDD2CrystalsToUse` (`Player.cs:42386-42409`): every DD2
/// tower popper costs 10 Eternia crystals; everything else 0.
pub(super) fn required_dd2_crystals(item_kind: u16) -> i32 {
    match item_kind {
        3818 | 3819 | 3820 | 3824 | 3825 | 3826 | 3829 | 3830 | 3831 | 3832 | 3833 | 3834 => 10,
        _ => 0,
    }
}

/// `Player.CountItem(3822, stopCountingAt)` (`Player.cs:9252-9267`) over the
/// port's split inventory: the 50-grid then the coin/ammo cells (the C# keeps
/// all 58 in one array). Sums the crystal stacks, stopping at `stop_at`.
fn count_crystals(inventory: &[ItemSlot], equip: &crate::inv::EquipSlots, stop_at: i32) -> i32 {
    let mut num = 0;
    let cells = inventory
        .iter()
        .chain(equip.coins.iter())
        .chain(equip.ammo.iter());
    for slot in cells {
        if slot.count > 0 && slot.kind == 3822 {
            num += slot.count as i32;
            if num >= stop_at {
                return num;
            }
        }
    }
    num
}

/// `Player.ConsumeItem(3822, reverseOrder: true)` (`Player.cs:9269-9308`):
/// one crystal off the LAST matching cell (the C# walks 57 → 0; the port's
/// reverse order is ammo[3..0], coins[3..0], grid[49..0]). A drained cell
/// turns to air (`SetDefaults(0)`). Returns whether one was consumed.
fn consume_one_crystal(inventory: &mut [ItemSlot], equip: &mut crate::inv::EquipSlots) -> bool {
    let cells = equip
        .ammo
        .iter_mut()
        .rev()
        .chain(equip.coins.iter_mut().rev())
        .chain(inventory.iter_mut().rev());
    for slot in cells {
        if slot.count > 0 && slot.kind == 3822 {
            slot.count -= 1;
            if slot.count == 0 {
                slot.kind = 0;
            }
            return true;
        }
    }
    false
}

/// `Player.CheckDD2CrystalPaymentLock` (`Player.cs:42376-42384`): outside the
/// event the use is free; during it the inventory must hold the full cost.
pub(super) fn dd2_crystal_payment_ok(
    inventory: &[ItemSlot],
    equip: &crate::inv::EquipSlots,
    item_kind: u16,
    dd2_ongoing: bool,
) -> bool {
    if !dd2_ongoing {
        return true;
    }
    let required = required_dd2_crystals(item_kind);
    count_crystals(inventory, equip, required) >= required
}

/// `Player.PayDD2CrystalsBeforeUse` (`Player.cs:42367-42374`): the
/// UNCONDITIONAL 10-crystal consume (the C# loops `required` times calling
/// `ConsumeItem`, which no-ops when the stock runs dry).
pub(super) fn pay_dd2_crystals(
    inventory: &mut [ItemSlot],
    equip: &mut crate::inv::EquipSlots,
    item_kind: u16,
) {
    let required = required_dd2_crystals(item_kind);
    for _ in 0..required {
        consume_one_crystal(inventory, equip);
    }
}

/// The DD2-turret use gates of `ItemCheck_CheckCanUse_Inner` (`Player.cs:
/// 52079-52111`), folded into one refusal test for a DD2 popper:
/// * `:52079` the crystal-payment lock;
/// * `:52083-52086` `!downedDD2EventAnyDifficulty && !Ongoing` refuses (the
///   port reads the world's three tier flags — the C# player field is their
///   per-tick latch, `:24450-24453` / `DD2Event.cs:89-99`);
/// * `:52087-52094` during the event, an overlap with an existing sentry at
///   the resting spot refuses (the aura kinds inflate the probe);
/// * `:52095-52111` the resting spot's tile-above-ground must be clear: the
///   aura kinds test a 3-tall `SolidTiles` column, the rest a single
///   `SolidTile`.
///
/// `downed_any` is `dd2_downed_t1 || t2 || t3`. Non-DD2 kinds pass trivially.
pub(super) fn dd2_sentry_can_use(
    world: &World,
    pool: &ProjectilePool,
    inventory: &[ItemSlot],
    equip: &crate::inv::EquipSlots,
    item_kind: u16,
    shoot: u16,
    cursor: Vec2,
    player_center: Vec2,
    facing: i32,
    dd2_ongoing: bool,
    downed_any: bool,
) -> bool {
    if !tr_core::dd2::is_dd2_turret(shoot) {
        return true;
    }
    if !dd2_crystal_payment_ok(inventory, equip, item_kind, dd2_ongoing) {
        return false;
    }
    if !downed_any && !dd2_ongoing {
        return false;
    }
    let (wx, wy, _) = find_sentry_resting_spot(world, shoot, cursor, player_center, facing);
    let lightning = shoot == 688 || shoot == 689 || shoot == 690;
    if dd2_ongoing && would_spot_overlap_with_sentry(pool, wx, wy, lightning) {
        return false;
    }
    // `:52097-52110` — the tile-above-ground refusal (runs regardless of the
    // event, the C# `num == Main.myPlayer` arm).
    let tx = wx / 16;
    let ty = wy / 16 - 1;
    if lightning {
        if solid_tiles(world, tx, tx, ty - 2, ty) {
            return false;
        }
    } else if solid_tile(world, tx, ty) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 64×800 world with a solid stone floor at row 300 (the shooter_app
    /// fixture shape).
    fn floor_world() -> World {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    fn crystal(count: u32) -> ItemSlot {
        ItemSlot {
            kind: 3822,
            name: "DD2 Energy Crystal",
            count,
            prefix: 0,
        }
    }

    fn empty_inventory() -> Vec<ItemSlot> {
        vec![crate::item::EMPTY_SLOT; 50]
    }

    /// `PickSentryLandingSpot` (`:49525-49543`): the falling kinds scan to the
    /// floor and step one row up (`spawnY = (300-1)*16 - 24 = 4760`); the
    /// hover kinds keep the cursor row (`spawnY = row*16 - 24`).
    #[test]
    fn the_landing_spot_scans_the_falling_kinds_and_hovers_the_rest() {
        let w = floor_world();
        // Cursor at (480, 4000) → col 30, row 250 (above the floor).
        let cursor = Vec2::new(480.0, 4000.0);
        let (sx, sy) = pick_sentry_landing_spot(&w, 1572, cursor);
        assert_eq!(sx, 480.0, "the raw cursor X, unsnapped");
        assert_eq!(sy, (300 - 1) as f32 * 16.0 - 24.0, "one row above the floor");
        // The Rainbow Crystal (3571) hovers: no scan, the cursor row stands.
        let (_, hy) = pick_sentry_landing_spot(&w, 3571, cursor);
        assert_eq!(hy, 250.0 * 16.0 - 24.0, "the hover kind keeps the cursor row");
        let (_, by) = pick_sentry_landing_spot(&w, 5463, cursor);
        assert_eq!(by, 250.0 * 16.0 - 24.0, "the barnacle hovers too");
    }

    /// `FindSentryRestingSpot` (`:53015-53059`): the Flameburst T1 (663)
    /// shifts X by the facing and lands `pushYUp = 41 - 14 = 27`; the T2/T3
    /// add 2/3; the traps reset to 20 + 2 - 14 = 8. `worldY = floorRow*16`.
    /// The cursor first rides `LimitPointToPlayerReachableArea`: on this
    /// 64-wide world the 1920-px view box clamps off the far edge (the C#
    /// `Utils.Clamp` max-first), pulling the cursor from col 30 to col 24
    /// (`point.X = 384`) — the spot keys off the CLAMPED column.
    #[test]
    fn the_resting_spot_offsets_match_the_csharp_switch() {
        let w = floor_world();
        let cursor = Vec2::new(480.0, 4000.0);
        let center = Vec2::new(480.0, 4000.0);
        let (wx, wy, push) = find_sentry_resting_spot(&w, 663, cursor, center, 1);
        assert_eq!(wx, 24 * 16 + 8 + 1, "clamped col centre + facing");
        assert_eq!(wy, 300 * 16, "the first solid row");
        assert_eq!(push, 27, "41 - 14");
        let (_, _, p2) = find_sentry_resting_spot(&w, 665, cursor, center, 1);
        assert_eq!(p2, 29, "665: +2");
        let (_, _, p3) = find_sentry_resting_spot(&w, 667, cursor, center, 1);
        assert_eq!(p3, 30, "667: +3");
        let (tx, _, tp) = find_sentry_resting_spot(&w, 691, cursor, center, -1);
        assert_eq!(tx, 24 * 16 + 8 - 1, "the trap shifts X by the facing");
        assert_eq!(tp, 8, "691: 20 + 2 - 14");
        // The Ballista (677) shifts X but keeps the base push.
        let (bx, _, bp) = find_sentry_resting_spot(&w, 677, cursor, center, 1);
        assert_eq!(bx, 24 * 16 + 8 + 1);
        assert_eq!(bp, 27);
    }

    /// `WouldSpotOverlapWithSentry` (`:52983-53013`): a sentry whose 30-px
    /// recentred hitbox covers a probe refuses; the aura's `Inflate(8, 0)`
    /// widens the RAW box 8 px each side (the C# `else if` — the recentre is
    /// skipped entirely on the aura branch). A clear spot passes.
    #[test]
    fn the_overlap_test_recentres_and_inflates_the_hitbox() {
        let mut pool = ProjectilePool::new();
        // A Frost Hydra (80×74) at top-left (100,100) — its left edge is past
        // 30, so the non-aura branch recentres: X = 100 + 80/2 - 30/2 = 125,
        // W 30 → [125,155)×[100,174).
        let hydra = pool
            .spawn_with_data_ai_owned(308, 0, 100.0, 100.0, 0.0, 0.0, 10, 0.0, 0.0, 0.0)
            .unwrap();
        // Probe (worldX, worldY-8): worldX 130, worldY 108 → (130,100) sits
        // inside the recentred box.
        assert!(would_spot_overlap_with_sentry(&pool, 130, 108, false));
        // A probe well outside passes.
        assert!(!would_spot_overlap_with_sentry(&pool, 400, 108, false));
        // The aura branch (`:52995-52998`): `hitbox.Inflate(8, 0)` on the RAW
        // box — the Hydra's [100,180) becomes [92,188). Probe (95,100) is
        // inside the inflated box but OUTSIDE the plain recentred [125,155):
        // the inflate widens the catch.
        assert!(would_spot_overlap_with_sentry(&pool, 95, 108, true));
        assert!(!would_spot_overlap_with_sentry(&pool, 95, 108, false));
        // A Lightning Aura kind (16×16) at (100,100): the aura branch inflates
        // its RAW [100,116) to [92,124); probe (93,100) is caught. The plain
        // branch recentres it to X = 100 + 16/2 - 15 = 93, W 30 → [93,123),
        // which also contains 93 — the aura-only delta is the ±8 reach past
        // the recentred box, proven by the Hydra case above.
        let aura = pool
            .spawn_with_data_ai_owned(688, 0, 100.0, 100.0, 0.0, 0.0, 4, 0.0, 0.0, 0.0)
            .unwrap();
        assert!(would_spot_overlap_with_sentry(&pool, 93, 108, true));
        // (123,100): inside the aura's inflated [92,124) but outside the
        // plain recentred [93,123) — the aura catches the extra column.
        assert!(would_spot_overlap_with_sentry(&pool, 123, 108, true));
        let _ = (hydra, aura);
    }

    /// `GetRequiredDD2CrystalsToUse` (`:42386-42409`): the 12 poppers cost 10,
    /// everything else 0.
    #[test]
    fn the_crystal_cost_is_ten_for_every_popper() {
        for k in [3818, 3819, 3820, 3824, 3825, 3826, 3829, 3830, 3831, 3832, 3833, 3834] {
            assert_eq!(required_dd2_crystals(k), 10, "kind {k}");
        }
        assert_eq!(required_dd2_crystals(1572), 0);
        assert_eq!(required_dd2_crystals(0), 0);
    }

    /// `PayDD2CrystalsBeforeUse` (`:42367-42374`) + `ConsumeItem` reverse
    /// order (`:9269-9308`): the consume drains the LAST cells first (ammo →
    /// coins → grid) and turns a drained cell to air.
    #[test]
    fn the_payment_drains_reverse_and_airs_the_empty_cell() {
        let mut inv = empty_inventory();
        inv[0] = crystal(4);
        inv[49] = crystal(3);
        let mut equip = crate::inv::EquipSlots::empty();
        equip.ammo[3] = crystal(5);
        pay_dd2_crystals(&mut inv, &mut equip, 3818);
        // 10 consumed: ammo[3] 5 → 0 (aired), grid[49] 3 → 0 (aired),
        // grid[0] 4 → 2.
        assert_eq!(equip.ammo[3].count, 0);
        assert_eq!(equip.ammo[3].kind, 0, "the drained ammo cell turns to air");
        assert_eq!(inv[49].count, 0);
        assert_eq!(inv[49].kind, 0);
        assert_eq!(inv[0].count, 2, "the remainder comes off the first grid cell");
    }

    /// `CheckDD2CrystalPaymentLock` (`:42376-42384`): free outside the event,
    /// the full cost required during it.
    #[test]
    fn the_payment_lock_gates_on_the_event() {
        let mut inv = empty_inventory();
        let equip = crate::inv::EquipSlots::empty();
        // Outside the event: no crystals needed.
        assert!(dd2_crystal_payment_ok(&inv, &equip, 3818, false));
        // During the event with no stock: refused.
        assert!(!dd2_crystal_payment_ok(&inv, &equip, 3818, true));
        inv[0] = crystal(10);
        assert!(dd2_crystal_payment_ok(&inv, &equip, 3818, true));
        inv[0] = crystal(9);
        assert!(!dd2_crystal_payment_ok(&inv, &equip, 3818, true), "9 < 10");
    }

    /// `GetBallistraShotDelay` (`Projectile.cs:68653-68669`) and
    /// `GetExplosiveTrapCooldown` (`:69376-69388`): the set/panic ladders.
    #[test]
    fn the_reload_ladders_match_the_csharp_overrides() {
        assert_eq!(ballistra_shot_delay(false, false), 160);
        assert_eq!(ballistra_shot_delay(true, false), 100);
        assert_eq!(ballistra_shot_delay(false, true), 60);
        assert_eq!(ballistra_shot_delay(true, true), 30);
        assert_eq!(explosive_trap_cooldown(false, false), 90);
        assert_eq!(explosive_trap_cooldown(true, false), 60);
        assert_eq!(explosive_trap_cooldown(false, true), 30);
        assert_eq!(explosive_trap_cooldown(true, true), 30, "T3 wins over T2");
    }

    /// The `:52083-52086` downed gate: a DD2 popper refuses before any tier is
    /// downed and outside the event; a downed tier or a live event opens it.
    #[test]
    fn the_dd2_use_needs_a_downed_tier_or_a_live_event() {
        let w = floor_world();
        let pool = ProjectilePool::new();
        let inv = empty_inventory();
        let equip = crate::inv::EquipSlots::empty();
        let cursor = Vec2::new(480.0, 4000.0);
        let center = Vec2::new(480.0, 4000.0);
        // No downed tier, no event → refused (the crystal lock passes offline).
        assert!(!dd2_sentry_can_use(
            &w, &pool, &inv, &equip, 3818, 663, cursor, center, 1, false, false
        ));
        // A downed tier opens it (the resting spot's tile-above is clear air).
        assert!(dd2_sentry_can_use(
            &w, &pool, &inv, &equip, 3818, 663, cursor, center, 1, false, true
        ));
        // A non-DD2 kind passes trivially.
        assert!(dd2_sentry_can_use(
            &w, &pool, &inv, &equip, 1572, 308, cursor, center, 1, false, false
        ));
    }
}
