#!/usr/bin/env node

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const testsDir = path.join(repoRoot, "backend", "tests");

const TOP_LEVEL_TARGETS = readdirSync(testsDir, { withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
  .map((entry) => entry.name.replace(/\.rs$/, ""))
  .sort();

function categoryForTarget(target) {
  if (target.includes("architecture")) {
    return "architecture";
  }

  if (target.includes("snapshot")) {
    return "snapshot";
  }

  if (
    target.endsWith("_api") ||
    target.endsWith("_stream_api") ||
    target.endsWith("_websocket_api") ||
    target.endsWith("_long_poll_api") ||
    target.includes("connectrpc") ||
    target === "hard_v1_routes" ||
    target === "omniroute"
  ) {
    return "e2e";
  }

  return "integration";
}

function countRustTests(directories) {
  const pattern = /#\[(?:tokio::)?test\]/g;
  let count = 0;
  for (const directory of directories) {
    const output = BunLikeWalk(directory);
    for (const file of output) {
      if (!file.endsWith(".rs")) {
        continue;
      }
      const content = readFileSync(file, "utf8");
      count += [...content.matchAll(pattern)].length;
    }
  }
  return count;
}

function BunLikeWalk(directory) {
  const entries = readdirSync(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...BunLikeWalk(fullPath));
    } else {
      files.push(fullPath);
    }
  }
  return files;
}

const groupedTargets = {
  architecture: [],
  snapshot: [],
  e2e: [],
  integration: [],
};

for (const target of TOP_LEVEL_TARGETS) {
  groupedTargets[categoryForTarget(target)].push(target);
}

const summary = {
  generatedAt: new Date().toISOString(),
  rustUnitTestAttributes: countRustTests([
    path.join(repoRoot, "backend", "src"),
    path.join(repoRoot, "crates", "testkit", "src"),
  ]),
  backendIntegrationTargets: TOP_LEVEL_TARGETS.length,
  categories: Object.fromEntries(
    Object.entries(groupedTargets).map(([category, targets]) => [
      category,
      {
        count: targets.length,
        targets,
      },
    ]),
  ),
};

const mode = process.argv[2] ?? "summary";
const requestedCategory = process.argv[3];

if (mode === "targets") {
  if (!requestedCategory || !groupedTargets[requestedCategory]) {
    console.error("Usage: backend-test-targets.mjs targets <architecture|snapshot|e2e|integration>");
    process.exit(2);
  }

  process.stdout.write(groupedTargets[requestedCategory].join(" "));
  process.exit(0);
}

process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
