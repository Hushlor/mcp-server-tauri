# Documentation Guide

## Overview

The MCP Server Tauri documentation is built using [VitePress](https://vitepress.dev/), a Vue-powered static site generator. Source files are in the `docs/` directory.

## Structure

- **`docs/`** - Documentation source files
  - `index.md` - Home page
  - `guides/` - Tutorial and guide pages
  - `api/` - API reference documentation
  - `public/` - Static assets (images, etc.)
  - `.vitepress/` - VitePress configuration

## Building Documentation

### Local Development

```bash
# Start dev server with hot reload
npm run docs:dev

# Build for production
npm run docs:build

# Preview production build
npm run docs:preview
```

The dev server runs at http://localhost:5173 by default.

### Production Deployment

Documentation is deployed to GitHub Pages via GitHub Actions:

1. The workflow runs automatically when changes are pushed to `docs/`
2. It builds the documentation and deploys to GitHub Pages
3. The site will be available at: https://hushlor.github.io/mcp-server-tauri/

To manually trigger deployment:
- Go to Actions tab in GitHub
- Select "Deploy Documentation" workflow
- Click "Run workflow"

## Writing Documentation

### Adding New Pages

1. Create a markdown file in the appropriate `docs/` subdirectory
2. Add frontmatter if needed:

```markdown
---
title: Page Title
---

# Your content here
```

3. Add the page to the sidebar in `docs/.vitepress/config.mts` if needed

### VitePress Features

- **Vue Components**: Use Vue components directly in markdown
- **Code Highlighting**: Automatic syntax highlighting for code blocks
- **Frontmatter**: YAML frontmatter for page metadata
- **Custom Containers**: Tips, warnings, and other callouts with `:::` syntax

### Content Organization

- `/` - Home page (`index.md`)
- `/guides/` - Getting started and tutorials
- `/api/` - API reference for all tools

## Rust Documentation

Generate Rust API documentation for the Tauri plugin:

```bash
npm run docs:rust
```

This opens the generated documentation in your browser.
