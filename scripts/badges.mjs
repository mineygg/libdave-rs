// Writes the shields.io endpoint files behind the README's build and test badges.
// Usage: node scripts/badges.mjs <jobs.json> <results directory> <output directory>
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join } from 'node:path';

const [jobsFile, resultsDir, outDir] = process.argv.slice(2);
if (!outDir) {
  throw new Error('usage: badges.mjs <jobs.json> <resultsdir> <outdir>');
}

const badge = (label, counts) => ({
  schemaVersion: 1,
  label,
  message: counts ? `${counts.passed}/${counts.total} passing` : 'no results',
  color: counts?.ok ? 'brightgreen' : 'red',
});

// One job per platform, named "build / <platform>" by the reusable workflow.
const allJobs = JSON.parse(readFileSync(jobsFile, 'utf8'));
const jobs = allJobs.filter((job) => job.name.startsWith('build / '));
const built = jobs.filter((job) => job.conclusion === 'success').length;
const builds = jobs.length
  ? { passed: built, total: jobs.length, ok: built === jobs.length }
  : null;

// Missing if the upstream tests did not get as far as running.
const resultsFile = join(resultsDir, 'results.json');
let tests = null;
if (existsSync(resultsFile)) {
  const { passed, total, complete } = JSON.parse(
    readFileSync(resultsFile, 'utf8'),
  );
  // A suite that crashed reports no count, so a partial total would mislead.
  if (complete && total > 0) {
    tests = { passed, total, ok: passed === total };
  }
}

// This repository's Rust tests, read from the summary line of each test binary.
function rustTests() {
  const log = join(resultsDir, 'rust-test.log');
  if (!existsSync(log)) return null;
  const text = readFileSync(log, 'utf8');
  let passed = 0;
  let failed = 0;
  for (const match of text.matchAll(
    /test result: \w+\. (\d+) passed; (\d+) failed;/g,
  )) {
    passed += Number(match[1]);
    failed += Number(match[2]);
  }
  // A crashed binary, or a failed job without a failed test, hides tests.
  const job = allJobs.find((j) => j.name === 'cargo test');
  const hidden =
    text.includes('(signal: ') || (job?.conclusion !== 'success' && !failed);
  return hidden ? null : { passed, total: passed + failed };
}

// The Node tests run once per platform. Each test counts once, and passes only
// if it passed on every platform.
function nodeTests() {
  const platforms = allJobs.filter((job) => job.name.startsWith('node '));
  const files = existsSync(resultsDir)
    ? readdirSync(resultsDir).filter((file) => /^node-.+\.json$/.test(file))
    : [];
  if (!files.length || files.length !== platforms.length) return null;

  const outcomes = new Map();
  for (const file of files) {
    const report = JSON.parse(readFileSync(join(resultsDir, file), 'utf8'));
    for (const suite of report.testResults) {
      // A file that failed to load lists no tests.
      if (suite.status === 'failed' && !suite.assertionResults.length) {
        return null;
      }
      for (const test of suite.assertionResults) {
        if (!['passed', 'failed'].includes(test.status)) continue;
        const name = `${suite.name.split(/[\\/]/).pop()} ${test.fullName}`;
        outcomes.set(
          name,
          (outcomes.get(name) ?? true) && test.status === 'passed',
        );
      }
    }
  }
  const passed = [...outcomes.values()].filter(Boolean).length;
  return { passed, total: outcomes.size };
}

const rust = rustTests();
const node = nodeTests();
let suite = null;
if (rust?.total && node?.total) {
  const passed = rust.passed + node.passed;
  const total = rust.total + node.total;
  // A job can fail after its tests pass (typecheck, packaging); stay honest.
  const ok = allJobs
    .filter((j) => j.name === 'cargo test' || j.name.startsWith('node '))
    .every((j) => j.conclusion === 'success');
  suite = { passed, total, ok: ok && passed === total };
}

mkdirSync(outDir, { recursive: true });
for (const [file, content] of [
  ['builds.json', badge('builds', builds)],
  ['tests.json', badge('libdave tests', tests)],
  ['suite.json', badge('tests', suite)],
]) {
  writeFileSync(join(outDir, file), `${JSON.stringify(content)}\n`);
  console.log(`${content.label}: ${content.message}`);
}
