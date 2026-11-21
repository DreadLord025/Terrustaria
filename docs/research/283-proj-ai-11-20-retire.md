# 283 — Projectile aiStyles 11-20 RETIRE FINAL (W4-d2, code-only)

Branch `task/W4-d2-proj-ai-11-20`, worktree TR-W4-d2. RETIRE base tip
`f430ccf83` (clean). Number per the collision registry
(`150-research-number-registry.md`, d54 refresh `72226e500` §6: 151-236 +
244-280 assigned, 281+ self-adjudicated) + a fresh all-tip
`git ls-tree docs/research/` sweep at the RETIRE fetch: 281 has SEVEN live
claimants (c9/c25/d106/d125/d30/d9/g48), 282 has two (d125
`282-d125-retire-final.md`, g48 `282-39-item-ownership-seed.md`) —
**283 is first-free at this tip**. Witnesses stay WRITTEN NOT RUN
(no cargo/tests/rustfmt/graphify per lane constraint).

## 1. The git-log census — what landed on band-11-20

Base `a725acb04` (wave-3 complete) .. RETIRE tip `f430ccf83`, five commits:

| commit | what |
| --- | --- |
| `ebf418031` | aiStyle-11 light pets verbatim (`Projectile.cs:24371-24500`): Shadow Orb 18 + fairies 72/86/87 — fairy head, affirmative keep-alive pin, owner seek + 800-px teleport + overshoot cap; 4 rows, the buff-pass spawn seam, 13 RNG-mirror witnesses (the commit message's "12" undercounts by one — `#[test]` census at tip = 13) |
| `15e5448a9` | aiStyle-13 chain weapons verbatim (`:24636-24829`): Harpoon 23 / Golem Fist 262 / Boxing Glove 271 / Chain Knife 273 / Chain Guillotine 481 — outbound flips + fade-in + caps, return seek/catch/burst, blocked-move return, on-hit return arms + StatusNPC debuff rolls; 5 rows, 17 witnesses (14 module + 3 damage_pve — the commit message's "16" undercounts by one) |
| `61c47ed79` | aiStyle-17 tombstones verbatim (`:25316-25366`): Tombstone 43 / grave markers 201-205 / rich gravestones 527-531 — grounded friction/spin/gravity, FTW disarm, tile-85 placement + Sign epitaph (`misc_text` field), ×−0.75/×−0.7 bounce + Next(3); the death spawn wired (`DropTombstone` launch draws, dead `TombFall` stand-in replaced); 11 rows, 10 witnesses (+3 death-pipeline witnesses re-pinned for the two launch draws) |
| `b1c4ec0a8` | fix round: style-13 movement probes tiles only (the `:17900` world-edge flag is aiStyle-3-only, not 13 — re-read verbatim: `this.aiStyle == 3 && (…leftWorld…)`); tombstone placement witness probe cell corrected to the air row 29 (y=450; y=460 landed on the solid floor row 30 and refused CanPlace) |
| `f430ccf83` | witness fixes: the outbound fade-in test parks the fist at ZERO velocity (a moving fist crossed the 500-px distance flip mid-loop and switched to the return state); the harpoon dead-zone comment corrected to the Manhattan sum 27 (\|8\|+\|19\|, was "11") |

Total: **20 rows** (4+5+11), **40 module witnesses** (13+17+10) + the
death.rs re-pins, all WRITTEN NOT RUN. Both fix commits are
test-side/comment-side
plus the one faithful movement correction (the world-edge probe removal —
the C# style-13 arm never reads the world edge).

## 2. The 261 question — RESOLVED: flail is aiStyle 15, NOT 14; 261 is a style-14 boulder, UNCLAIMED, band-11-20 owns it

The Lead's brief asked "261-flail-aiStyle15-NOT14 (d75e-census-correction;
movement-half-gap; your-band-owns-261? census)". Verdicts, each C#-cited
(direct-path reads, ≤100-line windows):

- **The correction STANDS.** `DefaultToFlail()` sets `this.aiStyle = 15`
  (`Projectile.cs:10753-10759`); its ten callers are the flail kinds
  25/26/35/63/154/247/757/947/948/1058. The flail machine is `AI_015_Flails`
  (the port's `ai_flail`, style-15 dispatch `projectile.rs:18107`, S1-P-i07).
  d75's part-4/part-5 receipt (`e1bed75cf`/`b59e3a874`: "the flail machine
  is aiStyle **15**, NOT 14") is correct and consumed.
- **261 is BoulderStaffOfEarth** (`ProjectileID.cs:1076`), `SetDefaults`
  `Projectile.cs:3452-3462`: 32×34, **aiStyle 14**, friendly, penetrate 6,
  magic, ignoreWater, extraUpdates 1. NOT a flail.
- **Does band-11-20 own 261? YES by the style axis** — style 14 is inside
  the band; the d-lane split is by aiStyle (d3 owns 21-30 etc.). **NO by
  landing** — the 261 row is UNPORTED on EVERY pushed tip: re-probed at
  this RETIRE's fetch via `git show <tip>:…/projectile/data.rs` on
  d3/d75/d77/f1/g2/e3 (all negative) + this branch's HEAD (no 261 arm,
  no `BOULDER_STAFF`/261 constant). d75's part-5 remaining-unrowed list
  agrees ("261 (row + the style-14 boulder movement)"). No other lane
  claims it: d3's boulder family (`241cbae09`) rows the TRAP boulders
  99/727/1013/1014/1021/1057/1048/1053/1054/1055/1005/655/1047 — the
  staff boulder 261 is not among them; g63/g64's dust axes don't row.
- **The gap is TWO halves, both unported** (d75: "no owed gate possible
  without the movement half — the row-porter owns both"):
  1. **The row + AI type-arm** (`:24989-25042`): the localAI[1] first-tick
     latch (localAI[0] = 80), `rotation += vx·0.05` + the airborne
     `spriteDirection·0.01`, `ai[0]++` capped 15 with the grounded ×0.97
     friction + the ≤0.01 Kill + the +0.2 gravity, the 80-tick dust
     countdown's TWO `Next(maxValue)` gates (5/10/15 by the <20/<10
     bands) feeding dust 31 + dust 228.
  2. **The movement half** (`HandleMovement`'s style-14 block): the
     ±3 gate `:19643` (`velocity.X != lastVelocity.X && (last.X < −3 ||
     > 3)` OR the Y twin) → 10× `Collision.HitTiles` + `PlaySound(0,
     Center)` `:19649` + `MakeBoulderOfEarthExplosion` `:20391-20431`
     (localAI[0] = 40, the 20/15/15 spread + 10/7/7 fluff + 50/30/30
     distFluff + 6/4/2 layers ladder off `velocity.Length()` 8/4 and
     \|vy\| 4/2, `CreateGroundExplosion` `:53401+` — the RNG-bearing
     body, 5+ draws in the first 100 lines — then the 10× dust-228 ring
     with the i<5 noGravity/1.8-scale branch). Plus the satellite sites:
     the `Damage` <3.5 halving `:13568-13572`, the `Damage_GetHitbox`
     26×26 col-box `:19952-19955`, the <1.5 rowless damage-refusal
     `:12448` (auto-activates on the row — d18's activation contract),
     the Kill arm `:74424-74441` (Dig at position + 15× dust-148 ×2
     vy−0.4 + 3× dust-228 noLight ×0.8 vy−0.3).
- **Disposition: TASK SEED `S1-P-i61`** (i57 is d53's wet-movement seed;
  i58 is DOUBLE-SEEDED — g65's TorchID.Dust `c356cb99c` + d3's 1023
  barrel `740026af6`; i59/i60 are g65's — i61 first free at the all-tip
  seed grep; ROADMAP row added). NOT ported in this RETIRE round — the movement
  half needs the `CreateGroundExplosion` RNG census + the generic
  style-14 `HandleMovement` arm the port does not model (the port's
  style-14 movement is the `ai_spear` tile-kill scan, the recorded
  S1-D-4c deviation). The seed specs row + AI arm + movement + Kill +
  the d75 owed Dig gate (d75 cannot arm it: no row, no movement probe).
  The 277 ThornBall shares the `:19952` col-box arm and the `:25044-25061`
  twin AI block — its row IS ported (`data.rs:2157`), its movement rides
  the same seed's style-14 half.

## 3. All-styles-armed / kinds-rowed + the remaining gaps

**Styles: 10/10 armed** at the RETIRE tip (the dispatch
`projectile.rs::update`'s `match data.ai_style`): 11 (`:16987`,
kind-gated LIGHT_PET_KINDS), 12 (`:17374`, ai_aqua_scepter → the 280
head), 13 (`:17009`, CHAIN_KINDS), 14 (`:17179`, glowstick gate +
frostmoon pair + the 480 dust + ai_spear tail), 15 (`:18107`, ai_flail),
16 (`:17354`, ai_bomb → ai_rocket), 17 (`:17056`, TOMB_KINDS), 18
(`:17368`, ai_demon_scythe), 19 (`:17267`, ai_spears + lance hold +
sub-shot drain), 20 (`:17507`, ai_drill; the Laser Drill 445 is aiStyle
75, NOT 20 — its `ai_drill` reuse at `:17820` is the style-75 arm). The
`s1_d_4c_every_rowed_style_has_exactly_one_live_arm` pin covers the set
(the UNARMED_ROWED list holds only 156/164/169/206 — none in the band).

**Kinds: the full C# census** (every `this.aiStyle = N;` assignment for
N in 11..20 + the three `DefaultTo*` helpers' callers, mechanically
swept):

| style | C# kinds (count) | rowed at tip | UNROWED remainder |
| --- | --- | --- | --- |
| 11 | 18, 72, 86, 87 (4) | **4/4** (this lane) | — |
| 12 | 22, 280, 288 (3) | 2/3 (22 i20, 280 i21) | **288** GoldenShowerHostile — d75 armed its owed first-tick Item17 gate (`20b2cea61`), inert until the row |
| 13 | 23, 262, 271, 273, 481 (5) | **5/5** (this lane) | — |
| 14 | 24, 50, 53, 183, 185, 196, 261, 277, 326-328, 346, 352, 378, 400-402, 450, 473, 480, 483, 515, 538, 870, 928, 1089, 1107-1109 (29) | 24/29 | **24** SpikyBall (the thrown one — NOT the trap 185), **196** SmokeBomb, **261** BoulderStaffOfEarth (§2 seed), **483** SeedlerNut, **538** Twinkle (d75: the `:25166`/`:25280` second-gate sites are C# dead code) |
| 15 | 25, 26, 35, 63, 154, 247, 757, 947, 948, 1058 (10, DefaultToFlail) | **10/10** (i07 + the mace pair) | — |
| 16 | 28, 29, 30, 37, 75, 102, 108, 133-144, 164, 303, 338-341, 397, 470, 516, 517, 519, 588, 637, 681, 773, 776-801+803-810, 862, 863, 903-906, 910, 911, 930, 1002, 1077, 1086, 1087, 1088 (82) | 65/82 | **75** HappyBomb, **303** RocketSkeleton, **470** StickyDynamite, **588** PartyGirlGrenade, **637** BouncyDynamite, **773** ScarabBomb, **903-906** wet/lava/honey/dry bombs, **910/911** dirt bombs, **930** SantankMountRocket, **1002** TNTBarrel, **1077** FreezeBomb, **1086/1087** super bombs (17; the STICKY_BOMBS table `data.rs:869` already NAMES 470/773/911/1087 — rows owed) |
| 17 | 43, 201-205, 527-531 (11) | **11/11** (this lane) | — |
| 18 | 44, 45, 263, 274, 1092 (5) | 2/5 (44/45 base) | **263** IceSickle, **274** DeathSickle (their shared `:25374-25390` arm: ×0.05+×0.5·(timeLeft/180) spin, ×0.95/×0.96 damp), **1092** LibrarianSkeletonBook (the `:25392+` book machine: Next(6)·9+Next(3) frame roll, the −3 vy pop, AngleTowards 0.25, the ai[1] countdown) |
| 19 | 46, 47, 49, 64, 66, 97, 105, 130, 153, 212, 215, 218, 222, 342, 367, 368, 730, 877-879, 1103 (21, DefaultToSpear) | **21/21** (i06) | — |
| 20 | 57-62, 107, 213, 214, 216, 217, 219, 220, 223, 224, 252, 369, 427-432, 509, 609, 610 (26, DefaultToDrillOrChainsaw) | 25/26 (i29) | **252** ChlorophyteJackhammer (`:3351-3357`: 18×18, scale 1.1) |

Band total: **196 C# kinds, 169 rowed, 27 unrowed** — every remainder
named above with its SetDefaults cite. NONE of the 27 has a claimant on
any pushed tip (the d75/d77/f1 remaining lists agree where they name
them: 288, 261, 303, 930, 903-906, 773, 263 are on d77's still-deferred
list; 538/483/24/196/252 are row-only follow-ups of this band's axis).

**Row-shape remainders (rowed but diverged, recorded not fixed here):**
- The 186 TRAP_SPEAR row keeps the WIRE-1 legacy 185-shape (aiStyle 14
  vs the C# 37) — the documented S1-D-4c deviation, unchanged.
- The style-14 movement half is the `ai_spear` tile-kill scan for ALL
  style-14 kinds (the C# `HandleMovement` bounce families — the 185
  ×−0.9 bounce `:19703-19713`, the 326-328/400-402 ×−0.1 `:19652-19662`,
  the 870 full-reverse `:19663-19670`, the 50 ×−0.2 `:19671-19681`, the
  ≥1089 ×−0.99 `:19682+` — are unmodelled; the glowstick family's own
  bounce IS ported in `projectile_ai_light_items`). The S1-P-i61 seed
  takes the 261 slice; the wider style-14 bounce census stays the
  recorded deviation until a movement lane takes it.
- The 876 ZapinatorLaser is NOT a band kind — its own `SetDefaults` arm
  (`:8416-8429`) sets `aiStyle = 1`; the `DefaultToSpear` call at
  `:8435` belongs to the NEXT arm (the 877-879 jousting lances,
  `:8430-8437`) — the helper-call sweep's one attribution slip,
  resolved by the full-arm read.

## 4. The d77c / f1 sounds compose — kill-tail DISJOINT confirmed

Re-read at the CURRENT fetched tips: d77 `5f10560b2`
(`kill_tail_cue` at its `projectile.rs:20211`, the full ~79-arm map
enumerated), f1 `3065492b0` (`kill_sound_cue` at `:21318`, the numeric +
named arms enumerated), d75 `b59e3a874`.

- **The band's OWN kinds (the 20 rows this lane landed) are DISJOINT from
  both kill maps**: no arm for 18/72/86/87, 23/262/271/273/481,
  43/201-205/527-531 in d77's map, f1's map, or either rocket census.
  C#-verified: the Kill range `:70978-78560` names NONE of these kinds
  (mechanical sweep — the `type == N` literals AND the `aiStyle ==
  11/13/17` / range-form gates both return zero hits) — the light pets die via timeLeft/buff lapse, the
  chains via catch/leash, the tombstones via the placement Kill (whose
  Dig thud rides the AI arm, `:25357`, already cued `(0,0)` at tile×16 in
  `projectile_ai_tombstone.rs:157`). The chains' in-flight cues ride d2's
  own machine: the 262 return-burst Item14 `(2,14)` (`:24781`,
  `projectile_ai_chain.rs:229`) and the blocked-move Dig `(0,0)`
  (`:19571`, `:298`). d75's part-3 table classifies both SIBLING-CUED
  (d2) — consumed, no owed arm.
- **The band's INHERITED rowed kinds overlap ONLY via keep-one-identical
  arms**: the 44/45 sickle/scythe Kill Item10 `:76300` (d77
  `44 | 45 => (2,10)` ≡ f1 `DEMON_SICKLE | DEMON_SCYTHE => (2,10)` — the
  d77-superset drop applies, plays once); the 928 Drippler extra-ball Dig
  `:74444` (both maps, identical `(0,0)`); the 346 ornament Item27
  `:74238` (both); the 183 beenade Item14 `:76433` (both). The bomb family 28/29/30/37/397/516/517/
  519 + the rocket families' cues ride the BASE `ai_bomb` detonation
  latch (`:78331`/`:78386` Item14 at the PRE-resize position — the i16/
  i28 lane, present on all three tips) + d77's `rocket_kill_census` cue
  pushes; f1's census pushes are DEPRECATED-BY-UNION (its own part-8
  banner). NO double-play after the f1-drop.
- **The d77c-monitor receipt is CONSUMED**: f1 part 8 re-pinned it at
  the moved d77 tip `5f10560b2` — the Kill-host wave `0573b70f2` absorbed
  f1's part-6 owed wave + the second wave's 523/12/321; the 710/617 hosts
  are d77's whole; the 410 drops via keep-one d8. The f1-UNIQUE fold
  remainder (the Flame Burst head 664/666/668/706 + the second owed wave
  minus 523/12/321 + the facade routes) touches NO band-11-20 kind —
  every kind in that list is outside 11-20's roster (checked against §3's
  table). The id-4 STYLE-vs-INT overload note (d75 part 5: d3's `(4,6)`
  is the style call, d77's drain routes int — keep both paths) and the
  id-29 truncation wrinkle are drain-side, disjoint from this lane's
  `(0,0)`/`(2,14)` pushes.
- **The one compose obligation this lane owes the merger**: the
  `wants_sound` 5-tuple pushes in the three new modules must forward
  through the d77 6-tuple queue seam (`volumeScale` 1.0 — f1 `4233ddb4b`
  adopted it verbatim, every stock caller 1.0). The pool's existing
  forward already destructures the sixth term on the sounds-lane tips;
  the merge is mechanical (the d2 pushes are stock-shape), no re-derivation.
- **The 261 Kill/bounce Dig sites stay UNCLAIMED on the sounds side**:
  d75 part 4/5 recorded "no owed gate possible" — with the S1-P-i61 seed
  the row-porter owns BOTH the `:19649` bounce Dig and the `:74426` Kill
  Dig (the seed specs them; the sounds lanes owe nothing).

## 5. The band-11-20 declaration (SEALED)

**Band 11-20 is COMPLETE on the style axis: 10/10 styles armed, all
three missing styles (11/13/17) landed verbatim with their movement
halves, spawn seams and RNG-mirror witnesses.** The kinds axis carries
27 named unrowed remainders (§3) — every one is a row+machine follow-up
outside this lane's landed scope, with the 261 two-half gap task-seeded
(S1-P-i61) and the 288/383-style owed sound gates already armed on d75.
Boundaries: style 10 is d3/base's (the snowball/debris arms), style 21
is d3's band (21-30) — no boundary overlap; the 186 trap-spear legacy
row and the style-14 tile-kill movement stand-in stay the recorded
deviations (not new gaps). The sounds compose (§4) is DISJOINT for every
kind this lane owns; the keep-one drops are the sounds lanes' own
published verdicts, consumed here. NO census/arm/row work is owed on
band 11-20 beyond the named seeds. Witnesses: 40 module + 3 death
re-pins + the 3 RETIRE witnesses below — ALL WRITTEN NOT RUN.

### The RETIRE witnesses (projectile.rs tests tail, written not run)

1. `w4_d2_retire_the_band_styles_all_have_live_arms` — the 10 style
   labels 11..20 each appear exactly once in the dispatch's arm scan
   (the s1_d_4c scan shape, band-scoped).
2. `w4_d2_retire_the_band_rows_match_the_landed_roster` — the 20 landed
   kinds' rows carry their band styles (4×11, 5×13, 11×17) and the
   kind-gate tables (LIGHT_PET_KINDS/CHAIN_KINDS/TOMB_KINDS) are exactly
   the landed sets.
3. `w4_d2_retire_261_stays_unrowed_until_the_seed_lands` — the
   activation-contract pin: `proj_data(261).is_none()` at this tip (the
   S1-P-i61 seed owns the row). The rowless `<1.5` damage-refusal term
   (`:12448`) is d18's ARM-TOTAL gate (`damage_kind_gate_refuses`,
   `83600da0a` — NOT on this branch's base), so its auto-activation is
   the d18-merge contract, cited not witnessed here; the witness flips
   to a row assertion when the seed lands.