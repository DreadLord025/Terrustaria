# 141 — W4-e9 NPC-sound FINAL: the 27-tail activation census, the f2/f47/d53/g2 co-lanes, the idle-loop census

**Phase:** final census + two fidelity fixes (code-only, no
cargo/tests/rustfmt/graphify; witnesses WRITTEN NOT RUN). Branch
`task/W4-e9-npc-sounds-2`, base `a725acb0` (main). Tips read at probe
(2× `git fetch origin --prune`, 2026-11-21):

| lane | pushed tip | state |
| --- | --- | --- |
| e9 (this) | `c7523984` (after this round) | 232-site claim complete; see §1 |
| f2 `task/W4-f2-npc-sounds-3` | `bcf4c299` | PUSHED, clean, **0 unpushed commits** |
| d33 `task/W4-d33-npc-ai-styles-95-127` | `dcb59b7f` | 15 UNWIRED style modules, **no kind rows** |
| d41 `task/W4-d41-npc-kinds-491-560` | `4ed22f0b` | **claim-only** (1 commit, zero code) |
| d42 `task/W4-d42-npc-kinds-561-630` | `88df5809` | **non-compiling WIP checkpoint** (578-587 enum arms, superseded numbering) |
| d43 `task/W4-d43-npc-kinds-631-697` | `a725acb0` | **zero commits** (branch == main) |
| d53 `task/W4-d53-dust-sites-projectile` | `6d09fe61` | batch 4 wet host LIVE (`fa7b331d`), pushed |
| g2 `task/W4-g2-proj-ai-missing-b` | `81c4ab85` | Shimmer() STAGED (`63a6db26`), unwired, pushed |
| f47 `task/W4-f47-liquid-shimmer-2` | `a2561d6d` | NPC wet-fields + buff-353 + GetShimmered drain LIVE, pushed |

**The prompt's `d33b`/`d41r`/`d42r`/`d43r` branches DO NOT EXIST** — no
local branch, no remote ref, no worktree directory, no `--grep` hit
across `--all` (two fetches). The "ACTIVE porting kinds
541-545/578/583-585/594-601/604-605/612-618/626-628/657/662/686/690/695-696"
have **no landed rows on any pushed tip**: d41's claim commit explicitly
skips 541-545 ("d33 rows") and 521 ("d9b"), d42's WIP preserves only the
578-587 enum shells (its own message: "Does not compile"), d43 never
started, and d33's census test
(`w4_d33_the_missing_styles_have_no_kind`) pins that styles
99-104/111-119/121/122/125-127 have **no `NpcKind`** — the modules key
on the vanilla `npc_type` parameter until the roster wave. The e9b
SandShark branch f2's research/94 §7 looked for also does not exist
(re-confirmed).

## 1. The 27-tail-sites activation census — ALL STAY UNWIRED (correctly)

> **TAIL-ACTIVATION-2 update (§8):** this section's tip table pre-dated
> the roster pushes — see §8 for the re-probe. The verdict survives
> with ONE amendment: :24449 (Antlion 69) is now machine-owned-LIVE on
> f42/g8 (a dup-port, not an e9 activation); d33's wired style-114
> axis (595-601) activates ZERO tail cues (the machine is sound-free).

`npc_sounds_tail.rs` (9b96eff4, `allow(dead_code)` a41b2b72) ports 27
sites on 17 kinds. Per-kind activation status against every pushed tip:

| site(s) | kind(s) | row on any pushed tip? | machine? | verdict |
| --- | --- | --- | --- | --- |
| :24449 Antlion spit Item5 | Antlion 69 | NO (d34 band 1-70 landed 5 kinds only, 69 not among them; no `NpcKind::Antlion` anywhere) | style-19 arm unported port-wide | UNWIRED stands |
| :37268/:37321 teleport Item8 | ShadowFlameApparition 472 / AncientCultistSquidhead 521 | 472 row on **d40 only** (`8fe1d6d3`, unmerged); 521 skipped to d9b (no branch) | style-86 machine unported (d32 covers 85-93 — 86 not in its landed set) | UNWIRED stands; d40-merge consumer note below |
| :40632 SandShark cry | SandShark 542 (+543-545) | NO (d41 claim skips to "d33 rows"; d33's style103 module has NO kind) | d33 `npc_ai_style103.rs` UNWIRED, carries its own `sounds: Vec<(15,4)/(14,542)>` seam | UNWIRED stands; keep-one note below |
| :42983-43001 Pal pain/joy | 695/696 | NO (d43 zero commits) | d33 `npc_ai_style127.rs` UNWIRED, `PAL_SOUNDS` trackables + `out.sounds` | UNWIRED stands |
| :43562-43608 StatueMimic | 690 | NO (d43 zero) | d33 `npc_ai_style126.rs` UNWIRED, `("statuemimic_*", x, y)` seam — pushes the BASE name `statuemimic_laugh`, e9's helper pins `_0` (the bank row); the merge consumer must map laugh→`_0` | UNWIRED stands |
| :43726/:47951/:48062 balloon pop | ClumsySlimeBalloon 686 / WindyBalloon 594 | NO (d43 zero / d42 WIP has no 594) | d33 style125 (686) + style113 (594) UNWIRED, both push `(4, 63)` | UNWIRED stands |
| :44848 PirateGhost spawn | 662 | NO (d43 zero) | d33 `npc_ai_style122.rs` UNWIRED, `(3, 6)` seam | UNWIRED stands |
| :45558/:45759 QueenSlime | 657 | NO (d43 zero) | d33 `npc_ai_style121.rs` UNWIRED, `(2, 167)` + `(65, 1)` seam | UNWIRED stands |
| :47263-47293 BloodNautilus | 618 | NO (d42 WIP: 618 not in the 578-587 shells) | d33 `npc_ai_style117.rs` UNWIRED, `(2, 170)/(2, 172)` seam | UNWIRED stands |
| :48327/:48473/:48545 fairy hums | FairyCritter 583-585 | NO (d42 WIP shells end at 587 but 583-585 arms are the *next* commits' work) | d33 `npc_ai_style112.rs` UNWIRED, `(27, 1)` seam | UNWIRED stands |
| :48682 FairyEffects Item4 | 583-585 release | same | d33 style112 `fairy_effects` pushes `(2, 4)` | UNWIRED stands |
| :56403 Medusa gaze | Medusa 480 | row on **d40 only** | the aiStyle-3 fighter machine (g4/d22) has NO Medusa gaze block (`num25` counter unported) | UNWIRED stands |
| :66429 BigMimic reflect | 473-476 | rows on **d40 only** | `NPC.ReflectProjectile(s)` (:66395-66429) unported port-wide — no `CanBeReflected`/`reflects_projectiles` field on the projectile side (the style53 module's outcome-only seam) | UNWIRED stands |

**Zero tail-cues can activate now**: every consumer seam is either an
UNWIRED d33 module (no kind → no dispatch), an unmerged d40 row (no
machine), or an unported machine. Wiring a helper into a live seam
today would be an invented play — the a41b2b72 `allow(dead_code)`
verdict stands unchanged.

**Merger keep-one notes (the d33 modules vs e9's helpers):** the d33
style modules carry their own `out.sounds` seams (the style75
convention) recording the SAME cues e9's helpers queue. At the roster
wave, keep ONE path per site: either the module's `sounds` vec drains
through `play_npc_sound_request` (map `(2, N)`→`ItemCue(N)`,
`(4, 63)`→`NpcDeath63`, `(3, 6)`→`NpcHit6`, `(14, 542)`→`SandSharkCry`,
`(65, _)`→`QueenSlimeMoan`, `(27, _)`→`Pixie`, trackable names→
`Trackable` with laugh→`statuemimic_laugh_0`), or the module calls
e9's decision helper at the cited line and drops its own push. The
helpers own the GATE maths (the latches, the Math.Sign zero-case, the
Next(360)/Next(30) roll plumbing) — the d33 modules own the same gates
inside their machines, so the keep-one is per-site, not per-module.
The e9 helpers stay the citation-pinned reference either way.

**The 232-complete claim vs f2:** `Select-String NPC.cs
"SoundEngine.PlaySound"` counts **232** sites (re-run this round).
f2's tip `bcf4c299` is fully pushed (local == origin, clean tree, 0
unpushed commits — the resume-dispatch answer: **nothing to rescue on
f2**). f2's research/94 already executed the e9-vs-f2 reconcile at
`9de95697`: the 32 idle arms + 8 water sites are dup-ports (f2 the
faithful superset on water, e9's VileSpit/Ghost extras adopted by f2
at `c6f74db1`), and f2's two doc-sweep ports (620 exit jump, lava
strike/buff) are NOT in e9's 27 — they are the movement/damage halves
of the water/lava seams, not PlaySound sites (the exit jump plays
nothing; the lava strike's sounds ride the pooled strike). The union
census: e9's 27 tail decisions + f2's water supersets + the shared
40 = the 232 covered; the two e9 fixes this round (§2/§3) close the
last two fidelity gaps f2's reconcile found in e9's half.

## 2. Fix 1 — the SECOND 147-quirk (commit `712e3c54`)

f2's `9d9c94fe` found e9's `water_splash_gate` applied `type != 147`
to all four arms. The C# re-read (≤100-line windows):
`:93119` (entry shimmer), `:93136` (entry honey), `:93152` (entry
water), `:93217` (exit shimmer), `:93234` (exit honey) all carry
`this.type != 147`; **`:93250` (exit water) DROPS it** — the IceSlime
147 exits water with a style-0 splash while its entry is silent.
`water_splash_gate` gains the `exit_water` selector; the exit-water
arm passes `true`, the other three `false`. Both 147 witness halves
extended (entry water+shimmer silent; exit water plays, exit
shimmer/honey silent). The `615` comment corrected: NPCID.cs:11913
names 615 **Dolphin** (the old "seahorse" label was wrong — the
Seahorse is 626).

## 3. Fix 2 — the critter-emitter key (commit `c7523984`)

`world_sim/npcs.rs`'s `UpdateNPC_CritterSounds` wiring keyed
`n.kind.to_u16()` — the PROTOCOL space. `NpcKind::GolemHead => 74`
(data.rs:403) collides with the C# **Bird 74**'s critter arm
(`NPC.cs:92271-92287`): every attached Golem head rolled the bird's
day `Next(400)` gate + `Next(3)` slot pick on the `UpdateNPCs` stream
— a live determinism drift in every Golem fight (the MERGER-L7
mis-route class e9's idle wiring pinned at `3793fef8`). The fix keys
`cs_id()`: the head folds onto the body's bestiary 245
(bestiary.rs:2035), outside the table. A full mechanical scan of the
`to_u16` table (data.rs:218-589) against the critter table's 23 ids
{74,297,298,300,361-365,442,445,447,602,603,610,611,671-675,687,689}
finds **exactly one collision: GolemHead 74**. The `cs_id` hand table
+ the bestiary net_id column have **zero** critter-table hits (the
roster still has no critter kinds — the emitter stays silent, now
correctly). The seam witness re-keyed + a new GolemHead-74 census
witness (protocol key enters the bird arm and draws; cs_id key
draw-free day AND night). f2's tip carries the same `to_u16` bug at
its :4732 — **cross-note for the merger: f2's critter wiring needs
the same one-line re-key** (f2's branch is pushed+clean; the fix is
not duplicated there per the never-touch-other-lanes rule).

## 4. The idle-loop census (32-arm ladder vs the port)

`NPC.IdleSounds` (`NPC.cs:90075-90208`) re-read whole this round. The
port's `ambient_sound::idle_sound_cue` (e9) carries **all 32 arms
verbatim** — f2's research/94 §1 arm-by-arm table confirms both
ladders MATCH on every gate/window/style/order term (the shimmer head
gate, the arm-6 wet term, the one-wide Scutlix Next(86,87)). The
wiring (`world_sim/npcs.rs:4497`) runs pre-AI on the `UpdateNPCs`
stream behind the `hp > 0` guard (f2 adopted the same guard).

**Which idle kinds are rowed on THIS base?** Of the ladder's ~50
types, the ported kinds resolving through `cs_id()` are: 62/156 —
NO (Demon/RedDevil unported; DesertDjinn is 533, Wizard 108 — the
protocol 62/156 hits were the to_u16 space, not cs_id); the LIVE
matches are **250 AngryNimbus, 288 DungeonSpirit, 389 GigaZapper,
471 GoblinSummoner, 520 MartianWalker, 388 MartianDrone, 391
Scutlix** (all via the bestiary net_id fallback or hand rows). Every
one of those arms is wired and rolls today (the nimbus stream witness
`w4_e9_the_nimbus_idle_roll_rides_the_stream_before_the_ai` pins the
contract). The other ~43 types (blood crawlers 239/240, derpling 177,
flying snake 226, tortoise 153, jellyfish 63/64/103, lihzahrds
198/199, pigrons 170/180/171, the antlion band 508/509/580-582,
crawdad 494, spheres 467/468, ghouls 524-527, shellies 496/497,
granite golem 482, Mothron 477, ladybug 258, parrot 252, possessed
469, reaper/wraith 253/82, sandpoors 498-506) have **no kind on this
base** — their arms are table-complete but silent until the roster
wave (the d34-d43 bands). **No NPC ambient loop is missing from the
port's table**: the census closes 32/32.

The sibling `UpdateNPC_CritterSounds` table (`:92244-92340`, 6 arms)
is likewise whole in `critter_sound_cue` (S1-E-25) — its only defect
was the key space (§3). The Pixie-75 `Next(40)` hum (`:24853-24856`)
stays with the unported Pixie kind (the `play_pixie` seam is
table-complete, S1-E-21g).

## 5. The water-collision co-lane (d53 LIVE + g2 STAGED + f47 shimmer)

**My NPC wet-fields landed** (`npc.rs:2010-2018` lava_wet/honey_wet/
shimmer_wet + wet_count/wet_prev; the wiring
`npc_water_collision::collision_water_step` at the UpdateCollision
call site, `world_sim/npcs.rs:4607`). Co-lane state:

- **d53 projectile-wet LIVE** (`fa7b331d`, pushed): `projectile_wet::
  wet_block` runs at the C# `:16790` slot per sub-step, the 8 splash
  sites queue→drain (`wants_wet_splash` → `pending_splashes` +
  `DustBurst::WetSplash`), shimmer entry 4 / exit 5. The projectile
  side mirrors e9's NPC side; the two lanes share the C# arm shapes
  but NOT code (different constants: 10-dust flat-1.3 projectile vs
  30/10-dust NPC, the exit-only wetCount=10 asymmetry) — no keep-one
  conflict.
- **g2 Shimmer() STAGED** (`63a6db26`, pushed, unwired): the full
  13-arm `Projectile.Shimmer()` (`:22361-22516`) helper exists; its
  single caller `:17011` (`if (base.shimmerWet) this.Shimmer()`) sits
  in the generic wet block d53 has NOW ported — **the compose is
  armed**: d53's `wet_block` doc cites `Shimmer()` as "the
  shimmer-transform lane (W4-e19/f47)" and does NOT call g2's helper.
  The merger seam: one call `if p.shimmer_wet { p.shimmer(world) }`
  at the wet-block tail (`:17009-17012` position, after the dry-clear
  per the C# order — re-read at the compose). g2's 5 witnesses drive
  the helper directly (shimmer_wet set by hand); the compose adds the
  dispatch witness. NOT wired here (g2's claimed follow-up, f47's
  research/121 §6 cross-note names it).
- **f47 shimmer-NPC wet-gravity co-lane** (tip `a2561d6d`, pushed —
  the prompt's "read latest" answered): f47 landed `shimmer_wet`/
  `honey_wet`/`shimmering`/`wants_shimmer_transform` on Npc, the
  buff-353 `try_adding_repeated_buff` adder, the transparency ramp +
  `drain_shimmer_transforms`, and `shimmer_wet_contact_pass` — a
  SECOND writer of the same `Collision_WaterCollision` probe
  (`:93052-93076`) e9's `collision_water_step` already runs.
  **KEEP-ONE at the merge: e9's `collision_water_step` is the
  superset** (the full entry/exit splash machine + wetCount +
  never-wet + the 116 pin + the dry-clear tail); f47's contact pass
  adds ONLY the buff-353 grant e9 cites as the buff-layer seam
  (`npc_water_collision.rs:283`). The fold: lift f47's
  `n.buffs.try_adding_repeated_buff(353, 100, 10)` into e9's shimmer
  arm (the adder + the `shimmering` flag derivation ride f47's
  npc_buffs.rs/npc.rs hunks), drop `shimmer_wet_contact_pass` whole.
  f47's own doc sanctions the timing fold ("the flag/gravity readers
  run at the next tick's head"). **The wet-gravity consumer**: the
  slime family's `slime_gravity` (npc_ai_slime.rs:409, the shimmer
  0.15/5.5 + honey 0.1/4 merges) is consumed by b1's
  `ai_001_slime_family` adapter (pushed `f010b85b`) which probes
  wet/honey/shimmer AT THE SITE — with e9's carried flags landed the
  adapter can read `self.shimmer_wet`/`self.honey_wet` instead (the
  b1 module header already sanctions "the carried Entity flags OR
  probes at the site"); that one-line feed is b1's lane, cross-noted
  not touched. f47's `shimmer_wet` field comment ("the wet gravity
  override reads it first :90939") composes with e9's field of the
  same name — same semantics, dedupe the field at the merge (e9's
  doc cites the water-collision writer, f47's the gravity reader;
  keep both citations).
- **The e9/f2 water union** (f2 research/94 §2-§5): f2's water pass
  is the faithful superset (the never-wet head, the 116 pin, the
  exit-tail order); e9's `collision_water_step` re-read this round
  carries the never-wet head (`never_wet`, the full :93055 list), the
  116 pin (:93077-93080), the exit velocity halving + the dry-clear
  tail — the two e9 gaps f2 named at `3793fef8` were closed by e9's
  own `d7a61594` water-wiring commit BEFORE f2's compare; the remaining
  exit-WATER 147-drop is §2's fix. The merge keep-one: f2's fields +
  census + one ladder, e9's wiring shape + the 27 tail decisions +
  the VileSpit/Ghost extras (f2 already adopted the extras).

## 6. Verification

Code-only: **no cargo/rustc/rustfmt/tests/dotnet/graphify invoked**.
Every C# window re-read ≤100 lines (the four 147-gate arms, the
never-wet head, IdleSounds whole, UpdateNPC_CritterSounds whole, the
Pixie-75 arm). The two fixes' witnesses are WRITTEN NOT RUN:
`w4_e9_the_water_collision_entry_splash_edges_and_styles` +
`..._exit_...` (the 147 halves) and
`w4_e9_the_critter_key_is_cs_id_not_the_protocol_golem_head_74` +
the re-keyed `the_critter_seam_consumes_no_rolls_outside_the_table`.
The mechanical scans (the 232 PlaySound count, the to_u16-vs-critter
collision walk, the cs_id/bestiary critter-hit walk, the d33b/d41r/
d42r/d43r branch existence probes) ran as read-only git/PowerShell.

## 7. Commit ledger (this final round)

| commit | family |
| --- | --- |
| `712e3c54` | water fix: the second 147-quirk + the Dolphin label + the 147 witness halves |
| `c7523984` | critter-key fix: cs_id() not to_u16 + the GolemHead-74 census witness |
| (this doc commit) | docs+anchors: research/141 + ANALYSIS + PORT_MAP + INDEX + 4 anchors rows |

Prior e9 ledger (unchanged): `4e14b51e` sites 1-2, `bdd12b3e` 3-12,
`6fa35999` 13-22, `cb6a488d` 23-32, `94aea7b7` 33-34, `cb53fa39`
35-42, `3793fef8` the cs_id comment, `d7a61594` water wiring,
`9b96eff4` tail 43-69, `255d7ccf` fix round, `a41b2b72` dead_code
allow.

## 8. TAIL-ACTIVATION-2 — the re-probe after the roster pushes (SUPERSEDES §1's tip table)

The §1 probe pre-dated the roster wave's pushes. Re-fetched
(`git fetch origin --prune`, 2026-11-21, second pass) — the lanes the
prompt named as `d33b`/`d42r` etc. are the SAME branches, advanced:

| lane | tip NOW | what landed since §1 |
| --- | --- | --- |
| d33 `task/W4-d33-npc-ai-styles-95-127` | `b08acf46` | styles 99-104/111-119/121/122/125-127 ported UNWIRED (`dcb59b7f` and back), THEN **style 114 WIRED** (`ba8b9b5c`/`e71010d3`): the seven dragonfly kinds 595-601 + the axis' `114 => self.ai_114_dragonfly` arm (`npc.rs:5030`) + the kind-match routing (`:6443`). **114 is the ONLY wired style** — no `111/112/113/115-119/121/122/125-127 =>` axis arm exists (mechanical grep). |
| d42 `task/W4-d42-npc-kinds-561-630` | `489308b5` | the 51-kind join (`9b22c091`) THEN the **vanilla-free reconcile** (`49346dba`): protocol == NPCID (the 354-404 band collided with d37's pushed 354-399), **43 kinds** — the 8 sibling-owned drops: 581/587 (c4 style-44), 592/607/615 (c4b style-16), 604/605 (e26b luck), 606 (c8 style-66). The unwired styles ride `{}` idle stand-ins (578/618 the raw `integrate_no_tile_collide` tail); the aiStyle-3 walkers (580/582/586/590/591/624/630) + aiStyle-16 swimmers + the BloodSquid 619 hover ARE wired. |
| f42 `task/W4-f42-sandstorm-2` | `75f055fc` | phase 3: the nine desert kinds (546/61/**69**/78/79/630/80/580/581, protocol 354-362) + `npc_ai_desert::antlion_ai` — the **Antlion 69's style-19 machine LIVE**, its `:24445-24452` cooldown block queues the Item5 cue inline (`npc_ai_desert.rs:202-210`, f42's own `NpcSoundCue::Item5` variant, drained `world_sim/npcs.rs:1143`). |
| d120 `task/W4-d120-town-chatter` | `2408233d` | **RETIRED** (research/143 FINAL): the four town kinds landed (Santa 142/Truffle 160/Steampunker 178/Cyborg 209 cs_id hand rows, protocol 241-244); the GetChat-40 census complete; the 579 spawn contract handed to the spawn lane. No tail-site consumer. |
| d41 / d43 | `4ed22f0b` / `a725acb0` | UNCHANGED — claim-only / zero commits. |
| f2 `task/W4-f2-npc-sounds-3` | `47738470` | closeout docs (research/94 FINAL §9-§11: the e9b re-adjudication + the 27-tail union census — 25 machine-owned DROP / 2 unclaimed staged). The critter `to_u16` bug STILL present (§8.3). |
| g8 `task/W4-g8-npc-kind-wire-1` | `82c93cec` | the <=230 machine wiring complete — the Antlion 69 rides d24's `npc_ai_style19` module through the axis' `19 =>` arm (`npc.rs:5112`), the `fx.sounds` style-5 tuple → `NpcSoundCue::Item5` (`:5118-5127`). |
| c4 `task/W4-c4-style44-flying-fish` | `479d70ad` | the style-44 axis arm LIVE (`npc.rs:4993`, the world-sim pre-pass `style44_step`); owns 581/587 (d42 dropped them). |

### 8.1 The 27-tail re-intersection (kind-row × machine × axis)

The prompt's activation rule: a tail cue activates only when its kind
is rowed AND its machine dispatches (d42-kind + its ai_style → d33
module IF wired). Re-run per site:

| site(s) | kind(s) | row NOW | machine NOW | verdict |
| --- | --- | --- | --- | --- |
| :24449 Antlion spit | 69 | **f42 + g8** | **LIVE both** (f42 inline `antlion_ai`; g8 axis-19 → d24 module) | **ACTIVATED — machine-owned, NOT e9's.** The site is a dup-port: f2's research/94 §10 row 1 already adjudicated DROP (the machine owns the gate; the `Item5` variants fold onto `ItemCue(5)` at the merge). e9's `antlion_spit_cue` stays the citation-pinned reference; the f42-inline vs g8-module keep-one is the style-19 machine merge (ONE machine, the d24 module is the faithful superset — f42's inline block is the same :24445-24452 maths). |
| :37268/:37321 teleport | 472/521 | 472 on d40 (`{}` stand-in); 521 nowhere (d9b no branch) | d32 `npc_ai_style86.rs` UNWIRED (the `sound_item8` out-field exists; NO `86 =>` axis arm on any tip) | UNWIRED stands |
| :40632 SandShark cry | 542-545 | NO (d41 claim-only) | d33 style103 UNWIRED | UNWIRED stands |
| :42983-43001 Pals | 695/696 | NO (d43 zero) | d33 style127 UNWIRED | UNWIRED stands |
| :43562-43608 StatueMimic | 690 | NO (d43 zero) | d33 style126 UNWIRED | UNWIRED stands |
| :43726 ClumsySlimeBalloon | 686 | NO (d43 zero) | d33 style125 UNWIRED | UNWIRED stands |
| :47951/:48062 WindyBalloon pop | 594 | **d42** (`{}` stand-in, `npc.rs:6815`) | d33 style113 UNWIRED | UNWIRED stands — kind rowed, machine not |
| :44848 PirateGhost | 662 | NO (d43 zero) | d33 style122 UNWIRED | UNWIRED stands |
| :45558/:45759 QueenSlime | 657 | NO (d43 zero) | d33 style121 UNWIRED | UNWIRED stands |
| :47263-47293 BloodNautilus | 618 | **d42** (raw `integrate_no_tile_collide`, `npc.rs:6730`) | d33 style117 UNWIRED | UNWIRED stands — kind rowed, machine not |
| :48327/:48473/:48545/:48682 fairies | 583-585 | **d42** (`{}` stand-ins, `npc.rs:6822+`) | d33 style112 UNWIRED | UNWIRED stands — kind rowed, machine not |
| :56403 Medusa gaze | 480 | d40 (`update_skeleton` family) | the gaze block (`num25`) unported port-wide | UNWIRED stands (f2 staged the helper, research/94 §10 row 14) |
| :66429 BigMimic reflect | 473-476 | d40 (`{}` stand-ins) | d32 style87's `reflect_request` out-field exists; NO branch ports the `ReflectProjectile` body | UNWIRED stands (f2 staged, §10 row 15) |

**The LIVE-axis check (595-601):** d33's wired style-114 machine
activates **ZERO tail cues** — `AI_114_Dragonflies`
(`NPC.cs:47716-47886`) contains NO `SoundEngine.PlaySound` site (the
232-site census has no hit in the range; the module carries no
`sounds` seam — mechanical grep), the `IdleSounds` ladder never names
595-601, and the critter table never names them either (the new
witness `w4_e9_tail_activation_2_the_landed_roster_ambient_census`
pins all three). The dragonflies' only sound-adjacent seam is the
`IsGoldCritter` sparkle (`:92345-92368`) — a DRAW-pass light, not a
PlaySound site (d33's own commit names it the light-feed wave). So
the §1 verdict stands with ONE amendment: **:24449 is now
machine-owned-LIVE on f42/g8** (a dup-port, not an e9 activation);
every other tail site still has no live consumer. The
`allow(dead_code)` stays (re-cited at `a4cacbfc`).

### 8.2 The activation contract (the merger's per-site map)

For each rowed-but-unwired kind, the trigger + the seam + the keep-one
(the §1 keep-one notes refined with the landed row citations):

| kind (d42 row) | trigger | seam at the trigger | keep-one |
| --- | --- | --- | --- |
| WindyBalloon 594 | d33 style113 wires (the axis `113 =>` arm + the `{}` stand-in replaced) | the module's pop out-field `(4, 63)` at the wet/slave-wet/collide gate (`NPC.cs:47939-47952`/`:48051-48063`) | drain `(4,63)` → `NpcSoundCue::NpcDeath63` through `play_npc_sound_request`, OR call e9's `balloon_pop_cue(npc, pop)` at the gate and drop the push. The slave-wet term is the machine's (e9's helper takes the composed `pop` bool). |
| FairyCritter 583-585 | d33 style112 wires | the module's `(27, 1)` entries (cases 3/4/5) + `fairy_effects`' `(2, 4)` | `(27,_)` → `NpcSoundCue::Pixie` (the `play_pixie` path), `(2,4)` → `ItemCue(4)`; OR e9's `fairy_circle_hum`/`fairy_far_hum`/`fairy_effects_cue` at the cited ticks. The far-hum's `Next(30)` roll stays on the caller's stream either way. |
| BloodNautilus 618 | d33 style117 wires | the module's `(2, 170)`/`(2, 172)` entries (the case-0 volley pick + the case-1 `ai[1]==89` scream) | `(2,N)` → `ItemCue(N)`; OR e9's `blood_nautilus_volley_cue`/`scream_cue`. The `%7==3` no-sound arm is the gate maths both copies carry. |
| DD2LightningBug 578 | d33 style111 wires | the `DD2_EtherianPortalSpawnEnemy` tracked cue at `localAI[1]==30` — NOT an e9 tail site (the 27 never claimed it; the d42 `dd2_lightning_bug_hurt_0/death_0` bank rows serve HitEffect/Kill) | no e9 keep-one — the module's own trackable seam. |
| WaterStrider 612/613, Seahorse 626/627, Dandelion 628 | d33 styles 116/118/119 wire | NO sound sites in any of the three machines (the C# arms are draw-only/seed-spawn) | no e9 keep-one — roster context only. |
| the d43 band (657/662/686/690/695/696) | d43 rows the kinds + d33 styles 121/122/125/126/127 wire | the §1 keep-one map unchanged (`(2,167)`/`(65,1)`/`(3,6)`/`(4,63)`/trackables/`PAL_SOUNDS`) | f2's research/94 §10 adjudicates DROP-helper + ADOPT-bank for these (the machine out-structs own the sites; the shared bank + dispatch serve them). e9's helpers stay the citation-pinned reference. |

### 8.3 The f2-critter-rekey hunk (EXACT, for f2c/the merger)

f2's tip `47738470` still keys the critter emitter on the protocol
space. The one-line fix (`crates/tr-app/src/world_sim/npcs.rs:4765`,
inside the `critter_sound_cue(` call at `:4764`):

```diff
-                        n.kind.to_u16() as i32,
+                        n.kind.cs_id() as i32,
```

plus the witness key-space consistency at `:7268`
(`NpcKind::Zombie.to_u16() as i32,` → `NpcKind::Zombie.cs_id() as
i32,` — both are 3 for the Zombie, no behaviour change; e9's
`c7523984` re-keyed the same witness). f2's IDLE wiring already keys
`cs_id` (`:4476`) — only the critter call needs the rekey. NOT
applied on f2 per the never-touch-other-lanes rule.

**The cross-lane extension (NEW this probe):** every lane that forked
before e9's `c7523984` carries the same `to_u16` key, and the
mechanical to_u16-table × critter-table scan (the 23 ids
{74,297,298,300,361-365,442,445,447,602,603,610,611,671-675,687,689})
run against TEN pushed tips finds the collision set is per-tip:

| tip | the critter call | protocol-space collisions (kind⇒to_u16 → the C# arm it mis-enters) | after the cs_id rekey |
| --- | --- | --- | --- |
| e9 (this, FIXED) | `npcs.rs:4685` cs_id | — (GolemHead 74 was the only one; fixed `c7523984`) | zero draws |
| f2 `47738470` | `:4765` to_u16 | GolemHead⇒74 → the Bird day arm | head folds to 245, silent |
| f42 `75f055fc` | `:4650` to_u16 | GolemHead⇒74; **WalkingAntlion⇒361 → the squirrel/underground arm (`:92264`, Next(200) case 31); FlyingAntlion⇒362 → the bunny day arm (`:92257`, Next(200) case 30)** | cs_id 580/581 outside the table, silent |
| g8 `82c93cec` | `:4643` to_u16 | GolemHead⇒74; **CursedHammer⇒361 (squirrel), EnchantedSword⇒362 / Mimic⇒363 / GreenJellyfish⇒364 / Gastropod⇒365 (bunny)** | cs_id 83/84/85/103/122 outside, silent |
| d37 (211-280) | `:4614` to_u16 | GolemHead⇒74; **Pirate⇒361 (squirrel), GoldfishWalker⇒362 / HornetFatty⇒363 / HornetHoney⇒364 / HornetLeafy⇒365 (bunny)** | cs_id 229/230/231-233 outside, silent |
| d42 `489308b5` | `:4614` to_u16 | GolemHead⇒74 ONLY — the vanilla-free band makes Seagull⇒602/Seagull2⇒603/Rat⇒610/Owl⇒611 **correct-by-band** (protocol == cs_id; those arms SHOULD fire) | the four keep firing (cs_id identical); the head goes silent |
| d40 (421-490) | `:4614` to_u16 | GolemHead⇒74 ONLY — GoldBird⇒442/GoldFrog⇒445/GoldMouse⇒447 correct-by-band | the golds keep firing; the head silent |
| d33 `b08acf46` | `:4647` to_u16 | GolemHead⇒74 (the dragonflies' 595-601 are outside the table in BOTH spaces) | silent |
| c4 `479d70ad` | `:4662` to_u16 | GolemHead⇒74 (581/587 outside in both spaces post-reconcile) | silent |
| d120 `2408233d` | `:4719` to_u16 | GolemHead⇒74 (the town four 142/160/178/209 outside) | silent |
| d41/d43 | `:4614` to_u16 | GolemHead⇒74 (no new kinds) | silent |

The merger applies the SAME one-line rekey on every lane that keeps
its own critter wiring (or keeps e9's fixed wiring as the keep-one
survivor — the e9/f2 union already names e9's wiring shape the
winner). The f42/g8/d37 mis-routes are LIVE determinism drifts on
those tips (a standing WalkingAntlion underground rolls the
squirrel's Next(200); a Mimic rolls the bunny's day Next(200)) — the
same MERGER-L7 class §3 pinned, now with a per-tip witness table.
The d42/d40 correct-by-band hits are the POSITIVE control: after the
rekey the Seagull/Rat/Owl/gold-critter arms still fire (their cs_id
equals the protocol id), so the rekey is not a blanket silence.

### 8.4 Verification (this round)

Code-only: no cargo/rustc/rustfmt/tests/dotnet/graphify invoked. The
C# re-reads ≤100-line windows: `UpdateNPC_CritterSounds` whole
(`:92244-92343`), `IdleSounds` whole (`:90075-90211`). The mechanical
scans ran as read-only git/PowerShell: the ten-tip to_u16×critter
collision walk (§8.3), the d33 axis-arm grep (114 only), the d42
stand-in/wired-family reads, the f42/g8 style-19 route reads, the
232-site range check for the dragonfly arm (no hit 47716-47886). The
new witness
`w4_e9_tail_activation_2_the_landed_roster_ambient_census`
(`ambient_sound.rs`) is WRITTEN NOT RUN — table-side only (the named
kinds live on d42/f42/c4, not this base). Commit `a4cacbfc` (code +
1 anchors row); this doc refresh + the ANALYSIS/PORT_MAP/INDEX
addenda ride the docs commit.