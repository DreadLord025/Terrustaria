# 149 — W4-c10 waterfowl band FINAL (aiStyle-68 + the walkers + the 1080 drop)

> **Renumber note (retire phase, 2026-11-21):** this doc was first pushed
> as `145-waterfowl-band-final.md` (`5d8387955`, 10:12:03 +02:00). The
> full origin scan (every `task/W4-*` tip's `docs/research/` tree) found
> key 145 THREE-way contested: W4-g89's
> `145-rod-teleport-union-composition.md` first-add `41dabfbc8`
> 09:48:08 wins per the d54 registry's first-pushed rule; W4-c4's
> `145-d42-keep-d42-verdicts-addendum.md` (10:12:10) is the other loser.
> 146 is NOT free (g24 `146-wg-sounds-final.md` + e20
> `146-c42-repick-critical.md`), 147 e20, 148 c4/e20, 150 = the d54
> collision registry (`150-research-number-registry.md` on
> `origin/task/W4-d54-dust-sites-world`), 151–236 reserved by that
> registry's rename block. **149 is the lowest number taken by NO tip** —
> this doc takes it. Body content-preserving; refs updated: INDEX row,
> ANALYSIS Item 5, PORT_MAP row, ROADMAP aiStyle-68 row, anchors row 906.

Branch: `task/W4-c10-style68-waterfowl`, final phase 2026-11-21,
**code-only mode** (no cargo/rustc/rustfmt/tests invoked; every witness
WRITTEN NOT RUN). Base lane: S1-D-3's `npc_ai_style68.rs` module
(`task/S1-D-3-aistyle68`, 16 witnesses). This report is the band's
closing ledger: what landed, the cross-lane contracts, the witness
index, the merge checklist. ANALYSIS "The waterfowl FINAL (W4-c10)"
carries the same verdicts with the full prose.

## 1. Band-complete declaration

**The aiStyle-68 waterfowl band is COMPLETE on this branch.** All
eight kinds (walkers Duck 362 / DuckWhite 364 / Seagull 602 / Grebe 608
aiStyle 7; swimmers Duck2 363 / DuckWhite2 365 / Seagull2 603 /
Grebe2 609 aiStyle 68) are rostered, wired, drawn, sounded and
transformable; their projectile (BirdDroppings 1080) is a full
AI+movement+death module. Nothing in the C#'s style-68 block
(`NPC.cs:34623-34879`), the walkers' takeoff gate (`:53180-53184`) or
the drop's three sites (`Projectile.cs:39985-40017` / `:18921-18928` /
`:70926-70954`) is left unported beyond the standing dust/gore-body
visual skip (draw censuses kept).

| piece | commit | C# anchor |
| --- | --- | --- |
| 8 kinds + SetDefaults/FindFrame/sheets/bestiary | `4d3ff5fc3` | `NPC.cs:13621-13658`/`:16745-16828`, `:73747-73886` |
| axis claim + `ai_068_waterfowl` adapter + 1080 module | `98a99fb1c` | `:34623-34879`, `Projectile.cs:39985-40017` |
| portrait/bestiary parity fix | `c48bbb1ee` | the exhaustive `portrait_data` + BESTIARY_NPC_SHEET_IDS 181→189 |
| 1080 Kill hook (Stinky/Item177/90-draw) + corpse drift | `8bd6afeb3` | `Projectile.cs:70926-70954`, `:20445`/`:17027` |
| keep-one verdict vs d20 | `4c2e2b707` | the nine-row arm-by-arm table (ANALYSIS) |
| TransformVisuals census module + waterfowl silence | `67191299b` | `NPC.cs:80963-81033` |
| sound census + the case-45 Dolphin fix | `9641e20db` | `:92243-92341`, `:23678-23681` |
| town-arm takeoff gate + the sim drain targets | `6a65deeb8` | `:53180-53184` (the b3 seam) |
| OTHER-kind flip sites wired + activation contract | `1e336ee93` | `:39398`/`:56865`/`:56897` → `:81136` |

(The roster-tail co-lane's `d7f422049` twelve-critter tree rides the
same branch between the sweep and this final — a separate band, its
own ANALYSIS section; it absorbed the worktree fixes for the sweep's
three mis-placed hunks, so HEAD is structurally sound.)

## 2. Cross-lane contracts (all published, none blocking)

1. **W4-b3 (town migration)** — their axis-7 claim
   (`96a5d88f`) routes the walkers through `town_npc_ai`; the takeoff
   gate is ALREADY inside that module (this lane landed it at the C#
   position). Their phase-2 merge line: `let _ = out` becomes
   `if let Some(ty) = out.waterfowl_takeoff { self.wants_transform = Some(ty); }`;
   `waterfowl_walker_step`'s in-place gate retires then. Both paths run
   the identical `transform_to_kind` — behaviour-preserving either way.
2. **W4-d20 (projectile 191-200)** — keep-one AGREED both directions
   (their `e4eeae9b7` research/109 §Item 3 read this lane's
   `4c2e2b707` verdict): at merge keep `projectile_bird_droppings.rs`
   + c10's dispatch/row/const; drop d20's `ai_bird_droppings` fn
   (`projectile_ai_misc.rs:711-761`), its `200 =>` arm
   (`projectile.rs:17381-17389`), its duplicate row/overlay/const and
   its 4 witnesses. A double `200 =>` arm is a compile error — the
   dedupe is mandatory, not optional.
3. **W4-d36 / g8 (vampire 158↔159)** — the TransformVisuals vampire
   arm (Item8 + 6 gore-kick draws) is module-ready; 159 ports on d36
   (proto 762+ per the `bcf4ac551` supersede-note), 158 skipped to g8
   (108-E2). Their flip callers (`NPC.cs:23353` aiStyle-14 /
   `:58850` aiStyle-3) call `transform_visuals(old, new, w, h, rng)`
   and queue the returned Item8 on `wants_sound`.
4. **W4-d40 (mothron 478→479)** — both kinds rostered on `8fe1d6d31`;
   the aiStyle-89 egg timer's `Transform(479)` (`NPC.cs:38380`) is
   their wiring step. Flip call: `transform_visuals(478, 479, 46, 30,
   rng)` — 219 draws, no cue.
5. **The roster-tail co-lane (this branch)** — FINAL STATUS at the
   retire phase (was LIVE at the final phase). `d7f422049` ("phase 5
   batch 1/4") carries the FULL twelve-kind tree per its ATOMICITY
   NOTE; batches 2-4 never committed on any ref (`git log --all
   --grep='batch [234]/4'` empty) — the census + doc slices live in
   ANALYSIS "The bug-net roster tail (W4-c10 phase 5)" and the
   co-lane's ROADMAP row (left UNSTAGED in the worktree, foreign hunk,
   never committed by this lane). The twelve: **Worm 357, GoldBird
   442, GoldBunny 443, GoldButterfly 444, GoldFrog 445,
   GoldGrasshopper 446, GoldMouse 447, GoldWorm 448,
   EnchantedNightcrawler 484, Grubby 485, Sluggy 486, Buggy 487**
   (vanilla protocol ids; the brief's "582" corrected to 484 — 582 is
   LarvaeAntlion). **ZERO overlap with the waterfowl eight** (362-365 /
   602-603 / 608-609) — the two bands share only the style-24 poop
   gate (GoldBird joins the walkers on `ai_024_bird`, the shared gate
   reused UNMODIFIED).

   **Dup-with-OTHER-lanes (the d37 E8 read, `a341daa12` 10:02:03 —
   written against c10's PRE-tail tip `9641e20db`, its c10 row still
   says "no new kinds"):** eleven of the twelve are ALSO rostered on
   `origin/task/W4-d40-npc-kinds-421-490` (`8fe1d6d3`, band 430-488:
   the gold septet 442-448 + the bait tail 484-487 — d40's
   `npc/data.rs` to_u16 arms verified identical ids) and Worm 357 on
   `origin/task/W4-c8-style66-crawling-critters`. d37's E8 already
   drops c8's GoldWorm/EnchantedNightcrawler/Grubby/Sluggy/Buggy/Worm
   copies to d40@448/484/485/486/487 + c8 Worm→737; the c10 tail is a
   THIRD copy of the same eleven + Worm. **KEEP-ONE (this lane's
   verdict, E8 ownership rule applied):** d40 is the kinds-lane owner
   of 442-448/484-487 (a kinds-lane owns over a wiring lane) — at
   merge DROP the c10 tail's duplicate `NpcKind` variants +
   to_u16/from_u16/ALL_KINDS/ENTRIES/stat rows for those eleven, KEEP
   d40's rows; the c10 tail's WIRING survives keyed on the canonical
   kinds (E3's rule): the `24 => ai_024_bird` / `65 => ai_065_butterfly`
   / `66 => ai_066_crawler` axis claims (d40 has NO axis claims — its
   tip's npc.rs has no `24 =>`/`65 =>`/`66 =>` arms), the d38
   `npc_ai_critter.rs` TownCritter subset + gold walk rows, the d30
   `npc_ai_style65.rs` module, the five sim-stamped Npc fields + the
   pre-stamps/light drain, the `critter_sound_cue` cs_id key fix.
   Bestiary/sheet/portrait rows dedupe to ONE copy per kind (both
   lanes carry them; keep the canonical owner's ENTRIES, this lane's
   portrait arms are SetDefaults-box-verified). Worm 357: E8 moves
   c8's copy to 737 — the c10 tail's Worm row drops with it (one Worm
   port survives at 737, the c8/c10 style-66 wiring shared). The
   co-lane's uncommitted ROADMAP row stays the co-lane's to commit;
   this verdict is published here + in the co-lane cross-note below.
6. **W4-d76 (proj-sounds-2) — the 1080 Kill superset, FINAL keep-one**
   (retire phase). `origin/task/W4-d76-proj-sounds-2` is LIVE, NOT in
   main (`ede24bb4f`/`dd90f1629` both non-ancestors of origin/main).
   Its `dd90f1629` ports `kill_tail_cue` — the four-site cue-only map
   (`Projectile.cs:70865` Jim's Drone/RC Car Item62, `:70904` Pretty
   Mirror Item106, `:70936` **BirdDroppings Item177**, `:70958` Torch
   God helper Item10) + five census rows (1020/1105/1085/1080/1084)
   + the `BIRD_DROPPINGS` const + UNARMED_ROWED +195/200/202/205.
   **The c10 hook (`8bd6afeb3`) is the STRICT SUPERSET of d76's 1080
   arm:** Stinky 120/600 on the living-local-player rect intersect
   (`:70928-70934`, d76 explicitly defers this as "the state
   follow-up"), the SAME Item177 cue at the int-truncated centre
   (`:70936`; d76 queues untruncated and relies on the id-2 façade —
   c10 truncates at the hook, identical drain), PLUS the 90-draw
   30-dust census (`:70937-70951`, d76 skips the bodies AND the draws
   — a stream deviation the superset closes) and the
   died_out_of_world skip. **KEEP-ONE at merge:** keep c10's
   `projectile_bird_droppings.rs` + the pool hook + the c10 row/const;
   from d76 keep `kill_tail_cue` for the OTHER four kinds
   (1020/1105/1085/1084) but **DROP its `BIRD_DROPPINGS =>` arm** —
   both hooks fire on the same death seam, so leaving the arm queues
   Item177 TWICE; drop d76's duplicate 1080 row + `BIRD_DROPPINGS`
   const (identical values, one definition survives — E0428/E0592
   otherwise) + its UNARMED_ROWED `200` entry (c10's style-200 is
   ARMED by the module; the census pin would misreport) + the 1080
   case row in its `s1_e_21h_d76_sweep_the_kill_tail_cue_map`
   witness (the c10 pool-hook witness covers the cue). The other
   four kinds' rows/arms/witness stay untouched — zero c10 overlap.
   This composes with §2.2: d20's `ai_bird_droppings` ALSO drops; the
   ONE surviving 1080 machine is c10's module + hook. Cross-note
   published in ANALYSIS "The waterfowl FINAL" Item 6.

## 3. Witness index (37, all WRITTEN NOT RUN)

Counted from each commit's added `#[test]` lines (phase 1: 1, phase 2:
6, phase 3: 1, kill-hook: 2, visuals: 5, ambient: 2, town-gate: 1,
final flips: 3, plus the S1-D-3 module's 16).

| file | count | names |
| --- | --- | --- |
| `npc_ai_style68.rs` | 16 | the S1-D-3 module suite (poop gate, perch, wet swim, water-line hover, takeoff, flutter, freeze, landing+Next(200), bounces, steering, ground probe, hover, ten-tick replay) |
| `npc.rs` (wiring) | 3 | `w4_c10_the_waterfowl_defaults_match_the_csharp_rows` (phase 1), `w4_c10_the_axis_claims_style_68_for_the_four_swimmers`, `w4_c10_the_dry_landing_flips_the_kind_to_the_walker` (phase 2) |
| `ui/bestiary.rs` | 1 | `w4_c10_the_waterfowl_portraits_rest_their_set_defaults_boxes` (phase 3) |
| `projectile_bird_droppings.rs` | 6 | `the_1080_row_matches_set_defaults`, `the_drop_falls_and_censuses_two_dust_rolls`, `the_drop_kills_on_the_player_rect`, `the_drop_splats_on_a_blocked_step` (phase 2) + the kill 90-draw order + the pool-hook arms (sweep item 2) |
| `npc_transform_visuals.rs` | 5 | eight-pair silence, vampire cue+kicks, mothron 219 order, stardust drawn-count replay, the two vortex rings |
| `ambient_sound.rs` | 2 | `w4_c10_the_waterfowl_ambient_rows_census`, `w4_c10_the_quack_and_cry_voices_draw_in_order` |
| `npc_ai_town.rs` | 1 | `w4_c10_the_waterfowl_takeoff_gate_emits_the_transform` (wet/fast-fall/fast-rise/strict-4/dry-slow/client/zero-draw/pre-state × 4 walkers) |
| `npc.rs` (final flips) | 3 | `w4_c10_final_the_stardust_growth_flip_draws_the_ring`, `w4_c10_final_the_larva_flip_draws_the_160_ring`, `w4_c10_final_the_hornet_flip_draws_the_240_ring` |

## 4. Merge checklist (the order the contracts resolve)

1. Run the catalog command on the merged tree
   (`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`)
   — the 37 witnesses have NEVER been compiled; expect first-compile
   mechanical fixes (the code-only convention).
2. d20 dedupe FIRST (a double `200 =>` arm is E0004-adjacent hard
   error): keep c10's module/dispatch/row, drop d20's five pieces (§2.2).
   THEN the d76 dedupe (§2.6): drop its `kill_tail_cue` BIRD_DROPPINGS
   arm (double Item177 otherwise), its duplicate 1080 row/const, its
   UNARMED_ROWED `200` entry + its witness's 1080 case; keep its other
   four kinds whole.
3. b3's drain line (§2.1) — one line, then retire the in-place gate.
4. The anchors canary: five c10 rows (`ai_068_waterfowl`,
   `bird_droppings_step`, `bird_droppings_kill_draws`,
   `transform_visuals`, `waterfowl_takeoff`) must survive the merge.
5. d36/g8/d40 activation (§2.3-4) lands on THEIR branches — no c10
   action; the module's `allow(dead_code)` narrows away as they wire.

## 5. Deviations (the standing conventions, all in ANALYSIS)

- Dust/gore BODIES unmodelled everywhere; the DRAW CENSUSES keep every
  `Main.rand` draw in C# order (the stream is the invariant).
- `netOffset` brackets fold to no-ops (single-player host).
- The 226-dust stardust growth ring (`NPC.cs:39403-39427`) and the
  larva's per-tick 229 dust (`:56869-56881`) stay the pillar lanes'
  documented skips — this lane wired only the TRANSFORM census.
- The walker walk rides the `update_guide` town stand-in until b3's
  axis claim (the published contract).
- The grebes 608/609 are AMBIENT-SILENT by census (no case row) — not
  a gap.
