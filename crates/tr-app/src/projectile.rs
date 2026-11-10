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
/// `ProjectileID.Count` (`ID/ProjectileID.cs` static ctor) — the id-space
/// bound `Player.ownedProjectileCounts` is sized by (`Player.cs:2992`).
pub const PROJ_COUNT: usize = 1136;
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
    /// The summon identification (M4-PROJ-8): `Projectile.minion` +
    /// `Projectile.minionSlots` (the stock false / 0f, `Projectile.cs:775-776`).
    pub minion: bool,
    pub sentry: bool,
    /// `Projectile.minionSlots` — the per-tick slot budget the pool's
    /// accumulate block adds (`:16719`); 0.5 for the spider family (never
    /// rounded to an integer count).
    pub minion_slots: f32,
    /// `Projectile.netImportant` (`:706`'s stock false) — carried on the
    /// summon rows (the plan constraint); the MP bookkeeping consumer is out
    /// of scope (module doc).
    pub net_important: bool,
    /// `Projectile.ignoreWater` (`:752`'s stock false) — carried on the summon
    /// rows; the `wet = false` spawn write (`:10886-10889`) lands with the
    /// projectile-liquid work.
    pub ignore_water: bool,
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

/// `ProjectileID.Sets.MinionSacrificable` (`ID/ProjectileID.cs:529`) — the
/// minion kinds the summon use-path may kill to free slots (Phase 6's
/// `FreeUpPetsAndMinions` sort loop). Verbatim C# list order; the lookup is
/// order-free (a `CreateBoolSet` set).
pub const MINION_SACRIFICABLE: &[u16] = &[
    191, 192, 193, 194, 266, 317, 373, 375, 387, 388, 390, 393, 407, 423, 533, 613, 755, 758, 759,
    831, 970, 864, 946, 951, 963, 1022, 1093, 1112, 1094, 1113, 1118, 1119, 625, 626, 627, 628,
];

/// `ProjectileID.Sets.SentryShot` (`ID/ProjectileID.cs:432`) — the shots the
/// sentry bodies spawn. Verbatim C# data carried now so the identification
/// sets land once; the shot consumers ride the later projectile-family tasks.
pub const SENTRY_SHOT: &[u16] = &[
    680, 664, 666, 668, 694, 695, 696, 644, 642, 378, 379, 309, 967, 1026,
];

/// The summon kinds (M4-PROJ-8) — the C# constant names from
/// `ID/ProjectileID.cs`. The rows below dispatch on them and Phases 3-6
/// branch on the ids.
pub const FROST_HYDRA: u16 = 308;
pub const HORNET: u16 = 373;
pub const FLYING_IMP: u16 = 375;
pub const RETANIMINI: u16 = 387;
pub const SPAZMAMINI: u16 = 388;
pub const TEMPEST: u16 = 407;
pub const UFO_MINION: u16 = 423;
pub const STARDUST_CELL: u16 = 613;
pub const STARDUST_GUARDIAN: u16 = 623;
pub const BABY_BIRD: u16 = 759;
pub const STORM_TIGER_1: u16 = 833;
pub const STORM_TIGER_2: u16 = 834;
pub const STORM_TIGER_3: u16 = 835;
pub const ABIGAIL: u16 = 963;
pub const QUEEN_SPIDER: u16 = 377;
pub const MOONLORD_TURRET: u16 = 641;
pub const RAINBOW_CRYSTAL: u16 = 643;
pub const FLAME_BURST_TOWER_1: u16 = 663;
pub const FLAME_BURST_TOWER_2: u16 = 665;
pub const FLAME_BURST_TOWER_3: u16 = 667;
pub const BALLISTRA_TOWER_1: u16 = 677;
pub const BALLISTRA_TOWER_2: u16 = 678;
pub const BALLISTRA_TOWER_3: u16 = 679;
pub const LIGHTNING_AURA_1: u16 = 688;
pub const LIGHTNING_AURA_2: u16 = 689;
pub const LIGHTNING_AURA_3: u16 = 690;
pub const EXPLOSIVE_TRAP_1: u16 = 691;
pub const EXPLOSIVE_TRAP_2: u16 = 692;
pub const EXPLOSIVE_TRAP_3: u16 = 693;
pub const HOUNDIUS_SHOOTIUS: u16 = 966;
pub const DEAD_CELLS_BARNACLE: u16 = 1025;

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
            // The summon rows (M4-PROJ-8 Phase 1) — the verbatim `SetDefaults`
            // slices for the 13 minion kinds and the 18 sentry kinds. The
            // damage/knockback stay the spawner's input; `extraUpdates`/`arrow`
            // /`trap` keep the stocks (no summon arm sets them). `timeLeft`:
            // the minions ride `timeLeft *= 5` on the stock 3600 (`:787`) =
            // 18000; the sentries set the `SentryLifeTime` 36000 (`:300`).
            // The aiStyle-62 flying minions (`Projectile.cs:4563` Hornet,
            // `:4587` FlyingImp, `:4878` Tempest, `:4960` UFOMinion, `:6598`
            // StardustCellMinion, `:9100` AbigailMinion) — UFO/Cell keep the
            // stock `tileCollide = true` (their arms set no tileCollide line).
            HORNET => (24.0, 26.0, 62, -1, 18000, 0, false, false, false),
            FLYING_IMP => (34.0, 26.0, 62, -1, 18000, 0, false, false, false),
            TEMPEST => (28.0, 40.0, 62, -1, 18000, 0, false, false, false),
            UFO_MINION => (28.0, 28.0, 62, -1, 18000, 0, true, false, false),
            STARDUST_CELL => (24.0, 24.0, 62, -1, 18000, 0, true, false, false),
            ABIGAIL => (30.0, 48.0, 62, -1, 18000, 0, false, false, false),
            // The 0.5-slot spider minions (`Projectile.cs:4707` Retanimini,
            // `:4723` Spazmamini) — aiStyle 66, the slot budget halves.
            RETANIMINI | SPAZMAMINI => (40.0, 20.0, 66, -1, 18000, 0, false, false, false),
            // The Stardust Guardian (`Projectile.cs:6709`) — aiStyle 120, the
            // 0-slot pet-grade minion.
            STARDUST_GUARDIAN => (50.0, 80.0, 120, -1, 18000, 0, false, false, false),
            // The Storm Tiger family (`Projectile.cs:8091`/`:8105`/`:8119`)
            // — aiStyle 67; these three arms set neither tileCollide nor
            // ignoreWater (the stock true / false).
            STORM_TIGER_1 => (26.0, 20.0, 67, -1, 18000, 0, true, false, false),
            STORM_TIGER_2 | STORM_TIGER_3 => (20.0, 30.0, 67, -1, 18000, 0, true, false, false),
            // The baby bird (`Projectile.cs:7829-7845` — the C#
            // `ProjectileID.BabyBird`) — the buff-216 pet-respawn minion
            // (Phase 2's arm), aiStyle 158; the `friendly`/`drawLayer` lines
            // have no port consumer (the module doc).
            BABY_BIRD => (10.0, 10.0, 158, -1, 18000, 0, true, false, false),
            // The sentries: the aiStyle-53 placeable staff sentries —
            // FrostHydra (`Projectile.cs:3997`; the `light = 0.25f`/
            // `coldDamage` lines have no port consumer, the module doc),
            // QueenSpider (`:4615`), HoundiusShootius (`:9143`); the Moon
            // Lord turret pair (`:6916`/`:6929` — aiStyle 123); the DD2
            // towers (`:7130-7151` FlameBurstTower T1-T3 — aiStyle 130, the
            // per-type box overrides T2 28×58 / T3 28×60; `:7241`
            // BallistraTower T1-T3 — aiStyle 134; `:7325` LightningAura
            // T1-T3 — aiStyle 137, penetrate -1 + the local pair; `:7340`
            // ExplosiveTrap T1-T3 — aiStyle 138, penetrate -1); DeadCells
            // Barnacle (`:9707` — aiStyle 197).
            FROST_HYDRA => (80.0, 74.0, 53, 1, 36000, 0, true, false, false),
            QUEEN_SPIDER => (66.0, 50.0, 53, 1, 36000, 0, true, false, false),
            HOUNDIUS_SHOOTIUS => (18.0, 60.0, 53, 1, 36000, 0, true, false, false),
            MOONLORD_TURRET | RAINBOW_CRYSTAL => {
                (32.0, 32.0, 123, 1, 36000, 0, false, false, false)
            }
            FLAME_BURST_TOWER_1 => (30.0, 54.0, 130, 1, 36000, 0, false, false, false),
            FLAME_BURST_TOWER_2 => (28.0, 58.0, 130, 1, 36000, 0, false, false, false),
            FLAME_BURST_TOWER_3 => (28.0, 60.0, 130, 1, 36000, 0, false, false, false),
            BALLISTRA_TOWER_1 | BALLISTRA_TOWER_2 | BALLISTRA_TOWER_3 => {
                (26.0, 54.0, 134, 1, 36000, 0, false, false, false)
            }
            LIGHTNING_AURA_1 | LIGHTNING_AURA_2 | LIGHTNING_AURA_3 => {
                (16.0, 16.0, 137, -1, 36000, 0, false, false, false)
            }
            EXPLOSIVE_TRAP_1 | EXPLOSIVE_TRAP_2 | EXPLOSIVE_TRAP_3 => {
                (16.0, 16.0, 138, -1, 36000, 0, false, false, false)
            }
            DEAD_CELLS_BARNACLE => (42.0, 38.0, 197, 1, 36000, 0, true, false, false),
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
        // The summon rows' local pairs: the Guardian (`Projectile.cs:6721-
        // 6722`), the Storm Tigers (`:8096-8097`/`:8110-8111`/`:8124-8125`),
        // Abigail (`:9111-9112`), the Lightning Auras (`:7332-7333`), the
        // baby bird (`:7841-7842`).
        STARDUST_GUARDIAN => (true, 5),
        STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3 => (true, 10),
        ABIGAIL => (true, 20),
        BABY_BIRD => (true, 15),
        LIGHTNING_AURA_1 | LIGHTNING_AURA_2 | LIGHTNING_AURA_3 => (true, 3),
        _ => (false, -2),
    };
    // The summon identification (M4-PROJ-8 Phase 1): the
    // `minion`/`minionSlots` slice of the arms above. Everything else keeps
    // the stock `minion = false` / `minionSlots = 0f`
    // (`Projectile.cs:775-776`).
    let (minion, sentry, minion_slots) = match kind {
        HORNET | FLYING_IMP | TEMPEST | UFO_MINION | STARDUST_CELL | BABY_BIRD => {
            (true, false, 1.0)
        }
        RETANIMINI | SPAZMAMINI => (true, false, 0.5),
        STARDUST_GUARDIAN | STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3 | ABIGAIL => {
            (true, false, 0.0)
        }
        FROST_HYDRA | QUEEN_SPIDER | MOONLORD_TURRET | RAINBOW_CRYSTAL | FLAME_BURST_TOWER_1
        | FLAME_BURST_TOWER_2 | FLAME_BURST_TOWER_3 | BALLISTRA_TOWER_1 | BALLISTRA_TOWER_2
        | BALLISTRA_TOWER_3 | LIGHTNING_AURA_1 | LIGHTNING_AURA_2 | LIGHTNING_AURA_3
        | EXPLOSIVE_TRAP_1 | EXPLOSIVE_TRAP_2 | EXPLOSIVE_TRAP_3 | HOUNDIUS_SHOOTIUS
        | DEAD_CELLS_BARNACLE => (false, true, 0.0),
        _ => (false, false, 0.0),
    };
    // Every summon arm sets `netImportant = true`; `ignoreWater = true`
    // everywhere but the Storm Tiger family (the stock false there).
    let net_important = minion || sentry;
    let ignore_water =
        (minion || sentry) && !matches!(kind, STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3);
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
        // `DefaultToYoyo` sets no immunity slice (`:10764-10771`) — the
        // Terrarian pair's idStatic-10 (`:6068-6069`, `:6081-6082`) plus the
        // summon rows' (Tempest `:4888-4889`, Retanimini `:4716-4717`,
        // Spazmamini `:4732-4733`).
        uses_id_static: matches!(
            kind,
            TERRARIAN_YOYO | TERRARIAN_BEAM | TEMPEST | RETANIMINI | SPAZMAMINI
        ),
        id_static_hit_cooldown: match kind {
            TERRARIAN_YOYO | TERRARIAN_BEAM | TEMPEST => 10,
            RETANIMINI => 16,
            SPAZMAMINI => 12,
            _ => -1,
        },
        minion,
        sentry,
        minion_slots,
        net_important,
        ignore_water,
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
    /// The C# `ai[0..2]` scratch array — the aiStyle-1 flight counter lives
    /// in `ai[0]` (the generic arm `ai[0] >= 15f`, `Projectile.cs:57572`).
    pub ai: [f32; 3],
    /// The C# `localAI[0..2]` scratch array — the yoyo family's lifetime tick
    /// counter (`localAI[0]`, `AI_099_2_Yoyos` `Projectile.cs:67745`) and the
    /// Terrarian's beam timer (`localAI[1]`, `:67778`).
    pub local_ai: [f32; 3],
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
    /// The tile-fall mode (`AI_010`'s `ai[0] == 1f`): the projectile carries a
    /// falling tile kind (`spawn_falling`), falls with `velocity.Y += 0.41f`
    /// per tick and places the tile on landing.
    pub falling_tile: Option<u16>,
    /// `AI_016`'s `timeLeft <= 3` trigger: the bomb waits for the pool's
    /// end-of-tick explosion pass (`Kill()`'s bomb path).
    pub detonated: bool,
    /// The summon identification (M4-PROJ-8): `Projectile.minion` /
    /// `Projectile.sentry` (the stock false, `Projectile.cs:700`/`:775`) —
    /// the flags the pool's per-tick minion/sentry block (`:16704-16740`,
    /// Phase 3) and the AI dispatch branch on.
    pub minion: bool,
    pub sentry: bool,
    /// `Projectile.minionSlots` — the slot budget this occupant consumes
    /// (`:16719`'s `slotsMinions += this.minionSlots`); 0.5 for the spiders.
    pub minion_slots: f32,
    /// `Projectile.minionPos` (`:398`) — the pool's write (`:16711`, Phase 3)
    /// orders the draw pass; carried as a plain field until the draw phase.
    pub minion_pos: u16,
    /// `Projectile.originalDamage` (`:352`) — the UNSCALED summon damage the
    /// per-tick recompute reads (`:16704-16707`); the spawn seam sets it from
    /// the summon spawn sites' post-spawn writes (`Player.cs:49601`/`:48209`).
    pub original_damage: i32,
    /// `Projectile.netImportant` (stock false, `:706`).
    pub net_important: bool,
    /// `Projectile.ignoreWater` (stock false, `:752`).
    pub ignore_water: bool,
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
            ai: [0.0; 3],
            local_ai: [0.0; 3],
            w: SIZE,
            h: SIZE,
            tile_collide: true,
            arrow: false,
            trap: false,
            direction: 1,
            scale: 1.0,
            alpha: 0,
            bounces: 0,
            falling_tile: None,
            detonated: false,
            minion: false,
            sentry: false,
            minion_slots: 0.0,
            minion_pos: 0,
            original_damage: 0,
            net_important: false,
            ignore_water: false,
        }
    }
}

impl Projectile {
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
        }
    }

    /// The `ProjectileKey` a slot's current occupant would travel under
    /// (`Projectile.cs:10854`'s key mint).
    pub fn key_of(&self, slot: usize) -> i32 {
        let p = &self.slots[slot];
        crate::net::projectile_key(p.owner, slot, p.generation)
    }

    /// Writes the slot's `originalDamage` after a spawn — the summon-spawn
    /// arms set it explicitly (`Player.cs:49601`, `:48209`, `:10713`).
    pub fn set_original_damage(&mut self, slot: usize, value: i32) {
        if let Some(p) = self.slots.get_mut(slot) {
            p.original_damage = value;
        }
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
            ai: [1.0, 0.0, 0.0],
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
            ai: [ai0, ai1, 0.0],
            w: data.w,
            h: data.h,
            tile_collide: data.tile_collide,
            arrow: data.arrow,
            trap: data.trap,
            scale: data.scale,
            bounces: 0,
            falling_tile: None,
            detonated: false,
            // The summon slice (M4-PROJ-8 Phase 1) — the identification flags
            // and the water/net markers ride the row; `minion_pos` stays the
            // stock 0 until the pool's per-tick block writes it
            // (`Projectile.cs:16711`, Phase 3). `originalDamage` is the
            // summon spawn sites' post-spawn write — the unscaled damage the
            // pool's recompute reads (`Player.cs:49601`/`:48209` →
            // `Projectile.cs:16704-16707`); the non-summon kinds keep the
            // stock 0 (`Projectile.cs:790`).
            minion: data.minion,
            sentry: data.sentry,
            minion_slots: data.minion_slots,
            net_important: data.net_important,
            ignore_water: data.ignore_water,
            original_damage: if data.minion || data.sentry {
                damage
            } else {
                0
            },
            ..Default::default()
        };
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
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
        inputs: YoyoInputs,
        rng: &mut dyn tr_core::loot::DropRng,
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
                // The aiStyle dispatch: the arrow/bullet family (aiStyle 1,
                // `AI()`: `if (this.aiStyle == 1) this.AI_001();`), the
                // spear/dart family (aiStyle 14), the yoyo family (aiStyle 99,
                // `:33156-33163`'s `else if (this.aiStyle == 99)` →
                // `AI_099_2_Yoyos`) and the yoyo beams (aiStyle 115,
                // `:34414-34437`). The falling-tile mode keeps its custom flight.
                else if let Some(data) = proj_data(self.slots[i].kind) {
                    match data.ai_style {
                        1 => self.slots[i].ai_arrow(world),
                        3 => self.slots[i].ai_boomerang(world, owner),
                        14 => self.slots[i].ai_spear(world),
                        16 => self.slots[i].ai_bomb(world),
                        115 => self.slots[i].ai_yoyo_beam(world),
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
            pool.update(
                &mut w,
                &mut [],
                Some((80.0, 308.0)),
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
        pool.slots[0].ai = [1.0, 0.0, 0.0];
        let mut w = world();
        let mut died = false;
        for _ in 0..120 {
            pool.update(
                &mut w,
                &mut [],
                Some((100.0, 200.0)),
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
}

#[cfg(test)]
mod summon_tests {
    use super::*;
    use tr_core::tile::Tile;

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    fn rng() -> tr_core::tile_light::FastRandom {
        tr_core::tile_light::FastRandom::new(7)
    }

    /// The 13 minion kinds (M4-PROJ-8 Phase 1's rows) — the aiStyle-62 family
    /// plus the aiStyle-66/67/120/158 flag-minions.
    const MINION_KINDS: [u16; 13] = [
        HORNET,
        FLYING_IMP,
        TEMPEST,
        UFO_MINION,
        STARDUST_CELL,
        ABIGAIL,
        RETANIMINI,
        SPAZMAMINI,
        STARDUST_GUARDIAN,
        BABY_BIRD,
        STORM_TIGER_1,
        STORM_TIGER_2,
        STORM_TIGER_3,
    ];

    /// The 18 sentry kinds — the aiStyle-53 staff sentries, the Moon Lord
    /// turret pair, the four DD2 tower families and the Dead Cells barnacle.
    const SENTRY_KINDS: [u16; 18] = [
        FROST_HYDRA,
        QUEEN_SPIDER,
        HOUNDIUS_SHOOTIUS,
        MOONLORD_TURRET,
        RAINBOW_CRYSTAL,
        FLAME_BURST_TOWER_1,
        FLAME_BURST_TOWER_2,
        FLAME_BURST_TOWER_3,
        BALLISTRA_TOWER_1,
        BALLISTRA_TOWER_2,
        BALLISTRA_TOWER_3,
        LIGHTNING_AURA_1,
        LIGHTNING_AURA_2,
        LIGHTNING_AURA_3,
        EXPLOSIVE_TRAP_1,
        EXPLOSIVE_TRAP_2,
        EXPLOSIVE_TRAP_3,
        DEAD_CELLS_BARNACLE,
    ];

    /// DW-1.1: the 31 summon rows carry the verbatim `SetDefaults` slice —
    /// the box, aiStyle, penetrate, the `timeLeft` override (18000 for the
    /// `*= 5` minions, `SentryLifeTime` 36000 for the sentries) and the
    /// identification flags. Data-verbatim pattern
    /// (`the_family_data_map_is_verbatim`, `arrow_tests`).
    #[test]
    fn test_DW_1_1_the_summon_rows_are_verbatim() {
        // (kind, w, h, ai_style, penetrate, time_left, tile_collide, minion,
        //  sentry, minion_slots) — every value read off the C# arm cited in
        // the row's comment.
        let rows: [(u16, f32, f32, u8, i32, i32, bool, bool, bool, f32); 31] = [
            // `Projectile.cs:4563-4573` (373) / `:4587-4597` (375) /
            // `:4878-4890` (407) / `:4960-4971` (423) / `:6598-6608` (613) /
            // `:9100-9113` (963) / `:4707-4718` (387) / `:4723-4734` (388) /
            // `:6709-6723` (623) / `:8091-8098` (833) / `:8105-8112` (834) /
            // `:8119-8126` (835) / `:7829-7845` (759).
            (HORNET, 24.0, 26.0, 62, -1, 18000, false, true, false, 1.0),
            (
                FLYING_IMP, 34.0, 26.0, 62, -1, 18000, false, true, false, 1.0,
            ),
            (TEMPEST, 28.0, 40.0, 62, -1, 18000, false, true, false, 1.0),
            (
                UFO_MINION, 28.0, 28.0, 62, -1, 18000, true, true, false, 1.0,
            ),
            (
                STARDUST_CELL,
                24.0,
                24.0,
                62,
                -1,
                18000,
                true,
                true,
                false,
                1.0,
            ),
            (ABIGAIL, 30.0, 48.0, 62, -1, 18000, false, true, false, 0.0),
            (
                RETANIMINI, 40.0, 20.0, 66, -1, 18000, false, true, false, 0.5,
            ),
            (
                SPAZMAMINI, 40.0, 20.0, 66, -1, 18000, false, true, false, 0.5,
            ),
            (
                STARDUST_GUARDIAN,
                50.0,
                80.0,
                120,
                -1,
                18000,
                false,
                true,
                false,
                0.0,
            ),
            (
                STORM_TIGER_1,
                26.0,
                20.0,
                67,
                -1,
                18000,
                true,
                true,
                false,
                0.0,
            ),
            (
                STORM_TIGER_2,
                20.0,
                30.0,
                67,
                -1,
                18000,
                true,
                true,
                false,
                0.0,
            ),
            (
                STORM_TIGER_3,
                20.0,
                30.0,
                67,
                -1,
                18000,
                true,
                true,
                false,
                0.0,
            ),
            // `:7829-7845` (759).
            (
                BABY_BIRD, 10.0, 10.0, 158, -1, 18000, true, true, false, 1.0,
            ),
            // `Projectile.cs:3997-4007` (308) / `:4615-4622` (377) /
            // `:9143-9150` (966) / `:6916-6927` (641) / `:6929-6938` (643) /
            // `:7130-7151` (663/665/667) / `:7241-7249` (677-679) /
            // `:7325-7339` (688-690) / `:7340-7352` (691-693) /
            // `:9707-9714` (1025).
            (
                FROST_HYDRA,
                80.0,
                74.0,
                53,
                1,
                36000,
                true,
                false,
                true,
                0.0,
            ),
            (
                QUEEN_SPIDER,
                66.0,
                50.0,
                53,
                1,
                36000,
                true,
                false,
                true,
                0.0,
            ),
            (
                HOUNDIUS_SHOOTIUS,
                18.0,
                60.0,
                53,
                1,
                36000,
                true,
                false,
                true,
                0.0,
            ),
            (
                MOONLORD_TURRET,
                32.0,
                32.0,
                123,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                RAINBOW_CRYSTAL,
                32.0,
                32.0,
                123,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                FLAME_BURST_TOWER_1,
                30.0,
                54.0,
                130,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                FLAME_BURST_TOWER_2,
                28.0,
                58.0,
                130,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                FLAME_BURST_TOWER_3,
                28.0,
                60.0,
                130,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                BALLISTRA_TOWER_1,
                26.0,
                54.0,
                134,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                BALLISTRA_TOWER_2,
                26.0,
                54.0,
                134,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                BALLISTRA_TOWER_3,
                26.0,
                54.0,
                134,
                1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                LIGHTNING_AURA_1,
                16.0,
                16.0,
                137,
                -1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                LIGHTNING_AURA_2,
                16.0,
                16.0,
                137,
                -1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                LIGHTNING_AURA_3,
                16.0,
                16.0,
                137,
                -1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                EXPLOSIVE_TRAP_1,
                16.0,
                16.0,
                138,
                -1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                EXPLOSIVE_TRAP_2,
                16.0,
                16.0,
                138,
                -1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                EXPLOSIVE_TRAP_3,
                16.0,
                16.0,
                138,
                -1,
                36000,
                false,
                false,
                true,
                0.0,
            ),
            (
                DEAD_CELLS_BARNACLE,
                42.0,
                38.0,
                197,
                1,
                36000,
                true,
                false,
                true,
                0.0,
            ),
        ];
        for (kind, w, h, ai, pen, tl, tc, minion, sentry, slots) in rows {
            let d = proj_data(kind).unwrap_or_else(|| panic!("no row for kind {kind}"));
            assert_eq!((d.w, d.h), (w, h), "the box for kind {kind}");
            assert_eq!(d.ai_style, ai, "the aiStyle for kind {kind}");
            assert_ne!(d.ai_style, 0, "no summon row dispatches aiStyle 0");
            assert_eq!(d.penetrate, pen, "the penetrate for kind {kind}");
            assert_eq!(d.time_left, tl, "the timeLeft for kind {kind}");
            assert_eq!(d.tile_collide, tc, "the tileCollide for kind {kind}");
            assert_eq!(d.minion, minion, "the minion flag for kind {kind}");
            assert_eq!(d.sentry, sentry, "the sentry flag for kind {kind}");
            assert_eq!(d.minion_slots, slots, "the minionSlots for kind {kind}");
            assert!(d.net_important, "the netImportant for kind {kind}");
            // The stocks no summon arm touches.
            assert!(!d.arrow, "the stock arrow for kind {kind}");
            assert!(!d.trap, "the stock trap for kind {kind}");
            assert_eq!(d.extra_updates, 0, "the stock extraUpdates for kind {kind}");
            assert_eq!(d.scale, 1.0, "the stock scale for kind {kind}");
        }
        // `ignoreWater = true` on every summon arm but the Storm Tigers
        // (`:8091/:8105/:8119` set neither tileCollide nor ignoreWater).
        for kind in MINION_KINDS {
            let d = proj_data(kind).unwrap();
            assert_eq!(
                d.ignore_water,
                !matches!(kind, STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3),
                "the ignoreWater for kind {kind}"
            );
        }
        for kind in SENTRY_KINDS {
            assert!(proj_data(kind).unwrap().ignore_water, "kind {kind}");
        }
        // The idStatic pairs: Tempest 10 (`:4888-4889`), Retanimini 16
        // (`:4716-4717`), Spazmamini 12 (`:4732-4733`).
        let tempest = proj_data(TEMPEST).unwrap();
        assert!(tempest.uses_id_static);
        assert_eq!(tempest.id_static_hit_cooldown, 10);
        let retanimini = proj_data(RETANIMINI).unwrap();
        assert!(retanimini.uses_id_static);
        assert_eq!(retanimini.id_static_hit_cooldown, 16);
        let spazmamini = proj_data(SPAZMAMINI).unwrap();
        assert!(spazmamini.uses_id_static);
        assert_eq!(spazmamini.id_static_hit_cooldown, 12);
        // The local pairs: Guardian 5 (`:6721-6722`), Storm Tigers 10
        // (`:8096-8097` etc.), Abigail 20 (`:9111-9112`), Lightning Auras 3
        // (`:7332-7333`).
        let guardian = proj_data(STARDUST_GUARDIAN).unwrap();
        assert!(guardian.uses_local_immunity);
        assert_eq!(guardian.local_hit_cooldown, 5);
        for kind in [STORM_TIGER_1, STORM_TIGER_2, STORM_TIGER_3] {
            let d = proj_data(kind).unwrap();
            assert!(d.uses_local_immunity, "kind {kind}");
            assert_eq!(d.local_hit_cooldown, 10, "kind {kind}");
        }
        let abigail = proj_data(ABIGAIL).unwrap();
        assert!(abigail.uses_local_immunity);
        assert_eq!(abigail.local_hit_cooldown, 20);
        let baby_bird = proj_data(BABY_BIRD).unwrap();
        assert!(baby_bird.uses_local_immunity);
        assert_eq!(baby_bird.local_hit_cooldown, 15);
        for kind in [LIGHTNING_AURA_1, LIGHTNING_AURA_2, LIGHTNING_AURA_3] {
            let d = proj_data(kind).unwrap();
            assert!(d.uses_local_immunity, "kind {kind}");
            assert_eq!(d.local_hit_cooldown, 3, "kind {kind}");
        }
        // The arms without an immunity slice keep the pool defaults
        // (`:464`/`:466`).
        let hornet = proj_data(HORNET).unwrap();
        assert!(!hornet.uses_local_immunity);
        assert_eq!(hornet.local_hit_cooldown, -2);
        assert!(!hornet.uses_id_static);
        assert_eq!(hornet.id_static_hit_cooldown, -1);
        let hydra = proj_data(FROST_HYDRA).unwrap();
        assert!(!hydra.uses_local_immunity);
        assert!(!hydra.uses_id_static);
        // The shot families are NOT this phase's rows (the sentry shots land
        // with the later projectile-family tasks) — 374/376/389/614 the
        // minion shots, 378/379 the Queen Spider's SentryShots.
        for kind in [374, 376, 389, 614, 378, 379] {
            assert!(proj_data(kind).is_none(), "kind {kind} has no row yet");
        }
    }

    /// DW-1.2: the spawned slot carries the identification slice, the
    /// `timeLeft` override and the `originalDamage` the pool's recompute
    /// reads; a non-summon spawn keeps the stocks.
    #[test]
    fn test_DW_1_2_spawn_carries_the_summon_slice() {
        let mut pool = ProjectilePool::new();
        // The Hornet (`Player.SpawnMinionOnCursor`'s shape — the unscaled
        // damage in, `originalDamage` carried out).
        let hornet = pool
            .spawn_with_data_ai_owned(HORNET, 0, 100.0, 200.0, 0.0, 0.0, 10, 2.0, 0.0, 0.0)
            .unwrap();
        let p = &pool.slots[hornet];
        assert!(p.minion);
        assert!(!p.sentry);
        assert_eq!(p.minion_slots, 1.0);
        assert_eq!(p.time_left, 18000, "the `timeLeft *= 5` override");
        assert_eq!(p.original_damage, 10, "the unscaled spawn damage");
        assert_eq!(p.damage, 10);
        assert!(p.net_important);
        assert!(p.ignore_water);
        assert!(!p.tile_collide);
        assert_eq!(p.minion_pos, 0, "the pool block writes it (Phase 3)");
        // The Frost Hydra sentry — `SentryLifeTime`.
        let hydra = pool
            .spawn_with_data_ai_owned(FROST_HYDRA, 0, 160.0, 200.0, 0.0, 0.0, 25, 0.0, 0.0, 0.0)
            .unwrap();
        let p = &pool.slots[hydra];
        assert!(!p.minion);
        assert!(p.sentry);
        assert_eq!(p.minion_slots, 0.0);
        assert_eq!(p.time_left, 36000);
        assert_eq!(p.original_damage, 25);
        assert_eq!(p.penetrate, 1, "the sentry body's stock penetrate");
        // The half-slot spider: exactly 0.5 — never rounded to an integer
        // count (the plan's edge case).
        let spider = pool
            .spawn_with_data_ai_owned(RETANIMINI, 0, 220.0, 200.0, 0.0, 0.0, 8, 0.0, 0.0, 0.0)
            .unwrap();
        let p = &pool.slots[spider];
        assert_eq!(p.minion_slots, 0.5);
        assert_eq!(p.minion_slots.fract(), 0.5, "fractional, not rounded");
        assert!(p.uses_id_static_immunity, "the Retanimini idStatic pair");
        assert_eq!(p.id_static_hit_cooldown, 16);
        // The 0-slot Storm Tiger keeps `ignore_water = false` (the C# arm
        // sets neither flag).
        let tiger = pool
            .spawn_with_data_ai_owned(STORM_TIGER_1, 0, 280.0, 200.0, 0.0, 0.0, 8, 0.0, 0.0, 0.0)
            .unwrap();
        let p = &pool.slots[tiger];
        assert!(p.minion);
        assert_eq!(p.minion_slots, 0.0);
        assert!(p.net_important);
        assert!(!p.ignore_water);
        // A non-summon spawn keeps the stocks: no flags, `originalDamage` 0
        // (the C# spawn sites for the other families never write it).
        let arrow = pool
            .spawn_with_data_ai_owned(WOODEN_ARROW, 0, 320.0, 200.0, 4.0, 0.0, 5, 0.0, 0.0, 0.0)
            .unwrap();
        let p = &pool.slots[arrow];
        assert!(!p.minion);
        assert!(!p.sentry);
        assert_eq!(p.minion_slots, 0.0);
        assert_eq!(p.original_damage, 0);
        assert!(!p.net_important);
        assert!(!p.ignore_water);
    }

    /// DW-1.2's dirty variant (the plan's edge case): a summon row whose
    /// aiStyle has no ported arm dispatches nothing — the slot holds
    /// position while its `timeLeft` runs down (the Hornet's aiStyle 62 and
    /// the Guardian's 120 are Phase 4/5 work).
    #[test]
    fn test_edge_unported_ai_style_holds_position() {
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut w = world();
        let hornet = pool
            .spawn_with_data_ai_owned(HORNET, 0, 320.0, 320.0, 1.5, 0.0, 0, 0.0, 0.0, 0.0)
            .unwrap();
        let guardian = pool
            .spawn_with_data_ai_owned(
                STARDUST_GUARDIAN,
                0,
                480.0,
                320.0,
                0.0,
                -1.0,
                0,
                0.0,
                0.0,
                0.0,
            )
            .unwrap();
        for _ in 0..5 {
            pool.update(
                &mut w,
                &mut npcs,
                None,
                None,
                YoyoInputs::default(),
                &mut rng(),
                0,
                true,
            );
        }
        let p = &pool.slots[hornet];
        assert!(p.active, "the minion outlives the 5 ticks");
        assert_eq!(
            (p.pos.x, p.pos.y),
            (320.0, 320.0),
            "no arm ran: the slot holds position"
        );
        assert_eq!(p.time_left, 18000 - 5, "the bookkeeping still runs");
        assert_eq!(p.minion_pos, 0);
        let p = &pool.slots[guardian];
        assert!(p.active);
        assert_eq!((p.pos.x, p.pos.y), (480.0, 320.0));
        assert_eq!(p.time_left, 18000 - 5);
    }

    /// DW-1.3: the two identification sets match the C# lists
    /// element-for-element (`CreateBoolSet` semantics — the lookup is
    /// order-free, so the test compares sorted sequences and pins the
    /// verbatim counts).
    #[test]
    fn test_DW_1_3_the_sets_match_the_csharp() {
        // `ProjectileID.Sets.MinionSacrificable` (`ID/ProjectileID.cs:529`).
        let sacrificable: [u16; 36] = [
            191, 192, 193, 194, 266, 317, 373, 375, 387, 388, 390, 393, 407, 423, 533, 613, 755,
            758, 759, 831, 970, 864, 946, 951, 963, 1022, 1093, 1112, 1094, 1113, 1118, 1119, 625,
            626, 627, 628,
        ];
        assert_eq!(MINION_SACRIFICABLE.len(), 36);
        let mut ported = MINION_SACRIFICABLE.to_vec();
        ported.sort_unstable();
        let mut csharp = sacrificable.to_vec();
        csharp.sort_unstable();
        assert_eq!(ported, csharp, "element-for-element against the C# set");
        // The membership facts the later phases lean on: the port's own
        // minion kinds are sacrifice-candidates (Phase 6's kill loop), the
        // dragon chain rides along even though its AI is excluded.
        for kind in [
            HORNET,
            FLYING_IMP,
            TEMPEST,
            UFO_MINION,
            STARDUST_CELL,
            ABIGAIL,
        ] {
            assert!(MINION_SACRIFICABLE.contains(&kind), "kind {kind}");
        }
        for kind in [625u16, 626, 627, 628] {
            assert!(MINION_SACRIFICABLE.contains(&kind), "kind {kind}");
        }
        // `ProjectileID.Sets.SentryShot` (`ID/ProjectileID.cs:432`).
        let sentry_shot: [u16; 14] = [
            680, 664, 666, 668, 694, 695, 696, 644, 642, 378, 379, 309, 967, 1026,
        ];
        assert_eq!(SENTRY_SHOT.len(), 14);
        let mut ported_shots = SENTRY_SHOT.to_vec();
        ported_shots.sort_unstable();
        let mut csharp_shots = sentry_shot.to_vec();
        csharp_shots.sort_unstable();
        assert_eq!(ported_shots, csharp_shots);
        // The Queen Spider's shots (378/379) and Abigail's (967) — the
        // SentryShot consumers the later tasks wire.
        for kind in [378u16, 379, 967] {
            assert!(SENTRY_SHOT.contains(&kind), "kind {kind}");
        }
    }

    /// The new fields leave every pre-existing row on the pool defaults —
    /// the no-behaviour-change freeze (the
    /// `test_the_resolved_rows_leave_the_existing_spawns_untouched`
    /// precedent, widened to the summon slice).
    #[test]
    fn the_summon_fields_leave_the_existing_rows_untouched() {
        for kind in [
            WOODEN_ARROW,
            FLAME_ARROW,
            MUSKET_BALL,
            POISON_DART,
            TRAP_SPEAR,
            BOMB,
        ] {
            let d = proj_data(kind).unwrap();
            assert!(!d.minion, "kind {kind}");
            assert!(!d.sentry, "kind {kind}");
            assert_eq!(d.minion_slots, 0.0, "kind {kind}");
            assert!(!d.net_important, "kind {kind}");
            assert!(!d.ignore_water, "kind {kind}");
            let mut pool = ProjectilePool::new();
            let slot = pool
                .spawn_with_data_ai_owned(kind, 0, 100.0, 200.0, 2.0, 0.0, 7, 0.0, 0.0, 0.0)
                .unwrap();
            let p = &pool.slots[slot];
            assert!(!p.minion);
            assert!(!p.sentry);
            assert_eq!(p.minion_slots, 0.0);
            assert_eq!(p.original_damage, 0, "kind {kind}");
            assert_eq!(p.minion_pos, 0);
            assert!(!p.net_important);
            assert!(!p.ignore_water);
        }
    }

    /// The seeded determinism pair (the Test Plan's determinism rule): two
    /// identical 60-tick runs over a mixed summon pool settle into identical
    /// state.
    #[test]
    fn the_summon_pool_state_is_deterministic() {
        let run = || {
            let mut pool = ProjectilePool::new();
            let mut npcs: Vec<Npc> = Vec::new();
            let mut w = world();
            pool.spawn_with_data_ai_owned(HORNET, 0, 300.0, 300.0, 0.5, 0.0, 9, 0.0, 0.0, 0.0);
            pool.spawn_with_data_ai_owned(RETANIMINI, 0, 340.0, 300.0, -0.5, 0.0, 9, 0.0, 0.0, 0.0);
            pool.spawn_with_data_ai_owned(FROST_HYDRA, 0, 380.0, 300.0, 0.0, 0.0, 9, 0.0, 0.0, 0.0);
            for _ in 0..60 {
                pool.update(
                    &mut w,
                    &mut npcs,
                    None,
                    None,
                    YoyoInputs::default(),
                    &mut rng(),
                    0,
                    true,
                );
            }
            (
                pool.slots[0].clone(),
                pool.slots[1].clone(),
                pool.slots[2].clone(),
            )
        };
        let a = run();
        let b = run();
        assert_eq!(a.0.pos, b.0.pos);
        assert_eq!(a.0.time_left, b.0.time_left);
        assert_eq!(a.1.pos, b.1.pos);
        assert_eq!(a.2.time_left, b.2.time_left);
        assert!(a.2.active, "the sentry outlives 60 ticks");
    }
}
