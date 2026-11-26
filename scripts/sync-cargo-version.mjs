// Copies the version in package.json to the Cargo manifests and lockfile, so
// `npm version` keeps the npm packages and the crates in step.
import { readFileSync, writeFileSync } from 'node:fs';

const { version } = JSON.parse(readFileSync('package.json', 'utf8'));

function replace(path, pattern, expected) {
  let count = 0;
  const text = readFileSync(path, 'utf8').replace(
    pattern,
    (_, before, after) => {
      count += 1;
      return `${before}${version}${after}`;
    },
  );
  if (count !== expected) {
    throw new Error(`${path}: expected ${expected} version(s), found ${count}`);
  }
  writeFileSync(path, text);
}

replace('Cargo.toml', /(\[workspace\.package\]\nversion = ")[^"]+(")/g, 1);
replace(
  'crates/libdave/Cargo.toml',
  /(libdave-rs-sys = \{[^}]*version = ")[^"]+(")/g,
  1,
);
replace(
  'Cargo.lock',
  /(name = "libdave-(?:rs|rs-sys|node)"\nversion = ")[^"]+(")/g,
  3,
);
