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
/// The Destroyer's segment laser (`NPC.cs`'s `num11 = 100`, `:49886`).
pub const DESTROYER_LASER: u16 = 100;
/// `ProjectileID.SnowBallHostile` (109) — the Mystery Snowman's aimed
/// throw (`aiStyle 10`, `Projectile.cs` type 109's SetDefaults).
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
/// `ProjectileID.EyeLaser` (83) — the Retinazer's phase-1 laser
/// (`NPC.cs:26628`'s `num412`).
pub const EYE_LASER: u16 = 83;
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
/// `ProjectileID.DeerclopsIceChunk` (962, `Projectile.cs:9199-9206`) — the
/// Deerclops' rubble chunk (the aiStyle-1 flight).
pub const DEERCLOPS_ICE_CHUNK: u16 = 962;
/// `ProjectileID.FlamingWood` (325, `Projectile.cs:3249-3259`) — the
/// Retinazer's shot.
pub const FLAMING_WOOD: u16 = 325;
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
/// `ProjectileID.LeafBlower` (`Projectile.cs:2937-2945`): the Leaf Blower's
/// 14x14 petal, aiStyle 40 (`AI_040_LeafBlower` `:38003-38082`).
pub const LEAF_BLOWER: u16 = 206;

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
/// `ProjectileID.LightsBane` (974) — the Light's Bane sword's slash wave
/// (`Projectile.cs:9237-9252`'s SetDefaults, `AI_188_LightsBane`
/// `:42229-42262`, `Main.DrawProj_LightsBane` `Main.cs:27455-27495`).
pub const LIGHTS_BANE: u16 = 974;

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
        ENCHANTED_BOOMERANG => 0.4,            // `:872`
        MUSKET_BALL => 0.5,                    // `:957`
        BULLET_SNOWMAN => 0.5,                 // `:1985`
        BALL_OF_FIRE => 0.8,                   // `:970`
        MAGIC_MISSILE => 0.8,                  // `:980`
        GREEN_LASER => 0.75,                   // `:1025`
        FLAMELASH => 0.8,                      // `:1168`
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
        _ => 0.0,
    }
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
            // `Projectile.cs:828-836` — the shuriken (3): the 22x22 box, aiStyle 2,
            // `penetrate = 4`, the stock 3600 life and the stock `tileCollide`.
            SHURIKEN => (22.0, 22.0, 2, 4, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:1310-1316` — the throwing knife (48): 12x12, aiStyle
            // 2, `penetrate = 2`.
            THROWING_KNIFE => (12.0, 12.0, 2, 2, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:2549-2557` — the snowball (166): 14x14, aiStyle 2.
            SNOWBALL => (14.0, 14.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
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
            // `Projectile.cs:1978-1991` — the Snowman Gangsta's bullet (110):
            // 4x4, aiStyle 1, hostile, `penetrate = -1`, `light = 0.5f`,
            // alpha 255, `scale = 1.2f`, `timeLeft = 600`, `extraUpdates = 1`
            // (S1-E-5: the Frost Legion throw requests it, `npc.rs`'s Gangsta
            // arm; the row lets the data-carrying spawn land it).
            BULLET_SNOWMAN => (4.0, 4.0, 1, -1, 600, 1, true, false, false, (false, -2, false, -1, 255, 1.2)),
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
            // `:2937-2945` Leaf — 14x14, aiStyle 40, `penetrate = 1`, alpha
            // 255, `timeLeft = 360`.
            LEAF => (14.0, 14.0, 40, 1, 360, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
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
            // `Projectile.cs:3516-3525` (266) — the Slime Staff's minion,
            // aiStyle 26 (the hop family, unported — the dispatch no-ops like
            // the other unported styles), 24x16, `penetrate = -1`,
            // `timeLeft *= 5` and the idStatic-12 pair.
            SLIME_MINION => (24.0, 16.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
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
            // docs union point 5). The aiStyle-194 flight
            // (`AI_194_HorsemanPumpkin`, `:41293`) is a follow-up.
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
        SLIME_MINION => (true, 12),
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
            // The slice-B pets set `netImportant` themselves
            // (`Projectile.cs:5419`/`:9643`/`:9993`).
            || matches!(kind, CRIMSON_HEART | DIRTIEST_BLOCK | BOULDER_PET)
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
            ),
        ignore_water: ((minion || sentry)
            && !matches!(
                kind,
                STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3
                    // The aiStyle-67 pirate/frog rows set neither
                    // ignoreWater (`Projectile.cs:4755-4768`/`:7814-7828`).
                    | ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN | VAMPIRE_FROG
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
            || matches!(kind, FORCE_BUBBLE | CTHULUNADO),
        // The aiStyle-67 summon rows set `decidesManualFallThrough`
        // (`:4767`/`:7825`/`:8092`/`:8106`/`:8120`); the pirates/frog's arm
        // drives `shouldFallThrough` (`:49561`/`:51206`).
        decides_manual_fall_through: matches!(
            kind,
            ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN | VAMPIRE_FROG
                | STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3
        ),
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

pub const SENTRY_SHOT: &[u16] = &[
    680, 664, 666, 668, 694, 695, 696, 644, 642, 378, 379, 309, 967, 1026,
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
        // The pirate trio's 15-frame sheet (`Main.cs:8518-8520`) — AI_067's
        // walk (0-3), bite (4-9) and flight (10-14) machines index it.
        ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN => 15,
        // The Vampire Frog's 24-frame sheet (`Main.cs:8631`).
        VAMPIRE_FROG => 24,
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
        _ => 1,
    }
}
