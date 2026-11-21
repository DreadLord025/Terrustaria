# 26h — Projectile aiStyles 171-180 FINAL band census (W4-d18)

> Researched 2026-11-21 at branch tip `5e51c66e` (band complete + the
> Damage_CanDealDamage census round), base `a725acb0` (wave-3 complete).
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
