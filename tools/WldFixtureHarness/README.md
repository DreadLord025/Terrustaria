# WldFixtureHarness — the S1-J-3 golden `.wld`/`.plr` fixture generator

Writes `assets/test/golden.wld` and `assets/test/golden.plr` through the **real
decompiled save path** of Terraria 1.4.5.8, so the Rust port can be compared
byte-for-byte against C#-written files (the S1-J-1 / S1-A-1 oracle pattern: the
decompile is the source, only the global lookups become shim state).

## How it works

* `extract.ps1` slices the save-path method bodies **verbatim** out of
  `decompiled/` into `Generated/*.cs`:
  `WorldFile.SaveWorld_Version2` + every `Save*` section writer +
  `SetTempToOngoing` (`WorldFile.cs:1040-1758`), `Player.Serialize` +
  `SaveTemporaryItemSlotContents`, `Tile` (minus `SmoothSlope`, which is
  graphics/network surgery, never on the save path), `TileEntity.Write` +
  every per-kind `WriteExtraData`, the bestiary/creative/town/banner/DD2/
  spawn/treetops `Save` bodies, `Item.Serialize`, `EquipmentLoadout.Serialize`
  and the small whole files (`BitsByte`, `Point16`, `FileMetadata`, `FileType`,
  `SetFactory`, `WorldManifest`, `GenPassResult`, ...).
  **W4-c44** extends the extraction to the state-producing chains behind the
  four unowned header fields: `BannerSystem.Load`/`AddNPCKillBy`/`AddKill`/
  `AddClaimableBanner`/`BannerToItem`/`NPCtoBanner`/`BannerToNPC`/
  `GetKillCount`/`GetClaimableBannerCounts`/`Clear`/the static cctor
  (`BannerSystem.cs:134-278,341-1457`), `TreeTopsInfo.Load`/
  `CopyExistingWorldInfo`/`GetTreeStyle` (`TreeTopsInfo.cs:56-94,222-237`),
  `NetworkText`'s construction + `ToString` path
  (`NetworkText.cs:8-63,115-157`), `Lang.GetNPCName` (`Lang.cs:180-191`) and
  the announcement sink `ChatHelper.BroadcastChatMessage`/`…As`/
  `DisplayMessage`/`ShouldCacheMessage` (`ChatHelper.cs:46-128`) +
  `NameTagHandler.GenerateTag` (`NameTagHandler.cs:13-16`).
* **One — and only one — textual substitution** is made:
  `DateTime.UtcNow` → `FixtureClock.UtcNow` (a pinned 2026-11-18T00:00:00Z).
  The vanilla writers stamp the wall clock (`SaveWorldFlags:1251`,
  `Player.Serialize:54909`); a reproducible fixture cannot carry it.
* `Shims/*.cs` declares the global statics the bodies read (`Main.*`, `NPC.*`,
  `WorldGen.*`, the `WorldFile._temp*` snapshot fields, XNA's
  `Point`/`Vector2`/`Color`, ...) at the C#'s types and defaults. No
  serialization logic lives in the shims.
* `Program.cs` builds the smallest real world (a fixed 40×30 tile array, every
  value a literal, **no genRand**) and a fresh player, then runs the extracted
  bodies in the C#'s own call shape (`SetTempToOngoing` →
  `SaveWorld_Version2` into a `MemoryStream`; version 326 + `FileMetadata.Write`
  + `Player.Serialize`, then the real AES-CBC envelope, key = IV =
  UTF-16LE `"h3y_gUyZ"`, **PKCS7** padding — the bare `new RijndaelManaged()`
  of `InternalSavePlayerFile`, `Player.cs:54734-54742`; S1-J-5c fixed the
  harness's earlier zero-pad, which had mirrored the *read* path's
  `PaddingMode.None` at `:55074`).
* `Main.tileFrameImportant` is parsed from the **real** `decompiled/Terraria/
  Main.cs` at runtime (direct assignments, for-loop ranges,
  `AddEchoFurnitureTile` calls) — the fixture's importance bitmap is the game's
  own table, not a hand transcription.

## Regenerate

```powershell
powershell -File tools/WldFixtureHarness/extract.ps1
dotnet run --project tools/WldFixtureHarness -- assets/test
```

Two runs must produce byte-identical files (the clock is pinned). The Rust
tests `s1_j_3_*` in `crates/tr-core/src/wld/mod.rs` and
`crates/tr-app/src/player_file.rs` pin the result;
`w4_c44_the_fixture_banner_treetop_manifest_state_matches_the_csharp_bodies`
walks the flags block and pins the four unowned payloads against the port's
banner tables. W4-c44 verified: two consecutive regens left `golden.wld`
SHA256 `5C32D867…F162A9` and `golden.plr` `13369854…A9E1A3E3` unchanged.

## Fixture coverage

* **tiles**: RLE runs (1-byte count), framed tiles, tile paint, wall + wall
  paint, 16-bit wall id (300), water/lava/honey/shimmer, the four wire colours
  singly and combined, half-brick, two slopes, actuator, inactive, the four
  coatings (invisible/fullbright block+wall), the four no-batch kinds
  (423/520/723/724, `TileID.cs:484`), chest 2×2 (21) and sign (55) anchors.
* **chests**: two records — named + empty-named, a prefixed item, a stack,
  and a banned item (58 Heart → the `(short)0` encoding, `ItemID.cs:345`).
* **signs**: two text records. **npcs**: two town records (variation index,
  homeless + homelessDespawn), one tower (422), the shimmered list.
* **tile entities**: one of every kind the port models (dummy, item frame,
  logic sensor, display doll, weapons rack, hat rack, food platter, pylon,
  dead-cells jar, kite anchor, critter anchor).
* **pressure plates**: two. **town rooms**: one. **bestiary**: kills/sights/
  chats. **creative powers**: all six persistent registrations.
* **flags**: a broad non-default sweep (blood moon, hardmode, downed flags,
  rain, sundial/moondial, banner counts, tree-top variation, ore tiers, the
  empty-manifest JSON). **W4-c44**: the four unowned fields are produced by
  the REAL C# bodies — 50 `BannerSystem.AddNPCKillBy(102, 0)` kills land
  `killCount[1]=50` / `claimable[1]=1` through `AddKill`/`AddClaimableBanner`
  (the milestone announcement fires once through the extracted
  `NetworkText`/`ChatHelper`/`Lang` chain into the `Main.NewTextMultiline`
  shim sink, pinned by the driver); the tree-top variation rides the real
  `TreeTopsInfo.Save`→`Load` round-trip (the pre-211 `CopyExistingWorldInfo`
  arm is exercised too); the manifest JSON is the real
  `WorldManifest.Serialize` (Newtonsoft) output, shape-pinned by the driver.
  The fixture bytes are unchanged by this (regen is SHA256-stable).
* **.plr**: the fresh-player defaults plus four inventory items (one
  favorited), pinned play time zero.

## Known divergences (documented, not faked)

The port's `.wld` re-write of the fixture is byte-identical for sections 2..10
and the footer; the header/flags block differs in exactly four unowned spots —
the banner kill/claimable counts (`killCount[1]`/`claimable[1]`, no `.wld`
store), the tree-top variations (`treeTop[0]`, no store) and the world-gen
manifest JSON (`manifest`, the port writes the empty string) — pinned
field-by-field by
`s1_j_3_the_header_flags_divergences_are_exactly_the_unowned_fields`.
W4-c44 makes the fixture side of those four divergences run through the real
C# state chains (see "Fixture coverage") and adds
`w4_c44_the_fixture_banner_treetop_manifest_state_matches_the_csharp_bodies`,
which pins the fixture's kill/claimable/tree-top/manifest payloads against
`tr_core::bestiary`'s `NPCtoBanner`/`BannerToItem`/`KillsToBanner` ports. The
port still writes its defaults there (the fields stay unowned); only the
fixture's provenance changed, so **`golden.wld` needs no regeneration**.

S1-J-3b closed the rest of the header/flags remainder: `CreationTime`, the
file-faithful `spawnTileY` (the old derived-spawn override removed), the seven
ore tiers (+ the post-load `CheckSavedOreTiers` re-derivation), the saved-NPC
flags (wizard/mech/angler/stylist/taxCollector/golfer), the angler quest index,
the cultist/Moon Lord/Queen Slime kills, the four tower downeds/actives + the
apocalypse phase (with the load-time shield restore, `LoadWorldFlags:2235-2250`),
and the party-girl/princess spawn unlocks. The fixture keeps all of those at
their C# defaults (`Shims/Globals.cs`), so the wiring is byte-identity-preserving
and **`golden.wld` needs no regeneration**. The `.plr` AES-envelope change
(PKCS7) belongs to the sibling `task/S1-J-5c-plr-crypto`; its `golden.plr` regen
is that task's, not this one's.

See ANALYSIS.md "The C#-written golden fixture (S1-J-3)".