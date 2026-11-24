#!/usr/bin/env python3
"""W4-f12 closeout item 4 — the FINAL UpdateBuffs keep-one re-sweep.

Script-diffs the C# `Player.UpdateBuffs` slot-loop case-list
(`decompiled/Terraria/Player.cs:9875-12044`) against EVERY ported
ownership set + the wave-E dispatch surface, then checks pairwise
disjointness (one live grant path per id). Re-run of the sweep-phase
census against the landed d73/f50/d115/f49 tips — the question is whether
any NEW dup-grant appeared. Code-only: no cargo.

Usage: python tools/w4_f12_closeout_sweep.py   (from the repo root)
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CS = ROOT / "decompiled" / "Terraria" / "Player.cs"
SRC = ROOT / "crates" / "tr-app" / "src"


def read(p):
    return p.read_text(encoding="utf-8", errors="replace")


def cs_heads():
    """The 270-id case-list: == heads (incl. the fairy trio's one line)
    plus the six 3-id range heads."""
    text = read(CS).splitlines()
    region = text[9874:12044]  # :9875-12044
    ids = set()
    ranges = []
    for line in region:
        for m in re.finditer(r"buffType\[j\]\s*==\s*(\d+)", line):
            ids.add(int(m.group(1)))
        m = re.search(
            r"buffType\[j\]\s*>=\s*(\d+)\s*&&\s*this\.buffType\[j\]\s*<=\s*(\d+)", line
        )
        if m:
            lo, hi = int(m.group(1)), int(m.group(2))
            ranges.append((lo, hi))
            ids.update(range(lo, hi + 1))
    return ids, ranges


def const_set(name):
    text = read(SRC / "buff_frame_wiring.rs")
    m = re.search(name + r":\s*&\[i32\]\s*=\s*&\[(.*?)\];", text, re.S)
    assert m, name
    return {int(x) for x in re.findall(r"\d+", m.group(1))}


def inline_owned():
    return const_set("INLINE_OWNED")


def status_ids():
    text = read(SRC / "buff_effects_status.rs")
    pets = {int(x) for x in re.findall(r"StatusPetArm \{ buff: (\d+)", text)}
    body = text.split("pub fn apply_status_effect_arm", 1)[1]
    body = body.split("#[cfg(test)]", 1)[0]
    arms = {int(x) for x in re.findall(r"^\s*(\d+) =>", body, re.M)}
    return pets | arms


def summon_ids():
    text = read(SRC / "player.rs")
    block = text.split("const SUMMON_BUFF_ARMS", 1)[1].split("];", 1)[0]
    keep = {int(x) for x in re.findall(r"buff: (\d+)", block)}
    return keep | {110, 150, 348}  # the cap rows the live pass owns


def accessory_ids():
    # The registry is the sweep-phase verified set; cross-check it against
    # the module's actual dispatch surface (explicit match arms + the
    # PET_BUFF_ARMS / IMBUE_ARMS tables). 376 is the documented DEFERRED
    # no-op arm (Torch God's Favor — the module header records it).
    reg = const_set("ACCESSORY_OWNED")
    text = read(SRC / "buff_effects_accessory.rs")
    body = text.split("pub fn update_accessory_buff_arms", 1)[1]
    body = body.split("#[cfg(test)]", 1)[0]
    # OUTER match arms only (12-space indent) — the nested Rabid roll
    # match (0-4) is not a buff-id arm.
    arms = set()
    for m in re.finditer(r"^ {12}(\d+(?: \| \d+)*) =>", body, re.M):
        arms |= {int(x) for x in m.group(1).split(" | ")}
    for table in ("PET_BUFF_ARMS", "IMBUE_ARMS"):
        block = text.split(table + ":", 1)[1].split("];", 1)[0]
        arms |= {int(x) for x in re.findall(r"\((\d+),", block)}
    # 146 Sunflower is the documented HALF-SPLIT co-ownership: the module
    # arms only `sunflower = true`, the movement half is INLINE_OWNED
    # (module header :20-22) — not a dup-grant.
    missing = arms - reg - {146}
    extra = reg - arms
    assert not missing, f"module arms not registered: {sorted(missing)}"
    assert extra <= {376}, f"registered ids with no module arm: {sorted(extra)}"
    return reg


def mount_ids():
    text = read(SRC / "mount.rs")
    # Named MountData consts carry `buff: N`; the horse()/cart() helpers
    # take (id, buff, sheet) positionally.
    ids = {int(x) for x in re.findall(r"buff: (\d+),", text)}
    ids |= {
        int(m.group(1))
        for m in re.finditer(
            r"(?:horse|cart|cart_vo|roller_skate|chillet)\(\d+, (\d+)[,)]", text
        )
    }
    ids.discard(0)  # BASE's placeholder
    return ids


def main():
    cs, ranges = cs_heads()
    inline = inline_owned()
    acc = accessory_ids()
    summon = summon_ids()
    debuff = const_set("DEBUFF_DISPATCH")
    misc = const_set("MISC_DISPATCH")
    status = status_ids()
    boosters = {192, 321, 396, 62}
    orbs = set(range(95, 101)) | set(range(170, 182))
    mounts = mount_ids()
    other = {103, 11, 16, 2}  # Wet / Shine / Archery / Regen lanes

    print(f"C# heads: {len(cs)} ids, {len(ranges)} range heads {ranges}")
    assert len(cs) == 270, f"expected 270, got {len(cs)}"
    assert len(ranges) == 6

    sets = {
        "INLINE_OWNED": inline,
        "ACCESSORY_OWNED": acc,
        "SUMMON_OWNED": summon,
        "DEBUFF_DISPATCH": debuff,
        "MISC_DISPATCH": misc,
        "wave-E status": status,
        "d73 non-orb": boosters,
        "g27 orbs": orbs,
        "mount head": mounts,
        "other lanes": other,
    }
    for name, s in sets.items():
        print(f"{name}: {len(s)}")

    # Pairwise disjointness among the DISPATCH paths, with the two
    # documented HALF-SPLIT co-ownerships excluded (each half applies a
    # DIFFERENT stat/flag — not a dup-grant):
    #   146 Sunflower: inline movement (`player.rs` `moveSpeed += 0.1;
    #       *= 1.1`) + accessory flag (`sunflower = true`) — module
    #       header :20-22 records the split.
    #   131/168/265/279/305: the mount head's five swim sub-arms are
    #       inline-owned (the mount re-affirm is the head's own write).
    half_split = {("INLINE_OWNED", "ACCESSORY_OWNED"): {146}}
    dispatch = {k: v for k, v in sets.items() if k != "mount head"}
    names = list(dispatch)
    dup = []
    for i, a in enumerate(names):
        for b in names[i + 1 :]:
            both = dispatch[a] & dispatch[b]
            both -= half_split.get((a, b), set()) | half_split.get((b, a), set())
            if both:
                dup.append((a, b, sorted(both)))
    for a, b, both in dup:
        print(f"OVERLAP {a} ∩ {b}: {both}")
    assert not dup, "dup-grant: an id sits in two dispatch paths"

    # The mount head's overlap with inline is exactly the five swim rows.
    swim = mounts & inline
    assert swim == {131, 168, 265, 279, 305}, swim
    assert not (mounts & (acc | summon | debuff | misc | status | boosters | orbs))

    # Coverage: every C# id has an owner.
    owned = set().union(*sets.values())
    missing = cs - owned
    extra = (owned - cs) - mounts  # mount rows outside the explicit list
    print(f"missing owners: {sorted(missing)}")
    print(f"extra owned ids (non-mount): {sorted(extra)}")
    assert not missing, "a C# arm has NO owner"
    assert not extra, "an owned id has NO C# arm"
    print("VERDICT: set-diff EMPTY, zero dup-grants — keep-one HOLDS")


if __name__ == "__main__":
    sys.exit(main())
