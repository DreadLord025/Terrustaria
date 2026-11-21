# 348 — W4-d32 RETIRE/SEAL: the NPC aiStyle band 85-93 FINAL census + declaration

**RENUMBERED 284→348** (registry-150 §8.7 AMENDMENT-3 FROZEN table,
d54 `9dddb32b4`, 2026-11-21): g52's `284-g52c-relay-host-arms-census.md`
first-add `0224fff80` 16:19:37 predates this doc's `4165e7dd9` 16:49:06 —
**g52 KEEPS 284** (executed pre-§8 authority); the §8.7 batch assigns
**d32 → 348**. Self-picks are INVALID outside the frozen table. Old
registry key 284 (this doc's former number); chain 284→348. All
cross-references (INDEX row,
ANALYSIS "The RETIRE/SEAL round", PORT_MAP ×2, ROADMAP W4-d32, the
anchors.txt seal canary, the npc.rs seal-test doc comments) re-pointed to
research/348 in the same commit.

Status: **SEALED** (W4-d32 retire round, 2026-11-21, worktree TR-W4-d32,
branch `task/W4-d32-npc-ai-styles-85-93`, tip `ac1d38e70`). Code-only —
no cargo/rustc/tests/rustfmt/graphify invoked. C# cites are
`decompiled/Terraria/NPC.cs` / `Projectile.cs` (stable), read in ≤100-line
windows, direct paths only (no junction recursion). Rust cites pin THIS
tip unless a lane tip is named.

**Registry number 348** per research/150 §8.7 AMENDMENT-3 (FROZEN:
d32-284→348; g52 KEEPS 284). History: the ORIGINAL pick was 284 per
§4-REFRESH ("Numbers 281+ are FREE", `38bfd1d22`+): the retire-tip scan
(2026-11-21 ~16:25 +02:00, all 361 remote refs + local heads,
`git ls-tree docs/research/` per tip) found **281 claimed by EIGHT lanes**
(d30 `ec89521fb` 15:42:58 first-add wins the key;
c25/c9/d106/d125/d9/f39/g48 later), **282 by three**
(d125/g48 15:44:38/15:50:24, f39 16:06:54), **283 by f4** (15:57:34),
**298 by g63** (16:08:43) — 284 first free at that scan, but it MISSED
g52's pre-§8 284 (`0224fff80` 16:19:37). §8.7 settles it: this doc takes
**348**; the 281/282 collisions were the registry's business (d54's), and
the §8/§8.7 frozen table is now the only authority.

## 1. The git-log census — what landed on the band (11 commits)

Base `a725acb04` (wave-3 landing, = origin/main at branch time; main has
since moved to `051600da8` — the band is NOT merged, this is a retire
seal, not a merge). `git log origin/main..HEAD` = exactly these 11:

| # | commit | scope |
| - | ------ | ----- |
| 1 | `a8e7d2a18` | style 85 axis claim: StardustCellBig 405 + BrainSuckler 421 PARTIAL machines move kind-match → axis arm (S1-D-2 migration, machines UNCHANGED verify-only), kind arms retired to `unreachable!`; 4 census witnesses |
| 2 | `a03232aa8` | style 86 port: shadow-flame apparitions 472/521, `npc_ai_style86.rs` (902 ln), UNWIRED; the real-DustPool in-step ring/trickle/box draws |
| 3 | `910d2c9b0` | style 87 port: Big Mimics 473-476, `npc_ai_style87.rs` (952 ln), UNWIRED; the 50-item stuff-cannon roster + the expert reflect seam |
| 4 | `425f1a1b0` | style 88 port: Mothron 477, `npc_ai_style88.rs` (1020 ln), UNWIRED; the 1000-iteration egg-site scan's two-draws-before-continue |
| 5 | `5393025df` | style 89 port: Mothron Egg 478, `npc_ai_style89.rs` (298 ln), UNWIRED; the Transform(479) request seam |
| 6 | `e91598b85` | style 90 port: Mothron Spawn 479, `npc_ai_style90.rs` (590 ln), UNWIRED; zero-draw by construction (env carries no stream) |
| 7 | `ca07f8e47` | style 91 port: Granite Flyer 483, `npc_ai_style91.rs` (572 ln), UNWIRED; the expert-only flinch short-circuit + the duplicate CanHit |
| 8 | `0449de3a5` | style 92 port: Target Dummy 488, `npc_ai_style92.rs` (348 ln), UNWIRED; the TETrainingDummy.Deactivate seam |
| 9 | `aa3a4a82c` | style 93 axis claim: FlyingDutchman 491 machine moves kind-match → axis arm (UNCHANGED verify-only), kind arm retired; cannons 492 keep the aiStyle-75 claim; 3 census witnesses |
| 10 | `839a4bcda` | fidelity fix 87/88/90/92: the dead/inactive-target gates read the LITERAL `Main.player[this.target]` slot — an unset 255/out-of-range slot is INACTIVE (dead false), not the slot-0 fallback; the steer reads keep the style74 fallback (the C# always retargets before them) |
| 11 | `ac1d38e70` | style 87 witness fix: the dead-target fixture seats target=0 (the literal-slot guard reads `Main.player[0]`) |

Every commit is code-only with witnesses WRITTEN NOT RUN. The band's
diff vs base: `main.rs` +25 (7 mod decls), `npc.rs` +246/-5 (the two
axis claims + the kind-arm retirements + the 7 census witnesses), the
seven new modules 4 682 ln. No other file touched
(`git diff --name-only a725acb04 HEAD` = 9 files, all band-owned).

## 2. All-styles-armed / kinds-rowed census + the remaining gaps

### 2a. The axis census at this tip (`npc.rs:4651-4959`, `update_rng_dust`)

30 style keys armed tree-wide: 5/9/21/25/27/28/29/30/31/34|35|36/43/47/
49/50/55/56/57/58/60/61/62/63/69/70/71/72/75/85/93. The band's rows:

| style | axis state | machine | kinds (cs_id) |
| ----- | ---------- | ------- | ------------- |
| 85 | **ARMED** (`npc.rs:4942`, kind-gated match) | `update_stardust_cell_big` (`:12826`) + `update_brain_suckler` (`:14875`) — the documented PARTIAL stand-ins, the row-85 verbatim audit OPEN | StardustCellBig 405, BrainSuckler 421 (base); DeadlySphere 467 = d40's row, see G2 |
| 86 | UNWIRED module (`main.rs:140`) | `style86_step` (`npc_ai_style86.rs:204`, pool-shaped `&mut [Npc]` + real DustPool/SpawnCtx) | 472 (d40), 521 (NOWHERE — G4) |
| 87 | UNWIRED (`main.rs:144`) | `style87_step` (`:324`, solo + `Style87Env`) | 473-476 (d40) |
| 88 | UNWIRED (`main.rs:148`) | `style88_step` (`:291`) | 477 (d40) |
| 89 | UNWIRED (`main.rs:152`) | `style89_step` (`:113`) | 478 (d40) |
| 90 | UNWIRED (`main.rs:156`) | `style90_step` (`:202`, pool-shaped, zero-draw env) | 479 (d40) |
| 91 | UNWIRED (`main.rs:160`) | `style91_step` (`:168`) | 483 (d40) |
| 92 | UNWIRED (`main.rs:164`) | `style92_step` (`:150`) | 488 (d40) |
| 93 | **ARMED** (`npc.rs:4958`) | `update_flying_dutchman` (`:11756`) — the row-93 verbatim audit OPEN (the `wants_scutlix` init latch / `crew_cannons` scan / `ai[0]` counter stand-ins + the missing sprite flip + 228 wake dusts, documented at the machine) | FlyingDutchman 491 (base) |
| 75 | ARMED (base, `npc.rs:4932`) | `style75_step` — the cannons' 240-reload/eight-ray-aim block (`npc_ai_style75.rs:594-668`) | FlyingDutchmanCannon 492 keeps its 75 claim (`data.rs:2392`) |

Zero call sites of `npc_ai_style8[6-9]::`/`npc_ai_style9[0-2]::` outside
their own modules (grep census) — the UNWIRED claim is exact.

### 2b. Kinds-rowed

- **Base (this tip)**: 405/421 (`data.rs:2410` → 85), 491 (`:2412` → 93),
  492 (`:2392` → 75). `CS_STYLES` (`npc.rs:31134`) carries the 491→93 pin.
- **d40 `8fe1d6d31`** (`task/W4-d40-npc-kinds-421-490`): rows 467/472/
  473-476/477/478/479/483/488 with `ai_style()` 85/86/87/88/89/90/91/92
  (its `data.rs:2790-2814`), the vanilla protocol ids kept, the
  SetDefaults/stat/sound/FindFrame-frame-0 arms landed, and the
  `update_kind_ai` arms are EMPTY `{}` ("the aiStyle machine is UNWIRED
  (the style73 precedent); the kind arm keeps the match exhaustive",
  its `npc.rs:6752-6785`). The d40 tree has NO 85/93 axis claim — this
  branch's claims are the band's only ones.
- **521 AncientCultistSquidhead**: NOT rowed on any roster tip
  (d39/d40/d41/d42/d43/c1 grep: only the `npc_ai_cultist.rs` doc cite
  `NPCID.cs:11719`). The 86 module's second kind waits on a roster wave.

### 2c. The remaining gaps (the merge fix-list)

| # | gap | owner / resolution |
| - | --- | ------------------ |
| G1 | styles 86-92 have NO axis arm — at the d40 merge the nine kinds tick AI-INERT (empty `{}` arms) | the wiring wave: add `86..=92` axis arms calling the modules. 86/90 are POOL-shaped (the sibling repulsion mutates both bodies) — they need the sim pre-pass over `App::npcs` (the style75 pooled-path precedent, `world_sim/npcs.rs`); 87/88/89/91/92 are solo adapters (the style74 precedent). The 88 brood-cap `CountNPCS(478)+CountNPCS(479)` + the 89 Transform(479) + the 92 Deactivate seams drain in the sim |
| G2 | **PANIC HAZARD**: d40 rows DeadlySphere 467 → `ai_style() == 85`, but this tip's 85 axis arm is `match kind { StardustCellBig, BrainSuckler, _ => unreachable! }` (`npc.rs:4942-4946`) — the d40 merge PANICS on the first 467 tick | merge fix-list: extend the 85 match with a `NpcKind::DeadlySphere` arm. The C# 467 branch of the aiStyle-85 arm (`NPC.cs:36930-37220`, the row-85 audit's unported third member) is the faithful target; until that audit lands, an empty `{}` arm with the documented stand-in note keeps the match exhaustive (the d40 empty-arm precedent). NOT fixed on this branch — the 467 kind does not exist here, the hazard is merge-time only |
| G3 | the row-85 + row-93 verbatim audits stay OPEN (the PARTIAL stand-ins are frozen verify-only) | the audit wave; the deviations are documented at `update_stardust_cell_big`/`update_brain_suckler`/`update_flying_dutchman` + the census-test module docs |
| G4 | 521 unrowed (the 86 module's second kind) | a roster wave (the cultist lanes cite it only); the 86 module's 521 parameter rows are already verbatim |
| G5 | the 478 MothronEgg Hide-no-ENTRIES-row bestiary gap | d40's, published by the g61 seal (`6e922fc60` §2: an own row per the g8 VampireBat precedent or a fold-aware lookup) — cross-ref only, not d32's |
| G6 | witness execution | the BUILD lane: **67 witnesses WRITTEN NOT RUN** (57 module: 86×11, 87×11, 88×10, 89×6, 90×7, 91×7, 92×5; 7 census in `npc.rs` `w4_d32_style85/93_census_tests`; 3 seal below). The commit-message counts (10 for 86/87) predate the fidelity-fix additions — the file census is authoritative |

RNG/stream: the band's live-path delta is ZERO new draws — the 85/93
claims route to the SAME machines with the same args (the frozen-A/B
witnesses pin 120-tick state+stream parity), the 86-92 modules are
unwired. The `839a4bcda` literal-slot guard is a fidelity fix inside
the unwired modules only.

## 3. Cross-lane keep-one receipts

**(a) d30 (band 65-72) — MUTUAL receipt, DISJOINT.**
d30's SEAL `ec89521fb` (research/281 §3a) pins d32 `ac1d38e70` as
disjoint: "ports 86-92 + claims 93 … grep ZERO for style65/butterfly
kinds 356/444/653/661 arms". The reciprocal holds at this tip: no
`npc_ai_style65.rs` in the tree (`git ls-tree HEAD` — the 66/67/68/70/
71/72 modules are base main's, not d30's), zero butterfly-kind arms,
the only "style72" hits are precedent comments citing main's landed
module. d30's four-copy style-65 keep-one (d30 canonical, f18/d39/d125
drops) is d30's business — d32 carries no copy, owes no drop.

**(b) d31 (band 75-83) — adjacent, DISJOINT styles, keep-both frictions.**
d31 `0487ea324`: claims 76-82 + ports 83 (`npc_ai_style83.rs`, UNWIRED).
No shared style with 85-93; the 75 claim is BASE main's (both lanes
inherit it unchanged). The merge frictions are textual, all keep-both:
1. `main.rs`: both insert mod decls after `mod npc_ai_style75;` (:139) —
   d31's `style83` + d32's `style86..92` compose in numeric order.
2. `npc.rs` axis: both insert after the `75 =>` arm (:4932) — d31's
   `76..=82` arms then d32's `85`/`93` arms, the C# if-chain order.
3. `npc.rs` kind-match retirements: disjoint kinds (d31: saucer core /
   Moon Lord trio / probe / true eye / leech blob; d32: suckler / big
   cell / ship) — no overlapping hunk.
4. `npc.rs` EOF: both append test modules after `s1_f_13d_tests`
   (:33546 base) — keep both (`w4_d31_axis_census_tests` +
   `w4_d32_style85/93_census_tests` + the seal module below).
5. docs: d31's ANALYSIS/PORT_MAP/ROADMAP/anchors rows + d32's are
   append-shaped — keep both.
The d10 activation contract (`649b18aef`, the 437/438 → style83 wiring +
the 526 drain owner re-pin) is d31/d10/d40's, untouched here.

**(c) g65-Kill-309 — COMPOSE, no keep-one, no drop owed.**
g65 `f6e07bb90` (research/26i, the Kill keep-one receipt at pushed tip
`b0842907c`): the "309" is TWO things, neither an NPC-style axis:
- the final accounting's "the drain's **309 distinct kind arms**"
  (ANALYSIS "Census completeness") is a COUNT (re-counted 317 in the
  26i receipt: 406 sites = 393 covered + 6 other-lane + 7 deferred);
- the drain's `309 =>` arm (`projectile_death_dust.rs:1100`) is
  **projectile 309 FrostBlastFriendly** (`projectile/data.rs:4326`,
  Kill `Projectile.cs:75347-75361`: 10 dust-185, per-dust Next(2) →
  vel×2/noGravity/scale×1.75). NPC 309 is **Scarecrow5**
  (`data.rs:448`, aiStyle 3) — a third id space. No band kind fires
  proj 309; the d30 seal witness pins the same disambiguation.

The band's ONLY projectile seam is the **492 cannon's 240 shell**:
`style75_step`'s 492 block pushes `HostileShotAi { kind: 240, … }`
(`npc_ai_style75.rs:630`, `NPC.cs:35771-35785` — the normalize×14 +
(0,-5) aim), the sim drain spawns **CannonballHostile 240**
(`projectile/data.rs:408` `DUTCHMAN_SHELL`, the row `:2522`). Its death
dust is **g65's 240 arm** (`projectile_death_dust.rs:3355`,
`Projectile.cs:75855-75894`: 20× dust-31 vel×1.4 + 10 pairs dust-6
(vel×5 noGravity / vel×3) + 4 gore draws Next(61,64)/Next(10,31)/
Next(-20,21); the 96-resize + Damage tail is g65-documented unported —
the kill-host lane's, not d32's). Compose verdict: the shot request is
d32's inherited 75 seam, the Kill body is g65's, the resize/Damage half
is the kill-host lane's — three disjoint owners, zero double-draw.
The 86 module's in-step DustPool draws are NPC-side on the UpdateNPCs
stream (the real `DustPool::new_dust` internals interleaved), never a
projectile Kill arm; the 87 stuff cannon is `Item.NewItem` requests
(the item lane, explicit-velocity no-fallback census); the 88/89/90/93
spawns are NPC spawns. d32 pushes ZERO cues and ZERO projectile Kill
dust — no g65/d77/g3 drop owed either direction.

**(d) d33 (band 95-127) — boundary confirmed.**
d33 `b08acf463` carries no 85-93 axis arm and no 86-92 module; its
405/421/491 kind arms are the BASE bodies (it branched pre-claims). At
merge, d32's axis claims + kind-arm retirements take those lines (d33's
diff does not touch them); d33's 95+ claims keep. No overlap.

## 4. FINAL declaration — the band 85-93

The NPC aiStyle band 85-93 is **CLAIM-COMPLETE**: every style has a
machine on a pushed branch — 85/93 axis-claimed on THIS branch (the
S1-D-2 migration step, machines frozen verify-only, the verbatim audits
open as G3), 86-92 ported verbatim as UNWIRED modules (the style86
precedent), the cannons' 75 claim inherited from base. The kinds are
rowed on d40 (467/472-479/483/488) + base (405/421/491/492); 521 is the
one unrowed census member (G4). The ONLY merge work is the §2c fix-list
(G1 the wiring wave's axis arms + sim drains, G2 the 467 panic-hazard
match extension) plus the §3b keep-both frictions with d31. No style,
kind, dust site or cue in the band is unclaimed; no keep-one drop is
owed to d30/d31/d33/g65. d32 owes nothing further — the follow-up BUILD
lane owns the 67-witness execution and the merge fix-lists.

**Seal witnesses** (in `npc.rs::w4_d32_seal_tests`, WRITTEN NOT RUN):
`w4_d32_seal_the_band_axis_census_is_exact` (the 85/93 kind sets + the
86-92 no-kind gap + the 492→75 split),
`w4_d32_seal_the_g65_kill_309_and_the_240_shell_are_other_id_spaces`
(the three-id-space pin + the shell const),
`w4_d32_seal_the_unwired_band_kinds_stay_unrowed_here` (the d40-gap
census at this tip).

**Tips read for this seal:** d32 `ac1d38e70` (this), main `051600da8`,
base `a725acb04`, d30 `ec89521fb` (research/281), d31 `0487ea324`,
d33 `b08acf463`, d40 `8fe1d6d31`, g65 `f6e07bb90`/`b0842907c`
(research/26i), d54 registry `38bfd1d22`+, g61 `6e922fc60` (the 478
gap), d10 `649b18aef` (the 83 contract, cross-ref only).
