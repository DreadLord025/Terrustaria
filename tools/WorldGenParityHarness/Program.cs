// S1-A-7 worldgen-parity driver.
//
// Runs the REAL decompiled genRand-driven worldgen leaf (WorldGen.TileRunner,
// extracted verbatim into Generated/WorldGenRunner.cs by extract.py) and writes
// the resulting tile array through the REAL decompiled save path (the S1-J-3
// WldFixtureHarness bodies, reused via the .csproj Compile links).
//
// Two modes:
//   * DIFF (default) — genRand replays the PORT's xorshift64 stream
//     (tr_core::rng::Xorshift64, state passed on the command line). The verbatim
//     C# TileRunner then draws the SAME sequence as the port's tile_runner, so
//     the two tile arrays can be compared byte-for-byte. This isolates the
//     ALGORITHM from the (deliberately different) stream — research/35 #5.
//   * REAL — genRand is the real Knuth UnifiedRandom(seed). This is the actual
//     vanilla stream; the .wld it writes is a genuine C#-generated world. It is
//     NOT byte-comparable with the port (the port draws xorshift, not Knuth);
//     the divergence is documented, not faked.
//
// Usage:
//   python tools/WorldGenParityHarness/extract.py
//   dotnet run --project tools/WorldGenParityHarness -- <out-dir> diff <port-state-u64> [main-cs]
//   dotnet run --project tools/WorldGenParityHarness -- <out-dir> real <seed> [main-cs]
//
// Writes <out-dir>/parity_diff.wld (DIFF) or <out-dir>/parity_real.wld (REAL),
// plus <out-dir>/parity_tiles.bin — the raw post-runner tile array in the
// port's Tile layout order (kind,wall,liquid,flags,slope,halfbrick,active) so
// the Rust test compares field-by-field without re-parsing the .wld RLE.

using System;
using System.IO;
using System.Text.RegularExpressions;
using Microsoft.Xna.Framework;
using Terraria;
using Terraria.ID;
using Terraria.IO;
using Terraria.Utilities;
using Terraria.WorldBuilding;

internal class Program
{
    // The differential scenario: a small stone world with a handful of
    // protected/special tiles, then a fixed set of TileRunner calls covering
    // the ore placement, the protection chain, the mud (kind 59) arms, the
    // liquid carve/fill (type -1/-2) and the mud-wall lining. Every call uses
    // only branches the port has ported (no 367/368 wall arms, no underground-
    // desert containment, no remix oceanDepths — tile_runner.rs module doc).
    private const int W = 200, H = 400;
    private const double WorldSurface = 150.0;
    private const double RockLayer = 200.0;

    private static void Main(string[] args)
    {
        // The Newtonsoft reference is the retail embedded dll with a non-standard
        // file name; resolve it from the output directory (the S1-J-3 pattern).
        System.Runtime.Loader.AssemblyLoadContext.Default.Resolving += (ctx, name) =>
        {
            if (name.Name == "Newtonsoft.Json")
            {
                string p = Path.Combine(AppContext.BaseDirectory, "Terraria.Libraries.JSON.NET.Newtonsoft.Json.dll");
                if (File.Exists(p)) return ctx.LoadFromAssemblyPath(p);
            }
            return null;
        };

        string outDir = args.Length > 0 ? args[0] : ".";
        string mode = args.Length > 1 ? args[1] : "diff";
        Directory.CreateDirectory(outDir);

        // --- the tile tables, parsed from the REAL decompiled Main.cs --------
        string mainCs = args.Length > 3 ? args[3] : FindMainCs();
        var tables = ParseTileTables(mainCs);
        Terraria.Main.tileFrameImportant = tables.frameImportant;
        Terraria.Main.tileSolid = tables.solid;
        Terraria.Main.tileSolidTop = tables.solidTop;
        Terraria.Main.tileCut = tables.cut;
        Terraria.Main.tileStone = tables.stone;
        Terraria.Main.tileContainer = new bool[TileID.Count];
        foreach (int t in new[] { 21, 88, 467, 470, 475 }) Terraria.Main.tileContainer[t] = true;
        // TileID.Sets.SaveSlopes[i] = Main.tileSolid[i] || NonSolidSaveSlopes[i]
        // (TileID.cs:468-471), the C# order.
        for (int i = 0; i < TileID.Count; i++)
            TileID.Sets.SaveSlopes[i] = Terraria.Main.tileSolid[i] || TileID.Sets.NonSolidSaveSlopes[i];

        // --- dimensions -------------------------------------------------------
        Terraria.Main.maxTilesX = W;
        Terraria.Main.maxTilesY = H;
        Terraria.Main.worldSurface = WorldSurface;
        Terraria.Main.rockLayer = RockLayer;
        Terraria.Main.leftWorld = 0f; Terraria.Main.rightWorld = W * 16f;
        Terraria.Main.topWorld = 0f; Terraria.Main.bottomWorld = H * 16f;

        // --- classic world: every secret-seed flag off ------------------------
        WorldGen.drunkWorldGen = false; WorldGen.remixWorldGen = false;
        WorldGen.getGoodWorldGen = false; WorldGen.notTheBees = false;
        WorldGen.dontStarveWorldGen = false; WorldGen.tenthAnniversaryWorldGen = false;
        WorldGen.beachDistance = 380;
        WorldGen.isGeneratingOrLoadingWorld = true;
        GenVars.mudWall = false; GenVars.waterLine = 0; GenVars.lavaLine = 0;
        GenVars.UndergroundDesertLocation = default; // empty: containment never fires

        // --- the tile array: all active stone, plus the special tiles ---------
        Terraria.Main.tile = new Tile[W, H];
        for (int x = 0; x < W; x++)
            for (int y = 0; y < H; y++)
            {
                var t = new Tile();
                t.type = 1; t.active(true); // stone
                Terraria.Main.tile[x, y] = t;
            }
        Put(40, 100, 41);            // silver brick: generation-protected
        Put(41, 100, 10);            // frame-important (door): guard skips, no draw
        Put(42, 100, 53);            // sand above the surface: protected
        Put(43, 100, 396);           // sandstone: yields to ore
        Put(60, 250, 53);            // sand below the surface
        // liquids for the kind-59 clear + the type -2 fill
        Liquid(100, 250, 255, lava: true);
        Liquid(101, 250, 128, lava: false);
        Liquid(150, 320, 255, lava: false);

        // --- genRand: the port stream (DIFF) or the real Knuth stream (REAL) --
        if (mode == "diff")
        {
            ulong state = ulong.Parse(args[2]);
            Terraria.Main.rand = UnifiedRandom.FromPortState(state);
        }
        else
        {
            int seed = int.Parse(args[2]);
            Terraria.Main.rand = new UnifiedRandom(seed);
        }

        // --- the TileRunner calls (the differential scenario) -----------------
        // Each is a real genRand-driven worldgen leaf, verbatim C#. The depth
        // anchors ride GenVars (the C# reads them globally, not as params).
        GenVars.mudWall = false; GenVars.waterLine = 0; GenVars.lavaLine = 0;
        WorldGen.TileRunner(100, 100, 60.0, 4, 7);    // ore over the protection zoo
        WorldGen.TileRunner(50, 50, 4.0, 1, 7);       // single-step ore disc
        GenVars.waterLine = 245;
        WorldGen.TileRunner(100, 250, 4.0, 1, 59);    // mud: stone draw + liquid clear
        GenVars.waterLine = 315; GenVars.lavaLine = 330;
        WorldGen.TileRunner(150, 320, 4.0, 1, -2);    // liquid fill
        GenVars.waterLine = 0; GenVars.lavaLine = 0;
        WorldGen.TileRunner(160, 320, 4.0, 1, -1);    // liquid carve
        GenVars.mudWall = true; GenVars.lavaLine = 200;
        WorldGen.TileRunner(50, 170, 4.0, 1, 7);      // mud-wall lining (l in the (surface, H-210) band)
        GenVars.mudWall = false;

        // --- dump the raw post-runner tile array (the field-by-field oracle) --
        DumpTiles(Path.Combine(outDir, "parity_tiles.bin"));

        // --- write the .wld through the REAL save path ------------------------
        SetupSaveIdentity(mode);
        WorldFile.SetTempToOngoing();
        byte[] wld;
        using (var ms = new MemoryStream())
        {
            using (var w = new BinaryWriter(ms)) WorldFile.SaveWorld_Version2(w);
            wld = ms.ToArray();
        }
        string name = mode == "diff" ? "parity_diff.wld" : "parity_real.wld";
        File.WriteAllBytes(Path.Combine(outDir, name), wld);

        ulong portState = Terraria.Main.rand.PortState;
        Console.WriteLine($"{name}  {wld.Length} bytes  mode={mode}  portState={portState}");
    }

    private static void Put(int x, int y, ushort type)
    {
        var t = Terraria.Main.tile[x, y];
        t.type = type; t.active(true);
    }
    private static void Liquid(int x, int y, byte amount, bool lava)
    {
        var t = Terraria.Main.tile[x, y];
        t.liquid = amount; t.lava(lava);
    }

    private static void DumpTiles(string path)
    {
        using var fs = File.Create(path);
        using var w = new BinaryWriter(fs);
        w.Write(W); w.Write(H);
        for (int x = 0; x < W; x++)
            for (int y = 0; y < H; y++)
            {
                Tile t = Terraria.Main.tile[x, y];
                w.Write(t.type);            // u16 kind
                w.Write(t.wall);            // u16 wall
                w.Write(t.liquid);          // u8 liquid
                w.Write((byte)(t.active() ? 1 : 0));
                w.Write(t.slope());         // u8 slope
                w.Write((byte)(t.halfBrick() ? 1 : 0));
                w.Write(t.lava() ? (byte)1 : (byte)0);
                w.Write(t.honey() ? (byte)1 : (byte)0);
                w.Write(t.shimmer() ? (byte)1 : (byte)0);
            }
    }

    private static void SetupSaveIdentity(string mode)
    {
        Terraria.Main.worldName = "S1A7Parity";
        Terraria.Main.ActiveWorldFileData = new WorldFileData
        {
            Name = "S1A7Parity",
            WorldGeneratorVersion = 1400159338497UL,
            UniqueId = new Guid("01234567-89ab-cdef-0123-456789abcdef"),
            WorldId = 777,
            CreationTime = new DateTime(2026, 11, 18, 0, 0, 0, DateTimeKind.Utc),
            GameMode = 0,
        };
        Terraria.Main.ActiveWorldFileData.SetSeedText("42");
        Terraria.Main.WorldFileMetadata = FileMetadata.FromCurrentSettings(FileType.World);
        Terraria.Main.spawnTileX = W / 2; Terraria.Main.spawnTileY = H / 4;
        Terraria.Main.time = 0.0; Terraria.Main.dayTime = true;
        Terraria.Main.moonPhase = 0; Terraria.Main.moonType = 0;
        Terraria.Main.chest = new Chest[8000];
        Terraria.Main.sign = new Sign[32000];
        Terraria.Main.npc = new NPC[201];
        for (int i = 0; i < Terraria.Main.npc.Length; i++) Terraria.Main.npc[i] = new NPC();
        Terraria.GameContent.Creative.CreativePowerManager.Initialize();
        WorldGen.Manifest = new Terraria.WorldBuilding.WorldManifest();
    }

    private static string FindMainCs()
    {
        string p = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "..", "decompiled", "Terraria", "Main.cs");
        p = Path.GetFullPath(p);
        if (!File.Exists(p)) throw new FileNotFoundException("Main.cs not found for tile-table parse", p);
        return p;
    }

    private struct TileTables
    {
        public bool[] frameImportant, solid, solidTop, cut, stone;
    }

    // Parse the five Main.* tile tables from the REAL decompiled Main.cs: the
    // direct `Main.<table>[n] = true` assignments and the for-loop ranges
    // (`for (int|ushort v = a; v <= b; v++) { ... Main.<table>[v] = true; ... }`),
    // plus the AddEchoFurnitureTile(n) helper (sets tileFrameImportant). This is
    // the game's own table, not a hand transcription (the S1-J-3 pattern).
    private static TileTables ParseTileTables(string mainCsPath)
    {
        string src = File.ReadAllText(mainCsPath);
        var t = new TileTables
        {
            frameImportant = new bool[TileID.Count],
            solid = new bool[TileID.Count],
            solidTop = new bool[TileID.Count],
            cut = new bool[TileID.Count],
            stone = new bool[TileID.Count],
        };
        var byName = new System.Collections.Generic.Dictionary<string, bool[]>
        {
            ["tileFrameImportant"] = t.frameImportant,
            ["tileSolid"] = t.solid,
            ["tileSolidTop"] = t.solidTop,
            ["tileCut"] = t.cut,
            ["tileStone"] = t.stone,
        };
        // Direct literal assignments.
        foreach (Match m in Regex.Matches(src, @"Main\.(tileFrameImportant|tileSolid|tileSolidTop|tileCut|tileStone)\[(\d+)\]\s*=\s*true"))
            byName[m.Groups[1].Value][int.Parse(m.Groups[2].Value)] = true;
        // For-loop ranges: capture the loop var + bounds, then the body's
        // `Main.<table>[var] = true` lines (the body may set several tables).
        foreach (Match m in Regex.Matches(src,
            @"for \((?:int|ushort) (\w+) = (\d+); \1 <= (\d+); \1\+\+\)\s*\{([^{}]*)\}"))
        {
            string v = m.Groups[1].Value;
            int a = int.Parse(m.Groups[2].Value), b = int.Parse(m.Groups[3].Value);
            string body = m.Groups[4].Value;
            foreach (Match bm in Regex.Matches(body, @"Main\.(tileFrameImportant|tileSolid|tileSolidTop|tileCut|tileStone)\[" + v + @"\]\s*=\s*true"))
            {
                bool[] arr = byName[bm.Groups[1].Value];
                for (int i = a; i <= b; i++) arr[i] = true;
            }
        }
        // AddEchoFurnitureTile(n) sets tileFrameImportant[n] (Main.cs helper).
        foreach (Match m in Regex.Matches(src, @"Main\.AddEchoFurnitureTile\((\d+)\)"))
            t.frameImportant[int.Parse(m.Groups[1].Value)] = true;
        return t;
    }
}
