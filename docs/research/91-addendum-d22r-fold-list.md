# 91 — ADDENDUM A: the d22r RESUME fold-list (g4 lane 15 residuals → d22 core)

W4-g4 branch `task/W4-g4-npc-style3-fighter`, residual phase (base DONE
`6dbed20d`). Code-only lane: witnesses WRITTEN NOT RUN (no cargo invoked).
This addendum is the **keep-both contract** the Lead's item (5) asks for:
d22's style-3 core is canonical; g4's P5/P6 hunks must survive its landing.
Publish the exact fold-list below for the d22r RESUME merge.

## 0. d22 tip re-probe (2026-11-21, this phase)

`git ls-remote origin task/W4-d22-npc-ai-styles-3-6` →
**`b24977d9`** (was `c1d26278` at the doc-sweep probe). Four pushed commits
on the wave-3 base `a725acb0`:

* `60712154` — style-3 core (`npc_ai_style3.rs` 454 lines, the generalised
  scan `fighter_obstacle_scan_g`, the GoblinPeon door-smash drain).
* `c1d26278` — style-4 EoC RNG census+fix.
* `204e0651` — style-5 hover family census+fix.
* `b24977d9` — style-6 worm head census+fix.

The style-3 core shape at `b24977d9` (read from `git show`, not the
worktree): `npc_ai_style3::claims` takes the 22 plain walkers (Skeleton,
UndeadMiner, the goblin trio, the pirate pair, GrayGrunt, Scutlix,
VortexSoldier, SolarSpearman, the scarecrow decade, Splinterling); the
zombie family keeps `update_zombie`; the shooters/martian head-arm kinds
(386/389/383/520), the frost siblings and the pillar machines keep their
own fns. The dispatch arm is `npc.rs`'s `3 if claims(kind) =>
fighter_core_step` (the `update_kind_ai` axis).

**The core's scan is STILL the pre-P5 shape**: `flag25 = false` (the
"graveyard zone unported" stand-in, `npc_ai_fighter_scan.rs:428-432` at
`b24977d9`), `fighter_obstacle_scan_g` takes NO rng, and the module header
still says "the Next(60) short-circuits on the unported graveyard zone".
P5's wiring is therefore NOT in d22's tip — the fold below is required.

## 1. The graveyard bash gate (P5 `743d3a02`) — FOLD INTO d22's scan

The C# (`NPC.cs:59997`): `flag25 = Main.player[target].ZoneGraveyard &&
Main.rand.Next(60) == 0`. The zone is LIVE in the port since d120
(`scene_metrics.zone_graveyard`); P5 wired it through the sim pre-stamp.
d22's generalised scan must carry the same wiring:

1. **`Npc::target_zone_graveyard: bool`** (P5, `npc.rs` ctor false) — keep
   the field; d22 does not touch it.
2. **The sim pre-pass** (P5, `world_sim/npcs.rs` `update_npcs`, right after
   the wind pre-pass): stamps every slot from the captured
   `zone_graveyard`. Keep verbatim — d22's `update_npcs` hunks (the gore
   thread, the style-5/6 seams) do not overlap the pre-pass region.
3. **`fighter_obstacle_scan_g` signature**: add the caller's
   `UnifiedRandom` parameter (P5's `fighter_obstacle_scan(npc, world,
   rng)` shape). d22's entry `fighter_obstacle_scan(npc, world)` (the
   zombie fold) must thread the same stream — `update_zombie` already
   passes its dispatch `rng` (P5).
4. **The flag25 line**: replace d22's `let flag25 = false;` with
   `let flag25 = npc.target_zone_graveyard && rng.next_max(60) == 0;`
   (P5, `npc_ai_fighter_scan.rs:304`). Outside a graveyard the `&&`
   short-circuits — ZERO draws (the legacy census holds for every
   non-graveyard tick, including d22's claimed set). Inside one: exactly
   one `Next(60)` per 60-tick bash.
5. **The discouraged chain term** (P5): `update_zombie`'s inline fold
   carries `&& !self.target_zone_graveyard` (`:60302`);
   `npc_ai::fighters_not_discouraged` reads the pre-stamp. d22's
   `fighter_core_step` calls `fighters_not_discouraged` — it inherits the
   term for free; do NOT re-add a local fold.
6. **The module header**: d22's "unported graveyard zone" note is stale —
   replace with P5's wired census (the zone pre-stamp, the one conditional
   draw, the moan-position remainder extended by it).
7. **The witnesses** (P5, written not run): keep all four —
   `w4_g4_outside_a_graveyard_the_bash_draws_nothing`,
   `w4_g4_in_a_graveyard_the_bash_spends_one_next_60`,
   `w4_g4_the_graveyard_roll_cancels_the_normal_night_reset`
   (`npc_ai_fighter_scan.rs`),
   `w4_g4_the_graveyard_stamp_cancels_the_day_discouragement`
   (`npc_ai.rs`), and the end-to-end
   `w4_g4_the_graveyard_bash_draws_on_the_update_npcs_stream`
   (`world_sim/npcs.rs`). The end-to-end drives `update_zombie` (the
   zombie family is NOT claimed by d22), so it survives unchanged; the
   scan-level witnesses call `fighter_obstacle_scan` — keep that entry's
   rng-threaded signature.

**Draw-position note (unchanged from the doc-sweep §2):** the C# rolls the
moans at the AI head BEFORE the scan's bash draw; the port's moans ride
the sim tail AFTER `update_rng`. In a graveyard the bash `Next(60)`
preceds the moan `Next(1000)` — the S1-E-21e remainder, extended by one
conditional draw. d22's "RNG draw-free" claim for the core becomes
"draw-free OUTSIDE a graveyard; one conditional `Next(60)` inside" — the
stream-parity witness pins the position.

## 2. The confused generic-walker arm (P6 `1419f7d1`) — NO FOLD NEEDED

P6 added the buff-31 facing flip to `npc_ai::walker_step` /
`walker_step_ladder`'s aggro arm (the ad-hoc dx-sign facing kinds). d22's
claimed set rides `target_closest` (which already carries the
`:77772-77775` flip), so the core needs NO confused hunk. The P6 arms
stay for the NON-claimed generic riders (the archer/shooter stand-ins,
the DD2 walkers, the nebula beast, etc.). Keep both; no overlap.

**Residual-phase correction (this addendum's commit):** the P6 witness
`w4_g4_the_confused_generic_walkers_reverse_the_walk` used
`VortexHeadcrab` (428) for the ladder twin — but 428 is Confused-immune
(`NPCID.cs:8555-8559`), so its buff-31 add is rejected and the flip never
fires for it in the C#. The witness now uses `VortexSoldier` (429, null
row, `NPCID.cs:8561`). d22 claims the Soldier on the core — the witness
drives `walker_step_ladder` directly (not the dispatch), so it stays
valid either way.

## 3. The confused-immunity census (residual P9) — d22 MUST NOT RE-FOLD

The residual phase fixed `npc_buffs::debuff_immune_ids` (the
`NPCID.Sets.DebuffImmunitySets` rows): the old table folded whole
families to `&[]` where the C# carries rows. The fighter-relevant
corrections:

* **Confused-immune ([31])**: GoblinSummoner 471 ([31, 153]),
  VortexHeadcrab 428, NebulaFloater 420, BrainSuckler 421,
  StardustCellBig 405 / CellSmall 406 / JellyfishBig 407,
  StardustWormHead 402, SolarSpearman 518, SolarCrawltipede 412-414,
  SolarCorite 418 ([20, 24, 31, 323]), PirateCaptain 216, and the whole
  DD2 walker roster (548/550/552-560/561-563/564/565/568-577 → [31];
  Betsy 551 → [24, 31, 323]; skeletons 566/567 → [20, 31]).
* **[20] rows**: Skeleton 21, UndeadMiner 44, the frost trio 143/144/145
  ([20, 44, 324]).
* **Null rows (Confused CAN land)**: the zombie family, the goblin
  invasion four, RayGunner 382, MartianEngineer 386, GigaZapper 389,
  VortexRifleman 425, VortexSoldier 429, Predictor 424, the spiders
  409/410, StardustSoldier 411, the drakomire pair 415/416, sroller 417,
  selenian 419, beast 423, hunter 426, alien 427, the pirate four
  212-215, the scarecrow decade, Yeti/Krampus ([44, 324] — no 31).

d22's core reads `target_closest`'s confused flip through
`buffs.find_buff_index(31)` — the immune table HIDES the buff for the
immune kinds, so the flip correctly never fires for them. **Do not
re-fold the table to `&[]`** (the pre-P9 shape) — that would let Confused
land on immune kinds and desync the walk direction. The P9 witness
`w4_g4_the_fighter_roster_immunity_rows_match_the_csharp_sets`
(`npc_buffs.rs`) pins the rows.

**P3 witness correction:** `w4_g4_the_confused_shooter_never_fires` used
BrainScrambler (381, Confused-immune) — an impossible state. It now uses
RayGunner (382, null row) and adds the scrambler's rejection pin. d22
does not claim either kind (the shooters keep their fns), so no fold.

## 4. The door-open / door-break census (residual items 1-2) — NO CODE GAP

The residual census re-read `NPC.cs:59949-60088` and the three
`WorldGen.OpenDoor` call sites (`:54047`/`:54056` town, `:60061` fighter,
`:64257` unicorn). Findings:

* **Fighters do NOT open doors "normally"** (no town-style 1/10 walk
  roll). The ONLY fighter door interaction is the 60-tick bash
  (`ai[2] >= 60`), and the open is gated on `ai[1] >= 10` (flag28). The
  port's `fighter_obstacle_scan` + the `wants_door_bash` drain are
  FAITHFUL — no missing arm.
* **The blood-moon distinction was INVERTED in the port's comments.**
  flag27 (`:60007`) = `(!bloodMoon || getGoodWorld) && !flag25 & flag24`.
  The flag24 types (the zombie family) reset `ai[1]` every bash on a
  NORMAL night and only accumulate on a BLOOD MOON (or a graveyard 1/60
  roll) — the blood-moon door-BREAK is the flag24 types' privilege. The
  Blood Zombie (489, NOT in flag24) never takes flag27, so it accumulates
  and opens doors on a NORMAL night too. The residual commit corrected
  the `DOOR_BASH_TYPES` doc + the gate witness's note and added
  `w4_g4_the_blood_zombie_opens_doors_on_a_normal_night` (the 489
  normal-night open + the flag24 zombie reset control).
* **d22's GoblinPeon door-SMASH** (`:60049-60056`, the type-26
  `WorldGen.KillTile` arm) is a DIFFERENT mechanic from the open — d22
  claims it in the drain (`world_sim/npcs.rs`, the `NpcKind::GoblinPeon`
  kill arm). Keep it; it does not overlap the open/shift arms. The Peon
  is NOT in flag24, so it accumulates on a normal night and SMASHES
  (destroys) the door rather than opening it.

## 5. The knockback census (residual item 3) — NO CODE GAP

The `StrikeNPC_Inner` knockback half (`NPC.cs:81387-81475`) is ported
faithfully in `tr_core::damage::knock_back_npc` (the soft caps
8/10/12/14/16, the crit ×1.4, the `num4 = damage*10 > lifeMax` gate, the
strong-hit X push + Y pop, the weak-hit outright set with the resist
multiplying twice). The per-kind `knockBackResist` table
(`npc.rs::knock_back_resist`) was re-censused against every g4 fighter
kind's `SetDefaults` block — ALL rows match (the scarecrow decade's
0.4/0.2/0.25/0.35/0.5 pairs, the DD2 roster, the lunar mobs, the pirates,
the frost trio). The AI_003 head's live `knockBackResist` writes
(`:55976`/`:56027` the 461 wet/dry, `:56100` the 586 wet, `:56495`/
`:56523`/`:56531` the 471 phases [P2 landed], `:57223`/`:57227` the 460,
`:57232`/`:57235` the 469) name NO g4-claimed kind except the Summoner
471 (already P2). The "confused-immune fighters" question is the
immunity census (§3), not a knockback gap. d22's core owns the velocity
integration; no knockback fold needed.

## 6. The fallen/star + coin drop census (residual item 4) — CROSS-NOTE ONLY

Per the Lead's scope, item (4) is a cross-note (g60 owns the death-feed
dust, f35 owns the loot drops). Findings for those lanes:

* **Coins** (`NPCLoot_DropMoney`, `NPC.cs:79619-80106`): ported in
  `tr_core::loot::drop_money`, driven by `world_sim/npcs.rs::spawn_coins`.
  TWO false stand-ins remain: `midas = false` and `blood_moon = false`.
  The C# reads `this.midas` (the buff-72 latch, `:92017-92020`) and
  `Main.bloodMoon` (`:79660-79663`, the +0-100 % `Next(101)` bump per
  roll). Both are LIVE inputs the port already carries
  (`npc.buffs.has_buff(72)`, `app.world.blood_moon`). Wiring them adds
  one draw per value roll on the UpdateNPCs stream — the f35/f39 death
  feeds must replay the same order. **This is f35's lane** (the drop
  trees); g4 does not wire it (cross-note only).
* **Hearts/stars** (`NPCLoot_DropHeals` → `NPCLoot_DropCommonLifeAndMana`,
  `NPC.cs:79098-79557`): the common life/mana arm (`:79531-79557`) rolls
  `RollLuck(6)` then a `Next(2)` mana-star (184) / heart (58) pick, gated
  on `lifeMax > 1 && damage > 0` and the player's missing mana/life; a
  second `RollLuck(2)` mana-star arm follows. The per-type `DropHeals`
  switch (`:79101-79194`) names the scarecrows 305-314 (1/4 heart), the
  Splinterling 326 (1/6), the Zombie Elf trio 338-340 (1/5), the
  Gingerbread Man 342 (2/3), the Present Mimic 341 (5-10 hearts), etc.
  **NOT ported** — no `spawn_heals` seam exists. This is the loot lane's
  (f35) remainder; g4 cross-notes it only.

## 7. Merge checklist (the keep-both contract)

When d22r RESUME lands the style-3 core onto a base that carries g4's
P5/P6/P9:

- [ ] Keep `Npc::target_zone_graveyard` + the sim pre-pass (P5).
- [ ] Thread `rng` into `fighter_obstacle_scan_g` + the zombie entry;
      replace `flag25 = false` with the pre-stamp && `next_max(60)` (P5).
- [ ] Keep the `fighters_not_discouraged` pre-stamp read + the
      `update_zombie` chain term (P5); d22's core inherits them.
- [ ] Keep the P6 generic-walker confused arms (non-claimed riders).
- [ ] Keep the P9 `debuff_immune_ids` rows + `whip_immune` (do NOT
      re-fold to `&[]`); keep the corrected P3/P6 witnesses.
- [ ] Keep d22's GoblinPeon door-smash drain (distinct from the open).
- [ ] Keep the residual blood-moon census correction + the 489
      normal-night witness.
- [ ] Update d22's module header: the graveyard zone is WIRED (P5), the
      core is draw-free outside a graveyard / one conditional `Next(60)`
      inside.
- [ ] The four P5 witnesses + the P9 census witness + the residual 489
      witness all stay (written not run; the merge runs them).