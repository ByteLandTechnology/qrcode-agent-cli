import { loadReleaseConfig, runCommand } from "./release-helpers.mjs";

const config = loadReleaseConfig();
const explicitTargets = process.argv.slice(2);
const targets =
  explicitTargets.length > 0
    ? explicitTargets
    : config.artifactTargets.map((entry) => entry.target);

if (targets.length === 0) {
  throw new Error("No artifact targets configured.");
}

for (const target of targets) {
  runCommand(process.execPath, [
    "scripts/release/build-cli-artifact.mjs",
    target,
  ]);
}

console.log(
  `Built ${targets.length} configured release artifact target(s): ${targets.join(", ")}.`,
);
