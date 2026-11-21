# 148 — BrokenTrapCleanup wg-seed: census + dedupe — d106 R5 ALREADY LANDED the same pass; two fidelity items + the seam resolution

> **STATUS: TASK-SEED (dedupe-checked) — 2026-11-21, branch
> `task/W4-e20-wiring-device-arms-2` (retire phase).** The e20 residual
> from research-71 §7c ("the BrokenTrapCleanup pass is the one residual —
> worldgen lanes") is CENSUSED here against the C# pass bodies and
> dedupe-checked against `origin/task/W4-d106-worldgen-passes-6`.
> **Verdict: SAME pass, already ported** (d106 R5 `e2c2609c5`, tip
> `e2c2609c512d36b58f4f8cd54413f498ef0955f2`) — do NOT re-port. The seed
> below is the verification/finish worklist for the wg lanes. Code-only:
> no cargo/tests run; C# read at the cited lines via direct junction
> file paths.

## 1. C# census — the pass family (WorldGen.cs)

| C# fn | Lines | Body summary |
| --- | --- | --- |
| `AddGenerationPass(GenPassNameID.BrokenTrapCleanup, …)` | `:22121-22140` | gate `!noTrapsWorldGen \|\| tenthAnniversaryWorldGen \|\| notTheBees`; `Lang.gen[82]` message; 50-margin x-outer/y-inner sweep; per red-wired (`tile.wire()` — RED only) cell not in the pass-wide `list`, call `ClearBrokenTraps`. Vanilla slot: after `SpeleothemsAndGemTrees` (stalac, ends `:22120`), before `FinalCleanup` (`:22141`). No RNG draws |
| `IsItATrap` | `:26788-26803` | `!active → false`; `actuator → true`; `IsAMechanism[type] → !IgnoreWhenValidatingTraps[type]`; else false |
| `IsItATrigger` | `:26805-26823` | active && (`IsATrigger[type]` \|\| `type==467 && frameX/36==4` \|\| `type==314 && Minecart.IsPressurePlate(tile)`) |
| `ClearAllBrokenTraps` | `:26825-26838` | the same 50-margin sweep without progress — **ZERO callers in the decompile** (grep: definition only). Dead C# API; no port needed |
| `ClearBrokenTraps` | `:26840-26958` | two-list swap BFS (`t`/`t2`), outer `num=2000` / inner `num2=2000` decremented per iteration; `InWorld(item,5)`; red-wire gate; record in `pointsWeAlreadyWentOver` + local `list`; trap/trigger flags; `flag2 & flag` → working circuit, break+return (leave alone); queue 4 neighbours not in `list` (order: −x, +x, −y, +y). Strip pass (`:26919-26957`): `wire(false)`; actuator → clear + boulder-above (`Sets.Boulders[type] && frameX%16==0`) resets the 2×2 (`i 0..2, j −2..0`) active boulder cells `ResetToType(1)`; `IsItATrap && type!=105` → `KillTile(x,y)`; else `IsItATrigger` → 314 sets `frameX=1` (NOT kill, `:26948-26951`), others `KillTile(x,y)` |
| `Minecart.IsPressurePlate` | `Minecart.cs:549-560` | null → false; `active && type==314 && (frameX==20 \|\| frameX==21)` — the PRESSED variants 22/23 do NOT count |
| `Tile.ResetToType` | `Tile.cs:249-259` | liquid 0, `sTileHeader=32` (active only), other headers 0, frames 0, new kind; wall kept |
| `WorldGen.KillTile` signature | `:63517` | `(int i, int j, bool fail = false, bool effectOnly = false, bool noItem = false)` — the pass calls the 2-arg form → **all three defaults FALSE** |

## 2. Dedupe check — d106 R5 is the SAME pass, landed

`origin/task/W4-d106-worldgen-passes-6` tip `e2c2609c5` ("W4-d106 R5:
port Remove Broken Traps pass …") lands in
`crates/tr-core/src/worldgen/cleanup.rs`:

| C# | d106 port | Check |
| --- | --- | --- |
| pass body `:22121-22140` | `remove_broken_traps` :705 + `pipeline.rs:1433` `step!("remove broken traps", …)` after stalac (`:2810` step fn) | gate De-Morgan-equivalent (`no_traps && !anniversary && !not_the_bees → return`); 50 margins; pass-wide `visited`; slot correct ✓ |
| `IsItATrap` | `is_it_a_trap` :539 (`SET_IS_A_MECHANISM`/`SET_IGNORE_WHEN_VALIDATING_TRAPS`) | verbatim ✓ |
| `IsItATrigger` | `is_it_a_trigger` :556 | verbatim incl. 467/36==4; the 314 arm INLINES `matches!(t.frame_x, 20\|21)` after the active gate — semantically identical to `IsPressurePlate` (see §3c) ✓ |
| `ClearBrokenTraps` | `clear_broken_traps` :591 | two-list swap BFS, both 2000 counters, InWorld-5, red-wire gate, both lists, veto, 4-neighbour order, strip pass, boulder 2×2 reset, 105 exemption, 314 `frame_x = 1` :691 ✓ (two edge items §3a/§3b) |
| `ResetToType` | `reset_to_type` :575 | `liquid=0, flags=FLAG_ACTIVE, meta=0, frames=0, kind` ✓ |
| `WireState::wire` red-bit | `wiring.rs:126` accessor (cites the ClearBrokenTraps family) | ✓ |

Witnesses (WRITTEN NOT RUN, d106 code-only): `r5_a_broken_trap_circuit_is_stripped`
:802, `r5_a_working_circuit_is_left_alone` :821,
`r5_the_pass_is_deterministic_and_no_traps_skips_it` :840. d106's PORT_MAP
row (:103) records the pass 🟡 "pins move by design, re-pin deferred to
gate". **No e20 code duplicates any of this** — e20 only landed the
`IsPressurePlate` dependency (`6d1754719`, `tr-app/src/minecart.rs:310`
`pub(crate) fn is_pressure_plate(tile: Option<&tr_core::Tile>)`,
`dead_code` until a consumer).

## 3. The seed — verification/finish items for the wg lanes (d106 or successor)

**Do NOT re-port the pass.** Worklist:

a. **`kill_tile` no-item deviation — VERIFY.** d106's strip calls
   `kill_tile(world, x, y, true)` (`edit/kill.rs:193`
   `pub fn kill_tile(world, x, y, no_item: bool)`); the C# pass calls
   `KillTile(x, y)` with **`noItem = false`** (`WorldGen.cs:63517`
   defaults). Either justify the `true` as a documented gen-mode stand-in
   (do the vanilla gen-time item spawns survive to the loaded world?
   check what `Main.item` does during generation) and record it in
   ANALYSIS/d106's deviations, or flip to `false` if the port's gen
   pipeline keeps spawned items. Affects the broken-trap/trigger death
   drops only; no RNG either way if the kill path is draw-free at gen.

b. **BFS leftover-cell edge — VERIFY.** C# `Utils.Swap(ref t, ref t2)`
   (`:26856`) does NOT clear the swapped-in `t2`: if the inner loop breaks
   on `num2 <= 0` with `t` still non-empty, the leftover cells survive in
   `t2` and are processed in later outer waves (neighbour queues APPEND to
   them). d106 does `t2.clear()` after the swap — leftovers are dropped.
   Only reachable for networks needing >2000 inner pops in one wave
   (huge wired blobs); decide: reproduce the C# leftover carry (remove
   the `clear()`, mirror the swap exactly) or document the cap-edge
   deviation. The outer `num` exhaustion path needs the same read.

c. **`IsPressurePlate` seam resolution (cross-crate).** d106's inline
   `matches!(frame_x, 20|21)` lives in tr-core; e20's
   `minecart::is_pressure_plate` lives in tr-app (`pub(crate)`) — tr-core
   cannot call it. Keep ONE truth: either move the helper to tr-core
   (beside the track table if it ever moves, research-71 §1) and re-point
   both, or keep d106's inline and re-doc e20's fn as the app-side mirror
   (its `dead_code` attribute then stays justified — its named consumer
   ports the predicate inline). The e20 witness
   (`6d1754719`, WRITTEN NOT RUN) pins the predicate either way.

d. **`ClearAllBrokenTraps` — no port needed** (zero callers, §1). Record
   as discarded-dead-API if a future census re-flags it.

e. **Pointer updates at merge:** research-71 §7c's "unported, worldgen
   lanes" residual line is superseded by d106 R5 — the retirement note
   (research-71 §8) and the devices-family bible (research-149) point
   here; d106's PORT_MAP row already names the pass. The worldgen
   deterministic pins move by design (d106's own note) — re-pin at the
   gate, not on this lane.

*Sources: `decompiled/Terraria/WorldGen.cs` :22110-22149/:26788-26960/
:63517, `Minecart.cs` :548-561 (direct file reads); d106 objects via
`git show origin/task/W4-d106-worldgen-passes-6:…`. No cargo/tests/
rustfmt/graphify (code-only retire lane).*
