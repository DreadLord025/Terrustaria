# 37 — ReLogic libs (content pipeline, XNA, JSON)

> Wave 4 · researched 2026-11-12 · sources: decompiled-libs/ReLogic + decompiled/Terraria + crates/tr-content, tr-render, tr-app + ROADMAP/ANALYSIS/PORT_MAP

## Scope

`decompiled-libs/ReLogic` is the engine library that sits *under* Terraria (168 `.cs`
files, ~40 000 lines; `docs/ANALYSIS.md:12`). Namespaces and sizes:

| Namespace | Files / lines | Responsibility |
| --- | --- | --- |
| `ReLogic.Content` | 17 / ~900 | asset repository, loaders, asset state, hot-reload watcher |
| `ReLogic.Content.Readers` | 3 / 114 | `XnbReader` (`.xnb`), `PngReader` (`.png`) |
| `ReLogic.Content.Sources` | 4 / 322 | name→path resolution: file system, XNA, XNA-direct, zip |
| `ReLogic.Graphics` | 5 / 513 | `DynamicSpriteFont` + reader, font page, debug drawer |
| `ReLogic.Text` | 7 / 323 | word wrapping (`WrappedTextBuilder`, `StringReaderWrapExtension`), glyph metrics |
| `ReLogic.Utilities` | 10 / 1 052 | `Crc32`, `MultiTimer`, `SlotId`/`SlotVector`, `Vector2D`, `TypeInstanceCollection`, `ReinterpretCast` |
| `ReLogic.Threading` | 4 / 176 | `FastParallel`, `AsyncActionDispatcher` |
| `ReLogic.OS*` | 21 / ~600 | window service, clipboard, path service per OS |
| `ReLogic.Peripherals.RGB*` | ~100 / ~12 000 | Chroma engine + Razer/Corsair/Logitech/SteelSeries SDKs |
| `ReLogic.Localization.IME*` | 6 | IME input (Windows) |
| `ReLogic.IO`, `ReLogic.Reflection` | 2 / 176 | console mirror, `IdDictionary` |

Responsibility in one paragraph: ReLogic is the *asset and platform* layer of Terraria
— it resolves content names to streams, keeps an `Asset<T>` repository with load
states and an optional file watcher, defines the game's custom binary sprite font,
wraps the OS for window/clipboard/path, and drives RGB peripherals. It is **not** the
renderer, the audio engine (that is `Terraria.Audio`), nor the UI (`Terraria.UI`).

Three boundary facts that drive the whole module:

1. **ReLogic does not parse XNB.** `ReLogic.Content.Readers.XnbReader` is a 55-line
   shim over FNA's `Microsoft.Xna.Framework.Content.ContentManager`
   (`XnbReader.cs:10` `InternalContentManager : ContentManager`, `:24`
   `ReadAsset<T>("XnaAsset", null)`, `:29` `OpenStream` returns the injected stream).
   The container format, the LZX decoder and the `Texture2D`/`Effect`/`Model`/
   `Music` content readers live in **FNA/XNA**, which is not in `decompiled-libs`.
   Hence the Rust XNB stack is a *from-scratch reimplementation*, not a port of
   ReLogic code.
2. **There is no JSON serializer in ReLogic.** A search of all 168 files for `Json`
   returns nothing; Terraria uses **Newtonsoft.Json** on the game side
   (`Terraria.Preferences`, `Terraria.Localization.LanguageManager`,
   `Terraria.Initializers.AssetInitializer:59` `Main.Configuration.Get("ResourcePacks", new JArray())`).
   The INDEX row "ReLogic libs (content pipeline, XNA, JSON)" is therefore a
   mislabel for the JSON half, and "XNA" belongs to FNA, not to this folder.
3. **There is no `UnifiedRandom` in ReLogic** either (grep over `decompiled-libs`
   returns zero hits; `Terraria.Initializers.AssetInitializer.cs:66` calls
   `new UnifiedRandom()` from the Terraria assembly). Module 35 owns a Terraria
   class, not a ReLogic one.

The Terraria-side glue (not ReLogic, but required to understand the seam) is
`Terraria.Initializers.AssetInitializer.CreateAssetServices`
(`AssetInitializer.cs:23-35`): it registers `PngReader(".png")` (`:24`),
`XnbReader(".xnb")` (`:25`), `FxReader(".fx")` (`:26`, the only Terraria-side
`IAssetReader`, in `Terraria/Testing/FxReader.cs`), builds
`AsyncAssetLoader(assetReaderCollection, 20)` (`:27`) and
`new AssetRepository(new AssetLoader(assetReaderCollection), asyncAssetLoader)` (`:31`).
`Terraria.Main.AssetSourceController` (`Main.cs:233`, built at `Main.cs:10782`)
switches content sources for resource packs.

## Key entities

- `ReLogic.Content.AssetRepository` (`ReLogic.Content/AssetRepository.cs:9-411`) — the
  content manager. `Request<T>(name, mode)` (`:82`), `SetSources` (`:64`),
  `LoadAsset` (`:205`), `TryLoadingAsset` (`:228`), `TransferCompletedAssets` (`:107`),
  `EnableAssetWatcher` (`:117`), `UpdateWatchedAsset` (`:144`),
  `ReloadAssetsIfSourceChanged` (`:189`), `FindSourceForAsset` (`:374`).
  It owns `Dictionary<string, IAsset> _assets` (`:11`) and the
  `_typeSpecificReloadActions` / `_typeSpecificUpdateActions` tables (`:15-17`).
- `ReLogic.Content.Asset<T>` (`ReLogic.Content/Asset.cs`, 75 lines) — one asset with a
  state (`NotLoaded`/`Loading`/`Loaded`), its `Source` and its value.
- `ReLogic.Content.AssetLoader` (`AssetLoader.cs:7-33`) — `TryLoad<T>(name, source, out)`
  (`:16`) resolves the extension, `TryLoad<T>(extension, getStream, out)` (`:21`) asks
  `AssetReaderCollection` whether it can read it and then `Read<T>(stream, extension)`.
- `ReLogic.Content.AssetReaderCollection` (27 lines) — extension → reader registry
  (`RegisterReader`), used by `AssetInitializer.cs:23-26` and by
  `Terraria.IO.ResourcePack.cs:110` (`_services.Get<AssetReaderCollection>().Read<Texture2D>(stream, ".png")`).
- `ReLogic.Content.Readers.XnbReader` (`XnbReader.cs:8-49`) — thread-local FNA
  `ContentManager`; **delegates the whole format parse to FNA** (see Scope).
- `ReLogic.Content.Readers.PngReader` (53 lines) — `Texture2D.FromStream` path for `.png`.
- `ReLogic.Content.Sources.*` — `FileSystemContentSource` (97), `XnaContentSource` (92),
  `XnaDirectContentSource` (76), `ZipContentSource` (137): `HasAsset`, `OpenStream`,
  `RejectAsset`, extension mapping.
- `ReLogic.Content.AsyncAssetLoader` (136 lines) — background loading;
  `RequireTypeCreationOnTransfer(typeof(Texture2D)/DynamicSpriteFont/SpriteFont)`
  (`AssetInitializer.cs:28-30`) forces GPU-resource creation back onto the main thread.
- `ReLogic.Content.AssetChangeWatcher` (42) + `FileChangeWatcher` (131) — hot reload of
  changed files (a tModLoader-era feature; the retail game only uses it for resource packs).
- `ReLogic.Graphics.DynamicSpriteFontReader` (`DynamicSpriteFontReader.cs:8-29`) — the
  `.xnb` font reader: `spacing` (f32, `:12`), `lineSpacing` (i32, `:13`),
  `defaultCharacter` (char, `:14`), page count (i32, `:16`), then per page a
  `Texture2D`, `List<Rectangle> glyphs`, `List<Rectangle> padding`, `List<char> characters`,
  `List<Vector3> kerning` (`:20-25`).
- `ReLogic.Graphics.DynamicSpriteFont` (349) / `FontPage` (20) — the font object and its
  per-page glyph tables (`DrawString` metrics).
- `ReLogic.Text.WrappedTextBuilder` (151) + `ReLogic.Text.StringReaderWrapExtension`
  (116) — word wrapping to a max width; `ReLogic.Text.GlyphMetrics` (20) / `IFontMetrics` (7).
- `ReLogic.Utilities.*` — `Crc32` (69), `MultiTimer` (88), `SlotId` (65), `SlotVector` (190),
  `Vector2D` (389), `TypeInstanceCollection` (70), `ReinterpretCast` (57),
  `AttributeUtilities` (30), `XnaExtensions` (9).
- `ReLogic.Threading.FastParallel` (82) / `AsyncActionDispatcher` (79) — parallel-for
  helpers used by the world/lighting loops.
- `ReLogic.OS.Platform` (59) + `PlatformType` (7) — window/clipboard/path abstraction
  per OS (`ReLogic.OS.Base.SDLWindowService`, `ReLogic.OS.Windows.*`, `.Linux.*`, `.OSX.*`).
- `ReLogic.Peripherals.RGB.*` — `ChromaEngine`, `ChromaPipeline`, `RgbDeviceGroup` and the
  four vendor back-ends. Wired on the Terraria side by
  `Terraria.Initializers.ChromaInitializer` and `Terraria.GameContent.ChromaHotkeyPainter`.
- `ReLogic.IO.ConsoleOutputMirror` (91), `ReLogic.Reflection.IdDictionary` (85).

## Dependencies

- Calls into: FNA (`Microsoft.Xna.Framework.Content.ContentManager`, `Texture2D`,
  `GraphicsDevice`), the OS APIs (win32/X11/Cocoa), the RGB vendor SDKs; Newtonsoft.Json
  is *not* called by ReLogic.
- Called by: `Terraria.Initializers.AssetInitializer` (`:23-35`),
  `Terraria.Main.AssetSourceController` (`Main.cs:233`, `:10782`),
  `Terraria.GameContent.AssetSourceController` (`:12`), `Terraria.IO.ResourcePack`
  (`:110`), `Terraria.Testing.FxReader`.
- Shared data structures: `Asset<T>`/`IAsset`, `IAssetRepository`, `IAssetLoader`,
  `IAssetReader`, `IContentSource`, `AssetRequestMode`, `AssetState`, `IRejectionReason`,
  `IContentValidator`.

## Port status

| ReLogic component | Purpose | Rust counterpart | Status | Notes |
| --- | --- | --- | --- | --- |
| `AssetRepository` / `Request<T>` / `Asset<T>` (AssetRepository.cs:9-411) | content manager, lazy per-name loads, load states | none — sheets are loaded by name at boot into one atlas (`tr-app/src/assets.rs::load_static_sheets:148`, `load_world_sheets:83`, `load_world_images:70`) | MISSING (by design) | no mods, no resource packs, no hot reload, no per-asset lazy loading; the memory profile differs from the original |
| `AssetLoader` + `AssetReaderCollection` (AssetLoader.cs:7, 27 lines) | extension→reader dispatch | flattened into direct functions: `tr-content/src/texture.rs::load_texture:33`, `font.rs::load_font:69`, `effect.rs::load_effect_pass_names:13` | PORTED (flattened) | the dispatch key changed from "extension" to "caller intent"; a `.png` in a `.xnb` slot is now a different code path |
| `ReLogic.Content.Readers.XnbReader` (XnbReader.cs:8-49) | hands an `.xnb` stream to FNA's `ContentManager` | `tr-content/src/xnb.rs::XnbFile::read:72` | PORTED (reimplemented) | ReLogic itself contains no container code; the Rust parser is written against the XNB v5 spec + `tools/XnbExtract` (which is **absent from this checkout**) |
| FNA/XNA container parse (outside ReLogic) | XNB v5 header + reader table | `tr-content/src/xnb.rs:72-138` | PORTED | see "Content formats" below |
| FNA/XNA LZX ("XMemCompress") decoder (outside ReLogic) | decompresses compressed `.xnb` | `tr-content/src/lzx.rs` (737 lines: Huffman tables, block decode) | PORTED | `PORT_MAP.md:249` cites `ReLogic/Content/Lzx*` — **no such file exists**; the decoder came from MonoGame/Xbox |
| XNA `Texture2DReader` — `SurfaceFormat.Color` | uncompressed RGBA surfaces | `tr-content/src/texture.rs::load_texture:33`, `RgbaImage:12` | PORTED | XNA `Color` is RGBA in memory, not BGRA (`ANALYSIS.md:1466-1467`) |
| XNA `Texture2DReader` — DXT1/3/5 | block-compressed surfaces | `tr-content/src/dxt.rs::dxt1_block:122`, `dxt3_block:169`, `dxt5_block:208` | PARTIAL | 23 retail `.xnb` files still fail (multi-mip / unhandled surface format, `ANALYSIS.md:1408-1411`) |
| `DynamicSpriteFontReader` (DynamicSpriteFontReader.cs:8-29) | reads the `.xnb` font (pages, glyphs, padding, chars, kerning) | `tr-content/src/font.rs::load_font:69`, `read_page:120`, `read_rect_list:237`, `read_char_list:268`, `read_vector3_list:304` | PORTED (with a documented divergence) | the shipped binary differs from `decompiled-libs`: UTF-8 char list, extra page-header fields (`ANALYSIS.md:1470-1475`); the Rust parser discovers the tables *structurally* instead of following the decompiled field order |
| `DynamicSpriteFont` / `FontPage` metrics + `DrawString` | glyph metrics, kerning, line spacing | `tr-render/src/font_atlas.rs::FontAtlas::build:114`, `FontPageData:24`, `PageRect:33`; `tr-app/src/assets.rs::UiFont:991`, `glyph:999`, `build_ui_font:1139` | PARTIAL | metrics are ported and the text draws; the `DrawString` line-break/kerning path is approximated |
| `ReLogic.Text.WrappedTextBuilder` (151) + `StringReaderWrapExtension` (116) | word wrap for long labels/tooltips | none — no `fn …wrap…` helper exists in `crates/` (grep over all `.rs`) | MISSING | long item names/tooltips will not wrap like the original |
| `PngReader` (53) | `.png` textures (resource packs) | `tr-app/src/assets.rs::load_png:962`, `load_misc_png:966`; `image` crate in the workspace deps | PORTED (as a path, not a reader) | the `image` crate replaces GDI+/FNA PNG decoding |
| `FxReader` (Terraria side) + `.fx` assets | compiled effects | `tr-content/src/effect.rs::load_effect_pass_names:13`, `pass_names_from_payload:27` | PARTIAL | only the **pass-name table** is parsed (22 passes, `ANALYSIS.md:1106`); the effect bytecode is not executed — the Rust filter is a hand-written WGSL mirror (`tr-render/src/post.rs`) |
| Music/sound assets (XACT `.xsb`/`.xwb`, `TerrariaMusic.xgs`) | music + SFX banks | `tr-content/src/xact.rs::parse_sound_bank:332`, `parse_wave_bank_entries:337`, `XactWaveBank::open:182`, `XactSoundBank::load:271`, `get_cue:318`, `music_track:327`; `adpcm.rs::decode_msadpcm:31`; `sound.rs::load_sound:20`; playback via `rodio` | PARTIAL | readers are ported; playback/mixing parity is module 32's subject |
| `AsyncAssetLoader` (136) + `RequireTypeCreationOnTransfer` | background loading, main-thread GPU creation | none — all content is decoded synchronously at boot | MISSING (not needed) | the Rust port has no background content thread |
| `AssetChangeWatcher` / `FileChangeWatcher` (42/131) | hot reload of changed assets | none | MISSING (not needed) | tModLoader/resource-pack feature |
| `IContentSource` family (322 lines) | name→path resolution, zip/resource-pack overlays | `tr-app/src/assets.rs::content_images_dir:943`, `content_dir:939` + `tr-content` path helpers | MISSING (replaced) | resolution is a fixed directory layout, no overlays |
| `ReLogic.Graphics.BasicDebugDrawer` (77) / `IDebugDrawer` (7) | debug line drawing | `tr-app/src/render/debug.rs` (different feature) | N/A | not a parity item |
| `ReLogic.OS.*` (window/clipboard/path, ~600 lines) | platform abstraction | `winit` + the Rust platform layer in `crates/tr-app` | REPLACED | XNA's window/clipboard behaviour is replaced by native crates |
| `ReLogic.Peripherals.RGB.*` (~100 files) | Chroma RGB peripherals | none | MISSING (not needed) | feature gap, not a pixel-parity gap |
| `ReLogic.Threading.FastParallel` / `AsyncActionDispatcher` | parallel-for helpers | `rayon` (workspace dep) | REPLACED | different threading model; the fixed-step order is the port's invariant |
| `ReLogic.Utilities.*` (1 052 lines) | `Crc32`, `MultiTimer`, `SlotVector`, `Vector2D`, `TypeInstanceCollection`, `ReinterpretCast` | not ported individually; Rust std + `anyhow` cover the needs | N/A | no parity requirement |
| `ReLogic.Reflection.IdDictionary` (85) | id→instance registry | none | N/A | |
| JSON (Newtonsoft, Terraria side) | preferences / localization / resource packs | `serde_json` (workspace dep, `crates/tr-app`) | REPLACED | not a ReLogic component — see Scope |

## Content formats the port depends on

**XNB v5 container** (`crates/tr-content/src/xnb.rs`):

| Field | Offset | Rust | Meaning |
| --- | --- | --- | --- |
| magic `"XNB"` | 0..3 | `xnb.rs:73` | reject anything else |
| platform byte (`'w'`) | 3 | `xnb.rs:76` | ignored (`_platform`) |
| format version | 4 | `xnb.rs:78` | must be `5`, else bail |
| flags (`0x80` = compressed) | 5 | `xnb.rs:81` | |
| int32 payload/file size | 6..10 | `xnb.rs:82` | end offset of the body |
| int32 decompressed size | 10..14 | `xnb.rs:86-90` | only present when compressed |
| body start | 14 (compressed) / 10 (plain) | `xnb.rs:11-12`, `:91-95` | `HEADER_SIZE` / `PLAIN_HEADER_SIZE` |
| reader table | body | `xnb.rs:109-119` | 7-bit-encoded count, then per reader: 7-bit length + UTF-8 name + i32 version |
| shared-resource count | body | `xnb.rs:120-123` | must be `0`, else bail |
| type id | body | `xnb.rs:124`, `asset_reader:130` | 1-based index into the reader table; `0` = shared resource |

- **Compression is LZX, not zlib/deflate.** Terraria's build uses the Xbox
  `XMemCompress` variant — "LZX, 16-bit window, headered blocks" (`ANALYSIS.md:1403-1405`)
  — decoded by `crates/tr-content/src/lzx.rs`. The prompt's "LZX/zlib" is half right:
  a zlib/deflate path is neither implemented nor needed (the retail files are LZX).
- The header is 10 bytes plain and **14 bytes compressed**; the Rust constant
  `HEADER_SIZE = 14` (`xnb.rs:11`) is the compressed case, `PLAIN_HEADER_SIZE = 10`
  (`:12`) the other.
- Surface formats supported: `Color` (RGBA), DXT1, DXT3, DXT5 (`dxt.rs`); anything else
  (multi-mip, other DXT/BC variants) is still unsupported — 23 retail files fail
  (`ANALYSIS.md:1408-1411`).
- **No separate sprite-sheet "meta" file exists.** The frame grid is a convention, not
  data: `Tiles_*` use a 18 px stride for 16×16 art, `Wall_*` a 36 px cell with an 8 px
  inset (`ANALYSIS.md:1413-1418`), mirrored in Rust by
  `tr-render/src/atlas.rs::SheetLayout::geometry:27` (`Tile = (18,0)`, `Wall = (36,8)`).
  The only "meta" that *is* data is the font's page tables inside the `.xnb`.
- **Raw file assets**: this build ships exactly three non-`.xnb` files under
  `assets/Content` — `Sound Bank.xsb`, `Wave Bank.xwb`, `TerrariaMusic.xgs` (15 994
  `.xnb` beside them). Everything else in the retail content dir is an XNB container;
  Terraria's raw JSON (preferences, localization, resource packs) lives outside
  `Content/`.
- The Rust pipeline reads the `.xnb` files **at runtime** from `assets/Content`
  (`ANALYSIS.md:1462-1466`: "no pre-extracted PNGs and no fallback"), and the audit
  checkout still ships the retail `.xnb`s (not extracted PNGs).

## Claim re-verification

| Claim | Verdict | Evidence |
| --- | --- | --- |
| AGENTS.md rendering invariant: "The renderer draws one atlas through one instanced sprite pipeline" | **PASS** (with 3 caveats) | `tr-render/src/atlas.rs::Atlas::build:64`/`build_ref:83` shelf-packs every sheet into one RGBA atlas; `tr-render/src/sprite.rs:1` "Instanced sprite renderer: one atlas, one pipeline, one draw call per layer", `SpriteRenderer:72`, instance ring `:91-93`, `wgpu::VertexStepMode::Instance` `:301-302`, `push_sprite:608`. Caveats: (1) font pages live in a **separate** BC2 texture array (`tr-render/src/font_atlas.rs::FontAtlas:105`), not the atlas; (2) the packed-atlas sampling is not identical to XNA's per-texture sampling — one documented case clamps the bottom UV instead of sampling the neighbouring sheet (`ANALYSIS.md:1180-1182`); (3) the atlas itself, its 2 px padding and nearest filtering (`atlas.rs:7-9`) are port inventions — XNA/SpriteBatch had no atlas, and adding sheets reshuffles the packer (89 edge pixels changed, `ANALYSIS.md:1074-1077`) |
| PORT_MAP.md:248-252 "Content decoding" table (xnb / LZX / XACT / font / atlas rows) | **PARTIAL** | the five Rust files exist and are real (`xnb.rs`, `lzx.rs`, `xact.rs`+`adpcm.rs`+`sound.rs`, `font.rs`+`font_atlas.rs`, `atlas.rs`), but the XACT row is honestly marked "partial" and the font row hides a divergence from `decompiled-libs` (§Deviations 3). The citation `ReLogic/Content/Lzx*` is **wrong**: no LZX file exists in `decompiled-libs` (the decoder is FNA/MonoGame's) |
| ANALYSIS.md:1462-1466 "Content pipeline (`tr-content`)" — original pipeline, no pre-extracted PNGs, no fallback | **PASS** | `tr-content` reads `assets/Content/**.xnb` at runtime; the audit checkout has 15 994 `.xnb` and 0 extracted PNGs under `assets/Content`; `tr-content/src/lib.rs` tests decode retail files directly |
| ANALYSIS.md:1406-1408 "`tools/XnbExtract` (C#/.NET 10) … writes RGBA PNGs" | **STALE** | `tools/` in this checkout contains only `GenTileData/Program.cs`; the pipeline moved to the runtime decoder in `tr-content`. The claim is history, not the current architecture |
| ANALYSIS.md:1470-1475 "The ReLogic in the shipped game differs from `decompiled-libs` in the primitive encoding" | **CONFIRMED / accepted** | the shipped font `.xnb` stores the glyph character list as a UTF-8 string and the page header with extra fields, while `DynamicSpriteFontReader.cs:20-25` reads `List<char>` and a fixed field order; the Rust parser therefore locates the page texture and the tables structurally (`font.rs::read_page:120` and the validators in `font.rs:183-330`) |
| INDEX.md:62 "ReLogic libs (content pipeline, XNA, JSON)" | **PARTIAL / mislabelled** | content pipeline: yes; "XNA": the XNA/FNA types are a *dependency* of ReLogic, not part of `decompiled-libs`; "JSON": no JSON serializer exists in ReLogic (Newtonsoft is a Terraria-side dependency) |

## Tests

Cargo tests covering this module (file::test):

- `crates/tr-content/src/lib.rs::decodes_tiles_deterministically` — FNV-1a pin of
  `Images/Tiles_0.xnb` (288×270, `HASH = 0xdb76_2213_37e7_e205`).
- `crates/tr-content/src/lib.rs::decodes_minimap_and_boss_bar_sheets_at_the_pinned_sizes`
- `crates/tr-content/src/lib.rs::decodes_p6_front_end_sheets_at_the_pinned_sizes`
- `crates/tr-content/src/lib.rs::test_DW_2_3_noise_xnb_decodes`
- `crates/tr-content/src/lib.rs::decodes_flame_and_glow_sheets_at_the_pinned_sizes`
- `crates/tr-content/src/font.rs::decodes_the_ui_fonts`,
  `crates/tr-content/src/font.rs::covers_the_scripts_the_localizations_use`,
  `crates/tr-content/src/font.rs::pins_page0_metrics`
- `crates/tr-content/src/effect.rs::test_DW_2_4_xnb_pass_names_match_initializer`,
  `crates/tr-content/src/effect.rs::test_pass_names_from_payload_pure`,
  `crates/tr-content/src/effect.rs::test_pass_names_from_payload_rejects_bad_size`
- `crates/tr-content/src/xact.rs::mini_format_decodes_the_msadpcm_music_format`,
  `crates/tr-content/src/xact.rs::reads_the_retail_wave_bank_metadata`,
  `crates/tr-content/src/xact.rs::sound_bank_maps_music_cues_to_wave_entries`
- `crates/tr-content/src/adpcm.rs::stereo_block_header_is_field_interleaved`,
  `crates/tr-content/src/adpcm.rs::decodes_retail_music_to_audio`,
  `crates/tr-content/src/adpcm.rs::decoded_music_is_smooth_across_block_boundaries`
- `crates/tr-render/src/atlas.rs::build_ref_matches_build_bit_for_bit` — the world-swap
  packing path must place every sheet exactly where `build` would.
- `crates/tr-app/src/assets.rs::test_the_world_and_static_split_concatenates_to_the_full_list`,
  `crates/tr-app/src/assets.rs::test_the_second_atlas_build_reuses_the_static_half`,
  `crates/tr-app/src/assets.rs::item_icon_uvs_are_normalized_and_atlas_fits`,
  `crates/tr-app/src/assets.rs::test_DW_4_1_item_icons_cover_enumerated_content_files`.

Gap: **no test pins the XNB header itself** (magic / platform / version / flags /
compressed-vs-plain); the current tests only pin decoded sizes and a pixel hash, so a
header-parsing regression would surface as a confusing decode error.

`crates/tr-core/src/binio.rs` (the "unified le-readers") is the *world/save* IO layer,
not an asset reader — ReLogic hands out `Stream`s and never parses game data, so it
has no ReLogic counterpart. (Not inspected line-by-line in this pass.)

## Deviations & risks

1. **No asset repository in the port** — severity: **behavioural** (accepted by design;
   ROADMAP/ANALYSIS does not spell it out for this subsystem). The C# loads every
   asset lazily through `AssetRepository.Request<T>` (`AssetRepository.cs:82`) with
   resource-pack overlays and hot reload; the Rust loads a fixed sheet list at boot.
   Impact: no mods/resource packs, no lazy loading, different memory profile.
2. **The XNB/LZX/DXT stack is a reimplementation, not a port** — severity:
   **behavioural** (mitigated by pixel-hash tests). Since ReLogic delegates to FNA
   (`XnbReader.cs:10-33`), there is no C# in `decompiled-libs` to port 1:1; the Rust
   was validated against the retail files only. Formats not exercised by a test
   (multi-mip, unknown surface formats — the 23 failures in `ANALYSIS.md:1408-1411`)
   can regress silently.
3. **Font parse follows the binary, not `decompiled-libs`** — severity:
   **behavioural/determinism** (documented in `ANALYSIS.md:1470-1475`, but the
   `PORT_MAP.md:252` wording "exact 1.4.5.8 parse" hides it). The Rust
   `font.rs` discovers the page texture and the metric tables *structurally*
   (find-and-validate) instead of following the decompiled field order; if the
   shipped format ever changes shape the parser could mis-locate a table instead of
   failing loudly.
4. **Packed atlas ≠ XNA per-texture sampling** — severity: **pixel** (documented).
   `ANALYSIS.md:1180-1182` (hair row 19 clamps the bottom UV), plus the 2 px padding /
   nearest filter (`atlas.rs:7-9`) and the packer reshuffle when sheets are added
   (`ANALYSIS.md:1074-1077`). Item icons and crafting placeholders are the visible
   consequence of the atlas carrying only the world's used frames
   (`ANALYSIS.md:1040-1041`).
5. **`.fx` effects are only name-parsed** — severity: **behavioural** (visual parity
   not guaranteed per pass). The Rust screen filter mirrors `ScreenShader.fx`'s named
   techniques with a WGSL module and a 22-entry `PASS_TABLE`
   (`ANALYSIS.md:1106`), it does not execute the compiled effect.
6. **Text wrapping is not ported** — severity: **behavioural** (UI). `ReLogic.Text`
   has no Rust counterpart (no `wrap` helper in `crates/`), so long labels/tooltips
   will not break like the original. Not mentioned in ANALYSIS.
7. **Chroma / RGB peripherals not ported** — severity: **none** for pixel parity
   (feature gap only).
8. **Stale/incorrect doc citations** — severity: **cosmetic**:
   `PORT_MAP.md:249` cites `ReLogic/Content/Lzx*` (no such file);
   `ANALYSIS.md:1406-1411` still describes the C# PNG-extraction pipeline and the
   "15 100 / 15 123" count while the runtime decoder replaced it;
   `INDEX.md:62` labels the module "(content pipeline, XNA, JSON)" although ReLogic
   contains neither a JSON serializer nor the XNA types.
9. **Asymmetric evidence in this checkout** — severity: **cosmetic**. The Rust
   `xnb.rs` header comment cites `tools/XnbExtract/Program.cs` as its C# reference,
   but `tools/` here contains only `GenTileData/Program.cs`; the citation cannot be
   re-verified inside this audit tree.

## Task seeds

- **S** — Fix the ReLogic citations in `docs/PORT_MAP.md` (`:249`): replace
  `ReLogic/Content/Lzx*` with "FNA/MonoGame `LzxDecoder` (not in `decompiled-libs`)"
  and add one line stating that ReLogic contains no JSON and no XNA types.
  Also correct `INDEX.md:62`'s "(content pipeline, XNA, JSON)".
  No dependencies.
- **S** — Add a `tr-content` test that pins the XNB *header* of one compressed and one
  plain retail `.xnb` (magic, platform, version 5, flags, sizes) so a header
  regression fails with a clear message instead of a decode error.
  No dependencies.
- **S** — Document the ReLogic ↔ FNA ↔ Rust boundary in `docs/ANALYSIS.md`
  ("Asset pipeline" / "Content pipeline" sections): the XNB/LZX/DXT stack is a
  reimplementation validated against retail files, not a 1:1 port.
  No dependencies.
- **M** — Port `ReLogic.Text.WrappedTextBuilder` + `StringReaderWrapExtension`
  (word wrap with the original's line-breaking rules) into the Rust UI text path
  (`tr-app/src/ui/text.rs`) and check long item names / tooltips against the original.
  Depends on the UI modules (18/20) being in their final shape.
- **M** — Make the font parser's structural discovery explicit: document the
  page-header offsets it assumes, pin them with a test against
  `Fonts/Mouse_Text.xnb` + `Fonts/Item_Stack.xnb`, and align the kerning/line-break
  behaviour with `DynamicSpriteFont.DrawString` (the current metrics are "close").
  No dependencies (font tests already exist in `font.rs`).
- **L** — Effect parity: either decode the `.xnb` effect into a real pass table with
  per-pass parameters, or hand-port the remaining `ScreenShader` passes (22 today) and
  each `ShaderData` family in `Terraria/Graphics/Shaders/*`.
  Depends on the renderer/post pipeline work; large.
- **L** — An `AssetRepository` equivalent in Rust (name → lazy load, `AssetRequestMode`,
  resource-pack overlays) — only if mod/resource-pack support enters scope; the
  current design deliberately skips it.
  Depends on a roadmap decision (no mod support in 1.4.5.8 parity).
- **M** — Cover the 23 unsupported retail textures: extend `dxt.rs`/`texture.rs` with
  multi-mip and the remaining surface formats (`ANALYSIS.md:1408-1411`).
  No dependencies.
