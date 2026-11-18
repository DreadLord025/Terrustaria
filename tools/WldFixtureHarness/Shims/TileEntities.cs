// S1-J-3 golden-fixture harness — the tile-entity class hierarchy the save path
// walks. TileEntity.Write/WriteInner are extracted verbatim
// (Generated/TileEntitySave.cs); each kind's WriteExtraData is extracted verbatim
// (Generated/TileEntityKinds.cs). This file declares the base statics, the
// per-kind fields those bodies read, and the registration-order type ids
// (TileEntitiesManager.RegisterAll, TileEntitiesManager.cs:26-38).

using System.Collections.Generic;
using System.IO;
using Terraria.DataStructures;

namespace Terraria.DataStructures
{
    public abstract partial class TileEntity
    {
        public static object EntityCreationLock = new object();
        public static Dictionary<int, TileEntity> ByID = new Dictionary<int, TileEntity>();
        public static Dictionary<Point16, TileEntity> ByPosition = new Dictionary<Point16, TileEntity>();
        public static int TileEntitiesNextID;

        public int ID;
        public Point16 Position;
        public byte type;

        // The base no-op the kinds override (TileEntity.cs:209-211).
        public virtual void WriteExtraData(BinaryWriter writer, bool networkSend) { }

        public static int AssignNewID() => TileEntitiesNextID++;

        // `Add` (TileEntity.cs:78-89) reduced to the two books the save reads.
        public static void Add(TileEntity ent)
        {
            ByID[ent.ID] = ent;
            ByPosition[ent.Position] = ent;
        }
    }
}

namespace Terraria.GameContent.Tile_Entities
{
    // Registration order = type id (TileEntitiesManager.RegisterAll):
    // 0 TrainingDummy, 1 ItemFrame, 2 LogicSensor, 3 DisplayDoll, 4 WeaponsRack,
    // 5 HatRack, 6 FoodPlatter, 7 TeleportationPylon, 8 DeadCellsDisplayJar,
    // 9 KiteAnchor, 10 CritterAnchor.

    public partial class TETrainingDummy : TileEntity
    {
        public int npc = -1;   // TETrainingDummy.cs:16
    }

    public partial class TEItemFrame : TileEntity
    {
        public Item item = new Item();
    }

    public partial class TELogicSensor : TileEntity
    {
        public enum LogicCheckType : byte { None, Day, Night, Water, Lava, Honey, Liquid, PlayerAbove, Walls }
        public LogicCheckType logicCheck;
        public bool On;
    }

    public partial class TEDisplayDoll : TileEntity
    {
        public Item[] _equip = NewItems(9);
        public Item[] _dyes = NewItems(9);
        public Item[] _misc = NewItems(1);
        public byte _pose;
        private static Item[] NewItems(int n)
        {
            var a = new Item[n];
            for (int i = 0; i < n; i++) a[i] = new Item();
            return a;
        }
    }

    public partial class TEWeaponsRack : TileEntity
    {
        public Item item = new Item();
    }

    public partial class TEHatRack : TileEntity
    {
        public Item[] _items = { new Item(), new Item() };
        public Item[] _dyes = { new Item(), new Item() };
    }

    public partial class TEFoodPlatter : TileEntity
    {
        public Item item = new Item();
    }

    // The pylon overrides neither WriteExtraData nor ReadExtraData — no extra
    // bytes (TileEntity.cs:209-215 base no-ops).
    public class TETeleportationPylon : TileEntity { }

    public partial class TEDeadCellsDisplayJar : TileEntity
    {
        public Item item = new Item();
    }

    // The kite/critter anchors share TELeashedEntityAnchorWithItem's single
    // `short itemType` (TELeashedEntityAnchorWithItem.cs:12-21).
    public abstract class TELeashedEntityAnchor : TileEntity { }
    public partial class TELeashedEntityAnchorWithItem : TELeashedEntityAnchor
    {
        protected int itemType;
        public void SetItemType(int t) => itemType = t;
    }
    public class TEKiteAnchor : TELeashedEntityAnchorWithItem { }
    public class TECritterAnchor : TELeashedEntityAnchorWithItem { }
}
