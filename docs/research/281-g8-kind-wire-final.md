# 281 — W4-g8 kind-wire FINAL (the E8 execution receipt + the kind-wire-family bible)

> W4-g8 lane (`task/W4-g8-npc-kind-wire-1`) · FINAL / SEAL · 2026-11-21 ·
> code-only mode (no cargo/tests/rustfmt/graphify; witnesses WRITTEN NOT RUN).
> Registry: **281** — the first free number above the all-origin-tip scan's
> max 280 (the research/150 first-pushed-wins convention; the scan walked
> every pushed `task/W4-*` ref's `docs/research/` tree). This lane's earlier
> claims-table rode **108** (`108-input-g8.md`, commit `1752c70df`) — that
> number COLLIDES with d37's `108-roster-master-reconcile.md`; the merger
> keeps d37's 108 (the roster authority) and this 281 is the g8 FINAL
> (the 108 file's E8-receipt banner points here).

## 1. The git-log census — what landed on this branch

The lane's whole arc, base `eb715aafc` (the d32 axis-arms restore) → tip:

| commit | scope |
|---|---|
| `80f7e7486` | lane 1 batch 1 — WIRE 10 kinds (Vulture 61/BlueJellyfish 63/PinkJellyfish 64/Antlion 69/SpikeBall 70/Pixie 75/Wraith 82/CursedHammer 83/EnchantedSword 84/Mimic 85), proto 354-363, axis 17/18/19/20/22/23/25 |
| `0a4b76e3f` | batch 2 — WIRE 11 kinds (GreenJellyfish 103/Gastropod 122/WallCreeper 164/WallCreeperWall 165/IceElemental 169/Herpling 174/Derpling 177/CrimsonAxe 179/FloatyGross 182/LostGirl 195/Nymph 196), proto 364-374, axis 18/22/23/40/41/42 + the style-3 cap-2 walk pair |
| `0f31e10e3` | batch 3 — WIRE 10 kinds (CursedSkull 34/Demon 62/GiantBat 93/Slimer 121/IlluminantBat 137/IceBat 150/Lavabat 151/GiantFlyingFox 152/VampireBat 158/FlyingSnake 226), proto 375-384, axis 10/14 |
| `82c93cecf` | batch 4 — WIRE the last 3 (Squid 221/Harpy 48/RedDevil 156), proto 385-387 — completes the 34-kind ≤230 d22-d33 machine wiring |
| `a6c4f5ae9` | machine round 1 — PORT the Harpy 48 feather-shot + wet arm (`NPC.cs:23333-23345` + `:23416-23436`) |
| `e1d9578a4` | machine round 2 — PORT the RedDevil 156 trident-shot (`NPC.cs:23458-23484`) |
| `5767ca7c7` | machine round 3 — PORT the VampireBat 158 surface-flee + Transform(159) AND ROW the Vampire 159 census (`NPC.cs:22899-22970` + `:23346-23360` + `:15480-15493`) |
| `058453096` | machine round 4 — PORT the FlyingSnake 226 facing/approach (`NPC.cs:22876-22894` + `:22971-23037`) |
| `1752c70df` | PUBLISH the claims table (`108-input-g8.md` + INDEX row) |
| this SEAL | E8 EXECUTED (the 17 moves + the Vampire collision fix + the stale-decl fix) + this FINAL |

The four machine rounds closed every cited ≤230 aiStyle-14 skip; **660**
(`NPC.cs:23038-23115`, the 0.35/0.3/6/5 approach) is the ONLY remaining
type branch unported — not a ≤230 d22-d33 kind, out of lane scope.

## 2. The d37-roster LAGGARD receipt — E8 EXECUTED (zero-movement → moved)

d37's ROSTER SEAL (`fbb948172`, research/108 S0-S4) flagged **g8+c9 as
LAGGARDS: "zero movement, stale ALL_KINDS"**. The g8 half is now consumed:

**The 17 moves (E8 g8 table, executed in `npc/data.rs` `to_u16`/`from_u16`
+ `npc.rs` `render_key`):**

| kind | old | new | cs | | kind | old | new | cs |
|---|---|---|---|---|---|---|---|---|
| PinkJellyfish | 356 | **745** | 64 | | LostGirl | 373 | **755** | 195 |
| Antlion | 357 | **746** | 69 | | Nymph | 374 | **756** | 196 |
| SpikeBall | 358 | **747** | 70 | | IceBat | 380 | **757** | 150 |
| WallCreeper | 366 | **748** | 164 | | Lavabat | 381 | **758** | 151 |
| WallCreeperWall | 367 | **749** | 165 | | GiantFlyingFox | 382 | **759** | 152 |
| IceElemental | 368 | **750** | 169 | | VampireBat | 383 | **760** | 158 |
| Herpling | 369 | **751** | 174 | | RedDevil | 387 | **761** | 156 |
| Derpling | 370 | **752** | 177 | | CrimsonAxe | 371 | **753** | 179 |
| FloatyGross | 372 | **754** | 182 | | | | | |

The old ids 356/357/358/366-374/380-383/387 decode `None` again — the
354-387 corridor returns to d37/d42 (the squatter vacates). Every `cs_id`
UNCHANGED (the vanilla `NPC.type` gates ride `cs_id`, only proto moved).

**The 17 DROP-PENDING dups** (keep the pushed id on this branch — the
merge keep-one drops the g8 row; the canonical owner's row+wiring lands
via its lane; the g8 AXIS WIRINGS all stay, the machine-wiring lane's
whole point): Vulture 354→d34@278, BlueJellyfish 355→d34@280, Pixie
359→d35@700, Wraith 360→d35@705, CursedHammer 361→d35@706,
EnchantedSword 362→d35@707, Mimic 363→d35@708, GreenJellyfish 364→d35@720,
Gastropod 365→d35@728, CursedSkull 375→d34@267, Demon 376→d34@279,
GiantBat 377→d35@710, Slimer 378→d35@727, IlluminantBat 379→d35@729,
FlyingSnake 384→d37@360, Squid 385→d37@358, **Harpy 386→d34@275**.
The drops are NOT executed here: d34/d35/d37's rows are not on this
branch, deleting the g8 rows now would orphan the 17 axis claims (the
`unreachable!` kind arms + the style dispatch reference live variants).
The d42 precedent (`49346dba6`) dropped only kinds whose canonical
owners' ids it itself carried; g8's dups all live on OTHER lanes.

**The Vampire 388→762 COLLISION FIX** (new finding, not in the E8 table —
the Vampire landed AFTER the E8 broadcast; RESUME-AMENDED): d37's
corridor owns 388 (`RustyArmoredBonesAxe`, verified at `origin/task/W4-d37`
tip `fbb948172`'s `npc/data.rs`). The in-flight draft took 788 — **a
DEFECT**: the d37 SEAL (`fbb948172`, research/108 line 1408) explicitly
reserves **788-790 for d39's 3 unclaimed kinds** ("d39's remaining 3
unclaimed kinds take 788-790"). The CORRECT id is **762** — the first
free id above the S4 CORE max 766: 763-766 = d120's towns per R6,
767-780 + 791-804 = d36's re-pin, 781-787 = d39's pushed seven, and
`_d37_seal_final.json` (the scripted SEAL receipts) carries **no 762**
in either its core `final` map (max 766) or its `grand` map (max 787).
The Vampire takes 762 (cs_id stays 159). FLAG to the merger: d39's
788-790 reservation and d36's 767-780 + 791-804 bands are unaffected.

**The stale ALL_KINDS decl FIXED: 270→305.** d37 R0 said "270→304"; the
measured array carries **305** rows (270 base + 34 lane-1 kinds + the
Vampire — R0's census predated machine round 3). The `s1_f_3` literal pin
270→305 lands with it. Scripted census (this SEAL, re-run at the resume
amendment): `to_u16` 305 arms / `from_u16` 305 arms / id sets IDENTICAL /
zero duplicates / max 762.

**The stale `s1_f_13d` pins re-derived** (broken since batch 3/4, found
by this SEAL): `from_u16(384)` was asserted `None` but batch 3 seated
FlyingSnake there — now `Some(FlyingSnake)` (drop-pending); `from_u16(387)`
is `None` again (RedDevil moved to 761). The martian-pair intent (384/387
are NOT ForceBubble/MartianTurret wire ids) holds.

**Witness:** `w4_g8_e8_the_owned_kinds_ride_745_761_and_the_cs_ids_stay`
(`npc.rs`, WRITTEN NOT RUN) — every move's new encode/decode + freed old
id + unchanged cs_id, the Vampire 762/388 pair (+ the `from_u16(788)`
None pin — d39's reservation stays free), all 17 drop-pending rows,
the 305 count pin.

**c9's half stays c9's** (the snail-family ALL_KINDS fix-hunk, d37
`0466ecf81`'s `tools/_d37_c9_allkinds_fix.patch`) — not g8's to execute.

## 3. The Harpy keep-ONE receipt (g44e SS6 consumed)

g44's islands RETIRE SS6 (`8fcd151c0`, research/87) published the harpy
spawn-band activation: **g8-386 vs d34-275 keep ONE row** (a duplicate
variant is a loud compile conflict; either survivor activates the band
identically — net_id 48 + ai_style 14 on both). The E8 table resolves it:
**keep d34@275, drop g8@386**. This SEAL marks the g8 Harpy row
DROP-PENDING (proto 386 stays live on this branch until the merge; the
axis-14 wiring + the machine-round-1 feather port are the g8
contributions that survive on the canonical kind).

**The feather-shot NOT-active claim is SUPERSEDED.** g44e's 300-branch
sweep (11:08, "the FEATHER SHOT IS NOT ACTIVE at any tip —
npc_ai_style14.rs:121/:135 are cited skips") predated machine round 1
(`a6c4f5ae9`, 12:05). At THIS tip the shot is ACTIVE and C#-spec-exact:

- `npc_ai_style14.rs` `bat_ai` type-48 arm: `ai[0]++` every tick, fires at
  **30/60/90**, `Collision.CanHit` gate, **speed 6f** normalized aim,
  two `Next(-100,101)` jitters **X-then-Y**, `Style14Shot { kind:
  HARPY_FEATHER (38), damage: 15 }` at the centre; the reset gate
  `flag24 && ai[0] >= 400 + Next(400)` (the `&&` short-circuit keeps the
  draw out of out-of-view ticks) — `NPC.cs:23416-23436` verbatim.
- The wet arm (`:23333-23345`): the 0.95 down-damp / −0.5 ascent / −4 cap
  / second TargetClosest, OUTSIDE WATER_GROUP (one generic approach pass
  then the lift).
- The drain: `update_cave_bat` → `wants_hostile_shot` → the sim's
  timed-shot list applies **timeLeft 300** (`world_sim/npcs.rs`, the
  `HARPY_FEATHER` arm of the override list, `NPC.cs:23429`).
- The HarpyFeather 38 proj_data row landed (`Projectile.cs:1207-1216`:
  14×14, aiStyle 1, penetrate −1, tileCollide).
- Witnesses: `w4_g8_s14_the_harpy_feather_draws_x_then_y` (module) +
  `w4_g8_machine_the_harpy_feather_drains_through_the_wrapper` (wiring),
  WRITTEN NOT RUN.

The g44e SS6 activation hunk itself (the spawner terminal-None block →
`spawn_hover(Harpy)`) stays g44's published merge input — g8 does NOT
touch spawner.rs (empty diff vs base, re-verified).

## 4. The g59-activations receipt (consumed, no re-work)

g59's ACTIVATIONS-2 (`d277f782b`, ANALYSIS batches 14-16) already
credited g8 `82c93cec`: the **13 no-row kinds** (Antlion 69, IceBat 150,
GiantFlyingFox 152, WallCreeper 164/165, VampireBat 158, IceElemental
169, Herpling 174, Derpling 177, CrimsonAxe 179, LostGirl 195, Nymph 196,
FloatyGross 182) ALL activated by g8's hand rows + g59's death-effect
arms; **64 PinkJellyfish stays the census artifact** (the color-only
nested `:85537` — g8's row landed, its (225,70,140) tint composes with
63's arm shape at merge, recorded not activated). This SEAL adds no
activation — the E8 renumber moves proto ids only, the cs_id-keyed
death-effect arms are untouched.

**The 158-flip-callers receipt (g61 research/273's GATED row → CLOSED):**
g61's spawn-family FINAL read g8 at `e1d9578a4` (machine round 2) and
flagged "the flip callers (`NPC.cs:23353` aiStyle-14 / `:58850`
aiStyle-3) are cited machine skips on BOTH branches — the row landed,
the Transform branch did not". Machine round 3 (`5767ca7c7`, 14:17 —
AFTER g61's read) landed BOTH: the bat→vampire Transform(159) gate
(`npc_ai_style14.rs`, within-200px + bottom-above + CanHit) and the
vampire→bat >300px return (`npc.rs::update_vampire`), plus the sim's
transform drain 158↔159 pair (`world_sim/npcs.rs`). The "still skips"
note is stale at this tip; the 158↔159 arm has BOTH rows + BOTH flip
callers. Witnesses: `w4_g8_machine_the_vampire_bat_transforms_into_the_vampire`
+ `w4_g8_machine_the_vampire_returns_to_the_bat_beyond_300px` (WRITTEN
NOT RUN). The 159 d36-census-only claim is superseded by g8's full row
(the d36 tip carries no 159 kind commit — g61's own receipt).

## 5. The kind-wire-family bible (one paragraph)

**The ≤230 d22-d33 machine-wiring family is g8's, the kind ROWS are the
roster lanes'.** g8 owns the 34 axis wirings (styles 10/14/17/18/19/20/
22/23/25/40/41/42 + the style-3 cap-2/cap-6 fighter riders) and the four
aiStyle-14 machine-round ports (Harpy feather/wet, RedDevil trident,
VampireBat flee/Transform + the Vampire census, FlyingSnake facing/
approach) — these survive EVERY keep-one because they wire the STYLE
axis, not the kind row. The 17 owned kinds ride proto **745-761** (E8
executed; cs_ids unchanged), the 17 dups drop at merge to d34 (Vulture
278/BlueJellyfish 280/CursedSkull 267/Demon 279/**Harpy 275**), d35
(Pixie 700/Wraith 705/CursedHammer 706/EnchantedSword 707/Mimic 708/
GiantBat 710/GreenJellyfish 720/Slimer 727/Gastropod 728/IlluminantBat
729), d37 (FlyingSnake 360/Squid 358); the Vampire rides **762** (the
388 collision fix; 788-790 stay reserved for d39). Merge order per d37 S4: d34→d38→d37→d40→d42→
d35-renumber-last→small-lanes (g8 among them) — at the g8 fold the 17
dup variants + their `to_u16`/`from_u16`/`render_key`/ALL_KINDS/bestiary/
sheet rows DROP, the axis claims + the machine ports + the 17 owned rows
KEEP, the bijection gate re-runs (the union ALL_KINDS 552 core / 583
grand per S4, g8 contributes its 18 survivors: 17 owned + Vampire).
The portrait slots dedupe to the owned kinds (the dups' slots land via
their owners or the R9 deferral list). The Harpy spawn-band activation
is g44e SS6's ONE published spawner hunk (keep d34@275); the feather
shot is ACTIVE (g8 machine round 1, the g44e NOT-active sweep
superseded). The 158↔159 flip pair is CLOSED (g61's GATED note stale).
660 is the family's only remaining aiStyle-14 cited skip (>230, out of
scope). No lane but g8 touches `npc_ai_style14.rs`'s machine arms; the
roster lanes own rows, g8 owns the wiring — **one row per kind, one arm
per style, the machine ports ride the axis through every keep-one.**

## 6. Verification (code-only)

- Scripted id-space census (re-run at the resume amendment,
  `tools/_g8_e8_bijection.py`): `to_u16` 305 arms, `from_u16` 305 arms,
  sets identical, zero duplicates, max 762 (the E8 band 745-761 + 762
  present; the Vampire-788 defect fixed).
- `render_key` census: the E8 moves REMOVED 4 pre-existing duplicate
  groups (370/371/372/373 — the g8 Derpling/CrimsonAxe/FloatyGross/
  LostGirl keys collided with the fishron family's vanilla-type keys;
  the moves to 752-755 cleared them). 19 duplicate groups remain, ALL
  pre-existing at base `eb715aafc` (18) + the batch-3 FlyingSnake-384/
  ForceBubble-384 pair (drop-pending; the merge keep-one clears it) —
  a recorded base deviation, NOT introduced by this SEAL, flagged for
  the G6-key lane/merger.
- The anchors canary rows repointed (918 Nymph 756, 920 RedDevil 761,
  923 Vampire `762 => NpcKind::Vampire` in data.rs) + the E8 SEAL row
  added (marker `NpcKind::PinkJellyfish => 745`); NO rows deleted, the
  batch rows' band descriptions carry the E8 annotation.
- Witnesses WRITTEN NOT RUN (no cargo/tests/rustfmt/graphify per lane
  constraint): the E8 receipt witness + the 4 machine-round witnesses
  re-pinned (the Vampire 762 round-trip, the batch band pins).
- Co-lane status: **dirty-12 ALIVE** (TR-W4-d12's docs/ANALYSIS.md +
  research/127 modified 14:26-14:27, seconds before this SEAL's first
  probe) — untouched, reported. The concurrent g8 commit `1752c70df`
  (14:37, the claims-table publish) is docs-only and composes.