# 143 — W4-d120 town chatter RETIRE: the f22c dynamic-gap patch, the d43 slime activation note, the 579 spawn contract, GetChat-40 COMPLETE

Status: RETIRE (research FINAL; code-only mode, no cargo/tests/rustfmt invoked).
Branch: `task/W4-d120-town-chatter` (DONE c5e68a80, retire phase continues the SAME branch).
Lane tips re-read 2026-11-21 from `origin`: f22 `be73b87b5` (sign hover/break round — the
sign-aware panel landed), d43 `a725acb04` (**0 commits ahead of main — claim-only, no
SlimeVariant kinds**), d42 `9b22c091a` (13 ahead — the 578-630 roster join COMPLETE,
batch 1 `3836a2d1a` carries the 579 row), d41 `4ed22f0b3` (claim-only, band 491-560 —
579 OUTSIDE it), f43 `79a037f5e` (town-spawn census, the license-chain tail arms UNWIRED).

## 1. INTERACTION_LINES dynamic gap — the exact 2-hunk fix for f22c (panel-merge lane)

The f22b-verify (ANALYSIS "W4-d120 final census") found `INTERACTION_LINES = 1` frozen
while the C# grows it: `_neededInteractionLines = (int)Math.Ceiling(count / 4f)`
(`NPCChatPanel.cs:174`, `AllowedInteractionsPerLine = 4` `:30`), the height adds
`30·lines + max(0, 2·(lines−1))` (`:68`) and the row reset steps y by `num / 4 * 22`
(`:106`). The panel file is f22's (its tip reworked `close_button_rect` into the
`interaction_rects` walk + the sign row), so the fix is PUBLISHED here for f22c —
no d120 edit (d120's own `ui/npc_chat.rs` is the pre-sign text-only version, unchanged).

Both hunks apply against `origin/task/W4-f22-ui-screens-4` tip `be73b87b5`,
`crates/tr-app/src/ui/npc_chat.rs` (the const at `:74`, `panel_rect` `:92-97`,
`interaction_rects` `:113-137`; `chat_box_height` already carries the
`30·il + 2·(il−1).max(0)` growth — `npc_chat.rs:874` — so NO selection-half change).

### Hunk 1 — `INTERACTION_LINES` const → dynamic fn + `panel_rect` takes the live count (lines 67-97)

```diff
-/// The interaction row count the port renders: `CloseChat`
-/// (`NPCInteractions.cs:271-287`, `Condition() => true`) plus the sign
-/// context's `OpenSign` (`:15-42`, `Condition() => LocalPlayer.sign > -1`,
-/// registered AFTER CloseChat at `:515-516`) — the shop / happiness /
-/// housing / … buttons are deferred (module header), so
-/// `_neededInteractionLines = ceil(count / 4) = 1` (`NPCChatPanel.cs:174`,
-/// `AllowedInteractionsPerLine = 4` `:30`).
-const INTERACTION_LINES: usize = 1;
+/// `_neededInteractionLines` (`PrepareInteractions`, `NPCChatPanel.cs:174`):
+/// `(int)Math.Ceiling(count / 4f)` (`AllowedInteractionsPerLine = 4`, `:30`)
+/// over the LIVE interaction count — Close + Sign stay 1 line today; the
+/// shop / happiness / housing rows (g58/f38's buttons) push the count past
+/// 4 (W4-d120 retire, research/143 §1). Zero interactions → 0 (`Ceiling(0f)`).
+pub(crate) fn interaction_lines(count: usize) -> usize {
+    count.div_ceil(4) // `:174`
+}
```

and `panel_rect` grows one parameter (the live `PrepareInteractions` list length):

```diff
-pub(crate) fn panel_rect(lookups: &Lookups, screen: Vec2, chat_text: &str) -> [f32; 4] {
+pub(crate) fn panel_rect(
+    lookups: &Lookups,
+    screen: Vec2,
+    chat_text: &str,
+    interaction_count: usize,
+) -> [f32; 4] {
     let base = chat_box_rect(screen.x as i32, screen.y as i32);
     let lines = panel_lines(lookups, chat_text).len();
-    let h = chat_box_height(lines, INTERACTION_LINES);
+    let h = chat_box_height(lines, interaction_lines(interaction_count));
     [base.x as f32, base.y as f32, base.w as f32, h as f32]
 }
```

Call-site updates (all on the f22 tip): `close_button_rect` `:105` passes the labels
through unchanged (it already forwards to `interaction_rects`); `draw_panel` `:184`
becomes `panel_rect(lookups, screen, chat_text, interactions(sign_open, edit_sign).len())`;
`ui/mod.rs:1275`'s `over_dialog` becomes
`npc_chat::panel_rect(lookups, screen, npc_chat_text, npc_chat::interactions(sign_open, false).len())`
(the gate only needs the count — Edit/Save both render 1 row).

### Hunk 2 — `interaction_rects` derives the line count + the y-step (lines 119-133)

```diff
-    let panel = panel_rect(lookups, screen, chat_text);
-    let il = INTERACTION_LINES as f32;
+    let panel = panel_rect(lookups, screen, chat_text, labels.len());
+    let il = interaction_lines(labels.len()) as f32;
     let base_x = panel[0] + 30.0;
     let y = panel[1] + panel[3] - 22.0 * il + (2.0 * (il - 1.0)).max(0.0) - 4.0;
     let mut x = base_x;
     let mut out = Vec::with_capacity(labels.len());
     for (i, label) in labels.iter().enumerate() {
         if i % 4 == 0 {
             x = base_x; // `:103-107`
+            // `:106` — `rectangle.Y = num / 4 * 22 + (int)vector.Y`.
+            out_y = y + (i / 4) as f32 * 22.0;
         }
```

with `let mut out_y = y;` before the loop and `out.push([x, out_y, w, 22.0]);`
replacing `out.push([x, y, w, 22.0]);` (the row-2 origin steps +22 per
`num / 4`; the 2-px inter-row GAP is already inside the panel height's
`2·(lines−1)` term `:68`, the rects themselves stay 22 tall `:97`).

The maths check (one text line): count 5 → il 2 → h = 30+30+(60+2) = 122,
row-1 y = panel.bottom − 44 + 2 − 4, row-2 y = row-1 + 22 — the C#'s
`vector.Y + num/4*22` with `vector = BottomLeft + (30, −22·2 + 2 − 4)` (`:93`).
Count ≤ 4 keeps today's pixels EXACTLY (il 1, y-step 0) — the existing
`the_close_row_rides_the_panel_bottom` / `the_sign_row_toggles_edit_and_save`
witnesses stay green; a new witness should pin count 5 → h 122 + row-2 y +22
(WRITTEN NOT RUN in this lane — the patch rides f22c).

## 2. d43 town-slime colour words — ALREADY type-keyed; the activation note

The d43 lane tip (`a725acb04`) is **0 commits ahead of main**: no `SlimeVariant`
town kinds landed, no enum rows, no claim doc. The Blue default is therefore still
the live behaviour — and the d120 arm is ALREADY wired for every colour word:

- `npc_chat.rs`'s town-slime arm keys on the VANILLA type
  (`ty if crate::npc_ai_town::is_town_slime(ty as i32)`, `:272` — the
  `NPCID.Sets.IsTownSlime` set 670/678-684, `NPCID.cs:10119`), NOT on a kind.
- `slime_chat_colour(ty)` (`npc_chat.rs:315-326`) ports the WHOLE
  `Lang.GetSlimeType` switch (`Lang.cs:375-406`, re-read): 670/default Blue,
  678 Green, 679 Old, 680 Purple, 681 Rainbow, 682 Red, 683 Yellow, 684 Copper —
  the witness `the_town_slime_colour_words_match_the_switch` pins all eight.
- The port's `SlimeVariant` enum (`npc/data.rs:14-24`) is the WILD slime's
  negative-net-id colour set (Blue/Green/Pinky/Baby/Black/Purple/Red/Yellow) —
  it has NO Old/Rainbow/Copper and is NOT the town-slime vehicle. d43's kinds
  must be separate `NpcKind` rows with `cs_id` 670/678-684.

**Activation note for d43/f43 (zero d120 edits needed):** the moment a kind with
`cs_id() == 670|678..684` spawns (f43's `unlockedSlime*Spawn` license arms,
`Main.cs:66042-66113` census), `get_chat`'s fallback arm answers the correct
`Slime<Colour>Chatter.Chatter` category automatically — `let ty = npc.kind.cs_id()`
(`npc_chat.rs:201`) feeds the guard and the colour switch. NO Blue-default
hardcode exists on the arm path; the "recorded Blue default" in the arms-D
notes describes today's unreachable-arm state (no port kind reaches it), not a
wiring gap. The event-text half rides `special_event_text(ctx, "Slime", ty)`
(`:273`) — the same `HasSpecialEventText` census as the pets.

## 3. The BartenderUnconscious 579 spawn contract (for d42b / the spawn lane)

WHO owns it: d41's tip is a claim-only 491-560 band (579 outside). **d42's
578-630 band INCLUDES 579 and its roster join is COMPLETE** (batch 1
`3836a2d1a` 06:46 — AFTER d120's claim `dad8db3d9` 05:48 + kinds push
`e4e400629` 06:28; the 579 row is now a DUPLICATE-kind collision, see §3.4).
The SPAWN arm itself is unclaimed on every pushed tip (grep `savedBartender`
hits only the `.wld` flag IO). d120 publishes the exact contract; the spawn
lane (d42b or c41/f43's town-spawn surgery) ports it. NO spawn code in d120.

### 3.1 The spawn arm — `NPC.SpawnAnNPC`'s top-level else-if (`NPC.cs:1656-1659`)

```csharp
else if (!NPC.savedBartender && DD2Event.ReadyToFindBartender && !NPC.AnyNPCs(579)
    && Main.rand.Next(80) == 0 && !this.waterTile)
{
    this.SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, 579);
}
```

- **Tab-depth pin**: a TOP-LEVEL else-if of the big ladder (3 tabs, `:1656`),
  the sibling AFTER the wall-244 marble-critter block (`:1584`, whose inner
  `spawnTileY > worldSurface` gate `:1597` holds the surface chain) and BEFORE
  the spider-wall arm (`:1660`). It fires on ANY spawn tile the earlier arms
  declined — no biome gate of its own beyond `!waterTile`.
- **Gate census**: `ReadyToFindBartender => NPC.downedBoss2` (`DD2Event.cs:87`
  — EoW/BoC down, NOT a DD2-invasion flag; the port's `world.downed_eow`
  already covers the shared flag, `world_sim/npcs.rs:5425-5427` BoC death).
  `savedBartender` is the world flag (the `.wld` flags block reads/writes it,
  `wld/sections/flags.rs:174-179` — currently hardcoded false/skip; the spawn
  lane owns the live field, the `saved_stylist`/`saved_golfer` precedent).
- **The draw is the BARE `Main.rand.Next(80)`** — no RollLuck, no rate stage.
  Short-circuit order matters: saved → boss2 → AnyNPCs(579) → Next(80) →
  waterTile (the roll is NOT paid when an earlier term fails).
- **Placement**: `SpawnNPC(x*16+8, y*16, 579)` (`:5238-5267` — the slime-1
  rerolls `:5240-5250` never fire for 579) → `NewNPC` natural-spawn source
  (`:80750`), `nPC.Bottom = (X, Y)` (`:80770`) — the 34×8 body's bottom-centre
  at the tile point: position = `(x*16+8 − 17, y*16 − 8)`. The port's
  `try_spawn_webbed_stylist` stand-in (`world_sim/npcs.rs:898-925`) is the
  shape precedent (the `town_look_timer == 0` cadence, the `UpdateTime`
  stream, `Next(8)` → here `Next(80)`); the faithful port keeps the arm's
  ladder position (any declined tile), the cadence stand-in is the recorded
  deviation if reused.
- **SetDefaults 579** (`NPC.cs:16554-16568`): friendly NOT townNPC, 34×8,
  aiStyle 0, damage 10, defense 15, lifeMax 250, NPCHit1/NPCDeath6,
  knockBackResist 0.5, scale 1, rarity 1. Both lanes' rows agree (d120
  `e4e400629`, d42 batch 1).

### 3.2 The talk transform — 579 → 550 raises savedBartender

- Trigger: aiStyle-0's `talkNPC == whoAmI` scan (`NPC.cs:19829-19831`), the
  `type == 579` arm (`:19857-19860`) runs `AI_000_TransformBoundNPC(i, 550)`
  (`:44884-44894`): `Transform(550)` THEN `AI_007_TownEntities_UpdateSavedStates`
  — whose `case 550` sets `NPC.savedBartender = true` (`:52959-52961`). NOT on
  the 579 spawn. The same update runs per-tick from the town AI (`:53185`), so
  a housed Tavernkeep keeps it raised.
- `Transform(550)` (`:81084-81143`): bottom-anchored reposition
  (`position.Y += height; SetDefaults; position.Y -= height` — the 34×8 → 18×40
  grows UP from the feet), hp carries the ratio `life·newMax/oldMax` min 1
  (`:81106-81110`), ai zeroed, `case 550` homes him homeless at his feet
  (`:81122-81128` `UpdateHomeTileState(homeless: true, centre-x/16, bottom/16)`),
  `townNPC → homeless = true` + `CheckAchievement_RealEstateAndTownSlimes`
  (`:81138-81141`).
- The port precedent is the rescue-fn family (`world_sim/npcs.rs:651-718` —
  `rescue_bound_goblin`/`rescue_bound_angler`/…): the spawn lane adds
  `rescue_bound_bartender(n, world)` (kind → `DD2Bartender`, `max_hp` restore
  250 — `data.rs:3563`, hp full per the family's stand-in vs the C# ratio,
  `home = None`, `world.saved_bartender = true`) + the
  `try_open_shop_at_cursor` hit arm between the Angler rescue (`:452-462`)
  and the quest turn-in. The 579 chat line (`BartenderSpecialText.FirstMeeting`,
  zero draws) is ALREADY landed (`npc_chat.rs:251-254`). ORDER pin: the C#
  answers the 579 FirstMeeting line on the OPENING talk (`GetChat` reads
  type 579 at `Main.cs:43081` — the AI transform runs the NEXT tick once
  `talkNPC == whoAmI`, `:19831`), and every later chat reads the 550
  bartender ladder. A wiring lane that rescues BEFORE the first `get_chat`
  (the bound-goblin shop-collapse precedent) would skip the FirstMeeting
  line — keep the 579 line on the opening click and transform after it (or
  record the skip as a deviation).

### 3.3 The bound-579 quirks (both already censused, keep them)

- Skips `TargetClosest` (`:19864-19868` — the `type != 376 && type != 579`
  gate) and floats in liquid (`:19869-19879` — `velocity.Y = -0.4`, the
  Sleeping Angler twin).
- `Sets.BoundTownNPCs` → `homeless = true` per AI tick (`:19823-19826`).
- `NPCAddHeight` 579 `num = -2f` (`Main.cs:22152`) — landed in `e4e400629`.

### 3.4 The kind collision (d42 batch 1 vs d120 kinds B) — for the d37c matrix

d42's batch 1 (`3836a2d1a` 06:46) re-lands the 579 `NpcKind` row AFTER d120's
claim-first push (`dad8db3d9` 05:48, kinds `e4e400629` 06:28). Same
claim-priority rule as the d36 collision (ANALYSIS "W4-d120 final census"):
**d120's row is the landed truth** (claimed + pushed first; protocol 245 vs
d42's port-local 355). The matrix keeps ONE `BartenderUnconscious` variant —
d120's (protocol 245, the Hide=true bestiary fold, the {31} immunities, the
still-stand + NPCAddHeight rows) — and d42's batch-1 579 hunks drop at the
merge (its OTHER 578/580-582 kinds are untouched). d42's own roster doc
(`9b22c091a`) predates this note; the cross-note is published here.

## 4. Research FINAL — the GetChat-40-complete declaration

**`NPC.GetChat` (`NPC.cs:93766-94821`) is COMPLETE: all 40 arms accounted for.**
The full census (ANALYSIS "W4-d120 final census" + the re-pin/kind-arms notes):

- **36 type-case arms** ported kind-keyed or type-keyed:
  17 Merchant / 18 Nurse / 19 ArmsDealer / 20 Dryad / 37 OldMan / 38 Demolitionist /
  54 Clothier / 105 BoundGoblin / 107 GoblinTinkerer / 106 BoundWizard /
  108 Wizard / 123 BoundMechanic / 124 Mechanic / 22 Guide / 142 SantaClaus /
  160 Truffle / 178 Steampunker / 207 DyeTrader / 208 PartyGirl / 209 Cyborg /
  227 Painter / 228 WitchDoctor / 229 Pirate / 354 WebbedStylist / 353 Stylist /
  368 TravellingMerchant / 376 SleepingAngler / 369 Angler / 453 SkeletonMerchant /
  441 TaxCollector / 579 BartenderUnconscious / 550 DD2Bartender /
  589 SleepingGolfer / 588 Golfer / 633 Zoologist / 663 Princess.
- **Arms D** (the LAST four): the town pets 637/638/656 (`:94804-94815` — event
  text then ONE reverse pick over the DOT-LESS `<Pet>Chatter.Chatter` filter,
  `Lang.cs:360-373`) + the `IsTownSlime` set arm (`:94816-94819`) with the full
  `GetSlimeType` colour switch — type-keyed, activation-automatic (§2).
- **NOT ladder arms** (the naming census): the happiness mood report rides the
  `ReportHappiness` BUTTON (`NPCInteractions.cs:289-312`, g58) and the homeless
  text `RequestHome` (`:314-368`) — buttons, not GetChat cases; the shop is the
  `OpenShop` button (`:44-77`, the 25-row registry census, d120's
  `shop_index_for_kind` rows landed). The Dryad cola override
  (`:94012-94015`) is the only in-ladder mood-adjacent line — ported.
- **Merge-lane notes standing**: the f38/f41 cake-block union (keep ONE copy,
  d120's gift-before-text talk seam pins the order), g58's zoologist shared-fn
  (keep g58's `should_bestiary_girl_be_lycantrope` call — same predicate), the
  f22c panel dynamic-gap patch (§1), the d43/f43 slime activation (§2), the
  579 spawn contract (§3).

Remaining GetChat-adjacent scope is OTHER lanes' (not gaps in the ladder):
`ConditionalDialogue`'s non-cake half (research/29 R-29-6 tail), the panel
buttons (f22c/g58/f38), the 579 spawn arm (§3), the d43/f43 kinds. The
selection ladder itself needs NO further ports.

Verification: code-only mode — every citation re-read against the decompiled
C# at the direct file paths (NPC.cs/Lang.cs/DD2Event.cs/NPCChatPanel.cs) and
the pushed lane tips (`git show origin/…`); no cargo/tests/rustfmt invoked;
witnesses WRITTEN NOT RUN where noted.