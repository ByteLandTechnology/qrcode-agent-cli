import { existsSync } from "node:fs";
import semanticRelease from "semantic-release";
import {
  detectGitHead,
  loadReleaseConfig,
  releaseEvidencePath,
  resolveOwnerRepository,
  rootDir,
  sourceReleaseUrl,
  writeJson,
} from "./release-helpers.mjs";

const dryRun = process.argv.includes("--dry-run");
const noCi = process.argv.includes("--no-ci");

const result = await semanticRelease(
  {
    ci: !noCi,
    dryRun,
  },
  {
    cwd: rootDir,
    env: process.env,
    stderr: process.stderr,
    stdout: process.stdout,
  },
);

const config = loadReleaseConfig();
const receiptPath = `${rootDir}/.work/release/last-publication-receipt.json`;

if (!existsSync(receiptPath)) {
  const ownerRepository = (() => {
    try {
      return resolveOwnerRepository(config);
    } catch {
      return config.githubRelease.ownerRepository;
    }
  })();

  const nextRelease = result && result !== false ? result.nextRelease : null;
  const gitTag = nextRelease?.gitTag ?? null;
  const receipt = {
    artifactResults: [],
    blockingReason:
      result === false
        ? "semantic-release found no releasable changes"
        : "semantic-release completed without generating release assets",
    githubReleaseUrl: gitTag ? sourceReleaseUrl(ownerRepository, gitTag) : null,
    installScriptPath: config.githubRelease.installScriptPath,
    optionalSecondaryPublicationEnabled: false,
    publicationMode: dryRun
      ? "dry_run"
      : process.env.GITHUB_ACTIONS === "true"
        ? "live_release"
        : "report_only",
    publicationResult: "skipped",
    publishRoot: ".work/release/github-release",
    publishedAt: new Date().toISOString(),
    releaseEvidencePath: existsSync(releaseEvidencePath(config))
      ? ".work/release/github-release/release-evidence.json"
      : null,
    runResult: result === false ? "no_release" : "prepared",
    sourceCommitSha: (() => {
      try {
        return detectGitHead();
      } catch {
        return null;
      }
    })(),
    sourceGitTag: gitTag,
    sourceRepository: ownerRepository,
    sourceSkillId: config.sourceSkillId,
    sourceVersion: nextRelease?.version ?? null,
  };

  writeJson(receiptPath, receipt);
}
