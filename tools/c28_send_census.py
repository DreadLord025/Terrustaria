#!/usr/bin/env python3
"""W4-c28 final: the FULL C# NetMessage.SendData trigger census.

Walks decompiled/Terraria/**/*.cs (direct file paths only, junction-safe),
extracts EVERY NetMessage.SendData(...) call site with its literal msg id,
the enclosing method and the nearest netMode guard, plus the id-named
NetMessage.Send* wrapper bodies (SendTileSquare=20, SendPlayerHurt=26, ...)
and their call-site counts. Output: tools/c28_send_census.md + .json.

Run from the repo root:  python tools/c28_send_census.py
The .cs sources are read-only inputs; the script never writes outside tools/.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "decompiled" / "Terraria"

CALL = re.compile(r"NetMessage\.(TrySendData|SendData)\s*\(")
# id: literal first arg, or named msgType: arg anywhere in the call
NAMED_ID = re.compile(r"msgType\s*:\s*(?:\(int\)\s*)?(?:MessageID\.)?(\w+)")
LIT_ID = re.compile(r"^\s*(?:\(byte\)\s*)?(?:MessageID\.)?([A-Za-z_]\w*|\d+)")
METHOD = re.compile(
    r"^\s*(?:\[[^\]]*\]\s*)*(?:public|private|internal|protected|static|async|override|virtual|sealed|partial|unsafe|extern|new)[\w\s<>,\[\]?]*?\s(\w+)\s*\(")
NETMODE = re.compile(r"Main\.netMode\s*(!=|==)\s*(\d)")


def strip_noise(line: str) -> str:
    # cheap: kill // comments and string literals for guard scans
    line = re.sub(r'"(?:[^"\\]|\\.)*"', '""', line)
    i = line.find("//")
    return line[:i] if i >= 0 else line


def balanced_call(text: str, open_idx: int) -> str:
    depth, i, n = 0, open_idx, len(text)
    while i < n:
        c = text[i]
        if c == '"':
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c == "'":
            i += 1
            while i < n and text[i] != "'":
                i += 2 if text[i] == "\\" else 1
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return text[open_idx + 1 : i]
        i += 1
    return text[open_idx + 1 : open_idx + 200]


def main() -> None:
    # MessageID table (two directions, kept separate)
    name2num, num2name = {}, {}
    mid = (SRC / "ID" / "MessageID.cs").read_text(encoding="utf-8-sig")
    for m in re.finditer(r"public const byte (\w+) = (\d+);", mid):
        name2num[m.group(1)] = m.group(2)
        num2name[m.group(2)] = m.group(1)

    def resolve(raw: str) -> str:
        if raw.isdigit():
            return raw
        if raw == "b":
            return "relay"  # MessageBuffer.GetData echoes the received id
        return name2num.get(raw, "?")

    sites = []          # every SendData call site
    wrappers = {}       # NetMessage.Send* helper bodies -> ids they send
    files = sorted(p for p in SRC.rglob("*.cs"))
    for path in files:
        rel = path.relative_to(SRC).as_posix()
        text = path.read_text(encoding="utf-8-sig", errors="replace")
        lines = text.splitlines()
        # method spans: line -> enclosing method name (last decl seen)
        method_at = []
        cur = "<top>"
        for ln in lines:
            m = METHOD.match(ln)
            if m and m.group(1) not in ("if", "while", "for", "switch", "catch", "using", "lock", "fixed"):
                cur = m.group(1)
            method_at.append(cur)
        # netMode guards: nearest preceding guard within 60 lines
        guards = [(i, mm.group(1), mm.group(2))
                  for i, ln in enumerate(lines)
                  for mm in [NETMODE.search(strip_noise(ln))] if mm]
        for m in CALL.finditer(text):
            line_no = text.count("\n", 0, m.start()) + 1
            if rel == "NetMessage.cs" and line_no == 72:
                continue  # TrySendData's internal forward, not a trigger
            args = balanced_call(text, m.end() - 1)
            nm = NAMED_ID.search(args)
            raw = nm.group(1) if nm else None
            if raw is None:
                lm = LIT_ID.match(args)
                raw = lm.group(1) if lm else "?"
            num = resolve(raw)
            name = num2name.get(num, "?")
            guard = ""
            for gi, op, val in reversed(guards):
                if gi < line_no and line_no - gi <= 60:
                    guard = f"netMode{op}{val}@{gi+1}"
                    break
            sites.append({
                "file": rel, "line": line_no, "id": num, "name": name,
                "raw": raw, "method": method_at[line_no - 1], "guard": guard,
                "api": m.group(1),
            })
        # wrapper bodies in NetMessage.cs: which ids each Send* helper emits
        if rel == "NetMessage.cs":
            for wm in re.finditer(r"public static void (Send\w+)\s*\(", text):
                if wm.group(1) in ("SendData", "TrySendData"):
                    continue  # the dispatchers themselves, not wrappers
                start = wm.end()
                # body = to next 'public static' or 400 lines
                nxt = text.find("public static", start)
                body = text[start : nxt if nxt > 0 else start + 20000]
                inner = set()
                for cm in CALL.finditer(body):
                    a = balanced_call(body, cm.end() - 1)
                    n2 = NAMED_ID.search(a)
                    r2 = n2.group(1) if n2 else (LIT_ID.match(a).group(1) if LIT_ID.match(a) else "?")
                    inner.add(resolve(r2))
                if inner:
                    wrappers[wm.group(1)] = sorted(inner, key=lambda x: (len(x), x))

    # wrapper call-site census (all files)
    wrap_calls = {w: [] for w in wrappers}
    wpat = re.compile(r"NetMessage\.(" + "|".join(wrappers) + r")\s*\(")
    for path in files:
        rel = path.relative_to(SRC).as_posix()
        text = path.read_text(encoding="utf-8-sig", errors="replace")
        for m in wpat.finditer(text):
            line_no = text.count("\n", 0, m.start()) + 1
            wrap_calls[m.group(1)].append(f"{rel}:{line_no}")

    per_id = {}
    for s in sites:
        per_id.setdefault(s["id"], []).append(s)

    # ---- phase 2: the netUpdate / netSpam increment-site census ----
    # C# arms `netUpdate = true` per event; the throttle counters only move
    # inside UpdateNetworkCode / the Projectile tail / the priority helpers.
    inc_pat = re.compile(
        r"(?<![\w.])(\w+(?:\[\w+\])?)\.(netUpdate2?|netSpam|netImportant|netUpdatePendingSpamCooldown|netUpdatePendingFullSpamCooldown|spawnNeedsSyncing)\s*(\+\+|--|\+=|-=|=(?!=))\s*([^;]*);")
    inc_sites = []
    for path in files:
        rel = path.relative_to(SRC).as_posix()
        text = path.read_text(encoding="utf-8-sig", errors="replace")
        lines = text.splitlines()
        method_at = []
        cur = "<top>"
        for ln in lines:
            m = METHOD.match(ln)
            if m and m.group(1) not in ("if", "while", "for", "switch", "catch", "using", "lock", "fixed"):
                cur = m.group(1)
            method_at.append(cur)
        for m in inc_pat.finditer(text):
            line_no = text.count("\n", 0, m.start()) + 1
            inc_sites.append({
                "file": rel, "line": line_no, "recv": m.group(1),
                "field": m.group(2), "op": m.group(3), "rhs": m.group(4).strip()[:40],
                "method": method_at[line_no - 1],
            })
    out = {"per_id_counts": {k: len(v) for k, v in sorted(per_id.items(), key=lambda kv: int(kv[0]) if kv[0].isdigit() else 999)},
           "total_sites": len(sites), "sites": sites,
           "wrappers": wrappers, "wrapper_calls": wrap_calls,
           "increments": inc_sites}
    # the priority-helper call sites (the netSpam bypass lanes)
    helper_calls = []
    hpat = re.compile(r"\.(NetUpdateIgnoreSpamLimit|NetUpdateLowPriority)\s*\(\s*\)")
    for path in files:
        rel = path.relative_to(SRC).as_posix()
        text = path.read_text(encoding="utf-8-sig", errors="replace")
        for m in hpat.finditer(text):
            helper_calls.append({"file": rel,
                                 "line": text.count("\n", 0, m.start()) + 1,
                                 "helper": m.group(1)})
    out["helper_calls"] = helper_calls
    (ROOT / "tools" / "c28_send_census.json").write_text(
        json.dumps(out, indent=1), encoding="utf-8")

    md = ["# W4-c28 — the C# NetMessage.SendData trigger census (generated)",
          "",
          f"Total literal call sites: **{len(sites)}** across "
          f"{len(set(s['file'] for s in sites))} files (decompiled/Terraria).",
          "Regenerate: `python tools/c28_send_census.py` (read-only over the C#).",
          "",
          "| id | MessageID name | sites | files (site counts) |",
          "| --- | --- | --- | --- |"]
    for k in sorted(per_id, key=lambda x: int(x) if x.isdigit() else 999):
        v = per_id[k]
        fc = {}
        for s in v:
            fc[s["file"]] = fc.get(s["file"], 0) + 1
        fs = ", ".join(f"{f} ({n})" for f, n in sorted(fc.items(), key=lambda kv: -kv[1]))
        md.append(f"| {k} | {num2name.get(k,'?')} | {len(v)} | {fs} |")
    md += ["", "## NetMessage.Send* wrappers (ids they emit) + call-site counts", ""]
    for w, inner in sorted(wrappers.items()):
        md.append(f"- `{w}` → ids {','.join(inner)} — {len(wrap_calls[w])} call sites")
    # netUpdate/netSpam increment census summary
    md += ["", "## netUpdate / netSpam / spawnNeedsSyncing write-site census", ""]
    by_field = {}
    for s in inc_sites:
        key = (s["field"], s["op"], s["rhs"] if s["field"] != "netUpdate" else "")
        by_field.setdefault(s["field"], {}).setdefault(s["file"], []).append(s)
    for field in sorted(by_field):
        fc = by_field[field]
        total = sum(len(v) for v in fc.values())
        md.append(f"### `{field}` — {total} write sites")
        md.append("")
        md.append("| file | sites | methods (site counts) |")
        md.append("| --- | --- | --- |")
        for f, v in sorted(fc.items(), key=lambda kv: -len(kv[1])):
            mc = {}
            for s in v:
                mc[s["method"]] = mc.get(s["method"], 0) + 1
            ms = ", ".join(f"{m} ({n})" for m, n in sorted(mc.items(), key=lambda kv: -kv[1])[:12])
            if len(mc) > 12:
                ms += f", … +{len(mc)-12} more"
            md.append(f"| {f} | {len(v)} | {ms} |")
        md.append("")
    md += ["## The netSpam priority-helper call sites", "",
           "| helper | sites | lines (NPC.cs unless noted) |",
           "| --- | --- | --- |"]
    for h in ("NetUpdateIgnoreSpamLimit", "NetUpdateLowPriority"):
        rows = [x for x in helper_calls if x["helper"] == h]
        ls = ", ".join(f"{x['file']}:{x['line']}" for x in rows)
        md.append(f"| `{h}` | {len(rows)} | {ls} |")
    (ROOT / "tools" / "c28_send_census.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print(f"sites={len(sites)} ids={len(per_id)} wrappers={len(wrappers)} increments={len(inc_sites)}")


if __name__ == "__main__":
    sys.exit(main())
