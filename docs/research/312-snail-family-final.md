# 312 — The snail-family FINAL bible + the W4-c9 seal census

Status: FINAL / SEAL RECEIPT — W4-c9, branch `task/W4-c9-style67-snails`.

> **REGISTRY-RENAME 281 → 312 (this round).** The seal commit `9252d6f31`
> claimed 281 citing the registry's round-2 banner ("Numbers 281+ are
> FREE") — but that banner is **SUPERSEDED by §7 (round 3)** in the same
> registry file (`150-research-number-registry.md` at
> `origin/task/W4-d54-dust-sites-world` tip `38bfd1d22`): §7 assigns the
> block **281–303** to rename losers (281 = f26's 7h addendum assignment,
> 282/283 = f26's 7i/7j) and states "**the free floor is now 304**… Do NOT
> pick max-on-origin+1". A fresh full sweep of **all 363 remote refs**
> (`git ls-tree docs/research/` per ref, numeric-key extraction ≥281)
> found 281 at **TEN** tips (c9/c25/c32/d30/d106/d125/d9/f39/g48/g8),
> 282 ×3, 283/284/293/298 taken — the seal's "zero claims ≥281" was a
> stale/incomplete sweep. The floor 304 is ALSO contested (d119
> housing-bible + g17 gore-world-final), 305 = f4, 308 = g13. **312 is
> free at every ref** and sits clear of the live 304-308 claim cluster
> (306/307/309-311 are the likely next max+1 picks) — claimed here per
> the first-pushed-wins rule. All refs re-pointed (INDEX row 312,
> ANALYSIS FINAL seal, PORT_MAP row 360, ROADMAP:424, anchors.txt:903).

The lane's
earlier register stays `109-snail-band-claims.md` (109 = c9's win per the
registry §4 correction row: first-add 07:13:30 `2042d77f1`; d20→228,
d36x→229, d36→236 are THEIR renames, not ours). Code-only mode: no
cargo/rustc/rustfmt invoked; every claim below is a scripted census or a
cited source read.

## 0. The alive-probe receipt (the c9b-resume question)

The seal brief warned a c9b-resume agent might be active on this
worktree (dirty-8 at head `c830aec3d`). Probe: full-diff SHA256
`4E0E4229BFF12A67DCDFBF00494E073A719982A7BDDC2F462E3274C68D1FCA4A` at
14:25:02, **unchanged at 14:26:58 and 14:29:09** (2×90s), head unchanged,
dirty-file mtimes 12:58–13:34 (>50 min stale at probe time). Verdict:
**NOT ALIVE / not advancing** — the dirty-8 is a complete-but-uncommitted
GoldBunny/GoldFrog wiring round. Landed verbatim as wave 2 (`49d4cfb61`),
no clobber. (No `c9b` branch exists on origin — d37's `0466ecf81`
already noted "no c9b branch exists - apply on the same lane".)

> **Successor-round addendum:** the co-lane DID wake later (its own
> 2×90s dead-probe at 14:25–14:29, then wave 2 `49d4cfb61` 15:33,
> witnesses `abf900c2d` 15:47, FINAL seal `9252d6f31` 15:57, pushed).
> The successor agent's probes: 15:07:32 / 15:14:55 (dirty-8 mtimes
> 12:58–13:34, identical hashes — idle, push executed), then the co-lane
> advanced mid-task; re-probes 16:02:57 / 16:05:17 at `9252d6f31` clean
> and stable (its seal declares no further commits) — this round's
> rename + drop-hunks landed on the quiet tree, no clobber.

## 1. Seal item 1 — style-67-snails + adjacent-critter-kinds: LANDED, census

**The snail family (phases 1–3 + residual, `37baf0d1c`/`f5fbc2b3c`/
`c23162847`/`2042d77f1`):** Snail 359 / GlowingSnail 360 / MagmaSnail 655
(aiStyle 67) + Shimmerfly 677 (aiStyle 64) — kinds, protocol slots
(vanilla-free), SetDefaults rows, bestiary ENTRIES, net seeds, debuff
null-rows, the `ai_067_snail` adapter (shimmer Transform(677) / statue
despawn seams, `snail_no_gravity` carrier, WalkDownSlope+TileCollision+
MoveSnailOnSlopes tail), the kind-gated `ai_064_shimmerfly` arm, the
find_frame 359/360/655/677 machines, `npc_lights::snail_light` →
EntityLights, sheets/portraits/rotation whitelist, the `shimmer_immune`
131-id verbatim set (`NPCID.cs:10467`) + the `NPC.cs:17838` write.

**The adjacent gold critters — all FIVE now landed:**
- Wave 1 (`c830aec3d`): GoldBird 442 (aiStyle 24) / GoldGrasshopper 446
  (aiStyle 1) / GoldMouse 447 (aiStyle 7) + the `npc_ai_critter` module.
- Wave 2 (`49d4cfb61`, the dead-probe leftovers + this seal's witnesses):
  GoldBunny 443 / GoldFrog 445 (aiStyle 7 both, `NPC.cs:14266`/`:14294`),
  18×20 / 12×10 (`:14264-14265`/`:14292-14293`), dmg 0 / def 0 / 5 HP /
  value 0 / kb 1, NPCHit1 + NPCDeath1 (`:14270-14271`/`:14298-14299`),
  the {31} Confused rows (`NPCID.cs:8612-8652`), catchItem 2890/2892
  (`bug_net.rs:152`'s `442..=448 => 2889 + type - 442` range covers them;
  `npc_catchable`'s `442..=448` arm at `:275`), both → the committed
  `npc_ai_critter::critter_ai` (the bunny default-cap-1/accel-0.07 row
  `:53865-53866`, the flag10 frog dry row `:53453`; the wet-swim override
  is the documented stand-in), the find_frame case-443 bunny-group arm
  (`NPC.cs:76851-76904` read verbatim this round: the grounded flip, the
  idle row-0 zero, the |vx|+1 STRICT >6 accumulator, the FULL-7 wrap —
  no squirrel decrement — the air rows 4/6) and the case-445 frog-group
  arm (`:76618-76685`: the wet |vx| rows 10/11/12, the grounded idle
  0..5 >6 step, the 6-tick walk phases 0/6/8/9 with the >=23 reset, the
  air rows 9/8), netglue `spawn_cave_walker` seeds, NPC_443/NPC_445
  sheets (`Main.cs:67128`/`:67118`, 7/13 rows — `NPC_FRAME_COUNT[443]=7`,
  `[445]=13` scripted-verified), the ENTRIES + portrait + draw arms,
  the shimmer targets (`shimmer_tables.rs:21` carries `(443,677)` and
  `(445,677)` — all five gold ids 442-447 present).

**The scripted census (current tree, wave-2 tip):**

| Roster surface | Count | Cross-check |
|---|---|---|
| `NpcKind` enum variants | **279** | == ALL_KINDS == from_u16 arms |
| `ALL_KINDS` rows (`npc.rs:24358`) | **279** | set-identical to the enum, zero dups |
| `from_u16` numeric arms (`npc/data.rs`) | **279** | bijection holds |
| `to_u16` arms | exhaustive match | compiler-forced (no `_` arm) |
| bestiary `ENTRIES` credit rows | **258** (256+2) | ids unique (the existing dedup witness) |
| `BESTIARY_NPC_SHEET_IDS` | **190** decl == 190 numeric rows | the Lookups vec has 190 `atlas.sheet` entries, same order |
| `NPC_FRAME_COUNT` | 697 slots | [359/360/655]=6, [677]=4, [442]=5, [443]=7, [445]=13, [446]=2, [447]=6 |
| s1_f_3 pin (`npc.rs:28261` region) | **279** | `covered + sentinel == 279` |

Witnesses extended this round (WRITTEN NOT RUN): the 443 walk +3.0/tick
strict->6 edge + the full-7 wrap + the air rows, the 445 wet |vx| ladder
+ the 6-tick phases + the 23 reset, the sheets witness now loops all
five gold ids (`w4_c9_the_gold_critter_frame_machines` /
`w4_c9_the_gold_critter_sheets_resolve`).

## 2. Seal item 2 — the d37 roster ALL_KINDS fix: EXECUTED-VERIFIED, superseded in-lane

d37's `0466ecf81` published `tools/_d37_c9_allkinds_fix.patch` (in the
TR-W4-d37 worktree; 3 sites: the decl 270→274 at `npc.rs:23972`, the
four snail rows after WyvernTail at `:24250`, the s1_f_3 pin 270→274 at
`:27862`) against c9's then-tip `2042d77f1`.

**EXECUTE receipt (this seal, scratch index, the real index untouched):**
- `git apply --cached --check` against a scratch index read-tree'd at
  `2042d77f1`: **exit 0**. `git apply --cached`: exit 0.
- Post-apply scripted census of the staged blob
  (`69cb3559ddea6fa69112597d2fd8e40da0d866cf`): decl **274** at line
  23972, rows **274**, Snail/GlowingSnail/MagmaSnail/Shimmerfly all
  present, s1_f_3 pin **274**. d37's hunk is byte-correct.
- Against the CURRENT tree the patch **does not apply (exit 1)** —
  expected and correct: c9 already fixed the same defect in-lane at
  `c830aec3d` ("the latent ALL_KINDS fix", 270→277 with the three gold
  kinds) and wave 2 advanced it to **279**. The round-trip invariant
  (decodable == ALL_KINDS.len) holds at every c9 tip from `c830aec3d`
  on. **No apply needed; the merger drops d37's hunk as
  already-satisfied** (the Shimmerfly row still dedupes to c7 at the
  108 merge — d37's own note).

## 3. Seal item 3 — the c8 SIGHT gate auto-covers the snails: VERIFIED

c8's generic provider gate (`origin/task/W4-c8-style66-crawling-critters`
tip `7c2de9c06`, `bestiary.rs::counts_as_a_critter`):
`max_hp() <= 5 && contact_damage() == 0 && cs_id != 594 && cs_id != 686`.
The c9 rows feed it directly: `npc/data.rs:3732` (Snail | GlowingSnail |
MagmaSnail | Shimmerfly => **5** HP) and `:4102` (same four => **0**
damage); none is 594/686. **Zero provider wiring needed on this lane.**
c8's own retire receipt (`7c2de9c06` s7a) already pins c9 at `2042d77f1`
("Snail/GlowingSnail/MagmaSnail :3681/:4048" — the pre-wave line
numbers); the rows are unchanged in content at the current tip (now
`:3732`/`:4102` after the gold appends). The five gold critters
(5 HP / 0 dmg, `:3735`/`:4105`) ride the same gate. The scan-level
sight provider (`set_was_seen_directly`) is c8's wiring — this lane
carries no sight code (grep: only the saveglue round-trip fixture).

## 4. Seal item 4 — 655 MagmaSnail find_frame: f26-7j DELEGATES, one machine

f26's extra-7j (`origin/task/W4-f26-findframe-extra-3` tip `b89812bf7`,
`npc_findframe_extra7j.rs`) ports 650-655 and its MagmaSnail arm
**DELEGATES to `npc_ai_style67::snail_find_frame`** — the module header:
"655's machine is ALREADY PORTED on this branch —
`npc_ai_style67.rs::snail_find_frame` (`:414`, its case-359/360/655 doc
at `:404`) is the verbatim twin; this module's MagmaSnail arm DELEGATES,
one machine no second copy (the d44 wave-C precedent)". The 180-combo
delegation-equality grid witness asserts
`(s.row, s.counter) == snail_find_frame(vx, vy, row, counter)`.
On THIS machine the same twin is the live path: `npc.rs:9452`'s
Snail | GlowingSnail | MagmaSnail find_frame arm calls
`crate::npc_ai_style67::snail_find_frame` (defined at
`npc_ai_style67.rs:424` — f26's `:414` cite is its own branch's line
numbering, same function). **Compose verdict: keep-one, zero semantic
delta** — the merger keeps c9's wired kind-match arm; f26's cs_id-hooked
seam entry for 655 folds onto it (the d44 wave-C composition precedent).

## 5. The snail-family bible (the merger's one-paragraph contract)

**Kinds + ids.** Snail 359 / GlowingSnail 360 / MagmaSnail 655 ride
aiStyle 67 (`NPC.cs:34348-34622` inline arm — NOT projectile aiStyle 67,
the pirates); Shimmerfly 677 rides the kind-gated aiStyle 64 arm. All
four ids are vanilla-free protocol slots (cs_id == proto id == bestiary
net_id); d37/g8/d42 squat 359/360 in their port-local 354-399 corridor
(109 §3). d37's FINAL adjudication (`a341daa12`, research/108 R0-R9 +
E8) settled it: **d37 KEEPS 354-399, c9 shifts its 2 arms** (the E1
HARD row, the E3 740/741 shift) — c9's "renumber breaks f47 keys"
objection was disproven by d37's code read (`resolve_npc_shimmer` keys
`kind.net_id()` = the bestiary ENTRIES net_id, NOT `to_u16`; catch/
light/immunity key cs_id), so the shift is witness-safe. **This lane
has NOT executed the shift** (d37's `fbb948172` lists c9 among the
laggards, zero movement) — it is a merge-wave obligation: apply the E8
per-lane renumber table to the 2 `to_u16`/`from_u16` arm pairs, keep
every cs_id/net_id key untouched, re-pin the two proto-id asserts in
the w4_c9 axis witness. 655/677 have zero squatters across all pushed
tips and never move.
c7's identical Shimmerfly definition (`c2cc6ec61`) dedupes to ONE
variant at the 108 merge: keep c7's grouped firefly stat arms + c9's
shimmer/transform wiring; never double-arm `to_u16` (the
`deny(unreachable_patterns)` guard).

**Behaviour.** The `snail_ai_step` machine: the shimmer probe
(`:34350-34360`, the inclusive maxTiles clamp quirk, cast-after-divide
tile probe, liquid>30) → Transform(677) or the statue despawn; the
359-only `Next(80,111)` scale roll + integer-half -2f resize; the
360/655 cast-before-divide lights (360 blue 0.1/0.2/0.7, 655 orange
0.6/0.3/0.1 at the truncated centre tile — draw-side
`npc_lights::snail_light`, the machine's light event discarded, one
feed); the 0.3/0.6 speed split; TargetClosest-once; the `Next(7200)`
fall roll + 6-tick airborne counter; the fall probes + flag70 wall
latch + stuck detector; the crawl state machine; the quadrant rotation
+ 0.1 ease with the >3.14 wrap. Two Main.rand sites, witness-replayed.
The adapter tail: `snail_no_gravity` carrier (init true), the dry
gravity row (base 0.3 depth-scaled cap 10 — aiStyle 67 never wets,
`:93055`), the ±0.005 deadband, `collision::walk_down_slope`
(`Collision.cs:1686-1789`), fall-through false, identity collision
params, TileCollision(fall:false,fall2:false), MoveWhileDry,
`snail_move_on_slopes` (`:92704-92721`, the `direction *= -direction`
quirk). Transform drain: the style75 one-tick fold (documented ORDER
FOLD in ANALYSIS); the statue arm: timer-0 + `wants_spawn_no_cycle` +
damage-override 0. FindFrame: the fall rows 4/5 (vy>1, the 11 reset) vs
the walk rows 0/1/2/3/2/1 (the 59 reset), zero velocity freezes both;
677's double-increment counter with the pixel-not-row wrap quirk
(documented fold).

**Immunity + shimmer compose.** `shimmer_immune` (the 131-id
`NPCID.cs:10467` set, cs_id-keyed) + the `NPC.cs:17838` unconditional
`buffImmune[353]` write: Shimmerfly 677 IS a member (never re-shimmers;
the f47 ramp reads the flag); the snail trio is NOT (their aiStyle-67
probe owns the transform). f47's wet-contact pass exempts aiStyle 67
(the `:93055` list) — the two shimmer lanes compose without
double-transform. Buff 353 band claims: d37/g8/d42 squat 359/360 (the
protocol ids, not the buff), 655/677 free (109 residual note).

**Catch + release.** catchItem 2006/2007/4849/5350 (NOT the 2121
frog/duck range — 109 §4a): `bug_net.rs` rows verbatim, `npc_catchable`
explicit/range arms, the swing pass keys `cs_id`, the 4849 lava-bait
gate, release via the generated itemdata `make_npc` rows → bestiary
net_id → kind. The cage furniture is f14's lane.

**Bestiary.** All nine c9 kinds (the four + the five gold) register via
`AddEmptyEntries_CrittersAndEnemies_Automated`'s `CountsAsACritter` set;
the c8 GENERIC sight gate auto-covers every one (5 HP / 0 dmg) with
zero per-lane provider wiring (§3). The gold peer table (13 rows) +
the 14-member collection are c8's (`research/238`); the 443/445 rows
are pre-tabled there.

**Sheets.** NPC_359/360/655 six rows, NPC_677 four, NPC_442 five,
NPC_443 seven, NPC_445 thirteen, NPC_446 two, NPC_447 six
(`Main.cs:67118-67128`, `NPC_FRAME_COUNT` scripted-equal). The eager
STATIC_SHEET_NAMES pack is the documented superset until the 113
s10.6 per-world NPC page lands (the c8 b7fd55a43 lazy-load contract).

**Merge order (this lane's obligations).** (1) Shimmerfly dedupe with
c7 (keep-one variant, c9 wiring survives); (2) the E8 renumber-INSTRUCTION
for c9: shift the 359/360 `to_u16`/`from_u16` arm pairs per d37's FINAL
table (the E3 740/741 shift), cs_id/net_id keys untouched, the w4_c9
axis witness re-pinned (§5) — merge-wave work, receipted not executed; (3) f26-7j's 655 seam folds onto the wired kind-match arm
(§4); (4) d37's ALL_KINDS hunk drops as already-satisfied (§2); (5) the
c8 sight gate needs nothing from this lane (§3); (6) the f47 ramp
composes via the 353 flag + the aiStyle-67 wet exemption; (7) the FIVE
gold-critter dup drops to c10 per the exact §6 hunks (the d37-LV +3
extended by wave 2's 443/445) — rows stay on-branch per the E8
dropped-lane rule, the drops ride the merge wave; the critter module
keep-one reconciles the walk-exit draw order (§6b).

## 6. The gold-critter DUP drop-hunks (c9 → c10, the d37-LV +3 extended to five)

d37's laggard-verify (`2ea22ce3e`, research/108 LV) ruled: c9's gold
critters at 442/446/447 **duplicate c10's pushed rows** (`5fcc68ae6`'s
tree, the `d7f422049` twelve-kind roster tail) — "kinds-lane rule: c10
keeps, c9 drops; c9's wiring composes". Wave 2 grew the dup set to FIVE
(443/445 are in c10's tail too, same commit). Per the E8 dropped-lane
rule (the d42 `bac408be0` precedent) **the rows STAY on this branch**;
the exact drop-hunks below are EXECUTE-prep for the merge wave. Line
numbers are 1-indexed inclusive at tip `9252d6f31` (the FINAL seal tree;
the rename commit changes docs only).

**Keeper verification (c10 tip `15f93d134`, scripted git-show reads):**
all five variants + `to_u16`/`from_u16` arms + ALL_KINDS rows (decl 290),
cs_id/sound(NPCHit1; GoldMouse NPCDeath4)/ai_style(24/7/7/1/7)/size/
lifeMax/damage/net_id tables, bestiary ENTRIES ×5, `npc_buffs` &[31]
(the `NPCID.cs:8612-8652` rows), NPC_442-447 sheets +
BESTIARY_NPC_SHEET_IDS (decl 201) + the Lookups vec, portrait_data arms,
the draw + rotation-whitelist arms, the netglue seed (the twelve-critter
bare-stats arm), FindFrame arms ×5 (`npc.rs:9782-9946` region). The C#
stat rows are identical on both lanes (both re-read
`NPC.cs:14247-14331`).

### 6a. DROP whole (c10's row survives)

| File | Lines | Content |
|---|---|---|
| `npc.rs` | 1115–1149 | the five enum variants + doc comments |
| `npc.rs` | 3619–3623 | the kb rows (0.8 bird / 1.0 others) |
| `npc.rs` | 4007–4009 | the defense-0 comment + row |
| `npc.rs` | 4385–4387 | the value-0 comment + row |
| `npc.rs` | 19275–19279 | the `to_u16` five arms |
| `npc.rs` | 24644–24650 | the ALL_KINDS five rows + comments; decl :24358 re-pins to the merge union (NOT 279−5 — every lane's decl reconciles per d37-S4) |
| `npc/data.rs` | 602–606 | the cs_id five arms |
| `npc/data.rs` | 945–949 | the from_u16 five arms |
| `npc/data.rs` | 1827–1829 | the NPCHit1 comment + row |
| `npc/data.rs` | 2139–2143 | the death-sound comments + rows (NPC_Killed_1 ×4 + GoldMouse NPC_Killed_4) |
| `npc/data.rs` | 2514–2521 | the ai_style comments + rows |
| `npc/data.rs` | 3061–3067 | the size rows + comment |
| `npc/data.rs` | 3186–3190 | the net_id five arms |
| `npc/data.rs` | 3733–3735 | the lifeMax-5 comment + row |
| `npc/data.rs` | 4103–4105 | the damage-0 comment + row |
| `npc_buffs.rs` | 491–494 | the Confused &[31] comment + row |
| `bestiary.rs` | 2211–2255 | the five ENTRIES rows + comment |
| `assets.rs` | 1285–1292 | the STATIC_SHEET_NAMES comment + NPC_442/446/447/443/445 |
| `assets.rs` | 1916–1918 | the SHEET_IDS comment + `442, 446, 447,` + `443, 445,`; decl :1878 190→185 (c10's own grid supersedes wholesale at merge) |
| `assets.rs` | 2687–2695 | the Lookups vec comments + five `atlas.sheet` entries |
| `render/entities.rs` | 1258–1262 | the GOLD_CRITTER_SHEETS doc + const |
| `render/entities.rs` | 519 | the `.chain(GOLD_CRITTER_SHEETS)` line (c10's own sheet chain survives) |
| `render/entities.rs` | 3365–3368 | the GoldBird rotation-whitelist comment + member (the snail members :3358–3364 STAY) |
| `ui/bestiary.rs` | 1605–1612 | the portrait comment + five arms |
| `app/netglue.rs` | 3585–3598 | the gold seed comment + the grasshopper/mouse, bunny/frog and bird arms (c10's twelve-critter bare-stats arm seeds them; the position overwrite + first-tick machine takeover make the ctor choice behaviour-identical) |

### 6b. DROP the arm, KEEP the machine re-keyed (the wiring-composes half)

- `npc.rs` 6755–6766 (the dispatch arms) + the `update_gold_bird` /
  `update_gold_grasshopper` wrappers (`:5973`/`:6050` regions): DROP as
  dispatch — c10 routes GoldBird through its aiStyle-24 axis
  (`ai_024_bird`, the shared poop gate) and GoldGrasshopper through
  `update_slime`. c9's wrappers are faithful re-ports of the same C#
  machines; keep-one per the d20b/1080 precedent, no double dispatch.
- `npc.rs` 9478–9640 (the five FindFrame kind-match arms): c10
  has its own five arms (`npc.rs:9782-9946` at its tip, same C# cases
  `:73627-73650`/`:76851-76904`/`:76618-76685`/`:70959-70980`/
  `:76514-76565`). Keep ONE set at merge — either is verbatim; the
  merger keeps c10's (the kinds-lane owner) and drops c9's.
- `npc_ai_critter.rs` (the whole module): c10 carries the d38-derived
  module with the SAME gold rows (`row()`: bunny 1/0.07/true, frog
  1/0.07/true, mouse 2/1/false — identical constants). Keep c10's
  module (it also serves 299/300/303/337). **One semantic delta the
  merge wave must reconcile:** c9's walk→idle draw is the TownCritter
  `300+Next(300)−Next(100)` (`NPC.cs:53817-53831` — re-read verbatim
  this round, the `:53824` subtraction); c10's module header cites
  `:54289-54296` (`60+Next(60)`, the sit/talk-state exit — a DIFFERENT
  arm of AI_007). For the wander state 1→0 the C# is `:53817-53831`;
  c9's draw order is the faithful one — the merged module must keep
  the `:53820`/`:53824` draws for the gold/critter walk exit (the
  c9 witnesses `w4_c9_the_gold_critter_walk_to_idle_subtracts` pin it).
- The w4_c9 gold witnesses (`npc.rs:34531-34671` region,
  `npc_ai_critter.rs` tests): keep as the merge's regression pins,
  re-keyed onto the surviving c10 module (the assertions are
  C#-literal, lane-agnostic).

**Net effect:** c9 keeps the snail four + ALL their wiring; the five
gold kinds resolve to c10's rows; ONE critter module (c10's body, c9's
walk-exit draw order); one FindFrame set; the bestiary/sheet/portrait
copies dedupe to c10's. No behaviour change on any seam (all stats
C#-identical, verified both tips).

## 7. Witness register (this seal round)

- The 2×90s alive-probe (§0) — scripted diff-hash equality.
- The d37 patch scratch-index apply + post-apply census (§2) — exit 0
  twice, blob `69cb3559d`, decl/rows/pin 274.
- The 279-roster scripted census (§1) — enum == array == from_u16,
  sets identical, zero dups; ENTRIES 258; SHEET_IDS 190==190.
- The C# verbatim re-reads: `NPC.cs:76851-76904` (the bunny group) and
  `:76618-76685` (the frog group), ≤80-line windows.
- The extended frame/sheets witnesses (§1) — WRITTEN NOT RUN
  (code-only mode).
- The §6 drop-hunk round (the successor agent): the c10-tip keeper
  verification (git-show reads of all five kinds across
  npc.rs/data.rs/bestiary/assets/netglue/render/ui/npc_buffs at
  `15f93d134`), the 28-site line-pin table read from the `9252d6f31`
  tree, the `NPC.cs:53817-53831` vs `:54289-54296` walk-exit re-read
  (the §6b draw-order delta), the registry §7 round-3 re-read + the
  fresh all-ref 281+ sweep (281×10 / 282×3 taken, the 304 floor also
  contested ×2, 312 free at every ref) — the 281→312 rename receipt in
  the header.
