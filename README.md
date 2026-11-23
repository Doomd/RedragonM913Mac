# RedragonM913Mac

Goal: configure the Redragon M913 Impact Elite MMO mouse (button mappings, including the left-side "fire" triple-click button) natively on macOS, without Redragon's Windows-only software.

Status: **parked idea** (23 Nov 2026). Not a priority yet; notes only.

## What we know
- Mouse used wireless via the 2.4GHz USB-A dongle plus a USB-C to A adapter.
- C-to-C cable (even a good iPhone data cable) does nothing: the mouse's USB-C port lacks the CC resistors, so it only works with A-to-C. The Mac showed no USB device at all on C-to-C.
- The fire button (left of left-click) sends triple-click from **firmware**; macOS only sees ordinary left clicks, so no Mac remapper can change it. It must be reprogrammed into the mouse's onboard memory.
- Side buttons mostly send keys 1 to 0 by default; Karabiner-Elements or Mac Mouse Fix can remap those on macOS now.
- Wine/CrossOver can't pass USB through, so Redragon's software likely won't see the mouse.
- A Windows 11 Arm VM in UTM with USB passthrough would work as a one-off: set mappings once, they persist on the mouse.

## Plan (when picked back up)
1. Search for existing M913 protocol work (GitHub, forums, OpenRGB/libratbag-style projects for Redragon).
2. Static route: download Redragon's M913 installer, unpack and decompile it on the Mac (often .NET, so readable), and find the HID feature/output reports it sends.
3. Live route: enumerate the dongle's HID interfaces on macOS (hidapi/ioreg), then carefully send test reports. Small risk of scrambling settings; a factory reset usually fixes it.
4. Optional shortcut: record USB traffic from Redragon's software in a UTM Windows VM.
5. Build a small CLI (Python + hidapi likely) to read and write button mappings, especially the fire button.

See CHANGELOG.md.
