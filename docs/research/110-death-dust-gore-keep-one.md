# 110 — NPC death dust × gore keep-one merger playbook (W4-g60)

**Status:** ✅ FINAL-FINAL (code-only; the wiring wave executes the strips; §12 the final-status refresh + §12.8 the bible phase — the g62 widening CONSUMED (g60's half EXECUTED, `19431e385`), f5 CHAIN COMPLETE (853/853, remaining window 0), the f10-overrun drop-hunks EXACT per-fn line-ranges, the 185-row strip total, the union-8-step re-pinned)
**Lane:** W4-g60 (`task/W4-g60-npc-death-effects-2`, wiring DONE `e55e2272`, playbook DONE `3b5f9b3c`, gore-tail DONE `ed7cef6e`, FINAL STATUS `402b32f2`, widening DONE `19431e385`, FINAL-FINAL this commit)
**Companions:** W4-g59 (lane 1), W4-d55 / W4-e5 / W4-f10 / W4-f5 (the gore lanes), W4-g62 (the hit-dust lane), W4-c12 (the in-flight death feed)

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

## 1. The lanes (FINAL-FINAL tip pin, 2026-11-21 — see §12/§12.8 for the movement log)

| lane | branch | tip read (FINAL-FINAL) | module(s) | scope |
| --- | --- | --- | --- | --- |
| **g59** | `task/W4-g59-npc-death-effects-1` | `29c13d103` (DOC-SWEEP b11-13 done) | `npc_death_dust.rs::death_dust` | death dust kinds **1-230**: **113 census kinds / 111 wired match types** (batches 1-13: the 64 present-clean + 24 d35 + 23 b11-13 row-gated activations); **56 witnesses** (53 module + 3 seam); the 83/84 + 87 gore-ARG tails consumed |
| **g60** | `task/W4-g60-npc-death-effects-2` | `19431e385` (widening EXECUTED) | `npc_death_dust.rs::death_dust_feed` → `death_dust_draws` | death dust kinds **231-697** (121 census-present / **120 `lane_covers`** cs_ids); **29 witnesses** (23 module + 2 npcs.rs + 4 npc.rs — the 2 g62-adopted widening witnesses added); the DeathFx ring WIDENED `[Vec2; 60]`+`trail_len` (§12.8) |
| **d55** | `task/W4-d55-gore-table-sites` | `ec78d152` (unchanged) | `npc_hit_effect_a.rs::hit_effect` | gore wave A, sites 1-30 (`NPC.cs:81660-81941`) |
| **e5** | `task/W4-e5-gore-sites-2` | `a62a32ce` (unchanged) | `npc_hit_effect_a.rs` (e5 copy) + `npc_hit_effect_b.rs::hit_effect_b` | gore wave B, sites 31-86 (`NPC.cs:81942-82216`) |
| **f10** | `task/W4-f10-gore-sites-3` | `91b082e3c` (batch 134, `:88220`) — **STILL OVERRUNNING, ALIVE** (§12.2/§12.8) | `npc_hit_effect_c.rs::hit_effect_c` | wave C sites 1-561 legit (`:82206-85559`); **batches 62-134 (sites 562-1149, `:85684-88220`) double-port f5's classic-mob chain — 88 cite-bearing fns + the cite-less `five_gore_ladder`, 85 gates, 97 witnesses** (§12.8 drop contract); **191 witnesses** total (94 legit) |
| **f5** | `task/W4-f5-dust-proj-3` | `d0619d8e9` (completion docs; code tip `fa3c526d5` batch 103) — **CHAIN COMPLETE 853/853** | `npc_hit_effect_d.rs::hit_effect_d` | wave D claim `97c70c97` EXECUTED WHOLE: `NPC.cs:85560-89835`, **853 gore sites, remaining window 0**; **142 witnesses**; the NO-gore arms recorded-not-ported (§12.8) |
| **g62** | `task/W4-g62-npc-hit-dust` | `32af98b44` (TRAILCACHE WIDENING record — the 6-hunk contract) | `npc_hit_dust.rs::replay_hit_dust` | the HIT (`life > 0`) halves; rings WIDENED `[60]`+`trail_len`; the 402 HIT `:83579` + 519 HIT `:83011` re-derived (36-gate / 7×5); **66 witnesses** (61 module + 5 npc.rs); g60's half EXECUTED (§12.8 receipt) |
| **c12** | `task/W4-c12-style74-martian-drone` | `5778e596` (phase 2 done, unchanged) | `npc_ai_style74.rs` `style74_death_feed` + `drone_hit_dust_plan`/`corite_hit_dust_plan` | the **388/418** in-flight death feed (count-plans, not yet pool-fed) |

All five branches share the merge-base `a725acb0` ("Land wave-3 magic+summon
lane"). `death.rs` (the death-cue/loot retain) is **byte-identical** across
d55/e5/f10/g59/g60 — none of these lanes touches it; the death **dust** seam is
`world_sim/npcs.rs`'s death retain, not `death.rs`.

**g59b/g59c note (FINAL-FINAL):** g59's tip is now `29c13d103` (batches 1-13 +
the DOC-SWEEP; **113 census kinds / 111 wired match types**, the 24 d35 +
23 b11-13 row-gated activations landed: 31/34/37/39/40/41/43/47/48/56/57/61/62/
63/157/175/217/218/219/220/221/226/229). The boundary check (§6) still holds —
every activated cs_id ≤ 229 ≤ 230, zero >230 flags (g59's own DOC-SWEEP
re-verified it). The g59 lane's NEW overlap is with the GORE lanes' dust
re-draws (f5's **93 kinds** — grew from 57+18 as g59 b11-13 activated the
town/skeleton/beetle/sea-snail/pirate families f5 already re-drew — + f10's 225
— §12.3/§12.8), not with g60.

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
**FINAL-STATUS delta (§12):** + **57** g59∩f5 (pushed) + **1** g59∩f10 (225) =
**168 actionable**; + **18** conditional g59∩f5 (f5 remaining window) = **186
eventual**. The f10↔f5 boundary overrun (§12.2) is a wholesale ~38-fn deletion
(f10's classic-mob duplicates), counted separately. The per-row mechanical
checklist is §12.4.

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

### g59 (42 witnesses at `dd336ef8`, `npc_death_dust.rs` tests — FINAL)
Batches 1-7's 32 (below) + batches 8-10's 10 d35 row-gated witnesses (the mummy
trio/wraith, the cursed pair's gore-velocity draws, pixie, goblin-scout/skeleton
pair, the mimic ai[3] selector, digger-seeker/corruptor, clinger, gastropod,
possessed-armor, green-jellyfish). The original 32:
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

### f10 (134 wave-C witnesses at `f7a9dd78`, `npc_hit_effect_c.rs` — FINAL)
Each batch's gore bodies + the dust-loop stream replay. **After the strips**, the
68 overlap kinds' dust-draw assertions move to g60's feed (225's to g59's); the
gore witnesses keep the bodies. The **566/567 six-gate** and **576/577 Next(5)**
witnesses must be re-pointed to read the g60-consumed gate values (the hand-off
seam), NOT re-roll. The f10-only kinds' witnesses stay whole. **The §12.2
overrun's witnesses (batches 62-92, the classic-mob duplicates) are DELETED with
their fns** — f5's 124 `w4_f5d_*` witnesses are the surviving verification set
for that window.

### f5 (124 wave-D witnesses at `d908a4fd`, `npc_hit_effect_d.rs` — FINAL)
The classic-mob chain's per-batch gore bodies + dust-census replays. **After the
strips** (§12.4), the 32 g60 kinds' + 57 g59 kinds' dust-draw assertions move to
the dust lanes' feeds; the gore witnesses keep the bodies. The **Plantera
position-pair / Brain velocity-pair / tombstone-pick / Drippler wing-pick**
witnesses must be re-pointed to the g60-consumed values (the hand-off seam).

### g62 (61 hit-dust witnesses at `53fdbf07`, `npc_hit_dust.rs` — FINAL)
The HIT-half lane's live-arm replays + the strike-capture seam + the adopted
g60 ring witnesses (the 2 `npc.rs` trail witnesses dedupe with g60's — §12.7).
No death-half overlap with g59/g60 (the `life > 0` gate); the 402 HIT/DEATH
sites are disjoint.

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
| `world_sim/npcs.rs` | +22 (the `death_dust` call) | +160 (the `death_dust_feed` call + 2 twin witnesses) | — | **keep BOTH calls** at the death-retain site (disjoint by kind); union the witness mods. g62 adds the hit-dust drain + the pooled `update_trails` tail call (§12.7) — keep both |
| `npc.rs` | +`last_hit_dir` field/ctor/3 strike records/bubble reset + Crimslime alpha-55 | +`last_hit_dir` (same shape) + `old_pos`/`old_rot` rings + `update_trails` | — | **`last_hit_dir` is added by c12/g60/g59** — dedupe to ONE declaration, keep BOTH fuse resets (§12.6); **`update_trails` + rings added by g60/g62 byte-identical** — dedupe to ONE (§12.7); keep g59's Crimslime alpha row |
| `main.rs` | +`mod npc_death_dust;` (line ~66) | +`mod npc_death_dust;` (line ~160) | +`mod npc_hit_effect_{a,b,c,d};` | **one `mod npc_death_dust;`** (the module is shared g59/g60 — §12.5); keep the gore mods + g62's `npc_hit_dust` |
| `npc_death_dust.rs` | g59's `death_dust` (1-230) | g60's `death_dust_feed`/`death_dust_draws`/`lane_covers` (231-697) | — | **UNION into one module** — both halves coexist (disjoint matches); the ONE semantic collision is the test `air()` world height (§12.5) |
| `anchors.txt` | +1 row (`pub fn death_dust`) | +1 row (`pub(crate) fn death_dust_draws`) | — | keep BOTH rows (distinct markers) |
| `death.rs` | unchanged | unchanged | unchanged | no conflict (byte-identical) |

**The `npc_death_dust.rs` collision is the key merge:** g59 and g60 BOTH create
`crates/tr-app/src/npc_death_dust.rs` with **different content** (g59's
`death_dust` for 1-230 vs g60's `death_dust_feed` for 231-697). The merger must
**union the two halves into one file** (g59's `death_dust` fn + g60's
`death_dust_feed`/`DeathFx`/`death_dust_draws`/`lane_covers`), keep one
`mod npc_death_dust;` in `main.rs`, and keep both death-retain calls. The two
halves share the `DustPool`/`SpawnCtx`/`UnifiedRandom` imports and the
`Npc.last_hit_dir` field. The production code is a pure textual collision
(concatenation, not pick-a-side — the AGENTS.md semantic-conflict rule); the ONE
semantic collision is the test `air()` helper's world height (g59 200×200 vs
g60 200×120) — keep BOTH under distinct names. **The full field/fn inventory
diff + the 8-step union plan are §12.5.**

---

## 9. Per-kind matrix (1-697)

Columns: **kind** (vanilla `NPC.type`; consecutive unowned kinds compressed to a
band) · **port kind(s)** (the `NpcKind` variant(s), `--` = no port row) ·
**dust-owner** · **gore-owner** (`d55(hat)` = the cross-cutting party-hat gore) ·
**draw-count** (the dust-owner's burst shape; `[+args: …]` = gore ARGUMENT draws
the dust lane consumes) · **STRIP-list** (the gore-lane dust loops to delete at
wiring) · **gore-tail** (g60 kinds: `gore-impl` vs the §11 re-census values
`gore-f10` / `gore-f5` / `gore-f5*` / `gore-none`).

> **FINAL-STATUS matrix caveat (§12):** the g59 rows' STRIP-list reads `-`
> because the playbook phase audited only the g60 overlaps. §12.3 adds the
> **g59∩f5 = 57 pushed kinds (+18 conditional)** and **g59∩f10 = {225}** — at
> wiring, every g59 kind listed in §12.3 carries an `f5:dust loop` (or
> `f10:dust loop` for 225) strip entry. The g60 `gore-f5*` rows: 18 of the 21
> are now PUSHED (`gore-f5*` → `gore-f5`); only 250/315/471 remain claimed
> (§12.1). The f10 classic-mob duplicates (§12.2) are deleted wholesale, not
> stripped.

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

## 10. Execution checklist (the keep-one-exec phase — FINAL ordering)

1. **Union `npc_death_dust.rs`** (g59 `death_dust` + g60 `death_dust_feed`),
   one `mod` in `main.rs`, both death-retain calls in `npcs.rs`; resolve the
   test `air()` height collision (§12.5). Dedupe `Npc.last_hit_dir` to ONE
   declaration, keep BOTH fuse resets (§12.6). Dedupe `update_trails` + the
   rings to ONE (g60 ≡ g62), keep g62's pooled tail call + `HitDustSnapshot`
   (§12.7). (§8)
2. **d55 wiring:** strip `deerclops_death`'s 120-dust burst (668); turn
   `empress_hit_dust_draws` (636) into a pool feed. (§5.1)
3. **e5 wiring:** strip `dd2_death_dust` from the 551-557/561-563 gore fns; keep
   `dd2_crystal_death`'s BOTH 551 branches. (§5.2)
4. **f10 wiring:** strip the dust loops for the 68 overlap kinds (§5.3's 44 +
   §11's 24; the §12.4 checklist); **delete `betsy_flipped_death`** (e5's twin
   keeps); strip the re-drawn 566/567 six gates + 576/577 Next(5) + 439
   bone-loop draws (read the g60-consumed values); strip 418's dust (c12 owns)
   + 225's dust (g59 owns). Keep the f10-only kinds' dust whole. (§5.3, §12.4)
5. **f10 OVERRUN deletion (§12.2):** delete the classic-mob duplicate surface —
   every fn in the §12.2 table (sites 562-816, `:85684-86784`) + their
   `hit_effect_c` dispatch gates + witnesses. f5 is the sole classic-mob gore
   writer (its claim predates f10's overrun).
6. **f5 wiring:** strip the dust loops for the 32 g60 kinds + the 57 g59 kinds
   (§12.3/§12.4); hand off the Plantera pairs / Brain velocities / tombstone
   picks / Drippler wings (read the g60-consumed values). Keep ALL f5 gore
   bodies + kicks. The 3 unpushed kinds (250/315/471) strip when they land.
   (§12.4)
7. **c12 wiring:** turn the 388/418 count-plans into pool feeds at the death
   retain. (§5.4)
8. **GORE-UNIMPL remainder — RE-CENSUSED (§11), FINAL (§12.1).** The 66 rows:
   f10 owns 24 (pushed), f5 owns 32 (29 pushed at `d908a4fd`, 3 in the remaining
   window), 10 have NO gore sites. **g60 ports NO gore bodies** — the ARG draws
   stay in `death_dust_draws`, exactly once. (§11 table, §12.1 census)
9. **Verify:** run the witness-union (§7 — g59 42 / g60 27 / d55 7 / e5 12 /
   f10 134 minus the overrun / f5 124 / g62 61) + the merger-added keep-one
   stream-parity witness; `cargo test --workspace --skip
   migrates_version_2_kinds_to_item_ids` green on the merged branch.

---

## 11. The gore-remainder re-census (the 66, g60 gore-remainder phase)

> **FINAL-STATUS update (§12.1):** the probe below is the gore-remainder-phase
> snapshot (f10 `67c51ec7` / f5 `b33ba0a9`). At the FINAL pin (f10 `f7a9dd78` /
> f5 `d908a4fd`): **29 of the 32 f5-owned kinds are PUSHED** (the 11 `gore-f5` +
> 18 of the 21 `gore-f5*`); only 250/315/471 remain in f5's window `:88817-
> 89835`. The `gore-f5*` "claimed-not-pushed" ARG hand-offs below are now PUSHED
> re-draws (Plantera b43/44, Brain b44, tombstones b63) — strip at wiring per
> §12.4. f10's continuation also OVERRAN `:85560` (§12.2) — its classic-mob
> duplicates are deleted, not stripped.

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

## 12. FINAL STATUS (the death-family final-status phase, 2026-11-21)

This section is the **final refresh** of the playbook: the lane-tip re-probe
(§12.1), the **f10↔f5 boundary overrun** (§12.2 — a NEW gore-lane-vs-gore-lane
overlap the earlier phases did not have), the **g59∩f5 strip extension**
(§12.3), the **110-row mechanical strip checklist** (§12.4), and the three
shared-surface contracts: the `npc_death_dust.rs` union (§12.5), the
`last_hit_dir` 3-way ONE-declaration (§12.6), and the `update_trails` ONE-fn
(§12.7). Code-only: no cargo/tests/rustfmt/graphify invoked.

### 12.1 Lane-tip movement log + f5 liveness + remaining-window census

> **FINAL-FINAL supersession (§12.8):** f5 COMPLETED the chain (batch 103,
> `fa3c526d5`; docs tip `d0619d8e9`) — the remaining window below is now
> **0**, every arm in the table pushed. The census stands as the historical
> hand-off artifact; the 18-conditional strip rows resolved to 17 (kind 23
> is f5-NO-gore, ports nothing — §12.8.2). f10 advanced b92→b134+ (§12.8.3).

**Probe method:** `git fetch origin --prune` then `git rev-parse` / `git log -1`
per lane branch, repeated across the session. The tips MOVED during the probe
(the death family is live). Final pin (this commit's read):

| lane | playbook-phase tip | FINAL tip | movement |
| --- | --- | --- | --- |
| g59 | `eea6975b` (b7) | `dd336ef8` | +b8/9/10 (24 d35 row-gated) + 2 DOC-SWEEP; **88 kinds / 90 cs_ids / 42 witnesses** |
| g60 | `e55e2272` | `ed7cef6e` | +playbook +gore-tail (this lane); 27 witnesses |
| d55 | `ec78d152` | `ec78d152` | unchanged |
| e5 | `a62a32ce` | `a62a32ce` | unchanged |
| f10 | `67c51ec7` (b60, `:85334`) | `f7a9dd78` (b92, `:86784`) | **+b61-92 — OVERRAN `:85560`** (§12.2); 134 witnesses |
| f5 | `b33ba0a9` (b33, `:86986`) | `d908a4fd` (b85, `:88816`) | **+b34-85 — ALIVE, advancing**; 124 witnesses |
| g62 | (not in playbook) | `53fdbf07` | hit-half COMPLETE + g60 ring adoption (§12.7); 61 witnesses |
| c12 | (in flight) | `5778e596` | phase 2 done (388/418 count-plans) |

**f5 is ALIVE, not stalled.** Its tip moved across this session's probes
`b33ba0a9` (b33) → `03918248` (b62) → `8ec8d16b` (b65) → `3469707f` (b68) →
`a043f25e` (b73) → `d999089c` (b83) → `d908a4fd` (b85) — **52 batches, `:86986`→
`:88816`, sites 290→733**, while this playbook was being written. The wave-D
claim `97c70c97` (the whole `:85560-89835`, 853 sites) is being executed in
source order at ~10 sites/batch. **No f5-followup takeover is needed.** The
remaining-window census below is published anyway as the f5-followup hand-off /
verification artifact (should f5 stall before `:89835`).

**f5 remaining window: `NPC.cs:88817-89835` (~1019 lines, ~120 sites, 733→853).**
The arms still to push (C# type-gates walked in source order; `g59`/`g60` marks
the dust-owner whose loop f5 must NOT re-draw):

| C# arm | kinds | dust-owner overlap | gore note |
| --- | --- | --- | --- |
| `:88818` | 169 | — | hit dust-92 + death 100× |
| `:88838` | 170/180 | — | |
| `:88862` | 171 | — | |
| `:88891` | 290 | — | |
| `:88909` | 87-92 (Wyvern family) | **g59: 87** | the 87 head's gore-ARG ladder is g59-consumed |
| `:88934` | 78/79/80/630 | **g59: 78/79/80** | |
| `:88959` | 82/158/159/250/253 | **g59: 82 · g60: 250** | the 250 nimbus 99×3 (`gore-f5*`, §11) |
| `:88984` | 316/330 | **g60: 330** (gore-none) | dust-only |
| `:89003` | 662 | — | |
| `:89026` | 315 | **g60: 315** | the 315 headless 99×3+446 (`gore-f5*`) |
| `:89053` | 254-261/634/635 | **g60: 261** (gore-none) | 261's 375/376/377 are `type!=261`-gated |
| `:89081` | 140 | **g59: 140** | |
| `:89099` | 39/40/41 | — | |
| `:89115` | 34 | — | |
| `:89144` | 289 | — | |
| `:89167` | 35/36/68 | **g59: 35/36** | |
| `:89210` | 139 | — | |
| `:89245` | 467 | — | |
| `:89266` | 134-136 | — | |
| `:89324` | 347 | — | |
| `:89355` | 346 | — | |
| `:89396` | 127 | — | |
| `:89433` | 128-131 | — | |
| `:89470` | 23 | **g59: 23** | meteor-head |
| `:89503` | 24 | **g59: 24** | fire-imp |
| `:89526` | 25 | — | |
| `:89538` | 33 | — | |
| `:89550` | 26/27/28/29/73/111 | **g59: 26/27/28/29/73/111** | the goblin quartet + scout/archer |
| `:89607` | 471 | **g60: 471** | the 471 summoner 675/676/677/678×2 (`gore-f5*`) |
| `:89627` | 472 | — | |
| `:89654` | 480 | — | |
| `:89674` | 185 | — | |
| `:89690` | 30/665 | — | |
| `:89702` | 42/176/205/231-235 | **g59: 42** | the hornet shared-arm (231-235 absent both lanes) |
| `:89742` | 43/56 | — | |
| `:89759` | 175 | — | |
| `:89776` | 48 | — | |
| `:89793` | 62/66 | **g59: 66** | |
| `:89819-89831` | final else 156 | — | dust-156 `:89821` + gores 184/185/185 `:89829-89831` (the chain's last site) |

**Of the §11 `gore-f5*` 21, 18 are now PUSHED** (f5 b34-85); only **250 / 315 /
471** remain in this window. The 11 `gore-f5` were already pushed. So **29 of the
32 f5-owned g60 kinds are pushed**; the 3 remaining are all in the table above.
The claimed-not-pushed ARG hand-offs (§11) are now **pushed re-draws** — see
§12.4's f5 block (Plantera pairs b43/44, Brain velocity b44, tombstones b63,
Drippler wings b25).

**The f10 `:85334-85560` gap — CLAIMED, no census needed.** The playbook-phase
gap between f10's then-tip (`67c51ec7`, `:85334`) and f5's claim start
(`:85560`) is now inside f10's pushed window: batch 61 (`b0b5b8cb`) landed the
121 Slimer (`:85451`, site 561 — the window's ONLY `NewGore` site, verified by
walking the C# `:85334-85559`). The other arms in the gap (147/184 ice-slime
pairs, 143-145 snowmen, 141, 112/666, 183 crimslime, 81/121 dust, 120/137/138,
122 gastropod, 75 pixie, 63/64/103 jellyfish) are **DUST-ONLY** — f10's
gore-table scope skips them (no dispatch gates at `f7a9dd78`); g59's feed owns
their dust (kinds ≤230). No unclaimed gore, no double-draw: the gap is closed.

### 12.2 The f10↔f5 boundary OVERRUN (the new gore-lane-vs-gore-lane overlap)

> **FINAL-FINAL supersession (§12.8.3):** the overrun grew to batches 62-134
> (sites 562-1149, `:85684-88220`, 88 fns + the cite-less `five_gore_ladder`,
> 85 gates, 97 witnesses at `91b082e3c`) and f10 is STILL advancing — the
> EXACT per-fn drop-hunks + the three contiguous band anchors + the re-census
> scripts are §12.8.3. The ~38-fn table below is the b92 snapshot (historical).

**Finding.** f10's claim boundary is `NPC.cs:82206-85559` (its module header:
"f10 owns EVERYTHING below `:85560`"; f5's header reciprocates: "wave D … the
classic-mob chain `:85560-89835` ONLY. Disjoint from f10's writer."). f5 claimed
`:85560-89835` FIRST (`97c70c97`, 03:43). **f10's continuation then overran:**
batch 61 (`b0b5b8cb`, 05:21) is the 121 Slimer `:85451` (site 561 — still legit,
`<85560`), but **batch 62 (`35bcd7e5`, 05:28) is the 50 King Slime `:85684`
(site 562) — INSIDE f5's claimed window.** Batches 62-92 (sites 562-816,
`:85684-86784`) re-port f5's classic-mob arms. f10's header at `f7a9dd78` shows
**no awareness of f5's claim** (no `:85560`/f5/wave-D mention).

**The double-port surface (f10 fns that duplicate f5 fns, both live-dispatched):**

| f10 fn (`npc_hit_effect_c.rs`) | f5 twin (`npc_hit_effect_d.rs`) | kinds | C# |
| --- | --- | --- | --- |
| `king_slime_death` | `king_slime_death` | 50 | `:85684` |
| `giant_tortoise_death` | `tortoise_death` | 153 | `:85718` |
| `derpling_death` | `derpling_death` | 177 | `:85737` |
| `crawdad_death` | `crawdad_death` | 494/495 | `:85757` |
| `giant_shelly_death` | `shelly_death` | 496/497 | `:85778` |
| `salamander_death` | `salamander_death` | 498-506 | `:85797` |
| `bat_family_death` | `bat_family_death` | 49/51/93/150/152/226 | `:85813` |
| `owl_death` | `owl_death` | 611/689 | `:85856` |
| `gnome_death` | `gnome_death` | 624 | `:85879` |
| `dandelion_death` | `dandelion_death` | 628 | `:85896` |
| `gem_bunny_death` | `gem_bunny_death` | 646-652 | `:85918` |
| `gem_squirrel_death` | `gem_squirrel_death` | 639-645 | `:85940` |
| `critter_family_death` | `critter_arm_death` | the 29-type critter arm | `:85958-86091` |
| `penguin_family_death` | `penguin_frog_death` | 148/149/168/470 | `:86108` |
| `frog_death` | `frog_361_death` | 361 | `:86142` |
| `pupfish_death` | `pupfish_death` | 607 | `:86187` |
| `pufferfish_death` | `pufferfish_death` | 688 | `:86206` |
| `scorpion_death` | `scorpion_death` | 366/367 | `:86233` |
| `duck_family_death` | `duck_death` | 362-365 | `:86260` |
| `seagull_death` | `seagull_death` | 602/603 | `:86287` |
| `grebe_death` | `grebe_death` | 608/609 | `:86307` |
| `nutcracker_death` | `nutcracker_death` | **348/349 (g60)** | `:86322` |
| `krampus_death` | `krampus_death` | **351 (g60)** | `:86342` |
| `elf_archer_death` | `elf_archer_death` | **350 (g60)** | `:86362` |
| `corrupt_critter_death` | `corrupt_critter_death` | 47/57/58/464/465 | `:86387` |
| `crimson_monster_death` | `crimson_mob_death` | 173/174/181/182/268 | `:86425` |
| `demon_eye_death` | `demon_eye_variant_death` | **190-194/317/318/2 (g59)** | `:86465-86501` |
| `arapaima_death` | `arapaima_death` | 157 | `:86514` |
| `wandering_eye_hit` | `wandering_eye_death` | 133 | `:86532` |
| `antlion_death` | `antlion_death` | 69/580/581/508/509/582 | `:86545-86605` |
| `vulture_death` | `vulture_death` | 61 | `:86617` |
| `raven_death` | `raven_death` | 301 | `:86639` |
| `parrot_death` | `parrot_death` | 252 | `:86652` |
| `blood_zombie_death`/`_ladder` | `blood_zombie_death` | **489 (g60)** | `:86669` |
| `zombie_merman_death` | `blood_zombie_death` (the shared 489/586 arm) | 586 | `:86683` |
| `demon_tax_collector_death` | `tax_collector_death` | 534 | `:86694` |
| `drippler_death` | `drippler_death` | **490 (g60)** | `:86724` |
| `shark_family_death` | `shark_death` | **65 (g59)**/692/542-545 | `:86746-86784` |

**Resolution (keep-one at the GORE-lane level).** f5 claimed `:85560-89835`
first and is executing it in source order; f10's boundary is `:85559`. **The
merger keeps f5's `npc_hit_effect_d.rs` for the whole classic-mob chain and
DELETES f10's overrun surface — every fn in the table above (sites 562-816,
`:85684-86784`) plus their `hit_effect_c` dispatch gates and witnesses.** f10
keeps sites 1-561 (`:82206-85559`: the DD2 band, solar/nebula/vortex/stardust,
martian, cultist, sharkron/duke, the eclipse/mothron/dutchman/desert families,
the golem/granite/ice arms, the critter gated-dust, umbrella 225, ice-tortoise
154, Slimer 121). This is a **wholesale fn deletion, NOT a dust-strip** — the
entire arm (dust AND gore) is f5's. It also auto-resolves the g60 kinds
348/349/350/351/489/490 and the g59 kind 65 that f10's overrun double-drew.

> **Why not keep f10's and delete f5's?** f5's claim is the authoritative
> boundary (both headers agree f10 stops at `:85559`), f5 claimed first, and f5
> is the lane actively advancing through the window. f10's overrun is the
> accident. The merger reconciles to the CLAIM, not to whichever pushed more.

### 12.3 The g59∩f5 strip extension (the NEW dust-vs-gore overlap) + g59∩f10

**Finding.** The playbook's §6 proved g59↔g60 CLEAN (disjoint type bands). It
did NOT audit g59 against the GORE lanes' dust re-draws. f5's classic-mob chain
re-ports the death-half dust loops for **57 g59-owned kinds (all ≤230)** — the
same `death_dust_loop` re-draw pattern the g60 strips use. Verified by walking
f5's `hit_effect_d` dispatch (`d908a4fd`, lines 4617-5163) against g59's 90
cs_ids: e.g. f5 `demon_eye_variant_death` cites `:86459-86462` (the 50× dust-5
burst) — **g59's exact kind-2 arm**.

**g59∩f5 = 57 pushed kinds** (strip f5's dust loop, keep the gore bodies):
`2,5,6,7,10,13,17,18,19,20,21,22,32,38,44,45,49,51,54,58,65,77,83,84,85,86,93,
94,95,96,97,98,99,100,101,106,107,108,110,117,123,124,190,191,192,193,194,207,
208,212,213,214,215,216,222,227,228`.
**+18 in the remaining window** (§12.1, conditional on f5 pushing them):
`23,24,26,27,28,29,35,36,42,66,73,78,79,80,82,87,111,140`. **Eventual g59∩f5 =
75.** (The other 15 g59 kinds — 75/103/120/122/137/138/143/144/145/147/183/184/
210/211/225 — live in f10's `:82206-85559` window as DUST-ONLY arms f10 skipped,
so no f5 re-draw; g59's feed pays their dust. **225 is the exception** — f10
ports its gore + re-draws its dust, the g59∩f10 row below. The other 14 have no
gore-lane dust re-draw at all.)

**g59∩f10 = {225}** (UmbrellaSlime): f10 `umbrella_slime_death` (`:85038-85042`,
site 550) re-draws the 50× type-4 tinted burst g59 owns (`npc_death_dust.rs`
kind-225 arm). **Strip f10's dust loop; keep the gore 314.** This is a legit
f10-window strip (NOT part of the §12.2 overrun).

**Strip-row total (FINAL):** the g60 baseline **110** (§4/§11) + **57** pushed
g59∩f5 + **1** g59∩f10 (225) = **168 actionable now**; + **18** conditional
g59∩f5 (remaining window) = **186 eventual**. The §12.2 f10-overrun is a
wholesale deletion (~38 fns), counted separately (not a dust-strip row).
**FINAL-FINAL (§12.8.4): 204 actionable, 0 conditional** — f5 complete
(17 of the 18 materialized, 23 is NO-gore) + g59 b11-13 added 19 more
intersection kinds → g59∩f5 = **93**; the f10-overrun delete is 88+1 fns
at `91b082e3c`, re-census at merge.

### 12.4 The mechanical strip checklist (the 110-row baseline + the FINAL extensions, per-row fn/hunk)

The merger's keep-one checklist. Each row: **lane · fn · STRIP this hunk · KEEP
this · ARG hand-off**. `dust loop` = the `death_dust_loop`/`new_dust` burst the
dust lane already pays. The g60 feed (`death_dust_draws`) is the single dust +
ARG-draw owner for every g60 kind; the g59 feed (`death_dust`) for every g59
kind. **Strip the gore lane's re-draw; never re-roll an ARG draw.**

**d55 (`npc_hit_effect_a.rs::hit_effect`) — 1 row:**
| kind | fn | STRIP | KEEP |
| --- | --- | --- | --- |
| 668 | `deerclops_death` | the 120-dust burst + per-dust `NextVector2Circular(4,4)` (`:81894-81900`) | the 7 gores 1274×2/1272/1273×2/1270/1271 (`:81902-81908`), kicks only |

(636 `empress_hit_dust_draws` is a count-plan → **pool feed** at wiring, NOT a
strip; e5 owns 636 whole. 686/594 gore-only, no strip. Party-hat/daybreak no
strip.)

**e5 (`npc_hit_effect_b.rs::hit_effect_b`) — 10 kinds in 4 fn-rows:**
| kind(s) | fn | STRIP | KEEP |
| --- | --- | --- | --- |
| 552/553/554 | `dd2_goblin_death` | `dd2_death_dust` 20×5 preamble (`:228-245`) | the per-tier 983/1067/1071 ladder |
| 561/562/563 | `dd2_javeliner_death` | `dd2_death_dust` | 987/1055/1061 ladder |
| 555/556/557 | `dd2_kobold_death` | `dd2_death_dust` | 1031/1034/1037 ladder |
| 551 | `dd2_crystal_death` | `dd2_death_dust` | **BOTH** 551 branches (`:82191-82203` + `:82206-82215` mirrored twin), the 10-gore fan |

(657/658-660/667 e5 owns whole, no strip. The 551 `CheckProgress` predicate
stays e5's.)

**f10 (`npc_hit_effect_c.rs::hit_effect_c`) — 68 kinds in 42 fn-rows (the 44
§5.3 kinds in 30 rows + the 24 §11 kinds in 12 rows):**
The 44 §5.3 rows keep their exact fn/hunk table (§5.3 above). The **24 §11
gore-f10 rows** (all constant-argument gore — strip the dust loop, keep bodies +
kicks, zero ARG hand-off):
| kind(s) | fn | STRIP (dust loop) | KEEP (gores) |
| --- | --- | --- | --- |
| 305-314 | `scarecrow_death` | 60×194 scale1.1 | 441-445 (+0/+20/+20/+34/+34 Y) |
| 325 | `mourning_wood_death` | 60×191 scale1.1 | 469-475 loop ×0 |
| 327 | `pumpking_death` | 60×189 scale1.1 | 476-484 loop ×0 |
| 344 | `everscream_death` | 60×196 | 508-512 + 509-512 loops ×0 |
| 385 | `gray_grunt_death` | 30×227 | 597/598/599 |
| 386 | `martian_engineer_death` | 30×227 | 600/601/602 |
| 387 | `martian_turret_death` | 30×226 scale0.5 | 603/603/604 |
| 389 | `giga_zapper_death` | 30×227 | 904/905/906 |
| 520 | `martian_walker_death` | 30×226 scale0.5 | 851/852 + 848/849/850 (+80Y) |
| 391 | `scutlix_death` | 30×227 | 607/608 (pos+UnitX×w/2f) / 609 (+20Y) |
| 399 | `martian_probe_death` | 10×226 scale0.5 | 898/899 (ai[0]==3 rotation, data) |
| 392/393/394/395 | `martian_saucer_death` | 30×228 scale0.5 **incl the 228-tweak re-roll** | 393:616 / 394:617 / 395:610-615 (392 spawns NONE) |

**f10 ARG hand-offs (the re-drawn gates — read the g60-consumed value, do NOT
re-roll):** 566/567 `skeleton_death` six `Next(2)` coin gates (`:82363-82386`);
576/577 `ogre_flipped_death`/`ogre_death` `Next(5)` head pick (`:82406`/`:82417`);
439 `cultist_boss_death`→`cultist_bone_spatter` `Next(1,4)`+n×`Next(11,14)`
(`:84150-84155`). **551 `betsy_flipped_death`: DELETE the whole fn** (e5's twin
keeps both branches). **418 `solar_corite_death`: strip dust** (c12 owns), keep
the 5 gores 841/842×2/843×2. **225 `umbrella_slime_death`: strip dust** (g59
owns, §12.3), keep gore 314.

**f5 (`npc_hit_effect_d.rs::hit_effect_d`) — 32 g60 kinds in 23 fn-rows + 57
g59 kinds (§12.3):**
The 32 g60-owned kinds (strip f5's dust loop; the gore bodies are f5's to KEEP).
Fn map at `d908a4fd`:
| kind(s) | f5 fn | ARG hand-off (g60-consumed, strip the re-draw) |
| --- | --- | --- |
| 262/263/264 | `plantera_death` | **9/2/2 × (Next(w)+Next(h)) position pairs** (b43/44 PUSHED — re-draws today) |
| 266 | `brain_death` | **6×2 Next(-30,31) velocity picks** (b44 PUSHED) |
| 267 | `creeper_death` | — |
| 329 | `hellhound_death` | — |
| 338/339/340 | `zombie_elf_death` | — |
| 341 | `present_mimic_death` | — (b85 PUSHED) |
| 342 | `gingerbread_death` | — |
| 343 | `yeti_death` | — |
| 348/349 | `nutcracker_death` | — |
| 350 | `elf_archer_death` | — |
| 351 | `krampus_death` | — |
| 353/354 | `stylist_death` | — (picks are data; b66 PUSHED) |
| 368 | `travelling_merchant_death` | — (b63 PUSHED) |
| 369/376/663 | `town_pet_smoke_death` | **4× Next(61,64) tombstone picks** (b63 PUSHED; ONE shared arm — strip once, not thrice) |
| 441 | `tax_collector_town_death` | — (b53 PUSHED) |
| 489 | `blood_zombie_death` | — |
| 490 | `drippler_death` | **2× Next(3) wing picks** (b25 PUSHED — re-draws today) |
| 550 | `bartender_death` | — (b61 PUSHED) |
| 588/589 | `golfer_death` | — (b61 PUSHED) |
| 633 | `zoologist_death` | — (picks are data; b62 PUSHED) |
| 250 | `ice_tortoise_smoke_death` @4801 (`:88959` shared arm — b88 PUSHED) | — |
| 315 | `headless_horseman_death` @4836 (`:89026` — b89 PUSHED; the pumpkin 446 ×0.1 post NOT 0.3) | — |
| 471 | `goblin_summoner_death` @5359 (`:89607` — b99 PUSHED) | — |

The **57 g59∩f5 rows** (§12.3): strip the SAME `death_dust_loop` hunk in each
cited f5 fn (the g59 feed owns the dust); keep the gore bodies. The 18
remaining-window kinds strip when f5 pushes them. **f5 keeps its gore bodies +
kicks for ALL its kinds** — only the dust loop (and the listed ARG re-draws)
strip. The §12.2 f10-overrun deletion removes f10's duplicate, so f5 is the sole
gore writer for the classic-mob chain.
**FINAL-FINAL (§12.8.4): the g59∩f5 set is 93 kinds in 47 fn-rows** (f5
complete + g59 b11-13; the per-fn gate list is `g60_g59_f5_inter.json`,
re-runnable via `g60_g59_f5_inter.py`). The 23 Meteor Head row NEVER
materializes (f5 NO-gore). The f5 fn-map above is at `d908a4fd`; the
FINAL-FINAL f5-tip lines: `stylist_death` @3907, `present_mimic_death`
@4514, `travelling_merchant_death` @3630, `town_pet_smoke_death` @3668,
`tax_collector_town_death` @3154, `bartender_death` @3514, `golfer_death`
@3549, `zoologist_death` @3590 (the rest unchanged in shape).

**c12 (`npc_ai_style74.rs`) — 2 rows:** 388/418 `drone_hit_dust_plan`/
`corite_hit_dust_plan` count-plans → **pool feeds** at the death retain (NOT a
second census). 418's f10 dust strips (above); 388 has no gore-lane dispatcher.

**gore-none (10):** 261/265/288/326/330/352/384/402/410/533 — nothing to strip
anywhere; the g60 feed is their whole death-effects port.

### 12.5 `npc_death_dust.rs` — the g59×g60 textual-collision union-shape (FINAL)

Both lanes create `crates/tr-app/src/npc_death_dust.rs` with different content.
**Field/fn inventory diff** (g59 `dd336ef8`, 1837 lines · g60 `ed7cef6e`, 1962
lines):

| surface | g59 | g60 | collision? |
| --- | --- | --- | --- |
| imports | `crate::npc::Npc` | `crate::npc::{Npc, NpcKind}` | **textual** — union to `{Npc, NpcKind}`; glam/tr_core identical |
| entry fn | `pub fn death_dust(npc, npc_type, dust, ctx, rng, world) -> bool` (`:227`) | `pub(crate) fn death_dust_feed(app, npc)` (`:242`) | none (distinct names) |
| census fn | — | `fn lane_covers(ty) -> bool` (`:281`) | none |
| draws fn | — | `pub(crate) fn death_dust_draws(fx)` (`:300`) | none |
| struct | — | `pub(crate) struct DeathFx<'a>` (`:115`) + impl (`d`/`d_full`/`burst`/`pair_loop`/`stardust_loop`/`smoke54`/`burst226`/`circular`) | none |
| test `ctx()` | identical body (1920×1080, 6000) | identical body | **textual** — dedupe to ONE |
| test `air()` | `World::new("t", 200, 200)` | `World::new("t", 200, 120)` | **SEMANTIC** — different world heights; keep BOTH under distinct names |
| test helpers | `corpse(kind)` | `replay_spawn(r, w, h)` | none |
| witnesses | 42 `w4_g59_*` | 23 `w4_g60_*` | none (distinct prefixes) |

**ONE-file union plan (concatenation, not pick-a-side — the AGENTS.md
semantic-conflict rule):**
1. One module header (union the two lane descriptions).
2. Imports: `use crate::npc::{Npc, NpcKind};` + the shared `glam::Vec2` /
   `tr_core::particles::{Color, DustPool, SpawnCtx}` / `tr_core::{UnifiedRandom,
   World}`.
3. g59's `pub fn death_dust` (kinds 1-230) **followed by** g60's `DeathFx` +
   impl + `death_dust_feed` + `lane_covers` + `death_dust_draws` (kinds 231-697).
   No name clash; the two matches are disjoint (§6).
4. ONE `mod tests`: keep ONE `ctx()` (identical). **Resolve `air()`:** rename
   g59's to `air_tall()` (200×200) and keep g60's `air()` (200×120); repoint each
   lane's witnesses to its own helper (g59's 42 witnesses call `air()` once
   each → `air_tall()`; g60's 23 witnesses keep `air()`). This is the ONE
   semantic test-helper collision — do NOT silently pick one height (it changes
   the cull-margin band the witnesses assert).
5. Keep `corpse()` (g59) + `replay_spawn()` (g60); all 65 witnesses coexist.
6. `main.rs`: ONE `mod npc_death_dust;` (g59 `:66` / g60 `:162` → dedupe).
7. `world_sim/npcs.rs`: keep BOTH death-retain calls (g59 `death_dust` `:5113`,
   g60 `death_dust_feed` `:5044`) — disjoint by kind, either order (§2).
8. `anchors.txt`: keep BOTH rows (`pub fn death_dust` g59 · `pub(crate) fn
   death_dust_draws` g60 — distinct markers).

### 12.6 `last_hit_dir` — the c12/g60/g59 ONE-declaration contract (FINAL)

Three lanes add the SAME `Npc.last_hit_dir` field. **Inventory diff:**

| surface | c12 `5778e596` | g60 `ed7cef6e` | g59 `dd336ef8` | shape |
| --- | --- | --- | --- | --- |
| field decl | `:1239` | `:1240` | `:1240` | `pub last_hit_dir: i32,` after `pub just_hit: bool,` — **identical** |
| ctor init | `:2944` | `:2954` | `:2942` | `last_hit_dir: 0,` — **identical** |
| `hurt` record | `:4401` | `:4415` | `:4412` | `self.last_hit_dir = dir;` — **identical** |
| `hurt_from_projectile` record | `:4456` | `:4470` | `:4467` | `self.last_hit_dir = dir;` — **identical** |
| `strike_pooled` record | `:4569` | `:4583` | `:4580` | `n.last_hit_dir = dir;` — **identical** |
| fuse reset | `ai_074_hover_charge` `:5759` | **none** | `ai_070_bubble` `:5654` | `self.last_hit_dir = 0;` — **DIFFERENT fns** |

(d55/e5 add NO `last_hit_dir` — 0 hits; g62 also 0 hits — its
`HitDustSnapshot.hit_dir` captures the strike side at strike time, a different
seam.)

**ONE-declaration contract:**
- **Dedupe to ONE** field decl + ONE ctor init + ONE set of the 3 strike records
  (the c12/g60/g59 additions are byte-identical statements; the merge keeps a
  single copy of each). The doc comment unions to name all three consumers (the
  c12/g59/g60 death feeds read the recorded side) + the C# default
  `hitDirection = 0` (`NPC.cs:81654`).
- **KEEP BOTH fuse resets** — they are DIFFERENT sites, not duplicates: g59's
  `ai_070_bubble` (the DetonatingBubble 594 fuse calls argument-less `HitEffect()`,
  `NPC.cs:34943-34946`) and c12's `ai_074_hover_charge` (the MartianDrone/Corite
  fuse, `NPC.cs:35497`). g60 adds no reset (its kinds have no argument-less fuse).
  Both resets stay; neither is deduped.

### 12.7 `update_trails` — the g62b×g60 oldPos-shared ONE-fn note (FINAL)

**g62b (`53fdbf07`) adopted g60's ring VERBATIM** (`2370316ce`: "adopt g60's
`Npc.old_pos[10]`/`old_rot[10]` + `update_trails` VERBATIM (0e0fbd6c7 field
shapes)"). Verified byte-identical: the `update_trails` body (g60 `npc.rs:4989-
5001` ≡ g62 `:5010-5022`), the two fields, the field-init lines, the
`update_rng_dust` tail call, and the two `npc.rs` witnesses.

**ONE-fn contract:**
- **Dedupe to ONE** copy of: `old_pos: [Vec2; 10]` + `old_rot: [f32; 10]` fields,
  their ctor init, `pub fn update_trails` (the mode-7 shift, `NPC.cs:90863-90872`,
  402-only gate), the solo `update_rng_dust` tail call (`npc.rs:4978`), and the
  two `npc.rs` witnesses. g60 ≡ g62 — trivial dedupe, identical shape.
- **KEEP g62's ONLY additions** (g60 did not touch them): the **pooled aiStyle-75
  tail call** (`world_sim/npcs.rs:4540`, `npcs[i].update_trails()` — the
  `NPC.cs:90512` order on the pooled path), the `HitDustSnapshot.old_pos` field
  (g62's own struct; g60's twin lives in `DeathFx` — different type, no
  collision), and the census doc notes.
- **The 402 sites are DISJOINT** (the giant else-if's two halves): **g60 owns the
  DEATH arm** (`:83599-83632`, the trail loop at `:83611`), **g62 owns the HIT arm**
  (`:83565-83598`, the trail loop at `:83579`). Both replay in their own lanes on
  the same ring. g62 also unblocked the **519 HIT trail** (`:83008-83023`, stride-3
  groups) — 519→7 is the SAME `TrailingMode` row as 402→7 (`NPCID.cs:10430`).
- **Recorded deviation — the TrailCacheLength under-carry** (`NPCID.cs:10450` +
  `SetDefaults` resize `NPC.cs:8453-8468`): the C# resizes per type — **402→36,
  519→20** (522/620→20, 677→60, 685→10; default 10). BOTH lanes carry the g60
  10-slot shape, so the 402 arms walk 10 of 36 slots and the 519 arm walks
  0/3/6/9 of 0/3/…/18 — the per-slot `Next(4)` gates / stride-3 groups are
  proportionally short. **The fix is ONE widening at merge** (a per-kind
  `trail_len` or a 60-slot ring + length — 685's row is the largest); until then
  the 10-slot shape is the recorded stand-in for BOTH lanes. The gate stays
  402-only until each kind's wiring lane lands (519's KIND is unported).
  **FINAL-FINAL: the widening is EXECUTED on BOTH branches — see §12.8's
  receipt; this deviation note is now historical (the merge dedupes the
  byte-identical widened hunks, it does not widen anything).**

### 12.8 THE BIBLE PHASE (FINAL-FINAL, 2026-11-21 — the widening executed, f5 complete, the drop-hunks exact)

Three movements since §12's FINAL STATUS (`402b32f2`): **g62 published the
EXACT 6-hunk widening contract** (`32af98b44`) and g60 EXECUTED its half;
**f5 COMPLETED the whole classic-mob chain** (853/853 sites, remaining window
0); **f10 kept overrunning** (b92→b134 during this phase alone). This section
is the merger's FINAL input: the widening receipt, the f5-completion census,
the EXACT f10-overrun drop-hunks (per-fn line-ranges at a pinned tip + the
re-census scripts), and the refreshed strip totals.

#### 12.8.1 The widening-executed receipt (g62's 6-hunk contract CONSUMED)

g62's `32af98b44` ANALYSIS ("TRAILCACHE WIDENING — LANDED", the EXACT
widening-hunk contract for g60) is CONSUMED. g60 commit **`19431e385`**
applied all six `Npc`-side hunks **byte-identically** (verified line-for-line
against g62's tip by `g60_verify_widen.ps1`: fields 34/34 lines equal, init
8/8, update_trails fn+doc 40/40, Transform 7/7, table 19/19, witnesses
105/105) plus g60's OWN `DeathFx` half:

| hunk | surface | g60 landing (post-commit line refs) |
| --- | --- | --- |
| 1 fields | `npc.rs` `pub old_pos: [Vec2; 60]` / `pub old_rot: [f32; 60]` / `pub trail_len: usize` + the census doc | `:1241-1274` (g62 `:1233-1266` shape, byte-identical) |
| 2 init | `Npc::new` `old_pos: [Vec2::ZERO; 60], old_rot: [0.0; 60], trail_len: kind.trail_cache_length(),` | `npc.rs:2977-2984` |
| 3 table | `NpcKind::trail_cache_length` (`npc/data.rs`, after `cs_id`): `StardustWormHead => 36, _ => 10` + the full-census doc | `data.rs:3102-3120` |
| 4 stride+gate | `update_trails` walks `(1..self.trail_len).rev()`; gate `matches!(kind, StardustWormHead \| DD2KoboldFlyerT2 \| DD2KoboldFlyerT3)` (945a6ea28's flyer widening adopted) | `npc.rs:5008-5048` |
| 5 Transform | `transform_to_kind` re-lengths + re-zeroes both rings after `self.kind = new_kind` | `npc.rs:5199-5205` |
| 6 witnesses | the two `w4_g60_*` trail witnesses REPLACED with g62's widened versions (36 logical slots, 34+2 shifts, `[Vec2::ZERO; 60]` inits, storage-past-trail_len zero asserts); g62's `w4_g62_the_trail_cache_census_and_resize` + `w4_g62_the_kobold_flyers_shift_the_mode_7_ring` ADOPTED too | `npc.rs:20563-20667` |
| g60e | `DeathFx.old_pos: [Vec2; 60]` + `pub trail_len: usize` captured at strike (`death_dust_feed` copies `npc.trail_len`) | `npc_death_dust.rs:124-137`, `:278-279` |
| g60-death-loop | the 402 DEATH arm (`:83611`, `num179 < this.oldPos.Length` NO min) bounds `for i in 0..fx.trail_len` — up to **36** per-slot Next(4) gates (was 10); the zero-break + the passing-slot 2× (Next(3) dust 17 + unconditional 229 ×3 noGravity) unchanged | `npc_death_dust.rs:809-846` |
| fresh-ring witness | `w4_g60_the_worm_head_trail_gates_until_the_zero_break` gate loop `0..10` → `0..36` exactly; the live-slot witness walks `ring[..36]` | `npc_death_dust.rs:1505-1650` |

All 22 non-402 `DeathFx` test sites carry `old_pos: [Vec2::ZERO; 60],
trail_len: 10` (the default row — their arms never read the ring). The
`Npc`-side hunks are ONE copy at merge (trivial dedupe — byte-identical);
g62's only additions to the shared surface stay g62's (the pooled-path tail
call, `HitDustSnapshot.old_pos[60]`+`trail_len`, the doc census notes). The
402 sites stay disjoint: g60 DEATH `:83611`, g62 HIT `:83579`. **The §12.7
"ONE widening at merge" item is CLOSED — both halves landed pre-merge.**

#### 12.8.2 f5 CHAIN COMPLETE — the remaining-window census closes at 0

f5's tip `d0619d8e9` (completion docs over code tip `fa3c526d5`, batch 103):
**all 853 gore sites of `NPC.cs:85560-89835` ported**, source-count
re-verified by f5 (853 `Gore.NewGore(` lines; 685 `gore.new_gore` call sites —
the difference is the shared `mech_smoke_burst`/`mech_quadrant_smoke`/ladder
helpers fanning out). The §12.1 remaining window `:88817-89835` is **EMPTY**
— every arm in its table pushed (b86-103: the Pigron trio, 290, the Wyvern
count-gore, the mummy smoke, the 250 nimbus, the 315 headless, the mushroom
zombies, 140, the Bone Serpent, the Skeletron RedHat split, the Probe/Deadly
Sphere/Destroyer/Elf Copter/Santa-NK1/Skeletron Prime mech band, the Fire
Imp, the goblin family, the 471 summoner, 472, Medusa, Snow Flinx, the
hornet/moth family, Man Eater/Snatcher, Angry Trapper, Harpy, Demon, the 156
tail). **No f5-followup takeover is needed — the lane finished its claim.**

**The NO-gore arms are recorded-not-ported** (f5's dispatcher comments +
completion note): 59/60/151 lava, 687/685, 265 Spore, 663 (`:88411`
unreachable), 277-280, 169, 316/330, 662, 34, 289, **23 Meteor Head**, 25
Burning Sphere, 33 Water Sphere, 30/665. Their dust censuses stay the dust
lanes' (g59/g60/g62) — **no f5 re-draw, no strip row** for them. This kills
ONE of §12.3's 18 conditional rows: **23** (f5 recorded it NO-gore, ports
nothing). The other 17 all pushed WITH dust re-draws (verified per-fn:
`fire_imp_death` 24, `goblin_death` 26/27/28/29/73/111, `skeletron_death`
35/36, `hornet_family_death` 42, `demon_death` 66, `mummy_death` 78/79/80,
`ice_tortoise_smoke_death` 82, `wyvern_death` 87, `possessed_armor_death`
140 — each carries its `death_dust_loop`/`new_dust` burst).

**The 3 remaining g60 `gore-f5*` kinds PUSHED:** 250
(`ice_tortoise_smoke_death` `:88959` shared arm, b88), 315
(`headless_horseman_death` `:89026`, b89 — the pumpkin 446's ×0.1 post NOT
0.3), 471 (`goblin_summoner_death` `:89607`, b99). **All 32 f5-owned g60
kinds are now pushed re-draws** — the §12.4 f5 block's three UNPUSHED rows
become strip rows (fn refs at f5's tip: `ice_tortoise_smoke_death` `:4801`,
`headless_horseman_death` `:4836`, `goblin_summoner_death` `:5359`).

**f5 stand-ins the merger inherits** (completion note): no `Npc.altTexture`
(default-0 keeps head gores), no `Npc.color` (`Color::default()` for dye
rows), `netMode` SP fold, sounds c13. The dust-6 seventh-draw witness fix
(`086827426`: `twin_dust6` replays the type-6 tweak's `Next(-10,6)`,
`Dust.cs:214-218`) is f5-internal — g60's `replay_spawn` already rides the
pool's own internals, unaffected.

**f5 compile-window flag (found by this probe, NOT fixed here — f5's lane):**
`HitEffectOutcomeD` (`npc_hit_effect_d.rs:103-141`) declares
`maggot_zombie_spawns`/`wof_screen_sweep`/`hungry_spawn` but **NOT
`king_slime_spawns`**, while `:221` writes `outcome.king_slime_spawns` and
two witnesses read it (`:6476`/`:6519`). The field declaration is missing —
the module will not compile as-is. The merger (or f5's next commit) must add
`pub king_slime_spawns: Option<(f32, f32)>,` to the struct (the doc comment
for it already sits at `:104-112`, merged into the maggot doc). Recorded
here so the compile window does not rediscover it blind.

#### 12.8.3 The f10-overrun DROP-HUNKS — EXACT per-fn line-ranges (pinned tip `91b082e3c`, batch 134)

§12.2's resolution stands (f5 claimed first, both headers agree f10 stops at
`:85559`; f5 is now COMPLETE — the claim is executed, not in flight). The
overrun GREW: batches 62-134, sites 562-1149, `NPC.cs:85684-88220`. **88
cite-bearing fns (87 arms + the `blood_zombie_ladder` helper) + the cite-less
`five_gore_ladder` = 89 fn bodies, 85 dispatch gates, 97 witnesses** — the wholesale
deletion surface at the pinned tip. f10 is STILL ADVANCING (b92→b134 during
this phase); the merger MUST re-run the census scripts at the merge-time tip
(`g60_f10_overrun.py` + `g60_f10_gates.py`, committed alongside this doc —
`python g60_f10_overrun.py <f10_c.rs> out.json` then
`python g60_f10_gates.py <f10_c.rs>`; both are pure-text, no deps). The
structure is stable: **the overrun is THREE CONTIGUOUS BANDS** (f10 appends
in source order), so the mechanical delete is band-shaped, not 81 scattered
edits:

| band | pinned-tip range (`91b082e3c`) | anchor START | anchor END | re-census rule |
| --- | --- | --- | --- | --- |
| **A: the fns** | `npc_hit_effect_c.rs:3648-6698` (3051 lines: `king_slime_death` doc → `stylist_death` closing brace) | the `/// The 50 KingSlime death` doc line (first cite ≥ `:85560`) | the last fn whose min-cite ≥ `:85560`, closing `}` | every top-level fn with min-cite ≥ 85560 (the script's 88-fn list); the two ladder helpers `blood_zombie_ladder` (`:4791-4825`) + `five_gore_ladder` (`:4946-4982`) sit INSIDE the band — all their callers are overrun fns |
| **B: the dispatch gates** | `:7122-7514` (393 lines: the KingSlime gate comment → the Stylist gate's closing brace) | the `// :85670-85703` comment after the Slimer gate (`:7116-7121` KEEPS — site 561 is legit) | the gate before `outcome` / `}` (`:7515-7516` KEEP) | every gate whose cite ≥ `:85560` OR whose callee is a band-A fn (the script catches the 2 comment-less gates: 586 `zombie_merman_death`, 263/264 `plantera_part_death`) |
| **C: the witnesses** | `:9862-11834` (1973 lines: the KingSlime witness doc → the Stylist witness closing brace) | the `/// NPC.cs:85684` doc after the Slimer witness (`:9860` `}` KEEPS) | the last overrun witness; the Betsy-dispatch witness (`:11836-11863`) KEEPS | every `w4_f10_*` test whose min-cite ≥ 85560 or whose body calls a band-A fn (97 at this tip; the 94 legit tests + the Betsy witness stay) |

**Plus two small out-of-band hunks:**
- the module-header scope-table rows `//! | 562 |` … `//! | 1140-1149 |`
  (`:136-233` at this tip — every row citing ≥ `:85560`; the `:85451` Slimer
  row `:135` KEEPS), and
- the `HitEffectOutcomeC` fields `king_slime_babies` (`:344-349`) +
  `maggot_zombie_spawns` (`:350-355`) — both seams are f5's now
  (`HitEffectOutcomeD.king_slime_spawns`/`.maggot_zombie_spawns`, the
  wiring wave reads f5's). `slimer_spawn` (`:339-343`) KEEPS (site 561).

**The per-fn drop list (88 cite-bearing fns = 87 arms + the
`blood_zombie_ladder` helper, PLUS the cite-less `five_gore_ladder` at row
39b = 89 fn bodies; pinned tip `91b082e3c`;
the script output `g60_f10_overrun.json` is authoritative — doc-start–end
ranges include each fn's doc comment):**

| # | fn (doc-start–end) | C# window | f5 twin (fn @ f5-tip line) |
| --- | --- | --- | --- |
| 1 | `king_slime_death` 3648-3679 | `:85680-85702` | `king_slime_death` @191 |
| 2 | `giant_tortoise_death` 3681-3706 | `:85714-85721` | `tortoise_death` @228 |
| 3 | `derpling_death` 3708-3733 | `:85733-85740` | `derpling_death` @262 |
| 4 | `crawdad_death` 3735-3762 | `:85752-85758` | `crawdad_death` @297 |
| 5 | `giant_shelly_death` 3764-3787 | `:85771-85775` | `shelly_death` @334 |
| 6 | `salamander_death` 3789-3818 | `:85788-85795` | `salamander_death` @367 |
| 7 | `bat_family_death` 3820-3858 | `:85807-85839` | `bat_family_death` @407 |
| 8 | `owl_death` 3860-3884 | `:85852-85858` | `owl_death` @458 |
| 9 | `gnome_death` 3886-3911 | `:85870-85877` | `gnome_death` @491 |
| 10 | `dandelion_death` 3913-3939 | `:85889-85896` | `dandelion_death` @525 |
| 11 | `gem_bunny_death` 3941-3970 | `:85900-85919` | `gem_bunny_death` @561 |
| 12 | `gem_squirrel_death` 3972-3998 | `:85923-85940` | `gem_squirrel_death` @598 |
| 13 | `critter_family_death` 4000-4174 | `:85942-86089` | `critter_arm_death` @648 |
| 14 | `penguin_family_death` 4176-4211 | `:86102-86125` | `penguin_frog_death` @803 |
| 15 | `frog_death` 4213-4235 | `:86138-86143` | `frog_361_death` @847 |
| 16 | `pupfish_death` 4237-4260 | `:86183-86188` | `pupfish_death` @1034 |
| 17 | `pufferfish_death` 4262-4293 | `:86200-86214` | `pufferfish_death` @880 |
| 18 | `seagull_death` 4295-4318 | `:86281-86288` | `seagull_death` @1002 |
| 19 | `scorpion_death` 4320-4349 | `:86227-86241` | `scorpion_death` @920 |
| 20 | `grebe_death` 4351-4375 | `:86301-86309` | `grebe_death` @1066 |
| 21 | `duck_family_death` 4377-4408 | `:86254-86268` | `duck_death` @961 |
| 22 | `nutcracker_death` 4410-4436 | `:86322-86330` | `nutcracker_death` @1099 |
| 23 | `krampus_death` 4438-4464 | `:86342-86350` | `krampus_death` @1134 |
| 24 | `elf_archer_death` 4466-4491 | `:86362-86369` | `elf_archer_death` @1169 |
| 25 | `corrupt_critter_death` 4493-4528 | `:86381-86406` | `corrupt_critter_death` @1206 |
| 26 | `crimson_monster_death` 4530-4567 | `:86419-86446` | `crimson_mob_death` @1256 |
| 27 | `demon_eye_death` 4569-4610 | `:86459-86501` | `demon_eye_variant_death` @1310 |
| 28 | `arapaima_death` 4612-4635 | `:86514-86519` | `arapaima_death` @1369 |
| 29 | `wandering_eye_hit` 4637-4672 | `:86523-86543` | `wandering_eye_death` @1403 (**see the latch note below**) |
| 30 | `antlion_death` 4674-4714 | `:86545-86605` | `antlion_death` @1474 |
| 31 | `vulture_death` 4716-4740 | `:86617-86623` | `vulture_death` @1434 |
| 32 | `raven_death` 4742-4765 | `:86635-86640` | `raven_death` @1535 |
| 33 | `parrot_death` 4767-4789 | `:86652-86657` | `parrot_death` @1566 |
| 34 | `blood_zombie_ladder` 4791-4825 (helper) | `:86669-86708` | f5's `blood_zombie_death` ladder @1603 |
| 35 | `blood_zombie_death` 4827-4839 | `:86675` | `blood_zombie_death` @1603 |
| 36 | `zombie_merman_death` 4841-4853 | `:86683` | the shared 489/586 arm @1603 |
| 37 | `demon_tax_collector_death` 4855-4867 | `:86704` | `tax_collector_death` @1649 |
| 38 | `drippler_death` 4869-4901 | `:86720-86728` | `drippler_death` @1691 |
| 39 | `shark_family_death` 4903-4944 | `:86730-86784` | `shark_death` @1740 |
| 39 | `shark_family_death` 4903-4944 | `:86730-86744` | `shark_death` @1740 |
| 39b | `five_gore_ladder` 4946-4982 (helper, ZERO `:NNNNN` cites — the script's cite-filter misses it; its 4 callers `nymph_death`/`icy_merman_death`/`gingerbread_man_death`/`moss_zombie_death` are ALL in this band, so it drops with them) | the +0/+20/+20/+34/+34 ladder | f5's per-arm ladders |
| 40 | `nymph_death` 4984-4997 | `:86797-86805` | f5's nymph arm |
| 41 | `icy_merman_death` 4999-5012 | `:86838-86846` | f5's icy-merman arm |
| 42 | `lihzahrd_death` 5014-5044 | `:86817-86826` | f5's lihzahrd arm |
| 43 | `gingerbread_man_death` 5046-5059 | `:86858-86866` | `gingerbread_death` @1916 |
| 44 | `zombie_elf_death` 5061-5095 | `:86878-86896` | `zombie_elf_death` @1958 |
| 45 | `yeti_death` 5097-5124 | `:86908-86915` | `yeti_death` @2005 |
| 46 | `moss_zombie_death` 5126-5139 | `:86927-86935` | `moss_zombie_death` @2045 |
| 47 | `zombie_block_death` 5141-5361 | `:86937-87138` | `zombie_arm_death` @2109 + the maggot seam |
| 48 | `cursed_weapon_death` 5363-5398 | `:87152-87162` | `cursed_weapon_death` @2329 |
| 49 | `plantera_death` 5400-5435 | `:87186-87207` | `plantera_death` @2382 |
| 50 | `plantera_part_death` 5437-5470 | `:87209-87217` | the 263/264 arm @2382 family |
| 51 | `brain_of_cthulhu_death` 5472-5501 | `:87248-87257` | `brain_death` @2445 |
| 52 | `creeper_death` 5503-5524 | `:87269-87273` | `creeper_death` @2479 |
| 53 | `servant_of_cthulhu_death` 5526-5549 | `:87351-87356` | `servant_death` @2610 |
| 54 | `wall_of_flesh_death` 5551-5586 | `:87368-87383` | the WoF arm + `wof_screen_sweep` seam |
| 55 | `the_hungry_death` 5588-5614 | `:87429-87434` | the Hungry arm + `hungry_spawn` seam |
| 56 | `leech_death` 5616-5640 | `:87446-87450` | `leech_death` @2746 |
| 57 | `beetle_death` 5642-5680 | `:87462-87478` | `beetle_death` @2777 |
| 58 | `queen_bee_death` 5682-5717 | `:87491-87500` | `queen_bee_death` @2824 |
| 59 | `eater_of_souls_death` 5719-5767 | `:87512-87534` | `eater_family_death` @2871 |
| 60 | `clinger_death` 5769-5797 | `:87547-87554` | `clinger_death` @2919 |
| 61 | `devourer_death` 5799-5824 | `:87566-87570` | `devourer_death` @2954 |
| 62 | `seeker_death` 5826-5849 | `:87583-87587` | the seeker arm @2987 family |
| 63 | `giant_worm_death` 5851-5875 | `:87600-87604` | `giant_worm_death` @3020 |
| 64 | `digger_death` 5877-5901 | `:87616-87620` | `digger_death` @3046 |
| 65 | `eater_of_worlds_death` 5903-5936 | `:87632-87649` | `eater_of_worlds_death` @3074 |
| 66 | `town_npc_death` 5938-5989 | `:87662-87693` | `guide_death` @3116 + `tax_collector_town_death` @3154 |
| 67 | `unicorn_death` 5991-6015 | `:87705-87715` | `unicorn_death` @3192 |
| 68 | `wolf_death` 6017-6041 | `:87727-87733` | f5's wolf arm |
| 69 | `hellhound_death` 6043-6067 | `:87745-87751` | `hellhound_death` @3291 |
| 70 | `black_recluse_death` 6069-6095 | `:87763-87771` | f5's black-recluse arm |
| 71 | `wall_creeper_death` 6097-6123 | `:87783-87791` | f5's wall-creeper arm |
| 72 | `blood_crawler_death` 6125-6149 | `:87803-87811` | f5's blood-crawler arm |
| 73 | `jungle_creeper_death` 6151-6176 | `:87823-87831` | f5's jungle-creeper arm |
| 74 | `blood_feeder_death` 6178-6201 | `:87843-87848` | f5's blood-feeder arm |
| 75 | `blood_jelly_death` 6203-6227 | `:87860-87868` | f5's blood-jelly arm |
| 76 | `bound_npc_death` 6229-6277 | `:87880-87900` | `bound_town_death` @3375 |
| 77 | `mechanic_death` 6279-6309 | `:87912-87920` | f5's mechanic arm |
| 78 | `guide_death` 6311-6341 | `:87932-87940` | `guide_death` @3116 |
| 79 | `bartender_death` 6343-6372 | `:87952-87960` | `bartender_death` @3514 |
| 80 | `golfer_death` 6374-6405 | `:87972-87983` | `golfer_death` @3549 |
| 81 | `bestiary_girl_death` 6407-6446 | `:87995-88021` | `zoologist_death` @3590 |
| 82 | `travelling_merchant_death` 6448-6480 | `:88033-88037` | `travelling_merchant_death` @3630 |
| 83 | `painter_death` 6482-6514 | `:88086-88097` | `painter_death` @3708 |
| 84 | `witch_doctor_death` 6516-6547 | `:88109-88117` | `witch_doctor_death` @3745 |
| 85 | `pirate_death` 6549-6597 | `:88129-88151` | `pirate_death` @3785 (the shimmer dust-split twin) |
| 86 | `santa_claus_death` 6599-6631 | `:88163-88167` | f5's santa arm |
| 87 | `steampunker_death` 6633-6663 | `:88186-88196` | `steampunker_death` @3868 family |
| 88 | `stylist_death` 6665-6698 | `:88208-88220` | `stylist_death` @3907 |

(88 cite-bearing fns + the cite-less `five_gore_ladder` = **89 fn bodies**
inside band A; the band delete takes them all regardless of the count.
Rows 40-42/68/70-75/77/86's f5 twins ride f5's shared arms — the f5
dispatch gate list in `g60_g59_f5_inter.json` names each.)

**The 133 WanderingEye latch note (the ONE non-mechanical bit):** f10's
`wandering_eye_hit` ports BOTH halves — the death arm (f5's twin covers it)
AND the hit-half below-half-life eye-pop gore 1 (`:86529-86533`, the
`localAI[0]` latch). f5's `wandering_eye_death` explicitly defers the latch
("g62's lane"); g62's `npc_hit_dust.rs` 133 arm says "the half-life gore
latch draws NOTHING (the body is d55's)" — but d55's pushed tip has NO 133
body. **The latch gore is unowned after the wholesale delete.** The merger
must hand the `:86529-86533` latch (one `NewGore(1)` + the `localAI[0]=1`
write, zero argument draws) to g62's hit-dust wiring (its 133 arm already
reads the gate) — a 5-line arm, NOT a reason to keep f10's fn.

**What f10 KEEPS** (sites 1-561, `:82206-85559`): the DD2 band, the
solar/nebula/vortex/stardust arms, the dutchman/desert/eclipse families, the
cultist/martian/granite/golem arms, the critter gated-dust, umbrella 225,
ice-tortoise 154, Slimer 121 (+ its `slimer_spawn` seam), the Betsy flipped
arm (e5's twin keeps both branches — the §12.4 DELETE stands), the 94 legit
witnesses + the Betsy-dispatch witness. The legit-window strips (§12.4's
f10 block: 68 kinds in 42 fn-rows + the ARG hand-offs) are UNAFFECTED by the
overrun delete — they live in the kept fns.

#### 12.8.4 The refreshed strip totals (FINAL-FINAL)

- **g59∩f5 = 93 kinds** (was 57 pushed + 18 conditional): g59's b11-13
  activations (61/62/157/175/217-219/220/221/226/229 + the skeleton-family
  21/31/32/44/45/77/110 + the town families 17/18/19/20/22/37/38/54/106/107/
  108/123/124/207/208/212-216/227/228/229) intersect f5's now-complete
  dispatch. Re-computed mechanically (`g60_g59_f5_inter.py` →
  `g60_g59_f5_inter.json`): **93 kinds in 47 f5 fn-rows**, every one verified
  to carry a dust re-draw (the 694 Water Bolt Mimic arm has none — not g59's
  anyway). The 18-conditional list loses **23** (f5 NO-gore, ports nothing)
  → 17 materialized; the other +19 come from g59's new activations. Full
  kind list: `2,5,6,7,10,13,17,18,19,20,21,22,24,26,27,28,29,31,32,35,36,37,
  38,39,42,43,44,45,47,48,49,51,54,56,57,58,61,62,65,66,73,77,78,79,80,82,83,
  84,85,86,87,93,94,95,96,97,98,99,100,101,106,107,108,110,111,117,123,124,
  140,157,175,190,191,192,193,194,207,208,212,213,214,215,216,217,218,219,
  220,221,222,226,227,228,229`.
- **g59∩f10 = {225}** unchanged (`umbrella_slime_death` @3451 at the pinned
  tip — a legit-window strip, NOT part of the overrun delete).
- **g60∩f5 = 32 kinds** — ALL pushed now (the 250/315/471 rows closed,
  §12.8.2). The §12.4 f5 block's fn map refreshes to f5's tip lines
  (the 23 fn-rows unchanged in shape; `stylist_death` @3907,
  `present_mimic_death` @4514, the three new rows above).
- **Strip-row total: 110 (g60 baseline) + 93 (g59∩f5) + 1 (g59∩f10 225) =
  204 actionable.** No conditional rows remain — f5 is complete. (The
  §12.3 "168 actionable / 186 eventual" is superseded: 168 + 17
  materialized + 19 g59-activation rows = 204.)
- **The f10-overrun delete is counted separately** (89 fn bodies +
  85 gates + 97 witnesses at `91b082e3c`, growing — re-census at merge).
  It auto-resolves the g60 kinds 348/349/350/351/489/490/262-264/266/267/
  329/338-340/342/343/353/354/368/369/376/441/550/588/589/633/663 and the
  g59 kinds in the window that f10 double-drew — those are NOT strip rows,
  they are DELETE rows (f5's twin keeps the gore, the dust strips ride
  f5's fn per §12.4).

#### 12.8.5 The union-8-step (re-pinned at the FINAL-FINAL tips)

§12.5's 8-step union plan stands, re-pinned: g59 `29c13d103` (2481 lines,
`death_dust` @314, `air()` 200×200 @1195, `ctx()` @1199, `corpse()` @1210,
**53 module witnesses**) × g60 `19431e385` (1998 lines, `DeathFx` @116
WIDENED, `death_dust_feed` @251, `lane_covers` @291, `death_dust_draws`
@310, `ctx()` @859, `air()` 200×120 @870, `replay_spawn` @878, **23 module
witnesses**). The ONE semantic collision is unchanged: the test `air()`
world height — rename g59's to `air_tall()`, keep g60's `air()`, repoint
g59's 53 witnesses (54 `air()` call sites incl. the helper). Imports union
to `{Npc, NpcKind}`. `main.rs` ONE `mod npc_death_dust;` (g59 `:66` / g60
`:162`). `world_sim/npcs.rs` keeps BOTH death-retain calls (g59 `:5114` /
g60 `:5044` — disjoint by kind). `anchors.txt` keeps BOTH rows. The
`last_hit_dir` ONE-declaration (§12.6) re-verified at the new tips: g59
field `:1240` / ctor `:2942` / records `:4412/:4467/:4580` / bubble reset
`:5654`; g60 field `:1240` / ctor `:2976` / records `:4441/:4496/:4609`
(no reset); c12 `5778e596` unchanged — dedupe to ONE declaration, KEEP
BOTH fuse resets. The `update_trails` ONE-fn (§12.7) is now the WIDENED
shape on both branches (§12.8.1) — trivial dedupe, plus g62's pooled-path
tail call + `HitDustSnapshot` fields stay g62's.

**Witness union (FINAL-FINAL):** g59 56 (53 module + 3 seam) + g60 29
(23 module + 2 npcs.rs + 4 npc.rs) + d55 7 + e5 12 + f10 94 (the legit
survivors of 191) + f5 142 + g62 66 (61 npc_hit_dust + 5 npc.rs) + c12 2
= 408 raw. The g60/g62 shared npc.rs witnesses are the SAME 4 byte-identical
tests (the 2 widened `w4_g60_*` + the 2 adopted `w4_g62_*`) — dedupe to ONE
copy: **404 distinct witnesses** owed a compile window at the merge (the
f10 overrun's 97 die with the delete).

---

*Generated from the lane tips d55 `ec78d152` / e5 `a62a32ce` / f10 `a69fb00c` /
g59 `eea6975b` / g60 `e55e2272` (merge-base `a725acb0`). The matrix was built by
walking each lane's full dispatch (`hit_effect`/`hit_effect_b`/`hit_effect_c`,
`death_dust`/`death_dust_draws`/`lane_covers`) against the g60 census artifacts
(`g60_kmap.json`, `g60_eligible.txt`, `g60_death_order.json`) and the C#
`NPC.cs:81654-89834` death arms. §11 re-censused at f10 `67c51ec7` / f5
`b33ba0a9` (claim `97c70c97`). §12 FINAL STATUS re-probed at f10 `f7a9dd78` /
f5 `d908a4fd` / g59 `dd336ef8` / g62 `53fdbf07` / c12 `5778e596` (g60
`ed7cef6e`). **§12.8 FINAL-FINAL re-probed at f10 `91b082e3c` (b134, ALIVE) /
f5 `d0619d8e9` (COMPLETE) / g59 `29c13d103` / g62 `32af98b44` / c12 `5778e596`
(g60 `19431e385`, the widening executed).** Code-only: no cargo/tests/rustfmt/
graphify invoked.*
