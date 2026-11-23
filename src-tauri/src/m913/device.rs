use std::collections::BTreeMap;

use hidapi::{HidApi, HidDevice};

use crate::m913::protocol::{
    ack_matches, build_commit_packet, build_key_kb_packet, build_modifier_kb_packet,
    build_read_packet, build_status_packet, build_write_packet, decode_kb_list,
    decode_mouse_action, parse_action, verify_reply_checksum, Button, ButtonState, EncodedAction,
    ALL_BUTTONS, ADDR_BTN_BASE, BTN_STRIDE, KB_LIST_MARKER, M913_PID_WIRED, M913_PID_WIRELESS,
    M913_VID, PACKET_SIZE, Packet,
};

pub struct M913Device {
    dev: HidDevice,
}

impl M913Device {
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
            return Err(
                "Redragon M913 (25a7:fa07 / 25a7:fa08) not found. Is the 2.4GHz receiver plugged in?"
                    .into(),
            );
        }

        let chosen = candidates
            .iter()
            .find(|d| d.usage_page() >= 0xFF00)
            .or_else(|| candidates.iter().find(|d| d.interface_number() == 1))
            .unwrap_or(&candidates[0]);

        let dev = chosen.open_device(&api).map_err(|e| {
            format!(
                "Failed to open M913 HID device ({e}). Grant Input Monitoring permission in System Settings → Privacy & Security → Input Monitoring."
            )
        })?;

        Ok(Self { dev })
    }

    pub fn transact(&self, req: &Packet) -> Result<Packet, String> {
        const ATTEMPTS: usize = 6;
        const SLICES: usize = 4;
        const SLICE_MS: i32 = 100;

        for _ in 0..ATTEMPTS {
            self.dev
                .send_feature_report(req)
                .map_err(|e| format!("send_feature_report failed: {e}"))?;

            let mut buf = [0u8; 64];
            for _ in 0..SLICES {
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
                    let _ = verify_reply_checksum(&reply);
                    return Ok(reply);
                }
            }
        }

        Err("No reply from mouse — wiggle the mouse to wake it up and try again.".into())
    }

    fn read_memory(&self, addrs: &[u16]) -> Result<BTreeMap<u16, u8>, String> {
        let mut mem = BTreeMap::new();
        for &addr in addrs {
            let req = build_read_packet(addr);
            let reply = self.transact(&req)?;
            for i in 0..10 {
                mem.insert(addr + i as u16, reply[6 + i]);
            }
        }
        Ok(mem)
    }

    pub fn read_all_buttons(&self) -> Result<Vec<ButtonState>, String> {
        let status_req = build_status_packet();
        let _ = self.transact(&status_req)?;

        let btn_addrs = [0x0060, 0x006a, 0x0074, 0x007e, 0x0088, 0x0092, 0x009c];
        let mut mem = self.read_memory(&btn_addrs)?;

        let mut kb_addrs = Vec::new();
        for btn in ALL_BUTTONS {
            let base = ADDR_BTN_BASE + (btn as u16) * BTN_STRIDE;
            let ab = [mem[&base], mem[&(base + 1)], mem[&(base + 2)], mem[&(base + 3)]];
            if ab == KB_LIST_MARKER {
                let kb_base = btn.kb_addr();
                kb_addrs.push(kb_base);
                kb_addrs.push(kb_base + 0x0a);
            }
        }
        if !kb_addrs.is_empty() {
            let kb_mem = self.read_memory(&kb_addrs)?;
            mem.extend(kb_mem);
        }

        let mut states = Vec::with_capacity(16);
        for btn in ALL_BUTTONS {
            let base = ADDR_BTN_BASE + (btn as u16) * BTN_STRIDE;
            let ab = [mem[&base], mem[&(base + 1)], mem[&(base + 2)], mem[&(base + 3)]];
            let action = if ab == KB_LIST_MARKER {
                let kb_base = btn.kb_addr();
                let kb_bytes: Vec<u8> = (0..20).map(|i| mem[&(kb_base + i)]).collect();
                decode_kb_list(&kb_bytes).unwrap_or_else(|| "unknown".into())
            } else {
                decode_mouse_action(ab).unwrap_or_else(|| "unknown".into())
            };

            let default_action = btn.factory_default().to_string();
            let is_modified = action != default_action;
            states.push(ButtonState {
                id: btn.id().to_string(),
                label: btn.label().to_string(),
                action,
                default_action,
                is_modified,
            });
        }

        Ok(states)
    }

    pub fn apply_mappings(&self, updates: &[(Button, String)]) -> Result<Vec<ButtonState>, String> {
        let btn_addrs = [0x0060, 0x006a, 0x0074, 0x007e, 0x0088, 0x0092, 0x009c];
        let mem = self.read_memory(&btn_addrs)?;

        let mut table = [[0u8; 4]; 16];
        for (idx, slot) in table.iter_mut().enumerate() {
            let base = ADDR_BTN_BASE + (idx as u16) * BTN_STRIDE;
            *slot = [mem[&base], mem[&(base + 1)], mem[&(base + 2)], mem[&(base + 3)]];
        }

        for (btn, spec) in updates {
            let encoded = parse_action(spec)?;
            let idx = *btn as usize;
            match encoded {
                EncodedAction::Modifier(mod_bit) => {
                    let pkt = build_modifier_kb_packet(*btn, mod_bit);
                    self.transact(&pkt)?;
                    table[idx] = KB_LIST_MARKER;
                }
                EncodedAction::Key(hid_key) => {
                    let pkt = build_key_kb_packet(*btn, hid_key);
                    self.transact(&pkt)?;
                    table[idx] = KB_LIST_MARKER;
                }
                EncodedAction::Raw(ab) => {
                    table[idx] = ab;
                }
            }
        }

        for pair_idx in 0..8 {
            let addr = ADDR_BTN_BASE + (pair_idx as u16) * 8;
            let mut payload = [0u8; 8];
            payload[0..4].copy_from_slice(&table[pair_idx * 2]);
            payload[4..8].copy_from_slice(&table[pair_idx * 2 + 1]);
            let pkt = build_write_packet(addr, &payload);
            self.transact(&pkt)?;
        }

        let commit = build_commit_packet();
        self.transact(&commit)?;

        self.read_all_buttons()
    }
}
