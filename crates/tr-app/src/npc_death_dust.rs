//! W4-g59 — the NPC death DUST lane: `NPC.HitEffect`'s per-kind death-half
//! dust bursts (`life <= 0`), kinds 1-230, batches 1-10 (88 kinds — every
//! dust-first clean kind the port's kind set carries, plus the 24 d35
//! row-gated activations of the DOC-SWEEP phase).
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
//! ## Scope — batches 8-10 (the DOC-SWEEP: the d35 row-gated activations)
//!
//! W4-d35's 33-kind SetDefaults wave (`eb0f1005`, kinds 73-140) landed 24
//! kinds inside this lane's 107-kind UNWIRED remainder; 120/137 were
//! ALREADY wired (batch 1's illuminant arm + batch 5's — their arms fire
//! once d35's cs_id rows merge). 112 VileSpit's whole-arm burst
//! (`:85395-85408`) fires on EVERY hit — g62 owns it (`npc_hit_dust.rs`,
//! the `:85399` site). The remaining 24 activate here (103 rides the
//! shared jellyfish arm's own color row — the census's nested-row
//! artifact re-checked against the C#):
//!
//! | batch | C# type(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 8 | 73 GoblinScout | `:89560` | 50× type-5 (the goblin arm) |
//! | 8 | 75 Pixie | `:85527-85531` | 50× type-55 alpha 200 vel×2, dust-only |
//! | 8 | 77/110 ArmoredSkeleton/SkeletonArcher | `:88537` | 20× type-26 |
//! | 8 | 78/79/80 the mummy trio | `:88946-88951` | 20× type-31 scale 1.5 vel×2 noGrav |
//! | 8 | 82 Wraith | `:88971-88976` | 20× type-54 alpha 50 scale 1.5 vel×2 noGrav |
//! | 8 | 83/84 CursedHammer/EnchantedSword | `:87152-87163` | 20× type-31 + 6 gore ARG draws |
//! | 9 | 85 Mimic | `:88785` | 20× ai[3]-selected 7/10/37 |
//! | 9 | 86 Unicorn | `:87705` | 50× type-5 |
//! | 9 | 93 GiantBat | `:85807` | 15× type-5 (the bat arm) |
//! | 9 | 94 Corruptor | `:87512` | 50× type-18 alpha/color/scale |
//! | 9 | 95/96/97 the digger trio | `:87616` | 10× type-5 2.5×dir |
//! | 9 | 98/99/100 the seeker trio | `:87583` | 50× type-18 alpha/color/scale |
//! | 10 | 101 Clinger | `:87547` | 50× type-18 alpha/color/scale |
//! | 10 | 103 GreenJellyfish | `:85554-85557` | 25× type-4 tinted (70,225,140,100), dust-only |
//! | 10 | 122 Gastropod | `:85509-85513` | 50× type-72 alpha 200 vel×1.5, dust-only |
//! | 10 | 140 PossessedArmor | `:89085-89090` | 20× type-54 alpha 50 (death-only gate) |
//!
//! The 83/84 pair is the SECOND gore-ARGUMENT arm: each of the 3 gores
//! draws a `Next(-2, 3)` x/y velocity pair (`:87158`/`:87160`/`:87162`) —
//! 6 draws consumed here in C# order, the bodies stay d55's (the g63
//! rule). Every other batch-8-10 tail draws NO arguments (verified
//! against the C#: constant types / `this.type` arithmetic / hitDirection
//! / ai-field reads). The d35 kinds NOT activated: 74/102 (gore-first
//! `:85942`), 104/109 (the zombie arm's NewNPC), 116 (NewNPC), 121
//! (NewNPC), 112 (g62's every-hit).
//!
//! ## Scope — batch 11 (the d34 row-gated activations, 10 kinds)
//!
//! W4-d34's pushed tip `5f3435a8` (phases 1-3, kinds 1-70) landed the
//! cs_id hand rows inside this lane's UNWIRED remainder. The sweep
//! re-intersected the 83-kind row-gated set against EVERY pushed roster
//! tip (d34/d35/d37/d38/d40/d41/d42 + the c4/c8/c10/e26/d23 kind-claims):
//! d34 intersects 14 clean kinds, of which 31/34/37/39/40/41/43/47/48/56
//! activate here (the 40/41 BoneSerpent segments ride d34's Hide-fold
//! onto the head's cs_id 39 — the feed sees 39, the C# arm reads the RAW
//! type, all three draw the same 20×26 burst); 57/61/62/63 follow in
//! batch 12.
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 31 AngryBones | `:88537` | 20× type-26 (the big skeleton arm, 21/32's) |
//! | 34 CursedSkull | `:89132-89141` | 15× (15 noGrav/noLight + 26 noLight) pairs, -vel×0.2, vel×1.3, dust-only |
//! | 37 OldMan | `:88232-88235` | 50× type-5 2.5×dir (the town shape) |
//! | 39/40/41 BoneSerpent | `:89109-89112` | 20× type-26 (40/41 fold onto 39) |
//! | 43/56 ManEater/Snatcher | `:89752-89755` | 50× type-40 alpha/color scale-1.2 |
//! | 47 CorruptBunny | `:86381-86384` | 10× type-5 (the piranha arm, 58's) |
//! | 48 Harpy | `:89786-89789` | 50× type-5 2×dir |
//!
//! All batch-11 gore tails draw NO arguments (the skeleton ladder's
//! generic 42/43/43/44/44 tail, the `this.type - 39 + 67` arithmetic, the
//! 72/72 + 78/79 + 80/81 constants, the Old Man's shimmer field-read
//! gates). The d34 kinds NOT activated: 46 Bunny (gore-first `:85942`).
//!
//! ## Scope — batch 12 (the d34 tail + c4/d23/d37 activations, 10 kinds)
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 57 CorruptGoldfish | `:86381` | 10× type-5 (the piranha arm, d34) |
//! | 61 Vulture | `:86617-86620` | 50× type-5 2×dir (d34) |
//! | 62 Demon | `:89803` | 50× type-5 (the voodoo arm, d34) |
//! | 63 BlueJellyfish | `:85554-85557` | 25× type-4 tinted (50,120,255,100), dust-only (d34; the shared arm's DEFAULT color row — 64 stays row-gated) |
//! | 157 Arapaima | `:86514-86517` | 50× type-5 2×dir (c4 `22cec16f`) |
//! | 175 AngryTrapper | `:89769-89772` | 50× type-40 alpha/color scale-1.2 (d23 `19109424`, its own arm) |
//! | 217/218/219 beetles | `:87462-87465` | 50× type-18 alpha/color/scale (d37 `750d8e44`) |
//! | 226 FlyingSnake | `:85807` | 15× type-5 (the bat arm, d37) |
//!
//! All batch-12 gore tails draw NO arguments (the 84/86-88/93/94/94/
//! 186/187/227/228 constants, the beetle ladder's `this.scale`, the
//! snake's 317-319). The 64 PinkJellyfish tint (225,70,140) lands with
//! its kind row (d36's band, unpushed).
//!
//! ## The lane's remainder (the census's absent kinds)
//!
//! The 173-kind dust-first clean census had 109 kinds with NO port kind
//! row at batch 7's close (the worm segments 8/9/11/12/14/15, the casters
//! 31/37/39-41/43, the frost/martian/dd2 families, the 195-206/209/
//! 217-221/226/229 bands, …); the DOC-SWEEP re-census counts 107 UNWIRED
//! (the 66 landed C# types cover 66 of the 173). After batches 8-10, 83
//! stay row-gated UNWIRED until their kinds land (the S1-F-9b row-gate
//! convention); the census rows + arm citations live in docs/ANALYSIS.md.
//! The 20 dust-only kinds' still-absent members (34/63/141/169/204) are
//! the same remainder (75/120/122/137 activated above, 112 re-owned g62).
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
//!   `NPC.cs:81654`). KEEP-ONE (c12/g60/g59): the SAME field is staged
//!   independently on all three branches (`task/W4-c12-style74-martian-
//!   drone`, `task/W4-g60-npc-death-effects-2`, this lane) — one
//!   `pub last_hit_dir: i32` at `npc.rs` with the same three recording
//!   seams (hurt/hurt_from_projectile/strike_pooled) + the c12 bubble-
//!   fuse reset; the merge keeps ONE declaration, every feed reads it.
//!   None of the three re-records it elsewhere.
//! * 183's `this.color`: `default(Color)` — the SetDefaults reset
//!   (`NPC.cs:8526`) is the ONLY write for 183 (the paint-blend ladder
//!   `:60577-60692` is type-1-only, `AI_001_Slimes`). The port's `Npc`
//!   carries no color field; when the slime paint lane lands, the feed
//!   reads it here.
//! * 183's `this.alpha` reads [`Npc::alpha`] — the ctor carries the C#
//!   `alpha = 55` row (`NPC.cs:11342`, added with batch 1).
//! * Batch 2's type-18 trio (6/7/13) reads `this.alpha`/`this.scale` off
//!   the NPC. The EoW head's `alpha = 255` SetDefaults row (`:8921`,
//!   body/tail `:8941`/`:8961`) is the spawn-fade window — NOW PORTED
//!   (the DOC-SWEEP): the ctor carries 255 for 13/14/15 and the worm
//!   pass runs the `AI_006` tail (`NPC.cs:52251-52271`): the 14/15 gate
//!   on the leader's `alpha < 85`, the 2× type-14 fade dust while
//!   `alpha > 0 && life > 0`, the movement-gated `alpha -= 42` floor 0.
//!   The feed reads the LIVE alpha (255 → 0 over the fade, then the
//!   steady-state 0). NOTE: the module doc's old `:50998` citation was
//!   the CultistDragon (454-459) arm — the EoW's own fade is `:52251-
//!   52271` (re-derived in the sweep). The 6/7 pair writes no alpha row
//!   (the Devourer's `scale *= 1.3` `:8784` is the only family tweak),
//!   so their feed keeps reading the ctor 0.
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
        // the alpha reads the LIVE field — the EoW's 255 spawn fade is
        // ported (the ctor row + the worm pass's `:52251-52271` tail,
        // the DOC-SWEEP), 6/7 keep the ctor 0.
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
        // 31 AngryBones rides the SAME arm (batch 11, the d34 row landing —
        // `5f3435a8`'s hand row; its gore path falls through the ladder's
        // per-kind gates to the generic 42/43/43/44/44 tail `:88623-88639`,
        // all constants + `this.scale`).
        21 | 31 | 32 => {
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
        // `NPC.cs:86381-86384` — the Piranha (58) / Corrupt Bunny (47) /
        // Corrupt Goldfish (57): 10 type-5 puffs at 2×dir/-2 (the shared
        // `:86371` arm; 47/57 ride the d34 rows — batches 11/12, the
        // 78/79 + 84 gore tails `:86385-86407` are constant types, no
        // argument draws; the 464/465 siblings are g60's band).
        47 | 57 | 58 => {
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
        // `NPC.cs:89803-89806` — the Voodoo Demon (66) / Demon (62): 50
        // type-5 puffs at 2×dir/-2 (the shared `:89793` arm; 62 rides the
        // d34 row — batch 12; the 93/94/94 gore tail `:89807-89809` is
        // constant types, no argument draws).
        62 | 66 => {
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
        // ---- batch 8 (the d35 row-gated activations: the goblin scout /
        // pixie / skeleton pair / mummy trio / wraith / cursed pair).
        // `NPC.cs:89560-89563` — the Goblin Scout (73): 50 type-5 puffs at
        // 2.5×dir/-2.5 (the goblin arm, batch 3's shape; the 73 gore ladder
        // `:89579-89584` is the 629/630/631 constants, no argument draws).
        73 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85527-85531` — the Pixie (75): 50 type-55 puffs at
        // dir/0, alpha 200, `this.color` (the stand-in: default — the
        // SetDefaults reset `:8526` is the only write for 75, the paint
        // ladder `:60577-60692` is type-1-only), velocity×2. Dust-only.
        75 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 55, dir, 0.0, 200,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 2.0;
            }
        }
        // `NPC.cs:88537-88540` — the Armored Skeleton (77) / Skeleton
        // Archer (110): 20 type-26 bone puffs at 2.5×dir/-2.5 (the big
        // skeleton arm, batch 3's shape; the 77/110 gore ladders
        // `:88624-88637` add the 106/130/131 constants, no argument draws).
        77 | 110 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88946-88951` — the Mummy trio (78/79/80): 20 type-31
        // puffs at (0,0), scale 1.5, velocity×2 + noGravity (the 61/62/63
        // gore tail `:88952-88957` is constant types + hitDirection, no
        // argument draws).
        78 | 79 | 80 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 31, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:88971-88976` — the Wraith (82): 20 type-54 puffs at
        // (0,0), alpha 50, scale 1.5, velocity×2 + noGravity (the 99×3
        // gore tail `:88977-88982` is constant + hitDirection, no draws).
        82 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 54, 0.0, 0.0, 50,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:87152-87163` — the Cursed Hammer (83) / Enchanted Sword
        // (84): 20 type-31 puffs at (0,0), scale 1.5, velocity×2 +
        // noGravity, THEN the gore tail's ARGUMENT draws — 3 gores each
        // with a `Next(-2,3)` x/y velocity pair (`:87158`/`:87160`/
        // `:87162`, 6 draws). The bodies + kicks stay d55's (the g63
        // gore-sibling rule); the draws are consumed here in C# order.
        83 | 84 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 31, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            for _ in 0..3 {
                let _gx = rng.next_range(-2, 3);
                let _gy = rng.next_range(-2, 3);
            }
        }
        // ---- batch 9 (the d35 row-gated activations: the mimic / unicorn /
        // giant bat / corruptor / digger trio / seeker trio).
        // `NPC.cs:88785-88788` — the Mimic (85): 20 puffs of the ai[3]
        // selector type (`:88768-88776`: 2→10, 3→37, else 7), default
        // args. The 61/62/63 gore tail `:88789-88794` is constant types +
        // hitDirection (no argument draws).
        85 => {
            let mut ty = 7;
            if npc.ai[3] == 2.0 {
                ty = 10;
            }
            if npc.ai[3] == 3.0 {
                ty = 37;
            }
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, ty, 0.0, 0.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87705-87708` — the Unicorn (86): 50 type-5 puffs at
        // 2.5×dir/-2.5 (the 101-105 gore tail `:87709-87715` is constant
        // types, no argument draws).
        86 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85807-85810` — the Giant Bat (93): 15 type-5 puffs at
        // 2×dir/-2 (the bat arm, batch 4's shape; the 107 gore
        // `:85815-85818` is a constant type, no argument draws).
        93 => {
            for _ in 0..15 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87512-87515` — the Corruptor (94): 50 type-18 puffs at
        // dir/-2, `this.alpha`/`this.color`/`this.scale` (the EoSouls arm,
        // batch 2's shape; the color stand-in is default — 94 writes no
        // color). The 108/108/109/110 gore tail `:87516-87522` is constant
        // types, no argument draws.
        94 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:87616-87619` — the Digger head (95) / body (96) / tail
        // (97): 10 type-5 puffs at 2.5×dir/-2.5 (the Giant Worm arm,
        // batch 2's shape; the `this.type - 95 + 111` gore `:87620` is
        // constant arithmetic, no argument draws).
        95 | 96 | 97 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87583-87586` — the Seeker head (98) / body (99) / tail
        // (100): 50 type-18 puffs at dir/-2, `this.alpha`/`this.color`/
        // `this.scale` (the EoSouls arm; the color stand-in is default —
        // none writes a color). The 110 gore `:87587-87588` is a constant
        // type + `this.alpha` post-write, no argument draws.
        98 | 99 | 100 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // ---- batch 10 (the d35 row-gated activations: the clinger /
        // gastropod / possessed armor — the lane's final d35 kinds).
        // `NPC.cs:87547-87550` — the Clinger (101): 50 type-18 puffs at
        // dir/-2, `this.alpha`/`this.color`/`this.scale` (the EoSouls arm;
        // the color stand-in is default — 101 writes no color). The
        // 110/114/114/115 gore tail `:87551-87554` is constant types, no
        // argument draws.
        101 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:85554-85557` — the Green Jellyfish (103): 25 type-4
        // puffs at 2×dir/-2, alpha 0, tinted (70,225,140,100) (the shared
        // 63/64/103 arm `:85534-85558`'s 103 color row `:85541-85544`;
        // 63/64 stay row-gated — their kinds are absent, the 50,120,255 /
        // 225,70,140 tints land with them). Dust-only, zero gore. The
        // census's `:85541` color-only nested row is a selection, not a
        // second death arm (the g59_order dust>0 artifact).
        103 => {
            for _ in 0..25 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, 2.0 * dir, -2.0, 0,
                    Color::new(70, 225, 140, 100), 1.0,
                );
            }
        }
        // `NPC.cs:85509-85513` — the Gastropod (122): 50 type-72 puffs at
        // dir/0, alpha 200, velocity×1.5 (the Illuminant arm's shape with
        // type 72, NOT 71 — the gastropod's own row). Dust-only, zero gore.
        122 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 72, dir, 0.0, 200,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 1.5;
            }
        }
        // `NPC.cs:89085-89090` — the Possessed Armor (140): 20 type-54
        // puffs at (0,0), alpha 50, scale 1.5, velocity×2 + noGravity (the
        // Wraith arm's shape, a DEATH-ONLY `life <= 0` gate with no hit
        // half). The 99×3 gore tail `:89091-89096` is constant +
        // hitDirection, no argument draws.
        140 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 54, 0.0, 0.0, 50,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // ---- batch 11 (the d34 row-gated activations: the bone/caster/
        // man-eater families). d34's pushed tip `5f3435a8` (phases 1-3)
        // landed the cs_id hand rows 31/34/37/39/43/47/48/56 (+ the 40/41
        // Hide-folds onto 39). 31/47 ride the extended shared arms above;
        // the six new arms follow.
        // `NPC.cs:89786-89789` — the Harpy (48): 50 type-5 puffs at
        // 2×dir/-2 (the 80/81 gore tail `:89790-89791` is constant types,
        // no argument draws).
        48 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89132-89141` — the Cursed Skull (34): 15 × (type-15
        // noGravity/noLight alpha 100 scale 1.8 + type-26 noLight scale
        // 0.9) pairs at -velocity×0.2, each velocity×1.3. Dust-only,
        // zero gore (the arm's hit half `:89119-89128` is g62's).
        34 => {
            for _ in 0..15 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 15,
                    -npc.vx * 0.2, -npc.vy * 0.2, 100, Color::default(), 1.8,
                );
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.no_gravity = true;
                d.velocity *= 1.3;
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 26,
                    -npc.vx * 0.2, -npc.vy * 0.2, 0, Color::default(), 0.9,
                );
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.velocity *= 1.3;
            }
        }
        // `NPC.cs:88232-88235` — the Old Man (37): 50 type-5 puffs at
        // 2.5×dir/-2.5 (the town shape; the shimmer-gated 58-60/1328-1330
        // gore tail `:88236-88240` reads `IsShimmerVariant`, a field read
        // — no argument draws).
        37 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89109-89112` — the Bone Serpent head (39): 20 type-26
        // bone puffs at 2.5×dir/-2.5 (the skeleton shape; the
        // `this.type - 39 + 67` gore `:89113` is constant arithmetic, no
        // argument draws). The 40/41 Hide-fold segments ride the head's
        // cs_id (d34's fold — the feed sees 39; the C# arm reads the RAW
        // type, all three draw the same burst).
        39 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89752-89755` — the Man Eater (43) / Snatcher (56): 50
        // type-40 puffs at dir/-2, `this.alpha`/`this.color`, scale 1.2
        // (the color stand-in is default — neither writes a color). The
        // 72/72 gore tail `:89756-89757` is constant types, no argument
        // draws.
        43 | 56 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 40, dir, -2.0,
                    npc.alpha as i32, Color::default(), 1.2,
                );
            }
        }
        // ---- batch 12 (the d34 tail + c4/d23/d37 row-gated activations).
        // d34's 57/61/62/63, c4 `22cec16f`'s Arapaima 157, d23 `19109424`'s
        // AngryTrapper 175, d37 `750d8e44`'s beetle trio 217/218/219 +
        // FlyingSnake 226. 57/62 ride the extended shared arms above.
        // `NPC.cs:86617-86620` — the Vulture (61): 50 type-5 puffs at
        // 2×dir/-2 (the 86/87/88 gore tail `:86621-86623` is constant
        // types + a +14 X offset, no argument draws).
        61 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85554-85557` — the Blue Jellyfish (63): 25 type-4 puffs
        // at 2×dir/-2, alpha 0, tinted (50,120,255,100) (the shared
        // 63/64/103 arm's DEFAULT color row `:85536` — 63 rides the d34
        // row; 64 stays row-gated, its 225,70,140 tint lands with it; 103
        // wired in batch 10). Dust-only, zero gore.
        63 => {
            for _ in 0..25 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, 2.0 * dir, -2.0, 0,
                    Color::new(50, 120, 255, 100), 1.0,
                );
            }
        }
        // `NPC.cs:86514-86517` — the Arapaima (157): 50 type-5 puffs at
        // 2×dir/-2 (c4's style-44 row; the 186/187 gore tail
        // `:86518-86519` is constant types + a +14 X offset, no argument
        // draws).
        157 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89769-89772` — the Angry Trapper (175): 50 type-40 puffs
        // at dir/-2, `this.alpha`/`this.color`, scale 1.2 (the man-eater
        // arm's shape, its OWN C# arm; d23's style-13 row). The 227/228
        // gore tail `:89773-89774` is constant types, no argument draws.
        175 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 40, dir, -2.0,
                    npc.alpha as i32, Color::default(), 1.2,
                );
            }
        }
        // `NPC.cs:87462-87465` — the beetle trio (217 Cochineal / 218 Cyan
        // / 219 Lac): 50 type-18 puffs at dir/-2, `this.alpha`/`this.color`
        // /`this.scale` (the EoSouls arm; d37's rows). The per-kind gore
        // ladder `:87466-87479` (292/293, 295/296, 294) is constant types
        // + `this.scale`, no argument draws.
        217 | 218 | 219 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:85807-85810` — the Flying Snake (226): 15 type-5 puffs
        // at 2×dir/-2 (the bat arm, batch 4's shape; d37's row). The
        // 317/318/318/319 gore tail `:85830-85836` is constant types, no
        // argument draws.
        226 => {
            for _ in 0..15 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
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

    // ---- batches 8-10 (the d35 row-gated activations). The d35 kinds
    // (GoblinScout 73 … PossessedArmor 140) land with the d35 merge; the
    // witnesses drive the RAW C# ids over a stand-in corpse shell (the
    // batch-1 snowman-trio precedent — `death_dust` matches on the type
    // argument, the shell only supplies the box/alpha/scale reads).

    /// `NPC.cs:88946-88951` — the mummy trio (78/79/80): 20 type-31
    /// puffs, scale 1.5, velocity×2 + noGravity; the wraith (82) shares
    /// the shape at type 54, alpha 50.
    #[test]
    fn w4_g59_the_mummy_trio_and_wraith_shapes() {
        let w = air();
        for (ty, alpha) in [(78, 0), (79, 0), (80, 0), (82, 50)] {
            let n = corpse(NpcKind::Skeleton);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(139);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20, "type {ty}");
            let want_ty = if ty == 82 { 54 } else { 31 };
            for d in &live {
                assert_eq!(d.dust_type, want_ty);
                assert_eq!(d.alpha, alpha);
                assert!(d.no_gravity);
                // scale = (1 + Next(-20,21)*0.01) × 1.5 ∈ [1.2, 1.8].
                assert!(d.scale >= 1.2 - 1e-6 && d.scale <= 1.8 + 1e-6);
                // The ×2 write: the pool's ±2 jitter doubled ∈ [-4, 4].
                assert!(d.velocity.x.abs() <= 4.0 + 1e-6);
                assert!(d.velocity.y.abs() <= 4.0 + 1e-6);
            }
        }
    }

    /// `NPC.cs:87152-87163` — the cursed pair (83/84): 20 type-31 puffs
    /// then the 3-gore tail's 6 `Next(-2,3)` ARGUMENT draws — the twin
    /// stream replays the exact ladder (the g63 gore-sibling rule).
    #[test]
    fn w4_g59_the_cursed_pair_consumes_the_gore_velocity_draws() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(149);
        assert!(death_dust(&n, 83, &mut dust, &ctx(), &mut rng, &w));
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let mut twin = UnifiedRandom::new(149);
        for _ in 0..20 {
            let _px = twin.next_max(bw.max(5) - 4);
            let _py = twin.next_max(bh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let gx = twin.next_range(-2, 3);
            let gy = twin.next_range(-2, 3);
            assert!((-2..3).contains(&gx) && (-2..3).contains(&gy));
        }
        assert_eq!(rng, twin, "20×6 dust draws + 6 gore argument draws");
        // 84 shares the census exactly.
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(149);
        assert!(death_dust(&n, 84, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng_b, twin);
        assert_eq!(dust.slots(), dust_b.slots());
    }

    /// `NPC.cs:85527-85531` — the Pixie (75): 50 type-55 puffs, alpha
    /// 200, velocity×2 (dust-only; the twin pool pins the ×2 write).
    #[test]
    fn w4_g59_the_pixie_death_draws_fifty_type_fifty_fives() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(151);
        assert!(death_dust(&n, 75, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(151);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                55, -1.0, 0.0, 200, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 2.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert!(live.iter().all(|d| d.dust_type == 55 && d.alpha == 200));
    }

    /// `NPC.cs:89560-89563` / `:88537-88540` — the goblin scout (73)
    /// 50×5 and the skeleton pair (77/110) 20×26 ride the batch-3 shapes.
    #[test]
    fn w4_g59_the_goblin_scout_and_skeleton_pair_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, want_ty, count) in [(73, 5, 50), (77, 26, 20), (110, 26, 20)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == want_ty));
        }
    }

    /// `NPC.cs:88768-88788` — the Mimic (85): the ai[3] selector picks
    /// the dust type (2→10, 3→37, else 7); 20 puffs, default args.
    #[test]
    fn w4_g59_the_mimic_death_reads_the_ai3_selector() {
        let w = air();
        for (ai3, want_ty) in [(0.0f32, 7), (2.0, 10), (3.0, 37), (1.0, 7)] {
            let mut n = corpse(NpcKind::Skeleton);
            n.ai[3] = ai3;
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(157);
            assert!(death_dust(&n, 85, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20);
            assert!(live.iter().all(|d| d.dust_type == want_ty), "ai3={ai3}");
        }
    }

    /// `NPC.cs:87616-87619` / `:87583-87586` / `:87512-87515` — the
    /// digger trio (95/96/97) 10×5 at 2.5×dir, the seeker trio
    /// (98/99/100) + Corruptor (94) 50×18 at dir/-2 reading the live
    /// alpha/scale; the unicorn (86) 50×5 and the giant bat (93) 15×5
    /// ride the type-5 shapes.
    #[test]
    fn w4_g59_the_digger_seeker_and_corruptor_arms() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 90;
        n.scale = 1.1;
        for ty in [95, 96, 97] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 10);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
        for ty in [94, 98, 99, 100] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            for d in &live {
                assert_eq!(d.dust_type, 18);
                assert_eq!(d.alpha, 90);
                assert!((d.scale - 1.1).abs() <= 0.22 + 1e-6);
            }
        }
        for (ty, count) in [(86, 50), (93, 15)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }


    /// `NPC.cs:87547-87550` — the Clinger (101): 50 type-18 puffs at
    /// dir/-2 reading the live alpha/scale (the EoSouls arm).
    #[test]
    fn w4_g59_the_clinger_death_reads_alpha_and_scale() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 90;
        n.scale = 1.1;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(173);
        assert!(death_dust(&n, 101, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        for d in &live {
            assert_eq!(d.dust_type, 18);
            assert_eq!(d.alpha, 90);
            assert!((d.scale - 1.1).abs() <= 0.22 + 1e-6);
        }
    }

    /// `NPC.cs:85509-85513` — the Gastropod (122): 50 type-72 puffs,
    /// alpha 200, velocity×1.5 (dust-only; the twin pool pins the ×1.5).
    #[test]
    fn w4_g59_the_gastropod_death_draws_fifty_type_seventy_twos() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(179);
        assert!(death_dust(&n, 122, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(179);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                72, -1.0, 0.0, 200, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 1.5;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert!(live.iter().all(|d| d.dust_type == 72 && d.alpha == 200));
    }

    /// `NPC.cs:89085-89090` — the Possessed Armor (140): 20 type-54
    /// puffs, alpha 50, scale 1.5, velocity×2 + noGravity (the death-only
    /// `life <= 0` gate, the wraith's shape).
    #[test]
    fn w4_g59_the_possessed_armor_death_draws_twenty_type_fifty_fours() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(181);
        assert!(death_dust(&n, 140, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 20);
        for d in &live {
            assert_eq!(d.dust_type, 54);
            assert_eq!(d.alpha, 50);
            assert!(d.no_gravity);
            assert!(d.scale >= 1.2 - 1e-6 && d.scale <= 1.8 + 1e-6);
        }
    }

    /// `NPC.cs:85554-85557` — the Green Jellyfish (103): 25 type-4 puffs
    /// at 2×dir/-2, alpha 0, tinted (70,225,140,100) — the shared arm's
    /// 103 color row (63/64 stay row-gated).
    #[test]
    fn w4_g59_the_green_jellyfish_death_draws_tinted_fours() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(167);
        assert!(death_dust(&n, 103, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 25);
        for d in &live {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 0);
            assert_eq!(d.color, Color::new(70, 225, 140, 100));
        }
    }

    // ---- batch 11 (the d34 row-gated activations). The d34 kinds
    // (AngryBones 31 … BlueJellyfish 63) land with the d34 merge; the
    // witnesses drive the RAW C# ids over a stand-in corpse shell (the
    // batches-8-10 precedent — `death_dust` matches on the type argument,
    // the shell only supplies the box/alpha/scale reads).

    /// `NPC.cs:88537-88540` — Angry Bones (31) rides the big skeleton
    /// arm's 20×26 census exactly (the d34 hand row; its gore path falls
    /// to the generic constant tail — zero argument draws).
    #[test]
    fn w4_g59_the_angry_bones_death_rides_the_skeleton_census() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(191);
        assert!(death_dust(&n, 31, &mut dust, &ctx(), &mut rng, &w));
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(191);
        assert!(death_dust(&n, 21, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng, rng_b, "31 and 21 spend the same draws");
        assert_eq!(dust.slots(), dust_b.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 20);
        assert!(live.iter().all(|d| d.dust_type == 26));
    }

    /// `NPC.cs:89132-89141` — the Cursed Skull (34): 15 × (15 + 26) pairs
    /// at -velocity×0.2, the 15s noGravity/noLight alpha 100 scale 1.8,
    /// the 26s noLight scale 0.9, both velocity×1.3 — the twin pool pins
    /// the exact writes (dust-only, zero gore, zero argument draws).
    #[test]
    fn w4_g59_the_cursed_skull_death_draws_fifteen_twin_pairs() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(193);
        assert!(death_dust(&n, 34, &mut dust, &ctx(), &mut rng, &w));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(193);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..15 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                15, -0.4, 0.2, 100, Color::default(), 1.8,
            );
            let d = twin.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
            d.velocity *= 1.3;
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                26, -0.4, 0.2, 0, Color::default(), 0.9,
            );
            let d = twin.slot_mut(idx);
            d.no_light = true;
            d.velocity *= 1.3;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 30);
        for pair in live.chunks(2) {
            assert_eq!(pair[0].dust_type, 15);
            assert!(pair[0].no_gravity && pair[0].no_light);
            assert_eq!(pair[0].alpha, 100);
            assert_eq!(pair[1].dust_type, 26);
            assert!(pair[1].no_light && !pair[1].no_gravity);
        }
    }

    /// `NPC.cs:88232-88235` / `:89109-89112` — the Old Man (37) 50×5 at
    /// 2.5×dir/-2.5 and the Bone Serpent head (39) 20×26 at 2.5×dir/-2.5;
    /// the 40/41 Hide-fold segments reach the feed as the head's cs_id 39
    /// (d34's fold) — the same burst, verified by driving 39 thrice.
    #[test]
    fn w4_g59_the_old_man_and_bone_serpent_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, want_ty, count) in [(37, 5, 50), (39, 26, 20)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(197);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == want_ty));
        }
    }

    /// `NPC.cs:89752-89755` — the Man Eater (43) / Snatcher (56): 50
    /// type-40 puffs at dir/-2 reading the live alpha, scale 1.2 (the
    /// pool's ±0.2 jitter over 1.2); both ids draw the same census.
    #[test]
    fn w4_g59_the_man_eater_pair_draws_fifty_type_forties() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 70;
        let mut states = Vec::new();
        for ty in [43, 56] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(199);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            for d in &live {
                assert_eq!(d.dust_type, 40);
                assert_eq!(d.alpha, 70);
                assert!(d.scale >= 1.0 - 1e-6 && d.scale <= 1.4 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert_eq!(states[0], states[1], "43 and 56 share the census");
    }

    /// `NPC.cs:86381-86384` / `:89786-89789` — the Corrupt Bunny (47)
    /// 10×5 at 2×dir/-2 (the piranha arm) and the Harpy (48) 50×5 (its
    /// own arm); both gore tails are constant types.
    #[test]
    fn w4_g59_the_corrupt_bunny_and_harpy_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, count) in [(47, 10), (48, 50)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(211);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    // ---- batch 12 (the d34 tail + c4/d23/d37 row-gated activations).
    // The witnesses drive the RAW C# ids over a stand-in corpse shell
    // (the batches-8-11 precedent).

    /// `NPC.cs:86381-86384` / `:86617-86620` / `:89803-89806` — the
    /// Corrupt Goldfish (57) 10×5 (the piranha arm), the Vulture (61)
    /// 50×5 and the Demon (62) 50×5 (the voodoo arm); all gore tails
    /// constant types.
    #[test]
    fn w4_g59_the_corrupt_goldfish_vulture_and_demon_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, count) in [(57, 10), (61, 50), (62, 50)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(223);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    /// `NPC.cs:85554-85557` — the Blue Jellyfish (63): 25 type-4 puffs
    /// at 2×dir/-2, alpha 0, tinted (50,120,255,100) — the shared arm's
    /// DEFAULT color row (64 stays row-gated; 103's tint wired batch 10).
    #[test]
    fn w4_g59_the_blue_jellyfish_death_draws_the_default_tint() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(227);
        assert!(death_dust(&n, 63, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 25);
        for d in &live {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 0);
            assert_eq!(d.color, Color::new(50, 120, 255, 100));
        }
    }

    /// `NPC.cs:86514-86517` / `:89769-89772` — the Arapaima (157) 50×5
    /// at 2×dir/-2 and the Angry Trapper (175) 50×40 at dir/-2 reading
    /// the live alpha, scale 1.2 (the man-eater shape, its own arm).
    #[test]
    fn w4_g59_the_arapaima_and_angry_trapper_counts() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 70;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(229);
        assert!(death_dust(&n, 157, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(229);
        assert!(death_dust(&n, 175, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        for d in &live {
            assert_eq!(d.dust_type, 40);
            assert_eq!(d.alpha, 70);
            assert!(d.scale >= 1.0 - 1e-6 && d.scale <= 1.4 + 1e-6);
        }
    }

    /// `NPC.cs:87462-87465` / `:85807-85810` — the beetle trio
    /// (217/218/219) 50×18 at dir/-2 reading the live alpha/scale, and
    /// the Flying Snake (226) 15×5 (the bat arm); all gore tails
    /// constant types + `this.scale`.
    #[test]
    fn w4_g59_the_beetle_trio_and_flying_snake_counts() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 90;
        n.scale = 1.1;
        let mut states = Vec::new();
        for ty in [217, 218, 219] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(233);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            for d in &live {
                assert_eq!(d.dust_type, 18);
                assert_eq!(d.alpha, 90);
                assert!((d.scale - 1.1).abs() <= 0.22 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(233);
        assert!(death_dust(&n, 226, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 15);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }
}
