# 73 — Doc-sweep town lane 2 (W4-f37)

Date: 2026-11-21. Code-only sweep — no cargo/tests run; every claim below is
re-read from the C# source and the pushed lane tips. The witnesses are
written (not run — the code-only swarm).

**Scope.** Mine `docs/ANALYSIS.md` + `docs/research/23/11/22/29` + the
shop/town docs for unported town mechanics with a C# source, port the REAL
gaps verbatim + witnesses, and record the co-lane claims / dead code /
recorded deviations so the merger has a clean ledger. The lane is the
doc-sweep continuation of W4-f37 (the shop price-math lane); the four ports
below are the gaps no pushed lane covers.

**Method.** For each candidate mechanic: (1) read the whole C# function +
its callers, (2) grep every pushed `origin/task/*` tip for the symbol /
C# cite to find the owning lane, (3) port only the unclaimed half verbatim
with a citation comment + a witness, (4) cross-note the co-lane halves.
The branch list IS the claim ledger (AGENTS.md) — a pushed branch touching
the symbol owns it; this lane never duplicates a pushed claim.

## The sweep table

| Mechanic | C# cite | Port status at sweep | Verdict | Lane / commit |
| --- | --- | --- | --- | --- |
| CollectTaxes anniversary ×2 (rate + cap) | `Player.cs:24133-24137` | MISSING — `collect_taxes` pinned 50c/25g | **PORTED** — `SeedFlags::TENTH_ANNIVERSARY` doubles both, the `>=` gate + `>` clamp ride the doubled cap | f37 `4aad255ee` |
| CollectTaxes payer filter | `Player.cs:24145` | WRONG — `is_town_resident` dropped the housed DD2 Bartender (550) | **PORTED** — `!homeless && !IsTownPet && TypeToDefaultHeadIndex>0`; the bound/rescue six excluded by kind (freed-row `net_id`s) | f37 `4aad255ee` |
| Nurse heal HealEffect text | `Main.cs:40112` | MISSING — the paid arm filled life with no combat text | **PORTED** — `spawn_heal_text_on` (the `potions.rs` seam, the caller's global stream), the missing amount BEFORE the fill | f37 `b21187fcc` |
| Nurse heal dialog ladder 227-230 | `Main.cs:40113-40128` | MISSING | **PORTED** — the PRE-heal life-band, strict `<` f64 compares, the exact-quarter boundaries fall to the next band | f37 `b21187fcc` |
| Nurse poor-purse dialog 52-54 | `Main.cs:40142-40154` | MISSING | **PORTED** — `Next(3)` on the global stand-in | f37 `b21187fcc` |
| Nurse nothing-to-heal dialog 55-57 + ChildSafety | `Main.cs:40159-40175` | MISSING | **PORTED** — `Next(3)` then RE-DRAW `Next(1,3)` under ChildSafety (both draws spend the stream, the blood line 55 drops) | f37 `b21187fcc` |
| Nurse sound-12 head + Item4 success cue | `Main.cs:40105`/`:40111` | claimed | **CO-LANE** — g21's `nurse_heal` hunk (the sound-12 FIRST + the Item4 `SOUND_STYLES` lookup) | g21 (not duplicated) |
| Nurse portrait hop | `Main.cs:40177` | claimed | **CO-LANE** — g58's `DoNPCPortraitHop` | g58 (not duplicated) |
| Tax payout dialog 380-383 (paid) | `Main.cs:40217` | MISSING | **PORTED** — `Next(380,384)` AFTER the coin split BEFORE the purse zero, global stand-in | f37 `79ebe0782` |
| Tax payout dialog 390-400 (empty) | `Main.cs:40222` | MISSING | **PORTED** — `Next(390,401)`, no coin spawns | f37 `79ebe0782` |
| Tax payout latch divide | `Main.cs:40184` | claimed | **CO-LANE** — g58 `a027138d8` (`tax_collector_payout` divides by `shopping_settings.price_adjustment`) | g58 (not duplicated) |
| Tax payout player-centre origin | `Main.cs:40192` | claimed | **CO-LANE** — g58 `a027138d8` (the spawn origin switches to `Main.LocalPlayer.Center`) | g58 (not duplicated) |
| Tax portrait hop | `Main.cs:40224` | claimed | **CO-LANE** — g58's `DoNPCPortraitHop` | g58 (not duplicated) |
| Angler reward `(mood+1)/2` rarity | `Player.cs:56306` | claimed | **CO-LANE** — g58 `468db5928` (the latch read; `angler.rs:164` already carries `(price_adjustment+1)/2`) | g58 (not duplicated) |
| Zoologist `altTexture==2` report suffix | `ShopHelper.cs:191-194` | claimed | **CO-LANE** — g58's `should_bestiary_girl_be_lycantrope` + the `Transformed` category suffix (the predicate `UpdateAltTexture` writes `altTexture=2` on) | g58 (not duplicated) |
| `NPC.UpdateAltTexture` state machine | `NPC.cs:90210-90234` | claimed | **CO-LANE** — g61's "alt-texture halves" claim (the transition effects `MakeTransitionEffectsForTextureChanges` + the `UpdateAltTexture` tick); f37 adds NO `alt_texture` field | g61 (not duplicated) |
| Reforge latch read | `Main.cs:41591` | claimed | **CO-LANE** — g58 `13be13a6a` (the reforge price reads `shopping_settings.price_adjustment`) | g58 (not duplicated) |
| Hair-window open/cancel/buy state | `Main.cs:43653-43693` | MISSING | **PORTED** — `hair_window.rs` open/cancel/buy (the latch, the Stylist-only talk close, the bought style stays) | f37 `b8daa7855` |
| Hair-window force-close gate | `Main.cs:43711-43715` | MISSING | **PORTED** — the six conditions cancel; `close_shop` rides the same seam | f37 `b8daa7855` |
| Hair-window price + Buy label | `Main.cs:43786-43852` | price helper existed (f37 phase 3), the window read MISSING | **PORTED** — `restyle_price` resolves the change flags vs the latch; `buy_label` splits the coins into `Lang.inter[15..18]` + `GameUI.BuyWithValue` | f37 `b8daa7855` |
| Hair-window HSL seed/recompose | `Main.cs:44001-44092` | MISSING | **PORTED** — the −1 bars seed from `rgbToHsl`, the recompose writes `hairColor` (the `ui::color` fns) | f37 `b8daa7855` |
| Hair-window Buy/Cancel click tail | `Main.cs:43974-43989` | MISSING | **PORTED** — the Buy hover charges the purse + closes, the Cancel hover restores | f37 `b8daa7855` |
| Stylist hairstyle list | `HairstyleUnlocksHelper.cs:47-112` | MISSING (the char-creation base list existed) | **PORTED** — `stylist_hairstyles` = the 213 base + Plantera 4 + Martian 10 + Moonlord 1 | f37 `b8daa7855` |
| `Player.hair`/`hairColor` fields | `Player.cs:2568`/`:2554` | MISSING | **PORTED** — the fields land, the hair-layer tint rides `PlayerCompositeState.hair_color`, the `render_key` gates the tint mix on non-default (G6 bit-identical default frames) | f37 `b8daa7855` |
| Hair-window scroll step + cell pick | `Main.cs:43726-43785`/`:44110-44119` | claimed | **CO-LANE** — g22's `hair_window_scroll`/`hair_cell_pick` pure fns (`main_sounds.rs`) | g22 (not duplicated) |
| Hair-window panel DRAW | `Main.cs:43709-44138` | deferred | **RECORDED DEVIATION** — M4-TOWN-17 (the HairStyleBack panel, the 15-cell grid, the HSL slider rows, the Buy/Cancel labels); the state machine lands unwired behind witnesses | M4-TOWN-17 |
| `Main.UnlockedMaxHair` | `Main.cs:43695-43707` | dead code | **DEAD CODE** — definition only, ZERO callers in the whole decompilation (the window reads `AvailableHairstyles.Count`); unported | — |
| Hair persistence across world swap | `Player.cs:54788` (save) | deferred | **RECORDED DEVIATION** — the `.trplayer` sidecar carries no hair row (M6-SAVE); a bought style does not survive a swap | M6-SAVE |
| Town pets `housingCategory` filter | `TownRoomManager.cs:153-161` | inert | **RECORDED DEVIATION** — no category-1 kind ships; the filter is a no-op until a pet kind lands (g58 armed it) | pet lane |
| `shopSpecialCurrency` (defender medals) | `CustomCurrencyManager.cs` | deferred | **RECORDED DEVIATION** — the S1-C-11 note stands; d122 owns the medal-savings arm | d122 |

## The four ports (detail)

### 1. CollectTaxes anniversary pair + head-index payer filter (`4aad255ee`)

`Player.CollectTaxes` (`Player.cs:24129-24155`):
- `:24131-24137` — `num = Item.buyPrice(0,0,0,50)` (50c), `num2 =
  Item.buyPrice(0,25)` (25g); `Main.tenthAnniversaryWorld` doubles BOTH
  (`num2 *= 2; num *= 2;`). The port's `collect_taxes` reads
  `World.seeds.contains(SeedFlags::TENTH_ANNIVERSARY)`.
- `:24138` — the `>= num2` early return reads the DOUBLED cap.
- `:24145` — the payer count: `active && !homeless &&
  !NPCID.Sets.IsTownPet[type] && NPC.TypeToDefaultHeadIndex(type) > 0`.
  The former port used `is_town_resident`, which dropped the housed DD2
  Bartender (550, head 24 — he is NOT in `is_town_resident`). The bound/
  rescue forms (105/106/123/354/376/589) ride freed-row `net_id`s through
  the bestiary fold, so the head table alone would hand them a payer slot;
  the six are excluded by kind (the `ui/housing.rs:100-116` bound-trio
  rejection extended to the webbed/sleeping pair).
- `:24151-24153` — the strict `>` clamp to the (doubled) cap.

Witnesses: `the_anniversary_world_doubles_the_tax_rate_and_the_cap`
(3×50 vs 3×100, the plain-cap-not-the-gate, the 500000 clamp) +
`the_tax_payer_filter_counts_heads_not_town_flags` (Guide+Bartender=2,
homeless/dead/bound refused).

### 2. The nurse heal presentation tail (`b21187fcc`)

`Main.NPCChatText_DoNurseHeal` (`Main.cs:40103-40178`):
- `:40112` — `HealEffect(statLifeMax2 - statLife)` BEFORE the life fills
  (`:40129`). The `potions.rs::spawn_heal_text` seam gains
  `spawn_heal_text_on(app, amount, rng)` for the caller's stream — the
  click runs outside every swap, so the two `CombatText.NewText` position
  jitters (`CombatText.cs:88-89`) ride the bare-`Main.rand` global
  stand-in (S1-A-11 §4), NOT the UpdatePlayers swap.
- `:40113-40128` — the dialog band reads the PRE-heal life: `< 0.25` →
  227, `< 0.5` → 228, `< 0.75` → 229, else 230. The `(double)` compares
  are exact f64 promotions of the i32 pair; the exact-quarter boundaries
  fall to the NEXT band (strict `<`).
- `:40142-40154` — the poor purse: `Next(3)` → 52/53/54.
- `:40159-40175` — nothing-to-heal: `Next(3)` draws FIRST, then the
  ChildSafety arm RE-DRAWS `Next(1,3)` (both draws spend the stream — the
  C# does not reuse the first roll); the blood line 55 drops under the
  gate, only 56/57 answer.

The sound-12 head (`:40105`) + the Item4 success cue (`:40111`) are g21's
`nurse_heal` hunk; the portrait hop (`:40177`) is g58's. Witnesses:
`the_nurse_dialog_band_reads_the_pre_heal_life` (six band rows incl the
exact-quarter boundaries + the HealLife text amount) +
`the_nurse_refusals_draw_the_dialog_rolls` (twin-stream replay, one/two-
draw counts, the 56/57-only gate).

### 3. The tax payout dialog tail (`79ebe0782`)

`Main.NPCChatText_DoTaxCollector` (`Main.cs:40180-40225`):
- `:40217` — the paid arm answers `Lang.dialog(Main.rand.Next(380, 384))`
  AFTER the coin split, BEFORE the purse zero (`:40218`).
- `:40222` — the empty purse answers `Next(390, 401)` (eleven lines
  390-400), no coin spawns.

Both draws ride the bare-`Main.rand` global stand-in (the click runs
outside every swap); the coin-spawn draws stay on UpdatePlayers. The
latch divide (`:40184`) + the player-centre origin (`:40192`) are g58's
`a027138d8` hunk; the portrait hop (`:40224`) is g58's. Witness:
`the_tax_payout_answers_the_dialog_rolls` (twin-stream replay both arms,
one-draw counts, no coins on the refusal).

### 4. The Stylist hair-window STATE machine (`b8daa7855`)

`Main.OpenHairWindow`/`CancelHairWindow`/`BuyHairWindow`/`DrawHairWindow`
(`Main.cs:43653-44138`) + `HairstyleUnlocksHelper.RebuildList`
(`HairstyleUnlocksHelper.cs:47-112`):
- **open** (`:43653-43664`): the HSL bars reset to −1, the inventory
  closes, the chat clears, `oldHairStyle`/`oldHairColor` latch, the window
  opens, sound 10.
- **cancel** (`:43666-43683`): restore the latched hair, close the window,
  `SetTalkNPC(-1)` ONLY when the talked NPC is the Stylist (353), sound 11
  unless quiet.
- **buy** (`:43685-43693`): sound 18, close the window, `SetTalkNPC(-1)`
  unconditional, the bought style STAYS.
- **force-close gate** (`:43711-43715`): chat-text / inventory / chest /
  shop / talk-end / craft each cancel; `close_shop` rides the same seam.
- **price** (`:43786-43799`): the change flags read the latch, the math is
  `shop::hair_restyle_cost` (f37 phase 3, the f32-bits banker's round).
- **Buy label** (`:43800-43852`): the coin split into `Lang.inter[15..18]`
  + `GameUI.BuyWithValue` (`"Buy ({0})"`), the zero price answers the bare
  `GameUI.Buy`; the `< 0 → 0` clamp is display-only.
- **HSL seed/recompose** (`:44001-44092`): the −1 bars seed from
  `rgbToHsl(hairColor)`, the recompose writes `hairColor = hslToRgb(h,s,l)`
  (the `ui::color` fns); the luminosity floors at 0.15.
- **Buy/Cancel click** (`:43974-43989`): the Buy hover charges the purse
  (`BuyItem(num11)`, the aggregate-purse gate) + closes, the Cancel hover
  restores.
- **stylist list** (`HairstyleUnlocksHelper.cs:51-111`): the 213 base
  (`ui::char_creation::HAIRSTYLES`) + the Plantera four (162/164/163/145)
  + the Martian ten (132..=123 descending) + the Moonlord one (133, gated
  on BOTH martians and moonlord).

`Player.hair`/`hair_color` fields land (`Player.cs:2568`/`:2554`), the
hair-layer tint rides `PlayerCompositeState.hair_color` (the default
(215,90,55) reproduces the former HAIR constant bit-for-bit), the
`render_key` gates the tint mix on non-default (the G6 rule: a new pixel
input rides the key, but an unchanged input must not shift the cache).

The panel DRAW stays M4-TOWN-17; the scroll/cell pure fns are g22's
(`main_sounds.rs::hair_window_scroll`/`hair_cell_pick`); `UnlockedMaxHair`
(`:43695-43707`) is DEAD CODE (definition only, zero callers — the window
reads `AvailableHairstyles.Count`). The `.trplayer` sidecar carries no hair
row, so a bought style does not survive a world swap (M6-SAVE). 9
witnesses written not run.

## Co-lane claims verified (NOT duplicated)

The sweep grepped every pushed `origin/task/*` tip for each symbol. The
following are owned by pushed lanes and were left untouched:

| Claim | Owner | Evidence |
| --- | --- | --- |
| Zoologist `Transformed` suffix + `should_bestiary_girl_be_lycantrope` | g58 | `happiness.rs` (the predicate + the category suffix), ANALYSIS "Shop happiness UI" |
| Tax payout latch divide + player-centre | g58 | `a027138d8` (`tax_collector_payout`) |
| Angler `(mood+1)/2` rarity latch | g58 | `468db5928` (`give_angler_reward` reads `shopping_settings.price_adjustment`); `angler.rs:164` already carries the formula |
| Nurse sound-12 + Item4 | g21 | the `nurse_heal` hunk (`play_menu_sound(12)` + the `SOUND_STYLES` Item4 lookup) |
| Reforge latch read | g58 | `13be13a6a` (`reforge_click` reads `shopping_settings.price_adjustment`) |
| Hair-window scroll + cell pick | g22 | `main_sounds.rs::hair_window_scroll`/`hair_cell_pick` |
| `UpdateAltTexture` + transition effects | g61 | `npc_spawn_visuals.rs::make_transition_effects_for_texture_changes` + the "alt-texture halves STAY g61" claim |
| Portrait hop (`DoNPCPortraitHop`) | g58 | `468db5928` (the counter + the EaseOutBounce lift) |

## Dead code

- `Main.UnlockedMaxHair` (`Main.cs:43695-43707`): definition only, ZERO
  callers in the whole decompilation (recursive grep). The hair window
  reads `Main.Hairstyles.AvailableHairstyles.Count` (`:43717`), not this.
  Unported.
- `ShopHelper.GetSkeletonMerchantPrices`/`GetTravelingMerchantPrices`
  (`ShopHelper.cs:64-93`): already recorded dead in f37 phase 4 (the
  travelling-merchant witness records the formula).

## Recorded deviations (unchanged)

- The hair-window panel DRAW (M4-TOWN-17): the state machine lands unwired
  behind witnesses; the HairStyleBack panel / 15-cell grid / HSL slider
  rows / Buy-Cancel labels wait for the UI wave.
- Hair persistence (M6-SAVE): the `.trplayer` sidecar carries no hair row;
  a bought style does not survive a world swap.
- The town pets' `housingCategory` filter: inert until a category-1 kind
  ships (g58 armed the filter).
- The `shopSpecialCurrency` arm: d122 owns the medal-savings half.

## Verification

This lane ran NO cargo/tests (code-only). The witnesses are written:
- `clock.rs::the_anniversary_world_doubles_the_tax_rate_and_the_cap`
- `world_sim/npcs.rs::the_tax_payer_filter_counts_heads_not_town_flags`
- `app/mod.rs::s1_e_18_tests::the_nurse_dialog_band_reads_the_pre_heal_life`
- `app/mod.rs::s1_e_18_tests::the_nurse_refusals_draw_the_dialog_rolls`
- `world_sim/npcs.rs::the_tax_payout_answers_the_dialog_rolls`
- `hair_window.rs::tests` × 9 (open/cancel/buy/force-close/price/buy-label/
  HSL/click/list)

The f32-bits price expectations were python-struct-verified
(`120000×0.95f → 114000 → 11.4 → banker's 11 → 110000`;
`96000×0.95f → 91200 → 9.12 → 9 → 90000`). The dialog rows were verified
present in `decompiled/Terraria.Localization.Content.en-US.Legacy.json`
(52-57, 227-230, 380-383, 390-400) and the Buy/Cancel/HairStyle keys in
`Terraria.Localization.Content.en-US.json` (`GameUI.BuyWithValue` =
`"Buy ({0})"`, `LegacyInterface.15..18` = platinum/gold/silver/copper).

The merger runs `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` after the resolution.
