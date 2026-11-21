# 199 — Projectile TRAIL dust activation checklist (W4-g63, for the merger)

(Number: was research/67; renumbered 67 → **199** per the d54
registry-150 §3 — the 67 key's winner is d54's `67-dust-orch-sites-claims.md`
(first-add 01:23:05 vs this doc's 07:50:44). The rename is EXECUTED on
this branch; doc-66's refs + the emitter comments updated with it.)

Status: ACTIVATION CHECKLIST (W4-g63 follow-up, 2026-11-21; REFRESHED at
the lane's RETIRE round the same day — every unlock tip re-probed, three
newly-rowed kinds promoted, the 357 seam re-verified at the moved f20
tip). The g63
emitters in `crates/tr-app/src/projectile_trail_dust.rs` are all landed
on `task/W4-g63-proj-trail-dust-1`; the UNWIRED ones wait on the merges
below. Each row names the unlocking branch + commit, the exact call
site, and the RNG-seam reconciliation the merge MUST do (a double-feed
desyncs the `UpdateProjectiles` stream — the determinism invariant).

Probe method: every pushed `origin/task/W4-*` tip's
`crates/tr-app/src/projectile/data.rs` swept for the kind's const +
literal/range match arms (direct `git grep <ref> -- <path>`, no
junction recursion). "Unrowed" below means no `proj_data` arm on ANY
pushed tip as of this round.

## Rowed-on-unmerged emitters (activate at the named merge)

| kind | emitter fn | C# site | unlock | call site + seam |
| ---- | ---------- | ------- | ------ | ---------------- |
| 12 NaturalFallingStar | `natural_falling_star_sparkle` | `Projectile.cs:23970-23987` | **d15** `c5e24faba` (row + the `ai_falling_star` 12 joins) | in `ai_falling_star`, kind-12 branch at the `:23970` position (after the spin, before/with the light 0.9 write). CONFLICT NOTE: d15's arm signature is `(world)` — g63's HEAD arm is already `(world, rng, dust, dust_ctx)` (batch 1); the merge keeps the dust-threaded signature and adds d15's daylight kill + `NATURAL_FALLING_STAR` to the `matches!` + light 0.9. GORE SEAM: g16 `36e03444` wired `falling_star_trail` (the same `:23973-23981` gore, `Next(6)` + SelectRandom `Next(4)` ×1/×2) for 12/955 — the g63 emitter consumes those SAME draws; at the three-way merge keep ONE consumer: either g16's gore fn + a dust-only variant of the g63 emitter (drop its gore-gate block), or the g63 emitter + g16's gore body folded in (the 162 `frost_shard_burst`/`tombstone_landing_gore` split). The anniversary RE-PICK draws TWO `Next(4)` (`:23975-23978`, second wins) — g63 + g16 agree (re-verified at retire against the C# + `projectile_gore_a.rs:199`, g16 tip `f1c60ffe9` unchanged there). RETIRE re-probe: d15 tip `5b7f028d3` — `ai_falling_star` byte-unchanged since `c5e24faba` (the daylight kill + light 0.9, the dust/gore draws cited-skipped); the 12's natural-starfall PRODUCER landed on d15 (`ff8d2126f`, `world_sim/growth.rs` — the 720 aiStyle-148 spawner loop, `WorldGen.cs:71828-71856`), so the merge activates a live spawn source too. THIRD-COPY FLAG: e3's `ai_falling_star` (tip `ac6392cf2`) ports the SAME 12/955 block for 955 via `projectile_dust::style5_star_dust` — its gore gate draws ONE `Next(4)` even on anniversary (the SAME re-pick error g63's batch 6 carried before `036901a5e`); at the three-way merge the surviving arm must draw BOTH (the g63/g16 form) or 955 desyncs the stream on anniversary worlds. e3's copy is 955-only (kind > 400, g64's range) — no 12-kind collision, but the shared block's draw census must reconcile to the two-draw form. SPAWN-DUST SEAL: the error re-confirmed STILL PRESENT at e3 tip `143b8cb67` (`projectile_dust.rs:597-599` + the mirror witness `:3126-3128`) — the EXACT two-draw fix-hunk + witness hunk are published in research/298 §3.2 for the e3 lane/merger. |
| 87 GreenFairy (+72/86 co-riders) | `fairy_sparkle` | `:24395-24409` | **d2** `ebf418031` (rows + `projectile_ai_light_pets::light_pet_step`) | REPLACE d2's `Next(6)` gate block (`light_pet_step`, its `:24395-24409` comment) with the `fairy_sparkle` call — the emitter owns the gate draw; keeping both double-draws. `dust_type` per kind: 72→56, 86→73, 87→74 (`:24397-24405`). Thread `dust`+`ctx` into `light_pet_step` (it already takes `rng`). The shader write `:24408` stays skipped (visual). |
| 298 SpiritHeal / 305 VampireHeal | `spirit_heal_trail` / `vampire_heal_trail` | `:28546-28558` / `:28533-28545` | **d6** `af75e6460` (rows + `ai_heal_orb`) | in `ai_heal_orb` AFTER the velocity blend (`:28531-28532`), kind-gated 305→3× / else→5×. The C# `Kill()` at `:28526` has NO return — the trail runs on the contact-kill tick too (d6 preserved the shape); call the emitter unconditionally after the kill check. No explicit draws in either loop — no census to reconcile. Thread `rng`+`dust`+`ctx` into `ai_heal_orb` (currently `(player, local_owner)`). |
| 358 WaterGun | `water_gun_stream` | `:29158-29196` | **d6** `5d81f2de0` (row + `projectile_ai_guns::gun_stream`) | REPLACE d6's 358 dust-census block (its `:29168-29194` comment: 6×3 `Next(2)` + the `Next(4)` extra, bodies skipped) with the `water_gun_stream` call — the emitter consumes the SAME 18+1 draws and spawns the bodies; keeping both double-draws. `gun_stream` already takes `rng`; add `dust`+`ctx`. The 406/1042 twins' censuses stay d6's (kinds > 400 — g64's dust; the 1042's `:29245/29284` are the e3 `style60_shimmer_gun_trail_1042` STAGED emitter's, doc-66's corrected census). KILL-HOOK KEEP-ONE (retire finding): the 50×211 (`:74029-74062`) is ported TWICE — d6 residual 8 (`1b54dcbbd`, `projectile_gun_splash.rs`, claiming g65's kind list SKIPS the guns) AND g65 batch 18 (`1bbeec5b5`, `projectile_death_dust.rs:2786` `smoke_puff` 358/406/1042 arms — g65's claim note is STALE, it did port them). Both draw the same census; at the d6+g65 merge keep ONE consumer (either body is faithful — the g65 `smoke_puff` covers all three guns in one helper; d6's carries the co-lane claim doc). One survivor, never both — a double-feed desyncs the stream. SPAWN-DUST SEAL ADJUDICATION (research/298 §3.3, exact drop-hunks published): **d6's `projectile_gun_splash` SURVIVES** — the true `old_velocity` latch + the scale-death ×5-kicked base (`d6 projectile.rs:20545`) + the corpse `p.vel = base * 0.2` write (`:20546`), vs g65's `DeathSnapshot::of` `old_vel: p.vel` self-declared stand-in and its local-only `new_vel` (no corpse write). g65's three `smoke_puff` arms (`:2786-2794` at tip `f6e07bb90`) + the `smoke_puff` helper (`:4834-4885`) + the `w4_g65_the_smoke_puffs` witness (`:6616-6634`) DROP; d6 keeps module + queue + hook + drain + all 3 witnesses. |
| 55 HornetStinger (hostile) | `hornet_stinger_trail` | `:55490-55494` | **d22** `204e0651` (row + `ai_hornet_stinger`) | in `ai_hornet_stinger` before/after the flight call (the C# dust runs inside `AI_001` before the flight tail; no state coupling — either position is stream-identical since the flight draws nothing for 55). d22's arm body-skips the dust with NO draws — nothing to reconcile. Thread `rng`+`dust`+`ctx` (currently `(world)`). RETIRE re-probe: d22 tip `a1a21235f` — the post-`204e0651` commits are the g4-reconcile NPC-side work + the banner-buff pool fields; the `ai_hornet_stinger` arm is byte-unchanged. |
| 299 Shadowflames | `shadowflames_dust` | `:56631-56656` | **c3** `f6b0468d` (row + slice-A `SHADOWFLAMES` arm) | in the slice-A `SHADOWFLAMES` arm, AFTER the Item8 cue and BEFORE `p.local_ai[0] += 1.0` — pass the PRE-increment counter (the emitter derives the post-increment trail gate as `local_ai0 + 1 > 6`; tick 6 fires burst AND trail, 43 dusts). c3's arm body-skips both loops with NO draws — nothing to reconcile. `ai001_slice_a_dust` already threads `rng`+`dust`+`ctx` (g63 batch 5). NOT d2's tombstones (`61c47ed7`/`f430ccf8` = kinds 43/201-205/527-531). RETIRE re-probe: c3 tip `9ea0ee90d` — the only post-`f6b0468d` code delta is the 299 witness's PRE-move cue-position capture (test-only); the arm shape stands. |
| 128 FrostBlastHostile | `frost_sword_trail` (the 119/128 shared emitter) | `:26620-26628` | **g8** `0a4b76e3f` (row, aiStyle 28) — NEW at retire | in g8's `ai_frost_bolt` (its `:26572-26609` "stay SKIPPED" note), kind-gated 128, BEFORE the `ai[1]` first-tick latch (the C# per-kind blocks precede it). g8's arm takes `(world)` only and draws NOTHING — thread `rng`+`dust`+`ctx` (the g63 HEAD `ai_frost_bolt` already carries the dust-threaded signature, batch 2's 309 arm — the merge keeps it and adds g8's 967 cue arm). The 119 half stays UNWIRED (no row anywhere). |
| 129 RuneBlast | `rune_blast_trail` | `:26638-26647` | **g5** `96f074959` (row + the census block) — NEW at retire | REPLACE g5's `ai_frost_bolt` 129 census block (its `:26638-26646` comment: 6×[`Next(4)` velocity + `Next(5)` scale], bodies skipped) with the `rune_blast_trail` call — the emitter consumes the SAME draws (per dust, POST-spawn: `new_dust` internals, then Next(4), then Next(5)) and spawns the bodies; keeping both double-draws (the 87 fairy precedent). g5's arm already takes `rng`; add `dust`+`ctx`. Position: before the `ai[1]` latch, the g5 block's own slot. |
| 229 ChlorophyteOrb | `chlorophyte_orb_trail` | `:27934-27939` | **g2** `ee857a368` (row + `ai_spear_subshot_44` 229 arm) — NEW at retire | in g2's `projectile_ai_spear::ai_spear_subshot_44` 229 arm at its `:27934-27939` "dust-only, skipped with the draws" comment — AFTER the age counter + gravity/friction block, BEFORE the `num321 = 6` frame-machine tail. g2 skips the dust AND its draws — the emitter re-arms both (4× [2 NextDouble point draws → NewDustPerfect internals], no post-spawn draws). Thread `rng`+`dust`+`ctx` into `ai_spear_subshot_44` (currently `(p, world)`; the 228/732 co-riders draw nothing — the added params are inert for them). |

## The 357 f20b seam (row already in main)

| kind | emitter fn | C# site | unlock | call site + seam |
| ---- | ---------- | ------- | ------ | ---------------- |
| 357 PulseBolt | `pulse_bolt_trail` | `:56753-56768` | **f20 slice D** `4bd2f496` (branch tip `31386266` at retire, f20c active) | the 357 row IS in main (`data.rs:3289`, 4×4 alpha 255 extraUpdates 2 scale 1.2); only the flight arm is missing. In `projectile_ai001_d`'s 357 arm: call `pulse_bolt_trail` gated `p.alpha < 170` BEFORE the fade-25 (`:56769-56771`) — the trail reads the PRE-fade alpha and copies it onto every dust (`:56762`). The 207 `chlorophyte_bullet_trail` twin (batch 5, wired) is the shape. Slice D's `step` needs a `step_dust` variant threading `rng`+`dust`+`ctx` (the batch-5 slice convention); the trail itself draws nothing explicit. f20's F7: the 357 takes NO gravity (the chain heads at its fade) — the emitter assumes nothing about flight. RETIRE re-verification (tip `31386266`, past the censused `5aa190e3`): the F4/F6/F7/F3/F5 commits changed `step`/`finish` to `(p, world, npcs, immune)` — the Harpy-charm `charm_tail` rides `finish` AFTER the flag8 clamp (`:57769-57793`, RNG-free by construction). The 357 `PULSE_BOLT` arm's fade-25 block + its position are UNCHANGED; the `step_dust` variant threads `npcs`+`immune` too, and the emitter call still seats BEFORE the fade-25 (C# order `:56753` trail < `:56769` fade < `:57769` charm tail — no stream collision). Cross-note for f20c: this emitter is READY for its slice-D merge; no f20-side dust work is owed beyond the one call + gate. |

## The 281 bunny (row unclaimed + machine on d5)

Activation needs THREE things (research/66's corrected census):
1. **The row** — `SetDefaults :3676-3687` (28×28, aiStyle 49, friendly,
   alpha 255, timeLeft 600, id-static cooldown 10, penetrate −1). NO
   branch carries it; unclaimed d-lane follow-up.
2. **d5's merge** (`5f7390a30`, `projectile_ai_style49.rs` UNWIRED) +
   its dispatch wiring (the `49 =>` arm). RETIRE re-probe: d5 tip
   `e46d9eb7f` — the style-49 module byte-unchanged since `5f7390a30`,
   still no `49 =>` dispatch arm on its `projectile.rs`.
3. **The one-line call**: `bunny_burst` inside d5's tick-3 block
   (`:28288-28314`). d5 consumes the gore `Next(61,64)` + the two
   NewGore kicks there (bodies skipped); the g63 emitter consumes the
   type roll and the gore lane (g16/d55) owns the kicks at merge —
   reconcile to ONE consumer (the 162 precedent). g16 `36e03444` also
   wired a 281 landing gore onto the DEAD aiStyle-2 twin (`:23355`) —
   re-point it at d5's tick-3 block or drop it (281 never reaches the
   style-2 arm). g65's Kill-hook lane overlaps d5's `bunny_kill` dust
   census (`:75759-75771`) — same one-consumer rule.

## Unrowed everywhere (no merge unlocks them yet)

- **174 IceSpike / 176 JungleSpike** (`ice_spike_dust` /
  `jungle_spike_dust`, batch 7): rows `:2602-2611` / `:2621-2629`
  (6×6, aiStyle 1, hostile, alpha 255, penetrate −1; 174 coldDamage).
  EXTRA WIRING DEBT: both read `base.oldPosition` — the port's
  `old_position` latch (`projectile.rs:20308`, C# `:17013`) is
  golf-gated (styles 149/150); the activating merge must widen the
  latch to these kinds (the C# latches EVERY projectile between AI
  and HandleMovement; the dust reads LAST tick's value). The
  `alpha == 0` window opens after the 5-step −50 fade (tick 7+).
- **374 HornetStinger minion / 376 ImpFireball**
  (`hornet_minion_stinger_dust` / `imp_fireball_dust`, batch 7): rows
  `:4576-4586` (10×10, penetrate 1, scale ×0.9) / `:4600-4614` (12×12,
  penetrate 5, timeLeft 100, alpha 255, extraUpdates 1, id-static 6).
  The 376 emitter takes the POST-increment counter; its
  `wet && !lavaWet` Kill (`:55580`) is the machine's.
- **Batch 6's remainder** — 93/118/119/177/196/261/291/307/335: no rows
  on any pushed tip AT RETIRE (the all-tip literal `NNN => (` sweep +
  the const sweep; the 118/119 `proj_data` grep hits are the 617/618
  NEBULA false positives, research/66). The 128/129/229 PROMOTED to
  activation rows above (g8/g5/g2 rows landed after the batch-6
  census). Each emitter's doc comment carries its C# site + constants.

## Verification

Code-only round (no cargo/tests/rustfmt/graphify per the task
contract). The 50 `#[test]` witnesses in `projectile_trail_dust.rs`
are WRITTEN NOT RUN; the merge-verify wave runs
`cargo test -p tr-app projectile_trail_dust` after EACH activating
merge (the row-gated emitters' witnesses pass unconditionally — they
exercise the fns directly; the STREAM parity needs the live wiring).
Row/branch facts above were probed by direct `git grep`/`git show`
against the pushed tips (hashes cited per row).

RETIRE round (2026-11-21): every unlock tip re-probed
(`merge-base --is-ancestor` + the post-cited-commit diffs read): d15
`5b7f028d3` / d2 `f430ccf83` / d6 `7edfcd495` / d22 `a1a21235f` / c3
`9ea0ee90d` / d5 `e46d9eb7f` / f20 `31386266` / g8 `058453096` / g5
`96f074959` / g2 `81c4ab85a` / g16 `f1c60ffe9` / g65 `b0842907c` / e3
`ac6392cf2` / g64 `9d9d89ac1`. The 87/298/305 unlock arms
(`light_pet_step` / `ai_heal_orb`) verified unchanged at their tips;
d6's post-`af75e6460` heal-orb deltas are the SPAWN-source
`pending_heal_orbs` queue (residual 3), not the trail seam. The
family declaration + the lane-close record: **research/298** (was
240; renumbered per registry-150 ROUND-3-FINAL §7.3 — g58 wins 240,
the unpushed round-2 281 draft superseded; the exact
e3 two-draw fix-hunk + the gun Kill keep-one drop-hunks are
published in its §3.2/§3.3). SPAWN-DUST SEAL re-probe: the
128/129/229 rows CURRENT at g8 `1752c70df` / g5 `96f074959` / g2
`81c4ab85a` (ancestry + row/arm greps); the e3 955 ONE-draw error
still present at e3 tip `143b8cb67`; the gun Kill keep-one
ADJUDICATED — d6's `projectile_gun_splash` SURVIVES (the true
`old_velocity` latch + the scale-death ×5 base + the corpse
`p.vel = base * 0.2` write, `d6 projectile.rs:20545-20546`), g65's
`smoke_puff` arms/helper/witness DROP (g65's `DeathSnapshot::of`
`old_vel: p.vel` is a self-declared stand-in — wrong for moving
guns; exact hunks in research/298 §3.3).