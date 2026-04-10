import { chmodSync, existsSync, readFileSync } from "node:fs";
import path from "node:path";
import {
  installScriptAbsolutePath,
  installScriptRelativePath,
  loadReleaseConfig,
  releaseEvidenceFilename,
  rootDir,
  runCommand,
} from "./release-helpers.mjs";

const config = loadReleaseConfig();
const readme = readFileSync(path.join(rootDir, "README.md"), "utf8");
const skillDoc = readFileSync(path.join(rootDir, "SKILL.md"), "utf8");
const installScriptPath = installScriptAbsolutePath(config);

if (!existsSync(installScriptPath)) {
  throw new Error(
    `Install helper is missing: ${installScriptRelativePath(config)}.`,
  );
}

chmodSync(installScriptPath, 0o755);

if (!readme.includes("scripts/install-current-release.sh")) {
  throw new Error(
    "README.md must document scripts/install-current-release.sh.",
  );
}

if (!readme.includes(releaseEvidenceFilename(config))) {
  throw new Error("README.md must mention release-evidence.json.");
}

if (!skillDoc.includes("GitHub Release")) {
  throw new Error("SKILL.md must mention GitHub Release installation.");
}

runCommand("cargo", ["fmt", "--check"]);
runCommand("cargo", ["clippy", "--", "-D", "warnings"]);
runCommand("cargo", ["test"]);

console.log("Release quality gates passed for qrcode-agent-cli.");
