use crate::protocol::{
    ack_matches, verify_reply_checksum, M913_PID_WIRED, M913_PID_WIRELESS, M913_VID, PACKET_SIZE,
    Packet,
};
use hidapi::{HidApi, HidDevice};

pub struct M913Device {
    dev: HidDevice,
}

impl M913Device {
    /// List all HID interfaces for the M913 (for debugging/discovery).
    pub fn list_interfaces() -> Result<(), String> {
        let api = HidApi::new().map_err(|e| format!("HidApi::new failed: {e}"))?;
        let mut found = 0;
        for info in api.device_list() {
            if info.vendor_id() == M913_VID
                && (info.product_id() == M913_PID_WIRELESS || info.product_id() == M913_PID_WIRED)
            {
                found += 1;
                println!(
                    "  VID={:#06x} PID={:#06x} iface={} usage_page={:#06x} usage={:#06x} product={:?} path={:?}",
                    info.vendor_id(),
                    info.product_id(),
                    info.interface_number(),
                    info.usage_page(),
                    info.usage(),
                    info.product_string(),
                    info.path()
                );
            }
        }
        if found == 0 {
            println!("  (No M913 interfaces found for VID 0x25a7)");
        }
        Ok(())
    }

    /// Open the M913 vendor/config HID interface (Interface 1, preferring vendor usage page 0xFF00..0xFF04).
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("HidApi::new failed: {e}"))?;
        #[cfg(target_os = "macos")]
        api.set_open_exclusive(false);

        let candidates: Vec<_> = api
            .device_list()
            .filter(|d| {
                d.vendor_id() == M913_VID
                    && (d.product_id() == M913_PID_WIRELESS || d.product_id() == M913_PID_WIRED)
            })
            .collect();

        if candidates.is_empty() {
            return Err("Redragon M913 (25a7:fa07 / 25a7:fa08) not found. Is the receiver plugged in?".into());
        }

        // Prefer a vendor-defined usage page (>= 0xFF00) on interface 1 if macOS splits collections,
        // otherwise fall back to interface 1.
        let chosen = candidates
            .iter()
            .find(|d| d.usage_page() >= 0xFF00)
            .or_else(|| candidates.iter().find(|d| d.interface_number() == 1))
            .unwrap_or(&candidates[0]);

        eprintln!(
            "Opening M913: VID={:#06x} PID={:#06x} iface={} usage_page={:#06x} usage={:#06x}",
            chosen.vendor_id(),
            chosen.product_id(),
            chosen.interface_number(),
            chosen.usage_page(),
            chosen.usage()
        );

        let dev = chosen.open_device(&api).map_err(|e| {
            format!(
                "Failed to open HID device ({e}).\n\
                 On macOS, grant Input Monitoring permission to your terminal app in:\n\
                 System Settings → Privacy & Security → Input Monitoring"
            )
        })?;

        Ok(Self { dev })
    }

    /// Send a 17-byte Feature Report (Report ID 0x08) and wait for the matching 17-byte Input Report reply (0x09).
    pub fn transact(&self, req: &Packet) -> Result<Packet, String> {
        const ATTEMPTS: usize = 6;
        const SLICES: usize = 4;
        const SLICE_MS: i32 = 100;

        for _attempt in 0..ATTEMPTS {
            self.dev
                .send_feature_report(req)
                .map_err(|e| format!("send_feature_report failed: {e}"))?;

            let mut buf = [0u8; 64];
            for _slice in 0..SLICES {
                let n = self
                    .dev
                    .read_timeout(&mut buf, SLICE_MS)
                    .map_err(|e| format!("read_timeout failed: {e}"))?;
                if n < PACKET_SIZE {
                    continue;
                }
                let mut reply = [0u8; PACKET_SIZE];
                reply.copy_from_slice(&buf[..PACKET_SIZE]);
                if ack_matches(req, &reply) {
                    if !verify_reply_checksum(&reply) {
                        eprintln!("Warning: reply checksum mismatch!");
                    }
                    return Ok(reply);
                }
            }
        }

        Err("No reply from mouse (is the mouse awake? Try moving it slightly and retry).".into())
    }

    /// Print raw HID input reports arriving on Interface 1 (press Ctrl+C to stop).
    pub fn watch(&self) -> Result<(), String> {
        println!("Watching raw HID reports from Interface 1 (press buttons 11 & 12, Ctrl+C to stop)...");
        let mut buf = [0u8; 64];
        loop {
            let n = self
                .dev
                .read_timeout(&mut buf, 250)
                .map_err(|e| format!("read_timeout failed: {e}"))?;
            if n > 0 {
                let hex = buf[..n]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("  [{n:2} bytes] {hex}");
            }
        }
    }
}
