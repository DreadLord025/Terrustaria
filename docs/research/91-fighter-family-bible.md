# 91 — FIGHTER-FAMILY BIBLE (SEALED): d22 canonical styles 3-6, g4 deprecated, g8/g9/g10 disjoint, d34-d43 roster table

W4-d22 SEAL phase (S4), branch `task/W4-d22-npc-ai-styles-3-6` (retire
tip `a1a21235`, DONE base `bcd42ddd`). Code-only lane: no cargo/tests/
rustfmt/graphify invoked. This is the single merger-facing record for
the fighter family — the union of R1-R4 (`fb99a047`/`28866c7c`/
`78386ab1`/`a1a21235`) plus the seal re-probes (2026-11-21). The
per-item detail lives in the cited notes; this bible is the index the
merger reads FIRST.

## 1. d22 is canonical for aiStyle 3 (+ the 4/5/6 census fixes)

Merge `task/W4-d22-npc-ai-styles-3-6` — it carries the whole family:

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
(R3) — **confirmed FINAL at the seal** (every KEEP row re-grepped at the
seal tip; the samples: `player.rs:852`, `edit.rs:828`,
`npc_ai_fighter_scan.rs:473`, `npc_ai.rs:1400`, `npc_buffs.rs:1173`,
`npc.rs:1732`/`:3102` all present, line cites unchanged).

## 2. g4 DEPRECATED — merge d22 ONLY, NEVER g4-after

`task/W4-g4-npc-style3-fighter` tip re-probed at the seal (2026-11-21,
`git fetch --prune` + rev-parse): **`a9369ec38`, no delta** since the
reconcile. Every g4 code hunk is folded (phases 1-4 + the retire); a
g4 merge AFTER d22 would conflict-revert the union scan signature, the
axis dispatch and the wave-3-shaped files. The g4-unique docs
(`91-doc-sweep-fighter.md`, `91-addendum-d22r-fold-list.md`) stay audit
trail on the g4 branch — cherry-pick the DOC FILES only if the Lead
wants them on main. **The Lead/owner retires the g4 branch after the
d22 merge is verified** (the step-6 ancestor + symbol checks); this
lane never deletes another agent's branch. The fold-list checklist:
all 9 boxes CLOSED (`91-addendum-final-receipt.md` §1).

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

## 5. The merge gate (written-not-run witnesses)

Run at the merge (AGENTS.md verification): the P5 graveyard set, the
P9 census, the 489 normal-night pair, the R1-re-derived graveyard
end-to-end + DW-3.1 replays, the style-4/5/6 stream-parity sets, and
the seal-fixed 110-tick `the_worm_head_digs_and_stumbles` (the c8
numeric mirror verifies it: speed 5.955 / vy −2.5 / y 2165 at tick
110). The c8 `w4_c8_compose_*` carries duplicate d22's
`w4_d22_the_*` single-tick witnesses by content — keep-one at the
merge (research/238 §6).

## 6. Seal receipt (the hashes)

| phase | commit | scope |
| --- | --- | --- |
| DONE base | `bcd42ddd` | reconcile phases 1-4 complete |
| R1 | `fb99a047` | moan-position CLOSED |
| R2 | `28866c7c` | the 123-roster-gap table |
| R3 | `78386ab1` | g4 DEPRECATED + KEEP/DROP FINAL |
| R4 | `a1a21235` | the addendum FINAL receipt |
| S1 | (this seal) | the c8 110-tick pin consumed (`npc.rs` fixture) |
| S2 | (this seal) | the fighter_walker_kind exact-hunk obligation |
| S3+S4 | (this seal) | the KEEP/DROP confirmation + this bible |

Verification: code-only — the g4/g8/g9/g10 tips re-probed after
`git fetch --prune`; the KEEP samples re-grepped; the c8
`worm_head_step` diff-read (identical type-10 kinematics — only the
375 rows + comments differ); brace-balance delta 0/0 on the touched
.rs; no cargo/tests/rustfmt/graphify invoked.
