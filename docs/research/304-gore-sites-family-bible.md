# 304 — Gore-sites family bible + the e5-vs-f10 HitEffect keep-one (W4-e5, task/W4-e5-gore-sites-2, code-only)

Number take: **304**, per the d54 collision registry
(`150-research-number-registry.md`, round 3 §7.3: "Numbers 304+ are FREE…
check this registry AND a fresh all-tip sweep first"). A fresh all-tip
sweep of every local + remote branch's `docs/research/` (this session)
finds **no doc ≥304 at any tip** — 304 is single-claimant, this lane.
(The registry's own round-3 block ends at 303; the post-cutoff f4 SEAL
`26728bd87` took 283 and flagged 243/244 cascades — none reach 304.)

This doc is the **gore-sites-family bible** from e5's seat: it consumes
the f5 GORE SEAL (`b68eac297`, research/111 §§5-10), the f10 wave-C
completion (`19e1139d6`, 1441 sites / 170 commits) and f10's own
overrun-delete (`afea4c0fc`), the d55 wave-A compose (`ec78d1524`), and
EXECUTES the one overlap e5 owns — the 551 flipped-twin keep-one (§2).
It also records a census correction the seal's f5-only walk could not
see (§3: the FIFTH hit-half site) and assigns its body (§4).

## 1. The family census — re-verified at this tip

The seal §10.2 gate-command re-run (direct file path, no junction
recursion), 2026-11-21:

| file | `Gore.NewGore*` sites | owner lane(s) |
| --- | --- | --- |
| `NPC.cs` | **1345** | d55/e5/f10/f5 (HitEffect) + f4-E/g61/reserved (other) |
| `Projectile.cs` | **165** | g16 (`projectile_gore_a..o`) |
| `WorldGen.cs` | **36** | g17 (`world_gore_a/b`) |
| `Player.cs` | **32** | g17 (`player_gore_a..e`) |
| `Main.cs` | **0** | — (only `Gore.goreTime`/TimeLogger rows) |
| **TOTAL** | **1578** | zero unowned |

The NPC HitEffect window `:81660-89835` = **1304** sites; the four
claim-first lane windows (seal §5.2, re-counted here by direct grep):

| lane | window | sites | module / tip |
| --- | --- | --- | --- |
| d55 wave A | `:81660-81941` | 26 | `npc_hit_effect_a.rs` (`ec78d1524`) |
| e5 wave B | `:81942-82205` | 40 | `npc_hit_effect_b.rs` (`a62a32ce6` → this keep-one) |
| f10 wave C legit | `:82206-85559` | 385 | `npc_hit_effect_c.rs` (`afea4c0fc`, post-overrun-delete) |
| f5 wave D | `:85560-89835` | 853 | `npc_hit_effect_d.rs` (`b68eac297`) |

26 + 40 + 385 + 853 = **1304** ✓. The 41 NPC sites OUTSIDE HitEffect
(17 f4-E `211-npc-gore-wave-e-claims.md` + 8 g61 `273-g61-spawn-family-final.md`
+ 16 reserved-argument) reconcile per seal §5.2 — unchanged.

## 2. KEEP-ONE — the 551 flipped twin (the e5-vs-f10 overlap)

**The overlap.** e5's module as pushed (`a62a32ce6`) ported BOTH
`spriteDirection` arms of the 551 death: the `!= -1` body (`:82193-82202`,
its sites 67-76) AND the `-1` mirrored twin (`:82206-82215`, its sites
77-86), under one `dd2_crystal_death` fn with an internal
`if npc.sprite_dir != -1 { … } else { … }`. But the seal's window census
(§5.2) draws the e5/f10 boundary at `:82205`/`:82206`: the flipped twin
`:82206-82215` is **f10's** `betsy_flipped_death` (its sites 71-80), the
FIRST arm of f10's legit 385. f10's own overrun-delete `afea4c0fc`
explicitly KEPT it ("KEEPS the legit 385 sites 1-561 (:82206-85559)… the
flipped-Betsy dispatch witness"). So both modules port the same 10 C#
sites — a double-port that would emit 20 gores + run the shared 20-dust
preamble twice on a flipped Betsy death.

**The verdict: e5 DROPS the flipped else-arm; f10 KEEPS `betsy_flipped_death`.**

* f10's claim is the seal's canonical window (`:82206-85559` = 385).
  Dropping f10's flipped arm instead would cut 385 → 375 and break the
  1578 reconciliation (1304 → 1294). Dropping e5's keeps both windows
  exact: e5 = 40 (`:81942-82205`), f10 = 385.
* The two dispatch gates are EXACT complements on the death path:
  e5 `npc_type == 551 && hp <= 0 && sprite_dir != -1` (this commit) vs
  f10 `npc_type == 551 && hp <= 0 && sprite_dir == -1` (`afea4c0fc`
  :3570). Exactly one fires per death.
* The shared 20-dust type-5 preamble (`:82187-82190`) runs exactly ONCE:
  e5's `dd2_death_dust` for `!= -1`, f10's own `death_dust_loop(…, 5, …, 20)`
  copy inside `betsy_flipped_death` for `== -1`. The C# runs the preamble
  before the `spriteDirection` branch (`:82187` precedes `:82191`), so
  each lane carrying its own copy gated on its arm reproduces the C#
  order with zero double-run. (This is the same shape as f10's b1
  header note "The shared 20-dust death loop is ported here; the merge
  keeps ONE copy" — resolved here by complementary gating, not by
  deleting either preamble.)
* The `CheckProgress` percent gate (`:82174-82177`, hit path) stays e5's
  (`dd2_crystal_crossed_percent` → `HitEffectOutcome::dd2_crystal_check_progress`);
  f10's 551 gate is death-only and never touches it. No overlap there.

**Executed in this commit** (`npc_hit_effect_b.rs`): the `else` branch of
`dd2_crystal_death` deleted (the 10 flipped `new_gore` calls), the fn
doc rewritten to the `!= -1`-only claim, the dispatch gate widened to
`hp <= 0 && sprite_dir != -1`, the module scope table cut to sites 31-76,
and the flipped-twin witness `w4_e5_the_crystal_minus_one_twin_mirrors_the_offsets`
REPLACED by `w4_e5_the_flipped_crystal_death_is_f10s_claim` (the `-1`
death draws NOTHING in e5 — no gores, no dust, `rng.peek()` unchanged —
while the `!= -1` body keeps its full 20-dust + 10-gore census).
WRITTEN NOT RUN (code-only).

**Naming reconciliation.** NPCID 551 = `DD2Betsy` (`NPCID.cs`). e5's
module calls it "the 551 DD2 crystal" (`dd2_crystal_*` fn names); f10
calls it "Betsy". They are the SAME kind — the Eternia Crystal is
548/522, a different arm. e5's names are kept for API stability with a
header note; the kind is Betsy. (Also corrected here: e5's "667 (Dark
Caster?)" → **667 GoldenSlime**, `NPCID.cs`.)

## 3. CENSUS CORRECTION — the fifth hit-half site (`:82045`)

The seal §1.2 split the 1304 HitEffect sites into **1300 death + 4 hit**
(`:86532/86947/86951/88648`), but its brace-walk covered only f5's
window `:85560-89835`. A FULL-window brace-walk (`:81654-89835`, tracking
`this.life <= 0` / `this.life > 0` block context with the `else`-flip)
finds a **FIFTH hit-half site: `:82045`** — the 667 GoldenSlime HIT arm's
3-gore twin, the `else` (`:82041`) of the `:82031 if (this.life <= 0)`
death gate. It sits in **e5's window**, which the seal's f5-only walk
never classified.

The corrected partition:

| lane | death-half | hit-half | total |
| --- | --- | --- | --- |
| d55 | 26 | 0 | 26 |
| e5 | 39 | 1 (`:82045`) | 40 |
| f10 legit | 385 | 0 | 385 |
| f5 | 849 | 4 | 853 |
| **HitEffect** | **1299** | **5** | **1304** |

(The full-window walk also confirms the every-hit / dmg-gated arms the
seal counted as death-half by convention — the 437 Cultist Tablet
`dmg == 9999` burst `:82676`, the party Blue Slime confetti `:84738`,
the Golem body/fist `:84968-84996`, the snail trio `:85271/85292/85313`
— are life-independent or death-gated; none adds a hit-half gore site
beyond `:82045`. The 5 hit-half sites are exactly
`:82045/86532/86947/86951/88648`.)

**The 1578 total and the zero-unowned verdict are UNCHANGED** — only the
death/hit partition moves by one site (1300/4 → 1299/5). e5's 40-site
window is 39 death + 1 hit, consistent with the seal's "e5 40" count.
The death-bible paragraph (seal §9.3) should read "1299 HitEffect
death-half gore sites" + "5 hit-half bodies"; the four-lane death counts
become d55 26 + e5 39 + f10 385 + f5 849 = 1299.

## 4. The `:82045` body — g62 EMIT-AT-WIRING (the seal §7 pattern, extended)

The `:82045` body is **UNPORTED by any window lane**: e5 ports only the
667 DEATH arm (`skeleton_667_death`, gated `hp <= 0`); f10's window
starts at `:82206`; d55 ends at `:81941`. The HIT arm's draws already
ride **g62** (`npc_hit_dust.rs`, tip `7ec3df62b`): its live 667 arm
(`s.life > 0 && npc_type == 667`) draws the 3×(`Next(1,10)` then
`Next(4)`) velocity-overwrite pairs (`:82043-82047`, `let _ =` discards)
BEFORE the 7-dust loop — but **NOT the 3×2 NewGore kicks**.

The C# `:82045` is `int num36 = Gore.NewGore(…, 1218)` — the pool draws
the kicks (`Next(10,31)` Y, `Next(-20,21)` X, `Gore.cs:1327-1328`) at
spawn, THEN `:82046` overwrites the slot velocity with
`Next(1,10)*0.3*2*hitDirection` / `-(2.5+Next(4)*0.3)`. So the assignment
(the seal §7 EMIT-AT-WIRING recipe, applied to this fifth site):

1. `replay_hit_dust` gains the `gore: &mut GorePool` param (the same
   wiring-caller change the seal §7 already mandates for `:86947/86951/88648`).
2. The 667 live arm's loop becomes, per iteration:
   `let idx = gore.new_gore(rng, world, Vec2::new(s.x, center_y - 10.0), Vec2::ZERO, 1218, 1.0);`
   (the pool draws the 2 kicks) THEN the existing
   `let _ = rng.next_range(1, 10); let _ = rng.next_max(4);` become the
   velocity OVERWRITE on `gore.slot_mut(idx)` (the ×2 / −(2.5+…) form,
   NOT the death arm's ×2.5 / −(3+…)).
3. **g62's discard census + witness gain +6 draws** (3×2 kicks) — the
   kicks must be inserted BEFORE each overwrite pair, in C# order. The
   existing g62 witness `w4_g62_the_queen_minion_live_gate_and_667_draws`
   (which replays 3×2 overwrite draws + 7 dust) must be re-derived to
   3×(2 kicks + 2 overwrite) + 7 dust. This is a g62-side witness fix at
   wiring, flagged here so it is not missed.

`center_y - 10.0` = `position.Y + height/2 - 10` (the C# `Center.Y - 10f`);
g62's snapshot carries the box, so no capture change. The body is
stream-correct ONLY with the kicks inserted — without them the 667 hit
arm desyncs by 6 draws on every live strike. **Owner: g62 emit-at-wiring;
executor: the wiring wave.** No window-lane port (e5/f10/d55/f5) takes it.
g62's header note "the bodies d55's" (`npc_hit_dust.rs:797`) is the same
blanket mislabel the seal §7 corrected for the four f5-window sites —
d55's window ends at `:81941` and has no 667 arm; this §4 assignment
(g62 emit-at-wiring) supersedes it for `:82045`.

## 5. The d55 compose (wave A, `ec78d1524`)

d55's wave A (`npc_hit_effect_a.rs`, 26 sites `:81660-81941`) is the
dispatch ROOT e5 appends to: `hit_effect` runs the party-hat rows, the
686/594 `NewGorePerfect` bursts, the daybreak dust arm, the 668 Deerclops
7-gore death, the 636 Empress dust census + first gore (`:81941`), then
the 636 8-gore tail (`empress_tail_gores`, e5's sites 31-38) and finally
calls `hit_effect_b`. The compose is clean:

* d55's window ends at `:81941` (the Empress first gore); e5's begins at
  `:81942` (the tail). No site overlap.
* The seal §7 verdict "d55 does NOT extend" holds — d55 has zero
  133/186/632/694 refs (verified at `ec78d1524`), so the four f5-window
  hit-half bodies are NOT d55's; and the fifth (`:82045`, e5's window) is
  g62's (§4), not d55's either. d55 stays wave-A-only.
* The `hit_direction`/`dmg` params e5 added to `hit_effect` (the C#
  `HitEffect(int hitDirection = 0, double dmg = 10.0)`, `:81654`) ride
  the whole dispatch; d55's arms ignore them (death halves use flat
  counts), e5's 551 `CheckProgress` gate consumes `dmg`. No conflict.

## 6. The family bible — one paragraph (e5 seat, post-keep-one)

**The `Gore.NewGore*` family of Terraria 1.4.5.8 is 1578 sites across 5
files (NPC 1345 / Projectile 165 / WorldGen 36 / Player 32 / Main 0),
every site owned: the NPC HitEffect window's 1304 split 1299 death-half
(d55 26 `:81660-81941` + e5 39 `:81942-82205` + f10-legit 385
`:82206-85559` + f5 849 `:85560-89835`) + 5 hit-half (`:82045` g62
emit-at-wiring §4 + the seal §7's `:86532` wiring-caller latch +
`:86947/86951/88648` g62 emit-at-wiring); the 41 NPC other-method sites
(f4-E 17 + g61 8 + 16 reserved-arg); Projectile 165 (g16) + WorldGen 36
+ Player 32 (g17). The ONE e5-owned overlap — the 551 flipped twin
`:82206-82215` — is RESOLVED: e5 drops its else-arm, f10 keeps
`betsy_flipped_death`, complementary `sprite_dir` gates run the shared
20-dust preamble exactly once (§2, EXECUTED this commit). The f10
overrun into f5's window was already deleted by f10 itself (`afea4c0fc`,
the seal §8 bands re-censused at the live tip). Zero unowned, zero
unassigned bodies, zero double-ports; all witnesses WRITTEN NOT RUN
(code-only), owed one compile window before merge.**

## 7. Verification performed (code-only)

No cargo/tests/rustfmt/graphify invoked (lane directive). Re-derived by
direct read/grep/brace-walk:

* The seal §10.2 gate-command re-run: NPC 1345 / Proj 165 / WorldGen 36 /
  Player 32 / Main 0 = **1578** (direct `Select-String` per file).
* The four-window recount: d55 26 (`:81660-81941`), e5 40
  (`:81942-82205`), f10 385 (`:82206-85559`), f5 853 (`:85560-89835`)
  = 1304; the e5 window's 40 literals enumerated (`:81942-81949` ×8,
  `:81956-81958` ×3, `:82012-82013` ×2, `:82037` + `:82045` loop sites,
  `:82103-82107` ×5, `:82134-82140` ×7, `:82167-82169` ×3,
  `:82193-82202` ×10).
* The full-window brace-walk (`:81654-89835`, 288 life-gates) → 5
  hit-half sites `:82045/86532/86947/86951/88648`; the `:82045` else-arm
  read directly (`NPC.cs:82041-82048`).
* f10's `betsy_flipped_death` + gate read at the LIVE tip `afea4c0fc`
  (`npc_hit_effect_c.rs:293-324` fn, `:3567-3572` gate, `:6343-6370`
  flipped dispatch witness) — the exact complement of e5's new gate.
* g62's live 667 arm read at `7ec3df62b` (`npc_hit_dust.rs:792-823`):
  the 3×(Next(1,10)+Next(4)) discards present, the NewGore kicks ABSENT
  (zero `new_gore`/`GorePool` refs in the module) — the §4 +6-draw fix.
* NPCID 551 = DD2Betsy, 667 = GoldenSlime, 548 = DD2EterniaCrystal
  (`NPCID.cs` direct grep).
* The registry-150 round-3 free floor (304) + a fresh all-tip sweep
  (no doc ≥304 at any tip) — 304 single-claimant.

Witnesses: the e5 module's 12 (the flipped-twin witness REPLACED by the
keep-one witness, §2) remain WRITTEN NOT RUN. The g62 667-hit witness
re-derivation (§4) is owed at wiring. No compile window run here.