# 64 — Main.cs dust-site batches (W4-g11 / W4-g12 / W4-g13): claims + census

Status: CLAIM doc (W4-g12, batch 2 of 3). Written early per the lane's
coordination rule — the branch list is the claim record; this doc pins the
line-order split so the three batches never overlap.

## Census

`decompiled/Terraria/Main.cs` carries **23** `Dust.NewDust*`-family call sites
(research/15's per-file table). Line-ordered:

| # | Line | Family | Enclosing method |
| - | ---- | ------ | ---------------- |
| 1 | 13088 | NewDust | `snowing()` (`:13032`) |
| 2 | 13243 | NewDust | `TeleportEffect` (`:13233`) |
| 3 | 13262 | NewDust | `TeleportEffect` |
| 4 | 13280 | NewDust | `TeleportEffect` |
| 5 | 13290 | NewDust | `TeleportEffect` |
| 6 | 13324 | NewDust | `TeleportEffect` |
| 7 | 13340 | NewDustDirect | `TeleportEffect` |
| 8 | 13358 | NewDustDirect | `TeleportEffect` |
| 9 | 13390 | NewDustDirect | `TeleportEffect` |
| 10 | 13438 | NewDustPerfect | `TeleportEffect` |
| 11 | 13452 | NewDust | `TeleportEffect` |
| 12 | 22744 | NewDust | `DrawNPCDirect` (`:22513`) |
| 13 | 22820 | NewDust | `DrawNPCDirect` |
| 14 | 24736 | NewDust | `DrawNPCDirect` |
| 15 | 24748 | NewDust | `DrawNPCDirect` |
| 16 | 24752 | NewDust | `DrawNPCDirect` |
| 17 | 24756 | NewDust | `DrawNPCDirect` |
| 18 | 27381 | NewDustDirect | `DrawNPCExtras` (`:26912`) |
| 19 | 27384 | NewDustDirect | `DrawNPCExtras` |
| 20 | 38664 | NewDust | (census in batch 3) |
| 21 | 57766 | NewDust | (census in batch 3) |
| 22 | 57789 | NewDust | (census in batch 3) |
| 23 | 57793 | NewDust | (census in batch 3) |

## Line-order split ( thirds )

- **W4-g11 `task/W4-g11-dust-main-sites-1`** — first third: sites 1-8
  (lines 13088-13358: `snowing()` + `TeleportEffect` through its second
  `NewDustDirect`).
- **W4-g12 `task/W4-g12-dust-main-sites-2`** — middle third: sites 9-16
  (lines 13390-24752: the `TeleportEffect` tail — the 267 sparkle lane +
  the 27 smoke — and the first five `DrawNPCDirect` sites).
- **W4-g13 `task/W4-g13-dust-main-sites-3`** — last third: sites 17-23
  (lines 24756-57793: the fourth `DrawNPCDirect` noGravity puff, the
  `DrawNPCExtras` pair, and the four late-file sites).

Boundary sites are owned whole: 13358 is g11's, 13390 is g12's; 24752 is
g12's, 24756 is g13's. The four-site `noGravity` cluster
(`:24736/24748/24752/24756`) straddles the g12/g13 boundary by the agreed
line-order rule — each batch ports its own sites verbatim; the shared
enclosing-arm context is cited in both.

Pool contract: `origin/task/W4-d50-dust-type-table`
(`tr_core::particles::DustPool::new_dust`, `Dust.cs:75-256` verbatim).
RNG: one slot per C# pass — each ported site drains on the stream of the
`Main.SwapRandom` pass that executes it in the C# (never a new stream,
S1-A-10); draw-path sites follow the port's draw-dust precedent.