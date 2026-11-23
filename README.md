# RedragonM913Mac

Native macOS configurator for the **Redragon M913 Impact Elite** MMO mouse (Areson `25a7:fa07` 2.4GHz wireless / `25a7:fa08` wired) — no Windows VM or background daemon required. Changes are written directly to the mouse's onboard memory.

Status: **v0.1.0 Proof of Concept working** (Rust CLI via `hidapi`; Tauri UI planned next).

## Quick Start (CLI)

### Prerequisites
- macOS with Rust (`cargo`) installed.
- Grant **Input Monitoring** permission to your terminal app in **System Settings → Privacy & Security → Input Monitoring** (required because the mouse's config HID interface also advertises a keyboard usage).

### Build & Run
```bash
cargo build

# List detected M913 HID interfaces
./target/debug/m913-mac list

# Read all 16 current onboard button mappings (wiggle mouse so it's awake)
./target/debug/m913-mac read

# Remap buttons (preserves all untouched buttons and commits to onboard flash)
./target/debug/m913-mac remap side11=option side12=control

# Revert side11/side12 back to factory defaults
./target/debug/m913-mac remap side11=numplus side12=numminus
```

## Notes
- Works directly over the **2.4GHz USB wireless receiver** (`VID: 0x25A7`, `PID: 0xFA07`) without `sudo`.
- USB-C to USB-C cables do not work with the M913 (missing CC resistors); wired mode requires USB-A to USB-C.
- Protocol reference: [Qehbr/m913-ctl](https://github.com/Qehbr/m913-ctl).

See [docs/DEVLOG.md](docs/DEVLOG.md) and [CHANGELOG.md](CHANGELOG.md).
