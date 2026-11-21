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

---

## 6. Fold receipt (W4-g88 superset branch — `task/W4-g88-life-fruit-use`)

Appended by the g88 v9 fold (commit `afa371f1`, superset lineage `717e0640` →
`e078d42b` → `a8b5299a` → `a07c83a0` → `62c255e0` → `2ceec691` → `afa371f1`).
The g89 text above is kept VERBATIM as the authoritative addendum; this
section records how the superset satisfies it.

- **§2's corruption warning is historical for this branch.** It describes
  the `717e0640` token-count reader. The superset's fold-3 reader
  (`a07c83a0`) ALREADY consumed the first post-dye line POSITIONALLY for
  v8; the v9 fold extends that same positional arm to `"8" | "9"` with
  token 3 → ambrosia. A v9 three-token line is never parsed as a life
  pair on this branch. The token-COUNT walk survives only for
  `version == "7"` (the three historical v7 shapes), exactly §3's rule.
- **§3 is satisfied at v9, not v10+ — deliberately.** §3's "version 10+
  for the next writer" counts the crystal tokens 4/5/6 as the next
  append. The superset's writer stays **9** (g89c's number) with the
  SAME 3-token line 8 plus the pairs/bread/loadout lines behind it —
  the superset's extra lines are shape-dispatched, not renumbered, so
  the version number tracks the unlock-line width only. When the
  crystal lane (g90b/a6) lands tokens 4/5/6, the superset bumps **v10**
  with the 3-token v9 back-read, per §3.1/§4.4. Reserved map after this
  fold: tokens 4/5/6 = `usedGalaxyPearl`/`usedArcaneCrystal`/
  `usedAegisCrystal` (vanilla `.plr` order `:54784`/`:54783`/`:54781` —
  the fruit-first deviation §3.1 records is already the sidecar's
  established order), token 7 = `downedDD2EventAnyDifficulty`.
- **The bare-v7 ambiguity resolution matches §3's recommendation**: the
  lone 1-token flag line reads AEGIS (g89's position was first), witness
  `g90s_bread_only_v7_loads_into_the_first_flag_slot`; documented in
  research/103 §3.
- **§4's recipe**: the superset took g89's line-8 shape + short-line
  back-reads + the flag-carrier declarations at g89's exact spots
  (`used_aegis_fruit` after `used_gummy_worm`; `used_ambrosia` after
  `used_aegis_fruit`, ctor `:56083`), then layered g88's pairs/clamps/
  legacy arm, g90's bread line and g55's loadout block. The g89 lane's
  USE arms (`use_aegis_fruit`/`use_gummy_worm`/`use_ambrosia`), booster
  rows (`:12853-12866`) and `.plr` halves stay g89's scope (the
  superset's carrier precedent, research/103 §5) — at merge, g89's rows
  land beside the superset's carriers with no semantic conflict.
- **§5's world-flag boundary honoured**: the combat-book vol-two flag
  (item 5336, `NPC.combatBookVolumeTwoWasUsed`, `Player.cs:45337-45349`;
  `.wld` `:1407`/`:2416-2423`) is NEVER a line-8 token. Census verified
  against `ItemID.cs:12355-12369`: 5336 CombatBookVolumeTwo, 5337
  AegisCrystal, 5338 AegisFruit, 5339 ArcaneCrystal, 5340 GalaxyPearl,
  5341 GummyWorm, 5342 Ambrosia, 5343 PeddlersSatchel; LifeFruit is 1291
  (`ItemID.cs:4265`), NOT 5343.
- **Witnesses added by the fold** (`persist.rs::tests`, written not run —
  code-only): `g89cs_unlock_only_v9_still_loads` (g89c's own 3-token
  writer), `a_two_token_unlock_line_reads_under_v9` (the v8 back-read),
  `the_v8_superset_still_loads_without_ambrosia` (this branch's own
  pre-fold v8 superset file), the re-scoped
  `reserved_unlock_tokens_are_skipped` (7-token line under v9: tokens
  1-3 live, 4+ skipped) and the updated full-block round-trip asserting
  the `9\n` header + the `1 1 1` unlock line.

---

## 7. The v10 crystal fold receipt (W4-g88 superset branch)

Appended by the g88 v10 fold atop `6998a4ec` (the v9 superset lineage
§6). The g89 text above (§§1-6) is kept VERBATIM; this section records
how the superset satisfies §3's v10 step now that the crystal lane has
landed.

- **§3.1 is satisfied — the crystal tokens landed as tokens 4/5/6.**
  g90b `e76ee133` (`origin/task/W4-g90-artisan-bread-use`) landed the
  four Player flags + consumers + the `.plr` 260+ halves; a6 `f23385e5`
  (`origin/task/W4-a6-crystals-powerups`) landed ALL six use arms
  (`powerups::use_shimmer_permanent`) + the regen/luck/booster
  consumers with no sidecar writer. The superset writer bumps **v10**:
  the unlock line is now `aegis worm ambrosia aegisCrystal arcaneCrystal
  galaxyPearl` — the first three UNCHANGED (g89c's fruit-first order),
  the crystal trio appended in the vanilla relative order verified
  against `Player.cs:54776-54787` (`:54781` crystal → `:54783` arcane →
  `:54784` pearl). The fruit-first deviation §3.1 records stands; the
  pre-v10 reservation map's pearl-first listing is corrected (the
  tokens were RESERVED, never written — no back-compat cost).
- **§3's reader rules kept**: the line-8 token-count back-read extends
  one step — a v9 three-token line loads the crystal trio false
  (witnesses `a_three_token_unlock_line_reads_under_v10`,
  `the_v9_superset_still_loads_without_the_crystals`); the v7
  token-COUNT walk is unchanged (g90b's bread+five-flag-line v7 block
  loads with the bounded lone-flag misread, witness
  `g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread` —
  research/103 §3's ambiguity table); missing lines keep fresh
  defaults; the `app/mod.rs` restore order stays maxima-before-hp-clamp
  with the three new flag restores beside the aegis/gummy/ambrosia
  triple.
- **§4.4's crystal step done**: the carriers are
  `Player.used_aegis_crystal`/`used_arcane_crystal`/`used_galaxy_pearl`
  at g90b's exact declaration spots (after `used_ambrosia`); the pearl
  persistence owner is the Player field with g90b's `update_luck`
  mirror into `LuckState.used_galaxy_pearl` (a6's use arm writes the
  LuckState field directly — the merger reconciles per research/138,
  unpushed at fold time; research/103 §5a records the keep-own
  adjudication: g88 keeps its `life_fruit.rs`/`crystals.rs` arms, the
  merger drops a6's three dups per 138).
- **§5's anchors**: g89's use-arm rows stay g89's; the `.plr` halves
  stay g90b's (`player_file.rs` untouched by this fold); the world-flag
  boundary stands (g90b `bb78974a`'s `.trwld` v18 two-byte tail carries
  the book-two/satchel world flags, never line 8).
- **research/135/138 cross-refs**: the brief's g90c fold-input
  (research/135) and a6b reconcile (research/138) were NOT pushed on
  either lane's tip at fold time (verified `git ls-tree`); this receipt
  proceeds on the landed code shapes and flags both for the merger.