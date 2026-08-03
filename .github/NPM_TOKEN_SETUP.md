# npm Trusted Publishing Setup

The Hushlor release workflow uses npm Trusted Publishing (OIDC) and provenance. It does not use a long-lived `NPM_TOKEN`.

For each package, open its npm **Access** page and add a GitHub Actions trusted publisher:

- Packages: `@hushlor/tauri-mcp-server`, `@hushlor/tauri-mcp-cli`, and `@hushlor/tauri-plugin-mcp-bridge`
- Provider: GitHub Actions
- Repository owner: `Hushlor`
- Repository: `mcp-server-tauri`
- Workflow: `release.yml`

The package must be manually published once before npm can attach a trusted publisher to a brand-new name. After that bootstrap, releases use `npm publish --provenance` with the workflow's `id-token: write` permission.

The renamed Rust crate uses a separate `CARGO_REGISTRY_TOKEN` repository secret. Keep that token scoped to publishing and rotate it periodically.
