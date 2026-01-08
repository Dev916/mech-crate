#!/usr/bin/env node

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = join(__dirname, '..');

const manifestPath = join(repoRoot, '.release-please-manifest.json');
const packageJsonPath = join(repoRoot, 'apps', 'ghostnn.ai', 'package.json');
const packageLockPath = join(repoRoot, 'apps', 'ghostnn.ai', 'package-lock.json');

const args = process.argv.slice(2);
let cliVersion;
let dryRun = false;

for (let i = 0; i < args.length; i += 1) {
  const arg = args[i];
  if (arg === '--version' || arg === '-v') {
    cliVersion = args[i + 1];
    if (!cliVersion) {
      console.error('Missing value after --version flag.');
      process.exit(1);
    }
    i += 1;
  } else if (arg === '--dry-run') {
    dryRun = true;
  } else {
    console.error(`Unknown argument: ${arg}`);
    process.exit(1);
  }
}

const envVersion = (process.env.VERSION ?? '').trim();
const targetVersion =
  (cliVersion ?? envVersion ?? '').trim() || detectLatestReleaseVersion();

if (!targetVersion) {
  console.error(
    'Unable to determine target version. Provide --version, set VERSION=, or create a ghostnn-ai-v* tag.'
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

updateJson(manifestPath, data => {
  if (typeof data !== 'object' || data === null) {
    throw new Error('Manifest file must be a JSON object.');
  }
  if (!Object.prototype.hasOwnProperty.call(data, 'apps/ghostnn.ai')) {
    console.warn(
      'Manifest is missing "apps/ghostnn.ai" entry. It will be added automatically.'
    );
  }
  data['apps/ghostnn.ai'] = targetVersion;
});

updateJson(packageJsonPath, data => {
  data.version = targetVersion;
});

updateJson(packageLockPath, data => {
  data.version = targetVersion;
  if (data.packages && data.packages['']) {
    data.packages[''].version = targetVersion;
  }
});

const prefix = dryRun ? '[release-sync dry-run]' : '[release-sync]';
if (updatedFiles.length === 0) {
  console.log(`${prefix} All files already set to version ${targetVersion}.`);
} else {
  console.log(
    `${prefix} Updated ${updatedFiles.length} file(s) to version ${targetVersion}:`
  );
  updatedFiles.forEach(file => console.log(`  - ${relativeToRepo(file)}`));
}

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

function detectLatestReleaseVersion() {
  try {
    const output = execSync("git tag --list 'ghostnn-ai-v*' --sort=version:refname", {
      cwd: repoRoot,
      encoding: 'utf8'
    }).trim();
    if (!output) {
      return '';
    }
    const tags = output
      .split('\n')
      .map(line => line.trim())
      .filter(Boolean);
    if (tags.length === 0) {
      return '';
    }
    const latestTag = tags[tags.length - 1];
    return latestTag.replace(/^ghostnn-ai-v/, '');
  } catch (error) {
    console.warn('Unable to detect latest tag:', error.message);
    return '';
  }
}

function relativeToRepo(filePath) {
  return filePath.startsWith(repoRoot)
    ? filePath.slice(repoRoot.length + 1)
    : filePath;
}




