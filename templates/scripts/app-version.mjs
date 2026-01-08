#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const appPackageJson = join(__dirname, '..', 'apps', 'ghostnn.ai', 'package.json');

const pkg = JSON.parse(readFileSync(appPackageJson, 'utf8'));
const version = typeof pkg.version === 'string' ? pkg.version.trim() : '';

if (!version) {
  console.error('Unable to read version from apps/ghostnn.ai/package.json');
  process.exit(1);
}

process.stdout.write(version);


