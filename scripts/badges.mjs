// Writes the shields.io endpoint files behind the README's build and test badges.
// Usage: node scripts/badges.mjs <jobs.json> <results.json> <output directory>
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [jobsFile, resultsFile, outDir] = process.argv.slice(2);
if (!outDir) {
  throw new Error('usage: badges.mjs <jobs.json> <results.json> <outdir>');
}

const badge = (label, counts) => ({
  schemaVersion: 1,
  label,
  message: counts ? `${counts.passed}/${counts.total} passing` : 'no results',
  color: counts?.ok ? 'brightgreen' : 'red',
});

// One job per platform, named "build / <platform>" by the reusable workflow.
const jobs = JSON.parse(readFileSync(jobsFile, 'utf8')).filter((job) =>
  job.name.startsWith('build / '),
);
const built = jobs.filter((job) => job.conclusion === 'success').length;
const builds = jobs.length
  ? { passed: built, total: jobs.length, ok: built === jobs.length }
  : null;

// Missing if the upstream tests did not get as far as running.
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

mkdirSync(outDir, { recursive: true });
for (const [file, content] of [
  ['builds.json', badge('builds', builds)],
  ['tests.json', badge('libdave tests', tests)],
]) {
  writeFileSync(join(outDir, file), `${JSON.stringify(content)}\n`);
  console.log(`${content.label}: ${content.message}`);
}
