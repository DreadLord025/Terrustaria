// S1-J-3 golden-fixture harness — the entity classes the save path serializes.
//
// These carry exactly the fields the extracted save bodies read, at the C#'s
// declared types and defaults. The fixture (Program.cs) populates them; the
// verbatim serializers then read them. Where a C# member is a property with a
// trivial backing field (NPC.GivenName, WorldFileData.SeedText), the harness
// keeps the same shape.

using System;
using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Terraria.DataStructures;
using Terraria.GameContent.Creative;
using Terraria.IO;

namespace Terraria
{
    // --- NPC instance fields SaveNPCs reads (NPC.cs:6005-6755) ---------------
    public partial class NPC
    {
        public bool active;
        public int type;
        public int netID;
        public bool townNPC;
        public int townNpcVariationIndex;
        public bool homeless;
        public bool homelessDespawn;
        public int homeTileX = -1;
        public int homeTileY = -1;
        public Vector2 position;
        private string _givenName = "";
        public string GivenName
        {
            get => _givenName;
            set => _givenName = value ?? "";
        }
    }

    // --- Item: the fields Serialize / the chest & TE writers read ------------
    public partial class Item
    {
        public int type;          // Item.cs:141
        public int stack;         // Item.cs:157
        public byte prefix;       // Item.cs:305
        public bool favorited;    // Item.cs:143
        // The C# ctor is empty (`public Item() {}`, Item.cs:48654) - a fresh
        // item is air (type 0, stack 0). SetDefaults is not on the save path.
        // AddClaimableBanner's guard (BannerSystem.cs:246); cctor Item.cs:49966.
        public static readonly int CommonMaxStack = 9999;
    }

    // --- Chest: SaveChests reads x/y/name/maxItems/item[] --------------------
    public partial class Chest
    {
        public readonly int index;
        public int x;
        public int y;
        public string name;
        public int maxItems;
        public Item[] item;

        // `private Chest(int index = 0, int x = 0, int y = 0, bool bank = false,
        // int maxItems = 40)` (Chest.cs:191-200) — verbatim shape.
        public Chest(int index = 0, int x = 0, int y = 0, bool bank = false, int maxItems = 40)
        {
            this.maxItems = maxItems;
            this.item = new Item[maxItems];
            this.index = index;
            this.x = x;
            this.y = y;
            this.name = string.Empty;
        }

        // `FillWithEmptyInstances` (Chest.cs:157-164).
        public void FillWithEmptyInstances()
        {
            for (int i = 0; i < this.maxItems; i++) this.item[i] = new Item();
        }

        public static Chest CreateBank(int index) => new Chest(index, 0, 0, bank: true);

        // `RemoveChest` (Chest.cs:132-140): nulls the slot. The harness keeps no
        // _chestsByCoords book (not read by the save path).
        public static void RemoveChest(int chestIndex) => Main.chest[chestIndex] = null;
    }

    // --- Sign: SaveSigns reads text/x/y --------------------------------------
    public class Sign
    {
        public const int maxSigns = 32000;
        public int x;
        public int y;
        public string text;
    }

    // --- Player: every field Player.Serialize reads, at the C# defaults ------
    public partial class Player
    {
        // The verbatim Serialize is private (Player.cs:54750); this public seam
        // calls it unchanged so the driver can run the real body.
        public static void SerializeForFixture(PlayerFileData pfd, Player p, BinaryWriter w)
            => Serialize(pfd, p, w);

        public string name = "";                                   // :826
        public byte difficulty;                                    // :1654
        public int hair;                                           // :2568
        public byte hairDye;                                       // :2546
        public int team;                                           // :1430
        public bool[] hideVisibleAccessory = new bool[10];         // :1708
        public BitsByte hideMisc = (byte)0;                        // :1710
        public int skinVariant;                                    // :1344
        public int statLife = 100;                                 // :1928
        public int statLifeMax = 100;                              // :1924
        public int statMana;                                       // :1930 (ctor sets 20)
        public int statManaMax;                                    // :1932 (ctor sets 20)
        public bool extraAccessory;                                // :866
        public bool unlockedBiomeTorches;                          // :2046
        public bool UsingBiomeTorches => unlockedBiomeTorches && builderAccStatus[11] == 0; // :3950
        public bool ateArtisanBread;                               // :2048
        public bool usedAegisCrystal, usedAegisFruit, usedArcaneCrystal; // :852-856
        public bool usedGalaxyPearl, usedGummyWorm, usedAmbrosia;  // :858-862
        public bool downedDD2EventAnyDifficulty;                   // :1916
        public int taxMoney;                                       // :828
        public int numberOfDeathsPVE, numberOfDeathsPVP;           // :834
        public Color hairColor = new Color(215, 90, 55);           // :2554
        public Color skinColor = new Color(255, 125, 90);          // :2556
        public Color eyeColor = new Color(105, 90, 75);            // :2558
        public Color shirtColor = new Color(175, 165, 140);        // :2560
        public Color underShirtColor = new Color(160, 180, 215);   // :2562
        public Color pantsColor = new Color(255, 230, 175);        // :2564
        public Color shoeColor = new Color(160, 105, 60);          // :2566
        public Item[] armor = new Item[20];                        // :1488
        public Item[] dye = new Item[10];                          // :1490
        public Item[] inventory = new Item[59];                    // :1576
        public Item[] miscEquips = new Item[5];                    // :1492
        public Item[] miscDyes = new Item[5];                      // :1494
        public readonly Chest bank = Chest.CreateBank(-2);         // :1582
        public readonly Chest bank2 = Chest.CreateBank(-3);        // :1584
        public readonly Chest bank3 = Chest.CreateBank(-4);        // :1586
        public readonly Chest bank4 = Chest.CreateBank(-5);        // :1588
        public BitsByte voidVaultInfo;                             // :1590
        public static readonly int maxBuffs = 44;                  // :57106
        public int[] buffType = new int[maxBuffs];                 // :1510
        public int[] buffTime = new int[maxBuffs];                 // :1512
        public string[] spN = new string[200];                     // :2488
        public int[] spX = new int[200];                           // :2484
        public int[] spY = new int[200];                           // :2486
        public int[] spI = new int[200];                           // :2490
        public bool hbLocked;                                      // :1436
        public bool[] hideInfo = new bool[13];                     // :812
        public int anglerQuestsFinished;                           // :1910
        public SelectionRadial DpadRadial = new SelectionRadial(); // :800
        public int[] builderAccStatus = new int[BuilderAccToggleIDs.Count]; // :814
        public int bartenderQuestLog;                              // :1914
        public bool dead;                                          // :1630
        public int respawnTimer;                                   // :1636
        public long lastTimePlayerWasSaved;                        // :1646
        public int golferScoreAccumulated;                         // :1912
        public CreativeUnlocksTracker creativeTracker;             // :794
        public bool unlockedSuperCart;                             // :2052
        public bool enabledSuperCart = true;                       // :2054
        public int CurrentLoadoutIndex;                            // :3207
        public EquipmentLoadout[] Loadouts = new EquipmentLoadout[3] // :3200
        {
            new EquipmentLoadout(), new EquipmentLoadout(), new EquipmentLoadout()
        };
        public int voiceVariant;                                   // :1346 (ctor sets 1)
        public float voicePitchOffset;                             // :1348
        public HashSet<string> oneTimeDialoguesSeen = new HashSet<string>(); // :3209
        public OverheadMessage chatOverhead;                       // :798

        // Player.OverheadMessage (Player.cs:765-784). NewMessage's snippet
        // parse + font measure (ChatManager/FontAssets) is UI-only; the shim
        // keeps the serialized-adjacent fields (text/time/color). The fixture
        // broadcasts with author 255, so this is never reached anyway.
        public struct OverheadMessage
        {
            public string chatText;
            public int timeLeft;
            public Color color;
            public void NewMessage(string message, int displayTime)
            {
                chatText = message;
                timeLeft = displayTime;
            }
        }

        // `BuilderAccToggleIDs.Count = 12` (Player.cs:47-75).
        public static class BuilderAccToggleIDs
        {
            public static readonly int Count = 12;
        }

        // `SelectionRadial` (Player.cs:570-621), Dpad4 default: 4 bindings, -1.
        public class SelectionRadial
        {
            public int[] Bindings;
            public SelectionRadial()
            {
                Bindings = new int[4];
                for (int i = 0; i < 4; i++) Bindings[i] = -1;
            }
        }

        // The C# ctor (Player.cs:56036-56094) reduced to the save-relevant
        // initializations: the item arrays filled, the banks filled, the stats
        // and the creative tracker. The visual/frame fields are not serialized.
        public Player()
        {
            voiceVariant = 1;
            for (int i = 0; i < 59; i++)
            {
                if (i < armor.Length) armor[i] = new Item();
                inventory[i] = new Item();
            }
            bank.FillWithEmptyInstances();
            bank2.FillWithEmptyInstances();
            bank3.FillWithEmptyInstances();
            bank4.FillWithEmptyInstances();
            for (int j = 0; j < dye.Length; j++) dye[j] = new Item();
            for (int k = 0; k < miscEquips.Length; k++) miscEquips[k] = new Item();
            for (int l = 0; l < miscDyes.Length; l++) miscDyes[l] = new Item();
            statManaMax = 20;
            statMana = 20;
            creativeTracker = new CreativeUnlocksTracker();
            builderAccStatus[0] = 1;
        }
    }

    // --- EquipmentLoadout: Serialize reads Armor/Dye/Hide (EquipmentLoadout.cs)
    public partial class EquipmentLoadout
    {
        public Item[] Armor;
        public Item[] Dye;
        public bool[] Hide;
        public EquipmentLoadout()
        {
            Armor = NewItems(20);
            Dye = NewItems(10);
            Hide = new bool[10];
        }
        private static Item[] NewItems(int n)
        {
            var a = new Item[n];
            for (int i = 0; i < n; i++) a[i] = new Item();
            return a;
        }
    }
}

namespace Terraria.IO
{
    // --- WorldFileData: the fields SaveWorldHeader reads ---------------------
    public class WorldFileData
    {
        public DateTime CreationTime;
        public ulong WorldGeneratorVersion;
        private string _seedText = "";
        public Guid UniqueId;
        public int WorldId;
        public int GameMode;
        public string Name;
        public string SeedText => _seedText;
        public void SetSeedText(string s) => _seedText = s;
    }

    // --- PlayerFileData: Serialize reads GetPlayTime().Ticks -----------------
    public class PlayerFileData
    {
        public Player Player;
        public FileMetadata Metadata;
        private TimeSpan _playTime = TimeSpan.Zero;
        // `GetPlayTime` (PlayerFileData.cs) with the stopwatch never started.
        public TimeSpan GetPlayTime() => _playTime;
        public void SetPlayTime(TimeSpan t) => _playTime = t;
    }
}
