---
name: qrcode-agent-cli
description: "Generate QR codes as terminal text or PNG images"
---

# qrcode-agent-cli

## Description

`qrcode-agent-cli` generates QR codes from inline text content. It can print a QR code
directly in the terminal as Unicode blocks or save a PNG image file, while also
exposing structured help, runtime-path discovery, and a persisted Active
Context for default render settings.

## Prerequisites

- Rust and Cargo are installed when building from source
- The caller can execute the `qrcode-agent-cli` binary from a shell or an agent tool
- The caller has write access to the target output path when using image mode
- GitHub Release installs use `scripts/install-current-release.sh` from a
  checked-out release tag

## Invocation

Canonical installed invocation:

```bash
qrcode-agent-cli run "https://example.com" --render text
```

Local development invocation:

```bash
cargo run -- run "https://example.com" --render image --output ./example.png
```

Release-binary invocation after building:

```bash
./target/release/qrcode-agent-cli run "hello" --render text
```

GitHub Release installation after checking out a tagged release:

```bash
./scripts/install-current-release.sh 0.1.0
```

## Input

- `run` accepts inline text content as the positional `CONTENT` argument
- `run --render text|image` chooses terminal or PNG output
- `run --output <FILE>` selects the PNG destination in image mode
- `run --size <PIXELS>` sets the PNG size in image mode
- `run --format yaml|json|toml` chooses the structured response format for
  non-text responses and for all structured errors
- `context set` persists default `render`, `output`, and `size` values for
  later invocations

## Output

- `run --render text` writes the QR code itself to stdout as terminal text
- `run --render image` writes a PNG file and returns structured metadata on
  stdout
- `paths`, `context show`, `context set`, `context clear`, and `help` return
  structured YAML, JSON, or TOML on stdout
- `--help` always stays plain-text for people and shells
- Repo-native GitHub Release assets include `release-evidence.json` alongside
  platform archives for verified installs

## Errors

- Leaf-command validation failures return structured stderr in the selected
  format
- Structured errors include at least `code` and `message`
- Image-mode failures also describe the path or filesystem operation that
  failed when available

## Examples

```bash
qrcode-agent-cli run "https://openai.com" --render text
qrcode-agent-cli run "hello png" --render image --output ./hello.png --format json
qrcode-agent-cli help run --format yaml
qrcode-agent-cli paths --format toml
qrcode-agent-cli context set --render image --size 512 --format json
qrcode-agent-cli context show --format yaml
```
