//! Redragon M913 Impact Elite (Areson 25a7:fa07 / 25a7:fa08) HID protocol.
//! Reference: Qehbr/m913-ctl (reverse-engineered from USB captures).

pub const M913_VID: u16 = 0x25a7;
pub const M913_PID_WIRELESS: u16 = 0xfa07;
pub const M913_PID_WIRED: u16 = 0xfa08;

pub const PACKET_SIZE: usize = 17;
pub const REPORT_ID_OUT: u8 = 0x08;
pub const REPORT_ID_IN: u8 = 0x09;

pub const ADDR_BTN_BASE: u16 = 0x0060;
pub const BTN_STRIDE: u16 = 4;
pub const ADDR_KB_BASE: u16 = 0x0100;
pub const KB_STRIDE: u16 = 0x0020;

/// 4-byte marker in the button table meaning "look up the keyboard event list at 0x0100 + idx*0x20".
pub const KB_LIST_MARKER: [u8; 4] = [0x05, 0x00, 0x00, 0x50];

pub type Packet = [u8; PACKET_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Side1 = 0,
    Side2 = 1,
    Side3 = 2,
    Side4 = 3,
    Side5 = 4,
    Side6 = 5,
    Right = 6,
    Left = 7,
    Side7 = 8,
    Side8 = 9,
    Middle = 10,
    Fire = 11,
    Side9 = 12,
    Side10 = 13,
    Side11 = 14,
    Side12 = 15,
}

pub const ALL_BUTTONS: [Button; 16] = [
    Button::Side1,
    Button::Side2,
    Button::Side3,
    Button::Side4,
    Button::Side5,
    Button::Side6,
    Button::Right,
    Button::Left,
    Button::Side7,
    Button::Side8,
    Button::Middle,
    Button::Fire,
    Button::Side9,
    Button::Side10,
    Button::Side11,
    Button::Side12,
];

impl Button {
    pub fn name(self) -> &'static str {
        match self {
            Button::Side1 => "side1",
            Button::Side2 => "side2",
            Button::Side3 => "side3",
            Button::Side4 => "side4",
            Button::Side5 => "side5",
            Button::Side6 => "side6",
            Button::Right => "right",
            Button::Left => "left",
            Button::Side7 => "side7",
            Button::Side8 => "side8",
            Button::Middle => "middle",
            Button::Fire => "fire",
            Button::Side9 => "side9",
            Button::Side10 => "side10",
            Button::Side11 => "side11",
            Button::Side12 => "side12",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "side1" | "1" => Some(Button::Side1),
            "side2" | "2" => Some(Button::Side2),
            "side3" | "3" => Some(Button::Side3),
            "side4" | "4" => Some(Button::Side4),
            "side5" | "5" => Some(Button::Side5),
            "side6" | "6" => Some(Button::Side6),
            "right" => Some(Button::Right),
            "left" => Some(Button::Left),
            "side7" | "7" => Some(Button::Side7),
            "side8" | "8" => Some(Button::Side8),
            "middle" => Some(Button::Middle),
            "fire" => Some(Button::Fire),
            "side9" | "9" => Some(Button::Side9),
            "side10" | "10" => Some(Button::Side10),
            "side11" | "11" => Some(Button::Side11),
            "side12" | "12" => Some(Button::Side12),
            _ => None,
        }
    }

    pub fn kb_addr(self) -> u16 {
        ADDR_KB_BASE + (self as u16) * KB_STRIDE
    }
}

/// Host → device checksum: `(0x55 - sum(bytes[0..15])) & 0xFF`.
pub fn compute_checksum(pkt: &Packet) -> u8 {
    let sum: u16 = pkt[..PACKET_SIZE - 1].iter().map(|&b| b as u16).sum();
    (0x55u16.wrapping_sub(sum) & 0xFF) as u8
}

/// Device → host checksum: `(0x4C - sum(bytes[1..15])) & 0xFF` (excludes Report ID 0x09).
pub fn verify_reply_checksum(pkt: &Packet) -> bool {
    let sum: u16 = pkt[1..PACKET_SIZE - 1].iter().map(|&b| b as u16).sum();
    pkt[PACKET_SIZE - 1] == (0x4Cu16.wrapping_sub(sum) & 0xFF) as u8
}

pub fn ack_matches(req: &Packet, reply: &Packet) -> bool {
    reply[0] == REPORT_ID_IN
        && reply[1] == req[1]
        && reply[3] == req[3]
        && reply[4] == req[4]
}

/// Build a status ping packet (`08 03 ...`).
pub fn build_status_packet() -> Packet {
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x03;
    p[16] = compute_checksum(&p);
    p
}

/// Build a 10-byte EEPROM read request at `addr`.
pub fn build_read_packet(addr: u16) -> Packet {
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x08;
    p[3] = (addr >> 8) as u8;
    p[4] = (addr & 0xFF) as u8;
    p[5] = 0x0a;
    p[16] = compute_checksum(&p);
    p
}

/// Build a write packet (`08 07 00 hi lo len data... cksum`).
pub fn build_write_packet(addr: u16, data: &[u8]) -> Packet {
    assert!(data.len() <= 10, "write payload max 10 bytes");
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x07;
    p[3] = (addr >> 8) as u8;
    p[4] = (addr & 0xFF) as u8;
    p[5] = data.len() as u8;
    p[6..6 + data.len()].copy_from_slice(data);
    p[16] = compute_checksum(&p);
    p
}

/// Build the commit packet that saves written settings to onboard flash.
pub fn build_commit_packet() -> Packet {
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x04;
    p[16] = compute_checksum(&p); // 0x49
    p
}

/// Build a single-modifier keyboard sub-packet (e.g. Option/Left Alt = 0x04, Left Ctrl = 0x01).
pub fn build_modifier_kb_packet(btn: Button, mod_bit: u8) -> Packet {
    let addr = btn.kb_addr();
    let isum: u8 = 0x55u8
        .wrapping_sub(0x02)
        .wrapping_sub(0x80)
        .wrapping_sub(mod_bit)
        .wrapping_sub(0x40)
        .wrapping_sub(mod_bit);
    let payload = [0x02, 0x80, mod_bit, 0x00, 0x40, mod_bit, 0x00, isum];
    build_write_packet(addr, &payload)
}

/// Build a plain single-key keyboard sub-packet (USB HID usage code).
pub fn build_key_kb_packet(btn: Button, hid_key: u8) -> Packet {
    let addr = btn.kb_addr();
    let isum: u8 = 0x55u8
        .wrapping_sub(0x02)
        .wrapping_sub(0x81)
        .wrapping_sub(hid_key)
        .wrapping_sub(0x41)
        .wrapping_sub(hid_key);
    let payload = [0x02, 0x81, hid_key, 0x00, 0x41, hid_key, 0x00, isum];
    build_write_packet(addr, &payload)
}

pub fn decode_mouse_action(ab: [u8; 4]) -> Option<String> {
    let s = match ab {
        [0x01, 0x01, 0x00, 0x53] => "left",
        [0x01, 0x02, 0x00, 0x52] => "right",
        [0x01, 0x04, 0x00, 0x50] => "middle",
        [0x01, 0x08, 0x00, 0x4c] => "backward",
        [0x01, 0x10, 0x00, 0x44] => "forward",
        [0x02, 0x01, 0x00, 0x52] => "dpi-cycle",
        [0x02, 0x02, 0x00, 0x51] => "dpi+",
        [0x02, 0x03, 0x00, 0x50] => "dpi-",
        [0x07, 0x00, 0x00, 0x4e] => "polling_switch",
        [0x08, 0x00, 0x00, 0x4d] => "led_toggle",
        [0x00, 0x00, 0x00, 0x55] => "none",
        [0x04, speed, times, _] => return Some(format!("fire (speed={speed}, times={times})")),
        _ => return None,
    };
    Some(s.to_string())
}

pub fn decode_modifier_bit(bit: u8) -> &'static str {
    match bit {
        0x01 => "ctrl (Left Control)",
        0x02 => "shift (Left Shift)",
        0x04 => "option/alt (Left Option)",
        0x08 => "command (Left Command)",
        0x10 => "ctrl_r (Right Control)",
        0x20 => "shift_r (Right Shift)",
        0x40 => "option_r (Right Option)",
        0x80 => "command_r (Right Command)",
        _ => "unknown_mod",
    }
}

pub fn decode_hid_key(code: u8) -> String {
    match code {
        0x04..=0x1d => ((b'a' + (code - 0x04)) as char).to_string(),
        0x1e..=0x26 => ((b'1' + (code - 0x1e)) as char).to_string(),
        0x27 => "0".into(),
        0x28 => "enter".into(),
        0x29 => "esc".into(),
        0x2a => "backspace".into(),
        0x2b => "tab".into(),
        0x2c => "space".into(),
        0x2d => "- (minus)".into(),
        0x2e => "= (equal)".into(),
        0x2f => "[".into(),
        0x30 => "]".into(),
        0x31 => "\\".into(),
        0x33 => ";".into(),
        0x34 => "'".into(),
        0x35 => "`".into(),
        0x36 => ",".into(),
        0x37 => ".".into(),
        0x38 => "/".into(),
        0x3a..=0x45 => format!("f{}", code - 0x3a + 1),
        0x4f => "arrow_right".into(),
        0x50 => "arrow_left".into(),
        0x51 => "arrow_down".into(),
        0x52 => "arrow_up".into(),
        0x56 => "numminus".into(),
        0x57 => "numplus".into(),
        _ => format!("hid:0x{code:02x}"),
    }
}

/// Decode a keyboard event list (up to 20 bytes read from `0x0100 + idx*0x20`).
pub fn decode_kb_list(buf: &[u8]) -> Option<String> {
    if buf.len() < 2 {
        return None;
    }
    let count = buf[0] as usize;
    if count == 0 || 1 + count * 3 + 1 > buf.len() {
        return None;
    }
    let sum: u16 = buf[..1 + count * 3].iter().map(|&b| b as u16).sum();
    let expected_inner = (0x55u16.wrapping_sub(sum) & 0xFF) as u8;
    if buf[1 + count * 3] != expected_inner {
        return None;
    }

    let mut parts = Vec::new();
    for i in 0..count {
        let ev_type = buf[1 + i * 3];
        let val = buf[2 + i * 3];
        match ev_type {
            0x80 => {
                for bit_idx in 0..8 {
                    let bit = 1u8 << bit_idx;
                    if (val & bit) != 0 {
                        parts.push(decode_modifier_bit(bit).to_string());
                    }
                }
            }
            0x81 => parts.push(decode_hid_key(val)),
            0x82 => parts.push(format!("media:0x{val:02x}")),
            0x40 | 0x41 | 0x42 => {} // key/mod up
            _ => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("+"))
    }
}

pub fn hex_packet(pkt: &[u8]) -> String {
    pkt.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}
