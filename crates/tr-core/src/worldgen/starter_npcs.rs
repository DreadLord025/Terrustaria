//! W4-g38 C5 — the starter town NPCs (`GenPassNameID.SpawnStarterNPCs`,
//! `WorldGen.cs:19685-19897`).
//!
//! The vanilla spawns one town NPC into `Main.npc` at the spawn point during
//! generation: the Guide (netID 22) on a classic world, or the seed's
//! substitute — the Dryad (20) on the infected seed, the Tax Collector (441)
//! on remix, the Mechanic (124) on drunk+don't-starve, the Merchant (17) on
//! not-the-bees, the Demolitionist (38) on for-the-worthy, the Party Girl
//! (208) on drunk. Every spawn is homeless, faces right (`direction = 1`)
//! and keeps the spawn tile as its home anchor.
//!
//! The port has no gen-time `Main.npc` array — the roster records ride
//! [`World::starter_npcs`] (the `save::NpcRecord` shape, the port's internal
//! kind ids from `npc::NpcKind::to_u16`), save with the world's native
//! section, and the App materialises live NPCs from them at load (the
//! `sections.native.npcs` path, `app/mod.rs`'s town-roster load). This
//! replaces the App's old hardcoded Guide-spawn fallback for freshly
//! generated worlds (the fallback stays for pre-v10 saves).
//!
//! Position: `NPC.NewNPC` sets `nPC.Bottom = (spawnTileX * 16, spawnTileY *
//! 16)` (`NPC.cs:80770`); every kind in the single-NPC chain is an 18×40
//! town twin (`NPC.cs` SetDefaults; the port's `npc.rs` kind docs), so the
//! record's top-left position is `(spawnTileX * 16 − 9, spawnTileY * 16 −
//! 40)`.
//!
//! Deferred (documented, flag-gated — a classic world never enters them):
//! * the tenth-anniversary birthday-party arm (`:19688-19822`) — it needs
//!   `BirthdayParty.CelebratingNPCs` (runtime state, unported), the
//!   `GetAdjustedFloorPosition`/`Scan3By3` floor adjust (`:24175-24229`),
//!   and kinds the port has no `NpcKind` for (the Steampunker 178, the
//!   Skeleton Merchant 453, the Town Bunny 656, the town slimes 680/681);
//! * the vampire-seed arm (`:19836-19843`, the Zoologist 633) — the port
//!   has no vampire seed (`wld/sections/flags.rs:251` writes it false);
//! * the `GivenName` roll (`NPC.GiveTownUniqueDataToNPCsThatNeedIt`,
//!   `NPC.cs:80839-80856` → `getNewNPCName` `:7281-7325`,
//!   `Language.RandomFromCategory` on `genRand`) — the worldgen layer has
//!   no loaded language table; the record carries an empty name (the
//!   kind's static name, the port's `Npc::new` default) and the name roll
//!   rides the App's naming seam. No RNG draw stands in for it, so the
//!   pass draws NOTHING on any ported arm;
//! * the infected arm's `townNpcVariationIndex = 1` (`:19830`) — the
//!   port's `NpcRecord` carries no variation index (`save.rs:88-90`);
//! * `NPC.unlockedMerchantSpawn` / `unlockedDemolitionistSpawn`
//!   (`:19869`/`:19878`) — unowned World state (the flags block writes
//!   them false, `wld/sections/flags.rs:226-227`); the owned flags
//!   (`savedTaxCollector`, `savedMech`, `unlockedPartyGirlSpawn`) are set.
//!
//! The pass writes no tiles and draws no RNG, so both world-hash pins are
//! untouched (the roster is not hashed).

use super::GenOptions;
use crate::save::NpcRecord;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::world::World;

/// The port's internal kind ids (`npc::NpcKind::to_u16`,
/// `tr-app/src/npc/data.rs:218-429`) for the chain's kinds. The vanilla
/// netIDs (`NPCID.cs`): Guide 22 (`:10721`), Dryad 20 (`:10717`), Tax
/// Collector 441 (`:11691`), Mechanic 124 (`:10925`), Merchant 17
/// (`:10711`), Demolitionist 38 (`:10753`), Party Girl 208 (`:11093`).
const KIND_GUIDE: u16 = 5;
const KIND_DRYAD: u16 = 35;
const KIND_TAX_COLLECTOR: u16 = 162;
const KIND_MECHANIC: u16 = 158;
const KIND_MERCHANT: u16 = 34;
const KIND_DEMOLITIONIST: u16 = 38;
const KIND_PARTY_GIRL: u16 = 168;

/// One homeless starter at the spawn point — the shared tail every arm
/// runs (`:19709-19712` / `:19826-19829` etc.): `homeTileX/Y = spawn`,
/// `direction = 1`, `homeless = true`. The port models a homeless NPC as
/// carrying no home (the `npc.rs from_wld` convention, `:18677-18685` —
/// the C#'s stale home coordinates are not carried into live state), so
/// the record's home pair is the save's `-1` sentinel.
fn starter_record(world: &World, kind: u16) -> NpcRecord {
    // `NPC.NewNPC`'s `Bottom` anchor (`NPC.cs:80770`) minus the 18×40 town
    // twin's half-width / height.
    NpcRecord {
        kind,
        given_name: String::new(),
        x: world.spawn_x as f32 * 16.0 - 9.0,
        y: world.spawn_y as f32 * 16.0 - 40.0,
        homeless: true,
        home_x: -1,
        home_y: -1,
    }
}

/// The SpawnStarterNPCs pass (`WorldGen.cs:19685-19897`). Writes
/// `world.starter_npcs` and the owned rescue/unlock flags; no tiles, no
/// RNG draws (see the module note).
pub(crate) fn pass_starter_npcs(world: &mut World, opts: &GenOptions) {
    let flags = opts.flags;
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let infected = opts.micro_flags.contains(MicroSeedFlags::WORLD_IS_INFECTED);
    let remix = flags.contains(SeedFlags::REMIX);
    let drunk = flags.contains(SeedFlags::DRUNK);
    let dont_starve = flags.contains(SeedFlags::DONT_STARVE);
    let bees = flags.contains(SeedFlags::NOT_THE_BEES);
    let get_good = flags.contains(SeedFlags::FOR_THE_WORTHY);

    if tenth {
        // `:19688-19822`: the birthday-party arm — deferred (see the
        // module note: BirthdayParty + unported kinds + the floor
        // adjust). No record, no draws.
        return;
    }
    if infected {
        // `:19823-19835`: the Dryad (20), variation 1 (deferred — the
        // record carries no variation index), remix raises
        // savedTaxCollector.
        world.starter_npcs.push(starter_record(world, KIND_DRYAD));
        if remix {
            world.saved_tax_collector = true;
        }
        return;
    }
    // `:19836-19843`: the vampire-seed arm (the Zoologist 633) — deferred,
    // the port has no vampire seed; the chain falls through as if the
    // flag were false (it always is).
    if remix {
        // `:19844-19852`: the Tax Collector (441) + savedTaxCollector.
        world.starter_npcs.push(starter_record(world, KIND_TAX_COLLECTOR));
        world.saved_tax_collector = true;
    } else if drunk && dont_starve {
        // `:19853-19861`: the Mechanic (124) + savedMech.
        world.starter_npcs.push(starter_record(world, KIND_MECHANIC));
        world.saved_mech = true;
    } else if bees && !dont_starve {
        // `:19862-19870`: the Merchant (17) + unlockedMerchantSpawn
        // (unowned World state — see the module note).
        world.starter_npcs.push(starter_record(world, KIND_MERCHANT));
    } else if get_good {
        // `:19871-19879`: the Demolitionist (38) + unlockedDemolitionistSpawn
        // (unowned World state — see the module note).
        world.starter_npcs.push(starter_record(world, KIND_DEMOLITIONIST));
    } else if drunk {
        // `:19880-19888`: the Party Girl (208) + unlockedPartyGirlSpawn.
        world.starter_npcs.push(starter_record(world, KIND_PARTY_GIRL));
        world.unlocked_party_girl = true;
    } else {
        // `:19889-19896`: the classic Guide (22).
        world.starter_npcs.push(starter_record(world, KIND_GUIDE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_at_spawn() -> World {
        let mut w = World::new("starter".into(), 120, 120);
        w.spawn_x = 60;
        w.spawn_y = 40;
        w
    }

    /// W4-g38 C5: a classic world gets one homeless Guide at the spawn
    /// point — the 18×40 Bottom-anchor position, the internal kind id 5.
    #[test]
    fn w4_g38_c5_classic_world_spawns_the_homeless_guide() {
        let mut w = world_at_spawn();
        pass_starter_npcs(&mut w, &GenOptions::default());
        assert_eq!(w.starter_npcs.len(), 1);
        let rec = &w.starter_npcs[0];
        assert_eq!(rec.kind, KIND_GUIDE);
        assert!(rec.homeless);
        assert_eq!(rec.home_x, -1);
        assert_eq!(rec.home_y, -1);
        // Bottom = (60*16, 40*16); top-left = Bottom - (9, 40).
        assert_eq!(rec.x, 60.0 * 16.0 - 9.0);
        assert_eq!(rec.y, 40.0 * 16.0 - 40.0);
        assert!(rec.given_name.is_empty());
    }

    /// W4-g38 C5: the seed substitutions — remix → the Tax Collector +
    /// savedTaxCollector, drunk → the Party Girl + unlockedPartyGirlSpawn,
    /// for-the-worthy → the Demolitionist, drunk+don't-starve → the
    /// Mechanic + savedMech (the chain order: the combo arm wins).
    #[test]
    fn w4_g38_c5_seed_substitutions() {
        let mut w = world_at_spawn();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::REMIX);
        pass_starter_npcs(&mut w, &opts);
        assert_eq!(w.starter_npcs[0].kind, KIND_TAX_COLLECTOR);
        assert!(w.saved_tax_collector);

        let mut w = world_at_spawn();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::DRUNK);
        pass_starter_npcs(&mut w, &opts);
        assert_eq!(w.starter_npcs[0].kind, KIND_PARTY_GIRL);
        assert!(w.unlocked_party_girl);

        let mut w = world_at_spawn();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::FOR_THE_WORTHY);
        pass_starter_npcs(&mut w, &opts);
        assert_eq!(w.starter_npcs[0].kind, KIND_DEMOLITIONIST);

        let mut w = world_at_spawn();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::DRUNK);
        opts.flags.insert(SeedFlags::DONT_STARVE);
        pass_starter_npcs(&mut w, &opts);
        assert_eq!(w.starter_npcs[0].kind, KIND_MECHANIC);
        assert!(w.saved_mech);
        assert!(!w.unlocked_party_girl, "the combo arm wins over drunk");
    }

    /// W4-g38 C5: the infected seed spawns the Dryad; the tenth-anniversary
    /// arm is deferred (no record, no flags).
    #[test]
    fn w4_g38_c5_infected_and_tenth_arms() {
        let mut w = world_at_spawn();
        let mut opts = GenOptions::default();
        opts.micro_flags.insert(MicroSeedFlags::WORLD_IS_INFECTED);
        pass_starter_npcs(&mut w, &opts);
        assert_eq!(w.starter_npcs[0].kind, KIND_DRYAD);

        let mut w = world_at_spawn();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::TENTH_ANNIVERSARY);
        pass_starter_npcs(&mut w, &opts);
        assert!(w.starter_npcs.is_empty(), "the birthday arm is deferred");
    }
}
