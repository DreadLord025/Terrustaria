# 147 — The quicksell SHIFT-click UI-hint census (W4-e19 final, SHIMMER item 4)

The quicksell coin math is COMPLETE (round 3, `e7e466d4` — the buy→shift-sell
round trip nets zero via the memo refund, the un-memo'd stack pays the fifth).
This census covers the UI HINT — what tells the player a shift-click will sell.

## The C# census

**The mouse path has NO text tooltip for the sell action.** The hint IS the
cursor override:

- `ItemSlot.GetAlternateClickAction` (`ItemSlot.cs:629-728`): Shift held +
  a hovered inventory slot (ctx 0/1/2) with no chest/reforge/guide/crafting
  arm matching → `GetSellOrTrash(item)` (`:687-690`).
- `GetSellOrTrash` (`:198-209`): `npcShop <= 0` → Trash (cursor 6); the coin
  kinds 71-74 → null (no hint, no action); everything else → Sell (cursor 10).
  The head guard (`:631-634`): air or favorited slots answer null.
- `AlternateClickAction.Sell = new(10, Lang.misc[75])` (`:218`) — the id 10
  is the cursor sheet; `Lang.misc[75]` ("Sell") is the **gamepadHintText**,
  consumed ONLY by the gamepad `BuildCommand` chain (`:4446`, the
  `SmartSelect` button hint under `PlayerInput.UsingGamepad`). The mouse
  path never reads it.
- The draw (`Main.cs:45269-45310`): `cursorOverride != -1` replaces the
  normal cursor; ids 3/6/7/8/9/10 draw NO shadow (`flag = false`), plain
  white, at `cursorScale` (`:45293-45301`). Id 2 (the smart-interact grab)
  keeps the 0.7 scale + the (0.1, 0.1) origin — not an AlternateClickAction.
- `Main.cursorOverride = -1` resets per interface frame (`Main.cs:44941`).

## The Rust state + the port

The hover evaluation was ALREADY ported (`clicks.rs::evaluate_cursor_override`,
the tail arm `:2347-2358` — the shop-open sell / no-shop trash / coin refusal
/ favourite refusal all faithful, pinned by the round-3 witnesses). The
**texture draw was the recorded follow-up** ("the Cursors_8/9 texture draw is
a recorded follow-up", the old `app/mod.rs:367` note) — PORTED here (the
unclaimed small):

1. `assets.rs`: the six override sheets (`UI/Cursor_{3,6,7,8,9,10}`) join the
   atlas request list + `Lookups::cursor_overrides` + the
   `Lookups::cursor_override(id)` resolver (the 4/5 gap collapsed; non-
   override ids answer None).
2. `ui/cursor.rs::draw_override`: the white/no-shadow/scale-1 draw
   (`Main.cs:45293-45309`); returns false when the id/sheet is absent so the
   caller falls back to the normal cursor.
3. `ui/mod.rs::draw_hud` takes `cursor_override: i32` (threaded from
   `self.cursor_override` at the `frame/draw.rs` call site); the tail cursor
   pass tries the override first.
4. The stale "recorded follow-up" notes retired (`app/mod.rs`, `chest.rs`,
   the DW-4.5 witness comment).

**The gamepad hint string stays unported** — the `BuildCommand` chain is
gamepad-only (`PlayerInput.UsingGamepad`), and the port has no gamepad input
layer (the standing deviation). No text hint is added to the item tooltip:
the C# `MouseHover` (`ItemSlot.cs:3164-3256`) never appends a sell line, and
inventing one would break the 1:1 rule.

## Witnesses (WRITTEN NOT RUN — code-only)

1. `ui/cursor.rs::w4_e19_the_cursor_override_ids_resolve_their_sheets` —
   3/6/7/8/9/10 resolve, -1/0/1/2/4/5/11 answer None, an absent sheet
   answers None (the fallback arm).
2. `clicks.rs::w4_e19_the_quicksell_hint_is_the_cursor_gate_ladder` — the
   full hint ladder: the sell cursor lights on shift-hover with a shop; the
   coin kind 71 refuses; the favourite refuses; no shop falls back to trash
   (6); the shift release clears to -1.

The existing round-3 coin witnesses (`w4_e19_the_bought_unit_quicksells_at_
the_buy_price`, `w4_e19_the_unmemoed_quicksell_pays_the_fifth`) already pin
the cursor-10 state at the sell click — unchanged.

## SHIMMER SEAL — the g86e-cursor-170-sites keep-one COMPOSE (g86e-active reads)

The seal round read the g86 tip (`e6d02d2f0`, its ANALYSIS altar block:
"the whole `cursorItemIcon*` system (170 write sites in Player.cs, the
`DrawCursor` icon draw) is UNPORTED — a separate large system") and
re-read the two C# draw windows (each ≤100 lines). **The two systems are
DIFFERENT layers, not competing ports — the compose KEEPS BOTH:**

| | e19 cursor-override (this lane, PORTED) | g86e cursorItemIcon (170 sites, its claim) |
| --- | --- | --- |
| C# state | `Main.cursorOverride` (a static int, −1 = none) | `Player.cursorItemIconEnabled/ID/Text/Push/Reversed` (per-player fields) |
| Set by | `ItemSlot.GetAlternateClickAction` hover evaluation (shift/alt + slot context) | `TileInteractionsMouseOver` + the 170 Player.cs write sites (the tile/NPC hover icon: altar 6135/6136, golem altar 1293, …) |
| Reset cadence | `Main.cursorOverride = -1` per interface frame (`Main.cs:44941`) | `MouseOversClear()` per UPDATE tick (`Main.cs:17035` → `:46503-46513` zeroes Enabled/ID/Text/Push) — a DIFFERENT clock, do not merge the resets |
| Draw site | `DrawInterface_36_Cursor` (`Main.cs:45254`, the override arm `:45269-45310`) — REPLACES the pointer texture | `DrawInterface_40_InteractItemIcon` (`Main.cs:45037-45115`) — an ITEM SPRITE overlay at `(mouseX + 10 + push, mouseY + 10 + push) + halfDrawHitbox*scale`, `ItemSlot.DrawItemIcon` ctx 21, white when the id is lit (`:45074-45078`), the ammo sub-icon + the `cursorItemIconText` MouseText ride along |
| Layer order | 36 (Cursor) | 40 (Interact Item Icon) — AFTER 36/37/38/39 (`Main.cs:44889-44913`'s layer list) |
| Exclusivity | replaces the normal cursor while lit | gated `cursorItemIconEnabled && num != 0 && MouseDisplayItem.IsAir` (`:45061`) — refuses while the cursor carries an item; NOT gated on cursorOverride |

**Compose contract for the merge (g86e executes on its own branch):**

1. e19's surface survives verbatim: the six `UI/Cursor_{3,6,7,8,9,10}`
   sheets + `Lookups::cursor_overrides` + `cursor_override(id)`
   (`assets.rs:2038`/`:2293`), `ui::cursor::draw_override`
   (`ui/cursor.rs:120`), the `draw_hud` tail's override-first pair
   (`ui/mod.rs:1211-1213`), `App.cursor_override` + the clicks.rs
   evaluation. g86e does NOT re-port any of it.
2. g86e adds the cursorItemIcon fields + the 170-site write census + its
   OWN draw pass, placed AFTER the `draw_hud` cursor tail (the layer-40
   position — the icon overlays whatever cursor is on screen, override or
   normal). Its reset rides the sim/update tick (`MouseOversClear`), NOT
   the interface-frame `cursor_override = -1` reset.
3. Both can be lit the same frame in the C# (the override replaces the
   pointer; the icon floats at +10+push beside it) — no keep-one DROP
   either way. The only shared asset surface is the item atlas the icon
   draw loads (`Main.instance.LoadItem(num)`, `:45090`) — the port's
   existing item-sheet lookup, no new sheets from e19's six.
4. The e19 id-2 note stands for g86e: id 2 (smart-interact grab) is a
   cursorOverride case (`:45278-45291`, the 0.7 scale + (0.1,0.1) origin),
   NOT an AlternateClickAction — it belongs to the smart-cursor lane, not
   to either cursor system here.
