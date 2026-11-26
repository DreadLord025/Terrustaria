//! `WorldGen.ShakeTree`'s loot ladder (`WorldGen.cs:63163-63403`), W4-g17
//! wiring wave 2 — the genRand item/NPC/projectile arms that run BETWEEN
//! the shake dedupe/walk (`tree::shake_tree_resolve`, `:63133-63162`) and
//! the FX tail (`tree::shake_tree_fx_at`, `:63404-63417`).
//!
//! The axe fail tick (`axe_tile_hit`, `Player.cs:45758` → `KillTile(fail:
//! true)` → `WorldGen.cs:63710-63714`) resolves the shake site ONCE, runs
//! this ladder, then the FX tail — the C# order. The i30 axe-chop lane owns
//! the chop/damage book and the per-cell BREAK drops (`KillTile_GetItemDrops`
//! via `break_tile`); this is the per-shake loot, a different system.
//!
//! ## RNG contract
//!
//! * The ladder's gates + type/stack picks ride `WorldGen.genRand` — the
//!   edit lane's `FastRandom` stand-in seeded by ONE `UpdatePlayers` throw
//!   per shake (the `spawn_things_from_pot` convention), drawn BEFORE the
//!   FX tail (which is RNG-free).
//! * The `Main.rand` draws (the getGood bomb's velocity `Next(-100,101)`,
//!   the fairy/owl/macaw `NextFromList` picks, the tenth `Next(4)`, the
//!   bird-burst point/velocity rolls, the gold-critter `RollLuck`, and the
//!   `Item.NewItem`/`NPC.NewNPC` internals) ride the `UpdatePlayers` stream
//!   (the ItemCheck swap, `Main.cs:17754`).
//! * `Player.GetClosestRollLuck` (`Player.cs:18236-18239` → `RollLuck` →
//!   `Luck.RollLuck`, `Luck.cs:5-14`) draws on `Main.rand` via
//!   `tr_core::loot::roll_luck`; `NPC.goldCritterChance` = 400
//!   (`NPC.cs:95159`).
//!
//! ## Documented stand-ins / drops
//!
//! * `Item.NewItem`'s `prefix: 0` (every arm but `:63225`) draws NO prefix
//!   (`Item.cs:511-514` returns before `CanHavePrefixes`); the port's
//!   `spawn_item_drop` rolls the natural `Prefix(-1)` — the house-wide
//!   recorded deviation (`items.rs:767-770`). The `:63225` arm's
//!   `noPrefix ? -4 : 0` maps to `roll_prefix(kind, -4, rng)` (clears to
//!   none, `Item.cs` `prefixWeWant == -4`) vs the natural roll.
//! * `Main.halloween`/`Main.xMas` have no port state (false) — the `:63215`
//!   halloween pumpkin arm draws its `Next(35)` gate but never spawns.
//! * Unported NPC kinds (the bird/squirrel/fairy/owl/macaw/butterfly/seagull/
//!   hell-butterfly critters, the Crimera 173) consume their rolls then
//!   spawn nothing — the `kind_from_net_id` convention (`npc/data.rs:3996`).
//! * `NPC.NewNPC`'s `Bottom` anchor (`NPC.cs` `nPC.Bottom = new Vector2(X,
//!   Y)`) maps to top-left `(X - w/2, Y - h)`; `timeLeft = activeTime*1.25`
//!   = 937 (`NPC.cs:95186` activeTime 750) maps to `despawn_timer`. The
//!   `CanBeReplacedByOtherNPCs` flag (the jungle bee `:63319`) is unported.
//! * `Projectile.NewProjectile` for the getGood bomb (28/75) and the jungle
//!   hive (655) rides `spawn_with_data_ai`; the `npcProj = true` flag
//!   (`:63177`) is unported (no projectile field). Unrowed kinds (75/655)
//!   consume their rolls then spawn nothing (`proj_data` returns None).

use glam::Vec2;
use tr_core::tree::{ShakeTreeSite, TreeType, is_palm_oasis_tree};
use tr_core::{UnifiedRandom, TILE_PX};

use crate::app::App;
use crate::npc::Npc;
use crate::world_sim::items::spawn_item_drop;

/// `NPC.goldCritterChance` (`NPC.cs:95159`).
const GOLD_CRITTER_CHANCE: i32 = 400;

/// `Main.halloween` — no port state (the seasonal flags are constants-off,
/// the `shop.rs` halloween-row convention). The `:63215`/`:63297` gates
/// still DRAW their genRand rolls, then fall through.
const HALLOWEEN: bool = false;

/// The world/player flags the ladder reads, snapshotted from `App` before
/// the mutable stream borrows. `Main.halloween`/`xMas` are unported (false).
struct ShakeFlags {
    tenth: bool,
    remix: bool,
    not_the_bees: bool,
    drunk: bool,
    get_good: bool,
    day_time: bool,
    raining: bool,
    /// `NPC.TooWindyForButterflies` (`NPC.cs:6885`): `|windSpeedTarget| >= 0.4`.
    too_windy: bool,
    max_tiles_y: i32,
    max_tiles_x: i32,
    /// The closest player's luck (`Player.GetClosestRollLuck` — the local one).
    luck: f32,
    /// The closest player's held axe power (`KillTile_GetTreeDrops`,
    /// `WorldGen.cs:66255-66256`).
    axe: i32,
}

impl ShakeFlags {
    fn from_app(app: &App) -> Self {
        use tr_core::seeds::SeedFlags as S;
        let seeds = app.world.seeds;
        let (luck, axe) = match &app.player {
            Some(p) => (
                p.luck(),
                crate::registry::get(app.inventory[app.selected].kind)
                    .map_or(0, |d| i32::from(d.axe)),
            ),
            None => (0.0, 0),
        };
        ShakeFlags {
            tenth: seeds.contains(S::TENTH_ANNIVERSARY),
            remix: seeds.contains(S::REMIX),
            not_the_bees: seeds.contains(S::NOT_THE_BEES),
            drunk: seeds.contains(S::DRUNK),
            get_good: seeds.contains(S::FOR_THE_WORTHY),
            day_time: app.world.day_time,
            raining: app.rain.is_raining(),
            too_windy: app.wind.speed_target.abs() >= 0.4,
            max_tiles_y: app.world.height,
            max_tiles_x: app.world.width,
            luck,
            axe,
        }
    }
}

/// `WorldGen.ShakeTree`'s loot ladder (`WorldGen.cs:63163-63403`): the full
/// genRand else-if chain. `gen_rand` is the per-shake genRand stand-in, `rng` the
/// `UpdatePlayers` stream (`Main.rand`). Runs after `shake_tree_resolve`
/// records the shake, before the FX tail.
pub(super) fn shake_tree_loot(
    app: &mut App,
    site: &ShakeTreeSite,
    i: i32,
    j: i32,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    rng: &mut UnifiedRandom,
) {
    let flags = ShakeFlags::from_app(app);
    let tt = site.tree_type;
    let (bx, by) = (site.bottom_x, site.bottom_y);
    // `:63163-63169` — the getGood bomb's gate range + type.
    let mut num2 = 17;
    let mut bomb_type: u16 = 28;
    if flags.tenth && !flags.remix {
        num2 *= 2;
        bomb_type = 75;
    }
    // The forest/snow/hallowed/ash band the acorn + fruit arms read.
    let fsha = matches!(tt, TreeType::Forest | TreeType::Snow | TreeType::Hallowed | TreeType::Ash);
    let fh = matches!(tt, TreeType::Forest | TreeType::Hallowed);
    let palm = matches!(tt, TreeType::Palm | TreeType::PalmCorrupt | TreeType::PalmCrimson | TreeType::PalmHallowed);
    let oasis = is_palm_oasis_tree(bx, flags.max_tiles_x);

    // `:63170-63173` — the tenth + notTheBees + !drunk honey grenade (1130).
    if flags.tenth && flags.not_the_bees && !flags.drunk && gen_rand.next(50) == 0 {
        let stack = rng.next_range(12, 21) as u32;
        spawn_item_drop(&mut app.items, rng, 1130, stack, i, j);
        return;
    }
    // `:63174-63178` — the getGood bomb projectile (npcProj).
    if flags.get_good && gen_rand.next(num2) == 0 {
        let vx = rng.next_range(-100, 101) as f32 * 0.002;
        // `Projectile.NewProjectile(..., bottomX*16, bottomY*16, vx, 0, type,
        // 0, 0, myPlayer, 16, 16)`; the ai0/ai1 args are 16/16.
        let _ = app.projectiles.spawn_with_data_ai(
            bomb_type, bx as f32 * TILE_PX, by as f32 * TILE_PX, vx, 0.0, 0, 0.0, 16.0, 16.0,
        );
        return;
    }
    // `:63179-63202` — the forest/jungle rare items (832/933/3360/3361/5517/4366).
    if gen_rand.next(300) == 0 && tt == TreeType::Forest {
        spawn_item_drop(&mut app.items, rng, 832, 1, bx, by);
        return;
    }
    if gen_rand.next(300) == 0 && tt == TreeType::Forest {
        spawn_item_drop(&mut app.items, rng, 933, 1, bx, by);
        return;
    }
    if gen_rand.next(200) == 0 && tt == TreeType::Jungle {
        spawn_item_drop(&mut app.items, rng, 3360, 1, bx, by);
        return;
    }
    if gen_rand.next(200) == 0 && tt == TreeType::Jungle {
        spawn_item_drop(&mut app.items, rng, 3361, 1, bx, by);
        return;
    }
    if gen_rand.next(1000) == 0 && tt == TreeType::Forest {
        spawn_item_drop(&mut app.items, rng, 5517, 1, bx, by);
        return;
    }
    if gen_rand.next(1000) == 0 && tt == TreeType::Forest {
        spawn_item_drop(&mut app.items, rng, 4366, 1, bx, by);
        return;
    }
    // `:63203-63210` — the 5629 rare + the acorn (27).
    if gen_rand.next(100) == 0 && fsha {
        spawn_item_drop(&mut app.items, rng, 5629, 1, bx, by);
        return;
    }
    if gen_rand.next(7) == 0 && fsha {
        let stack = gen_rand.next_range(1, 3) as u32;
        spawn_item_drop(&mut app.items, rng, 27, stack, bx, by);
        return;
    }
    // `:63211-63214` — the mushroom (194).
    if gen_rand.next(8) == 0 && tt == TreeType::Mushroom {
        let stack = gen_rand.next_range(1, 2) as u32;
        spawn_item_drop(&mut app.items, rng, 194, stack, bx, by);
        return;
    }
    // `:63215-63218` — the halloween pumpkin (1809). `Next(35) == 0 &&
    // Main.halloween` — the gate draws FIRST; halloween is unported (false),
    // so a hit falls THROUGH to the next else-if (the C# `&&` short-circuit).
    if gen_rand.next(35) == 0 && HALLOWEEN {
        let stack = gen_rand.next_range(1, 3) as u32;
        spawn_item_drop(&mut app.items, rng, 1809, stack, bx, by);
        return;
    }
    // `:63219-63227` — the per-cell wood drop (KillTile_GetItemDrops).
    if gen_rand.next(12) == 0 {
        shake_cell_drop(app, site, i, j, gen_rand, rng, &flags);
        return;
    }
    // `:63228-63259` — the coin arm (71 copper / 73 gold / 72 silver).
    if gen_rand.next(20) == 0 {
        let mut coin: u16 = 71;
        let mut stack = gen_rand.next_range(50, 100);
        if gen_rand.next(30) == 0 {
            coin = 73;
            stack = 1;
            if gen_rand.next(5) == 0 {
                stack += 1;
            }
            if gen_rand.next(10) == 0 {
                stack += 1;
            }
        } else if gen_rand.next(10) == 0 {
            coin = 72;
            stack = gen_rand.next_range(1, 21);
            if gen_rand.next(3) == 0 {
                stack += gen_rand.next_range(1, 21);
            }
            if gen_rand.next(4) == 0 {
                stack += gen_rand.next_range(1, 21);
            }
        }
        spawn_item_drop(&mut app.items, rng, coin, stack as u32, bx, by);
        return;
    }
    // `:63260-63275` — the bird/squirrel critter (forest/hallowed).
    if gen_rand.next(15) == 0 && fh {
        let mut ty: i32 = match gen_rand.next(5) {
            0 => 74,
            1 => 297,
            2 => 298,
            3 => 299,
            _ => 538,
        };
        if gold_critter_roll(rng, flags.luck) {
            ty = if gen_rand.next(2) != 0 { 539 } else { 442 };
        }
        spawn_shake_npc(app, rng, ty, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63276-63284` — the hallowed night fairy (583/584/585).
    if gen_rand.next(50) == 0 && tt == TreeType::Hallowed && !flags.day_time {
        let mut ty = [583i32, 584, 585][rng.next_max(3) as usize];
        if flags.tenth && rng.next_max(4) != 0 {
            ty = 583;
        }
        spawn_shake_npc(app, rng, ty, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63285-63290` — the forest night owl (611), velocity.Y = 1.
    if gen_rand.next(50) == 0 && tt == TreeType::Forest && !flags.day_time {
        spawn_shake_npc(app, rng, 611, bx as f32 * TILE_PX, by as f32 * TILE_PX, Some(Vec2::new(0.0, 1.0)), flags.get_good);
        return;
    }
    // `:63291-63296` — the jungle day macaw (671-675), velocity.Y = 1.
    if gen_rand.next(50) == 0 && tt == TreeType::Jungle && flags.day_time {
        let ty = [671i32, 672, 673, 674, 675][rng.next_max(5) as usize];
        spawn_shake_npc(app, rng, ty, bx as f32 * TILE_PX, by as f32 * TILE_PX, Some(Vec2::new(0.0, 1.0)), flags.get_good);
        return;
    }
    // `:63297-63300` — the forest night halloween raven (301). The gate
    // draws `Next(40)` + the family/night terms; halloween is unported, so
    // a hit falls THROUGH (the C# `&&` short-circuit).
    if gen_rand.next(40) == 0 && tt == TreeType::Forest && !flags.day_time && HALLOWEEN {
        spawn_shake_npc(app, rng, 301, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63301-63311` — the 5-bird burst (forest/hallowed).
    if gen_rand.next(50) == 0 && fh {
        for _ in 0..5 {
            let px = bx + rng.next_range(-2, 2);
            let py = by - 1 + rng.next_range(-2, 2);
            let ty = if !gold_critter_roll(rng, flags.luck) {
                [74i32, 297, 298][rng.next_max(3) as usize]
            } else {
                442
            };
            let vel = next_vector2_circular_edge(rng, 3.0, 3.0);
            spawn_shake_npc(app, rng, ty, px as f32 * TILE_PX, py as f32 * TILE_PX, Some(vel), flags.get_good);
        }
        return;
    }
    // `:63312-63322` — the 5-bee burst (jungle), ai[1] = 65.
    if gen_rand.next(40) == 0 && tt == TreeType::Jungle {
        for _ in 0..5 {
            let px = bx + rng.next_range(-2, 2);
            let py = by - 1 + rng.next_range(-2, 2);
            let ty = [210i32, 211][rng.next_max(2) as usize];
            let spawned = spawn_shake_npc(app, rng, ty, px as f32 * TILE_PX, py as f32 * TILE_PX, None, flags.get_good);
            // `obj4.ai[1] = 65f` — set on the freshly spawned bee (only when
            // the kind spawned — the C# writes the new slot, never a stale one).
            if spawned {
                if let Some(n) = app.npcs.last_mut() {
                    n.ai[1] = 65.0;
                }
            }
        }
        return;
    }
    // `:63323-63326` — the palm seagull (603), not an oasis tree.
    if gen_rand.next(20) == 0 && palm && !oasis {
        spawn_shake_npc(app, rng, 603, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63327-63330` — the crimson Crimera (-22 → 173).
    if gen_rand.next(30) == 0 && matches!(tt, TreeType::Crimson | TreeType::PalmCrimson) {
        spawn_shake_npc(app, rng, -22, bx as f32 * TILE_PX + 8.0, (by - 1) as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63331-63334` — the corrupt Eater of Souls (-11 → 6).
    if gen_rand.next(30) == 0 && matches!(tt, TreeType::Corrupt | TreeType::PalmCorrupt) {
        spawn_shake_npc(app, rng, -11, bx as f32 * TILE_PX + 8.0, (by - 1) as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63335-63338` — the jungle night bat (51).
    if gen_rand.next(30) == 0 && tt == TreeType::Jungle && !flags.day_time {
        spawn_shake_npc(app, rng, 51, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63339-63342` — the jungle hive projectile (655).
    if gen_rand.next(40) == 0 && tt == TreeType::Jungle {
        let _ = app.projectiles.spawn_with_data_ai(
            655, bx as f32 * TILE_PX + 8.0, (by - 1) as f32 * TILE_PX, 0.0, 0.0, 0, 0.0, 0.0, 0.0,
        );
        return;
    }
    // `:63343-63351` — the butterfly (356/444), day + !raining + !windy.
    if gen_rand.next(20) == 0 && fh && !flags.raining && !flags.too_windy && flags.day_time {
        let ty = if gold_critter_roll(rng, flags.luck) { 444 } else { 356 };
        spawn_shake_npc(app, rng, ty, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63352-63360` — the ash underworld critter (654/653/655).
    if gen_rand.next(20) == 0 && tt == TreeType::Ash && by > flags.max_tiles_y - 250 {
        let ty = match gen_rand.next(3) {
            0 => 654,
            1 => 653,
            _ => 655,
        };
        spawn_shake_npc(app, rng, ty, bx as f32 * TILE_PX, by as f32 * TILE_PX, None, flags.get_good);
        return;
    }
    // `:63361-63364` — the remix ash obsidian (965).
    if flags.remix && gen_rand.next(20) == 0 && tt == TreeType::Ash && by > flags.max_tiles_y - 250 {
        let stack = gen_rand.next_range(20, 41) as u32;
        spawn_item_drop(&mut app.items, rng, 965, stack, bx, by);
        return;
    }
    // `:63365-63403` — the per-biome fruit/seed items (the 12-gate band).
    if gen_rand.next(12) == 0 && tt == TreeType::Forest {
        let ty = match gen_rand.next(5) {
            0 => 4009,
            1 => 4293,
            2 => 4282,
            3 => 4290,
            _ => 4291,
        };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && tt == TreeType::Snow {
        let ty = if gen_rand.next(2) != 0 { 4295 } else { 4286 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && tt == TreeType::Jungle {
        let ty = if gen_rand.next(2) != 0 { 4292 } else { 4294 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && palm && !oasis {
        let ty = if gen_rand.next(2) != 0 { 4287 } else { 4283 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && matches!(tt, TreeType::Corrupt | TreeType::PalmCorrupt) {
        let ty = if gen_rand.next(2) != 0 { 4289 } else { 4284 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && matches!(tt, TreeType::Hallowed | TreeType::PalmHallowed) {
        let ty = if gen_rand.next(2) != 0 { 4288 } else { 4297 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && matches!(tt, TreeType::Crimson | TreeType::PalmCrimson) {
        let ty = if gen_rand.next(2) != 0 { 4285 } else { 4296 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    if gen_rand.next(12) == 0 && tt == TreeType::Ash {
        let ty = if gen_rand.next(2) != 0 { 5278 } else { 5277 };
        spawn_item_drop(&mut app.items, rng, ty, 1, bx, by);
        return;
    }
    // No arm hit — the FX tail still runs (the caller's).
}

/// `Player.GetClosestRollLuck(bottomX, bottomY, NPC.goldCritterChance) == 0f`
/// (`Player.cs:18236-18239` → `Luck.RollLuck`, `Luck.cs:5-14`): the gold
/// critter gate, on `Main.rand`.
fn gold_critter_roll(rng: &mut UnifiedRandom, luck: f32) -> bool {
    tr_core::loot::roll_luck(rng, luck, GOLD_CRITTER_CHANCE) == 0
}

/// `Utils.NextVector2CircularEdge` (`Utils.cs`): the unit draw ONLY (no
/// radius scale — unlike `NextVector2Circular`), times the half-extents.
fn next_vector2_circular_edge(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
    Vec2::new(unit.x * half_w, unit.y * half_h)
}

/// `NPC.NewNPC(source, X, Y, Type)` for the shake arms: the getGood
/// `Next(3) != 0` type-swap gate (`NPC.cs` NewNPC head — only 46/62 swap,
/// neither a shake kind, but the DRAW runs on every getGood spawn), the
/// `Bottom` anchor (`nPC.Bottom = new Vector2(X, Y)`) maps to top-left
/// `(X - w/2, Y - h)`, `timeLeft = activeTime*1.25` = 937 (`NPC.cs:95186`).
/// Negative ids resolve through `NPCID.FromNetId`'s `NetIdMap` (`NPCID.cs`).
/// Unported kinds consume the getGood draw then spawn nothing. Returns
/// whether an NPC was pushed (the bee burst's `ai[1] = 65` write guards on
/// it — the C# writes the freshly spawned bee, never a stale slot).
fn spawn_shake_npc(app: &mut App, rng: &mut UnifiedRandom, net_id: i32, x: f32, y: f32, vel: Option<Vec2>, get_good: bool) -> bool {
    // `NPC.NewNPC` head: `if (Main.getGoodWorld && Main.rand.Next(3) != 0)`
    // — the draw runs unconditionally on a getGood world (the 46/62 swaps
    // never match a shake kind).
    if get_good {
        let _ = rng.next_max(3);
    }
    let resolved = if net_id < 0 {
        // `NPCID.NetIdMap` (`NPCID.cs`): -11 → 6, -22 → 173.
        const NET_ID_MAP: [i32; 65] = [
            81, 81, 1, 1, 1, 1, 1, 1, 1, 1, 6, 6, 31, 31, 77, 42, 42, 176, 176, 176,
            176, 173, 173, 183, 183, 3, 3, 132, 132, 186, 186, 187, 187, 188, 188, 189,
            189, 190, 191, 192, 193, 194, 2, 200, 200, 21, 21, 201, 201, 202, 202, 203,
            203, 223, 223, 231, 231, 232, 232, 233, 233, 234, 234, 235, 235,
        ];
        NET_ID_MAP[(-net_id - 1) as usize]
    } else {
        net_id
    };
    let Some((kind, variant)) = crate::npc::kind_from_net_id(resolved) else {
        return false;
    };
    let (w, h) = kind.size();
    let mut n = Npc::new(kind, x - w * 0.5, y - h, 1, 0.0);
    n.slime = variant;
    n.hp = n.max_hp;
    // `NPC.NewNPC`: `timeLeft = (int)(activeTime * 1.25)` = 937.
    n.despawn_timer = 937;
    if let Some(v) = vel {
        n.vx = v.x;
        n.vy = v.y;
    }
    app.npcs.push(n);
    true
}

/// The `:63219-63227` per-cell wood drop: `KillTile_GetItemDrops(i, j,
/// Main.tile[i, j], out dropItem, out _, out _, out _, out noPrefix)` then
/// `Item.NewItem(..., dropItem, genRand.Next(1, 4), noPrefix ? -4 : 0)`.
/// The C# DISCARDS `dropItemStack`/`secondaryItem` (the `out _` args) — the
/// resolver's internal stack rolls still DRAW, but the spawn always uses
/// `genRand.Next(1, 4)`. Only the shakeable kinds reach here
/// (`SET_IS_SHAKEABLE`: 5/72/323/583-589/596/616/634).
fn shake_cell_drop(
    app: &mut App,
    site: &ShakeTreeSite,
    i: i32,
    j: i32,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    rng: &mut UnifiedRandom,
    flags: &ShakeFlags,
) {
    let Some(tile) = app.world.tile(i, j).copied() else {
        return;
    };
    let (bx, by) = (site.bottom_x, site.bottom_y);
    // The resolver returns `(dropItem, noPrefix)`; the stack/secondary it
    // computes internally are discarded (the C# `out _`), but its rolls draw.
    let (drop_item, no_prefix) = match tile.kind {
        // `case 5/596/616/634` (`:65214-65231`) → KillTile_GetTreeDrops.
        5 | 596 | 616 | 634 => {
            let item = shake_tree_drops(app, i, j, &tile, gen_rand, rng, flags);
            (item, false)
        }
        // `case 323` (`:65232-65267`) — the palm wood. The tenth stack roll
        // (`genRand.Next(2, 5)`) draws into the discarded dropItemStack.
        323 => {
            let mut item: u16 = 2504;
            if flags.tenth {
                let _ = gen_rand.next_range(2, 5);
            }
            // The ground-row scan picks the palm wood variant.
            let mut gy = j;
            while gy < flags.max_tiles_y {
                let t = app.world.tile(i, gy);
                if t.is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)) {
                    break;
                }
                gy += 1;
            }
            if let Some(t) = app.world.tile(i, gy) {
                match t.kind {
                    234 => item = 911,
                    116 => item = 621,
                    112 => item = 619,
                    _ => {}
                }
            }
            (item, false)
        }
        // `case 72` (`:65642-65652`) — the mushroom tree.
        72 => {
            if rng.next_max(40) == 0 {
                (194u16, false)
            } else if rng.next_max(2) == 0 {
                (183, false)
            } else {
                (0, false)
            }
        }
        // `case 583-589` (`:65699-65746`) — the gem trees (SetGemTreeDrops).
        583..=589 => {
            let gem: u16 = match tile.kind {
                583 => 180,
                584 => 181,
                585 => 177,
                586 => 179,
                587 => 178,
                588 => 182,
                _ => 999,
            };
            let item = if rng.next_max(10) == 0 { gem } else { 3 };
            // The canopy gemcorn is the SECONDARY item (discarded) — the
            // Next(2) roll still draws (C# order, SetGemTreeDrops :66104).
            if tile.frame_x >= 22 && tile.frame_y >= 198 {
                let _ = rng.next_max(2);
            }
            // The stone's `Next(1, 3)` stack draws into the discarded
            // dropItemStack (`:65701-65704`).
            if item == 3 {
                let _ = rng.next_range(1, 3);
            }
            (item, false)
        }
        _ => (0, false),
    };
    if drop_item > 0 {
        // `:63225` — the stack is ALWAYS genRand.Next(1, 4); the prefix is
        // `noPrefix ? -4 : 0` (-4 clears to none, 0 draws nothing — the
        // natural-roll deviation is spawn_item_drop's, documented).
        let stack = gen_rand.next_range(1, 4) as u32;
        if no_prefix {
            let _ = crate::prefix::roll_prefix(drop_item, -4, rng);
        }
        spawn_item_drop(&mut app.items, rng, drop_item, stack, bx, by);
    }
}

/// `WorldGen.KillTile_GetTreeDrops` (`WorldGen.cs:66189-66261`) for the
/// shake's `:63219` arm: the canopy acorn roll, the family wood, the bonus
/// wood. Returns `dropItem` only (the shake arm discards the secondary 27
/// and the bonus-wood stack bump). The bonus roll still DRAWS.
fn shake_tree_drops(
    app: &App,
    i: i32,
    j: i32,
    tile: &tr_core::tile::Tile,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    rng: &mut UnifiedRandom,
    flags: &ShakeFlags,
) -> u16 {
    // `:66191-66216` — the canopy marker's acorn roll (netMode != 1). Every
    // branch sets dropItem = 9 (the secondary acorn 27 is discarded here).
    let mut drop_item: u16 = if tile.frame_x >= 22 && tile.frame_y >= 198 {
        let _ = gen_rand.next(2);
        9
    } else {
        9
    };
    // `:66224-66254` — the family wood (GetTreeTypeAndTreeBottom).
    let (tt, _bx, _by) = tr_core::tree::tree_type_and_bottom(&app.world, i, j);
    match tt {
        TreeType::Ash => drop_item = 5215,
        TreeType::Corrupt => drop_item = 619,
        TreeType::Crimson => drop_item = 911,
        TreeType::Jungle => drop_item = 620,
        TreeType::Hallowed => drop_item = 621,
        TreeType::Snow => drop_item = 2503,
        TreeType::Mushroom => {
            drop_item = if gen_rand.next(2) == 0 { 183 } else { 0 };
        }
        _ => {}
    }
    // `:66255-66260` — the bonus wood roll draws (the stack bump is discarded).
    let _bonus = gen_rand.next(35) <= flags.axe || rng.next_max(3) == 0;
    drop_item
}