// S1-J-3 golden-fixture harness — the global statics the save path reads.
//
// The decompiled save methods read the game's globals (Main.*, NPC.*,
// WorldGen.*). The harness cannot boot the game, so these are the SAME fields
// with the SAME types/defaults, populated by Program.cs with the fixture's
// deterministic state. Only the fields the extracted save bodies actually read
// are declared here (the census is WorldFileSave.cs / PlayerSerialize.cs /
// TrackerSave.cs); nothing else is modelled. This is the S1-J-1 oracle pattern:
// the method bodies stay verbatim, the global lookups become parameters/state.

using System;
using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Terraria.DataStructures;
using Terraria.GameContent;
using Terraria.GameContent.Bestiary;
using Terraria.GameContent.Creative;
using Terraria.ID;
using Terraria.IO;
using Terraria.Localization;

namespace Terraria
{
    public static partial class Main
    {
        // --- world identity / dimensions (SaveWorldHeader) -------------------
        public static string worldName;
        public static WorldFileData ActiveWorldFileData;
        public static FileMetadata WorldFileMetadata;
        public static float leftWorld;
        public static float rightWorld;
        public static float topWorld;
        public static float bottomWorld;
        public static int maxTilesX;
        public static int maxTilesY;
        public static Tile[,] tile;
        public static Chest[] chest;
        public static Sign[] sign;
        public static NPC[] npc;

        // --- tile property tables (SaveWorldTiles / isTheSameAs) -------------
        public static bool[] tileFrameImportant;
        public static bool[] tileSolid;
        public static bool[] tileSolidTop;
        public static bool[] tileContainer;

        // --- SaveWorldFlags scalars -----------------------------------------
        public static int GameMode;
        public static bool drunkWorld;
        public static bool getGoodWorld;
        public static bool tenthAnniversaryWorld;
        public static bool dontStarveWorld;
        public static bool notTheBeesWorld;
        public static bool remixWorld;
        public static bool noTrapsWorld;
        public static bool zenithWorld;
        public static bool skyblockWorld;
        public static int moonType;
        public static int[] treeX = new int[4];
        public static int[] treeStyle = new int[4];
        public static int[] caveBackX = new int[4];
        public static int[] caveBackStyle = new int[4];
        public static int iceBackStyle;
        public static int jungleBackStyle;
        public static int hellBackStyle;
        public static int spawnTileX;
        public static int spawnTileY;
        public static double worldSurface;
        public static double rockLayer;
        public static double time;
        public static bool dayTime;
        public static int moonPhase;
        public static bool bloodMoon;
        public static bool eclipse;
        public static int dungeonX;
        public static int dungeonY;
        public static bool hardMode;
        public static bool afterPartyOfDoom;
        public static int invasionDelay;
        public static int invasionSize;
        public static int invasionType;
        public static double invasionX;
        public static double slimeRainTime;
        public static int sundialCooldown;
        public static bool raining;
        public static int rainTime;
        public static float maxRaining;
        public static float cloudBGActive;
        public static int numClouds;
        public static float windSpeedTarget;
        public static List<string> anglerWhoFinishedToday = new List<string>();
        public static int anglerQuest;
        public static int invasionSizeStart;
        public static bool fastForwardTimeToDawn;
        public static bool fastForwardTimeToDusk;
        public static int moondialCooldown;
        public static bool forceHalloweenForToday;
        public static bool forceXMasForToday;
        public static bool forceHalloweenForever;
        public static bool forceXMasForever;
        public static bool vampireSeed;
        public static bool infectedSeed;
        public static bool teamBasedSpawnsSeed;
        public static bool dualDungeonsSeed;
        public static bool moreLightningSeed;
        public static bool noLightningSeed;

        // --- SaveBestiary / SaveCreativePowers ------------------------------
        public static BestiaryUnlocksTracker BestiaryTracker = new BestiaryUnlocksTracker();

        // --- Player.Serialize's global reads --------------------------------
        public static bool[] buffNoSave = new bool[BuffID.Count];
        public static int myPlayer;
        public static Item mouseItem = new Item();
        public static Item guideItem = new Item();
        public static Item reforgeItem = new Item();
        public static CreativeUI CreativeMenu = new CreativeUI();

        // --- SaveWorldTiles status text (Lang.gen[49]) ----------------------
        public static string statusText;
    }

    // The `Main.GameMode` property reads ActiveWorldFileData.GameMode; the
    // fixture sets it directly, so a plain field is behaviour-identical here.

    public partial class NPC
    {
        public static bool[] ShimmeredTownNPCs = new bool[NPCID.Count];
        public static bool downedBoss1, downedBoss2, downedBoss3;
        public static bool downedQueenBee;
        public static bool downedMechBoss1, downedMechBoss2, downedMechBoss3, downedMechBossAny;
        public static bool downedPlantBoss, downedGolemBoss, downedSlimeKing;
        public static bool savedGoblin, savedWizard, savedMech;
        public static bool downedGoblins, downedClown, downedFrost, downedPirates;
        public static bool savedAngler, savedStylist, savedTaxCollector, savedGolfer, savedBartender;
        public static bool downedFishron, downedMartians, downedAncientCultist, downedMoonlord;
        public static bool downedHalloweenKing, downedHalloweenTree;
        public static bool downedChristmasIceQueen, downedChristmasSantank, downedChristmasTree;
        public static bool downedTowerSolar, downedTowerVortex, downedTowerNebula, downedTowerStardust;
        public static bool TowerActiveSolar, TowerActiveVortex, TowerActiveNebula, TowerActiveStardust;
        public static bool LunarApocalypseIsUp;
        public static bool downedEmpressOfLight, downedQueenSlime, downedDeerclops;
        public static bool unlockedSlimeBlueSpawn, unlockedMerchantSpawn, unlockedDemolitionistSpawn;
        public static bool unlockedPartyGirlSpawn, unlockedDyeTraderSpawn, unlockedTruffleSpawn;
        public static bool unlockedArmsDealerSpawn, unlockedNurseSpawn, unlockedPrincessSpawn;
        public static bool combatBookWasUsed, combatBookVolumeTwoWasUsed, peddlersSatchelWasUsed;
        public static bool unlockedSlimeGreenSpawn, unlockedSlimeOldSpawn, unlockedSlimePurpleSpawn;
        public static bool unlockedSlimeRainbowSpawn, unlockedSlimeRedSpawn, unlockedSlimeYellowSpawn, unlockedSlimeCopperSpawn;
        public static bool boughtCat, boughtDog, boughtBunny;
    }

    public static partial class WorldGen
    {
        public static bool crimson;
        public static bool shadowOrbSmashed;
        public static bool spawnMeteor;
        public static int shadowOrbCount;
        public static int altarCount;
        public static int treeBG1, treeBG2, treeBG3, treeBG4;
        public static int corruptBG, jungleBG, snowBG, hallowBG, crimsonBG, desertBG, oceanBG;
        public static int mushroomBG, underworldBG;
        public static TownRoomManager TownManager = new TownRoomManager();
        public static TreeTopsInfo TreeTops = new TreeTopsInfo();
        public static WorldBuilding.WorldManifest Manifest = new WorldBuilding.WorldManifest();
        public static int[] tileCounts = new int[TileID.Count];

        public static class SavedOreTiers
        {
            public static int Copper = 7;
            public static int Iron = 6;
            public static int Silver = 9;
            public static int Gold = 8;
            public static int Cobalt = 107;
            public static int Mythril = 108;
            public static int Adamantite = 111;
        }
    }

    // SaveWorldTiles writes `Lang.gen[49].Value + " " + pct + "%"` into
    // Main.statusText (a UI side effect, never serialized). A one-entry stub
    // keeps the verbatim line compilable without the localization manager.
    namespace Localization
    {
        public class LocalizedText
        {
            public string Value { get; }
            public LocalizedText(string value) { Value = value; }
        }

        public static class Lang
        {
            public static LocalizedText[] gen = MakeGen();
            private static LocalizedText[] MakeGen()
            {
                var a = new LocalizedText[64];
                for (int i = 0; i < a.Length; i++) a[i] = new LocalizedText("gen" + i);
                return a;
            }
        }
    }

    // The creative-menu slot the .plr temp-slot block reads (all air in the
    // fixture). Only GetItemByIndex is on the save path.
    namespace GameContent
    {
        public class CreativeUI
        {
            private readonly Item[] _itemSlotsForUI = { new Item() };
            public Item GetItemByIndex(int itemSlotContext)
                => itemSlotContext == 0 ? _itemSlotsForUI[0] : null;
        }
    }
}
