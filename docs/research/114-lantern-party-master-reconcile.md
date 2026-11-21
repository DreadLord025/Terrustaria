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
