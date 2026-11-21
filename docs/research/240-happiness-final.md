# 240 — The happiness final round (W4-g58): the buy/sell latch repoint, the report-UI census, the savings 3-way call-site census, the portrait-hop census

**SEALED** (W4-g58 retire, 2026-11-21): the happiness family is closed for
porting — the one-paragraph family bible (g58-latch canonical / f37-price-
math / d122-shops) lives in research/242 §3; the f37 residual-reader hunks
in research/241; the portrait-system remainder seed in research/242 §2.

Date: 2026-11-21. Code-only — no cargo/tests run; every claim re-read from
the C# (`decompiled/`) and the branch tips named below. This is g58's FINAL
round: the four remaining items of the "Shop happiness UI (W4-g58)"
ANALYSIS section, closed or censused.

**Renumbered 143 → 240** per the research/150 collision registry
(`origin/task/W4-d54-dust-sites-world` tip `38bfd1d22`, §2's one-sided rule:
"if those lanes later push them, they MUST take free numbers 237+"). The
registry scan found 143 ONE-SIDED (c4's `143-d40-crimsongoldfish-retire-hunk.md`
first-add 09:02:12 keeps the number); a fresh 356-tip `git ls-tree` scan this
round finds FIVE 143 claimants pushed after the scan — c4 (keeps 143),
d120 `143-d120-chatter-retire.md` (09:46:26 → 237), d42
`143-d42-roster-sheets.md` (09:50:22 → 238), c8
`143-c8-crawler-final-census.md` (10:03:55 → 239), g58 this doc (10:26:59 →
**240**, fourth loser in first-add order). Numbers 237–249 are free at every
tip; the 237/238/239 assignments are recorded here for those lanes/merger —
each renames on its OWN branch. Content below is unchanged from the 143
edition; refs updated (INDEX row, ANALYSIS §citations, the two code comments).

Refs examined (origin tips at the round's start):

| Lane | Tip | Scope |
| --- | --- | --- |
| g58 (this branch) | `84db0ffb` → `468db5928` (this round) | the happiness UI + DATA + latch rounds |
| f37 price-math | `908482d52` | the ProcessMood core + research/68 reconcile |
| d122 shop-restock | `d3b7903b2` | the savings keep-one (research/142) + the medal wallet |
| g12 dust-main-sites-2 | `f24dd76ac` | the doc-sweep savings pile (`b467699f2`) |
| d85 ui-screens-2 / g53 ui-achievement | `aeadb43a6` / `04013f53d` | the UI co-lanes (portrait census §4) |

## 1. The buy/sell-lane latch repoint — LANDED (`468db5928`)

The g58 latch round (`13be13a6a`) cross-noted the buy lane:
`Player.GetItemExpectedPrice` (`Player.cs:35359-35398`) reads
`currentShoppingSettings.PriceAdjustment` at `:35382-35383` (the buyOnce
arm, both directions DIVIDE) and `:35387-35388` (the plain arm: buy
MULTIPLIES, sell DIVIDES). `currentShoppingSettings` has exactly ONE write
site — `Player.SetTalkNPC` (`Player.cs:4383-4390`). The shop button's
Condition is `TalkNPCType == this._npcType` (`NPCInteractions.cs:59-62`,
the `OpenShop` action), so the talk latch is ALWAYS live at a vanilla
shop transaction; `Main.OpenShop` (`Main.cs:40459-40467`) does not
recompute it.

**Ownership census (the "shop-transaction-lane owner?" question):** the
buy/sell TRANSACTION halves (`App::buy_shop_item`/`sell_item`,
`app/mod.rs`) are S1-P-i44's port — g58's own files (the i44 witnesses
live in `app/mod.rs::s1_p_i44_tests` + `app/clicks.rs::
s1_p_i44_shop_click_tests`). f37's price-math lane owns `shop.rs`'s
`get_item_expected_price`/`row_prices`/`sell_payout` helpers and the
`shop_mood` capture seam, but its tip (`908482d52`) still feeds
`self.shop_mood` into `buy_shop_item`/`sell_item` — the repoint was NOT
claimed there (research/68 §1's "Nurse heal cost reads the latch" row
covers the nurse only; the buy/sell rows stay on the capture). d122's
currency lane (`565dd6b0c`/`91fc771a6`) owns the medal-wallet buy arm
(`buy_with_currency`), a DIFFERENT function — its coin buys ride
`buy_shop_item` too but its tip keeps `shop_mood`. **Unclaimed → g58
repoints its own call sites** (the transaction halves), leaving the
`shop.rs` helpers and the `shop_mood` field untouched (the f37 merge side
keeps its capture; research/68 §6's "both fields" resolution stands).

What landed:

- `buy_shop_item` (`app/mod.rs`): the charge reads
  `self.shopping_settings.price_adjustment` (was `self.shop_mood`).
- `sell_item` (`app/mod.rs`): the payout's adjustment likewise (the
  `Player.SellItem` → `GetItemExpectedPrice` chain, `Player.cs:35170`).
- The shop chrome tuple (`app/frame/draw.rs`): `Main.cs:41793` reads the
  LATCHED `currentShoppingSettings.PriceAdjustment` — the tuple's mood
  slot now carries `shopping_settings.price_adjustment` (the g58 chrome
  was already latch-shaped in intent; the feed was the stale capture).
- The shop-open capture (`world_sim/npcs.rs::try_open_town_shop_at_cursor`):
  gated `if app.talk_npc != Some(idx)` (the f37 capture-once contract,
  research/68 §4 — `Main.OpenShop` does not recompute; an ungated second
  pass would double-draw the Princess's `Main.rand` loves). The gated
  arm builds the FULL `shopping_settings` (multiplier + report) through
  the same `happiness::shopping_settings` builder the talk latch uses,
  so the port's collapsed chat-button entry (the recorded #9 deviation —
  the shop opens with `talk_npc = None`) still prices correctly.
  `shop_mood` mirrors the multiplier (the f37 field stays honest).
- The Angler reward rarity (`world_sim/npcs.rs::give_angler_reward`,
  `Player.cs:56306`): the constants-off `1.0` was STALE — the Angler's
  rows ARE ported (`happiness.rs`: Like Ocean / Dislike Desert,
  `PersonalityDatabasePopulator.cs:97-101`; the neighbor switch
  `AllPersonalitiesModifier.cs:141-158`). The vanilla turn-in is the
  AnglerQuest chat button (`Condition: TalkNPCType == 369`,
  `NPCInteractions.cs:137-140`), so the latch is live; the read is now
  `app.shopping_settings.price_adjustment` (a headless no-latch call
  keeps NotInShop 1.0).

The remaining latch readers stay with their lanes: the Stylist hair
price (`Main.cs:43799`, f37's `hair_restyle_cost` — the hair-window UI is
the M4-TOWN-17 deviation) and the nurse/tax/reforge (already repointed by
g58's latch round).

Witness (WRITTEN NOT RUN): `app/mod.rs::s1_p_i44_tests::
the_buy_and_sell_read_the_talk_latch_not_the_shop_capture` — a homeless
Merchant latched at the 1.5 NoHome ceiling: the Mining Helmet 88 (40000)
buys at 40000×1.5 = 60000 (NOT the stale capture's 40000); the sell
divides 40000/1.5f = 26666.666 → banker's 26667, /5 = 5333; the sellback
memo refunds the bought unit at its 60000 buy price (the round trip is
purse-neutral); a post-latch housing change is inert; a re-latch picks
the housed LoveSpace 0.95 up (the flow world's player centre, tile 4,3,
is in the beach band — x=4 < 380 and y=3 ≤ the ocean level
(2+3.6)/2+40 = 42.8 — so `zone_beach` holds and Forest fails; the
Merchant carries no Ocean row, so the housed-alone mood is LoveSpace
×0.95 → the buy charges 38000). The f32 expectations
were python-struct-verified (40000×1.5f = 60000.0 exact; 40000/1.5f =
26666.666015625; 40000×0.95f = 38000.0 exact).

## 2. The happiness REPORT UI census — COMPLETE vs the port

The task's "mood-icons/price-slider/neighbor-list" framing does not match
the C#: **there is NO price slider and NO neighbor-list widget anywhere
in the vanilla happiness UI.** A full `decompiled/Terraria` sweep for
`HappinessSlider|PriceSlider|NeighborList|MoodSlider|HappinessPanel|
UIHappiness|HappinessUI` returns ZERO hits. The happiness report is a
TEXT report shown in the chat panel; the only graphical happiness
elements are the shop chrome's 4-frame mood icon + the P0 percent. The
complete C# surface (every `HappinessReport`/`NPCHappiness`/
`CheckHappiness` reader):

| # | C# site | What it draws | Port | Status |
| --- | --- | --- | --- | --- |
| 1 | `Main.cs:41790` | the `Lang.inter[28]` "Shop" label at (504, invBottom) | `ui/shop_happiness.rs::draw` | COMPLETE |
| 2 | `Main.cs:41791` | `ItemSlot.DrawSavings(504, invBottom)` — the coin pile | `ui/savings.rs::draw_savings` (vertical) | COMPLETE (the 3-way keep-one: §3) |
| 3 | `Main.cs:41792-41796` | the `UI/NPCHappiness` 4-frame icon, band ≤0.82/≤1/≤1.1/else, centre origin at (504+12, invBottom+108) | `ui/shop_happiness.rs::icon_frame`/`icon_pos` + the strip draw | COMPLETE |
| 4 | `Main.cs:41797` | `priceAdjustment.ToString("P0")` at (504+26, invBottom+98) | `ui/shop_happiness.rs::price_percent_text`/`text_pos` | COMPLETE |
| 5 | `NPCInteractions.cs:289-312` | the `ReportHappiness` chat button (Condition `!CanShowHomelessText && report != ""`, label `UI.NPCCheckHappiness`, Interact sound 12 + `npcChatText = report` + hop) | `ui/npc_chat.rs::happiness_button_rect`/`draw_interaction_button` + `app/clicks.rs::handle_npc_chat_clicks` | COMPLETE (the hop landed this round, §4) |
| 6 | `ShopHelper.cs:188-196` + the report rows | the report TEXT (the `TownNPCMood_{name}` categories, the `{BiomeName}`/`{NPCName}` substitutes, the prepend order) | `happiness.rs::happiness_report`/`prepend_report` | COMPLETE |
| 7 | `NPCChatPanel.cs:52-86` | the panel that DISPLAYS the report text (the ChatBack splices, the wrapped lines, the button row) | `ui/npc_chat.rs::draw_panel`/`panel_lines` | COMPLETE |
| 8 | `Main.cs:40635-40672` | the LEGACY `DrawNPCChatButtons` happiness arm (`npcChatFocus4`) | — | DEAD CODE — `DrawNPCChatButtons` (`Main.cs:40474`) has NO callers in the whole decompilation (the `NPCChatPanel.DrawButtons` path replaced it); the `:40635` `HappinessReport == ""` gate lives inside that dead function. NOT ported, correctly. |

The report rows themselves carry NO rich-text color codes (the en-US
`TownNPCMood_Guide` rows are plain prose with `{BiomeName}`/`{NPCName}`
substitutes — verified in `Terraria.Localization.Content.en-US.Town.json`),
so the panel's `DrawColorCodedStringWithShadow` path renders them as
plain wrapped lines — the port's `draw_panel` shadow-ring + tint matches.
The `UI.NPCCheckHappiness` label is " Happiness" (a leading space, the
en-US row) — the port reads it through `crate::lang::text`, verbatim.

**Verdict: the report UI is COMPLETE.** No missing halves to port. The
only happiness-adjacent draw NOT in the port is the portrait (§4) and the
corner item (`Main.DrawNPCChatBottomRightItem`, `Main.cs:39789` — the
quest-item display, S1-E-10b's deferral, not a happiness element).

## 3. The savings 3-way input — the g58 CALL-SITE census (142-input)

d122's research/142 (`142-savings-keep-one.md`, tip `d3b7903b2`) already
censused the C# callers and decided the keep-one (KEEP d122's both-arms
module + g58b's faithful count-text origin). This section publishes the
**g58 side's call-site census** as the 142-input — the three Rust
`draw_savings` ports and where each wires:

| Port | Module | Arms | Call site(s) wired | Count-text origin | Coin strip |
| --- | --- | --- | --- | --- | --- |
| **g58** (`1328b118`/`0d683ec4`, this branch) | `ui/savings.rs` (319 lines) | VERTICAL only | the SHOP-PANEL `(504, invBottom)` call, through `ui/shop_happiness.rs::draw` (`:41791`, between the Shop label `:41790` and the icon `:41792`) | FAITHFUL: `ty = shopy + 75 + ls*0.75 − ls*scale` (`draw_money_vertical`, the scaled-origin formula) | `coin_strip_frame` (no sizeLimit clamp, no static fallback) |
| **d122** (`10420b965`→`d3b7903b2`) | `ui/savings.rs` (both arms + the medal dispatch) | HORIZONTAL + VERTICAL + the `_customCurrencyForSavings` medal arm | the REFORGE `(num51+130, invBottom)` = (180, 258) horizontal call (`:41637`) AND the shop-panel `(504, invBottom)` vertical call (`:41791`, through its own `ui/shop_happiness.rs` port) | FIXED to g58b's formula this round (`money_text_pos` takes the draw scale, both arms) | `coin_anim_uv` (the sizeLimit clamp + the `Item_71..74` static fallback — the superset) |
| **g12** (`b467699f2`, tip `f24dd76ac`) | `ui/savings.rs` (both arms) | HORIZONTAL + VERTICAL | the SHOP-PANEL `(504, 258)` vertical call (`:41791`), wired DIRECTLY in `ui/mod.rs`'s shop arm (NOT through a chrome module — g12 drew the pile before the g58 chrome lane) | the origin passes `(0, LineSpacing)` to `border_four_way`, which subtracts `origin*scale` — the SAME faithful net formula | `coin_strips` (the sizeLimit guard kept, never binds at 1024) |

**The C# caller census (ALL `ItemSlot.DrawSavings` call sites — exactly
three, all in `Main`, re-verified this round):**

| # | Site | Coordinates | Arm | Gate | Port coverage |
| --- | --- | --- | --- | --- | --- |
| 1 | `Main.cs:41637` (reforge menu, `DrawInventory`) | `num51 + 130, invBottom` = (180, 258) | `horizontal: true` | `Main.reforgeItem.type > 0` (`:41584`), menu open | d122 ONLY (g58/g12 are vertical-only) |
| 2 | `Main.cs:41791` (shop panel, `DrawInventory`) | `504, invBottom` = (504, 258) | vertical (default) | `npcShop > 0 && !PipsUseGrid` (`:41788`) | ALL THREE (g58 via the chrome, d122 via its chrome, g12 direct) |
| 3 | `Main.cs:43862` (`DrawHairWindow`) | `num18, num19 - 70` | `horizontal: true` | `num11 > 0` (the hairstyle price, `:43860`) | NONE — the hair window is unported (M4-TOWN-17); d122's horizontal arm covers its shape |

**The `DrawMoney` caller census (the savings-adjacent surface — three
callers, re-verified):** `ItemSlot.cs:3739` (the `DrawSavings` tail,
`fromSavings: true` — all three ports), `NPCChatPanel.cs:142` (the
interaction-button coin display, `TryAddCoins` — the nurse/heal cost
buttons, unported with the button-cost lane), `CoinLossRevengeSystem.cs:
272` (the revenge-marker coin pile, horizontal — the port's `revenge.rs`
keeps the markers but the DRAW is unported). Neither non-savings caller
is a happiness element; both stay with their lanes.

**Keep-one recommendation (the 142-input, g58's vote):** KEEP d122's
module (the both-arms + medal superset, the count-text origin already
fixed to g58's formula) — research/142's decision stands. g58's
`ui/savings.rs` and g12's are the SAME vertical-arm math at the SAME
(504, 258) call; the merge keeps ONE `draw_savings` (d122's signature,
the `currency` latch parameter) and ONE shop chrome. **g58's chrome
(`ui/shop_happiness.rs`) is the report-lane half d122 explicitly did NOT
port** (its module header: "g58b's happiness_button/npc_chat report gate
is NOT part of this port — it belongs to g58b's report lane"); the merge
keeps g58's `happiness_button` Condition feed (`app/frame/draw.rs`) +
`ui/npc_chat.rs` panel gate, and d122's chrome body adapted to the
merged savings signature (the two `shop_happiness::draw` bodies differ
ONLY in the savings-call signature — research/142 §2's cross-note). The
`assets.rs` coin-strip loaders are the same four `Coin_0..3` files under
three names (`coin_strips`/`coin_strip_frame` g58+g12, `coin_anim`/
`coin_anim_uv` d122) — keep ONE (d122's, the clamp+fallback superset).

**g58's own call-site count: ONE** — the shop-panel (504, invBottom)
vertical call, through the chrome. g58 does NOT wire the reforge (180,
258) call (d122's) nor the hair-window call (unported). The
`draw_savings` symbol has exactly ONE caller in g58's tree
(`ui/shop_happiness.rs:103`), verified by `git grep`.

## 4. The portrait-hop census — the state LANDED, the draw deferred

The "happiness-portrait NPC-head-hop anim" is `Main.DrawNPCPortrait`
(`Main.cs:40318-40447`) + `Main.DoNPCPortraitHop` (`:40675-40678`). The
full census:

**The hop trigger sites (every `DoNPCPortraitHop` caller):**

| C# site | Context | Port |
| --- | --- | --- |
| `Player.cs:4401` | `SetTalkNPC`'s portrait arm (the `NPCPortraits.TryGetValue` gate) | LANDED (`set_talk_npc`, `468db5928`) |
| `NPCInteractions.cs:310` | `ReportHappiness.Interact` | LANDED (`handle_npc_chat_clicks`) |
| `NPCInteractions.cs:99`/`:123` | the Dryad Stardew/Purification buttons | UNPORTED (the Dryad chat buttons, S1-E-10b) |
| `NPCInteractions.cs:217` | `GuideTip.Interact` | UNPORTED (the Guide tip button) |
| `NPCInteractions.cs:332` | `RequestHome.Interact` | UNPORTED (the housing button) |
| `Main.cs:39999` | `NPCChatText_TavernkeepAdvice` | UNPORTED (the Tavernkeep advice) |
| `Main.cs:40050` | `NPCChatText_PartyGirlSwapMusic` | UNPORTED (the music swap) |
| `Main.cs:40177` | `NPCChatText_DoNurseHeal`'s tail (UNCONDITIONAL — every arm) | LANDED (the nurse dispatch) |
| `Main.cs:40224` | `NPCChatText_DoTaxCollector`'s tail (both arms) | LANDED (`tax_collector_payout`) |
| `Main.cs:40262` | `NPCChatText_DoAnglerQuest`'s tail | LANDED (`angler_turn_in`) |
| `Main.cs:40267` | `DryadText_WorldPurityOrStardewValleyBit` | UNPORTED (the Dryad purity text) |

**The animation math (ported this round, `ui/npc_chat.rs`):**
`npcChatPortraitFrameCounter` (`Main.cs:1771`) reads THEN increments per
drawn frame (`:40335`); `DoNPCPortraitHop` resets it to 0. The lift
(`:40355-40359`): while `num4 = num/120 == 0` (the first 120 frames),
`num3 = num%120`, the centre lifts `-56·0.25·(1 − EaseOutBounce(
clamp(num3,0,80)/80))` px. `EaseOutBounce` (`Utils.cs:355-363`) =
`BounceEaseOut(x, 4, 2.0)` = `1 − (1−t)²·|sin(4πt)|`. Past frame 119 the
lift is 0 and STAYS 0 (the `num4 == 0` gate — no repeat). The C#
`if (0 != 0)` wobble (`:40347-40354`) and `flag2 = false` sway
(`:40360-40372`, `Utils.GetPortraitMovement` `:375-410`) are DEAD CODE —
dropped. Ported: `App.npc_chat_portrait_frame_counter` + the per-frame
tick (`app/frame/draw.rs`, gated `talk_npc.is_some()` — the panel draws
iff talking, and every ported town kind has an `NPCPortraits` row so the
provider/texture null-gates never bind) + `ease_out_bounce` +
`portrait_hop_lift` + `do_npc_portrait_hop`.

**The DRAW (deferred — no portrait system):** `DrawNPCPortrait` draws
the `NpcPortraitBackground` + the portrait texture (the `Detailed` mode,
`:40375-40377`), or a scissored `DrawNPCDirect` dummy at scale 3 (the
`CloseUp` mode, `:40379-40416`) or scale 2 (the `FullBodyRetro` mode,
`:40417-40443`), + the name label (`:40444-40446`). The
`DialoguePortraitPreference` setting (`Main.cs:2331`, the
`CycleNPCPortraitMode` `:40297-40316` four-way cycle) and the
`NPCPortraits`/`NPCPortraitsCloseUpOffsets`/`NPCPortraitsFullBodyRetroOffsets`
dictionaries (`NPCID.cs:5894`/`:10180`/`:10343`, the shimmer-conditioned
providers) are the portrait SYSTEM — unported (S1-E-10b's deferral). The
`NPCPortraits` dictionary covers 40 ids (the town humans + the pets +
the bound/sleeping forms + the Travelling/Skeleton merchants + the Old
Man + the town slimes); every ported town kind is in it, so the hop's
`TryGetValue` gate is always-true for the ported roster.

**The d85/g53 co-lane check:** both lanes' `assets.rs` carry the BESTIARY
portraits (`UI/Bestiary/Portrait_Front`, the `bestiary_custom_portraits`
map, the 22 NPC portrait sheets) — a DIFFERENT system (the bestiary info
page's portrait, not the chat portrait). Neither ports
`Main.DrawNPCPortrait`/`DoNPCPortraitHop`/`npcChatPortraitFrameCounter`
(a `git grep portrait` over both tips returns only the bestiary rows).
The chat-portrait lane is UNCLAIMED — g58 ports the hop STATE (the
animation clock + math, the small faithful half) and documents the draw
deferral; the draw itself rides the portrait-system lane (S1-E-10b's
successor) with the hop state already faithful.

Witnesses (WRITTEN NOT RUN): `ui/npc_chat.rs::
the_portrait_hop_lift_matches_the_bounce` (the ease at its exact
quarter-points — t=0.125 → 0.234375, t=0.375 → 0.609375, t=0.625 →
0.859375, the sin zeros at the quarters → 1; the lift frame 10 →
-10.71875, frame 30 → -5.46875, frame 50 → -1.96875, frame 80+ → 0; the
num4 gate frame 120/130/1000 → 0) + `the_portrait_hop_resets_on_the_talk_
and_the_report` (the talk open resets a mid-hop counter, the Happiness
button resets it again, the close does NOT hop — the provider-clear is
hop-free).

## 5. Cross-lane notes for the merger

- The buy/sell repoint (§1) touches `app/mod.rs`'s transaction halves —
  f37's `shop.rs` helpers and `shop_mood` field are UNTOUCHED, so the
  f37 merge keeps its capture seam; the merged `buy_shop_item`/`sell_item`
  read the latch (g58's repoint wins, it is the C#-faithful read).
- The savings keep-one (§3) is research/142's decision — g58 votes KEEP
  d122's module; the merge keeps g58's report-lane chrome gate + d122's
  chrome body/savings signature.
- The portrait hop state (§4) is g58-only (no co-lane claim); the merge
  takes it as-is, the draw deferral stands.
- The report UI (§2) is COMPLETE — no missing halves; the legacy
  `DrawNPCChatButtons` is dead code (no callers), correctly unported.