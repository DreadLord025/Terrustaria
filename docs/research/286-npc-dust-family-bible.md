# 286 — NPC dust family bible (W4-e2 SEAL)

> 2026-11-21 · code-only · the NPC.cs `Dust.NewDust*` family's final
> ownership map at the fetched lane tips. Companion docs: d54's
> `131-dust-master-status.md` §8.2 (the whole-Dust.cs family census,
> tip `38bfd1d22`), e2's `285-npc-dust-window-coverage.md` (the
> per-window table + the overlap register, renumbered 64→285 per
> registry-150 — the interim 284 pick lost to g52's executed push
> `0224fff80`), f4's `283-npc-dust-final.md` (the f4 slice + the
> keep-one confirmation, tip `26728bd87`). Number 286 verified
> zero-claimant at the final pre-push sweep (281×8 / 282×3 / 283=f4 /
> 284=g52 / 298=g63 taken; 285 this lane's coverage doc).

## 1. The family count (re-derived THIS round, direct-path)

`Select-String` on `decompiled/Terraria/NPC.cs` (the worktree junction
resolved to its direct target; no recursion):

| Family | Count | Window split |
| ------ | ----- | ------------ |
| `Dust.NewDust(` | 836 | AI/misc <50000: 159 · 50000-69999: 87 · spawn/Transform 70000-81653: 12 · **HitEffect 81654-89834: 629** · >89834: 40 |
| `Dust.NewDustPerfect(` | 16 | (included in the window split above) |
| `Dust.NewDustDirect(` | 75 | (included in the window split above) |
| **ND-family total** | **927** | 159+87+12+629+40 = 927 ✓ |
| `Dust.CloneDust(` | 5 | 2 inside HitEffect (g62's ledger), 3 elsewhere (owner lanes) |

Matches d54 bible §8.2's NPC row (**927 +5 Clone = 932**, HitEffect
629+2) exactly — the bible's count stands at the seal tips.

## 2. Ownership map (window → lane → status at the seal tips)

| C# window | Owner | Status | Tip / evidence |
| --------- | ----- | ------ | -------------- |
| AI_045/047 Golem 19344-19714 | d52 b1 | wired | `e01179d5e` (d52 `231b31555`) |
| AI_004 Eye 20064-20379 | d52 b2 | wired | `f86731ed6` |
| aiStyle-8 caster 20867-21368 | d52 b3/b4 | wired | `7ce6e95c8`/`e55b04b23` |
| aiStyle-9 sphere 21524-21562 | d52 b5 | wired | `eb084fa9a` |
| :21562-39039 gap (49 lines) | e2 b1-b3/b6 + c4 + style56 + frost-boss-b + style74 + saucer | wired 40 / **staged 9** | research/285 §2; the 9 staged = 24520/24850/24866/35157/35162/37289/37307/37337/37464 (`9912498fc`) |
| pillar 39007-39362 | e2 b4/b5p1 | wired | `1ec45ba28`/`0f0d7036e` |
| StardustCell 39421 | e2 b2 | wired | `2bfaa71a4` |
| NebulaFloater 39524/39559/39571 | e2 b5/b6 | wired (faithful aiStyle-97) | `31c91d902`/`6c424595f` |
| SolarGoop/AncientLight/AncientDoom/SandElemental 39963-40302 | e2 b5 | **staged** (4 kinds, 9 lines) | `31c91d902` + the b6 relabel `1cbbcb95d` |
| EterniaCrystal 40834/40851/40947 | **f4 b1 KEEP** (e2 b5 drops) | wired | `37d143c75`; register §5 below |
| MoonLordCore storm 41336-41403 | **f4 b6 KEEP** (e2 b5 partial drops) | wired | `abff45cf8` + b7 order-adoption `e7ea8b24b` |
| MoonLordHead fan 42222 | **f4 b6 KEEP** | wired | `abff45cf8`; e2 angle fix `d5ae8725c` |
| TrueEye pair+fan 42675/42873 | **f4 b6 KEEP** | wired | `abff45cf8`; e2 angle fix `d5ae8725c` |
| KingSlime trio 43251/43278/43365 | **f4 b2 KEEP** (e2 b6 drops inline) | wired | `23e313f60` / `1cbbcb95d` |
| Deerclops 44037 + Fishron 49029-49661 + StardustWorm 51046/51054 | f4 b1 | wired | `37d143c75` |
| Empress/HallowBoss/wyvern/misc AI 44937-69003 | empress / bow-launch / kind waves | cited (research/285 §2 other-lane table) | the 619/454-459/621-623 dormant blocks per f4 research/283 §2 |
| Destroyer fade 49801 | f4 b3 | wired | `0b7b14e29` |
| AI_005 hover gate 50702/50710 | f4 b5 | wired | `87da911e4` (619 arm dormant, cited) |
| leech entry 50976 | f4 b7 | wired | `e7ea8b24b` (the AI_006 seam's one live site) |
| Crawltipede 51656/51662 + EoW fade 52259 | f4 b4 | wired | `8d5e4e7cf` |
| FindFrame 71938 (case-398 open burst) | f4 b6 | wired | `abff45cf8` + the six-burst re-fire pin `bcf917437` |
| FindFrame other dust 74009-74846 | d44-d49 findframe lanes | their census | research/285 §2 |
| spawn/Transform/Teleport 77494-81058 + 90236-92241 | g61 | wired | `0a4af1908`/`6345f7b7b`/`8d23e2985`; SEAL `6e922fc60` (the g11 dispatcher keep-one per its research/238) |
| **HitEffect every-strike 81660-89834** | g62 | wired (29 batches, "0 uncovered live sites") | `32af98b44`; the +2 CloneDust ride its ledger; f5's 4 hit-half gore bodies EMIT-AT-WIRING here (f5 research/111 §7) |
| HitEffect death halves (dust-only kinds) | g59 | wired (16 batches, ACTIVATIONS-2 census) | `d277f782b` |
| HitEffect death-family matrix | g60 | wired + research/110 FINAL-FINAL | `30fea0fe1` (the g62 6-hunk widening byte-identical; the f10-overrun DROP-HUNKS published) |
| >89834 buff-VFX producers (incl. 90977 soul-drain 235) | buff lanes c16-c18/d71-d74 | **UNPORTED, cited** (d54 bible G6) | npc_buffs.rs:619 note |

## 3. The keep-one symbol list (merger input, MUTUAL seal)

Both sides sealed: e2 research/285 §5 + f4 research/283 §1 agree
verbatim.

| Sites | KEEP (f4 tip `26728bd87`) | DROP (e2 side) | e2 SUPPLEMENTS kept |
| ----- | ------------------------ | -------------- | ------------------- |
| 43251/43278/43365 | f4 inline `update_king_slime` arms (`23e313f60`) | e2 inline arms (`1cbbcb95d`) | `npc_dust.rs::king_slime_teleport_ring`/`king_slime_body_drip` + the grow-ring/body-drip machine witnesses |
| 40834/40851/40947 | f4 `update_eternia_crystal` (`37d143c75`) | e2 bodies (`31c91d902`) | the win/loss-ring machine witnesses |
| 41367/41396 | f4 FULL storm (`abff45cf8`, b7 order-adoption + both `(int)>>4` probes) | e2 partial (`31c91d902`+`6e304b960`) | `mirror_storm`-fed storm/smoke witnesses (e2's module keeps the main-inherited S1-D-3 pins) |
| 42222 | f4 head fan (widened-f32 angle + left-to-right velocity) | e2 body | `w4_e2_head_deathray_fan_one_then_two` (angle re-mirrored `d5ae8725c`) |
| 42675/42873 | f4 eye pair + fan | e2 bodies | `w4_e2_true_eye_orbit_spoke_pair` + `w4_e2_true_eye_deathray_fan_one_then_two` |

Fidelity receipts (the two rounds that made the bodies
draw-equivalent):
- f4 b7 (`e7ea8b24b`) absorbed the e2-b5 left-to-right f32 multiply
  order (`6e304b960`): `(rot*num7)*(nf*1.6+1.6)`,
  `(0,-1)*num7*(nf*0.9+1.6)`, `(n*3.5)*(10-num9*2)/10` — f32
  multiplication is not associative; and fixed both storm tile probes
  to `(int)x >> 4` (`Utils.cs:1807-1810`).
- e2 seal (`d5ae8725c`) adopted f4's widened-f32 fan ANGLE:
  `(float)NextDouble() * ((float)PI*2f)` (`NPC.cs:42221`/`:42872`) —
  truncate BEFORE the multiply, f64 cos/sin over the f32 angle
  (`Utils.cs:1665-1668`); both e2 witnesses re-mirror it.
- Iteration-count equivalence: f4's `ceil(ai[1]/60)` == e2's float
  `while num11 < ai[1]/60` for every in-band value (60 < ai[1] < 580,
  ai[1]%3==0 ⇒ ai[1] is an integer ≥ 63; for integer n, ceil(n/60)
  counts exactly the integers k with k < n/60 — equal at every tick,
  including the exact multiples).

## 4. Wiring obligations (the staged remainder, named owners)

| Staged emitter (`npc_dust.rs`) | Sites | Wiring owner / unlock |
| ------------------------------ | ----- | --------------------- |
| `antlion_fall_drip` | 24520 | Antlion 69 kind row (d35 band) + the aiStyle-19 machine; the two Next(2) gate draws stay at the machine's C# position |
| `pixie_ambient_drip` / `ice_elemental_ambient_drip` | 24850/24866 | Pixie 75 / IceElemental 169 kind rows + aiStyle-22; the :24862 AddLight/alpha=30 half rides the light lane (cited skip) |
| `turret_materialize_ring` | 35157/35162 | the `npc_ai_style73.rs::turret_materialize_dust_plan` seam (c11) — the plan's 8 placements feed the emitter verbatim |
| `shadowflame_apparition_spawn_ring` / `_fade_drip` | 37289/37307 | ShadowflameApparition 472 kind row (d40 band) + aiStyle-86; the fade count is the FLOAT `(1-localAI[0]/120)*20` |
| `squidhead_228_ring` | 37337/37464 | Squidhead 521 kind row + aiStyle-86 (NOT AncientDoom — the b6 attribution fix) |
| `solar_goop_launch_burst`/`_trail`, `ancient_light_launch_burst`/`_trail`, `ancient_doom_rings`, `sand_elemental_bottom_sand`/`_tether` | 39963-40302 | the four pillar-minion kinds (519/522/523/541) + their aiStyle 99/100/101/102 machines (the `npc_ai_style9` solar-flare precedent) |

All staged bodies + RNG orders are pinned with witnesses (21 emitter
witnesses in `npc_dust.rs`); the wiring wave draws nothing new — it
threads DustPool+SpawnCtx and calls at the C# stream position.

## 5. SEAL declaration

The NPC dust family is **fully claimed**: every one of the 927+5
sites maps to a lane row above; the only UNPORTED remainder is the
>89834 buff-VFX producer set (named buff lanes, d54 bible G6) and the
dormant kind-gated blocks (cited). The e2 lane is **SEALED** at
`d5ae8725c` + this doc round — no further e2 dust batches owed; the
f4 lane is SEALED at `26728bd87` (its research/283). The merger takes
f4's keep-one bodies + e2's supplements per §3, wires the staged
emitters per §4, and runs the witness UNION compile+run in the
merge-verify wave (d54 bible §8.1's owed row — every lane code-only).

**Verification record:** code-only round — no cargo/rustc/rustfmt/
tests/dotnet/graphify invoked; witnesses WRITTEN NOT RUN. C# reads
direct-path (`NPC.cs` windows :41336-41405/:42204-42229/:42668-42681/
:42857-42880, `Utils.cs:1665-1668`/`:1807-1810`); lane comparisons
`git show <tip>:<path>` at the exact tips named; the registry sweep
`git ls-tree` over all origin `task/W4-*` tips (281-298 probed, three
pre-push fetches — the 284 race caught and re-picked).