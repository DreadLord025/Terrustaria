# 353 — W4-d13 FINAL: the projectile aiStyles 121-130 band bible

> **CHAIN 305 → 353** (registry-150 §9 UNIFIED FINAL, `b243c7086`):
> §9.2 assigns d13's band-final **353**; **f4 KEEPS 305** (§7.4.3 —
> f4's `305-npc-dust-final.md`, self-corrected `331b29858`); the lane's
> retire FINAL chains 243→304→**336** (304=d50 §7.4). Chain history:
> number 305 was picked per registry round 3 (`51a28d09e`) —
> 151–236 + 246–303 assignment blocks, 237–245 winner-consumed,
> 304+ read FREE at that round ("Do NOT pick max-on-origin+1; check this
> registry AND a fresh all-tip sweep first"). Fresh sweep executed then
> (per-tip `git ls-tree docs/research/` over ALL origin + local refs AND
> the full-history `git log --all --name-only` scan): zero 304/305 docs
> anywhere; the SEAL round (`e310bd6e7`) executed the retire FINAL at
> 304 (renumbered 243→304, g58 owns 243) and this band bible at 305.
> §8/§9 later assigned 304 to d50 and confirmed 305 to f4, so this
> rename executes **305 → 353** per §9.2 on THIS branch only.
>
> Branch `task/W4-d13-proj-ai-121-130`, base `a725acb04` (wave-3
> complete), 2026-11-21. CODE-ONLY round: every witness is WRITTEN NOT
> RUN. This is the band's final ledger — one row per style arm, the
> port state, the commit hash, the C# anchor, the witness count, the
> cross-lane composition notes and the SEAL receipts. The per-arm
> censuses live in the module docs + the commit messages; the residual
> deviations in ANALYSIS "W4-d13 residuals" + "The activation wave";
> the retire census + the pet-light-family declaration in research/336.

## 1. The band table (all ten styles censused; five arms this lane's)

| style | kind(s) | C# arm | port fn | commit | witnesses | notes |
|-------|---------|--------|---------|--------|-----------|-------|
| 121 | StardustDragon 625/626/627/628 | `AI_121_StardustDragon` `Projectile.cs:42824-43080` (gate `:34892-34895`) | `dragon_step` (`projectile.rs:21857`, dispatch `:17643`) | PRE-EXISTING — the i25/i56 chain AI, V2 fix `b3f0db4cb` | (the i56/i25 ledger's) | the rows `data.rs:3507-3508` (24×24, pen −1, 18000, local 7, alpha 255; the 626/627 `minionSlots 0.5` `:6735-6738`); ONE `Next(30)` gate draw per chain slot (`:42850`), the dust bodies skipped; the `%120` netUpdate cadence folds to `update_count` (net bookkeeping only) |
| 122 | TowerDamageBolt 629 (+ PhantasmArrow 631 in C#) | inline `:34896-34965` | `ai_tower_damage_bolt` (`projectile.rs:4048`, dispatch `:18283`) | PRE-EXISTING — the B-6 pillar wave (the R71/R72 shield reconciliation) | (the B-6 ledger's) | the 629 row `data.rs:2497` (8×8, pen −1, extraUpdates 2, hostile); the contact's shield spend −1/tick (`Kill` `:72944-72979`, `tower_shield_absorb` the ONE spend path, the host-kind→tower-ordinal map). **The 631 half is UNPORTED** — see §3 R1 |
| 123 | MoonlordTurret 641 / RainbowCrystal 643 | `AI_123_FloatingTurrets` `:37051-37264` (gate `:34966-34969`) | `ai_floating_turrets` (`projectile.rs:9724`, dispatch `:19114`) | PRE-EXISTING — S1-D-4b-3 `a9f1e914b` (+ the 642 beam / 644 shard arms) | (the S1-D-4b ledger's) | the 641/643 rows `:6916-6940` (32×32, 36000, sentry, alpha 255); the `:69036-69043` mounted-centre hover anchor is the draw-side positioning arm; the disco hue rides the pool snapshot |
| 124 | the SEVEN pets 650/882/888/895/896/898/957 | inline `:34970-35696` | `ai_pets_style124` (`projectile_ai_pets124.rs`, dispatch `projectile.rs:17662`) | `69edd7fc4` + residuals `21a222d95` + the pointer wire `cf3a3ff20` + the grant fix `462c86097` + the retire `1e915e254` | **8** module + the pointer-quad pool witness | the seven rows ARE the arm's full kind census (`STYLE124_PETS`, `data.rs:1036`/`:3572-3575`); keep-alive pins `:34979-35006`, home ladder `:35044-35091`, flight `:35124-35325`, idle `:35326-35565`, the 957 ground bounce + `Next(150)` rest, the 650 ore scan + steering tail; the 650 pointer DRAW wired behind `draw_with_hand`'s generic push (`tentacle_pointer_quad`, `Lookups.extra_66`); ZERO AddLight/`this.light` — the PlotTileLine casts are the light-map-write layer's rows (research/336 §4) |
| 125 | WireKite 651 | inline `:35697-35833` | `ai_wire_kite` (`projectile_ai_wire_kite.rs`, dispatch `projectile.rs:17686`) | `3309b73ff` + the latch drain `dc8630b88` + the seal `1e915e254` | **6** module | the row `data.rs:3580` (10×10, pen −1, stock 3600); the two L-scans are C# NO-OPS (the `flag69` break never sets); the release latch `:35720` → the Kill-hook `(owner, ps, pe)` queue (`:72051-72067`) → `take_mass_wire_ops` → the app per-tick buffer; the APPLY is f11's (unmerged), the drain contract SEALED at the pass tail (research/336 §2/§6b, f11 research/235 §2 + 133 §8.4) |
| 126 | GeyserTrap 654 / OgreStomp 670 | inline `:35834-35997` | `ai_geyser_stomp` (`projectile_ai_geyser.rs`, dispatch `projectile.rs:17713`) | `fe72ebcc5` + the hostile-proxy fix `c41f61208` | **4** module + 2 pool/wiring | the rows `data.rs:3586`/`:3589` (30×30, alpha 255, 120, pen −1; the 654 friendly+hostile+trap, the 670 hostile-only); the anchor scan + the 16-step column growth (the 670's `320 − |localAI[1]|·64` cap), the tick-12 child split (`localAI[1] ±1`, the <3 cap); the `damage_player` full-box `Colliding` fix + the penetrate −1 survival (the EVP NO-kill tail `:14868-14899`); e20 owns the geyser TILE 443 wiring, the 654 shot chain is d13's |
| 127 | Sandnado 656 friendly / 657 hostile | inline `:35998-36085` | `ai_sandnado` (`projectile_ai_sandnado.rs`, dispatch `projectile.rs:17749`) | `e9be56fe6` + the Lerp fix `b23f8a618` | **4** module | the rows `data.rs:3594`/`:3598` (10×10, pen −1, 1200; the 656 magic + local 8, the 657 no pair); the 900/300 life caps, the Item82 cue latch, the 656 fade/owner-sight age jumps (`:36015-36023`), the vertical-expand box; the tail dust uses `MathHelper.Lerp` (the `b23f8a618` fix — `Utils.GetLerpValue`'s inverse ramp was WRONG) |
| 128 | SandnadoHostileMark 658 | inline `:36086-36197` | `ai_sandnado_mark` (`projectile_ai_sandnado_mark.rs`, dispatch `projectile.rs:17766`) | `6808d262e` + the Lerp fix `b23f8a618` | **5** module | the row `data.rs:3603` (14×14, pen 1 stock, 900, alpha 255, hostile); the first-tick tile-grid snap, the spin/fade/scale machine, the tick-60 hostile-657 spawn (the expert damage pick), the kill census; the `:36150` alpha==0 white·0.5 light is c19's site 28 — ACTIVATED by this row (research/336 §1) |
| 129 | SpiritFlame 659 | inline `:36198-36309` | `ai_spirit_flame` (`projectile.rs:8923`, dispatch `:18187`) | PRE-EXISTING — `c8dde2bc9` (the aimed-spells wave) | (that wave's) | the 659 parameters 3/7.5/1/420/240 override the 10/5/40/300/180 defaults (`:36200-36211`); the retarget scan + the orbit steering; the orbital dust (`:36253-36276`), the sound + the frame animation SKIPPED (the module doc's census) |
| 130 | DD2FlameBurstTower 663/665/667 | `AI_130_FlameBurstTower` `:68107-68360` (gate `:36310-36313`) | the dispatch-inline machine (`projectile.rs:18798`) | PRE-EXISTING — S1-D-4b-9 `331ac3b8d` (+ the aiStyle-131 shot) | (the S1-D-4b ledger's) | the rows `:7130-7151` (30×54, T2 28×58 / T3 28×60, 36000, sentry); the per-kind dust gates `Next(2)` + the two unconditional NextFloats + the smoke `Next(5)` ×1/2/3 per tier — the gate rolls run in C# order, the bodies skipped; the `:69145-69153` backpack reposition is the draw-side arm |

**Band verdict:** ten of ten styles censused; the five arms this lane
owns (124-128) are LANDED + WIRED with their rows, dispatch arms,
movement seams and dust censuses; the five pre-existing arms
(121/122/123/129/130) are re-verified present at HEAD with their own
ledgers' witnesses. ONE residual found by this bible's re-census: the
122 arm's 631 PhantasmArrow half (§3 R1) — a NAMED deferral with its
producer chain, not a silent gap.

## 2. The satellite lanes (landed on this branch, feeding the band)

| lane | C# anchor | port | commit | notes |
|------|-----------|------|--------|-------|
| the 813 BloodNautilusTears machine | `AI()`'s aiStyle-108 arm `:33679-34025` + the shared row `:6282-6293` | `ai_vortex_ritual`'s 813 extension + `BLOOD_NAUTILUS_TEARS` (the 578/579 shared row, stock pen 1, alpha 255, ignoreWater) | `21a222d95` + the wind-up scale fix `b56db9009` | 813 is aiStyle 108 NOT 124 (the brief's question resolved); the four-phase 13-site draw census on the UpdateProjectiles stream, the tick-90 Item8 cue + the BloodSquid 619 `wants_mark_spawn`; the wind-up holds scale 1.0 (the C# writes NO scale in that arm); the kill tick keeps drawing (`:33949` no return); the dust BODIES are g64c's (the activation contract table, ANALYSIS) |
| the 650 pointer draw | `DrawProjectile` `Main.cs:33674`/`:34007-34027` | `tentacle_pointer_pose` + `tentacle_pointer_quad` behind `draw_with_hand`'s generic push; `Extra_66` + `Lookups.extra_66` | `cf3a3ff20` (the pose decode `21a222d95`-era) | the generic sprite draws FIRST, the pointer BEHIND it; own sheet/origin/scale (`1 + floor(localAI[1])·0.02`), `proj.scale` rides the OFFSET only, SpriteEffects.None, the `GetAlpha` tint, `gfxOffY` on the anchor; the missing sheet skips the pointer arm |
| the geyser hostile proxy | `Damage_EVP` `:14773-14900` + `Wiring.GeyserTrap` `Wiring.cs:3084-3120` | `damage_player`'s `rects_intersect` box test + the penetrate>0 kill gate; the 654 hostile-proxy spawn | `c41f61208` | the 6-px centre probe → the full-box `Rectangle.Intersects` (`:15887-15890` strict edges); the unconditional post-Hurt kill dropped (the C# tail has NO generic kill); the finite-penetrate single-hit convention keeps its S1-C-7 witnesses; the end-to-end trip witness in `world_sim/wiring.rs` |
| the pet item-use 3600 grant | `ItemCheck_ApplyPetBuffs` `Player.cs:53091-53344` | `PET_ITEM_BUFF_GRANTS` (`data.rs:4637`) + the grant beside the i25 staff grant (`editing.rs:2779`) | `462c86097` + the classic-arms retire `1e915e254` | the buffTime=0 ghost fix: the 3577 arm + the 54-case vanity switch + the 26 CLASSIC single-item arms (the retire census); exclusions 603 (CE gate) / 425 (d2's fairy `Next(3)` lane); the g10 licenses ride `UsePetLicenses` NOT this fn; the keep-alives/eviction/FreeUpPetsAndMinions already ported |
| the kite latch drain | `Kill` `:72051-72067` | the death-hook queue + `take_mass_wire_ops` + `app.pending_mass_wire_ops` | `dc8630b88` + the seal `1e915e254` | the per-tick park (the C# applies synchronously inside Kill); the five-param union hunk pinned (research/336 §2/§6b); nothing calls the unmerged f11 fn |

## 3. The residuals (named owners, not silent gaps)

**R1 — the 122 arm's 631 PhantasmArrow half (this bible's re-census).**
The C# aiStyle-122 block handles TWO kinds: the 629 TowerDamageBolt
(ported, §1) and the 631 PhantasmArrow (`:6779-6792` row: 10×10, arrow,
friendly, ranged, tileCollide false, alpha 255, ignoreWater,
extraUpdates 1, timeLeft 1200; the `:34911-34964` homing/decay half:
the `ai[1]` hold countdown, the target-loss `tileCollide = true` +
`alpha += 10` fade-kill, the 12-px homing + `alpha -= 15` ramp, the
`velocity.ToRotation() − π/2` facing). The port has NO 631 row and NO
631 dispatch half (grep census at HEAD: zero `proj_data` 631 arms).
The PRODUCER chain is also unported, and was ALREADY a named deferral
before this lane: ANALYSIS `:27673-27675` (the S1-P-i17 special-arrows
note) — "The Phantasm's `phantasmTime` extra-arrow spawns
(`:13870-13878`) … are unported player flags — follow-ups." The full
chain: the Phantasm bow 3540 fires the 630 holder (`bow_launch.rs`'s
generic tail, landed — the row exists, `data.rs:3226`'s four-gunProj
arm); the 630's `AI_075` arm (`:67142-67240`, the 24−2·stage shot
interval, the `phantasmTime = 2` write `:67187`) is UNPORTED (the i22
OUT list — the f49 mana census names the 439/630-class holders as their
lanes'); the port's style-75 dispatch else currently runs the
`ai_last_prism` machine on a 630 slot (the fall-through — the C# 630 arm
is a DIFFERENT machine, the holder lane's fix); the on-hit triple-631
spawn (`:13870-13878`, the 0.3× damage, the ai1 0/15/30 stagger) rides
that flag. **Owner:
the Phantasm-holder lane (the i22 OUT list's AI_075 remainder), NOT
d13** — the 122 dispatch arm's 631 half lands WITH its producer (a row
without the producer is inert; the producer without the row desyncs).
The 631's Kill dust (`:72552-72563`, the `Next(5,10)` 229-burst) is the
g63 ≤400-axis-adjacent trail lane's census when the row lands; the
`:79064` GetAlpha white-fade is the draw layer's. No d13 change owed.

**R2 — the pets' PlotTileLine light casts** (`:35008-35043`): the ONE
standing light-map-write layer (the 870 FloodFillTile / c19 hook-rope
precedent). The rows are censused (research/336 §4): the 650
`(0.5,0.9,1)·2` self-20/owner-40, the 895 `(1,0.6,1)·1.5` ×1.5-self/
×0.75-owner + the `!NotRetro` ×0.8 gate, the 896 `(1,0.7,0.05)·1.5`
same ladder; the 882/888/898/957 cast NOTHING (dark in C# too). Owner:
the light-map-write task; no pet-side work remains before it.

**R3 — the kite apply drain**: the d13×f11×c42 union hunk (research/336
§2/§6b; f11 research/235 §2 + 133 §8.4). The buffer park is live; the
merger executes the direct drain (the buffer field drops). Owner: the
merge wave.

**R4 — the dust BODIES**: the 124 pets' 204/242/80 sparkles + the 813
four-phase 13 sites + the 126/127/128 column dusts carry their draw
censuses on the stream; the bodies are g64c's emitters (the 642/509
draws-kept-skip replacement pattern — REPLACE each census block, never
add beside it). Owner: g64c (the activation contract, ANALYSIS (2)).

**R5 — the equip-arm pet grants** (`UpdatePet`/`UpdatePetLight`
`Player.cs:17298-17340`): the i53 note's seam, UNCHANGED by the use-arm
fix. The CE lane (603) + the fairy-bell 425 `Next(3)` pick are named
exclusions (d2's light-pets lane). Owner: the equip lane / d2.

## 4. The SEAL receipts (the five brief items, consumed tips)

| # | item | consumed tip | verdict | receipt |
|---|------|--------------|---------|---------|
| 1 | the pet-item-use 3600 grant gap — activation now that g10/d43 landed? | g10 `5dee28650` (the 11 town pets), d43 `0253bde90` (63 kinds incl. the pals 695/696) | **NO consumption — different lanes.** The licenses 4829/4830/4910 ride `ItemCheck_UsePetLicenses` (`Player.cs:45352-45366` → `NPC.UnlockOrExchangePet` `NPC.cs:80894-80918`): the boughtCat/Dog/Bunny flags + the chat + the variation reroll, NO buff/projectile (the item rows buff_type 0, projectile None — registry-verified). The town slimes 678-684 unlock through the bound-slime finds (NPC-side). The d43 pals spawn wild (`NPC.cs:4371-4384`, the HasItem(5663/5664) + RollLuck(100)<40 pick); their rescue staves 5663/5664 (`AI_127_Pal_GiveRewerd` `:43026-43034`) are the i25 `MINION_STAFF_PET_BUFF_ITEMS` rows (ALREADY granted); the Chillets 5665/5666 are mounts 62/63. The license USE-arm itself is unported on every consumed tip — g10/f43's town-lane residual, NOT a grant gap. **One NEW gap found + fixed instead:** the CLASSIC single-item arms (26 items, the same buffTime=0 ghost) joined `PET_ITEM_BUFF_GRANTS` (`1e915e254`) | research/336 §3 + §6; the 2 retire witnesses (the Shadow Orb use, the classic-row/exclusion census) |
| 2 | the c19 67-site PET-light census compose | c19 `472ef867e` (round 3 FINAL: 58 ported / 64 contributions / 7 absent; the round-2 tip `a160ad9ff` also read), e6 `c8b072b69` | **CONFIRMED from both sides.** Zero pet kinds in ANY column at BOTH c19 tips (the four sim-armed rand-gated kinds 409/590/849/912 are not pets); the 107-site AddLight family closed with ZERO pet contributions; the `ProjLight` `light>0` gate never fires (no `this.light` in the seven rows); d13's 658 row ACTIVATED c19's site 28 (`:36150`); the pets' PlotTileLine casts are the SAME light-map-write deferral c19 records for the hooks. NO c19/e6 gap | research/336 §1 (the round-3 refresh paragraph) + §4 the family declaration |
| 3 | the f11 wire-kite activation | f11 `14ca0a8d5` (the retire; research/235 §2 renumbered 136→153→235, research/133 §8.4) | **SEALED.** The stream note CLOSED on f11 (`84ed6342b`: `mass_wire_operation` gained `drop_rng: RngKey`); d13's drain-contract doc trio + the composition witness re-pinned to the five-param shape with `RngKey::UpdateProjectiles`; the RAW ToolMode (no Actuator strip — that mask is the 3625 arm's `Player.cs:46908`), no cooldown gate, the owner==local_id gate, the tile-coord compose (`(int)X >> 4`, `Utils.cs:1807-1810`); the ONE-drain-shape rule (direct OR buffer, never both). The §6b re-target acknowledged: the union executes against c42's moved fn (`world_sim/wiring.rs:1089`, the 133 §8.4(a) param diff), NOT f11's `wiring_devices` module (drops per 133 §4a). f11 NOT merged; nothing calls the unmerged fn | research/336 §2 + §6b; the kite witness's stream pin (`drop_rng.name() == "UpdateProjectiles"`) |
| 4 | the 895/898/957 g64 dup — keep-d13-superset | g64 `3cdc64236` (the FINAL union-table; the dup adjudication `c82a17e19`) | **CONFIRMED byte-equal at the current tips.** The row tuples byte-equal (`(20.0, 20.0, 124, -1, 18000, 0, false, false, false, RESET_STATE)` — d13 `data.rs:3572-3575` seven-kind arm vs g64 `:3584-3586` three-kind arm); the flag lists value-identical on the shared kinds (d13's `STYLE124_PETS.contains` ⊇ g64's three-kind matches); the frame counts verbatim (895→6, 898→16, 957→12); `LIGHT_PET_PROJ` byte-equal. The machine differs by DESIGN (g64's dust-wired 7-arg vs d13's 5-arg census shape). **Merge keep-one: d13's seven-kind rows/arm (the superset); g64's dust-wired machine body folds INTO it** (g64's own FINAL ruling). No d13 change owed | research/336 §6a (the four-row byte-equality table) |
| 5 | the research FINAL (registry-number) + this band bible | d54 registry-150 round 3 `51a28d09e` → §9 UNIFIED FINAL `b243c7086` | **EXECUTED, then re-numbered per §9.2.** The SEAL round renumbered the retire FINAL 243→304 (g58's `243-seal-verification.md` `c9af393dd` 14:08:18 predates this lane's 243 push 15:20:29; the registry confirms 243 = g58 winner; the fresh all-tip + full-history sweep found zero 304/305 then); ALL refs updated (the doc + header, anchors ×2, ANALYSIS ×2, PORT_MAP ×2, the .rs doc comments ×5, the INDEX row). §9.2 UNIFIED FINAL supersedes both picks: the retire FINAL chains 243→304→**336** (304=d50 §7.4), this bible chains 305→**353** (f4 KEEPS 305, §7.4.3); ALL refs re-pointed in the §9 rename commit | research/336's header + §5/§6c; this doc's header |

## 5. The witness ledger (WRITTEN NOT RUN — code-only mode)

Counted from the landed code at HEAD (the `w4_d13_*` family, **40**
total; `git grep -c "fn w4_d13_"` per file):

- `projectile_ai_pets124.rs`: **8** — the seven-row SetDefaults census,
  the keep-alive pet-flag pins, the far-pet teleport-home ladder, the
  650 per-tile scan + ore point, the idle facing/rest pose, the 957
  grounded/airborne probe split, the pointer-pose decode, the
  steering-encode round-trip.
- `projectile_ai_wire_kite.rs`: **6** — the row census, the first-move
  tile-anchor latch, the three kill gates, the owner-velocity tail
  facing, the release-latch queue, the f11 shape-composition (SEALED:
  the five-param + the `UpdateProjectiles` stream pin).
- `projectile_ai_geyser.rs`: **4** — the two-row census, the state-0
  free-air anchor, the tick-12 stomp split, the child-cap shrink.
- `projectile_ai_sandnado.rs`: **4** — the two-row census, the cue
  latch + age ticks, the 656 fade age-jump, the four-float tail dust.
- `projectile_ai_sandnado_mark.rs`: **5** — the row census, the
  first-tick grid snap, the tick-60 expert-damage spawn, the 120-tick
  death, the kill gate/scale census.
- `projectile.rs`: **7** — the geyser box-hit + survival, the pointer
  quad pivot/scale, the 813 row share, the wind-up two-gate rings
  (scale 1.0 re-pin), the tick-90 BloodSquid spawn, the full-charge
  one-ring pick, the dissolve switch-pair + drip.
- `world_sim/editing.rs`: **5** — the 5089 Glommer use → 3600 → 18000 +
  the 957 spawn, the 3577 tentacle use → eviction + 650 flag, the
  grant-row/registry agreement (the 5117 exclusion), the Shadow Orb
  use → eviction + 19@3600 → 18000 + the 18 flag (retire), the
  classic-row/exclusion census (retire: 603/425/the licenses/the
  Chillets out, the pal staves on the i25 table).
- `world_sim/wiring.rs`: **1** — the geyser trip end-to-end (the
  hostile-column spawn).

The pre-existing arms' witnesses (121/122/123/129/130) ride their own
ledgers (i25/i56, B-6, S1-D-4b-3/-9, the aimed-spells wave) — not
re-counted here. NO new witness this bible round (docs-only); the
retire round's 2 new witnesses are in the ledger above.

## 6. Verification record

- Every C# anchor re-read in this worktree's `decompiled/Terraria/` at
  the cited lines (≤100-line windows); the consumed tips read via
  `git show origin/task/W4-…:<file>` (g10 `5dee28650`, d43 `0253bde90`,
  c19 `472ef867e`/`a160ad9ff`, e6 `c8b072b69`, f11 `14ca0a8d5`, c42
  `2910a7209`, g64 `3cdc64236`, d54 registry round 3 `51a28d09e`).
- The 631 residual: the zero-row grep census at HEAD + the
  full-history `-S "phantasmTime"` / `-S "631"` searches (the producer
  chain unported on every pushed branch; the S1-P-i17 ANALYSIS note
  `:27673-27675` is the standing deferral record).
- Number freeness: the fresh all-tip `ls-tree` + full-history
  `--name-only` sweep (zero 304/305 anywhere at the SEAL round —
  CHAIN-HISTORY); the registry-150 round-3 rule consumed then (304+ free
  at that round, no max+1 picks); §9 UNIFIED FINAL `b243c7086` supersedes
  with the FINAL assignments 336/353 (see the header chains).
- Code-only mandate honoured: **no cargo/tests/rustfmt/graphify
  invoked**; the witnesses stand WRITTEN NOT RUN. No merge, no primary
  checkout touched; publication on `task/W4-d13-proj-ai-121-130` only.
