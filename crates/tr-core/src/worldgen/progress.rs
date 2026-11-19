//! The worldgen progress feed seam (S1-E-17) — the port of the
//! `WorldGenerator` loop's progress bookkeeping (`WorldGenerator.cs:419-515`)
//! over the executed pipeline: `GenerateWorld` sums the enabled passes'
//! weights (`:424`), `RunPass` bookends every pass with
//! `_progress.Start(pass.Weight)` / `_progress.End()` (`:498`/`:507`), and
//! the pass bodies set their own `progress.Message` / `progress.Set`
//! (`WorldGen.cs:11513+`).
//!
//! The port's step adapters draw no RNG for progress (the feed is pure
//! bookkeeping — the world bytes are identical with and without it, pinned
//! by `s1_e_17_the_progress_hook_does_not_change_the_world`
//! (`worldgen/mod.rs`)), so the messages land at
//! the step boundaries with the C#'s per-pass wording. The per-pass
//! `progress.Set` fractions inside the pass bodies are the faithful-pass
//! work (the compact-model steps have no inner loops to report); the
//! boundary feed carries the weight-accurate `TotalProgress` and the
//! pass's message, which is what `UIWorldLoad` draws.
//!
//! Message sources: `Lang.gen[i]` = `LegacyWorldGen.i` (`Lang.cs:475-477`),
//! resolved through the caller's `text` closure (the App's `lang::text`;
//! tr-core stays free of the App's singleton). The step→message table is
//! derived from the `progress.Message = ...` site inside each pass's
//! registration block (`WorldGen.cs` anchors in `VANILLA_CHAIN`).

/// One executed step's progress message — the `progress.Message` assignment
/// inside the pass's `AddGenerationPass` body. `Gen(i)` is `Lang.gen[i]`
/// (`LegacyWorldGen.i`), `Key(k)` is `Language.GetTextValue(k)`.
#[derive(Clone, Copy, Debug)]
pub enum PassMessage {
    Gen(u32),
    Key(&'static str),
}

impl PassMessage {
    /// Resolve through the caller's `Language.GetTextValue` seam.
    pub fn resolve(self, text: &dyn Fn(&str) -> String) -> String {
        match self {
            PassMessage::Gen(i) => text(&format!("LegacyWorldGen.{i}")),
            PassMessage::Key(k) => text(k),
        }
    }
}

/// The per-step message table, keyed by the executed step's display name
/// (`EXECUTED_PIPELINE`'s `name`). Steps absent here carry no
/// `progress.Message` assignment in their vanilla pass body (the message
/// stays whatever the previous pass left — the C# `GenerationProgress`
/// keeps `_message` across passes until a setter runs).
///
/// Citations: the `WorldGen.cs` line of the `progress.Message` site; the
/// class-pass bodies cite their own files. Multi-step vanilla passes
/// (Terrain ×2, Micro Biomes ×2, Shell Piles ×2) share the pass's message —
/// the C# sets it once per pass, the port's split steps re-set the same
/// string (a no-op on the drawn text).
pub fn step_message(step: &str) -> Option<PassMessage> {
    Some(match step {
        // TerrainPass.cs:59 — both terrain steps are the one TerrainPass.
        "terrain height map" | "terrain fill" => PassMessage::Gen(0),
        // WorldGen.cs:11528 (DunesAndPyramidLocations).
        "dunes" => PassMessage::Gen(1),
        // WorldGen.cs:11584.
        "ocean sand" => PassMessage::Key("WorldGeneration.OceanSand"),
        // JunglePass.cs:21.
        "jungle" => PassMessage::Gen(11),
        // WorldGen.cs:12472.
        "mud caves to grass" => PassMessage::Gen(77),
        // WorldGen.cs:12504.
        "full desert" => PassMessage::Gen(78),
        // WorldGen.cs:12548.
        "mushroom patches" => PassMessage::Gen(13),
        // WorldGen.cs:11722 (Tunnels).
        "cave fields" => PassMessage::Key("WorldGeneration.SurfaceTunnels"),
        // WorldGen.cs:12761 (Marble; the granite half shares the pass slot).
        "granite and marble" => PassMessage::Gen(80),
        // WorldGen.cs:11900.
        "stone-in-dirt blobs" => PassMessage::Gen(4),
        // WorldGen.cs:11930.
        "dirt-in-stone blobs" => PassMessage::Gen(5),
        // WorldGen.cs:12166.
        "surface caves" => PassMessage::Gen(10),
        // WorldGen.cs:12275.
        "wavy caves" => PassMessage::Key("WorldGeneration.WavyCaves"),
        // WorldGen.cs:13127.
        "dirt to mud" => PassMessage::Gen(14),
        // WorldGen.cs:13649.
        "ash blobs" => PassMessage::Gen(18),
        // WorldGen.cs:13186.
        "ores and gems" => PassMessage::Gen(16),
        // WorldGen.cs:13945.
        "evil" => PassMessage::Gen(72),
        // WorldGen.cs:14560.
        "liquids" => PassMessage::Gen(19),
        // DungeonCrawler.cs:346 — the pass's first progress message
        // (the later UpdateDungeonProgress rows are the faithful-pass work).
        "dungeon" => PassMessage::Key("WorldGeneration.DungeonVariableSetup"),
        // WorldGen.cs:14899.
        "beaches" => PassMessage::Gen(22),
        // WorldGen.cs:15142.
        "gravitating sand" => PassMessage::Gen(24),
        // WorldGen.cs:15180.
        "ocean caves" => PassMessage::Gen(90),
        // WorldGen.cs:17845 (Jungle Trees).
        "underground jungle trees" => PassMessage::Gen(83),
        // WorldGen.cs:17215.
        "chests" => PassMessage::Gen(32),
        // WorldGen.cs:17361.
        "spider caves" => PassMessage::Gen(64),
        // WorldGen.cs:17432 (the same Lang.gen[64] row).
        "gem caves" => PassMessage::Gen(64),
        // WorldGen.cs:17476.
        "moss" => PassMessage::Gen(61),
        // WorldGen.cs:15755.
        "altars" => PassMessage::Gen(26),
        // WorldGen.cs:18000.
        "pots" => PassMessage::Gen(35),
        // WorldGen.cs:15931.
        "beehives" => PassMessage::Gen(71),
        // WorldGen.cs:15839.
        "lihzahrd temple" => PassMessage::Gen(70),
        // WorldGen.cs:16980.
        "cave houses" => PassMessage::Gen(30),
        // WorldGen.cs:20910 (Micro Biomes — the grouped steps).
        "campsites" | "sword shrines" | "mahogany trees" | "thin ice" => PassMessage::Gen(76),
        // WorldGen.cs:16309 (ShellPilesMarblePilesAndSpikePits — the
        // classic-world arm; the MarblePiles/SpikeCaves messages are the
        // Don't Starve / no-traps arms, not the classic feed).
        "spike pits" | "shell piles" => PassMessage::Key("WorldGeneration.ShellPiles"),
        // WorldGen.cs:20197.
        "vines" => PassMessage::Gen(43),
        // WorldGen.cs:12933.
        "floating islands" => PassMessage::Gen(12),
        // WorldGen.cs:16421.
        "slopes" => PassMessage::Gen(60),
        // WorldGen.cs:15480.
        "living trees" => PassMessage::Key("WorldGeneration.LivingTrees"),
        // WorldGen.cs:19944.
        "trees" => PassMessage::Gen(40),
        // WorldGen.cs:20068.
        "surface flora" => PassMessage::Gen(42),
        // WorldGen.cs:21314.
        "cacti" => PassMessage::Gen(38),
        // WorldGen.cs:21977.
        "water plants" => PassMessage::Gen(88),
        // WorldGen.cs:22489 — the GenerateWorld tail's own message
        // (the port's world-tail step is that tail's stand-in).
        "world tail" => PassMessage::Gen(87),
        _ => return None,
    })
}
