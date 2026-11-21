# 114 — Lantern-night / birthday-party MASTER reconcile (merger assist)

Code-only reconciliation of the c39 umbrella lane against every sibling lane
that touches the same slice. The reconcile pass itself changed **no lane
code** — it is the per-function KEEP/DROP verdict sheet for the merge, in
the shape of research/70 (which it extends: 70 adjudicated f38/d120/f34/f41
among themselves; this note folds **c39** into that sheet and adjudicates
the lantern-side 2-ways 70 never saw). **The c39-side deviations are now
EXECUTED on this branch** — see §7 (the per-item landing record); the
sibling-side KEEP pieces (f40's `render/lantern_sky.rs` module, d55's gore
table, f34's StationBuff route, d120's GetChat re-pin, f38's naming layer)
still land at the merge, not here.

Every C# claim below was re-read from the decompiled windows directly
(≤100-line reads, citations inline); every port claim was read from the
lane's own diff (`git diff a725acb0..<tip>`) or `git show <tip>:<file>` —
not memory. All lanes are siblings off wave-3 `a725acb0`; none contains any
other's work.

| Lane | Tip read | Scope (this slice) |
| --- | --- | --- |
| **W4-c39** (this branch) | `d3e63863` | the umbrella: triggers + scheduler completion (anniversary/prune/broadcasts) + luck feed + wind/rain gates + BOTH event skies + hats (draw AND gores) + aiStyle-166 + cake right-click + SugarRush + freeCake talk arm |
| **W4-f40** | `4e2deb3b` (origin tip; no f40b advance) | lantern second half: luck feed, shop wish row, `on_game_event_cleared(id)` + 16 boss-death wires, `render/lantern_sky.rs` (depth-band draw), world-load reset |
| **W4-f41** | `8e23136e` | research/70 itself + (at `91f195fa`) monolith/break-records/freeCake chain/SugarRush/CakeSlice |
| **W4-d17** | `84126544` | aiStyle-166 as `ai_dove`/`dove_movement`/`dove_bounce` inside `projectile.rs` (+ 161/162/163/168, out of scope here) |
| **W4-f38** | `7f647fda` | naming layer + keyed party broadcasts + `check_dialogue` FreeCake structure (per 70) |
| **W4-d120** | `6381e43e` | GetChat re-pin + live feeds + dusk freeCake clear (per 70) |
| **W4-d55** | `ec78d152` | `npc_hit_effect_a.rs` — HitEffect gore table wave A incl. the party-hat block (unwired, `#![allow(dead_code)]`) |

---

## 0. New C# findings that change BOTH sides' comments (fix at merge)

1. **Case 4 is Mourning Wood, NOT Pumpking.** `NPC.cs:79210-79221`
   (re-read): type 327 **Pumpking** → `SetEventFlagCleared(ref
   downedHalloweenKing, 5)` — case **5**, the switch's **default arm**, so
   Pumpking's first kill **ARMS** the lantern night. Type 325 **Mourning
   Wood** → id **4** — one of the three skips (`:78789-78792`). Both c39's
   helper doc ("cases 4 (Pumpking)…") and f40's ("cases 4 (Pumpking)") carry
   the wrong name; f40's witness comment too. The id TABLE both ports use
   (skip 4/21/22) is correct — only the label is wrong. Also 21 = Everscream
   (344, `:79225`), 22 = SantaNK1 (346, `:79237`); Ice Queen (345) is id 20
   (default arm, arms).
2. **c39's "none of those three kinds is ported" claim is false.**
   `NpcKind::Pumpking`/`Everscream`/`SantaNK1` exist (`npc.rs:832/1047/859`,
   AI styles 57/58/61 wired at `npc.rs:4873-4886`). What is absent is their
   **death-flag arms** (no `NpcKind::Pumpking` arm in `world_sim/npcs.rs`'s
   dead-match; `downedHalloweenKing/Tree/ChristmasTree/IceQueen/Santank`
   have no port fields). So c39's id-less helper is *observationally* right
   today but structurally wrong the moment a moon-boss death arm lands —
   and it cannot express "Pumpking arms, Mourning Wood skips". f40's
   id-parameterised helper is the shape that survives.
3. **`UsesPartyHat()` has NO life gate** (`NPC.cs:66611-66634`, re-read):
   the terms are type exclusions 441/37/633, Dryad-variation-1,
   `ForcePartyHatOn`, the dummy gates, `frame.Height > 0 && townNPC` →
   `PartyIsUp`. The **death** gate lives in the CALLER: `HitEffect`
   `:81660` = `townNPC && life <= 0 && UsesPartyHat()`. c39's
   `party_hat::uses_party_hat` bakes `n.hp > 0` into the predicate
   (`party_hat.rs:55`) — correct for the draw seam, **fatal for the death
   seam**: c39's `update_npcs` gore block (`world_sim/npcs.rs:5089`) calls
   it on `dead` clones (`hp <= 0`), so the predicate is always false and
   **c39's party-hat death gores never fire**. d55's split (caller-resolved
   `inputs.uses_party_hat`, block gate `town_npc && npc.hp <= 0 &&
   inputs.uses_party_hat`, `npc_hit_effect_a.rs:312`) is the faithful
   shape. The merge must resolve `UsesPartyHat` WITHOUT the hp term for
   HitEffect (keep the hp term only in the draw caller, where the C#
   `UpdateNPC`-drawn roster supplies it implicitly).
4. **The dusk/prune farewell is keyed `Lang.misc[99]` = "Party time's
   over!"** — re-verified in
   `Terraria.Localization.Content.en-US.Legacy.json` (`"99": "Party time's
   over!"`). c39 carries the stale base literal `"Everyone has gone home!"`
   in BOTH `check_night_party` (`world_events.rs:190`) and its new
   `update_time_party` prune broadcast (`:322`). research/70 §1i already
   ruled the literal a bug (f38's keyed row wins); the ruling extends to
   c39's second copy.

---

## 1. Per-function tables

### 1a. First-clear → lantern-night trigger (c39 vs f40, 2-way)

C# truth: `SetEventFlagCleared` (`NPC.cs:78759-78767`) fires
`OnGameEventClearedForTheFirstTime(gameEventId)` only on the flag's
false→true edge; the switch (`:78769-78793`) arms
`NextNightIsLanternNight` on default + 10/16/17/18, skips ONLY 4/21/22
(§0.1). Call-site census (grep, all 22 hits): invasion clears use ids
**0/1/2/3** (`Main.cs:64336/64341/64346/64351` — goblins/frost/pirates/
martians, all default arm); moon bosses 5/4/21/20/22 (`:79213-79237`);
bosses 6/7/23/25/24/8/9/10/11/17/12/13/14/15/18/16/19
(`:79346-79490`). WoF's id-19 edge is the PRE-kill `Main.hardMode`
captured at `:79484` before `StartHardmode()` (`:79490`).

| Piece | c39 | f40 | Verdict |
| --- | --- | --- | --- |
| helper | `on_game_event_cleared_first_time(ev, was_down)` — no id, comment claims the 3 skip-kinds unported (§0.2 false) | `on_game_event_cleared(ev, game_event_id, first)` — id table, 4/21/22 skip, anchor-bearing | **KEEP f40's helper** (structural 1:1; survives moon-boss death arms). Fix its case-4 label (§0.1). DROP c39's helper; re-point its call sites to f40's with the C# ids |
| boss-death wires | 16 arms (EoC/EoW-first-match/QueenBee/Skeletron/mech trio incl. Twins both-eyes/Plantera/WoF/KingSlime/Golem/Fishron/Deerclops/Cultist/MoonLord) | the same 16 arms, ids cited per `DoDeathEvents` row | one copy; **f40's id-passing form**, c39's WoF shape (capture `was_down` outside the guard, call unconditionally — the `:79484` capture) is the more structural of the two equivalent WoF arms |
| invasion-clear wires | YES — all 4 types, `was_down` captured per flag (`update_invasion_state`) | **NO** (f40 never touches `update_invasion_state`) | **KEEP c39's invasion arms**, re-pointed at f40's helper with ids 0/1/2/3 |
| EoW/Brain duplicate-flag note | c39 documents the second-match duplicate sees `was_down == true` (no double arm) | f40 wires only the first match | equivalent; keep c39's comment |
| moon-boss / Empress / QueenSlime arms | absent (no death flags ported) | absent | census note, both lanes: ids 5/4/21/20/22/23/24 unwired — the id table must be ready for them (f40's is) |
| anchors | none (c39 added NO anchor rows — see §4) | `world_events.rs\|fn on_game_event_cleared` | f40's row survives the KEEP |

Witnesses: f40 `w4_f40_the_first_event_clear_arms_the_next_lantern_night`
(id table + edge + persist-through-moon-boss) KEEP, fix the case-4 label;
f40 `w4_f40_the_first_boss_kill_arms_the_next_lantern_night` (EoC
integration) and c39 `w4_c39_the_first_boss_clear_arms_the_next_lantern_night`
(EoC integration + **invasion-clear arm**) — keep ONE EoC integration
(f40's) + RE-DERIVE c39's invasion tail onto f40's helper signature (the
asserts stand; only the call shape changes).

### 1b. LanternSky (c39 `event_skies.rs` vs f40 `render/lantern_sky.rs`, 2-way)

C# truth re-read whole: `LanternSky.cs:61-282`. Both ports carry the same
state machine (`GenerateLanterns` `maxTilesY/4` + X-then-Y draws `:93-95`,
`ResetLantern` depth ladder `(1-i/len)*4.4+1.6` / `-1.5-2.5*NextDouble` /
`Next(3)` / `(2000+Next(1200))*2` `:103-111`, `Update` ±0.01 opacity / sine
bob / float / wind drift / 300-px reset-or-leave `:113-152`, private
`UnifiedRandom` `:71`). The differences are all in the DRAW and the seams:

| Piece | c39 | f40 | Verdict |
| --- | --- | --- | --- |
| draw structure | ONE batch in the sky flush slot (`draw_lanterns`, after surface layers; recorded layering deviation) | per-`DrawToDepth` band: the `num`/`num2` index walk (`:169-187`) threaded through `draw_sky`/`AmbiencePass` boundaries + the remainder flush | **KEEP f40** — the depth-sandwich interleave IS the C# structure (`SkyManager.DrawToDepth`); c39's batch draws far lanterns over near trees |
| body tint | float `(250/255)*tier*body_a` with `body_a = (120/255)*tier*opacity` — **tier applied TWICE to rgb + an extra 120/255 factor** | `(250.0*num3) as u8` byte truncation per channel (`:224`'s int casts) × `_opacity` once | **KEEP f40** — c39's rgb is `tier²·(120/255)` off the C# byte product (a colour bug, not a convention) |
| glow | `[1,1,1,glow_a]` + explicit `BlendMode::Additive`, 3 copies rotated `(0,2)` offsets | `[glow_a,glow_a,glow_a,0]` premultiplied-source convention (`sprite.wgsl` tile-flame note), same 3-copy geometry | KEEP f40's form with its `push_screen_rot` seam (one pipeline convention per pass); c39's `push_back_sprite_origin_rot_blend` (`tr-render/sprite.rs`) stays ONLY if PartySky's retrofit needs it (§1e) |
| view maths | zoom-divided screen anchor (`camera - screen*0.5/zoom`), back-range world-space push | unscaled `screenPosition` convention (`camera - screen*0.5`), integer-halved centre (`screenWidth>>1`), zoom-scaled screen push, `>=` cull edges (`:234` `Rectangle.Contains`) | **KEEP f40** (matches the bg/ambience convention + the C# int halves/cull) |
| Generate band casts | f32 arithmetic `(y*0.7 - h) as i32` | `(y as f64 * 0.7 - h as f64) as i32` — the C# `:93` double casts | KEEP f40 |
| private RNG seed | `world_seed ^ 0x1A2B3C4D` (per-world scatter) | pinned 0 | either is a recorded deviation (C# time-seed unobservable); **prefer c39's world-seed form** grafted into f40's module (per-world variety, still deterministic) — note the swap in ANALYSIS |
| activate/deactivate edges | `weather.rs` off `update_time_lanterns`' `Option<bool>` (same seam) | same seam | one copy (identical semantics; f40's activate args are the unscaled convention) |
| 60 Hz cadence | `event_skies_phase` accumulator, `max(1,dayRate)` inner loop, `!paused` gate | `lantern_sky_step: FixedStepper` inside the ambience phase, same loop/gate | one copy; f40's `FixedStepper` is the house seam — extend it to drive PartySky too (§1e) |
| world-load reset | **absent** | `reset()` + `enter_loaded_world` `world_clear` + sky reset (`WorldGen.cs:6996` / `SkyManager.Reset`) | **KEEP f40's seam**, extend to the party sky |
| menu/PauseSkies folds | phase gate / in-world-only pass | same folds, recorded | tie |
| anchors | none | `render/lantern_sky.rs\|struct LanternSky` | f40's row survives |

**Net: f40's `render/lantern_sky.rs` is the surviving LanternSky.** From
c39's `event_skies.rs` the LANTERN half is dropped wholesale (state fns,
draw, `SkyCtx` lantern fields); the PARTY half survives (§1e). c39's
`assets.rs` `extra_134` lookup duplicates f40's `lantern_sky` lookup — keep
ONE (`Extra_134` appears once in `STATIC_SHEET_NAMES`).

Witnesses: f40's 3 (depth ladder + determinism, opacity walk + leaving,
surface-band reset) KEEP; c39's `w4_c39_the_lantern_depth_ladder_runs_far_to_near`
/ `…opacity_ramps_with_the_event` are subsets — DROP (same asserts, weaker
rig). c39's world-seed determinism aspect: RE-DERIVE one twin-seed assert
onto f40's module if the seed swap is taken.

### 1c. AI_166 (c39 `projectile_ai166.rs` vs d17 in-`projectile.rs`, 2-way)

C# truth re-read: `AI_166_Dove` `Projectile.cs:47540-47610` (frame latch
`Next(4)` `:47546`, 4/6-tick cadence, lantern rot `vx*0.2` / bob
`sin(timeLeft%200/200·2π)*0.25` / 0.5-cap ×0.8 damp / ±1.5 clamp / wind
`0.0525*(wind||0.1)` ±2 clamp / light `(1,0.6,0.1)*(1-alpha/255)` `:47569`;
dove rot `vx*0.15` / −0.025 climb cap −3 / ±5 clamp; fade +3 under 90,
alpha-255 `Kill()`); bounce arms `:18336-18361` (dove X×−0.8/Y×−0.5;
lantern X×−1/Y×−1, rising-Y ×0.25); `Update` order: `AI()` `:16775` →
`HandleMovement` `:17015` **unconditionally** (the `!active` return is at
`:17027`, AFTER movement) → `AutomaticallyChangesDirection` `:17016-17026`
(neither row sets `manualDirectionChange`).

| Piece | c39 | d17 | Verdict |
| --- | --- | --- | --- |
| AI body | verbatim (`ai_166_dove`) | verbatim (`ai_dove`) — line-for-line the same maths/order | tie; **KEEP d17's in-file form** (the pool's module convention; c39's separate file exists only because its movement needed `pub(crate)` seams) |
| alpha kill | `self.kill()` (needed the `pub(crate) kill` visibility bump, `d3e63863`) | inline `kill_time_left/active/time_left` writes | d17's inline form needs no visibility change — **revert c39's `kill` pub(crate) bump** if nothing else uses it |
| movement | per-axis `collides_rect` probes, zero blocked axes, bounce; **NO `Next(3)` draw** | `collision::tile_collision` + the consumed **`Next(3)`** (`:17948`, the ai026_c precedent) + bounce + integrate | **KEEP d17** — the blocked-move cut roll is a real stream draw; c39's probe silently skips it (RNG divergence on every bounce) |
| bounce fns | `lantern_bounce`/`dove_bounce(last, hit_x, hit_y)` | `dove_bounce(last)` off the collision-resolved velocity delta | same maths; d17's delta form matches the C# `velocity != lastVelocity` gates exactly — KEEP d17 |
| dispatch arm | `ai_166_dove` then `if active { dove_movement }` — **skips movement on the kill tick** | `ai_dove` + unconditional `dove_movement` + the `direction` sign write (`:17281-17288`) | **KEEP d17** — C# runs HandleMovement after the AI regardless (`:17015` precedes the `:17027` gate), and c39 drops the direction write |
| proj_data rows | 851/855 identical tuples + netImportant | identical + **`proj_frames => 4`** (`Main.cs:8657/:8660`) | **KEEP d17's rows** (superset: c39 never adds the 4-frame sheet count — the draw would use 1 frame) |
| light cast | **LIVE**: `(1,0.6,0.1)*(1-alpha/255)` at the lantern centre into `entity_lights` (`app/frame/draw.rs`, per-tick feed) | skipped ("no pool light layer") | **KEEP c39's light feed** — it is the frame-side seam the pool cannot reach; graft it beside d17's machine (the `:47568-47569` citation stays) |
| anchors | none | none (d17 added no rows) | add ONE row for the surviving machine (§4) |

Witnesses: d17's 2 (`w4_d17_the_dove_climbs_and_strides_four_ticks` —
rows/frames/latch/climb/wrap census; `w4_d17_the_lantern_rides_the_wind_and_bounces`
— wind fallback/bob phase/bounce + the `Next(3)` census) KEEP. c39's 4:
frame-latch-once (subset of d17 #1) DROP; fade-and-die-at-255 KEEP (d17
has no fade witness); lantern-bounce-upward-damp + dove-bounce-scales
(subsets of d17 #2's bounce asserts, but pin the DOWNWARD-Y arm d17
doesn't) — KEEP the downward-Y assert, fold into d17 #2 or keep c39's
bounce witnesses re-pointed at d17's `dove_bounce(last)` signature
(RE-DERIVE: the two-arg hit-flag form dies with c39's fns).

### 1d. Wish + luck (c39 vs f40)

| Piece | c39 | f40 | Verdict |
| --- | --- | --- | --- |
| shop wish row (`Chest.cs:2309-2312`, 4702 gated on `LanternsUp`) | **absent** | `StockWhen::LanternNight` + `ShopFlags.lanterns_up` + the shop-open feed + num-order witness, anchor-bearing | **KEEP f40 wholesale** (c39-unique? NO — f40-unique) |
| luck +0.3 feed (`Player.cs:29146-29149`) | `p.luck_state.lantern_night = lanterns_up(...)` before `update_with_grapple` (`world_sim/player.rs:~1104`) | same write beside the auras/zones snapshot (`:~583`) | behaviourally identical (the flag flips only at boundaries/UpdateTime, never mid-player-step — both lanes' recorded rationale). **KEEP ONE — f40's** beside-zones placement (its `luck.rs` doc rewrite rides it); DROP c39's duplicate write |
| `recalculate_luck` +0.3 term | pre-existing (M4-EVENT-4 neutral input) | same | untouched by both |

Witnesses: KEEP BOTH lanes' luck witnesses (disjoint rigs): c39
`w4_c39_the_lantern_night_adds_luck_live` (underground isolation + the
down-tick CLEAR) and f40
`w4_f40_the_lantern_night_adds_the_luck_bonus_through_the_player_tick`
(manual + genuine + down). f40's shop witnesses (row gate + num order)
KEEP.

### 1e. PartySky — c39 UNIQUE?

**YES, unique.** No sibling touches `PartySky.cs` (f40 is lantern-only;
f41's party work is scheduler/furniture/gift). c39's balloon half of
`event_skies.rs` is the only port: `GenerateBalloons` (`:93-111`),
`ResetBalloon` (near-first ladder `i/len*1.75+1.6`, 2-sheet pick + 1/30
rare third, `:113-123` — re-read, matches), `Update` (the
`MultipleSkyWorkaroundFix` dayRate-0 skip `:127-131`, 42-wrap frame
counter, rise + wind drift, reset/deactivate + the second 1/30 roll
`:153`, the `_active=false;_active=true` quirk `:164-168` kept verbatim),
`ramp_party_opacity` (`SceneState.cs:106`), `draw_balloons` (sky-tint
ladder, 3×3 grid, top-left origin). **KEEP wholesale**, with two recorded
follow-ups for the merger (NOT silent fixes):

1. The draw rides c39's flush-slot batch — the SAME layering deviation
   §1b rejects for the lanterns. `PartySky.Draw` (`:171-245`) is also a
   `DrawToDepth` band member; retrofitting it into f40's depth-boundary
   machinery is the faithful shape. Until then the deviation stands
   recorded for the balloons only.
2. The activate/deactivate edges (`weather.rs` off `update_time_party`'s
   `Option<bool>`) and the world-load reset need the f40 seam extended
   (`SkyManager.Reset` covers both skies). c39's `SkyCtx` zoom-divided
   anchor should move to the unscaled convention when the retrofit lands.

The scheduler half that FEEDS it is §1f. Anchors: none yet — add (§4).
Witnesses: c39's 3 balloon witnesses (depth ladder near-to-far, 42-wrap,
party opacity ramp) KEEP (unique).

### 1f. Party scheduler completion (c39 vs f41-base/f38/d120 — extends 70 §1a/1i)

C# truth re-read: `CanNPCParty` (`BirthdayParty.cs:60-71`),
`NaturalAttempt` (`:73-143`: the anniversary `maxValue = 7` `:84-88`,
`freeCake = true` `:106`, keyed broadcasts `:127-139` BEFORE
`SendData(7)`/`CheckForAchievement` `:140-141`), `UpdateTime`
(`:183-219`: the edge → sky activate/deactivate, the `RemoveAt(i)`
**no-decrement** prune quirk `:200-206`, the emptied-roster genuine-party
drop + `Lang.misc[99]` broadcast unless `ManualParty` `:207-215`).

| Piece | c39 | siblings | Verdict |
| --- | --- | --- | --- |
| `can_npc_party` helper | YES (`is_town_npc_like` + TaxCollector exclusion; bound forms fold) | f41/f38/d120 keep the inline base fold | **KEEP c39's helper** (one predicate for gather + prune, the C# shape) |
| anniversary `Next(7)` arm + `tenth_anniversary` param | YES (`check_morning_party` 5-arg, fed from `SeedFlags::TENTH_ANNIVERSARY` in `clock.rs`) | none (f41 keeps the 4-arg base) | **KEEP c39** — unique; f41's `w4_f41_the_free_cake_flag_arms_on_the_fresh_party_only` needs the extra `false` arg (mechanical RE-DERIVE, asserts stand) |
| `free_cake` field + `:106`/`:65222`/`:6945` writers | all three sites | identical copies in f38/d120/f41 | per 70 §1a: ONE copy (d120 wording); c39's copies are the same lines — drop-out is free |
| dawn `Game.BirthdayParty_{n}` broadcast | YES, keyed, but **AFTER `notify_progression_net(25)`** in `clock.rs`, names via `lang::npc_name` | f38: keyed, BEFORE the achievement tail, names via `npc_names::given_or_type_name` | **KEEP f38's ordering + naming layer** (70 §1i); graft c39's clock-arm wiring onto it (the C# order is broadcast `:127-139` → SendData(7) → achievement `:141`) |
| `update_time_party` roster prune + `RemoveAt` skip quirk + gone-home broadcast + `Option<bool>` sky edge | YES — the whole `:183-219` body | base is edge-bookkeeping only; no sibling ports the prune | **KEEP c39 wholesale** — unique, quirk-faithful (its 2 witnesses pin the stride), EXCEPT the stale literal → keyed `lang::misc(99)` (§0.4) in BOTH `update_time_party` and `check_night_party` (f38's row) |
| dusk `freeCake` clear (`Main.cs:65222`) | `clock.rs` StartNight before the event pair | identical in d120/f41/f38 | one copy (70 §1a); d120's anchor `clock.rs\|app.world_events.free_cake = false` survives any |
| `LanternGates::from_world` + live `lanterns_can_start` persistence (`LanternNight.cs:42-48`) | YES — boss/invasion mid-night drops the GENUINE lanterns; manual persists | f40 keeps the base `!day_time`-only gate | **KEEP c39** — unique fidelity (f40's weather hunk must take c39's gate line, not its own) |
| wind freeze (`Main.cs:58856-58859`) + rain gates (`:64825-64831`/`:64868-64871`/`:64890`) | YES — `Wind::lantern_night` own flag, `Rain::update_time` 3 new gate args + storm-deck clamp + stop-rain arm, weather-tick feeds | no sibling touches wind/rain | **KEEP c39 wholesale** (unique; 4 witnesses incl. the fire-seeded integration twin) |

### 1g. Hats (c39 vs d55)

| Piece | c39 | d55 | Verdict |
| --- | --- | --- | --- |
| `UsesPartyHat` predicate | `party_hat.rs::uses_party_hat` — full exclusions incl. Zoologist 633 + Dryad var-1, **but `hp > 0` baked in** (§0.3) | caller-resolved `inputs.uses_party_hat` (no ported predicate) | **KEEP c39's predicate for the DRAW**; for HitEffect the caller must resolve it WITHOUT the hp term (split the fn or pass a `life`-agnostic flag — the C# property has no life gate) |
| `GetPartyHatColor` hash | verbatim (`chars().count()` + first char) | verbatim (`len()` + first byte; ASCII-equivalent) + the `uses_party_hat` first gate (`:66638`) | tie on maths; **KEEP d55's** (it carries the property's first gate; one hash in the tree) — c39's draw caller adapts (`PartyHatColor` enum ↔ discriminant is a 1:1 map) |
| hat DRAW (`Main.DrawNPCExtras` `:26914-27188`: strip columns, HatOffsetY, framing groups, attack lift, anchor/origin/2-px trim, Extra_72 lookup, `entities.rs` seam) | YES — the whole layer | **absent** | **KEEP c39 wholesale** (c39-unique; d55 is gores only) |
| death gore table (`NPC.cs:81660-81728`, re-read) | `party_death_gores` — base rows only, NO shimmer variants, wired via the **dead hp>0 gate** (§0.3) on `RngKey::UpdateNpcs` (recorded deviation) | `party_hat_block` — full (normal, shimmer) pairs incl. 229's spawn-NOTHING shimmer arm (`:81682-81685`), `flag=true` hat always, strike-stream RNG contract, unwired (`#![allow(dead_code)]`) | **KEEP d55's table**; **DROP c39's `party_death_gores` + the `update_npcs` death-site block** (`:5082-5115`). Wiring d55's `hit_effect` into the strike path is its lane's declared follow-up — the merge must NOT keep c39's block as a "temporary" wire (it never fires, and its stream is the wrong one) |
| anchors | none | `npc_hit_effect_a.rs\|pub fn hit_effect` | d55's row survives; add c39 draw rows (§4) |

Witnesses: c39's hash-lands-in-1..5 + columns + draw-side KEEP (re-point
the hash witness at the surviving hash if d55's wins); c39's
`w4_c39_the_death_gore_rows_match_the_colour` DROP with the fn (d55's 7
witnesses cover the table incl. shimmer); d55's 7 KEEP.

### 1h. Cake side (c39 vs research/70's f34/f41 execution)

research/70 §1e/§1f already picked **f34's `StationBuff` dispatch + app
arm** (anchor `edit.rs|621 => Some((192, 7_200))`) and **f34's single
`192 =>` SugarRush arm**, dropping f41's `CakeSlice` route. c39 carries a
**THIRD copy** of the same route:

| c39 piece | Verdict under 70 |
| --- | --- |
| `tr-core/src/edit.rs` `TileInteraction::CakeEat` + the `t.kind == 621` dispatch + witness `w4_c39_interact_at_dispatches_the_cake_tile` | **DROP** — f34's `StationBuff { buff_type, buff_time }` covers 621 in the seven-row chain order; two dispatch arms for 621 cannot coexist |
| `doors.rs` `CakeEat` app arm (`add_buff(192,7200)` + unpitched `play_at("Item_2")`) + `interaction_events` arm + `app/mod.rs` `TR_TOGGLE` log arm | **DROP** — f34's shared `StationBuff` arm is the survivor (same unpitched e28 stand-in; f41's pitched seam stays 70 §1f's recorded follow-up) |
| `player.rs` `192 => { pick_speed_bonus += 0.2; move_speed += 0.2 }` | **DROP** — byte-identical to f34's/f41's; two `192 =>` arms in one match do not compile (70 drop 5 extends to the third copy) |
| witness `w4_c39_sugar_rush_adds_pick_and_move_speed` | **KEEP in the union** (a third disjoint rig: the full `Player::update` tick vs f34's grant/decrement and f41's rebase/expiry) — it tests the surviving arm unchanged |
| freeCake talk arm (`talk_to_npc`: `kind == PartyGirl && free_cake` direct-fire, world-drop gift at the NPC centre via `spawn_item_drop_px`, then Cake text) | **DROP** — this is d120's collapse shape with a WORSE gift arm (the C# `GetOrDropItem` fills the inventory FIRST, `Player.cs:7193-7200`; c39 always world-drops). 70 §3's ONE chain (f38 structure + f41 alive gate + d120/f41 gift-before-text order, single-slot fill, global stream) supersedes it. d120's anchor marker `app.world_events.free_cake = false` survives in 70's recipe step 3 |
| live-feed hunks (`party_is_up`/`lanterns_up`/`downed_moonlord` in the ctx) | **DROP** — d120's superset ctx (70 drop 8) |

---

## 2. KEEP sheet (one winner per function)

| Function | KEEP | Source |
| --- | --- | --- |
| first-clear helper | `on_game_event_cleared(ev, id, first)` (fix case-4 label) | f40 |
| boss-death wires | id-passing form, c39's WoF capture shape | f40 ⊕ c39 |
| invasion-clear wires | the 4 arms, re-pointed at f40's helper (ids 0/1/2/3) | c39 |
| LanternSky module + depth-band draw + world-load reset | `render/lantern_sky.rs` (world-seed RNG graft optional, noted) | f40 |
| PartySky state + draw + opacity ramp | `event_skies.rs` party half (layering retrofit = recorded follow-up) | c39 |
| aiStyle-166 machine + movement + rows + projFrames | `ai_dove`/`dove_movement`/`dove_bounce` in `projectile.rs` | d17 |
| lantern entity-light feed | the `draw.rs` per-tick cast | c39 |
| shop wish row | `StockWhen::LanternNight` + feed | f40 |
| luck feed | ONE write, beside auras/zones | f40 |
| wind/rain lantern gates + live persistence gate | `Wind::lantern_night`, `Rain::update_time` gates, `LanternGates::from_world` | c39 |
| party scheduler: anniversary/prune/RemoveAt quirk/sky edges/`can_npc_party` | the 5-arg `check_morning_party` + `update_time_party` | c39 |
| dawn/dusk broadcasts + naming | keyed rows, broadcast-before-achievement, `given_or_type_name`; `misc[99]` farewell replaces BOTH c39 literals | f38 |
| freeCake writers | one copy of each site | 70 §1a |
| FreeCake gift chain | 70 §3's recipe (c39's talk arm dropped) | f38 ⊕ f41 ⊕ d120 |
| hats draw layer | `party_hat.rs` draw half + `entities.rs`/`tiles.rs` seams + Extra_72 | c39 |
| hat colour hash + death gore table | `npc_hit_effect_a.rs` (UsesPartyHat resolved life-agnostic by the caller) | d55 |
| cake right-click + SugarRush arm | `StationBuff` route + the single `192 =>` arm | f34 (per 70) |
| monolith/break-records | wholesale | f41 (per 70 §1g/1h) |
| GetChat ladder + live feeds | the re-pinned body | d120 (per 70 §1j) |

## 3. Witness union (all WRITTEN NOT RUN — every lane is code-only)

KEEP: f40's trigger-table + EoC-integration + 3 lantern-sky + luck + shop
(2); c39's invasion-clear tail (RE-DERIVE call shape), luck-live, 4
wind/rain, persistence-gate, anniversary-roll, prune + RemoveAt-stride (2),
3 balloon, hat hash/columns/draw, sugar-rush full-tick, lantern fade-kill;
d17's 2 machine witnesses (+ c39's downward-Y bounce assert folded or
re-derived); d55's 7 gore witnesses; everything 70 §4 already keeps
(f38/d120/f34/f41), with 70's two d120 RE-DERIVEs and the f41 anchor
re-point unchanged.

DROP: c39's lantern-sky ladder/opacity witnesses (f40 supersets), c39's
frame-latch witness (d17 #1 superset), c39's death-gore-rows witness (fn
dropped), c39's cake-dispatch witness (route dropped), c39's lantern/
dove bounce witnesses IF their asserts fold into d17 #2 (else re-derive
onto `dove_bounce(last)`).

RE-DERIVE (mechanical): f41's `check_morning_party` witnesses gain the
`tenth_anniversary: false` arg; f40's WoF/first-clear comments fix the
case-4 name; any witness naming `uses_party_hat` on a corpse re-pins to
the life-agnostic resolution.

## 4. Anchor survival + the c39 gap

**c39 added ZERO `anchors.txt` rows** (`git diff a725acb0..d3e63863 --
anchors.txt` is empty) — every c39-unique survivor below is currently
un-canaried. The merge must ADD rows (never delete existing ones):

| Row to add (path\|marker) | Feature |
| --- | --- |
| `crates/tr-app/src/event_skies.rs\|fn update_party` | W4-c39 PartySky (the unique balloon sky: Generate/Reset/Update/42-wrap/1-in-30/opacity ramp) |
| `crates/tr-app/src/party_hat.rs\|fn hat_draw` | W4-c39 the DrawNPCExtras hat layer (columns/offsets/framing groups/anchor/trim) |
| `crates/tr-app/src/world_events.rs\|fn update_time_party` | W4-c39 the roster prune + RemoveAt stride quirk + gone-home edge (keyed misc[99] after the fix) |
| `crates/tr-app/src/world_events.rs\|tenth_anniversary` | W4-c39 the anniversary Next(7) arm |
| `crates/tr-core/src/wind.rs\|pub lantern_night` | W4-c39 the lantern wind freeze + rain gates |
| `crates/tr-app/src/projectile.rs\|fn ai_dove` | W4-d17/c39 the surviving aiStyle-166 machine (d17 added no row either) |
| `crates/tr-app/src/app/frame/draw.rs\|RELEASE_LANTERN` | W4-c39 the lantern entity-light feed |

Survivors from the siblings (all verified present at their tips): f40
`fn on_game_event_cleared` ✓ (helper kept), `struct LanternSky` ✓ (module
kept), `shop.rs|LanternNight,` ✓; d55 `pub fn hit_effect` ✓; f38
`give_town_unique_data` / `check_dialogue` ✓; d120 `mixed_chatter` /
`party_girl` / both `free_cake = false` markers / `select_random_from_dialog`
✓ (the talk-arm marker survives 70's recipe step 3, NOT c39's dropped
arm); f34 `621 => Some((192, 7_200))` ✓; f41 `check_2x2_style` /
`PartyMonolithToggle` ✓ + the re-pointed `crate::npc_chat::check_dialogue`
✓. **No anchor row names any dropped c39 piece** (there are none) — the
drops are anchor-safe by construction.

## 5. Merge hot-spot map (files two+ lanes edit)

`world_events.rs` (c39+f40+f41), `world_sim/npcs.rs` (c39+f40+f38+d120+f41
— the death-match, the talk seam, the shop feed), `world_sim/player.rs`
(c39+f40 luck write — keep one), `world_sim/weather.rs` (c39+f40 — take
c39's persistence gate + BOTH skies' edges on f40's module),
`world_sim/clock.rs` (c39+d120 — one dusk clear, f38 broadcast order),
`player.rs` (c39+f34+f41 — one `192 =>` arm), `projectile.rs`/`data.rs`
(c39+d17 — d17 wholesale + c39's light feed lives in `draw.rs`),
`assets.rs` (c39+f40 — dedupe Extra_134; keep c39's Extra_69-72),
`app/mod.rs`/`flow.rs`/`frame/world.rs` (c39's `event_skies` fields vs
f40's `lantern_sky` fields — the merged App carries f40's LanternSky +
c39's party half, ONE cadence stepper), `edit.rs`/`doors.rs` (c39+f34+f41
— f34's route only), `npc.rs` tests (f40 shop witness).

## 6. Verification

Code-only per the lane contract: **no cargo/rustc/tests/graphify
invoked**. Evidence: C# windows re-read directly —
`NPC.cs:78755-78804`/`:79205-79244`/`:79280-79349`/`:79478-79497`/
`:66605-66664`/`:81650-81734` + the 22-hit `SetEventFlagCleared` census;
`Main.cs:64330-64359`; `Projectile.cs:47540-47611`/`:18330-18365`/
`:16990-17079` (the AI→movement→!active order);
`LanternSky.cs:61-253`; `PartySky.cs:69-184`;
`BirthdayParty.cs:60-144`/`:183-222`; `Legacy.json` misc[99]. Port claims
read from `git diff a725acb0..<tip>` / `git show <tip>:<file>` for all
seven lanes (tips in the header table; f40 confirmed still at `4e2deb3b`,
no f40b). No working-tree code was modified by the reconcile pass itself.

---

## 7. EXECUTED — the c39-side landing record (this branch)

The c39-side deviations below are EXECUTED on `task/W4-c39-lantern-birthday`
(code-only, **WRITTEN NOT RUN** — no cargo/rustc/tests/rustfmt/graphify
invoked; every witness is written, none executed). One commit per verdict.
The sibling-side KEEP pieces are NOT landed here — they survive at the
merge per §2 (f40's `render/lantern_sky.rs` module + shop row + world-load
reset, d55's `npc_hit_effect_a.rs` gore table, f34's StationBuff route +
single `192 =>` arm, d120's GetChat re-pin + superset ctx, f38's naming
layer + broadcast order). Where a verdict says "f40/d17/d55/f34 canonical",
the c39 duplicate is retired/fixed HERE and the canonical sibling form is
re-derived in-place so this branch is self-consistent; the merge folds the
sibling's own module wholesale.

| # | Verdict (§) | Commit | What landed on c39 |
| --- | --- | --- | --- |
| 1 | triggers (§0.1/§0.2/§1a) | `b5ab90db` | the id-less `on_game_event_cleared_first_time(ev, was_down)` REPLACED by f40's id-table `on_game_event_cleared(ev, game_event_id, first)` with the FIXED case-4 label (Mourning Wood 325, NOT Pumpking; Pumpking 327 = id 5, the default arm — ARMS). All 20 call sites re-pointed at the C# ids (invasions 0/1/2/3 `Main.cs:64336-64351`; bosses 6/7/25/10/9/14/11/13/8/16/18/17/12/15/19; WoF keeps the outside-guard `was_down` capture + unconditional call). The five MISSING moon-boss death arms ADDED (Pumpking/MourningWood/Everscream/IceQueen/SantaNK1, `NPC.cs:79210-79238`, pumpkin/snow-moon gated) + their `World` flags + the `.wld` flags-block write/read + the port-save **v18** tail (5 bytes, the v17 fishron shape; 6 back-compat witnesses mechanically re-derived + a new round-trip witness). The gone-home literal → keyed `lang::misc(99)` in BOTH `check_night_party` and `update_time_party` (§0.4). Witnesses: the EoC-integration half DROPPED (f40's superset survives the union), the invasion tail RE-DERIVED (`w4_c39_the_invasion_clear_arms_the_next_lantern_night`) + new id-table (`w4_c39_the_event_clear_id_table_skips_only_the_three_moon_bosses`) and Pumpking-arms/MourningWood-skips (`w4_c39_pumpking_arms_and_mourning_wood_skips`) rigs. |
| 2 | LanternSky colour + PartySky (§1b/§1e) | `4d184569` | the lantern body tint's double-depth-tier rgb apply (tier²·120/255) DROPPED for f40's canonical `LanternSky.cs:224` byte-truncation ladder × `_opacity` ONCE (`lantern_body_tint`/`lantern_glow_tint` helpers + a colour-fix witness); the glow → premultiplied-source `[a,a,a,0]` under AlphaBlend (the tile-flame convention; was white-with-alpha under Additive, which zeroed it). The two superseded lantern ladder/opacity witnesses DROPPED (f40 supersets). PartySky KEPT wholesale + the §1e follow-up-1 retrofit: `draw_balloons` → `draw_balloons_band` (the `PartySky.cs:185-198` num/num2 index walk, unscaled screenPosition + integer-halved centre + zoom-scaled `push_screen` + `>=` cull edges), threaded through `draw_sky`'s depth boundaries + `AmbiencePass`/`draw_front_trees` (the f40 `lantern_sky` shape) — the near balloons now draw in front of the near trees, not one flush batch; `balloon_band_indices` helper + a strata witness. The lantern STATE fns stay interim (f40's module is the surviving LanternSky at the merge). |
| 3 | hats (§0.3/§1g) | `912f0ae2` | the baked `hp > 0` gate REMOVED from `uses_party_hat` (the C# property has NO life gate) + `n.hp > 0` added at the DRAW caller (`entities.rs`, the living UpdateNPC roster). `party_death_gores` + the `update_npcs` death-site gore block + the death-gore-rows witness DROPPED (d55's `npc_hit_effect_a.rs::party_hat_block` canonical: caller-gated `town_npc && hp <= 0 && uses_party_hat`, the strike-stream contract, the full normal/shimmer table — c39's block never fired on the dead clones and rode the wrong UpdateNpcs stream). New life-agnostic predicate witness. The hat DRAW layer + colour hash stay c39-unique. |
| 4 | cake-side (§1h, f41c `f1f95be4`) | `a493e8f1` | the third copies DROPPED: `TileInteraction::CakeEat` + the kind-621 dispatch arm + cake-dispatch witness (`edit.rs`); the CakeEat app arm + `interaction_events` arm (`doors.rs`) + the TR_TOGGLE log arm (`app/mod.rs`); the `192 =>` SugarRush arm (`player.rs` — byte-identical third copy, two arms cannot compile); the FreeCake talk arm (`talk_to_npc` — d120's collapse shape with a worse always-world-drop gift). The `party_is_up`/`lanterns_up`/`downed_moonlord` live-feed hunks revert to base defaults (d120's superset ctx, research/70 drop 8). The full-tick sugar-rush witness KEPT in the union (tests the surviving f34 arm); the `free_cake` field + its writers stay. |
| 5 | AI_166 (§1c, d17 canonical) | `bb19d79f` | the duplicate `projectile_ai166.rs` module RETIRED onto d17's in-file `ai_dove`/`dove_movement`/`dove_bounce` in `projectile.rs`: the consumed `Next(3)` blocked-move cut roll (`:17948`), the unconditional movement + direction sign write (`:17015` precedes the `:17027` `!active` return), the velocity-delta bounce gates, the inline alpha-kill (c39's `kill` `pub(crate)` bump `d3e63863` REVERTED). The `proj_frames` 851/855 => 4 rows ADDED (`Main.cs:8657`/`:8660`). c39's live entity-light feed KEPT as the graft beside d17's machine (`draw.rs`, the `:47568-47569` citation). Witnesses: d17's 2 re-derived in-file + c39's fade-kill and downward-Y bounce folded (re-pointed at `dove_bounce(last)`); the frame-latch witness dropped (d17 #1 superset). |
| 6 | anchors (§4) | `7aff2857` | the 7 enumerated rows ADDED to `anchors.txt` (c39 added ZERO at reconcile): `event_skies.rs\|fn update_party`, `party_hat.rs\|fn hat_draw`, `world_events.rs\|fn update_time_party`, `world_events.rs\|tenth_anniversary`, `wind.rs\|pub lantern_night`, `projectile.rs\|fn ai_dove`, `app/frame/draw.rs\|RELEASE_LANTERN`. All 7 markers verified present; no existing row deleted; no row names a dropped c39 piece. |
| 7 | luck (§1d, f40 placement) | `ce288dac` | the duplicate lantern-night luck write (the before-`update_with_grapple` placement) RETIRED onto f40's canonical beside-auras/zones snapshot (ONE write, `Player.cs:29146-29149`). The underground-isolation luck-live witness stays unchanged. |

**Witness union after the exec** (all WRITTEN NOT RUN): the c39 survivors
are the invasion-clear tail, the id-table + Pumpking/MourningWood rigs, the
lantern colour-fix + balloon band-strata + 3 balloon witnesses, the
life-agnostic hat predicate + hash/columns/draw, the sugar-rush full-tick,
the lantern fade-kill + downward-Y bounce (folded into d17's module), the
luck-live, the 4 wind/rain, the persistence-gate, the anniversary-roll, the
prune + RemoveAt-stride (2), the v18 moon-flag round-trip. The DROPPED c39
witnesses (lantern ladder/opacity, frame-latch, death-gore-rows,
cake-dispatch, the EoC-integration half) are superseded by the sibling
supersets per §3. The sibling witnesses (f40's trigger-table/EoC/3
lantern-sky/luck/shop, d17's 2 machine, d55's 7 gore, research/70's
f38/d120/f34/f41 set) land with their modules at the merge.

**Remaining merge scope** (NOT executed here — the sibling KEEP pieces):
f40's `render/lantern_sky.rs` module wholesale (the lantern STATE fns in
`event_skies.rs` are interim until it folds) + the world-seed RNG graft +
the shop wish row + the world-load reset; d55's `hit_effect` strike-path
wire (its lane's declared follow-up); f34's StationBuff dispatch + app arm
+ the single `192 =>` arm; d120's GetChat re-pin + superset ctx + the
FreeCake gift chain (research/70 §3's recipe); f38's naming layer +
broadcast-before-achievement order. The PartySky §1e follow-up-2 (the
world-load reset seam extended to the party sky) rides f40's reset at the
merge.

---

## 8. The TRWLD v18 UNION — c39's moon-boss block, byte layout in C# order (the 9-way contract)

c39's exec (`b5ab90db`, §7 row 1) bumped `save.rs` `VERSION` 17→**18** with
its OWN 5-byte tail: the moon-event boss first-kill flags, written in the
vanilla's own `SaveWorldFlags` order —
`downedHalloweenKing` / `downedHalloweenTree` / `downedChristmasIceQueen` /
`downedChristmasSantank` / `downedChristmasTree`
(`WorldFile.cs:1347-1351`, re-read; the writer `save.rs:437-446`, the reader
arm `:829-837`). **IceQueen precedes Santank precedes Tree** — the C# row
order, NOT the `NPC.cs:79210-79238` death-arm order (the death arms run
King/Tree/**Tree(21)/IceQueen/Santank**; the save block does not). The
`.wld` flags half carries the same five at the same positions
(`wld/sections/flags.rs:147-151` write, `:506-510` read).

g71's research/92 §1 (`origin/task/W4-g71-bg-layers-2`, tip `f08e8d41e`)
already contracts a **five-branch** union: g70 forest (16 B) → g71 surface
(3 B) → g69 crimson (1 B) → d125 empress (1 B) → d109 dense variants LAST
(`21 + width*height` total). The full v18 claimant set is **nine**:

| # | Lane | Tip | Block | Size | C# citation |
| --- | --- | --- | --- | --- | --- |
| 1 | g70 | `75265591` | `tree_x[0..3]` 3×i32 + `forest_bg_styles[0..4]` 4×u8 | 16 B | `WorldFile.cs:1253-1259`/`:1318`/`:1376-1379` |
| 2 | g71 | `f08e8d41` | `snow/desert/ocean_bg_style` 3×u8 | 3 B | `:1321`/`:1324-1325` |
| 3 | g69 | `20d0380f` | `crimson` 1×u8 bool | 1 B | `:1281` |
| 4 | g72 | `4a75715e` | `cave_back_x[0..3]` 3×i32 + `cave_back_style[0..4]` 4×**i32** + `ice/jungle/hell_back_style` 3×u8 | **31 B** | `:1260-1269` (the styles are i32 in the C# and in g72's `World` — NOT u8) |
| 5 | **c39** | `b5ab90db` | the five moon-boss bools, C# row order above | **5 B** | `:1347-1351` |
| 6 | g87 | `dfbcaf94` | `bought_cat/dog/bunny` 3×u8 bool | 3 B | `:1392-1394` |
| 7 | g90 | `94d33f2c` | `combat_book_volume_two_was_used` + `peddlers_satchel_was_used` 2×u8 bool | 2 B | `:1407-1408` |
| 8 | c43≡f45 | `07697080` / `6352d384` | world identity: `generator_version` u64 + `unique_id` 16×u8 + `world_id` i32 + `creation_time` u64 + `last_played` u64 (both branches carry the IDENTICAL block — keep ONE) | **44 B** | `:1225-1227` + `:1250-1251` |
| 9 | d125 | `d8047a10` | `downed_empress_of_light` 1×u8 bool | 1 B | `NPC.cs:6393`, case-636 `:79351-79353` |
| — | d109 | `81894095` | dense `World::variants`, `width*height`×u8, port-only | W×H B | `Tile.cs:464-482`; never in the C# `.wld` (`WorldFile.cs:1432-1604`) |

### The 9-way fixed head (106 bytes) + dense LAST

Order rule: **C# `.wld` field order** for the eight C#-backed blocks (the
g71 §1 "least-surprising" rule extended — identity rides FIRST because
`SaveWorldHeader` writes `:1225-1227` before `SaveWorldFlags` even starts,
and `CreationTime`/`LastPlayed` at `:1250-1251` precede every flags row;
c43/f45's own block already keeps the two groups in that order), then
**d109's dense variants LAST** (variable-length, port-only — the fixed head
stays at a known offset, g71 §1's rule):

```
… v17 downedFishron byte (1)
v18 UNION tail — fixed head (106 bytes) + dense variants (W×H bytes):
  offsets relative to the start of the v18 block:
  [0..8]    generator_version   u64 LE   ┐ c43/f45 identity (44 B)
  [8..24]   unique_id           16×u8    │  (WorldFile.cs:1225-1227)
  [24..28]  world_id            i32 LE   │
  [28..36]  creation_time       u64 LE   │  (:1250, DateTime.ToBinary)
  [36..44]  last_played         u64 LE   ┘  (:1251)
  [44..56]  tree_x[0..3]        3×i32 LE ┐ g70 forest (16 B, :1253-1259)
  [56..60]  forest_bg_styles    4×u8     ┘  (:1318/:1376-1379)
  [60..72]  cave_back_x[0..3]   3×i32 LE ┐ g72 cave (31 B, :1260-1269)
  [72..88]  cave_back_style     4×i32 LE │  (i32, NOT u8)
  [88]      ice_back_style      u8       │
  [89]      jungle_back_style   u8       │
  [90]      hell_back_style     u8       ┘
  [91]      crimson             u8 bool    g69 (:1281)
  [92]      snow_bg_style       u8       ┐ g71 surface (3 B, :1321/:1324-1325)
  [93]      desert_bg_style     u8       │
  [94]      ocean_bg_style      u8       ┘
  [95]      downed_halloween_king        u8 bool ┐ c39 moon quintet (5 B,
  [96]      downed_halloween_tree        u8 bool │  :1347-1351, the C# ROW
  [97]      downed_christmas_ice_queen   u8 bool │  order — IceQueen before
  [98]      downed_christmas_santank     u8 bool │  Santank before Tree)
  [99]      downed_christmas_tree        u8 bool ┘
  [100]     downed_empress_of_light      u8 bool   d125 (NPC.cs:6393)
  [101]     bought_cat            u8 bool ┐ g87 licenses (3 B, :1392-1394)
  [102]     bought_dog            u8 bool │
  [103]     bought_bunny          u8 bool ┘
  [104]     combat_book_volume_two_was_used u8 bool ┐ g90 (2 B, :1407-1408)
  [105]     peddlers_satchel_was_used     u8 bool ┘
  [106..106+W*H]  variants        W*H × u8   d109 dense, LAST
```

**Fixed head = 106 bytes: 44 + 16 + 31 + 1 + 3 + 5 + 1 + 3 + 2.**
**Total tail = 106 + width*height bytes.**

Note the one departure from strict C# order INSIDE the style bytes: g71's
snow/desert/ocean trio sits after crimson (`:1281`) although the C# writes
setBG3/6/7 at `:1321-1325` — g71 §1 already fixed the trio as a unit after
the g70 block; the 9-way keeps g70→g71 adjacency (the two bg-style groups
stay contiguous, the crimson byte rides with them as the group's tail) and
places g72's cave block at its C# position between forest and crimson.
The moon quintet + empress + licenses + boosters follow in strict row order.

**Writer** (one `if`-free tail after the v17 fishron write; d109's dense
block is the LAST thing `write_world` emits):
```rust
// v18 UNION fixed head, C# field order: identity, forest, cave, crimson,
// surface, moon quintet, empress, licenses, boosters.
w.write_all(&world.generator_version.to_le_bytes())?;   // [0..8]
w.write_all(&world.unique_id)?;                          // [8..24]
w.write_all(&world.world_id.to_le_bytes())?;             // [24..28]
w.write_all(&world.creation_time.to_le_bytes())?;        // [28..36]
w.write_all(&world.last_played.to_le_bytes())?;          // [36..44]
for x in world.tree_x { w.write_all(&x.to_le_bytes())?; }        // [44..56]
w.write_all(&world.forest_bg_styles)?;                           // [56..60]
for i in 0..3 { w.write_all(&world.cave_back_x[i].to_le_bytes())?; } // [60..72]
for &s in &world.cave_back_style { w.write_all(&s.to_le_bytes())?; } // [72..88] i32!
w.write_all(&[world.ice_back_style, world.jungle_back_style, world.hell_back_style])?;
w.write_all(&[u8::from(world.crimson)])?;                // [91]
w.write_all(&[world.snow_bg_style, world.desert_bg_style, world.ocean_bg_style])?;
w.write_all(&[                                            // [95..100] c39
    u8::from(world.downed_halloween_king),
    u8::from(world.downed_halloween_tree),
    u8::from(world.downed_christmas_ice_queen),
    u8::from(world.downed_christmas_santank),
    u8::from(world.downed_christmas_tree),
])?;
w.write_all(&[u8::from(world.downed_empress_of_light)])?; // [100]
w.write_all(&[u8::from(world.bought_cat), u8::from(world.bought_dog),
              u8::from(world.bought_bunny)])?;             // [101..104]
w.write_all(&[u8::from(world.combat_book_volume_two_was_used),
              u8::from(world.peddlers_satchel_was_used)])?; // [104..106]
// … then, at the very end of write_world (d109):
w.write_all(&world.variants)?;                            // dense [106..]
```

**Reader** (`if version >= 18`, the exact inverse; d109 sizes the dense
block from `world.tiles.len()` and sets `variants_from_save = true`).

### Back-compat + merge arithmetic

- **Pre-v18** (version ≤ 17): the whole tail unread; every field keeps its
  `World::new`/Reset default (each lane's own pre-v18 arm already does this;
  c39's five flags stay down — the `:1549-1556` witness shape).
- **Any single-lane v18 file** mis-parses under the union loader. No such
  file ships: all nine branches are unmerged task work off wave-3
  `a725acb0` (`origin/main` is still `VERSION = 17`); the union is the
  FIRST v18 on main — forward-only, nothing to grandfather.
- **Truncation constants** (g71 §1.5's table, extended): every fixed-head
  fixture moves to **106**; every end-relative fixture peels
  `v18_tail_len(buf)` which returns **`106 + width*height`**. c39's own six
  mechanically re-derived fixtures (`save.rs:1197`/`:1309`/`:1338`/
  `:1527`/`:1552`/`:1574`, the `+5`/`-5` shapes) become `+106`/`-106`
  (+ dense where end-relative); c43/f45's identity fixtures, g87/g90's
  1/2/3-byte peels and g72's 31-byte peel fold into the same 106.
- **c43≡f45 keep-one:** both branches' identity blocks are byte-identical
  (same five fields, same order, same citations — read off both tips); the
  merger takes ONE writer/reader, not two stacked blocks. The c43 .plr
  envelope half (`07697080`) is orthogonal (PlayerFile, not save.rs).
- **VERSION stays 18** — one bump, nine payloads; the gate pins 18
  (`dw_1_4_version_above_4_rejected`'s shape, c39 `save.rs:1108-1118`).

This section is the c39 input to g71's research/92 union contract — the
merger of the nine lanes takes THIS head layout (or g71's §1 extended with
the four rows it has not seen: g72, c39, g87, g90, c43/f45 — whichever
note lands last must reconcile to one 106-byte head; the ORDER here is the
C#-field-order rule both notes already use, so the extension is
mechanical).

---

## 9. Event-flag CENSUS — every C# `SetEventFlagCleared` site vs the port (COMPLETE)

Re-read from the decompiled tree (recursive `SetEventFlagCleared(` sweep:
ONLY `Main.cs` and `NPC.cs` call it — no GameContent/IO/other caller).
**26 C# call sites, covering every id 0–25 exactly once** (the Twins'
shared case 125/126 is ONE site). The helper itself
(`NPC.cs:78759-78766`) fires `OnGameEventClearedForTheFirstTime` on the
false→true edge only — the port's `first` parameter is that edge.

| id | C# site | Flag | Port death-arm (c39 tip) | Verdict |
| --- | --- | --- | --- | --- |
| 0 | `Main.cs:64336` | downedGoblins | `npcs.rs:2458` (invasion type 1, the `_` arm) | WIRED |
| 1 | `Main.cs:64341` | downedFrost (**frost legion**) | `npcs.rs:2437` (invasion type 2) | WIRED |
| 2 | `Main.cs:64346` | downedPirates | `npcs.rs:2444` (type 3) | WIRED |
| 3 | `Main.cs:64351` | downedMartians | `npcs.rs:2451` (type 4) | WIRED |
| 4 | `NPC.cs:79219` | downedHalloweenTree (**Mourning Wood 325**) | `npcs.rs:5137`, pumpkin-moon gated | WIRED — the SKIP row (§0.1's fixed label; NOT Pumpking) |
| 5 | `NPC.cs:79213` | downedHalloweenKing (**Pumpking 327**) | `npcs.rs:5129`, pumpkin-moon gated | WIRED — the DEFAULT arm, ARMS (the pumpkin-moon distinction the brief asks about: 325→4 skip, 327→5 arm, both live) |
| 6 | `NPC.cs:79346` | downedGolemBoss | `npcs.rs:5168` (Golem body) | WIRED |
| 7 | `NPC.cs:79349` | downedFishron | `npcs.rs:5178` | WIRED |
| 8 | `NPC.cs:79387` | downedQueenBee | `npcs.rs:5414` | WIRED |
| 9 | `NPC.cs:79394` | downedAncientCultist | `npcs.rs:5253` | WIRED |
| 10 | `NPC.cs:79398` | downedMoonlord | `npcs.rs:5225` (core, `checkDead`-only gate) | WIRED |
| 11 | `NPC.cs:79416` | downedSlimeKing | `npcs.rs:5384` | WIRED |
| 12 | `NPC.cs:79433` | downedPlantBoss | `npcs.rs:5458` | WIRED |
| 13 | `NPC.cs:79441` | downedBoss1 (EoC) | `npcs.rs:5404` | WIRED |
| 14 | `NPC.cs:79457` | downedBoss2 (EoW/Brain, cases 13/14/15/266) | `npcs.rs:5288` (EaterOfWorldsHead + BrainOfCthulhu shared arm) | WIRED |
| 15 | `NPC.cs:79463` | downedBoss3 (Skeletron) | `npcs.rs:5467` | WIRED |
| 16 | `NPC.cs:79476` | downedMechBoss1 (Prime 134) | `npcs.rs:5420` | WIRED |
| 17 | `NPC.cs:79426` | downedMechBoss2 (Twins 125/126, ONE C# site) | `npcs.rs:5439` + `:5451` (TWO port arms — one per eye, each gated on the sibling dead; the C# `this.boss` gate folds: only the boss eyes reach the arm) | WIRED (the one site→two-arm split) |
| 18 | `NPC.cs:79469` | downedMechBoss3 (Destroyer 127) | `npcs.rs:5426` | WIRED |
| 19 | `NPC.cs:79490` | hardMode (WoF 113, local `eventFlag`) | `npcs.rs:5486` (outside-guard capture + unconditional call) | WIRED |
| 20 | `NPC.cs:79231` | downedChristmasIceQueen (345) | `npcs.rs:5153`, snow-moon gated | WIRED — default arm, ARMS |
| 21 | `NPC.cs:79225` | downedChristmasTree (Everscream 344) | `npcs.rs:5145`, snow-moon gated | WIRED — SKIP |
| 22 | `NPC.cs:79237` | downedChristmasSantank (346) | `npcs.rs:5161`, snow-moon gated | WIRED — SKIP |
| 23 | `NPC.cs:79352` | downedEmpressOfLight (636) | **NONE on c39** — the kind is UNPORTED here | UNWIRED-BY-KIND: **d125** (`d8047a10`) ports `NpcKind::EmpressOfLight` + the death arm, but its arm writes `app.world.downed_empress_of_light = true` DIRECTLY — no `SetEventFlagCleared` edge, no lantern arm. **Merge obligation (cross-receipt to d125):** route its arm through `on_game_event_cleared(ev, 23, !was_down)` (id 23 is a DEFAULT row — the Empress's first kill ARMS the lantern night). c39 cannot wire it: no kind to match on this branch. |
| 24 | `NPC.cs:79358` | downedQueenSlime (657) | **NONE anywhere** — the kind is unported on EVERY sibling (no `NpcKind::QueenSlime` in any wave-4 branch read) | UNWIRED-BY-KIND: the `World.downed_queen_slime` flag exists (v15 `.trwld` byte + `.wld` `:1396` read, shop/scene consumers live) but only load/save writes it. When the Queen Slime kind lands, its death arm must call id 24 (default row — ARMS; also a dual-dungeon Hallow-tier wall clear, below). Nothing to wire today. |
| 25 | `NPC.cs:79355` | downedDeerclops (668) | `npcs.rs:5184` | WIRED |

**Count reconciliation:** 26 C# sites − Empress (23) − QueenSlime (24) =
24 wired ids; + the Twins' second port arm = **25 port call sites** (the
exec's "20 re-pointed + 5 added" — the 20 included the Twins pair). The
id-table witness (`w4_c39_the_event_clear_id_table_skips_only_the_three_
moon_bosses`, `world_events.rs:686`) already sweeps ALL 26 ids: 23 arm rows
(incl. 23/24 — the table is kind-agnostic) + the 3 skips + the repeat-edge
arm. **No id in 0–25 is missing from the table; the only unwired DEATH
ARMS are the two unported kinds.**

**Solar eclipse:** NOT an event-flag id — the C# has no `downedEclipse`
and no `SetEventFlagCleared` call for it (the eclipse is a rolled weather
event, `Main.cs:65062`'s `StartEclipse`, cleared by time, not by a kill).
Nothing to wire; the brief's question resolves NO-OP.

**`NPC.downedMechBossAny`** (`:79427/:79470/:79477` set it beside the mech
flags): the port derives it (`World::downed_mech_boss_any()`,
`world.rs:1070` — the OR of the three flags). Observationally identical:
the C# static latches true on the first mech death and never resets
within a world; the OR is true exactly when any flag is. Verified
equivalent — no separate field needed.

**`OnGameEventClearedForTheFirstTime`'s NON-lantern halves** (the helper
ports the lantern half only — `world_events.rs:372`; the remainders, all
recorded, none c39-scoped):
- case 10 `CreditsRollEvent.TryStartingCreditsRoll()` (`NPC.cs:78779`) —
  unported (the `npcs.rs:5224` comment records it; the net module
  `net.rs:1500` carries the remaining-time message shape only).
- cases 16/17/18 `WorldGen.GeneratePlanteraBulbOnAllMechsDefeated()`
  (`:78784-78787`, gated `netMode != 1 && hardMode && all three`) —
  unported (the helper doc records it; the bulb TILE exists,
  `wiring.rs:4886`, the generation pass does not).
- the `Main.dualDungeonsSeed` wall-clear switch (`:78794-78822`,
  `WorldGen.ClearUnbreakableWallsWithPaintUpTo` tiers EarlyGame/ EvilBoss/
  JungleBoss/Dungeon/Hallow for ids 11/13, 14, 8, 15/19, 16/17/18/24) —
  unported (the seed flag exists, `worldgen/pipeline.rs:3720`'s gate; the
  wall-clear fn does not). A worldgen lane's scope, not the lantern
  lane's; recorded here so the census is COMPLETE.

**Census verdict: no event-id is silently unwired.** Every ported kind's
death arm rides the id table; the two gaps are kind-gaps (d125's Empress
arm needs the helper route at merge — the cross-receipt above; the Queen
Slime waits on its kind), and the eclipse is a non-event. WRITTEN NOT RUN
(no new code this phase — the census is a doc verdict over the exec's
already-landed arms).

---

## 10. Wind/rain gates — COMPOSE-VERIFY FINAL against d130-landed (`560a02ea`)

d130's weather lane LANDED (research/83 FINAL, tip `560a02ea`): the f39
UpdateTime fold (surface gate, slime-clock-first, branch-entry sampling,
the `:64890` three-flag gate) + its OWN `Wind::lanterns_up` freeze
(`50c845e5`'s weather half). c39's `f5ae8fc0` gates re-verified against
that landed state, window by window (≤100-line reads, both tips):

### 10a. The wind freeze — ONE gate, two names, identical truth table

| Aspect | C# (`Main.cs:58830-58860`) | c39 (`wind.rs`, this branch) | d130 landed (`wind.rs:202-213`) | Verdict |
| --- | --- | --- | --- | --- |
| drift before the gate | `:58832-58849` ease-toward-target runs FIRST | `update_rain:184-198` before the returns | `:185-200` before the returns | IDENTICAL |
| creative/client/menu gate | `:58850-58854` | `if self.frozen { return; }` | `if self.frozen { return; }` (`:205`) | IDENTICAL |
| lantern gate | `:58856-58859`, NESTED inside `!FreezeWindDirectionAndStrength` | was `frozen \|\| lantern_night` one return — **now split into two sequential returns** (this phase) | `if self.lanterns_up { return; }` (`:211`), own return | IDENTICAL behaviour; the split makes the merge a mechanical rename |
| what the gate skips | `windCounter--` + the walk + the ±0.8 clamp (`:58860-58950`) | counter/walk/clamp after the returns | same | IDENTICAL |

The C# nest and two sequential returns are the same truth table: either
gate alone skips the walk; both together skip it once; the drift ran
either way. **Executed this phase:** the `||` split + a new compose
witness `w4_c39_the_creative_and_lantern_freezes_compose_like_the_csharp_
nest` (creative+lantern drifts bit-identically to either alone, counter
untouched, target unwalked). The existing
`w4_c39_the_lantern_night_freezes_the_wind_walk` stays (the drift-survives
arm). **Merge: keep ONE flag** — `lantern_night` (c39, the §1f/anchors
name) or `lanterns_up` (d130) — f40 research/105 §4's mechanical-rename
verdict; the feed line (`weather.rs:171` c39 / `:165` d130) keeps ONE
write. c39 samples inside the tick closure (per-iteration), d130 at the
tick head (per-frame): both read BEFORE the lantern walk can move the
flag; at one tick per frame they are the same read; under backlog
catch-up c39's is fresher (the C# reads live) — the merge may keep
either, the gate values only flip at UpdateTime/boundaries.

### 10b. The rain gates — signature re-pinned to d130's landed order

d130's landed `Rain::update_time` takes `(…, player_luck, slime_rain,
lanterns_up, next_night_lanterns)` — the f39 shape, slime FIRST. c39's
`f5ae8fc0` had `(…, lanterns_up, next_night_lantern, slime_rain)`.
**Executed this phase:** c39's signature + the `weather.rs` call + the
three gate-specific witness call sites re-pinned to d130's order (the
all-false call sites are order-agnostic, untouched); the gate LOGIC was
already identical (the `:64868-64871` StopRain arm — c39's early return
vs d130's if/else: the arm exits the ladder either way — and the
`:64890` three-flag start-arm block, each gate draw-free). The two
lantern witnesses (`w4_c39_the_lantern_night_stops_rain_and_blocks_the_
start_arms`, `w4_c39_the_lantern_head_clamps_the_storm_deck_before_the_
drain`) keep every assertion; d130's twins
(`the_lanterns_up_arm_stops_the_rain_without_draining_or_rolling`,
`the_64890_gate_flags_close_the_start_arms_before_any_roll`) are the same
rigs — the union keeps ONE pair (either; the assertions match).

### 10c. Cross-receipts to d130 (merge obligations, NOT c39 code)

1. **The storm-deck clamp is MISSING from d130's landed body.** d130's
   `update_time` (`:678-693`) goes forced-check → ladybug drain — no
   `:64825-64831` head. c39's head clamp (`lanterns_up → cloud_bg = 0`,
   before the drain, draw-free; the numClouds cap outcome-neutral at the
   100 stand-in) is UNIQUE (f40 research/105 §3 KEEP c39). **The merge
   must graft c39's head into d130's body** — dropping it loses the
   lantern-night storm-deck clear.
2. **The persistence gate:** d130's landed tick keeps the base
   `lantern_can_persist = !world.day_time` (`weather.rs:286`); c39's full
   `!day_time && lanterns_can_start(&LanternGates::from_world(...))`
   (`weather.rs:236-239`, the `LanternNight.cs:42-48` gate) is the KEEP
   per f40 §3 — the merged tick takes c39's gate line.
3. **The slime-clock/entry-sample restructure is d130's KEEP** (its f39
   fold: clock BEFORE the rain section under the `:64846` surface gate,
   `was_raining`/flag entry samples, the roll inside the else-if arm).
   c39's tick still carries the pre-fold clock position (after the event
   walks, ungated) — the merge takes d130's body for that region; c39's
   gate ARGS feed it unchanged (the re-pin above makes the call
   compile-compatible). c39 does NOT duplicate the restructure here.
4. **`StartSlimeRain`'s `:64744` gate** (remix refusal + exclusive
   `<= 50.0` surface) is d130's landed fix; c39's `start_slime_rain`
   still has the pre-fold `<= 0.0` check — d130's body wins at merge,
   no c39 action (out of the lantern-gate scope).

**Compose verdict: the c39 gates compose cleanly with d130-landed.** The
wind freeze is one gate under two names (split + compose witness landed
here); the rain signature is re-pinned to the landed order; the two
KEEP-c39 pieces d130's body lacks (the head clamp, the full persistence
gate) are cross-receipted for the merge. All witnesses WRITTEN NOT RUN
(code-only).

---

## 11. MERGE-SCOPE FINAL CHECKLIST (the complete c39-side hand-off)

The definitive list of what the merger must do with this branch. Items
1–5 are §7's remaining scope (unchanged); 6–10 are the final-phase
additions (§8/§9/§10). Every item names its canonical winner and its
evidence — nothing here is a guess.

| # | Item | Canonical / action | Evidence |
| --- | --- | --- | --- |
| 1 | **LanternSky module** | f40's `render/lantern_sky.rs` wholesale (c39's `event_skies.rs` lantern STATE fns are interim; the colour fix `4d184569` already matches f40's `:224` ladder). The world-seed RNG graft + the shop wish row ride f40. | §1b/§2; f40/105 §4 (the wind-drift witness `522d92bd` reads `speed_current` live — agnostic to which wind flag name survives) |
| 2 | **World-load reset order** | f40's clear-then-restore sites (`5a19073e`: the clear in `swap_world`/`client_install_world`, NOT `enter_loaded_world` — the tail clear would clobber f45's restored lantern quad). The PartySky reset (§1e follow-up-2) extends the SAME seam. c39's `world_clear` (`world_events.rs:398`) already leaves `next_night_is_lantern_night` alone — matches `LanternNight.WorldClear` (`:119-125`) and f40/105 §5's survival note. | f40/105 §5; f45 `31996439` (the quad: `SetTempToOngoing` `WorldFile.cs:1060-1063` → tail 1940..1946 → `SetOngoingToTemps` `:173-176`; the `WorldEvents` field names match c39's) |
| 3 | **Gore table wire** | d55's `npc_hit_effect_a.rs::party_hat_block` + its `hit_effect` strike-path wire (its lane's declared follow-up). c39's death-site block is DROPPED (`912f0ae2`); the hat DRAW layer + colour hash stay c39. | §1g/§7 row 3 |
| 4 | **Cake/buff route** | f34's StationBuff dispatch + app arm + the single `192 =>` SugarRush arm; f41's pitched case-2 cue (`b4ca66e9`). c39's third copies are DROPPED (`a493e8f1`). | §1h/§7 row 4; f41/70 §8a |
| 5 | **Party chatter chain** | d120's GetChat re-pin + superset ctx + the FreeCake gift chain (70 §3's ONE recipe); f38's naming layer + broadcast-before-achievement order + the keyed dawn/dusk rows. c39's talk arm DROPPED; the `free_cake` field + writers stay; the `lang::misc(99)` fix is landed in BOTH c39 sites (`b5ab90db`). | §1f/§7; f41/70 §1i/§3/§8e |
| 6 | **TRWLD v18 union** | §8's 9-way contract: ONE 106-byte fixed head in C# field order + d109 dense LAST; c43≡f45 identity keep-ONE; every truncation constant → 106 (+W×H end-relative); VERSION stays 18. c39's quintet rides at [95..100] in `WorldFile.cs:1347-1351` row order. | §8 (this note) + g71/92 §1 (five-branch base, `f08e8d41e`) |
| 7 | **Empress death arm** | d125's `NpcKind::EmpressOfLight` arm must route through `on_game_event_cleared(ev, 23, !was_down)` — its landed arm writes the flag directly (no lantern edge). Id 23 is a default row: the first kill ARMS. | §9 census row 23 (d125 tip `d8047a10` read) |
| 8 | **QueenSlime** | NO action — the kind is unported on every W4 branch; when it lands, its death arm calls id 24 (default row, ARMS). The v15 flag byte + consumers already live. | §9 census row 24 (branch-wide grep) |
| 9 | **d130 weather compose** | Keep ONE wind flag (mechanical rename, §10a — the split returns + compose witness landed here `9ac39893`); the rain signature is ALREADY re-pinned to d130's order (§10b). **Graft c39's storm-deck head clamp + the full `LanternsCanPersist` gate into d130's body** (its landed tick lacks both). d130's slime-clock restructure + `StartSlimeRain` fix are ITS keep. | §10; d130/83 FINAL (`560a02ea`); f40/105 §3 KEEP-c39 rows |
| 10 | **Anchors** | the 7 c39 rows (`7aff2857`) all still resolve on this tip — incl. `wind.rs\|pub lantern_night` (the field survives the §10a split; only the return shape changed). No row names a dropped piece. | §4/§7 row 6 |

### Cross-receipts — f40 research/105 (`a06b16e4b`) and f41 research/70 (`f21028f56`)

**105 (read in full, final phase):** its §3 gate table's four KEEP-c39
rows (persistence gate, wind freeze, rain stop/start gates, storm-deck
clamp) are confirmed against this tip — rows 1/3/4 unchanged, row 2's
flag now split-return (§10a). Its §4 wind-drift verification (the freeze
does NOT stop the drift; the sky reads `speed_current` live) matches
c39's `update_rain` structure exactly. Its §5 clear-then-restore order +
the `NextNightIsLanternNight` WorldClear survival match c39's
`world_clear` body (item 2 above). Its case-4 label fix (`8b4f87e9`)
matches c39's §0.1 — both sides now agree Mourning Wood 325 → 4 skip,
Pumpking 327 → 5 arm. Its invasion-clear wires (`a5547cfc`) and c39's
(`b5ab90db`) call the SAME helper signature with the SAME ids — the
merge keeps one set of call sites, not two. **Receipt: no open conflict
between c39 and f40/105.**

**70 (read §7/§8, final phase):** its §8f claims "c39's tip carries NO
docs/research/114" — STALE: 114 landed at `8ecb288a` (the exec record)
and this final phase extends it (§8–§11). The §8f keep-one verdicts
themselves are CONFIRMED EXECUTED on c39: the CakeEat variant/dispatch/
app arm/TR_TOGGLE arm dropped, the duplicate 192 arm dropped, the
collapsed FreeCake talk arm dropped (`a493e8f1` = §7 row 4); the
KEEP-ones it names (anniversary roll, `can_npc_party`, roster prune,
`party_hat.rs`, `event_skies.rs`, the lantern lanes) all survive on this
tip. Its §8e census (every natural-start arm claimed: c39's
anniversary/prune/`can_npc_party`, f38's broadcasts) matches §1f here.
**Receipt: 70 §8f's stale-existence claim is superseded by this note;
its verdicts and c39's exec agree item-for-item.**

### Final witness union delta (this phase)

ADDED: `w4_c39_the_creative_and_lantern_freezes_compose_like_the_csharp_
nest` (wind.rs). RE-PINNED (assertions unchanged): the two lantern rain
witnesses' call sites (d130 arg order). Everything else per §7's union
record. **All WRITTEN NOT RUN** — code-only lane contract; no
cargo/rustc/tests/rustfmt/graphify invoked in any phase.

### Final-phase commits (this branch)

| Hash | Phase |
| --- | --- |
| `5cdf6da6` | 1 — §8 the v18 9-way union input (moon-boss byte layout, C# order) |
| `96b13e3b` | 2 — §9 the event-flag census (26 sites, ids 0–25, two kind-gaps) |
| `9ac39893` | 3 — §10 wind/rain compose-verify (split returns + compose witness + arg re-pin) |
| `4f72847e` | 4 — §11 the FINAL checklist + 105/70 cross-receipts |

---

## 12. RETIRE — the 106-byte-union BROADCAST: f44/g71 FINAL read, CORRECTION-input

Both persist FINALs re-read off their pushed tips at retire (this phase):

- **f44 FINAL** = `bdd1cc79a` (`origin/task/W4-f44-save-plr-2`),
  research/132 §3-L3: the "reconciled FINAL **97-byte** head" —
  g70 forest `[0..16]` → g72 cave `[16..47]` → g71 surface `[47..50]` →
  g69 crimson `[50]` → d125 empress `[51]` → g90 books `[52..54]` →
  c43 identity `[54..97]` → d109 dense LAST.
- **g71 FINAL** = `f08e8d41e` (`origin/task/W4-g71-bg-layers-2`),
  research/**92-doc-sweep-bg.md** §1 (the union lives THERE — the
  same-tip `92-roadmap-reverification.md` is a different, ROADMAP doc):
  the **21-byte** five-branch head — g70 forest `[0..16]` → g71 surface
  `[16..19]` → g69 crimson `[19]` → d125 empress `[20]` → d109 dense LAST.

**NEITHER aligns with §8's 106-byte head. This is the CORRECTION-input**
(their-scope-missing blocks, not a layout dispute):

### 12a. The missing-quintet delta (both FINALs)

| Block | §8 seat | f44 97-head | g71 21-head |
| --- | --- | --- | --- |
| **c39 moon-boss quintet, 5 B** (`WorldFile.cs:1347-1351`, C# ROW order — IceQueen before Santank before Tree) | `[95..100]` | **ABSENT** | **ABSENT** |
| **g87 licenses, 3 B** (`:1392-1394`, tip `dfbcaf94`) | `[101..104]` | **ABSENT** | **ABSENT** |
| g72 cave, 31 B | `[60..91]` | present `[16..47]` | absent |
| g90 boosters, 2 B | `[104..106]` | present `[52..54]` | absent |
| c43/f45 identity, 44 B | `[0..44]` FIRST | present LAST `[54..97]` | absent |
| g70/g71/g69/d125 (16+3+1+1) | `[44..60]`/`[91..95]`/`[100]` | present | present |
| d109 dense LAST | `[106..]` | LAST | LAST |

Arithmetic: **21 + 85 = 106** (g71's head + cave 31 + quintet 5 +
licenses 3 + boosters 2 + identity 44); **97 + 9 = 106** only after
f44's own off-by-one is fixed (below). f44's §3-L3 claimant table lists
eight claimants (g70/g71/g69/d125/g90/g72/c43/d109) — **c39 and g87 are
not in its set at all**; g71's §1 seats five. d125 IS seated in both —
its empress byte is not part of the correction.

### 12b. f44's internal off-by-one (independent of the missing blocks)

f44's stated sum "16+31+3+1+1+2+44" = **98**, not 97 — and its own
offset table contradicts it: the c43 identity block is 44 B but seated
`[54..97]`, a **43-byte span**. Either the books end at `[54]` and
identity runs `[54..98]` (head 98), or one byte of identity is silently
dropped. The 97 total, the `v18_tail_len = 97 + W*H` return and every
"→ 97" truncation move in its §3-L3/§5 inherit the error. **The
correction stands regardless: neither 97 nor 98 carries the quintet or
the licenses.**

### 12c. The order divergence (identity FIRST vs LAST)

f44 seats identity LAST ("identity fields are not bg-family, so they
close the fixed head after the books"); §8 seats it FIRST because
`SaveWorldHeader` writes generator version/GUID/worldId at
`WorldFile.cs:1225-1227` and CreationTime/LastPlayed at `:1250-1251`
BEFORE `SaveWorldFlags` starts — the C# `.wld` field order both notes'
own order-rules invoke. §8 also keeps g70→g71 adjacency with crimson as
the group tail (g71 §1's unit rule), while f44 puts surface before
crimson. The merger must pick ONE order; **§8's is the C#-field-order
reading** and the one every §8 truncation constant already pins. If the
merger instead keeps f44's order, the quintet/licenses/boosters/empress
still ride in C# row order after the style groups — the correction
below re-seats mechanically either way.

### 12d. The CORRECTION-input (the exact insertions)

Into f44's head (after fixing 12b): insert **c39's quintet (5 B,
`[1347-1351]` row order) immediately after the empress byte**, then
**g87's licenses (3 B)** — the C# rows `:1347-1351` → `:1392-1394` with
empress (`NPC.cs:6393`'s flag, the `.wld` row between) preceding both;
the books (`:1407-1408`) close the flags. Into g71's head: seat g72's
31-B cave block between forest and surface (its own 64-add §2 seat),
g90's 2-B books, c43's 44-B identity, AND the same quintet+licenses.
**Either path reconciles to §8's 106-byte head** (or §8's block set in
f44's order — one head, one order, 106 fixed bytes + W×H dense, VERSION
18, forward-only). The `underworldBG` open seat (f44 §3-L3, C# `:1376`)
is orthogonal: if it lands, +1 byte → 107, re-pin once, both notes.

### 12e. Receipt state

- **f44**: DIVERGES — correction-input published (missing quintet +
  licenses; its own 97/98 arithmetic error; identity-order divergence).
- **g71**: DIVERGES — correction-input published (its five-branch §1
  predates the four later claims: g72/g90/c43-f45/c39/g87 — 85 bytes).
- **§8 stands as the authoritative c39 input** — no §8 byte moves; the
  quintet layout, the 106 head and the +106/-106 constants are unchanged
  on this tip (`4f72847e` lineage). The broadcast IS this section; the
  merger takes §8's head or mechanically re-seats per 12d — never a
  97/98/21-byte head that drops a claimant.

---

## 13. RETIRE — the d130f graft: EXACT 2 hunks (storm-deck clamp + LanternsCanPersist)

d130's retire tip re-read: **`97ed44f5d`**
(`origin/task/W4-d130-weather-ambience`, research/83 s7 adjudication —
the snow-stream verdict; the weather bodies are unchanged from the
`560a02ea` landing §10 verified). Both §10c obligations re-confirmed
MISSING on that tip:

- `crates/tr-app/src/rain.rs` `update_time` (`:696-712`): the
  forced/calm check at `:710-712` falls straight into the
  `// Main.cs:64833-64836.` ladybug drain at `:713` — **no `:64825-64831`
  head clamp**.
- `crates/tr-app/src/world_sim/weather.rs:292`:
  `let lantern_can_persist = !world.day_time;` — **the base gate only**,
  no `LanternsCanStart()` half.

### Hunk 1 — `crates/tr-app/src/rain.rs` (insert after `:712`, before `:713`)

Context: the closing `}` of the `if self.forced.is_some() || self.calm`
early return; the `lanterns_up` parameter already exists in d130's
landed signature (the f39 order §10b re-pinned c39 to) — the hunk
compiles against d130's signature unchanged.

```diff
         if self.forced.is_some() || self.calm {
             return;
         }
+        // W4-c39 (graft): `Main.cs:64825-64831` — `UpdateTime`'s head, BEFORE
+        // the ladybug drain: while the lanterns are up `cloudBGActive = 0f`
+        // and `numClouds` caps at 30. The port's `numClouds` stand-in is
+        // pinned at 100 (`NUM_CLOUDS_STAND_IN`) and both `> 150`/`> 100`
+        // band arms are dead at 100 AND at 30, so the cap changes no
+        // outcome (recorded); the storm-deck clear is observable.
+        if lanterns_up {
+            self.cloud_bg = 0.0;
+        }
         // `Main.cs:64833-64836`.
         if self.lady_bug_rain_boost > 0 {
             self.lady_bug_rain_boost -= day_rate;
```

Witness rides: `w4_c39_the_lantern_head_clamps_the_storm_deck_before_
the_drain` (c39 `rain.rs:1878` — deck 0.8 → 0 under lanterns, untouched
0.8 without; the boost still drains). d130 has no twin of this witness.

### Hunk 2 — `crates/tr-app/src/world_sim/weather.rs` (replace `:292`)

```diff
         // `LanternNight.UpdateTime`'s persistence gate: the lanterns ride
         // the night (`LanternsCanPersist` = `!dayTime && LanternsCanStart()`).
-        let lantern_can_persist = !world.day_time;
+        let lantern_gates =
+            crate::world_events::LanternGates::from_world(world, &app.npcs);
+        let lantern_can_persist =
+            !world.day_time && crate::world_events::lanterns_can_start(&lantern_gates);
         let _ = crate::world_events::update_time_lanterns(
             &mut app.world_events,
             lantern_can_persist,
```

`LanternGates::from_world(world, &app.npcs)` exists on BOTH tips with
the same shape (d130 `world_events.rs:89`, c39 `:93`'s impl) — the
`world`/`app` bindings are the tick's own, unchanged. **One mechanical
rider:** d130's `lanterns_can_start` (`world_events.rs:115`) is private
`fn`; c39's (`:122`) is `pub(crate)`. The merged `world_events.rs` must
export it — either keep c39's lantern module (its `LanternGates` +
`pub(crate) fn` already do) or bump d130's `:115` visibility one word.
The gate bodies are identical on both tips (the `LanternNight.cs:49-56`
seven-conjunct AND) — no logic choice.

### What the graft does NOT touch (d130's KEEP, §10c rows 3/4)

The slime-clock-first/entry-sample restructure, the `:64846` surface
gate position, the if/else `StopRain` arm shape and `StartSlimeRain`'s
`:64744` fix all stay d130's landed body. The wind rename
(`lantern_night` ↔ `lanterns_up`) stays f40/105 §4's mechanical verdict.
Exactly two hunks; WRITTEN NOT RUN (code-only — no cargo invoked).

---

## 14. RETIRE — QueenSlime-24 re-probe: STILL UNPORTED, task-seed d43r

The §9 row-24 claim re-probed at retire against the named lanes:

- **d43 band 631-697** (`origin/task/W4-d43-npc-kinds-631-697`): tip
  `a725acb04` — **the wave-3 base itself, 0 commits ahead of
  `origin/main`** (`git rev-list --count origin/main..` = 0). The band
  lane is claimed but has landed NOTHING: `NpcKind::QueenSlime` grep
  over its `crates/` tree returns zero hits (the `queen_slime` hits are
  the pre-existing pet/hook/mount/loot/town-chatter consumers of the
  `downed_queen_slime` FLAG, not the kind). **657 is NOT rowed.**
- **d33 styles 95-127** (`origin/task/W4-d33-npc-ai-styles-95-127`, tip
  `b08acf463`, 25 commits ahead): `npc_ai_style121.rs` EXISTS — the full
  `AI_121_QueenSlime` machine (`NPC.cs:45202-45812`), `style121_step`
  (`:354`) + `Style121Env`/`Style121Outcome` (shots 922/926, minions
  658/659/660, sounds, light, hide) + `queen_find_frame` (`:1007`). Its
  own module doc states it: "The port's roster has no queen-slime kind
  — the module lands UNWIRED (the `npc_ai_style73.rs` precedent)." No
  `NpcKind::QueenSlime` in d33's `npc.rs` either.

**Verdict: no event-wire hunk can publish — there is no kind to match
on any branch.** The §9 row-24 census stands (the id-table witness
already sweeps id 24; only the death arm waits). Task-seed:

### Task-seed d43r — row the QueenSlime 657 kind + wire the death arm

Scope (the band lane's first row, or a follow-on `d43r` branch off it):

1. **Row the kind** `NpcKind::QueenSlime` (vanilla id 657) through every
   exhaustive `NpcKind` match — the d33 dragonfly precedent
   (`ba8b9b5c`/`e71010d3`): `to_u16`/`from_u16` keep 657,
   `cs_id`/`net_id`/`render_key` ride it; `SetDefaults` row
   `NPC.cs:17310-17325` verbatim (114×100, aiStyle 121, damage 60,
   defense 26, lifeMax 18000, NPCHit1/NPCDeath64, lavaImmune,
   knockBackResist 0, value 60000, boss, npcSlots 5); `ALL_KINDS` +1,
   the `s1_f_3` gate +1; bestiary/bestiary-sheet rows per `NPCID.cs`
   (the boss keeps its own card).
2. **Wire the AI**: the `ai_style() == 121` axis arm + the kind-match
   route feed d33's `style121_step` (merge-order: d33's module lands
   first or rides the same merge); the `Style121Outcome` drain (shots →
   `Projectile.NewProjectile` 922/926, minions → `NPC.NewNPC`
   658/659/660 at `:45801`, sounds, the constant light, hide) — the
   env's `no_tile_collide`/`no_gravity` round-trip per the module doc.
   The minion kinds 658-660 are their own rows (the band's scope).
3. **The death arm — the c39 event-wire hunk, ready to apply the moment
   the kind exists** (`world_sim/npcs.rs`, the `DoDeathEvents` match,
   beside the d125 Empress arm; `NPC.cs:79357-79359` case 657 →
   `SetEventFlagCleared(ref downedQueenSlime, 24)`):

   ```rust
   // `DoDeathEvents`'s case 657 (`NPC.cs:79357-79359`):
   // `SetEventFlagCleared(ref downedQueenSlime, 24)`. Id 24 is a
   // DEFAULT row — the first kill ARMS the lantern night (the
   // census's row 24; the dual-dungeon Hallow-tier wall clear is
   // the worldgen lane's remainder, research/114 §9).
   NpcKind::QueenSlime => {
       let was_down = app.world.downed_queen_slime;
       app.world.downed_queen_slime = true;
       crate::world_events::on_game_event_cleared(&mut app.world_events, 24, !was_down);
   }
   ```

   The flag byte already round-trips (v15 `.trwld` + `.wld` `:1396`);
   `on_game_event_cleared`'s id-24 default arm already arms
   (`world_events.rs:372-382` — only 4/21/22 skip); the id-table
   witness already covers 24. **Zero helper-side work remains — the
   seed is purely the kind + its arm.**
4. Witness: extend the death-arm rig (first kill arms
   `next_night_is_lantern_night`, repeat kill does not) — the Golem/
   Fishron precedent shape (`npcs.rs:5165-5179`).

Nothing lands on c39 (no kind here either); the seed is recorded for
the band lane. WRITTEN NOT RUN (code-only).

---

## 15. RETIRE — the Empress-23-route receipt for d125b (flag-direct-write → helper-route)

d125's tip re-read at retire: **`d8047a109`**
(`origin/task/W4-d125-empress-r7-drain`, phase-5 docs FINAL). The §9
row-23 finding CONFIRMED unchanged on that tip:

- `world_sim/npcs.rs:5174-5176` — the case-636 arm:

  ```rust
  NpcKind::EmpressOfLight => {
      app.world.downed_empress_of_light = true;
  }
  ```

  A DIRECT flag write — no `was_down` capture, no
  `SetEventFlagCleared` edge, no lantern arm. The C#
  (`NPC.cs:79351-79353`) calls `SetEventFlagCleared(ref
  downedEmpressOfLight, 23)` — the false→true edge fires
  `OnGameEventClearedForTheFirstTime(23)`, a DEFAULT row: the first
  Empress kill ARMS the lantern night. d125's arm drops that edge.
- d125's `world_events.rs` has NO `on_game_event_cleared` (grep: only
  the `next_night_is_lantern_night` field + the check-night consumer) —
  the helper is c39's (`world_events.rs:372`); it lands with c39's
  module at merge. **The route hunk below compiles only after the c39
  helper is in the merged tree — merge order: c39's world_events first
  (or same merge), then this hunk.**

### The route hunk (apply on the merged tree, `world_sim/npcs.rs:5174`)

```diff
             // W4-d125 — `DoDeathEvents`'s case 636 (`NPC.cs:79351-79353`):
             // `SetEventFlagCleared(ref downedEmpressOfLight, 23)`. The
             // progression notification rides the flag (the fishron
             // precedent); the .wld flags block owns the byte.
             NpcKind::EmpressOfLight => {
+                let was_down = app.world.downed_empress_of_light;
                 app.world.downed_empress_of_light = true;
+                crate::world_events::on_game_event_cleared(&mut app.world_events, 23, !was_down);
             }
```

The exact shape of every re-pointed arm on c39 (the Golem/Fishron/
Deerclops precedent, `npcs.rs:5165-5184`): capture → write → route with
`!was_down` as the first-kill edge. Id 23 is NOT in the helper's
4/21/22 skip list — the default arm sets
`next_night_is_lantern_night = true` on the edge only; repeat kills
route with `first = false` and no-op (`world_events.rs:373-375`).

### Witness receipt (d125's existing rig survives)

d125's `the_lacewing_release_chain`-family witness asserts
`app.world.downed_empress_of_light` after the boss death
(`npcs.rs:18686`) — the routed arm keeps that assertion TRUE (the write
is unchanged). The merge ADDS two assertions to the same rig (the
c39 death-arm witness shape): after the first kill
`app.world_events.next_night_is_lantern_night` is TRUE; a second kill
(with the flag already up) leaves a freshly-cleared
`next_night_is_lantern_night` FALSE (the repeat edge no-ops). The
id-table witness (`world_events.rs:686`) already sweeps id 23 as an
ARM row — no table change.

**Receipt state: d125b's obligation is DISCHARGED-AS-HUNK** — the exact
route is published here; d125's own branch needs no further commit (the
hunk applies at merge against c39's helper). The v18 union side is
already aligned: d125's empress byte is seated in BOTH f44's and g71's
heads and at §8 `[100]` — the §12 correction does not touch it.
WRITTEN NOT RUN (code-only).
