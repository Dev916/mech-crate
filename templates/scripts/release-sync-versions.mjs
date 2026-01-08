#!/usr/bin/env node

/**
 * Release Sync Versions
 * 
 * Syncs version numbers across manifest files for a specific app.
 * 
 * Usage:
 *   node release-sync-versions.mjs --app myapp
 *   node release-sync-versions.mjs --app myapp --version 1.2.3
 *   node release-sync-versions.mjs --app myapp --dry-run
 */

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = join(__dirname, '..');

// Parse CLI arguments
const args = process.argv.slice(2);
let appName;
let cliVersion;
let dryRun = false;

for (let i = 0; i < args.length; i += 1) {
  const arg = args[i];
  if (arg === '--app' || arg === '-a') {
    appName = args[i + 1];
    if (!appName) {
      console.error('Missing value after --app flag.');
      process.exit(1);
    }
    i += 1;
  } else if (arg === '--version' || arg === '-v') {
    cliVersion = args[i + 1];
    if (!cliVersion) {
      console.error('Missing value after --version flag.');
      process.exit(1);
    }
    i += 1;
  } else if (arg === '--dry-run') {
    dryRun = true;
  } else if (arg === '--help' || arg === '-h') {
    printHelp();
    process.exit(0);
  } else {
    console.error(`Unknown argument: ${arg}`);
    printHelp();
    process.exit(1);
  }
}

if (!appName) {
  console.error('Error: --app is required.');
  printHelp();
  process.exit(1);
}

// Validate app exists
const appDir = join(repoRoot, 'apps', appName);
if (!existsSync(appDir)) {
  console.error(`Error: App directory not found: apps/${appName}`);
  process.exit(1);
}

// File paths
const manifestPath = join(repoRoot, '.release-please-manifest.json');
const packageJsonPath = join(appDir, 'package.json');
const packageLockPath = join(appDir, 'package-lock.json');

// Determine target version
const envVersion = (process.env.VERSION ?? '').trim();
const targetVersion =
  (cliVersion ?? envVersion ?? '').trim() || detectLatestReleaseVersion(appName);

if (!targetVersion) {
  console.error(
    `Unable to determine target version for ${appName}. Provide --version, set VERSION=, or create a ${appName}-v* tag.`
  );
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(targetVersion)) {
  console.error(
    `Version "${targetVersion}" does not look like a valid semver string (e.g., 1.2.3 or 1.2.3-beta.1).`
  );
  process.exit(1);
}

const updatedFiles = [];
const appManifestKey = `apps/${appName}`;

// Update manifest file
if (existsSync(manifestPath)) {
  updateJson(manifestPath, data => {
    if (typeof data !== 'object' || data === null) {
      throw new Error('Manifest file must be a JSON object.');
    }
    if (!Object.prototype.hasOwnProperty.call(data, appManifestKey)) {
      console.warn(
        `Manifest is missing "${appManifestKey}" entry. It will be added automatically.`
      );
    }
    data[appManifestKey] = targetVersion;
  });
}

// Update package.json
if (existsSync(packageJsonPath)) {
  updateJson(packageJsonPath, data => {
    data.version = targetVersion;
  });
}

// Update package-lock.json
if (existsSync(packageLockPath)) {
  updateJson(packageLockPath, data => {
    data.version = targetVersion;
    if (data.packages && data.packages['']) {
      data.packages[''].version = targetVersion;
    }
  });
}

// Output results
const prefix = dryRun ? '[release-sync dry-run]' : '[release-sync]';
if (updatedFiles.length === 0) {
  console.log(`${prefix} All files already set to version ${targetVersion} for ${appName}.`);
} else {
  console.log(
    `${prefix} Updated ${updatedFiles.length} file(s) to version ${targetVersion} for ${appName}:`
  );
  updatedFiles.forEach(file => console.log(`  - ${relativeToRepo(file)}`));
}

// Helper Functions

function updateJson(path, mutate) {
  const originalText = readFileSync(path, 'utf8');
  const data = JSON.parse(originalText);
  const before = JSON.stringify(data, null, 2) + '\n';
  mutate(data);
  const after = JSON.stringify(data, null, 2) + '\n';
  if (before === after) {
    return;
  }
  if (!dryRun) {
    writeFileSync(path, after);
  }
  updatedFiles.push(path);
}

function detectLatestReleaseVersion(app) {
  // Try multiple tag patterns: app-v*, app-vX.X.X, vX.X.X-app
  const tagPatterns = [
    `${app}-v*`,
    `v*-${app}`,
    `${app.replace(/[.-]/g, '')}-v*`
  ];
  
  for (const pattern of tagPatterns) {
    try {
      const output = execSync(`git tag --list '${pattern}' --sort=version:refname`, {
        cwd: repoRoot,
        encoding: 'utf8'
      }).trim();
      
      if (!output) continue;
      
      const tags = output
        .split('\n')
        .map(line => line.trim())
        .filter(Boolean);
        
      if (tags.length === 0) continue;
      
      const latestTag = tags[tags.length - 1];
      const versionMatch = latestTag.match(/v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)/);
      if (versionMatch) {
        return versionMatch[1];
      }
    } catch (error) {
      // Continue to next pattern
    }
  }
  
  // Fallback: try to read from package.json
  try {
    const pkgPath = join(repoRoot, 'apps', app, 'package.json');
    const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
    if (pkg.version) {
      return pkg.version;
    }
  } catch (error) {
    // Ignore
  }
  
  console.warn(`Unable to detect latest tag for ${app}`);
  return '';
}

function relativeToRepo(filePath) {
  return filePath.startsWith(repoRoot)
    ? filePath.slice(repoRoot.length + 1)
    : filePath;
}

function printHelp() {
  console.log(`
Usage: release-sync-versions.mjs --app <app-name> [options]

Syncs version numbers across manifest files for a specific app.

Options:
  --app, -a <name>     App name (required) - must exist in apps/ directory
  --version, -v <ver>  Target version (optional - auto-detected from tags)
  --dry-run            Preview changes without writing files
  --help, -h           Show this help message

Examples:
  node release-sync-versions.mjs --app myapp
  node release-sync-versions.mjs --app myapp --version 1.2.3
  node release-sync-versions.mjs --app myapp --dry-run

Environment:
  VERSION              Can also be set via environment variable
`);
}
