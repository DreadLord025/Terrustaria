// S1-A-7 worldgen-parity harness — the ID counts + the Sets the worldgen leaf
// bodies AND the reused S1-J-3 save path index. Superset of
// tools/WldFixtureHarness/Shims/Ids.cs (which is excluded from this project):
// the four save-path sets are the same verbatim lists, plus the three sets
// WorldGen.TileRunner reads, at the decompiled static-constructor lines.
//
//   TileID.Count = 754            (ID/TileID.cs)
//   NPCID.Count  = 697            (ID/NPCID.cs:12581)
//   ItemID.Count = 6196           (ID/ItemID.cs:14861)
//   BuffID.Count = 401            (ID/BuffID.cs:997)
//   WallID.Count = 367            (ID/WallID.cs:845)
//
//   TileID.Sets.AllowsSaveCompressionBatching = CreateBoolSet(true, 520,423,723,724)   (TileID.cs:484)
//   TileID.Sets.NonSolidSaveSlopes = CreateBoolSet(false, 131,351,336,340,342,341,343,344)  (TileID.cs:496)
//   TileID.Sets.SaveSlopes[i] = Main.tileSolid[i] || NonSolidSaveSlopes[i]              (TileID.cs:468-471)
//     — filled by Program.cs after the Main.cs tile-table parse, the C# order.
//   TileID.Sets.CanBeClearedDuringGeneration = CreateBoolSet(true, 396,400,401,397,398,399,404,368,367,41,43,44,481,482,483,226,237)  (TileID.cs:573)
//   TileID.Sets.Ore = CreateBoolSet(7,166,6,167,9,168,8,169,22,204,37,58,107,221,108,222,111,223,211)  (TileID.cs:614)
//   ItemID.Sets.ItemsThatShouldNotBeInInventory = CreateBoolSet(false, 58,184,1734,1735,1867,1868,3453,3454,3455,4143,5013,6135,6136)  (ItemID.cs:345)
//   NPCID.Sets.SavesAndLoads = CreateBoolSet(422,507,517,493)                          (NPCID.cs:10449)

namespace Terraria.ID
{
    public static class TileID
    {
        public const ushort Count = 754;
        public static class Sets
        {
            public static readonly bool[] AllowsSaveCompressionBatching =
                new SetFactory(Count).CreateBoolSet(true, 520, 423, 723, 724);
            public static readonly bool[] NonSolidSaveSlopes =
                new SetFactory(Count).CreateBoolSet(false, 131, 351, 336, 340, 342, 341, 343, 344);
            // TileID.cs:468-471 — Program.cs fills it from Main.tileSolid.
            public static readonly bool[] SaveSlopes = new bool[Count];
            public static readonly bool[] CanBeClearedDuringGeneration =
                new SetFactory(Count).CreateBoolSet(true, 396, 400, 401, 397, 398, 399, 404, 368, 367, 41, 43, 44, 481, 482, 483, 226, 237);
            public static readonly bool[] Ore =
                new SetFactory(Count).CreateBoolSet(7, 166, 6, 167, 9, 168, 8, 169, 22, 204, 37, 58, 107, 221, 108, 222, 111, 223, 211);
        }
    }

    public static class NPCID
    {
        public const int Count = 697;
        public const short Guide = 22;
        public const short Merchant = 17;
        public const short Nurse = 18;
        public const short TravellingMerchant = 368;
        public const short LunarTowerVortex = 422;
        public const short LunarTowerNebula = 507;
        public const short LunarTowerStardust = 493;
        public const short LunarTowerSolar = 517;
        // The reused tracker save path resolves the AnglerFish name
        // (`WldFixtureHarness/Shims/Globals.cs:223`); `NPCID.cs:10881`.
        public const short AnglerFish = 102;
        public static class Sets
        {
            public static readonly bool[] SavesAndLoads =
                new SetFactory(Count).CreateBoolSet(422, 507, 517, 493);
        }
    }

    public static class ItemID
    {
        public const int Count = 6196;
        public static class Sets
        {
            public static readonly bool[] ItemsThatShouldNotBeInInventory =
                new SetFactory(Count).CreateBoolSet(false, 58, 184, 1734, 1735, 1867, 1868, 3453, 3454, 3455, 4143, 5013, 6135, 6136);
            // The reused tracker save path reads the banner table
            // (`Generated/TrackerSave.cs:196`); `ItemID.cs:1303-1304`.
            public static int DefaultKillsForBannerNeeded = 50;
            public static readonly int[] KillsToBanner =
                new SetFactory(Count).CreateIntSet(DefaultKillsForBannerNeeded, 3838, 1000, 3845, 200, 3837, 500, 3844, 200, 3843, 50, 3839, 150, 3840, 100, 3842, 200, 3841, 100, 3846, 50, 2971, 150, 2982, 150, 2931, 100, 2961, 100, 2994, 100, 2985, 10, 4541, 10, 2969, 10, 2986, 10, 2915, 10, 4602, 10, 4542, 25, 4543, 25, 4546, 25, 4545, 25, 2901, 25, 2902, 25, 1631, 25, 2913, 25, 4688, 25, 3390, 25, 4973, 25, 4974, 25, 4975, 25, 2934, 25, 1670, 25, 1694, 25, 2958, 25, 2960, 25, 3441, 25, 3780, 25, 3397, 25, 3403, 25);
        }
    }

    public static class BuffID
    {
        public const int Count = 401;
    }

    public static class WallID
    {
        public const int Count = 367;
    }
}
