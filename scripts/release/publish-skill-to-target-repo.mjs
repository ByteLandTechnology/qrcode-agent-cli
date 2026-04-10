import { existsSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import {
  archiveFilenameForTarget,
  checksumFilenameForArchive,
  computeSha256,
  detectGitHead,
  ensureCleanDir,
  getArtifactTarget,
  installScriptRelativePath,
  loadReleaseConfig,
  readJson,
  relativeToRoot,
  releaseAssetsDir,
  releaseBuildBinaryPath,
  releaseEvidenceFilename,
  releaseEvidencePath,
  resolveOwnerRepository,
  resolveSourceRepository,
  rootDir,
  runCommand,
  sourceReleaseUrl,
  targetBuildMetadataPath,
  writeJson,
} from "./release-helpers.mjs";

const [version, gitTag, gitHeadArg] = process.argv.slice(2);

if (!version || !gitTag) {
  throw new Error(
    "Usage: node scripts/release/publish-skill-to-target-repo.mjs <version> <gitTag> [gitHead]",
  );
}

const config = loadReleaseConfig();
const ownerRepository = resolveOwnerRepository(config);
const sourceRepository = resolveSourceRepository(config);
const gitHead = gitHeadArg || detectGitHead();
const publishedAt = new Date().toISOString();
const githubReleaseUrl = sourceReleaseUrl(ownerRepository, gitTag);
const assetsDir = releaseAssetsDir(config);
const receiptPath = path.join(
  rootDir,
  ".work/release/last-publication-receipt.json",
);
const manifestPath = path.join(rootDir, config.metadataFilename);

function ensureBuildMetadata(target) {
  const metadata = readJson(targetBuildMetadataPath(config, target));
  if (!metadata) {
    throw new Error(
      `Missing build metadata for ${target}. Run release:build-artifact for every required target first.`,
    );
  }

  return metadata;
}

function packageArtifactForTarget(target) {
  const targetConfig = getArtifactTarget(config, target);
  const buildMetadata = ensureBuildMetadata(target);
  const binaryPath = releaseBuildBinaryPath(config, target);

  if (!existsSync(binaryPath) || !statSync(binaryPath).isFile()) {
    throw new Error(`Prepared binary for ${target} is missing: ${binaryPath}.`);
  }

  const archiveFilename = archiveFilenameForTarget(config, version, target);
  const archivePath = path.join(assetsDir, archiveFilename);
  const checksumFilename = checksumFilenameForArchive(archiveFilename);
  const checksumPath = path.join(assetsDir, checksumFilename);
  const archiveFormat = targetConfig.archiveFormat || "tar.gz";

  if (archiveFormat === "zip") {
    runCommand("zip", ["-j", archivePath, binaryPath]);
  } else {
    runCommand("tar", [
      "-czf",
      archivePath,
      "-C",
      path.dirname(binaryPath),
      path.basename(binaryPath),
    ]);
  }

  const sha256 = computeSha256(archivePath);
  writeFileSync(checksumPath, `${sha256}  ${archiveFilename}\n`, "utf8");

  return {
    archiveFilename,
    archivePath: relativeToRoot(archivePath),
    artifactOrigin: buildMetadata.artifactOrigin,
    binaryName: buildMetadata.binaryName,
    checksumFilename,
    checksumPath: relativeToRoot(checksumPath),
    required: targetConfig.required !== false,
    runner: targetConfig.runner,
    sha256,
    targetVariant: target,
  };
}

ensureCleanDir(assetsDir);

const artifactResults = config.artifactTargets.map((targetConfig) =>
  packageArtifactForTarget(targetConfig.target),
);

const releaseEvidence = {
  artifactResults,
  generatedPackageBoundary: config.generatedPackageBoundary,
  githubRelease: {
    assetDirectory: relativeToRoot(assetsDir),
    installCommandExample: config.githubRelease.installCommandExample,
    installScriptPath: installScriptRelativePath(config),
    ownerRepository,
    releaseEvidenceFilename: releaseEvidenceFilename(config),
    releaseUrl: githubReleaseUrl,
  },
  metadataVersion: 1,
  publicationMode:
    process.env.GITHUB_ACTIONS === "true" ? "live_release" : "dry_run",
  publishedAt,
  sourceCommitSha: gitHead,
  sourceGitTag: gitTag,
  sourceRepository,
  sourceSkillId: config.sourceSkillId,
  sourceVersion: version,
};

writeJson(releaseEvidencePath(config), releaseEvidence);

const manifest = {
  artifactResults,
  githubReleaseAssets: config.githubRelease.assetGlobPatterns,
  publicationMode: releaseEvidence.publicationMode,
  publishedAt,
  releaseEvidencePath: relativeToRoot(releaseEvidencePath(config)),
  releaseUrl: githubReleaseUrl,
  sourceCommitSha: gitHead,
  sourceGitTag: gitTag,
  sourceRepository,
  sourceSkillId: config.sourceSkillId,
  sourceVersion: version,
};

writeJson(manifestPath, manifest);

const receipt = {
  artifactResults,
  blockingReason: null,
  githubReleaseUrl,
  installScriptPath: installScriptRelativePath(config),
  optionalSecondaryPublicationEnabled: false,
  publicationMode: releaseEvidence.publicationMode,
  publicationResult:
    releaseEvidence.publicationMode === "live_release"
      ? "published"
      : "prepared",
  publishRoot: relativeToRoot(assetsDir),
  publishedAt,
  releaseEvidencePath: relativeToRoot(releaseEvidencePath(config)),
  runResult:
    releaseEvidence.publicationMode === "live_release"
      ? "published"
      : "prepared",
  sourceCommitSha: gitHead,
  sourceGitTag: gitTag,
  sourceRepository,
  sourceSkillId: config.sourceSkillId,
  sourceVersion: version,
};

writeJson(receiptPath, receipt);
process.stdout.write(`${JSON.stringify(receipt)}\n`);
