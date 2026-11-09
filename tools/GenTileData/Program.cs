// Parses the decompiled Terraria sources and emits the Rust module
// `crates/tr-core/src/tiledata/` (sets.rs, emissive.rs, merge.rs; the sibling
// mod.rs is hand-written).
//
// Extracted data:
//   * TileID.Sets.*  (all `Factory.CreateBoolSet(...)` arrays)
//   * Main.tileSolid / tileSolidTop / tileBrick / tileStone / tileRope / tilePile /
//     tileMoss / tileBlendAll / tileMergeDirt / tileLargeFrames
//   * the directed merge relation from WorldGen.TileFrameCosmetic
//
// The emitted files are generated: do not edit by hand.

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;

const int TileCount = 754;

var root = args.Length > 0 ? args[0] : @"C:\Users\DreadLord\Desktop\Terraria-Rust";
var decompiled = Path.Combine(root, "decompiled");
// Optional second arg: output path override (used to diff a regeneration
// against the committed file without touching it).
var outFile = args.Length > 1 ? args[1] : Path.Combine(root, "crates", "tr-core", "src", "tiledata.rs");

// The decompiler writes nested directories (`Terraria/ID/`, `Terraria/Graphics/Light/`).
var tileIdSrc = File.ReadAllText(Path.Combine(decompiled, "Terraria", "ID", "TileID.cs"));
var mainSrc = File.ReadAllText(Path.Combine(decompiled, "Terraria", "Main.cs"));
var worldGenSrc = File.ReadAllText(Path.Combine(decompiled, "Terraria", "WorldGen.cs"));

// Guard against the decompiled game version drifting from TileCount: `TileID.cs`
// assigns `TileID.Count = <n>;` in its static constructor.
var declaredCount = Regex.Match(tileIdSrc, @"TileID\.Count\s*=\s*(\d+)\s*;");
if (!declaredCount.Success || int.Parse(declaredCount.Groups[1].Value) != TileCount)
    throw new Exception(
        $"decompiled TileID.Count {(declaredCount.Success ? declaredCount.Groups[1].Value : "?")} != TileCount {TileCount}");

// ---------------------------------------------------------------- int[] fields
// Declarations (`public static int[] X;`) and initializations are split across
// static constructors; match the assignment (`RoomNeeds.X = new int[n] { ... }`).
var intArrays = new Dictionary<string, List<int>>();
foreach (Match m in Regex.Matches(tileIdSrc,
             @"((?:\w+\.)*\w+)\s*=\s*new int\[\d*\]\s*\{(?<body>[^}]*)\}"))
{
    var vals = new List<int>();
    foreach (var tok in m.Groups["body"].Value.Split(','))
    {
        var t = tok.Trim();
        if (t.Length == 0) continue;
        if (int.TryParse(t, out var v)) vals.Add(v);
    }
    intArrays[LastSegment(m.Groups[1].Value)] = vals;
}

// ---------------------------------------------------------------- Sets arrays
// Assigned in static constructors as `<Class.>Name = Sets.Factory.CreateBoolSet(...)`;
// the key is the last identifier segment of the left side.
var sets = new Dictionary<string, SortedSet<int>>();
foreach (Match m in Regex.Matches(tileIdSrc,
             @"((?:\w+\.)*\w+)\s*=\s*(?:[\w.]+\.)?Factory\.CreateBoolSet\((?<args>[^;]*?)\)\s*;",
             RegexOptions.Singleline))
{
    var name = LastSegment(m.Groups[1].Value);
    var ids = new SortedSet<int>();
    var tokens = SplitTopLevel(m.Groups["args"].Value).Select(x => x.Trim())
        .Where(x => x.Length > 0).ToList();

    // `CreateBoolSet(bool defaultState, params int[] types)` fills everything with
    // `defaultState`, then flips the listed ids.
    bool defaultState = false;
    if (tokens.Count > 0 && (tokens[0] == "true" || tokens[0] == "false"))
    {
        defaultState = tokens[0] == "true";
        tokens.RemoveAt(0);
    }

    var flipped = new SortedSet<int>();
    foreach (var t0 in tokens)
    {
        var t = t0;
        if (t == "default(int)") { flipped.Add(0); continue; }
        if (int.TryParse(t, out var v)) { flipped.Add(v); continue; }
        // Array references are class-qualified in the static constructors
        // (`CreateBoolSet(RoomNeeds.CountsAsChairTypes)`).
        if (t.Contains('.')) t = LastSegment(t);
        if (intArrays.TryGetValue(t, out var arr)) { foreach (var a in arr) flipped.Add(a); continue; }
        Console.Error.WriteLine($"warn: unknown CreateBoolSet token '{t0}' in {name}");
    }

    if (defaultState)
    {
        for (int i = 0; i < TileCount; i++)
            if (!flipped.Contains(i)) ids.Add(i);
    }
    else
    {
        foreach (var v in flipped) ids.Add(v);
    }

    sets[name] = ids;
}

// ---------------------------------------------------------------- Main arrays
string[] boolArrays =
[
    "tileSolid", "tileSolidTop", "tileBrick", "tileStone", "tileRope",
    "tilePile", "tileMoss", "tileBlendAll", "tileMergeDirt",
    "tileBlockLight", "tileNoSunLight", "tileLighted",
];
var mainBools = boolArrays.ToDictionary(n => n, _ => new bool[TileCount]);
var largeFrames = new byte[TileCount];

const int WallCount = 367;
var wallLight = new bool[WallCount];
foreach (Match m in Regex.Matches(mainSrc,
             @"^\s*(?:Main\.)?wallLight\[(\d+)\]\s*=\s*true\s*;",
             RegexOptions.Multiline))
{
    var idx = int.Parse(m.Groups[1].Value);
    if (idx >= 0 && idx < WallCount) wallLight[idx] = true;
}

foreach (Match m in Regex.Matches(mainSrc,
             @"^\s*(?:Main\.)?(tileSolid|tileSolidTop|tileBrick|tileStone|tileRope|tilePile|tileMoss|tileBlendAll|tileMergeDirt|tileBlockLight|tileNoSunLight|tileLighted|tileLargeFrames)\[(\d+)\]\s*=\s*(true|false|\d+)\s*;",
             RegexOptions.Multiline))
{
    var arr = m.Groups[1].Value;
    var idx = int.Parse(m.Groups[2].Value);
    var val = m.Groups[3].Value;
    if (idx < 0 || idx >= TileCount) continue;
    if (arr == "tileLargeFrames")
    {
        largeFrames[idx] = byte.Parse(val);
    }
    else
    {
        mainBools[arr][idx] = val switch
        {
            "true" => true,
            "false" => false,
            _ => int.Parse(val) != 0,
        };
    }
}

// BlockMergesWithMergeAllBlock is derived in Main.SetupTileMerge.
var mergesWithAll = new bool[TileCount];
for (int j = 0; j < TileCount; j++)
{
    bool excluded = j is 10 or 387 or 541;
    mergesWithAll[j] = !excluded && mainBools["tileSolid"][j] && !mainBools["tileSolidTop"][j];
}

// ---------------------------------------------------------------- int sets
// `CreateIntSet(defaultState, id, value, id, value, ...)` — used for
// `TileID.Sets.NeedsGrassFramingDirt` (which dirt a grass variant frames as).
var intSets = new Dictionary<string, int[]>();
foreach (Match m in Regex.Matches(tileIdSrc,
             @"((?:\w+\.)*\w+)\s*=\s*(?:[\w.]+\.)?Factory\.CreateIntSet\((?<args>[^;]*?)\)\s*;",
             RegexOptions.Singleline))
{
    var vals = SplitTopLevel(m.Groups["args"].Value)
        .Select(x => x.Trim())
        .Where(x => x.Length > 0)
        .Select(x => int.TryParse(x, out var v)
            ? v
            : intArrays.TryGetValue(LastSegment(x), out var arr) ? -1 : int.MinValue)
        .ToList();
    if (vals.Count < 1 || vals[0] == int.MinValue) continue;
    var table = Enumerable.Repeat(vals[0], TileCount).ToArray();
    for (int k = 1; k + 1 < vals.Count; k += 2)
    {
        var idx = vals[k];
        if (idx >= 0 && idx < TileCount) table[idx] = vals[k + 1];
    }
    intSets[LastSegment(m.Groups[1].Value)] = table;
}

// ---------------------------------------------------------------- merge relation
// Walk WorldGen.TileFrameCosmetic, tracking `switch (num)` cases so that
// `TileMergeAttempt(num, X, ...)` can be resolved to a concrete tile id.
var merge = new bool[TileCount, TileCount];
for (int i = 0; i < TileCount; i++) merge[i, i] = true;

var wgLines = worldGenSrc.Split('\n');
int bodyStart = FindLine(wgLines, "public static void TileFrameCosmetic");
int switchLine = -1;
for (int i = bodyStart; i < wgLines.Length; i++)
{
    if (wgLines[i].Contains("switch (num)") && i < bodyStart + 120) { switchLine = i; break; }
}

// cases[i] = tile ids whose `case` label is active at line i.
// Tracked with brace depth so nested `switch (num6)` labels (0/1/2) never leak in.
var caseAt = new List<int>?[wgLines.Length];
if (switchLine >= 0)
{
    var stack = new List<(int Depth, List<int> Ids)>();
    int depth = 0;
    bool entered = false;
    for (int i = switchLine; i < wgLines.Length; i++)
    {
        var line = StripComment(wgLines[i]);
        var trimmed = line.Trim();

        if (!entered)
        {
            if (line.Contains('{')) { entered = true; depth = 1; }
            caseAt[i] = [];
            continue;
        }

        // Close any case context that ended before this line.
        while (stack.Count > 0 && stack[^1].Depth > depth) stack.RemoveAt(stack.Count - 1);

        var cm = Regex.Match(trimmed, @"^case\s+(?<ids>[\d\s:case]+):");
        if (cm.Success)
        {
            var ids = Regex.Matches(cm.Groups["ids"].Value, @"\d+")
                .Select(x => int.Parse(x.Value)).ToList();
            if (stack.Count > 0 && stack[^1].Depth == depth) stack[^1] = (depth, ids);
            else stack.Add((depth, ids));
        }
        else if (trimmed.StartsWith("default:"))
        {
            if (stack.Count > 0 && stack[^1].Depth == depth) stack.RemoveAt(stack.Count - 1);
        }

        caseAt[i] = stack.Count > 0 ? stack[^1].Ids : [];

        depth += line.Count(c => c == '{') - line.Count(c => c == '}');
        if (depth <= 0) break;
    }
}

var mergeCalls = 0;
for (int i = bodyStart; i < wgLines.Length; i++)
{
    var line = StripComment(wgLines[i]);
    var callIdx = line.IndexOf("TileMergeAttempt", StringComparison.Ordinal);
    if (callIdx < 0) continue;
    var open = line.IndexOf('(', callIdx);
    if (open < 0) continue;
    // collect until balanced
    var sb = new StringBuilder();
    int d = 0, end = open;
    for (int k = open; k < line.Length; k++)
    {
        if (line[k] == '(') d++;
        else if (line[k] == ')') { d--; if (d == 0) { end = k; break; } }
        if (k > open) sb.Append(line[k]);
    }
    var callArgs = SplitTopLevel(sb.ToString()).Select(a => a.Trim()).ToList();
    if (callArgs.Count < 2) continue;

    // The first two arguments are (myType, lookfor); the rest are ref neighbour slots.
    var myTokens = ResolveMy(callArgs[0], i, caseAt, wgLines);
    if (myTokens.Count == 0) continue;
    var look = ResolveLook(callArgs[1], sets, mainBools, largeFrames);
    if (look.Count == 0) continue;

    foreach (var mine in myTokens)
        foreach (var other in look)
        {
            if (mine < 0 || mine >= TileCount || other < 0 || other >= TileCount) continue;
            if (mine == other) continue;
            merge[mine, other] = true;
            mergeCalls++;
        }
}

// Special normalisations used by the framer (stone variants frame as stone, etc.).
var stone = mainBools["tileStone"];

// ---------------------------------------------------------------- tile light
// `TileLightScanner.ApplyTileLight` lights tiles per type. Most cases just assign a
// constant R/G/B triple at the top level of the case; take the first such triple.
// Frame-dependent / conditional branches (torches, campfires...) are approximated in
// `crates/tr-core/src/lighting.rs` instead.
var scannerSrc = File.ReadAllLines(Path.Combine(decompiled, "Terraria", "Graphics",
                                               "Light", "TileLightScanner.cs"));
var emissive = new float[TileCount, 3];
int atl = FindLine(scannerSrc, "private void ApplyTileLight");
int atlEnd = atl;
{
    int d = 0;
    bool started = false;
    for (int i = atl; i < scannerSrc.Length; i++)
    {
        var line = StripComment(scannerSrc[i]);
        if (!started)
        {
            if (line.Contains('{')) { started = true; d = 1; }
            continue;
        }
        d += line.Count(c => c == '{') - line.Count(c => c == '}');
        if (d <= 0) { atlEnd = i; break; }
    }
}

var switchStack = new List<(int OpenDepth, bool IsType)>();
var pendingSwitch = 0; // 0 = none, 1 = switch (tile.type), 2 = other switch
List<int>? caseIds = null;
int caseDepth = -1;
int ldepth = 0;
float cr = 0, cg = 0, cb = 0;
bool hr = false, hg = false, hb = false;
int litCases = 0;

for (int i = atl; i <= atlEnd; i++)
{
    var line = StripComment(scannerSrc[i]);
    var trimmed = line.Trim();
    if (trimmed.Length == 0) continue;

    // Case labels are matched at their switch's body depth.
    if (trimmed.StartsWith("case ") && switchStack.Count > 0)
    {
        var top = switchStack[^1];
        if (top.IsType && ldepth == top.OpenDepth + 1)
        {
            var ids = Regex.Matches(trimmed[5..trimmed.IndexOf(':')], @"\d+")
                .Select(x => int.Parse(x.Value)).ToList();
            caseIds = ids;
            caseDepth = ldepth;
            hr = hg = hb = false;
        }
    }
    else if (trimmed.StartsWith("default:"))
    {
        if (switchStack.Count > 0)
        {
            var top = switchStack[^1];
            if (top.IsType && ldepth == top.OpenDepth + 1)
            {
                caseIds = null;
                caseDepth = -1;
            }
        }
    }
    else if (caseIds != null && ldepth == caseDepth && caseIds.Count > 0)
    {
        var m = Regex.Match(trimmed, @"^([RGB]) = ([\d.]+)f;$");
        if (m.Success)
        {
            var v = float.Parse(m.Groups[2].Value, System.Globalization.CultureInfo.InvariantCulture);
            switch (m.Groups[1].Value)
            {
                case "R": cr = v; hr = true; break;
                case "G": cg = v; hg = true; break;
                default: cb = v; hb = true; break;
            }
            if (hr && hg && hb)
            {
                foreach (var id in caseIds)
                {
                    if (emissive[id, 0] == 0f && emissive[id, 1] == 0f && emissive[id, 2] == 0f)
                    {
                        emissive[id, 0] = cr;
                        emissive[id, 1] = cg;
                        emissive[id, 2] = cb;
                        litCases++;
                    }
                }
                hr = hg = hb = false;
            }
        }
    }

    if (trimmed.StartsWith("switch "))
        pendingSwitch = trimmed.StartsWith("switch (tile.type)") ? 1 : 2;

    int opens = line.Count(c => c == '{');
    for (int k = 0; k < opens; k++)
    {
        ldepth++;
        if (pendingSwitch != 0)
        {
            switchStack.Add((ldepth - 1, pendingSwitch == 1));
            pendingSwitch = 0;
        }
    }
    int closes = line.Count(c => c == '}');
    for (int k = 0; k < closes; k++)
    {
        ldepth--;
        while (switchStack.Count > 0 && switchStack[^1].OpenDepth >= ldepth)
        {
            var wasType = switchStack[^1].IsType;
            switchStack.RemoveAt(switchStack.Count - 1);
            if (wasType)
            {
                caseIds = null;
                caseDepth = -1;
            }
        }
    }
}

// ---------------------------------------------------------------- emit
// The output is a module directory (`crates/tr-core/src/tiledata/`):
//   sets.rs     — TileSet bitsets from `Main.tile*` and `TileID.Sets.*`
//   emissive.rs — WALL_LIGHT + EMISSIVE
//   merge.rs    — MERGE relation + the framing tables (LARGE_FRAMES, GRASS_DIRT)
// `mod.rs` (counters, `TileSet`, re-exports) is hand-written and never touched
// here; if the emitted item set changes, update its re-exports.
// Optional second arg: output base directory override (used to diff a
// regeneration against the committed files without touching them).
var outDir = args.Length > 1 ? args[1] : Path.Combine(root, "crates", "tr-core", "src");
var tiledataDir = Path.Combine(outDir, "tiledata");
Directory.CreateDirectory(tiledataDir);

var sbSets = new StringBuilder();
EmitHeader(sbSets, "Tile-id bitset tables derived from `Main.tile*` and `TileID.Sets.*`.");
sbSets.AppendLine();
sbSets.AppendLine("use super::TileSet;");

var sbLight = new StringBuilder();
EmitHeader(sbLight, "Per-tile light tables: `Main.wallLight` and `TileLightScanner.ApplyTileLight`.");
sbLight.AppendLine();
sbLight.AppendLine("use super::{TILE_COUNT, WALL_COUNT};");

var sbMerge = new StringBuilder();
EmitHeader(sbMerge, "Directed merge relation from `WorldGen.TileFrameCosmetic` plus the framing",
                 "layout tables (`Main.tileLargeFrames`, `TileID.Sets.NeedsGrassFramingDirt`).");
sbMerge.AppendLine();
sbMerge.AppendLine("use super::TILE_COUNT;");

static void EmitHeader(StringBuilder sb, params string[] docLines)
{
    sb.AppendLine("// @generated by tools/GenTileData from the decompiled Terraria sources.");
    sb.AppendLine("// Do not edit by hand.");
    sb.AppendLine();
    foreach (var line in docLines)
        sb.AppendLine($"//! {line}");
}

static void EmitSet(StringBuilder sb, string rustName, IEnumerable<int> ids)
{
    var bits = new ulong[(TileCount + 63) / 64];
    foreach (var id in ids)
    {
        if (id < 0 || id >= TileCount) continue;
        bits[id >> 6] |= 1UL << (id & 63);
    }
    sb.AppendLine();
    sb.AppendLine($"pub static {rustName}: TileSet = TileSet(&[");
    for (int i = 0; i < bits.Length; i += 4)
    {
        sb.Append("    ");
        for (int k = i; k < Math.Min(i + 4, bits.Length); k++)
            sb.Append($"0x{bits[k]:x16}, ");
        sb.AppendLine();
    }
    sb.AppendLine("]);");
}

foreach (var name in boolArrays)
    EmitSet(sbSets, ToRustName(name), Enumerable.Range(0, TileCount).Where(i => mainBools[name][i]));
EmitSet(sbSets, "MERGES_WITH_ALL", Enumerable.Range(0, TileCount).Where(i => mergesWithAll[i]));

// TileID.Sets arrays used by framing.
string[] wantedSets =
[
    "Dirt", "Mud", "Ash", "Snow", "IcesSnow", "GrassSpecial", "JungleSpecial", "HellSpecial",
    "MergesWithClouds", "ForcedDirtMerging", "NeedsGrassFraming",
    "OreMergesWithMud", "tileMossBrick", "Platforms", "IsVine", "Falling", "Torches",
    "MergesWithDirtInASpecialWay", "CrackedBricks", "AllowLightInWater",
    // Grass framing (WorldGen.TileFrameCosmetic 82143/82185/83395):
    // Conversion.Grass, Sets.Ore and ChecksForMerge gate the grass family path.
    "Grass", "Ore", "ChecksForMerge",
    // Restored 2026-11-07: landed in sets.rs by M2-PHYS-1 without the generator
    // (Conversion.* TileID.cs:92ff, IceSkateSlippery :481, Clouds :505) — the
    // committed symbol names are kept via the dot-to-underscore emission below.
    "Conversion.Sand", "Conversion.HardenedSand", "Conversion.Sandstone",
    // The gem/vanity tree ground tests (`WorldGen.cs:30304-30341`): the
    // Stone ({1, 25, 117, 203}) and Moss (the 11 moss tiles) conversion
    // sets `GemTreeGroundTest` reads (TileID.cs:90/:96).
    "Conversion.Stone", "Conversion.Moss",
    "IceSkateSlippery", "Clouds",
    // `TileRunner`'s override check (`WorldGen.cs:76927`) — everything is clearable
    // during generation except these 17 (`TileID.cs:573`, `CreateBoolSet(true, …)`),
    // so the emitted set is the complement (default-true emission, Program.cs:92-96).
    "CanBeClearedDuringGeneration",
];
foreach (var name in wantedSets)
{
    var key = name.Contains('.') ? name[(name.IndexOf('.') + 1)..] : name;
    if (!sets.TryGetValue(key, out var ids))
    {
        Console.Error.WriteLine($"warn: TileID.Sets.{key} not found");
        continue;
    }
    // The Rust name follows the wanted entry with dots as underscores, so the
    // parser's last-segment key (`Sand`) still emits as SET_CONVERSION_SAND;
    // dot-less entries are unchanged.
    var rustName = "SET_" + ToRustName(name.Replace('.', '_')).ToUpperInvariant();
    EmitSet(sbSets, rustName, ids);
}

sbLight.AppendLine();
sbLight.AppendLine("/// `Main.wallLight`: walls that transmit sky light.");
sbLight.AppendLine("pub static WALL_LIGHT: [bool; WALL_COUNT] = [");
for (int i = 0; i < WallCount; i += 32)
{
    sbLight.Append("    ");
    for (int k = i; k < Math.Min(i + 32, WallCount); k++)
        sbLight.Append(wallLight[k] ? "true, " : "false, ");
    sbLight.AppendLine();
}
sbLight.AppendLine("];");

sbLight.AppendLine();
sbLight.AppendLine("/// Constant emissive light per tile type, from `TileLightScanner.ApplyTileLight`.");
sbLight.AppendLine("/// Types with frame-dependent light are handled procedurally in `lighting.rs`.");
sbLight.AppendLine("pub static EMISSIVE: [[f32; 3]; TILE_COUNT] = [");
for (int i = 0; i < TileCount; i++)
{
    var inv = System.Globalization.CultureInfo.InvariantCulture;
    sbLight.AppendLine(
        $"    [{emissive[i, 0].ToString("0.0#####", inv)}, {emissive[i, 1].ToString("0.0#####", inv)}, {emissive[i, 2].ToString("0.0#####", inv)}],");
}
sbLight.AppendLine("];");

sbMerge.AppendLine();
sbMerge.AppendLine("/// `Main.tileLargeFrames`: 0 = normal, 1/2 = 4-frame / 2x2 layouts.");
sbMerge.AppendLine("pub static LARGE_FRAMES: [u8; TILE_COUNT] = [");
for (int i = 0; i < TileCount; i += 32)
{
    sbMerge.Append("    ");
    for (int k = i; k < Math.Min(i + 32, TileCount); k++) sbMerge.Append($"{largeFrames[k]}, ");
    sbMerge.AppendLine();
}
sbMerge.AppendLine("];");

// NeedsGrassFramingDirt: which base dirt id a grass variant frames as.
if (intSets.TryGetValue("NeedsGrassFramingDirt", out var grassDirt))
{
    sbMerge.AppendLine();
    sbMerge.AppendLine("/// `TileID.Sets.NeedsGrassFramingDirt`: dirt id a grass variant is framed against.");
    sbMerge.AppendLine("pub static GRASS_DIRT: [u16; TILE_COUNT] = [");
    for (int i = 0; i < TileCount; i += 32)
    {
        sbMerge.Append("    ");
        for (int k = i; k < Math.Min(i + 32, TileCount); k++)
            sbMerge.Append($"{grassDirt[k]}, ");
        sbMerge.AppendLine();
    }
    sbMerge.AppendLine("];");
}

// Merge relation.
sbMerge.AppendLine();
sbMerge.AppendLine("/// Directed merge relation: `MERGE[a][b]` means tile `a` treats a neighbouring");
sbMerge.AppendLine("/// tile `b` as part of its own mass when choosing an autotile frame.");
var wordsPerRow = (TileCount + 63) / 64;
var mergeWords = new ulong[TileCount * wordsPerRow];
for (int a = 0; a < TileCount; a++)
    for (int b = 0; b < TileCount; b++)
        if (merge[a, b])
            mergeWords[a * wordsPerRow + (b >> 6)] |= 1UL << (b & 63);

sbMerge.AppendLine($"static MERGE: [u64; {mergeWords.Length}] = [");
for (int i = 0; i < mergeWords.Length; i += 4)
{
    sbMerge.Append("    ");
    for (int k = i; k < Math.Min(i + 4, mergeWords.Length); k++)
        sbMerge.Append($"0x{mergeWords[k]:x16}, ");
    sbMerge.AppendLine();
}
sbMerge.AppendLine("];");
sbMerge.AppendLine();
sbMerge.AppendLine("/// True when tile `a` merges with tile `b` for autotiling purposes.");
sbMerge.AppendLine("#[inline]");
sbMerge.AppendLine("pub fn merges(a: u16, b: u16) -> bool {");
sbMerge.AppendLine($"    let (a, b) = (a as usize, b as usize);");
sbMerge.AppendLine($"    if a >= TILE_COUNT || b >= TILE_COUNT {{ return false; }}");
sbMerge.AppendLine($"    (MERGE[a * {wordsPerRow} + (b >> 6)] >> (b & 63)) & 1 != 0");
sbMerge.AppendLine("}");

File.WriteAllText(Path.Combine(tiledataDir, "sets.rs"), sbSets.ToString());
File.WriteAllText(Path.Combine(tiledataDir, "emissive.rs"), sbLight.ToString());
File.WriteAllText(Path.Combine(tiledataDir, "merge.rs"), sbMerge.ToString());

Console.WriteLine($"sets: {sets.Count}, merge calls resolved: {mergeCalls}, output: {tiledataDir}");
Console.WriteLine($"  emissive tile types: {litCases}");
Console.WriteLine($"  wallLight: {wallLight.Count(v => v)}/{WallCount}");
Console.WriteLine($"  Dirt={sets.GetValueOrDefault("Dirt")?.Count ?? 0} " +
                  $"Mud={sets.GetValueOrDefault("Mud")?.Count ?? 0} " +
                  $"Snow={sets.GetValueOrDefault("Snow")?.Count ?? 0}");
Console.WriteLine($"  tileSolid={mainBools["tileSolid"].Count(v => v)} " +
                  $"tileBrick={mainBools["tileBrick"].Count(v => v)} " +
                  $"tileMergeDirt={mainBools["tileMergeDirt"].Count(v => v)} " +
                  $"stone={stone.Count(v => v)}");

// ---------------------------------------------------------------- helpers
static int FindLine(string[] lines, string needle)
{
    for (int i = 0; i < lines.Length; i++)
        if (lines[i].Contains(needle)) return i;
    throw new Exception($"not found: {needle}");
}

static string StripComment(string line)
{
    var i = line.IndexOf("//", StringComparison.Ordinal);
    return i >= 0 ? line[..i] : line;
}

static List<string> SplitTopLevel(string s)
{
    var parts = new List<string>();
    int depth = 0;
    var cur = new StringBuilder();
    foreach (var c in s)
    {
        switch (c)
        {
            case '(' or '[' or '{': depth++; cur.Append(c); break;
            case ')' or ']' or '}': depth--; cur.Append(c); break;
            case ',' when depth == 0: parts.Add(cur.ToString()); cur.Clear(); break;
            default: cur.Append(c); break;
        }
    }
    if (cur.Length > 0) parts.Add(cur.ToString());
    return parts;
}

static List<int> ResolveMy(string token, int line, List<int>?[] caseAt, string[] lines)
{
    var t = token.Trim();
    if (int.TryParse(t, out var lit)) return lit < 0 ? [] : [lit];
    // `type` / `tileCache.type` are the framed tile itself. The `numN` family
    // covers the `TileID.Sets.NeedsGrassFramingDirt[num]` local that decompilers
    // keep renaming (num11 in the previous decompilation, num9 in the current
    // one); it is approximated by the active case ids (the grass id itself),
    // which is what the framer expects grass variants to merge against.
    if (t == "type" || t == "tileCache.type" || Regex.IsMatch(t, @"^num\d*$"))
    {
        var cases = caseAt[line];
        if (cases is { Count: > 0 }) return cases;
        return [];
    }
    return [];
}

static List<int> ResolveLook(string token, Dictionary<string, SortedSet<int>> sets,
                             Dictionary<string, bool[]> mainBools, byte[] largeFrames)
{
    var t = token.Trim();
    if (int.TryParse(t, out var lit)) return [lit];
    if (t.StartsWith("TileID.Sets.")) t = t["TileID.Sets.".Length..];
    if (t.StartsWith("Main.")) t = t["Main.".Length..];
    if (sets.TryGetValue(t, out var s)) return s.ToList();
    if (mainBools.TryGetValue(t, out var arr))
        return Enumerable.Range(0, TileCount).Where(i => arr[i]).ToList();
    if (t == "tileLargeFrames")
        return Enumerable.Range(0, TileCount).Where(i => largeFrames[i] != 0).ToList();
    return [];
}

static string LastSegment(string qualified)
{
    var dot = qualified.LastIndexOf('.');
    return dot >= 0 ? qualified[(dot + 1)..] : qualified;
}

static string ToRustName(string name) =>
    Regex.Replace(name, "(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "_").ToUpperInvariant();
