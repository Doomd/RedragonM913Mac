# RedragonM913Mac

Native macOS configurator for the **Redragon M913 Impact Elite** MMO mouse (Areson `25a7:fa07` 2.4GHz wireless / `25a7:fa08` wired) — built with **Tauri v2**, **SvelteKit (Svelte 5)**, **Tailwind CSS v4**, and Rust **`hidapi`**. Changes are written directly to the mouse's onboard memory (no Windows VM or background daemon required).

Status: **v0.2.0** — Interactive Tauri desktop app with Side & Top SVG button callouts.

## Quick Start

### Prerequisites
- macOS with **Bun** and **Rust** (`cargo`) installed.
- Grant **Input Monitoring** permission to your terminal app (when running `tauri dev`) or the built `.app` bundle in **System Settings → Privacy & Security → Input Monitoring**.

### Run in Development Mode
```bash
bun install
bun run tauri dev
```
*(Wiggle the mouse before clicking **Read from Mouse** or **Apply to Mouse** if it has gone to sleep.)*

### Build macOS App Bundle
```bash
bun run tauri build
```

## Notes
- Works directly over the **2.4GHz USB wireless receiver** (`VID: 0x25A7`, `PID: 0xFA07`) without `sudo`.
- USB-C to USB-C cables do not work with the M913 (missing CC resistors); wired mode requires USB-A to USB-C.
- The 16 programmable buttons are the **12 side buttons + Left, Right, Middle, and Fire**. The two central DPI buttons are hardcoded in firmware.
- Protocol reference: [Qehbr/m913-ctl](https://github.com/Qehbr/m913-ctl).

See [docs/DEVLOG.md](docs/DEVLOG.md) and [CHANGELOG.md](CHANGELOG.md).
