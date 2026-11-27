// Runs Discord's own libdave tests against the libraries this repository builds.
// Only the pinned submodule is used; nothing is downloaded.
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const submodule = 'crates/libdave-sys/vendor/libdave';
const build = join(root, 'target', 'upstream-tests');

const run = (command, args) =>
  execFileSync(command, args, { cwd: root, stdio: 'inherit' });
const read = (command, args) =>
  execFileSync(command, args, {
    cwd: root,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'inherit'],
  }).trim();

if (process.platform === 'win32') {
  throw new Error('the upstream tests are only supported on Linux and macOS');
}
if (!existsSync(join(root, submodule, 'cpp', 'test'))) {
  throw new Error('submodules are missing: run `git submodule update --init`');
}

// Refuse to test any libdave other than the one this repository pins.
const pinned = read('git', ['ls-files', '--stage', submodule]).split(/\s+/)[1];
const actual = read('git', ['-C', submodule, 'rev-parse', 'HEAD']);
const modified = read('git', ['-C', submodule, 'status', '--porcelain']);
if (actual !== pinned || modified) {
  throw new Error(
    `libdave must be unmodified at the pinned commit ${pinned} (found ${actual}${
      modified ? ', with local changes' : ''
    }): run \`git submodule update --init\``,
  );
}
console.log(`Testing libdave ${pinned}`);

// Ask cargo where build.rs and openssl-sys put their libraries.
const scripts = read('cargo', [
  'build',
  '--locked',
  '-p',
  'libdave-rs-sys',
  '--message-format=json-render-diagnostics',
])
  .split('\n')
  .map((line) => JSON.parse(line))
  .filter((message) => message.reason === 'build-script-executed');
const script = (name) => {
  const found = scripts.find((message) =>
    new RegExp(`[/#]${name}[@#]`).test(message.package_id),
  );
  if (!found) throw new Error(`cargo reported no build script for ${name}`);
  return found;
};
const daveOut = script('(?:libdave-sys|libdave-rs-sys)').out_dir;
const openssl = script('openssl-sys')
  .linked_paths.map((path) => path.replace(/^native=/, ''))
  .find((path) => existsSync(join(path, 'libcrypto.a')));
if (!openssl) throw new Error('could not find the vendored libcrypto.a');

run('cmake', [
  '-S',
  'tests/upstream',
  '-B',
  build,
  `-DDAVE_SYS_OUT_DIR=${daveOut}`,
  `-DOPENSSL_INSTALL_LIB_DIR=${openssl}`,
]);
run('cmake', ['--build', build, '--parallel']);

// Counts for the README badge. GoogleTest writes its own report; the C API tests
// only print a summary, which ctest keeps in its log.
const gtestReport = join(build, 'gtest.json');
const ctestLog = join(build, 'Testing', 'Temporary', 'LastTest.log');
const results = join(build, 'results.json');
for (const file of [gtestReport, ctestLog, results]) {
  rmSync(file, { force: true });
}
try {
  execFileSync('ctest', ['--test-dir', build, '--output-on-failure'], {
    cwd: root,
    stdio: 'inherit',
    env: { ...process.env, GTEST_OUTPUT: `json:${gtestReport}` },
  });
} finally {
  writeResults();
}

function writeResults() {
  let passed = 0;
  let total = 0;
  let suites = 0;
  if (existsSync(gtestReport)) {
    const report = JSON.parse(readFileSync(gtestReport, 'utf8'));
    const ran = report.testsuites
      .flatMap((suite) => suite.testsuite)
      .filter((test) => test.result === 'COMPLETED');
    passed += ran.filter((test) => !test.failures).length;
    total += ran.length;
    suites += 1;
  }
  const capi =
    existsSync(ctestLog) &&
    /^Passed: (\d+)\s+Failed: \d+\s+Total:\s+(\d+)/m.exec(
      readFileSync(ctestLog, 'utf8'),
    );
  if (capi) {
    passed += Number(capi[1]);
    total += Number(capi[2]);
    suites += 1;
  }
  const complete = suites === 2;
  writeFileSync(results, `${JSON.stringify({ passed, total, complete })}\n`);
  console.log(`Upstream tests: ${passed}/${total} passed`);
}
