# NPC gore wave E — the HitEffect-remainder census + claim (W4-f4, 2026-11-21)

Code-only lane (`task/W4-f4-dust-npc-3`, phase 2). The dust THIRD-batch
lane (batch 1 `37d143c7`, pushed) had NO in-flight uncommitted work at the
probe (clean tree, local == origin), so this phase takes the gore remainder.

## 1. The HitEffect range ownership (verified against the pushed claims)

`NPC.HitEffect` spans `NPC.cs:81654-89834` (`:89836` is `CountNPCS`).

| range | owner | evidence (pushed) |
| --- | --- | --- |
| `:81654-81659` HEAD gap | NO gore — signature + `{` + `if (!this.active) return;` | direct read, this census |
| `:81660-81941` sites 1-30 | W4-d55 wave A | `ec78d152` (`npc_hit_effect_a.rs`) |
| `:81942-82218` sites 31-86 | W4-e5 wave B | `7758dc76`/`5bd25c87`/`78492a7e`/`135f39ce` (the 551 mirrored twin ends `:82215`; `:82216-82218` close the arm) |
| `:82206-85559` climbing | W4-f10 wave C | `215bcd2d`…`cf777d4b` (batches 1-40 pushed through `:84104`; the rest in-flight on its branch) |
| `:85560-89835` | W4-f5 wave D | `97c70c97` claim + batches 1-5 (`6963f8a4`…`73678dd0`) |

The e5/f10 boundary overlap at `:82206-82215` (the 551 flipped twin) is
THEIRS to reconcile (e5 commit 4 ports it; f10 batch 1 re-ports it) — this
lane does not touch it. **Every `Gore.NewGore*` call inside
`:81660-89834` is claimed. The HEAD gap carries zero gore. The HitEffect
remainder is EMPTY.**

## 2. The out-of-HitEffect gore census (1345 `Gore.NewGore*` sites total)

41 sites sit outside `:81654-89835`. Full method census:

| C# sites | method | status |
| --- | --- | --- |
| `:19717` | `AI_045_Golem` landing | d52-reserved (batch 1 armed the SafeGore 61-63 argument draws, `e01179d5`) — body stays the golem lane's |
| `:20367-20369` | `AI()` EoC spin hand-off | d52/d22-reserved (`f86731ed`/`c1d26278` armed the 2×3 DangerousGore draws) |
| `:26749-26751`, `:27363-27365` | `AI()` Twins hand-off | e2-reserved ("gore argument pairs kept for d55", `1da18d19`) — the twins machines' gore follow-up |
| `:29564`, `:29954` | `AI()` style39/41 explosions | e2-reserved ("gore argument pairs kept for d55", `98f3337e`) |
| `:32028-32030` | `AI()` FungiSpore 261 birth (`localAI[2]==0` one-shot) | **UNCLAIMED — taken (E1-E3)** |
| `:32410-32413` | `AI()` Brain retreat | e2-reserved (`2bfaa71a` armed the four argument pairs) |
| `:43233` | `AI_015_KingSlime` shrink tick-60 crown 734 | **UNCLAIMED — taken (E4)** |
| `:45424` | `AI_121_QueenSlime` vanish tick-60 crown 1258 | **UNCLAIMED — taken (E5)** |
| `:60934` | `AI_001_Slimes` confetti slime (`ai[1]==1345`) ambient gore | **UNCLAIMED — taken (E6)** — g62 batch 15 owns the *HitEffect* `:84732` confetti arm, NOT this AI-machine ambient site |
| `:62187` | `AI_110_Betsy` fire-breath tile-sweep `NewGoreDirect` | **UNCLAIMED — taken (E7)** |
| `:63247` | `AI_107_ImprovedWalkers` 572/573 kobold bomber explosion 5-gore | **UNCLAIMED — taken (E8)** |
| `:63609` | `AI_107_ImprovedWalkers` 567 DD2SkeletonT3 walk-drip 971/972 | **UNCLAIMED — taken (E9)** |
| `:65774` | `AI_108_DivingFlyer` 574/575 kobold glider explosion 5-gore | **UNCLAIMED — taken (E10)** |
| `:68953` | `FindFrame` 656 TownBunny last-row ear-flick gore 580-582 | **UNCLAIMED — taken (E11)** |
| `:71931-71934` | `FindFrame` 398 MoonLordCore open-frame quad 619-622 | **UNCLAIMED — taken (E12-E15)** |
| `:77511` | `BigMimicSpawnSmoke` 3×2 gore-99 grid | g61 owns (`0a4af190`, `npc_spawn_visuals.rs`) |
| `:80969-80989` | `TransformVisuals` | g61 owns (`0a4af190`) |
| `:81354` | `StrikeNPC_Inner` SantaNK1 346 half-hp gore 517 | **UNCLAIMED — taken (E16)** — g62's hit-dust claim is the HitEffect dust arms; this strike-body gore is outside every pushed range |
| `:91340` | `UpdateNPC_BuffApplyVFX` loveStruck heart 331 | **UNCLAIMED — taken (E17)** |

NPCLoot (`:78830+`), CheckDead, SpawnNPC/NewNPC (`:80750-80795`),
OnSpawn: **zero gore sites** (the 1345-site grep lists none in those
bodies; NewNPC's only draw is the getGoodWorld `Next(3)` — g61's census
`0a4af190` re-verified). TownNPC death/shimmer gore lives INSIDE
HitEffect's party-hat block (d55's) — no separate method.

## 3. The claim

**W4-f4 wave E claims the 17 sites E1-E17 above** (`npc_hit_effect_e.rs`,
standalone, the d55 module shape). Batch split (10 sites per commit):

* batch 1: E1-E10 (`:32028-32030`, `:43233`, `:45424`, `:60934`,
  `:62187`, `:63247`, `:63609`, `:65774`)
* batch 2: E11-E17 (`:68953`, `:71931-71934`, `:81354`, `:91340`)

RNG contract (the d55/g61 convention): every draw rides the caller's ONE
stream in C# evaluation order — argument expressions BEFORE the
`NewGore` internals (the `Gore.cs:1327-1328` Y-then-X kicks, pool-side),
post-writes' draws AFTER the allocator returned, sentinel slot included.
The interleaved DUST loops of the bomber/glider/Betsy/confetti hosts are
the dust lanes' (f4 THIRD-batch remainder / g62) — this module consumes
ONLY the gore-arm draws and documents the preceding dust draws as the
caller's obligation at wiring (the g59/g60 argument/body split precedent,
`eea6975b`/`e55e2272`).

Not taken: the d52/e2-reserved AI-machine gore bodies (the golem/EoC/
twins/style39/41/brain rows above) — their argument draws are already
armed by those lanes; the bodies stay with the machine owners. The g61
spawn/transform family stays g61's.

Witnesses: WRITTEN NOT RUN (code-only mode, no cargo per the lane
directive).
