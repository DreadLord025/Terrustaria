# 340 - W4-d21 FINAL: the projectile aiStyles 201-210 band table + the proj-AI-1-210-all-decades-CLOSED ledger

> **Renumber chain: 308 → 340.** registry-150 §9 UNIFIED FINAL (W4-d54
> tip `b243c7086`, §9.2 row 340): **g13 KEEPS 308** (§8.5 — g13's executed
> 16:31 pre-§8 push wins); d21's `308-projectile-ai-201-210-final.md`
> (`5a6ed468e`) contests that push → **d21 → 340**. This file is the
> renamed `308-projectile-ai-201-210-final.md`; all clean refs repointed
> (INDEX / ANALYSIS / PORT_MAP / anchors / the 162 census cross-ref).
> The historical §7.4 pick narrative below is kept verbatim as evidence.
>
> Numbered **308** (at seal time) per the research-number collision registry
> (`150-research-number-registry.md` §7.4, W4-d54 `0426245f2`: "Numbers
> 308+ are FREE", live total 148 = 151–236 + 246–280 + 281–303 + 304–307).
> Fresh all-tip sweep this round (2026-11-21 ~16:00, every
> `refs/remotes/origin/*` tip, `git ls-tree docs/research/`): max on-origin
> **298**; 308 verified free by `git log --all -- docs/research/308-*`
> (zero history commits). NOT a max+1 pick — the registry block + the
> sweep agree. Note: d9's post-extension push `88677f9de` took **281**
> (its own sweep said max 280) — that number is registry-assigned to f26
> (`68-findframe-census-addendum-7h.md` → 281); the collision is d9/f26's
> to adjudicate at the registry's next round, NOT d21's. d21 avoids the
> contested band entirely by taking 308.
>
> Branch `task/W4-d21-proj-ai-201-210`, base `a725acb0` (wave-3 complete),
> SEAL head `70fe977c3` (2 ahead of origin `9fa9e9ba0` at seal time:
> `b8a4d1288` folds 3+4 port + `70fe977c3` folds 3+4 wire). CODE-ONLY
> round: every witness WRITTEN NOT RUN — no cargo/rustc/rustfmt/tests/
> dotnet/graphify invoked. Verification = direct-file C# re-reads (≤100-line
> windows, direct paths, no junction recursion), git-log/ls-tree censuses,
> and hand arithmetic.
>
> The per-style census lives in `162-projectile-ai-201-210-census.md`
> (renamed from `26d-…` this round per registry §6 row
> `d21-proj-ai-201-210`: 26d → **162**, uncontested — d12 owns the 26d
> key at 00:06:06, d21's add 02:40:39 is a loser). The folds record is
> ANALYSIS "The projectile aiStyles 201-210 (W4-d21)".

## S1 — git-log census: what landed on this branch (band 201-210)

| commit | what landed | witnesses |
| --- | --- | --- |
| `aeea0ab89` | census doc (26d, now 162): 7 MISSING (201/202/203/204/205/208/209), 206 verify-only a9 `8c4fded8`, 207 empty/no kinds, 210 already `ai_yoyo_shot` | — |
| `20fe4b3ed` | the seven machines verbatim (`projectile_ai_201_210.rs`, 1998 lines) + 11 rows (`projectile/data.rs`) + dispatch arms + movement/death seams (`projectile.rs`) + digtoise `PickTile` drain (`world_sim/editing.rs` + `world_sim/projectiles.rs`) + `main.rs` mod | 11 written not run |
| `9fa9e9ba0` | ANALYSIS folds (gravDir/petting/Legendary/LightningGenerator/tracked-sound/PickTile-drain) + PORT_MAP row | — |
| `b8a4d1288` | folds 3+4 port: `lightning_generator.rs` (738 lines — `MultiPointHitbox` 42-line pure geometry + `LightningGenerator` verbatim; census proved NOT render-only: `SolidTileCollision` truncates bolts, re-centers the 203 kill, positions ARE the damage hitbox); own `LCG32Random(ai[2])` stream, zero `Main.rand`; `CalcRotations` stays the render-only skip | 8 written not run (9 fns incl. the split) |
| `70fe977c3` | folds 3+4 wire: `storm_lightning_kill` generates the real bolt (LCG32 `ai[2]` seed), tile truncation re-centers the 1091/1117 + picks `localAI[0]` 1/2/3, `MultiPointHitbox` rides pool `custom_hitbox` through `damage_pve_boxed`'s 203 arm; `StormLightningCollisionCheck` full chain (explosion rect + liquid `PlotLine` + multi-point) replaces the 64px fold; electrification `Next(4)` draws stay; world threaded through the boxed callers | 4 written not run |

Committed witness ledger at `70fe977c3`: **24** `w4_d21_*` fns —
15 in `projectile_ai_201_210.rs` (11 base + 4 kill-chain) + 9 in
`lightning_generator.rs`. The 11-base + 8 + 4 commit-message counts
(23) differ by one because the folds-3+4 port split one witness into
two fns (the LCG32 advance + the float-range/lerp pair); the fn census
is the authority.

**The aiStyle-space-ends-210 C# fact (re-verified this round, direct
reads):** `Projectile.cs::AI()` opens `:23199` with `if (this.aiStyle ==
1)` and its if/else chain CLOSES at `:36803-36807` — `else if (this.aiStyle
== 210) { this.AI_210_YoyoShots(); }` then `}`; `AI_210_YoyoShots` is the
last AI method (`:36809`). The whole-file `this.aiStyle = N` write census
(`SetDefaults` + the runtime flips): **809 writes, 210 distinct values,
MAX 210, ZERO writes > 210, ZERO writes of 207** (the 207 empty-AI fact,
F1 below). The g2 lane's independent scan (`d0708e3b3`, 1546+168 files,
max literal 210) agrees. **The C# aiStyle space is exactly 1..210.**

## S2 — d77c cue-1115 armed-on-d21: the compose receipt

The d77 Kill-host wave (`0573b70f2`, tip `5f10560b2` — re-probed this
round, UNCHANGED since the f1-RETIRE re-pin `3065492b0`) carries the
re-probe cue arm:

```
origin/task/W4-d77-proj-sounds-3:crates/tr-app/src/projectile.rs:20392
    1115 => Some((2, 27, pos.x, pos.y, 1.0)),
```

with the comment "`:75502` — the Glacier Fang shot's Item27 (the
Remap-scaled dust census is the g65 lane's; **the row + aiStyle-208 arm
live on d21 `9fa9e9ba`**)". C# anchor re-read: `Projectile.cs:75500-75512`
— `else if (this.type == 1115) { SoundEngine.PlaySound(SoundID.Item27,
base.position); … }` (the cue at `:75502`, position NOT centre; the
10+20·Remap dust loop is g65's). `SoundID.Item27` = id 2 style 27 — the
legacy id-2 family `play_proj_sound` already serves (`audio.rs:2014-2020`
on this branch).

**Compose verdict — ARMED-ON-D21, hunk-disjoint, no keep-one needed:**

- d21 owns the 1115 **row** (`projectile/data.rs:2957`, `GLACIER_FANG_SHOT
  => (10.0, 10.0, 208, …)`, C# `:10401-10412`) + the **aiStyle-208 arm**
  (`projectile.rs:19923`, `ai_glacier_fang_shot`) + the Colliding-1115 arm
  (`projectile.rs:23097`, C# `:15397-15405`). d77 owns ONLY the Kill-tail
  cue map entry.
- d77's `kill_tail_cue` hook does not exist on d21 (grep: zero hits); d21's
  Kill seams (`projectile.rs:20236-20247`) are the storm/digtoise/RC-car
  draws — kind-disjoint from d77's 1115 entry. At the merge the hook lands
  whole and the 1115 arm resolves against d21's row: the cue plays on the
  shot's death, the AI machine runs on its life. **No duplicate play** —
  d21 ports no Kill cue for 1115 (the AI_208 body has none; its two AI
  cues are the charge `GlacierFang_Charging` `:36940` and the release
  `GlacierFang_Muzzle` `:36961`, both id-2 legacy styles 210/201 —
  `SoundID.cs:1594`/`:1585` — currently the documented sound-body skip
  in `ai_glacier_fang_shot`, the `sound_delay` latch + mana pay run).
- The d76 lane (`dd90f1629`) owns the OTHER band-adjacent Kill cues:
  1020/1105 Item62 (`:70865`), 1084 Item10 (`:70958`), 1085 Item106,
  1080 Item177 — d21's `remote_control_car_kill_draws`
  (`projectile_ai_201_210.rs:1896`) keeps the Item62 cue as the documented
  skip and ports ONLY the explicit gore draws (`Next(5)` gate +
  `Next(61,64)`/`Next(10,31)`/`Next(-20,21)` kicks). At the merge d76's
  cue map supplies the sound; the draws stay d21's — **no double-draw,
  no double-cue** (the c10/d76 1080 keep-one is those lanes' adjudication,
  untouched here).
- The AI-body charge/muzzle cues (`:36940`/`:36961`) are NOT in any
  pushed sounds lane's map (d75/d76/d77/f1 grep: only the `soundid.rs`
  generated rows, no play sites) — they remain d21's documented skip,
  a sounds-lane follow-up (the `wants_sound` seam is live: the digtoise
  joy `Some((42, 289, …))` rides it in the in-flight fold-2 work).

## S3 — all-styles-armed / kinds-rowed census + remaining gaps

**This branch (`70fe977c3`), direct source scans:**

- Dispatch: `match data.ai_style` (`projectile.rs:16797-19945`) — **103
  distinct numeric armed styles** + the kind-gated Empress catch-arm
  (`:19650`, `_ if projectile_ai_empress::handles(kind)` — styles
  171/173/179/180 + 0 for the five kinds 872/873/874/919/923). The
  band-201-210 arms: 201 `:19851`, 202 `:19863`, 203 `:19878`, 204
  `:19891`, 205 `:19910`, 208 `:19923`, 209 `:19938`, 210 `:17874`
  (`ai_yoyo_shot`, pre-d21). 206/207 ride `_ => {}` — 206 is a9's
  verify-only machine, 207 the faithful empty-AI no-op
  (`ai_snapping_stone`, `projectile_ai_201_210.rs:274`, ZERO draws ZERO
  state; no `SetDefaults` writes 207 — the whole-file write census).
- Rows: `proj_data` (`projectile/data.rs:1627`, table `:1630-4049`) —
  **428 row tuples, 112 distinct rowed styles** (comments stripped).
  Rowed-not-in-style-match: **0** (the Empress style-0 no-AI row),
  **156/164/169/206** (the `UNARMED_ROWED` pin, `projectile.rs:24240` —
  a9's four machines, verify-only here), **171/173/179/180** (the
  kind-gated Empress arm serves them — the pin's `frost_gated` set).
  The coverage pin `s1_d_4c_every_rowed_style_has_exactly_one_live_arm`
  (`projectile.rs:24173`) passes on this shape: every rowed style has
  exactly one live arm or a documented exemption.
- Band-201-210 kinds rowed: **11** — 1082/1083 (201), 1084 (202),
  1091/1117/1122 (203), 1098/1123 (204), 1105 (205), 1115 (208), 1129
  (209). 210's 1130/1135 pre-existed. 206's 1119 row pre-existed
  (`data.rs:3443`, the i25 deferral); 207 has NO kinds (F1).

**The full 1-210 space across the 21 decade-lane tips (the ledger's
evidence — per-tip dispatch scans this round, origin tips):**

| band | lane | origin tip | styles missing from the numeric style-match | resolution |
| --- | --- | --- | --- | --- |
| 1-10 | d1 | `212516a82` | NONE | armed |
| 11-20 | d2 | `f430ccf83` | NONE | armed |
| 21-30 | d3 | `740026af6` | NONE | armed |
| 31-40 | d4 | `d7c3fbc02` | NONE | armed |
| 41-50 | d5 | `e46d9eb7f` | 41,42,43,46,49 | **UNWIRED machines** — `projectile_ai_style41/42/43/46/49.rs` landed + anchors rows, NO rows NO arms (the d5 anchors' own "unwired" label; the wiring wave's claim) |
| 51-60 | d6 | `174c0d541` | NONE | armed |
| 61-70 | d7 | `8ceb9c12c` | 63,69 | **UNWIRED machines** — `projectile_ai_style63.rs` (BabySpider) + `projectile_ai_style69.rs` (Flairon recall head 404) landed + anchors rows, NO rows NO arms |
| 71-80 | d8 | `c3edc9771` | NONE | armed |
| 81-90 | d9 | `88677f9de` | NONE | armed (d9 FINAL research/281 — its registry collision is d9/f26's) |
| 91-100 | d10 | `72431e61f` | NONE | armed (d10 FINAL research/247) |
| 101-110 | d11 | `ab42de2d7` | NONE | armed (d11 census "Coverage after this branch": all ten, one live arm each) |
| 111-120 | d12 | `1320b79f1` | NONE | armed (d12 PROJ-111 RETIRE, research/127) |
| 121-130 | d13 | `1e915e254` | NONE | armed (d13 PET-LIGHT RETIRE FINAL, research/243) |
| 131-140 | d14 | `1b9c6b930` | NONE | armed |
| 141-150 | d15 | `5b7f028d3` | NONE | armed |
| 151-160 | d16 | `1d3c4a4b5` | 156 | a9's machine (`8c4fded81`), UNARMED_ROWED on d16 |
| 161-170 | d17 | `f6e5d7d3a` | 164,169,170 | 164/169 a9's; **170 kind-gated** — `projectile_ai_light_items::GLOWSTICK_KINDS` dispatch gate (the fairy glowstick 870's post-flip hover, `AI_170_FairyGlowstick` `:45662-45716`; the g2 census G2) |
| 171-180 | d18 | `3aa087cf0` | 171,173,179,180 | **kind-gated** — `projectile_ai_empress::handles` + d18's `handles_fairy_weapon` tail (the 931/1039 friendly variants) |
| 181-190 | d19 | `6fdf8592a` | NONE | armed (d19 SEAL-VERIFY, research/26g s9) |
| 191-200 | d20 | `77334040b` | NONE | armed (d20 SEAL, research/109) |
| 201-210 | d21 | `9fa9e9ba0` + local `70fe977c3` | 206,207 | 206 a9 verify-only; 207 the empty-AI no-op (F1) — **band CLOSED** |

**Remaining gaps (the honest ledger — NOT zero):**

1. **The seven UNWIRED machines** (d5's 41/42/43/46/49 + d7's 63/69):
   the AI bodies + witnesses landed on their lanes, the rows + dispatch
   arms are the wiring wave's owed step (the d5/d7 anchors say "unwired"
   verbatim). Every style in 1-210 has a ported machine SOMEWHERE; these
   seven are not yet dispatch-live on any tip.
2. **a9's four machines** (156/164/169/206): armed on a9 `8c4fded81`
   (`UNARMED_ROWED: [u8; 0] = []` at its tip — the pin empties), NOT
   merged to main `a725acb0`; every other lane keeps the four-entry
   census pin. The a9 merge resolves all four (the d12 PROJ-111 R3
   receipt: hunk-disjoint from the 62-arm/614 seating).
3. **The kind-gated styles** (170 + the Empress 171/173/179/180): armed
   via kind guards, not numeric arms — the dispatch scans that miss them
   are scan artifacts, not gaps (the g2 census G2/G5 + the f20 26e
   method note).
4. **The 207 no-op**: faithful dead code — the C# body is `{ }` and no
   kind rows it; the Snapping Stone accessory proc (1121/1131/1132,
   aiStyle 0) is the on-hit layer's follow-up, NOT an AI gap.
5. **The d21 in-flight fold-2 petting layer** (the dirty-9, see S5):
   the 204's state-5 joy + the `TryPetting` right-click chain — written,
   NOT committed at seal time, NOT run.
6. **The documented folds that stay** (ANALYSIS): gravDir −1 (upright-
   only port), Legendary difficulty (single-difficulty port, the e25
   curves live only on their unmerged branch), the 205 tracked-loop
   sound body (d80's layer), the 208 charge/muzzle AI cues (the sounds
   lanes' follow-up, S2), the dust/gore/light/particle bodies (their
   lanes), the `CalcRotations` render-only skip.

## S4 — proj-AI-1-210-all-decades-CLOSED (the ledger declaration)

**Declaration:** the projectile aiStyle space **1..210 is CLOSED at the
machine level** — every one of the C#'s 210 style arms has a faithful
port landed on its decade lane (d1-d21) or a pre-existing main arm, with
the three documented non-machine verdicts: 207 empty-AI (no kinds, the
faithful no-op), the kind-gated 170/171/173/179/180 (armed via kind
guards), and a9's four (armed on a9, merge-pending). The space ENDS at
210 (S1's C# fact) — there is no 211+ to claim.

**CLOSED means:** census-complete + machine-ported + witnesses-written
per band. **CLOSED does NOT mean:** merged to main (all 21 lanes are
unmerged siblings of `a725acb0`), dispatch-live everywhere (the seven
unwired machines + a9's four await their wiring/merge steps), or
witnesses-run (every lane is code-only; the BUILD lane owns execution).

The per-band FINAL/RETIRE receipts: d9 research/281, d10 research/247,
d11 census (26d→164), d12 research/127, d13 research/243, d19
research/26g s9, d20 research/109, d21 THIS note + research/162. The
lower bands' censuses: d1-d8 + d14-d18 carry their 26d/26e-keyed docs
(the registry §6 renames 156-161/163-164 etc. are those lanes' owed
executions; d21 executed only its own 162).

## S5 — co-lane status (the dirty-9, NOT staged)

At seal time the worktree carries **9 modified files, uncommitted** —
the in-flight **fold-2 petting layer** (the 204's `player.petting` fold
unfolded): `player.rs` (+372: `PettingInfo`/`pet_animal`/
`update_petting_animal`/`stop_vanity_actions`/`change_dir` + the
`ItemCheck_ApplyHoldStyle` petting arm + 5 witnesses), `world_sim/
editing.rs` (+166: `try_petting` + the right-click chain + the
`BlockInteractionWithProjectiles` decrement), `audio.rs` (+75:
`PROJ_CUSTOM_CUES` + the id-42 trackable arm + the digtoise-joy
witness), `projectile_ai_201_210.rs` (+98: the state-5 joy LIVE +
`is_interactable`/`get_petting_info` + the `who_am_i` arg), `app/mod.rs`
(+16: the two App fields), `world_sim/player.rs` (+19: the
`TryGetTarget` feed + the `CanVisuallyHoldItem` refusals),
`player_lights.rs` (+30: the 4952 petting gate + witness),
`projectile.rs` (+1: the dispatch `who_am_i` arg), `app/flow.rs` (+2:
the test-rig fields). Total +747/−32.

**Idle verdict:** last write 13:03:07 (`audio.rs`), seal probe 15:57 —
~2h54m idle; SHA256 prefixes stable across both probes (flow
`EFF11E0F8BE8EC09`, mod `DAD7207B1579DB81`, audio `F12A8B14719D79C7`,
player `8811D86B5653470E`, player_lights `C6DF993F48EE8101`, projectile
`DFA8CDD04C5DADC8`, ai_201_210 `332B2B08A0FC0082`, editing
`2DFD68EF9343FAB7`, world_sim/player `B9E4159CFB4DB710`). The work is
statically coherent (every referenced symbol resolves — the
`pet_animal`/`can_snap_to`/`solid_tile_allow_bottom_slope`/
`remove_all_grappling_hooks`/`tile_in_reach_rect` chain re-read this
round) and adds 7 more `w4_d21_*` witnesses (31 total with the dirty
tree). **NOT staged, NOT committed** — the d20 SEAL precedent (its
dirty-2 T4a co-lane stayed unstaged): the fold-2 lane owns its commit.
This SEAL commits ONLY the docs (this note + the 162 rename + INDEX +
ANALYSIS + PORT_MAP + anchors).

## S6 - the fold-2 keep-one adjudication (g10 canonical, DISJOINT HALVES)

Post-landing addendum (the fold-2 work COMMITTED as `d3a7ff663`; S5's
dirty-9 census is history). The player-side petting machine duplicates
g10's canonical pets host (`origin/task/W4-g10-npc-kind-wire-3` tip
`7414ba339`: `player_petting.rs` 664 ln / 10 witnesses, landed
`91c8ed1eb`; the pets-family bible g10 research/360 §5 names
`player_petting.rs` g10-owned). Both files re-read at their live tips.

**VERDICT: DISJOINT HALVES — COMPOSE CONTRACT.** Not whole-drop: d21's
player-machine core is a semantic subset of g10's wired superset (g10
adds the NPC `GetPettingInfo` ladder, the talk gate, `PetMount`, the
achievement queue, the net bits, the Chillemt 0.36 arm, `math_sign`),
but d21 owns the whole PROJECTILE lane g10 explicitly scopes out
(g10 module census: "The Pal Digtoise's joy cue (`Projectile.cs:38517`)
is the projectile path's, out of this lane") plus three player seams
g10 lacks (`change_dir`, the `sit_down` vanity stop, the
`from_projectile` ctor).

The canonical KEEP-ONE table lives in ANALYSIS "Fold-2 keep-one
adjudication" (the pets-family convention — the bible paragraph is
ANALYSIS-canonical). Summary:

- **13 DROP hunks** onto g10 at the merge: the `PettingInfo` struct +
  `Default` (`player.rs:587-625`), the `petting` field/init
  (`:1190-1195`/`:2728`), the `sleep_down` strip (`:3201-3203`),
  `stop_petting_animal` (`:3234-3237`), the `stop_vanity_actions`
  helper (`:3239-3246` — g10 inlines all four sites, d21 ports only
  two), `pet_animal` (`:3248-3313`), `update_petting_animal`
  (`:3315-3355`), the `:26124` call site (`:5984-5991`), the inline
  back-arm pose (`:8252-8277`), the 5 player-machine witnesses
  (`:21908-22016`), the 4952-light gate (`player_lights.rs:484-495`),
  the `petting_blocks_hold` refusals (`world_sim/player.rs:1012-1018`).
- **10 UNIQUE-KEEP groups** (each verified absent from g10's tip): the
  `from_projectile` ctor + the proj `TryGetTarget` feed
  (`petting_target_bottom`), the `sit_down` petting stop (C#
  `PlayerSittingHelper.cs:124` — g10's `sit_down` misses it),
  `change_dir` (`Player.cs:42091-42116` — C# `PetAnimal:33390` calls
  it; g10's bare facing assign misses the pulley-2 reposition + the
  composite/itemRotation mirrors), the 4952-gate witness, the
  `TryPetting` right-click seam + chain (`world_sim/editing.rs`,
  `Main.cs:37925-37981`), the `BlockInteractionWithProjectiles`
  decrement (`Main.cs:16849-16852`), the two App fields + test rig,
  the `pal_digtoise_joy` id-42 trackable arm (`audio.rs`
  `PROJ_CUSTOM_CUES` — g10's `play_proj_sound` refuses `id != 2`;
  g10 `soundid.rs:492` already rows style 289, no row conflict), the
  proj `get_petting_info` ladder + `is_interactable` + the 204 state-5
  joy LIVE + the `who_am_i` dispatch arg + 3 witnesses.
- **5 splices** (S1-S5, ANALYSIS): the ctor relocates into g10's impl;
  g10's `pet_animal` gains the proj-bottom input + `change_dir`; its
  `update_petting_animal` gains the proj `TryGetTarget` arm
  (`PlayerPettingInfo.cs:70-75`); `try_petting` drains g10's
  `PetAnimalOutcome` (the achievement fold retires onto g10's
  `pending_pet_achievement`); d21's `signum` facing pick dies with the
  dropped machine halves (g10's `math_sign` is the C#-faithful keeper
  at offset 0).

Adjudication evidence (code-only, no builds): `git show d3a7ff663`
hunk-by-hunk vs `git show 7414ba339:crates/tr-app/src/player_petting.rs`
+ the g10-tip greps (`try_petting`/`from_projectile`/`change_dir`/
`petting_target_bottom`/`block_interaction`/`PROJ_CUSTOM_CUES` — all
zero on g10) + the C# re-reads (`Player.cs:33358-33399/:33470-33490`,
`PlayerPettingInfo.cs:54-76`, `PlayerSittingHelper.cs:113-131`). No
code changed; g10's file untouched. Drop-hunk count: 13.

## Verification (code-only contract)

- No cargo/rustc/rustfmt/tests/dotnet/graphify invoked.
- C# re-reads: `Projectile.cs` `:23199-23228` (AI head), `:36760-36811`
  (chain tail + AI_210), `:36930-36968` (the 208 charge cue + release
  muzzle), `:75498-75515` (the 1115 Kill cue); `SoundID.cs:1585/:1594`
  (the GlacierFang styles); the whole-file aiStyle-write census (809
  writes / 210 distinct / max 210 / zero 207).
- Rust re-reads: the dispatch arm scan (103 numeric + the Empress gate),
  the row-table scan (428 tuples / 112 styles), the Kill seams
  `:20236-20247`, the coverage pin `:24173-24258`, the fold-2 symbol
  resolution census.
- Git censuses: the 21 origin decade-tip dispatch scans (S3 table), the
  all-tip research-number sweep (max 298), the 308 history check (zero),
  the d77/d76/f1 cue-map greps, the a9 UNARMED_ROWED-empty check.
- Witnesses: 24 committed WRITTEN NOT RUN; the 7 fold-2 witnesses stay
  with their lane.
