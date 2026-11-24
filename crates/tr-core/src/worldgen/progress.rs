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
        // WorldGen.cs:11693 (Sand Patches, W4-d101).
        "sand patches" => PassMessage::Key("WorldGeneration.SandPatches"),
        // WorldGen.cs:11789 (Mount Caves, W4-d101 — Lang.gen[2]).
        "mount caves" => PassMessage::Gen(2),
        // WorldGen.cs:11862 (Dirt Wall Backgrounds, W4-d101 — Lang.gen[3]).
        "dirt wall backgrounds" => PassMessage::Gen(3),
        // WorldGen.cs:11963 (Clay, W4-d101 — Lang.gen[6]).
        "clay" => PassMessage::Gen(6),
        // WorldGen.cs:12012 (Small Holes, W4-d101 — Lang.gen[7]).
        "small holes" => PassMessage::Gen(7),
        // WorldGen.cs:12071 (Dirt Layer Caves, W4-d101 — Lang.gen[8]).
        "dirt layer caves" => PassMessage::Gen(8),
        // WorldGen.cs:12111 (Rock Layer Caves, W4-d101 — Lang.gen[9]).
        "rock layer caves" => PassMessage::Gen(9),
        // WorldGen.cs:12316 (Generate Ice Biome, W4-d101 — Lang.gen[56]).
        "generate ice biome" => PassMessage::Gen(56),
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
        // WorldGen.cs:12761 (Marble — the W4-d101 pass-9 split row).
        "marble" => PassMessage::Gen(80),
        // WorldGen.cs:12843 (Granite — Lang.gen[81], the pass-9 split row).
        "granite" => PassMessage::Gen(81),
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
        // WorldGen.cs:13147 (W4-d102).
        "silt" => PassMessage::Gen(15),
        // WorldGen.cs:13649.
        "ash blobs" => PassMessage::Gen(18),
        // WorldGen.cs:13186.
        "ores and gems" => PassMessage::Gen(16),
        // WorldGen.cs:13611 (W4-d102).
        "webs" => PassMessage::Gen(17),
        // WorldGen.cs:13945.
        "evil" => PassMessage::Gen(72),
        // WorldGen.cs:14560.
        "liquids" => PassMessage::Gen(19),
        // DitherSnakePass.cs:24 — the dual-dungeons snake pass's message.
        "Dual Dungeons Dither Snake" => PassMessage::Key("WorldGeneration.DualDungeonsDitherSnake"),
        // DungeonCrawler.cs:346 — the pass's first progress message
        // (the later UpdateDungeonProgress rows are the faithful-pass work).
        "dungeon" => PassMessage::Key("WorldGeneration.DungeonVariableSetup"),
        // DitherSnakePass.cs:24 — the gated dual-dungeon snake's message.
        "dither snake" => PassMessage::Key("WorldGeneration.DualDungeonsDitherSnake"),
        // WorldGen.cs:22513 — the gated skyblock generator's message.
        "skyblock" => PassMessage::Gen(12),
        // WorldGen.cs:14877 (W4-d102).
        "mountain caves" => PassMessage::Gen(21),
        // WorldGen.cs:14899.
        "beaches" => PassMessage::Gen(22),
        // WorldGen.cs:15142.
        "gravitating sand" => PassMessage::Gen(24),
        // WorldGen.cs:15054 (W4-d102).
        "gems" => PassMessage::Gen(23),
        // WorldGen.cs:15256 (W4-d102).
        "clean up dirt" => PassMessage::Gen(25),
        // WorldGen.cs:15180.
        "ocean caves" => PassMessage::Gen(90),
        // WorldGen.cs:15256 (W4-g36 A2).
        "clean up dirt" => PassMessage::Gen(25),
        // WorldGen.cs:17845 (Jungle Trees).
        "underground jungle trees" => PassMessage::Gen(83),
        // WorldGen.cs:17215.
        "chests" => PassMessage::Gen(32),
        // WorldGen.cs:17249 (UnderwaterChests).
        "water chests" => PassMessage::Gen(33),
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
        // W4-g37 B1/B2/B3/B4/B5 KEEP-ONE (matrix-88 §2.7-2.11): the statues /
        // surface chests / water chests / cave walls / floating island houses
        // arms are d103/g44-owned (the d103 arms sit below) — g37's duplicate
        // rows dropped. B6/B7/B8/B9 (quick cleanup / hellforge / spreading
        // grass / surface ore and stone) are d104-owned, unmerged at this gate;
        // the arms stay for the loading feed when d104 lands.
        // WorldGen.cs:17879 (W4-g37 B6).
        "quick cleanup" => PassMessage::Key("WorldGeneration.QuickCleanup"),
        // WorldGen.cs:18191 (W4-g37 B7).
        "hellforge" => PassMessage::Gen(36),
        // WorldGen.cs:18241 (W4-g37 B8).
        "spreading grass" => PassMessage::Gen(37),
        // WorldGen.cs:18447 (W4-g37 B9 — the pass sets progress.Set(1.0) with
        // no message; the slot keeps the prior step's message in vanilla, so
        // the port maps it to the same Gen(37) row for the loading feed).
        "surface ore and stone" => PassMessage::Gen(37),
        // WorldGen.cs:18524 (FallenLogsAndWaterFeatures - Lang.gen[85]).
        "fallen log" => PassMessage::Gen(85),
        // WorldGen.cs:18663 (Traps - Lang.gen[34]; the no-traps seed's
        // Lang.gen[91] override is a seed arm, the classic feed is [34]).
        "traps" => PassMessage::Gen(34),
        // WorldGen.cs:18792 (Piles - Lang.gen[89]).
        "piles" => PassMessage::Gen(89),
        // WorldGen.cs:19902 (SunflowersPart2 - Lang.gen[39]).
        "sunflowers" => PassMessage::Gen(39),
        // WorldGen.cs:19973 (AlchemyHerbs - Lang.gen[41]).
        "herbs" => PassMessage::Gen(41),
        // WorldGen.cs:20910 (Micro Biomes — the grouped steps).
        "campsites" | "sword shrines" | "mahogany trees" | "mining explosives" | "thin ice"
        | "dead man's chests" => PassMessage::Gen(76),
        // WorldGen.cs:16309 (ShellPilesMarblePilesAndSpikePits — the
        // classic-world arm; the MarblePiles/SpikeCaves messages are the
        // Don't Starve / no-traps arms, not the classic feed).
        "spike pits" | "shell piles" => PassMessage::Key("WorldGeneration.ShellPiles"),
        // WorldGen.cs:16247 (Oasis).
        "oasis" => PassMessage::Key("WorldGeneration.Oasis"),
        // WorldGen.cs:20197.
        "vines" => PassMessage::Gen(43),
        // WorldGen.cs:20451 (Flowers).
        "flowers" => PassMessage::Gen(44),
        // WorldGen.cs:20603 (Mushrooms).
        "mushrooms" => PassMessage::Gen(45),
        // WorldGen.cs:20773 (LongMoss).
        "moss grass" => PassMessage::Key("WorldGeneration.LongMoss"),
        // WorldGen.cs:20868-20900 (BeeLarvaInBeehives) sets no
        // progress.Message — only progress.Set(1.0) — so the step carries
        // none (the C# keeps the previous pass's message).
        // WorldGen.cs:20729-20768 (ExposedGemsUnderground) sets no
        // progress.Message — only progress.Set(1.0) — so the step carries
        // none (the C# keeps the previous pass's message).
        // WorldGen.cs:12933.
        "floating islands" => PassMessage::Gen(12),
        // WorldGen.cs:16136 (Settle Liquids, W4-f25) — `Lang.gen[27]`.
        "settle liquids" => PassMessage::Gen(27),
        // WorldGen.cs:17863 (the FloatingIslandHouses pass's message).
        "floating island houses" => PassMessage::Key("WorldGeneration.FloatingIslandHouses"),
        // WorldGen.cs:16421.
        "slopes" => PassMessage::Gen(60),
        // WorldGen.cs:16614 (Waterfalls).
        "waterfalls" => PassMessage::Gen(69),
        // WorldGen.cs:16701 (Wall Variety).
        "wall variety" => PassMessage::Gen(79),
        // WorldGen.cs:16771 (Life Crystals).
        "life crystals" => PassMessage::Gen(28),
        // WorldGen.cs:16857 (Statues).
        "statues" => PassMessage::Gen(29),
        // WorldGen.cs:17134 (Surface Chests).
        "surface chests" => PassMessage::Gen(31),
        // WorldGen.cs:17249 (Water Chests / UnderwaterChests).
        "water chests" => PassMessage::Gen(33),
        // WorldGen.cs:17718 (Cave Walls / CaveWallsInEnclosedSpaces).
        "cave walls" => PassMessage::Gen(63),
        // WorldGen.cs:15480.
        "living trees" => PassMessage::Key("WorldGeneration.LivingTrees"),
        // WorldGen.cs:15721 (W4-g36 A4 — the same LivingTrees message).
        "wood tree walls" => PassMessage::Key("WorldGeneration.LivingTrees"),
        // WorldGen.cs:17863 (Floating Island Houses, W4-d104).
        "floating island houses" => PassMessage::Key("WorldGeneration.FloatingIslandHouses"),
        // WorldGen.cs:17879 (Quick Cleanup, W4-d104).
        "quick cleanup" => PassMessage::Key("WorldGeneration.QuickCleanup"),
        // WorldGen.cs:18191 — Lang.gen[36] (Hellforge, W4-d104).
        "hellforge" => PassMessage::Gen(36),
        // WorldGen.cs:18241 — Lang.gen[37] (Spreading Grass, W4-d104).
        "spreading grass" => PassMessage::Gen(37),
        // WorldGen.cs:18451 — the pass sets only progress.Set(1.0), no
        // message (Surface Ore and Stone, W4-d104): absent from the table.
        // WorldGen.cs:18524 — Lang.gen[85] (Place Fallen Log, W4-d104).
        "place fallen log" => PassMessage::Gen(85),
        // WorldGen.cs:18663 — Lang.gen[34] (Traps, W4-d104; the noTraps
        // Lang.gen[91] arm is the seed lane's).
        "traps" => PassMessage::Gen(34),
        // WorldGen.cs:18792 — Lang.gen[89] (Piles, W4-d104).
        "piles" => PassMessage::Gen(89),
        // WorldGen.cs:19460 — the Spawn Point pass sets only
        // progress.Set(1.0), no message (W4-d104): absent from the table.
        // WorldGen.cs:15721 (W4-d102) — the same key as Living Trees.
        "wood tree walls" => PassMessage::Key("WorldGeneration.LivingTrees"),
        // WorldGen.cs:19944.
        "trees" => PassMessage::Gen(40),
        // WorldGen.cs:19902 (SunflowersPart2).
        "sunflowers" => PassMessage::Gen(39),
        // WorldGen.cs:19973 (AlchemyHerbs).
        "herbs" => PassMessage::Gen(41),
        // WorldGen.cs:19981-19994 (DyePlants) sets no progress.Message —
        // only progress.Set(1.0) — so the step carries none (the C# keeps
        // the previous pass's message).
        // WorldGen.cs:20068.
        "surface flora" => PassMessage::Gen(42),
        // WorldGen.cs:21254 — `Lang.gen[27]` (the settle message; the
        // extraLiquid seed overrides it to "WorldGeneration.WaterFeatures",
        // `:21257` — the classic feed is Gen(27)).
        "settle liquids again" => PassMessage::Gen(27),
        // WorldGen.cs:21314.
        "cacti" => PassMessage::Gen(38),
        // WorldGen.cs:21525 — `Lang.gen[84]`.
        "tile cleanup" => PassMessage::Gen(84),
        // WorldGen.cs:21977.
        "water plants" => PassMessage::Gen(88),
        // WorldGen.cs:22040 — `Language.GetTextValue("WorldGeneration.Speleothems")`.
        "stalac" => PassMessage::Key("WorldGeneration.Speleothems"),
        // WorldGen.cs:22125 — `Lang.gen[82]`.
        "remove broken traps" => PassMessage::Gen(82),
        // WorldGen.cs:22145 — `Lang.gen[86]`.
        "final cleanup" => PassMessage::Gen(86),
        // WorldGen.cs:22489 — the GenerateWorld tail's own message
        // (the port's world-tail step is that tail's stand-in).
        "world tail" => PassMessage::Gen(87),
        _ => return None,
    })
}
