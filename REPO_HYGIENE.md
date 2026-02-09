# Repo Hygiene (Working Copy Safety)

This project relies on local-first workflows and frequent iteration. To keep builds and reports trustworthy, we want `git status` and diffs to reflect reality.

## Policy
1. Do not rely on `skip-worktree` or `assume-unchanged` for tracked files.
2. If you need local-only config, prefer:
   - `.gitignore` (repo-level patterns), or
   - `.git/info/exclude` (local-only ignore patterns), or
   - a documented local config file pattern.

## How To Audit Flags
These commands are safe and read-only.

1. Check `skip-worktree` (sparse/index-only drift risk):
```bash
git ls-files -t | rg '^S '
```

2. Check `assume-unchanged` (hidden local drift risk):
```bash
git ls-files -v | rg '^[a-z] '
```

Both should return nothing in normal operation.

## How To Clear Flags
Only run if you explicitly want to remove hidden drift flags.

1. Clear `skip-worktree`:
```bash
git ls-files -t -z | perl -0ne 'for (split(/\\0/, $_)) { next unless /^S (.+)$/; print \"$1\\0\"; }' | xargs -0 -I{} git update-index --no-skip-worktree -- \"{}\"
```

2. Clear `assume-unchanged`:
```bash
git ls-files -v -z | perl -0ne 'for (split(/\\0/, $_)) { next unless /^[a-z] (.+)$/; print \"$1\\0\"; }' | xargs -0 -I{} git update-index --no-assume-unchanged -- \"{}\"
```

## Verification
After hygiene changes, keep canonical checks green:
```bash
pnpm test:run
pnpm test:bundle
cd src-tauri && cargo test --lib
pnpm tauri build --no-bundle --ci
```

