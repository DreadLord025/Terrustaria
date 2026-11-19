// S1-A-7 worldgen-parity harness — the global statics the extracted worldgen
// leaf bodies read (WorldGen.TileRunner / PlaceWall / CanBeClearedDuringGeneration),
// plus the genRand RNG wrapper. The S1-J-1 / S1-J-3 oracle pattern: the method
// bodies stay verbatim (Generated/WorldGenRunner.cs), only the global lookups
// become shim state.
//
// The reused S1-J-3 shims (Globals.cs) already declare Main.maxTilesX/Y, tile,
// tileFrameImportant, tileSolid, tileSolidTop, worldSurface, rockLayer and the
// WorldGen save-path statics. This file adds ONLY the worldgen-side fields those
// bodies read on top of the save path.

using System;
using Microsoft.Xna.Framework;
using Terraria.Utilities;

namespace Terraria
{
    public static partial class Main
    {
        // WorldGen.genRand aliases Main.rand (WorldGen.cs:4445). The harness
        // populates it with either the real Knuth stream or the port-stream
        // replay (Program.cs).
        public static UnifiedRandom rand;
        // TileRunner's frame guard + protection chain read these two tables.
        public static bool[] tileCut;
        public static bool[] tileStone;
    }

    public static partial class WorldGen
    {
        // WorldGen.cs:4445 — `public static UnifiedRandom genRand => Main.rand;`
        public static UnifiedRandom genRand => Main.rand;

        // The secret-seed flags TileRunner's arms read (WorldGen.cs:76772+).
        // A classic world keeps them all false (Program.cs sets them).
        public static bool drunkWorldGen;
        public static bool remixWorldGen;
        public static bool getGoodWorldGen;
        public static bool notTheBees;
        public static bool dontStarveWorldGen;
        public static bool tenthAnniversaryWorldGen;

        // WorldGen.cs:4154 / :86914 — `beachDistance = 380`.
        public static int beachDistance = 380;

        // CanBeClearedDuringGeneration's dual-dungeon gate reads this. The
        // harness runs a classic world (no dual dungeons), so it is false and
        // the gate short-circuits before DungeonUtils is ever consulted.
        public static bool isGeneratingOrLoadingWorld = true;

        // SquareWallFrame (WorldGen.cs:80198) is a VISUAL wall-framing pass the
        // port defers (tile_runner.rs place_wall doc): wall frames are runtime-
        // only and never serialized, so the comparison stays on saved fields.
        // A no-op here is exactly the port's convention. PlaceWall always calls
        // it with the default resetFrame, and the wall *id* write above it is
        // the only persisted effect.
        public static void SquareWallFrame(int i, int j, bool resetFrame = true) { }

        // oceanDepths (WorldGen.cs:10963) is read only inside the remix arm of
        // the type -2 liquid fill. A classic world never reaches it; the stub
        // keeps the verbatim line compilable.
        public static bool oceanDepths(int x, int y) => false;
    }

    // GenVars (WorldBuilding/GenVars.cs) — the four fields TileRunner reads.
    namespace WorldBuilding
    {
        public static class GenVars
        {
            public static bool mudWall;                       // :176
            public static int lavaLine;                       // :62
            public static int waterLine;                      // :64
            public static Rectangle UndergroundDesertLocation; // :138
        }
    }

    // SecretSeed.dualDungeons (WorldGen.cs:41 / :3056) — the Enabled gate
    // CanBeClearedDuringGeneration reads. Classic world: disabled.
    public class SecretSeed
    {
        public static SecretSeed dualDungeons = new SecretSeed();
        public bool Enabled => false;
    }
}

// SoundEngine.PlaySound — PlaceWall calls it only when !mute; every
// TileRunner call site passes mute:true, so it is never reached. A no-op
// keeps the verbatim line compilable. Top-level namespace so the generated
// `using Terraria.Audio;` + unqualified `SoundEngine` resolves.
namespace Terraria.Audio
{
    public static class SoundEngine
    {
        public static void PlaySound(int type, int x, int y) { }
    }
}

// DungeonUtils (GameContent/Generation/Dungeon/DungeonUtils.cs) — only reached
// when SecretSeed.dualDungeons.Enabled, which is false here. Stubs keep the
// verbatim CanBeClearedDuringGeneration line compilable.
namespace Terraria.GameContent.Generation.Dungeon
{
    public static class DungeonUtils
    {
        public static bool InAnyPotentialDungeonBounds(int x, int y, int fluff = 0, bool inner = false) => false;
        public static bool IsConsideredDungeonTile(int tileType, bool allDungeons = false) => false;
    }
}

// XNA Rectangle — GenVars.UndergroundDesertLocation.Contains(k, l). The S1-J-3
// Xna.cs shim has no Rectangle (the save path never touches one); the runner's
// sand/kind-59 containment check does. Empty rect => Contains always false,
// matching the port's deferred underground-desert containment (tile_runner.rs).
namespace Microsoft.Xna.Framework
{
    public struct Rectangle
    {
        public int X, Y, Width, Height;
        public Rectangle(int x, int y, int w, int h) { X = x; Y = y; Width = w; Height = h; }
        public bool Contains(int x, int y)
            => x >= X && x < X + Width && y >= Y && y < Y + Height;
    }
}

namespace Terraria.Utilities
{
    // The genRand wrapper. Two modes:
    //  * REAL  — wraps UnifiedRandomCore (the verbatim Knuth subtractive
    //            System.Random, Generated/UnifiedRandomCore.cs). This is the
    //            actual vanilla stream.
    //  * DIFF  — replays the port's xorshift64 stream (tr_core::rng::Xorshift64)
    //            so the verbatim C# TileRunner can be compared byte-for-byte
    //            against the port's tile_runner given the SAME draw sequence.
    //            This isolates the algorithm from the (deliberately different)
    //            stream — research/35 deviation #5.
    public class UnifiedRandom
    {
        private readonly UnifiedRandomCore _core;
        private readonly PortStream _port;

        public UnifiedRandom(int seed) { _core = new UnifiedRandomCore(seed); }
        private UnifiedRandom(PortStream port) { _port = port; }
        public static UnifiedRandom FromPortState(ulong state) => new UnifiedRandom(new PortStream(state));

        public int Next() => _port != null ? (int)(_port.NextU32() & 0x7FFFFFFF) : _core.Next();
        public int Next(int maxValue) => _port != null ? _port.Range(0, maxValue) : _core.Next(maxValue);
        public int Next(int minValue, int maxValue) => _port != null ? _port.Range(minValue, maxValue) : _core.Next(minValue, maxValue);
        public double NextDouble() => _port != null ? _port.NextU32() / 4294967296.0 : _core.NextDouble();

        // The port's final stream state (DIFF mode) — the differential test
        // pins it against the port's Rng.0 after the same call sequence.
        public ulong PortState => _port?.State ?? 0;
    }

    // tr_core::rng::Xorshift64 (crates/tr-core/src/rng.rs:26-79), byte-exact:
    //   next_u64 = the 13/7/17 triple; next_u32 = HIGH half of the stepped word;
    //   range(lo,hi) = lo + next_u32() % (hi-lo), empty range returns lo.
    // C# ulong shifts/arith here are unchecked by default (no overflow throw).
    public class PortStream
    {
        private ulong _s;
        public PortStream(ulong state) { _s = state; }
        public ulong State => _s;
        public ulong NextU64()
        {
            ulong x = _s;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            _s = x;
            return x;
        }
        public uint NextU32() => (uint)(NextU64() >> 32);
        public int Range(int lo, int hi)
        {
            if (hi == lo) return lo;
            return lo + (int)(NextU32() % (uint)(hi - lo));
        }
    }
}
