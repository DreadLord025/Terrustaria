# 26h — Projectile aiStyles 171-180 FINAL band census (W4-d18) — SEALED

> Researched 2026-11-21 at branch tip `5e51c66e` (band complete + the
> Damage_CanDealDamage census round), base `a725acb0` (wave-3 complete).
> **SEALED by the PROJ-171 RETIRE round** (tips `83600da0` / `f53de9c7` /
> `dd7b1b0a` / `6e808375` + the seal continuation's cross-lane publishes):
> the band is complete, the Damage-switch remainders
> are ported, the 978 dup is dropped (keep-d12c), the 601 row-shape is
> cross-noted (d6-lane). No further census work is owed on 171-180.
> C# cites are `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` /
> `projectile_ai_misc.rs` / `projectile_ai_empress.rs` at this revision.
> **Code-only mode: no cargo/rustc/rustfmt/tests/graphify invoked.**

## Method

Same shape as 26d (the 111-120 census): the `match data.ai_style` dispatch in
`ProjectilePool::update` plus the KIND-gated `_ if` catch-arms ahead of it
(the frost-shots/empress precedent); kind lists from `SetDefaults`'
`else if (this.type == N)` chain — the 171-180 style writes sit at
`:8354-9820`; arms located through `AI()`'s dispatch (`:36639-36682`).

## FINAL band-status table (171-180) — ALL ARMED + ROWED

| style | C# body (dispatch) | kinds (SetDefaults) | rowed | armed (Rust) | commit |
| --- | --- | --- | --- | --- | --- |
| 171 | `AI_171_HallowBossRainbowStreak` :45451-45558 (:36647-36650) | 873 (:8354-8366) hostile; 931 (:8826-8841) + 1039 (:9807-9820) FRIENDLY weapon variants | ALL 3 | KIND-gated: `projectile_ai_empress::handles` (873) + `handles_fairy_weapon` → `fairy_weapon_tick`/`fairy_weapon_bounce` (931/1039, the C# arm's own kind checks :45462/:45467/:45480/:45551) | main S1-D-4 (873) + e4c69e46 (tail) |
| 172 | `AI_172_HallowBossRainbowPelletStorm` :45569-45604 (:36651-36654) | 871 (:8380-8392) | YES | `172 =>` → `ai_pellet_storm` (projectile_ai_misc) | 8eab7db4 |
| 173 | `AI_173_HallowBossRainbowTrail` :45429-45440 (:36655-36658) | 872 (:8403-8415) | YES | KIND-gated: `empress_proj_tick` | main S1-D-4 |
| 174 | `AI_174_MultisegmentPet` :45345-45427 (:36659-36662) | 883 (:8474-8485), 887 (:8496-8507), 893 (:8531-8542) | ALL 3 | `174 =>` → `ai_multisegment_pet` | 14d08b48 |
| 175 | `AI_175_TitaniumStormShards` :45319-45343 (:36663-36666) | 908 (:8626-8637) | YES | `175 =>` → `ai_titanium_storm_shard` + `shard_group` | 16b95d89 |
| 176 | `AI_176_EdgyLightning` :45058-45095 (:36667-36670) | 916 (:8676-8688) | YES | `176 =>` → `ai_edgy_lightning` + `find_target_with_line_of_sight` reach | bf23ba5a |
| 177 | `AI_177_IceWhipSlicer` :45117-45265 (:36671-36674) | 917 (:8701-8713), 969 (:9169-9181) | BOTH | `177 =>` → `ai_ice_whip_slicer` + `weather_pain_sync_siblings` (post-core pool scan) | cb14160f |
| 178 | `AI_178_FireExplosion` :45267-45317 (:36675-36678) | 918 (:8689-8700) | YES | `178 =>` → `ai_fire_explosion` | d7d631e9 |
| 179 | `AI_179_FairyQueenLance` :45032-45056 (:36679-36682) | 919 (:8714-8726) | YES | KIND-gated: `empress_proj_tick` | main S1-D-4 |
| 180 | `AI_180_FairyQueenSunDance` :44984-45030 (:36683-36686) | 923 (:8757-8770) | YES | KIND-gated: `empress_proj_tick` | main S1-D-4 |

**Verdict: 10/10 styles armed, 15/15 kinds rowed.** The band boundary is
closed at BOTH ends: the 171 tail (the friendly 931/1039 weapon variants the
hostile five-kind empress gate does NOT claim) landed e4c69e46; the 180
empress boundary (923's boss-tethered beam) rides the empress gate. No
style-171..180 kind falls through to `_ => {}`.

## Documented remainders (NOT row/arm gaps)

- 874 DeathAurora is aiStyle **0** (`:8371`) — a pure visual, no `AI()` arm;
  rowed + KIND-gated into `empress_proj_tick` (which no-ops it faithfully).
- 924 FairyQueenHymn (`ProjectileID.cs:2402`) has NO `SetDefaults` arm in
  the C# either (the only 924 site is `Damage_EVP`'s BossNoCheese case
  `:14794`) — it rides the stock aiStyle-0 defaults; unrowed here, outside
  the band's kind census (the R7 spawn wiring owns it if it ever spawns).
- The empress spawner (R7's `EmpressRequest::Projectile` drain + the
  `NpcKind::HallowBoss` 636 row) is UNWIRED on main — `npc_ref`'s
  `is_empress` stays false until R7 merges (the S1-D-4 ANALYSIS note); the
  arms are faithful and witness-tested through `EmpressNpcRef`.
- Damage-side: the 919/923 telegraph invuln (`Damage_CanDealDamage`
  `:12448`) CLOSED this round (`damage_kind_gate_refuses`, ANALYSIS
  "W4-d18 Damage_CanDealDamage"); the 919 line / 923 beam / 872 oldPos-ring
  hitboxes and the BossNoCheese EVP group stay with the damage lane.
- Render-side: the trail rings / tints / 923 beam draw stay with the draw
  lane (S1-D-4 ANALYSIS).

## Census-residual round (18916087, same branch)

The all-branch claim scan found three kinds whose styles are armed on main
but which had NO row: 964 (187), 978 (189), 1106 (193) — rowed verbatim.
The 85/1106 `localAI[0] >= 54` smoke-tail damage gate (`:12460-12467`) and
the full `Damage_CanDealDamage` switch census landed 3783a2ad / 5e51c66e
(ANALYSIS "W4-d18 Damage_CanDealDamage" holds the term-by-term table).

## PROJ-171 RETIRE — the 978 dup drop-hunk (keep-d12c, EXACT publish)

**Verdict (d19 6e80e4201, phase 2):** the 978 Volcano row is a d12c/d18 DUP,
adjudicated **keep-d12c** — d12c's `9e668d0f0` (DOC-SWEEP item 7) is the
superset: it ALONE adds `proj_frames[978] = 5` (`Main.cs:8493`, re-read
verbatim: `Main.projFrames[978] = 5;`). The d18 copy (18916087b) lacks the
frames row. The 189 dispatch arm (`ai_volcano`, main) serves both; the row
is the spawn's only gate. **This branch DROPS its 978 copies** (the commit
after 83600da0); the 964/1106 residuals stay. The 978 spawn declines here
until d12c merges — merge-order, not coverage.

**The EXACT drop-hunk (5 sites, for merger/d12c — drop d18's, keep d12c's):**

* **D1 — `data.rs` const** (was `:825-828`): DROP
  `/// ProjectileID.Volcano (978 ...) ...` doc + `pub const VOLCANO: u16 = 978;`.
  d12c's const (its `:780-788` block, after `LIGHTS_BANE`) survives.
* **D2 — `data.rs` row** (was `:3964-3973`): DROP the
  `// Projectile.cs:9253-9269 — the Volcano explosion (978) ...` comment +
  `VOLCANO => (160.0, 160.0, 189, 3, 30, 0, false, false, false,
  (true, -1, false, -1, 255, 1.0)),` arm. d12c's row (its `:2977-2995`
  block) is value-identical — keep it.
* **D3 — `data.rs` ignoreWater overlay** (was `:4408-4411`): the d18 term
  `|| matches!(kind, INSANITY_SHADOW_FRIENDLY | VOLCANO)` → EDIT to
  `|| kind == INSANITY_SHADOW_FRIENDLY` (the 964 term STAYS — it is d18's
  own residual, NOT a dup). d12c's separate `|| kind == VOLCANO` overlay
  (its `:4373-4375`) survives. **Do NOT drop the whole matches! — the 964
  half is d18-unique.**
* **D4 — `data.rs` melee census** (was `:4482-4484`): DROP the
  `// W4-d18 tail: the Volcano explosion's melee = true ...` comment +
  `| VOLCANO` line. d12c's `| VOLCANO` (its `:4453-4455`) survives.
* **D5 — `projectile.rs` witness** (was `:40949-40956`, the
  `w4_d18_the_residual_rows_carry_the_csharp_set_defaults` test): DROP the
  `let d = proj_data(VOLCANO).expect("978 row"); ... assert!(d.melee ...)`
  block (8 lines); REPLACE with `assert!(proj_data(978).is_none(), ...)`.
  The 964/1106 witness halves STAY. d12c's own 978 witness (its
  `data.rs` tests, the `proj_frames(VOLCANO) == 5` pin) survives.

**NOT dropped (d18-unique, keep):** the 964 InsanityShadowFriendly const +
row + ignoreWater term + witness; the 1106 FoxsparksFlames const + row +
witness; the `ai_volcano` arm (main's, serves 1125 + 978-once-d12c-lands).
**No `proj_frames[978]` row was ever on d18** — that is the superset delta
d12c owns; nothing to drop there.

## Verification

Code-only: every SetDefaults block + AI body re-read at the cited windows;
the dispatch arms checked against the live match block (no duplicate style
arm — the deny(unreachable_patterns) contract); the row witness + gate
witnesses WRITTEN, NOT RUN. Build/test verification deferred to the merge
gate per the task contract.

## PROJ-171 RETIRE — the closeout ledger (this branch)

| phase | commit | scope |
| --- | --- | --- |
| 1 | `83600da0` | the Damage-switch rowless-conditionals (434/451/631/537/261/818/281/511-513/1128/1092/1091/1122 + aiStyles 25/93) ARM-TOTAL + the projPet `:12480` block (`proj_pet_contact_refuses`, the decoded exception ladder); 2 witnesses written not run; ANALYSIS census table updated (the two remainders CLOSED) |
| 2 | `f53de9c7` | the 978 Volcano dup DROP (keep-d12c superset `9e668d0f0`, the d19 verdict `6e80e4201`); the EXACT 5-site drop-hunk published above (D1-D5); the 964/1106 residuals + `ai_volcano` arm stay |
| 3 | `dd7b1b0a` | the 601 CANNON_EXPLOSIVE row-shape cross-note (d6-lane): aiStyle 1 NOT 0, penetrate 1 / alpha 255 the true C# shape, the M7-WIRE-2 row mislabeled; the teleport AI unported; the "death blast" claim withdrawn; 1 witness written not run |
| 4 | `6e808375` | 26h SEALED + the band-171-180 declaration |
| 5 | (this seal continuation) | the cross-lane publishes: the 601 d6 task-seed EXACT C# spec (R-26-13, row-shape + teleport halves), the f25 spray-rows DUP keep-one publish (keep-d18 superset, keep-f25 wiring), the projPet × d43/d33 compose contracts, the 978 D1-D5 d12c-superset re-confirmed at tip `c98bd8937` |

**Witnesses added this retire round (WRITTEN, NOT RUN):**
`w4_d18_the_rowless_conditionals_census`, `w4_d18_the_proj_pet_contact_ladder`,
`w4_d18_the_601_portal_bolt_gate_refusal_cross_note`; the residual-row witness
`w4_d18_the_residual_rows_carry_the_csharp_set_defaults` re-pointed (the 978
half → `proj_data(978).is_none`). **No compiler invoked** — build/test
verification deferred to the merge gate per the task contract.

## SEALED + band-171-180 declaration

**26h is SEALED.** The aiStyles 171-180 band is **10/10 styles armed, 15/15
kinds rowed** (the FINAL table above), boundaries closed at BOTH ends (the
171 friendly-weapon tail `e4c69e46` + the 180 empress boundary). The
PROJ-171 RETIRE round closes the last Damage-switch remainders:

* **Rowless-conditionals:** PORTED arm-total — the kind-literal +
  `proj_data().ai_style` gate shape means every row-less term AUTO-ACTIVATES
  when a d-lane lands the row (the activation contract, verified). No re-port
  is owed when the rows land.
* **projPet `:12480` block:** PORTED (`proj_pet_contact_refuses`) — the
  `PROJ_PET` table + the decoded exception ladder; the rowed attack-minions
  are ALL exceptions (their attack windows keep biting), the inert pets /
  row-less pygmies refuse faithful to C#.
* **978 dup:** DROPPED (keep-d12c) — the row rides d12c's `9e668d0f0`
  superset; the 978 spawn declines on d18 until d12c merges (merge-order,
  not coverage).
* **601 row-shape:** CROSS-NOTED to the d6 wiring/teleport lane (the row is
  M7-WIRE-2's, not d18's; the gate already refuses 601 faithfully).

**Remaining (NOT band gaps, owned elsewhere):** the 874 style-0 visual, the
924 row-less-in-C#-too kind, the R7 empress-spawner wiring, the hitbox/draw
slices (the S1-D-4 ANALYSIS notes), the 601 teleport-AI port (d6-lane), the
978 row merge (d12c). **No census/arm/row work is owed on 171-180.** The
band is declared COMPLETE + SEALED.

## SEAL continuation — cross-lane publishes (phase 5, this branch)

### (1) The 601 d6-gap task seed — R-26-13, EXACT C# spec for the d6 lane

The d6-`7edfcd49` lane (`task/W4-d6-proj-ai-51-60`) owns BOTH halves. Every
cite re-read whole at this seal; the seed is registered in
90-consolidated-findings §2(d).

**Half A — the row-shape fix (`data.rs:2368-2379` today).** The M7-WIRE-2
row (blame `85e1d416e`) reads `(10, 10, 1, -1, TIME_LEFT_DEFAULT, 30, true,
false, false, RESET_STATE)` — penetrate −1 / alpha 0, mislabeled "cannon
ammo 178/601" (178 is ConfettiGun, `ProjectileID.cs:910`). The true C#
`SetDefaults` arm (`Projectile.cs:6502-6510`, re-read verbatim):

```
width 10, height 10 (:6504-6505); aiStyle 1 (:6506, NOT 0);
alpha 255 (:6507); friendly true (:6508); extraUpdates 30 (:6509);
stock penetrate 1 (:774), stock tileCollide true (:775) — no override.
```

Target row: `PORTAL_GUN_BOLT => (10.0, 10.0, 1, 1, TIME_LEFT_DEFAULT, 30,
true, false, false, (false, -2, false, -1, 255, 1.0))` — rename the const
`CANNON_EXPLOSIVE` → `PORTAL_GUN_BOLT` (`ProjectileID.cs:1756`); the two
delta fields are penetrate −1→1 and alpha 0→255. `proj_frames(601) == 2`
is already rowed (`Main.cs:8624`, `data.rs:4805`). The damage gate's 601
refusal (`:12448`) is id-literal — it survives the rename untouched.

**Half B — the teleport AI, UNPORTED.** The crash hook is
`HandleMovement`'s `type == 601` block (`Projectile.cs:18084-18103`):
the `netMode != 2 && alpha > 0` sound gate (`ai[1] != 0` → Item114 else
Item115, `:18086-18096`); `owner == Main.myPlayer` →
`PortalHelper.TryPlacingPortal(this, lastVelocity, base.velocity)`
(`:18097-18100`); then `position += velocity; Kill()` (`:18101-18102`).
The `PortalHelper` spec (`GameContent/PortalHelper.cs`, windows re-read):

* statics `:10-62`: `PORTALS_PER_PERSON = 2`; `FoundPortals[256,2]` init
  −1; `PortalCooldownForPlayers[256]` / `ForNPCs[maxNPCs]`; `EDGES`
  {(0,1),(0,−1),(1,0),(−1,0)}; `SLOPE_EDGES` {(1,−1),(−1,−1),(1,1),(−1,1)}
  NORMALIZED; `SLOPE_OFFSETS` same as Points; `anyPortalAtAll`.
* `TryPlacingPortal` `:216-244`: `vector = velocity/|velocity|`;
  `FindCollision(position, position + velocity + vector*32)` → tile;
  `!SolidOrSlopedTile` → −1; the edge loop runs `halfBrick ? 2 : 4`
  edges, `Dot(EDGES[i], vector) > 0` + `FindValidLine(position,
  (int)EDGES[i].Y, (int)(0 − EDGES[i].X))` → `AddPortal(bolt,
  best*16+8 − EDGES[i]*(halfBrick ? 0 : 8), atan2(edge.Y, edge.X) + π/2,
  (int)ai[0], direction)`; the slope case (`slope != 0`):
  `Dot(SLOPE_EDGES[num−1], −vector) > 0` + `FindValidLine(position,
  −SLOPE_OFFSETS[num−1].Y, SLOPE_OFFSETS[num−1].X)` → `AddPortal(best*16+8,
  atan2 − π/2, …)`; else −1.
* the line helpers: `FindValidLine` `:246-266` (position, position−offset,
  position+offset, first valid wins); `IsValidLine` `:268-282` (the three
  `BlockPortals` side tiles, then `CanPlacePortalOn` ×3 + `HasSameSlope`
  ×2); `CanPlacePortalOn` `:284-291`; `DoesTileTypeSupportPortals`
  `:293-300` (only 496 refused); `BlockPortals` `:302-309` (`active &&
  !tileCut && !BreakableWhenPlacing && tileSolid`); `FindCollision`
  `:311-322` (`Utils.PlotLine` jump:false, callback `!SolidOrSlopedTile`,
  last cell ×16).
* `AddPortal` `:324-336`: `SupportedTilesAreFine` gate (`:470-554` +
  `SupportedSlope/Halfbrick/Normal` `:555-583`); `RemoveMyOldPortal(form)`
  `:338-350` (602 + owner==myPlayer + ai1==form → Kill);
  `RemoveIntersectingPortals` `:351-371`; spawn 602 at position, damage 0,
  owner myPlayer, ai0=angle, ai1=form; direction + netUpdate.
* the 602 portal row (`Projectile.cs:6511-6520`): 10×10, aiStyle 114,
  alpha 255, friendly, tileCollide false, netImportant.
  `GetPortalColor` `:373-392`: netMode 0 → portal!=0 `hsl(0.52,1,0.6)`
  else `hsl(0.12,1,0.5)`; else `hsl((0.5 + player*0.16 + portal*0.08)%1,
  1, 0.5)`; A=66 always.
* OUT of the bolt seed (follow-ups): `TryGoingThroughPortals` `:105-214`
  (the player/NPC teleport traversal) + `UpdatePortalPoints` `:64-99`;
  the item-3384 PortalGun spawn proc (`ItemID.cs:8451`, wiring lane).

**Already ported (do NOT re-port):** the `AI_001` 601 hook
(`:54066-54120`) — alpha fade 15 + the 2-frame cycle —
`projectile_ai001_b.rs:153-166`; the rotation group (`:57597-57608`) —
`:434`. Deferred slices: the light/dust/sound draws (`:54068-54095`,
`:54109-54119`), the Kill 6-portal-dust (`:72454-72468`), the `GetAlpha`
portal tint (`:79116-79118`) — dust-pool/draw lanes.

### (2) The f25 spray-rows DUP — keep-one publish for the merger

Both lanes rowed the same seven sprays (146-149 / 1015-1017) hours apart
on 2026-11-21: f25's closeout `5e83d3ecb` (09:22) and this lane's
`78ffe8e11` (09:41). Both cite the same C# (`Projectile.cs:2350-2397`
four byte-identical 145-copy blocks + `:9633-9636` → `DefaultToSpray`
`:10691-10702`, all re-read verbatim at this seal: 6×6, aiStyle 31,
friendly, alpha 255, penetrate −1, extraUpdates 2, tileCollide false,
ignoreWater true). **Same match-key `146..=149 | 1015..=1017` in the same
`proj_data` match — a keep-both merge is a DUPLICATE-ARM COMPILE ERROR,
not a silent drift. One arm MUST go.**

**KEEP d18's (`78ffe8e11`), DROP f25's (`5e83d3ecb`) — the superset rule
(the 978/d12c precedent, applied here in reverse):**

* d18's row (`data.rs:3408-3411` today) carries the FULL tail tuple
  `(false, -2, false, -1, 255, 1.0)` — **alpha 255** (`:2356` etc.,
  `:10697`). f25's row (`its :3323`) uses `RESET_STATE` — **alpha 0**,
  inheriting the 145 row's pre-existing M3-ITEM-6 dropped-alpha deviation
  its comment claims ("`alpha`/`ignoreWater` dropped"). That claim is
  WRONG for the seven NEW kinds: their C# blocks DO write alpha 255 +
  ignoreWater true; the 145 deviation is 145's alone and must not spread.
* d18's `ignoreWater` overlay (`data.rs:4448-4454`, the
  `matches!(kind, 146..=149 | 1015..=1017)` term) ports `:2360`/`:10701`;
  f25 has NO overlay term — its sprays wrongly collide with water.
* d18's census-flags extension (`data.rs:5020-5025`) + row witness
  `w4_d18_the_solution_spray_rows_are_the_145_shape` (`data.rs:5065`,
  pins alpha/ignoreWater/immunity/class/spawnable) are the superset
  witnesses; f25's `w4_f25_closeout_the_solution_sprays_carry_the_145_row`
  (`projectile_ai_misc.rs:804`) pins only the stats d18 also pins — drop
  it with the row.

**KEEP f25's (NOT dups — different files/functions):** the `ammo.rs`
Solution arm (both Clentaminators 779/5134), the C3/C4 `ai_spray`
conversion wiring (types 2/5/6/7) + its dispatch witness
`w4_f25_the_spray_dispatch_converts_each_solution_type`
(`projectile_ai_misc.rs:861`, re-pointed at the rowed fixture — d18's row
satisfies it). **The `ai_spray` doc-comment conflict:** take f25's
"all eight kinds rowed + C3/C4 armed" shape, keep d18's "147/148/149 stay
no-ops until d1/g42" caveat (both true post-merge). The d1c falling-retire
lane reads this publish: its spray-adjacent census should treat the rows
as CLAIMED (d18) + WIRED (f25), nothing owed there.

### (3) The projPet PROJ_PET table × the d43/d33 minion lanes — compose

The `PROJ_PET` table (`data.rs:4719-4727`, `Main.cs:8728-8852` verbatim,
re-checked at this seal) ALREADY CONTAINS the Pal/Clay/Forbidden minion
ids the d43/d33 lanes are landing: `1093/1094` (PalworldMinionCattiva/
Foxsparks, `ProjectileID.cs:2740/2742`), `1112/1113` (Trusty pair,
`:2778/2780`), `1118/1119` (ClayPotMinion/ForbiddenMinion, `:2790/2792`),
`1095/1096` (the Chillet pets, `:2744/2746`, `Main.projPet` `:8851-8852`)
— the table rows `Main.cs:8774-8779` + `:8851-8852` re-read. **The rows
are ALSO already rowed on this lane's base** (`data.rs:3528-3544`:
1093|1112 → style 67 local-9 alpha-255; 1094|1113 → style 26 alpha-255;
1118 → style 67 local-30; 1119 → style 206 local-30, per
`Projectile.cs:10195-10209/:10210-10221/:10362-10376/:10377-10388/
:10450-10463/:10464-10478` — re-read whole). **Activation on merge:**
`proj_pet_contact_refuses` (`projectile.rs:300-322`) admits 1093/1112/1118
UNCONDITIONALLY (the `:12480` ladder) — their attack windows bite the
moment a spawn proc lands; 1094/1113/1119 refuse contact (C#-faithful:
they are NOT on the ladder — the Foxsparks bites via its 1106 flames
shot, MINION_SHOT, not body contact; the Forbidden minion via its 1120
shot). **Compose contracts for the minion-kind lanes:**

* d43 (`0253bde9` tip) rows the NPC-side distress kinds 695/696
  (`aiStyle 127`) — the NPC axis; the PROJ side is this table + the
  rows above. No overlap, no dedupe: NPC 695/696 ≠ proj 1094/1113.
* d33 (`b08acf463` tip) ports the NPC style-127 machine
  (`npc_ai_style127.rs::style127_step`, the 695 witnesses) — same
  boundary: NPC-side only. The PROJ style-67 arm
  (`ai_freaking_pirates`, `projectile.rs:11696-11704`) gates 1093/1112
  (flag8) + 1118 (flag9) OUT today (deferred, documented there); the
  style-26 slices (`AI_026_A_KINDS`/`AI_026_B_KINDS`) exclude 1094/1113
  today; the style-206 arm is unported. When a d-lane extends those
  arms, the projPet gate needs NO change (kind-literal, the activation
  contract) and the MINION_SACRIFICABLE / MINION_TARGETING_FEATURE /
  minion-slot census rows are already in place (`data.rs:4711-4714`,
  `:4773-4778`, `:4237-4238`).

### (4) The 978 D1-D5 drop-hunks — d12c-superset CONFIRMED at its tip

Re-verified at this seal against `origin/task/W4-d12-proj-ai-111-120`
tip `c98bd8937` (the d12e proj111-retire reads): `9e668d0f0` IS an
ancestor (`git merge-base --is-ancestor` exit 0). The five d12c sites
all present: the const `VOLCANO` (`its data.rs:788`), the row
(`:2984-2995`, value-identical to the dropped d18 copy — 160×160/189/
pen 3/30/0/false×3/(true,−1,false,−1,255,1.0)), the ignoreWater overlay
(`:4375`), the melee census (`:4455`), `proj_frames[978] = 5` (`:4882`,
the superset delta d18 never had) + its witness (`:5113-5119`, pins
style/timeLeft/penetrate/local-pair/melee/frames). **The D1-D5 hunk
above is COMPLETE and EXACT — the merger drops d18's five sites (already
dropped on this branch: `data.rs:825-829` const-site comment, `:3975`
row-site comment, `:4413-4418` the 964-half-kept overlay, melee census,
`projectile.rs:40976-40979` the `proj_data(978).is_none` pin) and keeps
d12c's whole.** Post-merge the `is_none` pin INVERTS (the row exists) —
the merger must flip that one assert to d12c's own witness or drop the
line; the 964/1106 halves stay. The item-121 Volcano spawn proc already
landed on the d12 branch (`7cf1281c5`, DOC-SWEEP FINAL item 1) — the 978
spawn activates when d12 merges, not before (merge-order, not coverage).
