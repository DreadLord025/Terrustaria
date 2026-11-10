//! The projectile pool — the `Main.projectile[1000]` engine (`Projectile.cs`),
//! with the shuriken (`Projectile.cs` type 3, aiStyle 2) as its first AI family.
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - pool / spawn: `NewProjectileSetup` `:10823-10880`, `FindOldestProjectile`
//!   `:10811-10821`, the 1000-slot array `Main.cs:66992`
//! - per-tick bookkeeping: `Projectile.Update` `:17199-17208` (`timeLeft--` →
//!   `Kill`, `penetrate == 0` → `Kill`) with the AI sub-step count
//!   `1 + extraUpdates` (`:16668`)
//! - the PVE damage walk: `Damage` `:12487-12511`, `Damage_PVE` `:12562-12600`
//!   (the target-selection immunity gates), `Damage_PVE_Inner` `:12600+`
//!   (the per-owner gate `:12620-12624`) and the on-hit immunity application
//!   `:14087-14119`
//! - the ID-static system: `IsNPCIndexImmuneToProjectileType` `:604-607`
//!   (a hit is allowed when the stored stamp is `<= GameUpdateCount`), the
//!   stamp write `:13972`
//! - defaults: `timeLeft = 3600` (`:787`), `penetrate = 1`,
//!   `localNPCHitCooldown = -2` (`:464`), `idStaticNPCHitCooldown = -1`
//!   (`:466`), `immunityIdentity = type` (`:782`), the scratch arrays
//!   `ai`/`localAI` zeroed (`:736-740`), `alpha = 0` (`:779`), `scale = 1f`
//!   (`:320`/`:785`) — the aiStyle dispatch's arm-signature contract sits
//!   over the `match data.ai_style` in [`ProjectilePool::update`]
//!
//! NPC-side per-owner i-frames: `NPC.immune[256]` (decremented once per NPC
//! update, `NPC.cs:90454-90459`), stored on [`crate::npc::Npc`].
//!
//! Multiplayer bookkeeping (`key`, `slotGenerations`, net spam) is out of scope
//! for the single-player port — recorded in ANALYSIS "Projectile pool
//! (M4-PROJ-1)".

use std::collections::HashMap;

use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::{TILE_PX, World};
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::npc::Npc;

/// The pool's kind ids are the C# `ProjectileID` values (`ProjectileID.cs`
/// — the M4-PROJ-1 placeholder internal table was migrated to the verbatim
/// ids; the shuriken 3 is the vanilla id the registry's generated rows
/// carry, `itemdata.rs`).
pub const SHURIKEN: u16 = 3;
/// `ProjectileID.WoodenArrow` (`Projectile.cs:807-815`).
pub const WOODEN_ARROW: u16 = 1;
/// `ProjectileID.WoodenArrowHostile` (`ProjectileID.cs:716`) — the
/// Goblin Archer's shot; the box/flight mirror the wooden arrow.
pub const WOODEN_ARROW_HOSTILE: u16 = 81;
/// The Destroyer's segment laser (`NPC.cs`'s `num11 = 100`, `:49886`).
pub const DESTROYER_LASER: u16 = 100;
/// `ProjectileID.Fireball` (`ProjectileID.cs`'s 258) — the Golem's head
/// and free-head fireballs (`NPC.cs:31200`/`:31460`'s `num686`/`num720`).
pub const GOLEM_FIREBALL: u16 = 258;
/// `ProjectileID.EyeBeam` (259) — the Golem's eye beams
/// (`NPC.cs:31282`'s `num698`, `:31565`'s `num729`).
pub const GOLEM_EYE_BEAM: u16 = 259;
/// `ProjectileID.EyeLaser` (83) — the Retinazer's phase-1 laser
/// (`NPC.cs:26628`'s `num412`).
pub const EYE_LASER: u16 = 83;
/// `ProjectileID.CursedFlameHostile` (96) — the Spazmatism's phase-1 spit
/// (`NPC.cs:27223`'s `num451`).
pub const CURSED_FLAME: u16 = 96;
/// `ProjectileID.EyeFire` (101) — the Spazmatism's phase-2 spit
/// (`NPC.cs:27519`'s `num467`).
pub const EYE_FIRE: u16 = 101;
/// `ProjectileID.BrainScramblerBolt` (436) — the Scrambler's ray
/// (`NPC.cs`'s `num412`/`:26640`).
pub const BRAIN_SCRAMBLER: u16 = 436;
/// `ProjectileID.RayGunnerLaser` (438) — the Ray Gunner's ray.
pub const RAY_GUNNER: u16 = 438;
/// `ProjectileID.SaucerDeathray` (447) — the saucer core's deathray
/// (`NPC.cs`'s aiStyle-76 battery).
pub const SAUCER_DEATHRAY: u16 = 447;
/// `ProjectileID.SaucerMissile` (448) — the cannon's shells.
pub const SAUCER_MISSILE: u16 = 448;
/// `ProjectileID.SaucerLaser` (449) — the turret's rays.
pub const SAUCER_LASER: u16 = 449;
/// `ProjectileID.SaucerScrap` (450) — the shell's missiles.
pub const SAUCER_SCRAP: u16 = 450;
/// `ProjectileID.BrainScramblerShot` (436) — the martian ground shooter's
/// ray (`Projectile.cs:5064-5070`: 10x10, aiStyle 1, hostile,
/// ignoreWater).
pub const SCRAMBLER_SHOT: u16 = 436;
/// `ProjectileID.MartianRay` (438) — the Scutlix rider's ray
/// (`Projectile.cs:5081-5089`: 8x8, aiStyle 1, hostile, alpha 255,
/// extraUpdates 3, ignoreWater).
pub const MARTIAN_RAY: u16 = 438;
/// `ProjectileID.Arrow`-family hostile arrow (180) — the pirate Deadeye's
/// (`Projectile.cs:3922-3933`: 4x4, aiStyle 1, hostile, penetrate -1,
/// light 0.5, alpha 255, scale 1.2).
pub const PIRATE_ARROW: u16 = 180;
/// `ProjectileID.DD2JavelinHostile` (662) — the javelin throwers' spear
/// (`Projectile.cs:7122-7130`: 16x16, aiStyle 1, hostile, penetrate -1;
/// 685 the tier-3 twin).
pub const DD2_SPEAR: u16 = 662;
/// `ProjectileID.DD2JavelinHostile`'s tier-3 twin (685).
pub const DD2_SPEAR_T3: u16 = 685;
/// `ProjectileID.DD2GoblinBomb` (681) — the kobold bomber's bomb
/// (`Projectile.cs:7263-7272`: 14x14, aiStyle 16, hostile, penetrate -1,
/// ranged).
pub const DD2_BOMB: u16 = 681;
/// `ProjectileID.DD2SummonSkeleton`-family marker (673/674) — the Dark
/// Mage's summon/heal casts (`Projectile.cs:7208-7220`: 10x10, aiStyle
/// 133, alpha 255, tileCollide false, timeLeft 900).
pub const DARK_MAGE_MARKER: u16 = 673;
/// `ProjectileID.DD2DarkMageShot` (675) — the Dark Mage's shadowflame
/// (`Projectile.cs:7221-7230`: 10x10, aiStyle 1, alpha 255, penetrate -1).
pub const DARK_MAGE_FLAME: u16 = 675;
/// `ProjectileID.DD2BetsyBreath` (687) — Betsy's breath
/// (`AI_110`'s `NewProjectile(..., 687, 35, ...)`).
pub const BETSY_BREATH: u16 = 687;
/// `ProjectileID.DD2BetsyFireball` (686) — Betsy's fireball
/// (`AI_110`'s phase-4 volley: `NewProjectile(..., 686, 35, ...)`).
pub const BETSY_FIREBALL: u16 = 686;
/// `ProjectileID.PirateCannonBall`-family shell (240) — the Dutchman
/// cannon's (`Projectile.cs:3219-3228`: 16x16, aiStyle 2, hostile,
/// penetrate -1, alpha 255).
pub const DUTCHMAN_SHELL: u16 = 240;
/// `ProjectileID.DeerclopsIceSpike` (961) — the ice-spike volley
/// (`NPC.cs:44582`'s `TryMakingSpike`).
pub const DEERCLOPS_ICE_SPIKE: u16 = 961;
/// `ProjectileID.InsanityShadowHostile` (965) — the shadow-hand burst
/// (`NPC.cs:44394`'s `RandomizeInsanityShadowFor`).
pub const SHADOW_HAND: u16 = 965;
/// `ProjectileID.BombSkeletronPrime` (102) — the Prime cannon's bombs
/// (`NPC.cs:28665`'s `num528`; classic damage 0).
pub const BOMB_SKELETRON: u16 = 102;
/// `ProjectileID.FlameArrow` (`Projectile.cs:817-827`).
pub const FLAME_ARROW: u16 = 2;
/// `ProjectileID.Bullet` — the musket ball (`Projectile.cs:950-961`).
pub const MUSKET_BALL: u16 = 14;
/// `ProjectileID.PoisonDartBlowgun`-family dart the Dart Trap fires
/// (`Projectile.cs:1823-1832` — the Wiring emission uses 98, `Wiring.cs:1785`).
pub const TRAP_DART: u16 = 98;
/// The Super Dart Trap's projectile (`Projectile.cs:2707-2716`).
pub const TRAP_SUPERDART: u16 = 184;
/// The Spiky Ball Trap's projectile (`Projectile.cs:2717-2727` — the C#
/// `SpikyBallTrap = 185`; the WIRE-1 legacy named it TRAP_SPEAR).
pub const TRAP_SPIKY_BALL: u16 = 185;
/// The Spear Trap's projectile (`Projectile.cs:2717-2727`'s next arm — the
/// C# `SpearTrap = 186`, aiStyle 14, 14×14, `timeLeft = 900`).
pub const TRAP_SPEAR: u16 = 186;
/// The Venom Dart Trap's projectile (the C# `VenomDartTrap = 980`,
/// `Projectile.cs:9330-9339`; the WIRE-1 legacy named it TRAP_SPIKY).
pub const TRAP_VENOM_DART: u16 = 980;
/// `ProjectileID.Explosives` (108) — the explosives tile's blast
/// (`Projectile.cs:1953-1965`; aiStyle 16, the bomb family).
pub const EXPLOSIVES_BLAST: u16 = 108;
/// The land mine's explosion (164, `Projectile.cs:2524-2535`) — aiStyle 16,
/// 128×128, a ported shape with no vanilla constant name (`ProjectileID.cs`
/// calls it simply `Mine`).
pub const LANDMINE_BLAST: u16 = 164;
/// `ProjectileID.Bomb`-family cannon ammo (178, `Projectile.cs:2641-2649`).
pub const CANNON_BOMB: u16 = 178;
/// The explosive cannonball (601, `Projectile.cs:6499-6512`) — aiStyle 1 with
/// `extraUpdates = 30`.
pub const CANNON_EXPLOSIVE: u16 = 601;
/// The bunny cannon's projectile (281, `Projectile.cs:3676-3687` — aiStyle 49,
/// no ported AI; the constant anchors the `BunnyCannonCanFire` scan).
pub const CANNON_BUNNY: u16 = 281;
/// `ProjectileID.PoisonDart` — the Blowgun's dart (`Projectile.cs:6953-6966`).
pub const POISON_DART: u16 = 645;
/// `ProjectileID.EnchantedBoomerang` (`Projectile.cs:864-873`).
pub const ENCHANTED_BOOMERANG: u16 = 6;
/// `ProjectileID.WoodenBoomerang` (`Projectile.cs:1345-1353`).
pub const WOODEN_BOOMERANG: u16 = 52;
/// `ProjectileID.FrostBoomerang` — the Ice Boomerang (`Projectile.cs:2012-2022`).
pub const ICE_BOOMERANG: u16 = 113;
/// `ProjectileID.Bomb` (`Projectile.cs:1106-1112`).
pub const BOMB: u16 = 28;
/// `ProjectileID.Dynamite` (`Projectile.cs:1113-1121`).
pub const DYNAMITE: u16 = 29;
/// `ProjectileID.StickyBomb` (`Projectile.cs:1198-1205`).
pub const STICKY_BOMB: u16 = 37;
/// `ProjectileID.BouncyBomb` (`Projectile.cs:5862-5869`).
pub const BOUNCY_BOMB: u16 = 516;
/// `ProjectileID.BombFish` (`Projectile.cs:5889-5896`).
pub const BOMB_FISH: u16 = 519;
/// The sticky/hover bomb set of `AI_016_Bombs` (`Projectile.cs:46955`): the
/// kinds that snap to a wall instead of bouncing.
pub const STICKY_BOMBS: [u16; 7] = [37, 397, 470, 519, 773, 911, 1087];
/// The magic (caster) families of M4-PROJ-6: the kind ids below are the
/// verbatim `ProjectileID` values, their `SetDefaults` arms cited per row in
/// [`proj_data`]. The AI arms land in Phases 3-4 — the rows here declare the
/// vanilla `aiStyle` so the dispatch reaches them.
/// `ProjectileID.BallofFire` (15) — the Flower of Fire's shot.
pub const BALL_OF_FIRE: u16 = 15;
/// `ProjectileID.MagicMissile` (16) — the channeled aimed spell.
pub const MAGIC_MISSILE: u16 = 16;
/// `ProjectileID.GreenLaser` (20) — the Space Gun's aiStyle-1 rider.
pub const GREEN_LASER: u16 = 20;
/// `ProjectileID.WaterBolt` (27) — the Water Bolt spell.
pub const WATER_BOLT: u16 = 27;
/// `ProjectileID.Flamelash` (34) — the channeled aimed spell.
pub const FLAMELASH: u16 = 34;
/// `ProjectileID.DemonScythe` (45) — the Demon Scythe's shot.
pub const DEMON_SCYTHE: u16 = 45;
/// `ProjectileID.RainbowRodBullet` (79) — the Rainbow Rod's channeled shot.
pub const RAINBOW_ROD_BULLET: u16 = 79;
/// `ProjectileID.IceBlock` (80) — the Ice Rod's placed-block projectile.
pub const ICE_BLOCK: u16 = 80;
/// `ProjectileID.PurpleLaser` (88) — the Laser Rifle's aiStyle-1 rider.
pub const PURPLE_LASER: u16 = 88;
/// `ProjectileID.CrystalStorm` (94) — the Crystal Storm's shard volley.
pub const CRYSTAL_STORM: u16 = 94;
/// `ProjectileID.CursedFlame` (95) — the friendly companion of the hostile
/// `CursedFlameHostile` 96 ([`CURSED_FLAME`]).
pub const CURSED_FLAME_FRIENDLY: u16 = 95;
/// `ProjectileID.AmethystBolt` (121) — the gem-staff family (aiStyle 29).
pub const GEM_BOLT_AMETHYST: u16 = 121;
/// `ProjectileID.TopazBolt` (122).
pub const GEM_BOLT_TOPAZ: u16 = 122;
/// `ProjectileID.SapphireBolt` (123).
pub const GEM_BOLT_SAPPHIRE: u16 = 123;
/// `ProjectileID.EmeraldBolt` (124).
pub const GEM_BOLT_EMERALD: u16 = 124;
/// `ProjectileID.RubyBolt` (125).
pub const GEM_BOLT_RUBY: u16 = 125;
/// `ProjectileID.DiamondBolt` (126).
pub const GEM_BOLT_DIAMOND: u16 = 126;
/// `ProjectileID.BallofFrost` (253) — the Flower of Frost's shot.
pub const BALL_OF_FROST: u16 = 253;
/// `ProjectileID.MagnetSphereBall` (254) — the Magnet Sphere's ball (aiStyle
/// 47); it spawns the bolt 255.
pub const MAGNET_SPHERE_BALL: u16 = 254;
/// `ProjectileID.MagnetSphereBolt` (255) — the ball's shot (aiStyle 48).
pub const MAGNET_SPHERE_BOLT: u16 = 255;
/// `ProjectileID.InfernoFriendlyBolt` (295) — the Inferno Fork's bolt
/// (aiStyle 50); it spawns the blast 296 on death.
pub const INFERNO_FORK_BOLT: u16 = 295;
/// `ProjectileID.Blast` (296) — the fork's on-death explosion (aiStyle 50).
pub const INFERNO_BLAST: u16 = 296;
/// `ProjectileID.VenomFang` (355) — the Poison Staff's aiStyle-1 rider.
pub const VENOM_FANG: u16 = 355;
/// `ProjectileID.FrostBoltStaff` (359) — the Frost Staff's bolt.
pub const FROST_BOLT: u16 = 359;
/// `ProjectileID.AmberBolt` (597) — the Amber Staff's gem bolt.
pub const GEM_BOLT_AMBER: u16 = 597;
/// The seven gem-bolt kinds the C# spells out as
/// `type == 121 || type == 122 || type == 123 || type == 124 || type == 597 ||
/// type == 125 || type == 126` (`Projectile.cs:22057`'s `FinalizeProjectile`,
/// `:17996`'s `HandleMovement` bounce, `:75425`'s `Kill` burst) — the set this
/// phase's gem arm, its finalize and its death AoE are gated on.
pub const GEM_BOLTS: [u16; 7] = [
    GEM_BOLT_AMETHYST,
    GEM_BOLT_TOPAZ,
    GEM_BOLT_SAPPHIRE,
    GEM_BOLT_EMERALD,
    GEM_BOLT_RUBY,
    GEM_BOLT_DIAMOND,
    GEM_BOLT_AMBER,
];
/// `ProjectileID.SpiritFlame` (659) — the Spirit Flame's shot (aiStyle 129).
pub const SPIRIT_FLAME: u16 = 659;
/// `ProjectileID.SkyFracture` (660) — the Sky Fracture's aiStyle-1 rider.
pub const SKY_FRACTURE: u16 = 660;
/// `ProjectileID.BookOfSkullsSkull` (837) — the Book of Skulls' rider.
pub const BOOK_OF_SKULLS: u16 = 837;
/// `Main.projectile[1001]` — 1000 usable slots (`Projectile.cs:66992`,
/// `GetNextSlot` returns 1000 when full, `:10793`).
pub const MAX_PROJECTILES: usize = 1000;
/// `timeLeft`'s SetDefaults value (`Projectile.cs:787`).
pub const TIME_LEFT_DEFAULT: i32 = 3600;
/// The per-owner immunity a default penetrating hit applies
/// (`Projectile.cs:14117`).
const OWNER_IMMUNE_TICKS: i32 = 10;
const SIZE: f32 = 6.0;

/// `Projectile.CanExplodeTile`'s guard set (`Projectile.cs:78640-78700`):
/// the per-tile permission for the bomb crater. The hardmode gates (the
/// ores 107/108/111/221-223, the chests 37/58, the altars 77) stay closed —
/// the port has no hardmode flag yet (M5-HARDFORCE).
fn can_explode_tile(kind: u16, frame_y_variant: i32) -> bool {
    match kind {
        // `:78655-78668`: the chests, the dolls, the sensors, the other
        // untouchables.
        122 | 150 | 211 | 226 | 237 | 248 | 249 | 250 | 346 | 470 | 475 | 504 | 685 | 686 => false,
        // The hardmode ores (`:78669-78673`) — the `explodeHardmodeOres`
        // flag is only true for 1086/1087 (not ported).
        107 | 108 | 111 | 221 | 222 | 223 => false,
        // The shadow orb / crimson heart containers, pre-hardmode.
        37 | 58 => false,
        // The demon altar (`:78676-78679`).
        77 => false,
        // The pressure plates (`:78680-78684` — the getGood gate).
        48 | 232 => false,
        // The trap family (`:78685-78690`): the variants 1-4
        // (superdart/flame/spear/boulder) stay guarded until Golem.
        137 => !(1..=4).contains(&frame_y_variant),
        _ => true,
    }
}

/// The `SetDefaults` slice of the aiStyle-1/14 families (`Projectile.cs`):
/// the box, the AI style, the penetrate/timeLeft overrides, the flags the
/// flight behaves on and the defaults the C# `SetDefaults` leaves at their
/// reset value unless the per-type arm overrides them. The damage stays the
/// spawner's input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjData {
    pub w: f32,
    pub h: f32,
    /// `aiStyle` — 1 = the arrow/bullet family (`AI_001`), 14 = the
    /// spear/dart flight (`AI()`'s `else if (this.aiStyle == 14)` branch).
    pub ai_style: u8,
    pub penetrate: i32,
    pub time_left: i32,
    pub extra_updates: u32,
    pub tile_collide: bool,
    /// `Projectile.arrow`.
    pub arrow: bool,
    /// `Projectile.trap`.
    pub trap: bool,
    /// `Projectile.usesLocalNPCImmunity` (default false, reset `:714`) +
    /// `localNPCHitCooldown` (default −2, `:464`).
    pub uses_local_immunity: bool,
    pub local_hit_cooldown: i32,
    /// `Projectile.usesIDStaticNPCImmunity` (default false, reset `:715`) +
    /// `idStaticNPCHitCooldown` (default −1, `:466`).
    pub uses_id_static_immunity: bool,
    pub id_static_hit_cooldown: i32,
    /// `Projectile.alpha` (`:779`) — the draw-time fade, which the AI arms
    /// also read as a state marker (the magic families, Phase 3).
    pub alpha: i32,
    /// `Projectile.scale` (default `1f`, `:320`/`:785`).
    pub scale: f32,
}

/// The `(uses_local_immunity, local_hit_cooldown, uses_id_static_immunity,
/// id_static_hit_cooldown, alpha, scale)` group of a `proj_data` row that
/// overrides nothing: the C# `SetDefaults` reset — the immunity flags off
/// (`Projectile.cs:714-715`), the cooldowns −2/`-1` (`:464`, `:466`, reset
/// `:712-713`), `alpha = 0` (`:779`) and `scale = 1f` (`:785`).
const RESET_STATE: (bool, i32, bool, i32, i32, f32) = (false, -2, false, -1, 0, 1.0);

/// The ported families' `SetDefaults` slice. The boulder (186, aiStyle 37)
/// and the flame jet (187, aiStyle 38) traps stay passthroughs — their AI
/// families are M4-PROJ follow-ups. The magic (caster) families'
/// (M4-PROJ-6) rows sit in `ProjectileID` order at the end of the match:
/// their `aiStyle` values declare the dispatch arm Phases 3-4 add.
pub fn proj_data(kind: u16) -> Option<ProjData> {
    let (w, h, ai_style, penetrate, time_left, extra_updates, tile_collide, arrow, trap, state) =
        match kind {
            // `Projectile.cs:807-815` / `:817-827` (the flame arrow shares the
            // box; `timeLeft = 1200` on both).
            WOODEN_ARROW | FLAME_ARROW => {
                (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE)
            }
            // `:817-827`'s hostile twin — the same 10x10 box, the aiStyle-1
            // flight, `timeLeft = 1200`.
            WOODEN_ARROW_HOSTILE => (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:1871-1882` — the laser: 4x4, the aiStyle-1
            // flight, `penetrate = 3`, `extraUpdates = 2`, `timeLeft = 2700`
            // (the AI_037 arm overrides to 300 post-spawn).
            DESTROYER_LASER => (4.0, 4.0, 1, 3, 2700, 2, true, false, false, RESET_STATE),
            // `Projectile.cs:3417-3426` — the Fireball: 16x16, aiStyle 8,
            // `timeLeft = 300`, `penetrate = -1`, hostile, alpha 100.
            GOLEM_FIREBALL => (16.0, 16.0, 8, -1, 300, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:3427-3440` — the Eye Beam: 8x8, aiStyle 1,
            // `extraUpdates = 1`, `tileCollide = false`, `penetrate = -1`,
            // hostile, `timeLeft = 300` (the call sites' override).
            GOLEM_EYE_BEAM => (8.0, 8.0, 1, -1, 300, 1, false, false, false, RESET_STATE),
            // `Projectile.cs:1632-1645` — the EyeLaser: 4x4, aiStyle 1,
            // penetrate 3, extraUpdates 2, timeLeft 600, hostile.
            EYE_LASER => (4.0, 4.0, 1, 3, 600, 2, true, false, false, RESET_STATE),
            // `Projectile.cs:1803-1815` — the CursedFlameHostile: 16x16,
            // aiStyle 8, penetrate -1, hostile.
            CURSED_FLAME => (
                16.0,
                16.0,
                8,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:1885-1895` — the EyeFire: 6x6, aiStyle 23,
            // penetrate -1, extraUpdates 3, hostile.
            EYE_FIRE => (
                6.0,
                6.0,
                23,
                -1,
                TIME_LEFT_DEFAULT,
                3,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:1602-1610` — the Scrambler's bolt: 10x10,
            // aiStyle 1, hostile, ignoreWater.
            BRAIN_SCRAMBLER => (
                10.0,
                10.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:1639-1648` — the Ray Gunner's laser: 8x8,
            // aiStyle 1, extraUpdates 3.
            RAY_GUNNER => (
                8.0,
                8.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                3,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:1804-1818` — the deathray: 30x30, aiStyle 79
            // (the port's boomerang family's homing stand-in), penetrate
            // -1, tileCollide false, timeLeft 240.
            SAUCER_DEATHRAY => (30.0, 30.0, 79, -1, 240, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:1816-1825` — the missile: 14x14, aiStyle 80
            // (the port's boomerang return stand-in), penetrate -1.
            SAUCER_MISSILE => (
                14.0,
                14.0,
                80,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:1826-1837` — the laser: 5x5, aiStyle 1,
            // extraUpdates 1, timeLeft 600.
            SAUCER_LASER => (5.0, 5.0, 1, 1, 600, 1, true, false, false, RESET_STATE),
            // `Projectile.cs:1839-1848` — the scrap: 14x14, aiStyle 14,
            // penetrate -1, timeLeft 360.
            SAUCER_SCRAP => (14.0, 14.0, 14, -1, 360, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9270-9279` — the ice spike: 32x32, aiStyle 157
            // (the sharp-tears family), tileCollide false, ignoreWater.
            DEERCLOPS_ICE_SPIKE => (
                32.0,
                32.0,
                157,
                1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:9307-9317` — the shadow hand: 40x40, aiStyle
            // 187, penetrate -1, tileCollide false, timeLeft 300.
            SHADOW_HAND => (
                40.0,
                40.0,
                187,
                -1,
                300,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // `Projectile.cs:1896-1903` — the bomb: 22x22, aiStyle 16,
            // penetrate -1, hostile.
            BOMB_SKELETRON => (22.0, 22.0, 16, -1, 180, 0, true, false, false, RESET_STATE),
            // `:950-961`: 4×4, `extraUpdates = 1`, `timeLeft = 600`.
            MUSKET_BALL => (4.0, 4.0, 1, 1, 600, 1, true, false, false, RESET_STATE),
            // `:1823-1832` / `:2707-2716` / `:9330-9339`: the trap darts and
            // the spiky ball — `penetrate = -1`, `trap = true`.
            TRAP_DART | TRAP_SUPERDART | TRAP_SPIKY_BALL | TRAP_VENOM_DART => (
                10.0,
                10.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                true,
                RESET_STATE,
            ),
            // `:6953-6966`: the Blowgun's dart — `tileCollide = false`,
            // `extraUpdates = 5`, the local immunity cooldown −1.
            POISON_DART => (
                10.0,
                10.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                5,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // `:2717-2727`: the Spear Trap's spear rides aiStyle 14,
            // `timeLeft = 900`.
            TRAP_SPEAR => (14.0, 14.0, 14, -1, 900, 0, true, false, true, RESET_STATE),
            // The wiring.device arms (M7-WIRE-2): the explosives 141's
            // explosion (`Projectile.cs:1953-1965` — aiStyle 16, 260×260,
            // `timeLeft` 2, `tileCollide = false`, trap), the land mine's
            // (`:2524-2535` — aiStyle 16, 128×128, `timeLeft` 2, trap-less),
            // and the cannon ammo 178/601 (`:2641-2649` — aiStyle 1, 10×10,
            // `timeLeft` 2 / `extraUpdates` 30).
            EXPLOSIVES_BLAST => (260.0, 260.0, 16, -1, 2, 0, false, false, true, RESET_STATE),
            LANDMINE_BLAST => (128.0, 128.0, 16, -1, 2, 0, false, false, false, RESET_STATE),
            CANNON_BOMB => (10.0, 10.0, 1, -1, 2, 0, true, false, false, RESET_STATE),
            CANNON_EXPLOSIVE => (
                10.0,
                10.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                30,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // `:864-873` / `:1345-1353` / `:2012-2022`: the boomerangs —
            // 22×22, aiStyle 3, `penetrate = -1`, melee.
            ENCHANTED_BOOMERANG | WOODEN_BOOMERANG | ICE_BOOMERANG => (
                22.0,
                22.0,
                3,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // The bombs (`:1106-1121`, `:1198-1205`, `:5862-5896`); the
            // fuse seeds ride `NewProjectileSetup` (`:10970-10993`): 180
            // ticks for the bombs, 300 for the dynamite. The sticky set
            // {37, 519} disables the plain tile collide — their hover scan
            // stops them.
            BOMB | BOUNCY_BOMB => (22.0, 22.0, 16, -1, 180, 0, true, false, false, RESET_STATE),
            STICKY_BOMB => (22.0, 22.0, 16, -1, 180, 0, false, false, false, RESET_STATE),
            BOMB_FISH => (24.0, 24.0, 16, -1, 180, 0, false, false, false, RESET_STATE),
            DYNAMITE => (10.0, 10.0, 16, -1, 300, 0, true, false, false, RESET_STATE),
            // The martian family's guns + the Dutchman's cannon — the
            // `SetDefaults` rows packed into the data shape (the saucer
            // family's four consts ride the origin/main set).
            SCRAMBLER_SHOT => (
                10.0,
                10.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // The pirate trio's shots (`Projectile.cs`'s SetDefaults): the
            // Deadeye's arrow 180 (4x4, aiStyle 1, penetrate -1, scale 1.2)
            // and the Corsair's musket ball 82 (10x10, aiStyle 1).
            PIRATE_ARROW => (
                4.0,
                4.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                true,
                false,
                RESET_STATE,
            ),
            MUSKET_BALL => (
                10.0,
                10.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            MARTIAN_RAY => (
                8.0,
                8.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                3,
                true,
                false,
                false,
                RESET_STATE,
            ),
            DUTCHMAN_SHELL => (
                16.0,
                16.0,
                2,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // The DD2 families' weapons (`Projectile.cs`'s SetDefaults): the
            // javelin throwers' spears 662/685 (16x16, aiStyle 1, penetrate
            // -1) and the bombers' bombs 681 (14x14, aiStyle 16, the bomb
            // flight, penetrate -1).
            DD2_SPEAR => (
                16.0,
                16.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            DD2_SPEAR_T3 => (
                16.0,
                16.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            DD2_BOMB => (
                14.0,
                14.0,
                16,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // The Dark Mage's casts (`Projectile.cs:7208-7230`): the summon
            // marker 673 + the heal marker 674 (10x10, aiStyle 133, alpha
            // 255, tileCollide false, timeLeft 900) and the shadowflame
            // 675 (10x10, aiStyle 1, alpha 255, penetrate -1).
            DARK_MAGE_MARKER => (
                10.0,
                10.0,
                133,
                -1,
                900,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            DARK_MAGE_FLAME => (
                10.0,
                10.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // Betsy's breath 687 (`Projectile.cs`'s SetDefaults row: the
            // aiStyle-1 breath flight).
            BETSY_BREATH => (
                12.0,
                12.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // Betsy's fireball 686 (`AI_110`'s `NewProjectile(..., 686, 35,
            // ...)` — the 1500-px leaping shot).
            BETSY_FIREBALL => (
                22.0,
                22.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // --- The magic (caster) families (M4-PROJ-6, in `ProjectileID`
            // order) --- the aiStyle declaration is what Phases 3-4 need; the
            // AI arms land beside the arrow/boomerang/bomb arms in those
            // phases.
            // `:964-973` — the ball of fire: 16x16, aiStyle 8, alpha 100.
            BALL_OF_FIRE => (16.0, 16.0, 8, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 1.0)),
            // `:974-984` — the magic missile: 32x32, aiStyle 9, alpha 100,
            // ignoreWater (the local immunity stays at the reset — only
            // 34/79 carry the 12-tick cooldown).
            MAGIC_MISSILE => (32.0, 32.0, 9, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 1.0)),
            // `:1018-1029` — the green laser: 4x4, aiStyle 1 (the rider
            // hook), `penetrate = 3`, `extraUpdates = 2`, `scale = 0.65f`,
            // `timeLeft = 600`.
            GREEN_LASER => (4.0, 4.0, 1, 3, 600, 2, true, false, false, (false, -2, false, -1, 0, 0.65)),
            // `:1095-1105` — the water bolt: 16x16, aiStyle 8, alpha 255,
            // `penetrate = 10` and `timeLeft /= 2` off the 3600 reset
            // (`:787`) — 1800.
            WATER_BOLT => (16.0, 16.0, 8, 10, 1800, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:1162-1173` — the flamelash: 32x32, aiStyle 9, `penetrate =
            // 2`, the local-NPC immunity with the 12-tick cooldown.
            FLAMELASH => (32.0, 32.0, 9, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 12, false, -1, 0, 1.0)),
            // `:1283-1294` — the demon scythe: 48x48, aiStyle 18, alpha 100,
            // `penetrate = 5`, `scale = 0.9f`.
            DEMON_SCYTHE => (48.0, 48.0, 18, 5, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 0.9)),
            // `:1594-1606` — the rainbow rod's bullet: 32x32, aiStyle 9,
            // `penetrate = 3`, the local immunity with the 12-tick cooldown.
            RAINBOW_ROD_BULLET => (32.0, 32.0, 9, 3, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 12, false, -1, 0, 1.0)),
            // `:1607-1618` — the ice block: 16x16, aiStyle 22,
            // `tileCollide = false` (the placed phase writes tiles).
            ICE_BLOCK => (16.0, 16.0, 22, 1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `:1702-1712` — the purple laser: 6x6, aiStyle 1 (the rider
            // hook), `penetrate = 3`, `extraUpdates = 4`, `scale = 0.65f`,
            // `timeLeft = 600`.
            PURPLE_LASER => (6.0, 6.0, 1, 3, 600, 4, true, false, false, (false, -2, false, -1, 0, 0.65)),
            // `:1777-1790` — the crystal storm: 8x8, aiStyle 24, alpha 50,
            // `scale = 1.2f`, `penetrate = 1`, `timeLeft = 600`, ignoreWater.
            CRYSTAL_STORM => (8.0, 8.0, 24, 1, 600, 0, true, false, false, (false, -2, false, -1, 50, 1.2)),
            // `:1792-1804` — the friendly cursed flame: 16x16, aiStyle 8,
            // alpha 100, `penetrate = 2` (the hostile twin 96 rides above).
            CURSED_FLAME_FRIENDLY => (16.0, 16.0, 8, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 1.0)),
            // `:2105-2158` + `ApplyGemStaffStats` (`:10683-10689`) — the gem
            // bolts 121-126: 10x10, aiStyle 29, alpha 255, the shared
            // `friendly` / `usesLocalNPCImmunity = true` /
            // `localNPCHitCooldown = 60` / `timeLeft = 300` group (the
            // `penetrate = 1` reset holds).
            GEM_BOLT_AMETHYST
            | GEM_BOLT_TOPAZ
            | GEM_BOLT_SAPPHIRE
            | GEM_BOLT_EMERALD
            | GEM_BOLT_RUBY
            | GEM_BOLT_DIAMOND => (10.0, 10.0, 29, 1, 300, 0, true, false, false, (true, 60, false, -1, 255, 1.0)),
            // `:3358-3368` — the ball of frost: 16x16, aiStyle 8, alpha 100
            // (coldDamage — no slot field yet).
            BALL_OF_FROST => (16.0, 16.0, 8, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 1.0)),
            // `:3369-3377` — the magnet sphere's ball: 38x38, aiStyle 47,
            // `timeLeft = 900` (its bolt 255 rides aiStyle 48 below).
            MAGNET_SPHERE_BALL => (38.0, 38.0, 47, 1, 900, 0, true, false, false, RESET_STATE),
            // `:3378-3387` — the ball's bolt: 8x8, aiStyle 48,
            // `extraUpdates = 100`, `timeLeft = 100`.
            MAGNET_SPHERE_BOLT => (8.0, 8.0, 48, 1, 100, 100, true, false, false, RESET_STATE),
            // `:3849-3858` — the inferno fork's bolt: 12x12, aiStyle 50,
            // alpha 255.
            INFERNO_FORK_BOLT => (12.0, 12.0, 50, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:3859-3868` — the bolt's on-death blast: 150x150, aiStyle 50,
            // alpha 255, `tileCollide = false`, `penetrate = -1`.
            INFERNO_BLAST => (150.0, 150.0, 50, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:4451-4463` — the venom fang: 12x12, aiStyle 1 (the rider
            // hook), alpha 255, `penetrate = 5`, the local immunity with
            // the 10-tick cooldown, `timeLeft = 58`.
            VENOM_FANG => (12.0, 12.0, 1, 5, 58, 0, true, false, false, (true, 10, false, -1, 255, 1.0)),
            // `:4497-4507` — the frost staff's bolt: 14x14, aiStyle 28,
            // alpha 255, `penetrate = 2` (coldDamage — no slot field yet).
            FROST_BOLT => (14.0, 14.0, 28, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:6461-6470` + `ApplyGemStaffStats` (`:10683-10689`) — the
            // Amber Staff's bolt, the gem-bolt family's seventh kind.
            GEM_BOLT_AMBER => (10.0, 10.0, 29, 1, 300, 0, true, false, false, (true, 60, false, -1, 255, 1.0)),
            // `:7083-7094` — the spirit flame: 8x8, aiStyle 129, alpha 255,
            // `timeLeft = 240`, `tileCollide = false`, `penetrate = -1`.
            SPIRIT_FLAME => (8.0, 8.0, 129, -1, 240, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:7096-7107` — the sky fracture: 10x10, aiStyle 1 (the rider
            // hook), alpha 255, `timeLeft = 600`, `extraUpdates = 1`.
            SKY_FRACTURE => (10.0, 10.0, 1, 1, 600, 1, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:8124-8135` — the book of skulls' skull: 26x26, aiStyle 1
            // (the rider hook), alpha 255, `penetrate = 3`, `timeLeft =
            // 480`, `tileCollide = false`.
            BOOK_OF_SKULLS => (26.0, 26.0, 1, 3, 480, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            _ => return None,
        };
    let (
        uses_local_immunity,
        local_hit_cooldown,
        uses_id_static_immunity,
        id_static_hit_cooldown,
        alpha,
        scale,
    ) = state;
    Some(ProjData {
        w,
        h,
        ai_style,
        penetrate,
        time_left,
        extra_updates,
        tile_collide,
        arrow,
        trap,
        uses_local_immunity,
        local_hit_cooldown,
        uses_id_static_immunity,
        id_static_hit_cooldown,
        alpha,
        scale,
    })
}

/// `Projectile.GemStaffFeatures` (`Projectile.cs:37-164`) — the gem-staff
/// feature pack, a bitfield over the C#'s `FloatIntUnion`
/// (`FloatIntUnion.cs:6-59`): the union's `this[int bit]` accessor
/// (`:14-31`) reads and writes bit `bit` of the `int` view of the same
/// storage, and the struct's constructor (`:161-164`) takes the packed
/// `float` the shoot path hands `NewProjectile` as `ai0`. Bit 0 Homing, 1
/// FastThenSlow, 2 CanBounce, 3 AoeExplosion, 4 SwirlTwins, 5 ArPenSpread, 6
/// BiggerHitbox, 7 AllGems, 8 RepeatsGem, 9 AllGems2.
///
/// The write side (`Player.PickGemStaffFeatures`'s pack,
/// `Player.cs:49470-49489`) is Phase 5's shoot path; this phase is the reader
/// side the AI arm, the finalize and the death AoE run on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GemStaffFeatures {
    /// The union's `Int` view (`FloatIntUnion.cs:11-12`) — bit `i` of this
    /// word is feature `i`.
    bits: u32,
}

impl GemStaffFeatures {
    /// `Bits[0]` — `Projectile.cs:41-51`.
    pub const HOMING: u32 = 1 << 0;
    /// `Bits[1]` — `:53-63`.
    pub const FAST_THEN_SLOW: u32 = 1 << 1;
    /// `Bits[2]` — `:65-75`.
    pub const CAN_BOUNCE: u32 = 1 << 2;
    /// `Bits[3]` — `:77-87`.
    pub const AOE_EXPLOSION: u32 = 1 << 3;
    /// `Bits[4]` — `:89-99`.
    pub const SWIRL_TWINS: u32 = 1 << 4;
    /// `Bits[5]` — `:101-111`.
    pub const AR_PEN_SPREAD: u32 = 1 << 5;
    /// `Bits[6]` — `:113-123`.
    pub const BIGGER_HITBOX: u32 = 1 << 6;
    /// `Bits[7]` — `:125-135`.
    pub const ALL_GEMS: u32 = 1 << 7;
    /// `Bits[8]` — `:137-147`.
    pub const REPEATS_GEM: u32 = 1 << 8;
    /// `Bits[9]` — `:149-159`.
    pub const ALL_GEMS2: u32 = 1 << 9;

    /// `new GemStaffFeatures(aiState)` (`Projectile.cs:161-164`) — the C#
    /// implicit `float` → `FloatIntUnion` cast (`FloatIntUnion.cs:34-40`)
    /// reinterprets the float's bits, so `f32::to_bits` is the exact
    /// equivalent and every `ai[0]` round-trips bit for bit.
    pub fn from_ai(ai0: f32) -> Self {
        Self { bits: ai0.to_bits() }
    }

    /// The implicit `FloatIntUnion` → `float` cast
    /// (`FloatIntUnion.cs:42-45`) — the value the pack writes back into
    /// `ai[0]`.
    pub fn ai_state(self) -> f32 {
        f32::from_bits(self.bits)
    }

    /// `Bits[bit]`'s getter (`FloatIntUnion.cs:14-16`).
    pub fn bit(self, bit: u32) -> bool {
        self.bits & bit != 0
    }

    /// `Bits[bit] = value`'s setter (`FloatIntUnion.cs:21-31`) — the or /
    /// and-not pair.
    pub fn set_bit(&mut self, bit: u32, on: bool) {
        if on {
            self.bits |= bit;
        } else {
            self.bits &= !bit;
        }
    }

    /// `Homing` (`Projectile.cs:41-51`).
    pub fn homing(self) -> bool {
        self.bit(Self::HOMING)
    }

    /// `FastThenSlow` (`:53-63`).
    pub fn fast_then_slow(self) -> bool {
        self.bit(Self::FAST_THEN_SLOW)
    }

    /// `CanBounce` (`:65-75`).
    pub fn can_bounce(self) -> bool {
        self.bit(Self::CAN_BOUNCE)
    }

    /// `AoeExplosion` (`:77-87`).
    pub fn aoe_explosion(self) -> bool {
        self.bit(Self::AOE_EXPLOSION)
    }

    /// `SwirlTwins` (`:89-99`).
    pub fn swirl_twins(self) -> bool {
        self.bit(Self::SWIRL_TWINS)
    }

    /// `ArPenSpread` (`:101-111`).
    pub fn ar_pen_spread(self) -> bool {
        self.bit(Self::AR_PEN_SPREAD)
    }

    /// `BiggerHitbox` (`:113-123`).
    pub fn bigger_hitbox(self) -> bool {
        self.bit(Self::BIGGER_HITBOX)
    }

    /// `AllGems` (`:125-135`).
    pub fn all_gems(self) -> bool {
        self.bit(Self::ALL_GEMS)
    }

    /// `RepeatsGem` (`:137-147`).
    pub fn repeats_gem(self) -> bool {
        self.bit(Self::REPEATS_GEM)
    }

    /// `AllGems2` (`:149-159`).
    pub fn all_gems2(self) -> bool {
        self.bit(Self::ALL_GEMS2)
    }

    /// `GetDamageMultiplier` (`:166-186`): 1 for the AllGems pair, otherwise
    /// the minimum of 1, the SwirlTwins' 0.6, the CanBounce's 0.8 and the
    /// ArPenSpread's 0.5 — the shoot-side multiplier the death AoE divides
    /// by (`:75468`).
    pub fn damage_multiplier(self) -> f32 {
        if self.all_gems() || self.all_gems2() {
            return 1.0;
        }
        let mut num: f32 = 1.0;
        if self.swirl_twins() {
            num = num.min(0.6);
        }
        if self.can_bounce() {
            num = num.min(0.8);
        }
        if self.ar_pen_spread() {
            num = num.min(0.5);
        }
        num
    }
}

/// One pool slot (`Projectile.cs:296-470`, the engine-relevant subset).
#[derive(Clone, Debug)]
pub struct Projectile {
    pub active: bool,
    pub kind: u16,
    /// `Projectile.owner`; 255 = unset.
    pub owner: u8,
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub damage: i32,
    pub knock_back: f32,
    /// The crit decision rolled at throw time (the item stream lives in the
    /// app, not in the projectile step).
    pub crit: bool,
    /// `Projectile.friendly` (`:3115`): true = the player's shots (the pool's
    /// PVE pass hits NPCs), false = the hostile shots (the player-contact
    /// pass hurts the owner).
    pub friendly: bool,
    /// `timeLeft` — ticks down once per update, `Kill()` at `<= 0` (`:17201`).
    pub time_left: i32,
    /// `maxPenetrate` / `penetrate`; 1 = single hit.
    pub max_penetrate: i32,
    pub penetrate: i32,
    /// `usesLocalNPCImmunity` + `localNPCHitCooldown` (default −2, `:464`).
    pub uses_local_immunity: bool,
    pub local_hit_cooldown: i32,
    /// `usesIDStaticNPCImmunity` + `idStaticNPCHitCooldown` (default −1, `:466`).
    pub uses_id_static_immunity: bool,
    pub id_static_hit_cooldown: i32,
    /// `extraUpdates`: the AI runs `1 + extraUpdates` times per tick (`:16668`).
    pub extra_updates: u32,
    /// The C# `ai[0..2]` scratch array — the aiStyle-1 flight counter lives
    /// in `ai[0]` (the generic arm `ai[0] >= 15f`, `Projectile.cs:57572`).
    pub ai: [f32; 3],
    /// The C# `localAI[0..2]` (`Projectile.cs:336-338`, `maxAI = 3` `:79770`)
    /// — the second scratch array, reset to zero with `ai` by `SetDefaults`
    /// (`:736-740`). The magic families keep their phase counters there.
    pub local_ai: [f32; 3],
    /// `Projectile.alpha` (`:326`, reset `:779`) — the draw fade, which the AI
    /// arms also read as a state marker (the magic families, Phase 3).
    pub alpha: i32,
    /// `Projectile.scale` (`:320`, reset `:785`).
    pub scale: f32,
    /// The box (`base.width`/`base.height`); the shuriken keeps `SIZE`.
    pub w: f32,
    pub h: f32,
    /// `Projectile.tileCollide` (the default true, `:787`-defaults).
    pub tile_collide: bool,
    /// `Projectile.arrow`.
    pub arrow: bool,
    /// `Projectile.trap`.
    pub trap: bool,
    bounces: u8,
    /// The tile-fall mode (`AI_010`'s `ai[0] == 1f`): the projectile carries a
    /// falling tile kind (`spawn_falling`), falls with `velocity.Y += 0.41f`
    /// per tick and places the tile on landing.
    pub falling_tile: Option<u16>,
    /// `AI_016`'s `timeLeft <= 3` trigger: the bomb waits for the pool's
    /// end-of-tick explosion pass (`Kill()`'s bomb path).
    pub detonated: bool,
    /// The aiStyle-50 one-shot (`Projectile.cs:74341-74350`'s `Kill()` arm):
    /// the Inferno Fork bolt 295 spawns its blast 296 exactly once, however
    /// the bolt died (the arm's fade kill, `timeLeft`, or a penetrating hit).
    /// The flag also keeps the blast itself — which rides the same style —
    /// from ever spawning a second one.
    blast_spawned: bool,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            active: false,
            kind: 0,
            owner: 255,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            damage: 0,
            knock_back: 0.0,
            crit: false,
            friendly: true,
            time_left: TIME_LEFT_DEFAULT,
            max_penetrate: 1,
            penetrate: 1,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -1,
            extra_updates: 0,
            ai: [0.0; 3],
            local_ai: [0.0; 3],
            alpha: 0,
            scale: 1.0,
            w: SIZE,
            h: SIZE,
            tile_collide: true,
            arrow: false,
            trap: false,
            bounces: 0,
            falling_tile: None,
            detonated: false,
            blast_spawned: false,
        }
    }
}

impl Projectile {
    fn shuriken(x: f32, y: f32, vx: f32, vy: f32, crit: bool) -> Self {
        Self {
            active: true,
            kind: SHURIKEN,
            owner: 0,
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            damage: 10,
            knock_back: 12.0,
            crit,
            // The placeholder's `age > 2 s || bounces > 6` lifetime is replaced
            // by the faithful `timeLeft`; `penetrate` stays the vanilla 1.
            ..Default::default()
        }
    }

    /// The aiStyle-2 physics (the first pass's body, unchanged): two half-steps
    /// so 11 px/tick cannot tunnel, 0.6-damped bounces.
    fn ai_shuriken(&mut self, world: &World) {
        self.rot += if self.vel.x >= 0.0 { 0.42 } else { -0.42 };
        for _ in 0..2 {
            let step = self.vel * 0.5;
            let nx = self.pos.x + step.x;
            if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
                self.vel.x = -self.vel.x * 0.6;
                self.bounces += 1;
            } else {
                self.pos.x = nx;
            }
            let ny = self.pos.y + step.y;
            if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
                self.vel.y = -self.vel.y * 0.6;
                self.bounces += 1;
            } else {
                self.pos.y = ny;
            }
        }
    }

    /// `AI_001`'s generic arm (`Projectile.cs:57572-57583` + the rotation
    /// default `:57725-57728` + the ±16 velocity clamp `:57756-57763`): the
    /// flight counter `ai[0]++`, the gravity `velocity.Y += 0.1f` after 15
    /// ticks, the flight-direction sprite rotation, and the tile collision —
    /// the `Update` movement's blocked hit kills the projectile (the arrows
    /// break on contact; the water transforms are M4-PROJ polish).
    ///
    /// The body is the shared [`Projectile::ai_style1_flight`] (M4-PROJ-6
    /// Phase 3 extracted it so the aiStyle-1 riders can wrap the same
    /// generic flight instead of duplicating it) — the delegation is
    /// behaviour-identical, pinned by the arrow tests below.
    fn ai_arrow(&mut self, world: &World) {
        self.ai_style1_flight(world, true);
    }

    /// The aiStyle-14 core (`Projectile()`'s `else if (this.aiStyle == 14)`
    /// branch, the default arm `Projectile.cs:25143-25158` + the tile-kill
    /// scan `:25726-25737` + the lava/shimmer kills `:25739-25750`): the
    /// straight flight, `ai[0] > 5f` → `velocity.Y += 0.2f` with the 0.97
    /// ground-slide friction, `rotation += velocity.X * 0.1f`, and the
    /// center (+4, +8)-box vs solid-tile scan killing the projectile.
    fn ai_spear(&mut self, world: &World) {
        self.ai[0] += 1.0;
        if self.ai[0] > 5.0 {
            self.ai[0] = 5.0;
            if self.vel.y == 0.0 && self.vel.x != 0.0 {
                self.vel.x *= 0.97;
                if (-0.01..0.01).contains(&self.vel.x) {
                    self.vel.x = 0.0;
                }
            }
            self.vel.y += 0.2;
        }
        self.rot += self.vel.x * 0.1;
        // The tile-kill scan: the box inset to `(x + 4, y + 4)` with the
        // +8 px center probe against solid, non-127 tiles (`:25726-25737`).
        let cx = self.pos.x + 4.0;
        let cy = self.pos.y + 4.0;
        let tx0 = ((self.pos.x / 16.0) as i32 - 1).max(0);
        let tx1 = (((self.pos.x + self.w) / 16.0) as i32 + 2).min(world.width - 1);
        let ty0 = ((self.pos.y / 16.0) as i32 - 1).max(0);
        let ty1 = (((self.pos.y + self.h) / 16.0) as i32 + 2).min(world.height - 1);
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                let Some(t) = world.tile(tx, ty) else {
                    continue;
                };
                if !t.active()
                    || t.kind == 127
                    || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                    || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    continue;
                }
                let rx = tx as f32 * 16.0;
                let ry = ty as f32 * 16.0;
                if cx + 8.0 > rx && cx < rx + 16.0 && cy + 8.0 > ry && cy < ry + 16.0 {
                    self.active = false;
                    self.time_left = 0;
                    return;
                }
            }
        }
        self.pos += self.vel;
    }

    /// `AI_003_Boomerang`'s generic arms (`Projectile.cs:39024-39331` the
    /// outbound, `:39468-39545` the return). The OUT state flies straight
    /// (`ai[1]++`; at `>= 30f` — the Light Disc 106 at `>= 45f` — the return
    /// state sets) and bounces off a tile or an NPC hit by reversing the
    /// velocity and setting `ai[0] = 1` (`:14594-14602`, `:13068-13080`).
    /// The RETURN state disables tile collision, seeks the owner at the
    /// speed 9 / accel 0.4 (the per-type overrides are the special
    /// boomerangs; the plain 6/52/113 use the defaults) with the ±2×
    /// crossing boost, kills past 3000 px from the owner and when the
    /// owner's box intersects (`:39538-39544`).
    fn ai_boomerang(&mut self, world: &World, owner: Option<(f32, f32)>) {
        if self.ai[0] == 0.0 {
            // The outbound flight; the 106 Light Disc returns at 45 ticks.
            self.ai[1] += 1.0;
            let limit = if self.kind == 106 { 45.0 } else { 30.0 };
            if self.ai[1] >= limit {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
            }
            if self.tile_collide {
                let nx = self.pos.x + self.vel.x;
                let ny = self.pos.y + self.vel.y;
                let hit_x = crate::player::collides_rect(world, nx, self.pos.y, self.w, self.h);
                let hit_y = crate::player::collides_rect(world, self.pos.x, ny, self.w, self.h);
                if hit_x || hit_y {
                    // `:14594-14602`: the outbound tile hit reverses into the
                    // return state instead of killing.
                    if self.ai[0] == 0.0 {
                        self.vel = -self.vel;
                    }
                    self.ai[0] = 1.0;
                    return;
                }
                self.pos.x = nx;
                self.pos.y = ny;
            } else {
                self.pos += self.vel;
            }
        } else {
            // The return (`:39468-39545`): `tileCollide = false`, the owner
            // seek at the speed 9 / accel 0.4, the ±2× crossing boost.
            self.tile_collide = false;
            let Some((ox, oy)) = owner else {
                return;
            };
            let cx = self.pos.x + self.w * 0.5;
            let cy = self.pos.y + self.h * 0.5;
            let mut dx = ox - cx;
            let mut dy = oy - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            // `:39500-39505`: the 3000 px cut-off kills the stray.
            if dist > 3000.0 {
                self.active = false;
                self.time_left = 0;
                return;
            }
            // The `meleeSpeed` remap (`:39487-39497`) is ×1 at the port's
            // default melee speed.
            let (speed, accel) = (9.0f32, 0.4f32);
            let inv = speed / dist.max(f32::EPSILON);
            dx *= inv;
            dy *= inv;
            if self.vel.x < dx {
                self.vel.x += accel;
                if self.vel.x < 0.0 && dx > 0.0 {
                    self.vel.x += accel;
                }
            } else if self.vel.x > dx {
                self.vel.x -= accel;
                if self.vel.x > 0.0 && dx < 0.0 {
                    self.vel.x -= accel;
                }
            }
            if self.vel.y < dy {
                self.vel.y += accel;
                if self.vel.y < 0.0 && dy > 0.0 {
                    self.vel.y += accel;
                }
            } else if self.vel.y > dy {
                self.vel.y -= accel;
                if self.vel.y > 0.0 && dy < 0.0 {
                    self.vel.y -= accel;
                }
            }
            self.pos += self.vel;
            // `:39538-39544`: the owner's box catches the boomerang.
            // The 20x42 hitbox (/).
            let (pw, ph) = (crate::player::WIDTH, crate::player::HEIGHT);
            let hit = self.pos.x + self.w > ox - pw * 0.5
                && self.pos.x < ox + pw * 0.5
                && self.pos.y + self.h > oy - ph * 0.5
                && self.pos.y < oy + ph * 0.5;
            if hit {
                self.active = false;
                self.time_left = 0;
            }
        }
    }

    /// `AI_016_Bombs` (`Projectile.cs:46904-47545`): the thrown-bomb flight.
    /// `ai[0]` counts; the sticky set {37, 519} hovers against a solid (the
    /// scan sets `velocity = (0, -0.2)`); the gravity tail (`:47505-47520`)
    /// adds `velocity.Y += 0.2f` after 5 ticks with the 0.97 ground-slide
    /// (the dynamite 29 another ×0.99); the flight damage zeroes (the
    /// explosion carries it); the detonation fires when `timeLeft <= 3`
    /// (`:47165-47167`).
    fn ai_bomb(&mut self, world: &World) {
        self.ai[0] += 1.0;
        if STICKY_BOMBS.contains(&self.kind) {
            // The sticky scan (`:46956-47025`): the box inset ±4 px vs
            // solid, non-127, non-platform tiles — the hit snaps the
            // velocity to the hover.
            let scan = |world: &World, pos: Vec2, w: f32, h: f32| -> bool {
                let tx0 = ((pos.x / 16.0) as i32 - 1).max(0);
                let tx1 = (((pos.x + w) / 16.0) as i32 + 2).min(world.width - 1);
                let ty0 = ((pos.y / 16.0) as i32 - 1).max(0);
                let ty1 = (((pos.y + h) / 16.0) as i32 + 2).min(world.height - 1);
                for ty in ty0..=ty1 {
                    for tx in tx0..=tx1 {
                        let Some(t) = world.tile(tx, ty) else {
                            continue;
                        };
                        if !t.active()
                            || t.kind == 127
                            || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                            || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                        {
                            continue;
                        }
                        let rx = tx as f32 * 16.0;
                        let ry = ty as f32 * 16.0;
                        if pos.x + w - 4.0 > rx
                            && pos.x + 4.0 < rx + 16.0
                            && pos.y + h - 4.0 > ry
                            && pos.y + 4.0 < ry + 16.0
                        {
                            return true;
                        }
                    }
                }
                false
            };
            if scan(world, self.pos, self.w, self.h) {
                self.vel.x = 0.0;
                self.vel.y = -0.2;
            }
        }
        // The gravity tail (`:47505-47520`): the thrown arc — the gravity
        // waits `ai[0] > 5f`.
        if self.ai[0] > 5.0 {
            self.ai[0] = 10.0;
            if self.vel.y == 0.0 && self.vel.x != 0.0 {
                self.vel.x *= 0.97;
                if self.kind == DYNAMITE {
                    self.vel.x *= 0.99;
                }
                if (-0.01..0.01).contains(&self.vel.x) {
                    self.vel.x = 0.0;
                }
            }
            self.vel.y += 0.2;
        }
        // The tile collision (the aiStyle-16 arm of the `Update` movement,
        // `:19737-19762`): the bounce at −0.4 (the dynamite another ×0.8).
        if self.tile_collide {
            let nx = self.pos.x + self.vel.x;
            let ny = self.pos.y + self.vel.y;
            let damp = if self.kind == DYNAMITE { 0.32 } else { 0.4 };
            let hit_x = crate::player::collides_rect(world, nx, self.pos.y, self.w, self.h);
            if hit_x {
                self.vel.x = -self.vel.x * damp;
            } else {
                self.pos.x = nx;
            }
            let hit_y = crate::player::collides_rect(world, self.pos.x, ny, self.w, self.h);
            if hit_y {
                self.vel.y = -self.vel.y * damp;
            } else {
                self.pos.y = ny;
            }
        } else {
            self.pos += self.vel;
        }
        self.rot += self.vel.x * 0.1;
        // The flight damage zeroes (`:47175-47176` — the kinds outside the
        // keep-damage list; all the port's bombs are) — the explosion
        // re-arms the damage.
        self.damage = 0;
        // The detonation (`:47165-47167`): `timeLeft <= 3` on the local
        // owner.
        if self.time_left <= 3 {
            self.detonated = true;
        }
    }

    // --- The magic (caster) families of M4-PROJ-6, Phase 3: the
    // self-contained flight arms (aiStyles 8/18/22/24/28/50 and the
    // aiStyle-1 riders). Every arm carries its `Projectile.cs` citation;
    // the dust/sound spawns are SKIPPED (the port has no dust layer in the
    // pool) and each skip is noted at its point, for ANALYSIS by Phase 6.

    /// The generic aiStyle-1 flight (`AI_001`'s tail, `Projectile.cs:56038-56041`
    /// `if (flag3) ai[0]++` + the gravity arm `:57569-57581` + the rotation
    /// default `:57725-57728` + the ±16 clamp `:57756-57763` + the movement's
    /// default tile arm `:19843-19848`): the flight counter, the
    /// `velocity.Y += 0.1f` gravity after 15 ticks, the flight-direction
    /// rotation and the kill-on-tile-contact. Extracted from `ai_arrow` by
    /// Phase 3 so the aiStyle-1 riders can wrap it (the C# has one method:
    /// the per-type hooks at the top, the per-type rotation tail at the
    /// bottom, this flight in the middle).
    ///
    /// `count_flight` is the C#'s `flag3` — the per-type skip switch
    /// (`:55947-56032`) that holds the flight counter at rest. The plain
    /// arrows pass `true`; the M4-PROJ-6 riders 20/88/355/660 are all on that
    /// list, so they never reach the gravity at all.
    fn ai_style1_flight(&mut self, world: &World, count_flight: bool) {
        if count_flight {
            self.ai[0] += 1.0;
        }
        if self.ai[0] >= 15.0 {
            self.vel.y += 0.1;
        }
        self.vel.y = self.vel.y.clamp(-16.0, 16.0);
        self.rot = self.vel.y.atan2(self.vel.x) + 1.57;
        if self.tile_collide {
            let nx = self.pos.x + self.vel.x;
            let ny = self.pos.y + self.vel.y;
            let hit_x = crate::player::collides_rect(world, nx, self.pos.y, self.w, self.h);
            let hit_y = crate::player::collides_rect(world, self.pos.x, ny, self.w, self.h);
            if hit_x || hit_y {
                self.active = false;
                self.time_left = 0;
            } else {
                self.pos.x = nx;
                self.pos.y = ny;
            }
        } else {
            self.pos += self.vel;
        }
    }

    /// The aiStyle-1 riders that are not plain arrows (`AI_001`'s per-type
    /// hooks, M4-PROJ-6 Phase 3): the PurpleLaser 88 (`:54021-54045` +
    /// `:55930-55939`), the GreenLaser 20 (`:54047-54064`), the SkyFracture
    /// 660 (`:55672-55693` + `:57650-57652`) and the VenomFang 355
    /// (`:56728-56747`), wrapped around [`Projectile::ai_style1_flight`] in
    /// the C#'s own order — every per-type hook above the flight, the per-kind
    /// rotation tail below it. All four are on the `flag3` skip list
    /// (`:55955`/`:55957`/`:55988`/`:56020`), so the flight runs with a
    /// resting counter and no gravity: a laser or a fang flies exactly
    /// straight, as in the vanilla game.
    fn ai_style1_rider(&mut self, world: &World) {
        // The PurpleLaser (`:54021-54045`): the `extraUpdates` remap off
        // `ai[0]`, then the scale growth to 4 with the owner-only knockback
        // and the 8-tick counter. `ai[0]` sits at rest on this kind, so the
        // remap never fires — ported for its shape, pinned by the tests.
        if self.kind == PURPLE_LASER {
            if self.ai[0] != 0.0 {
                self.extra_updates = self.ai[0].min(15.0) as u32;
            }
            if self.scale < 4.0 {
                self.scale = (self.scale + 0.053).min(4.0);
                if self.owner == 0 {
                    self.knock_back *= 1.0045;
                    self.local_ai[0] += 1.0;
                    if self.local_ai[0] >= 8.0 {
                        self.local_ai[0] = 0.0;
                        // `damage += Main.rand.Next(1, 4)` and
                        // `penetrate += Main.rand.Next(0, 2)` (`:54041-54042`)
                        // are SKIPPED — the pool keeps zero RNG (the port's
                        // hard determinism rule); recorded for ANALYSIS.
                    }
                }
            }
        }
        // The GreenLaser (`:54047-54064`): the same shape with the 3 cap,
        // the 1.0025 knockback and the 25-tick counter.
        if self.kind == GREEN_LASER && self.scale < 3.0 {
            self.scale = (self.scale + 0.0189).min(3.0);
            if self.owner == 0 {
                self.knock_back *= 1.0025;
                self.local_ai[0] += 1.0;
                if self.local_ai[0] >= 25.0 {
                    self.local_ai[0] = 0.0;
                    // `damage += Main.rand.Next(1, 3)` / `penetrate++`
                    // (`:54059-54060`) SKIPPED — no RNG in the pool.
                }
            }
        }
        // The SkyFracture's first-tick marker (`:55676-55687`): the light
        // plot (`Utils.PlotTileLine` → `CastLightOpen`), the
        // `SoundID.Item9` cue, the `frame` roll and the 16× dust 180 are
        // SKIPPED; the alpha/scale write is the state the port keeps.
        if self.kind == SKY_FRACTURE && self.alpha > 0 {
            self.alpha = 0;
            self.scale = 1.1;
        }
        // The PurpleLaser's alpha fade (`:55928-55939`).
        if self.kind == PURPLE_LASER {
            if self.alpha > 0 {
                self.alpha -= 10;
            }
            self.alpha = self.alpha.max(0);
        }
        // The VenomFang's fade and its dust cue (`:56728-56747`): the
        // `alpha -= 50` state rides; the dust 205 at `alpha == 0` is SKIPPED.
        if self.kind == VENOM_FANG {
            self.alpha = (self.alpha - 50).max(0);
        }
        // The generic flight, then the per-kind rotation tail the C# order
        // below it: SkyFracture's `velocity.ToRotation() + π/4`
        // (`:57650-57652`).
        self.ai_style1_flight(world, false);
        if self.kind == SKY_FRACTURE {
            self.rot = self.vel.y.atan2(self.vel.x) + std::f32::consts::FRAC_PI_4;
        }
    }

    /// The Book of Skulls' rider (`AI_001`'s `else if (this.type == 207 ||
    /// this.type == 837)` block, `Projectile.cs:56778-56887`): the in-tile
    /// `timeLeft` drain, the cached spawn speed, the alpha fade, the NPC
    /// acquisition scan (the Manhattan-300 gate with the `Collision.CanHit`
    /// line) into `ai[1]`, the 1000f retain/drop rule and the steering at
    /// the weight 20 (the generic 8 for 207 has no ported row).
    ///
    /// The arm takes `npcs` — the targeting's `Main.npc` slice — exactly as
    /// the Phase-1 arm-signature contract's targeting case does.
    fn ai_book_of_skulls(&mut self, world: &World, npcs: &[Npc]) {
        // `:56794-56800`: inside a solid tile the skull burns 33 ticks off
        // its own life (the tile-collide false flight passes through).
        if self.time_left > 33 {
            let cx = (self.pos.x + self.w * 0.5) as i32;
            let cy = (self.pos.y + self.h * 0.5) as i32;
            if solid_tile(world, cx / 16, cy / 16) {
                self.time_left -= 33;
            }
        }
        // `:56801-56807`: the spawn speed is cached on the first tick and
        // the steering keeps that magnitude for the whole flight.
        let spawn_speed = if self.local_ai[0] == 0.0 {
            self.local_ai[0] = (self.vel.x * self.vel.x + self.vel.y * self.vel.y).sqrt();
            self.local_ai[0]
        } else {
            self.local_ai[0]
        };
        // `:56808-56812`: the alpha fade.
        if self.alpha > 0 {
            self.alpha -= 25;
        }
        self.alpha = self.alpha.max(0);
        // `:56813-56842`: the acquisition scan while no target is held. The
        // scan keeps the running minimum `num189` (seeded at 300) and only
        // writes `ai[1]` after the loop (`:56844-56847`), so the nearest
        // chaseable NPC inside the Manhattan 300 with a clear line wins — the
        // index is kept alongside the distance and committed once at the end.
        let mut target_xy = (self.pos.x, self.pos.y);
        let mut has_target = false;
        if self.ai[1] == 0.0 {
            let mut best = 300.0f32;
            let mut best_idx: Option<usize> = None;
            for (idx, n) in npcs.iter().enumerate() {
                if !chaseable(n) {
                    continue;
                }
                let (w, h) = n.size();
                let (nx, ny) = (n.x + w * 0.5, n.y + h * 0.5);
                let dist = (self.pos.x + self.w * 0.5 - nx).abs()
                    + (self.pos.y + self.h * 0.5 - ny).abs();
                if dist < best
                    && can_hit(world, self.pos.x, self.pos.y, self.w, self.h, n.x, n.y, w, h)
                {
                    best = dist;
                    best_idx = Some(idx);
                    target_xy = (nx, ny);
                }
            }
            if let Some(idx) = best_idx {
                self.ai[1] = (idx + 1) as f32;
            }
        }
        // `:56843-56858`: the held target's retain rule — the Manhattan 1000f
        // keeps it, everything else (dead, no longer chaseable, too far)
        // clears `ai[1]` and the skull flies on unsteered. The C# spells the
        // gate `active && CanBeChasedBy(ignoreDontTakeDamage: true) &&
        // !dontTakeDamage`, which is [`chaseable`] exactly.
        if self.ai[1] > 0.0 {
            let idx = self.ai[1] as usize - 1;
            match npcs.get(idx) {
                Some(n) if chaseable(n) => {
                    let (w, h) = n.size();
                    let (nx, ny) = (n.x + w * 0.5, n.y + h * 0.5);
                    if (self.pos.x + self.w * 0.5 - nx).abs()
                        + (self.pos.y + self.h * 0.5 - ny).abs()
                        < 1000.0
                    {
                        has_target = true;
                        target_xy = (nx, ny);
                    }
                }
                _ => self.ai[1] = 0.0,
            }
        }
        // `:56859-56861`: the hostile shots never steer.
        if !self.friendly {
            has_target = false;
        }
        // `:56862-56886`: the steering blend at the per-type weight.
        if has_target {
            let (cx, cy) = (self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
            let inv = spawn_speed / ((target_xy.0 - cx).hypot(target_xy.1 - cy)).max(f32::EPSILON);
            let (tx, ty) = ((target_xy.0 - cx) * inv, (target_xy.1 - cy) * inv);
            // `:56878-56882`: the weight is 20 for the Book of Skulls.
            let weight = 20.0;
            self.vel.x = (self.vel.x * (weight - 1.0) + tx) / weight;
            self.vel.y = (self.vel.y * (weight - 1.0) + ty) / weight;
        }
        // 837 is off the `flag3` skip list, so the flight counter and the
        // gravity run for the skull exactly as they do for an arrow.
        self.ai_style1_flight(world, true);
        // The per-kind rotation tail (`:57597-57602`): the skull's sprite
        // faces its flight direction without the generic +π/2 offset. The
        // C#'s `base.direction` field has no slot counterpart; it tracks
        // `velocity.X`'s sign after each sub-step (`:17016-17026`), so the
        // velocity sign is the port's stand-in (the same substitution the
        // spear/bomb arms make for their `direction` reads).
        self.rot = if self.vel.x < 0.0 {
            (-self.vel.y).atan2(-self.vel.x)
        } else {
            self.vel.y.atan2(self.vel.x)
        };
    }

    /// `AI()`'s aiStyle-8 arm (`Projectile.cs:24233-24362`) plus the same
    /// style's tile bounce in the movement (`HandleMovement`'s aiStyle-8
    /// branch, `:19573-19613`): the gravity timer `ai[1]++` with its three
    /// per-type skips, `velocity.Y += 0.2f` from `ai[1] >= 20f`, the 16 fall
    /// clamp, `rotation += 0.3f * direction`, and the per-type bounce kill
    /// threshold (5 default, 7 for the WaterBolt's lava bounce, 6 for the
    /// ball of fire, 8 for the ball of frost).
    fn ai_ball_of_fire(&mut self, world: &World) {
        // `:24235-24244`: the one-shot sound gates of the Golem fireball 258
        // and the hostile cursed flame 96. The SoundEngine call is SKIPPED
        // (no sound layer in the pool) and `localAI[0]`'s write rides with
        // it — the flag is read only by this same gate.
        if matches!(self.kind, GOLEM_FIREBALL | CURSED_FLAME) && self.local_ai[0] == 0.0 {
            self.local_ai[0] = 1.0;
        }
        // `:24245-24301`: the WaterBolt's own block — the `extraUpdates = 5`
        // remap first (read off the *previous* tick's `ai[1]`, as the C#
        // orders it), then the lava probe at the centre that reverses the fall
        // and marks `ai[1] = 1f`. The dust/gore spawns are SKIPPED.
        if self.kind == WATER_BOLT {
            if self.ai[1] == 1.0 {
                self.extra_updates = 5;
            }
            if self.vel.y > 0.0 {
                let cx = ((self.pos.x + self.w * 0.5) / TILE_PX) as i32;
                let cy = ((self.pos.y + self.h * 0.5) / TILE_PX) as i32;
                let in_lava = world
                    .tile(cx, cy)
                    .is_some_and(|t| t.liquid_kind() == tr_core::tile::LiquidKind::Lava);
                if in_lava {
                    self.ai[1] = 1.0;
                    self.vel.y = -self.vel.y;
                }
            }
        }
        // `:24304-24337`: the per-type dust spawns (the 502 disco light, the
        // 95/96 cursed flame, the 253 frost, the default 6) are SKIPPED.
        // `:24338-24341`: the gravity timer — the WaterBolt reuses `ai[1]` as
        // its lava-bounce marker and 96/258 as their sound gate, so all three
        // keep the counter at rest and never reach the gravity.
        if !matches!(self.kind, WATER_BOLT | CURSED_FLAME | GOLEM_FIREBALL) {
            self.ai[1] += 1.0;
        }
        if self.ai[1] >= 20.0 {
            self.vel.y += 0.2;
        }
        // `:24346-24357`: the 502 velocity rotation has no ported row; the
        // default is `rotation += 0.3f * direction` and nothing in this arm
        // writes `direction` (its Entity default 1 carries the flight).
        self.rot += 0.3;
        // `:24358-24361`: the fall clamp.
        if self.vel.y > 16.0 {
            self.vel.y = 16.0;
        }
        // The tile bounce (`:19573-19613`), the arm's own tile-collision
        // behaviour in the port's per-arm collision model.
        if self.tile_collide {
            let nx = self.pos.x + self.vel.x;
            let ny = self.pos.y + self.vel.y;
            let hit_x = crate::player::collides_rect(world, nx, self.pos.y, self.w, self.h);
            let hit_y = crate::player::collides_rect(world, self.pos.x, ny, self.w, self.h);
            if hit_x || hit_y {
                // The C#'s bounce arm is `else if (this.aiStyle == 8 &&
                // this.type != 96)` (`:19573`), so the hostile cursed flame 96
                // falls past it to the blocked-move chain's default
                // (`:19851-19855`): one last step into the tile, then the
                // kill — the port's [`Projectile::kill_step`], not a bounce.
                if self.kind == CURSED_FLAME {
                    self.kill_step(world);
                    return;
                }
                self.ai[0] += 1.0;
                // `:19577-19590`: the per-type bounce budget.
                let limit = match self.kind {
                    BALL_OF_FIRE => 6.0,
                    BALL_OF_FROST => 8.0,
                    WATER_BOLT if self.ai[1] == 1.0 => 7.0,
                    _ => 5.0,
                };
                if self.ai[0] >= limit {
                    // `:19591-19595`: the last bounce carries the velocity
                    // into the wall and kills.
                    self.pos += self.vel;
                    self.active = false;
                    self.time_left = 0;
                    return;
                }
                // `:19598-19612`: the reversal, with the ball of fire's 0.8
                // damp on a fast fall. The C# reads the post-`TileCollision`
                // velocity for the `> 4f` gate; the port has no swept
                // resolution, so the incoming velocity stands in.
                if self.kind == BALL_OF_FIRE && self.vel.y > 4.0 {
                    if hit_y {
                        self.vel.y = -self.vel.y * 0.8;
                    }
                } else if hit_y {
                    self.vel.y = -self.vel.y;
                }
                if hit_x {
                    self.vel.x = -self.vel.x;
                }
            }
        }
        // `UpdatePosition` (`:20468`).
        self.pos += self.vel;
    }

    /// `AI()`'s aiStyle-18 arm (`Projectile.cs:25367-25476`): the DemonScythe
    /// 45's flight — `rotation += 0.8f * direction`, the counter `ai[0]++`,
    /// the `velocity *= 1.06f` acceleration from tick 30 and the `ai[0] =
    /// 200f` clamp that stops it at 100. The 44/263/274/1092 riders of the
    /// same style have no ported row; the 2× dust 27 is SKIPPED.
    fn ai_demon_scythe(&mut self, world: &World) {
        self.rot += 0.8;
        self.ai[0] += 1.0;
        if self.ai[0] >= 30.0 {
            if self.ai[0] < 100.0 {
                self.vel *= 1.06;
            } else {
                // `:25465-25468`: parked at 200, so the acceleration stops.
                self.ai[0] = 200.0;
            }
        }
        self.kill_step(world);
    }

    /// `AI()`'s aiStyle-22 arm (`Projectile.cs:25607-25860`): the Ice Rod
    /// block's three phases on the `ai` signs, all dust SKIPPED.
    ///
    /// - **standing** (`ai[1] < 0`, `:25612-25680`): the failed placement's
    ///   fall — the 60-tick `timeLeft` cap, the ±0.3 rotation and the probe
    ///   kill scan.
    /// - **placed** (`ai[0] < 0`, `:25681-25703`): the block stands in the
    ///   tile, drains `ai[0]` until `-900` and is `KillTile`d away (owner
    ///   branch).
    /// - **flying** (`:25704-25860`): the probe kill scan, the spawn tile
    ///   target's walk to the nearest adjacent non-solid cell and the
    ///   `WorldGen.PlaceTile(127)` that stands the block down.
    ///
    /// The tile writes need `&mut World` — the one carve-out of the
    /// arm-signature doc block over the dispatch — and run only on the
    /// owner branch (`Main.myPlayer == this.owner`; the port's local player
    /// is slot `owner == 0`).
    fn ai_ice_block(&mut self, world: &mut World) {
        // `:25611`: a stopped block is fully opaque — the write runs against
        // the pre-place velocity, so the standing phase keeps its own alpha.
        if self.vel.x == 0.0 && self.vel.y == 0.0 {
            self.alpha = 255;
        }
        // The standing phase (`:25612-25680`).
        if self.ai[1] < 0.0 {
            if self.time_left > 60 {
                self.time_left = 60;
            }
            if self.vel.x > 0.0 {
                self.rot += 0.3;
            } else {
                self.rot -= 0.3;
            }
            // The kill scan (`:25633-25676`) — the standing phase's variant
            // counts an actuated tile as blocking (`active()`, not
            // `nactive()`).
            self.ice_block_probe_kill(world, true);
            // The dust 67 (`:25677-25679`) is SKIPPED.
            return;
        }
        // The placed phase (`:25681-25703`).
        if self.ai[0] < 0.0 {
            // `:25702-25710`: the 10× dust 67 burst on the placement tick and
            // the 1-in-30 puff afterwards are SKIPPED.
            let tx = (self.pos.x / TILE_PX) as i32;
            let ty = (self.pos.y / TILE_PX) as i32;
            let standing = world.tile(tx, ty).is_some_and(|t| t.active());
            if !standing {
                self.active = false;
                self.time_left = 0;
            }
            self.ai[0] -= 1.0;
            // `:25757-25760`: after 900 ticks the owner's block is mined away
            // and the projectile goes with it.
            if self.ai[0] <= -900.0
                && self.owner == 0
                && world
                    .tile(tx, ty)
                    .is_some_and(|t| t.active() && t.kind == MAGICAL_ICE_BLOCK)
            {
                let _ = tr_core::edit::kill_tile(world, tx, ty, false);
                self.active = false;
                self.time_left = 0;
            }
            return;
        }
        // The flying phase (`:25704-25860`): the kill scan's `nactive()`
        // variant — an actuated tile does not block.
        self.ice_block_probe_kill(world, false);
        if !self.active {
            return;
        }
        // The lava (`:25742-25744`) and shimmer (`:25746-25751`) kills are
        // SKIPPED — the slot carries no liquid-wet flags in this port.
        // `:25755-25759`: the spawn tile target, `ai[0]`/`ai[1]` — the Ice
        // Rod's `Player.tileTargetX/Y` (`Player.cs:49383-49387`) — walked to
        // the nearest adjacent non-solid cell (`:25760-25798`).
        let mut tx = self.ai[0] as i32;
        let mut ty = self.ai[1] as i32;
        if world.in_bounds(tx, ty) && solid_tile(world, tx, ty) {
            // The C#'s two centre gates (`Center.X < tile·16 + 8`,
            // `Center.Y < tile·16 + 8`).
            let left_of_centre = self.pos.x + self.w * 0.5 < tx as f32 * TILE_PX + 8.0;
            let above_centre = self.pos.y + self.h * 0.5 < ty as f32 * TILE_PX + 8.0;
            let open = |tx: i32, ty: i32| world.in_bounds(tx, ty) && !solid_tile(world, tx, ty);
            if self.vel.x.abs() > self.vel.y.abs() {
                // The C# repeats the up/down test twice (`:25769-25778`):
                // the second copy only fires when the Center.Y gate blocked
                // the first. Ported verbatim.
                if above_centre && open(tx, ty - 1) {
                    ty -= 1;
                } else if open(tx, ty + 1) {
                    ty += 1;
                } else if open(tx, ty - 1) {
                    ty -= 1;
                } else if left_of_centre && open(tx - 1, ty) {
                    tx -= 1;
                } else if open(tx + 1, ty) {
                    tx += 1;
                } else if open(tx - 1, ty) {
                    tx -= 1;
                }
            } else if left_of_centre && open(tx - 1, ty) {
                tx -= 1;
            } else if open(tx + 1, ty) {
                tx += 1;
            } else if open(tx - 1, ty) {
                tx -= 1;
            } else if above_centre && open(tx, ty - 1) {
                ty -= 1;
            } else if open(tx, ty + 1) {
                ty += 1;
            } else if open(tx, ty - 1) {
                ty -= 1;
            }
        }
        // `:25800-25803`: the spin.
        if self.vel.x > 0.0 {
            self.rot += 0.3;
        } else {
            self.rot -= 0.3;
        }
        // `:25804`: the placement is the owner's write alone.
        if self.owner != 0 {
            return;
        }
        let ctx = ((self.pos.x + self.w * 0.5) / TILE_PX) as i32;
        let cty = ((self.pos.y + self.h * 0.5) / TILE_PX) as i32;
        // `:25806-25820`: the block is placed when the projectile's own tile
        // has reached the target, or when the flight has carried it past the
        // target in both axes.
        let arrived = (ctx == tx && cty == ty)
            || ((self.vel.x <= 0.0 && ctx <= tx) || (self.vel.x >= 0.0 && ctx >= tx))
                && ((self.vel.y <= 0.0 && cty <= ty) || (self.vel.y >= 0.0 && cty >= ty));
        if !arrived {
            return;
        }
        // `:25842-25860`: the block goes down; a refused placement is the
        // C#'s `else { ai[1] = -1f }` — the block falls away.
        if tr_core::edit::place_tile(world, tx, ty, MAGICAL_ICE_BLOCK) {
            self.damage = 0;
            self.ai[0] = -1.0;
            self.vel = Vec2::ZERO;
            self.alpha = 255;
            self.pos.x = tx as f32 * TILE_PX;
            self.pos.y = ty as f32 * TILE_PX;
        } else {
            self.ai[1] = -1.0;
        }
    }

    /// The aiStyle-22 tile-kill scan (`Projectile.cs:25633-25676` standing,
    /// `:25704-25741` flying): the −1/+2 tile window over the box and the
    /// `(x + 4, y + 4)` + 8 px probe against every active solid that is not
    /// the placed block 127. `allow_actuated` carries the C#'s `active()` vs
    /// `nactive()` difference between the two phases.
    fn ice_block_probe_kill(&mut self, world: &World, allow_actuated: bool) {
        let tx0 = ((self.pos.x / TILE_PX) as i32 - 1).max(0);
        let tx1 = (((self.pos.x + self.w) / TILE_PX) as i32 + 2).min(world.width - 1);
        let ty0 = ((self.pos.y / TILE_PX) as i32 - 1).max(0);
        let ty1 = (((self.pos.y + self.h) / TILE_PX) as i32 + 2).min(world.height - 1);
        let px = self.pos.x + 4.0;
        let py = self.pos.y + 4.0;
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                let Some(t) = world.tile(tx, ty) else {
                    continue;
                };
                let blocking = t.active()
                    && t.kind != MAGICAL_ICE_BLOCK
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                    && (allow_actuated || t.flags & tr_core::tile::FLAG_INACTIVE == 0);
                if !blocking {
                    continue;
                }
                let rx = tx as f32 * TILE_PX;
                let ry = ty as f32 * TILE_PX;
                if px + 8.0 > rx && px < rx + TILE_PX && py + 8.0 > ry && py < ry + TILE_PX {
                    self.active = false;
                    self.time_left = 0;
                }
            }
        }
    }

    /// `AI()`'s aiStyle-24 arm (`Projectile.cs:25936-26014`): the rotation
    /// `+= velocity.X * 0.2f`, the counter `ai[1]++`, the CrystalStorm 94's
    /// ×0.985 drag and its fade from `ai[1] > 130f` (`scale -= 0.05f`, the
    /// kill at 0.2) — the generic kinds take ×0.96 and fade from 15. The
    /// `light = scale * 0.5f` write has no slot field yet (render-only) and
    /// the per-type dust is SKIPPED.
    fn ai_crystal_storm(&mut self, world: &World) {
        // `:25938-25943`'s 1134 first-tick sound/dust block is SKIPPED (no
        // ported row for the kind).
        self.rot += self.vel.x * 0.2;
        self.ai[1] += 1.0;
        // `:25944-25966` (the 94 shard) and `:25967-26013` (the generic; the
        // 1114 branch differs only by the wet kill, which no ported row
        // reaches).
        let (drag, fade_at) = if self.kind == CRYSTAL_STORM {
            (0.985, 130.0)
        } else {
            (0.96, 15.0)
        };
        self.vel *= drag;
        if self.ai[1] > fade_at {
            self.scale -= 0.05;
            if self.scale <= 0.2 {
                self.scale = 0.2;
                self.active = false;
                self.time_left = 0;
                return;
            }
        }
        self.kill_step(world);
    }

    /// `AI()`'s aiStyle-28 arm (`Projectile.cs:26555-26675`): the FrostStaff
    /// bolt 359's whole machine is the one-shot `ai[1]` gate — after the
    /// first tick the arm does nothing but fly. The first tick's 3× dust 92
    /// and the `SoundID.Item28` cue are SKIPPED (no dust or sound layer in
    /// the pool), as are the other kinds' frame animations (no ported row,
    /// no `frame`/`frameCounter` slot fields).
    fn ai_frost_bolt(&mut self, world: &World) {
        if self.ai[1] == 0.0 {
            self.ai[1] = 1.0;
            // The SKIP list of the first tick: the per-kind dust blocks
            // (`:26572-26609`) and the 967/1026/359 sound cues
            // (`:26611-26672`) — noted for ANALYSIS.
        }
        self.kill_step(world);
    }

    /// `AI()`'s aiStyle-50 arm (`Projectile.cs:28323-28401`): the InfernoFork
    /// family's dust-fade lifetime. The hostile 291 branch is out of scope
    /// (no ported row), the bolt 295's own arm is its 8× dust 174 and nothing
    /// else (`:28333-28340`), the dust spawns are SKIPPED, and the first-tick
    /// `SoundID.Item74` is SKIPPED — what survives of the blast's fade math is
    /// the counter `ai[0]++` (+3 for the blast 296) and the kill at
    /// `ai[0] > 540f`, where the C#'s dust count `25 - (ai[0] - 180) / 2`
    /// reaches zero.
    fn ai_inferno_fork(&mut self, world: &World) {
        // `:28333-28340`: the bolt 295's whole AI arm is its 8× dust 174 and
        // nothing else — the counter, the fade kill and the `+3` belong to the
        // blast 296 alone, so the bolt flies straight and dies on its
        // `timeLeft` (the `SetDefaults` default 3600, `:787`) or its
        // `penetrate`, whichever comes first. The dust is SKIPPED (no dust
        // layer in the pool) and the movement is the blocked-move default the
        // bolt shares with every other style-50 shot.
        if self.kind == INFERNO_FORK_BOLT {
            self.kill_step(world);
            return;
        }
        if self.local_ai[0] == 0.0 {
            self.local_ai[0] += 1.0;
        }
        self.ai[0] += 1.0;
        if self.kind == INFERNO_BLAST {
            self.ai[0] += 3.0;
        }
        // `:28376-28379`: `num350 = 25 - (ai[0] - 180) / 2` once
        // `ai[0] > 540f`. It is `<= 0` for every `ai[0] > 540f` (the smallest
        // gives 25 - 180.5), so the first tick past the window kills. The
        // blast's ×0.7 scaling (`:28385`) cannot clear it (25 × 0.7 > 0).
        if self.ai[0] > 540.0 {
            self.active = false;
            self.time_left = 0;
            return;
        }
        self.kill_step(world);
    }

    /// `AI_009_MagicMissiles` (`Projectile.cs:43195-43425`): the channeled
    /// aimed spells — Magic Missile 16, Flamelash 34 and Rainbow Rod's bolt
    /// 79 (the `type == 491` `_Old` variant has no ported row).
    ///
    /// The machine: while the owner holds the channel the spell keeps its aim
    /// (`ai[0..1]` = the clamped `MouseWorld`, `ai[1]`'s high bits carrying the
    /// 34 drag-lift flag); releasing it drops `ai[0]` to −1, hunts a homing
    /// target (`ai[1]` = the npc index, or −1 for none) and flies it at the
    /// cruise speed 32; the 64f close-range re-aim, the 4f min-boost, the time
    /// floor/cap and the rotation lerps follow. The `_Old` arm (491) and the
    /// 16/34 dust-and-sound blocks are SKIPPED (no RNG, sound or dust layer in
    /// the pool).
    fn ai_magic_missile(
        &mut self,
        world: &World,
        npcs: &[Npc],
        owner: Option<(f32, f32)>,
        player: Option<&crate::player::Player>,
    ) {
        // `:43199`: `num = 32f` — the cruise speed.
        const CRUISE: f32 = 32.0;
        // `:43201`: `num2 = Main.maxTilesY * 16` — the bitmap-height unit the
        // Flamelash packs its drag-lift flag into `ai[1]`'s high bits.
        let world_h = world.height as f32 * TILE_PX;
        // `:43210-43213`: `num3 = (int)(ai[1] / num2)` while aiming.
        let mut lift = if self.ai[0] >= 0.0 {
            (self.ai[1] / world_h) as i32
        } else {
            0
        };
        // `:43214`: `flag = ai[0] == -1f || ai[0] == -2f` — released or failed.
        let released = self.ai[0] == -1.0 || self.ai[0] == -2.0;
        // `:43215-43235`: the Flamelash block. Its `frameCounter++` / `frame`
        // animation (`:43216-43222`) has no slot fields — SKIPPED, like the
        // spell's `netUpdate` writes.
        if self.kind == FLAMELASH {
            if self.penetrate == 1 && self.ai[0] >= 0.0 && lift == 0 {
                // `:43223-43227`: the first NOT-penetrating pass arms the drag.
                self.ai[1] += world_h;
                lift = 1;
            }
            if self.penetrate == 1 && self.ai[0] == -1.0 {
                // `:43228-43231`: a release while still penetrating fails the
                // homing (−2) so the drag-lift counter keeps running.
                self.ai[0] = -2.0;
            }
            if lift > 0 || self.ai[0] == -2.0 {
                // `:43232-43235`: the drag-lift frame counter.
                self.local_ai[0] += 1.0;
            }
        }
        // `:43236-43275`: the owner block — the port's local player is
        // `self.owner == 0`. The `owner` centre context pins the release's
        // away-boost to the resolved player; the channel itself rides `player`.
        if self.owner == 0 {
            if self.ai[0] >= 0.0 {
                if let Some(p) = player {
                    // `:43239-43249`: hold — keep the aim only while the
                    // channel is open and the held item shoots this kind.
                    if p.channel && p.channel_item == self.kind {
                        if self.ai[0] != p.channel_target.x || self.ai[1] != p.channel_target.y {
                            // The C# writes `pointPosition.Y + num2 * num3`;
                            // when the 34 block above just armed the flag it
                            // raised `num3` to 1, so the sign test reads the
                            // new value exactly as the C# does.
                            self.ai[0] = p.channel_target.x;
                            self.ai[1] =
                                p.channel_target.y + world_h * lift as f32;
                        }
                    } else {
                        // `:43250-43262`: release.
                        self.ai[0] = -1.0;
                        self.ai[1] = -1.0;
                        // The parameterless overload's 800f default
                        // (`:45628`).
                        let target = find_target_with_line_of_sight(
                            world, npcs, self.pos, self.w, self.h, 800.0, &[],
                        );
                        if target != -1 {
                            self.ai[1] = target as f32;
                        } else if self.vel.length() < 2.0 {
                            // `:43259-43260`: `DirectionFrom(player.Center)`.
                            let Some((ox, oy)) = owner.or(Some(p.center())) else {
                                return;
                            };
                            let away = Vec2::new(
                                self.pos.x + self.w * 0.5 - ox,
                                self.pos.y + self.h * 0.5 - oy,
                            );
                            let dir = if away.length() > 0.0 {
                                away.normalize()
                            } else {
                                Vec2::ZERO
                            };
                            self.vel = dir * CRUISE;
                        } else {
                            // `:43261-43262`.
                            self.vel = safe_normalize(self.vel) * CRUISE;
                        }
                    }
                }
            }
            if released && self.ai[1] == -1.0 {
                // `:43266-43271`: the failed-homing re-search.
                // The parameterless overload's 800f default (`:45628`).
                let target =
                    find_target_with_line_of_sight(world, npcs, self.pos, self.w, self.h, 800.0, &[]);
                if target != -1 {
                    self.ai[1] = target as f32;
                }
            }
        }
        // `:43273-43336`: the aim resolution and the steering.
        // `:43278-43279`: the held aim (the high bits are the 34 flag).
        let mut aim: Option<Vec2> = if self.ai[0] > 0.0 && self.ai[1] > 0.0 {
            Some(Vec2::new(self.ai[0], self.ai[1] % world_h))
        } else {
            None
        };
        // `:43280-43298`: the released homing — the packed npc index.
        let mut amount = 1.0f32;
        if released && self.ai[1] >= 0.0 {
            let index = self.ai[1] as i32;
            if let Some(npc) = npcs.get(index as usize) {
                if chaseable(npc) {
                    let center = npc_center(npc);
                    aim = Some(center);
                    // `:43292-43295`: the distance-ramped blend — the C#
                    // wraps the ramp in `MathHelper.Lerp(0f, 0.2f, ...)`, so
                    // the blend never exceeds 0.2.
                    let t = self.vel.distance(center);
                    let ramp = get_lerp_value(
                        200.0,
                        20.0,
                        1.0 - get_lerp_value(0.0, 100.0, t) * get_lerp_value(600.0, 400.0, t),
                    );
                    amount = 0.2 * ramp;
                } else {
                    // `:43296-43297`: the mid-flight death of the target.
                    self.ai[1] = -1.0;
                }
            }
        }
        // `:43300-43303`: the rotation direction gate.
        let mut spin = released;
        if let Some(target) = aim {
            if target.distance(Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5))
                >= 64.0
            {
                // `:43306-43320`: the far case — the capped approach vector,
                // the 4f min-boost, the `amount`-lerped velocity.
                spin = true;
                let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                let delta = target - center;
                let approach = safe_normalize(delta) * CRUISE.min(delta.length());
                if self.vel.length() < 4.0 {
                    // `:43313-43314`: `velocity.SafeNormalize(Zero).RotatedBy
                    // (PI/4).SafeNormalize(Zero) * 4f` — the zero vector keeps
                    // its own direction (0, 0).
                    let boost = if self.vel == Vec2::ZERO {
                        Vec2::ZERO
                    } else {
                        self.vel
                            .normalize()
                            .rotate(Vec2::from_angle(std::f32::consts::FRAC_PI_4))
                            .normalize()
                            * 4.0
                    };
                    self.vel += boost;
                }
                if self.vel.is_nan() {
                    // `:43316-43317`: the NaN guard's `Kill()`.
                    self.active = false;
                    self.time_left = 0;
                    return;
                }
                // `:43319`: `Vector2.Lerp(velocity, value2, amount)`.
                self.vel += (approach - self.vel) * amount;
            } else {
                // `:43321-43327`: the close case — the direct pull and the
                // 2f speed requirement.
                self.vel *= 0.3;
                let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                self.vel += (target - center) * 0.3;
                spin = self.vel.length() >= 2.0;
            }
            // `:43328-43331`: the survival floor.
            if self.time_left < 60 {
                self.time_left = 60;
            }
        }
        if released && self.ai[1] < 0.0 {
            // `:43332-43338`: the lost target — the 32f fly-out and the cap.
            if self.vel.length() != CRUISE {
                // `MoveTowards(velocity.SafeNormalize(UnitY) * num, 4f)` — the
                // zero vector walks to (0, 32) in 4f steps.
                let target = safe_normalize(self.vel) * CRUISE;
                self.vel = move_towards(self.vel, target, 4.0);
            }
            if self.time_left > 300 {
                self.time_left = 300;
            }
        }
        // `:43339-43344`: the rotation lerps.
        if spin && self.vel != Vec2::ZERO {
            self.rot = angle_towards(self.rot, self.vel.y.atan2(self.vel.x), std::f32::consts::FRAC_PI_4);
        } else {
            // `AngleLerp(rotation, 0f, 0.2f)` — the shortest-way clamped turn
            // toward 0, which is what `angle_towards` computes for the 0 target.
            self.rot = angle_towards(self.rot, 0.0, 0.2 * std::f32::consts::PI);
        }
        // `:43346`: `flag3` (the turn-sharpness detector) is read only by the
        // 16/34 dust blocks below — both SKIPPED (no RNG, sound or dust layer
        // in the pool, the same skip the gem arm records). The 16 soundDelay
        // gate and its `SoundID.Item9` cue go with them.
        // `:43390+`: the Flamelash's `Utils.GetLerpValue(0f, 10f, localAI[0])`
        // trail tint feeds its dust blocks only — no draw state on the slot.
        // The movement: the family has no bounce arm, so the port's generic
        // blocked-move default (`:19851-19855`) applies — `kill_step`.
        self.kill_step(world);
    }

    /// `AI_029_GemStaffs` (`Projectile.cs:37643-37827`) → the bolts'
    /// `AI_029_GemStaffs_ActuallyGemStaffs` (`:37829-38001`): the gem-staff
    /// family (kinds 121-126 and 597 — the C#'s `else` arm; the 521/522/619/
    /// 620/731 rider kinds have no ported row and no arm here). The state —
    /// the owner-only target refresh, the homing steering, the SwirlTwins
    /// sinusoid pair, the FastThenSlow shrink and the rotation — then the
    /// movement, which is `HandleMovement`'s gem branch (`:17996-18056`) in
    /// the port's per-arm collision model.
    ///
    /// SKIP list (each noted at its point): the per-tick gem dust
    /// (`:37847-37855`), the first-tick staff sound (`:37856-37864`), the
    /// per-gem light tints (`:37954-38000`), the bounce's `HitTiles` plus
    /// impact sound (`:18047-18051`) and the Kill burst's dust
    /// (`:75439-75463`). Every `Main.rand` call in the C# arm belongs to one
    /// of those, so the arm keeps the pool's zero-RNG invariant.
    fn ai_gem_staff(
        &mut self,
        world: &World,
        npcs: &[Npc],
        // The C# reads `Main.player[this.owner]` only for its `Main.myPlayer`
        // gate, which the port folds to `self.owner == 0`; the two context
        // parameters ride the family signature the doc block pins.
        _owner: Option<(f32, f32)>,
        _player: Option<&crate::player::Player>,
    ) {
        // `:37831-37839`: the gem dust id `type - 121 + 86` (121→86 amethyst
        // … 126→91 diamond, 597→262 amber) — the two-per-tick dust spawns
        // that read it (`:37847-37855`) are SKIPPED.
        let features = GemStaffFeatures::from_ai(self.ai[0]);
        // The homing block (`:37876-37923`).
        if features.homing() {
            // `:37878-37882`: the 500f radius doubles for FastThenSlow or
            // CanBounce.
            let mut range = 500.0;
            if features.fast_then_slow() || features.can_bounce() {
                range *= 2.0;
            }
            let mut target = self.ai[2] as i32 - 1;
            // `:37884-37893`: the owner-only refresh — `this.owner ==
            // Main.myPlayer` is the port's `self.owner == 0`. The C#'s
            // `netUpdate = true` write is net code — SKIPPED.
            if self.owner == 0 {
                let held = target;
                target =
                    find_target_with_line_of_sight(world, npcs, self.pos, self.w, self.h, range, &[]);
                if target != held {
                    self.ai[2] = (target + 1) as f32;
                }
            }
            if target >= 0 {
                self.gem_bolt_steer(npcs, target as usize, range, &features);
            }
        }
        // The SwirlTwins sinusoid pair (`:37924-37935`).
        if features.swirl_twins() {
            let (freq, amplitude) = (0.05f32, 6.0f32);
            // `:37926-37930`: the twin index rides `ai[1]` as the sinusoid's
            // phase (the shoot path spawns the twin with `ai1 = 1f`), and the
            // C#'s `_ = gemStaffFeatures.AllGems;` read (`:37930`) discards
            // its result — ported as nothing.
            let phase = self.ai[1] as i32 as f32 * std::f32::consts::PI;
            let (n0, n1) = (
                (self.local_ai[0] * (std::f32::consts::PI * 2.0) * freq + phase).sin(),
                ((self.local_ai[0] + 1.0) * (std::f32::consts::PI * 2.0) * freq + phase).sin(),
            );
            self.local_ai[0] += 1.0;
            let angle = self.vel.y.atan2(self.vel.x) + std::f32::consts::FRAC_PI_2;
            self.pos += Vec2::new(angle.cos(), angle.sin()) * (n1 - n0) * amplitude;
        }
        // The FastThenSlow shrink (`:37936-37952`).
        if features.fast_then_slow() {
            if self.vel.length() > 0.1 {
                self.vel *= 0.97;
            } else {
                self.scale -= 0.005;
                if self.scale < 0.1 {
                    // `:37948-37951`'s `Kill()` — the pool's death hook runs
                    // the gem bolts' `Kill()` arm (`Projectile.cs:75425`) for
                    // the AoE.
                    self.active = false;
                    self.time_left = 0;
                    return;
                }
            }
        }
        // `:37953`: the rotation rides the flight direction.
        self.rot = self.vel.y.atan2(self.vel.x);
        // The `:37954-38000` per-gem light tints (`num24 = num - 85` — 3/5/4/
        // 1/6/2, the 262 amber default) are SKIPPED: the slot carries no
        // `light` field (recorded for ANALYSIS, as with Phase 3's light
        // writes).
        // The movement: `HandleMovement`'s gem branch (`:17996-18056`).
        self.gem_bolt_move(world, npcs, &features);
    }

    /// The homing steering block (`Projectile.cs:37894-37923`): the speed
    /// build at the per-family steer speed (8f, 16f for FastThenSlow) with the
    /// 1.1/0.1 accel pair, the distance-remapped turn rate, the
    /// `rotateTowards` turn (the speed preserved) and the close-range `Lerp`
    /// pull. `target` indexes the `Main.npc` slice; `range` is the homing
    /// radius the AI block computed (500f, doubled).
    fn gem_bolt_steer(
        &mut self,
        npcs: &[Npc],
        target: usize,
        range: f32,
        features: &GemStaffFeatures,
    ) {
        // `:37896-37903`: the steer speed and the drag / turn-rate constants.
        let steer_speed = if features.fast_then_slow() { 16.0 } else { 8.0 };
        let (accel, drag, turn) = (1.1f32, 0.1f32, 0.1f32);
        // `:37904-37909`: the capture runs *before* the acceleration, and
        // every `if (num13 < num9)` scaling below reads that captured value.
        let speed = self.vel.length();
        if speed < steer_speed {
            self.vel += safe_normalize(self.vel) * drag;
            self.vel *= accel;
        }
        let Some(npc) = npcs.get(target) else {
            return;
        };
        let npc_center = npc_center(npc);
        let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
        // `:37910`: `base.Distance(npc.Center)` — the centre-to-centre length.
        let distance = center.distance(npc_center);
        // `:37911`: `Remap(distance, range, 0, 0, 1) * 0.1 + 0.01`.
        let mut turn_rate = remap(distance, range, 0.0, 0.0, 1.0) * turn + 0.01;
        if speed < steer_speed {
            // `:37914`: the near-steer-speed ×10 pull-in.
            turn_rate = (turn_rate + 0.05) * remap(speed, 1.0, 3.0, 10.0, 1.0);
        }
        if features.fast_then_slow() {
            // `:37918`: the fast end of the sprint barely turns.
            turn_rate *= remap(speed, 30.0, 40.0, 1.0, 0.0);
        }
        // `:37920`: the turn — `Utils.rotateTowards`, the speed preserved.
        self.vel = rotate_towards(center, self.vel, npc_center, turn_rate);
        // `:37921`: the close-range pull — `Vector2.Zero.MoveTowards(npc -
        // Center, steerSpeed)`, blended by the 150f remap.
        let pull = move_towards(Vec2::ZERO, npc_center - center, steer_speed);
        self.vel += (pull - self.vel) * remap(distance, 150.0, 0.0, 0.0, 0.2);
    }

    /// `HandleMovement`'s gem branch (`Projectile.cs:17996-18056`): the
    /// blocked axes of the TileCollision resolution decide the branch, the
    /// port's stand-in being the per-axis destination-box probe the
    /// arrow/bomb arms use. CanBounce reverses each blocked axis at 0.8,
    /// spends one penetration, re-targets (owner-only, 2400f) and dies when
    /// the budget is spent; FastThenSlow bypasses the tile instead (the
    /// resolved velocity ×0.99, then the old direction pinned at 0.0001, and
    /// the bolt survives); a bolt with neither feature falls to the collision
    /// chain's default — one last step into the tile, then `Kill()`
    /// (`:19849-19853`).
    fn gem_bolt_move(&mut self, world: &World, npcs: &[Npc], features: &GemStaffFeatures) {
        if !self.tile_collide {
            // `UpdatePosition` (`:20433-20446`) on the collide-free kinds.
            self.pos += self.vel;
            return;
        }
        let last = self.vel;
        let hit_x = crate::player::collides_rect(world, self.pos.x + last.x, self.pos.y, self.w, self.h);
        let hit_y = crate::player::collides_rect(world, self.pos.x, self.pos.y + last.y, self.w, self.h);
        if !(hit_x || hit_y) {
            // `UpdatePosition` (`:20445`): the free move.
            self.pos += self.vel;
            return;
        }
        // `Collision.TileCollision`'s resolution zeroes the blocked axis
        // (`Collision.cs:299-330`) — the C#'s `velocity.X != lastVelocity.X`
        // test is exactly the port's `hit_x`.
        if hit_x {
            self.vel.x = 0.0;
        }
        if hit_y {
            self.vel.y = 0.0;
        }
        if features.can_bounce() {
            // `:18003-18011`: the ±0.8 reversal on the blocked axis.
            if hit_x {
                self.vel.x = -last.x * 0.8;
            }
            if hit_y {
                self.vel.y = -last.y * 0.8;
            }
            // `:18012-18013`: every bounce spends one penetration.
            self.penetrate -= 1;
            let survive = self.penetrate > 0;
            if features.homing() {
                // `:18014-18030`: the owner-only re-target at the 2400f
                // radius — the velocity is rewritten toward it at the old
                // speed ×0.8, and `ai[2] = num16 + 1` is written even when
                // the search failed (`num16 = -1` clears the index).
                let mut target: i32 = -1;
                if self.owner == 0 {
                    target = find_target_with_line_of_sight(
                        world, npcs, self.pos, self.w, self.h, 2400.0, &[],
                    );
                }
                if let Some(npc) = npcs.get(target as usize) {
                    let toward = npc_center(npc)
                        - Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                    // `:18025-18027`: a zero direction keeps the reversal.
                    if toward.length() > 0.0 {
                        self.vel = safe_normalize(toward) * last.length() * 0.8;
                    }
                }
                self.ai[2] = (target + 1) as f32;
            }
            // `:18047-18051`: `Collision.HitTiles` and the impact sound are
            // SKIPPED (no tile-effect or sound layer in the pool), and the
            // oldPos trail reset (`:18032-18038`) has no slot state.
            // `:18052-18055`: a spent budget kills on the bounce tick — the
            // C# still runs `UpdatePosition` first (`:19858`), which the
            // `pos += vel` below mirrors.
            self.pos += self.vel;
            if !survive {
                self.active = false;
                self.time_left = 0;
            }
            return;
        }
        if features.fast_then_slow() {
            // `:18040-18046`: the bypass — the resolved velocity ×0.99 (the
            // blocked axis already zeroed), then the old direction pinned at
            // 0.0001. The C#'s `flag7 = true` keeps the bolt alive and its
            // `flag8 = false` skips the HitTiles/sound pair.
            self.pos += self.vel * 0.99;
            self.vel = safe_normalize(last) * 0.0001;
            self.pos += self.vel;
            return;
        }
        // The collision chain's default (`:19849-19853`): one last step into
        // the tile, then the kill — the port's [`Projectile::kill_step`].
        self.kill_step(world);
    }

    /// `FinalizeProjectile`'s gem arm (`Projectile.cs:22057-22059`) →
    /// `FinalizeProjectile_GemStaves` (`:22063-22099`): the spawn-time
    /// bonuses the C# applies at the end of `NewProjectile`, after the `ai0`
    /// argument has landed — FastThenSlow hires one penetration and doubles
    /// `timeLeft`, CanBounce hires two, BiggerHitbox hires one, ArPenSpread
    /// adds four armor penetration, and the AllGems pair forces the 2/4 pair
    /// with `timeLeft = 90` instead of the ×2.
    fn gem_staves_finalize(&mut self) {
        let features = GemStaffFeatures::from_ai(self.ai[0]);
        let mut penetration = 0;
        let mut life_scale = 1;
        if features.fast_then_slow() {
            penetration = penetration.max(1);
            life_scale *= 2;
        }
        if features.can_bounce() {
            penetration = penetration.max(2);
        }
        if features.bigger_hitbox() {
            penetration = penetration.max(1);
        }
        // `:22084`: `armorPenetration += 4` — the slot carries no
        // armor-penetration field (recorded for ANALYSIS).
        if features.all_gems() || features.all_gems2() {
            penetration = 2;
            self.time_left = 90;
        } else {
            self.time_left *= life_scale;
        }
        self.max_penetrate += penetration;
        self.penetrate += penetration;
        // `:22098`: the armor-penetration write rides the +4 skip above.
    }

    /// `AI_047_MagnetSphere` (`Projectile.cs:41403-41444`): the ball's spin,
    /// its 4-frame animation, the ×0.98 damp over 2f and the same-type
    /// same-owner dedupe. The spawn-velocity cache lives in `ai[0..1]`, the
    /// frame counter in `localAI[3]` (the pool has no frame fields — the
    /// 5-frame sheet's wrap is the animation's only observable). Returns the
    /// bolt request `TryAttacking` fires: the spawn point and the NPC index.
    /// `AI_047_MagnetSphere` (`Projectile.cs:41403-41444`): the ball's spin,
    /// its 4-frame animation, the ×0.98 damp over 2f and — the pool-side
    /// half — the same-type same-owner dedupe. The spawn-velocity cache
    /// lives in `ai[0..1]`, the cooldown in `localAI[0]` and the frame
    /// counter in `localAI[3]` (the pool has no frame fields — the sheet's
    /// frame wrap is the animation's only observable). Returns the bolt
    /// request `TryAttacking` fires: the spawn point and the NPC index.
    fn ai_magnet_sphere(&mut self, world: &World, npcs: &[Npc]) -> Option<(Vec2, usize)> {
        // `:41405-41408`: the first tick caches the spawn velocity.
        if self.ai[0] == 0.0 && self.ai[1] == 0.0 {
            self.ai[0] = self.vel.x;
            self.ai[1] = self.vel.y;
        }
        // `:41409-41415`: the spin rate, signed by the flight direction.
        let rate = (self.vel.y.abs() + self.vel.x.abs()) * 0.001;
        if self.vel.x > 0.0 {
            self.rot += rate;
        } else {
            self.rot -= rate;
        }
        // `:41416-41423`: the 4-frame wrap (`frameCounter > 6` → `frame++`)
        // — SKIPPED: the slot carries no frame/frameCounter fields and the
        // projectile draw has no frame slicing yet (render follow-up).
        // `:41424-41427`: the damp while the flight outruns 2f.
        if self.vel.length() > 2.0 {
            self.vel *= 0.98;
        }
        // `AI_047_MagnetSphere_TryAttacking` (`:41446-41499`) — the owner
        // gate is the caller's. `:41448-41475`: the scan — the candidates
        // are the chaseable NPCs inside the Manhattan 300f with a clear
        // line; the C#'s `Main.rand.Next(num)` pick has no pool RNG, so the
        // first candidate in the `Main.npc` order (the port's slice order)
        // fires — the vanilla behaviour for the single-candidate case the
        // spawn usually sees (deviation, ANALYSIS).
        let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
        let mut target: Option<usize> = None;
        for (i, n) in npcs.iter().enumerate() {
            if !chaseable(n) {
                continue;
            }
            let c = npc_center(n);
            let manhattan = (center.x - c.x).abs() + (center.y - c.y).abs();
            if manhattan < 300.0
                && can_hit(world, center.x, center.y, 1.0, 1.0, n.x, n.y, n.size().0, n.size().1)
            {
                target = Some(i);
                break;
            }
        }
        // `:41489-41491`: the dying ball stops hunting (`timeLeft < 30`),
        // and a tick with no candidate arms nothing either.
        if self.time_left < 30 {
            return None;
        }
        let target = target?;
        // `:41494-41497`: the cooldown — `localAI[0]++` with a candidate
        // present, the bolt once past 8.
        self.local_ai[0] += 1.0;
        if self.local_ai[0] <= 8.0 {
            return None;
        }
        self.local_ai[0] = 0.0;
        Some((center + self.vel * 4.0, target))
    }

    /// `AI()`'s aiStyle-48 arm (`Projectile.cs:28204-28242`): the magnet
    /// bolt's lifetime — the out-of-world kill on a 5-tile margin, the
    /// delay-gated centre snapshot (`AI_048_GetStartPositionSettingDelay`
    /// `:53986-54000`: 255→4, 260/290→3, 294→9) and the alpha 255 write. The
    /// 290's one-shot sound is SKIPPED (no sound layer in the pool) and the
    /// movement is the generic blocked-move default (`kill_step`).
    fn ai_magnet_bolt(&mut self, world: &World) {
        let (cx, cy) = (self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
        let (tx, ty) = ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
        // `WorldGen.InWorld(..., 5)`: the 5-tile world margin.
        let in_world =
            tx >= 5 && ty >= 5 && tx < world.width - 5 && ty < world.height - 5;
        if !in_world {
            self.active = false;
            self.time_left = 0;
            return;
        }
        // `:28212-28214`: the snapshot at the delay tick.
        let delay = match self.kind {
            MAGNET_SPHERE_BOLT => 4,
            _ => 0,
        };
        if self.local_ai[0] == delay as f32 {
            self.local_ai[1] = cx;
            self.local_ai[2] = cy;
        }
        self.local_ai[0] += 1.0;
        // `:28216-28236`: every type writes alpha 255 (the 290's sound at
        // `localAI[0] == 1` SKIPPED).
        self.alpha = 255;
        self.kill_step(world);
    }

    /// `AI()`'s aiStyle-129 arm (`Projectile.cs:36198-36309`): the Spirit
    /// Flame's hunt-and-track machine. The 659's parameters (3/7.5/1/420/240
    /// — the steering weights, the retarget radius, the two timeLeft caps)
    /// override the defaults (10/5/40/300/180); the `ai[0]` slot is the NPC
    /// index (−1/−2 = the no-target states), `ai[1]` the orbit counter and
    /// `localAI[0]` the retarget cooldown. The orbital dust
    /// (`:36253-36276`), the sound and the frame animation are SKIPPED (no
    /// RNG, sound or dust layer in the pool, no frame fields on the slot).
    fn ai_spirit_flame(
        &mut self,
        world: &World,
        npcs: &[Npc],
        owner: Option<(f32, f32)>,
        player: Option<&crate::player::Player>,
    ) {
        let _ = (owner, player);
        // The 659 override (`:36202-36208`); the other kinds ride the
        // default shape.
        let (weight, boost, radius, retarget_life, no_target_life) =
            if self.kind == SPIRIT_FLAME {
                (3.0f32, 7.5f32, 1.0f32, 420i32, 240i32)
            } else {
                (10.0, 5.0, 40.0, 300, 180)
            };
        if self.kind == SPIRIT_FLAME {
            // `:36209-36210`: the retarget cooldown ticks down.
            if self.local_ai[0] > 0.0 {
                self.local_ai[0] -= 1.0;
            }
            // `:36212-36231`: the owner-only retarget — the closest-by-scan
            // chaseable NPC within 400f with a clear line to its centre or
            // top; the C# assigns in loop order (the last qualifying wins),
            // mirrored here. A failed search leaves `ai[0]` negative and the
            // flame keeps sitting.
            if self.local_ai[0] == 0.0 && self.ai[0] < 0.0 && self.owner == 0 {
                self.local_ai[0] = 5.0;
                let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                // The C# assigns inside the loop (`:36225`), so a later
                // candidate only wins when it is closer than the held index
                // (`:36218-36219`'s comparison turns live).
                let mut found: usize = usize::MAX;
                let mut found_distance = 400.0f32;
                for (i, n) in npcs.iter().enumerate() {
                    if !chaseable(n) {
                        continue;
                    }
                    let c = npc_center(n);
                    let distance = center.distance(c);
                    let closer = found == usize::MAX
                        || npc_center(&npcs[found]).distance(center) > distance;
                    let clear = can_hit_line(world, center.x, center.y, 0.0, 0.0, c.x, c.y, 0.0, 0.0)
                        || can_hit_line(world, center.x, center.y, 0.0, 0.0, n.x, n.y, 0.0, 0.0);
                    if closer && distance < 400.0 && clear {
                        found = i;
                        found_distance = distance;
                    }
                }
                let _ = found_distance;
                self.ai[0] = if found == usize::MAX {
                    -1.0
                } else {
                    found as f32
                };
                if found >= 0 {
                    // `:36228-36231`: a found target extends the life.
                    self.time_left = retarget_life;
                }
            }
            // `:36233-36244`: the alpha fade, the solid clamp, the floor.
            if self.time_left > 30 && self.alpha > 0 {
                self.alpha -= 12;
            }
            if self.time_left > 30
                && self.alpha < 128
                && crate::player::collides_rect(world, self.pos.x, self.pos.y, self.w, self.h)
            {
                self.alpha = 128;
            }
            if self.alpha < 0 {
                self.alpha = 0;
            }
            // `:36245-36252`: the 4-frame animation (`frameCounter > 4` →
            // `frame++`) — SKIPPED: no frame fields on the slot, no frame
            // slicing in the projectile draw (render follow-up).
            // `:36253-36276`: the orbital dust and its scale ramp — SKIPPED,
            // while `ai[1]++` (the orbit angle) rides the skip.
            self.ai[1] += 1.0;
        }
        // `:36278-36281`: the solid kill — the life drops to 2 inside a tile.
        if self.time_left > 2 && crate::player::collides_rect(world, self.pos.x, self.pos.y, self.w, self.h) {
            self.time_left = 2;
        }
        // `:36282-36293`: the tracking — while the target is live and past
        // the radius, the weighted blend pulls the flight.
        let target = self.ai[0] as i32;
        if target >= 0 {
            if let Some(n) = npcs.get(target as usize) {
                let c = npc_center(n);
                let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                let distance = center.distance(c);
                if distance > radius {
                    let mut toward = (c - center).normalize_or_zero();
                    if toward.is_nan() {
                        // `:36285-36287`: the NaN guard's `Vector2.UnitY`.
                        toward = Vec2::new(0.0, 1.0);
                    }
                    self.vel = (self.vel * (weight - 1.0) + toward * boost) / weight;
                }
                return;
            }
        }
        // `:36296-36308`: the no-target fallbacks — the −1 flame dies at 5,
        // the −2 waiter at the cap, and a stale index falls to −1.
        if self.ai[0] == -1.0 && self.time_left > 5 {
            self.time_left = 5;
        }
        if self.ai[0] == -2.0 && self.time_left > no_target_life {
            self.time_left = no_target_life;
        }
        if self.ai[0] >= 0.0 {
            self.ai[0] = -1.0;
        }
        self.kill_step(world);
    }

    /// The wind tail the C# applies after the AI sub-steps
    /// (`Projectile.cs:16777-16781`): `velocity.X += windSpeedCurrent *
    /// windPhysicsStrength`, under the altitude gate
    /// (`Center.Y < Main.worldSurface * 16`), a wall-free centre tile, and
    /// the three-way gate on the wind against the flight direction.
    /// `wind_speed` is the pool's `Main.windSpeedCurrent`
    /// ([`ProjectilePool::set_wind`]).
    ///
    /// The C#'s `MathHelper.Clamp(velocity.X, -16f, 16f)` on the next line
    /// discards its return value (`velocity.X` is a struct property, so the
    /// clamp cannot write back), which makes it a no-op in the source: the
    /// only bound on the tail is the `|velocity.X| < 16f` gate in front of it,
    /// so a single tick can carry `velocity.X` to at most 16 + |wind| × 0.1.
    /// Ported exactly as the C# runs, clamp included.
    fn wind_tail(&mut self, world: &World, wind_speed: f32) {
        let (cx, cy) = (self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
        let tx = (cx / TILE_PX) as i32;
        let ty = (cy / TILE_PX) as i32;
        let above_surface = (cy as f64) < world.world_surface * TILE_PX as f64;
        let wall_free = world.tile(tx, ty).is_none_or(|t| t.wall == 0);
        let headwind = (self.vel.x > 0.0 && wind_speed < 0.0)
            || (self.vel.x < 0.0 && wind_speed > 0.0)
            || self.vel.x.abs() < (wind_speed * WIND_PHYSICS_STRENGTH).abs() * 180.0;
        if !(above_surface && wall_free && headwind && self.vel.x.abs() < 16.0) {
            return;
        }
        self.vel.x += wind_speed * WIND_PHYSICS_STRENGTH;
    }

    /// The generic movement the C#'s `HandleMovement` runs for the styles
    /// without their own bounce or kill scan (`:17587` `Collision.TileCollision`
    /// + `:20468` `UpdatePosition`): a free move steps the box forward; a
    /// blocked move takes one last step into the tile and falls to the
    /// blocked-move chain's default — `base.position += base.velocity; Kill()`
    /// (`:19851-19855`) — so the projectile dies on contact. The port's blocked
    /// test is the per-axis destination-box probe the arrow/bomb arms use,
    /// standing in for the C#'s swept `TileCollision` (which zeroes the blocked
    /// axis, `Collision.cs:299-330`), so a dying projectile's exact death pixel
    /// can differ by the swept span while the kill does not.
    fn kill_step(&mut self, world: &World) {
        if !self.tile_collide {
            self.pos += self.vel;
            return;
        }
        let nx = self.pos.x + self.vel.x;
        let ny = self.pos.y + self.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, self.pos.y, self.w, self.h);
        let hit_y = crate::player::collides_rect(world, self.pos.x, ny, self.w, self.h);
        self.pos += self.vel;
        if hit_x || hit_y {
            self.active = false;
            self.time_left = 0;
        }
    }

    /// The per-slot draw (unchanged from the first pass).
    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        let Some(uv) = lookups.shuriken_uv() else {
            return;
        };
        let tx = (self.pos.x / TILE_PX) as i32;
        let ty = (self.pos.y / TILE_PX) as i32;
        let c = if light.w > 0 && light.h > 0 {
            light.get(
                tx.clamp(light.x0, light.x0 + light.w - 1),
                ty.clamp(light.y0, light.y0 + light.h - 1),
            )
        } else {
            [1.0; 3]
        };
        let pos = [
            self.pos.x - (camera.x - screen.x * 0.5 / zoom),
            self.pos.y - (camera.y - screen.y * 0.5 / zoom),
        ];
        r.push_ui_sprite_rot(
            pos,
            [SIZE * zoom, SIZE * zoom],
            uv,
            [c[0], c[1], c[2], 1.0],
            self.rot,
        );
    }
}

/// The magic (caster) families' shared constants (M4-PROJ-6, Phase 3).
/// `TileID.MagicalIceBlock` (`TileID.cs:883`) — the block the Ice Rod places
/// from its own projectile (`Projectile.cs:25842`).
pub(crate) const MAGICAL_ICE_BLOCK: u16 = 127;
/// `Main.windPhysicsStrength` (`Main.cs:66842`).
const WIND_PHYSICS_STRENGTH: f32 = 0.1;

/// The aiStyle-1 rider kinds of M4-PROJ-6 Phase 3 (`AI_001`'s per-type hooks):
/// GreenLaser 20, PurpleLaser 88, VenomFang 355, SkyFracture 660 — the fifth
/// (BookOfSkullsSkull 837) has its own arm and gates first.
const AI_STYLE_1_RIDERS: [u16; 4] = [GREEN_LASER, PURPLE_LASER, VENOM_FANG, SKY_FRACTURE];

/// `Projectile.ShouldUseWindPhysics`'s style list (`Projectile.cs:22110-22132`).
/// The styles whose arms this port has are 8/24/28 (Phase 3) and 29 (Phase
/// 4a's gem bolts) — the styles that already had ported arms (1 the arrows, 2
/// the shuriken, 16 the bombs) keep their pre-existing no-wind behaviour, and
/// the rest (10/14/…) port in a later phase or not at all.
fn uses_wind_physics(ai_style: u8) -> bool {
    matches!(ai_style, 8 | 24 | 28 | 29)
}

/// The `!tile.inActive() && tile.active() && tileSolid[type] &&
/// !tileSolidTop[type] && tile.slope() == 0 && !tile.halfBrick()`
/// conjunction `WorldGen.SolidTile` (`WorldGen.cs:70119-70127`) and
/// `Collision.CanHit`'s flank test (`Collision.cs:444-448`) share.
fn full_solid(t: &tr_core::tile::Tile) -> bool {
    t.flags & tr_core::tile::FLAG_INACTIVE == 0
        && t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`) — the C# reads a
/// *missing* tile as solid (`:70114-70116`, the try/catch of `:69965`), so
/// out-of-world coordinates come back `true`; the callers that care keep the
/// C#'s `WorldGen.InWorld` guard in front, exactly as the source does.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_none_or(full_solid)
}

/// `Collision.CanHit(Vector2, int, int, Vector2, int, int)`
/// (`Collision.cs:381-479`): the Manhattan line walk the C#'s per-type
/// `CanHit(...)` acquisition scans use (the Book of Skulls' targeting,
/// `Projectile.cs:56810-56828`). The anchors are the two boxes' tile centres
/// clamped into the world (`:388-423`, including the `maxTilesY - 40`
/// underworld cap); each step tests the pair of tiles beside the path for a
/// full-height block and stops when it steps onto a solid tile. A coordinate
/// outside the world reads as "cannot hit", like the C#'s null tiles.
fn can_hit(world: &World, x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
    let mut ax = ((x1 as i32 + (w1 * 0.5) as i32) / 16).clamp(1, world.width - 1);
    let mut ay = ((y1 as i32 + (h1 * 0.5) as i32) / 16).clamp(1, (world.height - 40).max(1));
    let bx = ((x2 as i32 + (w2 * 0.5) as i32) / 16).clamp(1, world.width - 1);
    let by = ((y2 as i32 + (h2 * 0.5) as i32) / 16).clamp(1, (world.height - 40).max(1));
    loop {
        if ax == bx && ay == by {
            return true;
        }
        if (ax - bx).abs() > (ay - by).abs() {
            ax += if ax >= bx { -1 } else { 1 };
            let (below, above) = (world.tile(ax, ay - 1), world.tile(ax, ay + 1));
            if below.is_none() || above.is_none() {
                return false;
            }
            if below.is_some_and(full_solid) && above.is_some_and(full_solid) {
                return false;
            }
        } else {
            ay += if ay >= by { -1 } else { 1 };
            let (left, right) = (world.tile(ax - 1, ay), world.tile(ax + 1, ay));
            if left.is_none() || right.is_none() {
                return false;
            }
            if left.is_some_and(full_solid) && right.is_some_and(full_solid) {
                return false;
            }
        }
        // The `do..while` exit (`:476-477`): the walk stops on the first
        // fully solid tile it steps onto.
        let Some(step) = world.tile(ax, ay) else {
            return false;
        };
        let walkable = step.flags & tr_core::tile::FLAG_INACTIVE != 0
            || !step.active()
            || !tr_core::tiledata::TILE_SOLID.has(step.kind)
            || tr_core::tiledata::TILE_SOLID_TOP.has(step.kind);
        if !walkable {
            return false;
        }
    }
}

/// `NPC.CanBeChasedBy(attacker, ignoreDontTakeDamage)` (`NPC.cs:90030-90041`):
/// alive, `lifeMax > 5` and open to damage. The C#'s `chaseable`, `friendly`
/// and `immortal` flags have no counterparts on the port's slot (every ported
/// NPC is chaseable and hostile and none is immortal), so the three fields the
/// pool can observe carry the gate. The C#'s `ignoreDontTakeDamage: true` form
/// plus its separate `!dontTakeDamage` test lands on the same predicate.
fn chaseable(n: &Npc) -> bool {
    n.hp > 0 && n.max_hp > 5 && !n.dont_take_damage
}

/// `NPC.Center` (`Entity.cs`: `position + Size / 2f`) for the pool's slots.
fn npc_center(n: &Npc) -> Vec2 {
    let (w, h) = n.size();
    Vec2::new(n.x + w * 0.5, n.y + h * 0.5)
}

/// `Collision.CanHitLine(Vector2, int, int, Vector2, int, int)`
/// (`Collision.cs:588-778`): the line-of-sight walk the mounts
/// (`Mount.cs:4912`/`:5220`), the Life Drain (`Player.cs:14067`) and the
/// ranged re-targets use. The anchors are the two boxes' tile centres clamped
/// into the world exactly as [`can_hit`] clamps them (`:594-625`); the walk
/// alternates the dominant axis, tests the three tiles beside the current
/// position for a solid block (`:679-697`, `:723-741`) and stops on the first
/// tile it steps onto that blocks (`:761-769`). A missing tile reads as a
/// blocked walk, like the C#'s try/catch (`:774-777`).
///
/// The walk's opacity test is the four-term `!inActive() && active() &&
/// tileSolid[type] && !tileSolidTop[type]` conjunction (`:694`, `:738`,
/// `:766`) — unlike [`can_hit`]'s flank test it carries no slope or halfBrick
/// term, so this helper does not reuse [`full_solid`].
#[allow(dead_code)] // Phase 4's later arms (aiStyle 129, the riders) call it
pub(crate) fn can_hit_line(world: &World, x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
    // `:590-593`: the anchors — `(int)((X + (float)(Width / 2)) / 16f)`.
    let clx = |v: i32| v.clamp(1, world.width - 1);
    let cly = |v: i32| v.clamp(1, (world.height - 40).max(1));
    let anchor_x = |x: f32, w: f32| clx(((x + (w * 0.5) as i32 as f32) / TILE_PX) as i32);
    let anchor_y = |y: f32, h: f32| cly(((y + (h * 0.5) as i32 as f32) / TILE_PX) as i32);
    let (mut ax, mut ay) = (anchor_x(x1, w1), anchor_y(y1, h1));
    let (bx, by) = (anchor_x(x2, w2), anchor_y(y2, h2));
    // `:626-627`: the axis deltas.
    let dx = (ax - bx).abs() as f32;
    let dy = (ay - by).abs() as f32;
    if dx == 0.0 && dy == 0.0 {
        // `:628-631`: the same tile always hits.
        return true;
    }
    // `:632-652`: the per-axis step weights — the dominant axis walks one
    // tile per sweep, the recessive one `dominant / recessive` (0f on a pure
    // axis).
    let (mut step_x, mut step_y) = (1.0f32, 1.0f32);
    if dx == 0.0 || dy == 0.0 {
        if dx == 0.0 {
            step_x = 0.0;
        }
        if dy == 0.0 {
            step_y = 0.0;
        }
    } else if dx > dy {
        step_x = dx / dy;
    } else {
        step_y = dy / dx;
    }
    // `:653-663`: the two error accumulators, the traversal state (2 = the
    // x sweep, 1 = the y sweep; it opens on 2 when the target is below), the
    // remaining steps per axis and the two directions.
    let (mut err_x, mut err_y) = (0.0f32, 0.0f32);
    let mut sweep = if ay < by { 2 } else { 1 };
    let (mut left_x, mut left_y) = (dx as i32, dy as i32);
    let dir_x = (bx - ax).signum();
    let dir_y = (by - ay).signum();
    let mut reached = false;
    let mut done = false;
    loop {
        // `:670-759`: one sweep of the current axis.
        if sweep == 2 {
            err_x += step_x;
            let steps = err_x as i32;
            err_x -= steps as f32;
            for _ in 0..steps {
                // `:679-690`: the null guards, then the (above, here, below)
                // trio — the first solid one blocks the walk.
                let (above, here, below) = (
                    world.tile(ax, ay - 1),
                    world.tile(ax, ay),
                    world.tile(ax, ay + 1),
                );
                if above.is_none() || here.is_none() || below.is_none() {
                    return false;
                }
                if above.is_some_and(line_solid)
                    || below.is_some_and(line_solid)
                    || here.is_some_and(line_solid)
                {
                    return false;
                }
                if left_x == 0 && left_y == 0 {
                    reached = true;
                    break;
                }
                ax += dir_x;
                left_x -= 1;
                if left_x == 0 && left_y == 0 && steps == 1 {
                    done = true;
                }
            }
            // `:710-713`: the other axis still has steps left.
            if left_y != 0 {
                sweep = 1;
            }
        } else {
            err_y += step_y;
            let steps = err_y as i32;
            err_y -= steps as f32;
            for _ in 0..steps {
                // `:723-734`: the (left, here, right) trio.
                let (left, here, right) = (
                    world.tile(ax - 1, ay),
                    world.tile(ax, ay),
                    world.tile(ax + 1, ay),
                );
                if left.is_none() || here.is_none() || right.is_none() {
                    return false;
                }
                if left.is_some_and(line_solid)
                    || right.is_some_and(line_solid)
                    || here.is_some_and(line_solid)
                {
                    return false;
                }
                if left_x == 0 && left_y == 0 {
                    reached = true;
                    break;
                }
                ay += dir_y;
                left_y -= 1;
                if left_x == 0 && left_y == 0 && steps == 1 {
                    done = true;
                }
            }
            if left_x != 0 {
                sweep = 2;
            }
        }
        // `:761-769`: the tile the sweep stepped onto.
        let Some(tile) = world.tile(ax, ay) else {
            return false;
        };
        if line_solid(tile) {
            return false;
        }
        // The `do..while (!(flag | flag2))` exit (`:770-772`).
        if reached || done {
            return true;
        }
    }
}

/// The `!tile.inActive() && tile.active() && tileSolid[type] &&
/// !tileSolidTop[type]` conjunction `Collision.CanHitLine`'s walk tests
/// (`Collision.cs:694`, `:738`, `:766`): the four terms without the slope and
/// halfBrick pair [`full_solid`] adds for `Collision.CanHit`'s flank test.
fn line_solid(t: &tr_core::tile::Tile) -> bool {
    t.flags & tr_core::tile::FLAG_INACTIVE == 0
        && t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
}

/// `Projectile.FindTargetWithLineOfSight` (`Projectile.cs:45627-45660`): the
/// nearest chaseable NPC within `max_range` that has a clear `Collision.CanHit`
/// line — `-1` when none is found. The search is distance-strict
/// (`num2 < num`, `:45656`) and the anchors are the targeter's box.
///
/// `immune` is the C#'s `projectile.localNPCImmunity[i] != 0` skip
/// (`:45645-45648`) — the row lives on the pool's per-slot array, which the
/// arm signature cannot reach, so the gem arm passes it empty and the
/// deviation is recorded there.
fn find_target_with_line_of_sight(
    world: &World,
    npcs: &[Npc],
    pos: Vec2,
    w: f32,
    h: f32,
    max_range: f32,
    immune: &[i32],
) -> i32 {
    let center = Vec2::new(pos.x + w * 0.5, pos.y + h * 0.5);
    let mut best = max_range;
    let mut result: i32 = -1;
    for (i, n) in npcs.iter().enumerate() {
        if immune.get(i).is_some_and(|v| *v != 0) {
            continue;
        }
        if !chaseable(n) {
            continue;
        }
        // `:45651`: `targeter.Distance(npc.Center)` — the centre-to-centre
        // length, then the CanHit line test.
        let distance = center.distance(npc_center(n));
        if distance < best && can_hit(world, pos.x, pos.y, w, h, n.x, n.y, n.size().0, n.size().1) {
            best = distance;
            result = i as i32;
        }
    }
    result
}

/// `Utils.GetLerpValue(from, to, t, clamped: true)` (`Utils.cs:315-340`):
/// the clamped ramp — 0/1 outside the from-range, linear inside. The branches
/// are the C#'s, since it treats a descending range (`from > to`) as clamped
/// from the *other* side.
fn get_lerp_value(from: f32, to: f32, t: f32) -> f32 {
    if from < to {
        if t < from {
            return 0.0;
        }
        if t > to {
            return 1.0;
        }
    } else {
        if t < to {
            return 1.0;
        }
        if t > from {
            return 0.0;
        }
    }
    (t - from) / (to - from)
}

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)` (`Utils.cs:345-348`)
/// — `MathHelper.Lerp(toMin, toMax, GetLerpValue(fromMin, fromMax, fromValue,
/// clamped: true))`, XNA's `a + (b - a) * amount`. The gem bolt's steering
/// uses it for the turn rate and the close-range pull.
fn remap(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * get_lerp_value(from_min, from_max, from_value)
}

/// `MathHelper.WrapAngle` (XNA's body: the `IEEERemainder(angle, TwoPi)`
/// remainder plus the ±2π fix-up). `round_ties_even` is
/// `IEEERemainder`'s round-to-nearest-integer.
fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    if angle > -PI && angle <= PI {
        return angle;
    }
    let rem = angle - (angle / TAU).round_ties_even() * TAU;
    if rem <= -PI {
        rem + TAU
    } else if rem > PI {
        rem - TAU
    } else {
        rem
    }
}

/// `Utils.AngleTowards(curAngle, targetAngle, maxChange)` (`Utils.cs:2053-2069`):
/// the wrapped shortest-way turn, clamped to `max_change`.
fn angle_towards(cur: f32, target: f32, max_change: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let mut cur = wrap_angle(cur);
    let target = wrap_angle(target);
    if cur < target {
        if target - cur > PI {
            cur += TAU;
        }
    } else if cur - target > PI {
        cur -= TAU;
    }
    cur += (target - cur).clamp(-max_change, max_change);
    wrap_angle(cur)
}

/// `Vector2.SafeNormalize(v, defaultValue)` (`Utils.cs:1837-1846`): the
/// zero/NaN direction falls back to `defaultValue`.
fn safe_normalize(v: Vec2) -> Vec2 {
    if v == Vec2::ZERO || v.is_nan() {
        Vec2::ZERO
    } else {
        v.normalize()
    }
}

/// `Vector2.MoveTowards(this Vector2, Vector2, float)` (`Utils.cs:1772-1786`):
/// `from` walks toward `target` by at most `max`, snapping on arrival.
fn move_towards(from: Vec2, target: Vec2, max: f32) -> Vec2 {
    let delta = target - from;
    if delta.length() < max {
        return target;
    }
    from + safe_normalize(delta) * max
}

/// `Utils.rotateTowards(currentPosition, currentVelocity, targetPosition,
/// maxChange)` (`Utils.cs:1934-1939`): the velocity turned by at most
/// `max_change` toward the target, its length preserved.
fn rotate_towards(from: Vec2, vel: Vec2, to: Vec2, max_change: f32) -> Vec2 {
    let len = vel.length();
    // `vel.ToRotation()` and `position.AngleTo(target)` (`Entity.cs:186-190`).
    let angle = angle_towards(
        vel.y.atan2(vel.x),
        (to.y - from.y).atan2(to.x - from.x),
        max_change,
    );
    Vec2::new(angle.cos(), angle.sin()) * len
}

/// The 1000-slot pool with the shared immunity state.
#[derive(Debug)]
pub struct ProjectilePool {
    slots: Vec<Projectile>,
    /// `localNPCImmunity[slot][npc]` — flattened `MAX_PROJECTILES × 256`
    /// (the C# per-projectile array is `Main.maxNPCs`-sized). The index is the
    /// NPC's position in the app's `Vec<Npc>`: stable between death compactions
    /// (documented in ANALYSIS — the C# indexes stable NPC slots).
    local_immunity: Vec<i32>,
    /// `perIDStaticNPCImmunity[immunityIdentity = type, npc]` → the update
    /// count after which a hit is allowed again (`:13972`, check `:604-607`).
    id_static: HashMap<(u16, u16), u64>,
    /// `Main.GameUpdateCount`.
    update_count: u64,
    /// `Main.windSpeedCurrent` as the AI arms' wind tail sees it
    /// (`Projectile.cs:16777-16781`). `0f` = off: the C# gate
    /// `Main.windPhysics` is `false` by default and never set true in the
    /// 1.4.5.8 decompile (`Main.cs:66841-66842`), so the tail is inert until
    /// the app's wind is wired in through [`ProjectilePool::set_wind`] — no
    /// ported arm's behaviour changes meanwhile.
    wind_speed: f32,
}

impl Default for ProjectilePool {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectilePool {
    pub fn new() -> Self {
        Self {
            slots: vec![Projectile::default(); MAX_PROJECTILES],
            local_immunity: vec![0; MAX_PROJECTILES * 256],
            id_static: HashMap::new(),
            update_count: 0,
            wind_speed: 0.0,
        }
    }

    /// The wind vector the AI arms' tail applies
    /// (`Projectile.cs:16777-16781`, the port's `Main.windSpeedCurrent`).
    pub fn set_wind(&mut self, wind_speed: f32) {
        self.wind_speed = wind_speed;
    }

    /// Sets the slot's crit flag (the throw path rolls it at spawn time).
    pub fn set_crit(&mut self, slot: usize, crit: bool) {
        if let Some(p) = self.slots.get_mut(slot) {
            p.crit = crit;
        }
    }

    pub fn slots(&self) -> &[Projectile] {
        &self.slots
    }

    #[cfg(test)]
    fn slots_mut(&mut self) -> &mut [Projectile] {
        &mut self.slots
    }

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|p| p.active).count()
    }

    /// `Projectile.NewProjectile` → `NewProjectileSetup` (`:10823-10880`): the
    /// first inactive slot; when the pool is full, recycle
    /// `FindOldestProjectile` (`:10811-10821` — lowest `timeLeft`, which puts
    /// dead slots first because `Kill` leaves them at 0). Returns `None` for
    /// the C#'s "1000" failure sentinel.
    pub fn spawn(
        &mut self,
        kind: u16,
        owner: u8,
        pos: Vec2,
        vel: Vec2,
        damage: i32,
        knock_back: f32,
        crit: bool,
    ) -> Option<usize> {
        let mut slot = None;
        for (i, p) in self.slots.iter().enumerate() {
            if !p.active {
                slot = Some(i);
                break;
            }
        }
        let slot = match slot {
            Some(i) => i,
            None => {
                let mut oldest = 0usize;
                let mut lowest = i32::MAX;
                for (i, p) in self.slots.iter().enumerate() {
                    if p.time_left < lowest {
                        oldest = i;
                        lowest = p.time_left;
                    }
                }
                oldest
            }
        };
        self.slots[slot] = Projectile {
            active: true,
            kind,
            owner,
            pos,
            vel,
            damage,
            knock_back,
            crit,
            ..Default::default()
        };
        // A spawned slot starts with clean local immunity.
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
        Some(slot)
    }

    /// The hostile shot spawn (`NPC.cs`'s `Projectile.NewProjectile`
    /// arms — the archer's arrow 1, the Destroyer's laser 100): the
    /// owner folds to 0 (the single-player quirk), `friendly = false`,
    /// the crit false (the hostile crits ride the expert rolls, not the
    /// item stream).
    pub fn spawn_hostile(
        &mut self,
        kind: u16,
        pos: Vec2,
        vel: Vec2,
        damage: i32,
        knock_back: f32,
    ) -> Option<usize> {
        // The vanilla's NewProjectile applies `SetDefaults` for hostile
        // shots (the box/flight/penetrate/extraUpdates) — the port routes
        // the data-carrying spawn, so the hostile kinds move like the
        // player's (the pre-base spawn path left them inert).
        let slot = self.spawn_with_data(kind, pos.x, pos.y, vel.x, vel.y, damage, knock_back)?;
        self.slots[slot].friendly = false;
        Some(slot)
    }

    /// The hostile shot with the explicit `timeLeft` override — the
    /// AI_037 arm's `Main.projectile[num12].timeLeft = 300`
    /// (`NPC.cs:49891`).
    pub fn spawn_hostile_timed(
        &mut self,
        kind: u16,
        pos: Vec2,
        vel: Vec2,
        damage: i32,
        knock_back: f32,
        time_left: i32,
    ) -> Option<usize> {
        let slot = self.spawn_hostile(kind, pos, vel, damage, knock_back)?;
        self.slots[slot].time_left = time_left;
        Some(slot)
    }

    /// Configure the immunity system of a just-spawned slot (the C# sets these
    /// in the per-type `SetDefaults`).
    pub fn set_immunity(
        &mut self,
        slot: usize,
        max_penetrate: i32,
        uses_local: bool,
        local_hit_cooldown: i32,
        uses_id_static: bool,
        id_static_hit_cooldown: i32,
    ) {
        let p = &mut self.slots[slot];
        p.max_penetrate = max_penetrate;
        p.penetrate = max_penetrate;
        p.uses_local_immunity = uses_local;
        p.local_hit_cooldown = local_hit_cooldown;
        p.uses_id_static_immunity = uses_id_static;
        p.id_static_hit_cooldown = id_static_hit_cooldown;
    }

    /// The tile-fall spawn (`SpawnFallingBlockProjectile`'s
    /// `Projectile.NewProjectile(..., 0f, 0.41f, projType, dmg, ...)` +
    /// `ai[0] = 1f`, `WorldGen.cs:86071-86073`): the 10×10 sand ball falls
    /// with the tile-fall gravity and places its tile on landing.
    pub fn spawn_falling(
        &mut self,
        tile_kind: u16,
        proj_kind: u16,
        damage: i32,
        x: f32,
        y: f32,
    ) -> Option<usize> {
        let mut slot = None;
        for (i, p) in self.slots.iter().enumerate() {
            if !p.active {
                slot = Some(i);
                break;
            }
        }
        let slot = slot?;
        self.slots[slot] = Projectile {
            active: true,
            kind: proj_kind,
            owner: 0,
            pos: Vec2::new(x, y),
            rot: 0.0,
            vel: Vec2::new(0.0, 0.41),
            damage,
            knock_back: 0.0,
            crit: false,
            time_left: tr_core::falling::TIME_LEFT_DEFAULT,
            max_penetrate: 1,
            penetrate: 1,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -1,
            extra_updates: 0,
            ai: [1.0, 0.0, 0.0],
            // The sand ball's `SetDefaults` arm (`:1131-1140`) touches neither
            // `alpha` nor `scale` — the reset values (`:779`, `:785`) hold.
            local_ai: [0.0; 3],
            alpha: 0,
            scale: 1.0,
            // The sand-ball box (`GetSandfallProjData`'s 10×10 family); the
            // flight is the custom falling-tile mode, not an aiStyle arm.
            w: 10.0,
            h: 10.0,
            tile_collide: false,
            arrow: false,
            trap: false,
            bounces: 0,
            falling_tile: Some(tile_kind),
            detonated: false,
            ..Default::default()
        };
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
        Some(slot)
    }

    /// `NewProjectile` + `SetDefaults` in one: applies the family's
    /// [`proj_data`] slice (the box, the penetrate/timeLeft overrides, the
    /// extraUpdates) on top of the spawn arguments. `None` when the kind has
    /// no ported data — the caller decides whether that is fatal.
    pub fn spawn_with_data(
        &mut self,
        kind: u16,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        damage: i32,
        knock_back: f32,
    ) -> Option<usize> {
        self.spawn_with_data_ai(kind, x, y, vx, vy, damage, knock_back, 0.0, 0.0)
    }

    /// `spawn_with_data` with the C# `NewProjectile`'s `ai0/ai1` arguments
    /// (the fireworks fountain's tile anchor, the bunny cannon's
    /// `1, owner + 1` — M7-WIRE-2). Every other arm passes 0; the C#
    /// signature's trailing `ai2` has no ported caller yet, so this seam is
    /// the one place a family that needs it extends. The port has a single
    /// player, so the C#'s `Owner` parameter (`:10862`, default 255 = unset)
    /// folds to `0` — the local player — here.
    pub fn spawn_with_data_ai(
        &mut self,
        kind: u16,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        damage: i32,
        knock_back: f32,
        ai0: f32,
        ai1: f32,
    ) -> Option<usize> {
        let data = proj_data(kind)?;
        let mut slot = None;
        for (i, p) in self.slots.iter().enumerate() {
            if !p.active {
                slot = Some(i);
                break;
            }
        }
        let slot = slot?;
        self.slots[slot] = Projectile {
            active: true,
            kind,
            owner: 0,
            pos: Vec2::new(x, y),
            rot: 0.0,
            vel: Vec2::new(vx, vy),
            damage,
            knock_back,
            crit: false,
            time_left: data.time_left,
            max_penetrate: data.penetrate,
            penetrate: data.penetrate,
            uses_local_immunity: data.uses_local_immunity,
            local_hit_cooldown: data.local_hit_cooldown,
            uses_id_static_immunity: data.uses_id_static_immunity,
            id_static_hit_cooldown: data.id_static_hit_cooldown,
            extra_updates: data.extra_updates,
            ai: [ai0, ai1, 0.0],
            // `SetDefaults` zeroes both scratch arrays (`:736-740`) — only the
            // C# `NewProjectile`'s `ai0/ai1` arguments ride along.
            local_ai: [0.0; 3],
            alpha: data.alpha,
            scale: data.scale,
            w: data.w,
            h: data.h,
            tile_collide: data.tile_collide,
            arrow: data.arrow,
            trap: data.trap,
            bounces: 0,
            falling_tile: None,
            detonated: false,
            blast_spawned: false,
            ..Default::default()
        };
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
        // `NewProjectile`'s tail (`Projectile.cs:11094`): `FinalizeProjectile`
        // runs after the `ai0/ai1` arguments have landed (`:10921-10923`), so
        // the gem bolts' spawn-time bonuses read the packed features out of
        // the fresh slot — the same order this seam reproduces.
        if GEM_BOLTS.contains(&kind) {
            self.slots[slot].gem_staves_finalize();
        }
        Some(slot)
    }

    /// `Projectile.Update` (`:16668-17208`): the AI sub-steps, the PVE damage
    /// walk, then the `timeLeft`/`penetrate` bookkeeping. The NPC per-owner
    /// i-frame decay runs with the NPC update (`NPC.cs:90454-90459`).
    pub fn update(
        &mut self,
        world: &mut World,
        npcs: &mut [Npc],
        owner: Option<(f32, f32)>,
        mut player: Option<&mut crate::player::Player>,
    ) {
        self.update_count += 1;
        for i in 0..MAX_PROJECTILES {
            if !self.slots[i].active {
                continue;
            }
            // `DecrementLocalImmuneTimeCounters` (`:16674`, `:17258-17277`):
            // the PVP frames plus the slot's local-NPC cooldowns decay first.
            if self.slots[i].uses_local_immunity {
                let base = i * 256;
                for v in &mut self.local_immunity[base..base + 256] {
                    *v = (*v - 1).max(0);
                }
            }
            let sub_steps = 1 + self.slots[i].extra_updates;
            for _ in 0..sub_steps {
                if self.slots[i].kind == SHURIKEN {
                    self.slots[i].ai_shuriken(world);
                }
                // The aiStyle dispatch (`AI()`'s if/else chain, the port's
                // `match data.ai_style`): the arrow/bullet family (aiStyle 1,
                // `if (this.aiStyle == 1) this.AI_001();`) and the
                // spear/dart family (aiStyle 14). The falling-tile mode
                // keeps its custom flight.
                //
                // Arm-signature contract for the magic (caster) families
                // (M4-PROJ-6, Phases 3-4): the owner-aware arms read
                // `Main.player[owner]` and the targeting reads `Main.npc`, so
                // the dispatch passes that context explicitly instead of
                // widening `update` (the `player`'s channel state rides the
                // `update` signature already):
                //
                // ```ignore
                // fn ai_<family>(&mut self, world: &World, npcs: &[Npc],
                //                 owner: Option<(f32, f32)>, player: Option<&Player>)
                // ```
                //
                // `owner` is the slot's owner center (the C# `Main.player
                // [this.owner]`); `spawn_with_data_ai` hardcodes
                // `owner: 0` = the local player, so a `None` owner center
                // means "no player resolved". The single carve-out is
                // `ai_ice_block` (aiStyle 22), which writes tiles
                // (`WorldGen.KillTile` `Projectile.cs:25695`,
                // `WorldGen.PlaceTile` `:25842` through
                // `tr_core::edit`): it takes `world: &mut World` via a
                // per-arm mutable reborrow at its dispatch call site.
                // The ported arms above keep their exact signatures.
                else if let Some(data) = proj_data(self.slots[i].kind) {
                    match data.ai_style {
                        // The aiStyle-1 riders of M4-PROJ-6 Phase 3 gate
                        // ahead of the generic arm: the Book of Skulls and
                        // the Laser/VenomFang/SkyFracture hooks wrap the
                        // shared flight instead of taking the plain arrow
                        // arm, so the plain arrows keep the generic flight
                        // only.
                        1 => match self.slots[i].kind {
                            BOOK_OF_SKULLS => self.slots[i].ai_book_of_skulls(world, npcs),
                            kind if AI_STYLE_1_RIDERS.contains(&kind) => {
                                self.slots[i].ai_style1_rider(world)
                            }
                            _ => self.slots[i].ai_arrow(world),
                        },
                        3 => self.slots[i].ai_boomerang(world, owner),
                        8 => self.slots[i].ai_ball_of_fire(world),
                        14 => self.slots[i].ai_spear(world),
                        16 => self.slots[i].ai_bomb(world),
                        18 => self.slots[i].ai_demon_scythe(world),
                        // The carve-out of the arm-signature doc block above:
                        // aiStyle 22 writes tiles, so it reborrows `world`
                        // mutably at its own call site.
                        22 => self.slots[i].ai_ice_block(world),
                        24 => self.slots[i].ai_crystal_storm(world),
                        28 => self.slots[i].ai_frost_bolt(world),
                        50 => self.slots[i].ai_inferno_fork(world),
                        // The aiStyle-29 gem bolts (`Projectile.cs:37643-38001`):
                        // the owner-aware arm of the family signature — its
                        // `FindTargetWithLineOfSight` refresh reads `Main.npc`,
                        // and the owner centre / the player ride along.
                        29 => {
                            self.slots[i].ai_gem_staff(world, npcs, owner, player.as_deref())
                        }
                        // The aiStyle-9 aimed spells (`:43195-43425`): the
                        // owner channel block reads the player's channel
                        // state, hence the family signature.
                        9 => self.slots[i].ai_magic_missile(world, npcs, owner, player.as_deref()),
                        // The aiStyle-47 magnet sphere (`:41403-41499`): the
                        // pool-side step — the dedupe walk and the bolt spawn
                        // need the whole pool, so it is not a slot arm.
                        47 => self.magnet_sphere_step(i, world, npcs),
                        // The aiStyle-48 magnet bolt (`:28204-28242`).
                        48 => self.slots[i].ai_magnet_bolt(world),
                        // The aiStyle-129 spirit flame (`:36198-36309`): the
                        // retarget reads `Main.npc` and the owner gate.
                        129 => {
                            self.slots[i].ai_spirit_flame(world, npcs, owner, player.as_deref())
                        }
                        _ => {}
                    }
                    // The wind tail (`:16777-16781`) runs after the AI
                    // sub-steps, for the wind-eligible new arms only (the
                    // ported arrow/bomb/shuriken arms keep their pre-existing
                    // no-wind behaviour — a recorded deviation, not a fix).
                    if uses_wind_physics(data.ai_style) {
                        self.slots[i].wind_tail(world, self.wind_speed);
                    }
                }
                // `AI_010`'s tile-fall mode (`ai[0] == 1f`): `velocity.Y +=
                // 0.41f` per tick, the position integrates the velocity (the
                // generic `position += velocity` of `Update`), and the
                // landing places the tile back.
                if self.slots[i].falling_tile.is_some() {
                    self.slots[i].vel.y =
                        (self.slots[i].vel.y + tr_core::falling::TILE_FALL_GRAVITY).min(10.0);
                    let vel = self.slots[i].vel;
                    self.slots[i].pos += vel;
                }
            }
            if let Some(tile_kind) = self.slots[i].falling_tile {
                // The landing: the bottom edge inside a solid tile → place
                // the tile back and kill the projectile.
                let p = &self.slots[i];
                if tr_core::falling::landed(world, p.pos.x, p.pos.y)
                    && tr_core::falling::can_place_back(world, p.pos.x, p.pos.y, tile_kind)
                {
                    let tx = (p.pos.x / 16.0) as i32;
                    let ty = (p.pos.y / 16.0) as i32;
                    if let Some(t) = world.tile_mut(tx, ty) {
                        t.kind = tile_kind;
                        t.flags |= tr_core::tile::FLAG_ACTIVE;
                        t.liquid = 0;
                    }
                    self.slots[i].active = false;
                    self.slots[i].time_left = 0;
                }
            }
            // `Kill()`'s bomb path (`:70662-70690`): `PrepareBombToBlow`
            // resizes and re-arms the damage, the PVE walk hits through the
            // resized box, `Kill_ExplodeTiles` digs the crater, then the
            // projectile dies.
            if self.slots[i].detonated {
                self.explode_bomb(i, world, npcs);
            }
            self.damage_pve(i, npcs);
            // The hostile shots' player contact (`Projectile.Damage_PVP`'s
            // single-player stand-in): the box overlap -> `Player.hurt`
            // (the iframes gate inside), the shot dies on the hit.
            if self.slots[i].active && !self.slots[i].friendly {
                self.damage_player(i, player.as_deref_mut());
            }
            // `:17199-17208`.
            let (time_left, penetrate) = {
                let p = &self.slots[i];
                (p.time_left - 1, p.penetrate)
            };
            let p = &mut self.slots[i];
            p.time_left = time_left;
            if time_left <= 0 || penetrate == 0 {
                p.active = false;
                p.time_left = 0;
            }
            // `Kill()`'s type-295 arm (`:74341-74350`): the Inferno Fork
            // bolt's death — whichever of the three paths above ended it —
            // spawns the blast, exactly once (the flag) and only for the bolt
            // kind (so the blast cannot chain a second spawn).
            if !self.slots[i].active {
                self.spawn_inferno_blast(i);
                // `Kill()`'s gem arm (`:75425-75484`): the gem bolts' death
                // burst — the AoeExplosion damage rides the dying slot, so it
                // runs at the same death hook the blast does.
                self.gem_bolt_aoe(i, npcs);
            }
        }
    }

    /// `Kill()`'s gem arm (`Projectile.cs:75425-75484`): the gem bolts' death
    /// burst. The sound and the 15 (+30) dust spawns are SKIPPED (no sound or
    /// dust layer in the pool), and the `AoeExplosion` feature's damage half
    /// rides the dying slot itself: `Resize(160, 160)` (`:22353-22359`
    /// recentres on the old centre), `damage × 0.35 / GetDamageMultiplier()`
    /// (the C#'s `(int)` truncation), `maxPenetrate = penetrate = -1`,
    /// `knockBack × 0.35`, the local-immunity reset and `Damage()` through the
    /// resized box, then the trailing `Resize(22, 22)`. Kind-gated to the
    /// seven [`GEM_BOLTS`] and feature-gated, so nothing else can die through
    /// it and the burst fires at most once per bolt (a dead slot never returns
    /// to `active`).
    fn gem_bolt_aoe(&mut self, slot: usize, npcs: &mut [Npc]) {
        let (kind, ai0) = match self.slots.get(slot) {
            Some(p) => (p.kind, p.ai[0]),
            None => return,
        };
        if !GEM_BOLTS.contains(&kind) {
            return;
        }
        let features = GemStaffFeatures::from_ai(ai0);
        if !features.aoe_explosion() {
            return;
        }
        // `:75433-75438`: the dust direction from the last velocity — dust
        // only, SKIPPED with the spawns above.
        let (center, damage, knock_back, cooldown) = {
            let p = &self.slots[slot];
            (
                Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5),
                p.damage,
                p.knock_back,
                p.local_hit_cooldown,
            )
        };
        let p = &mut self.slots[slot];
        // `:75467`: `Resize(160, 160)` — the recentre keeps the old centre.
        p.w = 160.0;
        p.h = 160.0;
        p.pos = Vec2::new(center.x - 80.0, center.y - 80.0);
        // `:75468-75472`: the damage half divides out the shoot side's
        // multiplier, the penetration opens to -1 and the knockback scales.
        let multiplier = 0.35 / features.damage_multiplier();
        p.damage = (damage as f32 * multiplier) as i32;
        p.max_penetrate = -1;
        p.penetrate = -1;
        p.knock_back = knock_back * 0.35;
        // `:75473-75481`: every local-immunity entry that is not the slot's
        // own cooldown clears (a no-op for a gem bolt, whose only writes are
        // the 60 the cooldown itself arms — ported for its shape).
        let base = slot * 256;
        for v in &mut self.local_immunity[base..base + 256] {
            if *v != cooldown {
                *v = 0;
            }
        }
        // `:75482`: `Damage()` — the walk through the resized box.
        // `Projectile.Damage()` (`:14087-14119`) tests the projectile's
        // rectangle against the NPC's, so the AoE rides the box walk, not
        // `damage_pve`'s fixed centre point (the point test would sit on
        // the already-hit centre NPC and the burst would deal nothing).
        self.damage_pve_boxed(slot, npcs);
        // `:75483`: the trailing `Resize(22, 22)`.
        let (cx, cy) = {
            let p = &self.slots[slot];
            (p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
        };
        let p = &mut self.slots[slot];
        p.w = 22.0;
        p.h = 22.0;
        p.pos = Vec2::new(cx - 11.0, cy - 11.0);
    }

    /// The dedupe scan (`:41428-41438`): every other slot of the same kind
    /// and owner with a shorter life and past 30 gets `timeLeft = 30`.
    fn magnet_sphere_dedupe(&mut self, slot: usize) {
        let (kind, owner, life) = {
            let p = &self.slots[slot];
            (p.kind, p.owner, p.time_left)
        };
        for i in 0..MAX_PROJECTILES {
            if i == slot {
                continue;
            }
            let other = &mut self.slots[i];
            if other.active
                && other.owner == owner
                && other.kind == kind
                && life > other.time_left
                && other.time_left > 30
            {
                other.time_left = 30;
            }
        }
    }

    /// The pool-side half of aiStyle 47: the dedupe walk, then the arm, then
    /// the bolt spawn (`:41498` — `NewProjectile(..., 255, damage,
    /// knockBack, owner)` at `Center + velocity * 4`, the velocity
    /// normalized to 6 toward the NPC centre). The C#'s `TryAttacking` is
    /// owner-only, so the whole step gates on the local player (slot owner
    /// 0).
    fn magnet_sphere_step(&mut self, slot: usize, world: &World, npcs: &[Npc]) {
        self.magnet_sphere_dedupe(slot);
        if self.slots[slot].owner != 0 {
            return;
        }
        let request = self.slots[slot].ai_magnet_sphere(world, npcs);
        let Some((spawn, target)) = request else {
            return;
        };
        let (damage, knock_back) = {
            let p = &self.slots[slot];
            (p.damage, p.knock_back)
        };
        let to = npc_center(&npcs[target]) - spawn;
        // `:41504-41507`: the 6f normalization (`6 / length`).
        let speed = 6.0 / to.length().max(f32::MIN_POSITIVE);
        self.spawn_with_data(
            MAGNET_SPHERE_BOLT,
            spawn.x,
            spawn.y,
            to.x * speed,
            to.y * speed,
            damage,
            knock_back,
        );
    }

    /// `Kill()`'s type-295 arm (`Projectile.cs:74341-74350`): the Inferno
    /// Fork bolt's death spawns its blast 296 at the bolt's centre with
    /// `damage * 0.75` (the C#'s `(int)` truncation) and the same
    /// knockback/owner. Owner-only (`Main.myPlayer == this.owner`), one-shot
    /// per bolt ([`Projectile::blast_spawned`]), and kind-gated so the blast
    /// itself — which rides the same aiStyle — never chains another spawn.
    fn spawn_inferno_blast(&mut self, slot: usize) {
        let p = &self.slots[slot];
        if p.blast_spawned || p.kind != INFERNO_FORK_BOLT || p.owner != 0 {
            return;
        }
        let (cx, cy) = (p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let damage = (p.damage as f32 * 0.75) as i32;
        let knock_back = p.knock_back;
        // One-shot per bolt: the flag rides the *bolt*, written before the
        // spawn — the C#'s `NewProjectile` finds the just-freed bolt slot free
        // (`FindOldestProjectile` starts at index 0), so the blast lands in the
        // bolt's own slot and the bolt's state does not survive the tick either
        // way. Writing it first keeps the guard true even if the pool is full.
        self.slots[slot].blast_spawned = true;
        // The C#'s `NewProjectile` takes the *centre* and offsets the box; the
        // port's spawn takes the top-left (`spawn_with_data_ai`'s
        // `position = (x, y)`), so the blast's 150×150 box is re-centred.
        let Some(row) = proj_data(INFERNO_BLAST) else {
            return;
        };
        self.spawn_with_data(
            INFERNO_BLAST,
            cx - row.w * 0.5,
            cy - row.h * 0.5,
            0.0,
            0.0,
            damage,
            knock_back,
        );
    }

    /// `Kill()`'s bomb path: `PrepareBombToBlow` (`Projectile.cs:46770-
    /// 46844`) resizes the box and re-arms the damage, `Damage()` walks the
    /// NPCs through the resized box, `Kill_ExplodeTiles` (`:77265-77380`)
    /// digs the radius circle, and the slot deactivates.
    fn explode_bomb(&mut self, slot: usize, world: &mut World, npcs: &mut [Npc]) {
        let (w, h, damage, knock_back) = match self.slots[slot].kind {
            // `:46775-46781`: the bombs — 128×128, damage 100, knockback 8.
            BOMB | STICKY_BOMB | BOUNCY_BOMB | BOMB_FISH => (128.0, 128.0, 100, 8.0),
            // `:46787-46791`: the dynamite — 250×250, damage 250,
            // knockback 10.
            DYNAMITE => (250.0, 250.0, 250, 10.0),
            _ => {
                self.slots[slot].detonated = false;
                return;
            }
        };
        let p = &mut self.slots[slot];
        // `PrepareBombToBlow`: the box recentres on the old center.
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        p.w = w;
        p.h = h;
        p.pos = Vec2::new(cx - w * 0.5, cy - h * 0.5);
        p.damage = damage;
        p.knock_back = knock_back;
        p.tile_collide = false;
        p.detonated = false;
        // `Damage()` — the PVE walk through the resized box.
        self.damage_pve(slot, npcs);
        // `Kill_ExplodeTiles`: the radius circle kills the tiles
        // (`ExplodeTiles` `:78707+` — the distance check per tile), the
        // 3×3 walls around each kill follow (`:78741-78762`).
        let radius = match self.slots[slot].kind {
            DYNAMITE => 7.0,
            _ => 4.0,
        };
        let p = &self.slots[slot];
        let ctx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
        let cty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
        let r = radius as i32;
        let min_i = (ctx - r).max(0);
        let max_i = (ctx + r).min(world.width - 1);
        let min_j = (cty - r).max(0);
        let max_j = (cty + r).min(world.height - 1);
        for ty in min_j..=max_j {
            for tx in min_i..=max_i {
                let dx = (tx as f32 - ctx as f32).abs();
                let dy = (ty as f32 - cty as f32).abs();
                if (dx * dx + dy * dy).sqrt() >= radius {
                    continue;
                }
                let Some(t) = world.tile(tx, ty) else {
                    continue;
                };
                if !t.active() || !can_explode_tile(t.kind, t.frame_y as i32 / 18) {
                    continue;
                }
                // The `KillTile` entry refuses the same three `CanKillTile`
                // cases (`CheckTileBreakability`, `WorldGen.cs:63537-63545`
                // — the container-above scan → 2, the locked door → 1); no
                // chest store is reachable here, so that case passes.
                if !tr_core::edit::can_kill_tile(world, tx, ty, |_, _| false) {
                    continue;
                }
                // `WorldGen.KillTile(i, j)` — the default drops stay on (the
                // C# passes no `noItem`).
                let _ = tr_core::edit::kill_tile(world, tx, ty, false);
                // `ExplodeTiles`' wall walk: the 3×3 around the killed tile.
                for wy in (ty - 1)..=(ty + 1) {
                    for wx in (tx - 1)..=(tx + 1) {
                        let Some(wt) = world.tile(wx, wy) else {
                            continue;
                        };
                        if wt.wall != 0 && wt.wall != 350 {
                            // `WorldGen.KillWall` — the port's wall break
                            // discards the item (the C# KillWall drops the
                            // wall item; the explosion path drops nothing —
                            // see ANALYSIS).
                            let _ = tr_core::edit::break_wall(world, wx, wy);
                        }
                    }
                }
            }
        }
        // The kill bookkeeping.
        self.slots[slot].active = false;
        self.slots[slot].time_left = 0;
    }

    /// The target-selection gate of `Damage_PVE` (`:12577-12583`): the slot's
    /// immunity system must allow a hit on NPC `npc` right now.
    fn target_allowed(&self, slot: usize, p: &Projectile, npc: u16) -> bool {
        if !p.uses_local_immunity && !p.uses_id_static_immunity {
            return true;
        }
        if p.uses_local_immunity {
            return self.local_immunity[slot * 256 + npc as usize] == 0;
        }
        match self.id_static.get(&(p.kind, npc)) {
            Some(stamp) => *stamp <= self.update_count,
            None => true,
        }
    }

    /// `Damage_PVE` + `Damage_PVE_Inner` (`:12562-12700`) with the on-hit
    /// immunity application (`:14087-14119`). Single-hit slots bypass the
    /// per-owner gate (`flag3`, `:12620-12624`) and set no immunity
    /// (`:14112-14119`).
    fn damage_pve(&mut self, slot: usize, npcs: &mut [Npc]) {
        let (damage, owner, uses_local, uses_id_static, max_penetrate, kb, crit, center) = {
            let p = &self.slots[slot];
            if p.damage <= 0 {
                return;
            }
            (
                p.damage,
                p.owner as usize,
                p.uses_local_immunity,
                p.uses_id_static_immunity,
                p.max_penetrate,
                p.knock_back,
                p.crit,
                p.pos + Vec2::splat(SIZE * 0.5),
            )
        };
        // `keepIterating`: the walk stops once a hit consumed the projectile.
        for (idx, n) in npcs.iter_mut().enumerate() {
            if n.hp <= 0 {
                continue;
            }
            let p = &self.slots[slot];
            if !self.target_allowed(slot, p, idx as u16) {
                continue;
            }
            // `:12620-12624`: the per-owner gate, bypassed by single-hitters.
            let single_hit = max_penetrate == 1 && !uses_local && !uses_id_static;
            if !single_hit && n.immune[owner] != 0 {
                continue;
            }
            if !n.contains(center.x, center.y) {
                continue;
            }
            let landed = n.hurt_from_projectile(damage, center.x, kb, crit);
            // `:14087-14119`: the immunity application.
            if uses_local && self.slots[slot].local_hit_cooldown != -2 {
                n.immune[owner] = 0;
                self.local_immunity[slot * 256 + idx] = self.slots[slot].local_hit_cooldown;
            } else if uses_id_static && self.slots[slot].id_static_hit_cooldown > 0 {
                n.immune[owner] = 0;
                self.id_static.insert(
                    (self.slots[slot].kind, idx as u16),
                    self.update_count + self.slots[slot].id_static_hit_cooldown as u64,
                );
            } else if max_penetrate != 1 {
                n.immune[owner] = OWNER_IMMUNE_TICKS;
            }
            if landed {
                let p = &mut self.slots[slot];
                p.penetrate -= 1;
                if p.penetrate == 0 {
                    break;
                }
            }
        }
    }

    /// The box walk `Projectile.Damage()` (`:14087-14119`) actually runs:
    /// the projectile's rectangle against the NPC's, the same immunity
    /// bookkeeping [`ProjectilePool::damage_pve`] applies. The point walk
    /// the normal PVE pass uses is the port's simplification for the
    /// flight-size boxes; the resized death bursts (the gem bolts' AoE)
    /// need the rectangle, or the burst's damage lands nowhere.
    fn damage_pve_boxed(&mut self, slot: usize, npcs: &mut [Npc]) {
        let (damage, owner, uses_local, uses_id_static, max_penetrate, kb, crit) = {
            let p = &self.slots[slot];
            if p.damage <= 0 {
                return;
            }
            (
                p.damage,
                p.owner as usize,
                p.uses_local_immunity,
                p.uses_id_static_immunity,
                p.max_penetrate,
                p.knock_back,
                p.crit,
            )
        };
        let (px, py, pw, ph) = {
            let p = &self.slots[slot];
            (p.pos.x, p.pos.y, p.w, p.h)
        };
        for (idx, n) in npcs.iter_mut().enumerate() {
            if n.hp <= 0 {
                continue;
            }
            let p = &self.slots[slot];
            if !self.target_allowed(slot, p, idx as u16) {
                continue;
            }
            let single_hit = max_penetrate == 1 && !uses_local && !uses_id_static;
            if !single_hit && n.immune[owner] != 0 {
                continue;
            }
            let (nw, nh) = n.size();
            // `myprojectile.GetRect().Intersects(npc.GetRect())` — the
            // rectangle overlap of the slot's box with the NPC's.
            let hit = px < n.x + nw && px + pw > n.x && py < n.y + nh && py + ph > n.y;
            if !hit {
                continue;
            }
            let (cx, cy) = (px + pw * 0.5, py + ph * 0.5);
            let landed = n.hurt_from_projectile(damage, cx, kb, crit);
            if uses_local && self.slots[slot].local_hit_cooldown != -2 {
                n.immune[owner] = 0;
                self.local_immunity[slot * 256 + idx] = self.slots[slot].local_hit_cooldown;
            } else if uses_id_static && self.slots[slot].id_static_hit_cooldown > 0 {
                n.immune[owner] = 0;
                self.id_static.insert(
                    (self.slots[slot].kind, idx as u16),
                    self.update_count + self.slots[slot].id_static_hit_cooldown as u64,
                );
            } else if max_penetrate != 1 {
                n.immune[owner] = OWNER_IMMUNE_TICKS;
            }
            if landed {
                let p = &mut self.slots[slot];
                p.penetrate -= 1;
                if p.penetrate == 0 {
                    break;
                }
            }
        }
    }

    /// The hostile shot's player contact: the box overlap with the
    /// player's rect hurts through `Player.hurt` (its iframe gate holds
    /// the per-tick repeats off) and kills the shot (the single-hit
    /// default).
    fn damage_player(&mut self, slot: usize, player: Option<&mut crate::player::Player>) {
        let Some(player) = player else {
            return;
        };
        let p = &self.slots[slot];
        if p.damage <= 0 {
            return;
        }
        let cx = p.pos.x + SIZE * 0.5;
        let cy = p.pos.y + SIZE * 0.5;
        let inside = cx >= player.x
            && cx < player.x + crate::player::WIDTH
            && cy >= player.y
            && cy < player.y + crate::player::HEIGHT;
        if !inside {
            return;
        }
        player.hurt(p.damage, cx);
        self.slots[slot].active = false;
        self.slots[slot].time_left = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn one_npc(world: &World) -> Npc {
        Npc::spawn_slime(world, 20).unwrap()
    }
    #[test]
    fn the_hostile_shot_hurts_the_player_and_dies() {
        // The hostile contact pass: the box overlap -> `Player.hurt`
        // (the iframes gate inside), the shot dies on the hit.
        let w = ground();
        let mut player = crate::player::Player::spawn(&w);
        let (pcx, pcy) = player.center();
        let mut pool = ProjectilePool::new();
        pool.spawn_hostile(
            100,
            Vec2::new(pcx - 5.0, pcy - 5.0),
            Vec2::new(2.0, 0.0),
            25,
            0.0,
        );
        assert!(pool.slots().iter().any(|p| p.active && !p.friendly));
        pool.update(&mut w.clone(), &mut [], Some((pcx, pcy)), Some(&mut player));
        assert!(player.hp < player.max_hp, "the shot hurt");
        assert!(!pool.slots().iter().any(|p| p.active), "the shot died");
        // The friendly shuriken at the same spot never hurts the player.
        let mut player2 = crate::player::Player::spawn(&w);
        let (pcx2, pcy2) = player2.center();
        let mut pool2 = ProjectilePool::new();
        pool2.spawn(
            SHURIKEN,
            0,
            Vec2::new(pcx2 - 5.0, pcy2 - 5.0),
            Vec2::new(2.0, 0.0),
            10,
            12.0,
            false,
        );
        pool2.update(
            &mut w.clone(),
            &mut [],
            Some((pcx2, pcy2)),
            Some(&mut player2),
        );
        assert_eq!(player2.hp, player2.max_hp, "the friendly shot spares");
    }

    /// One pool update plus the per-NPC decay the NPC update would run
    /// (`NPC.cs:90454-90459`), so multi-tick tests see the gates age.
    fn tick(world: &mut World, pool: &mut ProjectilePool, npc: &mut Npc) {
        for v in npc.immune.iter_mut() {
            *v = (*v - 1).max(0);
        }
        npc.hurt_cd = (npc.hurt_cd - 1.0 / 60.0).max(0.0);
        pool.update(world, std::slice::from_mut(npc), None, None);
    }

    #[test]
    fn spawn_takes_the_first_free_slot_and_recycles_the_dead() {
        let mut pool = ProjectilePool::new();
        let a = pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false);
        let b = pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false);
        assert_eq!((a, b), (Some(0), Some(1)));
        pool.slots_mut()[0].active = false;
        pool.slots_mut()[0].time_left = 0;
        assert_eq!(
            pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false),
            Some(0)
        );
    }

    #[test]
    fn full_pool_recycles_the_lowest_time_left() {
        let mut pool = ProjectilePool::new();
        for i in 0..MAX_PROJECTILES {
            pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false);
            pool.slots_mut()[i].time_left = 1000 - (i % 10) as i32;
        }
        // The lowest timeLeft (991) sits at slot 9 — `FindOldestProjectile`
        // scans front-to-back keeping the strict minimum (`:10811-10821`).
        assert_eq!(pool.slots()[9].time_left, 991);
        assert_eq!(
            pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false),
            Some(9)
        );
    }

    #[test]
    fn single_hit_shuriken_dies_without_touching_immunity() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        pool.update(&mut world, std::slice::from_mut(&mut npc), None, None);
        assert!(
            !pool.slots()[slot].active,
            "penetrate 1 → dead after the hit"
        );
        assert_eq!(pool.slots()[slot].penetrate, 0);
        assert!(npc.hp < npc.max_hp);
        // `:14112-14119`: a single-hit projectile applies no immunity.
        assert_eq!(npc.immune[0], 0);
    }

    #[test]
    fn local_immunity_gates_rehits_for_the_cooldown() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        // A 3-hit piercing shot with the 12-tick local cooldown (the C# sets
        // `usesLocalNPCImmunity` + `localNPCHitCooldown = 12` in SetDefaults).
        pool.set_immunity(slot, 3, true, 12, false, -1);
        let hp0 = npc.hp;
        tick(&mut world, &mut pool, &mut npc);
        assert!(pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].penetrate, 2);
        assert_eq!(npc.hp, hp0 - 10);
        // The local entry decays per update (`:17270-17277`) and gates every
        // re-hit until it reaches zero; a local hit zeroes
        // `NPC.immune[owner]` (`:14108-14109`). Set to 12 on tick 1, it frees
        // the gate on tick 13.
        for _ in 0..11 {
            tick(&mut world, &mut pool, &mut npc);
        }
        assert_eq!(npc.hp, hp0 - 10, "gated for the 11 middle ticks");
        tick(&mut world, &mut pool, &mut npc);
        assert_eq!(npc.hp, hp0 - 20, "tick 13: the local entry expired");
        assert_eq!(pool.slots()[slot].penetrate, 1);
    }

    #[test]
    fn default_penetrating_hits_use_the_owner_immunity_of_10() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        pool.set_immunity(slot, 5, false, -2, false, -1);
        let hp0 = npc.hp;
        tick(&mut world, &mut pool, &mut npc);
        assert_eq!(npc.hp, hp0 - 10);
        assert_eq!(npc.immune[0], 10, "`:14117`");
        for _ in 0..9 {
            tick(&mut world, &mut pool, &mut npc);
        }
        assert_eq!(npc.hp, hp0 - 10, "the owner gate holds for the 10 ticks");
        assert_eq!(pool.slots()[slot].penetrate, 4, "tick 1 already spent one");
        tick(&mut world, &mut pool, &mut npc);
        assert_eq!(npc.hp, hp0 - 20, "the 10th decay tick freed the gate");
        assert_eq!(npc.immune[0], 10, "re-applied");
        assert_eq!(
            pool.slots()[slot].penetrate,
            3,
            "the freed gate landed a hit"
        );
    }

    #[test]
    fn id_static_immunity_is_shared_across_slots_of_the_same_kind() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let a = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        let b = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        pool.set_immunity(a, 2, false, -2, true, 30);
        pool.set_immunity(b, 2, false, -2, true, 30);
        let hp0 = npc.hp;
        tick(&mut world, &mut pool, &mut npc);
        // Both slots overlap the NPC in the same tick: the ID-static stamp
        // written by the first hit (`:13972`) gates the second.
        assert_eq!(npc.hp, hp0 - 10);
        let stamp = *pool.id_static.get(&(SHURIKEN, 0)).unwrap();
        assert!(stamp > pool.update_count, "the stamp must still be live");
        for _ in 0..31 {
            tick(&mut world, &mut pool, &mut npc);
        }
        assert!(
            npc.hp < hp0 - 10,
            "the expired stamp (`<= GameUpdateCount`, `:604-607`) allowed a re-hit"
        );
    }

    #[test]
    fn time_left_expires_the_slot() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false)
            .unwrap();
        pool.slots_mut()[slot].time_left = 3;
        for _ in 0..3 {
            pool.update(&mut world, &mut [], None, None);
        }
        assert!(!pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].time_left, 0);
    }

    #[test]
    fn extra_updates_runs_the_ai_twice_per_tick() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(10.0 * TILE_PX, 20.0 * TILE_PX),
                Vec2::new(22.0, 0.0),
                10,
                0.0,
                false,
            )
            .unwrap();
        pool.slots_mut()[slot].extra_updates = 1;
        pool.update(&mut world, &mut [], None, None);
        assert!(
            (pool.slots()[slot].pos.x - (10.0 * TILE_PX + 44.0)).abs() < 0.01,
            "two AI steps × 22 px = 44 px, got {}",
            pool.slots()[slot].pos.x - 10.0 * TILE_PX
        );
    }
}
#[cfg(test)]
mod falling_tests {
    use super::*;

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    // The update needs `&mut World` (the landing placement) — the fixtures
    // build the world, then pass it mutably.

    fn put_solid(w: &mut tr_core::World, x: i32, y: i32) {
        use tr_core::tile::{FLAG_ACTIVE, Tile};
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_falling_block_falls_at_the_tile_fall_gravity() {
        let mut pool = ProjectilePool::new();
        let slot = pool.spawn_falling(53, 31, 10, 160.0, 200.0).unwrap();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut w = world();
        pool.update(&mut w, &mut npcs, None, None);
        // The spawn vel (0, 0.41) + one gravity step (0.41) = 0.82.
        assert!(
            (pool.slots()[slot].vel.y - 0.82).abs() < 0.001,
            "the tile-fall gravity (`AI_010`'s `velocity.Y += 0.41f`)"
        );
        assert!(pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].kind, 31, "the sand ball projectile");
    }

    #[test]
    fn the_falling_block_lands_and_places_the_tile() {
        let mut pool = ProjectilePool::new();
        // The spawn just above the solid row 24: the bottom edge (y + 10)
        // starts at 384 — the top of row 24 — and the first gravity step
        // pushes it inside.
        let slot = pool.spawn_falling(53, 31, 10, 160.0, 374.0).unwrap();
        let mut w = world();
        put_solid(&mut w, 10, 24);
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(&mut w, &mut npcs, None, None);
        assert!(!pool.slots()[slot].active, "the projectile died on landing");
        // The placement resolves through the box center → the tile row 23
        // (the last free row above the solid).
        let placed = w.tile(10, 23).unwrap();
        assert_eq!(placed.kind, 53, "the sand placed back");
        assert!(placed.active());
        assert_eq!(placed.liquid, 0);
    }

    #[test]
    fn the_falling_block_chain_refills_the_column() {
        // A two-sand column: breaking the support spawns the first fall; the
        // landing frees the second (the next edit pass re-checks).
        let mut pool = ProjectilePool::new();
        let mut w = world();
        put_solid(&mut w, 10, 24);
        use tr_core::tile::{FLAG_ACTIVE, Tile};
        for y in [21, 22] {
            let i = w.index(10, y);
            w.tiles[i] = Tile {
                kind: 53,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut npcs: Vec<Npc> = Vec::new();
        // Break the support (the tile at (10, 23)).
        {
            let i = w.index(10, 23);
            w.tiles[i] = Tile::default();
        }
        let Some(fall) = tr_core::falling::check_falling(&w, 10, 22) else {
            panic!("the sand at (10, 22) lost its support");
        };
        // Clear the falling tile (the C# `tileCache.ClearTile()`).
        {
            let i = w.index(10, 22);
            w.tiles[i] = Tile::default();
        }
        pool.spawn_falling(fall.tile_kind, fall.proj_kind, fall.damage, fall.x, fall.y);
        // Update until the landing.
        for _ in 0..40 {
            pool.update(&mut w, &mut npcs, None, None);
        }
        assert_eq!(w.tile(10, 23).unwrap().kind, 53, "the sand placed back");
        assert!(!pool.slots().iter().any(|p| p.active), "the fall is spent");
    }
}

#[cfg(test)]
mod arrow_tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    fn put_solid(w: &mut tr_core::World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_family_data_map_is_verbatim() {
        // `Projectile.cs:807-815` (the wooden arrow), `:817-827` (flame),
        // `:950-961` (the musket ball), `:1823-1832`/`:2707-2716`/`:9330-9339`
        // (the trap darts/spiky), `:6953-6966` (the poison dart),
        // `:2717-2727` (the spear).
        let arrow = proj_data(WOODEN_ARROW).unwrap();
        assert_eq!((arrow.w, arrow.h), (10.0, 10.0));
        assert_eq!(arrow.ai_style, 1);
        assert_eq!(arrow.time_left, 1200);
        assert!(arrow.arrow);
        let ball = proj_data(MUSKET_BALL).unwrap();
        assert_eq!((ball.w, ball.h), (4.0, 4.0));
        assert_eq!(ball.time_left, 600);
        assert_eq!(ball.extra_updates, 1);
        let dart = proj_data(TRAP_DART).unwrap();
        assert_eq!(dart.penetrate, -1);
        assert!(dart.trap);
        let poison = proj_data(POISON_DART).unwrap();
        assert!(!poison.tile_collide);
        assert_eq!(poison.extra_updates, 5);
        // The spear trap's spear (186, `Projectile.cs:2717-2727` — aiStyle 14)
        // and the spiky ball (185) both ride the ported families.
        let spear = proj_data(TRAP_SPEAR).unwrap();
        assert_eq!((spear.w, spear.h), (14.0, 14.0));
        assert_eq!(spear.ai_style, 14);
        assert_eq!(spear.time_left, 900);
        let spiky = proj_data(TRAP_SPIKY_BALL).unwrap();
        assert_eq!((spiky.w, spiky.h), (10.0, 10.0));
        assert_eq!(spiky.penetrate, -1);
        assert_eq!(TRAP_SPEAR, 186, "the C# ProjectileID.SpearTrap");
        // The flame jet 187 (aiStyle 38) is unported — no data, the emission
        // drops at the drain.
        assert!(proj_data(187).is_none());
        // The shuriken keeps its aiStyle-2 path, not the data table.
        assert!(proj_data(SHURIKEN).is_none());
        assert_eq!(SHURIKEN, 3, "the C# ProjectileID.Shuriken");
    }

    #[test]
    fn spawn_with_data_applies_the_family_slice() {
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(TRAP_SPEAR, 160.0, 200.0, 4.0, 0.0, 40, 2.0)
            .unwrap();
        let p = &pool.slots[slot];
        assert_eq!(p.kind, TRAP_SPEAR);
        assert_eq!(p.time_left, 900);
        assert_eq!(p.penetrate, -1);
        assert_eq!((p.w, p.h), (14.0, 14.0));
        assert!(p.trap);
        assert_eq!(p.damage, 40);
    }

    #[test]
    fn the_arrow_gains_gravity_after_15_ticks() {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOODEN_ARROW, 160.0, 200.0, 10.0, 0.0, 5, 0.0);
        let mut npcs = Vec::new();
        let mut w = world();
        // `ai[0]` increments on the same tick the check runs (`:57572` +
        // `:57580`), so the 15th update itself starts the gravity.
        for _ in 0..14 {
            pool.update(&mut w, &mut npcs, None, None);
        }
        assert_eq!(
            pool.slots[0].vel.y, 0.0,
            "no gravity for the first 14 ticks"
        );
        pool.update(&mut w, &mut npcs, None, None);
        assert!(
            (pool.slots[0].vel.y - 0.1).abs() < 1e-6,
            "the 15th tick starts the 0.1 gravity"
        );
        // The rotation rides the flight direction (`atan2 + 1.57`).
        assert!((pool.slots[0].rot - (0.1f32.atan2(10.0) + 1.57)).abs() < 1e-4);
    }

    #[test]
    fn the_arrow_breaks_on_tile_contact() {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOODEN_ARROW, 64.0, 10.0 * 16.0, 0.0, 16.0, 5, 0.0);
        let mut npcs = Vec::new();
        let mut w = world();
        for y in 20..24 {
            for x in 0..16 {
                put_solid(&mut w, x, y);
            }
        }
        // The 16 px/tick fall reaches the row-20 floor inside 15 ticks; the
        // blocked movement kills the arrow (`Update`'s tile collide).
        let mut died = false;
        for _ in 0..20 {
            pool.update(&mut w, &mut npcs, None, None);
            if !pool.slots[0].active {
                died = true;
                break;
            }
        }
        assert!(died, "the tile contact kills the arrow");
    }

    #[test]
    fn the_spear_dies_on_the_tile_scan() {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(TRAP_SPEAR, 8.0 * 16.0, 12.0 * 16.0, 0.0, 4.0, 40, 2.0);
        let mut npcs = Vec::new();
        let mut w = world();
        for x in 0..16 {
            put_solid(&mut w, x, 13);
        }
        // The scan runs before the move; the probe box reaches the row-13
        // floor on the third tick.
        let mut died = false;
        for _ in 0..5 {
            pool.update(&mut w, &mut npcs, None, None);
            if !pool.slots[0].active {
                died = true;
                break;
            }
        }
        assert!(
            died,
            "the aiStyle-14 center-probe scan kills on solid contact"
        );
    }

    #[test]
    fn the_spear_gains_gravity_after_5_ticks() {
        let mut pool = ProjectilePool::new();
        // Straight right over open air.
        pool.spawn_with_data(TRAP_SPEAR, 32.0, 16.0, 4.0, 0.0, 40, 2.0);
        let mut npcs = Vec::new();
        let mut w = world();
        for _ in 0..5 {
            pool.update(&mut w, &mut npcs, None, None);
        }
        assert_eq!(pool.slots[0].vel.y, 0.0, "no gravity for the first 5 ticks");
        pool.update(&mut w, &mut npcs, None, None);
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() < 1e-6,
            "the 6th tick adds 0.2"
        );
    }
}

#[cfg(test)]
mod boomerang_tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    fn put_solid(w: &mut tr_core::World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_registry_maps_the_boomerang_items() {
        // `itemdata.rs`: the Enchanted 55 / Wooden 284 / Ice 670 shoot the
        // projectiles 6 / 52 / 113, and the item data carries the throw
        // inputs the use path reads.
        assert_eq!(crate::registry::projectile(55), Some(ENCHANTED_BOOMERANG));
        assert_eq!(crate::registry::projectile(284), Some(WOODEN_BOOMERANG));
        assert_eq!(crate::registry::projectile(670), Some(ICE_BOOMERANG));
        let def = crate::registry::get(284).unwrap();
        assert_eq!(def.shoot_speed, 6.5);
        assert_eq!(def.knock_back, 5.0);
        assert_eq!(def.damage, 10);
        assert_eq!(def.class, crate::registry::DamageClass::Melee);
        assert!(!def.consumable, "the boomerang is not consumed on throw");
    }

    #[test]
    fn the_boomerang_data_is_verbatim() {
        // `Projectile.cs:864-873` / `:1345-1353` / `:2012-2022`: 22×22,
        // aiStyle 3, `penetrate = -1`.
        for kind in [ENCHANTED_BOOMERANG, WOODEN_BOOMERANG, ICE_BOOMERANG] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (22.0, 22.0));
            assert_eq!(d.ai_style, 3);
            assert_eq!(d.penetrate, -1);
            assert!(d.tile_collide);
        }
    }

    #[test]
    fn the_boomerang_switches_to_the_return_at_30_ticks() {
        // `:39024-39030` + `:39327-39331`: the outbound `ai[1]++`, the
        // return at `ai[1] >= 30f`.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOODEN_BOOMERANG, 320.0, 320.0, 10.0, 0.0, 10, 5.0);
        let mut w = world();
        for _ in 0..29 {
            pool.update(&mut w, &mut [], Some((320.0, 320.0)), None);
        }
        assert_eq!(pool.slots[0].ai[0], 0.0, "still outbound at tick 29");
        pool.update(&mut w, &mut [], Some((320.0, 320.0)), None);
        assert_eq!(pool.slots[0].ai[0], 1.0, "the return state at tick 30");
        assert_eq!(pool.slots[0].ai[1], 0.0);
    }

    #[test]
    fn the_outbound_boomerang_bounces_off_a_wall() {
        // `:14594-14602`: the outbound tile hit reverses the velocity and
        // sets the return state instead of killing.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(
            WOODEN_BOOMERANG,
            8.0 * 16.0,
            16.0 * 16.0,
            10.0,
            0.0,
            10,
            5.0,
        );
        let mut w = world();
        for y in 14..20 {
            for x in 12..24 {
                put_solid(&mut w, x, y);
            }
        }
        let mut bounced = false;
        for _ in 0..20 {
            pool.update(&mut w, &mut [], Some((100.0, 260.0)), None);
            if pool.slots[0].ai[0] == 1.0 {
                bounced = true;
                break;
            }
        }
        assert!(bounced, "the wall hit switched to the return state");
        assert!(
            pool.slots[0].vel.x < 0.0,
            "the velocity reversed (x = {})",
            pool.slots[0].vel.x
        );
    }

    #[test]
    fn the_returning_boomerang_passes_tiles_and_is_caught() {
        // The return state disables tile collision (`:39468-39470`) and the
        // owner's box catches it (`:39538-39544`).
        let mut pool = ProjectilePool::new();
        pool.slots[0] = Projectile {
            active: true,
            kind: WOODEN_BOOMERANG,
            owner: 0,
            pos: Vec2::new(400.0, 300.0),
            vel: Vec2::new(-9.0, 0.0),
            damage: 10,
            ..Default::default()
        };
        pool.slots[0].ai = [1.0, 0.0, 0.0];
        let mut w = world();
        // A wall directly on the path home.
        for y in 18..20 {
            for x in 20..22 {
                put_solid(&mut w, x, y);
            }
        }
        let mut caught = false;
        for _ in 0..40 {
            pool.update(&mut w, &mut [], Some((80.0, 308.0)), None);
            if !pool.slots[0].active {
                caught = true;
                break;
            }
        }
        assert!(caught, "the return flies through the wall and is caught");
    }

    #[test]
    fn the_stray_boomerang_dies_past_3000_px() {
        // `:39500-39505`: the 3000 px cut-off.
        let mut pool = ProjectilePool::new();
        pool.slots[0] = Projectile {
            active: true,
            kind: WOODEN_BOOMERANG,
            owner: 0,
            pos: Vec2::new(200.0, 200.0),
            vel: Vec2::new(30.0, 0.0),
            damage: 10,
            ..Default::default()
        };
        pool.slots[0].ai = [1.0, 0.0, 0.0];
        let mut w = world();
        let mut died = false;
        for _ in 0..120 {
            pool.update(&mut w, &mut [], Some((100.0, 200.0)), None);
            if !pool.slots[0].active {
                died = true;
                break;
            }
        }
        assert!(died, "the stray kills past the cut-off");
    }
}

#[cfg(test)]
mod bomb_tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    fn put(w: &mut tr_core::World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_bomb_data_is_verbatim() {
        // `Projectile.cs:1106-1112` + the fuse seeds `:10970-10993`: the
        // bomb 180 ticks, the dynamite 300; the sticky set has no plain
        // tile collide (`:1198-1205`).
        let bomb = proj_data(BOMB).unwrap();
        assert_eq!((bomb.w, bomb.h), (22.0, 22.0));
        assert_eq!(bomb.ai_style, 16);
        assert_eq!(bomb.time_left, 180);
        let dyna = proj_data(DYNAMITE).unwrap();
        assert_eq!((dyna.w, dyna.h), (10.0, 10.0));
        assert_eq!(dyna.time_left, 300);
        let sticky = proj_data(STICKY_BOMB).unwrap();
        assert!(!sticky.tile_collide);
        assert_eq!(proj_data(BOMB_FISH).unwrap().time_left, 180);
    }

    #[test]
    fn the_bomb_detonates_and_digs_the_crater() {
        // The fuse: 180 ticks → `timeLeft <= 3` → the kill path resizes to
        // 128×128/damage 100 (`PrepareBombToBlow`) and digs radius 4
        // (`Kill_ExplodeTiles`).
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(BOMB, 20.0 * 16.0, 20.0 * 16.0, 0.0, 0.0, 0, 0.0);
        let mut w = world();
        // A stone slab 20×20 centred on the bomb.
        for y in 14..34 {
            for x in 10..40 {
                put(&mut w, x, y, 1);
            }
        }
        let mut npcs: Vec<Npc> = Vec::new();
        for _ in 0..181 {
            pool.update(&mut w, &mut npcs, None, None);
        }
        assert!(!pool.slots[0].active, "the bomb exploded at the fuse end");
        // The crater: the tiles inside radius 4 are gone.
        assert!(
            w.tile(20, 20).unwrap().flags & FLAG_ACTIVE == 0,
            "the center dug"
        );
        assert!(
            w.tile(23, 20).unwrap().flags & FLAG_ACTIVE == 0,
            "3 tiles out dug"
        );
        // The tiles at distance ≥ 4 survive.
        assert!(
            w.tile(25, 20).unwrap().flags & FLAG_ACTIVE != 0,
            "5 tiles out survives"
        );
    }

    #[test]
    fn the_dynamite_digs_a_wider_crater() {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(DYNAMITE, 20.0 * 16.0, 20.0 * 16.0, 0.0, 0.0, 0, 0.0);
        let mut w = world();
        for y in 10..40 {
            for x in 5..45 {
                put(&mut w, x, y, 1);
            }
        }
        let mut npcs: Vec<Npc> = Vec::new();
        for _ in 0..301 {
            pool.update(&mut w, &mut npcs, None, None);
        }
        assert!(!pool.slots[0].active);
        assert!(
            w.tile(26, 20).unwrap().flags & FLAG_ACTIVE == 0,
            "6 tiles out dug"
        );
        assert!(
            w.tile(28, 20).unwrap().flags & FLAG_ACTIVE != 0,
            "8 tiles out survives"
        );
    }

    #[test]
    fn the_blast_guards_protect_the_chests_and_traps() {
        // `CanExplodeTile` (`:78640-78700`): the chests 21 survive via the
        // PreventsActuationUnder-style guard set (the 470/475 are in it;
        // the plain chest 21 dies in the C# — the port's tile 21 is the
        // chest, no guard) — the guarded ones are the sensors/dolls; the
        // trap 137 variants 1-4 survive pre-Golem, the dart variant 0 dies.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(BOMB, 20.0 * 16.0, 20.0 * 16.0, 0.0, 0.0, 0, 0.0);
        let mut w = world();
        for y in 14..34 {
            for x in 10..40 {
                put(&mut w, x, y, 1);
            }
        }
        // The display doll (475) and a guarded trap variant stand in the
        // blast radius.
        put(&mut w, 20, 19, 475);
        let i = w.index(21, 19);
        w.tiles[i] = Tile {
            kind: 137,
            frame_y: 18, // the superdart variant — guarded
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut npcs: Vec<Npc> = Vec::new();
        for _ in 0..181 {
            pool.update(&mut w, &mut npcs, None, None);
        }
        assert!(
            w.tile(20, 19).unwrap().flags & FLAG_ACTIVE != 0,
            "the doll survives the blast"
        );
        assert!(
            w.tile(21, 19).unwrap().flags & FLAG_ACTIVE != 0,
            "the guarded trap variant survives"
        );
    }

    #[test]
    fn the_flight_damage_zeroes_and_the_bounce_damps() {
        // `:47175-47176`: the flight damage zeroes on the first AI tick;
        // the bounce damp is −0.4 (`:19737-19762`).
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(BOMB, 16.0, 16.0, 8.0, 0.0, 55, 0.0);
        let mut w = world();
        pool.update(&mut w, &mut [], None, None);
        assert_eq!(pool.slots[0].damage, 0, "the flight damage zeroes");
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() > 1e-6,
            "no gravity before tick 5"
        );
        for _ in 0..5 {
            pool.update(&mut w, &mut [], None, None);
        }
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() < 1e-6,
            "the gravity tail adds 0.2"
        );
    }
}

/// The widened slot/row defaults (M4-PROJ-6 Phase 1): the C# `SetDefaults`
/// reset the magic families build on, and the `Player` channel state.
#[cfg(test)]
mod widened_state_tests {
    use super::*;
    use tr_core::World;
    use tr_core::tile::FLAG_ACTIVE;

    /// Every kind the data table carries today — the Phase-1 baseline set.
    /// The magic rows arrive in Phase 2 and carry their own state, so the
    /// sweep lives here rather than over `0..2048`.
    const BASELINE_ROWS: [u16; 48] = [
        WOODEN_ARROW,
        FLAME_ARROW,
        WOODEN_ARROW_HOSTILE,
        DESTROYER_LASER,
        GOLEM_FIREBALL,
        GOLEM_EYE_BEAM,
        EYE_LASER,
        CURSED_FLAME,
        EYE_FIRE,
        BRAIN_SCRAMBLER,
        RAY_GUNNER,
        SAUCER_DEATHRAY,
        SAUCER_MISSILE,
        SAUCER_LASER,
        SAUCER_SCRAP,
        DEERCLOPS_ICE_SPIKE,
        SHADOW_HAND,
        BOMB_SKELETRON,
        MUSKET_BALL,
        TRAP_DART,
        TRAP_SUPERDART,
        TRAP_SPIKY_BALL,
        TRAP_VENOM_DART,
        POISON_DART,
        TRAP_SPEAR,
        EXPLOSIVES_BLAST,
        LANDMINE_BLAST,
        CANNON_BOMB,
        CANNON_EXPLOSIVE,
        ENCHANTED_BOOMERANG,
        WOODEN_BOOMERANG,
        ICE_BOOMERANG,
        BOMB,
        BOUNCY_BOMB,
        STICKY_BOMB,
        BOMB_FISH,
        DYNAMITE,
        SCRAMBLER_SHOT,
        PIRATE_ARROW,
        MARTIAN_RAY,
        DUTCHMAN_SHELL,
        DD2_SPEAR,
        DD2_SPEAR_T3,
        DD2_BOMB,
        DARK_MAGE_MARKER,
        DARK_MAGE_FLAME,
        BETSY_BREATH,
        BETSY_FIREBALL,
    ];

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    /// The C# `SetDefaults` reset (`Projectile.cs:736-740`, `:741-745`,
    /// `:779`, `:785`, `:464`, `:466`, `:714-715`) every new field defaults to.
    fn assert_reset_defaults(p: &Projectile) {
        assert_eq!(
            p.local_ai, [0.0; 3],
            "both scratch arrays zero (`:736-740`)"
        );
        assert_eq!(p.alpha, 0, "`alpha = 0` (`:779`)");
        assert!(
            (p.scale - 1.0).abs() < f32::EPSILON,
            "`scale = 1f` (`:785`)"
        );
        assert!(
            !p.uses_local_immunity,
            "`usesLocalNPCImmunity` off (`:714`)"
        );
        assert_eq!(
            p.local_hit_cooldown, -2,
            "`localNPCHitCooldown = -2` (`:464`)"
        );
        assert!(
            !p.uses_id_static_immunity,
            "`usesIDStaticNPCImmunity` off (`:715`)"
        );
        assert_eq!(
            p.id_static_hit_cooldown, -1,
            "`idStaticNPCHitCooldown = -1` (`:466`)"
        );
    }

    /// The same reset on a data row.
    fn assert_reset_row(row: ProjData) {
        assert_eq!(row.alpha, 0, "`alpha = 0` (`:779`)");
        assert!(
            (row.scale - 1.0).abs() < f32::EPSILON,
            "`scale = 1f` (`:785`)"
        );
        assert!(
            !row.uses_local_immunity,
            "`usesLocalNPCImmunity` off (`:714`)"
        );
        assert_eq!(
            row.local_hit_cooldown, -2,
            "`localNPCHitCooldown = -2` (`:464`)"
        );
        assert!(
            !row.uses_id_static_immunity,
            "`usesIDStaticNPCImmunity` off (`:715`)"
        );
        assert_eq!(
            row.id_static_hit_cooldown, -1,
            "`idStaticNPCHitCooldown = -1` (`:466`)"
        );
    }

    #[test]
    fn test_dw_1_1_the_slot_carries_the_widened_ai_state() {
        // `Projectile.cs:336-338` (`maxAI = 3` `:79770`): the slot gains the
        // `localAI` array beside `ai`, plus `alpha`/`scale`.
        let p = Projectile::default();
        assert_eq!(p.local_ai, [0.0; 3]);
        assert_eq!(p.local_ai.len(), 3, "the C# `maxAI` is 3 (`:79770`)");
        assert_eq!(p.alpha, 0);
        assert!((p.scale - 1.0).abs() < f32::EPSILON);
        assert_reset_defaults(&p);
    }

    #[test]
    fn test_dw_1_1_every_row_carries_the_set_defaults_reset() {
        // No ported row overrides the six new fields yet (the magic rows are
        // Phase 2), so today's table must reproduce the C# reset verbatim —
        // the guard that the widening is behaviour-neutral.
        for kind in BASELINE_ROWS {
            let row = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} lost its row"));
            assert_reset_row(row);
        }
        // The magic kinds now carry rows (Phase 2) with their own state, so
        // they sit outside the baseline reset sweep — the Frost Staff bolt's
        // row (`:4497-4507`) overrides alpha/penetrate.
        let frost = proj_data(FROST_BOLT).unwrap_or_else(|| panic!("the Frost Staff bolt lost its row"));
        assert_eq!(frost.alpha, 255, "`alpha = 255` (`:4497-4507`)");
        assert_eq!(frost.penetrate, 2);
        assert!(
            proj_data(187).is_none(),
            "the flame jet stays a passthrough"
        );
    }

    #[test]
    fn test_dw_1_2_spawn_with_data_initialises_the_row_defaults() {
        // DW-1.2: the spawn applies the row on top of the arguments, and the
        // fields the row does not touch stay at the C# `SetDefaults` reset.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(TRAP_SPEAR, 160.0, 200.0, 4.0, 0.0, 40, 2.0)
            .unwrap();
        let p = &pool.slots[slot];
        assert_eq!((p.time_left, p.penetrate), (900, -1));
        assert_eq!((p.w, p.h), (14.0, 14.0));
        assert!(p.trap);
        assert_reset_defaults(p);
        assert_eq!(p.ai, [0.0, 0.0, 0.0], "`spawn_with_data` passes ai0/ai1 0");
    }

    #[test]
    fn test_dw_1_2_spawn_with_data_ai_passes_the_ai_args_and_zeroes_local_ai() {
        // The C# `NewProjectile`'s `ai0/ai1` arguments ride `ai`; `localAI` is
        // zeroed by `SetDefaults` (`:736-740`) — the Ice Rod's tile target
        // (Phase 5) arrives through `ai`, never `localAI`.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(EXPLOSIVES_BLAST, 1.0, 2.0, 0.0, 0.0, 500, 10.0, 7.0, -3.0)
            .unwrap();
        let p = &pool.slots[slot];
        assert_eq!(p.ai, [7.0, -3.0, 0.0]);
        assert_eq!(p.local_ai, [0.0; 3]);
        assert_reset_defaults(p);
    }

    #[test]
    fn test_dw_1_2_the_plain_spawn_paths_keep_the_reset_defaults() {
        // The `spawn` / `spawn_hostile*` constructors and a bare
        // `..Default::default()` slot literal all land on the same baseline.
        let mut pool = ProjectilePool::new();
        let a = pool
            .spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 12.0, false)
            .unwrap();
        assert_reset_defaults(&pool.slots[a]);
        let b = pool
            .spawn_hostile(EYE_LASER, Vec2::ZERO, Vec2::new(4.0, 0.0), 20, 2.0)
            .unwrap();
        assert_reset_defaults(&pool.slots[b]);
        // The tile-fall spawn (its `SetDefaults` arm `:1131-1140` touches
        // neither `alpha` nor `scale`).
        let c = pool.spawn_falling(53, 31, 10, 160.0, 200.0).unwrap();
        assert_reset_defaults(&pool.slots[c]);
    }

    #[test]
    fn test_dw_1_2_set_immunity_still_overrides_the_row_default() {
        // The `set_immunity` path keeps working on top of the widened spawn:
        // the C# sets the cooldown in the per-type `SetDefaults`
        // (`ApplyGemStaffStats`'s 60 ticks, `:10683-10689` — Phase 2 rides
        // the row, this test pins the manual path still lands).
        let mut w = world();
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        let mut npc = Npc::spawn_slime(&w, 20).unwrap();
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                TRAP_SPEAR,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                0.0,
                0.0,
                10,
                0.0,
            )
            .unwrap();
        pool.set_immunity(slot, 3, true, 12, false, -1);
        let p = &pool.slots[slot];
        assert!(p.uses_local_immunity);
        assert_eq!(p.local_hit_cooldown, 12);
        assert_eq!(p.max_penetrate, 3);
        // The gate the cooldown arms still bites: one hit, then the local
        // entry holds the re-hits off for 12 ticks.
        let hp0 = npc.hp;
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None);
        assert_eq!(npc.hp, hp0 - 10);
        assert_eq!(pool.slots[slot].penetrate, 2);
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None);
        assert_eq!(npc.hp, hp0 - 10, "the 12-tick cooldown gates the re-hit");
    }

    #[test]
    fn test_dw_1_2_an_unported_kind_spawns_nothing() {
        // The seam Phase 2 rides: a kind without a row returns `None` instead
        // of an inert slot (the M4-PROJ-1 silent-None trap). The magic kinds
        // have rows now (Phase 2); the follow-up aiStyles stay rowless.
        let mut pool = ProjectilePool::new();
        assert!(
            pool.spawn_with_data(114, 10.0, 10.0, 1.0, 0.0, 15, 3.0)
                .is_none(),
            "the Unholy Trident bolt (aiStyle 27) is a follow-up row"
        );
        assert!(
            pool.spawn_with_data(187, 10.0, 10.0, 1.0, 0.0, 15, 3.0)
                .is_none(),
            "the flame jet stays a passthrough"
        );
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_dw_1_3_the_player_channel_state_defaults_off() {
        // `Player.channel` (`Player.cs:52958-52968`) is off until a `channel`
        // item is held; Phase 5 writes it, Phase 4's style-9 arm reads it.
        let w = world();
        let p = crate::player::Player::spawn(&w);
        assert!(!p.channel);
        assert_eq!(p.channel_item, 0, "no item is being channelled");
        assert_eq!(p.channel_target, Vec2::ZERO);
    }
}

#[cfg(test)]
mod magic_data_tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// Every in-scope magic kind with its verbatim `SetDefaults` slice:
    /// (kind, w, h, aiStyle, penetrate, timeLeft, extraUpdates, tileCollide,
    /// alpha, scale, usesLocalNPCImmunity, localNPCHitCooldown) — the
    /// `Projectile.cs` per-type arms in `ProjectileID` order. `timeLeft`'s
    /// 3600 is the reset value (`:787`); the C# citation per arm rides the
    /// comment under each line.
    const MAGIC_ROWS: [(u16, f32, f32, u8, i32, i32, u32, bool, i32, f32, bool, i32); 28] = [
        // `:964-973` BallofFire — 16x16, aiStyle 8, alpha 100.
        (BALL_OF_FIRE, 16.0, 16.0, 8, 1, 3600, 0, true, 100, 1.0, false, -2),
        // `:974-984` MagicMissile — 32x32, aiStyle 9, alpha 100.
        (MAGIC_MISSILE, 32.0, 32.0, 9, 1, 3600, 0, true, 100, 1.0, false, -2),
        // `:1018-1029` GreenLaser — the aiStyle-1 rider.
        (GREEN_LASER, 4.0, 4.0, 1, 3, 600, 2, true, 0, 0.65, false, -2),
        // `:1095-1105` WaterBolt — `timeLeft /= 2` off the 3600 reset.
        (WATER_BOLT, 16.0, 16.0, 8, 10, 1800, 0, true, 255, 1.0, false, -2),
        // `:1162-1173` Flamelash — the 12-tick local cooldown.
        (FLAMELASH, 32.0, 32.0, 9, 2, 3600, 0, true, 0, 1.0, true, 12),
        // `:1283-1294` DemonScythe — scale 0.9.
        (DEMON_SCYTHE, 48.0, 48.0, 18, 5, 3600, 0, true, 100, 0.9, false, -2),
        // `:1594-1606` RainbowRodBullet — penetrate 3, cooldown 12.
        (RAINBOW_ROD_BULLET, 32.0, 32.0, 9, 3, 3600, 0, true, 0, 1.0, true, 12),
        // `:1607-1618` IceBlock — tileCollide false.
        (ICE_BLOCK, 16.0, 16.0, 22, 1, 3600, 0, false, 0, 1.0, false, -2),
        // `:1702-1712` PurpleLaser — extraUpdates 4.
        (PURPLE_LASER, 6.0, 6.0, 1, 3, 600, 4, true, 0, 0.65, false, -2),
        // `:1777-1790` CrystalStorm — alpha 50, scale 1.2.
        (CRYSTAL_STORM, 8.0, 8.0, 24, 1, 600, 0, true, 50, 1.2, false, -2),
        // `:1792-1804` CursedFlame — penetrate 2.
        (CURSED_FLAME_FRIENDLY, 16.0, 16.0, 8, 2, 3600, 0, true, 100, 1.0, false, -2),
        // `:2105-2158` + `:10683-10689` the gem bolts 121-126 — the shared
        // `ApplyGemStaffStats` group.
        (GEM_BOLT_AMETHYST, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        (GEM_BOLT_TOPAZ, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        (GEM_BOLT_SAPPHIRE, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        (GEM_BOLT_EMERALD, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        (GEM_BOLT_RUBY, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        (GEM_BOLT_DIAMOND, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        // `:3358-3368` BallofFrost.
        (BALL_OF_FROST, 16.0, 16.0, 8, 1, 3600, 0, true, 100, 1.0, false, -2),
        // `:3369-3377` MagnetSphereBall — timeLeft 900.
        (MAGNET_SPHERE_BALL, 38.0, 38.0, 47, 1, 900, 0, true, 0, 1.0, false, -2),
        // `:3378-3387` MagnetSphereBolt — extraUpdates 100, timeLeft 100.
        (MAGNET_SPHERE_BOLT, 8.0, 8.0, 48, 1, 100, 100, true, 0, 1.0, false, -2),
        // `:3849-3858` InfernoFriendlyBolt — alpha 255.
        (INFERNO_FORK_BOLT, 12.0, 12.0, 50, 1, 3600, 0, true, 255, 1.0, false, -2),
        // `:3859-3868` the bolt's blast — tileCollide false, penetrate -1.
        (INFERNO_BLAST, 150.0, 150.0, 50, -1, 3600, 0, false, 255, 1.0, false, -2),
        // `:4451-4463` VenomFang — the 10-tick cooldown, timeLeft 58.
        (VENOM_FANG, 12.0, 12.0, 1, 5, 58, 0, true, 255, 1.0, true, 10),
        // `:4497-4507` FrostBoltStaff — alpha 255, penetrate 2.
        (FROST_BOLT, 14.0, 14.0, 28, 2, 3600, 0, true, 255, 1.0, false, -2),
        // `:6461-6470` + `:10683-10689` AmberBolt — the family's seventh.
        (GEM_BOLT_AMBER, 10.0, 10.0, 29, 1, 300, 0, true, 255, 1.0, true, 60),
        // `:7083-7094` SpiritFlame — tileCollide false, penetrate -1.
        (SPIRIT_FLAME, 8.0, 8.0, 129, -1, 240, 0, false, 255, 1.0, false, -2),
        // `:7096-7107` SkyFracture — extraUpdates 1.
        (SKY_FRACTURE, 10.0, 10.0, 1, 1, 600, 1, true, 255, 1.0, false, -2),
        // `:8124-8135` BookOfSkullsSkull — tileCollide false, timeLeft 480.
        (BOOK_OF_SKULLS, 26.0, 26.0, 1, 3, 480, 0, false, 255, 1.0, false, -2),
    ];

    #[test]
    fn the_magic_family_data_map_is_verbatim() {
        // DW-2.1: every in-scope magic kind has its `proj_data` row, the
        // verbatim `SetDefaults` slice against the C# arms cited per row.
        for &(kind, w, h, ai, pen, tl, ex, tc, alpha, scale, uses_local, cd) in &MAGIC_ROWS {
            let row = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} lost its row"));
            assert_eq!(
                (row.w, row.h, row.ai_style, row.penetrate, row.time_left, row.extra_updates, row.tile_collide),
                (w, h, ai, pen, tl, ex, tc),
                "kind {kind}"
            );
            assert_eq!((row.alpha, row.scale), (alpha, scale), "kind {kind}");
            assert_eq!(
                (row.uses_local_immunity, row.local_hit_cooldown),
                (uses_local, cd),
                "kind {kind}"
            );
            // No magic arm touches the id-static immunity or the flags — the
            // C# reset holds (`:714-715`, `:466`, `:778-779`).
            assert!(!row.uses_id_static_immunity, "kind {kind}");
            assert_eq!(row.id_static_hit_cooldown, -1, "kind {kind}");
            assert!(!row.arrow && !row.trap, "kind {kind}");
        }
    }

    #[test]
    fn test_dw_2_1_the_rider_kinds_spawn_through_the_seam() {
        // DW-2.1: without a row, `spawn_with_data_ai`'s leading
        // `proj_data(kind)?` drops the shot silently (the M4-PROJ-1d trap) —
        // the five aiStyle-1 riders spawn and carry their rows onto the slot.
        let mut pool = ProjectilePool::new();
        // (kind, w, h, penetrate, timeLeft, extraUpdates, scale, alpha, tileCollide)
        let cases: [(u16, f32, f32, i32, i32, u32, f32, i32, bool); 5] = [
            (GREEN_LASER, 4.0, 4.0, 3, 600, 2, 0.65, 0, true),
            (PURPLE_LASER, 6.0, 6.0, 3, 600, 4, 0.65, 0, true),
            (BOOK_OF_SKULLS, 26.0, 26.0, 3, 480, 0, 1.0, 255, false),
            (VENOM_FANG, 12.0, 12.0, 5, 58, 0, 1.0, 255, true),
            (SKY_FRACTURE, 10.0, 10.0, 1, 600, 1, 1.0, 255, true),
        ];
        for (i, &(kind, w, h, pen, tl, ex, scale, alpha, tc)) in cases.iter().enumerate() {
            let slot = pool
                .spawn_with_data(kind, 32.0 * i as f32, 32.0, 4.0, 0.0, 15, 3.0)
                .unwrap_or_else(|| panic!("kind {kind} spawns nothing"));
            let p = &pool.slots[slot];
            assert_eq!(p.kind, kind);
            assert_eq!((p.w, p.h), (w, h), "kind {kind}");
            assert_eq!((p.penetrate, p.max_penetrate), (pen, pen), "kind {kind}");
            assert_eq!(p.time_left, tl, "kind {kind}");
            assert_eq!(p.extra_updates, ex, "kind {kind}");
            assert!((p.scale - scale).abs() < f32::EPSILON, "kind {kind}");
            assert_eq!((p.alpha, p.tile_collide), (alpha, tc), "kind {kind}");
            // The spawner's inputs ride on top.
            assert_eq!(p.damage, 15, "kind {kind}");
            assert_eq!(p.knock_back, 3.0, "kind {kind}");
            assert_eq!(pool.active_count(), i + 1);
        }
    }

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    #[test]
    fn test_dw_2_2_the_gem_bolt_rows_spawn_with_the_local_immunity() {
        // DW-2.2: `ApplyGemStaffStats` (`Projectile.cs:10683-10689`) —
        // friendly, `usesLocalNPCImmunity = true`, `localNPCHitCooldown =
        // 60`, `timeLeft = 300` — rides the row onto the slot through
        // `spawn_with_data`, for all seven gem kinds.
        let mut pool = ProjectilePool::new();
        for kind in [
            GEM_BOLT_AMETHYST,
            GEM_BOLT_TOPAZ,
            GEM_BOLT_SAPPHIRE,
            GEM_BOLT_EMERALD,
            GEM_BOLT_RUBY,
            GEM_BOLT_DIAMOND,
            GEM_BOLT_AMBER,
        ] {
            let slot = pool
                .spawn_with_data(kind, 32.0, 32.0, 0.0, 0.0, 15, 3.0)
                .unwrap_or_else(|| panic!("kind {kind} spawns nothing"));
            let p = &pool.slots[slot];
            assert!(p.uses_local_immunity, "kind {kind}");
            assert_eq!(p.local_hit_cooldown, 60, "kind {kind}");
            assert_eq!(p.time_left, 300, "kind {kind}");
            assert_eq!(p.alpha, 255, "kind {kind}");
            assert_eq!((p.w, p.h), (10.0, 10.0), "kind {kind}");
        }
        // The gate the cooldown arms still bites: one hit, then the local
        // entry holds the re-hits off for 60 ticks.
        let mut w = world();
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        let mut npc = Npc::spawn_slime(&w, 20).unwrap();
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn_with_data(GEM_BOLT_AMETHYST, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0, 10, 0.0)
            .unwrap();
        // The row's `penetrate = 1` would end the bolt on its first hit;
        // open the headroom so the 60-tick cooldown is what gates the second
        // hit (the immunity fields stay the row's).
        let p = &mut pool.slots_mut()[slot];
        p.max_penetrate = 3;
        p.penetrate = 3;
        let hp0 = npc.hp;
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None);
        assert_eq!(npc.hp, hp0 - 10);
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None);
        assert_eq!(npc.hp, hp0 - 10, "the 60-tick cooldown gates the re-hit");
    }

    #[test]
    fn test_dw_2_1_the_out_of_scope_kinds_stay_rowless() {
        // The follow-up aiStyles keep no row: the unported-kind spawn must
        // reach the caller as `None`, never an inert slot (the plan's OUT
        // list — the follow-up row and the hostile riders; the Golem's
        // fireball 258 already rides a pre-existing row, the hostile
        // style-8 rider of the same kind, and keeps it).
        for kind in [114u16, 116, 206, 297, 356, 409, 502, 617, 633] {
            assert!(proj_data(kind).is_none(), "kind {kind} must stay rowless");
        }
        assert!(proj_data(GOLEM_FIREBALL).is_some(), "the Golem's 258 row stays");
        // The saucer stand-ins (447/450, aiStyle 79/80 as style numbers) are
        // distinct from the magic kinds 79/80 — no collision.
        assert_eq!(proj_data(SAUCER_DEATHRAY).unwrap().ai_style, 79);
        assert_eq!(proj_data(RAINBOW_ROD_BULLET).unwrap().ai_style, 9);
        assert_eq!(proj_data(SAUCER_MISSILE).unwrap().ai_style, 80);
        assert_eq!(proj_data(ICE_BLOCK).unwrap().ai_style, 22);
        assert_eq!(proj_data(MAGIC_MISSILE).unwrap().ai_style, 9);
        assert_eq!(proj_data(BALL_OF_FROST).unwrap().ai_style, 8);
    }
}

#[cfg(test)]
mod magic_ai_tests {
    use super::*;
    use crate::npc::SlimeVariant;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A tile-free world: the flight tests run in open air, so the only
    /// collision that can trigger is the one the test places itself.
    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn put_tile(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// A solid wall one tile ahead of a `size`-sized projectile spawned at
    /// `(10·16, 10·16)`: the left edge starts on the tile-11 boundary, so the
    /// box's right edge stops just short of it and a step of `+2, 0` crosses
    /// in. Returns the x the projectile should spawn at. The C# tests the
    /// swept path; the port's per-arm model probes the destination box, so the
    /// wall has to sit exactly one tile past the edge.
    fn wall_ahead(w: &mut World, size: f32) -> f32 {
        let x = 11.0 * TILE_PX - size;
        let y0 = 10;
        let y1 = ((10.0 * TILE_PX + size - 0.01) / TILE_PX) as i32;
        for y in y0..=y1 {
            put_tile(w, 11, y, 1);
        }
        x
    }

    fn has_tile(w: &World, x: i32, y: i32, kind: u16) -> bool {
        w.tile(x, y).is_some_and(|t| t.active() && t.kind == kind)
    }

    /// One pool update with no NPC in range.
    fn step(pool: &mut ProjectilePool, w: &mut World) {
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(w, &mut npcs, None, None);
    }

    /// The `tick(world, pool, npc)` fixture of the main `mod tests`, for the
    /// calls that need an NPC slice.
    fn tick(world: &mut World, pool: &mut ProjectilePool, npc: &mut Npc) {
        for v in npc.immune.iter_mut() {
            *v = (*v - 1).max(0);
        }
        npc.hurt_cd = (npc.hurt_cd - 1.0 / 60.0).max(0.0);
        pool.update(world, std::slice::from_mut(npc), None, None);
    }

    /// The same fixture over a whole `Vec<Npc>` — the targeting arms read the
    /// `Main.npc` slice, so the scan tests hand the pool several at once.
    fn tick_npcs(world: &mut World, pool: &mut ProjectilePool, npcs: &mut [Npc]) {
        for n in npcs.iter_mut() {
            for v in n.immune.iter_mut() {
                *v = (*v - 1).max(0);
            }
            n.hurt_cd = (n.hurt_cd - 1.0 / 60.0).max(0.0);
        }
        pool.update(world, npcs, None, None);
    }

    // ---------------------------------------------------------------- DW-3.1
    // One multi-tick test per ported arm, pinning the C# constant the arm
    // turns on. Every assertion names its `Projectile.cs` line range.

    #[test]
    fn test_dw_3_1_style_8_gravity_starts_at_ai_1_20() {
        // `:24338-24345`: `ai[1]++` outside the 27/96/258 skips, then
        // `if (ai[1] >= 20f) velocity.Y += 0.2f`.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(
            BALL_OF_FIRE,
            10.0 * TILE_PX,
            10.0 * TILE_PX,
            1.0,
            0.0,
            20,
            0.0,
        );
        let mut w = world();
        for _ in 0..19 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[0];
        assert_eq!(p.ai[1], 19.0, "the timer ticks once per sub-step");
        assert_eq!(p.vel.y, 0.0, "no gravity through ai[1] = 19");
        step(&mut pool, &mut w);
        assert!(
            (pool.slots()[0].vel.y - 0.2).abs() < 1e-6,
            "ai[1] = 20 adds the first 0.2 (`:24341`)"
        );
        for _ in 0..5 {
            step(&mut pool, &mut w);
        }
        assert!(
            (pool.slots()[0].vel.y - 6.0 * 0.2).abs() < 1e-6,
            "one 0.2 per further tick"
        );
    }

    #[test]
    fn test_dw_3_1_style_8_the_fall_clamps_at_16_and_the_spin_rides_the_direction() {
        // `:24346-24357` (`rotation += 0.3f * direction`) and `:24358-24361`
        // (the `velocity.Y > 16f` clamp). `direction` has no slot field; it
        // tracks `velocity.X`'s sign (`:17016-17026`), positive here.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BALL_OF_FROST,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                1.0,
                0.0,
                20,
                0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].vel.y = 20.0;
        let mut w = world();
        step(&mut pool, &mut w);
        let p = &pool.slots()[slot];
        assert_eq!(p.vel.y, 16.0, "the fall is clamped to 16 (`:24359`)");
        assert!(
            (p.rot - 0.3).abs() < 1e-6,
            "one spin of 0.3 for a +X flight"
        );
    }

    #[test]
    fn test_dw_3_1_style_8_the_water_bolt_and_the_cursed_flame_skip_the_gravity_timer() {
        // `:24304-24306`: `if (type != 27 && type != 96 && type != 258)
        // ai[1]++;` — the three skips never reach the gravity. The third (the
        // Golem's 258) shares the branch.
        for (kind, note) in [(WATER_BOLT, "27"), (CURSED_FLAME, "96")] {
            let mut pool = ProjectilePool::new();
            pool.spawn_with_data(kind, 10.0 * TILE_PX, 10.0 * TILE_PX, 1.0, 0.0, 20, 0.0);
            let mut w = world();
            for _ in 0..40 {
                step(&mut pool, &mut w);
            }
            let p = &pool.slots()[0];
            assert_eq!(p.ai[1], 0.0, "kind {kind} ({note}) holds ai[1] at rest");
            assert_eq!(p.vel.y, 0.0, "kind {kind} ({note}) never gets gravity");
            assert!(p.active, "kind {kind} ({note}) keeps flying");
        }
    }

    #[test]
    fn test_dw_3_1_style_8_the_tile_bounce_counts_and_kills_at_the_type_threshold() {
        // The bounce arm (`:19573-19613`): `ai[0]++` on every blocked move and
        // the per-type budget — 5 default, 6 for the ball of fire 15
        // (`:19587`), 8 for the ball of frost 253 (`:19590`).
        for (kind, budget) in [(BALL_OF_FIRE, 6.0), (BALL_OF_FROST, 8.0)] {
            let mut pool = ProjectilePool::new();
            let slot = pool
                .spawn_with_data(kind, 10.0 * TILE_PX, 10.0 * TILE_PX, 2.0, 0.0, 20, 0.0)
                .unwrap();
            let mut w = world();
            // Walls on both sides of the 16×16 box: the first step hits tile 11
            // and the reversal carries the next one into tile 9, so the ball
            // keeps bouncing instead of escaping.
            put_tile(&mut w, 9, 10, 1);
            put_tile(&mut w, 11, 10, 1);
            pool.slots_mut()[slot].ai[0] = budget - 2.0;
            step(&mut pool, &mut w);
            assert!(
                pool.slots()[slot].active,
                "kind {kind} survives a bounce inside its {budget} budget"
            );
            assert!(
                (pool.slots()[slot].vel.x + 2.0).abs() < 1e-6,
                "the blocked axis reverses (`:19609-19612`)"
            );
            pool.slots_mut()[slot].ai[0] = budget - 1.0;
            step(&mut pool, &mut w);
            assert!(
                !pool.slots()[slot].active,
                "kind {kind} dies on the bounce past its budget {budget} (`:19591-19595`)"
            );
        }
    }

    #[test]
    fn test_dw_3_1_style_8_the_hostile_cursed_flame_dies_on_a_wall_instead_of_bouncing() {
        // The C#'s bounce arm is `else if (this.aiStyle == 8 && this.type !=
        // 96)` (`:19573`): the Spazmatism's spit falls through to the blocked-
        // move chain's default — `position += velocity; Kill()` (`:19851-19855`).
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(
            CURSED_FLAME,
            10.0 * TILE_PX,
            10.0 * TILE_PX,
            2.0,
            0.0,
            20,
            0.0,
        );
        let mut w = world();
        put_tile(&mut w, 11, 10, 1);
        step(&mut pool, &mut w);
        assert!(
            !pool.slots()[0].active,
            "the hostile cursed flame dies on the wall"
        );
        assert_eq!(
            pool.slots()[0].ai[0],
            0.0,
            "the bounce counter never runs for 96"
        );
    }

    #[test]
    fn test_dw_3_1_style_18_the_scythe_accelerates_from_ai_0_30_and_clamps_at_100() {
        // `:25455-25468`: nothing below 30, `velocity *= 1.06f` from 30 to
        // 99, and `ai[0] = 200f` at 100 stops the acceleration.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(
            DEMON_SCYTHE,
            10.0 * TILE_PX,
            10.0 * TILE_PX,
            1.0,
            0.0,
            20,
            0.0,
        );
        let mut w = world();
        for _ in 0..29 {
            step(&mut pool, &mut w);
        }
        assert_eq!(pool.slots()[0].ai[0], 29.0);
        assert_eq!(pool.slots()[0].vel.x, 1.0, "nothing happens below 30");
        step(&mut pool, &mut w);
        assert!(
            (pool.slots()[0].vel.x - 1.06).abs() < 1e-6,
            "the 30th tick is the first 1.06"
        );
        for _ in 0..4 {
            step(&mut pool, &mut w);
        }
        assert!(
            (pool.slots()[0].vel.x - 1.06f32.powi(5)).abs() < 1e-4,
            "1.06 per tick from 30"
        );
        // The clamp: park the counter at 99, one tick past the ceiling zeros
        // the acceleration instead of multiplying again.
        pool.slots_mut()[0].ai[0] = 99.0;
        let before = pool.slots()[0].vel.x;
        step(&mut pool, &mut w);
        assert_eq!(pool.slots()[0].ai[0], 200.0, "the 100-clamp (`:25466`)");
        let after = pool.slots()[0].vel.x;
        step(&mut pool, &mut w);
        assert_eq!(
            pool.slots()[0].vel.x,
            after,
            "parked at 200 the scythe stops accelerating"
        );
        assert!(
            before < after || after == before,
            "the clamp replaced the multiply"
        );
    }

    #[test]
    fn test_dw_3_1_style_22_the_placed_block_drains_ai_0_and_is_mined_away_at_minus_900() {
        // The placed phase (`:25681-25703`): `ai[0]--` per tick, the kill when
        // the tile under it is gone, and the `ai[0] <= -900f` KillTile of the
        // owner's own block (`:25757-25760`).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                ICE_BLOCK,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                0.0,
                0.0,
                20,
                0.0,
                -1.0,
                0.0,
            )
            .unwrap();
        let mut w = world();
        put_tile(&mut w, 20, 24, MAGICAL_ICE_BLOCK);
        assert!(pool.slots()[slot].tile_collide == false || pool.slots()[slot].ai[0] == -1.0);
        step(&mut pool, &mut w);
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[0], -2.0, "the placed phase drains one per tick");
        assert!(p.active, "the block stands while its tile does");
        assert_eq!(p.alpha, 255, "a stopped block is opaque (`:25611`)");
        // The missing tile ends the placed phase (`:25691-25694`).
        let idx = w.index(20, 24);
        w.tiles[idx].flags &= !FLAG_ACTIVE;
        step(&mut pool, &mut w);
        assert!(!pool.slots()[slot].active, "the block dies with its tile");
    }

    #[test]
    fn test_dw_3_1_style_22_the_minus_900_block_is_kill_tiled_away_on_the_owner() {
        // `:25755-25760`: the owner's block is removed at `-900`. The write is
        // the owner branch's alone (`Main.myPlayer == this.owner`).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                ICE_BLOCK,
                21.0 * TILE_PX,
                24.0 * TILE_PX,
                0.0,
                0.0,
                20,
                0.0,
                -1.0,
                0.0,
            )
            .unwrap();
        let mut w = world();
        put_tile(&mut w, 21, 24, MAGICAL_ICE_BLOCK);
        pool.slots_mut()[slot].ai[0] = -900.0;
        step(&mut pool, &mut w);
        assert!(!pool.slots()[slot].active, "the block goes with its tile");
        assert!(
            !has_tile(&w, 21, 24, MAGICAL_ICE_BLOCK),
            "`WorldGen.KillTile` removed the placed block (`:25757`)"
        );
        // -899 is one tick short of the removal.
        let mut pool2 = ProjectilePool::new();
        let slot2 = pool2
            .spawn_with_data_ai(
                ICE_BLOCK,
                22.0 * TILE_PX,
                24.0 * TILE_PX,
                0.0,
                0.0,
                20,
                0.0,
                -1.0,
                0.0,
            )
            .unwrap();
        let mut w2 = world();
        put_tile(&mut w2, 22, 24, MAGICAL_ICE_BLOCK);
        // -898 drains to -899 on the tick — one tick short of the removal.
        pool2.slots_mut()[slot2].ai[0] = -898.0;
        step(&mut pool2, &mut w2);
        assert!(
            pool2.slots()[slot2].active,
            "-899 drains but does not remove"
        );
        assert_eq!(pool2.slots()[slot2].ai[0], -899.0);
        assert!(has_tile(&w2, 22, 24, MAGICAL_ICE_BLOCK), "the block stays");
    }

    #[test]
    fn test_dw_3_1_style_22_the_flying_block_places_the_tile_and_zeroes_its_damage() {
        // `:25842-25860`: on arrival `WorldGen.PlaceTile(127)`,
        // `damage = 0`, `ai[0] = -1f`, `velocity = 0`, `alpha = 255` and the
        // position snaps to the tile corner — the switch into the placed phase.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                ICE_BLOCK,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                4.0,
                0.0,
                40,
                0.0,
                20.0,
                24.0,
            )
            .unwrap();
        let mut w = world();
        step(&mut pool, &mut w);
        let p = &pool.slots()[slot];
        assert!(
            has_tile(&w, 20, 24, MAGICAL_ICE_BLOCK),
            "the block is placed"
        );
        assert_eq!(p.damage, 0, "the placed block carries no damage (`:25852`)");
        assert_eq!(p.ai[0], -1.0, "the phase switch (`:25853`)");
        assert_eq!(p.vel, Vec2::ZERO, "the block stops (`:25854`)");
        assert_eq!(p.alpha, 255);
        assert_eq!(p.pos, Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX));
    }

    #[test]
    fn test_dw_3_1_style_22_the_flying_block_walks_a_solid_target_to_a_free_cell() {
        // `:25763-25798`: when the target tile is solid the walk prefers the
        // vertical neighbour (the flight is mostly horizontal) and the block
        // lands one tile up. `WorldGen.SolidTile` reads missing tiles as
        // solid (`WorldGen.cs:70114-70116`), so the neighbours are kept in
        // world.
        let mut pool = ProjectilePool::new();
        // 368 puts the box on row 23 only (its probe scan must not touch the
        // solid target one row below) while its centre still sits above the
        // target's middle — the C#'s `Center.Y < tile·16 + 8` gate.
        let slot = pool
            .spawn_with_data_ai(
                ICE_BLOCK,
                20.0 * TILE_PX,
                23.0 * TILE_PX,
                4.0,
                0.0,
                40,
                0.0,
                20.0,
                24.0,
            )
            .unwrap();
        let mut w = world();
        put_tile(&mut w, 20, 24, 1);
        step(&mut pool, &mut w);
        assert!(
            has_tile(&w, 20, 23, MAGICAL_ICE_BLOCK),
            "the block lands above the solid target (`:25769-25778`)"
        );
        assert!(
            has_tile(&w, 20, 24, 1),
            "the solid target itself is untouched"
        );
        assert_eq!(pool.slots()[slot].ai[0], -1.0);
    }

    #[test]
    fn test_dw_3_1_style_24_the_crystal_storm_fades_after_ai_1_130() {
        // `:25962-25968`: the CrystalStorm 94's `ai[1]++`, the ×0.985 drag and
        // the fade from `ai[1] > 130f` (`scale -= 0.05f`, kill at 0.2).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                CRYSTAL_STORM,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                1.0,
                0.0,
                20,
                0.0,
            )
            .unwrap();
        let mut w = world();
        let drag0 = pool.slots()[slot].vel.x;
        for _ in 0..130 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[1], 130.0, "one per tick");
        assert_eq!(p.scale, 1.2, "no fade while ai[1] <= 130");
        assert!(p.active, "the shard outlives its 130-tick window");
        step(&mut pool, &mut w);
        assert!(
            (pool.slots()[slot].scale - 1.15).abs() < 1e-6,
            "the 131st tick takes the first 0.05 (`:25966`)"
        );
        assert!(
            (pool.slots()[slot].vel.x - drag0 * 0.985f32.powi(131)).abs() < 1e-3,
            "the ×0.985 drag rides every tick (`:25962`)"
        );
        // Twenty fade steps from 1.2 land on the 0.2 kill.
        for _ in 0..22 {
            step(&mut pool, &mut w);
        }
        assert!(
            !pool.slots()[slot].active,
            "the fade kills at 0.2 (`:25967-25968`)"
        );
        assert_eq!(
            pool.slots()[slot].scale,
            0.2,
            "the kill pins the scale at 0.2"
        );
    }

    #[test]
    fn test_dw_3_1_style_28_the_frost_bolt_is_a_one_shot_gate() {
        // `:26558-26575`: `if (ai[1] != 0f) return;` — after the first tick the
        // arm is only its flight, so the bolt's state is the one-shot flag.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                FROST_BOLT,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                3.0,
                0.0,
                20,
                0.0,
            )
            .unwrap();
        let mut w = world();
        let x0 = pool.slots()[slot].pos.x;
        step(&mut pool, &mut w);
        assert_eq!(
            pool.slots()[slot].ai[1],
            1.0,
            "the first tick sets the gate"
        );
        assert_eq!(pool.slots()[slot].vel, Vec2::new(3.0, 0.0));
        assert_eq!(pool.slots()[slot].pos.x - x0, 3.0, "one spawn-speed step");
        for _ in 0..20 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[1], 1.0, "the gate never re-opens");
        assert_eq!(
            p.vel,
            Vec2::new(3.0, 0.0),
            "nothing else in the arm touches it"
        );
        assert!(p.active, "no gravity, no drag, no self-kill");
        assert_eq!(p.pos.x - x0, 21.0 * 3.0, "21 steps at 3 px, unchanged");
    }

    #[test]
    fn test_dw_3_1_style_50_the_bolt_spawns_the_blast_on_death() {
        // `Kill()`'s type-295 arm (`:74341-74350`): the bolt's death spawns
        // 296 at its centre with `(int)(damage * 0.75)` and the same
        // knockback. The C#'s NewProjectile takes the centre and offsets the
        // box, so the blast's 150×150 box is re-centred on the port's spawn.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                INFERNO_FORK_BOLT,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                0.0,
                0.0,
                40,
                6.0,
            )
            .unwrap();
        let (cx, cy) = {
            let p = &pool.slots()[slot];
            (p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
        };
        pool.slots_mut()[slot].time_left = 1;
        let mut w = world();
        step(&mut pool, &mut w);
        assert_eq!(
            pool.slots().iter().filter(|p| p.active).count(),
            1,
            "the bolt died and only the blast is left in the pool"
        );
        let blast = pool
            .slots()
            .iter()
            .find(|p| p.kind == INFERNO_BLAST)
            .expect("the blast spawns on the bolt's death");
        assert_eq!(blast.damage, 30, "(int)(40 * 0.75)");
        assert_eq!(blast.knock_back, 6.0);
        assert_eq!(
            (blast.w, blast.h),
            (150.0, 150.0),
            "the blast's row (`:3859-3868`)"
        );
        assert_eq!(blast.owner, 0, "the blast inherits the owner");
        assert!(
            (blast.pos.x + 75.0 - cx).abs() < 1e-3 && (blast.pos.y + 75.0 - cy).abs() < 1e-3,
            "the blast centres on the bolt's centre (`:74347`)"
        );
    }

    #[test]
    fn test_dw_3_1_style_50_the_bolt_never_double_spawns_the_blast() {
        // The plan's `penetrate == 0` edge case: a penetrate-killed bolt dies
        // inside the same tick as its fade and must still spawn once, and the
        // blast (the same aiStyle) must never chain a second one.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                INFERNO_FORK_BOLT,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                0.0,
                0.0,
                40,
                0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].penetrate = 0;
        let mut w = world();
        step(&mut pool, &mut w);
        assert_eq!(
            pool.slots()
                .iter()
                .filter(|p| p.kind == INFERNO_BLAST)
                .count(),
            1,
            "exactly one blast for a penetrate-killed bolt"
        );
        for _ in 0..10 {
            step(&mut pool, &mut w);
        }
        assert_eq!(
            pool.slots()
                .iter()
                .filter(|p| p.kind == INFERNO_BLAST)
                .count(),
            1,
            "the blast never chains a second spawn (`:74341`'s kind gate)"
        );
    }

    #[test]
    fn test_dw_3_1_style_50_the_blast_alone_runs_the_fade_counter() {
        // `:28323-28401`: the fade counter is the blast's alone (`ai[0]++`,
        // `+= 3f` for 296, kill past 540). The bolt 295 returns before it
        // (`:28333-28340`), so its life is its `timeLeft`/`penetrate`.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                INFERNO_BLAST,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                0.0,
                0.0,
                40,
                0.0,
            )
            .unwrap();
        let mut w = world();
        // `ai[0]++` then `+= 3f` = 4 per sub-step (`:28366-28374`), so the
        // 540 window closes at tick 135 (4·135 = 540) and 136 empties it.
        for _ in 0..135 {
            step(&mut pool, &mut w);
        }
        assert_eq!(pool.slots()[slot].ai[0], 540.0, "4 per sub-step");
        assert!(
            pool.slots()[slot].active,
            "540 is not past the window (`> 540f`)"
        );
        step(&mut pool, &mut w);
        assert!(
            !pool.slots()[slot].active,
            "ai[0] = 544 > 540 empties the dust count (`:28376-28380`)"
        );
    }

    #[test]
    fn test_dw_3_1_style_50_the_bolt_flies_until_its_time_left() {
        // The bolt's own arm is dust-only (`:28333-28340`), so it never runs
        // the fade counter: ten ticks later `ai[0]` is still 0.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                INFERNO_FORK_BOLT,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                2.0,
                0.0,
                40,
                0.0,
            )
            .unwrap();
        let mut w = world();
        for _ in 0..10 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[0], 0.0, "the bolt's AI arm never touches ai[0]");
        assert_eq!(p.local_ai[0], 0.0, "nor its sound gate");
        assert!(
            p.active && p.pos.x > 10.0 * TILE_PX,
            "it flies on its own lifetime"
        );
    }

    #[test]
    fn test_dw_3_1_the_styles_without_a_bounce_arm_die_on_tile_contact() {
        // 18/24/28/50 have no arm in `HandleMovement`'s chain, so a blocked
        // move takes the chain default: `position += velocity; Kill()`
        // (`:19851-19855`). The port's `kill_step` is that default.
        for kind in [DEMON_SCYTHE, CRYSTAL_STORM, FROST_BOLT] {
            let size = match kind {
                DEMON_SCYTHE => 48.0,
                CRYSTAL_STORM => 8.0,
                _ => 14.0,
            };
            let mut pool = ProjectilePool::new();
            let x = {
                let mut w = world();
                wall_ahead(&mut w, size)
            };
            pool.spawn_with_data(kind, x, 10.0 * TILE_PX, 2.0, 0.0, 20, 0.0);
            let mut w = world();
            wall_ahead(&mut w, size);
            step(&mut pool, &mut w);
            assert!(
                !pool.slots()[0].active,
                "kind {kind} dies on the blocked move"
            );
        }
    }

    // ---------------------------------------------------------------- DW-3.2
    // The aiStyle-1 riders gate ahead of the generic arm; the plain-arrow
    // behaviour and the 837 targeting rules.

    #[test]
    fn test_dw_3_2_the_rider_kinds_take_the_rider_arm_not_the_plain_arrow_arm() {
        // `AI_001`'s `flag3` skip switch (`:55947-56032`) lists 20/88/355/660:
        // the flight counter rests, so the gravity at `ai[0] >= 15f` never
        // fires for a rider while it does for the plain arrow.
        for kind in [GREEN_LASER, PURPLE_LASER, VENOM_FANG, SKY_FRACTURE] {
            let mut pool = ProjectilePool::new();
            pool.spawn_with_data(kind, 10.0 * TILE_PX, 10.0 * TILE_PX, 2.0, 0.0, 200, 0.0);
            let mut w = world();
            for _ in 0..30 {
                step(&mut pool, &mut w);
            }
            let p = &pool.slots()[0];
            assert_eq!(
                p.ai[0], 0.0,
                "kind {kind} holds ai[0] at rest (`:56039-56041`)"
            );
            assert_eq!(p.vel.y, 0.0, "kind {kind} flies straight — no gravity");
            assert!(p.active);
        }
        let mut arrow = ProjectilePool::new();
        arrow.spawn_with_data(
            WOODEN_ARROW,
            10.0 * TILE_PX,
            10.0 * TILE_PX,
            2.0,
            0.0,
            200,
            0.0,
        );
        let mut w = world();
        for _ in 0..15 {
            step(&mut arrow, &mut w);
        }
        assert!(
            (arrow.slots()[0].vel.y - 0.1).abs() < 1e-6,
            "the plain arrow keeps its gravity — the gating did not disturb it"
        );
    }
    #[test]
    fn test_dw_3_2_the_green_laser_grows_its_scale_and_knockback() {
        // `:54047-54064`: `scale += 0.0189f` (cap 3) with the owner-only
        // `knockBack *= 1.0025f` and the 25-tick `localAI[0]` counter whose
        // `damage`/`penetrate` rolls are SKIPPED (no RNG in the pool).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                GREEN_LASER,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                2.0,
                0.0,
                200,
                3.0,
            )
            .unwrap();
        // The C#'s growth rides the AI sub-step, so hold the row's
        // `extraUpdates = 2` at zero to keep one growth per pool tick — the
        // row's own value is re-asserted at the end.
        pool.slots_mut()[slot].extra_updates = 0;
        let (scale0, kb0) = {
            let p = &pool.slots()[slot];
            (p.scale, p.knock_back)
        };
        assert_eq!((scale0, kb0), (0.65, 3.0), "the row's SetDefaults slice");
        let mut w = world();
        for _ in 0..25 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert!(
            (p.scale - (scale0 + 25.0 * 0.0189)).abs() < 1e-5,
            "one 0.0189 per sub-step"
        );
        assert!(
            (p.knock_back - kb0 * 1.0025f32.powi(25)).abs() < 1e-4,
            "the owner's knockback compounds"
        );
        assert_eq!(
            p.local_ai[0], 0.0,
            "the 25-tick counter wrapped (`:54056-54057`)"
        );
        assert_eq!(
            p.damage, 200,
            "the rand bump is SKIPPED — the pool has no RNG"
        );
        assert_eq!(p.penetrate, 3, "and so is its penetrate++");
        assert_eq!(
            proj_data(GREEN_LASER).unwrap().extra_updates,
            2,
            "the row is untouched"
        );
        // The cap: the growth stops at 3.
        for _ in 0..200 {
            step(&mut pool, &mut w);
        }
        assert_eq!(pool.slots()[slot].scale, 3.0, "the 3 cap (`:54051-54053`)");
    }

    #[test]
    fn test_dw_3_2_the_purple_laser_grows_to_four_and_keeps_a_resting_flight_counter() {
        // `:54021-54045`: `scale += 0.053f` (cap 4), `knockBack *= 1.0045f`,
        // the 8-tick counter, and the `extraUpdates = (int)ai[0]` remap that
        // never fires because `flag3` holds `ai[0]` at rest for this kind.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                PURPLE_LASER,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                2.0,
                0.0,
                200,
                2.0,
            )
            .unwrap();
        pool.slots_mut()[slot].extra_updates = 0;
        let mut w = world();
        for _ in 0..8 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert!(
            (p.scale - (0.65 + 8.0 * 0.053)).abs() < 1e-5,
            "the 0.053 growth (`:54030-54033`)"
        );
        assert!((p.knock_back - 2.0 * 1.0045f32.powi(8)).abs() < 1e-4);
        assert_eq!(p.local_ai[0], 0.0, "the 8-tick counter wrapped");
        for _ in 0..200 {
            step(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert_eq!(p.scale, 4.0, "the 4 cap");
        assert_eq!(
            p.extra_updates, 0,
            "the remap never fires while ai[0] rests"
        );
        assert_eq!(p.ai[0], 0.0, "ai[0] rests, so the remap cannot fire");
        assert_eq!(
            proj_data(PURPLE_LASER).unwrap().extra_updates,
            4,
            "the row's extraUpdates rides untouched"
        );
    }

    #[test]
    fn test_dw_3_2_the_venom_fang_fades_and_the_sky_fracture_marks_its_first_tick() {
        // `:56728-56747` (VenomFang: `alpha -= 50`, the dust at 0 SKIPPED) and
        // `:55676-55687` + `:57650-57652` (SkyFracture's first-tick marker and
        // its `velocity.ToRotation() + π/4` tail).
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(
            VENOM_FANG,
            10.0 * TILE_PX,
            10.0 * TILE_PX,
            2.0,
            0.0,
            200,
            0.0,
        );
        let mut w = world();
        step(&mut pool, &mut w);
        assert_eq!(pool.slots()[0].alpha, 205, "255 - 50 (`:56732-56734`)");
        for _ in 0..4 {
            step(&mut pool, &mut w);
        }
        assert_eq!(pool.slots()[0].alpha, 5, "the fade keeps running");
        step(&mut pool, &mut w);
        assert_eq!(pool.slots()[0].alpha, 0, "clamped at 0 (`:56735-56738`)");

        let mut pool2 = ProjectilePool::new();
        let slot = pool2
            .spawn_with_data(
                SKY_FRACTURE,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                2.0,
                0.0,
                200,
                0.0,
            )
            .unwrap();
        assert_eq!(pool2.slots()[slot].alpha, 255, "the row's `alpha = 255`");
        let mut w2 = world();
        step(&mut pool2, &mut w2);
        let p = &pool2.slots()[slot];
        assert_eq!(p.alpha, 0, "the first-tick marker (`:55680-55685`)");
        assert_eq!(p.scale, 1.1, "with its 1.1 scale (`:55684`)");
        assert!(
            (p.rot - (0.0f32.atan2(2.0) + std::f32::consts::FRAC_PI_4)).abs() < 1e-6,
            "the +π/4 rotation tail (`:57650-57652`)"
        );
    }

    #[test]
    fn test_dw_3_2_the_book_of_skulls_acquires_the_nearest_target_inside_manhattan_300() {
        // `:56813-56847`: the scan keeps the running `num189` minimum seeded
        // at 300 and commits `ai[1] = num190 + 1` after the loop, so the
        // nearest chaseable NPC with a clear line wins — not the first.
        let mut w = world();
        let near = Npc::spawn_slime_sky(20.0 * TILE_PX + 32.0, 24.0 * TILE_PX, SlimeVariant::Blue);
        let far = Npc::spawn_slime_sky(20.0 * TILE_PX + 160.0, 24.0 * TILE_PX, SlimeVariant::Blue);
        let mut npcs = vec![far, near];
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BOOK_OF_SKULLS,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                2.0,
                0.0,
                480,
                0.0,
            )
            .unwrap();
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert_eq!(
            pool.slots()[slot].ai[1],
            2.0,
            "the nearer NPC is index 1 in the slice (`ai[1] = idx + 1`)"
        );
    }

    #[test]
    fn test_dw_3_2_the_book_of_skulls_keeps_the_target_inside_1000f() {
        // `:56851-56866`: a held target inside the Manhattan 1000f keeps
        // steering; the alpha also fades 25 per tick (`:56808-56812`).
        let mut w = world();
        let npc = Npc::spawn_slime_sky(20.0 * TILE_PX + 96.0, 24.0 * TILE_PX, SlimeVariant::Blue);
        let mut npcs = vec![npc];
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BOOK_OF_SKULLS,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                2.0,
                0.0,
                480,
                0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].ai[1] = 1.0;
        tick_npcs(&mut w, &mut pool, &mut npcs);
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[1], 1.0, "the target is held (`< 1000f`)");
        // The NPC's centre sits above the skull's, so the weight-20 blend
        // turns the flight upward — the steering ran.
        assert!(p.vel.y < 0.0, "the skull steers toward the target");
        assert_eq!(p.alpha, 230, "255 - 25");
    }

    #[test]
    fn test_dw_3_2_the_book_of_skulls_drops_the_target_past_1000f() {
        // `:56851-56866`: past the Manhattan 1000f the retain fails and the
        // skull flies unsteered. The C# clears `ai[1]` only in its `else`
        // (the NPC stopped being chaseable), so a merely-distant target keeps
        // its index — a remembered target re-steers if it comes back.
        let mut w = world();
        let npc = Npc::spawn_slime_sky(20.0 * TILE_PX + 1100.0, 24.0 * TILE_PX, SlimeVariant::Blue);
        let mut npcs = vec![npc];
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BOOK_OF_SKULLS,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                2.0,
                0.0,
                480,
                0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].ai[1] = 1.0;
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert_eq!(pool.slots()[slot].ai[1], 1.0, "the index is retained");
        assert_eq!(
            pool.slots()[slot].vel,
            Vec2::new(2.0, 0.0),
            "no steering past 1000f"
        );
        // Bring it back inside the window: the retained index re-steers.
        npcs[0].x = 20.0 * TILE_PX + 96.0;
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert!(
            pool.slots()[slot].vel.y < 0.0,
            "the retained target re-steers"
        );
    }

    #[test]
    fn test_dw_3_2_the_book_of_skulls_clears_the_index_of_a_dead_target() {
        // `:56866`'s `else`: a target that stops being chaseable clears
        // `ai[1]`, so the acquisition scan may take a new one.
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            20.0 * TILE_PX + 96.0,
            24.0 * TILE_PX,
            SlimeVariant::Blue,
        )];
        npcs[0].hp = 0;
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BOOK_OF_SKULLS,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                2.0,
                0.0,
                480,
                0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].ai[1] = 1.0;
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert_eq!(
            pool.slots()[slot].ai[1],
            0.0,
            "the dead target clears the index"
        );
        assert_eq!(
            pool.slots()[slot].vel,
            Vec2::new(2.0, 0.0),
            "and no steering"
        );
    }

    #[test]
    fn test_dw_3_2_the_book_of_skulls_steers_at_the_weight_20() {
        // `:56871-56881`: `velocity = (velocity * 19 + dir * spawnSpeed) / 20`
        // — the Book of Skulls' weight is 20 (the generic 8 belongs to 207).
        let mut w = world();
        let npc = Npc::spawn_slime_sky(20.0 * TILE_PX + 60.0, 24.0 * TILE_PX, SlimeVariant::Blue);
        let mut npcs = vec![npc];
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BOOK_OF_SKULLS,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                4.0,
                0.0,
                480,
                0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].ai[1] = 1.0;
        // The steering block runs inside the AI, before the flight moves the
        // box — the blend's anchor is the pre-tick centre.
        let (px, py) = {
            let p = &pool.slots()[slot];
            (p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
        };
        tick_npcs(&mut w, &mut pool, &mut npcs);
        let p = &pool.slots()[slot];
        let spawn = 4.0f32;
        let (nx, ny) = {
            let n = &npcs[0];
            (n.x + n.size().0 * 0.5, n.y + n.size().1 * 0.5)
        };
        let inv = spawn / (nx - px).hypot(ny - py);
        let expected_x = (4.0f32 * 19.0 + (nx - px) * inv) / 20.0;
        let expected_y = (0.0f32 * 19.0 + (ny - py) * inv) / 20.0;
        assert!(
            (p.vel.x - expected_x).abs() < 1e-4 && (p.vel.y - expected_y).abs() < 1e-4,
            "the weight-20 blend (`:56878-56881`)"
        );
    }

    #[test]
    fn test_dw_3_2_the_book_of_skulls_burns_time_left_inside_a_solid_tile() {
        // `:56794-56800`: standing in a solid tile the skull spends 33 ticks
        // a tick (its `tileCollide = false` flight lets it get there), plus the
        // 1 the pool's exit tail ages anyway.
        let mut w = world();
        put_tile(&mut w, 20, 24, 1);
        let mut npcs: Vec<Npc> = Vec::new();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BOOK_OF_SKULLS,
                20.0 * TILE_PX,
                24.0 * TILE_PX,
                0.0,
                0.0,
                480,
                0.0,
            )
            .unwrap();
        pool.update(&mut w, &mut npcs, None, None);
        assert_eq!(pool.slots()[slot].time_left, 480 - 1 - 33);
        assert!(pool.slots()[slot].active, "it survives the drain");
    }

    #[test]
    fn test_dw_3_2_the_existing_arrow_anchors_stay_green() {
        // DW-3.2's second half: the plain-arrow tests the phase must not
        // disturb. Re-asserted here so the whole family reads as one unit.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(
            WOODEN_ARROW,
            10.0 * TILE_PX,
            10.0 * TILE_PX,
            2.0,
            0.0,
            200,
            0.0,
        );
        let mut w = world();
        for _ in 0..14 {
            step(&mut pool, &mut w);
        }
        assert_eq!(
            pool.slots()[0].vel.y,
            0.0,
            "no gravity for the first 14 ticks"
        );
        step(&mut pool, &mut w);
        assert!((pool.slots()[0].vel.y - 0.1).abs() < 1e-6, "the 15th tick");

        let mut pool2 = ProjectilePool::new();
        let x = {
            let mut w2 = world();
            wall_ahead(&mut w2, 10.0)
        };
        pool2.spawn_with_data(WOODEN_ARROW, x, 10.0 * TILE_PX, 2.0, 0.0, 200, 0.0);
        let mut w2 = world();
        wall_ahead(&mut w2, 10.0);
        step(&mut pool2, &mut w2);
        assert!(!pool2.slots()[0].active, "the arrow breaks on tile contact");
    }

    // ---------------------------------------------------------------- DW-3.3
    // The wind tail, for the wind-eligible new arms only.

    #[test]
    fn test_dw_3_3_the_style_8_arm_takes_the_wind_tail_at_a_known_vector() {
        // `:16777-16781`: `velocity.X += windSpeedCurrent *
        // windPhysicsStrength` (0.1, `Main.cs:66842`), under the altitude
        // gate — a headwind pushing back on a +X flight.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                BALL_OF_FIRE,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                1.0,
                0.0,
                600,
                0.0,
            )
            .unwrap();
        pool.set_wind(-0.6);
        let mut w = world();
        step(&mut pool, &mut w);
        assert!(
            (pool.slots()[slot].vel.x - (1.0 - 0.6 * 0.1)).abs() < 1e-6,
            "one wind tick of -0.06 (`:16779-16780`)"
        );
        for _ in 0..4 {
            step(&mut pool, &mut w);
        }
        assert!(
            (pool.slots()[slot].vel.x - (1.0 - 5.0 * 0.6 * 0.1)).abs() < 1e-6,
            "the tail runs once per sub-step"
        );
    }

    #[test]
    fn test_dw_3_3_the_wind_tail_respects_the_altitude_and_wall_gates() {
        // Below `worldSurface * 16` the tail never runs; a walled centre tile
        // blocks it as well.
        let mut w = world();
        // The world's `world_surface` default is `height * 0.25` = 16 tiles.
        let below = (w.world_surface as f32 + 4.0) * TILE_PX;
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(BALL_OF_FIRE, 10.0 * TILE_PX, below, 1.0, 0.0, 600, 0.0)
            .unwrap();
        pool.set_wind(-0.6);
        step(&mut pool, &mut w);
        assert_eq!(pool.slots()[slot].vel.x, 1.0, "no wind below the surface");

        // The second pool's centre tile is row 10 (y = 10·16, h = 16 → the
        // centre 10·16 + 8 → row 10); a wall there blocks the tail.
        let widx = w.index(10, 10);
        w.tiles[widx].wall = 1;
        let mut pool2 = ProjectilePool::new();
        let slot2 = pool2
            .spawn_with_data(
                BALL_OF_FIRE,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                1.0,
                0.0,
                600,
                0.0,
            )
            .unwrap();
        pool2.set_wind(-0.6);
        step(&mut pool2, &mut w);
        assert_eq!(
            pool2.slots()[slot2].vel.x,
            1.0,
            "a walled centre tile blocks the tail"
        );
    }

    #[test]
    fn test_dw_3_3_the_ported_arrow_arm_stays_wind_free() {
        // The pre-existing deviation: the ported aiStyle-1 arrow arm has no
        // wind tail, and the phase must not give it one (behaviour frozen).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                WOODEN_ARROW,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                1.0,
                0.0,
                600,
                0.0,
            )
            .unwrap();
        pool.set_wind(-0.6);
        let mut w = world();
        for _ in 0..10 {
            step(&mut pool, &mut w);
        }
        assert_eq!(pool.slots()[slot].vel.x, 1.0, "the arrow ignores the wind");
        assert!(
            !uses_wind_physics(1),
            "aiStyle 1 is not wind-eligible in the port"
        );
        assert!(!uses_wind_physics(16), "nor the bomb's 16");
        assert!(uses_wind_physics(8) && uses_wind_physics(24) && uses_wind_physics(28));
        // Style 22 (the Ice Rod) is not on the C# list either (`:22110-22132`).
        assert!(!uses_wind_physics(22));
        assert!(!uses_wind_physics(18));
        assert!(!uses_wind_physics(50));
    }

    #[test]
    fn test_dw_3_3_the_wind_tail_stops_at_the_16_gate() {
        // `:16780`'s `|velocity.X| < 16f` gate is the tail's only bound: the
        // C#'s `MathHelper.Clamp` on the next line (`:16781`) discards its
        // return, so nothing clamps and the tail simply stops once the gate
        // closes. The frost bolt is the arm with no gravity and no drag, so the
        // flight stays exactly on that gate for the whole run.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                FROST_BOLT,
                10.0 * TILE_PX,
                10.0 * TILE_PX,
                15.9,
                0.0,
                600,
                0.0,
            )
            .unwrap();
        pool.set_wind(-1.0);
        let mut w = world();
        for _ in 0..319 {
            step(&mut pool, &mut w);
        }
        // 15.9 - 319 ticks of 0.1 = -16.0, the first value the gate rejects.
        assert!(
            (pool.slots()[slot].vel.x - 15.9 + 319.0 * 0.1).abs() < 2e-3,
            "the tail ran every tick under the gate"
        );
        let stuck = pool.slots()[slot].vel.x;
        assert!(stuck.abs() >= 16.0 - 2e-3, "the gate met: |vx| = {stuck}");
        for _ in 0..10 {
            step(&mut pool, &mut w);
        }
        assert_eq!(
            pool.slots()[slot].vel.x,
            stuck,
            "no clamp and no further wind past the gate"
        );
    }

    // ---- DW-4a: the gem-staff family (aiStyle 29) --------------------------

    /// `new GemStaffFeatures(aiState)` (`Projectile.cs:161-164`): the C#
    /// implicit float → `FloatIntUnion` cast reinterprets the float's bits,
    /// so every bit survives the `ai[0]` round-trip the shoot side packs.
    #[test]
    fn test_dw_4a_the_gem_staff_features_round_trip_through_ai0() {
        for bit in 0..10u32 {
            let mut features = GemStaffFeatures::from_ai(0.0);
            features.set_bit(1 << bit, true);
            let back = GemStaffFeatures::from_ai(features.ai_state());
            assert_eq!(back.bits, features.bits, "bit {bit} round-trips");
        }
        let mut all = GemStaffFeatures::from_ai(0.0);
        for bit in 0..10u32 {
            all.set_bit(1 << bit, true);
        }
        assert_eq!(all.bits, (1 << 10) - 1, "all ten bits set");
        assert!(
            all.homing() && all.swirl_twins() && all.all_gems2(),
            "all ten bits read back (bits={})",
            all.bits
        );
    }

    /// A gem bolt spawned with a feature-packed `ai[0]` — the shoot path's
    /// seam (`spawn_with_data_ai`'s `ai0`, Phase 5).
    fn gem_bolt(
        features: &GemStaffFeatures,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
    ) -> (ProjectilePool, usize) {
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                GEM_BOLT_RUBY,
                x,
                y,
                vx,
                vy,
                20,
                0.0,
                features.ai_state(),
                0.0,
            )
            .unwrap();
        (pool, slot)
    }

    fn only_homing() -> GemStaffFeatures {
        let mut features = GemStaffFeatures::from_ai(0.0);
        features.set_bit(GemStaffFeatures::HOMING, true);
        features
    }

    #[test]
    fn test_dw_4a_homing_steers_toward_the_target_and_the_plain_bolt_does_not() {
        // `:37876-37923`: the homing block bends the flight toward the
        // target; without the Homing bit the bolt keeps its line.
        let features = only_homing();
        let (mut pool, slot) = gem_bolt(&features, 20.0 * TILE_PX, 24.0 * TILE_PX, 2.0, 0.0);
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            20.0 * TILE_PX + 96.0,
            24.0 * TILE_PX + 48.0,
            SlimeVariant::Blue,
        )];
        tick_npcs(&mut w, &mut pool, &mut npcs);
        let steered = pool.slots()[slot].vel;
        assert!(
            steered.y > 0.0,
            "the turn bends the flight toward the target (vy={})",
            steered.y
        );
        let (mut pool, slot) = gem_bolt(
            &GemStaffFeatures::from_ai(0.0),
            20.0 * TILE_PX,
            24.0 * TILE_PX,
            2.0,
            0.0,
        );
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            20.0 * TILE_PX + 96.0,
            24.0 * TILE_PX + 48.0,
            SlimeVariant::Blue,
        )];
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert_eq!(
            pool.slots()[slot].vel,
            Vec2::new(2.0, 0.0),
            "no Homing bit, no steering (`:37876`)"
        );
    }

    #[test]
    fn test_dw_4a_homing_accelerates_to_the_steer_speed_cap() {
        // `:37896-37903`: the steer accel runs while under the steer speed —
        // 8f, doubled for FastThenSlow — so the two converge to different
        // speeds at a fixed 480f target (inside the 500f radius, past the
        // 150f close-range pull).
        let mut fast = only_homing();
        fast.set_bit(GemStaffFeatures::FAST_THEN_SLOW, true);
        for (features, label) in [(only_homing(), "8f"), (fast, "16f")] {
            let (mut pool, slot) = gem_bolt(&features, 20.0 * TILE_PX, 24.0 * TILE_PX, 2.0, 0.0);
            let mut w = world();
            let mut npcs = vec![Npc::spawn_slime_sky(
                20.0 * TILE_PX + 480.0,
                24.0 * TILE_PX,
                SlimeVariant::Blue,
            )];
            for _ in 0..30 {
                tick_npcs(&mut w, &mut pool, &mut npcs);
            }
            let speed = pool.slots()[slot].vel.length();
            if label == "8f" {
                assert!(
                    (7.5..9.0).contains(&speed),
                    "the 8f steer speed caps the accel (|v|={speed})"
                );
            } else {
                assert!(
                    speed > 10.0,
                    "the 16f steer speed keeps accelerating past the 8f cap (|v|={speed})"
                );
            }
        }
    }

    #[test]
    fn test_dw_4a_swirl_twins_offsets_the_two_bolts_in_opposite_phases() {
        // `:37924-37935`: the sinusoid pair — the twin rides `ai[1] = 1f`,
        // a PI phase shift, so the two bolts swing opposite ways. The
        // primary's first tick: sin(0) vs sin(2PI * 0.05) = 0.309, offset
        // 0.309 * 6 = 1.854 perpendicular to the (2, 0) flight.
        let mut w = world();
        let mut features = GemStaffFeatures::from_ai(0.0);
        features.set_bit(GemStaffFeatures::SWIRL_TWINS, true);
        for (twin, sign) in [(0.0, 1.0), (1.0, -1.0)] {
            let mut pool = ProjectilePool::new();
            let slot = pool
                .spawn_with_data_ai(
                    GEM_BOLT_RUBY,
                    20.0 * TILE_PX,
                    24.0 * TILE_PX,
                    2.0,
                    0.0,
                    20,
                    0.0,
                    features.ai_state(),
                    twin,
                )
                .unwrap();
            step(&mut pool, &mut w);
            let p = &pool.slots()[slot];
            assert!(
                (p.pos.y - (24.0 * TILE_PX + 1.854 * sign)).abs() < 1e-2,
                "the twin={twin} swing sign {sign} lands at {}",
                p.pos.y - 24.0 * TILE_PX
            );
            assert!(
                (p.pos.x - (20.0 * TILE_PX + 2.0)).abs() < 1e-2,
                "the flight keeps its direction"
            );
        }
    }

    #[test]
    fn test_dw_4a_fast_then_slow_shrinks_the_velocity_and_kills_at_the_0_1_scale() {
        // `:37936-37952`: the ×0.97 drag until the flight dies, then the
        // scale drops 0.005 per tick and the bolt dies below 0.1.
        let mut features = only_homing();
        features.set_bit(GemStaffFeatures::FAST_THEN_SLOW, true);
        let (mut pool, slot) = gem_bolt(&features, 20.0 * TILE_PX, 24.0 * TILE_PX, 4.0, 0.0);
        let mut w = world();
        for _ in 0..125 {
            step(&mut pool, &mut w);
        }
        let (speed, scale) = (pool.slots()[slot].vel.length(), pool.slots()[slot].scale);
        assert!(speed < 0.1, "the drag ends the flight (|v|={speed})");
        assert!(scale <= 1.0, "the scale holds until the flight dies");
        for _ in 0..200 {
            step(&mut pool, &mut w);
        }
        assert!(
            !pool.slots()[slot].active,
            "the bolt dies at the 0.1 scale (`:37948-37951`)"
        );
    }

    #[test]
    fn test_dw_4a_fast_then_slow_survives_a_solid_tile() {
        // `:18040-18046`: the bypass — the resolved velocity ×0.99 and the
        // old direction pinned at 0.0001, so the bolt never dies on tiles.
        let mut features = GemStaffFeatures::from_ai(0.0);
        features.set_bit(GemStaffFeatures::FAST_THEN_SLOW, true);
        let mut w = world();
        let x = wall_ahead(&mut w, 10.0);
        let (mut pool, slot) = gem_bolt(&features, x, 10.0 * TILE_PX, 2.0, 0.0);
        step(&mut pool, &mut w);
        let p = &pool.slots()[slot];
        assert!(p.active, "the FastThenSlow bolt survives the tile");
        assert!(p.vel.length() < 1e-3, "the old direction pinned at 0.0001");
    }

    #[test]
    fn test_dw_4a_can_bounce_reverses_the_flight_and_spends_penetration() {
        // `:18003-18030`: each blocked move reverses that axis at ×0.8 and
        // spends one penetration; the budget (1 + the finalize's 2) runs
        // out on the third bounce.
        let mut features = GemStaffFeatures::from_ai(0.0);
        features.set_bit(GemStaffFeatures::CAN_BOUNCE, true);
        let mut w = world();
        put_tile(&mut w, 9, 10, 1);
        let x = wall_ahead(&mut w, 10.0);
        let (mut pool, slot) = gem_bolt(&features, x, 10.0 * TILE_PX, 2.0, 0.0);
        let budget = pool.slots()[slot].penetrate;
        assert_eq!(budget, 3, "the finalize hires CanBounce's +2 (`:22077`)");
        // First bounce off tile 11: -2 × 0.8.
        step(&mut pool, &mut w);
        let p = &pool.slots()[slot];
        assert!(p.active, "survives inside the budget");
        assert!((p.vel.x + 1.6).abs() < 1e-6, "the axis reverses at 0.8");
        assert_eq!(p.penetrate, 2, "the bounce spends one");
        // Walk it back into the same wall: the second bounce.
        pool.slots_mut()[slot].pos.x = x;
        pool.slots_mut()[slot].vel.x = 2.0;
        step(&mut pool, &mut w);
        let p = &pool.slots()[slot];
        assert!(p.active, "survives the second bounce");
        assert_eq!(p.penetrate, 1, "the budget drains");
        // Third: the budget is spent.
        pool.slots_mut()[slot].pos.x = x;
        pool.slots_mut()[slot].vel.x = 2.0;
        step(&mut pool, &mut w);
        assert!(
            !pool.slots()[slot].active,
            "the spent budget kills on the bounce (`:18052-18055`)"
        );
    }

    #[test]
    fn test_dw_4a_the_finalize_bonuses_ride_the_spawn() {
        // `FinalizeProjectile_GemStaves` (`:22063-22099`): FastThenSlow
        // +1 penetration and ×2 life, CanBounce +2, BiggerHitbox +1, the
        // AllGems pair forces the 2/4 pair with timeLeft 90.
        for (bit, time_left, penetrate) in [
            (GemStaffFeatures::HOMING, 300, 1),
            (GemStaffFeatures::FAST_THEN_SLOW, 600, 2),
            (GemStaffFeatures::CAN_BOUNCE, 300, 3),
            (GemStaffFeatures::BIGGER_HITBOX, 300, 2),
            (GemStaffFeatures::ALL_GEMS2, 90, 3),
        ] {
            let mut features = GemStaffFeatures::from_ai(0.0);
            features.set_bit(bit, true);
            let (pool, slot) = gem_bolt(&features, 20.0 * TILE_PX, 24.0 * TILE_PX, 2.0, 0.0);
            let p = &pool.slots()[slot];
            assert_eq!(p.time_left, time_left, "bit {bit} timeLeft");
            assert_eq!(p.penetrate, penetrate, "bit {bit} penetration");
            assert_eq!(p.max_penetrate, penetrate, "bit {bit} maxPenetrate");
        }
    }

    #[test]
    fn test_dw_4a_the_aoe_explosion_damages_through_the_resized_box() {
        // `:75425-75484`: the death burst rides the dying slot — the 160×160
        // box around the old centre, damage ×0.35, the immunity reset
        // keeping the already-hit centre. An NPC inside the box but off the
        // centre takes the burst; one outside it takes nothing.
        let mut features = only_homing();
        features.set_bit(GemStaffFeatures::AOE_EXPLOSION, true);
        let (mut pool, slot) = gem_bolt(&features, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0);
        let mut w = world();
        let mut npcs = vec![
            Npc::spawn_slime_sky(20.0 * TILE_PX + 40.0, 24.0 * TILE_PX, SlimeVariant::Blue),
            Npc::spawn_slime_sky(20.0 * TILE_PX + 300.0, 24.0 * TILE_PX, SlimeVariant::Blue),
        ];
        let (near_before, far_before) = (npcs[0].hp, npcs[1].hp);
        pool.slots_mut()[slot].time_left = 1;
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert!(!pool.slots()[slot].active, "the bolt dies into the burst");
        assert!(
            npcs[1].hp == far_before,
            "outside the 160 box, no burst damage"
        );
        assert!(
            npcs[0].hp < near_before,
            "inside the box the burst lands ({} → {})",
            near_before,
            npcs[0].hp
        );
    }

    // ---- DW-4b: the aimed spells, the magnet sphere and the spirit flame --

    #[test]
    fn test_dw_4b_the_spell_holds_the_aim_while_the_channel_is_open() {
        // `:43239-43249`: the channel keeps `ai[0..1]` on the clamped
        // `MouseWorld` while the held item shoots this kind.
        let mut p = crate::player::Player::spawn(&world());
        p.channel = true;
        p.channel_item = MAGIC_MISSILE;
        p.channel_target = Vec2::new(500.0, 400.0);
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                MAGIC_MISSILE, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 30, 0.0, 320.0, 384.0,
            )
            .unwrap();
        let mut w = world();
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(
            &mut w,
            &mut npcs,
            Some((10.0 * TILE_PX, 10.0 * TILE_PX)),
            Some(&mut p),
        );
        let spell = &pool.slots()[slot];
        assert_eq!(spell.ai[0], 500.0, "the aim X rides the channel target");
        assert_eq!(spell.ai[1], 400.0, "the aim Y rides the channel target");
    }

    #[test]
    fn test_dw_4b_the_spell_releases_finds_a_target_and_floors_the_life() {
        // `:43250-43271`: the release drops the aim to −1, hunts a target
        // within line of sight, and the aiming floor holds `timeLeft` past
        // 60; a miss flies out at the cruise speed 32.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                MAGIC_MISSILE, 10.0 * TILE_PX, 10.0 * TILE_PX, 4.0, 0.0, 30, 0.0, 320.0, 384.0,
            )
            .unwrap();
        let mut p = crate::player::Player::spawn(&world());
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            10.0 * TILE_PX + 64.0,
            10.0 * TILE_PX,
            SlimeVariant::Blue,
        )];
        pool.update(
            &mut w,
            &mut npcs,
            Some((10.0 * TILE_PX, 10.0 * TILE_PX)),
            Some(&mut p),
        );
        let spell = &pool.slots()[slot];
        assert_eq!(spell.ai[0], -1.0, "the release clears the aim");
        assert_eq!(spell.ai[1], 0.0, "the found target rides ai[1] as the index + 1");
        assert!(spell.time_left >= 60, "the aiming floor holds the life");
        // A miss: no target, the fly-out at the cruise speed.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                MAGIC_MISSILE, 10.0 * TILE_PX, 10.0 * TILE_PX, 4.0, 0.0, 30, 0.0, 320.0, 384.0,
            )
            .unwrap();
        let mut p = crate::player::Player::spawn(&world());
        let mut w = world();
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(
            &mut w,
            &mut npcs,
            Some((10.0 * TILE_PX, 10.0 * TILE_PX)),
            Some(&mut p),
        );
        let spell = &pool.slots()[slot];
        assert_eq!(spell.ai[1], -1.0, "a miss leaves the slot indexless");
        assert!(
            (spell.vel.length() - 32.0).abs() < 1e-3,
            "the fly-out rides the cruise speed (|v|={})",
            spell.vel.length()
        );
    }

    #[test]
    fn test_dw_4b_the_lost_spell_caps_the_life_at_300() {
        // `:43341-43350`: a released spell with no target walks its flight
        // to 32 and caps `timeLeft` at 300.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                MAGIC_MISSILE, 10.0 * TILE_PX, 10.0 * TILE_PX, 3.0, 4.0, 30, 0.0, -1.0, -1.0,
            )
            .unwrap();
        pool.slots_mut()[slot].time_left = 900;
        let mut w = world();
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(&mut w, &mut npcs, None, None);
        let spell = &pool.slots()[slot];
        assert!(
            spell.time_left <= 300,
            "the lost spell caps at 300 (got {})",
            spell.time_left
        );
        assert!(
            spell.vel.length() > 5.0,
            "the flight walks toward 32 (|v|={})",
            spell.vel.length()
        );
    }

    #[test]
    fn test_dw_4b_the_magnet_sphere_dedupes_the_elder_ball() {
        // `:41428-41438`: the elder ball forces the younger same-type
        // same-owner ball's life to 30 (the tick's own decrement aside).
        let mut pool = ProjectilePool::new();
        let elder = pool
            .spawn_with_data(MAGNET_SPHERE_BALL, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 40, 0.0)
            .unwrap();
        let younger = pool
            .spawn_with_data(MAGNET_SPHERE_BALL, 12.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 40, 0.0)
            .unwrap();
        pool.slots_mut()[elder].time_left = 200;
        pool.slots_mut()[younger].time_left = 100;
        let mut w = world();
        step(&mut pool, &mut w);
        assert!(
            (pool.slots()[younger].time_left - 30).abs() <= 1,
            "the younger drops to 30 (got {})",
            pool.slots()[younger].time_left
        );
        assert!(pool.slots()[elder].time_left > 30, "the elder keeps its life");
    }

    #[test]
    fn test_dw_4b_the_magnet_sphere_fires_its_bolt_past_the_cooldown() {
        // `:41446-41499`: the cooldown 9 with a candidate inside the
        // Manhattan 300f spawns the bolt 255 — and no candidate arms
        // nothing.
        for (with_npc, label) in [(true, "with a candidate"), (false, "without")] {
            let mut pool = ProjectilePool::new();
            pool.spawn_with_data(MAGNET_SPHERE_BALL, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0, 40, 0.0)
                .unwrap();
            let mut w = world();
            let mut npcs = if with_npc {
                vec![Npc::spawn_slime_sky(
                    20.0 * TILE_PX + 48.0,
                    24.0 * TILE_PX,
                    SlimeVariant::Blue,
                )]
            } else {
                Vec::new()
            };
            for _ in 0..12 {
                tick_npcs(&mut w, &mut pool, &mut npcs);
            }
            // The spawned bolt leaves the world in its own spawn tick (101
            // sub-steps at speed 6), so the spawn itself is counted.
            let bolts = pool
                .slots()
                .iter()
                .filter(|p| p.kind == MAGNET_SPHERE_BOLT)
                .count();
            if with_npc {
                assert!(bolts >= 1, "the cooldown fires the bolt ({label})");
            } else {
                assert_eq!(bolts, 0, "no candidate, no bolt ({label})");
            }
        }
    }

    #[test]
    fn test_dw_4b_the_magnet_bolt_snapshots_at_the_delay_and_fades_out_of_world() {
        // `:28204-28242` + `AI_048_GetStartPositionSettingDelay` (`:53986-
        // 54000`): the 255's delay is 4 ticks, and the out-of-world kill
        // fires on the 5-tile margin.
        let mut pool = ProjectilePool::new();
        // The row's extraUpdates 100 runs the arm 101 sub-steps per tick,
        // so one tick probes the delay-4 snapshot: the 5th sub-step sees
        // the box at 100 + 4 * 0.6, centre 106.4.
        let slot = pool
            .spawn_with_data(MAGNET_SPHERE_BOLT, 100.0, 100.0, 0.6, 0.0, 20, 0.0)
            .unwrap();
        let mut w = world();
        step(&mut pool, &mut w);
        let bolt = &pool.slots()[slot];
        assert_eq!(bolt.local_ai[0], 101.0, "one sub-step per AI run");
        assert!(
            (bolt.local_ai[1] - 106.4).abs() < 1e-3,
            "the snapshot rides the 5th sub-step's centre (got {})",
            bolt.local_ai[1]
        );
        assert!((bolt.local_ai[2] - 104.0).abs() < 1e-3, "the y snapshot");
        assert_eq!(bolt.alpha, 255, "the alpha write (`:28216`)");
        // Past the margin: the bolt dies.
        pool.slots_mut()[slot].pos.x = (w.width - 3) as f32 * TILE_PX;
        pool.slots_mut()[slot].pos.y = 100.0;
        step(&mut pool, &mut w);
        assert!(
            !pool.slots()[slot].active,
            "outside the 5-tile margin the bolt dies"
        );
    }

    #[test]
    fn test_dw_4b_the_spirit_flame_retargets_inside_400f_and_caps_the_life() {
        // `:36202-36308`: the 659's parameters — the 400f retarget into
        // `ai[0]` with the life extended to 420, the −1 death at 5 and the
        // −2 wait cap at 240, and the tracking blend toward the target.
        let mut pool = ProjectilePool::new();
        // The shoot spawns the flame sitting still with `ai0 = -2`
        // (`Player.cs:48842-48851`).
        let slot = pool
            .spawn_with_data_ai(
                SPIRIT_FLAME, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0, 80, 0.0, -2.0, 0.0,
            )
            .unwrap();
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            20.0 * TILE_PX + 200.0,
            24.0 * TILE_PX,
            SlimeVariant::Blue,
        )];
        tick_npcs(&mut w, &mut pool, &mut npcs);
        let flame = &pool.slots()[slot];
        assert_eq!(flame.ai[0], 0.0, "the retarget stores the NPC index");
        assert!(
            flame.time_left >= 419,
            "a found target extends the life to 420 (got {})",
            flame.time_left
        );
        assert!(flame.vel.x > 0.0, "the tracking blend pulls toward the target");
        // The no-target deaths.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                SPIRIT_FLAME, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0, 80, 0.0, -1.0, 0.0,
            )
            .unwrap();
        let mut w = world();
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(&mut w, &mut npcs, None, None);
        assert!(
            pool.slots()[slot].time_left <= 5,
            "the aimless flame dies at 5"
        );
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                SPIRIT_FLAME, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0, 80, 0.0, -2.0, 0.0,
            )
            .unwrap();
        pool.slots_mut()[slot].time_left = 900;
        let mut w = world();
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(&mut w, &mut npcs, None, None);
        assert!(
            pool.slots()[slot].time_left <= 240,
            "the waiting flame caps at 240"
        );
    }

    // ---- Batch-review fixes (phase 4b) --------------------------------------

    #[test]
    fn test_fix_4b_the_released_spell_blends_at_most_0p2() {
        // `:43292-43295`: the aim blend wraps the ramp in
        // `MathHelper.Lerp(0f, 0.2f, ...)`, so the velocity never snaps the
        // whole way to the approach vector. A released spell with a target
        // ahead keeps most of its own flight.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                MAGIC_MISSILE, 10.0 * TILE_PX, 10.0 * TILE_PX, 4.0, 0.0, 30, 0.0, -1.0, 0.0,
            )
            .unwrap();
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            10.0 * TILE_PX + 64.0,
            10.0 * TILE_PX,
            SlimeVariant::Blue,
        )];
        pool.update(&mut w, &mut npcs, None, None);
        let spell = &pool.slots()[slot];
        // The blend cap: the flight stays in the approach direction but only
        // 20% toward it, so |v| is at most 32 * 1.2 = 38.4 and the direction
        // keeps a large share of the original +x.
        assert!(
            spell.vel.x > 3.0,
            "the 0.2 blend keeps the flight (vx={})",
            spell.vel.x
        );
        assert!(
            spell.vel.length() < 38.5,
            "the blend caps the approach (|v|={})",
            spell.vel.length()
        );
    }

    #[test]
    fn test_fix_4b_the_target_hunt_stops_at_the_800f_radius() {
        // The parameterless overload's 800f default (`:45628`): a target
        // past it is not acquired.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                MAGIC_MISSILE, 10.0 * TILE_PX, 10.0 * TILE_PX, 4.0, 0.0, 30, 0.0, 320.0, 384.0,
            )
            .unwrap();
        let mut p = crate::player::Player::spawn(&world());
        let mut w = world();
        let mut npcs = vec![Npc::spawn_slime_sky(
            10.0 * TILE_PX + 1000.0,
            10.0 * TILE_PX,
            SlimeVariant::Blue,
        )];
        pool.update(
            &mut w,
            &mut npcs,
            Some((10.0 * TILE_PX, 10.0 * TILE_PX)),
            Some(&mut p),
        );
        let spell = &pool.slots()[slot];
        assert_eq!(spell.ai[0], -1.0, "the release runs");
        assert_eq!(spell.ai[1], -1.0, "no target past the 800f radius");
    }

    #[test]
    fn test_fix_4b_the_spirit_flame_keeps_the_closest_candidate() {
        // `:36218-36225`: the in-loop assignment keeps the held-index
        // comparison live, so the closest candidate wins, not the last.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                SPIRIT_FLAME, 20.0 * TILE_PX, 24.0 * TILE_PX, 0.0, 0.0, 80, 0.0, -2.0, 0.0,
            )
            .unwrap();
        let mut w = world();
        let mut npcs = vec![
            Npc::spawn_slime_sky(20.0 * TILE_PX + 200.0, 24.0 * TILE_PX, SlimeVariant::Blue),
            Npc::spawn_slime_sky(20.0 * TILE_PX + 64.0, 24.0 * TILE_PX, SlimeVariant::Blue),
        ];
        tick_npcs(&mut w, &mut pool, &mut npcs);
        assert_eq!(
            pool.slots()[slot].ai[0], 1.0,
            "the nearer candidate wins regardless of the scan order"
        );
    }
}
