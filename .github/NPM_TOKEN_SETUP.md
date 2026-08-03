# npm Trusted Publishing Setup

The Hushlor release workflow uses npm Trusted Publishing (OIDC) and provenance. It does not use a long-lived `NPM_TOKEN`.

For each package, open its npm **Access** page and add a GitHub Actions trusted publisher:

- Packages: `@hushlor/tauri-mcp-server`, `@hushlor/tauri-mcp-cli`, and `@hushlor/tauri-plugin-mcp-bridge`
- Provider: GitHub Actions
- Repository owner: `Hushlor`
- Repository: `mcp-server-tauri`
- Workflow: `release.yml`

The package must be manually published once before npm can attach a trusted publisher to a brand-new name. After that bootstrap, releases use `npm publish --provenance` with the workflow's `id-token: write` permission.

The renamed Rust crate also uses Trusted Publishing. On crates.io, add a Trusted Publisher for repository `Hushlor/mcp-server-tauri` and workflow `release.yml`. The release workflow grants `id-token: write`, exchanges its OIDC identity for a short-lived crates.io token, and passes it to `cargo publish`; no long-lived `CARGO_REGISTRY_TOKEN` repository secret is required. The one-time local bootstrap helper may still use a local crates.io API token.
