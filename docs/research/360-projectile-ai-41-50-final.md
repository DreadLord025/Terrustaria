# 360 — Projectile aiStyles 41-50 FINAL + SEALED (W4-d5 RETIRE)

Status: FINAL / SEALED (W4-d5, 2026-11-21, worktree TR-W4-d5, branch
`task/W4-d5-proj-ai-41-50`). Code-only — no cargo/rustc/tests/rustfmt/
graphify invoked; every witness WRITTEN NOT RUN. C# cites are
`decompiled/Terraria/Projectile.cs` (stable), read in ≤100-line windows.

Research number **360** per registry-150 (`origin/task/W4-d54-dust-sites-world`
`docs/research/150-research-number-registry.md`, FROZEN §8.7 at tip
`9dddb32b4`/`06aea22e0`): new docs take ≥360 after a registry check — a
fresh `git ls-tree docs/research/` sweep of all origin refs finds NOTHING
≥350 filed (highest live keys 308/309/312, all §8-assigned); **360 is the
first new-doc key**. The census doc's own renumber 26d→**156** is EXECUTED
on this branch (registry §3 row: d12's 26d pushed 00:06:06 first, d5's
00:24:13 loses the key; refs-to-update "(none)" per the registry — the
five module headers, `main.rs:198` and the five anchors rows re-pointed
anyway, they cite this lane's own doc).

## 1. The landing census — what the band-41-50 git log carries

Six commits beyond main (`a725acb04` base → tip `e46d9eb7f` + this round):

| commit | scope | state |
| ------ | ----- | ----- |
| `503b08679` | the census: 41/42/43/46/49 MISSING (no arms, no rows), 44/45/47/48/50 ported verify-only, family gaps F1-F6 (research/156, was 26d) | docs |
| `fc17b6959` | `projectile_ai_style41.rs` — FlowerPetal 221 (`:27784-27792` arm + `:3066-3078` consts + the `:3076` spawn scale roll), 4 witnesses | UNWIRED |
| `615ac106a` | `projectile_ai_style42.rs` — CrystalLeaf 226 (`:27793-27859` orbit/scan/interception/227-spawn), `CrystalLeafOwner` snapshot, RNG-free, 7 witnesses | UNWIRED |
| `6004751f9` | `projectile_ai_style43.rs` — CrystalLeafShot 227 (`:27860-27892` + `:74524-74533` Kill), the 8×NextFloat kept-draws census (:106 at that tip), 5 witnesses | UNWIRED — **the e3 activation target, §2** |
| `96a18c21c` | `projectile_ai_style46.rs` — RainbowFront/Back 250/251 (`:28107-28199`), RNG-free, 6 witnesses | UNWIRED |
| `5f7390a30` | `projectile_ai_style49.rs` — ExplosiveBunny 281 (`:28243-28322` AI + `:18833-18860` bounce + `:75749-75803` Kill, the 20-draw gore census), F2 recorded (the `ai_style2` 162\|281 ladder mirrors the DEAD `AI_002` 281 branch `:23321`), 7 witnesses | UNWIRED |
| `e46d9eb7f` | anchors: one row per landed unwired machine (5 rows, :899-903) | canary |

Every machine is a PURE module keyed on the vanilla type with the
`SetDefaults` rows as module consts — NOT `proj_data` rows (adding rows
would arm the dispatch fall-through for kinds whose wiring is a later
round; the `npc_ai_style25` precedent, research/156 §"The missing styles").

## 2. The e3d-AI-inline-34-STAGED activation — style43 EXECUTED here

The e3 activation contract (research/231 §7 row d5, at e3 tip `d7bffece4`;
the RETIRE receipt `0b48616fa` re-verified this lane at `e46d9eb7f`:
"still-UNWIRED 8-NextFloat census :106, `:27873` 8×NextFloat"): the
`style43_crystal_leaf_burst_227` emitter (`projectile_dust.rs:1707` at the
e3 tip) REPLACES the machine's 8 kept call-site `NextFloat` draws — the
g63 body-skip supersede convention. **This round executes that REPLACE on
d5's own branch** (the contract's "d5's own wiring step" half that does
not need the row):

- `projectile_ai_style43.rs::crystal_leaf_burst_227` — the emitter body
  verbatim (merger keep-one with e3's copy; signature widened from e3's
  `ProjDust` bundle to `(p, world, dust, rng, ctx)` — the bundle's three
  fields, so the merge keeps either wrapper). Per dust: `DustPool::
  new_dust` internals FIRST, then `noGravity`, `velocity *= 3`,
  `scale = 1.5`, then the POST-spawn `velocity += base.velocity *
  NextFloat()` (`:27870-27873`). Sentinel behaviour is the pool's.
- `leaf_shot_step` widens to `(p, world, dust, rng, ctx, local_owner)`
  and calls the emitter inside the `soundDelay == 0` latch; the former
  bare 8-float loop is GONE (superseded, not doubled).
- `leaf_shot_kill` drops its 15-float census: the 15× dust-157 bodies +
  their POST-spawn blends (`:74529-74532`) are g65's death-drain 227 arm
  (`projectile_death_dust.rs:922` @ `b0842907c`, cites `:74524-74534`) —
  the g65-Kill-309 compose (§4). The hook keeps the draw-free id-6 cue.
- The `LeafDustRng` adapter mirrors e3's `ProjDustRng`
  (`projectile_dust.rs:47-59`) — merger keep-one.
- Witnesses re-derived: the first-tick test now pins 8 live dust-157
  bodies (noGravity/scale 1.5) + a NEW draw-mirror test replaying the
  pool internals + the post-float on a cloned `UnifiedRandom` (stream
  re-alignment pinned); the Kill test pins ZERO draws. The rotation,
  frame-latch and hue tests carry over with the widened signature.

The other 33 STAGED emitters belong to their own lanes' merges (d6/d9/
d10/d12/d13/d14 — research/231 §7's table); NONE targets this band's
other four modules (41/42/46 are RNG-kept or RNG-free machines; 49's
dust is the g63/g65 lanes', §4).

## 3. The all-styles-armed / kinds-rowed census (this tip + pushed lanes)

Dispatch arms (`projectile.rs` at this tip):

| style | arm | verdict |
| ----- | --- | ------- |
| 41/42/43/46/49 | **none** — the five modules are UNWIRED (`#![allow(dead_code)]`) | machine EXISTS, row+dispatch = the wiring step |
| 44 | `ai_spear_subshot_44` via dispatch :17226-17228 | ARMED |
| 45 | `ai_rain_cloud_moving`/`nimbus_step`/`ai_rain_drop` :17786-17789 | ARMED |
| 47 | `magnet_sphere_step` :17773 | ARMED |
| 48 | `ai_magnet_bolt` :17775 | ARMED |
| 50 | `ai_inferno_fork` :17459 | ARMED |

`proj_data` rows (`projectile/data.rs` at this tip):

| kind(s) | style | rowed here? | row lane |
| ------- | ----- | ----------- | -------- |
| 221 / 226 / 227 / 250 / 251 / 281 | 41/42/43/46/46/49 | **NO** (module consts only; 281 has the `CANNON_BUNNY` const :710, no row) | d5's own wiring step — UNCLAIMED on every pushed tip (the g63 281-correction `eea7156fe` all-branch sweep) |
| 228 / 732 | 44 | YES (:3897/:3900) | main |
| 229 | 44 | no | **g2** (`ee857a36`, F6: the orb branch + Kill arm `daea150f`+) |
| 237 / 238 / 239 | 45 | YES (:2711/:2715/:2719) | main |
| 243 / 244 / 245 / 264 | 45 | no | **g2** (F5: the blood chain + the live shimmer bounce) |
| 254 | 47 | YES (:2702) | main |
| 255 / 260 / 294 | 48 | YES (:2705/:3803/:2725) | main |
| 290 / 433 | 48 | no | **g2** (`af472214`, F1: the 290 delay-3 arm + cue) |
| 295 / 296 | 50 | YES (:2738/:2741) | main |
| 291 / 292 | 50 | no | **g2** (F4: the hostile arrival branch + the +1 fade) |

**Remaining gaps after the g2 merge** (the rowless-nine of g2's
`26d-projectile-rowless-kinds-w4-g2.md`, registry-assigned 163, rename
owed on g2's branch): NONE of F1/F4/F5/F6 stays
open — g2's pushed tip `5900648e0` carries all nine rows + arms + the
reconciliation round (the sibling class flags, the −20 collision shrink,
the STAGED `Projectile::shimmer`, the 1128 child owned by d10). The ONLY
unclaimed residue in the band is **d5's own six rows + five dispatch
arms + the player-side `crystalLeaf`/`crystalLeafCooldown` fields** (buff
60's `PetFlag::CrystalLeaf` already lands in `buff_effects_misc.rs:604`;
the 226 spawn producer is `crystal_leaf_step`'s request). The wiring
follow-ups recorded at port time stand: the id-6 sound-drain façade
(`world_sim/projectiles.rs:460-473` plays ids 0/2 only), the F3 magnet
5-frame visual skip, the F2 dead-twin ladder (faithful, kept).

## 4. The g65-Kill-309 compose (281 + 227)

The owed pool `g65-Kill-309` =
`Kill-remainder-309-distinct-kinds-ALL-406-Dust.NewDust-sites-ported-or-
accounted` (g65 FINAL research/346 §1, tip `37ec2f3aa`; the "309" was the
batch-28 drain census, corrected to 316 kinds / 241 arms at the final
tip, 313 post-merge after the d6 gun-splash drop). Its 281 and 227 arms
overlap this lane's Kill hooks — composed here, C# re-read whole
(`:75749-75803`, `:74524-74533`, `:28288-28314`):

| block | d5 keeps | the body lane | merge instruction |
| ----- | -------- | ------------- | ----------------- |
| 227 Kill (`:74524-74533`) | the id-6 cue intent (`leaf_shot_kill`, now draw-free) | **g65** drain 227 (`death_dust.rs:922`: 15× dust-157, noGravity + `old_vel * NextFloat` POST-spawn + scale 1.5) | d5's former 15-float census DROPPED this round (superseded, not doubled); the g65 drain is the single roller |
| 281 Kill (`:75749-75803`) | sound 4 + Item14 cues, the 20-draw gore census (76/77 pairs + kicks, 4× Next(61,64) + kicks), the damage>0 128×128 resize + hostile flip + Damage intent (`bunny_kill`) | **g65** drain 281 (`death_dust.rs:3295`: the SAME 20 gore draws kept interleaved + 20× dust-31 vel×1.4 + 10 pairs dust-6; resize/Damage "unported state" there); **g16** gore bodies (`projectile_gore_j.rs:99/:120` `tombstone_head_gores`/`corner_gores`); **d77** carries NO 281 cue (verified: zero 281 hits in its `kill_tail_cue` map) | the gore census exists on BOTH d5 and g65 — at the merge it drops from ONE side only: prefer dropping g65's census loops (d5's hook is the state host and runs at the death hook; g65 keeps its dust bodies, its gore draws go when g16's bodies activate). New witness `w4_d5_s49_the_g65_compose_census_is_stream_identical` pins the two censuses are draw-for-draw identical (20 draws, −3 inert) |
| 281 tick-3 fuse spark (`:28288-28314`) | the ONE gore `Next(61,64)` + 2 kicks (`bunny_step`'s ai0==3 block) | **g63** STAGED `bunny_burst` (`projectile_trail_dust.rs:818`: 10× dust-31 + 5 pairs dust-6 + the gore ARGUMENT draw; its doc cites THIS module `5f7390a30` as the activating machine); **g16** `tombstone_landing_gore` (wave B site 14, `:28308`) | at the row+dispatch wiring: g63's emitter ADDS the pool internals (the dust loops roll no explicit draws here); its kept `Next(61,64)` argument REPLACES d5's gore block when g16's body lands (the emitter's own activation note) — until then d5's 3-draw census is the single roller |

Stream-order note (the gallop-dust deviation, g65's module doc): the g65
drain runs at the pass tail, d5's hook at the death site — the gore
census and the dust internals interleave in C# but split across the two
seams in the port; the merge's one-side drop keeps the TOTAL draw count
exact (the recorded queue→drain stream-neutral convention).

## 5. The band-41-50 declaration

**The W4-d5 projectile aiStyles 41-50 family is DONE (code-only).** All
ten styles carry a faithful machine: five ported UNWIRED here (41/42/43/
46/49 — pure modules, C#-verbatim, witnessed), five verified ARMED on
main (44/45/47/48/50). The style-43 STAGED emitter is ACTIVATED (the
e3 REPLACE contract executed on this branch); the 227/281 Kill dust is
composed with the g65-Kill-309 pool (census single-roller instructions
published); the nine rowless verify-only kinds are g2's pushed lane
(F1-F6 all closed there). The residue is exactly ONE wiring step — d5's
six rows + five dispatch arms + the crystalLeaf player fields — plus the
recorded follow-ups (the id-6 drain façade, the F3 visual frame machine,
the g63/g16 281 activation). No further port rides this branch; later
tip movement is recorded as receipts, not re-openings. This doc is
SEALED.

Witnesses: 4 (style41) + 7 (style42) + 6 (style43, re-derived: +1
draw-mirror, the first-tick/Kill tests re-pinned) + 6 (style46) + 8
(style49, +1 g65-compose) = **31 WRITTEN NOT RUN**; verified by C#
re-read in ≤100-line windows and hand arithmetic on cloned
`UnifiedRandom` streams.

Gate command (post-merge, NOT run here): `cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids` — judge by the `test result: ok`
lines.
