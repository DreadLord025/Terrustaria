# 26f — AI_001 OVERLAYS round 2: the UNROWED style-1 kinds now rowed + the riders census (W4-f20)

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
| extraUpdates remap | `:54025` (`extraUpdates = (int)ai[0]`) | 88 | PORTED `ai_style1_rider` (`:6775`) — the port adds a `.min(15.0)` the C# lacks (the ai[0] rests at 0 on 88, so the remap never fires; the cap is inert, recorded) |
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
| shimmer lift | `:22361-22530` (`Shimmer()`, the `:17009-17011` gate) | the aiStyle-1/16/40 family `:22403` + the arrow list `:22380` | NOT ported pool-wide — the projectile shimmer layer is a separate system (ANALYSIS `:27666`, the g2 `26d` recorded gap "the Shimmer() family"); the 1006 flag4 inversion is the AI_001-internal half, ported. The `Shimmer()` bounce/lift is the f47/e19 item-side lane's sibling, deferred for projectiles |
| magicQuiver extraUpdates | `:17032-17035` (`extraUpdates < 1 → 1`) | the arrow-flag kinds | NOT ported in the pool (the `magic_quiver` flag lives in `ammo.rs` for the shoot-speed fuse; the Update-tail extraUpdates bump is unported) — recorded, the arrow kinds' sub-step count stays at the row value |
| Harpy-charm tail | `:57769-57793` + `ApplyHarpyCharm` `:57796-57824` + `AI_001_Features` `:239-289` | the arrow kinds (ai[2] bits) | NOT ported — no charm feature in the port (the slice-B doc `:48` records the skip); the `accHarpyCharm`/`accSeraphNecklace`/`hasPhoenixQuiver` player flags + `HarpyCharmEffect` (`Player.cs:28447-28490`) are unported. The ai[2] bits stay 0, so the tail's `!HarpyCharmed` early-return (`:57769`) makes it inert — recorded, deferred with the accessory lane |
| PhoenixQuivered dust | `:54005-54011` | the charmed arrows | dust-only + the ai[2] bit gate (inert, no charm) — skipped |

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
- **F3 (the 709 Kill-hook half is now due).** The 709's `Kill()`
  (`:54722`) triggers the death hook `:73338-73389` — the 459/709 shared
  dust/gore burst PLUS the 709-only `:73372-73388` half (the owner-gated
  1110 SkyDragonsFuryElectrosphere circle spawn + the 80-px sibling pull,
  with `timeLeft = 30 * Next(2,6)` and the tracked DD2_SkyDragonsFuryCircle
  cue). The g1 Kill-hook census (`26d-projectile-kill-hooks-census.md`)
  explicitly DEFERRED this half "with its AI arm" (1110 unrowed). Slice E
  lands the AI arm, so the death hook is now the g1 lane's due work —
  1110 needs a row + arm + the hook together (the F3 "row+arm together"
  pattern). The 1110 spawn draws RNG (`Next(2,6)`), so it is NOT RNG-free
  and stays out of slice E (the module takes no rng). Recorded for g1.
- **F4 (the 88 extraUpdates cap).** The port's `ai_style1_rider` adds
  `.min(15.0)` to the 88's `extraUpdates = (int)ai[0]` remap (`:6775`)
  that the C# (`:54025`) lacks. Inert (ai[0] rests at 0 on 88, the remap
  never fires) but a shape deviation — recorded, not fixed (the generic
  path is frozen).
- **F5 (the projectile shimmer layer is unported).** The `Shimmer()`
  family (`:22361-22530`, the `:17009-17011` shimmerWet gate) has no
  projectile-side port — the AI_001-internal 1006 flag4 inversion is
  ported (special_arrows), but the wet-shimmer bounce/lift/kill is the
  f47/e19 item-side lane's sibling, deferred for projectiles (ANALYSIS
  `:27666`, the g2 `26d` recorded gap). The rowed arrow kinds that ride
  the `:22380`/`:22403` shimmer arms take no shimmer interaction until
  that layer lands.
- **F6 (the magicQuiver Update-tail bump is unported).** `:17032-17035`
  bumps `extraUpdates` to 1 for the friendly arrow-flag kinds when the
  owner has magicQuiver. The `magic_quiver` flag is ported in `ammo.rs`
  (the shoot-speed fuse) but the Update-tail sub-step bump is not — the
  arrow kinds' sub-step count stays at the row value. Recorded, deferred
  with the accessory lane.
- **F7 (the Harpy-charm tail is inert-unported).** The `:57769-57793`
  tail + `ApplyHarpyCharm` + `AI_001_Features` (ai[2] bits) are unported
  (no charm feature, the slice-B doc skip). The ai[2] bits stay 0, so the
  tail's `!HarpyCharmed` early-return makes it inert — the port is
  behaviour-correct for the no-accessory case. Deferred with the
  accessory lane (the `accHarpyCharm`/`accSeraphNecklace`/
  `hasPhoenixQuiver` flags + `HarpyCharmEffect`).

## Verification

Code-only round (task contract: no cargo/tests/graphify). The slice-E
witness tests are WRITTEN NOT RUN — every expected value hand-derived
from the cited C# lines (derivations in the test doc-comments; the rows
hand-built, the d7 unwired precedent). A follow-up build must run
`cargo test -p tr-app projectile_ai001_e` before the merge, AND re-run it
after c11/d11/d15 merge (the arms activate then). The census tables were
derived by direct file reads + a mechanical `proj_data` parse of every
pushed branch (no junction recursion): the row diff, the AI_001
type-branch scan, and the riders match against the landed gates.
