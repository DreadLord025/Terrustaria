# 240 — PROJ-DUST RETIRE (W4-d53): the wetVelocity movement-gap census, the g2 staged-copy drop-hunk, the d53×e3 disjointness confirmation, the insurance-patch receipt, the family declaration

Status: RETIRE record (W4-d53, 2026-11-21, code-only). Number per the
research/150 collision registry (d54 `38bfd1d22`): the 151–236 renumber
block is fully assigned, 237/238/239 are taken at pushed tips (237:
e23+g48 both; 238: c8+d42+g61 three-way; 239: d42) — this doc takes
**240**, the first free number at the RETIRE-tip scan (all `task/W4-*`
remote tips, `git ls-tree docs/research`, keys ≥230 enumerated). d53's
existing research/70 keeps its number (the registry's key-70 winner).

Inputs: this lane's tip `88094c4ce` (Shimmer ACTIVATED `804ad571` + the
generic wet machine `fa7b331d3` landed), g2 `81c4ab85a`, f20 `31386266f`
(the F5 cross-note), e3 `ac6392cf2`, d54 `38bfd1d22` (the 131 bible +
the 150 registry), d3 `2d01693ca`.

## 1. The wetVelocity movement-gap census (the owed pool)

The ledger's "~96 sites" resolves EXACTLY on re-count: the port carries
**96 unclaimed integrate expressions** (§1c's table sums to 96). The
C# token count is different: `Projectile.cs` carries **20 `wetVelocity`
token lines** — the capture `:16765`, the two call sites
`:17015`/`:19858` (the `HandleMovement`→`UpdatePosition` pass), the
signature pair `:17469`/`:20433`, the drag block's 9 lines
(`:17618/17621/17625` shimmer ×0.375, `:17632/17635/17639` honey ×0.25,
`:17646/17649/17653` water ×0.5), the integrate `:20441`, the AI_015
bounce refresh pair `:19323`/`:43715`, and the DEAD
`AI_015_HandleMovementCollision_Old` pair `:44171`/`:44201` (no caller —
grep finds only the definition; cited dead, not owed). The ~96 is the
MOVEMENT-SITE pool the drag must reach, not the token count.

**The C# chain (re-read this round, ≤100-line windows):**
`:16765` `wetVelocity = base.velocity` captures at the Update head
(PRE-AI, post-gfxOffY); `:16775` `AI()` may change velocity;
`:17015` `HandleMovement(wetVelocity)` — the wet arm (`:17612-17655`)
runs FULL-velocity `TileCollision` then scales the RESULT into
`wetVelocity` (factor by shimmerWet/honeyWet/else-water), restoring each
CHANGED axis to the full post-collision value (the blocked-axis
restore); `:19323` the aiStyle-15 collision handler REFRESHES
`wetVelocity = base.velocity` on a wet bounce (`:43713-43716`);
`:19858` `UpdatePosition(wetVelocity)` — `:20435` the skip list
(aiStyles 4/38/84/148, the 440/449/606 ai[1]==1 latch, 93 ai[0]<0, the
540/756/818/856/961/1041/1125/933/1100 kinds, `IsAGolfBall`) returns
without integrating, else `:20439-20446` `base.wet ? wetVelocity :
velocity`.

### 1a. What the landed wet block covers: ZERO movement sites

`projectile_wet::wet_block` (`:16786-17007` verbatim, `fa7b331d3`) is the
FLAG machine — it writes `wet`/`lava_wet`/`honey_wet`/`shimmer_wet`/
`wet_count` and queues the splash edges. It contains **0 of the ~96
movement sites**: the only velocity write in the whole block is the
Beetle 155 exit `velocity.Y *= 0.5` (`:16920-16923`, a carve-out, not an
integrate). The drag block (`:17612-17655`) sits in `HandleMovement`,
OUTSIDE the claimed `:16790-17007` window — the census verdict of
`ab40c8a1c` stands, now quantified.

### 1b. Ported today (the two wet-carrying families)

* **The flail** (aiStyle 15) — `flail_move`'s displacement scale
  (`projectile.rs:6100-6118`) + `flail_wet_factor` (`:6135-6145`,
  shimmer 0.375 / honey 0.25 / water 0.5, the mid-tile probe idiom;
  S1-P-i07 fix 9). The `:43715` bounce-refresh is approximated: the
  factor is re-probed each tick instead of the captured-then-refreshed
  local (the recorded mid-tile idiom deviation).
* **The d3 boulder family** (aiStyle 25, `2d01693ca`) —
  `boulder_movement`'s wet arm ports `:17612-17655` VERBATIM (factor
  pick, full-velocity tile_collision, per-changed-axis restore) + the
  `UpdatePosition` wet/dry integrate. Not on this branch (d3's tip);
  cited, not counted below.
* **The golf engine** — `tr_core::golf::step_golf_ball` +
  `projectile_ai_golf.rs:228` (`p.pos = body.position`) is
  UpdatePosition-SKIP-LISTED (`:20435` `IsAGolfBall`): the C# never
  wet-integrates a golf ball, so the engine is OUT of the owed pool
  (not a gap — the earlier "~42 free moves incl golf" phrasing
  over-counted it).

### 1c. The remaining unclaimed pool (this tip's census) — **96 exactly**

The port carries **103 `.pos +=` code lines** across `projectile*.rs` +
`grapple.rs` + `special_arrows.rs` (the 104 grep hits minus
special_arrows.rs:379's comment mention). Minus the flail's 1 PORTED
integrate (:6114), minus 5 non-movement positional writes, minus the
aiStyle-4 fold's 1 skip-listed stand-in (:3038 — the C# `:20435` skip
list exempts aiStyle 4 from `UpdatePosition` entirely, so the wet drag
never reaches it; the port's `:19855-19858` citation there is itself
imprecise, noted for the movement lane) = **96 unclaimed integrate
expressions**. The kill_step funnel's 54 callers share its 2 integrate
lines — expressions, not callers, are the pool. Verified counts
(`Select-String -AllMatches` sums, this tip):

| class | count | sites |
| ----- | ----- | ----- |
| plain velocity integrates (`+= self.vel` / `p.vel` exactly) | 74 | projectile.rs 39 (the 40-line list minus :3038's skip-listed fold; incl the kill_step pair :10138/:10145, ai_style1_flight's free arm :6817, the vile-spit AI-internal step :2994 — C# :23732); the 14 satellite modules 28 (ai001_a/b/c 1+1+1, ai026_a 2 / ai026_b 1 / ai026_c 2, darts 2, empress 1, frostmoon 2, light_items 4, moonlord 3, thrown 2 (:116 the twin's integrate + :194), frost_shots 3, special_bullets 3); special_arrows.rs 7 (:260/:267/:321/:328/:380/:656/:663) |
| scaled / captured velocity integrates | 9 | projectile.rs :4128/:8451 (`vel * 0.99`), :12444 (pirate_movement's TileCollision `vector10`); ai026_c :901 (`vector10`), light_items :284/:618 (`v10`), :647 (`last_velocity`), moonlord :245 (`last`), spear :359 (`vel * offset`) |
| dispatch-level UpdatePosition mirrors | 11 | projectile.rs :17168/:17697/:17723/:18195/:19132/:19156/:19687/:19721/:19774/:19819/:20160 |
| the charged-blaster mirror | 1 | projectile.rs :21223 |
| the hook UpdatePosition mirror | 1 | grapple.rs :713 |
| **TOTAL unclaimed** | **96** | 74 + 9 + 11 + 1 + 1 |
| flail (PORTED, §1b — not owed) | 1 | projectile.rs :6114 |
| skip-listed fold (NOT owed — aiStyle 4, :20435) | 1 | projectile.rs :3038 |
| NOT movement (out of the pool) | 5 | :3960 (host-NPC velocity ride), :4896 (minion attachment delta), :8292 (whip oscillation), :21563 (venom-spread teleport walk), ai_misc :383 (rotated offset) |

(103 = 96 + 1 + 1 + 5.) The funnel structure (what makes 96 tractable):
the **5 swap points** carry the bulk — `kill_step`
(`projectile.rs:10136`, **54 callers**, the 2 integrates
:10138/:10145), `ai_style1_flight` (`:6795`, the free arm :6817 + the
blocked-test arm :6813-6814's `pos.x`/`pos.y` writes — one swap point,
not two pool entries), `special_arrows`' bounce integrates, the thrown
twin (`projectile_ai_thrown.rs:114`, **3 callers** :107/:273/:380), and
the 12 dispatch mirrors — plus the per-arm free moves in the satellite
modules. The C# side's own census: 59 `base.position +=` lines, 26 of
them `+= base.velocity` (the AI-internal moves the port mirrors), 1
`+= wetVelocity` (the single generic integrate every non-skip-listed
kind funnels through). **Remaining-count headline: 0 of the 96 covered
by the wet block; 96 integrate expressions unclaimed on this tip (the
flail's 1 ported here; d3's boulder arm ported on its own tip; golf
skip-listed).** RNG-neutral (the drag has no draws) — a physics
deviation, not a stream one; the one-tick position lead (the port
integrates inside the dispatch, the wet block runs post-AI) compounds
on a shot's FIRST wet tick, the recorded bobber `probe_wet` lag family.

### 1d. The movement-lane task seed (S1-P-i57)

Seeded in ROADMAP + research/90 §2(d): **the generic wetVelocity arm** —
a `wet_velocity` helper (the `:17612-17655` factor pick + blocked-axis
restore, verbatim; d3's `boulder_movement` wet arm is the reference
port) + the integrate swap at the 5 funnel points + the 12 dispatch
mirrors + the satellite free moves; the `flail_wet_factor` keep-one
(the helper supersedes the mid-tile idiom; the `:43715` bounce refresh
rides the helper's captured-local shape); the `UpdatePosition` skip
list (`:20435`) ported as the gate (golf/boomerang-return/latch kinds
keep their engines). Input ready: the wet machine's flags are the
authoritative writer since `fa7b331d3`. NOT this dust lane's window
(the claim is the dust SITES `:16851-22297` + the wet-machine host);
the movement lane owns `HandleMovement` (`:17469+`).

## 2. The g2 staged-copy drop-hunk (f20 F5 cross-note consumed)

f20's F5 (`31386266f`) cross-noted g2's Shimmer() as STAGED/unwired
pending the generic wet block, "nothing owed by this lane — port +
wiring stay g2's claim". **SUPERSEDED**: this lane's `804ad571`
ACTIVATED the family (the g2d-active probe found no pushed activation
branch; the wet machine `fa7b331d3` is the host g2 waited for, so the
`:17011` caller-gate landed HERE, right after `wet_block` in the
dispatch, `projectile.rs:20179-20196`). The keep-one direction is
**KEEP d53's ACTIVATED copy, DROP g2's staged copy** — the published
drop-hunk (g2 tip `81c4ab85a` line numbers, for the merger):

| # | file | hunk (g2-tip lines) | action |
| - | ---- | ------------------- | ------ |
| 1 | `crates/tr-app/src/projectile.rs` | **:10189-10377** — the `/// Projectile.Shimmer()` doc block (`:10189`), the `#[allow(dead_code)] // STAGED` marker (`:10208`), `fn shimmer` (`:10209-10364` incl the corrected-clamp comment), the `shimmer_two_cell_kill` doc + `#[allow(dead_code)]` (`:10365`) + body (`:10366-10376`), the trailing blank (`:10377`) | DROP whole (d53's `projectile.rs:13109-13296` is the surviving ACTIVATED copy — byte-identical bodies, the STAGED doc paragraphs + allow-markers dropped, the 25/17 clamp-comment typo fixed: −10 per `:22511-22514`, not −8) |
| 2 | `crates/tr-app/src/projectile.rs` | **:45185-45293** — the `// --- W4-g2 reconciliation: the Shimmer() family (STAGED, unwired)` banner + all 5 `w4_g2_shimmer_*` witnesses inside g2's `mod tests` (ends `:45293`, the mod's `}` at `:45294` STAYS) | DROP (d53's `w4_d53_shimmer_tests` `:44839-44994` is the superset: the same 5 arms re-derived + the style-1 bounce cap/age + the thrown −0.7/−20 arm + the DISPATCH-GATE witness — 7 vs 5) |
| 3 | `crates/tr-app/src/projectile/data.rs` | **:917** `pub const CHLOROPHYTE_ORB: u16 = 229;` | KEEP-ONE either copy (identical value; d53's `data.rs:712` carries the keep-one note). g2's ROW (`:4002`), light (`:1625`), frames (`:4830`) + machine arms STAY g2's — d53 added only the const; the `:22403` kind arm is inert on d53 until g2's row merges |

The g2 witnesses' arm coverage map (drop-receipt): orb-bounce →
d53 `w4_d53_shimmer_the_style1_bounce_caps_and_ages` + the 229 arm
rides the ACTIVATED helper (the orb row is g2's); two-cell-probe →
`w4_d53_shimmer_the_two_cell_probe_kills`; arrow-float →
`w4_d53_shimmer_the_arrow_floats`; hostile-gate →
`w4_d53_shimmer_the_hostile_gate_refuses`; star-bounce →
`w4_d53_shimmer_the_star_bounce_keeps_life`. No arm loses coverage.
g2's phase-4 docs (research + ANALYSIS "STAGED/unwired") re-classify
ACTIVATED-BY-D53 at the merge; the anchors row keeps d53's
(`projectile.rs|pub(crate) fn shimmer`), g2's staged-marker row (if
any) drops with hunk 1. The f20 F5 cross-note is hereby CONSUMED —
f20 owes nothing further; the 936 rainbow-slime spawn arm stays g2's
cited skip (rowless + the NPC-spawn/net/achievement layers).

## 3. The d53×e3 projectile_dust.rs fn-sets DISJOINT — CONFIRMED

The d54 bible (research/131 §8.3 step 9, tip `38bfd1d22`): "fn-name sets
are DISJOINT (verified this round — only the test helper `ctx`
shared)". Re-verified independently at the CURRENT tips (d53
`88094c4ce` × e3 `ac6392cf2`, past the bible's `8023aa4d`/`1d26efa94`
reads): full fn-name extraction from both `projectile_dust.rs` copies —
d53 **46** fn names, e3 **123** fn names, `Compare-Object -IncludeEqual`
intersection = **{`ctx`}** (the test helper) exactly. The batch-3
`lerp_color`/`next_vector2_circular` collision guess stays WRONG (the
FINAL-phase correction stands: e3's helpers are `hsl_color`/
`next_vector2_unit` etc.; d53's `lerp_color` is private and unshared).
The real union hazards are unchanged: the two `mod tests` blocks (one
`mod tests` per file — rename one at union) + the shared `use` lines.
The merger's naming call stands (union into one file or rename e3's
module); no body contest, no keep-one needed on any emitter.

## 4. The insurance patches — SUPERSEDED, receipt

The three temp patches (`%TEMP%\opencode\d53-batch3-*.patch`, written
2026-11-21 06:16-06:17 as the stood-down d53c session's insurance) are
**fully superseded by the landed batch-3 commit `6d09fe617`** (the
rescue-commit adapted to `fa7b331d3`'s drain_burst signature). Per-hunk
verification against this tip:

* `d53-batch3-data.patch` (2089 B, 2 hunks) — the
  `CULTIST_LIGHTNING_ORB_ARC` const + the 466 row (alpha-255 tuple):
  landed at `projectile/data.rs:309` + `:2477` (byte-equivalent).
* `d53-batch3-projectile.patch` (4563 B, 5 hunks) — the mode-1 gate
  comment, the 466|580 stationary-puff gate, the `trailing_mode` 466
  arm, the `pending_dust_bursts.extend` ring feed, the aiStyle-88
  kind-gate: landed at `projectile.rs:10189` / `:13686` / `:16978` /
  `:18271` (all five present).
* `d53-batch3-projectile_dust.patch` (7876 B, 4 hunks) — the module-doc
  batch-3 scope, the two-box `enchantment_burst -> Vec<DustBurst>`
  (the ±π/4 ring), the STAGED `super_star_dust`: landed at
  `projectile_dust.rs:186` / `:195` / `:836` (the FINAL phase later
  tightened the witnesses, `88094c4ce`).

**Disposition: DELETE-AT-CLEANUP, never apply.** Applying any of them
to this tip double-lands the hunks (the patch context no longer matches
anyway). They are temp-dir scratch, not repo files — nothing to commit;
this receipt is the record. The worktree lifecycle rule (no stray
`*.patch` on the Desktop) is satisfied: they live in
`%TEMP%\opencode\`, outside the repo.

## 5. The proj-dust-family declaration (the merger bible paragraph)

**ONE projectile-dust family, three site lanes + the wet host, merged
in site order d53 → e3 → f5 with the per-kind trails g63/g64 composing
by axis:** d53 (`task/W4-d53-dust-sites-projectile`, tip `88094c4ce` +
this RETIRE) owns sites 1-40 (`Projectile.cs:16851-22297`) — 29 wired +
1 armed + 4 staged + the ring builder + 1 keep-one-e3 + 5 cited skips
(research/70's FINAL ledger) — PLUS the two hosts the sites needed: the
generic wet machine (`projectile_wet.rs`, `:16786-17007` verbatim, the
authoritative wet-flag writer, 8 splash sites wired) and the Shimmer()
ACTIVATION (g2's helper adopted + the `:17011` gate wired; §2's
drop-hunk retires g2's staged copy, f20's F5 cross-note consumed). e3
(sites 41-80 + the :24328 edge pair) and f5 (81-120) keep their windows;
the `projectile_dust.rs` FILENAME collision d53×e3 resolves by union or
rename (§3: fn sets DISJOINT at the current tips, only `ctx` shared;
the two `mod tests` + `use` lines are the whole hazard). The d3 boulder
emitters (1053/1048) drop in per research/70's REPLACE contract (the
1048 ladder pairs DELETE, never double-draw); the 728 emitter drops
into d16's count-only gates; the 977/1045 keep-one is e3's
`star_slash_dust`. The dust-vs-sound split with d75 is clean (d3 queues
the 1053 Dig + 1048 Item177 cues; the wet-splash case-19 plays are this
lane's; d75's rows re-classify SIBLING-CUED at merge). **The ONE owed
follow-up outside the family: the generic wetVelocity movement arm
(§1, seed S1-P-i57) — the wet machine's flags are its ready input, the
movement lane owns the integrate; 0 of the 96 movement sites ride the wet
block, 96 integrate expressions unclaimed (the flail + d3 boulders
ported, golf + aiStyle-4 skip-listed).** RNG-neutral throughout the owed arm; the
family's witnesses (20+3 drain, 9+1 machine, 7 Shimmer — all WRITTEN
NOT RUN, code-only) ride the union.

## Verification (RETIRE)

Code-only: the C# chain re-read in ≤100-line windows (`:16700-16789`,
`:17469-17528`, `:17580-17674`, `:20433-20472`, `:43700-43724`); the
token/site counts by `Select-String -AllMatches` sums (20 wetVelocity
lines; 59/26/1 position-integrate lines; 96 `.pos +=` port lines, 42
exact `self.pos += self.vel` in projectile.rs); the fn-set intersection
by full extraction + `Compare-Object` at both current tips; the g2
drop-hunk bounds read at `81c4ab85a` directly; the registry free-number
scan over every pushed `task/W4-*` tip; the three patches read end-to-end
and each hunk grep-confirmed landed. **No cargo/rustc/rustfmt/tests/
dotnet/graphify invoked** (user directive); witnesses WRITTEN NOT RUN.