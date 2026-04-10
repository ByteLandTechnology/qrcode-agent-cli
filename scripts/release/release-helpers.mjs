import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);

export function normalizePath(value) {
  return value.replace(/\\/g, "/");
}

export function relativeToRoot(value) {
  return normalizePath(path.relative(rootDir, value));
}

export function ensureDir(directoryPath) {
  mkdirSync(directoryPath, { recursive: true });
}

export function ensureCleanDir(directoryPath) {
  rmSync(directoryPath, { recursive: true, force: true });
  ensureDir(directoryPath);
}

export function runCommand(command, args, options = {}) {
  return execFileSync(command, args, {
    cwd: options.cwd ?? rootDir,
    env: {
      ...process.env,
      ...options.env,
    },
    stdio: options.stdio ?? "inherit",
    encoding: options.encoding,
  });
}

export function loadReleaseConfig() {
  return JSON.parse(
    readFileSync(
      path.join(rootDir, "release/skill-release.config.json"),
      "utf8",
    ),
  );
}

export function readJson(filePath, fallbackValue = null) {
  if (!existsSync(filePath)) {
    return fallbackValue;
  }

  return JSON.parse(readFileSync(filePath, "utf8"));
}

export function ensureFile(filePath, description) {
  if (!existsSync(filePath) || !statSync(filePath).isFile()) {
    throw new Error(`${description} is missing: ${relativeToRoot(filePath)}.`);
  }
}

export function copyFile(sourcePath, destinationPath) {
  ensureDir(path.dirname(destinationPath));
  copyFileSync(sourcePath, destinationPath);
}

export function writeJson(filePath, value) {
  ensureDir(path.dirname(filePath));
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

export function computeSha256(filePath) {
  return createHash("sha256").update(readFileSync(filePath)).digest("hex");
}

export function isPlaceholderValue(value) {
  return typeof value === "string" && value.includes("REPLACE_WITH_");
}

export function isPlaceholderRepository(value) {
  return !value || value.includes("REPLACE_WITH_OWNER/REPO");
}

export function requiredArtifactTargets(config) {
  return config.artifactTargets.filter((entry) => entry.required !== false);
}

export function getArtifactTarget(config, target) {
  const match = config.artifactTargets.find((entry) => entry.target === target);
  if (!match) {
    throw new Error(`Unknown artifact target: ${target}.`);
  }
  return match;
}

export function archiveFilenameForTarget(config, version, target) {
  const artifactTarget = getArtifactTarget(config, target);
  const archiveFormat = artifactTarget.archiveFormat || "tar.gz";
  return `${config.sourceSkillId}-${version}-${target}.${archiveFormat}`;
}

export function checksumFilenameForArchive(archiveFilename) {
  return `${archiveFilename}.sha256`;
}

export function releaseArtifactsDir(config) {
  return path.join(rootDir, config.artifactBuild.artifactsDir);
}

export function targetArtifactsDir(config, target) {
  return path.join(releaseArtifactsDir(config), target);
}

export function targetBuildMetadataPath(config, target) {
  return path.join(targetArtifactsDir(config, target), "build-metadata.json");
}

export function releaseBuildBinaryPath(config, target) {
  const extension = target.includes("windows") ? ".exe" : "";
  return path.join(
    targetArtifactsDir(config, target),
    "binary",
    `${config.artifactBuild.binaryName}${extension}`,
  );
}

export function buildBinaryFromProjectPath(config, target) {
  const extension = target.includes("windows") ? ".exe" : "";
  return path.join(
    rootDir,
    "target",
    target,
    "release",
    `${config.artifactBuild.binaryName}${extension}`,
  );
}

export function releaseAssetsDir(config) {
  return path.join(rootDir, config.githubRelease.releaseAssetsDir);
}

export function releaseEvidenceFilename(config) {
  return config.githubRelease.releaseEvidenceFilename;
}

export function releaseEvidencePath(config) {
  return path.join(releaseAssetsDir(config), releaseEvidenceFilename(config));
}

export function installScriptRelativePath(config) {
  return config.githubRelease.installScriptPath;
}

export function installScriptAbsolutePath(config) {
  return path.join(rootDir, installScriptRelativePath(config));
}

export function resolveOwnerRepository(config) {
  const resolved =
    process.env[config.githubRelease.ownerRepositoryEnv] ||
    config.githubRelease.ownerRepository;

  if (isPlaceholderRepository(resolved)) {
    throw new Error(
      [
        "Release repository identity is unresolved.",
        "Set GITHUB_REPOSITORY or replace REPLACE_WITH_OWNER/REPO in release/skill-release.config.json.",
      ].join(" "),
    );
  }

  return resolved;
}

export function resolveSourceRepository(config) {
  const configured =
    process.env[config.sourceRepositoryEnv || "GITHUB_REPOSITORY"] ||
    config.sourceRepository;

  if (!configured || isPlaceholderRepository(configured)) {
    return resolveOwnerRepository(config);
  }

  return configured;
}

export function sourceReleaseUrl(ownerRepository, gitTag) {
  return `https://github.com/${ownerRepository}/releases/tag/${gitTag}`;
}

export function detectGitHead() {
  return runCommand("git", ["rev-parse", "HEAD"], {
    cwd: rootDir,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}
