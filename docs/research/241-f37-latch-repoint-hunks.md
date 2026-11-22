# 241 — The f37 latch-repoint hunks (W4-g58 retire, merger input)

Date: 2026-11-21. Code-only. **Target: `task/W4-f37-shop-price-math` tip
`908482d52`** (first-add of research/68 02:48:30 — the key-68 registry WINNER
per research/150 §1; tip age at this round's probe: ~8.5 h, no newer commit
at any fetch this session). The lane is PARKED, not confirmed dead: its
research/68 mood-reconcile is published and its tip is the registry's own
scan row, so the repoint ships as EXACT HUNKS for the f37 lane/merger to
apply — g58 does not touch the f37 branch (the registry's own-branch rule).

## 1. What g58 already landed (the merge-side truth)

g58's `468db5928` repointed ITS OWN transaction halves (`app/mod.rs`
`buy_shop_item`/`sell_item`, the S1-P-i44 files) + the shop chrome tuple
(`app/frame/draw.rs`) + the shop-open capture (`world_sim/npcs.rs
try_open_town_shop_at_cursor`, gated `talk_npc != Some(idx)`, builds the FULL
`shopping_settings`) + the Angler rarity (`Player.cs:56306`) from the
shop-open `shop_mood` capture to the TALK-OPEN LATCH
(`shopping_settings.price_adjustment` — `Player.GetItemExpectedPrice`
`Player.cs:35382-35388` reads `currentShoppingSettings.PriceAdjustment`,
whose only write site is `SetTalkNPC` `:4389`; the shop button Condition
`TalkNPCType == type`, `NPCInteractions.cs:59-62`, keeps it live at every
vanilla transaction). Full census: research/240 §1 (renumbered from 143).

f37's tip still feeds `self.shop_mood` at FIVE non-test read sites (below).
The merger takes g58's repointed halves; these hunks bring f37's OWN
remaining readers to the same C#-faithful latch so the f37 side composes
without stale-capture reads. **The `shop_mood` FIELD and the capture writes
STAY** (research/68 §6's both-fields resolution; g58's capture mirrors
`shop_mood = settings.price_adjustment` so the f37 field stays honest).

## 2. The exact hunks (f37 tip `908482d52` line numbers, 1-indexed)

All five are one-line expression swaps inside otherwise-untouched
functions. The field/constructor/capture sites (`app/mod.rs:211`,
`:1961`, `:3604`, `app/flow.rs:2204`, `app/clicks.rs:1101`,
`world_sim/npcs.rs:187`/`:231`/`:947`) are NOT touched.

### Hunk 1 — `crates/tr-app/src/app/mod.rs:1696-1697` (`buy_shop_item`)

```
-        let price =
-            crate::shop::buy_price_row(entry, self.discount_available, self.shop_mood) as u64;
+        // `Player.GetItemExpectedPrice` (`Player.cs:35382-35388`) reads the
+        // TALK-OPEN latch (`SetTalkNPC` `:4389`), not the shop-open capture.
+        let price = crate::shop::buy_price_row(
+            entry,
+            self.discount_available,
+            self.shopping_settings.price_adjustment,
+        ) as u64;
```

### Hunk 2 — `crates/tr-app/src/app/mod.rs:1759` (`sell_item`'s payout args)

```
             self.discount_available,
-            self.shop_mood,
+            self.shopping_settings.price_adjustment, // `Player.cs:35387-35388`
             item.count,
```

### Hunk 3 — `crates/tr-app/src/app/mod.rs:1849` (`reforge_click`'s cost args)

```
             self.reforge_slot.count.max(1),
-            self.shop_mood,
+            self.shopping_settings.price_adjustment, // `Main.cs:41591`
             self.discount_available,
```

(g58's latch round `13be13a6a` already repointed its own reforge charge +
draw price; this hunk is the f37-side twin. The reforge DRAW price below
follows.)

### Hunk 4 — `crates/tr-app/src/app/mod.rs:2090` (`nurse_heal_cost`)

```
         nurse_heal_cost_for(
             missing,
             debuffs,
-            self.shop_mood,
+            self.shopping_settings.price_adjustment, // `Main.cs:40100`
             &NurseCostFlags {
```

Also update the doc comment `:2076-2079` ("W4-f37: the adjustment is the
CAPTURED … `shop_mood` latch from the shop/talk open") to name the
`shopping_settings.price_adjustment` TALK latch (g58's `3ac5432d5` nurse
recompute already reads it on the g58 side).

### Hunk 5 — `crates/tr-app/src/app/frame/draw.rs:1694` + `:1746` (the two chrome/price tuples)

`:1691-1697` — the shop tuple:

```
                             self.shop_npc.map(|_| {
                                 (
                                     self.shop_stock.as_slice(),
-                                    self.shop_mood,
+                                    // `Main.cs:41793` reads the LATCHED
+                                    // `currentShoppingSettings.PriceAdjustment`.
+                                    self.shopping_settings.price_adjustment,
                                     self.discount_available,
                                 )
                             }),
```

`:1738-1748` — the reforge-menu cost:

```
                                             self.reforge_slot.count.max(1),
-                                            self.shop_mood,
+                                            self.shopping_settings.price_adjustment,
                                             self.discount_available,
```

## 3. The f37-side prerequisite: the latch must EXIST on the f37 tree

f37's tip has NO `shopping_settings` field (its `happiness.rs` carries
`process_mood` only — the `ShoppingSettings` struct + the
`shopping_settings` builder + `App.shopping_settings` + `set_talk_npc` are
g58's, `crates/tr-app/src/happiness.rs:623`/`:836` +
`app/mod.rs:178-186`/`:2006-2027` at g58 tip). Two apply orders:

- **Merger order (recommended):** merge g58 first (it carries the field +
  builder + the repointed transaction halves); the five hunks above then
  resolve the f37-side residual readers inside the merge — Hunk 1/2 fold
  into g58's already-repointed functions (keep g58's body), Hunk 3/4/5 apply
  to f37's `reforge_click`/`nurse_heal_cost`/draw tuples if the merge kept
  the f37 bodies.
- **f37-lane order (if the lane revives):** port g58's `ShoppingSettings` +
  `shopping_settings` + `App.shopping_settings` + `set_talk_npc` first
  (g58 `000c4a515`/`3ac5432d5`/`13be13a6a` carry them), then apply the five
  hunks; the close-site resets (`app/clicks.rs:1101`,
  `world_sim/npcs.rs:187`) must ALSO reset `shopping_settings` to
  `ShoppingSettings::default()` beside the `shop_mood = 1.0` write (g58's
  `set_talk_npc(None)` does both).

## 4. Sites deliberately NOT repointed

- `world_sim/npcs.rs:81-119` `tax_collector_payout`: f37's copy is the
  plain-1.0 constants-off payout spawning at the COLLECTOR's centre — it
  never reads `shop_mood`, so there is no hunk; it is SUPERSEDED by g58's
  `a027138d8` (the `/ PriceAdjustment` divide `Main.cs:40184` + the
  player-centre spawn `:40192`/`:40199`/`:40206`/`:40215`). The merge keeps
  g58's function whole.
- `shop.rs` helpers (`get_item_expected_price`/`row_prices`/`sell_payout`/
  `buy_price_row`/`reforge_cost`/`hair_restyle_cost`) take
  `price_adjustment: f32` PARAMETERS — pure price math, no field reads;
  nothing to change (research/240 §1's ownership census).
- `hair_restyle_cost` callers: the hair window is unported (M4-TOWN-17);
  when it lands its `Main.cs:43799` read is the latch too.
- The capture writes (`world_sim/npcs.rs:231` talk-open, `:947` shop-open
  gated arm): stay — they keep `shop_mood` honest for any f37-side reader
  the merge keeps; g58's gated arm also builds the full settings.
- The f37 witness block (`app/mod.rs:5708-5849`): the capture-feed tests
  (`app.shop_mood = 0.75` then buy) need the same one-line feed swap to
  `app.shopping_settings.price_adjustment = 0.75` when the hunks apply —
  otherwise they test the dead capture. g58's replacement witness
  (`the_buy_and_sell_read_the_talk_latch_not_the_shop_capture`,
  `s1_p_i44_tests`) pins the latch semantics end-to-end.

WRITTEN NOT RUN (code-only retire mandate; no cargo/tests).