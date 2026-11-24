# W4-f26 slice 7e replay-sweep witness — WRITTEN NOT RUN (code-only).
#
# Line-for-line Python transcriptions of the three C# FindFrame bodies
# this slice ports (npc_findframe_extra7e.rs), the hand-derived replay
# vectors pinned in the module's #[cfg(test)] witnesses, and the
# 5-kind x 27-combo x 10-tick strip sweep. Run later with:
#     python tools/_f26_7e_replay.py
#
# Sources (decompiled/Terraria/NPC.cs):
#   case 524/525/526/527  :69908-69945  (only 526/527 this wave)
#   case 528/529         :69946-69977
#   case 164/236/239/530 :73083-73111  (only 530 this wave)
# Frame counts Main.cs:67082 (:67136 entries 7/8 = 8, entries 9/10 = 9;
# :67137 entry 1 = 5), cross-checked vs render/entities.rs NPC_FRAME_COUNT.
#
# Precision note: the C# frameCounter is a double (NPC.cs:6485) fed by
# float products (velocity.X is float, * 1.1f is a float multiply);
# this witness rounds each product to f32 (struct) then accumulates in
# Python double — the C# order. The port's f32 counter diverges only
# in the last ulp; every pinned vector sits far from a >= / > boundary
# (the nearest is the scorpion vx-3 tick-7 counter ~5.1 vs 6).

import struct


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


class State:
    def __init__(self, row=0, counter=0.0, sprite=0, direction=0):
        self.row = row
        self.counter = counter
        self.sprite = sprite
        self.direction = direction


def ghoul(s, vx, vy, count=8):
    # NPC.cs:69908-69945 (the verbatim 7d machine).
    if vy == 0.0:                                   # :69912
        if s.direction != 0:                        # :69914
            s.sprite = s.direction                  # :69916
        if vx == 0.0:                               # :69918
            s.row = 0                               # :69920
            s.counter = 0.0                         # :69921
            return                                  # :69922 break
        if s.row <= 1:                              # :69924 (row <= num)
            s.row = 2                               # :69926
        s.counter += abs(vx)                        # :69928
        s.counter += 1.0                            # :69929 ++
        if s.counter > 9.0:                         # :69930 STRICT >
            s.row += 1                              # :69932
            s.counter = 0.0                         # :69933
        if s.row >= count:                          # :69935
            s.row = 2                               # :69937
    else:                                           # :69940
        s.row = 1                                   # :69942
        s.counter = 0.0                             # :69943


def lamia(s, vx, vy, count=9):
    # NPC.cs:69946-69977 — the ghoul shape MINUS the lift-to-2, wrap
    # at count-1 back to 1, air pins count-1.
    if vy == 0.0:                                   # :69948
        if s.direction != 0:                        # :69950
            s.sprite = s.direction                  # :69952
        if vx == 0.0:                               # :69954
            s.row = 0                               # :69956
            s.counter = 0.0                         # :69957
            return                                  # :69958 break
        s.counter += abs(vx)                        # :69960 (NO lift)
        s.counter += 1.0                            # :69961 ++
        if s.counter > 9.0:                         # :69962 STRICT >
            s.row += 1                              # :69964
            s.counter = 0.0                         # :69965
        if s.row >= count - 1:                      # :69967
            s.row = 1                               # :69969
    else:                                           # :69972
        s.row = count - 1                           # :69974
        s.counter = 0.0                             # :69975


def scorpion(s, vx, vy):
    # NPC.cs:73083-73111 (case 164/236/239/530; only 530 this wave).
    if vy != 0.0:                                   # :73087
        s.counter = 0.0                             # :73089 FIRST
        if vy < 0.0:                                # :73090
            s.row = 4                               # :73092
        else:
            s.row = 0                               # :73096
        return                                      # :73098 break
    s.sprite = s.direction                          # :73100 UNCONDITIONAL
    s.counter += f32(abs(vx) * f32(1.1))            # :73101 |vx|*1.1 NO +1
    while s.counter >= 6.0:                         # :73102 WHILE
        s.counter -= 6.0                            # :73104
        s.row += 1                                  # :73105
        if s.row > 3:                               # :73106 STRICT >
            s.row = 0                               # :73108


KINDS = {
    526: (ghoul, 8),
    527: (ghoul, 8),
    528: (lamia, 9),
    529: (lamia, 9),
    530: (scorpion, 5),
}


def tick(kind, s, vx, vy):
    fn, count = KINDS[kind]
    if fn is scorpion:
        fn(s, vx, vy)
    else:
        fn(s, vx, vy, count)


def rows_over(kind, vx, vy, ticks):
    s = State()
    out = []
    for _ in range(ticks):
        tick(kind, s, vx, vy)
        out.append(s.row)
    return out


def check_vectors():
    # The module's pinned replays (f26s7e_* tests), hand-derived and
    # re-checked line-for-line above.
    for k in (526, 527):
        assert rows_over(k, 3.0, 0.0, 19) == [
            2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 2, 2]
        assert rows_over(k, 1.0, 0.0, 22) == [
            2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6]
        s = State(row=1, sprite=1, direction=-1)
        tick(k, s, -3.0, 0.0)
        assert (s.row, s.counter, s.sprite) == (2, 4.0, -1)
        s = State(row=5, counter=7.0, sprite=-1, direction=0)
        tick(k, s, 0.0, 0.0)
        assert (s.row, s.counter, s.sprite) == (0, 0.0, -1)
        s = State(row=5, counter=7.0, sprite=1, direction=-1)
        tick(k, s, 3.0, 2.0)
        assert (s.row, s.counter, s.sprite) == (1, 0.0, 1)
    for k in (528, 529):
        assert rows_over(k, 3.0, 0.0, 25) == [
            0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4,
            5, 5, 5, 6, 6, 6, 7, 7, 7, 1, 1]
        s = State(row=7, counter=9.0)
        tick(k, s, 3.0, 0.0)
        assert (s.row, s.counter) == (1, 0.0)
        s = State(row=5, counter=7.0, sprite=-1, direction=0)
        tick(k, s, 0.0, 0.0)
        assert (s.row, s.counter, s.sprite) == (0, 0.0, -1)
        s = State(row=5, counter=7.0, sprite=1, direction=-1)
        tick(k, s, 3.0, -2.0)
        assert (s.row, s.counter, s.sprite) == (8, 0.0, 1)
        s = State(row=0, sprite=1, direction=-1)
        tick(k, s, -3.0, 0.0)
        assert (s.row, s.counter, s.sprite) == (0, 4.0, -1)
    k = 530
    assert rows_over(k, 3.0, 0.0, 12) == [0, 1, 1, 2, 2, 3, 3, 0, 0, 1, 2, 2]
    assert rows_over(k, 1.0, 0.0, 13) == [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2]
    s = State(row=2, counter=5.0, sprite=1, direction=-1)
    tick(k, s, 3.0, -2.0)
    assert (s.row, s.counter, s.sprite) == (4, 0.0, 1)
    s = State(row=2, counter=5.0, sprite=1, direction=-1)
    tick(k, s, 3.0, 2.0)
    assert (s.row, s.counter, s.sprite) == (0, 0.0, 1)
    s = State(row=3, counter=0.0)
    tick(k, s, 10.0, 0.0)
    assert (s.row, s.counter) == (0, 5.0)
    s = State(row=3, counter=0.0)
    tick(k, s, 20.0, 0.0)
    assert (s.row, s.counter) == (2, 4.0)
    s = State(row=2, counter=5.9, sprite=1, direction=0)
    tick(k, s, 0.0, 0.0)
    assert (s.row, s.counter, s.sprite) == (2, 5.9, 0)
    s = State(sprite=1, direction=-1)
    tick(k, s, -3.0, 0.0)
    assert s.sprite == -1


def sweep():
    # The 5-kind x 27-combo x 10-tick strip sweep: every row stays
    # inside its own sheet (the module's
    # f26s7e_the_5_kind_strip_sweep_stays_in_sheet twin).
    combos = 0
    for k, (_, count) in KINDS.items():
        for vx in (0.0, 1.0, 3.0):
            for vy in (0.0, -2.0, 2.0):
                for direction in (1, -1, 0):
                    s = State(direction=direction)
                    for _ in range(10):
                        tick(k, s, vx, vy)
                        assert 0 <= s.row < count, (k, vx, vy, direction, s.row)
                    combos += 1
    assert combos == 135  # 5 kinds x 27 combos


if __name__ == "__main__":
    check_vectors()
    sweep()
    print("f26 slice 7e: all replay vectors + the 135-combo sweep PASS")
