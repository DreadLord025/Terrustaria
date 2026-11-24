# W4-f26 slice 7h replay-sweep witness — WRITTEN NOT RUN (code-only).
#
# Line-for-line Python transcriptions of the two C# FindFrame bodies
# this slice ports (npc_findframe_extra7h.rs), the hand-derived replay
# vectors pinned in the module's #[cfg(test)] witnesses, and the
# 8-kind x 27-combo x 40-tick strip sweep + the Hallow Boss phase
# sweep. Run later with:
#     python tools/_f26_7h_replay.py
#
# Sources (decompiled/Terraria/NPC.cs):
#   case 636                :67022-67024 (the one-line phase pin;
#                           AI_120_HallowBoss_IsInPhase2 :45814-45821
#                           = ai[3] == 1 || ai[3] == 3, PURE)
#   case 299/538/539/639-645 :76458-76512 (only 639-645 this wave —
#                           the VERBATIM f27 slice-10 squirrel
#                           machine: the |vx|*0.5+1 >6 walk with the
#                           count-1 wrap to row 1, the air rows 2/4)
# Frame counts Main.cs:67082 (636 = 2 :67147 entry 6, 639-645 = 6
# :67148 entries 1-7), cross-checked vs render/entities.rs
# NPC_FRAME_COUNT (all 697 slots, mismatches []).
#
# Precision note: the C# frameCounter is a double (NPC.cs:6485) fed by
# float products; this witness rounds each product to f32 (struct)
# then accumulates in Python double — the C# order. The port's f32
# counter diverges only in the last ulp; every pinned cadence here
# uses exact half/integer feeds (vx 2 -> 2.0f*0.5f == 1.0f exactly,
# vx 1 -> 1.5 exact) so every step edge is exact in both.

import struct


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


class State:
    def __init__(self, row=0, counter=0.0, sprite=0):
        self.row = row
        self.counter = counter
        self.sprite = sprite


def hallow_boss(s, ai3):
    # NPC.cs:67022-67024 (case 636). The phase read :45814-45821 is
    # PURE ai[3]: ai[3] == 1f || ai[3] == 3f. The arm writes ONLY
    # frame.Y — the counter/sprite UNTOUCHED.
    phase2 = (ai3 == 1.0 or ai3 == 3.0)             # :45816-45820
    s.row = 1 if phase2 else 0                      # :67023 num : 0


def gem_squirrel(s, vx, vy, direction, count=6):
    # NPC.cs:76458-76512 (case 299/538/539/639-645; only 639-645
    # here — the VERBATIM f27 slice-10 machine). The aiStyle-3
    # backpedal branch :76478-76483 is DEAD (all seven aiStyle 7,
    # :17226-17236) — not carried, the documented dead branch.
    if vy == 0.0:                                   # :76468
        if direction == 1:                          # :76470
            s.sprite = 1                            # :76472
        if direction == -1:                         # :76474
            s.sprite = -1                           # :76476
        if vx == 0.0:                               # :76484
            s.row = 0                               # :76486
            s.counter = 0.0                         # :76487
            return                                  # :76488 break
        s.counter += f32(abs(vx) * f32(0.5))        # :76490
        s.counter += 1.0                            # :76491 ++
        if s.counter > 6.0:                         # :76492 STRICT >
            s.row += 1                              # :76494 (Y += num)
            s.counter = 0.0                         # :76495
        if s.row >= count - 1:                      # :76497
            s.row = 1                               # :76499 (Y = num)
    elif vy < 0.0:                                  # :76502
        s.counter = 0.0                             # :76504
        s.row = 2                                   # :76505 num * 2
    else:                                           # :76507 vy > 0
        s.counter = 0.0                             # :76509
        s.row = 4                                   # :76510 num * 4


KINDS = {636: ("boss", 2)}
for _id in range(639, 646):
    KINDS[_id] = ("squirrel", 6)


def tick(kind, s, vx, vy, direction=1, ai3=0.0):
    machine, count = KINDS[kind]
    if machine == "boss":
        hallow_boss(s, ai3)
    else:
        gem_squirrel(s, vx, vy, direction, count)


def rows_over(kind, vx, vy, ticks, direction=1, ai3=0.0):
    s = State()
    out = []
    for _ in range(ticks):
        tick(kind, s, vx, vy, direction, ai3)
        out.append(s.row)
    return out


def check_vectors():
    # The module's pinned replays (f26s7h_* tests), hand-derived and
    # re-checked line-for-line above.
    for ai3, row in [(0.0, 0), (1.0, 1), (2.0, 0), (3.0, 1),
                     (4.0, 0), (-1.0, 0), (1.5, 0)]:
        s = State(row=1, counter=7.0, sprite=-1)
        tick(636, s, 3.0, -2.0, direction=-1, ai3=ai3)
        assert (s.row, s.counter, s.sprite) == (row, 7.0, -1), ai3
    for k in range(639, 646):
        # vx 2: 2.0f*0.5f == 1.0f, ++ -> +2.0/tick, the >6 step every
        # 3rd tick; NO lift (0 -> 1), the >=5 wrap back to row 1.
        assert rows_over(k, 2.0, 0.0, 16) == [
            0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 1, 1], k
        s = State(row=4, counter=6.0)
        tick(k, s, 2.0, 0.0)
        assert (s.row, s.counter) == (1, 0.0), k
    k = 639
    # vx 1: 1.5/tick, the >6 step every 4th tick (6.0 NOT > 6.0).
    assert rows_over(k, 1.0, 0.0, 12) == [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3]
    s = State(row=2, counter=4.5)
    tick(k, s, 1.0, 0.0)
    assert (s.row, s.counter) == (2, 6.0)
    for k in range(639, 646):
        s = State(row=3, counter=5.0, sprite=-1)
        tick(k, s, 0.0, 0.0, direction=0)
        assert (s.row, s.counter, s.sprite) == (0, 0.0, -1), k
        s = State(row=1, counter=5.0, sprite=1)
        tick(k, s, 2.0, -2.0, direction=-1)
        assert (s.row, s.counter, s.sprite) == (2, 0.0, 1), k
        s = State(row=1, counter=5.0, sprite=-1)
        tick(k, s, -2.0, 2.0, direction=1)
        assert (s.row, s.counter, s.sprite) == (4, 0.0, -1), k
        s = State(sprite=1)
        tick(k, s, -2.0, 0.0, direction=-1)
        assert s.sprite == -1, k


def sweep():
    # The 8-kind x 27-combo x 40-tick strip sweep (the module's
    # f26s7h_the_8_kind_strip_sweep_stays_in_sheet twin).
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
    assert combos == 216  # 8 kinds x 27 combos
    # The Hallow Boss phase sweep: ai3 -1/0/1/2/3/4 x 18 velocity/
    # direction combos — the pure phase pin, counter/sprite held.
    phases = 0
    for ai3 in (-1.0, 0.0, 1.0, 2.0, 3.0, 4.0):
        for vx in (0.0, 2.0, -2.0):
            for vy in (0.0, -2.0, 2.0):
                for direction in (1, -1):
                    s = State(row=1, counter=5.0, sprite=-1)
                    tick(636, s, vx, vy, direction, ai3=ai3)
                    assert s.row == (1 if ai3 in (1.0, 3.0) else 0)
                    assert (s.counter, s.sprite) == (5.0, -1)
                    phases += 1
    assert phases == 108  # 6 x 18


if __name__ == "__main__":
    check_vectors()
    sweep()
    print("f26 slice 7h: all replay vectors + the 216-combo sweep "
          "+ the 108-phase sweep PASS")
