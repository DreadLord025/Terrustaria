# W4-f26 slice 7f replay-sweep witness — WRITTEN NOT RUN (code-only).
#
# Line-for-line Python transcriptions of the five C# FindFrame bodies
# this slice ports (npc_findframe_extra7f.rs), the hand-derived replay
# vectors pinned in the module's #[cfg(test)] witnesses, and the
# 5-kind x 27-combo x 40-tick strip sweep. Run later with:
#     python tools/_f26_7f_replay.py
#
# Sources (decompiled/Terraria/NPC.cs):
#   case 165/237/238/240/531 :73112-73145 (only 531 this wave, the
#                            num194 = 0.4 override :73119-73122)
#   case 508/532/580/582     :70067-70103 (only 532 this wave — the
#                            verbatim 7c antlion machine, 8-row count)
#   case 534 (humanoid)      :68615 -> :68647-69655 (num108 = 12
#                            :69528-69531, num57 = 0, air row 1)
#   case 535 (slime strip)   :70818 -> :70828-70845 (num2 :66696-66704)
#   case 536 (zombie walker) :76344 -> :76347-76390
# Frame counts Main.cs:67082 (:67137's 530-539 row: 531=4 entry 2,
# 532=8 entry 3, 534=8 entry 5, 535=2 entry 6, 536=3 entry 7),
# cross-checked vs render/entities.rs NPC_FRAME_COUNT (all 697 slots).
#
# Precision note: the C# frameCounter is a double (NPC.cs:6485) fed by
# float products (velocity.X is float, * 0.4f is a float multiply);
# this witness rounds each product to f32 (struct) then accumulates in
# Python double — the C# order. The port's f32 counter diverges only
# in the last ulp; the pinned wall-scorpion cadence uses vx 2.5
# (2.5f * 0.4f rounds to EXACTLY 1.0f) so every band edge is exact in
# both; the other arms' accumulators are integer/half steps.

import struct


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


class State:
    def __init__(self, row=0, counter=0.0, sprite=0):
        self.row = row
        self.counter = counter
        self.sprite = sprite


def wall_scorpion(s, vx, vy, direction, ai_action):
    # NPC.cs:73112-73145 (case 165/237/238/240/531; only 531 here).
    num194 = f32(0.4)                               # :73118-73122 (531)
    s.counter += f32((abs(vx) + abs(vy)) * num194)  # :73123 EVERY tick
    if s.counter < 6.0:                             # :73124
        s.row = 0                                   # :73126
    elif s.counter < 12.0:                          # :73128
        s.row = 1                                   # :73130
    elif s.counter < 18.0:                          # :73132
        s.row = 2                                   # :73134
    elif s.counter < 24.0:                          # :73136
        s.row = 3                                   # :73138
    else:                                           # :73140
        s.counter = 0.0                             # :73142 row NOT written


def beast(s, vx, vy, direction, ai_action, count=8):
    # NPC.cs:70067-70103 (case 508/532/580/582; only 532 here — the
    # verbatim 7c machine with the 8-row count).
    if vy != 0.0:                                   # :70071
        s.row = 1                                   # :70073 counter UNTOUCHED
        return                                      # :70074 break
    if vx == 0.0:                                   # :70076
        s.row = 0                                   # :70078 counter UNTOUCHED
        return                                      # :70079 break
    if direction < 0 and vx < 0.0:                  # :70081
        s.sprite = -1                               # :70083
    if direction > 0 and vx > 0.0:                  # :70085
        s.sprite = 1                                # :70087
    if s.row < 2:                                   # :70089
        s.row = 2                                   # :70091
    s.counter += 1.0 + abs(vx) / 2.0                # :70093
    if s.counter > 6.0:                             # :70094 STRICT >
        s.row += 1                                  # :70096
        s.counter = 0.0                             # :70097
    if s.row >= count:                              # :70099
        s.row = 2                                   # :70101


def tax_collector(s, vx, vy, direction, ai_action, count=8):
    # NPC.cs:68615 -> the humanoid fighter walk :68647-69655 with the
    # 534 num108 = 12 override (:69528-69531). num57 = 0 (aiStyle 3,
    # :15817, no townNPC), num109 = row 2 (not a town slime), idle
    # row 0 (not the 140/489 pair), air row 1 (not in the :69647 pin
    # list). The ai[0] pose zones :68662-69488 are the documented skip.
    if vy == 0.0:                                   # :68647
        if direction == 1:                          # :68649
            s.sprite = 1                            # :68651
        if direction == -1:                         # :68653
            s.sprite = -1                           # :68655
        if vx == 0.0:                               # :69489
            s.row = 0                               # :69517
            s.counter = 0.0                         # :69518
        else:                                       # :69521
            if s.row < 2:                           # :69572 (num109 = 2)
                s.row = 2                           # :69574
            s.counter += f32(abs(vx) * f32(2.0))    # :69552
            s.counter += 1.0                        # :69553 ++
            if s.counter > 12.0:                    # :69576 num108 = 12
                s.row += 1                          # :69578
                s.counter = 0.0                     # :69579
            if s.row >= count:                      # :69581 (num57 = 0)
                s.row = 2                           # :69583
    else:                                           # :69642
        s.counter = 0.0                             # :69644
        s.row = 1                                   # :69645


def spiked_slime(s, vx, vy, direction, ai_action, count=2):
    # NPC.cs:70818 -> :70828-70845 (the slime strip group). The
    # 302/304 sprite snap :70824-70827 is the town slimes' — 535
    # writes no flip.
    num2 = 0                                        # :66696
    if ai_action == 0:                              # :66697
        if vy < 0.0:
            num2 = 2                                # :66699
        elif vy > 0.0:
            num2 = 3
        elif vx != 0.0:
            num2 = 1
    elif ai_action == 1:                            # :66701
        num2 = 4                                    # :66703
    s.counter += 1.0                                # :70828 ++
    if num2 > 0:                                    # :70829
        s.counter += 1.0                            # :70831
    if num2 == 4:                                   # :70833
        s.counter += 1.0                            # :70835
    if s.counter >= 8.0:                            # :70837
        s.row += 1                                  # :70839
        s.counter = 0.0                             # :70840
    if s.row >= count:                              # :70842
        s.row = 0                                   # :70844


def bride(s, vx, vy, direction, ai_action):
    # NPC.cs:76344 -> :76347-76390 (the zombie walker group — the
    # verbatim wave-A zombie_walk_frame).
    if vy == 0.0:                                   # :76347
        if direction == 1:                          # :76349
            s.sprite = 1                            # :76351
        if direction == -1:                         # :76353
            s.sprite = -1                           # :76355
    if (vy != 0.0 or (direction == -1 and vx > 0.0)
            or (direction == 1 and vx < 0.0)):        # :76358
        s.counter = 0.0                             # :76360
        s.row = 2                                   # :76361
        return                                      # :76362 break
    if vx == 0.0:                                   # :76364
        s.counter = 0.0                             # :76366
        s.row = 0                                   # :76367
        return                                      # :76368 break
    s.counter += abs(vx)                            # :76370 NO +1
    if s.counter < 8.0:                             # :76371
        s.row = 0                                   # :76373
    elif s.counter < 16.0:                          # :76375
        s.row = 1                                   # :76377
    elif s.counter < 24.0:                          # :76379
        s.row = 2                                   # :76381
    elif s.counter < 32.0:                          # :76383
        s.row = 1                                   # :76385
    else:                                           # :76387
        s.counter = 0.0                             # :76389 row NOT written


KINDS = {
    531: (wall_scorpion, 4),
    532: (beast, 8),
    534: (tax_collector, 8),
    535: (spiked_slime, 2),
    536: (bride, 3),
}


def tick(kind, s, vx, vy, direction=1, ai_action=0):
    fn, count = KINDS[kind]
    if fn in (wall_scorpion, bride):
        fn(s, vx, vy, direction, ai_action)
    else:
        fn(s, vx, vy, direction, ai_action, count)


def rows_over(kind, vx, vy, ticks, direction=1, ai_action=0):
    s = State()
    out = []
    for _ in range(ticks):
        tick(kind, s, vx, vy, direction, ai_action)
        out.append(s.row)
    return out


def check_vectors():
    # The module's pinned replays (f26s7f_* tests), hand-derived and
    # re-checked line-for-line above.
    k = 531
    # vx 2.5: 2.5f * 0.4f == 1.0f EXACTLY — the band edges are exact.
    assert rows_over(k, 2.5, 0.0, 25) == [
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
        3, 3, 3, 3, 3, 3, 3, 0]
    s = State(row=3, counter=23.0)
    tick(k, s, 2.5, 0.0)
    assert (s.row, s.counter) == (3, 0.0)
    assert rows_over(k, 0.0, -2.0, 8) == [0, 0, 0, 0, 0, 0, 0, 1]
    s = State(row=2, counter=13.0)
    tick(k, s, 0.0, 0.0)
    assert (s.row, s.counter) == (2, 13.0)
    k = 532
    assert rows_over(k, 3.0, 0.0, 19) == [
        2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 2, 2]
    s = State(row=7, counter=5.0)
    tick(k, s, 3.0, 0.0)
    assert (s.row, s.counter) == (2, 0.0)
    s = State(row=4, counter=5.0, sprite=1)
    tick(k, s, -3.0, 2.0, direction=-1)
    assert (s.row, s.counter, s.sprite) == (1, 5.0, 1)
    s = State(row=4, counter=5.0)
    tick(k, s, 0.0, 0.0)
    assert (s.row, s.counter) == (0, 5.0)
    s = State(sprite=-1)
    tick(k, s, -3.0, 0.0, direction=1)
    assert (s.row, s.counter, s.sprite) == (2, 2.5, -1)
    s = State(sprite=1)
    tick(k, s, -3.0, 0.0, direction=-1)
    assert s.sprite == -1
    k = 534
    assert rows_over(k, 1.0, 0.0, 15) == [
        2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5]
    s = State(row=7, counter=11.0)
    tick(k, s, 1.0, 0.0)
    assert (s.row, s.counter) == (2, 0.0)
    s = State(sprite=1)
    tick(k, s, -1.0, 0.0, direction=-1)
    assert s.sprite == -1
    s = State(row=5, counter=7.0, sprite=-1)
    tick(k, s, 0.0, 0.0, direction=0)
    assert (s.row, s.counter, s.sprite) == (0, 0.0, -1)
    s = State(row=5, counter=7.0, sprite=1)
    tick(k, s, 3.0, -2.0, direction=-1)
    assert (s.row, s.counter, s.sprite) == (1, 0.0, 1)
    s = State(row=5, counter=7.0)
    tick(k, s, 3.0, 2.0)
    assert (s.row, s.counter) == (1, 0.0)
    k = 535
    assert rows_over(k, 0.0, 0.0, 9) == [0, 0, 0, 0, 0, 0, 0, 1, 1]
    assert rows_over(k, 1.0, 0.0, 5) == [0, 0, 0, 1, 1]
    assert rows_over(k, 0.0, -2.0, 5) == [0, 0, 0, 1, 1]
    assert rows_over(k, 0.0, 0.0, 9, ai_action=1) == [
        0, 0, 1, 1, 1, 0, 0, 0, 1]
    s = State(sprite=-1)
    tick(k, s, 2.0, 0.0)
    assert s.sprite == -1
    k = 536
    assert rows_over(k, 3.0, 0.0, 12) == [
        0, 0, 1, 1, 1, 2, 2, 1, 1, 1, 1, 0]
    s = State(row=1, counter=30.0)
    tick(k, s, 3.0, 0.0)
    assert (s.row, s.counter) == (1, 0.0)
    s = State(row=0, counter=9.0, sprite=1)
    tick(k, s, -3.0, 2.0, direction=-1)
    assert (s.row, s.counter, s.sprite) == (2, 0.0, 1)
    s = State(row=1, counter=9.0, sprite=1)
    tick(k, s, 3.0, 0.0, direction=-1)
    assert (s.row, s.counter, s.sprite) == (2, 0.0, -1)
    s = State(row=2, counter=9.0, sprite=-1)
    tick(k, s, 0.0, 0.0, direction=0)
    assert (s.row, s.counter, s.sprite) == (0, 0.0, -1)


def sweep():
    # The 5-kind x 27-combo x 40-tick strip sweep: every row stays
    # inside its own sheet (the module's
    # f26s7f_the_5_kind_strip_sweep_stays_in_sheet twin — 40 ticks so
    # the wall scorpion's 24-tick band cycle and the bride's 11-tick
    # lap both exercise their else-quirks).
    combos = 0
    for k, (_, count) in KINDS.items():
        for vx in (0.0, 1.0, 3.0):
            for vy in (0.0, -2.0, 2.0):
                for direction in (1, -1, 0):
                    s = State()
                    for _ in range(40):
                        tick(k, s, vx, vy, direction)
                        assert 0 <= s.row < count, (k, vx, vy, direction, s.row)
                    combos += 1
    assert combos == 135  # 5 kinds x 27 combos


if __name__ == "__main__":
    check_vectors()
    sweep()
    print("f26 slice 7f: all replay vectors + the 135-combo sweep PASS")
