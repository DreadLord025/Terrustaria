# 281 — W4-d30 RETIRE/SEAL: the NPC aiStyle band 65–72 FINAL census + keep-one bible

Branch `task/W4-d30-npc-ai-styles-65-72`, code tip `f23b43fde1acdd3b4fd612837e74e4faa6eb8f4b`
(base `a725acb04`, wave-3 complete). Scan date 2026-11-21, fetched
`origin/main = 051600da81339c7cb026b9a8190f71cbb47f738a`. Code-only lane: no
cargo/tests/rustfmt/graphify; witnesses WRITTEN NOT RUN. Registry number **281**
per research/150 §4-REFRESH ("Numbers 281+ are FREE"; tree census across all
origin tips: max used 245, 244–280 are §6 loser assignments; no tree or
commit-message claim on 281).

## 1. Git-log census — what landed in the band

Ancestry checked with `git merge-base --is-ancestor <sha> origin/main`.

| Style | Machine (file · commit · lane) | In main? | Wiring (commit · lane) | In main? |
| --- | --- | --- | --- | --- |
| 65 | `npc_ai_style65.rs::ai_065_butterflies` · `f23b43fde` · **d30 (this)** | NO | `d7f422049`/`5fcc68ae6` c10 (444 only); `a832d38ea` f18 + `a0d49dc1d` d39 (356, subset rewrites); `7a0482302` d125 (661 via own `npc_ai_lacewing.rs`) | NO |
| 66 | `npc_ai_style66.rs` · `33cfdb520` S1-D-3 (via L-J `15b8e9527`, fixes `cc12d2f5c`/`aa147ce08`) | YES (UNWIRED) | `3fadd5f14` + `42ffa7762` + sweeps · c8, retire `7c2de9c06` | NO |
| 67 | `npc_ai_style67.rs` · `12b022cc1` S1-D-3 | YES (UNWIRED) | `f5fbc2b3c` + `2042d77f1` · c9 | NO |
| 68 | `npc_ai_style68.rs` · `a62758fa2` S1-D-3 (via `5e92bfb87`, fixes `8eca3f9ef`/`aa147ce08`) | YES (UNWIRED) | `98a99fb1c` + `c48bbb1ee` + `5d8387955` · c10 | NO |
| 69 | `npc_ai_fishron.rs::fishron_ai_step` · `0cb14db09` W-9 | YES | same commit: `npc.rs:4897 69 => ai_069_fishron` | YES |
| 70 | `npc_ai_style70.rs` · `31e5fecb0`→`0cb14db09` | YES | `npc.rs:4901 70 => ai_070_bubble` | YES |
| 71 | `npc_ai_style71.rs` · `a991a15d9`→`0cb14db09` | YES | `npc.rs:4905 71 => ai_071_sharkron` | YES |
| 72 | `npc_ai_style72.rs` · `e2de964e2` S1-D-3 | YES | `29be1002f` wire-smallkinds: host pre-pass + `npc.rs:4735` movement tail | YES |

Axis note: W4-d8's `72/73/74/76` (`e66b41b00`/`7953d1513`, Bubble 410 /
fountains 419–422 / Scutlix laser 434) are **projectile** aiStyles — a different
dispatch, zero overlap with this NPC band.

## 2. All-styles-armed / kinds-rowed census + remaining gaps

**Armed on origin/main:** 69/70/71/72 fully (machine + axis arm + kinds
370/371/372/373 at `npc.rs:18584-18587`, 384 at `:18387`). 66/67/68 modules
present but UNWIRED (no axis arm; the S1-D-3 L-J recovery precedent). 65 absent
entirely (no module, no arm, no kind).

**Kinds rowed (by lane tip, NOT main unless noted):**
- 65 (four-kind C# census, `NPC.cs:13535/:14279/:17254/:17355`): 356 Butterfly —
  f18 `a832d38ea` + d39 `a0d49dc1d`; 444 GoldButterfly — c10 `5fcc68ae6`
  (`npc.rs:19610`, catch 2891 via the gold range `NPC.cs:14287`, re-verified by
  direct read `:14275-14289`); 653 HellButterfly + 661 EmpressButterfly — d43
  `0253bde90` (`:20650/:20658`, FindFrame `{}` stubs, NO machine/axis);
  661 ALSO rowed by d125 `7a0482302` as `PrismaticLacewing` (`:18940`, wired).
- 66: all nine 357/374/375/448/484–487/606 rowed + wired on c8 `7c2de9c06`
  (TruffleWorm 374 at `npc.rs:1099` — the earlier "374 unported" note in
  `d7f422049` is SUPERSEDED by the c8 tip evidence).
- 67: 359/360/655/677 on c9 `2042d77f1` (+ the 131-id ShimmerImmunity write).
- 68: the eight waterfowl + walkers on c10 `5fcc68ae6` (band-COMPLETE per its
  research/145→149→270 doc).
- 69–72: on main (above). FindFrame band: f26 `b89812bf7` rows 653 (butterfly
  group, no colour bank) + 655 (delegates to `npc_ai_style67::snail_find_frame`);
  c10 rows 444's flap case; d125 rows 661's case (`npc.rs:6543`).

**Remaining gaps (all merge-time, none unclaimed):**
1. Styles 65/66/67/68 wiring absent from main — the c8/c9/c10/d30-family lanes
   carry it; the merger composes per §3.
2. Style 65 has FOUR machine copies (§3 keep-one) — the only real dedupe work.
3. 661 dual kind-name (`EmpressButterfly` d43 vs `PrismaticLacewing` d125) —
   one NpcKind survives; d125's is the wired one; roster numbering is the d37
   bible's call.
4. No gap in 69–72: DONE on main, nothing owed.

## 3. Cross-lane keep-one receipts

**(a) d31/d32 adjacency — DISJOINT, verified at tips.**
d31 `0487ea324ecfd913bfc64449626c563d204b408b`: claims 76–82 + ports 83
(cultist ritual); d32 `ac1d38e70fe4d3a4f62acc9700b5baea42522797`: ports 86–92 +
claims 93. Both grep ZERO for style65/butterfly kinds 356/444/653/661 arms;
their only "style72" hits are precedent comments citing main's landed module.
No keep-one owed either direction.

**(b) style-65 machine keep-one — d30 CANONICAL.**
- c10 `5fcc68ae6` copy: `git diff f23b43fde 5fcc68ae6 -- npc_ai_style65.rs` =
  9+/6− **header-comment only** (UNWIRED→WIRED note); machine body
  byte-identical. KEEP d30 module + c10's adapter (`npc.rs:5162` axis arm,
  `ai_065_butterfly` `:6117`, threat-view pre-stamps, FindFrame 444).
- f18 `a832d38ea` / d39 `a0d49dc1d` rewrites (`ai_065_step`, ~350-line bodies):
  strict functional SUBSETS — no 661 rainbow light/fade/despawn/267-dust head,
  no 653 netOffset/ember/type-6 tail, fixed 15-tick flee (661's 10 lost), no
  real DustPool. DROP both module copies at merge; their 356 kind rows +
  wiring SURVIVE re-pointed onto `ai_065_butterflies` via the c10 adapter
  shape. (d39's `024c9877f` is a verbatim-preservation checkpoint of the f18
  wave — same drop.)
- d125 `7a0482302` `npc_ai_lacewing.rs` (549 ln, 661-only): carries NO copy of
  the d30 module (diff = whole-file absence). Its documented deviations
  (AddLight skipped, dust visuals skipped/draws consumed, session-player
  target fold) are subset-fidelity vs d30's full arm (real light effect, real
  DustPool with post-writes, HSL byte recovery). KEEP-ONE: d30 machine owns
  the 661 path; d125's WIRING survives as the sim adapter — the slot-turn
  threat-slice pre-pass (`world_sim/npcs.rs:4483-4525`, the no-double-step
  `lacewing_handled` gate), the `65 if kind == PrismaticLacewing` axis guard
  (`npc.rs:4969`), despawn→`encourage_despawn(0)`, and the `DoDeathEvents`
  case-661 → SpawnBoss(636) drain (`:5432+`) — re-pointed from `lacewing_step`
  onto `ai_065_butterflies` (its `Ai65Effects.despawn/light/opacity` map 1:1
  onto `LacewingOut.deactivate` + the light/opacity feeds). d125's
  `npc_ai_lacewing.rs` DROPS with its module; its witnesses re-target.

**(c) g65-Kill-309 — DISJOINT (id-space disambiguation).**
g65 `f6e07bb90c65c0674124d54c5a472629b577737a`: the "309" arm
(`projectile_death_dust.rs:1100`, batch 9 `1056c6388`) is **projectile kind
309 = FrostBlastFriendly** (`projectile/data.rs:4326`, Kill
`Projectile.cs:75347-75361`: 10 dust-185, per-dust Next(2) → vel×2/noGravity/
scale×1.75). The final census `b0842907c`'s "309 distinct kind arms" is a
COUNT of arms, not kind 309. NPC 309 is Scarecrow5 (`lang_tables.rs:386`) —
a third id space. The d30 module's dust is NPC-AI-side types 267 (661 shimmer)
and 6 (653 ember), never a projectile Kill arm. Zero shared bodies; no
keep-one, no drop owed either direction.

**(d) d77c-cues — DISJOINT, d30 owes no cue drop.**
d77 `5f10560b2c9621a33363c0159560c9125a658d93`: `kill_tail_cue`
(`projectile.rs:20211`) owns every projectile Kill cue per the d78 SEAL
(`7ea64ff75` §6c). Its `661` arm (`:20250` region) is **projectile 661 Onyx
Black Bolt** (centre−80), NOT NPC 661 Lacewing — the id collision is recorded
here so no future census folds them. The d30 module pushes ZERO sound cues
(grep cue/sound over `npc_ai_style65.rs` = no hits); the butterfly NPCDeath1
(`NPC.cs:14284`) rides the generic NPC death-sound path, not d77's map.
The g3-cue-position-fix and the "g3 census calls stay until g65 bodies push"
notes are projectile-lane business; d30 is neither g3 nor a Kill-dust lane.

## 4. FINAL declaration

The NPC aiStyle band 65–72 is **CLAIM-COMPLETE**: every style has a faithful
machine on a pushed branch; 69/70/71/72 are landed AND wired on main; 66/67/68
are landed-unwired on main with complete wiring on c8/c9/c10; 65 is the d30
module (canonical, four-kind, real-pool) with wiring distributed across
c10 (444), f18/d39 (356, to be re-pointed), d43 (653/661 rows) and d125 (661
sim/death wiring, to be re-pointed). The ONLY merge work is the §3 keep-one
table: drop the f18/d39 `ai_065_step` copies and d125's `npc_ai_lacewing.rs`,
keep d30's `ai_065_butterflies` as the single style-65 body, keep the sibling
adapters/kind rows, resolve the 661 kind-name duplicates to one NpcKind. No
style, kind, dust site or cue in the band is unclaimed. d30 owes nothing
further; the follow-up BUILD lane owns witness execution (15 module witnesses
+ 3 seal witnesses below, all WRITTEN NOT RUN) and the merge fix-lists.

**Seal witnesses** (in `npc_ai_style65.rs::tests`, WRITTEN NOT RUN):
`w4_d30_seal_the_gold_butterfly_catch_row_rides_the_gold_range` (444→2891,
`NPC.cs:14287` via `bug_net.rs:152`),
`w4_d30_seal_the_hell_butterfly_is_lava_bait_the_gold_is_not`
(`is_lava_bait` split 4845 vs 2891),
`w4_d30_seal_the_g65_309_and_d77_661_are_other_id_spaces`
(`FROST_BLAST_FRIENDLY == 309` pin + the module's kind consts stay NPC-side).

**Tips read for this seal:** d30 `f23b43fde`, main `051600da8`, c8 `7c2de9c06`,
c9 `2042d77f1`, c10 `5fcc68ae6`, d31 `0487ea324`, d32 `ac1d38e70`,
d39 `a0d49dc1d`, d43 `0253bde90`, d125 `7a0482302`, f18 `a832d38ea`,
f26 `b89812bf7`, g65 `f6e07bb90`, d77 `5f10560b2`, d54 registry `38bfd1d22`+.
