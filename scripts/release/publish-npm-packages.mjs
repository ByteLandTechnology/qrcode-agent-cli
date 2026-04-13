import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, copyFileSync, chmodSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "../..");

const version = process.env.VERSION;
if (!version) {
  throw new Error("VERSION environment variable is required");
}

const artifactsDir = path.join(rootDir, ".work/release/artifacts");
const stagingDir = path.join(rootDir, ".work/npm-staging");

const TARGET_TO_PLATFORM = {
  "x86_64-unknown-linux-gnu": "linux-x64",
  "aarch64-unknown-linux-gnu": "linux-arm64",
  "aarch64-apple-darwin": "darwin-arm64",
  "x86_64-apple-darwin": "darwin-x64",
  "x86_64-pc-windows-msvc": "win32-x64",
  "aarch64-pc-windows-msvc": "win32-arm64",
};

const PLATFORMS = Object.values(TARGET_TO_PLATFORM);

function run(cmd, args, opts = {}) {
  execFileSync(cmd, args, {
    cwd: opts.cwd ?? rootDir,
    stdio: opts.stdio ?? "inherit",
    env: { ...process.env, ...opts.env },
  });
}

function ensureDir(dir) {
  run("mkdir", ["-p", dir]);
}

function writeJson(filePath, data) {
  ensureDir(path.dirname(filePath));
  writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

// --- Platform packages ---

function buildPlatformPackage(platform) {
  const [os, cpu] = platform.split("-");
  const pkgName = `qrcode-agent-cli-${platform}`;
  const pkgDir = path.join(stagingDir, pkgName);

  ensureDir(pkgDir);

  // Find the target that maps to this platform
  const target = Object.entries(TARGET_TO_PLATFORM).find(
    ([, p]) => p === platform,
  )[0];

  const isWin = os === "win32";
  const binName = isWin ? "qrcode-agent-cli.exe" : "qrcode-agent-cli";
  const srcBinary = path.join(artifactsDir, target, "binary", binName);

  if (!existsSync(srcBinary)) {
    throw new Error(`Binary not found for ${platform}: ${srcBinary}`);
  }

  copyFileSync(srcBinary, path.join(pkgDir, binName));
  if (!isWin) {
    chmodSync(path.join(pkgDir, binName), 0o755);
  }

  const osField = os === "win32" ? "win32" : os;
  const cpuField = cpu;

  writeJson(path.join(pkgDir, "package.json"), {
    name: pkgName,
    version,
    description: `${os === "darwin" ? "macOS" : os === "linux" ? "Linux" : "Windows"} ${cpu} binary for qrcode-agent-cli`,
    files: [binName],
    os: [osField],
    cpu: [cpuField],
    license: "MIT",
    repository: {
      type: "git",
      url: "git+https://github.com/ByteLandTechnology/qrcode-agent-cli.git",
    },
  });

  return pkgName;
}

// --- Main package ---

const RUNNER_SCRIPT = `#!/usr/bin/env node
const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const platform = process.platform;
const arch = process.arch;

const osMap = { darwin: "darwin", linux: "linux", win32: "win32" };
const archMap = { arm64: "arm64", x64: "x64" };

const pkgOs = osMap[platform];
const pkgArch = archMap[arch];

if (!pkgOs || !pkgArch) {
  console.error("qrcode-agent-cli: unsupported platform " + platform + "-" + arch);
  process.exit(1);
}

const pkgName = "qrcode-agent-cli-" + pkgOs + "-" + pkgArch;
const binName = platform === "win32" ? "qrcode-agent-cli.exe" : "qrcode-agent-cli";

let binDir;
try {
  binDir = path.join(require.resolve(pkgName + "/package.json"), "..");
} catch {
  binDir = path.join(__dirname, "..", pkgName);
}

const binPath = path.join(binDir, binName);

if (!fs.existsSync(binPath)) {
  console.error("qrcode-agent-cli: platform binary for " + pkgOs + "-" + pkgArch + " not found");
  console.error("Try: npm install " + pkgName);
  process.exit(1);
}

try {
  execFileSync(binPath, process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  process.exitCode = e.status ?? 1;
}
`;

function buildMainPackage() {
  const pkgDir = path.join(stagingDir, "qrcode-agent-cli");
  ensureDir(pkgDir);

  const optionalDeps = {};
  for (const platform of PLATFORMS) {
    optionalDeps[`qrcode-agent-cli-${platform}`] = version;
  }

  writeJson(path.join(pkgDir, "package.json"), {
    name: "qrcode-agent-cli",
    version,
    description: "Generate QR codes as terminal text or PNG images",
    bin: {
      "qrcode-agent-cli": "./run.js",
    },
    files: ["run.js"],
    optionalDependencies: optionalDeps,
    keywords: ["qrcode", "qr", "cli", "terminal", "png", "agent"],
    license: "MIT",
    repository: {
      type: "git",
      url: "git+https://github.com/ByteLandTechnology/qrcode-agent-cli.git",
    },
  });

  const runJsPath = path.join(pkgDir, "run.js");
  writeFileSync(runJsPath, RUNNER_SCRIPT, "utf8");
  chmodSync(runJsPath, 0o755);
}

// --- Publish ---

function versionExists(pkgName, ver) {
  try {
    execFileSync("npm", ["view", `${pkgName}@${ver}`, "version"], {
      cwd: rootDir,
      stdio: "pipe",
      encoding: "utf8",
    });
    return true;
  } catch {
    return false;
  }
}

function npmPublish(pkgDir) {
  const pkgJson = JSON.parse(
    readFileSync(path.join(pkgDir, "package.json"), "utf8"),
  );
  const { name, version: ver } = pkgJson;

  if (versionExists(name, ver)) {
    console.log(`  Skipping ${name}@${ver} (already published)`);
    return;
  }

  console.log(`Publishing ${name}@${ver}...`);
  run("npm", ["publish", "--access", "public", "--provenance"], {
    cwd: pkgDir,
    stdio: "pipe",
  });
  console.log(`  Published ${name}@${ver}`);
}

// --- Main ---

console.log(`Assembling npm packages v${version}...`);

// Build and publish platform packages first
for (const platform of PLATFORMS) {
  const pkgName = buildPlatformPackage(platform);
  const pkgDir = path.join(stagingDir, pkgName);
  npmPublish(pkgDir);
}

// Build and publish main package last
buildMainPackage();
npmPublish(path.join(stagingDir, "qrcode-agent-cli"));

console.log("All npm packages published successfully.");
