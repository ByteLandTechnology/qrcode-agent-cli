import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import {
  ensureFile,
  installScriptAbsolutePath,
  installScriptRelativePath,
  isPlaceholderRepository,
  isPlaceholderValue,
  loadReleaseConfig,
  requiredArtifactTargets,
  resolveOwnerRepository,
  rootDir,
  runCommand,
} from "./release-helpers.mjs";

const config = loadReleaseConfig();

function verifyRequiredValue(value, fieldPath) {
  if (!value) {
    throw new Error(`${fieldPath} is required.`);
  }

  if (isPlaceholderValue(value)) {
    throw new Error(
      `${fieldPath} still contains a REPLACE_WITH_* placeholder.`,
    );
  }
}

function ensureRepositoryHasCommits() {
  try {
    runCommand("git", ["rev-parse", "HEAD"], {
      cwd: rootDir,
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch {
    throw new Error(
      "The repository has no commits yet. Create the initial commit before running release automation.",
    );
  }
}

function ensureGitRemoteConfigured() {
  const remotes = runCommand("git", ["remote"], {
    cwd: rootDir,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  })
    .split(/\r?\n/)
    .map((entry) => entry.trim())
    .filter(Boolean);

  if (remotes.length === 0) {
    throw new Error(
      "No git remote is configured. Add the GitHub repository remote before releasing.",
    );
  }
}

verifyRequiredValue(config.sourceSkillId, "sourceSkillId");
verifyRequiredValue(
  config.generatedSkill.skillName,
  "generatedSkill.skillName",
);
verifyRequiredValue(
  config.generatedSkill.description,
  "generatedSkill.description",
);
verifyRequiredValue(config.generatedSkill.author, "generatedSkill.author");
verifyRequiredValue(
  config.artifactBuild.binaryName,
  "artifactBuild.binaryName",
);
verifyRequiredValue(
  config.githubRelease.installScriptPath,
  "githubRelease.installScriptPath",
);
verifyRequiredValue(
  config.githubRelease.releaseEvidenceFilename,
  "githubRelease.releaseEvidenceFilename",
);

if (config.generatedSkill.skillName !== config.sourceSkillId) {
  throw new Error("generatedSkill.skillName must match sourceSkillId.");
}

if (config.artifactBuild.binaryName !== config.sourceSkillId) {
  throw new Error("artifactBuild.binaryName must match sourceSkillId.");
}

ensureFile(path.join(rootDir, "package.json"), "package.json");
ensureFile(path.join(rootDir, ".releaserc.json"), ".releaserc.json");
ensureFile(
  path.join(rootDir, ".github/workflows/release.yml"),
  ".github/workflows/release.yml",
);
ensureFile(
  path.join(rootDir, "release/skill-release.config.json"),
  "release/skill-release.config.json",
);
ensureFile(
  path.join(rootDir, "scripts/release/verify-release-config.mjs"),
  "scripts/release/verify-release-config.mjs",
);
ensureFile(
  path.join(rootDir, "scripts/release/run-quality-gates.mjs"),
  "scripts/release/run-quality-gates.mjs",
);
ensureFile(
  path.join(rootDir, "scripts/release/build-cli-artifact.mjs"),
  "scripts/release/build-cli-artifact.mjs",
);
ensureFile(
  path.join(rootDir, "scripts/release/publish-skill-to-target-repo.mjs"),
  "scripts/release/publish-skill-to-target-repo.mjs",
);
ensureFile(
  path.join(rootDir, "scripts/release/run-semantic-release.mjs"),
  "scripts/release/run-semantic-release.mjs",
);
ensureFile(path.join(rootDir, "README.md"), "README.md");
ensureFile(path.join(rootDir, "SKILL.md"), "SKILL.md");

const installScriptPath = installScriptAbsolutePath(config);
ensureFile(installScriptPath, installScriptRelativePath(config));

const installScriptContents = readFileSync(installScriptPath, "utf8");
if (!installScriptContents.includes("releases/tag/")) {
  throw new Error(
    `${installScriptRelativePath(config)} must resolve artifacts from a tagged GitHub Release URL.`,
  );
}

if (
  !Array.isArray(config.artifactTargets) ||
  config.artifactTargets.length === 0
) {
  throw new Error("artifactTargets must contain at least one entry.");
}

const requiredTargets = requiredArtifactTargets(config).map(
  (entry) => entry.target,
);
for (const requiredTarget of [
  "x86_64-unknown-linux-gnu",
  "aarch64-apple-darwin",
]) {
  if (!requiredTargets.includes(requiredTarget)) {
    throw new Error(`Missing required artifact target: ${requiredTarget}.`);
  }
}

if (config.optionalSecondaryPublication?.enabled) {
  throw new Error(
    "optionalSecondaryPublication.enabled must remain false for repo-native publish mode.",
  );
}

if (isPlaceholderRepository(config.githubRelease.ownerRepository)) {
  resolveOwnerRepository(config);
}

if (!process.env.GITHUB_TOKEN && !process.env.GITHUB_ACTIONS) {
  throw new Error(
    "GITHUB_TOKEN is not set. Provide a token with release permissions before running dry-run or live release.",
  );
}

ensureRepositoryHasCommits();
ensureGitRemoteConfigured();

if (!existsSync(path.join(rootDir, "templates/release-support.md"))) {
  throw new Error(
    "templates/release-support.md is missing. Keep release support assets in templates/ for rehearsal and docs checks.",
  );
}

console.log(
  `Release configuration verified for repo-native publication in ${resolveOwnerRepository(config)}.`,
);
