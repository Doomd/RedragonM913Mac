# Changelog

## 0.1.0 — 2026-11-23
- Built Rust `hidapi` proof-of-concept CLI (`m913-mac`) for macOS over the 2.4GHz wireless receiver (`25a7:fa07`).
- Added `list`, `read`, `remap`, and `watch` subcommands.
- Solved macOS IOKit access (Input Monitoring + `macos-shared-device` non-exclusive open, no `sudo` required).
- Verified reading all 16 onboard button mappings and remapping `side11` → Option and `side12` → Control on real hardware.
- Added [docs/DEVLOG.md](docs/DEVLOG.md) and M913 reference images.

## 2026-11-23 (initial)
- Repo created with notes from troubleshooting and planning discussion. No code yet.
