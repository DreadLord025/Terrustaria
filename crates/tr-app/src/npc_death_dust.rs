//! W4-g59 — the NPC death DUST lane: `NPC.HitEffect`'s per-kind death-half
//! dust bursts (`life <= 0`), kinds 1-230, batches 1-7 (64 kinds — every
//! dust-first clean kind the port's kind set carries).
//!
//! `HitEffect` runs inside `StrikeNPC_Inner` (`NPC.cs:81488`), BEFORE
//! `checkDead` (`:81561` → `:78040`); the port batches deaths into the NPC
//! pass (the S1-E-21c recorded deviation), so the feed rides the death
//! retain in `world_sim/npcs.rs` ahead of the death cue — the W4-c12
//! `style74_death_feed` precedent. Every draw hits the `UpdateNPCs` stream.
//!
//! ## Scope — batch 1 (dust-only death arms, zero gore, zero side effects)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 23 | MeteorHead | `:89487-89501` | 50× Next(2)-selected 25/6 + 50× type-6 vel×6 |
//! | 120/137/138 | IlluminantSlime (138) | `:85490-85494` | 50× type-71 alpha 200, vel×1.5 |
//! | 143/144/145 | the snowman trio | `:85370-85375` | 50× type-76 noGravity, scale×1.2 |
//! | 147/184 | IceSlime/SpikedIceSlime | `:85350-85355` | 30× (76 noGravity + 4 tinted) pairs |
//! | 183 | Crimslime | `:85420-85424` | 50× type-136 `this.alpha`/`this.color`, vel×2 |
//! | 210/211 | Bee/BeeSmall | `:85065-85077` | 6× type-150, Next(2) scale gate |
//!
//! ## Scope — batch 2 (the gore-tail kinds; the dust is the whole feed)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 2 | DemonEye | `:86459-86462` | 50× type-5, 2×dir/-2 |
//! | 5 | ServantOfCthulhu | `:87351-87354` | 20× type-5, 2×dir/-2 |
//! | 6/7/13 | EaterOfSouls/Devourer/EoW heads | `:87512`/`:87566`/`:87632` | 50× type-18 alpha |
//! | 10 | GiantWormHead | `:87600-87603` | 10× type-5, 2.5×dir/-2.5 |
//! | 17/18/19/20 | Merchant/Nurse/ArmsDealer/Dryad | `:87662` etc. | 50× type-5, 2.5×dir |
//!
//! Every batch-2 gore tail draws NO arguments (constant types or
//! `this.type` arithmetic — the shimmer/altTexture gates are field reads),
//! so the dust loop is the whole RNG census; the `NewGore` bodies + kicks
//! stay d55's lane (the g63 claim's gore-sibling rule).
//!
//! ## Scope — batch 3 (the skeleton/goblin/Skeletron families)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 21/32 | Skeleton/DarkCaster | `:88537-88540` | 20× type-26, 2.5×dir/-2.5 |
//! | 22 | Guide | `:87932-87935` | 50× type-5, 2.5×dir/-2.5 |
//! | 24 | FireImp | `:89514-89519` | 50× type-6 at velocity, noGravity, vel×2 |
//! | 26/27/28/29 | the goblin quartet | `:89560-89563` | 50× type-5, 2.5×dir/-2.5 |
//! | 35/36 | SkeletronHead/Hand | `:89177-89180` | 150× type-26, 2.5×dir/-2.5 |
//!
//! The batch-3 gore tails also draw NO arguments (the skeleton arm's
//! per-kind ladder, the goblin type ladder `:89564-89596` and the
//! Skeletron RedHat gate `:89181-89208` are constant data / ai-field
//! reads), so the dust loop is again the whole census.
//!
//! ## Scope — batch 4 (the bat/shark/piranha families + the hornet)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 38/54 | Demolitionist/Clothier | `:88335`/`:88252` | 50× type-5, 2.5×dir/-2.5 |
//! | 42 | Hornet | `:89712-89715` | 50× type-18 alpha/color/scale, dir |
//! | 44/45 | UndeadMiner/Tim | `:88537-88540` | 20× type-26, 2.5×dir/-2.5 |
//! | 49/51 | CaveBat/JungleBat | `:85807-85810` | 15× type-5, 2×dir/-2 |
//! | 58 | Piranha | `:86381-86384` | 10× type-5, 2×dir/-2 |
//! | 65 | Shark | `:86740-86743` | 75× type-5, 2×dir/-2 |
//! | 66 | VoodooDemon | `:89803-89806` | 50× type-5, 2×dir/-2 |
//!
//! The batch-4 gore tails draw NO arguments (verified: the hornet's
//! 205/176 ladder, the bat per-kind gores, the piranha/shark/voodoo
//! ladders are all constant data). Shared-arm siblings outside the port's
//! kind set (47/57, 62, 176/205/231-235, 542-545/692) stay cited skips.
//!
//! ## Scope — batch 5 (the wyvern/eye/town families + the leech)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 87 | WyvernHead | `:88921-88932` | 10× type-16 vel×2 + gore ARG draws |
//! | 106/107/108 | BoundWizard/GoblinTinkerer/Wizard | `:87880-87883` | 50× type-5, 2.5×dir/-2.5 |
//! | 111 | GoblinArcher | `:89560-89563` | 50× type-5, 2.5×dir/-2.5 |
//! | 117 | LeechHead | `:87446-87449` | 10× type-5, dir/-1 |
//! | 123/124 | BoundMechanic/Mechanic | `:87912-87915` | 50× type-5, 2.5×dir/-2.5 |
//! | 190/191 | CataractEye/SleepyEye | `:86459-86462` | 50× type-5, 2×dir/-2 |
//!
//! The Wyvern (87) is the FIRST arm with gore ARGUMENT draws: the
//! `Next(1,4)` gore count + the per-gore `Next(11,14)` type roll
//! (`:88927-88930`) are consumed in order here; the `NewGore` bodies +
//! kicks stay d55's (the g63 claim's gore-sibling rule). The other
//! batch-5 tails draw no arguments.
//!
//! ## Scope — batch 6 (the eye trio, the dye/party girls, the pirates)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 192/193/194 | Dialated/Green/PurpleEye | `:86459-86462` | 50× type-5, 2×dir/-2 |
//! | 207/208 | DyeTrader/PartyGirl | `:88378`/`:88401` | 50× type-5, 2.5×dir/-2.5 |
//! | 212-216 | the pirate family | `:88458-88461` | 50× type-5, 2.5×dir/-2.5 |
//!
//! All batch-6 gore tails draw NO arguments (the eye per-kind types and
//! the pirate ladder `:88462-88501` are constant data).
//!
//! ## Scope — batch 7 (the final present-clean kinds)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 222 | QueenBee | `:87491-87494` | 50× type-18 alpha/color/scale, dir |
//! | 225 | UmbrellaSlime | `:85038-85041` | 50× type-4 tinted (0,80,255,100) |
//! | 227/228 | Painter/WitchDoctor | `:88086`/`:88109` | 50× type-5, 2.5×dir/-2.5 |
//!
//! All batch-7 gore tails draw NO arguments (the Queen Bee's 6-gore
//! ladder `:87495-87500`, the Umbrella's 314 and the painter/witch
//! shimmer gates are constant data / field reads).
//!
//! ## The lane's remainder (the census's absent kinds)
//!
//! The 173-kind dust-first clean census has 109 kinds with NO port kind
//! row (the worm segments 8/9/11/12/14/15, the casters 31/37/39-41/43,
//! the frost/martian/dd2 families, the 195-206/209/217-221/226/229
//! bands, …). Their arms stay UNWIRED until the kind lands (the S1-F-9b
//! row-gate convention); the census rows + arm citations live in
//! docs/ANALYSIS.md. The 20 dust-only kinds' absent members (34/63/75/
//! 112/120/122/137/141/169/204) are the same remainder.
//!
//! The census (the full 1-230 walk, 173 dust-first clean kinds) and the
//! batch split live in docs/ANALYSIS.md "The NPC death dust table (W4-g59)".
//! The gore-bearing death arms are d55's lane (`npc_hit_effect_a.rs`,
//! ec78d152); the 231-697 band is W4-g60; death SPAWNS are W4-g61; the
//! `life > 0` hit halves are W4-g62. The 112/666 whole-arm burst
//! (`:85395-85408`) fires on EVERY hit, not death-only — g62's.
//!
//! ## RNG contract (the d50 DustPool contract)
//!
//! Argument draws first (the Next(2) selectors), then the pool internals
//! inside `DustPool::new_dust` (S1-E-1: the 6 default-block rolls; type 6
//! adds the tweak-chain `Next(-10, 6)`, `Dust.cs:214-220`; types
//! 4/18/25/71/76/136/150 hit no tweak). A cull refusal or a pressure-band
//! gate consumes exactly the C#'s draws (the pool owns them). The
//! post-spawn writes land on the returned slot, the sentinel included —
//! the C# writes `Main.dust[6000]` too (the `npc_ai_style56.rs` precedent).
//! Batch 1's types are all ChildSafety-safe; batch 2's type 5 IS dangerous
//! (`ChildSafety.cs`'s {5,227,273,237,248,249,250,251} — the
//! `DANGEROUS_DUST` set), but the re-type arm's draws are pool-owned and
//! the blood toggle defaults ON (`Disabled = true`, no re-type).
//!
//! ## Documented stand-ins
//!
//! * `hitDirection` rides [`Npc::last_hit_dir`] (recorded at the three
//!   strike seams; 0 for an argument-less `HitEffect()` — the C# default,
//!   `NPC.cs:81654`).
//! * 183's `this.color`: `default(Color)` — the SetDefaults reset
//!   (`NPC.cs:8526`) is the ONLY write for 183 (the paint-blend ladder
//!   `:60577-60692` is type-1-only, `AI_001_Slimes`). The port's `Npc`
//!   carries no color field; when the slime paint lane lands, the feed
//!   reads it here.
//! * 183's `this.alpha` reads [`Npc::alpha`] — the ctor carries the C#
//!   `alpha = 55` row (`NPC.cs:11342`, added with batch 1).
//! * Batch 2's type-18 trio (6/7/13) reads `this.alpha`/`this.scale` off
//!   the ctor: the EoW head's `alpha = 255` SetDefaults row (`:8921`) is
//!   the spawn-fade window (`AI_006_Worms` `:50998`'s `alpha -= 42`), which
//!   the port has not ported — the ctor keeps the steady-state 0, so the
//!   feed reads 0 (the fade window is the worm AI lane's remainder).
//! * The 143/144/145 trio shares ONE arm; the port's bestiary id rows for
//!   SnowBalla/MysterySnowman (144/145 vs `NPCID.cs:10965-10967`'s
//!   MisterStabby/SnowBalla naming) are dust-neutral — every id in the
//!   trio draws the same burst.

use crate::npc::Npc;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The batch-1 death dust feed: draws the kind's `HitEffect` DEATH half in
/// C# order. Returns true when an arm ran (the wiring gate mirrors the
/// match; the flag is the witness seam). `npc_type` is the C# `NPC.type`
/// (`NpcKind::cs_id`).
pub fn death_dust(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) -> bool {
    let (w, h) = npc.size();
    let (w, h) = (w as i32, h as i32);
    let pos = Vec2::new(npc.x, npc.y);
    let dir = npc.last_hit_dir as f32;
    match npc_type {
        // `NPC.cs:89487-89501` — the Meteor Head: 50 Next(2)-selected
        // 25/6 puffs, then 50 type-6 embers at velocity×0.2, scale 2.5,
        // velocity×6 + noGravity.
        23 => {
            for _ in 0..50 {
                let mut ty = 25;
                if rng.next_max(2) == 0 {
                    ty = 6;
                }
                dust.new_dust(
                    rng, ctx, world, pos, w, h, ty, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 6,
                    npc.vx * 0.2, npc.vy * 0.2, 100, Color::default(), 2.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 6.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:85490-85494` — the 120/137/138 arm's death half: 50
        // type-71 puffs, alpha 200, velocity×1.5.
        120 | 137 | 138 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 71, dir, 0.0, 200,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 1.5;
            }
        }
        // `NPC.cs:85370-85375` — the snowman trio: 50 type-76 puffs,
        // noGravity, scale×1.2 (the SpeedY stays -1, the hit half's value).
        143 | 144 | 145 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 76, dir, -1.0, 0,
                    Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.2;
            }
        }
        // `NPC.cs:85350-85355` — the 147/184 pair: 30 × (type-76
        // noGravity + type-4 tinted) pairs, SpeedY -2 on both.
        147 | 184 => {
            for _ in 0..30 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 76, dir, -2.0, 0,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).no_gravity = true;
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, dir, -2.0, 100,
                    Color::new(97, 200, 255, 100), 1.0,
                );
            }
        }
        // `NPC.cs:85420-85424` — the Crimslime: 50 type-136 puffs at
        // `this.alpha`/`this.color` (the color stand-in: default, see the
        // module doc), velocity×2.
        183 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 136, dir, 0.0,
                    npc.alpha as i32, Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 2.0;
            }
        }
        // `NPC.cs:85065-85077` — the bees' cond-death arm: 6 type-150
        // puffs; the Next(2) gate picks noGravity+scale 1.5 vs scale 0.8.
        210 | 211 => {
            for _ in 0..6 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 150, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                if rng.next_max(2) == 0 {
                    d.no_gravity = true;
                    d.scale = 1.5;
                } else {
                    d.scale = 0.8;
                }
            }
        }
        // ---- batch 2 (the gore-tail kinds; the dust is the whole feed —
        // every gore type below is a constant / `this.type` arithmetic, so
        // the tails draw NO arguments; the NewGore bodies + kicks are d55's).
        // `NPC.cs:86459-86462` — the Demon Eye: 50 type-5 puffs.
        2 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87351-87354` — the Servant of Cthulhu: 20 type-5 puffs.
        5 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87512-87515` / `:87566-87569` / `:87632-87635` — the
        // Eater of Souls (6), Devourer head (7) and EoW head (13): 50
        // type-18 puffs at `this.alpha`/`this.color`/`this.scale`. The
        // color stand-in is default (none of the three writes a color);
        // the alpha reads the ctor (the worm spawn-fade `AI_006` is the AI
        // lane's remainder — the steady-state 0 the port carries).
        6 | 7 | 13 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:87600-87603` — the Giant Worm head: 10 type-5 puffs at
        // 2.5×hitDirection, -2.5.
        10 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87662-87665` / `:88292-88295` / `:88315-88318` /
        // `:88358-88361` — the Merchant/Nurse/Arms Dealer/Dryad: 50 type-5
        // puffs at 2.5×hitDirection, -2.5 (the shimmer/altTexture gore
        // gates draw nothing).
        17 | 18 | 19 | 20 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 3 (the skeleton/goblin/Skeletron families).
        // `NPC.cs:88537-88540` — the Skeleton (21) / Dark Caster (32): 20
        // type-26 bone puffs at 2.5×dir/-2.5 (the big skeleton arm's dust
        // head; the gore tail's types are constants, no argument draws).
        21 | 32 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87932-87935` — the Guide: 50 type-5 puffs at
        // 2.5×dir/-2.5 (the shimmer gore gates are field reads).
        22 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89514-89519` — the Fire Imp: 50 type-6 embers at the
        // corpse's velocity, scale 2.5, noGravity + velocity×2.
        24 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 6, npc.vx, npc.vy, 100,
                    Color::default(), 2.5,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        // `NPC.cs:89560-89563` — the goblin quartet (26/27/28/29): 50
        // type-5 puffs at 2.5×dir/-2.5 (the per-kind gore-type ladder
        // `:89564-89596` is constant data, no argument draws).
        26 | 27 | 28 | 29 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89177-89180` — the Skeletron head (35) / hand (36): 150
        // type-26 bone puffs at 2.5×dir/-2.5 (the RedHat gore gate
        // `:89181-89208` reads ai[3]/localAI[3], no argument draws).
        35 | 36 => {
            for _ in 0..150 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 4 (the bat/shark/piranha families + the hornet).
        // `NPC.cs:88335-88338` / `:88252-88255` — the Demolitionist (38) /
        // Clothier (54): 50 type-5 puffs at 2.5×dir/-2.5.
        38 | 54 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89712-89715` — the Hornet (42): 50 type-18 puffs at
        // `this.alpha`/`this.color`/`this.scale`, SpeedX = dir (the 176/
        // 205/231-235 siblings are other lanes' kinds).
        42 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:88537-88540` — the Undead Miner (44) / Tim (45): 20
        // type-26 bone puffs (the big skeleton arm, batch 3's shape).
        44 | 45 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85807-85810` — the Cave Bat (49) / Jungle Bat (51): 15
        // type-5 puffs at 2×dir/-2 (the bat arm's dust head).
        49 | 51 => {
            for _ in 0..15 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86381-86384` — the Piranha (58): 10 type-5 puffs at
        // 2×dir/-2 (the 47/57 siblings are unported kinds).
        58 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86740-86743` — the Shark (65): 75 type-5 puffs at
        // 2×dir/-2 (the 542-545/692 siblings are other lanes' kinds).
        65 => {
            for _ in 0..75 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89803-89806` — the Voodoo Demon (66): 50 type-5 puffs
        // at 2×dir/-2 (the 62 sibling is an unported kind).
        66 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 5 (the wyvern/eye/town families + the leech).
        // `NPC.cs:88921-88932` — the Wyvern head (87): 10 type-16 puffs
        // (scale 1.5, vel×2, noGravity), then the gore tail's ARGUMENT
        // draws — Next(1,4) count + Next(11,14) per gore (the bodies +
        // kicks are d55's; the g63 claim's gore-sibling rule).
        87 => {
            for _ in 0..10 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 16, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            let count = rng.next_range(1, 4);
            for _ in 0..count {
                let _gore_type = rng.next_range(11, 14);
            }
        }
        // `NPC.cs:87880-87883` — the Bound Wizard (106) / Goblin Tinkerer
        // (107) / Wizard (108): 50 type-5 puffs at 2.5×dir/-2.5 (the
        // shimmer/altTexture gore gates are field reads).
        106 | 107 | 108 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89560-89563` — the Goblin Archer (111): 50 type-5 puffs
        // (the goblin arm, batch 3's shape; the 111 gore ladder adds the
        // 131 bone, constant data).
        111 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87446-87449` — the Leech head (117): 10 type-5 puffs at
        // dir/-1 (the SpeedY stays -1, NOT -2 — the leech arm's own
        // value; the 118/119 siblings fold onto the head's bestiary id).
        117 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, dir, -1.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87912-87915` — the Bound Mechanic (123) / Mechanic
        // (124): 50 type-5 puffs at 2.5×dir/-2.5.
        123 | 124 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86459-86462` — the Cataract Eye (190) / Sleepy Eye
        // (191): 50 type-5 puffs at 2×dir/-2 (the demon-eye arm, batch
        // 2's shape; the per-kind gore types are constants).
        190 | 191 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 6 (the eye trio, the dye/party girls, the pirates).
        // `NPC.cs:86459-86462` — the Dialated/Green/Purple Eyes
        // (192/193/194): 50 type-5 puffs at 2×dir/-2 (the demon-eye arm).
        192 | 193 | 194 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88378-88381` / `:88401-88404` — the Dye Trader (207) /
        // Party Girl (208): 50 type-5 puffs at 2.5×dir/-2.5.
        207 | 208 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88458-88461` — the pirate family (212-216): 50 type-5
        // puffs at 2.5×dir/-2.5 (the per-kind gore ladder `:88462-88501`
        // is constant data).
        212 | 213 | 214 | 215 | 216 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 7 (the Queen Bee, the umbrella slime, the painter/
        // witch doctor — the final present-clean kinds).
        // `NPC.cs:87491-87494` — the Queen Bee (222): 50 type-18 puffs at
        // `this.alpha`/`this.color`/`this.scale`, SpeedX = dir (the
        // 6-gore tail `:87495-87500` is constant types).
        222 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:85038-85041` — the Umbrella Slime (225): 50 type-4
        // puffs at 2×dir/-2, alpha 100, tinted (0,80,255,100) (the 314
        // gore is a constant type).
        225 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, 2.0 * dir, -2.0, 100,
                    Color::new(0, 80, 255, 100), 1.0,
                );
            }
        }
        // `NPC.cs:88086-88089` / `:88109-88112` — the Painter (227) /
        // Witch Doctor (228): 50 type-5 puffs at 2.5×dir/-2.5 (the
        // shimmer/altTexture gore gates are field reads).
        227 | 228 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        _ => return false,
    }
    true
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    fn corpse(kind: NpcKind) -> Npc {
        let mut n = Npc::new(kind, 100.0, 100.0, 1, 0.0);
        n.hp = 0;
        n.last_hit_dir = -1;
        n.vx = 2.0;
        n.vy = -1.0;
        n
    }

    /// `NPC.cs:85065-85077` — the bees' death: 6 type-150 puffs, each the
    /// 6 default-block rolls then the Next(2) gate (noGravity+1.5 vs 0.8).
    /// The hand-rolled ladder pins the exact draw order on the stream.
    #[test]
    fn w4_g59_the_bee_death_draws_six_gated_puffs() {
        let w = air();
        let n = corpse(NpcKind::Bee);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        assert!(death_dust(&n, 210, &mut dust, &ctx(), &mut rng, &w));
        // The twin: the C# draw ladder by hand.
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let mut twin = UnifiedRandom::new(7);
        let mut gates = Vec::new();
        for _ in 0..6 {
            let _px = twin.next_max(bw.max(5) - 4);
            let _py = twin.next_max(bh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            gates.push(twin.next_max(2) == 0);
        }
        assert_eq!(rng, twin, "the bee death spends exactly 6×(6+1) draws");
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 6, "the flat 6 dust 150");
        for (d, &gate) in live.iter().zip(&gates) {
            assert_eq!(d.dust_type, 150);
            if gate {
                assert!(d.no_gravity);
                assert_eq!(d.scale, 1.5);
            } else {
                assert!(!d.no_gravity);
                assert_eq!(d.scale, 0.8);
            }
        }
    }

    /// `NPC.cs:89487-89501` — the Meteor Head: the 50-puff selector loop
    /// then the 50-ember loop, replayed on a twin pool (the type-6 tweak
    /// draw rides inside `new_dust`). 100 slots, the embers noGravity.
    #[test]
    fn w4_g59_the_meteor_head_death_draws_two_fifty_loops() {
        let w = air();
        let n = corpse(NpcKind::MeteorHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        let mut twin_rng = rng.clone();
        let mut twin = DustPool::new();
        assert!(death_dust(&n, 23, &mut dust, &ctx(), &mut rng, &w));
        let (mw, mh) = n.size();
        let (mw, mh) = (mw as i32, mh as i32);
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let mut ty = 25;
            if twin_rng.next_max(2) == 0 {
                ty = 6;
            }
            twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, mw, mh, ty, -2.0, -2.0, 0,
                Color::default(), 1.0,
            );
        }
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, mw, mh, 6, 0.4, -0.2, 100,
                Color::default(), 2.5,
            );
            let d = twin.slot_mut(idx);
            d.velocity *= 6.0;
            d.no_gravity = true;
        }
        assert_eq!(rng, twin_rng, "the two loops mirror the C# draws");
        assert_eq!(dust.slots(), twin.slots(), "slot-for-slot equality");
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 100);
        assert!(live[..50].iter().all(|d| d.dust_type == 25 || d.dust_type == 6));
        assert!(live[50..].iter().all(|d| d.dust_type == 6 && d.no_gravity));
    }

    /// `NPC.cs:85370-85375` — the snowman trio: all three ids draw the
    /// SAME 50×76 census (noGravity, scale×1.2, SpeedY -1).
    #[test]
    fn w4_g59_the_snowman_trio_shares_one_death_census() {
        let w = air();
        let n = corpse(NpcKind::SnowmanGangsta);
        let mut states = Vec::new();
        for ty in [143, 144, 145] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(11);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            for d in &live {
                assert_eq!(d.dust_type, 76);
                assert!(d.no_gravity);
            }
            // The ×1.2 write: the twin's untouched scale ×1.2.
            let mut twin = DustPool::new();
            let mut twin_rng = UnifiedRandom::new(11);
            let (sw, sh) = n.size();
            for _ in 0..50 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                    sw as i32, sh as i32, 76, -1.0, -1.0, 0, Color::default(), 1.0,
                );
                let base = twin.slot(idx).scale;
                let d = twin.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.2;
                assert_eq!(d.scale, base * 1.2);
            }
            assert_eq!(rng, twin_rng);
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]), "one arm, three ids");
    }

    /// `NPC.cs:85350-85355` — the 147/184 pair: 30 × (76 + 4) pairs, the
    /// 4s tinted (97,200,255,100) alpha 100, both SpeedY -2.
    #[test]
    fn w4_g59_the_ice_slime_death_draws_thirty_pairs() {
        let w = air();
        let n = corpse(NpcKind::IceSlime);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        assert!(death_dust(&n, 147, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 60);
        for (i, d) in live.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(d.dust_type, 76);
                assert!(d.no_gravity);
            } else {
                assert_eq!(d.dust_type, 4);
                assert_eq!(d.alpha, 100);
                assert_eq!(d.color, Color::new(97, 200, 255, 100));
            }
        }
        // 184 draws the identical census.
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(5);
        assert!(death_dust(&n, 184, &mut dust2, &ctx(), &mut rng2, &w));
        assert_eq!(rng, rng2);
        assert_eq!(dust.slots(), dust2.slots());
    }

    /// `NPC.cs:85420-85424` — the Crimslime: 50 type-136 puffs at
    /// `this.alpha` (the ctor's 55 row) with velocity×2; the color
    /// stand-in is default (the module doc).
    #[test]
    fn w4_g59_the_crimslime_death_reads_the_alpha_row() {
        let w = air();
        let n = corpse(NpcKind::Crimslime);
        assert_eq!(n.alpha, 55, "the SetDefaults alpha row (`:11342`)");
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        assert!(death_dust(&n, 183, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 136 && d.alpha == 55));
        // The ×2 write against a twin's untouched velocity.
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(9);
        let (cw, ch) = n.size();
        for i in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                cw as i32, ch as i32, 136, -1.0, 0.0, 55, Color::default(), 1.0,
            );
            let base = twin.slot(idx).velocity;
            assert_eq!(live[i].velocity, base * 2.0);
        }
        assert_eq!(rng, twin_rng);
    }

    /// `NPC.cs:85490-85494` — the Illuminant Slime (138): 50 type-71
    /// puffs, alpha 200, velocity×1.5.
    #[test]
    fn w4_g59_the_illuminant_death_draws_fifty_seventy_ones() {
        let w = air();
        let n = corpse(NpcKind::IlluminantSlime);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        assert!(death_dust(&n, 138, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 71 && d.alpha == 200));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(13);
        let (iw, ih) = n.size();
        for i in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                iw as i32, ih as i32, 71, -1.0, 0.0, 200, Color::default(), 1.0,
            );
            let base = twin.slot(idx).velocity;
            assert_eq!(live[i].velocity, base * 1.5);
        }
        assert_eq!(rng, twin_rng);
    }

    /// The cull refusal (`Dust.cs:97-101`): an off-screen corpse's spawns
    /// return the sentinel; the argument draws (the bees' 6 gates) still
    /// run and the post-writes land on `Main.dust[6000]` — the C# shape.
    #[test]
    fn w4_g59_the_offscreen_cull_writes_the_sentinel_slot() {
        let w = air();
        let n = corpse(NpcKind::BeeSmall);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        let far = SpawnCtx {
            screen_position: Vec2::new(100000.0, 100000.0),
            ..ctx()
        };
        assert!(death_dust(&n, 211, &mut dust, &far, &mut rng, &w));
        let mut twin = UnifiedRandom::new(17);
        for _ in 0..6 {
            let _gate = twin.next_max(2);
        }
        assert_eq!(rng, twin, "only the 6 gate draws survive the cull");
        assert!(dust.slots()[..6000].iter().all(|d| !d.active));
        // The sentinel carries the LAST gate's write (the C# writes it too).
        let s = dust.slot(tr_core::particles::DUST_SENTINEL);
        assert!(s.scale == 1.5 || s.scale == 0.8);
    }

    /// The batch gate: a kind outside the match draws nothing and returns
    /// false (the gore-bearing arms are d55's; 231+ is g60's).
    #[test]
    fn w4_g59_a_non_batch_kind_draws_nothing() {
        let w = air();
        let n = corpse(NpcKind::Zombie);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        let before = rng.clone();
        assert!(!death_dust(&n, 3, &mut dust, &ctx(), &mut rng, &w));
        assert_eq!(rng, before, "no arm, no draws");
        assert!(dust.slots().iter().all(|d| !d.active));
    }

    /// `NPC.cs:86459-86462` — the Demon Eye: 50 type-5 puffs at
    /// 2×hitDirection/-2; the gore tail (1 + 2, `:86500-86501`) draws no
    /// arguments, so the twin replay is the dust loop alone.
    #[test]
    fn w4_g59_the_demon_eye_death_draws_fifty_type_fives() {
        let w = air();
        let n = corpse(NpcKind::DemonEye);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        assert!(death_dust(&n, 2, &mut dust, &ctx(), &mut rng, &w));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(23);
        let (dw, dh) = n.size();
        for _ in 0..50 {
            twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                dw as i32, dh as i32, 5, -2.0, -2.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng, twin_rng, "the gore tail draws no arguments");
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:87512-87515` / `:87566-87569` / `:87632-87635` — the
    /// type-18 trio (6/7/13): 50 puffs at `this.alpha`/`this.scale`,
    /// SpeedX = hitDirection (NOT doubled), SpeedY -2. All three ids draw
    /// the identical census on the same ctor state.
    #[test]
    fn w4_g59_the_type18_trio_reads_alpha_and_scale() {
        let w = air();
        let mut n = corpse(NpcKind::EaterOfSouls);
        n.alpha = 200;
        n.scale = 1.3;
        let mut states = Vec::new();
        for ty in [6, 7, 13] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(29);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            for d in &live {
                assert_eq!(d.dust_type, 18);
                assert_eq!(d.alpha, 200);
                // The pool's scale = (1 + Next(-20,21)*0.01) × this.scale.
                assert!((d.scale - 1.3).abs() <= 0.26 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]), "one shape, three ids");
    }

    /// `NPC.cs:87600-87603` — the Giant Worm head: 10 type-5 puffs at
    /// 2.5×dir/-2.5 (the 2.5 multiplier, NOT 2 — the worm family's own
    /// arm, `this.type - 7 + 18` gore draws nothing).
    #[test]
    fn w4_g59_the_giant_worm_death_draws_ten_at_two_point_five() {
        let w = air();
        let n = corpse(NpcKind::GiantWormHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        assert!(death_dust(&n, 10, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 5));
        // The SpeedX argument: -2.5 + the ±2 jitter (dir = -1).
        assert!(live.iter().all(|d| d.velocity.x >= -4.5 && d.velocity.x <= -0.5));
        assert!(live.iter().all(|d| d.velocity.y >= -4.5 && d.velocity.y <= -0.5));
    }

    /// `NPC.cs:87662-87665` etc. — the town quartet (17/18/19/20): 50
    /// type-5 puffs at 2.5×dir/-2.5 each; the shimmer/altTexture gore
    /// gates are field reads (no draws), so all four ids share the census.
    #[test]
    fn w4_g59_the_town_quartet_shares_one_census() {
        let w = air();
        let n = corpse(NpcKind::Merchant);
        let mut states = Vec::new();
        for ty in [17, 18, 19, 20] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:87351-87354` — the Servant of Cthulhu: 20 type-5 puffs
    /// (the 20-count arm, not 50).
    #[test]
    fn w4_g59_the_servant_death_draws_twenty() {
        let w = air();
        let n = corpse(NpcKind::ServantOfCthulhu);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        assert!(death_dust(&n, 5, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 20);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:88537-88540` — the Skeleton/Dark Caster: 20 type-26 bone
    /// puffs at 2.5×dir/-2.5; both ids share the big skeleton arm's dust
    /// head (the gore ladder is constant data).
    #[test]
    fn w4_g59_the_skeleton_pair_draws_twenty_bones() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut states = Vec::new();
        for ty in [21, 32] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(43);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20);
            assert!(live.iter().all(|d| d.dust_type == 26));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:89514-89519` — the Fire Imp: 50 type-6 embers at the
    /// corpse's velocity, scale 2.5, noGravity + velocity×2 (the type-6
    /// tweak draw rides inside the pool).
    #[test]
    fn w4_g59_the_fire_imp_death_draws_fifty_embers() {
        let w = air();
        let n = corpse(NpcKind::FireImp);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let mut twin_rng = rng.clone();
        let mut twin = DustPool::new();
        assert!(death_dust(&n, 24, &mut dust, &ctx(), &mut rng, &w));
        let (fw, fh) = n.size();
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                fw as i32, fh as i32, 6, n.vx, n.vy, 100, Color::default(), 2.5,
            );
            let d = twin.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 2.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 6 && d.no_gravity));
    }

    /// `NPC.cs:89560-89563` — the goblin quartet (26/27/28/29): 50 type-5
    /// puffs each; the per-kind gore-type ladder is constant data, so all
    /// four ids share the census.
    #[test]
    fn w4_g59_the_goblin_quartet_shares_one_census() {
        let w = air();
        let n = corpse(NpcKind::GoblinPeon);
        let mut states = Vec::new();
        for ty in [26, 27, 28, 29] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(53);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:89177-89180` — the Skeletron head/hand: 150 type-26 bone
    /// puffs at 2.5×dir/-2.5 (the largest batch-3 burst); the RedHat gore
    /// gate reads ai[3]/localAI[3], no argument draws.
    #[test]
    fn w4_g59_the_skeletron_pair_draws_one_fifty_bones() {
        let w = air();
        let n = corpse(NpcKind::SkeletronHead);
        let mut states = Vec::new();
        for ty in [35, 36] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(59);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 150);
            assert!(live.iter().all(|d| d.dust_type == 26));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:87932-87935` — the Guide: 50 type-5 puffs at 2.5×dir/-2.5.
    #[test]
    fn w4_g59_the_guide_death_draws_fifty() {
        let w = air();
        let n = corpse(NpcKind::Guide);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        assert!(death_dust(&n, 22, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:85807-85810` — the bat pair (49/51): 15 type-5 puffs at
    /// 2×dir/-2; both ids share the bat arm's dust head.
    #[test]
    fn w4_g59_the_bat_pair_draws_fifteen() {
        let w = air();
        let n = corpse(NpcKind::CaveBat);
        let mut states = Vec::new();
        for ty in [49, 51] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(67);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 15);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:86740-86743` — the Shark: 75 type-5 puffs at 2×dir/-2
    /// (the 75-count arm, the largest type-5 burst).
    #[test]
    fn w4_g59_the_shark_death_draws_seventy_five() {
        let w = air();
        let n = corpse(NpcKind::Shark);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        assert!(death_dust(&n, 65, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 75);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:89712-89715` — the Hornet: 50 type-18 puffs at
    /// `this.alpha`/`this.scale`, SpeedX = dir (the type-18 shape, batch
    /// 2's trio census on a new kind).
    #[test]
    fn w4_g59_the_hornet_death_draws_fifty_type_eighteens() {
        let w = air();
        let n = corpse(NpcKind::Hornet);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(73);
        assert!(death_dust(&n, 42, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 18));
    }

    /// `NPC.cs:86381-86384` / `:89803-89806` — the Piranha (10) and
    /// Voodoo Demon (50): the 2×dir/-2 type-5 shape at two counts.
    #[test]
    fn w4_g59_the_piranha_and_voodoo_counts() {
        let w = air();
        for (kind, ty, count, seed) in [
            (NpcKind::Piranha, 58, 10usize, 79),
            (NpcKind::VoodooDemon, 66, 50, 83),
        ] {
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    /// `NPC.cs:88921-88932` — the Wyvern head: 10 type-16 puffs (scale
    /// 1.5, vel×2, noGravity) then the gore tail's ARGUMENT draws — the
    /// Next(1,4) count + Next(11,14) per gore. The hand-rolled ladder
    /// pins the exact draw order (the gore bodies are d55's, NOT drawn).
    #[test]
    fn w4_g59_the_wyvern_death_consumes_the_gore_argument_draws() {
        let w = air();
        let n = corpse(NpcKind::WyvernHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(89);
        assert!(death_dust(&n, 87, &mut dust, &ctx(), &mut rng, &w));
        // The twin: the dust ladder + the gore argument draws by hand.
        let (ww, wh) = n.size();
        let (ww, wh) = (ww as i32, wh as i32);
        let mut twin = UnifiedRandom::new(89);
        for _ in 0..10 {
            let _px = twin.next_max(ww.max(5) - 4);
            let _py = twin.next_max(wh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let count = twin.next_range(1, 4);
        for _ in 0..count {
            let _ty = twin.next_range(11, 14);
        }
        assert_eq!(rng, twin, "the wyvern spends the dust + gore-arg ladder");
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 16 && d.no_gravity));
    }

    /// `NPC.cs:87446-87449` — the Leech head: 10 type-5 puffs at dir/-1
    /// (the SpeedY stays -1, the leech arm's own value — NOT the -2 the
    /// other 10-count arms use).
    #[test]
    fn w4_g59_the_leech_death_keeps_speed_y_minus_one() {
        let w = air();
        let n = corpse(NpcKind::LeechHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        assert!(death_dust(&n, 117, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        // SpeedY = -1 + the ±2 jitter → [-3, 1].
        assert!(live.iter().all(|d| d.velocity.y >= -3.0 && d.velocity.y <= 1.0));
        // SpeedX = dir(-1) + jitter → [-3, 1].
        assert!(live.iter().all(|d| d.velocity.x >= -3.0 && d.velocity.x <= 1.0));
    }

    /// `NPC.cs:87880-87883` / `:87912-87915` — the town families
    /// (106/107/108 and 123/124): 50 type-5 puffs at 2.5×dir/-2.5 each;
    /// all five ids share the census.
    #[test]
    fn w4_g59_the_town_families_share_one_census() {
        let w = air();
        let n = corpse(NpcKind::Wizard);
        let mut states = Vec::new();
        for ty in [106, 107, 108, 123, 124] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(101);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:86459-86462` — the eye pair (190/191): 50 type-5 puffs at
    /// 2×dir/-2 (the demon-eye arm's shape on the new kinds).
    #[test]
    fn w4_g59_the_eye_pair_draws_fifty() {
        let w = air();
        let n = corpse(NpcKind::CataractEye);
        let mut states = Vec::new();
        for ty in [190, 191] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(103);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:86459-86462` — the eye trio (192/193/194): the demon-eye
    /// arm's 50-count census on the remaining eye kinds.
    #[test]
    fn w4_g59_the_eye_trio_draws_fifty() {
        let w = air();
        let n = corpse(NpcKind::DialatedEye);
        let mut states = Vec::new();
        for ty in [192, 193, 194] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(107);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:88458-88461` — the pirate family (212-216): 50 type-5
    /// puffs each; the per-kind gore ladder is constant data, so all five
    /// ids share the census.
    #[test]
    fn w4_g59_the_pirate_family_shares_one_census() {
        let w = air();
        let n = corpse(NpcKind::PirateDeckhand);
        let mut states = Vec::new();
        for ty in [212, 213, 214, 215, 216] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(109);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:88378-88381` / `:88401-88404` — the Dye Trader / Party
    /// Girl: 50 type-5 puffs at 2.5×dir/-2.5 each.
    #[test]
    fn w4_g59_the_dye_and_party_girls_draw_fifty() {
        let w = air();
        for (kind, ty) in [(NpcKind::DyeTrader, 207), (NpcKind::PartyGirl, 208)] {
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(113);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    /// `NPC.cs:87491-87494` — the Queen Bee: 50 type-18 puffs at
    /// `this.alpha`/`this.scale`, SpeedX = dir (the type-18 shape).
    #[test]
    fn w4_g59_the_queen_bee_death_draws_fifty_type_eighteens() {
        let w = air();
        let n = corpse(NpcKind::QueenBee);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        assert!(death_dust(&n, 222, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 18));
    }

    /// `NPC.cs:85038-85041` — the Umbrella Slime: 50 type-4 puffs at
    /// 2×dir/-2, alpha 100, tinted (0,80,255,100).
    #[test]
    fn w4_g59_the_umbrella_slime_death_draws_tinted_fours() {
        let w = air();
        let n = corpse(NpcKind::UmbrellaSlime);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        assert!(death_dust(&n, 225, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        for d in &live {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.color, Color::new(0, 80, 255, 100));
        }
    }

    /// `NPC.cs:88086-88089` / `:88109-88112` — the Painter / Witch
    /// Doctor: 50 type-5 puffs at 2.5×dir/-2.5 each.
    #[test]
    fn w4_g59_the_painter_and_witch_doctor_draw_fifty() {
        let w = air();
        for (kind, ty) in [(NpcKind::Painter, 227), (NpcKind::WitchDoctor, 228)] {
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(137);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }
}
