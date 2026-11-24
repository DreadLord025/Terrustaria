# 151 — the shimmer-family SEAL (W4-f47 seal phase, the cross-lane bible)

> Coded 2026-11-21 on `task/W4-f47-liquid-shimmer-2` (base `057f44a68`,
> the ARBITER FINAL v2 tip). Code-only: no cargo/tests/rustfmt/graphify
> invoked; every verdict is a re-read of the cited C# window (≤100
> lines) against the lanes' pushed tips. **Number:** 151 — first-free
> per the all-tips `git ls-tree` sweep (149 taken e20, 150 taken d54,
> 151/152/153 free on every origin tip, 2026-11-21). Renumber at merge
> if the registry (d54's `150-research-number-registry.md`) assigns
> differently.
>
> **Companions (tips read):** e19 `0fb0ac4ad` (the SHIMMER SEAL tip —
> its entry-gate verbatim fix + its ANALYSIS merger bible read), b1
> `f010b85be`, d6 `7edfcd495` (residual 9 = `e2561efe1`), d43 `0253bde90`
> (batch 9 = `418336473`), g59 `d277f782b`, g60 `30fea0fe1`, g62
> `32af98b44`, e5 `a62a32ce` (via g60's bible pin). The f47 lane's own
> record: research/121 (FINAL) + research/277 (the keep-one, formerly
> 144 — renumbered per registry §6.3) +
> research/379-ARBITER-FINAL-v2 (THE definitive splash keeper; v1 kept
> as the lane record, renumbered 144→278 per registry §6.3).
>
> **This doc is the SEAL**: the five cross-lane compositions the brief
> ordered, each with a keep-one verdict + the exact merge contract.
> The splash keep-one is ALREADY definitive (379-ARBITER-FINAL-v2);
> this doc adds the LIGHTS mutual-receipt, the 676 HitEffect census,
> the b1 activation-contract, the d6 ShimmerImmunity compose and the
> d43 keep-one — then the family bible (the single-owner map for every
> shimmer surface, the f47-side complement of e19's ANALYSIS bible).

## 0. The family map (what "shimmer" touches, and who owns each piece)

| surface | C# | owner (merged) | f47's half |
| --- | --- | --- | --- |
| the item splash (entry/exit) | `WorldItem.cs:879-1035` | **f47** (`Splash`/`splash_due`/`ItemDrop::splash`) | the keeper (379-ARBITER-FINAL-v2) |
| the drop box 16 | `WorldItem.cs:138` | **f47** (the e19 fix folded) | `item::SIZE = 16.0` |
| the item transform chain | `WorldItem.cs:1691-1849` | **f47** resolve + **e19** apply | `resolve_shimmer`/`can_shimmer` (f47), the chain-apply `wet` writes (e19) |
| the lunar-brick moon-phase table | `ShimmerTransforms.cs:113-126` | **e19** (the RAW-enum fix) | folded here (§1.5) |
| the NPC transform / ramp / GetShimmered | `NPC.cs:91514-91592` | **f47** | `npc_shimmer_transparency_tick`/`resolve_npc_shimmer`/`drain_shimmer_transforms` |
| the NPC shimmer-wet contact + buff 353 | `NPC.cs:93052-93076` | **f47** | `shimmer_wet_contact_pass` |
| the buff-353 immune set | `NPCID.cs:10467` + `NPC.cs:17838` | **d6** (the set) + **f47** (the flag/ramp readers) | §4 compose |
| the town state-25 emergence | `NPC.cs:53187-53277` | **f47** seam + **b3** driver | `town_shimmer_step` (the ONE-driver contract) |
| the 676 roster row | `NPC.cs:17566-17578` | **f47** (the AI-arm superset) | §5 keep-one vs d43 |
| the 676 per-tick light/dust | `NPC.cs:61128-61150` | **f47** | `ai_001_slimes` arm + `shimmer_slime_light` + the sim drain |
| the 676 HitEffect (hit + death dust) | `NPC.cs:82058-82078` | **g62** (hit) + **g60** (death) | §2 census — NOT f47's |
| the 676 FindFrame | `NPC.cs:70820-70846` | **f47** (the count-read shape) | §5 keep-one vs d43 |
| the aiStyle-1 machine wiring | `NPC.cs:19945-19949` | **b1** | §3 activation-contract |
| the item/NPC lights (AddLight family) | `Lighting.cs:151-163` | **f47** (676) + **e19** (item 16-box anchors) | §1 mutual-receipt |

## 1. The e19 lights-resume compose (the mutual-receipt)

The brief: *e19-lights-resume compose (e19-16-box-fix-folded-by-you;
e19c-active — publish mutual-receipt: e19-box_is_wet-body-kept,
splash-keep-f47; e19-lights/AddLight-scope-disjoint).*

### 1.1 The 16-box fix — FOLDED (both directions acknowledged)

e19's round-3 `e2003946b` fixed the drop box to the C# 16
(`WorldItem.cs:138`). f47 folded it at `f80490bcf` (SIZE 14→16 across
every probe, the 14 px DRAW visual kept centred `pos+1`/`SIZE-2`, every
geometry witness re-derived). **Receipt:** the f47 tip is a canonical
16-pinned side; e19's `research/145-16box-cross-lane-repins.md` f47 row
("SIZE 14 — retire") is stale and struck by 278-ARBITER-FINAL (formerly 144) §"The
16-box repin reconciliation" (v2 upholds the reconciliation; the e19
seal's 145 execution table also marks f47 "already folded"). Both lanes
now read the identical 16-box; the merge dedupes the const + the
re-pins trivially (same values).

### 1.2 `box_is_wet` — the e19 NAME+BODY kept

The f47 doc-sweep 36 deletion of `ItemDrop::box_is_wet` left
`app/frame/draw.rs:419` calling a missing method (a compile break). e19
kept the name, re-pointed at the faithful `player::wet_collision` bool
half. f47 restored it with the e19 body at `f80490bcf`. **Receipt:**
`box_is_wet` is the e19 name+body, kept on f47; the merge keeps ONE
copy (the f47 restoration ≡ the e19 original — same inset maths, the
`player::wet_collision` bool half, SIZE 16 → the 10x8 probe at +3/+4).

### 1.3 The splash keeper — f47 (the ARBITER FINAL v2, definitive)

379-ARBITER-FINAL-v2 adjudicated the re-opened dispute at both NEW tips
(f47 `18fe75f2e` / e19 `0fb0ac4ad`, the SHIMMER SEAL): **KEEPER = the
f47 `Splash`/`splash_due`/`ItemDrop::splash` shape, UPHELD on the
witness-superset ground.** The NEW FACT: the e19 seal landed the entry
gate VERBATIM (its `item.rs:1415` `lava: lava_now` per C# `:886`
`!flag2`, the exit `:1429` sticky per `:965`) — the v1 verbatim-vs-
sticky tiebreak is MOOT (both halves 16-box + body-identical +
gate-verbatim). The keep reduces to WITNESS SUPERSET COVERAGE: f47
carries the latch-lifecycle twins + the 71/31 RNG-stream draw twins +
the pass-level quartet WITH the `max_dust_to_draw` 6000 budget fix the
e19 originals lack; the one e19-unique splash pin (the local-lava-probe
gate witness) folded as `w4_f47k2_the_entry_splash_gate_reads_the_local_lava_probe`.
The exact e19 drop-hunks are re-pinned at `0fb0ac4ad` in v2; the e19
KEEP list (the flags/gravity/repulsion family + the dry-integration
full-velocity SPLIT half) stands. **This seal adds nothing to the
splash verdict — it is closed.** (The e19 ANALYSIS merger bible's
"e19 KEEPS" splash row is its own lane's superseded claim — v2
overrules it; 141/master reads v2.)

### 1.4 The lights / AddLight scope — DISJOINT (the new receipt)

Census of the e19 SHIMMER-SEAL tip's light-related diff (`git diff
a725acb04..0fb0ac4ad` over the light files):

- **`npc_lights.rs`: e19 touches NOTHING** (0-line diff). The whole
  NPC-light feed module — including f47's `shimmer_slime_light` (the
  676 torch-23 centre light, `NPC.cs:61130`) — is f47's alone. e19 has
  no NPC-light claim.
- **`entity_lights.rs` / `item_lights.rs`: e19's ONLY light hunks are
  the 16-box fixture re-pins** (`e2003946b`): the `item_light`/
  `rand_light` test args 14.0→16.0, the anchor comments 167/183→168/184
  and −13→−12. These are the SAME re-pins f47 folded at `f80490bcf`
  (the `entity_lights.rs`/`item_lights.rs` fixture args + the anchor
  comments). **No production light code differs** — the `item_light`/
  `rand_light`/`XAnchor`/`tile_x`/`tile_y` bodies are byte-identical on
  both tips (the 16-box is a const + fixture change, not a light-engine
  change).
- **`app/frame/draw.rs`: e19's hunks are the aether `shimmer_alpha`
  feed + the cursor-override** (`6627efd6b`/`db8212430`) — NOT the NPC
  light feed. f47's `shimmer_slime_light` call (`draw.rs:458`) sits in
  the NPC-light block e19 never touches.

**The AddLight scope-disjoint proof.** The C# `Lighting.AddLight`
funnels every overload into `AddLight(int i, int j, r, g, b)`
(`Lighting.cs:157-163`). The `(Vector2, torchID)` overload
(`:151-155`) truncates BEFORE dividing (`(int)position.X / 16`). Two
distinct consumers:

- **f47's 676 light**: `AddLight(base.Center, 23)` (`NPC.cs:61130`) —
  the torchID overload, `XAnchor::CenterTruncated`, the animated
  `ShimmerTorchLight` colour (`TorchID.cs:131`/`:161`). Lives in
  `npc_lights::shimmer_slime_light`, fed from `draw.rs:458`.
- **e19's item lights**: the `item_light`/`rand_light` family
  (`WorldItem.cs:1366-1541`) — the `createTile == 4` torch arms + the
  flicker/lunar kinds, the `XAnchor` right-edge/centre truncation. Lives
  in `entity_lights.rs`/`item_lights.rs`, fed from the item draw pass.

The two families share the `XAnchor`/`tile_x`/`tile_y` vocabulary (the
`entity_lights.rs` helpers) but **no fn, no feed site, no witness**.
f47's 676 light is an NPC-source; e19's are item-sources. The merge
keeps BOTH feeds (the `entity_lights.add` list is shared, the sources
are disjoint). **DISJOINT — no keep-one needed on the lights; the
16-box fixture re-pins dedupe to one copy (same values).**

### 1.5 The lunar-brick moon-phase fix — e19's win, FOLDED here

e19's round-3 `cab982244` found the ONE gap in the item-transform
census: `lunar_brick_for_moon_phase` read an INVERTED 0=empty/4=full
convention. The C# `Main.GetMoonPhase()` is `(MoonPhase)moonPhase` over
the RAW enum (`Enums/MoonPhase.cs`: Full 0, ThreeQuartersAtLeft 1,
HalfAtLeft 2, QuarterAtLeft 3, Empty 4, QuarterAtRight 5, HalfAtRight 6,
ThreeQuartersAtRight 7; `Main.cs:3199-3202`, the `:46897-46928` info
names confirm Full=0/NewMoon=4). The switch must read the enum indices
(Full 0 → 5408, default Empty 4 → 5406).

**Census of the f47 tip:** `tr-core/src/shimmer.rs:60-71` carried the
INVERTED table (7→5407, 4→5408, …) — the SAME bug e19 found. The f47
`world.moon_phase` is the RAW clock value (`clock.rs:73-77` increments
0..7, `wld/sections/flags.rs:395` reads the raw byte, `TR_MOON` pins it
raw), and `ShimmerContext.moon_phase` threads `app.world.moon_phase`
verbatim (`shimmer.rs:427`). So the f47 lunar-brick arm was WRONG (a
Full-moon 3461 resolved to 5406 instead of 5408). **FOLDED** (this
seal): the switch re-pinned to the RAW enum indices, the witness
corrected — byte-identical to e19's `cab982244` fix. The merge dedupes
to one copy (e19's fix ≡ this fold).

**Scope note:** the OTHER moon-phase readers on the f47 tip
(`spawner.rs:1164`/`:1198`, `fishing.rs:84-93`, `world.rs:1005`,
`npc_chat.rs:823`, `shop.rs:1301`, `herbs.rs:114`) were censused — they
read the RAW convention CORRECTLY already (the spawner's `moonPhase ==
4` eye-roll cites `NPC.cs:4549` verbatim, the fishing 0=full/4=new
multipliers match `Player.cs`, the zoologist `moon_phase == 0`
full-moon lycantrope matches `NPC.cs:93753-93764`). ONLY the
lunar-brick transform table had the inverted stand-in. The fix is
scoped to `lunar_brick_for_moon_phase` + its witness — no other
moon-phase reader changes.

## 2. The 676 ShimmerSlime HitEffect-arm census (cited-unwired → owned)

The brief: *676-ShimmerSlime HitEffect-arm (:82058-82078 cited-unwired
— census C# scope, port small or task-seed g59-death-lane;
d43-also-ported-676-ShimmerSlime-kind → keep-one your-AI-arm vs
d43-row).*

### 2.1 The C# arm (verbatim, `NPC.cs:82058-82078`)

```csharp
if (this.type == 676)
{
    int num39 = 8;
    float num40 = 1.1f;
    short num41 = 310;
    if (this.life <= 0)
    {
        num40 = 1.5f;
        num39 = 40;
    }
    for (int num42 = 0; num42 < num39; num42++)
    {
        Dust dust5 = Main.dust[Dust.NewDust(base.position, base.width, base.height, num41, 2 * hitDirection, -1f, 80, default, num40)];
        if (Main.rand.Next(3) != 0)
        {
            dust5.noGravity = true;
        }
        dust5.velocity *= 1.5f;
        dust5.velocity += base.velocity * 0.1f;
    }
}
```

A STANDALONE `if (this.type == 676)` block (NOT in the giant else-if
chain) — it runs on EVERY strike, live or dead. The dust is type 310,
alpha 80, scale 1.1 (live, 8 count) / 1.5 (death, 40 count), velocity
`(2·hitDirection, -1)`, the per-dust `Next(3) != 0` noGravity gate, then
`velocity *= 1.5` + `velocity += base.velocity * 0.1`. NO gore, NO
light, NO sound — dust-only.

### 2.2 The HitEffect scope census (the C# method)

`NPC.HitEffect(int hitDirection = 0, double dmg = 10.0)`
(`NPC.cs:81654-89834`) — the giant per-kind cosmetic table, reached
from `StrikeNPC_Inner`'s `:81488` tail (`this.HitEffect(hitDirection,
num)`, AFTER the damage write + knockback, BEFORE `checkDead`). The
676 block sits at `:82058-82078`, between the 667 GoldenSlime block
(`:82025-82057`) and the 552-554 DD2 goblin block (`:82079-82109`). The
method's structure:

- the party-hat town prologue (`:81660-81729`, gore),
- the daybreak prologue (`:81730-81751`, dust + the 189 spread),
- the per-type blocks (`:81752-89832`): the 686/690/594/661/521/668/
  636/657/658-660/667/**676**/552-578/620/619/618/… ladder, then the
  giant else-if chain (the classic mobs).

The 676 block is DUST-ONLY (no gore sites) — it is NOT a gore-lane
(d55/e5/f10/f5) arm. It belongs to the DUST lanes.

### 2.3 The port status — g62 owns the HIT half, g60 the DEATH half

**The HitEffect dust pass is NOT unported port-wide** (the f47 keep-one
277 (formerly 144) note's "cited-unwired" was stale at the seal tips). Census:

- **g62 (`npc_hit_dust.rs`, tip `32af98b44`)** ports the LIVE
  (`life > 0`) halves of the whole chain — **629 live sites, 0 missed**
  (research/240 §4). The 676 LIVE arm IS ported: `replay_hit_dust`'s
  `if s.life > 0 && matches!(s.npc_type, 658..=660 | 667 | 676)` block,
  the `_ => (8, 310, Color::default(), 1.1)` row, the per-dust
  `Next(3) != 0` noGravity gate + the `velocity *= 1.5` +
  `velocity += Vec2::new(s.vx, s.vy) * 0.1` post-writes (`:82075-82076`).
  The strike-time `HitDustSnapshot` capture (`npc.rs::strike_pooled`/
  `hurt`/`hurt_from_projectile`) + the `update_npcs`-top drain on
  `UpdateNPCs`. A 676-live witness exists (the FLAT-8 dust-310 alpha-80
  twin).
- **g60 (`npc_death_dust.rs`, tip `30fea0fe1`)** ports the DEATH
  (`life <= 0`) halves for kinds 231-697. **The 676 DEATH half is NOT
  in g60's match** — g60's per-kind matrix (research/110 §9) compresses
  `669-685` to `--` (no port kind row at g60's census time). The 676
  death arm (`:82063-82067`, the 40-count scale-1.5 burst) is a
  **census gap**: g62's module header classifies it "the death halves'
  40-count bursts are g60's lane", but g60's `lane_covers` (120 cs_ids,
  250-668) does NOT include 676 (the roster row landed on f47/d43 AFTER
  g60's census). **The 676 death half is UNOWNED at the seal tips.**

### 2.4 The verdict — NOT f47's to port; the task-seed

The brief offered "port small or task-seed g59-death-lane". The census
answers: **the 676 HitEffect arm is the dust lanes' (g62 hit / g60
death), NOT f47's.** f47's shimmer lane owns the 676 PER-TICK light/dust
(`AI_001_Slimes` `:61128-61150`) — a DIFFERENT arm (the unconditional
per-tick look, not the on-hit burst). Porting the HitEffect arm on f47
would double-draw against g62's live half at merge.

**The keep-one:**
- **676 HIT half → g62** (already ported, `replay_hit_dust`). f47 ports
  NOTHING here.
- **676 DEATH half → g60** (the death-lane owner by the §9 matrix's
  band assignment), but g60's census missed it (the roster gap). This
  is a **task-seed for g60** (the death lane): add the 676 row to
  `lane_covers` + `death_dust_draws` (the 40-count scale-1.5 dust-310
  burst, the `Next(3)` gate, the vel ×1.5 + base.vel ×0.1 — the SAME
  post-writes as the live half, only the count/scale differ). The seed
  is recorded here; g60's lane executes it (f47 does NOT port it —
  that would cross the dust-lane boundary).
- **The 658-660/667/657 death halves → e5** (gore-FIRST arms, e5 owns
  them whole per g60 §5.2). The 676 block is dust-only, so it is NOT
  e5's — it is g60's (the death-dust lane), confirming the seed target.

**f47's residual note retired:** the keep-one 277 (formerly 144) "HitEffect 676 arm
stays cited-unwired (the HitEffect dust pass unported port-wide)" is
SUPERSEDED — the pass IS ported (g62 live / g60 death-except-676). The
f47 ANALYSIS note is corrected (§6 below).

## 3. The b1 slimes-adapter activation-contract

The brief: *b1-slimes-adapter (your-676-stand-in-route-draws-nothing-
until-b1-lands — b1-tip? activation-contract).*

### 3.1 The b1 tip (`f010b85be`)

b1 wires aiStyle-1 to the faithful `npc_ai_slime::ai_001_slimes` via
the `npc.rs::ai_001_slime_family` adapter (`npc.rs:5625`), dispatched
from the aiStyle axis (`npc.rs:4666`, `1 => self.ai_001_slime_family`).
The adapter:

1. `session_candidate(player)` → the slot-0 target.
2. The at-site `wet_collision` probe (`Collision.cs:1572-1651`) →
   `(wet, honey_wet, shimmer_wet)`, sets `self.wet`.
3. Builds `SlimeCtx { players, expert_mode, net_mode: 0, ticker: 0,
   wet, honey_wet, shimmer_wet }`.
4. Calls `ai_001_slimes(self, world, &ctx, rng)` → `SlimeEffects`.
5. Drains the effects: `reset_frame_counter` → `frame_counter = 0`;
   `shots` → `wants_hostile_shot` (the 184 spike volleys).
6. `sprite_dir = facing`.
7. The generic UpdateNPC tail: `slime_gravity` (the 0.15/5.5 shimmer
   merge) + cap, the ±0.005 vx snap, `integrate_flyer_collision`.

The ten wave-A kinds' kind arm becomes `unreachable!("aiStyle 1 rides
the style axis")`; the sin-timer `update_slime` body is REMOVED.

### 3.2 The activation-contract (the f47 676 arm goes LIVE at b1's merge)

**The f47 676 light/dust arm is wired INSIDE `ai_001_slimes`** (the
`t == 676` block, `npc_ai_slime.rs:545-578`). On the f47 tip, the 676
routes through the `update_slime` STAND-IN (`npc.rs:5796`,
`NpcKind::ShimmerSlime => self.update_slime`), which draws NOTHING (the
stand-in is the sin-timer hop, no 676 arm). **The faithful core + the
sim drain are wired, but the stand-in route never reaches them.**

At b1's merge:
- b1's aiStyle axis claims style 1 (`1 => ai_001_slime_family`). The
  676's `ai_style()` is 1 (`npc/data.rs:2466`), so the 676 rides
  `ai_001_slime_family` → `ai_001_slimes` → the f47 `t == 676` arm goes
  LIVE.
- b1's kind arm (`NpcKind::Slime | … | UmbrellaSlime => unreachable!`)
  does NOT list ShimmerSlime (b1's base predates the f47/d43 676 row).
  **The merge must ADD `NpcKind::ShimmerSlime` to b1's unreachable kind
  arm** (the 676 is aiStyle 1, so its kind arm is unreachable once the
  axis claims style 1). The f47 kind arm
  (`NpcKind::ShimmerSlime => self.update_slime`) is RETIRED (the
  stand-in is removed by b1).
- The `wants_shimmer_slime_dust` drain (`world_sim/npcs.rs:6073`/
  `:6197`) is f47's, already wired — it drains the requests the now-live
  arm produces. The `shimmer_slime_light` draw feed (`draw.rs:458`) is
  f47's, already wired — it lights the 676 every tick regardless of the
  AI route (the light is draw-side, RNG-free).

**The contract:** b1's adapter is the ACTIVATION switch for the f47 676
arm. The merge keeps b1's adapter + axis arm, ADDS ShimmerSlime to the
unreachable kind arm, RETIRES the f47 `update_slime` route, and keeps
the f47 drain + light feed. **No f47 code change needed for the
activation** — the arm is already in `ai_001_slimes`; b1's merge flips
the route. The f47 `update_slime` stand-in stays byte-identical until
then (the documented merge seam).

### 3.3 The b1 `ai_001_slimes` signature — UNCHANGED (the f47 arm fits)

b1's `npc_ai_slime.rs` diff is ONLY the `#![allow(dead_code)]` comment
(the wiring swap). The `ai_001_slimes` signature
(`npc, world, ctx, rng -> SlimeEffects`) is byte-identical on both tips.
The f47 676 arm (the `t == 676` block) is INSIDE that fn — b1's adapter
calls it verbatim. **The merge dedupes `npc_ai_slime.rs` to the f47
version** (the f47 676 arm + the b1 dead_code comment — the f47 version
is the superset). The `SlimeCtx`/`SlimeEffects`/`slime_gravity`/
`wet_collision` surfaces are byte-identical (b1 did not touch them).

## 4. The d6 ShimmerImmunity-131 compose (disjoint keep-one)

The brief: *d6c-ShimmerImmunity-131 compose (d6-7edfcd49 for_kind-set
vs your-shimmering-tail — disjoint keep-one).*

### 4.1 The d6 set (`e2561efe1`, residual 9)

d6 ports `NPCID.Sets.ShimmerImmunity` verbatim (131 ids,
`NPCID.cs:10467`) as `const SHIMMER_IMMUNITY: &[u16]` in
`npc_buffs.rs`, applied at `for_kind`:
`buffs.buff_immune[353] = SHIMMER_IMMUNITY.contains(&kind.cs_id())`
(`NPC.cs:17838`). The set gates buff 353 (Shimmering) — the ShimmerGun's
1042 contact `AddBuff(353, 100)` (`Projectile.cs:29039`) rides
`AddBuff`'s immune guard. The 4 null-row witnesses exclude index 353
(the null `DebuffImmunitySets` row pins every OTHER id false).

### 4.2 The f47 shimmering-tail (the readers)

f47 owns the buff-353 FLAG + RAMP readers:
- `npc_buffs.rs:694-705` — the `set_flags_tick` 353 arm: immune →
  `DelBuff(i)` mid-scan, else `flags.shimmering = true`.
- `npc.rs:5072` — `self.shimmering = flags.shimmering` (the
  `update_preamble` derivation).
- `npc.rs:5078-5083` — the transparency ramp reads
  `self.buffs.buff_immune[353]` (the immune split).
- `world_sim/npcs.rs:3520` — `shimmer_wet_contact_pass`'s
  `try_adding_repeated_buff(353, 100, 10)` (the liquid-contact writer,
  `NPC.cs:93073`).

### 4.3 The compose — DISJOINT (the set vs the readers)

**d6 owns the SET (the `for_kind` immune-flag write); f47 owns the
READERS (the flag/ramp/contact tail).** The two are disjoint:

- d6's `SHIMMER_IMMUNITY` const + the `for_kind` write are NEW (the f47
  tip has neither — `for_kind` ends at the 69→36 derive, no 353 write).
- f47's readers (`set_flags_tick` 353 arm, the ramp, the contact pass)
  are NEW (d6 did not touch them — d6's `npc_buffs.rs` diff is ONLY the
  const + the `for_kind` write + the witnesses).

**The keep-one:** the merge keeps BOTH — d6's set write feeds f47's
readers. The `buff_immune[353]` flag is written ONCE (d6's `for_kind`)
and read by f47's tail. **No overlap, no dedupe needed on the
production code.** The witness compose:

- d6's `w4_d6_the_shimmer_immunity_set_gates_buff_353` (the set gate) +
  the gun-contact witness stay d6's.
- f47's `w4_f47_buff_353_sets_the_shimmering_flag` /
  `w4_f47_buff_353_immune_deletes_mid_scan` /
  `w4_f47_try_adding_repeated_buff_respects_the_low_threshold` stay
  f47's. **BUT** the f47 immune witnesses set `buff_immune[353] = true`
  MANUALLY (on `NpcBuffs::default()`) — they still pass after d6's
  `for_kind` write (they don't use `for_kind`). The f47
  `w4_f47_the_shimmer_contact_adds_buff_353` uses `NpcKind::Zombie`
  (type 3, NOT in the immunity set) — still passes.
- **The 4 null-row witnesses** (`npc.rs:30793`/`:31832`/`:31945`/
  `:32170`) assert `buff_immune.iter().all(|&i| !i)` /
  `!.any(|x| *x)`. After d6's `for_kind` write, a kind IN the immunity
  set would gain `buff_immune[353]` and break the all-false assert.
  **Census of every kind in those 4 witnesses** (cs_id via the
  `npc/data.rs` hand rows + the bestiary fall-through): the Nutcracker
  trio 348/349/350, the five eye variants 190/191/192/193/194, the
  wave-A zombie band DoctorBones 52 / TheGroom 53 / BaldZombie 132 /
  PincushionZombie 186 / SlimedZombie 187 / SwampZombie 188 /
  TwiggyZombie 189 / FemaleZombie 200 / ZombieRaincoat 223, the holiday
  zombies 319/320/321/331/332, the blood-moon pair 489/490 — **NONE is
  in the 131-id `SHIMMER_IMMUNITY` set** (mechanically checked each
  cs_id against the verbatim `NPCID.cs:10467` list). So the f47
  null-row witnesses still pass UNCHANGED. d6's own diff updated its 4
  null-row witnesses to exclude index 353 defensively (its null-row
  kind set differs); the f47 null-row kinds happen to be outside the
  set, so the f47 witnesses need NO change. **The compose is clean: d6
  sets the flag, f47's null-row witnesses are unaffected.**

**The 676/677 immunity note:** 676 AND 677 ARE in the set
(`NPCID.cs:10467` lists both). So the ShimmerSlime 676 is IMMUNE to buff
353 — it can never be shimmered by the gun or the liquid contact (the
`try_adding_repeated_buff` refuses). This is FAITHFUL (the 676 is
already a shimmer creature; the C# exempts it from re-shimmering). The
f47 676 per-tick light/dust arm does NOT read `shimmerWet` or buff 353
(it is the unconditional look), so the immunity does NOT gate the f47
arm. **The compose is clean: d6's set makes the 676 immune, f47's arm
ignores the buff — both faithful.**

## 5. The d43 676-row keep-one (your-AI-arm vs d43-row)

The brief: *d43-also-ported-676-ShimmerSlime-kind → keep-one
your-AI-arm vs d43-row.*

### 5.1 The two rows (both verbatim `NPC.cs:17566-17578`)

d43 batch 9 (`418336473`) and f47 doc-sweep 36 (`e8bacc2f`) BOTH landed
the 676 ShimmerSlime roster row. The stats are IDENTICAL (both cite
`:17566-17578`): 24x18, aiStyle 1, dmg 20, def 5, 80 HP, scale 1, value
150 copper, kb 1.0 (the class default), NPCHit1/NPCDeath1. The
`NpcKind::ShimmerSlime` variant, the `cs_id`/`net_id`/`to_u16`/
`from_u16` 676, the size/life/damage/defense/value/kb rows, the
hit/death cues — all byte-identical values.

### 5.2 The differences (the keep-one picks)

| surface | f47 (`e8bacc2f`) | d43 (`418336473`) | keeper |
| --- | --- | --- | --- |
| the `debuff_immune_ids` row | `ShimmerSlime => &[]` (the null row, `npc_buffs.rs:494`) | `ShimmerSlime => &[]` (same) | either (identical) |
| the AI route | `NpcKind::ShimmerSlime => self.update_slime` (the stand-in, `npc.rs:5796`) | `NpcKind::ShimmerSlime => self.update_slime` (same stand-in) | **RETIRED by b1** (§3) — both become unreachable |
| the FindFrame | the DEDICATED `case 676` arm (`npc.rs:7898-7909`): the count read `npc_rows(676)`, the 302/304 snap noted-never-matches | the GROUP arm (`case 658/659/667/676`, `:70820-70843`): `self.rows()` wrap, the same counter logic | **f47** (the dedicated count-read shape — see §5.3) |
| the light/dust arm | WIRED (`ai_001_slimes` `t == 676` + `shimmer_slime_light` + the drain) | NOT ported (d43's `npc_ai_slime.rs:530` still "unported kind") | **f47** (the superset) |
| the protocol/wire id | 676 free (the f47 `to_u16`/`from_u16`/render-key/bestiary) | 676 (the d43 exhaustive-match arms) | dedupe (same id) |
| the bestiary/sheet/portrait | the f47 row | the d43 row | dedupe (same credit) |
| ALL_KINDS count | 270→271 (f47) | the d43 batch adds 5 (674-678) | the merge unions (the 676 counted ONCE) |

### 5.3 The FindFrame keep-one (the one real shape difference)

The C# `case 658 : case 659 : case 667 : case 676` (`:70820-70846`) is
ONE shared arm: the counter ticks once, doubled while moving
(`num2 > 0`), tripled in the crouch class (`num2 == 4`); `>= 8` advances
one row and wraps at `Main.npcFrameCount[type]`. The 302/304
spriteDirection snap (`:70824-70827`) never matches these kinds.

- **f47's shape**: a DEDICATED `NpcKind::ShimmerSlime` arm reading
  `npc_rows(676)` (= 2, `Main.cs:67116`'s row) explicitly, with the
  302/304 snap noted-never-matches. The counter logic is the same
  (`1 + (num2 > 0) + (num2 == 4)`).
- **d43's shape**: the GROUP arm (658/659/667/676 share one match arm),
  wrapping at `self.rows()` (the generic `npc_rows(texture_net_id)`).

Both are FAITHFUL (the C# is one shared arm; the wrap count is
`npcFrameCount[676]` = 2 either way). **The keep-one picks f47's
dedicated shape** — it is the more explicit count-read (the
`npc_rows(676)` call names the 676 row directly, matching the f47
roster-row's "the wrap rides `Main.npcFrameCount[676]`" comment), and
the f47 lane owns the 676 AI-arm superset. At merge, the d43 group arm
is split: the 658/659/667 siblings stay d43's (their kinds are d43's
batch-mates), the 676 rides f47's dedicated arm. **The behaviour is
identical** (the same counter, the same 2-row wrap) — the keep-one is a
shape dedupe, not a behaviour change.

### 5.4 The keep-one verdict

**KEEP the f47 676 row** (the AI-arm superset: the roster row + the
wired light/dust arm + the dedicated FindFrame). **DROP the d43 676
row's duplicate surfaces at merge** (the variant/const/stat rows dedupe
to one copy — identical values; the d43 FindFrame group arm loses its
676 member to f47's dedicated arm; the d43 `update_slime` route is
retired by b1 like f47's). The d43 batch-mates (674/675/677/678) stay
d43's. **The 676 is f47's kind** (the shimmer lane's transform target);
d43's row was the roster-gap fill that f47's superset subsumes.

## 6. The f47 doc corrections (this seal's edits)

- **ANALYSIS "The shimmer second half"**: the keep-one 277 (formerly 144) block's
  "HitEffect 676 arm stays a cited skip (the HitEffect dust pass is
  unported port-wide)" is SUPERSEDED — the pass IS ported (g62 live /
  g60 death-except-676, §2). Corrected to name the owners + the g60
  death-half seed.
- **`tr-core/src/shimmer.rs`**: the `lunar_brick_for_moon_phase` switch
  re-pinned to the RAW enum indices (the e19 `cab982244` fold, §1.5) +
  the witness corrected.
- **research/121 + research/277 (formerly 144)**: the FINAL pointers to this seal
  (the family bible).
- **INDEX**: the 151 row.
- **anchors.txt**: the lunar-brick RAW-enum row (the e19 fold).

## 7. The family bible (the single-owner map, the merge checklist)

The merger's keep-one checklist for the shimmer family (each C#
statement draws/writes exactly once):

1. **The item splash** → f47 `Splash`/`splash_due`/`ItemDrop::splash`
   (379-ARBITER-FINAL-v2). DROP the e19 `ItemSplash`/`pending_splash`/
   `take_splash`/`item_splash` (the exact hunks re-pinned at `0fb0ac4ad`
   in v2). KEEP the e19 flags/gravity/repulsion family + the
   dry-integration full-velocity SPLIT half.
2. **The 16-box** → ONE const (`item::SIZE = 16.0`) + ONE set of
   re-pins (f47 ≡ e19, same values). The e19 145 f47 row struck.
3. **`box_is_wet`** → ONE copy (the e19 name+body, f47-restored).
4. **The lunar-brick table** → ONE copy (the RAW-enum fix, e19 ≡ this
   fold). The OTHER moon-phase readers unchanged (already RAW-correct).
5. **The item transform chain** → f47 resolve + e19 apply (the
   `shimmer_tail`/`apply_decraft` `wet = true` writes are e19's; the
   f47 Transform arm's missing `wet` write is e19's to keep). The
   `rise_over_shimmer` two-row scan → e19's (the f47 single-tile read
   reconciles onto it). The collision engine → f47's
   `tile_collision_opts` (the shared seam).
6. **The NPC transform/ramp/GetShimmered** → f47 (no co-lane).
7. **The buff-353 immune set** → d6's `for_kind` write; **the flag/ramp/
   contact readers** → f47. Disjoint, both kept. The 676/676 immunity
   is faithful (the f47 per-tick arm ignores the buff).
8. **The town state-25** → f47 seam + b3 driver (the ONE-driver
   contract, research/121). The `shimmering:false` fold lifts to the
   live buff-353 flag at b3's merge.
9. **The 676 roster row** → f47 (the superset). The d43 duplicate
   surfaces dedupe; the d43 FindFrame group arm loses its 676 member.
10. **The 676 per-tick light/dust** → f47 (`ai_001_slimes` arm +
    `shimmer_slime_light` + the drain). ACTIVATED by b1's adapter (§3).
11. **The 676 HitEffect** → g62 (hit) + g60 (death, the seed §2.4). f47
    ports NOTHING here.
12. **The aiStyle-1 wiring** → b1's adapter + axis arm; ADD ShimmerSlime
    to the unreachable kind arm; RETIRE the f47/d43 `update_slime`
    route. The `npc_ai_slime.rs` dedupes to the f47 superset (the 676
    arm + the b1 dead_code comment).
13. **The lights/AddLight** → f47 (676 NPC-source) + e19 (item-sources),
    DISJOINT (§1.4). The `XAnchor`/`tile_x`/`tile_y` vocabulary shared,
    deduped to one copy. The 16-box fixture re-pins dedupe (same values).

**The witness union (the merger's verification set):** the f47 shimmer
witnesses (the 12 keep-one + the `w4_f47k2` v2 fold + the doc-sweep +
the phase witnesses) + the e19 KEEP list (the flags/gravity/repulsion
family + the dry-integration full-velocity SPLIT half) + d6's 2 +
g62's 676-live + the g60 death-half seed's witness (when g60 lands it).
All WRITTEN NOT RUN on the code-only lanes; the merge owes them a
compile window (`cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids`).

**The e19 ANALYSIS merger bible compose:** e19's SHIMMER SEAL published
its own 18-row family bible (ANALYSIS "The shimmer family — the merger
bible"). This doc is the f47-side complement — the two agree on every
row EXCEPT the splash keeper (e19's bible still says "e19 KEEPS", its
lane's superseded claim; 379-ARBITER-FINAL-v2 overrules it — f47 keeps,
the e19 drop-hunks apply). The rows this seal ADDS beyond e19's bible:
the 676 HitEffect owner split (g62 hit / g60 death + the seed), the b1
activation-contract detail (the unreachable-kind-arm addition), the d6
ShimmerImmunity compose (the set vs the readers), the d43 676-row
keep-one (the FindFrame group-arm split), and the lights/AddLight
disjointness proof. The merger reads BOTH bibles + v2 for the splash.

## Verification

Code-only per the lane contract: no cargo/tests/rustfmt/graphify. Every
verdict re-read against the cited C# windows (≤100-line reads) and the
lanes' pushed tips (`git show` for the co-lanes, direct reads for f47).
The lunar-brick fold is byte-identical to e19's `cab982244` (the switch
+ the witness). The 676 HitEffect census re-read `NPC.cs:82058-82078`
verbatim + the g62/g60 dispatch matches. The d6 set compose verified
the f47 null-row kinds are outside the 131-id set (no witness change).
The b1 activation-contract verified the `ai_001_slimes` signature is
byte-identical on both tips (the f47 676 arm fits b1's adapter). 1 code
fold (the lunar-brick RAW-enum fix + its witness) + the doc corrections,
WRITTEN NOT RUN.
