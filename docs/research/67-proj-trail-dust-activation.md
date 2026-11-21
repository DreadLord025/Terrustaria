# 67 — Projectile TRAIL dust activation checklist (W4-g63, for the merger)

Status: ACTIVATION CHECKLIST (W4-g63 follow-up, 2026-11-21). The g63
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
| 12 NaturalFallingStar | `natural_falling_star_sparkle` | `Projectile.cs:23970-23987` | **d15** `c5e24faba` (row + the `ai_falling_star` 12 joins) | in `ai_falling_star`, kind-12 branch at the `:23970` position (after the spin, before/with the light 0.9 write). CONFLICT NOTE: d15's arm signature is `(world)` — g63's HEAD arm is already `(world, rng, dust, dust_ctx)` (batch 1); the merge keeps the dust-threaded signature and adds d15's daylight kill + `NATURAL_FALLING_STAR` to the `matches!` + light 0.9. GORE SEAM: g16 `36e03444` wired `falling_star_trail` (the same `:23973-23981` gore, `Next(6)` + SelectRandom `Next(4)` ×1/×2) for 12/955 — the g63 emitter consumes those SAME draws; at the three-way merge keep ONE consumer: either g16's gore fn + a dust-only variant of the g63 emitter (drop its gore-gate block), or the g63 emitter + g16's gore body folded in (the 162 `frost_shard_burst`/`tombstone_landing_gore` split). The anniversary RE-PICK draws TWO `Next(4)` (`:23975-23978`, second wins) — both ports now agree. |
| 87 GreenFairy (+72/86 co-riders) | `fairy_sparkle` | `:24395-24409` | **d2** `ebf418031` (rows + `projectile_ai_light_pets::light_pet_step`) | REPLACE d2's `Next(6)` gate block (`light_pet_step`, its `:24395-24409` comment) with the `fairy_sparkle` call — the emitter owns the gate draw; keeping both double-draws. `dust_type` per kind: 72→56, 86→73, 87→74 (`:24397-24405`). Thread `dust`+`ctx` into `light_pet_step` (it already takes `rng`). The shader write `:24408` stays skipped (visual). |
| 298 SpiritHeal / 305 VampireHeal | `spirit_heal_trail` / `vampire_heal_trail` | `:28546-28558` / `:28533-28545` | **d6** `af75e6460` (rows + `ai_heal_orb`) | in `ai_heal_orb` AFTER the velocity blend (`:28531-28532`), kind-gated 305→3× / else→5×. The C# `Kill()` at `:28526` has NO return — the trail runs on the contact-kill tick too (d6 preserved the shape); call the emitter unconditionally after the kill check. No explicit draws in either loop — no census to reconcile. Thread `rng`+`dust`+`ctx` into `ai_heal_orb` (currently `(player, local_owner)`). |
| 358 WaterGun | `water_gun_stream` | `:29158-29196` | **d6** `5d81f2de0` (row + `projectile_ai_guns::gun_stream`) | REPLACE d6's 358 dust-census block (its `:29168-29194` comment: 6×3 `Next(2)` + the `Next(4)` extra, bodies skipped) with the `water_gun_stream` call — the emitter consumes the SAME 18+1 draws and spawns the bodies; keeping both double-draws. `gun_stream` already takes `rng`; add `dust`+`ctx`. The 406/1042 twins' censuses stay d6's (kinds > 400 — g64's dust). The Kill-hook 50×211 (`:74029`) is g65's. |
| 55 HornetStinger (hostile) | `hornet_stinger_trail` | `:55490-55494` | **d22** `204e0651` (row + `ai_hornet_stinger`) | in `ai_hornet_stinger` before/after the flight call (the C# dust runs inside `AI_001` before the flight tail; no state coupling — either position is stream-identical since the flight draws nothing for 55). d22's arm body-skips the dust with NO draws — nothing to reconcile. Thread `rng`+`dust`+`ctx` (currently `(world)`). |
| 299 Shadowflames | `shadowflames_dust` | `:56631-56656` | **c3** `f6b0468d` (row + slice-A `SHADOWFLAMES` arm) | in the slice-A `SHADOWFLAMES` arm, AFTER the Item8 cue and BEFORE `p.local_ai[0] += 1.0` — pass the PRE-increment counter (the emitter derives the post-increment trail gate as `local_ai0 + 1 > 6`; tick 6 fires burst AND trail, 43 dusts). c3's arm body-skips both loops with NO draws — nothing to reconcile. `ai001_slice_a_dust` already threads `rng`+`dust`+`ctx` (g63 batch 5). NOT d2's tombstones (`61c47ed7`/`f430ccf8` = kinds 43/201-205/527-531). |

## The 357 f20b seam (row already in main)

| kind | emitter fn | C# site | unlock | call site + seam |
| ---- | ---------- | ------- | ------ | ---------------- |
| 357 PulseBolt | `pulse_bolt_trail` | `:56753-56768` | **f20 slice D** `4bd2f496` (branch tip `5aa190e3`, f20c active) | the 357 row IS in main (`data.rs:3289`, 4×4 alpha 255 extraUpdates 2 scale 1.2); only the flight arm is missing. In `projectile_ai001_d`'s 357 arm: call `pulse_bolt_trail` gated `p.alpha < 170` BEFORE the fade-25 (`:56769-56771`) — the trail reads the PRE-fade alpha and copies it onto every dust (`:56762`). The 207 `chlorophyte_bullet_trail` twin (batch 5, wired) is the shape. Slice D's `step(&mut p, world)` needs a `step_dust` variant threading `rng`+`dust`+`ctx` (the batch-5 slice convention); the trail itself draws nothing explicit. f20's F7: the 357 takes NO gravity (the chain heads at its fade) — the emitter assumes nothing about flight. Cross-note for f20c: this emitter is READY for its slice-D merge; no f20-side dust work is owed beyond the one call + gate. |

## The 281 bunny (row unclaimed + machine on d5)

Activation needs THREE things (research/66's corrected census):
1. **The row** — `SetDefaults :3676-3687` (28×28, aiStyle 49, friendly,
   alpha 255, timeLeft 600, id-static cooldown 10, penetrate −1). NO
   branch carries it; unclaimed d-lane follow-up.
2. **d5's merge** (`5f7390a30`, `projectile_ai_style49.rs` UNWIRED) +
   its dispatch wiring (the `49 =>` arm).
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
- **Batch 6's remainder** — 93/118/119/128/129/177/196/229/261/291/
  307/335: no rows on any pushed tip (the 118/119/129 `proj_data`
  grep hits are the 617/618/659 false positives, research/66). Each
  emitter's doc comment carries its C# site + constants.

## Verification

Code-only round (no cargo/tests/rustfmt/graphify per the task
contract). The 50 `#[test]` witnesses in `projectile_trail_dust.rs`
are WRITTEN NOT RUN; the merge-verify wave runs
`cargo test -p tr-app projectile_trail_dust` after EACH activating
merge (the row-gated emitters' witnesses pass unconditionally — they
exercise the fns directly; the STREAM parity needs the live wiring).
Row/branch facts above were probed by direct `git grep`/`git show`
against the pushed tips (hashes cited per row).