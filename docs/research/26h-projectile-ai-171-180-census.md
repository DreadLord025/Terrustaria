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

## Verification

Code-only: every SetDefaults block + AI body re-read at the cited windows;
the dispatch arms checked against the live match block (no duplicate style
arm — the deny(unreachable_patterns) contract); the row witness + gate
witnesses WRITTEN, NOT RUN. Build/test verification deferred to the merge
gate per the task contract.
