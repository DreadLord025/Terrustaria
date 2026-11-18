//! S1-E-7 wave D — the `Player.UpdateBuffs` MOUNT/PET/MISC block
//! (`Player.cs:11400-12069`, plus the pet family's head `:11005-11241` and
//! the mount arm `:9881-9913` the block's pets/mounts belong to).
//!
//! Disjoint-wave design (waves A/B/C own `player.rs` + `buff_effects.rs`):
//! this module is a PURE effect table — `apply_misc_effect_arm` matches one
//! `buffType[j]` id and folds its writes into a `MiscEffects` accumulator,
//! exactly like the C# arm writes into the `Player` fields. The `player.rs`
//! `UpdateBuffs` scan wires it later by copying the accumulator onto the
//! live fields (the wiring is the merge step, not this wave's). Nothing here
//! touches `BuffSlots` — the timer engine (`buffs.rs`) is untouched.
//!
//! RNG: the only draw in the block is Rabid (148)'s `Main.rand` triple
//! (`Player.cs:11324-11327`); it rides the caller's `UnifiedRandom` in the
//! exact C# order — `Next(1200)`, then (on 0) `Next(6)`, then `Next(60,100)`.

use tr_core::unified_random::UnifiedRandom;

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
pub struct MiscEffectContext<'a> {
    /// `ownedProjectileCounts` (`Player.cs:12184-12215` recount) — the pet
    /// spawn gates read it (`:12237`).
    pub owned_projectile_counts: &'a [i32],
    /// `Main.dayTime` — the Werewolf arm's gate (`:11895`).
    pub day_time: bool,
    /// `wolfAcc` / `merman` (`:11895`) — the Moon Charm / Neptune's Shell
    /// accessory flags (`UpdateEquips` writes them, `:15136-15137`).
    pub wolf_acc: bool,
    pub merman: bool,
    /// `HeldItem.type` — Tipsy's Ale-throwing branch (`:11925`).
    pub held_item_type: i32,
    /// `head`/`body`/`legs` — the fairy colour override (`:11282`).
    pub armor_head: i32,
    pub armor_body: i32,
    pub armor_legs: i32,
    /// `lavaWet` (`:11742`) / `shimmerWet` (`:11801`).
    pub lava_wet: bool,
    pub shimmer_wet: bool,
    /// The entity box + facing — the pet spawn centre (`base.Center`,
    /// `:12241`) and Shimmering's tile scan / fallStart (`:11788-11816`).
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    pub direction: i32,
    /// `velocity.Y` — Webbed's epsilon-fall choice (`:11866-11873`).
    pub vy: f32,
    /// `Main.UnderworldLayer` (`:11793`).
    pub underworld_layer: f32,
    /// `WorldGen.SolidTile3(x, y)` (`:11811`) — supplied by the caller so
    /// this table stays world-free; the scan loop itself is ported here.
    pub solid3: &'a dyn Fn(i32, i32) -> bool,
}

/// The `Player` pet/light-pet flag fields the block's arms set. One variant
/// per C# field name (the `ref bool` argument of
/// `BuffHandle_SpawnPetIfNeededAndSetTime`, `Player.cs:12227`); the wiring
/// step maps each onto the live `Player` field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetFlag {
    LightOrb,                   // 19, `Player.cs:11008`
    CrimsonHeart,               // 155, `:11022`
    CompanionCube,              // 191, `:11035`
    PetFlagDd2Dragon,           // 202, `:11039`
    PetFlagUpbeatStar,          // 217, `:11043`
    PetFlagBabyShark,           // 219, `:11047`
    PetFlagLilHarpy,            // 258, `:11051`
    PetFlagFennecFox,           // 259, `:11055`
    PetFlagGlitteryButterfly,   // 260, `:11059`
    PetFlagBabyImp,             // 261, `:11063`
    PetFlagBabyRedPanda,        // 262, `:11067`
    PetFlagPlantero,            // 264, `:11071`
    PetFlagDynamiteKitten,      // 266, `:11075`
    PetFlagBabyWerewolf,        // 267, `:11079`
    PetFlagShadowMimic,         // 268, `:11083`
    PetFlagVoltBunny,           // 274, `:11087`
    PetFlagKingSlimePet,        // 284/341, `:11091`/`:11199`
    PetFlagEyeOfCthulhuPet,     // 285, `:11095`
    PetFlagEaterOfWorldsPet,    // 286, `:11099`
    PetFlagBrainOfCthulhuPet,   // 287, `:11103`
    PetFlagSkeletronPet,        // 288, `:11107`
    PetFlagQueenBeePet,         // 289, `:11111`
    PetFlagDestroyerPet,        // 290, `:11115`
    PetFlagTwinsPet,            // 291, `:11119`
    PetFlagSkeletronPrimePet,   // 292, `:11123`
    PetFlagPlanteraPet,         // 293, `:11127`
    PetFlagGolemPet,            // 294, `:11131`
    PetFlagDukeFishronPet,      // 295, `:11135`
    PetFlagLunaticCultistPet,   // 296, `:11139`
    PetFlagMoonLordPet,         // 297, `:11143`
    PetFlagFairyQueenPet,       // 298, `:11147`
    PetFlagPumpkingPet,         // 299, `:11151`
    PetFlagEverscreamPet,       // 300, `:11155`
    PetFlagIceQueenPet,         // 301, `:11159`
    PetFlagMartianPet,          // 302, `:11163`
    PetFlagDd2OgrePet,          // 303, `:11167`
    PetFlagDd2BetsyPet,         // 304, `:11171`
    PetFlagQueenSlimePet,       // 317/341, `:11175`/`:11200`
    PetFlagBerniePet,           // 327, `:11179`
    PetFlagGlommerPet,          // 328, `:11183`
    PetFlagDeerclopsPet,        // 329, `:11187`
    PetFlagPigPet,              // 330, `:11191`
    PetFlagChesterPet,          // 331, `:11195`
    PetFlagJunimoPet,           // 345, `:11204`
    PetFlagBlueChickenPet,      // 349, `:11208`
    PetFlagSpiffo,              // 351, `:11212`
    PetFlagCaveling,            // 352, `:11216`
    PetFlagDirtiestBlock,       // 354, `:11220`
    PetFlagBoulderPet,          // 373, `:11224`
    PetFlagRainbowBoulderPet,   // 382, `:11228`
    PetFlagDeadCellsSwarmBiter, // 356, `:11232`
    PetFlagPufferfish,          // 371, `:11236`
    PetFlagAxeFairyPet,         // 372, `:11240`
    PetFlagDd2Gato,             // 200, `:11244`
    PetFlagDd2Ghost,            // 201, `:11248`
    PetFlagSugarGlider,         // 218, `:11252`
    SuspiciouslookingTentacle,  // 190, `:11257`
    BlueFairy,                  // 27/101/102, `:11289`
    RedFairy,                   // `:11292`
    GreenFairy,                 // `:11295`
    Bunny,                      // 40, `:11310`
    Penguin,                    // 41, `:11358`
    MagicLantern,               // 152, `:11372`
    Puppy,                      // 91, `:11381`
    Grinch,                     // 92, `:11395`
    BlackCat,                   // 84, `:11409`
    Dino,                       // 61, `:11423`
    BabyFaceMonster,            // 154, `:11437`
    EyeSpring,                  // 65, `:11451`
    Snowman,                    // 66, `:11465`
    Turtle,                     // 42, `:11479`
    Eater,                      // 45, `:11493`
    Skeletron,                  // 50, `:11507`
    Hornet,                     // 51, `:11521`
    Tiki,                       // 52, `:11535`
    Lizard,                     // 53, `:11549`
    Parrot,                     // 54, `:11563`
    Truffle,                    // 55, `:11577`
    Sapling,                    // 56, `:11591`
    CSapling,                   // 85, `:11605`
    Spider,                     // 81, `:11619`
    Squashling,                 // 82, `:11633`
    Wisp,                       // 57, `:11647`
    CrystalLeaf,                // 60, `:11661`
    Zephyrfish,                 // 127, `:11682`
    MiniMinotaur,               // 136, `:11696`
}

/// The simple pet rows: `(buffType, pet projectile, flag)` — every arm whose
/// body is exactly `BuffHandle_SpawnPetIfNeededAndSetTime(j, ref flag, proj)`
/// (`Player.cs:11005-11252`, `:11355-11706`) or the inline copy of that
/// helper (`buffTime[j] = 18000; flag = true; spawn if count == 0`). The
/// helper's default `buffTimeToGive` is 18000 (`:12227`).
pub const PET_ARMS: &[(i32, u16, PetFlag)] = &[
    (19, 18, PetFlag::LightOrb),                    // `Player.cs:11005-11018`
    (155, 500, PetFlag::CrimsonHeart),              // `:11019-11032`
    (191, 653, PetFlag::CompanionCube),             // `:11033-11036`
    (202, 701, PetFlag::PetFlagDd2Dragon),          // `:11037-11040`
    (217, 764, PetFlag::PetFlagUpbeatStar),         // `:11041-11044`
    (219, 774, PetFlag::PetFlagBabyShark),          // `:11045-11048`
    (258, 815, PetFlag::PetFlagLilHarpy),           // `:11049-11052`
    (259, 816, PetFlag::PetFlagFennecFox),          // `:11053-11056`
    (260, 817, PetFlag::PetFlagGlitteryButterfly),  // `:11057-11060`
    (261, 821, PetFlag::PetFlagBabyImp),            // `:11061-11064`
    (262, 825, PetFlag::PetFlagBabyRedPanda),       // `:11065-11068`
    (264, 854, PetFlag::PetFlagPlantero),           // `:11069-11072`
    (266, 858, PetFlag::PetFlagDynamiteKitten),     // `:11073-11076`
    (267, 859, PetFlag::PetFlagBabyWerewolf),       // `:11077-11080`
    (268, 860, PetFlag::PetFlagShadowMimic),        // `:11081-11084`
    (274, 875, PetFlag::PetFlagVoltBunny),          // `:11085-11088`
    (284, 881, PetFlag::PetFlagKingSlimePet),       // `:11089-11092`
    (285, 882, PetFlag::PetFlagEyeOfCthulhuPet),    // `:11093-11096`
    (286, 883, PetFlag::PetFlagEaterOfWorldsPet),   // `:11097-11100`
    (287, 884, PetFlag::PetFlagBrainOfCthulhuPet),  // `:11101-11104`
    (288, 885, PetFlag::PetFlagSkeletronPet),       // `:11105-11108`
    (289, 886, PetFlag::PetFlagQueenBeePet),        // `:11109-11112`
    (290, 887, PetFlag::PetFlagDestroyerPet),       // `:11113-11116`
    (291, 888, PetFlag::PetFlagTwinsPet),           // `:11117-11120`
    (292, 889, PetFlag::PetFlagSkeletronPrimePet),  // `:11121-11124`
    (293, 890, PetFlag::PetFlagPlanteraPet),        // `:11125-11128`
    (294, 891, PetFlag::PetFlagGolemPet),           // `:11129-11132`
    (295, 892, PetFlag::PetFlagDukeFishronPet),     // `:11133-11136`
    (296, 893, PetFlag::PetFlagLunaticCultistPet),  // `:11137-11140`
    (297, 894, PetFlag::PetFlagMoonLordPet),        // `:11141-11144`
    (298, 895, PetFlag::PetFlagFairyQueenPet),      // `:11145-11148`
    (299, 896, PetFlag::PetFlagPumpkingPet),        // `:11149-11152`
    (300, 897, PetFlag::PetFlagEverscreamPet),      // `:11153-11156`
    (301, 898, PetFlag::PetFlagIceQueenPet),        // `:11157-11160`
    (302, 899, PetFlag::PetFlagMartianPet),         // `:11161-11164`
    (303, 900, PetFlag::PetFlagDd2OgrePet),         // `:11165-11168`
    (304, 901, PetFlag::PetFlagDd2BetsyPet),        // `:11169-11172`
    (317, 934, PetFlag::PetFlagQueenSlimePet),      // `:11173-11176`
    (327, 956, PetFlag::PetFlagBerniePet),          // `:11177-11180`
    (328, 957, PetFlag::PetFlagGlommerPet),         // `:11181-11184`
    (329, 958, PetFlag::PetFlagDeerclopsPet),       // `:11185-11188`
    (330, 959, PetFlag::PetFlagPigPet),             // `:11189-11192`
    (331, 960, PetFlag::PetFlagChesterPet),         // `:11193-11196`
    (345, 994, PetFlag::PetFlagJunimoPet),          // `:11202-11205`
    (349, 998, PetFlag::PetFlagBlueChickenPet),     // `:11206-11209`
    (351, 1003, PetFlag::PetFlagSpiffo),            // `:11210-11213`
    (352, 1004, PetFlag::PetFlagCaveling),          // `:11214-11217`
    (354, 1018, PetFlag::PetFlagDirtiestBlock),     // `:11218-11221`
    (373, 1056, PetFlag::PetFlagBoulderPet),        // `:11222-11225`
    (382, 1090, PetFlag::PetFlagRainbowBoulderPet), // `:11226-11229`
    (356, 1027, PetFlag::PetFlagDeadCellsSwarmBiter), // `:11230-11233`
    (371, 1046, PetFlag::PetFlagPufferfish),        // `:11234-11237`
    (372, 1050, PetFlag::PetFlagAxeFairyPet),       // `:11238-11241`
    (200, 703, PetFlag::PetFlagDd2Gato),            // `:11242-11245`
    (201, 702, PetFlag::PetFlagDd2Ghost),           // `:11246-11249`
    (218, 765, PetFlag::PetFlagSugarGlider),        // `:11250-11253`
    (190, 650, PetFlag::SuspiciouslookingTentacle), // `:11254-11267`
    (40, 111, PetFlag::Bunny),                      // `:11307-11320`
    (41, 112, PetFlag::Penguin),                    // `:11355-11368`
    (152, 492, PetFlag::MagicLantern),              // `:11369-11377` (`== 0` gate)
    (91, 334, PetFlag::Puppy),                      // `:11378-11391`
    (92, 353, PetFlag::Grinch),                     // `:11392-11405`
    (84, 319, PetFlag::BlackCat),                   // `:11406-11419`
    (61, 236, PetFlag::Dino),                       // `:11420-11433`
    (154, 499, PetFlag::BabyFaceMonster),           // `:11434-11447`
    (65, 268, PetFlag::EyeSpring),                  // `:11448-11461`
    (66, 269, PetFlag::Snowman),                    // `:11462-11475`
    (42, 127, PetFlag::Turtle),                     // `:11476-11489`
    (45, 175, PetFlag::Eater),                      // `:11490-11503`
    (50, 197, PetFlag::Skeletron),                  // `:11504-11517`
    (51, 198, PetFlag::Hornet),                     // `:11518-11531`
    (52, 199, PetFlag::Tiki),                       // `:11532-11545`
    (53, 200, PetFlag::Lizard),                     // `:11546-11559`
    (54, 208, PetFlag::Parrot),                     // `:11560-11573`
    (55, 209, PetFlag::Truffle),                    // `:11574-11587`
    (56, 210, PetFlag::Sapling),                    // `:11588-11601`
    (85, 324, PetFlag::CSapling),                   // `:11602-11615`
    (81, 313, PetFlag::Spider),                     // `:11616-11629`
    (82, 314, PetFlag::Squashling),                 // `:11630-11643`
    (57, 211, PetFlag::Wisp),                       // `:11644-11657`
    (127, 380, PetFlag::Zephyrfish),                // `:11679-11692`
    (136, 398, PetFlag::MiniMinotaur),              // `:11693-11706`
];

/// One tick's accumulated writes of the mount/pet/misc block. Flags OR,
/// stats ADD per slot exactly like the C# chain; the `Option` setters are
/// assignments (`buffTime[j] = …`, `meleeEnchant = …`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MiscEffects {
    /// `DelBuff(j)` (`:11744`, `:11797`, `:11823`, `:11907`).
    pub delete_buff: bool,
    /// The `j--` rescan after the delete — only Werewolf does it (`:11908`).
    pub rescan_slot: bool,
    /// `buffTime[j] = …` — 18000 (pets), 10 (mounts), 60/6 (Shimmering).
    pub buff_time_set: Option<i32>,
    /// `AddBuff` requests (Rabid's random debuff, `:11331-11346`).
    pub buff_grants: Vec<(i32, i32)>,
    /// The pet flags the arms set (`petBool = true`, `:12235`).
    pub pet_flags: Vec<PetFlag>,
    /// `Projectile.NewProjectile(source, center.X + dx, center.Y, 0, 0,
    /// kind, 0, 0f, whoAmI)` requests (`:12256`) — only when the owned
    /// count is 0. `dx` is the buff-341 ±10·direction offset (`:12244-12252`).
    pub pet_spawns: Vec<(u16, f32)>,
    /// Crystal Leaf (60): kill every owned 226 beyond the first
    /// (`:11663-11673`); the pool walk itself rides the wiring step.
    pub crystal_leaf_dedupe: bool,
    /// `mount.SetMount(type, this)` (`:9883`) — the mount type to re-affirm.
    pub mount_type: Option<i32>,
    // Mount sub-arm flags (`:9885-9912`).
    pub ignore_water: bool,
    pub acc_flipper: bool,
    pub can_float_in_water: bool,
    pub lava_immune: bool,
    pub lava_vision: bool,
    pub fire_walk: bool,
    // DoT / status flags (`:11707-11885`).
    pub venom: bool,                  // 70, `:11709`
    pub chlorophyte_spore: bool,      // 397, `:11713`
    pub poisoned: bool,               // 20, `:11717`
    pub potion_delay: Option<i32>,    // 21, `:11721` `potionDelay = buffTime[j]`
    pub blind: bool,                  // 22, `:11725`
    pub blackout: bool,               // 80, `:11729`
    pub no_items: bool,               // 23, `:11733`
    pub cursed: bool,                 // 23, `:11734`
    pub on_fire: bool,                // 24, `:11738`
    pub dripping: bool,               // 103, `:11747`
    pub dripping_slime: bool,         // 137, `:11754`
    pub dripping_sparkle_slime: bool, // 320, `:11758`
    pub burned: bool,                 // 67, `:11762`
    pub suffocating: bool,            // 68, `:11766`
    pub on_fire2: bool,               // 39, `:11770`
    pub on_fire3: bool,               // 323, `:11774`
    pub on_frost_burn: bool,          // 44, `:11778`
    pub on_frost_burn2: bool,         // 324, `:11782`
    pub shimmering: bool,             // 353, `:11786`
    pub frozen: bool,                 // 353, `:11787`
    pub headcovered: bool,            // 163, `:11828`
    pub bleed: bool,                  // 163, `:11829`
    pub vortex_debuff: bool,          // 164, `:11833`
    pub wind_pushed: bool,            // 194, `:11837`
    pub withered_armor: bool,         // 195, `:11841`
    pub ballista_panic: bool,         // 205, `:11845`
    pub withered_weapon: bool,        // 196, `:11849`
    pub slow_ogre_spit: bool,         // 197, `:11853`
    pub parry_damage_buff: bool,      // 198, `:11857`
    pub moon_leech: bool,             // 145, `:11861`
    pub webbed: bool,                 // 149, `:11865`
    pub defended_by_paladin: bool,    // 43, `:11884`
    pub were_wolf: bool,              // 28, `:11898`
    pub tipsy: bool,                  // 25, `:11920`
    pub well_fed: bool,               // 26/206/207, `:11933`/`:11949`/`:11965`
    pub hungry: bool,                 // 333, `:11981`
    pub starving: bool,               // 334, `:11996`
    pub hearty_meal: bool,            // 336, `:12011`
    pub rabid: bool,                  // 148, `:11323`
    pub melee_enchant: Option<i32>,   // 71/73-79, `:12013-12044`
    /// Wet's per-frame immunities (`buffImmune[24/323/67] = true`,
    /// `:11748-11750`).
    pub immune_on_fire: bool,
    pub immune_hellfire: bool,
    pub immune_burned: bool,
    /// Shimmering's `fallStart = (int)(position.Y / 16f)` (`:11788`).
    pub fall_start_set: Option<i32>,
    /// Webbed's movement lockdown (`:11866-11880`): velocity zero (the
    /// airborne epsilon rides `webbed_vy`), `Player.jumpHeight = 0`,
    /// `gravity = 0`, `moveSpeed = 0`, `dash = 0`, `dashType = 0`,
    /// `noKnockback = true`, `RemoveAllGrapplingHooks()`.
    pub webbed_lockdown: bool,
    pub webbed_vy: f32,
    pub no_knockback: bool,
    pub remove_grappling_hooks: bool,
    // Stat adds (per-slot, summed like the C# chain).
    pub stat_defense: i32,
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    pub melee_damage: f32,
    pub ranged_damage: f32,
    pub magic_damage: f32,
    pub minion_damage: f32,
    pub minion_kb: f32,
    pub melee_speed: f32,
    pub move_speed: f32,
    pub pick_speed: f32,
    pub mana_cost: f32,
    pub stat_mana_max2: i32,
    pub life_regen: i32,
}

/// `Player.BuffHandle_SpawnPetIfNeededAndSetTime` + `…SpawnPetIfNeeded`
/// (`Player.cs:12227-12258`): pin the time, set the flag, spawn at the
/// entity centre when the owned count is 0. The buff-341 centre offset
/// (`:12242-12253`): the Queen Slime pet (934) rides +10·direction, the
/// King Slime pet −10·direction.
fn spawn_pet_if_needed_and_set_time(
    e: &mut MiscEffects,
    ctx: &MiscEffectContext,
    buff_type: i32,
    flag: PetFlag,
    pet_proj_id: u16,
    buff_time_to_give: i32,
) {
    e.buff_time_set = Some(buff_time_to_give); // `:12229`
    e.pet_flags.push(flag); // `:12235`
    let mut dx = 0.0;
    if buff_type == 341 {
        // `:12244-12252`
        let num = 10.0;
        if pet_proj_id == 934 {
            dx = num * ctx.direction as f32;
        } else {
            dx = -(num * ctx.direction as f32);
        }
    }
    // `:12237-12240` + `:12254` — `whoAmI == Main.myPlayer` folds to true.
    if ctx
        .owned_projectile_counts
        .get(usize::from(pet_proj_id))
        .copied()
        .unwrap_or(0)
        == 0
    {
        e.pet_spawns.push((pet_proj_id, dx)); // `:12256`
    }
}

/// The mount arm (`Player.cs:9881-9913`): `BuffID.Sets.MountType[b] != -1`
/// re-affirms the mount and pins `buffTime[j] = 10`; the five water-mount
/// sub-arms add their swim/lava flags. Membership rides the ported
/// `mount_type_of_buff` table (`mount.rs`, the `BuffID.Sets.MountType` port).
fn apply_mount_arm(e: &mut MiscEffects, buff_type: i32) -> bool {
    let Some(m) = crate::mount::mount_type_of_buff(buff_type) else {
        return false;
    };
    e.mount_type = Some(m); // `:9883`
    e.buff_time_set = Some(10); // `:9884`
    match buff_type {
        // 131 (Cute Fishron) `:9885-9889`, 168 `:9890-9894` and 279
        // `:9900-9904` share the ignoreWater + accFlipper pair.
        131 | 168 | 279 => {
            e.ignore_water = true;
            e.acc_flipper = true;
        }
        265 => {
            // `:9895-9899`
            e.can_float_in_water = true;
            e.acc_flipper = true;
        }
        305 => {
            // `:9905-9912` — the Lava Shark mount's full lava suite.
            e.ignore_water = true;
            e.acc_flipper = true;
            e.lava_immune = true;
            e.lava_vision = true;
            e.fire_walk = true;
        }
        _ => {}
    }
    true
}

/// The fairy arm (`Player.cs:11268-11306`): 27/101/102 pick the colour
/// (72 blue / 86 red / 87 green); the `head == 45 && body == 26 && legs == 25`
/// set override forces blue (`:11282-11285`).
fn apply_fairy_arm(e: &mut MiscEffects, ctx: &MiscEffectContext, buff_type: i32) {
    e.buff_time_set = Some(18000); // `:11270`
    let mut num16: u16 = 72; // `:11272`
    match buff_type {
        101 => num16 = 86, // `:11275-11277`
        102 => num16 = 87, // `:11278-11280`
        _ => {}
    }
    if ctx.armor_head == 45 && ctx.armor_body == 26 && ctx.armor_legs == 25 {
        num16 = 72; // `:11282-11285`
    }
    let flag = match num16 {
        86 => PetFlag::RedFairy,   // `:11291-11293`
        87 => PetFlag::GreenFairy, // `:11294-11296`
        _ => PetFlag::BlueFairy,   // `:11288-11290`
    };
    e.pet_flags.push(flag);
    if ctx
        .owned_projectile_counts
        .get(usize::from(num16))
        .copied()
        .unwrap_or(0)
        == 0
    {
        e.pet_spawns.push((num16, 0.0)); // `:11298-11305`
    }
}

/// The Crystal Leaf arm (`Player.cs:11658-11678`): unlike the other pets it
/// walks the pool and kills every owned 226 beyond the first (`:11663-11673`);
/// it spawns only when NONE is owned. The kill walk rides the wiring step
/// (`crystal_leaf_dedupe`); the count gate is exact from the recount.
fn apply_crystal_leaf_arm(e: &mut MiscEffects, ctx: &MiscEffectContext) {
    e.buff_time_set = Some(18000); // `:11660`
    e.pet_flags.push(PetFlag::CrystalLeaf); // `:11661`
    let owned = ctx.owned_projectile_counts.get(226).copied().unwrap_or(0);
    if owned > 1 {
        e.crystal_leaf_dedupe = true; // `:11667-11670`
    }
    if owned == 0 {
        e.pet_spawns.push((226, 0.0)); // `:11674-11677`
    }
}

/// The Rabid arm (`Player.cs:11321-11354`): the 1/1200 random-debuff roll —
/// `Next(1200) == 0` then `Next(6)` picks the debuff and `Next(60,100)*0.01`
/// the seconds scale; the four damage adds always run. RNG order is the C#
/// order verbatim (the two inner draws happen ONLY on the 1/1200 hit).
fn apply_rabid_arm(e: &mut MiscEffects, rng: &mut UnifiedRandom) {
    e.rabid = true; // `:11323`
    if rng.next_max(1200) == 0 {
        // `:11324`
        let num17 = rng.next_max(6); // `:11326`
        let num18 = rng.next_range(60, 100) as f32 * 0.01; // `:11327`
        let (id, mult) = match num17 {
            0 => (22, 3.0),  // `:11330-11332` Darkness
            1 => (23, 0.75), // `:11333-11335` Cursed
            2 => (31, 1.5),  // `:11336-11338` Confused
            3 => (32, 3.5),  // `:11339-11341` Slow
            4 => (33, 5.0),  // `:11342-11344` Weak
            _ => (35, 1.0),  // `:11345-11347` Bleeding
        };
        e.buff_grants.push((id, (60.0 * num18 * mult) as i32));
    }
    e.melee_damage += 0.2; // `:11350`
    e.magic_damage += 0.2; // `:11351`
    e.ranged_damage += 0.2; // `:11352`
    e.minion_damage += 0.2; // `:11353`
}

/// The Shimmering arm (`Player.cs:11784-11825`): the absorbed state —
/// `shimmering + frozen`, `fallStart` reset; below the underworld layer the
/// buff dies, in shimmer it holds 60 ticks, overlapping a solid tile holds
/// 6, otherwise it dies. The `SolidTile3` box scan (`:11807-11816`) is
/// ported verbatim against the caller's tile predicate.
fn apply_shimmering_arm(e: &mut MiscEffects, ctx: &MiscEffectContext) {
    e.shimmering = true; // `:11786`
    e.frozen = true; // `:11787`
    e.fall_start_set = Some((ctx.y / 16.0) as i32); // `:11788`
    if ctx.y / 16.0 > ctx.underworld_layer {
        // `:11793-11800`
        e.delete_buff = true;
        return;
    }
    if ctx.shimmer_wet {
        // `:11801-11805`
        e.buff_time_set = Some(60);
        return;
    }
    let mut flag32 = false; // `:11806`
    let mut num20 = (ctx.x / 16.0) as i32;
    while num20 as f32 <= (ctx.x + ctx.width as f32) / 16.0 {
        let mut num21 = (ctx.y / 16.0) as i32;
        while num21 as f32 <= (ctx.y + ctx.height as f32) / 16.0 {
            if (ctx.solid3)(num20, num21) {
                flag32 = true; // `:11811-11815`
            }
            num21 += 1;
        }
        num20 += 1;
    }
    if flag32 {
        e.buff_time_set = Some(6); // `:11817-11820`
    } else {
        e.delete_buff = true; // `:11821-11824`
    }
}

/// The WellFed tiers (`Player.cs:11931-11978`) — one row per tier, the full
/// stat block including the minion halves and the negative `pickSpeed`
/// (a LOWER pickSpeed mines faster; the delta is kept signed like the C#).
fn apply_well_fed_arm(e: &mut MiscEffects, tier: i32) {
    e.well_fed = true;
    let (def, crit, dmg, kb, speed, pick) = match tier {
        // 26 `:11931-11946`
        26 => (2, 2, 0.05, 0.5, 0.2, -0.05),
        // 206 `:11947-11962`
        206 => (3, 3, 0.075, 0.75, 0.3, -0.1),
        // 207 `:11963-11978`
        _ => (4, 4, 0.1, 1.0, 0.4, -0.15),
    };
    e.stat_defense += def;
    e.melee_crit += crit;
    e.melee_damage += dmg;
    e.melee_speed += dmg;
    e.magic_crit += crit;
    e.magic_damage += dmg;
    e.ranged_crit += crit;
    e.ranged_damage += dmg;
    e.minion_damage += dmg;
    e.minion_kb += kb;
    e.move_speed += speed;
    e.pick_speed += pick;
}

/// The hunger tiers (`Player.cs:11979-12008`): Hungry (333) and Starving
/// (334) — the WellFed block negated (no `moveSpeed` line in the C#).
fn apply_hunger_arm(e: &mut MiscEffects, starving: bool) {
    let (def, crit, dmg, kb, pick) = if starving {
        e.starving = true; // `:11996`
        (4, 4, 0.1, 1.0, 0.15) // `:11997-12007`
    } else {
        e.hungry = true; // `:11981`
        (2, 2, 0.05, 0.5, 0.05) // `:11982-11992`
    };
    e.stat_defense -= def;
    e.melee_crit -= crit;
    e.melee_damage -= dmg;
    e.melee_speed -= dmg;
    e.magic_crit -= crit;
    e.magic_damage -= dmg;
    e.ranged_crit -= crit;
    e.ranged_damage -= dmg;
    e.minion_damage -= dmg;
    e.minion_kb -= kb;
    e.pick_speed += pick;
}

/// One arm of the `UpdateBuffs` mount/pet/misc block. Returns `true` when
/// the id is handled (the caller's `else if` chain stops), `false` for ids
/// outside this wave's range — the dispatch hook the `player.rs` scan wires
/// per active slot, in slot order. `buff_time` is the slot's CURRENT time
/// (after the timer head's decrement, `:9871-9874`); the Werewolf delete
/// sets `rescan_slot` for the C# `j--` (`:11908`).
pub fn apply_misc_effect_arm(
    e: &mut MiscEffects,
    ctx: &MiscEffectContext,
    rng: &mut UnifiedRandom,
    buff_type: i32,
    buff_time: i32,
) -> bool {
    // The mount arm sits FIRST in the C# chain after buff 1 (`:9881`).
    if apply_mount_arm(e, buff_type) {
        return true;
    }
    if let Some(&(_, proj, flag)) = PET_ARMS.iter().find(|&&(b, _, _)| b == buff_type) {
        spawn_pet_if_needed_and_set_time(e, ctx, buff_type, flag, proj, 18000);
        return true;
    }
    match buff_type {
        // The fairy trio (`:11268-11306`).
        27 | 101 | 102 => apply_fairy_arm(e, ctx, buff_type),
        // Crystal Leaf (`:11658-11678`).
        60 => apply_crystal_leaf_arm(e, ctx),
        // The Royal Gel pair pet buff (`:11197-11201`): BOTH pets, the
        // helper runs twice — the second call re-pins 18000.
        341 => {
            spawn_pet_if_needed_and_set_time(e, ctx, 341, PetFlag::PetFlagKingSlimePet, 881, 18000);
            spawn_pet_if_needed_and_set_time(
                e,
                ctx,
                341,
                PetFlag::PetFlagQueenSlimePet,
                934,
                18000,
            );
        }
        // Rabid (`:11321-11354`).
        148 => apply_rabid_arm(e, rng),
        // The DoT/status flag arms (`:11707-11885`).
        70 => e.venom = true,                   // `:11707-11710`
        397 => e.chlorophyte_spore = true,      // `:11711-11714`
        20 => e.poisoned = true,                // `:11715-11718`
        21 => e.potion_delay = Some(buff_time), // `:11719-11722`
        22 => e.blind = true,                   // `:11723-11726`
        80 => e.blackout = true,                // `:11727-11730`
        23 => {
            // `:11731-11735`
            e.no_items = true;
            e.cursed = true;
        }
        24 => e.on_fire = true, // `:11736-11739`
        103 => {
            // Wet (`:11740-11751`): lava deletes it (the C# `continue`
            // skips the flag writes), else dripping + the fire immunities.
            if ctx.lava_wet {
                e.delete_buff = true;
            } else {
                e.dripping = true;
                e.immune_on_fire = true;
                e.immune_hellfire = true;
                e.immune_burned = true;
            }
        }
        137 => e.dripping_slime = true,         // `:11752-11755`
        320 => e.dripping_sparkle_slime = true, // `:11756-11759`
        67 => e.burned = true,                  // `:11760-11763`
        68 => e.suffocating = true,             // `:11764-11767`
        39 => e.on_fire2 = true,                // `:11768-11771`
        323 => e.on_fire3 = true,               // `:11772-11775`
        44 => e.on_frost_burn = true,           // `:11776-11779`
        324 => e.on_frost_burn2 = true,         // `:11780-11783`
        353 => apply_shimmering_arm(e, ctx),    // `:11784-11825`
        163 => {
            // Obstructed (`:11826-11830`) — the C# sets BOTH fields.
            e.headcovered = true;
            e.bleed = true;
        }
        164 => e.vortex_debuff = true,     // `:11831-11834`
        194 => e.wind_pushed = true,       // `:11835-11838`
        195 => e.withered_armor = true,    // `:11839-11842`
        205 => e.ballista_panic = true,    // `:11843-11846`
        196 => e.withered_weapon = true,   // `:11847-11850`
        197 => e.slow_ogre_spit = true,    // `:11851-11854`
        198 => e.parry_damage_buff = true, // `:11855-11858`
        145 => e.moon_leech = true,        // `:11859-11862`
        149 => {
            // Webbed (`:11863-11881`): the full movement lockdown.
            e.webbed = true;
            e.webbed_lockdown = true;
            e.webbed_vy = if ctx.vy != 0.0 { 1e-6 } else { 0.0 }; // `:11866-11873`
            e.no_knockback = true; // `:11879`
            e.remove_grappling_hooks = true; // `:11880`
        }
        43 => e.defended_by_paladin = true, // `:11882-11885`
        29 => {
            // Magic Power (`:11886-11892`).
            e.magic_crit += 2;
            e.magic_damage += 0.05;
            e.stat_mana_max2 += 20;
            e.mana_cost -= 0.02;
        }
        28 => {
            // Werewolf (`:11893-11910`): night + Moon Charm + not merman,
            // else the buff dies AND the slot rescans (`j--`, `:11908`).
            if !ctx.day_time && ctx.wolf_acc && !ctx.merman {
                e.life_regen += 1;
                e.were_wolf = true;
                e.melee_crit += 2;
                e.melee_damage += 0.051;
                e.melee_speed += 0.051;
                e.stat_defense += 3;
                e.move_speed += 0.05;
            } else {
                e.delete_buff = true;
                e.rescan_slot = true;
            }
        }
        33 => {
            // Weak (`:11911-11917`).
            e.melee_damage -= 0.051;
            e.melee_speed -= 0.051;
            e.stat_defense -= 4;
            e.move_speed -= 0.1;
        }
        25 => {
            // Tipsy (`:11918-11930`): the Ale (3821) held adds the ranged
            // pair on top of the melee block.
            e.tipsy = true;
            e.stat_defense -= 4;
            e.melee_crit += 2;
            e.melee_damage += 0.1;
            e.melee_speed += 0.1;
            if ctx.held_item_type == 3821 {
                e.ranged_damage += 0.1;
                e.ranged_crit += 5;
            }
        }
        26 | 206 | 207 => apply_well_fed_arm(e, buff_type), // `:11931-11978`
        333 => apply_hunger_arm(e, false),                  // `:11979-11993`
        334 => apply_hunger_arm(e, true),                   // `:11994-12008`
        336 => e.hearty_meal = true,                        // `:12009-12012`
        // The flasks (`:12013-12044`): meleeEnchant 1..8. NOTE: 72 (Midas)
        // has NO arm here — it is an NPC-side debuff (the flask set's
        // odd member, `BuffID.cs:49`).
        71 => e.melee_enchant = Some(1), // `:12013-12016`
        73 => e.melee_enchant = Some(2), // `:12017-12020`
        74 => e.melee_enchant = Some(3), // `:12021-12024`
        75 => e.melee_enchant = Some(4), // `:12025-12028`
        76 => e.melee_enchant = Some(5), // `:12029-12032`
        77 => e.melee_enchant = Some(6), // `:12033-12036`
        78 => e.melee_enchant = Some(7), // `:12037-12040`
        79 => e.melee_enchant = Some(8), // `:12041-12044`
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(counts: &'a [i32]) -> MiscEffectContext<'a> {
        static SOLID: fn(i32, i32) -> bool = |_, _| false;
        MiscEffectContext {
            owned_projectile_counts: counts,
            day_time: true,
            wolf_acc: false,
            merman: false,
            held_item_type: 0,
            armor_head: 0,
            armor_body: 0,
            armor_legs: 0,
            lava_wet: false,
            shimmer_wet: false,
            x: 160.0,
            y: 320.0,
            width: 20,
            height: 42,
            direction: 1,
            vy: 0.0,
            underworld_layer: 1000.0,
            solid3: &SOLID,
        }
    }

    fn counts() -> Vec<i32> {
        vec![0; crate::projectile::PROJ_COUNT]
    }

    fn run(buff: i32, time: i32, c: &MiscEffectContext) -> MiscEffects {
        let mut e = MiscEffects::default();
        let mut rng = UnifiedRandom::new(1);
        assert!(apply_misc_effect_arm(&mut e, c, &mut rng, buff, time));
        e
    }

    #[test]
    fn every_pet_row_pins_18000_sets_its_flag_and_spawns_once() {
        // `Player.cs:12227-12258` via each row's arm — the flag is set even
        // when the pet already lives; the spawn only when the count is 0.
        let mut c0 = counts();
        for &(buff, proj, flag) in PET_ARMS {
            let e = run(buff, 5, &ctx(&c0));
            assert_eq!(e.buff_time_set, Some(18000), "buff {buff} time pin");
            assert_eq!(e.pet_flags, vec![flag], "buff {buff} flag");
            assert_eq!(e.pet_spawns, vec![(proj, 0.0)], "buff {buff} spawn");
            // Owned already: flag stays, no second spawn (`:12237-12240`).
            c0[usize::from(proj)] = 1;
            let e = run(buff, 5, &ctx(&c0));
            assert_eq!(e.pet_flags, vec![flag]);
            assert!(e.pet_spawns.is_empty(), "buff {buff} no dup spawn");
            c0[usize::from(proj)] = 0;
        }
    }

    #[test]
    fn the_fairy_trio_picks_its_colour_and_the_set_override_forces_blue() {
        // `Player.cs:11268-11306`.
        let c = counts();
        let e = run(27, 5, &ctx(&c));
        assert_eq!(e.pet_flags, vec![PetFlag::BlueFairy]);
        assert_eq!(e.pet_spawns, vec![(72, 0.0)]);
        let e = run(101, 5, &ctx(&c));
        assert_eq!(e.pet_flags, vec![PetFlag::RedFairy]);
        assert_eq!(e.pet_spawns, vec![(86, 0.0)]);
        let e = run(102, 5, &ctx(&c));
        assert_eq!(e.pet_flags, vec![PetFlag::GreenFairy]);
        assert_eq!(e.pet_spawns, vec![(87, 0.0)]);
        // The head 45 / body 26 / legs 25 override (`:11282-11285`).
        let mut cc = ctx(&c);
        cc.armor_head = 45;
        cc.armor_body = 26;
        cc.armor_legs = 25;
        let e = run(102, 5, &cc);
        assert_eq!(e.pet_flags, vec![PetFlag::BlueFairy]);
        assert_eq!(e.pet_spawns, vec![(72, 0.0)]);
    }

    #[test]
    fn the_royal_gel_buff_spawns_both_slime_pets_with_the_341_offsets() {
        // `Player.cs:11197-11201` + the centre offset `:12242-12253`.
        let c = counts();
        let mut cc = ctx(&c);
        cc.direction = -1;
        let e = run(341, 5, &cc);
        assert_eq!(
            e.pet_flags,
            vec![PetFlag::PetFlagKingSlimePet, PetFlag::PetFlagQueenSlimePet]
        );
        // King Slime (881): −10·direction = +10; Queen (934): +10·dir = −10.
        assert_eq!(e.pet_spawns, vec![(881, 10.0), (934, -10.0)]);
        assert_eq!(e.buff_time_set, Some(18000));
    }

    #[test]
    fn crystal_leaf_dedupes_the_extra_leaves_and_spawns_only_at_zero() {
        // `Player.cs:11658-11678`.
        let mut c = counts();
        let e = run(60, 5, &ctx(&c));
        assert_eq!(e.pet_spawns, vec![(226, 0.0)]);
        assert!(!e.crystal_leaf_dedupe);
        c[226] = 1;
        let e = run(60, 5, &ctx(&c));
        assert!(e.pet_spawns.is_empty());
        assert!(!e.crystal_leaf_dedupe);
        c[226] = 3;
        let e = run(60, 5, &ctx(&c));
        assert!(e.pet_spawns.is_empty());
        assert!(e.crystal_leaf_dedupe);
        assert_eq!(e.pet_flags, vec![PetFlag::CrystalLeaf]);
    }

    #[test]
    fn rabid_adds_the_four_damage_rows_and_mirrors_the_rng_order() {
        // `Player.cs:11321-11354`: the adds always run; the debuff roll is
        // Next(1200) → (on 0) Next(6) → Next(60,100), in that order.
        let c = counts();
        let mut e = MiscEffects::default();
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = UnifiedRandom::new(42);
        apply_misc_effect_arm(&mut e, &ctx(&c), &mut rng, 148, 5);
        assert!(e.rabid);
        assert_eq!(e.melee_damage, 0.2);
        assert_eq!(e.magic_damage, 0.2);
        assert_eq!(e.ranged_damage, 0.2);
        assert_eq!(e.minion_damage, 0.2);
        // The mirror consumes the identical stream — the grant (if any)
        // matches the C# formula and the streams stay aligned.
        let hit = mirror.next_max(1200) == 0;
        if hit {
            let which = mirror.next_max(6);
            let scale = mirror.next_range(60, 100) as f32 * 0.01;
            let (id, mult) = match which {
                0 => (22, 3.0),
                1 => (23, 0.75),
                2 => (31, 1.5),
                3 => (32, 3.5),
                4 => (33, 5.0),
                _ => (35, 1.0),
            };
            assert_eq!(e.buff_grants, vec![(id, (60.0 * scale * mult) as i32)]);
        } else {
            assert!(e.buff_grants.is_empty());
        }
        assert_eq!(rng.peek(), mirror.peek(), "RNG streams stay aligned");
    }

    #[test]
    fn rabid_grant_durations_use_the_truncating_cast() {
        // `(int)(60f * num18 * mult)` (`:11331` etc.) — find a seed whose
        // first roll hits and check the exact duration against the mirror.
        let c = counts();
        for seed in 0..200_000 {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(1200) != 0 {
                continue;
            }
            let which = probe.next_max(6);
            let raw = probe.next_range(60, 100);
            let scale = raw as f32 * 0.01;
            let (id, mult) = match which {
                0 => (22, 3.0),
                1 => (23, 0.75),
                2 => (31, 1.5),
                3 => (32, 3.5),
                4 => (33, 5.0),
                _ => (35, 1.0),
            };
            let mut e = MiscEffects::default();
            let mut rng = UnifiedRandom::new(seed);
            apply_misc_effect_arm(&mut e, &ctx(&c), &mut rng, 148, 5);
            assert_eq!(e.buff_grants, vec![(id, (60.0 * scale * mult) as i32)]);
            assert_eq!(rng.peek(), probe.peek());
            return;
        }
        panic!("no 1/1200 hit in 200k seeds — the roll is broken");
    }

    #[test]
    fn wet_drips_and_arms_the_fire_immunities_but_lava_deletes_it() {
        // `Player.cs:11740-11751`.
        let c = counts();
        let e = run(103, 5, &ctx(&c));
        assert!(e.dripping);
        assert!(e.immune_on_fire && e.immune_hellfire && e.immune_burned);
        assert!(!e.delete_buff);
        let mut cc = ctx(&c);
        cc.lava_wet = true;
        let e = run(103, 5, &cc);
        assert!(e.delete_buff);
        assert!(!e.dripping);
        assert!(!e.immune_on_fire);
    }

    #[test]
    fn shimmering_holds_60_in_shimmer_6_on_solid_and_dies_otherwise() {
        // `Player.cs:11784-11825`.
        let c = counts();
        let mut cc = ctx(&c);
        cc.shimmer_wet = true;
        let e = run(353, 5, &cc);
        assert!(e.shimmering && e.frozen);
        assert_eq!(e.fall_start_set, Some(20)); // 320/16
        assert_eq!(e.buff_time_set, Some(60));
        assert!(!e.delete_buff);
        // Out of shimmer, no solid under the box → DelBuff (`:11823`).
        let e = run(353, 5, &ctx(&c));
        assert!(e.delete_buff);
        assert_eq!(e.buff_time_set, None);
        // A solid tile in the box → 6 ticks (`:11817-11820`).
        let solid = |_: i32, _: i32| true;
        let mut cc = ctx(&c);
        cc.solid3 = &solid;
        let e = run(353, 5, &cc);
        assert_eq!(e.buff_time_set, Some(6));
        assert!(!e.delete_buff);
        // Below the underworld layer → dies before any time pin (`:11793`).
        let mut cc = ctx(&c);
        cc.underworld_layer = 10.0;
        let e = run(353, 5, &cc);
        assert!(e.delete_buff);
        assert_eq!(e.buff_time_set, None);
    }

    #[test]
    fn webbed_locks_the_movement_down_with_the_airborne_epsilon() {
        // `Player.cs:11863-11881`.
        let c = counts();
        let e = run(149, 5, &ctx(&c));
        assert!(e.webbed && e.webbed_lockdown);
        assert_eq!(e.webbed_vy, 0.0);
        assert!(e.no_knockback && e.remove_grappling_hooks);
        let mut cc = ctx(&c);
        cc.vy = 3.0;
        let e = run(149, 5, &cc);
        assert_eq!(e.webbed_vy, 1e-6);
    }

    #[test]
    fn werewolf_grants_at_night_with_the_charm_and_dies_otherwise() {
        // `Player.cs:11893-11910`.
        let c = counts();
        let mut cc = ctx(&c);
        cc.day_time = false;
        cc.wolf_acc = true;
        let e = run(28, 5, &cc);
        assert!(e.were_wolf);
        assert_eq!(e.life_regen, 1);
        assert_eq!(e.melee_crit, 2);
        assert_eq!(e.melee_damage, 0.051);
        assert_eq!(e.melee_speed, 0.051);
        assert_eq!(e.stat_defense, 3);
        assert_eq!(e.move_speed, 0.05);
        assert!(!e.delete_buff);
        // Daytime → DelBuff + the j-- rescan (`:11907-11908`).
        let e = run(28, 5, &ctx(&c));
        assert!(e.delete_buff && e.rescan_slot);
        assert!(!e.were_wolf);
        // Merman suppresses it too (`:11895`).
        let mut cc = ctx(&c);
        cc.day_time = false;
        cc.wolf_acc = true;
        cc.merman = true;
        let e = run(28, 5, &cc);
        assert!(e.delete_buff && e.rescan_slot);
    }

    #[test]
    fn magic_power_adds_the_crit_damage_mana_and_cost_rows() {
        // `Player.cs:11886-11892`.
        let c = counts();
        let e = run(29, 5, &ctx(&c));
        assert_eq!(e.magic_crit, 2);
        assert_eq!(e.magic_damage, 0.05);
        assert_eq!(e.stat_mana_max2, 20);
        assert_eq!(e.mana_cost, -0.02);
    }

    #[test]
    fn weak_subtracts_per_slot() {
        // `Player.cs:11911-11917` — two slots stack like the C# chain.
        let c = counts();
        let mut e = MiscEffects::default();
        let mut rng = UnifiedRandom::new(1);
        let cc = ctx(&c);
        apply_misc_effect_arm(&mut e, &cc, &mut rng, 33, 5);
        apply_misc_effect_arm(&mut e, &cc, &mut rng, 33, 5);
        assert_eq!(e.melee_damage, -0.102);
        assert_eq!(e.melee_speed, -0.102);
        assert_eq!(e.stat_defense, -8);
        assert_eq!(e.move_speed, -0.2);
    }

    #[test]
    fn tipsy_adds_the_ranged_pair_only_with_the_ale_held() {
        // `Player.cs:11918-11930` — item 3821.
        let c = counts();
        let e = run(25, 5, &ctx(&c));
        assert!(e.tipsy);
        assert_eq!(e.stat_defense, -4);
        assert_eq!(e.melee_crit, 2);
        assert_eq!(e.melee_damage, 0.1);
        assert_eq!(e.melee_speed, 0.1);
        assert_eq!(e.ranged_damage, 0.0);
        assert_eq!(e.ranged_crit, 0);
        let mut cc = ctx(&c);
        cc.held_item_type = 3821;
        let e = run(25, 5, &cc);
        assert_eq!(e.ranged_damage, 0.1);
        assert_eq!(e.ranged_crit, 5);
    }

    #[test]
    fn the_well_fed_tiers_carry_the_full_stat_block() {
        // `Player.cs:11931-11978`.
        let c = counts();
        let e = run(26, 5, &ctx(&c));
        assert!(e.well_fed);
        assert_eq!(e.stat_defense, 2);
        assert_eq!(e.melee_crit, 2);
        assert_eq!(e.magic_crit, 2);
        assert_eq!(e.ranged_crit, 2);
        assert_eq!(e.melee_damage, 0.05);
        assert_eq!(e.minion_damage, 0.05);
        assert_eq!(e.minion_kb, 0.5);
        assert_eq!(e.move_speed, 0.2);
        assert_eq!(e.pick_speed, -0.05);
        let e = run(206, 5, &ctx(&c));
        assert_eq!(e.stat_defense, 3);
        assert_eq!(e.melee_damage, 0.075);
        assert_eq!(e.minion_kb, 0.75);
        assert_eq!(e.move_speed, 0.3);
        assert_eq!(e.pick_speed, -0.1);
        let e = run(207, 5, &ctx(&c));
        assert_eq!(e.stat_defense, 4);
        assert_eq!(e.melee_damage, 0.1);
        assert_eq!(e.minion_kb, 1.0);
        assert_eq!(e.move_speed, 0.4);
        assert_eq!(e.pick_speed, -0.15);
    }

    #[test]
    fn hungry_and_starving_negate_the_fed_block_without_move_speed() {
        // `Player.cs:11979-12008` — no moveSpeed line in either arm.
        let c = counts();
        let e = run(333, 5, &ctx(&c));
        assert!(e.hungry && !e.starving);
        assert_eq!(e.stat_defense, -2);
        assert_eq!(e.melee_crit, -2);
        assert_eq!(e.melee_damage, -0.05);
        assert_eq!(e.minion_kb, -0.5);
        assert_eq!(e.move_speed, 0.0);
        assert_eq!(e.pick_speed, 0.05);
        let e = run(334, 5, &ctx(&c));
        assert!(e.starving && !e.hungry);
        assert_eq!(e.stat_defense, -4);
        assert_eq!(e.melee_damage, -0.1);
        assert_eq!(e.minion_kb, -1.0);
        assert_eq!(e.pick_speed, 0.15);
        // Hearty Meal is a bare flag (`:12009-12012`).
        let e = run(336, 5, &ctx(&c));
        assert!(e.hearty_meal);
    }

    #[test]
    fn the_flasks_map_to_melee_enchant_1_through_8() {
        // `Player.cs:12013-12044` — 72 (Midas) has no arm.
        let c = counts();
        for (buff, enchant) in [
            (71, 1),
            (73, 2),
            (74, 3),
            (75, 4),
            (76, 5),
            (77, 6),
            (78, 7),
            (79, 8),
        ] {
            let e = run(buff, 5, &ctx(&c));
            assert_eq!(e.melee_enchant, Some(enchant), "buff {buff}");
        }
        let mut e = MiscEffects::default();
        let mut rng = UnifiedRandom::new(1);
        assert!(!apply_misc_effect_arm(&mut e, &ctx(&c), &mut rng, 72, 5));
    }

    #[test]
    fn the_dot_and_event_flag_arms_set_their_flags() {
        // `Player.cs:11707-11885` — one assertion per bare-flag arm.
        let c = counts();
        let rows: &[(i32, fn(&MiscEffects) -> bool)] = &[
            (70, |e| e.venom),
            (397, |e| e.chlorophyte_spore),
            (20, |e| e.poisoned),
            (22, |e| e.blind),
            (80, |e| e.blackout),
            (23, |e| e.no_items && e.cursed),
            (24, |e| e.on_fire),
            (137, |e| e.dripping_slime),
            (320, |e| e.dripping_sparkle_slime),
            (67, |e| e.burned),
            (68, |e| e.suffocating),
            (39, |e| e.on_fire2),
            (323, |e| e.on_fire3),
            (44, |e| e.on_frost_burn),
            (324, |e| e.on_frost_burn2),
            (163, |e| e.headcovered && e.bleed),
            (164, |e| e.vortex_debuff),
            (194, |e| e.wind_pushed),
            (195, |e| e.withered_armor),
            (205, |e| e.ballista_panic),
            (196, |e| e.withered_weapon),
            (197, |e| e.slow_ogre_spit),
            (198, |e| e.parry_damage_buff),
            (145, |e| e.moon_leech),
            (43, |e| e.defended_by_paladin),
        ];
        for &(buff, check) in rows {
            let e = run(buff, 5, &ctx(&c));
            assert!(check(&e), "buff {buff} flag");
        }
        // Potion Sickness copies the slot's remaining time (`:11721`).
        let e = run(21, 1234, &ctx(&c));
        assert_eq!(e.potion_delay, Some(1234));
    }

    #[test]
    fn the_mount_arm_pins_10_and_the_water_mounts_add_their_flags() {
        // `Player.cs:9881-9913`.
        let c = counts();
        // A plain land mount (Rudolph, buff 90) — pin only.
        let e = run(90, 5, &ctx(&c));
        assert_eq!(e.buff_time_set, Some(10));
        assert!(e.mount_type.is_some());
        assert!(!e.acc_flipper);
        // The water sub-arms.
        for buff in [131, 168, 279] {
            let e = run(buff, 5, &ctx(&c));
            assert!(e.ignore_water && e.acc_flipper, "buff {buff}");
            assert!(!e.can_float_in_water);
        }
        let e = run(265, 5, &ctx(&c));
        assert!(e.can_float_in_water && e.acc_flipper);
        assert!(!e.ignore_water);
        let e = run(305, 5, &ctx(&c));
        assert!(
            e.ignore_water && e.acc_flipper && e.lava_immune && e.lava_vision && e.fire_walk,
            "the Lava Shark suite"
        );
    }

    #[test]
    fn ids_outside_the_block_are_not_handled() {
        let c = counts();
        let mut e = MiscEffects::default();
        let mut rng = UnifiedRandom::new(1);
        for buff in [0, 1, 2, 14, 46, 59, 110, 156, 216, 999] {
            assert!(
                !apply_misc_effect_arm(&mut e, &ctx(&c), &mut rng, buff, 5),
                "buff {buff} must fall through"
            );
        }
        assert_eq!(e, MiscEffects::default());
    }
}
