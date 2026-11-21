//! The projectile data tables — the `SetDefaults` slice (`proj_data`,
//! `Projectile.cs`), the kind-id constants (`ProjectileID.cs`), the yoyo
//! `ProjectileID.Sets` tables (`ProjectileID.cs:425-427`), the flail
//! parameter table (`Projectile.cs:43746-43831`), the summon-system
//! constants (M4-PROJ-8) and `Main.projFrames` (`Main.cs:8434-8727`).
//! Extracted verbatim from `projectile.rs` (S1-G-1 entity half, split A);
//! behaviour frozen — the rows, defaults and lookups are byte-identical.

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
/// `ProjectileID.VilethornBase` (7) — the Vilethorn's first segment
/// (`Projectile.cs:874-884`).
pub const VILE_BASE: u16 = 7;
/// `ProjectileID.VilethornTip` (8) — the Vilethorn's last segment, the type
/// the base chains into once `ai[1] >= 6` (`Projectile.cs:23737-23741`).
pub const VILE_TIP: u16 = 8;
/// `ProjectileID.ThrowingKnife` (48) — the thrown aiStyle-2 kind
/// (`Projectile.cs:1310-1316`).
pub const THROWING_KNIFE: u16 = 48;
/// `ProjectileID.PoisonedKnife` (54) — the thrown aiStyle-2 kind with the
/// 1/2 Poisoned-on-hit roll (`Projectile.cs:1366-1374`, `:11628-11634`).
pub const POISONED_KNIFE: u16 = 54;
/// `ProjectileID.HolyWater` (69) — the thrown aiStyle-2 bottle, the hallow
/// splash on death (`Projectile.cs:1488-1495`, `:77629-77645`).
pub const HOLY_WATER: u16 = 69;
/// `ProjectileID.UnholyWater` (70) — the corruption twin of the holy water
/// (`Projectile.cs:1496-1503`).
pub const UNHOLY_WATER: u16 = 70;
/// `ProjectileID.BloodWater` (621) — the crimson twin of the holy water
/// (`Projectile.cs:1504-1511`).
pub const BLOOD_WATER: u16 = 621;
/// `ProjectileID.Grenade` (30) — the thrown aiStyle-16 bomb
/// (`Projectile.cs:1122-1130`).
pub const GRENADE: u16 = 30;
/// `ProjectileID.StickyGrenade` (397) — the sticky aiStyle-16 bomb,
/// `tileCollide = false` (`Projectile.cs:4781-4790`).
pub const STICKY_GRENADE: u16 = 397;
/// `ProjectileID.BouncyGrenade` (517) — the bouncy aiStyle-16 bomb
/// (`Projectile.cs:5870-5878`).
pub const BOUNCY_GRENADE: u16 = 517;
/// `ProjectileID.MolotovCocktail` (399) — the thrown aiStyle-68 bottle that
/// bursts into the three greek fires on death (`Projectile.cs:4801-4811`,
/// `:73921-73951`).
pub const MOLOTOV_COCKTAIL: u16 = 399;
/// `ProjectileID.MolotovFire` (400) — the molotov's first ground-fire child,
/// aiStyle 14 (`Projectile.cs:4812-4835`).
pub const MOLOTOV_FIRE_1: u16 = 400;
/// `ProjectileID.MolotovFire2` (401) — the molotov's second fire child.
pub const MOLOTOV_FIRE_2: u16 = 401;
/// `ProjectileID.MolotovFire3` (402) — the molotov's third fire child.
pub const MOLOTOV_FIRE_3: u16 = 402;
/// `ProjectileID.Beenade` (183) — the thrown aiStyle-14 bee grenade that
/// bursts into a swarm of bees on death (`Projectile.cs:2697-2706`,
/// `:76431-76463`).
pub const BEENADE: u16 = 183;
/// `ProjectileID.Bee` (181) — the beenade's small bee child, aiStyle 36
/// (`Projectile.cs:2673-2685`).
pub const BEE: u16 = 181;
/// `ProjectileID.GiantBee` (566) — the beenade's strong-bee child, aiStyle 36
/// (`Projectile.cs:6172-6184`).
pub const GIANT_BEE: u16 = 566;
/// `ProjectileID.BoneJavelin` (598) — the thrown aiStyle-113 sticking javelin
/// (`Projectile.cs:6470-6480`).
pub const BONE_JAVELIN: u16 = 598;
/// `ProjectileID.Javelin` (507) — the Javelin item's (3094) thrown aiStyle-1
/// spear, the `AI_001` 507/508/662/680/685 gravity group (`Projectile.cs:5770-5778`,
/// `:57471-57478`).
pub const JAVELIN: u16 = 507;
/// `ProjectileID.Glowstick` (50) — the thrown light, aiStyle 14
/// (`Projectile.cs:1326-1336`). S1-P-i42.
pub const GLOWSTICK: u16 = 50;
/// `ProjectileID.StickyGlowstick` (53) — aiStyle 14, `tileCollide = false`
/// (`Projectile.cs:1354-1365`); the stick scan lives in the AI arm
/// (`:24911-24962`). S1-P-i42.
pub const STICKY_GLOWSTICK: u16 = 53;
/// `ProjectileID.Flare` (163) — the flare gun's shot, aiStyle 33
/// (`Projectile.cs:2515-2525`). S1-P-i42.
pub const FLARE: u16 = 163;
/// `ProjectileID.BlueFlare` (310) — aiStyle 33 (`Projectile.cs:4022-4032`).
pub const BLUE_FLARE: u16 = 310;
/// `ProjectileID.SpelunkerGlowstick` (473) — aiStyle 14, `timeLeft *= 2`
/// (`Projectile.cs:5485-5495`). S1-P-i42.
pub const SPELUNKER_GLOWSTICK: u16 = 473;
/// `ProjectileID.BouncyGlowstick` (515) — aiStyle 14
/// (`Projectile.cs:5851-5861`); the bounce rides `HandleMovement`'s
/// 515-517 arm (`:18479-18489`). S1-P-i42.
pub const BOUNCY_GLOWSTICK: u16 = 515;
/// `ProjectileID.FairyGlowstick` (870) — aiStyle 14 until the first bounce
/// flips it to 170 (`Projectile.cs:8343-8353`, `:24832-24835`). S1-P-i42.
pub const FAIRY_GLOWSTICK: u16 = 870;
/// `ProjectileID.SpelunkerFlare` (1008) — aiStyle 33
/// (`Projectile.cs:9580-9590`). S1-P-i42.
pub const SPELUNKER_FLARE: u16 = 1008;
/// `ProjectileID.CursedFlare` (1009) — aiStyle 33 (`Projectile.cs:9591-9601`).
pub const CURSED_FLARE: u16 = 1009;
/// `ProjectileID.RainbowFlare` (1010) — aiStyle 33 (`Projectile.cs:9602-9612`).
pub const RAINBOW_FLARE: u16 = 1010;
/// `ProjectileID.ShimmerFlare` (1011) — aiStyle 33 (`Projectile.cs:9613-9623`).
pub const SHIMMER_FLARE: u16 = 1011;
/// `ProjectileID.RainbowGlowstick` (1089) — aiStyle 14, no alpha override
/// (`Projectile.cs:10153-10162`); the disco light rides `ProjLight`
/// (`:16434-16439`). S1-P-i42.
pub const RAINBOW_GLOWSTICK: u16 = 1089;
/// `ProjectileID.Snowball` (166) — the thrown aiStyle-2 kind with the
/// 20-tick/`+0.3f` ladder (`Projectile.cs:2549-2557`, `:23566-23596`).
pub const SNOWBALL: u16 = 166;
/// `ProjectileID.WandOfSparkingSpark` (954) — the starting magic weapon's
/// spark, aiStyle 2 with the 20-tick alpha-out kill (`Projectile.cs:9002-9011`,
/// `:23520-23556`).
pub const SPARK: u16 = 954;
/// `ProjectileID.WoodenArrow` (`Projectile.cs:807-815`).
pub const WOODEN_ARROW: u16 = 1;
/// `ProjectileID.WoodenArrowHostile` (`ProjectileID.cs:716`) — the
/// Goblin Archer's shot; the box/flight mirror the wooden arrow.
pub const WOODEN_ARROW_HOSTILE: u16 = 81;
/// `ProjectileID.FlamingArrow` (`ProjectileID.cs:718`) — the ElfArcher's
/// shot (the AI_003 shooter block's `num170 = 82` default,
/// `NPC.cs:59403`); the hostile twin of the flame arrow 2.
pub const FLAMING_ARROW_HOSTILE: u16 = 82;
/// S1-P-i17 — the special-arrow family (`ProjectileID.cs:556-2566`; the
/// ammo items' `shoot` values, `itemdata.rs`). The per-kind effects live in
/// `special_arrows.rs` (the AI_001 arms, the StatusNPC debuffs, the Kill()
/// spawns) and the pool's seams.
/// `ProjectileID.UnholyArrow` (`ProjectileID.cs:562`).
pub const UNHOLY_ARROW: u16 = 4;
/// `ProjectileID.JestersArrow` (`ProjectileID.cs:564`).
pub const JESTERS_ARROW: u16 = 5;
/// `ProjectileID.HellfireArrow` (`ProjectileID.cs:636`).
pub const HELLFIRE_ARROW: u16 = 41;
/// `ProjectileID.HolyArrow` (`ProjectileID.cs:736`).
pub const HOLY_ARROW: u16 = 91;
/// `ProjectileID.FallingStar` (92) — the Holy Arrow's death spawn
/// (`Projectile.cs:75212-75248`, aiStyle 5).
pub const FALLING_STAR: u16 = 92;
/// `ProjectileID.CursedArrow` (`ProjectileID.cs:760`).
pub const CURSED_ARROW: u16 = 103;
/// `ProjectileID.BoneArrow` (`ProjectileID.cs:788`) — the Bone Bow's 117
/// (the merchant's 474 converts through `Player.cs:53511-53514`).
pub const BONE_ARROW: u16 = 117;
/// `ProjectileID.FrostArrow` (`ProjectileID.cs:794`) — the Ice Bow's shot.
pub const FROST_ARROW: u16 = 120;
/// `ProjectileID.FrostburnArrow` (`ProjectileID.cs:898`).
pub const FROSTBURN_ARROW: u16 = 172;
/// `ProjectileID.ChlorophyteArrow` (`ProjectileID.cs:1004`).
pub const CHLOROPHYTE_ARROW: u16 = 225;
/// `ProjectileID.IchorArrow` (`ProjectileID.cs:1110`).
pub const ICHOR_ARROW: u16 = 278;
/// `ProjectileID.VenomArrow` (`ProjectileID.cs:1118`).
pub const VENOM_ARROW: u16 = 282;
/// `ProjectileID.BeeArrow` (`ProjectileID.cs:1492`).
pub const BEE_ARROW: u16 = 469;
/// `ProjectileID.BoneArrowFromMerchant` (`ProjectileID.cs:1502`).
pub const BONE_ARROW_MERCHANT: u16 = 474;
/// `ProjectileID.ShadowFlameArrow` (`ProjectileID.cs:1544`).
pub const SHADOWFLAME_ARROW: u16 = 495;
/// `ProjectileID.BloodArrow` (`ProjectileID.cs:2192`) — the Blood Rain
/// Bow's rain shot.
pub const BLOOD_ARROW: u16 = 819;
/// `ProjectileID.ShimmerArrow` (`ProjectileID.cs:2566`).
pub const SHIMMER_ARROW: u16 = 1006;
/// The Destroyer's segment laser (`NPC.cs`'s `num11 = 100`, `:49886`).
pub const DESTROYER_LASER: u16 = 100;
/// `ProjectileID.SnowBallHostile` (109) — the Snow Balla's (net 145,
/// the port's `MysterySnowman`) aimed throw (`aiStyle 10`,
/// `Projectile.cs` type 109's SetDefaults).
pub const SNOW_BALL_HOSTILE: u16 = 109;
/// `ProjectileID.BulletSnowman` (110) — the Snowman Gangsta's bottle
/// throw (`aiStyle 1`, `Projectile.cs` type 110's SetDefaults).
pub const BULLET_SNOWMAN: u16 = 110;
/// `ProjectileID.Fireball` (`ProjectileID.cs`'s 258) — the Golem's head
/// and free-head fireballs (`NPC.cs:31200`/`:31460`'s `num686`/`num720`).
pub const GOLEM_FIREBALL: u16 = 258;
/// `ProjectileID.EyeBeam` (259) — the Golem's eye beams
/// (`NPC.cs:31282`'s `num698`, `:31565`'s `num729`).
pub const GOLEM_EYE_BEAM: u16 = 259;
/// `ProjectileID.QueenBeeStinger` (719, `ProjectileID.cs:1992`) — the
/// Queen Bee's state-3 barrage (`NPC.cs:30742`'s shot, dmg 11, the
/// post-spawn `timeLeft = 300` `:30743`).
pub const QUEEN_BEE_STINGER: u16 = 719;
/// `ProjectileID.Stinger` (55, `ProjectileID.cs`) — the Hornet's shot
/// (`NPC.cs:50796`'s type 55, dmg `(int)(10*scale)`, the post-spawn
/// `timeLeft = 300` `:50797`; `Projectile.cs:1375-1383`: 10×10, hostile,
/// aiStyle 1, penetrate −1, tileCollide; the `flag3 = false` flight list
/// `:55954` keeps the counter at rest — dead-straight, no gravity).
pub const HORNET_STINGER: u16 = 55;
/// `ProjectileID.EyeLaser` (83) — the Retinazer's phase-1 laser
/// (`NPC.cs:26628`'s `num412`).
pub const EYE_LASER: u16 = 83;
/// `ProjectileID.DeathLaser` (84) — the Probe's laser (`NPC.cs:50619`'s
/// `num11 = 84`; `Projectile.cs:1646-1659`: 4×4, aiStyle 1, hostile,
/// penetrate 3, extraUpdates 2, timeLeft 600).
pub const PROBE_LASER: u16 = 84;
/// `ProjectileID.CursedFlameHostile` (96) — the Spazmatism's phase-1 spit
/// (`NPC.cs:27223`'s `num451`).
pub const CURSED_FLAME: u16 = 96;
/// `ProjectileID.EyeFire` (101) — the Spazmatism's phase-2 spit
/// (`NPC.cs:27519`'s `num467`).
pub const EYE_FIRE: u16 = 101;
/// `ProjectileID.FlamesTrap` (188) — the Spazmatism's phase-2 flamethrower
/// (`Projectile.cs:2752-2763`: 6x6, aiStyle 23, friendly + hostile, trap,
/// penetrate -1, extraUpdates 2).
pub const FLAMES_TRAP: u16 = 188;
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
/// `ProjectileID.MoonlordBullet` (638) — the S.D.M.G.'s bullet
/// (`Projectile.cs:6858-6872`: 4x4, aiStyle 1, extraUpdates 5, timeLeft 600,
/// penetrate -1, ignoreWater, the local-immunity −1 pair). The AI_001 arm
/// rides `projectile_ai_moonlord` (S1-D-4).
pub const MOONLORD_BULLET: u16 = 638;
/// `ProjectileID.MoonlordArrow` (639) — the Phantasm's arrow
/// (`Projectile.cs:6873-6888`: 10x10, aiStyle 1, arrow, `MaxUpdates = 2`,
/// `timeLeft = MaxUpdates * 45` = 90, penetrate 4, the local-immunity −1
/// pair). Its `Kill()` spawns the 640 trail (`:73431-73452`).
pub const MOONLORD_ARROW: u16 = 639;
/// `ProjectileID.MoonlordArrowTrail` (640) — the arrow's ghost
/// (`Projectile.cs:6889-6903`: 10x10, aiStyle 1, `MaxUpdates = 3`,
/// timeLeft 90, penetrate 4, the local-immunity −1 pair). The `ai[1]`
/// spawn-delay countdown (`:16679-16683`) is pool-side.
pub const MOONLORD_ARROW_TRAIL: u16 = 640;
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
/// The Frost Moon projectile family (S1-D-4, `ProjectileID.cs:1244-1258`).
/// `ProjectileID.PineNeedleHostile` (345) — Everscream's needle spray
/// (`NPC.cs:32838`, `ai0 = Main.rand.Next(0, 31)`).
pub const PINE_NEEDLE_HOSTILE: u16 = 345;
/// `ProjectileID.OrnamentHostile` (346) — Everscream's ornament lob
/// (`NPC.cs:32885`, `ai1 = Main.rand.Next(2)` picks the frame); on its last
/// tick it bursts into five 347 shrapnel (`Projectile.cs:24883-24896`).
pub const ORNAMENT_HOSTILE: u16 = 346;
/// `ProjectileID.OrnamentHostileShrapnel` (347) — the ornament's burst
/// fragment (the aiStyle-2 5-tick arm, `Projectile.cs:23503-23510`).
pub const ORNAMENT_SHRAPNEL: u16 = 347;
/// `ProjectileID.FrostWave` (348) — IceQueen's phase-0 side charge
/// (`NPC.cs:33500`).
pub const FROST_WAVE: u16 = 348;
/// `ProjectileID.FrostShard` (349) — IceQueen's shard rain / spin barrage
/// (`NPC.cs:33607`'s `ai0 = Main.rand.Next(5)` frame pick, `:33656`).
pub const FROST_SHARD: u16 = 349;
/// `ProjectileID.Missile` (350) — SantaNK1's missile salvo
/// (`NPC.cs:33818`). Alias of `MISSILE` (the S1-F-13c row name) — one
/// const per id; the S1-D-4 santa slice and the F-13c wiring landed the
/// same ids independently (deduped at merge).
pub use MISSILE as SANTA_MISSILE;
/// `ProjectileID.Present` (351) — SantaNK1's present rain
/// (`NPC.cs:33840`, the aiStyle-58 drop). Alias of `PRESENT`.
pub use PRESENT as SANTA_PRESENT;
/// `ProjectileID.Spike` (352) — SantaNK1's ornament spike
/// (`NPC.cs:33798`, the aiStyle-14 alpha/scale pulse). Alias of `SPIKE`.
pub use SPIKE as SANTA_SPIKE;
/// `ProjectileID.DeerclopsIceChunk` (962, `Projectile.cs:9199-9206`) — the
/// Deerclops' rubble chunk (the aiStyle-1 flight).
pub const DEERCLOPS_ICE_CHUNK: u16 = 962;
/// `ProjectileID.FlamingWood` (325, `Projectile.cs:3249-3259`) — the
/// Retinazer's shot.
pub const FLAMING_WOOD: u16 = 325;
// ---- The Frost Moon bosses' hostile shots (S1-F-13c wiring,
// `ProjectileID.cs:1244-1258`). The Everscream / IceQueen / SantaNK1
// machines (`npc_ai_frost_boss*.rs`) queue these; the rows below let the
// drained `wants_hostile_shot(_ai)` requests spawn + fly. The 345/346/
// 348/349 consts are the S1-D-4 family block above (deduped at merge —
// one const per id; the rows are value-identical in both branches).
/// `ProjectileID.Missile` (350, `Projectile.cs:4400-4410`) — the SantaNK1's
/// missile salvo (14x14, aiStyle 1, penetrate -1, tileCollide false,
/// alpha 255, timeLeft 300).
pub const MISSILE: u16 = 350;
/// `ProjectileID.Present` (351, `Projectile.cs:4411-4420`) — the SantaNK1's
/// present rain (24x24, aiStyle 58, penetrate -1, tileCollide false,
/// alpha 255).
pub const PRESENT: u16 = 351;
/// `ProjectileID.Spike` (352, `Projectile.cs:4421-4429`) — the SantaNK1's
/// spike (30x30, aiStyle 14, penetrate -1, timeLeft 1200).
pub const SPIKE: u16 = 352;
/// `ProjectileID.GreekFire1` (326, `Projectile.cs:3260-3277`) — the
/// Retinazer's Greek fire.
pub const GREEK_FIRE_1: u16 = 326;
/// `ProjectileID.GreekFire2` (327) — the Greek fire's second size.
pub const GREEK_FIRE_2: u16 = 327;
/// `ProjectileID.GreekFire3` (328) — the Greek fire's third size.
pub const GREEK_FIRE_3: u16 = 328;
/// `ProjectileID.Skull` (270, `Projectile.cs:3558-3566`) — the Skeletron
/// head's spin shot (the expert mode's fired skull).
pub const SKULL: u16 = 270;
/// `ProjectileID.SeedPlantera` (275, `Projectile.cs:3614-3621`) —
/// Plantera's phase-1 seed (the aiStyle-1 flight, hostile, alpha 255).
pub const SEED_PLANTERA: u16 = 275;
/// `ProjectileID.PoisonSeedPlantera` (276, `Projectile.cs:3622-3629`) —
/// the poison seed (the same 14x14 aiStyle-1 row).
pub const POISON_SEED_PLANTERA: u16 = 276;
/// `ProjectileID.ThornBall` (277, `Projectile.cs:3630-3637`) — the thorn
/// ball (38x38, the aiStyle-14 flight, hostile, alpha 255).
pub const THORN_BALL: u16 = 277;
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
/// `ProjectileID.BulletHighVelocity` (242, `Projectile.cs:3238-3254`) — the
/// Sniper/Venus Magnum/Uzi conversion bullet (S1-P-i14's row; the constant is
/// S1-P-i18's `HIGH_VELOCITY_BULLET` below — the V5 gate kept one name).
/// `ProjectileID.Xenopopper` (444, `Projectile.cs:5150-5161`) — the Xenopopper
/// gun's bubble projectile (S1-P-i14).
pub const XENOPOPPER_BUBBLE: u16 = 444;
/// `ProjectileID.BlackBolt` (661, `Projectile.cs:7108-7121`) — the Onyx
/// Blaster's dark shot (S1-P-i14).
pub const ONYX_BLACK_BOLT: u16 = 661;
/// `ProjectileID.PewMaticHornShot` (968, `Projectile.cs:9160-9168`) — the
/// Pew-matic Horn's projectile (S1-P-i14).
pub const PEW_MATIC_SHOT: u16 = 968;
/// `ProjectileID.MeteorShot` (36, `ProjectileID.cs:626`) — the Meteor Shot
/// bullet: the one-bounce aiStyle-1 shot (`Projectile.cs:1182-1197`, the
/// bounce `HandleMovement` `:19278-19298`). S1-P-i18.
pub const METEOR_SHOT: u16 = 36;
/// `ProjectileID.CrystalBullet` (89, `ProjectileID.cs:732`) — the Crystal
/// Bullet: on death splits into two CrystalShard 90 (`Projectile.cs:1715-1728`,
/// the split `Kill()` `:75250-75268`). S1-P-i18.
pub const CRYSTAL_BULLET: u16 = 89;
/// `ProjectileID.CrystalShard` (90, `ProjectileID.cs:734`) — the Crystal
/// Bullet's death shard: aiStyle 24, `tileCollide = false`
/// (`Projectile.cs:1729-1742`). S1-P-i18.
pub const CRYSTAL_SHARD: u16 = 90;
/// `ProjectileID.CursedBullet` (104, `ProjectileID.cs:762`) — the Cursed
/// Bullet: inflicts CursedInferno 39 (`Projectile.cs:1915-1928`, `StatusNPC`
/// `:11665-11668`). S1-P-i18.
pub const CURSED_BULLET: u16 = 104;
/// `ProjectileID.ChlorophyteBullet` (207, `ProjectileID.cs:968`) — the
/// Chlorophyte Bullet: the homing aiStyle-1 shot (`Projectile.cs:2948-2961`,
/// the homing `AI_001` `:56778-56887`). S1-P-i18.
pub const CHLOROPHYTE_BULLET: u16 = 207;
/// `ProjectileID.BulletHighVelocity` (242, `ProjectileID.cs:1038`) — the
/// High Velocity Bullet: the fast pierce-3 shot the Sniper Rifle / Venus
/// Magnum / Uzi convert the musket ball into (`Projectile.cs:3238-3254`, the
/// conversion `Player.cs:47107-47118`, the speed fade `AI_001`
/// `:55660-55671`). S1-P-i18.
pub const HIGH_VELOCITY_BULLET: u16 = 242;
/// `ProjectileID.IchorBullet` (279, `ProjectileID.cs:1112`) — the Ichor
/// Bullet: inflicts Ichor 69 (`Projectile.cs:3650-3663`, `StatusNPC`
/// `:11669-11672`). S1-P-i18.
pub const ICHOR_BULLET: u16 = 279;
/// `ProjectileID.VenomBullet` (283, `ProjectileID.cs:1120`) — the Venom
/// Bullet: inflicts Venom 70 + PotentAcid 395 and bursts on death
/// (`Projectile.cs:3699-3712`, `StatusNPC` `:11673-11677`, the burst `Kill()`
/// `:75911-75974`). S1-P-i18.
pub const VENOM_BULLET: u16 = 283;
/// `ProjectileID.PartyBullet` (284, `ProjectileID.cs:1122`) — the Party
/// Bullet: the confetti death burst (`Projectile.cs:3713-3726`, `Kill()`
/// `:75975-75999`). S1-P-i18.
pub const PARTY_BULLET: u16 = 284;
/// `ProjectileID.NanoBullet` (285, `ProjectileID.cs:1124`) — the Nano Bullet:
/// inflicts Confused 31, bounces once and re-targets (`Projectile.cs:3727-3740`,
/// `StatusNPC` `:11541-11551`, the bounce `HandleMovement` `:18563-18591`, the
/// `PickAmmo` ×1.25 `Player.cs:53556-53559`). S1-P-i18.
pub const NANO_BULLET: u16 = 285;
/// `ProjectileID.ExplosiveBullet` (286, `ProjectileID.cs:1126`; the ITEM is
/// `ItemID.ExplodingBullet`) — the Exploding Bullet: the idStatic-60 AoE death
/// burst (`Projectile.cs:3741-3757`, `Kill()` `:76000-76034`). S1-P-i18.
pub const EXPLODING_BULLET: u16 = 286;
/// `ProjectileID.GoldenBullet` (287, `ProjectileID.cs:1128`) — the Golden
/// Bullet: inflicts Midas 72 (`Projectile.cs:3758-3771`, `StatusNPC`
/// `:11537-11540`). S1-P-i18.
pub const GOLDEN_BULLET: u16 = 287;
/// The special-bullet kind set S1-P-i18 owns (the aiStyle-1 arms in
/// `projectile_special_bullets`). Disjoint from the ai001 A/B/C kind lists,
/// the riders and the moon-lord set. 90 CrystalShard is aiStyle 24 (the
/// existing `ai_crystal_storm` arm) and 638 MoonlordBullet is S1-D-4's — both
/// excluded here.
pub const SPECIAL_BULLET_KINDS: [u16; 11] = [
    METEOR_SHOT,
    CRYSTAL_BULLET,
    CURSED_BULLET,
    CHLOROPHYTE_BULLET,
    HIGH_VELOCITY_BULLET,
    ICHOR_BULLET,
    VENOM_BULLET,
    PARTY_BULLET,
    NANO_BULLET,
    EXPLODING_BULLET,
    GOLDEN_BULLET,
];
/// `ProjectileID.PoisonDartBlowgun`-family dart the Dart Trap fires
/// (`Projectile.cs:1823-1832` — the Wiring emission uses 98, `Wiring.cs:1785`).
pub const TRAP_DART: u16 = 98;
/// The Super Dart Trap's projectile (`Projectile.cs:2707-2716`).
pub const TRAP_SUPERDART: u16 = 184;
/// The Spiky Ball Trap's projectile (`Projectile.cs:2717-2727` — the C#
/// `SpikyBallTrap = 185`, aiStyle 14, 14×14, `timeLeft = 900`; the WIRE-1
/// legacy named it TRAP_SPEAR; S1-D-4c fixed the row's aiStyle 1 → 14).
pub const TRAP_SPIKY_BALL: u16 = 185;
/// The Spear Trap's projectile (the C# `SpearTrap = 186`,
/// `Projectile.cs:2728-2740`: **aiStyle 37**, 10×14, `tileCollide = false`,
/// `ignoreWater`, `timeLeft = 300`). The port's row deliberately keeps the
/// WIRE-1 legacy values (aiStyle 14, 14×14, 900 — the C# 185's row, a
/// historical 185/186 swap); the faithful 37 row + `AI_037` port are a
/// follow-up — ANALYSIS "The spiky ball's row fix (S1-D-4c)".
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
pub const FLAIRON_BUBBLE: u16 = 405;
/// The spear family (aiStyle 19 — `AI_019_Spears` `Projectile.cs:45857-46073`,
/// S1-P-i06). Every row rides `DefaultToSpear()` (`:10720-10730`: aiStyle 19,
/// friendly, `penetrate = -1`, `tileCollide = false`, drawLayer 7,
/// `usesOwnerLight`, `ownerHitCheck`, melee); the per-kind `SetDefaults` rows
/// set the boxes and scales. The ids are the verbatim `ProjectileID`
/// (`ProjectileID.cs`). NOTE: the trap spear 186 (`TRAP_SPEAR`) is aiStyle 37,
/// NOT a member — the held-spear family is the player's useStyle-5 thrusts.
pub const DARK_LANCE: u16 = 46;
pub const TRIDENT: u16 = 47;
pub const SPEAR: u16 = 49;
pub const MYTHRIL_HALBERD: u16 = 64;
pub const ADAMANTITE_GLAIVE: u16 = 66;
pub const COBALT_NAGINATA: u16 = 97;
pub const GUNGNIR: u16 = 105;
pub const MUSHROOM_SPEAR: u16 = 130;
pub const THE_ROTTED_FORK: u16 = 153;
pub const PALLADIUM_PIKE: u16 = 212;
pub const ORICHALCUM_HALBERD: u16 = 215;
pub const TITANIUM_TRIDENT: u16 = 218;
pub const CHLOROPHYTE_PARTISAN: u16 = 222;
pub const NORTH_POLE: u16 = 342;
pub const OBSIDIAN_SWORDFISH: u16 = 367;
pub const SWORDFISH: u16 = 368;
pub const THUNDER_SPEAR: u16 = 730;
pub const JOUSTING_LANCE: u16 = 877;
pub const SHADOW_JOUSTING_LANCE: u16 = 878;
pub const HALLOW_JOUSTING_LANCE: u16 = 879;
pub const SLIME_SPEAR: u16 = 1103;
/// The whole aiStyle-19 weapon set — the 21 kinds `DefaultToSpear` covers
/// (`Projectile.cs:1300/1308/1324/1447/1465/1821/1934/2196/2416/3009/3029/
/// 3050/3084/4323/4522/4528/7692/8435/10287`).
pub const SPEAR_KINDS: [u16; 21] = [
    DARK_LANCE,
    TRIDENT,
    SPEAR,
    MYTHRIL_HALBERD,
    ADAMANTITE_GLAIVE,
    COBALT_NAGINATA,
    GUNGNIR,
    MUSHROOM_SPEAR,
    THE_ROTTED_FORK,
    PALLADIUM_PIKE,
    ORICHALCUM_HALBERD,
    TITANIUM_TRIDENT,
    CHLOROPHYTE_PARTISAN,
    NORTH_POLE,
    OBSIDIAN_SWORDFISH,
    SWORDFISH,
    THUNDER_SPEAR,
    JOUSTING_LANCE,
    SHADOW_JOUSTING_LANCE,
    HALLOW_JOUSTING_LANCE,
    SLIME_SPEAR,
];
/// The spear family's sub-shots (S1-P-i06) — the projectiles `AI_019_Spears`
/// spawns (`Projectile.cs:45891-45936`). They ride their OWN aiStyles, not 19:
/// the Mushroom Spear's spore 131 (aiStyle 30, `:2198-2208`), the Chlorophyte
/// Partisan's spore cloud 228 (aiStyle 44, `:3137-3150`), the Thunder Spear's
/// bolt 732 (aiStyle 44, `:7705-7715`), the North Pole's snowflake 343
/// (aiStyle 57, `:4326-4338`) and its falling flake 344 (aiStyle 1,
/// `:4339-4352`), and the Obsidian Swordfish's bones 1107-1109 (aiStyle 14,
/// `:10316-10339`).
pub const MUSHROOM_SPORE: u16 = 131;
pub const CHLOROPHYTE_SPORE_CLOUD: u16 = 228;
pub const THUNDER_SPEAR_BOLT: u16 = 732;
pub const NORTH_POLE_SNOWFLAKE: u16 = 343;
pub const NORTH_POLE_FLAKE: u16 = 344;
pub const SWORDFISH_BONE_1: u16 = 1107;
pub const SWORDFISH_BONE_2: u16 = 1108;
pub const SWORDFISH_BONE_3: u16 = 1109;
/// `ProjectileID.Bomb`-family cannon ammo (178, `Projectile.cs:2641-2649`).
pub const CANNON_BOMB: u16 = 178;
/// The explosive cannonball (601, `Projectile.cs:6499-6512`) — aiStyle 1 with
/// `extraUpdates = 30`.
pub const CANNON_EXPLOSIVE: u16 = 601;
/// The bunny cannon's projectile (281, `Projectile.cs:3676-3687` — aiStyle 49,
/// no ported AI; the constant anchors the `BunnyCannonCanFire` scan).
pub const CANNON_BUNNY: u16 = 281;
/// `ProjectileID.LeafBlower` (`Projectile.cs:2937-2945`): the Leaf Blower's
/// 14x14 petal, aiStyle 40 (`AI_040_LeafBlower` `:38003-38082`).
pub const LEAF_BLOWER: u16 = 206;

/// The Lunar Flare's shot (645, `ProjectileID.LunarFlare`,
/// `ProjectileID.cs:1844`) — the moon-lord drop spell
/// (`Projectile.cs:6953-6965`: 10x10, aiStyle 1, magic, tileCollide false,
/// extraUpdates 5, penetrate −1, the local-immunity −1 pair).
/// NAME DEVIATION (recorded, S1-D-4 census): the constant was ported as
/// `POISON_DART` citing `:6953-6966` as "the Blowgun's dart" — but the C#
/// `ProjectileID.PoisonDart` is **98** (`ProjectileID.cs:750`, the Dart
/// Trap's shot, already rowed as `TRAP_DART`); `:6953-6965` is the 645
/// LunarFlare block. The value 645 and the row are correct for the
/// LunarFlare; the rename is deferred to the in-flight ai001-C slice
/// (`task/S1-D-5-aistyle1-types-c` owns the 645 arm under this name).
pub const POISON_DART: u16 = 645;
/// `ProjectileID.EnchantedBoomerang` (`Projectile.cs:864-873`).
pub const ENCHANTED_BOOMERANG: u16 = 6;
/// `ProjectileID.WoodenBoomerang` (`Projectile.cs:1345-1353`).
pub const WOODEN_BOOMERANG: u16 = 52;
/// `ProjectileID.FrostBoomerang` — the Ice Boomerang (`Projectile.cs:2012-2022`).
pub const ICE_BOOMERANG: u16 = 113;
/// `ProjectileID.LightDisc` (106, `Projectile.cs:1936-1945`) — the Light Disc
/// (S1-P-i08): the stack-limit boomerang variant, up to 6 in flight
/// (`Player.cs:52172-52186`), the 45-tick outbound window
/// (`Projectile.cs:39035-39040`), the 16/1.2 return seek (`:39432-39436`),
/// the per-axis wall bounce that does NOT arm the return (`:19550-19560`)
/// and the 0.3 spin (`:39556-39559`).
pub const LIGHT_DISC: u16 = 106;
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
/// `ProjectileID.LightsBane` (974) — the Light's Bane sword's slash wave
/// (`Projectile.cs:9237-9252`'s SetDefaults, `AI_188_LightsBane`
/// `:42229-42262`, `Main.DrawProj_LightsBane` `Main.cs:27455-27495`).
pub const LIGHTS_BANE: u16 = 974;

// ===== S1-D-4: the Empress of Light's projectile family =====
// The five kinds the Empress `AI_120_HallowBoss` machine spawns (R7's
// `npc_ai_empress.rs` `EmpressRequest::Projectile` set). Ids verified against
// `ID/ProjectileID.cs`; the aiStyle + box + flags against each `SetDefaults`
// arm. The AI cores live in `projectile_ai_empress.rs` (unwired, code-only).
/// `ProjectileID.HallowBossLastingRainbow` (872, `ProjectileID.cs:2298`) —
/// the lasting-rainbow ring bolt, aiStyle 173 (`Projectile.cs:8403-8415`).
pub const HALLOW_BOSS_LASTING_RAINBOW: u16 = 872;
/// `ProjectileID.HallowBossRainbowStreak` (873, `ProjectileID.cs:2300`) —
/// the homing prismatic bolt, aiStyle 171 (`Projectile.cs:8354-8366`).
pub const HALLOW_BOSS_RAINBOW_STREAK: u16 = 873;
/// `ProjectileID.HallowBossDeathAurora` (874, `ProjectileID.cs:2302`) —
/// the death-aurora flash, aiStyle **0** (no AI arm, `Projectile.cs:8367-8379`).
pub const HALLOW_BOSS_DEATH_AURORA: u16 = 874;
/// `ProjectileID.FairyQueenLance` (919, `ProjectileID.cs:2392`) — the
/// telegraph-then-dash ethereal lance, aiStyle 179 (`Projectile.cs:8714-8726`).
pub const FAIRY_QUEEN_LANCE: u16 = 919;
/// `ProjectileID.FairyQueenSunDance` (923, `ProjectileID.cs:2400`) — the
/// boss-tethered sun-dance beam, aiStyle 180 (`Projectile.cs:8757-8770`).
pub const FAIRY_QUEEN_SUN_DANCE: u16 = 923;

/// The projectile kinds `proj_data` carries — the set the atlas packs
/// `Images/Projectile_{id}` for. Derived from the table itself rather than a
/// second hand-maintained list: the id space above the port's highest row is
/// empty, so the probe stops at 1200.
pub fn spawnable_kinds() -> Vec<u16> {
    (0..=1200u16).filter(|k| proj_data(*k).is_some()).collect()
}

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
/// `ProjectileID.AquaScepter` (22) — the Aqua Scepter's water stream
/// (aiStyle 12, `Projectile.cs:1040-1056`); the lava bounce seeds an
/// `LCG32Random` off the spawn `ai[2]` (`Player.cs:49409-49411`).
pub const AQUA_SCEPTER: u16 = 22;
/// `ProjectileID.WaterBolt` (27) — the Water Bolt spell.
pub const WATER_BOLT: u16 = 27;
/// `ProjectileID.Flamelash` (34) — the channeled aimed spell.
pub const FLAMELASH: u16 = 34;
/// `ProjectileID.DemonSickle` (44) — the Demon/VoodooDemon's hostile sickle
/// (`NPC.cs:23449`), the aiStyle-18 twin of the friendly DemonScythe 45
/// (`Projectile.cs:1271-1282`: hostile, penetrate −1).
pub const DEMON_SICKLE: u16 = 44;
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
/// `ProjectileID.RainCloudMoving` (237) — the Nimbus Rod's flying cloud
/// (aiStyle 45); on death it spawns the raining cloud 238.
pub const RAIN_CLOUD_MOVING: u16 = 237;
/// `ProjectileID.RainCloudRaining` (238) — the settled cloud (aiStyle 45);
/// it rains the friendly drop 239 every 9 ticks and caps the owner at two.
pub const RAIN_CLOUD_RAINING: u16 = 238;
/// `ProjectileID.RainFriendly` (239) — the cloud's rain drop (aiStyle 45).
pub const RAIN_FRIENDLY: u16 = 239;
/// `ProjectileID.GoldenShowerFriendly` (280) — the Golden Shower's stream
/// (aiStyle 12).
pub const GOLDEN_SHOWER: u16 = 280;
/// `ProjectileID.ShadowBeamFriendly` (294) — the Shadowbeam Staff's beam
/// (aiStyle 48); it reflects off tiles instead of dying.
pub const SHADOWBEAM: u16 = 294;
/// `ProjectileID.InfernoFriendlyBolt` (295) — the Inferno Fork's bolt
/// (aiStyle 50); it spawns the blast 296 on death.
pub const INFERNO_FORK_BOLT: u16 = 295;
/// `ProjectileID.Blast` (296) — the fork's on-death explosion (aiStyle 50).
pub const INFERNO_BLAST: u16 = 296;
/// `ProjectileID.VenomFang` (355) — the Poison Staff's aiStyle-1 rider.
pub const VENOM_FANG: u16 = 355;
/// `ProjectileID.FrostBoltStaff` (359) — the Frost Staff's bolt.
pub const FROST_BOLT: u16 = 359;
/// The fishing bobbers (S1-P-i35): the plain pole family 360-366
/// (`ProjectileID.BobberWooden`..`BobberSittingDuck`), the Fleshcatcher /
/// Hotline 381/382, the Blood rod 760 and the Scarab 775
/// (`Projectile.cs:4508-4516`), plus the glow-bobber accessory overrides
/// 986-993 (`:9422-9435`). All aiStyle 61, 14x14, penetrate -1,
/// netImportant, `bobber = true`.
pub const BOBBER_WOODEN: u16 = 360;
pub const BOBBER_REINFORCED: u16 = 361;
pub const BOBBER_FIBERGLASS: u16 = 362;
pub const BOBBER_FISHEROF_SOULS: u16 = 363;
pub const BOBBER_GOLDEN: u16 = 364;
pub const BOBBER_MECHANICS: u16 = 365;
pub const BOBBER_SITTING_DUCK: u16 = 366;
pub const BOBBER_FLESHCATCHER: u16 = 381;
pub const BOBBER_HOTLINE: u16 = 382;
pub const BOBBER_BLOOD: u16 = 760;
pub const BOBBER_SCARAB: u16 = 775;
/// The glow-bobber overrides (`Player.UpdateFishingBobber`,
/// `Player.cs:36673-36702`: items 5139-5146 map to 986-993).
pub const BOBBER_GLOW_START: u16 = 986;
pub const BOBBER_GLOW_END: u16 = 993;
/// Every kind the aiStyle-61 dispatch claims (the `Projectile.bobber` flag's
/// SetDefaults writers — `Projectile.cs:4515`/`:9429`).
pub const BOBBER_KINDS: [u16; 19] = [
    BOBBER_WOODEN,
    BOBBER_REINFORCED,
    BOBBER_FIBERGLASS,
    BOBBER_FISHEROF_SOULS,
    BOBBER_GOLDEN,
    BOBBER_MECHANICS,
    BOBBER_SITTING_DUCK,
    BOBBER_FLESHCATCHER,
    BOBBER_HOTLINE,
    BOBBER_BLOOD,
    BOBBER_SCARAB,
    986,
    987,
    988,
    989,
    990,
    991,
    992,
    993,
];
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
/// `ProjectileID.Bat` (316) — the Bat Scepter's homing bat (aiStyle 36).
pub const BAT_FRIENDLY: u16 = 316;
/// `ProjectileID.PineNeedleFriendly` (336) — the Razorpine's needle
/// (aiStyle 1); it flies straight for 50 ticks then falls at 0.5/tick.
pub const PINE_NEEDLE_FRIENDLY: u16 = 336;
/// `ProjectileID.Blizzard` (337) — the Blizzard Staff's icicle (aiStyle 1);
/// it falls through tiles until it nears the owner or the surface.
pub const BLIZZARD: u16 = 337;
/// `ProjectileID.BlueDungeonDebris` (736, `ProjectileID.cs:2026`) — the
/// cracked blue dungeon brick's fall shard: aiStyle 10, 10x10
/// (`Projectile.cs:7748-7755`).
pub const BLUE_DUNGEON_DEBRIS: u16 = 736;
/// `ProjectileID.GreenDungeonDebris` (737, `ProjectileID.cs:2028`) — the
/// green twin of the 736.
pub const GREEN_DUNGEON_DEBRIS: u16 = 737;
/// `ProjectileID.PinkDungeonDebris` (738, `ProjectileID.cs:2030`) — the
/// pink twin of the 736.
pub const PINK_DUNGEON_DEBRIS: u16 = 738;
/// The dungeon-debris trio the `AI_010` arm and the kill hook gate on — the
/// C# spells the set as `type >= 736 && type <= 738` (`Projectile.cs:57927`,
/// `:57942`, `:58073`, `:77369`).
pub const DUNGEON_DEBRIS: [u16; 3] =
    [BLUE_DUNGEON_DEBRIS, GREEN_DUNGEON_DEBRIS, PINK_DUNGEON_DEBRIS];
/// `ProjectileID.SpiritFlame` (659) — the Spirit Flame's shot (aiStyle 129).
pub const SPIRIT_FLAME: u16 = 659;
/// `ProjectileID.SkyFracture` (660) — the Sky Fracture's aiStyle-1 rider.
pub const SKY_FRACTURE: u16 = 660;
/// `ProjectileID.BookOfSkullsSkull` (837) — the Book of Skulls' rider.
pub const BOOK_OF_SKULLS: u16 = 837;
/// The followup caster families of M4-PROJ-6b: the kind ids below are the
/// verbatim `ProjectileID` values, their `SetDefaults` arms cited per row in
/// [`proj_data`]. The aiStyles 27/40/51/59/71/118/119/75/84 they declare
/// reach the dispatch in Phases 2-4; the rows alone land here.
/// `ProjectileID.UnholyTridentFriendly` (114) — the aiStyle-27 family's own
/// shot (`extraUpdates = 2`, `timeLeft = 180`, `armorPenetration = 25`).
pub const UNHOLY_TRIDENT: u16 = 114;
/// `ProjectileID.UnholyTridentHostile` (115) — the hostile twin: the same
/// 16x16 box with `penetrate = -1` and no friendly flag.
pub const UNHOLY_TRIDENT_HOSTILE: u16 = 115;
/// `ProjectileID.SwordBeam` (116) — the Beam Sword's beam
/// (`Item.cs:9827-9836`'s `shoot = 116`, melee, pen 2).
pub const SWORD_BEAM: u16 = 116;
/// `ProjectileID.TerraBeam` (132) — an aiStyle-27 rider carrying the family's
/// local-immunity group (`localNPCHitCooldown = 10`).
pub const TERRA_BEAM: u16 = 132;
/// `ProjectileID.LightBeam` (156) — the second rider with the same local
/// immunity group (melee, pen 2).
pub const LIGHT_BEAM: u16 = 156;
/// `ProjectileID.NightBeam` (157) — the 32x32 rider (`scale = 1.2f`).
pub const NIGHT_BEAM: u16 = 157;
/// `ProjectileID.EnchantedBeam` (173) — the Enchanted Sword's beam
/// (`Item.cs:12756-12764`'s `shoot = 173`, melee, pen 1).
pub const ENCHANTED_BEAM: u16 = 173;
/// `ProjectileID.Leaf` (206) — the Leaf Blower's leaf (aiStyle 40).
pub const LEAF: u16 = 206;
/// `ProjectileID.LostSoulHostile` (293) — the Spectre Staff's hostile soul.
pub const LOST_SOUL_HOSTILE: u16 = 293;
/// `ProjectileID.LostSoulFriendly` (297) — the friendly soul: `penetrate = 3`
/// with the id-static group (`idStaticNPCHitCooldown = 10`).
pub const LOST_SOUL: u16 = 297;
/// `ProjectileID.SpectreWrath` (356) — the set-bonus's ghost shot (aiStyle
/// 59, `tileCollide = false`, `extraUpdates = 3`).
pub const SPECTRE_WRATH: u16 = 356;
/// `ProjectileID.Typhoon` (409) — the Razorblade Typhoon's vortex (aiStyle
/// 71, 30x30, `penetrate = -1`, `timeLeft = 300`).
pub const TYPHOON: u16 = 409;
/// `ProjectileID.NebulaArcanum` (617) — the Arcanum's homing bolt (aiStyle
/// 118, local-immunity 8).
pub const NEBULA_ARCANUM: u16 = 617;
/// `ProjectileID.NebulaArcanumSubshot` (618) — the Arcanum's arm (aiStyle
/// 119, 18x30, `timeLeft = 420`, local-immunity 20).
pub const NEBULA_ARCANUM_SUBSHOT: u16 = 618;
// S1-P-i22 — the magic guns + tomes' shots (`Player.ItemCheck_Shoot` census,
// `Player.cs:47056-49468`). Every id is the verbatim `ProjectileID` value;
// the `SetDefaults` arms are cited per row in [`proj_data`].
/// `ProjectileID.Starfury` (9, `ProjectileID.cs:572`) — the falling star the
/// Starfury sword (65) and the Star Cannon's fallback share (aiStyle 5).
pub const STARFURY: u16 = 9;
/// `ProjectileID.QuarterNote` (76, `ProjectileID.cs:706`) — the Magical
/// Harp's first note kind; the shoot rolls `76 + Next(3)` (`Player.cs:47394`).
pub const QUARTER_NOTE: u16 = 76;
/// `ProjectileID.EighthNote` (77, `ProjectileID.cs:708`) — the harp's second.
pub const EIGHTH_NOTE: u16 = 77;
/// `ProjectileID.TiedEighthNote` (78, `ProjectileID.cs:710`) — the harp's third.
pub const TIED_EIGHTH_NOTE: u16 = 78;
/// `ProjectileID.NettleBurstRight` (150, `ProjectileID.cs:854`) — the Nettle
/// Burst's head segment (aiStyle 4, the Vilethorn family's chain).
pub const NETTLE_BURST_RIGHT: u16 = 150;
/// `ProjectileID.NettleBurstLeft` (151, `ProjectileID.cs:856`).
pub const NETTLE_BURST_LEFT: u16 = 151;
/// `ProjectileID.NettleBurstEnd` (152, `ProjectileID.cs:858`) — the chain's tip.
pub const NETTLE_BURST_END: u16 = 152;
/// `ProjectileID.HeatRay` (260, `ProjectileID.cs:1074`) — the Heat Ray's beam
/// (aiStyle 48, `extraUpdates = 100`).
pub const HEAT_RAY: u16 = 260;
/// `ProjectileID.PoisonFang` (265, `ProjectileID.cs:1084`) — the Poison
/// Staff's fang (aiStyle 1, `timeLeft = 37`, local immunity 10).
pub const POISON_FANG: u16 = 265;
/// `ProjectileID.ElectrosphereMissile` (442, `ProjectileID.cs:1438`) — the
/// Electrosphere Launcher's rocket; `Kill()` spawns the sphere 443.
pub const ELECTROSPHERE_MISSILE: u16 = 442;
/// `ProjectileID.Electrosphere` (443, `ProjectileID.cs:1440`) — the sphere the
/// missile's death spawns (aiStyle 77).
pub const ELECTROSPHERE: u16 = 443;
/// `ProjectileID.ChargedBlasterOrb` (459, `ProjectileID.cs:1472`) — the
/// cannon's charge orb (aiStyle 1, `extraUpdates = 1`).
pub const CHARGED_BLASTER_ORB: u16 = 459;
/// `ProjectileID.ChargedBlasterCannon` (460, `ProjectileID.cs:1474`) — the
/// channel holder (aiStyle 75, 14x18, `penetrate = -1`, `tileCollide = false`).
pub const CHARGED_BLASTER_CANNON: u16 = 460;
/// `ProjectileID.ChargedBlasterLaser` (461, `ProjectileID.cs:1476`) — the
/// holder's full-charge beam (aiStyle 84, the key-linked 460 rider).
pub const CHARGED_BLASTER_BEAM: u16 = 461;
/// `ProjectileID.CrystalPulse` (521, `ProjectileID.cs:1596`) — the Crystal
/// Serpent's pulse (aiStyle 29's own 521 arm, `extraUpdates = 1`).
pub const CRYSTAL_PULSE: u16 = 521;
/// `ProjectileID.SparkleGuitar` (856, `ProjectileID.cs:2266`) — the Sparkle
/// Guitar's note (aiStyle 167, the CatmullRom arc).
pub const SPARKLE_GUITAR: u16 = 856;
/// `ProjectileID.StarCannonStar` (955, `ProjectileID.cs:2464`) — the Star
/// Cannon's shot (aiStyle 5, `penetrate = -1`, `alpha = 50`).
pub const STAR_CANNON_STAR: u16 = 955;
/// `ProjectileID.LastPrism` (633) — the channel holder (aiStyle 75,
/// 14x18, `penetrate = -1`, `tileCollide = false`).
pub const LAST_PRISM: u16 = 633;
/// `ProjectileID.LastPrismLaser` (632) — the holder's beam (aiStyle 84,
/// 18x18, `alpha = 255`, `penetrate = -1`).
pub const LAST_PRISM_LASER: u16 = 632;
/// `ProjectileID.NebulaBlaze1` (634) — the Nebula Blaze's first arm: an
/// aiStyle-1 rider with `extraUpdates = 2` (the plain-[`ai_arrow`] flight).
pub const NEBULA_BLAZE_1: u16 = 634;
/// `ProjectileID.NebulaBlaze2` (635) — the 20%-roll twin: aiStyle 1 with
/// `extraUpdates = 3` (the C# arm sets `friendly` twice — a verbatim quirk
/// with no effect on the port's `friendly` default).
pub const NEBULA_BLAZE_2: u16 = 635;
/// `ProjectileID.HorsemanPumpkin` (1001) — the Headless Horseman's
/// 480-tick lob (`NPC.cs:62417`; `Projectile.cs:9511-9522`, aiStyle 194).
pub const HORSEMAN_PUMPKIN: u16 = 1001;
/// The drill projectiles (S1-P-i29) — `DefaultToDrillOrChainsaw`
/// (`Projectile.cs:10732-10742`): aiStyle 20, `penetrate = -1`,
/// `tileCollide = false`, `melee = true`, `ownerHitCheck = true`. The
/// per-kind boxes/scales ride the `SetDefaults` type arms (`:1407-1434`,
/// `:1946-1952`, `:3011-3092`, `:4990-5036`). The aiStyle-20 machine
/// (`Projectile.cs:25481-25584`) holds the owner's item animation via
/// `SetDummyItemTime(2)` and kills itself on the channel release.
pub const DRILL_COBALT: u16 = 59;
pub const DRILL_MYTHRIL: u16 = 60;
pub const DRILL_ADAMANTITE: u16 = 62;
pub const DRILL_PALLADIUM: u16 = 213;
pub const DRILL_ORICHALCUM: u16 = 216;
pub const DRILL_TITANIUM: u16 = 219;
pub const DRILL_CHLOROPHYTE: u16 = 223;
pub const DRILL_VORTEX: u16 = 428;
pub const DRILL_NEBULA: u16 = 430;
pub const DRILL_SOLAR: u16 = 432;
pub const DRILL_STARDUST: u16 = 609;
pub const DRAX: u16 = 107;
/// The chainsaw projectiles (S1-P-i30) — the SAME
/// `DefaultToDrillOrChainsaw` family as the drills (`Projectile.cs:
/// 10732-10742`): aiStyle 20, `penetrate = -1`, `tileCollide = false`,
/// `melee = true`, `ownerHitCheck = true`. The per-kind boxes/scales ride
/// the `SetDefaults` type arms (`:1394-1427`, `:3018-3024`, `:3038-3044`,
/// `:3059-3065`, `:3093-3099`, `:3351-3357`, `:4530-4535`, `:4983-5036`,
/// `:5787-5794`). The aiStyle-20 machine (`Projectile.cs:25481-25584`)
/// holds the owner's item animation and kills itself on the channel
/// release — the chainsaw's chop cadence is the item's `useTime` through
/// the `toolTime` channel countdown (`Player.cs:43582-43586`).
pub const CHAINSAW_COBALT: u16 = 57;
pub const CHAINSAW_MYTHRIL: u16 = 58;
pub const CHAINSAW_ADAMANTITE: u16 = 61;
pub const CHAINSAW_PALLADIUM: u16 = 214;
pub const CHAINSAW_ORICHALCUM: u16 = 217;
pub const CHAINSAW_TITANIUM: u16 = 220;
pub const CHAINSAW_CHLOROPHYTE: u16 = 224;
pub const CHAINSAW_SAWTOOTH_SHARK: u16 = 369;
pub const CHAINSAW_VORTEX: u16 = 427;
pub const CHAINSAW_NEBULA: u16 = 429;
pub const CHAINSAW_SOLAR: u16 = 431;
pub const CHAINSAW_BUTCHERS: u16 = 509;
pub const CHAINSAW_STARDUST: u16 = 610;
/// `ProjectileID.LaserDrill` (445) — the Laser Drill's beam, aiStyle 75
/// (`Projectile.cs:5162-5172`, `AI_075` `:66244-66358`), NOT a
/// `DefaultToDrillOrChainsaw` row. The port holds its channel the same way
/// (the aiStyle-75 beam's full laser AI is a follow-up).
pub const LASER_DRILL: u16 = 445;
/// `ProjectileID.DirtGolfBall` (721) + the 14 dyed twins 739-752
/// (`ProjectileID.cs:428`'s `IsAGolfBall` set) — aiStyle 149, 7x7,
/// friendly, penetrate -1, tileCollide false, netImportant
/// (`Projectile.cs:7626-7635`). S1-P-i55.
pub const GOLF_BALL: u16 = 721;
/// `ProjectileID.GolfClubHelper` (722) — aiStyle 150, 2x2, tileCollide
/// false, ignoreWater, penetrate -1, hide (`Projectile.cs:7636-7645`).
/// S1-P-i55.
pub const GOLF_CLUB_HELPER: u16 = 722;

/// `Main.projectile[1001]` — 1000 usable slots (`Projectile.cs:66992`,
/// `GetNextSlot` returns 1000 when full, `:10793`).
pub const MAX_PROJECTILES: usize = 1000;
/// `timeLeft`'s SetDefaults value (`Projectile.cs:787`).
pub const TIME_LEFT_DEFAULT: i32 = 3600;

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
    /// `Projectile.immunityIdentity` (`:782`, reset to `type`) — the key the
    /// id-static immunity table is indexed by. `UseGroupStaticImmunity(Type)`
    /// (`:10676-10681`) overrides it to a shared group id: the Bee Gun's bee
    /// 181 and giant bee 566 both carry 181, so one cooldown covers both.
    /// `0` = "use the kind" (every row that does not call the group helper).
    pub immunity_identity: u16,
    /// `Projectile.alpha` (`:779`) — the draw-time fade, which the AI arms
    /// also read as a state marker (the magic families, Phase 3).
    pub alpha: i32,
    /// `Projectile.scale` (default `1f`, `:320`/`:785`).
    pub scale: f32,
    /// `Projectile.light` (default `0f`, `:378`/`:773`) — the `SetDefaults`
    /// amount the `ProjLight` chain multiplies (`Projectile.cs:16153-16661`,
    /// S1-E-5); the data-carrying spawn copies it onto the slot.
    pub light: f32,
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
    /// `Projectile.decidesManualFallThrough` (`:460`, reset false `:708`) —
    /// `HandleMovement`'s `flag6` rides `shouldFallThrough` instead of the
    /// per-type ladder (`:17583-17586`); the aiStyle-67 summon rows set it
    /// (`:4767`/`:7825`/`:8092`/`:8106`/`:8120`/`:8980`).
    pub decides_manual_fall_through: bool,
    /// `Projectile.melee` (`:434`, reset false `:751`) — the damage-class
    /// flag `Damage_PVE_Inner`'s per-hit crit block reads (`:12890`). The
    /// yoyo/flail/spear/drill/shortsword families set it via their
    /// `DefaultTo*` helpers; the whip does NOT (`DefaultToWhip` `:10704`).
    pub melee: bool,
    /// `Projectile.ranged` (`:436`, reset false `:752`) — the crit block's
    /// ranged arm (`:12894`).
    pub ranged: bool,
    /// `Projectile.magic` (`:438`, reset false `:753`) — the crit block's
    /// magic arm (`:12898`).
    pub magic: bool,
    /// `Projectile.appliesImmunityTimeOnSingleHits` (`:717`, reset false) —
    /// the id-static immunity gate's single-hit override (`:13969`/`:14116`).
    /// The ported setters: the 1134 crystal shard (`:10643`), the 167-170
    /// fireworks (`:2568`, S1-P-i16) and the 286 Exploding Bullet (`:3754`,
    /// S1-P-i18).
    pub applies_immunity_time_on_single_hits: bool,
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

pub(super) fn yoyo_lifetime(kind: u16) -> f32 {
    yoyo_table(YOYO_LIFETIME, -1.0, kind)
}

pub(super) fn yoyo_max_range(kind: u16) -> f32 {
    yoyo_table(YOYO_MAX_RANGE, 200.0, kind)
}

pub(super) fn yoyo_top_speed(kind: u16) -> f32 {
    yoyo_table(YOYO_TOP_SPEED, 10.0, kind)
}

/// The shared (non-per-kind) locals of `AI_015_Flails`
/// (`Projectile.cs:43746-43760`), with the meleeSpeed-scaled ones marked.
pub(super) const FLAIL_THROW_MAX_DISTANCE: f32 = 800.0; // num3
pub(super) const FLAIL_RETRACT_ACCEL: f32 = 3.0; // num4 (scaled)
pub(super) const FLAIL_REEL_ACCEL: f32 = 6.0; // num6 (scaled)
pub(super) const FLAIL_SPIN_SOFT_ACCEL: f32 = 1.0; // num8 (scaled)
pub(super) const FLAIL_SPIN_PULL_SPEED: f32 = 14.0; // num9 (scaled)
pub(super) const FLAIL_SPIN_INNER_RADIUS: f32 = 60.0; // num10
pub(super) const FLAIL_HELD_NPC_COOLDOWN: f32 = 10.0; // num11
pub(super) const FLAIL_FLIGHT_NPC_COOLDOWN: f32 = 10.0; // num13

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

/// The ported rows' `SetDefaults` `this.light` slice (S1-E-5): the amount
/// `ProjLight` (`Projectile.cs:16153-16661`) multiplies into the frame's
/// `AddLight` contribution (`proj_lights.rs`). Only the kinds with a
/// `proj_data` row appear — a kind without a row never spawns through the
/// data-carrying seam, so its light never reaches a slot. The unlisted
/// kinds keep the stock `0f` reset (`:773`) and `ProjLight` early-returns.
pub(crate) fn set_defaults_light(kind: u16) -> f32 {
    match kind {
        FLAME_ARROW => 1.0,                    // `:824`
        UNHOLY_ARROW => 0.35,                  // `:844` (S1-P-i17)
        JESTERS_ARROW => 0.4,                  // `:856` (S1-P-i17)
        HELLFIRE_ARROW => 0.3,                 // `:1246` (S1-P-i17)
        CURSED_ARROW => 1.0,                   // `:1911` (S1-P-i17)
        FROSTBURN_ARROW => 1.0,                // `:2586` (S1-P-i17)
        ICHOR_ARROW => 1.0,                    // `:3645` (S1-P-i17)
        ENCHANTED_BOOMERANG => 0.4,            // `:872`
        MUSKET_BALL => 0.5,                    // `:957`
        // S1-P-i42 — the glowstick family's `this.light = 1f`
        // (`Projectile.cs:1334`/`:1362`/`:5493`/`:5859`/`:8351`/`:10160`).
        // The flares (163/310/1008-1011) set NO light — their glow is the
        // dust trail's (`AI_033`'s `Dust.NewDust`, `:27093`), not ProjLight.
        GLOWSTICK | STICKY_GLOWSTICK | SPELUNKER_GLOWSTICK | BOUNCY_GLOWSTICK
        | FAIRY_GLOWSTICK | RAINBOW_GLOWSTICK => 1.0,
        BULLET_SNOWMAN => 0.5,                 // `:1985`
        BALL_OF_FIRE => 0.8,                   // `:970`
        MAGIC_MISSILE => 0.8,                  // `:980`
        GREEN_LASER => 0.75,                   // `:1025`
        FLAMELASH => 0.8,                      // `:1168`
        DEMON_SICKLE => 0.2,                   // `:1276`
        DEMON_SCYTHE => 0.2,                   // `:1288`
        DUAL_HOOK_BLUE | DUAL_HOOK_RED => 0.4, // `:1546`
        RAINBOW_ROD_BULLET => 0.8,             // `:1600`
        ICE_BLOCK => 0.5,                      // `:1615`
        EYE_LASER => 0.75,                     // `:1639`
        PURPLE_LASER => 0.75,                  // `:1709`
        CRYSTAL_STORM => 0.5,                  // `:1784`
        CURSED_FLAME_FRIENDLY => 0.8,          // `:1798`
        CURSED_FLAME => 0.8,                   // `:1809`
        DESTROYER_LASER => 0.75,               // `:1878`
        ICE_BOOMERANG => 0.4,                  // `:2020`
        LIGHT_DISC => 0.4,                     // `:1944` (S1-P-i08)
        UNHOLY_TRIDENT => 0.5,                 // `:2029`
        UNHOLY_TRIDENT_HOSTILE => 0.5,         // `:2044`
        SWORD_BEAM => 0.5,                     // `:2054`
        TERRA_BEAM => 0.5,                     // `:2216`
        LIGHT_BEAM => 0.5,                     // `:2441`
        NIGHT_BEAM => 0.5,                     // `:2453`
        ENCHANTED_BEAM => 0.2,                 // `:2598`
        PIRATE_ARROW => 0.5,                   // `:2667`
        FROST_HYDRA => 0.25,                   // `:4003`
        BALL_OF_FROST => 0.8,                  // `:3364`
        MAGNET_SPHERE_BALL => 0.5,             // `:3376`
        GOLEM_EYE_BEAM => 0.3,                 // `:3436`
        CHRISTMAS_HOOK => 0.5,                 // `:4256`
        MINI_RETINA_LASER => 0.75,             // `:4736`
        ANTI_GRAVITY_HOOK => 0.7,              // `:5186`
        ILLUMINANT_HOOK => 0.3,                // `:5631`
        CRYSTAL_SHARD_MELEE => 0.5,            // `:10644`
        // S1-P-i18 — the special bullets' `SetDefaults` light amounts.
        METEOR_SHOT => 0.6,                    // `:1189`
        CRYSTAL_BULLET | CRYSTAL_SHARD => 0.5, // `:1722` / `:1736`
        CURSED_BULLET => 0.5,                  // `:1922`
        CHLOROPHYTE_BULLET => 0.5,             // `:2955`
        HIGH_VELOCITY_BULLET => 0.5,           // `:3245`
        ICHOR_BULLET => 0.5,                   // `:3657`
        VENOM_BULLET => 0.5,                   // `:3706`
        PARTY_BULLET => 0.5,                   // `:3720`
        NANO_BULLET => 0.5,                    // `:3734`
        EXPLODING_BULLET => 0.5,               // `:3747`
        GOLDEN_BULLET => 0.5,                  // `:3765`
        // S1-P-i35 — the glow bobbers 987-993 (`Projectile.cs:9430-9434`):
        // `light = 1f` (the 986 plain override keeps the stock 0).
        987..=993 => 1.0,
        _ => 0.0,
    }
}

/// The ported families' `SetDefaults` slice. The spear trap's spear (186,
/// aiStyle 37) keeps a stand-in row (see its arm) and the flame jet (187,
/// aiStyle 38) stays a passthrough — their AI families are M4-PROJ
/// follow-ups. The magic (caster) families' (M4-PROJ-6) rows sit in
/// `ProjectileID` order at the end of the match: their `aiStyle` values
/// declare the dispatch arm Phases 3-4 add.
///
/// One live arm per kind: the pirate/martian/moon-lord merges appended
/// duplicate arms the compiler only warns about (`unreachable_patterns`),
/// so a stale 10x10 `MUSKET_BALL` sat behind the real 4x4 row and the
/// spiky ball 185 rode the dart group's aiStyle 1 (S1-D-5). A duplicate is
/// a hard error here — the `npc/data.rs` `to_u16`/`from_u16` precedent.
#[deny(unreachable_patterns)]
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
            // `Projectile.cs:1625-1631` — the hostile FlamingArrow (82):
            // 10x10, aiStyle 1, hostile; the stock penetrate 1 (`:774`),
            // tileCollide (`:775`), timeLeft 3600 (`:787`) — the type arm
            // writes NO override (unlike the friendly 2's 1200). The
            // `arrow` flag stays false (the 81/82 arms never set it —
            // `:809`/`:819` are the friendly 1/2 only). The ElfArcher's
            // shot (S1-D-3 W-6); the OnFire-on-hit roll
            // (`StatusPlayer :12284`, `Next(3)`) is a HIT-time draw on
            // the projectile's own tick — the player-debuff layer is
            // unported, documented in ANALYSIS.
            FLAMING_ARROW_HOSTILE => {
                (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // `Projectile.cs:828-836` — the shuriken (3): the 22x22 box, aiStyle 2,
            // `penetrate = 4`, the stock 3600 life and the stock `tileCollide`.
            SHURIKEN => (22.0, 22.0, 2, 4, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:1310-1316` — the throwing knife (48): 12x12, aiStyle
            // 2, `penetrate = 2`.
            THROWING_KNIFE => (12.0, 12.0, 2, 2, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // S1-P-i42 — the glowstick family (aiStyle 14, the thrown light).
            // `Projectile.cs:1326-1336` (50) / `:5851-5861` (515) /
            // `:8343-8353` (870): 6x6, `penetrate = -1`, `alpha = 75`,
            // `light = 1f`, `timeLeft *= 5` (3600 → 18000). The stock
            // `tileCollide = true` (`:775`) stands — the bounce lives in
            // `HandleMovement`'s per-type arms, not the AI.
            GLOWSTICK | BOUNCY_GLOWSTICK | FAIRY_GLOWSTICK => (
                6.0,
                6.0,
                14,
                -1,
                TIME_LEFT_DEFAULT * 5,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 75, 1.0),
            ),
            // `Projectile.cs:1354-1365` — the sticky glowstick (53): the same
            // row PLUS `tileCollide = false` (`:1364`) — the stick scan in the
            // AI arm (`:24911-24962`) replaces the collision bounce.
            STICKY_GLOWSTICK => (
                6.0,
                6.0,
                14,
                -1,
                TIME_LEFT_DEFAULT * 5,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 75, 1.0),
            ),
            // `Projectile.cs:5485-5495` — the spelunker glowstick (473): 8x8,
            // `timeLeft *= 2` (3600 → 7200), the same alpha/light.
            SPELUNKER_GLOWSTICK => (
                8.0,
                8.0,
                14,
                -1,
                TIME_LEFT_DEFAULT * 2,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 75, 1.0),
            ),
            // `Projectile.cs:10153-10162` — the rainbow glowstick (1089): 6x6,
            // `timeLeft *= 5`, `light = 1f`, NO alpha override (the stock 0,
            // `:779`) — the disco colour rides `ProjLight` (`:16434-16439`).
            RAINBOW_GLOWSTICK => (
                6.0,
                6.0,
                14,
                -1,
                TIME_LEFT_DEFAULT * 5,
                0,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // S1-P-i42 — the flare family (aiStyle 33, the flare gun's shot).
            // `Projectile.cs:2515-2525` (163) / `:4022-4032` (310) /
            // `:9580-9623` (1008-1011): 6x6, `friendly = true`,
            // `penetrate = -1`, `alpha = 255`, `timeLeft = 36000`. The stock
            // `tileCollide = true` stands; the stick lives in the AI arm +
            // `HandleMovement`'s aiStyle-33 block (`:18619-18634`).
            FLARE | BLUE_FLARE | SPELUNKER_FLARE | CURSED_FLARE | RAINBOW_FLARE
            | SHIMMER_FLARE => (
                6.0,
                6.0,
                33,
                -1,
                36000,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:2549-2557` — the snowball (166): 14x14, aiStyle 2.
            SNOWBALL => (14.0, 14.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // S1-P-i35 — the fishing bobbers (`Projectile.cs:4508-4516`,
            // `:9422-9435`): 14x14, aiStyle 61, `penetrate = -1`,
            // `netImportant = true`, `bobber = true`; the stock 3600 life
            // (`AI_061` refreshes `timeLeft = 60` every tick, `:53662`).
            // The glow overrides 987-993 add `light = 1f` (`:9433`) via
            // `set_defaults_light`; their glowMask 318+(type-987) is a draw
            // seam (no port field).
            k if BOBBER_KINDS.contains(&k) => {
                (14.0, 14.0, 61, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // `Projectile.cs:9002-9011` — the Wand of Sparking's spark (954):
            // 10x10, aiStyle 2, `penetrate = 2`, the spawn `alpha = 255`.
            SPARK => (
                10.0,
                10.0,
                2,
                2,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // S1-P-i28 — the thrown-consumable family's missing rows.
            // `Projectile.cs:1366-1374` — the poisoned knife (54): 12x12,
            // aiStyle 2, `penetrate = 2` (the throwing knife's box).
            POISONED_KNIFE => (12.0, 12.0, 2, 2, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:1488-1511` — the three water bottles (69/70/621):
            // 14x14, aiStyle 2, `penetrate = 1`. The splash conversion rides
            // the death hook (`:77629-77645`), not the row.
            HOLY_WATER | UNHOLY_WATER | BLOOD_WATER => {
                (14.0, 14.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // `Projectile.cs:1122-1130` — the grenade (30): 14x14, aiStyle 16,
            // `penetrate = -1`; `NewProjectile` pins `timeLeft = 180`
            // (`:10993-10996`).
            GRENADE => (14.0, 14.0, 16, -1, 180, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4781-4790` — the sticky grenade (397): the same
            // box/style, `tileCollide = false`; `timeLeft = 180`
            // (`:11029-11032`).
            STICKY_GRENADE => (14.0, 14.0, 16, -1, 180, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5870-5878` — the bouncy grenade (517): 14x14,
            // aiStyle 16, `penetrate = -1`; `timeLeft = 180` (`:10997-11000`).
            BOUNCY_GRENADE => (14.0, 14.0, 16, -1, 180, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4801-4811` — the molotov cocktail (399): 14x14,
            // aiStyle 68, `penetrate = 1`, `alpha = 255`.
            MOLOTOV_COCKTAIL => (
                14.0,
                14.0,
                68,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:4812-4835` — the three greek fires (400/401/402):
            // aiStyle 14, `penetrate = 3`, `timeLeft = 360`; the three sizes
            // 14x16 / 12x14 / 6x12.
            MOLOTOV_FIRE_1 => (14.0, 16.0, 14, 3, 360, 0, true, false, false, RESET_STATE),
            MOLOTOV_FIRE_2 => (12.0, 14.0, 14, 3, 360, 0, true, false, false, RESET_STATE),
            MOLOTOV_FIRE_3 => (6.0, 12.0, 14, 3, 360, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2697-2706` — the beenade (183): 14x22, aiStyle 14,
            // `penetrate = 1`, `timeLeft = 180`.
            BEENADE => (14.0, 22.0, 14, 1, 180, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2673-2685` — the bee (181): 8x8, aiStyle 36,
            // `penetrate = 3`, `alpha = 255`, `timeLeft = 600`,
            // `extraUpdates = 3`, `UseGroupStaticImmunity(181)` (idStatic-10).
            BEE => (8.0, 8.0, 36, 3, 600, 3, true, false, false, (false, -2, true, 10, 255, 1.0)),
            // `Projectile.cs:6172-6184` — the giant bee (566): 16x16, aiStyle
            // 36, `penetrate = 4`, `alpha = 255`, `timeLeft = 660`,
            // `extraUpdates = 3`, the same group-static immunity (identity 181).
            GIANT_BEE => (16.0, 16.0, 36, 4, 660, 3, true, false, false, (false, -2, true, 10, 255, 1.0)),
            // `Projectile.cs:6470-6480` — the bone javelin (598): 16x16,
            // aiStyle 113, `penetrate = -1`, `alpha = 255`.
            BONE_JAVELIN => (
                16.0,
                16.0,
                113,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:5770-5778` — the Javelin (507): 16x16, aiStyle 1,
            // `penetrate = 3`, friendly+ranged; the stock `timeLeft = 3600`
            // (`:787`) and `tileCollide` (`:775`). The flight is the `AI_001`
            // 507/508/662/680/685 gravity arm (`:57471-57478`, the 60-tick then
            // X·0.98/Y+0.3 fall), owned by `projectile_ai_thrown::covers_ai001`.
            JAVELIN => (16.0, 16.0, 1, 3, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:874-884` — the Vilethorn's base (7) and tip (8):
            // 28x28, aiStyle 4, `penetrate = -1`, `tileCollide = false`,
            // `alpha = 255` and `ignoreWater`.
            VILE_BASE | VILE_TIP => (
                28.0,
                28.0,
                4,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:1871-1882` — the laser: 4x4, the aiStyle-1
            // flight, `penetrate = 3`, `extraUpdates = 2`, `timeLeft = 2700`
            // (the AI_037 arm overrides to 300 post-spawn).
            DESTROYER_LASER => (4.0, 4.0, 1, 3, 2700, 2, true, false, false, RESET_STATE),
            // `Projectile.cs:1966-1977` — the SnowBallHostile (109): 10x10,
            // aiStyle 10, hostile, `penetrate = -1`, `scale = 0.9f`, the
            // stock 3600 life (the Balla's throw overrides it to 300,
            // `NPC.cs:29088`). `knockBack = 6f` is overridden by the throw's
            // `KnockBack: 0f` (`NPC.cs:29086`); `coldDamage`/`ranged` have no
            // port field (the damage-class flags, ANALYSIS S1-D-3q).
            SNOW_BALL_HOSTILE => (
                10.0,
                10.0,
                10,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 0, 0.9),
            ),
            // `Projectile.cs:1978-1991` — the BulletSnowman (110): 4x4,
            // aiStyle 1, hostile, `penetrate = -1`, `alpha = 255`,
            // `scale = 1.2f`, `timeLeft = 600`, `extraUpdates = 1` (the
            // Gangsta's throw overrides the life to 300, `NPC.cs:29026`).
            // `light = 0.5f` (`:1985`) rides S1-E-5's `set_defaults_light`
            // row for this kind (the data-carrying spawn copies it).
            BULLET_SNOWMAN => (
                4.0,
                4.0,
                1,
                -1,
                600,
                1,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.2),
            ),
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
            // `Projectile.cs:1646-1659` — the Probe's DeathLaser: the same
            // 4x4 aiStyle-1 flight, penetrate 3, extraUpdates 2, timeLeft
            // 600, hostile (the light/alpha/scale/magic rows are draw-only).
            PROBE_LASER => (4.0, 4.0, 1, 3, 600, 2, true, false, false, RESET_STATE),
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
            // `Projectile.cs:2752-2763` — the FlamesTrap (188): 6x6, aiStyle
            // 23, friendly + hostile, alpha 255, penetrate -1, extraUpdates 2,
            // trap.
            FLAMES_TRAP => (
                6.0,
                6.0,
                23,
                -1,
                TIME_LEFT_DEFAULT,
                2,
                true,
                false,
                true,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:5064-5071` — the Scrambler's bolt (436): 10x10,
            // aiStyle 1, hostile, ignoreWater. (`SCRAMBLER_SHOT` is the
            // same-id alias — S1-D-5 deleted its duplicate arm.) The
            // ignoreWater flag has no row slot yet (the resolver derives it
            // for summons/Vilethorn only); recorded in ANALYSIS "The
            // projectile data-row deviations (S1-D-5)".
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
            // `Projectile.cs:5081-5090` — the Ray Gunner's laser (438): 8x8,
            // aiStyle 1, hostile, `alpha = 255`, extraUpdates 3, ignoreWater.
            // S1-D-5 carries the C# alpha: the slice-A fade (`:55660-55671`)
            // fades IN from 255 — the reset 0 made the laser visible from
            // tick 1. The generic draw path does not read alpha yet
            // (ANALYSIS "The projectile data-row deviations (S1-D-5)").
            // `MARTIAN_RAY` is the same-id alias; its duplicate arm was
            // deleted.
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
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:5188-5197` — the deathray: 30x30, aiStyle 79,
            // penetrate -1, tileCollide false, ignoreWater, timeLeft 240
            // (the arm is `ai_saucer_deathray`, S1-D-4b-4).
            SAUCER_DEATHRAY => (30.0, 30.0, 79, -1, 240, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5199-5206` — the missile: 14x14, aiStyle 80,
            // penetrate -1, tileCollide false (the arm is
            // `ai_saucer_missile`, S1-D-4b-4).
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
            // `Projectile.cs:3558-3566` — the skull: 26x26, aiStyle 1,
            // `penetrate = 3`, `alpha = 255`, hostile (the Skeletron head's
            // expert spin shot).
            SKULL => (
                26.0,
                26.0,
                1,
                3,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:3614-3629` — Plantera's seeds: 14x14, aiStyle
            // 1, hostile, `alpha = 255`; the stock penetrate 1 / tileCollide
            // / timeLeft 3600 (`:774-787`) — the AI's call site overrides
            // the seeds' timeLeft to 300 (`NPC.cs:31966-31969`).
            SEED_PLANTERA | POISON_SEED_PLANTERA => (
                14.0,
                14.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:3630-3637` — the thorn ball: 38x38, aiStyle
            // 14, hostile, `alpha = 255`; the stock life (no override).
            THORN_BALL => (
                38.0,
                38.0,
                14,
                1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:3249-3259` — the flaming wood: 14x14, aiStyle 1,
            // hostile, `penetrate = -1`, no tile collision, `timeLeft = 420`.
            FLAMING_WOOD => (
                14.0,
                14.0,
                1,
                -1,
                420,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:3260-3277` — the Greek fires: the aiStyle-14
            // flight, hostile, `penetrate = -1`, `timeLeft = 360` (the three
            // sizes: 14x14, 12x14, 6x12).
            GREEK_FIRE_1 => (
                14.0,
                14.0,
                14,
                -1,
                360,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 0, 1.0),
            ),
            GREEK_FIRE_2 => (
                12.0,
                14.0,
                14,
                -1,
                360,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 0, 1.0),
            ),
            GREEK_FIRE_3 => (
                6.0,
                12.0,
                14,
                -1,
                360,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:9199-9206` — the ice chunk: 16x16, aiStyle 1,
            // `penetrate = -1`, `extraUpdates = 2`, `timeLeft = 220`,
            // `alpha = 255`, water-ignoring (the Deerclops' rubble).
            DEERCLOPS_ICE_CHUNK => (
                16.0,
                16.0,
                1,
                -1,
                220,
                2,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
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
                // `Projectile.cs:9083` — the spike's `alpha = 255`.
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:9307-9317` — the shadow hand: 40x40, aiStyle
            // 187, penetrate -1, tileCollide false, timeLeft 300.
            // `InsanityShadowHostile` (`Projectile.cs:9116-9126`): 40x40,
            // aiStyle 187, `penetrate = -1`, `timeLeft = 300`, `alpha = 255`,
            // no tile collision, water-ignoring.
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
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:1896-1903` — the bomb: 22x22, aiStyle 16,
            // penetrate -1, hostile.
            BOMB_SKELETRON => (22.0, 22.0, 16, -1, 180, 0, true, false, false, RESET_STATE),
            // `:950-961`: 4×4, `extraUpdates = 1`, `timeLeft = 600`.
            MUSKET_BALL => (4.0, 4.0, 1, 1, 600, 1, true, false, false, RESET_STATE),
            // S1-P-i14 — the bullet-launch projectiles the gun arms fire
            // (the 242 HighVelocity row is S1-P-i18's `HIGH_VELOCITY_BULLET`
            // below — identical values, the V5 gate kept one arm).
            // `Projectile.cs:5150-5161` — the Xenopopper bubble 444 (the
            // Xenopopper 2797's arm fires it, `Player.cs:47621`): 10×10,
            // aiStyle 78, `alpha = 255`, `extraUpdates = 1`, `ignoreWater`;
            // the stock `penetrate = 1` / `tileCollide = true` / life 3600.
            XENOPOPPER_BUBBLE => (
                10.0, 10.0, 78, 1, TIME_LEFT_DEFAULT, 1, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7108-7121` — the Onyx Blaster's black bolt 661
            // (`Player.cs:48837`): 10×10, aiStyle 1, `alpha = 255`,
            // `timeLeft = 40`, `extraUpdates = 1`, `ignoreWater`, local
            // immunity −1; the stock `penetrate = 1` / `tileCollide = true`.
            ONYX_BLACK_BOLT => (
                10.0, 10.0, 1, 1, 40, 1, true, false, false,
                (true, -1, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:9160-9168` — the Pew-matic Horn's shot 968
            // (`Player.cs:47644`): 14×14, aiStyle 2, `alpha = 255`; the stock
            // `penetrate = 1` / `tileCollide = true` / life 3600. The 24-frame
            // sheet's `frame = (int)ai[1]` pin reads `proj_frames(968)`.
            PEW_MATIC_SHOT => (
                14.0, 14.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // S1-P-i18 — the special bullets' `SetDefaults` rows. Every one is
            // a 4×4 aiStyle-1 friendly ranged shot with `alpha = 255` and
            // `timeLeft = 600`; the per-kind slice is the penetrate /
            // extraUpdates / scale / immunity pair, cited per row.
            // `Projectile.cs:1182-1197` — MeteorShot 36: penetrate 2,
            // extraUpdates 1, scale 1.4, the LOCAL-immunity 7 pair (the
            // one-bounce shot, `HandleMovement` `:19278-19298`).
            METEOR_SHOT => (4.0, 4.0, 1, 2, 600, 1, true, false, false, (true, 7, false, -1, 255, 1.4)),
            // `Projectile.cs:1715-1728` — CrystalBullet 89: penetrate 1,
            // extraUpdates 1, scale 1.2 (the death split spawns the 90 shards).
            CRYSTAL_BULLET => (4.0, 4.0, 1, 1, 600, 1, true, false, false, (false, -2, false, -1, 255, 1.2)),
            // `Projectile.cs:1729-1742` — CrystalShard 90: 6×6, aiStyle **24**,
            // penetrate 1, alpha 50, scale 1.2, `tileCollide = false` (the
            // existing `ai_crystal_storm` arm flies it).
            CRYSTAL_SHARD => (6.0, 6.0, 24, 1, 600, 0, false, false, false, (false, -2, false, -1, 50, 1.2)),
            // `Projectile.cs:1915-1928` — CursedBullet 104: penetrate 1,
            // extraUpdates 2, scale 1.2 (CursedInferno on hit).
            CURSED_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.2)),
            // `Projectile.cs:2948-2961` — ChlorophyteBullet 207: penetrate 1,
            // extraUpdates 2, scale 1.2 (the homing arm).
            CHLOROPHYTE_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.2)),
            // `Projectile.cs:3238-3254` — BulletHighVelocity 242: penetrate 3,
            // extraUpdates 7, scale 1.18, `ignoreWater`, the LOCAL-immunity −1
            // pair (the speed-fade pierce shot).
            HIGH_VELOCITY_BULLET => (4.0, 4.0, 1, 3, 600, 7, true, false, false, (true, -1, false, -1, 255, 1.18)),
            // `Projectile.cs:3650-3663` — IchorBullet 279: penetrate 1,
            // extraUpdates 2, scale 1.25 (Ichor on hit).
            ICHOR_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.25)),
            // `Projectile.cs:3699-3712` — VenomBullet 283: penetrate 1,
            // extraUpdates 2, scale 1.25 (Venom + PotentAcid on hit, the
            // debuff-spread death burst).
            VENOM_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.25)),
            // `Projectile.cs:3713-3726` — PartyBullet 284: penetrate 1,
            // extraUpdates 2, scale 1.3 (the confetti death burst).
            PARTY_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.3)),
            // `Projectile.cs:3727-3740` — NanoBullet 285: penetrate 1,
            // extraUpdates 2, scale 1.3 (Confused on hit, the bounce +
            // re-target, the `PickAmmo` ×1.25).
            NANO_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.3)),
            // `Projectile.cs:3741-3757` — ExplosiveBullet 286: penetrate 1,
            // extraUpdates 2, scale 1.3, the ID-STATIC-immunity 60 pair +
            // `appliesImmunityTimeOnSingleHits` (the AoE death burst). The
            // override flag has no `ProjData` field; the burst arms it through
            // the `max_penetrate` stand-in (see `exploding_bullet_burst`).
            EXPLODING_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, true, 60, 255, 1.3)),
            // `Projectile.cs:3758-3771` — GoldenBullet 287: penetrate 1,
            // extraUpdates 2, scale 1.3 (Midas on hit).
            GOLDEN_BULLET => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.3)),
            // `:1823-1832` / `:2707-2716` / `:9330-9339`: the trap darts —
            // `penetrate = -1`, `trap = true`, aiStyle 1. (The spiky ball 185
            // left this group in S1-D-5/S1-D-4c: its own arm below carries
            // the C# aiStyle 14.)
            TRAP_DART | TRAP_SUPERDART | TRAP_VENOM_DART => (
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
            // `Projectile.cs:2717-2727` — the Spiky Ball Trap's ball (185,
            // `ProjectileID.SpikyBallTrap`): 14x14, **aiStyle 14** (the
            // shared S1-D-5/S1-D-4c fix — the rowed 1 sent the ball down the
            // plain-arrow flight), `penetrate = -1`, `timeLeft = 900`, trap.
            // The C# sets friendly AND hostile; the row shape carries
            // neither flag (ANALYSIS "The projectile data-row deviations").
            // The C# AI_014 has NO 185-type branch (`:24830-25307`) — 185
            // rides the generic default arm (`:25142-25160`); its bounce
            // lives in `HandleMovement`'s collision block (`:19703-19713`:
            // the changed axis flips to `lastVelocity * -0.9f`, the Y only
            // when `lastVelocity.Y > 1f`) — unported; the port's `ai_spear`
            // kills on the tile scan instead (same ANALYSIS section).
            TRAP_SPIKY_BALL => (14.0, 14.0, 14, -1, 900, 0, true, false, true, RESET_STATE),
            // `:6953-6965`: the Lunar Flare (645 — the `POISON_DART`
            // constant's name deviation is documented at its definition,
            // S1-D-4 doc-fix; the arm is ai001 slice C's) —
            // `tileCollide = false`, `extraUpdates = 5`, the local immunity
            // cooldown −1.
            // `Projectile.cs:2937-2945` — the Leaf Blower petal: 14x14,
            // aiStyle 40, penetrate 1, `alpha = 255` (the AI fades it in) and
            // the 360-tick lifetime.
            LEAF_BLOWER => (
                14.0,
                14.0,
                40,
                1,
                360,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
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
            // `:2728-2740` — the Spear Trap's spear (186) is really 10x14,
            // aiStyle **37**, tileCollide false, ignoreWater, friendly AND
            // hostile, penetrate -1, timeLeft 300, trap. aiStyle 37
            // (`AI()` `:27472`) has no ported dispatch arm, so the row keeps
            // the 185 stand-in values (the `:2717-2727` spiky-ball arm the
            // trap family shared before S1-D-5 split 185 out) — correcting
            // the row to 37 would park the spear in `_ => {}` forever. The
            // deviation is recorded in ANALYSIS "The projectile data-row
            // deviations (S1-D-5)" with the aiStyle-37 port as the follow-up.
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
            // `:1936-1945`: the Light Disc (S1-P-i08) — 32×32, aiStyle 3,
            // `penetrate = -1`, melee; the stock 3600 life / tileCollide.
            LIGHT_DISC => (
                32.0,
                32.0,
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
            // `FlaironBubble` (`Projectile.cs:4857-4868`): 14x14, aiStyle 70,
            // `penetrate = 1`, `alpha = 255`, `timeLeft = 120`.
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
                (false, -2, false, -1, 255, 1.0),
            ),            // The bombs (`:1106-1121`, `:1198-1205`, `:5862-5896`); the
            // fuse seeds ride `NewProjectileSetup` (`:10970-10993`): 180
            // ticks for the bombs, 300 for the dynamite. The sticky set
            // {37, 519} disables the plain tile collide — their hover scan
            // stops them.
            BOMB | BOUNCY_BOMB => (22.0, 22.0, 16, -1, 180, 0, true, false, false, RESET_STATE),
            STICKY_BOMB => (22.0, 22.0, 16, -1, 180, 0, false, false, false, RESET_STATE),
            BOMB_FISH => (24.0, 24.0, 16, -1, 180, 0, false, false, false, RESET_STATE),
            DYNAMITE => (10.0, 10.0, 16, -1, 300, 0, true, false, false, RESET_STATE),
            // S1-D-5: the `SCRAMBLER_SHOT` arm that sat here is deleted —
            // `SCRAMBLER_SHOT` = 436 = `BRAIN_SCRAMBLER` (the same-id alias,
            // `ProjectileID.BrainScramblerBolt`), so the arm above already
            // resolved it and this one was unreachable. The alias const
            // stays for its callers; one row per id.
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
            // `Projectile.cs:5329-5339` — the Phantasmal Bolt (462): 8x8,
            // aiStyle 1, hostile, `alpha = 255`, extraUpdates 3, tileCollide
            // false, ignoreWater. S1-D-5 carries the C# alpha (the slice-B
            // fade `:55660-55671` fades IN from 255). 462/622 are DISTINCT
            // C# ids (`ProjectileID.cs:1478`/`:1798`) — separate rows, not
            // aliases; the duplicate arms the moon-lord merge appended were
            // unreachable copies, deleted.
            PHANTASMAL_BOLT => (8.0, 8.0, 1, 1, 3600, 3, false, false, true, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:6691-6698` — the death-drama smoke (622): 10x10,
            // aiStyle 1, `alpha = 255`, extraUpdates 2. S1-D-5 carries the
            // C# alpha — the smoke is INVISIBLE for its whole 69-tick life
            // (`AI_001` `:56165-56183` never fades it; the visible effect is
            // the dust 229, a documented skip).
            BLOWUP_SMOKE_MOONLORD => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // Deadeye's arrow 180 (`Projectile.cs:2660-2672`): 4x4,
            // aiStyle 1, hostile, penetrate -1, light 0.5, alpha 255,
            // scale 1.2, timeLeft 180, extraUpdates 1. The ElfCopter's
            // Ice-Spike shot rides this id too (S1-D-3 W-6, `NPC.cs:33967`
            // Type 180) — the row is the C# `SetDefaults` arm verbatim
            // (GATE: the 180 verbatim row WINS main's stale placeholder).
            // `arrow` stays false (the 180 arm never sets it). The Corsair's
            // shot is kind 82 FlamingArrow (`Projectile.cs:1625-1631`),
            // rowed separately (S1-D-5 census).
            PIRATE_ARROW => (
                4.0,
                4.0,
                1,
                -1,
                180,
                1,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.2),
            ),
            // ---- The Frost Moon bosses' hostile shots (S1-F-13c wiring).
            // `Projectile.cs:4353-4360` — the Everscream's pine needle (345):
            // 4x4, aiStyle 1, hostile, `scale = 0.8f`; the stock penetrate 1
            // (`:774`), tileCollide true (`:775`), timeLeft 3600 (`:787`).
            PINE_NEEDLE_HOSTILE => (4.0, 4.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 0, 0.8)),
            // `Projectile.cs:4361-4369` — the ornament (346): 18x18,
            // aiStyle 14, hostile, `penetrate = -1`, `timeLeft = 300`.
            ORNAMENT_HOSTILE => (18.0, 18.0, 14, -1, 300, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4378-4389` — the FrostWave (348): 48x48,
            // aiStyle 1, hostile, `penetrate = -1`, `tileCollide = false`,
            // `extraUpdates = 1`, `timeLeft = 900`.
            FROST_WAVE => (48.0, 48.0, 1, -1, 900, 1, false, false, false, RESET_STATE),
            // `Projectile.cs:4390-4399` — the FrostShard (349): 12x12,
            // aiStyle 1, hostile, `penetrate = -1`, `timeLeft = 900`.
            FROST_SHARD => (12.0, 12.0, 1, -1, 900, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4400-4410` — the Missile (350): 14x14, aiStyle 1,
            // hostile, `penetrate = -1`, `tileCollide = false`, `alpha = 255`,
            // `timeLeft = 300`.
            MISSILE => (14.0, 14.0, 1, -1, 300, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:4411-4420` — the Present (351): 24x24, aiStyle 58,
            // hostile, `penetrate = -1`, `tileCollide = false`, `alpha = 255`.
            PRESENT => (24.0, 24.0, 58, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:4421-4429` — the Spike (352): 30x30, aiStyle 14,
            // hostile, `penetrate = -1`, `timeLeft = 1200`.
            SPIKE => (30.0, 30.0, 14, -1, 1200, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:3219-3227` — the Flying Dutchman's shell (240):
            // 16x16, aiStyle 2, `penetrate = -1`, `alpha = 255` (the arm's
            // `ai[0] > 2f` gate turns it invisible on tick 3). The C#'s
            // `hostile = true` rides the hostile spawn sites, not the row.
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
                (false, -2, false, -1, 255, 1.0),
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
            // `DD2DarkMageRaise` / `DD2DarkMageHeal` (`Projectile.cs:7208-7219`):
            // 10x10, aiStyle 133, `alpha = 255`, `timeLeft = 900`,
            // `penetrate = -1`, no tile collision, water-ignoring.
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
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7221-7230` — the Dark Mage's flame (675): the
            // SetDefaults arm writes `alpha = 255` (the slice-C fade-20
            // `:56468` fades IN from 255 — the reset 0 lost the fade).
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
                (false, -2, false, -1, 255, 1.0),
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
            // `:1040-1056` — the Aqua Scepter's stream: 18x18, aiStyle 12,
            // alpha 255, `penetrate = 5`, `extraUpdates = 2`, `ignoreWater`.
            // The remix-world local-immunity arm (`:1051-1055`) is unported
            // (no remix seed flag — ANALYSIS); the stock immunity reset holds.
            AQUA_SCEPTER => (18.0, 18.0, 12, 5, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:1095-1105` — the water bolt: 16x16, aiStyle 8, alpha 255,
            // `penetrate = 10` and `timeLeft /= 2` off the 3600 reset
            // (`:787`) — 1800.
            WATER_BOLT => (16.0, 16.0, 8, 10, 1800, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:1162-1173` — the flamelash: 32x32, aiStyle 9, `penetrate =
            // 2`, the local-NPC immunity with the 12-tick cooldown.
            FLAMELASH => (32.0, 32.0, 9, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 12, false, -1, 0, 1.0)),
            // `:1271-1282` — the demon sickle (the Demon/VoodooDemon's
            // hostile shot, `NPC.cs:23449`): 48x48, aiStyle 18, alpha 100,
            // `penetrate = -1`, `scale = 0.9f`; the spawn site overrides
            // `timeLeft = 300` (`NPC.cs:23450`).
            DEMON_SICKLE => (48.0, 48.0, 18, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 0.9)),
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
            // S1-P-i21 — the hardmode staves' missing rows (the six kinds the
            // census found row-less: without a `proj_data` row the
            // data-carrying spawn refuses the shot entirely).
            // `:3185-3192` — the Nimbus Rod's flying cloud: 28x28, aiStyle 45,
            // `penetrate = -1`, `netImportant` (the tail's kind list).
            RAIN_CLOUD_MOVING => (28.0, 28.0, 45, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `:3193-3203` — the settled raining cloud: 54x28, aiStyle 45,
            // `tileCollide = false`, `ignoreWater`, `penetrate = -1`,
            // `timeLeft = 18000`.
            RAIN_CLOUD_RAINING => (54.0, 28.0, 45, -1, 18000, 0, false, false, false, RESET_STATE),
            // `:3204-3217` — the rain drop: 4x40, aiStyle 45, `penetrate = 5`,
            // `timeLeft = 300`, `scale = 1.1f`, `extraUpdates = 1`, the
            // id-static immunity with the 10-tick cooldown.
            RAIN_FRIENDLY => (4.0, 40.0, 45, 5, 300, 1, true, false, false, (false, -2, true, 10, 0, 1.1)),
            // `:3664-3674` — the Golden Shower's stream: 32x32, aiStyle 12,
            // alpha 255, `penetrate = 5`, `extraUpdates = 2`, `ignoreWater`.
            GOLDEN_SHOWER => (32.0, 32.0, 12, 5, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:3838-3848` — the Shadowbeam Staff's beam: 4x4, aiStyle 48,
            // `extraUpdates = 100`, `timeLeft = 300`, `penetrate = -1`.
            SHADOWBEAM => (4.0, 4.0, 48, -1, 300, 100, true, false, false, RESET_STATE),
            // `:4088-4097` — the Bat Scepter's bat: 16x16, aiStyle 36,
            // alpha 255, `timeLeft = 600` (the stock `penetrate = 1`).
            BAT_FRIENDLY => (16.0, 16.0, 36, 1, 600, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:4287-4296` — the Razorpine's needle: 4x4, aiStyle 1,
            // `scale = 0.8f`, `extraUpdates = 1`.
            PINE_NEEDLE_FRIENDLY => (4.0, 4.0, 1, 1, TIME_LEFT_DEFAULT, 1, true, false, false, (false, -2, false, -1, 0, 0.8)),
            // `:4297-4307` — the Blizzard Staff's icicle: 10x10, aiStyle 1,
            // `tileCollide = false`, `extraUpdates = 1` (coldDamage — no slot
            // field yet).
            BLIZZARD => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 1, false, false, false, RESET_STATE),
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
            // The followup caster families (M4-PROJ-6b, Phase 1): the verbatim
            // `SetDefaults` rows in `ProjectileID` order, the aiStyles the
            // Phases 2-4 arms dispatch on. The C#'s `light`, `magic`/`melee`,
            // `hostile`, `ignoreWater`, `drawLayer` and `armorPenetration`
            // fields have no `ProjData` consumer — dropped, as everywhere else.
            // aiStyle 27 (`:2023-2035` UnholyTridentFriendly): 16x16, magic,
            // light 0.5, alpha 255, `extraUpdates = 2`, `timeLeft = 180`
            // (`armorPenetration = 25` dropped). The arm itself counts `ai[0]`
            // up from the zeroed scratch array (`:26426-26440`), so the row
            // sets no ai convention.
            UNHOLY_TRIDENT => (16.0, 16.0, 27, 1, 180, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:2036-2046` UnholyTridentHostile — the same 16x16 box with
            // `penetrate = -1` and no `timeLeft`/`extraUpdates` override (the
            // 3600 reset holds; `hostile` has no field in this port) — alpha
            // 255, as the friendly twin's row.
            UNHOLY_TRIDENT_HOSTILE => (16.0, 16.0, 27, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:2047-2056` SwordBeam — 16x16, melee, `penetrate = 2`,
            // light 0.5, alpha 255.
            SWORD_BEAM => (16.0, 16.0, 27, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:2209-2222` TerraBeam — 16x16, melee, `penetrate = 3`, alpha
            // 255, the family's local-immunity group (cooldown 10).
            TERRA_BEAM => (16.0, 16.0, 27, 3, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 10, false, -1, 255, 1.0)),
            // `:2434-2446` LightBeam — 16x16, melee, `penetrate = 2`, alpha
            // 255, local-immunity 10.
            LIGHT_BEAM => (16.0, 16.0, 27, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 10, false, -1, 255, 1.0)),
            // `:2447-2456` NightBeam — the 32x32 rider with `scale = 1.2f`
            // and light 0.5, alpha 255; no local-immunity pair (the port's
            // overlay leaves the row's own group alone).
            NIGHT_BEAM => (32.0, 32.0, 27, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.2)),
            // `:2591-2600` EnchantedBeam — 16x16, melee, `penetrate = 1`,
            // light 0.2, alpha 255.
            ENCHANTED_BEAM => (16.0, 16.0, 27, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // S1-D-5: the `LEAF` arm that sat here is deleted — `LEAF` =
            // 206 = `LEAF_BLOWER` (the same-id alias), and its tuple was
            // value-identical to the live `:2937-2945` row above (only the
            // formatting differed). Unreachable behind that winner; one row
            // per id. The alias const stays for its callers.
            // `:3826-3837` LostSoulHostile — 12x12, aiStyle 51, alpha 255,
            // `tileCollide = false`, `penetrate = -1`, `extraUpdates = 1`
            // (the `293` arm's hunt lands in Phase 3).
            LOST_SOUL_HOSTILE => (12.0, 12.0, 51, -1, TIME_LEFT_DEFAULT, 1, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:3870-3882` LostSoulFriendly — 12x12, aiStyle 51, alpha 255,
            // `penetrate = 3`, `extraUpdates = 1` and the id-static group
            // with the 10-tick cooldown.
            LOST_SOUL => (12.0, 12.0, 51, 3, TIME_LEFT_DEFAULT, 1, true, false, false, (false, -2, true, 10, 255, 1.0)),
            // `:4464-4473` SpectreWrath — 6x6, aiStyle 59, alpha 255,
            // `tileCollide = false`, `extraUpdates = 3`; the row keeps the
            // stock `penetrate = 1` and the 3600 reset (the ghostHurt spawn
            // is Phase 3's documented skip).
            SPECTRE_WRATH => (6.0, 6.0, 59, 1, TIME_LEFT_DEFAULT, 3, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:4903-4915` Typhoon — 30x30, aiStyle 71, `penetrate = -1`,
            // alpha 255, `timeLeft = 300`, `extraUpdates = 2`, `ignoreWater`
            // dropped.
            TYPHOON => (30.0, 30.0, 71, -1, 300, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:6644-6657` NebulaArcanum — 32x32, aiStyle 118, alpha 255,
            // `penetrate = 3`, local-immunity with the 8-tick cooldown
            // (`ignoreWater`/`drawLayer` dropped).
            NEBULA_ARCANUM => (32.0, 32.0, 118, 3, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 8, false, -1, 255, 1.0)),
            // `:6658-6670` NebulaArcanumSubshot — 18x30, aiStyle 119,
            // `tileCollide = false`, `penetrate = -1`, `timeLeft = 420`,
            // local-immunity 20; alpha at the 0 reset.
            NEBULA_ARCANUM_SUBSHOT => (18.0, 30.0, 119, -1, 420, 0, false, false, false, (true, 20, false, -1, 0, 1.0)),
            // `:6793-6804` LastPrism — 14x18, aiStyle 75, `penetrate = -1`,
            // `tileCollide = false` (`drawLayer = 7`/`ignoreWater` dropped);
            // the channel keep-alive and the 6x laser spawn are Phase 4's.
            LAST_PRISM => (14.0, 18.0, 75, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `:6805-6815` LastPrismLaser — 18x18, aiStyle 84, `penetrate =
            // -1`, alpha 255, `tileCollide = false`. Phase 4 splits the
            // style-84 dispatch by kind so 455 keeps its own arm.
            LAST_PRISM_LASER => (18.0, 18.0, 84, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:6816-6826` NebulaBlaze1 — 40x40, aiStyle 1 (the plain
            // [`ai_arrow`] flight), alpha 255, `extraUpdates = 2`
            // (`ignoreWater` dropped).
            NEBULA_BLAZE_1 => (40.0, 40.0, 1, 1, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:6827-6836` NebulaBlaze2 — the same 40x40 box with
            // `extraUpdates = 3`; the C# sets `friendly` twice (kept verbatim
            // in the row's doc, no effect on the port).
            NEBULA_BLAZE_2 => (40.0, 40.0, 1, 1, TIME_LEFT_DEFAULT, 3, true, false, false, (false, -2, false, -1, 255, 1.0)),
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
            CASCADE_EXPLOSION => (
                32.0,
                32.0,
                189,
                4,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                (true, -1, false, -1, 255, 1.0),
            ),
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
            // The VortexHunter's dart (`Projectile.cs:6306-6315`): 10x10, aiStyle
            // 1, hostile, penetrate -1 (alpha 255 = no fade).
            VORTEX_DART => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // S1-D-5: the Moon Lord / tower arms that sat here
            // (STARDUST_MARK, NEBULA_EYE, TOWER_DAMAGE_BOLT,
            // VORTEX_LIGHTNING | VORTEX_PORTAL, VORTEX_RITUAL_BOLT,
            // PHANTASMAL_EYE/SPHERE/DEATHRAY, MOON_LEECH, PHANTASMAL_BOLT,
            // BLOWUP_SMOKE_MOONLORD) were byte-identical duplicates of the
            // live arms above — every one unreachable behind its earlier
            // winner. Deleted; one row per id.
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
            // pool default for every one of them; the class flags ride the
            // census below the tuple (W3-i11 round 2 gave the player-fired
            // rows their `ranged` write back). The launcher rockets
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
            // plus the Snowman Cannon's 803-810. 779/783 are the CLUSTER
            // shrapnel the cluster rockets' Kill arm spawns
            // (`:78063-78086`, S1-P-i16) — the same row arm covers them
            // (the C# range block `:7939` includes both).
            // The cluster shrapnel 779/783 carry the `NewProjectileSetup`
            // fuse seed `timeLeft = 60` (`Projectile.cs:11053-11060`) — the
            // row folds it like the bomb rows fold their 180/300 seeds.
            779 | 783 => (14.0, 14.0, 16, -1, 60, 0, true, false, false, RESET_STATE),
            776..=801 | 803..=810 => (
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
            // The Snowman Cannon's cluster shrapnel 862/863
            // (`Projectile.cs:8275-8283`): 14x14, aiStyle 16, penetrate -1,
            // the `NewProjectileSetup` 60-tick seed (`:11061-11064`).
            862 | 863 => (14.0, 14.0, 16, -1, 60, 0, true, false, false, RESET_STATE),
            // S1-P-i16 — the dart guns' ammo projectiles. The Poison Dart
            // 267 (`Projectile.cs:3529-3537`): 14x14, aiStyle 1, alpha 255,
            // the stock penetrate 1 / timeLeft 3600.
            267 => (14.0, 14.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Crystal Dart 477 (`:5518-5528`): 14x14, aiStyle 1,
            // alpha 255, `penetrate = 5`, `extraUpdates = 1`.
            477 => (14.0, 14.0, 1, 5, TIME_LEFT_DEFAULT, 1, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Cursed Dart 478 (`:5529-5539`): 14x14, aiStyle 1,
            // alpha 255, `penetrate = 3`, `timeLeft = 300`.
            478 => (14.0, 14.0, 1, 3, 300, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Ichor Dart 479 (`:5540-5548`): 14x14, aiStyle 1,
            // alpha 255, the stock penetrate 1 (the split machine rewrites
            // it per tick, `:57307-57316`).
            479 => (14.0, 14.0, 1, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Cursed Dart's flame drop 480 (`:5549-5560`): 12x12,
            // aiStyle 14, `penetrate = 1`, `timeLeft = 180`, alpha 255
            // (`noEnchantments` has no port field).
            480 => (12.0, 12.0, 14, 1, 180, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Electrosphere Launcher's bolt 442 (`:5126-5136`): 14x14,
            // aiStyle 1, alpha 255, `timeLeft = 600`, the stock penetrate 1.
            442 => (14.0, 14.0, 1, 1, 600, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Electrosphere sphere 443 (`:5137-5149`): 80x80, aiStyle 77,
            // alpha 255, `penetrate = -1`, `tileCollide = false`,
            // `ignoreWater` (the overlay below), the `NewProjectileSetup`
            // 300-tick seed (`:11065-11068`).
            443 => (80.0, 80.0, 77, -1, 300, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // The Fireworks Launcher's rockets 167-170 (`:2558-2569`):
            // 14x14, aiStyle 34, `timeLeft = 45`, the idStatic-1 pair with
            // `appliesImmunityTimeOnSingleHits` (the plain PVE walk applies
            // the idStatic stamp regardless of penetrate — the flag's effect
            // for these pen-1 rows).
            167..=170 => (14.0, 14.0, 34, 1, 45, 0, true, false, false, (false, -2, true, 1, 0, 1.0)),
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
            // S1-P-i17 — the special-arrow rows (the ammo items' own `shoot`
            // ids, `itemdata.rs`). Every row is the C# `SetDefaults` arm
            // verbatim; the stock `timeLeft = 3600` (`:787`) stands wherever
            // the arm writes none.
            // `Projectile.cs:837-848` — the Unholy Arrow (4): 10x10, aiStyle
            // 1, `penetrate = 5`, `light = 0.35f` (the light slice below),
            // `timeLeft = 1200`.
            UNHOLY_ARROW => (10.0, 10.0, 1, 5, 1200, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:849-863` — the Jester's Arrow (5): `penetrate =
            // -1` (infinite piercing), `alpha = 100`, `ignoreWater` (the
            // overlay below), `extraUpdates = 1`, `timeLeft = 120` — the
            // SHORT life is the arrow's whole gimmick; `light = 0.4f`.
            JESTERS_ARROW => (
                10.0, 10.0, 1, -1, 120, 1, true, true, false,
                (false, -2, false, -1, 100, 1.0),
            ),
            // `Projectile.cs:1237-1247` — the Hellfire Arrow (41):
            // `penetrate = -1`, stock 3600 life, `light = 0.3f`. The
            // on-hit `timeLeft = 1` clamp (`:13252-13255`) and the Kill()
            // 64x64 blast (`:76308-76341`) ride `damage_pve` and the pool's
            // death hook (`special_arrows.rs`).
            HELLFIRE_ARROW => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:1743-1752` — the Holy Arrow (91): stock
            // penetrate 1, `timeLeft = 1200`. Its Kill() spawns the falling
            // star 92 (`:75212-75248`).
            HOLY_ARROW => (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:1753-1764` — the falling star (92): 24x24,
            // aiStyle 5, penetrate 1, `alpha = 50`, `scale = 0.8f`,
            // `tileCollide = false` (the AI_005 arm turns it on below the
            // `ai[1]` floor), stock 3600 life.
            FALLING_STAR => (
                24.0, 24.0, 5, 1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (false, -2, false, -1, 50, 0.8),
            ),
            // `Projectile.cs:1904-1914` — the Cursed Arrow (103): stock
            // penetrate 1, `light = 1f`, `timeLeft = 1200`. The
            // CursedInferno infliction is `StatusNPC` `:11665-11668`.
            CURSED_ARROW => (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:2093-2104` — the Frost Arrow (120, the Ice
            // Bow's shot): `coldDamage` (no port field), `extraUpdates = 1`,
            // `timeLeft = 1200`.
            FROST_ARROW => (10.0, 10.0, 1, 1, 1200, 1, true, true, false, RESET_STATE),
            // `Projectile.cs:2579-2590` — the Frostburn Arrow (172):
            // `light = 1f`, `coldDamage`, `timeLeft = 1200`. The 1/3
            // Frostburn roll is `StatusNPC` `:11556-11562`.
            FROSTBURN_ARROW => (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE),
            // `Projectile.cs:3100-3110` — the Chlorophyte Arrow (225):
            // `extraUpdates = 1`, `timeLeft = 1200`. The bounce/retarget
            // machine is `HandleMovement` `:18539-18561`.
            CHLOROPHYTE_ARROW => (10.0, 10.0, 1, 1, 1200, 1, true, true, false, RESET_STATE),
            // `Projectile.cs:3638-3649` — the Ichor Arrow (278): `light =
            // 1f`, `extraUpdates = 1`, `timeLeft = 1200`. Ichor 69 for 600
            // ticks is `StatusNPC` `:11669-11672`.
            ICHOR_ARROW => (10.0, 10.0, 1, 1, 1200, 1, true, true, false, RESET_STATE),
            // `Projectile.cs:3688-3698` — the Venom Arrow (282):
            // `extraUpdates = 1`, `timeLeft = 1200`. Venom 70/600 +
            // PotentAcid 395/120 is `StatusNPC` `:11673-11677`.
            VENOM_ARROW => (10.0, 10.0, 1, 1, 1200, 1, true, true, false, RESET_STATE),
            // `Projectile.cs:5474-5484` — the Bone Arrow From Merchant
            // (474): `timeLeft = 1200`, `penetrate = 2`.
            BONE_ARROW_MERCHANT => (10.0, 10.0, 1, 2, 1200, 0, true, true, false, RESET_STATE),
            // The Bee 181 / Giant Bee 566 rows ride the S1-P-i28 block above
            // (keep-one — identical values, the idStatic-10 group pair).
            // `Projectile.cs:9570-9579` — the Shimmer Arrow (1006): stock
            // penetrate 1, `timeLeft = 1200`; the INVERTED gravity is
            // `flag4` (`:56034-56036` + `:57571-57574`).
            SHIMMER_ARROW => (10.0, 10.0, 1, 1, 1200, 0, true, true, false, RESET_STATE),
            // The arrow overrides — 117 the Bone Arrow (`:2058-2068`,
            // `extraUpdates = 2`), 469 the Bee Arrow (`:5435-5445`,
            // S1-P-i17 carries the C# `alpha = 255` — the AI_001 fade
            // `:55197-55208` reads it), 485 the Hellwing (`:5604-5615` —
            // 24x24, `penetrate = -1`), 495 the Shadowflame (`:5681-5692` —
            // `penetrate = 3`) and 819 the Blood Rain arrow (`:7993-8002` —
            // `extraUpdates = 1`; S1-P-i17 carries the C# `alpha = 255` and
            // the stock `timeLeft = 3600` — the arm writes NO override, the
            // old 1200 was a row deviation; `hide` has no port field):
            // 10x10 aiStyle-1 arrows (485's 24x24 box aside).
            117 => (10.0, 10.0, 1, 1, 1200, 2, true, true, false, RESET_STATE),
            469 => (
                10.0, 10.0, 1, 1, 1200, 0, true, true, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            485 => (24.0, 24.0, 1, -1, 1200, 0, true, true, false, RESET_STATE),
            495 => (10.0, 10.0, 1, 3, 1200, 0, true, true, false, RESET_STATE),
            // 819 the VenomArrow's fang shot (`Projectile.cs:7993-8002`):
            // alpha 255, extraUpdates 1, the STOCK timeLeft 3600 (the block
            // sets none — S1-P-i17's fix over the pre-i13 1200 row).
            819 => (
                10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 1, true, true, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // S1-P-i13 — the bow-launch arrow family. The post-`PickAmmo`
            // `projToShoot` overrides (`Player.cs:47157-47176`) and the arrow
            // ammo's own `Item.shoot` resolve to these kinds; without a row the
            // data-carrying spawn declines them. Every row is the `SetDefaults`
            // body verbatim (the aiStyle-1 flight is the generic `ai_arrow`
            // arm; the per-kind gravity/rotation hooks are the S1-D-5 slices).
            // 357 the PulseBolt (`Projectile.cs:4474-4486`): the Pulse Bow
            // 2223's override (`Player.cs:47173-47176`), 4x4, penetrate 6,
            // alpha 255, extraUpdates 2, scale 1.2, timeLeft 600. NOT `arrow`
            // (the C# block sets no `this.arrow`).
            357 => (4.0, 4.0, 1, 6, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.2)),
            // 710 the DD2BetsyArrow (`Projectile.cs:7521-7535`): the Betsy Bow
            // 3859's `Item.shoot`, extraUpdates 1, `timeLeft = extraUpdates *
            // 60 * 5 = 300`, the LOCAL immunity pair with cooldown -1, alpha
            // 255, `ignoreWater` (the overlay below).
            710 => (10.0, 10.0, 1, 1, 300, 1, true, true, false, (true, -1, false, -1, 255, 1.0)),
            // 323 the Stake (`Projectile.cs:4160-4171`): the Stake Launcher
            // 1835's `Item.shoot` (useAmmo Stake), 14x14, penetrate 10,
            // extraUpdates 3, alpha 255, scale 0.8. NOT `arrow` (the C# block
            // sets no `this.arrow`); the stock timeLeft 3600.
            323 => (14.0, 14.0, 1, 10, TIME_LEFT_DEFAULT, 3, true, false, false, (false, -2, false, -1, 255, 0.8)),
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
            HORNET => (24.0, 26.0, 62, -1, 18000, 0, false, false, false, RESET_STATE),
            FLYING_IMP => (34.0, 26.0, 62, -1, 18000, 0, false, false, false, RESET_STATE),
            TEMPEST => (28.0, 40.0, 62, -1, 18000, 0, false, false, false, RESET_STATE),
            UFO_MINION => (28.0, 28.0, 62, -1, 18000, 0, true, false, false, RESET_STATE),
            STARDUST_CELL => (24.0, 24.0, 62, -1, 18000, 0, true, false, false, RESET_STATE),
            ABIGAIL => (30.0, 48.0, 62, -1, 18000, 0, false, false, false, RESET_STATE),
            // The 0.5-slot spider minions (`Projectile.cs:4707` Retanimini,
            // `:4723` Spazmamini) — aiStyle 66, the slot budget halves.
            RETANIMINI | SPAZMAMINI => (40.0, 20.0, 66, -1, 18000, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:4730-4741` (389 MiniRetinaLaser) — the Retanimini's
            // bolt: 4x4, aiStyle 1, `light = 0.75f` (no port consumer, the
            // module doc), `alpha = 255`, `extraUpdates = 2`, `scale = 1.2f`,
            // `timeLeft = 600`; `tileCollide`/`penetrate` stay stock.
            MINI_RETINA_LASER => (4.0, 4.0, 1, 1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.2)),
            // `Projectile.cs:5997-6012` (533 DeadlySphere) — the third
            // aiStyle-66 arm: 20x20, `penetrate = -1`, `timeLeft *= 5`,
            // a full slot, `tileCollide = false`, `ignoreWater`, and the
            // LOCAL immunity pair (12) instead of the twins' idStatic one.
            DEADLY_SPHERE => (20.0, 20.0, 66, -1, 18000, 0, false, false, false, (true, 12, false, -1, 0, 1.0)),
            // The Stardust Guardian (`Projectile.cs:6709`) — aiStyle 120, the
            // 0-slot pet-grade minion.
            STARDUST_GUARDIAN => (50.0, 80.0, 120, -1, 18000, 0, false, false, false, RESET_STATE),
            // The pirate minion trio (`Projectile.cs:4755-4768`) — aiStyle 67,
            // 20x30, `penetrate = -1`, `timeLeft *= 5`, a full slot, the LOCAL
            // immunity pair (18) and the manual fall-through decision; neither
            // tileCollide nor ignoreWater is set (the stock true / false).
            ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN => {
                (20.0, 30.0, 67, -1, 18000, 0, true, false, false, RESET_STATE)
            }
            // The Vampire Frog (`Projectile.cs:7814-7828`) — aiStyle 67,
            // 20x30, `friendly = true` (the arm re-pins it per tick, `:49587`),
            // the idStatic-10 pair and the manual fall-through decision.
            VAMPIRE_FROG => (20.0, 30.0, 67, -1, 18000, 0, true, false, false, RESET_STATE),
            // The aiStyle-67 pets, slice B (S1-D-4b-12): the Crimson Heart
            // (`Projectile.cs:5413-5424`) — 20x20, `scale = 0.8f`,
            // `ignoreWater`, the stock tileCollide true and no immunity pair;
            // the Dirtiest Block (`:9637-9647`) — 16x16; the Boulder Pet
            // (`:9987-9997`) — 32x32. All three: `penetrate = -1`,
            // `netImportant`, `timeLeft *= 5`, `friendly = true` (the arm
            // re-pins it false per tick, `:49562`), no minion slice.
            CRIMSON_HEART => (20.0, 20.0, 67, -1, 18000, 0, true, false, false, (false, -2, false, -1, 0, 0.8)),
            DIRTIEST_BLOCK => (16.0, 16.0, 67, -1, 18000, 0, true, false, false, RESET_STATE),
            BOULDER_PET => (32.0, 32.0, 67, -1, 18000, 0, true, false, false, RESET_STATE),

            // The Storm Tiger family (`Projectile.cs:8091`/`:8105`/`:8119`)
            // — aiStyle 67; these three arms set neither tileCollide nor
            // ignoreWater (the stock true / false).
            STORM_TIGER_1 => (26.0, 20.0, 67, -1, 18000, 0, true, false, false, RESET_STATE),
            STORM_TIGER_2 | STORM_TIGER_3 => (20.0, 30.0, 67, -1, 18000, 0, true, false, false, RESET_STATE),
            // The baby bird (`Projectile.cs:7829-7845` — the C#
            // `ProjectileID.BabyBird`) — the buff-216 pet-respawn minion
            // (Phase 2's arm), aiStyle 158; the `friendly`/`drawLayer` lines
            // have no port consumer (the module doc).
            BABY_BIRD => (10.0, 10.0, 158, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:3516-3528` (266) — the Slime Staff's minion,
            // aiStyle 26 (the ground hop machine, AI_026 slice C —
            // `projectile_ai026_c.rs`), 24x16, `penetrate = -1`,
            // `timeLeft *= 5` and the idStatic-12 pair (`:3526-3527`,
            // the state group below — NOT the local pair).
            SLIME_MINION => (24.0, 16.0, 26, -1, 18000, 0, true, false, false, (false, -2, true, 12, 0, 1.0)),
            // ===== S1-P-i25: the summon-staff minions' missing rows =====
            // The slot-counting / sacrifice machinery needs every summon
            // staff's minion kind to carry its `SetDefaults` slice; these
            // rows were absent, so the staves' spawns declined and the
            // budget never booked them. The AI machines stay the documented
            // deferrals (the aiStyle-26/54/156/164/169/206 dispatch no-ops).
            // `Projectile.cs:2791-2813` (191-194, the Pygmy Staff's four):
            // 18x18, aiStyle 26, penetrate -1, `timeLeft *= 5`, a full slot;
            // the 192/193/194 scale ladder (1.025/1.05/1.075).
            191 => (18.0, 18.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            192 => (18.0, 18.0, 26, -1, 18000, 0, true, false, false, (false, -2, false, -1, 0, 1.025)),
            193 => (18.0, 18.0, 26, -1, 18000, 0, true, false, false, (false, -2, false, -1, 0, 1.05)),
            194 => (18.0, 18.0, 26, -1, 18000, 0, true, false, false, (false, -2, false, -1, 0, 1.075)),
            // `Projectile.cs:4099-4111` (317, the Raven Staff): 28x28,
            // aiStyle 54, penetrate 1, `timeLeft *= 5`, a full slot, the
            // LOCAL pair (10).
            317 => (28.0, 28.0, 54, 1, 18000, 0, true, false, false, (true, 10, false, -1, 0, 1.0)),
            // `Projectile.cs:4742-4754` (390-392, the Spider Staff's trio):
            // 30x30, aiStyle 26, penetrate -1, `timeLeft *= 5`, a full slot,
            // the idStatic pair (15).
            390 | 391 | 392 => (30.0, 30.0, 26, -1, 18000, 0, true, false, false, (false, -2, true, 15, 0, 1.0)),
            // `Projectile.cs:7777-7792` (755, the Sanguine Staff's bat):
            // 10x10, aiStyle 156, penetrate -1, `timeLeft *= 5`, a full slot,
            // tileCollide false, ignoreWater, the LOCAL pair (-1).
            755 => (10.0, 10.0, 156, -1, 18000, 0, false, false, false, (true, -1, false, -1, 0, 1.0)),
            // `Projectile.cs:8067-8081` (831, the Storm Tiger Staff's gem):
            // 10x10, aiStyle 164, penetrate -1, `timeLeft = 60` (NOT *5), a
            // full slot, tileCollide false, ignoreWater, hide.
            831 => (10.0, 10.0, 164, -1, 60, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:8284-8300` (864, the Smolstar): 10x10, aiStyle
            // 169, penetrate -1, `timeLeft = 60`, a full slot, tileCollide
            // false, ignoreWater, the LOCAL pair (10), armorPenetration 25
            // (no port field — dropped).
            864 => (10.0, 10.0, 169, -1, 60, 0, false, false, false, (true, 10, false, -1, 0, 1.0)),
            // `Projectile.cs:8922-8940` (946, the Empress Blade): 10x10,
            // aiStyle 156, penetrate -1, `timeLeft *= 5`, a full slot,
            // tileCollide false, ignoreWater, the LOCAL pair (-1), scale
            // 0.75, manualDirectionChange/drawLayer (render-side, dropped).
            946 => (10.0, 10.0, 156, -1, 18000, 0, false, false, false, (true, -1, false, -1, 0, 0.75)),
            // `Projectile.cs:8969-8983` (951, the Flinx Staff): 26x26,
            // aiStyle 67, penetrate -1, `timeLeft *= 5`, a full slot, the
            // idStatic pair (10), decidesManualFallThrough.
            951 => (26.0, 26.0, 67, -1, 18000, 0, true, false, false, (false, -2, true, 10, 0, 1.0)),
            // `Projectile.cs:9182-9196` (970, Abigail's counter): 10x10,
            // aiStyle 164, penetrate -1, `timeLeft = 60`, a full slot,
            // tileCollide false, ignoreWater, hide.
            970 => (10.0, 10.0, 164, -1, 60, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:9667-9680` (1022, the Dead Cells Mushroom Boi):
            // 20x30, aiStyle 67, penetrate -1, `timeLeft *= 5`, a full slot,
            // the LOCAL pair (18), decidesManualFallThrough.
            1022 => (20.0, 30.0, 67, -1, 18000, 0, true, false, false, (true, 18, false, -1, 0, 1.0)),
            // `Projectile.cs:10195-10209` (1093, the Palworld Cattiva) /
            // `:10362-10376` (1112, the Trusty Cattiva): 14x22, aiStyle 67,
            // penetrate -1, `timeLeft *= 5`, a full slot, the LOCAL pair (9),
            // decidesManualFallThrough, alpha 255.
            1093 | 1112 => (14.0, 22.0, 67, -1, 18000, 0, true, false, false, (true, 9, false, -1, 255, 1.0)),
            // `Projectile.cs:10210-10221` (1094, the Palworld Foxsparks) /
            // `:10377-10388` (1113, the Trusty Foxsparks): 18x18, aiStyle 26,
            // penetrate -1, `timeLeft *= 5`, a full slot, alpha 255.
            1094 | 1113 => (18.0, 18.0, 26, -1, 18000, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:10450-10463` (1118, the Clay Pot Minion): 14x22,
            // aiStyle 67, penetrate -1, `timeLeft *= 5`, a full slot, the
            // LOCAL pair (30), decidesManualFallThrough.
            1118 => (14.0, 22.0, 67, -1, 18000, 0, true, false, false, (true, 30, false, -1, 0, 1.0)),
            // `Projectile.cs:10464-10478` (1119, the Forbidden Minion):
            // 14x22, aiStyle 206, penetrate -1, `timeLeft *= 5`, a full slot,
            // the LOCAL pair (30), tileCollide false, ignoreWater.
            1119 => (14.0, 22.0, 206, -1, 18000, 0, false, false, false, (true, 30, false, -1, 0, 1.0)),
            // `Projectile.cs:6729-6753` (625-628, the Stardust Dragon chain):
            // 24x24, aiStyle 121, penetrate -1, `timeLeft *= 5`, tileCollide
            // false, ignoreWater, alpha 255, the LOCAL pair (7); the 626/627
            // BODY segments carry 0.5 slots, the 625 head / 628 tail 0 (the
            // slot block excludes them, `:16709`). The chain AI is the 121
            // arm (`dragon_step`, `AI_121_StardustDragon` `:42824-43080`).
            STARDUST_DRAGON_HEAD | STARDUST_DRAGON_TAIL => (24.0, 24.0, 121, -1, 18000, 0, false, false, false, (true, 7, false, -1, 255, 1.0)),
            STARDUST_DRAGON_BODY_1 | STARDUST_DRAGON_BODY_2 => (24.0, 24.0, 121, -1, 18000, 0, false, false, false, (true, 7, false, -1, 255, 1.0)),
            // AI_026 slice B (S1-D-4b-11) — the two spinning boss-head
            // pets. `Projectile.cs:8486-8495` (the 884-886 range block;
            // 885 SkeletronPet) and `:8521-8530` (the 889-892 range
            // block; 889 SkeletronPrimePet): 20x20, aiStyle 26,
            // friendly, `penetrate = -1`, `timeLeft *= 5` (3600 →
            // 18000), the stock `tileCollide = true` reset (`:745`) —
            // the arm sets it false on its first tick (`:60096`), so
            // `HandleMovement` never collides them. `netImportant`
            // (`:8488`/`:8523`) rides the overlay below.
            SKELETRON_PET | SKELETRON_PRIME_PET => {
                (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE)
            }

            // AI_026 slice A (S1-D-4b-10) — the 14 early-return flyer pets.
            // Every row: aiStyle 26, friendly, `penetrate = -1`,
            // `timeLeft *= 5` (3600 → 18000), the stock `tileCollide = true`
            // reset (`Projectile.cs:745`) — each flight arm sets it false on
            // its first tick (`:59123` etc.), so `HandleMovement` never
            // collides them. `netImportant` (all but the 175 — its block has
            // no such line, `:2612-2620`) and `ignoreWater` (199/211 only)
            // ride the overlays below.
            // `Projectile.cs:2612-2620` (175 BabyEater): 34x34.
            BABY_EATER => (34.0, 34.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2831-2840` (197 BabySkeletronHead): 42x42.
            BABY_SKELETRON_HEAD => (42.0, 42.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2841-2850` (198 BabyHornet): 26x26.
            BABY_HORNET => (26.0, 26.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2851-2861` (199 TikiSpirit): 28x28, ignoreWater.
            TIKI_SPIRIT => (28.0, 28.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2992-3003` (211 Wisp): 24x24, `light = 1f`
            // (no port consumer), ignoreWater.
            WISP => (24.0, 24.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4645-4654` (380 ZephyrFish): 26x26.
            ZEPHYR_FISH => (26.0, 26.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:7911-7920` (774 SharkPup): 26x26.
            SHARK_PUP => (26.0, 26.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:7963-7972` (815 LilHarpy): 26x26.
            LIL_HARPY => (26.0, 26.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:7983-7992` (817 GlitteryButterfly): 26x26.
            GLITTERY_BUTTERFLY => (26.0, 26.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:8486-8495` (the 884-886 range block; 886
            // QueenBeePet): 20x20.
            QUEEN_BEE_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:8521-8530` (the 889-892 range block; 892
            // DukeFishronPet): 20x20.
            DUKE_FISHRON_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:8543-8552` (894 MoonLordPet): 20x20.
            MOON_LORD_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:8589-8598` (the 899-901 range block; 901
            // DD2BetsyPet): 20x20.
            DD2_BETSY_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9908-9917` (1046 Pufferfish): 26x26.
            PUFFERFISH => (26.0, 26.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
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
            FROST_HYDRA => (80.0, 74.0, 53, 1, 36000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4009` — the Frost Blast fired by the Frost Hydra:
            // 14x14, aiStyle 28, `penetrate = 3`, `extraUpdates = 3`.
            FROST_BLAST_FRIENDLY => (14.0, 14.0, 28, 3, 3600, 3, true, false, false, RESET_STATE),
            QUEEN_SPIDER => (66.0, 50.0, 53, 1, 36000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:4625` - the Queen Spider's egg: 16x16, aiStyle 14,
            // `penetrate = -1`, `timeLeft = 60`.
            SPIDER_EGG => (16.0, 16.0, 14, -1, 60, 0, true, false, false, RESET_STATE),
            HOUNDIUS_SHOOTIUS => (18.0, 60.0, 53, 1, 36000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9153` — the Houndius' fireball: 14x14, aiStyle 28.
            HOUNDIUS_SHOOTIUS_FIREBALL => (14.0, 14.0, 28, 1, 3600, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:6916-6938` — the turret pair's `alpha = 255`
            // fade-in rides the state group (the 641's `drawLayer = 1` is a
            // draw-side field with no pool consumer); `ignoreWater` rides
            // the sentry flag's tail below.
            MOONLORD_TURRET | RAINBOW_CRYSTAL => {
                (32.0, 32.0, 123, 1, 36000, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // `Projectile.cs:6904-6915` — the turret's beam (642): 18x18,
            // aiStyle 84 (the `AI_084` laser family), `penetrate = -1`,
            // `tileCollide = false`, `alpha = 255`, local-immunity 10.
            MOONLORD_TURRET_LASER => {
                (18.0, 18.0, 84, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (true, 10, false, -1, 255, 1.0))
            }
            // `Projectile.cs:6858-6872` — the S.D.M.G.'s bullet (638,
            // S1-D-4): 4x4, aiStyle 1, `alpha = 255`, `extraUpdates = 5`,
            // `timeLeft = 600`, `penetrate = -1`, the local-immunity −1
            // pair; `ignoreWater` rides the tail below.
            MOONLORD_BULLET => {
                (4.0, 4.0, 1, -1, 600, 5, true, false, false, (true, -1, false, -1, 255, 1.0))
            }
            // `Projectile.cs:6873-6888` — the Phantasm's arrow (639):
            // `arrow = true`, 10x10, aiStyle 1, `MaxUpdates = 2` (the
            // setter `:560-563` → extraUpdates 1), `timeLeft = MaxUpdates
            // * 45` = 90, `penetrate = 4`, `alpha = 255`, the local −1 pair.
            MOONLORD_ARROW => {
                (10.0, 10.0, 1, 4, 90, 1, true, true, false, (true, -1, false, -1, 255, 1.0))
            }
            // `Projectile.cs:6889-6903` — the arrow's ghost trail (640):
            // 10x10, aiStyle 1, `MaxUpdates = 3` → extraUpdates 2,
            // `timeLeft = 90`, `penetrate = 4`, `alpha = 255`, the local −1
            // pair; NOT an arrow (no `this.arrow` line in the C# block).
            MOONLORD_ARROW_TRAIL => {
                (10.0, 10.0, 1, 4, 90, 2, true, false, false, (true, -1, false, -1, 255, 1.0))
            }
            // `Projectile.cs:6941-6952` — the crystal's shard (644): 14x14,
            // aiStyle 112, `penetrate = 1`, `timeLeft = 900`,
            // `tileCollide = false`, `ignoreWater`, `alpha = 255`, the
            // idStatic pair with the stock −1 cooldown (`:466`).
            RAINBOW_CRYSTAL_EXPLOSION => {
                (14.0, 14.0, 112, 1, 900, 0, false, false, false, (false, -2, true, -1, 255, 1.0))
            }
            FLAME_BURST_TOWER_1 => (30.0, 54.0, 130, 1, 36000, 0, false, false, false, RESET_STATE),
            FLAME_BURST_TOWER_2 => (28.0, 58.0, 130, 1, 36000, 0, false, false, false, RESET_STATE),
            FLAME_BURST_TOWER_3 => (28.0, 60.0, 130, 1, 36000, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:7153-7162` — the Flame Burst shots (664/666/668):
            // 16x16, aiStyle 131, `alpha = 255`, the local-immunity pair with
            // `localNPCHitCooldown = -1` (one hit per NPC for the shot's life).
            // `friendly` rides the pool default; the stock `timeLeft = 3600`
            // (`:787`) and `penetrate = 1` (`:775`). S1-D-4b-9.
            FLAME_BURST_SHOT_1 | FLAME_BURST_SHOT_2 | FLAME_BURST_SHOT_3 => (
                16.0, 16.0, 131, 1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (true, -1, false, -1, 255, 1.0),
            ),
            BALLISTRA_TOWER_1 | BALLISTRA_TOWER_2 | BALLISTRA_TOWER_3 => {
                (26.0, 54.0, 134, 1, 36000, 0, false, false, false, RESET_STATE)
            }
            // `Projectile.cs:7253-7262` — the Ballista's bolt (680): 16x16,
            // aiStyle 1 (the plain arrow flight), `penetrate = 4`, the
            // local-immunity pair with `localNPCHitCooldown = -1` (one hit per
            // NPC for the bolt's life). `friendly` rides the pool default.
            BALLISTA_SHOT => (16.0, 16.0, 1, 4, TIME_LEFT_DEFAULT, 0, true, false, false, (true, -1, false, -1, 0, 1.0)),
            LIGHTNING_AURA_1 | LIGHTNING_AURA_2 | LIGHTNING_AURA_3 => {
                (16.0, 16.0, 137, -1, 36000, 0, false, false, false, RESET_STATE)
            }
            EXPLOSIVE_TRAP_1 | EXPLOSIVE_TRAP_2 | EXPLOSIVE_TRAP_3 => {
                (16.0, 16.0, 138, -1, 36000, 0, false, false, false, RESET_STATE)
            }
            // `Projectile.cs:7354-7365` — the trap explosions (694-696):
            // 144x144, aiStyle 139, `penetrate = -1`, `tileCollide = false`,
            // `alpha = 255` and the local-immunity pair (30); the stock
            // `timeLeft = 3600` (`:787`). `friendly` rides the pool default.
            EXPLOSIVE_TRAP_EXPLOSION_1
            | EXPLOSIVE_TRAP_EXPLOSION_2
            | EXPLOSIVE_TRAP_EXPLOSION_3 => (
                144.0, 144.0, 139, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, 30, false, -1, 255, 1.0),
            ),
            DEAD_CELLS_BARNACLE => (42.0, 38.0, 197, 1, 36000, 0, true, false, false, RESET_STATE),
            DEAD_CELLS_BARNACLE_SHOT => (14.0, 14.0, 28, 1, 3600, 2, true, false, false, RESET_STATE), // Projectile.cs:9717
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
            // The dungeon-debris trio (`Projectile.cs:7748-7755`): 10x10,
            // aiStyle 10, `penetrate = -1`; everything else keeps the reset
            // (`tileCollide` true `:775`, `timeLeft` 3600 `:787`,
            // `extraUpdates` 0 `:771`). The stock `friendly`/`hostile` are
            // both false (`:788`/`:764` — the arm touches neither), so the
            // trio damages nothing; the pool's slot default is
            // `friendly = true`, so `ai_dungeon_debris` holds the C# state
            // explicitly. The per-type `knockBack = 6f` is dropped — the
            // spawner's input wins (the sand-row precedent above; the debris
            // spawner passes 0, `WorldGen.cs:63844`).
            BLUE_DUNGEON_DEBRIS | GREEN_DUNGEON_DEBRIS | PINK_DUNGEON_DEBRIS => {
                (10.0, 10.0, 10, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // `Projectile.cs:9511-9522` — the Horseman's pumpkin lob
            // (1001, S1-F-13b2 slice B): 26×26, aiStyle 194, hostile,
            // `tileCollide = false`, `ignoreWater`, `timeLeft = 360`,
            // `alpha = 255`; the stock penetrate 1 (`:774`). The arm's
            // `scale = 1f + Main.rand.Next(51) * 0.01f` (`:9518`) is a
            // spawn-time UpdateNPCs-stream draw — the row carries the
            // 1f base, the roll rides the lob's queue site at union
            // (`npc_ai_unicorn_b.rs::horseman_pumpkin_scale`, module
            // docs union point 5 — wired by S1-D-4 W-10). The
            // aiStyle-194 flight (`AI_194_HorsemanPumpkin`, `:41293`)
            // is armed: `ai_horseman_pumpkin` (S1-D-4 census W-10).
            HORSEMAN_PUMPKIN => (
                26.0,
                26.0,
                194,
                1,
                360,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:9237-9252` — the Light's Bane slash (974):
            // 20x20, aiStyle 188, `penetrate = 2`, `tileCollide = false`,
            // `ignoreWater = true`, `timeLeft = 30`, the local-immunity pair
            // (`usesLocalNPCImmunity`, `localNPCHitCooldown = -1`). The
            // melee/armorPenetration/stopsDealingDamageAfterPenetrateHits
            // fields are damage-side and stay with the melee port.
            LIGHTS_BANE => (20.0, 20.0, 188, 2, 30, 0, false, false, false, (true, -1, false, -1, 0, 1.0)),
            // `Projectile.cs:7608-7616` — the Queen Bee's stinger (719):
            // 10x10, hostile, `penetrate = -1`, aiStyle 1 (the plain
            // flight), `tileCollide = true`; the stock 3600 life (the
            // AI_043 arm overrides to 300 post-spawn, `NPC.cs:30743`).
            QUEEN_BEE_STINGER => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-d22 — `Projectile.cs:1375-1383`: the Hornet's stinger
            // (55): 10x10, hostile, `penetrate = -1`, aiStyle 1,
            // `tileCollide = true`; the stock 3600 life (the AI_005 arm
            // overrides to 300 post-spawn, `NPC.cs:50797`). It is on the
            // `flag3 = false` flight-counter list (`:55954`), so it flies
            // dead straight (no gravity) — the `ai_hornet_stinger` arm.
            HORNET_STINGER => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // S1-D-3 W-9 fishron wiring — `Projectile.cs:4676-4685`: the
            // Sharknado bolt (385): 30x30, aiStyle 65, penetrate -1,
            // alpha 255, timeLeft 300 (the stock tileCollide true).
            SHARKNADO_BOLT => (30.0, 30.0, 65, -1, 300, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:4664-4675`: the ForceBubble (384): 150x42,
            // aiStyle 64, penetrate -1, tileCollide false, ignoreWater,
            // alpha 255, timeLeft 540.
            FORCE_BUBBLE => (150.0, 42.0, 64, -1, 540, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:4686-4697`: the Cthulunado (386): 150x42,
            // aiStyle 64, penetrate -1, tileCollide false, ignoreWater,
            // alpha 255, timeLeft 840.
            CTHULUNADO => (150.0, 42.0, 64, -1, 840, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // ===== S1-D-4: the Empress of Light's projectiles =====
            // `Projectile.cs:8403-8415` — the Lasting Rainbow (872): 30x30,
            // aiStyle 173, `alpha = 255`, `penetrate = -1`, hostile
            // (`friendly = false` rides the pool's `!friendly` proxy),
            // `timeLeft = 660`, `tileCollide = false`, `ignoreWater = true`
            // (the overlay below). No arrow/trap, stock extraUpdates 0.
            HALLOW_BOSS_LASTING_RAINBOW => (
                30.0, 30.0, 173, -1, 660, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:8354-8366` — the Rainbow Streak (873): 30x30,
            // aiStyle 171, `alpha = 255`, `penetrate = -1`, hostile,
            // `timeLeft = 200`, `tileCollide = false`, `ignoreWater = true`.
            HALLOW_BOSS_RAINBOW_STREAK => (
                30.0, 30.0, 171, -1, 200, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:8367-8379` — the Death Aurora (874): 30x30,
            // aiStyle **0** (`AI()` has no style-0 arm — a pure visual),
            // `alpha = 255`, `penetrate = -1`, hostile, `timeLeft = 210`,
            // `tileCollide = false`, `ignoreWater = true`.
            HALLOW_BOSS_DEATH_AURORA => (
                30.0, 30.0, 0, -1, 210, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:8714-8726` — the Fairy Queen Lance (919): 8x8,
            // aiStyle 179, `alpha = 255`, `penetrate = -1`, hostile,
            // `timeLeft = 240`, `tileCollide = false`, `ignoreWater = true`.
            FAIRY_QUEEN_LANCE => (
                8.0, 8.0, 179, -1, 240, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:8757-8770` — the Sun Dance (923): 30x30,
            // aiStyle 180, `alpha = 255`, `penetrate = -1`, hostile,
            // `timeLeft = 180`, `tileCollide = false`, `ignoreWater = true`.
            // `drawLayer = 2` (`:8769`) is render-side — the port's draw-layer
            // work owns it (documented skip, ANALYSIS S1-D-4).
            FAIRY_QUEEN_SUN_DANCE => (
                30.0, 30.0, 180, -1, 180, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // The Frost Moon family (S1-D-4, `ProjectileID.cs:1244-1258`):
            // the 345/346/348/349/350/351/352 rows are the S1-F-13c wiring
            // rows ABOVE (value-identical in both branches — deduped at
            // merge, deny(unreachable_patterns) keeps one arm per id).
            // `Projectile.cs:4370-4377` — the ornament's shrapnel (347):
            // 6x6, aiStyle 2, hostile, `penetrate = -1`; the stock life.
            ORNAMENT_SHRAPNEL => (6.0, 6.0, 2, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // S1-P-i22 — the magic guns + tomes' shots. The verbatim
            // `SetDefaults` arms, in `ProjectileID` order; `light`/`magic`/
            // `melee`/`ranged`/`ignoreWater`/`drawLayer`/`armorPenetration`
            // have no `ProjData` consumer (dropped, as the Phase-1 rows note).
            // `Projectile.cs:886-896` — the Starfury star (9): 24x24, aiStyle 5,
            // `penetrate = 2`, `scale = 0.8f`, `tileCollide = false`, melee.
            STARFURY => (24.0, 24.0, 5, 2, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.8)),
            // `Projectile.cs:1556-1593` — the harp notes (76/77/78): aiStyle 21,
            // `alpha = 100`, `penetrate = 5`, `timeLeft = 180`, ignoreWater; the
            // per-kind box (76: 10x22, 77: 18x24, 78: 22x24). The `switch (2)`
            // local-immunity arm (`:1588-1591`) sets `localNPCHitCooldown = 30`.
            QUARTER_NOTE => (10.0, 22.0, 21, 5, 180, 0, true, false, false, (true, 30, false, -1, 100, 1.0)),
            EIGHTH_NOTE => (18.0, 24.0, 21, 5, 180, 0, true, false, false, (true, 30, false, -1, 100, 1.0)),
            TIED_EIGHTH_NOTE => (22.0, 24.0, 21, 5, 180, 0, true, false, false, (true, 30, false, -1, 100, 1.0)),
            // `Projectile.cs:2398-2410` — the Nettle Burst segments (150-152):
            // 28x28, aiStyle 4, `penetrate = -1`, `tileCollide = false`,
            // `alpha = 255`, ignoreWater (`armorPenetration = 10` dropped).
            NETTLE_BURST_RIGHT | NETTLE_BURST_LEFT | NETTLE_BURST_END => (
                28.0, 28.0, 4, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:3441-3451` — the Heat Ray (260): 8x8, aiStyle 48,
            // `extraUpdates = 100`, `timeLeft = 200`, `penetrate = 1`.
            HEAT_RAY => (8.0, 8.0, 48, 1, 200, 100, true, false, false, RESET_STATE),
            // `Projectile.cs:3503-3515` — the Poison Fang (265): 12x12,
            // aiStyle 1, `alpha = 255`, `penetrate = 3`, local immunity 10,
            // `timeLeft = 37`.
            POISON_FANG => (12.0, 12.0, 1, 3, 37, 0, true, false, false, (true, 10, false, -1, 255, 1.0)),
            // The Electrosphere Missile 442 + sphere 443 rows live in main's
            // i16 dart block (numeric arms above; the sphere carries the
            // NewProjectileSetup 300-tick seed, Projectile.cs:11065-11068) —
            // deduped keep-one here, the named consts stay live.
            // `Projectile.cs:5292-5303` — the Charged Blaster Orb (459): 22x22,
            // aiStyle 1, `alpha = 255`, `extraUpdates = 1`, ignoreWater, magic.
            CHARGED_BLASTER_ORB => (22.0, 22.0, 1, 1, TIME_LEFT_DEFAULT, 1, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:5304-5316` — the Charged Blaster Cannon holder
            // (460): 14x18, aiStyle 75, `penetrate = -1`, `tileCollide = false`,
            // ignoreWater, magic (`drawLayer = 8`/`usesOwnerLight` dropped).
            CHARGED_BLASTER_CANNON => (14.0, 18.0, 75, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:5317-5328` — the Charged Blaster Laser (461):
            // 18x18, aiStyle 84, `penetrate = -1`, `alpha = 255`,
            // `tileCollide = false`, magic (`hide = true` dropped).
            CHARGED_BLASTER_BEAM => (18.0, 18.0, 84, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:5908-5918` — the Crystal Pulse (521): 14x14,
            // aiStyle 29, `alpha = 255`, `penetrate = 1`, `extraUpdates = 1`.
            CRYSTAL_PULSE => (14.0, 14.0, 29, 1, TIME_LEFT_DEFAULT, 1, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:8206-8219` — the Sparkle Guitar note (856): 32x32,
            // aiStyle 167, `alpha = 255`, `tileCollide = false`,
            // `extraUpdates = 2`, local immunity −1, ignoreWater, magic.
            SPARKLE_GUITAR => (32.0, 32.0, 167, 1, TIME_LEFT_DEFAULT, 2, false, false, false, (true, -1, false, -1, 255, 1.0)),
            // `Projectile.cs:9013-9023` — the Star Cannon star (955): 18x18,
            // aiStyle 5, `penetrate = -1`, `alpha = 50`, ranged (`light = 1f`
            // dropped).
            STAR_CANNON_STAR => (18.0, 18.0, 5, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 50, 1.0)),
            // The spear family (S1-P-i06, aiStyle 19 — `DefaultToSpear`
            // `Projectile.cs:10720-10730`: friendly, `penetrate = -1`,
            // `tileCollide = false`, melee, `ownerHitCheck`, `usesOwnerLight`,
            // drawLayer 7). The stock `timeLeft = 3600` (`:787`),
            // `extraUpdates = 0`, NO local/idStatic immunity slice. The
            // per-kind boxes/scales/alphas are the `SetDefaults` arms; the
            // `ownerHitCheck` damage gate is keyed on `SPEAR_KINDS` in
            // `damage_pve` (the only ported ownerHitCheck family here).
            // `:1296-1302` — Dark Lance (46): 20x20, scale 1.1.
            DARK_LANCE => (20.0, 20.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // `:1303-1309` — Trident (47): 18x18, scale 1.1.
            TRIDENT => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // `:1319-1325` — Spear (49): 18x18, scale 1.2.
            SPEAR => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.2)),
            // `:1442-1448` — Mythril Halberd (64): 18x18, scale 1.25.
            MYTHRIL_HALBERD => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.25)),
            // `:1460-1466` — Adamantite Glaive (66): 18x18, scale 1.27.
            ADAMANTITE_GLAIVE => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.27)),
            // `:1816-1822` — Cobalt Naginata (97): 18x18, scale 1.1.
            COBALT_NAGINATA => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // `:1929-1935` — Gungnir (105): 18x18, scale 1.3.
            GUNGNIR => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.3)),
            // `:2191-2197` — Mushroom Spear (130): 22x22, scale 1.2.
            MUSHROOM_SPEAR => (22.0, 22.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.2)),
            // `:2411-2417` — The Rotted Fork (153): 18x18, scale 1.1.
            THE_ROTTED_FORK => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // `:3004-3010` — Palladium Pike (212): 18x18, scale 1.12.
            PALLADIUM_PIKE => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.12)),
            // `:3024-3030` — Orichalcum Halberd (215): 18x18, scale 1.27.
            ORICHALCUM_HALBERD => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.27)),
            // `:3045-3051` — Titanium Trident (218): 18x18, scale 1.28.
            TITANIUM_TRIDENT => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.28)),
            // `:3079-3085` — Chlorophyte Partisan (222): 18x18, scale 1.3.
            CHLOROPHYTE_PARTISAN => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.3)),
            // `:4318-4325` — North Pole (342): 22x2, scale 1.1, coldDamage
            // (the cold flag has no port consumer — the debuff layer is out
            // of scope, ANALYSIS).
            NORTH_POLE => (22.0, 2.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // `:4517-4523` — Obsidian Swordfish (367): 18x18, scale 1.1.
            OBSIDIAN_SWORDFISH => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // `:4524-4529` — Swordfish (368): 18x18, NO scale line (stock 1f).
            SWORDFISH => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `:7688-7693` — Thunder Spear (730): 18x18, stock scale 1f.
            THUNDER_SPEAR => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `:8430-8437` — the jousting lances (877/878/879): 25x25,
            // `alpha = 255`, `netImportant` (the MP flag has no single-player
            // consumer). The channel hold + the speed-dust arms read the
            // 877-879 range in `ai_spear`.
            JOUSTING_LANCE | SHADOW_JOUSTING_LANCE | HALLOW_JOUSTING_LANCE => {
                (25.0, 25.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // `:10282-10289` — Slime Spear (1103): 18x18, scale 1.2, alpha 90.
            SLIME_SPEAR => (18.0, 18.0, 19, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 90, 1.2)),
            // ---- The spear family's sub-shots (S1-P-i06). They ride their
            // own aiStyles (the `AI_019_Spears` spawns, `:45891-45936`).
            // `:2198-2208` — the Mushroom Spear's spore (131): 22x22,
            // aiStyle 30, `penetrate = 1`, `tileCollide = false`, melee,
            // `light = 0.5f` (the light has no port consumer here). The
            // aiStyle-30 default arm fades it (alpha += 4, kill at 255).
            MUSHROOM_SPORE => (22.0, 22.0, 30, 1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `:3137-3150` — the Chlorophyte Partisan's spore cloud (228):
            // 30x30, aiStyle 44, `penetrate = -1`, `tileCollide = false`,
            // melee, scale 1.1, idStatic immunity cooldown 10.
            CHLOROPHYTE_SPORE_CLOUD => (30.0, 30.0, 44, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, true, 10, 0, 1.1)),
            // `:7705-7715` — the Thunder Spear's bolt (732): 15x15,
            // aiStyle 44, `penetrate = 1`, `tileCollide = true`, melee.
            THUNDER_SPEAR_BOLT => (15.0, 15.0, 44, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `:4326-4338` — the North Pole's snowflake (343): 10x10,
            // aiStyle 57, `penetrate = 3`, `alpha = 255`, melee, scale 1.1,
            // coldDamage (no port consumer), `timeLeft = 120`.
            NORTH_POLE_SNOWFLAKE => (10.0, 10.0, 57, 3, 120, 0, true, false, false, (false, -2, false, -1, 255, 1.1)),
            // `:4339-4352` — the North Pole's falling flake (344): 26x26,
            // aiStyle 1, `alpha = 255`, melee, scale 0.9, coldDamage,
            // `tileCollide = false`, `timeLeft = 240`.
            NORTH_POLE_FLAKE => (26.0, 26.0, 1, 1, 240, 0, false, false, false, (false, -2, false, -1, 255, 0.9)),
            // `:10316-10339` — the Obsidian Swordfish's bones (1107-1109):
            // aiStyle 14, `penetrate = 3`, melee, `timeLeft = 360` (the spawn
            // overrides to Next(30,150)). The boxes: 1107 14x16, 1108 12x14,
            // 1109 6x12. The generic AI_014 tail flies them (no type branch).
            SWORDFISH_BONE_1 => (14.0, 16.0, 14, 3, 360, 0, true, false, false, RESET_STATE),
            SWORDFISH_BONE_2 => (12.0, 14.0, 14, 3, 360, 0, true, false, false, RESET_STATE),
            SWORDFISH_BONE_3 => (6.0, 12.0, 14, 3, 360, 0, true, false, false, RESET_STATE),
            // S1-P-i29 — the drill family (`DefaultToDrillOrChainsaw`,
            // `Projectile.cs:10732-10742`): aiStyle 20, `penetrate = -1`,
            // `tileCollide = false`, the stock life (the AI pins
            // `timeLeft = 60` every tick, `:25483`). The per-kind boxes and
            // scales ride the `SetDefaults` type arms; the state group's
            // scale slot carries them.
            DRILL_COBALT => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.9)),
            DRILL_MYTHRIL => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.9)),
            DRILL_ADAMANTITE => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.9)),
            DRILL_PALLADIUM => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.92)),
            DRILL_ORICHALCUM => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.93)),
            DRILL_TITANIUM => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.95)),
            DRILL_CHLOROPHYTE => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.0)),
            DRILL_VORTEX => (26.0, 54.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            DRILL_NEBULA => (30.0, 54.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            DRILL_SOLAR => (30.0, 54.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            DRILL_STARDUST => (30.0, 54.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            DRAX => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            // S1-P-i30 — the chainsaw family (the same
            // `DefaultToDrillOrChainsaw` group; the boxes/scales are the
            // per-type `SetDefaults` arms cited on the constants). The
            // Butcher's Chainsaw 509 carries `usesIDStaticNPCImmunity =
            // true` + `idStaticNPCHitCooldown = 10` (`Projectile.cs:
            // 5792-5793`).
            CHAINSAW_COBALT => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            CHAINSAW_MYTHRIL => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.08)),
            CHAINSAW_ADAMANTITE => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.16)),
            CHAINSAW_PALLADIUM => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            CHAINSAW_ORICHALCUM => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.12)),
            CHAINSAW_TITANIUM => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.2)),
            CHAINSAW_CHLOROPHYTE => (18.0, 18.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 1.1)),
            CHAINSAW_SAWTOOTH_SHARK => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            CHAINSAW_VORTEX => (22.0, 56.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            CHAINSAW_NEBULA => (18.0, 56.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            CHAINSAW_SOLAR => (28.0, 64.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            CHAINSAW_BUTCHERS => (22.0, 22.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, true, 10, 0, 1.2)),
            CHAINSAW_STARDUST => (28.0, 64.0, 20, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // The Laser Drill's beam (445): aiStyle 75, 10x10,
            // `penetrate = -1`, `tileCollide = false` (`Projectile.cs:5162-
            // 5172`). The port holds its channel like the drills; the full
            // aiStyle-75 laser AI is a follow-up.
            LASER_DRILL => (10.0, 10.0, 75, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:7626-7635` — the golf ball (721) and the 14
            // dyed twins (739-752): 7x7, aiStyle 149, friendly, penetrate
            // -1, tileCollide false, netImportant (the overlay below);
            // the stock 3600 life (the AI floors it at 10 and the strike
            // re-arms it to 18000). S1-P-i55.
            GOLF_BALL | 739..=752 => {
                (7.0, 7.0, 149, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)
            }
            // `Projectile.cs:7636-7645` — the club helper (722): 2x2,
            // aiStyle 150, tileCollide false, ignoreWater (the overlay),
            // penetrate -1, hide (draw-only, no port field). S1-P-i55.
            GOLF_CLUB_HELPER => {
                (2.0, 2.0, 150, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)
            }
            _ => return None,
        };
    let (state_local, state_local_cooldown) = match kind {
        133 | 136 | 139 | 142 | 777 | 781 | 785 | 788 | 794 | 797 | 800 => (true, -1),
        85 => (true, -1),
        932 => (true, 60),
        // `DefaultToFlail`'s pair (`:10753-10762`) — the ten flail kinds.
        BALL_O_HURT | BLUE_MOON | SUNFURY | DAO_OF_POW | THE_MEATBALL | FLOWER_POW
        | DRIPPLER_FLAIL | MACE | FLAMING_MACE | FLAIRON => (true, 10),
        // The summon rows' local pairs (Phase 1): the Guardian
        // (`Projectile.cs:6721-6722`), the Storm Tigers
        // (`:8096-8097`/`:8110-8111`/`:8124-8125`), Abigail (`:9111-9112`),
        // the Lightning Auras (`:7332-7333`), the baby bird (`:7841-7842`).
        STARDUST_GUARDIAN => (true, 5),
        STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3 => (true, 10),
        // The pirate trio's LOCAL pair (`Projectile.cs:4765-4766`).
        ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN => (true, 18),
        ABIGAIL => (true, 20),
        BABY_BIRD => (true, 15),
        // The 266's pair is idStatic-12 (`Projectile.cs:3526-3527`) —
        // it rides its own row's state group, NOT this local overlay
        // (AI_026 slice C's row witness caught the misfile).
        LIGHTNING_AURA_1 | LIGHTNING_AURA_2 | LIGHTNING_AURA_3 => (true, 3),
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
    // The summon identification (M4-PROJ-8 Phase 1): the
    // `minion`/`minionSlots` slice of the arms below. Everything else keeps
    // the stock `minion = false` / `minionSlots = 0f`
    // (`Projectile.cs:775-776`).
    let (minion, sentry, minion_slots) = match kind {
        HORNET | FLYING_IMP | TEMPEST | UFO_MINION | STARDUST_CELL | BABY_BIRD
        | SLIME_MINION | DEADLY_SPHERE | ONE_EYED_PIRATE | SOULSCOURGE_PIRATE
        | PIRATE_CAPTAIN | VAMPIRE_FROG => (true, false, 1.0),
        RETANIMINI | SPAZMAMINI => (true, false, 0.5),
        STARDUST_GUARDIAN | STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3 | ABIGAIL => {
            (true, false, 0.0)
        }
        // S1-P-i25: the summon-staff minions' missing identification rows —
        // every one a full slot (`minionSlots = 1f`, the cited `SetDefaults`
        // arms), the dragon BODY segments (626/627) half a slot
        // (`Projectile.cs:6735-6738`), the head/tail (625/628) zero (the slot
        // block excludes them, `:16709`).
        191 | 192 | 193 | 194 | 317 | 390 | 391 | 392 | 755 | 831 | 864 | 946 | 951
        | 970 | 1022 | 1093 | 1094 | 1112 | 1113 | 1118 | 1119 => (true, false, 1.0),
        STARDUST_DRAGON_HEAD | STARDUST_DRAGON_TAIL => (true, false, 0.0),
        STARDUST_DRAGON_BODY_1 | STARDUST_DRAGON_BODY_2 => (true, false, 0.5),
        FROST_HYDRA | QUEEN_SPIDER | MOONLORD_TURRET | RAINBOW_CRYSTAL | FLAME_BURST_TOWER_1
        | FLAME_BURST_TOWER_2 | FLAME_BURST_TOWER_3 | BALLISTRA_TOWER_1 | BALLISTRA_TOWER_2
        | BALLISTRA_TOWER_3 | LIGHTNING_AURA_1 | LIGHTNING_AURA_2 | LIGHTNING_AURA_3
        | EXPLOSIVE_TRAP_1 | EXPLOSIVE_TRAP_2 | EXPLOSIVE_TRAP_3 | HOUNDIUS_SHOOTIUS
        | DEAD_CELLS_BARNACLE => (false, true, 0.0),
        _ => (false, false, 0.0),
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
                    | TEMPEST | RETANIMINI | SPAZMAMINI | VAMPIRE_FROG
            ),
        id_static_hit_cooldown: if matches!(kind, TEMPEST | RETANIMINI | SPAZMAMINI | VAMPIRE_FROG) {
            if kind == TEMPEST || kind == VAMPIRE_FROG {
                // `Projectile.cs:7827` — the frog's idStatic cooldown is 10.
                10
            } else if kind == RETANIMINI {
                16
            } else {
                12
            }
        } else if uses_id_static_immunity {
            id_static_hit_cooldown
        } else if matches!(kind, TERRARIAN_YOYO | TERRARIAN_BEAM | KRAKEN_WAVE) {
            10
        } else if kind == CRYSTAL_SHARD_MELEE {
            20
        } else {
            -1
        },
        // `UseGroupStaticImmunity(181)` (`Projectile.cs:10676-10681`) — the
        // Bee Gun's bee 181 and giant bee 566 share the identity 181, so the
        // id-static table keys both under one cooldown. Every other row keeps
        // 0 = "use the kind" (the `:782` reset `immunityIdentity = type`).
        immunity_identity: if matches!(kind, BEE | GIANT_BEE) { BEE } else { 0 },
        alpha,
        // the yoyo family's per-kind scale ladder (`DefaultToYoyo`); every
        // other row carries its own scale in the state group.
        scale: if YOYO_KINDS.contains(&kind) {
            set_defaults_scale(kind)
        } else {
            scale
        },
        // The `SetDefaults` `light` slice (S1-E-5): the `ProjLight` amount
        // the frame feed multiplies (`proj_lights.rs`); the unlisted kinds
        // keep the stock 0f and stay dark.
        light: set_defaults_light(kind),
        // The summon identification (M4-PROJ-8 Phase 1): the
        // `minion`/`minionSlots` slice; every summon arm sets `netImportant`
        // and `ignoreWater` (but the Storm Tigers keep the stock false).
        minion,
        sentry,
        minion_slots,
        net_important: minion
            || sentry
            // S1-P-i21: the Nimbus Rod's two clouds set `netImportant`
            // (`Projectile.cs:3186`/`:3194`); the rain drop 239 keeps false.
            || matches!(kind, RAIN_CLOUD_MOVING | RAIN_CLOUD_RAINING)
            // S1-P-i35 — the bobber family sets `netImportant` itself
            // (`Projectile.cs:4514`/`:9428`).
            || BOBBER_KINDS.contains(&kind)
            // The slice-B pets set `netImportant` themselves
            // (`Projectile.cs:5419`/`:9643`/`:9993`).
            || matches!(kind, CRIMSON_HEART | DIRTIEST_BLOCK | BOULDER_PET)
            // S1-P-i42 — the glowstick/flare family sets `netImportant = true`
            // (`Projectile.cs:1328`/`:1356`/`:2517`/`:4024`/`:5487`/`:5853`/
            // `:8345`/`:9582`/`:9593`/`:9604`/`:9615`/`:10155`).
            || matches!(
                kind,
                GLOWSTICK | STICKY_GLOWSTICK | SPELUNKER_GLOWSTICK | BOUNCY_GLOWSTICK
                    | FAIRY_GLOWSTICK | RAINBOW_GLOWSTICK | FLARE | BLUE_FLARE
                    | SPELUNKER_FLARE | CURSED_FLARE | RAINBOW_FLARE | SHIMMER_FLARE
            )
            // AI_026 slice B: the boss-head pets' `netImportant = true`
            // (`Projectile.cs:8488`/`:8523`).
            || matches!(kind, SKELETRON_PET | SKELETRON_PRIME_PET)
            // AI_026 slice A: the flyer pets' `netImportant = true`
            // (`Projectile.cs:2833/2843/2853/2994/4647/7913/7965/7985/
            // 8488/8523/8545/8591/9910`; the 175's block `:2612-2620`
            // carries no such line and keeps the stock false).
            || matches!(
                kind,
                BABY_SKELETRON_HEAD | BABY_HORNET | TIKI_SPIRIT | WISP | ZEPHYR_FISH
                    | SHARK_PUP | LIL_HARPY | GLITTERY_BUTTERFLY | QUEEN_BEE_PET
                    | DUKE_FISHRON_PET | MOON_LORD_PET | DD2_BETSY_PET | PUFFERFISH
            )
            // S1-P-i55 — `Projectile.cs:7628`: the golf balls (721 + the 14
            // dyed twins) set `netImportant = true` themselves.
            || matches!(kind, GOLF_BALL | 739..=752),
        ignore_water: ((minion || sentry)
            && !matches!(
                kind,
                STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3
                    // The aiStyle-67 pirate/frog rows set neither
                    // ignoreWater (`Projectile.cs:4755-4768`/`:7814-7828`).
                    | ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN | VAMPIRE_FROG
                    // The 266's block (`Projectile.cs:3516-3528`) sets no
                    // ignoreWater either — the stock false (`:761`) stands
                    // (AI_026 slice C; its wet-swim arm `:64475-64485` is a
                    // live C# branch the port folds dry with the wet flags).
                    | SLIME_MINION
                    // S1-P-i25: the new aiStyle-26/54/67 minion rows set no
                    // ignoreWater either — the pygmies (`:2791-2813`), the
                    // raven (`:4099-4111`), the spiders (`:4742-4754`), the
                    // foxsparks (`:10210-10221`/`:10377-10388`), the flinx
                    // (`:8969-8983`), the mushroom boi (`:9667-9680`), the
                    // cattivas (`:10195-10209`/`:10362-10376`) and the clay
                    // pot (`:10450-10463`).
                    | 191 | 192 | 193 | 194 | 317 | 390 | 391 | 392 | 1094 | 1113
                    | 951 | 1022 | 1093 | 1112 | 1118
            ))
            // `Projectile.cs:874-884`: the Vilethorn pair sets `ignoreWater`.
            || matches!(kind, VILE_BASE | VILE_TIP)
            // The slice-B pets set `ignoreWater` themselves
            // (`Projectile.cs:5422`/`:9646`/`:9996`) — they are not minions.
            || matches!(kind, CRIMSON_HEART | DIRTIEST_BLOCK | BOULDER_PET)
            // AI_026 slice A: the Tiki Spirit (`Projectile.cs:2860`) and the
            // Wisp (`:3002`) set `ignoreWater`; the other flyers keep false.
            || matches!(kind, TIKI_SPIRIT | WISP)
            // `Projectile.cs:9244`: the Light's Bane slash ignores water.
            || kind == LIGHTS_BANE
            // `Projectile.cs:6949`: the crystal shard's `ignoreWater = true`.
            || kind == RAINBOW_CRYSTAL_EXPLOSION
            // `Projectile.cs:9517` — the Horseman's pumpkin lob (1001).
            || kind == HORSEMAN_PUMPKIN
            // S1-D-3 W-9: the sharknado clouds' `ignoreWater = true`
            // (`Projectile.cs:4672`/`:4694`).
            || matches!(kind, FORCE_BUBBLE | CTHULUNADO)
            // S1-P-i21: the Nimbus Rod's raining cloud sets `ignoreWater`
            // (`Projectile.cs:3195`); the moving cloud 237 and the drop 239
            // keep the stock false.
            || kind == RAIN_CLOUD_RAINING
            // S1-D-4: the Empress family's five arms all set `ignoreWater`
            // (`Projectile.cs:8365`/`:8378`/`:8414`/`:8725`/`:8768`).
            || matches!(
                kind,
                HALLOW_BOSS_LASTING_RAINBOW
                    | HALLOW_BOSS_RAINBOW_STREAK
                    | HALLOW_BOSS_DEATH_AURORA
                    | FAIRY_QUEEN_LANCE
                    | FAIRY_QUEEN_SUN_DANCE
            )
            // `Projectile.cs:6868`/`:6883`/`:6898` — the moon-lord drop
            // shots' `ignoreWater = true` (S1-D-4).
            || matches!(kind, MOONLORD_BULLET | MOONLORD_ARROW | MOONLORD_ARROW_TRAIL)
            // S1-P-i20: the Aqua Scepter (22, `Projectile.cs:1049`) and the
            // Magic Missile (16, `:983`) set `ignoreWater = true`; the
            // Flamelash (34) does NOT (`:1162-1173` has no such line).
            || matches!(kind, AQUA_SCEPTER | MAGIC_MISSILE)
            // S1-P-i13/i17 — the bow-launch arrows that set `ignoreWater` in
            // their `SetDefaults` bodies: the JestersArrow 5
            // (`Projectile.cs:859`) and the DD2BetsyArrow 710 (`:7529`).
            || matches!(kind, 5 | 710)
            // S1-P-i16 — the Electrosphere sphere 443's `ignoreWater = true`
            // (`Projectile.cs:5147`, the `:5137-5149` row block).
            || kind == 443
            // S1-P-i14 — the bullet-launch projectiles that set `ignoreWater`:
            // the Xenopopper bubble 444 (`Projectile.cs:5159`) and the Onyx
            // black bolt 661 (`:7118`).
            || matches!(kind, XENOPOPPER_BUBBLE | ONYX_BLACK_BOLT)
            // S1-P-i18 — `Projectile.cs:3251`: the High Velocity Bullet's
            // `ignoreWater = true`.
            || kind == HIGH_VELOCITY_BULLET
            // S1-P-i55 — `Projectile.cs:7642`: the club helper (722) sets
            // `ignoreWater = true`.
            || kind == GOLF_CLUB_HELPER,
        // The aiStyle-67 summon rows set `decidesManualFallThrough`
        // (`:4767`/`:7825`/`:8092`/`:8106`/`:8120`); the pirates/frog's arm
        // drives `shouldFallThrough` (`:49561`/`:51206`).
        decides_manual_fall_through: matches!(
            kind,
            ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN | VAMPIRE_FROG
                | STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3
                // S1-P-i25: the new aiStyle-67 rows' `decidesManualFallThrough`
                // — the flinx (`:8980`), the mushroom boi (`:9679`), the
                // cattivas (`:10207`/`:10374`) and the clay pot (`:10462`).
                | 951 | 1022 | 1093 | 1112 | 1118
        ),
        // The damage-class flags (`Projectile.cs:751-753` reset false) the
        // per-hit crit block reads (`:12890-12898`). The census is the
        // `SetDefaults` `this.melee/ranged/magic = true` writes plus the
        // `DefaultTo*` helpers: yoyo/flail/spear/drill/shortsword are melee,
        // the whip sets NONE (`DefaultToWhip` `:10704-10718`). The bombs
        // (28/29/37/519) and the Crystal Storm item 518's proj set none.
        melee: YOYO_KINDS.contains(&kind)
            || FLAIL_KINDS.contains(&kind)
            || matches!(
                kind,
                // The explicit melee rows: the boomerangs (6/52/113,
                // `:871`/`:1352`/`:2019`), the sword beams (116/132/156/157/173,
                // `:2052`/`:2214`/`:2439`/`:2452`/`:2596`), the Light's Bane
                // slash (974, `:9245`), the Terrarian beam (604, `:6079`), the
                // yoyo on-hit shots (1125/1130/1134/1135,
                // `:10535`/`:10605`/`:10648`/`:10661`), the flail helpers
                // (248/405/928, `:3312`/`:4866`/`:8801`).
                ENCHANTED_BOOMERANG | WOODEN_BOOMERANG | ICE_BOOMERANG
                    | SWORD_BEAM | TERRA_BEAM | LIGHT_BEAM | NIGHT_BEAM | ENCHANTED_BEAM
                    | LIGHTS_BANE | TERRARIAN_BEAM
                    | CASCADE_EXPLOSION | EYE_OF_CTHULHU_DASH | CRYSTAL_SHARD_MELEE | KRAKEN_WAVE
                    | FLOWER_POW_PETAL | FLAIRON_BUBBLE | DRIPPLER_EXTRA_BALL
            ),
        ranged: matches!(
            kind,
            // The arrows/bullets (1/2/14, `:814`/`:825`/`:961`), the thrown
            // kinds (3/48/166, `:835`/`:1317`/`:2555`), the moon-lord drop
            // shots (638/639/640, `:6867`/`:6880`/`:6895`), the DD2 kobold
            // bomb (681, `:7270`).
            WOODEN_ARROW | FLAME_ARROW | MUSKET_BALL
                | SHURIKEN | THROWING_KNIFE | SNOWBALL
                | MOONLORD_BULLET | MOONLORD_ARROW | MOONLORD_ARROW_TRAIL
                | DD2_BOMB
            // W3-i11 round 2 — the ranged census completion (the player-fired
            // rows the first pass missed; the per-hit ranged crit never fired
            // for them). The Sandgun balls (42/65/68/354,
            // `:1254`/`:1456`/`:1484`/`:4447`), the Flamethrower's Flames (85,
            // `:1669`), the special arrows (117/469/485/495/819,
            // `:2066`/`:5443`/`:5611`/`:5688`/`:8003`), the launcher rockets
            // (133-144 `:2229-2336`, 338-341 `:4315`, the 776-801/803-810
            // group `:7946` — 779/783 are row-less, the guard above), the
            // gunProj rockets (615/630/705/714,
            // `:6629`/`:6776`/`:7581`), the Celebration rockets (715-718,
            // `:7592`/`:7604`) and the Empress shot (932, `:8844`).
                | 42 | 65 | 68 | 354 | 85 | 117 | 469 | 485 | 495 | 819
                | 133..=144 | 338..=341 | 615 | 630 | 705 | 714 | 715..=718 | 932
                | 776..=801 | 803..=810
            // The hostile snowmen's shots (109/110, `:1976`/`:1989`) — the
            // C# writes ranged on them too; the owner-255 gate keeps the
            // flags moot, the census stays verbatim.
                | SNOW_BALL_HOSTILE | BULLET_SNOWMAN
        ),
        magic: matches!(
            kind,
            // The Vilethorn pair (7/8, `:884`), the magic spells
            // (15/16/20/27/34/45/79/80/88/94/95,
            // `:972`/`:982`/`:1029`/`:1104`/`:1170`/`:1294`/`:1601`/`:1613`/
            // `:1713`/`:1788`/`:1800`), the gem bolts (121-126/597,
            // `:2111`+`ApplyGemStaffStats`), the Unholy Trident pair
            // (114/115, `:2028`/`:2042`), the frost/flower shots
            // (253/254/255, `:3366`/`:3374`/`:3384`), the Inferno Fork
            // (295/296, `:3856`/`:3866`), the Spectre pair (297/356,
            // `:3877`/`:4470`), the Poison Staff / Frost Staff (355/359,
            // `:4458`/`:4503`), the Typhoon (409, `:4914`), the Leaf (206,
            // `:2946`), the Nebula Arcanum pair (617/618,
            // `:6653`/`:6666`), the Last Prism pair (632/633,
            // `:6811`/`:6802`), the Nebula Blaze pair (634/635,
            // `:6825`/`:6836`), the Lunar Flare (645, `:6959`), the Spirit
            // Flame / Sky Fracture (659/660, `:7089`/`:7104`), the Book of
            // Skulls (837, `:8131`), the Wand of Sparking spark (954,
            // `:9008`).
            VILE_BASE | VILE_TIP
                | BALL_OF_FIRE | MAGIC_MISSILE | GREEN_LASER | WATER_BOLT | FLAMELASH
                | DEMON_SCYTHE | RAINBOW_ROD_BULLET | ICE_BLOCK | PURPLE_LASER
                | CRYSTAL_STORM | CURSED_FLAME_FRIENDLY
                | GEM_BOLT_AMETHYST | GEM_BOLT_TOPAZ | GEM_BOLT_SAPPHIRE | GEM_BOLT_EMERALD
                | GEM_BOLT_RUBY | GEM_BOLT_DIAMOND | GEM_BOLT_AMBER
                | UNHOLY_TRIDENT | UNHOLY_TRIDENT_HOSTILE
                | BALL_OF_FROST | MAGNET_SPHERE_BALL | MAGNET_SPHERE_BOLT
                | INFERNO_FORK_BOLT | INFERNO_BLAST | LOST_SOUL | SPECTRE_WRATH
                | VENOM_FANG | FROST_BOLT | TYPHOON | LEAF
                | NEBULA_ARCANUM | NEBULA_ARCANUM_SUBSHOT
                | LAST_PRISM_LASER | LAST_PRISM | NEBULA_BLAZE_1 | NEBULA_BLAZE_2
                | POISON_DART | SPIRIT_FLAME | SKY_FRACTURE | BOOK_OF_SKULLS | SPARK
        ),
        // `appliesImmunityTimeOnSingleHits` (`:717` reset false): the
        // ported setters are the 1134 crystal shard (`:10643`), the
        // 167-170 fireworks (`:2568`, S1-P-i16) and the 286 Exploding
        // Bullet (`:3754`, S1-P-i18) — the id-static gate's single-hit
        // override (`:13969`/`:14116`) reads this flag.
        applies_immunity_time_on_single_hits: matches!(kind, CRYSTAL_SHARD_MELEE | EXPLODING_BULLET)
            || matches!(kind, 167..=170),
    })
}

// ===== M4-PROJ-8: the summon system constants =====
pub const PROJ_COUNT: usize = 1136;

pub const FROST_HYDRA: u16 = 308;
pub const FROST_BLAST_FRIENDLY: u16 = 309;
pub const HORNET: u16 = 373;
pub const FLYING_IMP: u16 = 375;
pub const RETANIMINI: u16 = 387;
pub const SPAZMAMINI: u16 = 388;
pub const MINI_RETINA_LASER: u16 = 389;
/// `ProjectileID.OneEyedPirate`/`SoulscourgePirate`/`PirateCaptain`
/// (`ProjectileID.cs:1340-1344`) — the Pirate Staff's aiStyle-67 trio.
pub const ONE_EYED_PIRATE: u16 = 393;
pub const SOULSCOURGE_PIRATE: u16 = 394;
pub const PIRATE_CAPTAIN: u16 = 395;
/// `ProjectileID.VampireFrog` (`ProjectileID.cs:2070`) — the aiStyle-67 frog.
pub const VAMPIRE_FROG: u16 = 758;
/// `ProjectileID.CrimsonHeart` (`ProjectileID.cs:1554`) — the aiStyle-67
/// pet, slice B (S1-D-4b-12).
pub const CRIMSON_HEART: u16 = 500;
/// `ProjectileID.DirtiestBlock` (`ProjectileID.cs:2590`).
pub const DIRTIEST_BLOCK: u16 = 1018;
/// `ProjectileID.BoulderPet` (`ProjectileID.cs:2666`).
pub const BOULDER_PET: u16 = 1056;

pub const DEADLY_SPHERE: u16 = 533;
pub const TEMPEST: u16 = 407;
/// `ProjectileID.SharknadoBolt` (385) — Duke Fishron's sharknado bolt
/// (`Projectile.cs:4676-4685`): 30x30, hostile, penetrate -1, aiStyle 65,
/// alpha 255, timeLeft 300. S1-D-3 W-9 fishron wiring.
pub const SHARKNADO_BOLT: u16 = 385;
/// `ProjectileID.ForceBubble` (384) — the small sharknado cloud the
/// unaimed bolt's death spawns (`Projectile.cs:4664-4675`): 150x42,
/// hostile, penetrate -1, aiStyle 64, tileCollide false, ignoreWater,
/// alpha 255, timeLeft 540. Its arm spawns the Sharkron 372.
pub const FORCE_BUBBLE: u16 = 384;
/// `ProjectileID.Cthulunado` (386) — the Greater Sharknado
/// (`Projectile.cs:4686-4697`): 150x42, hostile, penetrate -1, aiStyle 64,
/// tileCollide false, ignoreWater, alpha 255, timeLeft 840. Its aiStyle-64
/// arm spawns the Sharkron2 373 (`:29502-29516`).
pub const CTHULUNADO: u16 = 386;
pub const UFO_MINION: u16 = 423;
pub const STARDUST_CELL: u16 = 613;
pub const STARDUST_GUARDIAN: u16 = 623;
/// `ProjectileID.StardustDragon1` (625) — the chain HEAD (`ProjectileID.cs:1804`):
/// the aiStyle-121 steering half (`Projectile.cs:42862-42992`), a 0-slot summon.
pub const STARDUST_DRAGON_HEAD: u16 = 625;
/// `ProjectileID.StardustDragon2` (626) — the first BODY segment
/// (`ProjectileID.cs:1806`): 0.5 slots, the key-linked follow half.
pub const STARDUST_DRAGON_BODY_1: u16 = 626;
/// `ProjectileID.StardustDragon3` (627) — the second BODY segment
/// (`ProjectileID.cs:1808`): 0.5 slots, the key-linked follow half.
pub const STARDUST_DRAGON_BODY_2: u16 = 627;
/// `ProjectileID.StardustDragon4` (628) — the chain TAIL (`ProjectileID.cs:1810`):
/// 0 slots; its contact with the HEAD kills both (`Projectile.cs:43022-43027`).
pub const STARDUST_DRAGON_TAIL: u16 = 628;
pub const BABY_BIRD: u16 = 759;
pub const SLIME_MINION: u16 = 266;
/// `ProjectileID.SkeletronPet` (885) — AI_026 slice B (S1-D-4b-11): the
/// spinning Skeletron-head pet (`Projectile.cs:60092-60253`'s shared
/// 885/889 arm; buff 288's pet, `Player.cs:11105`).
pub const SKELETRON_PET: u16 = 885;
/// `ProjectileID.SkeletronPrimePet` (889) — AI_026 slice B: the spinning
/// Skeletron-Prime-head pet with the `localAI[0]` laser-eye accumulator
/// (buff 292's pet, `Player.cs:11121`).
pub const SKELETRON_PRIME_PET: u16 = 889;

// AI_026 slice A (S1-D-4b-10): the 14 early-return flyer pets of
// `AI_026` (`Projectile.cs:59120-60091`) — `ProjectileID.cs:904/948/950/
// 952/976/1314/2102/2184/2188/2326/2338/2342/2356/2646`. Everything else
// in the mega-family (the idle/walk/jump/attack machine, `:60092+`) is
// slice B (task/S1-D-4b-11-melee-b).
pub const BABY_EATER: u16 = 175;
pub const BABY_SKELETRON_HEAD: u16 = 197;
pub const BABY_HORNET: u16 = 198;
pub const TIKI_SPIRIT: u16 = 199;
pub const WISP: u16 = 211;
pub const ZEPHYR_FISH: u16 = 380;
pub const SHARK_PUP: u16 = 774;
pub const LIL_HARPY: u16 = 815;
pub const GLITTERY_BUTTERFLY: u16 = 817;
pub const QUEEN_BEE_PET: u16 = 886;
pub const DUKE_FISHRON_PET: u16 = 892;
pub const MOON_LORD_PET: u16 = 894;
pub const DD2_BETSY_PET: u16 = 901;
pub const PUFFERFISH: u16 = 1046;
pub const STORM_TIGER_1: u16 = 833;
pub const STORM_TIGER_2: u16 = 834;
pub const STORM_TIGER_3: u16 = 835;
pub const ABIGAIL: u16 = 963;
pub const QUEEN_SPIDER: u16 = 377;
pub const SPIDER_EGG: u16 = 378;
pub const MOONLORD_TURRET: u16 = 641;
/// `ProjectileID.MoonlordTurretLaser` (642) — the turret's beam
/// (`Projectile.cs:6904-6915`: 18x18, aiStyle 84, `penetrate = -1`,
/// `alpha = 255`, `tileCollide = false`, local-immunity 10).
pub const MOONLORD_TURRET_LASER: u16 = 642;
pub const RAINBOW_CRYSTAL: u16 = 643;
/// `ProjectileID.RainbowCrystalExplosion` (644) — the crystal's shard
/// (`Projectile.cs:6941-6952`: 14x14, aiStyle 112, `penetrate = 1`,
/// `timeLeft = 900`, `tileCollide = false`, `ignoreWater`, `alpha = 255`,
/// `usesIDStaticNPCImmunity` with the stock `idStaticNPCHitCooldown = -1`,
/// `Projectile.cs:466` — the block-forever sentinel).
pub const RAINBOW_CRYSTAL_EXPLOSION: u16 = 644;
pub const FLAME_BURST_TOWER_1: u16 = 663;
pub const FLAME_BURST_TOWER_2: u16 = 665;
pub const FLAME_BURST_TOWER_3: u16 = 667;
/// `ProjectileID.DD2FlameBurstTowerT1Shot` (664) — the Flame Burst Tower's
/// shot (`Projectile.cs:7153-7162`: 16x16, aiStyle 131, `alpha = 255`,
/// local-immunity −1). S1-D-4b-9.
pub const FLAME_BURST_SHOT_1: u16 = 664;
/// `ProjectileID.DD2FlameBurstTowerT2Shot` (666).
pub const FLAME_BURST_SHOT_2: u16 = 666;
/// `ProjectileID.DD2FlameBurstTowerT3Shot` (668).
pub const FLAME_BURST_SHOT_3: u16 = 668;
pub const BALLISTRA_TOWER_1: u16 = 677;
pub const BALLISTRA_TOWER_2: u16 = 678;
pub const BALLISTRA_TOWER_3: u16 = 679;
/// `ProjectileID.BallistaShot` (680) — the Ballista's bolt
/// (`Projectile.cs:7253-7262`: 16x16, aiStyle 1, `penetrate = 4`,
/// local-immunity −1).
pub const BALLISTA_SHOT: u16 = 680;
pub const LIGHTNING_AURA_1: u16 = 688;
pub const LIGHTNING_AURA_2: u16 = 689;
pub const LIGHTNING_AURA_3: u16 = 690;
pub const EXPLOSIVE_TRAP_1: u16 = 691;
pub const EXPLOSIVE_TRAP_2: u16 = 692;
pub const EXPLOSIVE_TRAP_3: u16 = 693;
/// `ProjectileID.DD2ExplosiveTrapT1Explosion` (694) — the aiStyle-139 blast
/// the trap spawns (`Projectile.cs:69304`/`:69364`).
pub const EXPLOSIVE_TRAP_EXPLOSION_1: u16 = 694;
/// `ProjectileID.DD2ExplosiveTrapT2Explosion` (695).
pub const EXPLOSIVE_TRAP_EXPLOSION_2: u16 = 695;
/// `ProjectileID.DD2ExplosiveTrapT3Explosion` (696).
pub const EXPLOSIVE_TRAP_EXPLOSION_3: u16 = 696;
pub const HOUNDIUS_SHOOTIUS: u16 = 966;
pub const HOUNDIUS_SHOOTIUS_FIREBALL: u16 = 967;
pub const DEAD_CELLS_BARNACLE: u16 = 1025;
pub const DEAD_CELLS_BARNACLE_SHOT: u16 = 1026; // AI_197 follow-up

pub const MINION_SACRIFICABLE: &[u16] = &[
    191, 192, 193, 194, 266, 317, 373, 375, 387, 388, 390, 393, 407, 423, 533, 613, 755, 758, 759,
    831, 970, 864, 946, 951, 963, 1022, 1093, 1112, 1094, 1113, 1118, 1119, 625, 626, 627, 628,
];

/// `Main.projPet` (`Main.cs:8728-8852`): the projectile ids the game treats as
/// pets/minions for the use-path gates — `FreeUpPetsAndMinions`'s call gate
/// (`Player.cs:42778`) and the pet-buff shoot arm (`:48561`). Verbatim id list.
pub const PROJ_PET: &[u16] = &[
    492, 499, 653, 701, 703, 702, 764, 765, 319, 334, 324, 266, 313, 314, 317, 175, 111, 112, 127,
    191, 192, 193, 194, 197, 198, 199, 200, 208, 209, 210, 211, 236, 268, 269, 353, 373, 375, 380,
    387, 388, 390, 391, 392, 393, 394, 395, 1093, 1094, 1112, 1113, 1118, 1119, 398, 407, 423, 533,
    613, 623, 625, 626, 627, 628, 755, 758, 759, 774, 815, 816, 817, 821, 825, 831, 833, 834, 835,
    854, 858, 859, 860, 864, 875, 946, 951, 963, 970, 1022, 881, 882, 883, 884, 885, 886, 887, 888,
    889, 890, 891, 892, 893, 894, 895, 896, 897, 898, 899, 900, 901, 934, 956, 957, 958, 959, 960,
    994, 998, 1003, 1004, 1018, 1056, 1090, 1027, 1046, 1050, 1095, 1096,
];

/// `ProjectileID.Sets.LightPet` (`ProjectileID.cs:523`): the light-pet
/// projectile ids — the other half of `FreeUpPetsAndMinions`'s call gate
/// (`Player.cs:42778`, `LightPet[shoot] || projPet[shoot]`).
pub const LIGHT_PET_PROJ: &[u16] = &[18, 500, 72, 87, 86, 211, 492, 650, 702, 891, 896, 895];

/// `Player.ItemCheck_ApplyPetBuffs`' summon-staff rows (`Player.cs:53190-53281`):
/// the item types whose use grants `AddBuff(sItem.buffType, 3600)` at the
/// use-start (`:42766`), the minion-buff PERSISTENCE mechanism — the 3600-tick
/// grant is what the keep-alive arm's 18000 refresh then pins while the minion
/// lives (the `:42880` `item.buffTime` grant is 0 for the staves, a ghost).
/// Verbatim: the 1157/1309/1802/2364/2365 single-item arms (`:53190`/`:53194`/
/// `:53218`/`:53234`/`:53238`) plus the 24-case switch (`:53254-53277`).
pub const MINION_STAFF_PET_BUFF_ITEMS: &[u16] = &[
    1157, 1309, 1802, 2364, 2365, 2535, 2551, 2584, 2621, 2749, 3249, 3474, 3531, 4269, 4273,
    4281, 4607, 4758, 5005, 5069, 5114, 5456, 5663, 5664, 6148, 6149, 6161, 6164,
];

/// `ProjectileID.Sets.MinionShot` (`ProjectileID.cs:431`): the minion attack
/// shot kinds — `WhipTagEffect.CanRunHitEffects`'s gate
/// (`GameContent/Items/WhipTagEffect.cs:81-85`) treats them as summon shots
/// alongside the `minion`/`sentry` flags (S1-P-i24/S1-P-i25 landed the same
/// table; gate-merged to one definition).
pub const MINION_SHOT: &[u16] = &[374, 376, 389, 195, 408, 433, 614, 818, 1044, 1097, 1106, 1120];

pub const SENTRY_SHOT: &[u16] = &[
    680, 664, 666, 668, 694, 695, 696, 644, 642, 378, 379, 309, 967, 1026,
];

/// `WhipTagEffect.CanRunHitEffects` (`GameContent/Items/WhipTagEffect.cs:75-86`):
/// `optionalProjectile.OwnedBySomeone && (minion || MinionShot[type] ||
/// sentry || SentryShot[type])`. The owned check is the caller's (`owner !=
/// 255`); this folds the four-way shot test the tag-damage math gates on.
pub fn can_run_whip_hit_effects(minion: bool, sentry: bool, kind: u16) -> bool {
    minion || sentry || MINION_SHOT.contains(&kind) || SENTRY_SHOT.contains(&kind)
}

/// `ItemID.Sets.SummonerWeaponThatScalesWithAttackSpeed` (`ID/ItemID.cs:497`):
/// the 18 whip items — `ApplyItemAnimation` (`Player.cs:4474-4476`) scales
/// their swing by `summonerWeaponSpeedBonus * whipUseTimeMultiplier`.
pub const SUMMONER_WEAPON_SCALES_WITH_ATTACK_SPEED: &[u16] = &[
    4672, 4679, 4680, 4678, 4913, 4912, 4911, 4914, 5074, 5473, 5474, 5475, 5476, 5477, 5478,
    5479, 5480, 5688,
];

pub const MINION_TARGETING_FEATURE: &[u16] = &[
    191, 192, 193, 194, 266, 317, 373, 375, 387, 388, 390, 393, 407, 423, 533, 613, 625, 755,
    758, 759, 831, 833, 834, 835, 864, 946, 951, 963, 970, 1022, 1093, 1094, 1112, 1113, 1118,
    1119, 377, 308, 643, 641, 663, 665, 667, 677, 678, 679, 688, 689, 690, 691, 692, 693, 966,
    1025,
];

/// `Main.projFrames` (`Main.cs:1223` — the array `Initialize_TileAndNPCData1`
/// fills with 1 for every id at `Main.cs:8441-8443` and overrides per kind at
/// `:8444+`): the vertical frame count the draw slices `Projectile_<id>.xnb`
/// by. The slice carries the animated followup kinds, the Nebula Blaze pair
/// and the gem bolt's trail count; the 1.4.5.8 overrides outside the ported
/// rows (72/86/87/102/111/112/127/175/…, `Main.cs:8444-8722`) default here —
/// every id the slice omits reads 1, as the source's init sets.
pub(crate) const fn proj_frames(kind: u16) -> u32 {
    // The C#'s `Main.projFrames` table (`Main.cs:8434-8727`, defaulted to 1 by
    // the loop at `:8439-8442`) — the whole row set the atlas packs, so the
    // frame slicing covers every animated sheet, not only this task's kinds.
    match kind {
        34 => 6,
        102 => 2,
        206 => 5,
        254 => 5,
        436 => 4,
        447 => 4,
        448 => 3,
        450 => 5,
        454 => 2,
        456 => 4,
        462 => 5,
        519 => 4,
        574 => 2,
        601 => 2,
        645 => 7,
        652 => 6,
        659 => 4,
        837 => 3,
        1130 => 6,
        // The Flamelash's 6-frame sheet (`Main.cs:8454`). Its AI's
        // `frameCounter++`/`frame` writes (`Projectile.cs:43216-43222`) are a
        // Phase-6 documented skip, so `frame` stays 0 and the slice reads the
        // top row; the trail Phase 7 draws does not key on it.
        LEAF => 5,
        // The Typhoon's 3-frame sheet (`Main.cs:8524`) —
        // `localAI[1] / 3 % 3` slices it.
        TYPHOON => 3,
        // The Last Prism holder's 5-frame sheet (`Main.cs:8498`, rate 4 → 1).
        LAST_PRISM => 5,
        // The Nebula Blaze pair (`Main.cs:8552-8553`). The animating AI_001
        // hook (`Projectile.cs:54561-54569`) is unported, so `frame` stays 0.
        NEBULA_BLAZE_1 | NEBULA_BLAZE_2 => 4,
        // The baby bird's 5-frame sheet (`Main.cs:8506`) — `AI_158`'s
        // `frame == projFrames - 1` perch test reads it (`:48773`, `:12480`).
        BABY_BIRD => 5,
        // AI_026 slice B: the boss-head pets' sheets (`Main.cs:8673`
        // 885 → 10, `:8677` 889 → 11) — the spin arm pins frame 6
        // (`Projectile.cs:60192`) and the walk steps frames 0-5
        // (`:60240-60247`).
        SKELETRON_PET => 10,
        SKELETRON_PRIME_PET => 11,
        // AI_026 slice C: the slime minion's 12-frame sheet
        // (`Main.cs:8612`) — the fly-out frames 8-11
        // (`Projectile.cs:61209`) and the ground frames 0-7
        // (`:64445`/`:64470`) slice it.
        SLIME_MINION => 12,
        // The pirate trio's 15-frame sheet (`Main.cs:8518-8520`) — AI_067's
        // walk (0-3), bite (4-9) and flight (10-14) machines index it.
        ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN => 15,
        // The Vampire Frog's 24-frame sheet (`Main.cs:8631`).
        VAMPIRE_FROG => 24,
        // The Stardust Guardian's 19-frame sheet (`Main.cs:8497`) —
        // AI_120's idle machine wraps at 8 (`Projectile.cs:44693`), the
        // rest-hover at projFrames − 4 = 15 (`:44830`) and the dash at
        // 19 → 14 (`:44852-44855`). S1-P-i56.
        STARDUST_GUARDIAN => 19,
        // The Crimson Heart's 4-frame sheet (`Main.cs:8484`) — the pet's
        // 6/12-tick pulse wraps at it (`Projectile.cs:49473`). The 1018/1056
        // keep the init default 1 (no override in `Main.cs:8444-8727`).
        CRIMSON_HEART => 4,

        // The Explosive Trap family (`Main.cs:8467-8472`): the traps and the
        // T1/T2 blasts animate 4 frames, the T3 blast 5 — `AI_138`'s wrap
        // (`Projectile.cs:69336`) and `AI_139`'s kill test (`:69406`) read it.
        EXPLOSIVE_TRAP_1 | EXPLOSIVE_TRAP_2 | EXPLOSIVE_TRAP_3
        | EXPLOSIVE_TRAP_EXPLOSION_1 | EXPLOSIVE_TRAP_EXPLOSION_2 => 4,
        EXPLOSIVE_TRAP_EXPLOSION_3 => 5,
        // The Flame Burst towers' sheets (`Main.cs:8458-8460`): T1 animates 7
        // frames, T2/T3 9 — `AI_130`'s fire frame (`Projectile.cs:68331`,
        // `1 + ai[1] / 4`) and its idle frame 0 (`:68307`) slice them.
        FLAME_BURST_TOWER_1 => 7,
        FLAME_BURST_TOWER_2 | FLAME_BURST_TOWER_3 => 9,
        // The Ballista towers' 6-frame sheet (`Main.cs:8461-8463`) — `AI_134`'s
        // reload frame (`Projectile.cs:68612`, `1 + ai[1] / 5`) and its idle
        // frame 0 (`:68561`) read it.
        BALLISTRA_TOWER_1 | BALLISTRA_TOWER_2 | BALLISTRA_TOWER_3 => 6,
        // The Lightning Auras' sheets (`Main.cs:8464-8466`): the T1/T2 auras
        // animate 6 frames, the T3 aura 8 — `AI_137`'s 8-tick frame machine
        // (`Projectile.cs:68948-68955`) wraps at `projFrames`.
        LIGHTNING_AURA_1 | LIGHTNING_AURA_2 => 6,
        LIGHTNING_AURA_3 => 8,
        // The dungeon-debris trio's 3-frame sheets (`Main.cs:8445-8447`) —
        // `AI_010`'s first-tick frame roll (`Projectile.cs:57946`) picks the
        // row and the per-frame rotation (`:58073-58086`) spins it.
        BLUE_DUNGEON_DEBRIS | GREEN_DUNGEON_DEBRIS | PINK_DUNGEON_DEBRIS => 3,
        // AI_026 slice A (S1-D-4b-10) — the flyer pets' sheets
        // (`Main.cs:8585/8593/8594/8600/8623/8637/8639/8641/8674/8680/
        // 8682/8689/8718`). The 197 has no override anywhere in the table —
        // the init loop's 1 (`Main.cs:8441-8443`) stands; its arm never
        // reads `frame` (the spin rides `rotation`).
        BABY_EATER => 2,
        BABY_HORNET => 4,
        TIKI_SPIRIT => 8,
        WISP => 10,
        ZEPHYR_FISH => 4,
        SHARK_PUP => 8,
        LIL_HARPY => 10,
        GLITTERY_BUTTERFLY => 18,
        QUEEN_BEE_PET => 8,
        DUKE_FISHRON_PET => 6,
        MOON_LORD_PET => 8,
        DD2_BETSY_PET => 12,
        PUFFERFISH => 12,
        // The Rainbow Crystal's 8-frame sheet (`Main.cs:8476`) — `AI_123`'s
        // 3-tick frame machine (`Projectile.cs:37113-37120`) wraps at it.
        // The turret 641 has no `projFrames` row (the stock 1).
        RAINBOW_CRYSTAL => 8,
        // S1-D-3 W-9 fishron wiring — the sharknado family's strips
        // (`Main.cs:8512-8514`): the clouds 6 rows, the bolt 3.
        FORCE_BUBBLE | CTHULUNADO => 6,
        SHARKNADO_BOLT => 3,
        // The glow-stick's 4-frame sheet (`Main.cs:8664`) — the aiStyle-2
        // arm's 5-tick frame machine (`Projectile.cs:23477-23486`) wraps at
        // it, back to frame 1.
        861 => 4,
        // The 909's 6-frame sheet (`Main.cs:8691`) — the arm's
        // `1 + rand.Next(6)` frame roll (`Projectile.cs:23268-23274`) pins
        // one of its rows.
        909 => 6,
        // The 968's 24-frame sheet (`Main.cs:8706`) — the arm's
        // `frame = (int)ai[1]` pin (`Projectile.cs:23238`) reads it.
        968 => 24,
        // The Frost Moon family (S1-D-4): the present's 2-frame sheet
        // (`Main.cs:8529`) — AI()'s aiStyle-58 arm pins frame 0/1
        // (`Projectile.cs:28915`/`:28928`); the frost shard's 5-frame sheet
        // (`Main.cs:8530`) — the 349 hook's `frame = (int)ai[0]` pin
        // (`Projectile.cs:56223`); the ornament's 2-frame sheet
        // (`Main.cs:8565`) — the 346 arm's `frame = (int)ai[1]` pin
        // (`Projectile.cs:24874`, the `NPC.cs:32885` `Next(2)` payload); the
        // shrapnel's 2-frame sheet (`Main.cs:8566`, no AI frame writer — the
        // stock frame 0 reads the top row).
        SANTA_PRESENT => 2,
        FROST_SHARD => 5,
        ORNAMENT_HOSTILE => 2,
        ORNAMENT_SHRAPNEL => 2,
        // S1-P-i21 — the hardmode staves' sheets (`Main.cs:8569`/`:8573`/
        // `:8602-8603`): the Blizzard's 5 icicle rows (`frame = (int)ai[1]`,
        // `Projectile.cs:56948`), the bat's 4-frame flap (the aiStyle-36 arm
        // wraps at 3, `:27362`), the moving cloud's 4-frame drift and the
        // raining cloud's 6-frame sheet (the aiStyle-45 arm's frame machine,
        // `:27976-27985`/`:27996-28005`).
        BLIZZARD => 5,
        BAT_FRIENDLY => 4,
        RAIN_CLOUD_MOVING => 4,
        RAIN_CLOUD_RAINING => 6,
        // The spear family's sub-shots (S1-P-i06): the spore cloud's 5-frame
        // sheet (`Main.cs:8605`) — the aiStyle-44 frame machine
        // (`Projectile.cs:27941-27948`) wraps at it; the Thunder bolt's
        // 4-frame sheet (`Main.cs:8629`); the North Pole flake's 3-frame
        // sheet (`Main.cs:8568`) — the 344 hook's `frame = (int)ai[1]` pin
        // (`Projectile.cs:57078`).
        CHLOROPHYTE_SPORE_CLOUD => 5,
        THUNDER_SPEAR_BOLT => 4,
        NORTH_POLE_FLAKE => 3,
        // S1-P-i30 — the Butcher's Chainsaw (509): the 2-frame sheet
        // (`Main.cs:8481`), the aiStyle-20 machine's `frameCounter >= 2`
        // wrap (`Projectile.cs:25497-25509`).
        CHAINSAW_BUTCHERS => 2,
        // S1-P-i42 — the fairy glowstick's 4-frame sheet (`Main.cs:8667`) —
        // the aiStyle-170 hover's frame machine (`Projectile.cs:45676-45685`)
        // wraps at it. The other glowsticks/flares keep the stock 1 (no
        // `projFrames` override in `Main.cs:8444-8727`).
        FAIRY_GLOWSTICK => 4,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_damage_class_flags_match_the_set_defaults_census() {
        // W3-i11 — the per-hit crit block's class flags (`Projectile.cs:751-
        // 753` reset false, the per-type `this.melee/ranged/magic = true`
        // writes + the DefaultTo* helpers). The whip sets NONE.
        let flags = |k: u16| {
            let d = proj_data(k).expect("a row");
            (d.melee, d.ranged, d.magic)
        };
        // The ranged shots: arrows/bullets, the thrown kinds, the moon-lord
        // drop shots.
        assert_eq!(flags(WOODEN_ARROW), (false, true, false));
        assert_eq!(flags(FLAME_ARROW), (false, true, false));
        assert_eq!(flags(MUSKET_BALL), (false, true, false));
        assert_eq!(flags(SHURIKEN), (false, true, false));
        assert_eq!(flags(THROWING_KNIFE), (false, true, false));
        assert_eq!(flags(SNOWBALL), (false, true, false));
        assert_eq!(flags(MOONLORD_BULLET), (false, true, false));
        // W3-i11 round 2 — the ranged census completion: the sand balls, the
        // Flames, the special arrows, the rockets and the Empress shot.
        for k in [42, 65, 68, 354, 85, 117, 469, 485, 495, 819, 615, 630, 705, 714, 932] {
            assert_eq!(flags(k), (false, true, false), "kind {k}");
        }
        for k in (133..=144).chain(338..=341).chain(715..=718) {
            assert_eq!(flags(k), (false, true, false), "kind {k}");
        }
        for k in (776..=801).chain(803..=810) {
            if k == 779 || k == 783 {
                continue; // row-less (no launcher resolves to them)
            }
            assert_eq!(flags(k), (false, true, false), "kind {k}");
        }
        // The hostile snowmen's shots carry the flag verbatim (owner 255
        // keeps it moot).
        assert_eq!(flags(SNOW_BALL_HOSTILE), (false, true, false));
        assert_eq!(flags(BULLET_SNOWMAN), (false, true, false));
        // The Clentaminator spray (145) and the acorn (1088) set NO class
        // flag (`:2338-2349`/`:10145-10152`) — the census stays exact.
        assert_eq!(flags(145), (false, false, false));
        assert_eq!(flags(1088), (false, false, false));
        // The magic spells.
        assert_eq!(flags(CRYSTAL_STORM), (false, false, true));
        assert_eq!(flags(SPARK), (false, false, true));
        assert_eq!(flags(GEM_BOLT_AMETHYST), (false, false, true));
        assert_eq!(flags(GEM_BOLT_AMBER), (false, false, true));
        assert_eq!(flags(NEBULA_BLAZE_1), (false, false, true));
        assert_eq!(flags(NEBULA_BLAZE_2), (false, false, true));
        assert_eq!(flags(SPIRIT_FLAME), (false, false, true));
        assert_eq!(flags(SKY_FRACTURE), (false, false, true));
        assert_eq!(flags(LAST_PRISM), (false, false, true));
        assert_eq!(flags(LEAF), (false, false, true));
        // The melee families: yoyos (DefaultToYoyo), flails (DefaultToFlail),
        // the boomerangs, the sword beams, the yoyo on-hit shots.
        assert_eq!(flags(WOOD_YOYO), (true, false, false));
        assert_eq!(flags(TERRARIAN_YOYO), (true, false, false));
        assert_eq!(flags(BALL_O_HURT), (true, false, false));
        assert_eq!(flags(ENCHANTED_BOOMERANG), (true, false, false));
        assert_eq!(flags(SWORD_BEAM), (true, false, false));
        assert_eq!(flags(TERRARIAN_BEAM), (true, false, false));
        assert_eq!(flags(CRYSTAL_SHARD_MELEE), (true, false, false));
        assert_eq!(flags(EYE_OF_CTHULHU_DASH), (true, false, false));
        // The whip sets NO class flag (DefaultToWhip `:10704-10718`).
        assert_eq!(flags(BLAND_WHIP), (false, false, false));
        assert_eq!(flags(SWORD_WHIP), (false, false, false));
        // The bombs set none (`:1106-1121`/`:1198-1206`/`:5889-5897`).
        assert_eq!(flags(BOMB), (false, false, false));
        assert_eq!(flags(DYNAMITE), (false, false, false));
        assert_eq!(flags(STICKY_BOMB), (false, false, false));
        assert_eq!(flags(BOMB_FISH), (false, false, false));
    }

    #[test]
    fn the_single_hit_immunity_override_is_the_crystal_shard_only() {
        // W3-i11 — `appliesImmunityTimeOnSingleHits` (`:717` reset false):
        // the only ported setter is the 1134 crystal shard (`:10643`).
        assert!(proj_data(CRYSTAL_SHARD_MELEE).unwrap().applies_immunity_time_on_single_hits);
        // The 644 shard burst keeps the stock false (its idStatic pair rides
        // the −1 cooldown, not the override).
        assert!(!proj_data(RAINBOW_CRYSTAL_EXPLOSION).unwrap().applies_immunity_time_on_single_hits);
        assert!(!proj_data(WOODEN_ARROW).unwrap().applies_immunity_time_on_single_hits);
        assert!(!proj_data(BLAND_WHIP).unwrap().applies_immunity_time_on_single_hits);
    }
}
