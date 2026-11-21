# 110 — NPC death dust × gore keep-one merger playbook (W4-g60)

**Status:** ✅ merger playbook + gore-tail re-census (code-only; the wiring wave executes the strips; the gore-remainder phase re-classified the 66 — see §11)
**Lane:** W4-g60 (`task/W4-g60-npc-death-effects-2`, wiring DONE `e55e2272`, playbook DONE `3b5f9b3c`)
**Companions:** W4-g59 (lane 1), W4-d55 / W4-e5 / W4-f10 (the gore lanes), W4-c12 (the in-flight death feed)

This is the **keep-one-exec** playbook for the NPC death-effects cluster. Five
lanes port overlapping slices of `NPC.HitEffect` (`NPC.cs:81654-89834`)'s
per-kind **death halves** (`life <= 0`). The dust lanes (g59/g60) emit the dust
bursts **and consume the gore ARGUMENT draws**; the gore lanes (d55/e5/f10) emit
the gore **bodies** but their dispatchers **re-run the same dust loops** for the
overlap kinds. When the wiring wave claims the strike/death slot, each C#
statement must draw **exactly once**. This doc assigns a single owner per kind
and lists the precise strips at each lane's wiring.

> **Golden rule for the merger:** keep the dust lane's feed (g59/g60), keep the
> gore lane's `NewGore` bodies + kicks, and **strip the gore lane's re-drawn dust
> loop** for exactly the overlap kinds. The gore ARGUMENT draws (conditional-gore
> gates, type/velocity/count picks) are **already consumed by the dust lane** —
> the gore lane must NOT re-draw them. Constant-argument gore calls draw nothing
> outside the pool. (The e2 lane's split; the g63 claim's gore-sibling rule.)

---

## 1. The lanes (tips read for this playbook)

| lane | branch | tip read | module(s) | scope |
| --- | --- | --- | --- | --- |
| **g59** | `task/W4-g59-npc-death-effects-1` | `eea6975b` | `npc_death_dust.rs::death_dust` | death dust kinds **1-230** (64 port kinds / 66 C# types, batches 1-7) |
| **g60** | `task/W4-g60-npc-death-effects-2` | `e55e2272` | `npc_death_dust.rs::death_dust_feed` → `death_dust_draws` | death dust kinds **231-697** (121 census-present / **120 `lane_covers`** cs_ids) |
| **d55** | `task/W4-d55-gore-table-sites` | `ec78d152` | `npc_hit_effect_a.rs::hit_effect` | gore wave A, sites 1-30 (`NPC.cs:81660-81941`) |
| **e5** | `task/W4-e5-gore-sites-2` | `a62a32ce` | `npc_hit_effect_a.rs` (e5 copy) + `npc_hit_effect_b.rs::hit_effect_b` | gore wave B, sites 31-86 (`NPC.cs:81942-82216`) |
| **f10** | `task/W4-f10-gore-sites-3` | `67c51ec7` (gore-remainder re-read; playbook probe `a69fb00c`) | `npc_hit_effect_c.rs::hit_effect_c` | gore wave C, sites 1-560 (`NPC.cs:82206-85334` — the pushed `:83562-85559` continuation landed batches 24-60) |
| **f5** | `task/W4-f5-dust-proj-3` | `b33ba0a9` (gore-remainder read) | `npc_hit_effect_d.rs` | wave D claim `97c70c97`: the WHOLE `NPC.cs:85560-89835` classic-mob chain, **853 gore sites** (bodies AND dust census); batches 1-33 pushed (sites 1-290, up to `:86986`), the rest in flight |
| **c12** | (in flight) | — | `npc_ai_style74.rs` `style74_death_feed` + `drone_hit_dust_plan`/`corite_hit_dust_plan` | the **388/418** in-flight death feed (count-plans, not yet pool-fed) |

All five branches share the merge-base `a725acb0` ("Land wave-3 magic+summon
lane"). `death.rs` (the death-cue/loot retain) is **byte-identical** across
d55/e5/f10/g59/g60 — none of these lanes touches it; the death **dust** seam is
`world_sim/npcs.rs`'s death retain, not `death.rs`.

**g59b note:** no `g59b` branch exists on origin at probe time; g59's tip is
`eea6975b` (batch 7 + the gore-argument ownership note). If g59 advances, re-run
the boundary check (§6) — the g59 lane is kinds 1-230 only, so any new batch
stays below 231 and cannot collide with g60.

---

## 2. The seam (where the feeds ride)

The C# runs `HitEffect(hitDirection, num)` inside `StrikeNPC_Inner`
(`NPC.cs:81488`) on the **striking stage's** stream, BEFORE `checkDead`'s
DeathSound (`:81559` → `:78186`). The port **batches deaths into the NPC pass**
(the S1-E-21c recorded deviation), so both dust feeds ride the death retain in
`world_sim/npcs.rs::update_npcs`, **before the death cue**, on the `UpdateNPCs`
stream — the W4-c12 `style74_death_feed` precedent.

Both g59 and g60 insert their feed at the **identical point** (after the
moon-event kill reports `report_frost_moon_kill`/`report_pumpkin_moon_kill`,
before the `NPC.checkDead` DeathSound comment, ~`npcs.rs:5016`):

* g59: `crate::npc_death_dust::death_dust(npc, npc.kind.cs_id() as i32, &mut app.dust, &ctx, rng, &app.world)`
* g60: `crate::npc_death_dust::death_dust_feed(app, npc)` (snapshots the corpse, folds Sharkron2→373, gates `lane_covers`, skips the Spore hp==0 edge)

**Merge consequence:** the two calls are **disjoint by kind** (g59 matches
1-230, g60's `lane_covers` matches 250-668) and both no-op outside their match.
They can coexist at the same site in either order — neither draws for the
other's kinds. The merger keeps **both** calls; there is no dust-dust overlap
between g59 and g60 (verified §6).

---

## 3. Ownership model

For each vanilla type 1-697 the playbook assigns:

* **dust-owner** — the lane that emits the death-half dust burst (and consumes
  the interleaved gore ARGUMENT draws). Exactly one of `g59 / g60 / e5 / f10 /
  c12`, or `-` (no death dust: gore-only or unported).
* **gore-owner** — the lane(s) whose dispatcher carries the `NewGore` bodies.
  `d55(hat)` is the cross-cutting party-hat gore (NOT a death tail). A kind can
  have its dust in g60 and its gore bodies in e5/f10 — that is the normal split.
* **draw-count** — the dust-owner's burst shape (the RNG census), with
  `[+args: …]` marking the gore ARGUMENT draws the dust lane consumes.
* **STRIP-list** — the gore-lane dust loops to delete at that lane's wiring for
  this kind (keep-one). `-` = nothing to strip.
* **gore-tail** — for g60 kinds: `gore-impl` (a gore lane already carries the
  death gore bodies) vs `gore-f10` / `gore-f5` / `gore-none` (the gore-remainder
  re-census, §11 — the old blanket `GORE-UNIMPL` was probe-time-stale: f10's
  pushed continuation `67c51ec7` and f5's wave-D claim `97c70c97` own almost
  all of it). g60 still consumes the argument draws for stream parity in
  every case; the bodies are the owning lane's.

### The keep-one invariant

```
C# death half  =  [dust loop]  +  [gore ARG draws interleaved]  +  [NewGore bodies]
port (merged)  =  dust-lane feed (dust loop + ARG draws)        +  gore-lane bodies
                  └── ONE owner ──┘                               └── kicks only ──┘
```

The gore lane's dispatcher today re-runs `[dust loop]` (and sometimes the ARG
draws). **Strip exactly that** for the overlap kinds; keep the `NewGore` calls.
Where the gore lane re-draws an ARG draw the dust lane already consumed (the
566/567 six `Next(2)` gates, the 576/577 `Next(5)` head pick, the 439/440/454-459
`Next(1,4)`+`Next(11,14)` bone loop), strip the **draw** too and read the
already-advanced stream position — do NOT re-roll.

---

## 4. Per-kind single-owner assignment (summary)

| dust-owner | kinds | count | notes |
| --- | --- | --- | --- |
| **g59** | 2-228 (the 66 C# types of batches 1-7) | 66 | lane 1-230; 64 port kinds |
| **g60** | 250-668 (`lane_covers`) | 120 | lane 231-697; 121 census-present − 388/418 + 373 fold |
| **e5** | 657, 658, 659, 660, 667 | 5 | gore-FIRST / census-ineligible — e5 owns whole |
| **f10** | 379, 426, 429, 438, 440, 454-459, 460-463, 466, 468, 469, 477-479, 491, 492, 510-515, 524-532, 541, 546, 578, 618-623 | 47 | g60-absent kinds f10 covers (no port kind row yet) |
| **c12** | 388, 418 | 2 | the in-flight `style74_death_feed` (count-plans → pool feed at c12 wiring) |
| **e5(plan)** | 636 | 1 | d55 `empress_hit_dust_draws` count-plan → pool feed (NOT a second census) |
| **-** (gore-only / hit-half) | 437, 594, 686 | 3 | 437 tablet gore burst (no dust); 594 gore-only; 686 death gore-only (hit dust → g62) |

**Overlap (the strips):** g60 ∩ d55 = **{668}** (1); g60 ∩ e5 = **{551-557,
561-563}** (10); g60 ∩ f10 = **44 kinds** (the DD2 band 551/558-577 + the solar
412-417/419/518/416 + the pillar 407/409/411/420/421/423/424/425/427/428 + the
martian 381/382/383/390 + 370/372/373/439). e5 ∩ f10 = **{551}** (the flipped-arm
twin). Total strip rows: **54** (551 carries three strip entries).
**Gore-remainder delta (§11):** f10 +24 kinds (the pushed continuation) →
g60 ∩ f10 = **68**; g60 ∩ f5 = **32** (new lane, 11 pushed + 21 claimed).
Total strip rows: **54 + 24 + 32 = 110**.

> **The wiring comment under-counted f10.** The g60 module header / `npcs.rs`
> comment listed only the DD2 band + solar arms as the f10 overlap. The full
> `hit_effect_c` dispatch read shows f10 **also** re-draws dust for the martian
> (381/382/383/390), the cultist boss (439), the sharkron (372/373), Duke Fishron
> (370) and the pillar mobs (407/409/411/420/421/423/424/425/427/428). All 44 are
> in the STRIP-list below. The comment's *principle* is correct; its *list* was
> incomplete — this playbook is the authoritative set.

---

## 5. Strip-instructions at each lane's wiring

### 5.1 d55 (`npc_hit_effect_a.rs::hit_effect`) — wave A

* **668 Deerclops** (`deerclops_death`, `NPC.cs:81892-81909`): **STRIP the
  120-dust burst** (`:81894-81900`, the `new_dust` loop + the per-dust
  `NextVector2Circular(4,4)` velocity overwrite). g60 owns it
  (`npc_death_dust.rs:284-292`). **KEEP the 7 gores** (`:81902-81908`,
  1274×2/1272/1273×2/1270/1271) — constant-argument, kicks only. After the strip
  `deerclops_death` is gore-only; drop its `dust`/`dust_ctx` params or pass them
  unused. The d55 dispatch gate `npc_type == 668 && npc.hp <= 0` stays.
* **636 Empress** (`empress_hit_dust_draws` + `empress_first_gore`): NOT a g60
  kind (census-ineligible: the hit half mixes dust + the death half is
  dust-then-gore but the arm is gore-bearing throughout). **e5 owns 636 whole.**
  The d55 `empress_hit_dust_draws` is a **count-plan** (draws only, no emit) — at
  wiring it must become a **pool feed** (emit the 50× Next(2) 67/69 through
  `DustPool::new_dust`), NOT a second census. The 9 gores (first gore d55
  `:81941`, 8-gore tail e5 `empress_tail_gores` `:81942-81949`) stay.
* **686 Queen Slime / 594 Windy Balloon**: gore-only death halves (the 686 hit
  half's 100× hslToRgb dust is **g62's**, not death). No strip; the
  `Transform(680)`/unlock/slave-release seams stay d55's.
* **Party-hat block** (13 town rows + the hat): NO dust, NO draws (data +
  `NewGore` kicks). Cross-cutting — fires on the hat condition, not death. No
  strip. The g60 town kinds (353/368/588) keep their **death dust** in g60 and
  their **hat gore** in d55; the two are independent.
* **Daybreak arm** (`inputs.daybreak`, debuff-189 flag, `NPC.cs:81730-81751`):
  FLAG-gated, not type-gated — 20× `SelectRandom(6,259,158)` + the AddBuff(189)
  proximity sweep. No fixed cs_id; no g60 overlap (g60's kinds don't carry the
  daybreak death arm). Stays d55's; the buff sweep is the returned
  `daybreak_add_buff` seam.

### 5.2 e5 (`npc_hit_effect_b.rs::hit_effect_b`) — wave B

* **551/552-554/555-557/561-563** (the DD2 goblin/javeliner/kobold/bomber +
  Betsy): **STRIP `dd2_death_dust`** (the 20× type-5 preamble,
  `npc_hit_effect_b.rs:228-245`) from `dd2_goblin_death` / `dd2_javeliner_death` /
  `dd2_kobold_death` / `dd2_crystal_death`. g60 owns the 20×5
  (`npc_death_dust.rs:297-309`). **KEEP the gore bodies** (the per-tier base
  ladders 983/1067/1071, 987/1055/1061, 1031/1034/1037, and the 551 10-gore fan).
  The kobold's tier pick precedes the loop in C# but draws nothing — order is
  preserved by g60's burst. After the strip these fns are gore-only.
* **551 mirrored twin vs f10 flipped arm** (`dd2_crystal_death`'s
  `sprite_dir == -1` branch, `:82204-82216`): e5's `dd2_crystal_death` carries
  **BOTH** branches (the `!= -1` arm `:82191-82203` AND the `-1` mirrored twin
  `:82206-82215`). f10's `betsy_flipped_death` carries **only** the `-1` flipped
  branch — a **duplicate** of e5's twin. **KEEP e5's `dd2_crystal_death` (both
  branches); STRIP f10's `betsy_flipped_death` entirely** (its 20×5 dust AND its
  10 flipped gores — e5 already emits both). See §5.3.
* **657/658-660/667**: NOT g60 kinds (gore-FIRST arms / census-ineligible). **e5
  owns them whole** — no strip. 657's 150×4 tinted dust + 3 gores, 658-660's
  40× tinted dust (+660's 2 gores), 667's 40×10 dust + 8 gores (the gore velocity
  `Next(1,10)`/`Next(4)` draws are gore-side, e5 consumes them). The 667 hit
  arm's 3-gore twin is wave C's (f10), its dust rides e5.
* **551 `CheckProgress` gate** (`dd2_crystal_crossed_percent`, `:82174-82177`):
  the HIT-path percent-crossing predicate (returned `dd2_crystal_check_progress`).
  Not a death draw; stays e5's. The death retain only fires the `hp <= 0` arm.

### 5.3 f10 (`npc_hit_effect_c.rs::hit_effect_c`) — wave C

f10 re-draws dust for **44 g60 kinds**. Strip the dust loop in each cited fn;
keep the gore bodies. The fns and their dust loops:

| f10 fn | kinds | dust loop to STRIP | gore bodies to KEEP | ARG draws g60 already consumed (do NOT re-draw) |
| --- | --- | --- | --- | --- |
| `betsy_flipped_death` | 551 | **whole fn** (e5 twin keeps) | — (e5 emits) | — |
| `kobold_walker_death` | 572/573 | `death_dust_loop(…,5,…,20)` | 993/996/995/994×2 | — |
| `kobold_flyer_death` | 574/575 | `death_dust_loop(…,5,…,20)` | 1050-1054 | — |
| `drakin_death` | 570/571 | `death_dust_loop(…,5,…,20)` | 1075/979 tier | — |
| `wither_beast_death` | 568/569 | `death_dust_loop(…,5,…,20)` | 1020-1023 | — |
| `wyvern_death` | 558/559/560 | `death_dust_loop(…,5,…,20)` | 974/1040/1045 tier (7) | — |
| `skeleton_death` | 566/567 | `death_dust_loop(…,236,…,20)` | 970 + 6 gated | **6× `Next(2)` coin gates** (`:82363-82386`) |
| `ogre_flipped_death` / `ogre_death` | 576/577 | `death_dust_loop(…,273,…,20)` | 6 gores (head + 1000-1004) | **1× `Next(5)` head pick** (`:82406`/`:82417`) |
| `dark_mage_death` | 564/565 | `death_dust_loop(…,5,…,20)` | 1009 ladder (11) | — |
| `solar_spearman_death` | 518/416 | the 20×(241 + Next(3) gated 6) loop | 844/845/847/835/836 | the per-iter `Next(3)` gate (dust-side, g60 owns) |
| `solar_solenian_death` | 419 | the 20×(241 + Next(3) gated 6) loop | 844/845/846×2/847×2 | the per-iter `Next(3)` gate |
| `solar_crawltipede_death` | 412/413/414 | 6×241 + 2×6 full-args | 1 gore (`type-412+828`) | — |
| `solar_flare_death` | 415 | 40×(241 + 6 ungated) | 831/832/833/834 | — |
| `solar_sroller_death` | 417 | 30×(241 + Next(2) gated 6) | 837/838×3/839×2/840×2 | the per-iter `Next(2)` gate |
| `nebula_soldier_death` | 427 | 10×241 + 20× SelectRandom(229,229,240) | 802-805 | the `Next(3)` pick + `NextFloat` scale (dust-side) |
| `nebula_beast_death` | 425 | 20×(249 + Next(3) gated 242) | 790/791/792 | the per-iter `Next(3)` gate |
| `nebula_brain_death` | 428 | 10×241 + 10× Next(2) 229/240 | 806/807 | the per-iter `Next(2)` pick |
| `vortex_soldier_death` | 424 | `vortex_death_dust(…,20,Some(3))` | 793/794×2/795×2 | the per-iter `Next(3)` gate |
| `vortex_hornet_death` | 423 | `vortex_death_dust(…,20,None)` | 790/791/792 | — |
| `vortex_larva_death` | 421 | `vortex_death_dust(…,10,None)` | 785-787 | — |
| `vortex_rider_death` | 420 | `vortex_death_dust(…,10,None)` | 782-784 | — |
| `stardust_spider_death` | 409 | 20×(17 + 229 noGravity×3) | 775/776/777 | — |
| `stardust_cell_death` | 407 | 20×(Next(3)!=0 gated 17 + 229) | 778/779/780/781/780/781 | the per-iter `Next(3)` gate |
| `stardust_dragon_body_death` | 411 | 20×(17 + Next(2) gated 229) | 772/773×2/774×2 | the per-iter `Next(2)` gate |
| `cultist_boss_death` | 439 | 30×5 + 10×16 | 902/903/901×2 + bone loop | **`Next(1,4)` + n×`Next(11,14)`** bone loop (`:84150-84155`) |
| `sharkron_death` | 372/373 | 75×5 Center-(25,25) 50×50 vel/2 | 583/577/578/579 | — |
| `duke_fishron_death` | 370 | 150×5 | 576/574/575/573/574/575 | — |
| `brain_scrambler_death` | 381 | 30×227 | 587/590/588/589 | — |
| `ray_gunner_death` | 382/390 | 30×227 | 591/592/593 | — |
| `martian_officer_death` | 383 | 30×227 | 594/595/596 | — |

**Critical ARG-draw strips (the re-drawn gates):**

* **566/567** — f10 `skeleton_death` re-draws the **six `Next(2)` coin gates**
  (`rng.next_max(2)` at `:82363/82367/82371/82375/82379/82383`). g60 already
  consumed them (`npc_death_dust.rs:319-327`). At f10 wiring, **replace each
  `if rng.next_max(2) == 0` with a read of the already-drawn gate value** (the
  dust lane must hand the six booleans to the gore lane, OR the gore lane reads
  the stream position the dust lane left — the merger picks the hand-off seam).
  The conditional gore bodies (972×2/973×2/971×2) spawn on the SAME gate values.
* **576/577** — f10 `ogre_flipped_death`/`ogre_death` re-draw the **`Next(5)` head
  pick** (`1005 + (Next(5)==0 ? 1 : 0)`, `:82406`/`:82417`). g60 consumed it
  (`npc_death_dust.rs:335-338`). Strip the re-draw; the head type (1005/1006)
  reads the already-drawn value. The flipped branch's two `rotation = π`
  post-writes stay gore-side.
* **439** — f10 `cultist_boss_death`'s `cultist_bone_spatter` re-draws
  `Next(1,4)` count + per-gore `Next(11,14)`. g60 consumed them
  (`npc_death_dust.rs:589-601`). Strip the re-draws; the bone-loop bodies read
  the already-drawn count/types. (The 440/454-459 clone/dragon arms are
  **f10-only** — g60-absent — so f10 KEEPS their bone-loop draws; no strip.)

**f10-only kinds (NO strip — f10 owns the dust):** 379/438 (cultist archer),
426/429 (nebula floater/predictor — the brain-spawn / projectile-578 seams),
437 (tablet, gore-only), 440/454-459 (cultist clone/dragon), 460-463/466/468/469
(the eclipse 5-gore family), 477-479 (mothron), 491/492 (Flying Dutchman),
510-515 (tomb crawler / dune splicer), 524-532 (desert ghoul/lamia/scorpion/beast),
541/546 (sand elemental / tumbleweed), 578 (lightning bug), 618-623 (blood-moon
sea monsters). These have **no port kind row** (g60 census-absent), so f10's dust
loop is the only emitter — keep it whole until the kind lands and g60 (or a
successor) claims it. **418** is the exception: it is c12's in-flight feed, so
f10's `solar_corite_death` dust must ALSO strip at wiring (c12 owns the dust;
f10 keeps the 5 gores 841/842×2/843×2).

### 5.4 c12 (`npc_ai_style74.rs`) — the in-flight feed

* **388 MartianDrone / 418 SolarCorite**: g60's `lane_covers` **excludes** both
  (the module header: "388/418 are W4-c12's in-flight death feed"). c12's
  `drone_hit_dust_plan`/`corite_hit_dust_plan` are **count-plans** (counts only,
  the documented visual skip). At c12 wiring they become **pool feeds** on the
  death retain. **418's f10 `solar_corite_death` dust loop strips** (c12 owns the
  20×(241+6) pairs); f10 keeps the 5 gores. **388** has no gore-lane dispatcher
  (its 2 gores 605/606 are g60-absent / unimplemented) — c12 owns the whole arm.
* The merge reconciles c12's feed with g60's at the **same death-retain site**;
  the three feeds (g59/g60/c12) are disjoint by kind (1-230 / 250-668 minus
  388/418 / 388+418).

---

## 6. The g59 ↔ g60 boundary (1-230 vs 231-697) — CLEAN

**Verified disjoint.** g59's `death_dust` match carries **66 C# types, all
≤ 228** (the highest is 228 WitchDoctor); g60's `lane_covers` carries **120
cs_ids, all ≥ 250** (the lowest is 250 AngryNimbus). The gap 229-249 is unowned
by both (229 has only the d55 party-hat gore; 231-235 are the hornet shared-arm
siblings g59 cites as skips — "the 231-235 band is g60's", but they have **no
port kind row**, so g60's census marks them absent and `lane_covers` does not
include them; they stay unwired until the kind lands).

* **g59 kinds ≥ 231:** none.
* **g60 kinds ≤ 230:** none.
* **cs_id collision (g59 match ∩ g60 `lane_covers`):** ∅.

### The 109-unwired vs 121 question

g59's ANALYSIS records "**109 absent kinds**" — these are the **1-230** census's
dust-first clean kinds with **no port kind row** (the worm segments 8/9/11/12/
14/15, the casters 31/37/39-41/43, the frost/martian/dd2 families, the
195-206/209/217-221/226/229 bands). They stay **UNWIRED** (the S1-F-9b row-gate
convention). g59's 173-kind clean census = 64 present (wired) + 109 absent.

g60's "**121**" is the **231-697** census-present count (the `g60_kmap.json`
`present` set). g60's strict dust-first clean census (`g60_eligible.txt`) is
**281 kinds** = **106 present** + **175 absent** (no port kind row). The present
set is **121** = those 106 + **15 special-fold kinds** the strict walk rejects
but the lane still covers (the Plantera family 262/263/264 + Spore 265 with the
`life >= 0` gate, the Zombie Elf trio 338/339/340, the town smoke arm 369/376/
663, the saucer 393/394/395, Blood Zombie 489, Betsy 551). The `lane_covers` set
is **120** = 121 − {388, 418} (c12's feed) + {373} (the Sharkron2 fold — `cs_id`
folds 373 onto the 372 bestiary row, so `present` lists 372 only).

**Do kinds 231-697 overlap between g59's 109 and g60's 121?** **No.** g59's 109
absent kinds are **all ≤ 230** (its census walks 1-230 only). g60's 121 are
**all ≥ 250**. The two absent/present sets live in **disjoint type bands** — the
boundary is clean. The only shared-arm subtlety is the **hornet arm**
(`NPC.cs:89702`, `type == 42 || 176 || 205 || (231..=235)`): g59 wires **42**
(Hornet, present); the **231-235** siblings are g60's band but **absent** (no
port kind), so neither lane wires them — no double-draw. The 250 AngryNimbus
arm (`:88971`, shared with 82/158/159/253) is g60's; g59's half (82/158/159) is
absent. **No kind is wired by both dust lanes.**

---

## 7. Witness-union (the merger's verification set)

All witnesses are **WRITTEN NOT RUN** (code-only mode). The merge-verify wave
compiles + runs the union; no witness is duplicated across lanes (each pins its
own lane's draws). The union:

### g59 (32 witnesses, `npc_death_dust.rs` tests)
The bee gate ladder, meteor-head twin-pool, snowman trio shared census, ice-slime
pairs, Crimslime alpha/vel×2, Illuminant vel×1.5, off-screen sentinel, non-batch
no-draw gate (batch 1, 8); Demon Eye twin-pool, type-18 trio alpha/scale, Giant
Worm 2.5×, town quartet, Servant 20-count (batch 2, 5); skeleton pair 20-bone,
Fire Imp twin-pool, goblin quartet, Skeletron 150-bone, Guide 50-count (batch 3,
5); bat pair 15-count, Shark 75-count, Hornet type-18, Piranha/Voodoo counts
(batch 4, 4); Wyvern gore-argument ladder, Leech -1 SpeedY, town families, eye
pair (batch 5, 4); eye trio, pirate family, Dye/Party girls (batch 6, 3); Queen
Bee type-18, Umbrella tinted-4, Painter/Witch 50-count (batch 7, 3). **32 test
fns** (verified by count).

### g60 (27 witnesses)
**18** stream-replay witnesses (`npc_death_dust.rs`, batches 1-12: each replays
the cited draws on a pre-call RNG clone, asserts stream equality + slot writes) +
**4** wiring witnesses: the mode-7 trail shift + other-kinds no-shift (`npc.rs`,
2), and the **twin-app stream-parity pair** through the LIVE `update_npcs`
retain (`world_sim/npcs.rs`, 2):
* `w4_g60_the_death_feed_pays_the_burst_before_the_cue` - the 552 DD2 Goblin
  20×5 burst delta (twin apps, same seed; the delta is exactly 20 spawns × 6
  internals; every other stream twin-equal).
* `w4_g60_the_spore_zero_hp_edge_pays_no_dust` - the 265 hp==0 edge (the
  `life >= 0` gate returns with NO dust; twin delta ZERO).

**+5 gore-remainder sweeps** (§11, the `w4_g60_gore_tail_*` fns): per-kind
draw-count rows covering all 66 remainder kinds (the f5/f10/no-gore arms +
the Plantera trio + the Angler pair).

### d55 (7 witnesses, `npc_hit_effect_a.rs`)
The party-hat hash/color, the 686/594 `NewGorePerfect` burst stream, the
daybreak SelectRandom + buff sweep, the 668 deerclops 120-burst + circular
velocity, the 636 count-plan + first gore. **After the 668 strip**, the 668
witness's dust-draw assertion moves to g60's feed (the gore witness keeps the 7
gores + kicks).

### e5 (12 wave-B witnesses, `npc_hit_effect_b.rs`; + the 7 wave-A witnesses in its `npc_hit_effect_a.rs` copy)
The empress 8-gore tail (types + anchors + kicks-only twin), the 657/658-660/667
arms, the DD2 goblin/javeliner/kobold gore ladders, the 551 both-branch fan +
the `CheckProgress` predicate. **After the DD2 dust strip**, the preamble-draw
assertions move to g60; the gore witnesses keep the bodies + kicks. The 551
mirrored-twin witness is the **keep-one anchor** (e5's twin survives, f10's
flipped duplicate is stripped).

### f10 (68 wave-C witnesses, `npc_hit_effect_c.rs`)
Each batch's gore bodies + the dust-loop stream replay. **After the strips**, the
44 overlap kinds' dust-draw assertions move to g60's feed; the gore witnesses
keep the bodies. The **566/567 six-gate** and **576/577 Next(5)** witnesses must
be re-pointed to read the g60-consumed gate values (the hand-off seam), NOT
re-roll. The f10-only kinds' witnesses stay whole.

### Merger-added witnesses (the keep-one proof)
The wiring wave should add a **union stream-parity witness**: a twin-app pair
where one app runs the **merged** death retain (g59 + g60 + c12 feeds + the
stripped gore dispatchers) and the other replays the **C# statement order** by
hand for a representative overlap kind (e.g. 566 DD2 Skeleton: 20×236 dust + 6
gates + the gated gores; 551 Betsy: 20×5 + the 10-gore fan). The delta must be
ZERO — proving one draw per C# statement across the lane boundary.

---

## 8. Shared-file merge points (the hot files)

| file | g59 | g60 | d55/e5/f10 | merge action |
| --- | --- | --- | --- | --- |
| `world_sim/npcs.rs` | +22 (the `death_dust` call) | +160 (the `death_dust_feed` call + 2 twin witnesses) | — | **keep BOTH calls** at the death-retain site (disjoint by kind); union the witness mods |
| `npc.rs` | +`last_hit_dir` field/ctor/3 strike records/bubble reset + Crimslime alpha-55 | +`last_hit_dir` (same shape) + `old_pos`/`old_rot` rings + `update_trails` | — | **`last_hit_dir` is added by BOTH** — dedupe to one field (identical shape); keep g60's trail rings + g59's Crimslime alpha row |
| `main.rs` | +`mod npc_death_dust;` (line ~66) | +`mod npc_death_dust;` (line ~160) | +`mod npc_hit_effect_{a,b,c};` | **one `mod npc_death_dust;`** (the module is shared g59/g60 — see below); keep the gore mods |
| `npc_death_dust.rs` | g59's `death_dust` (1-230) | g60's `death_dust_feed`/`death_dust_draws`/`lane_covers` (231-697) | — | **UNION into one module** — both halves coexist (disjoint matches); the file is the shared death-dust lane |
| `anchors.txt` | +1 row (`pub fn death_dust`) | +1 row (`pub(crate) fn death_dust_draws`) | — | keep BOTH rows (distinct markers) |
| `death.rs` | unchanged | unchanged | unchanged | no conflict (byte-identical) |

**The `npc_death_dust.rs` collision is the key merge:** g59 and g60 BOTH create
`crates/tr-app/src/npc_death_dust.rs` with **different content** (g59's
`death_dust` for 1-230 vs g60's `death_dust_feed` for 231-697). The merger must
**union the two halves into one file** (g59's `death_dust` fn + g60's
`death_dust_feed`/`DeathFx`/`death_dust_draws`/`lane_covers`), keep one
`mod npc_death_dust;` in `main.rs`, and keep both death-retain calls. The two
halves share the `DustPool`/`SpawnCtx`/`UnifiedRandom` imports and the
`Npc.last_hit_dir` field — no semantic conflict, only a textual one (both
branches add the same module path). Resolve by concatenation, not by picking a
side (the AGENTS.md semantic-conflict rule).

---

## 9. Per-kind matrix (1-697)

Columns: **kind** (vanilla `NPC.type`; consecutive unowned kinds compressed to a
band) · **port kind(s)** (the `NpcKind` variant(s), `--` = no port row) ·
**dust-owner** · **gore-owner** (`d55(hat)` = the cross-cutting party-hat gore) ·
**draw-count** (the dust-owner's burst shape; `[+args: …]` = gore ARGUMENT draws
the dust lane consumes) · **STRIP-list** (the gore-lane dust loops to delete at
wiring) · **gore-tail** (g60 kinds: `gore-impl` vs the §11 re-census values
`gore-f10` / `gore-f5` / `gore-f5*` / `gore-none`).

| kind | port kind(s) | dust-owner | gore-owner | draw-count (dust shape) | STRIP-list | gore-tail |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | -- | - | - | - | - | - |
| 2 | DemonEye | g59 | - | 50x5 2dir/-2 | - | - |
| 3-4 | -- | - | - | - | - | - |
| 5 | ServantOfCthulhu | g59 | - | 20x5 2dir/-2 | - | - |
| 6 | EaterOfSouls | g59 | - | 50x18 alpha/color/scale | - | - |
| 7 | DevourerHead | g59 | - | 50x18 alpha/color/scale | - | - |
| 8-9 | -- | - | - | - | - | - |
| 10 | GiantWormHead | g59 | - | 10x5 2.5dir/-2.5 | - | - |
| 11-12 | -- | - | - | - | - | - |
| 13 | EaterOfWorldsHead | g59 | - | 50x18 alpha/color/scale | - | - |
| 14-16 | -- | - | - | - | - | - |
| 17 | Merchant | g59 | d55(hat) | 50x5 2.5dir/-2.5 | - | - |
| 18 | Nurse | g59 | d55(hat) | 50x5 2.5dir/-2.5 | - | - |
| 19 | ArmsDealer | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 20 | Dryad | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 21 | Skeleton | g59 | - | 20x26 2.5dir/-2.5 | - | - |
| 22 | Guide | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 23 | MeteorHead | g59 | - | 50x Next(2) 25/6 + 50x6 vel x6 | - | - |
| 24 | FireImp | g59 | - | 50x6 vel noGravity x2 | - | - |
| 25 | -- | - | - | - | - | - |
| 26 | GoblinPeon | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 27 | GoblinThief | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 28 | GoblinWarrior | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 29 | GoblinSorcerer | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 30-31 | -- | - | - | - | - | - |
| 32 | DarkCaster | g59 | - | 20x26 2.5dir/-2.5 | - | - |
| 33-34 | -- | - | - | - | - | - |
| 35 | SkeletronHead | g59 | - | 150x26 2.5dir/-2.5 | - | - |
| 36 | SkeletronHand | g59 | - | 150x26 2.5dir/-2.5 | - | - |
| 37 | -- | - | - | - | - | - |
| 38 | Demolitionist | g59 | d55(hat) | 50x5 2.5dir/-2.5 | - | - |
| 39-41 | -- | - | - | - | - | - |
| 42 | Hornet | g59 | - | 50x18 alpha/color/scale | - | - |
| 43 | -- | - | - | - | - | - |
| 44 | UndeadMiner | g59 | - | 20x26 2.5dir/-2.5 | - | - |
| 45 | Tim | g59 | - | 20x26 2.5dir/-2.5 | - | - |
| 46-48 | -- | - | - | - | - | - |
| 49 | CaveBat | g59 | - | 15x5 2dir/-2 | - | - |
| 50 | -- | - | - | - | - | - |
| 51 | JungleBat | g59 | - | 15x5 2dir/-2 | - | - |
| 52-53 | -- | - | - | - | - | - |
| 54 | Clothier | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 55-57 | -- | - | - | - | - | - |
| 58 | Piranha | g59 | - | 10x5 2dir/-2 | - | - |
| 59-64 | -- | - | - | - | - | - |
| 65 | Shark | g59 | - | 75x5 2dir/-2 | - | - |
| 66 | VoodooDemon | g59 | - | 50x5 2dir/-2 | - | - |
| 67-86 | -- | - | - | - | - | - |
| 87 | WyvernHead | g59 | - | 10x16 scale1.5 vel x2 noGravity [+args: Next(1,4)+n x Next(11,14)] | - | - |
| 88-105 | -- | - | - | - | - | - |
| 106 | BoundWizard | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 107 | GoblinTinkerer | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 108 | Wizard | g59 | d55(hat) | 50x5 2.5dir/-2.5 | - | - |
| 109-110 | -- | - | - | - | - | - |
| 111 | GoblinArcher | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 112-116 | -- | - | - | - | - | - |
| 117 | LeechHead | g59 | - | 10x5 dir/-1 | - | - |
| 118-119 | -- | - | - | - | - | - |
| 120 | -- | g59 | - | 50x71 alpha200 vel x1.5 | - | - |
| 121-122 | -- | - | - | - | - | - |
| 123 | BoundMechanic | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 124 | Mechanic | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 125-136 | -- | - | - | - | - | - |
| 137 | -- | g59 | - | 50x71 alpha200 vel x1.5 | - | - |
| 138 | IlluminantSlime | g59 | - | 50x71 alpha200 vel x1.5 | - | - |
| 139-141 | -- | - | - | - | - | - |
| 142 | -- | - | d55(hat) | - | - | - |
| 143 | SnowmanGangsta | g59 | - | 50x76 noGravity scale x1.2 | - | - |
| 144 | SnowBalla | g59 | - | 50x76 noGravity scale x1.2 | - | - |
| 145 | MysterySnowman | g59 | - | 50x76 noGravity scale x1.2 | - | - |
| 146 | -- | - | - | - | - | - |
| 147 | IceSlime | g59 | - | 30x (76+4-tinted) pairs | - | - |
| 148-177 | -- | - | - | - | - | - |
| 178 | -- | - | d55(hat) | - | - | - |
| 179-182 | -- | - | - | - | - | - |
| 183 | Crimslime | g59 | - | 50x136 alpha/color vel x2 | - | - |
| 184 | SpikedIceSlime | g59 | - | 30x (76+4-tinted) pairs | - | - |
| 185-189 | -- | - | - | - | - | - |
| 190 | CataractEye | g59 | - | 50x5 2dir/-2 | - | - |
| 191 | SleepyEye | g59 | - | 50x5 2dir/-2 | - | - |
| 192 | DialatedEye | g59 | - | 50x5 2dir/-2 | - | - |
| 193 | GreenEye | g59 | - | 50x5 2dir/-2 | - | - |
| 194 | PurpleEye | g59 | - | 50x5 2dir/-2 | - | - |
| 195-206 | -- | - | - | - | - | - |
| 207 | DyeTrader | g59 | d55(hat) | 50x5 2.5dir/-2.5 | - | - |
| 208 | PartyGirl | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 209 | -- | - | - | - | - | - |
| 210 | Bee | g59 | - | 6x150 Next(2) scale gate | - | - |
| 211 | BeeSmall | g59 | - | 6x150 Next(2) scale gate | - | - |
| 212 | PirateDeckhand | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 213 | PirateCaptainsMate | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 214 | PirateArcher | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 215 | PirateCorsair | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 216 | PirateCaptain | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 217-221 | -- | - | - | - | - | - |
| 222 | QueenBee | g59 | - | 50x18 alpha/color/scale | - | - |
| 223-224 | -- | - | - | - | - | - |
| 225 | UmbrellaSlime | g59 | - | 50x4 tinted alpha100 | - | - |
| 226 | -- | - | - | - | - | - |
| 227 | Painter | g59 | d55(hat) | 50x5 2.5dir/-2.5 | - | - |
| 228 | WitchDoctor | g59 | - | 50x5 2.5dir/-2.5 | - | - |
| 229 | -- | - | d55(hat) | - | - | - |
| 230-249 | -- | - | - | - | - | - |
| 250 | AngryNimbus | g60 | f5 | 20x54 smoke | f5:dust loop | gore-f5* |
| 251-260 | -- | - | - | - | - | - |
| 261 | FungiSpore | g60 | - | 20x165 alpha50 scale1.5 vel x2 noGravity | - | gore-none |
| 262 | Plantera | g60 | f5 | 150x167 [+args: n x (Next(w)+Next(h)) gore pairs (9/2/2)] | f5:dust loop | gore-f5* |
| 263 | PlanterasHook | g60 | f5 | 150x167 [+args: n x (Next(w)+Next(h)) gore pairs (9/2/2)] | f5:dust loop | gore-f5* |
| 264 | PlanterasTentacle | g60 | f5 | 150x167 [+args: n x (Next(w)+Next(h)) gore pairs (9/2/2)] | f5:dust loop | gore-f5* |
| 265 | Spore | g60 | - | 15x Next(3)!=0 166/167 [+args: hp==0 edge: NO dust] | - | gore-none |
| 266 | BrainOfCthulhu | g60 | f5 | 150x5 [+args: 6x 2x Next(-30,31) vel picks] | f5:dust loop | gore-f5* |
| 267 | Creeper | g60 | f5 | 20x5 | f5:dust loop | gore-f5* |
| 268-287 | -- | - | - | - | - | - |
| 288 | DungeonSpirit | g60 | - | 50x180 vel x2 noGravity scale1.4 | - | gore-none |
| 289-304 | -- | - | - | - | - | - |
| 305 | Scarecrow1 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 306 | Scarecrow2 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 307 | Scarecrow3 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 308 | Scarecrow4 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 309 | Scarecrow5 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 310 | Scarecrow6 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 311 | Scarecrow7 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 312 | Scarecrow8 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 313 | Scarecrow9 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 314 | Scarecrow10 | g60 | f10 | 60x194 scale1.1 | f10:dust loop | gore-f10 |
| 315 | HeadlessHorseman | g60 | f5 | 20x54 smoke | f5:dust loop | gore-f5* |
| 316-324 | -- | - | - | - | - | - |
| 325 | MourningWood | g60 | f10 | 60x191 scale1.1 | f10:dust loop | gore-f10 |
| 326 | Splinterling | g60 | - | 60x191 scale1.1 | - | gore-none |
| 327 | Pumpking | g60 | f10 | 60x189 scale1.1 | f10:dust loop | gore-f10 |
| 328 | -- | - | - | - | - | - |
| 329 | Hellhound | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 330 | Poltergeist | g60 | - | 50x192 alpha100 tinted noGravity | - | gore-none |
| 331-337 | -- | - | - | - | - | - |
| 338 | ZombieElf | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5 |
| 339 | ZombieElfBeard | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5 |
| 340 | ZombieElfGirl | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5 |
| 341 | PresentMimic | g60 | f5 | 20x196 | f5:dust loop | gore-f5* |
| 342 | GingerbreadMan | g60 | f5 | 50x0 2.5dir/-2.5 | f5:dust loop | gore-f5 |
| 343 | Yeti | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5 |
| 344 | Everscream | g60 | f10 | 60x196 | f10:dust loop | gore-f10 |
| 345-347 | -- | - | - | - | - | - |
| 348 | Nutcracker | g60 | f5 | 10x5 2dir/-2 | f5:dust loop | gore-f5 |
| 349 | NutcrackerSpinning | g60 | f5 | 10x5 2dir/-2 | f5:dust loop | gore-f5 |
| 350 | ElfArcher | g60 | f5 | 10x5 2dir/-2 | f5:dust loop | gore-f5 |
| 351 | Krampus | g60 | f5 | 10x5 2dir/-2 | f5:dust loop | gore-f5 |
| 352 | Flocko | g60 | - | 60x67 noGravity | - | gore-none |
| 353 | Stylist | g60 | d55(hat)+f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 354 | WebbedStylist | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 355-367 | -- | - | - | - | - | - |
| 368 | TravellingMerchant | g60 | d55(hat)+f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 369 | Angler | g60 | f5 | 25x31 alpha100 scale2 vel x1.4 noLight noGravity [+args: 4x Next(61,64) tombstones] | f5:dust loop | gore-f5* |
| 370 | DukeFishron | g60 | f10 | 150x5 | f10:dust loop | gore-impl |
| 371 | -- | - | - | - | - | - |
| 372 | Sharkron | g60 | f10 | 75x5 Center-(25,25) 50x50 vel/2 | f10:dust loop | gore-impl |
| 373 | -- | g60 | f10 | 75x5 Center-(25,25) 50x50 vel/2 | f10:dust loop | gore-impl |
| 374-375 | -- | - | - | - | - | - |
| 376 | SleepingAngler | g60 | f5 | 25x31 alpha100 scale2 vel x1.4 noLight noGravity [+args: 4x Next(61,64) tombstones] | f5:dust loop | gore-f5* |
| 377-378 | -- | - | - | - | - | - |
| 379 | -- | f10 | f10 | 30x5 | - | - |
| 380 | -- | - | - | - | - | - |
| 381 | BrainScrambler | g60 | f10 | 30x227 | f10:dust loop | gore-impl |
| 382 | RayGunner | g60 | f10 | 30x227 | f10:dust loop | gore-impl |
| 383 | MartianOfficer | g60 | f10 | 30x227 | f10:dust loop | gore-impl |
| 384 | ForceBubble | g60 | - | 70x226 scale0.5 | - | gore-none |
| 385 | GrayGrunt | g60 | f10 | 30x227 | f10:dust loop | gore-f10 |
| 386 | MartianEngineer | g60 | f10 | 30x227 | f10:dust loop | gore-f10 |
| 387 | MartianTurret | g60 | f10 | 30x226 scale0.5 | f10:dust loop | gore-f10 |
| 388 | MartianDrone | c12 | - | c12 style74_death_feed | - | - |
| 389 | GigaZapper | g60 | f10 | 30x227 | f10:dust loop | gore-f10 |
| 390 | ScutlixRider | g60 | f10 | 30x227 | f10:dust loop | gore-impl |
| 391 | Scutlix | g60 | f10 | 30x227 | f10:dust loop | gore-f10 |
| 392 | MartianSaucer | g60 | f10 | 30x228 scale0.5 (228 tweak inside) | f10:dust loop | gore-f10 |
| 393 | MartianSaucerTurret | g60 | f10 | 30x228 scale0.5 (228 tweak inside) | f10:dust loop | gore-f10 |
| 394 | MartianSaucerCannon | g60 | f10 | 30x228 scale0.5 (228 tweak inside) | f10:dust loop | gore-f10 |
| 395 | MartianSaucerCore | g60 | f10 | 30x228 scale0.5 (228 tweak inside) | f10:dust loop | gore-f10 |
| 396-398 | -- | - | - | - | - | - |
| 399 | MartianProbe | g60 | f10 | 10x226 scale0.5 | f10:dust loop | gore-f10 |
| 400-401 | -- | - | - | - | - | - |
| 402 | StardustWormHead | g60 | - | 5x(Next(3) gated 17 + 229) + oldPos trail Next(4) gates | - | gore-none |
| 403-406 | -- | - | - | - | - | - |
| 407 | StardustJellyfishBig | g60 | f10 | 20x(Next(3)!=0 gated 17 + 229) | f10:dust loop | gore-impl |
| 408 | -- | - | - | - | - | - |
| 409 | StardustSpiderBig | g60 | f10 | 20x(17 + 229 noGravity x3) | f10:dust loop | gore-impl |
| 410 | StardustSpiderSmall | g60 | - | 5x(17 + 229 noGravity x3) | - | gore-none |
| 411 | StardustSoldier | g60 | f10 | 20x(17 + Next(2) gated 229 noGravity) | f10:dust loop | gore-impl |
| 412 | SolarCrawltipedeHead | g60 | f10 | 6x241 + 2x6 full-args | f10:dust loop | gore-impl |
| 413 | SolarCrawltipedeBody | g60 | f10 | 6x241 + 2x6 full-args | f10:dust loop | gore-impl |
| 414 | SolarCrawltipedeTail | g60 | f10 | 6x241 + 2x6 full-args | f10:dust loop | gore-impl |
| 415 | SolarDrakomire | g60 | f10 | 40x(241 + 6 ungated) | f10:dust loop | gore-impl |
| 416 | SolarDrakomireRider | g60 | f10 | 20x(241 + Next(3) gated 6) | f10:dust loop | gore-impl |
| 417 | SolarSroller | g60 | f10 | 30x(241 + Next(2) gated 6) | f10:dust loop | gore-impl |
| 418 | SolarCorite | c12 | f10 | c12 style74_death_feed | - | - |
| 419 | SolarSelenian | g60 | f10 | 20x(241 + Next(3) gated 6) | f10:dust loop | gore-impl |
| 420 | NebulaFloater | g60 | f10 | 10x(249 + 242 ungated) | f10:dust loop | gore-impl |
| 421 | BrainSuckler | g60 | f10 | 10x(249 + 242 ungated) | f10:dust loop | gore-impl |
| 422 | -- | - | - | - | - | - |
| 423 | NebulaBeast | g60 | f10 | 20x(249 + 242 ungated) | f10:dust loop | gore-impl |
| 424 | Predictor | g60 | f10 | 20x(249 + Next(3) gated 242) | f10:dust loop | gore-impl |
| 425 | VortexRifleman | g60 | f10 | 20x(249 + Next(3) gated 242) | f10:dust loop | gore-impl |
| 426 | VortexHunter | f10 | f10 | 20x241 + 30x SelectRandom(229,240,161) | - | - |
| 427 | VortexAlien | g60 | f10 | 10x241 + 20x SelectRandom(229,229,240) | f10:dust loop | gore-impl |
| 428 | VortexHeadcrab | g60 | f10 | 10x241 + 10x Next(2) 229/240 | f10:dust loop | gore-impl |
| 429 | VortexSoldier | f10 | f10 | 20x(249 + Next(3) gated 242) | - | - |
| 430-436 | -- | - | - | - | - | - |
| 437 | -- | -(gore-only) | f10 | (no dust) 6 gore RotatedByRandom velocity | - | - |
| 438 | -- | f10 | f10 | 30x5 | - | - |
| 439 | AncientCultist | g60 | f10 | 30x5 + 10x16 [+args: Next(1,4) + n x Next(11,14)] | f10:dust loop | gore-impl |
| 440 | -- | f10 | f10 | 10x16 | - | - |
| 441 | TaxCollector | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 442-452 | -- | - | - | - | - | - |
| 453 | -- | - | d55(hat) | - | - | - |
| 454 | -- | f10 | f10 | 10x16 | - | - |
| 455 | -- | f10 | f10 | 10x16 | - | - |
| 456 | -- | f10 | f10 | 10x16 | - | - |
| 457 | -- | f10 | f10 | 10x16 | - | - |
| 458 | -- | f10 | f10 | 10x16 | - | - |
| 459 | -- | f10 | f10 | 10x16 | - | - |
| 460 | -- | f10 | f10 | 50x5 | - | - |
| 461 | -- | f10 | f10 | 50x5 | - | - |
| 462 | -- | f10 | f10 | 50x5 | - | - |
| 463 | -- | f10 | f10 | 50x5 | - | - |
| 464-465 | -- | - | - | - | - | - |
| 466 | -- | f10 | f10 | 50x5 | - | - |
| 467 | -- | - | - | - | - | - |
| 468 | -- | f10 | f10 | 50x5 | - | - |
| 469 | -- | f10 | f10 | 50x5 | - | - |
| 470 | -- | - | - | - | - | - |
| 471 | GoblinSummoner | g60 | f5 | 50x5 | f5:dust loop | gore-f5* |
| 472-476 | -- | - | - | - | - | - |
| 477 | -- | f10 | f10 | 50x237 | - | - |
| 478 | -- | f10 | f10 | 50x(237+Next(2)) | - | - |
| 479 | -- | f10 | f10 | 50x237 | - | - |
| 480-488 | -- | - | - | - | - | - |
| 489 | BloodZombie | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5 |
| 490 | Drippler | g60 | f5 | 50x5 2.5dir/-2.5 [+args: 2x Next(3) wing picks] | f5:dust loop | gore-f5 |
| 491 | FlyingDutchman | f10 | f10 | 120x31 tail | - | - |
| 492 | FlyingDutchmanCannon | f10 | f10 | 7x31 | - | - |
| 493-509 | -- | - | - | - | - | - |
| 510 | -- | f10 | f10 | 50x250 alpha/scale | - | - |
| 511 | -- | f10 | f10 | 50x250 alpha/scale | - | - |
| 512 | -- | f10 | f10 | 50x250 alpha/scale | - | - |
| 513 | -- | f10 | f10 | 50x250 alpha/scale | - | - |
| 514 | -- | f10 | f10 | 50x250 alpha/scale | - | - |
| 515 | -- | f10 | f10 | 50x250 alpha/scale | - | - |
| 516-517 | -- | - | - | - | - | - |
| 518 | SolarSpearman | g60 | f10 | 20x(241 + Next(3) gated 6) | f10:dust loop | gore-impl |
| 519 | -- | - | - | - | - | - |
| 520 | MartianWalker | g60 | f10 | 30x226 scale0.5 | f10:dust loop | gore-f10 |
| 521-523 | -- | - | - | - | - | - |
| 524 | -- | f10 | f10 | 20x(5 + Next(3) gated accent) | - | - |
| 525 | -- | f10 | f10 | 20x(5 + Next(3) gated accent) | - | - |
| 526 | -- | f10 | f10 | 20x(5 + Next(3) gated accent) | - | - |
| 527 | -- | f10 | f10 | 20x(5 + Next(3) gated accent) | - | - |
| 528 | -- | f10 | f10 | 20x5 | - | - |
| 529 | -- | f10 | f10 | 20x5 | - | - |
| 530 | -- | f10 | f10 | 20x250 alpha100 | - | - |
| 531 | -- | f10 | f10 | 20x250 alpha100 | - | - |
| 532 | -- | f10 | f10 | 20x5 alpha100 | - | - |
| 533 | DesertDjinn | g60 | - | 40x27 alpha50 scale1.5 vel x2 noGravity fadeIn1 | - | gore-none |
| 534-540 | -- | - | - | - | - | - |
| 541 | -- | f10 | f10 | 60x268 tinted noGravity | - | - |
| 542-545 | -- | - | - | - | - | - |
| 546 | -- | f10 | f10 | 40x268 tinted + 20x54 | - | - |
| 547-549 | -- | - | - | - | - | - |
| 550 | DD2Bartender | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 551 | DD2Betsy | g60 | e5+f10 | 20x5 | e5:dd2_death_dust; f10:dust loop; f10:betsy_flipped (e5 twin keeps) | gore-impl |
| 552 | DD2GoblinT1 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 553 | DD2GoblinT2 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 554 | DD2GoblinT3 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 555 | DD2GoblinBomberT1 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 556 | DD2GoblinBomberT2 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 557 | DD2GoblinBomberT3 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 558 | DD2WyvernT1 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 559 | DD2WyvernT2 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 560 | DD2WyvernT3 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 561 | DD2JavelinT1 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 562 | DD2JavelinT2 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 563 | DD2JavelinT3 | g60 | e5 | 20x5 | e5:dd2_death_dust | gore-impl |
| 564 | DD2DarkMage | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 565 | DD2DarkMageT3 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 566 | DD2SkeletonT1 | g60 | f10 | 20x236 [+args: 6x Next(2) gates] | f10:dust loop | gore-impl |
| 567 | DD2SkeletonT3 | g60 | f10 | 20x236 [+args: 6x Next(2) gates] | f10:dust loop | gore-impl |
| 568 | DD2WitherBeast | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 569 | DD2WitherBeastT3 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 570 | DD2DrakinT2 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 571 | DD2DrakinT3 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 572 | DD2KoboldWalkerT2 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 573 | DD2KoboldWalkerT3 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 574 | DD2KoboldFlyerT2 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 575 | DD2KoboldFlyerT3 | g60 | f10 | 20x5 | f10:dust loop | gore-impl |
| 576 | DD2OgreT2 | g60 | f10 | 20x273 [+args: 1x Next(5) head pick] | f10:dust loop | gore-impl |
| 577 | DD2OgreT3 | g60 | f10 | 20x273 [+args: 1x Next(5) head pick] | f10:dust loop | gore-impl |
| 578 | -- | f10 | f10 | 20x5 | - | - |
| 579-587 | -- | - | - | - | - | - |
| 588 | Golfer | g60 | d55(hat)+f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 589 | SleepingGolfer | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 590-593 | -- | - | - | - | - | - |
| 594 | -- | - | d55 | - | - | - |
| 595-617 | -- | - | - | - | - | - |
| 618 | -- | f10 | f10 | 200x5 direct | - | - |
| 619 | -- | f10 | f10 | 150x5 direct vel x2.5 fadeIn1.7 | - | - |
| 620 | -- | f10 | f10 | 150x5 direct vel x2.5 scale1.7 | - | - |
| 621 | -- | f10 | f10 | 50x5 alpha/scale | - | - |
| 622 | -- | f10 | f10 | 50x5 alpha/scale | - | - |
| 623 | -- | f10 | f10 | 50x5 alpha/scale | - | - |
| 624-632 | -- | - | - | - | - | - |
| 633 | Zoologist | g60 | f5 | 50x5 2.5dir/-2.5 | f5:dust loop | gore-f5* |
| 634-635 | -- | - | - | - | - | - |
| 636 | -- | e5(plan) | d55 | 50x Next(2) 67/69 (count-plan) | - | - |
| 637-656 | -- | - | - | - | - | - |
| 657 | -- | e5 | e5 | 150x4 tinted + Next(5) noGravity gate (gore-FIRST) | - | - |
| 658 | -- | e5 | e5 | 40x4 tinted noGravity Next(3) (hit 7x) | - | - |
| 659 | -- | e5 | e5 | 40x243 noGravity Next(3) (hit 7x) | - | - |
| 660 | -- | e5 | e5 | 40x4 tinted noGravity Next(3) (hit 7x) | - | - |
| 661-662 | -- | - | - | - | - | - |
| 663 | Princess | g60 | f5 | 25x31 alpha100 scale2 vel x1.4 noLight noGravity [+args: 4x Next(61,64) tombstones] | f5:dust loop | gore-f5* |
| 664-666 | -- | - | - | - | - | - |
| 667 | -- | e5 | e5 | 40x10 alpha80 scale1.5 noGravity Next(3) (hit 7x) | - | - |
| 668 | Deerclops | g60 | d55 | 120x5 box+15 scale1.5 + Circular(4,4) | d55:deerclops_death dust | gore-impl |
| 669-685 | -- | - | - | - | - | - |
| 686 | -- | -(hit:g62) | d55 | - | - | - |
| 687-697 | -- | - | - | - | - | - |

---

## 10. Execution checklist (the keep-one-exec phase)

1. **Union `npc_death_dust.rs`** (g59 `death_dust` + g60 `death_dust_feed`),
   one `mod` in `main.rs`, both death-retain calls in `npcs.rs`, dedupe
   `Npc.last_hit_dir`. (§8)
2. **d55 wiring:** strip `deerclops_death`'s 120-dust burst (668); turn
   `empress_hit_dust_draws` (636) into a pool feed. (§5.1)
3. **e5 wiring:** strip `dd2_death_dust` from the 551-557/561-563 gore fns; keep
   `dd2_crystal_death`'s BOTH 551 branches. (§5.2)
4. **f10 wiring:** strip the dust loops for the 44 overlap kinds (§5.3 table);
   **delete `betsy_flipped_death`** (e5's twin keeps); strip the re-drawn
   566/567 six gates + 576/577 Next(5) + 439 bone-loop draws (read the
   g60-consumed values); strip 418's dust (c12 owns). Keep the f10-only kinds'
   dust whole. (§5.3)
5. **c12 wiring:** turn the 388/418 count-plans into pool feeds at the death
   retain. (§5.4)
6. **GORE-UNIMPL remainder — RE-CENSUSED (§11).** The 66 rows are no longer
   unclaimed: f10's pushed continuation (`67c51ec7`, through `:85334`) owns 24
   of them, f5's wave-D claim (`97c70c97`, the whole `:85560-89835` chain) owns
   32, and 10 have NO gore sites at all. **g60 ports NO gore bodies** — the
   ARG draws stay in `death_dust_draws`, exactly once. The wiring strips grow:
   f10 +24 kinds, f5 +32 kinds (11 already pushed). (§11 table)
7. **Verify:** run the witness-union (§7) + the merger-added keep-one
   stream-parity witness; `cargo test --workspace --skip
   migrates_version_2_kinds_to_item_ids` green on the merged branch.

---

## 11. The gore-remainder re-census (the 66, g60 gore-remainder phase)

**Probe.** f10 tip re-read `67c51ec7` (wave-C batch 60, `NPC.cs:85271-85334` —
the pushed window now `:82206-85334`, batches 24-60 landed the continuation the
playbook probe `a69fb00c` listed as in-flight). f5 tip re-read `b33ba0a9`
(wave-D batch 33, `:86986`); its phase-2 claim `97c70c97` claims **the WHOLE
`NPC.cs:85560-89835` classic-mob chain, 853 `Gore.NewGore` sites** — the claim
text explicitly counts the gore bodies and ports the death-half dust loops as
their draw census ("keep-one at merge"). **f5 wave-D therefore claims the GORE
too**, not just the dust sites the task brief assumed.

**Consequence: ZERO unclaimed gore bodies among the 66.** Every death arm was
located in the C# and classified (per-kind draw counts below; the g60 feed
already pays every dust draw + every gore ARGUMENT draw — no port changes were
needed, only the census + witnesses). Buckets:

* **gore-f10 (24):** the gore bodies are PUSHED in `npc_hit_effect_c.rs` —
  305-314 (batch 51), 325 (54), 327 (52), 344 (55), 385/386/387 (45), 389/520
  (46), 391/399 (47), 392-395 (48). **Their dust loops are NEW strip rows** at
  f10 wiring (the §5.3 table's 44 grows to 68).
* **gore-f5 (11 pushed):** 338-340 (batch 32), 342 (31), 343 (32), 348/349
  (14), 350/351 (15), 489 (24), 490 (25). The dust loops are PUSHED re-draws —
  strip at f5 wiring; the 490 witness's TWO `Next(3)` wing picks are ARG draws
  g60 already consumed (f5 batch 25 re-draws them — the 566/567-gates pattern).
* **gore-f5\* (21 claimed, not yet pushed):** arms past `:86986` — 250/262-264/
  266/267/315/329/341/353/354/368/369/376/441/471/550/588/589/633/663. When
  those batches land they MUST NOT re-draw: the Plantera pairs (18/4/4), the
  Brain's 12 velocity draws, the four tombstone `Next(61,64)` picks (369/376/
  663 share ONE arm — strip once, not thrice).
* **gore-none (10):** 261/265/288/326/330/352/384/402/410/533 — the death half
  has **zero `NewGore` sites** (261's three gores are `type != 261`-gated;
  265's arm returns; 288/326/330/352/384/402/410/533 are dust-only ifs). No
  gore lane carries them because there is nothing to carry; g60's feed is
  their whole death-effects port. NOT a skip — nothing is missing.

### Per-kind census (the 66; gore types/velocities in stream order)

`vel` column: `v` = `base.velocity` unchanged; `(hd,0)×0.3` = argument
`(hitDirection,0)` + post-spawn `velocity *= 0.3`; `×0` = argument
`base.velocity * 0f`. All gores take `this.scale` unless noted. **ARG** =
argument draws g60's feed consumes (the gore lane must NOT re-draw).

| kind | death arm (`NPC.cs`) | gore sites (stream order) | vel / anchor notes | ARG draws (g60-owned) | bucket |
| --- | --- | --- | --- | --- | --- |
| 250 | :88971-88982 | 99 ×3 | (hd,0)×0.3 at Y-10 / Y+h/2-15 / Y+h-20 | — | gore-f5* |
| 261 | :89065-89079 | NONE (375/376/377 are `type!=261`-gated) | — | — | gore-none |
| 262 | :87186-87207 | 381/382/383/384/385/386/386/387/387 (9) | v at position+(Next(w),Next(h)) per gore | 9×(Next(w)+Next(h)) = 18 | gore-f5* |
| 263 | :87186-87212 | 390/391 (2) | same shape | 2 pairs = 4 | gore-f5* |
| 264 | :87186-87217 | 388/389 (2) | same shape | 2 pairs = 4 | gore-f5* |
| 265 | :87220-87236 | NONE (`life>=0` return; dust-only tail) | — | — | gore-none |
| 266 | :87248-87257 | 396/397/398/399/400/401 (6) | (Next(-30,31)×0.2, Next(-30,31)×0.2) at position, NO scale | 6×2 = 12 | gore-f5* |
| 267 | :87269-87273 | 402 (1) | v at position, NO scale | — | gore-f5* |
| 288 | :85000-85009 | NONE (dust-only if) | — | — | gore-none |
| 305-314 | :84806-84814 | 441/442/443/444/445 (5) | v at pos / +20×2 / +34×2 | — | gore-f10 (b51) |
| 315 | :89038-89051 | 99 ×3 + 446 (4) | (hd,0)×0.3 ×3; 446 ×0.1 at pos | — | gore-f5* |
| 325 | :84849-84856 | 469..475 loop (7) | ×0 at position | — | gore-f10 (b54) |
| 326 | :84903-84906 | NONE | — | — | gore-none |
| 327 | :84828-84835 | 476..484 loop (9) | ×0 at position | — | gore-f10 (b52) |
| 329 | :87745-87751 | 485/486/487 (3) | v at position, NO scale | — | gore-f5* |
| 330 | :88996-89000 | NONE | — | — | gore-none |
| 338 | :86878-86896 | 494 (head) + 497/498 (+20Y) + 499 (+34Y) (4) | v; +20/+34 Y ladder | — | gore-f5 (b32) |
| 339 | same arm | 495 head + 497/498/499 tail (4) | same | — | gore-f5 (b32) |
| 340 | same arm | 496 head + 497/498/499 tail (4) | same | — | gore-f5 (b32) |
| 341 | :88755-88764 | 61/62/63 (3) | (hd,0)×0.3 at Y-10 / Y+h/2-10 / Y+h-10 | — | gore-f5* |
| 342 | :86858-86866 | 500/501×2/502×2 (5) | v; +20/+34 Y | — | gore-f5 (b31) |
| 343 | :86908-86915 | 503 (pos) / 504/505 (+20Y) / 506 (+34Y) (4) | v | — | gore-f5 (b32) |
| 344 | :84878-84889 | 508..512 + 509..512 loops (9) | ×0 at position | — | gore-f10 (b55) |
| 348/349 | :86322-86330 | 528..532 (5) | v at position, scale 1f (f5 b14) | — | gore-f5 (b14) |
| 350 | :86362-86369 | 533..536 (4) | same | — | gore-f5 (b15) |
| 351 | :86342-86350 | 538..542 (5) | same | — | gore-f5 (b15) |
| 352 | :84788-84792 | NONE | — | — | gore-none |
| 353/354 | :88208-88220 | 544 (altTexture==0) / 543 / 546×2 (+20Y) / 545×2 (+34Y) — shimmer 1351/1352/1353/1354 | v; conditional head | — (picks are data) | gore-f5* |
| 368 | :88033-88044 | 567 (altTexture==0) / 568×2 (+20Y) / 569×2 (+34Y) — shimmer 1362/1364/1365 | v | — | gore-f5* |
| 369/376/663 | :88048-88073 (ONE shared arm; the :88411 second 663 arm is dead code) | Next(61,64) ×4 tombstones | default vel at pos+(w/2-24, h/2-24), scale=1f, velocity ±1 X/Y quadrant writes | 4× Next(61,64) | gore-f5* |
| 384 | :84355-84359 | NONE | — | — | gore-none |
| 385 | :84373-84379 | 597/598/599 (3) | v at pos/+20/+34 | — | gore-f10 (b45) |
| 386 | :84393-84399 | 600/601/602 (3) | same | — | gore-f10 (b45) |
| 387 | :84415-84422 | 603/603/604 (3) | same | — | gore-f10 (b45) |
| 389 | :84461-84467 | 904/905/906 (3) | same | — | gore-f10 (b46) |
| 391 | :84545-84551 | 607/608 (pos+UnitX×w/2f) / 609 (+20Y) | v | — | gore-f10 (b47) |
| 392 | :84567-84589 | NONE (the base saucer spawns no gores) | — | — | gore-f10 (b48, dust strip only) |
| 393 | same arm | 616 (1) | v at Center | — | gore-f10 (b48) |
| 394 | same arm | 617 (1) | v at Center | — | gore-f10 (b48) |
| 395 | same arm | 610/611/612/613/614/615 (6) | v at Center+(-20,-10)/(20,-10)/(-50,0)/(50,0)/(-60,10)/(0,30) | — | gore-f10 (b48) |
| 399 | :84515-84531 | 898/899 (2) | Center-(10,10); ai[0]==3 → rotation.ToRotationVector2 (X-flip on spriteDirection==1) ×5 — data, no draws | — | gore-f10 (b47) |
| 402 | :83601-83631 | NONE (head burst + oldPos trail, dust only) | — | trail Next(4)/Next(3) gates (dust-side) | gore-none |
| 410 | :83523-83529 | NONE (5×(17+229), ungated death half) | — | — | gore-none |
| 441 | :87685-87693 | 768 / 769×1 (+20Y) / 770 (+20Y) / 771×2 (+34Y) — shimmer 1358/1359/1361/1360 (NOTE the 770↔1361/771↔1360 cross) | v; NO scale | — | gore-f5* |
| 471 | :89617-89625 | 675/676/677/678/678 (5) | v at pos/+20×2/+34×2 | — | gore-f5* |
| 489 | :86669-86679 | 722/723×2/724×2 (5) | v; +20/+34 | — | gore-f5 (b24) |
| 490 | :86720-86728 | 725/726/727 + (725+Next(3)) ×2 (5) | v at position | 2× Next(3) — f5 b25 RE-DRAWS them today; strip at wiring | gore-f5 (b25) |
| 520 | :84438-84447 | 851/852 (pos) + 848/849/850 (+80Y legs) | v | — | gore-f10 (b46) |
| 533 | :83880-83886 | NONE | — | — | gore-none |
| 550 | :87952-87960 | 967 / 968×2 (+20Y, second at +8X) / 969×2 (+34Y, +8X twin) — shimmer 1355/1356/1357 | v; NO scale | — | gore-f5* |
| 588/589 | :87972-87983 | 1109 (altTexture==0) / 1110×2 / 1111×2 — shimmer 1311/1312/1313 | v; +20/+34 Y, +8X twins | — | gore-f5* |
| 633 | :87995-88021 | head/torso picks 1219/1220 (altTexture==2 → 1223/1224; shimmer → 1377/1379 or 1378/1380) + 1221/1222/1222 legs (shimmer 1382/1381/1381) | v; +20/+34, +8X twins; picks are DATA (no draws) | — | gore-f5* |

**Counts check:** 24 gore-f10 + 11 gore-f5 + 21 gore-f5\* + 10 gore-none = 66.

### New strip rows at wiring (the keep-one deltas)

* **f10** (`npc_hit_effect_c.rs`, pushed): strip the dust loops of the 24
  gore-f10 kinds (the §5.3 table's 44 → **68**). All their gore calls are
  constant-argument — bodies + kicks stay, zero draw hand-offs beyond the
  loops themselves. The saucer 392-395 strip includes the 228-tweak re-roll
  (g60's feed pays it inside `new_dust`).
* **f5** (`npc_hit_effect_d.rs`): the claim already says "the death-half dust
  loops ONLY as the gore bodies' draw census … keep-one at merge" — the
  wiring strips all 32 f5-owned kinds' loops. **The one ARG hand-off:** the
  490's two `Next(3)` wing picks (f5 batch 25 consumes them per-gore BEFORE
  the spawn — replace with the g60-consumed values, the 566/567-gates
  pattern). The 262-264 position pairs, the 266 velocity pairs and the
  369/376/663 tombstone picks are claimed-not-pushed: those batches must
  read the stream position g60 left, NOT re-roll.
* **gore-none (10):** nothing to strip anywhere; g60's feed is whole.

### Witnesses added (this phase, WRITTEN NOT RUN)

Five test fns in `npc_death_dust.rs`, covering **all 66 kinds with per-kind
draw-count rows** (the 18 pre-existing witnesses already pinned 261/265/266/
288/305/392/402/490/663):

* `w4_g60_gore_tail_f5_arms_pay_dust_only` — 24 rows (250/315/329/341/353/
  354/368/441/471/550/588/589/633/267/338/339/340/342/343/348/349/350/351/
  489 — one row each): exact dust count + type + zero gore draws.
* `w4_g60_gore_tail_f10_arms_pay_dust_only` — 23 martian/pumpkin rows (385/
  386/387/389/391/399/520/393/394/395/306-314/325/327/344) + the 410 pair
  block: 226/227/228-tweak/1.1-scale shapes pinned per arm.
* `w4_g60_gore_tail_plantera_trio_pays_per_kind_position_pairs` — 262/263/
  264: 150×167 + 18/4/4 pair draws.
* `w4_g60_gore_tail_angler_pair_pays_the_four_tombstone_picks` — 369/376:
  25×31 + 4 picks (663 rides the pre-existing tombstone witness).
* `w4_g60_gore_tail_no_gore_arms_pay_dust_only` — 326/330/352/384/533: the
  dust-only arms' exact counts + post-write shapes.

The witness-union (§7) grows g60 22 → **27**.

---

*Generated from the lane tips d55 `ec78d152` / e5 `a62a32ce` / f10 `a69fb00c` /
g59 `eea6975b` / g60 `e55e2272` (merge-base `a725acb0`). The matrix was built by
walking each lane's full dispatch (`hit_effect`/`hit_effect_b`/`hit_effect_c`,
`death_dust`/`death_dust_draws`/`lane_covers`) against the g60 census artifacts
(`g60_kmap.json`, `g60_eligible.txt`, `g60_death_order.json`) and the C#
`NPC.cs:81654-89834` death arms. §11 re-censused at f10 `67c51ec7` / f5
`b33ba0a9` (claim `97c70c97`). Code-only: no cargo/tests/graphify invoked.*
