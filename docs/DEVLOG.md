# Development Log — RedragonM913Mac

## 2026-11-23 — Proof of Concept: Native macOS Read & Remap over 2.4GHz Wireless

### Hardware Discovery
- Confirmed the mouse receiver connected via `ioreg` is **VID `0x25A7`, PID `0xFA07`** (`2.4G Wireless Receiver`, original **Areson** hardware revision of the Redragon M913 Impact Elite).
- macOS exposes two HID interfaces on the dongle:
  - Interface 0: standard mouse pointer/buttons (`UsagePage 0x01, Usage 0x02`).
  - Interface 1: composite keyboard (`UsagePage 0x01, Usage 0x06`) + vendor-defined config pages (`0xFF01`–`0xFF04`), with `MaxFeatureReportSize = 17` and `MaxInputReportSize = 17`.

### Protocol Reference
- Found [Qehbr/m913-ctl](https://github.com/Qehbr/m913-ctl), which already reverse-engineered and verified the Areson `25a7:fa07` USB HID protocol:
  - **Host → Mouse:** 17-byte HID Feature Report (Report ID `0x08`) on Interface 1. Checksum: `(0x55 - sum(bytes[0..15])) & 0xFF`.
  - **Mouse → Host:** 17-byte HID Input Report replies (Report ID `0x09`) on Interface 1. Checksum: `(0x4C - sum(bytes[1..15])) & 0xFF`.
  - **Onboard memory map (Areson):**
    - `0x0060..0x009F`: 16 buttons × 4 bytes each (`05 00 00 50` points to the button's keyboard event list).
    - `0x0100 + btn_idx * 0x20`: per-button keyboard/modifier event list (`0x80`/`0x40` for modifiers, `0x81`/`0x41` for keys).
    - Commit packet (`08 04 ... 49`) persists changes to onboard flash.

### macOS HID Gotchas Solved
1. **No `libusb` needed:** Using Rust's `hidapi` crate talks directly through macOS IOKit (`IOHIDManager`) without clashing with `AppleUserUSBHostHIDDevice` or requiring `sudo`. This also gives us a reusable Rust backend for a future Tauri UI.
2. **Input Monitoring permission (`0xE00002E2`):** Because Interface 1 also advertises a keyboard usage (`0x01:0x06`), macOS requires granting **Input Monitoring** permission (*System Settings → Privacy & Security → Input Monitoring*) to the terminal/app opening it.
3. **Non-exclusive open (`0xE00002C1`):** By default, `hidapi` opens macOS HID devices exclusively (`kIOHIDOptionsTypeSeizeDevice`), which fails with `privilege violation` on keyboard interfaces without root. Enabling `macos-shared-device` / `api.set_open_exclusive(false)` opens it with `kIOHIDOptionsTypeNone` and works cleanly as a normal user.

### Verified on Real Hardware
- **Read-back (`./target/debug/m913-mac read`):** Successfully read all 16 buttons over the 2.4GHz dongle (factory defaults: `side1`–`side10` = `1`–`0`, `side11` = `numplus` (`0x57`), `side12` = `numminus` (`0x56`), `fire` = `04 14 03 3a` triple-click speed 20).
- **Remap (`./target/debug/m913-mac remap side11=option side12=control`):** Successfully remapped `side11` → macOS **Option** (Left Alt `0x04`) and `side12` → macOS **Control** (Left Ctrl `0x01`), preserving all other buttons and committing to onboard memory. Verified both modifiers work natively on macOS.

### Next Steps
- Expand action support (full keyboard keys, combos like `cmd+c`, media keys, `fire` button remapping, DPI/LED).
- Refactor `src/protocol.rs` and `src/device.rs` into a shared library crate (`lib.rs`).
- Build a simple Tauri desktop app on top of the Rust library (with `images/side.webp` and `images/top.webp` as visual button layout references).
