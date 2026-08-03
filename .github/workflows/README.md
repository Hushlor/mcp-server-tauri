# GitHub Actions Workflows

This fork publishes only the Hushlor downstream artifacts from the `Hushlor/mcp-server-tauri` repository.

## Workflows

- `test.yml` runs Rust, TypeScript, test-app, CLI, skills, and lint checks on pushes and pull requests.
- `release.yml` accepts the one-time `v0.13.0` bootstrap tag and coordinated `v<major>.<minor>.<patch>` tags from `v0.14.0` onward. The historical bootstrap verifies that the manually published npm packages/crate already exist, skips upload jobs, and creates the GitHub release; later tags verify every committed manifest, run focused builds/tests, publish the three `@hushlor/*` npm packages and `tauri-plugin-hushlor-mcp-bridge` with Trusted Publishing/OIDC, and create a GitHub release. No long-lived npm or crates.io token is stored in GitHub Actions. It never mutates source versions at publish time.
- `publish-mcp-registry.yml` publishes the committed `io.github.Hushlor/mcp-server-tauri` metadata after a successful release.
- `deploy-docs.yml` deploys VitePress to <https://hushlor.github.io/mcp-server-tauri/>.

## Release prerequisites

1. Commit the coordinated version (currently `0.14.0`) in all package manifests, Cargo.toml, lockfiles, changelogs, and `packages/mcp-server/server.json`.
2. Validate the current release locally with `node scripts/release-package.js 0.14.0 --dry-run`; this does not require local registry login or tokens.
3. Configure npm Trusted Publishing for each package, with owner `Hushlor`, repository `mcp-server-tauri`, and workflow `release.yml`.
4. Configure the crates.io Trusted Publisher for `Hushlor/mcp-server-tauri` and workflow `release.yml`; normal workflow releases use OIDC and do not require a long-lived `CARGO_REGISTRY_TOKEN` repository secret.
5. Create and push the annotated `v0.14.0` tag so the workflow runs the normal OIDC release:

   ```bash
   git tag -a v0.14.0 -m "Release v0.14.0"
   git push personal main
   git push personal v0.14.0
   ```

The one-time `v0.13.0` bootstrap is historical and used local npm/crates.io credentials; do not rerun it for `0.14.0` or later. Package-specific tags are intentionally unsupported so the CLI cannot be published against a missing server version.

## Local checks

```bash
npm install
npm run build
npm test
npm run test:cli
npm run standards
```

Do not publish, tag, or push from a local dry run unless explicitly authorised.
