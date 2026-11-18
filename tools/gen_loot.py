#!/usr/bin/env python
"""Generate `crates/tr-app/src/loot_db_gen.rs` from the decompiled
`Terraria/GameContent/ItemDropRules/ItemDropDatabase.cs` `Populate()` (1.4.5.8)
- the full per-NPC loot registration table (643 register sites).

Sources (the decompiled tree is a local untracked reference in the main
checkout):
  decompiled/Terraria/GameContent/ItemDropRules/ItemDropDatabase.cs - Populate()
  decompiled/Terraria/ID/NPCID.cs                                   - NetIdMap

Usage: python tools/gen_loot.py [--check]

Handled forms (everything else is skipped with a warning):
  this.RegisterToNPC(id, RULE);                       - type/net-id entry (the
    type is expanded to the negative net-id family per `NPCID.NetIdMap`)
  this.RegisterToMultipleNPCs(RULE, ids...);          - RegisterToNPC per id
  this.RegisterToMultipleNPCsNotRemixSeed(RULE, ids)  - each wrapped in
    LeadingConditionRule(NotRemixSeedHardmode).OnSuccess(RULE)
  this.RegisterToGlobal(RULE)[.OnFailedRoll(RULE)];   - the global entries
  RULE := ItemDropRule.<ctor>(args) | new <RuleClass>(args) | rule.OnSuccess(R) |
          rule.OnFailedRoll(R)

Skipped (no port system yet): nothing — the NamedNPC/RedHatSkeletron conditions
and the FromOptionsWithoutRepeats/MechBossSpawners/SlimeBody/StatueMimic rule
classes emit since S1-E-14. `RegisterBoss_Skeletron`/`RegisterBoss_MoonLord`
splice through the `Populate → RegisterBosses → RegisterBoss_X` two-level
nesting (FOLLOW_METHODS); the rest of the boss subtree stays unspliced (its
rows are a separate coverage gap, kept out of the emitted diff).
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DECOMP = ROOT / "decompiled" / "Terraria"
OUT = ROOT / "crates" / "tr-app" / "src" / "loot_db_gen.rs"

# The conditions the generator can express (the `Cond` enum variants); the
# `FromCertainWaveAndAbove(wave)` condition carries an argument.
COND_NAMES = {
    "NotExpert": "NotExpert",
    "IsExpert": "IsExpert",
    "IsMasterMode": "IsMasterMode",
    "Easymode": "Easymode",
    "IsHardmode": "IsHardmode",
    "WindyEnoughForKiteDrops": "WindyEnoughForKiteDrops",
    "RemixSeed": "RemixSeed",
    "NotRemixSeed": "NotRemixSeed",
    "NotRemixSeedHardmode": "NotRemixSeedHardmode",
    "RemixSeedHardmode": "RemixSeedHardmode",
    "RemixSeedEasymode": "RemixSeedEasymode",
    "NotRemixSeedEasymode": "NotRemixSeedEasymode",
    "TenthAnniversaryIsUp": "TenthAnniversaryIsUp",
    "TenthAnniversaryIsNotUp": "TenthAnniversaryIsNotUp",
    "DontStarveIsUp": "DontStarveIsUp",
    "DontStarveIsNotUp": "DontStarveIsNotUp",
    "SkyblockIsUp": "SkyblockIsUp",
    "SkyblockIsUpNoSickle": "SkyblockIsUpNoSickle",
    "MechdusaKill": "MechdusaKill",
    "BeatAnyMechBoss": "BeatAnyMechBoss",
    "DownedAllMechBosses": "DownedAllMechBosses",
    "DownedPlantera": "DownedPlantera",
    "FirstTimeKillingPlantera": "FirstTimeKillingPlantera",
    "MissingTwin": "MissingTwin",
    "LegacyHack_IsABoss": "LegacyHack_IsABoss",
    "LegacyHack_IsBossAndExpert": "LegacyHack_IsBossAndExpert",
    "LegacyHack_IsBossAndNotExpert": "LegacyHack_IsBossAndNotExpert",
    "IsBloodMoonAndNotFromStatue": "IsBloodMoonAndNotFromStatue",
    "IsCrimsonAndNotExpert": "IsCrimsonAndNotExpert",
    "IsCorruptionAndNotExpert": "IsCorruptionAndNotExpert",
    "IsPumpkinMoon": "IsPumpkinMoon",
    "IsChristmas": "IsChristmas",
    "PumpkinMoonDropGatingChance": "PumpkinMoonDropGatingChance",
    "FrostMoonDropGatingChance": "FrostMoonDropGatingChance",
    "PumpkinMoonDropGateForTrophies": "PumpkinMoonDropGateForTrophies",
    "FrostMoonDropGateForTrophies": "FrostMoonDropGateForTrophies",
    "HalloweenWeapons": "HalloweenWeapons",
    "HalloweenGoodieBagDrop": "HalloweenGoodieBagDrop",
    "XmasPresentDrop": "XmasPresentDrop",
    "LivingFlames": "LivingFlames",
    "SoulOfLight": "SoulOfLight",
    "SoulOfNight": "SoulOfNight",
    "NotFromStatue": "NotFromStatue",
    "PirateMap": "PirateMap",
    "JungleKeyCondition": "JungleKeyCondition",
    "CorruptKeyCondition": "CorruptKeyCondition",
    "CrimsonKeyCondition": "CrimsonKeyCondition",
    "HallowKeyCondition": "HallowKeyCondition",
    "FrozenKeyCondition": "FrozenKeyCondition",
    "DesertKeyCondition": "DesertKeyCondition",
    "EyeOfCthulhuDefeatedAndNoAltarsInWorld": "EyeOfCthulhuDefeatedAndNoAltarsInWorld",
    "EmpressOfLightIsGenuinelyEnraged": "EmpressOfLightIsGenuinelyEnraged",
    "YoyoCascade": "YoyoCascade",
    "YoyosAmarok": "YoyosAmarok",
    "YoyosYelets": "YoyosYelets",
    "YoyosHelFire": "YoyosHelFire",
    "NeverTrue": "NeverTrue",
    # `Conditions.RedHatSkeletron` (`Conditions.cs:543` — the state machine
    # behind Skeletron's five RedHat items, `ItemDropDatabase.cs:570-574`).
    "RedHatSkeletron": "RedHatSkeletron",
}

# The `Populate → RegisterBosses → RegisterBoss_X` two-level nesting: every
# boss method the C# `RegisterBosses` calls is spliced through (S1-E-14a —
# the whole boss subtree, not just the first two). The emitted registrations
# stay in the C# statement order; each followed method gets a fresh local
# scope so its `short type = N;` id local does not leak across methods.
FOLLOW_METHODS = {
    "RegisterBoss_EOC", "RegisterBoss_BOC", "RegisterBoss_EOW",
    "RegisterBoss_QueenBee", "RegisterBoss_Skeletron", "RegisterBoss_WOF",
    "RegisterBoss_AncientCultist", "RegisterBoss_MoonLord",
    "RegisterBoss_LunarTowers", "RegisterBoss_Betsy", "RegisterBoss_Golem",
    "RegisterBoss_DukeFishron", "RegisterBoss_SkeletronPrime",
    "RegisterBoss_TheDestroyer", "RegisterBoss_Twins", "RegisterBoss_Plantera",
    "RegisterBoss_KingSlime", "RegisterBoss_FrostMoon",
    "RegisterBoss_PumpkinMoon", "RegisterBoss_HallowBoss",
    "RegisterBoss_QueenSlime", "RegisterBoss_Deerclops",
}


def body_of(src, sig):
    m = re.search(sig + r"\s*\n\t\{", src)
    if not m:
        return None
    j = src.index("{", m.start())
    d = 0
    for k in range(j, len(src)):
        if src[k] == "{":
            d += 1
        elif src[k] == "}":
            d -= 1
            if d == 0:
                return src[j + 1 : k]
    return None


def load_net_families():
    """NPCID.NetIdMap: net id -(i+1) -> type; group into type -> [net ids]."""
    src = (DECOMP / "ID" / "NPCID.cs").read_text(encoding="utf-8")
    m = re.search(r"NPCID\.NetIdMap = new int\[\d+\]\s*\{(.*?)\};", src, re.S)
    vals = [int(x) for x in re.findall(r"-?\d+", m.group(1))]
    fam = {}
    for i, t in enumerate(vals):
        net = -(i + 1)
        fam.setdefault(t, []).append(net)
    return fam


def load_item_ids():
    src = (ROOT / "crates" / "tr-app" / "src" / "itemdata.rs").read_text(encoding="utf-8")
    return {int(x) for x in re.findall(r"\bid: (\d+),", src)}


def split_statements(body):
    """Join the chained statements (`.OnSuccess(...)` on the next line) into
    single `;`-terminated strings."""
    out = []
    cur = []
    depth = 0
    for line in body.splitlines():
        l = line.strip()
        if not l or l.startswith("//"):
            continue
        cur.append(l)
        depth += l.count("{") - l.count("}")
        if depth == 0 and l.endswith(";"):
            out.append(" ".join(cur))
            cur = []
    if cur:
        out.append(" ".join(cur))
    return out


def match_call(expr, name):
    """Match `<expr-prefix>this.<name>(<args>);` -> args string, or None."""
    m = re.fullmatch(r"this\.%s\((.*)\);" % re.escape(name), expr, re.S)
    return m.group(1) if m else None


def split_args(args):
    """Top-level comma split (parens/brackets balanced)."""
    parts, depth, cur = [], 0, ""
    for ch in args:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur.strip())
    return parts


def extract_chains(expr):
    """Split a rule expression into (base_expr, [(chain_kind, rule_expr)...])."""
    chains = []
    while True:
        # A register-level chain sits at paren depth 0 (right after the
        # register call's closing paren); deeper `.OnSuccess(` occurrences
        # belong to the rule expression itself.
        m = None
        depth, i = 0, 0
        while i < len(expr):
            ch = expr[i]
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
            elif ch == "." and depth == 0:
                for nm in (".OnSuccess(", ".OnFailedRoll(", ".OnFailedConditions("):
                    if expr.startswith(nm, i):
                        m = (i, nm[1:-1])
                        break
                if m:
                    break
            i += 1
        if m is None:
            return expr, chains
        start, name = m
        kind = {"OnSuccess": "success", "OnFailedRoll": "failed_roll", "OnFailedConditions": "failed_conditions"}[name]
        # find the balanced argument of this chain call and cut the tail
        i = start + len(name) + 2
        depth, j = 1, i
        while j < len(expr):
            if expr[j] == "(":
                depth += 1
            elif expr[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        chains.append((kind, expr[i:j]))
        expr = expr[:start] + expr[j + 1 :]
    return expr, chains


class Gen:
    def __init__(self):
        self.families = load_net_families()
        self.item_ids = load_item_ids()
        self.warnings = []
        self.emitted = 0
        self.skipped = 0
        self.seen_neg = {}   # net_id -> [rule strs] (the Distinct dedupe)

    def warn(self, msg):
        self.warnings.append(msg)
        self.skipped += 1

    # --- rule expression -> rust expression -------------------------------

    def parse_rule(self, expr, where, locals_=None):
        """A rule expression -> a rust expression string, or None on skip."""
        expr, chains = extract_chains(expr.strip())
        base = self.parse_rule_base(expr, where, locals_)
        if base is None:
            return None
        for kind, sub in chains:
            sub_rust = self.parse_rule(sub, where, locals_)
            if sub_rust is None:
                # the chain fires a skipped rule: the base stays alone
                self.warn(f"{where}: chain {kind} skipped ({sub[:60]})")
                continue
            fn_name = {"success": "chain_on_success", "failed_roll": "chain_on_failed_roll", "failed_conditions": "chain_on_failed_conditions"}[kind]
            base = f"{fn_name}({base}, {sub_rust})"
        return base

    def parse_rule_base(self, expr, where, locals_=None):
        expr = expr.strip()
        m = re.fullmatch(r"ItemDropRule\.(\w+)\((.*)\)", expr, re.S)
        if m:
            return self.sugar(m.group(1), m.group(2), where, locals_)
        m = re.fullmatch(r"new (\w+)\((.*)\)", expr, re.S)
        if m:
            return self.raw_rule(m.group(1), m.group(2), where, locals_)
        self.warn(f"{where}: unhandled rule `{expr[:90]}`")
        return None

    def cond_arg(self, expr, where, locals_=None):
        """`new Conditions.X()` -> `Cond::X` (or None to skip).

        The struct conditions (`NamedNPC("K")`) return a `Box<dyn
        DropCondition>` expression instead of a `Cond::` enum value — the
        callers that can only take a `Cond` guard on the prefix.
        """
        expr = expr.strip()
        if locals_ and expr in locals_:
            return locals_[expr]
        m = re.fullmatch(r'new Conditions\.NamedNPC\("([^"]+)"\)', expr)
        if m:
            # `Conditions.NamedNPC` (`Conditions.cs:962-984`): the localized
            # given-name key comparison; the port keeps the KEY in the struct
            # (Decision D2) and resolves it through the runtime seam.
            return f'NamedNPC::new("{m.group(1)}")'
        m = re.fullmatch(r"new Conditions\.(\w+)\(\)", expr)
        if m:
            name = m.group(1)
            if name in COND_NAMES:
                return f"Cond::{COND_NAMES[name]}"
            self.warn(f"{where}: unknown condition {name}")
            return None
        m = re.fullmatch(r"new Conditions\.(\w+)\((-?\d+)\)", expr)
        if m and m.group(1) == "FromCertainWaveAndAbove":
            return f"Cond::FromCertainWaveAndAbove({m.group(2)})"
        m = re.match(r"new Conditions\.(\w+)\(", expr)
        if m:
            self.warn(f"{where}: unknown condition {m.group(1)}")
            return None
        self.warn(f"{where}: unhandled condition expr `{expr[:80]}`")
        return None

    def sugar(self, name, args, where, locals_=None):
        a = split_args(args)
        inta = []
        for x in a:
            try:
                inta.append(int(x))
            except ValueError:
                inta.append(None)   # a nested expr (conditions/rules)
        def I(i):
            return inta[i] if i < len(inta) and inta[i] is not None else 1
        if name == "Common":
            # Common(item[, denom[, min[, max]]])
            item = inta[0]
            denom = I(1)
            mn = I(2)
            mx = I(3)
            return self.check_item(item, where) or f"common({item}, {denom}, {mn}, {mx})"
        if name == "ByCondition":
            # ByCondition(cond, item[, denom[, min[, max[, numerator]]]])
            cond = self.cond_arg(a[0], where, locals_)
            if cond is None:
                return None
            if not cond.startswith("Cond::"):
                self.warn(f"{where}: ByCondition needs a Cond gate")
                return None
            item = inta[1]
            num = inta[5] if len(inta) > 5 and inta[5] is not None else 1
            return (
                self.check_item(item, where)
                or f"by_condition({cond}, {item}, {I(2)}, {I(3)}, {I(4)}, {num})"
            )
        if name == "OneFromOptions":
            items = [int(x) for x in a[1:]]
            bad = [i for i in items if not self.check_item(i, where, warn=False)]
            if bad:
                self.warn(f"{where}: OneFromOptions unknown items {bad}")
            return f"one_from_options({inta[0]}, &[{', '.join(map(str, items))}])"
        if name == "OneFromOptionsNotScalingWithLuck":
            items = [int(x) for x in a[1:]]
            return f"one_from_options_not_scaling_with_luck({inta[0]}, &[{', '.join(map(str, items))}])"
        if name == "Gel":
            return f"gel({I(0)}, {I(1)}, {I(2)})"
        if name == "NormalvsExpert":
            item = inta[0]
            return self.check_item(item, where) or f"normalvs_expert({item}, {I(1)}, {I(2)})"
        if name == "NormalvsExpertNotScalingWithLuck":
            return f"normalvs_expert_not_scaling_with_luck({inta[0]}, {I(1)}, {I(2)})"
        if name == "NormalvsExpertOneFromOptions":
            opts = [int(x) for x in a[2:]]
            return f"normalvs_expert_one_from_options({inta[0]}, {inta[1]}, &[{', '.join(map(str, opts))}])"
        if name == "NormalvsExpertOneFromOptionsNotScalingWithLuck":
            opts = [int(x) for x in a[2:]]
            return f"normalvs_expert_one_from_options_not_scaling_with_luck({inta[0]}, {inta[1]}, &[{', '.join(map(str, opts))}])"
        if name == "ExpertGetsRerolls":
            return self.check_item(inta[0], where) or f"expert_gets_rerolls({inta[0]}, {I(1)}, {I(2)})"
        if name == "WithRerolls":
            return self.check_item(inta[0], where) or f"with_rerolls({inta[0]}, {I(1)}, {I(2)}, {I(3)}, {I(4)})"
        if name == "BossBag":
            return self.check_item(inta[0], where) or f"boss_bag({inta[0]})"
        if name == "BossBagByCondition":
            cond = self.cond_arg(a[0], where, locals_)
            if cond is None:
                return None
            if not cond.startswith("Cond::"):
                self.warn(f"{where}: BossBagByCondition needs a Cond gate")
                return None
            return self.check_item(inta[1], where) or f"boss_bag_by_condition({cond}, {inta[1]})"
        if name == "MasterModeCommonDrop":
            return self.check_item(inta[0], where) or f"master_mode_common_drop({inta[0]})"
        if name == "MasterModeDropOnAllPlayers":
            return self.check_item(inta[0], where) or f"master_mode_drop_on_all_players({inta[0]}, {I(1)})"
        if name == "Food":
            return self.check_item(inta[0], where) or f"food({inta[0]}, {I(1)}, {I(2)}, {I(3)})"
        if name == "StatusImmunityItem":
            return self.check_item(inta[0], where) or f"status_immunity_item({inta[0]}, {I(1)})"
        if name == "NotScalingWithLuck":
            return self.check_item(inta[0], where) or f"not_scaling_with_luck({inta[0]}, {I(1)}, {I(2)}, {I(3)})"
        if name == "ScalingWithOnlyBadLuck":
            return self.check_item(inta[0], where) or f"scaling_with_only_bad_luck({inta[0]}, {I(1)}, {I(2)}, {I(3)})"
        if name == "DropNothing":
            return "drop_nothing()"
        if name == "OneFromOptionsWithNumerator":
            items = [int(x) for x in a[2:]]
            return (
                f"one_from_options({inta[0]}, &[{', '.join(map(str, items))}])"
                # the numerator rides the generic OneFromOptionsDrop default 1;
                # all the C# call sites pass 1.
                if inta[1] == 1
                else None
            )
        self.warn(f"{where}: unknown sugar ItemDropRule.{name}")
        return None

    def raw_rule(self, cls, args, where, locals_=None):
        if cls == "LeadingConditionRule":
            cond = self.cond_arg(args, where, locals_)
            if cond is None:
                return None
            if not cond.startswith("Cond::"):
                self.warn(f"{where}: LeadingConditionRule needs a Cond gate")
                return None
            return f"leading_condition({cond})"
        if cls == "ItemDropWithConditionRule":
            a = split_args(args)
            cond = self.cond_arg(a[4], where, locals_)
            if cond is None:
                return None
            item = int(a[0])
            num = int(a[5]) if len(a) > 5 else 1
            if cond.startswith("Cond::"):
                return (
                    self.check_item(item, where)
                    or f"by_condition({cond}, {item}, {a[1]}, {a[2]}, {a[3]}, {num})"
                )
            # A struct condition (`NamedNPC`) rides `ItemDropWithCondition`'s
            # `Box<dyn DropCondition>` ctor; `chanceNumerator` stays the
            # default 1 (all the C# sites pass 1/1/1 + the condition).
            return (
                self.check_item(item, where)
                or f"ItemDropWithCondition::new({cond}, {item}, {a[1]}, {a[2]}, {a[3]})"
            )
        if cls == "DropBasedOnExpertMode":
            a = split_args(args)
            normal = self.parse_rule(a[0], where, locals_)
            expert = self.parse_rule(a[1], where, locals_)
            if normal is None or expert is None:
                self.warn(f"{where}: DropBasedOnExpertMode with a skipped side")
                return None
            return f"DropBasedOnExpertMode::new({normal}, {expert})"
        if cls == "DropBasedOnMasterMode":
            a = split_args(args)
            normal = self.parse_rule(a[0], where, locals_)
            master = self.parse_rule(a[1], where, locals_)
            if normal is None or master is None:
                self.warn(f"{where}: DropBasedOnMasterMode with a skipped side")
                return None
            return f"DropBasedOnMasterMode::new({normal}, {master})"
        if cls == "CommonDrop":
            a = split_args(args)
            if len(a) >= 5:
                return (
                    self.check_item(int(a[0]), where)
                    or f"CommonDrop::with_numerator({a[0]}, {a[1]}, {a[2]}, {a[3]}, {a[4]})"
                )
            return (
                self.check_item(int(a[0]), where)
                or f"common({a[0]}, {a[1]}, {a[2]}, {a[3]})"
            )
        if cls == "DropBasedOnMasterAndExpertMode":
            # `new DropBasedOnMasterAndExpertMode(normal, expert, master)` -
            # the master side takes precedence over the expert side.
            a = split_args(args)
            normal = self.parse_rule(a[0], where, locals_)
            expert = self.parse_rule(a[1], where, locals_)
            master = self.parse_rule(a[2], where, locals_)
            if None in (normal, expert, master):
                self.warn(f"{where}: DropBasedOnMasterAndExpertMode with a skipped side")
                return None
            return (
                f"DropBasedOnMasterMode::new("
                f"DropBasedOnExpertMode::new({normal}, {expert}), {master})"
            )
        if cls == "CommonDropNotScalingWithLuck":
            a = split_args(args)
            return f"not_scaling_with_luck({a[0]}, {a[1]}, {a[2]}, {a[3]})"
        if cls == "OneFromOptionsNotScaledWithLuckDropRule":
            a = split_args(args)
            items = ", ".join(re.findall(r"-?\d+", a[2]))
            return f"one_from_options_not_scaling_with_luck({a[0]}, &[{items}])"
        if cls == "OneFromRulesRule":
            parts = split_args(args)
            rust = [self.parse_rule(x, where, locals_) for x in parts]
            if any(r is None for r in rust):
                self.warn(f"{where}: OneFromRulesRule with a skipped sub-rule")
                return None
            return f"OneFromRulesRule::new(vec![{', '.join(rust)}])"
        if cls == "FromOptionsWithoutRepeatsDropRule":
            # `FromOptionsWithoutRepeatsDropRule.cs` — the temp-list
            # `Next(count)`/`RemoveAt` loop (S1-E-14); `dropCount` first, the
            # options variadic.
            a = split_args(args)
            options = [int(x) for x in a[1:]]
            bad = [i for i in options if not self.check_item(i, where, warn=False)]
            if bad:
                self.warn(f"{where}: FromOptionsWithoutRepeats unknown items {bad}")
            return f"FromOptionsWithoutRepeatsDropRule::new({a[0]}, &[{', '.join(map(str, options))}])"
        if cls in ("MechBossSpawnersDropRule", "SlimeBodyItemDropRule", "StatueMimicItemDropRule"):
            # The three parameterless S1-E-14 rule classes (`MechBoss
            # SpawnersDropRule.cs`, `SlimeBodyItemDropRule.cs`,
            # `StatueMimicItemDropRule.cs`) — the engine carries their state
            # reads (`DropAttemptInfo`).
            return f"{cls}::new()"
        self.warn(f"{where}: unknown rule class {cls}")
        return None

    def check_item(self, item, where, warn=True):
        if item is None or item < 0:
            return None
        if item not in self.item_ids:
            if warn:
                self.warn(f"{where}: item {item} not in itemdata (emitted anyway)")
            return None
        return None

    # --- registration statements -----------------------------------------

    def emit_rule_for(self, net_ids, rule_rust, gen):
        for nid in net_ids:
            if nid < 0:
                # `TrimDuplicateRulesForNegativeIDs` dedupes the family
                # registrations (`ItemDropDatabase.cs:30-38`).
                seen = self.seen_neg.setdefault(nid, [])
                if rule_rust in seen:
                    continue
                seen.append(rule_rust)
            gen.append(f"    db.register_to_npc({nid}, {rule_rust});")
            self.emitted += 1

    def multiple_npc_args(self, a, where, locals_):
        """The `RegisterToMultipleNPCs(rule, ids...)` /
        `RegisterToMultipleNPCs(npcNetIds: new int[N] {..}, entry: rule)` /
        `RegisterToMultipleNPCs(rule, localArray)` argument shapes."""
        ids, rule_expr = [], None
        for x in a:
            x = re.sub(r"^\w+: ", "", x)   # named-parameter prefix
            m = re.fullmatch(r"new int\[\d*\]\s*\{([^}]*)\}", x, re.S)
            if m:
                ids += [int(v) for v in re.findall(r"-?\d+", m.group(1))]
            elif x in locals_ and isinstance(locals_[x], list):
                ids += locals_[x]
            elif x in locals_ and isinstance(locals_[x], int):
                # The `short type = N;` local id.
                ids += [locals_[x]]
            elif rule_expr is None:
                rule_expr = x
            else:
                ids += [int(x)]
        rule = self.parse_rule(rule_expr, where, locals_) if rule_expr else None
        if rule is None and rule_expr is not None:
            return None, ids
        return rule, ids

    def apply_chains(self, rule, chains, where, locals_=None):
        """Attach the `.OnSuccess(...)`/`.OnFailedRoll(...)` tail."""
        for kind, sub in chains:
            sub_rust = self.parse_rule(sub, where, locals_)
            if sub_rust is None:
                self.warn(f"{where}: chain {kind} skipped ({sub[:60]})")
                continue
            fn_name = {"success": "chain_on_success", "failed_roll": "chain_on_failed_roll", "failed_conditions": "chain_on_failed_conditions"}[kind]
            rule = f"{fn_name}({rule}, {sub_rust})"
        return rule

    def statement(self, expr, gen, locals_):
        expr = expr.strip()
        where = expr[:70]
        # The chains may hang off the register call itself (`RegisterToNPC(...) 
        # .OnFailedRoll(...)` returns the entry).
        expr, chains = extract_chains(expr.rstrip(";"))
        expr = expr.strip().rstrip(";")
        args = match_call(expr + ";", "RegisterToNPC")
        if args is not None:
            a = split_args(args)
            rule = self.parse_rule(a[1], where, locals_)
            if rule is None:
                return
            rule = self.apply_chains(rule, chains, where, locals_)
            # The `short type = N;` local (the boss methods' npc id).
            nid_arg = a[0]
            if locals_ and nid_arg in locals_ and isinstance(locals_[nid_arg], int):
                nid_arg = locals_[nid_arg]
            nid = int(nid_arg)
            ids = [nid]
            if nid > 0:
                ids += self.families.get(nid, [])
            self.emit_rule_for(ids, rule, gen)
            return
        args = match_call(expr + ";", "RegisterToMultipleNPCs")
        if args is not None:
            a = split_args(args)
            rule, ids = self.multiple_npc_args(a, where, locals_)
            if rule is None:
                return
            rule = self.apply_chains(rule, chains, where, locals_)
            for nid in ids:
                expand = [nid] + (self.families.get(nid, []) if nid > 0 else [])
                self.emit_rule_for(expand, rule, gen)
            return
        args = match_call(expr + ";", "RegisterToMultipleNPCsNotRemixSeed")
        if args is not None:
            a = split_args(args)
            rule, ids = self.multiple_npc_args(a, where, locals_)
            if rule is None:
                return
            for nid in ids:
                expand = [nid] + (self.families.get(nid, []) if nid > 0 else [])
                wrapped = f"chain_on_success(leading_condition(Cond::NotRemixSeedHardmode), {rule})"
                wrapped = self.apply_chains(wrapped, chains, where, locals_)
                self.emit_rule_for(expand, wrapped, gen)
            return
        args = match_call(expr + ";", "RegisterToMultipleNPCsRemixSeed")
        if args is not None:
            a = split_args(args)
            rule, ids = self.multiple_npc_args(a, where, locals_)
            if rule is None:
                return
            for nid in ids:
                expand = [nid] + (self.families.get(nid, []) if nid > 0 else [])
                self.emit_rule_for(
                    expand, f"chain_on_success(leading_condition(Cond::RemixSeed), {rule})", gen
                )
            return
        m = re.fullmatch(r"this\.RemoveFromMultipleNPCs\(this\.(RegisterToMultipleNPCs(?:NotRemixSeed|RemixSeed)?|RegisterToNPC)\((.*)\), (.*)\);", expr + ";", re.S)
        if m:
            # Register-then-remove: the net registration is the first id set
            # minus the second (`RegisterSlimeFamily`'s gel splits).
            inner_name, inner_args, remove_args = m.group(1), m.group(2), m.group(3)
            inner = split_args(inner_args)
            rule, ids = self.multiple_npc_args(inner, where, locals_)
            if rule is None:
                return
            remove_ids = []
            for x in split_args(remove_args):
                x = re.sub(r"^\w+: ", "", x)
                remove_ids += locals_.get(x) or [int(x)]
            net = [i for i in ids if i not in remove_ids]
            if inner_name in ("RegisterToMultipleNPCsNotRemixSeed", "RegisterToMultipleNPCsRemixSeed"):
                cond = "NotRemixSeedHardmode" if inner_name.startswith("RegisterToMultipleNPCsNotRemix") else "RemixSeed"
                for nid in net:
                    expand = [nid] + (self.families.get(nid, []) if nid > 0 else [])
                    self.emit_rule_for(
                        expand, f"chain_on_success(leading_condition(Cond::{cond}), {rule})", gen
                    )
            else:
                for nid in net:
                    expand = [nid] + (self.families.get(nid, []) if nid > 0 else [])
                    self.emit_rule_for(expand, rule, gen)
            return
        args = match_call(expr + ";", "RegisterToGlobal")
        if args is not None:
            rule = self.parse_rule(args, where, locals_)
            if rule is None:
                return
            rule = self.apply_chains(rule, chains, where, locals_)
            gen.append(f"    db.register_global({rule});")
            self.emitted += 1
            return
        if re.fullmatch(r"new Conditions\.\w+\(\)", expr):
            return   # a discarded expression statement
        # unknown statement (SetDefaults-style internals etc.)
        self.warn(f"{where}: unhandled statement")


ARRAY_RE = re.compile(r"int\[\] (\w+) = new int\[\d*\]\s*\{([^}]*)\};")
LOCAL_COND_RE = re.compile(r"[\w.]+ (\w+) = new Conditions\.(\w+)\(\);")


def process_stream(g, statements, gen, locals_, src=None):
    """Parse one method's statement stream with its local scope."""
    for st in statements:
        m = ARRAY_RE.search(st)
        if m and re.fullmatch(ARRAY_RE, st):
            locals_[m.group(1)] = [int(x) for x in re.findall(r"-?\d+", m.group(2))]
            continue
        m = LOCAL_COND_RE.fullmatch(st)
        if m:
            locals_[m.group(1)] = f"Cond::{m.group(2)}"
            continue
        # `short type = 35;` — the per-method npc-id local (the boss methods
        # register through it; `RegisterToNPC(type, ...)` resolves below).
        m = re.fullmatch(r"(?:short|ushort|int) (\w+) = (-?\d+);", st)
        if m:
            locals_[m.group(1)] = int(m.group(2))
            continue
        m = re.fullmatch(r"this\.(\w+)\(\);", st)
        if m and src is not None and m.group(1) in FOLLOW_METHODS:
            # The two-level nesting (`Populate → RegisterBosses →
            # RegisterBoss_X`): the followed methods splice with a fresh
            # local scope, keeping the C# statement order.
            meth = body_of(src, r"private void %s\(\)" % m.group(1))
            if meth is None:
                print(f"generator: method {m.group(1)} not found")
                continue
            process_stream(g, split_statements(meth), gen, {}, src)
            continue
        g.statement(st, gen, locals_)


def main():
    check = "--check" in sys.argv
    src = (DECOMP / "GameContent" / "ItemDropRules" / "ItemDropDatabase.cs").read_text(encoding="utf-8")
    populate = body_of(src, r"public void Populate\(\)")
    # The Populate body calls the register methods in order - splice them,
    # keeping each method's local int[] scope.
    g = Gen()
    gen = [
        "// GENERATED by `tools/gen_loot.py` from the decompiled",
        "// `ItemDropDatabase.cs` `Populate()` (1.4.5.8) - do not edit by hand;",
        "// rerun `python tools/gen_loot.py`. The registrations are in the C#",
        "// order; the type ids expand to the negative net-id families via",
        "// `NPCID.NetIdMap` (the C# `RegisterToNPC` expansion).",
        "",
        "use tr_core::loot::{",
        "    by_condition, boss_bag, boss_bag_by_condition, common, cond, drop_nothing,",
        "    expert_gets_rerolls, food, gel, leading_condition, master_mode_common_drop,",
        "    master_mode_drop_on_all_players, normalvs_expert, normalvs_expert_not_scaling_with_luck,",
        "    normalvs_expert_one_from_options, normalvs_expert_one_from_options_not_scaling_with_luck,",
        "    not_scaling_with_luck, one_from_options, one_from_options_not_scaling_with_luck,",
        "    scaling_with_only_bad_luck, status_immunity_item, with_rerolls, Cond,",
        "    chain_on_failed_roll, chain_on_success, CommonDrop, DropBasedOnExpertMode,",
        "    DropBasedOnMasterMode, FromOptionsWithoutRepeatsDropRule,",
        "    ItemDropDatabase, ItemDropWithCondition, MechBossSpawnersDropRule, NamedNPC,",
        "    OneFromRulesRule, SlimeBodyItemDropRule, StatueMimicItemDropRule,",
        "};",
        "",
        "pub fn populate(db: &mut ItemDropDatabase) {",
    ]

    for st in split_statements(populate):
        m = re.fullmatch(r"this\.(\w+)\(\);", st)
        if m and m.group(1) in ("PrepareNPCNetIDsByTypeDictionary", "TrimDuplicateRulesForNegativeIDs"):
            continue
        if m:
            meth = body_of(src, r"private void %s\(\)" % m.group(1))
            if meth is None:
                print(f"generator: method {m.group(1)} not found")
                continue
            process_stream(g, split_statements(meth), gen, {}, src)
            continue
        process_stream(g, [st], gen, {}, src)
    gen.append("}")
    text = "\n".join(gen) + "\n"

    if check:
        current = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if current != text:
            print("loot_db_gen.rs is out of date - rerun tools/gen_loot.py")
            return 1
        print(f"check ok ({g.emitted} registrations)")
        return 0

    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {OUT} ({g.emitted} registrations)")
    print(f"skipped: {g.skipped}")
    for w in g.warnings[:30]:
        print(f"  ! {w}")
    if len(g.warnings) > 30:
        print(f"  ... and {len(g.warnings) - 30} more")
    return 0


if __name__ == "__main__":
    sys.exit(main())
