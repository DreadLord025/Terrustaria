# 12g — W4-f25: worldgen passes, second lane (orphan adoption: Settle Liquids)

> Wave 4 · lane `task/W4-f25-worldgen-passes-8` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).
> Code-only mode (no cargo/tests/graphify on this branch); C# read in ≤100-line
> windows; ONE pass per commit+push; hash re-pins deferred to the merge gate
> (the S1-K-2-wave-B / d10x convention), `a == b` determinism halves untouched.

## In-flight probe (all pushed branches + census docs read at claim time)

`git ls-remote --heads origin` + every pushed `docs/research/12*` doc:

- **d101** `12b-d101-passes-1.md` @ 4194acf8 — 9/9 landed: Sand Patches,
  Mount Caves, Dirt Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves,
  Rock Layer Caves, Generate Ice Biome, Granite (own row).
- **d102** `12a-d102-passes-2.md` @ 28ec5561 — ledger Silt…Wood Tree Walls
  (Silt/Webs/Slush/Mountain Caves/Gems landed; Shimmer, Clean Up Dirt, Dirt
  Rock Wall Runner, Wood Tree Walls pending). Its lane-split table assigns
  **Settle Liquids to d103**; its own nine-pass ledger does NOT contain it.
- **d103** lane-split section in `12-worldgen.md` @ 561b3ab2 — own range
  Oasis…Floating Island Houses (Oasis/Waterfalls landed). Its table assigns
  **Settle Liquids to d102**; its own ledger does NOT contain it.
- **d104** `12d-worldgen-passes-4.md` @ 2d9cfcbb — claim only: Floating
  Island Houses…Spawn Point.
- **d105** `12e-worldgen-passes-5.md` @ 5bb4116e — claim only: Guide…Larva.
- **d106** `12f-worldgen-passes-6.md` @ ee0fcbef — Grass Wall (orphan adopted,
  landed) + Settle Liquids Again, Tile Cleanup, Stalac, Remove Broken Traps,
  Final Cleanup (pending).
- **g37** `12h-g37-worldgen-missing-b.md` @ 8ef3e111 — claim only: the
  remainder-quarter B1-B9 (Statues…Surface Ore and Stone), all duplicates of
  d103/d104 stated ranges (merger reconciles). g36/g38/g39 and f24 have
  pushed NO branches/docs at probe time — the a/c/d quarters are unclaimed
  but every pass in them is already in a d10x lane's own ledger.
- **d107** @ 26672912 — special-seed branches (drunk ore re-rolls, remix
  bands, SurfaceCaves gates, dualDungeons name fix); no classic pass rows.
- **d108** @ 7ac633e9 — temple seed branches + paint; no census rows.
- c33 (Water Chests/Oasis, referenced by d104/g37) — no pushed branch.

## Census re-derived on this branch's base (a725acb0)

Scripted diff of `pipeline.rs`: 106 classic (`gate == ""`) `VANILLA_CHAIN`
declarations (109 total − Skyblock ×2 − Dual Dungeons Dither Snake), 56
distinct executed `vanilla` names → **51 classic unexecuted passes**, the
same 51-name list as d101's `12b` census (Granite included until d101's
row-split merges; Generate Ice Biome included likewise).

Union of every lane's OWN ledger (not the cross-assigned lane-split tables):
d101 #1-9 ∪ d102 #10-18 ∪ d103 #20-28 ∪ d104 #28-36 ∪ d105 #38-46 ∪
d106 {Grass Wall, #47-51} = **50 distinct names**.

**Exactly one pass is in no lane's ledger: `Settle Liquids`
(`WorldGen.cs:16132-16210`, census #19).** d102's table hands it to d103,
d103's table hands it to d102, and neither landing ledger contains it — the
same orphan pattern d106 documented and adopted for Grass Wall (`12f`).
This lane adopts it, exactly as d106 adopted its orphan. Nothing here
touches any pass another lane has landed or listed in its own ledger.

## The adopted pass (C# body read in full, ≤100-line windows)

`WorldGen.cs:16132-16210` (`GenPassNameID.SettleLiquids`, weight 9398.525,
vanilla slot: after Jungle Chests `:15977`, before Remove Water From Sand
`:16211`):

1. `Skyblock.denyAllGeneration` outer gate (deferred seed arm — the port has
   no skyblock runtime gate; a classic world always enters).
2. The ocean retype sweep `:16137-16146`: every `oceanDepths(i,j)` cell with
   `liquid > 0 && liquidType() != 0` → water (`WorldGen.oceanDepths`
   `:10963`).
3. `SecretSeed.extraLiquid` / `notTheBees` arms `:16147-16155` (special-seed
   lanes d107/c30 own those bodies; flag-gated, classic no-op).
4. `Liquid.worldGenTilesIgnoreWater(true)` (`Liquid.cs:75-81`: tileSolid
   10/192/191/190 off) → `Liquid.QuickWater(3)` (`:104-148` + `SettleWaterAt`
   `:150-289`, the bottom-up drop-and-fan pre-settle) → `WorldGen.WaterCheck`
   (`:80415-80494`, the re-registration sweep).
5. The settle loop `:16159-16204`: `quickSettle = true`, 10 outer rounds,
   per-round budget `num3 * 5` on `Liquid.UpdateLiquid` until
   `numLiquid == 0`, `WaterCheck` between rounds.
6. `WorldGen.ClearPendingLiquid` (`:80496-80513`) → `quickSettle = false` →
   `worldGenTilesIgnoreWater(false)` → `Main.tileSolid[484] = false`
   (the Gems-pass sand-slide exemption, `:15050` band, made permanent).

Rust seam (planned): `worldgen/settle_liquids.rs` + the pipeline step
`"settle liquids"` (`vanilla: "Settle Liquids"`) at the vanilla slot; the
settle loop rides the port's `LiquidSystem` (`liquid.rs`, the faithful
`UpdateLiquid`/`AddWater`/buffer port) — the R-12-5 "minimal Liquid update"
dependency already exists. Per-pass instance stands in for the C# statics
(documented deviation: `ClearPendingLiquid` + the `numLiquid == 0` exit
invariant make the instance boundary equivalent). No RNG draws in the whole
pass (QuickWater/SettleWaterAt/WaterCheck/UpdateLiquid draw nothing) — the
insertion is ledger-neutral for every other step.

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| census + orphan claim (this doc) | (this commit) | claimed |
| Settle Liquids | — | pending |
