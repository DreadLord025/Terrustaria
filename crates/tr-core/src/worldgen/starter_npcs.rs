//! The gen-time town-NPC ledger — the `Guide` pass (`GenPassNameID.
//! SpawnStarterNPCs`, `WorldGen.cs:19685-19897`).
//!
//! Vanilla spawns the world's starter town NPCs with `NPC.NewNPC` inside the
//! pass; the port's `World` is tiles + save state, so the spawns land in a
//! ledger (`World.gen_npcs`) the App materialises at `enter_loaded_world` —
//! the same seam the `.trwld` town roster rides. The records store the
//! VANILLA netID (`NPC.NewNPC`'s `Type` argument); the App decodes through
//! `kind_from_net_id` (the bestiary's `net_id` rows).
//!
//! RNG model: `NPC.NewNPC` → `GiveTownUniqueDataToNPCsThatNeedIt`
//! (`NPC.cs:80839-80856`) → `getNewNPCName` (`:7276-7325`) draws
//! `Language.RandomFromCategory(<Kind>Names, WorldGen.genRand)` — one name
//! roll per town NPC. The pass draws it from its own `pass_rng("Guide")`
//! stream (the per-pass contract), so the insertion shifts no other step's
//! ledger. The `TownNPCProfiles` variation rolls and the shimmered-town
//! overrides are not ported (no profile table); the anniversary arms set
//! their fixed names/variation indices verbatim.
//!
//! Deviations (ANALYSIS "Starter NPCs (W4-d105)"):
//! - `Main.vampireSeed` has no port flag (the vampire seed is not in
//!   `SeedFlags`), so the `:19836-19843` arm never fires.
//! - The `BirthdayParty.CelebratingNPCs` list (`:19728+`) is runtime party
//!   state, not world state — the anniversary spawns land, the party does
//!   not start at generation.
//! - `NPC.unlockedMerchantSpawn`/`unlockedDemolitionistSpawn`/`boughtBunny`
//!   and `unlockedSlimePurpleSpawn` have no `World` field yet; the arms that
//!   set them are recorded here and the flags land with the town-arrival
//!   task. `savedTaxCollector`, `savedMech`, `unlockedPrincessSpawn`,
//!   `unlockedPartyGirlSpawn` and `unlockedSlimeRainbowSpawn` DO have fields
//!   and are set.
//! - The tenth-anniversary first spawn's home is `Main.spawnTileX/Y`
//!   (`:19709-19710`) while every later spawn homes at its own adjusted
//!   floor position — kept verbatim.
//! - The App materialisation decodes the netID through `kind_from_net_id`
//!   (the bestiary rows); the kinds the bestiary does not carry yet (the
//!   Steampunker 178, the white bunny 656, the purple/rainbow slimes
//!   680/681) skip gracefully — the ledger still records them, and they
//!   land when the bestiary gains the rows. The classic Guide 22 and the
//!   seven main seed-arm kinds (17/20/38/124/208/441/663) all decode.

use super::GenOptions;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::world::World;

/// One ledger entry — the `NPC.NewNPC` call's product. `kind` is the vanilla
/// netID; `x`/`y` the pixel position (`NewNPC`'s `Bottom` = the tile anchor
/// × 16); `home_x`/`home_y` the `homeTileX/Y` writes; `direction` the
/// `NPC.direction` write; `given_name_key` non-empty = a fixed name key
/// (the anniversary arms), empty = the kind's rolled name; `variation` the
/// `townNpcVariationIndex` write (−1 = untouched).
#[derive(Clone, Debug)]
pub struct StarterNpc {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub home_x: i32,
    pub home_y: i32,
    pub direction: i32,
    pub given_name_key: &'static str,
    pub variation: i32,
    /// The raw `RandomFromCategory` draw word (`LanguageManager.cs`:
    /// `value[random.Next(value.Count)]`) — the App resolves it through
    /// `lang::random_from_category(category, roll % size)` at
    /// materialisation (tr-core has no localization singleton). Ignored
    /// when `given_name_key` is non-empty (the fixed anniversary names).
    pub name_roll: u32,
}

/// `WorldGen.GetAdjustedFloorPosition` (`WorldGen.cs:24175-24198`) +
/// `Scan3By3` (`:24200-24229`): the 3×3 clear box with a solid floor row,
/// scanned up from `(x-1, y-2)` while blocked (down to `spawnTileY - 10`),
/// then down while floorless (up to `spawnTileY + 10`). Returns the box's
/// bottom-centre `(num + 1, num2 + 2)`.
fn adjusted_floor_position(world: &World, x: i32, y: i32, spawn_y: i32) -> (i32, i32) {
    let mut tx = x - 1;
    let mut ty = y - 2;
    let scan = |world: &World, tlx: i32, tly: i32| -> (bool, bool) {
        // `Scan3By3`: `isEmpty` = no solid cell in the 3×3; `hasFloor` = any
        // solid cell in the row below it. `SolidTile` reads OOB as false.
        let mut is_empty = true;
        for i in 0..3 {
            for j in 0..3 {
                if crate::edit::solid_tile(world, tlx + i, tly + j) {
                    is_empty = false;
                    break;
                }
            }
            if !is_empty {
                break;
            }
        }
        let mut has_floor = false;
        for j in 0..3 {
            if crate::edit::solid_tile(world, tlx + j, tly + 3) {
                has_floor = true;
                break;
            }
        }
        (is_empty, has_floor)
    };
    // The C# loops scan at the CURRENT row, then step only when the scan
    // still fails (`:24181-24196`) — the boundary checks ride the stepped
    // row, so the last scanned row can sit exactly on the ±10 limit.
    let mut is_empty = false;
    let mut has_floor = false;
    while !is_empty && ty > spawn_y - 10 {
        (is_empty, has_floor) = scan(world, tx, ty);
        if !is_empty {
            ty -= 1;
        }
    }
    while !has_floor && ty < spawn_y + 10 {
        (is_empty, has_floor) = scan(world, tx, ty);
        if !has_floor {
            ty += 1;
        }
    }
    tx += 1;
    (tx, ty + 2)
}

/// The `Guide` pass body (`WorldGen.cs:19685-19897`). `spawn_x`/`spawn_y`
/// are the pass-time `Main.spawnTileX/Y` — the port's Spawn Point product
/// (`step_world_tail`'s formula, published early by the adapter).
pub(crate) fn pass_starter_npcs(
    world: &mut World,
    opts: &GenOptions,
    spawn_x: i32,
    spawn_y: i32,
    rng: &mut super::Rng,
) {
    let flags = opts.flags;
    let micro = opts.micro_flags;
    let infected = micro.contains(MicroSeedFlags::WORLD_IS_INFECTED);
    let remix = flags.contains(SeedFlags::REMIX);
    let bees = flags.contains(SeedFlags::NOT_THE_BEES);
    let starve = flags.contains(SeedFlags::DONT_STARVE);
    let drunk = flags.contains(SeedFlags::DRUNK);
    let good = flags.contains(SeedFlags::FOR_THE_WORTHY);
    let skyblock = flags.contains(SeedFlags::SKYBLOCK);
    let anniversary = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    // One name draw per `NPC.NewNPC` of a town kind
    // (`NPC.cs:80844` → `:7285-7322`'s `RandomFromCategory(genRand)`). The
    // raw word rides the ledger; the App resolves it against the category.
    let mut name_draw = |rng: &mut super::Rng| -> u32 { rng.next_u32() };
    let mut push = |world: &mut World, n: StarterNpc| world.gen_npcs.push(n);
    if anniversary {
        // `:19690-19725` — the first spawn at the adjusted floor, the kind
        // by the seed trio, the fixed name + the spawn-point home.
        let (ax, ay) = adjusted_floor_position(world, spawn_x, spawn_y, spawn_y);
        let (kind, name_key, variation) = if infected {
            (20u16, "Lilith", 1)
        } else if !remix {
            if !bees {
                (22, "GuideNames.Andrew", -1)
            } else {
                (228, "SkeletonMerchantNames.Victor", -1)
            }
        } else {
            (441, "GuideNames.Andrew", -1)
        };
        let name_roll = name_draw(rng);
        // `:19695-19698` / `:19707` — the saved flag rides the remix arms
        // only (the infected non-remix Dryad and the classic Guide do not
        // set it).
        if remix {
            world.saved_tax_collector = true;
        }
        push(
            world,
            StarterNpc {
                kind,
                x: (ax * 16) as f32,
                y: (ay * 16) as f32,
                // `:19709-19710` — the FIRST anniversary spawn homes at the
                // spawn point itself, not its adjusted position.
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: name_key,
                variation,
                name_roll,
            },
        );
        if !skyblock {
            // `:19729-19750` — the +2 companion: the Mechanic (Don't Starve,
            // non-remix) or the Steampunker.
            let (ax, ay) = adjusted_floor_position(world, spawn_x + 2, spawn_y, spawn_y);
            let (kind, name_key) = if starve && !remix {
                (124u16, "MechanicNames.Korrie")
            } else {
                (178, "SteampunkerNames.Whitney")
            };
            let name_roll = name_draw(rng);
            push(
                world,
                StarterNpc {
                    kind,
                    x: (ax * 16) as f32,
                    y: (ay * 16) as f32,
                    home_x: ax,
                    home_y: ay,
                    direction: -1,
                    given_name_key: name_key,
                    variation: -1,
                    name_roll,
                },
            );
            // `:19751-19774` — the −2 companion: the Angler (Not the Bees,
            // non-remix) or the Princess.
            let (ax, ay) = adjusted_floor_position(world, spawn_x - 2, spawn_y, spawn_y);
            let (kind, name_key) = if bees && !remix {
                (17u16, "AnglerNames.Charles")
            } else {
                (663, "PrincessNames.Yorai")
            };
            let name_roll = name_draw(rng);
            if bees && !remix {
                // `NPC.unlockedMerchantSpawn = true` (`:19761`) — no World
                // field yet (the module header's deviation).
            } else {
                world.unlocked_princess = true;
            }
            push(
                world,
                StarterNpc {
                    kind,
                    x: (ax * 16) as f32,
                    y: (ay * 16) as f32,
                    home_x: ax,
                    home_y: ay,
                    direction: 1,
                    given_name_key: name_key,
                    variation: -1,
                    name_roll,
                },
            );
            // `:19775-19797` — the +4 companion: the Painter (drunk) or the
            // Party Girl.
            let (ax, ay) = adjusted_floor_position(world, spawn_x + 4, spawn_y, spawn_y);
            let (kind, name_key) = if drunk {
                (227u16, "PainterNames.Jim")
            } else {
                (208, "PartyGirlNames.Amanda")
            };
            let name_roll = name_draw(rng);
            if !drunk {
                world.unlocked_party_girl = true;
            }
            push(
                world,
                StarterNpc {
                    kind,
                    x: (ax * 16) as f32,
                    y: (ay * 16) as f32,
                    home_x: ax,
                    home_y: ay,
                    direction: -1,
                    given_name_key: name_key,
                    variation: -1,
                    name_roll,
                },
            );
            // `:19798-19820` — the −4 pet: the rainbow slime (remix), the
            // purple slime (drunk) or the white bunny.
            let (ax, ay) = adjusted_floor_position(world, spawn_x - 4, spawn_y, spawn_y);
            let (kind, name_key, variation) = if remix {
                world.unlocked_slime_rainbow_spawn = true;
                (681u16, "SlimeNames_Rainbow.Slimestar", -1)
            } else if drunk {
                // `NPC.unlockedSlimePurpleSpawn` (`:19809`) — no World field
                // yet (the module header's deviation).
                (680, "SlimeNames_Purple.Slimefors", -1)
            } else {
                // `NPC.boughtBunny = true` (`:19814`) — no World field yet
                // (the module header's deviation); the variation-1 white
                // bunny still lands.
                (656, "", 1)
            };
            let name_roll = name_draw(rng);
            push(
                world,
                StarterNpc {
                    kind,
                    x: (ax * 16) as f32,
                    y: (ay * 16) as f32,
                    home_x: ax,
                    home_y: ay,
                    direction: 1,
                    given_name_key: name_key,
                    variation,
                    name_roll,
                },
            );
        }
    } else if infected {
        // `:19823-19835` — the Dryad, variation 1, at the spawn point.
        let name_roll = name_draw(rng);
        if remix {
            world.saved_tax_collector = true;
        }
        push(
            world,
            StarterNpc {
                kind: 20,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: 1,
                name_roll,
            },
        );
    } else if remix {
        // `:19844-19852` — the Tax Collector, already saved.
        let name_roll = name_draw(rng);
        world.saved_tax_collector = true;
        push(
            world,
            StarterNpc {
                kind: 441,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: -1,
                name_roll,
            },
        );
    } else if drunk && starve {
        // `:19853-19861` — the Mechanic, already saved.
        let name_roll = name_draw(rng);
        world.saved_mech = true;
        push(
            world,
            StarterNpc {
                kind: 124,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: -1,
                name_roll,
            },
        );
    } else if bees && !starve {
        // `:19862-19870` — the Angler; `NPC.unlockedMerchantSpawn = true`
        // (`:19869`) has no World field yet (the module header's deviation).
        let name_roll = name_draw(rng);
        push(
            world,
            StarterNpc {
                kind: 17,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: -1,
                name_roll,
            },
        );
    } else if good {
        // `:19871-19879` — the Demolitionist; `NPC.unlockedDemolitionistSpawn`
        // (`:19878`) has no World field yet (the module header's deviation).
        let name_roll = name_draw(rng);
        push(
            world,
            StarterNpc {
                kind: 38,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: -1,
                name_roll,
            },
        );
    } else if drunk {
        // `:19880-19888` — the Party Girl, arrival unlocked.
        let name_roll = name_draw(rng);
        world.unlocked_party_girl = true;
        push(
            world,
            StarterNpc {
                kind: 208,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: -1,
                name_roll,
            },
        );
    } else {
        // `:19889-19896` — the classic Guide.
        let name_roll = name_draw(rng);
        push(
            world,
            StarterNpc {
                kind: 22,
                x: (spawn_x * 16) as f32,
                y: (spawn_y * 16) as f32,
                home_x: spawn_x,
                home_y: spawn_y,
                direction: 1,
                given_name_key: "",
                variation: -1,
                name_roll,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;
    use crate::worldgen::noise::Rng;

    /// A flat grass world: solid dirt from row `surface` down, air above.
    fn flat_world(width: i32, height: i32, surface: i32) -> World {
        let mut w = World::new("sn".into(), width, height);
        for x in 0..width {
            for y in surface..height {
                let i = w.index(x, y);
                w.tiles[i].kind = if y == surface { 2 } else { 0 };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    fn opts(bits: u16) -> GenOptions {
        GenOptions {
            flags: SeedFlags::from(bits),
            ..Default::default()
        }
    }

    /// The classic arm (`WorldGen.cs:19889-19896`): one homeless Guide (22)
    /// at the spawn point, direction 1, home = the spawn tile, one name draw.
    #[test]
    fn the_classic_world_spawns_one_homeless_guide() {
        let mut w = flat_world(200, 200, 100);
        let mut rng = Rng::from_seed(7);
        pass_starter_npcs(&mut w, &opts(0), 100, 94, &mut rng);
        assert_eq!(w.gen_npcs.len(), 1);
        let n = &w.gen_npcs[0];
        assert_eq!(n.kind, 22);
        assert_eq!((n.x, n.y), (1600.0, 1504.0));
        assert_eq!((n.home_x, n.home_y), (100, 94));
        assert_eq!(n.direction, 1);
        // Exactly one name draw consumed (`NPC.cs:80844`).
        let mut replay = Rng::from_seed(7);
        let _ = replay.next_u32();
        assert_eq!(rng.next_u32(), replay.next_u32());
    }

    /// The infected arm (`:19823-19835`): the Dryad (20), variation 1.
    #[test]
    fn the_infected_world_spawns_the_dryad() {
        let mut w = flat_world(200, 200, 100);
        let mut o = opts(0);
        o.micro_flags = MicroSeedFlags::from_sum(MicroSeedFlags::WORLD_IS_INFECTED);
        pass_starter_npcs(&mut w, &o, 100, 94, &mut Rng::from_seed(7));
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 20);
        assert_eq!(w.gen_npcs[0].variation, 1);
    }

    /// The remix arm (`:19844-19852`): the Tax Collector (441), saved flag.
    #[test]
    fn the_remix_world_spawns_the_saved_tax_collector() {
        let mut w = flat_world(200, 200, 100);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::REMIX),
            100,
            94,
            &mut Rng::from_seed(7),
        );
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 441);
        assert!(w.saved_tax_collector);
    }

    /// The drunk arm (`:19880-19888`): the Party Girl (208), arrival unlocked.
    #[test]
    fn the_drunk_world_spawns_the_party_girl() {
        let mut w = flat_world(200, 200, 100);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::DRUNK),
            100,
            94,
            &mut Rng::from_seed(7),
        );
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 208);
        assert!(w.unlocked_party_girl);
    }

    /// The anniversary arm (`:19688-19821`): five spawns on a classic
    /// anniversary world — Guide Andrew (home = the spawn point itself),
    /// Steampunker Whitney (+2), Princess Yorai (−2), Party Girl Amanda (+4),
    /// the variation-1 white bunny (−4). Five name draws.
    #[test]
    fn the_anniversary_world_spawns_the_party_of_five() {
        let mut w = flat_world(400, 200, 100);
        let mut rng = Rng::from_seed(7);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::TENTH_ANNIVERSARY),
            200,
            94,
            &mut rng,
        );
        let kinds: Vec<u16> = w.gen_npcs.iter().map(|n| n.kind).collect();
        assert_eq!(kinds, vec![22, 178, 663, 208, 656]);
        assert_eq!(w.gen_npcs[0].given_name_key, "GuideNames.Andrew");
        // The FIRST spawn homes at the spawn point (`:19709-19710`), the
        // companions at their adjusted floor positions.
        assert_eq!((w.gen_npcs[0].home_x, w.gen_npcs[0].home_y), (200, 94));
        assert_eq!(w.gen_npcs[1].given_name_key, "SteampunkerNames.Whitney");
        assert_eq!(w.gen_npcs[1].direction, -1);
        assert_eq!(w.gen_npcs[2].given_name_key, "PrincessNames.Yorai");
        assert!(w.unlocked_princess);
        assert_eq!(w.gen_npcs[3].given_name_key, "PartyGirlNames.Amanda");
        assert!(w.unlocked_party_girl);
        assert_eq!(w.gen_npcs[4].variation, 1);
        // The adjusted floor: the flat world's grass row 100 is the floor;
        // the scan's 3x3 box sits at rows 97..99 and the bottom-centre
        // anchor (`num2 + 2`) lands on row 99 — the last air row.
        for n in &w.gen_npcs[1..] {
            assert_eq!(n.home_y, 99, "the floor anchor sits above the grass row");
        }
        // Five name draws consumed.
        let mut replay = Rng::from_seed(7);
        for _ in 0..5 {
            let _ = replay.next_u32();
        }
        assert_eq!(rng.next_u32(), replay.next_u32());
    }

    /// The anniversary skyblock arm (`:19726`): only the first spawn lands.
    #[test]
    fn the_anniversary_skyblock_spawns_only_the_first() {
        let mut w = flat_world(400, 200, 100);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::TENTH_ANNIVERSARY | SeedFlags::SKYBLOCK),
            200,
            94,
            &mut Rng::from_seed(7),
        );
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 22);
    }

    /// The drunk+starve arm (`:19853-19861`): the saved Mechanic (124).
    #[test]
    fn the_drunk_starve_world_spawns_the_saved_mechanic() {
        let mut w = flat_world(200, 200, 100);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::DRUNK | SeedFlags::DONT_STARVE),
            100,
            94,
            &mut Rng::from_seed(7),
        );
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 124);
        assert!(w.saved_mech);
    }

    /// The not-the-bees arm (`:19862-19870`): the Angler (17).
    #[test]
    fn the_bees_world_spawns_the_angler() {
        let mut w = flat_world(200, 200, 100);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::NOT_THE_BEES),
            100,
            94,
            &mut Rng::from_seed(7),
        );
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 17);
    }

    /// The for-the-worthy arm (`:19871-19879`): the Demolitionist (38).
    #[test]
    fn the_good_world_spawns_the_demolitionist() {
        let mut w = flat_world(200, 200, 100);
        pass_starter_npcs(
            &mut w,
            &opts(SeedFlags::FOR_THE_WORTHY),
            100,
            94,
            &mut Rng::from_seed(7),
        );
        assert_eq!(w.gen_npcs.len(), 1);
        assert_eq!(w.gen_npcs[0].kind, 38);
    }
}
