# 131 — DUST CROSS-LANE MASTER STATUS (W4-d54 crosslane phase)

Code-only crosslane round, 2026-11-21, on `task/W4-d54-dust-sites-world`
(union EXECUTED `c792cf37`, research/66 §6). This doc is the **per-file
keep-one FINAL table** for the whole dust family, the merge-order
recommendation, the witness-union census and the outstanding-fold
ledger. Every keep-one decision below was re-confirmed against the
FETCHED lane tips (not the earlier probes); the decisions themselves
are owned by research/66 (d54/g14 seams), research/83 + its g11
addendum (snowing), research/116 (teleport), and the g73 waterfall
union contract — this doc records their FINAL state and the residual
work each lane still owes.

## 1. Tips read (git fetch, this round)

| Lane | Branch | Tip read | State |
| ---- | ------ | -------- | ----- |
| d54 (this) | task/W4-d54-dust-sites-world | `c792cf37` | union EXECUTED: g14 tr-core canonical + LIVE seams re-pointed (research/66 §6 ledger ce1276ba/8f8471cb/74ba4a2e/462a21ff/e41a90f0) |
| g14 | (folded into d54) | `a35efec2` via cherry-picks | tr-core worldgen_dust{,_wall,_tile}.rs, all 29 WorldGen sites, arm-11 torch round-trip + paint alpha-150 fixes applied on d54 |
| g11 | task/W4-g11-dust-main-sites-1 | `d892d8d4` (wiring tip `13a1c48e`) | snowing WIRED at the :17238 scene_scan slot + TeleportEffect 0-7 WIRED (player tick + wiring pads); tip adds research/83 addendum + research/116 union plan (docs-only) |
| g12 | task/W4-g12-dust-main-sites-2 | `665cd1e8` | main_dust_b.rs sites 9-16 (teleport cases 10/11/13 STAGED + bone-arm puffs 22744/22820 + Moon Lord flag8 trio 24736/24748/24752), 7 witnesses, UNWIRED |
| g13 | task/W4-g13-dust-main-sites-3 | `15fd99c9` | main_dust.rs sites 17-23 WIRED (draw-call feeds + DrawnThisUpdate relocation + item-sparkle 16px light-box fix + retro/Moon-Lord witness fixes), 18 witnesses |
| g61 | task/W4-g61-npc-spawn-dust | `a9628928` | npc_spawn_visuals.rs — widest teleport (11 styles + pylon 9 + NPC driver + per-tick visuals + alt-texture) + spawn family, UNWIRED, 18 witnesses; particles.rs DustRng generalisation |
| e4 | task/W4-e4-dust-sites-world-2 | `2a79551a` | orchestrator_dust.rs (orch 41-61, 9 witnesses) + mount_dust.rs (Mount 1-19, 10 witnesses), UNWIRED; lane complete 40/40 |
| f6 | task/W4-f6-dust-world-3 | `5daa1d70` | mount_dust.rs ALL 27 Mount sites (24 witnesses) + **batch 4 LANDED**: world_dust.rs 13 small-file sites (8 witnesses) + 40-site delivery docs + fairy-color witness hardening |
| g63 | task/W4-g63-proj-trail-dust-1 | `b1e78b19` | projectile_trail_dust.rs — rowed inline-machine kinds ≤400 re-armed end-to-end, WIRED via update_dust from world_sim/projectiles.rs, 19 witnesses |
| g64 | task/W4-g64-proj-trail-dust-2 | `77616a7d` | proj_trail_dust.rs — kinds >400 superset, 42 emitters + 16 re-armed call sites on the UpdateProjectiles stream (WIRED), 44 witnesses; particles.rs DustRng generalisation (second copy) |
| e3 | task/W4-e3-dust-sites-proj-2 | `8023aa4d` | projectile_dust.rs — aiStyle-generic window :22297-24318 (41 sites + the 24328 edge pair, all 42 ported), 21 witnesses + 3 keep-one contracts vs d16/d1/f48 (doc sweep tip) |
| d130 | task/W4-d130-weather-ambience | `ec1d73c7` | research/83 EXECUTED (`88cfbf38`): rain+splash union live on d130; snow.rs base kept — **the g11 addendum folds F1-F7 NOT yet applied** (still splitmix/1920/no pause gate at tip) |
| g73 | task/W4-g73-waterfall-draw | `82ae8b96` | waterfall family FINAL UNION (`eee87f98` folds g75 `e30ad7ba`); contract rewritten: canonical = g73 tip, 43-witness inventory — **but g75 pushed 3 commits AFTER the fold base** (§5) |
| g75 | task/W4-g75-honeyfall-draw | `2b9ed81f` | post-`e30ad7ba` final sweep: `d118f7ba` disco colour-source fix + `f4e3071f` 717 lava-fountain witnesses + `2b9ed81f` sweep docs — OWED a fold into g73 |

## 2. Per-file keep-one FINAL table

### WorldGen lane (RESOLVED on d54 — research/66 §6 EXECUTED)

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-core/worldgen_dust.rs` | **g14** (ash-shake consumes the gore argument draws :29669) | LANDED on d54 (`8f8471cb`), paint color-30 alpha-150 fix `74ba4a2e` |
| `tr-core/worldgen_dust_wall.rs` | **g14** (ladder multiset identical; slot return) | LANDED on d54 |
| `tr-core/worldgen_dust_tile.rs` | **g14** 8-chunk ladder + **d54 arm-11 torch `*255` round-trip** + `edit::solid_tile` OOB-false | LANDED on d54, fixes `74ba4a2e`, wrong-semantics witness rewritten |
| `tr-app/world_dust.rs` (d54) | bodies DROPPED; **7 supplement witnesses** re-pointed (ash oracle adapted to the gore draws) | LANDED `e41a90f0`; LIVE cutter seams call tr-core directly |
| `tr-app/world_dust_tile.rs` (d54) | **`tile_break_burst` wrapper only** (amount = `collision::tile_dust_amount`, kind-first) | LANDED `e41a90f0`; duplicate amount/solid_tile copies dropped |
| `tr-app/world_dust_wall.rs` (d54) | **`wall_break_burst` + `place_tile_torch_dust` wrappers** (caller-side 22/140 gate) | LANDED `e41a90f0` |
| `tr-app/world_dust.rs` (**f6**, batch 4) | **KEEP f6's file** — DISJOINT from d54's supplement-only file in content but SAME filename: chest unlock / Utils.PoofOfSmoke / pylon per-style / void-lens / popup flash / lava bubbles / rain splash / nebula level-up (13 sites) | NOT on d54; merge collision resolve below |
| `collision.rs tile_dust_amount` | main's copy (kind-first) — the ONE table | LANDED; d54's third copy dropped |

**f6-vs-g61-vs-d54 small-file overlap (NEW at f6 `680ac9d8`):** f6's
batch-4 `poof_of_smoke` (Utils.cs:1089) and `pylon_spawn_dust`
(TeleportPylonsSystem.cs:390) DUPLICATE g61's landed `poof_of_smoke` /
`pylon_spawn_in_world_dust` (npc_spawn_visuals.rs). Research/66 §2
already ruled: **g61's bodies win** (its pylon port is the style-9
union body per research/116 §1; its poof carries the gore-arg draws).
f6's `rain_splash` (Rain.cs:122) duplicates d130's EXECUTED splash
union (`spawn_splash`/`water_splash_arm`, research/83 §3) — **d130's
wins** (wired + the impact-gate fix). f6's UNIQUE six (chest unlock,
void-lens, popup flash, lava bubbles' private `_random` stream, nebula
level-up, and its `dust_water` helper if not already on main) keep.
Merge shape: f6's world_dust.rs unique emitters fold into d54's
supplement-witness module (or a sibling `world_dust_small.rs` — the
merger's naming call), the three duplicate bodies drop, f6's 8
witnesses supplement minus the poof/pylon/splash duplicates.

### Main lane

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-app/main_dust.rs` | **ONE file = g13's six bodies + feeds (sites 17-23, WIRED) + g11's teleport SEAMS dispatcher** (research/116 §3: `player_teleport`/`npc_teleport` replays, the Entry-OLD→Exit-NEW queue, the `Npc` teleport fields). g11's snowing body RETIRES (research/83: d130 canonical); g11's seven teleport arm bodies RETIRE (g61's body wins) | filename collision g11-vs-g13 unresolved on the branches — merger-wave resolve |
| `tr-app/main_dust_b.rs` (g12) | **KEEP the bone-arm tip puffs + Moon Lord flag8 trio halves**; the three `teleport_effect_case10/11/13` fns RETIRE into g61's body (research/116 §3 — draw-equivalent, g61 canonical hsl + slot-read clone guard) | g12 `665cd1e8` stands; case witnesses supplement |
| `tr-app/npc_spawn_visuals.rs` (g61) | **KEEP whole** — the widest teleport_effect (11 styles incl. pylon 9), poof/mimic/transform, alt-texture transitions; its env-level `npc_teleport` driver RETIRES (g11's App-level replay supersedes, + `other_position` per 116 gap 2) | UNWIRED; wiring per research/116 §3 |
| `tr-app/world_dust_main.rs` (d54) | **DROP both bodies at the main-lane resolve** (snowing → d130 per 83; teleport → g61 per 116); the 9 witnesses supplement (2 snowing → d130 re-pin, 7 teleport + colour oracle → g61 re-pin) | LATENT on d54 today — the DROP is merger-wave (this branch does not carry the surviving files) |
| `tr-app/snow.rs` (d130) | **KEEP as the snowing canonical** (research/83 EXECUTED on d130: wired + real `DustPool.snow_dust` counter + type-76 AI + draw) **+ the g11 addendum folds F1-F7** | F1-F7 OUTSTANDING (§5) |
| `tr-core/particles.rs` DustRng delta | **KEEP one generalisation** — g61 `a9628928` vs g64 `853039bf` diffs are DOC-COMMENT-ONLY (8 lines, verified this round: same trait, same generic signatures, same bodies); either lands, take g61's (first) and g64's comment wording is cosmetic | merge conflict is trivial-comment-level |

### Mount lane

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-app/mount_dust.rs` | **f6's superset (ALL 27 sites)** — CONFIRMED from tips: f6 `5daa1d70` carries batches 1-3 (10+10+7) incl. the lava-cart 5099 / pixie 5149 / failed-dismount 6339 / DoSpawnDust wolf-chillet-generic 6382-6493 + FairyQueen colour helper; e4's 19 are a subset (bodies spot-equivalent, research/66 §2). e4's mount-name correction stands (44=PirateShip, 45=SpookyWood — f6's names right) | e4's file DROPS at merge; its 10 witnesses supplement where f6's 24 don't pin the draws |
| f6 `drill_beam_dust` atan2 | **STILL F32 at f6 tip** (`d.y.atan2(d.x)`, verified `5daa1d70`) — the double-atan2 fix (e4's caller-passes-aim shape, `Utils.ToRotation` = `(float)Math.Atan2` DOUBLE) is OUTSTANDING, owed at the mount resolve | one-edit fix + witness re-pin |
| `tr-app/orchestrator_dust.rs` (e4) | **KEEP whole** — orch 41-61 DISJOINT from d54's 1-40 (boundary 2582/2653 verified, research/66 §2); the 272-draw RainbowRodHit pin is the prefix.rs count-only fold authority | UNWIRED; the prefix.rs fold is wiring-wave (never both — double-draw risk) |

### Projectile-trail lane (g63 vs g64 vs e3 — three DISJOINT scopes, one seam family)

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-app/projectile_trail_dust.rs` (g63) | **KEEP whole** — rowed inline-machine kinds ≤400 (35 emitters incl. the molotov 399 + the no-dust rowed census closure), WIRED via `ProjectilePool::update_dust` from world_sim/projectiles.rs | 19 witnesses |
| `tr-app/proj_trail_dust.rs` (g64) | **KEEP whole** — kinds >400 superset (42 emitters, 16 call sites re-armed on the UpdateProjectiles stream, the 533 frame-table fix, the 509 behaviour-feeding jitter, the 596 bool seam, the 644 prism-hue orbit), WIRED via the same update_dust seam shape | 44 witnesses; the 26 unrowed kinds stay cited (S1-F-9b) |
| `tr-app/projectile_dust.rs` (e3) | **KEEP whole** — the aiStyle-GENERIC window :22297-24318 (styles 2/4/5/6/8/152 arms; 42 sites, zero missed per `8023aa4d` census) — a DIFFERENT axis from g63/g64's per-kind arms; no body overlap | 21 witnesses + the 3 keep-one contracts below |
| g63-vs-g64 kind boundary | g63 ≤400 rowed / g64 >400 — the 387/388/533 twins + 669 flag35 block are g64's (the type-533 gate); g63's 399 arm and g64's 669 Betsy pair are complementary, both keep | verified from the batch docs |
| e3's STAGED 152 emitters | `star_slash_dust`/`star_slash_976` drop-in over **d16 `e122a6b2`**'s count-only skips (3-edit contract, e3 `ab75231f`: thread DustPool+SpawnCtx, REPLACE the Next(8) block — keeping d16's circular pair DOUBLE-DRAWS — REPLACE the Next(3)+localAI[1] blocks, latch AFTER the call) | merger-wave, d16 still count-only at its tip |
| e3's 1051 style6_burst | feed through **d1 `e271b5bc`**'s machine+rows (count-only powder twin), retire e3's ai_style6 duplicate; pick-first order pinned by e3's keep-one witness | merger-wave |
| e3's cloak-five rows (723-726/1133) | ROWED on **f48 `bba610bb`** (`ai_cloak_star`) — f48's rowed arm survives + takes the cloak dust half; **g64's trail_503/723/1133 twins** (proj_trail_dust.rs:765/:1485/:1859, verified at tip) are the unwired counterparts — keep-one at merge per e3's contract; 503/1037/1049 unrowed everywhere, gates stay no-op | merger-wave |

### Waterfall family (falls-adjacent — the dust-feeding liquid loops)

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| waterfall family (draw.rs/mod.rs/ambient_sound.rs/sound.rs/audio.rs + consumers) | **ONE branch: g73 tip** (`82ae8b96` contract FINAL; `eee87f98` already folds g75 `e30ad7ba` + g74 `cd462c1a`) — g73's `liquid_loop_update` superset (ambientLava Manhattan pick+sum, create-arm-no-pitch, live re-pan) delegates to g75's pure `sound::liquid_ambience`; g74/g75 branches DROP wholesale | **g75's 3 post-fold commits OWED a re-fold** (§5): `d118f7ba` removes the manager-local disco cycle g73 still carries (`update_disco_rgb`, verified at g73 tip) for the shared-statics `set_disco_rgb` latch; `f4e3071f`'s 2 lava-fountain witnesses (the 717 SPAWN arm itself is already in g73's mod.rs, verified) |

## 3. Merge-order recommendation

Ordered so each step's inputs are already canonical when it runs; no
step re-litigates a resolved decision.

1. **d54 (this branch)** — already carries the WorldGen resolve
   (research/66 §6). Lands first: the tr-core worldgen_dust trio +
   the LIVE editing/wiring seams + the orch 1-40 lane. Everything
   else re-points at these markers.
2. **g61** — npc_spawn_visuals.rs + the particles.rs DustRng
   generalisation (the comment-only g64 twin resolves trivially).
   The teleport body every other Main-lane step needs.
3. **g12** — main_dust_b.rs (drop the 3 staged teleport cases into
   g61's body per 116 §3; keep the bone-arm/Moon-Lord halves + all 7
   witnesses as supplements).
4. **g13 + g11 main_dust.rs resolve** — ONE file: g13's bodies/feeds +
   g11's seams dispatcher calling g61's body (116 §3), the 6 wiring
   gaps fixed (10/11/13 fall-through, other_position, style-9
   extraInfo2, water_style param, Player.Teleport head/tail arms,
   double grants). g11's arm bodies + snowing retire here.
5. **d130** — rain+splash+snow canonical (83 EXECUTED); apply the g11
   addendum folds F1-F7 (§5) in the same step; d54's
   world_dust_main.rs snowing half + g11's Snowflake-retirement notes
   re-point. The `sand.rs`/`rain.rs` 1920-pin deviation family stays
   flagged for their own lanes.
6. **d54 world_dust_main.rs DROP** — the teleport half retires once
   step 4's dispatcher is live; the 9 witnesses re-point (2 → d130,
   7 → g61). Anchors row re-points at the surviving markers (never
   delete without the replacement landing).
7. **e4 + f6 mount resolve** — f6's superset file + the double-atan2
   fix; e4's orchestrator_dust.rs lands alongside (disjoint); the
   prefix.rs RainbowRodHit count-only→emitter fold records here.
8. **f6 batch-4 small files** — the unique six fold per §2 (g61's
   poof/pylon + d130's splash win the three duplicates).
9. **g63 → g64 → e3** — the trail trio in scope order (≤400 rowed,
   >400 rowed, aiStyle-generic); then e3's three keep-one contracts
   execute against d16/d1/f48 (those branches merge AFTER their
   count-only skips are replaced, or the contracts ride their merge
   commits). The g64 unwired cloak twins resolve with f48.
10. **g73 waterfall family** — after re-folding g75's 3 post-fold
    commits (§5); g74/g75 drop wholesale.
11. **Merge-verify wave** — compile + run the witness UNION (§4);
    the anchors canary + `main_anchors` test guard every step.

## 4. Witness-union census (all WRITTEN NOT RUN — code-only lanes)

Counts re-tallied from the fetched tips (`#[test]` grep per file):

| Lane / file | Tip count | Union disposition |
| ----------- | --------- | ----------------- |
| d54 batch-1/2/3 supplements (world_dust{,_wall,_tile}.rs) | 18 (7+5+6) | KEEP, re-pointed at tr-core (landed `e41a90f0`); ash oracle adapted to the gore draws |
| d54 batch-4 (world_dust_main.rs) | 9 | 2 snowing → d130 re-pin (F7), 7 teleport + colour oracle → g61 re-pin (116 §3) |
| d54 orch batches 5-8 | 21 (7+3+5+6) | KEEP whole (sites 1-40) |
| g14 tr-core trio | 25 (24 + the rewritten torch witness counted once + 1 new paint alpha-150) | KEEP (landed on d54 `74ba4a2e`) |
| g11 main_dust.rs | 15 (7 style + 4 snowing + 4 wiring) | 4 snowing → d130 F7 re-pins; 7 style + 4 wiring keep pinning the dispatcher; `the_g12_styles_stay_untouched` INVERTS at the union (116 §3); the pad integration witness (wiring.rs) keeps |
| g12 main_dust_b.rs | 7 | 4 teleport-case witnesses supplement g61 (same oracles); 3 bone-arm/Moon-Lord keep |
| g13 main_dust.rs | 18 (14 body + 4 feed/wiring) | KEEP whole (no overlap) |
| g61 npc_spawn_visuals.rs | 18 | 9 teleport witnesses transfer with rig adaptation (draw-equivalent per 116 §1); 8 spawn-family + alt-texture keep; style-12 gate unconditional (`a9628928`) |
| e4 orchestrator_dust.rs | 9 | KEEP (the 272-draw RainbowRodHit pin is the prefix fold authority) |
| e4 mount_dust.rs | 10 | supplement f6 where unique (basilisk-charge-fold, witch-puff-type31 pins); exact-shape duplicates drop |
| f6 mount_dust.rs | 24 | KEEP base (fairy-color generic-path expectation derived through hsl_to_rgb, `5daa1d70`) |
| f6 world_dust.rs (batch 4) | 8 | KEEP minus the poof/pylon/splash duplicate oracles (g61/d130 bodies win) |
| g63 projectile_trail_dust.rs | 19 | KEEP whole |
| g64 proj_trail_dust.rs | 44 (40 batch + 4 wiring-wave) | KEEP whole (incl. the DropRng/concrete seam parity + the 642/509 stream-parity pair) |
| e3 projectile_dust.rs | 21 (incl. the 1051 pick-first keep-one witness, `ab75231f`) | KEEP whole |
| g73 waterfall family | 43 (contract inventory) | + g75's post-fold 5 (3 disco + 2 lava-fountain) = 48 after the re-fold |

Union total (dust lanes, pre-dedup): **309 witnesses** (the table
sums to 309, + g11's pad integration witness in wiring.rs = 310;
315 after the g75 post-fold re-fold); the exact
post-dedup tally is the merge-verify wave's census (checklist 6 of
research/66). No cargo/tests invoked this round.

## 5. Outstanding folds (the crosslane ledger)

| # | Fold | Owed by | Detail |
| - | ---- | ------- | ------ |
| 1 | **g73e — the g75 post-fold falls sweep** | g73 (or the family merger) | `git log e30ad7ba..origin/task/W4-g75-honeyfall-draw` = `d118f7ba` + `f4e3071f` + `2b9ed81f`. The disco colour-source fix is SEMANTIC: g73's mod.rs still carries the manager-local `update_disco_rgb` cycle (out of phase with the app's `advance_disco`, init 0 vs Main.Initialize's DiscoR=255 :66780) — replace with g75's `set_disco_rgb` latch fed from params.disco beside `set_show_invisible` (frame/draw.rs). The 717 spawn arm is already in g73; only the 2 witnesses + the 3 disco witnesses + the sweep docs fold. The g73 contract's own cross-note demands this check at merge time. |
| 2 | **g11c — the snowing details F1-F7** | d130 (the canonical branch) or the weather merger | research/83's g11 addendum (`d892d8d4`) lists 7 exact folds onto d130's snow.rs: F1 g11's body + the `global` stream (retire the Snow splitmix + App.snow field), F2 MAX_SCREEN_W 4096 HiDef (re-derive every budget expectation), F3 restore the in-body gamePaused gate, F4 counter budget once-per-call (d130's shape — g11's per-attempt live_type census retires; this SUPERSEDES research/66 §6's "g11b hoist" note: the hoist target is d130's counter param, not g11's loop), F5 int attempt-loop shape, F6 env re-point (g11's scene_scan SnowingEnv block → SnowInputs + game_paused), F7 the 4 g11 snow witnesses re-pinned. VERIFIED at d130 tip `ec1d73c7`: snow.rs still pins 1920/splitmix/no pause gate — NONE of F1-F7 applied yet. |
| 3 | **f6 double-atan2 drill fix** | the mount-resolve merger (step 7) | f6 tip still computes `d.y.atan2(d.x)` in F32; take e4's caller-passes-aim shape (double atan2, orchestrator_dust.rs:76-78 precedent). **EXACT 2-hunk patch published §7.1 (final-audit round)** — minimal in-place `(d.y as f64).atan2(d.x as f64) as f32` at mount_dust.rs:85 + the :1598 witness mirror; signature keeps (target, player_center), no RNG-order change. |
| 4 | **research/116's 6 wiring gaps** | the main-lane merger (step 4) | 10/11/13 fall-through on reachable callers (Queen-Slime hook, Shellphone), other_position on the reqs/queue, style-9 pylon extraInfo2, style-5 waterStyle param, Player.Teleport head/tail arms, the double-grant cleanup. |
| 5 | **e3's 3 keep-one contracts** | the proj-lane mergers (step 9) | 152 drop-in vs d16 `e122a6b2`, 1051 burst vs d1 `e271b5bc`, cloak-five vs f48 `bba610bb` + the g64 unwired twins. |
| 6 | **prefix.rs RainbowRodHit fold** | the wiring wave (step 7) | count-only drains → e4's full emitter, never both. |
| 7 | **The witness UNION compile+run** | the merge-verify wave (step 11) | every lane was code-only; ~330 witnesses WRITTEN NOT RUN. |

## 6. This branch's residuals (research/66 checklist-7 audit)

Re-audited this round on `c792cf37`: all 11 d54/g14 anchors rows
(899-909) carry markers that EXIST in the surviving files (verified
by grep: world_dust.rs:256, world_dust_wall.rs:27,
world_dust_tile.rs:32, world_dust_main.rs:238, orch_dust.rs:99,
orch_dust_lightning.rs:198, orch_dust_boulder.rs:163,
orch_dust_town.rs:419, worldgen_dust.rs:98, worldgen_dust_wall.rs:466,
worldgen_dust_tile.rs:1511). No dropped-body row lacks its
replacement marker. Two stale prose references found and fixed in
this round's residual commit: the anchors orch_dust row's
"delegating make_tile_dust" tail and ANALYSIS.md's tile-sweep
paragraph (:39276) still named the pre-union delegation — both now
name the landed `collision::tile_dust_amount` +
`kill_tile_make_tile_dust` re-point (`e41a90f0`). Checklist items
1-3/7 are EXECUTED on this branch; 4-5 are cross-doc (the decisions
stand, the code lives on the other lanes); 6 is the merge-verify
wave's. No unclaimed checklist item remains on d54-owned files.

## 7. FINAL AUDIT ADDENDUM (final-audit round, 2026-11-21)

### 7.1 f6 drill-atan2 — the EXACT patch hunk (owed fold §5 row 3)

Re-verified at the fetched f6 tip `5daa1d70`:
`crates/tr-app/src/mount_dust.rs::drill_beam_dust` (fn at :75) still
computes the aim in F32 at :85. The C# is
`Mount.cs:3107` `float num = (vector - mountedPlayer.Center).ToRotation();`
and `Utils.ToRotation` is **`Utils.cs:1655-1658`**:

```csharp
public static float ToRotation(this Vector2 v)
{
    return (float)Math.Atan2(v.Y, v.X);
}
```

`Math.Atan2(double, double)` — the floats are WIDENED, atan2 runs in
DOUBLE, the result is cast back to f32. `f32::atan2` is a different
libm path and can differ in the last ulp; the cos/sin at :93-95 then
amplify the angle delta into the velocity bytes. e4's port already
has the correct shape — `orchestrator_dust.rs:74-78` at e4 tip
`2a79551a`:

```rust
/// `Utils.ToRotation` (`Utils.cs:1655-1658`): `Math.Atan2(Y, X)` in
/// double of the widened floats, cast back.
pub(crate) fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}
```

**Hunk 1 — the body fix** (`mount_dust.rs:83-85` at f6 `5daa1d70`),
old:

```rust
    // `:3107` — `(vector - mountedPlayer.Center).ToRotation()` (atan2).
    let d = target - player_center;
    let num = d.y.atan2(d.x);
```

new:

```rust
    // `:3107` — `(vector - mountedPlayer.Center).ToRotation()`:
    // Utils.cs:1655-1658 `(float)Math.Atan2(v.Y, v.X)` — Math.Atan2 is
    // DOUBLE of the widened floats, cast back (e4 to_rotation precedent,
    // orchestrator_dust.rs:76-78).
    let d = target - player_center;
    let num = (d.y as f64).atan2(d.x as f64) as f32;
```

**Hunk 2 — the witness re-pin** (the mirror inside
`w4_f6_drill_beam_spawns_two_and_mirrors_the_draw_order`,
`mount_dust.rs:1598`), old:

```rust
        let num = d.y.atan2(d.x);
```

new:

```rust
        let num = (d.y as f64).atan2(d.x as f64) as f32;
```

Two one-line edits (+ the citation comment). The signature KEEPS
`(target, player_center)` — f6's file is the keep-one survivor (§2
Mount lane), so the minimal in-place fix beats reshaping to e4's
caller-passes-aim `drill_sparks(tile_centre, aim, …)`; the second
call-shape witness `w4_f6_culled_spawn_writes_the_sentinel` (:1965)
passes positions only and needs NO edit. No RNG draw order changes
(the atan2 is draw-free); the streams stay aligned. Owner: f6-owner
or the mount-resolve merger (merge-order step 7).

## Verification

Code-only per the task brief: no cargo/tests/rustfmt/graphify. Every
tip row was read via `git fetch` + `git log`/`git show` this round;
the witness counts are `#[test]` greps of the tip blobs; the
particles.rs g61-vs-g64 delta was diffed in full (8 comment lines);
the f6 atan2, d130 snow.rs pin state, g73 disco-cycle state and the
g75 post-fold commit list were verified against the fetched blobs,
not commit messages alone.