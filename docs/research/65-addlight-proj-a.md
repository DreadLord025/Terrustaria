# 65 — Projectile.cs AddLight first wave (sites 1-67, W4-c19) — CLAIM

Branch `task/W4-c19-addlight-projectiles`, base `a725acb0` (main tip at claim
time). Code-only round: witnesses WRITTEN NOT RUN, no compiler invoked.

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
