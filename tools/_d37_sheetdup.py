import subprocess, re
src = subprocess.run(["git","show","a725acb0:crates/tr-app/src/assets.rs"],
                     capture_output=True).stdout.decode("utf8","replace")
m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
body = re.sub(r"//[^\n]*", "", m.group(1))
names = re.findall(r'"(NPC_[^"]+)"', body)
print("base NPC_ rows:", len(names), "distinct:", len(set(names)))
print("dups:", sorted(n for n in set(names) if names.count(n) > 1))
