# Claude Code Rules — Swallpaper Windows

## Autonomous Mode

Claude Code operates in **full autonomous mode** within this repository.
No need to ask for permission — just implement, fix, commit, push, tag, and release.

## Allowed (automatic)

- Read, create, edit files inside this repo
- Run: npm install, npm run build, npm run tauri:build
- Run: cargo check, cargo build, cargo test, cargo clippy, cargo fmt
- Git: status, diff, log, add, commit, push, tag
- Fix compile/lint errors, iterate until CI passes
- Create GitHub Releases via tag push
- Update README, docs, config files

## Must NOT do

- Modify files outside this repository
- Delete, move, or overwrite files outside this repo
- Run destructive commands: rm -rf, git reset --hard, git clean, force push
- Read private documents, expose secrets, run sudo
- Bulk-delete or bulk-move files without explicit confirmation

## Workflow

1. Make changes → npm run build (verify frontend)
2. git add -A && git commit -m "..." && git push origin main
3. Wait for CI — if fails, read logs, fix, commit, push again
4. When ready for release: git tag vX.Y.Z && git push origin vX.Y.Z
5. Release workflow auto-builds and publishes MSI + EXE
