# 144 — The item liquid-splash keep-one (W4-e19 final, SHIMMER item 1)

**Status: verdict rendered + applied on `task/W4-e19-liquid-quicksell-shimmer`.**
The coordinating research/144 note was never pushed (a full remote scan of
every `origin/task/W4-*` branch found no `docs/research/14[1-9]*` file), so
this receipt IS the 144 verdict, rendered from the C# directly per the
keep-one contract.

## The two halves

Both lanes ported the same C# window — `WorldItem.MoveInWorld`'s entry/exit
splash arms (`WorldItem.cs:879-1035`) + the `wetCount` 20-tick cooldown
(`:883-885`/`:962-964`/`:1042-1045`) + the `!wet` dry clear (`:1036-1041`):

| | e19b (this branch) | f47b (`task/W4-f47-liquid-shimmer-2`) |
| --- | --- | --- |
| commit | `7d3cb5e39` (deferred 4) + `e2003946b` (16-box) | `d7eb4e913` (doc-sweep 36 items 1+2) |
| latch | `ItemDrop.pending_splash: Option<ItemSplash>` + `take_splash` | `ItemDrop.splash_due: Option<Splash>` |
| drain | free fn `world_sim/items.rs::item_splash`, per-drop post-update pre-flicker | method `ItemDrop::splash`, same position |
| box | **SIZE 16** (`WorldItem.cs:138` `base.width = (base.height = 16)`) | SIZE 14 (the pre-census stand-in) |
| anchor | `pos.X-6, pos.Y + 16/2 - 8 = pos.Y` (water exit: `pos.Y+8`) | `pos.X-6, pos.Y + 14/2 - 8 = pos.Y-1` (exit `pos.Y+7`) — **1 px high** |
| dust box | `width+12 = 28` → `next_max(24)` | `26` → `next_max(22)` — **2 px narrow** |
| entry lava gate | sticky `lava_wet` | LOCAL `flag2` (`lava_now`) |
| water style | live biome style, now + the `TR_WATER_STYLE` pin (this receipt's adopt) | `env::water_style().min(14)` else biome |

Everything else is identical by census: the four families (lava 35 ×5 /
shimmer 308 ×10 + the `Next(6)` four-colour switch / honey 152 ×5 / water
`dustWater()` ×10), the per-family post-writes (vy −1.5/−4/−1/−4, vx ×2.5,
scale SET 1.3/0.8/1.3 vs water MULTIPLY ×0.8, alpha 100 except shimmer 0,
noGravity), the exit-snapshot-before-dry-clear semantics, the case-19 cue
styles (shimmer 4 entry / 5 exit, else the 3-arg default 1), the draw order
(dust loop first, pitch after the attenuation gate), and the drain position
(same tick, slot order, `UpdateItems` stream — `Main.cs:18024`'s swap).

**The entry lava-gate difference is behaviourally void**: at an entry edge
`wet` was false at the tick head, so the previous tick's `!wet` tail
(`:1036-1041`) cleared `lava_wet`; if THIS tick's `flag2` is true the
`:866-869` arm re-sets it before the gate. Sticky == local at every entry
edge. Both halves faithful.

## Verdict: KEEP the e19 half, RETIRE the f47 dup-half at merge

The e19 half carries the **C#-correct 16-box** (`WorldItem.cs:138` — the
f47 half's 14 is the retired stand-in; its anchor sits 1 px high and its
dust box rolls 2 px narrow, shifting every splash position draw). The f47
half's only unique content is the `TR_WATER_STYLE` pin seam — **adopted
verbatim here** (`update_items` head: `env::water_style().map(min 14)` else
`calculate_water_style`, threaded into `item_splash` as `water_style: i32`
in place of the camera arg).

**Merge contract (141/master):** when both branches land, keep
`ItemSplash`/`pending_splash`/`take_splash`/`item_splash` (e19) and DROP
`Splash`/`splash_due`/`ItemDrop::splash` (f47) including its four
`w4_f47ds_*splash*` witnesses (superseded by the three 16-box census twins
below + the four existing e19 end-to-end witnesses). The f47 anchors row
`crates/tr-app/src/item.rs|pub fn splash|W4-f47 doc-sweep 36 …` retires with
it; the e19 row `crates/tr-app/src/world_sim/items.rs|fn item_splash|…`
survives. f47's 71/31-draw stream censuses stay valid as COUNTS (71 entry /
31 lava) but its 14-box `next_max(22)` twin rolls are stale — the 16-box
twins here re-pin `next_max(24)`.

## Witnesses (WRITTEN NOT RUN — code-only lane)

`crates/tr-app/src/world_sim/items.rs` (test mod):
1. `w4_e19_the_shimmer_entry_splash_draws_71_at_the_16_box` — the 71-draw
   census (10 × (6 NewDust + 1 colour) + 1 style-4 pitch), `next_max(24)`
   box roll, scale SET 0.8, alpha 0, the four-colour palette, the 16-box
   anchor bands.
2. `w4_e19_the_lava_splash_draws_31_and_wins_the_gate` — 31 draws (5 × 6 +
   style-1 pitch), lava wins over shimmer+honey, vy −2.0 (the type-35
   tweak's −0.5 then −1.5), scale 1.3, alpha 100.
3. `w4_e19_the_water_exit_anchor_drops_the_minus_8` — same-seed entry/exit
   pair: the dust Y delta is exactly 8 (`:1011` drops the −8), entry band
   at `pos.Y + 0` (the 16-box `height/2 − 8` cancellation).

Existing e19 witnesses unchanged (entry burst / exit after cooldown /
cooldown suppression / water-style family).
