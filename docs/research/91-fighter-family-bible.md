# 91 — FIGHTER-FAMILY BIBLE (SEALED, FINAL — corrected keep-one): d22 canonical styles 3-6 + f35 owns the heal seam, g4 never-wholesale (2 cherry hunks), g8/g9/g10 disjoint, d34-d43 roster table

W4-d22 SEAL phase (S4), branch `task/W4-d22-npc-ai-styles-3-6` (retire
tip `a1a21235`, DONE base `bcd42ddd`). Code-only lane: no cargo/tests/
rustfmt/graphify invoked. This is the single merger-facing record for
the fighter family — the union of R1-R4 (`fb99a047`/`28866c7c`/
`78386ab1`/`a1a21235`) plus the seal re-probes (2026-11-21). The
per-item detail lives in the cited notes; this bible is the index the
merger reads FIRST.

**CORRECTION (the g4 SEAL `e5915bfab`, re-probed 2026-11-21):** the
S3 "g4 `a9369ec38` no delta" probe was STALE — g4's origin tip had
moved to `55840d9a` (09:16, before R3's 12:39 probe) and then to its
own seal `e5915bfab`. The g4 seal RE-ADJUDICATED the heal seam: the
Lead's "g4-DropHeals-unique-KEEP" premise INVERTS — **f35 ported the
SAME seam FIRST as a SUPERSET; f35 owns it, g4's duplicate is RETIRED
by g4's own hand.** The merger instruction is corrected below (§2):
merge **d22 + f35**, NEVER g4 wholesale, cherry g4's two unique hunks.
The g4-side record: `91-seal-keep-one.md` on
`origin/task/W4-g4-npc-style3-fighter` @ `e5915bfab`.

## 0. The corrected keep-one (f35 heal-seam ownership INVERTED)

Both lanes ported `NPCLoot_DropHeals` (`NPC.cs:79098-79195`) +
`NPCLoot_DropCommonLifeAndMana` (`:79531-79557`); merging both would
DOUBLE-FEED the `UpdateNPCs` death stream. The adjudication (g4 seal
§1, re-verified at this correction):

* **f35 FIRST** — `b72cdfbfc` (08:19, `npc_death_heals.rs`) +
  `00a5b27db` (08:43, the `spawn_heals` death-path wiring), both in
  f35's tip ancestry (checked); g4's copy landed at `55840d9a` 09:16.
* **f35 SUPERSET** — `npc_death_heals.rs` also carries the boss
  potions/hearts tail (`DoDeathEvents_DropBossPotionsAndHearts`,
  `:78945-79016`), the EoC/WoF Badger's Hat 5004 latch + dusk reset,
  the EoW head's dynamic boss latch; g4's copy had none of these.
  Anchored (`anchors.txt` `pub fn drop_boss_potions_and_hearts` @
  f35) + PORT_MAP **done** row + 10 witnesses (checked at `1abae9ade`).
* **The stale row** — f35's census `:39` disclaimer sits in a "Port
  status BEFORE this task" table; f35 does NOT read g4-owns, it owns
  the seam itself. No cross-lane handoff needed.
* **g4 duplicate RETIRED** — g4's seal removed `spawn_heal_drops` +
  the two helpers + the death-path call + `HealScriptRng` + the 8
  seam witnesses; tombstone at g4 `world_sim/npcs.rs:7088` (the
  `91-seal-keep-one.md` marker), anchors row repointed (checked).

**Keep-one = f35's `spawn_heals`.** The death-path seam conflict at
the d22+f35 merge resolves to f35's call AFTER `spawn_loot`.

## 1. d22 is canonical for aiStyle 3 (+ the 4/5/6 census fixes)

Merge `task/W4-d22-npc-ai-styles-3-6` — it carries the whole AI
family (the death HEAL seam is f35's, §0; the `spawn_coins` LIVE
inputs arrive by the g4 cherry hunk, §2):

| piece | landing | provenance |
| --- | --- | --- |
| the style-3 axis core (22 plain walkers, `claims()` + `fighter_core_step`) | `npc_ai_style3.rs:60`/`:94` | d22 styles pass |
| the generalised scan (the signature UNION: rng-threaded zombie entry + axis entry, ONE shared flag25 line) | `npc_ai_fighter_scan.rs:271`/`:302`/`:473` | g4 P5 fold `0cfd08ac` |
| the graveyard pre-stamp + NotDiscouraged chain term + `update_zombie` term | `world_sim/npcs.rs:4389-4393`, `npc_ai.rs:294-302`, `npc.rs:9307` | `0cfd08ac` |
| the tall-gate drain (`shift_tall_gate` + gate-388 arm) + the GoblinPeon smash (distinct mechanic) | `tr-core/src/edit.rs:828`, `world_sim/npcs.rs` (the drain; smash `:4885` cite `:60049-60056`) | `0cfd08ac`, d22 `60712154` |
| the banner-buff strength lane (4 strike lanes + the aura grant) | `player.rs:852`/`:4721`/`:7033`, `world_sim/editing.rs:7722`, `npc.rs:18621`, `projectile.rs:23019`/`:23520`/`:15032` | g4 P1 fold `4150d374` |
| the summoner phase writes (471 flight/ground/pose kb+def) | `npc.rs:10880`/`:10990`/`:11017` | g4 P2 fold `4150d374` |
| the confused arms (shooter/rifleman/engineer-zapper + the generic-walker flips) | `npc.rs:11880`/`:13324`/`:5977`, `npc_ai.rs:100`/`:192` | g4 P3/P6 folds `4150d374` |
| the P9 immunity rows (do-NOT-re-fold contract) + whip_immune | `npc_buffs.rs:230`/`:277`/`:281`/`:408`/`:533` | g4 P9 fold `e6c83bef` |
| the blood-moon census correction + the 489 normal-night witness | `npc_ai_fighter_scan.rs` note + `:803` | `0cfd08ac` (`9bed77313` correction) |
| the styles 4/5/6 census fixes | `c1d26278`/`204e0651`/`b24977d9` | d22 styles pass |
| the moan-position closure (S1-E-21e): the emitter at the PRE-SLOT head, moan-then-bash C# order | `world_sim/npcs.rs:4559-4623` | R1 `fb99a047` |
| the c8 saturation-pin fix (the seal S1): `the_worm_head_digs_and_stumbles` runs 110 ticks | `npc.rs` (the fixture loop) | seal S1 — c8 `708522fc6` verdict consumed, `research/238` §3a `7c2de9c0` |

The full per-symbol KEEP/DROP adjudication: `91-g4-deprecated-keep-drop.md`
(R3) — the AI-arm table stands FINAL (every KEEP row re-grepped at the
seal tip; the samples: `player.rs:852`, `edit.rs:828`,
`npc_ai_fighter_scan.rs:473`, `npc_ai.rs:1400`, `npc_buffs.rs:1173`,
`npc.rs:1732`/`:3102` all present, line cites unchanged). Its R3
"g4 no-delta at `a9369ec38`" probe was STALE — the post-probe g4
delta is adjudicated in §0/§2 (the correction); the R3 item-4
cross-note ("f35's lane, NOT d22's") is CONFIRMED by the g4 seal.

## 2. g4 NEVER-WHOLESALE — merge d22 + f35, cherry g4's two unique hunks

**CORRECTED** (was "merge d22 ONLY" — the S3 probe was stale). The
g4 tip re-probed at this correction: **`e5915bfab`** (the g4 seal;
delta since `a9369ec38` = `55840d9a` FINAL code + `69d7737b` FINAL
docs + `e5915bfab` the seal, all in ancestry — checked). A g4 merge
AFTER d22 would still conflict-revert the union scan signature, the
axis dispatch and the wave-3-shaped files; after the g4 seal its heal
code is GONE by g4's own hand, so nothing unique is lost by never
merging it wholesale. The corrected instruction (g4 seal §2):

1. Merge **d22** (`a1a21235` + the seal) — canonical for EVERY fighter
   AI arm (the §1 table + the R3 KEEP table stand unchanged).
2. Merge **f35** (`1abae9ade`) — owns the heal seam (§0) + the
   bag/crate/present openers; the `spawn_heals` call resolves AFTER
   `spawn_loot` on the death path.
3. **Cherry-apply g4's TWO unique hunks after d22+f35** — **SATISFIED
   IN-BRANCH by the g4d fold** (the "in flight" wiring the FC observed,
   now committed; ANALYSIS "The g4d fold"): verify by SYMBOL at the
   merged tip, do NOT re-cherry:
   * the `spawn_coins` midas/blood-moon LIVE inputs — the 2-argument
     hunk (`npc.buffs.has_buff(72)` `NPC.cs:79631` +
     `world.blood_moon` `:79660`) + the witness
     `w4_g4_spawn_coins_midas_and_blood_moon_inputs_are_live` (g4
     `npcs.rs:8116`) + its anchors row. **Re-verified: d22 tip
     `world_sim/npcs.rs:7218-7219` AND f35 tip `:6954-6955` both still
     `false, false`** — neither lane carried the wiring at the FC;
     **the g4d fold has now landed the 2-argument hunk + the witness
     on d22** (f35's copy stays `false, false` — the merge keeps d22's
     LIVE form); the hunk
     applies cleanly to either body (otherwise identical).
   * the two scan witnesses — `w4_g4_the_goblin_thief_bash_adds_one`
     (g4 `npc_ai_fighter_scan.rs:652`) +
     `w4_g4_the_skeleton_family_bash_bonus_census` (`:678`).
     **Re-verified: d22 has NO Thief/Skeleton bonus witness** (grep at
     its tip, zero hits). Both call the UNION signature
     (`npc_ai_fighter_scan.rs:271`) — they compile against d22
     unchanged. **The g4d fold has now landed both on d22** (the Thief
     fixture's door row corrected to 27 — the port's 18×38 Thief box,
     `npc/data.rs` `SetDefaults` case 27, puts the probe row at
     `num199 - 1 = 27`, not g4's zombie-box 28).
4. **DROP g4-side (re-verified against d22's tip):**
   * the scan type-bonus arms — d22 `npc_ai_fighter_scan.rs:498-513`
     carries the SAME code (`t == 27` +1 `:498`,
     `matches!(t, 31 | 294 | 295 | 296)` +6 `:500`, `t == 460` flag28
     `:512` — re-read, identical). g4's "restore" was against its own
     pre-union zombie-only scan; the d22 axis scan never lost them.
   * the GoblinPeon 26 smash — **KEEP d22's `kill_tile` form**
     (`world_sim/npcs.rs:4890`, `tr_core::edit::kill_tile` +
     `kill_tile_sound` + the drop, cite `:60049-60056` — re-read).
     g4's `editing::break_tile` rides the PICK path (the `CanKillTile`
     refusals the C# `WorldGen.KillTile` does NOT run) — d22's form is
     the faithful one. The g4 Peon-smash witness
     (`w4_g4_the_goblin_peon_smashes_the_door`) pins the `break_tile`
     behaviour: re-derive its refusal-path assertions against d22's
     form at the merge gate (written-not-run either way).
5. The g4-unique docs (`91-doc-sweep-fighter.md`,
   `91-addendum-d22r-fold-list.md`, `91-final-fighter-census.md`,
   `91-seal-keep-one.md`) stay audit trail on the g4 branch —
   cherry-pick the DOC FILES only if the Lead wants them on main.
   **The Lead/owner retires the g4 branch after the d22+f35 merge is
   verified** (the step-6 ancestor + symbol checks); this lane never
   deletes another agent's branch. The fold-list checklist: all 9
   boxes CLOSED (`91-addendum-final-receipt.md` §1).

## 3. g8/g9/g10 COMPOSE — disjoint kind sets, keep both sides

Seal re-probe of the pushed tips (2026-11-21):

| lane | phase-4 tip | seal tip | delta since phase 4 | fighter-family verdict |
| --- | --- | --- | --- | --- |
| g8 | `82c93cec` | `e1d9578a` | `a6c4f5ae` + `e1d9578a` (Harpy 48 feather/wet + RedDevil 156 trident — aiStyle-14 machine ports) | DISJOINT: `npc_ai_style14.rs`, `update_cave_bat`, the style-14 timed-shot list (`HARPY_FEATHER`/`UNHOLY_TRIDENT_HOSTILE`), 2 projectile rows, 2 anchors — NO style-3 symbol touched. The 34-kind ≤230 wiring stands; WallCreeper 164 / Nymph 196 keep their `update_skeleton` kind arms (NOT axis-claimed) |
| g9 | `5d08fdf2` | `5d08fdf2` | none | UNCHANGED: ten kind-gated rows on the same machines |
| g10 | at base | `5dee2865` | `2d040332` + `5dee2865` (the ten town pets 637/638/656/670/678-683 + TownSlimeCopper 684 on the pet-aware town machine) | DISJOINT: aiStyle-7 town lane, ids 637+ — no fighter-band row |

The shared `npc.rs` kind-match region: disjoint kind sets — keep both
sides' arms; no re-fold. The g8 machine-round deltas ADD to the compose
(the style-14 shot drain is a superset, the d22 axis untouched).

## 4. d34-d43 take the ROSTER TABLE (the E6 input)

`91-d22-roster-gap-table.md` (R2): **191 C# aiStyle-3 types / 68 ported
kinds ALL routed / 123 kind-roster gaps** — 75 ROWED at pushed tips
(d34 2, d35 10, d37 24, d38 11, d39 1, d40 15, d41 3, d42 7, g8 2),
48 open (d36's whole 159-206 band E4-in-flight, d39's 380, d40's
430-436+449, d41's salamander/desert bands, d43 at base). The two
census corrections stand: 590+591 BOTH rowed (`:16666` ||-gate); 368
TravellingMerchant is the town lane's runtime flip, NOT a roster row.
The d36x fold decisions the E6 pass must read: **163 no-entry**,
**198->199**.

The lane obligations (the table §2 + the seal's exact-hunk note
`91-fighter-walker-kind-obligation.md`):

1. **The `fighter_walker_kind` extension** (`ambient_sound.rs:603`) —
   every new moan-listed walker kind MUST join the gate or its
   UpdateNPCs stream shortfalls per tick. The exact hunk + the
   per-lane variant names: the obligation note (S2).
2. **The axis-claim route** for the plain-walker gap kinds: move onto
   `claims()` + retire the `walker_step_fighter` stand-in kind arm
   (the ladder rows exist — `npc_ai3_speeds.rs`, W40). The SHOOTER
   types (110/206/290-293/379/380/449-452/468/481/498-506) keep their
   own machines — do NOT claim them.
3. The draw-bearing head arms (120 teleport, 463 spike burst, 624
   statue transform, 109 bomb) are per-kind ports threading the
   UpdateNPCs stream + the dust pool.
4. The flag24 bash list needs no extension (431-436/591 NOT flag24 —
   they accumulate EVERY night, the 489 correction).
5. **The MartianWalker 520 shooter-block = a d41 TASK SEED** (the
   corrected remainder lane — NOT d43, NOT a generic projectile
   lane; g4 seal §3). The kind is landed (wave-3 base: `npc.rs:113`
   — re-verified) and rides `update_skeleton` (the walk IS routed),
   but the C# 520 is a SHOOTER-BLOCK type (`NPC.cs:59069` list) whose
   offence is unported. 520 sits in the **491-560 band = d41**
   (`task/W4-d41-npc-kinds-491-560`, claim `4ed22f0b3` — ancestry
   checked); d43 covers 631-697, NOT 520. d41's own precedent ports a
   kind's shooter machine AND its projectile in-lane (batch 2
   `5d036721d`: `update_salamander` + the SalamanderSpit 572
   `proj_data` row — ancestry checked). The C# work order
   (re-read at this correction): cooldown `num162 = 15` (`:59109`),
   speed `num164 = 8` (`:59253`), **`num170 = 592` / `num169 = 35`**
   (`:59442-59445`, the 592 laser), walk `num180 = 4f / num181 = 1f /
   num182 = 0.7f` (`:59735-59739`), the **400-px stop**
   (`type == 520 && Distance < 400f && CanHitLine` → `flag22 = true;
   ai[3] = 0f`, `:59763-59766`), the **aim cycle** (`localAI[2]++`, at
   `>= 6` reset + `localAI[3] = player.DirectionFrom(Top + (0, 20))
   .ToRotation()`, `:59792-59799`). The engineer's turret-spawn
   (`:59004-59038`) + the zapper's spear (`:59062-59065`) stay with
   the projectile-work lane (kinds 386/389 outside the bands'
   unported sets).

## 5. The merge gate (written-not-run witnesses)

Run at the merge (AGENTS.md verification): the P5 graveyard set, the
P9 census, the 489 normal-night pair, the R1-re-derived graveyard
end-to-end + DW-3.1 replays, the style-4/5/6 stream-parity sets, and
the seal-fixed 110-tick `the_worm_head_digs_and_stumbles` (the c8
numeric mirror verifies it: speed 5.955 / vy −2.5 / y 2165 at tick
110). The c8 `w4_c8_compose_*` carries duplicate d22's
`w4_d22_the_*` single-tick witnesses by content — keep-one at the
merge (research/238 §6). **Added by the correction:** the f35 heal
seam's 10 witnesses (incl. the 33-kind × 5-seed × both-expert
same-seed stream-equality replays), the cherry-applied g4
`spawn_coins` LIVE-inputs witness + the Thief/Skeleton scan pair (all
three landed IN-BRANCH by the g4d fold), and
the Peon-smash witness re-derived against d22's `kill_tile` form
(§2.4).

## 6. Seal receipt (the hashes) — CORRECTED

| phase | commit | scope |
| --- | --- | --- |
| DONE base | `bcd42ddd` | reconcile phases 1-4 complete |
| R1 | `fb99a047` | moan-position CLOSED |
| R2 | `28866c7c` | the 123-roster-gap table |
| R3 | `78386ab1` | g4 DEPRECATED + KEEP/DROP FINAL |
| R4 | `a1a21235` | the addendum FINAL receipt |
| S1 | `3ec231f9` | the c8 110-tick pin consumed (`npc.rs` fixture) |
| S2 | `3ec231f9` | the fighter_walker_kind exact-hunk obligation |
| S3+S4 | `3ec231f9` | the KEEP/DROP confirmation + this bible |
| FC (this correction) | `7307eaae` | the FIGHTER-CORRECTION: the stale S3 probe owned, f35-owns-heal-seam (§0), merge d22+f35 never g4-wholesale + the 2 cherry hunks (§2), the 520 d41 task seed (§4.5), this receipt |
| g4d (the cherry fold) | (this commit) | the §2.3 hunks landed IN-BRANCH: the spawn_coins LIVE inputs + witness, the Thief/Skeleton scan pair (the Thief fixture row corrected 28->27 for the 18x38 box), the scan comment's AngryBones census fix; the Peon-smash witness stays gate-deferred |

The g4-side receipt (read at `e5915bfab`): DONE base `69d7737b` →
SEAL `e5915bfab`; f35 tip `1abae9ade` (heal port `b72cdfbfc` 08:19 +
wiring `00a5b27db` 08:43, both BEFORE g4's `55840d9a` 09:16 — commit
timestamps checked); d41 tip `db12805b` at this correction (claim
`4ed22f0b3` + the `5d036721d` salamander precedent in ancestry).

Verification: code-only — the g4/g8/g9/g10 tips re-probed after
`git fetch --prune`; the KEEP samples re-grepped; the c8
`worm_head_step` diff-read (identical type-10 kinematics — only the
375 rows + comments differ); brace-balance delta 0/0 on the touched
.rs; no cargo/tests/rustfmt/graphify invoked. The correction adds:
the g4/f35/d41 tips + the heal-port ancestry re-probed; the d22
`false, false` stand-ins (`npcs.rs:7218-7219`), the f35 stand-ins
(`:6954-6955`), the scan type-bonus arms (`npc_ai_fighter_scan.rs
:498-513`), the Peon `kill_tile` form (`npcs.rs:4890`), the missing
Thief/Skeleton witnesses (zero grep hits) and the f35 anchor/PORT_MAP
rows all re-read at the named tips; the C# 520 params re-read in
≤100-line windows (`NPC.cs:59105-59112`/`:59249-59256`/`:59440-59448`/
`:59733-59740`/`:59760-59770`/`:59790-59800`); docs-only, no .rs
touched by the correction. The g4d fold adds: the C# heal seam +
`:78897-78898` order + `:79631`/`:79660` gates + `:60027-60034` bonus
arms re-read (≤100-line windows); the g4 `55840d9a` diff read hunk by
hunk; the f35 `npc_death_heals.rs` superset re-read at `1abae9ade`
(the keep-one CONFIRMED — no heal code folded into d22); brace-balance
delta 0/0 on both touched .rs.
