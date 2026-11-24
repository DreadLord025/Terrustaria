# S1-E-7 wave C (W4-c17) — disjoint-range claim + live-set census

> Worker W4-c17, branch `task/W4-c17-buff-effects-wave-c` @ main `a725acb0`.
> Code-only wave: tests WRITTEN NOT RUN (no cargo/rustc per the worker contract);
> compile-by-inspection. This note is the early claim commit — the range below is
> this worker's; c16 takes the first missing block, c18 the third.

## The live-set census

C# chain: `Player.UpdateBuffs` (`decompiled/Terraria/Player.cs:9855-12069`),
every `buffType[j] == N` head plus the six range heads (`95..97`, `170..172`,
`98..100`, `176..178`, `173..175`, `179..181`) = **270 distinct ids**.

**Live-set in the buff_effects modules** (the task's census key): the arm heads
and table rows of `buff_effects.rs` (wave B, 47 ids), `buff_effects_misc.rs`
(wave D: mount family + pets + DoT/status + flasks, ~150 ids) and
`buff_effects_accessory.rs` (earlier wave-C live pass: 37, 38, 376, 146, the
light pets, fairies, rabid, `PET_BUFF_ARMS`, `IMBUE_ARMS`), plus the
`mount.rs::mount_type_of_buff` membership the misc module's mount arm reads.

**69 chain ids are missing from the modules.** In C# chain order they form
three contiguous blocks:

1. **First block (c16)** — `Player.cs:9914-10048` (21 ids):
   383, 159, 158, **192**, **321**, 6, 8, 9, 343, 10, 12, 13, 14, 15, 17, 18,
   **30**, 31, 32, 35, 160. (Most already run live in `player.rs`'s wave-A
   match — c16's job is the module table + the three with no live owner at
   all: 30 Bleeding `:10029`, 192 Sugar Rush `:9935`, 321 Brain of Confusion
   `:9940`.)
2. **SECOND block (c17 — THIS WORKER)** — `Player.cs:10318-10461` (**20 ids**):
   **94** (Mana Sickness — table arm; the live half already runs via
   `potions::apply_mana_sickness_arm` from `player.rs:6501`, the table mirrors
   it and is NOT wired twice), **396** (Mana Heat), **95-97** (Beetle
   Endurance), **170-172** (Solar Shield), **98-100** (Beetle Might),
   **176-178** (Nebula Mana), **173-175** (Nebula Life), **179-181** (Nebula
   Damage). Of these, **19 have no owner anywhere in the crate** (only 94 is
   live) — this is the largest wholly-unported contiguous block of the chain.
   The block ends before `62` (Ice Barrier, `:10462`), which already has a
   live partial (the endurance half, `player.rs:6526`) — NOT mine.
3. **Third block (c18)** — `Player.cs:10486-10972` (28 ids): the summon
   keep-alive family 49, 83, 64, 125, 126, 133, 134, 135, 214, 139, 140, 182,
   213, 216, 355, 385, 393, 394, 386, 389, 390, 325, 335, 263, 271, 322, 187,
   188, 161 (all live via `player.rs::SUMMON_BUFF_ARMS:2312-2458`; c18's job
   is their module-table form).

Cross-check (secondary census — ids with **no owner anywhere**, live matches
included): 22 ids = {30, 192, 321} ⊂ block 1, my 19 ⊂ block 2, none in block 3.
Both readings put the SAME second block on this worker: **`:10318-10461`,
ids 94, 396, 95-100, 170-181.** The ranges are disjoint by C# line:
c16 ⊂ `:9914-10048`, c17 = `:10318-10461`, c18 ⊂ `:10462-10972`.

## Shape

Pure accumulator table, the `buff_effects.rs` wave-B seam pattern:
`crates/tr-app/src/buff_effects_potions_sets.rs`. The simple arms (94, 396)
fold into a `PotionSetEffects` accumulator via
`apply_potion_set_effect_arm(e, buff_type, buff_time, ctx)`. The set-tier arms
(beetle/solar/nebula) read AND write the whole buff bar (their lower-tier
`DelBuff` walks scan every slot, and the Nebula downgrade rewrites
`buffType[b]`/`buffTime[b]`), so they ride a slot-ordered driver over
`&mut BuffSlots` + the accumulator, mirroring the C# `for j` loop with the
`DelBuff(j); j--` rescan. Wiring into the live tick is the merge step (not
this wave), recorded in ANALYSIS.

**RNG census of `:10318-10461`: zero draws.** No `Main.rand`/`rand.Next` in
any arm; the Nebula downgrade helper (`UpdateBuffs_NebulaBuffs`,
`Player.cs:12070-12079`) is deterministic timer arithmetic
(`buffTime == 2 && level > 1` → level−1, buff id−1, time 480).
