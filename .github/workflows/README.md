# GitHub Actions Workflows

This fork publishes only the Hushlor downstream artifacts from the `Hushlor/mcp-server-tauri` repository.

## Workflows

- `test.yml` runs Rust, TypeScript, test-app, CLI, skills, and lint checks on pushes and pull requests.
- `release.yml` accepts the one-time `v0.13.0` bootstrap tag and coordinated `v<major>.<minor>.<patch>` tags from `v0.13.1` onward. Bootstrap verifies that the manually published npm packages/crate already exist, skips upload jobs, and creates the GitHub release; later tags verify every committed manifest, run focused builds/tests, publish the three `@hushlor/*` npm packages using npm Trusted Publishing (Node 24/npm 11.5.1), publish `tauri-plugin-hushlor-mcp-bridge` with `CARGO_REGISTRY_TOKEN` in a separate retryable job, and create a GitHub release. It never mutates source versions at publish time.
- `publish-mcp-registry.yml` publishes the committed `io.github.hushlor/mcp-server-tauri` metadata after a successful release.
- `deploy-docs.yml` deploys VitePress to <https://hushlor.github.io/mcp-server-tauri/>.

## Release prerequisites

1. Commit version `0.13.0` in all package manifests, Cargo.toml, lockfiles, changelogs, and `packages/mcp-server/server.json`.
2. Authenticate locally (`npm login`, `npm whoami`, and `CARGO_REGISTRY_TOKEN`) and run `node scripts/release-package.js 0.13.0`; inspect each registry before retrying a partial upload.
3. Configure npm Trusted Publishing for each package, with owner `Hushlor`, repository `mcp-server-tauri`, and workflow `release.yml`.
4. Add the crates.io API token as the `CARGO_REGISTRY_TOKEN` repository secret.
5. Create and push the signed `v0.13.0` tag so the workflow creates the bootstrap GitHub release without republishing packages; subsequent coordinated releases use signed `v0.13.1` (or later) tags. Package-specific tags are intentionally unsupported so the CLI cannot be published against a missing server version.

## Local checks

```bash
npm install
npm run build
npm test
npm run test:cli
npm run standards
```

Do not publish, tag, or push from a local dry run unless explicitly authorised.
