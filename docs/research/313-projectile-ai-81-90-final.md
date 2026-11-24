# 313 — W4-d9 FINAL: the projectile aiStyles 81-90 band table

> Numbered **313** per the FROZEN research-number collision registry
> (`150-research-number-registry.md`, W4-d54, origin/task/W4-d54
> `a15cb634b`) §8.1 row 6: d9's `281-projectile-ai-81-90-final.md`
> (add `88677f9de` 15:55:17) self-picked into f26's assigned 281
> (addendum-7h) → re-issued **313**. Chain: 281→313. The original
> self-pick rationale (round-2 block ended at 280, "Numbers 281+ are
> FREE", a fresh all-tip sweep of 358 tips / 21 234 rows found max 280)
> is SUPERSEDED by the §8.1 assignment.
>
> Branch `task/W4-d9-proj-ai-81-90`, base `a725acb0` (wave-3 complete),
> 2026-11-21. CODE-ONLY round: every witness is WRITTEN NOT RUN (no
> cargo/rustc/tests/rustfmt/graphify invoked). This is the band's final
> ledger — one row per style arm, the port state, the commit hash, the C#
> anchor, the witness count, and the cross-lane composition notes. The
> per-style census lives in the module doc (`projectile_ai_styleNN.rs`)
> and the commit messages.

## The band table (all ten arms ARMED; five ported here, five main's)

The 81-90 band splits cleanly: the **five odd styles (81/86/87/89/90) were
gaps on main** — d9 ported the row + machine + dispatch for each. The
**five even styles (82/83/84/85/88) were already armed on main** (the
Moon Lord / pillar laser family); d9 censused them and left them
byte-identical. Every style's dispatch arm is present in
`projectile.rs`'s `match data.ai_style` (the `81 =>` :17938 … `90 =>`
:18011 block, the even arms interleaved).

| style | kind(s) | C# arm | port fn | commit | witnesses | owner / notes |
|-------|---------|--------|---------|--------|-----------|---------------|
| 81 | InfluxWaver 451 | inline `Projectile.cs:31001-31213` | `ai_influx_waver` + `influx_waver_move` | `3cb660138` | 7 | **d9** — the four-leg melee wave (launch fade / pulse / homing teleport / strike decay); the on-hit ladder `:14051-14065`, the damage window `:12448`, the tile bounce `:18111-18117`; ONE state-bearing draw (the teleport `NextDouble` :31128), the dust gates consumed |
| 82 | PhantasmalEye 452 | inline `:31214-31321` | `ai_phasmal_eye` | main | — | **main** — the Moon Lord hand's homing eye; row `data.rs:2477`; d9 census only, no edit |
| 83 | PhantasmalSphere 454 | inline `:31322-31376` | `ai_phasmal_sphere` | main | — | **main** — the flood orb; row `data.rs:2478`; d9 census only |
| 84 | laser family: PhantasmalDeathray 455, ChargedBlasterLaser 461, LastPrismLaser 632, MoonlordTurretLaser 642 (+ StardustSoldierLaser 537 UNROWED) | inline `:31377-31757` | `ai_phasmal_deathray` / `ai_charged_blaster_beam` / `ai_last_prism_laser` / `ai_moonlord_turret_laser` (kind-split) | main | — | **main** — the `84 =>` arm splits by kind (`projectile.rs:17877`); rows 455/461/632/642 present; **537 has NO row on any pushed tip** (the row-wave remainder, below) |
| 85 | MoonLeech 456 | inline `:31758-31826` | `ai_moon_leech` | main | — | **main** — the leech latch; row `data.rs:2480`; d9 census only |
| 86 | CultistBossIceMist 464 | inline `:31827-31900` | `ai_ice_mist` + `ice_mist_colliding` | `eb1f19b60` | 6 | **d9** — the rotating spawner (ai1=1, 30-tick child) / six-spoke mist; the `Colliding` 464 arm `:16095-16108` wired into PVE+EVP; 24 NextDouble/tick mist census; **wiring 5** drains the c1 boss request (`68b1ebca2`) |
| 87 | ClingerStaff 482 | inline `:31901-31936` | `ai_clinger_staff` + `clinger_staff_sibling_kill` | `28baa126b` | 5 | **d9** — the wall pins + keep-one; `StatusNPC` 39/420 `:11665`, flyer half-damage `:12951`; ZERO explicit draws (the dust curtain is the body-skip) |
| 88 | VortexLightning 580 (+ CultistBossLightningOrb 465 / Arc 466 UNROWED) | inline `:31937-32261` | `ai_vortex_bolt` | main | — | **main** — the `88 =>` arm runs the 580 wrap-latch/tile-stop/wand step (`projectile.rs:18081`); row `data.rs:2476`; **465/466 have NO row on any pushed tip** (the C# arm's `type==465`/`type==466` sub-blocks are the row-wave remainder) |
| 89 | CultistRitual 490 | inline `:32262-32386` | `ai_cultist_ritual` | `a9035b6fd` | 7 | **d9** — the alpha ramp + bursts + 300-tick 454/521 spawn + 320 kill + host validity; the kept dust draws consumed; **wiring 1** spawns the summons for real (`18b2f4916`) |
| 90 | MagicLantern 492 | inline `:32387-32514` | `ai_magic_lantern` + `is_tile_spelunkable` | `667e1f0c2` | 8 | **d9** — the light pet (death-kill + keep-alive + tileCollide flip + 10-tick spelunker scan + hover + rotation/frame); the `tileSpelunker` bitset (`Main.cs:9420-9467`+`:9651`, `IsTileSpelunkable` :21783) is the **canonical** spelunker form (g78 §4, below); **wiring 2** grants the buff + spawns (`c8c562cc3`) |

## The wiring lanes (landed on this branch, feeding the band)

| lane | C# anchor | port | commit | notes |
|------|-----------|------|--------|-------|
| wiring 1 — the ritual summons | `Projectile.cs:32346-32356` (the 454/521 spawn), `NPC.cs:14414-14470` (the d40 dragon rows), `NPC.cs:15768-15779` (the 521 squidhead) | `drain_ritual_npc_spawns` resolves the vanilla ids through `kind_from_net_id`, `NPC.NewNPC` Bottom placement + the `ai[1]` index write-back; the 454-459 dragon rows + the unrowed 521 ported; render sheets/portraits/health-bar scale + net seeds + `ALL_KINDS` 270→277 | `18b2f4916` | the drain-skip retires; 3 witnesses (1 `npc.rs` defaults + 2 `projectiles.rs` drain) |
| wiring 2 — the MagicLantern 3043 use | `Player.cs:53242-53245` (ApplyPetBuffs 3600 grant), `:11369-11377` (the buff-152 arm), `:18485` (the per-tick reset) | the 3043 grant joins `MINION_STAFF_PET_BUFF_ITEMS`; the buff-152 arm sets the dedicated `magicLantern` bool (`:11372`, NOT the pet flag) + spawns the 492 through the data-carrying seam at the player centre; the generic `:49437` shoot tail spawns the pet on use (shootSpeed 0) | `c8c562cc3` | the rowless `pool.spawn` left the slot inert — the `proj_data` dispatch now runs the aiStyle-90 keep-alive; 2 witnesses (1 `buff_effects_accessory.rs` + 1 `editing.rs`) |
| wiring 3 — the ClingerStaff 3014 use | `Player.cs:48591-48642` (ItemCheck_Shoot's 3014 arm) | the `CanHitLine` point-walk toward the reach-clamped cursor, the down scan (first solid / platform / planter 380, ≤30 tiles) + the up scan from the start row, the 15-tile gap cap, ai0=wall-top ai1=height (the aiStyle-87 pins `Projectile.cs:31903-31904`), the flag8 no-ceiling refusal; zero spawn velocity + the class-scaled magic damage, the tile-grid X snap | `f133cce47` | 2 witnesses (`editing.rs` corridor + refusal) |
| wiring 4 — the InfluxWaver 2880 use | `Player.cs:49405-49408` (the ai7=-1f launch-fade seed) | the seed rides the generic `:49437` shoot tail's `ai[0]` (the aiStyle-81 arm's `:31005` entry leg; without it the wave skips the fade and starts mid-pulse), zero RNG draw; progression verified — the Martian Saucer 395 drop (`loot_db_gen.rs:1292`) + the Zenith 4956 recipe ingredient (`recipes_gen.rs:17780`) | `7a418c636` | the item-2880 row + the 451 projectile id ride the d19/d20 bands (`itemdata.rs:3046`, `projectile: Some(451)`); 1 witness (`editing.rs`) |
| wiring 5 — the IceMist 464 drain | `NPC.cs:64914` (`NewProjectile(..., 464, dmg, 0f, Main.myPlayer, 0f, 1f)`), `Projectile.cs:10876-10877` (the box re-centre) | the c1 cultist adapter queues `HostileShotAi{kind 464, ai0 0, ai1 1}`; THIS branch owns the drain side — the `ai_shots` pass spawns it with the LOCAL owner (`spawn_hostile_ai_owned`, the fishron-bolt precedent; the generic drain's hostile 255 would leave the spawner to the hostile-local gate only) + re-centres the 60×60 box on the request point; the pool's aiStyle-86 dispatch then runs the rotating spawner (the 30-tick mist children ride `wants_dart_spawns`); every other kind keeps the verbatim generic drain | `68b1ebca2` | 2 witnesses (`npcs.rs` drain-owner + pool-leg) |

## SEAL item 2 — the e3 34-STAGED d9 dust-skip censuses (activation contract)

The W4-e3 PROJ-DUST master census (`125-proj-dust-coverage.md` on e3,
renumbered **231** per the registry §3) ported 34 STAGED emitters, five of
which activate at the **d9 merge** (the e3 RETIRE re-verified all five at
d9 `7a418c636`; this seal re-verifies them at the moved tip `68b1ebca2` —
every machine's RNG census header is intact). The e3 emitter bodies live
on e3's `projectile_dust.rs` (NOT on this branch); activation is a
merge-time drop-in at each machine's cited skip slot, per the e3
"staged-emitter activation contract." **The merge order is the activation
order.**

| e3 # | site(s) | kind / style | e3 emitter | shape | d9 skip slot (the census it replaces / extends) |
|------|---------|--------------|------------|-------|------------------------------------------------|
| 5-8 | 31017, 31053, 31149, 31207 | 451 / 81 | `style81_influx_spark_451` (one body, 4 call gates) | **REPLACE** | style81's consumed-draws census (the launch/pulse `SelectRandom(226,229)`+`Next(6)` gates, the homing `Next(3)`, the strike `SelectRandom`) — the emitter call supersedes the kept explicit draws |
| 9 | 31896 | 464 / 86 | `style86_ice_mist_ring_464` | **REPLACE** | style86's 24-NextDouble census (the six-spoke `RandomVector2(-8,8)` ×2 dusts ×6 spokes) — the emitter supersedes |
| 10 | 31928 | 482 / 87 | `style87_clinger_curtain_482` | **ADD** | style87 is a ZERO-explicit-draw machine (the dust curtain rolls nothing) — the emitter ADDS the pool internals after the kept gates |
| 11-12 | 32305, 32330 | 490 / 89 | `style89_ritual_ring_490` + `_puff_490` | **REPLACE** | style89's kept draws (the alpha-0 burst `Next(2,4)`/`Next(-10,11)`×2/`Next(8)` + the 60-tick burst `Next(1,3)`/`Next(-10,11)`×2/`Next(2)`) — the emitter supersedes |
| 13 | 32426 | 492 / 90 | `style90_lantern_spark_492` (per-cell) | **ADD** (the Next(4) head stays d9's) | style90's Next(4)-head-kept census — the per-cell `Next(4)` roll is consumed for EVERY cell (the `&&` head) and STAYS d9's; the emitter ADDS the dust-204 body after the kept gate |

The replace-vs-add split is the e3 contract's: the three kept-draw
machines (81/86/89) REPLACE, the two zero-draw / head-kept machines
(87/90) ADD. Sentinel behaviour is the pool's (post-writes land on slot
6000); `customData`/`shader` writes are the documented dropped fields. No
d9 code change is owed here — the censuses are the wiring contract the
merger executes when e3's emitters land.

## SEAL item 3 — the g78 spelunker bitset CANONICAL (the d13-delete dedupe)

`Main.IsTileSpelunkable` (`Main.cs:21783-21798`) has **four copies** across
the pushed tips (g78's `77-wiring-attach-contracts.md` §4 censused them).
**d9's `projectile_ai_style90` copy is the CANONICAL keep-one target** —
the 12-word `TileSet::from_words` bitset (the efficient form) + the 185/186
frame windows, both `pub` so the siblings can re-export:

| copy | where | form | dedupe disposition |
|------|-------|------|--------------------|
| **d9** | `projectile_ai_style90.rs:69` `pub static TILE_SPELUNKER: TileSet` + `:86` `pub fn is_tile_spelunkable` | the 12-word bitset + the 185/186 windows | **CANONICAL — the keep-one survivor** |
| g78 | `spelunker_helper.rs:68` `TILE_SPELUNKER` + `:85` `TILE_SPELUNKER_IDS` + the fn | the d9 bitset VERBATIM (g78's wiring phase) + the windows | **deletes** → `pub use crate::projectile_ai_style90::{TILE_SPELUNKER, is_tile_spelunkable};` (the `check_spot` call site unchanged) |
| d13 | `projectile_ai_pets124.rs:68` `is_tile_spelunkable` | an inline `&[u16]` 49-id table + the windows | **deletes** → `use crate::projectile_ai_style90::is_tile_spelunkable;` (the call site :588 unchanged) — **the d13-delete** |
| d56 | `tr_render::animated_tiles::is_tile_spelunkable` (:165) | the tr-render copy (no tr-core dep) | **stays** (the crate-boundary exception — tr-render CANNOT depend on tr-app) OR the bitset words move to a shared hand-transcribed `tr_core` module (NOT tiledata/, the generated-files rule); the merger's call, the `SWAYS_IN_WIND_BASIC` precedent |

The 185/186 frame windows are IDENTICAL in all four copies (`frameY == 18
&& frameX in 576..=882` / `frameX in 864..=1170` inclusive,
`Main.cs:21789-21796`) — the dedupe is lossless. The equivalence proof is
g78's `w4_g78_the_spelunker_table_census` (bitset == 49-id-list for EVERY
id in 0..TILE_COUNT); d9's own `w4_d9_the_spelunker_set_and_frame_exceptions`
pins every census id + the frame edges. **These are merge-time hunks on
g78/d13/d56's branches — d9 publishes the canonical form + the verdict; it
does not edit the siblings.** d9's bitset is already `pub`, so the
re-exports compose with no d9 change.

## SEAL item 4 — the band-81-90 census (the row-wave remainder)

The band's **rowed + armed** kinds are complete for the ten styles EXCEPT
three unrowed kinds the C# arms reference but no pushed tip rows:

- **537 StardustSoldierLaser** (aiStyle 84, `Projectile.cs:6124` row,
  `ProjectileID.cs:1628`): the `84 =>` kind-split has no 537 arm; the
  generic `ai_phasmal_deathray` fallback would run it, but the row is
  absent on every tip (the all-tip `data.rs` scan: zero `537 =>` /
  `STARDUST_SOLDIER_LASER` hits). **Row-wave remainder.**
- **465 CultistBossLightningOrb** + **466 CultistBossLightningOrbArc**
  (aiStyle 88, `Projectile.cs:5386`/`:5396` rows, `ProjectileID.cs:1484`/
  `:1486`): the `88 =>` arm's C# `type==465`/`type==466` sub-blocks
  (`:31939-32045` / `:32046+`) are unported — the port's `ai_vortex_bolt`
  runs only the 580. Zero `465 =>`/`466 =>`/`CULTIST_BOSS_LIGHTNING_ORB`
  hits on any tip. **Row-wave remainder** (the c1 cultist lane's boss
  lightning is the natural owner; d9 censused, did not claim).

These three are recorded, not owed by d9 (the band's five gap-styles are
the d9 claim; the even-style remainders ride the row wave / c1). The
454-459 dragon rows + the 521 squidhead (wiring 1) are the band's NPC-side
adds, already landed.

## The witness ledger (WRITTEN NOT RUN — code-only mode)

Counted from the landed code (the `w4_d9_*` family, `git grep -c "fn
w4_d9_" HEAD` per file):

- **Module witnesses: 33** — `projectile_ai_style81.rs` ×7,
  `projectile_ai_style86.rs` ×6, `projectile_ai_style87.rs` ×5,
  `projectile_ai_style89.rs` ×7, `projectile_ai_style90.rs` ×8.
  (7+6+5+7+8 = 33.)
- **Wiring witnesses: 10** —
  - wiring 1 ×3: `npc.rs` ×1 (the summon defaults) + `world_sim/projectiles.rs`
    ×2 (the dragon-head / squidhead drains);
  - wiring 2 ×2: `buff_effects_accessory.rs` ×1 (the buff-arm bool+spawn) +
    `world_sim/editing.rs` ×1 (the use grants+spawns);
  - wiring 3 ×2: `world_sim/editing.rs` (the corridor + the no-ceiling refusal);
  - wiring 4 ×1: `world_sim/editing.rs` (the launch-fade seed);
  - wiring 5 ×2: `world_sim/npcs.rs` (the drain-owner + the pool-leg children).
  (3+2+2+1+2 = 10.)

**Total: 43 witnesses** (33 + 10), every one WRITTEN NOT RUN. The
`editing.rs` ×4 split is wiring 2/3/4 (1+2+1); the `npcs.rs` ×2 is wiring 5.

## Merge notes (the sibling lanes)

1. **Base staleness (a725acb0 vs origin/main 051600da8):** d9 branched from
   `a725acb0` (wave-3 complete); origin/main has since absorbed
   `S1-L-11g-recipe-freeze-guard` (`051600da8`, the craft freeze guards).
   The two-dot `git diff origin/main..HEAD` shows the craft.rs /
   ROADMAP S1-L-11g deltas as REMOVALS — that is base-staleness, NOT a d9
   edit (the three-dot `git diff origin/main...HEAD` lists ONLY d9's 22
   task files, no craft.rs). **At the d9 merge, keep main's S1-L-11g
   craft.rs + anchors row + ROADMAP row; d9 never touched them.**
2. **c1 ⨉ d9 (the IceMist drain):** c1 (`f61243c08`) wires the boss
   439/aiStyle-84 and queues the 464 spawner request; d9's wiring 5 owns
   the drain side (the local-owner spawn + the box re-centre). No code
   overlap — the request shape (`HostileShotAi{kind 464, ai0 0, ai1 1}`)
   is the contract; c1's queue witness
   (`the_ice_mist_fires_at_four_and_ends_at_124`) pins the request, d9's
   two pin the drain + pool leg.
3. **d40 ⨉ d9 (the dragon rows):** wiring 1 ports the d40 454-459 rows
   (`NPC.cs:14414-14470`) + the unrowed 521; if d40's own tip rows them,
   keep-one at merge (the rows are verbatim-identical by construction).
4. **e3 ⨉ d9 (the dust activation):** the five STAGED emitters activate
   at the d9 merge per SEAL item 2 — the replace/add split is the
   contract; no d9 code change owed.
5. **g78/d13/d56 ⨉ d9 (the spelunker dedupe):** d9's bitset is canonical
   (SEAL item 3); g78's + d13's copies delete to re-exports at their
   merges, d56's tr-render copy is the recorded exception. No d9 change.
6. **d19/d20 ⨉ d9 (the InfluxWaver item):** the item-2880 row + the 451
   projectile id ride d19/d20 (`itemdata.rs:3046`); d9's wiring 4 reads
   them. The 451 proj_data ROW is d9's (the d19/d20 tips carry the item
   row, not the projectile row — the all-tip scan found no
   `(16.0, 16.0, 81,` on d19/d20). No keep-one conflict.

## RETIRE declaration (2026-11-21, the closing pass)

**Declaration: the W4-d9 projectile aiStyles 81-90 band is DONE.** The
five gap-styles (81/86/87/89/90) are ported (row + machine + dispatch),
witnessed (43 WRITTEN NOT RUN, code-only) and wired (the five item-use /
drain lanes). The five even styles (82/83/84/85/88) are main's, censused
and left byte-identical. The three unrowed kinds (537/465/466) are
recorded row-wave remainders, not d9 obligations. The e3 dust-activation
contracts + the g78 spelunker-canonical verdict are published for the
merger. No further style claims ride this branch; the residue is assigned
above. This doc is SEALED: later tip movement is recorded as receipts, not
re-openings.

### Verification

CODE-ONLY mode: no cargo/tests/rustfmt/graphify invoked (the task's
constraint). Every census re-read against the cited C# windows; the seam
call sites checked against the actual signatures by read/grep
(`spawn_hostile_ai_owned`, `HostileShotAi`, `Npc::new`, `flow_app`,
`World::new`, `Player::spawn`, the pool `update` signature, the
`projectile_ai_style90` pub bitset). The witnesses are written to the
house pattern (the mirrored-stream / literal-pin conventions) and marked
WRITTEN NOT RUN in every commit. The merge wave runs `cargo test
--workspace -- --skip migrates_version_2_kinds_to_item_ids` per AGENTS.md.
The registry-number sweep (21 234 rows, max 280) was the original 281
self-pick's evidence (SUPERSEDED — registry §8.1 row 6 re-issued this
doc to 313); the all-tip `data.rs` scans (zero 537/465/466 rows, zero
d19/d20 451 projectile rows) are the remainder / keep-one evidence.
