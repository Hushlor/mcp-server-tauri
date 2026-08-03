# AGENTS.md

## Project Overview

MCP server for Tauri v2 application development. Provides tools for CLI execution, configuration management, mobile device/emulator management, native UI automation, and debugging.

**Monorepo packages:**

- `packages/mcp-server/` - Main MCP server implementation
- `packages/tauri-plugin-mcp-bridge/` - Tauri plugin for automation bridge (Rust)
- `packages/test-app/` - Test application for E2E testing
- `packages/cli` - A CLI wrapper for the MCP server funcionality

## Setup Commands

```bash
npm install              # Install all dependencies
npm run build            # Build all packages
npm test                 # Run tests (requires build first)
npm run standards        # Run commitlint + eslint
```

## Code Style

- **TypeScript**: Strict mode, ESM with `.js` extensions in imports, ES2022 target
- **Naming**: camelCase for functions/variables, PascalCase for classes, kebab-case for files
- **Acronyms**: All lowercase or all caps, never mixed (e.g., `url` or `URL`, never `Url`)
- **Avoid**: `any` type, magic numbers, deeply nested blocks
- **Prefer**: Early returns, higher-order functions, immutability (`readonly`, `as const`)
- **Functions**: Arrow for simple (<3 instructions), named otherwise; use RO-RO pattern
- **JSDoc**: Document public classes and methods

## Adding New MCP Tools

All tools are defined in `packages/mcp-server/src/tools-registry.ts`:

1. Add Zod schema + handler in the appropriate module (`manager/`, `driver/`, `monitor/`)
2. Import and add entry to `TOOLS` array in `tools-registry.ts`
3. Add E2E test in `packages/mcp-server/tests/e2e/`

Tool categories: `PROJECT_MANAGEMENT`, `MOBILE_DEVELOPMENT`, `UI_AUTOMATION`, `IPC_PLUGIN`

## Testing Instructions

- Always build before testing: `npm run build`
- E2E tests launch `test-app` and connect via WebSocket
- Tests located in `packages/mcp-server/tests/e2e/`
- Prefer E2E tests over unit tests
- CI timeout is 8 minutes; local is 1 minute

## Manual MCP Testing (local dev MCP server)

Create `.mcp.json` at the workspace root (it's covered by a global `.gitignore` so it stays local):

```json
{
   "mcpServers": {
      "tauri-dev": {
         "type": "stdio",
         "command": "node",
         "args": ["packages/mcp-server/dist/index.js"]
      }
   }
}
```

This registers a project-scoped MCP server named `tauri-dev` that runs the local-built `packages/mcp-server/dist/index.js`, so edits to the MCP server show up the next time the agent reconnects. Do **not** invoke the published `mcp-server-tauri` from the global config for dev work — those tools run a different build.

Workflow:

1. Run `npm run dev` in a terminal. This starts the `tsc --watch` rebuild of `packages/mcp-server/dist/` and `tauri dev` for `test-app` on port 9300.
2. In Claude Code (this workspace), the `tauri-dev` MCP server is auto-launched on session start. Tool names are namespaced as `mcp__tauri-dev__<tool>` (e.g. `mcp__tauri-dev__webview_screenshot`).
3. To test plugin (Rust) changes: edit Rust → `tauri dev` rebuilds and relaunches `test-app` automatically.
4. To test server (TS) changes: edit TS → `tsc --watch` rewrites `dist/` → restart the MCP server connection in Claude Code (`/mcp` reconnect) so the spawned subprocess picks up the new build.
5. Plugin stderr/stdout (including `mcp_log_error` and `mcp_log_info`) appears in the `npm run dev` terminal under `[app]`. MCP server stderr appears in Claude Code's MCP debug pane.

This replaces ad-hoc Node scripts that talk raw WebSocket to `localhost:9300`. Reach for raw WS only when probing plugin behavior that isn't exposed as an MCP tool.

## Session Management

- Call `driver_session` with `action: 'start'` before using driver tools
- Always call with `action: 'stop'` to clean up
- WebSocket port range: 9223-9322

## Git Commits

Follow: <https://raw.githubusercontent.com/silvermine/standardization/refs/heads/master/commitlint.js>

## Releasing

This monorepo uses a **single version** across all packages. All packages share the same version number.

### Files to Update

**Version files (all must have the same version):**

- `packages/mcp-server/package.json` - `version` field
- `packages/cli/package.json` - `version` field and `@hushlor/tauri-mcp-server` dependency version
- `packages/cli/.claude-plugin/plugin.json` - `version` field
- `.claude-plugin/marketplace.json` - `metadata.version` field
- `gemini-extension.json` - `version` field
- `packages/cli/gemini-extension.json` - copied `version` and metadata fields
- `packages/tauri-plugin-mcp-bridge/package.json` - `version` field
- `packages/tauri-plugin-mcp-bridge/guest-js/package.json` - package name and `version` fields
- `packages/tauri-plugin-mcp-bridge/Cargo.toml` - `version` field

**Changelog files (all four must be updated):**

- `CHANGELOG.md` - Root changelog for overall project history
- `packages/cli/CHANGELOG.md` - CLI-specific changes
- `packages/mcp-server/CHANGELOG.md` - Server-specific changes
- `packages/tauri-plugin-mcp-bridge/CHANGELOG.md` - Plugin-specific changes

**Lock files (updated automatically but must be committed):**

- `package-lock.json` - Updated by `npm install`
- `packages/tauri-plugin-mcp-bridge/Cargo.lock` - Updated with a minimal root package/version change; do not refresh unrelated transitive dependencies
- `packages/test-app/src-tauri/Cargo.lock` - Updated with the matching minimal root package/dependency change; do not refresh unrelated transitive dependencies

### Release Checklist

1. **Review git log** to identify changes since the last release tag
2. **Determine version bump** (patch for fixes, minor for features, major for breaking)
3. **Update all four changelogs** with the new version entry:
   - Add entry under `## [Unreleased]` with the new version and date
   - Include changes relevant to each package (use `_No changes to this package._` if none)
   - **Do not skip any version numbers** - if v0.2.1 exists, the next must be v0.2.2, not v0.2.3
4. **Update version in package.json files** using npm (without git tag):

   ```bash
   npm version <version> --no-git-tag-version -w @hushlor/tauri-mcp-server -w @hushlor/tauri-mcp-cli -w @hushlor/tauri-plugin-mcp-bridge
   ```

5. **Update version-coupled metadata** manually to match:
   - `packages/cli/package.json` - update the `@hushlor/tauri-mcp-server` dependency version
   - `packages/cli/.claude-plugin/plugin.json`
   - `.claude-plugin/marketplace.json`
   - `gemini-extension.json`
   - `packages/cli/gemini-extension.json`
   - `packages/tauri-plugin-mcp-bridge/Cargo.toml`
   - `packages/tauri-plugin-mcp-bridge/guest-js/package.json`
6. **Update lock files**:

   ```bash
   npm install
   # Do not run cargo update: it can silently upgrade unrelated transitive crates.
   # Edit only the root package name/version and the exact path dependency in both
   # Cargo.lock files, then verify with locked checks:
   cargo check --locked --manifest-path packages/tauri-plugin-mcp-bridge/Cargo.toml --all-features
   cargo check --locked --manifest-path packages/test-app/src-tauri/Cargo.toml
   ```

   Review `git diff -- packages/tauri-plugin-mcp-bridge/Cargo.lock packages/test-app/src-tauri/Cargo.lock` and reject any dependency or version changes unrelated to the coordinated release.

7. **Verify versions** in manifests and lock files match the new version:

   ```bash
   grep -n '"version"' packages/cli/.claude-plugin/plugin.json gemini-extension.json
   grep -n '"version"' packages/cli/gemini-extension.json
   grep -n '"version"' .claude-plugin/marketplace.json packages/tauri-plugin-mcp-bridge/guest-js/package.json
   grep -n '"@hushlor/tauri-mcp-server"' packages/cli/package.json
   grep -A2 '"@hushlor/tauri-mcp-server"' package-lock.json | head -3
   grep -A2 '"@hushlor/tauri-mcp-cli"' package-lock.json | head -3
   grep -A2 '"@hushlor/tauri-plugin-mcp-bridge"' package-lock.json | head -3
   node --input-type=module -e "import fs from 'node:fs'; const root=JSON.parse(fs.readFileSync('gemini-extension.json')); const cli=JSON.parse(fs.readFileSync('packages/cli/gemini-extension.json')); if (JSON.stringify({...root, version:undefined}) !== JSON.stringify({...cli, version:undefined}) || cli.version !== root.version) throw new Error('CLI Gemini metadata diverges from root');"
   node --input-type=module -e "import fs from 'node:fs'; if (fs.readFileSync('GEMINI.md','utf8') !== fs.readFileSync('packages/cli/GEMINI.md','utf8')) throw new Error('CLI GEMINI.md diverges from root');"
   node --input-type=module -e "import fs from 'node:fs'; const v=JSON.parse(fs.readFileSync('packages/mcp-server/package.json')).version; const cli=JSON.parse(fs.readFileSync('packages/cli/.claude-plugin/plugin.json')); const market=JSON.parse(fs.readFileSync('.claude-plugin/marketplace.json')); const guest=JSON.parse(fs.readFileSync('packages/tauri-plugin-mcp-bridge/guest-js/package.json')); if (cli.version !== v || market.metadata?.version !== v || guest.version !== v || guest.name !== '@hushlor/tauri-plugin-mcp-bridge') throw new Error('Release metadata is not synchronized');"
   ```

8. **Stage all changed files**:
   - All four changelogs
   - `packages/mcp-server/package.json`
   - `packages/cli/package.json`
   - `packages/cli/.claude-plugin/plugin.json`
   - `packages/cli/gemini-extension.json`
   - `packages/cli/GEMINI.md`
   - `gemini-extension.json`
   - `.claude-plugin/marketplace.json`
   - `packages/tauri-plugin-mcp-bridge/package.json`
   - `packages/tauri-plugin-mcp-bridge/Cargo.toml`
   - `packages/tauri-plugin-mcp-bridge/guest-js/package.json`
   - package-lock.json
   - Both Cargo.lock files
9. **Commit**: `git commit -m "chore: version bump: v<version>"`
10. **Create signed tag**: `git tag -s v<version> -m "Release v<version>"`
11. **Verify the Hushlor fork remote and push only it**:
   ```bash
   git remote get-url personal
   git push personal main
   git push personal "v<version>"
   ```
   Never use an implicit `origin` push from this checkout: `personal` must resolve to the Hushlor fork.

### Common Mistakes to Avoid

- **Skipping changelog entries**: Every version must have an entry in all four changelogs
- **Forgetting lock files**: Both `package-lock.json` and both `Cargo.lock` files must be updated
- **Forgetting CLI metadata**: `packages/cli/.claude-plugin/plugin.json`, `gemini-extension.json`, and the CLI's pinned `@hushlor/tauri-mcp-server` dependency must match the release version
- **Version mismatch**: All version fields must match exactly
- **Missing intermediate versions**: If changelogs are missing entries for previous versions, add them before creating the new release

## Rust Code

Run `cargo fmt` and `cargo clippy` after changes in `packages/tauri-plugin-mcp-bridge/`.

## NPM Dependencies

Always use `--save-exact` flag when installing.

## Key Files

- `packages/mcp-server/src/tools-registry.ts` - Single source of truth for all MCP tools
- `packages/mcp-server/src/index.ts` - MCP server entry point
- `packages/mcp-server/src/driver/session-manager.ts` - WebSocket session management
- `packages/tauri-plugin-mcp-bridge/src/lib.rs` - Plugin entry point and WebSocket server setup
- `specs/` - Architecture docs, release process, and design decisions
