# 122 — `.trplayer` v9 addendum: the sidecar contract AFTER dye (supersedes the v7 contract)

Status: AUTHORITATIVE for the live-session `.trplayer` sidecar (`crates/tr-app/src/persist.rs`).
Written by W4-g89 (branch `task/W4-g89-aegis-fruit-use`, tip lineage `920014cb` → `9bf03ede` → `78766306`).
This note is the ADDENDUM the brief asked to hang off research/103 (`103-persist-v7-layout.md`,
g88c in-flight — that file does not exist in ANY pushed ref at probe time, so the addendum lives
here and 103 should link it rather than restate it). **v9 SUPERSEDES every v7-layout contract**:
g88's superset-v7 (`717e0640`) and g90's bread-v7 (`1f1b88f0`) were written against g89's FIRST
commit (`920014cb`, v7 = one flag token); g89 has since landed v8 (gummy) and v9 (ambrosia) on the
SAME line. All three lanes must build ON TOP of this layout, not re-fork v7.

## 1. The exact file layout (v9 writer, g89 HEAD)

One line per row, `\n`-terminated, tokens space-separated (the writer leaves a trailing space on
the inventory / kills / dye lines — `split_whitespace` on read tolerates it):

| # | Line | Tokens | Since |
|---|------|--------|-------|
| 1 | version | `9` | v1 (number bumped per added line) |
| 2 | position + hp | `x y hp` (f32 f32 i32) | v1 |
| 3 | inventory | `kind count` ×50 (u16 u32), trailing space | v2 (v1: ≤10 slots) |
| 4 | coins | `coins` (u64; absent in v1 → 0) | coin-line era |
| 5 | armor | `head body legs` (u16 ×3) | v4 |
| 6 | bestiary kills | `count (id count)*`, trailing space | v5 |
| 7 | dye | `kind` ×10 (u16), trailing space | v6 |
| 8 | **permanent shimmer unlocks** | `aegis worm ambrosia` — three 0/1 tokens | **v7/v8/v9** |

Line 8 is THE contested line. Its token order is the vanilla `.plr` unlock-block order
(`Player.cs:54782` usedAegisFruit → `:54785` usedGummyWorm → `:54786` usedAmbrosia; reads
`:55256`/`:55259`/`:55260`, release ≥260 block `:55253-55261`):

- **v7 file**: line 8 has ONE token (`aegis`). g89 `920014cb`.
- **v8 file**: line 8 has TWO tokens (`aegis worm`). g89 `9bf03ede`.
- **v9 file**: line 8 has THREE tokens (`aegis worm ambrosia`). g89 `78766306` (current).

The v9 reader (`persist.rs` load, version arm `"3"|"4"|"5"|"6"|"7"|"8"|"9"`) reads line 8 with
`tokens.next().is_some_and(|t| t == "1")` per flag: **short lines back-read** — a v7 one-token line
loads worm/ambrosia false, a v8 two-token line loads ambrosia false. v3–v6 files have no line 8 at
all (all three false). This back-read shape is the contract's core: **append-only tokens on line 8,
never reorder, never re-version for a token append alone** (the version number bumps, but old
numbers stay readable).

## 2. The two in-flight v7 forks (what they wrote, why they must rebase)

Both forked from g89 `920014cb` BEFORE v8/v9 existed:

- **g88 superset-v7** (`origin/task/W4-g88-life-fruit-use` `717e0640`): after dye, THREE lines —
  `aegis` (1 token, g89's position kept), then `statLifeMax statLifeMax2` pair, then
  `statManaMax statManaMax2` pair. Reader dispatches on TOKEN COUNT over all post-dye lines
  (1-token line = flag; first 2-token line = life pair, permanent half clamps 500; second = mana
  pair, clamps 200 — `Player.cs:55221-55229`). It reads g89's flag-only v7 correctly **but not v8/v9**:
  a v9 three-token line would be parsed as a life pair (`1` `0` → base 1) — CORRUPTION on merge if
  kept as-is. Its `Player.used_aegis_fruit` declaration is at g89's exact spot (clean merge dedup);
  `app/mod.rs` applies both pairs BEFORE the hp clamp.
- **g90 bread-v7** (`origin/task/W4-g90-artisan-bread-use` `1f1b88f0`): after dye, ONE line —
  `ateArtisanBread` 0/1 (`Player.cs:54779` write / release-259 read; ctor default `:56077`).
  Reader `if version == "7"` reads that line as the bread flag. Against a g89 v7 file it would
  MISREAD the aegis flag as bread — same collision class, opposite direction.

## 3. The reconciled contract (what g88c / g90b must build ON TOP)

ONE post-dye region, ordered lines, version **10+** for the next writer:

1. **Line 8 (unchanged from v9)**: the shimmer-unlock token line `aegis worm ambrosia`.
   Crystal bools (`usedAegisCrystal` `:54781`/`:55255`, `usedArcaneCrystal` `:54783`/`:55257`,
   `usedGalaxyPearl` `:54784`/`:55258` — a6's claimed 5337/5339/5340 lane) **append as tokens 4/5/6
   on this line → v10+**, in vanilla `.plr` order. Do NOT give them their own line; do NOT reorder
   the first three. (Vanilla order interleaves crystal/fruit — the sidecar's established order is
   fruit-first from g89 v7; keep it, append the crystals after ambrosia, and record the deviation
   here when a6 lands.)
2. **Line 9 (g88)**: the life pair `life_max_base max_hp` (2 tokens, permanent half read-clamps 500).
3. **Line 10 (g88)**: the mana pair `mana_max_base mana_max` (2 tokens, clamps 200).
4. **Line 11 (g90)**: the bread flag `ate_artisan_bread` (1 token).

Reader rules for the merged writer (v10+):

- Keep g89's line-8 token-count back-read EXACTLY (v7=1, v8=2, v9=3 tokens; missing → false).
- g88's token-COUNT dispatch over post-dye lines survives ONLY for `version == "7"` files (its
  three historical shapes: g89 flag-only, g88 life-only, g88 superset). For version ≥8 the region
  is POSITIONAL: line 8 = unlock tokens, then optional pair lines, then optional flag line —
  dispatch pairs vs flags by token count WITHIN the positional walk (2 tokens = a maxima pair in
  life→mana order; 1 token AFTER the pairs = bread). A g90 bread-only v7 (one 1-token line) is
  ambiguous with g89 aegis-only v7 by construction — the merged reader must pick ONE interpretation
  for bare v7 (recommend: aegis, g89's position was first) and document the loss; both lanes'
  writers move to v10+ so the ambiguity dies with the rebases.
- Missing lines keep fresh defaults (`None` pairs / false flags) — never fail the load.
- `app/mod.rs` restore order stays: maxima pairs BEFORE the hp clamp, flags after (g88's shape).

## 4. Merger persist merge-order (the recipe)

1. **Base = g89 v9** (`78766306` lineage): `persist.rs` with the 3-token line-8 reader/writer,
   `player.rs` three unlock bools, `player_file.rs` `.plr` 260+ block (six bools, three consumed-
   dropped for a6), `app/mod.rs` flag restore. Take these files WHOLE from g89.
2. **Then g88's additions**: `SavedPlayer.life_max`/`mana_max` `Option<(i32,i32)>` fields, the two
   pair lines appended AFTER line 8, the `version=="7"` token-count legacy arm, the 500/200 read
   clamps, `Player.life_max_base` (mana_max_base pre-existed), the mod.rs pair restore before the
   hp clamp. Dedup `used_aegis_fruit` (g88 declared it at g89's spot — identical, keep one).
   REBASE its writer version to the merged number (10+), and extend its v7 legacy arm per §3.
3. **Then g90's additions**: `SavedPlayer.ate_artisan_bread`, the flag line appended after the
   pairs, `Player.ate_artisan_bread` + `.plr` `:54779` slot, the mod.rs restore. Same version rebase.
4. **Then a6/g87 crystal tokens** (when they land): tokens 4/5/6 on line 8, v10+ bump, back-reads
   for the 3-token v9 line. g87's demon-heart lane touches NO sidecar line (`extraAccessory`
   `:54776` rides the `.plr` only — verify at merge).
5. Anchors: keep ALL g89 rows (`use_aegis_fruit`/`use_gummy_worm`/`use_ambrosia` in
   `world_sim/editing.rs`); the persist witnesses of every lane must survive the merge — the
   v7/v8 short-line back-read tests are the regression net for this contract.

## 5. Cross-references

- Vanilla unlock block: `Player.cs:54776-54786` (write), `:55253-55261` (read, release ≥260).
- Vanilla maxima: `:54773`/`:54775` (write), `:55220-55229` (read + clamps).
- `.wld` world flags are a SEPARATE lane (binary, version-gated bytes — `WorldFile.cs`); the
  combat-book vol-two flag (`:1407` write / `:2416-2423` read, version ≥259) is world-owned,
  NOT a sidecar token — see `wld/sections/flags.rs`. Never put world globals on line 8.
- g88 tip at probe: `717e0640`; g90 tip: `1f1b88f0`; a6 tip: `ea478936` (claims 5337/5339/5340
  consumed-dropped in g89's `.plr` reader — the a6 merger restores those three reads).

## 6. Receipt — W4-g89i lands the crystal token (v10, 2026-11-21)

The §3.1 crystal-token append landed on THIS branch (g89) as **v10**: the
unlock line widens to the six-token g88-v10 binding order (research/138 §6
adjudicated the written order crystal=4 / arcane=5 / pearl=6, overriding
103 §2a's old pearl-first reservation). g89 carries **token 4 only**
(`used_aegis_crystal`, the 5337 Aegis Crystal — its +0.2 lifeRegenTime
consumer is the g90 lane-7 `regen.rs` `aegis_crystal` slot, research/138
§3.3); tokens 5/6 (arcane/pearl) write `0` placeholders — their Player
fields + consumers stay the a6/g90 lanes'. The v7/v8/v9 short-line
back-reads are unchanged (a v9 three-token line loads the crystal false).
The `.plr` FIRST unlock bool (`:54781`/`:55255`) is now read+written
(previously consumed-dropped). At the merge: the sidecar dedups onto g88
v10's superset writer (research/138 §2/§8.2 — g88 owns tokens 1-6 + the
pairs + bread + loadouts; g89's token-4 reader/writer is the same shape,
keep g88's), the `.plr` bool onto f44's `used_upgrades[6]` index 0, the use
arm onto a6's `use_shimmer_permanent` machine. The §4.4 merge-order step
"then a6/g87 crystal tokens" is satisfied for the crystal by this commit;
arcane/pearl remain a6/g90's.

## 7. FINAL-FINAL receipt — the g88g-v10-token-4 SAME-slot check + the aegis-family declaration (W4-g89j, 2026-11-21)

Probe: `origin/task/W4-g88-life-fruit-use` tip **`a9f1c29d8`** (persist fold 7
FINAL — writer **v11**, the f33 torch fold; fold 6 `c8985b534` landed the v10
crystal trio), `origin/task/W4-a6-crystals-powerups` tip **`115ce655a`**
(research/138 FINAL, `138-powerup-master-reconcile.md` pushed),
`origin/task/W4-g90-artisan-bread-use` tip **`94d33f2ca`** (g90b `e76ee133e`
in its lineage, `git merge-base --is-ancestor` exit 0).

### 7a. The token-4 double-writer question — ANSWER: SAME slot, SAME order, NO conflict; g88's writer is the keep

Both branches write **line 8 token 4 = `usedAegisCrystal`** (the 138 §6
binding order crystal=4 / arcane=5 / pearl=6, vanilla `.plr` relative order
`:54781`/`:54783`/`:54784`):

- **g89 (this branch, `3c7adb0b6`)**: writer `persist.rs:104-110` —
  `"{} {} {} {} 0 0\n"` (tokens 1-4 live, **tokens 5/6 hardcoded `0`
  placeholders** — the arcane/pearl Player fields do not exist here);
  reader `:189-200` reads tokens 1-4 positionally, ignores 5/6 (the iterator
  drops them); version gate `"7"|"8"|"9"|"10"`.
- **g88 (`a9f1c29d8`)**: writer `persist.rs:260-267` — the full SIX-token
  line, all six from live Player fields; reader `:440-456` reads
  `unlock.get(3)/get(4)/get(5)` for the trio; the version gate is
  `"8"|"9"|"10"|"11"` and the writer is v11 (the torch pair appended LAST,
  after the loadout block — NOT an unlock token).

So YES, this is a **SAME-slot double-writer** in the branch-collision sense —
but NOT a semantic conflict: same line, same token position, same flag, same
0/1 encoding. The g88 superset is a strict **superset of this branch's
writer** (g89's `0 0` placeholders are exactly what g88 writes when
arcane/pearl are false). **Merger verdict (138 §2/§8.2 + 103 §9-L2 step 6):
take g88's `persist.rs` WHOLE (v11 writer + the tolerant v7-v11 reader);
DROP g89's v10 writer/reader.** No token remap, no version negotiation: a
g89-written v10 file (`10\n` header, six-token unlock line with `0 0` tail)
loads on g88's reader with crystal correct + arcane/pearl false — byte-level
compatible.

**One bounded cross-read loss (pre-existing, NOT introduced by the fold):**
g88's reader cannot load a g89 v10 file's LINE COUNT beyond the unlock line —
g89 v10 has NO pair/bread/loadout lines, and g88's shape walk keeps fresh
defaults for missing lines (103 §3 "missing pieces keep their fresh
defaults"), so the load is lossless for everything g89 wrote. The reverse
(g89's reader on a g88 v10/v11 file) DOES lose the post-unlock lines (g89's
reader stops after line 8 and its version gate rejects `"11"`) — irrelevant
at the merge because g89's reader is dropped, but it means **until the merge,
a save written by the g88 branch must not be loaded by a g89 build** (the
`"11"` version falls into the migrate arm's reject path). Recorded as the
standing pre-merge incompatibility; the merge kills it.

### 7b. g88e-folded-crystal-trio (from g90b) — this branch's write DEDUPES

g88 fold 6 (`c8985b534`) folded g90b `e76ee133e`'s four shimmer Player flags
(`usedAegisCrystal`/`usedArcaneCrystal`/`usedGalaxyPearl` + the ambrosia
overlap) into the superset: the Player declarations at g90b's exact spots
(after `used_ambrosia`, ctor `:56078`/`:56080`/`:56081`), the App restore
beside the aegis/gummy/ambrosia triple, the pearl `LuckState` mirror. This
branch's `used_aegis_crystal` declaration (g89i, the same spot after
`used_ambrosia`) + the App restore + the `.plr` `:54781`/`:55255` halves are
the SAME field at the SAME anchor — **textual dedup at the merge, keep one**
(103 §9-L1 step 3: all named bools dedup onto f44c's `used_upgrades[6]`,
crystal = index 0; 138 §2 L1). The g89i consumer (the `regen.rs`
`aegis_crystal` slot BETWEEN the base +1 and the crimson fold, the f32
accumulation order) is the canonical shape **a6 adopted** (138 §3.3 — "a6
now matches every claimant consumer shape") and g90b owns (`e76ee133e`);
three copies of one observation-identical slot — **keep g90's declaration +
consumer shape as the merge anchor** (138 §1 verdict), drop g89's and a6's
dups. No semantic conflict anywhere in the trio fold.

### 7c. The aegis-family declaration — ALL LANDED (5338/5341/5342/5337)

| Item | Flag | Use arm (this branch) | Consumer | `.plr` | `.trplayer` |
|------|------|----------------------|----------|--------|-------------|
| 5338 Aegis Fruit | `used_aegis_fruit` | `editing.rs::use_aegis_fruit` (g89 `920014cb`) | +4 defense row `:12853-12856` | `:54782`/`:55256` both halves | line 8 token 1 (v7) |
| 5341 Gummy Worm | `used_gummy_worm` | `use_gummy_worm` on the shared core (g89b `9bf03ede`) | +3 fishing (i35, pre-existing) | `:54785`/`:55259` | token 2 (v8) |
| 5342 Ambrosia | `used_ambrosia` | `use_ambrosia` (g89c `78766306`) | pick/tile/wall ±0.05 row `:12861-12866` — boosters COMPLETE 3/3 | `:54786`/`:55260` | token 3 (v9) |
| 5337 Aegis Crystal | `used_aegis_crystal` | `use_aegis_crystal` (g89i `3c7adb0b6`) | +0.2 lifeRegenTime, `regen.rs` f32-order slot `:19112-19120` | `:54781`/`:55255` | token 4 (v10) |

All four use arms ride the shared `use_shimmer_permanent_item` core (the
once-ever `!used*` gate + ApplyItemTime 45 + the consumable pay + TurnToAir);
all four `.plr` bools read+write; all four sidecar tokens live with the
v7/v8/v9 short-line back-reads. **Merger per 138/103** (the FINAL adjudications,
both pushed): the four use arms dedup onto **a6's `powerups.rs::
use_shimmer_permanent`** (138 §1 — the single six-arm machine; 103 §9-L2
step 8 confirms, superseding its old g89-signature preference); the Player
fields + consumers anchor on **g90's shapes** (a6 matches); the `.plr` bools
dedup onto **f44c's `used_upgrades[6]`** (103 §9-L1 step 3, indices crystal 0
/ fruit 1 / gummy 4 / ambrosia 5 — the `.plr` file order, NOT the token
order); the sidecar onto **g88's v11 superset** (§7a). The remaining two
cluster arms (5339 Arcane Crystal / 5340 Galaxy Pearl) stay a6/g90's — this
branch writes their `0` placeholders and never claims them.

### 7d. FINAL-FINAL status

research/122 is **FINAL**: §1-§5 the v9-era contract (historical, superseded
piecewise by the landed v10/v11), §6 the g89i token-4 receipt, §7 this
double-writer check + the family declaration. The authoritative live layout
is **g88's research/103 (FINAL, §9 the full merger persist-checklist)**;
this doc remains the v9-addendum lineage record + the g89-side receipts.
The merger inputs are complete: 138 (use arms/consumers/`.wld`/`.trwld`),
103 §9 (L1/L2/L3 checklist), 132 (cross-layer master map, f44c), this §7
(the g89-side token-4 dedup receipt). No open g89 persist question remains.