// S1-J-3 golden-fixture harness — the ID counts + the four Sets the save path
// indexes, built with the verbatim SetFactory (Generated/SetFactory.cs) and the
// exact id lists from the decompiled static constructors.
//
//   TileID.Count = 754            (ID/TileID.cs)
//   NPCID.Count  = 697            (ID/NPCID.cs:12581)
//   ItemID.Count = 6196           (ID/ItemID.cs:14861)
//   BuffID.Count = 401            (ID/BuffID.cs:997)
//   WallID.Count = 367            (ID/WallID.cs:845)
//
//   TileID.Sets.AllowsSaveCompressionBatching = CreateBoolSet(true, 520,423,723,724)   (TileID.cs:484)
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
