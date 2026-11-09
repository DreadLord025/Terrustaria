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
//!   (`:466`), `immunityIdentity = type` (`:782`)
//! - the whip family (M4-PROJ-7): `DefaultToWhip()` `:10704-10718` (aiStyle
//!   165), `AI_165_Whip()` `:47667-48270` (the swing machine — the curve and
//!   the sweep live in [`tr_core::whip`]), `Projectile.Colliding`'s whip arm
//!   `:15716-15737` (the swept-span multi-hit) and the on-hit arms
//!   (`StatusNPC`'s `ApplyWhipDebuffs` `:11727-11750`, the per-type switch
//!   `:13882-13956`); the tag stack itself is `tag_effects.rs`.
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
use tr_core::{World, TILE_PX};
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::npc::Npc;
use tr_core::loot::DropRng;

/// The Moon Lord whip's extra-arm spawn request — `AI_165_Whip`'s
/// `NewProjectile(this.type, ..., ai0 = 0, ai1 = Main.rand.NextFloatDirection(),
/// ai2 = 10 + this.ai[2])` (`Projectile.cs:47710`). The pool spawns it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WhipArmSpawn {
    pub ai1: f32,
    pub ai2: f32,
    /// The aim direction (`player.DirectionTo(Main.MouseScreen +
    /// Main.screenPosition).SafeNormalize(velocity) * velocity.Length()`,
    /// `:47709`) — the live aim when the step carries one.
    pub dir: Vec2,
}

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
/// `ProjectileID.PhantasmalEye` (452) — the Moon Lord hand's eye
/// volley (`Projectile.cs:5240-5249`: 14x14, aiStyle 82, hostile,
/// timeLeft 600).
/// `ProjectileID.DarkDividingDart`-family hostile dart (581) — the
/// VortexHunter's shot (`Projectile.cs:6306-6315`: 10x10, aiStyle 1,
/// hostile, penetrate -1).
pub const VORTEX_DART: u16 = 581;
pub const PHANTASMAL_EYE: u16 = 452;
/// `ProjectileID.PhantasmalSphere` (454) — the flood orb
/// (`Projectile.cs:5259-5269`: 40x40, aiStyle 83, hostile,
/// tileCollide false, timeLeft 600).
pub const PHANTASMAL_SPHERE: u16 = 454;
/// `ProjectileID.PhantasmalDeathray` (455) — the head's sweeping
/// deathray (`Projectile.cs:5270-5280`: 36x36, aiStyle 84, hostile,
/// tileCollide false, timeLeft 600).
pub const PHANTASMAL_DEATHRAY: u16 = 455;
/// `ProjectileID.MoonLeech` (456) — the head's leech seeker
/// (`Projectile.cs:5281-5291`: 16x16, aiStyle 85, hostile,
/// tileCollide false, timeLeft 600).
pub const MOON_LEECH: u16 = 456;
/// `ProjectileID.PhantasmalBolt` (462) — the retargeting lasers
/// (`Projectile.cs:5329-5339`: 8x8, aiStyle 1, hostile,
/// extraUpdates 3, tileCollide false, ignoreWater).
pub const PHANTASMAL_BOLT: u16 = 462;
/// `ProjectileID.BlowupSmokeMoonlord` (622) — the core's death-drama
/// smoke (`Projectile.cs:6691-6698`: 10x10, aiStyle 1, extraUpdates
/// 2, timeLeft 3600).
pub const BLOWUP_SMOKE_MOONLORD: u16 = 622;
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
// --- the whip family (M4-PROJ-7): `ProjectileID.Sets.IsAWhip`
// (`ProjectileID.cs:193`) — the 18 kinds, all `DefaultToWhip()`'s slice
// (`Projectile.cs:10704-10718`: 18x18, aiStyle 165, penetrate -1,
// tileCollide false, usesLocalNPCImmunity, localNPCHitCooldown -1).
/// `ProjectileID.BlandWhip` (841).
pub const BLAND_WHIP: u16 = 841;
/// `ProjectileID.SwordWhip` (847) — `extraUpdates = 2` (`:8157`).
pub const SWORD_WHIP: u16 = 847;
/// `ProjectileID.MaceWhip` (848).
pub const MACE_WHIP: u16 = 848;
/// `ProjectileID.ScytheWhip` (849) — `extraUpdates = 2` (`:8166`).
pub const SCYTHE_WHIP: u16 = 849;
/// `ProjectileID.CoolWhip` (912).
pub const COOL_WHIP: u16 = 912;
/// `ProjectileID.FireWhip` (913).
pub const FIRE_WHIP: u16 = 913;
/// `ProjectileID.ThornWhip` (914).
pub const THORN_WHIP: u16 = 914;
/// `ProjectileID.RainbowWhip` (915).
pub const RAINBOW_WHIP: u16 = 915;
/// `ProjectileID.BoneWhip` (952).
pub const BONE_WHIP: u16 = 952;
/// `ProjectileID.CobWhip` (1028) — the progress remap (`:48286-48289`).
pub const COB_WHIP: u16 = 1028;
/// `ProjectileID.CorruptWhip` (1029).
pub const CORRUPT_WHIP: u16 = 1029;
/// `ProjectileID.CrimsonWhip` (1030).
pub const CRIMSON_WHIP: u16 = 1030;
/// `ProjectileID.MeteorWhip` (1031).
pub const METEOR_WHIP: u16 = 1031;
/// `ProjectileID.FlowerWhip` (1032).
pub const FLOWER_WHIP: u16 = 1032;
/// `ProjectileID.EelWhip` (1033).
pub const EEL_WHIP: u16 = 1033;
/// `ProjectileID.ConstellationWhip` (1034).
pub const CONSTELLATION_WHIP: u16 = 1034;
/// `ProjectileID.MoonLordWhip` (1035) — the extra-arm spawner
/// (`:47703-47712`).
pub const MOON_LORD_WHIP: u16 = 1035;
/// `ProjectileID.SlimeWhip` (1104).
pub const SLIME_WHIP: u16 = 1104;
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
/// the box, the AI style, the penetrate/timeLeft overrides and the flags the
/// flight behaves on. The damage stays the spawner's input.
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
}

/// The ported families' `SetDefaults` slice. The boulder (186, aiStyle 37)
/// and the flame jet (187, aiStyle 38) traps stay passthroughs — their AI
/// families are M4-PROJ follow-ups.
pub fn proj_data(kind: u16) -> Option<ProjData> {
    let (w, h, ai_style, penetrate, time_left, extra_updates, tile_collide, arrow, trap) =
        match kind {
            // `Projectile.cs:807-815` / `:817-827` (the flame arrow shares the
            // box; `timeLeft = 1200` on both).
            WOODEN_ARROW | FLAME_ARROW => (10.0, 10.0, 1, 1, 1200, 0, true, true, false),
            // `:817-827`'s hostile twin — the same 10x10 box, the aiStyle-1
            // flight, `timeLeft = 1200`.
            WOODEN_ARROW_HOSTILE => (10.0, 10.0, 1, 1, 1200, 0, true, true, false),
            // `Projectile.cs:1871-1882` — the laser: 4x4, the aiStyle-1
            // flight, `penetrate = 3`, `extraUpdates = 2`, `timeLeft = 2700`
            // (the AI_037 arm overrides to 300 post-spawn).
            DESTROYER_LASER => (4.0, 4.0, 1, 3, 2700, 2, true, false, false),
            // `Projectile.cs:3417-3426` — the Fireball: 16x16, aiStyle 8,
            // `timeLeft = 300`, `penetrate = -1`, hostile, alpha 100.
            GOLEM_FIREBALL => (16.0, 16.0, 8, -1, 300, 0, true, false, false),
            // `Projectile.cs:3427-3440` — the Eye Beam: 8x8, aiStyle 1,
            // `extraUpdates = 1`, `tileCollide = false`, `penetrate = -1`,
            // hostile, `timeLeft = 300` (the call sites' override).
            GOLEM_EYE_BEAM => (8.0, 8.0, 1, -1, 300, 1, false, false, false),
            // `Projectile.cs:1632-1645` — the EyeLaser: 4x4, aiStyle 1,
            // penetrate 3, extraUpdates 2, timeLeft 600, hostile.
            EYE_LASER => (4.0, 4.0, 1, 3, 600, 2, true, false, false),
            // `Projectile.cs:1803-1815` — the CursedFlameHostile: 16x16,
            // aiStyle 8, penetrate -1, hostile.
            CURSED_FLAME => (16.0, 16.0, 8, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // `Projectile.cs:1885-1895` — the EyeFire: 6x6, aiStyle 23,
            // penetrate -1, extraUpdates 3, hostile.
            EYE_FIRE => (6.0, 6.0, 23, -1, TIME_LEFT_DEFAULT, 3, true, false, false),
            // `Projectile.cs:1602-1610` — the Scrambler's bolt: 10x10,
            // aiStyle 1, hostile, ignoreWater.
            BRAIN_SCRAMBLER => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // `Projectile.cs:1639-1648` — the Ray Gunner's laser: 8x8,
            // aiStyle 1, extraUpdates 3.
            RAY_GUNNER => (8.0, 8.0, 1, 1, TIME_LEFT_DEFAULT, 3, true, false, false),
            // `Projectile.cs:1804-1818` — the deathray: 30x30, aiStyle 79
            // (the port's boomerang family's homing stand-in), penetrate
            // -1, tileCollide false, timeLeft 240.
            SAUCER_DEATHRAY => (30.0, 30.0, 79, -1, 240, 0, false, false, false),
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
            ),
            // `Projectile.cs:1826-1837` — the laser: 5x5, aiStyle 1,
            // extraUpdates 1, timeLeft 600.
            SAUCER_LASER => (5.0, 5.0, 1, 1, 600, 1, true, false, false),
            // `Projectile.cs:1839-1848` — the scrap: 14x14, aiStyle 14,
            // penetrate -1, timeLeft 360.
            SAUCER_SCRAP => (14.0, 14.0, 14, -1, 360, 0, true, false, false),
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
            ),
            // `Projectile.cs:9307-9317` — the shadow hand: 40x40, aiStyle
            // 187, penetrate -1, tileCollide false, timeLeft 300.
            SHADOW_HAND => (40.0, 40.0, 187, -1, 300, 0, false, false, false),
            // `Projectile.cs:1896-1903` — the bomb: 22x22, aiStyle 16,
            // penetrate -1, hostile.
            BOMB_SKELETRON => (22.0, 22.0, 16, -1, 180, 0, true, false, false),
            // `:950-961`: 4×4, `extraUpdates = 1`, `timeLeft = 600`.
            MUSKET_BALL => (4.0, 4.0, 1, 1, 600, 1, true, false, false),
            // `:1823-1832` / `:2707-2716` / `:9330-9339`: the trap darts and
            // the spiky ball — `penetrate = -1`, `trap = true`.
            TRAP_DART | TRAP_SUPERDART | TRAP_SPIKY_BALL | TRAP_VENOM_DART => {
                (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, true)
            }
            // `:6953-6966`: the Blowgun's dart — `tileCollide = false`,
            // `extraUpdates = 5`, the local immunity cooldown −1.
            POISON_DART => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 5, false, false, false),
            // `:2717-2727`: the Spear Trap's spear rides aiStyle 14,
            // `timeLeft = 900`.
            TRAP_SPEAR => (14.0, 14.0, 14, -1, 900, 0, true, false, true),
            // The wiring.device arms (M7-WIRE-2): the explosives 141's
            // explosion (`Projectile.cs:1953-1965` — aiStyle 16, 260×260,
            // `timeLeft` 2, `tileCollide = false`, trap), the land mine's
            // (`:2524-2535` — aiStyle 16, 128×128, `timeLeft` 2, trap-less),
            // and the cannon ammo 178/601 (`:2641-2649` — aiStyle 1, 10×10,
            // `timeLeft` 2 / `extraUpdates` 30).
            EXPLOSIVES_BLAST => (260.0, 260.0, 16, -1, 2, 0, false, false, true),
            LANDMINE_BLAST => (128.0, 128.0, 16, -1, 2, 0, false, false, false),
            CANNON_BOMB => (10.0, 10.0, 1, -1, 2, 0, true, false, false),
            CANNON_EXPLOSIVE => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 30, true, false, false),
            // `:864-873` / `:1345-1353` / `:2012-2022`: the boomerangs —
            // 22×22, aiStyle 3, `penetrate = -1`, melee.
            ENCHANTED_BOOMERANG | WOODEN_BOOMERANG | ICE_BOOMERANG => {
                (22.0, 22.0, 3, -1, TIME_LEFT_DEFAULT, 0, true, false, false)
            }
            // The bombs (`:1106-1121`, `:1198-1205`, `:5862-5896`); the
            // fuse seeds ride `NewProjectileSetup` (`:10970-10993`): 180
            // ticks for the bombs, 300 for the dynamite. The sticky set
            // {37, 519} disables the plain tile collide — their hover scan
            // stops them.
            BOMB | BOUNCY_BOMB => (22.0, 22.0, 16, -1, 180, 0, true, false, false),
            STICKY_BOMB => (22.0, 22.0, 16, -1, 180, 0, false, false, false),
            BOMB_FISH => (24.0, 24.0, 16, -1, 180, 0, false, false, false),
            DYNAMITE => (10.0, 10.0, 16, -1, 300, 0, true, false, false),
            // The martian family's guns + the Dutchman's cannon — the
            // `SetDefaults` rows packed into the data shape (the saucer
            // family's four consts ride the origin/main set).
            // The VortexHunter's dart (`Projectile.cs:6306-6315`): 10x10, aiStyle
            // 1, hostile, penetrate -1 (alpha 255 = no fade).
            VORTEX_DART => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            SCRAMBLER_SHOT => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The pirate trio's shots (`Projectile.cs`'s SetDefaults): the
            // The Moon Lord family (`Projectile.cs`'s SetDefaults
            // rows): the 452/454/455/456 hostile set (timeLeft 600,
            // penetrate -1 except the bolt) and the drama smoke 622.
            PHANTASMAL_EYE => (14.0, 14.0, 82, -1, 600, 0, true, false, false),
            PHANTASMAL_SPHERE => (40.0, 40.0, 83, -1, 600, 0, false, false, false),
            PHANTASMAL_DEATHRAY => (36.0, 36.0, 84, -1, 600, 0, false, false, false),
            MOON_LEECH => (16.0, 16.0, 85, -1, 600, 0, false, false, false),
            PHANTASMAL_BOLT => (8.0, 8.0, 1, 1, 3600, 3, false, false, true),
            BLOWUP_SMOKE_MOONLORD => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 2, true, false, false),
            // Deadeye's arrow 180 (4x4, aiStyle 1, penetrate -1, scale 1.2)
            // and the Corsair's musket ball 82 (10x10, aiStyle 1).
            PIRATE_ARROW => (4.0, 4.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, true, false),
            MUSKET_BALL => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false),
            MARTIAN_RAY => (8.0, 8.0, 1, 1, TIME_LEFT_DEFAULT, 3, true, false, false),
            DUTCHMAN_SHELL => (16.0, 16.0, 2, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The DD2 families' weapons (`Projectile.cs`'s SetDefaults): the
            // javelin throwers' spears 662/685 (16x16, aiStyle 1, penetrate
            // -1) and the bombers' bombs 681 (14x14, aiStyle 16, the bomb
            // flight, penetrate -1).
            DD2_SPEAR => (16.0, 16.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            DD2_SPEAR_T3 => (16.0, 16.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            DD2_BOMB => (14.0, 14.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The Dark Mage's casts (`Projectile.cs:7208-7230`): the summon
            // marker 673 + the heal marker 674 (10x10, aiStyle 133, alpha
            // 255, tileCollide false, timeLeft 900) and the shadowflame
            // 675 (10x10, aiStyle 1, alpha 255, penetrate -1).
            DARK_MAGE_MARKER => (10.0, 10.0, 133, -1, 900, 0, false, false, false),
            DARK_MAGE_FLAME => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // Betsy's breath 687 (`Projectile.cs`'s SetDefaults row: the
            // aiStyle-1 breath flight).
            BETSY_BREATH => (12.0, 12.0, 1, -1, TIME_LEFT_DEFAULT, 1, true, false, false),
            // Betsy's fireball 686 (`AI_110`'s `NewProjectile(..., 686, 35,
            // ...)` — the 1500-px leaping shot).
            BETSY_FIREBALL => (22.0, 22.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The whip family — `DefaultToWhip()`'s slice (`:10704-10718`): 18x18,
            // aiStyle 165, penetrate -1, tileCollide false, timeLeft the 3600
            // default, extraUpdates 1 (2 for the SwordWhip/ScytheWhip,
            // `:8157`/`:8166`), `usesLocalNPCImmunity` + `localNPCHitCooldown
            // = -1` (applied by `set_whip_params`). The reach lives in the
            // `tr_core::whip` curve, not in the box.
            BLAND_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            SWORD_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                2,
                false,
                false,
                false,
            ),
            MACE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            SCYTHE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                2,
                false,
                false,
                false,
            ),
            COOL_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            FIRE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            THORN_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            RAINBOW_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            BONE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            COB_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            CORRUPT_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            CRIMSON_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            METEOR_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            FLOWER_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            EEL_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            CONSTELLATION_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            MOON_LORD_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            SLIME_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false,
            ),
            _ => return None,
        };
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
    })
}

/// One pool slot (`Projectile.cs:296-470`, the engine-relevant subset).
#[derive(Clone, Debug)]
pub struct Projectile {
    pub active: bool,
    pub kind: u16,
    /// `Projectile.owner`; 255 = unset.
    pub owner: u8,
    /// `ProjectileKey.generation` (`DataStructures/ProjectileKey.cs:16-31`):
    /// bumped on every spawn into a slot so a wire consumer can tell a fresh
    /// projectile from a stale one occupying the same index.
    pub generation: u16,
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
    /// The C# `ai[0..3]` scratch array — the aiStyle-1 flight counter lives
    /// in `ai[0]` (the generic arm `ai[0] >= 15f`, `Projectile.cs:57572`),
    /// `ai[1]` is the yoyo/whip spin seed and `ai[2]` the Moon Lord whip's
    /// extra-arm counter (`AI_165_Whip`'s `ai[2] >= 10f`, `:47703`).
    pub ai: [f32; 4],
    /// The C# `localAI[0..1]` scratch array — the Moon Lord whip's arm
    /// budget roll (`localAI[1] = 1 + Main.rand.Next(3)`, `:47669-47672`)
    /// and the per-type on-hit latches (`localAI[0]`, `:13907`).
    pub local_ai: [f32; 2],
    /// `Projectile.spriteDirection` (`:47679`).
    pub sprite_direction: i32,
    /// `WhipTagEffect.CanRunHitEffects`'s minion/sentry gate
    /// (`GameContent/Items/WhipTagEffect.cs:81-85`): `minion || MinionShot ||
    /// sentry || SentryShot`. Set by the minion/sentry spawns (M4-PROJ-8);
    /// false until then — the tag damage math only fires for those shots.
    pub is_summon_shot: bool,
    /// `ApplyStatsFromSource`'s `tagEffectType` (`:11125`) — the whip's item
    /// id when the firing item carries a `UniqueTagEffects` row (M4-PROJ-7).
    pub tag_effect_type: u16,
    /// `ApplyStatsFromSource`'s `bonusTagDamage` (`:11127`) — the item's
    /// (or the parent's) bonus tag damage, the minion prefixes' row.
    pub bonus_tag_damage: i32,
    /// The whip swing's `GetWhipSettings` cache — `itemAnimationMax *
    /// MaxUpdates` (`Projectile.cs:48339`), written by [`ProjectilePool::
    /// set_whip_params`] at spawn.
    pub whip_settings: Option<tr_core::whip::WhipSettings>,
    /// The whip's held-item `useAnimation` (`FillWhipControlPoints`'s
    /// `ContentSamples.ItemsByType[heldItem.type].useAnimation`, `:48301`).
    pub whip_use_animation: f32,
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
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            active: false,
            kind: 0,
            owner: 255,
            generation: 0,
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
            ai: [0.0; 4],
            local_ai: [0.0; 2],
            sprite_direction: 1,
            is_summon_shot: false,
            tag_effect_type: 0,
            bonus_tag_damage: 0,
            whip_settings: None,
            whip_use_animation: 0.0,
            w: SIZE,
            h: SIZE,
            tile_collide: true,
            arrow: false,
            trap: false,
            bounces: 0,
            falling_tile: None,
            detonated: false,
        }
    }
}

impl Projectile {
    /// The box center — `Projectile.Center` (`Entity.cs`'s `Center`), the
    /// pool's uniform center for the box tests (the whip's *hit* comes from
    /// the swept curve, not from this).
    pub fn center(&self) -> (f32, f32) {
        (self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5)
    }

    fn shuriken(x: f32, y: f32, vx: f32, vy: f32, crit: bool) -> Self {
        Self {
            active: true,
            kind: SHURIKEN,
            owner: 0,
            generation: 0,
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
    fn ai_arrow(&mut self, world: &World) {
        self.ai[0] += 1.0;
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

    /// `AI_165_Whip()` (`Projectile.cs:47667-48270`) — the whip swing machine's
    /// state half: the arm-budget roll, the rotation, the sub-step counter, the
    /// position from the arm anchor along the velocity, the `spriteDirection`
    /// and the `timeToFlyOut` kill. The curve and the sweep are
    /// [`tr_core::whip`]'s; the dust/orchestra arms are the render follow-up.
    /// Returns the Moon Lord whip's extra-arm spawn request.
    fn ai_whip(
        &mut self,
        owner: Option<(f32, f32)>,
        aim: Option<Vec2>,
        item_rng: &mut u64,
    ) -> Option<WhipArmSpawn> {
        let kind = self.kind;
        // `:47669-47672`: the Moon Lord whip's arm budget, rolled once.
        if kind == MOON_LORD_WHIP && self.local_ai[1] == 0.0 {
            let mut rng = crate::world_sim::npcs::LootRng(item_rng);
            self.local_ai[1] = 1.0 + rng.next_range(0, 3) as f32;
        }
        // `:47674`: the rotation is the velocity's, plus a quarter turn.
        self.rot = self.vel.to_angle() + std::f32::consts::PI / 2.0;
        // `:47675`.
        self.ai[0] += 1.0;
        let Some(settings) = self.whip_settings else {
            return None;
        };
        let time_fly_out = settings.time_fly_out;
        let Some((ax, ay)) = owner else {
            return None;
        };
        // `:47678`: `Center = GetArmPosition() + velocity * (ai[0] - 1)`. The
        // port's arm anchor is the player center (the deviation record).
        let arm = Vec2::new(ax, ay);
        let center = arm + self.vel * (self.ai[0] - 1.0);
        self.pos = center - Vec2::new(self.w * 0.5, self.h * 0.5);
        // `:47679`.
        self.sprite_direction = if self.vel.x < 0.0 { -1 } else { 1 };
        // `:47680-47684`.
        if self.ai[0] >= time_fly_out {
            self.active = false;
            self.time_left = 0;
            return None;
        }
        // `:47703-47712`: the Moon Lord whip's extra arms, up to three.
        if kind == MOON_LORD_WHIP && self.ai[2] < 10.0 {
            let num = self.ai[0] / time_fly_out * 3.0;
            if self.ai[2] < num {
                self.ai[2] += 1.0;
                // The spin roll: `Main.rand.NextFloatDirection()`
                // (`Utils.cs:1203`, [−1,1)); the aim is the live one or the
                // whip's own velocity (the pool tests have no mouse).
                let mut rng = crate::world_sim::npcs::LootRng(item_rng);
                let ai1 = rng.next_float() * 2.0 - 1.0;
                let dir = aim.unwrap_or_else(|| self.vel.normalize_or_zero());
                return Some(WhipArmSpawn {
                    ai1,
                    ai2: 10.0 + self.ai[2],
                    dir,
                });
            }
        }
        None
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
        }
    }

    /// Sets the slot's crit flag (the throw path rolls it at spawn time).
    /// The `ProjectileKey` a slot's current occupant would travel under
    /// (`Projectile.cs:10854`'s key mint).
    pub fn key_of(&self, slot: usize) -> i32 {
        let p = &self.slots[slot];
        crate::net::projectile_key(p.owner, slot, p.generation)
    }

    /// The `29` despawn: the slot dies only when the key matches its current
    /// occupant (`MessageBuffer.cs:1825-1850`'s key lookup).
    pub fn kill_remote(&mut self, slot: usize, owner: u8, generation: u16) {
        if slot >= MAX_PROJECTILES {
            return;
        }
        let p = &mut self.slots[slot];
        if p.active && p.owner == owner && p.generation == generation {
            p.active = false;
            p.time_left = 0;
        }
    }

    /// Creates or patches a slot from a `27 Projectile` body
    /// (`NewProjectileSetup` + the field assignment,
    /// `MessageBuffer.cs:1729-1755`): a key match patches in place, anything
    /// else (a fresh spawn, a recycled index) re-creates the slot with the
    /// wire's owner and generation.
    pub fn upsert_remote(&mut self, st: &crate::net::ProjectileState) {
        let (owner, slot, generation) = crate::net::projectile_key_parts(st.key);
        if slot >= MAX_PROJECTILES {
            return;
        }
        let matches = self.slots[slot].active
            && self.slots[slot].owner == owner
            && self.slots[slot].generation == generation;
        if !matches {
            let mut p = Projectile {
                active: true,
                kind: st.kind.max(0) as u16,
                owner,
                generation,
                ..Default::default()
            };
            p.pos = Vec2::new(st.x, st.y);
            p.vel = Vec2::new(st.vx, st.vy);
            p.damage = st.damage as i32;
            p.knock_back = st.knock_back;
            if let Some(v) = st.ai0 {
                p.ai[0] = v;
            }
            if let Some(v) = st.ai1 {
                p.ai[1] = v;
            }
            if let Some(v) = st.ai2 {
                p.ai[2] = v;
            }
            self.slots[slot] = p;
            let base = slot * 256;
            self.local_immunity[base..base + 256].fill(0);
            return;
        }
        let p = &mut self.slots[slot];
        p.pos = Vec2::new(st.x, st.y);
        p.vel = Vec2::new(st.vx, st.vy);
        if st.kind >= 0 {
            p.kind = st.kind as u16;
        }
        p.damage = st.damage as i32;
        p.knock_back = st.knock_back;
        if let Some(v) = st.ai0 {
            p.ai[0] = v;
        }
        if let Some(v) = st.ai1 {
            p.ai[1] = v;
        }
        if let Some(v) = st.ai2 {
            p.ai[2] = v;
        }
    }

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
        let slot = self.spawn_with_data_owned(
            kind, 255, pos.x, pos.y, vel.x, vel.y, damage, knock_back, 0.0, 0.0,
        )?;
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
            ai: [1.0, 0.0, 0.0, 0.0],
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

    /// The whip's spawn completion — `DefaultToWhip()`'s immunity pair
    /// (`Projectile.cs:10704-10718`), `ApplyStatsFromSource`'s tag copy
    /// (`:11125-11136`), the `itemAnimationMax` (`:48339`'s
    /// `timeToFlyOut` input) and the `GetWhipSettings` cache. Called by the
    /// fire path right after the spawn and by the Moon Lord arms.
    pub fn set_whip_params(
        &mut self,
        slot: usize,
        tag_effect_type: u16,
        bonus_tag_damage: i32,
        item_animation_max: f32,
    ) {
        let p = &mut self.slots[slot];
        if !tr_core::whip::is_whip(p.kind) {
            return;
        }
        p.uses_local_immunity = true;
        p.local_hit_cooldown = -1;
        p.tag_effect_type = tag_effect_type;
        p.bonus_tag_damage = bonus_tag_damage;
        p.whip_use_animation = item_animation_max;
        self.set_whip_settings(slot, item_animation_max);
    }

    /// The settings slice recomputed off the slot's `ai[2]` (the Moon Lord
    /// arms' `30 * MaxUpdates` override reads it, `Projectile.cs:48413`).
    fn set_whip_settings(&mut self, slot: usize, animation: f32) {
        let (kind, max_updates, ai2) = {
            let p = &self.slots[slot];
            (p.kind, 1 + p.extra_updates, p.ai[2])
        };
        self.slots[slot].whip_settings = Some(tr_core::whip::get_whip_settings(
            kind,
            animation,
            max_updates as f32,
            ai2,
            false,
        ));
    }

    /// The Moon Lord whip's arm spawn (`Projectile.cs:47710`) — a full copy of
    /// the parent from the player center, `ai1` the spin roll and `ai2` the arm
    /// counter, with the parent's tag data (`ApplyStatsFromSource`'s
    /// `EntitySource_Parent` arm, `:11136`) and its own `GetWhipSettings`
    /// override (`30 * MaxUpdates` for the arms, `:48410-48417`).
    fn spawn_whip_arm(&mut self, parent: usize, arm: WhipArmSpawn, owner: Option<(f32, f32)>) {
        let (kind, damage, kb, tag_type, tag_bonus, animation) = {
            let p = &self.slots[parent];
            (
                p.kind,
                p.damage,
                p.knock_back,
                p.tag_effect_type,
                p.bonus_tag_damage,
                p.whip_use_animation,
            )
        };
        // `:47710`: from the player center (`player.Center.X, player.Center.Y`).
        let (cx, cy) = match owner {
            Some(c) => c,
            None => self.slots[parent].center(),
        };
        let speed = self.slots[parent].vel.length();
        let dir = if arm.dir == Vec2::ZERO {
            Vec2::X
        } else {
            arm.dir
        };
        let Some(slot) = self.spawn_with_data_ai(
            kind,
            cx,
            cy,
            dir.x * speed,
            dir.y * speed,
            damage,
            kb,
            0.0,
            arm.ai1,
        ) else {
            return;
        };
        let p = &mut self.slots[slot];
        p.ai[2] = arm.ai2;
        p.tag_effect_type = tag_type;
        p.bonus_tag_damage = tag_bonus;
        self.set_whip_settings(slot, animation);
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
        self.spawn_with_data_owned(kind, 0, x, y, vx, vy, damage, knock_back, 0.0, 0.0)
    }

    /// `spawn_with_data` with the wire owner threaded through — the port's
    /// equivalent of `Projectile.NewProjectile`'s `owner` argument: the local
    /// player's id for a client's own shots (the host relays them as `27`).
    pub fn spawn_with_data_owned(
        &mut self,
        kind: u16,
        owner: u8,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        damage: i32,
        knock_back: f32,
        ai0: f32,
        ai1: f32,
    ) -> Option<usize> {
        self.spawn_with_data_ai_owned(kind, owner, x, y, vx, vy, damage, knock_back, ai0, ai1)
    }

    /// `spawn_with_data` with the C# `NewProjectile`'s `ai0/ai1` arguments
    /// (the fireworks fountain's tile anchor, the bunny cannon's
    /// `1, owner + 1` — M7-WIRE-2). Every other arm passes 0.
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
        self.spawn_with_data_ai_owned(kind, 0, x, y, vx, vy, damage, knock_back, ai0, ai1)
    }

    /// `spawn_with_data_ai` with the wire owner; the slot's generation bumps
    /// so a stale `27` for a recycled index cannot be mistaken for the new
    /// projectile.
    pub fn spawn_with_data_ai_owned(
        &mut self,
        kind: u16,
        owner: u8,
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
            owner,
            generation: self.slots[slot].generation.wrapping_add(1),
            pos: Vec2::new(x, y),
            rot: 0.0,
            vel: Vec2::new(vx, vy),
            damage,
            knock_back,
            crit: false,
            time_left: data.time_left,
            max_penetrate: data.penetrate,
            penetrate: data.penetrate,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -1,
            extra_updates: data.extra_updates,
            ai: [ai0, ai1, 0.0, 0.0],
            w: data.w,
            h: data.h,
            tile_collide: data.tile_collide,
            arrow: data.arrow,
            trap: data.trap,
            bounces: 0,
            falling_tile: None,
            detonated: false,
            ..Default::default()
        };
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
        Some(slot)
    }

    /// `Projectile.Update` (`:16668-17208`): the AI sub-steps, the PVE damage
    /// walk, then the `timeLeft`/`penetrate` bookkeeping. The NPC per-owner
    /// i-frame decay runs with the NPC update (`NPC.cs:90454-90459`).
    /// `item_rng` is the item RNG stream the whip family's gameplay rolls take
    /// (`AI_165_Whip`'s arm budget + spin, `:47671`/`:47710`; the ThornWhip's
    /// debuff roll `:11738`); the other families do not consume it.
    pub fn update(
        &mut self,
        world: &mut World,
        npcs: &mut [Npc],
        owner: Option<(f32, f32)>,
        aim: Option<Vec2>,
        mut player: Option<&mut crate::player::Player>,
        item_rng: &mut u64,
        local_owner: u8,
        hostile_local: bool,
    ) {
        self.update_count += 1;
        for i in 0..MAX_PROJECTILES {
            if !self.slots[i].active {
                continue;
            }
            // `Projectile.Update`'s first line (`Projectile.cs:17210`): only
            // the owner's machine simulates. A slot is locally simulated when
            // its owner is this machine's player id, or — on the host /
            // single-player — the server-owned hostile shots (owner 255).
            let simulated =
                self.slots[i].owner == local_owner || (self.slots[i].owner == 255 && hostile_local);
            if !simulated {
                continue;
            }
            // `DecrementLocalImmuneTimeCounters` (`:16674`, `:17258-17277`):
            // the PVP frames plus the slot's local-NPC cooldowns decay first.
            // Only **positive** counters decay — the negative sentinels (the
            // whip's `localNPCHitCooldown = -1`) block their NPC for the
            // slot's whole life, so the C# `if (x > 0) x--` arm must be kept.
            if self.slots[i].uses_local_immunity {
                let base = i * 256;
                for v in &mut self.local_immunity[base..base + 256] {
                    if *v > 0 {
                        *v -= 1;
                    }
                }
            }
            let sub_steps = 1 + self.slots[i].extra_updates;
            for _ in 0..sub_steps {
                if self.slots[i].kind == SHURIKEN {
                    self.slots[i].ai_shuriken(world);
                }
                // The aiStyle dispatch: the arrow/bullet family (aiStyle 1,
                // `AI()`: `if (this.aiStyle == 1) this.AI_001();`) and the
                // spear/dart family (aiStyle 14). The falling-tile mode
                // keeps its custom flight.
                else if let Some(data) = proj_data(self.slots[i].kind) {
                    match data.ai_style {
                        1 => self.slots[i].ai_arrow(world),
                        3 => self.slots[i].ai_boomerang(world, owner),
                        14 => self.slots[i].ai_spear(world),
                        16 => self.slots[i].ai_bomb(world),
                        165 => {
                            // `AI_165_Whip` (`:47667-48270`): the swing
                            // machine. The Moon Lord whip's extra arms are
                            // pool spawns, so the request comes back out.
                            if let Some(arm) = self.slots[i].ai_whip(owner, aim, item_rng) {
                                self.spawn_whip_arm(i, arm, owner);
                            }
                        }
                        _ => {}
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
            self.damage_pve(i, npcs, owner, player.as_deref_mut(), item_rng);
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
        }
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
        self.damage_pve(slot, npcs, None, None, &mut 0u64);
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
    /// (`:14112-14119`). The whips replace the point test with the
    /// swept-span curve (`Projectile.Colliding`'s whip arm `:15716-15737`)
    /// and run their on-hit arms (`:13579-13956`).
    fn damage_pve(
        &mut self,
        slot: usize,
        npcs: &mut [Npc],
        owner: Option<(f32, f32)>,
        mut player: Option<&mut crate::player::Player>,
        item_rng: &mut u64,
    ) {
        let is_whip = tr_core::whip::is_whip(self.slots[slot].kind);
        let (damage, owner_idx, uses_local, uses_id_static, max_penetrate, kb, crit, center) = {
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
        // The whip sweep — `FillWhipControlPoints`' live and previous frames
        // (`frameOffset` 0 / −1, `Projectile.cs:15718-15722`), filled once per
        // tick off the owner's arm anchor.
        let (whip_now, whip_prev) = if is_whip {
            let p = &self.slots[slot];
            let Some(settings) = p.whip_settings else {
                return;
            };
            let arm = owner
                .map(|(x, y)| Vec2::new(x, y))
                .unwrap_or(p.pos + Vec2::new(p.w * 0.5, p.h * 0.5));
            let build = |frame_offset: i32| {
                let inputs = tr_core::whip::WhipCurveInputs {
                    kind: p.kind,
                    ai0: p.ai[0],
                    frame_offset,
                    ai1: p.ai[1],
                    sprite_direction: p.sprite_direction,
                    rotation: p.rot,
                    arm_position: arm,
                    held_use_animation: p.whip_use_animation,
                    whip_range_multiplier: 1.0,
                    velocity_len: p.vel.length(),
                    get_actual_collision_points: true,
                };
                tr_core::whip::fill_whip_control_points(&inputs, &settings)
            };
            (build(0), build(-1))
        } else {
            (Vec::new(), Vec::new())
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
            if !single_hit && n.immune[owner_idx] != 0 {
                continue;
            }
            let touches = if is_whip {
                let (nw, nh) = n.size();
                tr_core::whip::sweep_hits(
                    &whip_now,
                    &whip_prev,
                    self.slots[slot].w as i32,
                    self.slots[slot].h as i32,
                    tr_core::whip::Rect::new(n.x as i32, n.y as i32, nw as i32, nh as i32),
                )
            } else {
                n.contains(center.x, center.y)
            };
            if !touches {
                continue;
            }
            let hit = center;
            // `:12919-12933`: the tag-damage math runs on the hit's damage —
            // the summon/sentry gate (`WhipTagEffect.CanRunHitEffects`)
            // rejects every ported shot until M4-PROJ-8, the synthetic tests
            // set the slot flag directly.
            let (dmg, hit_crit) =
                self.tag_damage_for(slot, idx as u16, damage, crit, player.as_deref(), item_rng);
            let landed = n.hurt_from_projectile(dmg, hit.x, kb, hit_crit);
            // `:14087-14119`: the immunity application.
            if uses_local && self.slots[slot].local_hit_cooldown != -2 {
                n.immune[owner_idx] = 0;
                self.local_immunity[slot * 256 + idx] = self.slots[slot].local_hit_cooldown;
            } else if uses_id_static && self.slots[slot].id_static_hit_cooldown > 0 {
                n.immune[owner_idx] = 0;
                self.id_static.insert(
                    (self.slots[slot].kind, idx as u16),
                    self.update_count + self.slots[slot].id_static_hit_cooldown as u64,
                );
            } else if max_penetrate != 1 {
                n.immune[owner_idx] = OWNER_IMMUNE_TICKS;
            }
            if landed {
                // `:13579-13581`: the whip debuffs (`StatusNPC`'s
                // `ApplyWhipDebuffs`).
                if is_whip {
                    self.whip_debuffs(slot, n, item_rng);
                }
                // `:13581-13584`: any tag-carrying item's projectile applies
                // its tag to the hit NPC (the whip's, in the port today).
                let tag_type = self.slots[slot].tag_effect_type;
                if tag_type > 0 {
                    if let Some(p) = player.as_deref_mut() {
                        p.tag_effects.try_apply_tag_to_npc(tag_type, idx, n.kind);
                    }
                }
                // `:14258`-onwards + `:13882-13956`: the aiStyle-165 on-hit
                // arms (the minion target, the per-type switch).
                if is_whip {
                    self.whip_on_hit(slot, idx as u16, player.as_deref_mut());
                }
                // `:13658`: the tag stack's on-hit pass — the proc consume
                // (and the Constellation's immediate proc enable, its
                // `OnTaggedHit`).
                self.tag_on_hit(slot, idx as u16, player.as_deref_mut());
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

    /// The tag-damage math of `Damage_PVE_Inner`'s `12919-12933` block: the
    /// player's stack `ModifyHit`es every active tag that marks this NPC, the
    /// damage goes through `SummonTagDamageMultiplier[type]` and the total
    /// multiplier, and the tag's forced crit wins over the slot's.
    /// `WhipTagEffect.CanRunHitEffects` (`:75-86`) gates on the summon/sentry
    /// shot kinds — the port has none yet, so `is_summon_shot` (set by
    /// M4-PROJ-8's minion/sentry spawns) rejects everything until then.
    fn tag_damage_for(
        &self,
        slot: usize,
        npc_idx: u16,
        base: i32,
        crit: bool,
        player: Option<&crate::player::Player>,
        item_rng: &mut u64,
    ) -> (i32, bool) {
        let p = &self.slots[slot];
        if !p.is_summon_shot || p.owner == 255 {
            return (base, crit);
        }
        let Some(player) = player else {
            return (base, crit);
        };
        let mut changes = tr_core::whip::TagDamageChanges::none();
        let mut rng = crate::world_sim::npcs::LootRng(item_rng);
        for tag_type in player.tag_effects.active_types() {
            player
                .tag_effects
                .modify_hit(npc_idx as usize, tag_type, &mut changes, &mut rng);
        }
        let crit = changes.crit.unwrap_or(crit);
        (
            crate::tag_effects::apply_tag_damage(base, &changes, p.bonus_tag_damage, p.kind),
            crit,
        )
    }

    /// `TagEffectStack.OnHit` (`Projectile.cs:13658` → `TagEffectStack.cs:115-121`):
    /// the proc-window consume. The proc bodies' spawns are the deferred
    /// follow-ups; the consume decision and the Constellation's immediate
    /// `TryEnableProcOnNPC` (`WhipTagEffect_Constellation.cs:10-14`) land here.
    fn tag_on_hit(
        &mut self,
        slot: usize,
        npc_idx: u16,
        mut player: Option<&mut crate::player::Player>,
    ) {
        let (tag_type, proj_kind) = {
            let p = &self.slots[slot];
            (p.tag_effect_type, p.kind)
        };
        if tag_type == 0 {
            return;
        }
        if let Some(p) = player.as_deref_mut() {
            p.tag_effects.on_hit(npc_idx as usize, tag_type, proj_kind);
            if tr_core::whip::tag_effect(tag_type)
                .is_some_and(|e| tr_core::whip::constellation_enables_proc_on_tag(e))
            {
                // `WhipTagEffect_Constellation.OnTaggedHit` — the proc arms
                // on the very hit that tagged it.
                p.tag_effects.try_enable_proc(tag_type, npc_idx as usize);
            }
        }
    }

    /// `ApplyWhipDebuffs` (`Projectile.cs:11727-11750`, called from `StatusNPC`
    /// at `:13580`).
    fn whip_debuffs(&mut self, slot: usize, npc: &mut Npc, item_rng: &mut u64) {
        let kind = self.slots[slot].kind;
        match kind {
            FIRE_WHIP => npc.buffs.add_buff(323, 240), // Hellfire
            COOL_WHIP => npc.buffs.add_buff(324, 240), // Frostburn
            THORN_WHIP => {
                // `:11738` — the 1/5 roll, from the item stream.
                if crate::world_sim::npcs::LootRng(item_rng).next_range(0, 5) == 0 {
                    npc.buffs.add_buff(20, 180); // Poisoned
                }
            }
            EEL_WHIP => npc.buffs.add_buff(362, 240), // Electrified
            SCYTHE_WHIP => npc.buffs.add_buff(310, 240), // Oiled
            _ => {}
        }
    }

    /// The whip on-hit arms of `Damage_PVE_Inner` (`Projectile.cs:13579-13956`):
    /// `StatusNPC`'s `ApplyWhipDebuffs` (`:11727-11750`, called from `StatusNPC`
    /// at `:13580`), the `MinionAttackTargetNPC` override (`:14258-14263`) and
    /// the per-type switch arms that do not need the tag module — the 912/1028
    /// unconditional player buffs (`:13923`, `:13934`) and the `local_ai[0]`
    /// latches (`:13907`). The tag stack's arms (`TryApplyTagToNPC`
    /// `:13583`, the `TryEnableProcOnNPC`/`CanProcOnNPC`/`RemoveAllProcs`
    /// seams `:13912-13956`) are `tag_effects.rs`'s — see the seams marked
    /// below.
    #[allow(clippy::too_many_arguments)]
    /// The aiStyle-165 on-hit arms after the tag application
    /// (`Projectile.cs:14258-13956`'s `TryDoingOnHitEffects` positions): the
    /// `MinionAttackTargetNPC` override (`:14258-14263`) and the per-type
    /// switch (`:13882-13956`) with the tag stack's seams filled.
    fn whip_on_hit(
        &mut self,
        slot: usize,
        npc_idx: u16,
        mut player: Option<&mut crate::player::Player>,
    ) {
        let kind = self.slots[slot].kind;
        let tag_type = self.slots[slot].tag_effect_type;
        // `:14258-14263`: every aiStyle-165 hit retargets the minions.
        if let Some(p) = player.as_deref_mut() {
            p.minion_attack_target_npc = npc_idx as i16;
        }
        // `:13882-13956`: the per-type switch.
        match kind {
            FIRE_WHIP | METEOR_WHIP | FLOWER_WHIP => {
                // `:13907-13911`: the first hit of this slot arms the proc
                // window in the player's stack.
                if self.slots[slot].local_ai[0] == 0.0 {
                    self.slots[slot].local_ai[0] = 1.0;
                    if let Some(p) = player.as_deref_mut() {
                        p.tag_effects.try_enable_proc(tag_type, npc_idx as usize);
                    }
                }
            }
            COOL_WHIP => {
                // `:13923` — the unconditional player buff grant (the deferred
                // 917 snowflake spawn waits on the derived buff gate).
                if let Some(p) = player.as_deref_mut() {
                    p.buffs.add_buff(312, 180);
                }
            }
            COB_WHIP => {
                // `:13934` — same shape for the CobWhip (the deferred 1036
                // spark + the 365 gate).
                if let Some(p) = player.as_deref_mut() {
                    p.buffs.add_buff(365, 180);
                }
            }
            SWORD_WHIP | EEL_WHIP | MOON_LORD_WHIP => {
                // `:13947-13951`: the proc arms on every hit.
                if let Some(p) = player.as_deref_mut() {
                    p.tag_effects.try_enable_proc(tag_type, npc_idx as usize);
                }
            }
            CONSTELLATION_WHIP => {
                // `:13938`: the first hit with the proc window armed spawns
                // the star fan (deferred) and clears every proc window.
                let armed = player
                    .as_deref()
                    .is_some_and(|p| p.tag_effects.can_proc(tag_type, npc_idx as usize));
                if self.slots[slot].local_ai[0] == 0.0 && armed {
                    self.slots[slot].local_ai[0] = 1.0;
                    if let Some(p) = player.as_deref_mut() {
                        p.tag_effects.remove_all_procs(tag_type);
                    }
                }
            }
            _ => {}
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
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
        pool.update(
            &mut w.clone(),
            &mut [],
            Some((pcx, pcy)),
            None,
            Some(&mut player),
            &mut 7u64,
            0,
            true,
        );
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
            None,
            Some(&mut player2),
            &mut 7u64,
            0,
            true,
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
        pool.update(
            world,
            std::slice::from_mut(npc),
            None,
            None,
            None,
            &mut 7u64,
            0,
            true,
        );
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
        pool.update(
            &mut world,
            std::slice::from_mut(&mut npc),
            None,
            None,
            None,
            &mut 7u64,
            0,
            true,
        );
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
            pool.update(&mut world, &mut [], None, None, None, &mut 7u64, 0, true);
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
        pool.update(&mut world, &mut [], None, None, None, &mut 7u64, 0, true);
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
        use tr_core::tile::{Tile, FLAG_ACTIVE};
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
        pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
        pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
        use tr_core::tile::{Tile, FLAG_ACTIVE};
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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
        }
        assert_eq!(w.tile(10, 23).unwrap().kind, 53, "the sand placed back");
        assert!(!pool.slots().iter().any(|p| p.active), "the fall is spent");
    }
}

#[cfg(test)]
mod arrow_tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
        }
        assert_eq!(
            pool.slots[0].vel.y, 0.0,
            "no gravity for the first 14 ticks"
        );
        pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
        }
        assert_eq!(pool.slots[0].vel.y, 0.0, "no gravity for the first 5 ticks");
        pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() < 1e-6,
            "the 6th tick adds 0.2"
        );
    }
}

#[cfg(test)]
mod boomerang_tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
            pool.update(&mut w, &mut [], Some((320.0, 320.0)), None, None, &mut 7u64, 0, true);
        }
        assert_eq!(pool.slots[0].ai[0], 0.0, "still outbound at tick 29");
        pool.update(&mut w, &mut [], Some((320.0, 320.0)), None, None, &mut 7u64, 0, true);
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
            pool.update(&mut w, &mut [], Some((100.0, 260.0)), None, None, &mut 7u64, 0, true);
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
        pool.slots[0].ai = [1.0, 0.0, 0.0, 0.0];
        let mut w = world();
        // A wall directly on the path home.
        for y in 18..20 {
            for x in 20..22 {
                put_solid(&mut w, x, y);
            }
        }
        let mut caught = false;
        for _ in 0..40 {
            pool.update(&mut w, &mut [], Some((80.0, 308.0)), None, None, &mut 7u64, 0, true);
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
        pool.slots[0].ai = [1.0, 0.0, 0.0, 0.0];
        let mut w = world();
        let mut died = false;
        for _ in 0..120 {
            pool.update(&mut w, &mut [], Some((100.0, 200.0)), None, None, &mut 7u64, 0, true);
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
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
            pool.update(&mut w, &mut npcs, None, None, None, &mut 7u64, 0, true);
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
        pool.update(&mut w, &mut [], None, None, None, &mut 7u64, 0, true);
        assert_eq!(pool.slots[0].damage, 0, "the flight damage zeroes");
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() > 1e-6,
            "no gravity before tick 5"
        );
        for _ in 0..5 {
            pool.update(&mut w, &mut [], None, None, None, &mut 7u64, 0, true);
        }
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() < 1e-6,
            "the gravity tail adds 0.2"
        );
    }
}

// --- the whip family (M4-PROJ-7) -----------------------------------------
#[cfg(test)]
mod whip_tests {
    use super::*;

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    /// A solid-floor world — the slime needs a spawn column
    /// (`Npc::spawn_slime` above solid tiles).
    fn ground() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
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

    // `DefaultToWhip()`'s slice + `AI_165_Whip`'s state machine, the swept
    // hit and the on-hit arms. The pure geometry is `tr_core::whip`'s tests.
    /// One swing sub-step with the owner's center as the arm anchor (the
    /// `GetArmPosition` stand-in) and the aim along +X.
    fn whip_tick(
        pool: &mut ProjectilePool,
        world: &mut World,
        npcs: &mut [Npc],
        center: (f32, f32),
    ) {
        let mut rng = 7u64;
        pool.update(
            world,
            npcs,
            Some(center),
            Some(Vec2::new(1.0, 0.0)),
            None,
            &mut rng,
            0,
            true,
        );
    }

    fn whip_at(center: (f32, f32), kind: u16, damage: i32) -> ProjectilePool {
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(kind, center.0, center.1, 4.0, 0.0, damage, 2.0)
            .expect("a free slot");
        pool.set_whip_params(slot, 0, 0, 30.0);
        pool
    }

    #[test]
    fn the_whip_data_slice_is_verbatim() {
        for kind in tr_core::whip::IS_A_WHIP {
            let data = proj_data(kind).expect("a whip row");
            assert_eq!((data.w, data.h), (18.0, 18.0), "{kind}");
            assert_eq!(data.ai_style, 165, "{kind}");
            assert_eq!(data.penetrate, -1, "{kind}");
            assert!(!data.tile_collide, "{kind}");
            let extra = if kind == SWORD_WHIP || kind == SCYTHE_WHIP {
                2
            } else {
                1
            };
            assert_eq!(data.extra_updates, extra, "{kind}");
        }
        assert_eq!(proj_data(MOON_LORD_WHIP).unwrap().extra_updates, 1);
        // Not a whip.
        assert!(proj_data(864).map_or(true, |d| d.ai_style != 165));
    }

    #[test]
    fn the_whip_swing_flies_out_from_the_anchor_and_dies() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let mut pool = whip_at(center, BLAND_WHIP, 14);
        // `:47674` — the rotation is the velocity's + a quarter turn, and the
        // `spriteDirection` faces the aim. `extraUpdates = 1` runs 2 sub-steps
        // per tick, so one tick lands `ai[0] = 2`.
        whip_tick(&mut pool, &mut w, &mut [], center);
        assert!((pool.slots[0].rot - std::f32::consts::PI / 2.0).abs() < 1e-6);
        assert_eq!(pool.slots[0].sprite_direction, 1);
        assert_eq!(pool.slots[0].ai[0], 2.0, "the sub-step counter");
        // `:47678`: the last sub-step's center sits at `arm + velocity * (ai0-1)`.
        let (cx, cy) = pool.slots[0].center();
        assert!((cx - (center.0 + 4.0)).abs() < 1e-3 && (cy - center.1).abs() < 1e-3);
        // The swing runs `timeToFlyOut = 30 * 2 = 60` sub-steps and then kills.
        let mut ticks = 1;
        while pool.slots[0].active {
            whip_tick(&mut pool, &mut w, &mut [], center);
            ticks += 1;
            assert!(ticks < 200, "the swing must end");
        }
        assert_eq!(ticks, 30, "60 sub-steps at 2 per tick");
        assert_eq!(pool.slots[0].ai[0], 60.0, "the sub-steps ran out");
        // The SwordWhip/ScytheWhip's 3 sub-steps per tick shorten the tick
        // count, not the sub-step count.
        let mut pool2 = {
            let mut p = ProjectilePool::new();
            let s = p
                .spawn_with_data(SWORD_WHIP, center.0, center.1, 4.0, 0.0, 55, 2.0)
                .unwrap();
            p.set_whip_params(s, 0, 0, 28.0);
            p
        };
        let mut ticks2 = 0;
        while pool2.slots[0].active {
            whip_tick(&mut pool2, &mut w, &mut [], center);
            ticks2 += 1;
            assert!(ticks2 < 200);
        }
        assert_eq!(ticks2, 28, "84 sub-steps at 3 per tick");
        assert_eq!(pool2.slots[0].local_hit_cooldown, -1, "the sentinel");
        assert_eq!(pool2.slots[0].ai[0], 28.0 * 3.0, "the sub-steps ran out");
    }

    #[test]
    fn the_whip_hits_each_npc_once_across_the_swing() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let mut npcs = [one_npc(&w)];
        // The NPC sits in the whip's reach along the aim.
        npcs[0].x = center.0 + 20.0;
        npcs[0].y = center.1 - 12.0;
        let start_hp = npcs[0].hp;
        let mut pool = whip_at(center, BLAND_WHIP, 14);
        let mut ticks = 0;
        while pool.slots[0].active {
            whip_tick(&mut pool, &mut w, &mut npcs, center);
            ticks += 1;
            assert!(ticks < 200);
        }
        // The `-1` sentinel blocks the re-hits for the slot's whole life, so
        // the total taken is exactly one hit (the decay regression: the old
        // `(v - 1).max(0)` decay turned the sentinel into 0 on the next tick).
        assert_eq!(npcs[0].hp, start_hp - 14, "exactly one hit per slot");
        assert_eq!(npcs[0].immune[0], 0, "the per-owner i-frames do not gate");
        // The slot's own immunity row for the NPC holds the sentinel.
        assert_eq!(pool.local_immunity[..256][0], -1);
    }

    #[test]
    fn the_whip_debuff_rows_are_verbatim() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let cases: [(u16, i32, i32); 5] = [
            (FIRE_WHIP, 323, 240),   // Hellfire
            (COOL_WHIP, 324, 240),   // Frostburn
            (EEL_WHIP, 362, 240),    // Electrified
            (SCYTHE_WHIP, 310, 240), // Oiled
            (THORN_WHIP, 20, 180),   // Poisoned (the 1/5 roll — force it)
        ];
        for (kind, buff, time) in cases {
            // The Slime is Poisoned-immune (`debuff_immune_ids`' kind-1
            // row), so the Zombie (no immunity rows) carries the debuff cases.
            let mut npcs = [Npc::spawn_zombie(&w, 20).unwrap()];
            npcs[0].x = center.0 + 20.0;
            npcs[0].y = center.1 - 12.0;
            let mut pool = whip_at(center, kind, 20);
            let mut rng = 3u64;
            // The swing's reach grows with the curve, so run the swing until
            // the sweep touches the NPC (the debuff lands with the hit).
            let mut ticks = 0;
            while npcs[0].buffs.find_buff_index(buff).is_none() {
                pool.update(
                    &mut w,
                    &mut npcs,
                    Some(center),
                    Some(Vec2::new(1.0, 0.0)),
                    None,
                    &mut rng,            0,
            true,

                );
                ticks += 1;
                assert!(ticks < 60, "{kind} must hit within its swing");
            }
            let idx = npcs[0].buffs.find_buff_index(buff).unwrap();
            assert_eq!(npcs[0].buffs.buff_time[idx], time, "{kind}");
        }
    }

    #[test]
    fn the_moon_lord_whip_spawns_its_arms() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let mut pool = whip_at(center, MOON_LORD_WHIP, 110);
        let mut rng = 7u64;
        // The arm budget roll (`:47669-47672`) fires on the first sub-step.
        pool.update(
            &mut w,
            &mut [],
            Some(center),
            Some(Vec2::new(1.0, 0.0)),
            None,
            &mut rng,            0,
            true,

        );
        assert!(
            (1.0..=3.0).contains(&pool.slots[0].local_ai[1]),
            "the budget roll is 1..3"
        );
        // The three arms spawn while `ai[0]/timeToFlyOut*3 > ai[2]` and carry
        // the ai[2] counter and the arms' settings override (`30 * MaxUpdates`)
        // — each lives 60 sub-steps of its own, so count the concurrency peak.
        let mut peak_arms = 0;
        for _ in 0..60 {
            pool.update(
                &mut w,
                &mut [],
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                None,
                &mut rng,            0,
            true,

            );
            let arms = pool
                .slots()
                .iter()
                .filter(|p| p.active && p.ai[2] >= 10.0)
                .count();
            peak_arms = peak_arms.max(arms);
        }
        assert_eq!(peak_arms, 3, "the Moon Lord whip throws its three arms");
        // The arms that are still alive carry the override.
        let alive: Vec<_> = pool
            .slots()
            .iter()
            .filter(|p| p.active && p.ai[2] >= 10.0)
            .collect();
        for p in alive {
            let settings = p.whip_settings.expect("the arm's settings");
            assert_eq!(settings.time_fly_out, 60.0, "the arm lives 30*MaxUpdates");
            assert_eq!(p.tag_effect_type, 0);
        }
    }

    #[test]
    fn the_whip_hit_tags_the_npc_for_the_summon_bonus() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let mut player = crate::player::Player::spawn(&w);
        let mut npcs = [Npc::spawn_zombie(&w, 20).unwrap()];
        npcs[0].x = center.0 + 20.0;
        npcs[0].y = center.1 - 12.0;
        let npc_idx = 0usize;
        let mut pool = whip_at(center, COOL_WHIP, 45);
        // `ApplyStatsFromSource` — the firing item's tag row (4911 = CoolWhip).
        pool.slots[0].tag_effect_type = 4911;
        let mut rng = 3u64;
        for _ in 0..60 {
            pool.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                &mut rng,            0,
            true,

            );
            if !pool.slots[0].active {
                break;
            }
        }
        // `:13581-13584`: the hit applied the tag to the player's stack.
        assert!(
            player.tag_effects.is_npc_tagged(4911, npc_idx),
            "the whip tags the NPC for the summoners"
        );
    }

    #[test]
    fn a_tagged_summon_shot_takes_the_damage_bonus() {
        let mut w = ground();
        let mut player = crate::player::Player::spawn(&w);
        let mut npcs = [Npc::spawn_zombie(&w, 20).unwrap()];
        // The tag is applied out-of-band (another whip already tagged it).
        player
            .tag_effects
            .try_apply_tag_to_npc(4672, 0, npcs[0].kind);
        let start = npcs[0].hp;
        // A friendly arrow-style shot over the NPC with the summon-shot flag
        // the tag math gates on (`WhipTagEffect.CanRunHitEffects`).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(
                WOODEN_ARROW,
                npcs[0].x + 5.0,
                npcs[0].y + 5.0,
                0.0,
                0.0,
                10,
                0.0,
            )
            .unwrap();
        pool.slots[slot].is_summon_shot = true;
        let mut rng = 3u64;
        pool.update(&mut w, &mut npcs, None, None, Some(&mut player), &mut rng, 0, true);
        // The BlandWhip's TagDamage 4 at the 1f multiplier: 10 + 4 = 14.
        assert_eq!(start - npcs[0].hp, 14, "the tagged summon shot takes +4");
        // Without the gate the shot is a plain hit.
        let start2 = npcs[0].hp;
        let mut pool2 = ProjectilePool::new();
        let s2 = pool2
            .spawn_with_data(
                WOODEN_ARROW,
                npcs[0].x + 5.0,
                npcs[0].y + 5.0,
                0.0,
                0.0,
                10,
                0.0,
            )
            .unwrap();
        pool2.slots[s2].is_summon_shot = false;
        pool2.update(&mut w, &mut npcs, None, None, Some(&mut player), &mut rng, 0, true);
        assert_eq!(start2 - npcs[0].hp, 10, "the ungated shot is plain");
    }

    #[test]
    fn the_constellation_first_hit_arms_and_clears_the_proc_window() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let mut player = crate::player::Player::spawn(&w);
        let mut npcs = [Npc::spawn_zombie(&w, 20).unwrap()];
        // Two swings must land on the same NPC — park it above its death.
        npcs[0].hp = 4000;
        npcs[0].x = center.0 + 20.0;
        npcs[0].y = center.1 - 12.0;
        let mut pool = whip_at(center, CONSTELLATION_WHIP, 130);
        pool.slots[0].tag_effect_type = 5479;
        let mut rng = 3u64;
        // The swing until the hit lands.
        let mut ticks = 0;
        while !player.tag_effects.is_npc_tagged(5479, 0) {
            pool.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                &mut rng,            0,
            true,

            );
            ticks += 1;
            assert!(ticks < 60, "the whip must hit");
        }
        // `WhipTagEffect_Constellation.OnTaggedHit` arms the window on the
        // tagging hit (the mark stands 480 ticks — the second swing's first
        // hit consumes it and clears every proc window).
        assert!(
            player.tag_effects.can_proc(5479, 0),
            "the tagging hit arms the proc window"
        );
        // The second swing: its first hit latches (the `OnTaggedHit` re-arm
        // keeps the window standing, so the latch is the observable).
        let mut pool2 = whip_at(center, CONSTELLATION_WHIP, 130);
        pool2.slots[0].tag_effect_type = 5479;
        let mut ticks2 = 0;
        while pool2.slots[0].local_ai[0] == 0.0 {
            pool2.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                &mut rng,
                0,
                true,
            );
            ticks2 += 1;
            assert!(ticks2 < 60, "the second whip must hit");
        }
        assert_eq!(pool2.slots[0].local_ai[0], 1.0, "the latch");
    }

    #[test]
    fn the_cool_whip_grants_its_buff_on_every_hit() {
        let mut w = ground();
        let center = (300.0, 200.0);
        let mut player = crate::player::Player::spawn(&w);
        let mut npcs = [one_npc(&w)];
        npcs[0].x = center.0 + 20.0;
        npcs[0].y = center.1 - 12.0;
        let mut pool = whip_at(center, COOL_WHIP, 45);
        pool.slots[0].tag_effect_type = 4911;
        let mut rng = 7u64;
        for _ in 0..60 {
            pool.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                &mut rng,            0,
            true,

            );
            if !pool.slots[0].active {
                break;
            }
        }
        // `:13923` — the unconditional 312/180 grant (the deferred 917 spawn).
        let idx = player
            .buffs
            .find_buff_index(312)
            .expect("the CoolWhip buff");
        assert_eq!(player.buffs.buff_time[idx], 180);
    }
}
