# 91 — G4 DEPRECATED: the fighter-family canonical record (d22) + the final KEEP/DROP symbol table

W4-d22 retire phase R3, branch `task/W4-d22-npc-ai-styles-3-6`
(`fb99a047` R1 + `28866c7c` R2 on the DONE base `bcd42ddd`). Code-only
lane: no cargo/tests invoked.

> **CORRECTION (the FIGHTER-CORRECTION, the g4 seal `e5915bfab`):**
> the §0 "no delta at `a9369ec38`" re-probe was STALE — g4's origin
> tip was `55840d9a` (09:16, before this 12:39 probe) and later its
> seal `e5915bfab`. The AI-arm KEEP/DROP table below stands FINAL;
> the post-probe g4 delta is adjudicated in
> `91-fighter-family-bible.md` §0/§2 (FINAL): f35 owns the DropHeals
> heal seam (g4's duplicate retired by g4's own seal), merge d22+f35
> never g4-wholesale, cherry g4's spawn_coins LIVE-inputs hunk + the
> Thief/Skeleton scan witnesses; DROP g4's scan type-bonus arms
> (d22 `:498-513` identical) + its pick-path Peon smash (KEEP d22's
> `kill_tile` form).

## 0. The deprecation

**`task/W4-g4-npc-style3-fighter` is DEPRECATED as a merge source.**
Its tip `a9369ec38` (residual docs, the d22r RESUME keep-both
fold-list) is fully consumed: the W4-d22r reconcile phases 1-4
(`0cfd08ac`/`4150d374`/`e6c83bef`/`bcd42ddd`) folded every g4 code hunk
into THIS branch's canonical style-3 core, per the keep-both contract
g4 itself published (`research/91-addendum-d22r-fold-list.md` §7 on
the g4 branch). The retire re-probe (2026-11-21) confirms
`origin/task/W4-g4-npc-style3-fighter` still points at `a9369ec38` —
**no delta since the reconcile; no re-fold needed.**

**d22 is canonical for the fighter family (aiStyle 3).** The merger
must take the d22 branch's versions of every symbol in the KEEP table
below; the g4 branch contributes NOTHING further. Do NOT merge g4
after d22 (its pre-union scan signature, its pre-axis `npc.rs` dispatch
and its wave-3-shaped files would revert the axis claim); do NOT
delete the g4 branch from this lane (another agent's branch — the
Lead/owner retires it after the d22 merge).

The g4-unique DOCS (`research/91-doc-sweep-fighter.md`,
`91-addendum-d22r-fold-list.md`) stay valuable as the audit trail; they
live on the g4 branch and are cited here by blob. Their CONTENT is
superseded by: the reconcile record (ANALYSIS "W4-d22r"), this note,
and `91-d22-roster-gap-table.md` (R2). If the Lead wants the audit
trail on main, cherry-pick the two doc files ONLY (no code).

## 1. The final KEEP/DROP symbol table

Every g4 commit (`f384797c4`..`a9369ec38`, 10 commits on the wave-3
base `a725acb0`) adjudicated. **KEEP** = the symbol/hunk landed in the
d22 tree (verified by read/grep at the retire tip, file:line cited);
the merger keeps the d22 version. **DROP** = g4-side scaffolding the
d22 fold replaced (the pre-union shapes); nothing to carry.

### P1 `f384797c4` — the banner-buff strength lane

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| `Player.npc_banner_buff` (the per-tick scene snapshot, `Player.cs:44197`) | KEEP | `player.rs:852` (field), `world_sim/player.rs:588` (the snapshot copy) |
| `Player::banner_buff_for_npc` (`GetBannerBuffEffect` `:20478-20492`) | KEEP | `player.rs:4721` |
| the `Auras.banner` → MonsterBanner 147 grant (`:25552`) | KEEP | `player.rs:7033` + witness `w4_g4_the_banner_aura_grants_the_monster_banner_buff` (`:13247`) |
| the melee strike arm (`Player.cs:44303`, pre-crit/DamageVar) | KEEP | `world_sim/editing.rs:7722` |
| the contact arm (`NPC.cs:31388`, post-DamageVar) | KEEP | `npc.rs:18621-18627` |
| the projectile PVE arm (`Projectile.cs:13353`) | KEEP | `projectile.rs:23019` + witness `w4_g4_the_projectile_pve_banner_buff_multiplies_the_damage_rng_free` (`:24643`) |
| the hostile EVP arm (`:14832`) + `banner_id_to_respond_to` (`FindBannerToAssociateTo` `:11104-11117`) | KEEP | `projectile.rs:743`/`:23520`, the pool snapshot `set_npc_banner_buff` (`:15032`) + witness `w4_g4_the_hostile_evp_banner_buff_cuts_the_damage_rng_free` (`:24698`) |

### P2 `3bb819a37` — the summoner phase writes

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| the 471 flight kb0/def28, ground kb0.35/def26 + localAI[3] zero, pose kb0/def36 (`NPC.cs:56495`/`:56523`/`:56531`) | KEEP | `npc.rs:10880-10893` (flight) / `:10990-10995` (ground) / `:11017` (pose) — the `kb_resist_override`/`defense_override` writes in `update_goblin_summoner` + witness `w4_g4_the_goblin_summoner_phases_write_knockback_and_defense` (`npc.rs:27801`) |

### P3 `c49ff03db` — the confused fighter arms

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| the shooter ai1/ai2 reset (`NPC.cs:59197`) | KEEP | `npc.rs:11880` + `npc_ai_frost_mob_b.rs:327` + witness `w4_g4_the_confused_shooter_never_fires` (`npc.rs:20988`, repointed to RayGunner 382 per the P9 correction) |
| the rifleman leap-clear/mirror/leap-gate/timer (`:58470`/`:58485`/`:58566`/`:58576`) | KEEP | `npc.rs:13324-13373` + witness `w4_g4_the_confused_rifleman_resets_the_leap_and_the_timer` (`:21031`) |
| the engineer/zapper `ai[2]=-60` dispatch resets (`:58986`/`:59048`) | KEEP | `npc.rs:5977` + witness `w4_g4_the_confused_engineer_and_zapper_park_the_work_timer` (`:21070`) |

### P4 `53063dd1` — the tall-gate bash

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| `tr_core::edit::shift_tall_gate` (`WorldGen.cs:51723-51775` verbatim) | KEEP | `tr-core/src/edit.rs:828` + witness `w4_g4_shift_tall_gate_opens_the_closed_gate` (`:1551`) |
| the drain's gate-388 arm (389 shift, occupancy gate, refusal `ai[3]=num57` `:60077`, PlaySound(8)) | KEEP | `world_sim/npcs.rs` (the `wants_door_bash` drain, kind==388 arm) + witness `w4_g4_the_door_bash_drain_shifts_the_tall_gate` (`:17949`) |
| the S1-D-3z unported stand-in | DROP (retired by the fold — already absent) | — |

### P5 `743d3a02` — the graveyard bash gate (the signature UNION)

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| `Npc::target_zone_graveyard` + ctor false | KEEP | `npc.rs:1732`/`:3102` |
| the sim pre-pass (every slot ← `scene_metrics.zone_graveyard`) | KEEP | `world_sim/npcs.rs:4389-4393` |
| `fighter_obstacle_scan(npc, world, rng)` (the zombie fold's rng-threaded entry) | KEEP | `npc_ai_fighter_scan.rs:271` — the UNION: g4's rng parameter + d22's generalised body |
| g4's PRE-union `fighter_obstacle_scan` body (the zombie-only constants) | DROP | d22's entry delegates to `fighter_obstacle_scan_g` (the axis generalisation); behaviour identical for the zombie family |
| `flag25 = npc.target_zone_graveyard && rng.next_max(60) == 0` (`:59997`) | KEEP | `npc_ai_fighter_scan.rs:473` (ONE shared line, both entries) |
| `fighters_not_discouraged`'s pre-stamp read (`:60302`) | KEEP | `npc_ai.rs:294-302` |
| `update_zombie`'s `&& !self.target_zone_graveyard` chain term | KEEP | `npc.rs:9307` |
| the 4 P5 witnesses | KEEP | `npc_ai_fighter_scan.rs:883`/`:897`/`:915`, `npc_ai.rs` (`w4_g4_the_graveyard_stamp_cancels_the_day_discouragement`), `world_sim/npcs.rs` (`w4_g4_the_graveyard_bash_draws_on_the_update_npcs_stream` — R1 re-derived to the moan-first C# order) |

### P6 `1419f7d1` — the confused GENERIC-walker arms

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| `walker_step`'s buff-31 facing flip (`NPC.cs:77772-77775`) | KEEP — **verified landed** | `npc_ai.rs:100-102` (the aggro arm, before the sprite lockstep + velocity write) |
| `walker_step_ladder`'s buff-31 flip | KEEP — **verified landed** | `npc_ai.rs:192-194` |
| the P6 witness (the VortexSoldier-429 ladder twin per the P9 correction) | KEEP | `npc_ai.rs:1400` (`w4_g4_the_confused_generic_walkers_reverse_the_walk`) |
| a confused hunk in the AXIS core | DROP (never needed — the addendum §2 "NO FOLD NEEDED") | the core's `target_closest` carries the ONE flip (`npc_target.rs:291`); the P6 arms serve the NON-claimed generic riders (the archer/shooter stand-ins, the DD2 walkers, the nebula beast) — both live, no overlap |

### DOC-SWEEP `6dbed20de` + residual `9bed77313` — the census corrections

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| the blood-moon flag24 census correction (the DOOR_BASH_TYPES note + the gate witness note) | KEEP | `npc_ai_fighter_scan.rs` (the corrected note) + `w4_g4_the_blood_zombie_opens_doors_on_a_normal_night` (`:803`) |
| the town Next(10) vs fighter Next(60) door census | KEEP (doc) | the reconcile record (ANALYSIS "W4-d22r") + the g4 doc-sweep §3 (audit trail) |
| the d22-core probe / merge-risk note | DROP (superseded — the reconcile happened) | this note §0 |

### Residual P9 `b85ff57de` — the immunity census

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| `debuff_immune_ids`'s fighter rows (Skeleton/UndeadMiner [20] `npc_buffs.rs:230`, the frost trio [20,44,324] `:281`, GoblinSummoner [31,153] `:277`, PirateCaptain [31] `:408`, the lunar [31] kinds + Corite, the DD2 roster `:347`) | KEEP — do NOT re-fold to `&[]` (the addendum §3 contract) | `npc_buffs.rs:157+` |
| `whip_immune` + BlazingWheel 72 + DD2LanePortal 549 | KEEP | `npc_buffs.rs:533-536`, the tag gate reads it (`tag_effects.rs:207`) |
| the P9 census witness | KEEP | `npc_buffs.rs:1173` (`w4_g4_the_fighter_roster_immunity_rows_match_the_csharp_sets`) |
| the knockback re-census (no code gap) | KEEP (doc) | the g4 addendum §5 (audit trail); the axis core owns the velocity integration |

### Residual docs `a9369ec38` — the fold-list itself

| symbol | verdict | d22 landing site |
| --- | --- | --- |
| `research/91-addendum-d22r-fold-list.md` §1-§7 | DROP as a work order (EXECUTED — phases 1-4 + this retire) / KEEP as the audit trail on the g4 branch | the reconcile record + this note |
| the item-4 cross-notes (spawn_coins midas/blood_moon false stand-ins, the unported DropHeals heart/star seam) | KEEP (cross-note — f35's lane, NOT d22's; g60 owns the death dust) | the g4 addendum §6; unchanged at the d22 tip |
| the 2 g4 anchors rows (the 489 witness + the P9 census) | KEEP — d22 carries its own copies | `anchors.txt:907-908` |

## 2. The g4-unique-KEEP question (the Lead's item 1)

**"confused-generic-walker-arms? your-fold-took-P9 —
g4-generic-walker_step-flips landed YOUR-tree? verify"** — VERIFIED:
both flips are in the d22 tree (`npc_ai.rs:100`/`:192`, read at the
retire tip), the P6 witness is present (`:1400`) with the
VortexSoldier-429 repoint, and the axis core correctly carries NO
second flip (its `target_closest` owns the one flip; the immune kinds
never flip because the P9 table hides buff 31 from
`find_buff_index(31)`). The P6 arms and the P9 rows COEXIST — the
addendum's keep-both contract holds in the merged shape.

## 3. Merger instruction (the fighter family)

1. Merge `task/W4-d22-npc-ai-styles-3-6` (this branch) — it carries the
   canonical aiStyle-3 axis core + the styles 4/5/6 census fixes + the
   whole g4 fold + the retire (the moan-position closure R1, the
   roster-gap table R2, this note R3, the final receipt R4).
2. Do NOT merge `task/W4-g4-npc-style3-fighter` afterwards — every KEEP
   symbol above is already in d22; a g4 merge would conflict-revert the
   union scan signature, the axis dispatch and the pre-axis files.
3. The kind-wire lanes compose (the phase-4 item-3 adjudication,
   unchanged): g8 `82c93cec` (34 kinds ≤230; WallCreeper 164 / Nymph 196
   keep their `update_skeleton` kind arms — disjoint from `claims()`),
   g9 `5d08fdf2` (ten kind-gated rows on the same machines), g10 (at
   base). The shared `npc.rs` kind-match region: disjoint kind sets,
   keep both sides' arms.
4. The d34-d43 kind lanes take `91-d22-roster-gap-table.md` (R2) as
   their E6 input — the fighter_walker_kind extension list is the
   stream-parity obligation; the axis-claim route retires their
   walker_step_fighter stand-in arms for the plain-walker gap kinds.
5. After the d22 merge lands and is verified (the AGENTS.md step-6
   checks), the Lead/owner retires the g4 branch (delete local+remote);
   this lane does not delete another agent's branch.

Verification: code-only — every KEEP row re-grepped/read at the retire
tip; the g4 tip re-probed (`git ls-remote`: `a9369ec38`, no delta);
no cargo/tests/rustfmt/graphify invoked.
