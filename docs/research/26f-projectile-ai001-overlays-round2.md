# 26f — AI_001 OVERLAYS round 2: the UNROWED style-1 kinds now rowed + the riders census (W4-f20) — SEALED

> Researched 2026-11-21 against `4bd2f496` (slice D landed on this branch;
> main == `a725acb0`). C# cites are `decompiled/Terraria/Projectile.cs`
> (stable); Rust cites are `crates/tr-app/src/projectile.rs` /
> `projectile/data.rs` at this revision. Round 1 (`26e`, `fa47d5ba`)
> censused the ROWED style-1 kinds and ported the seven missing overlays
> as slice D (`4bd2f496`). This round covers the OTHER direction: the
> kinds i13/i14/i17 left UNROWED whose rows NOW exist on the pushed
> round-b branches, plus the AI_001 RIDERS census (the `:57798`-boundary
> tail — extraUpdates / velocity-cap / wet / lava / shimmer per-type
> arms) vs the landed `ai_arrow`.
>
> RETIRE round (2026-11-21, base `31386266`): F5 refreshed to the
> d53 ACTIVATED receipt, F8 added (the F7 signature-ripple all-callers
> census + the d16 merge fix-list + the adapter contract), the f19
> base-census closure table added (all five findings CLOSED on their
> claiming branches), and the verify-family SEAL declared. Sibling
> tips re-probed: d53 `88094c4c`, g2 `81c4ab85`, d16 `41080e15`,
> d17 `f6e5d7d3`, d18 `8248a54b`, d10 `e1519df7`, d4 `f1480dc0`,
> d14 `eaa05ae9`, d3 `5ff9fe98`, a9 `8c4fded8`, f19 `ec5dec82`.

## Method

- Row census: every pushed `origin/task/W4-*` branch's `proj_data` was
  parsed mechanically (the match-arm tuples, incl. `|` lists, ranges and
  the `pub use … as …` aliases — `SANTA_MISSILE` = `MISSILE` = 350) and
  diffed against main's rows for NEW-or-CHANGED aiStyle-1 kinds. The
  local-only branches (d14 etc.) were scanned too: zero new style-1 rows.
- Claim cross-check: the prompt's "g2b/d1b/d7b/d12b/d15b/d18b rowless-row
  rounds" map to the PUSHED round-b work on the g2/d1/d7/d12/d15/d18
  lanes. Verified what each actually rowed:
  - **g2** (`8ac7d95e`, `26d-projectile-rowless-kinds-w4-g2.md`): the
    nine rowless kinds 229/243/244/245/264/290/291/292/433 — styles
    44/45/48/50, **NONE aiStyle 1**. No overlay owed on this axis.
  - **d1** (`e271b5bc`): the aiStyle-6 powder family 10/11/463/1019/1051
    — style 6, not 1.
  - **d7** (`8ceb9c12`): 379 BabySpider (63) / 404 Flairon head (69) as
    UNWIRED modules with module consts, explicitly NOT rows — not 1.
  - **d12** (`9152b259`): 586/590/836/602/607/612/624/953/1044 — styles
    111/112/114/116/117, not 1.
  - **d15** (`1c373008`): the DD2 decade 698-720 + **709 (aiStyle 1)** +
    12 FallingStar (5). The 709 is the one style-1 row.
  - **d18** (`d7d631e9`): 871/883/887/893/908/916/917/969/918 — styles
    172-178, not 1.
  - The scan also caught two kinds OUTSIDE the prompt's list: **435**
    (c11 `a9b22f05`, the MartianTurretBolt row) and **576** (d11, the
    NebulaLaser row) — both aiStyle 1, both overlay-less.
- AI_001 per-type scan: the 184 `type ==` lines over `:54002-57794`
  (round 1's scan, re-run), plus the `!=` exclusion arms and the
  `:57769-57793` Harpy tail past the `:57794` method close.
- Riders census: every extraUpdates / MaxUpdates / velocity-cap / wet /
  lava / shimmer / direction write in `AI_001` (`:54002-57794`) and the
  `Update` head/tail that feeds it (`:16786-16790`, `:17009-17035`,
  `:22361-22530` Shimmer), matched against the landed `ai_arrow` /
  `ai_style1_flight` / `ai_style1_rider` and the slice gates.

## Verdict — the UNROWED-now-rowed axis

**Exactly THREE aiStyle-1 kinds gained rows on the pushed branches whose
per-type AI_001 overlays are still missing: 435, 576, 709.** All three
fell through to the generic `ai_arrow` on their row branches (verified:
no dispatch arm, no ai001 gate mentions the kind on c11/d11/d15). This
lane ports them as `projectile_ai001_e.rs` (slice E), gated after slice
D, kind-disjoint from every existing gate.

| kind | name | rowed by | C# overlay blocks (state-bearing) | ported by |
| --- | --- | --- | --- | --- |
| 435 | MartianTurretBolt | c11 `a9b22f05` (`:5056-5063`) | `:54909-54933` (the ai[1] Item12 latch + fade-40 + spriteDirection + the 3-tick 4-frame machine + light) + `:57642-57648` (the direction-split ToRotation tail) | slice E |
| 576 | NebulaLaser | d11 (`:6258-6269`) | `:55361-55365` (the ai[1] Item12 latch) + `:55635-55649` (the localAI[1]>2 fade-15 alpha switch) + `:56296-56308` (light + the 5-dust-242 trail, skipped) | slice E |
| 709 | MonkStaffT3AltShot | d15 `79576fbf` (`:7508-7520`) | `:54696-54725` (fade-30 + spriteDirection + the 3-tick 3-frame machine + light + the tracked-sound fold + the ai[1] age 30-tick Kill) + `:57642-57648` (the direction-split ToRotation tail) | slice E |

All three are on the `flag3 = false` skip list (435 `:55992`, 576
`:56008`, 709 `:56025`) — the flight counter rests at 0, none is named in
any gravity-chain arm, so the generic `ai[0] >= 15f` tail (`:57568`) is
unreachable: every slice-E kind flies exactly straight (the slice-D
STAKE argument). 435/709 take the `:57642-57648` rotation arm
(`ToRotation()`, +π when `direction == -1` — the 459/682 siblings' arm);
576 takes the DEFAULT `:57750` (`Atan2(vy, vx) + 1.57f`). The 709's
`ai[1]` is its AGE counter (the d15 spawn carries `ai1: None`): the
`ai[1]++` then `>= 30f` → `Kill(); return` fires BEFORE the flight tail,
exactly as the C# orders.

**ROW-GATED INERTNESS (the d7 unwired precedent).** The rows live on
c11/d11/d15, NOT on this branch — `proj_data` returns `None` for 435/576/
709 here, so the dispatch's `else if let Some(data)` guard never routes
them until the row branches merge. The slice-E gate is kind-disjoint by
construction; when the rows land the arm activates with NO further edit
and NO merge-time dedupe owed (the census verified no other branch ports
these overlays). The witnesses hand-build the row slice (they never call
`proj_data`), so they run on this branch as-is.

RNG: NONE of the three ported state paths draws `Main.rand` — the only
dust is the 576's 5-dust-242 trail (`:56299-56307`, the plain `NewDust`
overload), SKIPPED with its internal draws (the ai001_a/b/c/d
convention). The 435/576 `SoundID.Item12` cues ride `wants_sound` (the
S1-E-21h drain, no draws); the 709's `PlayTrackedSound`
(`DD2_SkyDragonsFuryShot`, `:54717`) folds to the inert no-cue latch (the
pool has no tracked-sound layer, S1-E-23 — the d15 `projectile_ai_dd2`
ghast/twister precedent). The module takes no rng parameter, so the
`UpdateProjectiles` stream (`Main.cs:17775-17777`) is untouched BY
CONSTRUCTION for every kind. The lights (`:54932`, `:54714`, `:56298`)
are SKIPPED (the `proj_lights` layer owns the per-kind light census).

## The RIDERS census (the `:57798`-boundary tail vs `ai_arrow`)

The landed generic flight is `ai_style1_flight` (`projectile.rs:6731`):
the `flag3` counter, the `ai[0] >= 15` → `+0.1` gravity, the ±16 clamp,
the `atan2 + 1.57` rotation, the kill-on-tile move. `ai_arrow`
(`:5268`) calls it with `count_flight = true`; `ai_style1_rider`
(`:6768`) wraps it for 20/88/355/660/110. The per-type rider mechanics in
the C# tail, matched against the port:

| rider | C# | kinds | port status |
| --- | --- | --- | --- |
| extraUpdates remap | `:54025` (`extraUpdates = (int)ai[0]`) | 88 | PORTED `ai_style1_rider` (`:6775`) — the port adds a `.min(15.0)` the C# lacks. F4 census CORRECTION: the remap DOES fire — the Laser Rifle shoot path seeds `ai[0] = Next(3,6)` (`Player.cs:49381`, editing.rs), so the row's 4 remaps to 3..5 on the first AI tick; the flight counter rests (`flag3=false` `:55957`) so the seed never grows. The cap is inert across the whole reachable range (3..5 < 15; no ai[0]≥16 writer exists) — kept as the u32-cast guard, finalized |
| extraUpdates row | SetDefaults | 576 (2), 709 (1), 357 (2), 485 (1) | the rows carry it; the sub-step loop (`:16597`) reads `extra_updates` — slice D/E faithful |
| MaxUpdates kill | `:56159` (43×), `:56178` (23×) | 594 (unrowed), 622 | 622 PORTED slice B (`:316`, `23 * (1 + extra_updates)`); 594 unrowed, deferred |
| MaxUpdates frame | `:54471`/`:54480` (4×/3×), `:56990`/`:56995` (3×) | 634/635, 645 | PORTED slice C (`:239`, `:392`, `:400`) |
| MaxUpdates gravity | `:57544` (`timeLeft <= MaxUpdates*45 - 14`) | 639 | PORTED moonlord (`:197`) |
| velocity-cap 12 | `:57561-57564` | 710 | PORTED slice D (`Gravity::Betsy`) |
| velocity-cap 32 | `:57537-57540` | 686, 711 (unrowed) | 686 PORTED slice B (`:417`); 711 unrowed, deferred |
| velocity-cap ±16 | `:57760-57767` (flag8) | all but 577 | PORTED `ai_style1_flight` (`:6738`) — the port clamps BOTH bounds; the C# −16 is flag4-only (1006), the +16 universal. Slice B/D/E keep the C# +16-only shape; the generic keeps ±16 (recorded, the slice-B doc note) |
| flag4 inverted gravity | `:56034-56036`, `:57571-57573` | 1006 | PORTED special_arrows (`Gravity::Generic { inverted: true }`, `:140`) + the −16 clamp (`:239`) |
| wet kill (bee) | `:54012-54015` (`wet && !honeyWet && !shimmerWet`) | 469 | PORTED special_arrows `bee_water_kill` (`:281`, the dispatch gate `:16886`) |
| wet kill (phoenix) | `:55769-55773` | 706 (unrowed) | deferred with the row |
| wet/lava kill (spark) | `:55580-55584` (`wet && !lavaWet`) | 376 (unrowed) | deferred with the row |
| wet kill (betsy nested) | `:55409-55412` | 710 | UNREACHABLE — the row sets ignoreWater (`:7529`), base.wet stays false (`:16790`); slice D documented skip |
| wet clear | `:16786-16789` (`242/302/638 → wet = false`) | 242, 302 (unrowed), 638 | NOT ported pool-wide (the wet probe is per-arm, the 16-style bomb stand-in `:17276`); the 242/638 arms read no wet state, so inert — recorded |
| shimmer lift | `:22361-22530` (`Shimmer()`, the `:17009-17011` gate) | the aiStyle-1/16/40 family `:22403` + the arrow list `:22380` | PORTED g2 (`63a6db26`, STAGED) + ACTIVATED d53 (`804ad5715`, the caller-gate wired after `wet_block` — F5 retire-round receipt); keep-one at merge (byte-identical bar visibility/STAGED/−10-typo-fix). The 1006 flag4 inversion is the AI_001-internal half, ported here (special_arrows) |
| magicQuiver extraUpdates | `:17032-17035` (`extraUpdates < 1 → 1`) | the arrow-flag kinds | PORTED (this round, F6): the pool's per-sub-step tail between the movement and `damage_pve` — `friendly && arrow && extra_updates < 1 && owner == local_owner && player.magic_quiver` → `extra_updates = 1`; the `numUpdates` read-once (`:16675`) means the bump lands from the NEXT update. The ×1.1 shoot-speed fuse stays the `ammo.rs` half (f28/e16) |
| Harpy-charm tail | `:57769-57793` + `ApplyHarpyCharm` `:57796-57824` + `AI_001_Features` `:239-289` | the arrow kinds (ai[2] bits) | PORTED (this round, F7): the producer is the per-tick `Player.Update_HarpyCharm` accessory scan (`Player.cs:24565` → `:28436-28490`), NOT item-use — `projectile_harpy_charm.rs` ports the producer + `ApplyHarpyCharm` + the tail; the `accHarpyCharm`/`accSeraphNecklace`/`hasPhoenixQuiver` flags + the 90/72 cooldown land in `player.rs` (the equip grants `:14490-14493`/`:14588-14598`, the ResetEffects clears `:18471-18472`/`:18609`); the tail rides every aiStyle-1 flight body's end (after the clamp, before the movement) |
| PhoenixQuivered dust | `:54005-54011` | the charmed arrows | SKIPPED with the dust layer (the ai001 convention) — the bit-2 write is state-only; the `Next(3)` + `NextVector2Circular(3,3)` pair + `NextFloat` scale draws ride with the skipped body |

**The generic `ai_arrow` is CORRECT for every rowed style-1 kind that
falls through to it.** After slices A-E + the riders + the special
arrows/bullets + the frostmoon/moonlord/darts/javelin/North-Pole/Book-of-
Skulls gates, the only rowed style-1 kinds still on `ai_arrow` are
**1/2 (the plain arrows), 178 (CannonBomb) and 687 (BetsyBreath)** —
none has a per-type AI_001 block (the `:54002-57794` scan finds zero
`type == 1/2/178/687` mentions), so the generic flight IS their C#
behaviour. (687's row is the f19 F5 / 26e F5 aiStyle-136 deviation — d14
owes the row fix, not an overlay; 178 is the cannon's bomb, a plain
arrow-flight kind.) The 350 SantaMissile is gated by frostmoon (the
`SANTA_MISSILE` = `MISSILE` alias resolved it — round 1's parser missed
the alias, this round's resolves it).

## Findings

- **F1 (the three-kind gap).** 435/576/709 are the ONLY aiStyle-1 kinds
  whose rows landed on the pushed round-b branches without their
  overlays. The prompt's g2b/d1b/d7b/d12b/d18b rounds rowed NO style-1
  kinds (g2: 44/45/48/50; d1: 6; d7: unwired 63/69; d12: 111-117; d18:
  172-178) — only d15's 709 plus the off-list c11 435 / d11 576. Slice E
  closes all three.
- **F2 (the row-gated inertness).** The slice-E arms are INERT on this
  branch (the rows live on c11/d11/d15) and activate at those merges with
  no dedupe owed. This is the d7 unwired-module precedent applied to the
  overlay axis: the gate is kind-disjoint, the witnesses hand-build the
  rows, so the port is merge-safe and verifiable now.
- **F3 (the 709 Kill-hook half — PORTED this round, the 709-only half).**
  The 709's `Kill()` (`:54722`) triggers the death hook `:73338-73389` —
  the 459/709 shared dust/gore burst PLUS the 709-only `:73372-73388`
  half (the owner-gated 1110 SkyDragonsFuryElectrosphere circle spawn +
  the 80-px sibling pull, with `timeLeft = 30 * Next(2,6)` and the
  tracked DD2_SkyDragonsFuryCircle cue). The g1 Kill-hook census
  (`26d-projectile-kill-hooks-census.md`) explicitly DEFERRED this half
  "with its AI arm" (1110 unrowed); the verify round re-read the g1 tip
  (`origin/task/W4-g1-proj-ai-missing-a` @ `5bd68ce2`) and found it
  census-only — the half is still deferred there — so THIS lane ports
  the 709-ONLY half + the 1110 row/arm together (the F3 "row+arm
  together" pattern). Landed: the 1110 row (`data.rs`, the verbatim
  `SetDefaults` slice `:10340-10352` — 80×80 aiStyle 77 melee
  ignoreWater alpha 255 penetrate −1 + the `NewProjectileSetup` 300-tick
  seed `:11065-11068` + the 4-frame sheet `Main.cs:8559`); the row
  activates the EXISTING aiStyle-77 `sphere_pulse` arm (the 443 sibling)
  — no new AI arm owed. The hook (`projectile.rs`
  `sky_dragons_fury_kill`, seated in the death-hook chain before the
  gem/442 arms — kind-disjoint, order unobservable): the owner gate
  (`:73372`), the 80×80 box sibling pull-in (`ai[1] = 1` + the
  centre-minus-circle-centre / 5 velocity + netUpdate, `:73374-73383`),
  the corpse-flag-lift spawn at the centre with the shot's damage / 0
  knockback (`:73384`), and `timeLeft = 30 * Next(2,6)` (`:73386`) — the
  ONE explicit draw, riding the pool's rng AFTER the spawn (the C#
  order), which is why it lives in `projectile.rs`, not the RNG-free
  slice-E module. The tracked cue (`:73387`) folds inert (S1-E-23, the
  d15 dd2 precedent); `localAI[0] = 0` folds the sound-volume slot read
  (`:30673-30686`, the sound-layer skip). Riders widened for the new
  kind: the friendly-refusal gate (`443 | 1110`, the aiStyle-77 pull-in
  latch) and the immune = 8 arm (`:14008-14011`) in BOTH damage walks.
  The 459/709 SHARED burst (`:73338-73371`: the scale≥1 resize + Damage
  + the dust/gore rings) stays the g1 lane's claim — its `26d` lists
  459 among the six functional hooks. The 709 row itself lives on d15
  (row-gated inert here — the hook fires only for a 709 corpse, and no
  709 spawns until d15 merges); the witnesses hand-build the 709 slot.
  4 witnesses (`w4_f20_f3_*` in `s1_p_i22_magic_guns_tomes_tests`),
  WRITTEN NOT RUN.
- **F4 (the 88 extraUpdates cap — FINALIZED, inert).** The port's
  `ai_style1_rider` adds `.min(15.0)` to the 88's `extraUpdates =
  (int)ai[0]` remap (`:6775`) that the C# (`:54025`) lacks. Census
  CORRECTION to this note's first pass: the remap DOES fire — the Laser
  Rifle's shoot path seeds `ai[0] = Main.rand.Next(3, 6)`
  (`Player.cs:49381`, the port's `editing.rs` ai0 arm), so the bolt's
  row extraUpdates 4 remaps to 3..5 on its first AI tick (the flight
  counter rests — 88 is `flag3 = false` `:55957` — so the seed never
  grows and the remap is stable). The cap is nonetheless INERT for
  every reachable input: the producer range is 3..5 (all < 15), and no
  writer can push ai[0] ≥ 16 (the hostile 88 spawns ride ai0 0; the
  net-wire payload has no such producer). Not behaviour-visible → the
  cap stays as the u32-cast guard (a negative/huge float would wrap the
  bare `as u32`), the rider comment corrected, 2 witnesses added
  (`w4_f20_f4_the_seeded_purple_laser_remaps_extra_updates_to_the_seed`,
  `w4_f20_f4_the_cap_is_inert_across_the_reachable_seed_range`), WRITTEN
  NOT RUN. No behaviour change.
- **F5 (the projectile shimmer layer — ACTIVATED on d53, retire-round
  receipt).** The `Shimmer()` family (`:22361-22530`, the `:17009-17011`
  shimmerWet gate) had no projectile-side port at the census pass — the
  AI_001-internal 1006 flag4 inversion is ported (special_arrows), but
  the wet-shimmer bounce/lift/kill is the f47/e19 item-side lane's
  sibling, deferred for projectiles (ANALYSIS `:27666`). The verify
  round re-read `origin/task/W4-g2-proj-ai-missing-b` (tip `81c4ab85`)
  and recorded g2's phase-3 (`63a6db26`) VERBATIM port (all 13 arms,
  `:22361-22516`, RNG-neutral, 5 witnesses) as STAGED/UNWIRED pending
  the generic wet block. RETIRE-round re-read of
  `origin/task/W4-d53-dust-sites-projectile` (tip `88094c4c`): the
  staged-unwired state is SUPERSEDED — d53 batch 4 (`fa7b331d3`) landed
  the generic wet machine (`projectile_wet::wet_block`, `:16790-17007`,
  the authoritative `shimmer_wet` writer g2 waited for), and d53 FINAL
  item 2 (`804ad5715`) ACTIVATED the helper: `Projectile::shimmer` +
  `shimmer_two_cell_kill` adopted from g2's staged copy, the `:17011`
  caller-gate WIRED in the dispatch right after `wet_block`
  (d53-tree `projectile.rs:20179-20196`: `if self.slots[i].shimmer_wet
  { self.slots[i].shimmer(world); }`). The gate sits OUTSIDE the
  ignoreWater block (the C# `:17009` reads the flag, not the probe) and
  runs on a wet-block-killed slot too (no active guard — verbatim; a
  Shimmer Kill rides the slot's own kill latch). The g2 comment's "-8"
  clamp typo is corrected to the C# `:22511-22514` −10 (re-read this
  round: `velocity.Y = -10f`; d3's `shimmer_step` twin matches).
  `CHLOROPHYTE_ORB` const added row-gated inert (g2's 229 row unmerged
  on d53 — the kind arm activates at that merge). The rowed arrow kinds
  riding the `:22380`/`:22403` shimmer arms NOW take the shimmer
  interaction once d53 merges (the wet machine feeds the flag every
  sub-step; the bobber `probe_wet` pre-AI read-lag feed stays the i35
  deviation). KEEP-ONE at merge (d53's recorded contract, re-verified
  this round by a line-diff of the two bodies): g2's staged copy is
  byte-identical bar three deltas — `fn` → `pub(crate) fn` visibility,
  the two `#[allow(dead_code)]` STAGED markers (drop with the wiring),
  and the −8→−10 comment fix; the merge keeps ONE `shimmer` +
  `shimmer_two_cell_kill` pair (d53's corrected copy) and d53's dispatch
  gate. No `W4-g2c`/`d53g` branch exists (the `git branch -a` probe this
  round); `W4-g3-proj-ai-missing-c` carries only the flail-side
  `shimmer_wet` halves. Nothing owed by this lane — the port + the
  activation both landed on the claiming lanes; cross-note only, no
  code change here.
- **F6 (the magicQuiver Update-tail bump — PORTED this round).**
  `:17032-17035` bumps `extraUpdates` to 1 for the friendly arrow-flag
  kinds when the owner has magicQuiver. The `magic_quiver` flag was
  already ported (`player.rs:2008`, the `grant_ammo_mastery` seam; the
  `ammo.rs` ×1.1 shoot-speed fuse, f28/e16) — the missing half was the
  Update-tail sub-step bump, now landed in the pool's per-sub-step tail
  (`projectile.rs`, between the AI/movement and `damage_pve`, the C#'s
  `:17031`-after / `:17078`-before slot). The C# reads `numUpdates` once
  at `:16675`, so the bump takes effect from the NEXT update — the
  witness pins tick 1 = one timeLeft, tick 2 = two. The `!npcProj` half
  folds into `friendly` (the port's hostile proxy); the owner read folds
  to `owner == local_owner` + the local player's flag. RNG-free (the
  bump draws nothing). 5 witnesses in `w4_f20_f6_quiver_tests`, WRITTEN
  NOT RUN.
- **F7 (the Harpy-charm chain — PORTED this round, producer + tail).**
  The `:57769-57793` tail + `ApplyHarpyCharm` (`:57796-57824`) +
  `AI_001_Features` (ai[2] bits, `:239-290`) were recorded inert-unported
  in the first pass ("the ai[2] bits stay 0"). This round censused the
  WRITER: the producer is NOT item-use — it is the per-tick
  `Player.Update_HarpyCharm` accessory scan (`Player.cs:24565` →
  `:28436-28490`), unclaimed-small, so the whole chain ports together
  (the F3 "row+arm together" pattern applied to producer+tail).
  `projectile_harpy_charm.rs` carries: the `AI_001_Features` bit consts
  (the FloatIntUnion's low four mantissa bits), `update_harpy_charm`
  (the cooldown decrement + the effect gate), `harpy_charm_effect` (the
  1000-slot nearest-arrow scan: 400..1200 px window, 1600-px LOS target,
  the 90/72-tick cooldown, the Phoenix>Seraph>plain itemID pick),
  `apply_harpy_charm` (the π turn, the bit writes, the 300-tick life cap,
  the SendData→net_update fold, the particle-layer skip) and `charm_tail`
  (the per-sub-step π steer + the <1 normalize + the <12 ×1.05 accel, the
  Consumed-bit on a lost target). The player flags
  (`acc_harpy_charm`/`acc_seraph_necklace`/`has_phoenix_quiver` + the
  persistent cooldown) land in `player.rs` with the equip grants
  (`:14490-14493`/`:14588-14598`) and the ResetEffects clears
  (`:18471-18472`/`:18609`). The tail rides the END of every aiStyle-1
  flight body (the generic `ai_style1_flight`, the slice-A/D/E `finish`,
  `special_arrows::step`, the moonlord `finish_flight`/640 arm, the darts
  step) — after the `flag8` clamp, before the movement, the C#'s `:57769`
  slot; the early-return kill paths skip it as the C# `return` does. The
  producer rides `update_player` after `update_snake_band` (the C#
  `:24565` < `:24567` order, the split-frame fold). RNG-free by
  construction (the scan + the trig draw nothing); the PhoenixQuivered
  dust gate (`:54005-54011`) stays the dust-layer skip. 6 witnesses in
  `projectile_harpy_charm::tests`, WRITTEN NOT RUN.
- **F8 (the F7 signature-widening ripple — retire-round all-callers
  census + the merge-friction contract).** F7 (`23c266c47`) widened
  EVERY aiStyle-1 flight-body signature to feed `charm_tail`'s
  `npcs`/`immune`. The retire round censused every caller on this
  branch AND on the three sibling aiStyle lanes' pushed tips.
  **On this branch: all callers updated, zero stale.** The widened
  set (old → new): `Projectile::ai_style1_flight(world, count_flight)`
  → `(world, count_flight, npcs, immune)` (`projectile.rs:6735`, the
  tail at `:6751`); `ai_arrow(world)` → `(world, npcs, immune)`
  (`:5268`, forwards at `:5269`); `ai_style1_rider(world)` →
  `(world, npcs, immune)` (`:6780`, forwards at `:6870`);
  `ai_book_of_skulls`'s flight call passes `npcs, &[]` (`:7322` — the
  skull is not an arrow, the producer never charms it, the empty
  immune slice keeps the tail a no-op); `ai001_slice_a(p, world,
  owner)` → `+npcs, immune` (`projectile_ai001_a.rs:125`, the tail in
  `finish` `:428`); `projectile_ai001_d::step(p, world)` → `+npcs,
  immune` (`:103`, tail `:281`); `projectile_ai001_e::step` likewise
  (`:96`, tail `:256`); `special_arrows::step(p, world)` → `+npcs,
  immune` (`:250` tail); `projectile_ai_darts::step(p, world, npcs,
  rng, local_owner)` → `+immune` (tail `:170`);
  `projectile_ai_moonlord::step(p, world)` → `+npcs, immune` (tails
  `:117` finish_flight + `:235` the 640 arm). The dispatch feeds every
  gate `npcs` + `&self.local_immunity[i*256..(i+1)*256]`
  (`projectile.rs:16806-17020`); every witness call-site in the six
  modules + `projectile.rs:23799-23822` passes `&[], &[]` (or the
  fixture's slices) — a mechanical `ai_style1_flight(|ai_arrow(|
  ai_style1_rider(|charm_tail(|ai001_slice_a(|…::step(` scan over
  `crates/tr-app/src` finds no old-arity call left. Helpers promoted
  `pub(crate)`: `rotate_towards` / `safe_normalize_or`; the pool gained
  `local_immunity_slice`.
  **The sibling lanes (d16/d17/d18 pushed tips `41080e15`/`f6e5d7d3`/
  `8248a54b`; d16 re-scanned after its in-flight advance from
  `d414afbc` — the kite residuals 7/8/9 add no style-1 callers either):
  ZERO new aiStyle-1 callers.** A full-tree scan of each branch (every
  `.rs` under `crates/tr-app/src`) finds their new arms are styles
  151-155/159 (d16), 161-163/166/168 (d17), 172/174-178 (d18) — none
  calls `ai_style1_flight`/`ai_arrow`/`ai_style1_rider`/`charm_tail`/
  `ai001_slice_a`/`special_arrows::step`/`darts::step`/
  `moonlord::step` outside `projectile.rs`'s own (untouched-by-them)
  bodies. Their trees still carry the OLD signatures (main
  `a725acb0` base) and compile against them TODAY; the friction is
  merge-time only, and it is ONE file: **d16 `4d870ea1c` (residual 6,
  the TrailCacheLength ring fix) edits `projectile_ai_moonlord.rs`** —
  the same file F7 widened. FIRST-COMPILE-FIX-LIST for the d16×f20
  merge (whichever order): (1) d16's 11 two-arg `step(&mut p, &w)`
  witness calls in `projectile_ai_moonlord.rs` tests must take f20's
  four-arg shape `step(&mut p, &w, &[], &[])` (f20 already converted
  the same call-sites — keep f20's arity, d16's test bodies);
  (2) d16's settled-scan `p.old_pos[1..p.trail_len]` (`:55441` fix)
  lands INSIDE f20's widened `step` body — keep BOTH (the scan fix +
  the widened signature/tails); f20's tree has no `trail_len` field,
  so d16's `Projectile.trail_len`/`trail_cache_length`/
  `TRAIL_RING_CAP`/`reset_trailing_rings` additions in `projectile.rs`
  must survive the merge (f20's `projectile.rs` hunks touch the
  struct's neighbourhood — the charm producer feeds + `local_immunity_slice`);
  (3) d16's moonlord witness `p.old_pos[..p.trail_len].fill(...)` vs
  f20's `p.old_pos = [...; 10]` — keep d16's trail_len-aware fill.
  d17/d18 touch neither the widened files nor the style-1 dispatch
  region (their `projectile.rs` diffs are pure style-arm additions
  elsewhere + d17's one damage-var line) — NO compile fix owed beyond
  ordinary textual merge. SIGNATURE-ADAPTER CONTRACT (published for
  any lane adding an aiStyle-1 flight body after f20 merges): every
  flight body ends with `charm_tail(p, world, npcs, immune)` after the
  flag8 clamp, before the movement (`Projectile.cs:57769` slot); the
  entry signature carries `npcs: &[Npc], immune: &[i32]`; non-arrow
  kinds pass `&[]` for immune (the producer's `arrow` gate
  `Player.cs:28468` never charms them, the tail is the two-bit no-op);
  early-return kill paths skip the tail as the C# `return` does.
  RESIDUAL (recorded, behaviour-inert today): the slice-B/C
  `finish_flight`/`finish` replications, `projectile_special_bullets::
  step`, the 344 snowflake arm, the 507 javelin arm, the 442/459
  pool arms and the 336/337 needle/icicle arms do NOT carry the tail
  — every one of their kinds is `arrow = false` in `data.rs` (re-read
  per-row this round: the slice-B nine, the slice-C nine, the eleven
  special bullets, 344/507/442/459/336/337 all carry the false flag;
  the C# SetDefaults blocks set no `this.arrow` — 442 `:5126-5136`,
  459 `:5292-5303`, 344 `:4339-4352`), so the producer never sets
  their ai[2] bits (the `Player.cs:28468` gate re-read: `!arrow →
  continue`) and the C# tail would be the `:57769` two-bit
  early-return for them; the slice-B doc's "no charm feature in the
  port" skip line is now stale wording (the feature EXISTS, the kinds
  are uncharmable) — a follow-up may add the no-op tails for
  shape-completeness, no behaviour change is owed. The 837 Book of
  Skulls DOES carry the tail (its `ai_style1_flight` call passes
  `npcs, &[]`) despite also being `arrow = false` — the generic-flight
  kinds keep the C# shape unconditionally.

## The f19 base-census closure (verify-1 → verify-2 handoff, retire round)

f19 (`task/W4-f19-proj-ai-verify-1`, tip `ec5dec82`, research `26c` —
the doc lives on THAT branch, not in main) censused aiStyles 1-105
(`e7e3c723`: 63/105 armed, 42 missing, all claimed) and filed five
findings. This lane (verify-2) re-probed every finding against its
claiming branch's pushed tip this round — **ALL FIVE CLOSED
(branch-landed; none merged to main `a725acb0` yet, so main still
shows the gaps until those merges):**

| f19 finding | commit | claim | closure evidence (re-read this round) |
| --- | --- | --- | --- |
| F1 — aiStyle-99 counterweight sub-arm missing (556-561/1079) | `049cc8e2` | d10 | **CLOSED** `7ccca71c` ("style 99 (f19 F1)"): `AI_099_1_Counterweights` `:67380-67731` verbatim in `projectile_ai_91_100::ai_counterweight`, the `:33153` kind gate ahead of the yoyo arm (d10-tree `projectile.rs:18064/18103`), the seven rows `:6084-6093` (`data.rs:3056-3058`, `COUNTERWEIGHT_KINDS`); the mouse-delta seam un-parked `29a73a1d`, the spawn lane `6e0f1381`, the string grants `c6fd3bc0`; d10 tip `e1519df7` |
| F2 — rowed-but-unarmed 156/164/169/206 | `c156903d` | ~~d16/d17/d21~~ **a9** (26e F1 corrected the attribution) | **CLOSED** on `task/W4-a9-minion-four-machines` (tip `8c4fded8`): `bat_of_light_step` `0ee3a5cd`, `storm_tiger_gem_step` `db50ff98`, `smolstar_step` `dc35f100`, `forbidden_minion_step` `8c4fded8` — all four dispatch arms live (a9-tree `:17517/17551/17573/17602`), `UNARMED_ROWED` drops to EMPTY. d16/d17/d21 did NOT re-port (their trees keep the four-entry census pin; d21's 26d records 206 verify-only) — the a9 merge drops the entries |
| F3 — style-37 row is the 185 stand-in | `4a6b2419` | d4 | **CLOSED** `0f92565b` ("arm+row together (f19 F3)"): `ai_trap_spear` (`:27472-27499`) + the faithful 10×14/37/300 row (d4-tree `data.rs` `TRAP_SPEAR => (10.0, 14.0, 37, -1, 300, …)`, dispatch `37 =>` at `:17804`), the stale boulder-186 comments fixed (`:302`), the ANALYSIS S1-D-5 row RETIRED (d4-tree ANALYSIS `:32040-32047`); d4 tip `f1480dc0` |
| F4 — d3's style-25 over-claim | `807da87f` | d3 | **CLOSED** — the aspirational doc sentence became real code AFTER the censused `57abe80a`: `f9d2759e` (the 13-kind AI arm + bounce ladder + RainbowBoulder + Shimmer float + Kill helpers, `projectile_ai_boulder.rs` EXISTS), `241cbae0` (rows), `40f8cbe3` (dispatch `25 =>` at d3-tree `:17580` → `boulder_step` `:17589`), `affd7fa9` (drains), `986ab27b` (dust/gore bodies); d3 tip `5ff9fe98` |
| F5 — 687 aiStyle-1/12×12 + 686 22×22 row deviations | `ec5dec82` | d14 | **CLOSED** `97c5a984` ("style 136: BetsyBreath arm + f19 F5 row fixes"): 687 → 30×30/136/alpha 255, 686 → 30×30/alpha 255 (`:7305-7324` verbatim), the `136 =>` arm (d14-tree `:19623`) + `w4_d14_betsy_breath_tests`; d14 tip `eaa05ae9`. This branch's `26e` F5 refreshed from "still open" to CLOSED |

**f19-remaining: NONE.** F2/F4 were the two not-yet-closed at the
handoff; both re-probed CLOSED above (F2 on a9 — the corrected claim —
not on the decade lanes; F4 on d3's post-census wave). The verify
family's base-census axis (styles 1-105 f19 + 106-210 f20/26e + the
AI_001 overlay axis f20/26f) is fully closed or claimed; no unclaimed
missing arm or overlay remains on either axis.

## Verify-family declaration (SEAL)

**This document (`26f`, round 2 + the retire round) is SEALED as the
verify-family's AI_001-overlay record.** The verify family:
- **f19 / verify-1** (`task/W4-f19-proj-ai-verify-1`, `ec5dec82`):
  research `26c` — the aiStyles 1-105 coverage census + F1-F5 (all
  five closed per the table above).
- **f20 / verify-2** (THIS branch, `task/W4-f20-proj-ai-verify-2`):
  research `26e` (round 1: the 106-210 census + slice D) and `26f`
  (round 2: the unrowed-now-rowed axis + slice E + the riders census
  F1-F8). Ports landed here: slices D/E, F3 (the 709 Kill-hook half +
  the 1110 row), F4 (the 88 cap finalize), F6 (the magicQuiver bump),
  F7 (the Harpy-charm chain). Cross-notes: F5 (shimmer — g2 ported,
  d53 ACTIVATED, keep-one at merge), F8 (the signature ripple —
  all-callers census + the d16 first-compile-fix-list + the adapter
  contract).

No further verify round is owed on the AI_001 axis: every rowed
style-1 kind's per-type overlay is ported or claimed, every rider is
matched, and the two cross-lane systems (shimmer, charm) carry their
merge contracts. The follow-up BUILD lane owns the WRITTEN-NOT-RUN
witness execution (below) and the merge-time fix-lists; the verify
family's census work is DONE.

## Verification

Code-only round (task contract: no cargo/tests/graphify). The slice-E
witness tests are WRITTEN NOT RUN — every expected value hand-derived
from the cited C# lines (derivations in the test doc-comments; the rows
hand-built, the d7 unwired precedent). A follow-up build must run
`cargo test -p tr-app projectile_ai001_e` before the merge, AND re-run it
after c11/d11/d15 merge (the arms activate then). The verify-round ports
(F6/F7/F3) are likewise WRITTEN NOT RUN — the follow-up build must also
run `cargo test -p tr-app` filtered to `w4_f20_f*`,
`projectile_harpy_charm`, and the ai001/darts/moonlord/special_arrows
modules (the F7 tail widened every aiStyle-1 flight-body signature, so
the fixtures across those modules moved too). The census tables were
derived by direct file reads + a mechanical `proj_data` parse of every
pushed branch (no junction recursion): the row diff, the AI_001
type-branch scan, and the riders match against the landed gates.

Retire-round verification (this pass): direct-file re-reads only —
the d53 tip `88094c4c` dispatch gate + the two shimmer bodies
(line-diffed: 6 differing lines, all visibility/STAGED/typo-fix), the
C# `:17009-17012`/`:22505-22514` re-read, the F7 caller scan over this
branch's `crates/tr-app/src` + the three sibling tips' full trees
(d16 re-scanned at its advanced tip `41080e15`), the five f19 closure
commits' `--stat` + tree greps, and the `Player.cs:28448-28490` /
`Projectile.cs:57745-57794` charm-gate re-reads. No cargo/tests/
rustfmt/graphify run (task contract); no code changed this pass —
docs-only (26f F5/F8 + the closure table + the seal; 26e F5 refresh).
