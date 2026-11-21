# 103 — The `.trplayer` sidecar FINAL layout (the v7-collision superset, writer v11)

**Status: SEALED + ARBITRATED — THE MERGER CONTRACT (L2) + §L3-DEFINITIVE (the
107-byte `.trwld` v18 union, C#-order re-derived) + §L3-ARBITER FINAL (the
107-byte-order arbitration: D5 CONFIRMED against g71 92 §1-FULL's
books-in-prefix order, quoted C# Write() evidence) + the persist-family
merger-bible (the SEAL paragraph at the end).** Any merge of W4-g88 / W4-g89 /
W4-g89b / W4-g89c / W4-g90 / W4-g90b / W4-a6 / W4-g55 / W4-f33 (and the
future booster lanes) keeps the writer and reader described here —
`crates/tr-app/src/persist.rs` on `task/W4-g88-life-fruit-use`. This doc
is the token order, the token reservations and the back-compat matrix;
the module header of `persist.rs` is its short form. g89d's addendum
(`122-persist-v9-addendum.md`, `4344bd50`) is the g89-lane view of the
same line-8 contract + the v10 crystal reservation; its §6 fold
receipt records how this superset satisfies it (the writer stayed 9
until the crystal tokens landed, then bumped v10 — §7's receipt).
**The cross-layer owner map is f44c's
`132-persist-master-map.md` (`05e4020b`, read at this pass)** — this
doc is its L2 layer; §9 is the full merger persist-checklist with the
132 cross-refs, §8 the 135 token-order alignment note.

**v10 crystal fold inputs (read before folding):** g90b `e76ee133`
(`origin/task/W4-g90-artisan-bread-use` tip lineage `bb78974a` →
`e76ee133` → `0316b444` → `73367e0a`) landed the four shimmer Player
flags (`usedAegisCrystal`/`usedArcaneCrystal`/`usedGalaxyPearl`/
`usedAmbrosia`, ctor `:56078-56083`), their consumers, the `.plr` 260+
unlock bools both halves and a `.trplayer` v7 bread+five-flag-line
block; a6 `f23385e5` (`origin/task/W4-a6-crystals-powerups`) landed ALL
six shimmer use arms (`powerups::use_shimmer_permanent`, 5337-5342) +
the consumers (regen/luck/boosters) with NO sidecar writer — its pearl
flag rides `LuckState.used_galaxy_pearl` directly. research/135 (g90c
fold-input) was NOT pushed at v10 fold time — **it IS pushed now**
(`b3b82d07`, read FULL at this pass; the alignment note is §8, its
§2 six-line recognition rule LANDED in the v7 reader; the g90-FINAL
tip `94d33f2ca`'s §7 addendum receipt is §8 item 4). research/138
(a6 reconcile) was unpushed at the v11 fold — **it IS pushed now**
(`d7de8bd53`, read FULL at the receipts pass, plus the a6 worktree's
FINAL-probe revision §10) — §5a's keep-own is CONFIRMED and the
shimmer/g87 verdicts seated; the f44-FINAL cross-receipt is §1a.

**v11 torch fold inputs (this pass):** f33 `8a000407`
(`origin/task/W4-f33-torch-god-favor`) landed the Torch God's Favor
5043 use arm + consumers + a `.plr` torch-pair half + a `.trplayer` v7
torch-pair line (`unlockedBiomeTorches UsingBiomeTorches` directly
after dye); f44c `05e4020b` (`origin/task/W4-f44-save-plr-2`) landed
the COMPLETE `.plr` walk whose G7 unlock block INCLUDES the torch pair
(`player_file.rs:186-187`/`:1595-1596` named fields, `:54777-54778`
writes / `:55241-55242` reads gate 229) + research/132, the cross-layer
master map. The v11 fold appends f33's pair as the LAST post-dye line
(§2b — NOT an unlock token; token 7 stays DD2-reserved) and publishes
the conditional drop-recommendation (§2b) per 132 §2c's restore
precedence.

## 1. The collision (why this doc exists)

Eight pushed lane writers independently extended the post-dye block of the
`.trplayer` text sidecar (`persist::save`/`load`, the `.trwld`-adjacent
live-session save — NOT the vanilla `.plr`; that is `player_file.rs`,
each lane's own scope); a6's lane added use arms only (no writer):

| Lane | Commit | Version | What it appended after the dye line |
| --- | --- | --- | --- |
| W4-g89 | `920014cb` | 7 | ONE line: `usedAegisFruit` 0/1 (1 token) |
| W4-g88 (pre-reconciliation) | `f1eb9b60` | 7 | ONE line: `statLifeMax statLifeMax2` (2 tokens) |
| W4-g90 | `1f1b88f0` | 7 | ONE line: `ateArtisanBread` 0/1 (1 token) |
| W4-g55 | `d2210b91` (tip `cde865fc`) | 7 | the loadout block: index/share line (2 tokens) + 3 stored-set lines (41 tokens each) |
| W4-f33 | `7954d6ed` (tip `8a000407`) | 7 | ONE line: `unlockedBiomeTorches UsingBiomeTorches` (2 tokens, both 0/1) |
| W4-g89b | `9bf03ede` | 8 | widened g89's flag line to `usedAegisFruit usedGummyWorm` (2 tokens) |
| W4-g89c | `78766306` | 9 | widened it again to `usedAegisFruit usedGummyWorm usedAmbrosia` (3 tokens) |
| W4-g90b | `e76ee133` | 7 | the bread line + FIVE one-token shimmer flag lines (crystal/arcane/pearl/gummy/ambrosia, the `.plr` order `:54781-54786` minus the g89 fruit) |
| W4-a6 | `f23385e5` | — | NO sidecar writer — the six use arms + consumers only; the pearl flag rides `LuckState` |

A naive merge misparses one lane's file as another's (a 2-token loadout
index line reads as a life pair; a lone bread flag reads as aegis; a
3-token v9 unlock line reads as a life pair under the pre-fold-3
token-count reader — g89d's research/122 §2 records that corruption
class; g90b's six one-token v7 lines collapsed into the aegis/bread
slots under the pre-v11 v7 walk — 135 §2's rule now reads them
positionally; f33's torch pair reads as a life pair `(1,1)` under any
naive 2-token walk — the v7 all-0/1 recognition separates it). The
superset below loads ALL of them.

## 1a. The f44-FINAL cross-receipt (132 §2d ⨉ this §2b/§3 — ALIGNED)

f44's FINAL tip `bdd1cc79a` (2026-11-21 08:22, four minutes BEFORE this
branch's v11 fold `a9f1c29d8` 08:26) re-read research/132 FULL — its
§2d f33-fork input and this doc's v11 landing are the SAME adjudication
on both sides, with two version-seat deltas this receipt pins:

1. **The recognition rule — SAME.** 132 §2d rule 1:
   `post.len() == 1 && post[0].len() == 2 &&
   post[0].iter().all(|t| t == "0" || t == "1")` → f33 torch pair,
   resolved by VALUE RANGE (a life pair's first token is ≥ 100 — the
   ctor 100, clamped ≤ 500; "a hypothetical 1-hp life pair cannot
   exist"). This branch's `v7_torch_only` guard (`persist.rs:466-469`)
   is that predicate verbatim (over the non-empty-line projection);
   the rationale is the same value-range argument (§3's "no maxima
   pair can be all-0/1 — the permanent halves start at 100/20").
   132 §5's corruption row ("torch pair → life pair, hp feeds 1/1 —
   the value-range rule is mandatory") is closed by the witness
   `f33s_torch_only_v7_loads_positionally` (§4). **SAME rule, landed.**
2. **The seat — SAME LINE, version delta reconciled.** 132 §2d rule 2
   seats the pair as a NEW 2-token line AFTER the loadout block —
   line 16 — with "writer stays **v10**" (append-only, no pushed v10
   writer emitted line 16 yet). This branch spent the bump: the line
   landed at **v11** (§2b rule 2, §6 rule 5's append rule). Both are
   forward-only-safe — no pushed v10 writer emits line 16, and this
   reader back-reads a v10 file without the torch line
   (`the_v10_superset_still_loads_without_the_torch_pair`, §4). The
   v11 seat is STRICTER (the version self-identifies the torch arm)
   and is the binding one: this branch's writer is pushed. 132 §3-L2
   step 4's "writer stays v10" is superseded by this receipt the same
   way its "base = g88 `6998a4ecb` v9" is one pass stale (§9 step 6).
   The seat GEOMETRY agrees verbatim: last post-dye line, after the
   loadout block, restore via the slot-11 setter.
3. **The slot-11 fix — COMPATIBLE, verified against the C# this pass.**
   f44 `63204f85` made the `.plr` `:54778` byte the PROPERTY: write
   emits the getter `unlocked && builderAccStatus[11] == 0`
   (`Player.cs:3954-3958`, re-read), read lands on the setter
   `builderAccStatus[11] = !value` (`:3960-3963`), the later ≥230
   builder-array read overwrites slot 11 in C# order. This branch's
   live accessors (`player.rs:10652-10660`, f33's exact spot after
   `luck()`) are that getter/setter pair verbatim; the v11 writer
   emits `player.using_biome_torches()` (the GETTER, `persist.rs:303`)
   and the restore calls `set_using_biome_torches`
   (`app/mod.rs:3010`) — no stored toggle field anywhere, exactly
   132 §3-L1 rule 4's "do NOT resurrect a stored `using_biome_torches`
   live field". The sidecar's `SavedPlayer.using_biome_torches` is a
   transport bool (the getter's value at save time), the same shape
   f33's own writer used — 132 §2a's "only the derived bool rides L2".
4. **The L1 dedup direction — agreed.** 132 §3-L1 rule 4 deletes f33's
   `.plr` diff, keeps its live setter; this §2b rule 4 keeps f33/f44c's
   `.plr` halves untouched here. The conditional drop-recommendation
   (§2b rule 3: character-load hydration AND the exit-save) is this
   doc's addition — 132 §2c's precedence (character first, sidecar
   second, sidecar wins as newer state) is cited verbatim on both
   sides. 132 §2b's contested-field table already seats the torch pair
   as §2d's new line; the two docs' dual-carrier rulings agree.
5. **132's stale points the merger must carry** (this receipt is the
   correction of record): §2b rule 3's token-order parenthetical still
   cites 135's pearl=4 recommendation (superseded — §8 item 1, and
   138 §6's WRITTEN-order verdict below); its L2 base is the v9/v10
   worktree probe (now the pushed v11 tip); its L3 97-byte head does
   not seat g87's pets (138-FINAL's 68-byte five-way union predates
   g72/c43 — the definitive union is §L3-DEFINITIVE's 107-byte head;
   f44's own RETIRE `d35876af9` has since fixed the 97→107 itself,
   §L3-DEFINITIVE D5's broadcast receipt).

## 2. The FINAL layout (the superset writer — version 11)

Fixed prefix, unchanged since v6 (line numbers 1-based):

| Line | Content | Since |
| --- | --- | --- |
| 1 | `11` (the version) | v11 |
| 2 | `x y hp` (3 tokens, f32 f32 i32) | v1 |
| 3 | 50 × `kind count` inventory pairs (v1 wrote ≤10) | v2 |
| 4 | `coins` (u64) | v3-era |
| 5 | `armor[0] armor[1] armor[2]` kinds (3 tokens) | v4 |
| 6 | `count (npcId killCount)*` bestiary pairs | v5 |
| 7 | 10 dye kinds (10 tokens) | v6 |

Post-dye block (lines 8+, the superset contract, in order):

| # | Line | Tokens | Field(s) | C# anchor (`.plr` write / read) |
| --- | --- | --- | --- | --- |
| 8 | the unlock line | 6 (+1 RESERVED) | `usedAegisFruit usedGummyWorm usedAmbrosia usedAegisCrystal usedArcaneCrystal usedGalaxyPearl` 0/1 | `Player.cs:54782`/`:55256`, `:54785`/`:55259`, `:54786`/`:55260`, `:54781`/`:55255`, `:54783`/`:55257`, `:54784`/`:55258` (ctor `:56079`/`:56082`/`:56083`/`:56078`/`:56080`/`:56081`) |
| 9 | the life pair | 2 | `statLifeMax statLifeMax2` (permanent, effective); permanent half clamps 500 on read | `:54773` / `:55220-55224` |
| 10 | the mana pair | 2 | `statManaMax statManaMax2`; permanent half clamps 200 on read | `:54775` / `:55226-55229` |
| 11 | the bread flag | 1 | `ateArtisanBread` 0/1 | `:54779` / `:55245` (gate 256; ctor `:56077`) |
| 12 | the loadout index | 2 | `currentLoadoutIndex equipShareFlags`; index clamps 0..2 on read | `:54919` / `:55722` (`Utils.Clamp`, gate 262) |
| 13-15 | 3 stored-set lines | 41 each | per set: 20 armor kinds + 10 dye kinds + 10 hide 0/1 + the 30-bit share word (`EquipmentLoadout.favorited`) | `:54920-54923` / `:55723-55726`; `EquipmentLoadout.cs:31-45` Serialize / `:47-61` Deserialize |
| 16 | the biome-torch pair | 2 | `unlockedBiomeTorches UsingBiomeTorches` 0/1 (f33's pair, the v11 append — §2b) | `:54777`/`:54778` writes / `:55241`/`:55242` reads (gate 229); the toggle is the `builderAccStatus[11]` property `:3950-3964` |

Kinds-only, like the armor/dye lines: the sidecar carries no
prefix/stack/favorited-per-item — a restored slot is a single stack,
prefixless (`loadout_item`, `persist.rs`). The live share word
(`App.equip_share_flags`) is the equip-context `Item.favorited` bitmap
over armor[0..20] then dye[0..10] (`ItemSlot.cs:395-399`), NOT the grid
favourites.

### 2a. The unlock line's token map (the booster lanes)

The vanilla `.plr` writes the unlock family as consecutive bools
(`Player.cs:54780-54787`, reads `:55249`/`:55255-55260`/`:55265`). The
superset's unlock line keeps the fruit-first order g89 established
(vanilla interleaves crystal/fruit — the deviation research/122 §3.1
records) and appends the crystal trio after ambrosia in the vanilla
RELATIVE order among themselves (verified at the v10 fold against
`Player.cs:54776-54787`: `:54781` usedAegisCrystal → `:54782`
usedAegisFruit → `:54783` usedArcaneCrystal → `:54784` usedGalaxyPearl
→ `:54785` usedGummyWorm → `:54786` usedAmbrosia → `:54787`
downedDD2EventAnyDifficulty). The pre-v10 reservation table listed
pearl/arcane/crystal — those tokens were RESERVED (never written by any
pushed superset writer), so the v10 fold corrects the map to the
verified vanilla relative order crystal/arcane/pearl with no
back-compat cost. A future lane appends its flag at its reserved
position (token 7 DD2) and bumps the writer to **v12** (v11 was spent
on the torch line, §2b) — the reader already skips unknown tokens:

| Token | Field | C# write / read | Status |
| --- | --- | --- | --- |
| 1 | `usedAegisFruit` | `:54782` / `:55256` | LIVE (g89, folded) |
| 2 | `usedGummyWorm` | `:54785` / `:55259` | LIVE (g89b, folded) |
| 3 | `usedAmbrosia` (5342, `pickSpeed -= 0.05f; tileSpeed += 0.05f; wallSpeed += 0.05f`, `:12861-12866`; use arm `:45417-45422`) | `:54786` / `:55260` | LIVE (g89c `78766306`, folded — the v9 bump) |
| 4 | `usedAegisCrystal` (5337, +0.2 lifeRegenTime, `:19113-19116`; use arm `:45387-45392`) | `:54781` / `:55255` | LIVE (g90b `e76ee133` flag + a6 `f23385e5` use arm, folded — the v10 bump) |
| 5 | `usedArcaneCrystal` (5339, mana delay −0.05 `:19430-19433` + regen +max/50 `:19453-19456`; use arm `:45399-45404`) | `:54783` / `:55257` | LIVE (g90b/a6, folded — v10) |
| 6 | `usedGalaxyPearl` (5340, +0.03 luck `:29142-29145`; use arm `:45405-45410`) | `:54784` / `:55258` | LIVE (g90b/a6, folded — v10; the Player field is the persistence owner, `update_luck` mirrors into `LuckState`) |
| 7 | `downedDD2EventAnyDifficulty` | `:54787` / `:55265` (gate 182) | RESERVED — v12 |

(The vanilla also writes a dropped `false` at `:54780` between bread and
aegisCrystal — the port does NOT reserve a slot for it; the sidecar is
its own format. The combat-book vol-two flag — item **5336**,
`NPC.combatBookVolumeTwoWasUsed`, `Player.cs:45337-45349` — is a WORLD
flag (`.wld` `:1407`/`:2416-2423`), NEVER a line-8 token; census
`ItemID.cs:12355-12369`: 5336 CombatBookVolumeTwo / 5337 AegisCrystal /
5338 AegisFruit / 5339 ArcaneCrystal / 5340 GalaxyPearl / 5341 GummyWorm
/ 5342 Ambrosia / 5343 PeddlersSatchel; LifeFruit is **1291**,
`ItemID.cs:4265`.) A lane activating a reserved token MUST: read its
position in `load` (the v8/v9/v10 positional arm), emit it in `save`'s
unlock line, bump the writer version (v12 for token 7), keep the
short-line back-reads, restore it in `app/mod.rs` beside the
aegis/gummy/ambrosia/crystal flags, add a round-trip witness and update
this table. Tokens beyond 7 need a new row here first. **The v11 bump
was spent on the torch LINE (§2b), not token 7** — a future DD2 lane
activating token 7 bumps the writer to **v12** with the 6-token v11
back-read (the version number tracks the format, not the token count).

**The DD2 v12-contract-seat census (this receipts pass — NO lane claims
token 7).** The brief's question "which lane owns DD2-buy?" answered
against the pushed tips:

- **The field**: `downedDD2EventAnyDifficulty` (`Player.cs:1916` decl,
  `:54787` write / `:55265` read gate 182 — re-verified this pass). Its
  ONLY live writer is the per-tick mirror `Player.cs:24450-24453`:
  `if (DD2Event.DownedInvasionAnyDifficulty) downedDD2EventAnyDifficulty
  = true` — a DERIVED per-player cache of the WORLD tier latches
  (`DD2Event.cs:89-96`: T3, else T1/T2-derived). Its consumers: the
  DD2-turret use refusal (`:52083`, `!downedDD2EventAnyDifficulty &&
  !DD2Event.Ongoing`). It is NOT a shop gate.
- **DD2-buy (the Tavernkeep's Defender-Medal shop, `Chest.cs` case 21
  `:2963-3045`)** gates on `Main.hardMode && NPC.downedMechBossAny`
  (tier-2 rows, `:2965`) / `Main.hardMode && NPC.downedGolemBoss`
  (tier-3 rows, `:2966`) — the SAME progression gates as
  `DD2Event.ReadyForTier2/3` (`DD2Event.cs:127-149`), NOT on any
  per-player DD2 bool. The medal currency itself
  (`shopSpecialCurrency = CustomCurrencyID.DefenderMedals`, `:2987+`)
  is UNPORTED — d124's own ANALYSIS scope note pins it: "the
  defender-medal currency system (`Chest.cs:2987+`, the Tavernkeep's
  DD2 rows) is not in the port yet" (the `shopSpecialCurrency != -1`
  price arm, `Player.cs:35361-35365`); the medal SAVINGS display half
  is d122's lane (`draw_savings_medals`, research/142).
- **d124 (`origin/task/W4-d124-ooa-crystal-tiers` tip `56fa087e3`) owns
  the WORLD tier persistence, NOT the player token.** Its diff touches
  NEITHER `persist.rs` NOR `player_file.rs` NOR `save.rs` (verified
  `git diff --stat origin/main...tip` — empty for all three). Its
  claims: the three `DownedInvasionT1/2/3` WORLD latches
  (`DD2Event.Save/Load` `:151-156`/`:167-169`) persisted as
  `World::dd2_downed_t1/2/3` through `.wld` flags tail 1931-1933 behind
  the v178 gate (`WorldFile.cs:2214-2218`; below-177 clears) —
  `savedBartender@1930` stays unowned; the starter-crystal drops
  (`DropStarterCrystals` `DD2Event.cs:929-942`); the victory slice
  (`WinInvasionInternal` `:314-342` — the tier latches + 3/15/60 medal
  stacks + progression 23). The tier-latch WIRING pre-exists on main
  (`a0f08c844` S1-E-12c); d124's round adds the persistence witnesses +
  the tavernkeep-chat rewire onto the persisted latches.
- **The seat ruling**: token 7 stays **RESERVED — v12, UNCLAIMED**.
  132 §2a agrees ("if a DD2 lane ever needs it live, the token is
  there, but no pushed lane claims it"); 138 §2 routes `downedDD2`
  `:54787` to f44's L1-only list. Because the C# field is a derived
  mirror of the WORLD latches d124 already persists, a future DD2 lane
  activating token 7 has TWO faithful options: (a) hydrate the mirror
  from `World::dd2_downed_t*` at load (the `:24450` per-tick rule makes
  the sidecar token redundant within one tick of world load — the
  §2b-rule-3 conditional-drop pattern), or (b) carry the token for
  vanilla-`.plr`-only characters. Either way the bump is **v12** with
  the 6-token v11 back-read, per the activation checklist above. No
  v12 writer exists on any pushed tip (census: g88 v11 is the highest
  `.trplayer` writer; d124 writes no sidecar at all).

**SEAL-pass version-seat update (132-RETIRE §2e, `d35876af9`):** f44's
RETIRE proposes folding g87's `dh` sentinel line as line 17 at **v12**
(the newer input over 138-FINAL §10.4's DROP — §L3-DEFINITIVE D5's
receipt). IF the merger takes that fold, token 7's activation bump
moves to **v13** (the 7-token back-read reads a v12 6-token line);
every "v12" in this §2a is then read as "the next free version after
the dh fold". If the merger keeps the DROP, v12 stands as written.
Either way token 7 is UNCLAIMED and this branch's v11 writer is
unchanged — the seat is a merge-time decision, both options published.

### 2b. The f33 torch pair — line 16, NOT token 7/8 (the fold adjudication)

The brief's question: fold f33's torch-favor as unlock token 7, or a
sidecar token 8, or drop the line because the `.plr` carries it?
**Answer: NONE of the token options — the pair is not an unlock-family
field, and the line stays as a deliberate dual-carrier with a published
conditional drop-recommendation.**

1. **Token 7 stays DD2-reserved.** `downedDD2EventAnyDifficulty`
   (`Player.cs:54787`, gate 182) owns it (§2a). `unlockedBiomeTorches`
   is NOT in the `.plr` 260+ unlock block at all — vanilla writes it at
   `:54777`, BEFORE bread (`:54779`) and the six used-upgrades
   (`:54781-54786`), under gate 229 (verified `:54776-54787` /
   `:55239-55262` this pass). Giving it an unlock-line token would
   break §2a's "the unlock line mirrors the 260+ block" invariant.
   Token 8 (a new row) would work mechanically but buys nothing: the
   second half, `UsingBiomeTorches`, is not even a field — it is the
   `unlockedBiomeTorches && builderAccStatus[11] == 0` PROPERTY
   (`:3950-3964`), whose backing slot rides the `.plr`'s
   `builderAccStatus[12]` int array (`:54899-54902`), a different
   family again. A two-token pair on its own line models the C#
   faithfully; two unlock tokens would not.
2. **The line landed at v11** as the LAST post-dye line (§6 rule 5's
   append rule): f33's own v7 shape (the pair FIRST after dye) loads
   via the v7 all-0/1 recognition (§3); the superset's v11 writer
   appends it after the loadout block, and the v11 reader takes the
   last two-token line after the last set line. The restore
   (`app/mod.rs`) writes `Player.unlocked_biome_torches` + the toggle
   through `set_using_biome_torches` (f33's exact accessor spot after
   `luck()`), never a second bool field.
3. **The f44c-G7 redundancy — DROP-RECOMMENDATION (conditional).**
   f44c's complete `.plr` walk (`05e4020b`, research/132 §2c + the G7
   fix `180b7430`) models the SAME two vanilla bools
   (`PlayerFile.unlocked_biome_torches`/`using_biome_torches`,
   `player_file.rs:186-187` read `:739-740` / write `:1847-1848`, plus
   the legacy-279 subset bools 2-3 `:1456-1457`). Once BOTH of these
   hold, the sidecar torch line is REDUNDANT and the merger SHOULD
   drop it (writer back to the v11 layout minus line 16, reader keeps
   the line-16 arm as a back-read):
   - **(a)** the character-load path hydrates a live `Player` from the
     `.plr` G7 fields (today it feeds only the card list,
     `save_list.rs:294` — 132 §2c), AND
   - **(b)** the in-session `.plr` re-save lands (the vanilla
     `Player.SavePlayer` per world-exit — 132 §1's recorded
     remainder; today the `.plr` is written ONLY at character
     creation).
   Until then the line is NOT redundant: a mid-world Torch God's
   Favor use (5043, `:45226-45243`) mutates the live Player, the
   sidecar is the only carrier that survives a world reload, and
   dropping the line would lose the unlock — exactly the loss class
   132 §2b's dual-carrier rule exists to prevent. **Restore precedence
   when both carriers exist (132 §2c): character load FIRST
   (`App::new` spawns the Player), sidecar restore SECOND (world
   load) — the sidecar wins as the newer state.** A `.plr`-newer-than-
   sidecar file pair (vanilla play between port sessions) resolves
   the same way the other dual-carried fields do; the exit-save (b)
   is what makes that resolution lossless.
4. **The `.plr` halves stay f33/f44c's** (§6 rule 3): this branch's
   `player_file.rs` is untouched by the fold. At the L1 merge, f33's
   named torch fields dedup onto f44c's identical named fields (same
   names, same slots — a clean textual dedup, 132 §3-L1 rule 3's
   pattern; f44c's walk is the base).
5. **f33's research/93** (`93-torch-god-949-keep-one.md`, read at tip
   `8a000407`) is the 949-projectile keep-one recommendation vs d19 —
   a CONSUMER-side dedup, orthogonal to this persist contract (it
   names `unlocked_biome_torches` only as the AI's stop-gate,
   `Projectile.cs:42789`). The brief's "93-addendum" is NOT pushed
   (verified `git ls-tree` on the f33 tip — 93 exists, no addendum
   file, no addendum commit in `git log --all`); if f33b publishes
   one, it lands against this §2b adjudication, which already answers
   the persist half (the pair is a line, not a token; the
   drop-recommendation is conditional on the two 132 §2c seams).

## 3. The reader (tolerant, version-dispatched)

`persist::load` accepts versions 1-11 (1/2 migrate kinds). For the
post-dye block:

- **v8/v9/v10/v11**: the FIRST non-empty post-dye line is POSITIONALLY
  the unlock line (g89b/g89c's contract, widened at v10): token 1 →
  aegis, token 2 → gummy, token 3 → ambrosia, token 4 → aegis-crystal,
  token 5 → arcane-crystal, token 6 → galaxy-pearl; short lines
  back-read (a 1-token line loads worm/ambrosia/crystals false, a
  2-token line loads ambrosia/crystals false — g89b's v7 / g89c's v8
  back-read shapes; a 3-token line loads the crystal trio false — the
  v9→v10 back-read); token 7+ is skipped (reserved, DD2 → v12).
  The remaining lines dispatch on shape:
  - 1 token → the bread flag;
  - 2 tokens → the first is the life pair (clamp 500), the second the
    mana pair (clamp 200);
  - the 2-token line DIRECTLY BEFORE the first 41-token line → the
    loadout index line (g55's block self-identifies by its set lines;
    index clamps 0..2);
  - 41 tokens → stored sets 0,1,2 in order (kinds → `loadout_item`,
    hide flags, share word);
  - **v11 only**: the LAST 2-token line AFTER the last 41-token set
    line → the f33 biome-torch pair (§2b; a v10 file ends at the sets
    and back-reads the pair false);
  - any other shape → tolerated, defaults kept.
- **v7**: two LOSSLESS recognitions run BEFORE the shape walk:
  - **the g90b six-line rule (research/135 §2, landed)**: exactly six
    non-empty lines, ALL 1-token → the lane-7 block, read positionally
    per 135 §1: bread / aegis-crystal / arcane-crystal / galaxy-pearl /
    gummy / ambrosia (the `.plr` `:54781-54786` order minus the g89
    fruit, bread `:54779` leading). aegis stays false — g90b never
    wrote it. No other pushed writer emits six 1-token lines.
  - **the f33 torch-only rule**: exactly ONE non-empty line, a 2-token
    pair with BOTH tokens 0/1 → the biome-torch pair. No maxima pair
    can be all-0/1 (the permanent halves start at 100/20 — the fresh
    ctor `statLifeMax = 100`, `statManaMax = 20`), and no other pushed
    v7 writer emits a lone all-boolean pair (g55's index line always
    has its three 41-token set lines behind it).
  Otherwise the same shape walk, except the FIRST 1-token line → aegis
  and the SECOND → bread (no positional unlock line — g89/g90's
  single-flag writers both land here).
- **v3-v6**: no post-dye block; every superset field keeps its fresh
  default (`None` maxima, false flags, loadout 0, three empty sets,
  share word 0).

### Known ambiguity (bounded, ends at the fold)

A lone 1-token flag line cannot self-identify: g90's bread-only v7 file
reads into the AEGIS slot (slot 1 wins; witness
`g90s_bread_only_v7_loads_into_the_first_flag_slot`). Both single-flag
lanes' own readers map their line to their own flag, so a pre-fold
bread-only file loaded by the superset reader grants aegis instead of
bread. The superset writer emits all six flags positionally (v10+), so
files written after the fold are unambiguous. A g89-file misread is
impossible (its flag IS slot 1). This is the ONLY surviving ambiguity.

**g90b's v7 six-line block** (`e76ee133`: bread + crystal/arcane/pearl/
gummy/ambrosia, each its own 1-token line) loads LOSSLESSLY under the
v11 reader via 135 §2's recognition rule (witness
`g90bs_six_line_v7_loads_positionally`). The pre-v11 bounded misread
(bread→aegis slot, last line→bread, crystal trio lost — the old witness
`g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread`) is RETIRED;
135 §2 called the rule mandatory and it landed with the torch fold.

**f33's v7 torch-only block** (`8a000407`: one `unlocked using` 0/1
pair directly after dye) loads LOSSLESSLY via the all-0/1 recognition
(witness `f33s_torch_only_v7_loads_positionally`) — under a naive walk
it read as a life pair `(1,1)`, clamping to a bogus 1-life character;
the recognition prevents that corruption class.

## 4. The back-compat matrix (witnesses in `persist.rs::tests`)

| File shape (writer) | Version | Loads? | Result |
| --- | --- | --- | --- |
| v3 (no armor) | 3 | yes | empty armor/dye, no kills, all superset fields default (`version_3_loads_with_empty_armor`) |
| v4 (armor, no kills/dye) | 4 | yes | armor rides; dye empty (`version_4_loads_with_empty_dye`) |
| v5 (kills, no dye) | 5 | yes | dye empty, `None`/false (`version_5_loads_with_empty_dye`) |
| v6 (dye, no block) | 6 | yes | all superset fields default (`version_6_loads_without_life_maxima`, `w4_g55_a_v6_sidecar_loads_empty_loadouts`) |
| g89 flag-only | 7 | yes | aegis reads; pairs `None`, bread false (`w4_g89s_flag_only_v7_still_loads`) |
| g88 life-only | 7 | yes | life pair reads; mana `None`, flags false (`the_life_only_v7_still_loads`) |
| g90 bread-only | 7 | yes* | *lone flag → aegis slot (the bounded ambiguity above) |
| g55 loadout-only | 7 | yes | index/share + sets read; pairs `None`, flags false (`g55s_loadout_only_v7_still_loads`) |
| g88⨉g89 3-line superset (`717e0640`) | 7 | yes | flag + both pairs; bread false (`the_three_line_superset_still_loads_without_bread`) |
| v7 clamp probe (900/900 pairs) | 7 | yes | permanent halves clamp 500/200 (`version_7_clamps_the_permanent_caps`) |
| g89b unlock-only | 8 | yes | both tokens read; rest default (`g89bs_unlock_only_v8_still_loads`) |
| g89b 1-token back-read | 8 | yes | aegis reads, gummy false (`a_one_token_unlock_line_reads_under_v8`) |
| the pre-v9 superset (fold 3's writer: 2-token unlock + pairs + bread + loadouts) | 8 | yes | unlocks read, ambrosia false, block behind unchanged (`the_v8_superset_still_loads_without_ambrosia`) |
| g89c unlock-only | 9 | yes | all three tokens read; rest default (`g89cs_unlock_only_v9_still_loads`) |
| g89c 2-token back-read | 9 | yes | aegis+gummy read, ambrosia false (`a_two_token_unlock_line_reads_under_v9`) |
| g90b bread+five-flag-lines | 7 | yes | LOSSLESS since v11: 135 §2's six-line rule reads all six flags positionally (`g90bs_six_line_v7_loads_positionally`; the pre-v11 bounded misread retired) |
| f33 torch-only pair | 7 | yes | LOSSLESS: the lone all-0/1 pair recognition reads the torch pair, NOT a life pair (`f33s_torch_only_v7_loads_positionally`) |
| g87 `dh` sentinel line (`dfbcaf94b`) | 7 | yes* | *TOLERATED, lossy-not-corrupt: `dh 0 0` is a 3-token line, the shape walk's `_ => {}` arm keeps the defaults (both flags lost, NO hp/flag corruption — 138-FINAL §10.4's census, verified `persist.rs:565`). The line DROPS at merge (L1-only ruling); no recognition owed |
| the pre-v10 superset (the v9 writer: 3-token unlock + pairs + bread + loadouts) | 9 | yes | unlocks read, crystal trio false, block behind unchanged (`the_v9_superset_still_loads_without_the_crystals`) |
| v9 3-token back-read | 10 | yes | aegis+gummy+ambrosia read, crystal trio false (`a_three_token_unlock_line_reads_under_v10`) |
| v10 6-token unlock-only | 10 | yes | all six tokens read positionally (`the_six_token_unlock_line_reads_under_v10`) |
| future 7-token unlock line | 10 | yes | tokens 1-6 live, 7+ skipped; pairs/bread behind read (`reserved_unlock_tokens_are_skipped`) |
| the pre-v11 superset (the v10 writer: 6-token unlock + pairs + bread + loadouts, NO torch line) | 10 | yes | everything reads unchanged, the torch pair back-reads false (`the_v10_superset_still_loads_without_the_torch_pair`) |
| corrupt loadout index 9 | 7/8/9/10/11 | yes | clamps to 2 (`the_loadout_index_clamps_on_read`) |
| THE FULL SUPERSET round-trip | 11 | yes | every field survives, exact line order asserted incl. the `1 1` torch line LAST (`the_superset_round_trips_the_full_block`) |
| full-state round-trip | 11 | yes | position/inv/coins/armor/dye/kills + all superset fields (`round_trips_player_state`) |
| bread-only round-trip | 11 | yes | flag survives; fresh player loads false (`the_artisan_bread_flag_round_trips`) |
| loadout-block round-trip | 11 | yes | index/share/kinds/hide/word per set (`w4_g55_the_loadout_block_round_trips`) |
| torch-pair round-trip | 11 | yes | unlock + toggle survive; fresh player loads false (`w4_f33_the_biome_torch_pair_round_trips_the_sidecar`, f33's own witness re-pinned) |
| torch toggle-off / locked-getter | 11 | yes | the pair rides the PROPERTY getter: unlocked+off writes `1 0`, locked writes `0 0` (`the_torch_toggle_half_rides_the_property_getter`) |

Code-only mode: all witnesses WRITTEN, NOT RUN (the lane's constraint).

## 5. The carriers (fields restored, consumers stay with their lanes)

`App::new`'s sidecar restore (`app/mod.rs`, the `persist::load` block)
applies, in order: position (collision-gated) → life pair → mana pair →
aegis → gummy → ambrosia → aegis-crystal → arcane-crystal → galaxy-pearl
→ bread → torch unlock + toggle (via `set_using_biome_torches`) →
loadouts + index + live share word →
**hp clamp** (so a saved 405-life character keeps its 405 hp) →
coins/armor/dye/inv.

| Field | Declared at | Consumer (owner lane) |
| --- | --- | --- |
| `Player.life_max_base` / `max_hp` | g88 (`f1eb9b60`) | the fruit/crystal gates (g88, landed) |
| `Player.mana_max_base` / `mana_max` | pre-existing / g88 crystals | the mana gate (g88, landed) |
| `Player.used_aegis_fruit` | g89's exact spot (`player.rs`, after `used_gummy_worm`) | +4 defense `:12853-12856` + the 5338 use arm (g89; a6's `use_shimmer_permanent` 5338 arm dups it — §5a) |
| `Player.used_gummy_worm` | pre-existing (S1-P-i35) | +3 fishing `:12857-12860` (landed); the 5341 use arm (g89b; a6 dups — §5a) |
| `Player.used_ambrosia` | g89c's exact spot (`player.rs`, after `used_aegis_fruit`) | the three-speed row `:12861-12866` + the 5342 use arm (g89c; g90b's booster row + a6's use arm dup — §5a) |
| `Player.used_aegis_crystal` | g90b's exact spot (`player.rs`, after `used_ambrosia`, `e76ee133`) | the +0.2 life-regen ramp `:19113-19116` (g90b's `RegenInputs.aegis_crystal` slot — a6 NOW matches it, 138 §3.3's f32-order fix) + the 5337 use arm (a6's machine keeps per 138 §1 — §5a) |
| `Player.used_arcane_crystal` | g90b's exact spot (after `used_aegis_crystal`) | the mana rows `:19430-19433`/`:19453-19456` (g90b `RegenInputs.arcane_crystal` / a6 `RegenInputs.used_arcane_crystal`) + the 5339 use arm (§5a) |
| `Player.used_galaxy_pearl` | g90b's exact spot (after `used_arcane_crystal`) | the +0.03 luck `:29142-29145` via the `update_luck` mirror into `LuckState.used_galaxy_pearl` (g90b's shape kept); a6's 5340 use arm NOW sets the Player field too (138 §3.2's fix landed — the direct-LuckState write is gone, §5a) |
| `Player.ate_artisan_bread` | pre-existing (reach formula) | +4 reach `craft.rs::tile_reach` `:35671` (landed); shop gate `Chest.cs:2958` + the 5326 use arm (g90) |
| `Player.unlocked_biome_torches` | pre-existing (recipe gate, `player.rs`) | the `needTorchGodsFavor` recipe gate `craft.rs:658` (landed) + the torch-conversion consumers (`player_lights.rs`, f33's restored hold arms) + the 5043 use arm (f33 `torch_god_favor_use.rs`) |
| `Player.using_biome_torches()` / `set_using_biome_torches()` | f33's exact spot (`player.rs`, after `luck()`, `8a000407`) | the property over `builder_acc_status[11]` (`Player.cs:3950-3964`) — the v11 writer reads the getter, the restore writes the setter; no second field |
| `Player.loadouts` / `current_loadout_index` | g55's exact spot (`player.rs`, after `hide_visible_accessory`) | the swap machine + loadout UI (g55 Ph1/3/4) |
| `App.equip_share_flags` | g55's exact spot (`app/mod.rs`, after `dye_swap_counter`) | the share mechanic (g55 Ph4) |

### 5a. The use-arm dup adjudication (g88 keep-own — CONFIRMED by the pushed 138)

**138 IS PUSHED NOW**: `origin/task/W4-a6-crystals-powerups` tip
`d7de8bd53` (08:06, "POWERUP 4-way master reconcile: publish research/138
+ fix five a6 consumer-fidelity gaps") carries
`docs/research/138-powerup-master-reconcile.md` (387 lines pushed; the
a6 worktree carries a FINAL-probe revision, +368/-78 unpushed, read
in-tree at this pass — its §10 g87 re-adjudication is cited below as
138-FINAL). The v11-pass probe ("STILL not pushed", tip `8ed96ee5`) is
superseded. **138's two open merge actions are BOTH CLOSED by this
branch's pushed v11 tip `a9f1c29d8`** (08:26, after 138's probes):
its §6/§10.6 six-line-rule re-flag read the v10 tip `c8985b534` — the
`v7_six_line` guard (`persist.rs:463-465`) + the positional read
(`:470-481`) + the replacement witness
`g90bs_six_line_v7_loads_positionally` (`:1198`) landed at v11, the
bounded-misread witness RETIRED (§3/§8 item 2); its §6 token-order
verdict (the written crystal/arcane/pearl order binding) is this
doc's live map verbatim — one delta: 138 §6's "token 7 DD2
RESERVED→v11" predates the torch fold spending v11 on the LINE, so
the DD2 bump is **v12** (§2a/§6 rule 4). The verdicts, aligned item
by item:

- **29/109/1291 — g88 keep-own CONFIRMED verbatim.** 138 §1's row:
  "**g88 keep-own** use arms (`crystals.rs`/`life_fruit.rs`, 103 §5a —
  the `.trplayer` carrier lives there); a6's three dups DROP at merge;
  c36's `use_permanent_powerup` DROP (its `max_hp`-gate variant is less
  faithful than the permanent-base gate)". The provisional keep-own
  below is now the adjudication of record. 138 §3.1's row-targeted
  `complete_condition` fix landed on a6 (c36's shape) — the shared
  helper survives the merge; g88's `handle_special_event_life_mana`
  dispatch is named faithful.
- **The shimmer six (5337-5342) — a6's machine KEEPS, per 138 §1.**
  "**a6** `use_shimmer_permanent` machine = the single use-arm owner
  for all six ... dedup g89's `use_aegis_fruit`/`use_gummy_worm`/
  `use_ambrosia` + g90's `use_shimmer_permanent_item`." This SUPERSEDES
  §9 step 8's old "ONE `use_shimmer_permanent_item` core (g89's
  signature)" preference — the merger takes a6's machine (the only
  full-cluster seam), with **g90's Player field declarations + consumer
  shapes as the merge anchors** (138 §3.2-3.4 fixed a6 to match: the
  pearl Player-field + `update_luck` mirror, the aegis-crystal +0.2 f32
  slot BETWEEN base +1 and crimson, the fruit→worm→ambrosia booster
  order). The sidecar contract is UNAFFECTED — it carries the flags,
  whichever arm sets them; this branch's token-6 carrier already
  restores the Player-owned `used_galaxy_pearl` (§5), which is exactly
  the field 138 §3.2 made a6's 5340 arm set. The old "a6's 5340 arm
  must set Player.used_galaxy_pearl" flag is CLOSED (138-FINAL §10.7
  item 2 verifies the fix + witness in a6's tree).
- **5326 loaf — g90 keep-own** (138 §1: the skeleton-merchant stock
  gate `Chest.cs:2958` consumer); a6's dup drops. **4382/5336 books +
  5343 satchel — keep ONE, 138 recommends a6 or g89e/g90** (equal
  fidelity); the TRAINING_DAY progression-40 consumer lands once.
  **5043 torch — a6's use arm** (the faithful `builder_acc_status[11]
  = 0` setter, supersedes c36's fold), **f33's event machine + flag
  reads**, c36's helper — the three-way split; this doc's §2b torch
  LINE carries whichever arm's flags. **3335/5289/pet-licenses/drones
  — a6 keeps** (138-FINAL §10.1-10.3: g87's three arms DROP — the
  start-refusal gap, the spawn-walk gaps, the live-scan shape).
- **The g87-triple-flag (138-FINAL §10.4-10.5).** g87 `dfbcaf94b`
  (pushed 07:19) wrote a `.trplayer` v7 `dh <extra> <cart>` sentinel
  line + a `.trwld` v18 3-byte bought-pet tail + `.wld` gate-217 flips.
  138-FINAL's verdicts: the **`dh` sidecar line DROPS** (version
  collision + the L1-only ruling — `extraAccessory`/super-cart are
  f44's `.plr` fields, 122 §4.4 "no sidecar line"); the `.wld`
  bought-pet trio **dedups onto f45** (the complete flags block); the
  `.trwld` pets **ride the union at offsets 57-59** (§L3-DEFINITIVE
  D3 — ahead of g90's book pair, vanilla write order
  `WorldFile.cs:1392-1394` < `:1407-1408`; 138's old 19-21 seats and
  this doc's old 52-54 reconciliation are superseded). The SEAL pass
  adds 132-RETIRE §2e's newer counter-proposal (the `dh` fold as line
  17 at v12 — §2a's version-seat update + §L3-DEFINITIVE D5); the
  DROP below is the 138-era record, the merger picks per D5.
  **This branch's v11 reader tolerates a g87 `dh` file
  without corruption** — verified in code this pass: `dh 0 0` is a
  3-token line, the shape walk's `_ => {}` arm (`persist.rs:565`)
  keeps the defaults (the two flags are LOST, not misparsed — exactly
  138 §10.4's "silently loses both flags" census; no hp/flag
  corruption). The DROP verdict means no `dh` recognition is owed;
  if the merger ever wants the live-session carriers before f44's
  hydration lands, 138 §10.4 routes them to "g88's NEXT version
  (v11+) as new tokens — never a parallel v7": v11 is spent (§2b), so
  that seat is **v12+ alongside token 7** — a new §6-rule-5 LINE or
  tokens 8/9, NOT the `dh` sentinel.
- **The `.plr` halves** (g90b's 260+ unlock bools both halves +
  legacy-279 indices) stay g90b's scope — this branch's
  `player_file.rs` is untouched by the fold (the sidecar is a different
  file, §6 rule 3); at L1 they dedup onto f44's `used_upgrades[6]`
  (132 §3-L1, 138 §2's index map 0=crystal/1=fruit/2=arcane/3=pearl/
  4=worm/5=ambrosia — the same file order this §2a cites).

`loadout.rs` is byte-identical to g55's (blob `a647ae55`), so the merge
dedupes it cleanly. `persist::save`'s signature carries one extra arg
(`equip_share_flags: u32`, g55 Ph2's shape); all call sites pass it.

## 6. Merge guidance

1. Keep THIS `persist.rs` (writer v11 + the tolerant reader) over any
   lane's single-purpose version — it is the strict superset. g89d's
   research/122 §4 recipe (g89 v9 base whole, then g88 pairs/clamps,
   then g90 flag, then crystal tokens) resolves to this same reader:
   the superset already carries every layer (the crystal tokens landed
   at v10, the f33 torch pair at v11); at merge,
   g89's/g90b's/a6's/f33's use arms/booster rows/`.plr`
   halves land beside the carriers (the dup adjudication §5a; the full
   ordered checklist is §9).
2. Keep the field declarations at the spots in §5 — every lane declared
   its fields at the same anchor, so the merge dedupes without semantic
   conflicts.
3. A lane's `.plr` (`player_file.rs`) changes are INDEPENDENT of this
   sidecar contract (different file, vanilla format); merge those per
   their own gates (256 bread, 260 unlocks, 262 loadouts, 322 favorited).
4. New unlock flags: §2a's reserved tokens — append, never renumber;
   the crystal tokens 4/5/6 LANDED at v10 with the 3-token v9 back-read
   (research/122 §3.1/§4.4, §7's receipt); token 7 (DD2) bumps **v12**
   with the 6-token v11 back-read (v11 was spent on the torch LINE,
   §2b). World globals (combat-book
   vol-two 5336, peddler's satchel 5343) NEVER ride line 8 — they are
   `.wld` flags, a separate lane (g90b `bb78974a`'s `.trwld` v18 tail).
   Non-unlock per-player fields (the torch pair) get their own LINE
   per rule 5, never an unlock token (§2b's adjudication).
5. New post-dye lines: append AFTER the loadout block, give the line a
   self-identifying shape (token count distinct from 1/2/41, or a
   sentinel), extend the reader's shape walk, add a back-compat row here
   and a predecessor witness.

## 7. The v10 crystal fold receipt (this branch)

Folded on `task/W4-g88-life-fruit-use` atop the v9 superset (`6998a4ec`).
Inputs read: g90b `e76ee133` (the four Player flags + consumers + the
`.plr` halves + the v7 five-line block) and a6 `f23385e5` (the six use
arms + consumers, no sidecar). research/135/138 NOT pushed at fold time
(both pushed since — §8's alignment note + §5a's CONFIRMED adjudication
are the receipts of record).

- **The writer bumps 9 → 10**: the unlock line widens to SIX tokens —
  `usedAegisFruit usedGummyWorm usedAmbrosia usedAegisCrystal
  usedArcaneCrystal usedGalaxyPearl`. Tokens 1-3 keep g89c's fruit-first
  positions verbatim; tokens 4/5/6 append in the verified vanilla
  relative order (`Player.cs:54781`/`:54783`/`:54784` — read at
  `:54776-54787`). The pre-v10 reservation map's pearl/arcane/crystal
  listing is corrected to crystal/arcane/pearl (the tokens were never
  written, so no back-compat cost).
- **The reader**: the positional arm extends to `"8" | "9" | "10"` with
  `unlock.get(3/4/5)` → the crystal trio; the v9 three-token line
  back-reads all three false (witness
  `a_three_token_unlock_line_reads_under_v10` +
  `the_v9_superset_still_loads_without_the_crystals`); token 7+ stays
  skipped (`reserved_unlock_tokens_are_skipped`, re-scoped to v10).
- **The carriers**: `Player.used_aegis_crystal`/`used_arcane_crystal`/
  `used_galaxy_pearl` declared at g90b's exact spots (after
  `used_ambrosia`, ctor `:56078`/`:56080`/`:56081` false), restored in
  `app/mod.rs` beside the aegis/gummy/ambrosia triple BEFORE the bread
  line; the pearl mirror (`update_luck` seeds
  `LuckState.used_galaxy_pearl`, g90b's shape) lands with its witness
  (`w4_g88_v10_the_galaxy_pearl_feeds_the_luck_recalculation`). The
  consumers (regen rows, booster rows, use arms) stay g90b/a6's — this
  branch carries the flags only (the carrier precedent, §5).
- **Witnesses added/updated** (`persist.rs::tests`, written not run —
  code-only): `the_v9_superset_still_loads_without_the_crystals`,
  `a_three_token_unlock_line_reads_under_v10`,
  `the_six_token_unlock_line_reads_under_v10`,
  `g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread`; the full
  round-trip now asserts the `10\n` header + the `1 1 1 1 1 1` unlock
  line; `round_trips_player_state` carries all six flags; the v5/v6
  back-compat assertions extend to the crystal trio.
- **Cross-refs**: research/122 §7 (the g89-addendum fold receipt),
  §5a (the a6 dup adjudication — CONFIRMED by the pushed 138), the
  brief's research/135 (g90c fold-input, unpushed at v10 fold time —
  pushed since, §8's alignment note).

## 8. The research/135 alignment note (the token-order receipt)

g90c's research/135 (`b3b82d07`, `origin/task/W4-g90-artisan-bread-use`,
read FULL at the v11 pass) is the mechanical fold-input this superset
consumed one pass late: the v10 crystal fold ran on the landed code
shapes because 135 was unpushed then. This section records the
receipt-alignment, item by item:

1. **The token-order conflict — the C#-verified v10 order WINS.**
   135 §3 recommends the pre-v10 reservation map's pearl=4 / arcane=5 /
   crystal=6 ("103 §2a wins"). The v10 fold had ALREADY corrected that
   map to crystal=4 / arcane=5 / pearl=6 — the vanilla `.plr` RELATIVE
   order among the trio, verified against `Player.cs:54776-54787`
   (`:54781` usedAegisCrystal → `:54783` usedArcaneCrystal → `:54784`
   usedGalaxyPearl; re-verified this pass). 135 itself pins the
   decisive fact: **"No pushed writer has ever emitted a token 4+"** —
   neither order carries a back-compat obligation, the choice is free.
   The pushed v10 writer emits crystal/arcane/pearl; the C#-verified
   order is the better free choice (it matches the `.plr` byte order
   the merger's L1 array walk uses, f44c's `used_upgrades[6]` indices
   0/2/3), and re-ordering NOW would break the pushed v10 files for
   zero gain. **The live map stays §2a's: 4=aegisCrystal, 5=arcane,
   6=pearl.** 135 §3's mapping table column "UNION token (recommended)"
   is superseded by this note for rows 4/5/6; its rows 1/2/3/7 agree
   verbatim. f44c's research/132 §2b rule 3 cites 135's recommendation
   (pearl=4) — that citation is one pass stale; 132's own §2b table
   rows already say "RESERVED→g90 fold" without pinning an order, so
   only the rule-3 parenthetical needs the merger's attention. No
   back-compat cost either way (135's own words), so this is a
   documentation alignment, not a format conflict.
2. **The six-line v7 recognition — LANDED.** 135 §2's rule (exactly
   six 1-token lines → positional bread/crystal/arcane/pearl/gummy/
   ambrosia) is in the v7 reader verbatim (§3), with the witness
   `g90bs_six_line_v7_loads_positionally` replacing the retired
   bounded-misread witness. 135 §2's requested back-compat row is in
   §4. The corruption 135 §2 documented (bread→aegis, last-line-wins)
   no longer occurs on this reader.
3. **The rest of 135 consumed as published**: §1's line/writer/reader
   tables match the folded shapes; §3's dedup list matches §5a;
   §4's 21-byte `.trwld` union is the g71c/g70 merge's scope (L3 —
   this branch's `save.rs` untouched, §9's checklist carries it);
   §5's consumer census (the town ladder's b3 seam) stands as g90's
   recorded remainder; §6's merge checklist is subsumed by §9 below.
4. **The g90-FINAL receipt (tip `94d33f2ca`, 08:18 — read this pass).**
   135's latest commit is the §7 reconcile addendum + §5 census
   refresh; the doc's standing status is MECHANICAL FOLD-INPUT (its
   header), and its §7b probe predates this branch's v10/v11 folds
   ("135 NOT YET consumed ... writer stays v9"). **This receipt closes
   that gap:** the v10 fold consumed 135 §1's fields/lines and §2's
   rule (§7), the v11 fold landed the rule in code (§3). Witness-
   verified against g90's ACTUAL pushed writer this pass
   (`git show origin/task/W4-g90-artisan-bread-use:crates/tr-app/src/persist.rs`):
   the writer emits bread then `[used_aegis_crystal, used_arcane_crystal,
   used_galaxy_pearl, used_gummy_worm, used_ambrosia]` each as its own
   `1\n`/`0\n` line after dye — EXACTLY the six-line shape the
   `g90bs_six_line_v7_loads_positionally` witness constructs
   (`persist.rs:1198-1225`: bread=1/crystal=1/arcane=0/pearl=1/
   gummy=0/ambrosia=1, all six asserted positionally, aegis false,
   pairs None). **The g90b shape loads LOSSLESSLY under v11** — the
   §7b "not consumed" state is superseded; 135 §7a's keep-one proposal
   (a6 keeps its exclusives, g90 keeps carriers + consumers + the town
   feeder) is adjudicated by the NOW-PUSHED 138 (§5a's updated
   receipt). 135 §3's token-order recommendation stays superseded per
   item 1 above — 138 §6 independently reached the same verdict from
   the pushed writer ("the written order is binding").

## 9. The FULL merger persist-checklist (FINAL-FINAL — receipts complete, with the 132/135/138 cross-refs)

The complete ordered checklist for the persist merger. Layer numbers
are f44c's research/132 (`05e4020b`): L1 `.plr` / L2 `.trplayer` /
L3 world. This doc owns L2; 132 owns the cross-layer ownership rules.

**L1 — `player_file.rs` (132 §3-L1):**
1. Base = f44c's complete walk (`05e4020b`: the 71-row research/99
   table, G1-G7, the three shim depths, the `d331ea7c` timestamp fix).
2. Fold g55's loadout semantics onto f44c's `LoadoutData` transport
   (byte layout identical, 132 §3-L1 rule 2).
3. Dedup g89's three + g90's five named unlock bools onto f44c's
   `used_upgrades[6]` (indices: fruit 1, gummy 4, ambrosia 5, crystal
   0, arcane 2, pearl 3 — the `.plr` file order, NOT the sidecar token
   order); dedup f33's named torch fields onto f44c's IDENTICAL named
   fields (same names/slots — textual dedup, §2b rule 4); dedup g90's
   `ate_artisan_bread` onto f44c's G7 field (132 §2b rule 2).
4. Keep the timestamp fix whole; the g-lanes' `last_saved_binary: 0`
   session literals dedup onto `utc_now_binary()` (132 §3-L1 rule 4).
5. Re-pin the g-lanes' `.plr` witnesses onto the array fields
   (slot-value assertions — mechanically portable, 132 §3-L1 rule 5).

**L2 — `persist.rs` (this doc; 132 §3-L2):**
6. Base = THIS branch's tip (writer **v11**, the tolerant v7-v11
   reader, 135 §2's six-line rule + the f33 torch recognitions
   landed). 132 §3-L2's "base = g88 `6998a4ecb` v9" is one pass stale
   — the same branch, two folds later; the recipe's steps 2-3 (extend
   the v7 arm, activate tokens 4/5/6) are DONE (§7/§8).
7. Dedup g90's `SavedPlayer` flag fields + App restores onto the
   superset's (all folded); dedup f33's pair fields + accessors +
   restore onto the superset's (§2b/§5 — the accessors are at f33's
   exact spot).
8. The use-arm dups per §5a (NOW ADJUDICATED by the pushed 138
   `d7de8bd53` + its worktree FINAL revision): g88 keeps
   `life_fruit.rs`/`crystals.rs` (138 §1 confirms keep-own verbatim);
   the merger drops a6's three powerups dups + c36's
   `use_permanent_powerup`; the shimmer six take **a6's
   `use_shimmer_permanent` machine** (138 §1 — supersedes this
   checklist's old g89-signature preference) with g90's Player-field
   declarations + consumer shapes as the anchors (a6's five fidelity
   fixes landed, 138-FINAL §10.7); ONE `use_combat_book` (138
   recommends a6 or g89e); 5326 g90 keep-own; 5043 a6's arm + f33's
   event machine + c36's shared `complete_condition` helper; g87's
   3335/5289/license arms DROP (138-FINAL §10.1-10.3). The a6-5340
   pearl-field flag is CLOSED (138 §3.2's fix landed).
9. Token 7 (DD2) is the only reserved unlock token; a DD2 lane bumps
   v12 (§6 rule 4) — the §2a census confirms NO pushed lane claims it
   (d124 owns the WORLD tier latches, not the player token; the C#
   field is a derived mirror, `:24450-24453`). g87's `dh` sentinel
   line DROPS (138-FINAL §10.4); its fields, if ever wanted live, ride
   v12+ as new tokens/lines, never a parallel v7. The torch line's
   conditional drop-recommendation is §2b rule 3 — revisit ONLY when
   the character-load hydration AND the exit-save both land (132 §2c's
   precedence: sidecar wins as the newer state).

**L3 — world (132 §3-L3; NOT this branch's files):**
10. `.trwld`: the v18 union — **§L3-DEFINITIVE below is THE layout:
    a 107-byte fixed head + W×H dense**, re-derived from the C#
    `WorldFile.cs` write order over the full 13-branch claimant
    census. This step's old "reconciled 100-byte head" (132-FINAL's
    97 + g87's 3 pets) is SUPERSEDED: it missed the c39⨉f39 moon
    quintet (5 B, ONE deduped block), the g71 underworld seat (1 B,
    FIRED — rolled + `.wld`-persisted on g71's pushed tip) and
    inherited 132-FINAL's 97-vs-98 arithmetic slip (16+31+3+1+1+2+44
    = 98, c39 research/114 §12b). c39's research/114 §8 106-byte head
    (identity FIRST) carries the same block set minus underworld;
    §L3-DEFINITIVE = 106 + 1 with the block ORDER re-derived from the
    C# `.wld` rows (empress `:1395` AFTER pets `:1392-1394`, not
    before; underworld `:1376` AFTER the moon quintet, not before
    crimson; books `:1407-1408` LAST of the flags). The three live
    layouts (114 §8's 106, 132-RETIRE §3-L3's 107, 92 §1-FULL's 107)
    agree on the corrected total; §L3-DEFINITIVE adjudicates their
    order divergences — the shared frame (forest/cave/surface prefix,
    identity closes the head, dense LAST) stands, the 13-byte flag
    middle rides STRICT C# row order. Every claimant's truncation
    constants re-pin to **107** (`v18_tail_len` = `107 + width*height`;
    92 §1-FULL.4's per-branch site table is the merge arithmetic).
    VERSION stays 18, forward-only (no released v18 — every claimant
    is unmerged task work). The g71/g72 roll keep-one (132-RETIRE
    §3-L3 rule 2 + 92 §2.1-FINAL: g72's `cave_bg.rs` owns the cave
    roll + persistence trio, g71's surface rolls + wiring stay) is
    unchanged. L2 (this doc's writer/reader) is independent of L3.
11. `.wld` flags: ONE book-two byte (gate 259, tail 2036) + ONE
    satchel byte (gate 260, tail 2037) — **f45's complete flags block
    is the base** (138 §4's byte-compare: a6/g90/g89e/f45 write the
    SAME bytes at the SAME offsets; g87 joins the bought-pets trio).
    The bought-pets trio (gate 217, tail 2021-2023): a6 ⨉ f45 ⨉ g87
    all write it — KEEP f45, a6/g87 dedup (138-FINAL §10.5; g87's
    `World.bought_*` field names identical). d124's DD2 tier trio
    (gate 178, tail 1931-1933) is independent bytes, no conflict.
    World globals NEVER ride L2 (§6 rule 4, 132 §2b rule 4).

**Cross-layer (132 §2):**
12. One writer per file per save; one slot/token per field; the
    dual-carrier fields (132 §2b's table + this doc's §2b torch pair)
    keep BOTH carriers until the exit-save lands; the character-load
    restore runs FIRST, the sidecar restore SECOND (132 §2c).
13. e10's voice fields: transport f44c's (research/99 row 67),
    consumers e10's — the `u8`→`i32` hydration seam, no conflict
    (132 §4).

**Verification status:** code-only lane — every witness in §4 WRITTEN,
NOT RUN. The v11 pass touched `persist.rs` (writer/reader/witnesses),
`player.rs` (the two f33 accessors), `app/mod.rs` (the torch restore);
`player_file.rs`/`save.rs`/`wld/` untouched (§6 rule 3, the L1/L3
bases are other lanes' files).

**RECEIPTS COMPLETE (the FINAL-FINAL pass, 2026-11-21).** Every
external persist receipt this contract consumes is now read at its
latest pushed tip and aligned:

| Receipt | Tip read | Alignment seat | Verdict |
| --- | --- | --- | --- |
| f44 research/132 FINAL | `bdd1cc79a` (08:22) | §1a | recognition rule SAME (value-range = all-0/1); seat SAME line-16 geometry, version delta v10→v11 reconciled (v11 binding, pushed); slot-11 property semantics verified compatible (`player.rs:10652-10660` = the C# getter/setter); 132's stale points listed |
| g90 research/135 + §7 addendum | `94d33f2ca` (08:18) | §8 items 1-4 | six-line rule LANDED + witness-verified against g90's ACTUAL pushed writer (bread + crystal/arcane/pearl/gummy/ambrosia, each `1\n`); token-order recommendation superseded by the written order (138 §6 concurs); §7b's "not consumed" closed |
| a6 research/138 + worktree FINAL §10 | `d7de8bd53` (08:06) + in-tree revision | §5a, §9 steps 8-11 | g88 keep-own 29/109/1291 CONFIRMED; shimmer six → a6's machine (supersedes the g89-signature preference); g87's `dh` line DROP (v11 reader tolerates it lossy-not-corrupt, verified `persist.rs:565`); g87-triple-flag → f45 `.wld` dedup + union offsets 19-21 (superseded by §L3-DEFINITIVE D3's 57-59); no `g87b-143` branch exists (138-FINAL §10's census, re-verified `git branch -r`) |
| d124 DD2 claims | `56fa087e3` (2026-11-20 22:29) | §2a's v12-contract-seat census | d124 owns the WORLD tier latches (`.wld` tail 1931-1933, gate 178) + starter drops + victory slice; NO `.plr`/`.trplayer`/`save.rs` diff; token 7 UNCLAIMED, stays RESERVED → v12; DD2-buy gates are progression (hardmode/mech/golem), not the player bool |

The 103 ⨉ 132 cross-receipt is UNIFIED: §1a is the L2-side record of
132 §2d's f33 input (both docs now describe the same line-16 seat, the
same value-range recognition, the same slot-11 setter restore, the
same conditional dual-carrier drop); the only deltas are version-seat
(v11 binding) and the three stale points §1a item 5 lists for the
merger. No code change this pass — the v11 writer/reader/witnesses
stand as landed (`a9f1c29d8`); this pass is doc receipts only.

## L3-DEFINITIVE — the `.trwld` v18 union, re-derived from the C# write order (SEAL pass, 2026-11-21)

**THE definitive L3 layout: 107-byte fixed head + W×H dense LAST.**
This section supersedes step 10's old 100-byte arithmetic AND both
consolidators' current 107 layouts' flag-middle orders. Derivation
method (the brief's): enumerate EVERY v18 claimant from the pushed
branch list, read each one's actual `save.rs` writer bytes, then lay
the deduped block set out in the C# `WorldFile.cs` write order —
`SaveWorldHeader` `:1221-1236` then `SaveWorldFlags` `:1238-1430`,
re-read directly from `decompiled/Terraria/IO/WorldFile.cs` this pass.

### D1. The claimant census — 13 branches at VERSION 18, 11 distinct blocks

Sweep: every `origin/task/W4-*` tip's `crates/tr-core/src/save.rs`
grepped for `pub const VERSION` (13 hits ≠ 17), each claimant's
writer/reader diff read in full. **The c39⨉f39 question answered
first: the two quintets are the SAME five fields, byte-identical,
ONE deduped 5-byte block.** c39 `7dd785dc5` (`save.rs:437-446` write
/ `:831-837` read) and f39 `bd5a5a24a` (`save.rs:438-447` write /
`:832-837` read) both append `downed_halloween_king,
downed_halloween_tree, downed_christmas_ice_queen,
downed_christmas_santank, downed_christmas_tree` in that vanilla row
order (`WorldFile.cs:1347-1351`; IceQueen before Santank before Tree
— the SAVE rows, not the `NPC.cs:79210-79238` death-arm order). The
branches share NO ancestry (`git merge-base --is-ancestor` both
directions = 1) — independent parallel ports of the same five rows,
not inheritance. Their `.wld` halves are also field-identical
(both replace the same five `w.boolean(false)` rows at flags.rs
:147-151 with the same five `world.*` reads). f39 ADDITIONALLY owns
the `.wld` force-today pair (`forceHalloweenForToday`/`forceXMasForToday`,
`:1386-1387`, gate 212 — f45 `9c1a01a42` explicitly defers those
bytes); that pair is `.wld`-ONLY, **zero `.trwld` bytes** — f39's
resolved seasons re-derive at load/dawn (`World.refresh_seasons`,
`Main.cs:65389-65391`, `WorldFile.cs:651-652`), witnessed by its own
"the season flags re-derive, not persist" arm (`save.rs:1557-1566`).
Death-arm keep-one: f39's season-lane `DoDeathEvents` owns the
semantics (132-RETIRE §3-L3's adjudication); c39's exec arms dedup.

| # | Lane / tip (re-read this pass) | Block | Bytes | C# write anchor |
| --- | --- | --- | --- | --- |
| 1 | g70 `752655914` | forest: `tree_x` 3×i32 + `forest_bg_styles` 4×u8 | 16 | `:1253-1255` + `:1318`/`:1377-1379` (treeBG1..4) |
| 2 | g72 `4a75715ef` | cave: `cave_back_x` 3×i32 + `cave_back_style` 4×**i32** + ice/jungle/hell 3×u8 | 31 | `:1260-1269` (styles are C# `int`; ice/jungle/hell C# `int` compressed u8, port-owned range 0..2) |
| 3 | g71 `f2d8c2c67` | surface: snow/desert/ocean 3×u8 | 3 | `:1321`/`:1324-1325` |
| 4 | g69 `20d0380f9` | `crimson` u8 | 1 | `:1281` (`:3507` corruption fallback) |
| 4d | e19 `e7e466d46` | crimson DUP — adopted g69 verbatim (`0570e7136`: "tr-core diff vs g69 tip: empty, keep-one g69-canonical") | 0 | same |
| 5 | c39 `7dd785dc5` ⨉ f39 `bd5a5a24a` | moon quintet 5×u8, ONE deduped block | 5 | `:1347-1351` |
| 6 | g71 `f2d8c2c67` | underworld seat `underworld_bg_style` u8 — FIRED: rolled (`bg_rolls.rs` setBG 9) + `.wld`-persisted (`flags.rs:186`/`:577`), no `.trwld` byte yet | 1 | `:1376` |
| 7 | g87 `dfbcaf94b` | pets: `bought_cat/dog/bunny` 3×u8 | 3 | `:1392-1394` |
| 8 | d125 `d8047a109` | `downed_empress_of_light` u8 | 1 | `:1395` (`NPC.cs:6393`, case-636 `:79351-79353`) |
| 9 | g90 `740650d75` | books: `combat_book_volume_two_was_used` + `peddlers_satchel_was_used` 2×u8 | 2 | `:1407-1408` |
| 10 | c43 `076970809` ≡ f45 `6352d3840` | identity: `generator_version` u64 + `unique_id` 16×u8 + `world_id` i32 + `creation_time` i64 + `last_played` i64, byte-identical blocks — keep ONE | 44 | `:1225-1227` + `:1250-1251` |
| 11 | d109 `81894095e` | dense `World::variants` W×H×u8, port-only, LAST | W×H | `Tile.cs:464-482` (never in the `.wld` tile stream) |

**13 branches, 3 duplicate pairs (e19≡g69, f39≡c39, f45≡c43), g71
claims two blocks → 11 distinct blocks. Fixed-head sum:
16+31+3+1+5+1+3+1+2+44 = 107.** The old published totals all miss
claimants or mis-add: 103-step-10's 100 (= 97+3) inherits f44-FINAL's
97-vs-98 slip AND misses the quintet + underworld; c39 §8's 106 seats
everything except underworld; 132-RETIRE and 92 §1-FULL both reach
107 with the right block set (order adjudicated in D3).
`downedQueenSlime`/`downedDeerclops` (`:1396-1397`) are NOT claimants
— queen-slime already rides the v15 byte on main (`save.rs:416`),
deerclops unclaimed by any tip (c39 §14's d43r seed).

### D2. The C#-write-order derivation (the brief's re-derivation)

Walking `WorldFile.cs` top-to-bottom and seating each claimed field
at its C# position: identity's five fields ride FIRST in the C#
(`SaveWorldHeader:1225-1227` before `SaveWorldFlags` even starts,
timestamps `:1250-1251` before every flags row) → forest `:1253-1259`
→ cave `:1260-1269` → crimson `:1281` → surface `:1321-1325` → moon
quintet `:1347-1351` → underworld `:1376` (with treeBG2..4 `:1377-1379`
bridged into g70's block) → pets `:1392-1394` → empress `:1395` →
books `:1407-1408`. That pure walk is c39 §8's 106 (+underworld = 107)
with identity FIRST. Both consolidators instead keep the **64a
forest-first frame**: multi-byte bg blocks contiguous at the head
(g70's treeBG1/treeBG2-4 split bridged inside its 16 bytes, g72's
cave verbatim), crimson riding after the surface group, identity —
a header-family group with no flags-relative seat — closing the
fixed head, dense LAST. That frame is a PUBLISHED two-doc consensus
with stable prefix offsets (64a's [0..50] block boundaries), and the
deviation cost is identical either way (the bg-family regrouping
already breaks strict row order). **Adjudication: keep the shared
frame; order the 13-byte flag middle by STRICT C# row order** — the
one rule both consolidators invoke but neither fully applies:

- 132-RETIRE seats underworld BEFORE crimson (`:1376` before `:1281`
  — inverted) and books last of flags: correct on books, wrong on
  underworld.
- 92 §1-FULL seats books at [51..53] to keep 64a's 53-byte prefix
  stable (`:1407-1408` before `:1347-1351` — inverted), then moon →
  underworld → pets → empress: correct on the extension group,
  wrong on books. Its own order-rule text says "remaining claims
  append in C# `.wld` relative order" — books violate it.
- The definitive middle: crimson `:1281` → moon `:1347-1351` →
  underworld `:1376` → pets `:1392-1394` → empress `:1395` → books
  `:1407-1408`. Forward-only, zero back-compat cost (no released
  v18); 92's 53-byte-prefix-stability argument protects offsets of
  an UNMERGED proposal — the same argument 103 §8 used to move the
  crystal tokens. The merger re-pins truncation constants ONCE to
  107 regardless of middle order, so strict row order costs nothing
  extra and removes the last "which note's order" ambiguity.

### D3. THE layout (offsets relative to the v18 block start)

```
… v17 downedFishron byte (1)
v18 UNION tail — fixed head (107 bytes) + dense variants (W×H bytes):
  [0..12]    tree_x[0..3]         3×i32 LE ┐ g70 forest (16 B,
  [12..16]   forest_bg_styles     4×u8     ┘  :1253-1255/:1318/:1377-1379)
  [16..28]   cave_back_x[0..3]    3×i32 LE ┐ g72 cave (31 B, :1260-1269;
  [28..44]   cave_back_style      4×i32 LE │  caveBackX[3] NEVER persisted)
  [44]       ice_back_style       u8       │  (C# int → u8, range 0..2)
  [45]       jungle_back_style    u8       │
  [46]       hell_back_style      u8       ┘
  [47]       snow_bg_style        u8       ┐ g71 surface (3 B,
  [48]       desert_bg_style      u8       │  :1321/:1324-1325)
  [49]       ocean_bg_style       u8       ┘
  ---- the flag middle: STRICT C# row order ----
  [50]       crimson              u8 bool    g69 (:1281; e19 dup drops)
  [51]       downed_halloween_king        u8 ┐ c39⨉f39 quintet (5 B,
  [52]       downed_halloween_tree        u8 │  :1347-1351, ONE deduped
  [53]       downed_christmas_ice_queen   u8 │  block, save-row order)
  [54]       downed_christmas_santank     u8 │
  [55]       downed_christmas_tree        u8 ┘
  [56]       underworld_bg_style  u8          g71 FIRED seat (:1376)
  [57]       bought_cat           u8 bool ┐ g87 pets (3 B, :1392-1394)
  [58]       bought_dog           u8 bool │
  [59]       bought_bunny         u8 bool ┘
  [60]       downed_empress_of_light u8 bool  d125 (:1395 — AFTER pets)
  [61]       combat_book_volume_two_was_used u8 ┐ g90 books (2 B,
  [62]       peddlers_satchel_was_used       u8 ┘  :1407-1408, flags LAST)
  [63..71]   generator_version    u64 LE   ┐
  [71..87]   unique_id            16×u8    │ c43≡f45 identity (44 B,
  [87..91]   world_id             i32 LE   │  :1225-1227 + :1250-1251;
  [91..99]   creation_time        i64 LE   │  header-family group closes
  [99..107]  last_played          i64 LE   ┘  the fixed head)
  [107..107+W*H]  variants        W×H×u8     d109 dense, LAST
                                             (variants_from_save=true)
```

**Fixed head = 107 bytes: 16+31+3 +1+5+1+3+1+2 +44.
Total tail = 107 + width*height.** VERSION stays 18, forward-only.
`v18_tail_len(buf)` returns `107 + width*height`; every claimant's
truncation constants re-pin to 107 (92 §1-FULL.4's per-branch site
table is the merge arithmetic — the site lists stand, only the
middle-offset assertions inside round-trip witnesses move per D3).

Deltas vs the three live layouts (the broadcast content, D5):
- vs **c39 114 §8 (106)**: +1 underworld at [56]; identity moves
  FIRST→LAST (the 64a frame both consolidators published); empress
  moves before-pets→after-pets (`:1395` > `:1394` — 114 §8 seats
  empress [100] before licenses [101..104], inverting the C# rows);
  crimson moves after-cave→after-surface (the frame's group rule).
  Quintet CONTENT/order unchanged (114 §8's rows are correct).
- vs **132-RETIRE §3-L3 (107)**: underworld [50]→[56] (it sat before
  crimson, inverting `:1376` > `:1281`); crimson [51]→[50]; moon
  [52..57]→[51..56]; pets [57..60], empress [60], books [61..63],
  identity [63..107] all unchanged. ONE move: underworld leaves the
  pre-crimson seat and joins the row-ordered middle.
- vs **92 §1-FULL (107)**: books [51..53]→[61..63] (the 64a-prefix
  concession dropped); moon [53..58]→[51..56]; underworld [58]→[56];
  pets [59..62]→[57..60]; empress [62]→[60]; crimson [50] unchanged;
  identity [63..107] unchanged; forest/cave/surface [0..50]
  unchanged. The 53-byte-prefix-stability goal is retired with the
  unmerged 64a proposal.

### D4. The `.wld` side (unchanged verdicts, re-pinned)

f45's flags file is the base (138 §5, 132-RETIRE §3-L3 rule 3):
book-two gate 259 tail 2036, satchel gate 260 tail 2037, pets gate
217 tails 2021-2023 — a6/g87/g89e/g90 flips dedup onto f45. The five
moon-boss rows (v131+): c39⨉f39⨉f45 field-identical — keep f45's
writer half, ONE World-field set, f39's death arms own the semantics.
The force-today pair (gate 212, tails 2002-2003): **f39 ONLY** (f45
`9c1a01a42` defers). g71's underworld/mushroom/treeBG2-4 flips +
g72's cave arrays (i32 widths) ride f45's superset — keep one codec
half, g72's arrays must survive it. d124's DD2 tier trio (gate 178,
tails 1931-1933) independent. World globals NEVER ride L2 (§6 rule 4).

### D5. Broadcast + receipt state (the consolidation-active lanes)

This section is the g88 definitive-layout INPUT to the two active
consolidators, read at their CURRENT pushed tips this pass:

| Consolidator | Tip read | Their state | The input they take |
| --- | --- | --- | --- |
| f44 research/132 RETIRE | `d35876af9` (FINAL-FINAL merger-bible) | 107 head, TEN-claimant table (its #7 counts c39⨉f39 as one row), underworld-before-crimson order | D3's middle re-order (underworld [50]→[56], moon −1); D1's e19-dup + 13-branch census (its table + 92's registry agree on blocks); its 2e g87-`dh`-line-17-at-v12 fold is L2 input — see §2a's v13 note below |
| g71 research/92 §1-FULL | `f2d8c2c67` (V18 FINAL CONSOLIDATION) | 107 head, 13-branch registry (the same census this pass re-derived independently), books-in-prefix order | D3's middle re-order (books [51..53]→[61..63], the extension group slides −2); its registry's tip column has three stale hashes (e19 `d7de8bd5` is a6's tip — e19's actual tip `e7e466d46`; d125 `07697080` is c43's — d125's actual `d8047a109`; c43/f45 `e7e466d4` is e19's — c43's actual `076970809`; the BLOCK attributions are all correct, only the hash column slipped) |

Both consolidators' block sets, dedup verdicts (e19≡g69, f39≡c39,
f45≡c43), roll keep-one (g72 cave / g71 surface) and dense-LAST rule
are CONFIRMED by this independent re-derivation. The ONLY layout
delta is the flag-middle order (D3). If either lands a later revision
first, the merger takes D3's strict-row-order middle — the C# rows
are the tie-breaker, not publication timestamp.

**L2-side receipt from the same read (132-RETIRE §2e):** f44's RETIRE
now proposes folding g87's `dh` sentinel as **line 17 at v12** (the
sentinel recognition + the DD2 version reservation moving to v13),
superseding 138-FINAL §10.4's DROP verdict this doc's §5a recorded.
That is a MERGE-time L2 fold (this branch's v11 writer stands as
landed — no code change this pass); the version-seat consequence is
recorded in §2a: token 7's activation bump moves v12→**v13** IF the
merger takes the dh fold. Both options are published; the merger
picks one (132-RETIRE's fold is the newer input and keeps g87's two
unlocks lossless — recommended).

## L3-ARBITER FINAL — the 107-byte-order arbitration (2026-11-21, post-SEAL)

**VERDICT: D5 CONFIRMED — g88's strict-C#-row middle IS the C# save
sequence; g71 92 §1-FULL.2's books-in-prefix middle is OVERRULED.**
Arbitration authority (this pass's brief): *the C# ACTUAL write order is
DEFINITIVE — the port must match the C# save format for cross-compat.*
That directive supersedes the Lead-designation / prefix-stability /
publication-timestamp rationale g71's BG SEAL (`b706737a1`) recorded
for its own order. The ONE definitive layout is D3's table, restated
in A3 below — no byte moves from the SEAL.

### A1. The C# evidence — direct line-by-line re-read (definitive)

Read directly this pass from `decompiled/Terraria/IO/WorldFile.cs`,
`SaveWorldHeader` `:1221-1236` → `SaveWorldFlags` `:1238-1430`. Every
disputed field's `writer.Write(...)` call, quoted verbatim with its
line number, in EXACT sequential order:

```
:1225  writer.Write(Main.ActiveWorldFileData.WorldGeneratorVersion);   ┐ identity
:1226  writer.Write(Main.ActiveWorldFileData.UniqueId.ToByteArray());  │ (header,
:1227  writer.Write(Main.ActiveWorldFileData.WorldId);                 │ 44 B with
:1250  writer.Write(Main.ActiveWorldFileData.CreationTime.ToBinary()); │ the two
:1251  writer.Write(DateTime.UtcNow.ToBinary());                       ┘ timestamps)
:1281  writer.Write(WorldGen.crimson);                       ← crimson
:1347  writer.Write(NPC.downedHalloweenKing);                ┐ moon
:1348  writer.Write(NPC.downedHalloweenTree);                │ quintet
:1349  writer.Write(NPC.downedChristmasIceQueen);            │ (save-row
:1350  writer.Write(NPC.downedChristmasSantank);             │  order)
:1351  writer.Write(NPC.downedChristmasTree);                ┘
:1376  writer.Write((byte)WorldGen.underworldBG);            ← underworld
:1392  writer.Write(NPC.boughtCat);                          ┐ pet
:1393  writer.Write(NPC.boughtDog);                          │ licenses
:1394  writer.Write(NPC.boughtBunny);                        ┘
:1395  writer.Write(NPC.downedEmpressOfLight);               ← empress
:1407  writer.Write(NPC.combatBookVolumeTwoWasUsed);         ┐ books
:1408  writer.Write(NPC.peddlersSatchelWasUsed);             ┘ (vol2+satchel)
```

Order-proof interstitials (non-claimant writes BETWEEN the disputed
fields — they add zero `.trwld` bytes but pin the relative order):
`:1352-1374` towers/party/sandstorm/bartender + `DD2Event.Save(writer)`
between quintet and underworld; `:1375` mushroomBG + `:1377-1379`
treeBG2-4 (bridged into g70's forest block) + `:1380`
`writer.Write(NPC.combatBookWasUsed)` — the FIRST combat book, already
`.wld`-owned (`wld/sections/flags.rs:191`, read `:572`), NOT a `.trwld`
claimant, so the books block stays 2 B — + `:1386-1387` force-today
pair (f39, `.wld`-ONLY) + `:1388-1391` ore tiers between underworld
and pets; `:1396-1406` queenSlime/deerclops/unlocked-spawns between
empress and books. **Journey/bread carriers: NONE in this region** —
`Main.GameMode` `:1240` is the only journey-adjacent world write (the
mode int, pre-v18, not a claimant); the journey/bread unlock flags are
Player fields riding `.plr`/`.trplayer` (L1/L2), zero `.trwld` bytes
(g90's own SEAL `43fee29a4` states the same).

### A2. Adjudication — which layout matches the C# sequence?

Relative order of the seven claimed flag groups in the C# stream:
**crimson `:1281` < moon `:1347-1351` < underworld `:1376` < pets
`:1392-1394` < empress `:1395` < books `:1407-1408`.** (Identity rides
the header + timestamps; both layouts seat it LAST of the head under
the shared 64a frame — an already-published, order-agnostic regrouping
both consolidators invoke, unchanged by this arbitration.)

- **g88 D5/D3 middle** — crimson[50] → moon[51..56] → underworld[56] →
  pets[57..60] → empress[60] → books[61..63]: **matches the C#
  sequence EXACTLY, field-for-field, including empress AFTER pets
  (`:1395` > `:1394`) and books LAST of flags (`:1407-1408`).**
- **g71 92 §1-FULL.2 middle** — crimson[50] → books[51..53] →
  moon[53..58] → underworld[58] → pets[59..62] → empress[62]:
  **INVERTS the C# rows** — it seats books (`:1407-1408`, the LAST two
  flag writes of the whole function) BEFORE the moon quintet
  (`:1347-1351`), underworld, pets and empress. Its own §1-FULL.2 text
  concedes the inversion ("C# order, before books there; … group to
  keep 64a stable"). The 64a-stable-prefix rationale protects offsets
  of an UNMERGED proposal; under this pass's brief the C# order is
  definitive, so the concession is overruled.
- **f44 132-RETIRE middle** — underworld[50] before crimson[51]:
  inverts `:1376` > `:1281` — also non-matching (already superseded by
  D3; unchanged here).

NEITHER consolidator's prefix argument survives the cross-compat
directive: `.trwld` is forward-only (no released v18 — zero back-compat
cost either way, both docs agree), and the C# rows are the one
external, verifiable tie-breaker. **D5 stands as published.**

### A3. THE ONE definitive offset table (D3 restated — no byte moves)

```
v18 UNION tail — fixed head (107 bytes) + dense variants (W×H bytes):
  [0..16]    g70 forest   tree_x 3×i32 + forest_bg_styles 4×u8   (:1253-1255/:1318/:1377-1379)
  [16..47]   g72 cave     cave_back_x 3×i32 + cave_back_style 4×i32
                          + ice/jungle/hell 3×u8                (:1260-1269)
  [47..50]   g71 surface  snow/desert/ocean 3×u8                (:1321/:1324-1325)
  ---- flag middle: STRICT C# row order (A1) ----
  [50]       crimson                        u8   (:1281)
  [51..56]   moon quintet                   5×u8 (:1347-1351, king/tree/icequeen/santank/xmastree)
  [56]       underworld_bg_style            u8   (:1376)
  [57..60]   pets cat/dog/bunny             3×u8 (:1392-1394)
  [60]       downed_empress_of_light        u8   (:1395 — AFTER pets)
  [61..63]   books vol2/satchel             2×u8 (:1407-1408 — LAST flags)
  [63..107]  identity 44 B: generator_version u64 + unique_id 16 + world_id i32
                          + creation_time i64 + last_played i64 (:1225-1227/:1250-1251)
  [107..107+W*H]  d109 dense variants W×H×u8, LAST
```

Head sum 16+31+3+1+5+1+3+1+2+44 = **107**; `v18_tail_len` =
`107 + width*height`; VERSION 18 forward-only. Per-field detail and
the delta lists vs c39-106 / 132-RETIRE / 92 §1-FULL are D3's (above)
— all unchanged. 92 §1-FULL.4's per-branch truncation-site table
stands (the sites are order-agnostic); only the middle-offset
assertions inside round-trip witnesses follow A3, not §1-FULL.2.

### A4. Cross-receipt — the five re-pin dependents (notify table)

Published here as the standing receipt; each lane re-pins its
witnesses/constants to A3 (the D3 table) — tips read at this pass:

| Lane | Tip read | Re-pin action |
| --- | --- | --- |
| **92** g71 (`origin/task/W4-g71-bg-layers-2`) | `b706737a1` (BG SEAL, §1-SEAL.1 "64a-stable prefix WINS") | Its ORDER ADJUDICATION is OVERRULED by this arbitration's C#-definitive brief; §1-FULL.2's writer/reader snippet moves books [51..53]→[61..63], moon [53..58]→[51..56], underworld [58]→[56], pets [59..62]→[57..60], empress [62]→[60]. Registry/census/dedups/§1-FULL.4 site table all stand. Its own §1-SEAL.1 already records this exact move as "free" — this arbitration activates it |
| **132** f44 (`origin/task/W4-f44-save-plr-2`) | `d35876af9` (RETIRE) | underworld [50]→[56], crimson [51]→[50], moon [52..57]→[51..56] (D3's one-move delta); its L1/L2 authority + §8 merger-bible steps unchanged |
| **64b** g72 (`origin/task/W4-g72-bg-layers-3`) | `b069439eb` (retire) | 64b's underworld [58] seat → [56]; the 64a 53-byte-prefix offsets are RETIRED with the unmerged proposal (books leave [51..53]); its eleven truncation sites re-pin 53→107 per §1-FULL.4 regardless; cave-array + roll keep-one verdicts unchanged |
| **138** a6 (`origin/task/W4-a6-crystals-powerups`) | `8fad5555d` (POWERUP SEAL §12) | its §12 107-authority alignment receipt re-pins to THIS arbitration (A3 = D3, no byte change from what it receipted IF it took D3; if it followed 92's b706737a1 order, apply the 92-row move above) |
| **135** g90 (`origin/task/W4-g90-artisan-bread-use`) | `43fee29a4` (BOOSTER SEAL) | ALREADY on A3 — its seal explicitly took g88 `fc5cf11ae` D1-D5, books [61]/[62], journey/bread = L1/L2 not `.trwld`; its six truncation sites +107 stand. CONFIRMED, no action |

The merger seats A3's table VERBATIM. If any lane lands a later
revision first, the C# rows (A1) remain the tie-breaker per this
arbitration's brief — not publication timestamp, not designation.

## SEAL — the persist-family merger-bible (one paragraph, three layers)

**The whole persist merge in one paragraph:** merge **L1 → L2 → L3**.
**L1** (`.plr`, `player_file.rs`) takes **f44's complete walk** as the
base (research/132-RETIRE `d35876af9` §3-L1 / research/99's 71-row
table — the slot-11 `UsingBiomeTorches` PROPERTY fix `63204f85` and
the `d331ea7c` timestamp fix included, e10's voice transport stays
f44's `u8` with e10's picker/threading folding via an as-cast); every
g-lane's named bools dedup onto the array fields (`used_upgrades[6]`
indices crystal 0 / fruit 1 / arcane 2 / pearl 3 / worm 4 / ambrosia 5
— the `.plr` file order, NOT the sidecar token order), f33's torch
named fields dedup textually, g55's loadout semantics ride f44's
identical `LoadoutData` transport. **L2** (`.trplayer`, `persist.rs`)
takes **THIS branch's v11 superset** (`a9f1c29d8`, research/103 §2/§3
— the tolerant v7-v11 reader with the value-range life-pair guard,
the six-line positional rule and the lone-torch-pair rule; unlock
tokens 1-6 LIVE fruit-first, token 7 DD2 RESERVED; the torch pair is
line 16, NOT tokens) — g89/g89b/g89c/g90/g90b/f33's sidecar writers
and SavedPlayer fields all dedup onto it; use arms per §5a (g88 keeps
29/109/1291, the shimmer six take a6's `use_shimmer_permanent`, ONE
`use_combat_book`, 5326 g90, 5043 a6+f33+c36); g87's `dh` line either
DROPS (138-FINAL) or folds as line 17 at v12 (132-RETIRE §2e,
recommended — DD2's bump then moves v13); dual-carrier fields keep
both carriers until character-load hydration AND the exit-save land,
restore order character-first/sidecar-second (132 §2c). **L3**
(`.trwld` + `.wld`) takes **§L3-DEFINITIVE D3's 107-byte head + W×H
dense LAST** (13 branches → 11 blocks after the e19≡g69 / f39≡c39 /
f45≡c43 dedups; forest 16 + cave 31 + surface 3, then the STRICT-C#-row
flag middle crimson/moon-quintet/underworld/pets/empress/books = 13,
identity 44 closes the head; every truncation constant re-pins to
107, `v18_tail_len` = 107+W×H, VERSION 18 forward-only); the `.wld`
flags file is **f45's superset** with the D4 dedups (books/satchel/
pets/moon-quintet keep f45's writer, the force-today pair f39 ONLY,
g72's cave arrays survive, d124's DD2 trio independent); the cave/
surface roll keep-one is g72's `cave_bg.rs` + g71's surface rolls
(92 §2.1-FINAL). World globals never ride L2; one writer per file per
save; every witness in the family is WRITTEN NOT RUN (all lanes
code-only) — the merger runs the catalog test command once, after
the three layers land.