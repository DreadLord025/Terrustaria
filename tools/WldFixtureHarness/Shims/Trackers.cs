// S1-J-3 golden-fixture harness — the tracker classes whose Save bodies are
// extracted verbatim (Generated/TrackerSave.cs). Only the collections the Save
// bodies enumerate are declared; the fixture populates them directly.

using System.Collections.Concurrent;
using System.Collections.Generic;
using Microsoft.Xna.Framework;

namespace Terraria.GameContent.Bestiary
{
    // NPCKillsTracker.Save writes `_killCountsByNpcId` (Dictionary<string,int>).
    public partial class NPCKillsTracker
    {
        private Dictionary<string, int> _killCountsByNpcId = new Dictionary<string, int>();
        private object _entryCreationLock = new object();
        public void SetKill(string id, int count) => _killCountsByNpcId[id] = count;
    }

    // NPCWasNearPlayerTracker.Save writes `_wasNearPlayer` (HashSet<string>).
    public partial class NPCWasNearPlayerTracker
    {
        private HashSet<string> _wasNearPlayer = new HashSet<string>();
        private object _entryCreationLock = new object();
        public void AddSight(string id) => _wasNearPlayer.Add(id);
    }

    // NPCWasChatWithTracker.Save writes `_chattedWithPlayer` (HashSet<string>).
    public partial class NPCWasChatWithTracker
    {
        private HashSet<string> _chattedWithPlayer = new HashSet<string>();
        private object _entryCreationLock = new object();
        public void AddChat(string id) => _chattedWithPlayer.Add(id);
    }

    // BestiaryUnlocksTracker.Save calls Kills/Sights/Chats.Save in order.
    public partial class BestiaryUnlocksTracker
    {
        public NPCKillsTracker Kills = new NPCKillsTracker();
        public NPCWasNearPlayerTracker Sights = new NPCWasNearPlayerTracker();
        public NPCWasChatWithTracker Chats = new NPCWasChatWithTracker();
    }
}

namespace Terraria.GameContent.Creative
{
    // ItemsSacrificedUnlocksTracker.Save writes `false` then the
    // `_sacrificeCountByItemPersistentId` array (ConcurrentDictionary).
    public partial class ItemsSacrificedUnlocksTracker
    {
        private ConcurrentDictionary<string, int> _sacrificeCountByItemPersistentId
            = new ConcurrentDictionary<string, int>();
        public void SetSacrifice(string id, int count) => _sacrificeCountByItemPersistentId[id] = count;
    }

    // CreativeUnlocksTracker.Save calls ItemSacrifices.Save.
    public partial class CreativeUnlocksTracker
    {
        public ItemsSacrificedUnlocksTracker ItemSacrifices = new ItemsSacrificedUnlocksTracker();
    }
}

namespace Terraria.GameContent
{
    // TownRoomManager.Save writes `_roomLocationPairs` (List<Tuple<int,Point>>).
    public partial class TownRoomManager
    {
        public static object EntityCreationLock = new object();
        private List<System.Tuple<int, Point>> _roomLocationPairs = new List<System.Tuple<int, Point>>();
        public void SetRoom(int npcID, int x, int y)
            => _roomLocationPairs.Add(System.Tuple.Create(npcID, new Point(x, y)));
    }

    // BannerSystem.Save writes killCount[] then claimableBanners[]. W4-c44:
    // the arrays are allocated by the EXTRACTED static cctor
    // (BannerSystem.cs:1451-1457) and populated by the extracted kill chain
    // (AddNPCKillBy/AddKill/AddClaimableBanner, :231-278) - no shim setters.
    public static partial class BannerSystem
    {
        public static readonly int MaxBannerTypes;   // cctor: 293 (:1453)
        private static int[] killCount;              // cctor (:1454)
        private static ushort[] claimableBanners;    // cctor (:1455)
        private static bool[] newBanner;             // cctor (:1456)
        public static bool AnyNewClaimableBanners;   // :132

        // The netMode==2 broadcast sinks the extracted AddKill /
        // AddClaimableBanner guard on (:265-268, :274-277). The fixture runs
        // at netMode 0, so these are never reached; they throw rather than
        // model the network (no serialization logic lives in the shims).
        public class NetBannersModule
        {
            public static Terraria.Net.NetPacket WriteKillCountUpdate(int bannerId)
                => throw new System.InvalidOperationException("netMode 2 path not modelled");
            public static Terraria.Net.NetPacket WriteClaimCountUpdate(int bannerId)
                => throw new System.InvalidOperationException("netMode 2 path not modelled");
        }
    }

    // ExtraSpawnPointManager.Write writes extraSpawnPoints[] (Point[]).
    public static partial class ExtraSpawnPointManager
    {
        public static Point[] extraSpawnPoints = new Point[0];
    }

    // TreeTopsInfo.Save writes `_variations` (int[13]).
    public partial class TreeTopsInfo
    {
        public class AreaId
        {
            public static readonly int Count = 13;
        }
        private int[] _variations = new int[AreaId.Count];
        public void SetVariation(int area, int value) => _variations[area] = value;
    }

    // CraftingRequests.SavePossibleRefunds sums `_pendingCrafts` consumed items.
    public static partial class CraftingRequests
    {
        public struct RemoteCraftRequest
        {
            public List<Item> consumed;
        }
        private static Queue<RemoteCraftRequest> _pendingCrafts = new Queue<RemoteCraftRequest>();
    }

    // PressurePlateHelper.Save writes the PressurePlatesPressed dictionary keys.
    public static class PressurePlateHelper
    {
        public static object EntityCreationLock = new object();
        public static Dictionary<Point, bool[]> PressurePlatesPressed = new Dictionary<Point, bool[]>();
    }
}
