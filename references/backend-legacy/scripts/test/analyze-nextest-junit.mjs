#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      continue;
    }
    args[key.slice(2)] = value;
  }
  return args;
}

function parseAttributes(attributeText) {
  const attributes = {};
  const attributePattern = /([A-Za-z_:][A-Za-z0-9_.:-]*)="([^"]*)"/g;
  for (const match of attributeText.matchAll(attributePattern)) {
    attributes[match[1]] = match[2];
  }
  return attributes;
}

function percentile(sortedValues, ratio) {
  if (sortedValues.length === 0) {
    return 0;
  }
  const index = Math.min(
    sortedValues.length - 1,
    Math.max(0, Math.ceil(sortedValues.length * ratio) - 1),
  );
  return sortedValues[index];
}

function progressBar(completed, total, width = 30) {
  if (total === 0) {
    return `[${"-".repeat(width)}]`;
  }
  const filled = Math.min(width, Math.max(0, Math.round((completed / total) * width)));
  return `[${"#".repeat(filled)}${"-".repeat(width - filled)}]`;
}

const args = parseArgs(process.argv.slice(2));
const input = args.input;
const outputBase = args.output;
const suiteName = args.suite ?? "unknown";

if (!input || !outputBase) {
  console.error("Usage: analyze-nextest-junit.mjs --input <junit.xml> --output <path-prefix> [--suite <name>]");
  process.exit(2);
}

const xml = readFileSync(input, "utf8");
const testCasePattern = /<testcase\b([^>]*)>([\s\S]*?)<\/testcase>|<testcase\b([^>]*)\/>/g;
const tests = [];

for (const match of xml.matchAll(testCasePattern)) {
  const attributeText = match[1] ?? match[3] ?? "";
  const body = match[2] ?? "";
  const attrs = parseAttributes(attributeText);
  const durationSeconds = Number.parseFloat(attrs.time ?? "0");
  tests.push({
    classname: attrs.classname ?? "unknown",
    name: attrs.name ?? "unknown",
    timeSeconds: Number.isFinite(durationSeconds) ? durationSeconds : 0,
    failed: body.includes("<failure") || body.includes("<error"),
    flaky: body.includes("<flakyFailure"),
  });
}

const durations = tests.map((test) => test.timeSeconds).sort((left, right) => left - right);
const totalSeconds = durations.reduce((sum, value) => sum + value, 0);
const topSlow = [...tests]
  .sort((left, right) => right.timeSeconds - left.timeSeconds)
  .slice(0, 10)
  .map((test) => ({
    id: `${test.classname}::${test.name}`,
    timeSeconds: Number(test.timeSeconds.toFixed(3)),
    failed: test.failed,
    flaky: test.flaky,
  }));

const report = {
  suite: suiteName,
  generatedAt: new Date().toISOString(),
  source: path.relative(process.cwd(), input),
  totalTests: tests.length,
  failedTests: tests.filter((test) => test.failed).length,
  flakyTests: tests.filter((test) => test.flaky).map((test) => `${test.classname}::${test.name}`),
  totalSeconds: Number(totalSeconds.toFixed(3)),
  averageSeconds: Number((tests.length === 0 ? 0 : totalSeconds / tests.length).toFixed(3)),
  p95Seconds: Number(percentile(durations, 0.95).toFixed(3)),
  p99Seconds: Number(percentile(durations, 0.99).toFixed(3)),
  slowest: topSlow,
};

const markdown = [
  `# ${suiteName} nextest report`,
  "",
  `- Generated at: ${report.generatedAt}`,
  `- Source JUnit: \`${report.source}\``,
  `- Total tests: ${report.totalTests}`,
  `- Failed tests: ${report.failedTests}`,
  `- Flaky tests: ${report.flakyTests.length === 0 ? "none detected" : report.flakyTests.length}`,
  `- Total time: ${report.totalSeconds}s`,
  `- Average time: ${report.averageSeconds}s`,
  `- p95: ${report.p95Seconds}s`,
  `- p99: ${report.p99Seconds}s`,
  "",
  "## Slowest tests",
  "",
  ...report.slowest.map(
    (test, index) =>
      `${index + 1}. \`${test.id}\` - ${test.timeSeconds}s${test.flaky ? " (flaky)" : ""}${test.failed ? " (failed)" : ""}`,
  ),
  "",
].join("\n");

mkdirSync(path.dirname(outputBase), { recursive: true });
writeFileSync(`${outputBase}.json`, `${JSON.stringify(report, null, 2)}\n`);
writeFileSync(`${outputBase}.md`, markdown);

const passedTests = report.totalTests - report.failedTests;
const reportPath = path.relative(process.cwd(), `${outputBase}.md`);
console.log(
  `${suiteName} ${progressBar(report.totalTests, report.totalTests)} ${report.totalTests}/${report.totalTests} completed | passed ${passedTests} | failed ${report.failedTests} | flaky ${report.flakyTests.length} | total ${report.totalSeconds}s`,
);
console.log(`Report: ${reportPath}`);
