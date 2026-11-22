# 109 — The snail band claims (W4-c9): protocol ids, residuals, co-lane compose

Status: RESEARCH / CLAIMS REGISTER — companion to d37's
`108-roster-master-reconcile.md` (read at its pushed tip `08f7d25cf`,
branch `task/W4-d37-npc-kinds-211-280`). Author lane: W4-c9, branch
`task/W4-c9-style67-snails`, tip `c23162847` (phases 1-3) + this
residual commit. Code-only mode: no compiler/tests invoked; every
claim below is scripted git-show extraction or a cited source read.

## 1. The claim

| Kind | Protocol id (`to_u16`) | cs_id / bestiary net_id | aiStyle | Phase-1 commit |
|---|---|---|---|---|
| `NpcKind::Snail` | **359** (vanilla-free) | 359 | 67 | `37baf0d1c` |
| `NpcKind::GlowingSnail` | **360** (vanilla-free) | 360 | 67 | `37baf0d1c` |
| `NpcKind::MagmaSnail` | **655** (vanilla-free) | 655 | 67 | `37baf0d1c` |
| `NpcKind::Shimmerfly` | **677** (vanilla-free) | 677 | 64 | `37baf0d1c` |

The phase-1 "354+ band unclaimed" scan was run against the LANDED
families only (base `a725acb04`, max proto 353). It predates the
kinds-wave branches' pushes. §3 corrects the record.

Bijection witness (scripted, `to_u16` ↔ `from_u16` arm census over
git-show blobs, zero duplicate arms, `ALL_KINDS` length equal):

| Tip | Arms | Bijection |
|---|---|---|
| base `a725acb04` | 270 | OK |
| c9 `c23162847` | 274 | OK |
| e26 `048cca6cf` | 274 | OK |
| c4 `22cec16f7` | 283 | OK |
| d37 `08f7d25cf` | 316 | OK |

## 2. The lead's collision check (e26 ladybugs × c4 fish × c9 snails)

**CLEAN — all three pairwise intersections empty.**

- e26 `048cca6cf` new kinds: Frog **361**, Duck **362**, LadyBug
  **604**, GoldLadyBug **605** (vanilla-free; its commit message
  explicitly acknowledges "c9 claims snails 359/360/655/677").
- c4 `22cec16f7` new kinds: FlyingFish **354** (port-local),
  CorruptGoldfish **229**, Arapaima **237** (port-local), BloodFeeder
  **241**, CrimsonGoldfish **465**, GoldGoldfish **592**, Pupfish
  **607**, Dolphin **615**, GiantFlyingAntlion **509**, FlyingAntlion
  **581**, EyeballFlyingFish **587**, Pufferfish **688**, Orca **692**.
- c9 × e26 = ∅, c9 × c4 = ∅, e26 × c4 = ∅. The four snail ids are
  untouched by both lanes; 655/677 are free on every tip scanned (§3).

## 3. The REAL collisions: the 354-399 port-local corridor

Full scan: **353 pushed task branches**, 21 unique `npc/data.rs`
blobs (base-identical blobs skipped), `to_u16` new-arm extraction per
blob. Three branches squat c9's 359/360 — all of them picked the same
"next free above 353" corridor d37's §2a already flagged for d35×d40:

| Squatter | Tip (pushed) | Kind at proto 359 | Kind at proto 360 | Band |
|---|---|---|---|---|
| **d37** kinds wave | `750d8e446` (in `08f7d25cf`) | FlyingFish (cs 224) | FlyingSnake (cs 226) | 354-399 port-local |
| **d42** WIP checkpoint | `88df58090` | FairyCritterPink (cs 583) | FairyCritterGreen (cs 584) | 354-363, **self-declared SUPERSEDED** |
| **g8** kind-wire lane | `0f31e10e3` | Pixie (cs 75) | Wraith (cs 82) | 354-384 port-local |

655/677: **zero squatters** across all 353 branches.

Kind-NAME overlap (not an id collision): **c7** `c2cc6ec61` (pushed
2026-11-20 23:56, BEFORE c9's phase 1 at 01:08) also defines
`NpcKind::Shimmerfly` at proto/cs **677** with identical stats
(10×10, aiStyle 64, 5 HP, dmg 0, noGravity; bestiary row + hover net
seed). c9's phase-1 claim scan missed it (it scanned ids against the
landed roster, not pushed kind names). The two definitions are
semantically identical — the merge dedupes to ONE variant; c7 owns
the 355/358/654 trio + the pooled `style == 64` sim arm, c9 owns the
snail trio + the shimmer seams. §5 covers the compose.

Also on record (d37's playbook already lists them): e26 × d37 collide
at **361 Frog↔Pirate, 362 Duck↔GoldfishWalker**; c4 × d37 at
**354 FlyingFish↔CochinealBeetle**. d37's 108 §2a "d37 × any: 0" was
true only for its five-branch union — c9/e26/c4/g8/d42 were outside
its scan window.

### Numbering guidance (extends 108 §3, no code changed here)

The 108 plan renumbers ONLY d35 (→698-730) and keeps d37's 354-399.
That plan predates c9/e26/c4/g8's pushes. The corridor now has FIVE
claimants (d37, d35-renumbered-out, g8, d42-superseded, plus the
vanilla-free c9/e26/c4 rows at 354/359/360/361/362). Options for the
merge wave, in the 108 §3 principle's spirit ("renumber exactly the
port-local bands, keep vanilla-free rows untouched"):

1. **c9 keeps 359/360/655/677** (vanilla-free, the d38/d40 precedent;
   its witnesses + the `npc_ai_style67` type gates read the cs ids,
   which equal the proto ids — a renumber would split that invariant
   for zero gain). Same for e26's 361/362/604/605 and c4's vanilla-
   free rows.
2. **d37's 354-399 band must move** (it is port-local; 21+ of its
   slots now collide with pushed vanilla-free claims: 354 c4,
   359/360 c9, 361/362 e26). The 108 §3 "keep d37 untouched" row is
   superseded by the pushes. Suggested: extend the d35 renumber
   corridor upward (731+ after d35's 698-730), or interleave d37
   into the 400-428 + 489-518 free slots — the merge wave owns the
   pick; the constraint is only "no vanilla-free id may be taken".
3. **g8's 354-384 band must move** for the same reason (it also
   collides d37 wholesale). g8 is a wiring lane riding d22-d33
   kinds; its numbering should follow whichever kinds-wave band
   survives.
4. **d42's checkpoint is already self-superseded** (its commit
   message: "SUPERSEDED next commit by the 108 union plan's
   vanilla-free band") — no action beyond not merging the WIP blob.
5. **c7 × c9 Shimmerfly**: dedupe to one variant at 677. Keep c7's
   grouped stat arms (the firefly family shares them) and c9's
   shimmer/transform wiring; the enum doc, bestiary row and net seed
   are identical in effect. The exhaustive-match splice must not
   double-arm `to_u16` (the `deny(unreachable_patterns)` guard makes
   that a hard error, the S1-F-1 lesson).

## 4. Residual verification (the lead's three items)

### 4a. Snail catch — VERIFIED END-TO-END (no code needed)

The lead's "2121-range?" guess is wrong: 2121/2122 are the FROG/DUCK
catch items (e26's lane). The snail rows are:

| NPC | catchItem | Item name | bait | Item.makeNPC | C# cite |
|---|---|---|---|---|---|
| 359 Snail | **2006** | Snail | 10 | 359 | `NPC.cs:13585` |
| 360 GlowingSnail | **2007** | Glowing Snail | 15 | 360 | `NPC.cs:13599` |
| 655 MagmaSnail | **4849** | Magma Snail | 35 | 655 | `NPC.cs:17293` |
| 677 Shimmerfly | **5350** | Shimmerfly | 0 | 677 | `NPC.cs:17591` |

Chain (all read, all present on the c9 tip):
- `bug_net.rs::catch_item` rows 142/143/185/193 (verbatim, witnesses
  at :1146/:1147/:1178/:1186 already pin them).
- `Main.npcCatchable` census (`Main.cs:9326-9418`): 359/360 explicit
  (`:9343-9344`), 655 explicit (`:9410`), 677 explicit (`:9413`) —
  the port's `npc_catchable` matches (`bug_net.rs:219/220/270/273`;
  655 rides the `639..=655` range, the C# lists 639-655 individually).
- The swing pass keys `n.kind.cs_id()` (`bug_net.rs:668/757`) — the
  snail kinds' cs_id rows (phase 1) feed it; `catch_verdict`'s lava-
  bait gate (`is_lava_bait` 4845/4847/4849, `ItemID.cs:1306`) already
  carries the 4849 row + the lavaproof-net witness (:1261).
- Release: `registry::make_npc` reads the GENERATED itemdata rows —
  2006→359, 2007→360, 4849→655, 5350→677 all present
  (`itemdata.rs:2172/2173/5015/5516`, `make_npc` field). The release
  arm resolves `kind_from_net_id` → bestiary ENTRIES net_id
  359/360/655/677 (phase 1, `bestiary.rs:2181/2189/2197/2205`) →
  the four kinds. `release_plan`'s default arm (`NPC.cs:80160-80163`)
  is correct for all four (no 583/356/148/614 special row).
- The cage items (SnailCage/GlowingSnailCage/MagmaSnailCage) are
  furniture (tile 724 place row on the critter items themselves) —
  the f14 cage lane's territory, not a catch residual.

### 4b. GlowingSnail/MagmaSnail lights — VERIFIED (no code needed)

C# `NPC.cs:34385-34392`: 360 → `AddLight((int)Center.X/16,
(int)Center.Y/16, 0.1, 0.2, 0.7)`; 655 → (0.6, 0.3, 0.1). The cast is
BEFORE the int divide. Port: `npc_lights::snail_light`
(`XAnchor::CenterTruncated`, the exact colours, 359/677 → None) fed
by `app/frame/draw.rs:500-509` inside the per-NPC `EntityLights`
loop (hp>0 gate, netOffset added, slot order). The machine's
`SnailEvents::light` is deliberately discarded by the adapter
(`npc.rs:5803`) — one feed, the post-step position (the documented
one-step lag, the `npc_light` precedent). Witnesses:
`npc.rs:34031-34036` (the 360 blue / 655 orange / 359-none pins).

### 4c. Shimmerfly Transform-chain — VERIFIED + one residual FIXED

The c9 chain (all read on the c9 tip):
1. `snail_ai_step`'s shimmer probe (`npc_ai_style67.rs:154-169`,
   `NPC.cs:34350-34360`: the inclusive maxTiles clamp quirk, the
   cast-after-divide tile probe, liquid>30) → `shimmer_transform` /
   `shimmer_statue_despawn` intents.
2. The adapter (`npc.rs:5807-5828`) queues `wants_transform =
   Some(677)` (the style75 one-tick drain fold, documented) or the
   statue despawn (timer-0 + `wants_spawn_no_cycle` + damage 0).
3. The sim drain (`world_sim/npcs.rs:6207-6216`) maps 677 →
   `NpcKind::Shimmerfly` → `transform_to_kind` (`npc.rs:5158-5204`,
   `NPC.cs:81084-81114`: bottom-anchored reposition, SetDefaults
   subset, TargetClosest, integer life rescale floored at 1).
4. The fresh fly rides `ai_064_shimmerfly` (`npc.rs:5899-5925`) —
   the threat-view pre-pass (`world_sim/npcs.rs:4392-4412`) + the
   raw noTileCollide integration.
C# rows confirmed: `NPCID.cs:10470` lists 359→677, 360→677, 655→677
(and NOT 677→anything); `NPC.cs:91561-91565` the Transform arm;
`NPC.cs:17579-17593` the 677 SetDefaults row.

**The residual (FIXED in this commit):** `NPC.cs:17838` —
`buffImmune[353] = NPCID.Sets.ShimmerImmunity[type]` — was unported
(the port's `NpcBuffs::for_kind` stopped at the 20/69 derived rules).
`NPCID.cs:10467` lists **677** in the set: the Shimmerfly must reject
buff 353 (Shimmering) so it never re-shimmers. Without the write the
f47 transparency ramp (which reads `buff_immune[353]`) would accept
353 on the fly and `resolve_npc_shimmer(677)` → Nothing would leave
it stuck semi-transparent. The fix ports the set VERBATIM
(`shimmer_immune`, 131 ids, scripted zero-drift compare against
`NPCID.cs:10467`) + the unconditional post-dictionary write keyed on
`cs_id`. The snail trio is NOT in the set (susceptible — their
aiStyle-67 probe owns the transform); BlazingWheel 72 / ForceBubble
384 / KingSlime 50 gain the write their C# rows always had (a
faithfulness correction, not a behaviour change for any ported
shimmer path — none of them has a transform row). Witness:
`w4_c9_shimmer_immunity_write_keys_the_cs_type` (written not run).

### 4d. The f47 co-lane compose (read at `507a0da5c`)

f47 ports the GENERIC `GetShimmered` drain: the transparency ramp
(`npc_shimmer_transparency_tick`, the f64-crossing pin), the
`shimmer_wet_contact_pass` (buff 353 via `try_adding_repeated_buff`),
and `drain_shimmer_transforms` → `tr_core::shimmer::
resolve_npc_shimmer` (the `SHIMMER_TRANSFORM_TO_NPC` table —
scripted check: 359→677, 360→677, 655→677 present, 677 absent, 114
rows) → `kind_from_net_id(677)` → `transform_to_kind`.

Compose verdict: **the two lanes are complementary, with one
documented overlap.**
- The snail trio is EXEMPT from f47's wet-contact pass (`style == 67`
  in the `:93055` no-wet list — f47's own port has it,
  `shimmer_wet_contact_pass`), so the generic ramp never fires for
  them; the c9 in-AI probe is their ONLY shimmer path, exactly like
  the C# (the probe transforms on liquid contact, no 90-tick ramp).
- The Shimmerfly 677 is NOT exempt from the wet pass, but with the
  §4c fix its `buff_immune[353]` rejects the buff — the C# behaviour
  (`TryAddingRepeatedBuff`'s immune guard, `NPC.cs:93305`).
- The overlap: BOTH drains can resolve a snail transform if a future
  path ever arms `wants_shimmer_transform` on a snail (none does
  today — the wet pass exempts them). At merge, the c9 adapter's
  `wants_transform` queue and f47's `drain_shimmer_transforms` are
  idempotent per tick (`transform_to_kind` clears `wants_transform`;
  the f47 drain clears its own flag first), but the merge wave
  should fold the c9 statue-despawn latch into f47's `StatuePoof`
  arm (f47 plays `shimmer_effect` + sets `npc_no_spawn_cycle`; c9's
  adapter documented the ShimmerEffect visual skip — f47's port
  SUPERSEDES that skip: keep f47's effect call, drop the c9 skip
  note). The c9 one-tick drain fold (the old-velocity tail skip)
  stays documented either way.
- f47's `resolve_npc_shimmer` reads `kind.net_id()` — for the snail
  kinds net_id == cs_id == proto (the vanilla-free claim), so the
  table lookup is correct under §3's guidance. If the merge wave
  ever renumbers c9 (it should not), the f47 table keys would break
  — another reason the vanilla-free rows stay put.

### 4e. The c7 co-lane compose (read at `c2cc6ec61`)

c7's sim arm is `if style == 64` (ALL four members, pooled, slot
order, the threat views built at the fly's own slot turn); c9's is
kind-gated (`64 if Shimmerfly`) with a tick-start pre-pass (the
documented one-tick lag for later slots). At merge, c7's pooled arm
SUBSUMES c9's kind-gated arm for the Shimmerfly: keep c7's pool step
(the C#-exact slot-turn scan), keep c9's `shimmerfly_threat_views`
carrier ONLY if the solo `update_rng` path still needs it (the
style75 solo fold), and dedupe the Shimmerfly variant per §3.5. The
`ai_064_step` module itself is shared — c9's phase 2 edited only its
doc header (the 677 branches were already in the S1-D-3 module);
c7's phase edits are the same module's wiring doc — no semantic
conflict in the machine.

## 5. Witness ledger (this commit)

- Scripted: the 5-tip bijection census (§1), the 353-branch squatter
  scan (§3), the c7/c9 Shimmerfly definition compare, the f47
  transform-table membership check, the ShimmerImmunity 131-id
  zero-drift compare (§4c). All git-show based, reproducible from
  the named tips.
- Rust: `w4_c9_shimmer_immunity_write_keys_the_cs_type`
  (`npc_buffs.rs`) — WRITTEN NOT RUN (code-only mode).
- Reads: `NPC.cs:13573-13600/:17280-17294/:17579-17593/:34348-34392/
  :91500-91574/:93303-93313/:17820-17838`, `NPCID.cs:10460-10479`,
  `Main.cs:9326-9419`, the c9/e26/c4/d37/c7/f47/d42/g8 tip blobs.
