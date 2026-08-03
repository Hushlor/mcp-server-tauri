# Releasing the Hushlor downstream

This fork uses one coordinated semantic version across the three npm packages and the Rust crate:

- `@hushlor/tauri-mcp-server` (npm)
- `@hushlor/tauri-mcp-cli` (npm)
- `@hushlor/tauri-plugin-mcp-bridge` (npm bindings)
- `tauri-plugin-hushlor-mcp-bridge` (crates.io; consumers keep the dependency alias `tauri-plugin-mcp-bridge`)

The runtime binary and Rust library/import name remain stable. The plugin keeps the official `mcp-bridge` identifier and permission prefix; only the Hushlor package/crate coordinates differ, while consumers keep the source-compatible dependency alias `tauri-plugin-mcp-bridge`.

## Before tagging

1. Update all four changelogs with the new version and date; preserve historical upstream entries.
2. Set the same version in the three `package.json` files, `packages/tauri-plugin-mcp-bridge/Cargo.toml`, both Cargo lockfiles, `package-lock.json`, CLI metadata, and `packages/mcp-server/server.json`.
3. Ensure the CLI dependency on `@hushlor/tauri-mcp-server` is the exact same version.
4. Run `npm install`, `npm run build`, `npm test`, `npm run test:cli`, `npm run standards`, and the Rust checks described in `AGENTS.md`.
5. Review `git diff --check` and confirm the worktree is clean.

## Local validation helper

```bash
node scripts/release-package.js 0.14.0 --dry-run
```

The helper validates committed metadata, builds all packages, runs Cargo checks, packs all npm packages, and packages the Rust crate. It never rewrites versions, commits, tags, pushes, or publishes in dry-run mode.

## Automated release

### One-time bootstrap (`0.13.0`, historical)

This section records the one-time bootstrap that was completed for `0.13.0`. Do not rerun it for current releases; normal releases use the OIDC workflow described below. The bootstrap published the first downstream version from an authenticated local account before configuring npm Trusted Publishing. The helper validated both credentials before any upload, built/checked the commit, and published in this order: plugin npm bindings, Rust crate, server npm package, then CLI npm package. These registry uploads are external and effectively irreversible:

```bash
npm login
npm whoami
# PowerShell: $env:CARGO_REGISTRY_TOKEN = '<crates.io token>'
# POSIX:     export CARGO_REGISTRY_TOKEN='<crates.io token>'
node scripts/release-package.js 0.13.0
```

If the helper stops after an upload, inspect the registry before retrying. Continue one package at a time with the exact commands below rather than blindly repeating an already successful publication:

```bash
npm publish --workspace=@hushlor/tauri-plugin-mcp-bridge --access public
cargo publish --locked --manifest-path packages/tauri-plugin-mcp-bridge/Cargo.toml
npm publish --workspace=@hushlor/tauri-mcp-server --access public
npm publish --workspace=@hushlor/tauri-mcp-cli --access public
```

Configure Trusted Publishers for all three npm packages only after `0.13.0` is visible on npm. Then create and push the annotated bootstrap tag; the workflow verifies the existing npm/crates.io artifacts, skips all upload jobs, creates the GitHub release, and triggers MCP Registry publication:

```bash
git tag -a v0.13.0 -m "Release v0.13.0"
git push personal main
git push personal v0.13.0
```

For a normal clone, replace `personal` with the explicitly configured Hushlor fork remote; never push this release to an upstream `origin` by accident.

### Normal releases (`0.14.0` and later)

`.github/workflows/release.yml` accepts the exact `v0.13.0` bootstrap tag or `v<major>.<minor>.<patch>` tags at `0.14.0` and later. It verifies the tag against committed metadata, runs tests with locked Rust dependencies, publishes all three npm packages with Trusted Publishing/OIDC and public access, publishes the plugin crate in a separate retryable job with crates.io Trusted Publishing/OIDC, publishes the CLI only after the exact server version succeeds, and creates a GitHub release. It does not mutate source metadata at publish time. Normal releases do not require local `npm login`, `npm whoami`, or a local `CARGO_REGISTRY_TOKEN`.

For the current `0.14.0` release, validate locally and then create an annotated tag for the Hushlor fork remote:

```bash
node scripts/release-package.js 0.14.0 --dry-run
git tag -a v0.14.0 -m "Release v0.14.0"
git push personal main
git push personal v0.14.0
```

## Trusted publishing and bootstrap credentials

Each new scoped npm package must be published once manually before adding its Trusted Publisher (`Hushlor/mcp-server-tauri`, workflow `release.yml`). Thereafter the workflow uses `npm publish --provenance` with `id-token: write`; no `NPM_TOKEN` is required. Configure the crates.io Trusted Publisher for the same repository and workflow; `release.yml` exchanges its OIDC identity for a short-lived token and does not read a long-lived `CARGO_REGISTRY_TOKEN` repository secret. The local one-time bootstrap helper still requires `npm login` and a local crates.io token because it runs outside GitHub Actions.
