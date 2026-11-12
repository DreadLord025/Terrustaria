# Working with `main` in this repo — rules for every agent

**Read this before you merge anything.** It exists because landed work has been
silently deleted by other agents' merges **four times** in one day:

| # | what was lost | killer |
|---|---------------|--------|
| 1 | `App::enter_loaded_world` (BUG-21) | a ref-update merge from a stale tree |
| 2 | `App::enter_loaded_world` again (BUG-21 re-landed) | another stale-tree merge |
| 3 | `worldgen/dungeon_features.rs` + `touch_damage.rs` (spikes + contact damage) | `9f89c99` |
| 4 | `worldgen/dungeon_features.rs` (spikes + lights + wall variants) | `cc6298a`, 20 minutes after landing |

Nobody was malicious. Every loss came from the same two habits:

* building the merge **from a tree you made earlier** (`git commit-tree` over your
  branch's tree, `git read-tree`, `git merge -s ours`, restoring a stash, or
  copy-pasting files back), and
* trusting a **successful push** as proof that your work survived.

Git cannot help you here: a `commit-tree` merge records no merge base, so the
deletion appears in history as if the deleted work had never existed — no
conflict, no `git log --diff-filter=D` on your own branch, nothing.

---

## 1. The only merge procedure

```powershell
# 0. in your own worktree, with your work committed and pushed to your branch
git fetch origin
git merge origin/main            # merge FORWARD, never sideways, never a rebuild
#    resolve conflicts by reading the C# source for that system and deciding what
#    the PORT needs — never --ours/--theirs wholesale, never -X ours/theirs
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
cargo test -p tr-app --test main_anchors        # the tripwire, see §3
git push origin <your-branch>

# 1. the merge into main is a FAST-FORWARD of a commit whose first parent is main
git checkout -b merge-<task> origin/main
git merge --no-ff <your-branch> -m "Merge branch '<your-branch>': <summary>"
git push origin HEAD:main

# 2. prove it landed (a push is not proof)
git fetch origin
git merge-base --is-ancestor HEAD origin/main   # exit 0 = your merge is in main
git show origin/main:<one-file-you-changed> | Select-String '<a symbol you added>'
git show origin/main:anchors.txt | Select-String '<the row you added>'
```

Step 1 must start from **`origin/main` as it is right now**, and the merge commit
must have `origin/main`'s tip as its **first parent**. If `git merge --no-ff`
reports "Already up to date", your branch is already contained — verify with
step 2 instead of assuming.

## 2. Never do these

* `git commit-tree` / `git read-tree` / `git merge -s ours` / `git reset --hard
  origin/main` + copying files back — these *replace* a tree instead of merging
  one, and silently delete every other agent's landed work.
* `git merge -X ours` / `-X theirs` on a shared branch, or resolving a conflict by
  picking a side wholesale.
* `git push --force` to `main`, ever. If the push is rejected, `git fetch` and
  redo step 1 on the new tip.
* Editing or deleting rows in `anchors.txt` to make a build pass (§3).
* Working in the **primary checkout** (`Desktop/Terraria-Rust`) — other agents
  keep uncommitted state there. Use your own worktree (AGENTS.md step 1) and the
  two junction links (`assets`, `decompiled`).
* Declaring a task done because the push succeeded. Landed-check (§1 step 2) or
  it is not landed.

## 3. The tripwire: `anchors.txt`

`anchors.txt` (repo root) holds one row per landed feature:

```
path|marker|feature name
```

It is checked in two places, so a lossy merge cannot ship:

* `crates/tr-core/build.rs` — **the build fails** with the lost feature's name
  (escape hatch for experiments: `TR_SKIP_ANCHORS=1`),
* `cargo test -p tr-app --test main_anchors` — the same rows, readable failure.

When you land a feature: **add a row** (one per major module or entry point).
When a refactor legitimately renames a marker: update the row in the same commit.

If the check fails, that is not your bug to silence — some merge dropped landed
work. Re-land it (§4) and tell the user who lost what.

## 4. Re-landing dropped work (without touching anyone's checkout)

```powershell
git log --oneline --all -- <dropped-file>        # find the commit that had it
git worktree add ../TR-RELAND -b RELAND origin/main
cd ../TR-RELAND                                   # + the two mklink junctions
git cherry-pick <original-commit>                 # or: -m 1 <merge-commit>
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
# then §1 steps 1-2, and re-run the anchors check
```

Resolve cherry-pick conflicts the same way as a merge conflict: read the C#
source, decide what the port needs, keep both sides' intent.

## 5. Before you open a terminal, in one line

`worktree, not the primary checkout` → `commit + push your branch` → `merge
origin/main forward` → `tests + anchors` → `merge-<task> from origin/main, --no-ff`
→ `push HEAD:main` → `landed-check`.

## 6. Copy-paste prompt for an AI agent

> You are working in the Terraria-Rust repo (`Desktop/Terraria-Rust`) alongside
> other agents on the same `main`.
>
> * Work in your own `git worktree` with a `home-<task>` branch; never touch the
>   primary checkout, never run repo-wide `rustfmt`.
> * Commit and push your branch early, and merge `origin/main` **forward** into
>   it before landing.
> * Land with `git checkout -b merge-<task> origin/main` +
>   `git merge --no-ff <your-branch>` + `git push origin HEAD:main`. Never build
>   a merge from a saved tree (`commit-tree`, `read-tree`, `-s ours`, stashing
>   and restoring), never `-X ours/theirs`, never force-push `main`.
> * Immediately verify with `git fetch` +
>   `git merge-base --is-ancestor HEAD origin/main` + a symbol grep on
>   `git show origin/main:<file>`, and run
>   `cargo test -p tr-app --test main_anchors`. A push is not proof.
> * Add an `anchors.txt` row for every feature you land; never delete a row to
>   make the build pass — a failure there means another agent's merge dropped
>   landed work, and you must re-land it (§4) and say so.
> * Report what you verified (build, tests, live run) and what you did not.
