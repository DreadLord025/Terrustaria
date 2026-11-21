//! The recall / magic-mirror / ice-mirror / potion-of-return use path —
//! `Player.ItemCheck`'s spawn-recall arms and the `Player.Spawn`
//! (`PlayerSpawnContext.RecallFromItem`) spawn-point machinery they ride.
//!
//! ## What this ports (the C# arms, in `ItemCheck_OwnerOnlyCode` order)
//!
//! * **The mirror family** — Magic Mirror 50, Cell Phone 3124, Ice Mirror
//!   3199, Shellphone 5358 (`Player.cs:42976-42999`, the shared
//!   `item.type == 50 || 3124 || 3199 || 5358` block). Every tick the swing
//!   runs it rolls `Main.rand.Next(2)`; on 0 it spawns one type-15 dust
//!   (`:42978-42981`). At `ItemTimeIsZero` it `ApplyItemTime`s (`:42982-42985`);
//!   at `itemTime == item.useTime / 2` it fires the recall: 70 dust at the OLD
//!   position (`:42988-42991`), `RemoveAllGrapplingHooks` (`:42992`),
//!   `Spawn(RecallFromItem)` (`:42993`), then 70 dust at the NEW position
//!   (`:42994-42997`). **No stack consume, no immune save/restore** — the
//!   mirror keeps the 60-tick spawn immunity `Spawn` grants. Its `UseSound`
//!   (Item6) plays once at the use START (`ItemCheck_StartActualUse`,
//!   `:51696`), never at the teleport.
//! * **Recall Potion 2350** (`Player.cs:43107-43140`). At `ItemTimeIsZero`:
//!   `ApplyItemTime` + `SoundID.Item3` + 10 Cyan dust (`:43109-43117`). At the
//!   HARDCODED `itemTime == 20` (not `useTime / 2`): the held `UseSound`
//!   (Item6) + 70 Cyan dust at the OLD position (`:43120-43124`),
//!   `RemoveAllGrapplingHooks` (`:43125`), the immune/immuneTime SAVE
//!   (`:43126-43127`), `Spawn(RecallFromItem)` (`:43128`), the RESTORE
//!   (`:43129-43130`), 70 Cyan dust at the NEW position (`:43131-43134`), then
//!   `item.stack--` (`:43135-43138`). The save/restore means the recall grants
//!   NO net i-frames — the mirror/recall difference the witnesses pin.
//! * **Potion of Return 4870** (`Player.cs:43141-43172`). The recall twin: same
//!   Item3 + 10 dust start, same `itemTime == 20` teleport with 70+70 Cyan
//!   dust and `stack--`, but the spawn leg is
//!   `DoPotionOfReturnTeleportationAndSetTheComebackPoint` (`:37498-37510`),
//!   which records the use position + the post-spawn home position and also
//!   saves/restores immune.
//!
//! All three call `Spawn(RecallFromItem)` — NOT `Player.Teleport` — so there
//! is no `Main.TeleportEffect` dust/sound for them; the arm's own bursts and
//! `UseSound` are the whole visual/audio. (`Spawn`'s camera transition is the
//! `Main.StartCameraTransitionForPlayerTeleport` lerp, `:37676`.)
//!
//! ## The spawn-point machinery (`Player.Spawn`, RecallFromItem subset)
//!
//! `spawn_recall` ports the `PlayerSpawnContext.RecallFromItem` path of
//! `Player.Spawn` (`Player.cs:37534-37710`): the bed re-validation
//! (`FindSpawn` + `CheckSpawn`, `:37570-37575`), the vanity stop (`:37556`),
//! the lava-budget refill (`:37589`), the dead-flag/immune reset
//! (`:37612-37615`), the position pick (bed `Spawn_SetPosition` `:37621-37626`
//! else `Spawn_SetPositionAtWorldSpawn` `:37633`), the wet/velocity/fall reset
//! (`:37635-37641`), `oldPosition = position + (0,-1)` (`:37647`,
//! `BlehOldPositionFixer` `:3235`), `SetTalkNPC(-1)` (`:37648`), the 60-tick
//! immunity + `immuneNoBlink` (`:37665-37672`) and the camera transition
//! (`:37676`). The `statLife <= 0` revive-life arm (`:37596-37611`) is carried
//! but never fires for a living recaller.
//!
//! The position helpers are verbatim: `Spawn_IsAreaValidSpawn`
//! (`:37937-37985`), `Spawn_ValidSpawnFloor` (`:37896-37912`),
//! `Spawn_DescendFromDefaultSpace` + `_Inner` (`:37818-37894`),
//! `Spawn_GetPositionAtSpawn` (`:37723-37804`), `Spawn_SetPosition`
//! (`:37987-37991`), `Spawn_SetPositionAtWorldSpawn` (`:37806-37816`) and
//! `Spawn_ForceClearArea` (`:37914-37935`).
//!
//! ## RNG
//!
//! Every `Main.rand` draw rides the `UpdatePlayers` stream (`Player.ItemCheck`
//! runs under the `Main.cs:17754` swap), in C# order: the mirror's per-tick
//! `Next(2)` gate draws EVEN when the dust culls (the gate is outside
//! `NewDust`); each spawned `Dust.NewDust` then rolls its own six default-block
//! draws (`Dust.cs:170-175`, `:190`) through `DustPool::new_dust`. The recall
//! bursts are 10 / 70 / 70 `NewDust` calls in order, each followed by the
//! `.velocity *= 0.5f` slot write (the sentinel slot included, exactly the
//! C#'s `Main.dust[6000]`). The witnesses mirror the cited draws on a CLONED
//! `UnifiedRandom` and compare the full state (`UnifiedRandom: PartialEq`).
//!
//! ## Scope / deviations (ANALYSIS "The recall / mirror teleport (S1-P-i38)")
//!
//! * Single-player: the `Main.netMode` arms (`SendData`, the client `73`
//!   request) have no port; `whoAmI == Main.myPlayer` always holds.
//! * `Spawn`'s unported state is dropped with a citation: the head/body/leg
//!   position + rotation resets (`:37582-37587`, the composite arms recompute
//!   per frame), `lavaOpacity` (`:37590`), `insideUnbreakableWalls` +
//!   `DoUnbreakableWallScan` (`:37591-37592`, derived per tick in
//!   `scene_scan.rs`), `DamageTracker.Reset` (`:37593`), the advanced/social
//!   shadows (`:37642-37646`, unported), `UpdateBiomesIfMovedEnoughForBlackFade`
//!   (`:37677`, the black fade is M6 cinematics), the `SpawningIntoWorld` /
//!   `ReviveFromDeath`-only tails (`:37687-37709`).
//! * The Potion of Return RETURN leg (`DoPotionOfReturnReturnToOriginalUsePosition`,
//!   `:37512-37523`, `Teleport` style 8) and the Shellphone Spawn 5359 arm
//!   (`:43066-43106`, `Shellphone_Spawn` `:56164-56181`, `Teleport` style 11)
//!   are the `Player.Teleport`/`TeleportEffect` system — a separate follow-up;
//!   this module ports the to-spawn leg the recall/mirror cluster uses. The
//!   comeback fields ARE recorded (the 4870 arm needs them).
//! * `Spawn_ForceClearArea`'s `WorldGen.KillTile(i, j)` rides the port's
//!   `break_tile` (the `CanKillTile` refusal + sounds + drops); the liquid
//!   clear's `SquareTileFrame` object drops drain through `spawn_item_drop`.

use glam::Vec2;
use tr_core::tile::FLAG_LIQUID_LAVA;
use tr_core::tiledata as td;
use tr_core::{edit, World, TILE_PX};

use crate::app::App;
use crate::player::{HEIGHT, WIDTH};
use crate::registry;
use crate::soundid::SOUND_STYLES;
use crate::world_sim::editing::{bed_spawn_valid, break_tile, check_bed_spawn, start_swing_extras};
use crate::world_sim::items::spawn_item_drop;

/// `ItemID.MagicMirror` 50 / `CellPhone` 3124 / `IceMirror` 3199 /
/// `Shellphone` 5358 (`ItemID.cs:1783`, `:7931`, `:8081`, `:12399`) — the
/// shared `Player.cs:42976` block.
pub(crate) const MIRROR_FAMILY: [u16; 4] = [50, 3124, 3199, 5358];
/// `ItemID.RecallPotion` 2350 (`ItemID.cs:6383`, `Player.cs:43107`).
pub(crate) const RECALL_POTION: u16 = 2350;
/// `ItemID.PotionOfReturn` 4870 (`ItemID.cs:11423`, `Player.cs:43141`).
pub(crate) const POTION_OF_RETURN: u16 = 4870;

/// `Color.Cyan` (XNA) — the recall/return dust colour (`Player.cs:43115`).
const CYAN: tr_core::particles::Color = tr_core::particles::Color::new(0, 255, 255, 255);

/// One 60 Hz tick in seconds — the fixed step the item timers pace against.
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The C# INTEGER tick comparisons out of the port's per-frame seconds timers
/// (`round(secs * 60)` at the 60 Hz step) — the `editing.rs::ticks_of`
/// convention (`Player.cs:4064` `ItemTimeIsZero => itemTime == 0`).
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// `SoundID`'s legacy style index by name (the generated `SOUND_STYLES` table,
/// `SoundID.cs`'s ctor order) — `Item3` for the recall start cue
/// (`Player.cs:43112`), the held `UseSound` for the teleport cue.
fn style_index(name: &str) -> u16 {
    SOUND_STYLES
        .iter()
        .position(|s| s.name == name)
        .expect("the SoundID style is in the generated table") as u16
}

/// Whether the held kind rides one of the spawn-recall `ItemCheck` arms.
pub(crate) fn is_recall_cluster(kind: u16) -> bool {
    MIRROR_FAMILY.contains(&kind) || kind == RECALL_POTION || kind == POTION_OF_RETURN
}

fn is_mirror_family(kind: u16) -> bool {
    MIRROR_FAMILY.contains(&kind)
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `Tile.nactive()` — active and not actuated-off (`Tile.cs`).
fn nactive(t: &tr_core::tile::Tile) -> bool {
    t.active() && !t.inactive()
}

// --- The spawn-position helpers (verbatim) ----------------------------------

/// `Player.Spawn_ValidSpawnFloor` (`Player.cs:37896-37912`): a solid
/// non-solid-top tile, a platform, or tile 380 — but never the pressure plate
/// 135 (`tile.active()`, the C#'s own active-not-nactive test here) and never
/// tile 379.
pub(crate) fn spawn_valid_spawn_floor(world: &World, tx: i32, ty: i32) -> bool {
    let Some(tile) = world.tile(tx, ty) else {
        return false;
    };
    // `:37903`: `tile.active() && tile.type == 135` (active, NOT nactive).
    if tile.active() && tile.kind == 135 {
        return false;
    }
    // `:37907`: `tile.nactive() && ((tileSolid && !tileSolidTop) ||
    // Platforms || type == 380)`.
    if nactive(tile)
        && ((td::TILE_SOLID.has(tile.kind) && !td::TILE_SOLID_TOP.has(tile.kind))
            || td::SET_PLATFORMS.has(tile.kind)
            || tile.kind == 380)
    {
        return tile.kind != 379;
    }
    false
}

/// `Player.Spawn_IsAreaValidSpawn` (`Player.cs:37937-37985`, the
/// `generatingSpawn: false` path): the 3×3 column above the floor cell free of
/// pressure plates, solid non-solid-top tiles, hurt tiles (the floor row's
/// tile below) and liquid.
pub(crate) fn spawn_is_area_valid_spawn(world: &World, floor_x: i32, floor_y: i32) -> bool {
    let good_world = world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let dont_starve = world.seeds.contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    for i in floor_x - 1..floor_x + 2 {
        for j in floor_y - 3..floor_y {
            // `:37943`: `!InWorld(i, j, 5) || tile == null` → continue.
            if !in_world(world, i, j, 5) {
                continue;
            }
            let Some(tile) = world.tile(i, j) else {
                continue;
            };
            if j == floor_y - 1 {
                // `:37948-37961`: the floor row's tile BELOW must not be a
                // pressure plate or a hurt tile (`Collision.CanTileHurt`,
                // `player == null` → the fireWalk arm never skips).
                if let Some(below) = world.tile(i, j + 1) {
                    if below.active() {
                        if below.kind == 135 {
                            return false;
                        }
                        if crate::touch_damage::can_tile_hurt(below.kind, good_world, dont_starve, false)
                        {
                            return false;
                        }
                    }
                }
            }
            // `:37963-37973`: `tile.nactive()` → 135 / solid-non-top refuse.
            if nactive(tile) {
                if tile.kind == 135 {
                    return false;
                }
                if td::TILE_SOLID.has(tile.kind) && !td::TILE_SOLID_TOP.has(tile.kind) {
                    return false;
                }
            }
            // `:37974-37977`: any liquid refuses.
            if tile.liquid > 0 {
                return false;
            }
            // `:37978`: the `generatingSpawn && dualDungeonsSeed` dungeon arm
            // is the worldgen-only path (false here) — dropped.
        }
    }
    true
}

/// `Player.Spawn_DescendFromDefaultSpace_Inner` (`Player.cs:37831-37894`):
/// drop down to 50 rows looking for a valid floor, with the C#'s `num2`
/// horizontal-shift accumulator and the single-block fallback side probes.
/// Returns `(point, validSpace)`.
fn spawn_descend_inner(
    world: &World,
    x: i32,
    y: i32,
    single_block_allowed: bool,
) -> ((i32, i32), bool) {
    let mut x = x;
    let mut y = y;
    for i in 0..50 {
        let mut flag = false;
        let mut flag2 = false;
        let mut flag3 = false;
        let num = y + i;
        let mut num2 = 0;
        for j in -1..=1 {
            let num3 = x + j + num2;
            // `:37844`: `!InWorld(num3, num, 5) || tile[num3,num] == null ||
            // tile[num3,num-1] == null` → continue.
            if !in_world(world, num3, num, 5) {
                continue;
            }
            let Some(tile) = world.tile(num3, num) else {
                continue;
            };
            if world.tile(num3, num - 1).is_none() {
                continue;
            }
            // `:37849`: `tile.nactive() && type == 135` → bail to (x, y).
            if nactive(tile) && tile.kind == 135 {
                return ((x, y), false);
            }
            if spawn_valid_spawn_floor(world, num3, num)
                && spawn_is_area_valid_spawn(world, num3, num)
            {
                match j {
                    -1 => flag = true,
                    1 => flag3 = true,
                    _ => flag2 = true,
                }
            }
        }
        // `:37870-37883`: the side probes when the centre column found nothing.
        if !flag2
            && (single_block_allowed | flag)
            && spawn_valid_spawn_floor(world, x - 2, num)
            && spawn_is_area_valid_spawn(world, x - 1, num)
        {
            flag = true;
            flag2 = true;
            flag3 = false;
            num2 -= 1;
        } else if !flag2
            && (single_block_allowed | flag3)
            && spawn_valid_spawn_floor(world, x + 2, num)
            && spawn_is_area_valid_spawn(world, x + 1, num)
        {
            flag = false;
            flag2 = true;
            flag3 = true;
            num2 += 1;
        }
        // `:37884-37890`.
        if flag2 && (single_block_allowed | flag | flag3) {
            x += num2;
            y += i;
            return ((x, y), true);
        }
    }
    ((x, y), false)
}

/// `Player.Spawn_DescendFromDefaultSpace` (`Player.cs:37818-37829`): the inner
/// drop, then the single-block-allowed retry when the first found no space.
pub(crate) fn spawn_descend_from_default_space(world: &World, x: i32, y: i32) -> ((i32, i32), bool) {
    let (point, valid) = spawn_descend_inner(world, x, y, false);
    if !valid {
        let (point2, valid2) = spawn_descend_inner(world, x, y, true);
        return (point2, valid2);
    }
    (point, valid)
}

/// `Player.Spawn_GetPositionAtSpawn` (`Player.cs:37723-37804`): when the spawn
/// cell is not a valid area, search up to 30 rows up (centre, left, right) then
/// 30 down, then descend. Returns `(floorX, floorY, moved)` — `moved` is the
/// C#'s `flag2` (a search happened), which drives `Spawn_ForceClearArea`.
pub(crate) fn spawn_get_position_at_spawn(
    world: &World,
    spawn_x: i32,
    spawn_y: i32,
) -> (i32, i32, bool) {
    let mut num = spawn_x;
    let mut num2 = spawn_y;
    if !spawn_is_area_valid_spawn(world, num, num2) {
        let mut flag2 = false;
        // `:37733-37758`: the up-scan (the C#'s `flag` is constant true).
        for i in 0..30 {
            if spawn_is_area_valid_spawn(world, num, num2 - i) {
                num2 -= i;
                flag2 = true;
                break;
            }
            if spawn_is_area_valid_spawn(world, num - 1, num2 - i) {
                num -= 1;
                num2 -= i;
                flag2 = true;
                break;
            }
            if spawn_is_area_valid_spawn(world, num + 1, num2 - i) {
                num += 1;
                num2 -= i;
                flag2 = true;
                break;
            }
        }
        // `:37762-37787`: the down-scan.
        if !flag2 {
            for j in 0..30 {
                if spawn_is_area_valid_spawn(world, num, num2 + j) {
                    num2 += j;
                    flag2 = true;
                    break;
                }
                if spawn_is_area_valid_spawn(world, num - 1, num2 + j) {
                    num -= 1;
                    num2 += j;
                    flag2 = true;
                    break;
                }
                if spawn_is_area_valid_spawn(world, num + 1, num2 + j) {
                    num += 1;
                    num2 += j;
                    flag2 = true;
                    break;
                }
            }
        }
        // `:37789-37797`: a found spot descends; the C# returns true either way.
        if flag2 {
            let (point, _valid) = spawn_descend_from_default_space(world, num, num2);
            return (point.0, point.1, true);
        }
        return (num, num2, true);
    }
    // `:37799-37803`: the original cell was valid — descend, return false.
    let (point, _valid) = spawn_descend_from_default_space(world, num, num2);
    (point.0, point.1, false)
}

/// `Player.Spawn_SetPosition` (`Player.cs:37987-37991`): the feet at the spawn
/// cell's top, centred — `floorX * 16 + 8 - width / 2`, `floorY * 16 - height`.
pub(crate) fn spawn_set_position(floor_x: i32, floor_y: i32, width: f32, height: f32) -> (f32, f32) {
    (
        floor_x as f32 * TILE_PX + 8.0 - width * 0.5,
        floor_y as f32 * TILE_PX - height,
    )
}

/// `Player.Spawn_ForceClearArea` (`Player.cs:37914-37935`): the 3×3 column
/// above the floor — kill solid non-solid-top tiles (`WorldGen.KillTile(i, j)`,
/// the port's `break_tile` with its drops/sounds) and clear any liquid
/// (`lava(false)`, `liquid = 0`, `SquareTileFrame`).
pub(crate) fn spawn_force_clear_area(app: &mut App, floor_x: i32, floor_y: i32) {
    for i in floor_x - 1..floor_x + 2 {
        for j in floor_y - 3..floor_y {
            if !app.world.in_bounds(i, j) {
                continue;
            }
            let Some(tile) = app.world.tile(i, j).copied() else {
                continue;
            };
            // `:37922-37925`: `nactive && tileSolid && !tileSolidTop` → KillTile.
            if nactive(&tile)
                && td::TILE_SOLID.has(tile.kind)
                && !td::TILE_SOLID_TOP.has(tile.kind)
            {
                break_tile(app, i, j);
            }
            // `:37926-37931`: liquid → lava(false), liquid = 0, SquareTileFrame.
            let still = app.world.tile(i, j).copied();
            if let Some(t) = still {
                if t.liquid > 0 {
                    let idx = app.world.index(i, j);
                    app.world.tiles[idx].flags &= !FLAG_LIQUID_LAVA;
                    app.world.tiles[idx].liquid = 0;
                    // W4-g83: the liquid-wipe wake's framing dispatch rides
                    // the UpdatePlayers stream (contract §2).
                    let drops = edit::square_tile_frame_default(
                        &mut app.world,
                        i,
                        j,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    for d in drops {
                        // The Check*'s own Item.NewItem (the sunflower's 63)
                        // or the object's style-0 drop (editing.rs:2413-2420).
                        let id = d
                            .item
                            .map(|item| item as u16)
                            .or_else(|| registry::tile_drop(d.tile_kind, 0));
                        if let Some(id) = id {
                            spawn_item_drop(&mut app.items, rng, id, 1, d.x, d.y);
                        }
                    }
                    app.queue_map_ring(i, j);
                    app.world_key = None;
                }
            }
        }
    }
}

/// `Player.Spawn_SetPositionAtWorldSpawn` (`Player.cs:37806-37816`): the world
/// spawn cell, the position pick (which may move it), then `ForceClearArea`
/// when it moved and the area is still invalid. Returns the pixel position.
fn spawn_set_position_at_world_spawn(app: &mut App) -> (f32, f32) {
    let (sx, sy) = (app.world.spawn_x, app.world.spawn_y);
    let (floor_x, floor_y, moved) = spawn_get_position_at_spawn(&app.world, sx, sy);
    let pos = spawn_set_position(floor_x, floor_y, WIDTH, HEIGHT);
    if let Some(p) = app.player.as_mut() {
        p.x = pos.0;
        p.y = pos.1;
    }
    // `:37812-37815`: `flag && !Spawn_IsAreaValidSpawn` → ForceClearArea.
    if moved && !spawn_is_area_valid_spawn(&app.world, floor_x, floor_y) {
        spawn_force_clear_area(app, floor_x, floor_y);
    }
    pos
}

// --- Player.Spawn (RecallFromItem) ------------------------------------------

/// `Player.Spawn(PlayerSpawnContext.RecallFromItem)` (`Player.cs:37534-37710`),
/// the single-player recall subset. The bed re-validation, the vanity stop,
/// the lava refill, the dead/immune reset, the position pick, the
/// wet/velocity/fall reset, the 60-tick immunity and the camera transition.
/// The caller (the recall/return arms) saves/restores `immune`/`immuneTime`
/// around this when the C# does (2350/4870); the mirror does not.
pub(crate) fn spawn_recall(app: &mut App) {
    let Some(mut p) = app.player.take() else {
        return;
    };
    // `:37556` StopVanityActions — SitUp + StopSleeping (the respawn precedent).
    p.sit_up();
    p.stop_sleeping();
    // `:37570-37575` FindSpawn + CheckSpawn: the bed spawn re-validates; a
    // broken bed / invalid room reverts to the world spawn (SpawnX = -1). The
    // failure text rides the chat like the C#'s Main.NewText (the revive path,
    // world_sim/player.rs:365-378, runs the same check).
    let mut bed: Option<(i32, i32)> = None;
    if let Some((bx, by)) = app.world.bed_spawn {
        if check_bed_spawn(&app.world, bx, by, &mut app.chat) {
            bed = Some((bx, by));
        } else {
            app.world.bed_spawn = None;
        }
    }
    // `:37589` lavaTime = lavaMax (lavaOpacity :37590 has no port field).
    p.lava_time = p.lava_max;
    // `:37596-37611` the statLife <= 0 revive-life arm (never fires for a
    // living recaller; carried for the dead-but-not-flagged edge).
    if p.hp <= 0 {
        let pool = p.max_hp;
        p.hp = (pool / 2).max(100).min(pool.max(100));
        p.breath = crate::player::BREATH_MAX;
    }
    // `:37612-37615` immune = true, dead = false, deadTime = 0, immuneTime = 0.
    p.immune = true;
    p.dead = false;
    p.dead_time = 0;
    p.immune_time = 0;
    // `:37619-37620` spectating = -1 (no port); the camera reads the pre-spawn
    // position (SpectatingCameraPosition, :4082-4093, spectating < 0 → position).
    let old_pos = Vec2::new(p.x, p.y);
    // `:37621-37634` the position pick: bed Spawn_SetPosition else world spawn.
    match bed {
        Some((bx, by)) => {
            let (px, py) = spawn_set_position(bx, by, WIDTH, p.height);
            p.x = px;
            p.y = py;
        }
        None => {
            // Spawn_SetPositionAtWorldSpawn writes p.x/p.y itself; re-borrow.
            app.player = Some(p);
            let _ = spawn_set_position_at_world_spawn(app);
            p = app.player.take().unwrap();
        }
    }
    // `:37635-37641` wet/wetCount/lavaWet false, netOffset zero, ResetFallDamage,
    // velocity zero.
    p.wet = false;
    p.lava_wet = false;
    p.honey_wet = false;
    p.shimmer_wet = false;
    p.reset_wet_count();
    p.vx = 0.0;
    p.vy = 0.0;
    p.on_ground = false;
    p.reset_jump_boost();
    // `:37712-37716` ResetFallDamage: fallStart = fallStart2 = (int)(Y / 16).
    p.reset_fall_damage();
    // `:37647` oldPosition = position + BlehOldPositionFixer (-UnitY, :3235).
    p.prev_x = p.x;
    p.prev_y = p.y - 1.0;
    // `:37648-37652` SetTalkNPC(-1) + npcChatCornerItem = 0.
    app.talk_npc = None;
    // `:37665-37672` the RecallFromItem immune arm: immuneTime = 60 (not the
    // ReviveFromDeath 180 / pvpDeath 300), immuneNoBlink when !hostile.
    p.immune_time = 60;
    if p.immune_time > 0 && !p.hostile {
        p.immune_no_blink = true;
    }
    // `:37676` StartCameraTransitionForPlayerTeleport(oldPos, 0.1, 0) — the
    // Spawn lerp (delay 0, NOT the Teleport style-1 potion-recall 10).
    let mut state = std::mem::take(&mut app.camera_state);
    crate::world_sim::camera::start_camera_transition_for_player_teleport(
        &mut state, old_pos, 0.1, 0,
    );
    app.camera_state = state;
    // `:37677` UpdateBiomesIfMovedEnoughForBlackFade — the black fade is M6
    // cinematics (dropped). The unported Spawn tails are cited in the module doc.
    app.player = Some(p);
}

/// `Player.DoPotionOfReturnTeleportationAndSetTheComebackPoint`
/// (`Player.cs:37498-37510`): the 4870 spawn leg — RemoveAllGrapplingHooks,
/// record the use position (Bottom), save immune, StopVanityActions(false),
/// Spawn(RecallFromItem), record the home position (post-spawn Bottom), restore
/// immune. The `SendData(13)` net arm has no port.
fn do_potion_of_return_teleport(app: &mut App) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    // `:37500` RemoveAllGrapplingHooks (the request the player pass drains).
    p.release_grappling_hooks = true;
    // `:37501` PotionOfReturnOriginalUsePosition = base.Bottom.
    let bottom = (p.x + WIDTH * 0.5, p.y + p.height);
    p.potion_of_return_original_use_position = Some(bottom);
    // `:37502-37503` save immune / immuneTime.
    let was_immune = p.immune;
    let was_immune_time = p.immune_time;
    // `:37504` StopVanityActions(multiplayerBroadcast: false).
    p.sit_up();
    p.stop_sleeping();
    // `:37505` Spawn(RecallFromItem).
    spawn_recall(app);
    // `:37506` PotionOfReturnHomePosition = base.Bottom (post-spawn).
    if let Some(p) = app.player.as_mut() {
        p.potion_of_return_home_position = Some((p.x + WIDTH * 0.5, p.y + p.height));
        // `:37508-37509` restore immune / immuneTime.
        p.immune = was_immune;
        p.immune_time = was_immune_time;
    }
}

// --- The ItemCheck arms ------------------------------------------------------

/// The held item's prefixed `useTime` in seconds (`Player.ApplyItemTime` →
/// `SetItemTime(sItem.useTime)`, `:4416-4419`, the INSTANCE value) — the
/// `editing.rs::shooter_use_time_secs` convention, falling back to the raw
/// kind then the whole animation.
fn use_time_secs(held: crate::item::ItemSlot) -> f32 {
    let kind = held.kind;
    crate::prefix::prefixed_stats(kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| {
            registry::get(kind)
                .map(|d| d.use_time as f32 * ONE_TICK_SECS)
                .filter(|t| *t > 0.0)
                .unwrap_or_else(|| crate::item::use_animation_secs_of(kind, held.prefix))
        })
}

/// One `Dust.NewDust` on the `UpdatePlayers` stream + the post-spawn
/// `.velocity *= mult` slot write (the sentinel included, the C#'s
/// `Main.dust[6000]`). `position`/`width`/`height` are the player box.
fn new_dust(
    app: &mut App,
    pos: Vec2,
    width: i32,
    height: i32,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: tr_core::particles::Color,
    scale: f32,
    vel_mult: Option<f32>,
) {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let idx = app.dust.new_dust(
        rng, &ctx, &app.world, pos, width, height, dust_type, speed_x, speed_y, alpha, color, scale,
    );
    if let Some(m) = vel_mult {
        app.dust.slot_mut(idx).velocity *= m;
    }
}

/// The mirror family's teleport leg (`Player.cs:42988-42997`): 70 dust at the
/// OLD position, RemoveAllGrapplingHooks, Spawn(RecallFromItem) (NO immune
/// save — the mirror keeps the 60-tick immunity), 70 dust at the NEW position.
fn mirror_teleport(app: &mut App) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let old = Vec2::new(p.x, p.y);
    let (vx, vy) = (p.vx, p.vy);
    let (w, h) = (WIDTH as i32, p.height as i32);
    // `:42988-42991`: 70 × NewDust(15, vel*0.5, 150, default, 1.5).
    for _ in 0..70 {
        new_dust(app, old, w, h, 15, vx * 0.5, vy * 0.5, 150, Default::default(), 1.5, None);
    }
    // `:42992` RemoveAllGrapplingHooks.
    if let Some(p) = app.player.as_mut() {
        p.release_grappling_hooks = true;
    }
    // `:42993` Spawn(RecallFromItem) — no immune save/restore.
    spawn_recall(app);
    // `:42994-42997`: 70 × NewDust(15, 0, 0, 150, default, 1.5) at the NEW pos.
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let new = Vec2::new(p.x, p.y);
    let h = p.height as i32;
    for _ in 0..70 {
        new_dust(app, new, w, h, 15, 0.0, 0.0, 150, Default::default(), 1.5, None);
    }
}

/// The recall/return teleport leg's shared dust + spawn + consume shape
/// (`Player.cs:43120-43138` / `:43154-43170`): the held UseSound (Item6) at the
/// OLD position, 70 Cyan dust (vel*0.2, post *0.5), the spawn leg (2350 saves
/// immune inline; 4870 routes through the comeback-point fn), 70 Cyan dust at
/// the NEW position (0 speed, post *0.5), then stack--.
fn recall_teleport(app: &mut App, held: crate::item::ItemSlot) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let old = Vec2::new(p.x, p.y);
    let (vx, vy) = (p.vx, p.vy);
    let (w, h) = (WIDTH as i32, p.height as i32);
    // `:43120` / `:43154`: PlaySound(HeldItem.UseSound, base.position).
    if let Some(idx) = registry::get(held.kind).and_then(|d| d.use_sound) {
        let pitch = registry::get(held.kind).map_or(0.0, |d| d.use_sound_pitch);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        app.audio.play_use_sound(idx, old.x, old.y, pitch, rng);
    }
    // `:43121-43124` / `:43155-43158`: 70 × NewDust(15, vel*0.2, 150, Cyan, 1.2),
    // each .velocity *= 0.5.
    for _ in 0..70 {
        new_dust(app, old, w, h, 15, vx * 0.2, vy * 0.2, 150, CYAN, 1.2, Some(0.5));
    }
    if held.kind == RECALL_POTION {
        // `:43125-43130`: RemoveAllGrapplingHooks, save immune/immuneTime,
        // Spawn(RecallFromItem), restore.
        if let Some(p) = app.player.as_mut() {
            p.release_grappling_hooks = true;
        }
        let (was_immune, was_time) = app
            .player
            .as_ref()
            .map(|p| (p.immune, p.immune_time))
            .unwrap_or((false, 0));
        spawn_recall(app);
        if let Some(p) = app.player.as_mut() {
            p.immune = was_immune;
            p.immune_time = was_time;
        }
    } else {
        // `:43159-43162`: the 4870 comeback-point spawn leg (it does the hooks,
        // the immune save/restore and the Spawn itself).
        do_potion_of_return_teleport(app);
    }
    // `:43131-43134` / `:43163-43166`: 70 × NewDust(15, 0, 0, 150, Cyan, 1.2) at
    // the NEW position, each .velocity *= 0.5.
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let new = Vec2::new(p.x, p.y);
    let h = p.height as i32;
    for _ in 0..70 {
        new_dust(app, new, w, h, 15, 0.0, 0.0, 150, CYAN, 1.2, Some(0.5));
    }
    // `:43135-43138` / `:43167-43170`: item.stack-- (the port's count).
    let slot = &mut app.inventory[app.selected];
    slot.count = slot.count.saturating_sub(1);
    if slot.count == 0 {
        slot.kind = 0;
    }
}

/// The per-tick arm body (`Player.cs:42976-42999` / `:43107-43140` /
/// `:43141-43172`), run while `itemAnimation > 0`. The mirror rolls its
/// `Next(2)` dust gate EVERY tick (even when the dust culls); the recall/return
/// have no per-tick gate. `ItemTimeIsZero` → ApplyItemTime (+ the recall Item3
/// + 10 dust); the teleport fires at `useTime / 2` (mirror) or the hardcoded
/// `20` (recall/return).
fn run_recall_arm(app: &mut App, held: crate::item::ItemSlot) {
    if ticks_of(app.swing) <= 0 {
        return;
    }
    let kind = held.kind;
    let item_time_ticks = ticks_of(app.item_time);
    let use_secs = use_time_secs(held);
    let use_ticks = ticks_of(use_secs);
    if is_mirror_family(kind) {
        // `:42978-42981`: the per-tick Next(2) gate → one type-15 dust
        // (0 speed, 150 alpha, default colour, 1.1 scale). The gate draws even
        // when NewDust culls (it is outside the allocator).
        let gate = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            rng.next_max(2)
        };
        if gate == 0 {
            let Some(p) = app.player.as_ref() else {
                return;
            };
            let pos = Vec2::new(p.x, p.y);
            let (w, h) = (WIDTH as i32, p.height as i32);
            new_dust(app, pos, w, h, 15, 0.0, 0.0, 150, Default::default(), 1.1, None);
        }
        if item_time_ticks == 0 {
            // `:42982-42985` ApplyItemTime.
            app.item_time = use_secs;
        } else if item_time_ticks == use_ticks / 2 {
            // `:42986-42998` the recall.
            mirror_teleport(app);
        }
    } else if kind == RECALL_POTION || kind == POTION_OF_RETURN {
        if item_time_ticks == 0 {
            // `:43109-43117` / `:43143-43151`: ApplyItemTime + Item3 + 10 dust.
            app.item_time = use_secs;
            let Some(p) = app.player.as_ref() else {
                return;
            };
            let pos = Vec2::new(p.x, p.y);
            let (vx, vy) = (p.vx, p.vy);
            let (w, h) = (WIDTH as i32, p.height as i32);
            // `:43112` / `:43146` PlaySound(SoundID.Item3, base.position).
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            app.audio.play_use_sound(style_index("Item3"), pos.x, pos.y, 0.0, rng);
            // `:43113-43116` / `:43147-43150`: 10 × NewDust(15, vel*0.2, 150,
            // Cyan, 1.2), each .velocity *= 0.5.
            for _ in 0..10 {
                new_dust(app, pos, w, h, 15, vx * 0.2, vy * 0.2, 150, CYAN, 1.2, Some(0.5));
            }
        } else if item_time_ticks == 20 {
            // `:43118-43139` / `:43152-43171`: the HARDCODED 20-tick teleport.
            recall_teleport(app, held);
        }
    }
}

/// The recall/mirror/return `ItemCheck` machine, called once per tick from
/// `update_editing` (after the swing/item-time decay, before the cursor gates —
/// the C# arms run in `ItemCheck_OwnerOnlyCode` regardless of the cursor, the
/// boss-spawner precedent). `press_edge` is the caller's computed
/// `prev_mine_released && !prev_mine_down` (the port overwrites `mine_prev_down`
/// to the current frame at the top of `update_editing`, so the true edge must
/// come in — the `throw_ready` convention, `editing.rs:606`). Returns true when
/// it STARTED a use this tick (the caller consumes the tick like the
/// boss-spawner arms).
///
/// The start gate is the C#'s `controlUseItem && releaseUseItem &&
/// itemAnimation == 0 && useStyle != 0` press edge (`Player.cs:42733`); these
/// kinds are `autoReuse: false`. The per-tick arm runs whenever the swing is
/// live, even with the button released (the C# gate is `itemAnimation > 0`
/// only). `CCed` zeroes the animation (`:42598-42603`).
pub(crate) fn recall_item_check(app: &mut App, press_edge: bool) -> bool {
    let held = app.inventory[app.selected];
    if held.count == 0 || !is_recall_cluster(held.kind) {
        return false;
    }
    // `:42598-42603` CCed → itemAnimation = itemAnimationMax = 0, return. The
    // dead player runs UpdateDead instead of ItemCheck (Player.Update order).
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.item_time = 0.0;
        return false;
    }
    if ticks_of(app.swing) > 0 {
        run_recall_arm(app, held);
        return false;
    }
    // The press edge: button down now, was up last tick.
    if app.mine_down && !app.inventory_open && press_edge {
        // `ItemCheck_StartActualUse` → ApplyItemAnimation (`:51680`): the swing
        // lasts useAnimation; the use-start sound (mirror Item6; the recall's
        // SkipsInitialUseSound gate keeps 2350/4870 silent here).
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        // The C# start tick sees itemTime == 0 (StartActualUse leaves it; the
        // :42836 decrement only runs when > 0) → the arm's ItemTimeIsZero leg
        // fires THIS tick (ApplyItemTime, + the recall Item3/10 dust).
        app.item_time = 0.0;
        run_recall_arm(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::UnifiedRandom;

    /// A 200×200 dirt-ground world (floor at y=120), the world spawn on it.
    fn ground_world() -> World {
        let mut w = World::new("recall".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 100;
        w.spawn_y = 120;
        w.compute_column_surface();
        w
    }

    /// A headless rig: the player parked away from spawn, the camera on the
    /// player (so the dust cull rect covers it), the audio MUTED (the
    /// sound_style_random draw is audio-only; muting keeps the case-2 legacy
    /// draws off the UpdatePlayers stream so the RNG witnesses pin the dust
    /// draws alone — S1-E-21 pins the sound draws separately).
    fn rig(kind: u16, count: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = ground_world();
        let mut p = Player::spawn(&app.world);
        p.x = 60.0 * TILE_PX;
        p.y = 100.0 * TILE_PX;
        app.player = Some(p);
        app.camera = Vec2::new(60.0 * TILE_PX, 100.0 * TILE_PX);
        app.mouse_pos = Vec2::new(640.0, 360.0);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
        app.gfx_quality.max_dust_to_draw = 6000;
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
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
        // A clean press edge: released last tick, down now.
        app.mine_prev_down = false;
        app.mine_released = true;
        app.mine_down = true;
    }

    fn tick(app: &mut App) {
        // The update_editing timer decay the machine reads (swing/item_time),
        // then the machine itself — the same order update_editing runs. The
        // button stays held but no fresh press edge mid-swing.
        app.swing = (app.swing - ONE_TICK_SECS).max(0.0);
        app.item_time = (app.item_time - ONE_TICK_SECS).max(0.0);
        recall_item_check(app, false);
    }

    // ---- spawn helpers -----------------------------------------------------

    #[test]
    fn the_spawn_floor_accepts_solid_and_platforms_but_not_plates_or_379() {
        // `Spawn_ValidSpawnFloor` (`Player.cs:37896-37912`).
        let mut w = ground_world();
        assert!(spawn_valid_spawn_floor(&w, 100, 120), "dirt is a solid floor");
        // A platform (19) is a valid floor.
        let i = w.index(50, 100);
        w.tiles[i] = Tile { kind: 19, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(spawn_valid_spawn_floor(&w, 50, 100), "platforms pass");
        // The pressure plate 135 refuses (the active() test).
        let i = w.index(60, 100);
        w.tiles[i] = Tile { kind: 135, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!spawn_valid_spawn_floor(&w, 60, 100), "135 refuses");
        // Tile 379 refuses even though solid.
        let i = w.index(70, 100);
        w.tiles[i] = Tile { kind: 379, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!spawn_valid_spawn_floor(&w, 70, 100), "379 refuses");
        // Air is not a floor.
        assert!(!spawn_valid_spawn_floor(&w, 100, 100), "air refuses");
    }

    #[test]
    fn the_spawn_area_refuses_solid_liquid_and_plates_above() {
        // `Spawn_IsAreaValidSpawn` (`Player.cs:37937-37985`): the 3×3 above the
        // floor cell (100, 120) is air on the ground world → valid.
        let mut w = ground_world();
        assert!(spawn_is_area_valid_spawn(&w, 100, 120));
        // A solid block in the column refuses.
        let i = w.index(100, 118);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!spawn_is_area_valid_spawn(&w, 100, 120), "solid above refuses");
        // Clear it, add liquid → refuses.
        w.tiles[i] = Tile::default();
        let i = w.index(101, 119);
        w.tiles[i].liquid = 100;
        assert!(!spawn_is_area_valid_spawn(&w, 100, 120), "liquid refuses");
    }

    #[test]
    fn the_spawn_position_descends_to_the_floor() {
        // `Spawn_GetPositionAtSpawn` + `Spawn_DescendFromDefaultSpace`
        // (`Player.cs:37723-37804`, `:37818-37894`): a spawn cell in the air
        // above the ground descends to the floor row (120), no move flag.
        let w = ground_world();
        let (fx, fy, moved) = spawn_get_position_at_spawn(&w, 100, 100);
        assert_eq!((fx, fy), (100, 120), "descends to the dirt floor");
        assert!(!moved, "the original cell was a valid area (air above floor)");
        // Spawn_SetPosition centres the feet on the cell top.
        let (px, py) = spawn_set_position(fx, fy, WIDTH, HEIGHT);
        assert_eq!(px, 100.0 * TILE_PX + 8.0 - WIDTH * 0.5);
        assert_eq!(py, 120.0 * TILE_PX - HEIGHT);
    }

    // ---- spawn_recall ------------------------------------------------------

    #[test]
    fn the_recall_spawn_lands_at_the_world_spawn_with_60_tick_immunity() {
        // `Spawn(RecallFromItem)` (`Player.cs:37534-37710`): no bed → the world
        // spawn; immune 60 (NOT the revive 180); velocity zero; fall reset; a
        // living recaller is NOT healed.
        let mut app = rig(RECALL_POTION, 5);
        {
            let p = app.player.as_mut().unwrap();
            p.hp = 40;
            p.max_hp = 200;
            p.vx = 3.0;
            p.vy = -4.0;
            p.immune = false;
            p.immune_time = 0;
        }
        spawn_recall(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.x, 100.0 * TILE_PX + 8.0 - WIDTH * 0.5, "world spawn X");
        assert_eq!(p.y, 120.0 * TILE_PX - HEIGHT, "world spawn feet");
        assert_eq!(p.hp, 40, "a living recaller keeps its life (no revive heal)");
        assert_eq!(p.immune_time, 60, "the RecallFromItem 60-tick arm");
        assert!(p.immune, "Spawn sets immune");
        assert!(p.immune_no_blink, "!hostile → immuneNoBlink");
        assert_eq!((p.vx, p.vy), (0.0, 0.0), "velocity zeroed");
        assert_eq!(p.fall_start2, (p.y / TILE_PX) as i32, "ResetFallDamage");
        assert_eq!(p.prev_y, p.y - 1.0, "oldPosition = position + (0,-1)");
        assert!(!p.wet, "wet cleared");
    }

    #[test]
    fn the_recall_spawn_prefers_a_valid_bed() {
        // `Spawn`'s bed path (`Player.cs:37621-37626`): a valid bed spawn
        // overrides the world spawn (Spawn_SetPosition, no descend). The room
        // is the furnished-room fixture (`build_furnished_room(60, 100, 12,
        // 9)`); its furniture row (107) carries chair/table/torch/door at
        // columns 61..64. Gate V3: the bed goes at column 67 — clear of the
        // door (10, solid-non-top) at 64, which would otherwise sit inside
        // the bed's 3x3 obstruction box (`CheckSpawn`, `Player.cs:54556-
        // 54569`) and refuse the spawn. The spawn cell (67, 108) is the floor
        // row; the bed tile above is (67, 107).
        let mut app = rig(RECALL_POTION, 5);
        crate::test_world_helpers::build_furnished_room(&mut app.world, 60, 100, 12, 9);
        let i = app.world.index(67, 107);
        app.world.tiles[i] = Tile { kind: 79, frame_x: 0, frame_y: 0, ..Default::default() };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        app.world.bed_spawn = Some((67, 108));
        assert!(
            bed_spawn_valid(&app.world, 67, 108).is_ok(),
            "the bed validates through CheckSpawn's room check"
        );
        spawn_recall(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.x, 67.0 * TILE_PX + 8.0 - WIDTH * 0.5, "bed spawn X");
        assert_eq!(p.y, 108.0 * TILE_PX - HEIGHT, "bed spawn feet");
    }

    // ---- the mirror arm ----------------------------------------------------

    #[test]
    fn the_mirror_teleports_at_half_use_time_without_consuming() {
        // `Player.cs:42976-42999`: the mirror fires at itemTime == useTime/2
        // (90/2 = 45), keeps the stack, and keeps the 60-tick spawn immunity
        // (NO immune save/restore — the mirror/recall difference).
        let mut app = rig(50, 1);
        press(&mut app);
        recall_item_check(&mut app, true); // start tick: ApplyItemTime(90)
        assert!(app.swing > 0.0, "the swing started");
        assert_eq!(ticks_of(app.item_time), 90, "ApplyItemTime set useTime");
        // Tick to the teleport (itemTime 90 → 45 takes 45 ticks).
        for _ in 0..45 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.x, 100.0 * TILE_PX + 8.0 - WIDTH * 0.5, "teleported to spawn");
        assert_eq!(p.immune_time, 60, "the mirror KEEPS the spawn immunity");
        assert_eq!(app.inventory[app.selected].count, 1, "the mirror is not consumed");
        assert_eq!(app.inventory[app.selected].kind, 50, "the mirror stays");
    }

    #[test]
    fn the_ice_mirror_rides_the_same_arm() {
        // `Player.cs:42976`: 3199 is in the shared block — identical behaviour.
        let mut app = rig(3199, 1);
        press(&mut app);
        recall_item_check(&mut app, true);
        for _ in 0..45 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.x, 100.0 * TILE_PX + 8.0 - WIDTH * 0.5, "ice mirror teleports");
        assert_eq!(app.inventory[app.selected].count, 1, "not consumed");
        assert_eq!(p.immune_time, 60, "keeps the immunity");
    }

    // ---- the recall arm ----------------------------------------------------

    #[test]
    fn the_recall_teleports_at_twenty_ticks_consumes_and_restores_immunity() {
        // `Player.cs:43107-43140`: the recall fires at the HARDCODED itemTime
        // == 20 (not useTime/2 = 15), consumes one, and RESTORES the pre-spawn
        // immune/immuneTime (no net i-frames).
        let mut app = rig(RECALL_POTION, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.immune = true;
            p.immune_time = 12; // a pre-existing hurt cooldown
        }
        press(&mut app);
        recall_item_check(&mut app, true); // start: ApplyItemTime(30) + Item3 + 10 dust
        assert_eq!(ticks_of(app.item_time), 30, "ApplyItemTime set useTime 30");
        // itemTime 30 → 20 takes 10 ticks.
        for _ in 0..10 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.x, 100.0 * TILE_PX + 8.0 - WIDTH * 0.5, "teleported to spawn");
        assert_eq!(p.immune_time, 12, "the recall RESTORES the saved immuneTime");
        assert!(p.immune, "the saved immune flag restored");
        assert_eq!(app.inventory[app.selected].count, 2, "one potion consumed");
    }

    #[test]
    fn the_recall_does_not_fire_at_use_time_over_two() {
        // The hardcoded 20 vs useTime/2 = 15: at tick 15 (itemTime 15) the
        // recall has NOT teleported yet — it waits for itemTime 20 (tick 10).
        // This pins the mirror/recall timing difference: the recall is FASTER
        // (10 ticks) than half its useTime (15).
        let mut app = rig(RECALL_POTION, 3);
        let start_x = app.player.as_ref().unwrap().x;
        press(&mut app);
        recall_item_check(&mut app, true);
        for _ in 0..9 {
            tick(&mut app);
        }
        assert_eq!(app.player.as_ref().unwrap().x, start_x, "not yet at tick 9");
        tick(&mut app); // tick 10 → itemTime 20
        assert_ne!(app.player.as_ref().unwrap().x, start_x, "fires at itemTime 20");
    }

    // ---- the return arm ----------------------------------------------------

    #[test]
    fn the_potion_of_return_records_the_comeback_point_and_consumes() {
        // `Player.cs:43141-43172` + `:37498-37510`: the 4870 arm teleports at
        // itemTime 20, records the use position + the post-spawn home position,
        // restores immune, and consumes one.
        let mut app = rig(POTION_OF_RETURN, 2);
        let use_bottom = {
            let p = app.player.as_ref().unwrap();
            (p.x + WIDTH * 0.5, p.y + p.height)
        };
        {
            let p = app.player.as_mut().unwrap();
            p.immune = false;
            p.immune_time = 7;
        }
        press(&mut app);
        recall_item_check(&mut app, true);
        for _ in 0..10 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.x, 100.0 * TILE_PX + 8.0 - WIDTH * 0.5, "teleported to spawn");
        assert_eq!(
            p.potion_of_return_original_use_position,
            Some(use_bottom),
            "the use position recorded"
        );
        assert_eq!(
            p.potion_of_return_home_position,
            Some((p.x + WIDTH * 0.5, p.y + p.height)),
            "the post-spawn home position recorded"
        );
        assert_eq!(p.immune_time, 7, "immune restored");
        assert_eq!(app.inventory[app.selected].count, 1, "one consumed");
    }

    // ---- RNG witnesses (cloned UnifiedRandom, full-state compare) ----------

    /// Replay the mirror arm's cited `UpdatePlayers` draws on a clone: the
    /// per-tick `Next(2)` gate (every tick the swing runs) + the six
    /// `Dust.NewDust` default-block draws per spawned dust (`Dust.cs:170-175`,
    /// `:190`: Next(w-4), Next(h-4), Next(-20,21)×2, Next(3), Next(-20,21)),
    /// then the teleport's 70+70 burst draws. The pool is empty and the cull
    /// rect covers the player, so every NewDust spawns (no pressure-band draws
    /// below maxDustToDraw*0.5). Type 15 hits no tweak/ChildSafety arm.
    fn mirror_expected_draws(rng: &mut UnifiedRandom, gate_ticks: usize, w: i32, h: i32) {
        // The per-tick gates BEFORE the teleport tick (the start tick's gate
        // included). Each gate Next(2); on 0 the six NewDust draws.
        for _ in 0..gate_ticks {
            if rng.next_max(2) == 0 {
                new_dust_six(rng, w, h);
            }
        }
        // The teleport tick: its own gate, then 70 + 70 burst dusts (no gate
        // draw skipped — the gate runs every tick including the teleport one).
        if rng.next_max(2) == 0 {
            new_dust_six(rng, w, h);
        }
        for _ in 0..140 {
            new_dust_six(rng, w, h);
        }
    }

    /// The six `Dust.NewDust` default-block draws (`Dust.cs:170-175`, `:190`),
    /// in order, for a spawned type-15 dust (no pressure band, no ChildSafety,
    /// no per-type tweak).
    fn new_dust_six(rng: &mut UnifiedRandom, w: i32, h: i32) {
        let nw = w.max(5);
        let nh = h.max(5);
        rng.next_max(nw - 4); // position.X
        rng.next_max(nh - 4); // position.Y
        rng.next_range(-20, 21); // velocity.X
        rng.next_range(-20, 21); // velocity.Y
        rng.next_max(3); // frame.Y
        rng.next_range(-20, 21); // scale
    }

    #[test]
    fn the_mirror_arm_draws_ride_the_update_players_stream_in_order() {
        // The mirror's per-tick Next(2) gate + the NewDust internals + the
        // 70+70 teleport bursts all ride UpdatePlayers in C# order. Mirror the
        // cited draws on a CLONE and compare the full stream state. Audio is
        // muted (rig) so the use-start Item6 draws nothing on this stream.
        let mut app = rig(50, 1);
        let seed = 12345;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut oracle = UnifiedRandom::new(seed);
        let (w, h) = (WIDTH as i32, HEIGHT as i32);
        press(&mut app);
        recall_item_check(&mut app, true); // start tick gate
        // The start tick + 44 more gates before the teleport tick (45 total
        // pre-teleport gates), then the teleport tick's gate + 140 bursts.
        for _ in 0..44 {
            tick(&mut app);
        }
        tick(&mut app); // the teleport tick
        mirror_expected_draws(&mut oracle, 45, w, h);
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            &oracle,
            "the mirror arm's draws match the cited C# order on the clone"
        );
    }

    #[test]
    fn the_recall_bursts_draw_sixty_per_dust_in_order() {
        // The recall: 10 start dusts + 70 + 70 teleport dusts = 150 NewDust
        // calls, each six draws, on UpdatePlayers (audio muted → the Item3 /
        // Item6 cues draw nothing here). No per-tick gate (the recall has none).
        let mut app = rig(RECALL_POTION, 3);
        let seed = 987;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut oracle = UnifiedRandom::new(seed);
        let (w, h) = (WIDTH as i32, HEIGHT as i32);
        press(&mut app);
        recall_item_check(&mut app, true); // start: 10 dusts
        for _ in 0..10 {
            tick(&mut app); // tick 10: teleport, 70 + 70 dusts
        }
        for _ in 0..150 {
            new_dust_six(&mut oracle, w, h);
        }
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            &oracle,
            "the recall's 150 dusts draw 6 each in C# order"
        );
    }
}
