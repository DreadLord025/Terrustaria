// S1-J-3 golden-fixture driver.
//
// Builds the SMALLEST real world + player the save path accepts - a fixed
// 40x30 tile array, every value a literal, no genRand anywhere - and writes
// both through the REAL decompiled save bodies (Generated/*.cs, extracted by
// extract.ps1): `WorldFile.SaveWorld_Version2` (WorldFile.cs:1150-1168, the
// exact MemoryStream+BinaryWriter shape of InternalSaveWorld:926-934, after
// the real `SetTempToOngoing` snapshot) and `Player.Serialize`
// (Player.cs:54750+, the exact prologue of InternalSavePlayerFile:54738:
// version 326 + FileMetadata.Write + Serialize), then the real AES-CBC
// envelope (`h3y_gUyZ` key=IV, zero pad to 16).
//
// The ONE substitution extract.ps1 makes is DateTime.UtcNow -> FixtureClock.
// UtcNow (a pinned 2026-11-18T00:00:00Z), because the vanilla writers stamp
// the wall clock (SaveWorldFlags:1251, Player.Serialize:54909) and a
// reproducible fixture cannot carry it. Everything else is verbatim.
//
// `Main.tileFrameImportant` is parsed from the REAL decompiled Main.cs at
// runtime (the direct assignments, the for-loop ranges and AddEchoFurnitureTile
// calls) - the importance bitmap in the fixture is the game's own table, not a
// hand transcription.
//
// Usage:
//   powershell -File tools/WldFixtureHarness/extract.ps1
//   dotnet run --project tools/WldFixtureHarness -- <out-dir> [path-to-Main.cs]
// Writes <out-dir>/golden.wld and <out-dir>/golden.plr.
using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.Loader;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using Microsoft.Xna.Framework;
using Terraria;
using Terraria.DataStructures;
using Terraria.GameContent;
using Terraria.GameContent.Creative;
using Terraria.GameContent.Tile_Entities;
using Terraria.ID;
using Terraria.IO;
using Terraria.WorldBuilding;

internal class Program
{
    private static void Main(string[] args)
    {
        // The Newtonsoft reference is the retail embedded dll with a non-standard
        // file name; resolve it from the output directory.
        AssemblyLoadContext.Default.Resolving += (ctx, name) =>
        {
            if (name.Name == "Newtonsoft.Json")
            {
                string p = Path.Combine(AppContext.BaseDirectory, "Terraria.Libraries.JSON.NET.Newtonsoft.Json.dll");
                if (File.Exists(p)) return ctx.LoadFromAssemblyPath(p);
            }
            return null;
        };

        // --- world identity (SaveWorldHeader reads) ---------------------------
        Terraria.Main.worldName = "S1J3Golden";
        Terraria.Main.ActiveWorldFileData = new WorldFileData
        {
            Name = "S1J3Golden",
            WorldGeneratorVersion = 1400159338497UL, // the 1.4.5-era generator version
            UniqueId = new Guid("01234567-89ab-cdef-0123-456789abcdef"),
            WorldId = 123456789,
            CreationTime = new DateTime(2026, 11, 18, 0, 0, 0, DateTimeKind.Utc),
            GameMode = 0, // classic
        };
        Terraria.Main.ActiveWorldFileData.SetSeedText("12345");
        Terraria.Main.WorldFileMetadata = FileMetadata.FromCurrentSettings(FileType.World);

        // --- dimensions (WorldGen.cs:6551-6554's bounds rule) ------------------
        Terraria.Main.maxTilesX = 40;
        Terraria.Main.maxTilesY = 30;
        Terraria.Main.leftWorld = 0f;
        Terraria.Main.rightWorld = 40 * 16f;
        Terraria.Main.topWorld = 0f;
        Terraria.Main.bottomWorld = 30 * 16f;

        // --- tile tables: importance from the REAL Main.cs ---------------------
        Terraria.Main.tileFrameImportant = ParseFrameImportant(args.Length > 1 ? args[1] : null);
        Terraria.Main.tileSolid = new bool[TileID.Count];
        Terraria.Main.tileSolidTop = new bool[TileID.Count];
        Terraria.Main.tileContainer = new bool[TileID.Count];
        // isTheSameAs reads tileSolid/tileSolidTop only through collisionType,
        // which the save path never calls; the batching equality itself reads
        // tileFrameImportant + the headers. The solid set below is the common
        // terrain kinds (Main.cs InitializeTiles' tileSolid assignments).
        foreach (int t in new[] { 0, 1, 2, 23, 25, 37, 38, 39, 40, 53, 57, 59, 109, 112, 116, 123, 147, 161, 162, 199, 200, 203, 224, 234 })
            Terraria.Main.tileSolid[t] = true;
        Terraria.Main.tileSolidTop[19] = true; // platforms
        // SaveChests' container check (Main.cs:10076-10080).
        foreach (int t in new[] { 21, 88, 467, 470, 475 })
            Terraria.Main.tileContainer[t] = true;

        // --- the tile array -----------------------------------------------------
        Terraria.Main.tile = new Tile[40, 30];
        for (int x = 0; x < 40; x++)
            for (int y = 0; y < 30; y++)
                Terraria.Main.tile[x, y] = new Tile();

        // A dirt floor (rows 24-29): six identical rows per column -> RLE runs.
        for (int x = 0; x < 40; x++)
            for (int y = 24; y < 30; y++)
                SetTile(x, y, 0);
        // A stone strip with the slope/half-brick/actuator/inactive zoo.
        for (int x = 5; x < 10; x++)
            for (int y = 24; y < 27; y++)
                SetTile(x, y, 1);
        Terraria.Main.tile[5, 24].slope(1);
        Terraria.Main.tile[6, 24].slope(2);
        Terraria.Main.tile[7, 24].halfBrick(true);
        Terraria.Main.tile[8, 24].actuator(true);
        Terraria.Main.tile[9, 24].inActive(true);
        // Paint + the block coatings.
        Terraria.Main.tile[12, 24].color(3);
        Terraria.Main.tile[13, 24].invisibleBlock(true);
        Terraria.Main.tile[14, 24].fullbrightBlock(true);
        // The four wire colours, singly and combined.
        Terraria.Main.tile[16, 24].wire(true);
        Terraria.Main.tile[17, 24].wire2(true);
        Terraria.Main.tile[18, 24].wire3(true);
        Terraria.Main.tile[19, 24].wire4(true);
        Terraria.Main.tile[20, 24].wire(true);
        Terraria.Main.tile[20, 24].wire2(true);
        Terraria.Main.tile[20, 24].wire3(true);
        Terraria.Main.tile[20, 24].wire4(true);
        // A wall row: plain, painted, 16-bit id (300), invisible, fullbright.
        for (int x = 0; x < 40; x++)
            Terraria.Main.tile[x, 23].wall = 1;
        Terraria.Main.tile[22, 23].wallColor(5);
        Terraria.Main.tile[23, 23].wall = 300;
        Terraria.Main.tile[24, 23].invisibleWall(true);
        Terraria.Main.tile[25, 23].fullbrightWall(true);
        // The four liquid kinds.
        Terraria.Main.tile[27, 22].liquid = 255; // water
        Terraria.Main.tile[28, 22].liquid = 200; Terraria.Main.tile[28, 22].lava(true);
        Terraria.Main.tile[29, 22].liquid = 150; Terraria.Main.tile[29, 22].honey(true);
        Terraria.Main.tile[30, 22].liquid = 100; Terraria.Main.tile[30, 22].shimmer(true);

        // Furniture row: chest 2x2 (21), sign (55), the TE anchor tiles and the
        // four no-batch kinds (423/520/723/724, TileID.cs:484).
        SetTile(2, 20, 21, 0, 0);
        SetTile(3, 20, 21, 18, 0);
        SetTile(2, 21, 21, 0, 18);
        SetTile(3, 21, 21, 18, 18);
        SetTile(6, 20, 55);
        SetTile(7, 20, 55);
        SetTile(8, 20, 395);  // item frame anchor
        SetTile(10, 20, 471); // weapons rack anchor
        SetTile(12, 20, 423); // no-batch
        SetTile(14, 20, 597); // pylon anchor
        SetTile(16, 20, 475); // hat rack anchor
        SetTile(18, 20, 520); // no-batch
        SetTile(20, 20, 378); // training dummy anchor
        SetTile(22, 20, 470); // display doll anchor
        SetTile(24, 20, 723); // no-batch
        SetTile(26, 20, 724); // no-batch

        // --- chests ---------------------------------------------------------------
        Terraria.Main.chest = new Chest[8000];
        var chest = new Chest(0, 2, 20);
        chest.FillWithEmptyInstances();
        chest.name = "Golden Chest";
        chest.item[0] = new Item { type = 2, stack = 50, prefix = 0 };
        chest.item[1] = new Item { type = 8, stack = 99, prefix = 81 }; // prefixed torch
        chest.item[2] = new Item { type = 58, stack = 5, prefix = 0 };  // Heart: banned -> (short)0
        Terraria.Main.chest[0] = chest;
        var chest2 = new Chest(1, 2, 20);
        chest2.FillWithEmptyInstances();
        chest2.x = 2; chest2.y = 20;
        chest2.name = "";
        chest2.item[0] = new Item { type = 3, stack = 25, prefix = 0 };
        Terraria.Main.chest[1] = chest2;

        // --- signs ------------------------------------------------------------------
        Terraria.Main.sign = new Sign[32000];
        Terraria.Main.sign[0] = new Sign { text = "Hello golden world", x = 6, y = 20 };
        Terraria.Main.sign[1] = new Sign { text = "second sign", x = 7, y = 20 };

        // --- npcs: two town records + one tower + the shimmered list ------------------
        Terraria.Main.npc = new NPC[201];
        for (int i = 0; i < Terraria.Main.npc.Length; i++) Terraria.Main.npc[i] = new NPC();
        Terraria.Main.npc[0].active = true;
        Terraria.Main.npc[0].townNPC = true;
        Terraria.Main.npc[0].type = 22; Terraria.Main.npc[0].netID = 22;
        Terraria.Main.npc[0].GivenName = "Guido";
        Terraria.Main.npc[0].position = new Vector2(100f, 320f);
        Terraria.Main.npc[0].homeless = false;
        Terraria.Main.npc[0].homeTileX = 6; Terraria.Main.npc[0].homeTileY = 20;
        Terraria.Main.npc[0].townNpcVariationIndex = 2;
        Terraria.Main.npc[0].homelessDespawn = false;
        Terraria.Main.npc[1].active = true;
        Terraria.Main.npc[1].townNPC = true;
        Terraria.Main.npc[1].type = 17; Terraria.Main.npc[1].netID = 17;
        Terraria.Main.npc[1].GivenName = "Merchy";
        Terraria.Main.npc[1].position = new Vector2(140f, 320f);
        Terraria.Main.npc[1].homeless = true;
        Terraria.Main.npc[1].homeTileX = -1; Terraria.Main.npc[1].homeTileY = -1;
        Terraria.Main.npc[1].townNpcVariationIndex = 0;
        Terraria.Main.npc[1].homelessDespawn = true;
        Terraria.Main.npc[2].active = true;
        Terraria.Main.npc[2].type = 422; Terraria.Main.npc[2].netID = 422; // vortex pillar
        Terraria.Main.npc[2].position = new Vector2(500f, 200f);
        NPC.ShimmeredTownNPCs[22] = true;

        // --- tile entities: one of every kind the port models --------------------------
        int teId = 0;
        AddTE(new TETrainingDummy { npc = -1 }, 20, 20);
        var frame = AddTE(new TEItemFrame(), 8, 20);
        frame.item = new Item { type = 2, stack = 1, prefix = 0 };
        AddTE(new TELogicSensor { logicCheck = TELogicSensor.LogicCheckType.Day, On = true }, 12, 20);
        var doll = AddTE(new TEDisplayDoll(), 22, 20);
        doll._equip[0] = new Item { type = 2, stack = 1, prefix = 0 };
        doll._dyes[0] = new Item { type = 3, stack = 1, prefix = 0 };
        doll._misc[0] = new Item { type = 8, stack = 1, prefix = 0 };
        doll._pose = 1;
        var rack = AddTE(new TEWeaponsRack(), 10, 20);
        rack.item = new Item { type = 4, stack = 1, prefix = 81 };
        var hat = AddTE(new TEHatRack(), 16, 20);
        hat._items[0] = new Item { type = 2, stack = 1, prefix = 0 };
        var platter = AddTE(new TEFoodPlatter(), 18, 20);
        platter.item = new Item { type = 8, stack = 1, prefix = 0 };
        AddTE(new TETeleportationPylon(), 14, 20);
        var jar = AddTE(new TEDeadCellsDisplayJar(), 28, 20);
        jar.item = new Item { type = 2, stack = 1, prefix = 0 };
        SetTile(28, 20, 698);
        var kite = AddTE(new TEKiteAnchor(), 24, 20);
        kite.SetItemType(2);
        var critter = AddTE(new TECritterAnchor(), 26, 20);
        critter.SetItemType(3);

        // --- pressure plates / town rooms / bestiary / creative -------------------------
        PressurePlateHelper.PressurePlatesPressed = new Dictionary<Point, bool[]>();
        PressurePlateHelper.PressurePlatesPressed[new Point(4, 23)] = new bool[255];
        PressurePlateHelper.PressurePlatesPressed[new Point(5, 23)] = new bool[255];
        WorldGen.TownManager.SetRoom(22, 6, 20);
        Terraria.Main.BestiaryTracker.Kills.SetKill("BlueSlime", 3);
        Terraria.Main.BestiaryTracker.Kills.SetKill("Guide", 1);
        Terraria.Main.BestiaryTracker.Sights.AddSight("GreenSlime");
        Terraria.Main.BestiaryTracker.Sights.AddSight("Merchant");
        Terraria.Main.BestiaryTracker.Chats.AddChat("Guide");
        CreativePowerManager.Initialize();
        CreativePowerManager.Instance.GetPower<CreativePowers.FreezeTime>().SetPowerInfo(true);
        CreativePowerManager.Instance.GetPower<CreativePowers.StopBiomeSpreadPower>().SetPowerInfo(true);

        // --- header/flags state (all literals) --------------------------------------------
        Terraria.Main.spawnTileX = 20; Terraria.Main.spawnTileY = 18;
        Terraria.Main.worldSurface = 12.0; Terraria.Main.rockLayer = 20.0;
        Terraria.Main.time = 12345.0; Terraria.Main.dayTime = true;
        Terraria.Main.moonPhase = 3; Terraria.Main.moonType = 2;
        Terraria.Main.hellBackStyle = 1;
        Terraria.Main.bloodMoon = true; Terraria.Main.eclipse = false;
        Terraria.Main.hardMode = true;
        NPC.downedBoss1 = true; NPC.downedSlimeKing = true; NPC.downedGoblins = true;
        NPC.savedGoblin = true; NPC.downedDeerclops = true; NPC.unlockedSlimeRainbowSpawn = true;
        Terraria.Main.invasionSize = 0; Terraria.Main.invasionType = 0;
        Terraria.Main.raining = true; Terraria.Main.rainTime = 500; Terraria.Main.maxRaining = 0.5f;
        Terraria.Main.sundialCooldown = 3; Terraria.Main.moondialCooldown = 2;
        Terraria.Main.fastForwardTimeToDawn = true;
        WorldGen.shadowOrbSmashed = true; WorldGen.shadowOrbCount = 2; WorldGen.altarCount = 1;
        WorldGen.crimson = false;
        WorldGen.corruptBG = 1; WorldGen.jungleBG = 2; WorldGen.hallowBG = 3;
        WorldGen.crimsonBG = 4; WorldGen.mushroomBG = 5;
        // W4-c44: the banner counts come from the REAL kill chain, not shim
        // setters. 50 kills of the Angler Fish (NPCID 102 -> NPCtoBanner = 1,
        // BannerSystem.cs:407-408) run the extracted AddNPCKillBy/AddKill/
        // AddClaimableBanner (:231-278): KillsToBanner[BannerToItem(1)=1615]
        // is the default 50 (ItemID.cs:1303-1304), so the 50th kill lands
        // killCount[1]=50 + claimable[1]=1 and broadcasts the milestone
        // announcement through the extracted NetworkText/ChatHelper/Lang
        // bodies into the Main.NewTextMultiline shim sink.
        Terraria.Main.player = new Player[256]; // Main.cs:67020's Initialize shape
        for (int i = 0; i < Terraria.Main.player.Length; i++) Terraria.Main.player[i] = new Player();
        Terraria.Main.player[0].name = "Golden";
        for (int k = 0; k < 50; k++)
            BannerSystem.AddNPCKillBy(NPCID.AnglerFish, 0);
        if (BannerSystem.GetKillCount(1) != 50 || BannerSystem.GetClaimableBannerCounts()[1] != 1)
            throw new Exception("the kill chain did not land killCount[1]=50 / claimable[1]=1");
        string expectedAnnouncement = "Golden has defeated the 50th Angler Fish!";
        if (Terraria.Main.FixtureChatSink.Count != 1 || Terraria.Main.FixtureChatSink[0] != expectedAnnouncement)
            throw new Exception("the milestone announcement did not fire exactly once: "
                + string.Join(" | ", Terraria.Main.FixtureChatSink));
        // Tree-tops: the variation rides the REAL Save -> Load round-trip
        // (TreeTopsInfo.cs:47-68), and the pre-211 arm's CopyExistingWorldInfo
        // (:222-237) is exercised too; the final state stays variation 2.
        WorldGen.TreeTops.SetVariation(0, 2);
        byte[] treeTopBytes;
        using (var ms = new MemoryStream())
        {
            using (var w = new BinaryWriter(ms)) WorldGen.TreeTops.Save(w);
            treeTopBytes = ms.ToArray();
        }
        using (var ms = new MemoryStream(treeTopBytes))
        using (var r = new BinaryReader(ms))
            WorldGen.TreeTops.Load(r, 326);
        if (WorldGen.TreeTops.GetTreeStyle(0) != 2)
            throw new Exception("the tree-top Save/Load round-trip lost variation 2");
        using (var r = new BinaryReader(new MemoryStream()))
            WorldGen.TreeTops.Load(r, 210); // <211: nothing read, CopyExistingWorldInfo
        if (WorldGen.TreeTops.GetTreeStyle(0) != Terraria.Main.treeStyle[0]
            || WorldGen.TreeTops.GetTreeStyle(4) != WorldGen.corruptBG
            || WorldGen.TreeTops.GetTreeStyle(12) != WorldGen.underworldBG)
            throw new Exception("the pre-211 tree-top load did not copy the existing world info");
        using (var ms = new MemoryStream(treeTopBytes))
        using (var r = new BinaryReader(ms))
            WorldGen.TreeTops.Load(r, 326); // restore the fixture state
        WorldGen.SavedOreTiers.Copper = 7; // the real cctor default, restated
        WorldGen.Manifest = new WorldManifest(); // empty -> the null-field JSON
        // The manifest JSON is the REAL WorldManifest.Serialize (Newtonsoft,
        // the verbatim Generated/WorldManifest.cs) - pin its exact shape.
        if (WorldGen.Manifest.Serialize() != "{\"GenPassResults\":[],\"Version\":null,\"GitSHA\":null,\"FinalHash\":null}")
            throw new Exception("the empty-manifest JSON drifted: " + WorldGen.Manifest.Serialize());

        // --- the REAL save -------------------------------------------------------------------
        string outDir = args.Length != 0 ? args[0] : ".";
        Directory.CreateDirectory(outDir);
        WorldFile.SetTempToOngoing(); // WorldFile.cs:1040, as _SaveWorld does
        byte[] wld;
        using (var ms = new MemoryStream())
        {
            using (var w = new BinaryWriter(ms))
            {
                WorldFile.SaveWorld_Version2(w); // WorldFile.cs:1150
            }
            wld = ms.ToArray();
        }
        File.WriteAllBytes(Path.Combine(outDir, "golden.wld"), wld);

        // --- the REAL player save ----------------------------------------------------------------
        var player = new Player
        {
            name = "Golden",
            difficulty = 0,
            hair = 1,
            skinVariant = 0,
            statLife = 100,
            statLifeMax = 100,
            statMana = 20,
            statManaMax = 20,
        };
        player.inventory[0] = new Item { type = 3507, stack = 1, prefix = 0 }; // copper shortsword
        player.inventory[1] = new Item { type = 3509, stack = 1, prefix = 0 }; // copper axe
        player.inventory[2] = new Item { type = 3506, stack = 1, prefix = 0 }; // copper pickaxe
        player.inventory[3] = new Item { type = 2, stack = 50, prefix = 0 };
        player.inventory[3].favorited = true;
        var pfd = new PlayerFileData { Player = player };
        pfd.Metadata = FileMetadata.FromCurrentSettings(FileType.Player);
        pfd.SetPlayTime(TimeSpan.Zero);
        Terraria.Main.myPlayer = 0;
        byte[] plain;
        using (var ms = new MemoryStream())
        {
            using (var w = new BinaryWriter(ms))
            {
                // Player.InternalSavePlayerFile's prologue (Player.cs:54738+).
                w.Write(326);
                pfd.Metadata.Write(w);
                Player.SerializeForFixture(pfd, player, w);
            }
            plain = ms.ToArray();
        }
        // The real envelope: AES-CBC, key = IV = UTF-16LE("h3y_gUyZ"), PKCS7.
        // S1-J-5c: InternalSavePlayerFile (Player.cs:54734-54742) uses a bare
        // `new RijndaelManaged()` — default Padding = PKCS7 — and
        // FlushFinalBlock appends the pad. (The harness previously zero-padded
        // with PaddingMode.None, mirroring the READ path's :55074 setting;
        // that was a harness bug, not the vanilla write envelope.)
        byte[] key = Encoding.Unicode.GetBytes("h3y_gUyZ");
        byte[] cipher;
        using (var aes = Aes.Create())
        {
            aes.Key = key; aes.IV = key;
            aes.Mode = CipherMode.CBC; aes.Padding = PaddingMode.PKCS7;
            using var enc = aes.CreateEncryptor();
            cipher = enc.TransformFinalBlock(plain, 0, plain.Length);
        }
        File.WriteAllBytes(Path.Combine(outDir, "golden.plr"), cipher);

        Console.WriteLine($"golden.wld  {wld.Length} bytes");
        Console.WriteLine($"golden.plr  {cipher.Length} bytes (plaintext {plain.Length})");
        using (var ms = new MemoryStream(wld))
        using (var r = new BinaryReader(ms))
        {
            int version = r.ReadInt32();
            r.ReadBytes(20);
            short n = r.ReadInt16();
            Console.WriteLine($"version {version}, {n} sections");
            for (int i = 0; i < n; i++) Console.WriteLine($"  pointer[{i}] = {r.ReadInt32()}");
        }

        T AddTE<T>(T ent, int x, int y) where T : TileEntity
        {
            ent.ID = teId++;
            ent.Position = new Point16(x, y);
            ent.type = (byte)TeTypeId<T>();
            TileEntity.Add(ent);
            return ent;
        }

        // The importance table, parsed from the REAL decompiled Main.cs: the
        // direct `Main.tileFrameImportant[n] = true` assignments, the for-loop
        // ranges, and the AddEchoFurnitureTile(n) helper calls.
        static bool[] ParseFrameImportant(string mainCsPath)
        {
            var table = new bool[TileID.Count];
            if (mainCsPath == null)
            {
                mainCsPath = Path.Combine(AppContext.BaseDirectory,
                    "..", "..", "..", "..", "..", "decompiled", "Terraria", "Main.cs");
            }
            if (!File.Exists(mainCsPath))
                throw new FileNotFoundException("Main.cs not found for tileFrameImportant parse", mainCsPath);
            string src = File.ReadAllText(mainCsPath);
            foreach (Match m in Regex.Matches(src, @"Main\.tileFrameImportant\[(\d+)\]\s*=\s*true"))
                table[int.Parse(m.Groups[1].Value)] = true;
            foreach (Match m in Regex.Matches(src, @"for \(int (\w+) = (\d+); \1 <= (\d+); \1\+\+\)\s*\{\s*Main\.tileFrameImportant\[\1\] = true;"))
                for (int i = int.Parse(m.Groups[2].Value); i <= int.Parse(m.Groups[3].Value); i++)
                    table[i] = true;
            foreach (Match m in Regex.Matches(src, @"Main\.AddEchoFurnitureTile\((\d+)\)"))
                table[int.Parse(m.Groups[1].Value)] = true;
            return table;
        }

        static void SetTile(int x, int y, ushort type, short fx = 0, short fy = 0)
        {
            Tile t = Terraria.Main.tile[x, y];
            t.type = type;
            t.active(true);
            t.frameX = fx;
            t.frameY = fy;
        }

        // TileEntity.type ids (TileEntity.cs Read's switch / the TE classes).
        static int TeTypeId<T>() where T : TileEntity => typeof(T) switch
        {
            var t when t == typeof(TETrainingDummy) => 0,
            var t when t == typeof(TEItemFrame) => 1,
            var t when t == typeof(TELogicSensor) => 2,
            var t when t == typeof(TEDisplayDoll) => 3,
            var t when t == typeof(TEWeaponsRack) => 4,
            var t when t == typeof(TEHatRack) => 5,
            var t when t == typeof(TEFoodPlatter) => 6,
            var t when t == typeof(TETeleportationPylon) => 7,
            var t when t == typeof(TEDeadCellsDisplayJar) => 8,
            var t when t == typeof(TEKiteAnchor) => 9,
            var t when t == typeof(TECritterAnchor) => 10,
            _ => throw new Exception("unknown TE"),
        };
    }
}