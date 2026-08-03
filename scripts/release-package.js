#!/usr/bin/env node

/* eslint-env node */
/* eslint-disable no-process-exit, no-process-env, no-undef */

/**
 * Validate and publish the coordinated Hushlor downstream release.
 *
 * Usage: node scripts/release-package.js <version> [--dry-run]
 *
 * The version must already be committed in every package manifest, Cargo.toml,
 * lockfile and server.json. This script deliberately never rewrites versions,
 * creates tags, or pushes commits.
 */

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const rootDir = join(import.meta.dirname, '..');

const packages = {
   server: { name: '@hushlor/tauri-mcp-server', path: 'packages/mcp-server', cargo: false },
   plugin: { name: '@hushlor/tauri-plugin-mcp-bridge', path: 'packages/tauri-plugin-mcp-bridge', cargo: true },
   cli: { name: '@hushlor/tauri-mcp-cli', path: 'packages/cli', cargo: false },
};

function run(command, args, cwd = rootDir) {
   console.log(`\n> ${command} ${args.join(' ')}`);

   const useShell = process.platform === 'win32' && command === 'npm';

   execFileSync(command, args, { cwd, stdio: 'inherit', shell: useShell });
}

function readJson(relativePath) {
   return JSON.parse(readFileSync(join(rootDir, relativePath), 'utf8'));
}

function readCargoMetadata() {
   const cargo = readFileSync(join(rootDir, packages.plugin.path, 'Cargo.toml'), 'utf8');

   const name = /^name = "([^"]+)"$/m.exec(cargo)?.[1] ?? null,
         version = /^version = "([^"]+)"$/m.exec(cargo)?.[1] ?? null,
         links = /^links = "([^"]+)"$/m.exec(cargo)?.[1] ?? null;

   return { name, version, links };
}

function verifyAdditionalMetadata(version) {
   const rootGemini = readJson('gemini-extension.json');

   const cliGemini = readJson(join(packages.cli.path, 'gemini-extension.json'));

   for (const key of [ 'name', 'description', 'contextFileName' ]) {
      if (rootGemini[key] !== cliGemini[key]) {
         throw new Error(`CLI gemini-extension.json diverges from root metadata for ${key}`);
      }
   }

   if (rootGemini.version !== version || cliGemini.version !== version) {
      throw new Error('CLI gemini-extension.json must match the release version');
   }

   const cliPlugin = readJson(join(packages.cli.path, '.claude-plugin/plugin.json'));

   const marketplace = readJson('.claude-plugin/marketplace.json');

   if (cliPlugin.version !== version || marketplace.metadata?.version !== version) {
      throw new Error('CLI plugin and marketplace metadata must match the release version');
   }

   const guestPackage = readJson(join(packages.plugin.path, 'guest-js/package.json'));

   if (guestPackage.name !== '@hushlor/tauri-plugin-mcp-bridge' || guestPackage.version !== version) {
      throw new Error('Guest JS package metadata must match the release version');
   }

   const rootGeminiGuide = readFileSync(join(rootDir, 'GEMINI.md'), 'utf8');

   const cliGeminiGuide = readFileSync(join(rootDir, packages.cli.path, 'GEMINI.md'), 'utf8');

   if (rootGeminiGuide !== cliGeminiGuide) {
      throw new Error('packages/cli/GEMINI.md must be byte-for-byte identical to the root GEMINI.md');
   }
}

function verifyRelease(version) {
   if (!/^\d+\.\d+\.\d+$/.test(version)) {
      throw new Error(`Expected a semantic version, received ${version}`);
   }

   for (const pkg of Object.values(packages)) {
      const manifest = readJson(join(pkg.path, 'package.json'));

      if (manifest.name !== pkg.name || manifest.version !== version) {
         throw new Error(`${pkg.path}/package.json must contain ${pkg.name}@${version}`);
      }
   }

   const cli = readJson(join(packages.cli.path, 'package.json'));

   if (cli.dependencies?.['@hushlor/tauri-mcp-server'] !== version) {
      throw new Error('CLI server dependency must match the release version exactly');
   }

   const cargo = readCargoMetadata();

   if (cargo.name !== 'tauri-plugin-hushlor-mcp-bridge' || cargo.version !== version || cargo.links !== 'tauri-plugin-hushlor-mcp-bridge') {
      throw new Error(`Cargo.toml must contain tauri-plugin-hushlor-mcp-bridge@${version} with matching links metadata`);
   }

   const server = readJson('packages/mcp-server/server.json');

   const hasExpectedMetadata = server.name === 'io.github.hushlor/mcp-server-tauri' &&
      server.version === version &&
      server.packages?.[0]?.identifier === '@hushlor/tauri-mcp-server' &&
      server.packages?.[0]?.version === version;

   if (!hasExpectedMetadata) {
      throw new Error('packages/mcp-server/server.json is not synchronized with the release');
   }

   verifyAdditionalMetadata(version);
}

function checkCleanTree() {
   const status = execFileSync('git', [ 'status', '--porcelain' ], { cwd: rootDir, encoding: 'utf8' });

   if (status.trim()) {
      throw new Error('Release validation requires a clean, committed worktree');
   }
}

function checkNpmAuthentication() {
   try {
      run('npm', [ 'whoami' ]);
   } catch{
      throw new Error('npm authentication is required for the manual bootstrap; run `npm login` first');
   }
}

function ensureCargoToken() {
   if (!process.env.CARGO_REGISTRY_TOKEN?.trim()) {
      throw new Error('CARGO_REGISTRY_TOKEN is required to publish the Rust crate');
   }
}

function main() {
   const [ version, ...flags ] = process.argv.slice(2);

   const dryRun = flags.includes('--dry-run');

   if (!version) {
      throw new Error('Usage: node scripts/release-package.js <version> [--dry-run]');
   }

   verifyRelease(version);
   if (!dryRun) {
      checkCleanTree();
   }

   run('npm', [ 'run', 'build', '--workspace=@hushlor/tauri-mcp-server' ]);
   run('npm', [ 'run', 'build', '--workspace=@hushlor/tauri-plugin-mcp-bridge' ]);
   run('npm', [ 'run', 'build', '--workspace=@hushlor/tauri-mcp-cli' ]);
   run('cargo', [ 'check', '--locked', '--manifest-path', 'packages/tauri-plugin-mcp-bridge/Cargo.toml', '--all-features' ]);

   if (dryRun) {
      run('npm', [ 'pack', '--workspace=@hushlor/tauri-mcp-server', '--dry-run' ]);
      run('npm', [ 'pack', '--workspace=@hushlor/tauri-plugin-mcp-bridge', '--dry-run' ]);
      run('npm', [ 'pack', '--workspace=@hushlor/tauri-mcp-cli', '--dry-run' ]);
      run('cargo', [ 'package', '--locked', '--manifest-path', 'packages/tauri-plugin-mcp-bridge/Cargo.toml', '--allow-dirty' ]);
      return;
   }

   checkNpmAuthentication();
   ensureCargoToken();
   checkCleanTree();

   // The first downstream release is published from a locally authenticated
   // account. Trusted Publishing with npm provenance starts on the next tag.
   run('npm', [ 'publish', '--workspace=@hushlor/tauri-plugin-mcp-bridge', '--access', 'public' ]);
   run('cargo', [ 'publish', '--locked', '--manifest-path', 'packages/tauri-plugin-mcp-bridge/Cargo.toml' ]);
   run('npm', [ 'publish', '--workspace=@hushlor/tauri-mcp-server', '--access', 'public' ]);
   run('npm', [ 'publish', '--workspace=@hushlor/tauri-mcp-cli', '--access', 'public' ]);
   console.log(`Published Hushlor packages for v${version}. Create and push the signed v${version} tag separately.`);
}

try {
   main();
} catch(error) {
   console.error(`\n[ERROR] ${error instanceof Error ? error.message : String(error)}`);
   process.exit(1);
}
