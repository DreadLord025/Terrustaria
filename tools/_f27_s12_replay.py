#!/usr/bin/env python3
"""W4-f27 slice-12 FindFrame replay twin (WRITTEN; the vectors below are
re-derived by mechanical replay of line-for-line C# transcriptions).

Code-only mode: this script is the derivation aid for the Rust witness
expectations. It transcribes the C# FindFrame arms verbatim (integer
accumulators exact; the dark-mage hover walk's |v|*0.1+1 the one
non-integer feed) and replays them to pin every rung boundary.

C# ranges (decompiled/Terraria/NPC.cs):
  551 Betsy        :67258-67451  (ai[0] phase ladders)
  566/567 skeleton :67559-67583  (grounded walk, idle row 0)
  568/569 wither   :67025-67257  (ai[0]>0 aura ladder + walk)
  570/571 drakin   :67711-67755  (ai[1]>0&&ai[0]>0 ladder + walk)
  564/565 darkmage :67998-68421  (RAW-cell grid ladders + hover walk)
  576/577 ogre     :67584-67709  (RAW-cell grid ladders + walk)
"""
import math


def f32(x):
    # the port's f32 convention; integer accumulators are exact, the
    # hover walk's |v|*0.1+1 rounds to f32 (struct.unpack round-trip).
    import struct
    return struct.unpack('f', struct.pack('f', x))[0]


# ---- 566/567 skeleton (NPC.cs:67561-67582) ------------------------------
def skeleton(row, ctr, vx, vy, direction):
    if vy == 0.0:
        sprite = direction
    else:
        sprite = None  # untouched
    if vy != 0.0 or (direction == -1 and vx > 0.0) or (direction == 1 and vx < 0.0):
        return 8, 0.0
    if vx == 0.0:
        return 0, 0.0
    ctr += abs(vx)
    if ctr >= 64.0 or ctr < 0.0:
        ctr = 0.0
    return int(ctr / 8.0), ctr


# ---- 568/569 wither beast (NPC.cs:67027-67256) --------------------------
AURA = [8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 11, 12, 13, 14]


def wither(row, ctr, vx, vy, direction, ai0):
    if ai0 > 0.0:
        if row < 5 or row > 16:
            ctr = 0.0
        row = 7
        ctr += 1.0
        for k in range(1, 28):
            if ctr >= 5 * k:
                row = AURA[k - 1]
        if ctr >= 270.0:
            row = 14
            ctr -= 10.0
        return row, ctr
    if vy != 0.0 or (direction == -1 and vx > 0.0) or (direction == 1 and vx < 0.0):
        return 4, 0.0
    if vx == 0.0:
        return 6, 0.0
    ctr += abs(vx)
    if ctr >= 56.0 or ctr < 0.0:
        ctr = 0.0
    return int(ctr / 8.0), ctr


# ---- 570/571 drakin (NPC.cs:67713-67754) --------------------------------
def drakin(row, ctr, vx, vy, direction, ai0, ai1):
    if ai1 > 0.0 and ai0 > 0.0:
        if row < 10 or row > 15 or ai1 == 59.0:
            row = 10
            ctr = 0.0
        step = 35.0 if row == 14 else 5.0
        ctr += 1.0
        if ctr >= step and row < 15:
            ctr = 0.0
            row += 1
        return row, ctr
    if vy != 0.0 or (direction == -1 and vx > 0.0) or (direction == 1 and vx < 0.0):
        return 1, 0.0
    if vx == 0.0:
        return 9, 0.0
    ctr += abs(vx)
    if ctr >= 64.0 or ctr < 0.0:
        ctr = 0.0
    return int(ctr / 8.0 + 2.0), ctr


# ---- 564/565 dark mage (NPC.cs:68004-68418), RAW cell -------------------
def darkmage(cell, ctr, vx, vy, direction, ai0, ai1):
    if ai0 > 0.0 and ai1 == 0.0:
        if cell < 5 or cell > 13:
            ctr = 0.0
        cell = 5
        ctr += 1.0
        # 14 rungs, 7*k: 6,7,5,6,7,5,6,7,8,9,10,11,12, then reset
        rungs = [6, 7, 5, 6, 7, 5, 6, 7, 8, 9, 10, 11, 12]
        for k in range(1, 14):
            if ctr >= 7 * k:
                cell = rungs[k - 1]
        if ctr >= 7 * 14:  # 98
            cell = 5
            ctr = 0.0
        return cell, ctr
    if ai0 > 0.0 and ai1 == 1.0:
        if cell < 13 or cell > 25:
            ctr = 0.0
        cell = 13
        ctr += 1.0
        rungs = [14, 15, 16, 17, 18, 19, 20, 18, 19, 20, 21, 22, 23, 24, 25]
        for k in range(1, 16):
            if ctr >= 8 * k:
                cell = rungs[k - 1]
        if ctr >= 8 * 16:  # 128
            cell = 14
            ctr = 0.0
        return cell, ctr
    if ai0 > 0.0 and ai1 == 2.0:
        if cell < 26 or cell > 40:
            ctr = 0.0
        cell = 26
        ctr += 1.0
        rungs = [27, 28, 29, 26, 27, 28, 29, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40]
        for k in range(1, 23):
            if ctr >= 8 * k:
                cell = rungs[k - 1]
        if ctr >= 8 * 23:  # 184
            cell = 26
            ctr = 0.0
        return cell, ctr
    # hover walk :68413-68418
    ctr = f32(f32(ctr + f32(math.hypot(vx, vy) * 0.1)) + 1.0)
    if ctr >= 40.0 or ctr < 0.0:
        ctr = 0.0
    return int(ctr / 8.0), ctr


# ---- 576/577 ogre (NPC.cs:67589-67705), RAW cell ------------------------
def ogre(cell, ctr, vx, vy, direction, ai0, ai1):
    if ai0 > 0.0 and ai1 == 0.0:
        if cell < 11 or cell > 20:
            cell = 11
            ctr = 0.0
        stride = 4
        if cell == 13 or cell == 19:
            stride = 8
        if cell == 14 or cell == 18:
            stride = 2
        ctr += 1.0
        if ctr >= stride and cell < 20:
            ctr = 0.0
            cell += 1
        return cell, ctr
    if ai0 > 0.0 and ai1 == 2.0:
        if cell < 37 or cell > 47:
            cell = 39
            ctr = 0.0
        stride = 5
        if cell == 42:
            stride = 6
        if cell == 45:
            stride = 8
        if cell == 46:
            stride = 4
        if cell == 47:
            stride = 26
        if cell == 37 or cell == 38:
            stride = 7
        advance = True
        if cell == 46 and vy != 0.0:
            advance = False
        if cell == 38:
            advance = False
        if advance:
            ctr += 1.0
        if ctr >= stride:
            if cell < 47:
                ctr = 0.0
                cell += 1
            else:
                cell = 37
                ctr = 0.0
        return cell, ctr
    if ai0 > 0.0 and ai1 == 1.0:
        if cell < 21 or cell > 38:
            cell = 21
            ctr = 0.0
        ctr += 1.0
        if ctr >= 5.0 and cell < 38:
            ctr = 0.0
            cell += 1
        return cell, ctr
    # walk :67684-67705
    if vy != 0.0:
        return 43, 0.0
    if vx == 0.0:
        return 0, 0.0
    ctr += abs(vx)
    if ctr >= 60.0 or ctr < 0.0:
        ctr = 0.0
    return 1 + int(ctr / 6.0), ctr


def seq(fn, start, ticks, **kw):
    row, ctr = start
    out = []
    for _ in range(ticks):
        row, ctr = fn(row, ctr, **kw)
        out.append(row)
    return out


if __name__ == '__main__':
    print('skeleton vx4 walk:', seq(skeleton, (0, 0.0), 16, vx=4.0, vy=0.0, direction=1))
    print('wither aura ai0=1 from row0:', seq(wither, (0, 0.0), 40, vx=0.0, vy=0.0, direction=1, ai0=1.0))
    print('wither walk vx4:', seq(wither, (0, 0.0), 15, vx=4.0, vy=0.0, direction=1, ai0=0.0))
    print('drakin attack ai0=1 ai1=1:', seq(drakin, (0, 0.0), 45, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=1.0))
    print('drakin walk vx4:', seq(drakin, (0, 0.0), 16, vx=4.0, vy=0.0, direction=1, ai0=0.0, ai1=0.0))
    print('darkmage summon ai1=0:', seq(darkmage, (0, 0.0), 100, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=0.0))
    print('darkmage hover vx3:', seq(darkmage, (0, 0.0), 14, vx=3.0, vy=0.0, direction=1, ai0=0.0, ai1=0.0))
    print('ogre stomp ai1=0:', seq(ogre, (0, 0.0), 60, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=0.0))
    print('ogre walk vx4:', seq(ogre, (0, 0.0), 16, vx=4.0, vy=0.0, direction=1, ai0=0.0, ai1=0.0))
