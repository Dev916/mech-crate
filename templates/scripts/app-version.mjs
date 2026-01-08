#!/usr/bin/env node

/**
 * App Version
 * 
 * Reads and outputs the current version of a specified app.
 * 
 * Usage:
 *   node app-version.mjs --app myapp
 *   node app-version.mjs -a myapp
 */

import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = join(__dirname, '..');

// Parse CLI arguments
const args = process.argv.slice(2);
let appName;

for (let i = 0; i < args.length; i += 1) {
  const arg = args[i];
  if (arg === '--app' || arg === '-a') {
    appName = args[i + 1];
    if (!appName) {
      console.error('Missing value after --app flag.');
      process.exit(1);
    }
    i += 1;
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

const appPackageJson = join(repoRoot, 'apps', appName, 'package.json');

if (!existsSync(appPackageJson)) {
  console.error(`Unable to find package.json for app: ${appName}`);
  console.error(`Expected path: apps/${appName}/package.json`);
  process.exit(1);
}

try {
  const pkg = JSON.parse(readFileSync(appPackageJson, 'utf8'));
  const version = typeof pkg.version === 'string' ? pkg.version.trim() : '';

  if (!version) {
    console.error(`Unable to read version from apps/${appName}/package.json`);
    process.exit(1);
  }

  process.stdout.write(version);
} catch (error) {
  console.error(`Error reading package.json for ${appName}: ${error.message}`);
  process.exit(1);
}

function printHelp() {
  console.log(`
Usage: app-version.mjs --app <app-name>

Reads and outputs the current version of a specified app.

Options:
  --app, -a <name>  App name (required) - must exist in apps/ directory
  --help, -h        Show this help message

Examples:
  node app-version.mjs --app myapp
  node app-version.mjs -a web
`);
}
