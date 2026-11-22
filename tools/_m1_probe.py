import re
src = open('crates/tr-app/src/assets.rs', encoding='utf8').read()
m = re.search(r'STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];', src, re.S)
body = re.sub(r'//[^\n]*','',m.group(1))
allrows = re.findall(r'"(NPC_[^"]+)"', body)
plain = [r for r in allrows if re.fullmatch(r'NPC_\d+', r)]
print('all NPC rows', len(allrows), 'plain NPC_id', len(plain))
print('non-plain sample:', sorted(set(r for r in allrows if r not in plain))[:25])
print('concat exprs:', re.findall(r'\b[A-Z0-9_]+_SHEETS\b', body)[:10])
