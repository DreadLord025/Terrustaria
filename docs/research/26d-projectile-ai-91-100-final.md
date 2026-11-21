# 26d — W4-d10 FINAL: the projectile aiStyles 91-100 band table

> Branch `task/W4-d10-proj-ai-91-100`, base `a725acb0` (wave-3 complete),
> 2026-11-21. CODE-ONLY round: every witness is WRITTEN NOT RUN. This is
> the band's final ledger — one row per style arm, the port state, the
> commit hash, the C# anchor, the witness count, and the cross-lane
> composition notes. The per-style census lives in the module doc
> (`projectile_ai_91_100.rs`) and the commit messages; the spawn-source
> verdicts in `26d-projectile-ai-91-100-docsweep.md`, the keep-one /
> composition verdicts in `26d-projectile-ai-91-100-crosschecks.md`.

## The band table (all ten arms LANDED on this branch)

| style | kind(s) | C# arm | port fn | commit | witnesses | notes |
|-------|---------|--------|---------|--------|-----------|-------|
| 91 | ShadowFlame 496 | inline `Projectile.cs:32515-32557` | `ai_shadowflame` | `bbbd5c43` | 4 | the `:18710-18735` soft-reverse/hard-kill blocked-move arm replaces kill_step; ZERO AI draws (the dust loop is the body-skip) |
| 92 | ToxicCloud 511/512/513, GasTrap 1007, spore 1128 | inline `:32558-32820` | `ai_clouds` | `b34ad73c` | 5 | the 1128 row is d10's KEEP-ONE (g2's 229 Kill burst resolves against it at merge, `81c4ab85`); the 1128 first-tick `Next(5)` is the only AI draw; the GasTrap particle broadcast is the particle-layer skip |
| 93 | NailFriendly 514 | inline `:32821-32882` | `ai_nail` | `07115c55` | 5 | the `:18819` wall re-arm + the `:76343` Kill blast (SIX `Next(2)` call-site draws kept) + the `:13334` stick ×0.15/×1.35 + the `:12452` state gate |
| 94 | CoinPortal 518 | inline `:32883-32963` | `ai_coin_portal` | `5c229707` | 3 | the source is the POT break (`WorldGen.SpawnThingsFromPot:57176-57182`, luck-0), NOT the Coin Gun 905 (docsweep §94); the coin drop rides `wants_coin_drop` → ItemDrop 73 stream-neutral |
| 95 | ToxicBubble 523 | inline `:32964-33003` | `ai_toxic_bubble` | `3296d8be` | 4 | ZERO AI draws; the `:73257` Kill burst's 75 explicit draws wired at the death hook in C# body order |
| 96 | IchorSplash 524 | inline `:33004-33028` | `ai_ichor_splash` | `bb1bd193` | 3 | the `:18807` blocked-move +100 age; the `:13227` hit +50 age; joins `uses_wind_physics` with 93 (`:22129-22130`) |
| 97 | FlyingPiggyBank 525 | inline `:33029-33133` | `ai_flying_piggy` | `14b14488` | 4 | the source is the Money Trough 3213 (`Item.cs:28981` shoot=525), NOT the Void Vault 4076 (docsweep §97); ZERO draws; the container UI is the no-consumer skip |
| 98 | CultistBossParticle 526 | inline `:33134-33152` | `ai_cultist_particle` | `8814cb4e` | 2 (+1 composition `649b18ae`) | the producer is the CultistTablet 437 aiStyle-83 — CLAIMED by d31 `a31ff86e` (unwired `npc_ai_style83.rs`, all three 526 bursts) + d40 `7fec14aa` (the rows); the ACTIVATION CONTRACT (the drain's owner-0 re-pin) is docsweep §98 (`649b18ae`) |
| 99 | yoyos 534/541-555/562-564/603/999 + counterweights 556-561/1079 | `AI_099_1_Counterweights` `:67380-67731` (gate `:33153`) + `AI_099_2_Yoyos` `:67733+` (gate `:33157`) | `ai_counterweight` + `counterweight_move` (the yoyo half pre-existed: `projectile.rs::ai_yoyo`) | `7ccca71c` | 6 base (+2 mouse-push `29a73a1d`, +2 grants-read `c6fd3bc0`, +3 spawn-lane `6e0f1381`) | the f19 F1 finding: the counterweight SUB-arm was missing; the kind gate keeps the yoyo half byte-identical; ZERO draws in the machine |
| 100 | MedusaHead 535 | `AI_100_Medusa` `:44476-44632` (gate `:33161`) | `ai_medusa_head` | `d448013c` | 5 | the 536 rays are d11's claim (`6aca9b7f`): the keep-one const drop-hunk EXECUTED (`7ca50dad` — the literal 536 spawns, the witness re-pinned to d11's row shape); the 12-float + 4-range scatter rolls stay in d10's arm pre-spawn |

## The satellite lanes (landed on this branch, feeding the band)

| lane | C# anchor | port | commit | notes |
|------|-----------|------|--------|-------|
| the counterweight mouse-delta seam | `Projectile.cs:67521` + `PlayerInput.cs:2528-2542`/`:911-917` | `App.mouse_tick_prev/delta` → `YoyoInputs.mouse_delta` → the aiStyle-99 dispatch | `29a73a1d` | the (0,0) park removed; every sub-step reads the same per-tick delta; 4 witnesses (1 frame-head in `app/frame.rs`, 2 push/clamps in the module, 1 pool drain in `projectile.rs`) |
| the yoyo-string grants | `Player.cs:882/:880`, `:14710-14754`, `:18435/:18439` | `Player.yoyo_string/magic_string` + `YoyoInputs` + the lifetime ×1.5 / range ×1.25+30 / radius +25%+10 / ghost-split reads | `c6fd3bc0` | M3-EQUIP-3 landed the framework only; 6 witnesses (1 grant/clear in `player.rs`, 2 counterweight reads in the module, 3 yoyo reads in `projectile.rs`) |
| the Counterweight SPAWN lane | `Player.cs:12356-12415` (the `:13272`/`:12826` call sites), the grants `:14714-14766` + the vanity loop `:13009-13015` | `counterweight_spawn` + `counterweight_on_hit` + `Player.counter_weight/vanity_counter_weight/yoyo_glove` + the bags' `Next(7)`/`Next(6)` roll on the UpdatePlayers stream | `6e0f1381` | PER-HIT cadence, ZERO lane draws; the roll re-draws per tick while worn (the C# ResetEffects behaviour); 6 witnesses |
| the CultistTablet activation contract | `NPC.cs:36757-36925` (d31's port) + the `:36863/:36874/:36885` owner argument | docsweep §98: the wiring wave's dispatch + the 526 drain owner-0 re-pin | `649b18ae` | claimed by d31/d40 — NO re-port; 1 composition witness |
| the MEDUSA_HEAD_RAY drop-hunk | the dc7acb5e keep-one verdict | the const removed, the literal 536 spawns, the s100 witness re-pinned | `7ca50dad` | d11's const is the one definition at merge |
| the cross-checks | g2's 1128 census + d11's 536 composition | `26d-…-crosschecks.md` | `dc7acb5e` | the 1128 keep-one (d10 owns) + the 536 composition (key link / strike walk / scatter rolls) |
| the DOC-SWEEP | the 94/97/98 producer censuses | `26d-…-docsweep.md` + the stale 518 comment fix (`editing.rs:5717`) | `732d3480` | the pot break / the Money Trough / the tablet — the three dispatch guesses corrected |
| the anchors canaries | one row per style arm | `anchors.txt` (10 rows + the grants/lane rows) | `1f0b60e9` | every marker verified present in the module |

## The witness ledger (WRITTEN NOT RUN — code-only mode)

Counted from the landed code (the `w4_d10_*` family):

- `projectile_ai_91_100.rs`: **49** — s91 ×4, s92 ×5, s93 ×5, s94 ×3,
  s95 ×4, s96 ×3, s97 ×4, s98 ×3 (incl. the tablet composition),
  s99 ×13 (6 base + 2 mouse-push + 2 grants-read + 3 spawn-lane:
  `rows`, `refreshes_and_sweeps`, `channel_push_inverts`,
  `flag3_clamps`, `string_caps`, `return_seeks`, `dropped_twin_falls`,
  `yoyo_string_grows`, `magic_string_splits`, `counterweight_bounces`,
  `lane_spawns_the_first_weight`, `scan_counts`, `glove_clones`),
  s100 ×5. (4+5+5+3+4+3+4+3+13+5 = 49.)
- `projectile.rs`: **5** — the pool mouse-drain, the yoyo lifetime,
  the yoyo range, the yoyo ghost twin, the wired counterweight hit
  spawn.
- `player.rs`: **3** — the grant/clear cycle, the bag roll order, the
  pins + vanity.
- `app/frame.rs`: **1** — the frame-head mouse-delta shift.

**Total: 58 witnesses** (49 + 5 + 3 + 1), every one WRITTEN NOT RUN.
The s100 row witness's 536 half is a POST-MERGE pin (it asserts d11's
landed row shape; on this branch the row is absent — the documented
no-row decline).

## Merge notes (the sibling lanes)

1. **g2 ⨉ d10 (1128):** no conflict — d10's `SPORE_CLOUD=1128` row is
   the one row; g2's 229 Kill-burst spawn resolves against it, the 64
   kept draws stay aligned (crosschecks §1).
2. **d11 ⨉ d10 (536):** the const collision is PRE-RESOLVED — d10's
   duplicate is dropped (`7ca50dad`), the dispatch reads the literal;
   at merge d11's row + arm land and the spawns resolve (the s100 pin
   flips green). The `ai0=this.key` bit-pattern link + the strike walk
   compose (crosschecks §2).
3. **d31/d40 ⨉ d10 (526):** no code overlap — d31's `npc_ai_style83.rs`
   is unwired on its branch; the wiring wave owes the dispatch + the
   owner-0 drain re-pin (docsweep §98). d10's row + seek are live.
4. **c1 ⨉ d10:** none — c1 wires the boss 439/aiStyle-84; the 526
   producer is the tablet 437/aiStyle-83 (docsweep §98).
5. **M3-EQUIP-3 ⨉ d10:** the framework landed (`5ec90b7c`); d10's
   grants + spawn lane extend it — the `counterWeight`/`yoyoGlove`
   fields are d10's alone, no keep-one.

## Verification

CODE-ONLY mode: no cargo/tests/rustfmt run (the task's constraint). The
witnesses are written to the house pattern (the mirrored-stream /
literal-pin conventions) and marked WRITTEN NOT RUN in every commit.
The merge wave runs `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` per AGENTS.md; the s100 536 pin is
the one assertion that REQUIRES the d11 merge to pass.