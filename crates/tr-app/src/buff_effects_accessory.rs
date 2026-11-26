//! S1-E-7 wave C — the accessory/set-bonus buff-effect block of
//! `Player.UpdateBuffs` (`Player.cs:10800-11400`) plus the weapon-imbuement
//! arms (`Player.cs:12013-12044`).
//!
//! Disjoint from waves A (`:9875-10048`) and B (`:10049-10300`), which edit
//! the `player.rs` `UpdateBuffs` match: this module owns the arms whose C#
//! `else if` heads start in `10800..11400` and are NOT the summon keep-alive
//! arms already ported by `player.rs::SUMMON_BUFF_ARMS` (`:10483-10972`),
//! plus the flask/imbue arms. The dispatch hook runs at the existing
//! post-tick `UpdateBuffs` seam in `world_sim/player.rs` (right after
//! `update_summon_buff_arms`), so no line of the `player.rs` effect region
//! is touched.
//!
//! Boundary notes (recorded in ANALYSIS):
//! - Titan (108, `:10164`), Panic (63, `:10130`) and Flipper (109, `:10168`)
//!   are accessory-granted but their arms sit in wave B's line range; they
//!   are left to wave B so the effects are not applied twice.
//! - The summon keep-alives inside the block (`:10800-10976`) already landed
//!   with the summon-arms table; this loop leaves those slots alone.
//! - Sunflower (146, `:10999-11004`): the movement half already lives in the
//!   `player.rs` match (`:3816`); this module adds only the missing
//!   `sunflower = true` flag half so the two halves together are 1:1.
//! - Torch God's Favor (376, `:10995-10998` → `TorchGodsFlavor`
//!   `:40185-40235`) is DEFERRED: it needs `BiomeTorchPlaceStyle` and the
//!   aiStyle-1084 torch projectile, neither ported. The arm draws no RNG
//!   (the C# `Main.rand.Next(10)` gate included) so the port's streams are
//!   unchanged until the flavor lands.
//! - The C# `petFlag*`/light-pet bools map onto `Player::pet_flags`, indexed
//!   by the PET PROJECTILE id (the C# flags are per-pet, and buff 341 sets
//!   two of them — the King Slime and Queen Slime pet flags shared with
//!   buffs 284/317). `ResetEffects` clears them (`Player.cs:18413-18600`);
//!   the port clears at the head of this pass (the seam runs after
//!   `ResetEffects`, before any consumer — observably identical).

use crate::npc::{Npc, NpcKind};
use crate::player::{Player, HEIGHT, WIDTH};
use crate::projectile::{PROJ_COUNT, ProjectilePool};
use tr_core::loot::DropRng;

/// The `BuffHandle_SpawnPetIfNeededAndSetTime` rows
/// (`Player.cs:11033-11253`, `:11197-11201`): `(buff id, pet projectile)`.
/// Verbatim order of the C# chain; buff 341 is special-cased (two pets).
const PET_BUFF_ARMS: &[(i32, u16)] = &[
    (191, 653), // :11033 companionCube
    (202, 701), // :11037 petFlagDD2Dragon
    (217, 764), // :11041 petFlagUpbeatStar
    (219, 774), // :11045 petFlagBabyShark
    (258, 815), // :11049 petFlagLilHarpy
    (259, 816), // :11053 petFlagFennecFox
    (260, 817), // :11057 petFlagGlitteryButterfly
    (261, 821), // :11061 petFlagBabyImp
    (262, 825), // :11065 petFlagBabyRedPanda
    (264, 854), // :11069 petFlagPlantero
    (266, 858), // :11073 petFlagDynamiteKitten
    (267, 859), // :11077 petFlagBabyWerewolf
    (268, 860), // :11081 petFlagShadowMimic
    (274, 875), // :11085 petFlagVoltBunny
    (284, 881), // :11089 petFlagKingSlimePet
    (285, 882), // :11093 petFlagEyeOfCthulhuPet
    (286, 883), // :11097 petFlagEaterOfWorldsPet
    (287, 884), // :11101 petFlagBrainOfCthulhuPet
    (288, 885), // :11105 petFlagSkeletronPet
    (289, 886), // :11109 petFlagQueenBeePet
    (290, 887), // :11113 petFlagDestroyerPet
    (291, 888), // :11117 petFlagTwinsPet
    (292, 889), // :11121 petFlagSkeletronPrimePet
    (293, 890), // :11125 petFlagPlanteraPet
    (294, 891), // :11129 petFlagGolemPet
    (295, 892), // :11133 petFlagDukeFishronPet
    (296, 893), // :11137 petFlagLunaticCultistPet
    (297, 894), // :11141 petFlagMoonLordPet
    (298, 895), // :11145 petFlagFairyQueenPet
    (299, 896), // :11149 petFlagPumpkingPet
    (300, 897), // :11153 petFlagEverscreamPet
    (301, 898), // :11157 petFlagIceQueenPet
    (302, 899), // :11161 petFlagMartianPet
    (303, 900), // :11165 petFlagDD2OgrePet
    (304, 901), // :11169 petFlagDD2BetsyPet
    (317, 934), // :11173 petFlagQueenSlimePet
    (327, 956), // :11177 petFlagBerniePet
    (328, 957), // :11181 petFlagGlommerPet
    (329, 958), // :11185 petFlagDeerclopsPet
    (330, 959), // :11189 petFlagPigPet
    (331, 960), // :11193 petFlagChesterPet
    // 341 (:11197) — the dual King+Queen Slime pet arm, special-cased below.
    (345, 994),  // :11202 petFlagJunimoPet
    (349, 998),  // :11206 petFlagBlueChickenPet
    (351, 1003), // :11210 petFlagSpiffo
    (352, 1004), // :11214 petFlagCaveling
    (354, 1018), // :11218 petFlagDirtiestBlock
    (373, 1056), // :11222 petFlagBoulderPet
    (382, 1090), // :11226 petFlagRainbowBoulderPet
    (356, 1027), // :11230 petFlagDeadCellsSwarmBiter
    (371, 1046), // :11234 petFlagPufferfish
    (372, 1050), // :11238 petFlagAxeFairyPet
    (200, 703),  // :11242 petFlagDD2Gato
    (201, 702),  // :11246 petFlagDD2Ghost
    (218, 765),  // :11250 petFlagSugarGlider
];

/// The weapon-imbuement (flask) arms (`Player.cs:12013-12044`):
/// `(buff id, meleeEnchant value)`. The on-hit consumers
/// (`Player.cs:44925-45015`) are not ported yet; the flag is the ported half.
const IMBUE_ARMS: &[(i32, u8)] = &[
    (71, 1), // :12013 WeaponImbueVenom
    (73, 2), // :12017 WeaponImbueCursedFlames
    (74, 3), // :12021 WeaponImbueFire
    (75, 4), // :12025 WeaponImbueGold
    (76, 5), // :12029 WeaponImbueIchor
    (77, 6), // :12033 WeaponImbueNanites
    (78, 7), // :12037 WeaponImbueConfetti
    (79, 8), // :12041 WeaponImbuePoison
];

/// `Player.BuffHandle_SpawnPetIfNeededAndSetTime` +
/// `BuffHandle_SpawnPetIfNeeded` (`Player.cs:12227-12258`): pin the slot at
/// `buffTimeToGive` (18000 for every caller in this block), set the pet flag,
/// and spawn the pet at the player's center when none is owned. The buff-341
/// center offset (`:12242-12253`) shifts ±10 px along `direction`: the Queen
/// Slime pet (934) to the facing side, the King Slime pet (881) against it.
fn spawn_pet_if_needed(p: &mut Player, pool: &mut ProjectilePool, j: usize, proj: u16) {
    p.buffs.buff_time[j] = 18000;
    p.pet_flags[usize::from(proj)] = true;
    if p.owned_projectile_counts[usize::from(proj)] > 0 {
        return;
    }
    // `base.whoAmI == Main.myPlayer` always holds in the single-player port.
    let (mut cx, cy) = (p.x + WIDTH * 0.5, p.y + HEIGHT * 0.5);
    if p.buffs.buff_type[j] == 341 {
        let off = 10.0 * p.facing as f32;
        if proj == 934 {
            cx += off;
        } else {
            cx -= off;
        }
    }
    // `Projectile.NewProjectile(source, center.X, center.Y, 0f, 0f, proj,
    // 0, 0f, whoAmI)` — damage 0, knockback 0, no ai payload. W4-d2: the
    // aiStyle-11 light pets (18/72/86/87) now carry `proj_data` rows, so
    // THEY ride the data-carrying spawn — `NewProjectile` applies
    // SetDefaults and centres the box (`Projectile.cs:10876-10877`). The
    // other pets keep the plain spawn (the rowed style-67 trio's spawn
    // upgrade is the pet-AI task's follow-up — this task's scope is the
    // style-11 arm; recorded in ANALYSIS). The buff pin + flag are the
    // ported half either way.
    if crate::projectile_ai_light_pets::LIGHT_PET_KINDS.contains(&proj) {
        let d = crate::projectile::proj_data(proj).expect("the light-pet row");
    // 0, 0f, whoAmI)` — damage 0, knockback 0, no ai payload. W4-g64 (the
    // d13 seam): the rowed kinds (the aiStyle-124 pets 895/898/957 among
    // them) spawn through the SetDefaults-carrying seam (box/life/flags);
    // the rowless pets keep the plain spawn until their rows land. The C#
    // centres the box on the spawn point (`:10876-10877`).
    // 0, 0f, whoAmI)` — damage 0, knockback 0, no ai payload. W4-d13: the
    // rowed kinds (the aiStyle-124 pets among them) now spawn through the
    // SetDefaults-carrying seam (box/life/flags); the rowless pets keep the
    // plain spawn until their pet-AI task lands (recorded in ANALYSIS).
    // The C# centres the box on the spawn point (`:10876-10877`).
    if let Some(d) = crate::projectile::proj_data(proj) {
        pool.spawn_with_data_ai_owned(
            proj,
            0,
            cx - d.w * 0.5,
            cy - d.h * 0.5,
            0.0,
            0.0,
            0,
            0.0,
            0.0,
            0.0,
        );
        return;
    }
    pool.spawn(
        proj,
        0,
        glam::Vec2::new(cx, cy),
        glam::Vec2::ZERO,
        0,
        0.0,
        false,
    );
    } else {
        pool.spawn(
            proj,
            0,
            glam::Vec2::new(cx, cy),
            glam::Vec2::ZERO,
            0,
            0.0,
            false,
        );
    }
}

/// The wave-C dispatch: one slot-ordered pass over the buff bar with the C#
/// `DelBuff(j); j--` rescan (`Player.cs:10986-10988`), covering the arms whose
/// C# heads start in `10800..11400` (minus the already-ported summon
/// keep-alives) plus the imbue arms. Runs at the `world_sim/player.rs`
/// post-tick seam after `update_summon_buff_arms`; the per-tick flag clears
/// stand in for the `ResetEffects` lines cited per flag.
pub fn update_accessory_buff_arms(
    p: &mut Player,
    pool: &mut ProjectilePool,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
) {
    // `ResetEffects` clears (`Player.cs:18707` gross, `:18708` tongued,
    // `:18486` rabid, `:18758` meleeEnchant, `:18413-18600` the pet/light-pet
    // bools, the sunflower flag beside them). The seam runs after
    // `ResetEffects` and before any consumer, so clearing here is equivalent.
    p.gross = false;
    p.tongued = false;
    p.sunflower = false;
    p.rabid = false;
    p.melee_enchant = 0;
    p.pet_flags = [false; PROJ_COUNT];
    // W4-d9 wiring — `ResetEffects`' `magicLantern = false` (`Player.cs:
    // 18485`): the dedicated lantern bool clears per tick, the 152 arm
    // below re-sets it while the buff stands (the aiStyle-90 keep-alive
    // reads it, `Projectile.cs:32393-32396`).
    p.magic_lantern = false;
    // W4-f12: the `pet_flags` clear moved to the `ResetEffects` region
    // (`player.rs`) so it precedes BOTH the wiring pass's late-pet grants
    // (`buff_frame_wiring.rs`, `:11406-11706`) and this pass's pet grants —
    // clearing it here (after the wiring) would wipe the status pets.

    // `Main.wofNPCIndex >= 0 && npc.active && type == 113`
    // (`Player.cs:10979`): the port's live-WoF scan (hp > 0 is the port's
    // `active` stand-in, `world_sim/npcs.rs:193`).
    let wof_alive = npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind == NpcKind::WallOfFlesh);

    let mut j = 0usize;
    while j < crate::buffs::MAX_BUFFS {
        let buff = p.buffs.buff_type[j];
        // The C# chain guard (`Player.cs:9871-9874`): empty/expired slots skip.
        if buff <= 0 || p.buffs.buff_time[j] <= 0 {
            j += 1;
            continue;
        }
        let mut deleted = false;
        match buff {
            // `Player.cs:10977-10989` — Horrified (37): lives only while the
            // Wall of Flesh is alive; pinned at 10 ticks, else deleted.
            37 => {
                if wof_alive {
                    p.gross = true;
                    p.buffs.buff_time[j] = 10;
                } else {
                    p.buffs.del_buff(j);
                    deleted = true;
                }
            }
            // `Player.cs:10990-10994` — Tongued (38): pinned at 10 ticks.
            38 => {
                p.buffs.buff_time[j] = 10;
                p.tongued = true;
            }
            // `Player.cs:10995-10998` — Torch God's Favor (376): DEFERRED
            // (`TorchGodsFlavor` `:40185-40235` needs BiomeTorchPlaceStyle +
            // the 1084 torch AI). No RNG draw — see the module note.
            376 => {}
            // `Player.cs:10999-11004` — Sunflower (146): the flag half only;
            // the `moveSpeed += 0.1; *= 1.1` half already runs in the
            // `player.rs` match (`:3816`) — do NOT re-apply it here.
            146 => p.sunflower = true,
            // `Player.cs:11005-11018` — Shadow Orb light pet (19 → proj 18).
            19 => spawn_pet_if_needed(p, pool, j, 18),
            // `Player.cs:11019-11032` — Crimson Heart (155 → proj 500).
            155 => spawn_pet_if_needed(p, pool, j, 500),
            // `Player.cs:11254-11267` — Suspicious Looking Tentacle
            // (190 → proj 650).
            190 => spawn_pet_if_needed(p, pool, j, 650),
            // `Player.cs:11268-11306` — the fairies (27/101/102): the pet is
            // 72/86/87 by buff, but the full Red's armor set
            // (`head == 45 && body == 26 && legs == 25`, the texture ids)
            // forces the blue fairy. The port derives the texture ids from
            // the worn armor kinds via the item rows' head/body/leg slots.
            27 | 101 | 102 => {
                p.buffs.buff_time[j] = 18000;
                let mut proj: u16 = match buff {
                    101 => 86,
                    102 => 87,
                    _ => 72,
                };
                let slot_id = |kind: u16, f: fn(&crate::itemdata::ItemDef) -> i16| {
                    crate::registry::get(kind).map_or(-1, f)
                };
                let head = slot_id(p.equipped[0], |d| d.head_slot);
                let body = slot_id(p.equipped[1], |d| d.body_slot);
                let legs = slot_id(p.equipped[2], |d| d.leg_slot);
                if head == 45 && body == 26 && legs == 25 {
                    proj = 72;
                }
                // The fairy bools (`blueFairy`/`redFairy`/`greenFairy`,
                // `:11286-11297`) key on the pet projectile like the rest.
                p.pet_flags[usize::from(proj)] = true;
                if p.owned_projectile_counts[usize::from(proj)] == 0 {
                    let (cx, cy) = (p.x + WIDTH * 0.5, p.y + HEIGHT * 0.5);
                    // W4-d2 — the fairies carry aiStyle-11 rows now: the
                    // data-carrying spawn applies SetDefaults and centres
                    // the box (`Projectile.cs:10876-10877`).
                    let d = crate::projectile::proj_data(proj).expect("the fairy row");
                    pool.spawn_with_data_ai_owned(
                        proj,
                        0,
                        cx - d.w * 0.5,
                        cy - d.h * 0.5,
                        0.0,
                        0.0,
                        0,
                        0.0,
                        0.0,
                        0.0,
                    );
                }
            }
            // `Player.cs:11307-11320` — Bunny pet (40 → proj 111).
            40 => spawn_pet_if_needed(p, pool, j, 111),
            // `Player.cs:11321-11354` — Rabid / Feral Bite (148): the flag,
            // the 1/1200 random-debuff roll and the +0.2 all-class damage.
            148 => {
                p.rabid = true;
                // RNG order mirrored exactly: `Main.rand.Next(1200)`, then
                // ONLY on a hit `Next(6)` and `Next(60, 100)`
                // (`Player.cs:11324-11327`). The stream is the caller's
                // `UpdatePlayers` stage stream (S1-A-10).
                if rng.next_range(0, 1200) == 0 {
                    let which = rng.next_range(0, 6);
                    let num18 = rng.next_range(60, 100) as f32 * 0.01;
                    let (id, mult) = match which {
                        0 => (22, 3.0f32), // :11331 Darkness
                        1 => (23, 0.75),   // :11334 Cursed
                        2 => (31, 1.5),    // :11337 Confused
                        3 => (32, 3.5),    // :11340 Slow
                        4 => (33, 5.0),    // :11343 Weak
                        _ => (35, 1.0),    // :11346 Silenced
                    };
                    // `(int)(60f * num18 * mult)` — f32 product, truncating;
                    // the decimal literals are f32 like the C# (`0.75f` etc).
                    p.buffs.add_buff(id, (60.0f32 * num18 * mult) as i32);
                }
                // `:11350-11353`: +0.2 to all four class damage adds.
                p.melee_damage += 0.2;
                p.magic_damage += 0.2;
                p.ranged_damage += 0.2;
                p.minion_damage += 0.2;
            }
            // `Player.cs:11355-11368` — Penguin pet (41 → proj 112).
            41 => spawn_pet_if_needed(p, pool, j, 112),
            // `Player.cs:11369-11377` — Magic Lantern (152 → proj 492).
            // W4-d9 wiring: the arm sets the DEDICATED `magicLantern` bool
            // (`:11372`), NOT the generic pet flag — the aiStyle-90 arm's
            // keep-alive reads it (`Projectile.cs:32393-32396`). The spawn
            // rides the data-carrying seam (the 492 row landed with W4-d9,
            // so the pool's `proj_data` dispatch runs the lantern AI — the
            // rowless `pool.spawn` the shared handler uses would leave the
            // slot inert). `NewProjectile` centres the box on the player's
            // centre (`:11375`, `Projectile.cs:10876-10877`); the port's
            // spawn seam takes the top-left, so the half-box subtracts.
            152 => {
                p.buffs.buff_time[j] = 18000;
                p.magic_lantern = true;
                if p.owned_projectile_counts[492] == 0 {
                    let (cx, cy) = p.center();
                    let _ = pool.spawn_with_data_ai_owned(
                        492,
                        0,
                        cx - 9.0,
                        cy - 16.0,
                        0.0,
                        0.0,
                        0,
                        0.0,
                        0.0,
                        0.0,
                    );
                }
            }
            // `Player.cs:11378-11391` — Puppy (91 → proj 334).
            91 => spawn_pet_if_needed(p, pool, j, 334),
            // `Player.cs:11392-11405` — Baby Grinch (92 → proj 353).
            92 => spawn_pet_if_needed(p, pool, j, 353),
            // `Player.cs:11197-11201` — the Slime Crown pet duo (341): the
            // King Slime pet (881) then the Queen Slime pet (934), each via
            // the shared handler (both flags set, both times pinned).
            341 => {
                spawn_pet_if_needed(p, pool, j, 881);
                spawn_pet_if_needed(p, pool, j, 934);
            }
            _ => {
                if let Some(&(b, proj)) = PET_BUFF_ARMS.iter().find(|&&(b, _)| b == buff) {
                    let _ = b;
                    spawn_pet_if_needed(p, pool, j, proj);
                } else if let Some(&(_, enchant)) = IMBUE_ARMS.iter().find(|&&(b, _)| b == buff) {
                    // `Player.cs:12013-12044`: `meleeEnchant = N` — the last
                    // matching slot wins, like the C# chain's slot order.
                    p.melee_enchant = enchant;
                }
                // Not a wave-C arm — the chain leaves the slot alone (the
                // summon keep-alives and waves A/B own their ids).
            }
        }
        if !deleted {
            j += 1;
        }
        // else: `DelBuff(j); j--` — rescan the slot the compaction filled.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;
    use tr_core::world::World;

    fn player() -> Player {
        Player::spawn(&World::new("t".into(), 64, 800))
    }

    /// A scripted `DropRng` — the rabid roll mirror test feeds exact values.
    struct ScriptRng(Vec<i32>);
    impl DropRng for ScriptRng {
        fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
            self.0.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    /// A rng that never triggers the 1/1200 roll.
    struct NoTrigger;
    impl DropRng for NoTrigger {
        fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
            1
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    fn run(p: &mut Player, pool: &mut ProjectilePool, npcs: &[Npc], rng: &mut dyn DropRng) {
        update_accessory_buff_arms(p, pool, npcs, rng);
    }

    fn wof() -> Npc {
        let mut n = Npc::new(NpcKind::WallOfFlesh, 0.0, 0.0, 0, 0.0);
        n.hp = n.max_hp;
        n
    }

    #[test]
    fn s1_e_7c_horrified_lives_only_while_the_wof_is_alive() {
        // `Player.cs:10977-10989`.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(37, 60);
        run(&mut p, &mut pool, &[wof()], &mut NoTrigger);
        assert!(p.gross);
        let i = p.buffs.find_buff_index(37).unwrap();
        assert_eq!(p.buffs.buff_time[i], 10, "pinned at 10 while the WoF lives");
        // No WoF → the buff is deleted.
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(!p.gross);
        assert!(p.buffs.find_buff_index(37).is_none());
    }

    #[test]
    fn s1_e_7c_horrified_del_rescans_the_compacted_slot() {
        // Two adjacent Horrified slots, no WoF: the `j--` rescan deletes
        // both in one pass (`Player.cs:10986-10988`).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(37, 60);
        p.buffs.add_buff(37, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(p.buffs.find_buff_index(37).is_none());
    }

    #[test]
    fn s1_e_7c_tongued_pins_ten_and_sets_the_flag() {
        // `Player.cs:10990-10994`.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(38, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(p.tongued);
        let i = p.buffs.find_buff_index(38).unwrap();
        assert_eq!(p.buffs.buff_time[i], 10);
        // Expiry clears the flag on the next pass (ResetEffects stand-in).
        p.buffs.buff_time[i] = 0;
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(!p.tongued);
    }

    #[test]
    fn s1_e_7c_sunflower_sets_the_flag_without_double_moving() {
        // `Player.cs:10999-11004`: the movement half lives in the player.rs
        // match; this pass must only set the flag.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        let before = p.move_speed;
        p.buffs.add_buff(146, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(p.sunflower);
        assert_eq!(p.move_speed, before, "no second moveSpeed application");
    }

    #[test]
    fn s1_e_7c_light_pets_spawn_once_and_pin_18000() {
        // `Player.cs:11005-11032`, `:11254-11267`. W4-d9 wiring: the 152
        // MagicLantern arm left this loop — it sets the DEDICATED
        // `magicLantern` bool (`:11372`), not the generic pet flag; its own
        // witness (`w4_d9_the_magic_lantern_arm_*`) pins it.
        for (buff, proj) in [(19, 18u16), (155, 500), (190, 650)] {
            let mut p = player();
            let mut pool = ProjectilePool::default();
            p.buffs.add_buff(buff, 60);
            run(&mut p, &mut pool, &[], &mut NoTrigger);
            assert!(p.pet_flags[usize::from(proj)], "buff {buff} flag");
            let i = p.buffs.find_buff_index(buff).unwrap();
            assert_eq!(p.buffs.buff_time[i], 18000);
            let spawned = pool
                .slots()
                .iter()
                .filter(|s| s.active && s.kind == proj)
                .count();
            assert_eq!(spawned, 1, "buff {buff} spawns one pet");
            // Recount + second pass: no duplicate while one is owned.
            p.update_projectile_caches(&pool);
            run(&mut p, &mut pool, &[], &mut NoTrigger);
            let spawned = pool
                .slots()
                .iter()
                .filter(|s| s.active && s.kind == proj)
                .count();
            assert_eq!(spawned, 1, "buff {buff} does not double-spawn");
        }
    }

    /// W4-d9 wiring — the buff-152 MagicLantern arm (`Player.cs:11369-
    /// 11377`): the dedicated `magicLantern` bool (`:11372`), the 18000
    /// pin, the data-carrying 492 spawn at the player's centre (the
    /// aiStyle-90 keep-alive reads the bool, `Projectile.cs:32393-32396`).
    /// WRITTEN, NOT RUN (code-only mode).
    #[test]
    fn w4_d9_the_magic_lantern_arm_sets_the_bool_and_spawns_the_row() {
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(152, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(p.magic_lantern, "the dedicated bool, not the pet flag");
        assert!(!p.pet_flags[492], "the C# writes magicLantern, not petFlag");
        let i = p.buffs.find_buff_index(152).unwrap();
        assert_eq!(p.buffs.buff_time[i], 18000);
        let slot = pool.slots().iter().find(|s| s.active && s.kind == 492)
            .expect("the lantern spawned");
        // The data-carrying spawn: the 492 row's box + lifetime ride the
        // slot (the rowless pool.spawn would leave them at the defaults).
        assert_eq!((slot.w, slot.h), (18.0, 32.0), "the 492 row's box");
        assert_eq!(slot.time_left, 18000, "the row's 5x timeLeft");
        // NewProjectile centres the box on the player's centre.
        let (cx, cy) = p.center();
        assert_eq!((slot.pos.x, slot.pos.y), (cx - 9.0, cy - 16.0));
        // Recount + second pass: no duplicate while one is owned.
        p.update_projectile_caches(&pool);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert_eq!(pool.slots().iter().filter(|s| s.active && s.kind == 492).count(), 1);
        assert!(p.magic_lantern, "the bool re-sets each pass");
        // Expiry clears the bool on the next pass (the ResetEffects
        // stand-in, `Player.cs:18485`).
        p.buffs.buff_time[i] = 0;
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(!p.magic_lantern);
    }

    #[test]
    fn s1_e_7c_fairies_pick_the_colour_and_reds_armor_forces_blue() {
        // `Player.cs:11268-11306`.
        for (buff, proj) in [(27, 72u16), (101, 86), (102, 87)] {
            let mut p = player();
            let mut pool = ProjectilePool::default();
            p.buffs.add_buff(buff, 60);
            run(&mut p, &mut pool, &[], &mut NoTrigger);
            assert!(p.pet_flags[usize::from(proj)]);
            assert!(pool.slots().iter().any(|s| s.active && s.kind == proj));
        }
        // The full Red's set (666/667/668 → texture ids 45/26/25) forces 72
        // even for the red/green fairy buffs.
        for buff in [101, 102] {
            let mut p = player();
            let mut pool = ProjectilePool::default();
            p.equipped[0] = 666;
            p.equipped[1] = 667;
            p.equipped[2] = 668;
            p.buffs.add_buff(buff, 60);
            run(&mut p, &mut pool, &[], &mut NoTrigger);
            assert!(p.pet_flags[72], "Red's armor forces the blue fairy");
            assert!(pool.slots().iter().any(|s| s.active && s.kind == 72));
            assert!(!pool.slots().iter().any(|s| s.active && s.kind == 86));
            assert!(!pool.slots().iter().any(|s| s.active && s.kind == 87));
        }
    }

    #[test]
    fn s1_e_7c_rabid_adds_damage_and_rolls_the_debuff_in_c_sharp_order() {
        // `Player.cs:11321-11354`: Next(1200)==0 → Next(6)=3 (Slow) →
        // Next(60,100)=80 → num18=(f32)80*0.01f≈0.80000001 → the f32 product
        // 60f*num18*3.5f lands just UNDER 168 → (int) truncates to 167.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(148, 60);
        let (m0, g0, r0, s0) = (
            p.melee_damage,
            p.magic_damage,
            p.ranged_damage,
            p.minion_damage,
        );
        let mut rng = ScriptRng(vec![0, 3, 80]);
        run(&mut p, &mut pool, &[], &mut rng);
        assert!(p.rabid);
        assert!((p.melee_damage - (m0 + 0.2)).abs() < 1e-6);
        assert!((p.magic_damage - (g0 + 0.2)).abs() < 1e-6);
        assert!((p.ranged_damage - (r0 + 0.2)).abs() < 1e-6);
        assert!((p.minion_damage - (s0 + 0.2)).abs() < 1e-6);
        let i = p.buffs.find_buff_index(32).expect("the Slow debuff");
        assert_eq!(p.buffs.buff_time[i], 167, "the f32 truncation, not 168");
        assert!(rng.0.is_empty(), "exactly three draws, in order");
    }

    #[test]
    fn s1_e_7c_rabid_non_trigger_draws_once_and_adds_no_debuff() {
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(148, 60);
        // Next(1200) != 0 → no further draws (the C# short-circuit).
        let mut rng = ScriptRng(vec![7]);
        run(&mut p, &mut pool, &[], &mut rng);
        assert!(p.rabid);
        assert!(p.buffs.find_buff_index(32).is_none());
        assert!(rng.0.is_empty());
    }

    #[test]
    fn s1_e_7c_rabid_all_six_debuff_cases() {
        // The six switch arms (`Player.cs:11330-11347`), num18 = 0.70.
        for (case, id, mult) in [
            (0, 22, 3.0f32),
            (1, 23, 0.75),
            (2, 31, 1.5),
            (3, 32, 3.5),
            (4, 33, 5.0),
            (5, 35, 1.0),
        ] {
            let mut p = player();
            let mut pool = ProjectilePool::default();
            p.buffs.add_buff(148, 60);
            let mut rng = ScriptRng(vec![0, case, 70]);
            run(&mut p, &mut pool, &[], &mut rng);
            let i = p
                .buffs
                .find_buff_index(id)
                .unwrap_or_else(|| panic!("case {case}"));
            assert_eq!(p.buffs.buff_time[i], (60.0 * 0.70f32 * mult) as i32);
        }
    }

    #[test]
    fn s1_e_7c_pet_table_rows_spawn_their_projectiles() {
        // A representative slice of `PET_BUFF_ARMS` incl. the chain's tail.
        for &(buff, proj) in &[
            (191, 653u16),
            (284, 881),
            (317, 934),
            (331, 960),
            (345, 994),
            (382, 1090),
            (200, 703),
            (218, 765),
            (40, 111),
            (41, 112),
            (91, 334),
            (92, 353),
        ] {
            let mut p = player();
            let mut pool = ProjectilePool::default();
            p.buffs.add_buff(buff, 60);
            run(&mut p, &mut pool, &[], &mut NoTrigger);
            assert!(p.pet_flags[usize::from(proj)], "buff {buff}");
            let i = p.buffs.find_buff_index(buff).unwrap();
            assert_eq!(p.buffs.buff_time[i], 18000);
            assert!(
                pool.slots().iter().any(|s| s.active && s.kind == proj),
                "buff {buff} spawns {proj}"
            );
        }
    }

    #[test]
    fn s1_e_7c_slime_crown_duo_spawns_both_pets_with_the_offset() {
        // `Player.cs:11197-11201` + `:12242-12253`: 881 against the facing,
        // 934 along it, ±10 px; both flags set.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.facing = 1;
        p.buffs.add_buff(341, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert!(p.pet_flags[881] && p.pet_flags[934]);
        let cx = p.x + WIDTH * 0.5;
        let king = pool
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 881)
            .unwrap();
        let queen = pool
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 934)
            .unwrap();
        assert!((king.pos.x - (cx - 10.0)).abs() < 1e-4);
        assert!((queen.pos.x - (cx + 10.0)).abs() < 1e-4);
        // Facing left mirrors the pair.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.facing = -1;
        p.buffs.add_buff(341, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        let cx = p.x + WIDTH * 0.5;
        let king = pool
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 881)
            .unwrap();
        let queen = pool
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 934)
            .unwrap();
        assert!((king.pos.x - (cx + 10.0)).abs() < 1e-4);
        assert!((queen.pos.x - (cx - 10.0)).abs() < 1e-4);
    }

    #[test]
    fn s1_e_7c_weapon_imbues_set_melee_enchant_and_clear_on_expiry() {
        // `Player.cs:12013-12044` + the `ResetEffects` clear `:18758`.
        for &(buff, enchant) in super::IMBUE_ARMS {
            let mut p = player();
            let mut pool = ProjectilePool::default();
            p.buffs.add_buff(buff, 60);
            run(&mut p, &mut pool, &[], &mut NoTrigger);
            assert_eq!(p.melee_enchant, enchant, "buff {buff}");
        }
        // Expiry → the next pass clears the enchant.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(71, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert_eq!(p.melee_enchant, 1);
        let i = p.buffs.find_buff_index(71).unwrap();
        p.buffs.buff_time[i] = 0;
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        assert_eq!(p.melee_enchant, 0);
    }

    #[test]
    fn s1_e_7c_the_pass_leaves_other_waves_arms_alone() {
        // A summon keep-alive (161, `Player.cs:10961`) and a wave-B id (108)
        // must pass through untouched: no time pin, no deletion.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(161, 60);
        p.buffs.add_buff(108, 60);
        run(&mut p, &mut pool, &[], &mut NoTrigger);
        let i = p.buffs.find_buff_index(161).unwrap();
        assert_eq!(p.buffs.buff_time[i], 60, "the summon arm owns 161");
        let i = p.buffs.find_buff_index(108).unwrap();
        assert_eq!(p.buffs.buff_time[i], 60, "wave B owns 108");
    }
}
