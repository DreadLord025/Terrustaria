# W4-f27 slice 11 replay-sweep witness — WRITTEN NOT RUN (code-only).
#
# Line-for-line Python transcriptions of the C# FindFrame bodies this
# slice ports/fixes (npc_findframe_extra11.rs), the hand-derived replay
# vectors pinned in the module's #[cfg(test)] witnesses, and the
# 15-kind strip sweep. Run later with:
#     python tools/_f27_s11_replay.py
#
# Sources (decompiled/Terraria/NPC.cs):
#   case 430..436/591        :70879-70951 (435/436 this wave — the
#                            VERBATIM d49 armed-zombie machine)
#   case 550 (humanoid town) :68616 -> :68647-69655 (num57 = 9
#                            NPCID.cs:10464, num108 = 6, air row 1;
#                            the ai[0] pose zones :68662-69488 skipped)
#   case 552/553/554         :67452-67492 (the ai[0] attack ladder
#                            rows 10-13 + the >=64 walk reset)
#   case 555/556/557         :67797-67837 (the ai[1]>0 && ai[0]>0
#                            ladder rows 10-16 + the +2 walk strip)
#   case 558/559/560         :67838-67873 (the ToRotation aim +PI,
#                            the ai[0]!=2 vx*0.1 override, the dive
#                            row-4 pin / the >=6 full-4 step)
#   case 561/562/563         :67756-67796 (the goblin shape, the
#                            ai[1]==89 reset)
# Frame counts Main.cs:67082 (:67127's 430-439 row: 435/436 = 7;
# :67139's 550-559 row: 550=25, 552-554=14, 555-557=17, 558-560=5;
# :67140's 560-569 row: 561-563=14), cross-checked vs
# render/entities.rs NPC_FRAME_COUNT (all 697 slots).
#
# Precision note: the C# frameCounter is a double (NPC.cs:6485) fed by
# float products; every accumulator this wave is an INTEGER step
# (|vx| integer picks, ++, *2+1) so the f32 port and the double replay
# agree exactly on every pinned vector. The wyvern rotation is the one
# f32 transcendental (atan2 + PI) — the sweep checks rows only.

import math
import struct


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


class State:
    def __init__(self, row=0, counter=0.0, sprite=0, rotation=0.0):
        self.row = row
        self.counter = counter
        self.sprite = sprite
        self.rotation = rotation


def armed_zombie(s, vx, vy, direction, ai2):
    # NPC.cs:70879-70951 (case 430..436/591; 435/436 this wave).
    if ai2 > 0:  # :70887-70905
        if ai2 < 5:
            s.row = 3
        elif ai2 < 10:
            s.row = 4
        elif ai2 < 15:
            s.row = 5
        else:
            s.row = 6
        return
    if vy == 0:  # :70907-70917
        if direction == 1:
            s.sprite = 1
        if direction == -1:
            s.sprite = -1
    if vy != 0 or (direction == -1 and vx > 0) or (direction == 1 and vx < 0):
        s.counter = 0.0  # :70918-70923
        s.row = 2
        return
    if vx == 0:  # :70924-70929
        s.counter = 0.0
        s.row = 0
        return
    s.counter += abs(vx)  # :70930-70950
    if s.counter < 8:
        s.row = 0
    elif s.counter < 16:
        s.row = 1
    elif s.counter < 24:
        s.row = 2
    elif s.counter < 32:
        s.row = 1
    else:
        s.counter = 0.0  # the row NOT written (holds)


def bartender(s, vx, vy, direction, count=25):
    # NPC.cs:68616 -> :68647-69655 (the town walk, num57 = 9).
    if vy == 0:
        if direction == 1:  # :68649-68656
            s.sprite = 1
        if direction == -1:
            s.sprite = -1
        if vx == 0:  # :69515-69519
            s.row = 0
            s.counter = 0.0
        else:
            if s.row < 2:  # :69572-69575 (num109 = row 2)
                s.row = 2
            s.counter += abs(vx) * 2.0  # :69552-69553
            s.counter += 1.0
            if s.counter > 6:  # :69576-69580 (num108 = 6)
                s.row += 1
                s.counter = 0.0
            if s.row >= count - 9:  # :69581-69584
                s.row = 2
    else:
        s.counter = 0.0  # :69642-69646 (550 not in the :69647 list)
        s.row = 1


def goblin(s, vx, vy, direction, ai0, ai1):
    # NPC.cs:67452-67492 (case 552/553/554).
    if ai0 > 0:  # :67455-67469
        s.sprite = direction
        if s.row < 10 or s.row > 13 or ai1 == 29:
            s.row = 10
            s.counter = 0.0
        s.counter += 1.0
        if s.counter >= 6 and s.row < 13:
            s.counter = 0.0
            s.row += 1
        return
    if vy == 0:  # :67470-67473
        s.sprite = direction
    if vy != 0 or (direction == -1 and vx > 0) or (direction == 1 and vx < 0):
        s.counter = 0.0  # :67474-67479
        s.row = 8
        return
    if vx == 0:  # :67480-67485
        s.counter = 0.0
        s.row = 9
        return
    s.counter += abs(vx)  # :67486-67491
    if s.counter >= 64 or s.counter < 0:
        s.counter = 0.0
    s.row = int(s.counter / 8.0)


def bomber(s, vx, vy, direction, ai0, ai1):
    # NPC.cs:67797-67837 (case 555/556/557).
    if ai1 > 0 and ai0 > 0:  # :67800-67814
        s.sprite = direction
        if s.row < 10 or s.row > 16 or ai1 == 41:
            s.row = 10
            s.counter = 0.0
        s.counter += 1.0
        if s.counter >= 6 and s.row < 16:
            s.counter = 0.0
            s.row += 1
        return
    if vy == 0:  # :67815-67818
        s.sprite = direction
    if vy != 0 or (direction == -1 and vx > 0) or (direction == 1 and vx < 0):
        s.counter = 0.0  # :67819-67824
        s.row = 1
        return
    if vx == 0:  # :67825-67830
        s.counter = 0.0
        s.row = 0
        return
    s.counter += abs(vx)  # :67831-67836
    if s.counter >= 64 or s.counter < 0:
        s.counter = 0.0
    s.row = int(s.counter / 8.0) + 2


def wyvern(s, vx, vy, ai0):
    # NPC.cs:67838-67873 (case 558/559/560).
    if vx > 0:  # :67842-67849
        s.sprite = 1
    if vx < 0:
        s.sprite = -1
    num51 = f32(math.atan2(vy, vx))  # :67850 (Utils.cs:1655 ToRotation)
    if vx < 0:  # :67851-67854 — the REAL PI (not the eye 3.14f)
        num51 = f32(num51 + f32(math.pi))
    if ai0 != 2:  # :67855-67858
        num51 = f32(vx * f32(0.1))
    s.rotation = num51  # :67859
    if ai0 == 2:  # :67860-67863
        s.row = 4
    else:
        s.counter += 1.0  # :67864-67872
        if s.counter >= 6:
            s.row += 1
            s.counter = 0.0
            if s.row >= 4:
                s.row = 0


def javelin(s, vx, vy, direction, ai0, ai1):
    # NPC.cs:67756-67796 (case 561/562/563) — the goblin shape.
    if ai1 > 0 and ai0 > 0:  # :67759-67773
        s.sprite = direction
        if s.row < 10 or s.row > 13 or ai1 == 89:
            s.row = 10
            s.counter = 0.0
        s.counter += 1.0
        if s.counter >= 6 and s.row < 13:
            s.counter = 0.0
            s.row += 1
        return
    if vy == 0:  # :67774-67777
        s.sprite = direction
    if vy != 0 or (direction == -1 and vx > 0) or (direction == 1 and vx < 0):
        s.counter = 0.0  # :67778-67783
        s.row = 8
        return
    if vx == 0:  # :67784-67789
        s.counter = 0.0
        s.row = 9
        return
    s.counter += abs(vx)  # :67790-67795
    if s.counter >= 64 or s.counter < 0:
        s.counter = 0.0
    s.row = int(s.counter / 8.0)


def rows_over(fn, ticks, **kw):
    s = State()
    out = []
    for _ in range(ticks):
        fn(s, **kw)
        out.append(s.row)
    return out


# ---- the pinned vectors (the module witnesses' expectations) ----

# 435/436 vx 2: the bands + the >=32 reset-tick row-hold.
assert rows_over(armed_zombie, 17, vx=2.0, vy=0.0, direction=1, ai2=0.0) == [
    0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0
]
# The reset-tick hold: row 1 + counter 30 -> 32 zeroes, row stays 1.
s = State(row=1, counter=30.0)
armed_zombie(s, vx=2.0, vy=0.0, direction=1, ai2=0.0)
assert (s.row, s.counter) == (1, 0.0)

# 550 vx 1: the >6 step every 3rd tick, the lift-to-2.
assert rows_over(bartender, 12, vx=1.0, vy=0.0, direction=1) == [
    2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6
]
# The wrap: row 15 + counter 6 -> step 16 -> wrap 2.
s = State(row=15, counter=6.0)
bartender(s, vx=1.0, vy=0.0, direction=1)
assert (s.row, s.counter) == (2, 0.0)

# 552/553 vx 4: the counter/8 row + the >=64 reset tick row 0.
assert rows_over(goblin, 16, vx=4.0, vy=0.0, direction=1, ai0=0.0, ai1=0.0) == [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0
]
# The attack ladder: rows 10(x5)/11(x6)/12(x6)/13.
assert rows_over(goblin, 20, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=0.0) == [
    10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 13, 13, 13
]
# The row-13 counter-keep-growing quirk.
s = State(row=13, counter=5.0)
goblin(s, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=0.0)
assert (s.row, s.counter) == (13, 6.0)
# The ai[1] == 29 reset: row 10, counter 0 THEN ++ -> 1.
s = State(row=12, counter=3.0)
goblin(s, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=29.0)
assert (s.row, s.counter) == (10, 1.0)

# 555-557 vx 4: the +2 strip, the >=64 reset tick row 2.
assert rows_over(bomber, 16, vx=4.0, vy=0.0, direction=1, ai0=0.0, ai1=0.0) == [
    2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 2
]
# The attack ladder: rows 10(x5)/11..15(x6 each)/16.
assert rows_over(bomber, 40, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=1.0) == [
    10, 10, 10, 10, 10,
    11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12,
    13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14,
    15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16,
]

# 558-560: the >=6 step over rows 0-3 (the 5th row is the dive pin).
assert rows_over(wyvern, 20, vx=1.0, vy=0.0, ai0=0.0) == [
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3
]
# The dive pin: ai0 == 2 -> row 4, counter UNTOUCHED, rotation the aim.
s = State(row=1, counter=3.0)
wyvern(s, vx=-2.0, vy=-2.0, ai0=2.0)
assert (s.row, s.counter, s.sprite) == (4, 3.0, -1)
assert abs(s.rotation - f32(f32(math.atan2(-2.0, -2.0)) + f32(math.pi))) < 1e-6
# The non-dive override: rotation = vx*0.1 (f32 product).
s = State()
wyvern(s, vx=3.0, vy=9.0, ai0=0.0)
assert s.rotation == f32(3.0 * f32(0.1))

# 561-563: the goblin shape, the ai[1] == 89 reset.
assert rows_over(javelin, 16, vx=4.0, vy=0.0, direction=1, ai0=0.0, ai1=0.0) == [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0
]
s = State(row=12, counter=3.0)
javelin(s, vx=0.0, vy=0.0, direction=1, ai0=1.0, ai1=89.0)
assert (s.row, s.counter) == (10, 1.0)

# ---- the 15-kind strip sweep (40 ticks, every row inside its sheet) ----

COUNTS = {
    "armed_zombie": 7, "bartender": 25, "goblin": 14,
    "bomber": 17, "wyvern": 5, "javelin": 14,
}
violations = 0
combos = 0
for vx in (-4.0, -1.0, 0.0, 1.0, 4.0):
    for vy in (-3.0, 0.0, 3.0):
        for direction in (-1, 0, 1):
            for ai0 in (0.0, 1.0, 2.0):
                for ai1 in (0.0, 1.0, 29.0, 41.0, 89.0):
                    for ai2 in (0.0, 3.0, 7.0, 12.0, 20.0):
                        combos += 1
                        machines = [
                            ("armed_zombie", lambda s: armed_zombie(s, vx, vy, direction, ai2)),
                            ("bartender", lambda s: bartender(s, vx, vy, direction)),
                            ("goblin", lambda s: goblin(s, vx, vy, direction, ai0, ai1)),
                            ("bomber", lambda s: bomber(s, vx, vy, direction, ai0, ai1)),
                            ("wyvern", lambda s: wyvern(s, vx, vy, ai0)),
                            ("javelin", lambda s: javelin(s, vx, vy, direction, ai0, ai1)),
                        ]
                        for name, step in machines:
                            s = State()
                            for tick in range(40):
                                step(s)
                                if s.row >= COUNTS[name]:
                                    violations += 1
                                    print(f"VIOLATION {name} tick {tick} row {s.row} "
                                          f"(vx {vx}, vy {vy}, dir {direction}, "
                                          f"ai0 {ai0}, ai1 {ai1}, ai2 {ai2})")
print(f"sweep: {combos} combos x 6 machines x 40 ticks, {violations} violations")
assert violations == 0
print("all pinned vectors + the sweep agree with the C# transcriptions")
