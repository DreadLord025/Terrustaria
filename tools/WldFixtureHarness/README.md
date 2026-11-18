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
  UTF-16LE `"h3y_gUyZ"`, zero pad to 16).
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
`crates/tr-app/src/player_file.rs` pin the result.

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
  empty-manifest JSON).
* **.plr**: the fresh-player defaults plus four inventory items (one
  favorited), pinned play time zero.

## Known divergences (documented, not faked)

The port's `.wld` re-write of the fixture is byte-identical for sections 2..10
and the footer; the header/flags block differs in exactly the unowned fields
(`CreationTime`, derived `spawnTileY`, the seven ore tiers, banner counts,
tree-top variations, the manifest JSON) — pinned field-by-field by
`s1_j_3_the_header_flags_divergences_are_exactly_the_unowned_fields`.
See ANALYSIS.md "The C#-written golden fixture (S1-J-3)".