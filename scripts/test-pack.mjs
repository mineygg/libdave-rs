// Packs the umbrella package and this machine's platform package, installs both
// into an empty project and loads them through ESM and CommonJS.
import { execSync } from 'node:child_process';
import {
  copyFileSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const binaries = readdirSync(root).filter((file) => file.endsWith('.node'));
if (binaries.length !== 1) {
  throw new Error(`expected exactly one .node file, found ${binaries.length}`);
}
const [binary] = binaries;
const platform = binary.slice('libdave.'.length, -'.node'.length);
const platformDir = join(root, 'npm', platform);

const work = mkdtempSync(join(tmpdir(), 'libdave-pack-'));
const run = (command, cwd) => execSync(command, { cwd, stdio: 'inherit' });
const copied = join(platformDir, binary);
try {
  copyFileSync(join(root, binary), copied);
  run(`npm pack --ignore-scripts --pack-destination "${work}"`, platformDir);
  run(`npm pack --ignore-scripts --pack-destination "${work}"`, root);
  writeFileSync(join(work, 'package.json'), '{"private":true}');
  const tarballs = readdirSync(work)
    .filter((file) => file.endsWith('.tgz'))
    .map((file) => `"./${file}"`);
  run(
    `npm install --ignore-scripts --no-audit --no-fund ${tarballs.join(' ')}`,
    work,
  );
  const check = 'if (m.maxSupportedProtocolVersion() < 1) process.exit(1);';
  writeFileSync(
    join(work, 'check.mjs'),
    `import * as m from '@mineygg/libdave-rs';\n${check}\n`,
  );
  writeFileSync(
    join(work, 'check.cjs'),
    `const m = require('@mineygg/libdave-rs');\n${check}\n`,
  );
  run('node check.mjs', work);
  run('node check.cjs', work);
  console.log(`pack test ok (${platform})`);
} finally {
  rmSync(copied, { force: true });
  rmSync(work, { recursive: true, force: true });
}
