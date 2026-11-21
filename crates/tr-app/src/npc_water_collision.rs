//! W4-e9 — the NPC water-collision splash layer: the wiring half of
//! `npc.rs::water_collision_splash` (the pure entry/exit decision, sites
//! 35-42). `NPC.UpdateCollision` calls `Collision_WaterCollision(
//! Collision_LavaCollision())` (`NPC.cs:92526`) right after `AI()`
//! (`:90492`, the `!noTileCollide` branch — the raw-integration kinds
//! take the `:90494-90503` else and never reach this layer). The layer
//! owns the state writes the decision's doc assigns the caller:
//!
//! * the `Collision.WetCollision` / `LavaCollision` probes
//!   (`Collision.cs:1572-1651`/`:1653-1684`) — the centred min(10,w)×
//!   min(h/2,h) box, the `(256-liquid)/32*2` surface inset, the sloped
//!   active tile's above-liquid arm;
//! * the never-wet kinds' `wetCount = 0` + `lava = false` force
//!   (`NPC.cs:93055-93060`) and the aiStyle-116 `wetCount = 10` pin
//!   (`:93077-93080`);
//! * the sticky `honeyWet`/`shimmerWet` sets (`:93064-93070`) and the
//!   dry-tick clear of all three (`UpdateCollision`, `:92527-92532`);
//! * the entry/exit edge decision → the case-19 play queue
//!   (`wants_sound`, `NpcSoundCue::Splash`), the `wetCount = 10` arming
//!   even on a refused play, the exit's `velocity.X *= 0.5` halving
//!   (`:93179`), and the `base.wet` write (`:93175`/`:93180`);
//! * the `wetCount--` decay (`:92533-92535`).
//!
//! Deviations (documented, the decision fn's header carries the rest):
//! the 30×/10× dust bursts are the dust-layer skip; the shimmer entry's
//! `TryAddingRepeatedBuff(353, 100)` (`:93073`) and
//! `TryRemovingWaterPerishableEffects` (`:93083`) are buff-layer seams;
//! the lava entry's OnFire24 burn + 50-damage strike
//! (`Collision_LavaCollision`, `:93325-93337`) is the damage-layer seam
//! (the probe's `lavaWet = true` write IS ported); the type-620 exit
//! leap (`:93181-93184`) is unported-kind movement. The `wet_prev` latch
//! carries the C#'s `base.wet` across the pass — the port's flyer/
//! swimmer wrappers overwrite `self.wet` pre-AI for the machine reads
//! (the documented `is_wet` stand-in), so the edge keys on the latch.

use tr_core::{TILE_PX, World};

use crate::npc::{Npc, NpcSoundCue, NpcSoundRequest};

/// `Collision.WetCollision` (`Collision.cs:1572-1651`): returns
/// `(wet, honey, shimmer)` — the honey/shimmer flags are the C#'s
/// `Collision.honey`/`Collision.shimmer` statics (reset each call,
/// `:1574-1575`), set only by the FIRST overlapping cell (the early
/// return `:1623`/`:1645`).
pub(crate) fn wet_collision_probe(
    world: &World,
    x: f32,
    y: f32,
    w: i32,
    h: i32,
) -> (bool, bool, bool) {
    // `:1576-1587` — the centred box: num = min(10, width),
    // num2 = min(height/2, height) (INTEGER halves), anchored at the
    // centre minus half-extents (integer halves again).
    let num = 10.min(w);
    let num2 = (h / 2).min(h);
    let bx = x + (w / 2) as f32 - (num / 2) as f32;
    let by = y + (h / 2) as f32 - (num2 / 2) as f32;
    // `:1588-1595` — the scan bounds: (int) truncation, -1/+2 margins,
    // the y clamp at maxTilesY - 40.
    let x0 = ((x / TILE_PX) as i32 - 1).clamp(0, world.width as i32 - 1);
    let x1 = (((x + w as f32) / TILE_PX) as i32 + 2).clamp(0, world.width as i32 - 1);
    let y0 = ((y / TILE_PX) as i32 - 1).clamp(0, world.height as i32 - 40);
    let y1 = (((y + h as f32) / TILE_PX) as i32 + 2).clamp(0, world.height as i32 - 40);
    for i in x0..x1 {
        for j in y0..y1 {
            let Some(t) = world.tile(i, j) else { continue };
            if t.liquid > 0 {
                // `:1607-1613` — the liquid surface inset: the cell's
                // top drops by (256-liquid)/32*2, its height shrinks by
                // the truncated same.
                let num5 = (256.0 - t.liquid as f32) / 32.0;
                let cy = j as f32 * TILE_PX + num5 * 2.0;
                let ch = 16 - (num5 * 2.0) as i32;
                let cx = i as f32 * TILE_PX;
                if bx + num as f32 > cx
                    && bx < cx + TILE_PX
                    && by + num2 as f32 > cy
                    && by < cy + ch as f32
                {
                    use tr_core::tile::LiquidKind;
                    let (honey, shimmer) = match t.liquid_kind() {
                        LiquidKind::Honey => (true, false),
                        LiquidKind::Shimmer => (false, true),
                        _ => (false, false),
                    };
                    return (true, honey, shimmer);
                }
            } else if t.active() && t.slope() != 0 && j > 0 {
                // `:1626-1647` — the sloped active tile with liquid
                // ABOVE: the full-height cell overlaps wet, the above
                // cell's kind picks the flag (else-if, `:1637-1644`).
                let Some(above) = world.tile(i, j - 1) else { continue };
                if above.liquid <= 0 {
                    continue;
                }
                let cx = i as f32 * TILE_PX;
                let cy = j as f32 * TILE_PX;
                if bx + num as f32 > cx
                    && bx < cx + TILE_PX
                    && by + num2 as f32 > cy
                    && by < cy + TILE_PX
                {
                    use tr_core::tile::LiquidKind;
                    let (honey, shimmer) = match above.liquid_kind() {
                        LiquidKind::Honey => (true, false),
                        LiquidKind::Shimmer => (false, true),
                        _ => (false, false),
                    };
                    return (true, honey, shimmer);
                }
            }
        }
    }
    (false, false, false)
}

/// `Collision.LavaCollision` (`Collision.cs:1653-1684`): the FULL box
/// (not the centred one) against lava cells only, same surface inset.
pub(crate) fn lava_collision_probe(world: &World, x: f32, y: f32, w: i32, h: i32) -> bool {
    let x0 = ((x / TILE_PX) as i32 - 1).clamp(0, world.width as i32 - 1);
    let x1 = (((x + w as f32) / TILE_PX) as i32 + 2).clamp(0, world.width as i32 - 1);
    let y0 = ((y / TILE_PX) as i32 - 1).clamp(0, world.height as i32 - 40);
    let y1 = (((y + h as f32) / TILE_PX) as i32 + 2).clamp(0, world.height as i32 - 40);
    for i in x0..x1 {
        for j in y0..y1 {
            let Some(t) = world.tile(i, j) else { continue };
            if t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava {
                let num3 = (256.0 - t.liquid as f32) / 32.0;
                let cy = j as f32 * TILE_PX + num3 * 2.0;
                let ch = 16 - (num3 * 2.0) as i32;
                let cx = i as f32 * TILE_PX;
                if x + w as f32 > cx && x < cx + TILE_PX && y + h as f32 > cy && y < cy + ch as f32
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `Collision_WaterCollision`'s never-wet gate (`NPC.cs:93055`): the
/// Bunfish's idle phase (690 `ai[0] == 0`), the Blazing Wheel (72),
/// aiStyle 21/67, the two fish (376/579), the Sand Elemental (541) and
/// the town NPC's sleep pose (aiStyle 7 `ai[0] == 25`).
pub(crate) fn never_wet(npc_type: i32, ai_style: i32, ai0: f32) -> bool {
    (npc_type == 690 && ai0 == 0.0)
        || npc_type == 72
        || ai_style == 21
        || ai_style == 67
        || npc_type == 376
        || npc_type == 579
        || npc_type == 541
        || (ai_style == 7 && ai0 == 25.0)
}

/// W4-e9 — the kinds whose port machine integrates TILE collision (the
/// C#'s `!noTileCollide` branch, `NPC.cs:90490-90492`): the aiStyle-1/3/7
/// walker families, the archers/shooters, the DD2 improved walkers and
/// the King Slime. The raw-integration flyers/bosses (the eyes, the
/// bats/fish, the worms, the casters, the pillars, the saucer family, …)
/// take the `:90494-90503` else and never reach `Collision_WaterCollision`
/// — the predicate keeps the layer off them (a splash on a noTileCollide
/// NPC would be an invented play). The BlazingWheel (aiStyle 21) rides
/// the layer but the never-wet force refuses every edge (`:93055`).
pub(crate) fn water_collision_kind(kind: crate::npc::NpcKind) -> bool {
    use crate::npc::NpcKind::*;
    if kind.is_town_resident() || kind == DD2Bartender {
        return true; // aiStyle 7 — the sleep pose's never-wet term rides inside
    }
    if crate::ambient_sound::fighter_walker_kind(kind) {
        return true; // the zombie/skeleton/goblin/pirate/scarecrow walkers
    }
    matches!(
        kind,
        Slime
            | MotherSlime
            | LavaSlime
            | DungeonSlime
            | CorruptSlime
            | IlluminantSlime
            | IceSlime
            | Crimslime
            | SpikedIceSlime
            | UmbrellaSlime
            | KingSlime
            | GoblinArcher
            | PirateArcher
            | PirateCorsair
            | PirateCaptain
            | GrayGrunt
            | MartianEngineer
            | GigaZapper
            | MartianWalker
            | MartianOfficer
            | BrainScrambler
            | RayGunner
            | SnowmanGangsta
            | SnowBalla
            | MysterySnowman
            | BlazingWheel
            | DD2GoblinT1
            | DD2GoblinT2
            | DD2GoblinT3
            | DD2GoblinBomberT1
            | DD2GoblinBomberT2
            | DD2GoblinBomberT3
            | DD2JavelinT1
            | DD2JavelinT2
            | DD2JavelinT3
            | DD2SkeletonT1
            | DD2SkeletonT3
            // The DD2 support walkers (aiStyle 107's improved-walker arms,
            // `NPC.cs:63029-63412`): the wither beasts, the drakins, the
            // kobold walkers and the ogres — all `walker_step`-integrated.
            | DD2WitherBeast
            | DD2WitherBeastT3
            | DD2DrakinT2
            | DD2DrakinT3
            | DD2KoboldWalkerT2
            | DD2KoboldWalkerT3
            | DD2OgreT2
            | DD2OgreT3
            // The lunar tile-collision mobs (the aiStyle-3 walkers + the
            // spider/sroller/selenian/beast machines; the flyers — the
            // cells, the jellyfish, the floater, the suckler, the corite,
            // the alien — raw-integrate and stay off the layer).
            | SolarSpearman
            | VortexSoldier
            | StardustSpiderBig
            | StardustSpiderSmall
            | StardustSoldier
            | SolarDrakomire
            | SolarSroller
            | SolarSelenian
            | Predictor
            | VortexHunter
            | VortexHeadcrab
            | NebulaBeast
            | VortexRifleman
    )
}

/// One `Collision_WaterCollision` pass (`NPC.cs:93052-93275`) + the
/// `UpdateCollision` tail's wet-flag clear and `wetCount--`
/// (`:92527-92535`). The caller runs it at the C# call position — after
/// the AI machine's draws, before the queued-sound drain — and drains
/// the `NpcSoundCue::Splash` request through `play_splash_sound` on the
/// `UpdateNPCs` stream. RNG-free: the decision draws nothing and the
/// dust bursts are the documented skip; the case-19 pitch rides the
/// drain's play.
pub(crate) fn collision_water_step(npc: &mut Npc, world: &World) {
    let npc_type = npc.kind.cs_id() as i32;
    let ai_style = npc.kind.ai_style() as i32;
    let (w, h) = npc.size();
    let (wi, hi) = (w as i32, h as i32);
    // `Collision_LavaCollision` runs FIRST (`:92526`'s argument): the
    // probe + the type-441 refusal + the `lavaWet = true` latch
    // (`:93317-93324`); the burn/strike half is the damage-layer seam.
    let mut lava = lava_collision_probe(world, npc.x, npc.y, wi, hi);
    if npc_type == 441 {
        lava = false;
    }
    if lava {
        npc.lava_wet = true;
    }
    // `:93054-93076` — the never-wet force vs the wet probe + the
    // sticky honey/shimmer sets.
    let mut wet_now;
    if never_wet(npc_type, ai_style, npc.ai[0]) {
        wet_now = false;
        npc.wet_count = 0;
        lava = false;
    } else {
        let (wet, honey, shimmer) = wet_collision_probe(world, npc.x, npc.y, wi, hi);
        wet_now = wet;
        if honey {
            npc.honey_wet = true;
        }
        if shimmer {
            npc.shimmer_wet = true;
            // `:93071-93074` — TryAddingRepeatedBuff(353, 100) is the
            // buff-layer seam (the port's NPC buff engine has no
            // repeated-buff adder; the Shimmered debuff's grant lands
            // with it).
        }
    }
    // `:93077-93080` — aiStyle 116 (the Angry Dandelion pair 612/613,
    // unported) pins the cooldown; the term is ported for the census.
    if ai_style == 116 {
        npc.wet_count = 10;
    }
    // The edge decision: `was_wet` is the PREVIOUS pass's `base.wet`
    // (the `wet_prev` latch — see the module header), `lava` the fresh
    // probe on entry / the OLD `lavaWet` on exit (`:93090`/`:93186`).
    let was_wet = npc.wet_prev;
    let exit_lava = npc.lava_wet;
    let d = crate::npc::water_collision_splash(
        npc_type,
        ai_style,
        npc.kind.no_gravity(),
        wet_now,
        was_wet,
        npc.wet_count,
        if wet_now { lava } else { exit_lava },
        npc.shimmer_wet,
        npc.honey_wet,
    );
    if wet_now {
        if d.arm_cooldown {
            npc.wet_count = 10; // `:93089`
        }
        // `:93175` — `base.wet = true` sits AFTER the whole entry arm
        // (the decision already ran on the old flag).
        npc.wet_prev = true;
        npc.wet = true;
    } else if was_wet {
        // `:93179-93180` — the exit's velocity halving + the wet clear.
        // The type-620 leap (`:93181-93184`) is unported-kind movement.
        npc.vx *= 0.5;
        npc.wet_prev = false;
        npc.wet = false;
        if d.arm_cooldown {
            npc.wet_count = 10; // `:93187`
        }
    }
    if let Some(style) = d.style {
        // The case-19 play at `(int)position.X/Y` — the queue drains on
        // the `UpdateNPCs` stream through `play_splash_sound`.
        npc.wants_sound.push(NpcSoundRequest {
            cue: NpcSoundCue::Splash(style),
            x: npc.x,
            y: npc.y,
        });
    }
    // `UpdateCollision`'s tail (`:92527-92535`): the dry tick clears
    // all three sticky flags; the cooldown decays one per tick.
    if !npc.wet_prev {
        npc.lava_wet = false;
        npc.honey_wet = false;
        npc.shimmer_wet = false;
    }
    if npc.wet_count > 0 {
        npc.wet_count -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::LiquidKind;

    fn water_world() -> World {
        let mut w = World::new("t".into(), 200, 200);
        // A water column at tiles x 60..64, y 60..70 (full cells).
        for x in 60..64 {
            for y in 60..70 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        w
    }

    /// `Collision.cs:1572-1651` — the centred box overlaps the water
    /// column; a dry box misses; the honey/shimmer flags ride the cell
    /// kind; a half-full cell's surface inset can miss a shallow box.
    #[test]
    fn w4_e9_the_wet_probe_matches_the_centred_box_and_surface_inset() {
        let w = water_world();
        // A zombie-sized box (18x40) centred in the column: wet.
        let (wet, honey, shimmer) = wet_collision_probe(&w, 61.0 * 16.0, 62.0 * 16.0, 18, 40);
        assert!(wet && !honey && !shimmer);
        // Dry ground far away.
        assert!(!wet_collision_probe(&w, 10.0 * 16.0, 10.0 * 16.0, 18, 40).0);
        // A honey cell: the flag rides the kind.
        let mut hw = water_world();
        let i = hw.index(61, 62);
        hw.tiles[i].set_liquid_kind(LiquidKind::Honey);
        let (wet, honey, shimmer) = wet_collision_probe(&hw, 61.0 * 16.0, 62.0 * 16.0, 18, 40);
        assert!(wet && honey && !shimmer);
        // A shimmer cell.
        let mut sw = water_world();
        let i = sw.index(61, 62);
        sw.tiles[i].set_liquid_kind(LiquidKind::Shimmer);
        let (wet, honey, shimmer) = wet_collision_probe(&sw, 61.0 * 16.0, 62.0 * 16.0, 18, 40);
        assert!(wet && !honey && shimmer);
        // The surface inset: liquid 1 drops the cell top by ~15.9 px;
        // a box entirely above the inset surface misses.
        let mut lw = water_world();
        for x in 60..64 {
            for y in 60..70 {
                let i = lw.index(x, y);
                lw.tiles[i].liquid = 1;
            }
        }
        // The box's centred probe spans y+10..y+30; at y = 62*16 the
        // probe bottom (62*16+30) is above the inset surface
        // (62*16+15.9) only when the box sits higher — park it so the
        // probe bottom is 62*16+8: y = 62*16 - 22.
        assert!(!wet_collision_probe(&lw, 61.0 * 16.0, 62.0 * 16.0 - 22.0, 18, 40).0);
    }

    /// `Collision.cs:1653-1684` — the lava probe reads the FULL box and
    /// only lava cells.
    #[test]
    fn w4_e9_the_lava_probe_reads_the_full_box_lava_only() {
        let mut w = water_world();
        assert!(!lava_collision_probe(&w, 61.0 * 16.0, 62.0 * 16.0, 18, 40));
        let i = w.index(61, 62);
        w.tiles[i].set_liquid_kind(LiquidKind::Lava);
        assert!(lava_collision_probe(&w, 61.0 * 16.0, 62.0 * 16.0, 18, 40));
        // The full box reaches a cell the centred probe would miss:
        // the box's left edge at x = 61*16 - 12 still overlaps tile 60.
        let mut w2 = water_world();
        let i = w2.index(60, 62);
        w2.tiles[i].set_liquid_kind(LiquidKind::Lava);
        assert!(lava_collision_probe(&w2, 61.0 * 16.0 - 12.0, 62.0 * 16.0, 18, 40));
    }

    /// `NPC.cs:93055-93080` — the never-wet kinds force `wetCount = 0`
    /// and refuse the edge even inside water; the aiStyle-116 pin and
    /// the type-441 lava refusal hold.
    #[test]
    fn w4_e9_the_never_wet_kinds_and_the_116_pin() {
        let w = water_world();
        // The Blazing Wheel (72, aiStyle 21) never wets: no cooldown,
        // no play, the wet flags stay dry inside the column.
        let mut wheel = Npc::new(NpcKind::BlazingWheel, 61.0 * 16.0, 62.0 * 16.0, 1, 0.0);
        wheel.wet_count = 7;
        collision_water_step(&mut wheel, &w);
        assert_eq!(wheel.wet_count, 0, ":93058 forces 0");
        assert!(!wheel.wet && !wheel.wet_prev);
        assert!(wheel.wants_sound.is_empty());
        // A plain zombie inside the column: the entry edge arms the
        // cooldown (10, decayed to 9 by the tail) and queues style 0.
        let mut z = Npc::new(NpcKind::Zombie, 61.0 * 16.0, 62.0 * 16.0, 1, 0.0);
        collision_water_step(&mut z, &w);
        assert_eq!(z.wet_count, 9, "10 armed, then :92535's decay");
        assert!(z.wet && z.wet_prev);
        assert_eq!(z.wants_sound.len(), 1);
        assert_eq!(z.wants_sound[0].cue, NpcSoundCue::Splash(0));
        // The next tick: no edge (wet→wet), no second play.
        z.wants_sound.clear();
        collision_water_step(&mut z, &w);
        assert!(z.wants_sound.is_empty());
        assert_eq!(z.wet_count, 8);
        // Never-wet predicate: the Bunfish idle / the sleep pose.
        assert!(never_wet(690, 0, 0.0));
        assert!(!never_wet(690, 0, 1.0));
        assert!(never_wet(22, 7, 25.0));
        assert!(!never_wet(22, 7, 0.0));
        assert!(never_wet(3, 67, 0.0));
    }

    /// `NPC.cs:93177-93270` — the exit edge: the velocity halving, the
    /// wet clear, the style-0 queue and the cooldown churn guard (a
    /// re-entry inside the 10-tick window plays nothing).
    #[test]
    fn w4_e9_the_exit_edge_halves_velocity_and_the_churn_guard_holds() {
        let w = water_world();
        let mut z = Npc::new(NpcKind::Zombie, 61.0 * 16.0, 62.0 * 16.0, 1, 0.0);
        z.vx = 4.0;
        collision_water_step(&mut z, &w); // entry
        z.wants_sound.clear();
        // Teleport dry: the exit edge fires.
        z.x = 10.0 * 16.0;
        z.y = 10.0 * 16.0;
        collision_water_step(&mut z, &w);
        assert_eq!(z.vx, 2.0, ":93179's halving");
        assert!(!z.wet && !z.wet_prev);
        assert_eq!(z.wants_sound.len(), 1);
        assert_eq!(z.wants_sound[0].cue, NpcSoundCue::Splash(0));
        assert_eq!(z.wet_count, 9, "the exit arms 10, decayed to 9");
        // Re-entry inside the cooldown: no play, no arming.
        z.wants_sound.clear();
        z.x = 61.0 * 16.0;
        z.y = 62.0 * 16.0;
        collision_water_step(&mut z, &w);
        assert!(z.wants_sound.is_empty(), "the wetCount churn guard");
        assert!(z.wet && z.wet_prev, "base.wet still sets (:93175)");
    }
}