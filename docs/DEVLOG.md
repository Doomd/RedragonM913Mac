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

## 2026-11-23 (v0.2.0) — Tauri v2 + SvelteKit + Tailwind CSS v4 App

### What Was Built
- Cropped clean copies of the reference mouse images into `static/images/side.webp` (`1346×600`) and `static/images/top.webp` (`900×1160`, USB cable & dongle cropped out).
- Scaffolded a **Tauri v2** desktop app with **Svelte 5 / SvelteKit** (`@sveltejs/adapter-static` SPA mode), **Tailwind CSS v4** (`@tailwindcss/vite`), and **Bun** (`bun.lock`).
- Moved the M913 HID protocol and macOS `hidapi` driver into `src-tauri/src/m913/` and exposed Tauri commands:
  - `get_factory_defaults`, `read_mappings`, `apply_mappings`, `reset_all_defaults`, `open_repo`.
- Built an interactive **SVG vector circle callout** UI overlaid on both the Side View (12 buttons) and Top View (4 buttons):
  - **White circle** = factory default; **Green circle** = remapped from default (yellow ring for unsaved staged edits).
  - Clicking any button opens the inspector panel with quick macOS system modifiers (`⌥ Option`, `⌃ Control`, `⌘ Command`, `⇧ Shift`), **Capture Key Press**, full key/mouse action dropdown, and single-button **Reset to Default**.
  - Top toolbar has **Read from Mouse**, **Factory Reset All**, **Apply to Mouse**, and a **GitHub** repo link; Top View includes a footnote explaining that the two DPI buttons are firmware-fixed.

### Next Steps
- **Profiles (v0.3):** Save/load named collections of button mappings as local JSON profiles, with a built-in Factory Default profile.
- **DPI buttons note (pinned for later):** The two physical DPI+/DPI- buttons behind the scroll wheel are hardcoded in firmware (not in the 16-button remap table; even Redragon's Windows software cannot remap them). **Workaround:** add a **"Lock DPI"** option that sets active DPI stage count to 1 (or sets all 5 DPI slots to the same preferred DPI) so accidental presses don't change cursor speed.
- **Extra features:** Key combos (e.g. `cmd+c`), media keys, and LED control.
- **Public docs & onboarding:** Position clearly as a macOS-native tool (not tested on Windows), add screenshots/usage guide, add a `LICENSE`, and include an **AI/LLM-friendly install prompt** (or `llms.txt`) so users can hand setup directly to their coding agent.
- **macOS Distribution:** Consider a paid Apple Developer account (\$99/yr) to code-sign (**Developer ID Application**) and **notarize** `.app` / `.dmg` builds via Tauri + GitHub Actions Releases (eliminates Gatekeeper warnings and preserves macOS Input Monitoring permission across updates).
