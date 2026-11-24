# W4-f26 slice 7g replay-sweep witness — WRITTEN NOT RUN (code-only).
#
# Line-for-line Python transcriptions of the five C# FindFrame bodies
# this slice ports (npc_findframe_extra7g.rs), the hand-derived replay
# vectors pinned in the module's #[cfg(test)] witnesses, and the
# 5-kind x 27-combo x 40-tick strip sweep + the golem pose-zone sweep.
# Run later with:
#     python tools/_f26_7g_replay.py
#
# Sources (decompiled/Terraria/NPC.cs):
#   case 430..436/591       :70879-70951 (only 435/436 this wave — the
#                           verbatim d49 high2 machine: the ai[2]
#                           attack ladder rows 3..6 + the wave-A walk)
#   case 631                :75259-75313 (the golem: the ai[2] windup
#                           pose lerps + the LITERAL-8 walk cap)
#   case 632/635 (humanoid) :68619/:68621 -> :68647-69655 (632's
#                           num108 = 12 :69524-69527, 635's default 6,
#                           num57 = 0, air row 1)
# Frame counts Main.cs:67082 (435/436 = 7 :67127 entries 5/6, 631 = 21
# :67147 entry 1, 632 = 16 entry 2, 635 = 16 entry 5), cross-checked
# vs render/entities.rs NPC_FRAME_COUNT (all 697 slots).
#
# Precision note: the C# frameCounter is a double (NPC.cs:6485) fed by
# float products; this witness rounds each product to f32 (struct)
# then accumulates in Python double — the C# order. The port's f32
# counter diverges only in the last ulp; the pinned golem cadence uses
# vx 0.8 (0.2f + 0.8f rounds to EXACTLY 1.0f) so every step edge is
# exact in both; the other arms' accumulators are integer steps.

import struct


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


class State:
    def __init__(self, row=0, counter=0.0, sprite=0):
        self.row = row
        self.counter = counter
        self.sprite = sprite


def armed_zombie(s, vx, vy, direction, ai2):
    # NPC.cs:70879-70951 (case 430..436/591; only 435/436 here — the
    # verbatim d49 high2 machine).
    if ai2 > 0.0:                                   # :70887
        if ai2 < 5.0:                               # :70889
            s.row = 3                               # :70891
        elif ai2 < 10.0:                            # :70893
            s.row = 4                               # :70895
        elif ai2 < 15.0:                            # :70897
            s.row = 5                               # :70899
        else:                                       # :70901
            s.row = 6                               # :70903
        return                                      # :70905 counter/sprite UNTOUCHED
    if vy == 0.0:                                   # :70907
        if direction == 1:                          # :70909
            s.sprite = 1                            # :70911
        if direction == -1:                         # :70913
            s.sprite = -1                           # :70915
    if (vy != 0.0 or (direction == -1 and vx > 0.0)
            or (direction == 1 and vx < 0.0)):        # :70918
        s.counter = 0.0                             # :70920
        s.row = 2                                   # :70921
        return                                      # :70922 break
    if vx == 0.0:                                   # :70924
        s.counter = 0.0                             # :70926
        s.row = 0                                   # :70927
        return                                      # :70928 break
    s.counter += abs(vx)                            # :70930 NO +1
    if s.counter < 8.0:                             # :70931
        s.row = 0                                   # :70933
    elif s.counter < 16.0:                          # :70935
        s.row = 1                                   # :70937
    elif s.counter < 24.0:                          # :70939
        s.row = 2                                   # :70941
    elif s.counter < 32.0:                          # :70943
        s.row = 1                                   # :70945
    else:                                           # :70947
        s.counter = 0.0                             # :70949 row NOT written


def rock_golem(s, vx, vy, direction, ai2, facing_p=False, overlap=False,
               can_hit=False, count=21):
    # NPC.cs:75259-75313 (case 631). The pose zone's world reads
    # (:75270-75273) arrive as the three RESOLVED booleans (the
    # extra-7b seam precedent).
    if vy == 0.0:                                   # :75264
        s.sprite = direction                        # :75266 UNCONDITIONAL
        if ai2 != 0.0 and ai2 < 100.0:              # :75267
            amount = f32(f32(ai2 - 68.0) / 32.0)    # :75269
            if facing_p and not overlap and can_hit:  # :75273
                # :75275 — (int)MathHelper.Lerp(15f, count, amount)
                s.row = int(f32(15.0 + f32(f32(count - 15.0) * amount)))
            elif facing_p and overlap:              # :75277
                s.row = int(f32(10.0 + f32(5.0 * amount)))  # :75279
            else:                                   # :75281
                s.row = 0                           # :75283
            return                                  # :75285 counter UNTOUCHED
        if vx == 0.0:                               # :75287
            s.counter = 0.0                         # :75289
            s.row = 0                               # :75290
            return                                  # :75291 break
        s.counter += f32(0.2 + abs(vx))             # :75293
        if s.counter > 8.0:                         # :75294 STRICT >
            s.row += 1                              # :75296
            s.counter = 0.0                         # :75297
        if s.row >= 8:                              # :75299 LITERAL num31 = 8
            s.row = 1                               # :75301
        elif s.row < 1:                             # :75303
            s.row = 1                               # :75305
    else:                                           # :75308
        s.counter = 0.0                             # :75310
        s.row = 0                                   # :75311


def humanoid(s, vx, vy, direction, num108, count=16):
    # NPC.cs:68619/:68621 -> the humanoid fighter walk :68647-69655.
    # 632: num108 = 12 (:69524-69527); 635: the default 6 (:69523).
    # num57 = 0 (aiStyle 3 :17138/:17179, no townNPC), num109 = row 2,
    # idle row 0, air row 1 (neither in the :69647 pin list). The
    # ai[0] pose zones :68662-69488 are the documented skip.
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
            if s.counter > num108:                  # :69576
                s.row += 1                          # :69578
                s.counter = 0.0                     # :69579
            if s.row >= count:                      # :69581 (num57 = 0)
                s.row = 2                           # :69583
    else:                                           # :69642
        s.counter = 0.0                             # :69644
        s.row = 1                                   # :69645


KINDS = {
    435: ("zombie", 7),
    436: ("zombie", 7),
    631: ("golem", 21),
    632: ("humanoid12", 16),
    635: ("humanoid6", 16),
}


def tick(kind, s, vx, vy, direction=1, ai2=0.0, facing_p=False,
         overlap=False, can_hit=False):
    machine, count = KINDS[kind]
    if machine == "zombie":
        armed_zombie(s, vx, vy, direction, ai2)
    elif machine == "golem":
        rock_golem(s, vx, vy, direction, ai2, facing_p, overlap, can_hit, count)
    elif machine == "humanoid12":
        humanoid(s, vx, vy, direction, 12.0, count)
    else:
        humanoid(s, vx, vy, direction, 6.0, count)


def rows_over(kind, vx, vy, ticks, direction=1, ai2=0.0):
    s = State()
    out = []
    for _ in range(ticks):
        tick(kind, s, vx, vy, direction, ai2)
        out.append(s.row)
    return out


def check_vectors():
    # The module's pinned replays (f26s7g_* tests), hand-derived and
    # re-checked line-for-line above.
    for k in (435, 436):
        for ai2, row in [(1.0, 3), (4.0, 3), (5.0, 4), (9.0, 4),
                         (10.0, 5), (14.0, 5), (15.0, 6), (99.0, 6)]:
            s = State(row=1, counter=7.0, sprite=-1)
            tick(k, s, 3.0, 2.0, ai2=ai2)
            assert (s.row, s.counter, s.sprite) == (row, 7.0, -1), (k, ai2)
    k = 435
    assert rows_over(k, 3.0, 0.0, 12) == [0, 0, 1, 1, 1, 2, 2, 1, 1, 1, 1, 0]
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
    k = 631
    # The windup lerps: ai2 84 -> amount 0.5 -> rows 18/12; ai2 68 ->
    # amount 0 -> 15/10; ai2 99 -> 0.96875 -> (int)20.8125 = 20 / 14.
    for ai2, f, o, c, row in [
        (84.0, True, False, True, 18),
        (84.0, True, True, True, 12),
        (84.0, False, False, True, 0),
        (84.0, True, False, False, 0),
        (68.0, True, False, True, 15),
        (68.0, True, True, False, 10),
        (99.0, True, False, True, 20),
        (99.0, True, True, False, 14),
    ]:
        s = State(row=3, counter=5.0)
        tick(k, s, 1.0, 0.0, ai2=ai2, facing_p=f, overlap=o, can_hit=c)
        assert (s.row, s.counter) == (row, 5.0), (ai2, f, o, c)
    // ai2 100 exits the zone to the walk: vx 0.8 adds exactly 1.0,
    // counter 5 -> 6 (no ulp edge).
    s = State(row=3, counter=5.0)
    tick(k, s, 0.8, 0.0, ai2=100.0, facing_p=True, can_hit=True)
    assert (s.row, s.counter) == (3, 6.0)
    # vx 0.8: 0.2f + 0.8f == 1.0f EXACTLY — the >8 step every 9th
    # tick (the counter RESETS on the step).
    assert rows_over(k, 0.8, 0.0, 18) == [
        1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3]
    s = State(row=7, counter=8.0)
    tick(k, s, 0.8, 0.0)
    assert (s.row, s.counter) == (1, 0.0)
    s = State(row=5, counter=7.0, sprite=1)
    tick(k, s, 0.0, 0.0, direction=-1)
    assert (s.row, s.counter, s.sprite) == (0, 0.0, -1)
    s = State(row=5, counter=7.0, sprite=1)
    tick(k, s, 3.0, 2.0, direction=-1)
    assert (s.row, s.counter, s.sprite) == (0, 0.0, 1)
    s = State(sprite=1)
    tick(k, s, 0.8, 0.0, direction=0)
    assert s.sprite == 0
    k = 632
    assert rows_over(k, 1.0, 0.0, 15) == [
        2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5]
    s = State(row=15, counter=11.0)
    tick(k, s, 1.0, 0.0)
    assert (s.row, s.counter) == (2, 0.0)
    k = 635
    assert rows_over(k, 1.0, 0.0, 10) == [2, 2, 3, 3, 3, 4, 4, 4, 5, 5]
    s = State(row=15, counter=5.0)
    tick(k, s, 1.0, 0.0)
    assert (s.row, s.counter) == (2, 0.0)
    for k in (632, 635):
        s = State(row=5, counter=7.0, sprite=-1)
        tick(k, s, 0.0, 0.0, direction=0)
        assert (s.row, s.counter, s.sprite) == (0, 0.0, -1)
        s = State(row=5, counter=7.0, sprite=1)
        tick(k, s, 3.0, -2.0, direction=-1)
        assert (s.row, s.counter, s.sprite) == (1, 0.0, 1)
        s = State(sprite=1)
        tick(k, s, -1.0, 0.0, direction=-1)
        assert s.sprite == -1


def sweep():
    # The 5-kind x 27-combo x 40-tick strip sweep (the module's
    # f26s7g_the_5_kind_strip_sweep_stays_in_sheet twin).
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
    # The golem pose-zone sweep: ai2 68/84/99 x the 8 flag combos —
    # every row inside the 21-row sheet, the counter never touched.
    poses = 0
    for ai2 in (68.0, 84.0, 99.0):
        for f in (False, True):
            for o in (False, True):
                for c in (False, True):
                    s = State(row=3, counter=5.0)
                    tick(631, s, 1.0, 0.0, ai2=ai2, facing_p=f,
                         overlap=o, can_hit=c)
                    assert 0 <= s.row < 21, (ai2, f, o, c, s.row)
                    assert s.counter == 5.0
                    poses += 1
    assert poses == 24  # 3 x 8


if __name__ == "__main__":
    check_vectors()
    sweep()
    print("f26 slice 7g: all replay vectors + the 135-combo sweep "
          "+ the 24-pose sweep PASS")
