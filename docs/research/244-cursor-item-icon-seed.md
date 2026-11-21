# 244 — The cursorItemIcon system: census + task-seed + owner contract (W4-g86)

Worker W4-g86 (`task/W4-g86-altar-draw`, the owed-pool `cursorItemIcon-170-sites`
row the dead g86d session left). **Code-only**: file reads/writes + git only —
no cargo/rustc/tests/graphify; the witness tests are WRITTEN NOT RUN. All C#
cites are `decompiled/Terraria/…` (1.4.5.8); Rust cites are `crates/…` on this
branch unless a co-lane tip is named. Scan date: 2026-11-21.

## 0. The g86d question — did the prior session land anything?

**NO.** `git log --all --grep=g86d` finds no commits; the branch tree carries
ZERO `cursor_item_icon`/`cursorItemIcon` hits in `crates/` (grep census). The
only cursorItemIcon mentions on this branch are the g86 altar census docs
(ANALYSIS :39562, PORT_MAP :760) recording the system as deferred. The g86d
session died before writing code. This lane takes the row.

## 1. The "~170 sites" — the exact count

The ledger's 170 is the **`cursorItemIconID` write count in Player.cs: exactly
169** (Select-String `cursorItemIconID\s*=[^=]`, case-sensitive). The full
system is larger — every `cursorItemIcon*` line across the decompilation:

| File | lines | writes | reads |
| --- | --- | --- | --- |
| `Player.cs` | 340 | 326 | 14 (5 are the field decls :1850-1858) |
| `Main.cs` | 34 | 22 | 12 |
| `Mount.cs` | 5 | 5 | 0 |
| `GameContent/Drawing/TileDrawing.cs` | 1 | 0 | 1 (:737 the placement-preview gate) |
| `GameContent/ObjectInteractions/PotionOfReturnGateInteractionChecker.cs` | 2 | 2 | 0 |
| `GameContent/UI/WiresUI.cs` | 2 | 0 | 2 (:130/:136 the radial gates) |
| `GameContent/UI/ResourceSets/CommonResourceBarMethods.cs` | 2 | 2 | 0 |
| **total** | **386** | **357** | **29** |

Player.cs writes by field: `Enabled` 132 (127 `= true`), `ID` **169**, `Text`
14, `Push` 3, `Reversed` 8. By function (write counts):

| Function | writes | C# span |
| --- | --- | --- |
| `TileInteractionsMouseOver` | 280 | :33587-34743 |
| `TileInteractionsCheckLongDistance` | 13 | :32044-32116 |
| `TileInteractionsMouseOver_Containers` | 10 | :34755-34813 |
| `ItemCheck` (head) | 9 | :42615-42650 |
| `TryInteractingWithMinecartTrackInNearbyArea` | 2 | :31996-32000 |
| `ItemCheck_OwnerOnlyCode` (makeNPC latch) | 1 | :43616-43619 |
| `ItemCheck_UseMiningTools` | 1 | :45620-45624 |
| `ItemCheck_UseBuckets` | 1 | :46324-46328 |
| `ItemCheck_UseWiringTools` | 1 | :46756-46760 |
| `PlaceThing_Tiles` / `_Walls` / `_Paintbrush` / `_PaintRoller` / `_PaintScraper_TryScraping` / `_PaintScraper_LongMoss` / `ShootFromCannon` | 1 each | :39725 / :39606 / :41919 / :41901 / :41877 / :41828 / :42040 |

## 2. Ported vs unclaimed — the branch census

`git grep cursor_item_icon` across every `task/W4-*` tip (local + remote):

* **`task/W4-g81-wire-draw-overlay`** (tip `ebb5308a0`, the wire-draw lane) —
  the ONLY code claim. `18e53dd0f` ports `DrawInterface_40_InteractItemIcon`'s
  **held-kind arm** (`ui/mod.rs` :1210-1244: the icon at
  `mouse+(10,10)+hitbox*cursorScale/2`, context-21 no-clamp, the cache-linger
  arm), `ItemIconCacheUpdate/Verification` (`app/mod.rs`, the 10-frame
  re-arm / non-zero-move decrement / >4px zero / kind-switch zero),
  `MouseOversClear`'s **enabled-only** per-update clear (`frame.rs` :447),
  and the wiring+mining tools' in-reach latch (`editing.rs` :2170-2192,
  `Player.cs:46752-46760`/`:45620-45624`). `851a73e11` folds the radial's
  stand-in onto the sim latch. **Its own comment declares the
  `cursorItemIconID != 0` override arm UNPORTED** ("the tile-interaction icon
  system — the wiring tools keep ID 0, so the held kind is the icon").
  g81 carries `cursor_item_icon_enabled: bool` + the cache triple ONLY — no
  ID/Text/Push/Reversed fields.
* **`task/W4-g29-tile-draw-pass-2` / `task/W4-g31-tile-draw-pass-4`** — read
  the enabled flag as the placement-preview gate
  (`TileDrawing.cs:737`, `tile_pass.rs::preview_gates`), not a write claim.
* **`task/W4-f22-ui-screens-4`** (`signs.rs`) + **`task/W4-g53-ui-achievement-screen`**
  (`ui/world_hover.rs`) — doc-only notes that the cursorItemIcon clears /
  layer-40 are unported on their branches.
* Everything else: ZERO hits. **The ID/Text/Push/Reversed system, the
  MouseOversClear ID/Text/Push clears, the ~150 tile-hover arms and the
  layer-40 override draw are UNCLAIMED.**

## 3. The system architecture (what a full port needs)

### 3.1 The state (Player.cs:1850-1858)

```csharp
public bool cursorItemIconEnabled;   // :1850
public bool cursorItemIconReversed;  // :1852
public int  cursorItemIconID;        // :1854
public int  cursorItemIconPush;      // :1856
public string cursorItemIconText = ""; // :1858
```

The port keeps `cursorItemIconEnabled` on App (g81's field). The other four
have NO port home — the seed's first contract item (§4.1).

### 3.2 The per-update clear — `Main.MouseOversClear` (:46503-46513)

Called from `DoUpdate` :17035, EVERY update before the player pass:
`enabled=false, ID=0, Text="", Push=0` (+ `signHover=-1, signBubble=false`).
`Reversed` is cleared ONCE per frame at `DoUpdate` :16842 (NOT in
MouseOversClear). g81 ports the enabled clear only; the ID/Text/Push clears
are the seed's. **Order matters**: the clear runs BEFORE the player tick
re-arms, so a stale icon never survives a frame.

### 3.3 The draw — `DrawInterface_40_InteractItemIcon` (:45037-45115)

Layer 40 (`:44909-44913`, InterfaceScaleType.UI, after layer 39 MouseOver),
plus a second call inside the options-window layer (:46169). The arm order:

1. `HoveringOverAnNPC || mouseInterface` → return (:45039).
2. `num = HeldItem.type`; `UsingBiomeTorches` swaps 8/966 through
   `BiomeTorchHoldStyle`/`BiomeCampfireHoldStyle` (:45045-45056 — the port's
   `player_lights::biome_torch_hold_style` exists, the campfire twin is
   `editing.rs`'s precedent).
3. **`cursorItemIconID != 0` → `num = ID`** (:45057-45060) — THE OVERRIDE,
   g81's declared gap.
4. `flag = enabled && num != 0 && MouseDisplayItem.IsAir` (:45061).
5. `ItemIconCacheVerification()`; `flag | (cacheTime > 0 && mouseItem.type < 1)`
   gate (:45062-45066). The `!flag` cache path swaps `_itemIconCacheSelectedItemID`
   with `cursorItemIconID` around the draw (:45067-45071/:45111-45114) — the
   linger draws the CACHED kind, the override draws the live ID.
6. Colors: held-slot `GetAlpha/GetColor(White)`; **ID != 0 forces
   currentColor=White, color=default** (:45072-45078). `ItemSlot.GetItemLight`
   (:45081). `Reversed` → FlipHorizontally (:45083-45086).
7. `num > 0`: offset `10 + Push`, `LoadItem(num)`, the icon at
   `mouse+(offset,offset) + GetDrawHitbox(num).Size()*cursorScale*0.5`
   through `ItemSlot.DrawItemIcon(context 21, sizeLimit 9999)` (:45089-45092).
   The `TryGetAmmo` sub-icon (:45093-45100, :45117-45144): paint
   1071/1543/1072/1544 → `FindPaintOrCoating` at offset (22,22)/(28,28)
   scale 0.8; the FlexibleTileWand `ShowsHoverAmmoIcon` arm at (28,28).
8. `Text != ""` → `MouseText(Text, 0, 0)` (:45102-45105).
9. `ID == 0 && held.color != default` → the painted-held-item overlay
   (:45106-45110).

### 3.4 The feed passes (the write sites)

* **`TileInteractionsMouseOver`** (:33587-34743, 280 writes) — the tile-hover
  pass, called from `TileInteractionsCheck` :32039 (the right-click seam)
  and gated `myX/myY == tileTargetX/Y` (:33589). ~100 tile-kind arms; the
  full arm list is §5. Most arms: `noThrow=2; enabled=true; ID=<decode>`.
  Special shapes: the container/dresser TEXT arms (21/467/88/441/468 →
  `MouseOver_Containers`/the dresser decode, `ID=-1` + Text, the
  `dresserTypeToIcon`/`GetChestIcon` collapse), the `Reversed` arms
  (209/15/497/49/510/511), the TileEntity reads (520 food platter / 395 item
  frame / 698 dead-cells jar / 471 weapons rack → `result.item.type`), the
  `HasItem` gate (440 golf cups), the `CanBeExtractinated` held-item gate
  (219/642), the sign arm (:34287-34307, the `signHover` clear), and the
  gamepad tail (:34738-34742, `UsingGamepad && Text==""` → clear — PARKED).
* **`TileInteractionsCheckLongDistance`** (:32044-32116, 13 writes) — the
  long-distance chest/dresser/sign hover (the f22 sign lane's neighbour).
* **`ItemCheck` head** (:42615-42650, 9 writes) — the bait/solution arms,
  **PORTED batch 2** (`cursor_item_icon.rs::item_check_head_icon`).
* **The tool/place latches** (1 write each): mining :45620-45624 + wiring
  :46756-46760 (**g81**), buckets :46324-46328, makeNPC :43616-43619,
  PlaceThing_Tiles :39725 / _Walls :39606 / _Paintbrush :41919 /
  _PaintRoller :41901 / _PaintScraper_TryScraping :41877 /
  _PaintScraper_LongMoss :41828, ShootFromCannon :42040. All
  `enabled=true` (+ `ItemIconCacheUpdate` on the four ItemCheck ones),
  gated on reach/tool-kind; the PlaceThing ones have NO cache update.
* **`TryInteractingWithMinecartTrackInNearbyArea`** (:31996-32000, 2 writes)
  — the unaimed-track 314 hover, `!enabled` gate, ID 2343.
* **Main.cs projectile interactions** (9 writes): `TryInteractingWithVoidLens`
  :37700-37701 (4131), `TryInteractingWithMoneyTrough` :37774-37778
  (3213/5098), `TryDespawningProjectile` :37867-37868 (the
  `DespawnItemIcon` set, ProjectileID.cs:185 — 90+ pairs), `TryPetting`
  :37957-37958 (the caller's icon: 5664/6149/5667/6161/5663/6148). All ride
  `DrawProjDirect` (:28073/:33203/:33700/:34264) — the DRAW pass, not the
  sim; `noThrow=4` (not 2).
* **`Mount.TryPettingMount`** (:6095-6108, 5 writes) — the mount-pet hover,
  types 62-65 → 5665/5666/6150/6151, `noThrow=4`.
* **`PotionOfReturnGateInteractionChecker.DoHoverEffect`** (:18-20, 2 writes)
  — 4870, `noThrow=2`; the smart-interact potion provider's hover half
  (the g86 provider chain ported the VOTE, not this hover effect).
* **The clears**: `DrawMouseOver` :42897 (dropped-item hover) / :42923
  (stealth-player hover), `HoverOverNPCs` :43089 (the life-text hover;
  :43024-43026 is the 685 town-pet WRITE, ID 327), `GUIHotbarDrawInner`
  :43619 (hotbar hover), `DrawInterface_33_MouseText` :45416,
  `CommonResourceBarMethods` :10/:22 (life/mana bar hover) — all
  `enabled=false` when a text hover wins.
* **The reads**: `WiresUI.cs:130` (`enabled && ID!=0 && ID!=3625` — the
  radial's icon gate, g81's stand-in), `TileDrawing.cs:737` (the
  placement-preview gate, g29/g31), `DrawInterface_InstrumentMouseText`
  :45442 (`ID>0` suppresses the instrument text).

## 4. The owner contract (the seed)

**Owner**: unclaimed — the next lane takes `cursorItemIcon-170-sites` whole.
**Co-lane**: `task/W4-g81-wire-draw-overlay` (tip `ebb5308a0`) owns the
enabled latch + cache + held-kind draw; the merge keeps g81's bodies and
ADDS the four fields + the override arm. Do NOT re-port g81's halves.

### 4.1 State (one commit) — **LANDED** (the g81 carry `2174f0fbb` + the phase-1 completion)

Add to App beside g81's `cursor_item_icon_enabled`:
`cursor_item_icon_id: i32` (0), `cursor_item_icon_text: String` (""),
`cursor_item_icon_push: i32` (0), `cursor_item_icon_reversed: bool` (false).
Extend the `frame.rs` :447 clear to the full MouseOversClear quad
(ID=0/Text=""/Push=0) + the :16842 Reversed clear at the frame head.
Witness: the clear quad + the Reversed-once-per-frame order.

**Landing notes**: the g81 halves were carried first (`2174f0fbb`, the
cherry-pick-no-commit of `18e53dd0f` — g81 is UNMERGED on origin/main, so
the carry is the seed contract's "keep g81's bodies" prerequisite; the
merge dedupes to one copy). The carry's `flow_app` full-App literal gained
the four g81 rows; phase 1 adds the four new fields' rows to BOTH full
literals (`App::new` + `flow_app` — g81's own branch never compiled its
tests, the carry fixes that). The quad clear rides `frame()`'s world
branch (g81's placement — DEVIATION: the C# `MouseOversClear` call
`:17035` precedes the `gameMenu` return `:17166`, so vanilla clears under
menus too; the port's world-gate is safe because no hover writer and no
layer-40 draw runs behind a menu). The Reversed clear rides `frame_head`
(the `DoUpdate` head, every frame including menus — `:16842` verbatim).
Witnesses: `frame.rs::w4_g86_the_mouse_overs_clear_quad_runs_on_the_world_update`
+ `w4_g86_reversed_clears_every_frame_the_quad_only_in_world`,
`app/mod.rs::w4_g86_icon_cache_tests` (4 — the g81 cache pair had none),
`editing.rs::w4_g81_the_tool_in_reach_arms_the_icon_latch_and_cache`
(the carry's latch had none). All WRITTEN NOT RUN, zero RNG.

### 4.2 The layer-40 override draw (one commit) — **LANDED** (phase-1 draw)

Extend g81's `ui/mod.rs` block with §3.3 steps 1-9: the ID override, the
White/default color force, the Push offset, the Reversed flip, the Text
MouseText, the painted-held overlay, the TryGetAmmo sub-icon (the
FindPaintOrCoating scan is `Player.cs:41927+`, the FlexibleTileWand arm
needs the wand system — document as deferred if unported). The
`HoveringOverAnNPC`/`mouseInterface` gate reads the port's mouse-NPC /
over-UI state. Witnesses: the override-vs-held-kind precedence, the push
offset, the reversed flip, the text arm.

**Landing notes**: `ui/mod.rs::interact_icon_kind` is the pure
kind-selection arm (`:45057-45063` + the `:45067-45071` linger swap folded
to a direct cached-kind read — the `:45111-45114` swap-back is net-zero)
+ the `num > 0` draw gate (`:45087`, the sign arm's -1 sentinel draws
nothing). The draw block adds the `10 + Push` offset (`:45089`), the
Reversed UV mirror (`:45083-45086`, the `[uv[2], uv[1], uv[0], uv[3]]`
player_layers precedent), the Text `draw_mouse_text` arm (`:45102-45105`)
and the head gate (`:45039`): `bubble_npc` (the `HoveringOverAnNPC`
stand-in) OR `minimap_hover` OR the closed-hotbar `hovered_slot`. The
block rides AFTER `cursor::draw` (the C# layer list `:44889-44913` draws
40 after 36/37/38/39 — the e19-draw_override compose-note's layer-40
position, the icon overlays whatever cursor is on screen). The
`MouseDisplayItem.IsAir` gate folds to `mouse_item.kind == 0 ||
count == 0` (`Item.cs:385`'s `type <= 0 || stack <= 0`; FakeCursorItem
parked); the linger's OWN gate (`:45063`) is `mouseItem.type < 1` — kind
only, so `interact_icon_kind` takes the two air flags separately (an
empty stack of a real kind passes IsAir but refuses the linger).
DEFERRED (documented at the block): the UsingBiomeTorches 8/966
swap (`:45045-45056`), TryGetAmmo (`:45093-45100`/`:45117-45144`), the
painted-held overlay (`:45106-45110` — no per-item color on the port's
ItemSlot), GetItemLight (`:45081`) + the held-slot GetAlpha/GetColor
(`:45072-45073` — plain white, the g81 stand-in). The open-panel slot
hover has no pre-draw resolution (the panel hit-tests inside
`inventory::draw`) — recorded deviation. The g81 carry's
`item_icon_cache_verification()` call site was INLINED over the disjoint
fields (`frame/draw.rs`): a whole-`self` method call cannot compile
inside the `let App {…} = self` destructure's live field borrows; the
method stays for the cache witnesses. Witnesses:
`w4_g86_the_layer40_id_override_beats_the_held_kind` (10 asserts incl the
linger-cached-kind, the two-air-gate split + the -1 refusal),
`w4_g86_the_reversed_flag_mirrors_the_icon_uv`,
`w4_g86_the_push_offsets_the_icon_base_10`. WRITTEN NOT RUN.

### 4.3 The tile-hover pass (the bulk, ≤10 arms/commit) — **STARTED** (B1 wired)

Port `TileInteractionsMouseOver` as a `cursor_item_icon.rs` scan driven from
the right-click seam (`editing.rs`'s `interact_tile` neighbour — the C#
`TileInteractionsCheck` :32031-32042 runs MouseOver THEN Use on the active
tile). The arm list is §5; each commit takes ≤10 arms in C# order, witnesses
per arm (the decode + the noThrow/enabled/ID write order). The container
TEXT arms need the chest-name read (`Main.chest[num].name`) + the
`Lang.chestType/chestType2/dresserType` tables + `WorldGen.GetChestIcon` /
`Chest.dresserTypeToIcon` — check `registry.rs`'s `tile_drop` family first
(the g34 break-drop lane ported several `GetItemDrop_*` decoders; reuse,
don't duplicate). The TileEntity arms need the TE item reads
(`tile_entities.rs`). The `IsWithinSnappngRangeToTile` gate (79/15/102/89/
487/497) is `Player.cs`'s snap-range helper — port it once.

**Wiring-shape note (B1 landed)**: the hover decode runs at
`update_editing`'s tile-hover head — AFTER the smart-cursor repoint +
`smart_interact_lookup` (the POST-repoint target IS `tileTargetX/Y`, the
`:33589` gate) and BEFORE the tool latches / the ItemCheck head. The C#
order is `LookForTileInteractions` (`Player.cs:27430`) BEFORE `ItemCheck`
(`:31827`) inside `Player.Update`; the port's fused `update_editing`
keeps that relative order (hover → tool latch → head icons). DEVIATION:
the port's seam sits behind `update_editing`'s map/minimap/HUD/reach
early returns, so the hover icon only arms while the cursor is in the
world reach — the C# `TileInteractionsMouseOver` runs inside
`LookForTileInteractions`' own Simple-reach gate (`:31962-31965`) too,
but the port's circular reach stand-in + the HUD-blocked returns are
narrower; the long-distance pass (§4.4) covers the out-of-reach hover
when it lands. B1's arms
60+79 (237/26) are WIRED there (`noThrow=2` + `enabled` BEFORE the ID);
the batch-2 head arms are WIRED after the tool latch with the
`!enabled` gate keeping the hover icon. Witnesses:
`editing.rs::w4_g86_the_altar_hover_wires_the_cursor_icon_id` +
`w4_g86_the_pole_head_icon_wires_after_the_hover_pass`. The remaining
~99 arms stay the ≤10/commit seed. The `tile_drop` family check: the
port's `registry::tile_drop` is the style-0 slice ONLY — the styled
`GetItemDrop_*` tables (Beds/Chests/Dressers/Doors/…) are NOT yet ported
anywhere (the kill cascade defers them via `ObjectDrop.item: None`), so
the styled arms port their tables fresh into `cursor_item_icon.rs`
(the g34 lane's future decoders can re-home them).

### 4.4 The remaining feeds (small commits)

The long-distance pass (§3.4), the minecart-track hover, the bucket/makeNPC/
PlaceThing/cannon latches (each 1-2 sites, the g81 latch shape), the
projectile-interaction icons (the DrawProjDirect seam — the port's
projectile draw pass), the mount-pet hover, the potion-return hover effect
(the g86 provider's `potion_of_return` flag is the gate), the text-hover
clears (the port's tooltip/resources passes). The gamepad tail
(:34738-34742) + every `GamepadDisableCursorItemIcon` gate PARK (false).

### 4.5 Verification

`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
green; the layer-40 screenshot A/B (the icon appears on hover, the override
beats the held kind); anchors.txt rows per landed half. RNG: ZERO draws in
the whole system (every arm is integer/string maths) — the seed adds no
stream.

## 5. The TileInteractionsMouseOver arm list (the exact site-list)

C# order, `Player.cs:33587-34743`. `ID` = the cursorItemIconID write;
`dec` = the decode helper. All arms set `noThrow=2` + `enabled=true`
unless noted. **B1 = ported batch 1** (`altar_hover_icon`).

| # | kind(s) | lines | ID / decode |
| --- | --- | --- | --- |
| 1 | 79 bed | :33593-33610 | bottom-side: `GetItemDrop_Beds(frameY/36)`; else snap-96 gate → 5013 |
| 2 | 597 pylon | :33611-33658 | `GetPylonItemTypeFromTileStyle` then the style 1-10 overrides 4875/4916-4921/4951/5652/5653 |
| 3 | 621 | :33659-33664 | 3750 |
| 4 | 464 | :33665-33670 | 3814 |
| 5 | 699 | :33671-33676 | 5482 |
| 6 | 33 candles | :33677-33682 | `GetItemDrop_Candles(frameY/22)` |
| 7 | 21 chest | :33683-33686 | → Containers |
| 8 | 467 chest2 | :33687-33690 | → Containers |
| 9 | 441 | :33691-33711 | `GetChestIcon(frameX/36, false)` (the 2x2 origin decode) |
| 10 | 468 | :33712-33732 | `GetChestIcon(..., true)` (chestIcon2) |
| 11 | 88 dresser | :33733-33771 | TEXT: `dresserType[frameX/54]` / chest name; collapse → `dresserTypeToIcon`; ID=-1 sentinel |
| 12 | 10/11 doors | :33772-33801 | `GetDoorItem(style)`; style 11 → 1141 (TempleKey); ID≠0 gate |
| 13 | 104 clocks | :33802-33807 | `GetItemDrop_Clocks(frameX/36)` |
| 14 | 356 | :33808-33813 | 3064 |
| 15 | 663 | :33814-33819 | 5381 |
| 16 | 377 | :33820-33825 | 3198 |
| 17 | 209 | :33826-33858 | frameX<72→928 / <144→1337 / <216→3369 / <360→3664; Reversed = (frameX/18 mod 4)<2 |
| 18 | 216 | :33859-33871 | 970 + frameY/40 |
| 19 | 387/386 | :33872-33880 | 3239 (the GetTopLeftAndStyles call is dead — the ID is constant) |
| 20 | 389/388 | :33881-33886 | 3240 |
| 21 | 335 | :33887-33892 | 2700 |
| 22 | 15 chair | :33893-33902 | snap-40 gate; `GetItemDrop_Chair(frameY/40)`; Reversed if frameX/18<1 |
| 23 | 102 | :33903-33908 | snap-40 gate; 355 |
| 24 | 89 bench | :33909-33914 | snap-40 gate; `GetItemDrop_Benches(frameX/54)` |
| 25 | 487 picnic | :33915-33920 | snap-40 gate; `GetItemDrop_PicnicTables(frameX/72)` |
| 26 | 497 toilet | :33921-33930 | snap-40 gate; `GetItemDrop_Toilet(frameY/40)`; Reversed if frameX/18<1 |
| 27 | 410 | :33931-33936 | 3536 + min(frameX/36, 3) |
| 28 | 480 | :33937-33942 | 4054 |
| 29 | 509 | :33943-33948 | 4318 |
| 30 | 657 | :33949-33954 | 5345 |
| 31 | 658 | :33955-33960 | 5347 |
| 32 | 720 | :33961-33966 | 5598 |
| 33 | 721 | :33967-33972 | 5599 |
| 34 | 725 | :33973-33978 | 5655 |
| 35 | 733 | :33979-33984 | 5113 |
| 36 | 463 | :33985-33990 | 3813 |
| 37 | 491 | :33991-33996 | 4076 |
| 38 | 494 | :33997-34002 | 4089 |
| 39 | 411 | :34003-34008 | frameX<36 gate; 3545 |
| 40 | 338 | :34009-34014 | 2738 |
| 41 | 455 | :34015-34020 | 3747 |
| 42 | 219/642 | :34021-34027 | `CanBeExtractinated[held.type]` gate; extractinateHover=true; ID=held.type |
| 43 | 212 | :34028-34033 | 949 |
| 44 | 314 track | :34034-34039 | gravDir==1 gate; 2343 |
| 45 | 215 campfire | :34040-34082 | frameX/54 switch: 0→966, 14→5299, 15→5357, 8-13→4689+n-8, 7→3724, 6→3723, 5→3050, default 3046+n-1 |
| 46 | 4 torch | :34083-34145 | frameY/22 switch: 0→8, 8→523, 9→974, 10→1245, 11→1333, 12→2274, 13→3004, 14→3045, 15→3114, 16-21→4383-4388, 22→5293, 23→5353, default 426+n |
| 47 | 13 bottles | :34146-34151 | `GetItemDrop_Bottles(frameX/18)` |
| 48 | 29 | :34152-34157 | 87 |
| 49 | 97 | :34158-34163 | 346 |
| 50 | 510 | :34164-34181 | 4319; Reversed decode (:34175-34179) |
| 51 | 511 | :34182-34199 | 4320; Reversed decode (:34193-34197) |
| 52 | 49 | :34200-34205 | 148 |
| 53 | 372 | :34206-34211 | 3117 |
| 54 | 646 | :34212-34217 | 5322 |
| 55 | 174 | :34218-34223 | 713 |
| 56 | 50 | :34224-34232 | 165 (the :34229 conditional) |
| 57 | 139 music boxes | :34233-34243 | `GetItemDrop_MusicBoxes(num17)` |
| 58 | 207 fountains | :34244-34286 | frameX/18 ÷2 loop → 909/910/940-945/4922/4417 (10 styles) |
| 59 | tileSign | :34287-34307 | the sign hover: signBubble/signX/signY/signHover; the in-box clear enabled=false/ID=-1 |
| 60 | **237 Lihzahrd altar** | :34309-34314 | **1293 — B1** |
| 61 | 466 | :34315-34320 | (the :34318 arm) |
| 62 | 125 | :34321-34326 | (the :34324 arm) |
| 63 | 354 | :34327-34332 | (the :34330 arm) |
| 64 | 287 | :34333-34338 | (the :34336 arm) |
| 65 | 132 | :34339-34344 | (the :34342 arm) |
| 66 | 136 | :34345-34350 | (the :34348 arm) |
| 67 | 144 | :34351-34364 | (the :34354 arm) |
| 68 | 440 golf cups | :34365-34400 | frameX/54 → 1526/1524/1525/1523/1522/1527/3643; `frameY/54==1 \|\| HasItem(id)` gate |
| 69 | 470 | :34401-34415 | frameX%136/34: <2→498, <4→1989 |
| 70 | 475 | :34416-34421 | 3977 |
| 71 | 520 food platter | :34422-34430 | TE read → `result.item.type` |
| 72 | 395 item frame | :34431-34457 | the 2x2 origin decode + TE read → item.type |
| 73 | 698 dead-cells jar | :34458-34474 | the column origin decode + TE read → item.type |
| 74 | 471 weapons rack | :34475-34501 | the 3x3 origin decode + TE read → item.type |
| 75 | 334 | :34502-34505 | (the :34504 arm) |
| 76 | 16 anvil | :34506-34511 | (the :34509 arm) |
| 77 | 17 furnace | :34512-34517 | (the :34515 arm) |
| 78 | 18 workbench | :34518-34523 | `GetItemDrop_Workbenches(frameX/18>>1)` |
| 79 | **26 demon/crimson altar** | :34524-34536 | **6135/6136 — B1** |
| 80 | 77 forge | :34537-34542 | 221 |
| 81 | 86 | :34543-34548 | (the :34546 arm) |
| 82 | 94 | :34549-34554 | (the :34552 arm) |
| 83 | 96 | :34555-34567 | (the :34558 arm) |
| 84 | 101 | :34568-34580 | (the :34571 arm) |
| 85 | 106 | :34581-34586 | (the :34584 arm) |
| 86 | 114 | :34587-34592 | (the :34590 arm) |
| 87 | 133 | :34593-34605 | (the :34596 arm) |
| 88 | 134 | :34606-34611 | (the :34609 arm) |
| 89 | 217 | :34612-34617 | (the :34615 arm) |
| 90 | 218 | :34618-34623 | (the :34621 arm) |
| 91 | 220 | :34624-34629 | (the :34627 arm) |
| 92 | 228 | :34630-34635 | (the :34633 arm) |
| 93 | 243 | :34636-34641 | (the :34639 arm) |
| 94 | 247 | :34642-34647 | (the :34645 arm) |
| 95 | 283 | :34648-34653 | (the :34651 arm) |
| 96 | 300-308 | :34654-34707 | the 9 statue arms (one per kind) |
| 97 | 412 | :34708-34713 | (the :34711 arm) |
| 98 | 499 | :34714-34719 | (the :34717 arm) |
| 99 | 622 | :34720-34725 | (the :34723 arm) |
| 100 | 172 sinks | :34726-34731 | `GetItemDrop_Sinks(frameY/38)` |
| 101 | 355 | :34732-34737 | 3000 |
| — | gamepad tail | :34738-34742 | PARKED (UsingGamepad && Text=="" → clear) |

(The "(the :NNNN arm)" rows are the constant-ID arms whose exact item id is
at the cited line — the port reads them verbatim; the list pins the SITE,
the decode is a one-line read. The §5 table is the claim ledger: an owner
marks a row done per commit.)

## 6. This lane's landings (the self-contained batches)

* **Batch 1** (`835e89a65`): `cursor_item_icon.rs::altar_hover_icon` — arms
  60+79 (4 write sites), 3 witnesses. **WIRED** in the phase-1 draw commit
  (the `update_editing` tile-hover head).
* **Batch 2** (`90efad638`): `fishing_get_bait` + `item_check_head_icon` —
  the ItemCheck head (9 write sites), 3 witnesses. **WIRED** in the
  phase-1 draw commit (the ItemCheck-head seam after the tool latch).
* **The g81 carry** (`2174f0fbb`): the enabled latch + the 10-frame cache +
  the layer-40 held-kind draw + the wiring/mining latches, cherry-picked
  no-commit from `18e53dd0f` (g81 unmerged — the merge keeps one copy).
* **Phase 1 (§4.1 + §4.2 + the B1/B2 wiring)**: the four
  ID/Text/Push/Reversed fields + the full MouseOversClear quad + the
  :16842 Reversed frame-head clear + the layer-40 ID-override draw
  (`interact_icon_kind` + Push/Reversed/Text/head-gate) + both batches
  wired at their C# call positions; 14 new witnesses (4 clear/defaults +
  4 cache + 1 latch + 3 draw + 2 wiring).
* The rest of the system is the §4 seed — the ~99 remaining hover arms
  (§4.3, ≤10/commit) + the remaining feeds (§4.4).

### 6.1 The g53c cross-note (the flagging lane)

`task/W4-g53-ui-achievement-screen` (`ui/world_hover.rs:22`,
`docs/research/81-doc-sweep-ui.md` §2's GAP row, ANALYSIS :36164-36169 on
its branch) flagged the `cursorItemIcon`/`TileInteractionsMouseOver`
interact-icon subsystem as "a whole subsystem, too large for this lane"
and kept the `no_throw` field's check shape. **This lane (W4-g86
cursor-item-icon) now owns the row**: the census + seed is research/244,
the decode batches + the g81 carry + phase 1 are landed above. g53's
`world_hover.rs` clears (`DrawMouseOver` :42897/:42923, `HoverOverNPCs`
:43089 — the text-hover-wins `enabled=false` arms, §3.4's clears list)
stay g53's own claim; when its branch merges, its draw-pass clear sites
read the now-live `cursor_item_icon_enabled` field (the carry supplies
it). The elder-slime 685 icon-327 arm (Main.cs:43024-43026) stays PARKED
on both lanes — no `NpcKind` 685 row in the port's roster.

## 7. The altar-draw keep-one (the co-lane question)

**g86 altar-draw vs g34-cage vs g43-temple — DISJOINT, no keep-one needed:**

* **g86** (this branch, `origin/main..HEAD`): the smart-cursor OUTLINE
  system (`render/tile_outlines.rs`, `world_sim/smart_interact.rs`, the
  HighlightMask atlas pack, the `frame/draw.rs` fx-range hook at :1210, the
  `selection_mode` pylon glow) + the altar-26/237 summon-use CENSUS (docs).
  The altar's FRAMES (`drawdata.rs:327-347` height-18) + GLOW
  (`scanner.rs:1051-1061`) were already on main — g86 verified, not re-ported.
* **g34** (`task/W4-g34-drawdata-wire-2`, tip `cf85a1cf9`): the critter-CAGE
  animator + the break-drop cascade (`edit/kill.rs` Check6x3/3x2/2x2/1x2).
  Its altar-26 touch is the Check3x2 furniture arm: **altar 26 sweeps but
  records NO drop** (the SmashAltar ore-seeding is app-wired
  `editing.rs:1303`) + the boulder-refusal support judge. Its
  `frame/draw.rs` hunk is the cage-key mix at :1183-1220 (the
  `any_cage` WorldFrameKey contribution) — g86's outline hook is at :1210
  in the SAME region: **a merge CONFLICT ZONE, not a semantic overlap**
  (the cage key feeds the cached tile pass, the outline is the post-pass
  overlay; both survive, the merger interleaves). g34's `drawdata.rs`
  changes are the cage rows (726 etc.), NOT the altar height-18 group.
* **g43** (`task/W4-g43-biome-jungle-temple`, tip `9f53f18a0`): the temple
  USE-side (`doors.rs::use_golem_altar`/`spawn_golem_on_altar` fidelity
  fixes — the 58-slot scan, the exclusive scan bounds, the bottom-centre
  spawn, the target latch, the HasAwoken announcement; the Solar Tablet
  2767 + Temple Key 1141 + Abeemination jungle gate). g43's `doors.rs`
  blob (`377e0604`) DIFFERS from main/g86's (`610ea0bf`) — g86 did NOT
  touch doors.rs (the summon-use was already landed by the wave-3 lane;
  g86 only censused it). **g43's doors.rs is the keep-one survivor for the
  237 summon** (its fidelity fixes supersede the base body g86 cited).
  g43 has NO outline/smart-interact files — zero overlap with g86's system.
  g43's `editing.rs` hunks (:2013/:2062 the tablet dispatch, :15949+ the
  summon gates) are disjoint from g86's :2160 smart-interact seam.

**Verdict**: the three lanes touch the altar at three DIFFERENT layers —
g86 the selection outline (draw), g34 the break-drop (sim), g43 the
summon-use (sim). No duplicated body; the only merge work is the
`frame/draw.rs` :1183-1220 interleave (g34 cage key + g86 outline hook) and
taking g43's doors.rs wholesale. The cursorItemIcon hover icons (this
seed) are a FOURTH layer, unclaimed by all three.
