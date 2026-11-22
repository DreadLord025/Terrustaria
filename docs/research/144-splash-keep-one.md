# 144 — The item liquid-splash keep-one (W4-e19 final, SHIMMER item 1)

**Status: verdict rendered + applied on `task/W4-e19-liquid-quicksell-shimmer`;
SHIMMER SEAL consumed — the e19 keep UPHELD, the f47 keep-f47 counter-verdict
answered arm-by-arm below, the f47-drop-hunks published.**
The coordinating research/144 note was never pushed at the verdict round (a
full remote scan of every `origin/task/W4-*` branch found no
`docs/research/14[1-9]*` file), so this receipt IS the 144 verdict, rendered
from the C# directly per the keep-one contract. The f47 lane SINCE pushed
its own keep-f47 verdict (`144-item-splash-keep-one.md`, tip `1997be092`,
phases 1-4) — the divergent-verdict state the SHIMMER-seal brief names.

## The SHIMMER-seal arbitration (the f47 counter-verdict CONSUMED)

**ARBITER status**: at this seal round a fresh full-remote scan (360
`origin/task/W4-*` tips, `ls-tree -r docs/` per tip) found **no 144-ARBITER
doc on any branch** — the f47-branch arbiter has not published. Per the
brief's fallback the verdict is re-rendered here from the primary evidence:
`WorldItem.cs:865-1045` re-read line-by-line this round (both windows ≤100
lines), against BOTH lanes' pushed code (e19 tip `c7a38bea`, f47 tip
`1997be092`).

**RULING: the e19 keep is UPHELD.** The f47 144's own arm-by-arm census
agrees on every behaviour ("BOTH faithful" on cooldown, snapshot timing,
exit pre-clear, drain slot, dust bodies; §7 "the f47 lane was WRONG" on the
box). Its keep-f47 preference rests on STRUCTURE (the method-local latch
spend, the once-per-pass water style) plus ONE semantic objection — the
entry lava gate's sticky-vs-local read. That objection is now moot:

1. **The entry gate is verbatim on e19 as of this seal**: `item.rs`
   `move_in_world` parks `let lava_now = player::lava_collision(...)` and
   the ENTRY `ItemSplash` reads `lava: lava_now` (the C# `:886` `!flag2`
   local). The EXIT keeps the sticky `lava_wet` (`:965` reads
   `base.lavaWet` — both lanes already faithful there). Witness
   `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe` pins it.
2. **The water-style seam is already adopted** (the verdict round's adopt:
   `update_items` head `env::water_style().map(min 14)` else
   `calculate_water_style` — the f47 once-per-pass read verbatim, e19
   `world_sim/items.rs:128-130`). The f47 144 §5's "closer" argument is
   satisfied on the e19 half.
3. **The 16-box is no longer a differentiator**: f47's phase-1/2 fold
   landed SIZE 16 + every geometry re-pin on ITS branch too (its
   `item.rs` `pub const SIZE: f32 = 16.0`, the `next_max(24)` twins, the
   wiring-literal fix). Both lanes now run the C# 16 box. The e19 half
   wins the keep on the remaining structure: the free-fn drain is the
   **g67 rain-splash contract** (the shared `DustPool` + `UpdateItems`
   stream seam the co-lane already reads), and the e19 pass-level witness
   quartet + the latch/clear/grabbed witnesses have no f47 pass-level twin
   (the f47 144's own dedupe table concedes this: "KEEP the e19
   gravity/latch/grabbed witnesses").
4. **The f47 unique wins are FOLDED here** (this seal): the two probe
   geometry twins (`w4_f47ds_the_wet_probe_is_the_centre_inset_box`,
   `..._respects_the_liquid_surface_drop` — the touching-edge strictness +
   the quarter-full surface drop + the LavaCollision full-box pair) land
   as `w4_e19_the_wet_probe_is_the_centre_inset_box` /
   `w4_e19_the_wet_probe_respects_the_liquid_surface_drop` in `item.rs`,
   driven through the SAME engines e19's `move_in_world` calls
   (`npc_ai_slime::wet_collision` / `player::lava_collision`). The
   `box_is_wet` surface-offset twin was already e19's (f47's is the fold
   of it — identical body); the 71/31-draw twins + the water-exit −8 twin
   were re-pinned at the 16-box in the verdict round (below).

**f47-drop-hunks (PUBLISHED for 141/master — the loser side at merge):**

| # | f47 hunk (tip `1997be092`) | Fate at merge |
| --- | --- | --- |
| 1 | `item.rs` `pub struct Splash` + `ItemDrop.splash_due` field + the two `Some(Splash{..})` arm sites + `pub fn splash` (the method drain) | DROP — e19's `ItemSplash`/`pending_splash`/`take_splash` + the free-fn `item_splash` survive (the g67 seam) |
| 2 | `item.rs` witnesses `w4_f47ds_the_entry_splash_latches_with_the_sticky_flags` (`:2065`), `w4_f47ds_the_exit_splash_snapshots_the_pre_clear_flags` (`:2093`), `w4_f47ds_the_shimmer_entry_splash_draws_71_and_colours_verbatim` (`:2112`), `w4_f47ds_the_lava_splash_wins_the_gate_and_draws_31` (`:2174`), `w4_f47k1_the_water_exit_anchor_drops_the_minus_8` (`:2227`) | DROP — superseded by e19's `w4_e19_the_wet_flags_latch_and_clear` + `w4_e19_the_grabbed_arm_clears_every_wet_flag` (the latch/clear pass-level pair), the 16-box census trio in `world_sim/items.rs` (`..._draws_71_at_the_16_box`, `..._draws_31_and_wins_the_gate`, `..._the_minus_8`) and this seal's local-probe gate witness. NOTE: f47's exit-snapshot twin's ASSERTION survives behaviourally — the e19 exit arm snapshots pre-clear (the same `Some(ItemSplash{..})` reads before the `!wet` tail) — the e19 exit-after-cooldown witness exercises the drained burst |
| 3 | `world_sim/items.rs` `w4_f47k1_*` pass-level splash quartet (`:1452/:1494/:1519/:1538` — the e19 quartet re-driven through the f47 method) | DROP — the e19 originals (`:1615/:1654/:1678/:1696`) are the same four assertions on the keeper drain |
| 4 | `item.rs` `w4_f47k1_the_dry_flush_landing_carries_the_residual_then_rests` (`:1943`) + `w4_f47k1_the_wet_landing_keeps_the_flush_residual` (`:1961`) | DROP — e19's `w4_e19_the_dry_drop_integrates_at_full_velocity` landing/rest pair (`item.rs:2605`) + `w4_e19_the_wet_landing_keeps_the_flush_residual` (`:2631`) cover both residuals |
| 5 | `item.rs` `w4_f47ds_the_box_is_wet_probe_uses_the_liquid_surface_offset` (`:2043`) | DROP — byte-identical body to e19's `w4_e19_the_wet_probe_uses_the_liquid_surface_offset` (`:2425`; f47's own comment says "the e19 lane's box_is_wet witness folded") |
| 6 | `item.rs` `w4_f47ds_the_wet_probe_is_the_centre_inset_box` (`:1990`) + `..._respects_the_liquid_surface_drop` (`:2013`) | DROP — FOLDED into e19 this seal (the two new `w4_e19_*` probe twins, driven through the engines directly; the f47 versions call its `wet_probe`/`lava_probe` thin wrappers (`:1048`/`:1072`) over the SAME engines — the wrappers may stay or dedupe at merge, the twins are the superseded part) |
| 7 | anchors row `crates/tr-app/src/item.rs\|pub fn splash\|W4-f47 doc-sweep 36 …` | RETIRE with hunk 1 — the e19 row `world_sim/items.rs\|fn item_splash` survives |
| 8 | f47's `wiring.rs` five-field literal fix + `box_is_wet` restoration + the SIZE-16 const/quad/light-anchor re-pins | KEEP whichever side lands first — both lanes carry identical values now; the merge dedupes trivially (NOT a drop: these were the e19 wins f47 folded) |

**f47-keep-hunks (NOT dropped — the merge keeps them):** the 676 ShimmerSlime
light/dust arm (`npc_ai_slime` `wants_shimmer_slime_dust` +
`npc_lights::shimmer_slime_light`, the b1-adapter seam), the NPC-side
shimmer transforms/town state-25 (its phase-1 claims), the
`tile_collision_opts`/`slope_collision_opts` wrappers (the shared collision
seam — e19's hand-rolled TileCollision half reconciles onto them at merge,
the f47 144's "e19-owned hunks NOT folded" list agrees), and its research/121
doc arc. The e19-owned hunks f47 lists as NOT folded (the `shimmer_tail`
wet write, the `rise_over_shimmer` two-row scan, the chain-apply half) stay
e19's at merge — both receipts agree on that boundary.

**Number-key note**: the d54 registry (research/150) predates both pushes
and reserved 144 for c4's orca doc (ONE-SIDED at its scan). Both splash
verdicts now carry 144 at their tips with different filenames
(`144-splash-keep-one.md` e19 / `144-item-splash-keep-one.md` f47) — a
same-key fork for the registry's next sweep; THIS doc is the consuming
receipt, the f47 doc retires as the loser verdict at merge (its 121 FINAL
cross-ref re-points here).

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

`crates/tr-app/src/item.rs` (test mod, the SHIMMER seal):
4. `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe` — the `:886`
   verbatim local read: a lava-pool entry with the sticky flag false at
   the tick head queues `lava: true` (the local probe), the `:866-869` arm
   sets the sticky flag the same tick, `wet_count` 19.
5. `w4_e19_the_wet_probe_is_the_centre_inset_box` — folded f47 twin: the
   10x8 inset at (pos+3, pos+4), the touching X edge dry (strict `>`), the
   FIRST-cell-wins shimmer kind flag.
6. `w4_e19_the_wet_probe_respects_the_liquid_surface_drop` — folded f47
   twin: the quarter-full (liquid 64) 12 px surface drop, the touching
   probe bottom dry, the LavaCollision FULL-box pair at the same surface.
