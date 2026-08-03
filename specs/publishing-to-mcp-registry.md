# Publishing to the MCP Registry

The Hushlor server is registered as `io.github.Hushlor/mcp-server-tauri`. The CLI is npm-only and is not listed as a separate MCP server.

## Metadata

`packages/mcp-server/package.json` and `packages/mcp-server/server.json` must contain the same committed `0.14.0` version, `@hushlor/tauri-mcp-server` identifier, Hushlor repository URL, and `io.github.Hushlor/mcp-server-tauri` name. The release workflow verifies these values rather than generating them during publication.

## Automated publishing

After a successful `Release Hushlor Packages` workflow for a `v<major>.<minor>.<patch>` tag, `.github/workflows/publish-mcp-registry.yml` checks out the release commit, validates the committed metadata, authenticates with `mcp-publisher login github-oidc`, and publishes `server.json`.

## Manual publication fallback

```bash
npm run build --workspace=@hushlor/tauri-mcp-server
cd packages/mcp-server
mcp-publisher login github
mcp-publisher publish
```

This section is an external, irreversible publication fallback and requires explicit approval. In GitHub Actions use `mcp-publisher login github-oidc`; on a local machine use the interactive device login (`mcp-publisher login github`) instead. The normal release workflow handles this publication automatically after the coordinated GitHub release, so do not run these commands as a validation check or from a dry run.
