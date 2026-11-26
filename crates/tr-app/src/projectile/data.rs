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
/// `ProjectileID.TorchGod` (949) — the Torch God event's snuffed-torch
/// shot (`Projectile.cs:8948-8955`, W4-f33).
pub const TORCH_GOD: u16 = 949;
pub const SHURIKEN: u16 = 3;
/// `ProjectileID.VilethornBase` (7) — the Vilethorn's first segment
/// (`Projectile.cs:874-884`).
pub const VILE_BASE: u16 = 7;
/// `ProjectileID.VilethornTip` (8) — the Vilethorn's last segment, the type
/// the base chains into once `ai[1] >= 6` (`Projectile.cs:23737-23741`).
pub const VILE_TIP: u16 = 8;
/// `ProjectileID.CrystalVileShardHead` (493, `ProjectileID.cs:1540`) — the
/// Crystal Vile Shard's head segment (aiStyle 4, the Vilethorn family's
/// chain; `Projectile.cs:5667-5680`). The head spawns nothing; the shaft
/// chains into it at depth ≥ `7 + Next(2)` (`:23746-23760`) with a 1.25×
/// damage/knockBack bonus.
pub const CRYSTAL_VILE_HEAD: u16 = 493;
/// `ProjectileID.CrystalVileShardShaft` (494, `ProjectileID.cs:1542`) — the
/// Crystal Vile Shard's chaining segment (`Projectile.cs:5667-5680` shares
/// the head's row; the `:23746` chain arm is the shaft's alone).
pub const CRYSTAL_VILE_SHAFT: u16 = 494;
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
/// `ProjectileID.PurificationPowder` (10) — the aiStyle-6 purify cloud
/// (`Projectile.cs:897-912`): the 64×64 box, the tile-scan `Convert(…, 11, 1)`
/// (`:24193`).
pub const PURIFICATION_POWDER: u16 = 10;
/// `ProjectileID.VilePowder` (11) — the corruption twin of the powder
/// (`Projectile.cs:913-923`, `Convert(…, 1, 1)` `:24197`).
pub const VILE_POWDER: u16 = 11;
/// `ProjectileID.ViciousPowder` (463) — the crimson twin of the powder
/// (`Projectile.cs:5340-5350`, `Convert(…, 4, 1)` `:24201`; the dust type
/// remaps 10+463 → 231, `:24115-24118`).
pub const VICIOUS_POWDER: u16 = 463;
/// `ProjectileID.Fertilizer` (1019) — the aiStyle-6 sapling-grow cloud
/// (`Projectile.cs:9648-9658`): no conversion, the `flag2` tree-growth arm
/// (`:24203-24220`) and the slow-velocity kill (`:24224-24227`).
pub const FERTILIZER: u16 = 1019;
/// `ProjectileID.SuperFertilizer` (1051) — the fertilizer's `flag` twin
/// (`Projectile.cs:9967-9977`): the 50-dust burst with the per-dust
/// `Next(3)` type pick (`:24124-24133`) and the +15 tree-height addon
/// (`:24208-24211`).
pub const SUPER_FERTILIZER: u16 = 1051;
/// The aiStyle-6 powder/fertilizer family — the style's five rowed kinds
/// (`Projectile.cs:24100-24228`'s inline arm; W4-d1).
pub const POWDER_KINDS: [u16; 5] =
    [PURIFICATION_POWDER, VILE_POWDER, VICIOUS_POWDER, FERTILIZER, SUPER_FERTILIZER];
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
/// `ProjectileID.Ale` (669) — the Ale Throwing Glove's (item 3821) thrown
/// mug, aiStyle 68 (`Projectile.cs:7163-7173`), W4-d7 wiring (F4). Shares the
/// molotov's spin/fade/gravity/prime flight; its own `flag35` dust branch,
/// Tipsy debuff and death burst differ.
pub const ALE: u16 = 669;
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
/// `ProjectileID.Meowmere` (502, `ProjectileID.cs:1558`) — the Meowmere
/// sword's bouncing star: aiStyle 8 with its OWN kind-first bounce arm
/// (`Projectile.cs:18206-18254`, the five-bounce chime ladder the d75
/// part-4 owed gate serves), `penetrate = 5`, melee, local immunity 10
/// (`:5732-5742`). S1-E-21h d75 part 5 — the row that arms the gate.
pub const MEOWMERE: u16 = 502;
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
// W4-d2 — the aiStyle-11 light pets (`ProjectileID.cs:590/698/726/728`).
/// `ProjectileID.ShadowOrb` (18) — the Shadow Orb light pet, aiStyle 11
/// (`Projectile.cs:993-1007`).
pub const SHADOW_ORB: u16 = 18;
/// `ProjectileID.BlueFairy` (72) — aiStyle 11 (`Projectile.cs:1522-1535`).
pub const BLUE_FAIRY: u16 = 72;
/// `ProjectileID.PinkFairy` (86) — aiStyle 11 (`Projectile.cs:1674-1687`);
/// the C# player flag is `redFairy` (`Player.cs:2064`).
pub const PINK_FAIRY: u16 = 86;
/// `ProjectileID.GreenFairy` (87) — aiStyle 11 (`Projectile.cs:1688-1701`).
pub const GREEN_FAIRY: u16 = 87;
// W4-d2 — the aiStyle-13 chain weapons (`ProjectileID.cs:600/1078/1096/
// 1100/1516`).
/// `ProjectileID.Harpoon` (23) — aiStyle 13 (`Projectile.cs:1057-1068`).
pub const HARPOON: u16 = 23;
/// `ProjectileID.GolemFist` (262) — aiStyle 13 (`Projectile.cs:3463-3473`).
pub const GOLEM_FIST: u16 = 262;
/// `ProjectileID.BoxingGlove` (271) — aiStyle 13 (`Projectile.cs:3567-3577`).
pub const BOXING_GLOVE: u16 = 271;
/// `ProjectileID.ChainKnife` (273) — aiStyle 13 (`Projectile.cs:3588-3597`).
pub const CHAIN_KNIFE: u16 = 273;
/// `ProjectileID.ChainGuillotine` (481) — aiStyle 13
/// (`Projectile.cs:5561-5571`).
pub const CHAIN_GUILLOTINE: u16 = 481;
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
/// `ProjectileID.DesertDjinnCurse` (596, `ProjectileID.cs`) — the Djinn's
/// curse volley (`NPC.cs:21155`, W4-g5): 8×8, aiStyle 107, hostile,
/// alpha 255, ignoreWater, timeLeft 180, tileCollide false
/// (`Projectile.cs:6450-6460`). The aiStyle-107 flight + the Kill explosion
/// land with the projectile parity phase.
pub const DJINN_CURSE: u16 = 596;
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
/// `ProjectileID.FallingStar` (12, `ProjectileID.cs:578`) — the natural
/// night-sky falling star (aiStyle 5, `Projectile.cs:924-938`): 18×18,
/// penetrate −1, alpha 50, light 1, trap; the remix seed turns it hostile
/// (`:929-932`). Distinct from the Holy Arrow's death spawn 92.
pub const FALLING_STAR_NATURAL: u16 = 12;
/// `ProjectileID.StarWrath` (503, `ProjectileID.cs:1560`) — the Star Wrath's
/// falling star (aiStyle 5, `Projectile.cs:5743-5756`): 24×24, penetrate 2,
/// alpha 255, tileCollide false, melee, extraUpdates 1, local immunity 15.
pub const STAR_WRATH: u16 = 503;
/// `ProjectileID.ManaCloakStar` (723, `ProjectileID.cs:2000`) — the Mana
/// Cloak's retaliatory star (aiStyle 5, `Projectile.cs:7646-7658` shares the
/// 724/725/726/1133 row): 24×24, penetrate 2, alpha 50, tileCollide false,
/// local immunity 10, armorPenetration 25.
pub const MANA_CLOAK_STAR: u16 = 723;
/// `ProjectileID.BeeCloakStar` (724, `ProjectileID.cs:2002`) — the Bee Cloak's
/// star (the shared `:7646-7658` row).
pub const BEE_CLOAK_STAR: u16 = 724;
/// `ProjectileID.StarVeilStar` (725, `ProjectileID.cs:2004`) — the Star Veil's
/// star (the shared `:7646-7658` row).
pub const STAR_VEIL_STAR: u16 = 725;
/// `ProjectileID.StarCloakStar` (726, `ProjectileID.cs:2006`) — the Star
/// Cloak's star (the shared `:7646-7658` row).
pub const STAR_CLOAK_STAR: u16 = 726;
/// `ProjectileID.MeteorWhipMeteor` (1037, `ProjectileID.cs:2628`) — the
/// Meteor Whip's meteor (aiStyle 5, `Projectile.cs:9780-9792`): 24×24,
/// scale 0.8, extraUpdates 1, alpha 254, tileCollide false, local immunity −1.
pub const METEOR_WHIP_METEOR: u16 = 1037;
/// `ProjectileID.MeteorStormMeteor` (1049, `ProjectileID.cs:2652`) — the
/// Meteor Staff's meteor (aiStyle 5, `Projectile.cs:9938-9952`): 24×24,
/// hostile + friendly, scale 0.8, timeLeft 1200, extraUpdates 1, alpha 254,
/// penetrate −1, tileCollide false, trap.
pub const METEOR_STORM_METEOR: u16 = 1049;
/// `ProjectileID.SnakeBandStarCloakTinkerStar` (1133, `ProjectileID.cs:2820`)
/// — the Snake Band + Star Cloak tinker's star (the shared `:7646-7658` row).
pub const SNAKE_BAND_STAR: u16 = 1133;
/// The aiStyle-5 cloak-star family — the five kinds that share the
/// `Projectile.cs:7646-7658` `SetDefaults` arm (W4-d1).
pub const CLOAK_STAR_KINDS: [u16; 5] =
    [MANA_CLOAK_STAR, BEE_CLOAK_STAR, STAR_VEIL_STAR, STAR_CLOAK_STAR, SNAKE_BAND_STAR];
/// `ProjectileID.FallingStar` (12) — the natural night starfall's star
/// (`Projectile.cs:924-938`, aiStyle 5; W4-g70 lane 8). Distinct from 92:
/// 18x18, `trap = true`, `light = 1f`, the day-kill at damage 1000.
pub const NATURAL_FALLING_STAR: u16 = 12;
/// `ProjectileID.FallingStarSpawner` (720) — the invisible spawner the
/// starfall roll spawns (`Projectile.cs:7617-7625`, aiStyle 148); its
/// `AI_148_StarSpawner` (`:53094-53128`) calls `Star.StarFall` once and
/// converts into the 12 after 180 ticks.
pub const FALLING_STAR_SPAWNER: u16 = 720;
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
/// `ProjectileID.Stake` (`ProjectileID.cs:1200`) — the Stake Launcher's
/// shot (W4-f20 slice D, `Projectile.cs:4160-4171`).
pub const STAKE: u16 = 323;
/// `ProjectileID.PulseBolt` (`ProjectileID.cs:1268`) — the Pulse Bow's
/// override shot (W4-f20 slice D, `Projectile.cs:4474-4486`).
pub const PULSE_BOLT: u16 = 357;
/// `ProjectileID.Hellwing` (`ProjectileID.cs:1524`) — the Hellwing Bow's
/// homing bat (W4-f20 slice D, `Projectile.cs:5604-5615`).
pub const HELLWING: u16 = 485;
/// `ProjectileID.DD2BetsyArrow` (`ProjectileID.cs:1974`) — the Betsy Bow's
/// shot (W4-f20 slice D, `Projectile.cs:7521-7535`).
pub const BETSY_ARROW: u16 = 710;
/// `ProjectileID.ShimmerArrow` (`ProjectileID.cs:2566`).
pub const SHIMMER_ARROW: u16 = 1006;
/// `ProjectileID.DesertDjinnCurse` (596, `ProjectileID.cs:1746`) — the
/// Djinn's curse volley (`NPC.cs:21155`), aiStyle 107
/// (`Projectile.cs:6450-6460`: 8×8, hostile, alpha 255, ignoreWater,
/// timeLeft 180, tileCollide false).
pub const DESERT_DJINN_CURSE: u16 = 596;
/// `ProjectileID.SandBallFalling` (31, `ProjectileID.cs:616`) — the
/// Antlion's spit (`NPC.cs:24462`/`:24477`), aiStyle 10
/// (`Projectile.cs:1131-1140`: 10×10, knockBack 6, friendly + hostile,
/// penetrate −1). The Antlion's call site overrides damage 10, ai0 = 2,
/// timeLeft 300, friendly false.
pub const SAND_BALL_FALLING: u16 = 31;
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
/// `ProjectileID.BirdDroppings` (1080, `ProjectileID.cs:2714`) — the
/// aiStyle-24/68 birds' poop (`NPC.cs:25322`/`:34632`, zero
/// velocity/damage/knockback; `Projectile.cs:10078-10086`: 10x10,
/// aiStyle 200, alpha 255, penetrate -1, ignoreWater). W4-c10.
pub const BIRD_DROPPINGS: u16 = 1080;
/// `ProjectileID.Stinger` (55, `ProjectileID.cs`) — the Hornet's shot
/// (`NPC.cs:50796`'s type 55, dmg `(int)(10*scale)`, the post-spawn
/// `timeLeft = 300` `:50797`; `Projectile.cs:1375-1383`: 10×10, hostile,
/// aiStyle 1, penetrate −1, tileCollide; the `flag3 = false` flight list
/// `:55954` keeps the counter at rest — dead-straight, no gravity).
pub const HORNET_STINGER: u16 = 55;
/// `ProjectileID.SalamanderSpit` (572, `ProjectileID.cs`'s 572) — the
/// salamanders' shooter-block shot (`NPC.cs:59454`'s `num170 = 572`,
/// dmg 14 `:59455`, speed 7 `:59277`). W4-d41.
pub const SALAMANDER_SPIT: u16 = 572;
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
/// `ProjectileID.MartianTurretBolt` (435) — the Martian Turret's bolt
/// (`Projectile.cs:5056-5063`: 10x10, aiStyle 1, hostile, ignoreWater;
/// the aiStyle-73 shot request, `npc_ai_style73::TurretBolt`).
pub const MARTIAN_TURRET_BOLT: u16 = 435;
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
/// `ProjectileID.ScutlixLaser` (606) — the Scutlix mount's eye lasers
/// (`Projectile.cs:6530-6541`: 5×5, aiStyle 1, friendly, alpha 255,
/// extraUpdates 2, timeLeft 600, ignoreWater). M2-MOUNT-4.
pub const SCUTLIX_LASER: u16 = 606;
/// `ProjectileID.SantankMountRocket` (930) — the Santank mount's gift-box
/// rocket (`Projectile.cs:8817-8825`: 14×14, aiStyle 16, friendly,
/// penetrate -1, ranged). M2-MOUNT-4.
pub const SANTANK_MOUNT_ROCKET: u16 = 930;
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
/// `ProjectileID.MedusaHeadRay` (536) — the Medusa head's gaze ray,
/// aiStyle 101 (`Projectile.cs:6107-6119`: 10x10, friendly, penetrate
/// -1, tileCollide false, magic, ignoreWater). W4-d11.
pub const MEDUSA_HEAD_RAY: u16 = 536;
/// `ProjectileID.MinecartMechLaser` (591) — the mechanical minecart's
/// laser, aiStyle 101 (`Projectile.cs:6397-6408`: 8x8, friendly,
/// penetrate -1, tileCollide false, ignoreWater). W4-d11.
pub const MINECART_MECH_LASER: u16 = 591;
/// `ProjectileID.StardustJellyfishSmall` (539) — the stardust pillar's
/// small jellyfish orb, aiStyle 102 (`Projectile.cs:6142-6150`: 18x30,
/// hostile, penetrate -1, timeLeft 600). W4-d11.
pub const STARDUST_JELLYFISH_SMALL: u16 = 539;
/// `ProjectileID.NebulaBolt` (573) — the nebula soldier's bolt orb,
/// aiStyle 102 (`Projectile.cs:6230-6238`: 18x30, hostile, penetrate -1,
/// timeLeft 600). W4-d11.
pub const NEBULA_BOLT: u16 = 573;
/// `ProjectileID.NebulaLaser` (576) — the 574 eye's release laser,
/// aiStyle 1 (`Projectile.cs:6258-6269`: 4x4, hostile, penetrate -1,
/// alpha 255, extraUpdates 2, scale 1.2, timeLeft 600). W4-d11.
pub const NEBULA_LASER: u16 = 576;
/// `ProjectileID.BrainOfConfusion` (565) — the Brain of Confusion's
/// confusion burst, aiStyle 104 (`Projectile.cs:6161-6171`: 28x28,
/// penetrate -1, tileCollide false, ignoreWater, alpha 255, scale 0.8).
/// W4-d11.
pub const BRAIN_OF_CONFUSION: u16 = 565;
/// `ProjectileID.SporeTrap` (567) / `SporeTrap2` (568) — the Spore Sac
/// accessory's traps, aiStyle 105 (`Projectile.cs:6185-6205`: 14x14 /
/// 16x16, friendly, penetrate 1, alpha 255, timeLeft 3600, tileCollide
/// false, ignoreWater, `UseGroupStaticImmunity(567)`). W4-d11.
pub const SPORE_TRAP: u16 = 567;
pub const SPORE_TRAP_2: u16 = 568;
/// `ProjectileID.SporeGas` (569) / `SporeGas2` (570) / `SporeGas3` (571)
/// — the Spore Sac's released gas clouds, aiStyle 106
/// (`Projectile.cs:6206-6218`: 32x32, friendly, penetrate -1, alpha 255,
/// timeLeft 3600, tileCollide false, ignoreWater,
/// `UseGroupStaticImmunity(569)`). W4-d11.
pub const SPORE_GAS: u16 = 569;
pub const SPORE_GAS_2: u16 = 570;
pub const SPORE_GAS_3: u16 = 571;
/// `ProjectileID.NebulaSphere` (575) — the nebula pillar's seeking
/// sphere, aiStyle 107 (`Projectile.cs:6219-6229`: 24x24, hostile,
/// penetrate -1, tileCollide false, timeLeft 420, alpha 255). W4-d11.
pub const NEBULA_SPHERE: u16 = 575;
/// `ProjectileID.DesertDjinnCurse` (596) — the djinn's curse flame,
/// aiStyle 107 (`Projectile.cs:6450-6460`: 8x8, hostile, alpha 255,
/// ignoreWater, timeLeft 180, tileCollide false). W4-d11.
/// `ProjectileID.MechanicWrench` (582) — the Mechanic's thrown wrench,
/// aiStyle 109 (`Projectile.cs:6316-6324`: 20x20, friendly, penetrate
/// -1, MaxUpdates 2 = extraUpdates 1). W4-d11.
pub const MECHANIC_WRENCH: u16 = 582;
/// `ProjectileID.NurseSyringeHeal` (584) — the Nurse's healing syringe,
/// aiStyle 110 (`Projectile.cs:6340-6348`: 10x10, friendly, scale 0.8,
/// penetrate 3). W4-d11.
pub const NURSE_SYRINGE_HEAL: u16 = 584;
pub const VORTEX_LIGHTNING: u16 = 578;
pub const VORTEX_PORTAL: u16 = 579;
/// `ProjectileID.DesertDjinnCurse` (596) — the djinn's curse flame,
/// aiStyle 107 (`Projectile.cs:6450-6460`: 8x8, hostile, alpha 255,
/// ignoreWater, timeLeft 180, tileCollide false). W4-g64 (the d11
/// e0395afc row port — the bool-seam activation).
/// `ProjectileID.BloodNautilusTears` (813) — the Blood Nautilus' tear
/// ritual marker, aiStyle 108 (`Projectile.cs:6282-6293`: the shared
/// 32x32 row with the 578/579, friendly + hostile, alpha 255,
/// tileCollide false, ignoreWater, drawLayer 1). W4-g64 (the d11
/// e0395afc row port — the four-phase machine's dust wiring).
/// `ProjectileID.BloodNautilusTears` (813) — the Blood Nautilus' tear
/// ritual marker, aiStyle 108 (`Projectile.cs:6282-6293`: the shared
/// 32x32 row with the 578/579, friendly + hostile, alpha 255,
/// tileCollide false, ignoreWater, drawLayer 1). W4-d11.
pub const BLOOD_NAUTILUS_TEARS: u16 = 813;
pub const VORTEX_RITUAL_BOLT: u16 = 580;
/// `ProjectileID.CultistBossLightningOrbArc` (466) — the Lunatic Cultist's
/// lightning-orb arc (`Projectile.cs:5392-5403`: 14x14, aiStyle 88, hostile,
/// alpha 255, ignoreWater, extraUpdates 4, timeLeft 120*(4+1)=600). The
/// AI_088 466 branch (`:32046-32143`) is UNPORTED (the 88 dispatch rides
/// `ai_vortex_bolt`, the 580 machine); the row exists for the
/// `TrailingMode[466] = 1` history ring + the stationary trail dust
/// (`:17108-17116`, W4-d53 batch 3 — the 580 twin's body verbatim).
pub const CULTIST_LIGHTNING_ORB_ARC: u16 = 466;
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
/// `ProjectileID.InfluxWaver` (451) — the Terra Blade-family melee wave
/// (`Projectile.cs:5229-5239`: 16x16, aiStyle 81, melee, penetrate 3,
/// light 0.2, alpha 255). W4-d9.
pub const INFLUX_WAVER: u16 = 451;
/// `ProjectileID.CultistBossIceMist` (464) — the Lunatic Cultist's ice
/// mist (`Projectile.cs:5351-5361`: 60x60, aiStyle 86, hostile,
/// tileCollide false, penetrate −1, alpha 255, ignoreWater). W4-d9.
pub const CULTIST_BOSS_ICE_MIST: u16 = 464;
/// `ProjectileID.ClingerStaff` (482) — the Clinger Staff's cursed-flame
/// wall (`Projectile.cs:5572-5586`: 16x200, aiStyle 87, friendly,
/// tileCollide false, penetrate −1, alpha 255, ignoreWater, timeLeft
/// 18000, magic, idStatic-10). W4-d9.
pub const CLINGER_STAFF: u16 = 482;
/// `ProjectileID.CultistRitual` (490) — the ritual marker that spawns the
/// cultist dragon (`Projectile.cs:5655-5666`: 14x14, aiStyle 89, hostile,
/// alpha 255, ignoreWater, tileCollide false, timeLeft 600,
/// netImportant). W4-d9.
pub const CULTIST_RITUAL: u16 = 490;
/// `ProjectileID.MagicLantern` (492) — the light pet (`Projectile.cs:
/// 5645-5654`: 18x32, aiStyle 90, friendly, penetrate −1, netImportant,
/// timeLeft ×5). W4-d9.
pub const MAGIC_LANTERN: u16 = 492;
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
/// `ProjectileID.DD2DarkMageHeal` (674) — the heal cast's ground marker,
/// the 673 row's twin (`Projectile.cs:7208-7220`); the AI()'s aiStyle-133
/// block's 40-tick heal scan is `:36465-36497`.
pub const DARK_MAGE_HEAL: u16 = 674;
/// `ProjectileID.DD2ElderWins` (672) — the Old One's Army victory crystal's
/// destruction marker (`Projectile.cs:7196-7207`: 10x10, aiStyle 132,
/// alpha 255, tileCollide false, ignoreWater, timeLeft 900, hostile,
/// penetrate -1); the AI()'s inline 4-state machine is `:36318-36434`.
pub const DD2_ELDER_WINS: u16 = 672;
/// `ProjectileID.DD2OgreSmash` (683) — the Ogre's ground pound
/// (`Projectile.cs:7282-7293`: 30x30, aiStyle 135, alpha 255, tileCollide
/// false, ignoreWater, timeLeft 120, hostile, penetrate -1); the growing
/// box's tile walk is `AI_135_OgreStomp` `:68671-68787`.
pub const OGRE_SMASH: u16 = 683;
/// `ProjectileID.QueenSlimeSmash` (922) — the Queen Slime's ground pound,
/// the 683 row's twin (`Projectile.cs:8745-8756`); `AI_135`'s 922 arms
/// shrink the box to 30 and swap the gore for the crystal dust.
pub const QUEEN_SLIME_SMASH: u16 = 922;
/// `ProjectileID.MonkStaffT1` (697) — the Brand of the Inferno's swing
/// (`Projectile.cs:7366-7384`: 16x16, aiStyle 140, friendly+melee, scale
/// 1.35, penetrate -1, alpha 255, tileCollide false, ignoreWater, the
/// local-immunity 12 pair); `AI_140_MonkStaffT1` `:69416-69600`.
pub const MONK_STAFF_T1: u16 = 697;
/// `ProjectileID.DD2DarkMageShot` (675) — the Dark Mage's shadowflame
/// (`Projectile.cs:7221-7230`: 10x10, aiStyle 1, alpha 255, penetrate -1).
pub const DARK_MAGE_FLAME: u16 = 675;
/// `ProjectileID.DD2BetsyBreath` (687) — Betsy's breath
/// (`Projectile.cs:7315-7324`: 30x30, aiStyle 136, alpha 255, penetrate -1,
/// hostile; the spawn is `NPC.cs:62026`'s `NewProjectile(..., 687, num3,
/// 0f, Main.myPlayer, 0f, base.whoAmI)` — ai1 names the breathing NPC).
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
/// S1-E-21h d76 DOC-SWEEP — the Kill()-tail non-bomb cue kinds
/// (`Projectile.cs:70855-70975`, the d76 line-order range's last four
/// `PlaySound` sites; d77's map starts at `:71112`).
/// `ProjectileID.JimsDrone` (1020) — the Item62 blast (`:70865`).
pub const JIMS_DRONE: u16 = 1020;
/// `ProjectileID.BirdDroppings` (1080) — the Item177 splat (`:70936`).
// (BIRD_DROPPINGS const deduped into the W4-c10 definition above — same id 1080.)
/// `ProjectileID.TorchGodHelper` (1084) — the Item10 pop (`:70958`).
pub const TORCH_GOD_HELPER: u16 = 1084;
/// `ProjectileID.PrettyMirror` (1085) — the Item106 shatter (`:70904`).
pub const PRETTY_MIRROR: u16 = 1085;
/// `ProjectileID.RemoteControlCar` (1105) — shares the 1020 Item62 blast
/// (`:70855`'s `type == 1020 || type == 1105` arm).
pub const REMOTE_CONTROL_CAR: u16 = 1105;
pub const BETSY_BREATH: u16 = 687;
/// `ProjectileID.DD2BetsyFireball` (686) — Betsy's fireball
/// (`AI_110`'s phase-4 volley: `NewProjectile(..., 686, 35, ...)`).
pub const BETSY_FIREBALL: u16 = 686;
/// `ProjectileID.DD2PhoenixBowShot` (706) — the Phantom Phoenix's shot
/// (`Projectile.cs:7461-7473`: 66x66, aiStyle 1, the tracked
/// `DD2_PhantomPhoenixShot` loop, W4-d80).
pub const PHANTOM_PHOENIX: u16 = 706;
/// `ProjectileID.MonkStaffT3_AltShot` (709) — the Sky Dragon's Fury shot
/// (`Projectile.cs:7508-7520`: 12x12, aiStyle 1, the tracked
/// `DD2_SkyDragonsFuryShot` one-shot, W4-d80).
pub const SKY_DRAGONS_FURY_SHOT: u16 = 709;
/// `ProjectileID.RemoteControlCar` (1105) — the RC car, aiStyle 205
/// (`Projectile.cs:10294-10302`, `AI_205_RemoteControlCar` `:38076-38461`;
/// the `SoundID.RCCar` engine loop, W4-d80).
// (const deduped into the d76 REMOTE_CONTROL_CAR definition above — same id 1105.)
/// `ProjectileID.JimsDrone` (1020) — Jim's RC drone, aiStyle 195
/// (`Projectile.cs:9659-9666`, `AI_195_JimsDrone` `:40923-41067`; the
/// `SoundID.JimsDrone` engine loop, W4-d80).
// (const deduped into the d76 JIMS_DRONE definition above — same id 1020.)
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
/// `ProjectileID.FlamingScythe` (329, `Projectile.cs:4215-4225`) — the
/// Pumpking blade's volley shot (W4-c6 style-59): 80x80, aiStyle 56,
/// hostile, penetrate -1, no tile collide, timeLeft 420, light 0.25.
pub const FLAMING_SCYTHE: u16 = 329;
/// `ProjectileID.Skull` (270, `Projectile.cs:3558-3566`) — the Skeletron
/// head's spin shot (the expert mode's fired skull).
pub const SKULL: u16 = 270;
/// `ProjectileID.Shadowflames` (299, `Projectile.cs:3893-3905`) — the Giant
/// Cursed Skull's bolt (`NPC.cs:21808`, W4-c3): 6x6, aiStyle 1, hostile,
/// alpha 255, penetrate -1, extraUpdates 2, magic, ignoreWater,
/// tileCollide false.
pub const SHADOWFLAMES: u16 = 299;
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
/// `ProjectileID.SilverBullet` (981, `ProjectileID.cs:2516`) — the Silver
/// Bullet's shot (aiStyle 1, `Projectile.cs:9340-9353`): 4x4, friendly,
/// ranged, penetrate 1, light 0.5, alpha 255, scale 1.2, timeLeft 600,
/// extraUpdates 1. W4-g2d rowless-rider batch (the e3e flag): a plain
/// bullet — the generic aiStyle-1 flight is its whole AI (the only
/// per-type `AI_001` site is the `Next(6)` dust-279 gate `:56746-56752`,
/// dust-only, skipped with the layer like every other bullet's trail).
/// The `Damage_PVE_Inner` x3-vs-Werewolf arm (`:13385-13388`,
/// `targetNPC.type == 104`) is INERT: the Werewolf (`NPCID.cs:10885`) is
/// an unported NPC kind (the port's internal id 104 is DD2SkeletonT1,
/// NOT the vanilla id — `npc/data.rs:288`); recorded in research/26d-g2d.
pub const SILVER_BULLET: u16 = 981;
/// The special-bullet kind set S1-P-i18 owns (the aiStyle-1 arms in
/// `projectile_special_bullets`). Disjoint from the ai001 A/B/C kind lists,
/// the riders and the moon-lord set. 90 CrystalShard is aiStyle 24 (the
/// existing `ai_crystal_storm` arm) and 638 MoonlordBullet is S1-D-4's — both
/// excluded here.
/// `ProjectileID.SilverBullet` (981, `ProjectileID.cs:2516`) — the Silver
/// Bullet: the Werewolf-bane aiStyle-1 shot (`Projectile.cs:9340-9353`,
/// the resting counter `:56030`, the `Next(6)` drip gate `:56746-56752`,
/// the ×3 damage vs the Werewolf `Damage_PVE_Inner` `:13385-13388`, the
/// ProjLight 0.7/0.7 tint `:16343-16347`). W4-d1 SEAL — the e3/d75-flagged
/// unrowed kind (the e3 RETIRE census's 345-branch scan; the
/// `ai001_silver_bullet_drip_981` emitter's fade-chain slot `:56746`
/// between the 265/355 arms `:56728` and the 357 trail `:56753`).
/// The special-bullet kind set S1-P-i18 owns (the aiStyle-1 arms in
/// `projectile_special_bullets`). Disjoint from the ai001 A/B/C kind lists,
/// the riders and the moon-lord set. 90 CrystalShard is aiStyle 24 (the
/// existing `ai_crystal_storm` arm) and 638 MoonlordBullet is S1-D-4's — both
/// excluded here. W4-d1 SEAL adds the 981 SilverBullet (the e3/d75-flagged
/// unrowed sibling; the module's drip gate + Werewolf-bane arms).
pub const SPECIAL_BULLET_KINDS: [u16; 12] = [
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
    SILVER_BULLET,
];
/// `ProjectileID.FlyingKnife` (491, `ProjectileID.cs:1536`) — the Flying
/// Knife: the aiStyle-9 LEGACY spell (`Projectile.cs:5404-5412` —
/// 26×26, melee, penetrate −1). `AI_009_MagicMissiles`'s type-491 head
/// (`:43197-43201`) delegates to `AI_009_MagicMissiles_Old`
/// (`:43427-43653`) — the LIVE legacy machine the e3 RETIRE census
/// task-seeded to d1 (the `AI_019_Spears_Old` sibling is DEAD, zero
/// callers). The `_Old` body's 16/34/79 branches are dead for the only
/// reachable kind 491 — kept verbatim (the census-F2 convention).
pub const FLYING_KNIFE: u16 = 491;
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
/// `ignoreWater`, friendly AND hostile, `penetrate = -1`, `timeLeft = 300`,
/// trap). W4-d4 landed the faithful row WITH the `ai_trap_spear` arm (the
/// f19 F3 fix — the old 185 stand-in values are retired; correcting the row
/// before the arm existed would have parked the spear in `_ => {}`).
pub const TRAP_SPEAR: u16 = 186;
/// The Flamethrower Trap's jet (`ProjectileID.FlamethrowerTrap = 187`,
/// `Projectile.cs:2741-2751`: aiStyle 38, 6×6, `alpha = 255`,
/// `tileCollide = false`, `ignoreWater`, `timeLeft = 60`, trap — NEITHER
/// friendly nor hostile: the jet itself damages nothing, its 6-tick cycle
/// spawns the 188 FlamesTrap children). W4-d4 landed the row with the
/// `ai_flame_jet` arm.
pub const FLAMETHROWER_TRAP: u16 = 187;
/// The rope coils (aiStyle 35 — the `AI()` arm `Projectile.cs:27189-27291`):
/// the Rope Coil 171 (`ProjectileID.cs:896`), the Vine Rope Coil 475
/// (`:1504`), the Silk Rope Coil 505 (`:1564`) and the Web Rope Coil 506
/// (`:1566`). Every row is 14×14, `penetrate = -1`, `tileCollide = false`,
/// `timeLeft = 400` (`Projectile.cs:2570-2578` for 171/505/506,
/// `:5496-5504` for 475); the AI's placement walk drops the per-kind rope
/// tile (213/353/365/366). W4-d4.
pub const ROPE_COIL: u16 = 171;
pub const VINE_ROPE_COIL: u16 = 475;
pub const SILK_ROPE_COIL: u16 = 505;
pub const WEB_ROPE_COIL: u16 = 506;
/// The Beach Ball (`ProjectileID.BeachBall = 155`, `Projectile.cs:2426-2433`:
/// aiStyle 32, 44×44, friendly, netImportant). The bounce machine
/// (`:26887-27048`) kicks off players/NPCs/projectiles; the type-155
/// movement arm (`:18603-18618`) bounces ×−0.6 and broadcasts the
/// `Game.BallBounceResult` chat above 10 bounces. W4-d4.
pub const BEACH_BALL: u16 = 155;
/// The Mechanical Piranha (`ProjectileID.MechanicalPiranha = 190`,
/// `Projectile.cs:2779-2790`: aiStyle 39, 22×22, friendly, `penetrate = -1`,
/// `alpha = 255`, ranged, `usesLocalNPCImmunity` with
/// `localNPCHitCooldown = 14`). The Piranha Gun's shot — the chase/return
/// state machine (`:27513-27779`). Its death never cancels the channel
/// (`DontCancelChannelOnKill`, `ProjectileID.cs:524` — the port's
/// `try_cancel_channel` already exempts 190). W4-d4.
pub const MECHANICAL_PIRANHA: u16 = 190;
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
/// `ProjectileID.ChlorophyteOrb` (229, `ProjectileID.cs:1012`) — the
/// Chlorophyte Orb staff's orb (aiStyle 44, `Projectile.cs:3151-3163`).
/// W4-d53 FINAL: the Shimmer() activation's `:22403` kind arm names it;
/// the ROW + AI machine are the pushed W4-g2 lane's (`81c4ab85`) — this
/// constant is the merge keep-one (g2's data.rs carries the identical
/// `pub const CHLOROPHYTE_ORB: u16 = 229;`, drop one at merge). The orb is
/// rowless on THIS branch, so the kind arm is inert until g2's row lands.
pub const CHLOROPHYTE_ORB: u16 = 229;
pub const THUNDER_SPEAR_BOLT: u16 = 732;
pub const NORTH_POLE_SNOWFLAKE: u16 = 343;
pub const NORTH_POLE_FLAKE: u16 = 344;
pub const SWORDFISH_BONE_1: u16 = 1107;
pub const SWORDFISH_BONE_2: u16 = 1108;
pub const SWORDFISH_BONE_3: u16 = 1109;
/// `ProjectileID.Bomb`-family cannon ammo (178, `Projectile.cs:2641-2649`).
pub const CANNON_BOMB: u16 = 178;
/// The Portal Gun's bolt (601 = C# `ProjectileID.PortalGunBolt`,
/// `ProjectileID.cs:1756`; `Projectile.cs:6502-6510` — aiStyle 1, 10×10,
/// alpha 255, friendly, `extraUpdates = 30`, the STOCK penetrate 1).
/// NAME DEVIATION (recorded, W4-d6 retire — the d18 finding): the constant
/// was ported as "the explosive cannonball" citing `:6499-6512` under the
/// M7-WIRE-2 cannon-ammo block; `:6499-6501` is the TAIL of the previous
/// type arm, the true 601 block is `:6502-6510` and 601 is NOT cannon ammo.
/// The value 601, the aiStyle-1 slice-B arm (`projectile_ai001_b.rs`, the
/// portal fade/frame machine `:54066-54120`) and the trailing-mode-0 row
/// (`ProjectileID.cs:521` lists 601 → 0) are all correct for the bolt; the
/// rename is deferred (the `POISON_DART`/645 precedent below). The
/// teleport AI (`PortalHelper.TryPlacingPortal`, `Projectile.cs:18099`)
/// stays unported — a recorded remainder.
pub const CANNON_EXPLOSIVE: u16 = 601;
/// The bunny cannon's projectile (281, `Projectile.cs:3676-3687` — aiStyle 49,
/// no ported AI; the constant anchors the `BunnyCannonCanFire` scan).
pub const CANNON_BUNNY: u16 = 281;
/// `ProjectileID.ShadowFlame` (496, `ProjectileID.cs:1546`) — the
/// Shadowflame Hex Doll's aiStyle-91 flame (`Projectile.cs:5692-5704`).
pub const SHADOW_FLAME: u16 = 496;
/// The Toxic Flasks' cloud trio (511-513, `ProjectileID.cs:1576-1580`) —
/// aiStyle 92 (`Projectile.cs:5805-5840`), the `UseGroupStaticImmunity(511)`
/// shared identity. W4-d10.
pub const TOXIC_CLOUD: u16 = 511;
pub const TOXIC_CLOUD_2: u16 = 512;
pub const TOXIC_CLOUD_3: u16 = 513;
/// `ProjectileID.GasTrap` (1007, `ProjectileID.cs:2568`) — the dart trap's
/// poison gas, aiStyle 92 (`Projectile.cs:9558-9569`), hostile AND friendly
/// AND trap. W4-d10.
pub const GAS_TRAP: u16 = 1007;
/// `ProjectileID.ChlorophyteClaymoreSporeCloud` (1128,
/// `ProjectileID.cs:2810`) — the claymore's spore orb, aiStyle 92
/// (`Projectile.cs:10565-10579`), the idStatic-30 pair. W4-d10.
pub const SPORE_CLOUD: u16 = 1128;
/// `ProjectileID.NailFriendly` (514, `ProjectileID.cs:1582`) — the Nail
/// Gun's nail, aiStyle 93 (`Projectile.cs:5841-5850`). W4-d10.
pub const NAIL_FRIENDLY: u16 = 514;
/// `ProjectileID.CoinPortal` (518, `ProjectileID.cs:1590`) — the Coin
/// Gun's portal, aiStyle 94 (`Projectile.cs:5879-5888`). W4-d10.
pub const COIN_PORTAL: u16 = 518;
/// `ProjectileID.ToxicBubble` (523, `ProjectileID.cs:1600`) — the Toxic
/// Flask's bubble, aiStyle 95 (`Projectile.cs:5929-5938`). W4-d10.
pub const TOXIC_BUBBLE: u16 = 523;
/// `ProjectileID.IchorSplash` (524, `ProjectileID.cs:1602`) — the Ichor
/// Flask's splash, aiStyle 96 (`Projectile.cs:5939-5952`). W4-d10.
pub const ICHOR_SPLASH: u16 = 524;
/// `ProjectileID.FlyingPiggyBank` (525, `ProjectileID.cs:1604`) — the
/// Money Trough's flying piggy bank, aiStyle 97
/// (`Projectile.cs:5953-5961`). W4-d10.
pub const FLYING_PIGGY_BANK: u16 = 525;
/// `ProjectileID.CultistBossParticle` (526, `ProjectileID.cs:1606`) — the
/// Lunatic Cultist's homing particle, aiStyle 98
/// (`Projectile.cs:5962-5970`). W4-d10.
pub const CULTIST_BOSS_PARTICLE: u16 = 526;
/// `ProjectileID.MedusaHead` (535, `ProjectileID.cs:1624`) — the Medusa
/// Head's held channel marker, aiStyle 100 (`Projectile.cs:6094-6106`).
/// W4-d10.
pub const MEDUSA_HEAD: u16 = 535;
// `ProjectileID.MedusaHeadRay` (536) — the ray kind is d11's claim
// (`6aca9b7f`: the row + `ai_medusa_ray` + the `MEDUSA_HEAD_RAY` const).
// The keep-one drop-hunk EXECUTED (W4-d10, the crosschecks doc §2): d10's
// duplicate const is REMOVED; the 535 dispatch spawns the rays through
// the LITERAL 536 (`spawn_with_data_ai_owned(536, …)`) so this branch
// compiles standalone (the no-row spawn declines) AND merges without a
// const collision — d11's const is the one definition.
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
/// `ProjectileID.BouncingShield` (866, `Projectile.cs:8312-8324`) — the
/// Bouncing Shield item 4760's thrown shot (W4-f48 lane 20): 30×30,
/// aiStyle 3, `penetrate = 5`, melee, `extraUpdates = 1`, `light = 0.15`,
/// `usesLocalNPCImmunity` + `localNPCHitCooldown = -1`. The outbound leg
/// never counts `ai[1]` (`:39027-39030`); the homing seek runs on
/// `localAI[0]` ∈ [10, 360] (`:39220-39259`), the return seek is 16/1.2
/// (`:39427-39431`), the rotation tracks the velocity (`:39560-39568`)
/// and the on-hit store decays ×0.8 (`:13443-13445`) with its own
/// penetrate spend (`:14077-14104`, the generic `:14125` excludes it).
pub const BOUNCING_SHIELD: u16 = 866;
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
/// The Snapping Stone trio's strike effect projectiles (W4-e7 lane 23,
/// `ProjectileID.cs:2796`/`:2816`/`:2818`) — `TryMakingSnappingStoneStrike`
/// (`Projectile.cs:14318`/`:14322`/`:14327`) spawns them at the extra
/// targets; `snapping_stone.rs` owns the strike machine.
pub const SNAPPING_STONE_EFFECT: u16 = 1121;
pub const SNAPPING_STONE_MELEE_EFFECT: u16 = 1131;
pub const SNAPPING_STONE_SUMMON_EFFECT: u16 = 1132;
/// `ProjectileID.HiveFive` (999, the 1.4.4 yoyo — `Projectile.cs:9483-9490`).
pub const HIVE_FIVE: u16 = 999;
/// The yoyo accessory counterweights (`Projectile.cs:6084-6093`): 10×10,
/// `extraUpdates = 1`, `DefaultToYoyo()` (aiStyle 99, melee, penetrate −1),
/// `drawLayer = 0`, `scale = 1f`, `counterweight = true`. Their AI is the
/// style-99 SUB-arm `AI_099_1_Counterweights` (`:67380-67731`), gated ahead
/// of the yoyo arm by the C# `AI()` chain (`:33153-33156`) — W4-d10 (the
/// f19 F1 census finding).
pub const BLACK_COUNTERWEIGHT: u16 = 556;
pub const BLUE_COUNTERWEIGHT: u16 = 557;
pub const GREEN_COUNTERWEIGHT: u16 = 558;
pub const PURPLE_COUNTERWEIGHT: u16 = 559;
pub const RED_COUNTERWEIGHT: u16 = 560;
pub const YELLOW_COUNTERWEIGHT: u16 = 561;
/// `ProjectileID.PinkCounterweight` (1079, the 1.4.4 pink bag drop).
pub const PINK_COUNTERWEIGHT: u16 = 1079;
/// The seven counterweight kinds — the `AI()` sub-arm gate
/// (`(aiStyle == 99 && type >= 556 && type <= 561) || type == 1079`,
/// `Projectile.cs:33153`).
pub const COUNTERWEIGHT_KINDS: [u16; 7] = [
    BLACK_COUNTERWEIGHT,
    BLUE_COUNTERWEIGHT,
    GREEN_COUNTERWEIGHT,
    PURPLE_COUNTERWEIGHT,
    RED_COUNTERWEIGHT,
    YELLOW_COUNTERWEIGHT,
    PINK_COUNTERWEIGHT,
];
/// The counterweight kind test — the C# inline
/// `(type >= 556 && type <= 561) || type == 1079` (`:33153`, `:67397`'s
/// complement, `:67820`, `:18415`).
pub const fn is_counterweight(kind: u16) -> bool {
    matches!(kind, 556..=561 | 1079)
}
/// `ProjectileID.LightsBane` (974) — the Light's Bane sword's slash wave
/// (`Projectile.cs:9237-9252`'s SetDefaults, `AI_188_LightsBane`
/// `:42229-42262`, `Main.DrawProj_LightsBane` `Main.cs:27455-27495`).
pub const LIGHTS_BANE: u16 = 974;
/// `ProjectileID.Volcano` (978) — the Fiery Greatsword / Volcano (item 121)
/// melee on-hit explosion (`Projectile.cs:9253-9268`'s SetDefaults, aiStyle
/// 189 `AI_189_Volcano` `:42264-42371`). W4-d12 DOC-SWEEP item 7: d19's
/// a4185f89 landed the 189 ARM + the 182-186/190 rows but NOT this row, so it
/// ports here (the row is the spawn's only gate — `spawn_with_data` declines
/// rowless kinds). The spawn site is `Player.Volcano_TrySpawningVolcano`
/// (`Player.cs:44150-44158`), the item-121 melee proc — W4-d12 FINAL item 1
/// ports it (`spawn_volcano_proc` + the `_spawnVolcanoExplosion` latch).
pub const VOLCANO: u16 = 978;

// ===== W4-d21: the projectile aiStyles 201-210 family =====
// The missing machines' kinds (census: 26d-projectile-ai-201-210-census.md).
// Ids verified against `ID/ProjectileID.cs`; the aiStyle + box + flags
// against each `SetDefaults` arm (cited on the rows below).
/// `ProjectileID.AntlionClaw` (1082) / `StylistKilLaKillScissorsIWish`
/// (1083) — the thrown melee weapons, aiStyle 201 (`Projectile.cs:10095-
/// 10114`, `AI_201_ThrownMelee` `:39954-39983`).
pub const ANTLION_CLAW: u16 = 1082;
pub const STYLIST_SCISSORS: u16 = 1083;
/// `ProjectileID.TorchGodHelper` (1084) — aiStyle 202 (`:10115-10124`,
/// `AI_202_TorchGodHelper` `:39912-39952`).
/// `ProjectileID.StormLightning` (1091, the trap) / `LightningStrikeShot`
/// (1117, the weapon) / `ArcSurge` (1122) — aiStyle 203 (`:10175-10185`,
/// `:10427-10449`, `AI_203_StormLightning` `:39641-39855`).
pub const STORM_LIGHTNING: u16 = 1091;
pub const LIGHTNING_STRIKE_SHOT: u16 = 1117;
pub const ARC_SURGE: u16 = 1122;
/// `ProjectileID.PalworldDigtoise` (1098) / `PalworldTrustyDigtoise` (1123)
/// — aiStyle 204 (`:10241-10250`, `:10504-10513`, `AI_204_Digtoise`
/// `:38463-38801`).
pub const DIGTOISE: u16 = 1098;
pub const TRUSTY_DIGTOISE: u16 = 1123;
/// `ProjectileID.RemoteControlCar` (1105) — aiStyle 205 (`:10294-10302`,
/// `AI_205_RemoteControlCar` `:38076-38461`).
/// `ProjectileID.GlacierFangShot` (1115) — aiStyle 208 (`:10401-10412`,
/// `AI_208_GlacierFangShot` `:36903-37049`).
pub const GLACIER_FANG_SHOT: u16 = 1115;
/// `ProjectileID.ChlorophyteClaymoreBladeSlam` (1129) — aiStyle 209
/// (`:10580-10596`, `AI_209_ChlorophyteClaymoreBladeSlam` `:69622-69631`).
pub const CHLOROPHYTE_CLAYMORE_BLADE_SLAM: u16 = 1129;

// ===== S1-D-4: the Empress of Light's projectile family =====
// The five kinds the Empress `AI_120_HallowBoss` machine spawns (R7's
// `npc_ai_empress.rs` `EmpressRequest::Projectile` set). Ids verified against
// `ID/ProjectileID.cs`; the aiStyle + box + flags against each `SetDefaults`
// arm. The AI cores live in `projectile_ai_empress.rs` (unwired, code-only).
/// `ProjectileID.HallowBossSplitShotCore` (871, `ProjectileID.cs:2296`) —
/// the pellet-storm core, aiStyle 172 (`Projectile.cs:8380-8392`). W4-d18:
/// the empress module's census left it unclaimed (no `AI_120` spawner);
/// the AI arm is `projectile_ai_misc::ai_pellet_storm`.
pub const HALLOW_BOSS_SPLIT_SHOT_CORE: u16 = 871;
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
/// `ProjectileID.FairyQueenMagicItemShot` (931, `ProjectileID.cs:2416`) —
/// the Nightglow's homing prismatic bolt, aiStyle 171's FRIENDLY weapon
/// variant (`Projectile.cs:8826-8841`, `AI_171` `:45451-45558`). W4-d18
/// tail: the empress boundary left it unrowed; the friendly arm is
/// `projectile_ai_empress::fairy_weapon_tick`.
pub const FAIRY_QUEEN_MAGIC_ITEM_SHOT: u16 = 931;

// ===== W4-d18 tail: the residual armed-but-unrowed kinds (181-210 census) =====
// These kinds' aiStyles already have dispatch arms on main (187/189/193),
// but the kinds themselves had no `proj_data` row and no W4 branch claimed
// them (the all-branch claim scan). The arms serve them once rowed.
/// `ProjectileID.InsanityShadowFriendly` (964, `ProjectileID.cs:2482`) —
/// the friendly Insanity shadow, aiStyle 187 (`Projectile.cs:9127-9142`,
/// `AI_187_ShadowHand` `:42502-42591` — `ai_insanity_shadow`).
pub const INSANITY_SHADOW_FRIENDLY: u16 = 964;
// PROJ-171 retire (978 dup, keep-d12c): the `VOLCANO` const + row + the
// ignoreWater/melee overlay terms + the witness block that were here are
// DROPPED — d12c's 9e668d0f0 is the adjudicated superset (it alone adds
// `proj_frames[978] = 5`, `Main.cs:8493`; the d19 verdict 6e80e4201). The
// 978 row rides d12c; this branch keeps only the 964/1106 residuals. The
// exact drop-hunk is published in research/26h for the merger.
/// `ProjectileID.PalworldMinionFoxsparksFlames` (1106,
/// `ProjectileID.cs:2766`) — the Foxsparks flamethrower, aiStyle 193
/// (`Projectile.cs:10303-10315`, `AI_193_Flamethrower` `:41337-41401` —
/// `ai_flamethrower`'s 1106 constants arm).
pub const FOXSPARKS_FLAMES: u16 = 1106;

// ===== W4-d18: the aiStyle-174 multisegment pets =====
// `AI_174_MultisegmentPet` (`Projectile.cs:45345-45427`) — the three
// worm-segment pets share one arm; the per-type keep-alive flags ride the
// port's `pet_flags` array (the `projectile_ai026_b` precedent).
/// `ProjectileID.EaterOfWorldsPet` (883, `ProjectileID.cs:2320`) —
/// aiStyle 174 (`Projectile.cs:8474-8485`).
pub const EATER_OF_WORLDS_PET: u16 = 883;
/// `ProjectileID.DestroyerPet` (887, `ProjectileID.cs:2328`) —
/// aiStyle 174 (`Projectile.cs:8496-8507`).
pub const DESTROYER_PET: u16 = 887;
/// `ProjectileID.LunaticCultistPet` (893, `ProjectileID.cs:2340`) —
/// aiStyle 174 (`Projectile.cs:8531-8542`).
pub const LUNATIC_CULTIST_PET: u16 = 893;

// ===== W4-d18: the aiStyle-175 titanium storm shard =====
/// `ProjectileID.TitaniumStormShard` (908, `ProjectileID.cs:2370`) —
/// the orbiting defensive shard, aiStyle 175 (`Projectile.cs:8626-8637`,
/// `AI_175_TitaniumStormShards` `:45319-45343`).
pub const TITANIUM_STORM_SHARD: u16 = 908;

// ===== W4-d18: the aiStyle-176 edgy lightning =====
/// `ProjectileID.ScytheWhipProj` (916, `ProjectileID.cs:2386`) — the

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
/// `ProjectileID.SandBallFalling` (31) — the Antlion's hostile sand spit
/// (`NPC.cs:24462`/`:24477`), the aiStyle-10 falling ball the Antlion's
/// ai[0]==2 override steers (`Projectile.cs:1131-1140`: 10x10, hostile,
/// penetrate −1, knockBack 6). W4-g8 lane 1 batch 1.
/// `ProjectileID.PinkLaser` (84) — the Gastropod's hostile laser
/// (`NPC.cs:24818`), the aiStyle-1 bullet (`Projectile.cs:1646-1659`:
/// 4x4, hostile, penetrate 3, extraUpdates 2, timeLeft 600). W4-g8 lane 1.
pub const PINK_LASER: u16 = 84;
/// `ProjectileID.FrostBlastHostile` (128) — the IceElemental's hostile
/// frost blast (`NPC.cs:24894`), the aiStyle-28 bolt (`Projectile.cs:2169-2179`:
/// 14x14, hostile, penetrate −1, coldDamage). W4-g8 lane 1.
pub const FROST_BLAST_HOSTILE: u16 = 128;
/// `ProjectileID.WebSpit` (472) — the wall-crawler's expert web spit
/// (`NPC.cs:29863`), the aiStyle-1 bullet (`Projectile.cs:5463-5473`:
/// 8x8, hostile, penetrate −1, tileCollide, timeLeft 60). W4-g8 lane 1.
pub const WEB_SPIT: u16 = 472;
/// `ProjectileID.DemonSickle` (44) — the Demon/VoodooDemon's hostile sickle
/// (`NPC.cs:23449`), the aiStyle-18 twin of the friendly DemonScythe 45
/// (`Projectile.cs:1271-1282`: hostile, penetrate −1).
pub const DEMON_SICKLE: u16 = 44;
/// `ProjectileID.HarpyFeather` (38) — the Harpy's hostile feather
/// (`NPC.cs:23428`, W4-d23): 14x14, aiStyle 1, hostile, penetrate −1,
/// tileCollide true (`Projectile.cs:1207-1216` — the `aiStyle = 0` write
/// at `:1211` is overwritten by the `aiStyle = 1` at `:1214`).
/// `ProjectileID.HarpyFeather` (38) — the Harpy's hostile feather shot
/// (`NPC.cs:23428`), the aiStyle-1 bullet (`Projectile.cs:1207-1216`:
/// 14x14, hostile, penetrate −1, tileCollide; the arm writes aiStyle 0
/// then 1 — the final 1 stands). W4-g8 machine round.
pub const HARPY_FEATHER: u16 = 38;
/// `ProjectileID.Stinger` (55, `ProjectileID.cs:664`) — the Hornet family's
/// hostile shot (`NPC.cs:50796`, the aiStyle-5 tail's volley): 10x10,
/// aiStyle 1 on the `flag3 = false` skip list (`Projectile.cs:55954` — the
/// resting counter, straight flight), `penetrate = -1`, tileCollide; the
/// stock 3600 life the volley overrides to 300 post-spawn (`NPC.cs:50797`,
/// the sim's timed-shot list). W4-d36 E4 item 4 (`npc_ai_style5.rs`).
pub const STINGER: u16 = 55;
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
// W4-g2 — the rowless proj kinds of d5's findings F5/F6 (the census doc
// `26d-projectile-ai-41-50-census.md` on `origin/task/W4-d5-proj-ai-41-50`):
// the Chlorophyte Partisan's orb and the Blood Rain's cloud chain. Without a
// `proj_data` row the data-carrying spawn silently declines the shot.
/// `ProjectileID.ChlorophyteOrb` (229, `ProjectileID.cs:1012`) — the
/// Chlorophyte Partisan's orb (aiStyle 44, `Projectile.cs:3151-3163`): 30x30,
/// friendly, melee, penetrate -1, light 0.2, id-static 10. The stock
/// `tileCollide = true` (`:775`) stands — the orb dies on tile contact, the
/// spore cloud 228's `false` does not carry over.
/// `ProjectileID.BloodCloudMoving` (243, `ProjectileID.cs:1040`) — the Blood
/// Rain's flying cloud (aiStyle 45, `Projectile.cs:3255-3262`): 28x28,
/// penetrate -1, netImportant — the 237 twin, verbatim.
pub const BLOOD_CLOUD_MOVING: u16 = 243;
/// `ProjectileID.BloodCloudRaining` (244, `ProjectileID.cs:1042`) — the
/// settled blood cloud (aiStyle 45, `Projectile.cs:3263-3273`): 54x28,
/// tileCollide false, ignoreWater, penetrate -1, timeLeft 18000. Its rain
/// cadence is 10 (`:28030`) and its owner cap 1 (`:28072`) — the arm's
/// per-kind constants, not row fields.
pub const BLOOD_CLOUD_RAINING: u16 = 244;
/// `ProjectileID.BloodRain` (245, `ProjectileID.cs:1044`) — the blood drop
/// (aiStyle 45, `Projectile.cs:3274-3288`): 4x40, friendly, magic,
/// penetrate 2, timeLeft 300, scale 1.1, extraUpdates 1, ignoreWater,
/// id-static 10. The arm pins alpha 100 (`:28099`).
pub const BLOOD_RAIN: u16 = 245;
/// `ProjectileID.RainNimbus` (264, `ProjectileID.cs:1082`) — the hostile
/// rain-cloud drop (aiStyle 45, `Projectile.cs:3491-3502`): 4x40, hostile,
/// penetrate -1, timeLeft 120, scale 1.1, extraUpdates 1, ignoreWater. The
/// arm pins rotation `atan2(vy, vx) + 1.57` (`:28103`). Spawned by the
/// Angry Nimbus (`NPC.cs:31647`, `Owner: Main.myPlayer`).
pub const RAIN_NIMBUS: u16 = 264;
/// `ProjectileID.GoldenShowerFriendly` (280) — the Golden Shower's stream
/// (aiStyle 12).
pub const GOLDEN_SHOWER: u16 = 280;
/// `ProjectileID.ShadowBeamFriendly` (294) — the Shadowbeam Staff's beam
/// (aiStyle 48); it reflects off tiles instead of dying.
pub const SHADOWBEAM: u16 = 294;
/// `ProjectileID.InfernoFriendlyBolt` (295) — the Inferno Fork's bolt
/// (aiStyle 50); it spawns the blast 296 on death.
pub const INFERNO_FORK_BOLT: u16 = 295;
/// `ProjectileID.Blast` (296) - the fork's on-death explosion (aiStyle 50).
pub const INFERNO_BLAST: u16 = 296;
// W4-g2 — d5's F1/F4 rowless hostile twins (aiStyles 48/50).
/// `ProjectileID.ShadowBeamHostile` (290, `ProjectileID.cs:1134`) — the
/// Necromancer's beam (283/284, `NPC.cs:21204`) (aiStyle 48,
/// `Projectile.cs:3793-3803`): 4x4, hostile, magic, extraUpdates 100,
/// timeLeft 100, penetrate -1. The delay table maps 290→3 (`:53991`); the
/// arm's Item8 cue at `localAI[0] == 1` (`:28229-28232`) and the
/// HandleMovement reflect (`:18861-18904`) ride the 294's shape.
pub const SHADOWBEAM_HOSTILE: u16 = 290;
/// `ProjectileID.InfernoHostileBolt` (291, `ProjectileID.cs:1136`) — the
/// Diabolist's bolt (285/286, `NPC.cs:21207`) (aiStyle 50,
/// `Projectile.cs:3804-3814`): 12x12, hostile, alpha 255, magic,
/// tileCollide false, penetrate -1. The spawn passes the target centre as
/// `ai[0]/ai[1]` (`NPC.cs:21219`); the arm's arrival-dot kill (`:28333`)
/// reads them.
pub const INFERNO_HOSTILE_BOLT: u16 = 291;
/// `ProjectileID.InfernoHostileBlast` (292, `ProjectileID.cs:1138`) — the
/// bolt's on-death blast (aiStyle 50, `Projectile.cs:3815-3825`): 130x130,
/// hostile, alpha 255, magic, tileCollide false, penetrate -1. The Kill
/// spawn (`:74334-74340`) carries the bolt's FULL damage (no 0.75).
pub const INFERNO_HOSTILE_BLAST: u16 = 292;
/// `ProjectileID.UFOLaser` (433, `ProjectileID.cs:1420`) — the UFO minion's
/// laser (aiStyle 48, `Projectile.cs:5037-5046`): 8x8, friendly,
/// extraUpdates 100, timeLeft 100, ignoreWater. The stock penetrate 1
/// (`:774`) stands — the laser dies on its first hit, unlike the beams.
/// `Sets.MinionShot` (`ProjectileID.cs:431`) already lists 433.
pub const UFO_LASER: u16 = 433;
/// `ProjectileID.VenomFang` (355) — the Poison Staff's aiStyle-1 rider.
pub const VENOM_FANG: u16 = 355;
/// `ProjectileID.FrostBoltStaff` (359) — the Frost Staff's bolt.
pub const FROST_BOLT: u16 = 359;
/// `ProjectileID.RuneBlast` (129) — the Rune Wizard's shadow beam
/// (W4-g5, `Projectile.cs:2180-2190`: 14x14, aiStyle 28, alpha 255,
/// penetrate -1, hostile, tileCollide false).
pub const RUNE_BLAST: u16 = 129;
/// `ProjectileID.LibrarianSkeletonBook` (1092) — the Librarian Skeleton's
/// book shot (W4-g5, `Projectile.cs:10186-10194`: 16x16, aiStyle 18,
/// hostile, penetrate -1, timeLeft 420).
pub const LIBRARIAN_BOOK: u16 = 1092;
/// `ProjectileID.JungleSpike` (176, `ProjectileID.cs:906`) — the Spiked
/// Jungle Slime's hostile spike (`NPC.cs:61415`/`:61435`, the AI_001
/// shooter block): 6x6, aiStyle 1, hostile, `penetrate = -1`, alpha 255
/// (`Projectile.cs:2621-2629`). W4-d36 E4 batch 5.
pub const JUNGLE_SPIKE: u16 = 176;
/// `ProjectileID.IcewaterSpit` (177, `ProjectileID.cs:908`) — the Icy
/// Merman's hostile spit (`NPC.cs:59592`, the aiStyle-3 shooter block):
/// 10x10, aiStyle 28, hostile, `penetrate = -1`, alpha 255, coldDamage
/// (no port slot field — the module doc), `Projectile.cs:2630-2640`.
/// W4-d36 E4 batch 5.
pub const ICEWATER_SPIT: u16 = 177;
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
// ---- W4-d1 wave 3: the aiStyle-10 falling-block family (`AI_010`,
// `Projectile.cs:57826-58100`). The kinds `WorldGen.GetSandfallProjData`
// (`WorldGen.cs:86514-86570`, `tr_core::falling::sandfall_proj_data`) spawns
// when a `TileID.Sets.Falling` tile loses its support, plus the DirtBall
// the Dirt Rod fires. The rows are verbatim `SetDefaults` slices; the
// `friendly`/`hostile` flags the row shape carries no field for ride the
// arm (`projectile_ai_falling`) and the damage-pass gates (ANALYSIS "The
// projectile data-row deviations").
/// `ProjectileID.DirtBall` (17, `ProjectileID.cs:588`) — the Dirt Rod's
/// dirt ball (`Projectile.cs:985-992`): 10x10, aiStyle 10, friendly,
/// ignoreWater, the stock penetrate 1.
pub const DIRT_BALL: u16 = 17;
/// `ProjectileID.SandBallFalling` (31, `ProjectileID.cs:616`) — the sand
/// tile's fall (`Projectile.cs:1131-1140`): 10x10, aiStyle 10, friendly +
/// hostile, penetrate −1.
/// `ProjectileID.MudBall` (39, `ProjectileID.cs:632`) — the mud tile's
/// fall (`Projectile.cs:1217-1226`), the 31's shape.
pub const MUD_BALL_FALLING: u16 = 39;
/// `ProjectileID.AshBallFalling` (40, `ProjectileID.cs:634`) — the ash
/// tile's fall (`Projectile.cs:1227-1236`), the 31's shape. The channel
/// aim-steer excludes it (`:57956`).
pub const ASH_BALL_FALLING: u16 = 40;
/// `ProjectileID.EbonsandBallFalling` (56, `ProjectileID.cs:666`) — the
/// ebonsand tile's fall (`Projectile.cs:1384-1393`), the 31's shape.
pub const EBONSAND_BALL_FALLING: u16 = 56;
/// `ProjectileID.PearlSandBallFalling` (67, `ProjectileID.cs:688`) — the
/// pearlsand tile's fall (`Projectile.cs:1467-1476`), the 31's shape.
pub const PEARLSAND_BALL_FALLING: u16 = 67;
/// `ProjectileID.SiltBall` (71, `ProjectileID.cs:696`) — the silt tile's
/// fall (`Projectile.cs:1512-1521`), the 31's shape.
pub const SILT_BALL_FALLING: u16 = 71;
/// `ProjectileID.SlushBall` (179, `ProjectileID.cs:912`) — the slush
/// tile's fall (`Projectile.cs:2650-2659`), the 31's shape.
pub const SLUSH_BALL_FALLING: u16 = 179;
/// `ProjectileID.CrimsandBallFalling` (241, `ProjectileID.cs:1036`) — the
/// crimsand tile's fall (`Projectile.cs:3228-3237`), the 31's shape.
pub const CRIMSAND_BALL_FALLING: u16 = 241;
/// `ProjectileID.ShellPileFalling` (812, `ProjectileID.cs:2178`) — the
/// shell-pile tile's fall (`Projectile.cs:7953-7962`), the 31's shape.
pub const SHELL_PILE_FALLING: u16 = 812;
/// `ProjectileID.CopperCoinsFalling` (411, `ProjectileID.cs:1376`) — the
/// copper coin pile's fall (`Projectile.cs:4933-4941`): 10x10, aiStyle 10
/// ONLY — the stock friendly false / penetrate 1 stand (the arm holds the
/// C# friendly state, the debris precedent).
pub const COPPER_COINS_FALLING: u16 = 411;
/// `ProjectileID.SilverCoinsFalling` (412, `ProjectileID.cs:1378`) — the
/// silver twin of the 411.
pub const SILVER_COINS_FALLING: u16 = 412;
/// `ProjectileID.GoldCoinsFalling` (413, `ProjectileID.cs:1380`) — the
/// gold twin of the 411.
pub const GOLD_COINS_FALLING: u16 = 413;
/// `ProjectileID.PlatinumCoinsFalling` (414, `ProjectileID.cs:1382`) — the
/// platinum twin of the 411.
pub const PLATINUM_COINS_FALLING: u16 = 414;
/// `ProjectileID.MudBallPlayer` (1081, `ProjectileID.cs:2716`) — the
/// player Mud Ball (`Projectile.cs:10087-10094`): 12x12, aiStyle 10,
/// friendly, penetrate −1; its own 20-tick gravity delay + side-slide.
pub const MUD_BALL_PLAYER: u16 = 1081;
/// The four coin-pile kinds the C# spells `type >= 411 && type <= 414`
/// (`Projectile.cs:57853`, `:17598`).
pub const COIN_PILE_KINDS: [u16; 4] = [
    COPPER_COINS_FALLING, SILVER_COINS_FALLING, GOLD_COINS_FALLING, PLATINUM_COINS_FALLING,
];
/// The falling-block kinds whose `SetDefaults` arms set `hostile = true`
/// beside `friendly` (the natural falls: `Projectile.cs:1138`/`:1224`/
/// `:1234`/`:1391`/`:1474`/`:1519`/`:2657`/`:3235`/`:7960`). The row shape
/// carries neither flag; the pool's `!friendly` player-damage proxy reads
/// this set so the C# `Damage_EVP` pass runs for them (the DUNGEON_DEBRIS
/// exemption's mirror, ANALYSIS W4-d1). The sandgun balls 42/65/68/354 are
/// friendly + ranged (NOT hostile), the 17/1081 friendly only, the coin
/// piles stock both-false — none joins.
pub const FALLING_HOSTILE_KINDS: [u16; 9] = [
    SAND_BALL_FALLING, MUD_BALL_FALLING, ASH_BALL_FALLING, EBONSAND_BALL_FALLING,
    PEARLSAND_BALL_FALLING, SILT_BALL_FALLING, SLUSH_BALL_FALLING,
    CRIMSAND_BALL_FALLING, SHELL_PILE_FALLING,
];
/// `ProjectileID.Sets.FallingBlockDoesNotFallThroughPlatforms`
/// (`ProjectileID.cs:433`) — `HandleMovement`'s `flag6 = false` set
/// (`Projectile.cs:17544-17547`): these falling blocks stop ON platforms.
/// Verbatim order.
pub const FALLING_BLOCK_NO_PLATFORM_FALL: [u16; 18] = [
    71, 179, 40, 39, 17, 812, 411, 412, 413, 414, 31, 67, 56, 241, 42, 68, 65, 354,
];
/// The aiStyle-10 kinds the W4-d1 falling arm serves (the rowed set minus
/// the 109 hostile snowball and the 736-738 debris trio, which keep their
/// own arms). The rowed-but-inert sandgun balls 42/65/68/354 join here —
/// the S1-D-5 "42-row gap" closes (ANALYSIS W4-d1).
pub const FALLING_BLOCK_KINDS: [u16; 19] = [
    DIRT_BALL, SAND_BALL_FALLING, MUD_BALL_FALLING, ASH_BALL_FALLING,
    EBONSAND_BALL_FALLING, PEARLSAND_BALL_FALLING, SILT_BALL_FALLING,
    SLUSH_BALL_FALLING, CRIMSAND_BALL_FALLING, SHELL_PILE_FALLING,
    COPPER_COINS_FALLING, SILVER_COINS_FALLING, GOLD_COINS_FALLING,
    PLATINUM_COINS_FALLING, MUD_BALL_PLAYER,
    42, 65, 68, 354,
];
// kinds this lane's trail arms ride are rowed here (895/898 the flight
// sparkles, 957 the column-scan bounce); the 650/882/888/896 rows stay
// d13's (the machine's home-spot ladder covers all seven regardless).
/// `ProjectileID.SuspiciousTentacle` (650) — the Suspicious Looking Tentacle
/// light pet (aiStyle 124, the ore-highlight scan; `ProjectileID.cs:1698`).
pub const SUSPICIOUS_TENTACLE: u16 = 650;
/// `ProjectileID.EyeOfCthulhuPet` (882) — the aiStyle-124 pet row
/// (`ProjectileID.cs:2478`).
pub const EOC_PET: u16 = 882;
/// `ProjectileID.TwinsPet` (888) — the aiStyle-124 pet row
/// (`ProjectileID.cs:2496`).
pub const TWINS_PET: u16 = 888;
/// `ProjectileID.FairyQueenPet` (895) — the aiStyle-124 pet row
/// (`ProjectileID.cs:2517`).
pub const FAIRY_QUEEN_PET: u16 = 895;
/// `ProjectileID.PumpkingPet` (896) — the aiStyle-124 pet row
/// (`ProjectileID.cs:2520`); d13's row, named here for the ladder's reads.
/// (`ProjectileID.cs:2520`).
pub const PUMPKING_PET: u16 = 896;
/// `ProjectileID.IceQueenPet` (898) — the aiStyle-124 pet row
/// (`ProjectileID.cs:2526`).
pub const ICE_QUEEN_PET: u16 = 898;
/// `ProjectileID.GlommerPet` (957) — the aiStyle-124 pet row
/// (`ProjectileID.cs:2703`).
pub const GLOMMER_PET: u16 = 957;
/// `ProjectileID.SuspiciousTentacle` (650) — the light pet
/// (`ProjectileID.cs:1698`); the home-spot ladder's count read.
/// `ProjectileID.TwinsPet` (888) — the ladder's count read
/// (`ProjectileID.cs:2496`).
/// `ProjectileID.EyeOfCthulhuPet` (882) — the ladder's count read
/// (`ProjectileID.cs:2478`).
/// The seven aiStyle-124 rows — `AI()`'s `else if (this.aiStyle == 124)`
/// (`Projectile.cs:34970-35696`): the 650 light pet + the six flying pets
/// (the d13 `STYLE124_PETS` set; only 895/898/957 are `proj_data`-rowed on
/// this branch, the rest arrive with d13's merge).
/// The seven aiStyle-124 rows — `AI()`'s `else if (this.aiStyle == 124)`
/// (`Projectile.cs:34970-35696`): the 650 light pet + the six flying pets.
pub const STYLE124_PETS: [u16; 7] = [
    SUSPICIOUS_TENTACLE, EOC_PET, TWINS_PET, FAIRY_QUEEN_PET, PUMPKING_PET,
    ICE_QUEEN_PET, GLOMMER_PET,
];
/// `ProjectileID.WireKite` (651) — the Grand Design / Multicolor Wrench's
/// mass-wire kite (aiStyle 125, `Projectile.cs:6990-7000`).
pub const WIRE_KITE: u16 = 651;
/// `ProjectileID.GeyserTrap` (654) — the geyser's steam column (aiStyle 126,
/// `Projectile.cs:7024-7037`); the friendly + hostile + trap row.
pub const GEYSER_TRAP: u16 = 654;
/// `ProjectileID.DD2OgreStomp` (670) — the Ogre's stomp shockwave (aiStyle
/// 126, `Projectile.cs:7174-7185`); the hostile-only row, the ±3 split.
pub const OGRE_STOMP: u16 = 670;
/// `ProjectileID.SandnadoFriendly` (656) — the Sand Elemental's friendly
/// sandnado (aiStyle 127, `Projectile.cs:7048-7060`).
pub const SANDNADO_FRIENDLY: u16 = 656;
/// `ProjectileID.SandnadoHostile` (657) — the hostile sandnado (aiStyle 127,
/// `Projectile.cs:7061-7070`).
pub const SANDNADO_HOSTILE: u16 = 657;
/// `ProjectileID.SandnadoHostileMark` (658) — the hostile sandnado's spawn
/// mark (aiStyle 128, `Projectile.cs:7071-7082`).
pub const SANDNADO_MARK: u16 = 658;
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
/// `ProjectileID.SpiritHeal` (298, `ProjectileID.cs:1150`) — the Spectre
/// set bonus's heal orb (aiStyle 52, W4-d6).
pub const SPIRIT_HEAL: u16 = 298;
/// `ProjectileID.VampireKnife` (304, `ProjectileID.cs:1162`) — the Vampire
/// Knives' thrown knife (aiStyle 2, W4-d6 residual 5).
pub const VAMPIRE_KNIFE: u16 = 304;
/// `ProjectileID.VampireHeal` (305, `ProjectileID.cs:1164`) — the Vampire
/// Knives' heal orb (aiStyle 52, W4-d6).
pub const VAMPIRE_HEAL: u16 = 305;
/// `ProjectileID.FlamingJack` (321, `ProjectileID.cs:1196`) — the
/// Pumpking's homing jack head (aiStyle 55, W4-d6).
pub const FLAMING_JACK: u16 = 321;
/// `ProjectileID.FlamingScythe` (329, `ProjectileID.cs:1212`) — the
/// Pumpking blade's spinning scythe (aiStyle 56, W4-d6).
/// `ProjectileID.WaterGun` (358, `ProjectileID.cs:1270`) — the Water Gun's
/// stream blob (aiStyle 60, W4-d6).
pub const WATER_GUN: u16 = 358;
/// `ProjectileID.SlimeGun` (406, `ProjectileID.cs:1366`) — the Slime Gun's
/// stream blob (aiStyle 60, W4-d6).
pub const SLIME_GUN: u16 = 406;
/// `ProjectileID.ShimmerGunStream` (1042, `ProjectileID.cs:2638`) — the
/// Shimmer Gun's stream blob (aiStyle 60, W4-d6).
pub const SHIMMER_GUN_STREAM: u16 = 1042;
/// `ProjectileID.Typhoon` (409) — the Razorblade Typhoon's vortex (aiStyle
/// 71, 30x30, `penetrate = -1`, `timeLeft = 300`).
pub const TYPHOON: u16 = 409;
/// `ProjectileID.DryadsWardCircle` (586, `ProjectileID.cs:1726`) — the
/// Dryad's ward ring (aiStyle 111, `AI_111_DryadsWard`). W4-d12.
pub const DRYADS_WARD_CIRCLE: u16 = 586;
/// `ProjectileID.TruffleSpore` (590, `ProjectileID.cs:1734`) — the Truffle
/// worm's spore (aiStyle 112's 590 sub-arm). W4-d12.
pub const TRUFFLE_SPORE: u16 = 590;
/// `ProjectileID.PortalGunGate` (602, `ProjectileID.cs:1758`) — the Portal
/// Gun's gate (aiStyle 114). W4-d12.
pub const PORTAL_GUN_GATE: u16 = 602;
/// `ProjectileID.SolarFlareRay` (607, `ProjectileID.cs:1768`) — the Solar
/// armor's orbiting flare ray (aiStyle 116). W4-d12.
pub const SOLAR_FLARE_RAY: u16 = 607;
/// `ProjectileID.SolarCounter` (608, `ProjectileID.cs:1770`) — the Solar
/// shield's counter-explosion (aiStyle 117; the spawn sites `Kill()` it
/// immediately, so its burst rides the Kill path `:72177-72225`). W4-d12.
pub const SOLAR_COUNTER: u16 = 608;
/// `ProjectileID.SolarWhipSwordExplosion` (612, `ProjectileID.cs:1778`) —
/// the Solar whip's explosion (aiStyle 117). W4-d12.
pub const SOLAR_WHIP_SWORD_EXPLOSION: u16 = 612;
/// `ProjectileID.StardustGuardianExplosion` (624, `ProjectileID.cs:1802`) —
/// the Stardust Guardian's dash explosion (aiStyle 117, the pet-shader
/// dust family). W4-d12.
pub const STARDUST_GUARDIAN_EXPLOSION: u16 = 624;
/// `ProjectileID.DaybreakExplosion` (953, `ProjectileID.cs:2460`) — the
/// Daybreak's explosion (aiStyle 117). W4-d12.
pub const DAYBREAK_EXPLOSION: u16 = 953;
/// `ProjectileID.DeadCellsMushroomBoiMinionExplosion` (1044,
/// `ProjectileID.cs:2642`) — the Mushroom Boi's explosion (aiStyle 117,
/// the ParticleOrchestrator family). W4-d12.
pub const DEAD_CELLS_MUSHROOM_BOI_EXPLOSION: u16 = 1044;
/// `ProjectileID.DandelionSeed` (836, `ProjectileID.cs:2226`) — the
/// Dandelion's wind-borne seed (aiStyle 112's 836 sub-arm, hostile). W4-d12.
pub const DANDELION_SEED: u16 = 836;
/// `ProjectileID.StardustCellMinionShot` (614) — the Stardust Cell's homing
/// shot, aiStyle 113's chase branch (`Projectile.cs:6611-6619`). W4-d12 F1.
pub const STARDUST_CELL_MINION_SHOT: u16 = 614;
/// `ProjectileID.Daybreak` (636) — the Daybreak's thrown spear, aiStyle 113
/// (`Projectile.cs:6838-6849`); its Kill spawns the 953 explosion
/// (`:72126-72145`). W4-d12 F1.
pub const DAYBREAK: u16 = 636;
/// `ProjectileID.TentacleSpike` (971) — the Scourge of the Corruptor's
/// splitting spike, aiStyle 113 (`Projectile.cs:9197-9206`). W4-d12 F1.
pub const TENTACLE_SPIKE: u16 = 971;
/// `ProjectileID.BloodButcherer` (975) — the Blood Butcherer's butcher
/// blade, aiStyle 113 (`Projectile.cs:9287-9296`, `hide = true`). W4-d12 F1.
pub const BLOOD_BUTCHERER: u16 = 975;
/// `ProjectileID.DeadCellsKillingDeckCard` (1024) — the Killing Deck's
/// thrown card, aiStyle 113's three-state machine (`Projectile.cs:9693-9706`).
/// W4-d12 F1.
pub const DEAD_CELLS_KILLING_DECK_CARD: u16 = 1024;
/// `ProjectileID.NebulaArcanum` (617) — the Arcanum's homing bolt (aiStyle
/// 118, local-immunity 8).
pub const NEBULA_ARCANUM: u16 = 617;
/// `ProjectileID.NebulaArcanumSubshot` (618) — the Arcanum's arm (aiStyle
/// 119, 18x30, `timeLeft = 420`, local-immunity 20).
pub const NEBULA_ARCANUM_SUBSHOT: u16 = 618;
// W4-d78 batch 2 — the Kill child-spawn family's unrowed kinds (the 617/246/
// 117 death arms spawn these; without a row the data-carrying spawn declines
// the child and the burst vanishes). Ids verified against `ID/ProjectileID.cs`.
/// `ProjectileID.NebulaArcanumExplosionShotShard` (620, `ProjectileID.cs:1794`)
/// — the Arcanum bolt's death shard (aiStyle 29's own 620 arm,
/// `Projectile.cs:37672-37704`, 8x8, alpha 255, magic, `penetrate = 1`; the
/// 617 Kill arm `Projectile.cs:72354`/`:72370`). S1-E-21h d77's Kill-host
/// port; W4-d78 batch 2 rows the child-spawn family. ONE row for the kind
/// (the d77 `NEBULA_ARCANUM_SHARD` alias below — the merge dedupe).
pub const NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD: u16 = 620;
/// S1-E-21h d77's name for the same kind 620 (the merge keeps both lanes'
/// call sites; the `proj_data` row is single).
pub const NEBULA_ARCANUM_SHARD: u16 = NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD;
/// `ProjectileID.Stynger` (246, `ProjectileID.cs:1046`) — the Stynger
/// launcher's bolt (aiStyle 1, 10x10, alpha 255, ranged, `extraUpdates = 1`;
/// its Kill arm `Projectile.cs:76944-76986` spawns the 249 shrapnel).
pub const STYNGER: u16 = 246;
/// `ProjectileID.StyngerShrapnel` (249, `ProjectileID.cs:1052`) — the
/// Stynger bolt's death shrapnel (aiStyle 2, 12x12, ranged; its OWN Kill arm
/// `Projectile.cs:76987-77015` blasts a 100-box).
pub const STYNGER_SHRAPNEL: u16 = 249;
/// `ProjectileID.BoneShard` (1124, `ProjectileID.cs:2802`) — the Bone Arrow's
/// death shard (aiStyle 1, 6x6, ranged, `timeLeft = 600`, scale 0.95,
/// `extraUpdates = 1`; the 117 Kill arm `Projectile.cs:74748`).
pub const BONE_SHARD: u16 = 1124;
// W4-d8 — the aiStyle-72/73/74/76 census (the Bubble Gun, the firework
// fountains, the Scutlix mount's laser and the two mount crosshairs). Every
// id is the verbatim `ProjectileID` value; the `SetDefaults` arms are cited
// per row in [`proj_data`].
/// `ProjectileID.Bubble` (410) — the Bubble Gun's bubble (aiStyle 72,
/// `Projectile.cs:4917-4932`: 14x14, penetrate 1, alpha 255, timeLeft 100).
pub const BUBBLE: u16 = 410;
/// `ProjectileID.FireworkFountainYellow` (419) — the first firework fountain
/// (aiStyle 73, `Projectile.cs:4951-4959`); the tile-interaction spawn rolls
/// `419 + Next(4)` (`Player.cs:32374`).
pub const FIREWORK_FOUNTAIN_YELLOW: u16 = 419;
/// `ProjectileID.FireworkFountainRed` (420).
pub const FIREWORK_FOUNTAIN_RED: u16 = 420;
/// `ProjectileID.FireworkFountainBlue` (421).
pub const FIREWORK_FOUNTAIN_BLUE: u16 = 421;
/// `ProjectileID.FireworkFountainRainbow` (422).
pub const FIREWORK_FOUNTAIN_RAINBOW: u16 = 422;
/// The four firework fountains (419-422) — the aiStyle-73 family.
pub const FIREWORK_FOUNTAINS: [u16; 4] = [
    FIREWORK_FOUNTAIN_YELLOW,
    FIREWORK_FOUNTAIN_RED,
    FIREWORK_FOUNTAIN_BLUE,
    FIREWORK_FOUNTAIN_RAINBOW,
];
/// `ProjectileID.ScutlixLaserFriendly` (434) — the Scutlix mount's laser
/// (aiStyle 74, `Projectile.cs:5047-5055`: 1x1, extraUpdates 100, penetrate
/// −1). The Update-tail charge latch (`:17079-17086`) flips extraUpdates to 1.
pub const SCUTLIX_LASER_FRIENDLY: u16 = 434;
/// `ProjectileID.ScutlixLaser` (606, `ProjectileID.cs:1766`) — the Scutlix
/// mount's ACTUAL fired laser (`Mount.UseAbility` case 9, `Mount.cs:3264`/
/// `:3287`: two eye beams, damage 150, velocity 14 toward the clamped
/// cursor). aiStyle 1 (`Projectile.cs:6530-6541`: 5x5, alpha 255,
/// extraUpdates 2, timeLeft 600, ignoreWater). W4-d8 DOC-SWEEP (3): the
/// 434 "ScutlixLaserFriendly" has NO spawner in the 1.4.5.8 decompile —
/// the mount fires 606; the 434 row + aiStyle-74 arm stay for the census
/// (the id is live in `Damage_CanDealDamage`/the Update tails) but the
/// mount's beam is this kind.
/// `ProjectileID.ScutlixLaserCrosshair` (441) — the Scutlix mount's aim
/// crosshair (aiStyle 76, `Projectile.cs:5117-5125`: 1x1, hide, ignoreWater,
/// tileCollide false).
pub const SCUTLIX_LASER_CROSSHAIR: u16 = 441;
/// `ProjectileID.DrillMountCrosshair` (453) — the Drill Containment Unit's
/// aim crosshair (aiStyle 76, `Projectile.cs:5250-5258`).
pub const DRILL_MOUNT_CROSSHAIR: u16 = 453;
/// The two mount crosshairs (441/453) — the aiStyle-76 family. Both are
/// stock `friendly = false` (`:788`), so they damage nothing.
pub const MOUNT_CROSSHAIRS: [u16; 2] = [SCUTLIX_LASER_CROSSHAIR, DRILL_MOUNT_CROSSHAIR];
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
/// `ProjectileID.SkyDragonsFuryElectrosphere` (1110, `ProjectileID.cs:2774`) —
/// the Sky Dragon's Fury alt-shot's death circle (aiStyle 77, the 443 sibling;
/// W4-f20 F3, the 709 Kill-hook half `Projectile.cs:73372-73388`).
pub const SKY_DRAGONS_FURY_ELECTROSPHERE: u16 = 1110;
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
/// `ProjectileID.CrystalPulse2` (522, `ProjectileID.cs:1598`) — the pulse's
/// death-split shard (aiStyle 29's own 522 arm, `Projectile.cs:37742-37773`:
/// the ai[1] counter kills at > 40, the 0.2 gravity / 0.98 drag). W4-g1 lane a
/// (the 521 Kill hook's spawn target — row+arm+hook land together).
pub const CRYSTAL_PULSE_2: u16 = 522;
/// `ProjectileID.SparkleGuitar` (856, `ProjectileID.cs:2266`) — the Sparkle
/// Guitar's note (aiStyle 167, the CatmullRom arc).
pub const SPARKLE_GUITAR: u16 = 856;
/// `ProjectileID.ReleaseLantern` (855, `ProjectileID.cs:2264`) — the
/// lantern-night floating lantern (aiStyle 166, the dove machine's lantern
/// arm, `Projectile.cs:47540-47610`).
pub const RELEASE_LANTERN: u16 = 855;
/// `ProjectileID.ReleaseDoves` (851, `ProjectileID.cs:2258`) — the released
/// dove (aiStyle 166, the dove arm).
pub const RELEASE_DOVES: u16 = 851;
/// `ProjectileID.StarCannonStar` (955, `ProjectileID.cs:2464`) — the Star
/// Cannon's shot (aiStyle 5, `penetrate = -1`, `alpha = 50`).
pub const STAR_CANNON_STAR: u16 = 955;
// W4-f48 — the star-cloak family (`Player.Hurt`'s retaliate block spawns
// them, `Player.cs:38432-38481`; the overrides resolve LAST-wins starVeil
// 725 → beeCloak 724 → manaCloak 723 → snakeCloak 1133). All five share one
// `SetDefaults` arm (`Projectile.cs:7646-7658`) and one aiStyle-5 AI branch
// (`:23845-23856/:23914-23927/:23988-24031`).
/// `ProjectileID.ManaCloakStar` (723) — the Mana Cloak's star; the magic-hit
/// proc (`Projectile.cs:13831-13852`) spawns it with `ai2 = 1` (the
/// fall-through + star-drop half, unported families — cited in the AI arm).
/// `ProjectileID.BeeCloakStar` (724) — the Bee Cloak's star.
/// `ProjectileID.StarVeilStar` (725) — the Star Veil's star (the W4-f48 item).
/// `ProjectileID.StarCloakStar` (726) — the plain Star Cloak's star.
/// `ProjectileID.SnakeBandStarCloakTinkerStar` (1133) — the Serpent's Coil's
/// star (the snakeCloak override).
pub const SNAKE_CLOAK_STAR: u16 = 1133;
/// The five cloak-star kinds, one row / one AI branch
/// (`Projectile.cs:7646`/`:23845`). The 503 Holy star shares the AI branch's
/// kind list but not the row; it rides the i17 slice.
pub const CLOAK_STARS: [u16; 5] = [
    MANA_CLOAK_STAR,
    BEE_CLOAK_STAR,
    STAR_VEIL_STAR,
    STAR_CLOAK_STAR,
    SNAKE_CLOAK_STAR,
];
/// `ProjectileID.StarWrath` (503, `ProjectileID.cs:1560`) — the Star Wrath's
/// star (aiStyle 5, melee, `penetrate = 2`, `alpha = 255`, `tileCollide =
/// false`, `extraUpdates = 1`, local immunity 15; `Projectile.cs:5743-5756`).
/// W4-e3 coverage phase — the row unblocks `ai_falling_star_family`'s 503
/// gate + the `style5_spark_503` emitter (g64's unwired `trail_503` twin is
/// the merge keep-one; docs/research/231-proj-dust-coverage.md, was 125).
/// `ProjectileID.MeteorWhipMeteor` (1037, `ProjectileID.cs:2628`) — the
/// Meteor Whip's meteor (aiStyle 5, `alpha = 254`, `scale = 0.8`,
/// `extraUpdates = 1`, `tileCollide = false`, local immunity −1;
/// `Projectile.cs:9780-9792`). W4-e3 coverage phase.
/// `ProjectileID.MeteorStormMeteor` (1049, `ProjectileID.cs:2652`) — the
/// Meteor Staff's storm meteor (aiStyle 5, friendly AND hostile, `alpha =
/// 254`, `scale = 0.8`, `timeLeft = 1200`, `penetrate = -1`, `tileCollide =
/// false`, trap; `Projectile.cs:9938-9952`). The hostile half rides the
/// `spawn_hostile` friendly=false flip (the FLAMES_TRAP precedent); the
/// `netMode == 2` localAI[0] counter (`:23832-23840`) is server-only, dropped.
/// W4-e3 coverage phase.
/// W4-d17 — the rapier-stab family (aiStyle 161, `AI_161_RapierStabs`
/// `Projectile.cs:47612-47665`, the dispatch `:36607-36610`):
/// `DefaultToShortsword` (`:10773-10787`) — 18x18, `penetrate = -1`,
/// `tileCollide = false`, melee, `ownerHitCheck`, `extraUpdates = 1`,
/// `usesOwnerLight`, drawLayer 7. The Gladius (802, `:7921-7924`) and the
/// Ruler (842, `:8145-8148`) keep the stock 3600 life.
pub const GLADIUS_STAB: u16 = 802;
pub const RULER_STAB: u16 = 842;
/// The eight ore shortswords' stabs (938-945, `Projectile.cs:8917-8921`):
/// `DefaultToShortsword` + `timeLeft = 360`.
pub const SHORTSWORD_STAB_FIRST: u16 = 938;
pub const SHORTSWORD_STAB_LAST: u16 = 945;
/// `ProjectileID.ChlorophyteSaberStab` (1127, `ProjectileID.cs:2808`) — the
/// Chlorophyte Saber's stab (`Projectile.cs:10559-10564`):
/// `DefaultToShortsword` + the LOCAL immunity pair (−1). The `ai[1]`
/// melee-speed scale stretches the 16-tick machine (`:47621-47631`), the
/// Oiled-397 on-hit grant is `StatusNPC` `:11721-11724`.
pub const CHLOROPHYTE_SABER_STAB: u16 = 1127;

/// W4-d17 — `ProjectileID.WhiteTigerPounce` (818, `ProjectileID.cs:2190`) —
/// the Storm Tiger staff's pounce (aiStyle 162, `AI_162_TigerPounce`
/// `Projectile.cs:46701-46768`, the dispatch `:36611-36614`). The spawn is
/// the tiger's state-4 chain (`AI_164`'s sibling machine, `:49096-49115`):
/// `ai[0]` the negative chain delay, `ai[1]` the arc's tick budget.
pub const WHITE_TIGER_POUNCE: u16 = 818;

/// W4-d17 — `ProjectileID.ChumBucket` (820, `ProjectileID.cs:2194`) — the
/// chum bucket's bait (aiStyle 163, `AI_163_Chum`
/// `Projectile.cs:53474-53603`, the dispatch `:36615-36618`). The fishing
/// attraction book (`ChumBucketProjectileHelper`) LANDED in the W4-d17
/// doc-sweep: the pool's pending/last maps feed the bobber's pond scan
/// (`chumsInWater`), the reel head ages the baits
/// (`ReduceRemainingChumsInPool`).
pub const CHUM_BUCKET: u16 = 820;

/// W4-d17 — the dove/lantern release pair (aiStyle 166, `AI_166_Dove`
/// `Projectile.cs:47540-47610`, the dispatch `:36627-36630`):
/// `ProjectileID.ReleaseDoves` (851, `ProjectileID.cs:2256`) — the 4x4
/// dove (`:8173-8181`, timeLeft 300); `ProjectileID.ReleaseLantern` (855,
/// `:2264`) — the 12x12 lantern (`:8197-8205`, timeLeft 1000). Both
/// `netImportant`, `penetrate = -1`, the stock tileCollide true. The
/// blocked-move bounces are the OUTER type chain (`:18336-18361`).

/// W4-d17 — `ProjectileID.FirstFractal` (857, `ProjectileID.cs:2268`) — the
/// Zenith's component sword (aiStyle 168, `AI_168_FirstFractal`
/// `Projectile.cs:46537-46571`, the dispatch `:36635-36638`). The 60-tick
/// spin flight; `ai[0]` the per-tick rotation delta, `ai[1]` the hue
/// (`GetFirstFractalColor` = `hslToRgb(ai[1], 1, 0.5)`, `:70517-70520`).
pub const FIRST_FRACTAL: u16 = 857;

/// W4-d17 FOLLOW-UP — the whip proc-spawn children's kind aliases. The
/// ROWS for 916/917/918/1039 are d18's (`task/W4-d18-proj-ai-171-180`) and
/// 1038 is d20's (`task/W4-d20-proj-ai-191-200`) — NOT duplicated here (the
/// Zenith 933/1100 precedent); the proc spawns below reference the alias but
/// `proj_data` returns None until those rows merge, so the spawn no-ops.
/// Only 1036/1037 are UNCLAIMED — their rows land in this branch.
/// BATCH-D keep-one: d18's richer 916/917/918/1039 const docs + rows
/// survive (the d17 alias copies deduped here); the 1038 row landed with
/// W4-d20 (the aiStyle-196 petal) - the flower proc spawn is now LIVE.

/// `ProjectileID.ScytheWhipProj` (916) — the Dark Harvest black lightning.
pub const SCYTHE_WHIP_PROJ: u16 = 916;
/// `ProjectileID.CoolWhipProj` (917) — the Cool Whip snowflake minion.
pub const COOL_WHIP_PROJ: u16 = 917;
/// `ProjectileID.WeatherPainShot` (969, `ProjectileID.cs:2492`) — the
/// Weather Pain's twister, aiStyle 177 (`Projectile.cs:9169-9181`).
pub const WEATHER_PAIN_SHOT: u16 = 969;
/// `ProjectileID.FireWhipProj` (918) — the Firecracker proc explosion.
pub const FIRE_WHIP_PROJ: u16 = 918;
/// `ProjectileID.CobWhipProj` (1036, `ProjectileID.cs:2620`) — the Cob
/// Whip's spark minion (`Projectile.cs:13931`, the `cobWhipBuff` latch;
/// aiStyle 67's `flag7` chase `:49130`/`:50538`/`:50601`). UNCLAIMED — rowed.
pub const COB_WHIP_PROJ: u16 = 1036;
/// `ProjectileID.MeteorWhipMeteor` (1037, `ProjectileID.cs:2622`) — the
/// Starcrash (Meteor Whip) proc meteor
/// (`WhipTagEffect_Starcrash.cs:14-25`, aiStyle 5's 1037 arm). UNCLAIMED — rowed.
/// `ProjectileID.FlowerWhipPetal` (1038) — the Flower Whip petal fan.
pub const FLOWER_WHIP_PETAL: u16 = 1038;
/// `ProjectileID.ConstellationStar` (1039) — the Constellation star fan.
pub const CONSTELLATION_STAR: u16 = 1039;

/// W4-d17 — the aiStyle-161 kind set (the `DefaultToShortsword` callers:
/// `Projectile.cs:7921`/`:8145`/`:8917`/`:10559`). The `Colliding` stab arm
/// (`:15955`) and the `ownerHitCheck` gate (`:12637`) key on it.
pub fn is_rapier_stab(kind: u16) -> bool {
    matches!(
        kind,
        GLADIUS_STAB | RULER_STAB | SHORTSWORD_STAB_FIRST..=SHORTSWORD_STAB_LAST
            | CHLOROPHYTE_SABER_STAB
    )
}
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
// W4-d19 — the aiStyles 182-186/190 census slice's kinds (the
// `projectile_ai_sword_styles` module). Ids VERIFIED against
// `ID/ProjectileID.cs`; the rows carry the `SetDefaults` slices cited
// on each arm below.
/// `ProjectileID.SoulDrain` (476, `ProjectileID.cs:1506`) — the Soul
/// Drain's aiStyle-185 mana-pay latch (`Projectile.cs:5505-5517`).
pub const SOUL_DRAIN: u16 = 476;
/// `ProjectileID.ZoologistStrikeGreen` (880, `ProjectileID.cs:2314`) —
/// the Zoologist's green whip-slash, aiStyle 183 (`:8438-8450`).
pub const ZOOLOGIST_STRIKE_GREEN: u16 = 880;
/// `ProjectileID.ZoologistStrikeRed` (929, `ProjectileID.cs:2412`) —
/// the red twin, aiStyle 183 (`:8804-8816`).
pub const ZOOLOGIST_STRIKE_RED: u16 = 929;
/// `ProjectileID.FinalFractal` (933, `ProjectileID.cs:2420`) — the
/// Zenith's sword illusion, aiStyle 182 (`:8857-8873`).
pub const FINAL_FRACTAL: u16 = 933;
/// `ProjectileID.TorchGod` (949, `ProjectileID.cs:2452`) — the Torch
/// God's fireball, aiStyle 184 (`:8948-8955`).
/// `ProjectileID.PrincessWeapon` (950, `ProjectileID.cs:2454`) — the
/// Resonance Scepter's blast, aiStyle 186 (`:8956-8968`).
pub const PRINCESS_WEAPON: u16 = 950;
/// `ProjectileID.NightsEdge` (972, `ProjectileID.cs:2498`) — the
/// Night's Edge slash, aiStyle 190 (`:9298-9314`).
pub const NIGHTS_EDGE: u16 = 972;
/// `ProjectileID.Excalibur` (982, `ProjectileID.cs:2518`) — the
/// Excalibur slash, aiStyle 190 (`:9354-9370`).
pub const EXCALIBUR: u16 = 982;
/// `ProjectileID.TrueExcalibur` (983, `ProjectileID.cs:2520`) — the
/// True Excalibur slash, aiStyle 190 (`:9371-9387`).
pub const TRUE_EXCALIBUR: u16 = 983;
/// `ProjectileID.TerraBlade2` (984, `ProjectileID.cs:2522`) — the Terra
/// Blade's big-sword slash, aiStyle 190 (`:9388-9404`).
pub const TERRA_BLADE_2: u16 = 984;
/// `ProjectileID.TheHorsemansBlade` (997, `ProjectileID.cs:2548`) — the
/// Horseman's Blade slash, aiStyle 190 (`:9466-9482`).
pub const HORSEMANS_BLADE: u16 = 997;
/// `ProjectileID.DeadCellsFlintSlash` (1043, `ProjectileID.cs:2640`) —
/// the Flint's slash, aiStyle 190 (`:9858-9874`).
pub const DEAD_CELLS_FLINT_SLASH: u16 = 1043;
/// `ProjectileID.TrueCopperShortsword` (1100, `ProjectileID.cs:2754`) —
/// the Zenith's copper-sword illusion, aiStyle 182 (`:10265-10281`).
pub const TRUE_COPPER_SHORTSWORD: u16 = 1100;
/// `ProjectileID.ChlorophyteClaymoreBlade` (1126, `ProjectileID.cs:2806`)
/// — the Chlorophyte Claymore's slash + spore machine, aiStyle 190
/// (`:10543-10558`).
pub const CHLOROPHYTE_CLAYMORE_BLADE: u16 = 1126;
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
/// `ProjectileID.StardewJumino` (995) — the dryad's Stardew animation
/// (`Projectile.cs:9446-9457`): 240x104, aiStyle 192, friendly, penetrate
/// -1, tileCollide false, ignoreWater, netImportant, `timeLeft *= 5`.
/// Spawned by `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19226`, W4-g51
/// id-144). The AI_192 frame machine + the Stardew draw are the animation
/// lane's.
pub const STARDEW_JUMINO: u16 = 995;

/// `ProjectileID.WireKite` (651, `Projectile.cs:6990-7000`) — the Wire
/// Kite's placement ghost, aiStyle 125, drawLayer 5. S1-W4-g81 residuals.
/// `ProjectileID.SuperStar` (728, `ProjectileID.cs`'s row) — the Super
/// Star Cannon's star, aiStyle 151 (`Projectile.cs:7659-7671`). W4-d16.
pub const SUPER_STAR: u16 = 728;
/// `ProjectileID.SuperStarSlash` (729) — the cannon's slash, aiStyle 152
/// (`Projectile.cs:7672-7687`). W4-d16.
pub const SUPER_STAR_SLASH: u16 = 729;
/// `ProjectileID.BladeOfGrass` (976) — the Zenith component's grass-blade
/// slash, aiStyle 152 (`Projectile.cs:9270-9286`). W4-d16.
pub const ZENITH_GRASS_SLASH: u16 = 976;
/// `ProjectileID.Muramasa` (977) — the Zenith component's Muramasa slash,
/// aiStyle 152 (`Projectile.cs:9208-9224`). W4-d16.
pub const ZENITH_MURAMASA_SLASH: u16 = 977;
/// `ProjectileID.MoonLordWhipProc` (1045) — the Moon Lord whip's proc
/// slash, aiStyle 152 (`Projectile.cs:9890-9907`). W4-d16.
pub const MOON_LORD_WHIP_PROC: u16 = 1045;
/// `ProjectileID.ToiletEffect` (733) — the toilet's pop effect, aiStyle
/// 153 (`Projectile.cs:7716-7725`). W4-d16.
pub const TOILET_EFFECT: u16 = 733;
/// `ProjectileID.VoidLens` (734) — the Void Lens' placed lens, aiStyle
/// 154 (`Projectile.cs:7726-7735`). W4-d16.
pub const VOID_LENS: u16 = 734;
/// `ProjectileID.MysticSnakeCoil` (754) — the Mystic Snake's rope coil,
/// aiStyle 155 (`Projectile.cs:7767-7776`). W4-d16.
pub const MYSTIC_SNAKE_COIL: u16 = 754;
/// `ProjectileID.PaperAirplaneA` (761) / `PaperAirplaneB` (762) — the
/// thrown paper gliders, aiStyle 159 (`Projectile.cs:7846-7854`). W4-d16.
pub const PAPER_AIRPLANE_A: u16 = 761;
pub const PAPER_AIRPLANE_B: u16 = 762;

// W4-d15 — the DD2 weapon/pet slice's kinds (`ProjectileID.cs`'s values,
// the `SetDefaults` rows cited in [`proj_data`]).
/// `ProjectileID.MonkStaffT1Explosion` (698) — the Brand of the Inferno's
/// (T1, 697) ground blast, aiStyle 141 (`Projectile.cs:7385-7401`). The
/// 140 arm's 697 branch spawns it (`:69500`); the 141 arm is this slice's.
pub const MONK_STAFF_T1_EXPLOSION: u16 = 698;
/// `ProjectileID.MonkStaffT2` (699) — the Monk Staff T2's swing ghost,
/// aiStyle 142 (`Projectile.cs:7402-7418`).
pub const MONK_STAFF_T2: u16 = 699;
/// `ProjectileID.MonkStaffT2Ghast` (700) — the ghast the T2's first hit
/// summons (`SummonMonkGhast` `:15155-15184`), aiStyle 143
/// (`Projectile.cs:7419-7432`).
pub const MONK_STAFF_T2_GHAST: u16 = 700;
/// `ProjectileID.MonkStaffT3` (707) — the Monk Staff T3's normal swing,
/// aiStyle 140 (`Projectile.cs:7474-7490`). The row lands so the 3858's
/// normal use arm spawns. CROSS-NOTE (W4-d15 final phase): W4-d14
/// `eaa05ae9` landed `AI_140_MonkStaffT1` (`ai_monk_staff`, the 707 arm
/// `:69505-69591`) with a FIELD-IDENTICAL 707 row (16×16, 140, pen −1,
/// stock life, local immunity 6, alpha 255, scale 1) — the integrate is
/// a keep-one dedupe of this row + const, and the use arm's spawn
/// composes with their machine (no wiring gap).
pub const MONK_STAFF_T3: u16 = 707;
/// `ProjectileID.MonkStaffT3_Alt` (708) — the T3-alt swing ghost,
/// aiStyle 142 (`Projectile.cs:7491-7507`).
pub const MONK_STAFF_T3_ALT: u16 = 708;
/// `ProjectileID.MonkStaffT3_AltShot` (709) — the T3-alt's volley shot,
/// aiStyle 1 (`Projectile.cs:7508-7520`).
pub const MONK_STAFF_T3_ALT_SHOT: u16 = 709;
/// `ProjectileID.DD2PetDragon` (701) — the DD2 dragon pet, aiStyle 144
/// (`Projectile.cs:7433-7446`).
pub const DD2_PET_DRAGON: u16 = 701;
/// `ProjectileID.DD2PetGhost` (702) — the DD2 ghost pet, aiStyle 144
/// (the same row block).
pub const DD2_PET_GHOST: u16 = 702;
/// `ProjectileID.DD2PetGato` (703) — the DD2 gato pet, aiStyle 144
/// (the same row block).
pub const DD2_PET_GATO: u16 = 703;
/// `ProjectileID.DD2ApprenticeStorm` (704) — the Tome of Infinite
/// Wisdom's twister, aiStyle 145 (`Projectile.cs:7447-7460`).
pub const DD2_APPRENTICE_STORM: u16 = 704;
/// `ProjectileID.DD2Win` (713) — the DD2 victory banner, aiStyle 146
/// (an empty AI body, `Projectile.cs:53960-53962`; the row `:7560-7571`).
pub const DD2_VICTORY: u16 = 713;
/// `ProjectileID.UpbeatStar` (764) — the Upbeat Star pet, aiStyle 144
/// (`Projectile.cs:7866-7878`).
pub const UPBEAT_STAR: u16 = 764;
/// `ProjectileID.AxeFairyPet` (1050) — the Axe Fairy pet, aiStyle 144
/// (`Projectile.cs:9953-9966`).
pub const AXE_FAIRY_PET: u16 = 1050;
/// `ProjectileID.VoidLens` (734) — aiStyle 154, 30x24, tileCollide false,
/// `timeLeft = 10800`, `alpha = 255`, drawLayer 3 (`Projectile.cs:7726-7735`).
/// The Void Bag 4131 / Closed Void Bag 5325's shoot (`Item.cs:35875-35888`);
/// `IsInteractable` (`Projectile.cs:78828-78846`) and
/// `TryGetContainerIndex` → chest −5 (`:78820-78824`). W4-g56.

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
    /// `Projectile.glowMask` (`:332`, reset −1 `:780`) — the
    /// `TextureAssets.GlowMask` sheet the draw pass overlays
    /// (`Main.cs:34855-34942`); −1 = none. W4-g35: the `SetDefaults` slice
    /// [`set_defaults_glow_mask`] rides the data-carrying spawn.
    pub glow_mask: i16,
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
        // W4-d2 — the aiStyle-11 light pets' `this.light = 0.9f`
        // (`Projectile.cs:1000`/`:1529`/`:1681`/`:1695`).
        SHADOW_ORB | BLUE_FAIRY | PINK_FAIRY | GREEN_FAIRY => 0.9,
        MUSKET_BALL => 0.5,                    // `:957`
        // S1-P-i42 — the glowstick family's `this.light = 1f`
        // (`Projectile.cs:1334`/`:1362`/`:5493`/`:5859`/`:8351`/`:10160`).
        // The flares (163/310/1008-1011) set NO light — their glow is the
        // dust trail's (`AI_033`'s `Dust.NewDust`, `:27093`), not ProjLight.
        GLOWSTICK | STICKY_GLOWSTICK | SPELUNKER_GLOWSTICK | BOUNCY_GLOWSTICK
        | FAIRY_GLOWSTICK | RAINBOW_GLOWSTICK => 1.0,
        BULLET_SNOWMAN => 0.5,                 // `:1985`
        // W4-d1 SEAL — `Projectile.cs:9347`: the SilverBullet's light.
        SILVER_BULLET => 0.5,
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
        BOUNCING_SHIELD => 0.15,               // `:8314` (W4-f48 lane 20)
        UNHOLY_TRIDENT => 0.5,                 // `:2029`
        UNHOLY_TRIDENT_HOSTILE => 0.5,         // `:2044`
        SWORD_BEAM => 0.5,                     // `:2054`
        TERRA_BEAM => 0.5,                     // `:2216`
        LIGHT_BEAM => 0.5,                     // `:2441`
        NIGHT_BEAM => 0.5,                     // `:2453`
        ENCHANTED_BEAM => 0.2,                 // `:2598`
        INFLUX_WAVER => 0.2,                   // `:5236` (W4-d9)
        PIRATE_ARROW => 0.5,                   // `:2667`
        FROST_HYDRA => 0.25,                   // `:4003`
        BALL_OF_FROST => 0.8,                  // `:3364`
        MAGNET_SPHERE_BALL => 0.5,             // `:3376`
        GOLEM_EYE_BEAM => 0.3,                 // `:3436`
        FLAMING_SCYTHE => 0.25,                // `:4219` (W4-c6 style-59)
        CHRISTMAS_HOOK => 0.5,                 // `:4256`
        FLAMING_SCYTHE => 0.25,                // `:4219` (W4-d6)
        VAMPIRE_KNIFE => 0.2,                  // `:3961` (W4-d6 residual 5)
        MINI_RETINA_LASER => 0.75,             // `:4736`
        ANTI_GRAVITY_HOOK => 0.7,              // `:5186`
        ILLUMINANT_HOOK => 0.3,                // `:5631`
        CRYSTAL_SHARD_MELEE => 0.5,            // `:10644`
        // S1-P-i18 — the special bullets' `SetDefaults` light amounts.
        METEOR_SHOT => 0.6,                    // `:1189`
        // The natural falling star (12) sets `light = 1f` (`:936`); the
        // aiStyle-5 arm drops it to 0.9 per tick (`:23982`). W4-g70 lane 8.
        NATURAL_FALLING_STAR => 1.0,
        CRYSTAL_BULLET | CRYSTAL_SHARD => 0.5, // `:1722` / `:1736`
        CURSED_BULLET => 0.5,                  // `:1922`
        CHLOROPHYTE_BULLET => 0.5,             // `:2955`
        // W4-g2 (d5 F6) — the Partisan's orb (`Projectile.cs:3159`).
        CHLOROPHYTE_ORB => 0.2,                // `:3159`
        HIGH_VELOCITY_BULLET => 0.5,           // `:3245`
        ICHOR_BULLET => 0.5,                   // `:3657`
        VENOM_BULLET => 0.5,                   // `:3706`
        PARTY_BULLET => 0.5,                   // `:3720`
        NANO_BULLET => 0.5,                    // `:3734`
        EXPLODING_BULLET => 0.5,               // `:3747`
        GOLDEN_BULLET => 0.5,                  // `:3765`
        // W4-d1 — `Projectile.cs:5678`: the Crystal Vile Shard pair's light.
        CRYSTAL_VILE_HEAD | CRYSTAL_VILE_SHAFT => 0.2,
        // W4-d1 — `Projectile.cs:936`: the natural FallingStar's `light = 1f`.
        FALLING_STAR_NATURAL => 1.0,
        SILVER_BULLET => 0.5,                  // `:9347` (W4-g2d)
        // S1-P-i35 — the glow bobbers 987-993 (`Projectile.cs:9430-9434`):
        // `light = 1f` (the 986 plain override keeps the stock 0).
        987..=993 => 1.0,
        // W4-d15 — the natural FallingStar's `light = 1f`
        // (`Projectile.cs:936`).
        NATURAL_FALLING_STAR => 1.0,
        _ => 0.0,
    }
}

/// `Projectile.hostile`'s `SetDefaults` slice (W4-c19, the `:17341`
/// enchantment gate census): the reset false (`Projectile.cs:764`) and the
/// per-type `this.hostile = true` writes — the FULL 179-kind list, frozen
/// verbatim from the `SetDefaults` ladder (the multi-type arms' every kind
/// included: the 99/727/1013/1014/1053-1055/1057 boulder family `:1833`,
/// the 578/579/813 ritual trio `:6282`, the 673/674 pair `:7207`).
///
/// The only RUNTIME writes are the two bomb-death flips — aiStyle 49's
/// `ai[0] == -2` (`:28247`) and the 281's `ai[0] == -2` (`:75798`) — and
/// both kinds are classless bombs (`melee`/`ranged` false), so they can
/// never reach the `(melee || ranged)`-gated mount light this census
/// serves; the kind-derived flag is exact for every consumer.
pub(crate) fn set_defaults_hostile(kind: u16) -> bool {
    matches!(
        kind,
        12 | 31 | 38 | 39 | 40 | 43 | 44 | 55 | 56 | 67 | 71 | 75 | 81 | 82 | 83 | 84
            | 96 | 98 | 99 | 100 | 101 | 102 | 108 | 109 | 110 | 115 | 128 | 129 | 164
            | 174 | 176 | 177 | 179 | 180 | 184 | 185 | 186 | 188 | 201 | 202 | 203
            | 204 | 205 | 240 | 241 | 257 | 258 | 259 | 264 | 270 | 275 | 276 | 277
            | 288 | 290 | 291 | 292 | 293 | 299 | 300 | 302 | 303 | 325 | 327 | 329
            | 345 | 346 | 347 | 348 | 349 | 350 | 351 | 352 | 384 | 385 | 386 | 435
            | 436 | 437 | 438 | 447 | 448 | 449 | 450 | 452 | 454 | 455 | 456 | 462
            | 464 | 465 | 466 | 467 | 468 | 471 | 472 | 490 | 498 | 501 | 508 | 526
            | 537 | 538 | 539 | 540 | 572 | 573 | 574 | 575 | 576 | 577 | 578 | 579
            | 580 | 581 | 592 | 593 | 596 | 605 | 629 | 654 | 655 | 657 | 658 | 662
            | 670 | 671 | 672 | 673 | 674 | 675 | 676 | 681 | 682 | 683 | 685 | 686
            | 687 | 713 | 719 | 727 | 763 | 811 | 812 | 813 | 814 | 836 | 871 | 872
            | 873 | 874 | 909 | 919 | 920 | 921 | 922 | 923 | 926 | 961 | 962 | 965
            | 980 | 1001 | 1002 | 1005 | 1007 | 1013 | 1014 | 1021 | 1048 | 1049
            | 1053 | 1054 | 1055 | 1057 | 1073 | 1078 | 1091 | 1092
    )
}

/// `Projectile.noEnchantments`'s `SetDefaults` slice (W4-c19): the reset
/// false (`Projectile.cs:794`) and the four `this.noEnchantments = true`
/// writes — the 480 dart (`:5559`), the 1121/1131/1132 Dead Cells flint
/// helpers (`:10501`/`:10621`/`:10633`). NO runtime writes anywhere in
/// `Projectile.cs`; the kind-derived flag is exact.
pub(crate) fn set_defaults_no_enchantments(kind: u16) -> bool {
    matches!(kind, 480 | 1121 | 1131 | 1132)
}

/// `Projectile.noEnchantmentVisuals`'s `SetDefaults` slice (W4-c19): the
/// reset false (`Projectile.cs:795`) and the 21 `this.noEnchantmentVisuals
/// = true` writes — the outer `Update` gate (`:16700`) that skips
/// `UpdateEnchantmentVisuals` wholesale. The 1128 arm writes it twice
/// (`:10575`/`:10578`, a verbatim source quirk); the set is the same.
/// NO runtime writes; the kind-derived flag is exact.
pub(crate) fn set_defaults_no_enchantment_visuals(kind: u16) -> bool {
    matches!(
        kind,
        181 | 228 | 229 | 344 | 399 | 401 | 504 | 566 | 611 | 669 | 697 | 698 | 709
            | 732 | 933 | 954 | 979 | 1100 | 1108 | 1128 | 1129
    )
}
/// `TextureAssets.GlowMask` sheet index the projectile draw pass overlays
/// (`Main.cs:34855-34942` — the `proj.glowMask != -1` gate). The C# has
/// exactly 10 assignment sites (`Projectile.cs:4055`, `:4988-5035`,
/// `:9432`, `:10019`/`:10035`); the stock is −1 (`:780`). The 986 plain
/// bobber keeps −1 (the `:9430` gate starts at 987).
pub(crate) fn set_defaults_glow_mask(kind: u16) -> i16 {
    match kind {
        312 => 257, // JackOLantern `:4055`
        CHAINSAW_VORTEX => 2,      // `:4988`
        DRILL_VORTEX => 3,         // `:4995`
        CHAINSAW_NEBULA => 7,      // `:5002`
        DRILL_NEBULA => 8,         // `:5009`
        CHAINSAW_STARDUST => 179,  // `:5028`
        DRILL_STARDUST => 180,     // `:5035`
        // The glow bobbers `:9430-9434`: 318 + (type − 987).
        987 => 318,
        988 => 319,
        989 => 320,
        990 => 321,
        991 => 322,
        992 => 323,
        993 => 324,
        // The phaseblade/phasersaber boomerangs (`:10017-10020`/`:10033-36`).
        1101 => 372,
        1102 => 373,
        _ => -1, // the `:780` reset stock
    }
}

/// The ported families' `SetDefaults` slice. The spear trap's spear (186,
/// aiStyle 37) carries its faithful row since W4-d4 (the arm landed with
/// it — the f19 F3 fix); the flame jet (187, aiStyle 38) stays a
/// passthrough until its own W4-d4 commit. The magic (caster) families'
/// (M4-PROJ-6) rows sit in
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
            // `Projectile.cs:6248-6257` — the SalamanderSpit (572): the
            // 10x10 box, aiStyle 1, `alpha = 255`, `penetrate = -1`,
            // hostile; the stock `tileCollide = true` (`:775`) and
            // `timeLeft = 3600` (`:787`) stand. The salamanders' shot
            // (W4-d41, `NPC.cs:59454`); the OnFire-on-hit roll
            // (`:12189`, `Next(3)` then `Next(120, 240)`) is a HIT-time
            // draw on the projectile's own tick — the player-debuff
            // layer is unported (ANALYSIS, the flaming arrow's note).
            SALAMANDER_SPIT => (
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
            // `Projectile.cs:828-836` — the shuriken (3): the 22x22 box, aiStyle 2,
            // `penetrate = 4`, the stock 3600 life and the stock `tileCollide`.
            SHURIKEN => (22.0, 22.0, 2, 4, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-d2 — the aiStyle-11 light pets. `Projectile.cs:993-1007` (the
            // Shadow Orb 18): 32x32, aiStyle 11, friendly, `light = 0.9f`,
            // `alpha = 150`, `tileCollide = false`, `penetrate = -1`,
            // `timeLeft *= 5` (3600 → 18000), `ignoreWater`, `scale = 0.8f`,
            // `netImportant`. The fairies below share everything but the box
            // (18x18) and the alpha (the stock 0).
            SHADOW_ORB => (
                32.0,
                32.0,
                11,
                -1,
                TIME_LEFT_DEFAULT * 5,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 150, 0.8),
            ),
            // `Projectile.cs:1522-1535` (72) / `:1674-1687` (86) /
            // `:1688-1701` (87): 18x18, aiStyle 11, friendly, `light = 0.9f`,
            // `tileCollide = false`, `penetrate = -1`, `timeLeft *= 5`,
            // `ignoreWater`, `scale = 0.8f`, `netImportant` — NO alpha line
            // (the stock 0, `:779`).
            BLUE_FAIRY | PINK_FAIRY | GREEN_FAIRY => (
                18.0,
                18.0,
                11,
                -1,
                TIME_LEFT_DEFAULT * 5,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 0, 0.8),
            ),
            // W4-d2 — the aiStyle-13 chain weapons. `Projectile.cs:1057-
            // 1068` — the Harpoon (23): 4x4, penetrate 3, alpha 255,
            // ranged, usesLocalNPCImmunity + localNPCHitCooldown 12.
            HARPOON => (
                4.0,
                4.0,
                13,
                3,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (true, 12, false, -1, 255, 1.0),
            ),
            // `:3463-3473` — the Golem Fist (262): 30x30, penetrate −1,
            // alpha 255, melee, extraUpdates 1.
            GOLEM_FIST => (
                30.0,
                30.0,
                13,
                -1,
                TIME_LEFT_DEFAULT,
                1,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `:3567-3577` — the Boxing Glove (271): 20x20, penetrate
            // −1, alpha 255, melee, scale 1.2f.
            BOXING_GLOVE => (
                20.0,
                20.0,
                13,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.2),
            ),
            // `:3588-3597` — the Chain Knife (273): 26x26, penetrate
            // −1, alpha 255, melee.
            CHAIN_KNIFE => (
                26.0,
                26.0,
                13,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `:5561-5571` — the Chain Guillotine (481): 22x22,
            // penetrate −1, alpha 255, melee, extraUpdates 0 (the AI
            // arm re-arms it 1/2 per state, `:24663-24673`).
            CHAIN_GUILLOTINE => (
                22.0,
                22.0,
                13,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d2 — the aiStyle-17 tombstones. `Projectile.cs:1258-
            // 1270` (the Tombstone 43) / `:2872-2936` (the grave markers
            // 201-205) / `:5971-5983` (the rich gravestones 527-531):
            // 24x24, aiStyle 17, penetrate −1, the stock 3600 life and
            // tileCollide. The `knockBack = 12f` line is the SPAWNER's
            // input convention (the row carries no kb field — the
            // DropTombstone spawn passes it); the classic-world rows
            // set NO friendly/hostile (the stock false/false — the
            // getGoodWorld arm flips both, the spawn seam's FTW gate).
            43 | 201..=205 | 527..=531 => {
                (24.0, 24.0, 17, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d10 — the Shadowflame Hex Doll's flame (496,
            // `Projectile.cs:5692-5704`): 40×40, aiStyle 91, magic,
            // `MaxUpdates = 3` (extraUpdates 2, `:554-563`'s setter),
            // penetrate 3, the local-immunity −1 pair, alpha 255; the
            // stock tileCollide true / timeLeft 3600. The AI is
            // `projectile_ai_91_100::ai_shadowflame`.
            SHADOW_FLAME => (
                40.0,
                40.0,
                91,
                3,
                TIME_LEFT_DEFAULT,
                2,
                true,
                false,
                false,
                (true, -1, false, -1, 255, 1.0),
            ),
            // W4-d10 — the Toxic Cloud trio (`Projectile.cs:5805-5840`):
            // 32×32 / 40×38 / 30×28, aiStyle 92, penetrate −1, scale 1.1,
            // magic, alpha 255, `UseGroupStaticImmunity(511)` (the shared
            // identity below). The stock tileCollide stands at spawn — the
            // AI clears it every tick (`:32567`).
            TOXIC_CLOUD => (
                32.0, 32.0, 92, -1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, true, 10, 255, 1.1),
            ),
            TOXIC_CLOUD_2 => (
                40.0, 38.0, 92, -1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, true, 10, 255, 1.1),
            ),
            TOXIC_CLOUD_3 => (
                30.0, 28.0, 92, -1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, true, 10, 255, 1.1),
            ),
            // W4-d10 — the Gas Trap's poison gas (`Projectile.cs:9558-9569`):
            // 50×50, aiStyle 92, hostile AND friendly, penetrate −1, scale
            // 1.1, trap, hide (no port field — the draw layer).
            GAS_TRAP => (
                50.0, 50.0, 92, -1, TIME_LEFT_DEFAULT, 0, true, false, true,
                (false, -2, false, -1, 0, 1.1),
            ),
            // W4-d10 — the Chlorophyte Claymore's spore cloud
            // (`Projectile.cs:10565-10579`): 30×30, aiStyle 92, melee,
            // penetrate −1, scale 1.1, tileCollide false, the idStatic-30
            // pair (the double noEnchantmentVisuals write is one flag).
            SPORE_CLOUD => (
                30.0, 30.0, 92, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (false, -2, true, 30, 0, 1.1),
            ),
            // W4-d10 — the Nail Gun's nail (`Projectile.cs:5841-5850`):
            // 10×10, aiStyle 93, penetrate 3, alpha 255, ranged; the stock
            // tileCollide/timeLeft. The AI is `ai_nail` + the 4×4
            // collision box (`:19962`).
            NAIL_FRIENDLY => (
                10.0, 10.0, 93, 3, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d10 — the Coin Gun's portal (`Projectile.cs:5879-5888`):
            // 32×32, aiStyle 94, alpha 255, tileCollide false, ignoreWater;
            // the stock penetrate 1 / timeLeft 3600. No damage-class flag.
            COIN_PORTAL => (
                32.0, 32.0, 94, 1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d10 — the Toxic Flask's bubble (`Projectile.cs:5929-5938`):
            // 32×32, aiStyle 95, alpha 255, ranged, penetrate 1; the stock
            // tileCollide/timeLeft.
            TOXIC_BUBBLE => (
                32.0, 32.0, 95, 1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d10 — the Ichor Flask's splash (`Projectile.cs:5939-5952`):
            // 10×10, aiStyle 96, alpha 255, penetrate 5, ignoreWater,
            // melee, extraUpdates 5, the idStatic-10 pair.
            ICHOR_SPLASH => (
                10.0, 10.0, 96, 5, TIME_LEFT_DEFAULT, 5, true, false, false,
                (false, -2, true, 10, 255, 1.0),
            ),
            // W4-d10 — the Money Trough's flying piggy bank
            // (`Projectile.cs:5953-5961`): 30×24, aiStyle 97, tileCollide
            // false, timeLeft 10800, drawLayer 3 (no port field); the
            // stock penetrate 1, no damage-class flag (the bank is a
            // friendly interactable, `IsInteractable` :78832).
            FLYING_PIGGY_BANK => (
                30.0, 24.0, 97, 1, 10800, 0, false, false, false, RESET_STATE,
            ),
            // W4-d10 — the Lunatic Cultist's homing particle
            // (`Projectile.cs:5962-5970`): 8×8, aiStyle 98, tileCollide
            // false, timeLeft 120, alpha 255; the stock penetrate 1. The
            // C# row sets NO hostile flag — the particle is a visual
            // homing marker (the `:788` friendly default false stands;
            // the port's hostile-contact proxy keeps it inert).
            CULTIST_BOSS_PARTICLE => (
                8.0, 8.0, 98, 1, 120, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d10 — the Medusa Head's held marker
            // (`Projectile.cs:6094-6106`): 18×18, aiStyle 100, friendly,
            // penetrate −1, tileCollide false, magic, ignoreWater; the
            // stock timeLeft 3600 (the AI pins 2 every tick). drawLayer 8
            // + usesOwnerLight are draw-layer state (no port fields). The
            // `Damage_CanDealDamage` refusal (`:12448`'s `type == 535`)
            // means the marker itself never hits — its 536 rays carry the
            // damage (d11's claim).
            MEDUSA_HEAD => (
                18.0, 18.0, 100, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                RESET_STATE,
            ),
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
            // W4-d8 — the aiStyle-72 Bubble (410, `Projectile.cs:4917-4932`):
            // 14x14, friendly, penetrate 1, alpha 255, timeLeft 100, magic,
            // ignoreWater. The remix-world `armorPenetration = 20` arm
            // (`:4928-4931`) is unported (no remix seed flag — the 27 row's
            // precedent below); the stock 0 stands.
            BUBBLE => (
                14.0,
                14.0,
                72,
                1,
                100,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d8 — the firework fountains (419-422,
            // `Projectile.cs:4951-4959`): 4x4, aiStyle 73, friendly,
            // usesIDStaticNPCImmunity with cooldown 3. The stock penetrate 1
            // / tileCollide true stand; `timeLeft = 600` rides the
            // `NewProjectile` seed (`:11033-11048`, the 443/779 row
            // precedent). The C# keeps `friendly = true` (`:4956`); the
            // fountain never damages because the tile-interaction spawn
            // passes Damage 0 (`Player.cs:32374`) and `damage_pve`
            // early-returns on `damage <= 0`. The spawn passes ai0 = tileX,
            // ai1 = tileY.
            FIREWORK_FOUNTAIN_YELLOW | FIREWORK_FOUNTAIN_RED
            | FIREWORK_FOUNTAIN_BLUE | FIREWORK_FOUNTAIN_RAINBOW => (
                4.0,
                4.0,
                73,
                1,
                600,
                0,
                true,
                false,
                false,
                (false, -2, true, 3, 0, 1.0),
            ),
            // W4-d8 — the Scutlix laser (434, `Projectile.cs:5047-5055`):
            // 1x1, aiStyle 74, friendly, extraUpdates 100, penetrate −1.
            // The Update-tail charge latch (`:17079-17086`) flips
            // extraUpdates to 1 on the wire's last sub-step. The
            // `NewProjectile` ai[0]/ai[1] position pin (`:10927-10931`) is
            // a spawn-seam concern (the mount ability spawner is M2-MOUNT-4,
            // unported); the AI arm never reads ai[0]/ai[1], so the stock
            // spawn arguments stand until the spawner lands.
            SCUTLIX_LASER_FRIENDLY => (
                1.0,
                1.0,
                74,
                -1,
                TIME_LEFT_DEFAULT,
                100,
                true,
                false,
                false,
                RESET_STATE,
            ),
            // W4-d8 DOC-SWEEP (3) — the Scutlix mount's fired laser (606,
            // `Projectile.cs:6530-6541`): 5x5, aiStyle 1, friendly, alpha
            // 255, extraUpdates 2, scale 1, timeLeft 600, ignoreWater. The
            // stock penetrate 1 / tileCollide true stand. The AI_001 ramp
            // (`:54772-54828`) rides slice B's Martian-laser group: +5/tick
            // to the 150 cap while ai[1] == 0, −5/tick with the ≤ 0 kill
            // once the blocked-move latch (`:18278-18286`) sets ai[1] = 1.
            SCUTLIX_LASER => (
                5.0,
                5.0,
                1,
                1,
                600,
                2,
                true,
                false,
                true,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d8 — the mount crosshairs (441 Scutlix / 453 Drill,
            // `Projectile.cs:5117-5125`/`:5250-5258`): 1x1, aiStyle 76,
            // hide, ignoreWater, tileCollide false. The stock friendly
            // false (`:788`) — the port's slot default is true, so the arm
            // pins it per tick (the ai_dungeon_debris precedent). `hide`
            // has no slot field (the crosshair draws through the mount
            // layer, unported).
            SCUTLIX_LASER_CROSSHAIR | DRILL_MOUNT_CROSSHAIR => (
                1.0,
                1.0,
                76,
                1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                RESET_STATE,
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
            // W4-d1 — the aiStyle-6 powder/fertilizer family. The five arms
            // share the row shape: 64×64, aiStyle 6, friendly, penetrate
            // −1, alpha 255, tileCollide false, ignoreWater (the overlay
            // below); the stock timeLeft 3600 (`:787`) and extraUpdates 0
            // (`:771`) stand. `Projectile.cs:897-912` (10) / `:913-923`
            // (11) / `:5340-5350` (463) / `:9648-9658` (1019) /
            // `:9967-9977` (1051). The 10's infectedSeed box growth
            // (`:907-911`, width/height += /3*2 → 106×106) is applied at
            // the data-carrying spawn (the `Main.infectedSeed` read is a
            // world flag, not a row constant — `spawn_with_data_ai_owned`).
            PURIFICATION_POWDER | VILE_POWDER | VICIOUS_POWDER | FERTILIZER
            | SUPER_FERTILIZER => {
                (64.0, 64.0, 6, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                    (false, -2, false, -1, 255, 1.0))
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
            // W4-d7 wiring (F4) — `Projectile.cs:7163-7173` (669 Ale): 14x14,
            // aiStyle 68, friendly, `alpha = 255`, melee (the overlay below),
            // `noEnchantmentVisuals` (no port field — dropped), `penetrate =
            // -1`. The stock `timeLeft = 3600` / tileCollide true.
            ALE => (
                14.0,
                14.0,
                68,
                -1,
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
            // `Projectile.cs:5732-5742` — the Meowmere star (502): 16x16,
            // aiStyle 8, friendly (the pool's stock true), melee,
            // `penetrate = 5`, `usesLocalNPCImmunity = true` +
            // `localNPCHitCooldown = 10`. The stock `timeLeft = 3600`
            // (`:787`), `tileCollide` (`:775`), `extraUpdates = 0`, alpha 0
            // and scale 1 stand — the arm writes NO override. The bounce
            // ladder + chime ride the kind-first arm in `ai_ball_of_fire`
            // (`:18206-18254`, the d75 part-4 owed gate this row arms);
            // the `:24346-24352` velocity rotation is ported beside it.
            // S1-E-21h d75 part 5.
            MEOWMERE => (16.0, 16.0, 8, 5, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 10, false, -1, 0, 1.0)),
            // `Projectile.cs:10006-10021` — the phaseblade/phaser SMALL
            // group (1059-1064 + 1071/1075/1101): 38x38, aiStyle 3,
            // friendly (the pool's stock true), melee, `penetrate = -1`,
            // `netImportant`, `usesLocalNPCImmunity = true` +
            // `localNPCHitCooldown = 10`. The stock `timeLeft = 3600`
            // (`:787`), `tileCollide` (`:775`), `extraUpdates = 0` stand —
            // the arm writes no override (the 1101's `glowMask = 372` is
            // a draw-layer field with no port carrier). S1-E-21h d75
            // part 6 — the `IsAPhaseblade` set rows (`ProjectileID.cs:192`,
            // 19 kinds; the 1073 trap is NOT in the set, `:10038-10046`).
            1059..=1064 | 1071 | 1075 | 1101 => {
                (38.0, 38.0, 3, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 10, false, -1, 0, 1.0))
            }
            // `Projectile.cs:10022-10037` — the LARGE group (1065-1070 +
            // 1072/1076/1102): the same arm at 46x46 (the 1102's
            // `glowMask = 373` likewise rides the unported glow layer).
            1065..=1070 | 1072 | 1076 | 1102 => {
                (46.0, 46.0, 3, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 10, false, -1, 0, 1.0))
            }
            // `Projectile.cs:10047-10058` — the 1074 White Phaseblade's
            // OWN arm: 46x46, the same flags, no glowMask write. The
            // phaser's Item7/Item182 cue pair rides `ai_boomerang`'s
            // kind-gated arms (the 1074 exception at every site).
            1074 => (46.0, 46.0, 3, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 10, false, -1, 0, 1.0)),
            // W4-d12 F1 — `Projectile.cs:6611-6619` StardustCellMinionShot
            // (614): 16x16, aiStyle 113, friendly, `penetrate = -1`,
            // `alpha = 255`; the stock `timeLeft = 3600` (`:787`) and
            // `tileCollide` (`:775`). The chase branch reads `ai[1]` as the
            // target index (`:40592-40607`); MINION_SHOT already lists it.
            STARDUST_CELL_MINION_SHOT => {
                (16.0, 16.0, 113, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 F1 — `Projectile.cs:6838-6849` Daybreak (636): 16x16,
            // aiStyle 113, friendly, melee, `penetrate = -1`, `alpha = 255`,
            // `drawLayer = 3` (draw-only — the F3 skip), `MaxUpdates = 2` →
            // extraUpdates 1. Kill spawns the 953 explosion (`:72143`).
            DAYBREAK => {
                (16.0, 16.0, 113, -1, TIME_LEFT_DEFAULT, 1, true, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 F1 — `Projectile.cs:9197-9206` TentacleSpike (971):
            // 16x16, aiStyle 113, friendly, melee, `penetrate = -1`,
            // `alpha = 255`, `drawLayer = 3` (draw-only — the F3 skip).
            TENTACLE_SPIKE => {
                (16.0, 16.0, 113, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 F1 — `Projectile.cs:9287-9296` BloodButcherer (975):
            // 16x16, aiStyle 113, friendly, melee, `penetrate = -1`,
            // `alpha = 0`, `hide = true` (draw-only — the F3 skip).
            BLOOD_BUTCHERER => {
                (16.0, 16.0, 113, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d12 F1 — `Projectile.cs:9693-9706` DeadCellsKillingDeckCard
            // (1024): 16x16, aiStyle 113, friendly, magic, `penetrate = -1`,
            // `alpha = 255`, `drawLayer = 3` (draw-only — the F3 skip), the
            // local-immunity pair with cooldown −1 (one hit per NPC per
            // pass; the ai[0]=2 return re-arms it via
            // ResetLocalNPCHitImmunity `:40783`), `extraUpdates = 1`.
            DEAD_CELLS_KILLING_DECK_CARD => {
                (16.0, 16.0, 113, -1, TIME_LEFT_DEFAULT, 1, true, false, false, (true, -1, false, -1, 255, 1.0))
            }
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
            // W4-d1 — `Projectile.cs:5667-5680`: the Crystal Vile Shard pair
            // (493 head / 494 shaft): 32×32, aiStyle 4, `penetrate = -1`,
            // `tileCollide = false`, `alpha = 255`, `ignoreWater` (the overlay
            // below), `light = 0.2f` (the light slice), magic; the
            // `armorPenetration = 10` (`:5679`) has no row consumer (the
            // NETTLE_BURST precedent). The stock timeLeft 3600 (`:787`) and
            // extraUpdates 0 (`:771`) stand.
            CRYSTAL_VILE_HEAD | CRYSTAL_VILE_SHAFT => (
                32.0,
                32.0,
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
            // `Projectile.cs:5056-5063` — the Martian Turret's bolt (435):
            // 10x10, aiStyle 1, hostile, ignoreWater (the overlay below);
            // the stock penetrate 1 (`:774`), timeLeft 3600 (`:787`),
            // tileCollide (`:775`) — the arm writes no override. The
            // aiStyle-73 shot's carrier (W4-c11).
            MARTIAN_TURRET_BOLT => (
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
            // M2-MOUNT-4 — `Projectile.cs:6530-6541` — the Scutlix mount's
            // laser (606): 5x5, aiStyle 1, friendly, alpha 255,
            // extraUpdates 2, timeLeft 600, ignoreWater; the stock
            // penetrate 1 (`:774`) / tileCollide (`:775`) stand.

            // M2-MOUNT-4 — `Projectile.cs:8817-8825` — the Santank mount's
            // gift-box rocket (930): 14x14, aiStyle 16, friendly,
            // penetrate -1, ranged; the stock tileCollide (`:775`) /
            // timeLeft 3600 (`:787`) stand.
            SANTANK_MOUNT_ROCKET => (
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
            // W4-c3 — `Projectile.cs:3893-3905` — the Shadowflames bolt
            // (299): 6x6, aiStyle 1, hostile, `alpha = 255`,
            // `penetrate = -1`, `extraUpdates = 2`, magic, ignoreWater,
            // `tileCollide = false`; the stock timeLeft 3600 (`:787`). The
            // Giant Cursed Skull's tick-20 volley (`NPC.cs:21808`).
            SHADOWFLAMES => (
                6.0,
                6.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                2,
                false,
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
            // `Projectile.cs:4215-4225` — the Flaming Scythe (329, W4-c6
            // style-59): 80x80, aiStyle 56 (the spin flight, `:28845-28866`),
            // hostile, `penetrate = -1`, `tileCollide = false`,
            // `timeLeft = 420`; the `light = 0.25f` rides
            // `set_defaults_light` below.
            FLAMING_SCYTHE => (
                80.0,
                80.0,
                56,
                -1,
                420,
                0,
                false,
                false,
                false,
                RESET_STATE,
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
            // W4-g2d — `Projectile.cs:9340-9353`: the Silver Bullet 981:
            // 4x4, aiStyle 1, friendly, penetrate 1, alpha 255, scale 1.2,
            // timeLeft 600, extraUpdates 1 (NOT the hardmode bullets' 2),
            // ranged (`:9351`). The stock tileCollide TRUE stands. Light
            // 0.5 rides `set_defaults_light`.
            SILVER_BULLET => (4.0, 4.0, 1, 1, 600, 1, true, false, false, (false, -2, false, -1, 255, 1.2)),
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
            // `:2728-2740` — the Spear Trap's spear (186): 10x14, aiStyle
            // **37**, tileCollide false, ignoreWater (the overlay below),
            // friendly AND hostile, penetrate -1, timeLeft 300, trap.
            // W4-d4 landed the faithful row WITH the `ai_trap_spear` arm
            // (the f19 F3 fix — the old 185 stand-in 14x14/14/900 is
            // retired; the ANALYSIS S1-D-5 stand-in note is closed). The
            // row shape carries neither friendly nor hostile (the 185
            // convention — the spawn sites own the flags; the port's
            // `spawn_with_data` defaults friendly true).
            TRAP_SPEAR => (10.0, 14.0, 37, -1, 300, 0, false, false, true, RESET_STATE),
            // `:2741-2751` — the Flamethrower Trap's jet (187): 6x6,
            // aiStyle **38**, alpha 255, tileCollide false, ignoreWater
            // (the overlay below), timeLeft 60, trap; the stock penetrate
            // 1 (`:774` — the arm writes no override). NEITHER friendly
            // nor hostile in the C# — the spawn site holds friendly false
            // and the damage walks refuse the kind (the dungeon-debris
            // precedent); the jet's damage rides its 188 children.
            FLAMETHROWER_TRAP => (
                6.0,
                6.0,
                38,
                1,
                60,
                0,
                false,
                false,
                true,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `:2570-2578` (171/505/506) / `:5496-5504` (475) — the rope
            // coils: 14x14, aiStyle **35**, penetrate -1, tileCollide
            // false, timeLeft 400. The stock friendly false (`:788`) —
            // the coils damage nothing; the placement walk is the payload.
            ROPE_COIL | VINE_ROPE_COIL | SILK_ROPE_COIL | WEB_ROPE_COIL => (
                14.0,
                14.0,
                35,
                -1,
                400,
                0,
                false,
                false,
                false,
                RESET_STATE,
            ),
            // `:2426-2433` — the Beach Ball (155): 44x44, aiStyle **32**,
            // friendly, netImportant (the overlay below); the stock
            // penetrate 1 / tileCollide true / timeLeft 3600 (the AI pins
            // timeLeft = 10 every tick, `:26889`).
            BEACH_BALL => (44.0, 44.0, 32, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `:2779-2790` — the Mechanical Piranha (190): 22x22, aiStyle
            // **39**, friendly, penetrate -1, alpha 255, ranged (the
            // overlay below), the LOCAL-immunity 14 pair (`:2788-2789`);
            // the stock tileCollide true / timeLeft 3600 (the arm flips
            // tileCollide false in the chase/return states).
            MECHANICAL_PIRANHA => (
                22.0,
                22.0,
                39,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (true, 14, false, -1, 255, 1.0),
            ),
            // The wiring.device arms (M7-WIRE-2): the explosives 141's
            // explosion (`Projectile.cs:1953-1965` — aiStyle 16, 260×260,
            // `timeLeft` 2, `tileCollide = false`, trap), the land mine's
            // (`:2524-2535` — aiStyle 16, 128×128, `timeLeft` 2, trap-less),
            // and the cannon ammo 178 (`:2641-2649` — aiStyle 1, 10×10,
            // `timeLeft` 2, penetrate −1).
            EXPLOSIVES_BLAST => (260.0, 260.0, 16, -1, 2, 0, false, false, true, RESET_STATE),
            LANDMINE_BLAST => (128.0, 128.0, 16, -1, 2, 0, false, false, false, RESET_STATE),
            CANNON_BOMB => (10.0, 10.0, 1, -1, 2, 0, true, false, false, RESET_STATE),
            // PROJ-171 retire cross-note (d6-lane, NOT fixed here): 601 is
            // C# `PortalGunBolt` (`ProjectileID.cs:1756`), aiStyle 1
            // (`Projectile.cs:6506`) — NOT cannon ammo. This M7-WIRE-2 row
            // (blame 85e1d416e) carries penetrate -1 / alpha 0 where the true
            // C# 601 keeps stock penetrate 1 (`:774`) + alpha 255 (`:6507`).
            // The damage gate refuses 601 faithfully (`:12448`); the teleport
            // AI (`PortalHelper.TryPlacingPortal` `:18099`) is unported. The
            // row-shape fix + teleport port are the d6 wiring lane's — see
            // ANALYSIS "W4-d18 Damage_CanDealDamage" (the 601 finding).
            // W4-d6 retire (the d18 `dd7b1b0a` finding, fixed here): 601 is
            // C# `PortalGunBolt` (`ProjectileID.cs:1756`), aiStyle 1
            // (`Projectile.cs:6506`) — NOT cannon ammo. The M7-WIRE-2 row
            // (blame 85e1d416e) carried penetrate −1 / alpha 0 where the
            // true C# 601 keeps the stock penetrate 1 (`:774`) + alpha 255
            // (`:6507`); `friendly = true` (`:6508`) rides the spawn seam's
            // owner argument like every aiStyle-1 row. The stock 3600
            // `timeLeft` (`:787`) + `tileCollide` (`:775`) + extraUpdates 30
            // (`:6509`) stay. The d18 damage gate refuses 601 faithfully
            // (`:12448`); the teleport AI (`PortalHelper.TryPlacingPortal`
            // `:18099`) is unported (the const doc's remainder).
            CANNON_EXPLOSIVE => (
                10.0,
                10.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                30,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
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
            // `:8312-8324` — the Bouncing Shield (866, W4-f48 lane 20):
            // 30×30, aiStyle 3, friendly, `penetrate = 5`, melee (the
            // census below), `extraUpdates = 1`, `usesLocalNPCImmunity` +
            // `localNPCHitCooldown = -1` (one hit per NPC per throw); the
            // stock 3600 life / tileCollide; `light = 0.15` rides
            // `set_defaults_light`.
            BOUNCING_SHIELD => (
                30.0,
                30.0,
                3,
                5,
                TIME_LEFT_DEFAULT,
                1,
                true,
                false,
                false,
                (true, -1, false, -1, 0, 1.0),
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
            ),
            // W4-d7 wiring — `Projectile.cs:4847-4856` (404 Flairon, the recall
            // HEAD): 26x26, aiStyle 69, friendly, `penetrate = -1`,
            // `alpha = 255`, melee (the overlay below). The stock
            // `timeLeft = 3600` / tileCollide true; NOT the 1058 flail body.
            FLAIRON_HEAD => (
                26.0,
                26.0,
                69,
                -1,
                TIME_LEFT_DEFAULT,
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
            // 103, timeLeft 300, hostile, alpha 255 (`:6159` — W4-d11 row
            // fix: the fade-in starts invisible).
            STARDUST_MARK => (20.0, 20.0, 103, -1, 300, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:6239-6247`'s NebulaEye (the 574): 18x18, aiStyle 102,
            // timeLeft 600, tileCollide false. W4-d11 row fix: the stock
            // penetrate 1 (`:774` — the arm sets none), NOT -1.
            NEBULA_EYE => (18.0, 18.0, 102, 1, 600, 0, false, false, false, RESET_STATE),
            // W4-d11 — `Projectile.cs:6142-6150` (539, the
            // StardustJellyfishSmall): 18x30, aiStyle 102, penetrate -1,
            // timeLeft 600, the stock tileCollide.
            STARDUST_JELLYFISH_SMALL => {
                (18.0, 30.0, 102, -1, 600, 0, true, false, false, RESET_STATE)
            }
            // W4-d11 — `Projectile.cs:6230-6238` (573, the NebulaBolt):
            // 18x30, aiStyle 102, penetrate -1, timeLeft 600.
            NEBULA_BOLT => (18.0, 30.0, 102, -1, 600, 0, true, false, false, RESET_STATE),
            // W4-d11 — `Projectile.cs:6258-6269` (576, the NebulaLaser):
            // 4x4, aiStyle 1, penetrate -1, alpha 255, extraUpdates 2,
            // scale 1.2, timeLeft 600.
            NEBULA_LASER => {
                (4.0, 4.0, 1, -1, 600, 2, true, false, false, (false, -2, false, -1, 255, 1.2))
            }
            // W4-d11 — `Projectile.cs:6161-6171` (565, the
            // BrainOfConfusion): 28x28, aiStyle 104, penetrate -1,
            // tileCollide false, alpha 255, scale 0.8. The stock
            // friendly/hostile false — the burst is a visual marker.
            BRAIN_OF_CONFUSION => {
                (28.0, 28.0, 104, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 0.8))
            }
            // W4-d11 — `Projectile.cs:6185-6205` (567/568, the Spore
            // Sac traps): 14x14 / 16x16, aiStyle 105, penetrate 1,
            // alpha 255, timeLeft 3600, tileCollide false; the shared
            // `UseGroupStaticImmunity(567)` rides the idStatic overlay
            // below.
            SPORE_TRAP => {
                (14.0, 14.0, 105, 1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, true, 10, 255, 1.0))
            }
            SPORE_TRAP_2 => {
                (16.0, 16.0, 105, 1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, true, 10, 255, 1.0))
            }
            // W4-d11 — `Projectile.cs:6206-6218` (569-571, the SporeGas
            // clouds): 32x32, aiStyle 106, penetrate -1, alpha 255,
            // timeLeft 3600, tileCollide false; the shared
            // `UseGroupStaticImmunity(569)` rides the idStatic overlay.
            SPORE_GAS | SPORE_GAS_2 | SPORE_GAS_3 => {
                (32.0, 32.0, 106, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, true, 10, 255, 1.0))
            }
            // W4-d11 — `Projectile.cs:6219-6229` (575, the NebulaSphere):
            // 24x24, aiStyle 107, penetrate -1, tileCollide false,
            // timeLeft 420, alpha 255.
            NEBULA_SPHERE => {
                (24.0, 24.0, 107, -1, 420, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d11 — `Projectile.cs:6450-6460` (596, the
            // DesertDjinnCurse): 8x8, aiStyle 107, alpha 255, timeLeft
            // 180, tileCollide false; the stock penetrate 1 (`:774`).
            DJINN_CURSE => (8.0, 8.0, 107, 1, 180, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d11 — `Projectile.cs:6316-6324` (582, the
            // MechanicWrench): 20x20, aiStyle 109, penetrate -1,
            // MaxUpdates 2 (extraUpdates 1); the stock tileCollide.
            MECHANIC_WRENCH => {
                (20.0, 20.0, 109, -1, TIME_LEFT_DEFAULT, 1, true, false, false, RESET_STATE)
            }
            // W4-d11 — `Projectile.cs:6340-6348` (584, the
            // NurseSyringeHeal): 10x10, aiStyle 110, scale 0.8,
            // penetrate 3; the stock tileCollide.
            NURSE_SYRINGE_HEAL => {
                (10.0, 10.0, 110, 3, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 0, 0.8))
            }
            // W4-d11 — `Projectile.cs:6107-6119` (536, the MedusaHeadRay):
            // 10x10, aiStyle 101, penetrate -1, tileCollide false;
            // drawLayer 7 / usesOwnerLight are render-side (dropped).
            MEDUSA_HEAD_RAY => {
                (10.0, 10.0, 101, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)
            }
            // W4-d11 — `Projectile.cs:6397-6408` (591, the
            // MinecartMechLaser): 8x8, aiStyle 101, penetrate -1,
            // tileCollide false; drawLayer 7 / usesOwnerLight dropped.
            MINECART_MECH_LASER => {
                (8.0, 8.0, 101, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)
            }
            // `:6754-6766`'s TowerDamageBolt (the 629): 8x8, aiStyle 122,
            // ignoreWater, extraUpdates 2.
            TOWER_DAMAGE_BOLT => (8.0, 8.0, 122, -1, TIME_LEFT_DEFAULT, 2, false, false, true, RESET_STATE),
            // `:6282-6293`'s ritual markers (the 578 lightning + the 579
            // portal + the 813 Blood Nautilus tears, W4-g64/d11/d13): 32x32,
            // portal + the 813 Blood Nautilus tears, W4-d11/g64/d13): 32x32,
            // aiStyle 108, friendly AND hostile, alpha 255, tileCollide
            // false, ignoreWater, drawLayer 1 (a draw-side field, no pool
            // consumer). `penetrate` keeps the stock 1 (`:774` — the block
            // never sets it); the row shape carries neither friendly nor
            // hostile (the data-row deviation census). W4-g64 remainder 4:
            // the d13 a2588cd2 residual CORRECTED the d11 row shape (the
            // stock pen 1 + alpha 255 — the d11 -1/RESET_STATE was stale);
            // this branch takes the corrected shape for all three kinds.
            VORTEX_LIGHTNING | VORTEX_PORTAL | BLOOD_NAUTILUS_TEARS => {
                (32.0, 32.0, 108, 1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-g64 (the d11 e0395afc row port) — `Projectile.cs:6450-6460`
            // (596, the DesertDjinnCurse): 8x8, aiStyle 107, alpha 255,
            // timeLeft 180, tileCollide false; the stock penetrate 1
            // (`:774`). The 575 NebulaSphere row stays d11's (its
            // ai_nebula_sphere machine covers both at the merge keep-one).

            // the g64/d13 corrected shape adopted for all three kinds — the
            // old d11 `-1`/RESET_STATE (alpha 0) was stale; this tuple is
            // BYTE-EQUAL to g64 `3cdc6423` / d13 `b56db900` so the three-way
            // 813 keep-one merges clean (g64's dust-wired machine survives).

            // `:6294-6305`'s VortexLightning (the 580): 14x14, aiStyle 88,
            // extraUpdates 4, ignoreWater.
            VORTEX_RITUAL_BOLT => (14.0, 14.0, 88, -1, 600, 4, true, false, false, RESET_STATE),
            // `Projectile.cs:5392-5403` — the CultistBossLightningOrbArc
            // (466): the 580 shape (14x14, aiStyle 88, extraUpdates 4,
            // ignoreWater, tileCollide true, timeLeft 600) PLUS `alpha =
            // 255` (`:5398`). W4-d53 batch 3: the row lands for the
            // TrailingMode-1 ring + the stationary trail dust; the AI_088
            // 466 branch stays unported (the dispatch's `ai_vortex_bolt`
            // is the 580 machine — the kind-gated follow-up).
            CULTIST_LIGHTNING_ORB_ARC => (
                14.0, 14.0, 88, -1, 600, 4, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            PHANTASMAL_EYE => (14.0, 14.0, 82, -1, 600, 0, true, false, false, RESET_STATE),
            PHANTASMAL_SPHERE => (40.0, 40.0, 83, -1, 600, 0, false, false, false, RESET_STATE),
            PHANTASMAL_DEATHRAY => (36.0, 36.0, 84, -1, 600, 0, false, false, false, RESET_STATE),
            MOON_LEECH => (16.0, 16.0, 85, -1, 600, 0, false, false, false, RESET_STATE),
            // W4-d9 — `Projectile.cs:5229-5239`: the InfluxWaver (451): 16x16,
            // aiStyle 81, melee, penetrate 3, alpha 255, friendly. The stock
            // tileCollide true (`:775`) / timeLeft 3600 (`:787`) stand; the
            // light 0.2 rides `set_defaults_light`.
            INFLUX_WAVER => (16.0, 16.0, 81, 3, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d9 — `Projectile.cs:5351-5361`: the CultistBossIceMist (464):
            // 60x60, aiStyle 86, hostile, tileCollide false, penetrate −1,
            // alpha 255, ignoreWater. The stock timeLeft 3600 (`:787`) stands.
            CULTIST_BOSS_ICE_MIST => (60.0, 60.0, 86, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d9 — `Projectile.cs:5572-5586`: the ClingerStaff (482): 16x200,
            // aiStyle 87, friendly, tileCollide false, penetrate −1, alpha 255,
            // ignoreWater, timeLeft 18000, magic, idStatic-10.
            CLINGER_STAFF => (16.0, 200.0, 87, -1, 18000, 0, false, false, false, (false, -2, true, 10, 255, 1.0)),
            // W4-d9 — `Projectile.cs:5655-5666`: the CultistRitual (490): 14x14,
            // aiStyle 89, hostile, alpha 255, ignoreWater, tileCollide false,
            // timeLeft 600, netImportant. The stock penetrate 1 (`:774`) stands.
            CULTIST_RITUAL => (14.0, 14.0, 89, 1, 600, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d9 — `Projectile.cs:5645-5654`: the MagicLantern (492): 18x32,
            // aiStyle 90, friendly, penetrate −1, timeLeft ×5 = 18000,
            // netImportant. The stock tileCollide true (`:775`) stands.
            MAGIC_LANTERN => (18.0, 32.0, 90, -1, 5 * TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
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
            // `DD2ElderWins` (`Projectile.cs:7196-7207`): the victory
            // crystal's destruction marker — 10x10, aiStyle 132, alpha 255,
            // tileCollide false, ignoreWater, timeLeft 900, penetrate -1.
            // W4-d14.
            DD2_ELDER_WINS => (
                10.0,
                10.0,
                132,
                -1,
                900,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `DD2OgreSmash` / `QueenSlimeSmash` (`Projectile.cs:7282-7293` /
            // `:8745-8756`): 30x30, aiStyle 135, alpha 255, tileCollide
            // false, ignoreWater, timeLeft 120, penetrate -1 — the two rows
            // are byte-identical. W4-d14.
            OGRE_SMASH | QUEEN_SLIME_SMASH => (
                30.0,
                30.0,
                135,
                -1,
                120,
                0,
                false,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `MonkStaffT1` (`Projectile.cs:7366-7384`): 16x16, aiStyle 140,
            // friendly+melee, scale 1.35, penetrate -1, alpha 255,
            // tileCollide false, ignoreWater, the local-immunity 12 pair.
            // W4-d14.
            MONK_STAFF_T1 => (
                16.0,
                16.0,
                140,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                false,
                false,
                false,
                (true, 12, false, -1, 255, 1.35),
            ),
            // `DD2DarkMageRaise` / `DD2DarkMageHeal` (`Projectile.cs:7208-7219`):
            // 10x10, aiStyle 133, `alpha = 255`, `timeLeft = 900`,
            // `penetrate = -1`, no tile collision, water-ignoring. The 674
            // heal marker shares the row verbatim (the `||` group); W4-d14.
            DARK_MAGE_MARKER | DARK_MAGE_HEAL => (
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
            // Betsy's breath 687 (`Projectile.cs:7315-7324`): 30x30, aiStyle
            // 136 (`AI_136_BetsyBreath` `:68789-68852`), alpha 255, penetrate
            // -1, hostile. The stock `tileCollide = true` (`:775`),
            // extraUpdates 0 and timeLeft 3600 (`:787`) stand — the arm
            // writes NO override. W4-d14 / f19 F5: the old 12x12 aiStyle-1
            // extraUpdates-1 row deviated on every axis.
            BETSY_BREATH => (
                30.0,
                30.0,
                136,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // Betsy's fireball 686 (`Projectile.cs:7305-7314`): 30x30,
            // aiStyle 1 (the slice-B `AI_001` ladder), alpha 255, penetrate
            // -1, hostile. The stock tileCollide/timeLeft/extraUpdates stand.
            // W4-d14 / f19 F5: the box was 22x22 and the alpha reset 0.
            BETSY_FIREBALL => (
                30.0,
                30.0,
                1,
                -1,
                TIME_LEFT_DEFAULT,
                0,
                true,
                false,
                false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // W4-d80 — the Phantom Phoenix shot (706, `Projectile.cs:7461-
            // 7473`): 66x66, aiStyle 1, friendly, alpha 255, timeLeft 300,
            // ranged, penetrate -1, usesLocalNPCImmunity + cooldown 10.
            PHANTOM_PHOENIX => (
                66.0,
                66.0,
                1,
                -1,
                300,
                0,
                true,
                false,
                false,
                (true, 10, false, -1, 255, 1.0),
            ),
            // W4-d80 — the Sky Dragon's Fury shot (709, `Projectile.cs:7508-
            // 7520`): 12x12, aiStyle 1, friendly, melee, alpha 255,
            // ignoreWater, extraUpdates 1, scale 0.6.
            SKY_DRAGONS_FURY_SHOT => (
                12.0,
                12.0,
                1,
                1,
                TIME_LEFT_DEFAULT,
                1,
                true,
                false,
                true,
                (false, -2, false, -1, 255, 0.6),
            ),
            // W4-d80 — the RC car (1105, `Projectile.cs:10294-10302`): 26x26,
            // aiStyle 205, friendly, tileCollide true (the slope-collision
            // move). The stock penetrate 1.
            REMOTE_CONTROL_CAR => (26.0, 26.0, 205, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-d80 — Jim's drone (1020, `Projectile.cs:9659-9666`): 12x12,
            // aiStyle 195, friendly, penetrate -1, stock tileCollide true
            // (the speed-gated bounce `:18057-18067`).
            JIMS_DRONE => (12.0, 12.0, 195, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
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
            // W4-d1 SEAL — `Projectile.cs:9340-9353`: the SilverBullet
            // (981): 4x4, aiStyle 1, friendly, `penetrate = 1`,
            // `light = 0.5f` (the `set_defaults_light` row), `alpha = 255`,
            // `scale = 1.2f`, `timeLeft = 600`, ranged, `extraUpdates = 1`.
            // The stock `tileCollide = true` (`:775`) stands — the shot
            // dies on tile contact (the blocked-move default, the
            // `move_bullet` stand-in). The e3/d75-flagged unrowed sibling.

            // `:1162-1173` — the flamelash: 32x32, aiStyle 9, `penetrate =
            // 2`, the local-NPC immunity with the 12-tick cooldown.
            FLAMELASH => (32.0, 32.0, 9, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 12, false, -1, 0, 1.0)),
            // `:1207-1216` — the harpy feather (the Harpy 48's hostile shot,
            // `NPC.cs:23428`): 14x14, aiStyle 1 (the arm writes 0 then 1 —
            // the final 1 stands), hostile, `penetrate = -1`, tileCollide;
            // the spawn site overrides `timeLeft = 300` (`NPC.cs:23429`).
            HARPY_FEATHER => (14.0, 14.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-d1 SEAL — `Projectile.cs:5404-5412`: the FlyingKnife
            // (491): 26x26, aiStyle 9, friendly, melee, `penetrate = -1`.
            // The stock 3600 life (`:787`) / tileCollide true (`:775`) /
            // extraUpdates 0 / alpha 0 / scale 1 stand — the arm writes NO
            // override. The `AI_009_MagicMissiles` type-491 head
            // (`:43197-43201`) delegates to the LIVE `_Old` machine
            // (`ai_magic_missile_old`); the e3 RETIRE census's d1 task
            // seed (research/125 §2).
            FLYING_KNIFE => (26.0, 26.0, 9, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `:1271-1282` — the demon sickle (the Demon/VoodooDemon's
            // hostile shot, `NPC.cs:23449`): 48x48, aiStyle 18, alpha 100,
            // `penetrate = -1`, `scale = 0.9f`; the spawn site overrides
            // `timeLeft = 300` (`NPC.cs:23450`).
            DEMON_SICKLE => (48.0, 48.0, 18, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 100, 0.9)),
            // `:1207-1216` — the harpy feather (the Harpy's hostile shot,
            // `NPC.cs:23428`, W4-d23): 14x14, aiStyle 1 (the `:1214` write
            // over `:1211`'s 0), `penetrate = -1`, the stock 3600 life and
            // `tileCollide = true`; the spawn site overrides `timeLeft =
            // 300` (`NPC.cs:23429`). 38 IS on the `flag3 = false` list
            // (`:55953`) — the flight counter rests, no gravity: the
            // feather flies dead straight (`ai_style1_rider`'s resting
            // flight; the AI_STYLE_1_RIDERS gate carries the kind).

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
            // W4-g2 (d5 F5) — `:3255-3262`: the Blood Rain's flying cloud:
            // 28x28, aiStyle 45, `penetrate = -1`, `netImportant` — the 237
            // twin verbatim (the stock tileCollide true / timeLeft 3600).
            BLOOD_CLOUD_MOVING => (28.0, 28.0, 45, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-g2 (d5 F5) — `:3263-3273`: the settled blood cloud: 54x28,
            // aiStyle 45, `tileCollide = false`, `ignoreWater`,
            // `penetrate = -1`, `timeLeft = 18000`, `netImportant` — the 238
            // twin; its rain cadence 10 / cap 1 ride the arm's kind match.
            BLOOD_CLOUD_RAINING => (54.0, 28.0, 45, -1, 18000, 0, false, false, false, RESET_STATE),
            // W4-g2 (d5 F5) — `:3274-3288`: the blood drop: 4x40, aiStyle
            // 45, friendly, `penetrate = 2` (the 239's 5 does NOT carry
            // over), `timeLeft = 300`, `scale = 1.1f`, magic,
            // `extraUpdates = 1`, `ignoreWater`, the id-static 10 pair. The
            // arm pins alpha 100 (`:28099`).
            BLOOD_RAIN => (4.0, 40.0, 45, 2, 300, 1, true, false, false, (false, -2, true, 10, 0, 1.1)),
            // W4-g2 (d5 F5) — `:3491-3502`: the hostile rain-cloud drop:
            // 4x40, aiStyle 45, hostile, `penetrate = -1`, `timeLeft = 120`,
            // `scale = 1.1f`, `extraUpdates = 1`, `ignoreWater`. No class
            // flag, no immunity pair — the arm pins rotation (`:28103`).
            RAIN_NIMBUS => (4.0, 40.0, 45, -1, 120, 1, true, false, false, (false, -2, false, -1, 0, 1.1)),
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
            // W4-g2 (d5 F4) — `:3804-3814`: the Diabolist's bolt: 12x12,
            // aiStyle 50, hostile, alpha 255, magic, `tileCollide = false`,
            // `penetrate = -1`. The stock timeLeft 3600 (`:787`) stands.
            INFERNO_HOSTILE_BOLT => (12.0, 12.0, 50, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-g2 (d5 F4) — `:3815-3825`: the bolt's on-death blast:
            // 130x130 (the friendly 296's 150 does NOT carry over), aiStyle
            // 50, hostile, alpha 255, magic, `tileCollide = false`,
            // `penetrate = -1`.
            INFERNO_HOSTILE_BLAST => (130.0, 130.0, 50, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-g2 (d5 F1) — `:3793-3803`: the Necromancer's beam: 4x4,
            // aiStyle 48, hostile, magic, `extraUpdates = 100`,
            // `timeLeft = 100`, `penetrate = -1`. The delay table's 290→3
            // arm (`:53991`) rides `ai_magnet_bolt`.
            SHADOWBEAM_HOSTILE => (4.0, 4.0, 48, -1, 100, 100, true, false, false, RESET_STATE),
            // W4-g2 (d5 F1) — `:5037-5046`: the UFO minion's laser: 8x8,
            // aiStyle 48, friendly, `extraUpdates = 100`, `timeLeft = 100`,
            // `ignoreWater`. The stock penetrate 1 (`:774`) stands — the
            // laser dies on its first hit; the delay table's default 0
            // (`:53997-53998`) is correct for it.
            UFO_LASER => (8.0, 8.0, 48, 1, 100, 100, true, false, false, RESET_STATE),
            // `:4451-4463` — the venom fang: 12x12, aiStyle 1 (the rider
            // hook), alpha 255, `penetrate = 5`, the local immunity with
            // the 10-tick cooldown, `timeLeft = 58`.
            VENOM_FANG => (12.0, 12.0, 1, 5, 58, 0, true, false, false, (true, 10, false, -1, 255, 1.0)),
            // `:4497-4507` — the frost staff's bolt: 14x14, aiStyle 28,
            // alpha 255, `penetrate = 2` (coldDamage — no slot field yet).
            FROST_BOLT => (14.0, 14.0, 28, 2, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:2180-2190` — the Rune Wizard's RuneBlast (129, W4-g5):
            // 14x14, aiStyle 28, alpha 255, `penetrate = -1`, hostile,
            // `tileCollide = false`. The stock 3600 life (the NPC's shoot
            // overrides it to 300, `NPC.cs:21272`). The flight arm is
            // `ai_frost_bolt` (the shared aiStyle-28 first-tick Item28 cue
            // + the 129's 6-dust Next(4)/Next(5) block).
            RUNE_BLAST => (14.0, 14.0, 28, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:10186-10194` — the Librarian Skeleton's book (1092, W4-g5):
            // 16x16, aiStyle 18, hostile, `penetrate = -1`, `timeLeft =
            // 420`. The stock `tileCollide = true` (`:775`) stands. The
            // flight arm is `ai_librarian_book` (the 45-tick hover, the
            // aim turn, the 9-speed home, the page-gore trail).
            LIBRARIAN_BOOK => (16.0, 16.0, 18, -1, 420, 0, true, false, false, RESET_STATE),
            // W4-d36 E4 batch 5 — `Projectile.cs:2621-2629`: the Spiked
            // Jungle Slime's spike (176): 6x6, aiStyle 1, hostile,
            // `penetrate = -1`, alpha 255, the stock tileCollide true /
            // 3600 life. The `:55109-55128` alpha −50 fade + the Item17
            // first-tick cue ride the aiStyle-1 rider hook below.
            JUNGLE_SPIKE => (6.0, 6.0, 1, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d36 E4 batch 5 — `Projectile.cs:2630-2640`: the Icy
            // Merman's spit (177): 10x10, aiStyle 28 (the frost-bolt
            // flight), hostile, `penetrate = -1`, alpha 255, coldDamage
            // (no slot field yet — the module doc), the stock 3600 life.
            ICEWATER_SPIT => (10.0, 10.0, 28, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
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
            // W4-d6 — `:3883-3892` SpiritHeal (298): 6x6, aiStyle 52,
            // alpha 255, magic, `tileCollide = false`, `extraUpdates = 3`.
            // `friendly` stays the reset false (`:788` — the heal orb hits
            // nothing); the stock penetrate 1 / timeLeft 3600 stand.
            SPIRIT_HEAL => (6.0, 6.0, 52, 1, TIME_LEFT_DEFAULT, 3, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d6 residual 5 — `:3952-3964` VampireKnife (304): 30x30,
            // aiStyle 2, friendly, melee, `penetrate = 1`, alpha 255,
            // light 0.2 (`set_defaults_light` below), `ignoreWater` (the
            // overlay below), `extraUpdates = 0`. The stock tileCollide
            // true / timeLeft 3600 stand. The aiStyle-2 arm's spawn latch
            // (`:23240-23244`) + 30-tick decay (`:23632-23649`) were already
            // ported kind-gated; this row arms them.
            VAMPIRE_KNIFE => (30.0, 30.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d6 — `:3965-3973` VampireHeal (305): 6x6, aiStyle 52,
            // alpha 255, `tileCollide = false`, `extraUpdates = 10`. NO
            // magic flag (the C# arm sets none — the heal is classless).
            VAMPIRE_HEAL => (6.0, 6.0, 52, 1, TIME_LEFT_DEFAULT, 10, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d6 — `:4139-4148` FlamingJack (321): 30x30, aiStyle 55,
            // friendly, melee, `tileCollide = false`, `ignoreWater` (the
            // overlay below). The stock penetrate 1 / timeLeft 3600 stand.
            FLAMING_JACK => (30.0, 30.0, 55, 1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // W4-d6 — `:4215-4225` FlamingScythe (329): 80x80, aiStyle 56,
            // hostile (the spawn_hostile seam), `penetrate = -1`,
            // `tileCollide = false`, `timeLeft = 420`, `light = 0.25f`
            // (`set_defaults_light` below). ai0/ai1 carry the blade's
            // rotation/spriteDirection (`NPC.cs:33230`'s trailing args).

            // W4-d6 — `:4487-4496` WaterGun (358): 18x18, aiStyle 60,
            // alpha 255, `penetrate = -1`, `extraUpdates = 2`, `ignoreWater`
            // (the overlay below); the stock tileCollide true — the 1x1
            // collision probe (`GetCollisionParams`, `:19872-19876`) rides
            // the arm's movement stand-in.
            WATER_GUN => (18.0, 18.0, 60, -1, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d6 — `:4868-4877` SlimeGun (406): 14x14, aiStyle 60,
            // alpha 255, `penetrate = -1`, `extraUpdates = 2`, ignoreWater.
            SLIME_GUN => (14.0, 14.0, 60, -1, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d6 — `:9848-9857` ShimmerGunStream (1042): 18x18, aiStyle
            // 60, alpha 255, `penetrate = -1`, `extraUpdates = 2`,
            // ignoreWater.
            SHIMMER_GUN_STREAM => (18.0, 18.0, 60, -1, TIME_LEFT_DEFAULT, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:4903-4915` Typhoon — 30x30, aiStyle 71, `penetrate = -1`,
            // alpha 255, `timeLeft = 300`, `extraUpdates = 2`, `ignoreWater`
            // dropped.
            TYPHOON => (30.0, 30.0, 71, -1, 300, 2, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d12 — `:6358-6366` DryadsWardCircle — 26x26, aiStyle 111,
            // alpha 255, friendly, `penetrate = -1`; the stock 3600 life
            // (`:787`) and tileCollide (`:775`) stand — the AI's 570-tick
            // kill (`:41236-41239`) ends the ring long before either.
            DRYADS_WARD_CIRCLE => (26.0, 26.0, 111, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `:6644-6657` NebulaArcanum — 32x32, aiStyle 118, alpha 255,
            // `penetrate = 3`, local-immunity with the 8-tick cooldown
            // (`ignoreWater`/`drawLayer` dropped).
            NEBULA_ARCANUM => (32.0, 32.0, 118, 3, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 8, false, -1, 255, 1.0)),
            // `:6658-6670` NebulaArcanumSubshot — 18x30, aiStyle 119,
            // `tileCollide = false`, `penetrate = -1`, `timeLeft = 420`,
            // local-immunity 20; alpha at the 0 reset.
            NEBULA_ARCANUM_SUBSHOT => (18.0, 30.0, 119, -1, 420, 0, false, false, false, (true, 20, false, -1, 0, 1.0)),
            // `:6681-6690` NebulaArcanumExplosionShotShard — 8x8, aiStyle 29
            // (the per-kind shard arm), alpha 255, `penetrate = 1`, magic,
            // friendly; the stock tileCollide true / 3600 life (the 40-tick
            // ai[1] counter kills it first; `:774-775`/`:787` resets).
            // S1-E-21h d77 + W4-d78 batch 2 (the merge dedupe — one row,
            // both lanes' constants alias kind 620): the 617's Kill scatter
            // (`:72354`/`:72370`) spawns it with `ai[0]` = the 86/90 dust
            // pick; the AI_029 kind-gate reads it as the dust id only.
            NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD => (
                8.0, 8.0, 29, 1, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
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
            // W4-d10 (f19 F1) — the counterweight family
            // (`Projectile.cs:6084-6093`): 10×10, `extraUpdates = 1`,
            // `DefaultToYoyo()` (aiStyle 99, melee, penetrate −1),
            // `drawLayer = 0`, `scale = 1f`, `counterweight = true` (no
            // port field — the draw path knows the kinds, assets.rs:706).
            // The AI is the style-99 SUB-arm `AI_099_1_Counterweights`.
            BLACK_COUNTERWEIGHT | BLUE_COUNTERWEIGHT | GREEN_COUNTERWEIGHT
            | PURPLE_COUNTERWEIGHT | RED_COUNTERWEIGHT | YELLOW_COUNTERWEIGHT
            | PINK_COUNTERWEIGHT => (10.0, 10.0, 99, -1, TIME_LEFT_DEFAULT, 1, true, false, false, RESET_STATE),
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
            // W4-d12 DOC-SWEEP item 7 — `Projectile.cs:9253-9268` Volcano
            // (978): 160×160, aiStyle 189, friendly, melee, `alpha = 255`,
            // `ignoreWater`, `tileCollide = false`, `ownerHitCheck = true`
            // (no port field — the spear family's gate is the only ported
            // ownerHitCheck consumer, ANALYSIS), `penetrate = 3`,
            // local-immunity −1. The `timeLeft = 60` (`:9261`) is OVERWRITTEN
            // by the second `timeLeft = 30` (`:9268`) — the row keeps 30.
            VOLCANO => (
                160.0,
                160.0,
                189,
                3,
                30,
                0,
                false,
                false,
                false,
                (true, -1, false, -1, 255, 1.0),
            ),
            // `TheEyeOfCthulhuDash` (`:10596-10608`): 100×100, aiStyle 210,
            // `timeLeft = 600`, `penetrate = -1`, local-immunity −1.
            EYE_OF_CTHULHU_DASH => (100.0, 100.0, 210, -1, 600, 0, false, false, false, RESET_STATE),
            // W4-e7 lane 23 — the Snapping Stone strike effects
            // (`Projectile.cs:10491-10503` / `:10611-10623` /
            // `:10624-10635`): 10×10, aiStyle 0 (the `:778` reset — no AI
            // arm), the stock `penetrate = 1` (`:774`), `tileCollide =
            // false`, `timeLeft = 2`, local-immunity 60. The class flags
            // ride the census below (1121 ranged, 1131 melee, 1132 none);
            // `noEnchantments`/`hide` have no port field (draw/enchant-
            // only, the `hide` convention). The strike spawns them at the
            // extra targets (`snapping_stone.rs`, `:14339-14345` — the
            // melee arm flips 1131's class by the owner's crit comparison).
            SNAPPING_STONE_EFFECT | SNAPPING_STONE_MELEE_EFFECT | SNAPPING_STONE_SUMMON_EFFECT => {
                (10.0, 10.0, 0, 1, 2, 0, false, false, false, (true, 60, false, -1, 0, 1.0))
            }
            // `CrystalShardMelee` (`:10636-10650`): 6×6, aiStyle 24,
            // `penetrate = 1`, `timeLeft = 600`, idStatic-20.
            CRYSTAL_SHARD_MELEE => (6.0, 6.0, 24, 1, 600, 0, false, false, false, RESET_STATE),
            // `KrakenWave` (`:10656-10667`): 100×100, aiStyle 210,
            // `timeLeft = 600`, `penetrate = -1`, idStatic-10.
            KRAKEN_WAVE => (100.0, 100.0, 210, -1, 600, 0, false, false, false, RESET_STATE),
            // W4-d21 — `Projectile.cs:10095-10104` (1082, the Antlion Claw):
            // 26x26, aiStyle 201, friendly, penetrate 1, melee,
            // usesOwnerMeleeHitCD (no port field — the per-hit melee gate is
            // the damage_pve seam's, recorded). The stock tileCollide/timeLeft.
            ANTLION_CLAW => (26.0, 26.0, 201, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:10105-10114` (1083, the Stylist scissors):
            // 28x28, aiStyle 201, friendly, penetrate 2, melee.
            STYLIST_SCISSORS => (28.0, 28.0, 201, 2, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:10115-10124` (1084, the Torch God helper):
            // 4x4, aiStyle 202, extraUpdates 2, timeLeft 300, penetrate -1,
            // tileCollide false.
            TORCH_GOD_HELPER => (4.0, 4.0, 202, -1, 300, 2, false, false, false, RESET_STATE),
            // `Projectile.cs:10175-10185` (1091, the Storm Lightning trap):
            // 16x16, aiStyle 203, friendly + hostile, trap, ignoreWater,
            // netImportant. The hostileDamageScaling is the difficulty seam
            // (single-difficulty, recorded). The stock penetrate 1/timeLeft.
            STORM_LIGHTNING => (16.0, 16.0, 203, 1, TIME_LEFT_DEFAULT, 0, true, false, true, RESET_STATE),
            // `Projectile.cs:10427-10436` (1117, the Lightning Strike shot):
            // 16x16, aiStyle 203, friendly, magic, ignoreWater, tileCollide
            // false. The stock penetrate 1/timeLeft.
            LIGHTNING_STRIKE_SHOT => (16.0, 16.0, 203, 1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:10437-10449` (1122, the Arc Surge): 16x16,
            // aiStyle 203, friendly, magic, ignoreWater, tileCollide false,
            // timeLeft 1, idStatic-10.
            ARC_SURGE => (16.0, 16.0, 203, 1, 1, 0, false, false, false, (false, -2, true, 10, 0, 1.0)),
            // `Projectile.cs:10241-10250` (1098, the Digtoise): 34x34,
            // aiStyle 204, friendly, alpha 255, netImportant,
            // correctSlopeCollision (the slope pass rides the dispatch).
            DIGTOISE => (34.0, 34.0, 204, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:10504-10513` (1123, the Trusty Digtoise): the
            // same 34x34 aiStyle-204 row.
            TRUSTY_DIGTOISE => (34.0, 34.0, 204, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:10294-10302` (1105, the RC Car): 26x26,
            // aiStyle 205, friendly, tileCollide true, correctSlopeCollision.

            // `Projectile.cs:10401-10412` (1115, the Glacier Fang shot):
            // 10x10, aiStyle 208, friendly, magic, ignoreWater, drawLayer 4
            // (draw-only), the LOCAL pair (-1).
            GLACIER_FANG_SHOT => (10.0, 10.0, 208, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (true, -1, false, -1, 0, 1.0)),
            // `Projectile.cs:10580-10596` (1129, the Chlorophyte Claymore
            // blade slam): 16x16, aiStyle 209, alpha 255, tileCollide false,
            // ignoreWater, timeLeft 5, friendly, penetrate -1, melee, the
            // LOCAL pair (-1), ownerHitCheck (the spear seam's, recorded).
            CHLOROPHYTE_CLAYMORE_BLADE_SLAM => (16.0, 16.0, 209, -1, 5, 0, false, false, false, (true, -1, false, -1, 255, 1.0)),
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
            // W4-f20 F3 — the Sky Dragon's Fury circle 1110
            // (`Projectile.cs:10340-10352`): 80x80, aiStyle 77, friendly,
            // alpha 255, scale 1, melee, ignoreWater, tileCollide false,
            // penetrate -1; the `NewProjectileSetup` 300-tick seed
            // (`:11065-11068`, the 443 sibling). The 709's Kill spawns it
            // (`:73384`) and overrides timeLeft to `30 * Next(2,6)`.
            SKY_DRAGONS_FURY_ELECTROSPHERE => (80.0, 80.0, 77, -1, 300, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
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
            // W4-g8 lane 1 batch 1 — the Antlion's SandBallFalling (31,
            // `Projectile.cs:1131-1140`): 10x10, aiStyle 10, penetrate −1,
            // the stock tileCollide; the `ai[0] = 2` + `timeLeft = 300`
            // overrides ride the NPC shot drain (`NPC.cs:24463-24464`).
            SAND_BALL_FALLING => (10.0, 10.0, 10, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-g8 lane 1 batch 2 — the Gastropod's PinkLaser (84,
            // `Projectile.cs:1646-1659`): 4x4, aiStyle 1, penetrate 3,
            // extraUpdates 2, timeLeft 600; the IceElemental's FrostBlast
            // (128, `:2169-2179`): 14x14, aiStyle 28, penetrate −1, the
            // stock tileCollide + 3600 life.

            FROST_BLAST_HOSTILE => (14.0, 14.0, 28, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-g8 lane 1 batch 2 — the wall-crawler's WebSpit (472,
            // `Projectile.cs:5463-5473`): 8x8, aiStyle 1, penetrate −1,
            // tileCollide, timeLeft 60 (the expert spit, constants-off).
            WEB_SPIT => (8.0, 8.0, 1, -1, 60, 0, true, false, false, RESET_STATE),
            // W4-d1 wave 3 — the falling-block family (`AI_010`). The
            // DirtBall 17 (`Projectile.cs:985-992`): 10x10, aiStyle 10,
            // friendly, ignoreWater (the overlay below); the stock
            // penetrate 1 (`:774`) / 3600 life / tileCollide true stand.
            DIRT_BALL => (10.0, 10.0, 10, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // The natural falls (`Projectile.cs:1131-1140` 31, `:1217-1226`
            // 39, `:1227-1236` 40, `:1384-1393` 56): 10x10, aiStyle 10,
            // friendly + hostile, `penetrate = -1`; the per-type
            // `knockBack = 6f` is dropped — the spawner's input wins (the
            // sand-row precedent above; `SpawnFallingBlockProjectile`
            // passes KnockBack 0, `WorldGen.cs:86071`). The C# friendly +
            // hostile pair rides the arm's damage gates (the row shape
            // carries neither flag, ANALYSIS).
            MUD_BALL_FALLING | ASH_BALL_FALLING | EBONSAND_BALL_FALLING => {
                (10.0, 10.0, 10, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d1 wave 3b — the remaining natural falls
            // (`Projectile.cs:1467-1476` 67, `:1512-1521` 71, `:2650-2659`
            // 179, `:3228-3237` 241, `:7953-7962` 812): the 31's shape
            // verbatim — 10x10, aiStyle 10, friendly + hostile,
            // `penetrate = -1`, the stock life / tileCollide; the
            // `knockBack = 6f` drop is the sand-row precedent above.
            PEARLSAND_BALL_FALLING | SILT_BALL_FALLING | SLUSH_BALL_FALLING
            | CRIMSAND_BALL_FALLING | SHELL_PILE_FALLING => {
                (10.0, 10.0, 10, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d1 wave 3c — the coin piles 411-414
            // (`Projectile.cs:4933-4941`): 10x10, aiStyle 10 ONLY — the
            // stock friendly false (`:788`) / hostile false (`:764`) /
            // penetrate 1 (`:774`) / 3600 life stand (the arm touches
            // neither flag; the empty switch is a decompilation remnant).
            // The arm holds the C# friendly state and the PVE walk refuses
            // them (`Damage_PVE_Inner` `:12611`, the debris precedent).
            COPPER_COINS_FALLING | SILVER_COINS_FALLING | GOLD_COINS_FALLING
            | PLATINUM_COINS_FALLING => {
                (10.0, 10.0, 10, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d1 wave 3c — the player Mud Ball 1081
            // (`Projectile.cs:10087-10094`): 12x12, aiStyle 10, friendly,
            // `penetrate = -1`; the stock life / tileCollide stand. Its own
            // 20-tick gravity delay (`:57997-58004`) + the side-slide death
            // probe (`:77492-77502`) ride the falling arm.
            MUD_BALL_PLAYER => {
                (12.0, 12.0, 10, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
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
            // `Projectile.cs:924-938` — the natural falling star (12):
            // 18x18, aiStyle 5, `penetrate = -1`, `alpha = 50`, `light = 1f`,
            // `trap = true`; the stock `tileCollide = true` (`:775`) stands
            // (the aiStyle-5 generic latch arms it below the spawn floor).
            // `friendly = true` (the remix hostile flag has no port slot
            // field split — the slot default is friendly). W4-g70 lane 8.
            NATURAL_FALLING_STAR => (
                18.0, 18.0, 5, -1, TIME_LEFT_DEFAULT, 0, true, false, true,
                (false, -2, false, -1, 50, 1.0),
            ),
            // `Projectile.cs:7617-7625` — the starfall spawner (720): 16x16,
            // aiStyle 148, `tileCollide = false`, `penetrate = -1`,
            // `alpha = 255` (invisible). W4-g70 lane 8.
            FALLING_STAR_SPAWNER => (
                16.0, 16.0, 148, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
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
            // W4-d78 batch 2 — `Projectile.cs:10514-10527` the Bone Shard
            // (1124): 6x6, aiStyle 1, friendly, `penetrate = 1`,
            // `timeLeft = 600`, ranged, tileCollide true, `extraUpdates = 1`.
            // The arm writes `scale = 1.2f` (`:10521`) then OVERWRITES it
            // with `scale = 0.95f` (`:10525`) — the row keeps the LAST write
            // (the double-set quirk, verbatim). The 2-frame sheet is
            // `Main.cs:8562`; the 117 Kill arm (`:74748`) spawns it.
            BONE_SHARD => (6.0, 6.0, 1, 1, 600, 1, true, false, false, (false, -2, false, -1, 0, 0.95)),
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
            // ammo and then decline the spawn. (W4-d1 retire: the seven
            // SOLUTION sprays 146-149/1015-1017 are rowed below the 145 —
            // the d18/f25 keep-one, the verbatim alpha-255/ignoreWater
            // shape; the 145's own dropped pair stays its M3-ITEM-6 note.)
            85 => (6.0, 6.0, 193, 4, TIME_LEFT_DEFAULT, 2, true, false, false, RESET_STATE),
            145 => (6.0, 6.0, 31, -1, TIME_LEFT_DEFAULT, 2, false, false, false, RESET_STATE),
            // W4-d18 — the seven solution sprays: `Projectile.cs:2350-2397`
            // (146 Hallowed / 147 Corrupt / 148 Crimson / 149 Mushroom, four
            // byte-identical blocks) and `:9633-9636` → `DefaultToSpray`
            // (`:10691-10702`, 1015 Blue / 1016 Red / 1017 Purple — the
            // TerraToilet solutions). Every block is the 145 shape VERBATIM:
            // 6x6, aiStyle 31, friendly, `alpha = 255`, `penetrate = -1`,
            // `extraUpdates = 2`, `tileCollide = false`, `ignoreWater = true`
            // (the overlay below). Unclaimed on every pushed tip (f25 wired
            // the conversion cells at its `ai_spray` but left the rows — its
            // tip b004f4f1b has only 145; d4's tip the same); the `ai_spray`
            // ladder's conversion types 1-7 stay no-ops here until f25's
            // wiring merges (the counter/cap/spin run for every kind).
            // W4-d1 retire (the spray compose — the d18/f25 DUP keep-one,
            // research/352, registry-150 §9 chain 246→352): the seven
            // solution sprays 146-149 /
            // 1015-1017. `Projectile.cs:2350-2397` — the four 146-149
            // blocks are LINE-IDENTICAL copies of the 145 arm; `:9633-9636`
            // → `DefaultToSpray` (`:10691-10702`) for 1015-1017. Every
            // block: 6x6, aiStyle 31, friendly, `alpha = 255`,
            // `penetrate = -1`, `extraUpdates = 2`, `tileCollide = false`,
            // `ignoreWater = true` (the overlay below), the stock 3600
            // life (the arm caps it at 133), NO class flag, NO immunity
            // pair. The d18 copy (`78ffe8e11`) is the keep-one survivor —
            // it carries the alpha/ignoreWater the f25 copy (`5e83d3ecb`)
            // dropped on the 145-precedent; the f25 rows fold into this
            // one at merge (the ANALYSIS keep-one entry).
            146..=149 | 1015..=1017 => (
                6.0, 6.0, 31, -1, TIME_LEFT_DEFAULT, 2, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
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
            // W4-d7 F3 — the deferred aiStyle-67 pets' rows: the Companion
            // Cube (`Projectile.cs:7012-7023`: 30x30, penetrate -1,
            // netImportant, timeLeft *= 5, friendly, ignoreWater, scale 0.8)
            // and the Rainbow Boulder Pet (`:10163-10174`: 32x32, the same
            // slice but scale 0.5). Both re-pin friendly false per tick
            // (`:49562`), no minion slice (the slice-B pets' convention).
            COMPANION_CUBE => (30.0, 30.0, 67, -1, 18000, 0, true, false, false, (false, -2, false, -1, 0, 0.8)),
            RAINBOW_BOULDER_PET => (32.0, 32.0, 67, -1, 18000, 0, true, false, false, (false, -2, false, -1, 0, 0.5)),
            // W4-d7 F3 — the Cob Whip's spider (`Projectile.cs:9767-9779`):
            // 18x10, aiStyle 67, friendly, penetrate -1, netImportant, the
            // LOCAL immunity pair (20), armorPenetration 30 (dropped — no
            // ProjData field, the spawn-source convention, the 864 precedent),
            // decidesManualFallThrough. The stock timeLeft 3600 (NOT *5 — the
            // row sets no timeLeft) / tileCollide true.
            COB_WHIP_PROJ => (18.0, 10.0, 67, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (true, 20, false, -1, 0, 1.0)),

            // ===== W4-d3: the aiStyle-25 boulder family (census 26b) =====
            // `Projectile.cs:1833-1850` — the shared 31x31 trap block: 99
            // Boulder / 727 RollingCactus / 1013 BouncyBoulder / 1014
            // LifeCrystalBoulder / 1053 LavaBoulder / 1054 SpiderBoulder /
            // 1055 Ghoulder / 1057 BoulderThatSpawnsPet. friendly + hostile
            // (the hostile lane rides `projectile_ai_boulder::hurts_player`),
            // penetrate −1, trap; the 1054 timeLeft 300 (`:1844`), the 1055
            // 900 (`:1848`), the rest the stock 3600.
            BOULDER | ROLLING_CACTUS | BOUNCY_BOULDER | LAVA_BOULDER
            | LIFE_CRYSTAL_BOULDER | BOULDER_THAT_SPAWNS_PET => {
                (31.0, 31.0, 25, -1, TIME_LEFT_DEFAULT, 0, true, false, true, RESET_STATE)
            }
            SPIDER_BOULDER => (31.0, 31.0, 25, -1, 300, 0, true, false, true, RESET_STATE),
            GHOULDER => (31.0, 31.0, 25, -1, 900, 0, true, false, true, RESET_STATE),
            // `Projectile.cs:1851-1860` — the 1005 MiniBoulder: 15x15, the
            // same trap flags (the getGoodWorld explosion's shard).
            MINI_BOULDER => (15.0, 15.0, 25, -1, TIME_LEFT_DEFAULT, 0, true, false, true, RESET_STATE),
            // `Projectile.cs:1861-1870` — the 1021 MoonBoulder: 31x31,
            // hostile-ONLY (no friendly flag — the arm holds the slot's
            // friendly false, the debris precedent), penetrate −1,
            // extraUpdates 1, `timeLeft *= 3` (3600 → 10800), NOT a trap.
            MOON_BOULDER => (31.0, 31.0, 25, -1, TIME_LEFT_DEFAULT * 3, 1, true, false, false, RESET_STATE),
            // `Projectile.cs:7038-7047` — the 655 BeeHive: 31x31, the shared
            // trap flags.
            BEE_HIVE => (31.0, 31.0, 25, -1, TIME_LEFT_DEFAULT, 0, true, false, true, RESET_STATE),
            // `Projectile.cs:9918-9927` — the 1047 RainbowBoulder: 31x31,
            // friendly-only (NOT hostile, NOT trap), penetrate −1,
            // timeLeft 3600, ignoreWater (the tail's `|| kind ==
            // RAINBOW_BOULDER` arm below).
            RAINBOW_BOULDER => (31.0, 31.0, 25, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9928-9937` — the 1048 Poulder: 31x31, the
            // shared trap flags.
            POULDER => (31.0, 31.0, 25, -1, TIME_LEFT_DEFAULT, 0, true, false, true, RESET_STATE),
            // `Projectile.cs:7855-7865` — the 763 RollingCactusSpike (the
            // 727's bounce child): 10x10, hostile + friendly, penetrate −1,
            // aiStyle 1 (the generic arrow flight), tileCollide true,
            // extraUpdates 1.
            ROLLING_CACTUS_SPIKE => (10.0, 10.0, 1, -1, TIME_LEFT_DEFAULT, 1, true, false, false, RESET_STATE),

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
            // W4-a9 — `Projectile.cs:10479-10490` (1120, the Forbidden sign's
            // shot): 10x10, aiStyle 29 (the gem-bolt family's ActuallyGemStaffs
            // tail, `:37836-37846` — dust id 170, ONE unconditional dust, the
            // Item72 cue), alpha 255, friendly, the LOCAL pair (60),
            // `timeLeft = 75`, `extraUpdates = 1`. The 206 arm's fire spawns it
            // with the FastThenSlow feature bit (`:37588-37592`).
            1120 => (10.0, 10.0, 29, 1, 75, 1, true, false, false, (true, 60, false, -1, 255, 1.0)),
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
            // W4-g64 (the d13 314fa6cd row port) — the aiStyle-124 pets this
            // lane's trail arms ride (`AI()` `Projectile.cs:34970-35696`):
            // the Fairy Queen pet (`:8553-8565`), the Ice Queen pet
            // (`:8576-8588`) and the Glommer pet (`:9034-9046`). Every row:
            // 20x20, `penetrate = -1`, `netImportant`, `timeLeft *= 5`
            // (3600 → 18000), `friendly = true` (the pool default),
            // `ignoreWater`, `tileCollide = false`; `manualDirectionChange`
            // is a draw-side field with no pool consumer (the arm writes
            // `direction`/`spriteDirection` itself, `:35007`). The 650/882/
            // 888/896 rows stay d13's (the ladder consts are named above).
            FAIRY_QUEEN_PET | ICE_QUEEN_PET | GLOMMER_PET => {
                (20.0, 20.0, 124, -1, 18000, 0, false, false, false, RESET_STATE)
            }
            // W4-d13 — the aiStyle-124 pets (`AI()` `Projectile.cs:34970-
            // 35696`): the Suspicious Looking Tentacle (`:6977-6989`), the
            // Eye of Cthulhu pet (`:8461-8473`), the Twins pet (`:8508-8520`),
            // the Fairy Queen / Pumpking pair (`:8553-8565`), the Ice Queen
            // pet (`:8576-8588`) and the Glommer pet (`:9034-9046`). Every
            // row: 20x20, `penetrate = -1`, `netImportant`, `timeLeft *= 5`
            // (3600 → 18000), `friendly = true` (the pool default),
            // `ignoreWater`, `tileCollide = false`; `manualDirectionChange`
            // is a draw-side field with no pool consumer (the arm writes
            // `direction`/`spriteDirection` itself, `:35007`).
            SUSPICIOUS_TENTACLE | EOC_PET | TWINS_PET  | PUMPKING_PET
              => {
                (20.0, 20.0, 124, -1, 18000, 0, false, false, false, RESET_STATE)
            }
            // W4-d13 — `Projectile.cs:6990-7000` (651, the Wire Kite): 10x10,
            // aiStyle 125, `friendly = true` (the pool default), `ignoreWater`,
            // `tileCollide = false`, `penetrate = -1`; `drawLayer = 5` is a
            // draw-side field with no pool consumer. The stock 3600 life.
            WIRE_KITE => (10.0, 10.0, 125, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // W4-d13 — `Projectile.cs:7024-7037` (654, the Geyser Trap): 30x30,
            // aiStyle 126, `alpha = 255`, `tileCollide = false`, `ignoreWater`,
            // `timeLeft = 120`, friendly AND hostile, `penetrate = -1`, `trap`.
            // The row shape carries neither friendly nor hostile (the
            // data-row deviation census); the spawn path sets friendly=false.
            GEYSER_TRAP => (30.0, 30.0, 126, -1, 120, 0, false, false, true, (false, -2, false, -1, 255, 1.0)),
            // W4-d13 — `Projectile.cs:7174-7185` (670, the Ogre Stomp): the
            // same 30x30 aiStyle-126 row MINUS friendly/trap — hostile only.
            OGRE_STOMP => (30.0, 30.0, 126, -1, 120, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d13 — `Projectile.cs:7048-7060` (656, the friendly Sandnado):
            // 10x10, aiStyle 127, friendly, `magic = true` (the crit block's
            // overlay below), `tileCollide = false`, `penetrate = -1`, the
            // LOCAL immunity pair (8), `timeLeft = 1200`.
            SANDNADO_FRIENDLY => (10.0, 10.0, 127, -1, 1200, 0, false, false, false, (true, 8, false, -1, 0, 1.0)),
            // W4-d13 — `Projectile.cs:7061-7070` (657, the hostile Sandnado):
            // 10x10, aiStyle 127, hostile, `tileCollide = false`,
            // `penetrate = -1`, `timeLeft = 1200`, NO immunity pair.
            SANDNADO_HOSTILE => (10.0, 10.0, 127, -1, 1200, 0, false, false, false, RESET_STATE),
            // W4-d13 — `Projectile.cs:7071-7082` (658, the Sandnado Mark):
            // 14x14, aiStyle 128, `penetrate = 1` (the stock), `timeLeft =
            // 900`, `tileCollide = false`, `ignoreWater`, `alpha = 255`,
            // hostile.
            SANDNADO_MARK => (14.0, 14.0, 128, 1, 900, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d20 T4a — the pet-buff shoot arm's spawn kinds (the
            // `Player.cs:48561-48583` arm needs a row to spawn through
            // `spawn_with_data_ai`; the row-less fallback would carry no
            // box/aiStyle). The C# rows verbatim:
            // `Projectile.cs:8451-8460` (881 KingSlimePet): 20x20,
            // aiStyle 26, friendly, pen -1, timeLeft *= 5, netImportant.
            KING_SLIME_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:8874-8883` (934 QueenSlimePet): the 881 shape.
            QUEEN_SLIME_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9436-9445` (994 JunimoPet): the 881 shape.
            JUNIMO_PET => (20.0, 20.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9024-9033` (956 BerniePet): 20x28, the 881 shape.
            BERNIE_PET => (20.0, 28.0, 26, -1, 18000, 0, true, false, false, RESET_STATE),
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
            // W4-d7 wiring — `Projectile.cs:4635-4644` (379 BabySpider): 14x10,
            // aiStyle 63, friendly, `timeLeft = 300`, `penetrate = 1`,
            // `decidesManualFallThrough` (the overlay below). The stock
            // tileCollide true / no immunity pair / no minion slice (379 is a
            // SentryShot, NOT `minion`/`sentry` — `SENTRY_SHOT` lists it).
            BABY_SPIDER => (14.0, 10.0, 63, 1, 300, 0, true, false, false, RESET_STATE),
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
            // W4-d12 — `Projectile.cs:6385-6396` TruffleSpore (590): 14x14,
            // aiStyle 112, friendly, `penetrate = 1`, `alpha = 255`,
            // `timeLeft = 900`, `tileCollide = false`, `ignoreWater`.
            TRUFFLE_SPORE => {
                (14.0, 14.0, 112, 1, 900, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:8060-8066` DandelionSeed (836): 4x4,
            // aiStyle 112, HOSTILE (the pool's `spawn_hostile_ai` flips
            // `friendly`); the stock `penetrate = 1` (`:774`), `timeLeft =
            // 3600` (`:787`), `tileCollide = true` (`:775`), alpha 0.
            DANDELION_SEED => {
                (4.0, 4.0, 112, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d12 — `Projectile.cs:6511-6520` PortalGunGate (602): 10x10,
            // aiStyle 114, `alpha = 255`, friendly, `tileCollide = false`,
            // `netImportant`; the stock `penetrate = 1` (`:774`) and
            // `timeLeft = 3600` (`:787`) — the AI pins `timeLeft = 3` every
            // tick (`:34375`).
            PORTAL_GUN_GATE => {
                (10.0, 10.0, 114, 1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:6542-6553` SolarFlareRay (607): 10x10,
            // aiStyle 116, friendly, `alpha = 255`, `timeLeft = 600`,
            // `ignoreWater`, `tileCollide = false`, `penetrate = -1`.
            SOLAR_FLARE_RAY => {
                (10.0, 10.0, 116, -1, 600, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:6554-6566` SolarCounter (608): 160x160,
            // aiStyle 117, friendly, `alpha = 255`, `timeLeft = 3`,
            // `ignoreWater`, `tileCollide = false`, `penetrate = -1`,
            // `hide = true` (draw-only, no port field — F3). The spawn
            // sites `Kill()` it immediately (`Player.cs:21171`/`:38318`),
            // so the AI arm never runs in vanilla; the burst rides the
            // Kill path (`:72177-72225`).
            SOLAR_COUNTER => {
                (160.0, 160.0, 117, -1, 3, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:6584-6597` SolarWhipSwordExplosion
            // (612): 8x8, aiStyle 117, friendly, `alpha = 255`,
            // `ignoreWater`, `timeLeft = 60`, `tileCollide = false`,
            // `penetrate = -1`, local-immunity 6.
            SOLAR_WHIP_SWORD_EXPLOSION => {
                (8.0, 8.0, 117, -1, 60, 0, false, false, false, (true, 6, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:6717-6728` StardustGuardianExplosion
            // (624): 8x8, aiStyle 117, friendly, `alpha = 255`,
            // `ignoreWater`, `timeLeft = 60`, `tileCollide = false`,
            // `penetrate = -1` (no immunity pair — the stock reset).
            STARDUST_GUARDIAN_EXPLOSION => {
                (8.0, 8.0, 117, -1, 60, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:8988-9001` DaybreakExplosion (953):
            // 8x8, aiStyle 117, friendly, `alpha = 255`, `ignoreWater`,
            // `timeLeft = 60`, `tileCollide = false`, `penetrate = -1`,
            // local-immunity 80.
            DAYBREAK_EXPLOSION => {
                (8.0, 8.0, 117, -1, 60, 0, false, false, false, (true, 80, false, -1, 255, 1.0))
            }
            // W4-d12 — `Projectile.cs:9875-9889` DeadCellsMushroomBoiMinion-
            // Explosion (1044): 8x8, aiStyle 117, friendly, `alpha = 255`,
            // `ignoreWater`, `timeLeft = 60`, `tileCollide = false`,
            // `penetrate = -1`, `hide = true` (draw-only — F3),
            // local-immunity −1 (one hit per NPC for the shot's life).
            DEAD_CELLS_MUSHROOM_BOI_EXPLOSION => {
                (8.0, 8.0, 117, -1, 60, 0, false, false, false, (true, -1, false, -1, 255, 1.0))
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
            // W4-d20 — `Projectile.cs:10078-10086` (1080, the bird dropping):
            // 10x10, aiStyle 200, alpha 255, penetrate -1, ignoreWater, stock
            // tileCollide true. Stock friendly/hostile false — the arm's own
            // player-contact Kill() is the only interaction (the debris
            // precedent); the spawner passes damage 0 (NPC.cs:25322).
            BIRD_DROPPINGS => (10.0, 10.0, 200, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d20 — `Projectile.cs:10067-10077` (1078, the meteor ore):
            // 14x14, aiStyle 199, hostile+friendly+trap, penetrate -1,
            // netImportant, stock tileCollide true. The friendly damage rides
            // the generic PVE pass; the hostile player-contact is the port's
            // !friendly proxy deviation (the ore is friendly, so it hits NPCs).
            METEOR_ORE => (14.0, 14.0, 199, -1, TIME_LEFT_DEFAULT, 0, true, false, true, RESET_STATE),
            // W4-d20 — `Projectile.cs:9793-9806` (1038, the Flower Whip's
            // petal): 18x18, aiStyle 196, timeLeft 120, penetrate 1, the
            // LOCAL pair (-1), tileCollide false, ignoreWater
            // (armorPenetration = 50 dropped — no ProjData field, the
            // set_armor_penetration spawn seam carries it).
            FLOWER_WHIP_PETAL => (18.0, 18.0, 196, 1, 120, 0, false, false, false, (true, -1, false, -1, 0, 1.0)),
            // W4-d20 — `Projectile.cs:10413-10426` (1116, the Petalstorm
            // shot): 18x18, aiStyle 196, timeLeft 120, penetrate 1, the
            // LOCAL pair (-1), ignoreWater, magic, stock tileCollide true
            // (the blocked-move bounce :18068-18078).
            PETALSTORM_SHOT => (18.0, 18.0, 196, 1, 120, 0, true, false, false, (true, -1, false, -1, 0, 1.0)),
            // W4-d20 — `Projectile.cs:9446-9457` (995, the jumino stardrop
            // animation): 240x104, aiStyle 192, penetrate -1, tileCollide
            // false, ignoreWater, timeLeft = 3600 * 5 = 18000.
            STARDEW_JUMINO => {
                (240.0, 104.0, 192, -1, 18000, 0, false, false, false, RESET_STATE)
            }
            // W4-d20 — `Projectile.cs:9659-9666` (1020, Jim's drone): 12x12,
            // aiStyle 195, penetrate -1, stock tileCollide true (the
            // blocked-move bounce :18057-18067). friendly rides the pool
            // default; the drone damages nothing (no melee/ranged/magic).

            // W4-d20 — `Projectile.cs:9821-9833` (1040, the Dead Cells flint):
            // 2x2, aiStyle 198, tileCollide false, ignoreWater, penetrate -1,
            // alpha 255, manualDirectionChange (the arm owns the facing).
            // drawLayer 7 / usesOwnerLight are draw-side (no ProjData field).
            DEAD_CELLS_FLINT => (2.0, 2.0, 198, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0)),
            // W4-d20 — `Projectile.cs:9834-9847` (1041, the flint ground
            // spike): 32x32, aiStyle 157, friendly, melee, alpha 255,
            // tileCollide false, ignoreWater, penetrate 4, LOCAL pair (25).
            DEAD_CELLS_FLINT_SHOT => (32.0, 32.0, 157, 4, TIME_LEFT_DEFAULT, 0, false, false, false, (true, 25, false, -1, 255, 1.0)),
            // W4-d20 — `Projectile.cs:9858-9874` (1043, the flint release
            // BATCH-D keep-d19 drop-d20 (research/26g s9.7): the 1043
            // flint-slash row + the four 972/982+997/983/984 slash-row
            // copies are dropped here - d19's value-identical rows below
            // survive (the T5 use-arms spawn through them).
            // W4-d20 — `Projectile.cs:9315-9329` (973, the True Night's Edge
            // spin slash): 30x30, aiStyle 191, friendly, melee, penetrate 3,
            // LOCAL pair (10), tileCollide false, ignoreWater, alpha 255,
            // timeLeft 90.
            TRUE_NIGHTS_EDGE => (30.0, 30.0, 191, 3, 90, 0, false, false, false, (true, 10, false, -1, 255, 1.0)),
            // W4-d20 — `Projectile.cs:9405-9421` (985, the Terra Blade second
            // shot): 8x8, aiStyle 191, friendly, melee, penetrate -1, LOCAL
            // pair (-1), stock tileCollide true (the blocked-move retract
            // :18104-18110), ignoreWater, alpha 255, timeLeft 90,
            // ownerHitCheck (the arm sets it dynamically).
            TERRA_BLADE_2_SHOT => (8.0, 8.0, 191, -1, 90, 0, true, false, false, (true, -1, false, -1, 255, 1.0)),
            932 => (10.0, 10.0, 181, 5, 120, 0, true, true, false, RESET_STATE),
            // W4-d19 — the aiStyles 182-186/190 census slice's rows (the
            // `projectile_ai_sword_styles` arms). The damage-side fields
            // the rows set but the port carries no slot field for
            // (`ownerHitCheck`/`ownerHitCheckDistance`/`usesOwnerMeleeHitCD`/
            // `stopsDealingDamageAfterPenetrateHits`/`noEnchantmentVisuals`/
            // `hide`/`manualDirectionChange`) stay with the melee port —
            // the LIGHTS_BANE row's precedent.
            // `Projectile.cs:5505-5517` — the Soul Drain (476): 200x200,
            // aiStyle 185, penetrate -1, tileCollide false, alpha 255,
            // ignoreWater (the overlay), timeLeft 3, magic (the overlay).
            SOUL_DRAIN => (
                200.0, 200.0, 185, -1, 3, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:8438-8450` / `:8804-8816` — the Zoologist
            // strikes (880/929): 32x32, aiStyle 183, penetrate -1,
            // timeLeft 18, ignoreWater (the overlay), tileCollide false.
            ZOOLOGIST_STRIKE_GREEN | ZOOLOGIST_STRIKE_RED => (
                32.0, 32.0, 183, -1, 18, 0, false, false, false, RESET_STATE
            ),
            // `Projectile.cs:8857-8873` / `:10265-10281` — the Zenith
            // illusions (933/1100): 32x32, aiStyle 182, melee (the
            // overlay), tileCollide false, ignoreWater (the overlay),
            // alpha 255, extraUpdates 1, the local-immunity pair with
            // `localNPCHitCooldown = 15 / MaxUpdates` = 7 (INTEGER
            // division, MaxUpdates = 1 + extraUpdates = 2), penetrate -1.
            FINAL_FRACTAL | TRUE_COPPER_SHORTSWORD => (
                32.0, 32.0, 182, -1, TIME_LEFT_DEFAULT, 1, false, false, false,
                (true, 7, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:8948-8955` — the Torch God's fireball (949):
            // 4x4, aiStyle 184, penetrate -1, tileCollide false (the arm
            // re-arms it when ai[1] drains); the stock 3600 life.
            TORCH_GOD => (4.0, 4.0, 184, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE),
            // `Projectile.cs:8956-8968` — the Resonance Scepter's blast
            // (950): 104x104, aiStyle 186, magic (the overlay), alpha 255,
            // ignoreWater (the overlay), timeLeft 180, tileCollide false,
            // penetrate -1.
            PRINCESS_WEAPON => (
                104.0, 104.0, 186, -1, 180, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:9298-9314` — the Night's Edge slash (972):
            // 16x16, aiStyle 190, melee (the overlay), penetrate 2, the
            // local-immunity pair (-1), tileCollide false, ignoreWater
            // (the overlay).
            NIGHTS_EDGE => (
                16.0, 16.0, 190, 2, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:9354-9370` / `:9466-9482` — the Excalibur
            // (982) and the Horseman's Blade (997): the 972 shape with
            // penetrate 3.
            EXCALIBUR | HORSEMANS_BLADE => (
                16.0, 16.0, 190, 3, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:9371-9387` — the True Excalibur (983): the
            // 982 shape with penetrate 6.
            TRUE_EXCALIBUR => (
                16.0, 16.0, 190, 6, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:9388-9404` — the Terra Blade's slash (984):
            // the 982 shape (penetrate 3).
            TERRA_BLADE_2 => (
                16.0, 16.0, 190, 3, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:9858-9874` — the Flint's slash (1043): the
            // 972 shape (penetrate 2).
            DEAD_CELLS_FLINT_SLASH => (
                16.0, 16.0, 190, 2, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:10543-10558` — the Chlorophyte Claymore's
            // slash (1126): 16x16, aiStyle 190, melee (the overlay),
            // penetrate -1, the local-immunity pair (-1), tileCollide
            // false, ignoreWater (the overlay). The row sets NO
            // stopsDealingDamageAfterPenetrateHits (the 972 family does).
            CHLOROPHYTE_CLAYMORE_BLADE => (
                16.0, 16.0, 190, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // W4-d19 follow-up — the aiStyle-92 cloud family (the 1126
            // spore machine's children + the style's other rows,
            // `projectile_ai_clouds`).
            // `Projectile.cs:5805-5816` / `:5817-5828` / `:5829-5840` —
            // the Poison Staff's toxic clouds (511/512/513): 32x32 /
            // 40x38 / 30x28, aiStyle 92, friendly, penetrate -1, scale
            // 1.1, magic (the overlay), alpha 255,
            // `UseGroupStaticImmunity(511)` (the identity overlay below).
            // The stock `tileCollide = true` stands — the arm clears it
            // on the first tick (`:32567`).



            // `Projectile.cs:9558-9569` — the Gas Trap's gas (1007):
            // 50x50, aiStyle 92, hostile AND friendly (the port's
            // friendly default stands; the contact damage is refused by
            // `Damage_CanDealDamage` `:12448` — the gas buff is the
            // lane), penetrate -1, scale 1.1, trap; `hide` has no port
            // field (the draw seam).

            // `Projectile.cs:10565-10579` — the Chlorophyte Claymore's
            // spore cloud (1128): 30x30, aiStyle 92, tileCollide false,
            // friendly, melee (the overlay), scale 1.1, penetrate -1,
            // idStatic-30; `noEnchantmentVisuals` has no port field.

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
            // W4-f33 — `Projectile.cs:8948-8955`: the Torch God shot (949):
            // 4x4, aiStyle 184 (`AI_184_BadTorchLuck`), `penetrate = -1`,
            // `tileCollide = false` (the AI's ai[1] budget arms it later).
            // The stock `friendly`/`hostile` stay false (`:788`/`:764` —
            // the arm touches neither): the hurt rides the AI's own
            // `SelfHurtPlayers`, never the pool's PVE/EVP lanes. The stock
            // timeLeft 3600 (`:787`) / extraUpdates 0 (`:771`) stand.

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
            // `Projectile.cs:1375-1383` — the Hornet family's Stinger (55):
            // 10x10, hostile, `penetrate = -1`, aiStyle 1, `tileCollide =
            // true`; the stock 3600 life (the aiStyle-5 volley overrides to
            // 300 post-spawn, `NPC.cs:50797` — the sim's timed-shot list).
            // The `flag3 = false` skip-list membership (`:55954`) rides
            // `AI_STYLE_1_RIDERS` — the resting counter, straight flight
            // (W4-d36 E4 item 4, `npc_ai_style5.rs`).

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
            // W4-d18 — `Projectile.cs:8380-8392`: the Split-Shot Core (871):
            // 30x30, aiStyle 172, `alpha = 255`, `penetrate = -1`, hostile,
            // `timeLeft = 240`, `tileCollide = false`, `ignoreWater = true`
            // (the overlay below). The AI is the pellet-storm machine.
            HALLOW_BOSS_SPLIT_SHOT_CORE => (
                30.0, 30.0, 172, -1, 240, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // ===== W4-d18: the aiStyle-174 multisegment pets =====
            // `Projectile.cs:8474-8485` — the Eater of Worlds Pet (883):
            // 20x20, aiStyle 174, `penetrate = -1`, `timeLeft *= 5`
            // (the stock 3600 → 18000), friendly, `ignoreWater = true`,
            // `tileCollide = false`, `netImportant = true` (the overlays
            // below). NOT a minion (no `minion`/`minionSlots` write).
            EATER_OF_WORLDS_PET => (
                20.0, 20.0, 174, -1, 18000, 0, false, false, false, RESET_STATE,
            ),
            // `Projectile.cs:8496-8507` — the Destroyer Pet (887): the
            // identical block (20x20, 174, -1, ×5 life, the three flags).
            DESTROYER_PET => (
                20.0, 20.0, 174, -1, 18000, 0, false, false, false, RESET_STATE,
            ),
            // `Projectile.cs:8531-8542` — the Lunatic Cultist Pet (893):
            // the identical block again.
            LUNATIC_CULTIST_PET => (
                20.0, 20.0, 174, -1, 18000, 0, false, false, false, RESET_STATE,
            ),
            // ===== W4-d18: the aiStyle-175 titanium storm shard =====
            // `Projectile.cs:8626-8637` — the Titanium Storm Shard (908):
            // 14x14, aiStyle 175, `tileCollide = false`, `ignoreWater =
            // true` (the overlay below), friendly, `penetrate = 1`,
            // `timeLeft = 3600`, `drawLayer = 3` (draw-only, no port field).
            TITANIUM_STORM_SHARD => (
                14.0, 14.0, 175, 1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE,
            ),
            // ===== W4-d18: the aiStyle-176 edgy lightning =====
            // `Projectile.cs:8676-8688` — the Scythe Whip Proj (916):
            // 18x18, aiStyle 176, friendly, `tileCollide = false`,
            // `penetrate = 3`, the local-immunity pair (true, −1),
            // `extraUpdates = 2`. `armorPenetration = 50` is damage-side
            // (no port field — the melee port's slice).
            SCYTHE_WHIP_PROJ => (
                18.0, 18.0, 176, 3, TIME_LEFT_DEFAULT, 2, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // ===== W4-d18: the aiStyle-177 ice-whip slicers =====
            // `Projectile.cs:8701-8713` — the Cool Whip Proj (917):
            // 30x30, aiStyle 177, friendly, `tileCollide = false`,
            // `penetrate = -1`, local-immunity (true, 20), `coldDamage`
            // + `armorPenetration = 30` are damage-side (no port fields).
            COOL_WHIP_PROJ => (
                30.0, 30.0, 177, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, 20, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:9169-9181` — the Weather Pain Shot (969):
            // 56x64, aiStyle 177, friendly, magic (the overlay below),
            // `tileCollide = true`, `penetrate = 15`, the id-static pair
            // (true, 25), `armorPenetration = 10` damage-side.
            WEATHER_PAIN_SHOT => (
                56.0, 64.0, 177, 15, TIME_LEFT_DEFAULT, 0, true, false, false,
                (false, -2, true, 25, 0, 1.0),
            ),
            // ===== W4-d18: the aiStyle-178 fire explosion =====
            // `Projectile.cs:8689-8700` — the Fire Whip Proj (918):
            // 96x96, aiStyle 178, friendly, `tileCollide = false`,
            // `penetrate = -1`, the local-immunity pair (true, −1),
            // `alpha = 255`. The stock 3600 life (the AI kills at 10).
            FIRE_WHIP_PROJ => (
                96.0, 96.0, 178, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 255, 1.0),
            ),
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
            // ===== W4-d18 tail: the aiStyle-171 FRIENDLY weapon variants =====
            // `Projectile.cs:8826-8841` — the Nightglow bolt (931): 30x30,
            // aiStyle 171, `alpha = 255`, `penetrate = 3`, friendly,
            // `timeLeft = 240`, magic, `tileCollide = TRUE` (it BOUNCES —
            // `HandleMovement` `:18592-18602`), `ignoreWater = true`,
            // `extraUpdates = 1`, the local-immunity pair (true, 30).
            FAIRY_QUEEN_MAGIC_ITEM_SHOT => (
                30.0, 30.0, 171, 3, 240, 1, true, false, false,
                (true, 30, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:9807-9820` — the Constellation star (1039): the
            // identical 30x30/171/alpha255/friendly/240/tileCollide-true/
            // ignoreWater/extraUpdates-1 block EXCEPT `penetrate` stays the
            // stock 1 (`:774`) and the local cooldown is -1 (`:9819`). No
            // damage-class flag (the C# arm sets none).
            CONSTELLATION_STAR => (
                30.0, 30.0, 171, 1, 240, 1, true, false, false,
                (true, -1, false, -1, 255, 1.0),
            ),
            // ===== W4-d18 tail: the residual armed-but-unrowed kinds =====
            // `Projectile.cs:9127-9142` — the friendly Insanity shadow (964):
            // 40x40, aiStyle 187, `alpha = 255`, `penetrate = 3`, friendly,
            // local-immunity (true, -1), `ignoreWater = true`,
            // `tileCollide = false`, `extraUpdates = 1`, `scale = 0.7`.
            // `armorPenetration = 20` is damage-side (no port field). The
            // stock 3600 life (the AI's variation machine kills it).
            INSANITY_SHADOW_FRIENDLY => (
                40.0, 40.0, 187, 3, TIME_LEFT_DEFAULT, 1, false, false, false,
                (true, -1, false, -1, 255, 0.7),
            ),
            // PROJ-171 retire (978 dup, keep-d12c): the `VOLCANO => (...)`
            // row that sat here is DROPPED — d12c's 9e668d0f0 owns the 978
            // row (the superset with `proj_frames[978] = 5`). The 964/1106
            // residuals stay. BATCH-D: d12 IS merged — its 978 row (above)
            // + the melee flag serve the spawn; the keep-one is resolved.
            // `Projectile.cs:10303-10315` — the Foxsparks flames (1106):
            // 6x6, aiStyle 193, friendly, `alpha = 255`, `penetrate = 4`,
            // `extraUpdates = 2`, the ID-static pair (true, 15).
            // `drawLayer = 4` is render-side (no port field). The stock
            // tileCollide true / ignoreWater false (`:775`/`:752`).
            FOXSPARKS_FLAMES => (
                6.0, 6.0, 193, 4, TIME_LEFT_DEFAULT, 2, true, false, false,
                (false, -2, true, 15, 255, 1.0),
            ),
            // The Frost Moon family (S1-D-4, `ProjectileID.cs:1244-1258`):
            // the 345/346/348/349/350/351/352 rows are the S1-F-13c wiring
            // rows ABOVE (value-identical in both branches — deduped at
            // merge, deny(unreachable_patterns) keeps one arm per id).
            // `Projectile.cs:4370-4377` — the ornament's shrapnel (347):
            // 6x6, aiStyle 2, hostile, `penetrate = -1`; the stock life.
            ORNAMENT_SHRAPNEL => (6.0, 6.0, 2, -1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // W4-d78 batch 2 — the Stynger pair (the Kill child-spawn family).
            // `Projectile.cs:3289-3298` — the Stynger bolt (246): 10x10,
            // aiStyle 1, friendly, ranged, `alpha = 255`, `extraUpdates = 1`;
            // the stock penetrate 1 / timeLeft 3600 / tileCollide true
            // (`:774-775`/`:787` resets — the arm writes no override).
            STYNGER => (
                10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 1, true, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:3314-3321` — the Stynger shrapnel (249): 12x12,
            // aiStyle 2, friendly, ranged; the stock penetrate 1 / 3600 /
            // tileCollide true, alpha at the 0 reset. The 5-frame sheet is
            // `Main.cs:8610`.
            STYNGER_SHRAPNEL => (12.0, 12.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // S1-P-i22 — the magic guns + tomes' shots. The verbatim
            // `SetDefaults` arms, in `ProjectileID` order; `light`/`magic`/
            // `melee`/`ranged`/`ignoreWater`/`drawLayer`/`armorPenetration`
            // have no `ProjData` consumer (dropped, as the Phase-1 rows note).
            // `Projectile.cs:886-896` — the Starfury star (9): 24x24, aiStyle 5,
            // `penetrate = 2`, `scale = 0.8f`, `tileCollide = false`, melee.
            STARFURY => (24.0, 24.0, 5, 2, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 0, 0.8)),
            // W4-d1 — the aiStyle-5 falling-star family's unrowed kinds.
            // `Projectile.cs:924-938` — the natural FallingStar (12): 18×18,
            // aiStyle 5, penetrate −1, alpha 50, light 1 (the light slice),
            // trap; the remix seed's `hostile = true` (`:929-932`) is a world
            // flag, not a row constant (the stock friendly stands; the
            // `Main.remixWorld` read rides the arm, ANALYSIS). The stock
            // tileCollide true (`:775`) stands — the AI_005 arm arms it below
            // the `ai[1]` floor.

            // `Projectile.cs:5743-5756` — the Star Wrath star (503): 24×24,
            // aiStyle 5, penetrate 2, alpha 255, tileCollide false, melee,
            // extraUpdates 1, local immunity 15.
            STAR_WRATH => (
                24.0, 24.0, 5, 2, TIME_LEFT_DEFAULT, 1, false, false, false,
                (true, 15, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7646-7658` — the cloak-star family (723/724/725/
            // 726/1133): 24×24, aiStyle 5, penetrate 2, alpha 50, tileCollide
            // false, local immunity 10; `armorPenetration = 25` (`:7657`) has
            // no row consumer (the NETTLE_BURST precedent).
            MANA_CLOAK_STAR | BEE_CLOAK_STAR | STAR_VEIL_STAR | STAR_CLOAK_STAR
            | SNAKE_BAND_STAR => (
                24.0, 24.0, 5, 2, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, 10, false, -1, 50, 1.0),
            ),
            // `Projectile.cs:9780-9792` — the Meteor Whip meteor (1037):
            // 24×24, aiStyle 5, scale 0.8, extraUpdates 1, alpha 254,
            // tileCollide false, local immunity −1; the stock penetrate 1
            // (`:774`) stands (the arm writes none).
            METEOR_WHIP_METEOR => (
                24.0, 24.0, 5, 1, TIME_LEFT_DEFAULT, 1, false, false, false,
                (true, -1, false, -1, 254, 0.8),
            ),
            // `Projectile.cs:9938-9952` — the Meteor Staff meteor (1049):
            // 24×24, aiStyle 5, hostile + friendly, scale 0.8, timeLeft 1200,
            // extraUpdates 1, alpha 254, penetrate −1, tileCollide false,
            // trap. The `hostile` flag rides the arm's `friendly = false`
            // proxy (the hostile-spawn seam), not the row.
            METEOR_STORM_METEOR => (
                24.0, 24.0, 5, -1, 1200, 1, false, false, true,
                (false, -2, false, -1, 254, 0.8),
            ),
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
            // `Projectile.cs:5919-5928` — the Crystal Pulse 2 (522): 8x8,
            // aiStyle 29, `alpha = 255`, `penetrate = 1`, magic, friendly.
            // NO extraUpdates line (the stock 0). W4-g1 lane a — the 521
            // Kill hook's split shard (the row+arm+hook land together).
            CRYSTAL_PULSE_2 => (8.0, 8.0, 29, 1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 255, 1.0)),
            // `Projectile.cs:8206-8219` — the Sparkle Guitar note (856): 32x32,
            // aiStyle 167, `alpha = 255`, `tileCollide = false`,
            // `extraUpdates = 2`, local immunity −1, ignoreWater, magic.
            SPARKLE_GUITAR => (32.0, 32.0, 167, 1, TIME_LEFT_DEFAULT, 2, false, false, false, (true, -1, false, -1, 255, 1.0)),
            // W4-c39 — `Projectile.cs:8197-8205` — the Release Lantern (855):
            // 12x12, aiStyle 166, `penetrate = -1`, `timeLeft = 1000`,
            // `netImportant` (the tail below). The stock `tileCollide = true`
            // stands (the bounce is HandleMovement's `:18347-18361` arm).
            RELEASE_LANTERN => (12.0, 12.0, 166, -1, 1000, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:8173-8181` — the Release Doves (851): 4x4,
            // aiStyle 166, `penetrate = -1`, `timeLeft = 300`, netImportant.
            RELEASE_DOVES => (4.0, 4.0, 166, -1, 300, 0, true, false, false, RESET_STATE),
            // `Projectile.cs:9013-9023` — the Star Cannon star (955): 18x18,
            // aiStyle 5, `penetrate = -1`, `alpha = 50`, ranged (`light = 1f`
            // dropped).
            STAR_CANNON_STAR => (18.0, 18.0, 5, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, false, -1, 50, 1.0)),
            // W4-f48 — `Projectile.cs:7646-7658`: the five cloak stars
            // (723/724/725/726/1133) share ONE arm: 24x24, aiStyle 5,
            // friendly, `penetrate = 2`, `alpha = 50`, `tileCollide = false`
            // (the AI arms it at `Center.Y > ai[1]`, `:23853-23856`),
            // `usesLocalNPCImmunity` + `localNPCHitCooldown = 10`.
            // `armorPenetration = 25` (`:7657`) has no `ProjData` field — the
            // retaliate drain sets it through `set_armor_penetration` (the
            // gun_launch precedent). No damage-class flag (the stock false
            // trio — the C# arm sets none).

            // W4-e3 coverage phase — `Projectile.cs:5743-5756`: the Star
            // Wrath star (503): 24x24, aiStyle 5, friendly, `penetrate = 2`,
            // `alpha = 255`, `tileCollide = false`, melee, `extraUpdates = 1`,
            // local immunity 15. The stock timeLeft 3600 (`:787`).

            // `Projectile.cs:9780-9792`: the Meteor Whip meteor (1037): 24x24,
            // aiStyle 5, friendly, local immunity −1, `scale = 0.8`,
            // `extraUpdates = 1`, `alpha = 254`, `tileCollide = false`. The
            // stock penetrate 1 (`:774`) + timeLeft 3600 (`:787`).

            // `Projectile.cs:9938-9952`: the Meteor Staff storm meteor (1049):
            // 24x24, aiStyle 5, friendly AND hostile (the hostile half rides
            // `spawn_hostile`'s friendly=false flip — the FLAMES_TRAP
            // precedent), `scale = 0.8`, `timeLeft = 1200`, `extraUpdates = 1`,
            // `alpha = 254`, `penetrate = -1`, `tileCollide = false`, trap.

            // W4-d17 — `DefaultToShortsword` (`Projectile.cs:10773-10787`):
            // 18x18, aiStyle 161, `penetrate = -1`, `tileCollide = false`,
            // `extraUpdates = 1`, melee, `ownerHitCheck` (the damage gate is
            // keyed on the kind set in `damage_pve`, the spear precedent).
            // The Gladius (802, `:7921-7924`) / Ruler (842, `:8145-8148`)
            // keep the stock 3600 life and NO immunity slice.
            GLADIUS_STAB | RULER_STAB => (18.0, 18.0, 161, -1, TIME_LEFT_DEFAULT, 1, false, false, false, RESET_STATE),
            // `:8917-8921` — the eight ore shortswords (938-945):
            // `DefaultToShortsword` + `timeLeft = 360`.
            SHORTSWORD_STAB_FIRST..=SHORTSWORD_STAB_LAST => (18.0, 18.0, 161, -1, 360, 1, false, false, false, RESET_STATE),
            // `:10559-10564` — the Chlorophyte Saber stab (1127):
            // `DefaultToShortsword` + `usesLocalNPCImmunity` /
            // `localNPCHitCooldown = -1`.
            CHLOROPHYTE_SABER_STAB => (18.0, 18.0, 161, -1, TIME_LEFT_DEFAULT, 1, false, false, false, (true, -1, false, -1, 0, 1.0)),
            // W4-d17 — `Projectile.cs:8005-8018` — the White Tiger Pounce
            // (818): 132x132, aiStyle 162, `alpha = 255`,
            // `tileCollide = false`, `ignoreWater`, `penetrate = -1`,
            // `extraUpdates = 3`, the LOCAL pair (−1). The stock 3600 life.
            // `UpdatePosition`'s skip list names 818 (`:20435`) — the
            // CatmullRom arc IS the position (the dispatch folds the skip).
            WHITE_TIGER_POUNCE => (132.0, 132.0, 162, -1, TIME_LEFT_DEFAULT, 3, false, false, false, (true, -1, false, -1, 255, 1.0)),
            // W4-d17 — `Projectile.cs:8019-8029` — the Chum Bucket bait
            // (820): 12x12, aiStyle 163, `tileCollide = true`,
            // `netImportant`, `penetrate = -1`, `timeLeft = 86400`. The
            // stock extraUpdates 0; NO immunity slice (the bait is
            // friendly but deals no damage — the spawner's damage 0).
            CHUM_BUCKET => (12.0, 12.0, 163, -1, 86400, 0, true, false, false, RESET_STATE),
            // W4-d17 — `Projectile.cs:8173-8181` — the Release Doves (851):
            // 4x4, aiStyle 166, `penetrate = -1`, `timeLeft = 300`,
            // `netImportant`. The stock tileCollide true / extraUpdates 0.

            // `:8197-8205` — the Release Lantern (855): 12x12, aiStyle 166,
            // `penetrate = -1`, `timeLeft = 1000`, `netImportant`.

            // W4-d17 — `Projectile.cs:8220-8236` — the First Fractal (857):
            // 32x32, aiStyle 168, `alpha = 255`, melee, `tileCollide = false`,
            // `ignoreWater`, `extraUpdates = 1`, the LOCAL pair (10),
            // `manualDirectionChange`, `penetrate = -1`, drawLayer 6. NO
            // `projFrames[857]` override — the stock 1 stands (the draw reads
            // the hue sparkle, not a frame slice).
            FIRST_FRACTAL => (32.0, 32.0, 168, -1, TIME_LEFT_DEFAULT, 1, false, false, false, (true, 10, false, -1, 255, 1.0)),
            // W4-d17 FOLLOW-UP — `Projectile.cs:9767-9779` — the Cob Whip's
            // spark minion (1036): 18x10, aiStyle 67, `penetrate = -1`,
            // `netImportant`, the LOCAL pair (20), `armorPenetration = 30`
            // (the spawn stamp), `decidesManualFallThrough` (the `flag7`
            // chase `:49130`/`:50538`/`:50601` — the aiStyle-67 machine is
            // the documented follow-up; the row + the proc spawn land here).

            // `Projectile.cs:9780-9792` — the Starcrash proc meteor (1037):
            // 24x24, aiStyle 5, the LOCAL pair (−1), scale 0.8,
            // `extraUpdates = 1`, alpha 254, `tileCollide = false` (the AI's
            // `Bottom.Y >= ai[1]` latch re-arms it, `:23865-23868`). The
            // stock penetrate 1 / 3600 life.

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
            // W4-g2 (d5 F6) — `:3151-3163`: the Chlorophyte Partisan's orb:
            // 30x30, aiStyle 44, friendly, melee, `penetrate = -1`,
            // `light = 0.2f`, the id-static 10 pair. The stock tileCollide
            // TRUE stands (the 228's false does not carry over) — the orb
            // dies on tile contact. Scale 1 / alpha 0 (the stock).
            CHLOROPHYTE_ORB => (30.0, 30.0, 44, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2, true, 10, 0, 1.0)),
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
            // W4-c10 — `Projectile.cs:10078-10086`: the BirdDroppings
            // 1080: 10x10, aiStyle 200 (`AI_200_BirdDroppings`,
            // `:39985-40017`), alpha 255, penetrate -1, ignoreWater (the
            // overlay below); the stock tileCollide true (`:775`) and
            // timeLeft 3600 (`:787`) stand — the type arm writes neither.

            // `Projectile.cs:6450-6460` — the Djinn's curse (596, W4-g5):
            // 8x8, aiStyle 107, hostile, alpha 255, ignoreWater (the
            // overlay below), timeLeft 180, tileCollide false; the stock
            // penetrate 1 (`:774`). The flight arm is `ai_djinn_curse`
            // (the shared 107 homing tail `:33653-33677` + the 596 dust
            // ring's Next(3) early-RETURN quirk `:33617-33630`); the
            // death explosion is `djinn_curse_kill` (`:72470-72502`).

            // W4-f42 — the Djinn's curse (`Projectile.cs:6450-6460`):
            // 8×8, aiStyle 107, hostile, `alpha = 255`, `ignoreWater`,
            // `timeLeft = 180`, `tileCollide = false`. The stock penetrate 1
            // (`:774`) stands — the curse is a single-hit area burst (the
            // `timeLeft < 4` resize + `Damage()` arm, `:33631-33651`).

            // W4-f42 — the Antlion's sand ball (`Projectile.cs:1131-1140`):
            // 10×10, aiStyle 10, `penetrate = -1`, friendly + hostile (the
            // Antlion's call site sets friendly false). The stock 3600 life
            // (the call site overrides to 300, `NPC.cs:24464`); knockBack 6
            // is overridden by the call's `KnockBack: 0f`.

            // S1-E-21h d76 DOC-SWEEP — the Kill()-tail non-bomb cue kinds
            // (`Projectile.cs:70855-70975`). The rows land so the death-hook
            // cue map can spawn them; the AI machines for the four unarmed
            // styles (195/200/202/205) are the documented census follow-ups
            // (UNARMED_ROWED, the i25 precedent) — the cue lands on death.
            // (d76's JIMS_DRONE row deduped into the W4-d80 arm above —
            // term-identical; the Kill()-tail cue map still spawns 1020.)
            // `:10078-10086` — Bird Droppings (1080): 10x10, aiStyle 200,
            // alpha 255, penetrate -1, ignoreWater (the overlay below); the
            // stock friendly false / tileCollide true / 3600 life.

            // `:10115-10124` — the Torch God's helper (1084): 4x4, aiStyle
            // 202, extraUpdates 2, timeLeft 300, penetrate -1, tileCollide
            // false (the d21 row values, kept identical for the merge).

            // `:10125-10132` — the Pretty Mirror (1085): 18x18, aiStyle 2
            // (the thrown family — its rotation ladder `:23245` is already
            // in `ai_style2`), friendly, penetrate 1.
            PRETTY_MIRROR => (18.0, 18.0, 2, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE),
            // `:10294-10302` — the RC Car (1105): 26x26, aiStyle 205,
            // friendly, tileCollide true, correctSlopeCollision (the slope
            // pass is the d21 movement follow-up; the d21 row values kept).

            // S1-W4-g81 residuals — the Wire Kite (651,
            // `Projectile.cs:6990-7000`): 10x10, aiStyle 125, friendly,
            // ignoreWater, tileCollide false, penetrate -1, drawLayer 5
            // (the over-wire cache, `Main.cs:38177-38178` — the port draws
            // it in the wire-adjacent slot). The stock 3600 life; the
            // channel-release Kill owns the death (`:35716-35723`).

            // `Projectile.cs:9446-9457` — the Stardew Jumino animation
            // (995): 240x104, aiStyle 192, friendly, penetrate -1,
            // tileCollide false, ignoreWater (the overlay), netImportant
            // (the overlay), `timeLeft *= 5` (3600 -> 18000). The dryad's
            // `HaveDryadDoStardewAnimation` spawns it (`NPC.cs:19226`,
            // W4-g51 id-144); the AI_192 frame machine + the Stardew draw
            // (`Main.cs:28170-28173`) are the animation lane's.

            // W4-d15 — `Projectile.cs:7385-7401`: the Brand of the
            // Inferno's ground blast (698): 75x200, aiStyle 141, alpha 255,
            // tileCollide false, ignoreWater (the overlay), timeLeft 5,
            // penetrate -1, local immunity -1, melee (the overlay).
            // W4-d14's row is field-identical (the integrate keep-one).
            MONK_STAFF_T1_EXPLOSION => (
                75.0, 200.0, 141, -1, 5, 0, false, false, false,
                (true, -1, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7402-7418`: the T2 swing ghost (699): 24x24,
            // aiStyle 142, penetrate -1, alpha 255, tileCollide false,
            // ignoreWater (the overlay), melee (the overlay), idStatic
            // immunity 10. drawLayer 7 / usesOwnerLight / ownerHitCheck
            // are draw/hit-check seams — no port fields.
            MONK_STAFF_T2 => (
                24.0, 24.0, 142, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (false, -2, true, 10, 255, 1.0),
            ),
            // `Projectile.cs:7419-7432`: the T2 ghast (700): 40x40,
            // aiStyle 143, alpha 255, penetrate -1, melee (the overlay),
            // local immunity -1, ignoreWater (the overlay), tileCollide
            // false.
            MONK_STAFF_T2_GHAST => (
                40.0, 40.0, 143, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, -1, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7474-7490`: the T3 normal swing (707):
            // 16x16, aiStyle 140, penetrate -1, alpha 255, tileCollide
            // false, ignoreWater, local immunity 6, melee (the overlay).
            // drawLayer 7 / usesOwnerLight / ownerHitCheck are draw/
            // hit-check seams — no port fields. W4-d14's row is
            // field-identical and its `ai_monk_staff` arm serves it
            // (the integrate keep-one — the cross-note on the const).
            MONK_STAFF_T3 => (
                16.0, 16.0, 140, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, 6, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7491-7507`: the T3-alt swing ghost (708) —
            // the 699 box with LOCAL immunity 6 instead of idStatic 10.
            MONK_STAFF_T3_ALT => (
                24.0, 24.0, 142, -1, TIME_LEFT_DEFAULT, 0, false, false, false,
                (true, 6, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7508-7520`: the T3-alt volley shot (709):
            // 12x12, aiStyle 1 (the generic flight arm), stock penetrate
            // 1 / tileCollide true / 3600 life, alpha 255, ignoreWater
            // (the overlay), extraUpdates 1, scale 0.6, melee (the
            // overlay).

            // `Projectile.cs:7433-7446` (701/702/703) + `:7866-7878`
            // (764) + `:9953-9966` (1050): the DD2 pets — 20x20, aiStyle
            // 144, penetrate -1, netImportant (the overlay), timeLeft *=
            // 5 (18000), friendly, ignoreWater (the overlay), tileCollide
            // false, manualDirectionChange (no port field — the arms pin
            // facing themselves).
            DD2_PET_DRAGON | DD2_PET_GHOST | DD2_PET_GATO | UPBEAT_STAR | AXE_FAIRY_PET => (
                20.0, 20.0, 144, -1, TIME_LEFT_DEFAULT * 5, 0, false, false, false, RESET_STATE
            ),
            // `Projectile.cs:7447-7460`: the Book Staff twister (704):
            // 60x210, aiStyle 145, magic (the overlay), tileCollide
            // false, penetrate -1, local immunity -1, timeLeft 600,
            // ignoreWater (the overlay).
            DD2_APPRENTICE_STORM => (
                60.0, 210.0, 145, -1, 600, 0, false, false, false,
                (true, -1, false, -1, 0, 1.0),
            ),
            // `Projectile.cs:7560-7571`: the DD2 victory banner (713):
            // 10x10, aiStyle 146 (the empty body), alpha 255, tileCollide
            // false, ignoreWater (the overlay), timeLeft 900, penetrate
            // -1. `hostile = true` (`:7569`) rides the `spawn_hostile`
            // seam (the port's hostile convention — no data field).
            DD2_VICTORY => (
                10.0, 10.0, 146, -1, 900, 0, false, false, false,
                (false, -2, false, -1, 255, 1.0),
            ),
            // `Projectile.cs:7617-7625`: the starfall spawner (720):
            // 16x16, aiStyle 148, tileCollide false, penetrate -1,
            // alpha 255, stock 3600 life.

            // `Projectile.cs:924-938`: the natural FallingStar (12):
            // 18x18, aiStyle 5, penetrate -1, alpha 50, light 1f, trap,
            // stock tileCollide true / 3600 life. `hostile` on remix
            // (`:929-932`) rides the spawn seam (no data field).

            // W4-d16 — `Projectile.cs:7659-7671` (728, the Super Star
            // Cannon's star): 24x24, aiStyle 151, alpha 255, tileCollide
            // true (the stock), ranged + friendly (the overlays),
            // penetrate -1, the LOCAL pair (-1, `:7669-7670`).
            SUPER_STAR => {
                (24.0, 24.0, 151, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (true, -1, false, -1, 255, 1.0))
            }
            // W4-d16 — `Projectile.cs:7672-7687` (729, the Super Star
            // Cannon's slash): 20x20, aiStyle 152, friendly, tileCollide
            // false, ignoreWater (the overlay), ranged (the overlay),
            // penetrate -1, extraUpdates 2, timeLeft 10*MaxUpdates = 30,
            // the idStatic pair (10, `:7685-7686`). The `scale = 1f +
            // Next(30)*0.01f` (`:7682`) is a spawn-time draw — the row
            // carries the 1f base (the Horseman-pumpkin precedent).
            SUPER_STAR_SLASH => {
                (20.0, 20.0, 152, -1, 30, 2, false, false, false, (false, -2, true, 10, 0, 1.0))
            }
            // W4-d16 — `Projectile.cs:9208-9224` (977, the Zenith's
            // Muramasa slash): 20x20, aiStyle 152, friendly, tileCollide
            // false, ignoreWater, melee (the overlay), penetrate 1, the
            // LOCAL pair (-1), extraUpdates 2, timeLeft 10*MaxUpdates =
            // 30; armorPenetration 5 (`:9220`, no port row field — the
            // 864 precedent); the scale roll (`:9221`) is spawn-time.
            ZENITH_MURAMASA_SLASH => {
                (20.0, 20.0, 152, 1, 30, 2, false, false, false, (true, -1, false, -1, 0, 1.0))
            }
            // W4-d16 — `Projectile.cs:9270-9286` (976, the Zenith's
            // Blade-of-Grass slash): 20x20, aiStyle 152, friendly,
            // tileCollide false, ignoreWater, melee, penetrate 2, the
            // LOCAL pair (-1), extraUpdates 0, timeLeft 60*MaxUpdates =
            // 60; armorPenetration 20 (`:9285`, dropped); the scale roll
            // (`:9282`) is spawn-time.
            ZENITH_GRASS_SLASH => {
                (20.0, 20.0, 152, 2, 60, 0, false, false, false, (true, -1, false, -1, 0, 1.0))
            }
            // W4-d16 — `Projectile.cs:9890-9907` (1045, the Moon Lord
            // whip's proc slash): 20x20, aiStyle 152, friendly,
            // tileCollide false, ignoreWater, melee, penetrate 1, the
            // LOCAL pair (-1), extraUpdates 1, timeLeft 24*MaxUpdates =
            // 48, alpha 255 (`:9905`); armorPenetration 50 (`:9906`,
            // dropped); the scale roll (`:9902`) is spawn-time.
            MOON_LORD_WHIP_PROC => {
                (20.0, 20.0, 152, 1, 48, 1, false, false, false, (true, -1, false, -1, 255, 1.0))
            }
            // W4-d16 — `Projectile.cs:7716-7725` (733, the toilet's pop):
            // 2x2, aiStyle 153, tileCollide false, ignoreWater (the
            // overlay), penetrate -1, alpha 255. Stock friendly/hostile
            // false — the effect damages nothing (the debris precedent).
            TOILET_EFFECT => {
                (2.0, 2.0, 153, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d16 — `Projectile.cs:7726-7735` (734, the Void Lens):
            // 30x24, aiStyle 154, tileCollide false, timeLeft 10800,
            // alpha 255; drawLayer 3 (`:7734`) is render-side (dropped).
            // Stock penetrate 1, friendly/hostile false — the lens is a
            // marker, not a shot.
            VOID_LENS => {
                (30.0, 24.0, 154, 1, 10800, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d16 — `Projectile.cs:7767-7776` (754, the Mystic Snake's
            // rope coil): 16x16, aiStyle 155, tileCollide false,
            // ignoreWater (the overlay), penetrate -1, alpha 255.
            MYSTIC_SNAKE_COIL => {
                (16.0, 16.0, 155, -1, TIME_LEFT_DEFAULT, 0, false, false, false, (false, -2, false, -1, 255, 1.0))
            }
            // W4-d16 — `Projectile.cs:7846-7854` (761/762, the paper
            // airplanes): 10x10, aiStyle 159, friendly + ranged,
            // manualDirectionChange (no ProjData field — the arm owns the
            // direction, the dispatch skips the auto-flip). Stock
            // penetrate 1 / timeLeft 3600 / tileCollide true / alpha 0.
            PAPER_AIRPLANE_A | PAPER_AIRPLANE_B => {
                (10.0, 10.0, 159, 1, TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)
            }
            // W4-d16 — `DefaultToKite` (`Projectile.cs:10744-10751`), the
            // 24 kinds the SetDefaults blocks route to it (`:7889-7892`/
            // `:8040-8044`/`:8055-8059`/`:8136-8140`/`:8149-8153`/
            // `:8168-8172`/`:8182-8186`): 4x4, aiStyle 160, penetrate -1,
            // extraUpdates 60 (the first tick's 61 sub-steps settle to 0
            // at the numUpdates==1 sub-step, `:48516-48519`). Stock
            // timeLeft 3600 / tileCollide true / alpha 0 / friendly false
            // (the helper sets none — the kite is a held marker, not a
            // shot; the item rows carry damage -1).
            k if crate::projectile_ai_kites::KITE_KINDS.contains(&k) => {
                (4.0, 4.0, 160, -1, TIME_LEFT_DEFAULT, 60, true, false, false, RESET_STATE)
            }
            // (d76's REMOTE_CONTROL_CAR row deduped into the W4-d80 arm
            // above — term-identical.)
            // W4-g82 — the four rope coils (`Projectile.cs:2570-2578` for
            // 171/505/506, `:5496-5504` for 475 — the two arms are
            // term-identical): 14x14, aiStyle 35, `penetrate = -1`,
            // `tileCollide = false`, `timeLeft = 400`. The stock
            // extraUpdates 0 / alpha 0 / scale 1 stand (`:779`/`:785`);
            // the aiStyle-35 machine + the chain preview live in
            // `projectile_rope_coil.rs` (the kind constants there).


            // `Projectile.cs:9446-9457` — the Junimo: 240x104, aiStyle 192
            // (the AI_192 fall/opacity animation), friendly, penetrate -1,
            // tileCollide false, ignoreWater, `timeLeft *= 5` (3600 → 18000).
            // The AI_192 arm is unported (the dispatch no-ops until it lands);
            // the row carries the box/flight so the spawn is faithful.

            // W4-f30 — the gravestone family (aiStyle 17, the falling
            // tombstone): 24x24, knockBack 12, penetrate -1, the stock
            // tileCollide/timeLeft (`Projectile.cs:1258-1270` the plain 43,
            // `:2872-2937` the grave markers 201-205, `:5971-5983` the rich
            // gravestones 527-531 — every row identical bar the type). The
            // FTW `friendly && hostile` pair is a spawn-time gate
            // (`DropTombstone`), not a SetDefaults row.

            // W4-f42 — the Djinn's curse (`Projectile.cs:6450-6460`):
            // 8×8, aiStyle 107, hostile, `alpha = 255`, `ignoreWater`,
            // `timeLeft = 180`, `tileCollide = false`. The stock penetrate 1
            // (`:774`) stands — the curse is a single-hit area burst (the
            // `timeLeft < 4` resize + `Damage()` arm, `:33631-33651`).

            // W4-f42 — the Antlion's sand ball (`Projectile.cs:1131-1140`):
            // 10×10, aiStyle 10, `penetrate = -1`, friendly + hostile (the
            // Antlion's call site sets friendly false). The stock 3600 life
            // (the call site overrides to 300, `NPC.cs:24464`); knockBack 6
            // is overridden by the call's `KnockBack: 0f`.

            // W4-g56 — `Projectile.cs:7726-7735`: the Void Lens (734):
            // 30x24, aiStyle 154, tileCollide false, `timeLeft = 10800`,
            // `alpha = 255` (Opacity 0 — AI_154 fades it in at 0.03/tick,
            // `:52187-52194`). The stock penetrate 1 / friendly false stand
            // (the type arm writes neither); drawLayer 3 is a draw-order
            // flag with no port field (the projectile pass draws it).

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
        // id-static table keys both under one cooldown. Every other row keeps
        // 0 = "use the kind" (the `:782` reset `immunityIdentity = type`).
        // W4-d11: the Spore Sac traps share 567 (`UseGroupStaticImmunity
        // (567)`, `Projectile.cs:6204`). W4-d19: the toxic clouds 511/512/513
        // share 511 the same way (`:5814`/`:5826`/`:5838`).
        immunity_identity: if matches!(kind, BEE | GIANT_BEE) {
            BEE
        } else if matches!(kind, SPORE_TRAP | SPORE_TRAP_2) {
            SPORE_TRAP
        } else if matches!(kind, SPORE_GAS | SPORE_GAS_2 | SPORE_GAS_3) {
            SPORE_GAS
        } else if matches!(kind, 511 | 512 | 513) {
            511
        } else {
            0
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
        // The `SetDefaults` `glowMask` slice (W4-g35): the glow-sheet id the
        // draw pass overlays (`Main.cs:34855-34942`); −1 = none (`:780`).
        glow_mask: set_defaults_glow_mask(kind),
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
            // W4-g2 (d5 F5): the Blood Rain's two clouds set `netImportant`
            // (`Projectile.cs:3257`/`:3265`); the drop 245 keeps false.
            || matches!(kind, BLOOD_CLOUD_MOVING | BLOOD_CLOUD_RAINING)
            // W4-d10 — the Coin Portal sets `netImportant` (`:5881`).
            || kind == COIN_PORTAL
            // W4-d12 — `Projectile.cs:6519`: the Portal gate's
            // `netImportant = true`.
            || kind == PORTAL_GUN_GATE
            // S1-P-i35 — the bobber family sets `netImportant` itself
            // (`Projectile.cs:4514`/`:9428`).
            || BOBBER_KINDS.contains(&kind)
            // The slice-B pets set `netImportant` themselves
            // (`Projectile.cs:5419`/`:9643`/`:9993`).
            || matches!(kind, CRIMSON_HEART | DIRTIEST_BLOCK | BOULDER_PET)
            // W4-d2 — the aiStyle-11 light pets set `netImportant = true`
            // (`Projectile.cs:995`/`:1524`/`:1676`/`:1690`).
            || matches!(kind, SHADOW_ORB | BLUE_FAIRY | PINK_FAIRY | GREEN_FAIRY)
            // W4-d7 F3 — the deferred pets + the Cob Whip spider set
            // `netImportant` themselves (`Projectile.cs:7018`/`:10169`/`:9774`).
            || matches!(kind, COMPANION_CUBE | RAINBOW_BOULDER_PET | COB_WHIP_SPIDER)
            // S1-P-i42 — the glowstick/flare family sets `netImportant = true`
            // (`Projectile.cs:1328`/`:1356`/`:2517`/`:4024`/`:5487`/`:5853`/
            // `:8345`/`:9582`/`:9593`/`:9604`/`:9615`/`:10155`).
            // W4-d17 FOLLOW-UP — `Projectile.cs:9774`: the Cob Whip spark.
            || kind == COB_WHIP_PROJ
            || matches!(
                kind,
                GLOWSTICK | STICKY_GLOWSTICK | SPELUNKER_GLOWSTICK | BOUNCY_GLOWSTICK
                    | FAIRY_GLOWSTICK | RAINBOW_GLOWSTICK | FLARE | BLUE_FLARE
                    | SPELUNKER_FLARE | CURSED_FLARE | RAINBOW_FLARE | SHIMMER_FLARE
            )
            // W4-d17 — `Projectile.cs:8026`: the chum bait (820) sets
            // `netImportant = true`.
            || kind == CHUM_BUCKET
            // W4-d17 — `Projectile.cs:8174`/`:8198`: the dove/lantern pair
            // (851/855) set `netImportant = true`.
            || matches!(kind, RELEASE_DOVES | RELEASE_LANTERN)
            // AI_026 slice B: the boss-head pets' `netImportant = true`
            // (`Projectile.cs:8488`/`:8523`).
            || matches!(kind, SKELETRON_PET | SKELETRON_PRIME_PET)
            // W4-d13 — the aiStyle-124 pets set `netImportant` themselves
            // (`Projectile.cs:6983`/`:8467`/`:8514`/`:8559`/`:8582`/`:9036`).
            || STYLE124_PETS.contains(&kind)
            // W4-d18: the multisegment pets' `netImportant = true`
            // (`Projectile.cs:8484`/`:8506`/`:8541`).
            || matches!(kind, EATER_OF_WORLDS_PET | DESTROYER_PET | LUNATIC_CULTIST_PET)
            // W4-g64 (d13) — the aiStyle-124 pets set `netImportant`
            // themselves (`Projectile.cs:8559`/`:8582`/`:9036` for the
            // three rowed kinds; d13's arm covers all seven).
            || matches!(kind, FAIRY_QUEEN_PET | ICE_QUEEN_PET | GLOMMER_PET)
            // W4-d20 T4a — the pet-buff shoot arm's spawn kinds set
            // `netImportant = true` (`Projectile.cs:8453`/`:8876`/
            // `:9438`/`:9026`).
            || matches!(kind, KING_SLIME_PET | QUEEN_SLIME_PET | JUNIMO_PET | BERNIE_PET)
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
            // W4-d9 — `Projectile.cs:5665`: the CultistRitual's netImportant.
            || kind == CULTIST_RITUAL
            // W4-d9 — `Projectile.cs:5647`: the MagicLantern's netImportant.
            || kind == MAGIC_LANTERN
            // S1-P-i55 — `Projectile.cs:7628`: the golf balls (721 + the 14
            // dyed twins) set `netImportant = true` themselves.
            || matches!(kind, GOLF_BALL | 739..=752)
            // S1-E-21h d75 part 6 — the phaseblade rows' `netImportant = true`
            // (`Projectile.cs:10014`/`:10030`/`:10055`, the 19-kind
            // `IsAPhaseblade` set).
            || matches!(kind, 1059..=1072 | 1074 | 1075 | 1076 | 1101 | 1102)
            // W4-d4 — `Projectile.cs:2428`: the Beach Ball (155) sets
            // `netImportant = true`.
            || kind == BEACH_BALL
            // W4-d15 — the DD2 pets set `netImportant = true` themselves
            // (`Projectile.cs:7439`/`:7872`/`:9959`).
            || matches!(
                kind,
                DD2_PET_DRAGON | DD2_PET_GHOST | DD2_PET_GATO | UPBEAT_STAR | AXE_FAIRY_PET
            )
            // W4-d16 — the 822+ kites set `netImportant = true` themselves
            // (`Projectile.cs:8042`/`:8057`/`:8138`/`:8151`/`:8170`/
            // `:8184`); the 766-771 block (`:7889-7892`) keeps the stock
            // false (the `DefaultToKite` helper sets none).
            || matches!(kind, 822..=824 | 826..=830 | 838..=840 | 843..=846 | 850 | 852..=853)
            // W4-d20 — `Projectile.cs:10076`: the meteor ore (1078) sets
            // `netImportant = true`.
            || kind == METEOR_ORE
            // W4-d21 — the storm-lightning trap + the two digtoises set
            // `netImportant = true` (`Projectile.cs:10178`/`:10248`/`:10511`).
            || matches!(kind, STORM_LIGHTNING | DIGTOISE | TRUSTY_DIGTOISE)
            // W4-g51 id-144 — `Projectile.cs:9448`: the Stardew Jumino
            // animation sets `netImportant = true`.
            || kind == STARDEW_JUMINO
            // W4-c39 — `Projectile.cs:8175`/`:8199`: the released dove (851)
            // and lantern (855) set `netImportant = true`.
            || matches!(kind, RELEASE_DOVES | RELEASE_LANTERN),
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
            // W4-d1 — `Projectile.cs:5676`: the Crystal Vile Shard pair's
            // `ignoreWater = true` (the shared `:5667-5680` arm).
            || matches!(kind, CRYSTAL_VILE_HEAD | CRYSTAL_VILE_SHAFT)
            // The slice-B pets set `ignoreWater` themselves
            // (`Projectile.cs:5422`/`:9646`/`:9996`) — they are not minions.
            || matches!(kind, CRIMSON_HEART | DIRTIEST_BLOCK | BOULDER_PET)
            // W4-d2 — the aiStyle-11 light pets' `ignoreWater = true`
            // (`Projectile.cs:1005`/`:1533`/`:1685`/`:1699`).
            || matches!(kind, SHADOW_ORB | BLUE_FAIRY | PINK_FAIRY | GREEN_FAIRY)
            // W4-d3 — the 1047 RainbowBoulder sets `ignoreWater`
            // (`Projectile.cs:9926`).
            || kind == RAINBOW_BOULDER
            // W4-d7 F3 — the Companion Cube + Rainbow Boulder Pet set
            // `ignoreWater` themselves (`Projectile.cs:7021`/`:10173`); the
            // Cob Whip spider keeps the stock false (`:9767-9779` sets none).
            || matches!(kind, COMPANION_CUBE | RAINBOW_BOULDER_PET)
            // W4-d13 — the aiStyle-124 pets set `ignoreWater` themselves
            // (`Projectile.cs:6986`/`:8470`/`:8516`/`:8562`/`:8584`/`:9043`).
            || STYLE124_PETS.contains(&kind)
            // W4-d13 — the Wire Kite / geyser / stomp / sandnado mark set
            // `ignoreWater` (`Projectile.cs:6996`/`:7031`/`:7181`/`:7079`).
            || matches!(kind, WIRE_KITE | GEYSER_TRAP | OGRE_STOMP | SANDNADO_MARK)
            // W4-d13 residual — the aiStyle-108 ritual markers' shared block
            // sets `ignoreWater` (`Projectile.cs:6290`); the 578/579 rows
            // predated the flag list, the 813 joins them.
            || matches!(kind, VORTEX_LIGHTNING | VORTEX_PORTAL | BLOOD_NAUTILUS_TEARS)
            // AI_026 slice A: the Tiki Spirit (`Projectile.cs:2860`) and the
            // Wisp (`:3002`) set `ignoreWater`; the other flyers keep false.
            || matches!(kind, TIKI_SPIRIT | WISP)
            // W4-g64 (d13) — the aiStyle-124 pets set `ignoreWater`
            // themselves (`Projectile.cs:8562`/`:8584`/`:9043`).
            || matches!(kind, FAIRY_QUEEN_PET | ICE_QUEEN_PET | GLOMMER_PET)
            // `Projectile.cs:9244`: the Light's Bane slash ignores water.
            || kind == LIGHTS_BANE
            // `Projectile.cs:6949`: the crystal shard's `ignoreWater = true`.
            || kind == RAINBOW_CRYSTAL_EXPLOSION
            // W4-d12 — `Projectile.cs:6395`: the Truffle spore's
            // `ignoreWater = true` (the 836 seed keeps the stock false).
            || kind == TRUFFLE_SPORE
            // W4-d12 — `Projectile.cs:6550`: the Solar flare ray's
            // `ignoreWater = true`.
            || kind == SOLAR_FLARE_RAY
            // W4-d12 — the aiStyle-117 explosion family's `ignoreWater`
            // (`Projectile.cs:6562`/`:6591`/`:6724`/`:8995`/`:9882`).
            || matches!(
                kind,
                SOLAR_COUNTER | SOLAR_WHIP_SWORD_EXPLOSION | STARDUST_GUARDIAN_EXPLOSION
                    | DAYBREAK_EXPLOSION | DEAD_CELLS_MUSHROOM_BOI_EXPLOSION
            )
            // W4-d12 DOC-SWEEP item 7 — `Projectile.cs:9260`: the Volcano
            // explosion's `ignoreWater = true`.
            || kind == VOLCANO
            // `Projectile.cs:9517` — the Horseman's pumpkin lob (1001).
            || kind == HORSEMAN_PUMPKIN
            // S1-D-3 W-9: the sharknado clouds' `ignoreWater = true`
            // (`Projectile.cs:4672`/`:4694`).
            || matches!(kind, FORCE_BUBBLE | CTHULUNADO)
            // W4-c10 — the BirdDroppings' `ignoreWater = true`
            // (`Projectile.cs:10085`).
            || kind == BIRD_DROPPINGS
            // W4-f42: the Djinn's curse sets `ignoreWater = true`
            // (`Projectile.cs:6457`).
            || kind == DESERT_DJINN_CURSE
            // S1-P-i21: the Nimbus Rod's raining cloud sets `ignoreWater`
            // (`Projectile.cs:3195`); the moving cloud 237 and the drop 239
            // keep the stock false.
            || kind == RAIN_CLOUD_RAINING
            // W4-g2 (d5 F5/F1): the blood cloud 244 (`Projectile.cs:3267`),
            // the blood drop 245 (`:3276`), the hostile nimbus drop 264
            // (`:3493`) and the UFO laser 433 (`:5045`) set `ignoreWater`;
            // the moving blood cloud 243 keeps the stock false (`:3255-
            // 3262` has no such line — the 237 twin).
            || matches!(kind, BLOOD_CLOUD_RAINING | BLOOD_RAIN | RAIN_NIMBUS | UFO_LASER)
            // W4-d10 — the Coin Portal sets `ignoreWater` (`:5887`); the
            // Ichor Splash too (`:5947`); the Medusa Head marker (`:6105`).
            || matches!(kind, COIN_PORTAL | ICHOR_SPLASH | MEDUSA_HEAD)
            // S1-D-4: the Empress family's five arms all set `ignoreWater`
            // (`Projectile.cs:8365`/`:8378`/`:8414`/`:8725`/`:8768`).
            || matches!(
                kind,
                HALLOW_BOSS_SPLIT_SHOT_CORE
                    | HALLOW_BOSS_LASTING_RAINBOW
                    | HALLOW_BOSS_RAINBOW_STREAK
                    | HALLOW_BOSS_DEATH_AURORA
                    | FAIRY_QUEEN_LANCE
                    | FAIRY_QUEEN_SUN_DANCE
            )
            // W4-d18 tail: the two aiStyle-171 weapon variants'
            // `ignoreWater = true` (`Projectile.cs:8837`/`:9816`).
            || matches!(kind, FAIRY_QUEEN_MAGIC_ITEM_SHOT | CONSTELLATION_STAR)
            // W4-d18 tail: the friendly Insanity shadow (`:9137`) sets
            // `ignoreWater = true`; the Foxsparks flames (1106) do NOT
            // (`:10303-10315` has no line). PROJ-171 retire: the 978
            // Volcano's `ignoreWater` term (`:9260`) is DROPPED with its
            // row (keep-d12c — d12c's 9e668d0f0 owns the 978 overlay).
            || kind == INSANITY_SHADOW_FRIENDLY
            // W4-d18: the multisegment pets' `ignoreWater = true`
            // (`Projectile.cs:8482`/`:8504`/`:8539`).
            || matches!(kind, EATER_OF_WORLDS_PET | DESTROYER_PET | LUNATIC_CULTIST_PET)
            // W4-d18: the titanium storm shard's `ignoreWater = true`
            // (`Projectile.cs:8629`).
            || kind == TITANIUM_STORM_SHARD
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
            // W4-f20 F3 — the Sky Dragon's Fury circle 1110's `ignoreWater`
            // (`Projectile.cs:10349`, the `:10340-10352` row block).
            || kind == SKY_DRAGONS_FURY_ELECTROSPHERE
            // S1-P-i14 — the bullet-launch projectiles that set `ignoreWater`:
            // the Xenopopper bubble 444 (`Projectile.cs:5159`) and the Onyx
            // black bolt 661 (`:7118`).
            || matches!(kind, XENOPOPPER_BUBBLE | ONYX_BLACK_BOLT)
            // S1-P-i18 — `Projectile.cs:3251`: the High Velocity Bullet's
            // `ignoreWater = true`.
            || kind == HIGH_VELOCITY_BULLET
            // W4-d9 — `Projectile.cs:5360`: the CultistBossIceMist's
            // `ignoreWater = true`.
            || kind == CULTIST_BOSS_ICE_MIST
            // W4-d9 — `Projectile.cs:5581`: the ClingerStaff's `ignoreWater`.
            || kind == CLINGER_STAFF
            // W4-d9 — `Projectile.cs:5662`: the CultistRitual's `ignoreWater`.
            || kind == CULTIST_RITUAL
            // W4-d8 — the aiStyle-72 Bubble (410, `Projectile.cs:4927`) and
            // the two aiStyle-76 mount crosshairs (441/453,
            // `:5123`/`:5256`) set `ignoreWater = true`.
            || matches!(kind, BUBBLE | SCUTLIX_LASER_CROSSHAIR | DRILL_MOUNT_CROSSHAIR)
            // W4-d8 DOC-SWEEP (3) — the Scutlix mount's fired laser (606,
            // `Projectile.cs:6540`) sets `ignoreWater = true`.
            || kind == SCUTLIX_LASER
            // W4-d11 — the aiStyle-101 rays set `ignoreWater`
            // (`Projectile.cs:6118` the 536, `:6407` the 591).
            || matches!(kind, MEDUSA_HEAD_RAY | MINECART_MECH_LASER)
            // W4-d11 — the 565 BrainOfConfusion sets `ignoreWater`
            // (`Projectile.cs:6168`).
            || kind == BRAIN_OF_CONFUSION
            // W4-d11 — the Spore Sac traps set `ignoreWater`
            // (`Projectile.cs:6203`).
            || matches!(kind, SPORE_TRAP | SPORE_TRAP_2)
            // W4-d11 — the SporeGas clouds set `ignoreWater`
            // (`Projectile.cs:6216`).
            || matches!(kind, SPORE_GAS | SPORE_GAS_2 | SPORE_GAS_3)
            // W4-d11 — the 596 DesertDjinnCurse sets `ignoreWater`
            // (`Projectile.cs:6457`); the 575 sphere does NOT
            // (`:6219-6229` has no such line).
            || kind == DESERT_DJINN_CURSE
            // W4-d11 RETIRE item 3 (the g64 remainder-4 / d13 a2588cd2
            // residual adopted) — the aiStyle-108 ritual markers' shared
            // block sets `ignoreWater` (`Projectile.cs:6290`); the 578/579
            // rows predated the flag list, the 813 joins them.
            || matches!(kind, VORTEX_LIGHTNING | VORTEX_PORTAL | BLOOD_NAUTILUS_TEARS)
            // W4-d16 — `Projectile.cs:7679`/`:9215`/`:9277`/`:9897`/`:7722`/
            // `:7773`: the four aiStyle-152 slashes + the toilet pop + the
            // snake coil set ignoreWater.
            || matches!(kind, SUPER_STAR_SLASH | ZENITH_MURAMASA_SLASH | ZENITH_GRASS_SLASH | MOON_LORD_WHIP_PROC | TOILET_EFFECT | MYSTIC_SNAKE_COIL)
            // S1-P-i55 — `Projectile.cs:7642`: the club helper (722) sets
            // `ignoreWater = true`.
            || kind == GOLF_CLUB_HELPER
            // W4-c3 — `Projectile.cs:3903`: the Shadowflames bolt (299) sets
            // `ignoreWater = true`.
            || kind == SHADOWFLAMES
            // W4-d14 — `Projectile.cs:7215`: the Dark Mage cast markers
            // (673/674) set `ignoreWater = true` (the 673 row's flag was
            // missing from the overlay; the 674 heal twin shares it).
            // `:7203`: the 672 victory marker sets it too.
            || matches!(kind, DARK_MAGE_MARKER | DARK_MAGE_HEAL | DD2_ELDER_WINS)
            // `Projectile.cs:7289`/`:8752` — the Ogre/QueenSlime stomps.
            || matches!(kind, OGRE_SMASH | QUEEN_SLIME_SMASH)
            // `Projectile.cs:7379` — the monk staff T1 swing (the 698/707
            // twins fold into the d15 family list below, :7392/:7486).
            || kind == MONK_STAFF_T1
            // W4-d15 — the DD2 slice's `ignoreWater = true` rows: the 698
            // (`Projectile.cs:7392`), the 699 (`:7414`), the 700 (`:7430`),
            // the 707 (`:7486`), the 708 (`:7503`) and the 709 (`:7516`).
            || matches!(
                kind,
                MONK_STAFF_T1_EXPLOSION | MONK_STAFF_T2 | MONK_STAFF_T2_GHAST
                    | MONK_STAFF_T3 | MONK_STAFF_T3_ALT | MONK_STAFF_T3_ALT_SHOT
            )
            // W4-d15 — the DD2 pets' `ignoreWater = true`
            // (`Projectile.cs:7443`/`:7875`/`:9963`).
            || matches!(
                kind,
                DD2_PET_DRAGON | DD2_PET_GHOST | DD2_PET_GATO | UPBEAT_STAR | AXE_FAIRY_PET
            )
            // W4-d15 — the Book Staff twister (`Projectile.cs:7459`) and
            // the victory banner (`:7567`).
            || matches!(kind, DD2_APPRENTICE_STORM | DD2_VICTORY)
            // W4-d17 — `Projectile.cs:8013`: the White Tiger Pounce (818)
            // sets `ignoreWater = true`.
            || kind == WHITE_TIGER_POUNCE
            // W4-d17 — `Projectile.cs:8229`: the First Fractal (857) sets
            // `ignoreWater = true`.
            || kind == FIRST_FRACTAL
            // W4-d18 — the seven solution sprays' `ignoreWater = true`
            // (`Projectile.cs:2358`/`:2370`/`:2382`/`:2394` — the four
            // 146-149 blocks — and `DefaultToSpray` `:10701` for
            // 1015-1017). The 145 row's dropped alpha/ignoreWater is the
            // pre-existing M3-ITEM-6 deviation (its row comment), unchanged.
            || matches!(kind, 146..=149 | 1015..=1017)
            // W4-g5 — `Projectile.cs:6457`: the Djinn's curse (596) sets
            // `ignoreWater = true` (the `wet = false` spawn write
            // `:10886-10889`).
            || kind == DJINN_CURSE
            // W4-d1 — the aiStyle-6 powder/fertilizer family's five arms
            // all set `ignoreWater = true` (`Projectile.cs:906`/`:922`/
            // `:5349`/`:9657`/`:9976`).
            || POWDER_KINDS.contains(&kind)
            // W4-d1 — `Projectile.cs:991`: the DirtBall 17's
            // `ignoreWater = true` (the only falling-family kind that sets
            // it; the natural falls' arms touch neither).
            || kind == DIRT_BALL
            // W4-d1 retire (the spray compose) — the seven solution sprays'
            // `ignoreWater = true` (`Projectile.cs:2360`/`:2372`/`:2384`/
            // `:2396` — the four 146-149 blocks — and `DefaultToSpray`
            // `:10701` for 1015-1017). The 145 row's dropped
            // alpha/ignoreWater stays the pre-existing M3-ITEM-6 deviation
            // (its row comment), unchanged.
            || matches!(kind, 146..=149 | 1015..=1017)
            // W4-d4 — `Projectile.cs:2735`: the Spear Trap's spear (186)
            // sets `ignoreWater = true` (the row landed with the 37 arm).
            || kind == TRAP_SPEAR
            // W4-d4 — `Projectile.cs:2748`: the flame jet (187) sets
            // `ignoreWater = true` (the row landed with the 38 arm).
            || kind == FLAMETHROWER_TRAP
            // W4-d6 — `Projectile.cs:4147`: the FlamingJack (321) ignores water.
            || kind == FLAMING_JACK
            // W4-d6 residual 5 — `Projectile.cs:3962`: the VampireKnife.
            || kind == VAMPIRE_KNIFE
            // W4-d6 — `Projectile.cs:4495`/`:4876`/`:9856`: the three gun
            // streams ignore water.
            || matches!(kind, WATER_GUN | SLIME_GUN | SHIMMER_GUN_STREAM)
            || matches!(kind, 146..=149 | 1015..=1017)
            || matches!(kind, 146..=149 | 1015..=1017)
            // W4-d19 — the sword-style rows' `ignoreWater = true`
            // (`Projectile.cs:5514`/`:8445`/`:8811`/`:8865`/`:10273`/
            // `:8964`/`:9308`/`:9364`/`:9381`/`:9398`/`:9476`/`:9868`/
            // `:10553`). The 949 Torch God sets NONE (`:8948-8955`).
            || matches!(
                kind,
                SOUL_DRAIN | ZOOLOGIST_STRIKE_GREEN | ZOOLOGIST_STRIKE_RED
                    | FINAL_FRACTAL | TRUE_COPPER_SHORTSWORD | PRINCESS_WEAPON
                    | NIGHTS_EDGE | EXCALIBUR | TRUE_EXCALIBUR | TERRA_BLADE_2
                    | HORSEMANS_BLADE | DEAD_CELLS_FLINT_SLASH | CHLOROPHYTE_CLAYMORE_BLADE
            )
            // W4-d20 — `Projectile.cs:10085`: the bird dropping (1080) sets
            // `ignoreWater = true`.
            || kind == BIRD_DROPPINGS
            // W4-d20 — `Projectile.cs:9804`/`:10423`: the two aiStyle-196
            // petals set `ignoreWater = true`.
            || matches!(kind, FLOWER_WHIP_PETAL | PETALSTORM_SHOT)
            // W4-d20 — `Projectile.cs:9455`: the jumino animation (995) sets
            // `ignoreWater = true`.
            || kind == JUMINO_STARDROP_ANIMATION
            // W4-d20 — `Projectile.cs:9827`/`:9843`/`:9868`: the flint trio
            // (1040/1041/1043) set `ignoreWater = true`.
            || matches!(kind, DEAD_CELLS_FLINT | DEAD_CELLS_FLINT_SHOT | DEAD_CELLS_FLINT_SLASH)
            // W4-d20 — `Projectile.cs:9325`/`:9415`: the two aiStyle-191
            || matches!(kind, TRUE_NIGHTS_EDGE | TERRA_BLADE_2_SHOT)
            // W4-d21 — the storm-lightning trio + the Glacier Fang shot +
            // the blade slam set `ignoreWater` (`Projectile.cs:10182`/
            // `:10431`/`:10441`/`:10408`/`:10587`).
            || matches!(kind, STORM_LIGHTNING | LIGHTNING_STRIKE_SHOT | ARC_SURGE | GLACIER_FANG_SHOT | CHLOROPHYTE_CLAYMORE_BLADE_SLAM)
            // S1-E-21h d76 DOC-SWEEP — `Projectile.cs:10085`: the Bird
            // Droppings (1080) sets `ignoreWater = true`.
            || kind == BIRD_DROPPINGS
            // M2-MOUNT-4 — `Projectile.cs:6540`: the Scutlix laser (606)
            // sets `ignoreWater = true`.
            || kind == SCUTLIX_LASER
            || matches!(kind, WATER_GUN | SLIME_GUN | SHIMMER_GUN_STREAM)
            // W4-d4 — `Projectile.cs:2735`: the Spear Trap's spear (186)
            // sets `ignoreWater = true` (the row landed with the 37 arm).
            || kind == TRAP_SPEAR
            // W4-d4 — `Projectile.cs:2748`: the flame jet (187) sets
            // `ignoreWater = true` (the row landed with the 38 arm).
            || kind == FLAMETHROWER_TRAP
            || matches!(kind, 146..=149 | 1015..=1017)
            // S1-W4-g81 residuals — `Projectile.cs:6996`: the Wire Kite
            // (651) sets `ignoreWater = true`.
            || kind == WIRE_KITE
            || matches!(kind, 146..=149 | 1015..=1017)
            // W4-g51 id-144 — `Projectile.cs:9455`: the Stardew Jumino
            // animation sets `ignoreWater = true`.
            || kind == STARDEW_JUMINO
            // W4-g64 (the d11 row port) — the 596 DesertDjinnCurse sets
            // `ignoreWater` (`Projectile.cs:6457`); the 575 sphere does
            // NOT (`:6219-6229` has no such line — d11's census).
            || kind == DESERT_DJINN_CURSE
            // W4-g64 remainder 4 (the d13 a2588cd2 residual) — the
            // aiStyle-108 ritual markers' shared block sets `ignoreWater`
            // (`Projectile.cs:6290`); the 578/579 rows predated the flag
            // list, the 813 joins them.
            || matches!(kind, VORTEX_LIGHTNING | VORTEX_PORTAL | BLOOD_NAUTILUS_TEARS)
            // W4-d80 — the Sky Dragon's Fury shot (709, `:7516`).
            || kind == SKY_DRAGONS_FURY_SHOT
            // S1-E-21h d76 DOC-SWEEP — `Projectile.cs:10085`: the Bird
            // Droppings (1080) sets `ignoreWater = true`.
            || kind == BIRD_DROPPINGS,
        // The aiStyle-67 summon rows set `decidesManualFallThrough`
        // (`:4767`/`:7825`/`:8092`/`:8106`/`:8120`); the pirates/frog's arm
        // drives `shouldFallThrough` (`:49561`/`:51206`).
        decides_manual_fall_through: matches!(
            kind,
            ONE_EYED_PIRATE | SOULSCOURGE_PIRATE | PIRATE_CAPTAIN | VAMPIRE_FROG
                | STORM_TIGER_1 | STORM_TIGER_2 | STORM_TIGER_3
                // W4-d7 wiring — the BabySpider's `decidesManualFallThrough`
                // (`Projectile.cs:4643`); the aiStyle-63 arm drives
                // `shouldFallThrough` (`:29345`/`:29349`).
                | BABY_SPIDER
                // W4-d7 F3 — the Cob Whip spider's `decidesManualFallThrough`
                // (`Projectile.cs:9778`).
                | COB_WHIP_SPIDER
                // S1-P-i25: the new aiStyle-67 rows' `decidesManualFallThrough`
                // — the flinx (`:8980`), the mushroom boi (`:9679`), the
                // cattivas (`:10207`/`:10374`) and the clay pot (`:10462`).
                | 951 | 1022 | 1093 | 1112 | 1118
                // W4-d17 FOLLOW-UP — `Projectile.cs:9778`: the Cob Whip
                // spark's `decidesManualFallThrough = true`.

        ),
        // The damage-class flags (`Projectile.cs:751-753` reset false) the
        // per-hit crit block reads (`:12890-12898`). The census is the
        // `SetDefaults` `this.melee/ranged/magic = true` writes plus the
        // `DefaultTo*` helpers: yoyo/flail/spear/drill/shortsword are melee,
        // the whip sets NONE (`DefaultToWhip` `:10704-10718`). The bombs
        // (28/29/37/519) and the Crystal Storm item 518's proj set none.
        melee: YOYO_KINDS.contains(&kind)
            // W4-d10 — the counterweights ride `DefaultToYoyo` too
            // (`Projectile.cs:6089` → `:10770`).
            || is_counterweight(kind)
            || FLAIL_KINDS.contains(&kind)
            // W4-d17 — `DefaultToShortsword` sets melee (`Projectile.cs:10783`):
            // the rapier-stab family (802/842/938-945/1127).
            || matches!(kind, GLADIUS_STAB | RULER_STAB | SHORTSWORD_STAB_FIRST..=SHORTSWORD_STAB_LAST | CHLOROPHYTE_SABER_STAB)
            // W4-d17 — `Projectile.cs:8227`: the First Fractal (857) sets
            // `melee = true`.
            || kind == FIRST_FRACTAL
            || matches!(
                kind,
                // The explicit melee rows: the boomerangs (6/52/113,
                // `:871`/`:1352`/`:2019`), the sword beams (116/132/156/157/173,
                // `:2052`/`:2214`/`:2439`/`:2452`/`:2596`), the Light's Bane
                // slash (974, `:9245`), the Terrarian beam (604, `:6079`), the
                // yoyo on-hit shots (1125/1130/1134/1135,
                // `:10535`/`:10605`/`:10648`/`:10661`), the flail helpers
                // (248/405/928, `:3312`/`:4866`/`:8801`).
                // S1-E-21h d75 part 5 — the Meowmere star (502, `:5738`).
                // S1-E-21h d75 part 6 — the 19 phaseblade/phaser kinds
                // (`ProjectileID.cs:192`'s `IsAPhaseblade` set; the melee
                // writes `:10013`/`:10029`/`:10054`).
                ENCHANTED_BOOMERANG | WOODEN_BOOMERANG | ICE_BOOMERANG
                    | SWORD_BEAM | TERRA_BEAM | LIGHT_BEAM | NIGHT_BEAM | ENCHANTED_BEAM
                    | LIGHTS_BANE | TERRARIAN_BEAM | MEOWMERE
                    | 1059..=1072 | 1074 | 1075 | 1076 | 1101 | 1102
                    | CASCADE_EXPLOSION | EYE_OF_CTHULHU_DASH | CRYSTAL_SHARD_MELEE | KRAKEN_WAVE
                    | FLOWER_POW_PETAL | FLAIRON_BUBBLE | DRIPPLER_EXTRA_BALL
                    // PROJ-171 retire: the 978 Volcano's `melee = true`
                    // term (`Projectile.cs:9267`) is DROPPED with its row
                    // (keep-d12c — d12c's 9e668d0f0 owns the 978 melee flag).
                    // W4-d9 — the InfluxWaver's melee wave (`Projectile.cs:5234`).
                    | INFLUX_WAVER
                    // W4-f20 F3 — the Sky Dragon's Fury circle 1110's
                    // `this.melee = true` (`Projectile.cs:10348`).
                    | SKY_DRAGONS_FURY_ELECTROSPHERE
                    // W4-g2 (d5 F6): the Partisan's orb sets `melee = true`
                    // (`Projectile.cs:3157`). The 228/732 siblings set it too
                    // (`:3144`/`:7713`) — the W4-g2 reconciliation round closes
                    // the recorded gap (the spore cloud 228 and the Thunder
                    // Spear's bolt 732 join the melee census).
                    | CHLOROPHYTE_ORB | CHLOROPHYTE_SPORE_CLOUD | THUNDER_SPEAR_BOLT
                    // W4-d1 — `Projectile.cs:5752`: the Star Wrath star's
                    // `melee = true` (the Starfury 9's `:893` is the S1-P-i22
                    // row's documented drop — the 503 carries it verbatim).
                    | STAR_WRATH
                    // W4-d1 SEAL — `Projectile.cs:5410`: the FlyingKnife's
                    // `melee = true`.
                    | FLYING_KNIFE
                    // W4-d2 — the aiStyle-13 melee chains (`Projectile.cs:
                    // 3471`/`:3575`/`:3596`/`:5569`).
                    | GOLEM_FIST | BOXING_GLOVE | CHAIN_KNIFE | CHAIN_GUILLOTINE
                    // W4-d6 — the FlamingJack's `melee = true` (`:4145`).
                    | FLAMING_JACK
                    // W4-d6 residual 5 — the VampireKnife's `melee = true`
                    // (`Projectile.cs:3960`).
                    | VAMPIRE_KNIFE
                    // W4-d7 wiring — the Flairon recall head's `melee = true`
                    // (`Projectile.cs:4855`).
                    | FLAIRON_HEAD
                    // W4-d7 wiring (F4) — the Ale's `melee = true`
                    // (`Projectile.cs:7170`).
                    | ALE
                    // W4-d21 — the thrown melee weapons (1082/1083,
                    // `:10102`/`:10112`) + the blade slam (1129, `:10593`).
                    | ANTLION_CLAW | STYLIST_SCISSORS | CHLOROPHYTE_CLAYMORE_BLADE_SLAM
                    // W4-g2 (d5 F6): the Partisan's orb sets `melee = true`
                    // (`Projectile.cs:3157`). The 228/732 siblings set it too
                    // (`:3144`/`:7713`) — the W4-g2 reconciliation round closes
                    // the recorded gap (the spore cloud 228 and the Thunder
                    // Spear's bolt 732 join the melee census).

                    // W4-f48 lane 20 — the Bouncing Shield (866, `:8320`).
                    | BOUNCING_SHIELD
                    // W4-e7 lane 23 — the Snapping Stone melee effect
                    // (1131, `:10620`); the strike's crit-comparison flip
                    // (`:14343-14345`) overwrites the slot flags at spawn.
                    | SNAPPING_STONE_MELEE_EFFECT
                    // W4-d9 — the InfluxWaver's melee wave (`Projectile.cs:5234`).

                // W4-d10 — the Chlorophyte Claymore's spore cloud (1128,
                // `:10572`) and the Ichor Flask's splash (524, `:5948`).
                    | SPORE_CLOUD | ICHOR_SPLASH
                    // W4-e3 coverage phase — the Star Wrath star sets
                    // `melee = true` (`Projectile.cs:5752`).

                    // W4-d12 DOC-SWEEP item 7 — the Volcano explosion's melee
                    // flag (`Projectile.cs:9267`). BATCH-D keep-one: d18's
                    // PROJ-171 retire dropped ITS copy (keep-d12c — the
                    // merged d12 branch owns the 978 row + melee flag).
                    | VOLCANO
                    // W4-d12 F1 — the aiStyle-113 melee sticks: the Daybreak
                    // (636, `:6844`), the Tentacle Spike (971, `:9203`) and
                    // the Blood Butcherer (975, `:9293`).
                    | DAYBREAK | TENTACLE_SPIKE | BLOOD_BUTCHERER
                    // W4-d14 — the monk staff T1 swing (`Projectile.cs:7372`).
                    | MONK_STAFF_T1
                    // W4-d15 — the Sleepy Octopod's blast (`:7398`), the
                    // Monk Staff swings (`:7408`/`:7480`/`:7497`), the ghast
                    // (`:7427`) and the volley shot (`:7514`).
                    | MONK_STAFF_T1_EXPLOSION | MONK_STAFF_T2 | MONK_STAFF_T2_GHAST
                    | MONK_STAFF_T3 | MONK_STAFF_T3_ALT | MONK_STAFF_T3_ALT_SHOT
                    // W4-d16 — the three melee aiStyle-152 slashes: the
                    // Muramasa (`:9216`), the Blade of Grass (`:9278`),
                    // the Moon Lord whip proc (`:9898`).
                    | ZENITH_MURAMASA_SLASH | ZENITH_GRASS_SLASH | MOON_LORD_WHIP_PROC
                    // W4-d19 — the Zenith illusions (`:8863`/`:10271`) and
                    // the seven aiStyle-190 slashes (`:9304`/`:9360`/
                    // `:9377`/`:9394`/`:9472`/`:9864`/`:10549`).
                    | FINAL_FRACTAL | TRUE_COPPER_SHORTSWORD
                    | NIGHTS_EDGE | EXCALIBUR | TRUE_EXCALIBUR | TERRA_BLADE_2
                    | HORSEMANS_BLADE | DEAD_CELLS_FLINT_SLASH | CHLOROPHYTE_CLAYMORE_BLADE
                    // W4-d19 follow-up — the spore cloud's melee
                    // (`Projectile.cs:10572`).

                    // W4-d20 - `Projectile.cs:9841`: the flint spike (1041)
                    // sets melee (the 1043 slash already rides the d19
                    // seven-slash list above - BATCH-D keep-d19 drop-d20).
                    | DEAD_CELLS_FLINT_SHOT
                    // W4-d20 - `Projectile.cs:9321`/`:9411`: the two
                    // aiStyle-191 sword shots (973/985) set melee.
                    | TRUE_NIGHTS_EDGE | TERRA_BLADE_2_SHOT
                    // W4-d21 — the thrown melee weapons (1082/1083,
                    // `:10102`/`:10112`) + the blade slam (1129, `:10593`).

                    // W4-d80 — the Sky Dragon's Fury shot (709, `:7514`).

            ),
        ranged: matches!(
            kind,
            // W4-d16 — `Projectile.cs:7666`/`:7680`/`:7852`: the Super Star
            // (728), its slash (729) and the paper airplanes (761/762) set
            // `ranged = true` (the 151/152/159 rows).
            SUPER_STAR | SUPER_STAR_SLASH | PAPER_AIRPLANE_A | PAPER_AIRPLANE_B |
            // The arrows/bullets (1/2/14, `:814`/`:825`/`:961`), the thrown
            // kinds (3/48/166, `:835`/`:1317`/`:2555`), the moon-lord drop
            // shots (638/639/640, `:6867`/`:6880`/`:6895`), the DD2 kobold
            // bomb (681, `:7270`).
            WOODEN_ARROW | FLAME_ARROW | MUSKET_BALL
                | SHURIKEN | THROWING_KNIFE | SNOWBALL
                | MOONLORD_BULLET | MOONLORD_ARROW | MOONLORD_ARROW_TRAIL
                | DD2_BOMB
                // W4-d2 — the Harpoon's `this.ranged = true`
                // (`Projectile.cs:1065`).
                | HARPOON
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
            // M2-MOUNT-4 — the Santank gift-box rocket (930,
            // `Projectile.cs:8824` sets ranged).
                | SANTANK_MOUNT_ROCKET
            // The hostile snowmen's shots (109/110, `:1976`/`:1989`) — the
            // C# writes ranged on them too; the owner-255 gate keeps the
            // flags moot, the census stays verbatim.
                | SNOW_BALL_HOSTILE | BULLET_SNOWMAN
            // W4-d1 SEAL — `Projectile.cs:9351`: the SilverBullet's
            // `ranged = true`.
                | SILVER_BULLET
            // W4-d4 — `Projectile.cs:2787`: the Mechanical Piranha (190)
            // sets `ranged = true`.
                | MECHANICAL_PIRANHA
            // W4-e7 lane 23 — the Snapping Stone ranged effect (1121,
            // `:10500`); the summon effect 1132 sets NO class (`:10624-10635`).
                | SNAPPING_STONE_EFFECT
            // W4-d10 — the Nail Gun's nail (514, `:5849`) and the Toxic
            // Flask's bubble (523, `:5935`).
                | NAIL_FRIENDLY | TOXIC_BUBBLE
            // W4-d80 — the Phantom Phoenix shot (706, `:7469`).
                | PHANTOM_PHOENIX
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
                // W4-d1 — `Projectile.cs:5677`: the Crystal Vile Shard pair's
                // `magic = true` (the shared `:5667-5680` arm).
                | CRYSTAL_VILE_HEAD | CRYSTAL_VILE_SHAFT
                | BALL_OF_FIRE | MAGIC_MISSILE | GREEN_LASER | WATER_BOLT | FLAMELASH
                | DEMON_SCYTHE | RAINBOW_ROD_BULLET | ICE_BLOCK | PURPLE_LASER
                | CRYSTAL_STORM | CURSED_FLAME_FRIENDLY
                | GEM_BOLT_AMETHYST | GEM_BOLT_TOPAZ | GEM_BOLT_SAPPHIRE | GEM_BOLT_EMERALD
                | GEM_BOLT_RUBY | GEM_BOLT_DIAMOND | GEM_BOLT_AMBER
                | UNHOLY_TRIDENT | UNHOLY_TRIDENT_HOSTILE
                | BALL_OF_FROST | MAGNET_SPHERE_BALL | MAGNET_SPHERE_BOLT
                | INFERNO_FORK_BOLT | INFERNO_BLAST | LOST_SOUL | SPECTRE_WRATH
                // W4-d6 — the SpiritHeal's `magic = true` (`:3889`); the
                // VampireHeal 305 sets none (classless, `:3965-3973`).
                | SPIRIT_HEAL
                | VENOM_FANG | FROST_BOLT | TYPHOON | LEAF
                | NEBULA_ARCANUM | NEBULA_ARCANUM_SUBSHOT
                // The Arcanum's death shard (620, `:6687`) — d77 + d78's
                // co-claim, ONE set member at the merge (the two constants
                // alias kind 620, research/372 §3).
                | NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD
                | LAST_PRISM_LASER | LAST_PRISM | NEBULA_BLAZE_1 | NEBULA_BLAZE_2
                | POISON_DART | SPIRIT_FLAME | SKY_FRACTURE | BOOK_OF_SKULLS | SPARK
                // W4-c3 — the Shadowflames bolt's `magic = true`
                // (`Projectile.cs:3902`).
                | SHADOWFLAMES
            // W4-d11 — the 536 MedusaHeadRay sets `magic = true`
            // (`Projectile.cs:6117`); the 591 mech laser does NOT
            // (`:6397-6408` has no such line).
                | MEDUSA_HEAD_RAY
                // W4-d12 F1 — the Killing Deck card (1024, `:9699`).
                | DEAD_CELLS_KILLING_DECK_CARD
            // W4-d13 — the friendly Sandnado's `magic = true`
            // (`Projectile.cs:7054`).
                | SANDNADO_FRIENDLY
                // W4-d15 — the Book Staff twister (`Projectile.cs:7453`).
                | DD2_APPRENTICE_STORM
            // W4-d18: the Weather Pain twister's `magic = true`
            // (`Projectile.cs:9175`).
                | WEATHER_PAIN_SHOT
            // W4-d18 tail: the Nightglow bolt's `magic = true`
            // (`Projectile.cs:8835`); the Constellation star sets none.
                | FAIRY_QUEEN_MAGIC_ITEM_SHOT
                // W4-d9 — the ClingerStaff's magic wall (`Projectile.cs:5583`).
                | CLINGER_STAFF
                // W4-d8 — the aiStyle-72 Bubble (410, `Projectile.cs:4926`).
                | BUBBLE
                // W4-d19 — the Soul Drain (`:5516`) and the Resonance
                // Scepter's blast (`:8962`).
                | SOUL_DRAIN | PRINCESS_WEAPON
                // W4-d19 follow-up — the toxic clouds' magic
                // (`Projectile.cs:5813`/`:5825`/`:5837`).
                | 511 | 512 | 513
                // W4-d20 — `Projectile.cs:10424`: the Petalstorm shot (1116)
                // sets magic (the 1038 Flower Whip petal does NOT).
                | PETALSTORM_SHOT
                // W4-d21 — the Lightning Strike shot (1117, `:10434`), the
                // Arc Surge (1122, `:10443`) + the Glacier Fang shot (1115,
                // `:10407`). The 1091 trap sets NO magic flag (`:10175-10185`).
                | LIGHTNING_STRIKE_SHOT | ARC_SURGE | GLACIER_FANG_SHOT
                // W4-g1 lane a — the Crystal Serpent pair's `magic = true`
                // (`Projectile.cs:5914`/`:5925`).
                | CRYSTAL_PULSE | CRYSTAL_PULSE_2
                // W4-g2 (d5 F5/F4/F1): the blood drop 245 (`Projectile.cs:
                // 3284`), the hostile inferno pair 290/291/292 (`:3799`/
                // `:3811`/`:3822`). The 294/239 siblings' magic writes were
                // missed by the earlier census — the W4-g2 reconciliation
                // round closes them: the Shadowbeam 294 (`:3844`) and the
                // rain drop 239 (`:3214`) join the magic census. The 264/433
                // set NO class flag (`:3491-3502`/`:5037-5046`).
                | BLOOD_RAIN | SHADOWBEAM_HOSTILE | INFERNO_HOSTILE_BOLT | INFERNO_HOSTILE_BLAST
                | SHADOWBEAM | RAIN_FRIENDLY
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
/// `ProjectileID.CompanionCube` (`ProjectileID.cs:1860`) — the aiStyle-67
/// pet, W4-d7 F3 (`Projectile.cs:7012-7023`).
pub const COMPANION_CUBE: u16 = 653;
/// `ProjectileID.RainbowBoulderPet` (`ProjectileID.cs:2734`) — the aiStyle-67
/// pet, W4-d7 F3 (`Projectile.cs:10163-10174`).
pub const RAINBOW_BOULDER_PET: u16 = 1090;
/// `ProjectileID.CobWhipSpider` (`ProjectileID.cs:2626`) — the Cob Whip's
/// aiStyle-67 spider, W4-d7 F3 (`Projectile.cs:9767-9779`).
pub const COB_WHIP_SPIDER: u16 = 1036;

// ---- W4-d3: the aiStyle-25 boulder family (census 26b) -------------------
// The 13 kinds the style's `SetDefaults` arms row (`Projectile.cs:1833-1870`,
// `:7038-7047`, `:9918-9937`) plus the RollingCactus's bounce spike (763,
// aiStyle 1, `:7855-7865`). The AI arm, the HandleMovement bounce ladder, the
// RainbowBoulder smart-bounce machine and the Kill hooks live in
// `crate::projectile_ai_boulder`.
/// `ProjectileID.Boulder` (99) — the trap/statue boulder.
pub const BOULDER: u16 = 99;
/// `ProjectileID.RollingCactus` (727) — the bounce spawns 763 spikes.
pub const ROLLING_CACTUS: u16 = 727;
/// `ProjectileID.RollingCactusSpike` (763) — the 727's bounce child
/// (aiStyle 1, extraUpdates 1, `Projectile.cs:7855-7865`).
pub const ROLLING_CACTUS_SPIKE: u16 = 763;
/// `ProjectileID.MiniBoulder` (1005) — the 15x15 getGoodWorld explosion shard.
pub const MINI_BOULDER: u16 = 1005;
/// `ProjectileID.BouncyBoulder` (1013).
pub const BOUNCY_BOULDER: u16 = 1013;
/// `ProjectileID.LifeCrystalBoulder` (1014) — the death drops the Life
/// Crystal item 29; `frame` rides `Main.tileFrame[665]`.
pub const LIFE_CRYSTAL_BOULDER: u16 = 1014;
/// `ProjectileID.MoonBoulder` (1021) — hostile-ONLY (no friendly flag),
/// extraUpdates 1, `timeLeft *= 3`.
pub const MOON_BOULDER: u16 = 1021;
/// `ProjectileID.RainbowBoulder` (1047) — friendly-only, ignoreWater, the
/// smart-bounce treasure/enemy/player jump machine.
pub const RAINBOW_BOULDER: u16 = 1047;
/// `ProjectileID.Poulder` (1048) — the death spreads the poop pyramid (666).
pub const POULDER: u16 = 1048;
/// `ProjectileID.LavaBoulder` (1053) — the death floods lava.
pub const LAVA_BOULDER: u16 = 1053;
/// `ProjectileID.SpiderBoulder` (1054) — timeLeft 300, trails cobwebs (51),
/// the death spawns a WallCreeper (164/165 — unported NPC kinds).
pub const SPIDER_BOULDER: u16 = 1054;
/// `ProjectileID.Ghoulder` (1055) — timeLeft 900, the bounce arms the
/// player-chase flight (`ai[1] == 1`).
pub const GHOULDER: u16 = 1055;
/// `ProjectileID.BoulderThatSpawnsPet` (1057) — the landing kills, the death
/// drops the Boulder Pet item 5523.
pub const BOULDER_THAT_SPAWNS_PET: u16 = 1057;
/// `ProjectileID.BeeHive` (655) — the death spawns bees (210/211) or drops
/// honey (1124) when wet.
pub const BEE_HIVE: u16 = 655;

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
/// `ProjectileID.KingSlimePet` (881, `ProjectileID.cs:2316`) — the King
/// Slime pet, aiStyle 26 (`Projectile.cs:8451-8460`). W4-d20 T4a (the
/// pet-buff shoot arm's spawn kind; the aiStyle-26 slice C owns the AI).
pub const KING_SLIME_PET: u16 = 881;
/// `ProjectileID.QueenSlimePet` (934, `ProjectileID.cs:2422`) — the Queen
/// Slime pet, aiStyle 26 (`Projectile.cs:8874-8883`). W4-d20 T4a.
pub const QUEEN_SLIME_PET: u16 = 934;
/// `ProjectileID.JunimoPet` (994, `ProjectileID.cs:2542`) — the Jumino
/// pet, aiStyle 26 (`Projectile.cs:9436-9445`). W4-d20 T4a (the 5276
/// gift item's pet; the aiStyle-26 slice C owns the AI).
pub const JUNIMO_PET: u16 = 994;
/// `ProjectileID.BerniePet` (956, `ProjectileID.cs:2466`) — Bernie the
/// button-eyed pet, aiStyle 26 (`Projectile.cs:9024-9033`). W4-d20 T4a.
pub const BERNIE_PET: u16 = 956;
pub const STORM_TIGER_1: u16 = 833;
pub const STORM_TIGER_2: u16 = 834;
pub const STORM_TIGER_3: u16 = 835;
pub const ABIGAIL: u16 = 963;
pub const QUEEN_SPIDER: u16 = 377;
pub const SPIDER_EGG: u16 = 378;
/// `ProjectileID.BabySpider` (`ProjectileID.cs:1312`) — the Queen Spider's
/// aiStyle-63 minion shot (`Projectile.cs:4635-4644`), W4-d7 wiring.
pub const BABY_SPIDER: u16 = 379;
/// `ProjectileID.Flairon` (`ProjectileID.cs:1362`) — the Flairon's aiStyle-69
/// recall HEAD (`Projectile.cs:4847-4856`), W4-d7 wiring. NOT [`FLAIRON`]
/// (1058 = FlaironFlail, the aiStyle-15 flail BODY): item 2611 Flairon shoots
/// 404 (`Item.cs:25528`), item 5526 FlaironFlail shoots 1058 (`Item.cs:43534`).
pub const FLAIRON_HEAD: u16 = 404;
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
/// `ProjectileID.MeteorOre` (1078) — the meteor ore chunk, aiStyle 199
/// (`Projectile.cs:10067-10077`). W4-d20.
pub const METEOR_ORE: u16 = 1078;
/// `ProjectileID.FlowerWhipPetal` (1038) - the Flower Whip's petal, aiStyle
/// 196 (`Projectile.cs:9793-9806`). W4-d20 row; the const keeps d17's alias
/// (BATCH-D dedupe, the row-gate now OPEN).
/// `ProjectileID.PetalstormShot` (1116) — the Petalstorm's shot, aiStyle 196
/// (`Projectile.cs:10413-10426`). W4-d20.
pub const PETALSTORM_SHOT: u16 = 1116;
/// `ProjectileID.JuminoStardropAnimation` (995) — the jumino stardrop
/// animation, aiStyle 192 (`Projectile.cs:9446-9457`). W4-d20.
pub const JUMINO_STARDROP_ANIMATION: u16 = 995;
/// `ProjectileID.JimsDrone` (1020) — Jim's RC drone, aiStyle 195
/// (`Projectile.cs:9659-9666`). W4-d20.
/// `ProjectileID.DeadCellsFlint` (1040) — the Dead Cells flint channel
/// weapon, aiStyle 198 (`Projectile.cs:9821-9833`). W4-d20.
pub const DEAD_CELLS_FLINT: u16 = 1040;
/// `ProjectileID.DeadCellsFlintShot` (1041) — the flint's ground spike,
/// aiStyle 157 (`Projectile.cs:9834-9847`). W4-d20.
pub const DEAD_CELLS_FLINT_SHOT: u16 = 1041;
/// `ProjectileID.TrueNightsEdge` (973) — the True Night's Edge spin slash,
/// aiStyle 191 (`Projectile.cs:9315-9329`). W4-d20.
pub const TRUE_NIGHTS_EDGE: u16 = 973;
/// `ProjectileID.TerraBlade2Shot` (985) — the Terra Blade's second shot,
/// aiStyle 191 (`Projectile.cs:9405-9421`). W4-d20.
pub const TERRA_BLADE_2_SHOT: u16 = 985;

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
/// W4-d9 wiring: the 3043 MagicLantern light-pet arm (`:53242-53245`) joins
/// the list — the same `AddBuff(sItem.buffType, 3600)` grant.
pub const MINION_STAFF_PET_BUFF_ITEMS: &[u16] = &[
    1157, 1309, 1802, 2364, 2365, 2535, 2551, 2584, 2621, 2749, 3043, 3249, 3474, 3531, 4269, 4273,
    4281, 4607, 4758, 5005, 5069, 5114, 5456, 5663, 5664, 6148, 6149, 6161, 6164,
];

/// W4-d13 — `ItemCheck_ApplyPetBuffs`' PET-item rows: the same use-start
/// `AddBuff(sItem.buffType, 3600)` grant, for the pet/light-pet items whose
/// `Item.buffTime` is 0 (the `:42880` grant is a ghost for them too — the
/// 3600 grant is what the `UpdateBuffs` keep-alive arm's 18000 refresh
/// pins). Verbatim: the 3577 Suspicious Looking Tentacle single-item arm
/// (`Player.cs:53117-53120`) + the 54-case vanity-pet switch
/// (`:53286-53341`). The six aiStyle-124 pet items are 4798 (EOC), 4804
/// (Twins), 4811 (Fairy Queen), 4812 (Pumpking), 4814 (Ice Queen), 5089
/// (Glommer — `Item.cs:40791-40795` `DefaultToVanitypet(957, 328)`; NOT
/// 5117, the Xenopopper gun `:40997-41013`).
/// RETIRE census (research/336 §3): the function's remaining single-item
/// arms — the CLASSIC pets/light pets (`:53097-53116`, `:53146-53189`,
/// `:53198-53233`, `:53242-53249`) — carry the SAME buffTime=0 ghost and
/// their keep-alives are ALL ported (`buff_effects_misc.rs`'s pet table +
/// `buff_effects_accessory.rs`), so they join the grant verbatim. TWO
/// exclusions: 603 (the Carrot — the `:53093` arm gates on
/// `Main.runningCollectorsEdition`, a flag the port does not carry; the
/// non-CE default false keeps the arm dead) and 425 (the Fairy Bell —
/// the `:53121-53145` `Next(3)` 27/101/102 buff pick is d2's light-pets
/// lane, a different arm shape). The g10 town pets do NOT consume this
/// table: the licenses 4829/4830/4910 ride `ItemCheck_UsePetLicenses`
/// (`Player.cs:45352-45366` → `NPC.UnlockOrExchangePet` `NPC.cs:80894-
/// 80918`), no buff, no projectile. The d43 pal reward staves 5663/5664/
/// 6148/6149 are the i25 `MINION_STAFF_PET_BUFF_ITEMS` rows; 5665/5666
/// are Chillet MOUNTS (`Item.cs:44509-44532`, mountType 62/63), no buff.
pub const PET_ITEM_BUFF_GRANTS: &[u16] = &[
    3577, // :53117 — the Suspicious Looking Tentacle (buff 190 → 650)
    // :53286-53341 — the vanity-pet switch, verbatim order.
    3855, 3856, 3857, 4365, 4366, 4425, 4550, 4551, 4603, 4604, 4605, 4701, 4735, 4736, 4737,
    4777, 4797, 4798, 4799, 4800, 4801, 4802, 4803, 4804, 4805, 4806, 4807, 4808, 4809, 4810,
    4811, 4812, 4813, 4814, 4815, 4816, 4817, 4960, 5088, 5089, 5090, 5091, 5098, 5131, 5276,
    5297, 5332, 5333, 5400, 5466, 5513, 5517, 5523, 5654,
    // The classic single-item arms, verbatim C# order (the retire census).
    669,  // :53097 — Fish (buff 41 → the 112 Penguin)
    115,  // :53101 — Shadow Orb (buff 19 → 18, light pet)
    3060, // :53105 — Bone Rattle (buff 154 → the 499 Baby Face Monster)
    3628, // :53109 — Companion Cube (buff 191 → 653)
    3062, // :53113 — Crimson Heart (buff 155 → 500, light pet)
    753,  // :53146 — Seaweed (buff 42 → the 127 Turtle)
    994,  // :53150 — Eater's Bone (buff 45 → the 175 Baby Eater)
    1169, 1170, 1171, 1172, // :53154-53169 — Bone Key/Nectar/Tiki Totem/Lizard Egg (50-53)
    1180, 1181, 1182, 1183, // :53170-53185 — Parrot Cracker/Mushroom/Seedling/Wisp (54-57)
    1242, // :53186 — Amber Mosquito (buff 61 → the 236 Dino)
    1311, // :53198 — Eye Spring (buff 65 → 268)
    1837, // :53202 — Cursed Sapling (buff 85 → 324)
    1312, // :53206 — Toy Sled (buff 66 → the 269 Snowman)
    1798, 1799, // :53210/:53214 — Spider Egg/Magical Pumpkin Seed (81/82)
    1810, // :53222 — Unlucky Yarn (buff 84 → the 319 Black Cat)
    1927, 1959, // :53226/:53230 — Dog Whistle/Grinch Whistle (91/92)
    3043, // :53242 — Magic Lantern (buff 152 → 492, light pet)
    2420, // :53246 — Zephyr Fish (buff 127 → 380)
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

/// `Main.projHostile` (`Main.cs:6612-6619`): the init loop stamps every kind
/// whose `SetDefaults(i).hostile` is true (`Projectile.cs:697-10673`, the
/// `hostile = false` reset `:764`). The loop runs in
/// `Initialize_AlmostEverything` BEFORE any world load, with the seed flags
/// still false (`Main.cs:66532-66536`'s static-ctor defaults), so the
/// seed-gated writes — 12's `Main.remixWorld` (`Projectile.cs:929-932`), 43 /
/// 201-205 / 527-531's `Main.getGoodWorld` (`:1265-1269`, `:2879-2936`,
/// `:5978-5982`) — never land in the table. The 173 kinds below are the
/// unconditional `this.hostile = true` arms, verbatim (S1-I-1 W4-d93: the
/// `27` host read gate, `MessageBuffer.cs:1725-1728`).
pub const PROJ_HOSTILE: &[u16] = &[
    31, 38, 39, 40, 44, 55, 56, 67, 71, 75, 81, 82,
    83, 84, 96, 98, 99, 100, 101, 102, 108, 109, 110, 115,
    128, 129, 164, 174, 176, 177, 179, 180, 184, 185, 186, 188,
    240, 241, 257, 258, 259, 264, 270, 275, 276, 277, 288, 290,
    291, 292, 293, 299, 300, 302, 303, 325, 326, 327, 328, 329,
    345, 346, 347, 348, 349, 350, 351, 352, 384, 385, 386, 435,
    436, 437, 438, 447, 448, 449, 450, 452, 454, 455, 456, 462,
    464, 465, 466, 467, 468, 471, 472, 490, 498, 501, 508, 537,
    538, 539, 540, 572, 573, 574, 575, 576, 577, 578, 579, 580,
    581, 592, 593, 596, 605, 629, 654, 655, 657, 658, 662, 670,
    671, 672, 673, 674, 675, 676, 681, 682, 683, 685, 686, 687,
    713, 719, 727, 763, 811, 812, 813, 814, 836, 871, 872, 873,
    874, 909, 919, 920, 921, 922, 923, 926, 961, 962, 965, 980,
    1001, 1002, 1005, 1007, 1013, 1014, 1021, 1048, 1049, 1053, 1054, 1055,
    1057, 1073, 1078, 1091, 1092,
];

/// `Main.projHostile[type]` — the `27` host gate's hostile-kind test
/// (`MessageBuffer.cs:1725`). An unlisted kind reads false, exactly like the
/// C# array's untouched entries.
pub fn proj_hostile(kind: u16) -> bool {
    PROJ_HOSTILE.contains(&kind)
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
        // W4-c11 — `Main.cs:8532`: the Martian Turret bolt's 4-frame sheet.
        435 => 4,
        436 => 4,
        // W4-f20 F3 — the Electrosphere sphere 443 + the Sky Dragon's Fury
        // circle 1110 share the 4-frame sheet (`Main.cs:8536`/`:8559`); the
        // aiStyle-77 pulse's 3-tick frame machine wraps at 4
        // (`Projectile.cs:30724-30733`).
        443 | SKY_DRAGONS_FURY_ELECTROSPHERE => 4,
        447 => 4,
        448 => 3,
        450 => 5,
        454 => 2,
        456 => 4,
        462 => 5,
        // W4-f20 slice D — the Hellwing bat's 5-frame sheet
        // (`Main.cs:8482`); the AI_001 rotation tail's 2-tick frame
        // machine wraps at 5 (`Projectile.cs:57716-57725`).
        HELLWING => 5,
        519 => 4,
        // W4-d10 — the aiStyle-92/94/97/100 sheets: the Coin Portal's
        // 4-frame spin (`Main.cs:8486`), the Flying Piggy Bank's 5-frame
        // flap (`:8479`), the Medusa Head's 12-frame charge (`:8548`), the
        // spore cloud's 5-frame first-tick roll (`:8726`).
        518 => 4,
        525 => 5,
        535 => 12,
        1128 => 5,
        574 => 2,
        601 => 2,
        645 => 7,
        652 => 6,
        659 => 4,
        // W4-g64 — the style-28 trail kinds' sheets (`Main.cs:8705`/`:8716`):
        // the 967's 6-tick frame cycle wraps at 8 (`Projectile.cs:26596`),
        // the 1026's 4-tick cycle at 4 (`:26575`).
        967 => 8,
        1026 => 4,
        // W4-g56 — the Void Lens's 8-frame sheet (`Main.cs:8504`) —
        // AI_154's 5-tick frame machine wraps at it (`Projectile.cs:52179-
        // 52186`), `VoidLensHelper.DrawToDrawData` slices `Frame(1, 8, ..)`.
        VOID_LENS => 8,
        837 => 3,
        1130 => 6,
        // W4-d15 — the DD2 pets' sheets (`Main.cs:8626-8628`): the gato 8
        // (the arm overrides the wrap to 4), the dragon 3, the ghost 4;
        // the 1050's 16 (`Main.cs:8719`) — the tree-shake frames 8/12
        // (`Projectile.cs:70215`) index it. The 764 has no override (the
        // init loop's 1).
        DD2_PET_GATO => 8,
        DD2_PET_DRAGON => 3,
        DD2_PET_GHOST => 4,
        AXE_FAIRY_PET => 16,
        // W4-d15 — the T2 ghast's 4-frame sheet (`Main.cs:8473`) — the
        // 5-tick frame machine (`Projectile.cs:70003-70010`) wraps at it.
        MONK_STAFF_T2_GHAST => 4,
        // W4-d15 — the T3-alt volley shot's 3-frame sheet (`Main.cs:8554`).
        MONK_STAFF_T3_ALT_SHOT => 3,
        // W4-d21 — the aiStyles 201-210 sheets: the Glacier Fang shot's
        // 3-frame charge ladder (`Main.cs:8563`, the AI's ai[0]>=20/40
        // frame writes `Projectile.cs:37010-37017`), the two digtoises'
        // 11-frame appear/dig sheets (`Main.cs:8723-8724`) and the RC car's
        // 7-frame drive/fly sheet (`Main.cs:8725`).
        1115 => 3,
        1098 | 1123 => 11,
        1105 => 7,
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
        // W4-a9 — the Sanguine bat's 5-frame sheet (`Main.cs:8505`) —
        // `AI_156_BatOfLight`'s 6-tick flap wraps at projFrames − 1
        // (`Projectile.cs:51535-51542`). The 946 blade keeps the stock 1
        // (no override; its arm runs no frame machine).
        755 => 5,
        // W4-a9 — the Storm Tiger gem / Abigail counter's 6-frame sheets
        // (`Main.cs:8648-8649`) — `AI_164_StormTigerGem`'s 4-tick pulse
        // wraps at the literal 6 (`Projectile.cs:46607-46614`/
        // `:46631-46638`).
        831 | 970 => 6,
        // W4-a9 — the Smolstar's 2-frame sheet (`Main.cs:8665`) — the
        // aiStyle-169 arm runs no frame machine (the stock frame 0 reads
        // the top row); the row lands for the draw slice.
        864 => 2,
        // W4-a9 — the Forbidden Minion's 9-frame sheet (`Main.cs:8561`) —
        // `AI_206_ForbiddenMinion`'s idle machine steps frames 0-4 and the
        // attack machine 5-8 (`Projectile.cs:37605-37640`).
        1119 => 9,
        // AI_026 slice B: the boss-head pets' sheets (`Main.cs:8673`
        // 885 → 10, `:8677` 889 → 11) — the spin arm pins frame 6
        // (`Projectile.cs:60192`) and the walk steps frames 0-5
        // (`:60240-60247`).
        SKELETRON_PET => 10,
        SKELETRON_PRIME_PET => 11,
        // W4-d18: the multisegment pets' sheets (`Main.cs:8671` 883 → 3,
        // `:8675` 887 → 3, `:8681` 893 → 4). `AI_174` writes no `frame`
        // (the worm segments draw from the trail, not the sheet rows).
        EATER_OF_WORLDS_PET | DESTROYER_PET => 3,
        LUNATIC_CULTIST_PET => 4,
        // W4-d18: the titanium storm shard's 12-frame sheet
        // (`Main.cs:8690`) — the first-tick `rand.Next(12)` frame roll
        // (`Projectile.cs:45330`) picks one row.
        TITANIUM_STORM_SHARD => 12,
        // W4-d18: the scythe whip bolt's 6-frame sheet (`Main.cs:8453`)
        // — the 3-tick frame machine's `rand.Next(projFrames)` roll
        // (`Projectile.cs:45088`) picks one row.
        SCYTHE_WHIP_PROJ => 6,
        // W4-d18: the Weather Pain twister's 8-frame sheet
        // (`Main.cs:8707`) — the 5-tick frame machine wraps at it
        // (`Projectile.cs:45233`). The 917 snowflake has no override
        // (the init loop's 1 — its arm never reads `frame`).
        WEATHER_PAIN_SHOT => 8,
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
        // W4-d3 — the boulder family's sheets (`Main.cs:8713-8714`): the
        // 1014 LifeCrystalBoulder's 11 frames (its `frame` rides
        // `Main.tileFrame[665]`, `Projectile.cs:26028`) and the 1055
        // Ghoulder's 10 (the chase frame machine, `:26157-26236`). The
        // 1047 RainbowBoulder's 100-frame hue sheet is unlisted in
        // `Main.projFrames` (the stock 1 — the frame is a draw-layer hue
        // index, not a sheet row).
        LIFE_CRYSTAL_BOULDER => 11,
        GHOULDER => 10,
        // W4-d4 — the Mechanical Piranha's 4-frame sheet (`Main.cs:8592`)
        // — the aiStyle-39 arm's 4-tick cycle wraps at 4 (`:27765-27774`).
        MECHANICAL_PIRANHA => 4,
        // The Stardust Guardian's 19-frame sheet (`Main.cs:8497`) —
        // AI_120's idle machine wraps at 8 (`Projectile.cs:44693`), the
        // rest-hover at projFrames − 4 = 15 (`:44830`) and the dash at
        // 19 → 14 (`:44852-44855`). S1-P-i56.
        STARDUST_GUARDIAN => 19,
        // The Crimson Heart's 4-frame sheet (`Main.cs:8484`) — the pet's
        // 6/12-tick pulse wraps at it (`Projectile.cs:49473`). The 1018/1056
        // keep the init default 1 (no override in `Main.cs:8444-8727`).
        // W4-d20 — the meteor ore's 3-frame sheet (`Main.cs:8443`) — the
        // first-tick `Next(3)` frame roll picks it (`Projectile.cs:40024`).
        METEOR_ORE => 3,
        // W4-d20 — the two aiStyle-196 petals' 8-frame sheets
        // (`Main.cs:8555-8556`) — the 4-tick cycle wraps at 8
        // (`Projectile.cs:40492-40499`).
        FLOWER_WHIP_PETAL | PETALSTORM_SHOT => 8,
        // W4-d20 — the jumino animation's 20-frame sheet (`Main.cs:8709`) —
        // the fall machine wraps at 20 (`Projectile.cs:41547-41550`).
        JUMINO_STARDROP_ANIMATION => 20,
        CRIMSON_HEART => 4,
        // W4-d7 wiring — the BabySpider's 4-frame walk sheet (`Main.cs:8622`);
        // the aiStyle-63 arm's frame machine (`Projectile.cs:29381-29407`)
        // indexes it. The 404 Flairon head keeps the stock 1 (no override).
        BABY_SPIDER => 4,
        // W4-d7 F3 — the Cob Whip spider's 4-frame sheet (`Main.cs:8511`);
        // the aiStyle-67 walk machine (`Projectile.cs:50768-50797`) indexes
        // it. The Storm Tigers' sheets (`Main.cs:8650-8652`): the 833 walks
        // 0-7 / flies 8, the 834/835 walk 0-9 / fly 10-11. The 653/1090 keep
        // the stock 1 (no `projFrames` override in `Main.cs:8444-8727`).
        COB_WHIP_SPIDER => 4,
        STORM_TIGER_1 => 10,
        STORM_TIGER_2 | STORM_TIGER_3 => 12,
        // W4-g64 (the d13 314fa6cd row port) — the rowed aiStyle-124 pets'
        // sheets (`Main.cs:8683` 895 → 6, `:8686` 898 → 16, `:8697` 957 →
        // 12). The flight/idle frame machines (`Projectile.cs:35204-35565`)
        // wrap at these; the 957's ground bounce reads `projFrames * 2` /
        // `* 3`. The 650/882/888/896 sheets arrive with d13's rows.
        FAIRY_QUEEN_PET => 6,
        ICE_QUEEN_PET => 16,
        GLOMMER_PET => 12,
        // W4-d13 — the aiStyle-124 pets' sheets (`Main.cs:8500` 650 → 4,
        // `:8670` 882 → 20, `:8676` 888 → 36, `:8683-8684` 895 → 6 /
        // 896 → 16, `:8686` 898 → 16, `:8697` 957 → 12). The flight/idle
        // frame machines (`Projectile.cs:35124-35565`) wrap at these; the
        // 957's ground bounce reads `projFrames * 2` / `* 3`.
        SUSPICIOUS_TENTACLE => 4,
        EOC_PET => 20,
        TWINS_PET => 36,
        FAIRY_QUEEN_PET => 6,
        PUMPKING_PET | ICE_QUEEN_PET => 16,
        GLOMMER_PET => 12,
        // W4-d16 — the Void Lens' 8-frame sheet (`Main.cs:8504`) —
        // `AI_154`'s 5-tick cycle wraps at it (`Projectile.cs:52179-52186`).
        VOID_LENS => 8,
        // W4-d16 — the kite sheets (`Main.cs:8632-8636`/`:8643`/`:8645-
        // :8647`/`:8655-8656`/`:8658`): the 766-770 five 4-frame kites,
        // the 824/839/840/853 wind-flap fours, the 826 pigron's 3, the
        // 828/829 jellyfish pair's 2. The rest keep the stock 1 (no
        // override in `Main.cs:8444-8727`). `KiteLogic`'s frame machine
        // (`Projectile.cs:48627-48691`) reads them.
        766..=770 => 4,
        824 | 839 | 840 | 853 => 4,
        826 => 3,
        828 | 829 => 2,

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
        // W4-d12 — the Dandelion seed's 4-frame sheet (`Main.cs:8653`) —
        // the aiStyle-112 836 arm's 6-tick machine wraps at 4
        // (`Projectile.cs:34128-34135`). The 590 spore has NO `projFrames`
        // row (the init 1, `Main.cs:8441`); its arm's frame machine walks
        // the literal 0-2 / 3-5 bands (`:34196-34210`), not `projFrames`.
        DANDELION_SEED => 4,
        // W4-d12 — the Portal gate's 4-frame sheet (`Main.cs:8625`) — the
        // aiStyle-114 arm's 6-tick machine wraps at `projFrames`
        // (`Projectile.cs:34404-34411`).
        PORTAL_GUN_GATE => 4,
        // W4-d12 — the aiStyle-117 explosions' 5-frame sheets
        // (`Main.cs:8492` 612, `:8509` 953) — the shared arm's 3-tick
        // machine wraps at `projFrames` (`Projectile.cs:34472-34479`) and
        // the `3 * projFrames` life cap reads it (`:34467`). The 608/624/
        // 1044 keep the init 1 (no override — `Main.cs:8441`).
        SOLAR_WHIP_SWORD_EXPLOSION | DAYBREAK_EXPLOSION => 5,
        // W4-d12 DOC-SWEEP item 7 — the Volcano explosion's 5-frame sheet
        // (`Main.cs:8493`); the aiStyle-189 arm's 2-tick machine wraps at 5
        // (`Projectile.cs:42282-42289`).
        VOLCANO => 5,
        // W4-d12 F1 — the aiStyle-113 family's sheets: the Stardust Cell's
        // shot is 4 frames (`Main.cs:8495`), the Killing Deck card 8
        // (`Main.cs:8444`). The 598/636/971/975 keep the init 1 (no
        // override — `Main.cs:8441`); the card's frame machine wraps at
        // `projFrames` (`Projectile.cs:40579-40589`).
        STARDUST_CELL_MINION_SHOT => 4,
        DEAD_CELLS_KILLING_DECK_CARD => 8,
        // S1-D-3 W-9 fishron wiring — the sharknado family's strips
        // (`Main.cs:8512-8514`): the clouds 6 rows, the bolt 3.
        FORCE_BUBBLE | CTHULUNADO => 6,
        SHARKNADO_BOLT => 3,
        // The glow-stick's 4-frame sheet (`Main.cs:8664`) — the aiStyle-2
        // arm's 5-tick frame machine (`Projectile.cs:23477-23486`) wraps at
        // it, back to frame 1.
        861 => 4,
        // W4-d17 (canonical, research/114 §1c) — the Release Doves (851) /
        // Release Lantern (855) 4-frame sheets (`Main.cs:8657`/`:8660`):
        // `AI_166_Dove`'s frame machine wraps at 4 (`Projectile.cs:47556`).
        // c39's rows omitted the sheet count (the draw would use 1 frame).
        RELEASE_DOVES | RELEASE_LANTERN => 4,
        // W4-d19 follow-up — the spore cloud's 5-frame sheet
        // (`Main.cs:8726`): the aiStyle-92 first-tick roll
        // (`Projectile.cs:32565`) picks the row.
        1128 => 5,
        // The 909's 6-frame sheet (`Main.cs:8691`) — the arm's
        // `1 + rand.Next(6)` frame roll (`Projectile.cs:23268-23274`) pins
        // one of its rows.
        909 => 6,
        // The 968's 24-frame sheet (`Main.cs:8706`) — the arm's
        // `frame = (int)ai[1]` pin (`Projectile.cs:23238`) reads it.
        968 => 24,
        // W4-d78 batch 2 — the Kill child-spawn family's sheets: the Stynger
        // shrapnel's 5 rows (`Main.cs:8610`; the aiStyle-2 arm has no frame
        // writer for it, the slice reads frame 0) and the Bone Shard's 2 rows
        // (`Main.cs:8562`; the AI_001 first-tick `frame = Next(2)` roll,
        // `Projectile.cs:54016-54020`). The Stynger bolt 246 and the Arcanum
        // shard 620 keep the stock 1 (no `projFrames` override).
        STYNGER_SHRAPNEL => 5,
        BONE_SHARD => 2,
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
        // W4-g2 (d5 F6/F5) — the Partisan orb's 4-frame sheet
        // (`Main.cs:8606`, the aiStyle-44 frame machine wraps at it) and the
        // Blood Rain clouds' sheets (`Main.cs:8608-8609`: the moving cloud 4,
        // the raining cloud 6 — the 237/238 twins). The drops 245/264 and the
        // beams/bolts/blasts 290-292/433 have no override — the stock 1.
        CHLOROPHYTE_ORB => 4,
        BLOOD_CLOUD_MOVING => 4,
        BLOOD_CLOUD_RAINING => 6,
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
        // W4-g5 — the 1092 book's 54-frame sheet (`Main.cs:8720`): the 6×9
        // grid (`Main.cs:33197`'s `Frame(6, 9, frame/9, frame%9)`). The
        // 129 RuneBlast keeps the stock 1 (no `projFrames` override).
        LIBRARIAN_BOOK => 54,
        // W4-d2 — the fairies' 4-frame sheets (`Main.cs:8578-8580`); the
        // aiStyle-11 arm's `frameCounter >= 4` / `frame >= 4` cycle
        // (`Projectile.cs:24385-24394`). The Shadow Orb keeps the stock 1
        // (no `projFrames[18]` override — its arm spins, never frames).
        BLUE_FAIRY | PINK_FAIRY | GREEN_FAIRY => 4,
        // W4-g64 (the d11 row port) — the 596 curse flame's 4-frame sheet
        // (`Main.cs:8491`) — the aiStyle-107 5-tick cycle wraps at it
        // (`Projectile.cs:33598-33605`). The 575 sphere's sheet stays
        // d11's (`Main.cs:8550`).
        DESERT_DJINN_CURSE => 4,
        // W4-d11 — the tower orbs' sheets (`Main.cs:8549-8551`): the 539
        // jellyfish + the 575 sphere 4 rows, the 574 eye 2 rows (the
        // aiStyle-102 machine's frame wraps + the 574's `frame = 1` pin,
        // `Projectile.cs:33310-33317`/`:33352`/`:33410-33417`).
        STARDUST_JELLYFISH_SMALL => 4,
        NEBULA_EYE => 2,
        // W4-d11 — the 575 sphere + the 596 curse flame's 4-frame sheets
        // (`Main.cs:8550`/`:8491`) — the aiStyle-107 5-tick cycles wrap
        // at them (`Projectile.cs:33572-33579`/`:33598-33605`).
        NEBULA_SPHERE => 4,
        DESERT_DJINN_CURSE => 4,
        // W4-d11 — the 565's 4-frame sheet (`Main.cs:8478`) — the
        // aiStyle-104 7-tick cycle wraps at it (`Projectile.cs:33499-
        // 33508`).
        BRAIN_OF_CONFUSION => 4,
        // W4-d17 — the chum bait's 4-frame sheet (`Main.cs:8452`) — the
        // first-tick `frame = Next(4)` roll (`Projectile.cs:53553`) picks
        // the row; no frame machine steps it after.
        CHUM_BUCKET => 4,
        // W4-d17 — the dove/lantern pair's 4-frame sheets (`Main.cs:8657`/
        // `:8660`) — the aiStyle-166 frame machine (`Projectile.cs:47553-
        // 47560`) wraps at 3 (the dove 4-tick, the lantern 6-tick stride).
        RELEASE_DOVES | RELEASE_LANTERN => 4,
        // W4-d19 — the Zoologist strikes' 8-frame sheets
        // (`Main.cs:8694-8695`) — `AI_183_ZoologistStrike`'s per-tick
        // frame advance clamps at 7 (`Projectile.cs:43104-43108`).
        ZOOLOGIST_STRIKE_GREEN | ZOOLOGIST_STRIKE_RED => 8,
        // W4-d80 — the Phantom Phoenix's 8-frame sheet (`Main.cs:8456`) —
        // `frame = frameCounter / 5` (`Projectile.cs:55841`); the Sky
        // Dragon's Fury shot's 3-frame sheet (`Main.cs:8554`) — the 3-tick
        // frame machine (`:54704-54712`) wraps at 3.
        PHANTOM_PHOENIX => 8,
        SKY_DRAGONS_FURY_SHOT => 3,
        // W4-d80 — the RC car's 7-frame drive/fly sheet (`Main.cs:8725`) —
        // the aiStyle-205 frame machines wrap at 4 (ground) / 7 (fly).
        REMOTE_CONTROL_CAR => 7,
        _ => 1,
    }
}

/// `Main.projPet` (`Main.cs:8728-8852`) — the pet/light-pet projectile set.
/// The `SendData(27)` relay's flag3 section-cull exemption reads it
/// (`NetMessage.cs:1777`); the whole 125-row census lands verbatim (the
/// array is zero-initialised and only these writes set it true — the other
/// `Main.projPet[` reads in Player/Projectile/MessageBuffer are consumers).
pub const fn proj_pet(kind: u16) -> bool {
    matches!(
        kind,
        111 | 112
            | 127
            | 175
            | 191
            | 192
            | 193
            | 194
            | 197
            | 198
            | 199
            | 200
            | 208
            | 209
            | 210
            | 211
            | 236
            | 266
            | 268
            | 269
            | 313
            | 314
            | 317
            | 319
            | 324
            | 334
            | 353
            | 373
            | 375
            | 380
            | 387
            | 388
            | 390
            | 391
            | 392
            | 393
            | 394
            | 395
            | 398
            | 407
            | 423
            | 492
            | 499
            | 533
            | 613
            | 623
            | 625
            | 626
            | 627
            | 628
            | 653
            | 701
            | 702
            | 703
            | 755
            | 758
            | 759
            | 764
            | 765
            | 774
            | 815
            | 816
            | 817
            | 821
            | 825
            | 831
            | 833
            | 834
            | 835
            | 854
            | 858
            | 859
            | 860
            | 864
            | 875
            | 881
            | 882
            | 883
            | 884
            | 885
            | 886
            | 887
            | 888
            | 889
            | 890
            | 891
            | 892
            | 893
            | 894
            | 895
            | 896
            | 897
            | 898
            | 899
            | 900
            | 901
            | 934
            | 946
            | 951
            | 956
            | 957
            | 958
            | 959
            | 960
            | 963
            | 970
            | 994
            | 998
            | 1003
            | 1004
            | 1018
            | 1022
            | 1027
            | 1046
            | 1050
            | 1056
            | 1090
            | 1093
            | 1094
            | 1095
            | 1096
            | 1112
            | 1113
            | 1118
            | 1119
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S1-I-1 W4-d93: the `Main.projHostile` census pin — the 173 kinds are
    /// the unconditional `this.hostile = true` arms of `SetDefaults`
    /// (`Projectile.cs:697-10673`) plus the 326..328 range arm (`:4193-4212`);
    /// the seed-gated writes (12 remix `:929-932`, 43 / 201-205 / 527-531
    /// getGoodWorld `:1265-1269`, `:2872-2936`, `:5978-5982`) never land in
    /// the table because the init loop (`Main.cs:6612-6619`) runs before any
    /// world load with the seed flags false (`Main.cs:66531-66536`).
    #[test]
    fn the_proj_hostile_table_matches_the_set_defaults_census() {
        assert_eq!(PROJ_HOSTILE.len(), 173);
        let mut sorted = PROJ_HOSTILE.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 173, "no duplicate kinds");
        assert_eq!(sorted, PROJ_HOSTILE, "the table is ascending");
        // The seed-gated kinds read false — the table's whole point.
        for k in [12, 43, 201, 202, 203, 204, 205, 527, 528, 529, 530, 531] {
            assert!(
                !proj_hostile(k),
                "seed-gated kind {k} is not hostile at init"
            );
        }
        // The range arm's three kinds all land (`:4193` covers 326..=328).
        for k in [326, 327, 328] {
            assert!(proj_hostile(k), "range-arm kind {k}");
        }
        // The multi-kind arms: the boulder trap family (`:1833`) and the
        // cannonball pair (`:7925`).
        for k in [99, 727, 1013, 1014, 1053, 1054, 1055, 1057, 811, 814] {
            assert!(proj_hostile(k), "multi-kind arm {k}");
        }
        // Friendly staples read false (the `hostile = false` reset `:764`).
        for k in [0, 1, 2, 14, 602, 651, 1006] {
            assert!(!proj_hostile(k), "friendly kind {k}");
        }
        // An out-of-range kind reads false, like the untouched array tail.
        assert!(!proj_hostile(u16::MAX));
    }

    /// W4-g52 — the `Main.projPet` census (`Main.cs:8728-8852`): all 125
    /// `= true` write sites, and nothing else. The boundary kinds pin the
    /// gaps (12 is flag3 by its own type test, NOT a pet; 72/18/500/650 are
    /// the `Player.cs:48561` companion special-cases, not `projPet` rows).
    #[test]
    fn w4_g52_proj_pet_matches_the_main_census() {
        const ROWS: &[u16] = &[
            111, 112, 127, 175, 191, 192, 193, 194, 197, 198, 199, 200, 208, 209, 210, 211, 236,
            266, 268, 269, 313, 314, 317, 319, 324, 334, 353, 373, 375, 380, 387, 388, 390, 391,
            392, 393, 394, 395, 398, 407, 423, 492, 499, 533, 613, 623, 625, 626, 627, 628, 653,
            701, 702, 703, 755, 758, 759, 764, 765, 774, 815, 816, 817, 821, 825, 831, 833, 834,
            835, 854, 858, 859, 860, 864, 875, 881, 882, 883, 884, 885, 886, 887, 888, 889, 890,
            891, 892, 893, 894, 895, 896, 897, 898, 899, 900, 901, 934, 946, 951, 956, 957, 958,
            959, 960, 963, 970, 994, 998, 1003, 1004, 1018, 1022, 1027, 1046, 1050, 1056, 1090,
            1093, 1094, 1095, 1096, 1112, 1113, 1118, 1119,
        ];
        assert_eq!(ROWS.len(), 125, "the census grep count (Main.cs)");
        for k in 0..=1200u16 {
            assert_eq!(
                proj_pet(k),
                ROWS.contains(&k),
                "projPet[{k}] vs Main.cs:8728-8852"
            );
        }
        assert!(!proj_pet(12), "type 12 is flag3 by its own arm, not projPet");
        assert!(!proj_pet(72) && !proj_pet(18) && !proj_pet(500) && !proj_pet(650));
    }

    #[test]
    fn w4_g35_the_glow_mask_slice_matches_the_set_defaults_census() {
        // The 10 C# assignment sites (Projectile.cs:4055, :4988-5035,
        // :9432, :10019/:10035) + the −1 reset stock (:780).
        assert_eq!(set_defaults_glow_mask(312), 257); // JackOLantern
        assert_eq!(set_defaults_glow_mask(CHAINSAW_VORTEX), 2);
        assert_eq!(set_defaults_glow_mask(DRILL_VORTEX), 3);
        assert_eq!(set_defaults_glow_mask(CHAINSAW_NEBULA), 7);
        assert_eq!(set_defaults_glow_mask(DRILL_NEBULA), 8);
        assert_eq!(set_defaults_glow_mask(CHAINSAW_STARDUST), 179);
        assert_eq!(set_defaults_glow_mask(DRILL_STARDUST), 180);
        // The solar pair sets NO glowMask (:5011-5022) — stock −1.
        assert_eq!(set_defaults_glow_mask(CHAINSAW_SOLAR), -1);
        assert_eq!(set_defaults_glow_mask(DRILL_SOLAR), -1);
        // The bobber ladder 318+(type−987); the 986 plain keeps −1.
        assert_eq!(set_defaults_glow_mask(986), -1);
        assert_eq!(set_defaults_glow_mask(987), 318);
        assert_eq!(set_defaults_glow_mask(988), 319);
        assert_eq!(set_defaults_glow_mask(989), 320);
        assert_eq!(set_defaults_glow_mask(990), 321);
        assert_eq!(set_defaults_glow_mask(991), 322);
        assert_eq!(set_defaults_glow_mask(992), 323);
        assert_eq!(set_defaults_glow_mask(993), 324);
        // The Rainbow phaseblade/phasersaber (the :10006/:10022 shared
        // boomerang arms' inner type gates).
        assert_eq!(set_defaults_glow_mask(1101), 372);
        assert_eq!(set_defaults_glow_mask(1102), 373);
        // Their non-glowing arm-mates keep −1.
        assert_eq!(set_defaults_glow_mask(1059), -1);
        assert_eq!(set_defaults_glow_mask(1065), -1);
        // The unlisted kinds stay dark.
        assert_eq!(set_defaults_glow_mask(WOODEN_ARROW), -1);
        assert_eq!(set_defaults_glow_mask(GLOWSTICK), -1);
    }

    #[test]
    fn w4_g35_the_spawn_copies_the_glow_mask() {
        // The data-carrying spawn writes the slice onto the slot, like the
        // light slice (test_s1_e_5_the_spawn_copies_the_set_defaults_light).
        for k in [CHAINSAW_VORTEX, DRILL_VORTEX, CHAINSAW_NEBULA, DRILL_NEBULA,
                  CHAINSAW_STARDUST, DRILL_STARDUST] {
            let d = proj_data(k).expect("the drill/chainsaw rows exist");
            assert_eq!(d.glow_mask, set_defaults_glow_mask(k), "kind {k}");
        }
        // A row-less kind keeps the −1 stock through Default.
        assert_eq!(crate::projectile::Projectile::default().glow_mask, -1);
    }

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
        // W4-d78 batch 2 — the Kill child-spawn family's ranged kinds
        // (246/249 `:3295`/`:3320`, 1124 `:10523`).
        for k in [STYNGER, STYNGER_SHRAPNEL, BONE_SHARD] {
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
        // W4-d18 — the seven solution sprays set none either
        // (`:2350-2397`/`DefaultToSpray` `:10691-10702`).
        // W4-d1 retire (the spray compose) — the seven solution sprays set
        // none either (`:2350-2397`/`DefaultToSpray` `:10691-10702`).
        for k in (146..=149).chain(1015..=1017) {
            assert_eq!(flags(k), (false, false, false), "kind {k}");
        }
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
        // W4-d78 batch 2 — the Arcanum's death shard (620, `:6687`).
        assert_eq!(flags(NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD), (false, false, true));
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

    /// W4-d18 — the seven solution-spray rows (`Projectile.cs:2350-2397`
    /// the four 146-149 blocks, `:9633-9636` → `DefaultToSpray`
    /// `:10691-10702` for 1015-1017): every block is the 145 shape
    /// verbatim — 6x6, aiStyle 31, penetrate −1, the stock 3600 life (the
    /// arm caps it at 133), extraUpdates 2, tileCollide false, alpha 255,
    /// ignoreWater true, NO class flag, NO immunity pair. WRITTEN NOT RUN
    /// (code-only mode).
    #[test]
    fn w4_d18_the_solution_spray_rows_are_the_145_shape() {
    /// W4-d1 retire (the spray compose — the d18/f25 DUP keep-one): the
    /// seven solution-spray rows (`Projectile.cs:2350-2397` the four
    /// 146-149 blocks, `:9633-9636` → `DefaultToSpray` `:10691-10702` for
    /// 1015-1017): every block is the 145 shape VERBATIM — 6x6, aiStyle
    /// 31, penetrate −1, the stock 3600 life (the arm caps it at 133),
    /// extraUpdates 2, tileCollide false, alpha 255, ignoreWater true, NO
    /// class flag, NO immunity pair. The d18 copy (`78ffe8e11`) is the
    /// keep-one survivor (the alpha/ignoreWater the f25 copy dropped).
    /// WRITTEN NOT RUN (code-only retire round).
    #[test]
    fn w4_d1_retire_the_solution_spray_rows_are_the_145_shape() {
        for k in (146..=149u16).chain(1015..=1017) {
            let d = proj_data(k).unwrap_or_else(|| panic!("kind {k} row"));
            assert_eq!((d.w, d.h), (6.0, 6.0), "kind {k} box");
            assert_eq!(d.ai_style, 31, "kind {k} style");
            assert_eq!(d.penetrate, -1, "kind {k} penetrate");
            assert_eq!(d.time_left, 3600, "kind {k} stock life");
            assert_eq!(d.extra_updates, 2, "kind {k} extraUpdates");
            assert!(!d.tile_collide, "kind {k} tileCollide false");
            assert!(d.ignore_water, "kind {k} ignoreWater (`:2358`/`:10701`)");
            assert_eq!(d.alpha, 255, "kind {k} alpha (`:2356`/`:10697`)");
            assert!(!d.uses_local_immunity && !d.uses_id_static_immunity, "kind {k} stock immunity");
            assert!(!d.melee && !d.ranged && !d.magic, "kind {k} no class flag");
            assert!(d.ignore_water, "kind {k} ignoreWater (`:2360`/`:10701`)");
            assert_eq!(d.alpha, 255, "kind {k} alpha (`:2356`/`:10697`)");
            assert!(!d.uses_local_immunity && !d.uses_id_static_immunity, "kind {k} stock immunity");
            assert!(!d.melee && !d.ranged && !d.magic, "kind {k} no class flag");
            assert!(!d.arrow && !d.trap, "kind {k} no arrow/trap flag");
        }
        // The rows enter the atlas pack set (the sheets exist for all seven).
        let spawnable = spawnable_kinds();
        for k in (146..=149u16).chain(1015..=1017) {
            assert!(spawnable.contains(&k), "kind {k} packs");
        }
    /// W4-e7 lane 23 — the Snapping Stone strike effect rows
    /// (`Projectile.cs:10491-10503`/`:10611-10635`): 10×10, aiStyle 0,
    /// stock penetrate 1, tileCollide false, timeLeft 2, local-immunity 60;
    /// 1121 ranged, 1131 melee, 1132 no class. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_e7_the_snapping_stone_effect_rows_match_set_defaults() {
        for k in [SNAPPING_STONE_EFFECT, SNAPPING_STONE_MELEE_EFFECT, SNAPPING_STONE_SUMMON_EFFECT] {
            let d = proj_data(k).expect("the strike effect row");
            assert_eq!((d.w, d.h), (10.0, 10.0), "kind {k} (:10493-10494)");
            assert_eq!(d.ai_style, 0, "kind {k} — the :778 reset, no AI arm");
            assert_eq!(d.penetrate, 1, "kind {k} — the stock :774");
            assert_eq!(d.time_left, 2, "kind {k} (:10499)");
            assert!(!d.tile_collide, "kind {k} (:10495)");
            assert!(d.uses_local_immunity, "kind {k} (:10497)");
            assert_eq!(d.local_hit_cooldown, 60, "kind {k} (:10498)");
            assert!(!d.uses_id_static_immunity, "kind {k}");
            assert_eq!(d.alpha, 0, "kind {k} — the stock :779");
            assert_eq!(d.scale, 1.0, "kind {k} — the stock :785");
        }
        assert_eq!(proj_data(SNAPPING_STONE_EFFECT).unwrap().ranged, true, "(:10500)");
        assert_eq!(proj_data(SNAPPING_STONE_EFFECT).unwrap().melee, false);
        assert_eq!(proj_data(SNAPPING_STONE_MELEE_EFFECT).unwrap().melee, true, "(:10620)");
        assert_eq!(proj_data(SNAPPING_STONE_MELEE_EFFECT).unwrap().ranged, false);
        let summon = proj_data(SNAPPING_STONE_SUMMON_EFFECT).unwrap();
        assert!(!summon.melee && !summon.ranged && !summon.magic, "(:10624-10635) sets none");
    }
    /// S1-E-21h d75 part 5 — the 502 Meowmere row (`Projectile.cs:5732-5742`):
    /// the verbatim `SetDefaults` arm — 16x16, aiStyle 8, `penetrate = 5`,
    /// melee, local immunity 10; everything else keeps the stock reset.
    #[test]
    fn s1_e_21h_d75_p5_the_meowmere_row_is_the_verbatim_arm() {
        let d = proj_data(MEOWMERE).expect("the 502 row arms the owed gate");
        assert_eq!((d.w, d.h), (16.0, 16.0));
        assert_eq!(d.ai_style, 8);
        assert_eq!(d.penetrate, 5);
        assert!(d.melee, "`:5738`");
        assert!(!d.ranged && !d.magic);
        assert!(d.uses_local_immunity, "`:5740`");
        assert_eq!(d.local_hit_cooldown, 10, "`:5741`");
        assert!(!d.uses_id_static_immunity);
        // The stock reset the arm never overrides.
        assert_eq!(d.time_left, TIME_LEFT_DEFAULT);
        assert_eq!(d.extra_updates, 0);
        assert!(d.tile_collide);
        assert_eq!(d.alpha, 0);
        assert_eq!(d.scale, 1.0);
        assert_eq!(d.light, 0.0, "no `this.light` line — ProjLight stays off");
        assert!(!d.arrow && !d.trap && !d.minion && !d.sentry);
        assert!(!d.ignore_water && !d.net_important);
        assert!(!d.applies_immunity_time_on_single_hits);
        // The sheet packs (`spawnable_kinds` feeds the atlas walk).
        assert!(spawnable_kinds().contains(&MEOWMERE));
    }

    /// S1-E-21h d75 part 6 — the 19 phaseblade/phaser rows
    /// (`ProjectileID.cs:192`'s `IsAPhaseblade` set): the small group
    /// (`Projectile.cs:10006-10021`, 38x38) and the large group
    /// (`:10022-10037`, 46x46) + the 1074's own arm (`:10047-10058`,
    /// 46x46) — all aiStyle 3, melee, `penetrate = -1`, local immunity
    /// 10, `netImportant`; the stock reset stands. The 1073 trap
    /// (`:10038-10046`) is NOT in the set and stays rowless.
    #[test]
    fn s1_e_21h_d75_p6_the_phaseblade_rows_are_the_verbatim_groups() {
        for kind in [1059u16, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067,
            1068, 1069, 1070, 1071, 1072, 1074, 1075, 1076, 1101, 1102]
        {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} rowed"));
            let small = matches!(kind, 1059..=1064 | 1071 | 1075 | 1101);
            let box_size = if small { 38.0 } else { 46.0 };
            assert_eq!((d.w, d.h), (box_size, box_size), "kind {kind}");
            assert_eq!(d.ai_style, 3, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert!(d.melee, "kind {kind}: `:10013`/`:10029`/`:10054`");
            assert!(!d.ranged && !d.magic, "kind {kind}");
            assert!(d.uses_local_immunity, "kind {kind}");
            assert_eq!(d.local_hit_cooldown, 10, "kind {kind}");
            assert!(d.net_important, "kind {kind}: `:10014`/`:10030`/`:10055`");
            // The stock reset the arms never override.
            assert_eq!(d.time_left, TIME_LEFT_DEFAULT, "kind {kind}");
            assert_eq!(d.extra_updates, 0, "kind {kind}");
            assert!(d.tile_collide, "kind {kind}");
            assert_eq!(d.alpha, 0, "kind {kind}");
            assert_eq!(d.scale, 1.0, "kind {kind}");
            assert!(!d.arrow && !d.trap && !d.minion && !d.sentry, "kind {kind}");
            assert!(!d.ignore_water, "kind {kind}");
            assert!(spawnable_kinds().contains(&kind), "kind {kind} packs");
        }
        // The 1073 DartTrap shot is NOT a phaseblade (`:10038-10046`:
        // 10x10 aiStyle 1 hostile trap — a different lane's row).
        assert!(proj_data(1073).is_none(), "the 1073 stays out of the set");
    }

    /// W4-g2 reconciliation — the four sibling rows the first census missed
    /// (`163-projectile-rowless-kinds-w4-g2.md` "Recorded gaps"): the melee
    /// 228/732 (`Projectile.cs:3144`/`:7713`) and the magic 239/294
    /// (`:3214`/`:3844`). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn the_w4_g2_sibling_class_flags_are_census_complete() {
        let flags = |k: u16| {
            let d = proj_data(k).expect("a row");
            (d.melee, d.ranged, d.magic)
        };
        // The Partisan's spore cloud 228: melee (`:3144`), no ranged/magic.
        assert_eq!(flags(CHLOROPHYTE_SPORE_CLOUD), (true, false, false));
        // The Thunder Spear's bolt 732: melee (`:7713`).
        assert_eq!(flags(THUNDER_SPEAR_BOLT), (true, false, false));
        // The Nimbus Rod's rain drop 239: magic (`:3214`).
        assert_eq!(flags(RAIN_FRIENDLY), (false, false, true));
        // The Shadowbeam Staff's beam 294: magic (`:3844`).
        assert_eq!(flags(SHADOWBEAM), (false, false, true));
    }

    #[test]
    fn w4_d6_retire_the_601_row_is_the_portal_bolt_not_cannon_ammo() {
        // The d18 finding (dd7b1b0a, the PROJ-171 retire phase-3 census) fixed
        // here: 601 = `ProjectileID.PortalGunBolt` (`ProjectileID.cs:1756`).
        // The C# block `Projectile.cs:6502-6510`: 10x10, aiStyle 1, alpha
        // 255, friendly, extraUpdates 30 — penetrate stays the stock 1
        // (`:774`), timeLeft the stock 3600 (`:787`), tileCollide the stock
        // true (`:775`). The old M7-WIRE-2 row carried -1/alpha 0 under the
        // 178-cannonball comment; the 178 row is untouched.
        let bolt = proj_data(CANNON_EXPLOSIVE).expect("the 601 row");
        assert_eq!(CANNON_EXPLOSIVE, 601);
        assert_eq!((bolt.w, bolt.h), (10.0, 10.0), ":6504-6505");
        assert_eq!(bolt.ai_style, 1, ":6506");
        assert_eq!(bolt.alpha, 255, ":6507");
        assert_eq!(bolt.penetrate, 1, "the stock :774 — NOT the 178's -1");
        assert_eq!(bolt.extra_updates, 30, ":6509");
        assert_eq!(bolt.time_left, TIME_LEFT_DEFAULT, "the stock :787");
        assert!(bolt.tile_collide, "the stock :775");
        assert!(!bolt.arrow && !bolt.trap);
        let bomb = proj_data(CANNON_BOMB).expect("the 178 row");
        assert_eq!(bomb.penetrate, -1, ":2647");
        assert_eq!(bomb.time_left, 2, ":2648");
        // The aiStyle-1 slice-B fade consumes the 255 spawn: 255 - 17*15 = 0
        // clamps on tick 18 (`:54096-54100`); the trailing-mode row keeps 0
        // (`ProjectileID.cs:521` lists 601 -> 0).
        assert_eq!(crate::projectile::trailing_mode(CANNON_EXPLOSIVE), 0);
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

    /// W4-d8 — the aiStyle-72/73/74/76 rows keep their verbatim `SetDefaults`
    /// slices (the data half of the census). Every value is hand-derived from
    /// the cited `Projectile.cs` lines; the stock defaults (`:714-715`,
    /// `:764`, `:774-775`, `:779`, `:785`, `:787`) stand where the arm writes
    /// nothing. WRITTEN, NOT RUN (code-only mode).
    /// W4-d78 batch 2 — the Kill child-spawn family's rows (WRITTEN, NOT RUN
    /// — code-only mode). Each is the verbatim `SetDefaults` slice against
    /// the C# arm cited per row; the class flags and frame counts ride the
    /// census pins above.
    #[test]
    fn w4_d78_b2_the_child_spawn_family_rows_match_set_defaults() {
        // The Stynger bolt (246, `:3289-3298`): 10x10 aiStyle 1, ranged,
        // alpha 255, extraUpdates 1, the stock penetrate 1 / 3600 / collide.
        let d = proj_data(STYNGER).expect("the 246 row");
        assert_eq!((d.w, d.h, d.ai_style), (10.0, 10.0, 1));
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (1, 3600, 1));
        assert!(d.tile_collide);
        assert_eq!(d.alpha, 255);
        assert_eq!((d.melee, d.ranged, d.magic), (false, true, false));
        assert!(!d.arrow && !d.trap);
        // The Stynger shrapnel (249, `:3314-3321`): 12x12 aiStyle 2, ranged,
        // the stock penetrate 1 / 3600 / collide, alpha at the 0 reset.
        let d = proj_data(STYNGER_SHRAPNEL).expect("the 249 row");
        assert_eq!((d.w, d.h, d.ai_style), (12.0, 12.0, 2));
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (1, 3600, 0));
        assert!(d.tile_collide);
        assert_eq!(d.alpha, 0);
        assert_eq!((d.melee, d.ranged, d.magic), (false, true, false));
        // The Arcanum death shard (620, `:6681-6690`): 8x8 aiStyle 29, magic,
        // alpha 255, penetrate 1, the stock 3600 / collide.
        let d = proj_data(NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD).expect("the 620 row");
        assert_eq!((d.w, d.h, d.ai_style), (8.0, 8.0, 29));
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (1, 3600, 0));
        assert!(d.tile_collide);
        assert_eq!(d.alpha, 255);
        assert_eq!((d.melee, d.ranged, d.magic), (false, false, true));
        // The Bone Shard (1124, `:10514-10527`): 6x6 aiStyle 1, ranged,
        // penetrate 1, timeLeft 600, extraUpdates 1, scale 0.95 (the LAST
        // write wins over the 1.2 double-set), alpha at the 0 reset.
        let d = proj_data(BONE_SHARD).expect("the 1124 row");
        assert_eq!((d.w, d.h, d.ai_style), (6.0, 6.0, 1));
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (1, 600, 1));
        assert!(d.tile_collide);
        assert_eq!(d.alpha, 0);
        assert_eq!(d.scale, 0.95);
        assert_eq!((d.melee, d.ranged, d.magic), (false, true, false));
        // The frame slices: 249 = 5, 1124 = 2; 246/620 keep the stock 1.
        assert_eq!(proj_frames(STYNGER_SHRAPNEL), 5);
        assert_eq!(proj_frames(BONE_SHARD), 2);
        assert_eq!(proj_frames(STYNGER), 1);
        assert_eq!(proj_frames(NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD), 1);
        // Every child kind reaches the spawner (the rows are not dead data).
        for k in [STYNGER, STYNGER_SHRAPNEL, NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD, BONE_SHARD] {
            assert!(spawnable_kinds().contains(&k), "kind {k} packs its sheet");
        }
        // The aiStyle-74 Scutlix laser (434, `:5047-5055`): 1x1, extraUpdates
        // 100, penetrate −1, the stock timeLeft 3600 / tileCollide true.
        let s = proj_data(SCUTLIX_LASER_FRIENDLY).unwrap();
        assert_eq!((s.w, s.h), (1.0, 1.0));
        assert_eq!(s.ai_style, 74);
        assert_eq!(s.extra_updates, 100);
        assert_eq!(s.penetrate, -1);
        assert_eq!(s.time_left, 3600);
        assert!(s.tile_collide);
        // The aiStyle-76 mount crosshairs (441/453, `:5117-5125`/`:5250-5258`):
        // 1x1, tileCollide false, ignoreWater; the stock penetrate 1 /
        // timeLeft 3600 / friendly false (`:788`, held by the arm).
        for k in MOUNT_CROSSHAIRS {
            let d = proj_data(k).unwrap();
            assert_eq!((d.w, d.h), (1.0, 1.0), "kind {k}");
            assert_eq!(d.ai_style, 76, "kind {k}");
            assert!(!d.tile_collide, "kind {k}");
            assert!(d.ignore_water, "kind {k}");
            assert_eq!(d.penetrate, 1, "kind {k}");
            assert_eq!(d.time_left, 3600, "kind {k}");
        }
    }
    }

    /// W4-d78 FINAL — the 620-row co-claim equivalence pin (WRITTEN, NOT RUN
    /// — code-only mode). d77's Kill-host wave (`0573b70f`, tip `5f10560b`)
    /// rows the SAME kind as `NEBULA_ARCANUM_SHARD` (its data.rs:1058/:2833):
    /// `(8, 8, 29, 1, TIME_LEFT_DEFAULT, 0, true, false, false,
    /// (false, -2, false, -1, 255, 1.0))` — field-for-field THIS row. The
    /// merge keeps ONE row/const (research/372 §3); this witness pins the
    /// values both sides carry so either survivor passes it verbatim.
    #[test]
    fn w4_d78_final_the_shard_row_matches_the_d77_co_claim() {
        let d = proj_data(NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD).expect("the 620 row");
        // d77's tuple, term by term (its data.rs:2833).
        assert_eq!((d.w, d.h), (8.0, 8.0));
        assert_eq!(d.ai_style, 29);
        assert_eq!(d.penetrate, 1);
        assert_eq!(d.time_left, TIME_LEFT_DEFAULT);
        assert_eq!(d.extra_updates, 0);
        assert!(d.tile_collide);
        assert!(!d.melee && !d.ranged && d.magic);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.scale, 1.0);
        // The stock immunity tuple both rows carry (no local pair, no
        // id-static, alpha 255 — the AI_029 fuse kills at 40 ticks).
        assert!(!d.uses_local_immunity);
        assert_eq!(d.local_hit_cooldown, -2);
        assert!(!d.uses_id_static_immunity);
    /// W4-g2 (d5 F1/F4/F5/F6) — the nine rowless kinds' `SetDefaults`
    /// slices, WRITTEN NOT RUN (code-only mode): every value re-read from
    /// the cited C# arms. (w, h, aiStyle, penetrate, timeLeft, extraUpdates,
    /// tileCollide, alpha, scale).
    #[test]
    fn the_w4_g2_rowless_rows_are_verbatim() {
        let cases: [(u16, f32, f32, u8, i32, i32, u32, bool, i32, f32); 9] = [
            // `Projectile.cs:3151-3163` — the orb: stock tileCollide TRUE.
            (CHLOROPHYTE_ORB, 30.0, 30.0, 44, -1, 3600, 0, true, 0, 1.0),
            // `:3255-3262` — the moving blood cloud (the 237 twin).
            (BLOOD_CLOUD_MOVING, 28.0, 28.0, 45, -1, 3600, 0, true, 0, 1.0),
            // `:3263-3273` — the settled blood cloud (the 238 twin).
            (BLOOD_CLOUD_RAINING, 54.0, 28.0, 45, -1, 18000, 0, false, 0, 1.0),
            // `:3274-3288` — the blood drop: penetrate 2, NOT the 239's 5.
            (BLOOD_RAIN, 4.0, 40.0, 45, 2, 300, 1, true, 0, 1.1),
            // `:3491-3502` — the hostile nimbus drop.
            (RAIN_NIMBUS, 4.0, 40.0, 45, -1, 120, 1, true, 0, 1.1),
            // `:3793-3803` — the hostile beam.
            (SHADOWBEAM_HOSTILE, 4.0, 4.0, 48, -1, 100, 100, true, 0, 1.0),
            // `:3804-3814` — the hostile bolt.
            (INFERNO_HOSTILE_BOLT, 12.0, 12.0, 50, -1, 3600, 0, false, 255, 1.0),
            // `:3815-3825` — the hostile blast: 130, NOT the 296's 150.
            (INFERNO_HOSTILE_BLAST, 130.0, 130.0, 50, -1, 3600, 0, false, 255, 1.0),
            // `:5037-5046` — the UFO laser: STOCK penetrate 1.
            (UFO_LASER, 8.0, 8.0, 48, 1, 100, 100, true, 0, 1.0),
        ];
        for (kind, w, h, style, pen, life, extra, collide, alpha, scale) in cases {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} rowed"));
            assert_eq!((d.w, d.h), (w, h), "kind {kind} box");
            assert_eq!(d.ai_style, style, "kind {kind} aiStyle");
            assert_eq!(d.penetrate, pen, "kind {kind} penetrate");
            assert_eq!(d.time_left, life, "kind {kind} timeLeft");
            assert_eq!(d.extra_updates, extra, "kind {kind} extraUpdates");
            assert_eq!(d.tile_collide, collide, "kind {kind} tileCollide");
            assert_eq!(d.alpha, alpha, "kind {kind} alpha");
            assert_eq!(d.scale, scale, "kind {kind} scale");
        }
        // The immunity pairs: the orb / blood drop ride idStatic-10
        // (`:3161-3162`/`:3286-3287`); the rest keep the reset.
        for kind in [CHLOROPHYTE_ORB, BLOOD_RAIN] {
            let d = proj_data(kind).unwrap();
            assert!(d.uses_id_static_immunity, "kind {kind} idStatic");
            assert_eq!(d.id_static_hit_cooldown, 10, "kind {kind} cooldown");
            assert!(!d.uses_local_immunity, "kind {kind} no local pair");
        }
        for kind in [BLOOD_CLOUD_MOVING, BLOOD_CLOUD_RAINING, RAIN_NIMBUS,
            SHADOWBEAM_HOSTILE, INFERNO_HOSTILE_BOLT, INFERNO_HOSTILE_BLAST, UFO_LASER]
        {
            let d = proj_data(kind).unwrap();
            assert!(!d.uses_local_immunity && !d.uses_id_static_immunity, "kind {kind} reset");
        }
        // The class flags: melee 229 (`:3157`), magic 245/290/291/292
        // (`:3284`/`:3799`/`:3811`/`:3822`); the clouds/drops 243/244/264
        // and the laser 433 set NONE.
        assert!(proj_data(CHLOROPHYTE_ORB).unwrap().melee);
        for kind in [BLOOD_RAIN, SHADOWBEAM_HOSTILE, INFERNO_HOSTILE_BOLT, INFERNO_HOSTILE_BLAST] {
            assert!(proj_data(kind).unwrap().magic, "kind {kind} magic");
        }
        for kind in [BLOOD_CLOUD_MOVING, BLOOD_CLOUD_RAINING, RAIN_NIMBUS, UFO_LASER] {
            let d = proj_data(kind).unwrap();
            assert!(!d.melee && !d.ranged && !d.magic, "kind {kind} no class flag");
        }
        // netImportant: the two blood clouds (`:3257`/`:3265`); the drop
        // 245 and the rest keep false.
        assert!(proj_data(BLOOD_CLOUD_MOVING).unwrap().net_important);
        assert!(proj_data(BLOOD_CLOUD_RAINING).unwrap().net_important);
        assert!(!proj_data(BLOOD_RAIN).unwrap().net_important);
        // ignoreWater: 244/245/264/433 (`:3267`/`:3276`/`:3493`/`:5045`);
        // the moving cloud 243 keeps false.
        for kind in [BLOOD_CLOUD_RAINING, BLOOD_RAIN, RAIN_NIMBUS, UFO_LASER] {
            assert!(proj_data(kind).unwrap().ignore_water, "kind {kind} ignoreWater");
        }
        assert!(!proj_data(BLOOD_CLOUD_MOVING).unwrap().ignore_water);
        // The orb's light 0.2 (`:3159`) — every other new kind stays dark.
        assert_eq!(proj_data(CHLOROPHYTE_ORB).unwrap().light, 0.2);
        // The frame sheets (`Main.cs:8606-8609`): orb 4, clouds 4/6; the
        // drops/beams/bolts/blasts keep the stock 1.
        assert_eq!(proj_frames(CHLOROPHYTE_ORB), 4);
        assert_eq!(proj_frames(BLOOD_CLOUD_MOVING), 4);
        assert_eq!(proj_frames(BLOOD_CLOUD_RAINING), 6);
        for kind in [BLOOD_RAIN, RAIN_NIMBUS, SHADOWBEAM_HOSTILE,
            INFERNO_HOSTILE_BOLT, INFERNO_HOSTILE_BLAST, UFO_LASER]
        {
            assert_eq!(proj_frames(kind), 1, "kind {kind} stock sheet");
        }
    }

    /// W4-g2d rowless-rider batch — the Silver Bullet 981's `SetDefaults`
    /// slice (`Projectile.cs:9340-9353`), WRITTEN NOT RUN (code-only mode).
    #[test]
    fn the_w4_g2d_silver_bullet_row_is_verbatim() {
        let d = proj_data(SILVER_BULLET).expect("the 981 row landed");
        assert_eq!((d.w, d.h), (4.0, 4.0), ":9342-9343");
        assert_eq!(d.ai_style, 1, ":9344");
        assert_eq!(d.penetrate, 1, ":9346");
        assert_eq!(d.light, 0.5, ":9347");
        assert_eq!(d.alpha, 255, ":9348");
        assert_eq!(d.scale, 1.2, ":9349");
        assert_eq!(d.time_left, 600, ":9350");
        assert!(d.ranged && !d.melee && !d.magic, ":9351");
        assert_eq!(d.extra_updates, 1, ":9352 — NOT the hardmode 2");
        assert!(d.tile_collide, "the stock true (:775) stands");
        assert!(!d.ignore_water && !d.net_important && !d.arrow && !d.trap);
        assert!(!d.uses_local_immunity && !d.uses_id_static_immunity);
        assert_eq!(proj_frames(SILVER_BULLET), 1, "no Main.projFrames override");
        // The rowed siblings stay rowless (the g2d task-seeds, research/26d-g2d):
        // the 19 phaseblades need the AI_003 machine, the 491 needs _Old.
        for kind in [1059, 1065, 1074, 1075, 1076, 1101, 1102, 491] {
            assert!(proj_data(kind).is_none(), "kind {kind} stays rowless (task-seed)");

        }        // The merge compose: d75 part 6 ROWED + MACHINED the 19 phaseblades
        // (the AI_003 machine landed with the rows — the g2d task-seed
        // RETIRED, research/32's SEAL). Only the 491 keeps its rowless
        // seed (the _Old machine is still owed).
        for kind in [1059, 1065, 1074, 1075, 1076, 1101, 1102] {
            assert!(proj_data(kind).is_some(), "kind {kind} rowed by d75 part 6");
        }
        assert!(proj_data(491).is_none(), "the 491 stays rowless (task-seed)");
    }
    /// W4-d8 — the aiStyle-72/73/74/76 rows keep their verbatim `SetDefaults`
    /// slices (the data half of the census). Every value is hand-derived from
    /// the cited `Projectile.cs` lines; the stock defaults (`:714-715`,
    /// `:764`, `:774-775`, `:779`, `:785`, `:787`) stand where the arm writes
    /// nothing. WRITTEN, NOT RUN (code-only mode).
    #[test]
    fn w4_d8_the_proj_ai_71_80_missing_rows_match_set_defaults() {
        // The aiStyle-72 Bubble (410, `:4917-4932`): 14x14, penetrate 1,
        // timeLeft 100, alpha 255, magic, ignoreWater; the stock tileCollide
        // true (`:775`) and extraUpdates 0 stand. The remix `armorPenetration
        // = 20` arm (`:4928-4931`) is unported (no remix flag).
        let b = proj_data(BUBBLE).unwrap();
        assert_eq!((b.w, b.h), (14.0, 14.0));
        assert_eq!(b.ai_style, 72);
        assert_eq!(b.penetrate, 1);
        assert_eq!(b.time_left, 100);
        assert_eq!(b.alpha, 255);
        assert_eq!(b.extra_updates, 0);
        assert!(b.tile_collide);
        assert!(b.magic);
        assert!(b.ignore_water);
        // The aiStyle-73 firework fountains (419-422, `:4951-4959`): 4x4,
        // the idStatic-3 pair, the stock penetrate 1 / tileCollide true,
        // timeLeft 600 (the `NewProjectile` seed `:11033-11048`). NO class
        // flag (`:4951-4959` sets none).
        for k in FIREWORK_FOUNTAINS {
            let d = proj_data(k).unwrap();
            assert_eq!((d.w, d.h), (4.0, 4.0), "kind {k}");
            assert_eq!(d.ai_style, 73, "kind {k}");
            assert_eq!(d.penetrate, 1, "kind {k}");
            assert_eq!(d.time_left, 600, "kind {k}");
            assert!(d.tile_collide, "kind {k}");
            assert!(d.uses_id_static_immunity, "kind {k}");
            assert_eq!(d.id_static_hit_cooldown, 3, "kind {k}");
            assert_eq!((d.melee, d.ranged, d.magic), (false, false, false), "kind {k}");
        // The aiStyle-74 Scutlix laser (434, `:5047-5055`): 1x1, extraUpdates
        // 100, penetrate −1, the stock timeLeft 3600 / tileCollide true.
        let s = proj_data(SCUTLIX_LASER_FRIENDLY).unwrap();
        assert_eq!((s.w, s.h), (1.0, 1.0));
        assert_eq!(s.ai_style, 74);
        assert_eq!(s.extra_updates, 100);
        assert_eq!(s.penetrate, -1);
        assert_eq!(s.time_left, 3600);
        assert!(s.tile_collide);
        // The aiStyle-76 mount crosshairs (441/453, `:5117-5125`/`:5250-5258`):
        // 1x1, tileCollide false, ignoreWater; the stock penetrate 1 /
        // timeLeft 3600 / friendly false (`:788`, held by the arm).
        for k in MOUNT_CROSSHAIRS {
            assert_eq!((d.w, d.h), (1.0, 1.0), "kind {k}");
            assert_eq!(d.ai_style, 76, "kind {k}");
            assert!(!d.tile_collide, "kind {k}");
            assert!(d.ignore_water, "kind {k}");
            assert_eq!(d.time_left, 3600, "kind {k}");
        }

        }    /// W4-c11 witness — `Projectile.cs:5056-5063`: the Martian Turret's
    /// bolt (435) is 10x10, aiStyle 1, hostile, `ignoreWater = true`, with
    /// the stock penetrate 1 (`:774`), tileCollide (`:775`), timeLeft 3600
    /// (`:787`) and extraUpdates 0 — the arm writes no override. The
    /// aiStyle-73 shot (`npc_ai_style73::TurretBolt`) spawns through
    /// `spawn_hostile`, which reads exactly this row.
    fn the_martian_turret_bolt_row_matches_the_csharp() {
        let d = proj_data(MARTIAN_TURRET_BOLT).expect("the 435 row");
        assert_eq!((d.w, d.h), (10.0, 10.0), ":5058-5059");
        assert_eq!(d.ai_style, 1, ":5060");
        assert!(d.ignore_water, ":5062");
        assert_eq!(d.penetrate, 1, ":774 stock");
        assert!(d.tile_collide, ":775 stock");
        assert_eq!(d.time_left, 3600, ":787 stock");
        assert_eq!(d.extra_updates, 0);
        assert!(!d.arrow && !d.trap, "the arm sets neither");
        assert_eq!(d.alpha, 0, ":779 stock");
        assert_eq!(d.scale, 1.0, ":785 stock");
        // `Main.projFrames[435] = 4` (`Main.cs:8532`) — the sheet slices 4.
        assert_eq!(proj_frames(MARTIAN_TURRET_BOLT), 4);
        // The Scrambler twin (436) shares the shape but is a distinct row
        // (its ignoreWater stays the documented S1-D-5 derivation gap).
        assert_ne!(proj_data(MARTIAN_TURRET_BOLT), proj_data(BRAIN_SCRAMBLER));
    }

// balance-close (union shared delimiter)
    /// W4-d12 census F1 — the EXACT aiStyle-113 SetDefaults census: six kinds
    /// ride `this.aiStyle = 113` (`Projectile.cs:6474`/`:6615`/`:6842`/
    /// `:9201`/`:9291`/`:9697`); every one carries a row. The shadowflame
    /// family is NOT on the style: 495 is aiStyle 1 (`:5681`, the arrow row),
    /// 496 is aiStyle 91 (`:5692`, d10's decade), 659 is aiStyle 129
    /// (`:7083`, the rowed Spirit Flame).
    #[test]
    fn w4_d12_f1_the_113_census_rows() {
        for k in [BONE_JAVELIN, STARDUST_CELL_MINION_SHOT, DAYBREAK, TENTACLE_SPIKE,
                  BLOOD_BUTCHERER, DEAD_CELLS_KILLING_DECK_CARD] {
            let d = proj_data(k).expect("the 113 census row");
            assert_eq!(d.ai_style, 113, "kind {k}");
            assert_eq!((d.w, d.h), (16.0, 16.0), "kind {k}: the 16x16 box");
            assert_eq!(d.penetrate, -1, "kind {k}: penetrate -1");
            assert_eq!(d.time_left, 3600, "kind {k}: the stock life (:787)");
            assert!(d.tile_collide, "kind {k}: the stock tileCollide (:775)");
            assert!(!d.arrow && !d.trap, "kind {k}");
        }
        // The shadowflame guess stays OUT of the style.
        assert_eq!(proj_data(495).unwrap().ai_style, 1, ":5681");
        assert_eq!(proj_data(SPIRIT_FLAME).unwrap().ai_style, 129, ":7083");
        // 496 ShadowFlame rides aiStyle 91 (`:5692`, d10's decade) — rowless
        // on this branch; when d10 lands its row the style must be 91.
        if let Some(d) = proj_data(496) {
            assert_eq!(d.ai_style, 91, ":5692");
        }
        // The per-kind state: alpha / class flags / updates / immunity.
        let cell = proj_data(STARDUST_CELL_MINION_SHOT).unwrap();
        assert_eq!(cell.alpha, 255, ":6617");
        assert_eq!(cell.extra_updates, 0, ":6611-6619 no MaxUpdates");
        assert!(!cell.melee && !cell.ranged && !cell.magic, "the cell sets no class");
        assert!(!cell.uses_local_immunity, "the stock reset");
        let day = proj_data(DAYBREAK).unwrap();
        assert_eq!(day.alpha, 255, ":6846");
        assert_eq!(day.extra_updates, 1, ":6848 MaxUpdates = 2");
        assert!(day.melee, ":6844");
        let spike = proj_data(TENTACLE_SPIKE).unwrap();
        assert_eq!(spike.alpha, 255, ":9204");
        assert!(spike.melee, ":9203");
        let butcher = proj_data(BLOOD_BUTCHERER).unwrap();
        assert_eq!(butcher.alpha, 0, ":9294 (hide=true is the F3 skip)");
        assert!(butcher.melee, ":9293");
        let card = proj_data(DEAD_CELLS_KILLING_DECK_CARD).unwrap();
        assert_eq!(card.alpha, 255, ":9700");
        assert_eq!(card.extra_updates, 1, ":9705");
        assert!(card.magic, ":9699");
        assert!(card.uses_local_immunity && card.local_hit_cooldown == -1, ":9702-9703");
        // The frame sheets (`Main.cs:8495`/`:8444`; the rest init 1).
        assert_eq!(proj_frames(STARDUST_CELL_MINION_SHOT), 4);
        assert_eq!(proj_frames(DEAD_CELLS_KILLING_DECK_CARD), 8);
        for k in [BONE_JAVELIN, DAYBREAK, TENTACLE_SPIKE, BLOOD_BUTCHERER] {
            assert_eq!(proj_frames(k), 1, "kind {k}: no override (Main.cs:8441)");
        }
    }

    /// W4-d12 census F1 — the explosion kinds the d12 round armed ALL carry
    /// rows (`spawn_with_data` declines rowless kinds): 612/624/953/1044 the
    /// 117 family, 608 the Kill-path counter, 644 the 112 crystal shard.
    #[test]
    fn w4_d12_f1_the_explosion_kinds_all_have_rows() {
        for (k, style) in [
            (SOLAR_WHIP_SWORD_EXPLOSION, 117u8),
            (STARDUST_GUARDIAN_EXPLOSION, 117),
            (DAYBREAK_EXPLOSION, 117),
            (DEAD_CELLS_MUSHROOM_BOI_EXPLOSION, 117),
            (SOLAR_COUNTER, 117),
            (RAINBOW_CRYSTAL_EXPLOSION, 112),
        ] {
            let d = proj_data(k).unwrap_or_else(|| panic!("kind {k} rowless"));
            assert_eq!(d.ai_style, style, "kind {k}");
        }
        // The 978 flag58 term: Volcano's SetDefaults is aiStyle 189
        // (`Projectile.cs:9253-9268`). W4-d12 DOC-SWEEP item 7: d19's
        // a4185f89 landed the 189 ARM but NOT the row, so the row ports here
        // (`VOLCANO`). The flag58 term in the 117 arm stays unreachable for
        // it (verbatim) — the 978 rides the 189 dispatch.
        let d = proj_data(VOLCANO).expect("the Volcano row landed (DOC-SWEEP item 7)");
        assert_eq!(d.ai_style, 189, ":9257");
        assert_eq!(d.time_left, 30, "the second timeLeft = 30 wins (:9268)");
        assert_eq!(d.penetrate, 3, ":9264");
        assert!(d.uses_local_immunity && d.local_hit_cooldown == -1, ":9265-9266");
        assert!(d.melee, ":9267");
        assert_eq!(proj_frames(VOLCANO), 5, "Main.cs:8493");
    }

    /// W4-d16 residual 5 — the aiStyle-161 shortsword census (the g4 G4
    /// finding: styles 160/161 ride the `DefaultToKite`/`DefaultToShortsword`
    /// helpers). `DefaultToShortsword` (`Projectile.cs:10773-10787`): 18x18,
    /// aiStyle 161, friendly, penetrate −1, tileCollide FALSE, scale 1,
    /// ownerHitCheck, melee, extraUpdates 1, drawLayer 7, usesOwnerLight.
    /// The FOUR call sites cover exactly ELEVEN kinds: the Gladius 802
    /// (`:7921-7924`), the Ruler 842 (`:8145-8148`), the eight ore
    /// shortswords 938-945 (`:8917-8921`, + timeLeft 360) and the
    /// Chlorophyte Saber 1127 (`:10559-10564`, + usesLocalNPCImmunity /
    /// localNPCHitCooldown −1). The dispatch is `AI_161_RapierStabs`
    /// (`:36607-36610` → `:47612-47665`). NO other `aiStyle = 161` write
    /// exists in Projectile.cs (census). The port rows are UNROWED — the
    /// 161-170 AI lane is d17's read claim; this witness pins the census
    /// so the rowing pass lands the exact helper shape. WRITTEN NOT RUN.
    #[test]
    fn w4_d16_161_the_shortsword_helper_census() {
        // The eleven kinds the four DefaultToShortsword call sites cover.
        const SHORTSWORD_KINDS: [u16; 11] =
            [802, 842, 938, 939, 940, 941, 942, 943, 944, 945, 1127];
        // The item rows shoot exactly these kinds (the useStyle-13 stab
        // items — the generated itemdata census).
        const SHORTSWORD_ITEMS: [(u16, u16); 11] = [
            (4463, 802),  // Gladius
            (486, 842),   // Ruler
            (3507, 938),  // Copper Shortsword
            (3501, 939),  // Tin Shortsword
            (6, 940),     // Iron Shortsword
            (3495, 941),  // Lead Shortsword
            (3513, 942),  // Silver Shortsword
            (3489, 943),  // Tungsten Shortsword
            (3519, 944),  // Gold Shortsword
            (3483, 945),  // Platinum Shortsword
            (1227, 1127), // Chlorophyte Saber
        ];
        for (item, shoot) in SHORTSWORD_ITEMS {
            let d = crate::registry::get(item).unwrap_or_else(|| panic!("item {item}"));
            assert_eq!(d.projectile, Some(shoot), "item {item}");
            assert_eq!(d.use_style, 13, "the stab useStyle");
            assert!(SHORTSWORD_KINDS.contains(&shoot));
        }
        // The port rows are UNROWED today (d17 owns the 161-170 lane):
        // when the rowing pass lands, every kind must carry the helper
        // shape — 18x18, aiStyle 161, pen −1, tileCollide false,
        // extraUpdates 1, melee. Until then `proj_data` is None and the
        // dispatch's `_ => {}` keeps the slot inert (the census pin).
        for k in SHORTSWORD_KINDS {
            if let Some(d) = proj_data(k) {
                assert_eq!((d.w, d.h, d.ai_style), (18.0, 18.0, 161), "kind {k}");
                assert_eq!(d.penetrate, -1);
                assert!(!d.tile_collide);
                assert_eq!(d.extra_updates, 1);
                assert!(d.melee);
                assert_eq!(d.scale, 1.0);
                // The per-site overrides: the ore eight's timeLeft 360,
                // the saber's local-immunity pair.
                if (938..=945).contains(&k) {
                    assert_eq!(d.time_left, 360, "kind {k}");
                }
                if k == 1127 {
                    assert!(d.uses_local_immunity);
                    assert_eq!(d.local_hit_cooldown, -1);
                }
            }
        }
    }
}
    /// W4-e3 coverage phase — the three UNROWED aiStyle-5 kinds' rows land
    /// verbatim (`Projectile.cs:5743-5756`/`:9780-9792`/`:9938-9952`), the
    /// census in docs/research/231-proj-dust-coverage.md (was 125). They unblock the
    /// `ai_falling_star_family` kind gate (the e3 batch-3 emitters were
    /// already written; g64's unwired `trail_503` twin is the merge
    /// keep-one). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_e3_the_star_wrath_and_meteor_rows_are_verbatim() {
        let star = proj_data(STAR_WRATH).expect("the 503 row");
        assert_eq!((star.w, star.h), (24.0, 24.0), ":5745-5746");
        assert_eq!(star.ai_style, 5, ":5747");
        assert_eq!(star.penetrate, 2, ":5749");
        assert_eq!(star.alpha, 255, ":5750");
        assert!(!star.tile_collide, ":5751");
        assert!(star.melee, ":5752");
        assert_eq!(star.extra_updates, 1, ":5753");
        assert!(star.uses_local_immunity, ":5754");
        assert_eq!(star.local_hit_cooldown, 15, ":5755");
        assert_eq!(star.time_left, TIME_LEFT_DEFAULT, "the stock :787");
        assert_eq!(star.scale, 1.0, "the stock :785");
        let whip = proj_data(METEOR_WHIP_METEOR).expect("the 1037 row");
        assert_eq!((whip.w, whip.h), (24.0, 24.0), ":9782-9783");
        assert_eq!(whip.ai_style, 5, ":9784");
        assert!(whip.uses_local_immunity, ":9786");
        assert_eq!(whip.local_hit_cooldown, -1, ":9787");
        assert_eq!(whip.scale, 0.8, ":9788");
        assert_eq!(whip.extra_updates, 1, ":9789");
        assert_eq!(whip.alpha, 254, ":9790");
        assert!(!whip.tile_collide, ":9791");
        assert_eq!(whip.penetrate, 1, "the stock :774 — the arm sets none");
        assert!(!whip.melee && !whip.ranged && !whip.magic, "no class flag");
        let storm = proj_data(METEOR_STORM_METEOR).expect("the 1049 row");
        assert_eq!((storm.w, storm.h), (24.0, 24.0), ":9940-9941");
        assert_eq!(storm.ai_style, 5, ":9942");
        assert_eq!(storm.scale, 0.8, ":9945");
        assert_eq!(storm.time_left, 1200, ":9946");
        assert_eq!(storm.extra_updates, 1, ":9947");
        assert_eq!(storm.alpha, 254, ":9948");
        assert_eq!(storm.penetrate, -1, ":9949");
        assert!(!storm.tile_collide, ":9950");
        assert!(storm.trap, ":9951");
        // The friendly+hostile pair (`:9943-9944`): the row carries friendly
        // (the pool's spawn default true); the hostile half is the
        // `spawn_hostile` flip, not a ProjData field (the FLAMES_TRAP shape).
        assert!(!storm.melee && !storm.ranged && !storm.magic, "no class flag");
    }
}
}}
