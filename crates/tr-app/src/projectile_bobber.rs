//! The fishing bobber's flight and bite machine — `AI_061_FishingBobber`
//! (`Projectile.cs:53658-53861`), its water line (`:53863-53891`), the splash
//! (`:53893-53905`), the `FishingCheck` orchestration (`:20583-20595`) and the
//! death-side catch hand-off (`Kill`'s bobber arm `:71986-71994` →
//! `AI_061_FishingBobber_GiveItemToPlayer` `:53907-53958`). S1-P-i35.
//!
//! The pure roll/scan engine lives in `tr_core::fishing` (the attempt builder,
//! the drop-level/enemy/item rolls, the bite accumulator, the pull decision,
//! the bait roll and the catch stacks); this module is the per-tick state
//! machine that drives it and the pool/world-sim wiring.
//!
//! The arm cannot see the App (the pool iterates one slot), so the player /
//! world / zone reads `FishingCheck` needs ride a per-tick [`BobberInputs`]
//! snapshot the world sim feeds through [`crate::projectile::ProjectilePool::
//! set_bobber_inputs`] (the `set_disco` precedent). The catch hand-off and the
//! splash ride back out as pool drains (`wants_mark_spawn` intent pattern).
//!
//! Deviations (ANALYSIS "Fishing rod (S1-P-i35)"):
//! - The wet/lava/honey/shimmer flags are probed at the arm head from the
//!   world tiles; the C# `Update` sets them AFTER the AI (`:16790-17007`), so
//!   the port reads a one-tick-lagged state (the standing projectile-liquid
//!   deviation, `projectile.rs`'s slot doc).
//! - `SpawnFakeFishInWater` (`:20477-20581`) and its `Next(5)`/`Next(10)`/
//!   `NextFromList` sparkle draws are SKIPPED (the dust/particle-skip
//!   convention) — a recorded RNG-order deviation on the `UpdateProjectiles`
//!   stream.
//! - The sonar popup (`PopupText.AssignAsSonarText`) is a draw seam with no
//!   port consumer; the `SoundID.SonarPotion` trackable cue IS ported
//!   (S1-E-21h d75 part 2): it queues on `wants_sound` (id 42, style 311)
//!   inside the bite arms and rides the pool's sound drain — the
//!   `GetRandomPitch` NextFloat is audio-only, case 42 draws nothing.
//! - The chum-bucket projectile book is unported, so `chumsInWater` is 0 and
//!   `ReduceRemainingChumsInPool` (`:53712`) is a no-op.
//! - The blood-moon fish enemies (586/587/618/620/621) and the red town slime
//!   (682) have no `NpcKind` rows: the enemy ROLL runs verbatim (RNG order)
//!   but the actual spawn is skipped for the unported kinds; the 682 pull
//!   still raises `unlockedSlimeRedSpawn` (`Player.cs:52429-52432`). The Duke
//!   Fishron (370) IS ported and spawns.
//!
//! CODE-ONLY MODE (S1-P-i35): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked); every expected value is hand-derived from the
//! cited C# lines.

use std::sync::OnceLock;

use tr_core::fishing::{
    self, AttemptInputs, AttemptRefusal, FishDropRuleList, FishingContext,
};
use tr_core::loot::DropRng;
use tr_core::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
use tr_core::World;

use crate::projectile::{Projectile, BOBBER_KINDS};

/// `ItemID.Count` (`ItemID.cs:14861`) — the kill hook's upper bound on the
/// caught item id (`Projectile.cs:71989`).
const ITEM_ID_COUNT: f32 = 6196.0;

/// The per-tick player/world/zone snapshot `AI_061` + `FishingCheck` read out
/// of `Main.player[owner]` / `Main` / the scene metrics. The world sim builds
/// it once per projectile pass and hands it to the pool.
#[derive(Clone, Copy, Debug, Default)]
pub struct BobberInputs {
    // The kill gates (`Projectile.cs:53664-53679`).
    /// `player.inventory[selectedItem].fishingPole`.
    pub held_pole_power: i32,
    /// `player.inventory[selectedItem].shoot` (0 when the held item shoots
    /// nothing).
    pub held_shoot: u16,
    /// `player.CCed`.
    pub cced: bool,
    /// `player.noItems` (`Projectile.cs:53664` — the Cursed debuff 23).
    pub no_items: bool,
    /// `player.pulley`.
    pub pulley: bool,
    /// `player.dead`.
    pub dead: bool,
    // `GetFishingConditions` (`Player.cs:42176-42206`).
    /// `FinalFishingLevel` — the bite accumulator's level (`:53826`).
    pub final_level: i32,
    /// `BaitItemType`.
    pub bait_type: u16,
    /// `PoleItemType`.
    pub pole_type: u16,
    // The submerged-player gate (`:20599`).
    /// `Main.player[owner].wet`.
    pub player_wet: bool,
    /// `RotatedRelativePoint(MountedCenter).Y`.
    pub player_mounted_y: f32,
    // The attempt builder's scalars (`:20610-20683`).
    /// `Main.player[owner].luck`.
    pub luck: f32,
    /// `Main.player[owner].cratePotion`.
    pub crate_potion: bool,
    /// `Main.player[owner].sonarPotion` (the popup is a skipped draw seam).
    pub sonar_potion: bool,
    /// `Main.player[owner].accLavaFishing`.
    pub acc_lava_fishing: bool,
    /// `Main.notTheBeesWorld`.
    pub not_the_bees: bool,
    /// `Main.remixWorld`.
    pub remix: bool,
    // The enemy roll's world flags (`:20806-20833`).
    pub blood_moon: bool,
    pub day_time: bool,
    pub hard_mode: bool,
    /// `NPC.unlockedSlimeRedSpawn`.
    pub unlocked_slime_red_spawn: bool,
    /// `NPC.AnyNPCs(370)` — a living Duke Fishron.
    pub duke_alive: bool,
    // The quest probe (`:21495-21510`).
    pub quest_item: i32,
    pub player_has_quest_item: bool,
    /// `NPC.AnyNPCs(369)` — a living Angler.
    pub angler_alive: bool,
    pub angler_quest_finished: bool,
    // `PrepareFishingContext`'s zone lanes (`:20857-20908`).
    pub zone_corrupt: bool,
    pub zone_crimson: bool,
    pub zone_jungle: bool,
    pub zone_snow: bool,
    pub zone_dungeon: bool,
    pub zone_desert: bool,
    pub zone_hallow: bool,
    pub zone_glowshroom: bool,
    pub zone_beach: bool,
    /// `Main.downedBoss3` (Skeletron) — the dungeon gate.
    pub downed_boss3: bool,
    /// `NPC.combatBookWasUsed`.
    pub combat_book_was_used: bool,
}

/// `ItemID.Sets.IsLavaBait` (`ItemID.cs:1306`): the three lava baits.
fn is_lava_bait(bait: u16) -> bool {
    matches!(bait, 4849 | 4845 | 4847)
}

/// `ItemID.Sets.CanFishInLava` (`ItemID.cs:1305`): the Hotline pole 2422.
fn pole_fishes_lava(pole: u16) -> bool {
    pole == 2422
}

/// The vanilla drop table, built once (`GameContentFishDropPopulator.Populate`,
/// the 158 rules). `FishingCheck` walks it on every bite.
fn vanilla_rules() -> &'static FishDropRuleList {
    static RULES: OnceLock<FishDropRuleList> = OnceLock::new();
    RULES.get_or_init(fishing::vanilla_drop_rules)
}

/// One tick of `AI_061_FishingBobber` (`Projectile.cs:53658-53861`). `owner`
/// is the owner player's centre (`Main.player[owner].Center`); `None` = no
/// player link (the orphan bobber kills). `rng` is the pool's
/// `UpdateProjectiles` stream (`Main.cs:17775`). Returns `true` when the slot
/// is still active (the caller integrates movement only for a live slot).
pub(crate) fn bobber_step(
    p: &mut Projectile,
    world: &World,
    owner: Option<(f32, f32)>,
    inputs: &BobberInputs,
    rng: &mut dyn DropRng,
) -> bool {
    // `:53661` — the glow-bobber family ignores the shoot-mismatch gate.
    let glow = (986..=993).contains(&p.kind);
    // `:53662` — the life refresh (the bobber never times out while held).
    p.time_left = 60;
    // The wet-flag probe (the C# `Update` tail `:16790-17007`, read here at
    // the head — the one-tick-lag deviation, module doc).
    probe_wet(p, world);
    // `:53664-53684` — the kill gates.
    let kill = inputs.held_pole_power == 0
        || inputs.cced
        || inputs.no_items
        || (inputs.held_shoot != p.kind && !glow)
        || inputs.pulley
        || inputs.dead;
    if kill {
        p.active = false;
        p.time_left = 0;
        return false;
    }
    let Some((pcx, pcy)) = owner else {
        p.active = false;
        p.time_left = 0;
        return false;
    };
    // `:53686-53693` — a reeled catch clears localAI[1] + splashes.
    if p.ai[1] > 0.0 && p.local_ai[1] != 0.0 {
        p.local_ai[1] = 0.0;
        if !p.lava_wet && !p.honey_wet {
            do_a_splash(p);
        }
    }
    // `:53694-53735` — the reel state (ai[0] >= 1).
    if p.ai[0] >= 1.0 {
        if p.ai[0] == 2.0 {
            p.ai[0] += 1.0;
            // `SoundID.Item17` (`:53699`) — the reel cue.
            p.wants_sound = Some((2, 17, p.pos.x, p.pos.y, 0.0));
            if !p.lava_wet && !p.honey_wet {
                do_a_splash(p);
            }
        }
        if p.local_ai[0] < 100.0 {
            p.local_ai[0] += 1.0;
        }
        if p.frame_counter == 0 {
            p.frame_counter = 1;
            // `ReduceRemainingChumsInPool` (`:53712`) — the chum book is
            // unported (module doc), a no-op.
        }
        p.tile_collide = false;
        let (cx, cy) = p.center();
        let mut dx = pcx - cx;
        let mut dy = pcy - cy;
        let mut dist = (dx * dx + dy * dy).sqrt();
        if dist > 3000.0 {
            p.active = false;
            p.time_left = 0;
            return false;
        }
        // `:53724-53728` — the velocity lerp toward the owner at 15.9/dist.
        dist = 15.9 / dist;
        dx *= dist;
        dy *= dist;
        p.vel.x = (p.vel.x * 9.0 + dx) / 10.0;
        p.vel.y = (p.vel.y * 9.0 + dy) / 10.0;
        p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
        // `:53730-53733` — the owner hitbox intersect kills (the catch rides
        // the pool's death hook). The player box is WIDTH x HEIGHT.
        let px = pcx - crate::player::WIDTH * 0.5;
        let py = pcy - crate::player::HEIGHT * 0.5;
        if intersects(p.pos.x, p.pos.y, p.w, p.h, px, py, crate::player::WIDTH, crate::player::HEIGHT) {
            p.active = false;
            p.time_left = 0;
            return false;
        }
        return true;
    }
    // `:53736-53744` — the cast state: the rotation faces the owner, and a
    // >900 px line snaps to the reel.
    let (cx, cy) = p.center();
    let dx = pcx - cx;
    let dy = pcy - cy;
    p.rot = dy.atan2(dx) + 1.57;
    if (dx * dx + dy * dy).sqrt() > 900.0 {
        p.ai[0] = 1.0;
    }
    // `:53745-53811` — the wet/dry physics.
    let mut settled = false;
    if p.wet {
        // `:53747-53758` — the shimmer arm: the localAI[2] latch + the sound
        // 19 style 2. The `AddBuff(353, 60)` grant rides the player (the arm
        // cannot reach it) — deferred to the shimmer wiring (module doc).
        if p.shimmer_wet {
            if p.local_ai[2] == 0.0 {
                p.local_ai[2] = 1.0;
                // `SoundEngine.PlaySound(19, x, y, 2)` (`:53756`) — the
                // splash family, style 2 (the shimmer cut), at the PLAYER's
                // position (the C# reads `Main.player[owner].position`).
                p.wants_splash = Some((2, pcx, pcy));
            }
        }
        p.rot = 0.0;
        p.vel.x *= 0.9;
        if p.vel.y > 0.0 {
            p.vel.y *= 0.5;
        }
        let water_line = fishing::bobber_water_line(
            world,
            (cx / 16.0) as i32,
            (cy / 16.0) as i32,
            p.pos.y,
            p.h,
        );
        if cy > water_line {
            p.vel.y -= 0.1;
            if p.vel.y < -8.0 {
                p.vel.y = -8.0;
            }
            if cy + p.vel.y < water_line {
                p.vel.y = water_line - cy;
            }
        } else {
            p.vel.y = water_line - cy;
        }
        if (-0.01..=0.01).contains(&p.vel.y) {
            settled = true;
        }
    } else {
        if p.vel.y == 0.0 {
            p.vel.x *= 0.95;
        }
        p.vel.x *= 0.98;
        p.vel.y += 0.2;
        if p.vel.y > 15.9 {
            p.vel.y = 15.9;
        }
    }
    // `:53816-53819` — a running bite countdown counts as settled.
    if p.ai[1] != 0.0 {
        settled = true;
    }
    if !settled {
        return true;
    }
    // `:53824-53842` — the bite accumulator (ai[1] == 0, local owner).
    if p.ai[1] == 0.0 {
        let t = fishing::accumulate_bite(p.local_ai[1], inputs.final_level, rng);
        p.local_ai[1] = t;
        if p.local_ai[1] > 660.0 {
            p.local_ai[1] = 0.0;
            fishing_check(p, world, inputs, rng);
        }
    } else if p.ai[1] < 0.0 {
        // `:53843-53860` — the negative-countdown kick + advance.
        let (ai1, la1, kick, net) =
            fishing::bite_kick(p.ai[1], p.local_ai[1], p.vel.y, p.honey_wet, rng);
        if let Some((vx, vy)) = kick {
            p.vel.x = vx;
            p.vel.y = vy;
            p.wet = false;
            p.lava_wet = false;
            p.honey_wet = false;
        }
        p.ai[1] = ai1;
        p.local_ai[1] = la1;
        if net {
            p.net_update = true;
        }
    }
    true
}

/// `FishingCheck` (`Projectile.cs:20583-20595`): the attempt builder, the
/// (skipped) fake-fish sparkle, the hit roll and the results. The RNG draw
/// order is the C#'s verbatim (module doc records the sparkle skip).
fn fishing_check(p: &mut Projectile, world: &World, inputs: &BobberInputs, rng: &mut dyn DropRng) {
    let (cx, cy) = p.center();
    let x = (cx / 16.0) as i32;
    let y = (cy / 16.0) as i32;
    let can_lava = pole_fishes_lava(inputs.pole_type)
        || is_lava_bait(inputs.bait_type)
        || inputs.acc_lava_fishing;
    let mut zones = FishingContext {
        zone_corrupt: inputs.zone_corrupt,
        zone_crimson: inputs.zone_crimson,
        zone_jungle: inputs.zone_jungle,
        zone_snow: inputs.zone_snow,
        zone_dungeon: inputs.zone_dungeon,
        zone_desert: inputs.zone_desert,
        zone_hallow: inputs.zone_hallow,
        zone_glowshroom: inputs.zone_glowshroom,
        zone_beach: inputs.zone_beach,
        downed_boss3: inputs.downed_boss3,
        hard_mode: inputs.hard_mode,
        remix_world: inputs.remix,
        not_the_bees_world: inputs.not_the_bees,
        blood_moon: inputs.blood_moon,
        combat_book_was_used: inputs.combat_book_was_used,
        world_width: world.width,
        ..Default::default()
    };
    let attempt_inputs = AttemptInputs {
        player_wet: inputs.player_wet,
        bobber_center_y: cy,
        player_mounted_y: inputs.player_mounted_y,
        not_the_bees: inputs.not_the_bees,
        remix: inputs.remix,
        base_level: inputs.final_level,
        bait_type: inputs.bait_type,
        can_fish_in_lava: can_lava,
        luck: inputs.luck,
        crate_potion: inputs.crate_potion,
        chums_in_water: 0,
        duke_alive: inputs.duke_alive,
        quest_item: inputs.quest_item,
        player_has_quest_item: inputs.player_has_quest_item,
        angler_alive: inputs.angler_alive,
        angler_quest_finished: inputs.angler_quest_finished,
    };
    match fishing::try_build_fishing_attempt(world, x, y, p.pos.y, &attempt_inputs, &mut zones, rng) {
        // `:20620-20631` — the Truffle Worm's duke arm writes the countdown.
        Err(AttemptRefusal::TruffleWorm { duke_arm: true }) => {
            p.ai[1] = rng.next_range(-180, -60) as f32 - 100.0;
            p.local_ai[1] = 1.0;
            p.local_ai[2] = 2673.0;
            p.net_update = true;
        }
        Err(_) => {}
        Ok(attempt) => {
            // `SpawnFakeFishInWater` (`:20588`) — skipped (module doc).
            // `:20589-20591` — the hit roll.
            if fishing::fishing_check_roll(attempt.fishing_level, rng) {
                let res = fishing::set_fishing_check_results(
                    &attempt,
                    &zones,
                    vanilla_rules(),
                    inputs.bait_type,
                    inputs.pole_type,
                    inputs.blood_moon,
                    inputs.day_time,
                    inputs.hard_mode,
                    inputs.unlocked_slime_red_spawn,
                    is_lava_bait(inputs.bait_type),
                    inputs.acc_lava_fishing,
                    rng,
                );
                p.ai[1] = res.ai1;
                p.local_ai[1] = res.local_ai1;
                p.local_ai[2] = res.local_ai2;
                if res.net_update {
                    p.net_update = true;
                }
                // S1-E-21h d75 part 2 — the sonar bite cues (`:20746-20751`/
                // `:20762-20766`): the popup stays a skipped draw seam.
                sonar_bite_cues(p, inputs.sonar_potion, res.rolled_item, res.rolled_enemy);
            }
        }
    }
}

/// `AI_061_FishingBobber_DoASplash` (`Projectile.cs:53893-53905`): the 100
/// water dusts are SKIPPED (the dust layer, no port consumer); the sound 19
/// style 0 rides the pool's splash drain.
fn do_a_splash(p: &mut Projectile) {
    p.wants_splash = Some((0, p.pos.x, p.pos.y));
}

/// S1-E-21h d75 part 2 — `SetFishingCheckResults`'s sonar cues
/// (`Projectile.cs:20746-20751`/:20762-20766): `SoundID.SonarPotion`
/// (id 42, style 311 — the `Sounds/Custom/sonar_potion` trackable,
/// `SoundID.cs:1740`) at `base.position`, one play per landed roll, the
/// ITEM arm first (the C# statement order — the single intent slot keeps
/// the ENEMY cue when both land, the turret-pair fold). The style's
/// `GetRandomPitch` NextFloat (variance 0.03) is audio-only and case 42
/// draws nothing from the pass stream. The popup text stays skipped.
fn sonar_bite_cues(p: &mut Projectile, sonar: bool, rolled_item: u16, rolled_enemy: i32) {
    if !sonar {
        return;
    }
    if rolled_item > 0 {
        p.wants_sound = Some((42, 311, p.pos.x, p.pos.y, 0.0));
    }
    if rolled_enemy > 0 {
        p.wants_sound = Some((42, 311, p.pos.x, p.pos.y, 0.0));
    }
}

/// The kill hook's catch hand-off (`Projectile.cs:71986-71994` →
/// `GiveItemToPlayer` `:53907-53958`): when a bobber dies carrying a reeled
/// item (`0 < ai[1] < ItemID.Count`), roll its stack and queue the catch;
/// `ai[1]` zeroes either way. Returns `(item, stack)` for the pool to drain.
/// `level` is the owner's `GetFishingConditions().FinalFishingLevel`.
pub(crate) fn kill_hook(p: &mut Projectile, level: i32, rng: &mut dyn DropRng) -> Option<(u16, i32)> {
    let mut out = None;
    if p.ai[1] > 0.0 && p.ai[1] < ITEM_ID_COUNT {
        let item = p.ai[1] as u16;
        let stack = fishing::catch_stack(item, level, rng);
        out = Some((item, stack));
    }
    p.ai[1] = 0.0;
    out
}

/// `Collision.WetCollision` (`Collision.cs:1572-1651`) + `LavaCollision`
/// (`:1653-1684`) folded into one probe: the inset rectangle for wet/honey/
/// shimmer, the FULL box for lava. Writes the slot's four flags.
fn probe_wet(p: &mut Projectile, world: &World) {
    let (x, y, w, h) = (p.pos.x, p.pos.y, p.w, p.h);
    let wi = w as i32;
    let hi = h as i32;
    let inset_w = 10.min(wi);
    let inset_h = (hi / 2).min(hi);
    let px = x + (wi / 2) as f32 - (inset_w / 2) as f32;
    let py = y + (hi / 2) as f32 - (inset_h / 2) as f32;
    let max_y = (world.height - 40).max(0);
    let x0 = (((x / 16.0) as i32) - 1).clamp(0, world.width - 1);
    let x1 = ((((x + wi as f32) / 16.0) as i32) + 2).clamp(0, world.width - 1);
    let y0 = (((y / 16.0) as i32) - 1).clamp(0, max_y);
    let y1 = ((((y + hi as f32) / 16.0) as i32) + 2).clamp(0, max_y);
    let mut wet = false;
    let mut honey = false;
    let mut shimmer = false;
    let mut lava = false;
    for tx in x0..x1 {
        for ty in y0..y1 {
            let Some(t) = world.tile(tx, ty) else { continue };
            if t.liquid > 0 {
                let left = tx as f32 * 16.0;
                let num5 = (256 - t.liquid as i32) as f32 / 32.0;
                let top = ty as f32 * 16.0 + num5 * 2.0;
                let depth = 16.0 - (num5 * 2.0) as i32 as f32;
                // The inset wet probe (`:1613`).
                if px + inset_w as f32 > left
                    && px < left + 16.0
                    && py + inset_h as f32 > top
                    && py < top + depth
                {
                    wet = true;
                    if t.flags & FLAG_LIQUID_HONEY != 0 {
                        honey = true;
                    }
                    if t.flags & FLAG_LIQUID_SHIMMER != 0 {
                        shimmer = true;
                    }
                }
                // The full-box lava probe (`:1676`).
                if t.flags & FLAG_LIQUID_LAVA != 0
                    && x + w > left
                    && x < left + 16.0
                    && y + h > top
                    && y < top + depth
                {
                    lava = true;
                }
            }
        }
    }
    p.wet = wet;
    p.honey_wet = honey;
    p.shimmer_wet = shimmer;
    p.lava_wet = lava;
}

/// `Rectangle.Intersects` — the axis-aligned box overlap.
fn intersects(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

/// Whether a kind is a fishing bobber (the aiStyle-61 dispatch gate).
pub(crate) fn handles(kind: u16) -> bool {
    BOBBER_KINDS.contains(&kind)
}

// --- S1-P-i35 witnesses (WRITTEN NOT RUN — code-only mode) -----------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::BOBBER_WOODEN;
    use tr_core::tile_light::FastRandom;

    fn bobber(x: f32, y: f32) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = BOBBER_WOODEN;
        p.owner = 0;
        p.w = 14.0;
        p.h = 14.0;
        p.pos = glam::Vec2::new(x, y);
        p
    }

    fn world() -> World {
        World::new("t".into(), 1200, 400)
    }

    fn inputs() -> BobberInputs {
        BobberInputs {
            held_pole_power: 5,
            held_shoot: BOBBER_WOODEN,
            final_level: 100,
            bait_type: 2674,
            pole_type: 2289,
            ..Default::default()
        }
    }

    #[test]
    fn the_kill_gates_drop_the_bobber() {
        // `:53664-53684`: no held pole / CC / pulley / dead kills.
        let w = world();
        let mut rng = FastRandom::new(1);
        let mut ins = inputs();
        let mut p = bobber(100.0, 100.0);
        ins.held_pole_power = 0;
        assert!(!bobber_step(&mut p, &w, Some((100.0, 100.0)), &ins, &mut rng));
        assert!(!p.active);
        // The shoot mismatch kills a plain bobber but not a glow one.
        let mut p = bobber(100.0, 100.0);
        ins.held_pole_power = 5;
        ins.held_shoot = 999;
        assert!(!bobber_step(&mut p, &w, Some((100.0, 100.0)), &ins, &mut rng));
        let mut g = bobber(100.0, 100.0);
        g.kind = 986;
        assert!(bobber_step(&mut g, &w, Some((100.0, 100.0)), &ins, &mut rng));
    }

    #[test]
    fn the_reel_state_flies_home_and_kills_on_intersect() {
        // `:53694-53733`: ai[0] >= 1 lerps toward the owner; the hitbox
        // intersect kills.
        let w = world();
        let mut rng = FastRandom::new(1);
        let ins = inputs();
        let mut p = bobber(200.0, 100.0);
        p.ai[0] = 1.0;
        // Owner far enough not to intersect, the velocity lerps toward it.
        assert!(bobber_step(&mut p, &w, Some((100.0, 100.0)), &ins, &mut rng));
        assert!(p.vel.x < 0.0, "pulled toward the owner at -x");
        assert!(!p.tile_collide);
        // Owner on top of the bobber: the intersect kills.
        let mut p2 = bobber(100.0, 100.0);
        p2.ai[0] = 1.0;
        assert!(!bobber_step(&mut p2, &w, Some((100.0, 100.0)), &ins, &mut rng));
        assert!(!p2.active);
    }

    #[test]
    fn the_cast_state_snaps_to_reel_past_900() {
        // `:53741-53744`: a >900 px line sets ai[0] = 1.
        let w = world();
        let mut rng = FastRandom::new(1);
        let ins = inputs();
        let mut p = bobber(0.0, 0.0);
        p.pos = glam::Vec2::new(16.0, 16.0);
        bobber_step(&mut p, &w, Some((2000.0, 16.0)), &ins, &mut rng);
        assert_eq!(p.ai[0], 1.0);
    }

    #[test]
    fn the_kill_hook_gives_the_reeled_item() {
        // `:71986-71994`: 0 < ai[1] < 6196 gives the item; ai[1] zeroes.
        let mut rng = FastRandom::new(1);
        let mut p = bobber(0.0, 0.0);
        p.ai[1] = 2300.0; // a plain fish — stack 1
        let catch = kill_hook(&mut p, 100, &mut rng);
        assert_eq!(catch, Some((2300, 1)));
        assert_eq!(p.ai[1], 0.0);
        // A negative ai[1] (an enemy / countdown) gives nothing.
        let mut p2 = bobber(0.0, 0.0);
        p2.ai[1] = -586.0;
        assert_eq!(kill_hook(&mut p2, 100, &mut rng), None);
        // An out-of-range id gives nothing.
        let mut p3 = bobber(0.0, 0.0);
        p3.ai[1] = 7000.0;
        assert_eq!(kill_hook(&mut p3, 100, &mut rng), None);
    }

    #[test]
    fn the_bite_accumulator_fires_fishing_check() {
        // `:53824-53842`: a settled bobber accumulates; >660 fires the check.
        // A dry bobber at rest (vel 0) over a big pond accumulates toward the
        // bite. Level 300 adds ~12-74/tick, so a few ticks cross 660.
        let mut w = world();
        for x in 20..60 {
            for y in 110..130 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        let mut rng = FastRandom::new(3);
        let mut ins = inputs();
        ins.final_level = 300;
        // Gate V3: `bobber_step` computes velocity only (the pool integrates
        // position). To probe the SETTLED bite accumulator, park the bobber's
        // centre exactly on the water line: column 30, the pond's top row 110
        // (liquid 255) gives waterLine = (110+1)*16 − 255/16 = 1761. Centre.Y
        // = pos.Y + h/2 = 1761 → pos.Y = 1754. Then `cy > waterLine` is false,
        // vel.Y = waterLine − cy = 0 → settled, and the bite timer accumulates.
        let mut p = bobber(30.0 * 16.0, 1754.0);
        p.wet = true;
        p.vel = glam::Vec2::ZERO;
        let mut fired = false;
        for _ in 0..200 {
            bobber_step(&mut p, &w, Some((30.0 * 16.0, 100.0 * 16.0)), &ins, &mut rng);
            if p.ai[1] != 0.0 || p.local_ai[2] != 0.0 {
                fired = true;
                break;
            }
        }
        assert!(fired, "the bite timer crossed 660 and FishingCheck ran");
    }

    /// S1-E-21h d75 part 2 — the sonar bite cues (`:20746-20751`/
    /// `:20762-20766`): the trackable intent (42, 311) at `base.position`,
    /// one per landed roll, gated on the potion flag; the item arm first,
    /// the enemy arm's intent REPLACES it when both land (the single-slot
    /// fold). No potion, no roll → no intent.
    #[test]
    fn s1_e_21h_d75_p2_the_sonar_bite_queues_the_trackable() {
        let mut p = bobber(480.0, 320.0);
        // No potion: nothing queues.
        sonar_bite_cues(&mut p, false, 2300, 586);
        assert_eq!(p.wants_sound, None);
        // No roll: nothing queues either.
        sonar_bite_cues(&mut p, true, 0, 0);
        assert_eq!(p.wants_sound, None);
        // The item arm.
        sonar_bite_cues(&mut p, true, 2300, 0);
        assert_eq!(p.wants_sound, Some((42, 311, 480.0, 320.0, 0.0)));
        // The enemy arm alone.
        p.wants_sound = None;
        sonar_bite_cues(&mut p, true, 0, 586);
        assert_eq!(p.wants_sound, Some((42, 311, 480.0, 320.0, 0.0)));
        // Both land: the enemy intent wins the slot (the C# order).
        p.wants_sound = None;
        sonar_bite_cues(&mut p, true, 2300, 586);
        assert_eq!(p.wants_sound, Some((42, 311, 480.0, 320.0, 0.0)));
    }
}