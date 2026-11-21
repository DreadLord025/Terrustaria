# 142 — The savings draw, two-way keep-one (W4-d122 × W4-g58)

Two branches ported `ItemSlot.DrawSavings` independently:

* **W4-d122** (`10420b96`, this branch): `crates/tr-app/src/ui/savings.rs`
  (428 lines) — BOTH `DrawMoney` arms (horizontal + vertical), the
  `CoinSlot.Savings[4]` machine in `ui/coin_slot.rs`, the `Coin_0..3` spin
  strips as `Lookups::coin_anim` + `coin_anim_uv` (with the C# `sizeLimit`
  clamp and a static-icon fallback), wired at the REFORGE call site
  (`ui/mod.rs`, `(180, 258)`, horizontal).
* **W4-g58** (`1328b118`, docs tail `0d683ec4`): `ui/savings.rs` (319 lines)
  — the VERTICAL arm only, the same `Savings[4]` machine, the spin strips
  as `Lookups::coin_strips` + `coin_strip_frame` (no clamp, no fallback),
  wired at the SHOP-PANEL call site through the new
  `ui/shop_happiness.rs` chrome (Shop label + NPCHappiness icon + P0
  percent).

## The C# caller census (ALL `ItemSlot.DrawSavings` call sites)

`Select-String DrawSavings` over `decompiled/Terraria/Main.cs` — exactly
three callers, all in `Main`:

| # | Site | Coordinates | Arm | Gate |
| --- | --- | --- | --- | --- |
| 1 | `Main.cs:41637` (reforge menu, `DrawInventory`) | `num51 + 130, invBottom` = (180, 258) | `horizontal: true` | inside `Main.reforgeItem.type > 0` (`:41584`), menu open (`InReforgeMenu`, `:41557`) |
| 2 | `Main.cs:41791` (shop panel, `DrawInventory`) | `504, invBottom` = (504, 258) | vertical (default) | `npcShop > 0 && !PipsUseGrid` (`:41788`); between the Shop label (`:41790`) and the happiness icon (`:41792`) |
| 3 | `Main.cs:43862` (`DrawHairWindow`) | `num18, num19 - 70` = `(screenWidth/2 − HairStyleBack.Width()/2 + 22 + 18, screenHeight/2 + 60 + 156 − 70)` | `horizontal: true` | `num11 > 0` (the hairstyle-change price, `:43860`) |

**Both branches' call sites are REAL — the union is one module + all
three call sites.** Caller 3's window (the Stylist's in-game hair window,
`Main.hairWindow` → `DrawInterface_21_HairWindow`, `Main.cs:45726-45732`)
is UNPORTED (the port's hair picker is char-creation only,
`ui/char_creation.rs`), so caller 3 stays a recorded no-consumer until
the hair window lands; the module's horizontal arm already covers its
shape (`horizontal: true`, price-gated).

## Arm-by-arm comparison vs the C#

Reference: `ItemSlot.DrawSavings` (`ItemSlot.cs:3704-3741`),
`ItemSlot.DrawMoney` (`:3652-3702`), `CoinSlot.DrawItemCoin`
(`CoinSlot.cs:240-254`), `Utils.CoinsCount/CoinsSplit/CoinsCombineStacks`
(`Utils.cs:1019-1076`), `Utils.CenteredRectangle` (`:947-950`).

| Arm | C# | d122 | g58b | Verdict |
| --- | --- | --- | --- | --- |
| CoinsCount | `:1019-1046` ×1/×100/×10000/×1000000, no ignore slots at these calls | same (`coins_count`) | same | tie |
| CoinsCombineStacks | `:1062-1076` running sum, `>= 9999999999` returns the cap | same | same | tie |
| CoinsSplit | `:1048-1060` platinum-first walk, `/100` per step | same | same | tie |
| `> 0` gate | `:3717` | same | same | tie |
| Pile centres | vault 4076 (+70,+45), forge 3813 (+92,+45), safe 346 (+80,+50), piggy 87 (+70,+60); one per funded bank, draw order bank4→bank | same (`pile_icons`) | same (inline) | tie |
| Pile rect | `CenteredRectangle(center, frame.Size()*0.65)` — `(int)` truncations on position AND size (`Utils.cs:949`); safe/piggy use the TEXTURE size (`:3733/:3737`), vault/forge the frame size — identical for all four (no `itemAnimations` row) | truncates both, texture size | same (`centered_rect`) | tie |
| Savings label | `:3654` `Lang.inter[66]` at `(shopx, shopy+40)`, FourWay ±2 border, `White * mouseTextColor/255` | same | same | tie |
| Coin cell positions | horizontal `:3680` `(shopx + MeasureString(text).X + 24i + 45, shopy+50)`; vertical `:3699` `(shopx + 11 + 24j, shopy+75)`; jump offset subtracts from the COIN only | same (`money_coin_pos`) | vertical same | d122 (both arms) |
| Display clamp | `:3672-3675`/`:3691-3694` count clamps at 999; the CLAMPED count feeds `UpdateSavings` | same | same | tie |
| Platinum text nudge | `:3689` `num2 = -6` iff `j == 0 && coinsArray[3] > 99` (RAW count, vertical only) | same (raw at i==0) | same | tie |
| **Count-text origin** | `:3682`/`:3700` position `y + LineSpacing*0.75`, origin `(0, LineSpacing)`, scale `0.75*stackTextScale`; `DrawString` scales the origin (`DynamicSpriteFont.InternalDrawFast:194` `position -= origin * scale`) → net top-left `y + ls*0.75 − ls*0.75*sts`. At rest (sts=1) that is EXACTLY `y`; during the pulse the text lifts | **WRONG**: froze scale at 1 → `y − 0.25*ls` (off by `0.25*ls` ≈ 4-5 px at rest, and no pulse lift) | **faithful**: `ty = shopy + 75 + ls*0.75 − ls*scale` | **g58b — ported into d122's module (this round)** |
| Coin strip draw | `DrawItemCoin:242-252` `Coin[type-71].Frame(1,8,0,frame)`, longer-side `sizeLimit` clamp (`:247-250`), centred origin, `GetAlpha` tint (coins 71-74 → plain white) | full clamp + `Item_71..74` static fallback when the strip file is absent (documented stand-in; the retail `Coin_0..3.xnb` exist) | no clamp ("1024 never binds"), no fallback | d122 (superset — identical pixels at the savings calls where the 1024 limit never binds; the clamp is needed for the ctx-1 grid coins' 32 px limit) |
| Savings machine | `CoinSlot.UpdateSavings:225-228` `Savings[slot].UpdateState(71+slot, count, 10)`; `UpdateSlotAnims:256-274` steps Savings FIRST | same | same | tie (semantically identical fields; d122 keeps its names) |

## Keep-one decision

**KEEP the d122 module** (`ui/savings.rs`, both arms, the clamp +
fallback, `coin_anim`/`coin_anim_uv` names) and:

1. **Fix its count-text origin** to g58b's faithful formula (the one arm
   g58b got right and d122 got wrong) — `money_text_pos` takes the draw
   scale; net `y + ls*0.75 − ls*scale` in BOTH arms. DONE this round.
2. **Port g58b's shop chrome** (`ui/shop_happiness.rs`: Shop label
   `:41790`, NPCHappiness 4-frame icon `:41792-41796` with the
   ≤0.82/≤1/≤1.1/else bands, P0 percent `:41797`, offsets (26,98)/(12,108)
   `Main.cs:67266-67269`) adapted to d122's `draw_savings` signature, and
   wire the `(504, invBottom)` vertical call between the label and the
   icon. DONE this round. g58b's `happiness_button` /
   `npc_chat::draw_panel` gate (its `ReportHappiness.Condition` slice)
   is NOT ported here — that belongs to g58b's own report lane; the
   eventual main merge keeps g58b's panel gate and this branch's chrome
   (cross-note for the merger: the two `shop_happiness::draw` bodies
   differ only in the savings-call signature).
3. **Port the medal arm** (below) — d122's recorded deviation, g58b
   declared it "inert" (wrong for the Tavernkeep: the port HAS the medal
   wallet since `565dd6b0`, so the latch has a real producer).
4. g58b's `coin_strips`/`coin_strip_frame` rename is NOT adopted (pure
   rename; d122's `coin_anim_uv` is the superset). Merge note: the two
   `assets.rs` coin-strip loaders are the same four files — keep one.

## The `_customCurrencyForSavings` medal arm (census + port)

* Latch: `ItemSlot._customCurrencyForSavings` (`ItemSlot.cs:308`), init
  −1 (`:363`); `UpdateInterface` (`:516-521`, called every update at
  `Main.cs:17046`) resets to −1 when `!playerInventory || talkNPC == -1`;
  `MouseHover` (`:3175-3177`) stamps `item.shopSpecialCurrency` on EVERY
  non-air hovered item (any context — a plain item's −1 resets it). The
  shop grid's per-cell `ItemSlot.Handle` (`Main.cs:41814`) runs
  `MouseHover` AFTER that frame's `DrawSavings` (`:41791`), so a shop-row
  hover latches the medal display for the NEXT frame; the grid/coin
  hovers (`:41022-41039`/`:41762`) run BEFORE the reforge draw.
* Branch: `DrawSavings:3707-3711` — `!= -1` →
  `CustomCurrencyManager.DrawSavings(sb, currency, shopx, shopy,
  horizontal)` and RETURN (the coin arm never runs).
* `CustomCurrencyManager.DrawSavings` (`CustomCurrencyManager.cs:28-62`):
  per-bank `CountCurrency` (medal 3817, cap 999 mid-walk,
  `CustomCurrencySystem.cs:26-47`) + `CombineStacks` (cap 999,
  `:49-63`); `> 0` gate; pile centres DIFFER from the coin arm: vault
  (+96,+50), forge (+80,+50), safe (+80,+50) — the safe and forge OVERLAP
  (C# as-is, `:49/:53`), piggy (+70,+60); the Savings label (`:59`) is
  the same; then `DrawSavingsMoney`.
* `CustomCurrencySingleCoin.DrawSavingsMoney`
  (`CustomCurrencySingleCoin.cs:76-101`): the STATIC `Item_3817` texture
  (not a coin strip), `CoinSlot.UpdateCustom(3817, (int)total)`
  (`CoinSlot.cs:230-238` — the `Custom` dictionary keyed by the ITEM id,
  `UpdateState(3817, count, SavingsCoinJumpScale=10)`; the 3817 pulse arm
  is `CoinEntry.UpdateState:75`); icon at scale `CurrencyDrawScale = 0.8`
  (`:14`), centred origin, NO size clamp, NO 999 display clamp (the
  wallet caps at 999 anyway); horizontal centre
  `(shopx + labelWidth + 45, shopy + 50 − jump)` with the text at
  `(pos.X − 11, pos.Y)` — the TEXT RIDES THE JUMP in this arm — origin
  `(0.3, 0.3)` scaled by `0.75*sts`; vertical centre
  `(shopx + 11, shopy + 75 − jump)`, text `(shopx + num2, shopy + 75)`
  (no jump on the text), `num2 = −6` iff `total > 99`.
* `UpdateSlotAnims:270-273` steps the `Custom` dictionary LAST (after
  Savings/Chest/Inventory).

Ported this round: `CoinSlotState::update_custom` + the Custom map,
`custom_currency::count_savings`, the medal arm in `ui/savings.rs`
(`draw_savings` grows the `currency` latch parameter and dispatches),
`App.custom_currency_for_savings` with the `UpdateInterface` reset (in
`update_shop_close`'s per-frame cluster) and the `MouseHover` stamps in
`draw_hud` (grid hover → −1 before the reforge draw; shop-row hover →
the row's currency after the shop pile, matching the C# order).

## The coin-purse display census (the ctx-1 coin column)

The purse DISPLAY is the four coin slots at x 497, context 1, scale 0.6
(`Main.cs:41752-41764`) — plain `ItemSlot.Draw`, NOT `DrawSavings`. Its
coin path (`ItemSlot.cs:2349/:2515/:2697-2706`):

* `CoinSlot.UpdateDrawState(slot, context, item)` (`CoinSlot.cs:292-312`)
  feeds EVERY slot draw (the port's machine has this — S1-H-9 — but
  `ui/slot.rs::draw_slot` never calls it).
* `item.IsACoin || fadeItem > 0` → `CoinSlot.DrawItemCoin` with the SPIN
  strip, `sizeLimit = 32`, the jump offset × `inventoryScale`, tint
  `GetAlpha(lightColor)`, fade `itemFade * fadeScale`; the medal 3817
  draws its STATIC icon but still rides the jump offset (`:2515`'s third
  arm).
* The stack text (`:2697-2706`) draws when `stack > 1 ||
  stackTextDrawFadeOverload > 0`, scale `inventoryScale * stackTextScale`,
  tint × the fade overload — the pulse/fade the port's `draw_slot`
  comment (`slot.rs:437-439`) already records as unimplemented.

The dedicated follow-up LANDED in the final phase (W4-d122, commit
`34686466`): `ui/slot.rs::draw_slot` now consumes `UpdateDrawState` on
every mapped-context slot (grid 0 / coins 1 / ammo 2 / chest 3 / bank 4),
threading `Option<&mut CoinSlotState>` + the vanilla slot index through
every caller; the coin spin strip / medal jump / fading-coin flag4 /
stack-text pulse+fade / dye-slot dim all land (ANALYSIS "The S1-H-9
UpdateDrawState follow-up"). The neutral contexts (hotbar/mouse/shop/
equip/trash/crafting) pass `None`. The fifth-round `coin_total` fix was the
Stylist GATE arithmetic (`Chest.cs:2714-2746`), unrelated to this draw.

## The hair window (caller 3) — the price core + the task-seed

The third `DrawSavings` caller (`Main.cs:43862`, `DrawHairWindow`) is the
Stylist's in-game hair-change UI (`Main.hairWindow` →
`DrawInterface_21_HairWindow`, `Main.cs:45726-45732`; the trigger
`NPCInteractions.cs:425-441`'s `StylistHairWindow` → `Main.OpenHairWindow`).
g21c's sound census records hosts 80-90 (`Main.cs:43663-43936`,
`OpenHairWindow`/`CancelHairWindow`/`BuyHairWindow`/`DrawHairWindow`) as
unported — the hair window is deferred to the hair system.

**The window is NOT self-contained.** `DrawHairWindow`
(`Main.cs:43709-44138`, ~430 lines) needs:

* the **hair-style grid** (`:44098-44137`): 15 cells (5×3) each drawing the
  head (`Players[0,0]` skinColor), eye-whites (`[0,1]`), eyes (`[0,2]`
  eyeColor) and the hair (`PlayerHair[num33]` selColor) — the port decodes
  ONLY `Player_Hair_1` (`assets.rs:348`, the recorded char-creation
  deviation, `char_creation.rs:1701`), so the 213-style grid cannot draw;
* `Main.LoadHair` (`Main.cs:59146-59153`, the per-style `PlayerHair` +
  `PlayerHairAlt` request) — unported;
* `Player.GetHairDrawOffset` (`Player.cs:4549-4561`, the 163/164 nudges) —
  unported;
* the in-game `HairstyleUnlocksHelper.UpdateUnlocks` list
  (`HairstyleUnlocksHelper.cs:19-113` — the char-creation 213 + the
  Plantera 162/164/163/145 + the Martian 123-132 + the Moonlord 133) — the
  port has only the char-creation `HAIRSTYLES` constant
  (`char_creation.rs:264`);
* the HSL colour sliders (`:43995-44092`) — the port HAS the shared
  `sliders::draw_color_sliders`/`HslBars` (`menu/mod.rs:999`), reusable;
* the window state (`hairWindow`/`oldHairStyle`/`oldHairColor`/`hairStart`/
  `grabColorSlider`/`npcChatFocus1/2`) + the Stylist chat trigger + the
  `DrawInterface_21` layer wiring — unported.

**The self-contained price + savings-apply core IS ported** (the final
phase, `ui/hair_window.rs`):

* `hair_price` (`Main.cs:43786-43799`): style 100000 + colour 20000, the
  Discount Card ×0.8 `(int)` truncation, then the happiness latch
  `(int)Math.Round((float)(int)(num11 * PriceAdjustment) / 10000f) * 10000`
  (the inner `(int)` truncates, `/10000f` float-divides, `Math.Round`
  bankers via `shop::round_half_to_even`, `*10000` restores copper). The
  price reads the TALK-OPEN latch (`currentShoppingSettings.
  PriceAdjustment`, g58's census — like the reforge/nurse, NOT a fresh
  mood);
* `hair_price_text` (`:43800-43852`): the `CoinsSplit` platinum-first walk,
  each non-zero denomination `"<count> <Lang.inter[15..18]> "` (platinum/
  gold/silver/copper, lowercase), `GameUI.BuyWithValue` ("Buy ({0})") /
  `GameUI.Buy` at zero;
* `hair_savings_pos` (`:43857-43862`): `(screenW/2 − backW/2 + 18,
  screenH/2 + 146)`, the horizontal `draw_savings` call gated on
  `price > 0` — the module's horizontal arm already covers its shape.

**Task-seed (the hair-system lane):** port `DrawHairWindow` once the
213 `PlayerHair` textures decode (the char-creation grid's same blocker),
`LoadHair`/`GetHairDrawOffset`/the in-game `UpdateUnlocks` land, and the
Stylist chat trigger + the `DrawInterface_21` layer wire. The price core
(`ui/hair_window.rs`) + the savings horizontal arm + the shared HSL sliders
are ready; the buy rides `Player.BuyItem` (`Player.cs:35400`, the coin
purse + banks spend, the shop-transaction lane's `buy_shop_item` cousin).
The cancel restores `oldHairStyle`/`oldHairColor` (`CancelHairWindow`,
`:43666-43683`); the buy plays sound 18 + `SetTalkNPC(-1)` (`BuyHairWindow`,
`:43685-43693`).

## Witnesses

Written not run (code-only): the fixed text-origin formula (rest +
pulse, both arms), the medal pile centres + overlap, the medal text
positions (jump-riding horizontal, nudged vertical), the latch
dispatch, `update_custom`'s 3817 pulse + independent rank + step order,
`count_savings`' per-bank/combined 999 caps, the shop chrome's bands /
P0 / offsets (g58b's three, kept), the NPCHappiness strip packing.

The final phase adds: the S1-H-9 consumption witnesses (the coin-row
pulse/jump/fade, the dye dim, the neutral tail, the stack-text
rest/pulse/ctx43 — `coin_slot.rs`/`slot.rs`), and the hair-window price
core (the price ladder + banker's rounding, the coin-text denomination
walk, the savings position — `ui/hair_window.rs`). The shop-FULL census
is doc-only (the per-case witnesses already pin the d122 rows).
