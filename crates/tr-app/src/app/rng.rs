//! `Main.SwapRandom`'s named per-stage RNG streams (`Main.cs:3204-3215`).
//!
//! The C# keeps one dictionary on `Main` (`Main.cs:1294-1295`), fills it lazily
//! with `new UnifiedRandom(Main.ActiveWorldFileData.Seed)` per key
//! (`:3210-3212`) and swaps the entry into `Main.rand` for the duration of a
//! `using (Main.SwapRandom("Key"))` block (`:17732-17809`). A world load throws
//! the whole dictionary away (`Main.cs:6465`), so every stream restarts from the
//! new world's seed.
//!
//! The port needs no swap: `Main.rand` is only ever read through the stream of
//! the stage that is running, so the stage asks the registry for its key and
//! passes the `&mut UnifiedRandom` down. Every key is seeded with the same
//! integer the C# passes to `UnifiedRandom`'s constructor — the streams diverge
//! only by how many draws each stage has consumed, exactly like vanilla.

use std::collections::BTreeMap;

use tr_core::UnifiedRandom;

/// The `Main.SwapRandom` keys of the 1.4.5.8 source: the 13 nested stage
/// wrappers of `DoUpdateInWorld_Inner` (`Main.cs:17749-17809`) plus the outer
/// `DoUpdateInWorld` wrapper (`:17732`), the `SpawnNPC` wrapper in
/// `NPC.NewNPC`, `UpdateLiquid` and `UpdateWorldGen`.
///
/// The research note counted 13 (`docs/research/35-rng-utils.md`) — that is the
/// `DoUpdateInWorld_Inner` block alone; the four wrappers outside it bring the
/// total to 17.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RngKey {
    DoUpdateInWorld,
    SpawnNpc,
    SpawnNpcs,
    UpdateDust,
    UpdateFloatingText,
    UpdateGores,
    UpdateItems,
    UpdateLeashedEntities,
    UpdateLiquid,
    UpdateNpcs,
    UpdateParticleSystemsWorld,
    UpdatePlayers,
    UpdatePressurePlates,
    UpdateProjectiles,
    UpdateServerOrClient,
    UpdateTime,
    UpdateWorldGen,
}

impl RngKey {
    /// The exact string literal the C# passes to `Main.SwapRandom`.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            RngKey::DoUpdateInWorld => "DoUpdateInWorld",
            RngKey::SpawnNpc => "SpawnNPC",
            RngKey::SpawnNpcs => "SpawnNPCs",
            RngKey::UpdateDust => "UpdateDust",
            RngKey::UpdateFloatingText => "UpdateFloatingText",
            RngKey::UpdateGores => "UpdateGores",
            RngKey::UpdateItems => "UpdateItems",
            RngKey::UpdateLeashedEntities => "UpdateLeashedEntities",
            RngKey::UpdateLiquid => "UpdateLiquid",
            RngKey::UpdateNpcs => "UpdateNPCs",
            RngKey::UpdateParticleSystemsWorld => "UpdateParticleSystems_World",
            RngKey::UpdatePlayers => "UpdatePlayers",
            RngKey::UpdatePressurePlates => "UpdatePressurePlates",
            RngKey::UpdateProjectiles => "UpdateProjectiles",
            RngKey::UpdateServerOrClient => "UpdateServerOrClient",
            RngKey::UpdateTime => "UpdateTime",
            RngKey::UpdateWorldGen => "UpdateWorldGen",
        }
    }

    /// Every key the decompiled source uses, in source order.
    pub(crate) const ALL: [RngKey; 17] = [
        RngKey::DoUpdateInWorld,
        RngKey::SpawnNpc,
        RngKey::SpawnNpcs,
        RngKey::UpdateDust,
        RngKey::UpdateFloatingText,
        RngKey::UpdateGores,
        RngKey::UpdateItems,
        RngKey::UpdateLeashedEntities,
        RngKey::UpdateLiquid,
        RngKey::UpdateNpcs,
        RngKey::UpdateParticleSystemsWorld,
        RngKey::UpdatePlayers,
        RngKey::UpdatePressurePlates,
        RngKey::UpdateProjectiles,
        RngKey::UpdateServerOrClient,
        RngKey::UpdateTime,
        RngKey::UpdateWorldGen,
    ];
}

/// `Main._rngs` (`Main.cs:1294-1295`): one `UnifiedRandom` per `SwapRandom`
/// key, created on first use and seeded from the world's seed.
///
/// `BTreeMap` rather than the C# `Dictionary` only so iterating the created
/// keys is deterministic (tests/diagnostics); lookup semantics are identical.
pub(crate) struct NamedRngs {
    world_seed: i32,
    streams: BTreeMap<&'static str, UnifiedRandom>,
}

impl NamedRngs {
    /// An empty registry for a world. Streams are created lazily like the C#
    /// (`Main.cs:3210-3212`).
    pub(crate) fn new(world_seed: i32) -> Self {
        Self {
            world_seed,
            streams: BTreeMap::new(),
        }
    }

    /// `Main._rngs = new Dictionary<string, UnifiedRandom>()` (`Main.cs:6465`):
    /// a new world starts every stream from scratch, seeded with its seed.
    ///
    /// The port rebuilds `App` on a world load (`App::swap_world`), so production
    /// takes the fresh-`NamedRngs` path; this is the explicit form of the C#
    /// operation and is unit-tested for a future in-place load.
    #[allow(dead_code)]
    pub(crate) fn reset(&mut self, world_seed: i32) {
        self.world_seed = world_seed;
        self.streams.clear();
    }

    /// `Main.SwapRandom(key)` without the swap: returns the key's stream,
    /// creating it as `new UnifiedRandom(Main.ActiveWorldFileData.Seed)`
    /// (`Main.cs:3210-3212`) on first use.
    pub(crate) fn stream(&mut self, key: RngKey) -> &mut UnifiedRandom {
        let seed = self.world_seed;
        self.streams
            .entry(key.name())
            .or_insert_with(|| UnifiedRandom::new(seed))
    }

    /// Whether the key's stream has been instantiated yet.
    #[allow(dead_code)]
    pub(crate) fn is_created(&self, key: RngKey) -> bool {
        self.streams.contains_key(key.name())
    }

    /// The keys created so far, in name order (diagnostics/tests).
    #[allow(dead_code)]
    pub(crate) fn created(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.streams.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{NamedRngs, RngKey};
    use tr_core::UnifiedRandom;

    /// The literals `Main.SwapRandom` is called with, transcribed from
    /// `decompiled/Terraria/Main.cs:17732-17809` and the three wrappers outside
    /// that block (`NPC.NewNPC`, `UpdateLiquid`, `UpdateWorldGen`).
    const CSHARP_KEYS: [&str; 17] = [
        "DoUpdateInWorld",
        "SpawnNPC",
        "SpawnNPCs",
        "UpdateDust",
        "UpdateFloatingText",
        "UpdateGores",
        "UpdateItems",
        "UpdateLeashedEntities",
        "UpdateLiquid",
        "UpdateNPCs",
        "UpdateParticleSystems_World",
        "UpdatePlayers",
        "UpdatePressurePlates",
        "UpdateProjectiles",
        "UpdateServerOrClient",
        "UpdateTime",
        "UpdateWorldGen",
    ];

    /// The first eight `Next()` values of `new UnifiedRandom(42)`, generated by
    /// running the decompiled `UnifiedRandom.cs` under `dotnet` (the same vector
    /// `tr-core::unified_random`'s `GOLDEN_42` pins). Every registry stream is
    /// seeded with the world seed, so each one must reproduce this prefix.
    const GOLDEN_42: [i32; 8] = [
        1434747710, 302596119, 269548474, 1122627734, 361709742, 563913476, 1555655117, 1101493307,
    ];

    #[test]
    fn the_registry_uses_the_csharp_key_names() {
        let names: Vec<&str> = RngKey::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(names, CSHARP_KEYS, "keys must match Main.cs's literals");
        // No duplicate keys: a duplicate would silently share one stream.
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "duplicate SwapRandom key");
    }

    #[test]
    fn every_stream_is_seeded_with_the_world_seed() {
        let mut rngs = NamedRngs::new(42);
        for key in RngKey::ALL {
            let got: Vec<i32> = (0..GOLDEN_42.len())
                .map(|_| rngs.stream(key).next())
                .collect();
            assert_eq!(got, GOLDEN_42, "{} stream's C# prefix", key.name());
        }
    }

    #[test]
    fn streams_are_independent() {
        const DRAWS: usize = 5;
        let mut rngs = NamedRngs::new(42);
        let items: Vec<i32> = (0..DRAWS)
            .map(|_| rngs.stream(RngKey::UpdateItems).next())
            .collect();
        assert_eq!(items, GOLDEN_42[..DRAWS]);

        // Drawing from UpdateItems must not advance any other stage: each key
        // starts its own `UnifiedRandom` from the same seed (`Main.cs:3212`).
        assert_eq!(rngs.stream(RngKey::UpdateNpcs).next(), GOLDEN_42[0]);
        assert_eq!(rngs.stream(RngKey::UpdateTime).next(), GOLDEN_42[0]);
    }

    #[test]
    fn asking_twice_continues_the_same_stream() {
        let mut rngs = NamedRngs::new(42);
        assert_eq!(rngs.stream(RngKey::UpdateProjectiles).next(), GOLDEN_42[0]);
        // Second call returns the same instance (the C# dictionary entry), not
        // a fresh one.
        assert_eq!(rngs.stream(RngKey::UpdateProjectiles).next(), GOLDEN_42[1]);
    }

    #[test]
    fn keys_are_created_lazily() {
        let mut rngs = NamedRngs::new(1);
        assert_eq!(
            rngs.created().count(),
            0,
            "no stream exists before first use"
        );
        rngs.stream(RngKey::UpdateGores);
        assert!(rngs.is_created(RngKey::UpdateGores));
        assert!(!rngs.is_created(RngKey::UpdateItems));
        assert_eq!(rngs.created().collect::<Vec<_>>(), vec!["UpdateGores"]);
    }

    #[test]
    fn reset_restarts_every_stream_from_the_new_seed() {
        let mut rngs = NamedRngs::new(42);
        rngs.stream(RngKey::UpdateItems).next();
        rngs.stream(RngKey::UpdateNpcs).next();
        rngs.reset(7);

        assert_eq!(rngs.created().count(), 0, "reset clears the dictionary");
        let first = UnifiedRandom::new(7).next();
        assert_eq!(rngs.stream(RngKey::UpdateItems).next(), first);
        assert_eq!(rngs.stream(RngKey::UpdateNpcs).next(), first);
    }
}
