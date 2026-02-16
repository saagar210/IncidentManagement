# IncidentMgmt .codex command map

| Action | Command | Source |
| --- | --- | --- |
| setup deps | `pnpm install --frozen-lockfile` | `README.md`, `.github/workflows/ci.yml` |
| lint fallback | `pnpm run build` | `package.json` (no dedicated lint script) |
| test (frontend) | `pnpm run test:run` | `.github/workflows/ci.yml`, `package.json` |
| test (backend) | `cd src-tauri && cargo test --lib` | `.github/workflows/ci.yml`, `README.md` |
| build | `pnpm tauri build` | `README.md` |
| lean dev | `pnpm run dev:lean` | `README.md`, `package.json` |
