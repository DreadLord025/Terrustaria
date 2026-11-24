# 148 — W4-e19 research FINAL (the liquid/quicksell/shimmer lane closes)

**Status: SEALED (the SHIMMER-seal round appended below).**
**Branch**: `task/W4-e19-liquid-quicksell-shimmer` (worktree TR-W4-e19).
**Mode**: code-only throughout — every witness WRITTEN NOT RUN; no cargo,
no rustfmt, no graphify (the lane's standing constraint). The gate run
belongs to 141/master after the merge.

## The lane's full arc (22 commits, `e351dd438` → this note)

### Phases 1-4 (the f47-second-half census)
1. `e351dd438` — the GetShimmered chain census fixes (the shared tail
   `WorldItem.cs:1835-1852`, ShimmerEffect `:1661-1689`, the
   customShimmerResults override, the SpawnShimmeredItem fan, the release
   budget loop, the equivalent-type resolve).
2. `f0ab3fa76` — the UpdateShimmer/MoveInWorld liquid-physics census.
3. `5829df441` — the items-pass witnesses + the ramp-tick re-pin.
4. `0e75028b9` — docs + anchor.
5. `f7561091f` — the witness fix round (the tail/decraft spawns decay
   shimmerTime post-reset — the `:538-545` decay arm is NOT gated on
   !shimmered).

### Round 2 (the deferred list, `6d4982240` → `bedb1083a`)
The six deferred seams: the repulsion scan (`:790-847`), the ItemNoGravity
float (`:774-786`), the wetVelocity blend (`:379-396`/`:1046-1084`), the
entry/exit splash dust (`:881-1045`), the honey/water gravity arms + the
per-drop wet flags, the crimson/corruption decraft variant
(ShimmerTransforms `:15-31`). Compile-seam round `3627ce22d`.

### Round 3 (`0570e7136` → `e7e466d4`)
1. The decraft evil-preference CONSUME: g69's persisted `World.crimson`
   adopted verbatim (keep-one g69-canonical), the thread-local hook deleted.
2. **The 16-box**: `item::SIZE` 14 → 16 (`WorldItem.cs:138`
   `base.width = (base.height = 16)`) across every probe; the DRAW quad keeps
   the 14 px visual centred. All witnesses re-pinned.
3. The dry flush-rest zero-velocity deviation retired (the blocked-Y arm
   keeps the TileCollision flush residual in BOTH arms, `Collision.cs:2394`/
   `:2444`).
4. The shimmer ITEM-transform table census: all six sets order-identical
   (316/114/6/1/6/4); the ONE gap — lunar_brick_for_moon_phase read an
   inverted moon convention, re-pinned to the RAW MoonPhase enum indices.
5. The quicksell COIN-math census: base COMPLETE (sell_payout /
   get_item_expected_price verbatim, the 71-74 coin refusal, the memo
   refund); 2 end-to-end coin witnesses.

### The FINAL phase (this round, `cd2fbfa10` → this note)
1. **`cd2fbfa10` — the splash keep-one** (research/144): the coordinating
   144 note was never pushed, so the receipt IS the verdict, rendered from
   the C#: KEEP the e19 16-box half, RETIRE the f47 dup-half at merge
   (its 14-box anchor sits 1 px high, its dust box rolls 2 px narrow; the
   entry lava-gate sticky-vs-local difference proven behaviourally void);
   ADOPT f47's unique TR_WATER_STYLE pin seam. 3 census witnesses re-pinned
   at the 16-box (71-draw entry, 31-draw lava, the water-exit −8 drop).
2. **`c1586b9cb` — the 16-box cross-lane repin list** (research/145): every
   pushed co-lane's 14-geometry witnesses censused for 141/master — the grab
   edge 44/45 → 42/43, the magnet spawns 293/193 → 292/192, the resting
   fixtures floor−14 → floor−SIZE, f49's nebula pc+93/pc−7 → pc+92/pc−8; the
   NO-repin rows proven (the Rat 14×14 PLAYER hitbox, the explicit-size
   helper probes, the pc−4 touch spawns); the f6-mount-grab census (no
   pushed mount-specific grab witness — the grab rect reads the live player
   hitbox, the drop side is the 16-box).
3. **`6627efd6b` — the AETHER biome census** (research/146): the C# Aether
   IS SceneMetrics.ZoneShimmer; the metric + the music gate (91) already
   faithful; PORTED the three unclaimed smalls — the bg-skip
   (`Main.cs:52800-52805`), the natural-star dim/skip (`:62086-62089`/
   `:63107-63110`), the monolith ramp terms (`SceneState.cs:285-289`); the
   liquidAlpha[13] blue boost censused DEAD CODE (no C# writer raises index
   13). 2 witnesses. The artificial star pass + the underground star update
   stay g69d's claim; the ugBackTransition zero notes g72; the shimmerShine
   tint notes g76.
4. **`db8212430` — the quicksell SHIFT-click UI-hint census** (research/147):
   the C# mouse hint IS the cursor-10 icon (no text tooltip; the
   Lang.misc[75] gamepadHintText is gamepad-only); PORTED the cursor-override
   texture draw (the six sheets + the resolver + draw_override + the draw_hud
   thread). 2 witnesses. 2 anchors rows.
5. **This note** — the research FINAL wrap-up.

## The witness ledger (all WRITTEN NOT RUN)

| Round | New witnesses | Re-pins |
| --- | --- | --- |
| phases 1-4 | the chain/physics/items-pass sets | the ramp-tick 90..=93 band |
| round 2 | 18 (4+2+3+4+3+2) | — |
| round 3 | 2 coin end-to-end + 1 persisted-flag | the 16-box geometry set, the moon-phase pair, the flush landing/rest |
| final | 3 splash census + 2 aether + 2 quicksell-hint | the f47 71/31 twins at next_max(24) |

## The merge contracts (for 141/master)

1. **Splash keep-one** (research/144 + its SHIMMER-seal section): keep
   e19's `ItemSplash`/`pending_splash`/`take_splash`/`item_splash`; drop
   f47's `Splash`/`splash_due`/`ItemDrop::splash` — the full 8-row
   f47-drop-hunks table (witnesses + anchors row included) is in 144's
   seal section, with the f47-KEEP rows explicit (the 676 arm, the NPC
   transforms, the collision-opts wrappers).
2. **The 16-box repins** (research/145 + its SHIMMER-seal per-lane
   execution instructions): each lane repins on its OWN branch per the
   seal table; any straggler inherited block repins once at the first
   16-box merge, later merges take the 16-pinned side. The lane-unique
   fixtures (f49's nebula spawns, the editing.rs resting fixtures) need
   the per-lane edits.
3. **World.crimson keep-one** (round 3): g69-canonical — the identical row
   dedupes on merge.
4. **The aether co-lanes** (research/146 + its SHIMMER-seal boundary
   confirmation): g72's `draw_background` shimmer-skip is the same arm
   (either side faithful); its `ug_back_step` keeps the `> 0`
   transition-zero; g69d owns the artificial star pass + the underground
   star update (DISJOINT from e19's natural-star dim — confirmed at the
   C# windows this seal); g76 owns the shimmerShine tint.
5. **The cursor-override draw** (research/147 + its SHIMMER-seal compose):
   the six sheets + the resolver + draw_override are e19's surface;
   g86e's cursorItemIcon (170 sites) is a DIFFERENT layer (40 vs 36) —
   keep BOTH, the compose contract in 147's seal section.

## The standing deviations (all in ANALYSIS.md)

- The aggregate coin purse folds the C#'s four-denomination coin-stack spawn
  (the copper total is the same).
- The gamepad input layer is unported (the BuildCommand hint chain, the
  ApplyShader accessory seam, the TOW music swap).
- The `Main.liquidAlpha[13]` blue boost is dead code in the C# — the 0 feed
  is faithful.
- The artificial star pass + the star fall/respawn machine are g69d's claim.
- The aether black-backdrop quad needs a screen-fx seam (the g72/g69d merge
  lane).

## Verification

Code-only lane: NO cargo/rustfmt/graphify run (the constraint). Every
witness is hand-derived against its cited C# window (≤100 lines per read).
The gate run (`cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids`) belongs to 141/master after the merge
contracts above are applied.

## The SHIMMER SEAL (this round — the lane's final receipts)

The seal brief's five items, executed on this branch after `c7a38bea`:

1. **SPLASH-ARBITER consumed** (research/144's new seal section): the
   f47-branch arbiter has NOT published (a fresh 360-tip full-remote scan
   found no 144-ARBITER doc anywhere); the f47 lane DID push its own
   keep-f47 counter-verdict (`144-item-splash-keep-one.md`, tip
   `1997be092`). Re-rendered from `WorldItem.cs:865-1045` (both windows
   re-read ≤100 lines): **the e19 keep is UPHELD** — f47's own census
   concedes every behaviour arm and the 16-box; its one semantic objection
   (the sticky entry lava gate) is fixed VERBATIM on e19 this seal
   (`lava_now` local read, `:886`), its water-style seam was already
   adopted. The **f47-drop-hunks table is PUBLISHED** in 144 (8 rows: the
   Splash latch/method, the 5 superseded f47ds/f47k1 item witnesses, the
   pass-level quartet, the flush pair, the box_is_wet twin, the 2 folded
   probe twins, the anchors row; the KEEP rows — the 676 arm, the NPC
   transforms, the collision-opts wrappers — explicit). Two f47-unique
   probe twins FOLDED as e19 witnesses + 1 new local-gate witness
   (3 new, WRITTEN NOT RUN).
2. **16-box repin EXECUTION instructions published** (research/145's new
   seal section): per-lane, own-branch, byte-accurate line numbers
   re-verified at every lane's CURRENT tip (g66 `248f0ff4d`, e1
   `3245d624c`, f6 `0f9fd642f`, f49 `0d1f941af` +92/−8 nebula spawns,
   g15 `7981aa5bf` (its census-round line numbers corrected), d79
   `03ebefad7`, g56 `54c63d8d4`, f40 `c4c6b4f4f`, g25/g79/d98/g48
   (new tip `2b61530b4` re-verified)/g52; g20/f39 NO-repin; f47 already
   done on its branch). The shared inherited-block edit spelled out
   step-by-step + the mechanical grep-verify recipe.
3. **AETHER g69d-star-visual boundary CONFIRMED** (research/146's new seal
   section): all four C# windows re-read (`:62086-62089` natural skip,
   `:63107-63110` natural dim, `:62199-62206` artificial pass,
   `:17256-17274` underground update). The artificial pass + the
   underground star update STAY g69d's claim; e19's natural-star dim/skip
   is the `artificial: false` half — DISJOINT, compose without conflict.
   g69e-clouds-retire-active reads: its retire touches only the cloud
   arms, not the star field.
4. **quicksell-cursor g86e compose published** (research/147's new seal
   section): the g86 tip (`e6d02d2f0`) read; `Main.cursorOverride` (layer
   36, replaces the pointer) vs `Player.cursorItemIcon*` (layer 40, the
   +10+push item-sprite overlay, 170 write sites) are DIFFERENT systems —
   **keep-one = keep BOTH**, the 4-row compose contract spelled out (the
   two reset clocks stay separate: interface-frame `:44941` vs update-tick
   `MouseOversClear` `:17035`/`:46503-46513`).
5. **This seal + the shimmer-family bible** (ANALYSIS "The shimmer family
   — the merger bible"): the family-wide owner map (e19/f47/g69d/g69e/
   g72/g76/g86e/b1/g67), the merge order, the standing deviations.

### The seal witness ledger (all WRITTEN NOT RUN)

| File | New witnesses |
| --- | --- |
| `item.rs` | `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe`, `w4_e19_the_wet_probe_is_the_centre_inset_box` (f47 fold), `w4_e19_the_wet_probe_respects_the_liquid_surface_drop` (f47 fold) |

Code change: `move_in_world`'s entry gate reads the LOCAL lava probe
(`lava_now`), the `lava_wet` field doc updated. No behaviour change at any
reachable state (the 144 equivalence proof) — the read is now the verbatim
C# shape.
