import json, glob, re, collections

GLOBALS = set("""Nurse Merchant ArmsDealer Dryad Demolitionist Clothier Guide Wizard GoblinTinkerer
Mechanic Truffle Steampunker DyeTrader PartyGirl Cyborg Painter WitchDoctor Pirate Stylist
TravelingMerchant Angler Bartender WorldName Day BloodMoon Eclipse MoonLordDefeated GolemDefeated
DukeFishronDefeated FrostLegionDefeated MartiansDefeated PumpkingDefeated IceQueenDefeated HardMode
Homeless InventoryKey PlayerName GolfGuy TaxCollector Rain Graveyard AnglerCompletedQuestsCount
TotalDeathsCount WorldEvilStone InputTriggerUI_BuildFromInventory InputTriggerUI_TrashEnabled
InputTriggerUI_Trash InputTriggerUI_FavoriteItem InputTrigger_QuickEquip InputTrigger_Grapple
InputTrigger_LockOn InputTrigger_RadialQuickbar InputTrigger_RadialHotbar InputTrigger_SmartCursor
InputTrigger_UseOrAttack InputTrigger_Jump InputTrigger_InteractWithTile InputTrigger_InteractWithTileUI
InputTrigger_ToggleOrOpen InputTrigger_SmartSelect InputTrigger_Mount InputTrigger_Healing
InputTrigger_ManaHealing InputTrigger_QuickBuff ToggleArmorSetBonusKey""".split())

pat = re.compile(r'\{([?!]?)([A-Za-z][\w\.]*)\}')
trail = re.compile(r',\s*([}\]])')
all_global = 0
partial = 0
none_global = 0
cats = collections.Counter()
cond_rows = []
uncovered = collections.Counter()
for f in sorted(glob.glob('decompiled/Terraria.Localization.Content.en-US*.json')):
    j = json.loads(trail.sub(r'\1', open(f, encoding='utf-8-sig').read()))
    for cat, vals in j.items():
        for k, v in vals.items():
            if not isinstance(v, str):
                continue
            ms = pat.findall(v)
            if not ms:
                continue
            names = [n for _, n in ms]
            g = [n for n in names if n in GLOBALS]
            if len(g) == len(names):
                all_global += 1
                cats[cat] += 1
            elif g:
                partial += 1
            else:
                none_global += 1
                for n in names:
                    uncovered[n] += 1
            if any(m for m, _ in ms):
                cond_rows.append((cat + '.' + k, v[:70]))
print('all tokens global :', all_global)
print('partially global  :', partial)
print('no global tokens  :', none_global)
print('conditional rows  :', len(cond_rows))
print()
print('all-global categories:', dict(cats))
print()
print('non-global token names:', dict(uncovered))
print()
for r in cond_rows:
    print(r[0], '|', r[1])
