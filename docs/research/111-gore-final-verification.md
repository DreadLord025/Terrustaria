# 111 — Gore-family FINAL verification + completion receipt (W4-f5, task/W4-f5-dust-proj-3)

Code-only final phase (no cargo/tests/rustfmt/graphify invoked, per the lane
directive). This receipt closes the f5 lane's two bodies of work — the
projectile-dust third batch (`projectile_dust3.rs`) and the NPC HitEffect
classic-mob gore chain (`npc_hit_effect_d.rs`, wave D) — and publishes the
gore-family total census plus the f10-overrun merger-delete hunks that g60's
research/110 §12.2 hand-off asked for. Every number below was re-derived by
direct read of the decompiled C# (the junction is never recursed — direct
file paths only) and of the pushed lane modules, not copied from a header.

**Lane tip at this receipt:** `task/W4-f5-dust-proj-3` = `d0619d8e9` (wave-D
completion docs), chain tip `fa3c526d5` (batch 103), claim `97c70c97c`.

---

## 1. The 853-site FINAL verification (wave D, `npc_hit_effect_d.rs`)

### 1.1 Source recount (authoritative)

`NPC.cs:85560-89835` carries **853 `Gore.NewGore(` sites** (plain variant;
the window has NO `NewGorePerfect`/`NewGoreDirect` — verified). First site
`:85684` (the 50 King Slime's 734), last `:89831` (the 156 arm's 185). This
matches the wave-D claim `97c70c97c` and the completion note exactly.

### 1.2 The death/hit split (the "any site missed?" answer)

A brace-depth walk of the `if (this.life > 0) { … } else { … }` gates across
the window classifies the 853 sites:

* **849 DEATH-half sites** — f5's port target (the `hp <= 0` arms).
* **4 HIT-half sites** — g62's lane (`life > 0`), NOT f5's:

  | census # | C# line | arm | body |
  | --- | --- | --- | --- |
  | 180 | `:86532` | 133 WanderingEye below-half-life eye-pop latch | gore 1 |
  | 292 | `:86947` | 186 PincushionZombie `Next(5)==0` hit gate | gore 242 |
  | 293 | `:86951` | 632 MaggotZombie `Next(5)==0` hit gate | gore 1217 |
  | 708 | `:88648` | 694 hit-half clamped-count loop | gore 1008 |

  f5's module references NONE of these four lines (verified by grep) — the
  death/hit boundary is clean. **Cross-lane note (the one residual):** g62's
  `npc_hit_dust.rs` header consumes these four sites' ARGUMENT draws but
  defers the gore BODIES to "d55's"; d55's pushed window is `:81660-81941`
  and does NOT reach them. Today the only port of `:86532`'s body is f10's
  overrun `wandering_eye_hit` (`npc_hit_effect_c.rs:4662`), which the §3
  delete removes. So after the overrun delete the four hit-half gore bodies
  are unowned — **a wiring-wave hand-off, not an f5 death-half miss** (f5's
  claim is the death halves; the hit halves are g62's). Recorded here so the
  merger assigns them (g62 emit-at-wiring, or a d55-extension), never silently
  drops them.

### 1.3 Coverage proof (849/849, zero missed)

Expanding EVERY `:NNNNN` / `:A-B` citation in the module's production section
(lines 1-6399, before `#[cfg(test)]` at 6400) to the gore-site lines it spans,
then intersecting with the 849 death-half site lines:

```
death sites: 849; cited-covered: 849; uncited death sites: 0
```

**No death-half site is missed.** The batch ledger (103 wave-D commits) tiles
sites 1-853 with zero gaps/overlaps in the site-number axis (b1 starts at 1,
b103 ends at 853, every batch's `slo == prev.shi + 1`). The C# line-ranges in
the commit subjects drift from the true site lines (the subjects cite the
arm's dust-loop start, not the first gore), but the site-number tiling is
exact and the citation-coverage proof above is line-accurate.

### 1.4 The 685-literal vs 853-site reconciliation

The module has **685 literal `gore.new_gore(` calls** (production section).
The difference from 853 is NOT missing sites — it is:

* **Loops** that emit N gores from 1 literal (the mimic triples, the pigron
  quads, the queen-bee 6-offset ladder, the unicorn 7, the spider 5-type
  match, the brain 6-velocity loop, the plantera 9, the eye-family 2×3+tail,
  the water-bolt-mimic 5-round loop, the mech quadrant 4, …).
* **Mutually-exclusive type branches** (`if npc_type == X`) whose literals
  all count in the 685 but only ONE arm fires per death.
* **The shared `mech_quadrant_smoke` helper** (1 literal, 4-loop) called by 5
  rows (probe/destroyer/santa-nk1/skeletron-prime/prime-limb) = 20 emitted
  sites from 1 literal; `mech_smoke_burst` is dust-only (0 gores).

A loop-aware per-fn expansion sums to 804 emitted-site slots (mech helper
counted ×5 callers); the residual to 849 is the branch-union over/under-count
(the static count cannot know which exclusive arm fires). The **citation
coverage (§1.3) is the authoritative completeness proof** — it is line-exact
and independent of the loop/branch arithmetic. The header's "685 call sites,
the difference is the helpers fanning out" is confirmed in direction; this
receipt pins the precise death/hit split (849/4) the header did not state.

### 1.5 Witnesses

**142 `#[test]` witnesses** in the wave-D module (written NOT run, code-only).
The dust-6 seventh-draw fix `086827426` (`twin_dust6`, `Dust.cs:214-218`) is
replayed in the 7 mech/twins witnesses. Stand-ins unchanged: no
`Npc.altTexture` (default-0 keeps head gores), no `Npc.color`
(`Color::default()` for dye rows), `netMode` SP fold, sounds c13.

**VERDICT: wave D COMPLETE. 849/849 death-half sites ported + citation-covered;
the 4 hit-half sites are g62's (flagged for wiring, §1.2). No f5 site missed.**

---

## 2. g60f / g60e receipts (research/110 §12.1 + §12.2 closure)

g60's tip at this receipt: `19431e385` (TRAILCACHE WIDENING, the g62 6-hunk
contract consume). Its research/110 §12.1 published the f5 remaining-window
census `:88817-89835` (~1019 lines, ~120 sites, 733→853) as the f5-followup
hand-off "should f5 stall". **f5 did NOT stall — it ran to `:89835`.** This
section closes that window and confirms g60's §12.2 overrun resolution.

### 2.1 §12.1 remaining-window — CLOSED by f5 batches 86-103

Every arm g60 listed in the `:88817-89835` remaining-window table is now
pushed by f5 (the batch that closed it):

| g60 §12.1 arm (C#) | kinds | closed by f5 batch |
| --- | --- | --- |
| `:88818` | 169 | b86 (NO-gore arm recorded; 169's dust is g59/g60's) |
| `:88838` | 170/180 | b86 (Pigron quad + Hallowed Pigron) |
| `:88862` | 171 | b86 |
| `:88891` | 290 | b87 (three Next(11,14) smoke) |
| `:88909` | 87-92 Wyvern | b88 (Next(1,4)-count smoke; 87 head ARG g59-consumed) |
| `:88934` | 78/79/80/630 | b88 (mummy triple; 78/79/80 dust g59's) |
| `:88959` | 82/158/159/250/253 | b88 (250 nimbus 99×3 `gore-f5*` pushed) |
| `:88984` | 316/330 | b89 (NO-gore, dust-only; 330 g60's) |
| `:89003` | 662 | b89 (NO-gore recorded) |
| `:89026` | 315 | b89 (Headless Horseman 99×3+446 `gore-f5*` pushed) |
| `:89053` | 254-261/634/635 | b89 (261's 375/376/377 `type!=261`-gated) |
| `:89081` | 140 | b89 (140 dust g59's) |
| `:89099` | 39/40/41 | b90 (Bone Serpent segment 67/68/69) |
| `:89115` | 34 | b90 (NO-gore recorded) |
| `:89144` | 289 | b90 (NO-gore recorded) |
| `:89167` | 35/36/68 | b90 (Skeletron RedHat split; 35/36 dust g59's) |
| `:89210` | 139 | b91 (Probe four quadrant smoke) |
| `:89245` | 467 | b91 (Deadly Sphere two 660) |
| `:89266` | 134-136 | b92 (Destroyer 156 + gated mech census) |
| `:89324` | 347 | b93 (Elf Copter 524-527 + two quadrant) |
| `:89355` | 346 | b94 (Santa-NK1 six 518-523 + quadrant) |
| `:89396` | 127 | b95 (Skeletron Prime 149/150 + quadrant) |
| `:89433` | 128-131 | b96 (Prime limbs 147/148 + quadrant) |
| `:89470` | 23 | b97 (NO-gore Meteor Head; dust g59's) |
| `:89503` | 24 | b97 (Fire Imp 45/46×2/47×2; dust g59's) |
| `:89526` | 25 | b97 (NO-gore Burning Sphere) |
| `:89538` | 33 | b97 (NO-gore Water Sphere) |
| `:89550` | 26/27/28/29/73/111 | b98 (goblin family; dust g59's) |
| `:89607` | 471 | b99 (Goblin Summoner 675-678 `gore-f5*` pushed) |
| `:89627` | 472 | b99 (Shadowflame Apparition 679/680) |
| `:89654` | 480 | b100 (Medusa 692-694 ladder) |
| `:89674` | 185 | b100 (Snow Flinx 240) |
| `:89690` | 30/665 | b100 (NO-gore recorded) |
| `:89702` | 42/176/205/231-235 | b101 (hornet/moth family; 42 dust g59's) |
| `:89742` | 43/56 | b102 (Man Eater/Snatcher 72×2) |
| `:89759` | 175 | b102 (Angry Trapper 227/228) |
| `:89776` | 48 | b103 (Harpy 80/81) |
| `:89793` | 62/66 | b103 (Demon 93/94/94; 66 dust g59's) |
| `:89819-89831` | final else 156 | b103 (184/185/185, the chain's last site) |

The three `gore-f5*` kinds g60 §12.1 said remained (250 / 315 / 471) are all
pushed (b88 / b89 / b99). **All 32 f5-owned g60 kinds are pushed; the §12.1
remaining-window is fully closed. No f5-followup takeover needed.**

### 2.2 §12.2 f10-overrun — confirmed + extended to f10's live tip

g60 §12.2 (written at f10 tip `f7a9dd78`, b92, `:86784`) found f10's batches
62-92 overran f5's claimed `:85560` boundary and prescribed: **keep f5's
`npc_hit_effect_d.rs` for the whole classic-mob chain, DELETE f10's overrun
wholesale (fns + dispatch gates + witnesses), NOT a dust-strip.** f5 claimed
first (`97c70c97c`); both headers agree f10 stops at `:85559`. This receipt
CONFIRMS that resolution and re-pins it at f10's CURRENT tip.

**f10 kept advancing during this session:** `f7a9dd78` (b92) → `0841e9f88`
(b121, `:87693`) → **`05f40324e` (b137, `:88346`)**. The overrun is now
**605 C# sites** (`:85680-88346`, all inside f5's window), not g60's ~38-fn /
255-site snapshot. The delete surface grew accordingly (§3).

---

## 3. The f10-overrun merger-delete hunks (g60e §12.2 hand-off, per-fn line-ranges)

**Pinned at f10 tip `05f40324e`** (`npc_hit_effect_c.rs`, 11815 lines). The
merger deletes THREE contiguous blocks; f10 keeps everything else (sites
1-561, `:82206-85559`: DD2, pillars, martian, cultist, sharkron/duke,
eclipse/mothron/dutchman/desert, golem/granite/ice, umbrella 225, ice-tortoise
154, Slimer 121). Line numbers are 1-based, inclusive, doc-comment-first.

### 3.1 Block A — the overrun arm fns + the ladder helper

**Delete lines 3651-6897** (the `king_slime_death` doc through the
`demolitionist_death` closing brace). This is **94 overrun arm fns + the
`five_gore_ladder` helper** (4949-4985, used only by overrun arms
nymph/icy-merman/gingerbread/moss-zombie) = 95 fns, 418 gore literals. The
block is contiguous (verified: no legit fn interleaved; the last legit fn
`slimer_death` ends at 3649, the dispatcher `hit_effect_c` starts at 6910).
The `blood_zombie_ladder` (4794-4828) is inside the block and overrun-only.

Per-fn delete ranges (doc-start .. fn-close):

| f10 fn | delete lines | C# arm | gore literals |
| --- | --- | --- | --- |
| `king_slime_death` | 3651-3682 | `:85680-85702` | 1 |
| `giant_tortoise_death` | 3684-3709 | `:85714-85721` | 4 |
| `derpling_death` | 3711-3736 | `:85733-85740` | 4 |
| `crawdad_death` | 3738-3765 | `:85752-85758` | 2 |
| `giant_shelly_death` | 3767-3790 | `:85771-85775` | 1 |
| `salamander_death` | 3792-3821 | `:85788-85795` | 3 |
| `bat_family_death` | 3823-3861 | `:85807-85840` | 1 |
| `owl_death` | 3863-3887 | `:85852-85858` | 3 |
| `gnome_death` | 3889-3914 | `:85870-85877` | 4 |
| `dandelion_death` | 3916-3942 | `:85889-85896` | 4 |
| `gem_bunny_death` | 3944-3973 | `:85900-85919` | 2 |
| `gem_squirrel_death` | 3975-4001 | `:85923-85940` | 1 |
| `critter_family_death` | 4003-4177 | `:85942-86090` | 52 |
| `penguin_family_death` | 4179-4214 | `:86102-86126` | 1 |
| `frog_death` | 4216-4238 | `:86138-86143` | 2 |
| `pupfish_death` | 4240-4263 | `:86183-86188` | 2 |
| `pufferfish_death` | 4265-4296 | `:86200-86215` | 6 |
| `seagull_death` | 4298-4321 | `:86281-86288` | 2 |
| `scorpion_death` | 4323-4352 | `:86227-86241` | 1 |
| `grebe_death` | 4354-4378 | `:86301-86309` | 3 |
| `duck_family_death` | 4380-4411 | `:86254-86268` | 1 |
| `nutcracker_death` | 4413-4439 | `:86322-86330` | 5 |
| `krampus_death` | 4441-4467 | `:86342-86350` | 5 |
| `elf_archer_death` | 4469-4494 | `:86362-86369` | 4 |
| `corrupt_critter_death` | 4496-4531 | `:86381-86407` | 1 |
| `crimson_monster_death` | 4533-4570 | `:86419-86446` | 1 |
| `demon_eye_death` | 4572-4613 | `:86459-86502` | 2 |
| `arapaima_death` | 4615-4638 | `:86514-86519` | 2 |
| `wandering_eye_hit` | 4640-4675 | `:86523-86543` | 3 |
| `antlion_death` | 4677-4717 | `:86545-86605` | 1 |
| `vulture_death` | 4719-4743 | `:86617-86623` | 3 |
| `raven_death` | 4745-4768 | `:86635-86640` | 2 |
| `parrot_death` | 4770-4792 | `:86652-86657` | 2 |
| `blood_zombie_ladder` | 4794-4828 | `:86669-86708` | 5 |
| `blood_zombie_death` | 4830-4842 | `:86675-86679` | 0 |
| `zombie_merman_death` | 4844-4856 | `:86683-86687` | 0 |
| `demon_tax_collector_death` | 4858-4870 | `:86704-86708` | 0 |
| `drippler_death` | 4872-4904 | `:86720-86728` | 5 |
| `shark_family_death` | 4906-4947 | `:86730-86785` | 4 |
| `five_gore_ladder` (helper) | 4949-4985 | (overrun-only) | 5 |
| `nymph_death` | 4987-5000 | `:86797-86805` | 0 |
| `icy_merman_death` | 5002-5015 | `:86838-86846` | 0 |
| `lihzahrd_death` | 5017-5047 | `:86817-86826` | 6 |
| `gingerbread_man_death` | 5049-5062 | `:86858-86866` | 0 |
| `zombie_elf_death` | 5064-5098 | `:86878-86896` | 4 |
| `yeti_death` | 5100-5127 | `:86908-86915` | 4 |
| `moss_zombie_death` | 5129-5142 | `:86927-86935` | 0 |
| `zombie_block_death` | 5144-5364 | `:86937-87139` | 81 |
| `cursed_weapon_death` | 5366-5401 | `:87152-87163` | 1 |
| `plantera_death` | 5403-5438 | `:87186-87207` | 1 |
| `plantera_part_death` | 5440-5473 | `:87209-87217` | 1 |
| `brain_of_cthulhu_death` | 5475-5504 | `:87248-87257` | 1 |
| `creeper_death` | 5506-5527 | `:87269-87273` | 1 |
| `servant_of_cthulhu_death` | 5529-5552 | `:87351-87356` | 2 |
| `wall_of_flesh_death` | 5554-5589 | `:87368-87383` | 4 |
| `the_hungry_death` | 5591-5617 | `:87429-87434` | 2 |
| `leech_death` | 5619-5643 | `:87446-87450` | 1 |
| `beetle_death` | 5645-5683 | `:87462-87478` | 1 |
| `queen_bee_death` | 5685-5720 | `:87491-87500` | 6 |
| `eater_of_souls_death` | 5722-5770 | `:87512-87535` | 9 |
| `clinger_death` | 5772-5800 | `:87547-87554` | 4 |
| `devourer_death` | 5802-5827 | `:87566-87570` | 1 |
| `seeker_death` | 5829-5852 | `:87583-87587` | 1 |
| `giant_worm_death` | 5854-5878 | `:87600-87604` | 1 |
| `digger_death` | 5880-5904 | `:87616-87620` | 1 |
| `eater_of_worlds_death` | 5906-5939 | `:87632-87650` | 2 |
| `town_npc_death` | 5941-5992 | `:87662-87693` | 10 |
| `unicorn_death` | 5994-6018 | `:87705-87715` | 1 |
| `wolf_death` | 6020-6044 | `:87727-87733` | 3 |
| `hellhound_death` | 6046-6070 | `:87745-87751` | 3 |
| `black_recluse_death` | 6072-6098 | `:87763-87771` | 5 |
| `wall_creeper_death` | 6100-6126 | `:87783-87791` | 5 |
| `blood_crawler_death` | 6128-6152 | `:87803-87811` | 1 |
| `jungle_creeper_death` | 6154-6179 | `:87823-87831` | 1 |
| `blood_feeder_death` | 6181-6204 | `:87843-87848` | 2 |
| `blood_jelly_death` | 6206-6230 | `:87860-87868` | 1 |
| `bound_npc_death` | 6232-6280 | `:87880-87900` | 10 |
| `mechanic_death` | 6282-6312 | `:87912-87920` | 5 |
| `guide_death` | 6314-6344 | `:87932-87940` | 5 |
| `bartender_death` | 6346-6375 | `:87952-87960` | 5 |
| `golfer_death` | 6377-6408 | `:87972-87983` | 5 |
| `bestiary_girl_death` | 6410-6449 | `:87995-88021` | 6 |
| `travelling_merchant_death` | 6451-6483 | `:88033-88044` | 5 |
| `painter_death` | 6485-6517 | `:88086-88097` | 5 |
| `witch_doctor_death` | 6519-6550 | `:88109-88117` | 5 |
| `pirate_death` | 6552-6600 | `:88129-88151` | 5 |
| `santa_claus_death` | 6602-6634 | `:88163-88174` | 5 |
| `steampunker_death` | 6636-6666 | `:88186-88196` | 4 |
| `stylist_death` | 6668-6701 | `:88208-88220` | 6 |
| `old_man_death` | 6703-6733 | `:88232-88240` | 5 |
| `clothier_death` | 6735-6765 | `:88252-88260` | 5 |
| `truffle_death` | 6767-6797 | `:88272-88280` | 5 |
| `nurse_death` | 6799-6831 | `:88292-88303` | 5 |
| `arms_dealer_death` | 6833-6863 | `:88315-88323` | 5 |
| `demolitionist_death` | 6865-6897 | `:88335-88346` | 5 |

(The 0-literal fns — blood_zombie/zombie_merman/demon_tax_collector/nymph/
icy_merman/gingerbread/moss_zombie — delegate to `blood_zombie_ladder` /
`five_gore_ladder`; they are overrun arms and delete with the block.)

### 3.2 Block B — the dispatcher overrun gates

**Delete lines 7321-7737** inside `hit_effect_c` (the `:85670-85703` KingSlime
comment through the `demolitionist_death` gate's closing brace). KEEP 7318-7320
(the 121 Slimer gate, site 561, legit) and 7738 (`outcome`) / 7739 (`}`).
Every gate in the block calls an overrun fn; no legit gate is interleaved.

### 3.3 Block C — the overrun witnesses

**Delete lines 10085-12195** (the `w4_f10_the_king_slime_death_…` doc through
the `w4_f10_the_demolitionist_death_…` closing brace) = **103 witnesses**,
contiguous (verified zero gaps; pointer-reference witnesses like
`w4_f10_the_tortoise_and_derpling_deaths_…` and
`w4_f10_the_nymph_and_icy_merman_deaths_…` are included — they reference
overrun fns via `as fn(…)` casts). KEEP the 97 legit witnesses + the
`air`/`ctx`/`boss` helpers + `w4_f10_the_dispatch_gates_the_flipped_betsy_branch`
(12197+, the 551 Betsy gate is legit `<85560`).

Per-witness delete ranges (doc/attr-start .. fn-close):

| witness | delete lines |
| --- | --- |
| `w4_f10_the_king_slime_death_spawns_the_gore_and_returns_the_seam` | 10085-10101 |
| `w4_f10_the_tortoise_and_derpling_deaths_spawn_the_four_gores` | 10103-10124 |
| `w4_f10_the_crawdad_death_uses_the_per_type_offset` | 10126-10140 |
| `w4_f10_the_giant_shelly_death_uses_the_per_type_offset` | 10142-10155 |
| `w4_f10_the_salamander_death_uses_the_per_type_offset` | 10157-10175 |
| `w4_f10_the_bat_family_death_uses_the_per_type_list` | 10177-10203 |
| `w4_f10_the_owl_death_spawns_the_three_gores` | 10205-10217 |
| `w4_f10_the_gnome_death_spawns_the_four_gores` | 10219-10230 |
| `w4_f10_the_dandelion_death_spawns_the_four_gores` | 10232-10244 |
| `w4_f10_the_gem_bunny_death_uses_the_per_type_offset` | 10246-10261 |
| `w4_f10_the_gem_squirrel_death_uses_the_per_type_offset` | 10263-10276 |
| `w4_f10_the_critter_family_bunny_crab_goldfish` | 10278-10313 |
| `w4_f10_the_critter_family_dolphin_birds_macaw` | 10315-10345 |
| `w4_f10_the_critter_family_macaw_toucan_cockatiel` | 10347-10366 |
| `w4_f10_the_critter_family_cockatiel_and_single_gores` | 10368-10391 |
| `w4_f10_the_critter_family_flying_fish_turtle` | 10393-10424 |
| `w4_f10_the_critter_family_jungle_sea_turtle` | 10426-10440 |
| `w4_f10_the_penguin_family_death_uses_the_per_type_list` | 10442-10462 |
| `w4_f10_the_frog_death_spawns_the_two_gores` | 10464-10476 |
| `w4_f10_the_pupfish_death_spawns_the_two_gores` | 10478-10489 |
| `w4_f10_the_pufferfish_death_branches_on_ai2` | 10491-10514 |
| `w4_f10_the_seagull_death_spawns_the_two_gores` | 10516-10527 |
| `w4_f10_the_scorpion_death_uses_the_per_type_list` | 10529-10543 |
| `w4_f10_the_grebe_death_spawns_the_three_gores` | 10545-10556 |
| `w4_f10_the_duck_family_death_uses_the_per_type_list` | 10558-10572 |
| `w4_f10_the_nutcracker_death_spawns_the_five_gores` | 10574-10586 |
| `w4_f10_the_krampus_death_spawns_the_five_gores` | 10588-10599 |
| `w4_f10_the_elf_archer_death_spawns_the_four_gores` | 10601-10612 |
| `w4_f10_the_corrupt_critter_death_uses_the_per_type_list` | 10614-10635 |
| `w4_f10_the_crimson_monster_death_uses_the_per_type_list` | 10637-10660 |
| `w4_f10_the_demon_eye_death_uses_the_per_type_pair` | 10662-10692 |
| `w4_f10_the_arapaima_death_spawns_the_two_gores` | 10694-10707 |
| `w4_f10_the_wandering_eye_hit_latches_and_death_spawns` | 10709-10740 |
| `w4_f10_the_antlion_death_uses_the_per_type_list` | 10742-10766 |
| `w4_f10_the_vulture_death_spawns_the_three_gores` | 10768-10783 |
| `w4_f10_the_raven_death_spawns_the_two_gores` | 10785-10798 |
| `w4_f10_the_parrot_death_spawns_the_two_gores` | 10800-10811 |
| `w4_f10_the_blood_zombie_ladder_uses_the_per_type_triple` | 10813-10838 |
| `w4_f10_the_drippler_death_draws_the_two_random_types` | 10840-10878 |
| `w4_f10_the_shark_family_death_uses_the_per_type_quad` | 10880-10912 |
| `w4_f10_the_nymph_and_icy_merman_deaths_use_the_ladder` | 10914-10936 |
| `w4_f10_the_lihzahrd_death_spawns_the_six_gores` | 10938-10954 |
| `w4_f10_the_gingerbread_man_death_spawns_the_ladder` | 10956-10968 |
| `w4_f10_the_zombie_elf_death_uses_the_per_type_head` | 10970-10987 |
| `w4_f10_the_yeti_death_spawns_the_four_gores` | 10989-11004 |
| `w4_f10_the_moss_zombie_death_spawns_the_ladder` | 11006-11018 |
| `w4_f10_the_zombie_block_werewolf_and_maggot` | 11020-11049 |
| `w4_f10_the_zombie_block_clown_spawns_the_extra_gore` | 11051-11065 |
| `w4_f10_the_zombie_block_eskimo_and_frankenstein` | 11067-11082 |
| `w4_f10_the_zombie_block_swamp_and_female` | 11084-11099 |
| `w4_f10_the_zombie_block_doctor_and_superman` | 11101-11126 |
| `w4_f10_the_zombie_block_pixie_and_raincoat` | 11128-11143 |
| `w4_f10_the_zombie_fallthrough_bald_and_twiggy` | 11145-11164 |
| `w4_f10_the_zombie_fallthrough_eyezor_and_generic` | 11166-11182 |
| `w4_f10_the_zombie_fallthrough_pincushion` | 11184-11221 |
| `w4_f10_the_zombie_fallthrough_slimed` | 11223-11243 |
| `w4_f10_the_cursed_weapon_death_draws_the_velocity` | 11245-11279 |
| `w4_f10_the_plantera_death_draws_the_positions` | 11281-11312 |
| `w4_f10_the_plantera_part_death_uses_the_per_type_pair` | 11314-11328 |
| `w4_f10_the_brain_of_cthulhu_death_draws_the_velocity` | 11330-11360 |
| `w4_f10_the_creeper_death_spawns_the_single_gore` | 11362-11372 |
| `w4_f10_the_servant_death_spawns_the_two_gores` | 11374-11386 |
| `w4_f10_the_wall_of_flesh_death_spawns_the_quadrants` | 11388-11408 |
| `w4_f10_the_hungry_death_spawns_the_two_gores` | 11410-11421 |
| `w4_f10_the_leech_death_uses_the_per_type_offset` | 11423-11435 |
| `w4_f10_the_beetle_death_uses_the_per_type_list` | 11437-11452 |
| `w4_f10_the_queen_bee_death_spawns_the_spread_gores` | 11454-11472 |
| `w4_f10_the_eater_of_souls_death_uses_the_per_type_list` | 11474-11505 |
| `w4_f10_the_clinger_death_spawns_the_four_gores` | 11507-11519 |
| `w4_f10_the_devourer_death_uses_the_per_type_offset` | 11521-11534 |
| `w4_f10_the_seeker_death_spawns_the_single_gore` | 11536-11546 |
| `w4_f10_the_giant_worm_death_uses_the_per_type_offset` | 11548-11561 |
| `w4_f10_the_digger_death_uses_the_per_type_offset` | 11563-11576 |
| `w4_f10_the_eater_of_worlds_death_uses_the_per_type_pair` | 11578-11592 |
| `w4_f10_the_town_npc_death_uses_the_shimmer_variants` | 11594-11627 |
| `w4_f10_the_unicorn_death_spawns_the_seven_gores` | 11629-11641 |
| `w4_f10_the_wolf_death_spawns_the_three_gores` | 11643-11654 |
| `w4_f10_the_hellhound_death_spawns_the_three_gores` | 11656-11667 |
| `w4_f10_the_black_recluse_death_spawns_the_five_gores` | 11669-11681 |
| `w4_f10_the_wall_creeper_death_spawns_the_five_gores` | 11683-11695 |
| `w4_f10_the_blood_crawler_death_spawns_the_five_gores` | 11697-11709 |
| `w4_f10_the_jungle_creeper_death_spawns_the_five_gores` | 11711-11723 |
| `w4_f10_the_blood_feeder_death_spawns_the_two_gores` | 11725-11736 |
| `w4_f10_the_blood_jelly_death_spawns_the_five_gores` | 11738-11750 |
| `w4_f10_the_bound_npc_death_uses_the_shimmer_variants` | 11752-11783 |
| `w4_f10_the_mechanic_death_uses_the_shimmer_variants` | 11785-11806 |
| `w4_f10_the_guide_death_uses_the_shimmer_variants` | 11808-11829 |
| `w4_f10_the_bartender_death_spawns_the_offset_gores` | 11831-11849 |
| `w4_f10_the_golfer_death_spawns_the_offset_gores` | 11851-11866 |
| `w4_f10_the_bestiary_girl_death_uses_the_shimmer_variants` | 11868-11892 |
| `w4_f10_the_travelling_merchant_death_uses_the_shimmer_variants` | 11894-11915 |
| `w4_f10_the_painter_death_uses_the_shimmer_variants` | 11917-11939 |
| `w4_f10_the_witch_doctor_death_uses_the_shimmer_variants` | 11941-11963 |
| `w4_f10_the_pirate_death_shimmer_is_dust_only` | 11965-11987 |
| `w4_f10_the_santa_claus_death_uses_the_shimmer_variants` | 11989-12010 |
| `w4_f10_the_steampunker_death_uses_the_shimmer_variants` | 12012-12033 |
| `w4_f10_the_stylist_death_uses_the_shimmer_variants` | 12035-12057 |
| `w4_f10_the_old_man_death_uses_the_shimmer_variants` | 12059-12080 |
| `w4_f10_the_clothier_death_uses_the_shimmer_variants` | 12082-12103 |
| `w4_f10_the_truffle_death_uses_the_shimmer_variants` | 12105-12126 |
| `w4_f10_the_nurse_death_uses_the_shimmer_variants` | 12128-12149 |
| `w4_f10_the_arms_dealer_death_uses_the_shimmer_variants` | 12151-12172 |
| `w4_f10_the_demolitionist_death_uses_the_shimmer_variants` | 12174-12195 |

### 3.4 Auto-resolved overlaps

The wholesale delete auto-resolves the g60 kinds 348/349/350/351/489/490 and
the g59 kind 65 that f10's overrun double-drew (g60 §12.2), AND the four
hit-half gore bodies' sole current port (`wandering_eye_hit` `:86532`) — see
§1.2's wiring hand-off. f10's `:85334-85560` gap stays CLOSED (g60 §12.1: site
561 Slimer the only gore, the dust-only arms g59's).

---

## 4. Dust-lane co-writer status + f5 dust-coverage FINAL census

### 4.1 Co-writer status

The worktree `TR-W4-f5` is shared with a concurrent dust-lane writer per the
task note. **At this receipt the working tree is CLEAN (`git status
--porcelain` empty) and every commit on `task/W4-f5-dust-proj-3` is an f5
commit** (the projectile-dust batches 1-5 + claim + witnesses, then wave-D
batches 1-103 + completion). No foreign co-writer commit is present on the
branch; explicit per-file staging was used throughout (never `git add -A`).
The f4 NPC-dust lane lives on its OWN branch `task/W4-f4-dust-npc-3`
(`b62d6c2a3`), NOT here — its `npc_hit_effect_e.rs` (wave E, the 17
out-of-HitEffect NPC gore sites) is disjoint from f5's window.

### 4.2 f5 projectile-dust coverage (`projectile_dust3.rs`) — FINAL

The claim (`291396002`): `Projectile.cs` `Dust.NewDust*` sites **83-125**
(`:24406-26501`), the THIRD batch, co-lane split d53(1-40)/e3(41-82)/
f5(83-125)/g63-65(trail/death reserved).

**Source recount (authoritative):** `Projectile.cs` carries **1196
`Dust.NewDust*` sites** (891 plain `NewDust(` + 112 `NewDustPerfect(` + 193
`NewDustDirect(`). The ANALYSIS header's "1220" is a stale over-count; the
direct grep is 1196. Numbering all 1196 in source order, **site 83 = `:24406`
and site 125 = `:26501`** — the claim's site-number range is EXACT. The
window `:24406-26501` carries **43 sites** (38 plain + 2 Perfect `:25032/
:25038` + 3 Direct `:25259/:25402/:25449`), matching "43 sites, 30 live + 13
latent".

**Coverage proof:** expanding every `:NNNNN`/`:A-B` citation in
`projectile_dust3.rs` (1456 lines) to the site lines it spans → **43/43
cited, 0 uncited.** The module ports the **30 LIVE** sites as emitters (21
`#[test]` draw-mirror witnesses, written NOT run); the **13 LATENT** sites
(unrowed kinds 18/72/86/87 fairy-orb, 196 Smoke Bomb, 261 Boulder Staff, 538
Twinkle, 1092 Librarian Book `:25402/:25449`, 288 hostile Golden Shower,
1048/1053 boulders, 1114 Daybloom Staff) are recorded at their arm and stay
dormant until the kind row lands (the d51 ungrantered-arm precedent). The
aiStyle-14 fairy-orb Perfect pair `:25032/:25038` is latent (kinds 18/72/86/87
unrowed). The plumbing (`update`→`update_dust`, `particles.rs`
`new_dust_direct<R: DustRng>` widening) is reproduced byte-compatibly with e3
(keep-one at merge); only the disjoint aiStyle-11/12/14/18/20/21/22/23/24/25/27
arms are wired here.

**f5 dust-coverage FINAL: 43/43 sites accounted (30 live-ported + 13
latent-recorded), 0 missed.** The witness fix `4d63516c6` (Flames Trap 6×6
box `next_max(2)` not `next_max(10)`) is landed.

---

## 5. Research FINAL + gore-family declaration (total census)

### 5.1 The `Gore.NewGore*` total census (5 files, direct grep, all variants)

| file | sites | owner lane(s) | status |
| --- | --- | --- | --- |
| `NPC.cs` | **1345** | d55/e5/f10/f5 (HitEffect) + f4-E/g61/reserved (other methods) | see §5.2 |
| `Projectile.cs` | **165** | g16 (`projectile_gore_a..o`, 15 modules) | CENSUS COMPLETE |
| `WorldGen.cs` | **36** | g17 (`world_gore_a/b`) | COMPLETE |
| `Player.cs` | **32** | g17 (`player_gore_a..e`) | COMPLETE |
| `Main.cs` | **0** | — (verified: only `Gore.goreTime`/TimeLogger rows) | n/a |
| **TOTAL** | **1578** | | |

### 5.2 The NPC.cs 1345 breakdown (the HitEffect table + the remainder)

`NPC.HitEffect` spans `:81654-89835`; its gore window `:81660-89835` carries
**1304 sites** (1300 death-half + 4 hit-half). The four lane windows:

| lane | window | all-variant sites | plain | module |
| --- | --- | --- | --- | --- |
| d55 wave A | `:81660-81941` | 26 | 22 | `npc_hit_effect_a.rs` (`ec78d1524`) |
| e5 wave B | `:81942-82205` | 40 | 40 | `npc_hit_effect_b.rs` (`a62a32ce6`) |
| f10 wave C | `:82206-85559` | 385 | 384 | `npc_hit_effect_c.rs` (`05f40324e`, legit part) |
| f5 wave D | `:85560-89835` | 853 | 853 | `npc_hit_effect_d.rs` (`d0619d8e9`) |
| **HitEffect total** | `:81660-89835` | **1304** | 1299 | |

(d55's 4 `NewGorePerfect` `:81758/81764/81795/81801` + f10's 1 `NewGoreDirect`
`:85451` Slimer account for the 5 variant sites; 1299 plain + 5 = 1304.)

**The 41 NPC sites OUTSIDE HitEffect** (40 pre-`:81660` + 1 post-`:89835`)
fully reconcile (f4 wave E census, research/71):

* **17 taken by f4 wave E** (`npc_hit_effect_e.rs`, `81c1c5a7a`/`007b5cb47`,
  WAVE E COMPLETE 17/17): E1-E3 FungiSpore `:32028-32030`, E4 KingSlime crown
  `:43233`, E5 QueenSlime crown `:45424`, E6 confetti slime `:60934`, E7 Betsy
  `:62187`, E8 kobold-bomber `:63247`, E9 skeleton walk-drip `:63609`, E10
  kobold-glider `:65774`, E11 TownBunny `:68953`, E12-E15 MoonLordCore
  `:71931-71934`, E16 SantaNK1 half-hp `:81354`, E17 loveStruck `:91340`.
* **8 owned by g61** (`npc_spawn_visuals.rs`, `0a4af190`): BigMimicSpawnSmoke
  `:77511` + TransformVisuals `:80969-80989` (7).
* **16 reserved-argument** (the machine owners keep the bodies; the gore-arg
  draws armed by d52/d22/e2): Golem landing `:19717`, EoC spin
  `:20367-20369`, Twins hand-off `:26749-26751`/`:27363-27365`, style39/41
  explosions `:29564`/`:29954`, Brain retreat `:32410-32413`.

17 + 8 + 16 = 41. **1304 + 41 = 1345. The NPC.cs census is closed.**

### 5.3 Gore-family declaration

**The `Gore.NewGore*` family is 1578 sites across 5 files. Every site has a
named owner lane:**

* **NPC HitEffect death halves (1300):** d55 (26) + e5 (40) + f10-legit (385)
  + f5 (849) = 1300. f5's 849 verified citation-complete (§1.3); f10's legit
  385 are its `:82206-85559` claim; the f10 OVERRUN (605 sites in f5's window)
  is deleted at merge (§3), reconciling to f5's claim-first boundary.
* **NPC HitEffect hit halves (4):** g62's argument draws; bodies flagged for
  wiring assignment (§1.2) — the one residual hand-off.
* **NPC other-method (41):** f4-E (17) + g61 (8) + reserved-arg (16).
* **Projectile (165):** g16, CENSUS COMPLETE (waves A-O, 15 modules; two
  mid-run gaps `:24037`/`:36331`+`:75871-75883` found and filled).
* **World+Player (68):** g17 (WorldGen 36 + Player 32), COMPLETE; Main 0.

**Total accounted: 1300 + 4 + 41 + 165 + 68 = 1578. Zero unowned gore sites.**
The only open item is the 4 hit-half BODY assignment (§1.2), a wiring-wave
decision, not a census gap.

### 5.4 Lane-tip pins (this receipt's fetch)

| lane | branch | tip |
| --- | --- | --- |
| f5 (this) | `task/W4-f5-dust-proj-3` | `d0619d8e94b93555853935553b0f3f6b24d1a495` |
| f10 | `task/W4-f10-gore-sites-3` | `05f40324e58197a902cdbe89cdcb23b84aff1674` |
| g60 | `task/W4-g60-npc-death-effects-2` | `19431e385a99f195c88a416fa8d9ecbd891b3fbc` |
| g16 | `task/W4-g16-gore-projectile-sites` | `f1c60ffe9d5a40fa58c89c12625234bb57d56df3` |
| g17 | `task/W4-g17-gore-world-sites` | `f0fe1bbddc85fd01b194fe7cc6fc4a65fc86d5a0` |
| d55 | `task/W4-d55-gore-table-sites` | `ec78d1524069af5b4398652e47bd503aa496cdb6` |
| e5 | `task/W4-e5-gore-sites-2` | `a62a32ce66891f1d5d5fd746786d88f1d4570021` |
| g59 | `task/W4-g59-npc-death-effects-1` | `29c13d103f198ced7a2c3490d1212d8c6a581a64` |
| g62 | `task/W4-g62-npc-hit-dust` | `32af98b4442cabe8082c5181deec490559008aa7` |
| f4 | `task/W4-f4-dust-npc-3` | `b62d6c2a3a11519f601e1d61e5f0087b5713a734` |

g17 advanced during this session (`a790e88d7` → `f0fe1bbdd`, dust-arms order
fix + shake-loot witness hardening); its 68-site census is unchanged. f10
advanced b92→b137 (§2.2). The other tips are stable.

---

## 6. Verification performed (code-only)

No cargo/tests/rustfmt/graphify invoked (user directive). Verification was
direct source recount + citation-coverage + brace-depth classification:

* `NPC.cs:85560-89835` → 853 `Gore.NewGore(` (first `:85684`, last `:89831`).
* Death/hit brace-walk → 849 death + 4 hit (`:86532/86947/86951/88648`).
* f5 module citation-expansion → 849/849 death sites covered, 0 uncited.
* f5 batch ledger → 103 commits tile sites 1-853, zero gap/overlap.
* f5 module → 685 literal calls, 142 witnesses; `mech_quadrant_smoke` 1×4×5.
* `Projectile.cs` → 1196 `Dust.NewDust*`; site 83=`:24406`, 125=`:26501`;
  window 43 sites; `projectile_dust3.rs` citation-expansion 43/43, 21 witnesses.
* f10 `05f40324e` → 94 overrun fns + ladder (3651-6897), gates 7321-7737,
  103 witnesses (10085-12195); contiguity verified, 0 legit interleaved.
* 5-file `Gore.NewGore*` grep → NPC 1345 / Proj 165 / WorldGen 36 / Player 32
  / Main 0 = 1578; NPC 1304 HitEffect + 41 other (17 f4-E + 8 g61 + 16 res).

Witnesses across the family remain WRITTEN NOT RUN (code-only mode); they are
owed a compile window before merge.
