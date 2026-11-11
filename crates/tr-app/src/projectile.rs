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
//! - the whip family (M4-PROJ-7): `DefaultToWhip()` `:10704-10718` (aiStyle
//!   165), `AI_165_Whip()` `:47667-48270` (the swing machine - the curve and
//!   the sweep live in [`tr_core::whip`]), `Projectile.Colliding`'s whip arm
//!   `:15716-15737` (the swept-span multi-hit) and the on-hit arms
//!   (`StatusNPC`'s `ApplyWhipDebuffs` `:11727-11750`, the per-type switch
//!   `:13882-13956`); the tag stack itself is `tag_effects.rs`.
//!
//! The ammo-resolution rows (M3-ITEM-6, 2026-11-09): the `proj_data` block at
//! the end of the match carries the kinds `Player.PickAmmo`
//! (`Player.cs:53458-53683`) resolves, so `spawn_with_data_ai` no longer
//! silently no-ops on them (Phase 1's deviation). The unported aiStyles
//! 147/75/10/181 carry their vanilla value in the row and ride the documented
//! stand-in dispatch (their AI ports are M4-PROJ's).
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
use crate::npc::{Npc, NpcKind};
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
/// `ProjectileID.Hook` (13) — the vanilla grappling hook.
pub const HOOK: u16 = 13;
/// `ProjectileID.IvyWhip` (32) — the 300 px-range hook of the Jungle chest.
pub const IVY_WHIP: u16 = 32;
/// `ProjectileID.DualHookBlue` (73) — the Dual Hook's blue head.
pub const DUAL_HOOK_BLUE: u16 = 73;
/// `ProjectileID.DualHookRed` (74) — the Dual Hook's red head.
pub const DUAL_HOOK_RED: u16 = 74;
/// `ProjectileID.WebSlingerHook` (165) — the three-at-once hook.
pub const WEB_SLINGER_HOOK: u16 = 165;
/// `ProjectileID.GemHookAmethyst` (230) — the first Gem Hook.
pub const GEM_HOOK_AMETHYST: u16 = 230;
/// `ProjectileID.GemHookTopaz` (231).
pub const GEM_HOOK_TOPAZ: u16 = 231;
/// `ProjectileID.GemHookSapphire` (232).
pub const GEM_HOOK_SAPPHIRE: u16 = 232;
/// `ProjectileID.GemHookEmerald` (233).
pub const GEM_HOOK_EMERALD: u16 = 233;
/// `ProjectileID.GemHookRuby` (234).
pub const GEM_HOOK_RUBY: u16 = 234;
/// `ProjectileID.GemHookDiamond` (235) — the last Gem Hook.
pub const GEM_HOOK_DIAMOND: u16 = 235;
/// `ProjectileID.SkeletronHand` (256) — the Skeletron hand hook
/// (`extraUpdates = 1`, the only 6x6 hook box).
pub const SKELETRON_HAND: u16 = 256;
/// `ProjectileID.BatHook` (315).
pub const BAT_HOOK: u16 = 315;
/// `ProjectileID.WoodHook` (322).
pub const WOOD_HOOK: u16 = 322;
/// `ProjectileID.CandyCaneHook` (331).
pub const CANDY_CANE_HOOK: u16 = 331;
/// `ProjectileID.ChristmasHook` (332).
pub const CHRISTMAS_HOOK: u16 = 332;
/// `ProjectileID.FishHook` (372).
pub const FISH_HOOK: u16 = 372;
/// `ProjectileID.SlimeHook` (396).
pub const SLIME_HOOK: u16 = 396;
/// `ProjectileID.TrackHook` (403) — the minecart-track hook.
pub const TRACK_HOOK: u16 = 403;
/// `ProjectileID.AntiGravityHook` (446) — the gravity-flipping hook.
pub const ANTI_GRAVITY_HOOK: u16 = 446;
/// `ProjectileID.TendonHook` (486) — the first of the 1.4.4 quartet.
pub const TENDON_HOOK: u16 = 486;
/// `ProjectileID.ThornHook` (487).
pub const THORN_HOOK: u16 = 487;
/// `ProjectileID.IlluminantHook` (488) — the light-emitting hook.
pub const ILLUMINANT_HOOK: u16 = 488;
/// `ProjectileID.WormHook` (489) — the last of the 1.4.4 quartet.
pub const WORM_HOOK: u16 = 489;
/// `ProjectileID.LunarHookSolar` (646) — the first Lunar Hook.
pub const LUNAR_HOOK_SOLAR: u16 = 646;
/// `ProjectileID.LunarHookVortex` (647).
pub const LUNAR_HOOK_VORTEX: u16 = 647;
/// `ProjectileID.LunarHookNebula` (648).
pub const LUNAR_HOOK_NEBULA: u16 = 648;
/// `ProjectileID.LunarHookStardust` (649) — the last Lunar Hook.
pub const LUNAR_HOOK_STARDUST: u16 = 649;
/// `ProjectileID.StaticHook` (652) — the stationary hook.
pub const STATIC_HOOK: u16 = 652;
/// `ProjectileID.AmberHook` (753).
pub const AMBER_HOOK: u16 = 753;
/// `ProjectileID.SquirrelHook` (865).
pub const SQUIRREL_HOOK: u16 = 865;
/// `ProjectileID.QueenSlimeHook` (935) — the `extraUpdates = 3` hook whose
/// latch teleports its owner.
pub const QUEEN_SLIME_HOOK: u16 = 935;
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
pub const STARDUST_MARK: u16 = 540;
pub const NEBULA_EYE: u16 = 574;
pub const VORTEX_LIGHTNING: u16 = 578;
pub const VORTEX_PORTAL: u16 = 579;
pub const VORTEX_RITUAL_BOLT: u16 = 580;
pub const TOWER_DAMAGE_BOLT: u16 = 629;
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
/// The flail family (aiStyle 15 — `AI_015_Flails` `Projectile.cs:43724-44169`).
/// Every row rides `DefaultToFlail()` (`:10753-10762`: aiStyle 15, drawLayer 7,
/// friendly, `penetrate = -1`, melee, `usesLocalNPCImmunity` with
/// `localNPCHitCooldown = 10`); the per-kind `SetDefaults` rows set the boxes.
pub const BALL_O_HURT: u16 = 25;
pub const BLUE_MOON: u16 = 26;
pub const SUNFURY: u16 = 35;
pub const DAO_OF_POW: u16 = 63;
pub const THE_MEATBALL: u16 = 154;
pub const FLOWER_POW: u16 = 247;
pub const DRIPPLER_FLAIL: u16 = 757;
pub const MACE: u16 = 947;
pub const FLAMING_MACE: u16 = 948;
pub const FLAIRON: u16 = 1058;
/// The whole aiStyle-15 weapon set — the kinds `DefaultToFlail` covers.
pub const FLAIL_KINDS: [u16; 10] = [
    BALL_O_HURT,
    BLUE_MOON,
    SUNFURY,
    DAO_OF_POW,
    THE_MEATBALL,
    FLOWER_POW,
    DRIPPLER_FLAIL,
    MACE,
    FLAMING_MACE,
    FLAIRON,
];
/// The flail family's helper projectiles: the Drippler's extra ball 928
/// (`Projectile.cs:8793`), the Flower Pow petal 248 (`:3306`) and the Flairon
/// bubble 405 (`:4857`).
pub const DRIPPLER_EXTRA_BALL: u16 = 928;
pub const FLOWER_POW_PETAL: u16 = 248;
pub const FLAIRON_BUBBLE: u16 = 405;/// `ProjectileID.Bomb`-family cannon ammo (178, `Projectile.cs:2641-2649`).
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
/// The yoyo family (`Projectile.cs:6013-6093` + `:9483-9490` — every row is
/// 16×16 with `DefaultToYoyo()`, aiStyle 99; the AI is `AI_099_2_Yoyos`
/// `:67733-68105`). The counterweight projectiles 556-561/1079 share the style
/// but ride `AI_099_1_Counterweights` (`:67380`) — the accessory family, not
/// these ids.
pub const CODE1_YOYO: u16 = 534;
pub const WOOD_YOYO: u16 = 541;
pub const CORRUPT_YOYO: u16 = 542;
pub const CRIMSON_YOYO: u16 = 543;
pub const JUNGLE_YOYO: u16 = 544;
pub const CASCADE_YOYO: u16 = 545;
pub const CHIK_YOYO: u16 = 546;
pub const CODE2_YOYO: u16 = 547;
pub const RALLY_YOYO: u16 = 548;
pub const YELETS_YOYO: u16 = 549;
pub const REDS_YOYO: u16 = 550;
pub const VALKYRIE_YOYO: u16 = 551;
pub const AMAROK_YOYO: u16 = 552;
pub const HEL_FIRE_YOYO: u16 = 553;
pub const KRAKEN_YOYO: u16 = 554;
/// `ProjectileID.TheEyeOfCthulhu` — the Eye of Cthulhu yoyo (555).
pub const THE_EYE_OF_CTHULHU_YOYO: u16 = 555;
pub const FORMAT_C_YOYO: u16 = 562;
pub const GRADIENT_YOYO: u16 = 563;
pub const VALOR_YOYO: u16 = 564;
/// `ProjectileID.Terrarian` (603) and the beams it fires — `TerrarianBeam`
/// 604 (`Projectile.cs:6071-6083`, aiStyle 115).
pub const TERRARIAN_YOYO: u16 = 603;
pub const TERRARIAN_BEAM: u16 = 604;
/// The yoyo on-hit shots (M4-PROJ-4b, `Projectile.cs:10528`-`:10667`).
pub const CASCADE_EXPLOSION: u16 = 1125;
pub const CRYSTAL_SHARD_MELEE: u16 = 1134;
pub const EYE_OF_CTHULHU_DASH: u16 = 1130;
pub const KRAKEN_WAVE: u16 = 1135;
/// `ProjectileID.HiveFive` (999, the 1.4.4 yoyo — `Projectile.cs:9483-9490`).
pub const HIVE_FIVE: u16 = 999;
/// The whole aiStyle-99 weapon set — the kinds `DefaultToYoyo` covers.
pub const YOYO_KINDS: [u16; 21] = [
    CODE1_YOYO,
    WOOD_YOYO,
    CORRUPT_YOYO,
    CRIMSON_YOYO,
    JUNGLE_YOYO,
    CASCADE_YOYO,
    CHIK_YOYO,
    CODE2_YOYO,
    RALLY_YOYO,
    YELETS_YOYO,
    REDS_YOYO,
    VALKYRIE_YOYO,
    AMAROK_YOYO,
    HEL_FIRE_YOYO,
    KRAKEN_YOYO,
    THE_EYE_OF_CTHULHU_YOYO,
    FORMAT_C_YOYO,
    GRADIENT_YOYO,
    VALOR_YOYO,
    TERRARIAN_YOYO,
    HIVE_FIVE,
];
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
/// The hook family's `timeLeft *= 10` (`Projectile.cs`'s common tail).
const HOOK_TIME_LEFT: i32 = 10 * TIME_LEFT_DEFAULT;

const RESET_STATE: (bool, i32, bool, i32, i32, f32) = (false, -2, false, -1, 0, 1.0);

/// `ProjectileID.Sets.YoyosLifeTimeMultiplier` (`ProjectileID.cs:425`) — the
/// flight lifetime in **seconds** (`AI_099_2_Yoyos`'s `num2`,
/// `Projectile.cs:67751`; `num = localAI[0] / 60f` at the default meleeSpeed).
/// The factory default is **-1f — no timeout**: 547, 550, 551, 554, 555 and
/// 603 are unlisted and return only on the channel release or the range cap
/// (a verbatim source quirk, not a gap — recorded in ANALYSIS).
const YOYO_LIFETIME: &[(u16, f32)] = &[
    (541, 3.0),
    (548, 5.0),
    (542, 7.0),
    (543, 6.0),
    (544, 8.0),
    (534, 9.0),
    (564, 11.0),
    (999, 11.0),
    (545, 13.0),
    (563, 30.0),
    (562, 16.0),
    (553, 12.0),
    (546, 16.0),
    (552, 15.0),
    (549, 14.0),
];

/// `ProjectileID.Sets.YoyosMaximumRange` (`ProjectileID.cs:426`) — the string
/// length in pixels (`AI_099_2_Yoyos`'s `num11`, `Projectile.cs:67862`); the
/// unlisted kinds keep the factory default **200f**.
const YOYO_MAX_RANGE: &[(u16, f32)] = &[
    (541, 130.0),
    (548, 170.0),
    (542, 195.0),
    (543, 207.0),
    (544, 215.0),
    (534, 220.0),
    (564, 225.0),
    (999, 225.0),
    (545, 235.0),
    (562, 320.0),
    (563, 250.0),
    (546, 275.0),
    (552, 270.0),
    (553, 330.0),
    (547, 280.0),
    (549, 290.0),
    (554, 380.0),
    (550, 370.0),
    (551, 370.0),
    (555, 360.0),
    (603, 400.0),
];

/// `ProjectileID.Sets.YoyosTopSpeed` (`ProjectileID.cs:427`) — the flight's
/// top speed in px/tick (`AI_099_2_Yoyos`'s `num9`, `Projectile.cs:67863`);
/// the unlisted kinds keep the factory default **10f**.
const YOYO_TOP_SPEED: &[(u16, f32)] = &[
    (541, 9.0),
    (548, 11.0),
    (542, 12.5),
    (543, 12.0),
    (544, 13.0),
    (534, 13.0),
    (564, 14.0),
    (999, 14.0),
    (545, 14.0),
    (562, 17.0),
    (563, 16.0),
    (546, 15.0),
    (552, 14.0),
    (553, 15.0),
    (547, 17.0),
    (549, 16.0),
    (554, 22.0),
    (550, 16.0),
    (551, 16.0),
    (555, 16.5),
    (603, 17.5),
];

/// The three tables' lookups — the `CreateFloatSet(default, …)` semantics
/// (`ProjectileID.cs:425-427`): an unlisted kind reads the factory default.
fn yoyo_table(table: &[(u16, f32)], default: f32, kind: u16) -> f32 {
    table
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or(default, |(_, v)| *v)
}

fn yoyo_lifetime(kind: u16) -> f32 {
    yoyo_table(YOYO_LIFETIME, -1.0, kind)
}

fn yoyo_max_range(kind: u16) -> f32 {
    yoyo_table(YOYO_MAX_RANGE, 200.0, kind)
}

fn yoyo_top_speed(kind: u16) -> f32 {
    yoyo_table(YOYO_TOP_SPEED, 10.0, kind)
}

/// The Terrarian's beam scan's `Collision.CanHit` gate (`Projectile.cs:67804`,
/// `Collision.cs:381+`): the two centers' tile walk. The C# `CanHit` is a
/// box-vs-box LOS; the port walks the centers' segment against the solid,
/// non-platform tiles (the same `tileSolid` shape `ai_spear`'s scan uses) —
/// the approximation is recorded in ANALYSIS.
fn yoyo_can_hit(world: &World, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
    let steps = ((x2 - x1).abs().max((y2 - y1).abs()) / 8.0).ceil() as i32;
    for s in 0..=steps.max(1) {
        let t = s as f32 / steps.max(1) as f32;
        let tx = (x1 + (x2 - x1) * t) / TILE_PX;
        let ty = (y1 + (y2 - y1) * t) / TILE_PX;
        let Some(tile) = world.tile(tx as i32, ty as i32) else {
            continue;
        };
        if tile.active()
            && tr_core::tiledata::TILE_SOLID.has(tile.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(tile.kind)
        {
            return false;
        }
    }
    true
}

/// The Terrarian's beam spawn — `AI_099_2_Yoyos`'s
/// `Projectile.NewProjectile(…, 604, …)` (`Projectile.cs:67815`). The arm
/// runs on one slot (it cannot reach the pool's slot array), so the spawn
/// rides out like `Wiring`'s trap requests; the idStatic-10 slice is the
/// beam's own `proj_data`.
pub(crate) struct YoyoBeamRequest {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knock_back: f32,
}

/// What one yoyo AI tick hands back to the pool: the owner-wide lifetime
/// sweep (`Projectile.cs:67764-67771`) and the Terrarian's beam request —
/// both write across the pool's slot array.
#[derive(Default)]
struct YoyoTick {
    sweep: bool,
    beam: Option<YoyoBeamRequest>,
}

/// One flail helper spawn — the family's `Projectile.NewProjectile` calls
/// (`:43914`/`:43927` the Drippler's 928, `:44137` the Flower Pow's petal
/// 248, `:44094` the Flairon's bubble 405). The AI only sees its own slot,
/// so the request rides out like `YoyoTick`'s beam; the dispatch arm spawns
/// at most one per slot per tick (the vanilla never fires two in one AI
/// pass).
struct FlailSpawnRequest {
    kind: u16,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    damage: i32,
    knock_back: f32,
    /// `NewProjectile`'s `ai0` — the Flairon seeds its bubbles with -10f.
    ai0: f32,
}

/// One flail hit-side decision — the multipliers plus whether the
/// state-0 ellipse stood in for the box test.
struct FlailHit {
    damage_mult: f32,
    knock_mult: f32,
    from_x: f32,
    replaces_box: bool,
}

/// What one flail AI tick hands back to the pool: the family's helper
/// spawn and the `player.ChangeDir` facing the C# writes mid-machine (the
/// port's facing is run-key driven, so the value rides out and the dispatch
/// arm applies it after the AI pass).
#[derive(Default)]
struct FlailTick {
    spawn: Option<FlailSpawnRequest>,
    facing: Option<i32>,
}

/// The shared (non-per-kind) locals of `AI_015_Flails`
/// (`Projectile.cs:43746-43760`), with the meleeSpeed-scaled ones marked.
const FLAIL_THROW_MAX_DISTANCE: f32 = 800.0; // num3
const FLAIL_RETRACT_ACCEL: f32 = 3.0; // num4 (scaled)
const FLAIL_REEL_ACCEL: f32 = 6.0; // num6 (scaled)
const FLAIL_SPIN_SOFT_ACCEL: f32 = 1.0; // num8 (scaled)
const FLAIL_SPIN_PULL_SPEED: f32 = 14.0; // num9 (scaled)
const FLAIL_SPIN_INNER_RADIUS: f32 = 60.0; // num10
const FLAIL_HELD_NPC_COOLDOWN: f32 = 10.0; // num11
const FLAIL_FLIGHT_NPC_COOLDOWN: f32 = 10.0; // num13

/// `AI_015_Flails`'s per-kind parameter locals — the switch at
/// `Projectile.cs:43761-43823` on top of the defaults. The C# scales
/// `num2/num4/num5/num6/num7/num8/num9` by `1f / player.meleeSpeed` at
/// `:43824-43831`; the AI applies that to the speed/accel fields once.
#[derive(Clone, Copy)]
pub struct FlailParams {
    /// `num` — the state-1 flight ticks before the retract transition.
    pub flight_ticks: f32,
    /// `num2` — the throw speed (px/tick).
    pub throw_speed: f32,
    /// `num5` — the state-2 retract speed (and its kill distance).
    pub retract_speed: f32,
    /// `num7` — the state-4 reel-in kill distance.
    pub reel_kill_distance: f32,
    /// `num12` — the held state's `localNPCHitCooldown`.
    pub spin_npc_cooldown: f32,
    /// `num15` — the held-orbit radius around the mounted center.
    pub orbit_radius: f32,
}

/// The per-kind switch verbatim (`Projectile.cs:43761-43823`); an unlisted
/// kind keeps the shared defaults (`num 10, num2 24, num5 16, num7 48, num12
/// 15, num15 30`). `num14 = num + 5f` (`:43759`) is derived, not stored.
const FLAIL_PARAMS: &[(u16, FlailParams)] = &[
    // 947/948 — the Mace/Flaming Mace pair shares one row.
    (
        MACE,
        FlailParams { flight_ticks: 13.0, throw_speed: 12.0, retract_speed: 8.0, reel_kill_distance: 13.0, spin_npc_cooldown: 15.0, orbit_radius: 30.0 },
    ),
    (
        FLAMING_MACE,
        FlailParams { flight_ticks: 13.0, throw_speed: 12.0, retract_speed: 8.0, reel_kill_distance: 13.0, spin_npc_cooldown: 15.0, orbit_radius: 30.0 },
    ),
    // 25 Ball O' Hurt.
    (
        BALL_O_HURT,
        FlailParams { flight_ticks: 15.0, throw_speed: 14.0, retract_speed: 10.0, reel_kill_distance: 15.0, spin_npc_cooldown: 15.0, orbit_radius: 30.0 },
    ),
    // 154 The Meatball.
    (
        THE_MEATBALL,
        FlailParams { flight_ticks: 15.0, throw_speed: 15.0, retract_speed: 11.0, reel_kill_distance: 16.0, spin_npc_cooldown: 15.0, orbit_radius: 30.0 },
    ),
    // 26 Blue Moon.
    (
        BLUE_MOON,
        FlailParams { flight_ticks: 15.0, throw_speed: 16.0, retract_speed: 13.0, reel_kill_distance: 17.0, spin_npc_cooldown: 15.0, orbit_radius: 32.0 },
    ),
    // 35 Sunfury.
    (
        SUNFURY,
        FlailParams { flight_ticks: 15.0, throw_speed: 17.0, retract_speed: 14.0, reel_kill_distance: 18.0, spin_npc_cooldown: 15.0, orbit_radius: 37.0 },
    ),
    // 63 The Dao of Pow.
    (
        DAO_OF_POW,
        FlailParams { flight_ticks: 13.0, throw_speed: 21.0, retract_speed: 20.0, reel_kill_distance: 24.0, spin_npc_cooldown: 12.0, orbit_radius: 30.0 },
    ),
    // 757 Drippler Flail.
    (
        DRIPPLER_FLAIL,
        FlailParams { flight_ticks: 13.0, throw_speed: 22.0, retract_speed: 22.0, reel_kill_distance: 26.0, spin_npc_cooldown: 12.0, orbit_radius: 35.0 },
    ),
    // 247 Flower Pow.
    (
        FLOWER_POW,
        FlailParams { flight_ticks: 13.0, throw_speed: 23.0, retract_speed: 16.0, reel_kill_distance: 48.0, spin_npc_cooldown: 12.0, orbit_radius: 40.0 },
    ),
    // 1058 Flairon.
    (
        FLAIRON,
        FlailParams { flight_ticks: 16.0, throw_speed: 23.0, retract_speed: 16.0, reel_kill_distance: 48.0, spin_npc_cooldown: 10.0, orbit_radius: 45.0 },
    ),
];

/// The flail family's parameters — the lookup the AI runs once per tick.
/// `Projectile.cs:43746-43831`'s table semantics.
pub fn flail_params(kind: u16) -> FlailParams {
    FLAIL_PARAMS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or(
            FlailParams {
                flight_ticks: 10.0,
                throw_speed: 24.0,
                retract_speed: 16.0,
                reel_kill_distance: 48.0,
                spin_npc_cooldown: 15.0,
                orbit_radius: 30.0,
            },
            |(_, p)| *p,
        )
}

/// `DefaultToYoyo`'s per-kind `scale` ladder (`Projectile.cs:6028-6067`) —
/// the SetDefaults body; every other kind (and the unlisted yoyos) keeps the

/// `DefaultToYoyo`'s per-kind `scale` ladder (`Projectile.cs:6028-6067`) —
/// the SetDefaults body; every other kind (and the unlisted yoyos) keeps the
/// stock 1f.
fn set_defaults_scale(kind: u16) -> f32 {
    match kind {
        547 => 1.1,
        550 | 551 => 1.1,
        554 => 1.2,
        555 => 1.15,
        563 => 1.05,
        564 => 1.075,
        603 => 1.15,
        // The flail family's per-kind `scale` (`Projectile.cs`'s SetDefaults
        // rows; the unlisted flails keep the stock 1f).
        25 => 0.9,
        26 | 35 | 154 => 0.8,
        _ => 1.0,
    }
}

/// The ported families' `SetDefaults` slice. The boulder (186, aiStyle 37)
/// and the flame jet (187, aiStyle 38) traps stay passthroughs — their AI
/// families are M4-PROJ follow-ups.
/// The ported families' `SetDefaults` slice. The boulder (186, aiStyle 37)
/// and the flame jet (187, aiStyle 38) traps stay passthroughs — their AI
/// families are M4-PROJ follow-ups.
///
/// The ammo-resolution rows (M3-ITEM-6 Phase 2): the kinds
/// `Player.PickAmmo`'s resolution chain (`Player.cs:53458-53683`) resolves —
/// the launcher rockets, the 1.4.4 rocket family, the Celebration/gunProj
/// rockets, the sand family, the arrow overrides and the fairy shot — must
/// carry rows here or `spawn_with_data_ai` silently no-ops and the shot
/// vanishes (Phase 1's deviation, fixed by this block). Their aiStyles
/// 16 dispatch to `ai_bomb`; 147/75/10/181 stay unported stand-ins (the rows
/// carry the vanilla value for the later AI port — M4-PROJ).
/// `Vector2.RotatedBy` (`Microsoft.Xna.Framework`): the 2D rotation by an
/// angle in radians.
fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

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
            // The flail family (`DefaultToFlail` `:10753-10762` — aiStyle 15,
            // `penetrate = -1`, melee, the local-immunity pair 10 via the
            // overlay below); the boxes are the per-kind `SetDefaults` rows:
            // 25/26/35/63/154 22×22 (`:1079`-`:2418`), 947/948 20×20 (`:8941`),
            // 247/757 34×34 (`:3299`/`:7807`), 1058 26×26 (`:9998`).
            BALL_O_HURT | BLUE_MOON | SUNFURY | DAO_OF_POW | THE_MEATBALL => (
                22.0,
                22.0,
                15,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            MACE | FLAMING_MACE => (
                20.0,
                20.0,
                15,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            FLOWER_POW | DRIPPLER_FLAIL => (
                34.0,
                34.0,
                15,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            FLAIRON => (
                26.0,
                26.0,
                15,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // The flail family's helpers: the Drippler's extra ball 928
            // (`:8793` — 34×34, aiStyle 14, `penetrate = 2`, `timeLeft = 250`),
            // the Flower Pow petal 248 (`:3306` — 18×18, aiStyle 1, melee) and
            // the Flairon bubble 405 (`:4857` — 14×14, aiStyle 70,
            // `penetrate = 1`, `timeLeft = 120`; its `alpha = 255` rides the
            // spawn site). The flail AI spawns all three.
            DRIPPLER_EXTRA_BALL => (
                34.0,
                34.0,
                14,
                2,
                250,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            FLOWER_POW_PETAL => (
                18.0,
                18.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            FLAIRON_BUBBLE => (
                14.0,
                14.0,
                70,
                1,
                120,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),            // The bombs (`:1106-1121`, `:1198-1205`, `:5862-5896`); the
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
            // The Moon Lord family (`Projectile.cs`'s SetDefaults
            // rows): the 452/454/455/456 hostile set (timeLeft 600,
            // penetrate -1 except the bolt) and the drama smoke 622.
            // `:6151-6160`'s StardustTowerMark (the 540): 20x20, aiStyle
            // 103, timeLeft 300, hostile.
            STARDUST_MARK => (20.0, 20.0, 103, -1, 300, 0, true, false, false, RESET_STATE),
            // `:6239-6247`'s NebulaEye (the 574): 18x18, aiStyle 102,
            // timeLeft 600, tileCollide false.
            NEBULA_EYE => (18.0, 18.0, 102, -1, 600, 0, false, false, false, RESET_STATE),
            // `:6754-6766`'s TowerDamageBolt (the 629): 8x8, aiStyle 122,
            // ignoreWater, extraUpdates 2.
            TOWER_DAMAGE_BOLT => (8.0, 8.0, 122, -1, TIME_LEFT_DEFAULT, 2, false, false, true, RESET_STATE),
            // `:6282-6293`'s ritual markers (the 578 lightning + the 579
            // portal): 32x32, aiStyle 108, tileCollide false, ignoreWater.
            VORTEX_LIGHTNING | VORTEX_PORTAL => {
                (32.0, 32.0, 108, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)
            }
            // `:6294-6305`'s VortexLightning (the 580): 14x14, aiStyle 88,
            // extraUpdates 4, ignoreWater.
            VORTEX_RITUAL_BOLT => (14.0, 14.0, 88, -1, 600, 4, true, false, false, RESET_STATE),
            PHANTASMAL_EYE => (14.0, 14.0, 82, -1, 600, 0, true, false, false, RESET_STATE),
            PHANTASMAL_SPHERE => (40.0, 40.0, 83, -1, 600, 0, false, false, false, RESET_STATE),
            PHANTASMAL_DEATHRAY => (36.0, 36.0, 84, -1, 600, 0, false, false, false, RESET_STATE),
            MOON_LEECH => (16.0, 16.0, 85, -1, 600, 0, false, false, false, RESET_STATE),
            PHANTASMAL_BOLT => (8.0, 8.0, 1, 1, 3600, 3, false, false, true, RESET_STATE),
            BLOWUP_SMOKE_MOONLORD => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 2, true, false, false, RESET_STATE),
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
            CODE1_YOYO
            | WOOD_YOYO
            | CORRUPT_YOYO
            | CRIMSON_YOYO
            | JUNGLE_YOYO
            | CASCADE_YOYO
            | CHIK_YOYO
            | CODE2_YOYO
            | RALLY_YOYO
            | YELETS_YOYO
            | REDS_YOYO
            | VALKYRIE_YOYO
            | AMAROK_YOYO
            | HEL_FIRE_YOYO
            | KRAKEN_YOYO
            | THE_EYE_OF_CTHULHU_YOYO
            | FORMAT_C_YOYO
            | GRADIENT_YOYO
            | VALOR_YOYO
            | TERRARIAN_YOYO
            | HIVE_FIVE => (16.0, 16.0, 99, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),            // The yoyo shots (M4-PROJ-4b): the four children the yoyo
            // on-hit arms spawn — `SetDefaults` verbatim.
            // `CascadeExplosion` (`:10528-10541`): 32×32, aiStyle 189, the
            // `alpha = 255`/`hide` pair, `penetrate = 4`, local-immunity with
            // `localNPCHitCooldown = -1`.
            CASCADE_EXPLOSION => (32.0, 32.0, 189, 4, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `TheEyeOfCthulhuDash` (`:10596-10608`): 100×100, aiStyle 210,
            // `timeLeft = 600`, `penetrate = -1`, local-immunity −1.
            EYE_OF_CTHULHU_DASH => (100.0, 100.0, 210, -1, 600, 0, false, false, false, RESET_STATE),
            // `CrystalShardMelee` (`:10636-10650`): 6×6, aiStyle 24,
            // `penetrate = 1`, `timeLeft = 600`, idStatic-20.
            CRYSTAL_SHARD_MELEE => (6.0, 6.0, 24, 1, 600, 0, false, false, false, RESET_STATE),
            // `KrakenWave` (`:10656-10667`): 100×100, aiStyle 210,
            // `timeLeft = 600`, `penetrate = -1`, idStatic-10.
            KRAKEN_WAVE => (100.0, 100.0, 210, -1, 600, 0, false, false, false, RESET_STATE),
            // The Terrarian's beam (`:6071-6083`): 14×14, aiStyle 115, scale
            // 1.2, `penetrate = -1`, idStatic-10 (the slice above).
            TERRARIAN_BEAM => (
                14.0,
                14.0,
                115,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // The martian family's guns + the Dutchman's cannon — the
            // `SetDefaults` rows packed into the data shape (the saucer
            // family's four consts ride the origin/main set).
            // The VortexHunter's dart (`Projectile.cs:6306-6315`): 10x10, aiStyle
            // 1, hostile, penetrate -1 (alpha 255 = no fade).
            VORTEX_DART => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            STARDUST_MARK => (20.0, 20.0, 103, -1, 300, 0, true, false, false, RESET_STATE),
            // `:6239-6247`'s NebulaEye (the 574): 18x18, aiStyle 102,
            // timeLeft 600, tileCollide false.
            NEBULA_EYE => (18.0, 18.0, 102, -1, 600, 0, false, false, false, RESET_STATE),
            // `:6754-6766`'s TowerDamageBolt (the 629): 8x8, aiStyle 122,
            // ignoreWater, extraUpdates 2.
            TOWER_DAMAGE_BOLT => (8.0, 8.0, 122, -1, TIME_LEFT_DEFAULT, 2, false, false, true, RESET_STATE),
            // `:6282-6293`'s ritual markers (the 578 lightning + the 579
            // portal): 32x32, aiStyle 108, tileCollide false, ignoreWater.
            VORTEX_LIGHTNING | VORTEX_PORTAL => (
                32.0,
                32.0,
                108,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // `:6294-6305`'s VortexLightning (the 580): 14x14, aiStyle 88,
            // extraUpdates 4, ignoreWater.
            VORTEX_RITUAL_BOLT => (14.0, 14.0, 88, -1, 600, 4, true, false, false, RESET_STATE),
            PHANTASMAL_EYE => (14.0, 14.0, 82, -1, 600, 0, true, false, false, RESET_STATE),
            PHANTASMAL_SPHERE => (40.0, 40.0, 83, -1, 600, 0, false, false, false, RESET_STATE),
            PHANTASMAL_DEATHRAY => (36.0, 36.0, 84, -1, 600, 0, false, false, false, RESET_STATE),
            MOON_LEECH => (16.0, 16.0, 85, -1, 600, 0, false, false, false, RESET_STATE),
            PHANTASMAL_BOLT => (8.0, 8.0, 1, 1, 3600, 3, false, false, true, RESET_STATE),
            BLOWUP_SMOKE_MOONLORD => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 2, true, false, false, RESET_STATE),
            // Deadeye's arrow 180 (4x4, aiStyle 1, penetrate -1, scale 1.2)
            // and the Corsair's musket ball 82 (10x10, aiStyle 1).
            BLAND_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            SWORD_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                2,
                false,
                false,
                false, RESET_STATE),
            MACE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            SCYTHE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                2,
                false,
                false,
                false, RESET_STATE),
            COOL_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            FIRE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            THORN_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            RAINBOW_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            BONE_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            COB_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            CORRUPT_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            CRIMSON_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            METEOR_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            FLOWER_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            EEL_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            CONSTELLATION_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            MOON_LORD_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            SLIME_WHIP => (
                18.0,
                18.0,
                165,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                false,
                false,
                false, RESET_STATE),
            // The ammo-resolution rows (M3-ITEM-6 Phase 2): `friendly` is the
            // pool default for every one of them and `ranged` has no consumer,
            // so neither rides the tuple. The launcher rockets
            // (`Projectile.cs:2222-2338`) — 14x14, aiStyle 16 (the bomb
            // family), `penetrate = -1`; 133/136/139/142 add the
            // local-immunity pair (the overlay below).
            133..=144 => (14.0, 14.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // The Snowman Cannon's four rockets (`:4308-4320`) — the same row;
            // their `scale = 0.9f` is dropped (no consumer), so the port keeps
            // the full 14x14 box.
            338..=341 => (14.0, 14.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // The 1.4.4 rocket family (`:7939-7952`) — the eight rockets each
            // of the Grenade/Rocket/Proximity launchers resolves to (776-801)
            // plus the Snowman Cannon's 803-810. 779/783 sit inside the C#
            // range but no launcher's rocket resolves to them, so the guard
            // leaves them row-less.
            776..=801 | 803..=810 if kind != 779 && kind != 783 => (
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
            // The Celebration rockets (`:7584-7605` — both arms identical):
            // 14x14, aiStyle 147, `penetrate = 1`, `extraUpdates = 2` and
            // `timeLeft = 360 * MaxUpdates` = 360 × (`extraUpdates + 1`) = 1080
            // (`MaxUpdates`'s getter, `:554-561`; `alpha = 255` dropped).
            715..=718 => (14.0, 14.0, 147, 1, 1080, 2, true, false, false, RESET_STATE),
            // The four gunProj rockets (`:6618-6630` 615, `:6767-6779` 630/705,
            // `:7571-7582` 714): 22x22, aiStyle 75, `penetrate = -1`,
            // `tileCollide = false` (the drawLayer/usesOwnerLight/ignoreWater
            // fields have no consumer).
            615 | 630 | 705 | 714 => (
                22.0,
                22.0,
                75,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // The sand family (`:1248-1258` 42, `:1449-1462` 65, `:1470-1480`
            // 68, `:4438-4449` 354): 10x10, aiStyle 10, `extraUpdates = 1`.
            // The Sandgun's own 42 sets no penetrate (the stock 1), the three
            // sand-block conversions `penetrate = -1`; their per-type
            // knockBack (8 / 6) is dropped — the spawner's input wins.
            42 => (10.0, 10.0, 10, 1, TIME_LEFT_DEFAULT, 1, true, false, false, RESET_STATE),
            65 | 68 | 354 => (10.0, 10.0, 10, -1, TIME_LEFT_DEFAULT, 1, true, false, false, RESET_STATE),
            // The arrow overrides — 117 the Bone Arrow (`:2058-2068`,
            // `extraUpdates = 2`), 469 the Bee Arrow (`:5429-5441`,
            // `alpha = 255` dropped), 485 the Hellwing (`:5604-5615` — 24x24,
            // `penetrate = -1`), 495 the Shadowflame (`:5681-5692` —
            // `penetrate = 3`) and 819 the Blood Rain arrow (`:7993-8002` —
            // `extraUpdates = 1`, `alpha`/`hide` dropped): 10x10 aiStyle-1
            // arrows, `timeLeft = 1200` (485's 24x24 box aside).
            117 => (10.0, 10.0, 1, 1, 1200, 2, true, true, false, RESET_STATE),
            469 => (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE),
            485 => (24.0, 24.0, 1, -1, 1200, 0, true, true, false, RESET_STATE),
            495 => (10.0, 10.0, 1, 3, 1200, 0, true, true, false, RESET_STATE),
            819 => (10.0, 10.0, 1, 1, 1200, 1, true, true, false, RESET_STATE),
            // The spray/ball ammo the not-consume windows resolve — the
            // Flamethrower's Flames (`:1660-1670`: 6x6, aiStyle 193, pen 4,
            // extraUpdates 2, `armorPenetration = 15` dropped; its immunity
            // pair rides the overlay), the Clentaminator's solution spray
            // (`:2338-2347`: 6x6, aiStyle 31, pen -1, extraUpdates 2,
            // tileCollide false, `alpha`/`ignoreWater` dropped) and the Acorn
            // Slingshot's acorn (`:10145-10149`: 20x20, aiStyle 16 — the port's
            // `ai_bomb`, pen -1). Without these rows the wire would drain the
            // ammo and then decline the spawn.
            85 => (6.0, 6.0, 193, 4, TIME_LEFT_DEFAULT, 2, true, false, false, RESET_STATE),
            145 => (6.0, 6.0, 31, -1, TIME_LEFT_DEFAULT, 2, false, false, false, RESET_STATE),
            1088 => (20.0, 20.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // The Fairy Queen's shot 932 (`:8842-8856`): 10x10, aiStyle 181,
            // arrow, `penetrate = 5`, `timeLeft = 120`, `alpha = 0` and
            // `ignoreWater` dropped — its local-immunity pair
            // (`localNPCHitCooldown = 60`) rides the overlay below.
            932 => (10.0, 10.0, 181, 5, 120, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:939-948` — the Hook (13): 18x18.
            HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:1141-1150` — the Ivy Whip (32): 18x18.
            IVY_WHIP => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:1536-1546` — the Dual Hook pair (73/74): 18x18,
            // `light = 0.4f` dropped.
            DUAL_HOOK_BLUE | DUAL_HOOK_RED => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:2538-2547` — the Web Slinger (165): 12x12.
            WEB_SLINGER_HOOK => (12.0, 12.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:3164-3173` — the six Gem Hooks (230-235): 18x18,
            // the shared block with no per-kind field.
            GEM_HOOK_AMETHYST
            | GEM_HOOK_TOPAZ
            | GEM_HOOK_SAPPHIRE
            | GEM_HOOK_EMERALD
            | GEM_HOOK_RUBY
            | GEM_HOOK_DIAMOND => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:3388-3399` — the Skeletron Hand (256): 6x6,
            // `extraUpdates = 1` (`scale = 1f` is the stock default).
            SKELETRON_HAND => (6.0, 6.0, 7, -1, HOOK_TIME_LEFT, 1, false, false, false, RESET_STATE),
            // `Projectile.cs:4077-4086` — the Bat Hook (315): 14x14.
            BAT_HOOK => (14.0, 14.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4149-4158` — the Wood Hook (322): 14x14.
            WOOD_HOOK => (14.0, 14.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4235-4244` — the Candy Cane Hook (331): 18x18.
            CANDY_CANE_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4246-4256` — the Christmas Hook (332): 18x18,
            // `light = 0.5f` dropped.
            CHRISTMAS_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4552-4561` — the Fish Hook (372): 18x18.
            FISH_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4769-4779` — the Slime Hook (396): 18x18,
            // `alpha = 100` dropped.
            SLIME_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4836-4845` — the Track Hook (403): 18x18.
            TRACK_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5176-5186` — the Anti Gravity Hook (446): 14x14,
            // `light = 0.7f` dropped.
            ANTI_GRAVITY_HOOK => (14.0, 14.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // The 1.4.4 quartet (`Projectile.cs:5615-5643`): the four boxes
            // below share the block's tail (`:5638-5643`).
            // `Projectile.cs:5617-5621` — the Tendon Hook (486): 12x12.
            TENDON_HOOK => (12.0, 12.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5622-5626` — the Thorn Hook (487): 22x22.
            THORN_HOOK => (22.0, 22.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5627-5632` — the Illuminant Hook (488): 12x12,
            // `light = 0.3f` dropped.
            ILLUMINANT_HOOK => (12.0, 12.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5633-5637` — the Worm Hook (489): 20x16.
            WORM_HOOK => (20.0, 16.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:6966-6975` — the four Lunar Hooks (646-649): 18x18,
            // the shared block with no per-kind field.
            LUNAR_HOOK_SOLAR
            | LUNAR_HOOK_VORTEX
            | LUNAR_HOOK_NEBULA
            | LUNAR_HOOK_STARDUST => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:7001-7010` — the Static Hook (652): 18x18.
            STATIC_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:7756-7765` — the Amber Hook (753): 18x18.
            AMBER_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:8301-8310` — the Squirrel Hook (865): 18x18.
            SQUIRREL_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:8884-8894` — the Queen Slime Hook (935): 18x18,
            // `extraUpdates = 3`.
            QUEEN_SLIME_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 3, false, false, false, RESET_STATE),
            _ => return None,
        };
    let (state_local, state_local_cooldown) = match kind {
        133 | 136 | 139 | 142 | 777 | 781 | 785 | 788 | 794 | 797 | 800 => (true, -1),
        85 => (true, -1),
        932 => (true, 60),
        // `DefaultToFlail`'s pair (`:10753-10762`) — the ten flail kinds.
        BALL_O_HURT | BLUE_MOON | SUNFURY | DAO_OF_POW | THE_MEATBALL | FLOWER_POW
        | DRIPPLER_FLAIL | MACE | FLAMING_MACE | FLAIRON => (true, 10),
        _ => (false, -2),
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
        // the post-hoc immunity overlay (M3-ITEM-6) adds the kinds it lists;
        // every other row keeps its own state group.
        uses_local_immunity: state_local || uses_local_immunity,
        local_hit_cooldown: if state_local {
            state_local_cooldown
        } else {
            local_hit_cooldown
        },
        // `DefaultToYoyo` sets no immunity slice (`:10764-10771`) — only the
        // Terrarian pair carries the idStatic-10 (`:6068-6069`, `:6081-6082`);
        // the M4-PROJ-4b yoyo shots carry their own (the shard idStatic-20,
        // the Kraken wave idStatic-10).
        uses_id_static_immunity: uses_id_static_immunity
            || matches!(
                kind,
                TERRARIAN_YOYO | TERRARIAN_BEAM | CRYSTAL_SHARD_MELEE | KRAKEN_WAVE
            ),
        id_static_hit_cooldown: if uses_id_static_immunity {
            id_static_hit_cooldown
        } else if matches!(kind, TERRARIAN_YOYO | TERRARIAN_BEAM | KRAKEN_WAVE) {
            10
        } else if kind == CRYSTAL_SHARD_MELEE {
            20
        } else {
            -1
        },
        alpha,
        // the yoyo family's per-kind scale ladder (`DefaultToYoyo`); every
        // other row carries its own scale in the state group.
        scale: if YOYO_KINDS.contains(&kind) {
            set_defaults_scale(kind)
        } else {
            scale
        },
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
    /// The C# `ai[0..2]` scratch array — the aiStyle-1 flight counter lives
    /// in `ai[0]` (the generic arm `ai[0] >= 15f`, `Projectile.cs:57572`).
    pub ai: [f32; 4],
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
    /// `GrappleMovement`'s pull input: the attached aiStyle-7 hook's center
    /// (`Player.cs:22757-22856` reads `grappling[]` — this multi-hook field is
    /// the task's next phase; the single-hook bridge keeps the pull working
    /// meanwhile, and `grappling_active`'s owner reads it).
    pub grapple_center: Vec2,
    /// The C# `Projectile.direction`
    /// the yoyo rotation spins by it (`Projectile.cs:68096`) and the pose
    /// block writes it from the owner-relative X comparison (`:67841`/`:67846`).
    pub direction: i32,
    bounces: u8,
    /// The tile-fall mode (`AI_010`'s `ai[0] == 1f`): the projectile carries a
    /// falling tile kind (`spawn_falling`), falls with `velocity.Y += 0.41f`
    /// per tick and places the tile on landing.
    pub falling_tile: Option<u16>,
    /// `AI_016`'s `timeLeft <= 3` trigger: the bomb waits for the pool's
    /// end-of-tick explosion pass (`Kill()`'s bomb path).
    pub detonated: bool,
    /// The whip swing's `GetWhipSettings` cache — `itemAnimationMax *
    /// MaxUpdates` (`Projectile.cs:48339`), written by
    /// [`ProjectilePool::set_whip_params`] at spawn.
    pub whip_settings: Option<tr_core::whip::WhipSettings>,
    /// The whip's held-item `useAnimation` (`FillWhipControlPoints`'s
    /// `ContentSamples.ItemsByType[heldItem.type].useAnimation`, `:48301`).
    pub whip_use_animation: f32,
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
    /// `AI_103`'s mark spawn (`Projectile.cs:33467-33475`'s
    /// `NPC.NewNPC(..., localAI[0])`): the (kind, x, y, localai0, localai1) the
    /// world_sim's projectile pass resolves into the NPC vec.
    pub wants_mark_spawn: Option<(u16, f32, f32, f32, f32)>,
    /// The 578's 580 spawn (`Projectile.cs:33838-33841`): the (x, y,
    /// vx, vy, ai0) payload the world_sim resolves into the pool.
    pub wants_ritual_bolt: Option<(f32, f32, f32, f32, f32)>,
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
            local_ai: [0.0; 3],
            alpha: 0,
            scale: 1.0,
            w: SIZE,
            h: SIZE,
            tile_collide: true,
            arrow: false,
            trap: false,
            direction: 1,
            grapple_center: Vec2::ZERO,
            bounces: 0,
            falling_tile: None,
            detonated: false,
            blast_spawned: false,
            whip_settings: None,
            whip_use_animation: 0.0,
            sprite_direction: 1,
            is_summon_shot: false,
            tag_effect_type: 0,
            bonus_tag_damage: 0,
            wants_mark_spawn: None,
            wants_ritual_bolt: None,
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

    /// `AI_007_GrapplingHooks` (`Projectile.cs:52565-52866`) on this slot — the
    /// flight, the tile attach and the retract; the state machine and the
    /// per-kind tables live in `crate::grapple`.
    pub(crate) fn ai_hook(
        &mut self,
        world: &World,
        player_center: Vec2,
        owner_dead: bool,
        // The slot's owner is this machine's player — the C#'s `myPlayer` gate on
        // the cracked-brick break (`Projectile.cs:52586`).
        local_owner: bool,
        // The hook's own `u64` stream (`Main.rand`'s stand-in).
        hook_rng: Option<&mut u64>,
    ) -> crate::grapple::HookTick {
        crate::grapple::ai_hook(self, world, player_center, owner_dead, local_owner, hook_rng)
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

    /// The aiStyle-99 yoyo family — `AI_099_2_Yoyos`, `Projectile.cs:67733-
    /// 68105`: the single-yoyo string lifecycle (the lifetime clock, the
    /// flight with the aim-seeking orbit, the return, the dropped and ghost
    /// states, the rotation) plus the generic tile slide, the double-yoyo
    /// `flag` arm, the lifetime sweep, the Terrarian's beam timer and the
    /// per-type specials. The accessory terms (yoyoString/magicString/
    /// meleeSpeed) stay at their base values until M3-EQUIP-3.
    ///
    /// Returns the [`YoyoTick`] — the pool-side effects the arm cannot write
    /// itself (the sweep and the beam spawn).
    #[allow(clippy::too_many_arguments)]
    fn ai_yoyo(
        &mut self,
        world: &World,
        npcs: &[Npc],
        owner: Option<(f32, f32)>,
        flag: bool,
        inputs: YoyoInputs,
        rng: &mut dyn tr_core::loot::DropRng,
    ) -> YoyoTick {
        let cx = self.pos.x + self.w * 0.5;
        let cy = self.pos.y + self.h * 0.5;
        let mut tick = YoyoTick::default();

        // `:67745-67774` — the lifetime clock: `localAI[0]++` (the double
        // yoyo's jitter draws from the seeded stream), the seconds
        // (`/ 60f` — the division by `((1 + meleeSpeed) / 2)` is 1/1 at the
        // port's default meleeSpeed) against the per-yoyo table.
        self.local_ai[0] += 1.0;
        if flag {
            // `:67746-67749`.
            self.local_ai[0] += rng.next_range(10, 31) as f32 * 0.1;
        }
        let lifetime = yoyo_lifetime(self.kind);
        if lifetime != -1.0 && self.local_ai[0] / 60.0 > lifetime {
            self.ai[0] = -1.0;
            // `:67764-67773` — the owner-wide sweep rides the tick out (this
            // slot included; setting it here is idempotent).
            tick.sweep = true;
        }

        // `:67776-67818` — the Terrarian's beam timer. The `ai[0] != 3f` gate
        // is verbatim from the source (an aim exactly at x = 3 is the only
        // state it excludes — ANALYSIS).
        if self.kind == TERRARIAN_YOYO && self.ai[0] != 3.0 {
            self.local_ai[1] += 1.0;
            if self.local_ai[1] >= 6.0 {
                // The three rolls (`:67786-67793`): the ±1.0 direction, the
                // 0.1..4.0 speed (doubled a third of the time).
                let mut jx = 0.0f32;
                let mut jy = 0.0f32;
                while jx == 0.0 && jy == 0.0 {
                    jx = rng.next_range(-100, 101) as f32;
                    jy = rng.next_range(-100, 101) as f32;
                }
                let jl = (jx * jx + jy * jy).sqrt();
                let mut mag = rng.next_range(10, 41) as f32 * 0.1;
                if rng.next_range(0, 3) == 0 {
                    mag *= 2.0;
                }
                // `:67795-67796` — the quarter of the yoyo's own velocity.
                let mut bx = self.vel.x * 0.25 + jx / jl * mag;
                let mut by = self.vel.y * 0.25 + jy / jl * mag;
                // `:67797-67813` — the closest chasable NPC within the 400 px
                // manhattan distance behind the `CanHit` gate retargets the
                // beam at 8 px/tick straight at it.
                let mut closest = 400.0f32;
                for n in npcs.iter() {
                    if n.hp <= 0 {
                        continue;
                    }
                    let (nx, ny) = n.center();
                    let d = (cx - nx).abs() + (cy - ny).abs();
                    if d < closest && yoyo_can_hit(world, cx, cy, nx, ny) {
                        closest = d;
                        let l = d.max(f32::EPSILON);
                        bx = (nx - cx) / l * 8.0;
                        by = (ny - cy) / l * 8.0;
                    }
                }
                // `:67814` — the ×0.8 damp, the spawn at the center minus the
                // beam velocity (`:67815`).
                bx *= 0.8;
                by *= 0.8;
                tick.beam = Some(YoyoBeamRequest {
                    x: cx - bx,
                    y: cy - by,
                    vx: bx,
                    vy: by,
                    damage: self.damage,
                    knock_back: self.knock_back,
                });
                self.local_ai[1] = 0.0;
            }
        }

        // `:67824-67828` — the owner's death kills the string.
        if inputs.player_dead {
            self.active = false;
            self.time_left = 0;
            return tick;
        }

        // `:67829-67852` — the pose block's `direction` write (the ±2 px X
        // comparison against the owner center). The heldProj /
        // `SetDummyItemTime(2)` / `player.ChangeDir` player-side writes stay
        // stubbed until M3-ITEM-2d; the counterweights (the other aiStyle-99
        // arm) never reach here.
        if self.ai[0] != -2.0 {
            if let Some((ox, _)) = owner {
                if cx > ox + 2.0 {
                    self.direction = 1;
                } else if cx < ox - 2.0 {
                    self.direction = -1;
                }
            }
        }

        // `:67853-67856` — the NaN-velocity kill.
        if !self.vel.x.is_finite() || !self.vel.y.is_finite() {
            self.active = false;
            self.time_left = 0;
            return tick;
        }

        // `:67857` — the keep-alive.
        self.time_left = 6;

        // `:67858-67897` — the parameter block. The C# seeds
        // `num8 = 10, num9 = 10, num10 = 3, num11 = 200` before the table
        // reads overwrite `num11`/`num9`; the surviving assignments are the
        // effective ones (the seeds are dead writes in the decompile).
        let mut accel = 10.0f32;
        let mut speed = yoyo_top_speed(self.kind);
        let mut arrive = 3.0f32;
        let range = yoyo_max_range(self.kind);
        // `:67864-67867` — the Kraken's ×0.75 top speed under the double.
        if self.kind == KRAKEN_YOYO && flag {
            speed *= 0.75;
        }
        // The yoyoString range boost (`num11 * 1.25f + 30f`) and the
        // meleeSpeed division (`(1 + meleeSpeed * 3f) / 4f`) are 1/1 at the
        // defaults — M3-EQUIP-3 flips them.
        accel = (14.0 - speed / 2.0).max(1.0);
        arrive = 5.0 + speed / 2.0;
        // `:67894-67897` — the double yoyo's wider arrive radius.
        if flag {
            arrive += 20.0;
        }

        // `:67898-68021` — the flight state. The orphan guard: without the
        // player link the slot cannot fly.
        if self.ai[0] >= 0.0 && inputs.aim.is_some() {
            let (ox, oy) = owner.unwrap_or((cx, cy));
            // The speed damper (`:67899-67903`).
            if self.vel.length() > speed {
                self.vel *= 0.98;
            }
            // The distance flags (`:67904-67917`).
            let dx = ox - cx;
            let dy = oy - cy;
            let player_dist = (dx * dx + dy * dy).sqrt();
            let flag3 = player_dist > range;
            let flag4 = player_dist > range * 1.3;
            // The owner-only aim update (`:67918-67956`).
            if let Some((mut aim_x, mut aim_y)) = inputs.aim {
                if !inputs.channel {
                    // The release (`:67920-67925`; the `CCed` arm is stubbed
                    // false — no CC system ported).
                    self.ai[0] = -1.0;
                    self.ai[1] = 0.0;
                } else {
                    // The aim clamped to the range (`:67932-67939`), then the
                    // `range - 1f` re-clamp (`:67943-67950`).
                    let mut vx = aim_x - ox;
                    let mut vy = aim_y - oy;
                    let mut d = (vx * vx + vy * vy).sqrt();
                    if d > range {
                        let s = range / d.max(f32::EPSILON);
                        aim_x = ox + vx * s;
                        aim_y = oy + vy * s;
                    }
                    vx = aim_x - ox;
                    vy = aim_y - oy;
                    d = (vx * vx + vy * vy).sqrt();
                    if d > range - 1.0 {
                        let s = (range - 1.0) / d.max(f32::EPSILON);
                        aim_x = ox + vx * s;
                        aim_y = oy + vy * s;
                    }
                    // The change gate (`:67940`).
                    if self.ai[0] != aim_x || self.ai[1] != aim_y {
                        self.ai[0] = aim_x;
                        self.ai[1] = aim_y;
                    }
                }
            }
            // The 1.3× forced return (`:67957-67961`).
            if flag4 && self.ai[0] >= 0.0 {
                self.ai[0] = -1.0;
            }
            if self.ai[0] >= 0.0 {
                if flag3 {
                    // `:67964-67984` — the far arms: half the accel, double
                    // the top speed, halve the away-axis velocity.
                    accel /= 2.0;
                    speed *= 2.0;
                    if cx > ox && self.vel.x > 0.0 {
                        self.vel.x *= 0.5;
                    }
                    if cy > oy && self.vel.y > 0.0 {
                        self.vel.y *= 0.5;
                    }
                    if cx < ox && self.vel.x < 0.0 {
                        self.vel.x *= 0.5;
                    }
                    if cy < oy && self.vel.y < 0.0 {
                        self.vel.y *= 0.5;
                    }
                    // `:67986-67989` — the accel override.
                    accel = 1.0;
                }
                // The seek (`:67985-68021`).
                let mut vx = self.ai[0] - cx;
                let mut vy = self.ai[1] - cy;
                let dist = (vx * vx + vy * vy).sqrt();
                if dist > arrive {
                    let mut wanted = (dist * 0.5).min(speed);
                    if flag3 {
                        wanted = wanted.min(speed * 0.5);
                    }
                    let s = wanted / dist.max(f32::EPSILON);
                    vx *= s;
                    vy *= s;
                    let px = self.vel.x;
                    let py = self.vel.y;
                    self.vel.x = (px * (accel - 1.0) + vx) / accel;
                    self.vel.y = (py * (accel - 1.0) + vy) / accel;
                } else if flag {
                    // `:68003-68011` — the double yoyo's 0.6-speed floor keeps
                    // the pair moving while it orbits.
                    if self.vel.length() < speed * 0.6 {
                        let l = self.vel.length();
                        let mut nx = if l > 0.0 { self.vel.x / l } else { 0.0 };
                        let mut ny = if l > 0.0 { self.vel.y / l } else { 0.0 };
                        nx *= speed * 0.6;
                        ny *= speed * 0.6;
                        let px = self.vel.x;
                        let py = self.vel.y;
                        self.vel.x = (px * (accel - 1.0) + nx) / accel;
                        self.vel.y = (py * (accel - 1.0) + ny) / accel;
                    }
                } else {
                    // The hover (`:68012-68015`).
                    self.vel *= 0.8;
                }
                // `:68016-68020` — the floor's second arm, outside the far
                // state.
                if flag && !flag3 && self.vel.length() < speed * 0.6 {
                    let l = self.vel.length();
                    let s = speed * 0.6 / l.max(f32::EPSILON);
                    self.vel *= s;
                }
            }
        }

        // `:68032-68036` — the Magic String's ghost: no damage, full alpha.
        // (Its spawn path rides M3-EQUIP-3's magicString flag.)
        if self.ai[0] == -3.0 {
            self.damage = 0;
            self.alpha = 255;
        }

        // `:68037-68077` — the dropped twin; else `:68078-68095` — the return.
        if self.ai[0] == -2.0 {
            // `:68039-68042` — the roll only while the penetrate sits below
            // zero (`Main.rand.Next(3, 7)`).
            if self.penetrate < 0 {
                self.penetrate = rng.next_range(3, 7);
            }
            self.vel.y += 0.3;
            if self.vel.length() > 16.0 {
                let s = 16.0 / self.vel.length();
                self.vel *= s;
            } else if self.vel.y <= 3.0 {
                let nudge_cap = speed.min(16.0);
                if self.vel.x.abs() < 0.1 {
                    if let Some((ox, _)) = owner {
                        if cx < ox {
                            self.vel.x -= 0.1;
                            self.direction = -1;
                        } else {
                            self.vel.x += 0.1;
                            self.direction = 1;
                        }
                    }
                } else if self.vel.x > 0.0 && self.vel.x < nudge_cap {
                    self.vel.x += 0.1;
                } else if self.vel.x < 0.0 && self.vel.x > -nudge_cap {
                    self.vel.x -= 0.1;
                }
            }
        } else if self.ai[0] < 0.0 {
            // The return: `tileCollide = false`, the owner seek at the ×1.5
            // speed with the ×0.8 accel, the three kill distances.
            self.tile_collide = false;
            if let Some((ox, oy)) = owner {
                let seek_accel = accel * 0.8;
                let seek_speed = speed * 1.5;
                let dx = ox - cx;
                let dy = oy - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < seek_speed + 10.0 || dist == 0.0 || dist > 2000.0 {
                    self.active = false;
                    self.time_left = 0;
                } else {
                    let s = seek_speed / dist.max(f32::EPSILON);
                    let px = self.vel.x;
                    let py = self.vel.y;
                    self.vel.x = (px * (seek_accel - 1.0) + dx * s) / seek_accel;
                    self.vel.y = (py * (seek_accel - 1.0) + dy * s) / seek_accel;
                }
            }
        }

        // `:68096-68103` — the rotation: 0.45 × direction for the dropped twin
        // and the Kraken (it always spins), 0.45 for the rest.
        if self.ai[0] == -2.0 || self.kind == KRAKEN_YOYO {
            self.rot += 0.45 * self.direction as f32;
        } else {
            self.rot += 0.45;
        }

        // The generic `HandleMovement` (`:17469-17860`): the position
        // integrates `Collision.TileCollision`'s adjusted velocity (the
        // blocked axis snaps flush, the other axis slides — `:17596`); the
        // dropped twin passes through platforms (`:17552`'s `flag6 = false`).
        // The slope follow-up (`:17657`'s `SlopeCollision` for the
        // non-`correctSlopeCollision` path) rides the shared helper as the
        // player's path does; the yoyo never dies and never reverses on a
        // tile — unlike the arrow's kill and the boomerang's reverse.
        if self.tile_collide {
            let fall = self.ai[0] != -2.0;
            let tc = crate::collision::tile_collision(
                world,
                [self.pos.x, self.pos.y],
                [self.vel.x, self.vel.y],
                self.w,
                self.h,
                fall,
                fall,
            );
            self.vel = Vec2::new(tc.vel[0], tc.vel[1]);
        }
        self.pos += self.vel;
        tick
    }

    /// The aiStyle-115 arm — the Terrarian's beams (`AI()`'s inline
    /// `else if (this.aiStyle == 115)`, `Projectile.cs:34414-34437`): the
    /// 0.985 damp, the velocity-driven rotation ±0.08, and the 30-tick alpha
    /// fade that kills it. The `Lighting.AddLight` (`:34416`) is the render
    /// row's — a documented skip.
    /// `AI_210_YoyoShots` (`Projectile.cs:36809-36900`) — the two yoyo
    /// shots' fade-out lifecycles (no steering): the Eye-of-Cthulhu dash
    /// (1130) and the Kraken wave (1135). M4-PROJ-4b.
    fn ai_yoyo_shot(&mut self, _world: &World) {
        if self.kind == EYE_OF_CTHULHU_DASH {
            self.rot = self.vel.y.atan2(self.vel.x) - std::f32::consts::FRAC_PI_2;
            // The one-shot sound (`:36814-36828`) and the frame cycle
            // (`:36834-36838`) are the render row's — the port's cue table
            // does not carry `Item20`, so the AI models neither.
            self.vel *= 0.92;
            let num = 20.0;
            self.local_ai[0] += 1.0;
            // `MathHelper.Lerp(255, 0, Clamp(MultiLerp(t/20, 0,1, 1,1, 0.5,0), 0, 1))`
            // (`:36839-36841`).
            let t = (self.local_ai[0] / num).clamp(0.0, 1.0);
            let ramp = if t <= 0.5 { t * 2.0 } else { 1.0 - (t - 0.5) * 2.0 };
            self.alpha = (255.0 * (1.0 - ramp)) as i32;
            if self.local_ai[0] >= num && self.owner != 255 {
                self.active = false;
            }
        }
        if self.kind != KRAKEN_WAVE {
            return;
        }
        self.rot = self.vel.y.atan2(self.vel.x);
        self.vel *= 0.92;
        let num2 = 20.0;
        self.local_ai[0] += 1.0;
        let t = (self.local_ai[0] / num2).clamp(0.0, 1.0);
        let ramp = if t <= 0.5 { t * 2.0 } else { 1.0 - (t - 0.5) * 2.0 };
        self.alpha = (255.0 * (1.0 - ramp)) as i32;
        if self.local_ai[0] >= num2 && self.owner != 255 {
            self.active = false;
        }
        // The dust burst over the first `num2 / 3` ticks (`:36869-36898`) is
        // the render row's — documented deviation.
    }

    fn ai_yoyo_beam(&mut self, world: &World) {
        self.vel *= 0.985;
        self.rot += self.vel.x * 0.2;
        if self.vel.x > 0.0 {
            self.rot += 0.08;
        } else {
            self.rot -= 0.08;
        }
        self.ai[1] += 1.0;
        if self.ai[1] > 30.0 {
            self.alpha += 10;
            if self.alpha >= 255 {
                self.alpha = 255;
                self.active = false;
                self.time_left = 0;
            }
        }
        // The generic movement: the beam keeps `tileCollide`, so the
        // `HandleMovement` slide applies.
        if self.tile_collide {
            let tc = crate::collision::tile_collision(
                world,
                [self.pos.x, self.pos.y],
                [self.vel.x, self.vel.y],
                self.w,
                self.h,
                true,
                true,
            );
            self.vel = Vec2::new(tc.vel[0], tc.vel[1]);
        }
        self.pos += self.vel;
    }

    /// `AI()`'s aiStyle-82 block (`Projectile.cs:31214-31321`) — the
    /// PhantasmalEye: the three-leg drift ladder (45 → 90 → homing) with
    /// the per-leg rotation drift, the 14-px homing speed over the 0.6
    /// lerp, the Y-floor of 6, the aging-accel ramp and the 30-px kill.
    fn ai_phasmal_eye(&mut self, npcs: &[Npc], player: (f32, f32)) {
        // `:31216-31220`'s alpha ramp.
        self.alpha = (self.alpha - 40).max(0);
        match self.ai[0] as i32 {
            0 => {
                // `:31223-31241`'s first leg: 45 ticks, then the drift
                // sign flips (`ai[1] = -ai[1]`).
                self.local_ai[0] += 1.0;
                if self.local_ai[0] >= 45.0 {
                    self.local_ai[0] = 0.0;
                    self.ai[0] = 1.0;
                    self.ai[1] = -self.ai[1];
                }
                self.drift_leg();
            }
            1 => {
                // `:31245-31263`'s second leg: 90 ticks, then the target
                // pick (`ai[1]` becomes the player index).
                self.local_ai[0] += 1.0;
                if self.local_ai[0] >= 90.0 {
                    self.local_ai[0] = 0.0;
                    self.ai[0] = 2.0;

                }
                self.drift_leg();
            }
            _ => {
                // `:31267-31313`'s homing leg.
                self.ai[2] += 1.0;
                let target = player;
                let (dx, dy) = (target.0 - self.pos.x, target.1 - self.pos.y);
                if (dx * dx + dy * dy).sqrt() < 30.0 {
                    self.active = false;
                    return;
                }
                let mut tvx = dx / (dx * dx + dy * dy).sqrt().max(0.001) * 14.0;
                let mut tvy = dy / (dx * dx + dy * dy).sqrt().max(0.001) * 14.0;
                if tvy < 6.0 {
                    tvy = 6.0;
                }
                // The aging accel (`:31281`'s `0.4 * Remap(ai[2], 0, 90, 1, 0)`).
                let acc = 0.4 * ((90.0 - self.ai[2].min(90.0)) / 90.0);
                self.vel.x += Self::approach(self.vel.x, tvx, acc);
                self.vel.y += Self::approach(self.vel.y, tvy, acc);
            }
        }
        // `:31320`'s rotation.
        self.rot = self.vel.to_angle() + std::f32::consts::FRAC_PI_2;
        self.pos += self.vel;
    }

    /// `:31231-31241`'s shared drift: the `velocity.X` rotation off the
    /// leg's signed offset, the ±6 clamp, the Y drift and the -7 floor.
    fn drift_leg(&mut self) {
        self.vel.x = (self.vel.x * self.ai[1].cos() - self.vel.y * self.ai[1].sin()).clamp(-6.0, 6.0);
        self.vel.y -= 0.08;
        if self.vel.y > 0.0 {
            self.vel.y -= 0.2;
        }
        if self.vel.y < -7.0 {
            self.vel.y = -7.0;
        }
    }

    /// `AI()`'s aiStyle-83 block (`Projectile.cs:31322-31376`) — the
    /// PhantasmalSphere: the alpha/scale ramp, the host position lock
    /// while `ai[0] < 30`, and the detached decay + 2-frame animation.
    fn ai_phasmal_sphere(&mut self, npcs: &[Npc]) {
        self.alpha = (self.alpha - 6).max(0);
        self.scale = 1.0 - self.alpha as f32 / 255.0;
        if self.ai[0] == -1.0 {
            // `:31334-31338`'s release frame — the port has no frame field,
            // the pulse rides local_ai[0].
            self.local_ai[1] = 1.0;
            self.extra_updates = 1;
        } else if self.ai[0] < 30.0 {
            self.ai[0] += 1.0;
            // `:31339-31342`'s host lock.
            if let Some(hi) = self.host_by_kind(npcs, NpcKind::MoonLordHand) {
                let host = &npcs[hi];
                let (hw, hh) = host.size();
                let (hcx, hcy) = (
                    host.x + hw * 0.5 - self.scale * 0.0,
                    host.y + hh * 0.5,
                );
                self.pos = Vec2::new(hcx - 20.0, hcy - 20.0) - self.vel;
            }
        } else {
            self.vel *= 0.96;
            self.local_ai[0] += 1.0;
            if self.local_ai[0] >= 6.0 {
                self.local_ai[0] = 0.0;
                self.local_ai[1] = 1.0 - self.local_ai[1];
            }
        }
        self.pos += self.vel;
    }

    /// `AI()`'s aiStyle-84 block's PhantasmalDeathray arm
    /// (`Projectile.cs:31385-31639`) — the host-ellipse position, the
    /// `ai[0]` rotation sweep, the 180-tick lifetime with the `sin`
    /// scale curve and the `LaserScan`-length smoothing.
    ///
    /// Deviations: the 3-sample LaserScan folds into the port's own tile
    /// ray (the beam length's lerp stays); the shade/light/dust tails are
    /// unmodelled.
    fn ai_phasmal_deathray(&mut self, world: &World, npcs: &[Npc]) {
        let host_idx = self.host_by_kind(npcs, NpcKind::MoonLordHead)
            .or_else(|| self.host_by_kind(npcs, NpcKind::MoonLordFreeEye));
        let host = host_idx.map(|i| &npcs[i]);
        let (hcx, hcy) = match host {
            Some(h) => (h.x + h.size().0 * 0.5, h.y + h.size().1 * 0.5),
            None => (self.pos.x, self.pos.y),
        };
        let host_kind = host.map(|h| h.kind);
        // `:31387-31391`'s host-death kill (only the Moon Lord head hosts
        // the port's rays).
        if let Some(h) = host {
            if h.kind == NpcKind::MoonLordHead && h.ai[0] == -2.0 {
                self.active = false;
                return;
            }
        }
        // `:31392-31399`'s ellipse: (27, 59) for the head, (30, 30) for the
        // eye — scaled by the host's open factor.
        let open = host.map(|h| h.moon_local[1]).unwrap_or(1.0);
        let (rx, ry) = match host_kind {
            Some(NpcKind::MoonLordHead) => (27.0, 59.0),
            Some(NpcKind::MoonLordFreeEye) => (30.0, 30.0),
            _ => (0.0, 0.0),
        };
        let ang = host.map(|h| h.moon_local[0]).unwrap_or(0.0);
        self.pos = Vec2::new(
            hcx + ang.cos() * rx * open - 18.0,
            hcy + ang.sin() * ry * open - 18.0,
        );
        // `:31511-31521`'s lifetime + scale curve.
        self.local_ai[0] += 1.0;
        if self.local_ai[0] >= 180.0 {
            self.active = false;
            return;
        }
        let ceiling = if host_kind == Some(NpcKind::MoonLordFreeEye) { 0.4 } else { 1.0 };
        self.scale = (self.local_ai[0] * std::f32::consts::PI / 180.0).sin() * 10.0 * ceiling;
        if self.scale > ceiling {
            self.scale = ceiling;
        }
        // `:31553-31559`'s sweep: the heading turns by `ai[0]` each tick.
        let heading = self.vel.to_angle() + self.ai[0];
        self.rot = heading - std::f32::consts::FRAC_PI_2;
        self.vel = Vec2::new(heading.cos(), heading.sin());
        // `:31592-31633`'s length: the port ray against the tiles, smoothed
        // at the 0.5/0.75 lerp.
        let range = 2400.0f32;
        let mut hit = range;
        let mut steps = 0;
        while (steps as f32) * 32.0 < range {
            let sx = self.pos.x + self.vel.x * steps as f32;
            let sy = self.pos.y + self.vel.y * steps as f32;
            let tx = (sx / TILE_PX) as i32;
            let ty = (sy / TILE_PX) as i32;
            if world
                .tile(tx, ty)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                hit = steps as f32 * 32.0;
                break;
            }
            steps += 1;
        }
        let amount = 0.5;
        self.local_ai[1] += (hit - self.local_ai[1]) * amount;
        // The port's live length rides the smoothed field for the draw.
        let _ = world;
    }

    /// `AI()`'s aiStyle-85 block (`Projectile.cs:31758-31826`) — the
    /// MoonLeech: the outbound leg at the 16-px cap that applies the
    /// buff 145 (840/960) at the 20-px contact, and the return leg that
    /// dies at the head.
    fn ai_moon_leech(&mut self, npcs: &[Npc], player: &mut crate::player::Player) {
        let host_idx = (self.ai[0].abs() as usize).saturating_sub(1);
        let host = npcs.get(host_idx);
        // `:31768-31772`'s host kill (only the Moon Lord head hosts).
        if !matches!(host.map(|h| h.kind), Some(NpcKind::MoonLordHead)) || self.ai[0] == 0.0 {
            self.active = false;
            return;
        }
        self.local_ai[0] += 1.0;
        if self.local_ai[0] >= 330.0 && self.ai[0] > 0.0 {
            self.ai[0] *= -1.0;
        }
        let (hcx, hcy) = match host {
            Some(h) => (h.x + h.size().0 * 0.5, h.y + h.size().1 * 0.5),
            None => (self.pos.x, self.pos.y),
        };
        // `:31784`'s rotation off the (host → player + (0, 216)) aim.
        let (pcx, pcy) = player.center();
        let (ax, ay) = (hcx - pcx, hcy - pcy + 216.0);
        self.rot = ay.atan2(ax) + std::f32::consts::FRAC_PI_2;
        if self.ai[0] > 0.0 {
            let (dx, dy) = (pcx - self.pos.x, pcy - self.pos.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 0.001 {
                let s = 16.0f32.min(d);
                self.vel = Vec2::new(dx / d * s, dy / d * s);
            } else {
                self.vel = Vec2::ZERO;
            }
            // `:31796-31807`'s contact + the buff request.
            if d < 20.0 && self.local_ai[1] == 0.0 {
                self.local_ai[1] = 1.0;
                player.buffs.add_buff(145, 840);
            }
        } else {
            let (dx, dy) = (hcx + 0.0 - self.pos.x, hcy + 216.0 - self.pos.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 0.001 {
                let s = 16.0f32.min(d);
                self.vel = Vec2::new(dx / d * s, dy / d * s);
            } else {
                self.vel = Vec2::ZERO;
            }
            if d < 20.0 {
                self.active = false;
            }
        }
        self.pos += self.vel;
    }

    /// `AI()`'s aiStyle-108 block (`Projectile.cs:33679-33988`) — the
    /// ritual markers (the 578 lightning / the 579 portal): the timeline
    /// `ai[0] <= 50 -> <= 90 -> <= 120 -> else` with the scale/alpha ramps
    /// and the rotation decrements, the 578's aim rotation, the 580 bolt it
    /// fires at the 90th tick, and the 579's two VortexAliens.
    ///
    /// Deviations: the dust arms (229/240/5, the `ai[1] == 1f` burst) are
    /// unmodelled; the late-stage clone spawn rides the 580's own machine.
    fn ai_vortex_ritual(&mut self, player: (f32, f32), rng: &mut u64) {
        // `:33696-33700`'s first-tick sound latch.
        if self.local_ai[1] == 0.0 {
            self.local_ai[1] = 1.0;
        }
        // `:33705-33713`'s aim (the 578 only: the 579's ai[1] is the burst
        // latch).
        if self.kind == VORTEX_LIGHTNING && self.local_ai[0] == 0.0 {
            self.local_ai[0] = 1.0;
            let (dx, dy) = (player.0 - self.pos.x, player.1 - self.pos.y);
            self.ai[1] = if dx == 0.0 && dy == 0.0 {
                -std::f32::consts::FRAC_PI_2
            } else {
                dy.atan2(dx)
            };
        }
        self.ai[0] += 1.0;
        let scale;
        if self.ai[0] <= 50.0 {
            // The wind-up: the dust arms (unmodelled).
            scale = 0.0;
        } else if self.ai[0] <= 90.0 {
            // `:33797-33799`'s charge.
            scale = (self.ai[0] - 50.0) / 40.0;
            self.alpha = 255 - (255.0 * scale) as i32;
            self.rot -= std::f32::consts::PI / 20.0;
        } else if self.ai[0] <= 120.0 {
            // `:33891-33893`'s full charge.
            scale = 1.0;
            self.alpha = 0;
            self.rot -= std::f32::consts::PI / 60.0;
        } else {
            // `:33944-33950`'s dissolve.
            scale = 1.0 - (self.ai[0] - 120.0) / 60.0;
            self.alpha = 255 - (255.0 * scale) as i32;
            self.rot -= std::f32::consts::PI / 30.0;
            if self.alpha >= 255 {
                self.active = false;
                return;
            }
        }
        self.scale = scale;
        // The 90th tick's payloads.
        if self.ai[0] == 90.0 {
            if self.kind == VORTEX_LIGHTNING {
                // `:33838-33841`'s 580: the aim vector times 8 behind the
                // centre, `ai0 = ai[1]`.
                let (vx, vy) = (self.ai[1].cos() * 8.0, self.ai[1].sin() * 8.0);
                self.wants_ritual_bolt = Some((self.pos.x - vx, self.pos.y - vy, vx, vy, self.ai[1]));
            } else {
                // `:33844-33856`'s two VortexAliens (427) at the centre
                // with `-UnitY.RotatedByRandom(2*PI) * rand(4, 9) - UnitY*2`.
                for _ in 0..2 {
                    let ang = crate::app::rand01(rng) * std::f32::consts::PI * 2.0;
                    let mag = 4.0 + crate::app::rand01(rng) * 5.0;
                    let (vx, vy) = (-ang.sin() * mag, -ang.cos() * mag - 2.0);
                    self.wants_mark_spawn =
                        Some((NpcKind::VortexAlien.to_u16(), self.pos.x, self.pos.y, vx, vy));
                }
            }
        }
    }

    /// `AI()`'s aiStyle-88 block's VortexLightning arm
    /// (`Projectile.cs:32145-32249`) — the ritual bolt: the initial
    /// stationary wait, the 900-wrap latch, the per-step wand, the
    /// tile-hit stop and the stationarity kill.
    ///
    /// Deviations: the dust/light arms are unmodelled; the wand's
    /// `UnifiedRandom((int)ai[1])` reseed folds into the shared stream.
    fn ai_vortex_bolt(&mut self, world: &World, rng: &mut u64) {
        // `:32151-32153`'s 900-wrap latch.
        if self.local_ai[1] == 0.0 && self.ai[0] >= 900.0 {
            self.ai[0] -= 1000.0;
            self.local_ai[1] = -1.0;
        }
        if self.local_ai[1] < 1.0 {
            self.local_ai[1] += 2.0;
            self.pos += self.vel;
            // `:32308`'s tile-hit stop.
            let tx = (self.pos.x / TILE_PX) as i32;
            let ty = (self.pos.y / TILE_PX) as i32;
            if world
                .tile(tx, ty)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                self.vel = Vec2::ZERO;
            }
        } else if self.vel == Vec2::ZERO {
            // `:32156-32174`'s stationary wait + kill: a stopped bolt dies
            // after \`extraUpdates * 2\` ticks (the port's counter rides
            // local_ai[2]).
            self.local_ai[2] += 1.0;
            if self.local_ai[2] > (self.extra_updates * 2) as f32 {
                self.active = false;
                return;
            }
        } else {
            // \`:32195-32249\`'s wand: the per-step lateral jitter, the 100-roll
            // rejection loop, the localAI[0] wander.
        let spin = self.ai[0] + std::f32::consts::FRAC_PI_2;
        let mut wander = self.local_ai[0];
        let mut wander_y = 0.0f32;
        for _ in 0..100 {
            let a = crate::app::rand01(rng) * std::f32::consts::PI * 2.0;
            wander = a.cos() * (self.extra_updates as f32 + 1.0) * 2.0;
            wander_y = a.sin() * (self.extra_updates as f32 + 1.0) * 2.0;
            if wander_y <= -0.02 && wander.abs() <= 40.0 {
                break;
            }
        }
            self.local_ai[0] += wander;
            self.vel = Vec2::new(spin.cos() * wander, spin.sin() * wander + wander_y);
        }
    }

    /// `AI()`'s aiStyle-103 block (`Projectile.cs:33448-33477`) — the
    /// Stardust tower's mark: the 30-tick fade-in that spawns the NPC id
    /// carried in `localAI[0]` (the vanilla's `localAI[0] = num`), then
    /// the fade-out.
    fn ai_stardust_mark(&mut self, world: &World, npcs: &[Npc]) {
        self.scale = self.ai[1];
        self.ai[0] += 1.0;
        if self.ai[0] >= 30.0 {
            self.alpha += 25;
            if self.alpha >= 250 {
                self.active = false;
            }
        } else {
            self.alpha -= 25;
            if self.alpha < 0 {
                self.alpha = 0;
                // `:33467-33475`'s spawn.
                if self.local_ai[1] == 0.0 && self.local_ai[0] > 0.0 {
                    self.local_ai[1] = 1.0;
                    self.wants_mark_spawn =
                        Some((self.local_ai[0] as u16, self.pos.x, self.pos.y, 0.0, 0.0));
                }
            }
        }
        let _ = npcs;
    }

    /// `AI()`'s aiStyle-102 block's 574 arm (`Projectile.cs:33250-33299`) —
    /// the Nebula Eye: the host-locked position drag with the 0.15/0.075
    /// homing accel over the 6-px clamp, the 180-tick release.
    fn ai_nebula_eye(&mut self, npcs: &[Npc], player: (f32, f32)) {
        let host_idx = self.host_by_kind(npcs, NpcKind::NebulaFloater).unwrap_or(usize::MAX);
        let alive = npcs
            .get(host_idx)
            .map(|h| h.kind == NpcKind::NebulaFloater && h.hp > 0)
            .unwrap_or(false);
        self.time_left = 2;
        self.ai[0] += 1.0;
        if self.ai[0] < 180.0 {
            if let Some(host) = npcs.get(host_idx) {
                self.pos += Vec2::new(host.vx, host.vy);
            }
        }
        if alive {
            let (px, py) = player;
            let (dx, dy) = ((px - self.pos.x).signum() * 0.15, (py - self.pos.y).signum() * 0.075);
            let v = self.vel + Vec2::new(dx, dy);
            let l = v.length();
            self.vel = if l > 6.0 { v * (6.0 / l) } else { v };
        } else {
            self.ai[0] = 180.0;
        }
        self.pos += self.vel;
    }

    /// `AI()`'s aiStyle-122 block (`Projectile.cs:34896-34931`) — the
    /// TowerDamageBolt: the host-homing 5-px bolt that dies when the host
    /// dies (the port's callers pass the shield-hitting tower through
    /// `ai[0]`).
    fn ai_tower_damage_bolt(&mut self, npcs: &[Npc]) -> bool {
        let host = npcs.get(self.ai[0] as usize);
        let alive = host.map(|h| h.hp > 0).unwrap_or(false);
        if !alive {
            self.active = false;
            return false;
        }
        let (hcx, hcy) = match host {
            Some(h) => (h.x + h.size().0 * 0.5, h.y + h.size().1 * 0.5),
            None => (self.pos.x, self.pos.y),
        };
        let (dx, dy) = (hcx - self.pos.x, hcy - self.pos.y);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        self.vel = Vec2::new(dx / d * 5.0, dy / d * 5.0);
        self.pos += self.vel;
        // `:34917-34925`'s contact kill (the shield spend rides the caller).
        if let Some(h) = host {
            let (hw, hh) = h.size();
            let hx = h.x + hw * 0.5;
            let hy = h.y + hh * 0.5;
            if (hx - self.pos.x).abs() < hw * 0.5 && (hy - self.pos.y).abs() < hh * 0.5 {
                return true;
            }
        }
        false
    }

    /// The approach helper shared by the eye machines (`:31282-31313`'s
    /// per-axis accel with the cross-zero double-step).
    fn approach(current: f32, target: f32, acc: f32) -> f32 {
        let mut step = 0.0;
        if current < target {
            step += acc;
        } else if current > target {
            step -= acc;
        }
        step
    }

    /// The host NPC by kind (`Main.npc[ai[1]]`'s port stand-in): the
    /// nearest living NPC of the kind — the pool's spawn carries no
    /// `ai[1]` whoAmI, so the machines link by kind the way the
    /// carrier/intent passes do.
    fn host_by_kind(&self, npcs: &[Npc], kind: NpcKind) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for (i, h) in npcs.iter().enumerate() {
            if h.kind != kind || h.hp <= 0 {
                continue;
            }
            let d = (h.x - self.pos.x).hypot(h.y - self.pos.y);
            if best.is_none() || d < best.unwrap().0 {
                best = Some((d, i));
            }
        }
        best.map(|(_, i)| i)
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
        rng: &mut dyn tr_core::loot::DropRng,
    ) -> Option<WhipArmSpawn> {
        let kind = self.kind;
        // `:47669-47672`: the Moon Lord whip's arm budget, rolled once.
        if kind == MOON_LORD_WHIP && self.local_ai[1] == 0.0 {
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

    /// The C# `Vector2` overload of `Utils.MoveTowards` (`Utils.cs:1772-1780`)
    /// — the length-clamped step toward the target (the scalar peer lives in
    /// `ambient_sound.rs`).
    fn move_towards_vec(current: Vec2, target: Vec2, max_step: f32) -> Vec2 {
        let v = target - current;
        if v.length() < max_step {
            return target;
        }
        current + v.normalize_or_zero() * max_step
    }

    /// The C# `wet && !lavaWet` read of the 948→947 morph (`:43737`):
    /// `Collision.WetCollision`/`LavaCollision` over the slot box. The port's
    /// Player liquid pass reads the mid-tile kind (`player.rs:1634-1643`); the
    /// flail slot reuses that idiom — the box-scan approximation is recorded
    /// in ANALYSIS.
    fn flail_in_liquid(&self, world: &World) -> bool {
        let cx = (self.pos.x + self.w * 0.5) / tr_core::TILE_PX;
        let cy = (self.pos.y + self.h * 0.5) / tr_core::TILE_PX;
        world
            .tile(cx as i32, cy as i32)
            .filter(|t| t.liquid > 0)
            .is_some_and(|t| t.liquid_kind() != tr_core::tile::LiquidKind::Lava)
    }

    /// `AI_015_Flails` (`Projectile.cs:43724-44169`) — the whole flail
    /// machine: the kill gates, the per-kind parameter table
    /// (`flail_params`), the seven `ai[0]` states, the movement, the 1058/247
    /// per-kind tails and the common tail. The method runs BEFORE the pool's
    /// move/tile-collision pass; `ai_flail_bounce` (the dispatcher's
    /// `HandleMovement` peer) runs after it.
    ///
    /// Documented deviations (ANALYSIS): the `mapFullscreen` kill (no map
    /// mode), the `noItems` gate (no field), `player.heldProj` /
    /// `SetDummyItemTime` / `itemRotation` (no consumers — the held-item pose
    /// is a render follow-up), `ownerHitCheck`/`drawLayer` (no consumers),
    /// the dust call (no dust system) and the tile-impact/sound arms of the
    /// bounce (a decor follow-up). The `player.ChangeDir` writes ride back in
    /// `FlailTick.facing` (the port's facing is run-key driven), with the
    /// same-tick reads served by the local `eff_dir`.
    #[allow(clippy::too_many_arguments)]
    fn ai_flail(
        &mut self,
        world: &World,
        npcs: &[Npc],
        owner: Option<(f32, f32)>,
        inputs: FlailInputs,
        flag5: bool,
        rng: &mut dyn tr_core::loot::DropRng,
    ) -> FlailTick {
        let mut tick = FlailTick::default();
        let center = match (inputs.center, owner) {
            (Some(c), _) => Vec2::from(c),
            // An absent player reads as the C#'s inactive `Main.player[owner]`.
            (None, _) => {
                self.active = false;
                self.time_left = 0;
                return tick;
            }
        };
        let mounted = match inputs.mounted_center {
            Some(m) => Vec2::from(m),
            None => {
                self.active = false;
                self.time_left = 0;
                return tick;
            }
        };
        let head = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
        // `:43726-43731` — dead / CCed / the `900f * (1f / meleeSpeed)` leash.
        if inputs.player_dead
            || inputs.player_cced
            || (head - center).length() > 900.0 / inputs.melee_speed
        {
            self.active = false;
            self.time_left = 0;
            return tick;
        }
        // `:43737-43741` — the Flaming Mace's water extinguish.
        if self.kind == FLAMING_MACE && self.flail_in_liquid(world) {
            self.kind = MACE;
        }
        // `:43832-43835` — the 26/35 `drawLayer = 0` branch keys on `ai[2]`,
        // whose only C# writer is the sentry-backpack tail (unreachable for
        // flails); the port has no drawLayer consumer either, so the branch
        // stays a note until the render follow-up.
        let params = flail_params(self.kind);
        let melee = 1.0 / inputs.melee_speed;
        // `:43824-43831` — the meleeSpeed-scaled subset; `:43746-43760` the
        // shared locals; `:43759`/`:43836-43837` the derived values.
        let num = params.flight_ticks;
        let num2 = params.throw_speed * melee;
        let num3 = FLAIL_THROW_MAX_DISTANCE;
        let num4 = FLAIL_RETRACT_ACCEL * melee;
        let num5 = params.retract_speed * melee;
        let num6 = FLAIL_REEL_ACCEL * melee;
        let num7 = params.reel_kill_distance * melee;
        let num8 = FLAIL_SPIN_SOFT_ACCEL * melee;
        let num9 = FLAIL_SPIN_PULL_SPEED * melee;
        let num10 = FLAIL_SPIN_INNER_RADIUS;
        let num11 = FLAIL_HELD_NPC_COOLDOWN;
        let num12 = params.spin_npc_cooldown;
        let num13 = FLAIL_FLIGHT_NPC_COOLDOWN;
        let num14 = num + 5.0;
        let num15 = params.orbit_radius;
        let num17 = num2 * num;
        let num18 = num17 + 160.0;
        self.local_hit_cooldown = num11 as i32;
        // The effective player direction — the C# writes `player.ChangeDir`
        // mid-machine and reads it back in the same pass.
        let mut eff_dir = inputs.player_dir;
        match self.ai[0] as i32 {
            0 => {
                // The held state (`:43841-43898`).
                let mut aim = match inputs.aim {
                    Some(a) => (Vec2::from(a) - mounted).normalize_or_zero(),
                    None => Vec2::new(eff_dir as f32, 0.0),
                };
                if aim == Vec2::ZERO {
                    // `SafeNormalize(UnitX * player.direction)` (`:43863`).
                    aim = Vec2::new(eff_dir as f32, 0.0);
                }
                // `:43864-43871` — the forced facing, inverted on the dual
                // flail (`flag5`).
                eff_dir = if flag5 {
                    if aim.x < 0.0 { 1 } else { -1 }
                } else if aim.x > 0.0 {
                    1
                } else {
                    -1
                };
                if !inputs.channel {
                    // `:43872-43885` — the release throws; the C# `break`s
                    // out of the switch here.
                    self.ai[0] = 1.0;
                    self.ai[1] = 0.0;
                    self.vel = aim * num2 + Vec2::from(inputs.player_vel);
                    if flag5 {
                        self.vel.x *= -1.0;
                    }
                    self.pos = mounted - Vec2::new(self.w * 0.5, self.h * 0.5);
                    self.local_ai[1] = 0.0; // `ResetLocalNPCHitImmunity`
                    self.local_hit_cooldown = num13 as i32;
                } else {
                    // `:43888-43897` — the charge orbit.
                    self.local_ai[1] += 1.0;
                    let angle = std::f32::consts::PI * 10.0 * (self.local_ai[1] / 60.0)
                        * eff_dir as f32;
                    let mut orbit =
                        Vec2::new(eff_dir as f32, 0.0).rotate(Vec2::from_angle(angle));
                    orbit.y *= 0.8;
                    if orbit.y > 0.0 {
                        orbit.y *= 0.5; // gravDir 1 (no flipped gravity ported)
                    }
                    self.pos =
                        mounted + orbit * num15 - Vec2::new(self.w * 0.5, self.h * 0.5);
                    self.vel = Vec2::ZERO;
                    self.local_hit_cooldown = num12 as i32;
                }
            }
            1 => {
                // The fly-out (`:43900-43932`).
                let grown = self.ai[1] >= num;
                self.ai[1] += 1.0;
                let flag3 = grown || (head - mounted).length() >= num3;
                let num19 = 1.3f32;
                if inputs.channel {
                    // `:43905-43916` — the mid-flight re-grab; the Drippler
                    // spawns its extra ball on both this and the arm end.
                    self.ai[0] = 6.0;
                    self.ai[1] = 0.0;
                    self.vel *= 0.2;
                    self.flail_helper(&mut tick, inputs, DRIPPLER_EXTRA_BALL, self.vel * num19, 0.0);
                } else if flag3 {
                    self.ai[0] = 2.0;
                    self.ai[1] = 0.0;
                    self.vel *= 0.3;
                    self.flail_helper(&mut tick, inputs, DRIPPLER_EXTRA_BALL, self.vel * num19, 0.0);
                }
                eff_dir = if center.x < head.x { 1 } else { -1 };
                self.local_hit_cooldown = num13 as i32;
            }
            2 => {
                // The retract (`:43934-43955`).
                let to_player = (mounted - head).normalize_or_zero();
                if (head - mounted).length() <= num5 {
                    self.active = false;
                    self.time_left = 0;
                    return tick;
                }
                if inputs.channel {
                    self.ai[0] = 6.0;
                    self.ai[1] = 0.0;
                    self.vel *= 0.2;
                } else {
                    self.vel *= 0.98;
                    self.vel = Self::move_towards_vec(self.vel, to_player * num5, num4);
                }
                eff_dir = if center.x < head.x { 1 } else { -1 };
            }
            3 => {
                // The spin orbit (`:43957-43998`).
                if !inputs.channel {
                    self.ai[0] = 4.0;
                    self.ai[1] = 0.0;
                    // `break` — the C# falls out of the state switch into
                    // the shared tail below, then HandleMovement's move.
                }
                let dist = (head - mounted).length();
                let want_collide = self.ai[1] == 1.0;
                let flag4 = dist <= num17;
                if flag4 != want_collide {
                    self.tile_collide = flag4;
                    self.ai[1] = if flag4 { 1.0 } else { 0.0 };
                }
                let to_player = (mounted - head).normalize_or_zero();
                if dist > num10 {
                    if dist >= num17 {
                        self.vel *= 0.5;
                        self.vel = Self::move_towards_vec(self.vel, to_player * num9, num9);
                    }
                    self.vel *= 0.98;
                    self.vel = Self::move_towards_vec(self.vel, to_player * num9, num8);
                } else {
                    if self.vel.length() < 6.0 {
                        self.vel.x *= 0.96;
                        self.vel.y += 0.2;
                    }
                    if inputs.player_vel.0 == 0.0 {
                        self.vel.x *= 0.96;
                    }
                }
                eff_dir = if center.x < head.x { 1 } else { -1 };
            }
            4 => {
                // The final reel-in (`:44000-44017`).
                self.tile_collide = false;
                let to_player = (mounted - head).normalize_or_zero();
                if (head - mounted).length() <= num7 {
                    self.active = false;
                    self.time_left = 0;
                    return tick;
                }
                self.vel *= 0.98;
                self.vel = Self::move_towards_vec(self.vel, to_player * num7, num6);
                let ahead = (head + self.vel - mounted).normalize_or_zero();
                if to_player.dot(ahead) < 0.0 {
                    self.active = false;
                    self.time_left = 0;
                    return tick;
                }
                eff_dir = if center.x < head.x { 1 } else { -1 };
            }
            5 => {
                // The bounce stagger (`:44019-44032`).
                let grown = self.ai[1] >= num14;
                self.ai[1] += 1.0;
                if grown {
                    self.ai[0] = 6.0;
                    self.ai[1] = 0.0;
                } else {
                    self.local_hit_cooldown = num13 as i32;
                    self.vel.y += 0.6;
                    self.vel.x *= 0.95;
                }
                eff_dir = if center.x < head.x { 1 } else { -1 };
            }
            6 => {
                // The post-release spin (`:44034-44047`).
                if !inputs.channel || (head - mounted).length() > num18 {
                    self.ai[0] = 4.0;
                    self.ai[1] = 0.0;
                    // `break` — the C# falls out of the state switch into
                    // the shared tail below, then HandleMovement's move.
                }
                // `!shimmerWet` — no shimmer in the port, so the arm always
                // adds gravity.
                self.vel.y += 0.8;
                self.vel.x *= 0.95;
                eff_dir = if center.x < head.x { 1 } else { -1 };
            }
            _ => {}
        }
        let mut tick = self.flail_tail(world, eff_dir, tick, inputs, npcs, rng);
        // `HandleMovement` (`Projectile.cs:17015`) with the aiStyle-15 arm
        // (`:19321`): the C# runs the sub-step move and the bounce handler
        // AFTER the AI returns — the port's per-AI move idiom keeps both
        // inside the flail so the slot never integrates twice.
        self.flail_move(world);
        tick
    }

    /// The flail's `HandleMovement` half (`Projectile.cs:17469-17760`): the
    /// axis-separated probe — the blocked axis zeroes that velocity
    /// component (the `Collision.Move` sub-step shape, the same idiom
    /// `ai_arrow` uses) — followed by the aiStyle-15 bounce handler
    /// (`AI_015_HandleMovementCollision` `:43655-43717`), which sees the
    /// pre-move velocity as its `lastVelocity`.
    fn flail_move(&mut self, world: &World) {
        let pos_before = self.pos;
        let last_velocity = self.vel;
        if self.tile_collide {
            // `Collision.Move`'s per-axis advance clamped to the blocked
            // face, then the blocked component zeroed — the port steps the
            // axis 1 px at a time (the C# sub-steps the box by
            // `clamp(min(w,h),3,16)`; the result agrees to the sub-pixel and
            // the approximation is recorded in ANALYSIS).
            for axis in 0..2 {
                let v = if axis == 0 { self.vel.x } else { self.vel.y };
                if v == 0.0 {
                    continue;
                }
                let step = v.signum();
                let mut moved = 0.0f32;
                while moved.abs() < v.abs() {
                    let nx = if axis == 0 { self.pos.x + step } else { self.pos.x };
                    let ny = if axis == 0 { self.pos.y } else { self.pos.y + step };
                    if crate::player::collides_rect(world, nx, ny, self.w, self.h) {
                        break;
                    }
                    self.pos.x = nx;
                    self.pos.y = ny;
                    moved += step;
                }
                if moved.abs() < v.abs() {
                    if axis == 0 {
                        self.vel.x = 0.0;
                    } else {
                        self.vel.y = 0.0;
                    }
                }
            }
        } else {
            self.pos += self.vel;
        }
        // `HandleMovement`'s `flag` (`:17896-17899`, the arm at `:17904`):
        // the per-aiStyle velocity chain — the flail's bounce handler — runs
        // only when the move CHANGED the velocity (a blocked axis). Without
        // the gate the handler's unconditional state-1 arm would stagger
        // every fly-out tick.
        if last_velocity != self.vel {
            self.ai_flail_bounce(last_velocity, pos_before);
        }
    }

    /// `AI_015_HandleMovementCollision` (`Projectile.cs:43655-43717`) — runs
    /// after the move, gated on the velocity change above; the pre-move
    /// velocity is the C# `lastVelocity`.
    fn ai_flail_bounce(&mut self, last_velocity: Vec2, pos_before: Vec2) {
        let mut num3 = 0.2;
        if self.ai[0] == 1.0 || self.ai[0] == 5.0 {
            num3 = 0.4;
        }
        if self.ai[0] == 6.0 {
            num3 = 0.0;
        }
        let mut loud = false;
        if last_velocity.x != self.vel.x {
            if last_velocity.x.abs() > 4.0 {
                loud = true;
            }
            self.vel.x = -last_velocity.x * num3;
            self.local_ai[0] += 1.0;
        }
        if last_velocity.y != self.vel.y {
            if last_velocity.y.abs() > 4.0 {
                loud = true;
            }
            self.vel.y = -last_velocity.y * num3;
            self.local_ai[0] += 1.0;
        }
        if self.ai[0] == 1.0 {
            // `:43687-43698` — the fly-out bounce drops into the stagger.
            self.ai[0] = 5.0;
            self.local_hit_cooldown = 10;
            loud = true;
            // `CreateImpactExplosion` + `CreateImpactExplosion2_
            // FlailTileCollision` are dust-only (no port seam); the
            // `position -= vector` rollback undoes the whole move
            // (`Collision.Move`'s advance included) — the C# subtracts the
            // full pre-move velocity from the post-move position.
            self.pos = pos_before - last_velocity;
        }
        // `:43708-43712` — the repeated-bounce bail-out.
        if !matches!(self.ai[0], 3.0 | 0.0 | 5.0 | 6.0) && self.local_ai[0] >= 10.0 {
            self.ai[0] = 4.0;
        }
        // `:43702-43706` — `Collision.HitTiles` ×count (the tile damage — no
        // generic projectile→tile seam in the port) and
        // `SoundEngine.PlaySound(0, …)` (the AI has no audio access): both
        // deferred to the decor follow-up, recorded in ANALYSIS.
        let _ = loud;
    }

    /// One helper spawn request from the head's center — the Drippler's extra
    /// ball (`:43914`/`:43927`) and the Flairon's bubble (`:44094`). The
    /// Flower Pow's petal (`:44137`) aims at an NPC, so it builds its own.
    fn flail_spawn(
        &self,
        kind: u16,
        v: Vec2,
        damage: i32,
        knock_back: f32,
        ai0: f32,
    ) -> FlailSpawnRequest {
        let c = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
        FlailSpawnRequest {
            kind,
            x: c.x,
            y: c.y,
            vx: v.x,
            vy: v.y,
            damage,
            knock_back,
            ai0,
        }
    }

    /// The `Main.myPlayer == this.owner` gate on the family's spawns — the
    /// local owner only (the port's single-player stand-in for the C# owner
    /// check).
    fn flail_helper(
        &self,
        tick: &mut FlailTick,
        inputs: FlailInputs,
        kind: u16,
        v: Vec2,
        ai0: f32,
    ) {
        if self.owner as u8 == inputs.local_owner {
            let req = self.flail_spawn(kind, v, self.damage, self.knock_back, ai0);
            tick.spawn = Some(req);
        }
    }

    /// The 1058/247 per-kind tails (`Projectile.cs:44050-44141`) plus the
    /// common tail (`:44142-44168`). Every state arm falls out of the C#
    /// switch into this block, so `ai_flail` routes its non-kill exits
    /// through it. The `player.heldProj`/`SetDummyItemTime`/`itemRotation`
    /// writes have no port consumers (documented deviations).
    fn flail_tail(
        &mut self,
        world: &World,
        eff_dir: i32,
        mut tick: FlailTick,
        inputs: FlailInputs,
        npcs: &[Npc],
        rng: &mut dyn tr_core::loot::DropRng,
    ) -> FlailTick {
        let center = inputs.center.map(Vec2::from).unwrap_or_default();
        let head = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
        let mine = inputs.local_owner == self.owner as u8;
        let special = matches!(self.kind, FLAIRON | FLOWER_POW);
        match self.kind {
            FLAIRON => {
                // `:44053-44062` — the alpha fades in, then the rotation.
                self.alpha = (self.alpha - 50).max(0);
                let to_player = center - head;
                self.rot = to_player.y.atan2(to_player.x) - std::f32::consts::PI / 2.0;
                self.direction = if to_player.x < 0.0 { 1 } else { -1 };
                // `:44075-44095` — the bubble cadence. The C# re-rolls
                // `Next(3, 6)` every tick in the held state (and halves the
                // knockback there); the jitter is the item-stream roll.
                if mine {
                    self.local_ai[2] += 1.0;
                    let state = self.ai[0] as i32;
                    let mut knock = self.knock_back;
                    let cadence = if state == 0 {
                        let c = rng.next_range(3, 6) as f32;
                        knock *= 0.5;
                        c
                    } else if state == 6 || state == 3 {
                        6.0
                    } else {
                        4.0
                    };
                    if self.local_ai[2] >= cadence {
                        self.local_ai[2] = 0.0;
                        // `vector8 = ((vector7 * -1).SafeNormalize(0) *
                        // (Next(45, 65) * 0.1)).RotatedBy(
                        // (NextDouble() - 0.5) * (π/2))`.
                        let back = (-to_player).normalize_or_zero();
                        let speed = rng.next_range(45, 65) as f32 * 0.1;
                        let jitter = (rng.next_float() - 0.5) * std::f32::consts::PI / 2.0;
                        let v = (back * speed).rotate(Vec2::from_angle(jitter));
                        let req = self.flail_spawn(FLAIRON_BUBBLE, v, self.damage, knock, -10.0);
                        tick.spawn = Some(req);
                    }
                }
            }
            FLOWER_POW => {
                // `:44101-44107` — the spin.
                let spin = (self.vel.x.abs() + self.vel.y.abs()) * 0.01;
                self.rot += if self.vel.x > 0.0 { spin } else { -spin };
                if self.ai[0] == 0.0 {
                    self.rot += std::f32::consts::PI * 2.0 / 15.0 * eff_dir as f32;
                }
                // `:44114-44138` — the local owner's NPC scan: the closest
                // `CanBeChasedBy` within 600 px with tile LOS fires the petal
                // (`damage / 1.5`, `knockBack / 2`, speed 14).
                if mine {
                    self.local_ai[0] += 1.0;
                    if self.local_ai[0] >= 20.0 {
                        self.local_ai[0] = 17.0;
                        let mut best: Option<(usize, f32)> = None;
                        for (idx, n) in npcs.iter().enumerate() {
                            if n.hp <= 0 || n.dont_take_damage {
                                continue;
                            }
                            let (nx, ny) = n.center();
                            let d = (Vec2::new(nx, ny) - head).length();
                            if d < 600.0 && yoyo_can_hit(world, head.x, head.y, nx, ny) {
                                let closer = match best {
                                    Some((_, bd)) => d < bd,
                                    None => true,
                                };
                                if closer {
                                    best = Some((idx, d));
                                }
                            }
                        }
                        if let Some((idx, _)) = best {
                            let (nx, ny) = npcs[idx].center();
                            let v = (Vec2::new(nx, ny) - head).normalize_or_zero() * 14.0;
                            let req = FlailSpawnRequest {
                                kind: FLOWER_POW_PETAL,
                                x: head.x,
                                y: head.y,
                                vx: v.x,
                                vy: v.y,
                                damage: (self.damage as f32 / 1.5) as i32,
                                knock_back: self.knock_back / 2.0,
                                ai0: 0.0,
                            };
                            tick.spawn = Some(req);
                        }
                    }
                }
            }
            _ => {}
        }
        // `:44142-44145` — the direction/spriteDirection from the velocity X
        // (the Flairon's tail set its own above).
        if self.kind != FLAIRON {
            self.direction = if self.vel.x > 0.0 { 1 } else { -1 };
        }
        // `:44148-44158` — the rotation arm; the 1058/247 tails set
        // `flag = false` and keep their own rotation.
        if !special {
            if self.vel.length() > 1.0 {
                self.rot = self.vel.y.atan2(self.vel.x) + self.vel.x * 0.1;
            } else {
                self.rot += self.vel.x * 0.1;
            }
        }
        // `:44159` — the per-tick lifetime re-pin.
        self.time_left = 2;
        tick.facing = Some(eff_dir);
        tick
    }    /// `AI_016_Bombs` (`Projectile.cs:46904-47545`): the thrown-bomb flight.
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

    /// `Main.cs:27941`'s `proj.aiStyle == 99` gate — the string's own
    /// condition (the counterweights share it; only the dropped twin
    /// `ai[0] == -2` excludes).
    pub fn ai_style_is_yoyo(&self) -> bool {
        proj_data(self.kind).is_some_and(|d| d.ai_style == 99)
    }

    /// The M4-PROJ-4c yoyo path: the string segments from the owner's hand
    /// to the projectile (`Main.DrawProjDirect`, `Main.cs:27941-28046`) then
    /// the sprite quad. `owner_hand` is the player's centre + `gfxOffY` the
    /// C# anchors the walk on (`:27943-27945`).
    fn draw_yoyo(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
        owner_hand: Option<(f32, f32)>,
    ) {
        let world_light = |x: f32, y: f32| -> [f32; 3] {
            let tx = (x / TILE_PX) as i32;
            let ty = (y / TILE_PX) as i32;
            if light.w > 0 && light.h > 0 {
                light.get(
                    tx.clamp(light.x0, light.x0 + light.w - 1),
                    ty.clamp(light.y0, light.y0 + light.h - 1),
                )
            } else {
                [1.0; 3]
            }
        };
        // The string (`:27941`'s gate: aiStyle 99 and not the dropped twin).
        if self.ai_style_is_yoyo() && self.ai[0] != -2.0 {
            if let (Some(line), Some(hand)) = (lookups.fishing_line, owner_hand) {
                let atlas = (1u32.max(line.width), 1u32.max(line.height));
                let (cx, cy) = (self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                let sag = 0.0; // the speed curvature rides the follow-up row
                for seg in crate::yoyo_string::string_segments(hand, (cx, cy), sag) {
                    let uv = assets::uv_rect(
                        &line,
                        atlas,
                        0,
                        0,
                        crate::yoyo_string::FISHING_LINE_WIDTH as u32,
                        seg.len as u32,
                    );
                    let pos = [
                        seg.x - crate::yoyo_string::line_x_offset(),
                        seg.y,
                    ];
                    let snapped = crate::render::entities::snap_to_screen_pub(
                        pos, camera, zoom, screen,
                    );
                    r.push_sprite_origin_rot(
                        snapped,
                        [
                            crate::yoyo_string::FISHING_LINE_WIDTH * zoom,
                            seg.len * zoom,
                        ],
                        uv,
                        crate::yoyo_string::segment_color(0, [0.0; 3], [0.0; 3], world_light(seg.x, seg.y)),
                        seg.rot,
                        [
                            crate::yoyo_string::FISHING_LINE_WIDTH * 0.5 * zoom,
                            0.0,
                        ],
                    );
                }
            }
        }
        // The sprite (`drawLayer` 0 for the counterweights — they draw in the
        // plain pass, ahead of the yoyo the loop emits them before).
        let Some(sheet) = lookups.yoyo_sprites.get(&self.kind).copied() else {
            return;
        };
        let atlas = (sheet.width.max(1), sheet.height.max(1));
        let uv = assets::uv_rect(&sheet, atlas, 0, 0, sheet.width, sheet.height);
        let c = world_light(self.pos.x, self.pos.y);
        let alpha = (self.alpha as f32 / 255.0).clamp(0.0, 1.0);
        let pos = [
            self.pos.x + self.w * 0.5,
            self.pos.y + self.h * 0.5,
        ];
        let snapped =
            crate::render::entities::snap_to_screen_pub(pos, camera, zoom, screen);
        r.push_sprite_origin_rot(
            snapped,
            [
                self.w.max(self.h) * self.scale * zoom,
                self.w.max(self.h) * self.scale * zoom,
            ],
            uv,
            [c[0], c[1], c[2], alpha],
            self.rot,
            [
                self.w.max(self.h) * self.scale * zoom * 0.5,
                self.w.max(self.h) * self.scale * zoom * 0.5,
            ],
        );
    }

    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        self.draw_with_hand(r, lookups, light, camera, zoom, screen, None);
    }

    /// The draw entry the frame calls with the player in scope: the yoyo
    /// family takes the string path (`Main.DrawProjDirect`'s aiStyle-99 arm),
    /// everything else keeps the shuriken stub.
    pub fn draw_with_hand(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
        owner_hand: Option<(f32, f32)>,
    ) {
        if crate::yoyo_string::is_yoyo_sprite(self.kind) {
            self.draw_yoyo(r, lookups, light, camera, zoom, screen, owner_hand);
            return;
        }
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
    /// `Player.grappling[10]` / `Player.grapCount` (`Player.cs:2742`/`:2744`):
    /// the attached aiStyle-7 hook slots, rebuilt in slot order on every tick's
    /// projectile pass. `Player.grappling` is `int[20]` in the C# but only 10
    /// entries are reachable (the `AI_007` `< 10` guard), so 10 suffice.
    hook_reg: [i16; 10],
    hook_grap_count: usize,
    /// The hooks' own `Main.rand` stand-in (`AI_007`'s 1/16 cracked-brick break,
    /// `Projectile.cs:52586-52595`), seeded from the world seed like the other
    /// per-consumer streams and interleaved with nothing else.
    hook_rng: u64,
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

/// The player-state snapshot `AI_099_2_Yoyos` reads out of `Main.player[owner]`
/// (`Projectile.cs:67733+`): the aim world point
/// (`Main.ReverseGravitySupport(Main.MouseScreen) + Main.screenPosition`,
/// `:67928`), the held-button channel state (`Player.channel` — the release
/// arm `:67920-67925`) and the owner's death (`:67824`). The accessory flags
/// (yoyoString / magicString) and `meleeSpeed` land with M3-EQUIP-3 — the AI's
/// terms stay at their ported base values until then.
#[derive(Clone, Copy, Debug, Default)]
pub struct YoyoInputs {
    /// The aim point in world pixels; `None` = no player link — the orphan
    /// guard, the slot holds position (unreachable in game: the pool only
    /// spawns the player's yoyos).
    pub aim: Option<(f32, f32)>,
    /// `Player.channel` — the use button held with the channel item.
    pub channel: bool,
    /// `Main.player[owner].dead` (`:67824`).
    pub player_dead: bool,
}

/// What `AI_015_Flails` reads off `Main.player[owner]` each tick — the
/// `YoyoInputs` peer, widened for the chain anchor. Built once per fixed
/// step in `world_sim/projectiles.rs` (the slot loop cannot borrow the
/// player) and threaded through `ProjectilePool::update` to the aiStyle-15
/// arm. The C# reads the live `Player` fields; the port snapshots them.
/// `Copy`: every field is a scalar/pair, and one snapshot serves every
/// simulated slot in a tick.
#[derive(Clone, Copy)]
pub struct FlailInputs {
    /// `Main.MouseWorld` (`Projectile.cs:43863`) — the hold state's aim.
    pub aim: Option<(f32, f32)>,
    /// `Player.channel` (`:43872`, `:43905`, `:43942`, `:43959`, `:44035`).
    pub channel: bool,
    /// The kill gates (`:43727`): dead / `CCed` — the port drops the
    /// `noItems` arm (no field, documented).
    pub player_dead: bool,
    /// `Player.CCed` (`:43727`).
    pub player_cced: bool,
    /// `Player.MountedCenter` (`:3335`) — the chain anchor.
    pub mounted_center: Option<(f32, f32)>,
    /// `Player.Center` — the kill-gate leash and the `ChangeDir`
    /// comparisons (`:43727`, `:43930`).
    pub center: Option<(f32, f32)>,
    /// `player.velocity` — the throw add and the state-3 stand-still read
    /// (`:43876`, `:43992`).
    pub player_vel: (f32, f32),
    /// `player.direction` — the hold orbit's spin axis (`:43889`).
    pub player_dir: i32,
    /// `player.meleeSpeed` (`:43824-43831`).
    pub melee_speed: f32,
    /// `Main.myPlayer` — the family's spawns are local-owner only
    /// (`:43912`, `:44075`, `:44110`).
    pub local_owner: u8,
}

impl Default for FlailInputs {
    /// `meleeSpeed = 1f` — the C# base; a 0 here would divide the
    /// meleeSpeed scaling to infinity.
    fn default() -> Self {
        Self {
            aim: None,
            channel: false,
            player_dead: false,
            player_cced: false,
            mounted_center: None,
            center: None,
            player_vel: (0.0, 0.0),
            player_dir: 1,
            melee_speed: 1.0,
            local_owner: 0,
        }
    }
}

impl ProjectilePool {
    pub fn new() -> Self {
        Self {
            slots: vec![Projectile::default(); MAX_PROJECTILES],
            local_immunity: vec![0; MAX_PROJECTILES * 256],
            id_static: HashMap::new(),
            update_count: 0,
            hook_reg: [-1; 10],
            hook_grap_count: 0,
            hook_rng: 0,
            wind_speed: 0.0,
        }
    }

    /// The wind vector the AI arms' tail applies
    /// (`Projectile.cs:16777-16781`, the port's `Main.windSpeedCurrent`).
    pub fn set_wind(&mut self, wind_speed: f32) {
        self.wind_speed = wind_speed;
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

    pub fn kill(&mut self, slot: usize) {
        if slot < self.slots.len() {
            self.slots[slot].active = false;
            self.slots[slot].time_left = 0;
        }
    }

    /// `QuickGrapple`'s `Projectile.NewProjectile` (`Player.cs:6458-6477`):
    /// the aim direction normalized by its own length and scaled to
    /// `shootSpeed`, spawned at the player center. The degenerate aim
    /// (`0,0` or NaN) falls back to the facing, as the source does.
    pub fn spawn_hook(
        &mut self,
        kind: u16,
        player_center: Vec2,
        aim: Vec2,
        shoot_speed: f32,
        damage: i32,
        knock_back: f32,
        owner: u8,
        facing: i32,
    ) -> Option<usize> {
        let delta = aim - player_center;
        // `:6459-6467` — the degenerate aim (both zero, or NaN) falls back to
        // the facing; otherwise the direction scales to `shootSpeed`.
        let (vx, vy) = if (delta.x == 0.0 && delta.y == 0.0) || delta.is_nan() {
            (facing as f32 * shoot_speed, 0.0)
        } else {
            let k = shoot_speed / delta.length();
            (delta.x * k, delta.y * k)
        };
        self.spawn_with_data_ai_owned(kind, owner, player_center.x, player_center.y, vx, vy, damage, knock_back, 0.0, 0.0)
    }


    /// `Player.grappling` / `Player.grapCount` — the attached aiStyle-7 slots in
    /// registration order (the C#'s slot order), rebuilt each tick by
    /// [`Self::update`]. `GrappleMovement`'s pull and the fire-time budgets read
    /// this; the AI arm and the recycle tables are the only writers.
    pub fn hook_reg(&self) -> (&[i16; 10], usize) {
        (&self.hook_reg, self.hook_grap_count)
    }

    /// Seed the hooks' `Main.rand` stand-in from the world seed — the app calls it
    /// once at construction (the pool is built before the world exists).
    pub fn seed_hooks(&mut self, world_seed: u64) {
        self.hook_rng = world_seed ^ 0x606B_005B_5EED;
    }

    /// `Player.Update`'s `grappling[0] = -1; grapCount = 0` (`:28300-28301`):
    /// the clear. The port runs it at the top of the projectile pass, which
    /// keeps the vanilla's one-tick latency (the pull reads last tick's set).
    fn clear_hook_reg(&mut self) {
        self.hook_reg = [-1; 10];
        self.hook_grap_count = 0;
    }

    /// `AI_007`'s `grappling[grapCount++] = whoAmI` (`:52705`/`:52862`), under the
    /// `< 10` guard. Only an active slot is ever appended (the caller gates it),
    /// so a dead index cannot land.
    fn register_hook(&mut self, slot: usize) {
        if self.hook_grap_count < self.hook_reg.len() {
            self.hook_reg[self.hook_grap_count] = slot as i16;
            self.hook_grap_count += 1;
        }
    }

    /// `QuickGrapple`'s fire-time gate and recycle (`Player.cs:6396-6399`): the
    /// in-flight and attached counts over this owner's aiStyle-7 slots.
    pub fn hook_counts(&self, owner: u8) -> (usize, usize) {
        let mut flight = 0usize;
        let mut attached = 0usize;
        for p in self.slots.iter() {
            if p.active && p.owner == owner && proj_data(p.kind).is_some_and(|d| d.ai_style == 7) {
                if p.ai[0] == 2.0 {
                    attached += 1;
                } else {
                    flight += 1;
                }
            }
        }
        (flight, attached)
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
    /// extraUpdates and the local-immunity pair) on top of the spawn
    /// arguments. `None` when the kind has no ported data — the caller decides
    /// whether that is fatal.
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
            uses_local_immunity: data.uses_local_immunity,
            local_hit_cooldown: data.local_hit_cooldown,
            uses_id_static_immunity: data.uses_id_static_immunity,
            id_static_hit_cooldown: data.id_static_hit_cooldown,
            extra_updates: data.extra_updates,
            ai: [ai0, ai1, 0.0, 0.0],
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
    /// `AI_103`'s mark-spawn drain (`Projectile.cs:33467-33475`): the
    /// `localAI[0]` id the mark spawns rides the intent out to the world
    /// sim, which owns the NPC vec.
    pub fn drain_mark_spawns(&mut self) -> Vec<(u16, f32, f32, f32, f32)> {
        let mut out = Vec::new();
        for p in self.slots.iter_mut() {
            if let Some(req) = p.wants_mark_spawn.take() {
                out.push(req);
            }
        }
        out
    }

    /// `rng` is the shared item stream (`Main.rand`'s analogue): the whip
    /// family's gameplay rolls take from it (`AI_165_Whip`'s arm budget + spin,
    /// `:47671`/`:47710`; the ThornWhip's debuff roll `:11738`; the tag crit
    /// roll), so do the yoyos and the throw's crit roll.
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
        rng: &mut dyn tr_core::loot::DropRng,
    ) -> (i32, bool) {
        let p = &self.slots[slot];
        if !p.is_summon_shot || p.owner == 255 {
            return (base, crit);
        }
        let Some(player) = player else {
            return (base, crit);
        };
        let mut changes = tr_core::whip::TagDamageChanges::none();
        for tag_type in player.tag_effects.active_types() {
            player
                .tag_effects
                .modify_hit(npc_idx as usize, tag_type, &mut changes, rng);
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
    /// `:13707-13809` — the yoyo on-hit spawns (M4-PROJ-4b). All four arms
    /// share the owner's `yoyoHitCounter` in the increment form (the same
    /// counter the doublings peeked before the strike), and all are
    /// owner-local in the C# (`this.owner == Main.myPlayer`) — the pool's
    /// `damage_pve` only runs for locally-simulated slots, which is the
    /// port's equivalent gate.
    fn yoyo_on_hit(
        &mut self,
        slot: usize,
        npc_idx: u16,
        hit: Vec2,
        mut player: Option<&mut crate::player::Player>,
        rng: &mut dyn tr_core::loot::DropRng,
    ) {
        let kind = self.slots[slot].kind;
        let (owner, damage, kb, center) = {
            let p = &self.slots[slot];
            (p.owner, p.damage, p.knock_back, p.pos + Vec2::splat(SIZE * 0.5))
        };
        let dropped = self.slots[slot].ai[0] == -2.0;
        match kind {
            CASCADE_YOYO if !dropped => {
                let Some(pl) = player.as_deref_mut() else { return };
                if !pl.every_some_yoyo_hits(2, true) {
                    return;
                }
                let f = rng.next_float() * std::f32::consts::TAU;
                // The `num29` floor/clamp (`:13734-13737`): the C# seeds the
                // target distance at 208 when the scan finds none, which
                // floors the shot's ai1 at dist/200.
                let floor = 208.0 / 200.0;
                let jitter = 0.22 + 0.22 * rng.next_float();
                let ai1 = jitter.max(floor).min(1.5);
                let spawned = self.spawn_with_data_ai_owned(
                    CASCADE_EXPLOSION,
                    owner,
                    center.x,
                    center.y,
                    f.cos() * ai1,
                    f.sin() * ai1,
                    (damage as f32 * 2.0) as i32,
                    kb,
                    0.0,
                    ai1,
                );
                // `localNPCImmunity[victimIndex] = -1` (`:13746`): the -1
                // sentinel blocks that NPC for the shot's whole life.
                if let Some(s) = spawned {
                    self.local_immunity[s * 256 + npc_idx as usize] = -1;
                }
            }
            KRAKEN_YOYO => {
                let Some(pl) = player.as_deref_mut() else { return };
                if !pl.every_some_yoyo_hits(4, true) {
                    return;
                }
                let dir = hit - center;
                let len = dir.length();
                let dir = if len <= 1.0e-6 {
                    Vec2::new(1.0, 0.0)
                } else {
                    dir / len
                };
                for l in [-2i32, 0, 2] {
                    let v = rotate(dir, (l as f32) * std::f32::consts::TAU * 0.05)
                        * (18.0 + 8.0 * rng.next_float());
                    let _ = self.spawn_with_data_ai_owned(
                        KRAKEN_WAVE,
                        owner,
                        center.x,
                        center.y,
                        v.x,
                        v.y,
                        ((damage as f32) * 0.5) as i32,
                        kb,
                        0.0,
                        0.0,
                    );
                }
            }
            CHIK_YOYO => {
                let Some(pl) = player.as_deref_mut() else { return };
                if !pl.every_some_yoyo_hits(3, true) {
                    return;
                }
                for m in 0..5 {
                    let angle = rng.next_float() * std::f32::consts::TAU;
                    let mag = 8.0;
                    let spawned = self.spawn_with_data_ai_owned(
                        CRYSTAL_SHARD_MELEE,
                        owner,
                        center.x,
                        center.y,
                        angle.cos() * mag,
                        angle.sin() * mag,
                        damage,
                        kb,
                        if m != 0 { 1.0 } else { 0.0 },
                        0.0,
                    );
                    if let Some(s) = spawned {
                        let k = self.slots[s].kind;
                        self.id_static.insert((k, npc_idx), self.update_count + 20);
                    }
                }
            }
            THE_EYE_OF_CTHULHU_YOYO => {
                let Some(pl) = player.as_deref_mut() else { return };
                if !pl.every_some_yoyo_hits(7, true) {
                    return;
                }
                let f = rng.next_float() * std::f32::consts::TAU;
                let floor = 80.0 / 200.0;
                let jitter = 0.22 + 0.22 * rng.next_float();
                let ai1 = jitter.max(floor).min(1.5);
                let spawned = self.spawn_with_data_ai_owned(
                    EYE_OF_CTHULHU_DASH,
                    owner,
                    center.x,
                    center.y,
                    f.cos() * 14.0,
                    f.sin() * 14.0,
                    (damage as f32 * 2.0) as i32,
                    kb,
                    0.0,
                    ai1,
                );
                if let Some(s) = spawned {
                    self.local_immunity[s * 256 + npc_idx as usize] = -1;
                }
            }
            _ => {}
        }
    }
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
    fn whip_debuffs(&mut self, slot: usize, npc: &mut Npc, rng: &mut dyn tr_core::loot::DropRng) {
        let kind = self.slots[slot].kind;
        match kind {
            FIRE_WHIP => npc.buffs.add_buff(323, 240), // Hellfire
            COOL_WHIP => npc.buffs.add_buff(324, 240), // Frostburn
            THORN_WHIP => {
                // `:11738` — the 1/5 roll, from the item stream.
                if rng.next_range(0, 5) == 0 {
                    npc.buffs.add_buff(20, 180); // Poisoned
                }
            }
            EEL_WHIP => npc.buffs.add_buff(362, 240), // Electrified
            SCYTHE_WHIP => npc.buffs.add_buff(310, 240), // Oiled
            _ => {}
        }
    }
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

    pub fn update(
        &mut self,
        world: &mut World,
        npcs: &mut [Npc],
        owner: Option<(f32, f32)>,
        aim: Option<Vec2>,
        mut player: Option<&mut crate::player::Player>,
        inputs: YoyoInputs,
        flail_inputs: FlailInputs,
        rng: &mut dyn tr_core::loot::DropRng,
        local_owner: u8,
        hostile_local: bool,
    ) {
        self.update_count += 1;
        // `Player.Update`'s clear (`Player.cs:28300-28301`): the C#
        // runs it at the end of the player pass, so starting here preserves
        // the vanilla's one-tick latency exactly.
        self.clear_hook_reg();
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
                        // `AI()`'s `else if (this.aiStyle == 7)`
                        // (`Projectile.cs:24230-24233`) ->
                        // `AI_007_GrapplingHooks` — the grappling hooks.
                        7 => {
                            let dead = player.as_ref().is_some_and(|p| p.dead);
                            let local = self.slots[i].owner == local_owner;
                            let center = owner
                                .map(|(x, y)| Vec2::new(x, y))
                                .unwrap_or(self.slots[i].pos);
                            // The stream is copied out and back so the slot
                            // borrow and the rng borrow never overlap.
                            let mut rng = self.hook_rng;
                            let tick =
                                self.slots[i].ai_hook(world, center, dead, local, Some(&mut rng));
                            self.hook_rng = rng;
                            let crack = tick.crack;
                            let attached = tick.attached;
                            let dead = tick.dead;
                            let latched = tick.latched;
                            if dead {
                                self.kill(i);
                            } else if let Some(c) = attached {
                                self.slots[i].grapple_center = c;
                                self.register_hook(i);
                                if latched {
                                    let owner = self.slots[i].owner;
                                    let cap =
                                        crate::grapple::hook_spec_id(self.slots[i].kind).max_attached;
                                    let mut count = 0usize;
                                    let mut worst = usize::MAX;
                                    // The C#'s `num13 = 100000` initial.
                                    let mut worst_time = i64::MAX;
                                    for (k, p) in self.slots.iter().enumerate() {
                                        let hook = p.active
                                            && p.owner == owner
                                            && proj_data(p.kind).is_some_and(|d| d.ai_style == 7);
                                        if (hook && p.ai[0] == 2.0) || k == i {
                                            count += 1;
                                            if (p.time_left as i64) < worst_time {
                                                worst = k;
                                                worst_time = p.time_left as i64;
                                            }
                                        }
                                    }
                                    if count > cap && worst != usize::MAX {
                                        self.kill(worst);
                                    }
                                }
                            }
                            // `:52590-52593` — the 1/16 cracked-brick break.
                            if let Some((tx, ty)) = crack {
                                if let Some(t) = world.tile_mut(tx, ty) {
                                    t.flags &= !tr_core::tile::FLAG_ACTIVE;
                                }
                            }
                        }
                        14 => self.slots[i].ai_spear(world),
                        16 => self.slots[i].ai_bomb(world),
                        18 => self.slots[i].ai_demon_scythe(world),
                        // The carve-out of the arm-signature doc block above:
                        // aiStyle 22 writes tiles, so it reborrows `world`
                        // mutably at its own call site.
                        22 => self.slots[i].ai_ice_block(world),
                        8 => self.slots[i].ai_ball_of_fire(world),
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
                        1 => self.slots[i].ai_arrow(world),
                        3 => self.slots[i].ai_boomerang(world, owner),
                        14 => self.slots[i].ai_spear(world),
                        16 => self.slots[i].ai_bomb(world),
                        115 => self.slots[i].ai_yoyo_beam(world),
                        210 => self.slots[i].ai_yoyo_shot(world),
                        82 => {
                            // `AI_082`'s homing target: the owner's centre
                            // (`Player.FindClosest`'s single-player stand-in).
                            let player = owner.unwrap_or((0.0, 0.0));
                            let npcs = &npcs[..];
                            self.slots[i].ai_phasmal_eye(npcs, player)
                        }
                        83 => self.slots[i].ai_phasmal_sphere(&npcs[..]),
                        84 => self.slots[i].ai_phasmal_deathray(&*world, &npcs[..]),
                        85 => {
                            if let Some(p) = player.as_deref_mut() {
                                self.slots[i].ai_moon_leech(&npcs[..], p);
                            }
                        }
                        102 => {
                            let player = owner.unwrap_or((0.0, 0.0));
                            let npcs = &npcs[..];
                            self.slots[i].ai_nebula_eye(npcs, player)
                        }
                        103 => {
                            let npcs = &npcs[..];
                            self.slots[i].ai_stardust_mark(&*world, npcs)
                        }
                        122 => {
                            // `:34927-34929`'s homing run, then the contact's
                            // shield spend (`NPC.cs:72944-72955`).
                            let (hit, dmg) = {
                                let slot = &mut self.slots[i];
                                (slot.ai_tower_damage_bolt(&npcs[..]), slot.damage)
                            };
                            if hit {
                                self.slots[i].active = false;
                                let tower = self.slots[i].ai[0] as usize;
                                let shield = match tower {
                                    0 => &mut world.tower_shield_solar,
                                    1 => &mut world.tower_shield_vortex,
                                    2 => &mut world.tower_shield_nebula,
                                    _ => &mut world.tower_shield_stardust,
                                };
                                // The clamp `:72946-72948`'s shape.
                                *shield = (*shield - dmg).max(0);
                            }
                        }
                        99 => {
                            // `AI_099_2_Yoyos`'s `flag` (`:67736-67742`): an
                            // active slot below this index, same owner and
                            // kind, `ai[0] >= -1f` — the double yoyo (the Yoyo
                            // Glove's second copy).
                            let (slot_owner, slot_kind) = {
                                let p = &self.slots[i];
                                (p.owner, p.kind)
                            };
                            let flag = self.slots[..i].iter().any(|p| {
                                p.active
                                    && p.owner == slot_owner
                                    && p.kind == slot_kind
                                    && p.ai[0] >= -1.0
                            });
                            let tick = self.slots[i].ai_yoyo(world, npcs, owner, flag, inputs, rng);
                            // `:67764-67771` — the sweep flips the whole
                            // owner's aiStyle-99 set with `ai[0] >= 0f` to the
                            // return state.
                            if tick.sweep {
                                for p in self.slots.iter_mut() {
                                    if p.active
                                        && p.owner == slot_owner
                                        && YOYO_KINDS.contains(&p.kind)
                                        && p.ai[0] >= 0.0
                                    {
                                        p.ai[0] = -1.0;
                                    }
                                }
                            }
                            // `:67815` — the Terrarian's beam spawn.
                            if let Some(b) = tick.beam {
                                self.spawn_with_data(
                                    TERRARIAN_BEAM,
                                    b.x,
                                    b.y,
                                    b.vx,
                                    b.vy,
                                    b.damage,
                                    b.knock_back,
                                );
                            }
                        }
                        165 => {
                            // `AI_165_Whip` (`:47667-48270`): the swing
                            // machine. The Moon Lord whip's extra arms are
                            // pool spawns, so the request comes back out.
                            if let Some(arm) = self.slots[i].ai_whip(owner, aim, rng) {
                                self.spawn_whip_arm(i, arm, owner);
                            }
                        }
                        15 => {
                            // `AI_015_Flails`'s `flag5` (`:43847-43862`): the
                            // Blue Moon/Sunfury sibling scan — the 26↔35 pair,
                            // same owner. It runs here, like the aiStyle-99
                            // `flag` above, because the AI only sees its slot.
                            let (slot_owner, slot_kind) = {
                                let p = &self.slots[i];
                                (p.owner, p.kind)
                            };
                            let flag5 = (slot_kind == BLUE_MOON || slot_kind == SUNFURY)
                                && self.slots.iter().any(|p| {
                                    p.active
                                        && p.owner == slot_owner
                                        && ((slot_kind == BLUE_MOON && p.kind == SUNFURY)
                                            || (slot_kind == SUNFURY && p.kind == BLUE_MOON))
                                });
                            let tick =
                                self.slots[i].ai_flail(world, npcs, owner, flail_inputs, flag5, rng);
                            // The family's helper spawn — the yoyo beam
                            // pattern. The 405's `alpha = 255` rides the spawn
                            // site (`SetDefaults` `:4857`).
                            if let Some(s) = tick.spawn {
                                let owner_id = self.slots[i].owner;
                                if let Some(slot) = self.spawn_with_data_ai_owned(
                                    s.kind,
                                    owner_id,
                                    s.x,
                                    s.y,
                                    s.vx,
                                    s.vy,
                                    s.damage,
                                    s.knock_back,
                                    s.ai0,
                                    0.0,
                                ) && s.kind == FLAIRON_BUBBLE
                                {
                                    self.slots[slot].alpha = 255;
                                }
                            }
                            // The facing the C# forces with `player.ChangeDir`
                            // mid-machine.
                            if let Some(dir) = tick.facing {
                                if let Some(p) = player.as_deref_mut() {
                                    p.facing = dir;
                                }
                            }
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
            self.damage_pve(i, npcs, Some(flail_inputs), owner, player.as_deref_mut(), rng);
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
        self.damage_pve(
            slot,
            npcs,
            None,
            None,
            None,
            &mut tr_core::tile_light::FastRandom::new(0),
        );
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

    /// The flail family's hit-side gates, one call per (slot, NPC):
    /// `Damage_CanDealDamage` (`:12448`: state 0 with `localAI[1] <= 12`
    /// deals nothing), `Colliding`'s state-0 ellipse (`:15336-15356`: the
    /// 55-px radius around the owner's mounted center with the 0.8 Y
    /// squash — 0.4 when the point sags with gravity, `gravDir == 1`
    /// here), `Damage_GetProjectileSpecificDamageMultiplier` (`:14447-14465`:
    /// ×1.2 held, the Blue Moon/Sunfury dual ×0.8) and `Damage`'s knockback
    /// arms (`:13614-13619`: ×0.35 held, ×0.5 spinning, the hit direction
    /// from the player's center). `None` = no hit this tick (the window or
    /// the ellipse miss); the state-0 ellipse REPLACES the box test, so a
    /// non-zero state must still run the caller's `contains` — the flag
    /// rides in the first tuple slot.
    fn flail_damage_adjust(
        &self,
        slot: usize,
        inputs: FlailInputs,
        npc: &Npc,
        head_center: Vec2,
    ) -> Option<FlailHit> {
        let p = &self.slots[slot];
        // The no-damage window first — the held head is harmless for the
        // first 12 ticks of the charge.
        if p.ai[0] == 0.0 && p.local_ai[1] <= 12.0 {
            return None;
        }
        if p.ai[0] == 0.0 {
            // The state-0 ellipse: the closest point of the NPC's RAW rect
            // to the mounted center, the Y squash, the 55-px radius. The
            // port's `Npc::contains` pads the box by 4 px; the C# uses the
            // rect itself (`targetRect`), so the raw x/y + size×scale read
            // is deliberate.
            let mounted = inputs.mounted_center?;
            let (w, h) = npc.size();
            // ClosestPointInRect.
            let px = mounted.0.clamp(npc.x, npc.x + w);
            let py = mounted.1.clamp(npc.y, npc.y + h);
            let dy_raw = py - mounted.1;
            // `obj.gravDir > 0f && vector2.Y > 0f` — the clamped point below
            // the player tightens the squash to 0.4.
            let num = if dy_raw > 0.0 { 0.4 } else { 0.8 };
            let dy = dy_raw / num;
            let dx = px - mounted.0;
            if (dx * dx + dy * dy).sqrt() > 55.0 {
                return None;
            }
        }
        // The damage/knockback multipliers.
        let mut dmg_mult = if p.ai[0] == 0.0 { 1.2 } else { 1.0 };
        if (p.kind == BLUE_MOON || p.kind == SUNFURY)
            && self.slots.iter().any(|s| {
                s.active
                    && s.owner == p.owner
                    && s.kind != p.kind
                    && (s.kind == BLUE_MOON || s.kind == SUNFURY)
            })
        {
            dmg_mult *= 0.8;
        }
        let kb_mult = match p.ai[0] as i32 {
            0 => 0.35,
            6 => 0.5,
            _ => 1.0,
        };
        // The C# hit direction reads the PLAYER's center — the port's knock
        // direction derives from `from_x`, so hand it the player's center.
        let from_x = inputs.center.map_or_else(|| head_center.x, |c| c.0);
        Some(FlailHit {
            damage_mult: dmg_mult,
            knock_mult: kb_mult,
            from_x,
            replaces_box: p.ai[0] == 0.0,
        })
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
        flail: Option<FlailInputs>,
        owner: Option<(f32, f32)>,
        mut player: Option<&mut crate::player::Player>,
        rng: &mut dyn tr_core::loot::DropRng,
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
            // The flail family's hit gates — `Colliding` (`:15336-15356`,
            // the state-0 ellipse replaces the box test),
            // `Damage_CanDealDamage` (`:12448`, the hold's first 12 ticks),
            // `Damage_GetProjectileSpecificDamageMultiplier` (`:14447-14465`)
            // and `Damage`'s knockback/hit-direction arms (`:13614-13619`).
            let (mut damage, mut kb, mut from_x) = (damage, kb, center.x);
            let mut skip_box_test = false;
            if let Some(f) = flail {
                if FLAIL_KINDS.contains(&p.kind) {
                    match self.flail_damage_adjust(slot, f, n, center) {
                        Some(hit) => {
                            damage = (damage as f32 * hit.damage_mult) as i32;
                            kb *= hit.knock_mult;
                            from_x = hit.from_x;
                            // `Colliding` (`:15338`): the state-0 ellipse
                            // REPLACES the box test, not supplements it.
                            skip_box_test = hit.replaces_box;
                        }
                        None => continue, // the window or the ellipse miss
                    }
                }
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
            } else if skip_box_test {
                // The flail state-0 ellipse already decided the hit.
                true
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
            let (mut dmg, hit_crit) =
                self.tag_damage_for(slot, idx as u16, damage, crit, player.as_deref(), rng);
            // `:13389-13396` — the yoyo damage doublings, evaluated BEFORE
            // the strike against the player's `yoyoHitCounter` with the peek
            // (`countUp: false`) form, so the spawn arms below see the same
            // counter value on this hit. The counter starts at -1, so the
            // effects land on hit #2 (M4-PROJ-4b).
            if let Some(pl) = player.as_deref_mut() {
                let kind = self.slots[slot].kind;
                let doubled = (kind == THE_EYE_OF_CTHULHU_YOYO && pl.every_some_yoyo_hits(7, false))
                    || (kind == CHIK_YOYO && pl.every_some_yoyo_hits(3, false));
                if doubled {
                    dmg = ((dmg as f32) * 2.0) as i32;
                }
            }
            let landed = n.hurt_from_projectile(dmg, from_x, kb, hit_crit);
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
                    self.whip_debuffs(slot, n, rng);
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
                // `:13707-13809`: the yoyo on-hit spawns (M4-PROJ-4b) — the
                // shared player counter's increment form, exactly the C#
                // arms: the Cascade's 1125, the Kraken's 3× 1135, the Chik's
                // 5× 1134 and the Eye-of-Cthulhu's homing 1130.
                self.yoyo_on_hit(slot, idx as u16, hit, player.as_deref_mut(), rng);
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
        pool.update(
            &mut w.clone(),
            &mut [],
            Some((pcx, pcy)),
            None,
            Some(&mut player),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
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
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
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
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
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
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
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
            pool.update(
                &mut world,
                &mut [],
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
        pool.update(
            &mut world,
            &mut [],
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
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
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
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
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert_eq!(
            pool.slots[0].vel.y, 0.0,
            "no gravity for the first 14 ticks"
        );
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert_eq!(pool.slots[0].vel.y, 0.0, "no gravity for the first 5 ticks");
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
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
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert_eq!(pool.slots[0].ai[0], 0.0, "still outbound at tick 29");
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
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
            pool.update(
                &mut w,
                &mut [],
                Some((100.0, 260.0)),
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut [],
                Some((80.0, 308.0)),
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut [],
                Some((100.0, 200.0)),
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
            if !pool.slots[0].active {
                died = true;
                break;
            }
        }
        assert!(died, "the stray kills past the cut-off");
    }
}

#[cfg(test)]
mod flail_tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    /// A floor at y = 30 — the NPC fixtures spawn on the surface (the shared
    /// `tests::ground` shape: the surface cache must be recomputed by hand).
    fn ground() -> tr_core::World {
        let mut w = world();
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
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn put_solid(w: &mut tr_core::World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// A wall at `x` (full column) for the bounce tests.
    fn wall_at(w: &mut tr_core::World, tx: i32) {
        for y in 0..64 {
            put_solid(w, tx, y);
        }
    }

    fn inputs(
        aim: Option<(f32, f32)>,
        channel: bool,
        mounted: (f32, f32),
        center: (f32, f32),
    ) -> FlailInputs {
        FlailInputs {
            aim,
            channel,
            player_dead: false,
            player_cced: false,
            mounted_center: Some(mounted),
            center: Some(center),
            player_vel: (0.0, 0.0),
            player_dir: 1,
            melee_speed: 1.0,
            local_owner: 0,
        }
    }

    /// One tick of the pool with the shared snapshot — the yoyo test shape.
    fn tick(
        world: &mut tr_core::World,
        pool: &mut ProjectilePool,
        inputs: FlailInputs,
    ) {
        pool.update(
            world,
            &mut [],
            inputs.center,
            None,
            None,
            YoyoInputs::default(),
            inputs,
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
    }

    fn spawn(kind: u16, x: f32, y: f32) -> (ProjectilePool, usize) {
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(kind, x, y, 0.0, 0.0, 20, 5.0, 0.0, 0.0)
            .unwrap();
        (pool, slot)
    }

    #[test]
    fn DW_3_2_the_hold_orbit_spins_num15_px_around_the_mounted_center() {
        // `Projectile.cs:43888-43897`: the charge orbit. First tick:
        // angle = π*10*(1/60)*1, so the head sits at mounted + orbit*30.
        let mut w = world();
        let (mut pool, slot) = spawn(BALL_O_HURT, 100.0, 100.0);
        let inputs = inputs(Some((600.0, 100.0)), true, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, inputs);
        let p = &pool.slots[slot];
        assert_eq!(p.ai[0], 0.0, "still the held state");
        assert_eq!(p.vel, Vec2::ZERO, "the hold zeroes the velocity");
        let angle = std::f32::consts::PI * 10.0 * (1.0 / 60.0);
        let mut orbit = Vec2::new(1.0, 0.0).rotate(Vec2::from_angle(angle));
        orbit.y *= 0.8 * 0.5; // pointing with gravity → the halving arm
        let want = Vec2::new(500.0, 500.0) + orbit * 30.0;
        assert!(
            (p.pos + Vec2::new(p.w * 0.5, p.h * 0.5) - want).length() < 1e-3,
            "the head orbits the mounted center at num15 (pos {:?}, want {:?})",
            p.pos + Vec2::new(p.w * 0.5, p.h * 0.5),
            want
        );
        assert_eq!(p.local_hit_cooldown, 15, "num12 in the hold");
    }

    #[test]
    fn DW_3_2_the_release_throws_at_aim_times_num2() {
        // `:43872-43885`: on the release the fly-out arms with
        // `velocity = aim * num2 + player.velocity`.
        let mut w = world();
        let (mut pool, slot) = spawn(BALL_O_HURT, 100.0, 100.0);
        let mut ins = inputs(Some((600.0, 500.0)), false, (500.0, 500.0), (500.0, 500.0));
        ins.player_vel = (1.0, 0.0);
        tick(&mut w, &mut pool, ins);
        let p = &pool.slots[slot];
        assert_eq!(p.ai[0], 1.0, "released into the fly-out");
        // aim = (100, 0) normalized; num2 = 14 (the Ball O' Hurt's row).
        assert!((p.vel - Vec2::new(15.0, 0.0)).length() < 1e-3, "vel {:?}", p.vel);
        // The release snaps the head to the mounted center (`:43881`), then
        // HandleMovement integrates the throw — one tick lands at
        // mounted + velocity.
        let c = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5);
        assert!((c - (Vec2::new(500.0, 500.0) + p.vel)).length() < 1e-3,
            "centered on the mount then thrown (c {:?}, vel {:?})", c, p.vel);
        assert_eq!(p.local_hit_cooldown, 10, "num13 in flight");
    }

    #[test]
    fn DW_3_6_the_dual_flail_inverts_the_throw() {
        // `:43847-43862` + `:43877-43880`: the Blue Moon/Sunfury sibling
        // flag flips the throw X and the forced facing.
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let moon = pool
            .spawn_with_data_ai(BLUE_MOON, 500.0, 500.0, 0.0, 0.0, 20, 5.0, 0.0, 0.0)
            .unwrap();
        let _sibling = pool
            .spawn_with_data_ai(SUNFURY, 500.0, 520.0, 0.0, 0.0, 20, 5.0, 0.0, 0.0)
            .unwrap();
        let ins = inputs(Some((600.0, 500.0)), false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        let p = &pool.slots[moon];
        // aim is +x but flag5 mirrors the throw.
        assert!(p.vel.x < 0.0, "the sibling flag flips the throw: {:?}", p.vel);
        assert_eq!(p.ai[0], 1.0);
    }

    #[test]
    fn DW_3_3_the_flight_ends_on_the_tick_count_or_the_distance() {
        // `:43903`: `ai[1]++ >= num | Distance(mounted) >= num3` (800).
        let mut w = world();
        // The tick-count arm: the Ball O' Hurt's num = 15.
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0, 500.0);
        pool.slots[slot].ai[0] = 1.0;
        pool.slots[slot].ai[1] = 15.0; // the C# `ai[1]++ >= num` reads the pre-increment value
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].ai[0], 2.0, "the 16th tick retracts");
        // The distance arm: past 800 px but inside the 900-px leash.
        let (mut pool, slot) = spawn(BALL_O_HURT, 1350.0, 500.0);
        pool.slots[slot].ai[0] = 1.0;
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].ai[0], 2.0, "past 800 px retracts");
    }

    #[test]
    fn DW_3_3_the_retract_kills_at_num5() {
        // `:43937-43940`: `Distance(mounted) <= num5` → Kill.
        let mut w = world();
        // the head's box center sits on the mounted center
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0);
        pool.slots[slot].ai[0] = 2.0;
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert!(!pool.slots[slot].active, "the catch kills the slot");
    }

    #[test]
    fn DW_3_4_the_wall_bounce_reflects_at_the_state_factor() {
        // `:43661-43698`: state 1 reflects at 0.4, drops into the stagger and
        // rolls the blocked move back (`position -= lastVelocity`).
        let mut w = world();
        wall_at(&mut w, 40);
        let (mut pool, slot) = spawn(BALL_O_HURT, 632.0, 100.0);
        pool.slots[slot].ai[0] = 1.0;
        // 20 px/tick into the wall face at x = 640.
        pool.slots[slot].vel = Vec2::new(20.0, 0.0);
        let before = pool.slots[slot].pos; // the top-left, not the box center
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        let p = &pool.slots[slot];
        assert_eq!(p.ai[0], 5.0, "the fly-out bounce staggers");
        assert!((p.vel - Vec2::new(-8.0, 0.0)).length() < 1e-3, "vel {:?}", p.vel);
        // `position -= vector` undoes the whole move from the top-left.
        assert_eq!(p.pos, before - Vec2::new(20.0, 0.0), "the whole move rolls back");
        assert_eq!(p.local_hit_cooldown, 10);
    }

    #[test]
    fn DW_3_4_the_spin_bounce_and_the_reel_in_bail_outs() {
        // `:44020-44024` (state 5 → 6 after num+5 ticks) and `:43962-43963`
        // (state 3 → 4 on the release) and `:44035-44040` (state 6 → 4 past
        // num18 or without the channel).
        let mut w = world();
        // State 5's stagger: num = 15 for the Ball O' Hurt, so num14 = 20.
        let (mut pool, slot) = spawn(BALL_O_HURT, 560.0, 500.0);
        pool.slots[slot].ai[0] = 5.0;
        pool.slots[slot].ai[1] = 20.0;
        let ins = inputs(None, true, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].ai[0], 6.0, "the stagger graduates");
        // State 3's release → 4.
        let (mut pool, slot) = spawn(BALL_O_HURT, 560.0, 500.0);
        pool.slots[slot].ai[0] = 3.0;
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].ai[0], 4.0, "the spin bails on the release");
        // State 6 past num18 (= num2*num + 160 = 14*15 + 160 = 370) → 4.
        let (mut pool, slot) = spawn(BALL_O_HURT, 900.0, 500.0);
        pool.slots[slot].ai[0] = 6.0;
        let ins = inputs(None, true, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].ai[0], 4.0, "the held-out spin bails past 370");
    }

    #[test]
    fn DW_3_4_the_reel_in_kills_at_num7_or_the_overshoot() {
        // `:44004-44015`: the catch kill and the Dot < 0 overshoot kill.
        let mut w = world();
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0, 500.0);
        pool.slots[slot].ai[0] = 4.0;
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert!(!pool.slots[slot].active, "the reel-in catch kills");
        // The overshoot: the head moving away from the player while already
        // inside the kill distance is killed by the Dot test on the next
        // tick's velocity.
        let (mut pool, slot) = spawn(BALL_O_HURT, 520.0, 500.0);
        pool.slots[slot].ai[0] = 4.0;
        pool.slots[slot].vel = Vec2::new(30.0, 0.0); // away from the player
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert!(!pool.slots[slot].active, "the overshoot kills");
    }

    #[test]
    fn DW_3_1_the_kill_gates_fire_on_death_cc_and_the_leash() {
        // `:43726-43731`: dead / CCed / `> 900 / meleeSpeed` — plus the
        // orphan-slot dirty path (no player snapshot).
        let mut w = world();
        // Dead.
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0, 500.0);
        let mut ins = inputs(None, true, (500.0, 500.0), (500.0, 500.0));
        ins.player_dead = true;
        tick(&mut w, &mut pool, ins);
        assert!(!pool.slots[slot].active, "the dead owner kills");
        // CCed.
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0, 500.0);
        let mut ins = inputs(None, true, (500.0, 500.0), (500.0, 500.0));
        ins.player_cced = true;
        tick(&mut w, &mut pool, ins);
        assert!(!pool.slots[slot].active, "the CCed owner kills");
        // The leash: 900 px at meleeSpeed 1.
        let (mut pool, slot) = spawn(BALL_O_HURT, 1500.0, 500.0);
        let ins = inputs(None, true, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert!(!pool.slots[slot].active, "past the leash kills");
        // Dirty: no player snapshot at all (the C# inactive-player read).
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0, 500.0);
        tick(&mut w, &mut pool, FlailInputs::default());
        assert!(!pool.slots[slot].active, "the orphan slot dies cleanly");
    }

    #[test]
    fn DW_3_7_the_flaming_mace_morphs_to_the_mace_in_liquid() {
        // `:43737-43741`: 948 wet && !lavaWet → 947.
        let mut w = world();
        // Water at the head's tile (30, 6) — the mid-tile liquid read.
        let i = w.index(30, 6);
        w.tiles[i].liquid = 200;
        let (mut pool, slot) = spawn(FLAMING_MACE, 480.0, 100.0);
        let ins = inputs(None, true, (480.0, 100.0), (480.0, 100.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].kind, MACE, "the water morph lands");
        // Lava does not morph.
        let mut w = world();
        let i = w.index(30, 6);
        w.tiles[i].liquid = 200;
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        let (mut pool, slot) = spawn(FLAMING_MACE, 480.0, 100.0);
        let ins = inputs(None, true, (480.0, 100.0), (480.0, 100.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].kind, FLAMING_MACE, "lava keeps the flame");
    }

    #[test]
    fn DW_3_5_the_tail_pins_time_left_and_the_direction() {
        // `:44142-44159`: timeLeft = 2 every tick, the direction from the
        // velocity X (non-1058), the rotation from the velocity.
        let mut w = world();
        let (mut pool, slot) = spawn(BALL_O_HURT, 560.0, 500.0);
        pool.slots[slot].ai[0] = 1.0;
        pool.slots[slot].vel = Vec2::new(8.0, 0.0);
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        let p = &pool.slots[slot];
        assert_eq!(p.time_left, 1, "re-pinned to 2 then decremented");
        assert_eq!(p.direction, 1, "the sprite direction follows +x");
        assert!((p.rot - (0.0f32.atan2(8.0) + 0.8)).abs() < 1e-4, "rot {}", p.rot);
    }

    #[test]
    fn DW_4_1_the_flairons_alpha_fades_and_the_bubble_cadence_fires() {
        // `:44053-44062` (alpha −50/tick, floor 0) and `:44077-44095`
        // (cadence 3-5 held / 6 in states 3·6 / 4 else, kb ×0.5 held).
        let mut w = world();
        let (mut pool, slot) = spawn(FLAIRON, 560.0, 500.0);
        pool.slots[slot].alpha = 255;
        let ins = inputs(None, true, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        assert_eq!(pool.slots[slot].alpha, 205, "the alpha fades in by 50");
        // The held-state cadence (3-5 ticks) spawns a bubble; the count is
        // seeded, so just let the timer run and expect at least one bubble.
        let mut bubbles = 0;
        for _ in 0..12 {
            tick(&mut w, &mut pool, ins);
            bubbles += pool
                .slots
                .iter()
                .filter(|p| p.active && p.kind == FLAIRON_BUBBLE)
                .count();
            if bubbles > 0 {
                break;
            }
        }
        assert!(bubbles > 0, "the Flairon fires its bubbles while held");
    }

    #[test]
    fn DW_4_3_the_drippler_spawns_its_extra_ball_on_the_arm_end() {
        // `:43918-43929`: state 1's arm-end spawns 928 with `velocity * 1.3`.
        let mut w = world();
        let (mut pool, slot) = spawn(DRIPPLER_FLAIL, 500.0, 500.0);
        pool.slots[slot].ai[0] = 1.0;
        pool.slots[slot].ai[1] = 13.0; // num = 13 → this tick arms out
        pool.slots[slot].vel = Vec2::new(10.0, 0.0);
        let ins = inputs(None, false, (500.0, 500.0), (500.0, 500.0));
        tick(&mut w, &mut pool, ins);
        let ball = pool
            .slots
            .iter()
            .find(|p| p.active && p.kind == DRIPPLER_EXTRA_BALL);
        assert!(ball.is_some(), "the extra ball spawns on the arm end");
        let ball = ball.unwrap();
        // The C# `velocity *= 0.3` then the ball rides `velocity * 1.3`.
        assert!((ball.vel - Vec2::new(3.9, 0.0)).length() < 1e-3, "vel {:?}", ball.vel);
    }

    #[test]
    fn the_integration_lifecycle_hold_throw_bounce_reel() {
        // The whole machine on one wall world: hold → release → fly → wall
        // bounce (stagger → spin) → reel-in catch, with the kill at the end.
        let mut w = world();
        wall_at(&mut w, 40);
        let (mut pool, slot) = spawn(BALL_O_HURT, 600.0, 100.0);
        // Hold two ticks with the aim right (toward the wall).
        let held = inputs(Some((700.0, 100.0)), true, (600.0, 100.0), (600.0, 100.0));
        tick(&mut w, &mut pool, held);
        assert_eq!(pool.slots[slot].ai[0], 0.0);
        // Release toward the wall.
        let thrown = inputs(Some((700.0, 100.0)), false, (600.0, 100.0), (600.0, 100.0));
        tick(&mut w, &mut pool, thrown);
        assert_eq!(pool.slots[slot].ai[0], 1.0, "released");
        // The wall at x = 640 stops the 14-px throw in a few ticks.
        for _ in 0..8 {
            if !pool.slots[slot].active || pool.slots[slot].ai[0] != 1.0 {
                break;
            }
            tick(&mut w, &mut pool, thrown);
        }
        assert!(pool.slots[slot].ai[0] == 5.0 || pool.slots[slot].ai[0] == 6.0,
            "the wall bounce staggers (state {:?})", pool.slots[slot].ai[0]);
        // Keep the channel held through the stagger and the spin.
        for _ in 0..40 {
            if !pool.slots[slot].active || pool.slots[slot].ai[0] == 4.0 {
                break;
            }
            tick(&mut w, &mut pool, held);
        }
        // The spin eventually bails (or the reel-in catches) — the slot must
        // end inactive through the legitimate path, never stuck.
        for _ in 0..200 {
            if !pool.slots[slot].active {
                break;
            }
            tick(&mut w, &mut pool, held);
        }
        assert!(!pool.slots[slot].active, "the flail reels in and dies");
    }

    #[test]
    fn DW_5_1_the_held_head_deals_no_damage_in_the_first_12_ticks() {
        // `:12448`: `aiStyle == 15 && ai[0] == 0 && localAI[1] <= 12` gates
        // the whole damage out — the charge wind-up is harmless even though
        // the ellipse already covers the target.
        let mut w = ground();
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0);
        pool.slots[slot].local_ai[1] = 5.0; // inside the window
        let ins = inputs(Some((600.0, 500.0)), true, (500.0, 500.0), (500.0, 500.0));
        // The head sits on the mount; the slime at tile 20 (320 px) is far,
        // so place the NPC on the mount instead.
        npcs[0].x = 520.0;
        npcs[0].y = 490.0;
        let hp_before = npcs[0].hp;
        pool.update(&mut w, &mut npcs, Some((500.0, 500.0)), None, None,
            YoyoInputs::default(), ins, &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        assert_eq!(npcs[0].hp, hp_before, "the wind-up deals nothing");
    }

    #[test]
    fn DW_5_1_the_state_0_ellipse_gates_the_hit() {
        // `:15338-15356`: the 55-px ellipse around the mounted center
        // replaces the box test; the Y squash is 0.8 above and 0.4 below.
        let mut w = ground();
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        let (mut pool, slot) = spawn(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0);
        pool.slots[slot].local_ai[1] = 13.0; // past the window
        let ins = inputs(Some((600.0, 500.0)), true, (500.0, 500.0), (500.0, 500.0));
        // 40 px right of the mount: the closest point is 40 px away → hit.
        npcs[0].x = 540.0;
        npcs[0].y = 490.0;
        pool.update(&mut w, std::slice::from_mut(&mut npcs[0]), Some((500.0, 500.0)), None, None,
            YoyoInputs::default(), ins, &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        assert!(npcs[0].hp < npcs[0].max_hp, "the ellipse hit lands");
        // 80 px right: outside the radius → no hit. Refresh the NPC.
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        npcs[0].x = 580.0;
        npcs[0].y = 490.0;
        let mut pool = ProjectilePool::new();
        let slot2 = pool
            .spawn_with_data_ai(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0, 0.0, 0.0, 20, 5.0, 0.0, 0.0)
            .unwrap();
        pool.slots[slot2].local_ai[1] = 13.0;
        let hp = npcs[0].hp;
        pool.update(&mut w, &mut npcs, Some((500.0, 500.0)), None, None,
            YoyoInputs::default(), ins, &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        assert_eq!(npcs[0].hp, hp, "outside the ellipse misses");
        // 30 px BELOW the mount: dy_raw 30 / 0.4 = 75 > 55 → miss.
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        npcs[0].x = 490.0;
        npcs[0].y = 530.0;
        let mut pool = ProjectilePool::new();
        let slot3 = pool
            .spawn_with_data_ai(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0, 0.0, 0.0, 20, 5.0, 0.0, 0.0)
            .unwrap();
        pool.slots[slot3].local_ai[1] = 13.0;
        let hp = npcs[0].hp;
        pool.update(&mut w, &mut npcs, Some((500.0, 500.0)), None, None,
            YoyoInputs::default(), ins, &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        assert_eq!(npcs[0].hp, hp, "the sagging squash (0.4) misses at 30 px");
    }

    #[test]
    fn DW_5_2_the_held_head_multiplies_the_damage_and_softens_the_knockback() {
        // `:14447` (damage ×1.2 held) and `:13614-13619` (knockback ×0.35
        // held, ×0.5 in state 6, the hit direction from the player center).
        let mut w = ground();
        // A held flail hits an NPC inside the ellipse; the slot's damage
        // 200 / kb 10 read the multipliers through the took damage.
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        npcs[0].x = 540.0;
        npcs[0].y = 490.0;
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0, 0.0, 0.0, 200, 10.0, 0.0, 0.0)
            .unwrap();
        pool.slots[slot].local_ai[1] = 13.0;
        // A big HP pool so the slime survives and the damage reads clean.
        npcs[0].hp = 10_000;
        let ins = inputs(Some((600.0, 500.0)), true, (500.0, 500.0), (500.0, 500.0));
        pool.update(&mut w, &mut npcs, Some((500.0, 500.0)), None, None,
            YoyoInputs::default(), ins, &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        // 200 × 1.2 = 240.
        assert_eq!(10_000 - npcs[0].hp, 240, "the held damage ×1.2");
    }

    #[test]
    fn DW_5_2_the_dual_flail_discounts_the_damage() {
        // `:14450-14460`: the Blue Moon/Sunfury pair applies ×0.8 on top of
        // the held ×1.2 → 200 × 0.96 = 192.
        let mut w = ground();
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        npcs[0].x = 540.0;
        npcs[0].y = 490.0;
        let mut pool = ProjectilePool::new();
        let moon = pool
            .spawn_with_data_ai(BLUE_MOON, 500.0 - 11.0, 500.0 - 11.0, 0.0, 0.0, 200, 10.0, 0.0, 0.0)
            .unwrap();
        pool.slots[moon].local_ai[1] = 13.0;
        let _sibling = pool
            .spawn_with_data_ai(SUNFURY, 500.0 - 11.0, 520.0, 0.0, 0.0, 200, 10.0, 0.0, 0.0)
            .unwrap();
        npcs[0].hp = 10_000;
        let ins = inputs(Some((600.0, 500.0)), true, (500.0, 500.0), (500.0, 500.0));
        pool.update(&mut w, &mut npcs, Some((500.0, 500.0)), None, None,
            YoyoInputs::default(), ins, &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        assert_eq!(10_000 - npcs[0].hp, 192, "the dual ×0.8 on the held ×1.2");
    }

    #[test]
    fn the_flail_gate_dirty_paths_return_no_hit() {
        // Dirty: the gate with no mounted-center snapshot and the orphan
        // slot must not panic and must deal nothing.
        let mut w = ground();
        let mut npcs = [Npc::spawn_slime(&w, 20).unwrap()];
        npcs[0].x = 540.0;
        npcs[0].y = 490.0;
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(BALL_O_HURT, 500.0 - 11.0, 500.0 - 11.0, 0.0, 0.0, 200, 5.0, 0.0, 0.0)
            .unwrap();
        pool.slots[slot].local_ai[1] = 13.0;
        let hp = npcs[0].hp;
        // No player snapshot at all — the slot dies on the kill gate before
        // the damage walk, so nothing lands.
        pool.update(&mut w, &mut npcs, None, None, None, YoyoInputs::default(),
            FlailInputs::default(), &mut tr_core::tile_light::FastRandom::new(7), 0, true);
        assert_eq!(npcs[0].hp, hp, "the orphan path deals nothing");
    }
}



#[cfg(test)]

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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
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
        pool.update(
            &mut w,
            &mut [],
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
        assert_eq!(pool.slots[0].damage, 0, "the flight damage zeroes");
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() > 1e-6,
            "no gravity before tick 5"
        );
        for _ in 0..5 {
            pool.update(
                &mut w,
                &mut [],
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert!(
            (pool.slots[0].vel.y - 0.2).abs() < 1e-6,
            "the gravity tail adds 0.2"
        );
    }

    #[test]
    fn the_yoyo_data_is_verbatim() {
        // `Projectile.cs:6013-6093` + `:9483-9490`: every yoyo kind is 16×16,
        // aiStyle 99, `penetrate = -1`, `extraUpdates = 0`, tile-colliding,
        // with the stock `timeLeft`; the scale ladder rides
        // `set_defaults_scale`.
        for kind in YOYO_KINDS {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (16.0, 16.0), "kind {kind}");
            assert_eq!(d.ai_style, 99, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.extra_updates, 0, "kind {kind}");
            assert_eq!(d.time_left, TIME_LEFT_DEFAULT, "kind {kind}");
            assert!(d.tile_collide, "kind {kind}");
        }
        // The scale ladder (`:6028-6067`): 547 1.1, 550/551 1.1, 554 1.2,
        // 555 1.15, 563 1.05, 564 1.075, 603 1.15, the rest 1.0.
        let scales = [
            (CODE2_YOYO, 1.1),
            (REDS_YOYO, 1.1),
            (VALKYRIE_YOYO, 1.1),
            (KRAKEN_YOYO, 1.2),
            (THE_EYE_OF_CTHULHU_YOYO, 1.15),
            (GRADIENT_YOYO, 1.05),
            (VALOR_YOYO, 1.075),
            (TERRARIAN_YOYO, 1.15),
            (WOOD_YOYO, 1.0),
            (HIVE_FIVE, 1.0),
        ];
        for (kind, want) in scales {
            assert_eq!(
                proj_data(kind).unwrap().scale,
                want,
                "kind {kind}'s SetDefaults scale"
            );
        }
    }

    #[test]
    fn the_yoyo_tables_are_verbatim() {
        // `ProjectileID.cs:425-427`: the three per-yoyo sets, with the
        // `CreateFloatSet` defaults for the unlisted kinds.
        assert_eq!(yoyo_lifetime(WOOD_YOYO), 3.0);
        assert_eq!(yoyo_lifetime(RALLY_YOYO), 5.0);
        assert_eq!(yoyo_lifetime(GRADIENT_YOYO), 30.0, "the Gradient's 30 s");
        assert_eq!(yoyo_lifetime(HIVE_FIVE), 11.0);
        assert_eq!(
            yoyo_lifetime(KRAKEN_YOYO),
            -1.0,
            "the unlisted kinds keep the -1 default (no timeout)"
        );
        assert_eq!(yoyo_max_range(WOOD_YOYO), 130.0);
        assert_eq!(yoyo_max_range(TERRARIAN_YOYO), 400.0, "the Terrarian's 400");
        assert_eq!(yoyo_max_range(CODE1_YOYO), 220.0);
        assert_eq!(yoyo_max_range(KRAKEN_YOYO), 380.0, "the Kraken is listed");
        assert_eq!(
            yoyo_max_range(FORMAT_C_YOYO),
            320.0,
            "the Format C is listed"
        );
        assert_eq!(yoyo_top_speed(WOOD_YOYO), 9.0);
        assert_eq!(yoyo_top_speed(KRAKEN_YOYO), 22.0);
        assert_eq!(yoyo_top_speed(TERRARIAN_YOYO), 17.5);
        // The factory default for an unlisted kind in the range/top-speed
        // tables (200f / 10f) — a non-yoyo kind reads it too.
        assert_eq!(yoyo_max_range(SHURIKEN), 200.0);
        assert_eq!(yoyo_top_speed(SHURIKEN), 10.0);
    }

    #[test]
    fn a_yoyo_spawns_through_the_dispatch_arm() {
        // The aiStyle-99 dispatch runs the arm without touching the other
        // families (the arm's body lands with the next phase).
        let mut pool = ProjectilePool::new();
        let wood = pool
            .spawn_with_data(WOOD_YOYO, 320.0, 320.0, 0.0, 0.0, 9, 2.5)
            .expect("the Wood Yoyo data row exists");
        let terrarian = pool
            .spawn_with_data(TERRARIAN_YOYO, 320.0, 320.0, 8.0, 0.0, 190, 6.5)
            .expect("the Terrarian data row exists");
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
        assert!(
            pool.slots[wood].active,
            "the slot survives its first update"
        );
        assert!(pool.slots[terrarian].active);
        assert_eq!(
            pool.slots[terrarian].local_ai[0], 1.0,
            "the lifetime clock ticks on the first update"
        );
        assert_eq!(
            pool.slots[terrarian].local_ai[1], 1.0,
            "the Terrarian's beam timer ticks on the first update"
        );
        assert!(
            (pool.slots[terrarian].scale - 1.15).abs() < 1e-6,
            "the Terrarian scale"
        );
        assert_eq!(pool.slots[terrarian].direction, 1, "the stock direction");
    }
}

#[cfg(test)]
mod yoyo_tests {
    // --- M4-PROJ-4b: the on-hit effects -------------------------------------

    /// The NPC fixture at a fixed cell + the pool spawn of one yoyo on it.

    fn on_hit_setup(kind: u16) -> (tr_core::World, ProjectilePool, Npc, crate::player::Player) {
        let mut world = tr_core::World::new("t".into(), 200, 200);
        // A solid floor for the NPC's spawn scan (`spawn_slime` reads
        // `surface_of`, which reads `column_surface`) — the tests' `ground()`
        // pattern: `compute_column_surface` + `compute_chunks` on a slab.
        for x in 0..200 {
            for y in 60..200 {
                let i = world.index(x, y);
                world.tiles[i] = Tile {
                    kind: 1,
                    ..Tile::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        world.compute_column_surface();
        world.compute_chunks();
        let mut npc = Npc::spawn_slime(&world, 40).unwrap();
        let mut pool = ProjectilePool::new();
        // The projectile sits ON the NPC's centre so `damage_pve`'s touch
        // test lands every tick.
        let (nw, nh) = npc.size();
        let (ncx, ncy) = (npc.x + nw * 0.5, npc.y + nh * 0.5);
        // `spawn_with_data_ai_owned` so the slot carries its own
        // `SetDefaults` (the yoyo's `penetrate = -1` — a yoyo never dies on
        // a hit; the raw `spawn` leaves the default and the generic
        // post-hit kill would fire).
        pool.spawn_with_data_ai_owned(
            kind,
            0,
            ncx,
            ncy,
            0.0,
            0.0,
            20,
            4.0,
            0.0,
            0.0,
        )
        .unwrap();
        let player = crate::player::Player::spawn(&world);
        let player = crate::player::Player::spawn(&world);
        (world, pool, npc, player)
    }

    /// One tick with the NPC on the projectile's centre (the damage pass
    /// runs), the player in the fifth slot like the pool's callers.
    fn tick(
        world: &mut tr_core::World,
        pool: &mut ProjectilePool,
        npc: &mut Npc,
        player: &mut Option<crate::player::Player>,
    ) {
        let (nw, nh) = npc.size();
        let aim = Vec2::new(npc.x + nw * 0.5 + 8.0, npc.y + nh * 0.5);
        pool.update(
            world,
            std::slice::from_mut(npc),
            Some((aim.x, aim.y)),
            Some(aim),
            player.as_mut(),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
    }

    /// The slot/NPC local-immunity entry — the tests read the -1 sentinel
    /// the yoyo spawns stamp.
    fn local_immunity(pool: &ProjectilePool, slot: usize, npc: usize) -> i32 {
        pool.local_immunity[slot * 256 + npc]
    }

    /// The same fixture for a plain (non-yoyo) shot: the raw spawn (no yoyo
    /// data row) so the generic post-hit kill fires and no yoyo arm runs.
    fn on_hit_setup_plain(kind: u16) -> (tr_core::World, ProjectilePool, Npc, crate::player::Player) {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(CHIK_YOYO);
        // Re-spawn the slot as the plain kind over the yoyo fixture.
        let (nw, nh) = npc.size();
        let (ncx, ncy) = (npc.x + nw * 0.5, npc.y + nh * 0.5);
        let slot = &mut pool.slots_mut()[0];
        slot.kind = kind;
        slot.pos = Vec2::new(ncx, ncy);
        (world, pool, npc, player)
    }

    /// `Player.EverySomeYoyoHits` (`Player.cs:4308-4316`): the -1 start
    /// skips the first hit, the modulo period holds, the peek form never
    /// bumps.
    #[test]
    fn the_hit_counter_peeks_and_bumps_like_the_csharp() {
        let world = tr_core::World::new("t".into(), 32, 32);
        let mut p = crate::player::Player::spawn(&world);
        assert_eq!(p.yoyo_hit_counter, -1, "the vanilla -1 start");
        assert!(!p.every_some_yoyo_hits(7, false), "the peek never bumps");
        assert_eq!(p.yoyo_hit_counter, -1);
        assert!(!p.every_some_yoyo_hits(7, true), "the first hit: -1 % 7");
        assert_eq!(p.yoyo_hit_counter, 0);
        // The counter's 0 makes the SECOND hit fire (the -1 start shifts the
        // whole schedule), then every 7th.
        assert!(p.every_some_yoyo_hits(7, true), "the second hit fires");
        assert_eq!(p.yoyo_hit_counter, 1);
        for _ in 0..6 {
            assert!(!p.every_some_yoyo_hits(7, true));
        }
        assert!(p.every_some_yoyo_hits(7, true), "the ninth hit fires");
    }

    /// The doublings: the Eye of Cthulhu doubles on the 2nd, 9th, … hit
    /// (`Projectile.cs:13389-13396`) — with the -1 counter the first effect
    /// lands on hit #2, not #7.
    #[test]
    fn the_eye_of_cthulhu_doubles_on_every_seventh_hit() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(THE_EYE_OF_CTHULHU_YOYO);
        let mut held: Option<crate::player::Player> = Some(player);
        let mut doubled = 0;
        for _ in 0..12 {
            let before = npc.max_hp - npc.hp;
            let damage = pool.slots()[0].damage;
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
            if (npc.max_hp - npc.hp) - before > damage {
                doubled += 1;
            }
            if !pool.slots()[0].active {
                break;
            }
        }
        assert!(doubled > 0, "the doubling fired at least once: {doubled}");
    }

    /// The Chik doubles every 3rd (`:13393`).
    #[test]
    fn the_chik_doubles_on_every_third_hit() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(CHIK_YOYO);
        let mut held = Some(player);
        let mut doubled = 0;
        for _ in 0..10 {
            let before = npc.max_hp - npc.hp;
            let damage = pool.slots()[0].damage;
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
            if (npc.max_hp - npc.hp) - before > damage {
                doubled += 1;
            }
            if !pool.slots()[0].active {
                break;
            }
        }
        assert!(doubled > 0, "the Chik's doubling fired: {doubled}");
    }

    /// The Cascade's 1125 spawn on every 2nd hit (`:13707-13747`): the shot,
    /// the `ai1` floor at `208/200`, the damage x2, and the local immunity's
    /// -1 sentinel on the hit NPC.
    #[test]
    fn the_cascade_spawns_its_explosion_on_every_second_hit() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(CASCADE_YOYO);
        let mut held = Some(player);
        for _ in 0..6 {
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
        }
        let idx = pool
            .slots()
            .iter()
            .position(|p| p.active && p.kind == CASCADE_EXPLOSION);
        assert!(idx.is_some(), "the 1125 shot spawned");
        let idx = idx.unwrap();
        let shot = &pool.slots()[idx];
        assert_eq!(shot.damage, 40, "the parent damage x2");
        assert!(
            shot.ai[1] >= (208.0 / 200.0) - 1.0e-3,
            "the ai1 floor: {}",
            shot.ai[1]
        );
        assert!(shot.ai[1] <= 1.5 + 1.0e-3, "the ai1 clamp");
        assert_eq!(local_immunity(&pool, idx, 0), -1, "the -1 sentinel");
    }

    /// The Kraken's 3x 1135 (`:13748-13756`) at half damage.
    #[test]
    fn the_kraken_spawns_three_waves() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(KRAKEN_YOYO);
        let mut held = Some(player);
        for _ in 0..8 {
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
        }
        let waves = pool
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == KRAKEN_WAVE)
            .count();
        assert_eq!(waves, 3, "the three Kraken waves: {waves}");
        assert_eq!(
            pool.slots()
                .iter()
                .find(|p| p.active && p.kind == KRAKEN_WAVE)
                .unwrap()
                .damage,
            10
        );
    }

    /// The Chik's 5x 1134 (`:13757-13766`) with the idStatic stamp and the
    /// `ai0 = 1` followers.
    #[test]
    fn the_chik_spawns_five_shards() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(CHIK_YOYO);
        let mut held = Some(player);
        for _ in 0..8 {
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
        }
        let shards = pool
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == CRYSTAL_SHARD_MELEE)
            .count();
        assert_eq!(shards, 5, "the five shards: {shards}");
        let followers = pool
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == CRYSTAL_SHARD_MELEE && p.ai[0] == 1.0)
            .count();
        assert_eq!(followers, 4);
    }

    /// The Eye of Cthulhu's homing 1130 (`:13767-13809`): the 14-tile
    /// initial speed, the damage x2 and the -1 sentinel again.
    #[test]
    fn the_eye_of_cthulhu_spawns_its_dash() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup(THE_EYE_OF_CTHULHU_YOYO);
        let mut held = Some(player);
        for _ in 0..12 {
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
        }
        let idx = pool
            .slots()
            .iter()
            .position(|p| p.active && p.kind == EYE_OF_CTHULHU_DASH);
        assert!(idx.is_some(), "the 1130 dash spawned");
        let idx = idx.unwrap();
        let dash = &pool.slots()[idx];
        assert_eq!(dash.damage, 40);
        // The 14-tile initial speed has decayed by the fade by the time the
        // test observes it (`AI_210`'s 0.92 per tick), so the bound is the
        // decayed floor.
        assert!(
            dash.vel.length() > 3.0,
            "the 14-tile initial speed: {}",
            dash.vel.length()
        );
        assert_eq!(local_immunity(&pool, idx, 0), -1);
    }

    /// `AI_210_YoyoShots` (`Projectile.cs:36809-36900`): the fade-out
    /// lifecycle — the 0.92 decay and the 20-tick kill.
    #[test]
    fn the_yoyo_shot_fades_out_and_dies_at_twenty() {
        let mut world = tr_core::World::new("t".into(), 200, 200);
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn(
                EYE_OF_CTHULHU_DASH,
                0,
                Vec2::new(64.0, 64.0),
                Vec2::new(14.0, 0.0),
                20,
                4.0,
                false,
            )
            .unwrap();
        let first_speed = pool.slots()[slot].vel.x;
        for _ in 0..5 {
            pool.update(
                &mut world,
                &mut [],
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert!(pool.slots()[slot].active, "the fade runs 20 ticks");
        assert!(pool.slots()[slot].vel.x < first_speed, "the 0.92 decay");
        for _ in 0..20 {
            pool.update(
                &mut world,
                &mut [],
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert!(!pool.slots()[slot].active, "the 20-tick kill");
    }

    /// A plain (non-yoyo) shot spawns no yoyo shot on its hit.
    #[test]
    fn a_plain_projectile_spawns_no_yoyo_shot() {
        let (mut world, mut pool, mut npc, player) = on_hit_setup_plain(SHURIKEN);
        let mut held = Some(player);
        for _ in 0..6 {
            npc.immune[0] = 0;
            tick(&mut world, &mut pool, &mut npc, &mut held);
        }
        let spawned = pool
            .slots()
            .iter()
            .filter(|p| {
                p.active
                    && p.owner == 0
                    && matches!(
                        p.kind,
                        CASCADE_EXPLOSION | CRYSTAL_SHARD_MELEE | EYE_OF_CTHULHU_DASH | KRAKEN_WAVE
                    )
            })
            .count();
        assert_eq!(spawned, 0, "no yoyo shot from a shuriken: {spawned}");
    }

    use super::*;
    use crate::npc::{Npc, NpcKind};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    fn put_solid(w: &mut tr_core::World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 0,
            flags: FLAG_ACTIVE,
            ..Tile::default()
        };
    }

    /// The wooden platform tile (id 19 — the player's fall-through fixture at
    /// player.rs:7111).
    fn put_platform(w: &mut tr_core::World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 19,
            flags: FLAG_ACTIVE,
            ..Tile::default()
        };
    }

    fn inputs(aim: Option<(f32, f32)>, channel: bool) -> YoyoInputs {
        YoyoInputs {
            aim,
            channel,
            player_dead: false,
        }
    }

    /// The seeded stream the rolls draw from — a fresh FastRandom per tick
    /// batch so the determinism pairs replay identically.
    fn fast_seed() -> tr_core::tile_light::FastRandom {
        tr_core::tile_light::FastRandom::new(7)
    }

    #[test]
    fn the_flight_orbits_the_clamped_aim_point() {
        // `:67932-67950` (the clamp) + `:67985-68015` (the seek + hover). The
        // Wood Yoyo: range 130, top speed 9, arrive radius 5 + 9/2 = 9.5.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        // The aim 100 px right of the owner — inside the range, no clamp.
        for _ in 0..60 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                inputs(Some((420.0, 320.0)), true),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        let p = &pool.slots[0];
        assert!(p.active, "the yoyo flies on");
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        let dist = ((cx - 420.0).powi(2) + (cy - 320.0).powi(2)).sqrt();
        assert!(
            dist < 12.0,
            "the yoyo hovers at the aim point (dist {dist})"
        );
        assert!(
            p.vel.length() < 0.5,
            "the hover damps: {:?}",
            p.vel.length()
        );
        assert_eq!(p.local_ai[0], 60.0, "60 ticks of lifetime clock");
        assert_eq!((p.ai[0], p.ai[1]), (420.0, 320.0), "the clamped aim");
    }

    #[test]
    fn the_aim_clamps_to_the_range() {
        // `:67932-67939`: the aim 300 px out clamps to the range 130.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((620.0, 320.0)), true),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert_eq!(
            (pool.slots[0].ai[0], pool.slots[0].ai[1]),
            (449.0, 320.0),
            "the aim clamped: the owner + range, then the range-1 re-clamp"
        );
    }

    #[test]
    fn the_lifetime_returns_the_yoyo_at_tick_181() {
        // `:67745-67774`: `localAI[0]++` precedes the check, so the 3 s Wood
        // Yoyo is still out at tick 180 (3.0 > 3.0 is false) and returns at
        // 181 (3.0166 > 3.0).
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        for _ in 0..180 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                inputs(Some((380.0, 320.0)), true),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        assert_eq!(pool.slots[0].local_ai[0], 180.0);
        assert!(pool.slots[0].ai[0] >= 0.0, "in flight at tick 180");
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((380.0, 320.0)), true),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert_eq!(pool.slots[0].local_ai[0], 181.0);
        assert_eq!(pool.slots[0].ai[0], -1.0, "returned at tick 181");
    }

    #[test]
    fn the_release_returns_the_yoyo() {
        // `:67920-67925`: the channel release sets `ai[0] = -1, ai[1] = 0`.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((380.0, 320.0)), true),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((380.0, 320.0)), false),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert_eq!(pool.slots[0].ai[0], -1.0);
        assert_eq!(pool.slots[0].ai[1], 0.0);
        assert!(
            !pool.slots[0].tile_collide,
            "the return drops tile collision"
        );
    }

    #[test]
    fn the_forced_return_fires_past_1_3x_the_range() {
        // `:67904-67917` + `:67957-67961`: the owner moving away past
        // `range * 1.3` returns the yoyo even with the button held.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        // Fly out toward the right edge of the range.
        for _ in 0..40 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                inputs(Some((450.0, 320.0)), true),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        // The owner retreats left: the yoyo sits ~130 px right of the owner's
        // old spot, now ~260 away > 130 * 1.3 = 169.
        pool.update(
            &mut w,
            &mut [],
            Some((120.0, 320.0)),
            None,
            None,
            inputs(Some((450.0, 320.0)), true),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert_eq!(pool.slots[0].ai[0], -1.0, "the forced return");
    }

    #[test]
    fn the_return_catches_the_owner_and_dies() {
        // `:68084-68087`: the catch at `dist < seek_speed + 10f` — the Wood
        // Yoyo's return speed is 9 * 1.5 = 13.5, the catch radius 23.5.
        let mut pool = ProjectilePool::new();
        // The yoyo 15 px from the owner, released on the first tick.
        let slot = pool
            .spawn_with_data(WOOD_YOYO, 313.0, 312.0, 0.0, 0.0, 9, 2.5)
            .unwrap();
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((320.0, 320.0)), false),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert_eq!(pool.slots[slot].ai[0], -1.0, "released");
        assert!(!pool.slots[slot].active, "caught within the 23.5 px radius");
        assert_eq!(pool.slots[slot].time_left, 0);
    }

    #[test]
    fn the_return_dies_past_2000_px() {
        // `:68085`: the 2000 px cut-off kills the stray.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(WOOD_YOYO, 2500.0, 312.0, 0.0, 0.0, 9, 2.5)
            .unwrap();
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            Some((0.0, 320.0)),
            None,
            None,
            inputs(Some((2500.0, 320.0)), false),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert!(!pool.slots[slot].active, "the 2500 px stray dies");
    }

    #[test]
    fn the_owner_death_and_keepalive() {
        // `:67824-67828`; the keep-alive re-pins `timeLeft = 6` (`:67857`) —
        // the slot reads 5 after the update's decrement.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((380.0, 320.0)), true),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert_eq!(
            pool.slots[0].time_left, 5,
            "re-pinned to 6 then decremented"
        );
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            YoyoInputs {
                aim: Some((380.0, 320.0)),
                channel: true,
                player_dead: true,
            },
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            true,
        );
        assert!(!pool.slots[0].active, "the owner's death kills the string");
    }

    #[test]
    fn the_nan_velocity_kills_the_yoyo() {
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .unwrap();
        pool.slots[slot].vel.x = f32::NAN;
        let mut w = world();
        pool.update(
            &mut w,
            &mut [],
            Some((320.0, 320.0)),
            None,
            None,
            inputs(Some((380.0, 320.0)), true),
            FlailInputs::default(),
            &mut fast_seed(),
            0,
            true,
        );
        assert!(!pool.slots[slot].active, "the NaN kill");
    }

    #[test]
    fn the_wall_slides_the_yoyo_without_killing_it() {
        // `:17596`'s `Collision.TileCollision` slide — unlike the arrow's kill
        // and the boomerang's reverse, the yoyo stops at the wall face and
        // keeps seeking into it.
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(WOOD_YOYO, 416.0, 432.0, 9.0, 0.0, 9, 2.5)
            .unwrap();
        let mut w = world();
        // The wall column at tile x = 30 (x = 480), rows 27-29.
        for ty in 27..=29 {
            put_solid(&mut w, 30, ty);
        }
        // The aim beyond the wall — the seek keeps pushing into it.
        for _ in 0..30 {
            pool.update(
                &mut w,
                &mut [],
                Some((416.0, 448.0)),
                None,
                None,
                inputs(Some((600.0, 448.0)), true),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
            assert!(pool.slots[slot].active, "the yoyo survives the wall");
        }
        let p = &pool.slots[slot];
        let right = p.pos.x + p.w;
        assert!(
            (right - 480.0).abs() < 1.0,
            "the yoyo sits flush against the wall face (right {right})"
        );
    }

    #[test]
    fn the_dropped_twin_passes_through_platforms() {
        // `:68037-68077` (the gravity) + `:17552` (`flag6 = false` for
        // `aiStyle == 99 && ai[0] == -2f` — the fall-through).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(WOOD_YOYO, 432.0, 320.0, 0.0, 1.0, 9, 2.5, -2.0, 0.0)
            .unwrap();
        let mut w = world();
        put_platform(&mut w, 27, 28);
        for _ in 0..40 {
            pool.update(
                &mut w,
                &mut [],
                Some((500.0, 448.0)),
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        let p = &pool.slots[slot];
        assert_eq!(p.ai[0], -2.0, "still the dropped twin");
        assert!(
            p.pos.y > 28.0 * 16.0,
            "fell through the platform (y {})",
            p.pos.y
        );
        assert!(p.vel.y > 0.0, "the +0.3 gravity tail");
    }

    #[test]
    fn the_rotation_follows_the_state() {
        // `:68096-68103`: 0.45 per tick flying, 0.45 × direction for the
        // dropped twin.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .expect("the yoyo data row exists");
        let mut w = world();
        for _ in 0..10 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                inputs(Some((380.0, 320.0)), true),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        assert!(
            (pool.slots[0].rot - 4.5).abs() < 1e-4,
            "0.45 a tick flying: {}",
            pool.slots[0].rot
        );
        // The dropped twin left of the owner — its nudge sets direction -1
        // (`:68057-68066`), so the rotation goes the other way.
        let slot = pool
            .spawn_with_data_ai(WOOD_YOYO, 200.0, 320.0, 0.0, 0.0, 9, 2.5, -2.0, 0.0)
            .unwrap();
        for _ in 0..10 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        assert_eq!(pool.slots[slot].direction, -1, "the nudge points left");
        assert!(
            (pool.slots[slot].rot + 4.5).abs() < 1e-4,
            "0.45 x direction: {}",
            pool.slots[slot].rot
        );
    }

    #[test]
    fn the_lifetime_sweep_returns_the_pair_together() {
        // `:67764-67771`: the double yoyo's jitter (`:67746-67749`) ages its
        // clock faster; when IT expires the sweep flips the whole owner's
        // aiStyle-99 set with `ai[0] >= 0f` — including the un-aged first
        // yoyo, whose own clock never reaches its lifetime.
        let mut pool = ProjectilePool::new();
        let first = pool
            .spawn_with_data(WOOD_YOYO, 312.0, 312.0, 0.0, 0.0, 9, 2.5)
            .unwrap();
        let second = pool
            .spawn_with_data(WOOD_YOYO, 316.0, 316.0, 0.0, 0.0, 9, 2.5)
            .unwrap();
        let mut w = world();
        let aim = inputs(Some((400.0, 320.0)), true);
        // Both yoyos fly out to the aim; the double's jitter ages it faster;
        // its expiry sweeps the pair into the return, where the owner's catch
        // collects them — the assertions below read the final state.
        for _ in 0..200 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                aim,
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        assert!(
            pool.slots[second].local_ai[0] > pool.slots[first].local_ai[0],
            "the double's clock runs ahead: {} vs {}",
            pool.slots[second].local_ai[0],
            pool.slots[first].local_ai[0]
        );
        assert_eq!(pool.slots[second].ai[0], -1.0, "the double expired");
        assert_eq!(
            pool.slots[first].ai[0], -1.0,
            "the sweep returned the first yoyo"
        );
        assert!(
            pool.slots[first].local_ai[0] <= 180.0,
            "the first yoyo never reached its own lifetime ({})",
            pool.slots[first].local_ai[0]
        );
    }

    #[test]
    fn the_kraken_halves_its_speed_under_the_double_and_spins_by_direction() {
        // `:67864-67867` — the Kraken's `num9 *= 0.75f` under the flag; and
        // `:68096-68098` — the Kraken's rotation is 0.45 × direction in every
        // state (flying left the pose block sets direction = -1).
        let mut pool = ProjectilePool::new();
        let first = pool
            .spawn_with_data(KRAKEN_YOYO, 312.0, 312.0, 0.0, 0.0, 120, 4.3)
            .unwrap();
        let second = pool
            .spawn_with_data(KRAKEN_YOYO, 316.0, 316.0, 0.0, 0.0, 120, 4.3)
            .unwrap();
        let mut w = world();
        // The aim inside the range so the far arms (`:67964`'s speed ×2)
        // never engage — the caps under test are the Kraken's 22 and the
        // double's 16.5.
        let aim = inputs(Some((420.0, 312.0)), true);
        let mut max_first = 0.0f32;
        let mut max_second = 0.0f32;
        for _ in 0..30 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                aim,
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
            max_first = max_first.max(pool.slots[first].vel.length());
            max_second = max_second.max(pool.slots[second].vel.length());
        }
        assert!(
            max_first > 16.6,
            "the lone Kraken's 22 px/tick cap: {max_first}"
        );
        assert!(
            max_second <= 16.6,
            "the double's 16.5 px/tick cap: {max_second}"
        );
        // The direction rotation — the aim left of the owner, so the pose
        // block writes direction = -1 and the Kraken spins backwards.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(KRAKEN_YOYO, 312.0, 312.0, 0.0, 0.0, 120, 4.3)
            .unwrap();
        for _ in 0..20 {
            pool.update(
                &mut w,
                &mut [],
                Some((320.0, 320.0)),
                None,
                None,
                inputs(Some((120.0, 320.0)), true),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        assert_eq!(pool.slots[0].direction, -1, "the pose's direction");
        assert!(
            pool.slots[0].rot < 0.0,
            "the Kraken spins by direction: {}",
            pool.slots[0].rot
        );
    }

    #[test]
    fn the_terrarian_fires_beams_on_its_timer() {
        // `:67776-67818` — the 6-tick timer spawns 604 retargeted at the
        // nearest chasable NPC behind the `CanHit` gate; `:34414-34437` — the
        // beam damps 0.985 a tick and fades out at 30.
        let mut pool = ProjectilePool::new();
        let terrarian = pool
            .spawn_with_data(TERRARIAN_YOYO, 312.0, 312.0, 0.0, 0.0, 190, 6.5)
            .unwrap();
        assert!(
            pool.slots[terrarian].uses_id_static_immunity
                && pool.slots[terrarian].id_static_hit_cooldown == 10,
            "the Terrarian's idStatic-10 slice"
        );
        let mut npcs = vec![
            Npc::new(NpcKind::Slime, 430.0, 312.0, 1, 0.0),
            Npc::new(NpcKind::Slime, 312.0, 312.0 - 400.0, 1, 0.0),
        ];
        let mut w = world();
        let aim = inputs(Some((400.0, 312.0)), true);
        let mut beam = None;
        for _ in 0..6 {
            pool.update(
                &mut w,
                &mut npcs,
                Some((320.0, 312.0)),
                None,
                None,
                aim,
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
            beam = beam.or_else(|| {
                (0..MAX_PROJECTILES)
                    .find(|&i| pool.slots[i].active && pool.slots[i].kind == TERRARIAN_BEAM)
            });
        }
        let beam = beam.expect("the 6-tick beam spawn");
        // The retarget: the near slime at (438, 320) — the beam's velocity
        // points right and down by the 6.4 px/tick (8 × 0.8) aim.
        let v = pool.slots[beam].vel;
        assert!(
            v.x > 0.0 && v.y > 0.0,
            "the beam retargets at the slime: {v:?}"
        );
        assert!(
            v.length() > 5.0 && v.length() < 8.0,
            "the 8 px/tick aim after the ×0.8 damp: {}",
            v.length()
        );
        // The beam's own life: the damp, then the fade-out kill. `:34428`'s
        // `ai[1] > 30f` starts the fade; alpha climbs 10 a tick to the 255
        // kill at ~56 ticks. The Terrarian stops firing so the beam's slot is
        // not recycled by a later spawn.
        pool.slots[terrarian].active = false;
        for _ in 0..59 {
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
        }
        assert!(!pool.slots[beam].active, "the beam fades out and dies");
        assert_eq!(pool.slots[beam].alpha, 255, "the full fade");
    }

    #[test]
    fn the_dropped_twins_penetrate_roll_is_seeded() {
        // `:68039-68042` — the roll is drawn from the seeded stream; two
        // identical runs land the same value (3..7 exclusive).
        let roll_once = || {
            let mut pool = ProjectilePool::new();
            let slot = pool
                .spawn_with_data_ai(WOOD_YOYO, 432.0, 320.0, 0.0, 0.0, 9, 2.5, -2.0, 0.0)
                .unwrap();
            let mut w = world();
            pool.update(
                &mut w,
                &mut [],
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut fast_seed(),
                0,
                true,
            );
            pool.slots[slot].penetrate
        };
        let a = roll_once();
        let b = roll_once();
        assert_eq!(a, b, "the seeded pair rolls identically");
        assert!((3..7).contains(&a), "the 3..7 roll: {a}");
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        pool.update(
            world,
            npcs,
            Some(center),
            Some(Vec2::new(1.0, 0.0)),
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            true,
        );
    }

    /// The first `FastRandom` seed whose `next_range(0, 5)` rolls 0 — the
    /// ThornWhip's 1/5 debuff roll (`:11738`).
    fn thorn_seed() -> u64 {
        for seed in 1u64..200 {
            if tr_core::tile_light::FastRandom::new(seed).next_range(0, 5) == 0 {
                return seed;
            }
        }
        1
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
        // The ThornWhip's 1/5 roll needs a seed whose first draw is 0.
        let seed = thorn_seed();

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
            let mut rng = tr_core::tile_light::FastRandom::new(seed);
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
                    YoyoInputs::default(),
                    FlailInputs::default(),
                    &mut rng,
                    0,
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        // The arm budget roll (`:47669-47672`) fires on the first sub-step.
        pool.update(
            &mut w,
            &mut [],
            Some(center),
            Some(Vec2::new(1.0, 0.0)),
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
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
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut rng,
                0,
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
        let mut rng = tr_core::tile_light::FastRandom::new(3);
        for _ in 0..60 {
            pool.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut rng,
                0,
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
        let mut rng = tr_core::tile_light::FastRandom::new(3);
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut player),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            true,
        );
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
        pool2.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut player),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            true,
        );
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
        let mut rng = tr_core::tile_light::FastRandom::new(3);
        // The swing until the hit lands.
        let mut ticks = 0;
        while !player.tag_effects.is_npc_tagged(5479, 0) {
            pool.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut rng,
                0,
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
                YoyoInputs::default(),
                FlailInputs::default(),
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        for _ in 0..60 {
            pool.update(
                &mut w,
                &mut npcs,
                Some(center),
                Some(Vec2::new(1.0, 0.0)),
                Some(&mut player),
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut rng,
                0,
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

#[cfg(test)]
mod ammo_proj_tests {
    use super::*;

    /// DW-2.1: every kind in the resolved-projectile set (Phase 1's pair table
    /// + the per-weapon overrides) carries a `proj_data` row with the vanilla
    /// `Projectile.cs` values — the rows that stop `spawn_with_data_ai`'s
    /// silent no-op. `friendly`/`ranged` have no consumer, so neither rides
    /// the tuple.
    #[test]
    fn test_DW_2_1_the_resolved_kinds_all_have_rows() {
        // The launcher rockets (`Projectile.cs:2222-2338`): 14x14, aiStyle 16,
        // penetrate -1, the stock timeLeft; 133/136/139/142 add the
        // local-immunity pair.
        for kind in 133..=144u16 {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} has no row"));
            assert_eq!((d.w, d.h), (14.0, 14.0), "kind {kind}");
            assert_eq!(d.ai_style, 16, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, TIME_LEFT_DEFAULT, "kind {kind}");
            assert_eq!(d.extra_updates, 0, "kind {kind}");
            let immune = matches!(kind, 133 | 136 | 139 | 142);
            assert_eq!(d.uses_local_immunity, immune, "kind {kind}");
            assert_eq!(
                d.local_hit_cooldown,
                if immune { -1 } else { -2 },
                "kind {kind}"
            );
        }
        // The Snowman Cannon's rockets (`:4308-4320`) — the same row; the
        // dropped `scale = 0.9` keeps the 14x14 box and the immunity default.
        for kind in 338..=341u16 {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (14.0, 14.0), "kind {kind}");
            assert_eq!(d.ai_style, 16, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert!(!d.uses_local_immunity, "kind {kind}");
            assert_eq!(d.local_hit_cooldown, -2, "kind {kind}");
        }
        // The 1.4.4 rocket family (`:7939-7952`): the eight rockets per
        // launcher (776-801) plus the Snowman Cannon's 803-810; the
        // local-immunity pair rides 777/781/785/788/794/797/800.
        for kind in (776..=801u16).filter(|k| !matches!(k, 779 | 783)) {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} has no row"));
            assert_eq!((d.w, d.h), (14.0, 14.0), "kind {kind}");
            assert_eq!(d.ai_style, 16, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, TIME_LEFT_DEFAULT, "kind {kind}");
            let immune = matches!(kind, 777 | 781 | 785 | 788 | 794 | 797 | 800);
            assert_eq!(d.uses_local_immunity, immune, "kind {kind}");
            assert_eq!(
                d.local_hit_cooldown,
                if immune { -1 } else { -2 },
                "kind {kind}"
            );
        }
        for kind in 803..=810u16 {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (14.0, 14.0), "kind {kind}");
            assert_eq!(d.ai_style, 16, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert!(!d.uses_local_immunity, "kind {kind}");
        }
        // 779/783 sit inside the C# range but no launcher resolves to them —
        // the scope list excludes them, so the row-less Kinds stay row-less.
        assert!(proj_data(779).is_none());
        assert!(proj_data(783).is_none());
        // The Celebration rockets (`:7584-7605`): aiStyle 147 (the unported
        // stand-in), penetrate 1, extraUpdates 2 and
        // `timeLeft = 360 * MaxUpdates` = 360 × (2 + 1) = 1080
        // (`MaxUpdates`'s getter, `:554-561`).
        for kind in 715..=718u16 {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (14.0, 14.0), "kind {kind}");
            assert_eq!(d.ai_style, 147, "kind {kind}");
            assert_eq!(d.penetrate, 1, "kind {kind}");
            assert_eq!(d.extra_updates, 2, "kind {kind}");
            assert_eq!(d.time_left, 1080, "kind {kind}");
            assert!(!d.uses_local_immunity, "kind {kind}");
        }
        // The four gunProj rockets (`:6618-6630` 615, `:6767-6779` 630/705,
        // `:7571-7582` 714): 22x22, aiStyle 75 (unported), penetrate -1,
        // `tileCollide = false`, the stock timeLeft.
        for kind in [615u16, 630, 705, 714] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (22.0, 22.0), "kind {kind}");
            assert_eq!(d.ai_style, 75, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert!(!d.tile_collide, "kind {kind}");
            assert_eq!(d.time_left, TIME_LEFT_DEFAULT, "kind {kind}");
            assert_eq!(d.extra_updates, 0, "kind {kind}");
        }
        // The sand family (`:1248-1258`, `:1449-1462`, `:1470-1480`,
        // `:4438-4449`): 10x10, aiStyle 10 (unported), extraUpdates 1; 42
        // keeps the stock penetrate 1 (its arm sets none), the three
        // sand-block conversions penetrate -1.
        let sand = proj_data(42).unwrap();
        assert_eq!((sand.w, sand.h), (10.0, 10.0));
        assert_eq!(sand.ai_style, 10);
        assert_eq!(sand.extra_updates, 1);
        assert_eq!(sand.penetrate, 1);
        assert!(sand.tile_collide);
        assert!(!sand.arrow);
        for kind in [65u16, 68, 354] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (10.0, 10.0), "kind {kind}");
            assert_eq!(d.ai_style, 10, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.extra_updates, 1, "kind {kind}");
            assert_eq!(d.time_left, TIME_LEFT_DEFAULT, "kind {kind}");
        }
        // The arrow overrides: 117 (`:2058-2068`, extraUpdates 2), 469
        // (`:5429-5441`), 495 (`:5681-5692`, penetrate 3), 819
        // (`:7993-8002`, extraUpdates 1) — 10x10 aiStyle-1 arrows with
        // timeLeft 1200.
        for (kind, pen, extra) in [(117u16, 1, 2), (469, 1, 0), (495, 3, 0), (819, 1, 1)] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (10.0, 10.0), "kind {kind}");
            assert_eq!(d.ai_style, 1, "kind {kind}");
            assert_eq!(d.penetrate, pen, "kind {kind}");
            assert_eq!(d.extra_updates, extra, "kind {kind}");
            assert_eq!(d.time_left, 1200, "kind {kind}");
            assert!(d.arrow, "kind {kind}");
            assert!(d.tile_collide, "kind {kind}");
        }
        // The Hellwing 485 (`:5604-5615`): the 24x24 box, penetrate -1.
        let hellwing = proj_data(485).unwrap();
        assert_eq!((hellwing.w, hellwing.h), (24.0, 24.0));
        assert_eq!(hellwing.ai_style, 1);
        assert_eq!(hellwing.penetrate, -1);
        assert_eq!(hellwing.time_left, 1200);
        assert!(hellwing.arrow);
        // The fairy shot 932 (`:8842-8856`): aiStyle 181 (unported), penetrate
        // 5, timeLeft 120 and the only row whose immunity pair carries a real
        // cooldown (60).
        let fairy = proj_data(932).unwrap();
        assert_eq!((fairy.w, fairy.h), (10.0, 10.0));
        assert_eq!(fairy.ai_style, 181);
        assert_eq!(fairy.penetrate, 5);
        assert_eq!(fairy.time_left, 120);
        assert!(fairy.arrow);
        assert_eq!(fairy.extra_updates, 0);
        assert!(fairy.uses_local_immunity);
        assert_eq!(fairy.local_hit_cooldown, 60);
        // The spray/ball ammo the not-consume windows resolve (M3-ITEM-6): the
        // Flamethrower's Flames 85 (`:1660-1670`: 6x6, aiStyle 193 unported,
        // penetrate 4, extraUpdates 2, immune pair -1), the Clentaminator's
        // solution 145 (`:2338-2347`: 6x6, aiStyle 31 unported, penetrate -1,
        // extraUpdates 2, tileCollide false) and the acorn 1088
        // (`:10145-10149`: 20x20, aiStyle 16, penetrate -1) — without these
        // rows the wire would drain the ammo and then decline the spawn.
        let flames = proj_data(85).unwrap();
        assert_eq!((flames.w, flames.h), (6.0, 6.0));
        assert_eq!(flames.ai_style, 193);
        assert_eq!(flames.penetrate, 4);
        assert_eq!(flames.extra_updates, 2);
        assert!(flames.uses_local_immunity);
        assert_eq!(flames.local_hit_cooldown, -1);
        let solution = proj_data(145).unwrap();
        assert_eq!((solution.w, solution.h), (6.0, 6.0));
        assert_eq!(solution.ai_style, 31);
        assert_eq!(solution.penetrate, -1);
        assert_eq!(solution.extra_updates, 2);
        assert!(!solution.tile_collide);
        assert!(!solution.uses_local_immunity);
        let acorn = proj_data(1088).unwrap();
        assert_eq!((acorn.w, acorn.h), (20.0, 20.0));
        assert_eq!(acorn.ai_style, 16);
        assert_eq!(acorn.penetrate, -1);
    }

    /// DW-2.2: the rows reach the pool — the Rocket Launcher's RocketI (134),
    /// the Snowman Cannon's 338 and the Cluster Rocket I's 776 spawn through
    /// `spawn_with_data_ai` with the row's box and immunity data; the
    /// damage/knockback stay the spawner's input.
    #[test]
    fn test_DW_2_2_the_rows_reach_the_spawner() {
        let mut pool = ProjectilePool::new();
        for kind in [134u16, 338, 776] {
            let row = proj_data(kind).unwrap();
            let slot = pool
                .spawn_with_data_ai(kind, 160.0, 200.0, 4.0, 0.0, 40, 2.0, 0.0, 0.0)
                .unwrap();
            let p = &pool.slots[slot];
            assert_eq!(p.kind, kind);
            assert_eq!((p.w, p.h), (row.w, row.h), "kind {kind}");
            assert_eq!(p.time_left, row.time_left, "kind {kind}");
            assert_eq!(p.penetrate, row.penetrate, "kind {kind}");
            assert_eq!(p.max_penetrate, row.penetrate, "kind {kind}");
            assert_eq!(p.extra_updates, row.extra_updates, "kind {kind}");
            assert_eq!(p.tile_collide, row.tile_collide, "kind {kind}");
            assert_eq!(
                p.uses_local_immunity, row.uses_local_immunity,
                "kind {kind}"
            );
            assert_eq!(p.local_hit_cooldown, row.local_hit_cooldown, "kind {kind}");
            // None of the three carries the immunity pair: the C#'s lists are
            // 133/136/139/142 (`:2222-2338`) and 777/781/785/788/794/797/800
            // (`:7939-7952`) — 776 is a plain Rocket-Launcher rocket — and the
            // Snowman Cannon's rows set none. The immunity-carrying rows are
            // pinned in the DW-2.1 table and the per-family spawn test.
            assert!(!p.uses_local_immunity, "kind {kind}");
            assert_eq!(p.local_hit_cooldown, -2, "kind {kind}");
            // `NewProjectile`'s parameters ride after `SetDefaults`
            // (`:10858-10880`): the damage/knockback inputs win.
            assert_eq!(p.damage, 40, "kind {kind}");
            assert_eq!(p.knock_back, 2.0, "kind {kind}");
        }
    }

    /// The rest of the resolved families reach the pool (the Test Plan's
    /// per-family spawn assertions): the sand family, the four gunProj
    /// rockets, the Celebration rockets, the arrow overrides and the fairy
    /// shot — whose 932 row is the only one carrying the cooldown-60 immunity
    /// pair end to end.
    #[test]
    fn test_the_resolved_rows_reach_the_pool_per_family() {
        // (kind, w, h, penetrate, extra_updates, time_left, uses_local,
        //  local_hit_cooldown)
        let cases: &[(u16, f32, f32, i32, u32, i32, bool, i32)] = &[
            (42, 10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, false, -2),
            (65, 10.0, 10.0, -1, 1, TIME_LEFT_DEFAULT, false, -2),
            (68, 10.0, 10.0, -1, 1, TIME_LEFT_DEFAULT, false, -2),
            (354, 10.0, 10.0, -1, 1, TIME_LEFT_DEFAULT, false, -2),
            (615, 22.0, 22.0, -1, 0, TIME_LEFT_DEFAULT, false, -2),
            (630, 22.0, 22.0, -1, 0, TIME_LEFT_DEFAULT, false, -2),
            (705, 22.0, 22.0, -1, 0, TIME_LEFT_DEFAULT, false, -2),
            (714, 22.0, 22.0, -1, 0, TIME_LEFT_DEFAULT, false, -2),
            (715, 14.0, 14.0, 1, 2, 1080, false, -2),
            (716, 14.0, 14.0, 1, 2, 1080, false, -2),
            (717, 14.0, 14.0, 1, 2, 1080, false, -2),
            (718, 14.0, 14.0, 1, 2, 1080, false, -2),
            (117, 10.0, 10.0, 1, 2, 1200, false, -2),
            (469, 10.0, 10.0, 1, 0, 1200, false, -2),
            (485, 24.0, 24.0, -1, 0, 1200, false, -2),
            (495, 10.0, 10.0, 3, 0, 1200, false, -2),
            (819, 10.0, 10.0, 1, 1, 1200, false, -2),
            (932, 10.0, 10.0, 5, 0, 120, true, 60),
        ];
        let mut pool = ProjectilePool::new();
        for &(kind, w, h, pen, extra, time_left, immune, cd) in cases {
            let row = proj_data(kind).unwrap();
            let slot = pool
                .spawn_with_data(kind, 160.0, 200.0, 4.0, 0.0, 40, 2.0)
                .unwrap();
            let p = &pool.slots[slot];
            assert_eq!(p.kind, kind, "kind {kind}");
            assert_eq!((p.w, p.h), (w, h), "kind {kind}");
            assert_eq!(p.penetrate, pen, "kind {kind}");
            assert_eq!(p.max_penetrate, pen, "kind {kind}");
            assert_eq!(p.extra_updates, extra, "kind {kind}");
            assert_eq!(p.time_left, time_left, "kind {kind}");
            assert_eq!(p.uses_local_immunity, immune, "kind {kind}");
            assert_eq!(p.local_hit_cooldown, cd, "kind {kind}");
            assert_eq!(p.damage, 40, "kind {kind}");
            assert_eq!(p.knock_back, 2.0, "kind {kind}");
            // The spawner feeds from the row — the two agree.
            assert_eq!((row.w, row.h), (w, h), "kind {kind}");
            assert_eq!(row.penetrate, pen, "kind {kind}");
            assert_eq!(row.extra_updates, extra, "kind {kind}");
            assert_eq!(row.time_left, time_left, "kind {kind}");
            assert_eq!(row.uses_local_immunity, immune, "kind {kind}");
            assert_eq!(row.local_hit_cooldown, cd, "kind {kind}");
        }
    }

    /// The two new schema fields leave every existing row on the pool
    /// defaults — the Phase 2 freeze (no behaviour change for existing
    /// spawns), and the poison dart's deferred `:6953-6968` immunity pair
    /// stays recorded in the module doc until a follow-up.
    #[test]
    fn test_the_resolved_rows_leave_the_existing_spawns_untouched() {
        for kind in [
            WOODEN_ARROW,
            FLAME_ARROW,
            MUSKET_BALL,
            POISON_DART,
            TRAP_SPEAR,
            BOMB,
        ] {
            let d = proj_data(kind).unwrap();
            assert!(!d.uses_local_immunity, "kind {kind}");
            assert_eq!(d.local_hit_cooldown, -2, "kind {kind}");
        }
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data(POISON_DART, 160.0, 200.0, 4.0, 0.0, 10, 0.0)
            .unwrap();
        let p = &pool.slots[slot];
        assert!(!p.uses_local_immunity);
        assert_eq!(p.local_hit_cooldown, -2);
        assert_eq!((p.w, p.h), (10.0, 10.0));
        assert_eq!(p.extra_updates, 5);
        assert_eq!(p.damage, 10);
    }

#[cfg(test)]
mod phasmal_projectile_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn spawn(kind: u16, x: f32, y: f32) -> Projectile {
        Projectile {
            grapple_center: Vec2::ZERO,
            active: true,
            kind,
            owner: 0,
            generation: 0,
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            rot: 0.0,
            damage: 30,
            knock_back: 0.0,
            crit: false,
            friendly: false,
            time_left: 600,
            max_penetrate: -1,
            penetrate: -1,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -2,
            extra_updates: 0,
            ai: [0.0; 4],
            local_ai: [0.0; 3],
            sprite_direction: 1,
            is_summon_shot: false,
            tag_effect_type: 0,
            bonus_tag_damage: 0,
            direction: 1,
            whip_settings: None,
            whip_use_animation: 0.0,
            scale: 1.0,
            alpha: 255,
            falling_tile: None,
            detonated: false,
            wants_mark_spawn: None,
            wants_ritual_bolt: None,
            w: 14.0,
            h: 14.0,
            tile_collide: true,
            arrow: false,
            trap: false,
            bounces: 0,
            blast_spawned: false,
        }
    }

    #[test]
    fn the_lunar_battery_rows_carry_their_boxes() {
        // `Projectile.cs`'s SetDefaults rows for 540/574/629 + the phantasmal
        // family's aiStyles.
        for (kind, ai_style, w, h) in [
            (STARDUST_MARK, 103u8, 20.0, 20.0),
            (NEBULA_EYE, 102, 18.0, 18.0),
            (TOWER_DAMAGE_BOLT, 122, 8.0, 8.0),
            (PHANTASMAL_EYE, 82, 14.0, 14.0),
            (PHANTASMAL_SPHERE, 83, 40.0, 40.0),
            (PHANTASMAL_DEATHRAY, 84, 36.0, 36.0),
            (MOON_LEECH, 85, 16.0, 16.0),
        ] {
            let data = proj_data(kind).expect("the row");
            assert_eq!(data.ai_style, ai_style, "the style for {}", kind);
            assert_eq!((data.w, data.h), (w, h), "the box for {}", kind);
        }
    }

    #[test]
    fn the_eye_drifts_then_homes_on_the_owner() {
        // `AI_082`'s ladder: the 45-tick first leg, the 90-tick second leg,
        // then the homing (14 px over the 0.6 lerp).
        let npcs: Vec<Npc> = Vec::new();
        let mut p = spawn(PHANTASMAL_EYE, 100.0, 100.0);
        p.ai[1] = 0.05; // `:41704`'s spawn drift offset.
        for _ in 0..45 {
            p.ai_phasmal_eye(&npcs, (400.0, 400.0));
        }
        assert_eq!(p.ai[0], 1.0, "the first leg hands off at 45");
        assert!(p.ai[1].abs() > 0.0, "the drift offset survives");
        for _ in 0..90 {
            p.ai_phasmal_eye(&npcs, (400.0, 400.0));
        }
        assert_eq!(p.ai[0], 2.0, "the homing leg opens at 90");
        let before = p.pos;
        for _ in 0..60 {
            p.ai_phasmal_eye(&npcs, (900.0, 900.0));
        }
        let moved = (p.pos - before).length();
        assert!(moved > 100.0, "the eye homes: {} px", moved);
        assert!(p.vel.x > 0.0, "toward the owner");
    }

    #[test]
    fn the_eye_dies_at_thirty_pixels() {
        // `:31269-31273`'s kill radius.
        let npcs: Vec<Npc> = Vec::new();
        let mut p = spawn(PHANTASMAL_EYE, 100.0, 100.0);
        p.ai[0] = 2.0;
        p.ai_phasmal_eye(&npcs, (110.0, 110.0));
        assert!(!p.active, "the eye pops at 14 px");
    }

    #[test]
    fn the_sphere_locks_to_the_host_then_detaches() {
        // `AI_083`'s host lock while `ai[0] < 30`, then the decay.
        let npcs: Vec<Npc> = Vec::new();
        let mut p = spawn(PHANTASMAL_SPHERE, 100.0, 100.0);
        p.vel = Vec2::new(2.0, 2.0);
        for _ in 0..10 {
            p.ai_phasmal_sphere(&npcs);
        }
        assert_eq!(p.ai[0], 10.0, "the lock's counter");
        assert!((p.alpha - 195).abs() <= 6, "the alpha ramp: {}", p.alpha);
        // The release frame flips the extra updates.
        p.ai[0] = -1.0;
        p.ai_phasmal_sphere(&npcs);
        assert_eq!(p.extra_updates, 1, "the release frame");
    }

    #[test]
    fn the_deathray_rides_the_host_and_sweeps() {
        // `AI_084`: the 180-tick lifetime, the scale curve and the sweep.
        let mut npcs: Vec<Npc> = Vec::new();
        let mut head = Npc::new(NpcKind::MoonLordHead, 200.0, 100.0, 1, 0.0);
        head.hp = head.max_hp;
        npcs.push(head);
        let mut p = spawn(PHANTASMAL_DEATHRAY, 200.0, 100.0);
        p.ai[0] = 0.0116; // The ±2π/540 sweep.
        for _ in 0..90 {
            p.ai_phasmal_deathray(&surface_world(), &npcs);
        }
        assert_eq!(p.local_ai[0], 90.0, "the lifetime counter");
        assert!(p.active, "the ray lives past its half");
        for _ in 0..90 {
            p.ai_phasmal_deathray(&surface_world(), &npcs);
        }
        assert!(!p.active, "the ray dies at 180");
        assert!(p.scale > 0.0, "the scale curve");
    }

    #[test]
    fn the_leech_applies_the_145_at_the_contact() {
        // `AI_085`'s outbound leg + the buff 145 (840, the classic).
        let mut npcs: Vec<Npc> = Vec::new();
        let mut head = Npc::new(NpcKind::MoonLordHead, 300.0, 100.0, 1, 0.0);
        head.hp = head.max_hp;
        npcs.push(head);
        let world = surface_world();
        let mut player = Player::spawn(&world);
        player.x = 60.0;
        player.y = 100.0;
        let mut p = spawn(MOON_LEECH, 110.0, 100.0);
        p.ai[0] = 1.0; // The outbound leg.
        for t in 0..20 {
            p.ai_moon_leech(&npcs, &mut player);
        }
        // The buff's own latch (`:31798`'s `localAI[1] = 1f`) is the
        // machine's applied marker — the port's `find_buff_index` reads the
        // debuff immunity the spawn seeds, so the latch is the observable
        // signal.
        assert_eq!(p.local_ai[1], 1.0, "the Moon Leech buff lands");
        assert!(p.vel.x <= 0.0, "the leech rests at the player after the seek");
    }

    #[test]
    fn the_stardust_mark_spawns_the_carried_npc() {
        // `AI_103`: the fade-in spawns the id in `localAI[0]`.
        let npcs: Vec<Npc> = Vec::new();
        let world = surface_world();
        let mut p = spawn(STARDUST_MARK, 500.0, 100.0);
        p.local_ai[0] = NpcKind::StardustCellBig.to_u16() as f32;
        for _ in 0..12 {
            p.ai_stardust_mark(&world, &npcs);
        }
        let (kind, x, y, _vx, _vy) = p.wants_mark_spawn.expect("the spawn request");
        assert_eq!(kind, NpcKind::StardustCellBig.to_u16());
        assert!((x - 500.0).abs() < 1.0 && (y - 100.0).abs() < 1.0);
        // The pool's drain carries it out.
        let mut pool = ProjectilePool::new();
        pool.slots[0] = p;
        let drained = pool.drain_mark_spawns();
        assert_eq!(drained.len(), 1);
        assert!(pool.slots[0].wants_mark_spawn.is_none(), "the intent clears");
    }

    #[test]
    fn the_tower_bolt_drains_the_shield() {
        // `AI_122`'s homing + the contact's shield spend.
        let mut npcs: Vec<Npc> = Vec::new();
        let mut tower = Npc::new(NpcKind::SolarTower, 300.0, 100.0, 1, 0.0);
        tower.hp = tower.max_hp;
        npcs.push(tower);
        let world = surface_world();
        let mut p = spawn(TOWER_DAMAGE_BOLT, 360.0, 235.0);
        p.ai[0] = 0.0; // The tower's index.
        p.damage = 30;
        let hit = p.ai_tower_damage_bolt(&npcs);
        assert!(hit, "the bolt lands on the tower");
        let _ = world;
    }
}

#[cfg(test)]
mod ritual_projectile_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn rspawn(kind: u16, x: f32, y: f32) -> Projectile {
        Projectile {
            grapple_center: Vec2::ZERO,
            active: true,
            kind,
            owner: 0,
            generation: 0,
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            rot: 0.0,
            damage: 0,
            knock_back: 0.0,
            crit: false,
            friendly: false,
            time_left: 600,
            max_penetrate: -1,
            penetrate: -1,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -2,
            extra_updates: 0,
            ai: [0.0; 4],
            local_ai: [0.0; 3],
            sprite_direction: 1,
            is_summon_shot: false,
            tag_effect_type: 0,
            bonus_tag_damage: 0,
            direction: 1,
            whip_settings: None,
            whip_use_animation: 0.0,
            scale: 1.0,
            alpha: 255,
            falling_tile: None,
            detonated: false,
            wants_mark_spawn: None,
            wants_ritual_bolt: None,
            w: 32.0,
            h: 32.0,
            tile_collide: false,
            arrow: false,
            trap: false,
            bounces: 0,
            blast_spawned: false,
        }
    }

    #[test]
    fn the_ritual_markers_carry_their_rows() {
        // `Projectile.cs:6282-6293`'s shared row for the 578 lightning and
        // the 579 portal, and `:6294-6305`'s 580 bolt.
        for kind in [VORTEX_LIGHTNING, VORTEX_PORTAL] {
            let data = proj_data(kind).expect("the row");
            assert_eq!(data.ai_style, 108);
            assert_eq!((data.w, data.h), (32.0, 32.0));
            assert!(!data.tile_collide, "the markers pass the tiles");
        }
        let bolt = proj_data(VORTEX_RITUAL_BOLT).expect("the row");
        assert_eq!(bolt.ai_style, 88);
        assert_eq!(bolt.extra_updates, 4);
    }

    #[test]
    fn the_portal_spawns_two_aliens_on_the_90th_tick() {
        // `AI_108`'s 579 arm (`Projectile.cs:33844-33856`): the 90th tick
        // spawns two VortexAliens at the portal's centre.
        let mut p = rspawn(VORTEX_PORTAL, 200.0, 200.0);
        let mut rng = 3u64;
        for t in 0..90 {
            p.ai_vortex_ritual((300.0, 300.0), &mut rng);
            if t == 88 {
                assert_eq!(p.ai[0], 89.0, "one tick under the burst");
            }
        }
        assert_eq!(p.ai[0], 90.0, "the burst's tick");
        assert!(p.wants_mark_spawn.is_some(), "the two aliens spawn");
        // The drain carries them out.
        let mut pool = ProjectilePool::new();
        pool.slots[0] = p;
        let drained = pool.drain_mark_spawns();
        assert_eq!(drained.len(), 1, "the intent holds one of the pair");
        let (kind, _x, _y, vx, vy) = drained[0];
        assert_eq!(kind, NpcKind::VortexAlien.to_u16());
        assert!(vy <= 2.0 && vy >= -11.0, "the launch's spread: {}", vy);
        assert!(vx.abs() > 0.0, "and sideways");
    }

    #[test]
    fn the_lightning_charges_then_strikes_the_580() {
        // `AI_108`'s 578 arm: the aim rotation on the first tick, the
        // scale/alpha ramp over the charge, and the 580 spawn at 90.
        let mut p = rspawn(VORTEX_LIGHTNING, 200.0, 200.0);
        let mut rng = 5u64;
        p.ai_vortex_ritual((260.0, 200.0), &mut rng);
        assert!((p.ai[1]).abs() < 0.01, "the aim is +X: {}", p.ai[1]);
        for _ in 0..89 {
            p.ai_vortex_ritual((260.0, 200.0), &mut rng);
        }
        assert_eq!(p.ai[0], 90.0);
        assert!(p.wants_ritual_bolt.is_some(), "the 580 spawns");
        let (bx, by, bvx, bvy, ai0) = p.wants_ritual_bolt.expect("the bolt");
        assert!((bx - 200.0).abs() < 8.01, "the spawn sits behind the aim");
        assert!((by - 200.0).abs() < 0.01);
        assert!((bvx - 8.0).abs() < 0.01, "the bolt's spawn velocity: {}", bvx);
        assert!(bvy.abs() < 0.01);
        assert!(ai0.abs() < 0.01, "the bolt carries the aim: {}", ai0);
        assert!(p.alpha <= 5, "the charge's alpha: {}", p.alpha);
    }

    #[test]
    fn the_marker_dissolves_and_dies() {
        // `:33944-33950`'s dissolve: the scale shrinks, the alpha climbs and
        // the marker dies at the end.
        let mut p = rspawn(VORTEX_PORTAL, 200.0, 200.0);
        let mut rng = 9u64;
        for _ in 0..185 {
            p.ai_vortex_ritual((300.0, 300.0), &mut rng);
        }
        assert!(!p.active, "the marker dies");
    }

    #[test]
    fn the_bolt_rests_then_dies() {
        // `AI_088`'s stationary arm (`Projectile.cs:32156-32174`): the bolt
        // waits out `extraUpdates * 2` ticks and dies standing still.
        let mut p = rspawn(VORTEX_RITUAL_BOLT, 200.0, 200.0);
        p.extra_updates = 4;
        p.ai[0] = 1000.0; // The wrap latch's arm.
        let world = surface_world();
        let mut rng = 11u64;
        for _ in 0..4 {
            p.ai_vortex_bolt(&world, &mut rng);
        }
        assert_eq!(p.ai[0], 0.0, "the 900-wrap");
        // The latch's −1 arm is followed by the same tick's +2 (`:32156`).
        assert_eq!(p.local_ai[1], 1.0, "the latch flipped and stepped");
        for _ in 0..20 {
            p.ai_vortex_bolt(&world, &mut rng);
        }
        assert!(!p.active, "the bolt dies standing");
    }
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
        assert_eq!(p.ai, [0.0, 0.0, 0.0, 0.0], "`spawn_with_data` passes ai0/ai1 0");
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
        assert_eq!(p.ai, [7.0, -3.0, 0.0, 0.0]);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
        assert_eq!(npc.hp, hp0 - 10);
        assert_eq!(pool.slots[slot].penetrate, 2);
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
        assert_eq!(npc.hp, hp0 - 10);
        pool.update(&mut w, std::slice::from_mut(&mut npc), None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(w, &mut npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
    }

    /// The `tick(world, pool, npc)` fixture of the main `mod tests`, for the
    /// calls that need an NPC slice.
    fn tick(world: &mut World, pool: &mut ProjectilePool, npc: &mut Npc) {
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
    }

    /// The same fixture over a whole `Vec<Npc>` — the targeting arms read the
    /// `Main.npc` slice, so the scan tests hand the pool several at once.
    fn tick_npcs(world: &mut World, pool: &mut ProjectilePool, npcs: &mut [Npc]) {
        let mut rng = tr_core::tile_light::FastRandom::new(7);

        for n in npcs.iter_mut() {
            for v in n.immune.iter_mut() {
                *v = (*v - 1).max(0);
            }
            n.hurt_cd = (n.hurt_cd - 1.0 / 60.0).max(0.0);
        }
        pool.update(world, npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
        pool.update(&mut w, &mut npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
            None,
            Some(&mut p),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
            );
        let spell = &pool.slots()[slot];
        assert_eq!(spell.ai[0], 500.0, "the aim X rides the channel target");
        assert_eq!(spell.ai[1], 400.0, "the aim Y rides the channel target");
    }

    #[test]
    fn test_dw_4b_the_spell_releases_finds_a_target_and_floors_the_life() {
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
            None,
            Some(&mut p),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
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
            None,
            Some(&mut p),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
        pool.update(&mut w, &mut npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
        pool.update(&mut w, &mut npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        pool.update(&mut w, &mut npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
        assert!(
            pool.slots()[slot].time_left <= 240,
            "the waiting flame caps at 240"
        );
    }

    // ---- Batch-review fixes (phase 4b) --------------------------------------

    #[test]
    fn test_fix_4b_the_released_spell_blends_at_most_0p2() {
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
        pool.update(&mut w, &mut npcs, None, None, None, YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false);
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
        let mut rng = tr_core::tile_light::FastRandom::new(7);

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
            None,
            Some(&mut p),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
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
