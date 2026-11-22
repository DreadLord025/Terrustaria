# 189 — Projectile.cs AddLight first wave (sites 1-67, W4-c19) — SEALED

> Renumbered 65→189 per registry-150's key-65 row (`origin/task/W4-d54-dust-sites-world`,
> `docs/research/150-research-number-registry.md`: "W4-c19 addlight-proj-a
> 01:54:05 → **189**"; the rename row lists no refs — the ROADMAP mention is
> updated in this commit). Original number kept in the round headers below
> for the audit trail.

Branch `task/W4-c19-addlight-projectiles`, base `a725acb0` (main tip at claim
time). Code-only round: witnesses WRITTEN NOT RUN, no compiler invoked.

## Outcome (2026-11-21)

`crates/tr-app/src/proj_lights_a.rs::proj_lights_a` ports sites 1-67 in seven
~10-site commits. **23 ported sites / 32 light contributions, 2 audit-landed
(the ProjLight tail), 42 absent** (unrowed kinds + the rand-gated 409/849/912).
18 witness tests written not run. With W4-e6's sites 68-107, **67 + 40 = 107 —
the Projectile.cs AddLight family is CLOSED**. ANALYSIS "The projectile AI-arm
lights, first wave (S1-E-5, W4-c19)" carries the full register + the two
state-driven deviations (the Storm Tigers / 601 portal read their spawn `ai[0]`)
and the hooks' PlotTileLine light-map skip. anchors.txt + ROADMAP S1-E-5 +
PORT_MAP rows landed at slice close.

## Claim

The **first 67** of `Projectile.cs`'s 107 `Lighting.AddLight` call sites
(`:16655`…`:54071` in line order), the slice the W4-f7 census freeze
(`origin/task/W4-f7-addlight-proj-3` `67110ccf`,
`tests/proj_addlight_census.rs`) pins verbatim and reserves for W4-c19.
Sites 68-107 are W4-e6's LANDED second wave
(`origin/task/W4-e6-addlight-proj-2` `c8b072b6`, `proj_ai_lights.rs`);
67 + 40 = 107 — this slice CLOSES the Projectile.cs AddLight family.

## Already-landed subset (no re-port)

* Sites 1-2 (`:16655` / `:16659`) are the `ProjLight` chain tail
  (`:16153-16661`), already ported by `proj_lights.rs::proj_light` (S1-E-5
  first wave): the `:16659` integer-halved-centre tail is live for every lit
  row; the `:16655` 870 `FloodFillTile` half is the documented skip (no port
  for the light-map flood write; the centre AddLight lands). This round's
  register RE-AUDITS both against the census table, no code change.

## Work plan

* Sites 3-67 (`:17343`…`:54071`): new `proj_lights_a.rs` following the e6
  `proj_ai_lights.rs` pattern — one pure per-slot collector returning
  `Vec<(i32, i32, [f32; 3])>` in C# call order, feeding the ONE
  `entity_lights::EntityLights` pair from `app/frame/draw.rs` beside the
  existing `proj_light` feed slot (e6's feed placement precedent).
* Absent-wholesale rule (e6 precedent): an arm whose kind has no
  `proj_data` row cannot be carried by any slot — registered absent with
  its C# citation.
* RNG-free rule (e6 precedent): rand-bearing halves (dust arms etc.) are
  skipped; a light gated on a `Main.rand` draw is absent wholesale. The
  feed draws no RNG, so its placement cannot move any stream.
* Batches of ~10 sites per commit; witness tests per batch (written not
  run); full 67-site register in the module docs; ANALYSIS/ROADMAP/anchors
  rows at slice close.

## Census pin

The site lines this round ports are frozen verbatim in
`tests/proj_addlight_census.rs` on `origin/task/W4-f7-addlight-proj-3`
(the 107-row table). A decomp refresh that moves any line fails that test
before this port can silently drift.

## Round 2 — the 42-absent re-census (2026-11-21, commits `1b85dfec` / `88ee3165`)

The four blockers the first wave left, re-censused against the pushed d-lanes:

1. **The `!hostile && !noEnchantments` gate fields (site 3, `:17341`).** The
   `SetDefaults` census: `hostile = true` writes **179 kinds**, `noEnchantments`
   **4** (480/1121/1131/1132), `noEnchantmentVisuals` **21** (the outer `:16700`
   gate). The only runtime `hostile` writes are the two classless bomb-death
   flips (`:28247` aiStyle-49 / `:75798` type-281), which the `(melee || ranged)`
   gate already excludes — so the flags are exact kind-derived `SetDefaults`
   facts. They land as `data.rs::set_defaults_hostile / _no_enchantments /
   _no_enchantment_visuals`; `chillet_mount_light` feeds site 3 ahead of the AI
   arms (`:16702` order), the aiStyle-190/191 swords emitting the two rotated
   swing rects (`:17299-17305`). The Chillet `MountData` rows 62-65 already
   existed (`mount.rs`); the feed resolves the owner's active mount type.
2. **The rand-gated 409/849/912.** Re-censused: all three lights sit INSIDE a
   `Main.rand` gate — 409 `:30378` in `Next(3)==0` (`:30358`), 849 `:47846` in
   `Next(9)!=0` (`:47829`), 912 `:48011` in `NextFloat()<num28/2` (`:47983`).
   The e6 RNG rule does NOT allow arming them: the pure feed cannot take the
   draw without moving the stream. They stay absent. (The 590 truffle spore
   `:34226` is the same case — its colour rides `Next(28,42)`; the d12 row
   landed but the rand gate keeps it absent.)
3. **The unrowed kinds now rowed by the d-lanes.** ~20 kinds activated, each
   citing its source lane: d12 (602/612/953/614/636), d13 (658), d17
   (855/1127), d19 (984/1126/933/1100), d20 (1078/1038/1116/1020/985), d21
   (1115/1105/1098/1123/1091/1117). The collector matches `p.kind` directly,
   so an arm is live as soon as its lane's row lets a slot carry the kind; the
   row itself stays the lane's to merge (the keep-one-dedupe convention — this
   branch copies no rows). **Two first-wave misattributions corrected:** site
   45 (`:41775`) is the `type == 985` arm (after the `type != 985` return
   `:41726`), NOT the 973; site 46 (`:42181`) is the `type == 1126` arm, NOT
   the 972 (which has no light arm). The 1091's `moreLightning && noLightning`
   seed-green folds to the teal default (the feed carries no micro-seed flags).
4. **The tiger/601 state-driven deviations.** Re-censused: W4-d7 still gates
   833-835 out of `ai_freaking_pirates` (only 393/394/395/758 + the pets run);
   no lane ported the 601 portal teleport AI (`ai001_b` runs only the
   fade/frame). Both keep their spawn `ai[0]`; the deviations are unchanged.

**Round-2 accounting: 44 ported sites / 53 light contributions, 2 audit-landed,
21 absent** (the still-unrowed kinds + the rand-gated 409/590/849/912). 12 new
witnesses written not run (30 total: the mount gate/census/swings + the
per-lane activations + the conditional row invariant).

## Round 3 — lights-final (2026-11-21, commits `0f8c1c077`…`fb58e88aa`)

The four rand-gated sites ARM IN THE SIM (the `item_lights.rs` named-stream
precedent — the e6 pure-feed rule's documented exception): the AI arms take
their gate draws on the pass's `UpdateProjectiles` stream and queue
`(tile x, tile y, rgb)` intents into `ProjectilePool::pending_ai_lights`; the
world-sim pass drains them into the ONE `EntityLights` pair
(`world_sim/projectiles.rs`, `drain_ai_lights`). The draw-pass feed stays pure.

1. **409 Typhoon (`:30378`, site 11).** `ai_typhoon` now takes the
   `localAI[1] > 10 && Next(3) == 0` gate (`:30358`); the taken branch runs the
   fade/spin/frame block + the light, and censuses the six-dust ring's 6 ×
   (`NextDouble`, `Next(3,8)`) draws (`:30364`). The unconditional-fade
   deviation is RETIRED; the DW-6.1 frame witness re-derived on an always-0
   gate twin.
2. **590 TruffleSpore (`:34226`, site 22).** New `ai_truffle_spore`
   (aiStyle-112 type arm, `:34194-34233`): the 4-tick frame machine (the
   `ai[2]==1` grown strip 3-5), the 15/tick fade, the `Next(28,42)` flicker
   (`n = draw·0.005 + (270−mouseTextColor)/500`, the 0.35-scaled teal), the
   sine hover with the 180 wrap. `Main.mouseTextColor` rides a pool snapshot
   (`set_mouse_text_color`, the `set_disco` precedent). The 590 row is d12's
   (keep-one — the dispatch + seam witness are conditional on it).
3. **849/912 whips (`:47846`/`:48011`, sites 56-57).** New `whip_dust_lights`
   (the AI_165 dust-switch tail): the 849's `t > 0.4 && Next(9) != 0` gate +
   the `Next(2)` particle draw; the 912's `num28 > 0.1 && NextFloat() <
   num28/2` gate + the three `Next(2)` dust-261 draws. The light anchor is the
   curve tip's `CenteredRectangle(p, 30).Center` (truncate `p−15`, add 15,
   truncate /16); the curve is `fill_whip_control_points` with the slot's
   `ai[1]` spin seed (the dust cases call the default overload, NOT the
   collision sweep's forced 1f). Every other whip type's dust case stays
   draw-skipped wholesale — the dust pass's lane (the inter-case stream
   deviation, ANALYSIS).

**The d9/d10/d11 activations (10 sites / 11 contributions).** The pushed
d-lanes rowed nine more absent kinds: d9 451/464/490/492, d10 518/535, d11
567/568/575. Their arms activate in `proj_lights_a` (the collector matches
`p.kind`; the rows stay the lanes' to merge): the 451 alpha-scaled
truncate-first light, the 464 spin/ring pair (the spawn `ai[1]==0` reads the
6-point ring), the 490 ritual (the `flag43` NPC gate folds to the `ai[1]>=0`
slot-state half), the 492 lantern's top-left anchor, the 518 coin portal, the
535 medusa head (the channel-Kill gate folds live), the 567/568 spore-sac head
(`(1−alpha/255)·scale`, dark at the alpha-255 spawn), the 575 nebula sphere.
The 978 Volcano's d12 row is aiStyle 189, NOT the `:34487` trio's 117 — the
third term stays unreachable (a style mismatch, not a missing row).

**Round-3 accounting: 58 ported sites (54 feed + 4 sim) / 64 light
contributions, 2 audit-landed, 7 absent** (502, 1092, 465, 466, 731, the
phaseblade pair, 756 — all still unrowed on every pushed lane). 12 new
witnesses written not run (42 total): the typhoon gate census/fade/seam, the
spore flicker/frames/conditional seam, the whip gate censuses + seam, the
world-sim drain witness, the d9/d10/d11 activation colours.

## Round 4 — the 7-absent re-probe + SEAL (2026-11-21, commit `538369d63`)

The brief's re-probe hints verified against the CURRENT pushed lane tips
(every tip re-fetched; `git show <tip>:crates/tr-app/src/projectile/data.rs`
+ the lane's own commit messages read directly):

| Absent (round 3) | Verdict | Evidence (lane tip) |
| --- | --- | --- |
| 502 Meowmere (site 4) | **NOW ROWED → WIRED here** | d75 `b59e3a874` (part 5): the row `d328fad8c` (`:5732-5742` verbatim, 16x16 aiStyle 8 friendly melee penetrate 5, local immunity 10) + the `:24346-24352` velocity rotation in `ai_ball_of_fire`. d75's own note keeps the "disco AddLight" a skipped effect — THIS feed is its owner (site 4), no double-play |
| 1092 LibrarianSkeletonBook (site 7) | **NOW ROWED → WIRED here** | g5 `96f074959` (phase 5): `LIBRARIAN_BOOK => (16.0, 16.0, 18, -1, 420, …)` (`:10186-10194`) + `ai_librarian_book` wired to the live caster arm + the 54-frame sheet. The `:25394` light precedes the rand frame pick — unconditional, RNG-free |
| 466 CultistBossLightningOrbArc (site 16) | **NOW ROWED → WIRED here** | d53 `c0dc7d37f` tip, row landed `6d09fe617` (batch 3): `:5392-5403` the 580 shape + alpha 255, for the TrailingMode-1 ring + stationary trail dust. The AI_088 466 branch stays UNPORTED there (the dispatch kind-gates `ai_vortex_bolt` to the 580) — the `:32049` light is unconditional after `frameCounter++`, reads none of the machine |
| 465 CultistBossLightningOrb (site 15) | **STILL ABSENT — gate armed, row owed** | d75 `20b2cea61` (part 4) armed `cultist_lightning_latch` (`:31943` first-tick Item121, `kind == 465 && local_ai[1] == 0`) on the in-tree style-88 head, but NO lane rows the 465 (`:5382-5390`: 80x80 aiStyle 88 hostile alpha 255 ignoreWater tileCollide false) — d75's part-5 FINAL census lists it "rows-only owed with gates armed". **Task seed**: the row lands on a d-lane (the c1-cultist-ai lane or the d9 decade); this feed's arm activates the moment a slot can carry the kind |
| 731 ThunderStaffShot (site 31) | **STILL ABSENT** | no row on any pushed tip (whole-branch `git grep` over `data.rs`: zero hits outside c19's own hostile census). **Task seed**: the row is `:7694-7704` (10x10 aiStyle 29, alpha 255, magic+friendly, extraUpdates 1, timeLeft 90); the AI_029 731 arm (`:37774-37822`) is the fade/zigzag machine, the light unconditional after the rand dust |
| phaseblade pair (sites 35/36) | **STILL ABSENT** | d75 part 5 re-confirms the 19-kind set 1059-1072/1074-1076/1101/1102 unrowed (`ProjectileID.cs:192`; the 1073 NOT in the set). g35's `1101 => 372 / 1102 => 373` hits are GLOWMASK rows, not `proj_data`. **Task seed**: the rows are `:10006-10058` (38x38 / 46x46 / the 1074 46x46, aiStyle 3, melee, penetrate −1, local immunity 10); the colour is `Item.GetPhaseColor` (`Item.cs:49053-49168`: eight direct hues ×(0.1..1.05) channels; the 1101/1102 ride `Utils.MultiLerp(miscCounterNormalized, _phaseColors)` — the port's `misc_counter` field exists, `world_sim/player.rs:1085` ticks it, but the 300-wrap `UpdateMiscCounter` is unwitnessed there); the 1074 lights NOTHING (`type != 1074` gate `:39018`) |
| 756 SharpTears (site 62) | **STILL ABSENT** | no row on any pushed tip. **Task seed**: the row is `:7793-7806` (32x32 aiStyle 157, friendly+magic, alpha 255, penetrate 4, local immunity 25); the rowed 961 rides the same style but the `:51490` light gates on 756 specifically |

**The three activations (`538369d63`).** `proj_lights_a` gains a `disco:
[u8; 3]` parameter (the draw-pass `App::disco` snapshot, the 79 ProjLight
`LightParams::disco` precedent) for the 502's `(0.5 + Disco/255)/2` tail;
the 1092 head lights (0.3, 0.25, 0.1) unconditionally; the 466 arm lights
the 580 teal (0.3, 0.45, 0.5) unconditionally (the alpha-255 spawn does not
gate it). The rows stay the lanes' to merge (keep-one — this branch copies
no rows; the conditional row-invariant witness gains 502/1092/466). 3 new
witnesses written not run (45 total): the Meowmere disco fixtures (the
255/0/0 init, full-white, all-zero floor, the mid-cycle byte no-rounding),
the book's state-independent head, the orb-arc's alpha-independent teal.

**Round-4 accounting: 61 ported sites (57 feed + 4 sim) / 67 light
contributions, 2 audit-landed, 4 absent** (465, 731, the phaseblade pair,
756 — each with its task-seed row citation above; the 978 trio term stays
site 26's unreachable third kind).

## Compose receipts (SEAL, all tips re-fetched 2026-11-21)

### e6 (`origin/task/W4-e6-addlight-proj-2` `c8b072b69`) — DISJOINT, keep-one

Both branches fork `a725acb04`. The two waves' code is site-disjoint by
construction (the f7 census freeze splits at `:54071`/`:54173`): e6's
`proj_ai_lights.rs` (535 lines, sites 68-107) vs this branch's
`proj_lights_a.rs` (sites 1-67) — **zero shared kind arms** (e6's kinds
436/634/635/459/442/449/438/350/325/819/1094/1113/613/445/663/665/667/
691/692/693; none appears in this feed's chain). The merge hunks:

* `main.rs`: e6 inserts `mod proj_ai_lights;` BEFORE `mod proj_lights;`,
  c19 inserts `mod proj_lights_a;` AFTER it — adjacent-line union, keep
  both (alphabetical: `proj_ai_lights`, `proj_lights`, `proj_lights_a`).
* `app/frame/draw.rs`: e6's feed loop (the 445 hand anchor,
  `rotated_relative_point`) and c19's feed loop (mounted centre + Chillet
  mount + the round-4 `self.disco` arg) both insert AFTER the `ProjLight`
  feed at the same seam — keep BOTH loops, e6's first (its sites follow
  the AI order after the first wave's). Both are pure; the ONE
  `EntityLights` pair max-merges, so loop order does not change pixels.
* `anchors.txt`: one row each (`fn proj_ai_lights` / `fn proj_lights_a`)
  — union, no collision.
* The `craft.rs` 340-line "deletion" in BOTH branches' `origin/main..`
  diffs is NOT a deletion: it is main's LATER S1-L-11g addition
  (`03711f456`, post-`a725acb04`) absent from both fork trees. The merge
  keeps main's copy; neither branch touches `craft.rs`
  (`git diff a725acb04..HEAD --stat` lists no craft.rs). Same for the
  ROADMAP S1-L-11g row. **No keep-one work is owed on craft.rs.**

### d13 (`origin/task/W4-d13-proj-ai-121-130` `1e915e254`) — PET-light-67 census consumed

d13's `docs/research/243-d13-pet-light-retire-final.md` (§1 + §4) consumed
c19 at the round-2 tip `a160ad9ff` and CONFIRMED the zero-pet census from
both sides. Re-verified at THIS tip (round 4): the seven aiStyle-124 pets
(650/882/888/895/896/898/957) appear in NO column of the 67-site register
(ported or absent) and in NONE of e6's 40 — the 107-site family carries
ZERO pet contributions. The pets' actual light is the
PlotTileLine/CastLightOpen light-map WRITE layer (`:35008-35043`): the
650 `(0.5,0.9,1)*2` self-20/owner-40 casts, the 895 `(1,0.6,1)*1.5` +
`!NotRetro` ×0.8 ladder, the 896 `(1,0.7,0.05)*1.5` same ladder; the
882/888/898/957 cast NOTHING (dark in C# too). That is the SAME standing
deferral as this register's 870 FloodFillTile spread + the hooks'
PlotTileLine rope skips — when the light-map-write task lands it consumes
d13's §4 table + this register's hook rows together. d13's 658 row
activation (site 28) is already live here since round 2. The code-side
overlap is ONE seam: `world_sim/projectiles.rs` — d13 adds
`set_screen_width`/`set_wire_kite_inputs` after the same `set_disco`
anchor where c19 adds `set_mouse_text_color` (adjacent-line union), and
d13's kite-hold/mass-wire block vs c19's `drain_ai_lights` block both sit
after the swing/item-time tail (keep both, c19's drain first — the light
append is order-free). The `projectile.rs` hunks are disjoint ranges
(d13: the 3745-3921 pet/kite machines + the 14472/14911/14996 pool fields
— the SAME three field-block anchors c19 uses for `pending_ai_lights`/
`mouse_text_color`, adjacent-line union; d13's 17397+ kite methods vs
c19's 17561+ typhoon/spore/whip arms — disjoint). NO semantic conflict.

### g18 (`origin/task/W4-g18-addlight-main-1` `22d529c5c`) — DISJOINT

g18 is Main.cs batch 1: a distrust-verbatim audit + the f32 behavioural
oracle `tests/main_addlight_batch1.rs` (211 lines, 5 witnesses) for the
332 Christmas-hook chain — the PORT copy stays e8's in-flight
`tile_lights.rs::christmas_hook_chain_lights` (g18 ports nothing). File
sets: g18 {tests/main_addlight_batch1.rs, ANALYSIS, ROADMAP, anchors} vs
c19 {proj_lights_a.rs, projectile.rs, data.rs, world_sim/projectiles.rs,
draw.rs, main.rs, ANALYSIS, PORT_MAP, ROADMAP, anchors, research/189} —
the only shared files are the three doc ledgers (append-union: g18's
ANALYSIS "Main.cs census batch 1" section, its ROADMAP S1-E-5 tail
sentence, its one anchors row; c19's round-3/4 S1-E-5 tail sentences —
the merger concatenates both tails, neither edits the other's text).
g19 (`9733c82a7`, the Main census freeze `main_addlight_census.rs`) is
likewise file-disjoint. NO kind/site overlap: Main.cs's 5 sites are
outside Projectile.cs's 107.

## The addlight-proj-family bible (SEAL declaration)

**ONE family, three registers, 107 sites, CLOSED.** `Projectile.cs`'s
`Lighting.AddLight` call sites in line order, frozen verbatim by the f7
census (`origin/task/W4-f7-addlight-proj-3` `67110ccfb`,
`tests/proj_addlight_census.rs` — a decomp refresh that moves any line
fails that test before any port can drift):

| Slice | Owner | Module | Sites | Final accounting |
| --- | --- | --- | --- | --- |
| 1-67 (`:16655`…`:54071`) | W4-c19 (this branch) | `proj_lights_a.rs` + the sim arms (`ai_typhoon`/`ai_truffle_spore`/`whip_dust_lights` → `drain_ai_lights`) | 67 | 61 ported (57 feed + 4 sim) / 67 contributions / 2 audit-landed / 4 absent |
| 68-107 (`:54173`…`:78556`) | W4-e6 `c8b072b69` | `proj_ai_lights.rs` | 40 | 20 ported / 23 contributions / 20 absent |
| The `ProjLight` chain itself (`:16153-16661`) | S1-E-5 first wave (landed) | `proj_lights.rs::proj_light` | (sites 1-2 re-audited) | live for every lit row |

* **The two feed rules** (e6 precedent, both waves): an arm whose kind
  has no `proj_data` row is absent wholesale; a rand-gated light is
  absent FROM THE PURE FEED. Round 3's documented exception arms the
  four rand-gated sites IN THE SIM on the `UpdateProjectiles` stream
  (the `item_lights` named-stream precedent) — the draw-pass feeds stay
  pure, so their placement cannot move any stream.
* **The activation contract**: the collectors match `p.kind` directly;
  a d-lane row activates its arm with no further work here. Round 2
  consumed d12/d13/d17/d19/d20/d21 (~20 kinds), round 3 d9/d10/d11 (9
  kinds), round 4 d75/g5/d53 (3 kinds). The rows stay the lanes' to
  merge — this branch copies NONE (keep-one).
* **The 4 remaining absents are TASK SEEDS, not gaps of this family**:
  465 (row `:5382-5390`, the d75 Item121 gate already armed), 731 (row
  `:7694-7704`), the 19-kind phaseblade set (rows `:10006-10058`, the
  GetPhaseColor/MultiLerp colour census above; the 1074 lights nothing),
  756 (row `:7793-7806`). Each activates its site the moment its row
  lands on any lane; the site lines, colours, anchors and gates are
  pinned in the module register. The phaseblade seed carries ONE extra
  prerequisite: the 1101/1102 cycle reads `miscCounterNormalized`
  (`Player.cs:3389` = miscCounter/300) and the port's `misc_counter`
  tick (`world_sim/player.rs:1085`) is a bare wrapping increment — the
  C# `UpdateMiscCounter` 300-wrap (`Player.cs:29295-29302`) is unported;
  the row-owning lane must land the wrap (or fold it) with the rows.
* **The standing light-map-WRITE deferral** (one task, three registers
  cite it): the 870 FloodFillTile spread (site 1's skipped half), the
  hooks' PlotTileLine rope spreads (`:52613`/`:52644`), and d13's pet
  casts (`:35008-35043`, research/243 §4's colour/width/anchor table).
* **The state-driven deviations** (ANALYSIS, unchanged): the Storm
  Tigers/601 portal read their spawn `ai[0]`; the 451 reads its spawn
  alpha; the 464 spawn reads the ring; the 490's flag43 folds to the
  slot-state half; the 535's channel gate folds live; the 567/568 are
  dark at the alpha-255 spawn; the 1091 seed-green folds teal; the
  round-4 466 reads none of its (unported) machine — unconditional.
* **Merge order**: c19 × e6 are independent (disjoint modules, the
  draw.rs/main.rs/anchors unions above); either lands first. The d-lane
  rows land on their own lanes; nothing here blocks them.

**SEAL**: no further census/arm/row work is owed on sites 1-67 by this
lane. The four task seeds belong to the row-owning lanes; the
light-map-write layer is its own task. This note (research/189, ex-65)
is the family's first-wave bible; e6's ANALYSIS second-wave section +
the f7 census freeze are the other two authorities.
