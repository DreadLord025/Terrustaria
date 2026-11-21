# 319 — W4-d125 RETIRE FINAL: the empress-family declaration + the 636/661 dedupe + the BossNoCheese compose

Rename chain: 282 → **319** (the frozen research/150 registry on
`origin/task/W4-d54-dust-sites-world`, `a15cb634b` — §8.2: key 282's
winner is §7.2's owner f26; the losers g48/d125/f39-seal → 318/**319**/320).
The companion sheets ledger renumbers 125 → 281 → **309** (§8.1 row 2):
`309-empress-sheets-ledger.md`. The §5 record below preserves the
RETIRE-time 281/282 self-picks as history; the numbers NOW live as 309/319.

Status: ✅ FINAL (code-only mode — no cargo/tests/rustfmt/graphify run; every
witness WRITTEN NOT RUN). Branch: `task/W4-d125-empress-r7-drain`, worktree
TR-W4-d125. Number per the research/150 collision registry
(`origin/task/W4-d54-dust-sites-world`, `38bfd1d22`): the free block 151–236
is consumed; a fresh all-tip `ls-tree` scan at RETIRE time found 237–246,
262, 264, 266, 269, 271, 273, 274, 279, 280 taken by later lanes (244 alone
is four-way: d53/f39/g48/g86). The RETIRE-time self-picks were **281** (the
sheets ledger, renumbered from 125 — key 125's verdict was later CORRECTED
by 150 §7.1: d125's 06:07:29 doc WINS 125, d38's crossnote revoked → 293)
and **282** (this doc), then verified free across all 355+ remote tips;
the frozen §8 registry (`a15cb634b`) renumbered both — 281 = f26 →
ledger **309** (§8.1), 282 = f26 → this FINAL **319** (§8.2).

## 0. Alive-status + the d37 REGRESSED flag — RESOLVED

- Session liveness at RETIRE entry: HEAD `7a0482302` (12:22:47), newest
  tracked-file mtime 12:22:25, clean tree; two probes 95 s apart (13:00 →
  13:06) showed zero movement, >40 min idle → **NOT ALIVE**, RETIRE
  authorised. A foreign uncommitted hunk (`world_sim/editing.rs`, the
  Terraprisma use-census witness, mtime 13:09:06) appeared mid-RETIRE; two
  further probes 95 s apart (14:56 → 14:58) showed it frozen (no advance,
  no new files, HEAD unchanged) → treated as the prior session's last
  write, preserved verbatim in its own commit (the lane's own file, the
  lane's own witness; NOT reverted, NOT attributed to this phase).
- **d37-roster-REGRESSED-flag RESOLVED**: research/108's S0 row ("origin
  REGRESSED to the phase-5-docs tip `d8047a109`; sheets/draw/hitbox
  LOCAL-ONLY, push before merge") is stale. At RETIRE entry
  `git rev-list --left-right --count origin/…...HEAD` = `0 4`, then after
  `git fetch origin` = **`0 0`** — origin's tip IS `7a0482302`; the sheets
  (`2dfe026d2`), draw (`cd2b1b1e3`), hitbox (`86e3adfd3`) and BossNoCheese
  (`7a0482302`) commits are ALL PUSHED. No push-rescue was needed. The
  merger reads origin-now-`7a0482302`-not-`d8047a109`.

## 1. The 108 R4/E8 work order — EXECUTED on this branch

| Kind | old to_u16/from_u16 | new id | cs_id | evidence |
|---|---|---|---|---|
| EmpressOfLight | 229 | **636** | 636 | protocol==cs RESTORED (R4: 229 is Pirate's reserved hole, d37-owned) |
| PrismaticLacewing | 237 | **742** | 661 | E3/E8: 237 is JungleCreeperWall's hole; 742 keeps the small-lane block contiguous |

- `npc/data.rs` both arms moved (the `#[deny(unreachable_patterns)]`
  bijection guard holds — 636/742 were free in this branch's protocol
  space, verified by grep before the edit); the roster witness
  `the_empress_roster_row` (world_sim/npcs.rs) updated to 636/742.
- **ALL_KINDS decl 270→272 fixed** (the R0 stale-pin: the array already
  listed 272 arms — machine-counted at RETIRE time — the declaration was
  the compile-time mismatch).
- Everything net-keyed stays: ENTRIES net_id 636/661, sheets
  NPC_636/NPC_661, `kind_from_net_id`, the bestiary credit ids
  HallowBoss/EmpressButterfly, the .wld/.trwld downed flag — none reads
  the protocol id.

## 2. The 636/661 DEDUPE — keep-one d125 vs d43 (for the d37d-E8 merge)

d43 batch 1 (`510abc9dd`, kinds 631/632/634/635/636) and batch 6
(`ee665ab63`, 657–661) ported 636 `HallowBoss` + 661 `EmpressButterfly`
as **data-only stand-ins**: `NpcKind::HallowBoss => {}` /
`NpcKind::EmpressButterfly => {}` AI arms ("holds still until its machine
lands", the MartianTurret precedent), protocol==cs 636/661, verbatim
SetDefaults rows, bestiary/sheet-name/find_frame/stat-witness rows.

**Verdict: KEEP d125's `EmpressOfLight`/`PrismaticLacewing` for BOTH
kinds; DROP d43's `HallowBoss`/`EmpressButterfly` variants + their arms at
merge.** d125 is the strict superset: the full AI_120 machine wired
(`ai_120_empress`), the AI_065 661 branch whole (`npc_ai_lacewing.rs`),
the release/death→SpawnBoss chain, the day-enrage one-shot, the packed
sheets + the dedicated draw + the hitboxes + the BossNoCheese group.
d43's rows it does NOT supersede (fold-list for the merger):

| d43 hunk | disposition |
|---|---|
| `NpcKind::HallowBoss`/`EmpressButterfly` variants + to_u16/from_u16 636/661 arms | **DROP** — d125's kinds now own protocol 636 (R4 executed); 661 decodes via d125's ENTRIES net row (proto 742). Two variants for one cs id is the bijection gate's hard error. |
| data.rs stat rows (size/hp/dmg/def/kb/value/ai_style/sounds/cs_id/no_gravity 661) | **DROP** — byte-equivalent to d125's rows (both cite NPC.cs:17188-17206/:17351-17366; verified field-by-field at RETIRE). |
| ctor `boss:`/`dont_take_damage:` HallowBoss arms | **DROP** — d125's EmpressOfLight arms carry the same flags. |
| find_frame 636 (ai[3] 1\|3 row pin) + 661 (7-tick bands) | **DROP** — d125's arms are the same port (661's adds the 27-wrap, the C# :72771 tail). |
| bestiary ENTRIES HallowBoss/EmpressButterfly + ui/bestiary portrait arms | **DROP** — d125's entries identical (net_id/credit/quick_unlock); d125's portraits ride the REAL sheet dims (the d35 shape), d43's divide by frame-count on an unpacked name. |
| assets.rs STATIC_SHEET_NAMES/BESTIARY_NPC_SHEET_IDS/Lookups NPC_636/NPC_661 rows | **DROP the 636/661 entries only** — d125 PACKS the sheets (D125_SHEETS, slots 181→183); d43 only names them (no pack — its `pumpkin_moon_draw(assets.npc_sheet(636),…)` reads a sheet that does not exist in its atlas). d43's OTHER kinds' rows (631/632/634/635/657-660/…) stay. |
| audio.rs `("NPC_Killed_65", 4)` | **FOLDED INTO d125 at RETIRE** (this branch's SFX_CUES lacked it — the death_sound arm referenced an unregistered cue; d43's row kept verbatim). `NPC_Hit_41`/`NPC_Killed_43` are d43-unique (RockGolem), stay on d43. |
| netglue hover seed HallowBoss | **DROP** — d125's arm covers EmpressOfLight + PrismaticLacewing with the same spawn_hover shape. |
| npc_buffs `HallowBoss => &[31]` | **DROP** — d125's combined 636\|661 Confused-immunity row (NPCID.cs:9749/:9923) is the superset (d43's 661 row is absent — its batch-6 forgot the immunity; d125 has both). |
| d43 stat-witness rows (636/661 tuples) + ALL_KINDS entries | **DROP with the variants**; d43's ALL_KINDS 333 recount loses 2 → 331 at merge (the merge-wave 550-union reconcile owns the final number). |

Protocol-space note for the merger: after the drop, cs 661 has NO direct
protocol arm on the union (the Lacewing rides 742 per E8; `kind_from_net_id(661)`
resolves it for net/save). That is the 108-planned shape, not a gap.

## 3. The BossNoCheese counter COMPOSE — keep-one publish (d125 × d78 × g1)

Three ports of the same C# seams existed:

| piece | d125 (this branch) | d78 (`3fd3cc27f` part 2a) | g1 (LOCAL-ONLY, untracked) |
|---|---|---|---|
| `Player.hurt_cooldowns[6]` + ctor + update_immunity tail | yes (i32 counter param) | yes (Option<usize> param) | no |
| `hurt_inner_cooldown` gate (`:38208`) | identical semantics | identical semantics | — |
| non-General iframe write (`:38391-38394`) | wrote unconditionally | **`== 0` guard (verbatim C#)** | — |
| `set_immune_time_for_all_types` array write (`:38017-38020`) | **MISSING (fidelity bug)** | verbatim loop | — |
| EVP damage chain (`DamageVar −owner.luck` `:14831`, difficulty `:14842`, ×2 `:14844`) | **MISSING on damage_player (raw p.damage)** | full, in `damage_evp_boxed` | raw `hurt_with_source` (26f §1: dropped) |
| consumer sites | NPC contact (ML parts + 636, ai[0] skip, flag4 thorns) + live projectile contact (11 kinds) | Kill-time death-blasts 452/454/448 (`hostile_death_blast`) | Kill-time 452/454/448 (dup, dropped per 26f) |

**Published keep-one (executed on this branch at RETIRE):**
1. **player.rs seam: d125's shape keeps the lane** (the i32
   `COOLDOWN_GENERAL`/`COOLDOWN_BOSS_NO_CHEESE` constants — the C#
   `ImmunityCooldownID` values — vs d78's Option<usize>), **with d78's two
   verbatim C# behaviours composed in**: the `== 0` write guard
   (`Player.cs:38392`) and the `SetImmuneTimeForAllTypes` array loop
   (`:38017-38020` — a cross-dodge DOES arm the BossNoCheese window; the
   pre-compose d125 shape let boss hits land through a shadow-dodge, a
   fidelity bug vs C#). At merge, d78's player.rs seam hunks drop in
   favour of this one; d78's `hurt_inner_cooldown(.., Some(1))` call sites
   re-route to `COOLDOWN_BOSS_NO_CHEESE` (mechanical: `Some(1usize)` →
   `COOLDOWN_BOSS_NO_CHEESE`, `None` → `COOLDOWN_GENERAL`).
2. **EVP damage chain: ONE keeper per path.** d78's `damage_evp_boxed`
   stays the Kill-time death-blast port (452/454/448 through the resized
   box — d125 never claimed the Kill hooks). d125's live-contact
   `damage_player` NOW carries the same verbatim chain (composed at
   RETIRE: DamageVar with the local-owner luck via a new `local_owner`
   param, the expert/classic difficulty sample, the ×2) — the pre-compose
   raw-`p.damage` shape was the same deviation class 26f §1 condemned in
   g1. The two functions share the C# body but walk different boxes at
   different times (live `Colliding` special-case hitboxes vs the
   Kill-resized box); they are NOT duplicates — both keep.
3. **g1 raw-damage deviation: CONFIRMED DROPPED** (26f §1 stands): g1's
   `death_burst_player_hit` (LOCAL-ONLY in TR-W4-g1, verified at RETIRE —
   `projectile_kill_hooks_a.rs` untracked, `projectile.rs` dirty) hurts
   with RAW damage, no DamageVar/difficulty/×2/cooldown lane; its General
   immune gate would refuse the boss blasts the C# `:14799` lands. g1
   keeps only its unique 459/521/522/80.
4. Moon Lord census note (verified at RETIRE): the port's
   `MoonLordHead/Hand/Core/FreeEye/LeechBlob` kinds ARE cs 396/397/398/
   400/401 (`NPCID.cs:11469-11479`; the protocol ids 122-126 are the
   port-local band) — the contact kind-match is the correct membership.

Witnesses (WRITTEN NOT RUN): `w4_d125_retire_the_dodge_arms_every_cooldown_counter`
(player.rs — the array write + the ==0 guard + the per-counter drain),
`w4_d125_retire_the_live_evp_hit_carries_the_damage_chain` (projectile.rs —
the scripted DamageVar roll, classic ×2 = 120 / expert ×2×2 = 240 on a
60-damage aurora). The three BossNoCheese-phase witnesses stay valid
(their assertions are inequality/window-shape, not raw-damage pins).

## 4. The empress-family DECLARATION

**The Empress of Light family is COMPLETE on this branch** (code-only;
the render remainders are sheet-gated, owned by the repack wave):

| member | C# | port | state |
|---|---|---|---|
| 636 HallowBoss (EmpressOfLight) | AI_120 `NPC.cs:45953-46942` + enrage `:45925-45951` + DoMagicEffect `:45832-45923` | `npc_ai_empress.rs` (28 witnesses) + `npc.rs::ai_120_empress` wiring | ✅ WIRED |
| 661 EmpressButterfly (PrismaticLacewing) | AI_065 661 branch `NPC.cs:44896-45110` + release/death→SpawnBoss `:79493-79502` | `npc_ai_lacewing.rs` (8 witnesses) + the sim threat pre-pass + the death drain | ✅ WIRED |
| proj 872/873/874/919/923 | the five AIs (S1-D-4) | `projectile_ai_empress.rs` (9 witnesses) + rows + the owned drain | ✅ WIRED |
| proj 871 SplitShotCore / 924 FairyQueenHymn | `ProjectileID.cs:2296/:2402` | NO rows — 871 spawns only via the 872 split (the pure core requests the five rowed kinds); 924 is the hymn visual (no hostile damage path claimed) | declared remainder, BossNoCheese kind-match already covers both |
| sheets NPC_636/NPC_661 | real .xnb 166×352 / 24×72 | D125_SHEETS packed, slots 181→183 (research/309) | ✅ PACKED |
| draw | Main.cs:26471-26652 + GetAlpha :93501-93505 | `render/empress_draw.rs` (15 witnesses) + `draw_empress` | ✅ WIRED (5 tall strips deferred, research/309 §2) |
| hitboxes 919/923/872 | Colliding :15625-15694 | damage_player's REPLACE arms + trail_extra 120-ring | ✅ WIRED |
| BossNoCheese group | Player.cs:31332-31349 + Projectile.cs:14780-14797 | the counter seam + both consumer sites (§3) | ✅ WIRED + COMPOSED |
| Terraprisma (5005) use → 946 blade | Item.cs:40278-40296 + Player.cs:53268/:48556/:10912 | the use-census witness (the prior session's frozen hunk, preserved) | witness WRITTEN NOT RUN; the 946 AI_156 machine is the a9 co-lane's claim |
| downedEmpressOfLight | .wld tail 2024 | writer/reader/.trwld v18 + fixtures | ✅ |

Family remainders (NOT this lane's): the arm/wing overlay strips + ghost
rings (sheet-gated, research/113 repack wave), the 872/873 trail-ring
DRAW + GetAlpha tints (render side), the 946 AI_156 machine (a9), the
871/924 rows if a future lane claims the split-core/hymn visuals.

## 5. Research-number registry compliance (per 150)

- The sheets ledger: key 125 → 281 → **`309-empress-sheets-ledger.md`**
  (key 125 collision at RETIRE: the ledger took 281 as the next free
  number; the frozen 150 registry `a15cb634b` §7.1 later CORRECTED key
  125's winner to d125 itself — d38's crossnote revoked → 293 — and §8.1
  renumbered the 281 self-pick to **309**, f26 owning 281). Header updated;
  every ref updated on this branch: ANALYSIS ×2, PORT_MAP, ROADMAP,
  anchors.txt, assets.rs, empress_draw.rs, entities.rs ×3,
  tools/_d125_sheets_measure.js.
- This FINAL doc took **282** at RETIRE (281/282 verified free across all
  remote tips then; 244/245/246 were taken mid-session by d53/f39/g48/g86/
  g48/d10/g43 — the first RETIRE draft's 244/245 numbers were re-pinned
  before any commit). The frozen 150 registry `a15cb634b` §8.2 adjudicated
  282 to §7.2's owner f26: this doc renumbers to **319**
  (`319-d125-retire-final.md`).
- The registry itself (150) lives on d54's branch — not touched here.

## 6. Verification record

- Alive-probes: 13:00/13:06 (95 s apart, zero movement) at entry;
  14:56/14:58 (95 s apart, the foreign editing.rs hunk frozen at 13:09:06)
  mid-session. HEAD `7a0482302` unchanged throughout.
- `git fetch origin` + `rev-list --left-right --count` = `0 0` (the
  REGRESSED flag resolved, §0).
- Protocol-bijection pre-check: grep for `=> 636,`/`636 => NpcKind`/742
  arms in npc/data.rs before the renumber — zero collisions.
- ALL_KINDS arm count machine-counted (272) before the decl fix.
- d43/d78/g1 hunks read via `git show` at the exact tips
  (`0253bde90`/`fd59c51a0`/g1 dirty tree) — read-only, no other branch
  touched.
- C# re-reads at every compose decision: Player.cs:38013-38021/
  :38197-38216/:38378-38394/:18887-18893/:31325-31430,
  Projectile.cs:14773-14900/:15406-15450/:12487-12516/:17190-17212,
  NPC.cs:17188-17206/:17351-17366, NPCID.cs:11469-11479,
  ProjectileID.cs:2296-2402.
- No cargo/tests/rustfmt/graphify (code-only mandate). Witnesses WRITTEN
  NOT RUN. Commit + push on `task/W4-d125-empress-r7-drain` only; no
  merge, primary untouched.
