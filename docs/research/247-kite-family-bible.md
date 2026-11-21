# 247 — KITE-FAMILY BIBLE (SEALED): d16 canonical 151-160, the compose map

W4-d16 SEAL phase, branch `task/W4-d16-proj-ai-151-160` (DONE base
`41080e159`, main base `a725acb04`). Code-only lane: no cargo/tests/
rustfmt/graphify invoked. The single merger-facing record for the kite
family and its lane neighbours — the per-item detail lives in
`246-d16-research-final.md` + ANALYSIS "W4-d16 kite residuals"; this
bible is the index the merger reads FIRST (the `91-fighter-family-bible`
precedent).

## 1. d16 is canonical for aiStyles 151-160

Merge `task/W4-d16-proj-ai-151-160` — it carries the whole band:

| piece | landing | C# |
| --- | --- | --- |
| 151 SuperStar (728) | `projectile_ai_misc.rs::ai_super_star` + the row | `Projectile.cs:22138-22168` / `:7659-7671` |
| 152 SuperStarSlash (729/977/976/1045) | `ai_super_star_slash` + the four rows | `:22230-22351` / `:7672-7687`/`:9208-9224`/`:9270-9286`/`:9890-9907` |
| 153 ToiletPop (733) | `ai_toilet_pop` + the row | `:52198-52236` |
| 154 VoidLens (734) | `ai_void_lens` + the row + the pool dedupe | `:52154-52196` |
| 155 MysticSnakeCoil (754) | `ai_mystic_snake_coil` + the row + the LIVE rope seams | `:51939-52152` |
| 159 PaperAirplanes (761/762) | `ai_paper_airplanes` + the rows + the Kill recovery drop (4343/4344, SELF-ARMED cue) | `:48695-48755` / `:76083-76095` |
| 160 Kites (the 24 DefaultToKite kinds) | `projectile_ai_kites.rs::kite_step` + the rows + kite_ctx/visual_scene | `:48421-48693` |
| the kite use-path | `world_sim/editing/kites.rs` (LMB/RMB/channel/holdStyle/refusals) | `Player.cs:49437`/`:43537-43541`/`:42792-42794`/`:52858-52874`/`:52059` |
| the afk auto-kite | `kites.rs` counters/scan/override + the `world_sim/player.rs` seam | `Player.cs:25497-25514`/`:12843-12905`/`:17421-17428` |
| the 723 anchor chain | `kite_anchor_place_hook` + `tr-core/edit/kill.rs::anchor_teardown` | `Player.cs:40144` → `TEKiteAnchor`/`TELeashedEntityAnchorWithItem`; `WorldGen.cs:63691-63694`/`:63873-63880` |
| the kite string DRAW | `kite_string.rs` + `Projectile::draw_kite` + the Extra sheets | `Main.cs:37134-37569`/`:28054-28058` |
| the TrailCacheLength ring | `Projectile::trail_len`/`trail_cache_length`/`TRAIL_RING_CAP`/`reset_trailing_rings` | `Projectile.cs:718-735` / `ProjectileID.cs:521-522` |
| the wind interplay | the `in_a_place_with_wind` SKIP fix (BOTH copies) + the WindForVisuals hover | `WorldGen.cs:86858-86882` / `Projectile.cs:48520-48565` |
| the Item192 reel cue | the case-2 192 no-draw arm + the `Item_192` bank + the pitchOffset seam | `LegacySoundPlayer.cs:628-630` / `Projectile.cs:48508` |

The 156 BatOfLight and 157/158 are NOT d16's (the residual-7 census pins
156 = `AI_156_BatOfLight` `:51516` as the band boundary; whether its arm
is rowed is the owning lane's claim, not d16's). The 161 shortsword
census (residual 5) is d17's rowing INPUT — the rows stay UNROWED on d16.

## 2. The compose map (who owns what at the merge)

| neighbour | tip | relation | merger instruction |
| --- | --- | --- | --- |
| **f20** (proj-ai-verify-2) | `0883a80bc` | F7 widened `projectile_ai_moonlord.rs::step` to four args + the charm tail; d16 `4d870ea1c` edits the SAME file (the trail_len settled-scan + witness fill) | The ONE merge friction. Take f20's four-arg signature + charm tails; keep BOTH d16 trail_len lines (`p.old_pos[1..p.trail_len]` scan, `p.old_pos[..p.trail_len].fill` witness); the 11 two-arg witness calls + the dispatch call convert to f20's shape (f20's FIRST-COMPILE-FIX-LIST, `research/26f` F8). d16's `Projectile` ring fields live in `projectile.rs` — keep-both, they survive. BUILD lane executes; neither lane pre-converts (it would break its own compile). |
| **d77** (proj-sounds-3) | `5f10560b2` | `0573b70f2` arms the 728/729 Item10 death cues on `kill_tail_cue`; d16 owns the rows + the 151/152 arms + the SELF-ARMED 761/762 cue | Keep d77's `728 \| 729` map row (one play per death); d16 does NOT self-arm the pair. Keep d16's 761/762 death-hook block verbatim (d77's keep-one list names them self-armed). The map is kind-keyed — d16's rows gate the spawn, d77's witness drives hand-built slots. |
| **d17** (proj-ai-161-170) | `f6e5d7d3a` | the 161-170 AI lane; d16's residual-5 census is its rowing input | Disjoint styles. d17 rows the 11 shortsword kinds off d16's `w4_d16_161_the_shortsword_helper_census`; no shared symbol. |
| **g65/g16** (proj-death-dust / gore-projectile) | — | the 728/729/761/762 Kill dust+gore bodies | d16 arms the CUES only; the dust/gore censuses are g65/g16's. Keep both, kind-disjoint layers. |
| **d53** (dust-sites-projectile) | — | the generic wet machine (`wet_block`) | The 159 wet kill + the kite wind hover read the visual_scene seam, not the wet block; no overlap. |

## 3. The keep-one rules (the hot symbols)

- `in_a_place_with_wind`: TWO copies exist by design — the kite pool's
  own (`projectile_ai_kites.rs`, reads `world.world_surface`/seeds) and
  the shared `tr_core::wind::Wind` method (reads the Wind struct). BOTH
  carry the out-of-world SKIP fix; they are NOT duplicates (different
  state sources), keep both. A future unification is its own task.
- `trail_cache_length`/`TRAIL_RING_CAP`: ONE definition (`projectile.rs`);
  the empress 872 (120) and the unported 41/51/71 kinds raise the cap
  when they row — never fork a second ring-depth table.
- The 728/729 death cue: ONE arm (d77's map). The 761/762 death cue: ONE
  arm (d16's death-hook block). Never both lanes' arms for one kind.

## 4. The standing deviations (the merger must NOT "fix" these)

The 640's 10-entry ring (C# 20), the once-per-update ring shift (C#
per-sub-step), the selectedKite PRE-override re-pin (one-tick-early,
observation-identical), the pool-derived holdStyle (no item-instance
field), the M7-TE-2 leashed-kite visual + the CheckAnchor reorient skips,
the single-player netMode folds. All recorded in ANALYSIS "W4-d16 kite
residuals" + `246` §4 — they are deliberate, not merge losses.

## 5. Verification

The SEAL re-probes (2026-11-21): the f20 retire doc + widened moonlord
(git object), the d77 cue map + witness (git object), the 150 registry +
the fresh 382-tip ls-tree scan (max taken 245 → 246/247 free), the C#
windows re-read direct (no junction recursion). 69 witnesses WRITTEN NOT
RUN (the ledger in `246` §2). No cargo/tests/rustfmt/graphify. No other
branch touched; publication on `task/W4-d16-proj-ai-151-160` only.