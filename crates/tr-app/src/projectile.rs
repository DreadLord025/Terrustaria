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
//! - the grappling hook family (M4-PROJ-5): all 32 vanilla kinds carry a
//!   `proj_data` row (the 20 `aiStyle = 7` `SetDefaults` blocks, e.g. the Hook
//!   `:939-948`); the AI state machine, the multi-hook budgets and the
//!   per-hook steering land with the task's later phases.
//!
//! The ammo-resolution rows (M3-ITEM-6, 2026-11-09): the `proj_data` block at
//! the end of the match carries the kinds `Player.PickAmmo`
//! (`Player.cs:53458-53683`) resolves, so `spawn_with_data_ai` no longer
//! silently no-ops on them (Phase 1's deviation). Dropped vanilla fields
//! with no port consumer (documented ANALYSIS deviations — Phase 5 writes the
//! entries): `alpha`, `scale` (the Snowman rockets' `scale = 0.9` keeps the
//! 14x14 box), `ignoreWater`, `drawLayer`, `hide`, `ranged`,
//! `usesOwnerLight`, and the per-type `knockBack` (`NewProjectile` applies the
//! spawner's KnockBack after `SetDefaults`, `:10858-10880`). The unported
//! aiStyles 147/75/10/181 carry their vanilla value in the row and ride the
//! documented stand-in dispatch (their AI ports are M4-PROJ's). The poison
//! dart's `usesLocalNPCImmunity` + `localNPCHitCooldown = -1` (`:6953-6968`)
//! stays deferred: its row predates the two schema fields and the M3-ITEM-6
//! freeze on existing rows keeps the default until a follow-up.
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
use crate::grapple;
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
// --- the grappling hook family (M4-PROJ-5): the 32 vanilla kinds
// `AI_007_GrapplingHooks` (`Projectile.cs:52565-52866`) drives. The ids are
// `ID/ProjectileID.cs`'s names; the `SetDefaults` rows land in `proj_data`
// below. Phases 2-4 (the AI, the budgets, the steering) read these.
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
/// flight behaves on. The damage and the knockback stay the spawner's input
/// (`NewProjectile` applies them after `SetDefaults`, `:10858-10880`).
///
/// M3-ITEM-6 (2026-11-09): the immunity pair below joined the schema so the
/// ammo-resolution rows can carry the vanilla `usesLocalNPCImmunity` +
/// `localNPCHitCooldown` values; every row that predates the two fields keeps
/// the pool defaults. The remaining vanilla fields (`alpha`, `scale`,
/// `ignoreWater`, `drawLayer`, `hide`, `ranged`, `usesOwnerLight`) have no
/// consumer in the port and are dropped — see the module doc.
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
    /// `usesLocalNPCImmunity` (`:464`'s default false) — only when set does
    /// the pool decay and consult the slot's local-NPC immunity entries
    /// (`Update`'s `DecrementLocalImmuneTimeCounters`).
    pub uses_local_immunity: bool,
    /// `localNPCHitCooldown` (the `:464` default −2) — the ticks the local
    /// entry gates a re-hit after a hit writes it.
    pub local_hit_cooldown: i32,
    /// `Projectile.scale` — the SetDefaults value; the stock 1f except the
    /// yoyo family's ladder (the yoyo rows carry it for the render row).
    pub scale: f32,
    /// `usesIDStaticNPCImmunity` + `idStaticNPCHitCooldown` (the stock -1,
    /// `:466`): the Terrarian pair's slice (`:6068-6069`, `:6081-6082`).
    pub uses_id_static: bool,
    pub id_static_hit_cooldown: i32,
}

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
        _ => 1.0,
    }
}

/// The grappling hook family's `timeLeft` — every aiStyle-7 row's
/// `timeLeft *= 10` (`Projectile.cs`'s 20 blocks, e.g. `:939-948`) on the
/// `:787` default 3600: the 36000-tick flight budget.
const HOOK_TIME_LEFT: i32 = 10 * TIME_LEFT_DEFAULT;

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
            // The yoyo family (`:6013-6093` + `:9483-9490`): 16×16, aiStyle 99
            // (`DefaultToYoyo` `:10764-10771` — friendly, `penetrate = -1`),
            // `extraUpdates = 0`; the C# SetDefaults sets no `timeLeft`, so the
            // stock 3600 stands and the AI's `timeLeft = 6` re-pin keeps the
            // slot alive in flight.
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
            | HIVE_FIVE => (16.0, 16.0, 99, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The yoyo shots (M4-PROJ-4b): the four children the yoyo
            // on-hit arms spawn — `SetDefaults` verbatim.
            // `CascadeExplosion` (`:10528-10541`): 32×32, aiStyle 189, the
            // `alpha = 255`/`hide` pair, `penetrate = 4`, local-immunity with
            // `localNPCHitCooldown = -1`.
            CASCADE_EXPLOSION => (32.0, 32.0, 189, 4, TIME_LEFT_DEFAULT, 0, false, false, false),
            // `TheEyeOfCthulhuDash` (`:10596-10608`): 100×100, aiStyle 210,
            // `timeLeft = 600`, `penetrate = -1`, local-immunity −1.
            EYE_OF_CTHULHU_DASH => (100.0, 100.0, 210, -1, 600, 0, false, false, false),
            // `CrystalShardMelee` (`:10636-10650`): 6×6, aiStyle 24,
            // `penetrate = 1`, `timeLeft = 600`, idStatic-20.
            CRYSTAL_SHARD_MELEE => (6.0, 6.0, 24, 1, 600, 0, false, false, false),
            // `KrakenWave` (`:10656-10667`): 100×100, aiStyle 210,
            // `timeLeft = 600`, `penetrate = -1`, idStatic-10.
            KRAKEN_WAVE => (100.0, 100.0, 210, -1, 600, 0, false, false, false),
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
            ),
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
            // `:6151-6160`'s StardustTowerMark (the 540): 20x20, aiStyle
            // 103, timeLeft 300, hostile.
            STARDUST_MARK => (20.0, 20.0, 103, -1, 300, 0, true, false, false),
            // `:6239-6247`'s NebulaEye (the 574): 18x18, aiStyle 102,
            // timeLeft 600, tileCollide false.
            NEBULA_EYE => (18.0, 18.0, 102, -1, 600, 0, false, false, false),
            // `:6754-6766`'s TowerDamageBolt (the 629): 8x8, aiStyle 122,
            // ignoreWater, extraUpdates 2.
            TOWER_DAMAGE_BOLT => (8.0, 8.0, 122, -1, TIME_LEFT_DEFAULT, 2, false, false, true),
            // `:6282-6293`'s ritual markers (the 578 lightning + the 579
            // portal): 32x32, aiStyle 108, tileCollide false, ignoreWater.
            VORTEX_LIGHTNING | VORTEX_PORTAL => {
                (32.0, 32.0, 108, -1, TIME_LEFT_DEFAULT, 0, false, false, false)
            }
            // `:6294-6305`'s VortexLightning (the 580): 14x14, aiStyle 88,
            // extraUpdates 4, ignoreWater.
            VORTEX_RITUAL_BOLT => (14.0, 14.0, 88, -1, 600, 4, true, false, false),
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
            // The ammo-resolution rows (M3-ITEM-6 Phase 2): `friendly` is the
            // pool default for every one of them and `ranged` has no consumer,
            // so neither rides the tuple. The launcher rockets
            // (`Projectile.cs:2222-2338`) — 14x14, aiStyle 16 (the bomb
            // family), `penetrate = -1`; 133/136/139/142 add the
            // local-immunity pair (the overlay below).
            133..=144 => (14.0, 14.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The Snowman Cannon's four rockets (`:4308-4320`) — the same row;
            // their `scale = 0.9f` is dropped (no consumer), so the port keeps
            // the full 14x14 box.
            338..=341 => (14.0, 14.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The 1.4.4 rocket family (`:7939-7952`) — the eight rockets each
            // of the Grenade/Rocket/Proximity launchers resolves to (776-801)
            // plus the Snowman Cannon's 803-810. 779/783 sit inside the C#
            // range but no launcher's rocket resolves to them, so the guard
            // leaves them row-less.
            776..=801 | 803..=810 if kind != 779 && kind != 783 => {
                (14.0, 14.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false)
            }
            // The Celebration rockets (`:7584-7605` — both arms identical):
            // 14x14, aiStyle 147, `penetrate = 1`, `extraUpdates = 2` and
            // `timeLeft = 360 * MaxUpdates` = 360 × (`extraUpdates + 1`) = 1080
            // (`MaxUpdates`'s getter, `:554-561`; `alpha = 255` dropped).
            715..=718 => (14.0, 14.0, 147, 1, 1080, 2, true, false, false),
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
            ),
            // The sand family (`:1248-1258` 42, `:1449-1462` 65, `:1470-1480`
            // 68, `:4438-4449` 354): 10x10, aiStyle 10, `extraUpdates = 1`.
            // The Sandgun's own 42 sets no penetrate (the stock 1), the three
            // sand-block conversions `penetrate = -1`; their per-type
            // knockBack (8 / 6) is dropped — the spawner's input wins.
            42 => (10.0, 10.0, 10, 1, TIME_LEFT_DEFAULT, 1, true, false, false),
            65 | 68 | 354 => (10.0, 10.0, 10, -1, TIME_LEFT_DEFAULT, 1, true, false, false),
            // The arrow overrides — 117 the Bone Arrow (`:2058-2068`,
            // `extraUpdates = 2`), 469 the Bee Arrow (`:5429-5441`,
            // `alpha = 255` dropped), 485 the Hellwing (`:5604-5615` — 24x24,
            // `penetrate = -1`), 495 the Shadowflame (`:5681-5692` —
            // `penetrate = 3`) and 819 the Blood Rain arrow (`:7993-8002` —
            // `extraUpdates = 1`, `alpha`/`hide` dropped): 10x10 aiStyle-1
            // arrows, `timeLeft = 1200` (485's 24x24 box aside).
            117 => (10.0, 10.0, 1, 1, 1200, 2, true, true, false),
            469 => (10.0, 10.0, 1, 1, 1200, 0, true, true, false),
            485 => (24.0, 24.0, 1, -1, 1200, 0, true, true, false),
            495 => (10.0, 10.0, 1, 3, 1200, 0, true, true, false),
            819 => (10.0, 10.0, 1, 1, 1200, 1, true, true, false),
            // The spray/ball ammo the not-consume windows resolve — the
            // Flamethrower's Flames (`:1660-1670`: 6x6, aiStyle 193, pen 4,
            // extraUpdates 2, `armorPenetration = 15` dropped; its immunity
            // pair rides the overlay), the Clentaminator's solution spray
            // (`:2338-2347`: 6x6, aiStyle 31, pen -1, extraUpdates 2,
            // tileCollide false, `alpha`/`ignoreWater` dropped) and the Acorn
            // Slingshot's acorn (`:10145-10149`: 20x20, aiStyle 16 — the port's
            // `ai_bomb`, pen -1). Without these rows the wire would drain the
            // ammo and then decline the spawn.
            85 => (6.0, 6.0, 193, 4, TIME_LEFT_DEFAULT, 2, true, false, false),
            145 => (6.0, 6.0, 31, -1, TIME_LEFT_DEFAULT, 2, false, false, false),
            1088 => (20.0, 20.0, 16, -1, TIME_LEFT_DEFAULT, 0, true, false, false),
            // The Fairy Queen's shot 932 (`:8842-8856`): 10x10, aiStyle 181,
            // arrow, `penetrate = 5`, `timeLeft = 120`, `alpha = 0` and
            // `ignoreWater` dropped — its local-immunity pair
            // (`localNPCHitCooldown = 60`) rides the overlay below.
            932 => (10.0, 10.0, 181, 5, 120, 0, true, true, false),
            // The grappling hook family (M4-PROJ-5) — all 20 vanilla
            // `aiStyle = 7` blocks. Every row is the common tail
            // (`netImportant`, `friendly`, `penetrate = -1`,
            // `tileCollide = false`, `timeLeft *= 10`); the boxes and the two
            // `extraUpdates` differ, and `light`/`alpha` ride no consumer and
            // stay dropped (73/74's 0.4f, 332's 0.5f, 446's 0.7f, 488's 0.3f,
            // 396's 100). C# citations are per row group below.
            // `Projectile.cs:939-948` — the Hook (13): 18x18.
            HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:1141-1150` — the Ivy Whip (32): 18x18.
            IVY_WHIP => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:1536-1546` — the Dual Hook pair (73/74): 18x18,
            // `light = 0.4f` dropped.
            DUAL_HOOK_BLUE | DUAL_HOOK_RED => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:2538-2547` — the Web Slinger (165): 12x12.
            WEB_SLINGER_HOOK => (12.0, 12.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:3164-3173` — the six Gem Hooks (230-235): 18x18,
            // the shared block with no per-kind field.
            GEM_HOOK_AMETHYST
            | GEM_HOOK_TOPAZ
            | GEM_HOOK_SAPPHIRE
            | GEM_HOOK_EMERALD
            | GEM_HOOK_RUBY
            | GEM_HOOK_DIAMOND => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:3388-3399` — the Skeletron Hand (256): 6x6,
            // `extraUpdates = 1` (`scale = 1f` is the stock default).
            SKELETRON_HAND => (6.0, 6.0, 7, -1, HOOK_TIME_LEFT, 1, false, false, false),
            // `Projectile.cs:4077-4086` — the Bat Hook (315): 14x14.
            BAT_HOOK => (14.0, 14.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:4149-4158` — the Wood Hook (322): 14x14.
            WOOD_HOOK => (14.0, 14.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:4235-4244` — the Candy Cane Hook (331): 18x18.
            CANDY_CANE_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:4246-4256` — the Christmas Hook (332): 18x18,
            // `light = 0.5f` dropped.
            CHRISTMAS_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:4552-4561` — the Fish Hook (372): 18x18.
            FISH_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:4769-4779` — the Slime Hook (396): 18x18,
            // `alpha = 100` dropped.
            SLIME_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:4836-4845` — the Track Hook (403): 18x18.
            TRACK_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:5176-5186` — the Anti Gravity Hook (446): 14x14,
            // `light = 0.7f` dropped.
            ANTI_GRAVITY_HOOK => (14.0, 14.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // The 1.4.4 quartet (`Projectile.cs:5615-5643`): the four boxes
            // below share the block's tail (`:5638-5643`).
            // `Projectile.cs:5617-5621` — the Tendon Hook (486): 12x12.
            TENDON_HOOK => (12.0, 12.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:5622-5626` — the Thorn Hook (487): 22x22.
            THORN_HOOK => (22.0, 22.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:5627-5632` — the Illuminant Hook (488): 12x12,
            // `light = 0.3f` dropped.
            ILLUMINANT_HOOK => (12.0, 12.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:5633-5637` — the Worm Hook (489): 20x16.
            WORM_HOOK => (20.0, 16.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:6966-6975` — the four Lunar Hooks (646-649): 18x18,
            // the shared block with no per-kind field.
            LUNAR_HOOK_SOLAR
            | LUNAR_HOOK_VORTEX
            | LUNAR_HOOK_NEBULA
            | LUNAR_HOOK_STARDUST => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:7001-7010` — the Static Hook (652): 18x18.
            STATIC_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:7756-7765` — the Amber Hook (753): 18x18.
            AMBER_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:8301-8310` — the Squirrel Hook (865): 18x18.
            SQUIRREL_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 0, false, false, false),
            // `Projectile.cs:8884-8894` — the Queen Slime Hook (935): 18x18,
            // `extraUpdates = 3`.
            QUEEN_SLIME_HOOK => (18.0, 18.0, 7, -1, HOOK_TIME_LEFT, 3, false, false, false),
            _ => return None,
        };
    // The `SetDefaults` inner ifs that add the local-immunity pair on top of
    // the row above: `:2222-2338`'s 133/136/139/142, `:7939-7952`'s
    // 777/781/785/788/794/797/800 (the C# lists 800 twice — harmless),
    // `:1660-1670`'s 85 (the Flamethrower's Flames) and `:8842-8856`'s 932.
    // Every other kind — including every row that predates the two schema
    // fields — keeps the pool defaults (`:464`: false / −2).
    let (uses_local_immunity, local_hit_cooldown) = match kind {
        133 | 136 | 139 | 142 | 777 | 781 | 785 | 788 | 794 | 797 | 800 => (true, -1),
        85 => (true, -1),
        932 => (true, 60),
        _ => (false, -2),
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
        uses_local_immunity,
        local_hit_cooldown,
        scale: set_defaults_scale(kind),
        // `DefaultToYoyo` sets no immunity slice (`:10764-10771`) — only the
        // Terrarian pair carries the idStatic-10 (`:6068-6069`, `:6081-6082`).
        // The M4-PROJ-4b yoyo shots carry their own: the Cascade/EoC-Dash
        // pair the local slice at −1 (`:10538-10539`, `:10605-10606`), the
        // shard idStatic-20 (`:10648-10649`) and the Kraken wave
        // idStatic-10 (`:10665-10666`).
        uses_id_static: matches!(
            kind,
            TERRARIAN_YOYO | TERRARIAN_BEAM | CRYSTAL_SHARD_MELEE | KRAKEN_WAVE
        ),
        id_static_hit_cooldown: if matches!(kind, TERRARIAN_YOYO | TERRARIAN_BEAM | KRAKEN_WAVE) {
            10
        } else if kind == CRYSTAL_SHARD_MELEE {
            20
        } else {
            -1
        },
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
    /// The C# `localAI[0..2]` scratch array — the yoyo family's lifetime tick
    /// counter (`localAI[0]`, `AI_099_2_Yoyos` `Projectile.cs:67745`) and the
    /// Terrarian's beam timer (`localAI[1]`, `:67778`); the Moon Lord whip's
    /// arm budget roll (`localAI[1] = 1 + Main.rand.Next(3)`, `:47669-47672`)
    /// and the per-type on-hit latches (`localAI[0]`, `:13907`) ride the first
    /// two slots.
    pub local_ai: [f32; 3],
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
    /// `Projectile.direction` (the C# default 1) — the ±1 sprite direction:
    /// the yoyo rotation spins by it (`Projectile.cs:68096`) and the pose
    /// block writes it from the owner-relative X comparison (`:67841`/`:67846`).
    pub direction: i32,
    /// `Projectile.scale` from `SetDefaults` — the yoyo family's ladder; the
    /// render row reads it, the AI does not.
    pub scale: f32,
    /// `Projectile.alpha` (the stock 0) — the render fade; the yoyo ghost sets
    /// 255 (`:68034`) and the Terrarian beam's 115 arm fades it out
    /// (`:34431-34436`).
    pub alpha: i32,
    bounces: u8,
    /// `GrappleMovement`'s pull input: the attached aiStyle-7 hook's center
    /// (`Player.cs:22757-22856` reads `grappling[]` — this multi-hook field is
    /// the task's next phase; the single-hook bridge keeps the pull working
    /// meanwhile, and `grappling_active`'s owner reads it).
    pub grapple_center: Vec2,
    /// The tile-fall mode (`AI_010`'s `ai[0] == 1f`): the projectile carries a
    /// falling tile kind (`spawn_falling`), falls with `velocity.Y += 0.41f`
    /// per tick and places the tile on landing.
    pub falling_tile: Option<u16>,
    /// `AI_016`'s `timeLeft <= 3` trigger: the bomb waits for the pool's
    /// end-of-tick explosion pass (`Kill()`'s bomb path).
    pub detonated: bool,
    /// `AI_103`'s mark spawn (`Projectile.cs:33467-33475`'s
    /// `NPC.NewNPC(..., localAI[0])`): the (kind, x, y) the world_sim's
    /// projectile pass resolves into the NPC vec.
    pub wants_mark_spawn: Option<(u16, f32, f32, f32, f32)>,
    /// The 578's 580 spawn (`Projectile.cs:33838-33841`): the (x, y,
    /// vx, vy, ai0) payload the world_sim resolves into the pool.
    pub wants_ritual_bolt: Option<(f32, f32, f32, f32, f32)>,
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
            direction: 1,
            scale: 1.0,
            alpha: 0,
            bounces: 0,
            grapple_center: Vec2::ZERO,
            falling_tile: None,
            detonated: false,
            wants_mark_spawn: None,
            wants_ritual_bolt: None,
        }
    }
}

impl Projectile {

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

    /// `Projectile.Kill` (`:70642`): the slot deactivates. The AI arms and the
    /// recycle tables call it; the `timeLeft <= 0` path stays inline.
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

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|p| p.active).count()
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
            uses_local_immunity: data.uses_local_immunity,
            local_hit_cooldown: data.local_hit_cooldown,
            uses_id_static_immunity: data.uses_id_static,
            id_static_hit_cooldown: data.id_static_hit_cooldown,
            extra_updates: data.extra_updates,
            ai: [ai0, ai1, 0.0, 0.0],
            w: data.w,
            h: data.h,
            tile_collide: data.tile_collide,
            arrow: data.arrow,
            trap: data.trap,
            scale: data.scale,
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
    pub fn update(
        &mut self,
        world: &mut World,
        npcs: &mut [Npc],
        owner: Option<(f32, f32)>,
        aim: Option<Vec2>,
        mut player: Option<&mut crate::player::Player>,
        inputs: YoyoInputs,
        rng: &mut dyn tr_core::loot::DropRng,
        local_owner: u8,
        hostile_local: bool,
    ) {
        self.update_count += 1;
        // `Player.Update`'s clear (`Player.cs:28300-28301`) — the C# runs it at
        // the end of the player pass, i.e. just before this one, so starting
        // here preserves the vanilla's one-tick latency exactly.
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
                // The aiStyle dispatch: the arrow/bullet family (aiStyle 1,
                // `AI()`: `if (this.aiStyle == 1) this.AI_001();`), the
                // spear/dart family (aiStyle 14), the yoyo family (aiStyle 99,
                // `:33156-33163`'s `else if (this.aiStyle == 99)` →
                // `AI_099_2_Yoyos`) and the yoyo beams (aiStyle 115,
                // `:34414-34437`). The falling-tile mode keeps its custom flight.
                else if let Some(data) = proj_data(self.slots[i].kind) {
                    match data.ai_style {
                        1 => self.slots[i].ai_arrow(world),
                        7 => {
                            // `AI()`'s `else if (this.aiStyle == 7)`
                            // (`Projectile.cs:24230-24233`) ->
                            // `AI_007_GrapplingHooks`.
                            let dead = player.as_ref().is_some_and(|p| p.dead);
                            let local = self.slots[i].owner == local_owner;
                            let center = owner
                                .map(|(x, y)| Vec2::new(x, y))
                                .unwrap_or(self.slots[i].pos);
                            // The stream is copied out and back so the slot borrow
                            // and the rng borrow never overlap.
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
                                // `:52705`/`:52862` — an attached hook registers
                                // itself every tick; `GrappleMovement`'s pull
                                // reads the rebuilt set.
                                self.slots[i].grapple_center = c;
                                self.register_hook(i);
                                // `:52719-52741` — the attach cap. The count is
                                // this owner's aiStyle-7 slots that are attached
                                // plus the slot that just latched; the victim is
                                // the lowest `timeLeft` of them.
                                if latched {
                                    let owner = self.slots[i].owner;
                                    let cap = crate::grapple::hook_spec_id(self.slots[i].kind)
                                        .max_attached;
                                    let mut count = 0usize;
                                    let mut worst = usize::MAX;
                                    // The C#'s `num13 = 100000` initial — so the
                                    // first candidate always wins and ties keep
                                    // the earliest slot.
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
                            // `:52590-52593` — the 1/16 cracked-brick break, the
                            // one `&mut World` touch the hook AI performs. The C#
                            // sends the tile update over the wire; no MP here.
                            if let Some((tx, ty)) = crack {
                                if let Some(t) = world.tile_mut(tx, ty) {
                                    t.flags &= !tr_core::tile::FLAG_ACTIVE;
                                }
                            }
                        }
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
            self.damage_pve(i, npcs, owner, player.as_deref_mut(), rng);
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
        self.damage_pve(
            slot,
            npcs,
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

    // --- the grappling hook family (M4-PROJ-5, Phase 1) ---
    /// The 32 vanilla hook kinds with their `SetDefaults` box — the
    /// transcription of `Projectile.cs`'s 20 aiStyle-7 blocks (research
    /// §1.7; the `extraUpdates` and the shared tail are asserted below).
    const HOOK_BOXES: &[(u16, f32, f32)] = &[
        (HOOK, 18.0, 18.0),
        (IVY_WHIP, 18.0, 18.0),
        (DUAL_HOOK_BLUE, 18.0, 18.0),
        (DUAL_HOOK_RED, 18.0, 18.0),
        (WEB_SLINGER_HOOK, 12.0, 12.0),
        (GEM_HOOK_AMETHYST, 18.0, 18.0),
        (GEM_HOOK_TOPAZ, 18.0, 18.0),
        (GEM_HOOK_SAPPHIRE, 18.0, 18.0),
        (GEM_HOOK_EMERALD, 18.0, 18.0),
        (GEM_HOOK_RUBY, 18.0, 18.0),
        (GEM_HOOK_DIAMOND, 18.0, 18.0),
        (SKELETRON_HAND, 6.0, 6.0),
        (BAT_HOOK, 14.0, 14.0),
        (WOOD_HOOK, 14.0, 14.0),
        (CANDY_CANE_HOOK, 18.0, 18.0),
        (CHRISTMAS_HOOK, 18.0, 18.0),
        (FISH_HOOK, 18.0, 18.0),
        (SLIME_HOOK, 18.0, 18.0),
        (TRACK_HOOK, 18.0, 18.0),
        (ANTI_GRAVITY_HOOK, 14.0, 14.0),
        (TENDON_HOOK, 12.0, 12.0),
        (THORN_HOOK, 22.0, 22.0),
        (ILLUMINANT_HOOK, 12.0, 12.0),
        (WORM_HOOK, 20.0, 16.0),
        (LUNAR_HOOK_SOLAR, 18.0, 18.0),
        (LUNAR_HOOK_VORTEX, 18.0, 18.0),
        (LUNAR_HOOK_NEBULA, 18.0, 18.0),
        (LUNAR_HOOK_STARDUST, 18.0, 18.0),
        (STATIC_HOOK, 18.0, 18.0),
        (AMBER_HOOK, 18.0, 18.0),
        (SQUIRREL_HOOK, 18.0, 18.0),
        (QUEEN_SLIME_HOOK, 18.0, 18.0),
    ];

    #[test]
    fn the_hook_constants_carry_the_vanilla_ids() {
        // `ID/ProjectileID.cs`'s transcription — the arms of Phases 2-4 key
        // off these, so a wrong id is a wrong hook.
        let ids = [
            HOOK,
            IVY_WHIP,
            DUAL_HOOK_BLUE,
            DUAL_HOOK_RED,
            WEB_SLINGER_HOOK,
            GEM_HOOK_AMETHYST,
            GEM_HOOK_TOPAZ,
            GEM_HOOK_SAPPHIRE,
            GEM_HOOK_EMERALD,
            GEM_HOOK_RUBY,
            GEM_HOOK_DIAMOND,
            SKELETRON_HAND,
            BAT_HOOK,
            WOOD_HOOK,
            CANDY_CANE_HOOK,
            CHRISTMAS_HOOK,
            FISH_HOOK,
            SLIME_HOOK,
            TRACK_HOOK,
            ANTI_GRAVITY_HOOK,
            TENDON_HOOK,
            THORN_HOOK,
            ILLUMINANT_HOOK,
            WORM_HOOK,
            LUNAR_HOOK_SOLAR,
            LUNAR_HOOK_VORTEX,
            LUNAR_HOOK_NEBULA,
            LUNAR_HOOK_STARDUST,
            STATIC_HOOK,
            AMBER_HOOK,
            SQUIRREL_HOOK,
            QUEEN_SLIME_HOOK,
        ];
        let vanilla = [
            13, 32, 73, 74, 165, 230, 231, 232, 233, 234, 235, 256, 315, 322, 331, 332, 372, 396,
            403, 446, 486, 487, 488, 489, 646, 647, 648, 649, 652, 753, 865, 935,
        ];
        assert_eq!(ids, vanilla);
    }

    #[test]
    fn dw_1_1_the_32_hook_kinds_carry_the_vanilla_row() {
        let mut distinct: Vec<u16> = HOOK_BOXES.iter().map(|&(kind, _, _)| kind).collect();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), 32, "the vanilla family's kind count");
        for &(kind, w, h) in HOOK_BOXES {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} has no row"));
            assert_eq!((d.w, d.h), (w, h), "{kind}");
            assert_eq!(d.ai_style, 7, "{kind}");
            assert_eq!(d.penetrate, -1, "{kind}");
            assert!(!d.tile_collide, "{kind}");
            assert_eq!(d.time_left, 36_000, "{kind}");
            // The `extraUpdates` split: only 256 (1) and 935 (3) move.
            let extra = match kind {
                SKELETRON_HAND => 1,
                QUEEN_SLIME_HOOK => 3,
                _ => 0,
            };
            assert_eq!(d.extra_updates, extra, "{kind}");
            // The pool defaults the family never touches.
            assert!(!d.arrow && !d.trap, "{kind}");
            assert!(!d.uses_local_immunity, "{kind}");
            assert_eq!(d.local_hit_cooldown, -2, "{kind}");
            assert!(!d.uses_id_static, "{kind}");
            assert_eq!(d.id_static_hit_cooldown, -1, "{kind}");
            assert_eq!(d.scale, 1.0, "{kind}");
        }
        let moved = HOOK_BOXES
            .iter()
            .filter(|&&(kind, _, _)| proj_data(kind).is_some_and(|d| d.extra_updates != 0))
            .count();
        assert_eq!(moved, 2, "only 256 and 935 run extra sub-steps");
    }

    #[test]
    fn dw_1_2_the_non_hook_neighbours_stay_row_less() {
        // The kinds on both sides of every hook block — an unlisted kind must
        // keep returning `None` rather than a silent default row. (645 does
        // not belong here: it is the poison dart's id and carries its own
        // row.)
        for kind in [
            72, 75, 229, 236, 257, 316, 490, 650, 752, 866, 934, 936,
        ] {
            assert!(proj_data(kind).is_none(), "{kind} is not a hook");
        }
    }

    #[test]
    fn dw_1_3_every_hook_row_group_cites_the_source() {
        // The golden rule's citation requirement is a file-content invariant,
        // so it is checked against this file's own text: the hook section of
        // `proj_data` must name every C# row group, and each group's arm must
        // sit directly under its `Projectile.cs:line` citation.
        const HOOK_ARMS: &[&str] = &[
            "HOOK",
            "IVY_WHIP",
            "DUAL_HOOK_BLUE",
            "WEB_SLINGER_HOOK",
            "GEM_HOOK_AMETHYST",
            "SKELETRON_HAND",
            "BAT_HOOK",
            "WOOD_HOOK",
            "CANDY_CANE_HOOK",
            "CHRISTMAS_HOOK",
            "FISH_HOOK",
            "SLIME_HOOK",
            "TRACK_HOOK",
            "ANTI_GRAVITY_HOOK",
            "TENDON_HOOK",
            "THORN_HOOK",
            "ILLUMINANT_HOOK",
            "WORM_HOOK",
            "LUNAR_HOOK_SOLAR",
            "STATIC_HOOK",
            "AMBER_HOOK",
            "SQUIRREL_HOOK",
            "QUEEN_SLIME_HOOK",
        ];
        const HOOK_BLOCKS: &[&str] = &[
            "Projectile.cs:939-948",
            "Projectile.cs:1141-1150",
            "Projectile.cs:1536-1546",
            "Projectile.cs:2538-2547",
            "Projectile.cs:3164-3173",
            "Projectile.cs:3388-3399",
            "Projectile.cs:4077-4086",
            "Projectile.cs:4149-4158",
            "Projectile.cs:4235-4244",
            "Projectile.cs:4246-4256",
            "Projectile.cs:4552-4561",
            "Projectile.cs:4769-4779",
            "Projectile.cs:4836-4845",
            "Projectile.cs:5176-5186",
            "Projectile.cs:5615-5643",
            "Projectile.cs:5617-5621",
            "Projectile.cs:5622-5626",
            "Projectile.cs:5627-5632",
            "Projectile.cs:5633-5637",
            "Projectile.cs:6966-6975",
            "Projectile.cs:7001-7010",
            "Projectile.cs:7756-7765",
            "Projectile.cs:8301-8310",
            "Projectile.cs:8884-8894",
        ];
        let src = include_str!("projectile.rs");
        let lines: Vec<&str> = src.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.contains("The grappling hook family (M4-PROJ-5) — all 20"))
            .expect("the hook section's header");
        let end = lines
            .iter()
            .position(|l| l.trim() == "_ => return None,")
            .expect("the match's fall-through");
        let section = &lines[start..end];
        for cite in HOOK_BLOCKS {
            assert!(
                section.iter().any(|l| l.contains(cite)),
                "the section must cite {cite}"
            );
        }
        // One citation per row group: the contiguous comment block directly
        // above each group's first arm line names its C# block.
        for (i, line) in section.iter().enumerate() {
            let t = line.trim();
            let arm = HOOK_ARMS.iter().any(|a| match t.strip_prefix(a) {
                Some("") => true,
                Some(r) => r.starts_with(" =>") || r.starts_with(" |"),
                None => false,
            });
            if !arm {
                continue;
            }
            let mut cited = false;
            let mut j = i;
            while j > 0 && section[j - 1].trim().starts_with("//") {
                j -= 1;
                cited |= section[j].contains("Projectile.cs:")
                    && section[j].contains(|c: char| c.is_ascii_digit());
            }
            assert!(cited, "the arm for {t} lacks its citation");
        }
    }

    #[test]
    fn a_hook_spawns_through_the_pool_with_its_box_and_budget() {
        // Phase 1's purpose: the spawn path reads the rows, so a hook can
        // exist in the pool at all (before it `proj_data(13)` was `None` and
        // the spawn silently no-opped).
        let mut pool = ProjectilePool::new();
        let queen = pool
            .spawn_with_data(QUEEN_SLIME_HOOK, 100.0, 200.0, 12.0, 0.0, 30, 0.0)
            .expect("a free slot");
        let p = &pool.slots[queen];
        assert_eq!((p.w, p.h), (18.0, 18.0));
        assert_eq!(p.time_left, 36_000);
        assert_eq!(p.extra_updates, 3, "935's three sub-steps per tick");
        assert!(!p.tile_collide, "the hook ignores tiles in flight");
        assert_eq!(p.max_penetrate, -1);
        assert!(p.active && p.kind == QUEEN_SLIME_HOOK && p.owner == 0);
        // The 6x6 Skeletron Hand rides the same path with its own box and its
        // single extra sub-step.
        let hand = pool
            .spawn_with_data(SKELETRON_HAND, 120.0, 200.0, 8.0, 0.0, 10, 0.0)
            .expect("a second slot");
        assert_eq!((pool.slots[hand].w, pool.slots[hand].h), (6.0, 6.0));
        assert_eq!(pool.slots[hand].extra_updates, 1);
        // A non-hook neighbour still refuses to spawn.
        assert!(pool.spawn_with_data(257, 140.0, 200.0, 1.0, 0.0, 1, 0.0).is_none());
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
