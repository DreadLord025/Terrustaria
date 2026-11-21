# 365 — W4-e21 wiring-tail family SEAL: the five receipts at the pushed tips

> **STATUS: SEAL — 2026-11-21, branch
> `task/W4-e21-wiring-device-arms-3` (seal phase, the FINAL doc).**
> The e21 lane is closed: no further rounds. This doc receipts the five
> seal items against the CURRENT origin tips (fetched this round), the
> merge-time keep-one instructions, and the verification record.
> Code-only lane: no cargo/tests/rustfmt/graphify invoked; every witness
> on the branch is WRITTEN NOT RUN. Number chain 285→365: the seal-time
> self-pick 285 (lowest free key at the full remote-tip scan under the
> d54 registry-150 §6.3 "281+ FREE" rule) was a post-§9 self-pick into
> §7.2's 285 (the §7.2 owner KEEPS 285); the registry's §9 UNIFIED FINAL
> TABLE (§9.2 row 365, d54 tip `b4c94432c`) re-issues this doc as **365**.

## 0. The lane's landed estate (this branch, pre-seal tip `4782d663`)

| Commit | Scope |
| --- | --- |
| `8ebc25dd2` | pump `XferWater` drops the invented `active()` gate (`Wiring.cs:481` reads liquid directly) — a liquid-bearing inactive body corner transfers; witness pins the corner-emptying transfer |
| `fdbf37d98` | monolith flip gated to the nine C# kinds (`WorldGen.cs:51037-51134` — 410 ±56, 480/509/657/720/721/725/733 ±54, 658 +54-wrap-162; non-monolith cells keep their frames), grate 546/557 wake drops queued not discarded (`Wiring.cs:2543`/`:2548`); witnesses pin the kind-gate, the 410/658/733 steps and the grate swap+drop |
| `12fb1a2ef` | boulder statue projectile 99 rides the shared device-shot seam (`Wiring.cs:2002-2010` `NewProjectile 99/70/10f`) instead of a hardcoded drop — auto-spawns once the aiStyle-25 family rows land (§4) |
| `178a6d37f` | the bunny statue's pair: `NpcKind::Bunny` 46 / `PartyBunny` 540 (one `SetDefaults` row `NPC.cs:9336-9347`: 18x20 aiStyle 7 dmg 0 def 0 5HP catchItem 2019 NPCHit1/NPCDeath1, no value/kb rows → head 0/1f), the style-9 party pick at the `HitWireSingle` site (`Wiring.cs:2301-2313`), the case-46/540 FindFrame hop verbatim (`NPC.cs:76851-76904`), `update_guide` aiStyle-7 stand-in reads `self.kind.size()`, protocol 229/237 + G6 render-key 229/237, cs_id 46/540, NPC_46/NPC_540 sheets + bestiary Critter entries, ALL_KINDS 270→272 + s1_f_3 pin; 6 witnesses WRITTEN NOT RUN |
| `4782d6631` | ANALYSIS "The bunny statue's pair" + "The HitWireSingle census"; the stale PartyIsUp-unmodelled + Bunny-46/540-roster-gap notes corrected; anchors row |

The four fixes are the `12fb1a2e` chain tip the devices-family bible
(research-149) names: pump `8ebc25dd2` (the bible's `7c9a67a2f` is the
SAME fix re-picked onto c42's tree — `git branch -r --contains` shows it
only on `origin/task/W4-c42-wiring-device-arms`; on THIS branch the
ancestor is `8ebc25dd2`, verified `merge-base --is-ancestor` exit 0),
monolith/grate `fdbf37d98`, boulder seam `12fb1a2ef`.

## 1. Receipt 1 — the HitWireSingle census: 66/66 claimed, ZERO unclaimed

The full C# `HitWireSingle` (`Wiring.cs:996-2554`) top-level tile-case
list — 66 cases — plus the non-case arms (the 255..268 type-pair swap,
the `TileID.Sets.Torches`/`Campfires` gates, 130/131, doors 10/11,
trapdoors 386/387, tall gates 388/389, 216, 497/15) is fully claimed
across the four co-lanes (ANALYSIS "The HitWireSingle census"):

- the S1-L-6 baseline arms are in every lane;
- 314/21/467/219/642 (+ track pieces/hammer/IsPressurePlate) are
  **e20**'s — f11 duplicates dropped per the devices-family bible;
- torch/campfire/slab/doors/trapdoors/tall-gates are f11's originals
  folded into **c42**'s union-exec;
- the 144/421/422 wakes + actuator guard, 235/226 `downedPlantBoss`
  and the 506 shared validator are **c42**'s `de176ccdd` fix round;
- the pump `XferWater` gate, the monolith nine-kinds gate, the grate
  546/557 wakes and the boulder-statue 99 seam are **e21**'s.

**No still-unclaimed arm remains.** The bunny statue's pair 46/540 is
ported (`178a6d37f`) — the style-9 arm's request id now resolves to a
rostered kind instead of dropping at the drain.

## 2. Receipt 2 — the c42 wiring seal (`2910a720`, research/240→280)

Re-read at `origin/task/W4-c42-wiring-device-arms` (tip `69148794c`,
the registry-rename exec; the seal commit `2910a7209` is its parent):

- **The research/146 compile blocker is FIXED on-branch**: the hopper
  slot arms take `&mut Vec<ItemSlot>` (the e20 `a2d6cdad0` shape
  verbatim — `try_move_coins_in_chest`/`try_put_item_in_chest` grow the
  chest Vec via `Chest.Resize`); the c42 seal's §1.3 re-picks the
  census remainder merger-side from the exact `6d1754719` chain.
- **e20 owns** 314/hoppers/extractinators/trap-fixes (+ track pieces,
  hammer, `IsPressurePlate` — state `02ac64ab6`, the retire tip
  `46fdcfd8e`).
- **c42 owns** doors/torch/campfire/slab + the `de176ccdd` fix round
  (144/421/422/235/226/506) + fold-133 + the Toggle-unification.
- **e21's four fixes** ride `12fb1a2e` (§0) — the bible's e21 row
  ("union the two adjacency regions; no semantic overlap with e20")
  stands; c42's g78 compose receipt keeps e21's nine-kind
  `switch_monolith` body as the keep-one.
- The seal doc renumbered 240→**280** per registry-150 §6.3 (g58's
  executed 240 wins the three-way collision); this seal cites it as
  research/280 (`280-c42-wiring-seal.md` at the c42 tip).

## 3. Receipt 3 — the bunny 46/540 pair: keep-one vs d34, d39/c8 clean

Probed at the CURRENT origin tips (this round's fetch):

- **d39** (`task/W4-d39-npc-kinds-351-420`, tip `048c74f8f` the SEAL):
  zero `Bunny`/`PartyBunny` kind rows (`npc/data.rs` grep exit 1); its
  ANALYSIS still lists "the Bunny 46/540" among the unported statue
  kinds — a stale pre-e21 note on their side, no claim.
- **c8** (`task/W4-c8-style66-crawling-critters`, tip `10192602a` the
  SEAL): zero bunny kind rows; research/262's only bunny mention is
  the GoldBunny 443 → {46 Bunny} gold-peer fold row (d40's kind).
- **d34** (`task/W4-d34-npc-kinds-1-70`, tip `f0d92564e` phase 4):
  ports `NpcKind::Bunny` at protocol **270** (`npc/data.rs:596`/`:941`,
  the `30bfb41be` phase-1 row; render-key 270, `npc.rs:19377`), the
  same one-row SetDefaults stats, `bunny_frame()` FindFrame hop
  (`npc.rs:9790`, rows read `npc_rows(cs_id)`), `update_guide`
  stand-in, bestiary "Bunny" net 46. **d34 holds the d37-E8 KEEP
  receipt, CONSUMED 15/15 at 266-280** (research/108 S1: "d34 | KEEP
  receipt | CONSUMED (15/15 at 266-280) | none") — d34 is the roster
  canon for the band-1-70 kinds.
- **The keep-one instruction (merger-side)**: e21's `Bunny` row drops
  to d34@270 per the E8 kinds-lane rule (one roster row per kind; the
  wiring lane's duplicate enum/protocol/bestiary/sheet rows drop, its
  wiring re-keys onto the canonical kind — the E3 rule). e21's
  `PartyBunny`@540 is UNIQUE (no other tip ports it — the
  `NpcKind::Bunny =>` `-S` pickaxe over all branches finds exactly
  `178a6d37f` (e21) and `30bfb41be` (d34); `PartyBunny` only e21 +
  the shared `lang_tables.rs` name row). The style-9 party pick, the
  bunny-group cap (`wiring_stub.rs:16` — {46,540,303,337}, cs-id
  keyed, survives any protocol renumber), the FindFrame hop and the
  sheets re-key onto d34's `Bunny` + e21's `PartyBunny`.
- **The vanilla-46 collision is resolved on e21's side**: the protocol
  ids ride the port-local **229/237** holes (the vanilla 46 is the
  WallOfFlesh's `to_u16` slot — `npc/data.rs:377`), the G6 render-key
  terms ride the same 229/237 (`npc.rs:18672-18673`, the wyvern
  renumber's precedent). The full-tip `=> 229,`/`=> 237,` scan finds
  229 claimed ONLY by e21; 237 also by f18's `Firefly`@237 — but the
  d39 SEAL §6c already verdicts f18's whole 237-258 corridor off-plan
  (Firefly drops to c7@735), so 237 frees at merge and e21's
  PartyBunny keeps it. d125's old 229/237 squats (EmpressOfLight/
  PrismaticLacewing) are EXECUTED away (636/742 at tip `5a26ae348`,
  the d37-LV receipt).
- **The faithful AI_007 TownCritter arm** (`NPC.cs:53730-53760`/
  `:54191-54194`; `NPCID.Sets.TownCritter` carries 46/540,
  `NPCID.cs:10459`) stays the M4-NPC critter wave's — e21's
  `update_guide` stand-in is the documented deviation. f18's
  `npc_ai_town_critter.rs` (the Frog/Duck/Scorpion subset) is the
  splice candidate the d39 §6c table already routes; the bunny pair
  joins that wave, not this seal.

## 4. Receipt 4 — the boulder aiStyle-25 rows are d3's, PUSHED; the seam keep-one

The earlier census note ("the pushed branch is census-only, the rows
are in flight in its worktree") is SUPERSEDED — the re-probe at
`origin/task/W4-d3-proj-ai-21-30` (tip `5ff9fe983`) finds the family
landed: `241cbae09` the 13-kind rows (99/727/1013/1014/1053/1054/
1055/1057/1005/1021/655/1047/1048 + the 763 spike child; the shared
31x31 trap block `Projectile.cs:1833-1850`), `40f8cbe3d` the dispatch/
death/contact wiring (`projectile_ai_boulder.rs`), `affd7fa90` the
world-sim drains + d3's OWN `StatueSpawn::Boulder` arm, `986ab27bd`
the dust/gore bodies wave. The boulder-statue seam auto-spawns when
they merge: e21's `12fb1a2ef` arm pushes a `DeviceShot{kind:99,
damage:70, knock_back:10}` and `spawn_device_kind` spawns it the
moment `proj_data(99)` exists.

**The merge-time keep-one (TWO arms for one request):**

- **d3's arm** (`world_sim/wiring.rs:238` at its tip) spawns directly:
  `spawn_with_data(BOULDER, x - 15.5, y - 15.5, 0, 0, 70, 10)` — the
  half-box subtraction is the faithful `NewProjectile` centre→top-left
  conversion (`Projectile.cs:10876-10877`: `position = X - width/2`;
  the 31x31 box `:1835-1836` → 15.5).
- **e21's arm** (`world_sim/wiring.rs:244` at THIS tip) rides the
  shared device-shot seam and passes the C# centre anchor straight to
  `spawn_with_data_ai`, whose `pos` is the TOP-LEFT
  (`projectile.rs:15988` `pos: Vec2::new(x, y)`; the `spawn_hook` doc
  `:15481-15482` states callers subtract the half-box). **The seam
  therefore lands the boulder 15.5 px right/down of the C# position
  once the rows exist** — a latent anchor defect in e21's arm,
  invisible while the family drops.
- **Instruction**: the merger keeps ONE arm. Preferred: d3's
  direct-spawn arm (already faithful, already witnessed on its lane).
  If the device-shot seam is kept instead, apply the half-box lift at
  the boulder arm (`x - 15.5, y - 15.5`) — the same conversion d3
  uses; do NOT widen `spawn_device_kind` (the blast families
  108/164/178/601 and the trap shots share it and their anchors are
  the port's existing convention). The core request
  (`StatueSpawn::Boulder`, `boulder_statue` `Wiring.cs:1994-2013`,
  `CheckMech(900)`, anchor `((num89+1)*16, num90*16+28)`) is
  byte-identical on both trees — no core conflict.

## 5. Receipt 5 — the research-149 devices-family bible + the wiring tail SEALED

The e20 retire FINAL (`46fdcfd8e`, research-71 §8 SEAL banner +
research-149 `149-devices-family-bible.md`) is the ownership map this
seal consumes: the §1 ownership table (e20 devices / c42 canonical /
e21 four fixes / d106 BrokenTrapCleanup do-NOT-re-port / M2 contracts /
MP discards) and the §2 three merger blocking rules (c42 device halves
re-pick — now FIXED per receipt 2; one truth per table/predicate;
`TrapRequest::Spear` shape-breaking). The 149 key is contested with
c10's waterfowl doc (c10 renumbered 145→149→**270** per registry-150
§6.3; e20's 149 KEEPS — the registry's "e20 149/147/71 KEEP" row).

**The wiring-tail family is SEALED**: e21's four fixes + the bunny
pair + the census + this seal are the lane's whole estate; no further
e21 commits are planned after this seal. The merger executes the
gate-command (`cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids` — judge by the `test result: ok`
lines; cargo may exit 1 on success) + the anchors canary
(`tr-core/build.rs` + `cargo test -p tr-app --test main_anchors`);
this lane never ran them (code-only).

## 6. Verification record (this seal, code-only)

- Tip probes: `git fetch origin --prune` this round; the cited tips
  (`69148794c` c42, `46fdcfd8e` e20, `5ff9fe983` d3, `048c74f8f` d39,
  `10192602a` c8, `f0d92564e` d34, `2ea22ce3e` d37, `5a26ae348` d125,
  `a832d38ea` f18) read via `git show`/`git grep` at the refs — no
  working-tree reads of other lanes, no junction recursion.
- Keep-one evidence: the `-S "NpcKind::Bunny =>"` pickaxe (exactly two
  landing commits), the full-tip `=> 229,`/`=> 237,` scan (e21 + f18
  only), d34's E8 KEEP receipt CONSUMED (research/108 S1), d39 §6c's
  f18-corridor verdict, d125's executed 636/742 renumber.
- C# re-reads (direct paths, ≤100-line windows): `Wiring.cs:2301-2313`
  (the style-9 pick), `:1994-2013` (the boulder statue), `:481` (the
  pump liquid read), `:2541-2550` (the grate pair); `NPC.cs:9336-9347`
  (the one SetDefaults row), `:76851-76904` (the FindFrame hop group),
  `:7443-7449` (the bunny cap group); `WorldGen.cs:51037-51134` (the
  nine monolith kinds); `Projectile.cs:1833-1850` (the 31x31 boulder
  block), `:10876-10877` (the centre→top-left conversion);
  `NPCID.cs:10459` (TownCritter carries 46/540).
- The anchor-defect finding (§4) is a code READ at both tips
  (`spawn_with_data_ai`'s `pos` literal vs d3's −15.5 arm), not a test
  run — the witnesses stay WRITTEN NOT RUN.
- Number chain 285→365: the seal-time full remote-tip `docs/research/`
  scan picked 285 (lowest free then); registry §9.2 (d54 `b4c94432c`)
  rules 285 = §7.2 owner KEEPS, this doc re-issued 365.
- No cargo/tests/rustfmt/graphify invoked (lane rule). The seal commit
  touches docs + anchors only — zero `crates/` diff.
