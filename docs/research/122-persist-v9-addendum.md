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