# 65 — Projectile.cs AddLight first wave (sites 1-67, W4-c19) — LANDED

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
