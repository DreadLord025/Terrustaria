//! The vanilla world-generation configuration, embedded and parsed.
//!
//! C# anchors: `WorldGen.cs:11080` (`GenVars.configuration =
//! WorldGenConfiguration.FromEmbeddedPath(...)`),
//! `WorldBuilding/WorldGenConfiguration.cs` (the `Biomes`/`Passes` split,
//! `GetPassConfiguration` returning an empty view for unknown names,
//! `CreateBiome`), `IO/GameConfiguration.cs` (`Get<T>`), and
//! `WorldBuilding/WorldGenRange.cs` (`Min`/`Max`/`ScaleWith`, `GetRandom =
//! Next(ScaledMin, ScaledMax + 1)`, scaling `WorldArea =
//! maxTilesX*maxTilesY/5040000`, `WorldWidth = maxTilesX/4200`, values
//! truncated like C# `(int)(num * value)`).
//!
//! The JSON itself is the game's own embedded
//! `Terraria.GameContent.WorldBuilding.Configuration.json`, copied verbatim
//! next to this file (derived data, same licence position as the tiledata
//! regen). The copy keeps its `// Min/Max` hint comments and the UTF-8 BOM;
//! [`parse`] prefilters both before `serde_json`, which is strict about them.
//! Never hand-edit the copy.
//!
//! The order of the passes is NOT in this file — vanilla declares it in code
//! (`WorldGen.AddPasses`, `WorldGen.cs:11502`); the JSON is a name-keyed
//! option table. See `pipeline.rs` for the pass chain.

use super::noise::Rng;
use serde_json::{Map, Value};

/// The parsed configuration root (`WorldGenConfiguration`).
pub(crate) struct WorldGenConfig {
    biomes: Map<String, Value>,
    passes: Map<String, Value>,
}

/// One name-keyed section of the config — a pass entry from `Passes` or a
/// biome entry from `Biomes` (`GameConfiguration`'s `_root`).
pub(crate) struct ConfigSection {
    entries: Map<String, Value>,
}

impl WorldGenConfig {
    /// `WorldGenConfiguration.FromEmbeddedPath` (`WorldGen.cs:11080`). Panics
    /// on a corrupt embedded file — it is a compile-time constant, so a parse
    /// failure is a programming error, and generating without the config
    /// would silently change the world.
    pub(crate) fn parse() -> WorldGenConfig {
        let raw = include_str!("worldgen_configuration.json");
        let clean = strip_json_comments(raw);
        let root: Value = match serde_json::from_str(&clean) {
            Ok(v) => v,
            Err(e) => panic!("embedded worldgen configuration failed to parse: {e}"),
        };
        let empty = Map::new();
        let biomes = root
            .get("Biomes")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or(empty.clone());
        let passes = root
            .get("Passes")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or(empty);
        WorldGenConfig { biomes, passes }
    }

    /// `GetPassConfiguration(name)` — a pass's own options; an unknown name
    /// yields the empty view, exactly like the C# returning an empty
    /// `JObject`.
    pub(crate) fn pass(&self, name: &str) -> ConfigSection {
        ConfigSection {
            entries: self.passes.get(name).and_then(Value::as_object).cloned().unwrap_or_default(),
        }
    }

    /// `CreateBiome<T>(name)`'s lookup half — a biome's param entry, or `None`
    /// when absent (the C# falls back to `new T()`; the caller owns its
    /// defaults, so absence is surfaced instead of hidden).
    pub(crate) fn biome(&self, name: &str) -> Option<ConfigSection> {
        Some(ConfigSection { entries: self.biomes.get(name)?.as_object()?.clone() })
    }
}

impl ConfigSection {
    /// `GameConfiguration.Get<T>` for the plain-number entries
    /// (`ChanceOfPyramid`, `FlatBeachPadding`, the biome chances, ...).
    pub(crate) fn get_f64(&self, entry: &str) -> Option<f64> {
        self.entries.get(entry)?.as_f64()
    }

    /// `Get<WorldGenRange>(entry)` — the structured `Min`/`Max`/`ScaleWith`
    /// counts.
    pub(crate) fn get_range(&self, entry: &str) -> Option<WorldGenRange> {
        let v = self.entries.get(entry)?;
        WorldGenRange::from_value(v)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// `WorldBuilding/WorldGenRange.cs` — a count option with an optional world
/// scaling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorldGenRange {
    pub(crate) min: i32,
    pub(crate) max: i32,
    pub(crate) scale: ScalingMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalingMode {
    None,
    WorldArea,
    WorldWidth,
}

impl WorldGenRange {
    fn from_value(v: &Value) -> Option<WorldGenRange> {
        let min = v.get("Min")?.as_i64()? as i32;
        let max = v.get("Max")?.as_i64()? as i32;
        let scale = match v.get("ScaleWith").map(Value::as_str) {
            None | Some(None) => {
                // The C# JsonConverter defaults a missing/blank mode to None
                // (StringEnumConverter leaves the field at its default).
                ScalingMode::None
            }
            Some(Some("None")) => ScalingMode::None,
            Some(Some("WorldArea")) => ScalingMode::WorldArea,
            Some(Some("WorldWidth")) => ScalingMode::WorldWidth,
            Some(Some(other)) => panic!("unknown WorldGenRange scaling mode: {other}"),
        };
        Some(WorldGenRange { min, max, scale })
    }

    /// `ScaleValue` — the factor truncation is C#-exact: the scale is computed
    /// in f64, multiplied, then `(int)`-truncated per value.
    fn scale_value(&self, value: i32, world_width: i32, world_height: i32) -> i32 {
        let num = match self.scale {
            ScalingMode::None => 1.0,
            ScalingMode::WorldArea => {
                (world_width as f64 * world_height as f64) / 5040000.0
            }
            ScalingMode::WorldWidth => world_width as f64 / 4200.0,
        };
        (num * value as f64) as i32
    }

    /// `GetRandom` = `random.Next(ScaledMin, ScaledMax + 1)` — the port's
    /// `Rng::range` is exclusive-high like C# `Next(min, max)`, so the +1
    /// carries over verbatim.
    pub(crate) fn get_random(
        &self,
        rng: &mut Rng,
        world_width: i32,
        world_height: i32,
    ) -> i32 {
        let lo = self.scale_value(self.min, world_width, world_height);
        let hi = self.scale_value(self.max, world_width, world_height);
        rng.range(lo, hi + 1)
    }
}

/// Strips the copy's UTF-8 BOM and its whole-line `// Min/Max` hint comments
/// so the strict `serde_json` parser accepts the otherwise-verbatim file.
fn strip_json_comments(raw: &str) -> String {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let mut out = String::with_capacity(raw.len());
    for line in raw.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_verbatim_copy_with_comments_and_bom() {
        // Raw serde_json must reject the file (the prefilter exists for a
        // reason); the stripped form must parse.
        let raw = include_str!("worldgen_configuration.json");
        assert!(raw.starts_with('\u{feff}'));
        assert!(serde_json::from_str::<Value>(raw).is_err());
        assert!(serde_json::from_str::<Value>(&strip_json_comments(raw)).is_ok());

        let cfg = WorldGenConfig::parse();
        assert!(!cfg.pass("Terrain").is_empty());
    }

    #[test]
    fn exposes_the_declared_pass_options() {
        let cfg = WorldGenConfig::parse();
        let dunes = cfg.pass("Dunes");
        let count = dunes.get_range("Count").expect("Dunes Count");
        assert_eq!(
            count,
            WorldGenRange { min: 1, max: 2, scale: ScalingMode::WorldWidth }
        );
        assert_eq!(dunes.get_f64("ChanceOfPyramid"), Some(0.8));

        let micro = cfg.pass("Micro Biomes");
        assert_eq!(
            micro.get_range("LivingTreeCount"),
            Some(WorldGenRange { min: 6, max: 11, scale: ScalingMode::WorldWidth })
        );
        assert_eq!(
            micro.get_range("LongTrackLength"),
            Some(WorldGenRange { min: 400, max: 1000, scale: ScalingMode::WorldWidth })
        );

        let terrain = cfg.pass("Terrain");
        assert_eq!(terrain.get_f64("FlatBeachPadding"), Some(5.0));

        let buried = cfg.pass("Buried Chests");
        assert_eq!(
            buried.get_range("AdditionalDesertHouseCount"),
            Some(WorldGenRange { min: 2, max: 2, scale: ScalingMode::WorldArea })
        );
    }

    #[test]
    fn unknown_names_fall_back_like_the_csharp() {
        let cfg = WorldGenConfig::parse();
        let ghost = cfg.pass("No Such Pass");
        assert!(ghost.is_empty());
        assert_eq!(ghost.get_f64("Count"), None);
        assert_eq!(ghost.get_range("Count"), None);
        assert!(cfg.biome("No Such Biome").is_none());
    }

    #[test]
    fn exposes_the_biome_entries() {
        let cfg = WorldGenConfig::parse();
        let cave_house = cfg.biome("CaveHouseBiome").expect("CaveHouseBiome");
        assert_eq!(cave_house.get_f64("IceChestChance"), Some(1.0));
        assert_eq!(cave_house.get_f64("DesertChestChance"), Some(1.0));
        let dunes = cfg.biome("DunesBiome").expect("DunesBiome");
        assert_eq!(dunes.get_f64("HeightScale"), Some(1.0));
        assert_eq!(
            dunes.get_range("SingleDunesWidth"),
            Some(WorldGenRange { min: 150, max: 250, scale: ScalingMode::None })
        );
        let sword = cfg.biome("EnchantedSwordBiome").expect("EnchantedSwordBiome");
        assert_eq!(sword.get_f64("ChanceOfRealSword"), Some(1.0));
    }

    #[test]
    fn world_gen_range_matches_the_csharp_scaling_and_bounds() {
        // The classic 4200x1200 world is exactly the scale-1 anchor for both
        // modes: WorldArea = 5040000/5040000 = 1, WorldWidth = 4200/4200 = 1.
        let area = WorldGenRange { min: 3, max: 8, scale: ScalingMode::WorldArea };
        let mut rng = Rng(0x1234_5678_9ABC_DEF0);
        for _ in 0..1000 {
            let n = area.get_random(&mut rng, 4200, 1200);
            assert!((3..=8).contains(&n));
        }
        // 6300x1800: WorldArea factor 2.25, WorldWidth 1.5 — values truncate
        // like `(int)(num * value)`: min 3 -> 6, max 8 -> 18.
        let scaled = area.get_random(&mut rng, 6300, 1800);
        let _ = scaled;
        let lo = area.scale_value(3, 6300, 1800);
        let hi = area.scale_value(8, 6300, 1800);
        assert_eq!((lo, hi), (6, 18));
        for _ in 0..1000 {
            let n = area.get_random(&mut rng, 6300, 1800);
            assert!((6..=18).contains(&n));
        }
        // WorldWidth 1.5: min 400 -> 600.
        let track = WorldGenRange { min: 400, max: 1000, scale: ScalingMode::WorldWidth };
        assert_eq!(track.scale_value(400, 6300, 1800), 600);
        assert_eq!(track.scale_value(1000, 6300, 1800), 1500);
        // A fixed (min == max) range always yields that value — the
        // AdditionalDesertHouseCount 2-2 case — regardless of the draw.
        let fixed = WorldGenRange { min: 2, max: 2, scale: ScalingMode::WorldArea };
        for _ in 0..50 {
            assert_eq!(fixed.get_random(&mut rng, 4200, 1200), 2);
        }
    }

    #[test]
    fn the_embedded_copy_is_unmodified() {
        // Pins the copy against accidental hand edits (the provenance
        // sha256 check against the decompile lives in the task's execution
        // log; here the FNV-1a-64 + length catch any byte change).
        let raw = include_str!("worldgen_configuration.json");
        assert_eq!(raw.len(), 5843);
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for b in raw.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        }
        assert_eq!(hash, 0x8da9_6e85_d7d8_dd3b);
    }
}
