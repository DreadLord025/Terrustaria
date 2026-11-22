# S1-E-7 wave D (W4-c18) — disjoint-range claim + live-set census

> Worker W4-c18, branch `task/W4-c18-buff-effects-wave-d` @ main `a725acb0`.
> Code-only wave: tests WRITTEN NOT RUN (no cargo/rustc per the worker contract);
> compile-by-inspection. This note is the early claim commit — the range below is
> this worker's; c16 takes the first missing block (`:9914-10048`), c17 the
> second (`:10318-10461`, claim `fa1e8930`, `28c-wave-c-claim.md`), c18 the THIRD.

## The live-set census (re-derived at `a725acb0`)

C# chain: `Player.UpdateBuffs` (`decompiled/Terraria/Player.cs:9855-12069`),
every `buffType[j] == N` head plus the six range heads (`95..97`, `170..172`,
`98..100`, `176..178`, `173..175`, `179..181`) = **270 distinct ids**
(head scan reproduced below — 244 `==` heads + the six ranges' 18 ids, minus
the `<= 0` guard at `:9867` and shared ids).

**Live-set in the buff_effects modules** (c17's census key): the arm heads and
table rows of `buff_effects.rs` (wave B, 47 arms), `buff_effects_misc.rs`
(mount family `131/168/265/279/305` + the 82-row `PET_ARMS` table + DoT/status
+ flasks) and `buff_effects_accessory.rs` (37/38/376/146, light pets, fairies,
rabid, `PET_BUFF_ARMS`, the 8 `IMBUE_ARMS`).

**69 chain ids are missing from the modules.** In C# chain order they form
three contiguous blocks (c17's partition, re-verified — the counts match):

1. **First block (c16)** — `Player.cs:9914-10048` (21 ids):
   383, 159, 158, 192, 321, 6, 8, 9, 343, 10, 12, 13, 14, 15, 17, 18, 30,
   31, 32, 35, 160. NOT mine.
2. **Second block (c17)** — `Player.cs:10318-10461` (20 ids):
   94, 396, 95-97, 170-172, 98-100, 176-178, 173-175, 179-181. NOT mine
   (claimed `fa1e8930`).
3. **THIRD block (c18 — THIS WORKER)** — `Player.cs:10462-10976` (**30 ids**):
   **62** (Ice Barrier — the endurance half is live in `player.rs:6526` but
   the DelBuff-when-healthy half, the `iceBarrier` flag and the
   `iceBarrierFrameCounter`/`iceBarrierFrame` advance have no owner; the
   light already rides `player_lights.rs`), then the 29 summon keep-alive
   arms **49, 83, 64, 125, 126, 133, 134, 135, 214, 139, 140, 182, 213, 216,
   355, 385, 393, 394, 386, 389, 390, 325, 335, 263, 271, 322, 187, 188,
   161** — all live via `player.rs::SUMMON_BUFF_ARMS` (`:2312-2458`) +
   `update_summon_buff_arms` (`:4000-4067`); c18's job is their module-table
   form (the wave brief's "~40 ids" overcounts: the live-set census puts 30
   chain ids in this block; 29 summon + 62).

Boundary: the block ends at 161's arm close (`:10976`); the next head 37
(`:10977`, Horrified) is already live in `buff_effects_accessory.rs` — NOT
mine. The block starts at 62 (`:10462`), the first head after c17's Nebula
Damage range head (`:10453`). Disjoint by C# line:
c16 ⊂ `:9914-10048`, c17 = `:10318-10461`, c18 = `:10462-10976`.

Per-id arm heads (the witnesses' spine): 62 `:10462`, 49 `:10486`, 83
`:10505`, 64 `:10521`, 125 `:10537`, 126 `:10553`, 133 `:10569`, 134 `:10585`,
135 `:10601`, 214 `:10617`, 139 `:10633`, 140 `:10649`, 182 `:10665`, 213
`:10681`, 216 `:10697`, 355 `:10729`, 385 `:10745`, 393 `:10761`, 394
`:10777`, 386 `:10793`, 389 `:10809`, 390 `:10825`, 325 `:10841`, 335
`:10857`, 263 `:10877`, 271 `:10897`, 322 `:10913`, 187 `:10929`, 188
`:10945`, 161 `:10961`.

## RNG census of `:10462-10976`: ZERO direct draws

No `Main.rand`/`rand.Next` in any arm of the block (scan of the exact line
window: zero hits). Indirect draws, all deferred with their seams:
- **216**'s respawn `Projectile.NewProjectile` (`:10712`) — the projectile
  init rolls; the spawn seam (`spawn_with_data_ai_owned`) owns them, exactly
  as the live `run_baby_bird_arm` already models.
- **335/263**'s status updaters `UpdateAbigailStatus` (`Player.cs:12274-12293`)
  / `UpdateStormTigerStatus` (`:12295-12335`) — `Kill`/`NewProjectile` side
  effects need the pool + `FindNewestAI_164Minion` (`:12260-12272`); the live
  port carries both as STUBS (`player.rs:4137/:4142`, M4-PROJ-8 phases 4/6).
  The module ports the DETERMINISTIC decision cores (`GetDesiredStormTigerMinionRank`
  `:12337-12354`, the kill-vs-spawn choice) as pure functions and records the
  pool side effects as requests; no stream moves until the wiring.
- **62**'s `Lighting.AddLight` (`:10466`, RGB 0.1/0.2/0.45) draws nothing —
  recorded as a light request like wave B's `inferno_light`.

## Inferno note (roster dependency)

The Inferno (116) NPC/PvP aura scan (`Player.cs:10224-10263`) is **NOT in this
block's line range** (`:10212-10264` ⊂ wave B's `:10049-10317`); wave B already
ported the flag/light/cadence with the `INFERNO_*` constants and deferred the
world-roster scan to the wiring step. This wave adds no Inferno code — the
roster dependency note stands unchanged (ANALYSIS "Buff effect table (S1-E-7
wave B)").

## Shape

Pure accumulator table, the `buff_effects.rs` wave-B seam pattern:
`crates/tr-app/src/buff_effects_summons.rs`. `apply_summon_effect_arm` matches
one id and folds its writes into a `SummonEffects` accumulator (per-minion
flags keyed by an enum, the 18000 refresh as a return value, 216/335/263's
pool requests recorded, not executed). `run_summon_buff_chain` is the
slot-ordered driver over `&mut BuffSlots` with the C# `DelBuff(j); j--`
rescan (`:10482-10483` etc.), mirroring `player.rs::update_summon_buff_arms`
but pool-free. Wiring into the live tick is the merge step (not this wave):
the live `SUMMON_BUFF_ARMS` path stays authoritative until the waves merge;
this module is the disjoint table form the merge reconciles onto.
