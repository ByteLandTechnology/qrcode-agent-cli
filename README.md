# qrcode-agent-cli

Generate QR codes as terminal text or PNG images.

## What It Does

`qrcode-agent-cli` is a small Rust CLI skill that turns inline text into a QR code.
It keeps the agent-facing workflow simple:

- terminal rendering for quick copy or visual inspection
- PNG rendering for files you want to keep or share
- structured help and structured errors for automation
- persisted default settings through a user-scoped Active Context

## Commands

```bash
qrcode-agent-cli run "https://example.com" --render text
qrcode-agent-cli run "https://example.com" --render image --output ./example.png --format json
qrcode-agent-cli paths --format yaml
qrcode-agent-cli context show --format json
qrcode-agent-cli context set --render image --size 512
qrcode-agent-cli help run --format toml
```

## Output Model

- `--help` is always plain-text
- `help` is the machine-readable help surface
- `run --render text` prints the QR code itself to stdout
- `run --render image` writes a PNG file and returns structured metadata
- structured commands support `yaml`, `json`, and `toml`

## Active Context

The Active Context file stores persisted defaults for:

- `default_render`
- `default_output`
- `default_image_size`

Explicit `run` flags override persisted defaults for the current invocation and
do not mutate the saved file.

## Runtime Directories

`qrcode-agent-cli paths` reports separate config, data, state, and cache locations.
For local testing you can override them with:

- `QRCODE_AGENT_CLI_CONFIG_DIR`
- `QRCODE_AGENT_CLI_DATA_DIR`
- `QRCODE_AGENT_CLI_STATE_DIR`
- `QRCODE_AGENT_CLI_CACHE_DIR`

## Build And Test

```bash
cargo build
cargo clippy -- -D warnings
cargo fmt --check
cargo test
```

## GitHub Release Installation

The repo-native release path is the authoritative install surface for this
project. After cloning the repository and checking out a released tag, install
the matching binary with:

```bash
git clone https://github.com/<owner>/<repo>.git
cd <repo>
git checkout v0.1.0
./scripts/install-current-release.sh 0.1.0
```

Each GitHub Release is expected to attach version-matched archives, checksum
files, and a `release-evidence.json` payload that records the exact commit and
artifact hashes used for that release.

## Development Notes

The canonical interface is the bare `qrcode-agent-cli` command. `cargo run -- ...`
is only the local-development convenience path, and
`./target/release/qrcode-agent-cli ...` is the direct release-binary path after a
local release build.
