# 121 — the shimmer doc-sweep (W4-f47 lane 36, DOC-SWEEP phase) — FINAL

> **SEALED (research/151, 2026-11-21)**: the shimmer-family cross-lane
> seal (`151-shimmer-family-seal.md`) composes this lane's work with the
> co-lanes — the e19 lights mutual-receipt, the 676 HitEffect census
> (g62 hit / g60 death), the b1 activation-contract, the d6
> ShimmerImmunity compose, the d43 676-row keep-one. The splash keeper
> is definitive in `379-ARBITER-FINAL-v2.md` (renumbered
> 144→379 per registry §10). The family bible
> (single-owner map + merge checklist) is research/151 §7.

> Coded 2026-11-21 on `task/W4-f47-liquid-shimmer-2` (base `507a0da5`,
> the DONE tip), the DOC-SWEEP lane 36 continuation. C# cites are
> `decompiled/Terraria/*` (stable); Rust cites are this branch's tip.
> Code-only mode: no cargo/tests/rustfmt/graphify invoked; every witness
> is WRITTEN NOT RUN, verified by re-reading the cited C# blocks, hand
> arithmetic and mechanical PowerShell diffs of the data tables.
> Co-lane claims read from the pushed tips: W4-b3 `96a5d88f`, W4-g2
> `8ac7d95e`, W4-e19 `f7561091`, W4-c7 `c2cc6ec6`, W4-c9 `c2316284`,
> W4-f43 `79a037f5`, W4-b1 `f010b85b`.
>
> **FINAL (the keep-one round, 2026-11-21)**: the e19 co-lane pushed its
> own splash port (`7d3cb5e3` + the round-3 fixes, tip `939e45dc8`) —
> the 2-way keep-one verdict + fold-hunks + witness dedupe live in
> `docs/research/277-item-splash-keep-one.md` (renumbered 144→277 per
> registry §6.3). The folded wins on this
> branch: the C# 16 drop box (`WorldItem.cs:138`, every probe + the
> witness re-pins, the 14 px DRAW visual kept centred), the `box_is_wet`
> restoration (the doc-sweep 36 deletion was a compile break —
> `app/frame/draw.rs:419` still called it), the wiring.rs literal field
> fixes (a second compile break), the e19 pass-level splash quartet +
> the flush landing/rest pair + the surface-offset witness folded as
> `w4_f47k1_*`/`w4_f47ds_*`. The f47 `Splash` latch shape KEPT (the
> verbatim LOCAL entry-lava gate, the once-per-pass water style); the
> e19 `ItemSplash` queue is the merge dedupe loser. The 676 light/dust
> arm (`NPC.cs:61128-61150`) WIRED (the censused skip retired — §4
> below updated). The §1/§2 records stand as the lane's original
> derivation; the 14→16 geometry supersedes every 14-box number in
> them (the probe is 10x8 at +3/+4, the splash roll `Next(24)`).

## The six deviations, resolved

### 1. The splash dust/sound arms — PORTED (the cited skip retired)

The f47 phase-3 note cited-skipped `WorldItem.MoveInWorld`'s splash arms
(`WorldItem.cs:879-1035`) because their `Next(6)` colour draws would shift
the pinned `UpdateItems` stream witnesses. The census found the arms
faithful-critical (they are the ONLY liquid-entry feedback an item drop
has, and the shimmer arm's style-4/5 cues are the aether's signature
sound), so they are ported, not skipped:

- **Entry** (`:879-957`): on `flag3 && !base.wet && wetCount == 0` the
  counter arms 20 and ONE of four bodies runs — the `!flag2` gate reads
  the LOCAL LavaCollision result:
  - shimmer (`:888-913`): 10x dust **308** at
    `(pos.X-6, pos.Y + height/2 - 8)` box `(width+12, 24)`, each
    `vy -= 4`, `vx *= 2.5`, `scale = 0.8` (SET, not *=), `noGravity`,
    then ONE `Next(6)` colour (0 → (255,255,210), 1 → (190,245,255),
    2 → (255,150,255), default → (190,175,255)); `PlaySound(19, x, y, 4)`.
    NO alpha write (the C# leaves NewDust's 0).
  - honey (`:915-926`): 5x dust **152**, `vy -= 1`, `vx *= 2.5`,
    `scale = 1.3`, `alpha = 100`, `noGravity`; style-1 cue.
  - water (`:928-939`): 10x `Dust.dustWater()` (`Dust.cs:402-418`, the
    `Main.waterStyle` switch), `vy -= 4`, `vx *= 2.5`, `scale *= 0.8`
    (MULTIPLY), `alpha = 100`, `noGravity`; style-1 cue.
  - lava (`:942-953`): 5x dust **35**, `vy -= 1.5`, `vx *= 2.5`,
    `scale = 1.3`, `alpha = 100`, `noGravity`; style-1 cue.
- **Exit** (`:959-1034`): the same four bodies, but the gates read the
  STICKY Entity flags (`:965` base.lavaWet), the shimmer cue is style
  **5** (`:992`), and the water arm's anchor DROPS the −8 Y fix
  (`:1011` — `pos.Y + height/2`, no −8).
- The per-dust `NewDust` internals (the position pair, the velocity pair,
  the frame `Next(3)`, the scale `Next(-20,21)`, the pressure bands) ride
  the landed `DustPool::new_dust` port (`tr-core/particles.rs`, S1-E-1);
  the type-308 (`vel *= 0.5, vy += 1`), type-33-family (`alpha 170,
  vel *= 0.5, vy += 1`) and type-35/152 (`vel *= 0.1, vy = -0.5`) tweak
  chains run inside it BEFORE the arm's modifiers — the lava dust's
  final vy is −0.5 + −1.5 = −2.0, the shimmer dust's is
  (roll×0.5 + 1) − 4.
- The 3-arg `PlaySound(19, x, y)` overload's default Style is **1**
  (`SoundEngine.cs:102`); the case-19 pitch rolls ride the landed
  `Audio::play_splash_sound` (styles 4/5: ×0.75 + `Next(-20,1)`;
  style 1: `Next(-10,11)`, `sound.rs::splash_case19`).

**Port shape**: `ItemDrop::splash` (`item.rs`) + the `Splash` latch
(`splash_due: Option<Splash>`). `move_in_world` arms the latch at the
C# position, SNAPSHOTTING the arm-time gates + position — the exit
snapshot keeps the sticky flags past the `:1036-1041` dry clear that
runs before the drain. The items pass (`world_sim/items.rs`) drains it
right after `drop.update`, BEFORE the `UpdateItem_VisualEffects`
flicker draws (`:436` runs after `:413`'s MoveInWorld) — same tick, same
slot order, all draws on `UpdateItems` (`Main.cs:18024`'s swap). The
water style feeds from the live biome scan
(`biome::calculate_water_style`, the `TR_WATER_STYLE` pin capped at 14
like the draw pass).

**Stream re-pin notes (stale-by-design)**: the pinned `UpdateItems`
witnesses that count draws now see the splash draws too. The
`shimmer_tests`/`rand_light_witness` fixtures rest their drops IN liquid
(the shimmer pool / the dry world's floor rows are liquid-free — the
dry-world pins stay exact); the shimmer-pool fixtures gain the entry
splash's 71 draws (10×6 NewDust + 10 colours + 1 pitch) on tick 1 and
the transform-chain draws after. The DW-4.1 pair witness is
replay-equality (both runs shift identically — still green by
construction). The `the_flicker_arms_draw_the_item_stream_in_slot_order`
and `a_world_without_flicker_kinds_stays_draw_free` pins use the DRY
world (no liquid cells) — unchanged. The
`the_revenge_share_draws_next_50_76_plus_the_coin_extras` fixture uses
`test_world_helpers::ground()` (dry) — unchanged. Any future fixture
that rests a drop in liquid must mirror the splash draws in its twin
(the 71/31-draw censuses are pinned in the new item.rs witnesses).

### 2. The wet probe — FIXED (the full-box stand-in retired)

The C# probes are two DIFFERENT boxes:

- `Collision.LavaCollision` (`Collision.cs:1653-1684`, called at
  `WorldItem.cs:865`): the FULL 14x14 box against the surface-dropped
  lava cells (`(256-liquid)/32*2` px drop, `:1673-1675`).
- `Collision.WetCollision` (`Collision.cs:1572-1651`, called at `:870`):
  the CENTRE-INSET box — `min(10, width) × min(height/2, height)`
  (INTEGER halves) at `(pos + size/2 - inset/2)` = **10x7 at
  (pos.x+2, pos.y+4)** for the 14x14 drop; the same surface drop; the
  FIRST overlapping cell wins (early return) and sets the
  `Collision.honey`/`Collision.shimmer` statics from THAT cell; a
  liquid-less SLOPED tile under liquid counts via the tile above
  (`:1628-1646`, honey ELSE-IF shimmer there).

Both were already ported for other callers — the fix is a swap, not a
new port: `ItemDrop::wet_probe` delegates to
`npc_ai_slime::wet_collision` (the W4-f47 phase-2 NPC port, verbatim
`:1572-1651`), `ItemDrop::lava_probe` to `player::lava_collision`
(the teleport-scan port, verbatim `:1653-1684`). The old
`box_in_liquid` cell-grid stand-in (no inset, no surface drop, no
first-cell-wins) is deleted.

**The sticky-flag fidelity fix**: the C# Entity flags are STICKY —
`:866-877` only ever SETS them, the `:1036-1041` dry clear resets them
when `!wet`. The old port re-probed `shimmer_wet` with the full box at
the update HEAD (before `UpdateShimmer`), which (a) used the wrong box
and (b) gave `UpdateShimmer` a FRESH probe instead of the C#'s
previous-tick flag. The head re-probe is retired: `UpdateShimmer`
(`:403`) now reads the flag `MoveInWorld` (`:875-877`) set LAST tick,
exactly the C# order. Consequence: a fresh drop's first tick only
probes; the ramp starts tick 2 — the fresh-drop transform crossing
re-pins **91 → 92** (`a_drop_in_shimmer_transforms_on_the_ninetieth_tick`,
the ramp arithmetic itself unchanged: the f32 undershoot at the 90th
sum puts the >0.9 crossing on ramp-tick 91).

### 3. The town state-25 UNWIRED — WIRED (the seam), b3 contract documented

The b3 co-lane (`task/W4-b3-npc-ai-town-wire`, pushed `96a5d88f`) owns
the FULL aiStyle-7 migration: the axis claim
(`NPC.cs:20812-20815`), the `update_guide` retirement, the door latch,
the NO-OP `TownDoorOps`, the generic `UpdateNPC` tail
(`integrate_town_collision`). Its `ai_007_town_family` adapter calls
`town_npc_ai`, which reaches the `:53187` state-25 arm — but its
`TownEnv` folds `shimmering: false` and its `TownOutputs` drain
(including `shimmer_fx`) is phase 2's.

This branch does NOT duplicate that migration (the hot-file conflict
would be semantic). Instead it wires the EMERGENCE SEAM only:

- `npc_ai_town::town_shimmer_step` (new): for a living town-like NPC at
  `ai[0] == 25` — the state `drain_shimmer_transforms`' TownVariant arm
  sets (`NPC.cs:91578-91591`) — builds the solo classic `TownEnv` (the
  b3 adapter's fold, but `shimmering` reads the LIVE buff-353 flag),
  runs `town_shimmer_state25` on the caller's `UpdateNpcs` stream, and
  drains `ShimmerFx`: the hue light onto `EntityLights` (the
  `(int)(v/16)` truncation, the `npc_lights.rs` convention), the 309
  dust via `NewDustPerfect`, the particle requests recorded (the
  orchestrator unported). Returns the progression-43 flag.
- `world_sim/npcs.rs`'s slot loop calls it at the NPC's own slot turn
  (the C# AI() position), INSTEAD of `update_rng_dust` for that tick
  (the C# arm RETURNS at `:53277` — the dry walk never runs). The
  preamble runs first (the buff block precedes AI()); the generic slot
  tail (just_hit/sounds/look-for-home) still runs after. The
  progression-43 latch drains post-pass (`notify_progression_net(43)`).
- `timeForVisualEffects` = `anim_time × 60 mod 216000`
  (`Main.cs:17162-17164`; the TR_SHOT 1/60 pin keeps the tick count
  exact).

**Order deviation (documented)**: the C# spawns each dust INLINE
between the light and the particle roll; the port drains the whole
`shimmer_fx` list after the arm, so the dust allocator's own six draws
per `NewDustPerfect` land AFTER the same tick's particle-triple draws.
The in-arm gate/direction draws keep their exact C# order; only the
allocator draws shift within the tick — the pool-pass convention, same
as the item splash drain.

**MERGE CONTRACT (b3)**: when b3 lands, its adapter reaches the same
`:53187` arm through `town_npc_ai`. Keep ONE driver: either fold this
seam into the adapter's `TownOutputs` drain (the adapter's `let _ = out`
becomes a real drain) or drop the adapter's state-25 call in favour of
this pre-pass. The draws are identical either way (the same in-arm
order on the same stream). The adapter's `shimmering: false` fold MUST
become the live buff-353 flag (this seam already reads it). The
`Teleport(pos, 12)` effect/teleportTime latch and the particle
orchestrator stay cited visual seams.

### 4. The 676/677 roster gap — 676 LANDED, 677 cross-noted

The census (`NPCID.cs:12029`/`:12031`, `NPC.cs:17566-17593`):

- **676 ShimmerSlime**: NOT a town pet — a hostile aiStyle-1 slime,
  24x18, dmg 20, def 5, 80 HP, scale 1, value `Item.buyPrice(0,0,1,50)`
  = 150 copper, NPCHit1/NPCDeath1, the class-default kb 1 (the row
  writes none). It is the `ShimmerTransformToNPC` target of **17**
  rows (the mechanical recount; the slime/critter band 1/302/335/336/
  334/333/225/141/16/147/184/537/204/81/183/138/121). The port had NO
  kind — every `-> 676` transform was a no-op (the drain's
  `kind_from_net_id(676)` returned None). **LANDED** this sweep:
  `NpcKind::ShimmerSlime` + the full row (protocol id 676 free), the
  shared `update_slime` stand-in arm (the b1 wiring's aiStyle-1 family
  claims it at merge), the case-676 FindFrame (`NPC.cs:70823-70846` —
  the 8-tick hop strip wrapping `npcFrameCount[676]` = 2), the NPC_676
  sheet request + draw arm + portrait, the null debuff row, the netglue
  surface seed, the bestiary credit. ALL_KINDS 270 → 271. The 676
  light/dust arm (`NPC.cs:61128-61150`) was the censused skip — **WIRED
  by the keep-one round** (research/277, formerly 144): the gate + the 4-NextFloat
  body roll in `ai_001_slimes`, the 306 `NewDustPerfect` via
  `wants_shimmer_slime_dust` → the sim drain, the torch-23 centre light
  via `npc_lights::shimmer_slime_light` (the draw-side feed). The live
  stand-in route draws nothing until b1's adapter lands (the merge
  seam, the activation contract research/151 §3); the `HitEffect` 676
  arm (`:82058-82078`) is the DUST lanes' — g62 ports the LIVE half,
  g60 the death halves (the 676 row a g60 census gap, the 40-count
  death half seeded — research/151 §2; the old "cited-unwired, the
  HitEffect dust pass unported port-wide" note is superseded).
- **677 Shimmerfly**: NOT a town pet either — a 10x10 aiStyle-64
  critter, noGravity/noTileCollide, catchItem 5350, the
  `ShimmerTransformToNPC` target of **82** critter rows. **NOT landed
  here** — the co-lanes already pushed it: W4-c7 `81094ca0` (the four
  aiStyle-64 kinds + rows, `c2cc6ec6` the wiring) and W4-c9 `37baf0d1`
  (the snail family + the 677 target, `c2316284` the wiring incl. the
  shimmer Transform(677) seams). Landing it here would duplicate their
  `NpcKind::Shimmerfly` + bestiary + protocol-id arms — a hard merge
  conflict. Cross-note only.
- The ledger's "town pet band 670/678-684" is the TownSlime family
  (`NPC.cs:17594-17607`, townNPC true) — a DIFFERENT band, still
  unported (the f43 town-spawn census `79a037f5` carries their spawn
  arms UNWIRED). 676/677 are not in it.

### 5. The COIN/ITEM transform table completeness — CENSUS CLEAN

The mechanical diffs (PowerShell regex extraction, pair-by-pair):

| table | C# source | pairs | diff |
| --- | --- | --- | --- |
| `ShimmerTransformToItem` | `ItemID.cs:353` | 316 | **0** |
| `ShimmerTransformToNPC` | `NPCID.cs:10470` | 114 | **0** |
| `ShimmerTransformToItem` (NPC) | `NPCID.cs:10468` | 15 | **0** |
| `ShimmerTownTransform` | `NPCID.cs:10469` | 29 | **0** |
| `ShimmerCountsAsItem` | `ItemID.cs:354` | 6 | 0 (spot) |
| `ShimmerCountsAsItemForDecraft` | `ItemID.cs:355` | 1 | 0 (spot) |
| `ShimmerPostMoonlord` | `ItemID.cs:356` | 6 | 0 (spot) |

Nothing to port. The e19 item-side fixes (`e351dd43`: the shared tail,
the equivalent-type resolve, the coin-luck no-eager-recalculate) are on
`task/W4-e19-liquid-quicksell-shimmer` (pushed `f7561091`), NOT in this
branch's base — they stay e19's claim; the two branches' `shimmer.rs`
halves are disjoint (e19 edits the chain's apply, f47 the NPC drain).

**The COIN side**: `GetShimmered`'s first branch
(`WorldItem.cs:1697-1718`) reads `ItemID.Sets.CommonCoin`
(`ItemID.cs:1318`, {71,72,73,74}) — the coins are NOT
`ShimmerTransformToItem` entries (71-74 carry no set rows, by design).
The tier multipliers (silver ×100, gold ×10000, platinum floor-1
×1000000) live in the chain (`resolve_shimmer`'s CoinMerge arm +
`apply_due_transforms`' platinum floor), already ported
(M7-SHIMMER-1). The `ShimmerTransforms.cs` file itself carries NO
data tables — only the resolution helpers (`GetTransformToItem`'s
set → createTile-139 placeStyle → 3461 moon-phase order, the decraft
recipe-index helpers, the post-Skeletron/Golem locks), all ported.

The new census witness (`tr-core/shimmer.rs::
w4_f47ds_the_transform_table_census_pins_the_family_rows`) pins the
load-bearing rows by name (the ore/gem cycles, the 23-row 5353 torch
family, the post-Moonlord members, the tail pairs, the 17/82 NPC
bands) so a future extraction drift fails loudly.

### 6. The g2 Shimmer() helper co-lane — CROSS-NOTE, no dup

The g2 co-lane (`task/W4-g2-proj-ai-missing-b`, pushed `8ac7d95e`,
research `26d-projectile-rowless-kinds-w4-g2.md`) records the
`Projectile.Shimmer()` family (`Projectile.cs:22361-22516`, called at
`:17009-17012` under `base.shimmerWet`) as a sibling gap:

- The 264 RainNimbus's shimmer bounce (`:28086-28092`) is LIVE on g2
  (the d5 fold-out was stale — the tile carries
  `liquid_kind() == Shimmer`).
- The 229 orb's `Shimmer()` kill/bounce arm (`:22403-22422`, the
  `aiStyle 1/16/40 || type 229` branch) is unported port-wide — g2's
  recorded follow-up.
- The 469/181/566 velocity kick (`:22372-22378`) and the arrow-family
  float (`:22380-22383`) are unported (the ANALYSIS
  `:27774-27776` note).

This sweep does NOT port the helper — it is g2's claimed follow-up
(their doc names it explicitly). The full census for the future port:
the head guard (`:22363` — hostile non-25/17, the 99 yoyo's ai0 −1, the
15 flail's ai0 2/4 all RETURN), the per-type bodies (502: vy −0.4 +
timeLeft −5; 469/181/566: vy −0.2, the 469 kills when vy < 0; the
20-kind arrow list: vy −0.4; the phaseblade family: the ai0/ai1 reset +
the −0.5 rise / ×1.1 X / ±14 clamp; the 1/16/40/229 branch: the
full-shimmer-cell Kill probe + the vy-flip bounce with the 600-cap
−60 timeLeft; the 5/8/21/28/29/33/36/47/93/115/151/152/1024/669 branch:
the same Kill probe + the bare vy-flip; 15: the flip + −0.4 / −8 floor;
99: the flip + the −8 SET; the 7/9/13/35/53/197/130/134/137/138/149/
160/204 styles: Kill; 2: vy −0.7 + the 936 rainbow-slime unlock + the
timeLeft −20 else; 14/16: the >3000 timeLeft −60 + vy −0.5; 25/17: the
>10 ×0.97 + −0.7 / −10 floor). The call site is the wet-block tail
(`:17009`), AFTER the `:16998-17007` dry clear + wetCount decrement —
the same sticky-flag shape this sweep's item port now carries.

## Commits (this round, on `task/W4-f47-liquid-shimmer-2`)

1. `d7eb4e91` — items 1+2: the faithful wet probes + the splash arms
   (the inseparable probe/splash change), the 91→92 crossing re-pin,
   6 witnesses.
2. `6f126bd9` — item 3: the town state-25 emergence seam + the b3
   merge contract, 2 end-to-end witnesses.
3. `e8bacc2f` — item 4: the ShimmerSlime 676 roster row (677
   cross-noted to c7/c9), 1 row witness.
4. `4a9d1912` — item 5: the transform-table completeness census (zero
   missing entries), 1 census witness.
5. `eaa6f112` — item 6's cross-note + the sweep record.
6. `a2561d6d` — the two anchors rows.
7. (the keep-one round) — the e19 2-way verdict (research/277, formerly 144): the 16
   box fold + the `box_is_wet`/wiring compile-break fixes + the 676
   light/dust wiring + the folded witnesses; this doc's FINAL header.

## Verification

Code-only: no cargo/tests/rustfmt/graphify invoked (the lane contract).
The data-table diffs ran mechanically (PowerShell, pair-by-pair, zero
diffs); every ported block was re-read against its cited C# window
(≤100 lines); the witnesses are WRITTEN NOT RUN. The stream-pin
re-derivations are hand-counted (the 71/31-draw splash censuses, the
91→92 probe-lag re-pin) and recorded above for the gate run.