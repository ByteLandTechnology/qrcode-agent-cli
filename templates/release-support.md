# Release Support Assets

This repository keeps repo-native GitHub Release support files under:

- `.github/workflows/release.yml`
- `release/skill-release.config.json`
- `scripts/release/`
- `scripts/install-current-release.sh`

The release workflow publishes archive assets plus `release-evidence.json` for
each tagged GitHub Release. Optional npm distribution remains intentionally out
of scope for this repository.
